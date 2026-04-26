//! 颜色处理服务模块
//!
//! 提供图片主色调提取功能

use crate::error::{AppError, AppResult};
use image::imageops::FilterType;

/// 从图片中提取主色调
///
/// 使用颜色量化算法提取最具代表性的颜色
/// 优先选择饱和度较高的颜色
///
/// # 返回
/// RGB 元组 (r, g, b)
pub fn extract_dominant_color(image_path: &str) -> AppResult<(u8, u8, u8)> {
    let img_data = load_image_data(image_path)?;
    
    if img_data.is_empty() {
        return Ok((60, 80, 100)); // 默认颜色
    }
    
    let img = image::load_from_memory(&img_data)
        .map_err(|e| AppError::parse(format!("无法加载图片：{}", e)))?;
    
    // 缩小图片以提高处理速度
    let resized = img.resize_exact(80, 80, FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    
    // 颜色量化统计
    let mut color_count = std::collections::HashMap::new();
    for pixel in rgba.pixels() {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        
        // 跳过透明和黑白像素
        if a < 128 || (r == 0 && g == 0 && b == 0) || (r == 255 && g == 255 && b == 255) {
            continue;
        }
        
        // 量化颜色（减少颜色数量）
        let qr = r / 12 * 12;
        let qg = g / 12 * 12;
        let qb = b / 12 * 12;
        
        *color_count.entry((qr, qg, qb)).or_insert(0) += 1;
    }
    
    // 按饱和度和出现频率排序
    let mut colors: Vec<_> = color_count.into_iter().collect();
    colors.sort_by(|a, b| {
        let ((r1, g1, b1), c1) = a;
        let ((r2, g2, b2), c2) = b;
        
        // 计算饱和度
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
        
        // 综合评分：饱和度 * log(出现次数)
        let score1 = sat1 * (*c1 as f32).ln();
        let score2 = sat2 * (*c2 as f32).ln();
        
        score2.partial_cmp(&score1).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // 返回最佳颜色（稍微提亮）
    if let Some(((r, g, b), _)) = colors.first() {
        let rr = r.saturating_add(12).min(255);
        let gg = g.saturating_add(12).min(255);
        let bb = b.saturating_add(12).min(255);
        Ok((rr, gg, bb))
    } else {
        Ok((40, 50, 60)) // 备用默认颜色
    }
}

/// RGB 转十六进制颜色字符串
///
/// # 示例
/// ```
/// let hex = rgb_to_hex(255, 128, 0);
/// assert_eq!(hex, "#ff8000");
/// ```
pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// 加载图片数据
///
/// 支持多种路径格式：
/// - 绝对路径
/// - 相对于 Packages 目录的路径
/// - 相对于 AppData 目录的路径
fn load_image_data(image_path: &str) -> AppResult<Vec<u8>> {
    let path = std::path::Path::new(image_path);
    
    // 尝试直接读取
    if path.exists() {
        return std::fs::read(path)
            .map_err(|e| AppError::io(format!("读取图片失败：{}", e)));
    }
    
    // 尝试备用路径
    let appdata = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .unwrap_or_default();
    
    let fallbacks = vec![
        path.to_path_buf(),
        std::path::PathBuf::from(&appdata).join("Packages").join(image_path),
        std::path::PathBuf::from(&appdata).join(image_path.replace("\\", "/")),
    ];
    
    for fallback_path in &fallbacks {
        if fallback_path.exists() {
            return std::fs::read(fallback_path)
                .map_err(|e| AppError::io(format!("读取图片失败：{}", e)));
        }
    }
    
    Err(AppError::not_found(format!("图片不存在：{}", image_path)))
}
