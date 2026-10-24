//! Run for at most 60 seconds with:
//! `cargo run --offline --example c2-media-probe --features c2-media-probe`
//!
//! This builds a windowless Tauri AppHandle and invokes only Isle's production
//! SMTC listener/EventBus path. It does not call the product run/setup flow.

use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
#[link(name = "resource", kind = "static")]
unsafe extern "C" {}

const OBSERVATION_LIMIT: Duration = Duration::from_secs(60);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg != "--no-selected-player") {
        return Err("usage: c2-media-probe [--no-selected-player]".into());
    }
    let no_selected_player = args.iter().any(|arg| arg == "--no-selected-player");

    let mut context = tauri::generate_context!();
    context.config_mut().identifier = "com.isle-app.c2-media-probe".to_string();
    context.config_mut().app.windows.clear();

    let app_state = isle_lib::AppState::default();
    if no_selected_player {
        app_state
            .settings
            .lock()
            .expect("probe AppState settings mutex poisoned")
            .selected_player_ids = Some(Vec::new());
        println!("media_probe_mode selection=empty_allowlist");
    }

    let app = tauri::Builder::default()
        .manage(app_state)
        .build(context)?;

    let task = isle_lib::start_c2_media_probe_listener(app.handle().clone());
    let started = Instant::now();
    while started.elapsed() < OBSERVATION_LIMIT && !task.is_finished() {
        let remaining = OBSERVATION_LIMIT.saturating_sub(started.elapsed());
        std::thread::sleep(remaining.min(Duration::from_millis(250)));
    }

    // Stop is sent through the same bounded channel used by production SMTC
    // callbacks. The worker removes its registered event tokens before join.
    let summary = task.stop_and_join()?;
    app.cleanup_before_exit();
    println!("{summary}");
    Ok(())
}
