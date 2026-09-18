//! 显示器相关命令模块
//!
//! 提供显示器信息获取和窗口移动命令

use crate::error::{AppError, AppResult};
use crate::models::MonitorInfo;
use tauri::{AppHandle, Manager};

/// 获取所有显示器信息
///
/// 返回显示器列表，包括名称、分辨率和是否为主显示器
#[tauri::command]
pub async fn get_monitors(app: AppHandle) -> AppResult<Vec<MonitorInfo>> {
    tauri::async_runtime::spawn_blocking(move || get_monitors_blocking(app))
        .await
        .map_err(|error| AppError::window(format!("读取显示器信息任务失败：{}", error)))?
}

fn get_monitors_blocking(app: AppHandle) -> AppResult<Vec<MonitorInfo>> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
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

            let position = m.position();
            let size = m.size();
            let point = POINT {
                x: position.x + size.width as i32 / 2,
                y: position.y + size.height as i32 / 2,
            };
            let handle = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
            let mut native = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let has_work_area = unsafe { GetMonitorInfoW(handle, &mut native) }.as_bool();
            let work = if has_work_area {
                native.rcWork
            } else {
                windows::Win32::Foundation::RECT {
                    left: position.x,
                    top: position.y,
                    right: position.x + size.width as i32,
                    bottom: position.y + size.height as i32,
                }
            };
            MonitorInfo {
                index: idx as u32,
                name: m
                    .name()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| format!("显示器 {}", idx + 1)),
                width: size.width,
                height: size.height,
                is_primary,
                x: position.x,
                y: position.y,
                work_x: work.left,
                work_y: work.top,
                work_width: (work.right - work.left).max(0) as u32,
                work_height: (work.bottom - work.top).max(0) as u32,
                scale_factor: m.scale_factor(),
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
        return Err(AppError::business(
            4002,
            format!("Invalid monitor index: {}", monitor_index),
        ));
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
