use base64::{engine::general_purpose, Engine as _};
use color_quant::NeuQuant;
use image::{imageops::FilterType, DynamicImage, GenericImageView, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Duration;
use std::fs;
use std::path::PathBuf;
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
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{FftPlanner, num_complex::Complex};
use window_vibrancy::{apply_acrylic, clear_acrylic};

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
    // ===== MV 播放 =====
    pub enable_mv_playback: bool,
    // ===== 锁定悬浮窗 =====
    pub lock_floating_window: bool,
    // ===== 专辑封面设置 =====
    pub enable_hd_cover: bool,
    pub enable_pixel_art: bool,
    pub enable_halftone: bool,
    // ===== 缓存目录设置 =====
    pub cache_directory: Option<String>,
    // ===== 开机启动 =====
    pub auto_start: bool,
    // ===== UI 显示控制 =====
    pub hide_settings_button: bool,
    pub hide_monitor_selector: bool,
    pub hide_floating_window: bool,
    // ===== 灵动岛样式 =====
    pub expanded_corner_radius: u32,
    // ===== 顶部栏显示 =====
    pub always_show_top_bar: bool,
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
            enable_mv_playback: true, // 默认开启 MV 播放
            lock_floating_window: false, // 默认不锁定悬浮窗
            enable_hd_cover: true, // 默认开启高清封面获取
            enable_pixel_art: false, // 默认关闭像素化
            enable_halftone: false, // 默认关闭网点效果
            cache_directory: None, // 默认使用系统缓存目录
            auto_start: false, // 默认关闭开机启动
            hide_settings_button: false, // 默认显示设置按钮
            hide_monitor_selector: false, // 默认显示显示器选择
            hide_floating_window: false, // 默认显示悬浮窗按钮
            expanded_corner_radius: 16, // 默认圆角 16px
            always_show_top_bar: true, // 默认固定显示顶部栏
        }
    }
}

#[derive(Default)]
struct MediaCache {
    track_id: String,
    base64_img: String,
}

// 缓存元数据
#[derive(Clone, Serialize, Deserialize)]
struct CacheMetadata {
    key: String,
    file_path: String,
    created_at: u64,
    size: u64,
    content_type: String, // "mv" 或 "cover"
}

struct AppState {
    settings: Mutex<AppSettings>,
    media_cache: Mutex<MediaCache>,
}

// 全局缓存目录（懒加载）
static CACHE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
static CACHE_METADATA: Mutex<Option<Vec<CacheMetadata>>> = Mutex::new(None);

// 初始化缓存系统
fn init_cache_system(app_handle: &AppHandle) -> Result<(), String> {
    // 先从配置读取缓存目录
    let settings = read_settings_file(app_handle).unwrap_or_default();
    
    let cache_dir = if let Some(custom_dir) = &settings.cache_directory {
        PathBuf::from(custom_dir)
    } else {
        app_handle
            .path()
            .app_cache_dir()
            .map_err(|e| format!("无法获取缓存目录：{}", e))?
            .join("media_cache")
    };
    
    // 创建缓存目录
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("无法创建缓存目录：{}", e))?;
    
    // 加载元数据
    let metadata_file = cache_dir.join("metadata.json");
    let metadata = if metadata_file.exists() {
        let content = fs::read_to_string(&metadata_file)
            .map_err(|e| format!("无法读取元数据：{}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    // 存储到全局变量
    {
        let mut cache_dir_global = CACHE_DIR.lock().map_err(|_| "无法锁定缓存目录")?;
        *cache_dir_global = Some(cache_dir);
    }
    
    {
        let mut metadata_global = CACHE_METADATA.lock().map_err(|_| "无法锁定缓存元数据")?;
        *metadata_global = Some(metadata);
    }
    
    Ok(())
}

// 切换悬浮窗显示/隐藏
#[tauri::command]
fn toggle_floating_window(_app: tauri::AppHandle) -> Result<(), String> {
    // 此函数已废弃，使用前端直接控制
    Ok(())
}

// 保存缓存文件
fn save_cache_file(url: &str, content: &[u8], content_type: &str) -> Result<String, String> {
    let cache_dir = CACHE_DIR.lock()
        .map_err(|_| "无法锁定缓存目录")?
        .clone()
        .ok_or("缓存系统未初始化")?;
    
    // 生成缓存键（使用 URL 的哈希）
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let key = format!("{:x}", hasher.finish());
    
    // 确定文件扩展名
    let extension = match content_type {
        "video/mp4" | "video/webm" => "mp4",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        _ => "dat",
    };
    
    let file_path = cache_dir.join(format!("{}.{}", key, extension));
    
    // 保存文件
    fs::write(&file_path, content)
        .map_err(|e| format!("无法写入缓存文件：{}", e))?;
    
    // 更新元数据
    {
        let mut metadata_global = CACHE_METADATA.lock().map_err(|_| "无法锁定缓存元数据")?;
        let metadata = metadata_global.as_mut().ok_or("缓存元数据未初始化")?;
        
        // 检查是否已存在
        if let Some(pos) = metadata.iter().position(|m| m.key == key) {
            metadata.remove(pos);
        }
        
        metadata.push(CacheMetadata {
            key,
            file_path: file_path.to_string_lossy().to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            size: content.len() as u64,
            content_type: content_type.to_string(),
        });
        
        let metadata_file = cache_dir.join("metadata.json");
        let content = serde_json::to_string_pretty(&*metadata)
            .map_err(|e| format!("无法序列化元数据：{}", e))?;
        fs::write(&metadata_file, content)
            .map_err(|e| format!("无法写入元数据：{}", e))?;
    }
    
    // 直接返回文件路径，由前端使用 convertFileSrc 转换
    Ok(file_path.to_string_lossy().to_string())
}

// 获取缓存文件路径
#[tauri::command]
fn get_cached_media(url: String) -> Result<Option<String>, String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let key = format!("{:x}", hasher.finish());
    
    let metadata_global = CACHE_METADATA.lock()
        .map_err(|_| "无法锁定缓存元数据")?;
    let metadata = metadata_global.as_ref().ok_or("缓存元数据未初始化")?;
    
    if let Some(meta) = metadata.iter().find(|m| m.key == key) {
        if PathBuf::from(&meta.file_path).exists() {
            // 直接返回文件路径，由前端使用 convertFileSrc 转换
            return Ok(Some(meta.file_path.clone()));
        }
    }
    
    Ok(None)
}

// 下载并缓存文件
#[tauri::command]
async fn download_and_cache(url: String, content_type: String) -> Result<String, String> {
    // 先检查缓存
    if let Some(cached_path) = get_cached_media(url.clone())? {
        return Ok(cached_path);
    }
    
    // 下载文件
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await
        .map_err(|e| format!("下载失败：{}", e))?;
    
    let bytes = response.bytes().await
        .map_err(|e| format!("读取数据失败：{}", e))?;
    
    // 保存缓存
    save_cache_file(&url, &bytes, &content_type)
}

// 清理缓存
#[tauri::command]
fn clear_cache() -> Result<(), String> {
    let cache_dir = CACHE_DIR.lock()
        .map_err(|_| "无法锁定缓存目录")?
        .clone()
        .ok_or("缓存系统未初始化")?;
    
    // 删除所有缓存文件
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)
            .map_err(|e| format!("无法清理缓存：{}", e))?;
        
        // 重新创建目录
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("无法创建缓存目录：{}", e))?;
        
        // 清空元数据
        {
            let mut metadata_global = CACHE_METADATA.lock().map_err(|_| "无法锁定缓存元数据")?;
            *metadata_global = Some(Vec::new());
        }
    }
    
    Ok(())
}

// 获取缓存统计信息
#[tauri::command]
fn get_cache_stats() -> Result<serde_json::Value, String> {
    let metadata_global = CACHE_METADATA.lock()
        .map_err(|_| "无法锁定缓存元数据")?;
    let metadata = metadata_global.as_ref().ok_or("缓存元数据未初始化")?;
    
    let total_size: u64 = metadata.iter().map(|m| m.size).sum();
    let mv_count = metadata.iter().filter(|m| m.content_type.starts_with("video")).count();
    let cover_count = metadata.iter().filter(|m| m.content_type.starts_with("image")).count();
    
    Ok(serde_json::json!({
        "total_size_mb": total_size as f64 / (1024.0 * 1024.0),
        "total_files": metadata.len(),
        "mv_count": mv_count,
        "cover_count": cover_count,
    }))
}

// 获取当前缓存目录
#[tauri::command]
fn get_cache_directory(_app: AppHandle) -> Result<String, String> {
    let cache_dir = CACHE_DIR.lock()
        .map_err(|_| "无法锁定缓存目录")?
        .clone()
        .ok_or("缓存系统未初始化")?;
    
    Ok(cache_dir.to_string_lossy().to_string())
}

// 设置缓存目录
#[tauri::command]
fn set_cache_directory(app: AppHandle, new_path: String) -> Result<(), String> {
    use std::path::PathBuf;
    
    let new_cache_dir = PathBuf::from(&new_path);
    
    // 确保目录存在
    if !new_cache_dir.exists() {
        fs::create_dir_all(&new_cache_dir)
            .map_err(|e| format!("创建目录失败：{}", e))?;
    }
    
    // 更新缓存目录
    {
        let mut cache_dir_global = CACHE_DIR.lock()
            .map_err(|_| "无法锁定缓存目录")?;
        *cache_dir_global = Some(new_cache_dir.clone());
    }
    
    // 保存配置
    let mut settings = read_settings_file(&app).unwrap_or_default();
    settings.cache_directory = Some(new_path);
    write_settings_file(&app, &settings)?;
    
    Ok(())
}

// 选择缓存目录
#[tauri::command]
fn pick_cache_directory(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    // 使用 blocking_pick_folder 阻塞式选择
    let selected_path = app.dialog().file().blocking_pick_folder();
    
    if let Some(path) = selected_path {
        let path_str = path.to_string();
        
        // 保存配置
        let mut settings = read_settings_file(&app).unwrap_or_default();
        settings.cache_directory = Some(path_str.clone());
        write_settings_file(&app, &settings)?;
        
        // 更新缓存目录
        set_cache_directory(app, path_str.clone())?;
        
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}

// 设置开机启动
#[tauri::command]
fn set_auto_start(_app: AppHandle, enable: bool) -> Result<(), String> {
    use std::process::Command;

    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败：{}", e))?;
    let exe_path_str = exe_path.to_string_lossy().to_string();

    if enable {
        // 添加注册表项
        let output = Command::new("reg")
            .args(&[
                "add",
                "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "Wind0wsDynamicIsland",
                "/t",
                "REG_SZ",
                "/d",
                &exe_path_str,
                "/f",
            ])
            .output()
            .map_err(|e| format!("执行 reg 命令失败：{}", e))?;

        if !output.status.success() {
            return Err(format!(
                "添加注册表项失败：{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    } else {
        // 删除注册表项
        let output = Command::new("reg")
            .args(&[
                "delete",
                "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "Wind0wsDynamicIsland",
                "/f",
            ])
            .output()
            .map_err(|e| format!("执行 reg delete 命令失败：{}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("系统找不到指定的注册表值")
                && !stderr.contains("The system cannot find the registry key")
            {
                eprintln!("删除注册表项警告：{}", stderr);
            }
        }
    }

    Ok(())
}

// 获取开机启动状态
#[tauri::command]
fn get_auto_start() -> Result<bool, String> {
    use std::process::Command;
    
    // 查询注册表项
    let output = Command::new("reg")
        .args(&[
            "query",
            "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v",
            "Wind0wsDynamicIsland",
        ])
        .output();
    
    match output {
        Ok(out) => Ok(out.status.success()),
        Err(_) => Ok(false),
    }
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

fn get_media_info_internal(app: &AppHandle) -> Result<MediaState, String> {
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
        // 返回空状态而不是错误，让前端显示"等待播放..."
        return Ok(MediaState {
            title: String::new(),
            artist: String::new(),
            album_art: String::new(),
            is_playing: false,
            position_ms: 0,
            duration_ms: 0,
            last_updated_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            source: String::new(),
            source_display: String::new(),
        });
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

    let (session, source_type, raw_id) = match best_session {
        Some(s) => s,
        None => {
            // 没有有效会话，返回空状态
            return Ok(MediaState {
                title: String::new(),
                artist: String::new(),
                album_art: String::new(),
                is_playing: false,
                position_ms: 0,
                duration_ms: 0,
                last_updated_timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                source: String::new(),
                source_display: String::new(),
            });
        }
    };

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

        // 只在成功获取到封面时才更新缓存
        if !thumbnail_base64.is_empty() {
            let state = app.state::<AppState>();
            if let Ok(mut cache) = state.media_cache.lock() {
                cache.track_id = track_id;
                cache.base64_img = thumbnail_base64.clone();
            };
        }
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
        album_art: thumbnail_base64,
        is_playing,
        position_ms,
        duration_ms,
        last_updated_timestamp,
        source: source_type.to_string(),
        source_display: raw_id,
    })
}

#[tauri::command]
fn control_media(app: AppHandle, action: String) -> Result<(), String> {
    unsafe {
        windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        ).ok();
    }
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

            let app_clone = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(100));
                unsafe {
                    windows::Win32::System::Com::CoInitializeEx(
                        None,
                        windows::Win32::System::Com::COINIT_MULTITHREADED,
                    ).ok();
                }
                if let Ok(info) = get_media_info_internal(&app_clone) {
                    let _ = app_clone.emit("media-update", info);
                }
            });
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
    std::thread::spawn(move || {
        unsafe {
            windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            ).ok();
        }

        loop {
            if let Ok(info) = get_media_info_internal(&handle) {
                let _ = handle.emit("media-update", info);
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });
}

#[derive(Clone, Serialize)]
struct SpectrumPayload {
    bands: Vec<f32>,
    bands_expanded: Vec<f32>,
}

fn start_audio_visualizer(app: AppHandle) {
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                eprintln!("[音频] 未找到默认音频输出设备");
                return;
            }
        };

        let config = match device.default_output_config() {
            Ok(c) => c.config(),
            Err(e) => {
                eprintln!("[音频] 获取音频配置失败: {}", e);
                return;
            }
        };

        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0 as f32;

        const FFT_SIZE: usize = 1024;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let mut sample_buffer: Vec<f32> = Vec::with_capacity(FFT_SIZE);
        let mut frame_count: usize = 0;

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                frame_count += 1;
                for frame in data.chunks(channels) {
                    let mono = frame.iter().sum::<f32>() / channels as f32;
                    sample_buffer.push(mono);

                    if sample_buffer.len() == FFT_SIZE {
                        let mut buffer: Vec<Complex<f32>> = sample_buffer
                            .iter()
                            .enumerate()
                            .map(|(i, &val)| {
                                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE as f32 - 1.0)).cos());
                                Complex { re: val * window, im: 0.0 }
                            })
                            .collect();

                        fft.process(&mut buffer);

                        let magnitudes: Vec<f32> = buffer.iter()
                            .take(FFT_SIZE / 2)
                            .map(|c| c.norm())
                            .collect();

                        let bands = calculate_bands(&magnitudes, 5, sample_rate, FFT_SIZE);
                        let bands_expanded = calculate_bands(&magnitudes, 40, sample_rate, FFT_SIZE);

                        if frame_count % 3 == 0 {
                            let _ = app.emit("audio-spectrum", SpectrumPayload {
                                bands,
                                bands_expanded
                            });
                        }

                        sample_buffer.drain(0..FFT_SIZE / 2);
                    }
                }
            },
            |err| eprintln!("[音频] 捕获流错误: {}", err),
            None
        );

        match stream {
            Ok(s) => {
                s.play().unwrap();
                loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
            }
            Err(e) => eprintln!("[音频] 构建捕获流失败: {}", e),
        }
    });
}


fn calculate_bands(magnitudes: &[f32], num_bands: usize, sample_rate: f32, fft_size: usize) -> Vec<f32> {
    let mut bands = vec![0.0; num_bands];
    let freq_resolution = sample_rate / fft_size as f32;

    let min_freq: f32 = 80.0;
    let max_freq: f32 = 8000.0;
    let log_min = min_freq.log2();
    let log_max = max_freq.log2();
    let log_step = (log_max - log_min) / num_bands as f32;

    for i in 0..num_bands {
        let start_freq = 2.0_f32.powf(log_min + i as f32 * log_step);
        let end_freq = 2.0_f32.powf(log_min + (i + 1) as f32 * log_step);

        let mut start_bin = (start_freq / freq_resolution).round() as usize;
        let mut end_bin = (end_freq / freq_resolution).round() as usize;

        start_bin = start_bin.clamp(1, magnitudes.len() - 1);
        end_bin = end_bin.clamp(start_bin + 1, magnitudes.len());

        let mut sum = 0.0;
        let mut count = 0.0;
        for j in start_bin..end_bin {
            let freq = j as f32 * freq_resolution;
            let weight = (freq / 1000.0).powf(1.15).clamp(0.05, 10.0);
            sum += magnitudes[j] * weight;
            count += 1.0;
        }

        let avg = if count > 0.0 { sum / count } else { 0.0 };

        let db = if avg > 0.0001 { 20.0 * avg.log10() } else { -100.0 };

        let mut normalized = (db + 35.0) / 50.0;

        if normalized < 0.12 { normalized = 0.0; }

        bands[i] = normalized.clamp(0.0, 1.0).powf(1.8);
    }
    bands
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
        // 窗口已存在，调整大小并居中显示
        let _ = window.set_size(tauri::PhysicalSize::new(1000, 750));
        let _ = window.set_min_size(Some(tauri::PhysicalSize::new(800, 600)));
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let _ = tauri::WebviewWindowBuilder::new(
        &app,
        "settings-window",
        tauri::WebviewUrl::App("/settings".into()),
    )
    .title("Isle - 设置")
    .inner_size(1000.0, 750.0)
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
    let auto_start = settings.auto_start;
    let state = app.state::<AppState>();
    
    let mut old_settings: Option<AppSettings> = None;
    {
        let mut state_settings = state
            .settings
            .lock()
            .map_err(|_| "Failed to lock settings")?;
        old_settings = Some(state_settings.clone());
        *state_settings = settings.clone();
    }

    // 保存到配置文件
    write_settings_file(&app, &settings)?;

    // 应用置顶
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(always_on_top);
    }

    // 更新开机启动状态
    if auto_start {
        set_auto_start(app.clone(), true)?;
    } else {
        let _ = set_auto_start(app.clone(), false);
    }

    // ===== 广播各设置项的单独变更事件 =====
    if let Some(old) = old_settings {
        if old.enable_halftone != settings.enable_halftone {
            let _ = app.emit("halftone-changed", serde_json::json!({
                "enableHalftone": settings.enable_halftone
            }));
        }
        if old.enable_pixel_art != settings.enable_pixel_art {
            let _ = app.emit("pixel-art-changed", serde_json::json!({
                "enablePixelArt": settings.enable_pixel_art
            }));
        }
        if old.enable_hd_cover != settings.enable_hd_cover {
            let _ = app.emit("hd-cover-changed", serde_json::json!({
                "enableHDCover": settings.enable_hd_cover
            }));
        }
        if old.enable_mv_playback != settings.enable_mv_playback {
            let _ = app.emit("mv-playback-changed", serde_json::json!({
                "enable": settings.enable_mv_playback
            }));
        }
        if old.lock_floating_window != settings.lock_floating_window {
            let _ = app.emit("lock-floating-window-changed", serde_json::json!({
                "lock": settings.lock_floating_window
            }));
        }
        if old.always_on_top != settings.always_on_top {
            let _ = app.emit("always-on-top-changed", serde_json::json!({
                "isAlwaysOnTop": settings.always_on_top
            }));
        }
        if old.expanded_corner_radius != settings.expanded_corner_radius {
            let _ = app.emit("corner-radius-changed", settings.expanded_corner_radius);
        }
    }

    // ===== 广播完整设置更新事件 =====
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
        GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetClassNameW,
        GWL_STYLE, WS_CAPTION,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Ok(false);
        }

        let mut class_name = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut class_name);
        if len > 0 {
            let class_str = String::from_utf16_lossy(&class_name[..len as usize]);
            if class_str == "WorkerW" || class_str == "Progman" || class_str == "Shell_TrayWnd" {
                return Ok(false);
            }
        }

        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

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

            let no_caption = (style & WS_CAPTION.0 as isize) == 0;

            return Ok(no_caption);
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

// 关闭悬浮窗
#[tauri::command]
fn close_floating_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating_player") {
        let _ = window.close();
        // 发送事件通知前端更新状态
        let _ = app.emit("floating-window-closed", ());
    }
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
fn set_floating_window_resizable(app: AppHandle, resizable: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating_player") {
        window
            .set_resizable(resizable)
            .map_err(|e| e.to_string())?;
        println!("[窗口] 悬浮窗可调整大小已设置为：{}", resizable);
    }
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

// 打开指定的应用程序
#[tauri::command]
fn open_application(name: String) -> Result<(), String> {
    use std::process::Command;
    
    // 优先使用协议处理器打开应用（更可靠）
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
                println!("[应用] 通过协议打开：{}", name);
                return Ok(());
            }
        }
    }
    
    // 如果协议处理失败，尝试常见的安装路径
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
        let expanded = path_template.replace("%APPDATA%", &std::env::var("APPDATA").unwrap_or_default())
            .replace("%LOCALAPPDATA%", &std::env::var("LOCALAPPDATA").unwrap_or_default())
            .replace("%PROGRAMFILES%", &std::env::var("ProgramFiles").unwrap_or_default())
            .replace("%PROGRAMFILES(X86)%", &std::env::var("ProgramFiles(x86)").unwrap_or_default());
        
        if std::path::Path::new(&expanded).exists() {
            let output = Command::new("cmd")
                .args(["/C", "start", "", &expanded])
                .output();
            
            if let Ok(out) = output {
                if out.status.success() {
                    println!("[应用] 从路径打开：{}", expanded);
                    return Ok(());
                }
            }
        }
    }
    
    // 最后尝试直接启动（适用于已加入PATH的应用）
    let executable_name = match name.as_str() {
        "NeteaseCloudMusic" => "cloudmusic.exe",
        "Spotify" => "spotify.exe",
        "Bilibili" => "bilibili.exe",
        "QQMusic" => "QQMusic.exe",
        "AppleMusic" => "AppleMusic.exe",
        _ => &name,
    };
    
    let output = Command::new("cmd")
        .args(["/C", "start", "", executable_name])
        .output();
    
    match output {
        Ok(_) => {
            println!("[应用] 已尝试打开：{}", executable_name);
            Ok(())
        }
        Err(e) => {
            eprintln!("[应用] 打开失败：{} - {}", executable_name, e);
            Err(format!("无法打开应用：{}", e))
        }
    }
}

// ===== 高级像素化处理函数 =====
/// 核心像素化处理函数 (使用 NeuQuant 调色板 + Floyd-Steinberg 抖动)
fn pixelate_image_advanced(
    img_data: &[u8],
    pixel_size: u32,
    num_colors: usize,
) -> Result<Vec<u8>, String> {
    // 从 PNG 数据加载图片
    let img = image::load_from_memory(img_data)
        .map_err(|e| format!("无法加载图片：{}", e))?;
    
    let (width, height) = img.dimensions();
    
    if width < pixel_size || height < pixel_size {
        return Err("图片太小，无法像素化".to_string());
    }
    
    // 1. 计算小图尺寸并进行平滑降采样
    let target_width = width / pixel_size;
    let target_height = height / pixel_size;
    
    if target_width == 0 || target_height == 0 {
        return Err("图片太小，无法像素化".to_string());
    }
    
    let downscaled = img.resize_exact(target_width, target_height, FilterType::Lanczos3);
    let mut downscaled_rgba: RgbaImage = downscaled.into_rgba8();
    
    // 2. 提取像素供 NeuQuant 使用
    let pixels: Vec<u8> = downscaled_rgba
        .pixels()
        .flat_map(|p| vec![p[0], p[1], p[2], p[3]])
        .collect();
    
    // 3. 生成全局调色板
    let nq = NeuQuant::new(10, num_colors, &pixels);
    let palette = nq.color_map_rgb(); // 返回拍平的调色板数组 [R,G,B, R,G,B...]
    
    let (dw, dh) = downscaled_rgba.dimensions();
    let dw_usize = dw as usize;
    let dh_usize = dh as usize;
    
    // 创建误差缓冲区
    let mut error_buffer = vec![vec![[0.0_f32; 3]; dw_usize]; dh_usize];
    
    // 4. 应用调色板并进行误差扩散
    for y in 0..dh {
        for x in 0..dw {
            let x_usize = x as usize;
            let y_usize = y as usize;
            
            let pixel = downscaled_rgba.get_pixel(x, y);
            
            let err_r = error_buffer[y_usize][x_usize][0];
            let err_g = error_buffer[y_usize][x_usize][1];
            let err_b = error_buffer[y_usize][x_usize][2];
            
            let r = (pixel[0] as f32 + err_r).clamp(0.0, 255.0);
            let g = (pixel[1] as f32 + err_g).clamp(0.0, 255.0);
            let b = (pixel[2] as f32 + err_b).clamp(0.0, 255.0);
            
            let color_idx = nq.index_of(&[r as u8, g as u8, b as u8, pixel[3]]);
            
            // 安全检查：确保索引不越界
            let safe_idx = color_idx.min((palette.len() / 3).saturating_sub(1));
            
            // palette 现在是 RGB 格式，每个颜色占 3 个字节
            let pr = palette[safe_idx * 3];
            let pg = palette[safe_idx * 3 + 1];
            let pb = palette[safe_idx * 3 + 2];
            let pa = pixel[3]; // 保持原有透明度
            
            downscaled_rgba.put_pixel(x, y, Rgba([pr, pg, pb, pa]));
            
            let quant_error_r = r - pr as f32;
            let quant_error_g = g - pg as f32;
            let quant_error_b = b - pb as f32;
            
            let distribute_error = |eb: &mut Vec<Vec<[f32; 3]>>, dx: usize, dy: usize, ratio: f32| {
                if dx < dw_usize && dy < dh_usize {
                    eb[dy][dx][0] += quant_error_r * ratio;
                    eb[dy][dx][1] += quant_error_g * ratio;
                    eb[dy][dx][2] += quant_error_b * ratio;
                }
            };
            
            // 使用 saturating_add 防止溢出
            let ux_next = x_usize.saturating_add(1);
            let uy_next = y_usize.saturating_add(1);
            
            distribute_error(&mut error_buffer, ux_next, y_usize, 7.0 / 16.0);
            if uy_next < dh_usize {
                if x_usize > 0 {
                    distribute_error(&mut error_buffer, x_usize - 1, uy_next, 3.0 / 16.0);
                }
                distribute_error(&mut error_buffer, x_usize, uy_next, 5.0 / 16.0);
                distribute_error(&mut error_buffer, ux_next, uy_next, 1.0 / 16.0);
            }
        }
    }
    
    // 5. 锐利升采样
    let result = DynamicImage::ImageRgba8(downscaled_rgba)
        .resize_exact(width, height, FilterType::Nearest);
    
    // 6. 转换为 PNG 格式
    let mut output_data = Vec::new();
    result
        .write_to(&mut std::io::Cursor::new(&mut output_data), image::ImageFormat::Png)
        .map_err(|e| format!("无法保存 PNG: {}", e))?;
    
    Ok(output_data)
}

// Tauri 命令：处理像素化图片
#[tauri::command]
async fn pixelate_cover(
    image_path: String,
    pixel_size: u32,
    num_colors: usize,
) -> Result<String, String> {
    use std::time::Instant;
    let start = Instant::now();
    
    // 处理不同的图片来源
    let img_data = if image_path.starts_with("data:") {
        // base64 数据
        let parts: Vec<&str> = image_path.split(',').collect();
        if parts.len() < 2 {
            return Err("无效的 base64 图片".to_string());
        }
        let base64_data = parts[1];
        base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| format!("无法解码 base64: {}", e))?
    } else if image_path.starts_with("http") {
        // 网络图片，先下载到缓存
        let client = reqwest::Client::new();
        let response = client.get(&image_path).send().await
            .map_err(|e| format!("无法下载图片：{}", e))?;
        response.bytes().await
            .map_err(|e| format!("无法读取图片数据：{}", e))?
            .to_vec()
    } else {
        // 本地文件路径（包括 asset://）
        let actual_path = if image_path.starts_with("asset://") {
            image_path.strip_prefix("asset://").unwrap_or(&image_path)
        } else {
            &image_path
        };
        fs::read(actual_path)
            .map_err(|e| format!("无法读取图片：{} ({})", e, actual_path))?
    };
    
    // 处理像素化
    let processed_data = pixelate_image_advanced(&img_data, pixel_size, num_colors)?;
    
    // 保存到缓存目录
    let cache_dir = CACHE_DIR.lock()
        .map_err(|_| "无法锁定缓存目录")?
        .clone()
        .ok_or("缓存系统未初始化")?;
    
    // 生成唯一的文件名
    let file_name = format!("pixel_{}.png", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());
    
    let output_path = cache_dir.join(&file_name);
    
    fs::write(&output_path, &processed_data)
        .map_err(|e| format!("无法保存处理后的图片：{}", e))?;
    
    let elapsed = start.elapsed();
    println!("[像素化] 处理完成：{:?}ms, 输出：{}", elapsed.as_millis(), output_path.display());
    
    Ok(output_path.to_string_lossy().to_string())
}

/// 解析本地文件路径（支持多种协议格式）
fn resolve_local_path(image_path: &str) -> String {
    if image_path.starts_with("asset://") {
        // 移除 asset:// 前缀
        let path = image_path.strip_prefix("asset://").unwrap_or(image_path);
        path.replace("asset.localhost/", "")
    } else if image_path.contains("asset.localhost") {
        // 处理 http://asset.localhost/... 格式
        let path = image_path
            .replace("http://asset.localhost/", "")
            .replace("https://asset.localhost/", "");
        // URL 解码：处理常见的编码字符
        path.replace("%3A", ":")
            .replace("%5C", "\\")
            .replace("%2F", "/")
            .replace("%20", " ")
    } else if image_path.starts_with("file://") {
        image_path.strip_prefix("file://").unwrap_or(image_path).to_string()
    } else {
        image_path.to_string()
    }
}

/// 带有备用方案的文件读取
fn read_file_with_fallbacks(path: &str, log_prefix: &str) -> Result<Vec<u8>, String> {
    println!("{} 读取文件：{}", log_prefix, path);
    
    if let Ok(data) = fs::read(path) {
        println!("{} 文件读取成功：{} bytes", log_prefix, data.len());
        return Ok(data);
    }
    
    let alt_path = path.replace("/", "\\");
    println!("{} 尝试备用路径（\\）：{}", log_prefix, alt_path);
    
    if let Ok(data) = fs::read(&alt_path) {
        println!("{} 备用路径读取成功：{} bytes", log_prefix, data.len());
        return Ok(data);
    }
    
    let alt_path2 = alt_path.trim_start_matches('\\').to_string();
    println!("{} 尝试备用路径 2：{}", log_prefix, alt_path2);
    
    fs::read(&alt_path2).map_err(|e| {
        format!("无法读取图片：{} ({})", e, path)
    })
}

/// 加载图片数据（支持 base64、网络、本地文件）
fn load_image_data(image_path: &str, log_prefix: &str) -> Result<Vec<u8>, String> {
    println!("{} 收到路径：{}", log_prefix, image_path.chars().take(100).collect::<String>());
    
    let is_local_asset = image_path.contains("asset.localhost");
    
    if image_path.starts_with("data:") {
        println!("{} 检测到 base64 数据", log_prefix);
        let parts: Vec<&str> = image_path.split(',').collect();
        if parts.len() < 2 {
            return Err("无效的 base64 图片".to_string());
        }
        
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(parts[1])
            .map_err(|e| format!("无法解码 base64: {}", e))?;
        
        println!("{} base64 解码完成：{} bytes", log_prefix, decoded.len());
        Ok(decoded)
    } else if image_path.starts_with("http") && !is_local_asset {
        println!("{} 下载网络图片", log_prefix);
        let client = reqwest::blocking::Client::new();
        let response = client.get(image_path).send()
            .map_err(|e| format!("无法下载图片：{}", e))?;
        
        let bytes = response.bytes()
            .map_err(|e| format!("无法读取图片数据：{}", e))?
            .to_vec();
        
        println!("{} 下载完成：{} bytes", log_prefix, bytes.len());
        Ok(bytes)
    } else {
        let actual_path = resolve_local_path(image_path);
        read_file_with_fallbacks(&actual_path, log_prefix)
    }
}

// Tauri 命令：提取图片主色
#[tauri::command]
fn extract_dominant_color(image_path: String) -> Result<(u8, u8, u8), String> {
    let img_data = load_image_data(&image_path, "[颜色提取]")?;
    
    if img_data.is_empty() {
        println!("[颜色提取] 图片数据为空，返回默认颜色");
        return Ok((60, 80, 100));
    }
    
    println!("[颜色提取] 尝试加载图片...");
    let img = image::load_from_memory(&img_data)
        .map_err(|e| {
            println!("[颜色提取] 图片加载失败：{}", e);
            format!("无法加载图片：{} (数据大小：{} bytes)", e, img_data.len())
        })?;
    
    println!("[颜色提取] 图片加载成功：{}x{}", img.width(), img.height());
    
    // 缩小到 80x80 进行分析
    let resized = img.resize_exact(80, 80, FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    
    // 统计颜色频率
    let mut color_count = std::collections::HashMap::new();
    for pixel in rgba.pixels() {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        if a < 128 || (r == 0 && g == 0 && b == 0) || (r == 255 && g == 255 && b == 255) {
            continue; // 跳过透明、纯黑、纯白
        }
        
        // 量化颜色（12 位分组）
        let qr = r / 12 * 12;
        let qg = g / 12 * 12;
        let qb = b / 12 * 12;
        
        *color_count.entry((qr, qg, qb)).or_insert(0) += 1;
    }
    
    // 计算饱和度并排序
    let mut colors: Vec<_> = color_count.into_iter().collect();
    colors.sort_by(|a, b| {
        let ((r1, g1, b1), c1) = a;
        let ((r2, g2, b2), c2) = b;
        
        fn saturation(r: u8, g: u8, b: u8) -> f32 {
            let rn = r as f32 / 255.0;
            let gn = g as f32 / 255.0;
            let bn = b as f32 / 255.0;
            let max = rn.max(gn).max(bn);
            let min = rn.min(gn).min(bn);
            let l = (max + min) / 2.0;
            if max == min {
                0.0
            } else if l > 0.5 {
                (max - min) / (2.0 - max - min)
            } else {
                (max - min) / (max + min)
            }
        }
        
        let sat1 = saturation(*r1, *g1, *b1);
        let sat2 = saturation(*r2, *g2, *b2);
        let score1 = sat1 * (*c1 as f32).ln();
        let score2 = sat2 * (*c2 as f32).ln();
        
        score2.partial_cmp(&score1).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // 返回最佳颜色（稍微调亮）
    if let Some(((r, g, b), _)) = colors.first() {
        let rr = r.saturating_add(12).min(255);
        let gg = g.saturating_add(12).min(255);
        let bb = b.saturating_add(12).min(255);
        Ok((rr, gg, bb))
    } else {
        // 默认颜色
        Ok((40, 50, 60))
    }
}

// 设置隐藏灵动岛设置按钮
#[tauri::command]
fn set_hide_settings_button(app: AppHandle, enable: bool) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    settings.hide_settings_button = enable;
    
    // 保存设置到文件
    write_settings_file(&app, &settings)?;
    
    // 发送事件通知前端更新 UI
    if let Err(e) = app.emit("settings-changed", "hide_settings_button") {
        eprintln!("发送设置变更事件失败：{}", e);
    }
    
    Ok(())
}

// 设置隐藏显示器选择按钮
#[tauri::command]
fn set_hide_monitor_selector(app: AppHandle, enable: bool) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    settings.hide_monitor_selector = enable;
    
    // 保存设置到文件
    write_settings_file(&app, &settings)?;
    
    // 发送事件通知前端更新 UI
    if let Err(e) = app.emit("settings-changed", "hide_monitor_selector") {
        eprintln!("发送设置变更事件失败：{}", e);
    }
    
    Ok(())
}

// 设置隐藏悬浮窗按钮
#[tauri::command]
fn set_hide_floating_window(app: AppHandle, enable: bool) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    settings.hide_floating_window = enable;
    
    // 保存设置到文件
    write_settings_file(&app, &settings)?;
    
    // 发送事件通知前端更新 UI
    if let Err(e) = app.emit("settings-changed", "hide_floating_window") {
        eprintln!("发送设置变更事件失败：{}", e);
    }
    
    Ok(())
}

// 设置展开后的圆角大小
#[tauri::command]
fn set_expanded_corner_radius(app: AppHandle, radius: u32) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
    // 限制圆角范围在 0-32px 之间
    settings.expanded_corner_radius = radius.min(32);
    
    // 保存设置到文件
    write_settings_file(&app, &settings)?;

    // 发送事件通知前端更新 UI
    if let Err(e) = app.emit("corner-radius-changed", radius.min(32)) {
        eprintln!("发送圆角变更事件失败：{}", e);
    }

    Ok(())
}

// Tauri 命令：应用/移除系统毛玻璃效果
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

// Tauri 命令：处理图片并返回 base64（支持像素化）
#[tauri::command]
async fn process_image(
    image_path: String,
    enable_pixel_art: bool,
) -> Result<String, String> {
    use std::time::Instant;
    let start = Instant::now();
    
    println!("[图片处理] 像素化：{}", enable_pixel_art);
    
    let img_data = load_image_data(&image_path, "[图片处理]")?;
    
    if img_data.is_empty() {
        println!("[图片处理] 图片数据为空，返回错误");
        return Err("图片数据为空".to_string());
    }
    
    println!("[图片处理] 验证图片格式...");
    let _img = image::load_from_memory(&img_data)
        .map_err(|e| format!("无法加载图片：{} (数据大小：{} bytes)", e, img_data.len()))?;
    
    println!("[图片处理] 图片验证成功");
    
    // 如果需要像素化
    if enable_pixel_art {
        let processed_data = pixelate_image_advanced(&img_data, 12, 32)?;
        
        // 转换为 base64 返回
        let base64_result = base64::engine::general_purpose::STANDARD.encode(&processed_data);
        let elapsed = start.elapsed();
        println!("[图片处理] 像素化完成：{:?}ms", elapsed.as_millis());
        Ok(format!("data:image/png;base64,{}", base64_result))
    } else {
        // 直接返回原图的 base64
        let base64_result = base64::engine::general_purpose::STANDARD.encode(&img_data);
        let elapsed = start.elapsed();
        println!("[图片处理] 原图转换完成：{:?}ms", elapsed.as_millis());
        Ok(format!("data:image/png;base64,{}", base64_result))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            settings: Mutex::new(AppSettings::default()),
            media_cache: Mutex::new(MediaCache::default()),
        })
        .invoke_handler(tauri::generate_handler![
            control_media,
            show_main_window,
            show_settings_window,
            open_floating_window,
            close_floating_window,
            get_settings,
            save_settings,
            check_fullscreen_app,
            sync_window_bounds,
            get_player_weights,
            set_player_weight,
            emit_event,
            save_floating_window_position,
            get_floating_window_position,
            set_floating_window_resizable,
            open_application,
            // 缓存相关 API
            get_cached_media,
            download_and_cache,
            clear_cache,
            get_cache_stats,
            get_cache_directory,
            set_cache_directory,
            pick_cache_directory,
            // 开机启动相关 API
            set_auto_start,
            get_auto_start,
            // 像素化处理 API
            pixelate_cover,
            // 图片处理 API（支持像素化）
            process_image,
            // 颜色提取 API
            extract_dominant_color,
            // UI 显示控制 API
            set_hide_settings_button,
            set_hide_monitor_selector,
            set_hide_floating_window,
            set_expanded_corner_radius,
            // 系统毛玻璃 API
            set_window_vibrancy,
        ])
        .setup(|app| {
            // 初始化缓存系统
            init_cache_system(&app.handle())?;
            
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
            start_audio_visualizer(app.handle().clone());

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
