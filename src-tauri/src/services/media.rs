//! 媒体服务模块
//!
//! 提供 Windows 媒体会话访问和网易云音乐 API 集成

use crate::error::{AppError, AppResult};
use crate::models::{MediaCapabilities, MediaSessionInfo, MediaState, NeteaseSong, ResolvedCover};
use crate::state::AppState;
use base64::{engine::general_purpose, Engine};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use windows::{
    Media::{Control::*, MediaPlaybackAutoRepeatMode},
    Storage::Streams::*,
};

const COVER_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const COVER_NEGATIVE_CACHE_TTL: Duration = Duration::from_secs(15 * 60);
const COVER_MATCH_THRESHOLD: u8 = 65;
const APPLE_REQUESTS_PER_MINUTE: usize = 18;

#[derive(Clone, Debug)]
struct CoverCandidate {
    title: String,
    artist: String,
    url: String,
    source_id: Option<u64>,
}

static COVER_CACHE: OnceLock<Mutex<HashMap<String, (Instant, Option<ResolvedCover>)>>> =
    OnceLock::new();
static APPLE_REQUESTS: OnceLock<Mutex<VecDeque<Instant>>> = OnceLock::new();
static LAST_AUTO_SESSION_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static TIMELINE_CACHE: OnceLock<Mutex<HashMap<String, TimelineSnapshot>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct TimelineSnapshot {
    position_ms: u64,
    duration_ms: u64,
}

fn stabilize_timeline(
    previous: Option<TimelineSnapshot>,
    reported: TimelineSnapshot,
    is_playing: bool,
) -> TimelineSnapshot {
    let Some(previous) = previous else {
        return reported;
    };
    let effective_duration = if reported.duration_ms > 0 {
        reported.duration_ms
    } else {
        previous.duration_ms
    };
    let previous_position = if effective_duration > 0 {
        previous.position_ms.min(effective_duration)
    } else {
        previous.position_ms
    };
    let reported_position = if effective_duration > 0 {
        reported.position_ms.min(effective_duration)
    } else {
        reported.position_ms
    };
    TimelineSnapshot {
        position_ms: if !is_playing {
            previous_position
        } else if reported_position < 1_000 && previous_position > 0 {
            // Some SMTC providers briefly publish a zero timeline while they
            // refresh an actively playing session. Keep the last valid value;
            // the frontend clock will continue advancing from it until a
            // usable snapshot arrives.
            previous_position
        } else {
            reported_position
        },
        // A missing duration is transient for several desktop players and
        // must never replace a known duration, regardless of playback state.
        duration_ms: effective_duration,
    }
}

fn normalize_media_text(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn candidate_score(title: &str, artist: &str, candidate: &CoverCandidate) -> u8 {
    let wanted_title = normalize_media_text(title);
    let wanted_artist = normalize_media_text(artist);
    let found_title = normalize_media_text(&candidate.title);
    let found_artist = normalize_media_text(&candidate.artist);
    if wanted_title.is_empty() {
        return 0;
    }
    let title_score = if wanted_title == found_title {
        60
    } else if wanted_title.contains(&found_title) || found_title.contains(&wanted_title) {
        40
    } else {
        0
    };
    let artist_score = if wanted_artist.is_empty() {
        0
    } else if wanted_artist == found_artist {
        40
    } else if wanted_artist.contains(&found_artist) || found_artist.contains(&wanted_artist) {
        25
    } else {
        0
    };
    title_score + artist_score
}

fn best_cover_candidate(
    title: &str,
    artist: &str,
    candidates: Vec<CoverCandidate>,
) -> Option<CoverCandidate> {
    let threshold = if normalize_media_text(artist).is_empty() {
        60
    } else {
        COVER_MATCH_THRESHOLD
    };
    candidates
        .into_iter()
        .map(|candidate| (candidate_score(title, artist, &candidate), candidate))
        .filter(|(score, _)| *score >= threshold)
        .max_by_key(|(score, _)| *score)
        .map(|(_, candidate)| candidate)
}

fn netease_hd_url(url: &str) -> String {
    let base_url = url.split('?').next().unwrap_or(url);
    format!("{base_url}?param=1200y1200")
}

fn apple_hd_url(url: &str) -> String {
    url.replace("100x100bb", "1200x1200bb")
        .replace("100x100-75", "1200x1200-75")
}

async fn download_cover_as_data_url(client: &reqwest::Client, url: &str) -> AppResult<String> {
    const MAX_COVER_BYTES: u64 = 12 * 1024 * 1024;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| AppError::network(format!("Cover download failed: {error}")))?
        .error_for_status()
        .map_err(|error| AppError::network(format!("Cover download status failed: {error}")))?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_COVER_BYTES)
    {
        return Err(AppError::business(3004, "封面图片过大"));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| AppError::network(format!("Cover bytes failed: {error}")))?;
    if bytes.len() as u64 > MAX_COVER_BYTES {
        return Err(AppError::business(3004, "封面图片过大"));
    }
    let format = image::guess_format(&bytes)
        .map_err(|error| AppError::parse(format!("Cover image format failed: {error}")))?;
    let mime = match format {
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::WebP => "image/webp",
        image::ImageFormat::Gif => "image/gif",
        image::ImageFormat::Bmp => "image/bmp",
        _ => return Err(AppError::business(3002, "不支持的封面图片格式")),
    };
    Ok(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(&bytes)
    ))
}

fn allow_apple_request() -> bool {
    let now = Instant::now();
    let Ok(mut requests) = APPLE_REQUESTS
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
    else {
        return false;
    };
    while requests
        .front()
        .map(|started| now.duration_since(*started) >= Duration::from_secs(60))
        .unwrap_or(false)
    {
        requests.pop_front();
    }
    if requests.len() >= APPLE_REQUESTS_PER_MINUTE {
        return false;
    }
    requests.push_back(now);
    true
}

fn cover_provider_order(source: &str) -> [&'static str; 2] {
    if matches!(source.to_lowercase().as_str(), "apple" | "apple_music") {
        ["apple", "netease"]
    } else {
        ["netease", "apple"]
    }
}

async fn cover_from_netease(
    client: &reqwest::Client,
    title: &str,
    artist: &str,
) -> AppResult<Option<ResolvedCover>> {
    // The legacy search endpoint often ranks covers and similarly named songs
    // above the original recording. Search more than the first five entries,
    // and retry without the artist because the service tokenizes CJK queries
    // inconsistently. The score threshold below still prevents loose matches.
    let combined_query = format!("{title} {artist}");
    let queries = if artist.trim().is_empty() {
        vec![title.to_string()]
    } else {
        vec![combined_query, title.to_string()]
    };
    let mut candidate = None;
    for query in queries {
        let keyword = urlencoding::encode(query.trim());
        let url =
            format!("https://music.163.com/api/search/get/?s={keyword}&type=1&limit=30&offset=0");
        let json: Value = client
            .get(url)
            .header(reqwest::header::REFERER, "https://music.163.com/")
            .send()
            .await
            .map_err(|error| AppError::network(format!("Netease cover request failed: {error}")))?
            .error_for_status()
            .map_err(|error| AppError::network(format!("Netease cover status failed: {error}")))?
            .json()
            .await
            .map_err(|error| AppError::parse(format!("Netease cover response failed: {error}")))?;
        let candidates = json["result"]["songs"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|song| {
                let candidate_title = song["name"].as_str()?.to_string();
                // Both shapes exist in responses from NetEase's desktop and
                // web endpoints: artists/album and ar/al.
                let artists = song["artists"].as_array().or_else(|| song["ar"].as_array());
                let candidate_artist = artists
                    .into_iter()
                    .flatten()
                    .filter_map(|item| item["name"].as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                let album = song.get("album").or_else(|| song.get("al"));
                let cover_url = album
                    .and_then(|value| value["picUrl"].as_str())
                    .unwrap_or_default()
                    .to_string();
                Some(CoverCandidate {
                    title: candidate_title,
                    artist: candidate_artist,
                    url: cover_url,
                    source_id: song["id"].as_u64(),
                })
            })
            .collect();
        candidate = best_cover_candidate(title, artist, candidates);
        if candidate.is_some() {
            break;
        }
    }
    let Some(candidate) = candidate else {
        return Ok(None);
    };
    let cover_url = if candidate.url.is_empty() {
        let Some(song_id) = candidate.source_id else {
            return Ok(None);
        };
        let detail_url =
            format!("https://music.163.com/api/song/detail/?id={song_id}&ids=%5B{song_id}%5D");
        let detail: Value = client
            .get(detail_url)
            .header(reqwest::header::REFERER, "https://music.163.com/")
            .send()
            .await
            .map_err(|error| AppError::network(format!("Netease detail failed: {error}")))?
            .error_for_status()
            .map_err(|error| AppError::network(format!("Netease detail status failed: {error}")))?
            .json()
            .await
            .map_err(|error| AppError::parse(format!("Netease detail response failed: {error}")))?;
        detail["songs"][0]["album"]["picUrl"]
            .as_str()
            .or_else(|| detail["songs"][0]["album"]["blurPicUrl"].as_str())
            .unwrap_or_default()
            .to_string()
    } else {
        candidate.url
    };
    if cover_url.is_empty() {
        return Ok(None);
    }
    Ok(Some(ResolvedCover {
        url: netease_hd_url(&cover_url),
        provider: "netease".to_string(),
    }))
}

async fn cover_from_apple(
    client: &reqwest::Client,
    title: &str,
    artist: &str,
) -> AppResult<Option<ResolvedCover>> {
    if !allow_apple_request() {
        return Ok(None);
    }
    let query = format!("{title} {artist}");
    let term = urlencoding::encode(&query);
    let url = format!(
        "https://itunes.apple.com/search?term={term}&country=US&media=music&entity=song&limit=5"
    );
    let json: Value = client
        .get(url)
        .send()
        .await
        .map_err(|error| AppError::network(format!("Apple cover request failed: {error}")))?
        .error_for_status()
        .map_err(|error| AppError::network(format!("Apple cover status failed: {error}")))?
        .json()
        .await
        .map_err(|error| AppError::parse(format!("Apple cover response failed: {error}")))?;
    let candidates = json["results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some(CoverCandidate {
                title: item["trackName"].as_str()?.to_string(),
                artist: item["artistName"].as_str()?.to_string(),
                url: item["artworkUrl100"].as_str()?.to_string(),
                source_id: item["trackId"].as_u64(),
            })
        })
        .collect();
    Ok(
        best_cover_candidate(title, artist, candidates).map(|candidate| ResolvedCover {
            url: apple_hd_url(&candidate.url),
            provider: "apple".to_string(),
        }),
    )
}

pub async fn resolve_hd_cover(
    title: &str,
    artist: &str,
    source: &str,
) -> AppResult<Option<ResolvedCover>> {
    let cache_key = format!(
        "{}|{}|{}",
        normalize_media_text(title),
        normalize_media_text(artist),
        source.to_lowercase()
    );
    if cache_key.starts_with('|') {
        return Ok(None);
    }
    if let Ok(cache) = COVER_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        if let Some((created, value)) = cache.get(&cache_key) {
            let ttl = if value.is_some() {
                COVER_CACHE_TTL
            } else {
                COVER_NEGATIVE_CACHE_TTL
            };
            if created.elapsed() < ttl {
                return Ok(value.clone());
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("Isle/0.1 album-art resolver")
        .build()
        .map_err(|error| AppError::network(format!("Cover client failed: {error}")))?;
    let mut resolved = None;
    for provider in cover_provider_order(source) {
        let attempt = match provider {
            "apple" => cover_from_apple(&client, title, artist).await,
            _ => cover_from_netease(&client, title, artist).await,
        };
        if let Ok(Some(mut cover)) = attempt {
            match crate::services::cache::download_and_cache(&cover.url, "image/jpeg").await {
                Ok(cached_path) => {
                    cover.url = cached_path;
                    resolved = Some(cover);
                    break;
                }
                Err(error) => {
                    // Keep high-resolution artwork working if the install
                    // directory is temporarily unavailable.
                    match download_cover_as_data_url(&client, &cover.url).await {
                        Ok(data_url) => {
                            cover.url = data_url;
                            resolved = Some(cover);
                            break;
                        }
                        Err(fallback_error) => tracing::warn!(
                            "[高清封面] {} 图片缓存失败: {}; 回退下载失败: {}",
                            provider,
                            error,
                            fallback_error
                        ),
                    }
                }
            }
        }
    }
    if let Ok(mut cache) = COVER_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        cache.insert(cache_key, (Instant::now(), resolved.clone()));
    }
    Ok(resolved)
}

fn source_for_id(raw_id: &str) -> &'static str {
    let id = raw_id.to_lowercase();
    if id.contains("cloudmusic") {
        "netease"
    } else if id.contains("spotify") {
        "spotify"
    } else if id.contains("bilibili") {
        "bilibili"
    } else if id.contains("qqmusic") {
        "qqmusic"
    } else if id.contains("apple") && id.contains("music") {
        "apple"
    } else {
        "generic"
    }
}

fn source_display(source: &str, raw_id: &str) -> String {
    match source {
        "netease" => "网易云音乐".to_string(),
        "spotify" => "Spotify".to_string(),
        "bilibili" => "Bilibili".to_string(),
        "qqmusic" => "QQ 音乐".to_string(),
        "apple" => "Apple Music".to_string(),
        _ => raw_id
            .rsplit(['!', '\\'])
            .next()
            .unwrap_or(raw_id)
            .trim_end_matches(".exe")
            .to_string(),
    }
}

fn selected_player_ids(app: &AppHandle) -> Option<Vec<String>> {
    app.state::<AppState>()
        .settings
        .lock()
        .ok()
        .and_then(|settings| settings.selected_player_ids.clone())
}

fn player_order_ids(app: &AppHandle) -> Vec<String> {
    app.state::<AppState>()
        .settings
        .lock()
        .ok()
        .map(|settings| settings.player_order_ids.clone())
        .unwrap_or_default()
}

fn selected_session(
    app: &AppHandle,
) -> AppResult<Option<(GlobalSystemMediaTransportControlsSession, String, String)>> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|op| op.get())
        .map_err(|e| AppError::media(format!("RequestAsync failed: {:?}", e)))?;
    let sessions = manager
        .GetSessions()
        .map_err(|e| AppError::media(format!("GetSessions failed: {:?}", e)))?;
    let filter = selected_player_ids(app);
    if matches!(filter, Some(ref ids) if ids.is_empty()) {
        return Ok(None);
    }
    let mut candidates = Vec::new();
    for index in 0..sessions.Size().unwrap_or(0) {
        if let Ok(session) = sessions.GetAt(index) {
            let raw_id = session
                .SourceAppUserModelId()
                .unwrap_or_default()
                .to_string();
            let playing = session
                .GetPlaybackInfo()
                .ok()
                .and_then(|info| info.PlaybackStatus().ok())
                .map(|status| status.0 == 4)
                .unwrap_or(false);
            let updated = session
                .GetTimelineProperties()
                .ok()
                .and_then(|timeline| timeline.LastUpdatedTime().ok())
                .map(|time| time.UniversalTime)
                .unwrap_or(i64::MIN);
            candidates.push((session, raw_id, playing, updated));
        }
    }
    if let Some(ids) = filter {
        for selected_id in ids {
            if let Some((session, raw_id, _, _)) = candidates
                .iter()
                .filter(|(_, id, _, _)| id == &selected_id)
                .max_by_key(|(_, _, playing, updated)| (*playing, *updated))
            {
                let source = source_for_id(raw_id).to_string();
                return Ok(Some((session.clone(), source, raw_id.clone())));
            }
        }
        return Ok(None);
    }
    let order = player_order_ids(app);
    if !order.is_empty() {
        for ordered_id in order {
            if let Some((session, raw_id, _, _)) = candidates
                .iter()
                .filter(|(_, id, _, _)| id == &ordered_id)
                .max_by_key(|(_, _, playing, updated)| (*playing, *updated))
            {
                let source = source_for_id(raw_id).to_string();
                return Ok(Some((session.clone(), source, raw_id.clone())));
            }
        }
    }
    let previous_id = LAST_AUTO_SESSION_ID
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|value| value.clone());
    let selected = candidates
        .iter()
        .filter(|(_, _, playing, _)| *playing)
        .max_by_key(|(_, _, _, updated)| *updated)
        .or_else(|| {
            previous_id
                .as_ref()
                .and_then(|id| candidates.iter().find(|(_, raw_id, _, _)| raw_id == id))
        })
        .or_else(|| candidates.iter().max_by_key(|(_, _, _, updated)| *updated))
        .map(|(session, raw_id, _, _)| (session.clone(), raw_id.clone()));
    if let Some((_, raw_id)) = &selected {
        if let Ok(mut previous) = LAST_AUTO_SESSION_ID.get_or_init(|| Mutex::new(None)).lock() {
            *previous = Some(raw_id.clone());
        }
    }
    Ok(selected.map(|(session, raw_id)| {
        let source = source_for_id(&raw_id).to_string();
        (session, source, raw_id)
    }))
}

pub fn list_media_sessions() -> AppResult<Vec<MediaSessionInfo>> {
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
    let mut result = Vec::new();
    for index in 0..sessions.Size().unwrap_or(0) {
        if let Ok(session) = sessions.GetAt(index) {
            let id = session
                .SourceAppUserModelId()
                .unwrap_or_default()
                .to_string();
            if id.is_empty() || result.iter().any(|entry: &MediaSessionInfo| entry.id == id) {
                continue;
            }
            let is_playing = session
                .GetPlaybackInfo()
                .ok()
                .and_then(|info| info.PlaybackStatus().ok())
                .map(|status| status.0 == 4)
                .unwrap_or(false);
            let source = source_for_id(&id).to_string();
            result.push(MediaSessionInfo {
                display_name: source_display(&source, &id),
                id,
                source,
                is_playing,
            });
        }
    }
    Ok(result)
}

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
    let (session, source_type, raw_id) = match selected_session(app)? {
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
                    if size > 0
                        && reader
                            .LoadAsync(size)
                            .map_err(|e| AppError::media(e.to_string()))?
                            .get()
                            .is_ok()
                    {
                        let mut buffer = vec![0u8; size as usize];
                        if reader.ReadBytes(&mut buffer).is_ok() {
                            thumbnail_base64 = format!(
                                "data:image/png;base64,{}",
                                general_purpose::STANDARD.encode(&buffer)
                            );
                        }
                    }
                }
            }
        }

        // 更新缓存
        if !thumbnail_base64.is_empty() {
            let state = app.state::<AppState>();
            if let Ok(mut cache) = state.media_cache.lock() {
                cache.track_id = track_id.clone();
                cache.base64_img = thumbnail_base64.clone();
            };
        }
    }

    // 获取播放状态
    let playback_info = session
        .GetPlaybackInfo()
        .map_err(|e| AppError::media(format!("GetPlaybackInfo failed: {:?}", e)))?;
    let playback_status = playback_info
        .PlaybackStatus()
        .unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus(0));

    let capabilities = playback_info
        .Controls()
        .map(|controls| MediaCapabilities {
            previous: controls.IsPreviousEnabled().unwrap_or(false),
            play_pause: controls.IsPlayPauseToggleEnabled().unwrap_or(false),
            next: controls.IsNextEnabled().unwrap_or(false),
            seek: controls.IsPlaybackPositionEnabled().unwrap_or(false),
            shuffle: controls.IsShuffleEnabled().unwrap_or(false),
            repeat: controls.IsRepeatEnabled().unwrap_or(false),
        })
        .unwrap_or_default();

    let is_playing = playback_status.0 == 4; // Playing = 4

    // 计算实际播放位置（考虑时间差）
    let dur_ms = (timeline.EndTime().unwrap_or_default().Duration / 10000) as u64;
    let snapshot_pos_ms = (timeline.Position().unwrap_or_default().Duration / 10000) as u64;
    // Position 已经是 SMTC 的当前快照。LastUpdatedTime 在部分播放器中是
    // 曲目创建时间或旧时间戳，用它再次外推会直接把进度推到 100%。
    // 前端收到快照后会从接收时刻继续本地计时，因此这里不再重复外推。
    let real_pos_ms = if dur_ms > 0 {
        snapshot_pos_ms.min(dur_ms)
    } else {
        snapshot_pos_ms
    };

    // 处理直播流（时长为0或超大）
    let is_live_logic = dur_ms > 360000000;
    let (position_ms, duration_ms) = if is_live_logic {
        (0u64, 0u64)
    } else {
        (real_pos_ms, dur_ms)
    };
    let timeline_key = format!(
        "{}|{}|{}",
        raw_id.to_lowercase(),
        normalize_media_text(&title),
        normalize_media_text(&artist)
    );
    let reported_timeline = TimelineSnapshot {
        position_ms,
        duration_ms,
    };
    let timeline = if let Ok(mut cache) = TIMELINE_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        let stabilized = stabilize_timeline(
            cache.get(&timeline_key).copied(),
            reported_timeline,
            is_playing,
        );
        if cache.len() >= 256 && !cache.contains_key(&timeline_key) {
            cache.clear();
        }
        cache.insert(timeline_key, stabilized);
        stabilized
    } else {
        reported_timeline
    };

    Ok(MediaState {
        title,
        artist,
        album_art: thumbnail_base64,
        is_playing,
        position_ms: timeline.position_ms,
        duration_ms: timeline.duration_ms,
        last_updated_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        source: source_type,
        source_display: raw_id,
        capabilities,
        shuffle_active: playback_info
            .IsShuffleActive()
            .ok()
            .and_then(|value| value.Value().ok()),
        repeat_mode: playback_info.AutoRepeatMode().ok().and_then(|value| {
            value.Value().ok().map(|mode| match mode.0 {
                1 => "track".to_string(),
                2 => "list".to_string(),
                _ => "none".to_string(),
            })
        }),
    })
}

/// 从网易云音乐 API 获取歌曲信息
///
/// 通过歌曲名称和艺术家搜索，返回：
/// - 歌曲时长
/// - 专辑封面 URL
/// - MV ID 和播放 URL
pub async fn get_netease_song_info(
    song_name: &str,
    artist: &str,
) -> AppResult<Option<NeteaseSong>> {
    let keyword = format!("{} {}", artist, song_name);
    let encoded_keyword = urlencoding::encode(&keyword);
    let url = format!(
        "https://music.163.com/api/search/get/?s={}&type=1&limit=1",
        encoded_keyword
    );

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

    let Some((session, _, _)) = selected_session(app)? else {
        return Err(AppError::media("未找到可控制的媒体会话"));
    };

    // GSMTC control methods return an IAsyncOperation.  Dropping that
    // operation immediately makes the request fire-and-forget; in practice
    // Windows can cancel it before the player receives the command.  Wait for
    // completion so the frontend gets the real result and can restore its
    // optimistic state when the player rejects the action.
    let result = match action {
        "play_pause" => session
            .TryTogglePlayPauseAsync()
            .and_then(|operation| operation.get()),
        "next" => session
            .TrySkipNextAsync()
            .and_then(|operation| operation.get()),
        "prev" => session
            .TrySkipPreviousAsync()
            .and_then(|operation| operation.get()),
        _ => return Err(AppError::media(format!("不支持的媒体操作: {action}"))),
    };

    result.map_err(|error| AppError::media(format!("执行媒体操作 {action} 失败: {error:?}")))?;
    Ok(())
}

pub fn seek_media(app: &AppHandle, position_ms: u64) -> AppResult<()> {
    if let Some((session, _, _)) = selected_session(app)? {
        session
            .TryChangePlaybackPositionAsync(
                (position_ms.saturating_mul(10_000)).min(i64::MAX as u64) as i64,
            )
            .and_then(|op| op.get())
            .map_err(|e| AppError::media(format!("Seek failed: {:?}", e)))?;
    }
    Ok(())
}

pub fn toggle_shuffle(app: &AppHandle) -> AppResult<()> {
    if let Some((session, _, _)) = selected_session(app)? {
        let current = session
            .GetPlaybackInfo()
            .ok()
            .and_then(|info| info.IsShuffleActive().ok())
            .and_then(|value| value.Value().ok())
            .unwrap_or(false);
        session
            .TryChangeShuffleActiveAsync(!current)
            .and_then(|op| op.get())
            .map_err(|e| AppError::media(format!("Shuffle failed: {:?}", e)))?;
    }
    Ok(())
}

pub fn cycle_repeat(app: &AppHandle) -> AppResult<()> {
    if let Some((session, _, _)) = selected_session(app)? {
        let current = session
            .GetPlaybackInfo()
            .ok()
            .and_then(|info| info.AutoRepeatMode().ok())
            .and_then(|value| value.Value().ok())
            .unwrap_or(MediaPlaybackAutoRepeatMode(0));
        let next = match current.0 {
            0 => MediaPlaybackAutoRepeatMode(2),
            2 => MediaPlaybackAutoRepeatMode(1),
            _ => MediaPlaybackAutoRepeatMode(0),
        };
        session
            .TryChangeAutoRepeatModeAsync(next)
            .and_then(|op| op.get())
            .map_err(|e| AppError::media(format!("Repeat failed: {:?}", e)))?;
    }
    Ok(())
}

#[cfg(test)]
mod cover_tests {
    use super::{
        apple_hd_url, best_cover_candidate, candidate_score, cover_provider_order, netease_hd_url,
        normalize_media_text, stabilize_timeline, CoverCandidate, TimelineSnapshot,
    };

    fn candidate(title: &str, artist: &str, url: &str) -> CoverCandidate {
        CoverCandidate {
            title: title.to_string(),
            artist: artist.to_string(),
            url: url.to_string(),
            source_id: None,
        }
    }

    #[test]
    fn normalizes_punctuation_and_case_for_matching() {
        assert_eq!(
            normalize_media_text("Midnight City (Live)"),
            "midnightcitylive"
        );
        assert_eq!(
            candidate_score(
                "Midnight City",
                "M83",
                &candidate("Midnight City", "M83", "cover")
            ),
            100
        );
    }

    #[test]
    fn orders_cover_providers_by_media_source() {
        assert_eq!(cover_provider_order("apple"), ["apple", "netease"]);
        assert_eq!(cover_provider_order("apple_music"), ["apple", "netease"]);
        assert_eq!(cover_provider_order("netease"), ["netease", "apple"]);
        assert_eq!(cover_provider_order("spotify"), ["netease", "apple"]);
    }

    #[test]
    fn keeps_last_timeline_when_smtc_snapshot_resets_to_zero() {
        let previous = TimelineSnapshot {
            position_ms: 42_500,
            duration_ms: 180_000,
        };
        let reset = TimelineSnapshot::default();
        assert_eq!(stabilize_timeline(Some(previous), reset, false), previous);
        assert_eq!(stabilize_timeline(Some(previous), reset, true), previous);
        let stale = TimelineSnapshot {
            position_ms: 12_000,
            duration_ms: 180_000,
        };
        assert_eq!(stabilize_timeline(Some(previous), stale, false), previous);
        assert_eq!(stabilize_timeline(Some(previous), stale, true), stale);
    }

    #[test]
    fn keeps_known_duration_during_a_playing_metadata_gap() {
        let previous = TimelineSnapshot {
            position_ms: 42_500,
            duration_ms: 180_000,
        };
        let missing_duration = TimelineSnapshot {
            position_ms: 43_000,
            duration_ms: 0,
        };
        assert_eq!(
            stabilize_timeline(Some(previous), missing_duration, true),
            TimelineSnapshot {
                position_ms: 43_000,
                duration_ms: 180_000,
            }
        );
    }

    #[test]
    fn rejects_wrong_artist_and_selects_confident_candidate() {
        let selected = best_cover_candidate(
            "同名歌曲",
            "正确歌手",
            vec![
                candidate("同名歌曲", "其他歌手", "wrong"),
                candidate("同名歌曲 Live", "正确歌手", "right"),
            ],
        )
        .expect("matching cover");
        assert_eq!(selected.url, "right");
    }

    #[test]
    fn accepts_an_exact_title_when_the_player_omits_artist() {
        let selected = best_cover_candidate(
            "Midnight City",
            "",
            vec![candidate("Midnight City", "M83", "cover")],
        )
        .expect("title-only matching cover");
        assert_eq!(selected.url, "cover");
    }

    #[test]
    fn upgrades_provider_artwork_urls() {
        assert_eq!(
            netease_hd_url("https://p.example/cover.jpg"),
            "https://p.example/cover.jpg?param=1200y1200"
        );
        assert_eq!(
            netease_hd_url("https://p.example/cover.jpg?param=100y100"),
            "https://p.example/cover.jpg?param=1200y1200"
        );
        assert!(apple_hd_url("https://a.example/100x100bb.jpg").contains("1200x1200bb"));
    }
}
