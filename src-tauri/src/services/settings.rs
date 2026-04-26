//! 设置服务模块
//!
//! 提供应用设置的读写和开机自启动管理

use std::fs;
use tauri::{AppHandle, Manager};
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;

/// 从配置文件读取设置
///
/// 配置文件位于应用数据目录下的 settings.json
/// 如果文件不存在或解析失败，返回 None
pub fn read_settings_file(app: &AppHandle) -> Option<AppSettings> {
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
pub fn write_settings_file(app: &AppHandle, settings: &AppSettings) -> AppResult<()> {
    let config_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::config(format!("获取配置目录失败：{}", e)))?;
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| AppError::config(format!("创建配置目录失败：{}", e)))?;
    }
    
    let config_path = config_dir.join("settings.json");
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| AppError::Serialization(e))?;
    
    fs::write(config_path, content)
        .map_err(|e| AppError::Io(e))
}

/// 设置开机自启动
///
/// 通过修改 Windows 注册表实现
/// 添加/删除 HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run 下的启动项
pub fn set_auto_start(enable: bool) -> AppResult<()> {
    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::config(format!("获取可执行文件路径失败：{}", e)))?;
    let exe_path_str = exe_path.to_string_lossy().to_string();

    if enable {
        // 添加注册表启动项
        let output = std::process::Command::new("reg")
            .args(&[
                "add",
                "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "Wind0wsDynamicIsland",
                "/t",
                "REG_SZ",
                "/d",
                &exe_path_str,
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
            .args(&[
                "query",
                "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "Wind0wsDynamicIsland",
            ])
            .output();
        
        if let Ok(check) = check_output {
            if check.status.success() {
                let output = std::process::Command::new("reg")
                    .args(&[
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
                    eprintln!("删除注册表项失败：{}", stderr);
                }
            }
        }
    }

    Ok(())
}

/// 检查是否已设置开机自启动
///
/// 通过查询注册表判断启动项是否存在
pub fn get_auto_start() -> AppResult<bool> {
    let output = std::process::Command::new("reg")
        .args(&[
            "query",
            "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v",
            "Wind0wsDynamicIsland",
        ])
        .output()
        .map_err(|e| AppError::config(format!("执行 reg query 命令失败：{}", e)))?;

    Ok(output.status.success())
}
