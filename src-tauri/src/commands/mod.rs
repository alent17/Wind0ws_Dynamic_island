//! Tauri 命令处理模块
//!
//! 提供所有前端可调用的 IPC 命令
//! 命令层只负责参数解析和服务调用，不包含业务逻辑

mod cache;
mod media;
mod monitor;
mod settings;
mod window;

pub use cache::*;
pub use media::*;
pub use monitor::*;
pub use settings::*;
pub use window::*;
