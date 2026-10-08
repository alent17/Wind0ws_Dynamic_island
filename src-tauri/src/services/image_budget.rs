//! Shared artwork budgets. Encoded input, decoder output and work admission
//! are independent from the disk cache quota.
use crate::error::{AppError, AppResult};
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, Limits};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::sync::{Condvar, Mutex};

pub const MAX_IMAGE_BYTES: usize = 12 * 1024 * 1024;
pub const MAX_IMAGE_URI_CHARS: usize = MAX_IMAGE_BYTES / 3 * 4 + 128;
pub const MAX_IMAGE_EDGE: u32 = 8192;
pub const MAX_IMAGE_PIXELS: u64 = 16 * 1024 * 1024;
pub const MAX_DECODE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_ARTWORK_EDGE: u32 = 1280; // 640 logical px at 2x DPI.

static WORK: (Mutex<(usize, usize)>, Condvar) = (Mutex::new((0, 0)), Condvar::new());
pub struct ImageWork;
impl ImageWork {
    pub fn enter() -> AppResult<Self> {
        let mut state = WORK
            .0
            .lock()
            .map_err(|_| AppError::lock("图片工作锁失败"))?;
        if state.0 >= 2 && state.1 >= 4 {
            return Err(AppError::business(3005, "图片处理队列已满"));
        }
        state.1 += 1;
        while state.0 >= 2 {
            state = WORK
                .1
                .wait(state)
                .map_err(|_| AppError::lock("图片工作等待失败"))?;
        }
        state.1 -= 1;
        state.0 += 1;
        Ok(Self)
    }
}
impl Drop for ImageWork {
    fn drop(&mut self) {
        if let Ok(mut state) = WORK.0.lock() {
            state.0 = state.0.saturating_sub(1);
            WORK.1.notify_one();
        }
    }
}

pub fn checked_stream_size(size: u64) -> AppResult<u32> {
    if size == 0 || size > MAX_IMAGE_BYTES as u64 {
        return Err(AppError::business(3004, "封面字节数超出预算"));
    }
    u32::try_from(size).map_err(|_| AppError::business(3004, "封面大小转换溢出"))
}

pub fn read_limited(mut input: impl Read) -> AppResult<Vec<u8>> {
    let mut bytes = Vec::new();
    input
        .by_ref()
        .take(MAX_IMAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(AppError::business(3004, "图片超过 12 MiB"));
    }
    Ok(bytes)
}

fn reader(bytes: &[u8]) -> AppResult<ImageReader<Cursor<&[u8]>>> {
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
        return Err(AppError::business(3004, "图片输入为空或超出字节预算"));
    }
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| AppError::parse(format!("无法识别图片：{e}")))?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_EDGE);
    limits.max_image_height = Some(MAX_IMAGE_EDGE);
    limits.max_alloc = Some(MAX_DECODE_BYTES);
    reader.limits(limits);
    Ok(reader)
}

pub fn dimensions(bytes: &[u8]) -> AppResult<(u32, u32)> {
    let (width, height) = reader(bytes)?
        .into_dimensions()
        .map_err(|e| AppError::parse(format!("无法读取图片尺寸：{e}")))?;
    if width == 0
        || height == 0
        || width > MAX_IMAGE_EDGE
        || height > MAX_IMAGE_EDGE
        || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS
    {
        return Err(AppError::business(3004, "图片尺寸或像素数超出预算"));
    }
    Ok((width, height))
}

pub fn decode(bytes: &[u8]) -> AppResult<DynamicImage> {
    dimensions(bytes)?; // Header inspection only; exactly one full decode.
    reader(bytes)?
        .decode()
        .map_err(|e| AppError::parse(format!("图片解码失败：{e}")))
}

pub fn fit(img: DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    if width <= MAX_ARTWORK_EDGE && height <= MAX_ARTWORK_EDGE {
        img
    } else {
        img.resize(
            MAX_ARTWORK_EDGE,
            MAX_ARTWORK_EDGE,
            image::imageops::FilterType::Lanczos3,
        )
    }
}

struct Output(Cursor<Vec<u8>>);
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let end = self
            .0
            .position()
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| io::Error::other("图片输出溢出"))?;
        if end > MAX_IMAGE_BYTES as u64 {
            return Err(io::Error::other("图片输出超过预算"));
        }
        self.0.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Seek for Output {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let previous = self.0.position();
        let next = self.0.seek(position)?;
        if next > MAX_IMAGE_BYTES as u64 {
            self.0.set_position(previous);
            return Err(io::Error::other("图片输出位置超过预算"));
        }
        Ok(next)
    }
}
pub fn encode_png(img: &DynamicImage) -> AppResult<Vec<u8>> {
    let mut output = Output(Cursor::new(Vec::new()));
    img.write_to(&mut output, ImageFormat::Png)
        .map_err(|e| AppError::parse(format!("图片输出失败：{e}")))?;
    Ok(output.0.into_inner())
}

pub fn normalize(bytes: &[u8]) -> AppResult<Vec<u8>> {
    let _work = ImageWork::enter()?;
    encode_png(&fit(decode(bytes)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_smtc_zero_oversize_and_u64_overflow_before_allocating() {
        assert!(checked_stream_size(0).is_err());
        assert!(checked_stream_size(MAX_IMAGE_BYTES as u64 + 1).is_err());
        assert!(checked_stream_size(u64::MAX).is_err());
        assert_eq!(checked_stream_size(1024).unwrap(), 1024);
    }
    #[test]
    fn rejects_corrupt_input_and_bounds_normalized_artwork() {
        assert!(decode(b"not an image").is_err());
        let image = DynamicImage::new_rgb8(2560, 1440);
        let normalized = normalize(&encode_png(&image).unwrap()).unwrap();
        assert_eq!(dimensions(&normalized).unwrap(), (1280, 720));
        assert_eq!(image::guess_format(&normalized).unwrap(), ImageFormat::Png);
    }
    #[test]
    fn rejects_an_oversized_bmp_header_without_pixel_allocation() {
        let mut header = vec![0u8; 54];
        header[..2].copy_from_slice(b"BM");
        header[10..14].copy_from_slice(&54u32.to_le_bytes());
        header[14..18].copy_from_slice(&40u32.to_le_bytes());
        header[18..22].copy_from_slice(&65536i32.to_le_bytes());
        header[22..26].copy_from_slice(&65536i32.to_le_bytes());
        header[26..28].copy_from_slice(&1u16.to_le_bytes());
        header[28..30].copy_from_slice(&24u16.to_le_bytes());
        assert!(decode(&header).is_err());
    }
    #[test]
    fn byte_reader_stops_before_an_unbounded_input_is_retained() {
        let input = io::repeat(0).take(MAX_IMAGE_BYTES as u64 + 1024);
        assert!(read_limited(input).is_err());
    }
    #[test]
    fn rejects_a_valid_image_beyond_the_edge_budget() {
        let encoded = encode_png(&DynamicImage::new_rgb8(MAX_IMAGE_EDGE + 1, 1)).unwrap();
        assert!(decode(&encoded).is_err());
    }
    #[test]
    fn rejects_truncated_png_pixel_data() {
        let encoded = encode_png(&DynamicImage::new_rgb8(8, 8)).unwrap();
        assert!(decode(&encoded[..encoded.len() / 2]).is_err());
    }
}
