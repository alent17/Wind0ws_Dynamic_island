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
export type HostGeometry = { width: number; height: number };
export type Point = { x: number; y: number };
export type MonitorBounds = Point & { width: number; height: number };
export type WindowBounds = MonitorBounds;
export type InteractionExtraRect = { x: number; y: number; width: number; height: number; radius: number };

export type IslandRegionChange = {
  geometry: IslandGeometry;
  radii: CornerRadii;
  polygon: Point[];
  extraRects?: InteractionExtraRect[];
  settled: boolean;
};

export const ISLAND_GEOMETRY = {
  compact: { width: 80, height: 28, radius: 14 },
  hover: { width: 90, height: 30, radius: 15 },
  expanded: { width: 300, height: 160, radius: 45 },
  hidden: { width: 80, height: 28, radius: 14 },
} as const;

export const ISLAND_GAP = 22;
export const ISLAND_OVERSHOOT = 4;
export const FEATURE_RAIL_RESERVE = 176;
export const ISLAND_HOST = { width: ISLAND_GEOMETRY.expanded.width + FEATURE_RAIL_RESERVE * 2, height: 186, top: ISLAND_GAP } as const;

export function isVerticalEdge(edge: IslandEdge) {
  return edge === "left" || edge === "right";
}

export function clampExpandedRadius(radius: number) {
  return Math.min(80, Math.max(0, Number.isFinite(radius) ? radius : 45));
}

export function clampShoulderRadius(radius: number) {
  return Math.min(16, Math.max(0, Number.isFinite(radius) ? radius : 8));
}

export function clampCompactLength(length: number) {
  return Math.min(300, Math.max(80, Number.isFinite(length) ? Math.round(length) : 80));
}

export function geometryFor(
  mode: IslandMode,
  expandedRadius = 45,
  edge: IslandEdge = "top",
  compactLength = 80,
): IslandGeometry {
  const length = clampCompactLength(compactLength);
  const geometry = mode === "expanded"
    ? ISLAND_GEOMETRY.expanded
    : mode === "hover"
      ? { ...ISLAND_GEOMETRY.hover, width: Math.min(300, length + 10) }
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

export function hostFor(_style: IslandStyle, edge: IslandEdge, compactLength = 80): HostGeometry {
  // Floating and attached silhouettes share one native window envelope. Mode
  // changes can then stay entirely inside the compositor instead of resizing
  // the WebView on every switch. Edge mode simply leaves the gap transparent.
  const gap = ISLAND_GAP;
  const length = clampCompactLength(compactLength);
  return isVerticalEdge(edge)
    ? { width: ISLAND_GEOMETRY.expanded.width + FEATURE_RAIL_RESERVE * 2, height: Math.max(ISLAND_GEOMETRY.expanded.height, length) }
    : { width: ISLAND_GEOMETRY.expanded.width + FEATURE_RAIL_RESERVE * 2, height: ISLAND_GEOMETRY.expanded.height + gap + ISLAND_OVERSHOOT };
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
  const shoulder = style === "edge" ? Math.min(clampShoulderRadius(shoulderRadius), w / 4, h / 2) : 0;
  const points: Point[] = [];
  const curve = (a: Point, b: Point, c: Point, d: Point) => {
    for (let index = 0; index < 9; index += 1) points.push(cubicPoint(a, b, c, d, index / 8));
  };

  // Fixed point count is intentional: clip-path can retarget mid-transition.
  if (style === "edge") {
    // True iPhone-notch silhouette: a wide screen seam flows through a
    // concave shoulder into a narrower, straight-sided body.
    const bodyRadius = Math.min(r, (w - shoulder * 2) / 2, (h - shoulder) / 2);
    curve({ x: 0, y: 0 }, { x: shoulder * .55, y: 0 }, { x: shoulder, y: shoulder * .45 }, { x: shoulder, y: shoulder });
    curve({ x: shoulder, y: shoulder }, { x: shoulder, y: h * .35 }, { x: shoulder, y: h * .65 }, { x: shoulder, y: h - bodyRadius });
    curve({ x: shoulder, y: h - bodyRadius }, { x: shoulder, y: h - bodyRadius * .45 }, { x: shoulder + bodyRadius * .45, y: h }, { x: shoulder + bodyRadius, y: h });
    curve({ x: shoulder + bodyRadius, y: h }, { x: w * .35, y: h }, { x: w * .65, y: h }, { x: w - shoulder - bodyRadius, y: h });
    curve({ x: w - shoulder - bodyRadius, y: h }, { x: w - shoulder - bodyRadius * .45, y: h }, { x: w - shoulder, y: h - bodyRadius * .45 }, { x: w - shoulder, y: h - bodyRadius });
    curve({ x: w - shoulder, y: h - bodyRadius }, { x: w - shoulder, y: h * .65 }, { x: w - shoulder, y: h * .35 }, { x: w - shoulder, y: shoulder });
    curve({ x: w - shoulder, y: shoulder }, { x: w - shoulder, y: shoulder * .45 }, { x: w - shoulder * .55, y: 0 }, { x: w, y: 0 });
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
): Point {
  const gap = style === "floating" ? ISLAND_GAP : 0;
  if (edge === "top") return { x: (host.width - geometry.width) / 2, y: gap };
  if (edge === "right") return { x: host.width - gap - geometry.width, y: (host.height - geometry.height) / 2 };
  if (edge === "bottom") return { x: (host.width - geometry.width) / 2, y: host.height - gap - geometry.height };
  return { x: gap, y: (host.height - geometry.height) / 2 };
}

export function placementFor(
  monitor: MonitorBounds,
  host: HostGeometry,
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

export function stableEnvelope(current: IslandGeometry, target: IslandGeometry): IslandGeometry {
  const currentArea = current.width * current.height;
  const targetArea = target.width * target.height;
  return currentArea >= targetArea ? { ...current } : { ...target };
}

export function expansionForWidth(width: number) {
  return Math.min(1, Math.max(0, (width - ISLAND_GEOMETRY.hover.width) /
    (ISLAND_GEOMETRY.expanded.width - ISLAND_GEOMETRY.hover.width)));
}

export function expansionForGeometry(geometry: IslandGeometry, edge: IslandEdge, compactLength = 80) {
  const compact = geometryFor("hover", geometry.radius, edge, compactLength);
  const compactArea = compact.width * compact.height;
  const expandedArea = ISLAND_GEOMETRY.expanded.width * ISLAND_GEOMETRY.expanded.height;
  return Math.min(1, Math.max(0,
    (geometry.width * geometry.height - compactArea) / (expandedArea - compactArea),
  ));
}
