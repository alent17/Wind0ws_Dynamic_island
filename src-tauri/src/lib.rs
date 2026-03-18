use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;
use std::time::Duration;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

// 用 Mutex 存储子进程句柄，方便后续关闭
pub struct ApiProcess(pub Mutex<Option<Child>>);

#[derive(Clone, Serialize, Deserialize)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub album_art: String,
    pub is_playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub api_duration_ms: u64, // 从网易云 API 获取的精准时长
    pub last_updated_timestamp: u64, // 快照产生的 Unix 毫秒时间戳
    pub api_cover_url: String, // API 返回的高清封面
    pub song_alias: String, // 歌曲别名
    pub source: String, // 播放器来源：netease, spotify, bilibili, generic
    pub source_display: String, // 原始 AppUserModelId
}

// 网易云 API 配置
// 优先从环境变量读取，如果没有设置则使用默认值
fn get_netease_api_base() -> String {
    std::env::var("NETEASE_API_BASE")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
}

// 网易云 API 搜索结果结构（使用 /cloudsearch 接口）
#[derive(Debug, Deserialize)]
struct CloudSearchResult {
    result: Option<CloudSearchData>,
}

#[derive(Debug, Deserialize)]
struct CloudSearchData {
    songs: Option<Vec<CloudSearchSong>>,
}

#[derive(Debug, Deserialize, Clone)]
struct CloudSearchSong {
    id: u64,
    name: String,
    dt: u64, // 时长（毫秒）
    ar: Option<Vec<CloudSearchArtist>>,
    al: Option<CloudSearchAlbum>,
    alia: Option<Vec<String>>, // 别名
}

#[derive(Debug, Deserialize, Clone)]
struct CloudSearchArtist {
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct CloudSearchAlbum {
    picUrl: String, // 高清封面 URL
    name: String,
}

// 通过网易云 API 搜索歌曲并获取详细信息
async fn fetch_song_info_from_api(title: &str, artist: &str) -> Option<(u64, String, String)> {
    let client = reqwest::Client::new();
    
    // 构建搜索关键词
    let keywords = format!("{} {}", title, artist);
    
    // 调用网易云 API /cloudsearch 接口
    let api_base = get_netease_api_base();
    let url = format!("{}/cloudsearch?keywords={}&type=1", api_base, urlencoding::encode(&keywords));
    
    match client.get(&url).send().await {
        Ok(response) => {
            match response.json::<CloudSearchResult>().await {
                Ok(data) => {
                    if let Some(result) = data.result {
                        if let Some(songs) = result.songs {
                            if let Some(first_song) = songs.first() {
                                // 返回：(时长，高清封面，别名)
                                let duration = first_song.dt;
                                let cover_url = first_song.al.as_ref()
                                    .map(|al| al.picUrl.clone())
                                    .unwrap_or_default();
                                let alias = first_song.alia.as_ref()
                                    .and_then(|alia| alia.first())
                                    .cloned()
                                    .unwrap_or_default();
                                return Some((duration, cover_url, alias));
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("网易云 API JSON 解析失败：{:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("网易云 API 请求失败：{:?}", e);
        }
    }
    
    None
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

    // 【核心：通过网易云 API 获取精准时长、高清封面、别名】
    // 仅在识别为网易云音乐时才请求 API
    let (api_duration_ms, api_cover_url, song_alias) = if source_type == "netease" && !title.is_empty() && is_playing {
        // 异步调用 API（不会阻塞主线程）
        match fetch_song_info_from_api(&title, &artist).await {
            Some((duration, cover, alias)) => (duration, cover, alias),
            None => (0, String::new(), String::new()),
        }
    } else {
        (0, String::new(), String::new())
    };

    Ok(MediaState {
        title: title.clone(),
        artist: artist.clone(),
        album_art: if !api_cover_url.is_empty() {
            api_cover_url.clone() // 优先使用 API 的高清封面
        } else if thumbnail_base64.is_empty() {
            "https://picsum.photos/400/400?random=1".to_string()
        } else {
            thumbnail_base64
        },
        is_playing,
        position_ms,
        duration_ms,
        api_duration_ms,
        last_updated_timestamp,
        api_cover_url,
        song_alias,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![control_media])
        .manage(ApiProcess(Mutex::new(None))) // 注入 API 进程管理状态
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_focus().unwrap();

            // --- 启动网易云 API ---
            // 尝试在项目根目录的 bin/api-enhanced-main 中查找 API
            let api_path = std::env::current_dir()
                .unwrap_or_default()
                .join("bin")
                .join("api-enhanced-main");
            
            // 检查 API 目录是否存在
            if api_path.exists() {
                let child = Command::new("node")
                    .arg("app.js")
                    .current_dir(&api_path)
                    .stdout(Stdio::null()) // 静默运行，不污染控制台
                    .stderr(Stdio::null())
                    .spawn();

                if let Ok(c) = child {
                    let state = app.state::<ApiProcess>();
                    *state.0.lock().unwrap() = Some(c);
                    println!("✅ 网易云 API 已随程序自动启动 (Port: 3000)");
                    println!("   路径：{}", api_path.display());
                } else {
                    eprintln!("⚠️  无法启动网易云 API，请确保 bin/api-enhanced-main/app.js 存在");
                }
            } else {
                eprintln!("⚠️  未找到网易云 API 目录：{}", api_path.display());
                eprintln!("   请将 NeteaseCloudMusicApi 放置在项目根目录的 bin/api-enhanced-main 文件夹中");
            }

            // 启动后台监听线程
            start_media_listener(app.handle().clone());

            Ok(())
        });

    // 添加窗口事件处理器
    builder = builder.on_window_event(|window, event| {
        // --- 关闭程序时杀掉 API 进程 ---
        if let tauri::WindowEvent::Destroyed = event {
            let state = window.state::<ApiProcess>();
            let mut locked = state.0.lock().unwrap();
            if let Some(mut child) = locked.take() {
                let _ = child.kill(); // 强行结束 Node 进程
                println!("✅ 网易云 API 已关闭");
            }
        }
    });

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
