//! 缓存服务模块
//!
//! 提供媒体文件（封面、MV）的本地缓存管理

use crate::error::{AppError, AppResult};
use crate::models::{CacheMetadata, CacheStats};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::Manager;

/// 全局缓存目录路径
static CACHE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// 全局缓存元数据列表
static CACHE_METADATA: Mutex<Option<Vec<CacheMetadata>>> = Mutex::new(None);
static LAST_METADATA_FLUSH_AT: AtomicU64 = AtomicU64::new(0);

const MAX_CACHE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_IMAGE_CACHE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_VIDEO_CACHE_BYTES: u64 = 384 * 1024 * 1024;
const METADATA_FLUSH_INTERVAL_SECS: u64 = 2;

fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn persist_metadata(cache_dir: &PathBuf, metadata: &[CacheMetadata]) -> AppResult<()> {
    let metadata_file = cache_dir.join("metadata.json");
    let content = serde_json::to_string_pretty(metadata).map_err(AppError::Serialization)?;
    fs::write(&metadata_file, content)
        .map_err(|e| AppError::cache(format!("无法写入元数据：{}", e)))?;
    LAST_METADATA_FLUSH_AT.store(now_seconds(), Ordering::Relaxed);
    Ok(())
}

fn persist_metadata_if_due(cache_dir: &PathBuf, metadata: &[CacheMetadata]) -> AppResult<()> {
    let now = now_seconds();
    let last = LAST_METADATA_FLUSH_AT.load(Ordering::Relaxed);
    if now.saturating_sub(last) >= METADATA_FLUSH_INTERVAL_SECS {
        persist_metadata(cache_dir, metadata)?;
    }
    Ok(())
}

fn is_image(content_type: &str) -> bool {
    content_type.starts_with("image/")
}

fn is_over_cache_limit(metadata: &[CacheMetadata]) -> bool {
    let total = metadata.iter().map(|item| item.size).sum::<u64>();
    let image_total = metadata
        .iter()
        .filter(|item| is_image(&item.content_type))
        .map(|item| item.size)
        .sum::<u64>();
    let video_total = total.saturating_sub(image_total);
    total > MAX_CACHE_BYTES
        || image_total > MAX_IMAGE_CACHE_BYTES
        || video_total > MAX_VIDEO_CACHE_BYTES
}

fn evict_to_limits(_cache_dir: &PathBuf, metadata: &mut Vec<CacheMetadata>) {
    while is_over_cache_limit(metadata) {
        let total = metadata.iter().map(|item| item.size).sum::<u64>();
        let image_total = metadata
            .iter()
            .filter(|item| is_image(&item.content_type))
            .map(|item| item.size)
            .sum::<u64>();
        let video_total = total.saturating_sub(image_total);
        let image_over = image_total > MAX_IMAGE_CACHE_BYTES;
        let video_over = video_total > MAX_VIDEO_CACHE_BYTES;

        let oldest_index = metadata
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if image_over {
                    is_image(&item.content_type)
                } else if video_over {
                    !is_image(&item.content_type)
                } else {
                    true
                }
            })
            .min_by_key(|(_, item)| (item.last_accessed_at, item.created_at))
            .map(|(index, _)| index);

        let Some(index) = oldest_index else { break };
        let removed = metadata.remove(index);
        if let Err(error) = fs::remove_file(&removed.file_path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                tracing::debug!("[Cache] 清理文件失败 {}: {}", removed.file_path, error);
            }
        }
    }
}

/// 初始化缓存系统
///
/// 在应用启动时调用，执行以下操作：
/// 1. 确定缓存目录（自定义或默认）
/// 2. 创建缓存目录（如果不存在）
/// 3. 加载现有缓存的元数据
pub fn init_cache_system(app_handle: &tauri::AppHandle) -> AppResult<()> {
    // Keep portable installs self-contained: artwork and MV previews live next
    // to the executable by default. If that location is read-only (for
    // example a manually copied binary under Program Files), gracefully fall
    // back to the per-user application cache.
    let install_cache = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("cache").join("media")));
    let fallback_cache = app_handle
        .path()
        .app_cache_dir()
        .map_err(|e| AppError::cache(format!("无法获取缓存目录：{}", e)))?
        .join("media_cache");
    let cache_dir = match install_cache {
        Some(path) if fs::create_dir_all(&path).is_ok() => path,
        _ => fallback_cache,
    };

    // 创建缓存目录
    fs::create_dir_all(&cache_dir)
        .map_err(|e| AppError::cache(format!("无法创建缓存目录：{}", e)))?;

    // 加载元数据
    let metadata_file = cache_dir.join("metadata.json");
    let mut metadata: Vec<CacheMetadata> = if metadata_file.exists() {
        let content = fs::read_to_string(&metadata_file)
            .map_err(|e| AppError::cache(format!("无法读取元数据：{}", e)))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    let now = now_seconds();
    for item in &mut metadata {
        if item.last_accessed_at == 0 {
            item.last_accessed_at = item.created_at.max(now);
        }
    }
    evict_to_limits(&cache_dir, &mut metadata);

    // 存储到全局状态
    {
        let mut cache_dir_global = CACHE_DIR
            .lock()
            .map_err(|_| AppError::lock("无法锁定缓存目录"))?;
        *cache_dir_global = Some(cache_dir.clone());
    }

    {
        let mut metadata_global = CACHE_METADATA
            .lock()
            .map_err(|_| AppError::lock("无法锁定缓存元数据"))?;
        *metadata_global = Some(metadata);
    }

    if let Ok(metadata_global) = CACHE_METADATA.lock() {
        if let Some(metadata) = metadata_global.as_ref() {
            let _ = persist_metadata(&cache_dir, metadata);
        }
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
        LAST_METADATA_FLUSH_AT.store(0, Ordering::Relaxed);
        persist_metadata(&cache_dir, &[])?;
    }

    Ok(())
}

/// 获取缓存统计信息
///
/// 返回缓存大小、文件数量等统计数据
pub fn get_cache_stats() -> AppResult<CacheStats> {
    let (total_size, total_files, mv_count, cover_count) = {
        let metadata_global = CACHE_METADATA
            .lock()
            .map_err(|_| AppError::lock("无法锁定缓存元数据"))?;
        let metadata = metadata_global
            .as_ref()
            .ok_or_else(|| AppError::cache("缓存元数据未初始化"))?;

        (
            metadata.iter().map(|m| m.size).sum::<u64>(),
            metadata.len() as u32,
            metadata
                .iter()
                .filter(|m| m.content_type.starts_with("video"))
                .count() as u32,
            metadata
                .iter()
                .filter(|m| m.content_type.starts_with("image"))
                .count() as u32,
        )
    };

    let cache_dir = CACHE_DIR
        .lock()
        .map_err(|_| AppError::lock("无法锁定缓存目录"))?
        .clone()
        .unwrap_or_default();

    Ok(CacheStats {
        total_size_mb: total_size as f64 / (1024.0 * 1024.0),
        total_files,
        mv_count,
        cover_count,
        cache_directory: cache_dir.to_string_lossy().to_string(),
    })
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

    let cache_dir = CACHE_DIR
        .lock()
        .map_err(|_| AppError::lock("无法锁定缓存目录"))?
        .clone()
        .ok_or_else(|| AppError::cache("缓存系统未初始化"))?;
    let mut metadata_global = CACHE_METADATA
        .lock()
        .map_err(|_| AppError::lock("无法锁定缓存元数据"))?;
    let metadata = metadata_global
        .as_mut()
        .ok_or_else(|| AppError::cache("缓存元数据未初始化"))?;

    // 查找缓存记录
    if let Some(meta) = metadata.iter_mut().find(|m| m.key == key) {
        if PathBuf::from(&meta.file_path).exists() {
            meta.last_accessed_at = now_seconds();
            let result = meta.file_path.clone();
            persist_metadata_if_due(&cache_dir, metadata)?;
            return Ok(Some(result));
        }
    }

    Ok(None)
}

/// 下载并缓存媒体文件
///
/// 如果文件已缓存，直接返回路径
/// 否则下载、保存并返回路径
pub async fn download_and_cache(url: &str, content_type: &str) -> AppResult<String> {
    let parsed = reqwest::Url::parse(url).map_err(|_| AppError::business(3003, "无效的 URL"))?;
    match parsed.scheme() {
        "https" => {}
        "http" => {}
        _ => return Err(AppError::business(3003, "仅支持 HTTP/HTTPS 协议")),
    }
    if let Some(host) = parsed.host_str() {
        if host == "localhost"
            || host == "127.0.0.1"
            || host == "::1"
            || host.starts_with("169.254.")
        {
            return Err(AppError::business(3003, "不允许访问内部地址"));
        }
    }

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
        .map_err(|e| AppError::network(format!("下载失败：{}", e)))?
        .error_for_status()
        .map_err(|e| AppError::network(format!("下载响应失败：{}", e)))?;

    const MAX_MEDIA_BYTES: u64 = 128 * 1024 * 1024;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_MEDIA_BYTES)
    {
        return Err(AppError::business(3004, "媒体文件超过 128 MB"));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::network(format!("读取数据失败：{}", e)))?;
    if bytes.len() as u64 > MAX_MEDIA_BYTES {
        return Err(AppError::business(3004, "媒体文件超过 128 MB"));
    }

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
            let old = metadata.remove(pos);
            if old.file_path != file_path.to_string_lossy().as_ref() {
                let _ = fs::remove_file(old.file_path);
            }
        }

        // 添加新记录
        let now = now_seconds();
        metadata.push(CacheMetadata {
            key,
            file_path: file_path.to_string_lossy().to_string(),
            created_at: now,
            last_accessed_at: now,
            size: content.len() as u64,
            content_type: content_type.to_string(),
        });

        evict_to_limits(&cache_dir, metadata);

        // 持久化元数据
        persist_metadata(&cache_dir, metadata)?;
    }

    Ok(file_path.to_string_lossy().to_string())
}
