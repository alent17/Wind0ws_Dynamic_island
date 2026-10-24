use serde::Serialize;
use tauri::Manager;

static SETUP: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseConnection {
    connected: bool,
    configured: bool,
    executable_path: String,
    shortcut_path: String,
    bound_session_id: Option<String>,
    binding_failed: bool,
}

fn configure(
    action: &'static str,
    preferred_path: Option<String>,
) -> Result<NeteaseConnection, String> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let already_connected = crate::services::netease_cdp::LocalCdpClient::default()
            .playback_mode()
            .is_ok();
    let mut command = Command::new("powershell.exe");
    command.env_remove("ISLE_NETEASE_PATH");
    if let Some(path) = preferred_path {
        command.env("ISLE_NETEASE_PATH", path);
    }
    let mut child = command
        .args(["-NoProfile", "-NonInteractive", "-Command"])
        .arg(format!(
            "& {{ {} }} -Action {}",
            include_str!("../../windows/netease-connect.ps1"),
            action
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
    let deadline = Instant::now() + Duration::from_secs(30);
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
    let output = child
        .wait_with_output()
        .map_err(|_| "NETEASE_SETUP_FAILED")?;
    if !output.status.success() {
        return Err(
            if String::from_utf8_lossy(&output.stderr).contains("NETEASE_NOT_FOUND") {
                "NETEASE_NOT_FOUND"
            } else {
                "NETEASE_SETUP_FAILED"
            }
            .into(),
        );
    }
    let data: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|_| "NETEASE_SETUP_FAILED")?;
    let mut connected = already_connected;
    let attempts = if action == "Check" {
        1
    } else if action == "Restore" && data["running"].as_bool() != Some(true) {
        0
    } else {
        4
    };
    for attempt in 0..attempts {
        if connected {
            break;
        }
        if attempt > 0 {
            std::thread::sleep(Duration::from_secs(2));
        }
        if crate::services::netease_cdp::LocalCdpClient::default()
            .playback_mode()
            .is_ok()
        {
            connected = true;
            break;
        }
    }
    Ok(NeteaseConnection {
        connected,
        configured: data["configured"].as_bool().unwrap_or(false),
        executable_path: data["executablePath"].as_str().unwrap_or_default().into(),
        shortcut_path: data["shortcutPath"].as_str().unwrap_or_default().into(),
        bound_session_id: None,
        binding_failed: false,
    })
}

#[tauri::command]
pub async fn setup_netease_connection(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
    restart: bool,
    bind: Option<bool>,
) -> Result<NeteaseConnection, String> {
    let _permit = SETUP.try_acquire().map_err(|_| "NETEASE_SETUP_BUSY")?;
    let preferred_path = state
        .settings
        .lock()
        .ok()
        .and_then(|settings| settings.netease_executable_path.clone());
    let mut result = super::native_task::run_background(move || {
        configure(if restart { "Connect" } else { "Check" }, preferred_path)
    })
    .await
    .map_err(|_| "NETEASE_SETUP_BUSY".to_string())??;
    if result.configured && bind.unwrap_or(restart) {
        super::settings::remember_netease_connection(
            app.clone(),
            &state,
            result.executable_path.clone(),
            true,
        )
        .await
        .map_err(|_| "NETEASE_SETUP_FAILED".to_string())?;
    }
    if result.connected && bind.unwrap_or(restart) {
        let session_id = super::native_task::run_background(|| {
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
        .await
        .map_err(|_| "NETEASE_SETUP_BUSY".to_string())?;
        if let Some(id) = session_id {
            if super::settings::bind_netease_player(app, &state, id.clone())
                .await
                .is_ok()
            {
                result.bound_session_id = Some(id);
            } else {
                result.binding_failed = true;
            }
        }
    }
    Ok(result)
}

pub(crate) fn start_netease_auto_restore(app: tauri::AppHandle) {
    let preferences = app
        .state::<crate::state::AppState>()
        .settings
        .lock()
        .ok()
        .map(|settings| settings.clone());
    let Some(preferences) = preferences.filter(|settings| settings.netease_auto_connect) else {
        return;
    };
    tauri::async_runtime::spawn(async move {
        let Ok(_permit) = SETUP.try_acquire() else {
            return;
        };
        let restored = super::native_task::run_background(move || {
            configure("Restore", preferences.netease_executable_path)
        })
        .await;
        let Ok(Ok(result)) = restored else {
            tracing::debug!("[NetEase] Automatic connection restore deferred");
            return;
        };
        let state = app.state::<crate::state::AppState>();
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
