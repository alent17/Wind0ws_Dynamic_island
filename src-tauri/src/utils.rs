//! 通用工具函数模块

use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose, Engine as _};

/// 校验路径安全性
fn validate_path(image_path: &str) -> AppResult<()> {
    let path = std::path::Path::new(image_path);
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(AppError::business(3001, "路径包含非法的目录遍历"));
        }
    }
    let allowed_extensions = ["jpg", "jpeg", "png", "bmp", "gif", "webp"];
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if !allowed_extensions.contains(&ext.to_lowercase().as_str()) {
            return Err(AppError::business(3002, "不支持的图片格式"));
        }
    }
    Ok(())
}

/// 加载图片数据
///
/// 支持多种路径格式：
/// - 绝对路径
/// - 相对于 Packages 目录的路径
/// - 相对于 AppData 目录的路径
pub fn load_image_data(image_path: &str) -> AppResult<Vec<u8>> {
    if let Some(encoded) = image_path
        .strip_prefix("data:image/")
        .and_then(|value| value.split_once(","))
        .filter(|(metadata, _)| metadata.ends_with(";base64"))
        .map(|(_, encoded)| encoded)
    {
        return general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| AppError::parse(format!("无法解码图片数据：{}", e)));
    }

    validate_path(image_path)?;

    let path = std::path::Path::new(image_path);

    if path.exists() {
        return std::fs::read(path).map_err(|e| AppError::io(format!("读取图片失败：{}", e)));
    }

    let appdata = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .unwrap_or_default();

    let fallbacks = vec![
        path.to_path_buf(),
        std::path::PathBuf::from(&appdata)
            .join("Packages")
            .join(image_path),
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

#[cfg(test)]
mod tests {
    use super::load_image_data;

    #[test]
    fn loads_base64_image_data_urls() {
        assert_eq!(
            load_image_data("data:image/png;base64,AQID").expect("decoded image bytes"),
            vec![1, 2, 3]
        );
    }
}
