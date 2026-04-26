//! 媒体相关命令模块
//!
//! 提供媒体信息获取、播放控制和图片处理命令

use tauri::{AppHandle, State};
use crate::error::{AppError, AppResult};
use crate::models::{MediaState, NeteaseSong};
use crate::state::AppState;
use crate::event_bus::EVENT_BUS;

/// 获取当前播放的媒体信息
#[tauri::command]
pub fn get_media_info_cmd(app: AppHandle) -> AppResult<MediaState> {
    crate::services::media::get_media_info(&app)
}

/// 从网易云音乐获取歌曲信息
#[tauri::command]
pub async fn get_netease_song_info_cmd(song_name: String, artist: String) -> AppResult<Option<NeteaseSong>> {
    crate::services::media::get_netease_song_info(&song_name, &artist).await
}

/// 获取网易云音乐 MV 播放 URL
#[tauri::command]
pub async fn get_netease_mv_url_cmd(mv_id: u64) -> AppResult<Option<String>> {
    crate::services::media::get_netease_mv_url(mv_id).await
}

/// 控制媒体播放
///
/// 支持的操作：
/// - "play_pause": 播放/暂停切换
/// - "next": 下一曲
/// - "prev": 上一曲
///
/// 操作完成后会自动更新媒体状态并通知前端
#[tauri::command]
pub fn control_media(app: AppHandle, action: String) -> AppResult<()> {
    let app_clone = app.clone();
    let action_clone = action.clone();
    
    // 在后台线程执行控制操作
    std::thread::spawn(move || {
        if let Err(e) = crate::services::media::control_media(&app_clone, &action_clone) {
            eprintln!("[control_media] Error: {:?}", e);
        }
        
        // 播放/暂停后更新媒体状态
        if action_clone == "play_pause" {
            let app_for_update = app_clone.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(150));
                unsafe {
                    let _ = windows::Win32::System::Com::CoInitializeEx(
                        None,
                        windows::Win32::System::Com::COINIT_MULTITHREADED,
                    );
                }
                if let Ok(info) = crate::services::media::get_media_info(&app_for_update) {
                    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_MEDIA_UPDATE, &info);
                }
            });
        }
    });

    Ok(())
}

/// 提取图片主色调
#[tauri::command]
pub fn extract_dominant_color(image_path: String) -> AppResult<(u8, u8, u8)> {
    crate::services::color::extract_dominant_color(&image_path)
}

/// 处理图片（可选像素化效果）
#[tauri::command]
pub async fn process_image(
    image_path: String,
    enable_pixel_art: bool,
) -> AppResult<String> {
    crate::services::image::process_image(&image_path, enable_pixel_art).await
}

/// 像素化封面图片
#[tauri::command]
pub fn pixelate_cover(image_path: String, pixel_size: u32) -> AppResult<String> {
    crate::services::image::pixelate_cover(&image_path, pixel_size)
}

/// 设置是否隐藏设置按钮
#[tauri::command]
pub fn set_hide_settings_button(
    app: AppHandle,
    state: State<'_, AppState>,
    enable: bool,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.hide_settings_button = enable;

    crate::services::write_settings_file(&app, &settings)?;

    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_SETTINGS_CHANGED, "hide_settings_button");

    Ok(())
}
