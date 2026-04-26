//! 缓存相关命令模块
//!
//! 提供缓存管理、目录选择和媒体缓存命令

use tauri::{AppHandle, Manager, State};
use crate::error::{AppError, AppResult};
use crate::models::CacheStats;
use crate::state::AppState;
use crate::services::{write_settings_file, read_settings_file};

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

/// 获取缓存目录路径
#[tauri::command]
pub fn get_cache_directory(_app: AppHandle) -> AppResult<String> {
    crate::services::cache::get_cache_directory()
}

/// 设置缓存目录
///
/// 同时更新配置文件中的缓存目录设置
#[tauri::command]
pub fn set_cache_directory(app: AppHandle, new_path: String) -> AppResult<()> {
    crate::services::cache::set_cache_directory(&app, &new_path)?;
    
    let mut settings = read_settings_file(&app).unwrap_or_default();
    settings.cache_directory = Some(new_path);
    write_settings_file(&app, &settings)?;
    
    Ok(())
}

/// 打开目录选择对话框选择缓存目录
#[tauri::command]
pub fn pick_cache_directory(app: AppHandle) -> AppResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    
    let selected_path = app.dialog().file().blocking_pick_folder();
    
    if let Some(path) = selected_path {
        let path_str = path.to_string();
        
        // 更新设置
        let mut settings = read_settings_file(&app).unwrap_or_default();
        settings.cache_directory = Some(path_str.clone());
        write_settings_file(&app, &settings)?;
        
        // 更新缓存系统
        crate::services::cache::set_cache_directory(&app, &path_str)?;
        
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
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
