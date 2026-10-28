//! 应用程序全局状态
//!
//! 使用 Mutex 实现线程安全的状态共享
//! 通过 Tauri 的状态管理机制注入到命令处理函数中

use super::media_actor::MediaActorHandle;
use crate::models::{AppPreferences, NeteaseProgressMode};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
    Mutex,
};
use tokio::sync::Mutex as AsyncMutex;

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
/// ```rust,ignore
/// #[tauri::command]
/// fn get_preferences(state: State<'_, AppState>) -> AppResult<AppPreferences> {
///     let settings = state.settings.lock()?;
///     Ok(settings.clone())
/// }
/// ```
pub struct AppState {
    /// 应用设置（持久化到配置文件）
    pub settings: Mutex<AppPreferences>,
    /// Serializes the complete merge → persist → publish transaction.
    pub settings_writer: AsyncMutex<()>,
    /// Monotonic revision for successful settings commits.
    pub settings_revision: AtomicU64,
    /// Latest persisted NetEase progress policy, read by the media actor even
    /// when its bounded wake queue is full.
    netease_progress_mode: AtomicU8,
    pub(crate) netease_cdp_available: AtomicBool,
    /// 媒体缓存（运行时临时缓存）
    pub media_cache: Mutex<MediaCache>,
    /// Handle to the sole SMTC snapshot actor, installed during setup.
    pub(crate) media_actor: Mutex<Option<MediaActorHandle>>,
}

impl AppState {
    pub(crate) fn netease_enhanced(&self) -> bool {
        self.settings.lock().is_ok_and(|settings| settings.netease_connection_mode == crate::models::NeteaseConnectionMode::Enhanced)
    }

    pub(crate) fn set_netease_cdp_available(&self, available: bool) {
        if self.netease_cdp_available.swap(available, Ordering::AcqRel) != available {
            if let Ok(actor) = self.media_actor() { actor.wake_for_transport_change(); }
        }
    }

    pub(crate) fn set_netease_progress_mode(&self, mode: NeteaseProgressMode) {
        self.netease_progress_mode.store(
            match mode {
                NeteaseProgressMode::Real => 0,
                NeteaseProgressMode::Estimate => 1,
            },
            Ordering::Release,
        );
    }

    pub(crate) fn netease_progress_mode(&self) -> NeteaseProgressMode {
        match self.netease_progress_mode.load(Ordering::Acquire) {
            1 => NeteaseProgressMode::Estimate,
            _ => NeteaseProgressMode::Real,
        }
    }

    pub(crate) fn set_media_actor(&self, actor: MediaActorHandle) -> Result<(), String> {
        let mut slot = self.media_actor.lock().map_err(|error| error.to_string())?;
        *slot = Some(actor);
        Ok(())
    }

    pub(crate) fn media_actor(&self) -> Result<MediaActorHandle, String> {
        self.media_actor
            .lock()
            .map_err(|error| error.to_string())?
            .clone()
            .ok_or_else(|| "media actor is not ready".to_string())
    }
}

/// 默认状态实现
impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(AppPreferences::default()),
            settings_writer: AsyncMutex::new(()),
            settings_revision: AtomicU64::new(0),
            netease_progress_mode: AtomicU8::new(0),
            netease_cdp_available: AtomicBool::new(false),
            media_cache: Mutex::new(MediaCache::default()),
            media_actor: Mutex::new(None),
        }
    }
}
