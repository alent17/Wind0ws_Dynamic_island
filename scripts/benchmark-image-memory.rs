//! Isolated allocation diagnostic using the production image modules.
//! Requested Rust heap bytes, not Windows working set or WebView/GPU memory.
use std::{alloc::{GlobalAlloc, Layout, System}, sync::atomic::{AtomicUsize, Ordering}, time::Instant};
use base64::{engine::general_purpose, Engine};
use image::GenericImageView;

struct CountingAllocator;
static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
fn record(bytes: usize) {
    let current = CURRENT.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK.fetch_max(current, Ordering::Relaxed);
    ALLOCATED.fetch_add(bytes, Ordering::Relaxed);
}
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc(layout);
        if !pointer.is_null() { record(layout.size()); }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
        CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = System.realloc(pointer, layout, size);
        if !next.is_null() {
            CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
            record(size);
        }
        next
    }
}
#[global_allocator] static ALLOCATOR: CountingAllocator = CountingAllocator;

mod error {
    #[derive(Debug)] pub struct AppError(pub String);
    pub type AppResult<T> = Result<T, AppError>;
    impl AppError {
        pub fn business(_:u32, message:impl Into<String>)->Self {Self(message.into())}
        pub fn lock(message:impl Into<String>)->Self {Self(message.into())}
        pub fn parse(message:impl Into<String>)->Self {Self(message.into())}
        pub fn not_found(message:impl Into<String>)->Self {Self(message.into())}
    }
    impl From<std::io::Error> for AppError {fn from(value:std::io::Error)->Self {Self(value.to_string())}}
}
pub mod artwork_jobs {
    pub fn checkpoint()->crate::error::AppResult<()> {Ok(())}
    pub fn trace_bytes(_: &str, _: &[u8]) {}
}
#[path="../src-tauri/src/services/image_budget.rs"] pub mod image_budget;
mod services { pub use crate::image_budget; }
#[path="../src-tauri/src/utils.rs"] mod utils;
#[path="../src-tauri/src/services/image.rs"] mod optimized;

// The previous algorithm deliberately retains its source raster while creating
// the output, and encodes Base64 into a temporary before formatting the URI.
fn legacy(path: &str, pixelated: bool) -> String {
    let bytes = image_budget::read_limited(std::fs::File::open(path).unwrap(), 0).unwrap();
    let original = image_budget::fit(image_budget::decode(&bytes).unwrap());
    if !pixelated {
        let png=image_budget::encode_png(&original).unwrap();
        return format!("data:image/png;base64,{}",general_purpose::STANDARD.encode(png));
    }
    let mut output = original.to_rgba8();
    if pixelated {
        let (width,height) = original.dimensions();
        for y in (0..height).step_by(12) { for x in (0..width).step_by(12) {
            let (right,bottom) = ((x+12).min(width),(y+12).min(height));
            let mut sum=[0u64;3];
            let count=u64::from((right-x)*(bottom-y));
            for py in y..bottom { for px in x..right {
                let pixel=original.get_pixel(px,py);
                for channel in 0..3 { sum[channel]+=u64::from(pixel[channel]); }
            }}
            for py in y..bottom { for px in x..right {
                let pixel=output.get_pixel_mut(px,py);
                for channel in 0..3 { pixel[channel]=(sum[channel]/count) as u8; }
            }}
        }}
    }
    let pixels=image::DynamicImage::ImageRgba8(output);
    let png=image_budget::encode_png(&pixels).unwrap();
    format!("data:image/png;base64,{}",general_purpose::STANDARD.encode(png))
}

fn main() {
    let args:Vec<_>=std::env::args().collect();
    if args.get(1).map(String::as_str)==Some("fixture") {
        let mut seed=0x12345678u32;
        let image=image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(1280,1280,|_,_| {
            let mut pixel=[0u8;4];
            for value in &mut pixel { seed^=seed<<13;seed^=seed>>17;seed^=seed<<5;*value=seed as u8; }
            image::Rgba(pixel)
        }));
        std::fs::write(&args[2],image_budget::encode_png(&image).unwrap()).unwrap();
        return;
    }
    let mode=&args[1];let path=&args[2];let pixelated=mode.ends_with("pixel");
    let baseline=CURRENT.load(Ordering::Relaxed);
    PEAK.store(baseline,Ordering::Relaxed);ALLOCATED.store(0,Ordering::Relaxed);
    let start=Instant::now();
    let output=if mode.starts_with("legacy") {legacy(path,pixelated)} else {optimized::process_image(path,pixelated).unwrap()};
    let ms=start.elapsed().as_secs_f64()*1000.0;
    let peak=PEAK.load(Ordering::Relaxed).saturating_sub(baseline);
    let allocated=ALLOCATED.load(Ordering::Relaxed);
    use std::hash::{Hash,Hasher};let mut hash=std::collections::hash_map::DefaultHasher::new();output.hash(&mut hash);
    println!("{{\"mode\":\"{mode}\",\"elapsedMs\":{ms},\"peakRequestedBytes\":{peak},\"totalAllocatedBytes\":{allocated},\"outputBytes\":{},\"outputHash\":\"{:016x}\"}}",output.len(),hash.finish());
    std::hint::black_box(output);
}
