import { invoke } from "@tauri-apps/api/core";
import type { AppPreferences, AppSettings } from "./types";

export const settingsApi = {
  getPreferences: () => invoke<AppPreferences>("get_preferences"),
  savePreferences: (preferences: AppPreferences) =>
    invoke<void>("save_preferences", { preferences }),

  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) =>
    invoke<void>("save_settings", { settings }),
  updateSettings: (patch: Partial<AppSettings>) =>
    invoke<void>("update_settings", { patch }),

  setAlwaysOnTop: (enable: boolean) =>
    invoke<void>("set_always_on_top", { enable }),
  setWindowOpacity: (opacity: number) =>
    invoke<void>("set_window_opacity", { opacity }),
  getPlayerWeights: () =>
    invoke<Record<string, number>>("get_player_weights"),
  setPlayerWeight: (player: string, weight: number) =>
    invoke<void>("set_player_weight", { player, weight }),
  setPlayerWeights: (weights: Record<string, number>) =>
    invoke<void>("set_player_weights", { weights }),
  setAutoStart: (enable: boolean) =>
    invoke<void>("set_auto_start_cmd", { enable }),
  getAutoStart: () => invoke<boolean>("get_auto_start"),

  saveFloatingWindowPosition: (
    x: number,
    y: number,
    width: number,
    height: number,
  ) =>
    invoke<void>("save_floating_window_position", { x, y, width, height }),
  getFloatingWindowPosition: () =>
    invoke<[number, number, number, number] | null>("get_floating_window_position"),

  setHideSettingsButton: (enable: boolean) =>
    invoke<void>("set_hide_settings_button", { enable }),
  setHideMonitorSelector: (enable: boolean) =>
    invoke<void>("set_hide_monitor_selector", { enable }),
  setHideFloatingWindow: (enable: boolean) =>
    invoke<void>("set_hide_floating_window", { enable }),
  setExpandedCornerRadius: (radius: number) =>
    invoke<void>("set_expanded_corner_radius", { radius }),
};
