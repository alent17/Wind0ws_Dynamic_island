use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Duration;
use std::fs;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, image::Image,
};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;

const SHOW_MENU_ID: &str = "show";
const SETTINGS_MENU_ID: &str = "settings";
const QUIT_MENU_ID: &str = "quit";

#[derive(Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub island_theme: String,
    pub auto_hide: bool,
    pub show_spectrum: bool,
    pub enable_animations: bool,
    pub window_opacity: u8,
    pub always_on_top: bool,
    // ===== 高级设置 =====
    pub hardware_acceleration: bool,
    pub reduce_animations: bool,
    pub show_debug_info: bool,
    pub log_level: String,
    // ===== 显示器设置 =====
    pub monitor_index: u32,
    // ===== 播放器权重 =====
    pub player_weights: std::collections::HashMap<String, u32>,
    // ===== 悬浮窗位置和大小 =====
    pub floating_window_x: Option<i32>,
    pub floating_window_y: Option<i32>,
    pub floating_window_width: Option<u32>,
    pub floating_window_height: Option<u32>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let mut player_weights = std::collections::HashMap::new();
        // 默认权重：数值越大优先级越高
        player_weights.insert("netease".to_string(), 50);
        player_weights.insert("spotify".to_string(), 50);
        player_weights.insert("bilibili".to_string(), 50);
        player_weights.insert("qqmusic".to_string(), 50);
        player_weights.insert("apple".to_string(), 50);
        player_weights.insert("generic".to_string(), 10);
        
        Self {
            island_theme: "original".to_string(),
            auto_hide: true,
            show_spectrum: true,
            enable_animations: true,
            window_opacity: 255,
            always_on_top: true,
            hardware_acceleration: true,
            reduce_animations: false,
            show_debug_info: false,
            log_level: "Info".to_string(),
            monitor_index: 0, // 默认使用第一个显示器
            player_weights,
            floating_window_x: None,
            floating_window_y: None,
            floating_window_width: None,
            floating_window_height: None,
        }
    }
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

// 读取配置文件
fn read_settings_file(app: &AppHandle) -> Option<AppSettings> {
    let config_dir = app.path().app_data_dir().ok()?;
    let config_path = config_dir.join("settings.json");
    
    if !config_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(config_path).ok()?;
    serde_json::from_str(&content).ok()
}

// 保存配置文件
fn write_settings_file(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let config_dir = app.path().app_data_dir()
        .map_err(|e| format!("获取配置目录失败：{}", e))?;
    
    // 确保目录存在
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("创建配置目录失败：{}", e))?;
    }
    
    let config_path = config_dir.join("settings.json");
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("序列化配置失败：{}", e))?;
    
    fs::write(config_path, content)
        .map_err(|e| format!("写入配置文件失败：{}", e))
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

    // 获取所有会话
    let sessions = manager
        .GetSessions()
        .map_err(|e| format!("GetSessions failed: {:?}", e))?;
    
    let session_count = sessions.Size().unwrap_or(0) as usize;
    
    if session_count == 0 {
        return Err("No active media session found".to_string());
    }

    // 获取权重设置
    let weights = {
        let state = app.state::<AppState>();
        let settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
        settings.player_weights.clone()
    };

    // 根据权重选择会话：始终选择权重最高的，无论是否在播放
    let mut best_session: Option<(GlobalSystemMediaTransportControlsSession, String, String)> = None;
    let mut best_weight = 0u32;

    for i in 0..session_count {
        if let Ok(session) = sessions.GetAt(i as u32) {
            let raw_id: String = session
                .SourceAppUserModelId()
                .unwrap_or_default()
                .to_string();
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

            let weight = weights.get(&source_type[..]).copied().unwrap_or(10);

            // 选择权重最高的会话（权重相同时选择第一个遇到的）
            if weight > best_weight {
                best_weight = weight;
                best_session = Some((session, source_type.to_string(), raw_id));
            }
        }
    }

    let (session, source_type, raw_id) = best_session
        .ok_or_else(|| "No valid session found".to_string())?;

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
            if let Ok(stream) = thumbnail_ref
                .OpenReadAsync()
                .map_err(|e| e.to_string())?
                .get()
            {
                if let Ok(reader) = DataReader::CreateDataReader(&stream) {
                    let size = stream.Size().unwrap_or(0) as u32;
                    if size > 0 {
                        if let Ok(_) =
                            reader.LoadAsync(size).map_err(|e| e.to_string())?.get()
                        {
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

    let mut dur_ms =
        (timeline.EndTime().unwrap_or_default().Duration / 10000) as u64;
    let snapshot_pos_ms =
        (timeline.Position().unwrap_or_default().Duration / 10000) as u64;
    let last_updated_filetime =
        timeline.LastUpdatedTime().unwrap_or_default().UniversalTime;

    let last_updated_timestamp = if last_updated_filetime > 0 {
        ((last_updated_filetime as i64 / 10000) - 11644473600000) as u64
    } else {
        0
    };

    let now_filetime = chrono::Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or(0) / 100
        + 116444736000000000;
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
async fn control_media(app: AppHandle, action: String) -> Result<(), String> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| format!("RequestAsync failed: {:?}", e))?
        .get()
        .map_err(|e| format!("Await failed: {:?}", e))?;

    // 获取所有会话
    let sessions = manager
        .GetSessions()
        .map_err(|e| format!("GetSessions failed: {:?}", e))?;
    
    let session_count = sessions.Size().unwrap_or(0) as usize;
    
    if session_count == 0 {
        return Err("No active media session found".to_string());
    }

    // 获取权重设置
    let weights = {
        let state = app.state::<AppState>();
        let settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
        settings.player_weights.clone()
    };

    // 根据权重选择会话：与 get_media_info_internal 使用相同逻辑
    let mut best_session: Option<GlobalSystemMediaTransportControlsSession> = None;
    let mut best_weight = 0u32;

    for i in 0..session_count {
        if let Ok(session) = sessions.GetAt(i as u32) {
            let raw_id: String = session
                .SourceAppUserModelId()
                .unwrap_or_default()
                .to_string();
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

            let weight = weights.get(&source_type[..]).copied().unwrap_or(10);

            // 选择权重最高的会话
            if weight > best_weight {
                best_weight = weight;
                best_session = Some(session);
            }
        }
    }

    let session = best_session
        .ok_or_else(|| "No valid session found".to_string())?;

    match action.as_str() {
        "play_pause" => {
            let _ = session
                .TryTogglePlayPauseAsync()
                .map_err(|e| format!("TogglePlayPause failed: {:?}", e))?;
        }
        "next" => {
            let _ = session
                .TrySkipNextAsync()
                .map_err(|e| format!("SkipNext failed: {:?}", e))?;
        }
        "prev" => {
            let _ = session
                .TrySkipPreviousAsync()
                .map_err(|e| format!("SkipPrevious failed: {:?}", e))?;
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
    let settings = state
        .settings
        .lock()
        .map_err(|_| "Failed to lock settings")?;
    Ok(settings.clone())
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let always_on_top = settings.always_on_top;
    let state = app.state::<AppState>();
    {
        let mut state_settings = state
            .settings
            .lock()
            .map_err(|_| "Failed to lock settings")?;
        *state_settings = settings.clone();
    }

    // 保存到配置文件
    write_settings_file(&app, &settings)?;

    // 应用置顶
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(always_on_top);
    }

    // ===== 关键：广播设置变更事件给所有窗口 =====
    let _ = app.emit("settings-updated", settings);

    Ok(())
}

#[tauri::command]
fn check_fullscreen_app(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: i32,
    monitor_height: i32,
) -> Result<bool, String> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowLongPtrW, GetWindowRect,
        GWL_STYLE, WS_CAPTION, WS_POPUP, WS_THICKFRAME,
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
            
            // 检查窗口是否在目标显示器上
            let window_center_x = rect.left + width / 2;
            let window_center_y = rect.top + height / 2;
            
            // 检查窗口中心点是否在目标显示器范围内
            let is_on_target_monitor = 
                window_center_x >= monitor_x && 
                window_center_x < monitor_x + monitor_width &&
                window_center_y >= monitor_y && 
                window_center_y < monitor_y + monitor_height;
            
            if !is_on_target_monitor {
                return Ok(false);
            }
            
            // 检查窗口是否占满目标显示器
            let near_fs = width >= monitor_width - 10 && height >= monitor_height - 10;
            let no_border = (style & WS_POPUP.0 as isize) != 0
                && (style & WS_CAPTION.0 as isize) == 0
                && (style & WS_THICKFRAME.0 as isize) == 0;
            let at_monitor_origin = rect.left <= monitor_x && rect.top <= monitor_y;
            
            return Ok(no_border || (near_fs && at_monitor_origin));
        }
        Ok(false)
    }
}

#[tauri::command]
async fn open_floating_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating_player") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    // 获取保存的位置和大小
    let saved_position = {
        let state = app.state::<AppState>();
        let settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
        (
            settings.floating_window_x,
            settings.floating_window_y,
            settings.floating_window_width,
            settings.floating_window_height,
        )
    };

    let mut builder = tauri::WebviewWindowBuilder::new(
        &app,
        "floating_player",
        tauri::WebviewUrl::App("index.html?window=floating".into()),
    )
    .title("Mini Player")
    .min_inner_size(200.0, 200.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .always_on_top(true);

    // 如果有保存的位置和大小，使用它们
    if let (Some(x), Some(y), Some(w), Some(h)) = saved_position {
        builder = builder.inner_size(w as f64, h as f64);
        builder = builder.position(x as f64, y as f64);
    } else {
        // 否则使用默认大小和位置
        builder = builder.inner_size(360.0, 360.0);
    }

    let _window = builder
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn sync_window_bounds(
    app: tauri::AppHandle,
    width: i32,
    height: i32,
    x: i32,
    y: i32,
) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOZORDER,
    };

    if let Some(window) = app.get_webview_window("main") {
        if let Ok(hwnd) = window.hwnd() {
            unsafe {
                let _ = SetWindowPos(
                    HWND(hwnd.0 as _),
                    HWND_TOP,
                    x, y, width, height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn get_player_weights(app: AppHandle) -> Result<std::collections::HashMap<String, u32>, String> {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    Ok(settings.player_weights.clone())
}

#[tauri::command]
fn emit_event(app: AppHandle, event: String, payload: serde_json::Value) -> Result<(), String> {
    app.emit(&event, payload).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_player_weight(
    app: AppHandle,
    player: String,
    weight: u32,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    settings.player_weights.insert(player, weight);
    
    // 触发设置更新事件
    let _ = app.emit("settings-updated", &*settings);
    
    // 保存到配置文件
    write_settings_file(&app, &settings)?;
    
    Ok(())
}

#[tauri::command]
fn save_floating_window_position(
    app: AppHandle,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    
    settings.floating_window_x = Some(x);
    settings.floating_window_y = Some(y);
    settings.floating_window_width = Some(width);
    settings.floating_window_height = Some(height);
    
    // 保存到配置文件
    write_settings_file(&app, &settings)?;
    
    Ok(())
}

#[tauri::command]
fn get_floating_window_position(app: AppHandle) -> Result<Option<(i32, i32, u32, u32)>, String> {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    
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
            open_floating_window,
            get_settings,
            save_settings,
            check_fullscreen_app,
            sync_window_bounds,
            get_player_weights,
            set_player_weight,
            emit_event,
            save_floating_window_position,
            get_floating_window_position,
        ])
        .setup(|app| {
            // 从配置文件读取设置
            let saved_settings = read_settings_file(&app.handle());
            let initial_settings = saved_settings.unwrap_or_else(AppSettings::default);
            
            // 更新应用状态
            let state = app.state::<AppState>();
            let mut state_settings = state
                .settings
                .lock()
                .map_err(|_| "Failed to lock settings")?;
            *state_settings = initial_settings;
            
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

            // 加载自定义托盘图标 - 使用 include_bytes 嵌入图标数据
            let icon_bytes = include_bytes!("../icons/256x256.png");
            let icon = Image::from_bytes(icon_bytes).map_err(|e| e.to_string())?;
            
            let _ = TrayIconBuilder::new()
                .icon(icon)
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
                        if let Some(w) = app.get_webview_window("main") {
                            if let Ok(v) = w.is_visible() {
                                if !v { let _ = w.show(); let _ = w.set_focus(); }
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
