//! 应用状态管理模块
//!
//! 提供全局共享状态的定义和管理

mod app_state;
mod media_actor;
pub(crate) use media_actor::run_if_netease_seek_basis;

pub use app_state::*;
pub(crate) use media_actor::*;
