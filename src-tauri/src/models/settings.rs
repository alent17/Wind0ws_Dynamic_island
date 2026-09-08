use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Persisted settings shared by the legacy player surfaces and the current
/// Isle Studio. Missing fields from newer, reduced settings files are filled
/// from the legacy defaults.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppPreferences {
    pub island_theme: String,
    pub auto_hide: bool,
    pub show_spectrum: bool,
    pub enable_animations: bool,
    pub window_opacity: u8,
    pub always_on_top: bool,
    pub hardware_acceleration: bool,
    pub reduce_animations: bool,
    pub show_debug_info: bool,
    pub log_level: String,
    pub monitor_index: u32,
    pub player_weights: HashMap<String, u32>,
    pub floating_window_x: Option<i32>,
    pub floating_window_y: Option<i32>,
    pub floating_window_width: Option<u32>,
    pub floating_window_height: Option<u32>,
    pub enable_mv_playback: bool,
    pub lock_floating_window: bool,
    pub enable_hd_cover: bool,
    pub enable_pixel_art: bool,
    pub enable_halftone: bool,
    pub cache_directory: Option<String>,
    pub auto_start: bool,
    pub hide_settings_button: bool,
    pub hide_monitor_selector: bool,
    pub hide_floating_window: bool,
    pub expanded_corner_radius: u32,
    pub always_show_top_bar: bool,
}

pub type AppSettings = AppPreferences;

impl Default for AppPreferences {
    fn default() -> Self {
        let mut player_weights = HashMap::new();
        player_weights.insert("netease".to_string(), 50);
        player_weights.insert("spotify".to_string(), 50);
        player_weights.insert("bilibili".to_string(), 50);
        player_weights.insert("qqmusic".to_string(), 50);
        player_weights.insert("apple".to_string(), 50);
        player_weights.insert("generic".to_string(), 10);

        Self {
            island_theme: "original".to_string(),
            auto_hide: true,
            show_spectrum: true,
            enable_animations: true,
            window_opacity: 255,
            always_on_top: true,
            hardware_acceleration: true,
            reduce_animations: false,
            show_debug_info: false,
            log_level: "Info".to_string(),
            monitor_index: 0,
            player_weights,
            floating_window_x: None,
            floating_window_y: None,
            floating_window_width: None,
            floating_window_height: None,
            enable_mv_playback: true,
            lock_floating_window: false,
            enable_hd_cover: true,
            enable_pixel_art: false,
            enable_halftone: false,
            cache_directory: None,
            auto_start: false,
            hide_settings_button: false,
            hide_monitor_selector: false,
            hide_floating_window: false,
            expanded_corner_radius: 45,
            always_show_top_bar: true,
        }
    }
}
