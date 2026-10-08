//! 缓存服务模块
//!
//! 提供媒体文件（封面、MV）的本地缓存管理

use crate::error::{AppError, AppResult};
use crate::models::{CacheMetadata, CacheStats};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
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
    let total = metadata
        .iter()
        .map(|item| item.size)
        .fold(0u64, u64::saturating_add);
    let image_total = metadata
        .iter()
        .filter(|item| is_image(&item.content_type))
        .map(|item| item.size)
        .fold(0u64, u64::saturating_add);
    let video_total = total.saturating_sub(image_total);
    total > MAX_CACHE_BYTES
        || image_total > MAX_IMAGE_CACHE_BYTES
        || video_total > MAX_VIDEO_CACHE_BYTES
}

fn evict_to_limits(_cache_dir: &PathBuf, metadata: &mut Vec<CacheMetadata>) {
    while is_over_cache_limit(metadata) {
        let total = metadata
            .iter()
            .map(|item| item.size)
            .fold(0u64, u64::saturating_add);
        let image_total = metadata
            .iter()
            .filter(|item| is_image(&item.content_type))
            .map(|item| item.size)
            .fold(0u64, u64::saturating_add);
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

pub(crate) fn cleanup_dead_partials(directory: &std::path::Path) {
    let mut processes = sysinfo::System::new();
    processes.refresh_processes();
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let owner = name
                .strip_prefix(".download-")
                .or_else(|| name.strip_prefix(".normalize-"))
                .filter(|_| name.ends_with(".partial"))
                .and_then(|tail| tail.split('-').next())
                .and_then(|pid| pid.parse::<u32>().ok());
            let regular = entry.file_type().is_ok_and(|kind| kind.is_file());
            if regular
                && owner.is_some_and(|pid| processes.process(sysinfo::Pid::from_u32(pid)).is_none())
            {
                let _ = fs::remove_file(entry.path());
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
    let install_cache = std::env::current_exe().ok().and_then(|path| {
        path.parent()
            .map(|parent| parent.join("cache").join("media"))
    });
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

    cleanup_dead_partials(&cache_dir);

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
            metadata
                .iter()
                .map(|m| m.size)
                .fold(0u64, u64::saturating_add),
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

    let hit = metadata
        .iter_mut()
        .find(|entry| entry.key == key)
        .filter(|entry| PathBuf::from(&entry.file_path).exists())
        .map(|entry| {
            entry.last_accessed_at = now_seconds();
            (entry.file_path.clone(), is_image(&entry.content_type))
        });
    if hit.is_some() {
        persist_metadata_if_due(&cache_dir, metadata)?;
    }
    drop(metadata_global);
    let Some((path, artwork)) = hit else {
        return Ok(None);
    };
    if artwork {
        let bytes = super::image_budget::read_limited(fs::File::open(&path)?)?;
        let (width, height) = super::image_budget::dimensions(&bytes)?;
        if width > super::image_budget::MAX_ARTWORK_EDGE
            || height > super::image_budget::MAX_ARTWORK_EDGE
        {
            let normalized = super::image_budget::normalize(&bytes)?;
            let temporary = stage_image(&cache_dir, &normalized)?;
            return save_cache_file(
                url,
                &temporary.0,
                normalized.len() as u64,
                "image/png",
                true,
            )
            .map(Some);
        }
    }
    Ok(Some(path))
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

    let directory = CACHE_DIR
        .lock()
        .map_err(|_| AppError::lock("缓存目录锁失败"))?
        .clone()
        .ok_or_else(|| AppError::cache("缓存系统未初始化"))?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::network(format!("创建下载客户端失败：{e}")))?;
    let limit = if is_image(content_type) {
        super::image_budget::MAX_IMAGE_BYTES as u64
    } else {
        128 * 1024 * 1024
    };
    let mut downloaded = super::http_budget::download(&client, url, &directory, limit).await?;
    let mut stored_type = content_type;
    if is_image(content_type) {
        let bytes = super::image_budget::read_limited(fs::File::open(&downloaded.path)?)?;
        let normalized = super::image_budget::normalize(&bytes)?;
        fs::write(&downloaded.path, &normalized)?;
        downloaded.size = normalized.len() as u64;
        stored_type = "image/png";
    }
    save_cache_file(url, &downloaded.path, downloaded.size, stored_type, false)
}

struct TemporaryImage(PathBuf);
impl Drop for TemporaryImage {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn stage_image(directory: &std::path::Path, bytes: &[u8]) -> AppResult<TemporaryImage> {
    use std::io::Write;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = directory.join(format!(
        ".normalize-{}-{}.partial",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let temporary = TemporaryImage(path);
    file.write_all(bytes)?;
    file.flush()?;
    Ok(temporary)
}

/// 保存缓存文件
///
/// 将数据保存到缓存目录，并更新元数据
fn save_cache_file(
    url: &str,
    temporary: &std::path::Path,
    size: u64,
    content_type: &str,
    replace_existing: bool,
) -> AppResult<String> {
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

    let suffix = if is_image(content_type) { "-1280" } else { "" };
    let file_path = cache_dir.join(format!("{}{suffix}.{}", key, extension));

    // Serialize publication and accounting so concurrent identical URL
    // downloads and recovered orphan files cannot bypass the byte quota.
    {
        let mut metadata_global = CACHE_METADATA
            .lock()
            .map_err(|_| AppError::lock("无法锁定缓存元数据"))?;
        let metadata = metadata_global
            .as_mut()
            .ok_or_else(|| AppError::cache("缓存元数据未初始化"))?;
        if file_path.exists() && !replace_existing {
            if let Some(existing) = metadata
                .iter()
                .find(|entry| entry.key == key && entry.file_path == file_path.to_string_lossy())
            {
                return Ok(existing.file_path.clone());
            }
            // An orphan from a previous interrupted publish belongs to this
            // cache key. Replace it with the newly validated complete file.
        }
        fs::rename(temporary, &file_path)
            .map_err(|e| AppError::cache(format!("无法发布缓存文件：{e}")))?;

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
            size,
            content_type: content_type.to_string(),
        });

        evict_to_limits(&cache_dir, metadata);

        // 持久化元数据
        persist_metadata(&cache_dir, metadata)?;
    }

    Ok(file_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn public_cache_lookup_migrates_an_oversized_existing_image() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let directory = std::env::temp_dir().join(format!(
            "isle-cache-budget-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let original_dir = CACHE_DIR.lock().unwrap().replace(directory.clone());
        let original_metadata = CACHE_METADATA.lock().unwrap().replace(Vec::new());
        let source =
            super::super::image_budget::encode_png(&image::DynamicImage::new_rgb8(2560, 1440))
                .unwrap();
        let temporary = stage_image(&directory, &source).unwrap();
        let path = save_cache_file(
            "https://example.test/legacy-cover",
            &temporary.0,
            source.len() as u64,
            "image/png",
            false,
        )
        .unwrap();
        let bounded = get_cached_media("https://example.test/legacy-cover")
            .unwrap()
            .unwrap();
        assert_eq!(bounded, path);
        assert_eq!(
            super::super::image_budget::dimensions(&fs::read(&bounded).unwrap()).unwrap(),
            (1280, 720)
        );
        let metadata = CACHE_METADATA.lock().unwrap();
        assert_eq!(metadata.as_ref().unwrap().len(), 1);
        assert_eq!(
            metadata.as_ref().unwrap()[0].size,
            fs::metadata(&bounded).unwrap().len()
        );
        drop(metadata);
        *CACHE_DIR.lock().unwrap() = original_dir;
        *CACHE_METADATA.lock().unwrap() = original_metadata;
        for entry in fs::read_dir(&directory).unwrap().flatten() {
            fs::remove_file(entry.path()).unwrap();
        }
        fs::remove_dir(&directory).unwrap();
    }
}
