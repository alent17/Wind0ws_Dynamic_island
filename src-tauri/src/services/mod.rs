//! 业务服务层模块
//!
//! 实现核心业务逻辑，与命令层分离
//! 命令层负责 Tauri IPC，服务层负责具体业务实现

pub mod settings;
pub mod media;
pub mod cache;
pub mod spectrum;
pub mod color;
pub mod image;

pub use settings::{read_settings_file, write_settings_file, set_auto_start, get_auto_start};
pub use media::{get_media_info, get_netease_song_info, get_netease_mv_url, control_media};
pub use cache::{
    init_cache_system, clear_cache, get_cache_stats, get_cache_directory,
    set_cache_directory, get_cached_media, download_and_cache,
};
pub use spectrum::process_spectrum_data;
pub use color::{extract_dominant_color, rgb_to_hex};
pub use image::{process_image, pixelate_cover};
