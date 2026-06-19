import { Moon, Sun } from "lucide-svelte";

export interface AppSettings {
  island_theme: string;
  auto_hide: boolean;
  show_spectrum: boolean;
  enable_animations: boolean;
  window_opacity: number;
  always_on_top: boolean;
  hardware_acceleration: boolean;
  reduce_animations: boolean;
  show_debug_info: boolean;
  log_level: string;
  monitor_index: number;
  player_weights: Record<string, number>;
  floating_window_x: number | null;
  floating_window_y: number | null;
  floating_window_width: number | null;
  floating_window_height: number | null;
  enable_mv_playback: boolean;
  lock_floating_window: boolean;
  enable_hd_cover: boolean;
  enable_pixel_art: boolean;
  cache_directory: string | null;
  auto_start: boolean;
  hide_settings_button: boolean;
  hide_monitor_selector: boolean;
  hide_floating_window: boolean;
  expanded_corner_radius: number;
}

export interface CacheStats {
  total_size_mb: number;
  total_files: number;
  mv_count: number;
  cover_count: number;
}

export interface ThemeInfo {
  id: string;
  name: string;
  icon: any;
  color: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  island_theme: "original",
  auto_hide: true,
  show_spectrum: true,
  enable_animations: true,
  window_opacity: 255,
  always_on_top: true,
  hardware_acceleration: true,
  reduce_animations: false,
  show_debug_info: false,
  log_level: "Info",
  monitor_index: 0,
  player_weights: {
    netease: 50,
    spotify: 50,
    bilibili: 50,
    qqmusic: 50,
    apple: 50,
    generic: 10,
  },
  floating_window_x: null,
  floating_window_y: null,
  floating_window_width: null,
  floating_window_height: null,
  enable_mv_playback: false,
  lock_floating_window: false,
  enable_hd_cover: true,
  enable_pixel_art: false,
  cache_directory: null,
  auto_start: false,
  hide_settings_button: false,
  hide_monitor_selector: false,
  hide_floating_window: false,
  expanded_corner_radius: 16,
};

export const PLAYER_NAMES: Record<string, string> = {
  netease: "网易云音乐",
  spotify: "Spotify",
  bilibili: "Bilibili",
  qqmusic: "QQ 音乐",
  apple: "Apple Music",
  generic: "其他播放器",
};

export const THEMES: ThemeInfo[] = [
  {
    id: "original",
    name: "极简经典",
    icon: Moon,
    color: "#1a1a1a",
  },
  {
    id: "glassmorphism",
    name: "毛玻璃",
    icon: Sun,
    color: "#1a1a2e",
  },
];
