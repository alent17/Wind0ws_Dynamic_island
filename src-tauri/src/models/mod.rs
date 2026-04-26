//! 数据模型模块
//!
//! 定义应用程序中使用的所有数据结构
//! 包括设置、媒体状态、缓存元数据等

mod settings;
mod media;
mod cache;

pub use settings::AppSettings;
pub use media::{MediaState, SpectrumData, NeteaseSong, NeteaseLyric, MonitorInfo};
pub use cache::{CacheMetadata, CacheStats};
