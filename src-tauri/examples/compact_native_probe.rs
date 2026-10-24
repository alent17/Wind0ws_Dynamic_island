//! cargo run --features compact-native-probe --example compact_native_probe -- NEW_OUTPUT [floating|edge] [top|right|bottom|left]
#[cfg(not(feature = "compact-native-probe"))]
fn main() {
    eprintln!("requires --features compact-native-probe");
}

#[cfg(feature = "compact-native-probe")]
mod probe {
    use std::{
        fs::OpenOptions,
        io::Write,
        path::PathBuf,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    #[link(name = "resource", kind = "static")]
    unsafe extern "C" {}
    fn record(path: &std::path::Path, text: &str) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(
                file,
                "{}\t{}\t{}",
                chrono::Utc::now().to_rfc3339(),
                std::process::id(),
                text
            );
        }
    }
    struct ProbeState {
        log: PathBuf,
        applied: Arc<AtomicBool>,
    }
    #[tauri::command]
    async fn apply_compact(
        window: tauri::WebviewWindow,
        region: serde_json::Value,
        state: tauri::State<'_, ProbeState>,
    ) -> Result<serde_json::Value, String> {
        let log = state.log.clone();
        let applied = state.applied.clone();
        let target = window.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        window
            .run_on_main_thread(move || {
                let result = isle_lib::apply_compact_probe_region(&target, region);
                record(&log, &format!("apply {result:?}"));
                let result = result.and_then(|receipt| {
                    // Begin accepting input only after the real SetWindowRgn success.
                    applied.store(true, Ordering::Release);
                    target.show().map_err(|e| e.to_string())?;
                    Ok(receipt)
                });
                let _ = tx.send(result);
            })
            .map_err(|e| e.to_string())?;
        rx.await.map_err(|e| e.to_string())?
    }
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<_> = std::env::args().collect();
        let output = PathBuf::from(args.get(1).ok_or("new output directory required")?);
        std::fs::create_dir(&output)?;
        let output = output.canonicalize()?;
        let log = output.join("events.tsv");
        let style = args.get(2).map(String::as_str).unwrap_or("floating");
        let edge = args.get(3).map(String::as_str).unwrap_or("top");
        if !["floating", "edge"].contains(&style)
            || !["top", "right", "bottom", "left"].contains(&edge)
        {
            return Err("invalid style/edge".into());
        }
        let page = format!("ui-tests/compact-native-fixture.html?style={style}&edge={edge}");
        let mut context = tauri::generate_context!();
        context.config_mut().app.windows.clear();
        context.config_mut().build.dev_url = Some("http://127.0.0.1:1426".parse()?);
        context.config_mut().identifier = "com.isle-app.compact-native-probe".into();
        let applied = Arc::new(AtomicBool::new(false));
        let event_log = log.clone();
        let title_log = log.clone();
        let applied_event = applied.clone();
        let app = tauri::Builder::default()
            .manage(ProbeState {
                log: log.clone(),
                applied,
            })
            .invoke_handler(tauri::generate_handler![apply_compact])
            .setup(move |app| {
                let window = WebviewWindowBuilder::new(
                    app,
                    "compact-native-probe",
                    WebviewUrl::App(page.into()),
                )
                .title("Compact native loading")
                .inner_size(640.0, 360.0)
                .position(80.0, 100.0)
                .decorations(false)
                .shadow(false)
                .transparent(true)
                .always_on_top(true)
                .resizable(false)
                .visible(false)
                .data_directory(output.join("webview-profile"))
                .on_document_title_changed(move |_, title| record(&title_log, &title))
                .build()?;
                let invalidation_window = window.clone();
                window.on_window_event(move |event| {
                    if matches!(
                        event,
                        tauri::WindowEvent::Moved(_)
                            | tauri::WindowEvent::Resized(_)
                            | tauri::WindowEvent::ScaleFactorChanged { .. }
                    ) && applied_event.load(Ordering::Acquire)
                    {
                        record(&event_log, "invalidated host frame changed");
                        let _ = invalidation_window.hide();
                    }
                });
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(90));
                    handle.exit(0);
                });
                Ok(())
            })
            .build(context)?;
        app.run(move |_, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                record(&log, "exit");
            }
        });
        Ok(())
    }
}
#[cfg(feature = "compact-native-probe")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    probe::run()
}
