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
use windows052::{
    Media::{Control::*, MediaPlaybackAutoRepeatMode},
    Storage::Streams::*,
};

const COVER_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const COVER_NEGATIVE_CACHE_TTL: Duration = Duration::from_secs(30);
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
    _is_playing: bool,
) -> TimelineSnapshot {
    let Some(previous) = previous else {
        return reported;
    };
    let effective_duration = if reported.duration_ms > 0 {
        reported.duration_ms
    } else {
        previous.duration_ms
    };
    let reported_position = if effective_duration > 0 {
        reported.position_ms.min(effective_duration)
    } else {
        reported.position_ms
    };
    TimelineSnapshot {
        position_ms: reported_position,
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
    let directory = std::env::temp_dir().join("isle-artwork-downloads");
    std::fs::create_dir_all(&directory)?;
    super::cache::cleanup_dead_partials(&directory);
    let downloaded = super::http_budget::download(
        client,
        url,
        &directory,
        super::image_budget::MAX_IMAGE_BYTES as u64,
    )
    .await?;
    let bytes = super::image_budget::read_limited(std::fs::File::open(&downloaded.path)?)?;
    let prepared = super::image_budget::prepare_artwork(&bytes)?;
    Ok(format!(
        "data:{};base64,{}",
        prepared.content_type,
        general_purpose::STANDARD.encode(&prepared.bytes)
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

async fn matched_netease_song(
    client: &reqwest::Client,
    title: &str,
    artist: &str,
) -> AppResult<Option<Value>> {
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
    let mut matched_song = None;
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
        if let Some(ref selected) = candidate {
            matched_song = json["result"]["songs"].as_array().and_then(|songs| {
                songs
                    .iter()
                    .find(|song| song["id"].as_u64() == selected.source_id)
                    .cloned()
            });
            break;
        }
    }
    let Some(candidate) = candidate else {
        return Ok(None);
    };
    let Some(song_id) = candidate.source_id else {
        return Ok(matched_song);
    };
    let detail_url =
        format!("https://music.163.com/api/song/detail/?id={song_id}&ids=%5B{song_id}%5D");
    // Search responses often omit artwork. Detail failure must not discard duration.
    if let Ok(response) = client
        .get(detail_url)
        .header(reqwest::header::REFERER, "https://music.163.com/")
        .send()
        .await
    {
        if let Ok(response) = response.error_for_status() {
            if let Ok(detail) = response.json::<Value>().await {
                if let Some(song) = detail["songs"].as_array().and_then(|songs| {
                    songs
                        .iter()
                        .find(|song| song["id"].as_u64() == Some(song_id))
                }) {
                    return Ok(Some(song.clone()));
                }
            }
        }
    }
    Ok(matched_song)
}

fn parse_netease_song(song: &Value) -> NeteaseSong {
    let album = song.get("album").or_else(|| song.get("al"));
    NeteaseSong {
        duration: song["duration"]
            .as_u64()
            .filter(|value| *value > 0)
            .or_else(|| song["dt"].as_u64()),
        album_pic: album
            .and_then(|album| {
                album["picUrl"]
                    .as_str()
                    .filter(|url| !url.is_empty())
                    .or_else(|| album["blurPicUrl"].as_str())
            })
            .filter(|url| !url.is_empty())
            .map(netease_hd_url),
        mv_id: song["mv"].as_i64().or_else(|| song["mvid"].as_i64()),
        mv_url: None,
    }
}

async fn cover_from_netease(
    client: &reqwest::Client,
    title: &str,
    artist: &str,
) -> AppResult<Option<ResolvedCover>> {
    Ok(matched_netease_song(client, title, artist)
        .await?
        .and_then(|song| parse_netease_song(&song).album_pic)
        .map(|url| ResolvedCover {
            url,
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
    resolve_hd_cover_versioned(title, artist, source, "").await
}
pub async fn resolve_hd_cover_versioned(
    title: &str,
    artist: &str,
    source: &str,
    version: &str,
) -> AppResult<Option<ResolvedCover>> {
    let cache_key = format!(
        "{}|{}|{}|{}",
        normalize_media_text(title),
        normalize_media_text(artist),
        source.to_lowercase(),
        version
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
    let mut failed = false;
    for provider in cover_provider_order(source) {
        let attempt = match provider {
            "apple" => cover_from_apple(&client, title, artist).await,
            _ => cover_from_netease(&client, title, artist).await,
        };
        if attempt.is_err() {
            failed = true;
        }
        if let Ok(Some(mut cover)) = attempt {
            if !version.is_empty() {
                let mut url = reqwest::Url::parse(&cover.url)
                    .map_err(|_| AppError::parse("封面 URL 无效"))?;
                url.set_fragment(Some(version));
                cover.url = url.to_string();
            }
            match crate::services::cache::download_and_cache(&cover.url, "image/jpeg").await {
                Ok(cached_path) => {
                    cover.url = cached_path;
                    resolved = Some(cover);
                    break;
                }
                Err(error) => {
                    failed = true;
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
    super::artwork_jobs::checkpoint()?;
    if resolved.is_none() && failed {
        return Err(AppError::network("高清封面请求失败，请重试"));
    }
    if let Ok(mut cache) = COVER_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        cache.retain(|_, (created, value)| {
            created.elapsed()
                < if value.is_some() {
                    COVER_CACHE_TTL
                } else {
                    COVER_NEGATIVE_CACHE_TTL
                }
        });
        let incoming_bytes = cache_key.len()
            + resolved
                .as_ref()
                .map_or(0, |cover| cover.url.len() + cover.provider.len());
        while !cache.is_empty()
            && (cache.len() >= 64
                || cache
                    .iter()
                    .map(|(key, (_, value))| {
                        key.len()
                            + value
                                .as_ref()
                                .map_or(0, |cover| cover.url.len() + cover.provider.len())
                    })
                    .fold(0usize, usize::saturating_add)
                    .saturating_add(incoming_bytes)
                    > 8 * 1024 * 1024)
        {
            let oldest = cache
                .iter()
                .min_by_key(|(_, (created, _))| *created)
                .map(|(key, _)| key.clone());
            if let Some(key) = oldest {
                cache.remove(&key);
            }
        }
        if incoming_bytes <= 8 * 1024 * 1024 {
            cache.insert(cache_key, (Instant::now(), resolved.clone()));
        }
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

fn source_with_override(raw_id: &str, override_source: Option<&str>) -> &'static str {
    let detected = source_for_id(raw_id);
    // An explicit per-session choice may classify an unknown Windows ID.
    // Keep known player identities and the original SMTC selection ID intact.
    if detected == "generic" && override_source == Some("netease") {
        "netease"
    } else {
        detected
    }
}

fn session_source(app: &AppHandle, raw_id: &str) -> &'static str {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().ok();
    source_with_override(
        raw_id,
        settings.as_ref().and_then(|settings| {
            settings
                .player_source_overrides
                .get(raw_id)
                .map(String::as_str)
        }),
    )
}

fn unique_netease_binding(
    candidates: &[(String, String, String, String)],
    title: &str,
    artist: &str,
) -> Option<String> {
    let matches: Vec<_> = candidates
        .iter()
        .filter(|(_, source, candidate_title, candidate_artist)| {
            source == "netease"
                || (source == "generic"
                    && !title.is_empty()
                    && candidate_title == title
                    && candidate_artist == artist)
        })
        .collect();
    if matches.len() == 1 {
        Some(matches[0].0.clone())
    } else {
        None
    }
}

/// Invoked only by explicit connection setup, never in periodic media sampling.
pub(crate) fn verified_netease_binding() -> Option<String> {
    let _apartment = super::apartment::Apartment::enter().ok()?;
    let progress = super::netease_cdp::LocalCdpClient::default()
        .read_progress_snapshot()
        .ok()?;
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .ok()?
        .get()
        .ok()?;
    let sessions = manager.GetSessions().ok()?;
    let size = sessions.Size().ok()?;
    if size > 64 {
        return None;
    }
    let mut candidates = Vec::new();
    for index in 0..size {
        let session = sessions.GetAt(index).ok()?;
        let id = session.SourceAppUserModelId().ok()?.to_string();
        let source = source_for_id(&id);
        if !matches!(source, "generic" | "netease") {
            continue;
        }
        let properties = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
        candidates.push((
            id,
            source.to_string(),
            properties.Title().ok()?.to_string(),
            properties.Artist().ok()?.to_string(),
        ));
    }
    unique_netease_binding(&candidates, &progress.title, &progress.artist)
}

fn source_display(source: &str, raw_id: &str) -> String {
    match source {
        "netease" => "网易云音乐".to_string(),
        "spotify" => "Spotify".to_string(),
        "bilibili" => "Bilibili".to_string(),
        "qqmusic" => "QQ 音乐".to_string(),
        "apple" => "Apple Music".to_string(),
        _ => {
            let lower = raw_id.to_ascii_lowercase();
            if lower.contains("zunemusic") {
                return "Windows 媒体播放器".to_string();
            }
            if lower.starts_with("chrome") {
                return browser_display("Chrome", raw_id);
            }
            if lower.starts_with("msedge") {
                return browser_display("Microsoft Edge", raw_id);
            }
            let app = raw_id
                .split('!')
                .next()
                .unwrap_or(raw_id)
                .rsplit('\\')
                .next()
                .unwrap_or(raw_id);
            if raw_id.contains('!') {
                return app.split('_').next().unwrap_or(app).replace('.', " ");
            }
            app.trim_end_matches(".exe").to_string()
        }
    }
}

fn browser_display(name: &str, raw_id: &str) -> String {
    let lower = raw_id.to_ascii_lowercase();
    if let Some(profile) = lower.find(".profile") {
        let number = raw_id[profile + ".profile".len()..]
            .split(|ch: char| !ch.is_ascii_digit())
            .next()
            .unwrap_or("");
        if !number.is_empty() {
            return format!("{name} · 个人资料 {number}");
        }
    }
    name.to_string()
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

fn ordered_selected_ids<'a>(order: &'a [String], selected: &'a [String]) -> Vec<&'a str> {
    order
        .iter()
        .filter(|id| selected.contains(id))
        .chain(selected.iter().filter(|id| !order.contains(id)))
        .map(String::as_str)
        .collect()
}

fn selected_session(
    app: &AppHandle,
    manager: &GlobalSystemMediaTransportControlsSessionManager,
) -> AppResult<Option<(GlobalSystemMediaTransportControlsSession, String, String)>> {
    let sessions = manager
        .GetSessions()
        .map_err(|e| AppError::media(format!("GetSessions failed: {:?}", e)))?;
    let filter = selected_player_ids(app);
    let order = player_order_ids(app);
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
        // The visible priority is player_order_ids. selected_player_ids is only
        // the allowlist; older settings may contain IDs absent from the order.
        for selected_id in ordered_selected_ids(&order, &ids) {
            if let Some((session, raw_id, _, _)) = candidates
                .iter()
                .filter(|(_, id, _, _)| id == selected_id)
                .max_by_key(|(_, _, playing, updated)| (*playing, *updated))
            {
                let source = session_source(app, raw_id).to_string();
                return Ok(Some((session.clone(), source, raw_id.clone())));
            }
        }
        return Ok(None);
    }
    if !order.is_empty() {
        for ordered_id in order {
            if let Some((session, raw_id, _, _)) = candidates
                .iter()
                .filter(|(_, id, _, _)| id == &ordered_id)
                .max_by_key(|(_, _, playing, updated)| (*playing, *updated))
            {
                let source = session_source(app, raw_id).to_string();
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
        let source = session_source(app, &raw_id).to_string();
        (session, source, raw_id)
    }))
}

const LISTENER_METADATA_TTL: Duration = Duration::from_secs(5);

#[derive(Clone, Default)]
struct MediaMetadata {
    title: String,
    artist: String,
    album_art: String,
}

/// Owned by the media listener's COM apartment and bounded to its one selected session.
#[derive(Default)]
pub(crate) struct MediaListenerMetadataCache {
    session: Option<GlobalSystemMediaTransportControlsSession>,
    metadata: Option<MediaMetadata>,
    refreshed_at: Option<Instant>,
}

impl MediaListenerMetadataCache {
    pub(crate) fn clear(&mut self) {
        self.session = None;
        self.metadata = None;
        self.refreshed_at = None;
    }
}

pub(crate) struct MediaListenerSnapshot {
    pub(crate) result: AppResult<MediaState>,
    pub(crate) metadata_read_attempted: bool,
    pub(crate) metadata_updated: bool,
    pub(crate) metadata_reused: bool,
}

pub(crate) fn media_sessions_have_same_identity(
    left: &GlobalSystemMediaTransportControlsSession,
    right: &GlobalSystemMediaTransportControlsSession,
) -> bool {
    use windows052::core::ComInterface;

    left.as_unknown() == right.as_unknown()
}

fn listener_metadata_refresh_due(
    same_session: bool,
    has_metadata: bool,
    invalidated: bool,
    refreshed_at: Option<Instant>,
    now: Instant,
) -> bool {
    !same_session
        || !has_metadata
        || invalidated
        || refreshed_at.is_none_or(|refreshed_at| {
            now.saturating_duration_since(refreshed_at) >= LISTENER_METADATA_TTL
        })
}

#[cfg(test)]
mod listener_metadata_cache_tests {
    use super::listener_metadata_refresh_due;
    use std::time::{Duration, Instant};

    #[test]
    fn metadata_refresh_policy_covers_identity_invalidation_and_five_second_ttl() {
        let refreshed_at = Instant::now();
        assert!(listener_metadata_refresh_due(
            false,
            true,
            false,
            Some(refreshed_at),
            refreshed_at + Duration::from_secs(1),
        ));
        assert!(listener_metadata_refresh_due(
            true,
            false,
            false,
            Some(refreshed_at),
            refreshed_at + Duration::from_secs(1),
        ));
        assert!(listener_metadata_refresh_due(
            true,
            true,
            true,
            Some(refreshed_at),
            refreshed_at + Duration::from_secs(1),
        ));
        assert!(!listener_metadata_refresh_due(
            true,
            true,
            false,
            Some(refreshed_at),
            refreshed_at + Duration::from_millis(4999),
        ));
        assert!(listener_metadata_refresh_due(
            true,
            true,
            false,
            Some(refreshed_at),
            refreshed_at + Duration::from_secs(5),
        ));
    }
}

fn selected_session_from_current_manager(
    app: &AppHandle,
) -> AppResult<Option<(GlobalSystemMediaTransportControlsSession, String, String)>> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|operation| operation.get())
        .map_err(|error| AppError::media(format!("RequestAsync failed: {error:?}")))?;
    selected_session(app, &manager)
}

pub fn list_media_sessions(app: &AppHandle) -> AppResult<Vec<MediaSessionInfo>> {
    let _apartment = super::apartment::Apartment::enter().map_err(|error| {
        AppError::media(format!("Media apartment initialization failed: {error}"))
    })?;
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
            let source = session_source(app, &id).to_string();
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
#[allow(dead_code)]
pub fn get_media_info(app: &AppHandle) -> AppResult<MediaState> {
    let _apartment = super::apartment::Apartment::enter().map_err(|error| {
        AppError::media(format!("Media apartment initialization failed: {error}"))
    })?;
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|operation| operation.get())
        .map_err(|error| AppError::media(format!("RequestAsync failed: {error:?}")))?;
    get_media_info_from_manager(app, &manager)
}

/// Read the selected session from a manager owned by the caller's media actor.
/// Keeping the manager alive lets the listener subscribe once and refresh from
/// SMTC events without reacquiring the WinRT singleton for every snapshot.
#[allow(dead_code)]
pub(crate) fn get_media_info_from_manager(
    app: &AppHandle,
    manager: &GlobalSystemMediaTransportControlsSessionManager,
) -> AppResult<MediaState> {
    let (session, source_type, raw_id) = match selected_session(app, manager)? {
        Some(s) => s,
        None => return Ok(MediaState::default()),
    };

    let metadata = read_media_metadata(app, &session, &raw_id)?;
    build_media_state(&session, source_type, raw_id, metadata)
}

/// Listener-only snapshot path. The cache belongs to the caller's actor and
/// contains the concrete COM session interface, not only its source/AppID.
pub(crate) fn get_media_info_from_manager_for_listener(
    app: &AppHandle,
    manager: &GlobalSystemMediaTransportControlsSessionManager,
    cache: &mut MediaListenerMetadataCache,
    metadata_invalidated: bool,
) -> MediaListenerSnapshot {
    let selected = match selected_session(app, manager) {
        Ok(selected) => selected,
        Err(error) => {
            return MediaListenerSnapshot {
                result: Err(error),
                metadata_read_attempted: false,
                metadata_updated: false,
                metadata_reused: false,
            };
        }
    };
    let Some((session, source_type, raw_id)) = selected else {
        cache.clear();
        return MediaListenerSnapshot {
            result: Ok(MediaState::default()),
            metadata_read_attempted: false,
            metadata_updated: false,
            metadata_reused: false,
        };
    };

    let same_session = cache
        .session
        .as_ref()
        .is_some_and(|cached| media_sessions_have_same_identity(cached, &session));
    if !same_session {
        cache.clear();
        cache.session = Some(session.clone());
    }

    let refresh_metadata = listener_metadata_refresh_due(
        same_session,
        cache.metadata.is_some(),
        metadata_invalidated,
        cache.refreshed_at,
        Instant::now(),
    );
    let (metadata, metadata_read_attempted, metadata_updated, metadata_reused) = if refresh_metadata
    {
        match read_media_metadata(app, &session, &raw_id) {
            Ok(metadata) => {
                cache.metadata = Some(metadata.clone());
                cache.refreshed_at = Some(Instant::now());
                (metadata, true, true, false)
            }
            Err(error) => {
                return MediaListenerSnapshot {
                    result: Err(error),
                    metadata_read_attempted: true,
                    metadata_updated: false,
                    metadata_reused: false,
                };
            }
        }
    } else {
        // A cache entry can only exist with a concrete matching session.
        let metadata = cache
            .metadata
            .as_ref()
            .expect("metadata refresh decision requires a populated cache")
            .clone();
        (metadata, false, false, true)
    };

    MediaListenerSnapshot {
        result: build_media_state(&session, source_type, raw_id, metadata),
        metadata_read_attempted,
        metadata_updated,
        metadata_reused,
    }
}

fn read_media_metadata(
    app: &AppHandle,
    session: &GlobalSystemMediaTransportControlsSession,
    raw_id: &str,
) -> AppResult<MediaMetadata> {
    let info = session
        .TryGetMediaPropertiesAsync()
        .map_err(|e| AppError::media(format!("TryGetMediaPropertiesAsync failed: {:?}", e)))?
        .get()
        .map_err(|e| AppError::media(format!("Await media properties failed: {:?}", e)))?;

    let title = info.Title().unwrap_or_default().to_string();
    let artist = info.Artist().unwrap_or_default().to_string();
    // Thumbnail versions are content identities, not just title/artist. This
    // stream is read on invalidation or the listener's bounded metadata TTL.
    let mut album_art = String::new();
    if let Ok(thumbnail_ref) = info.Thumbnail() {
        if let Ok(stream) = thumbnail_ref
            .OpenReadAsync()
            .and_then(|operation| operation.get())
        {
            if let Ok(reader) = DataReader::CreateDataReader(&stream) {
                if let Ok(size) =
                    super::image_budget::checked_stream_size(stream.Size().unwrap_or(0))
                {
                    if reader
                        .LoadAsync(size)
                        .and_then(|operation| operation.get())
                        .ok()
                        == Some(size)
                    {
                        let mut bytes = vec![0u8; size as usize];
                        if reader.ReadBytes(&mut bytes).is_ok() {
                            let identity =
                                format!("{}|{}", raw_id, super::artwork_jobs::identity(&bytes));
                            let state = app.state::<AppState>();
                            if let Ok(mut cache) = state.media_cache.lock() {
                                if cache.track_id == identity && !cache.base64_img.is_empty() {
                                    album_art = cache.base64_img.clone();
                                } else if let Ok(prepared) =
                                    super::image_budget::prepare_artwork(&bytes)
                                {
                                    album_art = format!(
                                        "data:{};base64,{}",
                                        prepared.content_type,
                                        general_purpose::STANDARD.encode(&prepared.bytes)
                                    );
                                    cache.track_id = identity;
                                    cache.base64_img = album_art.clone();
                                }
                            };
                        }
                    }
                }
            }
        }
    }

    Ok(MediaMetadata {
        title,
        artist,
        album_art,
    })
}

fn build_media_state(
    session: &GlobalSystemMediaTransportControlsSession,
    source_type: String,
    raw_id: String,
    metadata: MediaMetadata,
) -> AppResult<MediaState> {
    // 获取时间线属性（播放进度）
    let timeline = session.GetTimelineProperties().ok();
    let title = metadata.title;
    let artist = metadata.artist;
    let thumbnail_base64 = metadata.album_art;

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
            // 网易云实际跳转走本机 CDP；其它播放器遵循 SMTC 能力声明。
            seek: seek_supported(
                &source_type,
                controls.IsPlaybackPositionEnabled().unwrap_or(false),
            ),
            shuffle: controls.IsShuffleEnabled().unwrap_or(false),
            repeat: controls.IsRepeatEnabled().unwrap_or(false),
        })
        .unwrap_or_default();

    let is_playing = playback_status.0 == 4; // Playing = 4

    // 计算实际播放位置（考虑时间差）
    let dur_ms = (timeline
        .as_ref()
        .and_then(|value| value.EndTime().ok())
        .unwrap_or_default()
        .Duration
        / 10000)
        .max(0) as u64;
    let snapshot_pos_ms = (timeline
        .as_ref()
        .and_then(|value| value.Position().ok())
        .unwrap_or_default()
        .Duration
        / 10000)
        .max(0) as u64;
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
            if timeline.is_none() {
                cache
                    .get(&timeline_key)
                    .copied()
                    .unwrap_or(reported_timeline)
            } else {
                reported_timeline
            },
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
        progress_known: true,
        netease_progress_available: None,
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
        native_epoch: String::new(),
        native_revision: 0,
        native_track_generation: 0,
    })
}

/// 从网易云音乐 API 获取歌曲信息
///
/// 通过歌曲名称和艺术家搜索，返回：
/// - 歌曲时长
/// - 专辑封面 URL
/// - MV ID（播放 URL 通过独立请求获取）
pub async fn get_netease_song_info(
    song_name: &str,
    artist: &str,
) -> AppResult<Option<NeteaseSong>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("Isle/1.0 media resolver")
        .build()
        .map_err(|error| AppError::network(error.to_string()))?;
    let Some(song) = matched_netease_song(&client, song_name, artist).await? else {
        return Ok(None);
    };
    let mut info = parse_netease_song(&song);
    // Artwork is fetched by the backend so WebView CORS/hotlink rules do not hide it.
    if let Some(url) = info.album_pic.as_ref() {
        info.album_pic = download_cover_as_data_url(&client, url).await.ok();
    }
    // MV lookup is independent; callers can request it using mv_id when needed.
    Ok(Some(info))
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

fn require_control_session<T>(session: Option<T>) -> AppResult<T> {
    session.ok_or_else(|| AppError::media("未找到可控制的媒体会话"))
}

fn seek_supported(source: &str, declared: bool) -> bool {
    declared || source == "netease"
}

fn finish_media_request(action: &str, result: windows052::core::Result<bool>) -> AppResult<()> {
    let accepted = result
        .map_err(|error| AppError::media(format!("执行媒体操作 {action} 失败: {error:?}")))?;
    if !accepted {
        return Err(AppError::media(format!("播放器拒绝媒体操作: {action}")));
    }
    Ok(())
}

fn request_seek<F>(source: &str, session_id: &str, declared: bool, request: F) -> AppResult<()>
where
    F: FnOnce() -> windows052::core::Result<bool>,
{
    if !seek_supported(source, declared) {
        return Err(AppError::media(format!(
            "媒体会话不支持调整播放进度: {session_id}"
        )));
    }
    finish_media_request("seek", request())
}

/// 控制媒体播放，并传播播放器的受理或拒绝结果。
/// 支持 play_pause、next 和 prev。
pub fn control_media(app: &AppHandle, action: &str) -> AppResult<()> {
    let _apartment = super::apartment::Apartment::enter().map_err(|error| {
        AppError::media(format!("Media apartment initialization failed: {error}"))
    })?;
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|operation| operation.get())
        .map_err(|error| AppError::media(format!("RequestAsync failed: {error:?}")))?;
    control_media_with_manager(app, &manager, action)
}

/// Execute a control request against the manager already owned by the media
/// actor. The caller must be on that manager's COM apartment.
pub(crate) fn control_media_with_manager(
    app: &AppHandle,
    manager: &GlobalSystemMediaTransportControlsSessionManager,
    action: &str,
) -> AppResult<()> {
    let (session, _, _) = require_control_session(selected_session(app, manager)?)?;

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

    finish_media_request(action, result)
}

pub fn seek_media(
    app: &AppHandle,
    position_ms: u64,
    expected_title: &str,
    expected_artist: &str,
) -> AppResult<Option<super::netease_cdp::NeteaseProgress>> {
    let _apartment = super::apartment::Apartment::enter().map_err(|error| {
        AppError::media(format!("Media apartment initialization failed: {error}"))
    })?;
    let (session, source, raw_id) =
        require_control_session(selected_session_from_current_manager(app)?)?;
    if source == "netease" {
        return super::netease_cdp::LocalCdpClient::default()
            .seek_position(position_ms, expected_title, expected_artist)
            .map(Some)
            .map_err(|error| AppError::media(error.to_string()));
    }
    let declared = session
        .GetPlaybackInfo()
        .and_then(|info| info.Controls())
        .and_then(|controls| controls.IsPlaybackPositionEnabled())
        .map_err(|error| AppError::media(format!("读取 seek 能力失败: {error:?}")))?;
    request_seek(&source, &raw_id, declared, || {
        session
            .TryChangePlaybackPositionAsync(
                (position_ms.saturating_mul(10_000)).min(i64::MAX as u64) as i64,
            )
            .and_then(|op| op.get())
    })?;
    Ok(None)
}

pub fn toggle_shuffle(app: &AppHandle) -> AppResult<()> {
    let _apartment = super::apartment::Apartment::enter().map_err(|error| {
        AppError::media(format!("Media apartment initialization failed: {error}"))
    })?;
    let (session, _, _) = require_control_session(selected_session_from_current_manager(app)?)?;
    let info = session
        .GetPlaybackInfo()
        .map_err(|error| AppError::media(error.to_string()))?;
    let enabled = info
        .Controls()
        .and_then(|controls| controls.IsShuffleEnabled())
        .map_err(|error| AppError::media(error.to_string()))?;
    let current = info
        .IsShuffleActive()
        .and_then(|value| value.Value())
        .map_err(|error| AppError::media(format!("无法读取随机播放状态: {error}")))?;
    request_smtc_mode("shuffle", enabled, || {
        session
            .TryChangeShuffleActiveAsync(!current)
            .and_then(|op| op.get())
    })
}

fn request_smtc_mode<F>(action: &str, enabled: bool, request: F) -> AppResult<()>
where
    F: FnOnce() -> windows052::core::Result<bool>,
{
    if !enabled {
        return Err(AppError::media(format!("播放器不支持切换模式: {action}")));
    }
    finish_media_request(action, request())
}

pub fn cycle_repeat(app: &AppHandle) -> AppResult<()> {
    let _apartment = super::apartment::Apartment::enter().map_err(|error| {
        AppError::media(format!("Media apartment initialization failed: {error}"))
    })?;
    let (session, _, _) = require_control_session(selected_session_from_current_manager(app)?)?;
    let info = session
        .GetPlaybackInfo()
        .map_err(|error| AppError::media(error.to_string()))?;
    let enabled = info
        .Controls()
        .and_then(|controls| controls.IsRepeatEnabled())
        .map_err(|error| AppError::media(error.to_string()))?;
    let current = info
        .AutoRepeatMode()
        .and_then(|value| value.Value())
        .map_err(|error| AppError::media(format!("无法读取循环播放状态: {error}")))?;
    let next = match current.0 {
        0 => MediaPlaybackAutoRepeatMode(2),
        2 => MediaPlaybackAutoRepeatMode(1),
        _ => MediaPlaybackAutoRepeatMode(0),
    };
    request_smtc_mode("repeat", enabled, || {
        session
            .TryChangeAutoRepeatModeAsync(next)
            .and_then(|op| op.get())
    })
}

#[cfg(test)]
mod control_tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn smtc_modes_require_declared_support_and_propagate_player_rejection() {
        for action in ["shuffle", "repeat"] {
            let calls = Cell::new(0);
            assert!(request_smtc_mode(action, false, || {
                calls.set(calls.get() + 1);
                Ok(true)
            })
            .is_err());
            assert_eq!(calls.get(), 0);
            assert!(request_smtc_mode(action, true, || {
                calls.set(calls.get() + 1);
                Ok(false)
            })
            .is_err());
            assert_eq!(calls.get(), 1);
            assert!(request_smtc_mode(action, true, || {
                calls.set(calls.get() + 1);
                Ok(true)
            })
            .is_ok());
            assert_eq!(calls.get(), 2);
        }
    }

    #[test]
    fn one_click_binding_requires_one_matching_session_and_does_not_guess_from_an_unknown_id() {
        let session = |id: &str, source: &str, title: &str, artist: &str| {
            (id.into(), source.into(), title.into(), artist.into())
        };
        let generated = session(
            "Microsoft.AutoGenerated.new-PC",
            "generic",
            "Track",
            "Artist",
        );
        assert_eq!(
            unique_netease_binding(&[generated.clone()], "Track", "Artist"),
            Some(generated.0.clone())
        );
        assert_eq!(
            unique_netease_binding(&[generated.clone()], "Other track", "Artist"),
            None
        );
        assert_eq!(
            unique_netease_binding(&[generated.clone()], "Track", "Other artist"),
            None
        );
        let duplicate = session(
            "Microsoft.AutoGenerated.other",
            "generic",
            "Track",
            "Artist",
        );
        assert_eq!(
            unique_netease_binding(&[generated.clone(), duplicate], "Track", "Artist"),
            None
        );
        let spotify = session("Spotify.exe", "spotify", "Track", "Artist");
        assert_eq!(unique_netease_binding(&[spotify], "Track", "Artist"), None);
        assert_eq!(unique_netease_binding(&[], "", ""), None);
    }

    #[test]
    fn unknown_session_identity_override_controls_source_without_relabeling_other_players() {
        let id = "Microsoft.AutoGenerated.{034E9048-FE2F-9805-08CC-310B}";
        assert_eq!(source_with_override(id, None), "generic");
        assert_eq!(source_with_override(id, Some("netease")), "netease");
        assert_eq!(
            source_display(source_with_override(id, Some("netease")), id),
            "网易云音乐"
        );
        assert_eq!(source_with_override(id, Some("spotify")), "generic");
        assert_eq!(
            source_with_override("Spotify.exe", Some("netease")),
            "spotify"
        );
        assert_eq!(source_with_override("cloudmusic.exe", None), "netease");
        assert!(seek_supported("netease", false));
    }

    #[test]
    fn accepted_request_is_success_but_rejection_is_serialized_error() {
        assert!(finish_media_request("play_pause", Ok(true)).is_ok());
        for action in ["play_pause", "next", "prev", "seek"] {
            let error = finish_media_request(action, Ok(false)).unwrap_err();
            let payload = serde_json::to_value(&error).unwrap();
            assert_eq!(payload["code"], 2001);
            assert!(payload["message"].as_str().unwrap().contains("拒绝"));
            assert!(payload["message"].as_str().unwrap().contains(action));
        }
    }

    #[test]
    fn winrt_failure_is_not_replaced_with_success_or_rejection() {
        let native = windows052::core::Error::new(
            windows052::core::HRESULT(0x80004005u32 as i32),
            windows052::core::HSTRING::from("test native failure"),
        );
        let error = finish_media_request("next", Err(native)).unwrap_err();
        assert_eq!(error.code(), 2001);
        assert!(error.to_string().contains("执行媒体操作 next 失败"));
    }

    #[test]
    fn unsupported_seek_never_dispatches_to_the_player() {
        let calls = Cell::new(0);
        let error = request_seek(
            "apple",
            "AppleInc.AppleMusicWin_nzyj5cx40ttqa!App",
            false,
            || {
                calls.set(calls.get() + 1);
                Ok(true)
            },
        )
        .unwrap_err();
        assert_eq!(calls.get(), 0);
        assert_eq!(error.code(), 2001);
        assert!(error.to_string().contains("不支持调整播放进度"));
        assert!(error.to_string().contains("AppleInc.AppleMusicWin"));
    }

    #[test]
    fn declared_seek_and_netease_compatibility_dispatch_once_and_check_result() {
        for (source, declared) in [("apple", true), ("netease", false)] {
            let calls = Cell::new(0);
            assert!(request_seek(source, "session", declared, || {
                calls.set(calls.get() + 1);
                Ok(true)
            })
            .is_ok());
            assert_eq!(calls.get(), 1);
            assert!(request_seek(source, "session", declared, || Ok(false)).is_err());
        }
        assert!(!seek_supported("spotify", false));
    }

    #[test]
    fn missing_session_is_a_structured_error() {
        let error = require_control_session::<()>(None).unwrap_err();
        assert_eq!(serde_json::to_value(error).unwrap()["code"], 2001);
        assert_eq!(require_control_session(Some(42)).unwrap(), 42);
    }
}

#[cfg(test)]
mod cover_tests {
    use super::{
        apple_hd_url, best_cover_candidate, candidate_score, cover_provider_order, netease_hd_url,
        normalize_media_text, ordered_selected_ids, source_display, stabilize_timeline,
        CoverCandidate, TimelineSnapshot,
    };

    #[test]
    fn selected_players_follow_visible_order() {
        let order = vec!["second".to_string(), "first".to_string()];
        let selected = vec![
            "first".to_string(),
            "second".to_string(),
            "legacy".to_string(),
        ];
        assert_eq!(
            ordered_selected_ids(&order, &selected),
            ["second", "first", "legacy"]
        );
    }

    #[test]
    fn formats_common_windows_session_ids() {
        assert_eq!(
            source_display(
                "generic",
                "Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic"
            ),
            "Windows 媒体播放器"
        );
        assert_eq!(
            source_display("generic", "Chrome.UserData.Profile2"),
            "Chrome · 个人资料 2"
        );
        assert_eq!(
            source_display("generic", "OpenAI.Codex_2p2nqsd0c76g0!App"),
            "OpenAI Codex"
        );
    }

    fn candidate(title: &str, artist: &str, url: &str) -> CoverCandidate {
        CoverCandidate {
            title: title.to_string(),
            artist: artist.to_string(),
            url: url.to_string(),
            source_id: None,
        }
    }

    #[test]
    fn parses_netease_detail_shapes_without_requiring_mv() {
        let legacy = super::parse_netease_song(&serde_json::json!({
            "duration": 180000, "album": { "blurPicUrl": "https://p1.music.126.net/cover.jpg" }, "mvid": 123
        }));
        assert_eq!(legacy.duration, Some(180000));
        assert!(legacy.album_pic.unwrap().contains("cover.jpg"));
        assert_eq!(legacy.mv_id, Some(123));
        let modern = super::parse_netease_song(&serde_json::json!({
            "dt": 240000, "al": { "picUrl": "https://p1.music.126.net/new.jpg" }, "mv": 0
        }));
        assert_eq!(modern.duration, Some(240000));
        assert!(modern.album_pic.unwrap().contains("new.jpg"));
        assert!(modern.mv_url.is_none());
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
    fn accepts_paused_seeks_and_restart_at_zero() {
        let previous = TimelineSnapshot {
            position_ms: 42_500,
            duration_ms: 180_000,
        };
        let reset = TimelineSnapshot::default();
        assert_eq!(
            stabilize_timeline(Some(previous), reset, false),
            TimelineSnapshot {
                position_ms: 0,
                ..previous
            }
        );
        assert_eq!(
            stabilize_timeline(Some(previous), reset, true),
            TimelineSnapshot {
                position_ms: 0,
                ..previous
            }
        );
        let stale = TimelineSnapshot {
            position_ms: 12_000,
            duration_ms: 180_000,
        };
        assert_eq!(stabilize_timeline(Some(previous), stale, false), stale);
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
