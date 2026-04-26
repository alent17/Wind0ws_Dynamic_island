//! 事件总线模块
//!
//! 提供统一的事件管理和发送机制，支持：
//! - 事件节流（防止高频事件 flooding）
//! - 事件优先级管理
//! - 事件统计和监控
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                      EventBus                           │
//! │  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
//! │  │ AppHandle   │  │ Throttle     │  │ Stats         │  │
//! │  │ (Tauri)     │  │ States       │  │ Tracking      │  │
//! │  └─────────────┘  └──────────────┘  └───────────────┘  │
//! └─────────────────────────────────────────────────────────┘
//!                          │
//!                          ▼
//!              ┌───────────────────────┐
//!              │   Frontend Listeners  │
//!              │   (Svelte Components) │
//!              └───────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! // 初始化（在 setup 中）
//! EVENT_BUS.initialize(app_handle)?;
//!
//! // 发送事件
//! EVENT_BUS.emit("my-event", &payload)?;
//!
//! // 带节流发送
//! EVENT_BUS.emit_with_options(
//!     "frequent-event",
//!     &data,
//!     Some(ThrottleConfig {
//!         interval_ms: 100,
//!         priority: EventPriority::Low,
//!     }),
//! )?;
//! ```

use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

// ============================================================================
// 事件名称常量
// ============================================================================

/// 媒体信息更新事件
///
/// 当播放状态、歌曲信息变化时触发
/// 节流间隔：500ms
pub const EVENT_MEDIA_UPDATE: &str = "media-update";

/// 音频频谱数据事件
///
/// 实时音频可视化数据，高频触发
/// 节流间隔：50ms
pub const EVENT_AUDIO_SPECTRUM: &str = "audio-spectrum";

/// 设置更新事件
///
/// 当设置被保存到文件后触发
pub const EVENT_SETTINGS_UPDATED: &str = "settings-updated";

/// 设置变更事件
///
/// 单个设置项变更时触发
pub const EVENT_SETTINGS_CHANGED: &str = "settings-changed";

/// 主题变更事件
pub const EVENT_THEME_CHANGED: &str = "theme-changed";

/// 圆角半径变更事件
pub const EVENT_CORNER_RADIUS_CHANGED: &str = "corner-radius-changed";

/// 浮动窗口关闭事件
pub const EVENT_FLOATING_WINDOW_CLOSED: &str = "floating-window-closed";

/// 浮动窗口锁定状态变更事件
pub const EVENT_LOCK_FLOATING_WINDOW_CHANGED: &str = "lock-floating-window-changed";

/// 窗口置顶状态变更事件
pub const EVENT_ALWAYS_ON_TOP_CHANGED: &str = "always-on-top-changed";

/// 高清封面设置变更事件
pub const EVENT_HD_COVER_CHANGED: &str = "hd-cover-changed";

/// 像素艺术效果变更事件
pub const EVENT_PIXEL_ART_CHANGED: &str = "pixel-art-changed";

/// 半色调效果变更事件
pub const EVENT_HALFTONE_CHANGED: &str = "halftone-changed";

/// MV 播放状态变更事件
pub const EVENT_MV_PLAYBACK_CHANGED: &str = "mv-playback-changed";

// ============================================================================
// 事件优先级
// ============================================================================

/// 事件优先级枚举
///
/// 用于控制节流行为和事件处理顺序
/// 数值越小优先级越高
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// 关键事件：错误、状态变更
    ///
    /// 这类事件不会被清理，始终保留在节流状态中
    Critical = 0,

    /// 高优先级：用户交互
    ///
    /// 这类事件保留时间较长
    High = 1,

    /// 普通优先级：设置更新
    Normal = 2,

    /// 低优先级：频谱数据
    ///
    /// 这类事件可以频繁丢弃
    Low = 3,
}

// ============================================================================
// 节流配置
// ============================================================================

/// 节流配置
///
/// 定义事件的节流行为
pub struct ThrottleConfig {
    /// 节流间隔（毫秒）
    ///
    /// 两次事件发送之间的最小间隔
    pub interval_ms: u64,

    /// 事件优先级
    ///
    /// 影响节流状态的保留时间
    pub priority: EventPriority,
}

// ============================================================================
// 事件统计
// ============================================================================

/// 事件统计信息
///
/// 用于监控事件发送情况，调试和性能分析
#[derive(Clone, Default)]
pub struct EventStats {
    /// 成功发送的事件总数
    pub total_emitted: u64,

    /// 因节流被丢弃的事件总数
    pub total_dropped: u64,

    /// 最后一次发送时间
    pub last_emit_time: Option<Instant>,
}

// ============================================================================
// 事件总线
// ============================================================================

/// 事件总线 - 统一管理事件发送
///
/// ## 线程安全
///
/// 所有内部状态都使用 `Arc<Mutex>` 包装，支持多线程访问。
///
/// ## 节流机制
///
/// 当配置了节流时，如果在指定间隔内重复发送同一事件，
/// 后续的事件会被丢弃，避免前端被高频事件淹没。
///
/// ## 统计追踪
///
/// 记录每个事件的发送次数和丢弃次数，便于调试和性能分析。
pub struct EventBus {
    /// Tauri AppHandle 引用
    ///
    /// 用于调用 `emit` 方法向前端发送事件
    app_handle: Arc<Mutex<Option<AppHandle>>>,

    /// 节流状态映射
    ///
    /// Key: 事件名称
    /// Value: (上次发送时间, 节流配置)
    throttle_states: Arc<Mutex<HashMap<String, (Instant, ThrottleConfig)>>>,

    /// 事件统计映射
    ///
    /// Key: 事件名称
    /// Value: 统计信息
    stats: Arc<Mutex<HashMap<String, EventStats>>>,

    /// 是否启用事件总线
    ///
    /// 禁用后所有事件都会被静默丢弃
    enabled: Arc<Mutex<bool>>,
}

impl EventBus {
    /// 创建新的事件总线
    ///
    /// 初始状态下：
    /// - AppHandle 为 None（需要后续初始化）
    /// - 事件总线处于启用状态
    /// - 无节流状态和统计数据
    pub fn new() -> Self {
        Self {
            app_handle: Arc::new(Mutex::new(None)),
            throttle_states: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(HashMap::new())),
            enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// 初始化 AppHandle
    ///
    /// 必须在 Tauri 的 setup 阶段调用，否则事件无法发送。
    ///
    /// # 参数
    ///
    /// - `app`: Tauri AppHandle 实例
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(())`，失败返回错误信息
    pub fn initialize(&self, app: AppHandle) -> Result<(), String> {
        let mut handle = self.app_handle.lock().map_err(|_| "Failed to lock app_handle")?;
        *handle = Some(app);
        Ok(())
    }

    /// 启用/禁用事件总线
    ///
    /// 禁用后，所有 `emit` 调用都会立即返回 `Ok(())`，
    /// 但不会实际发送任何事件。
    ///
    /// # 参数
    ///
    /// - `enabled`: true 启用，false 禁用
    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        let mut e = self.enabled.lock().map_err(|_| "Failed to lock enabled")?;
        *e = enabled;
        Ok(())
    }

    /// 发送事件（自动节流）
    ///
    /// 使用默认配置发送事件，不进行节流。
    ///
    /// # 参数
    ///
    /// - `event`: 事件名称
    /// - `payload`: 事件负载，必须实现 `Serialize + Clone`
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(())`，失败返回错误信息
    pub fn emit<T: Serialize + Clone>(
        &self,
        event: &str,
        payload: T,
    ) -> Result<(), String> {
        self.emit_with_options(event, payload, None)
    }

    /// 发送事件（带选项）
    ///
    /// 核心发送方法，支持节流配置。
    ///
    /// # 处理流程
    ///
    /// 1. 检查事件总线是否启用
    /// 2. 如果配置了节流，检查是否在节流间隔内
    /// 3. 获取 AppHandle 并调用 Tauri 的 emit
    /// 4. 更新统计数据
    ///
    /// # 参数
    ///
    /// - `event`: 事件名称
    /// - `payload`: 事件负载
    /// - `throttle_config`: 可选的节流配置
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(())`，被节流时也返回 `Ok(())`
    pub fn emit_with_options<T: Serialize + Clone>(
        &self,
        event: &str,
        payload: T,
        throttle_config: Option<ThrottleConfig>,
    ) -> Result<(), String> {
        // 检查是否启用
        {
            let enabled = self.enabled.lock().map_err(|_| "Lock failed")?;
            if !*enabled {
                return Ok(());
            }
        }

        // 检查节流
        if let Some(config) = throttle_config {
            let now = Instant::now();
            let mut states = self.throttle_states.lock().map_err(|_| "Lock failed")?;

            // 检查是否在节流间隔内
            if let Some((last_time, existing_config)) = states.get(event) {
                if now.duration_since(*last_time).as_millis() < existing_config.interval_ms as u128
                {
                    // 更新丢弃统计
                    let mut stats = self.stats.lock().map_err(|_| "Lock failed")?;
                    let entry = stats.entry(event.to_string()).or_default();
                    entry.total_dropped += 1;
                    return Ok(()); // 被节流，直接返回
                }
            }

            // 更新节流状态
            states.insert(event.to_string(), (now, config));
        }

        // 获取 AppHandle
        let app = {
            let handle = self.app_handle.lock().map_err(|_| "Lock failed")?;
            match handle.as_ref() {
                Some(app) => app.clone(),
                None => return Err("AppHandle not initialized".to_string()),
            }
        };

        // 发送事件
        match app.emit(event, payload) {
            Ok(_) => {
                // 更新统计
                let mut stats = self.stats.lock().map_err(|_| "Failed to lock stats")?;
                let entry = stats.entry(event.to_string()).or_default();
                entry.total_emitted += 1;
                entry.last_emit_time = Some(Instant::now());
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("[EventBus] 发送事件失败 {}: {}", event, e);
                Err(err_msg)
            }
        }
    }

    /// 批量发送事件
    ///
    /// 用于一次性发送多个相关事件，如设置更新场景。
    ///
    /// # 参数
    ///
    /// - `events`: 事件列表，每个元素是 (事件名称, 负载) 元组
    ///
    /// # 返回
    ///
    /// 每个事件的发送结果列表
    pub fn emit_batch<T: Serialize + Clone>(
        &self,
        events: Vec<(&str, T)>,
    ) -> Vec<Result<(), String>> {
        events
            .into_iter()
            .map(|(event, payload)| self.emit(event, payload))
            .collect()
    }

    /// 获取事件统计
    ///
    /// 返回所有事件的统计信息副本。
    ///
    /// # 返回
    ///
    /// 事件名称到统计信息的映射
    pub fn get_stats(&self) -> HashMap<String, EventStats> {
        match self.stats.lock() {
            Ok(stats) => stats.clone(),
            Err(_) => HashMap::new(),
        }
    }

    /// 重置统计
    ///
    /// 清空所有事件的统计数据。
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.clear();
        }
    }

    /// 清理过期的节流状态
    ///
    /// 移除超过 1 分钟且优先级低于 High 的节流状态，
    /// 防止内存无限增长。
    ///
    /// 建议定期调用（如每 5 分钟）。
    pub fn cleanup_throttle_states(&self) {
        if let Ok(mut states) = self.throttle_states.lock() {
            let now = Instant::now();
            states.retain(|_, (last_time, config)| {
                // 保留 1 分钟内的状态，或高优先级事件的状态
                now.duration_since(*last_time).as_secs() < 60
                    || config.priority <= EventPriority::High
            });
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 全局实例
// ============================================================================

/// 全局事件总线实例
///
/// 使用 `lazy_static` 确保全局唯一，可在任何地方访问。
///
/// ## 使用示例
///
/// ```rust
/// use crate::event_bus::EVENT_BUS;
///
/// // 发送事件
/// EVENT_BUS.emit("my-event", &data)?;
/// ```
lazy_static::lazy_static! {
    pub static ref EVENT_BUS: EventBus = EventBus::new();
}

// ============================================================================
// 便捷函数
// ============================================================================

/// 发送媒体更新事件
///
/// 使用 500ms 节流，避免频繁更新。
///
/// # 参数
///
/// - `payload`: 媒体状态数据
pub fn emit_media_update<T: Serialize + Clone>(payload: T) -> Result<(), String> {
    EVENT_BUS.emit_with_options(
        EVENT_MEDIA_UPDATE,
        payload,
        Some(ThrottleConfig {
            interval_ms: 500,
            priority: EventPriority::Normal,
        }),
    )
}

/// 发送频谱数据事件
///
/// 使用 50ms 节流，适合实时可视化（约 20 FPS）。
///
/// # 参数
///
/// - `payload`: 频谱数据
pub fn emit_audio_spectrum<T: Serialize + Clone>(payload: T) -> Result<(), String> {
    EVENT_BUS.emit_with_options(
        EVENT_AUDIO_SPECTRUM,
        payload,
        Some(ThrottleConfig {
            interval_ms: 50,
            priority: EventPriority::Low,
        }),
    )
}

/// 发送设置更新事件
///
/// 无节流，确保设置变更立即通知前端。
///
/// # 参数
///
/// - `payload`: 设置数据
pub fn emit_settings_updated<T: Serialize + Clone>(payload: T) -> Result<(), String> {
    EVENT_BUS.emit(EVENT_SETTINGS_UPDATED, payload)
}

/// 发送设置变更事件
///
/// 通知前端某个设置项已变更。
///
/// # 参数
///
/// - `setting_name`: 变更的设置项名称
pub fn emit_settings_changed(setting_name: &str) -> Result<(), String> {
    EVENT_BUS.emit(EVENT_SETTINGS_CHANGED, setting_name)
}

/// 发送主题变更事件
///
/// # 参数
///
/// - `payload`: 主题数据
pub fn emit_theme_changed<T: Serialize + Clone>(payload: T) -> Result<(), String> {
    EVENT_BUS.emit(EVENT_THEME_CHANGED, payload)
}
