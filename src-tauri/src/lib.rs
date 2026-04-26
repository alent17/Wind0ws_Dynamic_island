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

use serde::Serialize;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, image::Image,
};

// ============================================================================
// 模块声明
// ============================================================================

mod error;
mod models;
mod state;
mod services;
mod commands;
mod event_bus;

// ============================================================================
// 公开导出
// ============================================================================

pub use error::{AppError, AppResult};
pub use models::{AppSettings, MediaState, CacheStats, NeteaseSong, MonitorInfo};
pub use state::AppState;
pub use services::{read_settings_file, write_settings_file, set_auto_start, get_auto_start};

// ============================================================================
// 外部依赖
// ============================================================================

use event_bus::EVENT_BUS;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{FftPlanner, num_complex::Complex};

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

/// 频谱数据负载
///
/// 发送到前端的音频频谱数据结构
#[derive(Clone, Serialize)]
struct SpectrumPayload {
    /// 简化的频段数据（5 个频段，用于简单可视化）
    bands: Vec<f32>,

    /// 扩展的频段数据（40 个频段，用于详细可视化）
    bands_expanded: Vec<f32>,
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
        eprintln!("[EventBus] 初始化失败: {}", e);
    }

    std::thread::spawn(move || {
        // 初始化 COM 组件（Windows 媒体 API 需要）
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

// ============================================================================
// 音频可视化
// ============================================================================

/// 启动音频可视化器
///
/// 捕获系统音频输出，进行 FFT 分析，生成频谱数据。
///
/// ## 技术实现
///
/// 1. 使用 `cpal` 获取默认音频输出设备
/// 2. 创建音频捕获流（回环捕获）
/// 3. 使用 `rustfft` 进行 FFT 变换
/// 4. 计算对数频谱并归一化
/// 5. 通过 EventBus 发送 `audio-spectrum` 事件
///
/// ## 性能优化
///
/// - FFT 大小：1024 点
/// - 采样降频：每 3 帧发送一次
/// - 节流间隔：50ms（约 20 FPS）
fn start_audio_visualizer(_app: AppHandle) {
    std::thread::spawn(move || {
        // 获取音频主机和设备
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                eprintln!("[音频] 未找到默认音频输出设备");
                return;
            }
        };

        // 获取音频配置
        let config = match device.default_output_config() {
            Ok(c) => c.config(),
            Err(e) => {
                eprintln!("[音频] 获取音频配置失败: {}", e);
                return;
            }
        };

        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0 as f32;

        // FFT 配置
        const FFT_SIZE: usize = 1024;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let mut sample_buffer: Vec<f32> = Vec::with_capacity(FFT_SIZE);
        let mut frame_count: usize = 0;

        // 创建音频捕获流
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                frame_count += 1;

                // 将多声道混合为单声道
                for frame in data.chunks(channels) {
                    let mono = frame.iter().sum::<f32>() / channels as f32;
                    sample_buffer.push(mono);

                    // 当缓冲区满时进行 FFT
                    if sample_buffer.len() == FFT_SIZE {
                        // 应用汉宁窗函数并准备 FFT 输入
                        let mut buffer: Vec<Complex<f32>> = sample_buffer
                            .iter()
                            .enumerate()
                            .map(|(i, &val)| {
                                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE as f32 - 1.0)).cos());
                                Complex { re: val * window, im: 0.0 }
                            })
                            .collect();

                        // 执行 FFT
                        fft.process(&mut buffer);

                        // 计算幅度谱
                        let magnitudes: Vec<f32> = buffer.iter()
                            .take(FFT_SIZE / 2)
                            .map(|c| c.norm())
                            .collect();

                        // 计算频段数据
                        let bands = calculate_bands(&magnitudes, 5, sample_rate, FFT_SIZE);
                        let bands_expanded = calculate_bands(&magnitudes, 40, sample_rate, FFT_SIZE);

                        // 每 3 帧发送一次，降低发送频率
                        if frame_count % 3 == 0 {
                            let _ = event_bus::emit_audio_spectrum(SpectrumPayload {
                                bands,
                                bands_expanded
                            });
                        }

                        // 保留一半数据，实现滑动窗口
                        sample_buffer.drain(0..FFT_SIZE / 2);
                    }
                }
            },
            |err| eprintln!("[音频] 捕获流错误: {}", err),
            None
        );

        // 启动音频流
        match stream {
            Ok(s) => {
                let _ = s.play();
                // 保持线程运行
                loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
            }
            Err(e) => eprintln!("[音频] 构建捕获流失败: {}", e),
        }
    });
}

/// 计算频段数据
///
/// 将 FFT 结果转换为对数频谱，并归一化到 [0, 1] 范围。
///
/// ## 参数
///
/// - `magnitudes`: FFT 幅度谱
/// - `num_bands`: 输出频段数量
/// - `sample_rate`: 采样率
/// - `fft_size`: FFT 大小
///
/// ## 算法
///
/// 1. 使用对数频率刻度划分频段（80Hz - 8000Hz）
/// 2. 对每个频段计算加权平均幅度
/// 3. 转换为分贝值
/// 4. 归一化并应用 gamma 校正
fn calculate_bands(magnitudes: &[f32], num_bands: usize, sample_rate: f32, fft_size: usize) -> Vec<f32> {
    let mut bands = vec![0.0; num_bands];
    let freq_resolution = sample_rate / fft_size as f32;

    // 对数频率范围
    let min_freq: f32 = 80.0;
    let max_freq: f32 = 8000.0;
    let log_min = min_freq.log2();
    let log_max = max_freq.log2();
    let log_step = (log_max - log_min) / num_bands as f32;

    for i in 0..num_bands {
        // 计算频段的频率范围
        let start_freq = 2.0_f32.powf(log_min + i as f32 * log_step);
        let end_freq = 2.0_f32.powf(log_min + (i + 1) as f32 * log_step);

        // 转换为 FFT bin 索引
        let mut start_bin = (start_freq / freq_resolution).round() as usize;
        let mut end_bin = (end_freq / freq_resolution).round() as usize;

        start_bin = start_bin.clamp(1, magnitudes.len() - 1);
        end_bin = end_bin.clamp(start_bin + 1, magnitudes.len());

        // 计算加权平均幅度
        let mut sum = 0.0;
        let mut count = 0.0;
        for j in start_bin..end_bin {
            let freq = j as f32 * freq_resolution;
            // 高频加权，补偿人耳感知
            let weight = (freq / 1000.0).powf(1.15).clamp(0.05, 10.0);
            sum += magnitudes[j] * weight;
            count += 1.0;
        }

        let avg = if count > 0.0 { sum / count } else { 0.0 };

        // 转换为分贝
        let db = if avg > 0.0001 { 20.0 * avg.log10() } else { -100.0 };

        // 归一化到 [0, 1]
        let mut normalized = (db + 35.0) / 50.0;

        // 阈值处理，消除噪声
        if normalized < 0.12 { normalized = 0.0; }

        // Gamma 校正，增强视觉效果
        bands[i] = normalized.clamp(0.0, 1.0).powf(1.8);
    }
    bands
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
fn set_window_vibrancy(window: tauri::Window, enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if enable {
            apply_acrylic(&window, Some((0, 0, 0, 0)))
                .map_err(|e| format!("Failed to apply acrylic: {}", e))?;
        } else {
            clear_acrylic(&window)
                .map_err(|e| format!("Failed to clear acrylic: {}", e))?;
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
        ])
        // Setup 回调
        .setup(|app| {
            // 初始化缓存系统
            if let Err(e) = services::cache::init_cache_system(&app.handle()) {
                eprintln!("[Cache] 初始化失败: {}", e);
            }

            // 加载保存的设置
            let saved_settings = read_settings_file(&app.handle());
            let initial_settings = saved_settings.unwrap_or_else(models::AppSettings::default);

            // 获取目标显示器索引
            let monitor_index = initial_settings.monitor_index;

            // 更新应用状态
            let state = app.state::<state::AppState>();
            if let Ok(mut state_settings) = state.settings.lock() {
                *state_settings = initial_settings.clone();
            }

            // 获取主窗口
            let window = app.get_webview_window("main").unwrap();

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
            window.set_focus().unwrap();

            // 启动后台服务
            start_media_listener(app.handle().clone());
            start_audio_visualizer(app.handle().clone());

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
                    SHOW_MENU_ID => { let _ = commands::show_main_window(app.clone()); }
                    SETTINGS_MENU_ID => { let _ = commands::show_settings_window(app.clone()); }
                    QUIT_MENU_ID => { app.exit(0); }
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
                                if !v { let _ = w.show(); let _ = w.set_focus(); }
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        });

    // 运行应用
    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
