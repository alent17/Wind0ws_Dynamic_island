//! 数据模型模块
//!
//! 定义应用程序中使用的所有数据结构
//! 包括设置、媒体状态、缓存元数据等

mod cache;
mod media;
mod settings;

pub use cache::{CacheMetadata, CacheStats};
pub use media::{MediaState, MonitorInfo, NeteaseSong, SpectrumData};
pub use settings::AppSettings;
