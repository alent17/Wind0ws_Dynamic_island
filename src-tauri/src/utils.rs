//! 通用工具函数模块

use crate::error::{AppError, AppResult};
use crate::services::image_budget::{read_limited, MAX_IMAGE_BYTES, MAX_IMAGE_URI_CHARS};
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
    if image_path.len() > MAX_IMAGE_URI_CHARS {
        return Err(AppError::business(3004, "图片输入字符串超过预算"));
    }
    if let Some(encoded) = image_path
        .strip_prefix("data:image/")
        .and_then(|value| value.split_once(","))
        .filter(|(metadata, _)| metadata.ends_with(";base64"))
        .map(|(_, encoded)| encoded)
    {
        if encoded.len() > MAX_IMAGE_BYTES / 3 * 4 + 4 {
            return Err(AppError::business(3004, "Base64 图片超过预算"));
        }
        let bytes = general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| AppError::parse(format!("无法解码图片数据：{}", e)))?;
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err(AppError::business(3004, "图片超过预算"));
        }
        return Ok(bytes);
    }

    validate_path(image_path)?;

    let path = std::path::Path::new(image_path);

    if path.exists() {
        return read_image_file(path);
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
            return read_image_file(fallback_path);
        }
    }

    Err(AppError::not_found(format!("图片不存在：{}", image_path)))
}

fn read_image_file(path: &std::path::Path) -> AppResult<Vec<u8>> {
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > MAX_IMAGE_BYTES as u64 {
        return Err(AppError::business(3004, "图片文件超过预算"));
    }
    read_limited(file) // Also bounds a file that grows after the metadata check.
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
    #[test]
    fn rejects_oversize_files_before_reading_their_content() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let path = std::env::temp_dir().join(format!("isle-input-{}-{}.png", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(super::MAX_IMAGE_BYTES as u64 + 1).unwrap();
        drop(file);
        assert!(load_image_data(path.to_str().unwrap()).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
