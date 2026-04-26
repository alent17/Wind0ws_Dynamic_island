//! 图片处理服务模块
//!
//! 提供图片像素化等特效处理功能

use crate::error::{AppError, AppResult};
use image::{ImageFormat, GenericImageView};
use std::io::Cursor;
use base64::{engine::general_purpose, Engine};

/// 处理图片（可选像素化效果）
///
/// # 参数
/// - `image_path`: 图片路径
/// - `enable_pixel_art`: 是否启用像素化效果
///
/// # 返回
/// Base64 编码的 PNG 图片数据（带 data URI 前缀）
pub async fn process_image(image_path: &str, enable_pixel_art: bool) -> AppResult<String> {
    let img_data = load_image_data(image_path)?;
    
    if img_data.is_empty() {
        return Err(AppError::parse("图片数据为空"));
    }
    
    let _img = image::load_from_memory(&img_data)
        .map_err(|e| AppError::parse(format!("无法加载图片：{}", e)))?;
    
    if enable_pixel_art {
        let processed_data = pixelate_image_advanced(&img_data, 12, 32)?;
        let base64_result = general_purpose::STANDARD.encode(&processed_data);
        Ok(format!("data:image/png;base64,{}", base64_result))
    } else {
        let base64_result = general_purpose::STANDARD.encode(&img_data);
        Ok(format!("data:image/png;base64,{}", base64_result))
    }
}

/// 像素化封面图片
///
/// # 参数
/// - `image_path`: 图片路径
/// - `pixel_size`: 像素块大小
///
/// # 返回
/// Base64 编码的 PNG 图片数据
pub fn pixelate_cover(image_path: &str, pixel_size: u32) -> AppResult<String> {
    let img_data = load_image_data(image_path)?;
    
    if img_data.is_empty() {
        return Err(AppError::parse("图片数据为空"));
    }
    
    let img = image::load_from_memory(&img_data)
        .map_err(|e| AppError::parse(format!("无法加载图片：{}", e)))?;
    
    let pixelated = pixelate_image(&img, pixel_size);
    
    let mut buffer = Cursor::new(Vec::new());
    pixelated
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| AppError::io(format!("写入图片失败：{}", e)))?;
    
    let base64_result = general_purpose::STANDARD.encode(&buffer.into_inner());
    Ok(format!("data:image/png;base64,{}", base64_result))
}

/// 像素化图片
///
/// 将图片分割成 pixel_size x pixel_size 的块，
/// 每个块填充该区域的平均颜色
fn pixelate_image(img: &image::DynamicImage, pixel_size: u32) -> image::DynamicImage {
    let (width, height) = img.dimensions();
    let mut result = img.to_rgba8();
    
    for y in (0..height).step_by(pixel_size as usize) {
        for x in (0..width).step_by(pixel_size as usize) {
            // 计算块内平均颜色
            let mut r_sum = 0u32;
            let mut g_sum = 0u32;
            let mut b_sum = 0u32;
            let mut count = 0u32;
            
            for dy in 0..pixel_size {
                for dx in 0..pixel_size {
                    let px = x + dx;
                    let py = y + dy;
                    if px < width && py < height {
                        let pixel = img.get_pixel(px, py);
                        r_sum += pixel[0] as u32;
                        g_sum += pixel[1] as u32;
                        b_sum += pixel[2] as u32;
                        count += 1;
                    }
                }
            }
            
            // 填充块
            if count > 0 {
                let r_avg = (r_sum / count) as u8;
                let g_avg = (g_sum / count) as u8;
                let b_avg = (b_sum / count) as u8;
                
                for dy in 0..pixel_size {
                    for dx in 0..pixel_size {
                        let px = x + dx;
                        let py = y + dy;
                        if px < width && py < height {
                            let pixel = result.get_pixel_mut(px, py);
                            pixel[0] = r_avg;
                            pixel[1] = g_avg;
                            pixel[2] = b_avg;
                        }
                    }
                }
            }
        }
    }
    
    image::DynamicImage::ImageRgba8(result)
}

/// 高级像素化处理
///
/// 支持自定义像素块大小和调色板大小
fn pixelate_image_advanced(img_data: &[u8], pixel_size: u32, _palette_size: u32) -> AppResult<Vec<u8>> {
    let img = image::load_from_memory(img_data)
        .map_err(|e| AppError::parse(format!("无法加载图片：{}", e)))?;
    
    let pixelated = pixelate_image(&img, pixel_size);
    
    let mut buffer = Cursor::new(Vec::new());
    pixelated
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| AppError::io(format!("写入图片失败：{}", e)))?;
    
    Ok(buffer.into_inner())
}

/// 加载图片数据
///
/// 支持多种路径格式：
/// - 绝对路径
/// - 相对于 Packages 目录的路径
/// - 相对于 AppData 目录的路径
fn load_image_data(image_path: &str) -> AppResult<Vec<u8>> {
    let path = std::path::Path::new(image_path);
    
    if path.exists() {
        return std::fs::read(path)
            .map_err(|e| AppError::io(format!("读取图片失败：{}", e)));
    }
    
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
