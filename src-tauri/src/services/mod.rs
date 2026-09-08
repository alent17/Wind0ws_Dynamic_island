//! 业务服务层模块
//!
//! 实现核心业务逻辑，与命令层分离
//! 命令层负责 Tauri IPC，服务层负责具体业务实现

pub mod cache;
pub mod color;
pub mod image;
pub mod media;
pub mod settings;
pub mod spectrum;

pub use settings::{get_auto_start, read_settings_file, set_auto_start, write_settings_file};
