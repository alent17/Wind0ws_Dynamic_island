import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "./types";

export const settingsApi = {
  async getSettings(): Promise<AppSettings> {
    return invoke<AppSettings>("get_settings");
  },

  async saveSettings(settings: AppSettings): Promise<void> {
    return invoke("save_settings", { settings });
  },

  async setTheme(theme: string): Promise<void> {
    return invoke("set_theme", { theme });
  },

  async getTheme(): Promise<string> {
    return invoke<string>("get_theme");
  },

  async setAlwaysOnTop(enable: boolean): Promise<void> {
    return invoke("set_always_on_top", { enable });
  },

  async setWindowOpacity(opacity: number): Promise<void> {
    return invoke("set_window_opacity", { opacity });
  },

  async getPlayerWeights(): Promise<Record<string, number>> {
    return invoke<Record<string, number>>("get_player_weights");
  },

  async setPlayerWeight(player: string, weight: number): Promise<void> {
    return invoke("set_player_weight", { player, weight });
  },

  async setPlayerWeights(weights: Record<string, number>): Promise<void> {
    return invoke("set_player_weights", { weights });
  },

  async setAutoStart(enable: boolean): Promise<void> {
    return invoke("set_auto_start_cmd", { enable });
  },

  async getAutoStart(): Promise<boolean> {
    return invoke<boolean>("get_auto_start");
  },

  async saveFloatingWindowPosition(x: number, y: number): Promise<void> {
    return invoke("save_floating_window_position", { x, y });
  },

  async getFloatingWindowPosition(): Promise<{ x: number | null; y: number | null }> {
    return invoke("get_floating_window_position");
  },

  async setHideSettingsButton(enable: boolean): Promise<void> {
    return invoke("set_hide_settings_button", { enable });
  },

  async setHideMonitorSelector(enable: boolean): Promise<void> {
    return invoke("set_hide_monitor_selector", { enable });
  },

  async setHideFloatingWindow(enable: boolean): Promise<void> {
    return invoke("set_hide_floating_window", { enable });
  },

  async setExpandedCornerRadius(radius: number): Promise<void> {
    return invoke("set_expanded_corner_radius", { radius });
  },
};
