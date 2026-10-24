use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct WeatherLocation {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl Default for WeatherLocation {
    fn default() -> Self {
        Self {
            name: String::new(),
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct IdleContentItem {
    pub id: String,
    pub kind: String,
    pub enabled: bool,
    pub text: String,
}

impl Default for IdleContentItem {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: "custom".to_string(),
            enabled: true,
            text: String::new(),
        }
    }
}

fn default_idle_items() -> Vec<IdleContentItem> {
    ["clock", "weather"]
        .into_iter()
        .map(|kind| IdleContentItem {
            id: kind.to_string(),
            kind: kind.to_string(),
            enabled: true,
            text: String::new(),
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NeteaseProgressMode {
    #[default]
    Real,
    Estimate,
}

/// Persisted settings shared by the legacy player surfaces and the current
/// Isle Studio. Missing fields from newer, reduced settings files are filled
/// from the legacy defaults.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct AppPreferences {
    pub netease_progress_mode: NeteaseProgressMode,
    pub island_style: String,
    pub island_edge: String,
    pub island_edge_position: u8,
    #[serde(alias = "edgeShoulderRadius")]
    pub collapsed_edge_shoulder_radius: u8,
    pub expanded_edge_shoulder_radius: u8,
    pub floating_compact_length: u16,
    pub edge_compact_length: u16,
    pub language: String,
    pub font_id: String,
    pub capture_hide_on_screenshot: bool,
    pub capture_hide_on_recording: bool,
    #[serde(alias = "autoHide")]
    pub capture_hide_on_fullscreen: bool,
    pub capture_hide_on_screen_share: bool,
    pub show_spectrum: bool,
    pub spectrum_mode: String,
    pub enable_animations: bool,
    pub window_opacity: u8,
    pub always_on_top: bool,
    pub floating_window_always_on_top: bool,
    pub reduce_animations: bool,
    pub show_debug_info: bool,
    pub log_level: String,
    pub monitor_index: u32,
    pub player_weights: HashMap<String, u32>,
    pub player_order_ids: Vec<String>,
    pub selected_player_ids: Option<Vec<String>>,
    pub idle_content_enabled: bool,
    pub idle_rotation_seconds: u16,
    pub idle_items: Vec<IdleContentItem>,
    pub weather_location: Option<WeatherLocation>,
    pub floating_window_x: Option<i32>,
    pub floating_window_y: Option<i32>,
    pub floating_window_width: Option<u32>,
    pub floating_window_height: Option<u32>,
    pub show_settings_tool: bool,
    pub show_floating_tool: bool,
    pub show_volume_tool: bool,
    pub show_timer_tool: bool,
    pub show_hide_tool: bool,
    pub show_clock_tool: bool,
    pub show_weather_tool: bool,
    pub show_custom_function_panel: bool,
    pub enable_mv_playback: bool,
    pub lock_floating_window: bool,
    pub enable_hd_cover: bool,
    pub floating_circular_album: bool,
    pub enable_pixel_art: bool,
    pub enable_halftone: bool,
    pub cache_directory: Option<String>,
    pub auto_start: bool,
    pub hide_settings_button: bool,
    pub hide_monitor_selector: bool,
    pub hide_floating_window: bool,
    pub floating_expanded_corner_radius: u32,
    pub edge_expanded_corner_radius: u32,
    pub always_show_top_bar: bool,
    pub clock_time_zone: String,
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
            netease_progress_mode: NeteaseProgressMode::Real,
            island_style: "floating".to_string(),
            island_edge: "top".to_string(),
            island_edge_position: 50,
            collapsed_edge_shoulder_radius: 8,
            expanded_edge_shoulder_radius: 32,
            floating_compact_length: 80,
            edge_compact_length: 80,
            language: "system".to_string(),
            font_id: "misans".to_string(),
            capture_hide_on_screenshot: true,
            capture_hide_on_recording: true,
            capture_hide_on_fullscreen: true,
            capture_hide_on_screen_share: true,
            show_spectrum: true,
            spectrum_mode: "realtime".to_string(),
            enable_animations: true,
            window_opacity: 255,
            always_on_top: true,
            floating_window_always_on_top: true,
            reduce_animations: false,
            show_debug_info: false,
            log_level: "Info".to_string(),
            monitor_index: 0,
            player_weights,
            player_order_ids: Vec::new(),
            selected_player_ids: None,
            idle_content_enabled: true,
            idle_rotation_seconds: 5,
            idle_items: default_idle_items(),
            weather_location: None,
            floating_window_x: None,
            floating_window_y: None,
            floating_window_width: None,
            floating_window_height: None,
            show_settings_tool: true,
            show_floating_tool: true,
            show_volume_tool: true,
            show_timer_tool: true,
            show_hide_tool: true,
            show_clock_tool: true,
            show_weather_tool: true,
            show_custom_function_panel: true,
            enable_mv_playback: true,
            lock_floating_window: false,
            enable_hd_cover: true,
            floating_circular_album: false,
            enable_pixel_art: false,
            enable_halftone: false,
            cache_directory: None,
            auto_start: false,
            hide_settings_button: false,
            hide_monitor_selector: false,
            hide_floating_window: false,
            floating_expanded_corner_radius: 45,
            edge_expanded_corner_radius: 45,
            always_show_top_bar: true,
            clock_time_zone: "system".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppPreferences, NeteaseProgressMode};

    #[test]
    fn defaults_and_migrates_edge_shoulder_radius() {
        assert_eq!(AppPreferences::default().collapsed_edge_shoulder_radius, 8);
        assert_eq!(AppPreferences::default().expanded_edge_shoulder_radius, 32);
        let loaded: AppPreferences = serde_json::from_str("{}").expect("defaulted settings");
        assert_eq!(loaded.collapsed_edge_shoulder_radius, 8);
        assert_eq!(loaded.floating_compact_length, 80);
        assert_eq!(loaded.edge_compact_length, 80);
        assert_eq!(loaded.language, "system");
        assert_eq!(loaded.font_id, "misans");
        assert_eq!(loaded.spectrum_mode, "realtime");
        assert_eq!(loaded.netease_progress_mode, NeteaseProgressMode::Real);
        assert_eq!(loaded.floating_expanded_corner_radius, 45);
        assert_eq!(loaded.edge_expanded_corner_radius, 45);
        assert!(loaded.capture_hide_on_screenshot);
        assert!(loaded.capture_hide_on_recording);
        assert!(loaded.capture_hide_on_fullscreen);
        assert!(loaded.capture_hide_on_screen_share);
        assert!(loaded.selected_player_ids.is_none());
        assert!(loaded.show_settings_tool);
        assert!(loaded.show_hide_tool);
        assert!(loaded.show_clock_tool);
        assert!(loaded.show_weather_tool);
        assert!(loaded.show_custom_function_panel);
        assert_eq!(loaded.idle_items.len(), 2);
    }

    #[test]
    fn migrates_legacy_auto_hide_to_fullscreen_capture_setting() {
        let loaded: AppPreferences =
            serde_json::from_str(r#"{"autoHide":false,"edgeShoulderRadius":19}"#)
                .expect("legacy settings");
        assert!(!loaded.capture_hide_on_fullscreen);
        assert_eq!(loaded.collapsed_edge_shoulder_radius, 19);
    }

    #[test]
    fn ignores_removed_floating_background_settings_without_resetting_preferences() {
        let loaded: AppPreferences = serde_json::from_str(
            r##"{"language":"en","floatingFillColor":"#abcdef","floatingUseAlbumColor":true}"##,
        )
        .expect("legacy background fields should be ignored");

        assert_eq!(loaded.language, "en");
        let persisted = serde_json::to_value(&loaded).expect("serialize settings");
        assert!(persisted.get("floatingFillColor").is_none());
        assert!(persisted.get("floatingUseAlbumColor").is_none());
    }

    #[test]
    fn netease_progress_mode_roundtrips_and_rejects_unknown_values() {
        let mut settings = AppPreferences::default();
        settings.netease_progress_mode = NeteaseProgressMode::Estimate;
        let encoded = serde_json::to_value(&settings).unwrap();
        assert_eq!(encoded["neteaseProgressMode"], "estimate");
        let decoded: AppPreferences = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded.netease_progress_mode, NeteaseProgressMode::Estimate);

        let invalid = serde_json::json!({"neteaseProgressMode": "silently_fake_real"});
        assert!(serde_json::from_value::<AppPreferences>(invalid).is_err());
    }
}
