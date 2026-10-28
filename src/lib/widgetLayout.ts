export const WIDGET_KINDS = ["timer", "weather", "clock", "calendar", "volume", "system"] as const;
export type IslandWidgetKind = typeof WIDGET_KINDS[number];
export type WidgetSide = "left" | "right";
export type IslandWidgets = Record<WidgetSide, IslandWidgetKind[]>;
export type IslandPanelKind = IslandWidgetKind | "music";
export function widgetPanelOrder(layout: IslandWidgets): IslandPanelKind[] {
  const normalized = normalizeWidgets(layout);
  return [...normalized.left, "music", ...normalized.right];
}
export function widgetsFromPanelOrder(order: IslandPanelKind[]): IslandWidgets {
  const pivot = order.indexOf("music");
  return normalizeWidgets({left:order.slice(0,pivot),right:order.slice(pivot+1)});
}
export function moveLayoutPanel(order: IslandPanelKind[], panel: IslandPanelKind, index: number): IslandPanelKind[] {
  const next = order.filter(item => item !== panel);
  next.splice(Math.max(0,Math.min(index,next.length)),0,panel);
  return next;
}
// Give function panels 80% of the music width for larger, readable controls.
export function widgetPanelWidth(expandedLength = 600) {
  const length = Number.isFinite(expandedLength) ? Math.min(900, Math.max(480, Math.round(expandedLength))) : 600;
  return length * 0.8;
}
export const WIDGET_GAP = 12;
export const EDITOR_HEIGHT = 132;
export const EDITOR_TOP_SPACE = 58;

export function normalizeWidgets(value: unknown): IslandWidgets {
  const result: IslandWidgets = { left: [], right: [] };
  if (!value || typeof value !== "object") return result;
  const seen = new Set<IslandWidgetKind>();
  for (const side of ["left", "right"] as const) {
    const items = (value as Record<string, unknown>)[side];
    if (!Array.isArray(items)) continue;
    for (const kind of items) {
      if (seen.size >= 2) break;
      if (!WIDGET_KINDS.includes(kind) || seen.has(kind)) continue;
      seen.add(kind);
      result[side].push(kind);
    }
  }
  return result;
}

export function placeWidget(layout: IslandWidgets, kind: IslandWidgetKind, side: WidgetSide, index = layout[side].length): IslandWidgets {
  const next = normalizeWidgets(layout);
  const previous = next[side].indexOf(kind);
  next.left = next.left.filter(item => item !== kind);
  next.right = next.right.filter(item => item !== kind);
  if (next.left.length + next.right.length >= 2) return normalizeWidgets(layout);
  const insertion = Math.max(0, Math.min(next[side].length, index - (previous >= 0 && previous < index ? 1 : 0)));
  next[side].splice(insertion, 0, kind);
  return next;
}

export function widgetExtraWidth(count: number, expandedLength = 600) {
  return Math.min(2, Math.max(0, count)) * (widgetPanelWidth(expandedLength) + WIDGET_GAP);
}
