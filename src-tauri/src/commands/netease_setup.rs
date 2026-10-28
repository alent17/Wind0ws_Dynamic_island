use serde::Serialize;
use tauri::Manager;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, atomic::{AtomicBool, Ordering}};

static SETUP: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);
static MONITOR_STARTED: AtomicBool = AtomicBool::new(false);
static LAUNCH_WATCH: OnceLock<Mutex<Option<LaunchWatch>>> = OnceLock::new();

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NeteaseSetupStage { Waiting, Detecting, Configuring, WaitingPlayer, Binding, Saving }

fn report_stage(progress: Option<&tauri::ipc::Channel<NeteaseSetupStage>>, stage: NeteaseSetupStage) {
    if let Some(progress) = progress { let _ = progress.send(stage); }
}

fn read_setup_output<R: std::io::Read + Send + 'static>(stream: R, limit: u64)
    -> std::sync::mpsc::Receiver<Result<Vec<u8>, ()>> {
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stream.take(limit + 1).read_to_end(&mut bytes)
            .map_err(|_| ()).and_then(|_| if bytes.len() as u64 > limit { Err(()) } else { Ok(bytes) });
        let _ = send.send(result);
    });
    receive
}

struct LaunchWatch {
    paths: Vec<PathBuf>,
    fingerprint: u64,
}

fn launch_fingerprint(paths: &[PathBuf]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    for path in paths {
        path.hash(&mut hash);
        let metadata = std::fs::metadata(path).ok();
        metadata.as_ref().map(|value| (value.len(), value.modified().ok(), value.is_dir())).hash(&mut hash);
    }
    hash.finish()
}

fn remember_launch_watch(paths: Vec<PathBuf>) {
    if paths.is_empty() { return; }
    let fingerprint = launch_fingerprint(&paths);
    if let Ok(mut watch) = LAUNCH_WATCH.get_or_init(|| Mutex::new(None)).lock() {
        *watch = Some(LaunchWatch { paths, fingerprint });
    }
}

fn launch_entries_changed() -> Option<bool> {
    let watch = LAUNCH_WATCH.get_or_init(|| Mutex::new(None)).lock().ok()?;
    let watch = watch.as_ref()?;
    Some(launch_fingerprint(&watch.paths) != watch.fingerprint)
}

fn connection_probe_attempts(action: &str, restart_required: bool) -> u8 {
    match action {
        "OpenStandard" | "Restore" => 0,
        "OpenEnhanced" if restart_required => 0,
        "Check" => 1,
        _ => 12,
    }
}

fn should_bind_player(configure_only: bool, connected: bool, bind: bool) -> bool {
    !configure_only && connected && bind
}

/// Only metadata is checked every 30 seconds. Full discovery and PowerShell run
/// after a change; a successful repair records its own writes as the baseline.
fn start_launch_monitor(app: tauri::AppHandle) {
    if MONITOR_STARTED.swap(true, Ordering::AcqRel) { return; }
    tauri::async_runtime::spawn(async move {
        let mut bootstrap_attempted = false;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let enabled = app.state::<crate::state::AppState>().settings.lock().is_ok_and(|settings|
                settings.netease_auto_connect && settings.netease_connection_mode == crate::models::NeteaseConnectionMode::Enhanced);
            if !enabled { bootstrap_attempted = false; continue; }
            let changed = super::native_task::run_background(launch_entries_changed).await;
            match changed {
                Ok(Some(true)) => start_netease_auto_restore(app.clone()),
                Ok(None) if !bootstrap_attempted => {
                    bootstrap_attempted = true;
                    start_netease_auto_restore(app.clone());
                }
                _ => {}
            }
        }
    });
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseConnection {
    connected: bool,
    configured: bool,
    executable_path: String,
    shortcut_path: String,
    bound_session_id: Option<String>,
    binding_failed: bool,
    launch_entries: u64,
    restart_required: bool,
    pending_start_menu_entries: u64,
    #[serde(skip)]
    watch_paths: Vec<PathBuf>,
}

fn configure(
    action: &'static str,
    preferred_path: Option<String>,
) -> Result<NeteaseConnection, String> {
    configure_with_progress(action, preferred_path, None, false)
}

fn configure_with_progress(
    action: &'static str,
    preferred_path: Option<String>,
    progress: Option<&tauri::ipc::Channel<NeteaseSetupStage>>,
    repair_start_menu: bool,
) -> Result<NeteaseConnection, String> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    report_stage(progress, NeteaseSetupStage::Detecting);
    let already_connected = action != "OpenStandard" && crate::services::netease_cdp::LocalCdpClient::default()
            .connection_ready_until(Instant::now() + Duration::from_secs(5))
            .is_ok();
    report_stage(progress, if action == "Check" { NeteaseSetupStage::Detecting } else { NeteaseSetupStage::Configuring });
    let mut command = Command::new("powershell.exe");
    command.env_remove("ISLE_NETEASE_PATH");
    if let Some(path) = preferred_path {
        command.env("ISLE_NETEASE_PATH", path);
    }
    let mut child = command
        .args(["-NoProfile", "-NonInteractive", "-Command"])
        .arg(format!(
            "& {{ {} }} -Action {} {}",
            include_str!("../../windows/netease-connect.ps1"),
            action,
            if repair_start_menu { "-RepairCommonStartMenu" } else { "" }
        ))
        .env(
            "ISLE_NETEASE_CONNECTED",
            if already_connected { "1" } else { "0" },
        )
        .creation_flags(0x08000000)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "NETEASE_SETUP_FAILED".to_string())?;
    // The watch list can exceed the OS pipe buffer. Drain both streams while
    // the child is running, with bounds, instead of waiting for exit first.
    let stdout = read_setup_output(child.stdout.take().ok_or("NETEASE_SETUP_FAILED")?, 512 * 1024);
    let stderr = read_setup_output(child.stderr.take().ok_or("NETEASE_SETUP_FAILED")?, 16 * 1024);
    // Explicit common-menu repair may wait for a Windows UAC response.
    let deadline = Instant::now() + Duration::from_secs(if repair_start_menu { 120 } else { 30 });
    loop {
        if child
            .try_wait()
            .map_err(|_| "NETEASE_SETUP_FAILED")?
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("NETEASE_SETUP_TIMEOUT".into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let status = child
        .wait()
        .map_err(|_| "NETEASE_SETUP_FAILED")?;
    let stdout = stdout.recv_timeout(Duration::from_secs(1)).map_err(|_| "NETEASE_SETUP_FAILED")?
        .map_err(|_| "NETEASE_SETUP_FAILED")?;
    let stderr = stderr.recv_timeout(Duration::from_secs(1)).map_err(|_| "NETEASE_SETUP_FAILED")?
        .map_err(|_| "NETEASE_SETUP_FAILED")?;
    if !status.success() {
        return Err(
            if String::from_utf8_lossy(&stderr).contains("NETEASE_NOT_FOUND") {
                "NETEASE_NOT_FOUND"
            } else if String::from_utf8_lossy(&stderr).contains("NETEASE_RESTART_FAILED") {
                "NETEASE_RESTART_FAILED"
            } else {
                "NETEASE_SETUP_FAILED"
            }
            .into(),
        );
    }
    let data: serde_json::Value =
        serde_json::from_slice(&stdout).map_err(|_| "NETEASE_SETUP_FAILED")?;
    // Discovery/repair can take seconds. Manual results must confirm current
    // readiness instead of reporting the pre-discovery connection as success.
    let mut connected = already_connected && action == "Restore";
    // Repair is not a connection retry loop; the media sampler retries CDP.
    let attempts = connection_probe_attempts(action, data["restartRequired"].as_bool().unwrap_or(false));
    let probe_deadline = Instant::now() + Duration::from_secs(15);
    if !connected && attempts > 0 { report_stage(progress, NeteaseSetupStage::WaitingPlayer); }
    for attempt in 0..attempts {
        if connected {
            break;
        }
        if attempt > 0 {
            std::thread::sleep(Duration::from_secs(1));
        }
        if Instant::now() >= probe_deadline { break; }
        if crate::services::netease_cdp::LocalCdpClient::default()
            .connection_ready_until(probe_deadline)
            .is_ok()
        {
            connected = true;
            break;
        }
    }
    let result = NeteaseConnection {
        connected,
        configured: data["configured"].as_bool().unwrap_or(false),
        executable_path: data["executablePath"].as_str().unwrap_or_default().into(),
        shortcut_path: data["shortcutPath"].as_str().unwrap_or_default().into(),
        bound_session_id: None,
        binding_failed: false,
        launch_entries: data["launchEntries"].as_u64().unwrap_or(0),
        restart_required: !connected && data["restartRequired"].as_bool().unwrap_or(false),
        pending_start_menu_entries: data["pendingStartMenuEntries"].as_u64().unwrap_or(0),
        watch_paths: data["watchPaths"].as_array().map(|values| values.iter().take(512)
            .filter_map(|value| value.as_str().map(PathBuf::from)).collect()).unwrap_or_default(),
    };
    if result.configured && matches!(action, "Restore" | "Connect" | "OpenEnhanced") {
        remember_launch_watch(result.watch_paths.clone());
    }
    Ok(result)
}

#[tauri::command]
pub async fn setup_netease_connection(
    app: tauri::AppHandle,
    webview: tauri::Webview,
    state: tauri::State<'_, crate::state::AppState>,
    restart: bool,
    bind: Option<bool>,
    configure_only: Option<bool>,
    progress: Option<tauri::ipc::JavaScriptChannelId>,
    repair_start_menu: Option<bool>,
) -> Result<NeteaseConnection, String> {
    let progress = progress.map(|id| id.channel_on(webview));
    // An automatic repair may already be running when the user clicks Connect.
    // Serialize behind it instead of rejecting that click as a failed setup.
    report_stage(progress.as_ref(), NeteaseSetupStage::Waiting);
    let _permit = SETUP.acquire().await.map_err(|_| "NETEASE_SETUP_BUSY")?;
    if !state.netease_enhanced() { return Err("NETEASE_ENHANCED_DISABLED".into()); }
    let configure_only = configure_only.unwrap_or(false);
    setup_connection(app, state, restart, bind, configure_only, configure_only, progress, repair_start_menu.unwrap_or(false) && configure_only).await
}

async fn setup_connection(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
    restart: bool,
    bind: Option<bool>,
    configure_only: bool,
    preserve_preferences: bool,
    progress: Option<tauri::ipc::Channel<NeteaseSetupStage>>,
    repair_start_menu: bool,
) -> Result<NeteaseConnection, String> {
    let preferred_path = state
        .settings
        .lock()
        .ok()
        .and_then(|settings| settings.netease_executable_path.clone());
    // SETUP already bounds this work. Sharing the fail-fast artwork/background
    // pool made manual connection fail whenever its two slots were occupied.
    let worker_progress = progress.clone();
    let mut result = tauri::async_runtime::spawn_blocking(move || {
        configure_with_progress(if configure_only { "Restore" } else if restart { "Connect" } else { "Check" }, preferred_path, worker_progress.as_ref(), repair_start_menu)
    })
    .await
    .map_err(|_| "NETEASE_SETUP_FAILED".to_string())??;
    state.set_netease_cdp_available(result.connected);
    start_launch_monitor(app.clone());
    if result.configured && bind.unwrap_or(restart) {
        report_stage(progress.as_ref(), NeteaseSetupStage::Saving);
        super::settings::remember_netease_connection(
            app.clone(),
            &state,
            result.executable_path.clone(),
            !preserve_preferences,
        )
        .await
        .map_err(|_| "NETEASE_SETUP_FAILED".to_string())?;
    }
    if should_bind_player(configure_only, result.connected, bind.unwrap_or(restart)) {
        report_stage(progress.as_ref(), NeteaseSetupStage::Binding);
        let session_id = tauri::async_runtime::spawn_blocking(|| {
            for attempt in 0..12 {
                if attempt > 0 {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                if let Some(id) = crate::services::media::verified_netease_binding() {
                    return Some(id);
                }
            }
            None
        })
        .await
        .map_err(|_| "NETEASE_SETUP_FAILED".to_string())?;
        if let Some(id) = session_id {
            report_stage(progress.as_ref(), NeteaseSetupStage::Saving);
            let binding = if preserve_preferences {
                super::settings::restore_netease_player(app, &state, id.clone()).await
            } else {
                super::settings::bind_netease_player(app, &state, id.clone()).await
            };
            if binding.is_ok() {
                result.bound_session_id = Some(id);
                if let Ok(actor) = state.media_actor() {
                    actor.wake_for_player_identity_change();
                }
            } else {
                tracing::warn!("[NetEase] Could not save player binding: {:?}", binding.err());
                result.binding_failed = true;
            }
        }
    }
    Ok(result)
}

/// CDP is the primary launch path. An already running player is never restarted
/// implicitly; a missing startup flag is reported for explicit reconnection.
pub(crate) async fn open_netease_player(app: tauri::AppHandle) -> Result<(), String> {
    let _permit = SETUP.acquire().await.map_err(|_| "NETEASE_SETUP_BUSY")?;
    let preferred_path = app.state::<crate::state::AppState>().settings.lock().ok()
        .and_then(|settings| settings.netease_executable_path.clone());
    let enhanced = app.state::<crate::state::AppState>().netease_enhanced();
    let result = tauri::async_runtime::spawn_blocking(move || configure(if enhanced { "OpenEnhanced" } else { "OpenStandard" }, preferred_path))
        .await.map_err(|_| "NETEASE_SETUP_BUSY")??;
    if enhanced {
        let state = app.state::<crate::state::AppState>();
        state.set_netease_cdp_available(result.connected);
        if result.configured {
            super::settings::remember_netease_connection(app.clone(), &state, result.executable_path, true)
                .await.map_err(|_| "NETEASE_SETUP_FAILED")?;
        }
        start_launch_monitor(app.clone());
    }
    if let Ok(actor) = app.state::<crate::state::AppState>().media_actor() {
        actor.wake_for_player_identity_change();
    }
    Ok(())
}

pub(crate) fn start_netease_auto_restore(app: tauri::AppHandle) {
    start_launch_monitor(app.clone());
    let preferences = app
        .state::<crate::state::AppState>()
        .settings
        .lock()
        .ok()
        .map(|settings| settings.clone());
    let Some(preferences) = preferences.filter(|settings| settings.netease_auto_connect
        && settings.netease_connection_mode == crate::models::NeteaseConnectionMode::Enhanced) else {
        return;
    };
    tauri::async_runtime::spawn(async move {
        let Ok(_permit) = SETUP.try_acquire() else {
            return;
        };
        let worker_app = app.clone();
        let restored = super::native_task::run_background(move || {
            // Recheck after admission: an opt-out may have arrived while this
            // request was queued. Do not start a repair using stale preferences.
            let enabled = worker_app.state::<crate::state::AppState>().settings.lock().is_ok_and(|settings|
                settings.netease_auto_connect && settings.netease_connection_mode == crate::models::NeteaseConnectionMode::Enhanced);
            if !enabled { return Err("NETEASE_AUTO_DISABLED".to_string()); }
            // Repair updated entries without starting or restarting NetEase.
            configure("Restore", preferences.netease_executable_path)
        })
        .await;
        let Ok(Ok(result)) = restored else {
            tracing::debug!("[NetEase] Automatic connection restore deferred");
            return;
        };
        let state = app.state::<crate::state::AppState>();
        state.set_netease_cdp_available(result.connected);
        if result.configured {
            let _ = super::settings::remember_netease_connection(
                app.clone(),
                &state,
                result.executable_path,
                false,
            )
            .await;
        }
        if result.connected {
            let session = super::native_task::run_background(|| {
                for attempt in 0..4 {
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    if let Some(id) = crate::services::media::verified_netease_binding() {
                        return Some(id);
                    }
                }
                None
            })
            .await;
            if let Ok(Some(id)) = session {
                let _ = super::settings::restore_netease_player(app.clone(), &state, id).await;
            }
            if let Ok(actor) = state.media_actor() {
                actor.wake_for_player_identity_change();
            }
        }
    });
}

#[cfg(test)]
mod launch_watch_tests {
    use super::*;

    #[test]
    fn configuring_launch_entries_never_binds_or_waits_for_a_media_session() {
        assert!(!should_bind_player(true, true, true));
        assert!(!should_bind_player(false, false, true));
        assert!(!should_bind_player(false, true, false));
        assert!(should_bind_player(false, true, true));
    }

    #[test]
    #[ignore = "requires a running NetEase player; read-only local setup probe"]
    fn live_netease_setup_check() {
        let result = configure("Check", None).expect("read-only setup discovery failed");
        assert!(result.connected, "NetEase CDP did not become ready");
        assert!(!result.restart_required);
        let id = crate::services::media::verified_netease_binding();
        assert!(id.is_some(), "No unique NetEase Windows session found");
        println!("Read-only setup succeeded: configured={}, session={}", result.configured, id.unwrap());
    }

    #[test]
    fn setup_output_handles_large_watch_lists_and_rejects_overflow() {
        let bytes = vec![b'x'; 128 * 1024];
        let result = read_setup_output(std::io::Cursor::new(bytes.clone()), 512 * 1024)
            .recv_timeout(std::time::Duration::from_secs(1)).unwrap().unwrap();
        assert_eq!(result, bytes);
        assert!(read_setup_output(std::io::Cursor::new(vec![b'x'; 1025]), 1024)
            .recv_timeout(std::time::Duration::from_secs(1)).unwrap().is_err());
    }

    #[test]
    fn maintenance_and_a_running_player_without_flags_do_not_wait_for_pointless_retries() {
        assert_eq!(connection_probe_attempts("OpenStandard", false), 0);
        assert_eq!(connection_probe_attempts("Restore", false), 0);
        assert_eq!(connection_probe_attempts("OpenEnhanced", true), 0);
        assert_eq!(connection_probe_attempts("OpenEnhanced", false), 12);
        assert_eq!(connection_probe_attempts("Connect", false), 12);
        assert_eq!(connection_probe_attempts("Check", true), 1);
    }

    #[test]
    fn unchanged_entries_are_quiet_but_update_replacement_and_removal_are_detected() {
        let directory = std::env::temp_dir().join(format!("isle-launch-watch-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let entry = directory.join("NetEase.lnk");
        let executable = directory.join("cloudmusic.exe");
        std::fs::write(&entry, "configured arguments").unwrap();
        std::fs::write(&executable, "version-one").unwrap();
        let paths = vec![entry.clone(), executable.clone()];
        let original = launch_fingerprint(&paths);
        assert_eq!(original, launch_fingerprint(&paths));
        std::fs::write(&entry, "reset by updater").unwrap();
        assert_ne!(original, launch_fingerprint(&paths));
        let repaired = launch_fingerprint(&paths);
        assert_eq!(repaired, launch_fingerprint(&paths));
        std::fs::write(&executable, "version-two-with-a-new-build").unwrap();
        let updated = launch_fingerprint(&paths);
        assert_ne!(repaired, updated);
        std::fs::remove_file(&entry).unwrap();
        assert_ne!(updated, launch_fingerprint(&paths));
        std::fs::remove_file(&executable).unwrap();
        std::fs::remove_dir(&directory).unwrap();
    }
}
