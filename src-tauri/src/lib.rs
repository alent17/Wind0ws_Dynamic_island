use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;

// 菜单项 ID
const SHOW_MENU_ID: &str = "show";
const SETTINGS_MENU_ID: &str = "settings";
const QUIT_MENU_ID: &str = "quit";

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub island_theme: String,
    pub auto_hide: bool,
    pub show_spectrum: bool,
    pub enable_animations: bool,
    pub window_opacity: u8,
    pub always_on_top: bool,
}

#[derive(Default)]
struct MediaCache {
    track_id: String,
    base64_img: String,
}

struct AppState {
    settings: Mutex<AppSettings>,
    media_cache: Mutex<MediaCache>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub album_art: String,
    pub is_playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub last_updated_timestamp: u64,
    pub source: String,
    pub source_display: String,
}

async fn get_media_info_internal(app: &AppHandle) -> Result<MediaState, String> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| format!("RequestAsync failed: {:?}", e))?
        .get()
        .map_err(|e| format!("Await failed: {:?}", e))?;

    let session = manager
        .GetCurrentSession()
        .map_err(|_| "No active media session found".to_string())?;

    let timeline = session
        .GetTimelineProperties()
        .map_err(|e| format!("GetTimelineProperties failed: {:?}", e))?;

    let info = session
        .TryGetMediaPropertiesAsync()
        .map_err(|e| format!("TryGetMediaPropertiesAsync failed: {:?}", e))?
        .get()
        .map_err(|e| format!("Await media properties failed: {:?}", e))?;

    let title = info.Title().unwrap_or_default().to_string();
    let artist = info.Artist().unwrap_or_default().to_string();
    let track_id = format!("{}|{}", title, artist);

    let mut thumbnail_base64 = String::new();
    let mut needs_update = true;

    {
        let state = app.state::<AppState>();
        if let Ok(cache) = state.media_cache.lock() {
            if cache.track_id == track_id && !cache.base64_img.is_empty() {
                thumbnail_base64 = cache.base64_img.clone();
                needs_update = false;
            }
        };
    }

    if needs_update {
        if let Ok(thumbnail_ref) = info.Thumbnail() {
            if let Ok(stream) = thumbnail_ref.OpenReadAsync().map_err(|e| e.to_string())?.get() {
                if let Ok(reader) = DataReader::CreateDataReader(&stream) {
                    let size = stream.Size().unwrap_or(0) as u32;
                    if size > 0 {
                        if let Ok(_) = reader.LoadAsync(size).map_err(|e| e.to_string())?.get() {
                            let mut buffer = vec![0u8; size as usize];
                            if let Ok(_) = reader.ReadBytes(&mut buffer) {
                                thumbnail_base64 = format!(
                                    "data:image/png;base64,{}",
                                    general_purpose::STANDARD.encode(&buffer)
                                );
                            }
                        }
                    }
                }
            }
        }
        
        let state = app.state::<AppState>();
        if let Ok(mut cache) = state.media_cache.lock() {
            cache.track_id = track_id;
            cache.base64_img = thumbnail_base64.clone();
        };
    }

    let playback_status = session
        .GetPlaybackInfo()
        .map_err(|e| format!("GetPlaybackInfo failed: {:?}", e))?
        .PlaybackStatus()
        .unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus(0));

    let is_playing = playback_status.0 == 4;

    let mut dur_ms = (timeline.EndTime().unwrap_or_default().Duration / 10000) as u64;
    let snapshot_pos_ms = (timeline.Position().unwrap_or_default().Duration / 10000) as u64;
    let last_updated_filetime = timeline.LastUpdatedTime().unwrap_or_default().UniversalTime;

    let last_updated_timestamp = if last_updated_filetime > 0 {
        ((last_updated_filetime as i64 / 10000) - 11644473600000) as u64
    } else {
        0
    };

    let now_filetime = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) / 100 + 116444736000000000;
    let diff_ms = if now_filetime > last_updated_filetime {
        ((now_filetime - last_updated_filetime) / 10000) as u64
    } else {
        0
    };

    let real_pos_ms = if is_playing && dur_ms > 0 {
        (snapshot_pos_ms + diff_ms).min(dur_ms)
    } else {
        snapshot_pos_ms
    };

    if dur_ms == 0 && is_playing {
        dur_ms = 1;
    }

    let is_live_logic = dur_ms == 0 || dur_ms > 360000000;
    let (position_ms, duration_ms) = if is_live_logic {
        (0u64, 0u64)
    } else {
        (real_pos_ms, dur_ms)
    };

    let raw_id = session.SourceAppUserModelId().unwrap_or_default().to_string();
    let app_id_lower = raw_id.to_lowercase();

    let source_type = if app_id_lower.contains("cloudmusic") {
        "netease"
    } else if app_id_lower.contains("spotify") {
        "spotify"
    } else if app_id_lower.contains("bilibili") {
        "bilibili"
    } else if app_id_lower.contains("qqmusic") {
        "qqmusic"
    } else if app_id_lower.contains("apple") && app_id_lower.contains("music") {
        "apple"
    } else {
        "generic"
    };

    Ok(MediaState {
        title,
        artist,
        album_art: if thumbnail_base64.is_empty() {
            "https://picsum.photos/400/400?random=1".to_string()
        } else {
            thumbnail_base64
        },
        is_playing,
        position_ms,
        duration_ms,
        last_updated_timestamp,
        source: source_type.to_string(),
        source_display: raw_id,
    })
}

#[tauri::command]
async fn control_media(action: String) -> Result<(), String> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| format!("RequestAsync failed: {:?}", e))?
        .get()
        .map_err(|e| format!("Await failed: {:?}", e))?;

    let session = manager
        .GetCurrentSession()
        .map_err(|_| "No active media session found".to_string())?;

    match action.as_str() {
        "play_pause" => {
            let _ = session.TryTogglePlayPauseAsync().map_err(|e| format!("TogglePlayPause failed: {:?}", e))?;
        }
        "next" => {
            let _ = session.TrySkipNextAsync().map_err(|e| format!("SkipNext failed: {:?}", e))?;
        }
        "prev" => {
            let _ = session.TrySkipPreviousAsync().map_err(|e| format!("SkipPrevious failed: {:?}", e))?;
        }
        _ => return Err(format!("Unknown action: {}", action)),
    }
    Ok(())
}

fn start_media_listener(handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            if let Ok(info) = get_media_info_internal(&handle).await {
                let _ = handle.emit("media-update", info);
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    });
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
fn show_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings-window") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let _ = tauri::WebviewWindowBuilder::new(
        &app,
        "settings-window",
        tauri::WebviewUrl::App("/settings.html".into()),
    )
    .title("Wind0ws Dynamic Island - 设置")
    .inner_size(960.0, 680.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .decorations(false) 
    .transparent(true)
    .build()
    .map_err(|e| format!("创建设置窗口失败: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    Ok(settings.clone())
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut state_settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    *state_settings = settings;
    Ok(())
}

#[tauri::command]
fn check_fullscreen_app() -> Result<bool, String> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect, GWL_STYLE,
        SM_CXSCREEN, SM_CYSCREEN, WS_CAPTION, WS_POPUP, WS_THICKFRAME,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Ok(false);
        }

        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);

            let is_near_fullscreen = width >= screen_width - 10 && height >= screen_height - 10;
            let has_no_border = (style & WS_POPUP.0 as isize) != 0
                && (style & WS_CAPTION.0 as isize) == 0
                && (style & WS_THICKFRAME.0 as isize) == 0;
            let is_at_origin = rect.left <= 0 && rect.top <= 0;
            let is_browser_fullscreen = is_near_fullscreen && is_at_origin;

            let is_fullscreen = has_no_border || is_browser_fullscreen;
            return Ok(is_fullscreen);
        }
        Ok(false)
    }
}

// ========== 终极性能优化：底层原生原子化同步缩放 ==========
#[tauri::command]
fn sync_window_bounds(app: tauri::AppHandle, width: i32, height: i32, x: i32, y: i32) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOZORDER, SWP_NOACTIVATE, HWND_TOP};
    use windows::Win32::Foundation::HWND;

    if let Some(window) = app.get_webview_window("main") {
        if let Ok(hwnd) = window.hwnd() {
            unsafe {
                // 【核心修复】：去除了 SWP_ASYNCWINDOWPOS。
                // 强制 Windows 在同一帧内原子化地同步更新 X 坐标和 Width 宽度。
                // 这样在物理窗口缩小时，绝不会出现哪怕 1 帧的画面错位跳动！
                let _ = SetWindowPos(
                    HWND(hwnd.0 as _),
                    HWND_TOP,
                    x, y, width, height,
                    SWP_NOZORDER | SWP_NOACTIVATE
                );
            }
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            settings: Mutex::new(AppSettings::default()),
            media_cache: Mutex::new(MediaCache::default()),
        })
        .invoke_handler(tauri::generate_handler![
            control_media,
            show_main_window,
            show_settings_window,
            get_settings,
            save_settings,
            check_fullscreen_app,
            sync_window_bounds // <-- 注册新命令
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_focus().unwrap();

            start_media_listener(app.handle().clone());

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, SHOW_MENU_ID, "显示主窗口", true, None::<&str>)?,
                    &MenuItem::with_id(app, SETTINGS_MENU_ID, "设置", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, QUIT_MENU_ID, "退出", true, None::<&str>)?,
                ],
            )?;

            let _ = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    SHOW_MENU_ID => { let _ = show_main_window(app.clone()); }
                    SETTINGS_MENU_ID => { let _ = show_settings_window(app.clone()); }
                    QUIT_MENU_ID => { app.exit(0); }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if let Ok(is_visible) = window.is_visible() {
                                if !is_visible {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}