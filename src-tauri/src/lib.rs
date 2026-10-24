//! Windows Dynamic Island - 主入口模块
//!
//! 这是一个模仿 macOS Dynamic Island 的 Windows 桌面应用，
//! 提供音乐播放控制和可视化功能。
//!
//! ## 架构概览
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        lib.rs (入口)                        │
//! │  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐  │
//! │  │ 托盘图标    │  │ 媒体监听器   │  │ 音频可视化器     │  │
//! │  │ (Tray)      │  │ (Media)      │  │ (Spectrum)        │  │
//! │  └─────────────┘  └──────────────┘  └───────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//!         │                   │                    │
//!         ▼                   ▼                    ▼
//! ┌─────────────┐  ┌──────────────────┐  ┌───────────────────┐
//! │  Commands   │  │  Services        │  │  EventBus         │
//! │  (IPC)      │  │  (业务逻辑)      │  │  (事件总线)       │
//! └─────────────┘  └──────────────────┘  └───────────────────┘
//! ```
//!
//! ## 主要功能
//!
//! - **媒体控制**: 播放/暂停、上一曲/下一曲
//! - **音频可视化**: 实时频谱显示
//! - **窗口管理**: 多显示器支持、窗口置顶
//! - **缓存系统**: 媒体文件本地缓存
//!
//! ## 模块结构
//!
//! - `error`: 统一错误处理
//! - `models`: 数据模型定义
//! - `state`: 应用状态管理
//! - `services`: 业务服务层
//! - `commands`: Tauri IPC 命令
//! - `event_bus`: 事件总线

// The Tauri-generated resource library embeds Common Controls v6 in the app
// executable. Cargo's library unit-test harness does not receive Tauri's
// `rustc-link-arg-bins`, so link it only into Windows unit-test executables.
#[cfg(all(test, target_os = "windows"))]
#[link(name = "resource", kind = "static")]
unsafe extern "C" {}

use state::{MediaActorHandle, MediaActorMessage};
use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State,
};

// ============================================================================
// 模块声明
// ============================================================================

mod audio;
mod commands;
mod error;
mod event_bus;
mod models;
mod services;
mod state;
mod utils;

// ============================================================================
// 公开导出
// ============================================================================

pub use error::{AppError, AppResult};
// Only the independent example registers this helper; never the product IPC handler.
#[cfg(feature = "compact-native-probe")]
pub use commands::apply_compact_probe_region;
pub use models::{
    AppPreferences, AudioDeviceInfo, CacheStats, MediaState, MonitorInfo, NeteaseSong,
    SystemAudioState,
};
pub use services::{get_auto_start, read_settings_file, set_auto_start, write_settings_file};
pub use state::AppState;

// ============================================================================
// 外部依赖
// ============================================================================

use event_bus::EVENT_BUS;

#[cfg(target_os = "windows")]
use window_vibrancy::{apply_acrylic, clear_acrylic};

// ============================================================================
// 常量定义
// ============================================================================

/// 托盘菜单 - 显示主窗口
const SHOW_MENU_ID: &str = "show";

/// 托盘菜单 - 打开设置
const STUDIO_MENU_ID: &str = "studio";

/// 托盘菜单 - 退出应用
const QUIT_MENU_ID: &str = "quit";

fn resolved_ui_language(preference: &str) -> &'static str {
    match preference {
        "zh-CN" => "zh-CN",
        "ja" => "ja",
        "en" => "en",
        _ => {
            let primary_language =
                unsafe { windows052::Win32::Globalization::GetUserDefaultUILanguage() & 0x03ff };
            match primary_language {
                0x04 => "zh-CN",
                0x11 => "ja",
                _ => "en",
            }
        }
    }
}

fn tray_labels(language: &str) -> (&'static str, &'static str, &'static str) {
    match language {
        "ja" => ("メイン画面を表示", "Isle Studio", "終了"),
        "en" => ("Show main window", "Isle Studio", "Quit"),
        _ => ("显示主窗口", "Isle Studio", "退出"),
    }
}

// ============================================================================
// 数据结构
// ============================================================================

struct SpectrumState(Mutex<(audio::SpectrumCapture, std::collections::HashSet<String>)>);

#[tauri::command]
fn start_spectrum(
    state: State<SpectrumState>,
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<(), String> {
    let mut resources = state.0.lock().map_err(|e| e.to_string())?;
    resources.1.insert(window.label().to_string());
    resources.0.start(app)
}

fn release_spectrum(state: &SpectrumState, label: &str) {
    if let Ok(mut resources) = state.0.lock() {
        resources.1.remove(label);
        if resources.1.is_empty() {
            resources.0.stop();
        }
    }
}

#[tauri::command]
fn stop_spectrum(state: State<SpectrumState>, window: tauri::WebviewWindow) -> Result<(), String> {
    release_spectrum(&state, window.label());
    Ok(())
}

#[tauri::command]
fn restart_spectrum(state: State<SpectrumState>, app: tauri::AppHandle) -> Result<(), String> {
    let resources = state.0.lock().map_err(|e| e.to_string())?;
    if resources.1.is_empty() {
        return Ok(());
    }
    resources.0.stop();
    resources.0.start(app)
}

// ============================================================================
// 媒体监听器
// ============================================================================

fn media_event_payload(
    info: models::MediaState,
    artwork_omitted: bool,
) -> Result<serde_json::Value, serde_json::Error> {
    let mut payload = serde_json::to_value(info)?;
    if artwork_omitted {
        if let Some(fields) = payload.as_object_mut() {
            fields.remove("albumArt");
        }
    }
    Ok(payload)
}

#[derive(Default)]
struct MediaEmitBaseline {
    artwork_track: String,
    artwork: String,
    empty_state_emitted: bool,
    transport: Option<(String, String, String, bool, Option<bool>, Option<String>)>,
}

impl MediaEmitBaseline {
    fn transport_changed(&self, info: &models::MediaState) -> bool {
        self.transport
            .as_ref()
            .is_none_or(|(source, title, artist, playing, shuffle, repeat)| {
                source != &info.source
                    || title != &info.title
                    || artist != &info.artist
                    || *playing != info.is_playing
                    || shuffle != &info.shuffle_active
                    || repeat != &info.repeat_mode
            })
    }

    fn acknowledge_transport(
        &mut self,
        info: &models::MediaState,
        outcome: Option<event_bus::EmitOutcome>,
    ) {
        if outcome == Some(event_bus::EmitOutcome::Emitted) {
            self.transport = Some((
                info.source.clone(),
                info.title.clone(),
                info.artist.clone(),
                info.is_playing,
                info.shuffle_active,
                info.repeat_mode.clone(),
            ));
        }
    }
    fn artwork_unchanged(&self, track: &str, artwork: &str) -> bool {
        track == self.artwork_track && artwork == self.artwork
    }

    fn should_emit_empty_state(&self, missing_session_polls: u8) -> bool {
        missing_session_polls >= 2 && !self.empty_state_emitted
    }

    /// Only a real AppHandle emission acknowledges producer state. Throttled,
    /// disabled, serialization-failed and transport-error attempts are retried
    /// from a future actor snapshot instead of being remembered as delivered.
    fn acknowledge(
        &mut self,
        outcome: Option<event_bus::EmitOutcome>,
        empty_state: bool,
        artwork_candidate: Option<(String, String)>,
    ) {
        if outcome != Some(event_bus::EmitOutcome::Emitted) {
            return;
        }
        if empty_state {
            self.empty_state_emitted = true;
            self.artwork_track.clear();
            self.artwork.clear();
        } else if let Some((track, artwork)) = artwork_candidate {
            self.artwork_track = track;
            self.artwork = artwork;
        }
    }
}

/// 启动媒体状态监听器
///
/// 在后台线程中持续监听系统媒体播放状态，
/// 并通过事件总线向前端发送更新。
///
/// ## 工作流程
///
/// 1. 初始化 COM 组件（Windows API 要求）
/// 2. 每秒查询一次媒体状态
/// 3. 通过 EventBus 发送 `media-update` 事件
///
/// ## 线程安全
///
/// 运行在独立的后台线程中，不会阻塞主线程。
fn media_poll_interval(
    has_session: bool,
    is_playing: bool,
    missing_session_confirmed: bool,
) -> std::time::Duration {
    if has_session && is_playing {
        std::time::Duration::from_secs(1)
    } else if has_session {
        std::time::Duration::from_secs(2)
    } else if missing_session_confirmed {
        std::time::Duration::from_secs(5)
    } else {
        // Confirm a transient missing session quickly before entering the
        // idle cadence.
        std::time::Duration::from_secs(1)
    }
}

struct MediaSessionSubscriptions {
    session: windows052::Media::Control::GlobalSystemMediaTransportControlsSession,
    timeline: Option<windows052::Foundation::EventRegistrationToken>,
    playback: Option<windows052::Foundation::EventRegistrationToken>,
    media: Option<windows052::Foundation::EventRegistrationToken>,
    probe: Option<MediaProbeObserver>,
}

struct MediaManagerSubscriptions {
    manager: windows052::Media::Control::GlobalSystemMediaTransportControlsSessionManager,
    current_session: Option<windows052::Foundation::EventRegistrationToken>,
    sessions: Option<windows052::Foundation::EventRegistrationToken>,
    probe: Option<MediaProbeObserver>,
}

#[derive(Clone, Copy)]
enum MediaRefreshReason {
    ManagerReady,
    Event,
    Fallback,
}

impl MediaRefreshReason {
    #[cfg(feature = "c2-media-probe")]
    fn label(self) -> &'static str {
        match self {
            Self::ManagerReady => "manager-ready",
            Self::Event => "event",
            Self::Fallback => "fallback",
        }
    }
}

#[derive(Default)]
struct MediaProbeCounters {
    manager_acquired: u64,
    manager_failed: u64,
    manager_ready_refreshes: u64,
    event_refreshes: u64,
    fallback_refreshes: u64,
    snapshots_nonempty: u64,
    snapshots_empty: u64,
    snapshot_errors: u64,
    subscriptions_added: u64,
    subscriptions_removed: u64,
    subscription_remove_errors: u64,
    snapshot_total_micros: u128,
    snapshot_max_micros: u128,
    snapshot_attempts: u64,
    metadata_reads: u64,
    metadata_reuses: u64,
    callback_manager_current_session: u64,
    callback_manager_sessions: u64,
    callback_session_timeline: u64,
    callback_session_playback: u64,
    callback_session_media: u64,
    refresh_enqueued: u64,
    refresh_queue_full: u64,
    refresh_queue_disconnected: u64,
    playback_state_changes: u64,
    selected_session_changes: u64,
    #[cfg(feature = "c2-media-probe")]
    poll_records: u64,
    #[cfg(feature = "c2-media-probe")]
    netease_snapshots: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_real_mode_snapshots: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_due: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_started: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_inflight_busy: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_read_ok: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_read_identity_mismatch: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_read_no_page: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_read_timeout: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_read_other_error: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_basis_discard: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_published: u64,
    #[cfg(feature = "c2-media-probe")]
    progress_publish_rejected: u64,
    #[cfg(feature = "c2-media-probe")]
    empty_emit_calls: u64,
    #[cfg(feature = "c2-media-probe")]
    empty_emit_success: u64,
    #[cfg(feature = "c2-media-probe")]
    empty_emit_error: u64,
    #[cfg(feature = "c2-media-probe")]
    empty_emit_throttled: u64,
    #[cfg(feature = "c2-media-probe")]
    empty_emit_disabled: u64,
    #[cfg(feature = "c2-media-probe")]
    empty_payload_error: u64,
}

#[derive(Clone)]
struct MediaProbeObserver {
    counters: std::sync::Arc<std::sync::Mutex<MediaProbeCounters>>,
    #[cfg(feature = "c2-media-probe")]
    started: std::time::Instant,
}

impl Default for MediaProbeObserver {
    fn default() -> Self {
        Self {
            counters: std::sync::Arc::default(),
            #[cfg(feature = "c2-media-probe")]
            started: std::time::Instant::now(),
        }
    }
}

impl MediaProbeObserver {
    fn update(&self, update: impl FnOnce(&mut MediaProbeCounters)) {
        if let Ok(mut counters) = self.counters.lock() {
            update(&mut counters);
        }
    }

    #[cfg(feature = "c2-media-probe")]
    fn mark_ready(&self, manager_tokens: u64, session_tokens: u64) {
        let elapsed_ms = self.started.elapsed().as_millis();
        println!(
            "media_probe_ready elapsed_ms={elapsed_ms} manager_tokens={manager_tokens} session_tokens={session_tokens} total_tokens={}",
            manager_tokens + session_tokens,
        );
    }

    #[cfg(feature = "c2-media-probe")]
    fn record_poll(
        &self,
        reason: MediaRefreshReason,
        snapshot: &'static str,
        next_interval: std::time::Duration,
    ) {
        let Ok(mut counters) = self.counters.lock() else {
            return;
        };
        if counters.poll_records >= 16 {
            return;
        }
        counters.poll_records += 1;
        let sequence = counters.poll_records;
        drop(counters);
        println!(
            "media_probe_poll seq={sequence} elapsed_ms={} reason={} snapshot={snapshot} next_interval_ms={}",
            self.started.elapsed().as_millis(),
            reason.label(),
            next_interval.as_millis(),
        );
    }

    #[cfg(feature = "c2-media-probe")]
    fn record_progress_read_result(&self, kind: &'static str) {
        self.update(|counts| match kind {
            "ok" => counts.progress_read_ok += 1,
            "identity_mismatch" => counts.progress_read_identity_mismatch += 1,
            "no_initialized_page" => counts.progress_read_no_page += 1,
            "timeout" => counts.progress_read_timeout += 1,
            _ => counts.progress_read_other_error += 1,
        });
    }

    #[cfg(feature = "c2-media-probe")]
    fn summary(&self, elapsed: std::time::Duration) -> String {
        let Ok(counters) = self.counters.lock() else {
            return "media_probe_summary counters=unavailable".to_string();
        };
        let elapsed_ms = elapsed.as_millis();
        let mean_micros = if counters.snapshot_attempts == 0 {
            0
        } else {
            counters.snapshot_total_micros / counters.snapshot_attempts as u128
        };
        format!(
            "media_probe_summary elapsed_ms={elapsed_ms} manager_ok={} manager_error={} refresh_manager_ready={} refresh_event={} refresh_fallback={} callback_current={} callback_sessions={} callback_timeline={} callback_playback={} callback_media={} refresh_enqueued={} refresh_queue_full={} refresh_queue_disconnected={} playback_changes={} selected_session_changes={} metadata_reads={} metadata_reuses={} poll_records={} netease_snapshots={} progress_real_mode_snapshots={} progress_due={} progress_started={} progress_inflight_busy={} progress_read_ok={} progress_read_identity_mismatch={} progress_read_no_page={} progress_read_timeout={} progress_read_other_error={} progress_basis_discard={} progress_published={} progress_publish_rejected={} empty_emit_calls={} empty_emit_success={} empty_emit_throttled={} empty_emit_disabled={} empty_emit_error={} empty_payload_error={} snapshot_ok={} snapshot_empty={} snapshot_error={} subscriptions_added={} subscriptions_removed={} subscription_remove_error={} snapshot_mean_us={mean_micros} snapshot_max_us={}",
            counters.manager_acquired,
            counters.manager_failed,
            counters.manager_ready_refreshes,
            counters.event_refreshes,
            counters.fallback_refreshes,
            counters.callback_manager_current_session,
            counters.callback_manager_sessions,
            counters.callback_session_timeline,
            counters.callback_session_playback,
            counters.callback_session_media,
            counters.refresh_enqueued,
            counters.refresh_queue_full,
            counters.refresh_queue_disconnected,
            counters.playback_state_changes,
            counters.selected_session_changes,
            counters.metadata_reads,
            counters.metadata_reuses,
            counters.poll_records,
            counters.netease_snapshots,
            counters.progress_real_mode_snapshots,
            counters.progress_due,
            counters.progress_started,
            counters.progress_inflight_busy,
            counters.progress_read_ok,
            counters.progress_read_identity_mismatch,
            counters.progress_read_no_page,
            counters.progress_read_timeout,
            counters.progress_read_other_error,
            counters.progress_basis_discard,
            counters.progress_published,
            counters.progress_publish_rejected,
            counters.empty_emit_calls,
            counters.empty_emit_success,
            counters.empty_emit_throttled,
            counters.empty_emit_disabled,
            counters.empty_emit_error,
            counters.empty_payload_error,
            counters.snapshots_nonempty,
            counters.snapshots_empty,
            counters.snapshot_errors,
            counters.subscriptions_added,
            counters.subscriptions_removed,
            counters.subscription_remove_errors,
            counters.snapshot_max_micros,
        )
    }
}

#[derive(Clone, Copy)]
enum MediaProbeCallbackKind {
    ManagerCurrentSession,
    ManagerSessions,
    SessionTimeline,
    SessionPlayback,
    SessionMedia,
}

fn send_media_refresh(
    events: &std::sync::mpsc::SyncSender<MediaActorMessage>,
    probe: Option<&MediaProbeObserver>,
    metadata_dirty: Option<&std::sync::atomic::AtomicU64>,
    refresh_generation: &std::sync::atomic::AtomicU64,
    refresh_signal_pending: &std::sync::atomic::AtomicBool,
    callback: MediaProbeCallbackKind,
) {
    let _ = refresh_generation.fetch_update(
        std::sync::atomic::Ordering::AcqRel,
        std::sync::atomic::Ordering::Acquire,
        |generation| Some(generation.saturating_add(1)),
    );
    if matches!(
        callback,
        MediaProbeCallbackKind::ManagerCurrentSession
            | MediaProbeCallbackKind::ManagerSessions
            | MediaProbeCallbackKind::SessionMedia
    ) {
        if let Some(generation) = metadata_dirty {
            generation.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        }
    }
    if let Some(probe) = probe {
        probe.update(|counts| match callback {
            MediaProbeCallbackKind::ManagerCurrentSession => {
                counts.callback_manager_current_session += 1
            }
            MediaProbeCallbackKind::ManagerSessions => counts.callback_manager_sessions += 1,
            MediaProbeCallbackKind::SessionTimeline => counts.callback_session_timeline += 1,
            MediaProbeCallbackKind::SessionPlayback => counts.callback_session_playback += 1,
            MediaProbeCallbackKind::SessionMedia => counts.callback_session_media += 1,
        });
    }

    let outcome = if refresh_signal_pending
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
        )
        .is_ok()
    {
        Some(match events.try_send(MediaActorMessage::Refresh) {
            Ok(()) => 0,
            Err(std::sync::mpsc::TrySendError::Full(_)) => {
                refresh_signal_pending.store(false, std::sync::atomic::Ordering::Release);
                1
            }
            Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                refresh_signal_pending.store(false, std::sync::atomic::Ordering::Release);
                2
            }
        })
    } else {
        None
    };
    if let (Some(probe), Some(outcome)) = (probe, outcome) {
        probe.update(|counts| match outcome {
            0 => counts.refresh_enqueued += 1,
            1 => counts.refresh_queue_full += 1,
            _ => counts.refresh_queue_disconnected += 1,
        });
    }
}

#[cfg(feature = "c2-media-probe")]
pub struct C2MediaProbeTask {
    stop: MediaActorHandle,
    join: Option<std::thread::JoinHandle<()>>,
    observer: MediaProbeObserver,
    started: std::time::Instant,
}

#[cfg(feature = "c2-media-probe")]
impl C2MediaProbeTask {
    pub fn is_finished(&self) -> bool {
        self.join
            .as_ref()
            .is_none_or(std::thread::JoinHandle::is_finished)
    }

    /// Request the worker to unregister its SMTC event tokens, then wait for it.
    pub fn stop_and_join(mut self) -> Result<String, String> {
        self.stop.stop();
        let Some(join) = self.join.take() else {
            return Err("media listener worker handle is missing".to_string());
        };
        join.join()
            .map_err(|_| "media listener worker panicked".to_string())?;
        Ok(self.observer.summary(self.started.elapsed()))
    }
}

impl MediaManagerSubscriptions {
    fn remove(self) {
        if let Some(token) = self.current_session {
            let removed = self.manager.RemoveCurrentSessionChanged(token).is_ok();
            if let Some(probe) = &self.probe {
                probe.update(|counts| {
                    counts.subscriptions_removed += u64::from(removed);
                    counts.subscription_remove_errors += u64::from(!removed);
                });
            }
        }
        if let Some(token) = self.sessions {
            let removed = self.manager.RemoveSessionsChanged(token).is_ok();
            if let Some(probe) = &self.probe {
                probe.update(|counts| {
                    counts.subscriptions_removed += u64::from(removed);
                    counts.subscription_remove_errors += u64::from(!removed);
                });
            }
        }
    }
}

impl MediaSessionSubscriptions {
    fn remove(self) {
        if let Some(token) = self.timeline {
            let removed = self.session.RemoveTimelinePropertiesChanged(token).is_ok();
            if let Some(probe) = &self.probe {
                probe.update(|counts| {
                    counts.subscriptions_removed += u64::from(removed);
                    counts.subscription_remove_errors += u64::from(!removed);
                });
            }
        }
        if let Some(token) = self.playback {
            let removed = self.session.RemovePlaybackInfoChanged(token).is_ok();
            if let Some(probe) = &self.probe {
                probe.update(|counts| {
                    counts.subscriptions_removed += u64::from(removed);
                    counts.subscription_remove_errors += u64::from(!removed);
                });
            }
        }
        if let Some(token) = self.media {
            let removed = self.session.RemoveMediaPropertiesChanged(token).is_ok();
            if let Some(probe) = &self.probe {
                probe.update(|counts| {
                    counts.subscriptions_removed += u64::from(removed);
                    counts.subscription_remove_errors += u64::from(!removed);
                });
            }
        }
    }
}

fn subscribe_media_session(
    session: windows052::Media::Control::GlobalSystemMediaTransportControlsSession,
    events: std::sync::mpsc::SyncSender<MediaActorMessage>,
    probe: Option<MediaProbeObserver>,
    metadata_dirty: std::sync::Arc<std::sync::atomic::AtomicU64>,
    refresh_generation: std::sync::Arc<std::sync::atomic::AtomicU64>,
    refresh_signal_pending: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> MediaSessionSubscriptions {
    let timeline_events = events.clone();
    let timeline_probe = probe.clone();
    let timeline_refresh_generation = refresh_generation.clone();
    let timeline_refresh_pending = refresh_signal_pending.clone();
    let timeline = session
        .TimelinePropertiesChanged(&windows052::Foundation::TypedEventHandler::<
            windows052::Media::Control::GlobalSystemMediaTransportControlsSession,
            windows052::Media::Control::TimelinePropertiesChangedEventArgs,
        >::new(
            move |_, _| -> windows052::core::Result<()> {
                send_media_refresh(
                    &timeline_events,
                    timeline_probe.as_ref(),
                    None,
                    &timeline_refresh_generation,
                    &timeline_refresh_pending,
                    MediaProbeCallbackKind::SessionTimeline,
                );
                Ok(())
            },
        ))
        .ok();

    let playback_events = events.clone();
    let playback_probe = probe.clone();
    let playback_refresh_generation = refresh_generation.clone();
    let playback_refresh_pending = refresh_signal_pending.clone();
    let playback = session
        .PlaybackInfoChanged(&windows052::Foundation::TypedEventHandler::<
            windows052::Media::Control::GlobalSystemMediaTransportControlsSession,
            windows052::Media::Control::PlaybackInfoChangedEventArgs,
        >::new(
            move |_, _| -> windows052::core::Result<()> {
                send_media_refresh(
                    &playback_events,
                    playback_probe.as_ref(),
                    None,
                    &playback_refresh_generation,
                    &playback_refresh_pending,
                    MediaProbeCallbackKind::SessionPlayback,
                );
                Ok(())
            },
        ))
        .ok();

    let media_events = events;
    let media_probe = probe.clone();
    let media_dirty = metadata_dirty;
    let media_refresh_generation = refresh_generation;
    let media_refresh_pending = refresh_signal_pending;
    let media = session
        .MediaPropertiesChanged(&windows052::Foundation::TypedEventHandler::<
            windows052::Media::Control::GlobalSystemMediaTransportControlsSession,
            windows052::Media::Control::MediaPropertiesChangedEventArgs,
        >::new(
            move |_, _| -> windows052::core::Result<()> {
                send_media_refresh(
                    &media_events,
                    media_probe.as_ref(),
                    Some(&media_dirty),
                    &media_refresh_generation,
                    &media_refresh_pending,
                    MediaProbeCallbackKind::SessionMedia,
                );
                Ok(())
            },
        ))
        .ok();

    MediaSessionSubscriptions {
        session,
        timeline,
        playback,
        media,
        probe,
    }
}

fn start_media_listener(handle: AppHandle) -> MediaActorHandle {
    let (sender, _join) = spawn_media_listener(handle, None);
    MediaActorHandle::new(sender)
}

#[cfg(feature = "c2-media-probe")]
pub fn start_c2_media_probe_listener(handle: AppHandle) -> C2MediaProbeTask {
    let observer = MediaProbeObserver::default();
    let (stop, join) = spawn_media_listener(handle, Some(observer.clone()));
    C2MediaProbeTask {
        stop: MediaActorHandle::new(stop),
        join: Some(join),
        observer,
        started: std::time::Instant::now(),
    }
}

fn spawn_media_listener(
    handle: AppHandle,
    probe: Option<MediaProbeObserver>,
) -> (
    std::sync::mpsc::SyncSender<MediaActorMessage>,
    std::thread::JoinHandle<()>,
) {
    // 初始化事件总线
    if let Err(e) = EVENT_BUS.initialize(handle.clone()) {
        tracing::error!("[EventBus] 初始化失败: {}", e);
    }

    let (events_tx, events_rx) = std::sync::mpsc::sync_channel(state::MEDIA_ACTOR_QUEUE_CAPACITY);
    let stop = events_tx.clone();
    let metadata_dirty = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let worker_metadata_dirty = metadata_dirty.clone();
    let refresh_generation = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let worker_refresh_generation = refresh_generation.clone();
    let refresh_signal_pending = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let worker_refresh_signal_pending = refresh_signal_pending.clone();
    let join = std::thread::spawn(move || {
        // Keep this dedicated worker initialized; service calls own their
        // nested references and release interfaces before apartment teardown.
        let _apartment = match services::apartment::Apartment::enter() {
            Ok(apartment) => apartment,
            Err(error) => {
                tracing::error!("[Media] Thread initialization failed: {error}");
                return;
            }
        };

        use windows052::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

        let native_epoch = match windows052::core::GUID::new() {
            Ok(epoch) => format!(
                "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
                (epoch.to_u128() >> 96) as u32,
                (epoch.to_u128() >> 80) as u16,
                (epoch.to_u128() >> 64) as u16,
                (epoch.to_u128() >> 48) as u16,
                epoch.to_u128() & 0x0000_ffff_ffff_ffff,
            ),
            Err(error) => {
                tracing::error!("[Media] Cannot create native media epoch: {error}");
                return;
            }
        };

        let mut manager_subscriptions: Option<MediaManagerSubscriptions> = None;
        let mut session_subscriptions: Vec<MediaSessionSubscriptions> = Vec::new();
        let mut metadata_cache = services::media::MediaListenerMetadataCache::default();
        let mut metadata_generation_seen = 0u64;
        let mut next_manager_retry = std::time::Instant::now();
        let mut next_refresh = std::time::Instant::now();
        let mut last_sampled_refresh_generation = 0u64;
        let mut has_session = false;
        let mut is_playing = false;
        let mut consecutive_manager_query_failures = 0u8;
        let mut emit_baseline = MediaEmitBaseline::default();
        let mut snapshot_cache = state::NativeSnapshotCache::new(native_epoch);
        let mut netease_seek_anchor: Option<state::NeteaseSeekAnchor> = None;
        let mut netease_timeline_sample: Option<state::NeteaseTimelineSample> = None;
        let mut netease_progress_scheduler = state::NeteaseProgressScheduler::default();
        let mut netease_progress_mode = handle.state::<state::AppState>().netease_progress_mode();
        let netease_progress_inflight = state::NeteaseProgressInFlight::default();
        let mut netease_progress_pending = false;
        let mut netease_seek_generation = 0u64;
        let mut pending_snapshots: Vec<
            std::sync::mpsc::SyncSender<Result<models::MediaState, String>>,
        > = Vec::new();
        let mut missing_session_polls = 0u8;
        let mut next_refresh_reason = MediaRefreshReason::Fallback;
        #[cfg(feature = "c2-media-probe")]
        let mut probe_ready = false;
        let mut probe_has_snapshot_state = false;
        let mut probe_last_playback = false;
        let mut probe_last_selected_session = None;
        // SMTC events trigger prompt snapshots; a bounded poll remains as a
        // fallback for providers that omit events and for manager recovery.
        loop {
            let desired_progress_mode = handle.state::<state::AppState>().netease_progress_mode();
            if state::reconcile_netease_progress_mode(
                &mut netease_progress_mode,
                desired_progress_mode,
                &mut netease_seek_generation,
                &mut netease_progress_scheduler,
            ) {
                // Re-entering real mode schedules an immediate read; an old
                // worker retains the single-flight permit until it exits.
                next_refresh = std::time::Instant::now();
                next_refresh_reason = MediaRefreshReason::Event;
            }
            let now = std::time::Instant::now();
            if manager_subscriptions.is_none() && now >= next_manager_retry {
                next_manager_retry = now + std::time::Duration::from_secs(5);
                match GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                    .and_then(|operation| operation.get())
                {
                    Ok(manager) => {
                        metadata_cache.clear();
                        metadata_generation_seen =
                            worker_metadata_dirty.load(std::sync::atomic::Ordering::Acquire);
                        if let Some(probe) = &probe {
                            probe.update(|counts| counts.manager_acquired += 1);
                        }
                        let current_events = events_tx.clone();
                        let current_probe = probe.clone();
                        let current_dirty = worker_metadata_dirty.clone();
                        let current_refresh_generation = worker_refresh_generation.clone();
                        let current_refresh_pending = worker_refresh_signal_pending.clone();
                        let current_session = manager
                            .CurrentSessionChanged(&windows052::Foundation::TypedEventHandler::<
                                windows052::Media::Control::GlobalSystemMediaTransportControlsSessionManager,
                                windows052::Media::Control::CurrentSessionChangedEventArgs,
                            >::new(
                                move |_, _| -> windows052::core::Result<()> {
                                    send_media_refresh(
                                        &current_events,
                                        current_probe.as_ref(),
                                        Some(&current_dirty),
                                        &current_refresh_generation,
                                        &current_refresh_pending,
                                        MediaProbeCallbackKind::ManagerCurrentSession,
                                    );
                                    Ok(())
                                },
                            ))
                            .ok();
                        let sessions_events = events_tx.clone();
                        let sessions_probe = probe.clone();
                        let sessions_dirty = worker_metadata_dirty.clone();
                        let sessions_refresh_generation = worker_refresh_generation.clone();
                        let sessions_refresh_pending = worker_refresh_signal_pending.clone();
                        let sessions = manager
                            .SessionsChanged(&windows052::Foundation::TypedEventHandler::<
                                windows052::Media::Control::GlobalSystemMediaTransportControlsSessionManager,
                                windows052::Media::Control::SessionsChangedEventArgs,
                            >::new(
                                move |_, _| -> windows052::core::Result<()> {
                                    send_media_refresh(
                                        &sessions_events,
                                        sessions_probe.as_ref(),
                                        Some(&sessions_dirty),
                                        &sessions_refresh_generation,
                                        &sessions_refresh_pending,
                                        MediaProbeCallbackKind::ManagerSessions,
                                    );
                                    Ok(())
                                },
                            ))
                            .ok();
                        manager_subscriptions = Some(MediaManagerSubscriptions {
                            manager,
                            current_session,
                            sessions,
                            probe: probe.clone(),
                        });
                        if let Some(probe) = &probe {
                            let added = u64::from(
                                manager_subscriptions
                                    .as_ref()
                                    .unwrap()
                                    .current_session
                                    .is_some(),
                            ) + u64::from(
                                manager_subscriptions.as_ref().unwrap().sessions.is_some(),
                            );
                            probe.update(|counts| counts.subscriptions_added += added);
                        }
                        next_refresh = now;
                        next_refresh_reason = MediaRefreshReason::ManagerReady;
                    }
                    Err(error) => {
                        if let Some(probe) = &probe {
                            probe.update(|counts| counts.manager_failed += 1);
                        }
                        tracing::debug!("[Media] SMTC manager unavailable: {error:?}")
                    }
                }
            }

            if let Some(subscriptions) = manager_subscriptions.as_ref() {
                if let Ok(sessions) = subscriptions.manager.GetSessions() {
                    let mut current_sessions = Vec::new();
                    for index in 0..sessions.Size().unwrap_or(0) {
                        if let Ok(session) = sessions.GetAt(index) {
                            if !current_sessions.iter().any(|current| {
                                services::media::media_sessions_have_same_identity(
                                    current, &session,
                                )
                            }) {
                                current_sessions.push(session.clone());
                                if !session_subscriptions.iter().any(|item| {
                                    services::media::media_sessions_have_same_identity(
                                        &item.session,
                                        &session,
                                    )
                                }) {
                                    session_subscriptions.push(subscribe_media_session(
                                        session,
                                        events_tx.clone(),
                                        probe.clone(),
                                        worker_metadata_dirty.clone(),
                                        worker_refresh_generation.clone(),
                                        worker_refresh_signal_pending.clone(),
                                    ));
                                    if let Some(probe) = &probe {
                                        let added =
                                            session_subscriptions.last().map_or(0, |item| {
                                                u64::from(item.timeline.is_some())
                                                    + u64::from(item.playback.is_some())
                                                    + u64::from(item.media.is_some())
                                            });
                                        probe.update(|counts| counts.subscriptions_added += added);
                                    }
                                }
                            }
                        }
                    }
                    let mut active = Vec::with_capacity(session_subscriptions.len());
                    for subscription in session_subscriptions.drain(..) {
                        if current_sessions.iter().any(|current| {
                            services::media::media_sessions_have_same_identity(
                                &subscription.session,
                                current,
                            )
                        }) {
                            active.push(subscription);
                        } else {
                            subscription.remove();
                        }
                    }
                    session_subscriptions = active;

                    #[cfg(feature = "c2-media-probe")]
                    if !probe_ready {
                        if let Some(probe) = &probe {
                            let manager_tokens = manager_subscriptions.as_ref().map_or(0, |item| {
                                u64::from(item.current_session.is_some())
                                    + u64::from(item.sessions.is_some())
                            });
                            let session_tokens = session_subscriptions
                                .iter()
                                .map(|item| {
                                    u64::from(item.timeline.is_some())
                                        + u64::from(item.playback.is_some())
                                        + u64::from(item.media.is_some())
                                })
                                .sum();
                            probe.mark_ready(manager_tokens, session_tokens);
                            probe_ready = true;
                        }
                    }
                }
            }

            if std::time::Instant::now() >= next_refresh {
                let refresh_reason = next_refresh_reason;
                if let Some(probe) = &probe {
                    probe.update(|counts| match refresh_reason {
                        MediaRefreshReason::ManagerReady => counts.manager_ready_refreshes += 1,
                        MediaRefreshReason::Event => counts.event_refreshes += 1,
                        MediaRefreshReason::Fallback => counts.fallback_refreshes += 1,
                    });
                }
                let snapshot_started = std::time::Instant::now();
                let refresh_generation_at_sample =
                    worker_refresh_generation.load(std::sync::atomic::Ordering::Acquire);
                let snapshot = manager_subscriptions.as_ref().map(|subscriptions| {
                    let generation =
                        worker_metadata_dirty.load(std::sync::atomic::Ordering::Acquire);
                    let snapshot = services::media::get_media_info_from_manager_for_listener(
                        &handle,
                        &subscriptions.manager,
                        &mut metadata_cache,
                        generation != metadata_generation_seen,
                    );
                    if let Some(probe) = &probe {
                        probe.update(|counts| {
                            counts.metadata_reads += u64::from(snapshot.metadata_read_attempted);
                            counts.metadata_reuses += u64::from(snapshot.metadata_reused);
                        });
                    }
                    if snapshot.metadata_updated {
                        metadata_generation_seen = generation;
                    }
                    snapshot
                });
                let query_available = snapshot.is_some();
                #[cfg(feature = "c2-media-probe")]
                let mut snapshot_status = if query_available {
                    "error"
                } else {
                    "unavailable"
                };
                let mut poll_interval = if query_available {
                    media_poll_interval(has_session, is_playing, emit_baseline.empty_state_emitted)
                } else {
                    std::time::Duration::from_secs(5)
                };
                if let Some(services::media::MediaListenerSnapshot {
                    result: Ok(mut info),
                    ..
                }) = snapshot
                {
                    if info.source.is_empty() {
                        // A single missed SMTC selection is provisional. Keep the
                        // last real CDP timeline until removal is actually emitted.
                    } else {
                        if info.source == "netease"
                            && snapshot_cache.current().is_none_or(|current| {
                                current.source != "netease"
                                    || current.source_display != info.source_display
                            })
                        {
                            // Also restore when the player starts after Isle. This is
                            // session-driven, not another periodic process scan.
                            commands::start_netease_auto_restore(handle.clone());
                        }
                        if info.source == "netease"
                            && snapshot_cache.current().is_none_or(|current| {
                                current.source != info.source
                                    || current.title != info.title
                                    || current.artist != info.artist
                                    || current.is_playing != info.is_playing
                            })
                        {
                            netease_progress_scheduler.reset();
                            netease_seek_generation = netease_seek_generation.saturating_add(1);
                        }
                        if let Some(current) = snapshot_cache.current().filter(|current| {
                            current.source == info.source
                                && current.title == info.title
                                && current.artist == info.artist
                        }) {
                            info.native_epoch = current.native_epoch;
                            info.native_track_generation = current.native_track_generation;
                            info.netease_progress_available = current.netease_progress_available;
                        }
                        if info.source == "netease" {
                            info.progress_known = false;
                        }
                        let retain_anchor = netease_seek_anchor.as_mut().is_none_or(|anchor| {
                            anchor.apply(
                                &mut info,
                                std::time::Instant::now(),
                                netease_progress_mode == models::NeteaseProgressMode::Estimate,
                            )
                        });
                        if !retain_anchor {
                            netease_seek_anchor = None;
                        }
                        if netease_timeline_sample.as_mut().is_some_and(|sample| {
                            sample.apply(&mut info, std::time::Instant::now())
                        }) {
                            info.progress_known = true;
                            // Keep the last true CDP sample over SMTC's empty timeline.
                            // The frontend interpolates from its timestamp; the actor does
                            // not repeatedly add elapsed time to the sampled position.
                        } else {
                            netease_timeline_sample = None;
                            if info.source == "netease"
                                && netease_progress_mode == models::NeteaseProgressMode::Estimate
                                && netease_seek_anchor.is_none()
                            {
                                info.position_ms = 0;
                                info.progress_known = false;
                            }
                        }
                    }
                    last_sampled_refresh_generation = refresh_generation_at_sample;
                    #[cfg(feature = "c2-media-probe")]
                    {
                        snapshot_status = if info.source.is_empty() {
                            "empty"
                        } else {
                            "ok"
                        };
                    }
                    if let Some(probe) = &probe {
                        use std::hash::{Hash, Hasher};
                        let selected_session = if info.source.is_empty() {
                            None
                        } else {
                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            info.source.hash(&mut hasher);
                            Some(hasher.finish())
                        };
                        probe.update(|counts| {
                            if info.source.is_empty() {
                                counts.snapshots_empty += 1;
                            } else {
                                counts.snapshots_nonempty += 1;
                            }
                            if probe_has_snapshot_state {
                                counts.playback_state_changes +=
                                    u64::from(probe_last_playback != info.is_playing);
                                counts.selected_session_changes +=
                                    u64::from(probe_last_selected_session != selected_session);
                            }
                        });
                        probe_has_snapshot_state = true;
                        probe_last_playback = info.is_playing;
                        probe_last_selected_session = selected_session;
                    }
                    consecutive_manager_query_failures = 0;
                    let mut artwork_omitted = false;
                    let mut emit_empty_state = false;
                    let mut artwork_candidate = None;
                    if info.source.is_empty() {
                        // GSMTC may briefly return no selected session while a
                        // player refreshes its metadata. Confirm the removal over
                        // two polls so one missed snapshot cannot reset progress
                        // or stop the spectrum animation.
                        missing_session_polls = missing_session_polls.saturating_add(1);
                        has_session = false;
                        is_playing = false;
                        // The first absent snapshot is provisional even if the
                        // previous session was paused and had a slower cadence.
                        poll_interval =
                            media_poll_interval(false, false, emit_baseline.empty_state_emitted);
                        if emit_baseline.should_emit_empty_state(missing_session_polls) {
                            emit_empty_state = true;
                            netease_seek_anchor = None;
                            netease_timeline_sample = None;
                            // Keep retries at the existing empty-state cadence
                            // until an actual broadcast acknowledges removal.
                            poll_interval = media_poll_interval(false, false, true);
                        }
                    } else {
                        missing_session_polls = 0;
                        emit_baseline.empty_state_emitted = false;
                        has_session = true;
                        is_playing = info.is_playing;
                        poll_interval = media_poll_interval(true, info.is_playing, false);
                        let artwork_track =
                            format!("{}|{}|{}", info.source, info.title, info.artist);
                        if emit_baseline.artwork_unchanged(&artwork_track, &info.album_art) {
                            // Artwork is often hundreds of KB. Sending the same Base64
                            // payload to every WebView once per second caused release
                            // builds to stutter even though only progress had changed.
                            artwork_omitted = true;
                        } else {
                            artwork_candidate = Some((artwork_track, info.album_art.clone()));
                        }
                    }
                    let publish_snapshot = !info.source.is_empty() || emit_empty_state;
                    let published_snapshot = if publish_snapshot {
                        match snapshot_cache.publish(info.clone()) {
                            Ok(snapshot) => Some(snapshot),
                            Err(error) => {
                                tracing::error!("[Media] {error}; stopping the media actor to avoid unsafe revisions");
                                for reply in pending_snapshots.drain(..) {
                                    let _ = reply.send(Err(error.clone()));
                                }
                                break;
                            }
                        }
                    } else {
                        None
                    };
                    if let Some(snapshot) = published_snapshot.as_ref() {
                        // Keep the cache and command replies complete; only the
                        // event copy may omit an acknowledged same-track cover.
                        info = snapshot.clone();
                    }
                    if publish_snapshot {
                        let mut event_info = info.clone();
                        if artwork_omitted {
                            event_info.album_art.clear();
                        }
                        match media_event_payload(event_info, artwork_omitted) {
                            Ok(payload) => {
                                #[cfg(feature = "c2-media-probe")]
                                if emit_empty_state {
                                    if let Some(probe) = &probe {
                                        probe.update(|counts| counts.empty_emit_calls += 1);
                                    }
                                }
                                let result = if emit_baseline.transport_changed(&info) {
                                    event_bus::emit_media_immediate_update_outcome(payload)
                                } else {
                                    event_bus::emit_media_update_outcome(payload)
                                };
                                emit_baseline
                                    .acknowledge_transport(&info, result.as_ref().ok().copied());
                                emit_baseline.acknowledge(
                                    result.as_ref().ok().copied(),
                                    emit_empty_state,
                                    artwork_candidate,
                                );
                                #[cfg(feature = "c2-media-probe")]
                                if emit_empty_state {
                                    if let Some(probe) = &probe {
                                        probe.update(|counts| match result {
                                            Ok(event_bus::EmitOutcome::Emitted) => {
                                                counts.empty_emit_success += 1;
                                            }
                                            Ok(event_bus::EmitOutcome::Throttled) => {
                                                counts.empty_emit_throttled += 1;
                                            }
                                            Ok(event_bus::EmitOutcome::Disabled) => {
                                                counts.empty_emit_disabled += 1;
                                            }
                                            Err(_) => counts.empty_emit_error += 1,
                                        });
                                    }
                                }
                                #[cfg(not(feature = "c2-media-probe"))]
                                let _ = result;
                            }
                            #[cfg(feature = "c2-media-probe")]
                            Err(_) if emit_empty_state => {
                                if let Some(probe) = &probe {
                                    probe.update(|counts| counts.empty_payload_error += 1);
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    if let Some(snapshot) = published_snapshot {
                        state::complete_snapshot_requests(&snapshot, &mut pending_snapshots);
                    }
                } else if query_available {
                    #[cfg(feature = "c2-media-probe")]
                    {
                        snapshot_status = "error";
                    }
                    if let Some(probe) = &probe {
                        probe.update(|counts| counts.snapshot_errors += 1);
                    }
                    // A failed SMTC query is not proof that its session was
                    // removed, so it must break the consecutive-miss sequence.
                    missing_session_polls = 0;
                    consecutive_manager_query_failures =
                        consecutive_manager_query_failures.saturating_add(1);
                    for reply in pending_snapshots.drain(..) {
                        let _ = reply.send(Err("native media snapshot failed".to_string()));
                    }
                    if consecutive_manager_query_failures >= 2 {
                        // A manager can remain alive after its WinRT service
                        // connection has gone stale. Tear down all event tokens
                        // and reacquire it on the next loop; the normal bounded
                        // retry below handles RequestAsync failures.
                        for subscription in session_subscriptions.drain(..) {
                            subscription.remove();
                        }
                        if let Some(subscriptions) = manager_subscriptions.take() {
                            subscriptions.remove();
                        }
                        metadata_cache.clear();
                        metadata_generation_seen =
                            worker_metadata_dirty.load(std::sync::atomic::Ordering::Acquire);
                        consecutive_manager_query_failures = 0;
                        next_manager_retry = std::time::Instant::now();
                    }
                }
                if !query_available {
                    for reply in pending_snapshots.drain(..) {
                        let _ = reply.send(Err("media actor is unavailable".to_string()));
                    }
                }
                if query_available {
                    if let Some(probe) = &probe {
                        let micros = snapshot_started.elapsed().as_micros();
                        probe.update(|counts| {
                            counts.snapshot_total_micros =
                                counts.snapshot_total_micros.saturating_add(micros);
                            counts.snapshot_max_micros = counts.snapshot_max_micros.max(micros);
                            counts.snapshot_attempts += 1;
                        });
                    }
                }
                next_refresh = std::time::Instant::now() + poll_interval;
                next_refresh_reason = MediaRefreshReason::Fallback;
                #[cfg(feature = "c2-media-probe")]
                if let Some(probe) = &probe {
                    probe.record_poll(refresh_reason, snapshot_status, poll_interval);
                }
            }

            let now = std::time::Instant::now();
            if let Some(snapshot) = snapshot_cache.current() {
                let now = std::time::Instant::now();
                let netease_selected = has_session && snapshot.source == "netease";
                let progress_mode_enabled =
                    netease_progress_scheduler.mode_enabled(netease_progress_mode);
                let progress_due = netease_progress_scheduler.due(
                    now,
                    netease_selected && progress_mode_enabled,
                    snapshot.is_playing,
                );
                #[cfg(feature = "c2-media-probe")]
                if let Some(probe) = &probe {
                    probe.update(|counts| {
                        if netease_selected {
                            counts.netease_snapshots += 1;
                            counts.progress_real_mode_snapshots += u64::from(progress_mode_enabled);
                            counts.progress_due +=
                                u64::from(progress_due && !netease_progress_pending);
                        }
                    });
                }
                if !netease_progress_pending && progress_due {
                    netease_progress_pending = spawn_netease_progress_read(
                        events_tx.clone(),
                        netease_progress_inflight.clone(),
                        snapshot.native_epoch.clone(),
                        snapshot.title.clone(),
                        snapshot.artist.clone(),
                        snapshot.native_track_generation,
                        netease_seek_generation,
                        snapshot.native_revision,
                        probe.clone(),
                    );
                    if !netease_progress_pending {
                        netease_progress_scheduler.retry_soon(now);
                    }
                    #[cfg(feature = "c2-media-probe")]
                    if let Some(probe) = &probe {
                        probe.update(|counts| {
                            counts.progress_started += u64::from(netease_progress_pending);
                            counts.progress_inflight_busy += u64::from(!netease_progress_pending);
                        });
                    }
                }
            }

            let mut wait = next_refresh.saturating_duration_since(now);
            if !netease_progress_pending {
                if let Some(snapshot) = snapshot_cache.current() {
                    let selected = has_session
                        && manager_subscriptions.is_some()
                        && snapshot.source == "netease"
                        && netease_progress_scheduler.mode_enabled(netease_progress_mode);
                    if let Some(due) =
                        netease_progress_scheduler.next_due(now, selected, snapshot.is_playing)
                    {
                        wait = wait.min(due.saturating_duration_since(now));
                    }
                }
            }
            if manager_subscriptions.is_none() {
                wait = wait.min(next_manager_retry.saturating_duration_since(now));
            }
            match events_rx.recv_timeout(wait) {
                Ok(MediaActorMessage::Refresh) => {
                    worker_refresh_signal_pending
                        .store(false, std::sync::atomic::Ordering::Release);
                    if worker_refresh_generation.load(std::sync::atomic::Ordering::Acquire)
                        > last_sampled_refresh_generation
                    {
                        next_refresh = std::time::Instant::now();
                        next_refresh_reason = MediaRefreshReason::Event;
                    }
                }
                Ok(MediaActorMessage::Snapshot(reply)) => {
                    if manager_subscriptions.is_none() && snapshot_cache.current().is_none() {
                        let _ =
                            reply
                                .send(Err("media actor is not ready: SMTC manager unavailable"
                                    .to_string()));
                    } else if pending_snapshots.len() >= state::MEDIA_ACTOR_QUEUE_CAPACITY {
                        let _ = reply.send(Err(
                            "media actor pending request capacity reached".to_string()
                        ));
                    } else {
                        let should_refresh = state::queue_snapshot_request(
                            &snapshot_cache,
                            reply,
                            &mut pending_snapshots,
                        );
                        if should_refresh {
                            next_refresh = std::time::Instant::now();
                            next_refresh_reason = MediaRefreshReason::Event;
                        }
                    }
                }
                Ok(MediaActorMessage::TransportChanged) => {
                    netease_progress_scheduler.reset();
                    netease_seek_generation = netease_seek_generation.saturating_add(1);
                    next_refresh = std::time::Instant::now();
                    next_refresh_reason = MediaRefreshReason::Event;
                }
                Ok(MediaActorMessage::NeteaseProgressReadCompleted {
                    expected_epoch,
                    expected_title,
                    expected_artist,
                    expected_track_generation,
                    expected_seek_generation,
                    started_native_revision,
                    result,
                }) => {
                    netease_progress_pending = false;
                    let now = std::time::Instant::now();
                    let Some(current) = snapshot_cache.current() else {
                        continue;
                    };
                    let basis_is_current = netease_seek_generation == expected_seek_generation
                        && current.native_epoch == expected_epoch
                        && state::progress_read_basis_is_current(
                            &current,
                            &expected_epoch,
                            &expected_title,
                            &expected_artist,
                            expected_track_generation,
                            expected_seek_generation,
                            netease_seek_generation,
                            started_native_revision,
                        );
                    if !basis_is_current {
                        #[cfg(feature = "c2-media-probe")]
                        if let Some(probe) = &probe {
                            probe.update(|counts| counts.progress_basis_discard += 1);
                        }
                        continue;
                    }
                    match result {
                        Ok(progress) => {
                            match snapshot_cache.publish_netease_progress(
                                &expected_epoch,
                                &expected_title,
                                &expected_artist,
                                expected_track_generation,
                                &progress,
                            ) {
                                Ok(snapshot) => {
                                    #[cfg(feature = "c2-media-probe")]
                                    if let Some(probe) = &probe {
                                        probe.update(|counts| counts.progress_published += 1);
                                    }
                                    netease_progress_scheduler.success(now);
                                    netease_seek_anchor = None;
                                    netease_timeline_sample =
                                        Some(state::NeteaseTimelineSample::from_progress(
                                            &snapshot, &progress, now,
                                        ));
                                    let artwork_track = format!(
                                        "{}|{}|{}",
                                        snapshot.source, snapshot.title, snapshot.artist
                                    );
                                    let artwork_omitted = emit_baseline
                                        .artwork_unchanged(&artwork_track, &snapshot.album_art);
                                    let artwork_candidate = (!artwork_omitted)
                                        .then(|| (artwork_track, snapshot.album_art.clone()));
                                    let mut event_snapshot = snapshot;
                                    if artwork_omitted {
                                        event_snapshot.album_art.clear();
                                    }
                                    match media_event_payload(event_snapshot, artwork_omitted) {
                                        Ok(payload) => {
                                            let result =
                                                event_bus::emit_media_immediate_update_outcome(
                                                    payload,
                                                );
                                            emit_baseline.acknowledge(
                                                result.as_ref().ok().copied(),
                                                false,
                                                artwork_candidate,
                                            );
                                            if let Err(error) = result {
                                                tracing::warn!("[Media] Could not broadcast NetEase progress: {error}");
                                            }
                                        }
                                        Err(error) => tracing::warn!(
                                            "[Media] Could not serialize NetEase progress: {error}"
                                        ),
                                    }
                                }
                                Err(error) => {
                                    #[cfg(feature = "c2-media-probe")]
                                    if let Some(probe) = &probe {
                                        probe
                                            .update(|counts| counts.progress_publish_rejected += 1);
                                    }
                                    tracing::debug!("[Media] Discarding invalid NetEase progress sample: {error}");
                                    netease_progress_scheduler.failure(now);
                                }
                            }
                        }
                        Err(error) => {
                            tracing::debug!("[Media] NetEase progress read deferred: {error}");
                            netease_progress_scheduler.failure(now);
                            if netease_progress_mode == models::NeteaseProgressMode::Real {
                                let mut unavailable = current.clone();
                                unavailable.netease_progress_available = Some(false);
                                if let Ok(unavailable) = snapshot_cache.publish(unavailable) {
                                    let unchanged = emit_baseline.artwork_unchanged(
                                        &format!(
                                            "{}|{}|{}",
                                            unavailable.source,
                                            unavailable.title,
                                            unavailable.artist
                                        ),
                                        &unavailable.album_art,
                                    );
                                    let candidate = (!unchanged).then(|| {
                                        (
                                            format!(
                                                "{}|{}|{}",
                                                unavailable.source,
                                                unavailable.title,
                                                unavailable.artist
                                            ),
                                            unavailable.album_art.clone(),
                                        )
                                    });
                                    let mut event_snapshot = unavailable;
                                    if unchanged {
                                        event_snapshot.album_art.clear();
                                    }
                                    if let Ok(payload) =
                                        media_event_payload(event_snapshot, unchanged)
                                    {
                                        if let Ok(outcome) =
                                            event_bus::emit_media_update_outcome(payload)
                                        {
                                            emit_baseline.acknowledge(
                                                Some(outcome),
                                                false,
                                                candidate,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(MediaActorMessage::NeteaseProgressModeChanged) => {}
                Ok(MediaActorMessage::PlayerIdentityChanged) => {
                    netease_progress_scheduler.reset();
                    netease_seek_generation = netease_seek_generation.saturating_add(1);
                    next_refresh = std::time::Instant::now();
                    next_refresh_reason = MediaRefreshReason::Event;
                }
                Ok(MediaActorMessage::VerifiedNeteaseSeek {
                    expected_epoch,
                    expected_title,
                    expected_artist,
                    expected_track_generation,
                    position_ms,
                    duration_ms,
                    track_id,
                    play_id,
                    reply,
                }) => {
                    if pending_snapshots.len() >= state::MEDIA_ACTOR_QUEUE_CAPACITY {
                        let _ = reply.send(Err(
                            "media actor pending request capacity reached".to_string()
                        ));
                        continue;
                    }
                    match snapshot_cache.publish_verified_netease_seek(
                        &expected_epoch,
                        &expected_title,
                        &expected_artist,
                        expected_track_generation,
                        &track_id,
                        &play_id,
                        position_ms,
                        duration_ms,
                    ) {
                        Ok(snapshot) => {
                            netease_seek_generation =
                                netease_seek_generation.checked_add(1).unwrap_or(u64::MAX);
                            netease_progress_scheduler.success(std::time::Instant::now());
                            netease_seek_anchor =
                                Some(state::NeteaseSeekAnchor::from_snapshot(&snapshot));
                            let progress = services::netease_cdp::NeteaseProgress {
                                track_id,
                                play_id,
                                title: snapshot.title.clone(),
                                artist: snapshot.artist.clone(),
                                position_ms: snapshot.position_ms,
                                duration_ms: snapshot.duration_ms,
                            };
                            netease_timeline_sample =
                                Some(state::NeteaseTimelineSample::from_progress(
                                    &snapshot,
                                    &progress,
                                    std::time::Instant::now(),
                                ));
                            match media_event_payload(snapshot.clone(), false) {
                                Ok(payload) => {
                                    if let Err(error) =
                                        event_bus::emit_media_immediate_update_outcome(payload)
                                    {
                                        tracing::warn!("[Media] Could not broadcast verified NetEase seek: {error}");
                                    }
                                }
                                Err(error) => tracing::warn!(
                                    "[Media] Could not serialize verified NetEase seek: {error}"
                                ),
                            }
                            let _ = reply.send(Ok(snapshot));
                        }
                        Err(error) => {
                            let _ = reply.send(Err(error));
                        }
                    }
                }
                #[cfg(feature = "c2-media-probe")]
                Ok(MediaActorMessage::Stop) => {
                    for reply in pending_snapshots.drain(..) {
                        let _ = reply.send(Err("media actor stopped".to_string()));
                    }
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        for subscription in session_subscriptions {
            subscription.remove();
        }
        if let Some(subscriptions) = manager_subscriptions {
            subscriptions.remove();
        }
    });
    (stop, join)
}

fn spawn_netease_progress_read(
    events: std::sync::mpsc::SyncSender<MediaActorMessage>,
    inflight: state::NeteaseProgressInFlight,
    expected_epoch: String,
    expected_title: String,
    expected_artist: String,
    expected_track_generation: u64,
    expected_seek_generation: u64,
    started_native_revision: u64,
    probe: Option<MediaProbeObserver>,
) -> bool {
    let Some(permit) = inflight.begin() else {
        return false;
    };
    let _worker = tauri::async_runtime::spawn_blocking(move || {
        let _release = permit;
        let result = services::netease_cdp::LocalCdpClient::default()
            .read_progress(&expected_title, &expected_artist);
        #[cfg(feature = "c2-media-probe")]
        if let Some(probe) = &probe {
            let kind = result
                .as_ref()
                .map_or_else(|error| error.diagnostic_kind(), |_| "ok");
            probe.record_progress_read_result(kind);
        }
        let result = result.map_err(|error| error.to_string());
        let _ = events.send(MediaActorMessage::NeteaseProgressReadCompleted {
            expected_epoch,
            expected_title,
            expected_artist,
            expected_track_generation,
            expected_seek_generation,
            started_native_revision,
            result,
        });
    });
    #[cfg(not(feature = "c2-media-probe"))]
    let _ = probe;
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CaptureMonitorPreferences {
    screenshot: bool,
    recording: bool,
    fullscreen: bool,
}

impl CaptureMonitorPreferences {
    fn from_settings(settings: &models::AppPreferences) -> Self {
        Self {
            screenshot: settings.capture_hide_on_screenshot,
            recording: settings.capture_hide_on_recording,
            fullscreen: settings.capture_hide_on_fullscreen,
        }
    }

    fn any_native_detection(self) -> bool {
        self.screenshot || self.recording || self.fullscreen
    }

    fn poll_interval(self) -> std::time::Duration {
        if self.screenshot {
            std::time::Duration::from_millis(50)
        } else if self.recording || self.fullscreen {
            std::time::Duration::from_millis(500)
        } else {
            std::time::Duration::from_secs(1)
        }
    }
}

fn capture_monitor_preferences(handle: &AppHandle) -> CaptureMonitorPreferences {
    handle
        .state::<state::AppState>()
        .settings
        .lock()
        .map(|settings| CaptureMonitorPreferences::from_settings(&settings))
        // A poisoned settings lock must not silently disable a requested
        // privacy feature. Keep all native detectors active in that case.
        .unwrap_or(CaptureMonitorPreferences {
            screenshot: true,
            recording: true,
            fullscreen: true,
        })
}

/// 启动统一捕获状态监控器
///
/// 在后台线程中监控截图快捷键与全屏状态，并向前端发布统一快照。
///
/// ## 工作流程
///
/// 1. 每 50ms 检测截图快捷键
/// 2. 每 500ms 检测一次全屏与 Windows AppCapture 视频捕获状态
/// 3. 状态变化时立即发布，并周期重发以同步晚加载的窗口
fn start_capture_monitor(handle: AppHandle) {
    use std::time::{Duration, Instant};
    use windows052::Media::Capture::AppCapture;
    use windows052::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_LWIN, VK_RWIN, VK_S, VK_SHIFT, VK_SNAPSHOT,
    };

    std::thread::spawn(move || {
        let mut preferences = capture_monitor_preferences(&handle);
        let mut settings_revision = handle
            .state::<state::AppState>()
            .settings_revision
            .load(std::sync::atomic::Ordering::Acquire);
        let mut previous = (false, false, false);
        let mut screenshot_until: Option<Instant> = None;
        let mut previous_shortcut_down = false;
        let mut is_fullscreen = false;
        let mut is_recording = false;
        let mut next_fullscreen_check = Instant::now();
        let mut next_capture_check = Instant::now();
        let mut next_capture_capability_probe = Instant::now();
        let mut last_emit = Instant::now() - Duration::from_secs(2);

        let capture_apartment = services::apartment::Apartment::enter();
        let mut video_capture = None;

        loop {
            let now = Instant::now();
            let revision = handle
                .state::<state::AppState>()
                .settings_revision
                .load(std::sync::atomic::Ordering::Acquire);
            if revision != settings_revision {
                let next_preferences = capture_monitor_preferences(&handle);
                if !preferences.recording && next_preferences.recording {
                    // A user re-enable is an explicit reason to retry now,
                    // even if an earlier capability failure left a backoff.
                    next_capture_check = now;
                    next_capture_capability_probe = now;
                }
                preferences = next_preferences;
                settings_revision = revision;
            }

            if preferences.fullscreen && now >= next_fullscreen_check {
                is_fullscreen = detect_fullscreen_app(&handle);
                next_fullscreen_check = now + Duration::from_millis(500);
            } else if !preferences.fullscreen {
                is_fullscreen = false;
            }

            if preferences.recording && now >= next_capture_check {
                if video_capture.is_none() && now >= next_capture_capability_probe {
                    video_capture = capture_apartment
                        .as_ref()
                        .ok()
                        .and_then(|_| AppCapture::GetForCurrentView().ok());
                    if video_capture.is_none() {
                        next_capture_capability_probe = now + Duration::from_secs(30);
                        tracing::debug!("[Capture Mode] Windows AppCapture video status is unavailable; retrying later");
                    }
                }
                match video_capture
                    .as_ref()
                    .map(|capture| capture.IsCapturingVideo())
                {
                    Some(Ok(capturing)) => is_recording = capturing,
                    Some(Err(error)) => {
                        // A stale AppCapture object must not be polled every
                        // 500 ms forever (for example after a suspend/resume).
                        video_capture = None;
                        is_recording = false;
                        next_capture_capability_probe = now + Duration::from_secs(30);
                        tracing::debug!("[Capture Mode] Windows AppCapture video status failed; retrying later: {error:?}");
                    }
                    None => is_recording = false,
                }
                next_capture_check = now + Duration::from_millis(500);
            } else if !preferences.recording {
                is_recording = false;
                // Releasing this on the actor thread keeps disabled recording
                // detection free of a retained WinRT capture interface.
                video_capture = None;
            }

            let shortcut_down = if preferences.screenshot {
                unsafe {
                    let print_screen = GetAsyncKeyState(VK_SNAPSHOT.0 as i32) < 0;
                    let win = GetAsyncKeyState(VK_LWIN.0 as i32) < 0
                        || GetAsyncKeyState(VK_RWIN.0 as i32) < 0;
                    let shift = GetAsyncKeyState(VK_SHIFT.0 as i32) < 0;
                    let s = GetAsyncKeyState(VK_S.0 as i32) < 0;
                    print_screen || (win && shift && s)
                }
            } else {
                screenshot_until = None;
                false
            };
            if shortcut_down && !previous_shortcut_down {
                screenshot_until = Some(now + Duration::from_millis(1500));
            }
            previous_shortcut_down = shortcut_down;

            let screenshot = screenshot_until.is_some_and(|until| now < until);
            if !screenshot {
                screenshot_until = None;
            }

            let current = (screenshot, is_recording, is_fullscreen);
            if current != previous
                || (preferences.any_native_detection()
                    && last_emit.elapsed() >= Duration::from_secs(2))
            {
                let payload = serde_json::json!({
                    "screenshot": screenshot,
                    "recording": is_recording,
                    "fullscreen": is_fullscreen,
                    "screenShare": false,
                });
                if let Err(error) = event_bus::emit_capture_mode_changed(payload) {
                    tracing::error!("[Capture Mode] 发送状态失败: {}", error);
                }
                previous = current;
                last_emit = Instant::now();
            }

            std::thread::sleep(preferences.poll_interval());
        }
    });
}

fn detect_fullscreen_app(_handle: &AppHandle) -> bool {
    use windows052::Win32::Foundation::RECT;
    use windows052::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows052::Win32::UI::WindowsAndMessaging::{
        GetDesktopWindow, GetForegroundWindow, GetShellWindow, GetWindowLongPtrW, GetWindowRect,
        GetWindowThreadProcessId, IsWindowVisible, GWL_STYLE, WS_CAPTION,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0
            || hwnd == GetShellWindow()
            || hwnd == GetDesktopWindow()
            || !IsWindowVisible(hwnd).as_bool()
        {
            return false;
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == std::process::id() {
            return false;
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
        let win_w = rect.right - rect.left;
        let win_h = rect.bottom - rect.top;
        if win_w <= 0 || win_h <= 0 {
            return false;
        }

        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if hmon.0 == 0 {
            return false;
        }
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..MONITORINFO::default()
        };
        if !GetMonitorInfoW(hmon, &mut mi).as_bool() {
            return false;
        }
        let mon_w = mi.rcMonitor.right - mi.rcMonitor.left;
        let mon_h = mi.rcMonitor.bottom - mi.rcMonitor.top;
        let covers_screen = win_w >= mon_w - 10 && win_h >= mon_h - 10;
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        covers_screen && (style & WS_CAPTION.0 as isize) == 0
    }
}

// ============================================================================
// 窗口效果
// ============================================================================

/// 设置窗口亚克力效果
///
/// Windows 专属功能，为窗口添加毛玻璃效果。
///
/// ## 参数
///
/// - `window`: Tauri 窗口实例
/// - `enable`: true 启用，false 禁用
///
/// ## 平台兼容
///
/// - Windows: 使用 `window_vibrancy` 实现
/// - 其他平台: 静默忽略
#[tauri::command]
fn set_window_vibrancy(window: tauri::Window, enable: bool) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        if enable {
            apply_acrylic(&window, Some((0, 0, 0, 0)))
                .map_err(|e| AppError::Window(format!("Failed to apply acrylic: {}", e)))?;
        } else {
            clear_acrylic(&window)
                .map_err(|e| AppError::Window(format!("Failed to clear acrylic: {}", e)))?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        let _ = enable;
    }

    Ok(())
}

// ============================================================================
// 应用入口
// ============================================================================

/// 应用主入口
///
/// 初始化并运行 Tauri 应用。
///
/// ## 初始化流程
///
/// 1. 注册插件（全局快捷键、文件系统、对话框）
/// 2. 注册状态管理
/// 3. 注册所有 IPC 命令
/// 4. 执行 setup 回调：
///    - 初始化缓存系统
///    - 加载保存的设置
///    - 定位窗口到指定显示器
///    - 启动媒体监听器
///    - 创建频谱捕获状态（按可见消费者按需启停）
///    - 创建系统托盘
///
/// ## 托盘菜单
///
/// - 显示主窗口
/// - 打开设置
/// - 退出应用
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        // 注册插件
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        // 注册状态
        .manage(state::AppState::default())
        .manage(SpectrumState(Mutex::new((
            audio::SpectrumCapture::new(),
            std::collections::HashSet::new(),
        ))))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::ScaleFactorChanged { scale_factor, .. } = event {
                commands::update_cursor_scale_factor_from_event(window.label(), *scale_factor);
            }
            if matches!(event, tauri::WindowEvent::Destroyed) {
                commands::main_work_area_listener_window_destroyed(window.label());
                commands::cursor_monitor_window_destroyed(window.label());
                services::artwork_jobs::registry().close_window(window.label());
                release_spectrum(&window.state::<SpectrumState>(), window.label());
            }
        })
        .on_page_load(|webview, payload| {
            if webview.label() == "main"
                && payload.event() == tauri::webview::PageLoadEvent::Finished
            {
                commands::schedule_main_work_area_listener(webview.window());
            }
        })
        // 注册 IPC 命令
        .invoke_handler(tauri::generate_handler![
            // 设置相关命令
            commands::get_preferences,
            commands::save_preferences,
            commands::get_settings,
            commands::save_settings,
            commands::update_settings,
            commands::set_always_on_top,
            commands::set_floating_window_always_on_top,
            commands::set_window_opacity,
            commands::get_player_weights,
            commands::set_player_weight,
            commands::set_player_weights,
            commands::set_auto_start_cmd,
            commands::get_auto_start,
            // 窗口相关命令
            commands::save_floating_window_position,
            commands::get_floating_window_position,
            commands::show_main_window,
            commands::show_studio_window,
            commands::toggle_studio_window,
            commands::toggle_floating_window,
            commands::open_floating_window,
            commands::open_timer_window,
            commands::toggle_timer_window,
            commands::close_floating_window,
            commands::reset_floating_window,
            commands::sync_window_bounds,
            commands::animate_window_bounds,
            commands::set_island_interaction_region,
            commands::experimental_region_open,
            commands::experimental_region_control,
            commands::set_floating_window_resizable,
            commands::open_application,
            commands::check_fullscreen_app,
            commands::get_system_audio_state,
            commands::list_audio_output_devices,
            commands::set_system_volume,
            commands::set_default_audio_output,
            commands::get_available_monitors,
            commands::get_current_monitor_index,
            commands::set_current_monitor_index,
            // 媒体相关命令
            commands::get_media_info_cmd,
            commands::list_media_sessions,
            commands::get_idle_snapshot,
            commands::get_weather_snapshot,
            commands::search_weather_locations,
            commands::get_netease_song_info_cmd,
            commands::get_netease_playback_mode,
            commands::netease_song_like,
            commands::setup_netease_connection,
            commands::get_netease_mv_url_cmd,
            commands::resolve_hd_cover,
            commands::request_artwork,
            commands::cancel_artwork,
            commands::control_media,
            commands::cycle_netease_playback_mode,
            commands::seek_media,
            commands::toggle_shuffle,
            commands::cycle_repeat,
            commands::extract_dominant_color,
            commands::process_image,
            commands::pixelate_cover,
            // 缓存相关命令
            commands::clear_cache,
            commands::get_cache_stats,
            commands::get_cached_media,
            commands::download_and_cache,
            // 显示器相关命令
            commands::get_monitors,
            commands::move_to_monitor,
            // 窗口效果
            set_window_vibrancy,
            // 音频频谱
            start_spectrum,
            stop_spectrum,
            restart_spectrum,
        ])
        // Setup 回调
        .setup(|app| {
            // 初始化日志系统
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .init();

            // 初始化缓存系统
            if let Err(e) = services::cache::init_cache_system(app.handle()) {
                tracing::error!("[Cache] 初始化失败: {}", e);
            }

            // 加载保存的设置
            let saved_settings = read_settings_file(app.handle());
            let initial_settings = saved_settings.unwrap_or_default();

            // 启动时修复注册表中的路径，并清理旧版本留下的启动项。
            // 注册表只保存当前安装位置，升级后需要重新写入一次。
            if let Err(error) = set_auto_start(initial_settings.auto_start) {
                tracing::warn!("[Setup] 同步开机启动失败: {}", error);
            }

            let (show_label, studio_label, quit_label) =
                tray_labels(resolved_ui_language(&initial_settings.language));

            // 获取目标显示器索引
            let monitor_index = initial_settings.monitor_index;

            // 更新应用状态
            let state = app.state::<state::AppState>();
            if let Ok(mut state_settings) = state.settings.lock() {
                *state_settings = initial_settings.clone();
            }
            state.set_netease_progress_mode(initial_settings.netease_progress_mode);

            // 获取主窗口
            let window = match app.get_webview_window("main") {
                Some(w) => w,
                None => {
                    tracing::error!("[Setup] 未找到主窗口 'main'");
                    return Ok(());
                }
            };
            commands::schedule_main_work_area_listener(window.as_ref().window());

            // 定位窗口到指定显示器
            if let Ok(all_monitors) = window.available_monitors() {
                if monitor_index < all_monitors.len() as u32 {
                    let target_monitor = &all_monitors[monitor_index as usize];
                    let position = target_monitor.position();
                    let size = target_monitor.size();
                    let scale = window.scale_factor().unwrap_or(1.0);
                    let host_width = (300.0 * scale).round() as u32;
                    let host_height = (182.0 * scale).round() as u32;
                    let _ = window.set_size(tauri::PhysicalSize::new(host_width, host_height));
                    let x = position.x + (size.width as i32 - host_width as i32) / 2;
                    let y = position.y;
                    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
                }
            }

            if let Err(error) = commands::configure_main_window_visibility(&window) {
                tracing::warn!("[Setup] 配置灵动岛窗口可见性失败: {}", error);
            }

            if let Err(error) = commands::install_island_cursor_passthrough(&window) {
                tracing::warn!("[Setup] 灵动岛点击穿透初始化失败: {}", error);
            }
            commands::reset_content_protection(app.handle());

            // The transparent WebView must not be shown before the frontend
            // has mounted and applied its monitor-aware host bounds. The
            // frontend reveals this window after its first placement pass.
            // Tao may restore WS_EX_APPWINDOW while processing its queued show
            // event, so reapply the task-switcher styles on the UI thread after it.
            let task_switcher_window = window.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                let window_for_style = task_switcher_window.clone();
                if let Err(error) = task_switcher_window.run_on_main_thread(move || {
                    if let Err(error) =
                        commands::hide_main_window_from_task_switcher(&window_for_style)
                    {
                        tracing::warn!("[Setup] 刷新灵动岛任务切换器样式失败: {}", error);
                    }
                }) {
                    tracing::warn!("[Setup] 调度灵动岛任务切换器样式刷新失败: {}", error);
                }
            });

            // 启动后台服务
            let actor = start_media_listener(app.handle().clone());
            if let Err(error) = app.state::<state::AppState>().set_media_actor(actor) {
                tracing::error!("[Media] Failed to register media actor handle: {error}");
            }
            start_capture_monitor(app.handle().clone());
            commands::start_netease_auto_restore(app.handle().clone());

            // 创建托盘菜单
            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, SHOW_MENU_ID, show_label, true, None::<&str>)?,
                    &MenuItem::with_id(app, STUDIO_MENU_ID, studio_label, true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, QUIT_MENU_ID, quit_label, true, None::<&str>)?,
                ],
            )?;

            // 加载托盘图标
            let icon_bytes = include_bytes!("../icons/256x256.png");
            let icon = Image::from_bytes(icon_bytes).map_err(|e| e.to_string())?;

            // 创建托盘图标
            let _ = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    SHOW_MENU_ID => {
                        let _ = commands::show_main_window(app.clone());
                    }
                    STUDIO_MENU_ID => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = commands::show_studio_window(app).await;
                        });
                    }
                    QUIT_MENU_ID => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键点击显示/隐藏窗口
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if let Ok(v) = w.is_visible() {
                                if !v {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        });

    // 运行应用
    if let Err(e) = builder.run(tauri::generate_context!()) {
        eprintln!("应用运行失败: {}", e);
    }
}

#[cfg(test)]
mod media_delta_tests {
    #[test]
    fn transport_changes_bypass_progress_throttle_until_successfully_delivered() {
        use crate::event_bus::EmitOutcome;
        let mut baseline = super::MediaEmitBaseline::default();
        let mut info = crate::models::MediaState {
            source: "netease".into(),
            title: "Track".into(),
            is_playing: true,
            ..Default::default()
        };
        assert!(baseline.transport_changed(&info));
        baseline.acknowledge_transport(&info, Some(EmitOutcome::Emitted));
        info.position_ms = 1000;
        assert!(!baseline.transport_changed(&info));
        info.is_playing = false;
        assert!(baseline.transport_changed(&info));
        for outcome in [
            None,
            Some(EmitOutcome::Throttled),
            Some(EmitOutcome::Disabled),
        ] {
            baseline.acknowledge_transport(&info, outcome);
            assert!(baseline.transport_changed(&info));
        }
        baseline.acknowledge_transport(&info, Some(EmitOutcome::Emitted));
        assert!(!baseline.transport_changed(&info));
        info.title = "Next track".into();
        assert!(baseline.transport_changed(&info));
        baseline.acknowledge_transport(&info, Some(EmitOutcome::Emitted));
        info.shuffle_active = Some(true);
        assert!(baseline.transport_changed(&info));
        baseline.acknowledge_transport(&info, Some(EmitOutcome::Emitted));
        info.repeat_mode = Some("track".into());
        assert!(baseline.transport_changed(&info));
    }
    use crate::event_bus::EmitOutcome;

    #[test]
    fn unchanged_artwork_is_omitted_but_explicit_empty_snapshot_is_preserved() {
        let omitted =
            super::media_event_payload(super::models::MediaState::default(), true).unwrap();
        assert!(omitted.get("albumArt").is_none());
        let cleared =
            super::media_event_payload(super::models::MediaState::default(), false).unwrap();
        assert_eq!(cleared.get("albumArt").unwrap(), "");
    }

    #[test]
    fn native_revision_stamp_is_shared_by_full_snapshot_and_artwork_delta() {
        let mut cache = crate::state::NativeSnapshotCache::new("test-epoch".into());
        let full = cache
            .publish(crate::models::MediaState {
                album_art: "cover".into(),
                ..crate::models::MediaState::default()
            })
            .unwrap();
        let event = super::media_event_payload(full.clone(), true).unwrap();
        assert_eq!(event["nativeEpoch"], "test-epoch");
        assert_eq!(event["nativeRevision"], 1);
        assert!(event.get("albumArt").is_none());
        let command_snapshot = cache.current().unwrap();
        assert_eq!(command_snapshot.native_epoch, full.native_epoch);
        assert_eq!(command_snapshot.native_revision, full.native_revision);
        assert_eq!(command_snapshot.album_art, "cover");

        let empty = cache.publish(crate::models::MediaState::default()).unwrap();
        let empty_event = super::media_event_payload(empty, false).unwrap();
        assert_eq!(empty_event["nativeEpoch"], "test-epoch");
        assert_eq!(empty_event["nativeRevision"], 2);
        assert_eq!(empty_event["albumArt"], "");
        assert_eq!(cache.current().unwrap().native_revision, 2);
    }

    #[test]
    fn media_polling_is_fast_only_while_playing_or_confirming_removal() {
        assert_eq!(super::media_poll_interval(true, true, false).as_secs(), 1);
        assert_eq!(super::media_poll_interval(true, false, false).as_secs(), 2);
        assert_eq!(super::media_poll_interval(false, false, false).as_secs(), 1);
        assert_eq!(super::media_poll_interval(false, false, true).as_secs(), 5);
    }

    #[test]
    fn media_property_invalidation_survives_a_full_refresh_queue() {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::mpsc::sync_channel;

        let (events, _receiver) = sync_channel(1);
        events.send(super::MediaActorMessage::Refresh).unwrap();
        let dirty = AtomicU64::new(0);
        let refresh_generation = AtomicU64::new(0);
        let refresh_pending = std::sync::atomic::AtomicBool::new(false);
        let probe = super::MediaProbeObserver::default();

        super::send_media_refresh(
            &events,
            Some(&probe),
            Some(&dirty),
            &refresh_generation,
            &refresh_pending,
            super::MediaProbeCallbackKind::SessionMedia,
        );

        assert_eq!(dirty.load(Ordering::Acquire), 1);
        assert!(probe.counters.lock().unwrap().refresh_queue_full == 1);
        assert!(probe.counters.lock().unwrap().callback_session_media == 1);
    }

    #[test]
    fn repeated_media_callbacks_coalesce_to_one_refresh_marker() {
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        use std::sync::mpsc::sync_channel;

        let (events, receiver) = sync_channel(crate::state::MEDIA_ACTOR_QUEUE_CAPACITY);
        let generation = AtomicU64::new(0);
        let pending = AtomicBool::new(false);
        for _ in 0..3 {
            super::send_media_refresh(
                &events,
                None,
                None,
                &generation,
                &pending,
                super::MediaProbeCallbackKind::SessionTimeline,
            );
        }
        assert_eq!(generation.load(Ordering::Acquire), 3);
        assert!(pending.load(Ordering::Acquire));
        assert!(matches!(
            receiver.try_recv(),
            Ok(super::MediaActorMessage::Refresh)
        ));
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn artwork_replacement_and_clear_baselines_commit_only_after_real_emission() {
        let mut baseline = super::MediaEmitBaseline::default();
        let replacement = ("player|song|artist".to_string(), "cover-a".to_string());

        baseline.acknowledge(
            Some(EmitOutcome::Throttled),
            false,
            Some(replacement.clone()),
        );
        assert!(!baseline.artwork_unchanged(&replacement.0, &replacement.1));
        baseline.acknowledge(None, false, Some(replacement.clone())); // error/serialization failure
        assert!(!baseline.artwork_unchanged(&replacement.0, &replacement.1));
        baseline.acknowledge(Some(EmitOutcome::Emitted), false, Some(replacement.clone()));
        assert!(baseline.artwork_unchanged(&replacement.0, &replacement.1));
        assert!(baseline.artwork_unchanged(&replacement.0, &replacement.1)); // same-track snapshot may omit art

        let clear = (replacement.0.clone(), String::new());
        baseline.acknowledge(Some(EmitOutcome::Throttled), false, Some(clear.clone()));
        assert!(baseline.artwork_unchanged(&replacement.0, &replacement.1));
        baseline.acknowledge(Some(EmitOutcome::Emitted), false, Some(clear.clone()));
        assert!(baseline.artwork_unchanged(&clear.0, &clear.1));
    }

    #[test]
    fn confirmed_empty_state_retries_until_emitted_then_stops_and_clears_art() {
        let mut baseline = super::MediaEmitBaseline::default();
        baseline.artwork_track = "player|song|artist".into();
        baseline.artwork = "cover".into();

        baseline.acknowledge(Some(EmitOutcome::Throttled), true, None);
        assert!(!baseline.empty_state_emitted);
        assert!(baseline.should_emit_empty_state(2));
        assert!(baseline.artwork_unchanged("player|song|artist", "cover"));
        baseline.acknowledge(None, true, None); // error or payload serialization failure
        assert!(!baseline.empty_state_emitted);
        assert!(baseline.should_emit_empty_state(2));

        baseline.acknowledge(Some(EmitOutcome::Emitted), true, None);
        assert!(baseline.empty_state_emitted);
        assert!(!baseline.should_emit_empty_state(2));
        assert!(baseline.artwork_track.is_empty());
        assert!(baseline.artwork.is_empty());
        assert!(baseline.artwork_unchanged("", ""));
    }
}

#[cfg(test)]
mod capture_monitor_tests {
    use super::CaptureMonitorPreferences;

    #[test]
    fn screenshot_detection_keeps_the_shortcut_poll_responsive() {
        let preferences = CaptureMonitorPreferences {
            screenshot: true,
            recording: false,
            fullscreen: false,
        };
        assert_eq!(preferences.poll_interval().as_millis(), 50);
    }

    #[test]
    fn non_shortcut_detectors_use_their_own_half_second_cadence() {
        let preferences = CaptureMonitorPreferences {
            screenshot: false,
            recording: true,
            fullscreen: true,
        };
        assert_eq!(preferences.poll_interval().as_millis(), 500);
    }

    #[test]
    fn disabled_native_detectors_drop_to_the_idle_cadence() {
        let preferences = CaptureMonitorPreferences {
            screenshot: false,
            recording: false,
            fullscreen: false,
        };
        assert!(!preferences.any_native_detection());
        assert_eq!(preferences.poll_interval().as_secs(), 1);
    }
}
