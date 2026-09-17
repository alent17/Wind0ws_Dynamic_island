//! 设置服务模块
//!
//! 提供应用设置的读写和开机自启动管理

use crate::error::{AppError, AppResult};
use crate::models::AppPreferences;
use std::fs;
use tauri::{AppHandle, Manager};

/// 从配置文件读取设置
///
/// 配置文件位于应用数据目录下的 settings.json
/// 如果文件不存在或解析失败，返回 None
pub fn read_settings_file(app: &AppHandle) -> Option<AppPreferences> {
    let config_dir = app.path().app_data_dir().ok()?;
    let config_path = config_dir.join("settings.json");

    if !config_path.exists() {
        return None;
    }

    let content = fs::read_to_string(config_path).ok()?;
    serde_json::from_str(&content).ok()
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
    let content = serde_json::to_string_pretty(settings).map_err(AppError::Serialization)?;

    fs::write(config_path, content).map_err(AppError::Io)
}

/// 设置开机自启动
///
/// 通过修改 Windows 注册表实现
/// 添加/删除 HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run 下的启动项
pub fn set_auto_start(enable: bool) -> AppResult<()> {
    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::config(format!("获取可执行文件路径失败：{}", e)))?;
    let registry_value = format!("\"{}\"", exe_path.display());

    if enable {
        // 添加注册表启动项
        let output = std::process::Command::new("reg")
            .args([
                "add",
                "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "Wind0wsDynamicIsland",
                "/t",
                "REG_SZ",
                "/d",
                &registry_value,
                "/f",
            ])
            .output()
            .map_err(|e| AppError::config(format!("执行 reg 命令失败：{}", e)))?;

        if !output.status.success() {
            return Err(AppError::config(format!(
                "添加注册表项失败：{}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    } else {
        // 检查并删除注册表启动项
        let check_output = std::process::Command::new("reg")
            .args([
                "query",
                "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "Wind0wsDynamicIsland",
            ])
            .output();

        if let Ok(check) = check_output {
            if check.status.success() {
                let output = std::process::Command::new("reg")
                    .args([
                        "delete",
                        "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                        "/v",
                        "Wind0wsDynamicIsland",
                        "/f",
                    ])
                    .output()
                    .map_err(|e| AppError::config(format!("执行 reg delete 命令失败：{}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!("删除注册表项失败：{}", stderr);
                }
            }
        }
    }

    Ok(())
}

/// 检查是否已设置开机自启动
///
/// 通过查询注册表并比较当前可执行文件路径判断启动项是否有效
pub fn get_auto_start() -> AppResult<bool> {
    let current_exe = std::env::current_exe()
        .map_err(|e| AppError::config(format!("获取可执行文件路径失败：{}", e)))?;
    let output = std::process::Command::new("reg")
        .args([
            "query",
            "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v",
            "Wind0wsDynamicIsland",
        ])
        .output()
        .map_err(|e| AppError::config(format!("执行 reg query 命令失败：{}", e)))?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let registry_value = stdout
        .lines()
        .find_map(|line| {
            line.find("REG_SZ")
                .map(|index| line[index + "REG_SZ".len()..].trim())
        })
        .unwrap_or_default();

    Ok(normalize_windows_path(registry_value)
        == normalize_windows_path(&current_exe.to_string_lossy()))
}

fn normalize_windows_path(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .replace('/', "\\")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::normalize_windows_path;

    #[test]
    fn normalizes_quoted_windows_paths_for_registry_comparison() {
        assert_eq!(
            normalize_windows_path(r#""C:/Apps/Isle/Isle.exe""#),
            r#"c:\apps\isle\isle.exe"#
        );
    }
}
