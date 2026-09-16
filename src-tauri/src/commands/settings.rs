//! 设置相关命令模块
//!
//! 提供应用设置的读取、保存和各项配置的修改命令

use crate::error::{AppError, AppResult};
use crate::event_bus::EVENT_BUS;
use crate::models::{AppPreferences, AppSettings};
use crate::services::{
    get_auto_start as get_auto_start_service, set_auto_start, write_settings_file,
};
use crate::state::AppState;
use std::collections::HashMap;
use tauri::{AppHandle, Manager, State};

pub(crate) fn apply_capture_protection(app: &AppHandle, settings: &AppPreferences) {
    let protect = settings.capture_hide_on_screenshot
        || settings.capture_hide_on_recording
        || settings.capture_hide_on_screen_share;
    for label in ["main", "floating_player"] {
        if let Some(window) = app.get_webview_window(label) {
            if let Err(error) = window.set_content_protected(protect) {
                tracing::warn!("[Capture Mode] 设置窗口 {} 内容保护失败: {}", label, error);
            }
        }
    }
}

/// 获取当前应用设置
#[tauri::command]
pub fn get_settings(_app: AppHandle, state: State<'_, AppState>) -> AppResult<AppSettings> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    Ok(settings.clone())
}

/// Current Isle Studio compatibility endpoint.
#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> AppResult<AppPreferences> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock preferences"))?;
    Ok(settings.clone())
}

/// 保存应用设置
///
/// 同时处理：
/// - 持久化到配置文件
/// - 更新窗口置顶状态
/// - 更新开机自启动
/// - 发送相关事件通知前端
#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    mut settings: AppSettings,
) -> AppResult<()> {
    settings.compact_length = settings.compact_length.clamp(80, 300);
    settings.island_edge_position = settings.island_edge_position.min(100);
    settings.edge_shoulder_radius = settings.edge_shoulder_radius.min(16);
    settings.expanded_corner_radius = settings.expanded_corner_radius.min(80);
    settings.idle_rotation_seconds = settings.idle_rotation_seconds.clamp(2, 60);
    if !matches!(
        settings.font_id.as_str(),
        "system" | "misans" | "source-han-serif-cn-bold" | "alibaba-puhuiti-heavy"
    ) {
        settings.font_id = "system".to_string();
    }
    if let Some(ids) = settings.selected_player_ids.as_mut() {
        let mut seen = std::collections::HashSet::new();
        ids.retain(|id| !id.trim().is_empty() && seen.insert(id.clone()));
    }
    settings.idle_items.retain_mut(|item| {
        if item.kind == "custom" {
            item.text = item.text.trim().chars().take(120).collect();
            !item.text.is_empty()
        } else {
            matches!(
                item.kind.as_str(),
                "clock" | "date" | "weather" | "network" | "cpu" | "memory" | "battery"
            )
        }
    });
    let always_on_top = settings.always_on_top;
    let auto_start = settings.auto_start;

    // 保存旧设置用于比较变更
    let old_settings: Option<AppSettings> = {
        let mut state_settings = state
            .settings
            .lock()
            .map_err(|_| AppError::lock("Failed to lock settings"))?;
        let old = state_settings.clone();
        *state_settings = settings.clone();
        Some(old)
    };

    // 持久化到文件
    let app_for_write = app.clone();
    let settings_for_write = settings.clone();
    tauri::async_runtime::spawn_blocking(move || {
        write_settings_file(&app_for_write, &settings_for_write)
    })
    .await
    .map_err(|error| AppError::config(format!("保存设置任务失败：{}", error)))??;

    // 仅在相关设置真正变化时调用 Windows API。发布版中注册表和
    // SetWindowDisplayAffinity 都可能阻塞数秒，不能在每次普通保存时执行。
    if old_settings
        .as_ref()
        .is_some_and(|old| old.always_on_top != settings.always_on_top)
    {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_always_on_top(always_on_top);
        }
    }
    if old_settings.as_ref().is_some_and(|old| {
        old.capture_hide_on_screenshot != settings.capture_hide_on_screenshot
            || old.capture_hide_on_recording != settings.capture_hide_on_recording
            || old.capture_hide_on_screen_share != settings.capture_hide_on_screen_share
    }) {
        apply_capture_protection(&app, &settings);
    }

    // 只有开关本身发生变化时才访问开机启动注册表。
    if old_settings
        .as_ref()
        .is_some_and(|old| old.auto_start != settings.auto_start)
    {
        if auto_start {
            set_auto_start(true)?;
        } else {
            let _ = set_auto_start(false);
        }
    }

    // 发送设置变更事件
    if let Some(old) = old_settings {
        if old.enable_halftone != settings.enable_halftone {
            let _ = EVENT_BUS.emit(
                crate::event_bus::EVENT_HALFTONE_CHANGED,
                serde_json::json!({"enableHalftone": settings.enable_halftone}),
            );
        }
        if old.enable_pixel_art != settings.enable_pixel_art {
            let _ = EVENT_BUS.emit(
                crate::event_bus::EVENT_PIXEL_ART_CHANGED,
                serde_json::json!({"enablePixelArt": settings.enable_pixel_art}),
            );
        }
        if old.enable_hd_cover != settings.enable_hd_cover {
            let _ = EVENT_BUS.emit(
                crate::event_bus::EVENT_HD_COVER_CHANGED,
                serde_json::json!({"enableHDCover": settings.enable_hd_cover}),
            );
        }
        if old.enable_mv_playback != settings.enable_mv_playback {
            let _ = EVENT_BUS.emit(
                crate::event_bus::EVENT_MV_PLAYBACK_CHANGED,
                serde_json::json!({"enable": settings.enable_mv_playback}),
            );
        }
        if old.lock_floating_window != settings.lock_floating_window {
            let _ = EVENT_BUS.emit(
                crate::event_bus::EVENT_LOCK_FLOATING_WINDOW_CHANGED,
                serde_json::json!({"lock": settings.lock_floating_window}),
            );
        }
        if old.always_on_top != settings.always_on_top {
            let _ = EVENT_BUS.emit(
                crate::event_bus::EVENT_ALWAYS_ON_TOP_CHANGED,
                serde_json::json!({"isAlwaysOnTop": settings.always_on_top}),
            );
        }
        if old.expanded_corner_radius != settings.expanded_corner_radius {
            let _ = EVENT_BUS.emit(
                crate::event_bus::EVENT_CORNER_RADIUS_CHANGED,
                settings.expanded_corner_radius,
            );
        }
    }

    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_SETTINGS_UPDATED, settings.clone());

    Ok(())
}

/// Current Isle Studio compatibility endpoint. Both surfaces intentionally
/// persist the same complete settings object.
#[tauri::command]
pub async fn save_preferences(
    app: AppHandle,
    state: State<'_, AppState>,
    preferences: AppPreferences,
) -> AppResult<()> {
    save_settings(app, state, preferences).await
}

/// 设置主题
#[tauri::command]
pub fn set_theme(app: AppHandle, state: State<'_, AppState>, theme: String) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.island_theme = theme.clone();
    write_settings_file(&app, &settings)?;
    Ok(())
}

/// 获取当前主题
#[tauri::command]
pub fn get_theme(state: State<'_, AppState>) -> AppResult<String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    Ok(settings.island_theme.clone())
}

/// 设置窗口置顶
#[tauri::command]
pub fn set_always_on_top(
    app: AppHandle,
    state: State<'_, AppState>,
    enable: bool,
) -> AppResult<()> {
    {
        let mut settings = state
            .settings
            .lock()
            .map_err(|_| AppError::lock("Failed to lock settings"))?;
        settings.always_on_top = enable;
        write_settings_file(&app, &settings)?;
    }

    if let Some(window) = app.get_webview_window("main") {
        window
            .set_always_on_top(enable)
            .map_err(|e| AppError::window(e.to_string()))?;
    }
    Ok(())
}

/// 设置窗口不透明度
#[tauri::command]
pub fn set_window_opacity(
    app: AppHandle,
    state: State<'_, AppState>,
    opacity: u8,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.window_opacity = opacity;
    write_settings_file(&app, &settings)?;
    Ok(())
}

/// 获取播放器优先级权重配置
#[tauri::command]
pub fn get_player_weights(state: State<'_, AppState>) -> AppResult<HashMap<String, u32>> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    Ok(settings.player_weights.clone())
}

/// 设置单个播放器的优先级权重
#[tauri::command]
pub fn set_player_weight(
    app: AppHandle,
    state: State<'_, AppState>,
    player: String,
    weight: u32,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.player_weights.insert(player, weight);

    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_SETTINGS_UPDATED, &*settings);

    write_settings_file(&app, &settings)?;

    Ok(())
}

/// 批量设置播放器优先级权重
#[tauri::command]
pub fn set_player_weights(
    app: AppHandle,
    state: State<'_, AppState>,
    weights: HashMap<String, u32>,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.player_weights = weights;
    write_settings_file(&app, &settings)?;
    Ok(())
}

/// 设置开机自启动
#[tauri::command]
pub fn set_auto_start_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    enable: bool,
) -> AppResult<()> {
    set_auto_start(enable)?;
    let settings = {
        let mut settings = state
            .settings
            .lock()
            .map_err(|_| AppError::lock("Failed to lock settings"))?;
        settings.auto_start = enable;
        settings.clone()
    };
    write_settings_file(&app, &settings)
}

/// 获取开机自启动状态
#[tauri::command]
pub fn get_auto_start() -> AppResult<bool> {
    get_auto_start_service()
}

/// 保存悬浮窗口位置和大小
#[tauri::command]
pub fn save_floating_window_position(
    app: AppHandle,
    state: State<'_, AppState>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;

    settings.floating_window_x = Some(x);
    settings.floating_window_y = Some(y);
    settings.floating_window_width = Some(width.max(200));
    settings.floating_window_height = Some(height.max(200));

    write_settings_file(&app, &settings)?;

    Ok(())
}

/// 获取悬浮窗口位置和大小
#[tauri::command]
pub fn get_floating_window_position(
    state: State<'_, AppState>,
) -> AppResult<Option<(i32, i32, u32, u32)>> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;

    if let (Some(x), Some(y), Some(w), Some(h)) = (
        settings.floating_window_x,
        settings.floating_window_y,
        settings.floating_window_width,
        settings.floating_window_height,
    ) {
        Ok(Some((x, y, w, h)))
    } else {
        Ok(None)
    }
}
