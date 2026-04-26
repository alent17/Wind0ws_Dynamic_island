//! 显示器相关命令模块
//!
//! 提供显示器信息获取和窗口移动命令

use tauri::{AppHandle, Manager, State};
use crate::error::{AppError, AppResult};
use crate::models::MonitorInfo;

/// 获取所有显示器信息
///
/// 返回显示器列表，包括名称、分辨率和是否为主显示器
#[tauri::command]
pub fn get_monitors(app: AppHandle) -> AppResult<Vec<MonitorInfo>> {
    let window = app
        .get_webview_window("main")
        .or_else(|| app.get_webview_window("floating"))
        .ok_or_else(|| AppError::window("Failed to get window"))?;
    
    let monitors = window
        .available_monitors()
        .map_err(|e| AppError::window(format!("Failed to get monitors: {}", e)))?;
    
    // 获取主显示器用于判断
    let primary_monitor = window.primary_monitor().ok().flatten();
    let primary_position = primary_monitor.as_ref().map(|m| m.position());
    
    let monitor_infos: Vec<MonitorInfo> = monitors
        .iter()
        .enumerate()
        .map(|(idx, m)| {
            // 通过位置比较判断是否为主显示器
            let is_primary = primary_position
                .map(|pp| pp.x == m.position().x && pp.y == m.position().y)
                .unwrap_or(false);
            
            MonitorInfo {
                index: idx as u32,
                name: m
                    .name()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| format!("显示器 {}", idx + 1)),
                width: m.size().width,
                height: m.size().height,
                is_primary,
            }
        })
        .collect();
    
    Ok(monitor_infos)
}

/// 将窗口移动到指定显示器
#[tauri::command]
pub fn move_to_monitor(app: AppHandle, monitor_index: u32) -> AppResult<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::window("Failed to get main window"))?;
    
    let monitors = window
        .available_monitors()
        .map_err(|e| AppError::window(format!("Failed to get monitors: {}", e)))?;
    
    // 验证显示器索引
    if monitor_index >= monitors.len() as u32 {
        return Err(AppError::business(4002, format!("Invalid monitor index: {}", monitor_index)));
    }
    
    let target_monitor = &monitors[monitor_index as usize];
    let position = target_monitor.position();
    
    // 移动窗口到目标显示器
    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: position.x,
            y: position.y,
        }))
        .map_err(|e| AppError::window(e.to_string()))?;
    
    Ok(())
}
