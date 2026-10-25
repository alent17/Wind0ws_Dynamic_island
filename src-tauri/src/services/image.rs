//! 图片处理服务模块
//!
//! 提供图片像素化等特效处理功能

use super::image_budget::{self, ImageWork, MAX_ARTWORK_EDGE};
use crate::error::{AppError, AppResult};
use crate::utils::load_image_data;

fn data_url(image: &image::DynamicImage) -> AppResult<String> {
    let bytes = image_budget::encode_png(image)?;
    Ok(image_budget::artwork_data_uri(&bytes, "image/png"))
}

pub fn process_image(image_path: &str, enable_pixel_art: bool) -> AppResult<String> {
    let _work = ImageWork::enter()?;
    let bytes = load_image_data(image_path)?;
    process_loaded_image(&bytes, enable_pixel_art)
}
#[cfg(test)]
fn process_image_bytes(bytes: &[u8], enable_pixel_art: bool) -> AppResult<String> {
    let _work = ImageWork::enter()?;
    process_loaded_image(bytes, enable_pixel_art)
}
fn process_loaded_image(bytes: &[u8], enable_pixel_art: bool) -> AppResult<String> {
    let image = image_budget::fit(image_budget::decode(bytes)?);
    let image = if enable_pixel_art {
        pixelate_image(image, 12)
    } else {
        image
    };
    data_url(&image)
}

pub fn pixelate_cover(image_path: &str, pixel_size: u32) -> AppResult<String> {
    if pixel_size == 0 || pixel_size > MAX_ARTWORK_EDGE {
        return Err(AppError::business(3006, "pixel_size 必须在 1 到 1280 之间"));
    }
    let _work = ImageWork::enter()?;
    let bytes = load_image_data(image_path)?;
    let image = image_budget::fit(image_budget::decode(&bytes)?);
    data_url(&pixelate_image(image, pixel_size))
}

/// 像素化图片
///
/// 将图片分割成 pixel_size x pixel_size 的块，
/// 每个块填充该区域的平均颜色
fn pixelate_image(img: image::DynamicImage, pixel_size: u32) -> image::DynamicImage {
    // Consume RGBA input instead of cloning a full-size output alongside it.
    let mut result = img.into_rgba8();
    let (width, height) = result.dimensions();

    for y in (0..height).step_by(pixel_size as usize) {
        for x in (0..width).step_by(pixel_size as usize) {
            // 计算块内平均颜色
            let mut r_sum = 0u64;
            let mut g_sum = 0u64;
            let mut b_sum = 0u64;
            let mut count = 0u64;

            for py in y..y.saturating_add(pixel_size).min(height) {
                for px in x..x.saturating_add(pixel_size).min(width) {
                    if px < width && py < height {
                        let pixel = result.get_pixel(px, py);
                        r_sum += pixel[0] as u64;
                        g_sum += pixel[1] as u64;
                        b_sum += pixel[2] as u64;
                        count += 1;
                    }
                }
            }

            // 填充块
            if let (Some(r_avg), Some(g_avg), Some(b_avg)) = (
                r_sum.checked_div(count),
                g_sum.checked_div(count),
                b_sum.checked_div(count),
            ) {
                let r_avg = r_avg as u8;
                let g_avg = g_avg as u8;
                let b_avg = b_avg as u8;

                for py in y..y.saturating_add(pixel_size).min(height) {
                    for px in x..x.saturating_add(pixel_size).min(width) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine};
    use image::GenericImageView;

    fn reference_pixelation(img: &image::DynamicImage, block: u32) -> image::RgbaImage {
        let (width, height) = img.dimensions();
        let mut output = img.to_rgba8();
        for y in (0..height).step_by(block as usize) {
            for x in (0..width).step_by(block as usize) {
                let (right, bottom) = ((x + block).min(width), (y + block).min(height));
                let mut sums = [0u64; 3];
                let count = u64::from((right - x) * (bottom - y));
                for py in y..bottom {
                    for px in x..right {
                        let pixel = img.get_pixel(px, py);
                        for channel in 0..3 {
                            sums[channel] += u64::from(pixel[channel]);
                        }
                    }
                }
                for py in y..bottom {
                    for px in x..right {
                        let pixel = output.get_pixel_mut(px, py);
                        for channel in 0..3 {
                            pixel[channel] = (sums[channel] / count) as u8;
                        }
                    }
                }
            }
        }
        output
    }

    #[test]
    fn pixelation_matches_original_for_partial_blocks_formats_and_transparency() {
        let rgba = image::RgbaImage::from_fn(29, 17, |x, y| {
            image::Rgba([
                (x * 7) as u8,
                (y * 13) as u8,
                (x * y) as u8,
                ((x + y) * 5) as u8,
            ])
        });
        let source = image::DynamicImage::ImageRgba8(rgba);
        let sources = [
            source.clone(),
            image::DynamicImage::ImageRgb8(source.to_rgb8()),
            image::DynamicImage::ImageLuma8(source.to_luma8()),
            image::DynamicImage::ImageLumaA8(source.to_luma_alpha8()),
            image::DynamicImage::ImageRgba16(source.to_rgba16()),
        ];
        for source in sources {
            for block in [1, 12, 1280] {
                let expected = reference_pixelation(&source, block);
                let actual = pixelate_image(source.clone(), block).into_rgba8();
                assert_eq!(
                    actual,
                    expected,
                    "format {:?}, block {block}",
                    source.color()
                );
            }
        }
    }

    #[test]
    fn pixelation_reuses_the_rgba_allocation() {
        let pixels = image::RgbaImage::new(1280, 1280);
        let pointer = pixels.as_ptr();
        let output = pixelate_image(image::DynamicImage::ImageRgba8(pixels), 12).into_rgba8();
        assert_eq!(output.as_ptr(), pointer);
        assert_eq!(output.len(), 1280 * 1280 * 4);
    }

    #[test]
    fn ipc_png_roundtrip_keeps_dimensions_and_pixels() {
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(29, 17, |x, y| {
            image::Rgba([(x * 7) as u8, (y * 13) as u8, 80, (x * y) as u8])
        }));
        let bytes = image_budget::encode_png(&image).unwrap();
        for pixelated in [false, true] {
            let uri = process_image_bytes(&bytes, pixelated).unwrap();
            let png = general_purpose::STANDARD
                .decode(uri.strip_prefix("data:image/png;base64,").unwrap())
                .unwrap();
            let actual = image_budget::decode(&png).unwrap().into_rgba8();
            let expected = if pixelated {
                reference_pixelation(&image, 12)
            } else {
                image.to_rgba8()
            };
            assert_eq!(actual, expected);
        }
    }
    #[test]
    fn rejects_zero_and_huge_pixel_sizes_before_loading_a_file() {
        assert!(pixelate_cover("missing.png", 0)
            .unwrap_err()
            .to_string()
            .contains("pixel_size"));
        assert!(pixelate_cover("missing.png", u32::MAX)
            .unwrap_err()
            .to_string()
            .contains("pixel_size"));
    }
    #[test]
    fn clipped_large_block_preserves_average_and_alpha() {
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(2, 1, |x, _| {
            if x == 0 {
                image::Rgba([0, 20, 40, 128])
            } else {
                image::Rgba([200, 40, 60, 255])
            }
        }));
        let result = pixelate_image(image, 1280).to_rgba8();
        assert_eq!(result.get_pixel(0, 0).0, [100, 30, 50, 128]);
        assert_eq!(result.get_pixel(1, 0).0, [100, 30, 50, 255]);
    }
}
