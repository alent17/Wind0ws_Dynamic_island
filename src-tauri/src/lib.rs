use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, tray::{TrayIconBuilder, TrayIconEvent, MouseButtonState}, menu::{Menu, MenuItem, PredefinedMenuItem}};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;
use std::time::Duration;

// 菜单项 ID
const SHOW_MENU_ID: &str = "show";
const QUIT_MENU_ID: &str = "quit";

#[derive(Clone, Serialize, Deserialize)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub album_art: String,
    pub is_playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub last_updated_timestamp: u64, // 快照产生的 Unix 毫秒时间戳
    pub source: String, // 播放器来源：netease, spotify, bilibili, generic
    pub source_display: String, // 原始 AppUserModelId
}

// 内部异步函数获取媒体信息
async fn get_media_info_internal() -> Result<MediaState, String> {
    // 使用阻塞方式请求会话管理器（在后台线程中执行）
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| format!("RequestAsync failed: {:?}", e))?
        .get()
        .map_err(|e| format!("Await failed: {:?}", e))?;

    // 获取当前会话
    let session = manager
        .GetCurrentSession()
        .map_err(|_| "No active media session found".to_string())?;

    // 获取时间线属性
    let timeline = session
        .GetTimelineProperties()
        .map_err(|e| format!("GetTimelineProperties failed: {:?}", e))?;

    // 获取媒体属性
    let info = session
        .TryGetMediaPropertiesAsync()
        .map_err(|e| format!("TryGetMediaPropertiesAsync failed: {:?}", e))?
        .get()
        .map_err(|e| format!("Await media properties failed: {:?}", e))?;

    // 处理封面图（转换为 Base64）
    let mut thumbnail_base64 = String::new();
    if let Ok(thumbnail_ref) = info.Thumbnail() {
        if let Ok(stream) = thumbnail_ref
            .OpenReadAsync()
            .map_err(|e| e.to_string())?
            .get()
        {
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

    // 获取播放状态
    let playback_status = session
        .GetPlaybackInfo()
        .map_err(|e| format!("GetPlaybackInfo failed: {:?}", e))?
        .PlaybackStatus()
        .unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus(0));

    let is_playing = playback_status.0 == 4; // 4 = Playing

    // 获取进度（转换为毫秒）
    // 1. 获取总时长 (Ticks -> ms)
    let mut dur_ms = (timeline.EndTime().unwrap_or_default().Duration / 10000) as u64;
    
    // 2. 获取快照位置
    let snapshot_pos_ms = (timeline.Position().unwrap_or_default().Duration / 10000) as u64;
    
    // 3. 获取快照产生时的系统时间 (关键！)
    let last_updated_filetime = timeline.LastUpdatedTime().unwrap_or_default().UniversalTime;
    
    // 4. 转换为 Unix 毫秒时间戳 (从 1970-01-01 开始)
    // Windows UniversalTime 是从 1601-01-01 开始的 100ns 间隔
    let last_updated_timestamp = if last_updated_filetime > 0 {
        ((last_updated_filetime as i64 / 10000) - 11644473600000) as u64
    } else {
        0
    };
    
    // 5. 计算从"快照产生"到"现在"过了多久
    let now_filetime = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) / 100 + 116444736000000000;
    let diff_ms = if now_filetime > last_updated_filetime {
        ((now_filetime - last_updated_filetime) / 10000) as u64
    } else {
        0
    };

    // 6. 真实的当前位置 = 快照位置 + 物理流逝的时间 (如果正在播放)
    let real_pos_ms = if is_playing && dur_ms > 0 {
        (snapshot_pos_ms + diff_ms).min(dur_ms)
    } else {
        snapshot_pos_ms
    };
    
    // 【重要：网易云修正逻辑】
    // 如果获取到的是 0，但状态是 Playing，说明是网易云的"懒加载"
    if dur_ms == 0 && is_playing {
        dur_ms = 1;
    }

    // 【重新修正判定】
    // 很多直播流会返回 i64::MAX 或者 0。
    // 我们把上限调高到 100 小时 (360,000,000 ms)，下限保持在 0。
    let is_live_logic = dur_ms == 0 || dur_ms > 360000000;

    let (position_ms, duration_ms) = if is_live_logic {
        (0u64, 0u64)
    } else {
        (real_pos_ms, dur_ms)
    };

    // 获取歌曲标题和艺术家
    let title = info.Title().unwrap_or_default().to_string();
    let artist = info.Artist().unwrap_or_default().to_string();

    // 【核心：智能识别播放器来源】
    // 获取 Windows AppUserModelId 来识别是哪个播放器
    let raw_id = session.SourceAppUserModelId().unwrap_or_default().to_string();
    
    // 转换为小写进行匹配，确保识别准确
    let app_id_lower = raw_id.to_lowercase();
    
    // 根据 AppUserModelId 识别播放器类型
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
        title: title.clone(),
        artist: artist.clone(),
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

// 媒体控制命令
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
        },
        "next" => {
            let _ = session.TrySkipNextAsync().map_err(|e| format!("SkipNext failed: {:?}", e))?;
        },
        "prev" => {
            let _ = session.TrySkipPreviousAsync().map_err(|e| format!("SkipPrevious failed: {:?}", e))?;
        },
        _ => return Err(format!("Unknown action: {}", action)),
    }
    Ok(())
}

// 启动后台监听线程
fn start_media_listener(handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // 在后台线程获取信息，不阻塞主 UI
            if let Ok(info) = get_media_info_internal().await {
                // 发送事件给前端
                let _ = handle.emit("media-update", info);
            }
            // 每秒检查一次即可，不需要太快
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    });
}

// 显示主窗口
#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![control_media])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_focus().unwrap();

            // 启动后台监听线程
            start_media_listener(app.handle().clone());

            // 创建托盘菜单
            let menu = Menu::with_items(app, &[
                &MenuItem::with_id(app, SHOW_MENU_ID, "显示主窗口", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, QUIT_MENU_ID, "退出", true, None::<&str>)?,
            ])?;

            // 创建系统托盘图标（完全在代码中控制）
            let _ = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    SHOW_MENU_ID => {
                        let _ = show_main_window(app.clone());
                    }
                    QUIT_MENU_ID => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 只在左键点击时显示主窗口（右键点击会显示菜单，不应触发此事件）
                    if let TrayIconEvent::Click { 
                        button: tauri::tray::MouseButton::Left,
                        button_state: MouseButtonState::Up, 
                        .. 
                    } = event {
                        let app = tray.app_handle();
                        // 检查主窗口是否可见，如果可见则不处理，让右键菜单正常显示
                        if let Some(window) = app.get_webview_window("main") {
                            if let Ok(is_visible) = window.is_visible() {
                                // 只有窗口不可见时才显示
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
