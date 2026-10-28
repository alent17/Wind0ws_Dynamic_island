import { widgetExtraWidth, EDITOR_HEIGHT, EDITOR_TOP_SPACE } from "$lib/widgetLayout";
import { FEATURE_RAIL_GAP, FEATURE_RAIL_HEIGHT, FEATURE_RAIL_WIDTH } from "$lib/featureRail";

export type IslandMode = "compact" | "hover" | "expanded" | "hidden";
export type IslandStyle = "floating" | "edge";
export type IslandEdge = "top" | "right" | "bottom" | "left";

export type CornerRadii = {
  topLeft: number;
  topRight: number;
  bottomRight: number;
  bottomLeft: number;
};

export type IslandGeometry = { width: number; height: number; radius: number };
/** WebView/CSS logical pixels. */
export type HostGeometry = { width: number; height: number };
/** Native window dimensions in physical screen pixels. */
export type PhysicalWindowSize = { width: number; height: number };
export type Point = { x: number; y: number };
/** Monitor work-area origin and dimensions reported by Win32, in physical pixels. */
export type MonitorBounds = Point & { width: number; height: number };
/** Native window screen bounds in physical pixels. */
export type WindowBounds = MonitorBounds;
export type InteractionExtraRect = { x: number; y: number; width: number; height: number; radius: number };

export type IslandRegionChange = {
  geometry: IslandGeometry;
  radii: CornerRadii;
  polygon: Point[];
  extraRects?: InteractionExtraRect[];
  settled: boolean;
  anchorGap?: number;
};

export const ISLAND_GEOMETRY = {
  compact: { width: 80, height: 28, radius: 14 },
  hover: { width: 80, height: 28, radius: 14 },
  expanded: { width: 600, height: 210, radius: 45 },
  hidden: { width: 80, height: 28, radius: 14 },
} as const;

export const ISLAND_GAP = 22;
export const ISLAND_OVERSHOOT = 4;
export const ISLAND_HOST = {
  width: Math.max(ISLAND_GEOMETRY.expanded.width, FEATURE_RAIL_WIDTH),
  height: ISLAND_GEOMETRY.expanded.height + ISLAND_GAP + FEATURE_RAIL_GAP + FEATURE_RAIL_HEIGHT + ISLAND_OVERSHOOT,
  top: ISLAND_GAP,
} as const;

export function isVerticalEdge(edge: IslandEdge) {
  return edge === "left" || edge === "right";
}

export function clampExpandedRadius(radius: number) {
  return Math.min(80, Math.max(0, Number.isFinite(radius) ? radius : 45));
}

export function clampShoulderRadius(radius: number) {
  return Math.min(64, Math.max(0, Number.isFinite(radius) ? radius : 8));
}

export function clampExpandedLength(length: number) {
  return Math.min(900, Math.max(480, Number.isFinite(length) ? Math.round(length) : 600));
}

export function clampCompactLength(length: number) {
  return Math.min(300, Math.max(80, Number.isFinite(length) ? Math.round(length) : 80));
}

export function geometryFor(
  mode: IslandMode,
  expandedRadius = 45,
  edge: IslandEdge = "top",
  compactLength = 80,
  expandedExtraWidth = 0,
): IslandGeometry {
  const length = clampCompactLength(compactLength);
  const geometry = mode === "expanded"
    ? { ...ISLAND_GEOMETRY.expanded, width: Math.max(200, ISLAND_GEOMETRY.expanded.width + expandedExtraWidth) }
    : mode === "hover"
      ? { ...ISLAND_GEOMETRY.compact, width: length }
      : { ...ISLAND_GEOMETRY[mode], width: length };
  const vertical = isVerticalEdge(edge) && mode !== "expanded";
  return {
    width: vertical ? geometry.height : geometry.width,
    height: vertical ? geometry.width : geometry.height,
    radius: mode === "expanded" ? clampExpandedRadius(expandedRadius) : geometry.radius,
  };
}

export function radiiFor(
  geometry: IslandGeometry,
  style: IslandStyle = "floating",
  edge: IslandEdge = "top",
): CornerRadii {
  const radius = Math.min(geometry.radius, geometry.width / 2, geometry.height / 2);
  if (style === "floating") {
    return { topLeft: radius, topRight: radius, bottomRight: radius, bottomLeft: radius };
  }
  if (edge === "top") return { topLeft: 0, topRight: 0, bottomRight: radius, bottomLeft: radius };
  if (edge === "right") return { topLeft: radius, topRight: 0, bottomRight: 0, bottomLeft: radius };
  if (edge === "bottom") return { topLeft: radius, topRight: radius, bottomRight: 0, bottomLeft: 0 };
  return { topLeft: 0, topRight: radius, bottomRight: radius, bottomLeft: 0 };
}

export function borderRadiusCss(radii: CornerRadii) {
  return `${radii.topLeft}px ${radii.topRight}px ${radii.bottomRight}px ${radii.bottomLeft}px`;
}

export function hostFor(_style: IslandStyle, edge: IslandEdge, compactLength = 80, expandedExtraWidth = 0): HostGeometry {
  // Floating and attached silhouettes share one native window envelope. Mode
  // changes can then stay entirely inside the compositor instead of resizing
  // the WebView on every switch. Edge mode simply leaves the gap transparent.
  const gap = ISLAND_GAP;
  const length = clampCompactLength(compactLength);
  const hostWidth = Math.max(ISLAND_HOST.width, ISLAND_GEOMETRY.expanded.width + Math.max(0, expandedExtraWidth));
  return isVerticalEdge(edge)
    ? { width: hostWidth, height: Math.max(ISLAND_HOST.height, length) }
    : { width: hostWidth, height: ISLAND_HOST.height };
}

function cubicPoint(a: Point, b: Point, c: Point, d: Point, t: number): Point {
  const mt = 1 - t;
  return {
    x: mt ** 3 * a.x + 3 * mt ** 2 * t * b.x + 3 * mt * t ** 2 * c.x + t ** 3 * d.x,
    y: mt ** 3 * a.y + 3 * mt ** 2 * t * b.y + 3 * mt * t ** 2 * c.y + t ** 3 * d.y,
  };
}

function canonicalTopPolygon(geometry: IslandGeometry, style: IslandStyle, shoulderRadius: number): Point[] {
  const { width: w, height: h } = geometry;
  const r = Math.min(geometry.radius, w / 2, h / 2);
  const shoulder = style === "edge" ? clampShoulderRadius(shoulderRadius) : 0;
  // Keep the window's outer dimensions fixed and narrow the clipped body by
  // the configured shoulder radius.
  const effectiveShoulder = shoulder;
  const shoulderInset = style === "edge"
    ? Math.min(effectiveShoulder, w / 4, h / 2)
    : 0;
  const points: Point[] = [];
  const curve = (a: Point, b: Point, c: Point, d: Point) => {
    for (let index = 0; index < 9; index += 1) points.push(cubicPoint(a, b, c, d, index / 8));
  };

  // Fixed point count is intentional: clip-path can retarget mid-transition.
  if (style === "edge") {
    // True iPhone-notch silhouette: a wide screen seam flows through a
    // concave shoulder into a narrower, straight-sided body.
    const bodyRadius = Math.min(r, w / 2, h / 2);
    const shoulderDepth = Math.min(effectiveShoulder, h / 2, h - bodyRadius);
    const shoulderCurve = Math.min(shoulderInset, shoulderDepth);
    curve({ x: 0, y: 0 }, { x: shoulderInset - shoulderCurve * .45, y: 0 }, { x: shoulderInset, y: shoulderDepth - shoulderCurve * .45 }, { x: shoulderInset, y: shoulderDepth });
    curve({ x: shoulderInset, y: shoulderDepth }, { x: shoulderInset, y: h * .35 }, { x: shoulderInset, y: h * .65 }, { x: shoulderInset, y: h - bodyRadius });
    curve({ x: shoulderInset, y: h - bodyRadius }, { x: shoulderInset, y: h - bodyRadius * .45 }, { x: shoulderInset + bodyRadius * .45, y: h }, { x: shoulderInset + bodyRadius, y: h });
    curve({ x: shoulderInset + bodyRadius, y: h }, { x: w * .35, y: h }, { x: w * .65, y: h }, { x: w - shoulderInset - bodyRadius, y: h });
    curve({ x: w - shoulderInset - bodyRadius, y: h }, { x: w - shoulderInset - bodyRadius * .45, y: h }, { x: w - shoulderInset, y: h - bodyRadius * .45 }, { x: w - shoulderInset, y: h - bodyRadius });
    curve({ x: w - shoulderInset, y: h - bodyRadius }, { x: w - shoulderInset, y: h * .65 }, { x: w - shoulderInset, y: h * .35 }, { x: w - shoulderInset, y: shoulderDepth });
    curve({ x: w - shoulderInset, y: shoulderDepth }, { x: w - shoulderInset, y: shoulderDepth - shoulderCurve * .45 }, { x: w - shoulderInset + shoulderCurve * .45, y: 0 }, { x: w, y: 0 });
    curve({ x: w, y: 0 }, { x: w * .65, y: 0 }, { x: w * .35, y: 0 }, { x: 0, y: 0 });
    return points;
  }

  curve({ x: r, y: 0 }, { x: r * .45, y: 0 }, { x: 0, y: r * .45 }, { x: 0, y: r });
  curve({ x: 0, y: r }, { x: 0, y: h * .35 }, { x: 0, y: h * .65 }, { x: 0, y: h - r });
  curve({ x: 0, y: h - r }, { x: 0, y: h - r * .45 }, { x: r * .45, y: h }, { x: r, y: h });
  curve({ x: r, y: h }, { x: w * .35, y: h }, { x: w * .65, y: h }, { x: w - r, y: h });
  curve({ x: w - r, y: h }, { x: w - r * .45, y: h }, { x: w, y: h - r * .45 }, { x: w, y: h - r });
  curve({ x: w, y: h - r }, { x: w, y: h * .65 }, { x: w, y: h * .35 }, { x: w, y: r });
  curve({ x: w, y: r }, { x: w, y: r * .45 }, { x: w - r * .45, y: 0 }, { x: w - r, y: 0 });
  curve({ x: w - r, y: 0 }, { x: w * .65, y: 0 }, { x: w * .35, y: 0 }, { x: r, y: 0 });
  return points;
}

export function shapePolygonFor(
  geometry: IslandGeometry,
  style: IslandStyle = "floating",
  edge: IslandEdge = "top",
  shoulderRadius = 8,
): Point[] {
  const vertical = isVerticalEdge(edge);
  const canonical = canonicalTopPolygon(
    vertical ? { ...geometry, width: geometry.height, height: geometry.width } : geometry,
    style,
    shoulderRadius,
  );
  const w = geometry.width;
  const h = geometry.height;
  if (edge === "top") return canonical;
  if (edge === "bottom") return canonical.map(({ x, y }) => ({ x: w - x, y: h - y }));
  if (edge === "left") return canonical.map(({ x, y }) => ({ x: y, y: x }));
  return canonical.map(({ x, y }) => ({ x: w - y, y: h - x }));
}

export function polygonCss(points: Point[]) {
  return `polygon(${points.map(({ x, y }) => `${x.toFixed(2)}px ${y.toFixed(2)}px`).join(",")})`;
}

export function interpolatePolygon(from: Point[], to: Point[], progress: number): Point[] {
  const t = Math.min(1, Math.max(0, progress));
  if (from.length !== to.length) return t < .5 ? from : to;
  if (t === 0) return from;
  if (t === 1) return to;
  return from.map((point, index) => ({
    x: point.x + (to[index].x - point.x) * t,
    y: point.y + (to[index].y - point.y) * t,
  }));
}

export function surfaceOffsetFor(
  host: HostGeometry,
  geometry: IslandGeometry,
  style: IslandStyle,
  edge: IslandEdge,
  scale = 1,
): Point {
  // Geometry passed to the WebView and the native hit test is CSS-logical px.
  // Use the same content scale as the visible anchor; the .62 floor belongs
  // only to native host sizing, not to the silhouette's visual gap.
  const gap = style === "floating" ? islandAnchorGap(scale) : 0;
  if (edge === "top") return { x: (host.width - geometry.width) / 2, y: gap };
  if (edge === "right") return { x: host.width - gap - geometry.width, y: (host.height - geometry.height) / 2 };
  if (edge === "bottom") return { x: (host.width - geometry.width) / 2, y: host.height - gap - geometry.height };
  return { x: gap, y: (host.height - geometry.height) / 2 };
}

function normalizedContentScale(scale: number) {
  return Number.isFinite(scale) ? Math.min(1, Math.max(0, scale)) : 1;
}

export function scaleIslandGeometry(geometry: IslandGeometry, scale: number): IslandGeometry {
  const safeScale = normalizedContentScale(scale);
  return {
    width: geometry.width * safeScale,
    height: geometry.height * safeScale,
    radius: geometry.radius * safeScale,
  };
}

export function islandAnchorGap(scale: number, styleProgress = 0) {
  const progress = Number.isFinite(styleProgress) ? Math.min(1, Math.max(0, styleProgress)) : 0;
  return ISLAND_GAP * normalizedContentScale(scale) * (1 - progress);
}

/** Region origin in CSS-logical px, aligned to the same animated anchor as IslandSurface. */
export function interactionSurfaceOffsetFor(
  host: HostGeometry,
  geometry: IslandGeometry,
  style: IslandStyle,
  edge: IslandEdge,
  scale: number,
  anchorGap?: number,
): Point {
  const offset = surfaceOffsetFor(host, geometry, style, edge, scale);
  if (anchorGap === undefined) return offset;
  const delta = anchorGap - (style === "floating" ? islandAnchorGap(scale) : 0);
  if (edge === "top") offset.y += delta;
  else if (edge === "bottom") offset.y -= delta;
  else if (edge === "left") offset.x += delta;
  else offset.x -= delta;
  return offset;
}

/** Convert a CSS-logical host size to the integer physical pixels used by native window bounds. */
export function physicalHostSizeFor(host: HostGeometry, scaleFactor: number): PhysicalWindowSize {
  const dpr = Number.isFinite(scaleFactor) && scaleFactor > 0 ? scaleFactor : 1;
  return { width: Math.round(host.width * dpr), height: Math.round(host.height * dpr) };
}

export function placementFor(
  monitor: MonitorBounds,
  host: PhysicalWindowSize,
  edge: IslandEdge,
  positionPercent: number,
): WindowBounds {
  const ratio = Math.min(1, Math.max(0, positionPercent / 100));
  const horizontalTravel = Math.max(0, monitor.width - host.width);
  const verticalTravel = Math.max(0, monitor.height - host.height);
  let x = monitor.x;
  let y = monitor.y;
  if (edge === "top" || edge === "bottom") {
    x += horizontalTravel * ratio;
    if (edge === "bottom") y += verticalTravel;
  } else {
    y += verticalTravel * ratio;
    if (edge === "right") x += horizontalTravel;
  }
  return { x: Math.round(x), y: Math.round(y), width: host.width, height: host.height };
}

export function overlapAttachedEdge(
  bounds: WindowBounds,
  edge: IslandEdge,
  physicalPixels = 1,
): WindowBounds {
  const overlap = Math.max(0, Math.round(physicalPixels));
  if (edge === "top") return { ...bounds, y: bounds.y - overlap };
  if (edge === "right") return { ...bounds, x: bounds.x + overlap };
  if (edge === "bottom") return { ...bounds, y: bounds.y + overlap };
  return { ...bounds, x: bounds.x - overlap };
}

export function hiddenPlacementFor(
  shown: WindowBounds,
  offset: Point,
  compact: IslandGeometry,
  edge: IslandEdge,
  visiblePixels = 2,
): WindowBounds {
  const hidden = { ...shown };
  if (edge === "top") hidden.y -= offset.y + compact.height - visiblePixels;
  else if (edge === "right") hidden.x += shown.width - offset.x - visiblePixels;
  else if (edge === "bottom") hidden.y += shown.height - offset.y - visiblePixels;
  else hidden.x -= offset.x + compact.width - visiblePixels;
  hidden.x = Math.round(hidden.x);
  hidden.y = Math.round(hidden.y);
  return hidden;
}

export type IslandPage = "music" | "timer" | "volume" | "clock" | "weather" | "system" | "files";

export function featureExtensionGeometry(radius = 45): IslandGeometry {
  return { width: 700, height: 620, radius: clampExpandedRadius(radius) };
}

export function worldClockExtensionGeometry(radius = 45): IslandGeometry {
  return { width: 1000, height: 620, radius: clampExpandedRadius(radius) };
}

export function volumeExtensionGeometry(rows: number, hasPages: boolean, radius = 45, blockInset = 0): IslandGeometry {
  const count = Math.min(5, Math.max(1, Math.floor(Number.isFinite(rows) ? rows : 0)));
  return { width: 480, height: 154 + count * 69 + (hasPages ? 44 : 0) + Math.max(0, blockInset) * 2, radius: clampExpandedRadius(radius) };
}

export function navigationGeometry(page: IslandPage, toolCount: number, radius = 45,
  style: IslandStyle = "floating", edge: IslandEdge = "top", shoulder = 32, expandedLength = 600, widgetCount = 0, editing = false): IslandGeometry {
  const inset = style === "edge" ? clampShoulderRadius(shoulder) : 0;
  const vertical = isVerticalEdge(edge);
  const toolbar = toolCount > 0 ? 40 : 0;
  const height = page === "music" ? 249.1 : page === "files" ? 390 : page === "system" ? 280 : page === "weather" ? 240 : 188;
  const length = clampExpandedLength(expandedLength);
  const width = page === "music" ? length : Math.max(page === "files" ? 560 : page === "system" ? 380 : 300, length / 2) + (vertical ? 0 : inset * 2);
  return { width: width + (page === "music" ? widgetExtraWidth(widgetCount, expandedLength) + (widgetCount > 0 && !vertical ? inset * 2 : 0) : 0), height: height + toolbar + (vertical ? inset * 2 : 0) + (editing ? EDITOR_HEIGHT + EDITOR_TOP_SPACE : 0), radius: clampExpandedRadius(radius) };
}

const NAVIGATION_REFERENCE_WORK_AREA = { width: 2100, height: 1180 };

export function navigationScaleForWorkArea(workWidth: number, workHeight: number, scaleFactor = 1) {
  const dpr = Number.isFinite(scaleFactor) && scaleFactor > 0 ? scaleFactor : 1;
  const logicalWidth = Math.max(1, workWidth) / dpr;
  const logicalHeight = Math.max(1, workHeight) / dpr;
  // Preserve the existing 62% minimum layout scale. App's separate 0.625
  // content-fit factor then yields the intended 0.3875 minimum Island scale.
  return Math.max(0.62, Math.min(1, logicalWidth / NAVIGATION_REFERENCE_WORK_AREA.width, logicalHeight / NAVIGATION_REFERENCE_WORK_AREA.height));
}

// The monitor work area is supplied in physical pixels; content geometry stays
// in CSS-logical px and is scaled down from the reference work area.
export function islandContentScaleForWorkArea(workWidth: number, workHeight: number, scaleFactor = 1) {
  return navigationScaleForWorkArea(workWidth, workHeight, scaleFactor) * 0.625;
}

// Reserve the largest page once; only the visible silhouette and hit region morph.
export function navigationHostFor(style: IslandStyle, edge: IslandEdge, compactLength = 80, scale = 1, expandedLength = 600, widgetCount = 0): HostGeometry {
  const base = hostFor(style, edge, compactLength, clampExpandedLength(expandedLength) - 600);
  // Keep the legacy envelope for a single page. Composed layouts must follow
  // their fitted scale or the native window can extend beyond the monitor.
  const safeScale = widgetCount > 0 ? normalizedContentScale(scale) : Math.min(1, Math.max(0.62, scale));
  // Expanded content remains horizontal on side edges. A floating island needs
  // its outer gap inside the native host on those edges, just as top/bottom
  // layouts reserve the gap in host height.
  const horizontalGap = style === "floating" && isVerticalEdge(edge) ? ISLAND_GAP : 0;
  return {
    width: Math.max(Math.ceil((isVerticalEdge(edge) ? 28 : clampCompactLength(compactLength)) + horizontalGap * scale), Math.ceil(Math.max(base.width + widgetExtraWidth(widgetCount, expandedLength) + (style === "edge" && !isVerticalEdge(edge) && widgetCount > 0 ? 128 : 0) + horizontalGap, 1000 + ISLAND_GAP) * safeScale)) + (isVerticalEdge(edge) ? 0 : 24),
    height: Math.max(Math.ceil((isVerticalEdge(edge) ? clampCompactLength(compactLength) : 28) + ISLAND_GAP * scale), Math.ceil(Math.max(base.height, 620 + ISLAND_GAP + ISLAND_OVERSHOOT, style === "edge" && isVerticalEdge(edge) ? 249.1 + 128 + EDITOR_HEIGHT + EDITOR_TOP_SPACE + ISLAND_OVERSHOOT : 0) * normalizedContentScale(scale))),
  };
}

/** Fit the composed native envelope, not just the visible panel, inside the work area. */
export function islandLayoutScaleForWorkArea(workWidth: number, workHeight: number, scaleFactor: number,
  style: IslandStyle, edge: IslandEdge, expandedLength: number, widgetCount: number) {
  const base = islandContentScaleForWorkArea(workWidth, workHeight, scaleFactor);
  if (widgetCount === 0) return base;
  const dpr = Number.isFinite(scaleFactor) && scaleFactor > 0 ? scaleFactor : 1;
  const envelope = navigationHostFor(style, edge, 80, 1, expandedLength, widgetCount);
  return Math.min(base, Math.max(1, workWidth / dpr - 24) / envelope.width,
    Math.max(1, workHeight / dpr - 24) / envelope.height);
}

/** Bound rendering to the actual WebView while native window resizing is pending. */
export function fitIslandViewport(host: HostGeometry, contentScale: number, viewport: HostGeometry) {
  const width = Math.max(1, Math.min(host.width, viewport.width));
  const height = Math.max(1, Math.min(host.height, viewport.height));
  const fit = Math.min(1, width / host.width, height / host.height);
  return { host: {width, height}, scale: contentScale * fit };
}
