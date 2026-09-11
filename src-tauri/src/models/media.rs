//! 媒体相关数据模型
//!
//! 定义媒体状态、频谱数据、网易云歌曲信息等

use serde::{Deserialize, Serialize};

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
    pub upload_bytes_per_second: u64,
    pub download_bytes_per_second: u64,
    pub battery_percent: Option<u8>,
    pub battery_charging: Option<bool>,
    pub weather_temperature: Option<f32>,
    pub weather_code: Option<u16>,
    pub weather_updated_at: Option<u64>,
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
    /// 最后更新时间戳（毫秒）
    pub last_updated_timestamp: u64,
    /// 媒体来源标识（如 netease, spotify 等）
    pub source: String,
    /// 媒体来源显示名称
    pub source_display: String,
    pub capabilities: MediaCapabilities,
    pub shuffle_active: Option<bool>,
    pub repeat_mode: Option<String>,
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
            last_updated_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            source: String::new(),
            source_display: String::new(),
            capabilities: MediaCapabilities::default(),
            shuffle_active: None,
            repeat_mode: None,
        }
    }
}

/// 音频频谱数据
///
/// 用于可视化音频波形
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SpectrumData {
    /// 频谱条数据（归一化值 0.0-1.0）
    pub bars: Vec<f32>,
    /// 数据时间戳
    pub timestamp: u64,
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
