export type IslandTool = "settings" | "floating" | "volume" | "timer" | "hide" | "clock" | "weather" | "system" | "files";

import type { AppSettings } from "$lib/api/types";
import type { TranslationKey } from "$lib/i18n";

type ToolPreference = "showSettingsTool" | "showFloatingTool" | "showVolumeTool" | "showTimerTool" | "showHideTool" | "showClockTool" | "showWeatherTool";
type ToolDefinition = { id: IslandTool; label: TranslationKey; shortLabel: TranslationKey; preference?: ToolPreference };

/** Order, labels and availability of the built-in feature rail. */
export const BUILTIN_TOOLS: readonly ToolDefinition[] = [
  { id: "files", label: "fileSort", shortLabel: "fileSort" },
  { id: "timer", label: "timer", shortLabel: "timer", preference: "showTimerTool" },
  { id: "volume", label: "volume", shortLabel: "menuVolume", preference: "showVolumeTool" },
  { id: "floating", label: "floatingTool", shortLabel: "menuFloating", preference: "showFloatingTool" },
  { id: "settings", label: "settingsTool", shortLabel: "menuSettings", preference: "showSettingsTool" },
  { id: "hide", label: "hideTool", shortLabel: "menuHide", preference: "showHideTool" },
  { id: "clock", label: "clock", shortLabel: "clock", preference: "showClockTool" },
  { id: "weather", label: "weather", shortLabel: "weather", preference: "showWeatherTool" },
  { id: "system", label: "systemMonitor", shortLabel: "systemMonitor" },
];

export function enabledBuiltinTools(settings: AppSettings): IslandTool[] {
  return BUILTIN_TOOLS.filter(tool => !tool.preference || settings[tool.preference]).map(tool => tool.id);
}

// The tool rail now lives beneath the expanded island as a segmented control.
// Keep the geometry in one place so the native interaction region matches the
// visual control during both the expanded transition and the settled state.
export const FEATURE_RAIL_WIDTH = 276;
export const FEATURE_RAIL_BUTTON_SIZE = 40;
export const FEATURE_RAIL_COLLAPSED_WIDTH = FEATURE_RAIL_BUTTON_SIZE;
export const FEATURE_RAIL_GAP = 10;
export const FEATURE_RAIL_HEIGHT = FEATURE_RAIL_BUTTON_SIZE;

export function featureRailHeight(toolCount: number): number {
  return toolCount > 0 ? FEATURE_RAIL_HEIGHT : 0;
}
