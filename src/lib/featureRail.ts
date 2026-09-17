export type IslandTool = "floating" | "timer";

export const FEATURE_RAIL_WIDTH = 168;
export const FEATURE_RAIL_BUTTON_SIZE = 32;
export const FEATURE_RAIL_COLLAPSED_WIDTH = FEATURE_RAIL_BUTTON_SIZE;
export const FEATURE_RAIL_GAP = 6;
export const FEATURE_RAIL_HEIGHT = FEATURE_RAIL_BUTTON_SIZE * 2 + FEATURE_RAIL_GAP;

export function featureRailHeight(toolCount: number): number {
  const count = Math.max(0, Math.min(2, Math.floor(toolCount)));
  return count === 0 ? 0 : count * FEATURE_RAIL_BUTTON_SIZE + (count - 1) * FEATURE_RAIL_GAP;
}
