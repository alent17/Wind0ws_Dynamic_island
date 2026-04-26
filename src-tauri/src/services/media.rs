//! 媒体服务模块
//!
//! 提供 Windows 媒体会话访问和网易云音乐 API 集成

use crate::error::{AppError, AppResult};
use crate::models::{MediaState, NeteaseSong};
use crate::state::AppState;
use tauri::{AppHandle, Manager};
use base64::{engine::general_purpose, Engine};
use windows::{
    Foundation::*,
    Media::Control::*,
    Storage::Streams::*,
};
use serde_json::Value;

/// 获取当前播放的媒体信息
///
/// 通过 Windows GlobalSystemMediaTransportControlsSession API
/// 获取当前正在播放的音频/视频信息
///
/// 支持的播放器：
/// - 网易云音乐 (cloudmusic)
/// - Spotify
/// - Bilibili
/// - QQ音乐 (qqmusic)
/// - Apple Music
/// - 其他通用播放器
pub fn get_media_info(app: &AppHandle) -> AppResult<MediaState> {
    // 请求媒体传输控制会话管理器
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| AppError::media(format!("RequestAsync failed: {:?}", e)))?
        .get()
        .map_err(|e| AppError::media(format!("Await failed: {:?}", e)))?;

    // 获取所有媒体会话
    let sessions = manager
        .GetSessions()
        .map_err(|e| AppError::media(format!("GetSessions failed: {:?}", e)))?;
    
    let session_count = sessions.Size().unwrap_or(0) as usize;
    
    if session_count == 0 {
        return Ok(MediaState::default());
    }

    // 获取播放器优先级权重配置
    let weights = {
        let state = app.state::<AppState>();
        let settings = state
            .settings
            .lock()
            .map_err(|_| AppError::lock("Failed to lock settings"))?;
        settings.player_weights.clone()
    };

    // 根据权重选择最佳会话
    let mut best_session: Option<(GlobalSystemMediaTransportControlsSession, String, String)> = None;
    let mut best_weight = 0u32;

    for i in 0..session_count {
        if let Ok(session) = sessions.GetAt(i as u32) {
            let raw_id: String = session
                .SourceAppUserModelId()
                .unwrap_or_default()
                .to_string();
            let app_id_lower = raw_id.to_lowercase();

            // 识别播放器类型
            let source_type = if app_id_lower.contains("cloudmusic") {
                "netease"
            } else if app_id_lower.contains("spotify") {
                "spotify"
            } else if app_id_lower.contains("bilibili") {
                "bilibili"
            } else if app_id_lower.contains("qqmusic") {
                "qqmusic"
            } else if app_id_lower.contains("apple") && app_id_lower.contains("music") {
                "apple"
            } else {
                "generic"
            };

            let weight = weights.get(&source_type[..]).copied().unwrap_or(10);

            if weight > best_weight {
                best_weight = weight;
                best_session = Some((session, source_type.to_string(), raw_id));
            }
        }
    }

    let (session, source_type, raw_id) = match best_session {
        Some(s) => s,
        None => return Ok(MediaState::default()),
    };

    // 获取时间线属性（播放进度）
    let timeline = session
        .GetTimelineProperties()
        .map_err(|e| AppError::media(format!("GetTimelineProperties failed: {:?}", e)))?;

    // 获取媒体属性（标题、艺术家、封面）
    let info = session
        .TryGetMediaPropertiesAsync()
        .map_err(|e| AppError::media(format!("TryGetMediaPropertiesAsync failed: {:?}", e)))?
        .get()
        .map_err(|e| AppError::media(format!("Await media properties failed: {:?}", e)))?;

    let title = info.Title().unwrap_or_default().to_string();
    let artist = info.Artist().unwrap_or_default().to_string();
    let track_id = format!("{}|{}", title, artist);

    // 处理封面图片（带缓存）
    let mut thumbnail_base64 = String::new();
    let mut needs_update = true;

    {
        let state = app.state::<AppState>();
        if let Ok(cache) = state.media_cache.lock() {
            if cache.track_id == track_id && !cache.base64_img.is_empty() {
                thumbnail_base64 = cache.base64_img.clone();
                needs_update = false;
            }
        };
    }

    if needs_update {
        if let Ok(thumbnail_ref) = info.Thumbnail() {
            if let Ok(stream) = thumbnail_ref
                .OpenReadAsync()
                .map_err(|e| AppError::media(e.to_string()))?
                .get()
            {
                if let Ok(reader) = DataReader::CreateDataReader(&stream) {
                    let size = stream.Size().unwrap_or(0) as u32;
                    if size > 0 {
                        if let Ok(_) =
                            reader.LoadAsync(size).map_err(|e| AppError::media(e.to_string()))?.get()
                        {
                            let mut buffer = vec![0u8; size as usize];
                            if let Ok(_) = reader.ReadBytes(&mut buffer) {
                                thumbnail_base64 = format!(
                                    "data:image/png;base64,{}",
                                    general_purpose::STANDARD.encode(&buffer)
                                );
                            }
                        }
                    }
                }
            }
        }

        // 更新缓存
        if !thumbnail_base64.is_empty() {
            let state = app.state::<AppState>();
            if let Ok(mut cache) = state.media_cache.lock() {
                cache.track_id = track_id;
                cache.base64_img = thumbnail_base64.clone();
            };
        }
    }

    // 获取播放状态
    let playback_status = session
        .GetPlaybackInfo()
        .map_err(|e| AppError::media(format!("GetPlaybackInfo failed: {:?}", e)))?
        .PlaybackStatus()
        .unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus(0));

    let is_playing = playback_status.0 == 4; // Playing = 4

    // 计算实际播放位置（考虑时间差）
    let mut dur_ms =
        (timeline.EndTime().unwrap_or_default().Duration / 10000) as u64;
    let snapshot_pos_ms =
        (timeline.Position().unwrap_or_default().Duration / 10000) as u64;
    let last_updated_filetime =
        timeline.LastUpdatedTime().unwrap_or_default().UniversalTime;

    // Windows FILETIME 转换为 Unix 时间戳
    let last_updated_timestamp = if last_updated_filetime > 0 {
        ((last_updated_filetime as i64 / 10000) - 11644473600000) as u64
    } else {
        0
    };

    let now_filetime = chrono::Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or(0) / 100
        + 116444736000000000;
    let diff_ms = if now_filetime > last_updated_filetime {
        ((now_filetime - last_updated_filetime) / 10000) as u64
    } else {
        0
    };

    // 计算实时播放位置
    let real_pos_ms = if is_playing && dur_ms > 0 {
        (snapshot_pos_ms + diff_ms).min(dur_ms)
    } else {
        snapshot_pos_ms
    };

    if dur_ms == 0 && is_playing {
        dur_ms = 1;
    }

    // 处理直播流（时长为0或超大）
    let is_live_logic = dur_ms == 0 || dur_ms > 360000000;
    let (position_ms, duration_ms) = if is_live_logic {
        (0u64, 0u64)
    } else {
        (real_pos_ms, dur_ms)
    };

    Ok(MediaState {
        title,
        artist,
        album_art: thumbnail_base64,
        is_playing,
        position_ms,
        duration_ms,
        last_updated_timestamp,
        source: source_type,
        source_display: raw_id,
    })
}

/// 从网易云音乐 API 获取歌曲信息
///
/// 通过歌曲名称和艺术家搜索，返回：
/// - 歌曲时长
/// - 专辑封面 URL
/// - MV ID 和播放 URL
pub async fn get_netease_song_info(song_name: &str, artist: &str) -> AppResult<Option<NeteaseSong>> {
    let keyword = format!("{} {}", artist, song_name);
    let encoded_keyword = urlencoding::encode(&keyword);
    let url = format!("https://music.163.com/api/search/get/?s={}&type=1&limit=1", encoded_keyword);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| AppError::network(format!("Request failed: {}", e)))?;

    let text = response
        .text()
        .await
        .map_err(|e| AppError::network(format!("Read response failed: {}", e)))?;

    let json: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::parse(format!("JSON parse failed: {}", e)))?;

    if let Some(songs) = json["result"]["songs"].as_array() {
        if let Some(first_song) = songs.first() {
            let duration = first_song["duration"].as_u64();
            let album_pic = first_song["album"]["picUrl"]
                .as_str()
                .map(|s| s.to_string());
            let mv_id = first_song["mv"].as_i64();

            // 如果有 MV，获取 MV URL
            let mut mv_url: Option<String> = None;
            if let Some(id) = mv_id {
                if id > 0 {
                    mv_url = get_netease_mv_url_internal(id as u64).await?;
                }
            }

            return Ok(Some(NeteaseSong {
                duration,
                album_pic,
                mv_id,
                mv_url,
            }));
        }
    }

    Ok(None)
}

/// 获取网易云音乐 MV 播放 URL
pub async fn get_netease_mv_url(mv_id: u64) -> AppResult<Option<String>> {
    get_netease_mv_url_internal(mv_id).await
}

/// 内部函数：获取 MV URL
async fn get_netease_mv_url_internal(mv_id: u64) -> AppResult<Option<String>> {
    let url = format!("https://music.163.com/api/mv/detail?id={}", mv_id);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| AppError::network(format!("MV request failed: {}", e)))?;

    let text = response
        .text()
        .await
        .map_err(|e| AppError::network(format!("Read MV response failed: {}", e)))?;

    let json: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::parse(format!("MV JSON parse failed: {}", e)))?;

    // 尝试从 brs 数组获取 URL
    if let Some(data) = json.get("data") {
        if let Some(brs) = data.get("brs").and_then(|v| v.as_array()) {
            for br in brs {
                if let Some(url) = br.get("url").and_then(|v| v.as_str()) {
                    return Ok(Some(url.to_string()));
                }
            }
        }
        // 备用：直接从 data.url 获取
        if let Some(url) = data.get("url").and_then(|v| v.as_str()) {
            return Ok(Some(url.to_string()));
        }
    }

    Ok(None)
}

/// 控制媒体播放
///
/// 支持的操作：
/// - "play_pause": 播放/暂停切换
/// - "next": 下一曲
/// - "prev": 上一曲
pub fn control_media(app: &AppHandle, action: &str) -> AppResult<()> {
    // 初始化 COM
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );
    }

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|op| op.get())
        .map_err(|e| AppError::media(format!("RequestAsync failed: {:?}", e)))?;

    let sessions = manager
        .GetSessions()
        .map_err(|e| AppError::media(format!("GetSessions failed: {:?}", e)))?;

    let session_count = sessions.Size().unwrap_or(0) as usize;
    if session_count == 0 {
        return Ok(());
    }

    // 获取播放器权重
    let weights = {
        let state = app.state::<AppState>();
        let guard = state.settings.lock();
        guard.map(|g| g.player_weights.clone()).unwrap_or_default()
    };

    // 选择最佳会话
    let mut best_session = None;
    let mut best_weight = 0u32;

    for i in 0..session_count {
        if let Ok(session) = sessions.GetAt(i as u32) {
            let raw_id = session.SourceAppUserModelId().unwrap_or_default().to_string();
            let app_id_lower = raw_id.to_lowercase();

            let source_type = if app_id_lower.contains("cloudmusic") { "netease" }
                else if app_id_lower.contains("spotify") { "spotify" }
                else if app_id_lower.contains("bilibili") { "bilibili" }
                else if app_id_lower.contains("qqmusic") { "qqmusic" }
                else if app_id_lower.contains("apple") && app_id_lower.contains("music") { "apple" }
                else { "generic" };

            let weight = weights.get(source_type).copied().unwrap_or(10);
            if weight > best_weight {
                best_weight = weight;
                best_session = Some(session);
            }
        }
    }

    // 执行控制操作
    if let Some(session) = best_session {
        match action {
            "play_pause" => {
                let _ = session.TryTogglePlayPauseAsync();
            }
            "next" => { let _ = session.TrySkipNextAsync(); }
            "prev" => { let _ = session.TrySkipPreviousAsync(); }
            _ => {}
        }
    }

    Ok(())
}
