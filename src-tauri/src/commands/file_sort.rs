use chrono::{Datelike, Local};
use serde::Serialize;
use std::{collections::HashSet, fs::{self, OpenOptions}, io, path::{Path, PathBuf}};
use tauri::{Emitter, State};
use tauri_plugin_dialog::DialogExt;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SortResult { source: String, status: String, destination: Option<String>, message: String }

#[tauri::command]
pub async fn choose_sort_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().blocking_pick_folder().and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().into_owned())
    }).await.map_err(|e| e.to_string())
}

fn category(path: &Path) -> &'static [&'static str] {
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "heic" | "heif" | "svg" | "ico" | "raw" => &["图片"],
        "mp4" | "mov" | "mkv" | "avi" | "wmv" | "flv" | "webm" | "m4v" | "mpeg" | "mpg" | "3gp" => &["视频"],
        "doc" | "docx" | "docm" | "dot" | "dotx" | "odt" | "rtf" => &["文档", "Word"],
        "ppt" | "pptx" | "pptm" | "pps" | "ppsx" | "odp" => &["文档", "PPT"],
        "xls" | "xlsx" | "xlsm" | "xlsb" | "csv" | "ods" => &["文档", "Excel"],
        "pdf" | "txt" | "md" | "epub" | "html" | "htm" => &["文档", "其他文档"],
        _ => &["其他"],
    }
}

fn move_file(source: &Path, directory: &Path) -> io::Result<PathBuf> {
    let mut input = fs::File::open(source)?;
    let original = input.metadata()?;
    let expected = original.len();
    let modified = original.modified().ok();
    fs::create_dir_all(directory)?;
    let name = source.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "无效文件名"))?;
    let stem = source.file_stem().unwrap_or(name).to_string_lossy();
    let ext = source.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    for number in 0..100_000 {
        let candidate = if number == 0 { directory.join(name) } else { directory.join(format!("{stem} ({number}){ext}")) };
        let mut output = match OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let copied = io::copy(&mut input, &mut output).and_then(|n| {
            let current = input.metadata()?;
            if n != expected || current.len() != expected || (modified.is_some() && current.modified().ok() != modified) { return Err(io::Error::new(io::ErrorKind::Other, "文件在复制期间发生变化")); }
            output.sync_all()
        });
        drop(output);
        if let Err(error) = copied { let _ = fs::remove_file(&candidate); return Err(error); }
        drop(input);
        if let Err(error) = fs::remove_file(source) {
            let _ = fs::remove_file(&candidate);
            return Err(error);
        }
        return Ok(candidate);
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "同名文件过多"))
}

fn find_dropped_directories(paths: &[String]) -> Result<Vec<String>, String> {
    let mut folders = Vec::new();
    for path in paths {
        let metadata = fs::symlink_metadata(path).map_err(|error| format!("无法检查 {path}：{error}"))?;
        if metadata.file_type().is_dir() { folders.push(path.clone()); }
    }
    Ok(folders)
}

// Fail closed if any part of a dropped folder cannot be inspected. Links are
// not traversed because the classifier also skips them.
fn reject_program_folders(folders: &[String]) -> Result<(), String> {
    let mut pending: Vec<PathBuf> = folders.iter().map(PathBuf::from).collect();
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("无法检查文件夹 {}：{error}，已取消分类", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("无法检查文件夹 {}：{error}，已取消分类", directory.display()))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("无法检查 {}：{error}，已取消分类", path.display()))?;
            let executable = path.extension().and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"));
            if executable && (metadata.is_file() || metadata.file_type().is_symlink()) {
                return Err(format!("文件夹中发现 EXE 文件（{}），可能是程序文件夹，已拒绝分类", path.display()));
            }
            if metadata.is_dir() && !metadata.file_type().is_symlink() { pending.push(path); }
        }
    }
    Ok(())
}

fn require_folder_confirmation(paths: &[String], confirmed_folders: &[String]) -> Result<(), String> {
    let folders = find_dropped_directories(paths)?;
    reject_program_folders(&folders)?;
    if folders.iter().any(|folder| !confirmed_folders.contains(folder)) {
        return Err("文件夹需要确认后才能分类".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn inspect_sort_inputs(paths: Vec<String>) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let folders = find_dropped_directories(&paths)?;
        reject_program_folders(&folders)?;
        Ok(folders)
    })
        .await.map_err(|error| error.to_string())?
}

// Scan before moving files, excluding the destination and deduplicating paths.
enum SortInput { File(PathBuf), Issue(SortResult) }

fn collect_inputs(paths: Vec<String>, destination: &Path) -> Vec<SortInput> {
    let destination = fs::canonicalize(destination).ok();
    let mut seen = HashSet::new();
    let mut stack: Vec<PathBuf> = paths.into_iter().rev().map(PathBuf::from).collect();
    let mut inputs = Vec::new();
    while let Some(path) = stack.pop() {
        let source = path.to_string_lossy().into_owned();
        let issue = |status: &str, message: String| SortInput::Issue(SortResult {
            source: source.clone(), status: status.into(), destination: None, message,
        });
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => { inputs.push(issue("failed", error.to_string())); continue; }
        };
        if metadata.file_type().is_symlink() {
            inputs.push(issue("skipped", "跳过符号链接".into())); continue;
        }
        let identity = match fs::canonicalize(&path) {
            Ok(identity) => identity,
            Err(error) => { inputs.push(issue("failed", error.to_string())); continue; }
        };
        if !seen.insert(identity.clone()) { continue; }
        if metadata.is_file() { inputs.push(SortInput::File(path)); }
        else if metadata.is_dir() {
            if destination.as_ref().is_some_and(|root| identity.starts_with(root)) {
                inputs.push(issue("skipped", "跳过目标文件夹，避免重复分类".into())); continue;
            }
            match fs::read_dir(&path) {
                Ok(entries) => {
                    let mut children = Vec::new();
                    let mut read_failed = false;
                    for entry in entries { match entry {
                        Ok(entry) => children.push(entry.path()),
                        Err(error) => { read_failed = true; inputs.push(issue("failed", format!("读取文件夹内容失败：{error}"))); },
                    }}
                    if children.is_empty() && !read_failed { inputs.push(issue("skipped", "空文件夹".into())); }
                    children.sort(); stack.extend(children.into_iter().rev());
                }
                Err(error) => inputs.push(issue("failed", format!("读取文件夹失败：{error}"))),
            }
        } else { inputs.push(issue("skipped", "跳过非普通文件".into())); }
    }
    inputs
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SortProgress { batch_id: String, completed: usize, total: usize }

#[tauri::command]
pub async fn sort_dropped_files(paths: Vec<String>, batch_id: String, confirmed_folders: Vec<String>, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<Vec<SortResult>, String> {
    let (root, date_mode) = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        (settings.file_sort_directory.clone().filter(|s| !s.trim().is_empty()).ok_or("请先选择目标文件夹")?, settings.file_sort_date.clone())
    };
    tauri::async_runtime::spawn_blocking(move || {
        require_folder_confirmation(&paths, &confirmed_folders)?;
        let now = Local::now();
        let inputs = collect_inputs(paths, Path::new(&root));
        // Catch files appearing between the safety scan and batch collection.
        for input in &inputs {
            if let SortInput::File(path) = input {
                if confirmed_folders.iter().any(|folder| path.starts_with(Path::new(folder)))
                    && path.extension().and_then(|extension| extension.to_str()).is_some_and(|extension| extension.eq_ignore_ascii_case("exe")) {
                    return Err(format!("文件夹中发现 EXE 文件（{}），可能是程序文件夹，已拒绝分类", path.display()));
                }
            }
        }
        let total = inputs.len();
        let _ = app.emit_to("main", "file-sort-progress", SortProgress { batch_id: batch_id.clone(), completed: 0, total });
        Ok(inputs.into_iter().enumerate().map(|(index, input)| {
            let result = match input {
              SortInput::Issue(result) => result,
              SortInput::File(path) => (|| {
            let source = path.to_string_lossy().into_owned();
            let path = path.as_path();
            let mut result = SortResult { source: source.clone(), status: "failed".into(), destination: None, message: String::new() };
            let metadata = match fs::symlink_metadata(path) { Ok(value) => value, Err(e) => { result.message = e.to_string(); return result; } };
            if !metadata.file_type().is_file() { result.status = "skipped".into(); result.message = "跳过文件夹或非普通文件".into(); return result; }
            let root_path = Path::new(&root);
            if !root_path.is_dir() { result.message = "目标文件夹不存在或不可访问".into(); return result; }
            let mut destination = root_path.to_path_buf();
            for part in category(path) { destination.push(part); }
            if date_mode == "year" || date_mode == "month" { destination.push(now.year().to_string()); }
            if date_mode == "month" { destination.push(format!("{}月", now.month())); }
            match move_file(path, &destination) {
                Ok(target) => { result.status = "moved".into(); result.destination = Some(target.to_string_lossy().into_owned()); },
                Err(e) => result.message = e.to_string(),
            }
            result
            })(),
            };
            let _ = app.emit_to("main", "file-sort-progress", SortProgress {
                batch_id: batch_id.clone(), completed: index + 1, total,
            });
            result
        }).collect())
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn classifies_extensions() {
        assert_eq!(category(Path::new("a.PPTX")), &["文档", "PPT"]);
        assert_eq!(category(Path::new("a.xlsx")), &["文档", "Excel"]);
        assert_eq!(category(Path::new("a.docx")), &["文档", "Word"]);
        assert_eq!(category(Path::new("a.png")), &["图片"]);
        assert_eq!(category(Path::new("a.mp4")), &["视频"]);
        assert_eq!(category(Path::new("a.unknown")), &["其他"]);
    }
    #[test] fn rejects_nested_executable_before_any_file_is_moved() {
        let root = std::env::temp_dir().join(format!("isle-exe-{}-{}", std::process::id(), Local::now().timestamp_nanos_opt().unwrap()));
        let nested = root.join("nested"); fs::create_dir_all(&nested).unwrap();
        let doc = root.join("report.docx"); let exe = nested.join("Program.EXE");
        fs::write(&doc, b"document").unwrap(); fs::write(&exe, b"program").unwrap();
        let folder = root.to_string_lossy().into_owned();
        let error = reject_program_folders(&[folder.clone()]).unwrap_err();
        assert!(error.contains("EXE"));
        assert!(require_folder_confirmation(&[folder.clone()], &[folder]).is_err());
        assert_eq!(fs::read(&doc).unwrap(), b"document"); assert!(exe.exists());
        fs::remove_file(&exe).unwrap();
        assert!(reject_program_folders(&[root.to_string_lossy().into_owned()]).is_ok());
        fs::remove_dir_all(root).unwrap();
    }
    #[test] fn requires_confirmation_for_every_dropped_folder() {
        let root = std::env::temp_dir().join(format!("isle-confirm-{}-{}", std::process::id(), Local::now().timestamp_nanos_opt().unwrap()));
        let folder = root.join("folder"); fs::create_dir_all(&folder).unwrap();
        let file = root.join("file.txt"); fs::write(&file, b"keep").unwrap();
        let file_path = file.to_string_lossy().into_owned(); let folder_path = folder.to_string_lossy().into_owned();
        assert!(require_folder_confirmation(&[file_path.clone()], &[]).is_ok());
        assert!(require_folder_confirmation(&[file_path.clone(), folder_path.clone()], &[]).is_err());
        assert_eq!(fs::read(&file).unwrap(), b"keep");
        assert!(require_folder_confirmation(&[file_path, folder_path.clone()], &[folder_path]).is_ok());
        fs::remove_dir_all(root).unwrap();
    }
    #[test] fn traverses_nested_files_deduplicates_and_excludes_destination() {
        let root = std::env::temp_dir().join(format!("isle-scan-{}-{}", std::process::id(), Local::now().timestamp_nanos_opt().unwrap()));
        let nested = root.join("nested"); let target = root.join("target");
        fs::create_dir_all(&nested).unwrap(); fs::create_dir_all(&target).unwrap();
        let file = nested.join("a.txt"); fs::write(&file, b"new").unwrap(); fs::write(target.join("existing.txt"), b"existing").unwrap();
        let inputs = collect_inputs(vec![root.to_string_lossy().into_owned(), file.to_string_lossy().into_owned()], &target);
        assert_eq!(inputs.iter().filter(|item| matches!(item, SortInput::File(_))).count(), 1);
        assert!(inputs.iter().any(|item| matches!(item, SortInput::Issue(result) if result.message.contains("目标文件夹"))));
        assert!(file.exists()); assert!(target.join("existing.txt").exists());
        for input in inputs {
            if let SortInput::File(path) = input {
                let mut directory = target.clone();
                for part in category(&path) { directory.push(part); }
                move_file(&path, &directory).unwrap();
            }
        }
        assert!(!file.exists());
        assert_eq!(fs::read(target.join("文档").join("其他文档").join("a.txt")).unwrap(), b"new");
        assert!(nested.is_dir()); assert!(target.join("existing.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test] fn reports_empty_and_missing_directories() {
        let root = std::env::temp_dir().join(format!("isle-empty-{}-{}", std::process::id(), Local::now().timestamp_nanos_opt().unwrap()));
        fs::create_dir_all(&root).unwrap();
        let inputs = collect_inputs(vec![root.to_string_lossy().into_owned(), root.join("missing").to_string_lossy().into_owned()], &root.join("target"));
        assert!(inputs.iter().any(|item| matches!(item, SortInput::Issue(result) if result.status == "skipped" && result.message == "空文件夹")));
        assert!(inputs.iter().any(|item| matches!(item, SortInput::Issue(result) if result.status == "failed")));
        fs::remove_dir_all(root).unwrap();
    }
    #[test] fn preserves_existing_and_moves_source() {
        let root = std::env::temp_dir().join(format!("isle-sort-{}-{}", std::process::id(), Local::now().timestamp_nanos_opt().unwrap()));
        let source_dir = root.join("source"); let target = root.join("target");
        fs::create_dir_all(&source_dir).unwrap(); fs::create_dir_all(&target).unwrap();
        let source = source_dir.join("a.txt"); fs::write(&source, b"new").unwrap(); fs::write(target.join("a.txt"), b"old").unwrap();
        let moved = move_file(&source, &target).unwrap();
        assert_eq!(fs::read(moved).unwrap(), b"new"); assert_eq!(fs::read(target.join("a.txt")).unwrap(), b"old"); assert!(!source.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
