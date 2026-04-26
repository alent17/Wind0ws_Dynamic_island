//! 应用程序全局状态
//!
//! 使用 Mutex 实现线程安全的状态共享
//! 通过 Tauri 的状态管理机制注入到命令处理函数中

use std::sync::Mutex;
use crate::models::AppSettings;

/// 媒体缓存状态
///
/// 用于缓存当前播放媒体的封面图片
/// 避免重复请求和 Base64 编码
#[derive(Default)]
pub struct MediaCache {
    /// 当前音轨 ID（用于判断是否需要更新缓存）
    pub track_id: String,
    /// Base64 编码的封面图片数据
    pub base64_img: String,
}

/// 应用程序全局状态
///
/// 通过 Tauri 的 `.manage()` 注册，在命令中通过 `State` 参数访问
///
/// # 示例
/// ```rust
/// #[tauri::command]
/// fn get_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
///     let settings = state.settings.lock()?;
///     Ok(settings.clone())
/// }
/// ```
pub struct AppState {
    /// 应用设置（持久化到配置文件）
    pub settings: Mutex<AppSettings>,
    /// 媒体缓存（运行时临时缓存）
    pub media_cache: Mutex<MediaCache>,
}

/// 默认状态实现
impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(AppSettings::default()),
            media_cache: Mutex::new(MediaCache::default()),
        }
    }
}
