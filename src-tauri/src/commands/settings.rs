//! 设置相关命令模块
//!
//! 提供应用设置的读取、保存和各项配置的修改命令

use crate::error::{AppError, AppResult};
use crate::event_bus::EVENT_BUS;
use crate::models::{AppPreferences, AppSettings};
use crate::services::{
    get_auto_start as get_auto_start_service, set_auto_start, write_settings_file,
};
use crate::state::AppState;
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager, State};

/// Clear the Windows-wide capture protection flag.
///
/// `set_content_protected` is a single window-level switch on Windows. It
/// cannot distinguish screenshots, recordings, or screen sharing, so the
/// individual capture preferences are handled by the capture-mode monitor and
/// the front-end visibility flow instead of being OR-ed into this flag.
pub(crate) fn reset_content_protection(app: &AppHandle) {
    for label in ["main", "floating_player"] {
        if let Some(window) = app.get_webview_window(label) {
            if let Err(error) = window.set_content_protected(false) {
                tracing::warn!("[Capture Mode] 设置窗口 {} 内容保护失败: {}", label, error);
            }
        }
    }
}

/// 获取当前应用设置
#[tauri::command]
pub fn get_settings(_app: AppHandle, state: State<'_, AppState>) -> AppResult<AppSettings> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    Ok(settings.clone())
}

/// Current Isle Studio compatibility endpoint.
#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> AppResult<AppPreferences> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock preferences"))?;
    Ok(settings.clone())
}

fn normalize_settings(settings: &mut AppPreferences) {
    settings.island_widgets.normalize();
    settings.floating_expanded_length = settings.floating_expanded_length.clamp(480, 900);
    settings.edge_expanded_length = settings.edge_expanded_length.clamp(480, 900);
    settings.floating_compact_length = settings.floating_compact_length.clamp(80, 300);
    settings.edge_compact_length = settings.edge_compact_length.clamp(80, 300);
    settings.island_edge_position = settings.island_edge_position.min(100);
    settings.collapsed_edge_shoulder_radius = settings.collapsed_edge_shoulder_radius.min(16);
    settings.expanded_edge_shoulder_radius = settings.expanded_edge_shoulder_radius.min(64);
    settings.floating_expanded_corner_radius = settings.floating_expanded_corner_radius.min(80);
    settings.edge_expanded_corner_radius = settings.edge_expanded_corner_radius.min(80);
    settings.idle_rotation_seconds = settings.idle_rotation_seconds.clamp(2, 60);
    settings.font_id = "misans".to_string();
    settings
        .player_source_overrides
        .retain(|id, source| !id.trim().is_empty() && id.len() <= 1024 && source == "netease");
    if let Some(ids) = settings.selected_player_ids.as_mut() {
        let mut seen = std::collections::HashSet::new();
        ids.retain(|id| !id.trim().is_empty() && seen.insert(id.clone()));
    }
    let mut seen = std::collections::HashSet::new();
    settings
        .player_order_ids
        .retain(|id| !id.trim().is_empty() && seen.insert(id.clone()));
    settings.idle_items.retain_mut(|item| {
        if item.kind == "custom" {
            item.text = item.text.trim().chars().take(120).collect();
            !item.text.is_empty()
        } else {
            matches!(
                item.kind.as_str(),
                "clock" | "date" | "weather" | "network" | "cpu" | "memory" | "battery"
            )
        }
    });
}

async fn apply_auto_start(enable: bool) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || set_auto_start(enable))
        .await
        .map_err(|error| AppError::config(format!("更新开机启动任务失败：{error}")))?
}

async fn apply_external_changes(
    app: &AppHandle,
    from: &AppPreferences,
    to: &AppPreferences,
) -> AppResult<()> {
    let top_changed = from.always_on_top != to.always_on_top;
    let floating_top_changed =
        from.floating_window_always_on_top != to.floating_window_always_on_top;
    let auto_start_changed = from.auto_start != to.auto_start;
    let capture_preferences_changed = from.capture_hide_on_screenshot
        != to.capture_hide_on_screenshot
        || from.capture_hide_on_recording != to.capture_hide_on_recording
        || from.capture_hide_on_fullscreen != to.capture_hide_on_fullscreen
        || from.capture_hide_on_screen_share != to.capture_hide_on_screen_share;

    if top_changed {
        if let Err(error) = apply_island_always_on_top(app, to.always_on_top) {
            return Err(error);
        }
    }
    if floating_top_changed {
        if let Err(error) =
            apply_floating_window_always_on_top(app, to.floating_window_always_on_top)
        {
            let mut rollback_errors = Vec::new();
            if top_changed {
                if let Err(rollback) = apply_island_always_on_top(app, from.always_on_top) {
                    rollback_errors.push(rollback.to_string());
                }
            }
            return Err(if rollback_errors.is_empty() {
                error
            } else {
                AppError::config(format!(
                    "{error}；外部设置回滚失败：{}",
                    rollback_errors.join("；")
                ))
            });
        }
    }
    if auto_start_changed {
        if let Err(error) = apply_auto_start(to.auto_start).await {
            let mut rollback_errors = Vec::new();
            if let Err(rollback) = apply_auto_start(from.auto_start).await {
                rollback_errors.push(rollback.to_string());
            }
            if floating_top_changed {
                if let Err(rollback) =
                    apply_floating_window_always_on_top(app, from.floating_window_always_on_top)
                {
                    rollback_errors.push(rollback.to_string());
                }
            }
            if top_changed {
                if let Err(rollback) = apply_island_always_on_top(app, from.always_on_top) {
                    rollback_errors.push(rollback.to_string());
                }
            }
            return Err(if rollback_errors.is_empty() {
                error
            } else {
                AppError::config(format!(
                    "{error}；外部设置回滚失败：{}",
                    rollback_errors.join("；")
                ))
            });
        }
    }
    if capture_preferences_changed {
        reset_content_protection(app);
    }
    Ok(())
}

async fn commit_settings<F>(app: AppHandle, state: &AppState, mutate: F) -> AppResult<()>
where
    F: FnOnce(&mut AppPreferences) -> AppResult<()> + Send,
{
    let apply_app = app.clone();
    let rollback_app = app.clone();
    commit_settings_transaction(
        state,
        mutate,
        move |old, next| async move { apply_external_changes(&apply_app, &old, &next).await },
        move |next| async move {
            let app_for_write = app.clone();
            tauri::async_runtime::spawn_blocking(move || write_settings_file(&app_for_write, &next))
                .await
                .map_err(|error| AppError::config(format!("保存设置任务失败：{error}")))?
        },
        move |next, old| async move { apply_external_changes(&rollback_app, &next, &old).await },
    )
    .await
}

pub(crate) async fn bind_netease_player(
    app: AppHandle,
    state: &AppState,
    id: String,
) -> AppResult<()> {
    commit_settings(app, state, move |settings| {
        settings
            .player_source_overrides
            .insert(id.clone(), "netease".into());
        settings.player_order_ids.retain(|saved| saved != &id);
        settings.player_order_ids.insert(0, id.clone());
        if let Some(selected) = settings.selected_player_ids.as_mut() {
            if !selected.contains(&id) {
                selected.push(id);
            }
        }
        settings.netease_progress_mode = crate::models::NeteaseProgressMode::Real;
        Ok(())
    })
    .await
}

pub(crate) async fn remember_netease_connection(
    app: AppHandle,
    state: &AppState,
    path: String,
    enable: bool,
) -> AppResult<()> {
    commit_settings(app, state, move |settings| {
        settings.netease_executable_path = Some(path);
        if enable {
            settings.netease_auto_connect = true;
        }
        Ok(())
    })
    .await
}

fn restore_netease_identity(settings: &mut AppPreferences, id: &str) {
    let old_ids: std::collections::HashSet<String> = settings
        .player_order_ids
        .iter()
        .chain(settings.selected_player_ids.iter().flatten())
        .filter(|old| {
            old.to_ascii_lowercase().contains("cloudmusic")
                || settings
                    .player_source_overrides
                    .get(*old)
                    .is_some_and(|source| source == "netease")
        })
        .cloned()
        .collect();
    settings
        .player_source_overrides
        .insert(id.to_string(), "netease".into());
    // Replace the launch-specific ID at its saved priority, preserving unrelated
    // player choices and keeping an explicitly deselected NetEase deselected.
    if !settings.player_order_ids.iter().any(|saved| saved == id) {
        if let Some(slot) = settings
            .player_order_ids
            .iter_mut()
            .find(|saved| old_ids.contains(*saved))
        {
            *slot = id.to_string();
        }
    }
    if let Some(selected) = settings.selected_player_ids.as_mut() {
        if !selected.iter().any(|saved| saved == id) {
            for saved in selected.iter_mut().filter(|saved| old_ids.contains(*saved)) {
                *saved = id.to_string();
            }
        }
    }
}

pub(crate) async fn restore_netease_player(
    app: AppHandle,
    state: &AppState,
    id: String,
) -> AppResult<()> {
    commit_settings(app, state, move |settings| {
        restore_netease_identity(settings, &id);
        Ok(())
    })
    .await
}

async fn commit_settings_transaction<F, A, AFut, P, PFut, R, RFut>(
    state: &AppState,
    mutate: F,
    apply: A,
    persist: P,
    rollback: R,
) -> AppResult<()>
where
    F: FnOnce(&mut AppPreferences) -> AppResult<()> + Send,
    A: FnOnce(AppPreferences, AppPreferences) -> AFut + Send,
    AFut: Future<Output = AppResult<()>> + Send,
    P: FnOnce(AppPreferences) -> PFut + Send,
    PFut: Future<Output = AppResult<()>> + Send,
    R: FnOnce(AppPreferences, AppPreferences) -> RFut + Send,
    RFut: Future<Output = AppResult<()>> + Send,
{
    let _writer = state.settings_writer.lock().await;
    let old = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?
        .clone();
    let mut next = old.clone();
    mutate(&mut next)?;
    normalize_settings(&mut next);
    if next == old {
        return Ok(());
    }

    apply(old.clone(), next.clone()).await?;
    if let Err(error) = persist(next.clone()).await {
        return match rollback(next, old).await {
            Ok(()) => Err(error),
            Err(rollback) => Err(AppError::config(format!(
                "{error}；外部设置回滚失败：{rollback}"
            ))),
        };
    }

    *state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))? = next.clone();
    state.set_netease_progress_mode(next.netease_progress_mode);
    if old.netease_progress_mode != next.netease_progress_mode {
        if let Ok(actor) = state.media_actor() {
            actor.wake_for_netease_progress_mode_change();
        }
    }
    if old.player_source_overrides != next.player_source_overrides {
        if let Ok(actor) = state.media_actor() {
            actor.wake_for_player_identity_change();
        }
    }
    let revision = state.settings_revision.fetch_add(1, Ordering::SeqCst) + 1;
    emit_setting_diffs(&old, &next);
    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_SETTINGS_UPDATED, next.clone());
    let _ = EVENT_BUS.emit(
        crate::event_bus::EVENT_SETTINGS_COMMITTED,
        serde_json::json!({"revision": revision}),
    );
    Ok(())
}

fn emit_setting_diffs(old: &AppPreferences, settings: &AppPreferences) {
    if old.enable_halftone != settings.enable_halftone {
        let _ = EVENT_BUS.emit(
            crate::event_bus::EVENT_HALFTONE_CHANGED,
            serde_json::json!({"enableHalftone": settings.enable_halftone}),
        );
    }
    if old.enable_pixel_art != settings.enable_pixel_art {
        let _ = EVENT_BUS.emit(
            crate::event_bus::EVENT_PIXEL_ART_CHANGED,
            serde_json::json!({"enablePixelArt": settings.enable_pixel_art}),
        );
    }
    if old.enable_hd_cover != settings.enable_hd_cover {
        let _ = EVENT_BUS.emit(
            crate::event_bus::EVENT_HD_COVER_CHANGED,
            serde_json::json!({"enableHDCover": settings.enable_hd_cover}),
        );
    }
    if old.enable_mv_playback != settings.enable_mv_playback {
        let _ = EVENT_BUS.emit(
            crate::event_bus::EVENT_MV_PLAYBACK_CHANGED,
            serde_json::json!({"enable": settings.enable_mv_playback}),
        );
    }
    if old.lock_floating_window != settings.lock_floating_window {
        let _ = EVENT_BUS.emit(
            crate::event_bus::EVENT_LOCK_FLOATING_WINDOW_CHANGED,
            serde_json::json!({"lock": settings.lock_floating_window}),
        );
    }
    if old.always_on_top != settings.always_on_top {
        let _ = EVENT_BUS.emit(
            crate::event_bus::EVENT_ALWAYS_ON_TOP_CHANGED,
            serde_json::json!({"isAlwaysOnTop": settings.always_on_top}),
        );
    }
    if old.floating_window_always_on_top != settings.floating_window_always_on_top {
        let _ = EVENT_BUS.emit(
            crate::event_bus::EVENT_FLOATING_WINDOW_ALWAYS_ON_TOP_CHANGED,
            serde_json::json!({"isAlwaysOnTop": settings.floating_window_always_on_top}),
        );
    }
    if old.monitor_index != settings.monitor_index {
        let _ = EVENT_BUS.emit(crate::event_bus::EVENT_SETTINGS_CHANGED, "monitor_index");
    }
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> AppResult<()> {
    commit_settings(app, &state, move |current| {
        *current = settings;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    patch: serde_json::Value,
) -> AppResult<()> {
    let patch_object = patch
        .as_object()
        .cloned()
        .ok_or_else(|| AppError::business(3001, "设置补丁必须是对象"))?;
    commit_settings(app, &state, move |current| {
        let mut merged = serde_json::to_value(&*current)?
            .as_object()
            .cloned()
            .ok_or_else(|| AppError::config("当前设置格式无效"))?;
        for (key, value) in patch_object {
            merged.insert(key, value);
        }
        *current = serde_json::from_value(serde_json::Value::Object(merged))?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn save_preferences(
    app: AppHandle,
    state: State<'_, AppState>,
    preferences: AppPreferences,
) -> AppResult<()> {
    save_settings(app, state, preferences).await
}

/// 设置窗口置顶
#[tauri::command]
pub async fn set_always_on_top(
    app: AppHandle,
    state: State<'_, AppState>,
    enable: bool,
) -> AppResult<()> {
    commit_settings(app, &state, move |settings| {
        settings.always_on_top = enable;
        Ok(())
    })
    .await
}

/// Apply the configured topmost state to the main island only.
fn apply_island_always_on_top(app: &AppHandle, enable: bool) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_always_on_top(enable)
            .map_err(|error| AppError::window(format!("设置灵动岛置顶失败：{}", error)))?;
    }
    Ok(())
}

/// Set the floating player's independent topmost preference.
#[tauri::command]
pub async fn set_floating_window_always_on_top(
    app: AppHandle,
    state: State<'_, AppState>,
    enable: bool,
) -> AppResult<()> {
    commit_settings(app, &state, move |settings| {
        settings.floating_window_always_on_top = enable;
        Ok(())
    })
    .await
}

fn apply_floating_window_always_on_top(app: &AppHandle, enable: bool) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("floating_player") {
        window
            .set_always_on_top(enable)
            .map_err(|error| AppError::window(format!("设置悬浮窗置顶失败：{}", error)))?;
    }
    Ok(())
}

/// 设置窗口不透明度
#[tauri::command]
pub async fn set_window_opacity(
    app: AppHandle,
    state: State<'_, AppState>,
    opacity: u8,
) -> AppResult<()> {
    commit_settings(app, &state, move |settings| {
        settings.window_opacity = opacity;
        Ok(())
    })
    .await
}

/// 获取播放器优先级权重配置
#[tauri::command]
pub fn get_player_weights(state: State<'_, AppState>) -> AppResult<HashMap<String, u32>> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    Ok(settings.player_weights.clone())
}

/// 设置单个播放器的优先级权重
#[tauri::command]
pub async fn set_player_weight(
    app: AppHandle,
    state: State<'_, AppState>,
    player: String,
    weight: u32,
) -> AppResult<()> {
    commit_settings(app, &state, move |settings| {
        settings.player_weights.insert(player, weight);
        Ok(())
    })
    .await
}

/// 批量设置播放器优先级权重
#[tauri::command]
pub async fn set_player_weights(
    app: AppHandle,
    state: State<'_, AppState>,
    weights: HashMap<String, u32>,
) -> AppResult<()> {
    commit_settings(app, &state, move |settings| {
        settings.player_weights = weights;
        Ok(())
    })
    .await
}

/// 设置开机自启动
#[tauri::command]
pub async fn set_auto_start_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    enable: bool,
) -> AppResult<()> {
    commit_settings(app, &state, move |settings| {
        settings.auto_start = enable;
        Ok(())
    })
    .await
}

/// 获取开机自启动状态
#[tauri::command]
pub async fn get_auto_start() -> AppResult<bool> {
    tauri::async_runtime::spawn_blocking(get_auto_start_service)
        .await
        .map_err(|error| AppError::config(format!("读取开机启动状态任务失败：{}", error)))?
}

/// 保存悬浮窗口位置和大小
#[tauri::command]
pub async fn save_floating_window_position(
    app: AppHandle,
    state: State<'_, AppState>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> AppResult<()> {
    commit_settings(app, &state, move |settings| {
        settings.floating_window_x = Some(x);
        settings.floating_window_y = Some(y);
        settings.floating_window_width = Some(width.max(200));
        settings.floating_window_height = Some(height.max(200));
        Ok(())
    })
    .await
}

/// 获取悬浮窗口位置和大小
#[tauri::command]
pub fn get_floating_window_position(
    state: State<'_, AppState>,
) -> AppResult<Option<(i32, i32, u32, u32)>> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;

    if let (Some(x), Some(y), Some(w), Some(h)) = (
        settings.floating_window_x,
        settings.floating_window_y,
        settings.floating_window_width,
        settings.floating_window_height,
    ) {
        Ok(Some((x, y, w, h)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod settings_commit_tests {
    #[test]
    fn restore_replaces_launcher_id_without_changing_player_preferences_or_progress_mode() {
        let mut preferences = crate::models::AppPreferences::default();
        preferences.player_order_ids = vec!["Spotify.exe".into(), "cloudmusic.exe".into()];
        preferences.selected_player_ids = Some(vec!["Spotify.exe".into(), "cloudmusic.exe".into()]);
        preferences.netease_progress_mode = crate::models::NeteaseProgressMode::Estimate;
        super::restore_netease_identity(&mut preferences, "Microsoft.AutoGenerated.new");
        assert_eq!(
            preferences.player_order_ids,
            ["Spotify.exe", "Microsoft.AutoGenerated.new"]
        );
        assert_eq!(
            preferences.selected_player_ids.as_ref().unwrap(),
            &["Spotify.exe", "Microsoft.AutoGenerated.new"]
        );
        assert_eq!(
            preferences.netease_progress_mode,
            crate::models::NeteaseProgressMode::Estimate
        );
        preferences.selected_player_ids = Some(vec!["Spotify.exe".into()]);
        super::restore_netease_identity(&mut preferences, "Microsoft.AutoGenerated.later");
        assert_eq!(preferences.selected_player_ids.unwrap(), ["Spotify.exe"]);
    }
    use super::commit_settings_transaction;
    use crate::models::AppPreferences;
    use crate::services::settings::{write_atomic_with_backup, write_settings_snapshot};
    use crate::state::AppState;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Instant;

    #[tokio::test]
    async fn one_hundred_concurrent_setting_and_position_writes_keep_disk_memory_and_revision_aligned(
    ) {
        let root = std::env::temp_dir().join(format!("isle-settings-race-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create test directory");
        let path = root.join("settings.json");
        let state = Arc::new(AppState::default());
        let started = Instant::now();
        let mut tasks = Vec::with_capacity(100);

        for index in 0..100_u32 {
            let state = Arc::clone(&state);
            let path = path.clone();
            tasks.push(tokio::spawn(async move {
                commit_settings_transaction(
                    &state,
                    move |next| {
                        next.player_weights.insert(format!("writer-{index}"), index);
                        next.floating_window_x = Some(index as i32);
                        next.floating_window_y = Some(-(index as i32));
                        next.floating_window_width = Some(200 + index);
                        next.floating_window_height = Some(300 + index);
                        Ok(())
                    },
                    |_, _| async { Ok(()) },
                    move |next| async move {
                        tokio::task::yield_now().await;
                        write_settings_snapshot(&path, &next)
                    },
                    |_, _| async { Ok(()) },
                )
                .await
                .expect("atomic settings commit");
            }));
        }

        for task in tasks {
            task.await.expect("settings writer task");
        }
        let elapsed = started.elapsed();
        let memory = state.settings.lock().expect("settings lock").clone();
        let disk: AppPreferences =
            serde_json::from_slice(&std::fs::read(&path).expect("persisted settings"))
                .expect("parse persisted settings");
        assert_eq!(memory, disk);
        assert_eq!(state.settings_revision.load(Ordering::SeqCst), 100);
        for index in 0..100_u32 {
            assert_eq!(
                memory.player_weights.get(&format!("writer-{index}")),
                Some(&index)
            );
        }
        assert!(memory.floating_window_x.is_some());
        assert!(memory.floating_window_y.is_some());
        assert!(memory.floating_window_width.unwrap_or_default() >= 200);
        assert!(memory.floating_window_height.unwrap_or_default() >= 300);
        eprintln!("100 serialized atomic settings commits: {elapsed:?}");
        assert!(
            elapsed.as_secs() < 10,
            "settings writes exceeded the broad latency guard: {elapsed:?}"
        );
        std::fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn reports_direct_write_and_atomic_replace_latency_for_equal_payloads() {
        let root =
            std::env::temp_dir().join(format!("isle-settings-latency-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create test directory");
        let direct_path = root.join("direct.json");
        let atomic_path = root.join("settings.json");
        let payload = serde_json::to_vec(&AppPreferences::default()).expect("serialize settings");
        let direct_started = Instant::now();
        for _ in 0..100 {
            std::fs::write(&direct_path, &payload).expect("direct baseline write");
        }
        let direct_elapsed = direct_started.elapsed();
        let atomic_started = Instant::now();
        for _ in 0..100 {
            write_atomic_with_backup(&atomic_path, &payload).expect("atomic settings write");
        }
        let atomic_elapsed = atomic_started.elapsed();
        assert_eq!(
            std::fs::read(&direct_path).expect("direct bytes"),
            std::fs::read(&atomic_path).expect("atomic bytes")
        );
        eprintln!("100 equal-payload writes: direct={direct_elapsed:?}, atomic={atomic_elapsed:?}");
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[tokio::test]
    async fn failed_disk_and_external_operations_leave_committed_settings_unchanged() {
        let root =
            std::env::temp_dir().join(format!("isle-settings-failure-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create test directory");
        let invalid_target = root.join("settings.json");
        std::fs::create_dir(&invalid_target).expect("force a persistence failure");
        let state = AppState::default();
        let external_state = Arc::new(AtomicBool::new(false));
        let applied = Arc::clone(&external_state);
        let compensated = Arc::clone(&external_state);
        let result = commit_settings_transaction(
            &state,
            |next| {
                next.window_opacity = 120;
                Ok(())
            },
            move |_, _| async move {
                applied.store(true, Ordering::SeqCst);
                Ok(())
            },
            move |next| async move { write_settings_snapshot(&invalid_target, &next) },
            move |_, _| async move {
                compensated.store(false, Ordering::SeqCst);
                Ok(())
            },
        )
        .await;
        assert!(result.is_err());
        assert_eq!(
            state.settings.lock().expect("settings lock").window_opacity,
            255
        );
        assert_eq!(state.settings_revision.load(Ordering::SeqCst), 0);
        assert!(!external_state.load(Ordering::SeqCst));

        let state = AppState::default();
        let disk_attempted = Arc::new(AtomicBool::new(false));
        let observed = Arc::clone(&disk_attempted);
        let result = commit_settings_transaction(
            &state,
            |next| {
                next.window_opacity = 90;
                Ok(())
            },
            |_, _| async {
                Err(crate::error::AppError::window(
                    "simulated OS setting failure",
                ))
            },
            move |_| async move {
                observed.store(true, Ordering::SeqCst);
                Ok(())
            },
            |_, _| async { Ok(()) },
        )
        .await;
        assert!(result.is_err());
        assert_eq!(
            state.settings.lock().expect("settings lock").window_opacity,
            255
        );
        assert_eq!(state.settings_revision.load(Ordering::SeqCst), 0);
        assert!(!disk_attempted.load(Ordering::SeqCst));
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
