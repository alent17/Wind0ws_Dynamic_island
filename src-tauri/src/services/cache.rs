//! 缓存服务模块
//!
//! 提供媒体文件（封面、MV）的本地缓存管理

use crate::error::{AppError, AppResult};
use crate::models::{CacheMetadata, CacheStats};
use crate::services::read_settings_file;
use std::path::PathBuf;
use std::fs;
use std::sync::Mutex;
use tauri::Manager;

/// 全局缓存目录路径
static CACHE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// 全局缓存元数据列表
static CACHE_METADATA: Mutex<Option<Vec<CacheMetadata>>> = Mutex::new(None);

/// 初始化缓存系统
///
/// 在应用启动时调用，执行以下操作：
/// 1. 确定缓存目录（自定义或默认）
/// 2. 创建缓存目录（如果不存在）
/// 3. 加载现有缓存的元数据
pub fn init_cache_system(app_handle: &tauri::AppHandle) -> AppResult<()> {
    let settings = read_settings_file(app_handle).unwrap_or_default();
    
    // 确定缓存目录
    let cache_dir = if let Some(custom_dir) = &settings.cache_directory {
        PathBuf::from(custom_dir)
    } else {
        app_handle
            .path()
            .app_cache_dir()
            .map_err(|e| AppError::cache(format!("无法获取缓存目录：{}", e)))?
            .join("media_cache")
    };
    
    // 创建缓存目录
    fs::create_dir_all(&cache_dir)
        .map_err(|e| AppError::cache(format!("无法创建缓存目录：{}", e)))?;
    
    // 加载元数据
    let metadata_file = cache_dir.join("metadata.json");
    let metadata = if metadata_file.exists() {
        let content = fs::read_to_string(&metadata_file)
            .map_err(|e| AppError::cache(format!("无法读取元数据：{}", e)))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    // 存储到全局状态
    {
        let mut cache_dir_global = CACHE_DIR
            .lock()
            .map_err(|_| AppError::lock("无法锁定缓存目录"))?;
        *cache_dir_global = Some(cache_dir);
    }
    
    {
        let mut metadata_global = CACHE_METADATA
            .lock()
            .map_err(|_| AppError::lock("无法锁定缓存元数据"))?;
        *metadata_global = Some(metadata);
    }
    
    Ok(())
}

/// 清空缓存
///
/// 删除所有缓存文件和元数据
pub fn clear_cache() -> AppResult<()> {
    let cache_dir = CACHE_DIR
        .lock()
        .map_err(|_| AppError::lock("无法锁定缓存目录"))?
        .clone()
        .ok_or_else(|| AppError::cache("缓存系统未初始化"))?;
    
    if cache_dir.exists() {
        // 删除并重建目录
        fs::remove_dir_all(&cache_dir)
            .map_err(|e| AppError::cache(format!("无法清理缓存：{}", e)))?;
        
        fs::create_dir_all(&cache_dir)
            .map_err(|e| AppError::cache(format!("无法创建缓存目录：{}", e)))?;
        
        // 清空元数据
        {
            let mut metadata_global = CACHE_METADATA
                .lock()
                .map_err(|_| AppError::lock("无法锁定缓存元数据"))?;
            *metadata_global = Some(Vec::new());
        }
    }
    
    Ok(())
}

/// 获取缓存统计信息
///
/// 返回缓存大小、文件数量等统计数据
pub fn get_cache_stats() -> AppResult<CacheStats> {
    let metadata_global = CACHE_METADATA
        .lock()
        .map_err(|_| AppError::lock("无法锁定缓存元数据"))?;
    let metadata = metadata_global
        .as_ref()
        .ok_or_else(|| AppError::cache("缓存元数据未初始化"))?;
    
    // 计算统计数据
    let total_size: u64 = metadata.iter().map(|m| m.size).sum();
    let mv_count = metadata.iter().filter(|m| m.content_type.starts_with("video")).count();
    let cover_count = metadata.iter().filter(|m| m.content_type.starts_with("image")).count();
    
    let cache_dir = CACHE_DIR
        .lock()
        .map_err(|_| AppError::lock("无法锁定缓存目录"))?
        .clone()
        .unwrap_or_default();
    
    Ok(CacheStats {
        total_size_mb: total_size as f64 / (1024.0 * 1024.0),
        total_files: metadata.len() as u32,
        mv_count: mv_count as u32,
        cover_count: cover_count as u32,
        cache_directory: cache_dir.to_string_lossy().to_string(),
    })
}

/// 获取缓存目录路径
pub fn get_cache_directory() -> AppResult<String> {
    let cache_dir = CACHE_DIR
        .lock()
        .map_err(|_| AppError::lock("无法锁定缓存目录"))?
        .clone()
        .ok_or_else(|| AppError::cache("缓存系统未初始化"))?;
    
    Ok(cache_dir.to_string_lossy().to_string())
}

/// 设置缓存目录
///
/// 切换到新的缓存目录（不会迁移现有缓存）
pub fn set_cache_directory(_app_handle: &tauri::AppHandle, new_path: &str) -> AppResult<()> {
    let new_cache_dir = PathBuf::from(new_path);
    
    if !new_cache_dir.exists() {
        fs::create_dir_all(&new_cache_dir)
            .map_err(|e| AppError::cache(format!("创建目录失败：{}", e)))?;
    }
    
    {
        let mut cache_dir_global = CACHE_DIR
            .lock()
            .map_err(|_| AppError::lock("无法锁定缓存目录"))?;
        *cache_dir_global = Some(new_cache_dir);
    }
    
    Ok(())
}

/// 获取已缓存的媒体文件路径
///
/// 如果文件已缓存，返回本地路径；否则返回 None
pub fn get_cached_media(url: &str) -> AppResult<Option<String>> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // 计算 URL 哈希作为缓存键
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let key = format!("{:x}", hasher.finish());
    
    let metadata_global = CACHE_METADATA
        .lock()
        .map_err(|_| AppError::lock("无法锁定缓存元数据"))?;
    let metadata = metadata_global
        .as_ref()
        .ok_or_else(|| AppError::cache("缓存元数据未初始化"))?;
    
    // 查找缓存记录
    if let Some(meta) = metadata.iter().find(|m| m.key == key) {
        if PathBuf::from(&meta.file_path).exists() {
            return Ok(Some(meta.file_path.clone()));
        }
    }
    
    Ok(None)
}

/// 下载并缓存媒体文件
///
/// 如果文件已缓存，直接返回路径
/// 否则下载、保存并返回路径
pub async fn download_and_cache(url: &str, content_type: &str) -> AppResult<String> {
    // 检查是否已缓存
    if let Some(cached_path) = get_cached_media(url)? {
        return Ok(cached_path);
    }
    
    // 下载文件
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::network(format!("下载失败：{}", e)))?;
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::network(format!("读取数据失败：{}", e)))?;
    
    save_cache_file(url, &bytes, content_type)
}

/// 保存缓存文件
///
/// 将数据保存到缓存目录，并更新元数据
fn save_cache_file(url: &str, content: &[u8], content_type: &str) -> AppResult<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let cache_dir = CACHE_DIR
        .lock()
        .map_err(|_| AppError::lock("无法锁定缓存目录"))?
        .clone()
        .ok_or_else(|| AppError::cache("缓存系统未初始化"))?;
    
    // 生成缓存键和文件名
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let key = format!("{:x}", hasher.finish());
    
    // 根据内容类型确定扩展名
    let extension = match content_type {
        "video/mp4" | "video/webm" => "mp4",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        _ => "dat",
    };
    
    let file_path = cache_dir.join(format!("{}.{}", key, extension));
    
    // 写入文件
    fs::write(&file_path, content)
        .map_err(|e| AppError::cache(format!("无法写入缓存文件：{}", e)))?;
    
    // 更新元数据
    {
        let mut metadata_global = CACHE_METADATA
            .lock()
            .map_err(|_| AppError::lock("无法锁定缓存元数据"))?;
        let metadata = metadata_global
            .as_mut()
            .ok_or_else(|| AppError::cache("缓存元数据未初始化"))?;
        
        // 移除旧记录（如果存在）
        if let Some(pos) = metadata.iter().position(|m| m.key == key) {
            metadata.remove(pos);
        }
        
        // 添加新记录
        metadata.push(CacheMetadata {
            key,
            file_path: file_path.to_string_lossy().to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            size: content.len() as u64,
            content_type: content_type.to_string(),
        });
        
        // 持久化元数据
        let metadata_file = cache_dir.join("metadata.json");
        let content = serde_json::to_string_pretty(&*metadata)
            .map_err(|e| AppError::Serialization(e))?;
        fs::write(&metadata_file, content)
            .map_err(|e| AppError::cache(format!("无法写入元数据：{}", e)))?;
    }
    
    Ok(file_path.to_string_lossy().to_string())
}
