//! 媒体相关命令模块
//!
//! 提供媒体信息获取、播放控制和图片处理命令

use crate::error::{AppError, AppResult};
use crate::event_bus::EVENT_BUS;
use crate::models::{MediaSessionInfo, MediaState, NeteaseSong, ResolvedCover};
use tauri::AppHandle;

#[tauri::command]
pub fn cancel_artwork(window: tauri::WebviewWindow, slot: String, revision: u64) -> AppResult<()> {
    validate_artwork_owner(&slot, revision)?;
    crate::services::artwork_jobs::registry().cancel(window.label(), &slot, revision);
    Ok(())
}
fn validate_artwork_owner(slot: &str, revision: u64) -> AppResult<()> {
    if !["cover", "color", "pixel"].contains(&slot)
        || revision > 9_007_199_254_740_991
        || revision == 0
    {
        return Err(AppError::business(3006, "无效封面所有者"));
    }
    Ok(())
}
#[tauri::command]
pub async fn request_artwork(
    window: tauri::WebviewWindow,
    slot: String,
    revision: u64,
    image_path: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    source: Option<String>,
    cover_version: Option<String>,
    enable_pixel_art: Option<bool>,
) -> AppResult<serde_json::Value> {
    use crate::services::{artwork_jobs as jobs, color, image, media};
    validate_artwork_owner(&slot, revision)?;
    static READS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(8);
    let _admission = READS
        .try_acquire()
        .map_err(|_| AppError::business(3005, "封面请求已满"))?;
    if slot == "cover" {
        let title = title.unwrap_or_default();
        let artist = artist.unwrap_or_default();
        let source = source.unwrap_or_default();
        let version = cover_version.unwrap_or_default();
        if title.len() + artist.len() + source.len() > 4096 || version.len() > 128 {
            return Err(AppError::business(3006, "封面身份过长"));
        }
        let key = jobs::identity(
            serde_json::to_string(&("cover", &title, &artist, &source, &version))?.as_bytes(),
        );
        let result = jobs::registry()
            .run(window.label(), &slot, revision, key, async move {
                media::resolve_hd_cover_versioned(&title, &artist, &source, &version).await
            })
            .await?;
        Ok(serde_json::to_value(result)?)
    } else {
        let path = image_path.unwrap_or_default();
        if path.len() > crate::services::image_budget::MAX_IMAGE_URI_CHARS {
            return Err(AppError::business(3004, "封面输入过长"));
        }
        let pixel = slot == "pixel" && enable_pixel_art.unwrap_or(true);
        let pixel_key = [u8::from(pixel)];
        let key = jobs::identity_parts(&[slot.as_bytes(), &pixel_key, path.as_bytes()]);
        if slot == "color" {
            let result = jobs::registry()
                .run(window.label(), &slot, revision, key, async move {
                    jobs::blocking(move || {
                        let bytes = crate::utils::load_image_data(&path)?;
                        color::extract_dominant_color_bytes(&bytes)
                    })
                    .await
                })
                .await?;
            Ok(serde_json::to_value(result)?)
        } else {
            let result = jobs::registry()
                .run(window.label(), &slot, revision, key, async move {
                    jobs::blocking(move || {
                        let bytes = crate::utils::load_image_data(&path)?;
                        image::process_image_bytes(&bytes, pixel)
                    })
                    .await
                })
                .await?;
            Ok(serde_json::to_value(result)?)
        }
    }
}

/// 获取当前播放的媒体信息
#[tauri::command]
pub async fn get_media_info_cmd(app: AppHandle) -> AppResult<MediaState> {
    tauri::async_runtime::spawn_blocking(move || crate::services::media::get_media_info(&app))
        .await
        .map_err(|error| AppError::media(format!("读取媒体信息任务失败：{}", error)))?
}

/// Cycle a NetEase playback mode through the bounded local CDP adapter.
/// All socket work runs on Tauri's blocking pool, never the UI thread.
#[tauri::command]
pub async fn cycle_netease_playback_mode() -> AppResult<String> {
    tauri::async_runtime::spawn_blocking(|| {
        use crate::services::netease_cdp::{mode_key, next_mode, LocalCdpClient};

        let client = LocalCdpClient::default();
        let current = client
            .playback_mode()
            .map_err(|error| AppError::media(error.to_string()))?
            .ok_or_else(|| AppError::media("网易云当前播放模式无法识别"))?;
        let requested = next_mode(current);
        let action = client
            .set_playback_mode(requested)
            .map_err(|error| AppError::media(error.to_string()))?;
        Ok(mode_key(action.observed).to_string())
    })
    .await
    .map_err(|error| AppError::media(format!("网易云播放模式任务失败：{error}")))?
}

#[tauri::command]
pub async fn get_netease_playback_mode() -> AppResult<Option<String>> {
    tauri::async_runtime::spawn_blocking(|| {
        use crate::services::netease_cdp::{mode_key, LocalCdpClient};

        LocalCdpClient::default()
            .playback_mode()
            .map(|mode| mode.map(mode_key).map(str::to_string))
            .map_err(|error| AppError::media(error.to_string()))
    })
    .await
    .map_err(|error| AppError::media(format!("读取网易云播放模式任务失败：{error}")))?
}

#[tauri::command]
pub async fn list_media_sessions() -> AppResult<Vec<MediaSessionInfo>> {
    tauri::async_runtime::spawn_blocking(crate::services::media::list_media_sessions)
        .await
        .map_err(|error| AppError::media(format!("读取媒体会话任务失败：{}", error)))?
}

/// 从网易云音乐获取歌曲信息
#[tauri::command]
pub async fn get_netease_song_info_cmd(
    song_name: String,
    artist: String,
) -> AppResult<Option<NeteaseSong>> {
    crate::services::media::get_netease_song_info(&song_name, &artist).await
}

/// 获取网易云音乐 MV 播放 URL
#[tauri::command]
pub async fn get_netease_mv_url_cmd(mv_id: u64) -> AppResult<Option<String>> {
    crate::services::media::get_netease_mv_url(mv_id).await
}

#[tauri::command]
pub async fn resolve_hd_cover(
    title: String,
    artist: String,
    source: String,
) -> AppResult<Option<ResolvedCover>> {
    crate::services::media::resolve_hd_cover(&title, &artist, &source).await
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
    // Wait for the GSMTC operation here. The command used to return before
    // Windows had completed the request, which made the island appear to
    // toggle while playback did not change and hid all backend errors.
    crate::services::media::control_media(&app, &action)?;

    // Refresh play/pause state shortly after the player applies the command.
    // This keeps the island responsive without relying solely on the 1-second
    // media listener interval.
    if action == "play_pause" {
        std::thread::sleep(std::time::Duration::from_millis(150));
        if let Ok(info) = crate::services::media::get_media_info(&app) {
            let _ = EVENT_BUS.emit(crate::event_bus::EVENT_MEDIA_UPDATE, &info);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn seek_media(app: AppHandle, position_ms: u64) -> AppResult<()> {
    crate::services::media::seek_media(&app, position_ms)
}

#[tauri::command]
pub fn toggle_shuffle(app: AppHandle) -> AppResult<()> {
    crate::services::media::toggle_shuffle(&app)
}

#[tauri::command]
pub fn cycle_repeat(app: AppHandle) -> AppResult<()> {
    crate::services::media::cycle_repeat(&app)
}

/// 提取图片主色调
#[tauri::command]
pub fn extract_dominant_color(image_path: String) -> AppResult<(u8, u8, u8)> {
    crate::services::color::extract_dominant_color(&image_path)
}

#[tauri::command]
pub async fn process_image(image_path: String, enable_pixel_art: bool) -> AppResult<String> {
    crate::services::image::process_image(&image_path, enable_pixel_art).await
}

#[tauri::command]
pub fn pixelate_cover(image_path: String, pixel_size: u32) -> AppResult<String> {
    crate::services::image::pixelate_cover(&image_path, pixel_size)
}
