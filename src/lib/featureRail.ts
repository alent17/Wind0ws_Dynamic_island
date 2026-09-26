export type IslandTool = "settings" | "floating" | "volume" | "timer" | "hide" | "clock" | "weather";

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
