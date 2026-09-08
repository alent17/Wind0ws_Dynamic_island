//! 缓存相关命令模块
//!
//! 提供缓存管理、目录选择和媒体缓存命令

use crate::error::AppResult;
use crate::models::CacheStats;

/// 清空缓存
#[tauri::command]
pub fn clear_cache() -> AppResult<()> {
    crate::services::cache::clear_cache()
}

/// 获取缓存统计信息
#[tauri::command]
pub fn get_cache_stats() -> AppResult<CacheStats> {
    crate::services::cache::get_cache_stats()
}

/// 获取已缓存的媒体文件路径
///
/// 如果文件已缓存，返回本地路径；否则返回 None
#[tauri::command]
pub fn get_cached_media(url: String) -> AppResult<Option<String>> {
    crate::services::cache::get_cached_media(&url)
}

/// 下载并缓存媒体文件
///
/// 如果文件已缓存，直接返回路径
/// 否则下载、保存并返回路径
#[tauri::command]
pub async fn download_and_cache(url: String, content_type: String) -> AppResult<String> {
    crate::services::cache::download_and_cache(&url, &content_type).await
}
