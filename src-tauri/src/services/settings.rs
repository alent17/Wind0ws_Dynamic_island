//! 设置服务模块
//!
//! 提供应用设置的读写和开机自启动管理

use crate::error::{AppError, AppResult};
use crate::models::AppPreferences;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Manager};

const AUTOSTART_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const AUTOSTART_VALUE: &str = "Isle";
const LEGACY_AUTOSTART_VALUE: &str = "Wind0wsDynamicIsland";
const AUTOSTART_ARGUMENT: &str = "--startup";
static SETTINGS_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// 从配置文件读取设置
///
/// 配置文件位于应用数据目录下的 settings.json
/// 如果文件不存在或解析失败，返回 None
pub fn read_settings_file(app: &AppHandle) -> Option<AppPreferences> {
    let config_dir = app.path().app_data_dir().ok()?;
    let config_path = config_dir.join("settings.json");
    read_settings_from_path(&config_path)
}

fn read_settings_from_path(config_path: &Path) -> Option<AppPreferences> {
    parse_settings(&fs::read_to_string(config_path).ok()?)
        .or_else(|| parse_settings(&fs::read_to_string(backup_path(config_path)).ok()?))
}

fn parse_settings(content: &str) -> Option<AppPreferences> {
    let mut value: serde_json::Value = serde_json::from_str(content).ok()?;
    if let Some(settings) = value.as_object_mut() {
        // Older releases allowed several families; keep the setting key but use MiSans everywhere.
        settings.insert(
            "fontId".to_string(),
            serde_json::Value::String("misans".to_string()),
        );
        // Older releases shared one topmost preference between the island
        // and floating player. Seed the new independent value from that
        // preference once so existing users keep their current behavior.
        if !settings.contains_key("floatingWindowAlwaysOnTop") {
            let was_always_on_top = settings
                .get("alwaysOnTop")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            settings.insert(
                "floatingWindowAlwaysOnTop".to_string(),
                serde_json::Value::Bool(was_always_on_top),
            );
        }
        // The old appearance controls were shared across floating and edge
        // modes. Seed each new mode-specific value from that shared value,
        // unless a newer backup already contains its own value.
        let compact_length = settings
            .get("compactLength")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(80)
            .clamp(80, 300);
        for key in ["floatingCompactLength", "edgeCompactLength"] {
            settings
                .entry(key.to_string())
                .or_insert(serde_json::Value::from(compact_length));
        }
        let expanded_radius = settings
            .get("expandedCornerRadius")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(45)
            .min(80);
        for key in ["floatingExpandedCornerRadius", "edgeExpandedCornerRadius"] {
            settings
                .entry(key.to_string())
                .or_insert(serde_json::Value::from(expanded_radius));
        }
    }
    serde_json::from_value(value).ok()
}

/// 将设置写入配置文件
///
/// 自动创建配置目录（如果不存在）
/// 使用美观的 JSON 格式（带缩进）
pub fn write_settings_file(app: &AppHandle, settings: &AppPreferences) -> AppResult<()> {
    let config_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::config(format!("获取配置目录失败：{}", e)))?;

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| AppError::config(format!("创建配置目录失败：{}", e)))?;
    }

    let config_path = config_dir.join("settings.json");
    write_settings_snapshot(&config_path, settings)
}

pub(crate) fn write_settings_snapshot(
    config_path: &Path,
    settings: &AppPreferences,
) -> AppResult<()> {
    let content = serde_json::to_string_pretty(settings).map_err(AppError::Serialization)?;
    write_atomic_with_backup(config_path, content.as_bytes())
}

pub(crate) fn write_atomic_with_backup(config_path: &Path, content: &[u8]) -> AppResult<()> {
    let parent = config_path
        .parent()
        .ok_or_else(|| AppError::config("设置文件路径没有父目录"))?;
    fs::create_dir_all(parent).map_err(AppError::Io)?;
    let sequence = SETTINGS_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temp_path = parent.join(format!(".settings-{}-{sequence}.tmp", std::process::id()));
    let backup_path = backup_path(config_path);

    let result = (|| {
        let mut temp = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(AppError::Io)?;
        temp.write_all(content).map_err(AppError::Io)?;
        temp.sync_all().map_err(AppError::Io)?;
        drop(temp);
        replace_file_preserving_backup(&temp_path, config_path, &backup_path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn backup_path(config_path: &Path) -> PathBuf {
    let name = config_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("settings");
    config_path.with_file_name(format!("{name}.bak"))
}

fn replace_file_preserving_backup(temp: &Path, target: &Path, backup: &Path) -> AppResult<()> {
    if target.exists() {
        let target_is_recoverable = fs::read_to_string(target)
            .ok()
            .and_then(|content| parse_settings(&content))
            .is_some();
        let backup_is_recoverable = fs::read_to_string(backup)
            .ok()
            .and_then(|content| parse_settings(&content))
            .is_some();
        if target_is_recoverable || !backup_is_recoverable {
            fs::copy(target, backup).map_err(AppError::Io)?;
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(backup)
                .and_then(|file| file.sync_all())
                .map_err(AppError::Io)?;
        }
    }
    // Rust's Windows implementation uses MoveFileExW with
    // MOVEFILE_REPLACE_EXISTING; on Unix this is an atomic same-volume rename.
    fs::rename(temp, target).map_err(AppError::Io)
}

/// 设置开机自启动
///
/// 通过修改当前用户的 Windows 注册表实现。
///
/// 启动项使用当前安装的可执行文件路径，并附带一个明确的启动参数。
/// 这样升级后可以重新写入新路径，也不会依赖 `reg.exe` 的本地化输出格式。
pub fn set_auto_start(enable: bool) -> AppResult<()> {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        if enable {
            let (run_key, _) = hkcu
                .create_subkey(AUTOSTART_KEY)
                .map_err(|e| AppError::config(format!("打开开机启动注册表失败：{}", e)))?;
            let exe_path = std::env::current_exe()
                .map_err(|e| AppError::config(format!("获取可执行文件路径失败：{}", e)))?;
            let command = startup_command(&exe_path);

            run_key
                .set_value(AUTOSTART_VALUE, &command)
                .map_err(|e| AppError::config(format!("写入开机启动项失败：{}", e)))?;

            // 清理早期版本使用的名称，避免同一个应用登录时启动两次。
            let _ = run_key.delete_value(LEGACY_AUTOSTART_VALUE);
        } else {
            match hkcu.open_subkey_with_flags(AUTOSTART_KEY, winreg::enums::KEY_SET_VALUE) {
                Ok(run_key) => {
                    for value_name in [AUTOSTART_VALUE, LEGACY_AUTOSTART_VALUE] {
                        if let Err(error) = run_key.delete_value(value_name) {
                            if error.kind() != ErrorKind::NotFound {
                                return Err(AppError::config(format!(
                                    "删除开机启动项失败：{}",
                                    error
                                )));
                            }
                        }
                    }
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(AppError::config(format!(
                        "打开开机启动注册表失败：{}",
                        error
                    )))
                }
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = enable;
        Err(AppError::config("开机启动仅支持 Windows"))
    }
}

/// 检查是否已设置开机自启动
///
/// 通过查询注册表并比较启动命令中的可执行文件路径判断启动项是否有效。
pub fn get_auto_start() -> AppResult<bool> {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let current_exe = std::env::current_exe()
            .map_err(|e| AppError::config(format!("获取可执行文件路径失败：{}", e)))?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = match hkcu.open_subkey(AUTOSTART_KEY) {
            Ok(key) => key,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(AppError::config(format!(
                    "读取开机启动注册表失败：{}",
                    error
                )))
            }
        };

        for value_name in [AUTOSTART_VALUE, LEGACY_AUTOSTART_VALUE] {
            if let Ok(command) = run_key.get_value::<String, _>(value_name) {
                if normalize_windows_path(command_executable(&command))
                    == normalize_windows_path(&current_exe.to_string_lossy())
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

fn startup_command(exe_path: &Path) -> String {
    format!("\"{}\" {}", exe_path.display(), AUTOSTART_ARGUMENT)
}

fn command_executable(command: &str) -> &str {
    let command = command.trim();
    if let Some(quoted) = command.strip_prefix('"') {
        return quoted.split('"').next().unwrap_or_default();
    }

    command.split_whitespace().next().unwrap_or_default()
}

fn normalize_windows_path(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .replace('/', "\\")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{
        backup_path, command_executable, normalize_windows_path, read_settings_from_path,
        startup_command, write_atomic_with_backup, write_settings_snapshot,
    };
    use crate::models::AppPreferences;
    use std::fs;
    use std::path::Path;

    #[test]
    fn normalizes_quoted_windows_paths_for_registry_comparison() {
        assert_eq!(
            normalize_windows_path(r#""C:/Apps/Isle/Isle.exe""#),
            r#"c:\apps\isle\isle.exe"#
        );
    }

    #[test]
    fn extracts_executable_from_startup_command() {
        assert_eq!(
            command_executable(r#""C:\\Apps\\Isle\\isle.exe" --startup"#),
            r"C:\\Apps\\Isle\\isle.exe"
        );
        assert_eq!(
            command_executable(r#"C:\\Apps\\Isle\\isle.exe --startup"#),
            r"C:\\Apps\\Isle\\isle.exe"
        );
    }

    #[test]
    fn builds_quoted_startup_command() {
        assert_eq!(
            startup_command(Path::new(r"C:\Apps\Isle\isle.exe")),
            r#""C:\Apps\Isle\isle.exe" --startup"#
        );
    }

    #[test]
    fn atomic_replacement_keeps_previous_snapshot_and_does_not_destroy_failed_target() {
        let root = std::env::temp_dir().join(format!("isle-settings-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("test temp directory");
        let path = root.join("settings.json");
        write_atomic_with_backup(&path, br#"{"revision":1}"#).expect("initial write");
        write_atomic_with_backup(&path, br#"{"revision":2}"#).expect("replacement");
        assert_eq!(
            fs::read_to_string(&path).expect("primary"),
            r#"{"revision":2}"#
        );
        assert_eq!(
            fs::read_to_string(root.join("settings.json.bak")).expect("backup"),
            r#"{"revision":1}"#
        );

        let invalid_target = root.join("directory-target");
        fs::create_dir(&invalid_target).expect("target directory");
        assert!(write_atomic_with_backup(&invalid_target, b"new").is_err());
        assert!(invalid_target.is_dir());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn corrupted_primary_recovers_the_previous_parseable_settings_snapshot() {
        let root =
            std::env::temp_dir().join(format!("isle-settings-recovery-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("test temp directory");
        let path = root.join("settings.json");
        let previous = AppPreferences {
            window_opacity: 177,
            ..AppPreferences::default()
        };
        let current = AppPreferences {
            window_opacity: 88,
            ..AppPreferences::default()
        };
        write_settings_snapshot(&path, &previous).expect("initial settings");
        write_settings_snapshot(&path, &current).expect("replacement settings");
        fs::write(&path, b"truncated json").expect("simulate interrupted/corrupt primary");
        assert_eq!(read_settings_from_path(&path), Some(previous.clone()));
        let recovered = AppPreferences {
            window_opacity: 30,
            ..AppPreferences::default()
        };
        write_settings_snapshot(&path, &recovered).expect("write after recovery");
        assert_eq!(
            read_settings_from_path(&backup_path(&path)),
            Some(previous),
            "a corrupt primary must not replace a valid recovery snapshot"
        );
        assert_eq!(read_settings_from_path(&path), Some(recovered));
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn migrates_legacy_shared_appearance_values_into_each_mode() {
        let migrated = super::parse_settings(
            r#"{"compactLength":164,"expandedCornerRadius":62,"floatingCompactLength":220,"floatingExpandedCornerRadius":70}"#,
        )
        .expect("legacy settings should migrate");

        assert_eq!(migrated.floating_compact_length, 220);
        assert_eq!(migrated.edge_compact_length, 164);
        assert_eq!(migrated.floating_expanded_corner_radius, 70);
        assert_eq!(migrated.edge_expanded_corner_radius, 62);

        let stored = serde_json::to_value(migrated).expect("serialize migrated settings");
        assert!(stored.get("compactLength").is_none());
        assert!(stored.get("expandedCornerRadius").is_none());
    }
}
