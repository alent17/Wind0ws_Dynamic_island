//! 缓存相关数据模型
//!
//! 定义缓存元数据和统计信息

use serde::{Deserialize, Serialize};

/// 缓存元数据
///
/// 记录缓存文件的详细信息
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheMetadata {
    /// 缓存键（URL 哈希值）
    pub key: String,
    /// 文件路径
    pub file_path: String,
    /// 创建时间戳（毫秒）
    pub created_at: u64,
    /// 文件大小（字节）
    pub size: u64,
    /// 内容类型（如 image/jpeg, video/mp4）
    pub content_type: String,
}

/// 缓存统计信息
///
/// 用于显示缓存使用情况
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStats {
    /// 总缓存大小（MB）
    pub total_size_mb: f64,
    /// 缓存文件总数
    pub total_files: u32,
    /// MV 缓存数量
    pub mv_count: u32,
    /// 封面缓存数量
    pub cover_count: u32,
    /// 缓存目录路径
    pub cache_directory: String,
}
