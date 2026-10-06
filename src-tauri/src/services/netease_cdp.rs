//! Bounded, IPv4-loopback-only CDP access for NetEase playback modes.
//!
//! This module is intentionally not wired to a Tauri command yet. Callers
//! should run its synchronous operations away from the UI thread.

use serde_json::{json, Value};
use std::{
    io::{Read, Write},
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
const IO_TIMEOUT: Duration = Duration::from_millis(500);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(5);

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

#[derive(Debug)]
pub enum CdpError {
    InvalidPort,
    Io(std::io::Error),
    InvalidResponse,
    UnsafeWebSocketEndpoint,
    NoInitializedPage,
    AmbiguousPage,
    UnsupportedCurrentMode,
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
            Self::VerificationFailed => f.write_str("网易云播放模式读回校验失败"),
            Self::InvalidCommand => f.write_str("CDP 命令无效或超出长度限制"),
            Self::TimedOut => f.write_str("本机 CDP 操作超时"),
            Self::Protocol => f.write_str("本机 CDP 返回了错误响应"),
            Self::WebSocket(_) => f.write_str("本机 CDP WebSocket 连接失败"),
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
        let (_, raw_mode) = self.initialized_page(deadline)?;
        Ok(mode_from_provider(&raw_mode))
    }

    /// Sets an allowlisted ordinary mode and verifies the store's read-back.
    /// AI/FM and unknown modes are never overwritten.
    pub fn set_playback_mode(
        self,
        requested: PlaybackMode,
    ) -> Result<VerifiedModeAction, CdpError> {
        let deadline = Instant::now() + OPERATION_TIMEOUT;
        let (mut connection, current) = self.initialized_page(deadline)?;
        if mode_from_provider(&current).is_none() {
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

    fn initialized_page(self, deadline: Instant) -> Result<(CdpConnection, String), CdpError> {
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
            let Ok(response) = connection.request(
                "Runtime.evaluate",
                &json!({
                    "expression": CLOUD_MUSIC_READ_EXPRESSION,
                    "returnByValue": true,
                    "awaitPromise": true
                }),
                deadline,
            ) else {
                continue;
            };
            if response.get("exceptionDetails").is_some() {
                continue;
            }
            if let Some(mode) = response
                .pointer("/result/value/mode")
                .and_then(Value::as_str)
                .filter(|mode| !mode.is_empty() && mode.len() <= 64)
            {
                matches.push((connection, mode.to_owned()));
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
        let max_response = (MAX_HEADER_BYTES + MAX_BODY_BYTES + 1) as u64;
        let mut response = Vec::with_capacity(1024);
        stream.take(max_response).read_to_end(&mut response)?;
        Ok(parse_http_json_body(&response)?.to_vec())
    }

    fn connect_tcp(self, deadline: Instant) -> Result<TcpStream, CdpError> {
        let timeout = remaining_timeout(deadline, CONNECT_TIMEOUT)?;
        let address = SocketAddr::V4(SocketAddrV4::new(LOOPBACK, self.port()));
        Ok(TcpStream::connect_timeout(&address, timeout)?)
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

fn parse_http_json_body(response: &[u8]) -> Result<&[u8], CdpError> {
    if response.len() > MAX_HEADER_BYTES + MAX_BODY_BYTES {
        return Err(CdpError::InvalidResponse);
    }
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or(CdpError::InvalidResponse)?;
    if header_end > MAX_HEADER_BYTES {
        return Err(CdpError::InvalidResponse);
    }
    let headers =
        std::str::from_utf8(&response[..header_end]).map_err(|_| CdpError::InvalidResponse)?;
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
    let body = &response[header_end + 4..];
    if !is_json || body.len() != content_length.ok_or(CdpError::InvalidResponse)? {
        return Err(CdpError::InvalidResponse);
    }
    Ok(body)
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

const CLOUD_MUSIC_CAPTURE_TOOL: &str = r#"function captureTool() {
  const queue = globalThis.webpackJsonp;
  if (!Array.isArray(queue) || queue.push === Array.prototype.push) return {};
  const main = queue.find(chunk => chunk && chunk[1] && chunk[1][8] &&
    chunk[2] && chunk[2].some(entry => entry[0] === 1424));
  if (!main || !String(main[1][8]).includes('this.app._store.getState()')) return {};
  const entries = queue.flatMap(chunk => chunk && Array.isArray(chunk[2]) ? chunk[2] : []);
  const previousEntry = entries.length ? entries[entries.length - 1][0] : 1424;
  const id = '__isle_mode_read_' + Date.now() + '_' + Math.random().toString(36).slice(2);
  let req;
  const chunk = [[], { [id]: (_module, _exports, runtime) => { req = runtime; } }, [[id]]];
  try {
    queue.push(chunk);
    if (!req || !req.c || !req.c[8] || !req.c[previousEntry]) return {};
    const tool = req.c[8].exports.a;
    if (!tool || !tool.inited || !tool.app || !tool.app._store || typeof tool.getStore !== 'function') return {};
    return tool;
  } catch (_) {
    return {};
  } finally {
    if (req) { delete req.m[id]; delete req.c[id]; req.s = previousEntry; }
    const index = queue.indexOf(chunk);
    if (index >= 0) queue.splice(index, 1);
  }
}"#;

const CLOUD_MUSIC_READ_EXPRESSION: &str = r#"(() => {
  const tool = (__CAPTURE_TOOL__)();
  const playing = tool.getStore().playing;
  return playing && typeof playing.playingMode === 'string' ? { mode: playing.playingMode } : {};
})()"#;

fn cloud_music_set_expression(requested: &str) -> String {
    let requested = serde_json::to_string(requested).unwrap_or_else(|_| "null".to_string());
    let expression = r#"(async () => {
  const tool = (__CAPTURE_TOOL__)();
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
    use super::*;

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
    }
}
