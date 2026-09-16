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
            let primary_language = unsafe {
                windows::Win32::Globalization::GetUserDefaultUILanguage() & 0x03ff
            };
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

struct SpectrumState(Mutex<audio::SpectrumCapture>);

#[tauri::command]
fn start_spectrum(state: State<SpectrumState>, app: tauri::AppHandle) -> Result<(), String> {
    let capture = state
        .inner()
        .0
        .lock()
        .map_err(|e| format!("Mutex poisoned: {}", e))?;
    capture.start(app)
}

#[tauri::command]
fn stop_spectrum(state: State<SpectrumState>) -> Result<(), String> {
    let capture = state
        .inner()
        .0
        .lock()
        .map_err(|e| format!("Mutex poisoned: {}", e))?;
    capture.stop();
    Ok(())
}

// ============================================================================
// 媒体监听器
// ============================================================================

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
fn start_media_listener(handle: AppHandle) {
    // 初始化事件总线
    if let Err(e) = EVENT_BUS.initialize(handle.clone()) {
        tracing::error!("[EventBus] 初始化失败: {}", e);
    }

    std::thread::spawn(move || {
        // 初始化 COM 组件（Windows 媒体 API 需要）
        // SAFETY: CoInitializeEx must be called before using Windows COM APIs (media sessions).
        // Using COINIT_MULTITHREADED for thread-safe COM access.
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }

        let mut last_artwork_track = String::new();
        let mut last_artwork = String::new();
        // 持续监听媒体状态
        loop {
            if let Ok(mut info) = services::media::get_media_info(&handle) {
                let artwork_track = format!("{}|{}|{}", info.source, info.title, info.artist);
                if artwork_track == last_artwork_track && info.album_art == last_artwork {
                    // Artwork is often hundreds of KB. Sending the same Base64
                    // payload to every WebView once per second caused release
                    // builds to stutter even though only progress had changed.
                    info.album_art.clear();
                } else {
                    last_artwork_track = artwork_track;
                    last_artwork = info.album_art.clone();
                }
                let _ = event_bus::emit_media_update(info);
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });
}

fn start_system_audio_monitor() {
    std::thread::spawn(move || {
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }
        let mut previous = services::get_system_audio_state().ok();
        loop {
            if let Ok(current) = services::get_system_audio_state() {
                if previous.as_ref() != Some(&current) {
                    let _ = EVENT_BUS.emit(event_bus::EVENT_SYSTEM_AUDIO_CHANGED, &current);
                    previous = Some(current);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    });
}

/// 启动统一捕获状态监控器
///
/// 在后台线程中监控截图快捷键与全屏状态，并向前端发布统一快照。
///
/// ## 工作流程
///
/// 1. 每 50ms 检测截图快捷键
/// 2. 每 500ms 检测一次全屏状态
/// 3. 状态变化时立即发布，并周期重发以同步晚加载的窗口
fn start_capture_monitor(handle: AppHandle) {
    use std::time::{Duration, Instant};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_LWIN, VK_RWIN, VK_S, VK_SHIFT, VK_SNAPSHOT,
    };

    std::thread::spawn(move || {
        let mut previous = (false, false);
        let mut screenshot_until: Option<Instant> = None;
        let mut previous_shortcut_down = false;
        let mut is_fullscreen = false;
        let mut next_fullscreen_check = Instant::now();
        let mut last_emit = Instant::now() - Duration::from_secs(2);

        loop {
            if Instant::now() >= next_fullscreen_check {
                is_fullscreen = detect_fullscreen_app(&handle);
                next_fullscreen_check = Instant::now() + Duration::from_millis(500);
            }
            let shortcut_down = unsafe {
                let print_screen = GetAsyncKeyState(VK_SNAPSHOT.0 as i32) < 0;
                let win = GetAsyncKeyState(VK_LWIN.0 as i32) < 0
                    || GetAsyncKeyState(VK_RWIN.0 as i32) < 0;
                let shift = GetAsyncKeyState(VK_SHIFT.0 as i32) < 0;
                let s = GetAsyncKeyState(VK_S.0 as i32) < 0;
                print_screen || (win && shift && s)
            };
            if shortcut_down && !previous_shortcut_down {
                screenshot_until = Some(Instant::now() + Duration::from_millis(1500));
            }
            previous_shortcut_down = shortcut_down;

            let screenshot = screenshot_until.is_some_and(|until| Instant::now() < until);
            if !screenshot {
                screenshot_until = None;
            }

            let current = (screenshot, is_fullscreen);
            if current != previous || last_emit.elapsed() >= Duration::from_secs(2) {
                let payload = serde_json::json!({
                    "screenshot": screenshot,
                    "recording": false,
                    "fullscreen": is_fullscreen,
                    "screenShare": false,
                });
                if let Err(error) = event_bus::emit_capture_mode_changed(payload) {
                    tracing::error!("[Capture Mode] 发送状态失败: {}", error);
                }
                previous = current;
                last_emit = Instant::now();
            }

            std::thread::sleep(Duration::from_millis(50));
        }
    });
}

fn detect_fullscreen_app(_handle: &AppHandle) -> bool {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
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
///    - 启动音频可视化器
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
        .manage(SpectrumState(Mutex::new(audio::SpectrumCapture::new())))
        // 注册 IPC 命令
        .invoke_handler(tauri::generate_handler![
            // 设置相关命令
            commands::get_preferences,
            commands::save_preferences,
            commands::get_settings,
            commands::save_settings,
            commands::set_theme,
            commands::get_theme,
            commands::set_always_on_top,
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
            commands::toggle_floating_window,
            commands::open_floating_window,
            commands::close_floating_window,
            commands::reset_floating_window,
            commands::sync_window_bounds,
            commands::animate_window_bounds,
            commands::set_island_interaction_region,
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
            commands::search_weather_locations,
            commands::get_netease_song_info_cmd,
            commands::get_netease_mv_url_cmd,
            commands::resolve_hd_cover,
            commands::control_media,
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
            let (show_label, studio_label, quit_label) =
                tray_labels(resolved_ui_language(&initial_settings.language));

            // 获取目标显示器索引
            let monitor_index = initial_settings.monitor_index;

            // 更新应用状态
            let state = app.state::<state::AppState>();
            if let Ok(mut state_settings) = state.settings.lock() {
                *state_settings = initial_settings.clone();
            }

            // 获取主窗口
            let window = match app.get_webview_window("main") {
                Some(w) => w,
                None => {
                    tracing::error!("[Setup] 未找到主窗口 'main'");
                    return Ok(());
                }
            };

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

            if let Err(error) = commands::install_island_cursor_passthrough(&window) {
                tracing::warn!("[Setup] 灵动岛点击穿透初始化失败: {}", error);
            }
            commands::apply_capture_protection(app.handle(), &initial_settings);

            // 设置窗口焦点
            if let Err(e) = window.set_focus() {
                tracing::warn!("[Setup] 设置窗口焦点失败: {}", e);
            }

            // 启动后台服务
            start_media_listener(app.handle().clone());
            start_system_audio_monitor();
            start_capture_monitor(app.handle().clone());

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
                        let _ = commands::show_studio_window(app.clone());
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
