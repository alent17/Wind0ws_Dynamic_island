use tauri::{AppHandle, Manager};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::services::{write_settings_file};
use crate::event_bus::EVENT_BUS;

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| AppError::window(e.to_string()))?;
        window.set_focus().map_err(|e| AppError::window(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_settings_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("settings-window") {
        let _ = window.set_size(tauri::PhysicalSize::new(1000, 750));
        let _ = window.set_min_size(Some(tauri::PhysicalSize::new(800, 600)));
        let _ = window.center();
        window.show().map_err(|e| AppError::window(e.to_string()))?;
        window.set_focus().map_err(|e| AppError::window(e.to_string()))?;
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        &app,
        "settings-window",
        tauri::WebviewUrl::App("settings.html".into()),
    )
    .title("Isle - 设置")
    .inner_size(1000.0, 750.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .decorations(false)
    .transparent(false)
    .build()
    .map_err(|e| AppError::window(format!("创建设置窗口失败: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub fn toggle_floating_window(_app: AppHandle) -> AppResult<()> {
    Ok(())
}

#[tauri::command]
pub async fn open_floating_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("floating_player") {
        window.show().map_err(|e| AppError::window(e.to_string()))?;
        window.set_focus().map_err(|e| AppError::window(e.to_string()))?;
        return Ok(());
    }

    let state = app.state::<AppState>();
    let saved_position = {
        let settings = state
            .settings
            .lock()
            .map_err(|_| AppError::lock("Failed to lock settings"))?;
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

    if let (Some(x), Some(y), Some(w), Some(h)) = saved_position {
        builder = builder.inner_size(w as f64, h as f64);
        builder = builder.position(x as f64, y as f64);
    } else {
        builder = builder.inner_size(360.0, 360.0);
    }

    builder
        .build()
        .map_err(|e| AppError::window(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn close_floating_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("floating_player") {
        window.close().map_err(|e| AppError::window(e.to_string()))?;
        let _ = EVENT_BUS.emit(crate::event_bus::EVENT_FLOATING_WINDOW_CLOSED, ());
    }
    Ok(())
}

#[tauri::command]
pub fn sync_window_bounds(
    app: AppHandle,
    width: i32,
    height: i32,
    x: i32,
    y: i32,
) -> AppResult<()> {
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
pub fn set_floating_window_resizable(app: AppHandle, resizable: bool) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("floating_player") {
        window
            .set_resizable(resizable)
            .map_err(|e| AppError::window(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_application(name: String) -> AppResult<()> {
    use std::process::Command;
    
    let protocol = match name.as_str() {
        "NeteaseCloudMusic" => "orpheus://",
        "Spotify" => "spotify:",
        "Bilibili" => "bilibili://",
        "QQMusic" => "qqmusic://",
        "AppleMusic" => "https://music.apple.com",
        _ => "",
    };
    
    if !protocol.is_empty() {
        let output = Command::new("cmd")
            .args(["/C", "start", "", protocol])
            .output();
        
        if let Ok(out) = output {
            if out.status.success() {
                return Ok(());
            }
        }
    }
    
    let common_paths: Vec<&str> = match name.as_str() {
        "Spotify" => vec![
            r"%APPDATA%\Spotify\Spotify.exe",
            r"%LOCALAPPDATA%\Spotify\Spotify.exe",
        ],
        "NeteaseCloudMusic" => vec![
            r"%LOCALAPPDATA%\Netease\CloudMusic\cloudmusic.exe",
            r"%PROGRAMFILES%\Netease\CloudMusic\cloudmusic.exe",
            r"%PROGRAMFILES(X86)%\Netease\CloudMusic\cloudmusic.exe",
        ],
        "QQMusic" => vec![
            r"%PROGRAMFILES%\Tencent\QQMusic\QQMusic.exe",
            r"%PROGRAMFILES(X86)%\Tencent\QQMusic\QQMusic.exe",
        ],
        "Bilibili" => vec![
            r"%LOCALAPPDATA%\Programs\bilibili\bilibili.exe",
            r"%PROGRAMFILES%\Bilibili\bilibili.exe",
        ],
        "AppleMusic" => vec![
            r"%LOCALAPPDATA%\Microsoft\WindowsApps\AppleMusic.exe",
            r"%PROGRAMFILES%\WindowsApps\AppleMusic.exe",
        ],
        _ => vec![],
    };
    
    for path_template in &common_paths {
        let expanded = path_template
            .replace("%APPDATA%", &std::env::var("APPDATA").unwrap_or_default())
            .replace("%LOCALAPPDATA%", &std::env::var("LOCALAPPDATA").unwrap_or_default())
            .replace("%PROGRAMFILES%", &std::env::var("ProgramFiles").unwrap_or_default())
            .replace("%PROGRAMFILES(X86)%", &std::env::var("ProgramFiles(x86)").unwrap_or_default());
        
        if std::path::Path::new(&expanded).exists() {
            let output = Command::new("cmd")
                .args(["/C", "start", "", &expanded])
                .output();
            
            if let Ok(out) = output {
                if out.status.success() {
                    return Ok(());
                }
            }
        }
    }
    
    Err(AppError::business(4001, format!("无法打开应用: {}", name)))
}

#[tauri::command]
pub fn check_fullscreen_app(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: i32,
    monitor_height: i32,
) -> AppResult<bool> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetClassNameW,
        GWL_STYLE, WS_CAPTION,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Ok(false);
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return Ok(false);
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        if width <= 0 || height <= 0 {
            return Ok(false);
        }

        let is_on_target_monitor =
            rect.left <= monitor_x + monitor_width &&
            rect.right >= monitor_x &&
            rect.top <= monitor_y + monitor_height &&
            rect.bottom >= monitor_y;

        if !is_on_target_monitor {
            return Ok(false);
        }

        let covers_screen = width >= monitor_width - 2 && height >= monitor_height - 2;

        if !covers_screen {
            return Ok(false);
        }

        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let no_caption = (style & WS_CAPTION.0 as isize) == 0;

        return Ok(no_caption);
    }
}

#[tauri::command]
pub async fn get_available_monitors(app: AppHandle) -> AppResult<Vec<String>> {
    let window = app
        .get_webview_window("main")
        .or_else(|| app.get_webview_window("floating"))
        .ok_or_else(|| AppError::window("Failed to get window"))?;
    
    let monitors = window
        .available_monitors()
        .map_err(|e| AppError::window(format!("Failed to get monitors: {}", e)))?;
    
    let monitor_names: Vec<String> = monitors
        .iter()
        .enumerate()
        .map(|(idx, m)| {
            m.name()
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("显示器 {}", idx + 1))
        })
        .collect();
    
    Ok(monitor_names)
}

#[tauri::command]
pub async fn get_current_monitor_index(
    state: tauri::State<'_, AppState>,
) -> AppResult<u32> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    Ok(settings.monitor_index)
}

#[tauri::command]
pub fn set_current_monitor_index(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    index: u32,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.monitor_index = index;

    write_settings_file(&app, &settings)?;

    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_SETTINGS_CHANGED, "monitor_index");

    Ok(())
}

#[tauri::command]
pub fn set_hide_monitor_selector(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    enable: bool,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.hide_monitor_selector = enable;

    write_settings_file(&app, &settings)?;

    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_SETTINGS_CHANGED, "hide_monitor_selector");

    Ok(())
}

#[tauri::command]
pub fn set_hide_floating_window(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    enable: bool,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.hide_floating_window = enable;

    write_settings_file(&app, &settings)?;

    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_SETTINGS_CHANGED, "hide_floating_window");

    Ok(())
}

#[tauri::command]
pub fn set_expanded_corner_radius(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    radius: u32,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.expanded_corner_radius = radius.min(32);

    write_settings_file(&app, &settings)?;

    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_CORNER_RADIUS_CHANGED, settings.expanded_corner_radius);

    Ok(())
}
