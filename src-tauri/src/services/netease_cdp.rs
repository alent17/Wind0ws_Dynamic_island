//! Bounded, IPv4-loopback-only CDP access for NetEase playback and favorites.
//!
//! Synchronous operations are dispatched away from the UI thread.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
    num::NonZeroU16,
    time::{Duration, Instant},
};
use tungstenite::{
    client::{client_with_config, IntoClientRequest},
    handshake::HandshakeError,
    protocol::WebSocketConfig,
    Message, WebSocket,
};

const LOOPBACK: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_PORT: u16 = 9223;
const MAX_HEADER_BYTES: usize = 8 * 1024;
const MAX_BODY_BYTES: usize = 32 * 1024;
const MAX_PAGE_TARGETS: usize = 64;
const MAX_COMMAND_BYTES: usize = 16 * 1024;
const MAX_UNMATCHED_MESSAGES: usize = 8;
const CONNECT_TIMEOUT: Duration = Duration::from_millis(400);
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(5);
const PROGRESS_READ_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackMode {
    Sequential,
    RepeatList,
    RepeatOne,
    Shuffle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerifiedModeAction {
    pub requested: PlaybackMode,
    pub observed: PlaybackMode,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseProgress {
    pub track_id: String,
    pub play_id: String,
    pub title: String,
    pub artist: String,
    pub position_ms: u64,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub enum CdpError {
    InvalidPort,
    Io(std::io::Error),
    InvalidResponse,
    UnsafeWebSocketEndpoint,
    NoInitializedPage,
    AmbiguousPage,
    UnsupportedCurrentMode,
    UnsupportedTrack,
    InvalidPosition,
    TrackChanged,
    VerificationFailed,
    InvalidCommand,
    TimedOut,
    Protocol,
    WebSocket(tungstenite::Error),
}

impl std::fmt::Display for CdpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPort => f.write_str("CDP 端口无效"),
            Self::Io(_) => f.write_str("本机 CDP 服务不可用"),
            Self::InvalidResponse => f.write_str("本机 CDP 响应无效或超出大小限制"),
            Self::UnsafeWebSocketEndpoint => f.write_str("CDP 返回了非本机 WebSocket 地址"),
            Self::NoInitializedPage => f.write_str("未找到已初始化的网易云播放页面"),
            Self::AmbiguousPage => f.write_str("发现多个已初始化的网易云播放页面"),
            Self::UnsupportedCurrentMode => f.write_str("当前网易云播放模式不支持切换"),
            Self::UnsupportedTrack => f.write_str("网易云当前歌曲不是可控制的普通音频"),
            Self::InvalidPosition => f.write_str("网易云跳转位置超出当前歌曲时长"),
            Self::TrackChanged => f.write_str("网易云在操作期间切换了歌曲或播放实例"),
            Self::VerificationFailed => f.write_str("网易云操作读回校验失败"),
            Self::InvalidCommand => f.write_str("CDP 命令无效或超出长度限制"),
            Self::TimedOut => f.write_str("本机 CDP 操作超时"),
            Self::Protocol => f.write_str("本机 CDP 返回了错误响应"),
            Self::WebSocket(_) => f.write_str("本机 CDP WebSocket 连接失败"),
        }
    }
}

impl CdpError {
    #[cfg(feature = "c2-media-probe")]
    pub(crate) fn diagnostic_kind(&self) -> &'static str {
        match self {
            Self::TrackChanged => "identity_mismatch",
            Self::NoInitializedPage => "no_initialized_page",
            Self::AmbiguousPage => "ambiguous_page",
            Self::TimedOut => "timeout",
            Self::Io(_) => "io",
            Self::InvalidResponse | Self::Protocol | Self::WebSocket(_) => "protocol",
            Self::UnsafeWebSocketEndpoint => "unsafe_endpoint",
            Self::InvalidPort | Self::InvalidCommand => "invalid_request",
            Self::UnsupportedCurrentMode | Self::UnsupportedTrack | Self::InvalidPosition => {
                "unsupported"
            }
            Self::VerificationFailed => "verification_failed",
        }
    }
}

impl std::error::Error for CdpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::WebSocket(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CdpError {
    fn from(error: std::io::Error) -> Self {
        if matches!(
            error.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ) {
            Self::TimedOut
        } else {
            Self::Io(error)
        }
    }
}

impl From<tungstenite::Error> for CdpError {
    fn from(error: tungstenite::Error) -> Self {
        if matches!(
            &error,
            tungstenite::Error::Io(io_error)
                if matches!(io_error.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock)
        ) {
            Self::TimedOut
        } else {
            Self::WebSocket(error)
        }
    }
}

/// A CDP client whose network destination is always `127.0.0.1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalCdpClient {
    port: NonZeroU16,
}

impl Default for LocalCdpClient {
    fn default() -> Self {
        Self::new(DEFAULT_PORT).unwrap_or(Self {
            port: NonZeroU16::MIN,
        })
    }
}

impl LocalCdpClient {
    pub(crate) fn player_identity(self) -> Result<Value, CdpError> {
        let (_, value) = self.initialized_page_using(Instant::now() + OPERATION_TIMEOUT, CLOUD_MUSIC_IDENTITY_EXPRESSION, "ready")?;
        Ok(value)
    }
    pub fn song_like(
        self,
        title: &str,
        artist: &str,
        expected: Option<(&str, &str, bool)>,
    ) -> Result<Value, CdpError> {
        if title.is_empty() || title.len() + artist.len() > 4096 {
            return Err(CdpError::InvalidCommand);
        }
        let deadline = Instant::now() + OPERATION_TIMEOUT;
        let (mut connection, value) =
            self.initialized_page_using(deadline, CLOUD_MUSIC_LIKE_READ_EXPRESSION, "ready")?;
        validate_like_identity(&value, title, artist)?;
        if let Some((track_id, play_id, liked)) = expected {
            if value.get("trackId").and_then(Value::as_str) != Some(track_id)
                || value.get("playId").and_then(Value::as_str) != Some(play_id)
            {
                return Err(CdpError::TrackChanged);
            }
            let expression = CLOUD_MUSIC_LIKE_SET_EXPRESSION
                .replace("__LIKE_READ__", CLOUD_MUSIC_LIKE_READ_EXPRESSION)
                .replace(
                    "__EXPECTED__",
                    &json!({"trackId":track_id,"playId":play_id,"title":value["title"],"artist":value["artist"]})
                        .to_string(),
                )
                .replace("__LIKED__", if liked { "true" } else { "false" });
            let mut params = runtime_evaluate_params(&expression);
            params["awaitPromise"] = json!(true);
            let response = connection.request("Runtime.evaluate", &params, deadline)?;
            let result = response
                .pointer("/result/value")
                .ok_or(CdpError::InvalidResponse)?;
            if response.get("exceptionDetails").is_some()
                || result.get("verified").and_then(Value::as_bool) != Some(true)
            {
                return Err(CdpError::VerificationFailed);
            }
            return Ok(result.clone());
        }
        Ok(value)
    }

    pub fn new(port: u16) -> Result<Self, CdpError> {
        Ok(Self {
            port: NonZeroU16::new(port).ok_or(CdpError::InvalidPort)?,
        })
    }

    pub const fn port(self) -> u16 {
        self.port.get()
    }

    /// Reads the current ordinary mode from exactly one initialized NetEase page.
    pub fn playback_mode(self) -> Result<Option<PlaybackMode>, CdpError> {
        let deadline = Instant::now() + OPERATION_TIMEOUT;
        let (_, value) =
            self.initialized_page_using(deadline, CLOUD_MUSIC_READ_EXPRESSION, "mode")?;
        let mode = value
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or_default();
        Ok(mode_from_provider(mode))
    }

    /// Sets an allowlisted ordinary mode and verifies the store's read-back.
    /// AI/FM and unknown modes are never overwritten.
    pub fn set_playback_mode(
        self,
        requested: PlaybackMode,
    ) -> Result<VerifiedModeAction, CdpError> {
        let deadline = Instant::now() + OPERATION_TIMEOUT;
        let (mut connection, current_value) =
            self.initialized_page_using(deadline, CLOUD_MUSIC_READ_EXPRESSION, "mode")?;
        let current = current_value
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if mode_from_provider(current).is_none() {
            return Err(CdpError::UnsupportedCurrentMode);
        }

        let requested_provider = provider_mode(requested);
        let expression = cloud_music_set_expression(requested_provider);
        let response = connection.request(
            "Runtime.evaluate",
            &json!({
                "expression": expression,
                "returnByValue": true,
                "awaitPromise": true
            }),
            deadline,
        )?;
        if response.get("exceptionDetails").is_some() {
            return Err(CdpError::Protocol);
        }
        let value = response
            .pointer("/result/value")
            .ok_or(CdpError::InvalidResponse)?;
        let observed_raw = required_string(value, "actual")?;
        let observed = mode_from_provider(observed_raw).ok_or(CdpError::UnsupportedCurrentMode)?;
        if value.get("verified").and_then(Value::as_bool) != Some(true)
            || value.get("requested").and_then(Value::as_str) != Some(requested_provider)
            || observed != requested
        {
            return Err(CdpError::VerificationFailed);
        }
        Ok(VerifiedModeAction {
            requested,
            observed,
        })
    }

    /// Executes the only verified NetEase seek action and checks actual playback progress.
    pub fn seek_position(
        self,
        position_ms: u64,
        expected_title: &str,
        expected_artist: &str,
    ) -> Result<NeteaseProgress, CdpError> {
        let deadline = Instant::now() + Duration::from_secs(6);
        let (mut connection, initial) =
            self.initialized_page_using(deadline, CLOUD_MUSIC_PROGRESS_EXPRESSION, "ready")?;
        let initial_progress = parse_bound_netease_progress(&initial, expected_title, expected_artist)?;
        if position_ms >= initial_progress.duration_ms {
            return Err(CdpError::InvalidPosition);
        }
        let expression = cloud_music_seek_expression(
            position_ms as f64 / 1000.0,
            expected_title,
            required_string(&initial, "artist")?,
        );
        let response = connection.request(
            "Runtime.evaluate",
            &json!({ "expression": expression, "returnByValue": true, "awaitPromise": true }),
            deadline,
        )?;
        if response.get("exceptionDetails").is_some() {
            return Err(CdpError::Protocol);
        }
        let value = response
            .pointer("/result/value")
            .ok_or(CdpError::InvalidResponse)?;
        if value.get("verified").and_then(Value::as_bool) != Some(true) {
            if value.get("unsupported").and_then(Value::as_bool) == Some(true) {
                return Err(CdpError::UnsupportedTrack);
            }
            if value.get("invalidPosition").and_then(Value::as_bool) == Some(true) {
                return Err(CdpError::InvalidPosition);
            }
            if value.get("trackChanged").and_then(Value::as_bool) == Some(true) {
                return Err(CdpError::TrackChanged);
            }
            if value.get("identityMismatch").and_then(Value::as_bool) == Some(true) {
                return Err(CdpError::TrackChanged);
            }
            if value.get("timedOut").and_then(Value::as_bool) == Some(true) {
                return Err(CdpError::TimedOut);
            }
            return Err(CdpError::VerificationFailed);
        }
        let mut progress = parse_netease_progress(value)?;
        validate_expected_identity(&progress, expected_title, required_string(&initial, "artist")?)?;
        progress.artist = expected_artist.to_string();
        Ok(progress)
    }

    /// Reads an identity-bound progress sample from the initialized player page.
    pub fn read_progress(
        self,
        expected_title: &str,
        expected_artist: &str,
    ) -> Result<NeteaseProgress, CdpError> {
        let (_, value) = self.initialized_page_using(Instant::now() + PROGRESS_READ_TIMEOUT, CLOUD_MUSIC_PROGRESS_EXPRESSION, "ready")?;
        parse_bound_netease_progress(&value, expected_title, expected_artist)
    }

    /// Read-only identity used by explicit setup to match a Windows session.
    pub(crate) fn read_progress_snapshot(self) -> Result<NeteaseProgress, CdpError> {
        self.read_progress_before(Instant::now() + OPERATION_TIMEOUT)
    }

    fn read_progress_before(self, deadline: Instant) -> Result<NeteaseProgress, CdpError> {
        let (_, value) =
            self.initialized_page_using(deadline, CLOUD_MUSIC_PROGRESS_EXPRESSION, "ready")?;
        parse_netease_progress(&value)
    }

    fn initialized_page_using(
        self,
        deadline: Instant,
        expression: &str,
        ready_key: &str,
    ) -> Result<(CdpConnection, Value), CdpError> {
        // Validate the browser-level endpoint returned by CDP as well as every
        // page endpoint. We never connect to a host supplied by that response.
        let version = self.get_json("/json/version", deadline)?;
        let version: Value =
            serde_json::from_slice(&version).map_err(|_| CdpError::InvalidResponse)?;
        validate_required_string(&version, "Browser")?;
        validate_required_string(&version, "Protocol-Version")?;
        let browser_ws = required_string(&version, "webSocketDebuggerUrl")?;
        validate_endpoint(browser_ws, self.port(), "browser")?;

        let body = self.get_json("/json/list", deadline)?;
        let targets: Value =
            serde_json::from_slice(&body).map_err(|_| CdpError::InvalidResponse)?;
        let targets = targets.as_array().ok_or(CdpError::InvalidResponse)?;
        if targets.len() > MAX_PAGE_TARGETS {
            return Err(CdpError::InvalidResponse);
        }

        let mut matches = Vec::new();
        for target in targets {
            if Instant::now() >= deadline {
                return Err(CdpError::TimedOut);
            }
            if target.get("type").and_then(Value::as_str) != Some("page") {
                continue;
            }
            let Some(url) = target.get("url").and_then(Value::as_str) else {
                continue;
            };
            if !is_netease_page_url(url) {
                continue;
            }
            let Some(endpoint) = target.get("webSocketDebuggerUrl").and_then(Value::as_str) else {
                continue;
            };
            let path = validate_endpoint(endpoint, self.port(), "page")?;
            let Ok(mut connection) = self.connect_path(path, deadline) else {
                continue;
            };
            let params = runtime_evaluate_params(expression);
            let Ok(response) = connection.request("Runtime.evaluate", &params, deadline) else {
                continue;
            };
            if response.get("exceptionDetails").is_some() {
                continue;
            }
            if let Some(value) = response.pointer("/result/value") {
                let ready = if ready_key == "ready" {
                    value.get("ready").and_then(Value::as_bool) == Some(true)
                } else {
                    value
                        .get(ready_key)
                        .and_then(Value::as_str)
                        .is_some_and(|field| !field.is_empty() && field.len() <= 64)
                };
                if ready {
                    matches.push((connection, value.clone()));
                }
            }
        }

        match matches.len() {
            0 if Instant::now() >= deadline => Err(CdpError::TimedOut),
            0 => Err(CdpError::NoInitializedPage),
            1 => matches.pop().ok_or(CdpError::NoInitializedPage),
            _ => Err(CdpError::AmbiguousPage),
        }
    }

    fn get_json(self, path: &str, deadline: Instant) -> Result<Vec<u8>, CdpError> {
        if !matches!(path, "/json/version" | "/json/list") {
            return Err(CdpError::InvalidCommand);
        }
        let mut stream = self.connect_tcp(deadline)?;
        set_timeouts(&stream, deadline)?;
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\nAccept: application/json\r\n\r\n",
            self.port()
        )?;
        read_http_json_response(&mut stream, deadline)
    }

    fn connect_tcp(self, deadline: Instant) -> Result<TcpStream, CdpError> {
        let timeout = remaining_timeout(deadline, CONNECT_TIMEOUT)?;
        let address = SocketAddr::V4(SocketAddrV4::new(LOOPBACK, self.port()));
        let stream = TcpStream::connect_timeout(&address, timeout)?;
        // CDP uses tiny local HTTP/WebSocket writes. Do not let Nagle's
        // algorithm delay a request while waiting for an earlier small ACK.
        stream.set_nodelay(true)?;
        Ok(stream)
    }

    fn connect_path(self, path: &str, deadline: Instant) -> Result<CdpConnection, CdpError> {
        let stream = self.connect_tcp(deadline)?;
        set_timeouts(&stream, deadline)?;
        let endpoint = format!("ws://127.0.0.1:{}{path}", self.port());
        let request = endpoint
            .into_client_request()
            .map_err(CdpError::WebSocket)?;
        let config = WebSocketConfig::default()
            .read_buffer_size(4096)
            .write_buffer_size(0)
            .max_write_buffer_size(MAX_COMMAND_BYTES + 1024)
            .max_message_size(Some(MAX_BODY_BYTES))
            .max_frame_size(Some(MAX_BODY_BYTES));
        let (socket, _) =
            client_with_config(request, stream, Some(config)).map_err(|error| match error {
                HandshakeError::Failure(error) => CdpError::from(error),
                HandshakeError::Interrupted(_) => CdpError::Protocol,
            })?;
        Ok(CdpConnection { socket, next_id: 1 })
    }
}

pub fn mode_from_provider(mode: &str) -> Option<PlaybackMode> {
    match mode {
        "playOrder" => Some(PlaybackMode::Sequential),
        "playCycle" => Some(PlaybackMode::RepeatList),
        "playOneCycle" => Some(PlaybackMode::RepeatOne),
        "playRandom" => Some(PlaybackMode::Shuffle),
        _ => None,
    }
}

pub fn provider_mode(mode: PlaybackMode) -> &'static str {
    match mode {
        PlaybackMode::Sequential => "playOrder",
        PlaybackMode::RepeatList => "playCycle",
        PlaybackMode::RepeatOne => "playOneCycle",
        PlaybackMode::Shuffle => "playRandom",
    }
}

pub const fn next_mode(mode: PlaybackMode) -> PlaybackMode {
    match mode {
        PlaybackMode::Sequential => PlaybackMode::RepeatList,
        PlaybackMode::RepeatList => PlaybackMode::RepeatOne,
        PlaybackMode::RepeatOne => PlaybackMode::Shuffle,
        PlaybackMode::Shuffle => PlaybackMode::Sequential,
    }
}

pub const fn mode_key(mode: PlaybackMode) -> &'static str {
    match mode {
        PlaybackMode::Sequential => "sequential",
        PlaybackMode::RepeatList => "repeat_list",
        PlaybackMode::RepeatOne => "repeat_one",
        PlaybackMode::Shuffle => "shuffle",
    }
}

fn remaining_timeout(deadline: Instant, cap: Duration) -> Result<Duration, CdpError> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(CdpError::TimedOut);
    }
    Ok(remaining.min(cap))
}

fn set_timeouts(stream: &TcpStream, deadline: Instant) -> Result<(), CdpError> {
    let timeout = remaining_timeout(deadline, IO_TIMEOUT)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    Ok(())
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, CdpError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && value.len() <= 1024)
        .ok_or(CdpError::InvalidResponse)
}

fn validate_required_string(value: &Value, key: &str) -> Result<(), CdpError> {
    required_string(value, key).map(|_| ())
}

fn parse_http_json_headers(headers: &[u8]) -> Result<usize, CdpError> {
    if headers.len() > MAX_HEADER_BYTES {
        return Err(CdpError::InvalidResponse);
    }
    let headers = std::str::from_utf8(headers).map_err(|_| CdpError::InvalidResponse)?;
    let mut lines = headers.split("\r\n");
    let status = lines.next().ok_or(CdpError::InvalidResponse)?;
    if status != "HTTP/1.1 200 OK" && status != "HTTP/1.0 200 OK" {
        return Err(CdpError::InvalidResponse);
    }
    let mut content_length = None;
    let mut is_json = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            return Err(CdpError::InvalidResponse);
        };
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return Err(CdpError::InvalidResponse);
            }
            let parsed = value
                .trim()
                .parse::<usize>()
                .map_err(|_| CdpError::InvalidResponse)?;
            if parsed > MAX_BODY_BYTES {
                return Err(CdpError::InvalidResponse);
            }
            content_length = Some(parsed);
        } else if name.eq_ignore_ascii_case("content-type") {
            is_json = value
                .trim()
                .split(';')
                .next()
                .is_some_and(|mime| mime.eq_ignore_ascii_case("application/json"));
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(CdpError::InvalidResponse);
        }
    }
    if !is_json {
        return Err(CdpError::InvalidResponse);
    }
    content_length.ok_or(CdpError::InvalidResponse)
}

fn parse_http_json_body(response: &[u8]) -> Result<&[u8], CdpError> {
    if response.len() > MAX_HEADER_BYTES + 4 + MAX_BODY_BYTES {
        return Err(CdpError::InvalidResponse);
    }
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or(CdpError::InvalidResponse)?;
    let content_length = parse_http_json_headers(&response[..header_end])?;
    let body = &response[header_end + 4..];
    if body.len() != content_length {
        return Err(CdpError::InvalidResponse);
    }
    Ok(body)
}

fn read_http_json_response(stream: &mut TcpStream, deadline: Instant) -> Result<Vec<u8>, CdpError> {
    let mut reader = BufReader::with_capacity(4096, stream);
    let max_header_section = MAX_HEADER_BYTES + 4;
    let mut response = Vec::with_capacity(1024);
    let header_end = loop {
        if let Some(header_end) = response.windows(4).position(|window| window == b"\r\n\r\n") {
            break header_end;
        }
        let room = max_header_section.saturating_sub(response.len());
        if room == 0 {
            return Err(CdpError::InvalidResponse);
        }
        reader
            .get_mut()
            .set_read_timeout(Some(remaining_timeout(deadline, IO_TIMEOUT)?))?;
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Err(CdpError::InvalidResponse);
        }
        let count = available.len().min(room);
        response.extend_from_slice(&available[..count]);
        reader.consume(count);
    };
    if header_end > MAX_HEADER_BYTES {
        return Err(CdpError::InvalidResponse);
    }

    let content_length = parse_http_json_headers(&response[..header_end])?;
    let expected_length = header_end
        .checked_add(4)
        .and_then(|header| header.checked_add(content_length))
        .filter(|length| *length <= MAX_HEADER_BYTES + 4 + MAX_BODY_BYTES)
        .ok_or(CdpError::InvalidResponse)?;
    if response.len() > expected_length {
        return Err(CdpError::InvalidResponse);
    }

    while response.len() < expected_length {
        reader
            .get_mut()
            .set_read_timeout(Some(remaining_timeout(deadline, IO_TIMEOUT)?))?;
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Err(CdpError::InvalidResponse);
        }
        let remaining = expected_length - response.len();
        if available.len() > remaining {
            return Err(CdpError::InvalidResponse);
        }
        let count = available.len();
        response.extend_from_slice(available);
        reader.consume(count);
    }

    Ok(parse_http_json_body(&response)?.to_vec())
}

fn validate_endpoint<'a>(
    endpoint: &'a str,
    expected_port: u16,
    kind: &str,
) -> Result<&'a str, CdpError> {
    let prefix = format!("ws://127.0.0.1:{expected_port}/devtools/{kind}/");
    let Some(id) = endpoint.strip_prefix(&prefix) else {
        return Err(CdpError::UnsafeWebSocketEndpoint);
    };
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(CdpError::UnsafeWebSocketEndpoint);
    }
    let authority = format!("ws://127.0.0.1:{expected_port}");
    Ok(&endpoint[authority.len()..])
}

fn is_netease_page_url(url: &str) -> bool {
    let normalized = url.to_ascii_lowercase();
    normalized.starts_with("orpheus:")
        || [
            "http://orpheus/",
            "http://orpheus.",
            "https://orpheus/",
            "https://orpheus.",
        ]
        .iter()
        .any(|prefix| normalized.starts_with(prefix))
        || (normalized.starts_with("file:")
            && normalized.contains("cloudmusic")
            && normalized.ends_with("app.html"))
}

const CLOUD_MUSIC_CAPTURE_TOOL: &str = r#"function captureTool(withProgress = false) {
  const queue = globalThis.webpackJsonp;
  if (!Array.isArray(queue) || queue.push === Array.prototype.push) return {};
  const main = queue.find(chunk => chunk && chunk[1] && chunk[1][8] &&
    String(chunk[1][8]).includes('this.app._store.getState()'));
  if (!main) return {};
  const entries = queue.flatMap(chunk => chunk && Array.isArray(chunk[2]) ? chunk[2] : []);
  if (!entries.length) return {};
  const previousEntry = entries[entries.length - 1][0];
  const id = '__isle_mode_read_' + Date.now() + '_' + Math.random().toString(36).slice(2);
  let req;
  const chunk = [[], { [id]: (_module, _exports, runtime) => { req = runtime; } }, [[id]]];
  try {
    queue.push(chunk);
    if (!req || !req.c || !req.c[8] || !req.c[previousEntry]) return {};
    const tool = req.c[8].exports.a;
    if (!tool || !tool.inited || !tool.app || !tool.app._store || typeof tool.getStore !== 'function') return {};
    const progress = withProgress && req.c[148] && req.c[148].exports && req.c[148].exports.b;
    if (withProgress && typeof progress !== 'function') return {};
    return { tool, progress };
  } catch (_) {
    return {};
  } finally {
    if (req) { delete req.m[id]; delete req.c[id]; req.s = previousEntry; }
    const index = queue.indexOf(chunk);
    if (index >= 0) queue.splice(index, 1);
  }
}"#;

const CLOUD_MUSIC_IDENTITY_EXPRESSION: &str = r#"(() => {
  const { tool } = (__CAPTURE_TOOL__)();
  const p = tool && tool.getStore().playing;
  const track = p && p.curPlaying && p.curPlaying.track;
  if (!track || p.resourceType !== 'track' || p.isLoadingFirst || p.isPlayingVideo) return {};
  const artists = track.artists || track.ar;
  const title = String(p.resourceName || track.name || '');
  if (!title || title.length > 1024) return {};
  return { ready: true, title, artist: String(p.resourceArtist || (Array.isArray(artists) ? artists.map(a => a && a.name || '').filter(Boolean).join(', ') : '')), primaryArtist: String(Array.isArray(artists) && artists[0] && artists[0].name || '') };
})()"#;

const CLOUD_MUSIC_LIKE_READ_EXPRESSION: &str = r#"(() => {
  const { tool } = (__CAPTURE_TOOL__)();
  const s = tool && tool.getStore(), p = s && s.playing, h = s && s['async:hostResource'];
  if (!p || !h || !s.app || !s.app.isOnline || !h.starPlaylistId || !h.likeTracksMap || !Array.isArray(h.likeTrackIds) || p.resourceType !== 'track' || p.isLoadingFirst || p.restoreResource || p.isPlayingVideo || !p.playId) return {};
  const track = p.curPlaying && p.curPlaying.track;
  const id = String(p.onlineResourceId || '');
  if (!track || !id || String(track.id) !== id) return {};
  const artists = track.artists || track.ar;
  return { ready: true, trackId: id, playId: String(p.playId), title: String(p.resourceName || track.name || ''), primaryArtist: String(Array.isArray(artists) && artists[0] && artists[0].name || ''), artist: String(p.resourceArtist || (Array.isArray(artists) ? artists.map(a => a && a.name || '').filter(Boolean).join(', ') : '')), liked: h.likeTracksMap[id] === true };
})()"#;

const CLOUD_MUSIC_LIKE_SET_EXPRESSION: &str = r#"(async () => {
  const read = () => __LIKE_READ__;
  const expected = __EXPECTED__, liked = __LIKED__;
  const same = value => value.ready && ['trackId','playId','title','artist'].every(k => value[k] === expected[k]);
  const before = read();
  if (!same(before)) return { verified: false };
  if (before.liked === liked) return { ...before, verified: true };
  const { tool } = (__CAPTURE_TOOL__)();
  const p = tool.getStore().playing;
  if (!same(read())) return { verified: false };
  await tool.getDispatch()({ type: 'async:hostResource/setLikeTrack', payload: { isLike: liked, resource: p.curPlaying.track, resourceType: 'track' } });
  for (let i = 0; i < 20; i++) {
    const after = read();
    if (!same(after)) return { verified: false };
    if (after.liked === liked) return { ...after, verified: true };
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  return { verified: false };
})()"#;

const CLOUD_MUSIC_READ_EXPRESSION: &str = r#"(() => {
  const { tool } = (__CAPTURE_TOOL__)();
  const playing = tool.getStore().playing;
  return playing && typeof playing.playingMode === 'string' ? { mode: playing.playingMode } : {};
})()"#;

const CLOUD_MUSIC_PROGRESS_EXPRESSION: &str = r#"(() => {
  const captured = (__CAPTURE_TOOL__)(true);
  const tool = captured.tool;
  const playing = tool && tool.getStore().playing;
  if (!playing || playing.isLoadingFirst || playing.restoreResource || playing.isPlayingVideo || !playing.playId) return {};
  const native = captured.progress();
  const track = playing.curPlaying && playing.curPlaying.track;
  const artists = track && (track.artists || track.ar);
  const title = String(playing.resourceName || track && track.name || '');
  const artist = String(playing.resourceArtist || (Array.isArray(artists) ? artists.map(item => item && item.name || '').filter(Boolean).join(', ') : ''));
  const position = Number(native && native.current);
  const duration = Number(playing.resourceDuration);
  const trackId = String(playing.onlineResourceId || '');
  if (!native || native.playId !== playing.playId || !trackId || !title || !Number.isFinite(position) || !Number.isFinite(duration) || position < 0 || duration <= 0 || position > duration) return {};
  return { ready: true, trackId, playId: String(playing.playId), title, artist, primaryArtist: String(Array.isArray(artists) && artists[0] && artists[0].name || ''), positionMs: Math.round(position * 1000), durationMs: Math.round(duration * 1000) };
})()"#;

const CLOUD_MUSIC_SEEK_EXPRESSION: &str = r#"(async () => {
  const captured = (__CAPTURE_TOOL__)(true);
  const tool = captured.tool;
  const requested = __REQUESTED_SECONDS__;
  const expectedIdentity = __EXPECTED_IDENTITY__;
  const read = () => {
    const playing = tool && tool.getStore().playing;
    if (!playing || playing.isLoadingFirst || playing.restoreResource || playing.isPlayingVideo || !playing.playId) return null;
    const native = captured.progress();
    const track = playing.curPlaying && playing.curPlaying.track;
    const artists = track && (track.artists || track.ar);
    const artist = String(playing.resourceArtist || (Array.isArray(artists) ? artists.map(item => item && item.name || '').filter(Boolean).join(', ') : ''));
    const position = Number(native && native.current);
    const duration = Number(playing.resourceDuration);
    const trackId = String(playing.onlineResourceId || '');
    if (!native || native.playId !== playing.playId || !trackId || !Number.isFinite(position) || !Number.isFinite(duration) || position < 0 || duration <= 0 || position > duration) return null;
    return { trackId, playId: playing.playId, title: String(playing.resourceName || track && track.name || ''), artist, position, duration };
  };
  const before = read();
  if (!before) return { verified: false, unsupported: true };
  if (before.title !== expectedIdentity.title || before.artist !== expectedIdentity.artist) return { verified: false, identityMismatch: true };
  if (!Number.isFinite(requested) || requested < 0 || requested >= before.duration) return { verified: false, invalidPosition: true };
  await tool.getDispatch()({ type: 'playing/setPlayingPosition', payload: { duration: requested } });
  const deadline = Date.now() + 4000;
  while (Date.now() <= deadline) {
    const after = read();
    if (!after || after.trackId !== before.trackId || after.playId !== before.playId || after.title !== before.title || after.artist !== before.artist) return { verified: false, trackChanged: true };
    if (Math.abs(after.position - requested) <= 1) {
      return { verified: true, trackId: after.trackId, playId: String(after.playId), title: after.title, artist: after.artist, positionMs: Math.round(after.position * 1000), durationMs: Math.round(after.duration * 1000) };
    }
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  return { verified: false, timedOut: true };
})()"#;

fn cloud_music_seek_expression(
    requested_seconds: f64,
    expected_title: &str,
    expected_artist: &str,
) -> String {
    let requested =
        serde_json::to_string(&requested_seconds).unwrap_or_else(|_| "null".to_string());
    let expected_identity = serde_json::to_string(&json!({
        "title": expected_title,
        "artist": expected_artist,
    }))
    .unwrap_or_else(|_| "null".to_string());
    CLOUD_MUSIC_SEEK_EXPRESSION
        .replace("__CAPTURE_TOOL__", CLOUD_MUSIC_CAPTURE_TOOL)
        .replace("__REQUESTED_SECONDS__", &requested)
        .replace("__EXPECTED_IDENTITY__", &expected_identity)
}

fn runtime_evaluate_params(expression: &str) -> Value {
    json!({
        "expression": expression.replace("__CAPTURE_TOOL__", CLOUD_MUSIC_CAPTURE_TOOL),
        "returnByValue": true,
        "awaitPromise": true
    })
}

fn parse_netease_progress(value: &Value) -> Result<NeteaseProgress, CdpError> {
    let position_ms = value
        .get("positionMs")
        .and_then(Value::as_u64)
        .ok_or(CdpError::InvalidResponse)?;
    let duration_ms = value
        .get("durationMs")
        .and_then(Value::as_u64)
        .ok_or(CdpError::InvalidResponse)?;
    let track_id = required_string(value, "trackId")?;
    let play_id = required_string(value, "playId")?;
    let title = required_string(value, "title")?;
    let artist = required_string(value, "artist")?;
    if position_ms > duration_ms
        || duration_ms == 0
        || track_id.len() > 128
        || play_id.len() > 128
        || title.len() > 1024
        || artist.len() > 1024
    {
        return Err(CdpError::InvalidResponse);
    }
    Ok(NeteaseProgress {
        track_id: track_id.to_string(),
        play_id: play_id.to_string(),
        title: title.to_string(),
        artist: artist.to_string(),
        position_ms,
        duration_ms,
    })
}

fn validate_like_identity(value: &Value, title: &str, artist: &str) -> Result<(), CdpError> {
    // Windows SMTC may expose only the first artist of a collaboration.
    // Match a complete artist name, never a substring of another artist.
    let matches_artist = value.get("artist").and_then(Value::as_str) == Some(artist)
        || (!artist.is_empty() && value.get("primaryArtist").and_then(Value::as_str) == Some(artist));
    if value.get("title").and_then(Value::as_str) != Some(title) || !matches_artist {
        return Err(CdpError::TrackChanged);
    }
    Ok(())
}

fn parse_bound_netease_progress(value: &Value, title: &str, artist: &str) -> Result<NeteaseProgress, CdpError> {
    validate_like_identity(value, title, artist)?;
    let mut progress = parse_netease_progress(value)?;
    // The actor retains its exact SMTC identity fences. Only reconcile the
    // artist label after matching a complete provider artist or primary artist.
    progress.artist = artist.to_string();
    Ok(progress)
}

fn validate_expected_identity(
    progress: &NeteaseProgress,
    expected_title: &str,
    expected_artist: &str,
) -> Result<(), CdpError> {
    if progress.title != expected_title || progress.artist != expected_artist {
        return Err(CdpError::TrackChanged);
    }
    Ok(())
}

fn cloud_music_set_expression(requested: &str) -> String {
    let requested = serde_json::to_string(requested).unwrap_or_else(|_| "null".to_string());
    let expression = r#"(async () => {
  const tool = (__CAPTURE_TOOL__)().tool;
  const modes = ['playOrder', 'playCycle', 'playOneCycle', 'playRandom'];
  const requested = __REQUESTED_MODE__;
  const previous = tool.getStore().playing.playingMode;
  if (!modes.includes(previous) || !modes.includes(requested)) throw new Error('Unsupported playback mode');
  if (previous !== requested) await tool.getDispatch()({ type: 'playing/switchPlayingMode', payload: {
    playingMode: requested, triggerScene: 'miniPlayer', HeartBeatFlage: false
  } });
  const actual = tool.getStore().playing.playingMode;
  return { requested, actual, verified: actual === requested };
})()"#;
    expression
        .replace("__CAPTURE_TOOL__", CLOUD_MUSIC_CAPTURE_TOOL)
        .replace("__REQUESTED_MODE__", &requested)
}

pub struct CdpConnection {
    socket: WebSocket<TcpStream>,
    next_id: u64,
}

impl CdpConnection {
    fn request(
        &mut self,
        method: &str,
        params: &Value,
        deadline: Instant,
    ) -> Result<Value, CdpError> {
        if method.is_empty()
            || method.len() > 128
            || !method
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(CdpError::InvalidCommand);
        }
        let id = self.next_id;
        self.next_id = id.checked_add(1).ok_or(CdpError::InvalidCommand)?;
        let payload = serde_json::to_string(&json!({"id": id, "method": method, "params": params}))
            .map_err(|_| CdpError::InvalidCommand)?;
        if payload.len() > MAX_COMMAND_BYTES {
            return Err(CdpError::InvalidCommand);
        }
        self.socket
            .get_mut()
            .set_write_timeout(Some(remaining_timeout(deadline, IO_TIMEOUT)?))?;
        self.socket.send(Message::Text(payload.into()))?;

        for _ in 0..=MAX_UNMATCHED_MESSAGES {
            self.socket
                .get_mut()
                .set_read_timeout(Some(remaining_timeout(deadline, IO_TIMEOUT)?))?;
            let response = self.socket.read()?;
            let Message::Text(response) = response else {
                if response.is_close() {
                    return Err(CdpError::Protocol);
                }
                continue;
            };
            let response: Value =
                serde_json::from_str(response.as_str()).map_err(|_| CdpError::InvalidResponse)?;
            if response.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if response.get("error").is_some() {
                return Err(CdpError::Protocol);
            }
            return response
                .get("result")
                .cloned()
                .ok_or(CdpError::InvalidResponse);
        }
        Err(CdpError::TimedOut)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn favorite_identity_accepts_smtc_primary_artist_but_rejects_other_tracks() {
        let state = serde_json::json!({"title":"you know 2(Phonk)","artist":"Trispect, Kyrex","primaryArtist":"Trispect"});
        assert!(super::validate_like_identity(&state, "you know 2(Phonk)", "Trispect").is_ok());
        assert!(super::validate_like_identity(&state, "you know 2(Phonk)", "Trispect, Kyrex").is_ok());
        assert!(super::validate_like_identity(&state, "Other", "Trispect").is_err());
        assert!(super::validate_like_identity(&state, "you know 2(Phonk)", "Tri").is_err());
        assert!(super::validate_like_identity(&state, "you know 2(Phonk)", "Other artist").is_err());
    }
    #[test]
    fn local_cdp_connection_sends_small_requests_without_nagle_delay() {
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let client = super::LocalCdpClient::new(listener.local_addr().unwrap().port()).unwrap();
        let stream = client
            .connect_tcp(std::time::Instant::now() + std::time::Duration::from_secs(1))
            .unwrap();
        assert!(stream.nodelay().unwrap());
    }
    use super::*;
    use std::{net::TcpListener, thread};

    fn read_test_response(
        parts: Vec<Vec<u8>>,
        keep_open_after_body: bool,
        fragment_delay: Duration,
        deadline: Duration,
    ) -> Result<Vec<u8>, CdpError> {
        let listener = TcpListener::bind(SocketAddrV4::new(LOOPBACK, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let server = thread::spawn(move || {
            let (mut peer, _) = listener.accept().unwrap();
            for (index, part) in parts.iter().enumerate() {
                if peer.write_all(part).is_err() {
                    return;
                }
                let _ = peer.flush();
                if index + 1 < parts.len() && !fragment_delay.is_zero() {
                    thread::sleep(fragment_delay);
                }
            }
            if keep_open_after_body {
                let _ = release_rx.recv_timeout(Duration::from_secs(2));
            }
        });

        let mut client = TcpStream::connect(address).unwrap();
        let started = Instant::now();
        let result = read_http_json_response(&mut client, started + deadline);
        drop(client);
        if keep_open_after_body {
            let _ = release_tx.send(());
        }
        server.join().unwrap();
        result
    }

    #[test]
    fn validates_browser_and_page_websocket_host_port_and_path() {
        assert_eq!(
            validate_endpoint(
                "ws://127.0.0.1:9223/devtools/browser/abc-123",
                9223,
                "browser"
            )
            .unwrap(),
            "/devtools/browser/abc-123"
        );
        assert_eq!(
            validate_endpoint("ws://127.0.0.1:9223/devtools/page/page_1", 9223, "page").unwrap(),
            "/devtools/page/page_1"
        );
        for endpoint in [
            "ws://localhost:9223/devtools/page/id",
            "ws://127.0.0.2:9223/devtools/page/id",
            "ws://127.0.0.1:9224/devtools/page/id",
            "ws://127.0.0.1:9223/devtools/browser/id",
            "ws://user@127.0.0.1:9223/devtools/page/id",
            "ws://127.0.0.1:9223/devtools/page/id?host=evil",
            "ws://127.0.0.1:9223/devtools/page/../id",
        ] {
            assert!(
                validate_endpoint(endpoint, 9223, "page").is_err(),
                "{endpoint}"
            );
        }
    }

    #[test]
    fn maps_only_the_four_known_cloud_music_modes() {
        let mappings = [
            (PlaybackMode::Sequential, "playOrder"),
            (PlaybackMode::RepeatList, "playCycle"),
            (PlaybackMode::RepeatOne, "playOneCycle"),
            (PlaybackMode::Shuffle, "playRandom"),
        ];
        for (mode, provider_value) in mappings {
            assert_eq!(mode_from_provider(provider_value), Some(mode));
            assert_eq!(provider_mode(mode), provider_value);
        }
        assert_eq!(mode_from_provider("playAi"), None);
        assert_eq!(mode_from_provider("playFm"), None);
        assert_eq!(mode_from_provider("futureMode"), None);
    }

    #[test]
    fn cycles_through_only_supported_cloud_music_modes() {
        let sequence = [
            PlaybackMode::Sequential,
            PlaybackMode::RepeatList,
            PlaybackMode::RepeatOne,
            PlaybackMode::Shuffle,
            PlaybackMode::Sequential,
        ];
        for pair in sequence.windows(2) {
            assert_eq!(next_mode(pair[0]), pair[1]);
        }
    }

    #[test]
    fn runtime_evaluate_params_expands_mode_and_progress_capture_tool() {
        for (template, expects_progress_capture) in [
            (CLOUD_MUSIC_READ_EXPRESSION, false),
            (CLOUD_MUSIC_PROGRESS_EXPRESSION, true),
        ] {
            let params = runtime_evaluate_params(template);
            let expression = params
                .get("expression")
                .and_then(Value::as_str)
                .expect("Runtime.evaluate expression string");
            assert!(!expression.contains("__CAPTURE_TOOL__"));
            assert!(expression.contains("function captureTool(withProgress = false)"));
            assert_eq!(
                params.get("returnByValue").and_then(Value::as_bool),
                Some(true)
            );
            assert_eq!(
                params.get("awaitPromise").and_then(Value::as_bool),
                Some(true)
            );
            if expects_progress_capture {
                assert!(expression.contains("const captured = (function captureTool"));
                assert!(expression.contains("})(true);"));
            } else {
                assert!(expression.contains("const { tool } = (function captureTool"));
                assert!(expression.contains("})();"));
            }
        }
    }

    #[test]
    fn recognizes_only_netease_page_schemes_and_initialization_urls() {
        for url in [
            "orpheus://orpheus/pub/app.html",
            "https://orpheus.example/app.html",
            "file:///C:/Program%20Files/NetEase/CloudMusic/app.html",
        ] {
            assert!(is_netease_page_url(url), "{url}");
        }
        for url in [
            "https://example.com/",
            "https://notorpheus.example/app.html",
            "file:///C:/Program%20Files/Other/app.html",
            "file:///C:/CloudMusic/other.html",
        ] {
            assert!(!is_netease_page_url(url), "{url}");
        }
    }

    #[test]
    fn rejects_incomplete_oversized_or_chunked_http_bodies() {
        let valid =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
        assert_eq!(parse_http_json_body(valid).unwrap(), &b"{}"[..]);
        let incomplete =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 3\r\n\r\n{}";
        assert!(parse_http_json_body(incomplete).is_err());
        let chunked = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n{}";
        assert!(parse_http_json_body(chunked).is_err());
        let extra =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}x";
        assert!(parse_http_json_body(extra).is_err());
        let duplicate_length = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\ncontent-length: 2\r\n\r\n{}";
        assert!(parse_http_json_body(duplicate_length).is_err());
        let oversized = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
            MAX_BODY_BYTES + 1
        );
        assert!(parse_http_json_body(oversized.as_bytes()).is_err());
    }

    #[test]
    fn reads_content_length_body_while_server_keeps_connection_open() {
        let header =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n";
        let body = b"{}";
        let result = read_test_response(
            vec![header.to_vec(), body.to_vec()],
            true,
            Duration::from_millis(20),
            Duration::from_secs(1),
        );
        assert_eq!(result.unwrap(), body);
    }

    #[test]
    fn rejects_truncated_content_length_body_when_peer_closes() {
        let response =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 3\r\n\r\n{}";
        let result = read_test_response(
            vec![response.to_vec()],
            false,
            Duration::ZERO,
            Duration::from_secs(1),
        );
        assert!(matches!(result, Err(CdpError::InvalidResponse)));
    }

    #[test]
    fn slow_fragmented_http_response_respects_the_overall_deadline() {
        let response =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
        let parts = response.chunks(10).map(|chunk| chunk.to_vec()).collect();
        let result = read_test_response(
            parts,
            false,
            Duration::from_millis(60),
            Duration::from_millis(220),
        );
        assert!(matches!(result, Err(CdpError::TimedOut)));
    }

    #[test]
    fn parses_progress_sample_with_track_and_play_instance_identity() {
        let value = json!({
            "ready": true,
            "trackId": "track-17",
            "playId": "play-29",
            "title": "Song",
            "artist": "Artist",
            "positionMs": 12_500,
            "durationMs": 180_000,
        });
        let progress = parse_netease_progress(&value).unwrap();
        assert_eq!(progress.track_id, "track-17");
        assert_eq!(progress.play_id, "play-29");
        assert_eq!(progress.position_ms, 12_500);
        assert!(validate_expected_identity(&progress, "Other", "Artist").is_err());
        assert!(parse_netease_progress(&json!({
            "trackId": "track-17", "positionMs": 10, "durationMs": 180_000,
        }))
        .is_err());
    }

    #[test]
    fn progress_accepts_exact_primary_artist_and_preserves_actor_identity_fences() {
        let value = json!({"trackId":"song-1","playId":"play-1","title":"Novocaine 2","artist":"Cloke, Shiloh Dynasty","primaryArtist":"Cloke","positionMs":2590,"durationMs":147000});
        let progress = parse_bound_netease_progress(&value, "Novocaine 2", "Cloke").unwrap();
        assert_eq!(progress.artist, "Cloke");
        assert_eq!(progress.track_id, "song-1");
        assert_eq!(progress.play_id, "play-1");
        assert!(validate_expected_identity(&progress, "Novocaine 2", "Cloke").is_ok());
        assert!(parse_bound_netease_progress(&value, "Novocaine 2", "Cloke, Shiloh Dynasty").is_ok());
        assert!(parse_bound_netease_progress(&value, "Other song", "Cloke").is_err());
        assert!(parse_bound_netease_progress(&value, "Novocaine 2", "Clo").is_err());
        assert!(parse_bound_netease_progress(&value, "Novocaine 2", "Other artist").is_err());
    }
}
