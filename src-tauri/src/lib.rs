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
pub use models::{AppSettings, CacheStats, MediaState, MonitorInfo, NeteaseSong};
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
const SETTINGS_MENU_ID: &str = "settings";

/// 托盘菜单 - 退出应用
const QUIT_MENU_ID: &str = "quit";

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

        // 持续监听媒体状态
        loop {
            if let Ok(info) = services::media::get_media_info(&handle) {
                let _ = event_bus::emit_media_update(info);
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });
}

/// 启动全屏状态监控器
///
/// 在后台线程中持续监控全屏状态，
/// 只有状态改变时才通知前端。
///
/// ## 工作流程
///
/// 1. 每 500ms 检测一次全屏状态
/// 2. 只有状态变化时才发送事件
/// 3. 避免高频轮询造成的性能浪费
fn start_fullscreen_monitor(handle: AppHandle) {
    std::thread::spawn(move || {
        let mut was_fullscreen = false;

        loop {
            // 获取当前显示器和窗口信息
            if let Some(window) = handle.get_webview_window("main") {
                if let Ok(all_monitors) = window.available_monitors() {
                    if let Ok(current_monitor) = window.current_monitor() {
                        // 获取当前窗口所在的显示器
                        let target_monitor =
                            current_monitor.or_else(|| all_monitors.first().cloned());

                        if let Some(monitor) = target_monitor {
                            let position = monitor.position();
                            let size = monitor.size();

                            // 调用全屏检测逻辑
                            // SAFETY: Win32 API calls for fullscreen detection. All FFI calls check return values
                            // and handle null/zero cases safely.
                            let is_fullscreen = unsafe {
                                use windows::Win32::Foundation::RECT;
                                use windows::Win32::UI::WindowsAndMessaging::{
                                    GetForegroundWindow, GetWindowLongPtrW, GetWindowRect,
                                    GWL_STYLE, WS_CAPTION,
                                };

                                let hwnd = GetForegroundWindow();
                                if hwnd.0 != 0 {
                                    let mut rect = RECT::default();
                                    if GetWindowRect(hwnd, &mut rect).is_ok() {
                                        let width = rect.right - rect.left;
                                        let height = rect.bottom - rect.top;

                                        if width > 0 && height > 0 {
                                            let is_on_target_monitor = rect.left
                                                <= position.x + size.width as i32
                                                && rect.right >= position.x
                                                && rect.top <= position.y + size.height as i32
                                                && rect.bottom >= position.y;

                                            if is_on_target_monitor {
                                                let covers_screen = width >= size.width as i32 - 2
                                                    && height >= size.height as i32 - 2;

                                                if covers_screen {
                                                    let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
                                                    (style & WS_CAPTION.0 as isize) == 0
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            };

                            // 只有状态变化时才发送事件
                            if is_fullscreen != was_fullscreen {
                                if let Err(e) = event_bus::emit_fullscreen_changed(is_fullscreen) {
                                    tracing::error!("[全屏监控] 发送事件失败：{}", e);
                                } else {
                                    tracing::info!(
                                        "[全屏监控] 状态变化：{} -> {}",
                                        if was_fullscreen {
                                            "全屏"
                                        } else {
                                            "非全屏"
                                        },
                                        if is_fullscreen { "全屏" } else { "非全屏" }
                                    );
                                }
                                was_fullscreen = is_fullscreen;
                            }
                        }
                    }
                }
            }

            // 每 500ms 检测一次
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
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
            commands::show_settings_window,
            commands::toggle_floating_window,
            commands::open_floating_window,
            commands::close_floating_window,
            commands::sync_window_bounds,
            commands::set_floating_window_resizable,
            commands::open_application,
            commands::check_fullscreen_app,
            commands::get_available_monitors,
            commands::get_current_monitor_index,
            commands::set_current_monitor_index,
            commands::set_hide_monitor_selector,
            commands::set_hide_floating_window,
            commands::set_expanded_corner_radius,
            // 媒体相关命令
            commands::get_media_info_cmd,
            commands::get_netease_song_info_cmd,
            commands::get_netease_mv_url_cmd,
            commands::control_media,
            commands::extract_dominant_color,
            commands::process_image,
            commands::pixelate_cover,
            commands::set_hide_settings_button,
            // 缓存相关命令
            commands::clear_cache,
            commands::get_cache_stats,
            commands::get_cache_directory,
            commands::set_cache_directory,
            commands::pick_cache_directory,
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
                    // 计算居中位置
                    let x = position.x + (size.width as i32 / 2) - 190;
                    let y = position.y + 20;
                    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
                }
            }

            // 设置窗口焦点
            if let Err(e) = window.set_focus() {
                tracing::warn!("[Setup] 设置窗口焦点失败: {}", e);
            }

            // 启动后台服务
            start_media_listener(app.handle().clone());
            start_fullscreen_monitor(app.handle().clone());

            // 创建托盘菜单
            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, SHOW_MENU_ID, "显示主窗口", true, None::<&str>)?,
                    &MenuItem::with_id(app, SETTINGS_MENU_ID, "设置", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, QUIT_MENU_ID, "退出", true, None::<&str>)?,
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
                    SETTINGS_MENU_ID => {
                        let _ = commands::show_settings_window(app.clone());
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
