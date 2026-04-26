import { invoke } from "@tauri-apps/api/core";
import type { MonitorInfo } from "./types";

export const windowApi = {
  async showMainWindow(): Promise<void> {
    return invoke("show_main_window");
  },

  async showSettingsWindow(): Promise<void> {
    return invoke("show_settings_window");
  },

  async openFloatingWindow(): Promise<void> {
    return invoke("open_floating_window");
  },

  async closeFloatingWindow(): Promise<void> {
    return invoke("close_floating_window");
  },

  async syncWindowBounds(width: number, height: number, x: number, y: number): Promise<void> {
    return invoke("sync_window_bounds", { width, height, x, y });
  },

  async setFloatingWindowResizable(resizable: boolean): Promise<void> {
    return invoke("set_floating_window_resizable", { resizable });
  },

  async openApplication(name: string): Promise<void> {
    return invoke("open_application", { name });
  },

  async checkFullscreenApp(
    monitorX: number,
    monitorY: number,
    monitorWidth: number,
    monitorHeight: number
  ): Promise<boolean> {
    return invoke("check_fullscreen_app", { monitorX, monitorY, monitorWidth, monitorHeight });
  },

  async getAvailableMonitors(): Promise<string[]> {
    return invoke<string[]>("get_available_monitors");
  },

  async getCurrentMonitorIndex(): Promise<number> {
    return invoke<number>("get_current_monitor_index");
  },

  async setCurrentMonitorIndex(index: number): Promise<void> {
    return invoke("set_current_monitor_index", { index });
  },

  async getMonitors(): Promise<MonitorInfo[]> {
    return invoke<MonitorInfo[]>("get_monitors");
  },

  async moveToMonitor(monitorIndex: number): Promise<void> {
    return invoke("move_to_monitor", { monitorIndex });
  },
};
