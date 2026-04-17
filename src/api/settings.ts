import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, CacheStats } from "../types/settings";

export const settingsApi = {
  async getSettings(): Promise<AppSettings> {
    return await invoke<AppSettings>("get_settings");
  },

  async saveSettings(settings: AppSettings): Promise<void> {
    return await invoke("save_settings", { settings });
  },

  async setTheme(theme: string): Promise<void> {
    return await invoke("set_theme", { theme });
  },

  async getTheme(): Promise<string> {
    return await invoke<string>("get_theme");
  },

  async setAlwaysOnTop(enable: boolean): Promise<void> {
    return await invoke("set_always_on_top", { enable });
  },

  async setWindowOpacity(opacity: number): Promise<void> {
    return await invoke("set_window_opacity", { opacity });
  },

  async setPlayerWeights(weights: Record<string, number>): Promise<void> {
    return await invoke("set_player_weights", { weights });
  },

  async clearCache(): Promise<void> {
    return await invoke("clear_cache");
  },

  async getCacheStats(): Promise<CacheStats> {
    return await invoke<CacheStats>("get_cache_stats");
  },

  async pickCacheDirectory(): Promise<string | null> {
    return await invoke<string | null>("pick_cache_directory");
  },

  async getCacheDirectory(): Promise<string> {
    return await invoke<string>("get_cache_directory");
  },

  async setAutoStart(enable: boolean): Promise<void> {
    return await invoke("set_auto_start", { enable });
  },

  async getAutoStart(): Promise<boolean> {
    return await invoke<boolean>("get_auto_start");
  },

  async setFloatingWindowPosition(x: number, y: number): Promise<void> {
    return await invoke("set_floating_window_position", { x, y });
  },

  async setFloatingWindowSize(width: number, height: number): Promise<void> {
    return await invoke("set_floating_window_size", { width, height });
  },

  async setMonitorIndex(index: number): Promise<void> {
    return await invoke("set_monitor_index", { index });
  },

  async setEnableMVPlayback(enable: boolean): Promise<void> {
    return await invoke("set_enable_mv_playback", { enable });
  },

  async setLockFloatingWindow(lock: boolean): Promise<void> {
    return await invoke("set_lock_floating_window", { lock });
  },

  async setEnableHDCover(enable: boolean): Promise<void> {
    return await invoke("set_enable_hd_cover", { enable });
  },

  async setEnablePixelArt(enable: boolean): Promise<void> {
    return await invoke("set_enable_pixel_art", { enable });
  },

  async setAutoHide(autoHide: boolean): Promise<void> {
    return await invoke("set_auto_hide", { autoHide });
  },

  async setShowSpectrum(show: boolean): Promise<void> {
    return await invoke("set_show_spectrum", { show });
  },

  async setEnableAnimations(enable: boolean): Promise<void> {
    return await invoke("set_enable_animations", { enable });
  },

  async setHardwareAcceleration(enable: boolean): Promise<void> {
    return await invoke("set_hardware_acceleration", { enable });
  },

  async setReduceAnimations(reduce: boolean): Promise<void> {
    return await invoke("set_reduce_animations", { reduce });
  },

  async setShowDebugInfo(show: boolean): Promise<void> {
    return await invoke("set_show_debug_info", { show });
  },

  async setLogLevel(level: string): Promise<void> {
    return await invoke("set_log_level", { level });
  },

  // UI 显示控制 API
  async setHideSettingsButton(enable: boolean): Promise<void> {
    return await invoke("set_hide_settings_button", { enable });
  },

  async setHideMonitorSelector(enable: boolean): Promise<void> {
    return await invoke("set_hide_monitor_selector", { enable });
  },

  async setHideFloatingWindow(enable: boolean): Promise<void> {
    return await invoke("set_hide_floating_window", { enable });
  },

  // 灵动岛样式 API
  async setExpandedCornerRadius(radius: number): Promise<void> {
    return await invoke("set_expanded_corner_radius", { radius });
  },

  // 实时频谱 API
  async setRealTimeSpectrum(enable: boolean): Promise<void> {
    return await invoke("set_real_time_spectrum", { enable });
  },
};