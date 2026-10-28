//! 媒体相关数据模型
//!
//! 定义媒体状态、频谱数据、网易云歌曲信息等

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaFeature {
    pub source: String,
    pub readable: bool,
    pub controllable: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseFeatures {
    pub progress: MediaFeature,
    pub like: MediaFeature,
    pub mode: MediaFeature,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCapabilities {
    pub previous: bool,
    pub play_pause: bool,
    pub next: bool,
    pub seek: bool,
    pub shuffle: bool,
    pub repeat: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSessionInfo {
    pub id: String,
    pub display_name: String,
    pub source: String,
    pub is_playing: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCover {
    pub url: String,
    pub provider: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdleSnapshot {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub gpu_percent: Option<f32>,
    pub upload_bytes_per_second: u64,
    pub download_bytes_per_second: u64,
    pub battery_percent: Option<u8>,
    pub battery_charging: Option<bool>,
    pub weather_temperature: Option<f32>,
    pub weather_code: Option<u16>,
    pub weather_updated_at: Option<u64>,
    pub weather_forecast: Vec<WeatherForecastDay>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherSnapshot {
    pub weather_temperature: Option<f32>,
    pub weather_code: Option<u16>,
    pub weather_updated_at: Option<u64>,
    pub weather_forecast: Vec<WeatherForecastDay>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherForecastDay {
    pub date: String,
    pub weather_code: u16,
    pub temperature_max: f32,
    pub temperature_min: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherLocationCandidate {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: String,
    pub admin1: String,
    pub admin2: String,
}

/// 媒体播放状态
///
/// 表示当前正在播放的媒体信息
/// 通过 Windows GlobalSystemMediaTransportControlsSession 获取
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    /// 歌曲标题
    pub title: String,
    /// 艺术家名称
    pub artist: String,
    /// 专辑封面 URL（Base64 或网络地址）
    pub album_art: String,
    /// 是否正在播放
    pub is_playing: bool,
    /// 当前播放位置（毫秒）
    pub position_ms: u64,
    /// 总时长（毫秒）
    pub duration_ms: u64,
    /// Whether the reported timeline has a confirmed anchor for this track.
    #[serde(default = "default_true")]
    pub progress_known: bool,
    /// Last real NetEase CDP availability result; None means not checked yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netease_progress_available: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netease_features: Option<NeteaseFeatures>,
    /// 最后更新时间戳（毫秒）
    pub last_updated_timestamp: u64,
    /// 媒体来源标识（如 netease, spotify 等）
    pub source: String,
    /// 媒体来源显示名称
    pub source_display: String,
    pub capabilities: MediaCapabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shuffle_active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_mode: Option<String>,
    /// Ordering domain for snapshots emitted by the single native media actor.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub native_epoch: String,
    /// Actor-owned counter, kept within JavaScript's exact integer range.
    #[serde(default, skip_serializing_if = "is_zero_revision")]
    pub native_revision: u64,
    /// Actor-owned generation incremented only when the selected media identity changes.
    #[serde(default, skip_serializing_if = "is_zero_revision")]
    pub native_track_generation: u64,
}

fn is_zero_revision(revision: &u64) -> bool {
    *revision == 0
}

fn default_true() -> bool {
    true
}

/// 默认媒体状态实现
impl Default for MediaState {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            album_art: String::new(),
            is_playing: false,
            position_ms: 0,
            duration_ms: 0,
            progress_known: true,
            netease_progress_available: None,
            netease_features: None,
            last_updated_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            source: String::new(),
            source_display: String::new(),
            capabilities: MediaCapabilities::default(),
            shuffle_active: None,
            repeat_mode: None,
            native_epoch: String::new(),
            native_revision: 0,
            native_track_generation: 0,
        }
    }
}

/// 网易云音乐歌曲信息
///
/// 通过网易云 API 获取的附加歌曲信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseSong {
    /// 歌曲时长（毫秒）
    pub duration: Option<u64>,
    /// 专辑封面 URL
    pub album_pic: Option<String>,
    /// MV ID
    pub mv_id: Option<i64>,
    /// MV 播放 URL
    pub mv_url: Option<String>,
}

/// 网易云音乐歌词
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct NeteaseLyric {
    /// 歌词时间点（毫秒）
    pub time: u64,
    /// 歌词文本
    pub text: String,
}

/// 显示器信息
///
/// 用于多显示器环境下的窗口定位
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    /// 显示器索引
    pub index: u32,
    /// 显示器名称
    pub name: String,
    /// 分辨率宽度
    pub width: u32,
    /// 分辨率高度
    pub height: u32,
    /// 是否为主显示器
    pub is_primary: bool,
    pub x: i32,
    pub y: i32,
    pub work_x: i32,
    pub work_y: i32,
    pub work_width: u32,
    pub work_height: u32,
    pub scale_factor: f64,
}
