//! 媒体相关命令模块
//!
//! 提供媒体信息获取、播放控制和图片处理命令

use crate::error::{AppError, AppResult};
use crate::models::{MediaSessionInfo, MediaState, NeteaseSong, ResolvedCover};
use crate::state::AppState;
use tauri::{AppHandle, Manager};

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
pub async fn netease_song_like(
    title: String,
    artist: String,
    track_id: Option<String>,
    play_id: Option<String>,
    liked: Option<bool>,
) -> AppResult<serde_json::Value> {
    super::native_task::run_background(move || {
        let expected = match (&track_id, &play_id, liked) {
            (Some(track), Some(play), Some(liked)) => Some((track.as_str(), play.as_str(), liked)),
            (None, None, None) => None,
            _ => return Err(AppError::media("无效的歌曲身份")),
        };
        crate::services::netease_cdp::LocalCdpClient::default()
            .song_like(&title, &artist, expected)
            .map_err(|e| AppError::media(e.to_string()))
    })
    .await
    .map_err(|error| AppError::media(format!("网易云喜欢操作任务失败：{error}")))?
}

#[tauri::command]
pub async fn get_media_info_cmd(app: AppHandle) -> AppResult<MediaState> {
    let actor = app
        .state::<AppState>()
        .media_actor()
        .map_err(AppError::media)?;
    actor.snapshot().await.map_err(AppError::media)
}

/// Cycle a NetEase playback mode through the bounded local CDP adapter.
/// All socket work runs on Tauri's blocking pool, never the UI thread.
#[tauri::command]
pub async fn cycle_netease_playback_mode() -> AppResult<String> {
    super::native_task::run_background(|| {
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
    super::native_task::run_background(|| {
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
pub async fn list_media_sessions(app: AppHandle) -> AppResult<Vec<MediaSessionInfo>> {
    super::native_task::run_native(move || crate::services::media::list_media_sessions(&app))
        .await
        .map_err(|error| AppError::media(format!("读取媒体会话任务失败：{error}")))?
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
pub async fn control_media(app: AppHandle, action: String) -> AppResult<()> {
    if !matches!(action.as_str(), "play_pause" | "next" | "prev") {
        return Err(AppError::media("不支持的媒体操作"));
    }
    let worker_app = app.clone();
    super::native_task::run_transport(move || {
        crate::services::media::control_media(&worker_app, &action)
    })
    .await
    .map_err(|error| AppError::media(format!("媒体控制后台任务失败：{error}")))??;
    if let Ok(actor) = app.state::<AppState>().media_actor() {
        actor.wake_for_transport_change();
    }
    Ok(())
}

#[tauri::command]
pub async fn seek_media(
    app: AppHandle,
    position_ms: u64,
    expected_native_epoch: Option<String>,
    expected_title: Option<String>,
    expected_artist: Option<String>,
    expected_track_generation: Option<u64>,
) -> AppResult<Option<crate::services::netease_cdp::NeteaseProgress>> {
    let actor = app
        .state::<AppState>()
        .media_actor()
        .map_err(AppError::media)?;
    let before = actor.snapshot().await.map_err(AppError::media)?;
    let netease_basis = if before.source == "netease" {
        let (Some(epoch), Some(title), Some(artist), Some(track_generation)) = (
            expected_native_epoch,
            expected_title,
            expected_artist,
            expected_track_generation,
        ) else {
            return Err(AppError::media("网易云 seek 缺少当前媒体快照身份"));
        };
        if epoch.len() > 128 || title.len() > 1024 || artist.len() > 1024 {
            return Err(AppError::media("网易云 seek 媒体快照身份无效"));
        }
        crate::state::run_if_netease_seek_basis(
            &before,
            &epoch,
            &title,
            &artist,
            track_generation,
            || (),
        )
        .map_err(|_| AppError::media("媒体曲目已变化，请重试进度调整"))?;
        Some((epoch, title, artist, track_generation))
    } else {
        None
    };
    let worker_app = app.clone();
    let expected_title_for_cdp = netease_basis
        .as_ref()
        .map_or_else(String::new, |basis| basis.1.clone());
    let expected_artist_for_cdp = netease_basis
        .as_ref()
        .map_or_else(String::new, |basis| basis.2.clone());
    let progress = super::native_task::run_native(move || {
        crate::services::media::seek_media(
            &worker_app,
            position_ms,
            &expected_title_for_cdp,
            &expected_artist_for_cdp,
        )
    })
    .await
    .map_err(|error| AppError::media(format!("seek 后台任务失败：{error}")))??;
    if let Some(verified) = progress.as_ref() {
        let Some((epoch, title, artist, track_generation)) = netease_basis else {
            return Err(AppError::media("网易云 seek 的媒体快照身份已失效"));
        };
        actor
            .confirmed_netease_seek(
                epoch,
                title,
                artist,
                track_generation,
                verified.track_id.clone(),
                verified.play_id.clone(),
                verified.position_ms,
                verified.duration_ms,
            )
            .await
            .map_err(AppError::media)?;
    }
    Ok(progress)
}

#[tauri::command]
pub async fn toggle_shuffle(app: AppHandle) -> AppResult<()> {
    let worker_app = app.clone();
    super::native_task::run_transport(move || crate::services::media::toggle_shuffle(&worker_app))
        .await
        .map_err(|error| AppError::media(format!("shuffle 后台任务失败：{error}")))??;
    if let Ok(actor) = app.state::<AppState>().media_actor() {
        actor.wake_for_transport_change();
    }
    Ok(())
}

#[tauri::command]
pub async fn cycle_repeat(app: AppHandle) -> AppResult<()> {
    let worker_app = app.clone();
    super::native_task::run_transport(move || crate::services::media::cycle_repeat(&worker_app))
        .await
        .map_err(|error| AppError::media(format!("repeat 后台任务失败：{error}")))??;
    if let Ok(actor) = app.state::<AppState>().media_actor() {
        actor.wake_for_transport_change();
    }
    Ok(())
}

/// 提取图片主色调
#[tauri::command]
pub async fn extract_dominant_color(image_path: String) -> AppResult<(u8, u8, u8)> {
    super::native_task::run_background(move || {
        crate::services::color::extract_dominant_color(&image_path)
    })
    .await
    .map_err(|error| AppError::business(3010, format!("取色后台任务失败：{error}")))?
}

#[tauri::command]
pub async fn process_image(image_path: String, enable_pixel_art: bool) -> AppResult<String> {
    super::native_task::run_background(move || {
        crate::services::image::process_image(&image_path, enable_pixel_art)
    })
    .await
    .map_err(|error| AppError::business(3010, format!("图片处理后台任务失败：{error}")))?
}

#[tauri::command]
pub async fn pixelate_cover(image_path: String, pixel_size: u32) -> AppResult<String> {
    super::native_task::run_background(move || {
        crate::services::image::pixelate_cover(&image_path, pixel_size)
    })
    .await
    .map_err(|error| AppError::business(3010, format!("像素化后台任务失败：{error}")))?
}
