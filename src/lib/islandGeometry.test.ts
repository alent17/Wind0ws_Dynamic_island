import { navigationGeometry, navigationHostFor } from "./islandGeometry";
import { describe, expect, it } from "vitest";
import {
  expansionForGeometry,
  expansionForWidth,
  geometryFor,
  hiddenPlacementFor,
  hostFor,
  interpolatePolygon,
  ISLAND_HOST,
  overlapAttachedEdge,
  placementFor,
  radiiFor,
  shapePolygonFor,
  stableEnvelope,
  surfaceOffsetFor,
} from "./islandGeometry";

describe("island geometry", () => {
  it("keeps the restored compact, hover and expanded dimensions", () => {
    expect(geometryFor("compact")).toEqual({ width: 80, height: 28, radius: 14 });
    expect(geometryFor("hover")).toEqual({ width: 90, height: 30, radius: 15 });
    expect(geometryFor("expanded", 38)).toEqual({ width: 600, height: 210, radius: 38 });
    expect(geometryFor("expanded", 0)).toEqual({ width: 600, height: 210, radius: 0 });
    expect(geometryFor("hidden")).toEqual({ width: 80, height: 28, radius: 14 });
    expect(ISLAND_HOST).toEqual({ width: 600, height: 286, top: 22 });
  });

  it("rotates compact geometry on side edges but keeps expanded content horizontal", () => {
    expect(geometryFor("compact", 45, "left")).toEqual({ width: 28, height: 80, radius: 14 });
    expect(geometryFor("hover", 45, "right")).toEqual({ width: 30, height: 90, radius: 15 });
    expect(geometryFor("expanded", 32, "left")).toEqual({ width: 600, height: 210, radius: 32 });
  });

  it("makes only the screen-attached corners square", () => {
    const geometry = geometryFor("expanded", 40);
    expect(radiiFor(geometry, "edge", "top")).toEqual({ topLeft: 0, topRight: 0, bottomRight: 40, bottomLeft: 40 });
    expect(radiiFor(geometry, "edge", "right")).toEqual({ topLeft: 40, topRight: 0, bottomRight: 0, bottomLeft: 40 });
    expect(radiiFor(geometry, "floating", "left")).toEqual({ topLeft: 40, topRight: 40, bottomRight: 40, bottomLeft: 40 });
  });

  it("sizes and offsets the stable native host for every edge style", () => {
    expect(hostFor("floating", "top")).toEqual({ width: 600, height: 286 });
    expect(hostFor("floating", "left")).toEqual({ width: 600, height: 286 });
    expect(hostFor("edge", "bottom")).toEqual({ width: 600, height: 286 });
    expect(hostFor("edge", "right")).toEqual(hostFor("floating", "right"));
    expect(surfaceOffsetFor(hostFor("floating", "right"), geometryFor("compact", 45, "right"), "floating", "right"))
      .toEqual({ x: 550, y: 103 });
  });

  it("expands the native envelope only for the optional function panel", () => {
    expect(geometryFor("compact", 45, "top", 80, 300)).toEqual({ width: 80, height: 28, radius: 14 });
    expect(geometryFor("expanded", 45, "top", 80, 300)).toEqual({ width: 900, height: 210, radius: 45 });
    expect(hostFor("floating", "top", 80, 300)).toEqual({ width: 900, height: 286 });
    expect(hostFor("floating", "left", 300, 300)).toEqual({ width: 900, height: 300 });
  });

  it("keeps expanded content at a fixed music width", () => {
    expect(geometryFor("expanded").width).toBe(600);
  });

  it("supports an adjustable compact long axis and caps hover at expanded width", () => {
    expect(geometryFor("compact", 45, "top", 200)).toEqual({ width: 200, height: 28, radius: 14 });
    expect(geometryFor("hover", 45, "top", 200)).toEqual({ width: 210, height: 30, radius: 15 });
    expect(geometryFor("hover", 45, "top", 300)).toEqual({ width: 300, height: 30, radius: 15 });
    expect(geometryFor("compact", 45, "left", 300)).toEqual({ width: 28, height: 300, radius: 14 });
    expect(hostFor("edge", "left", 300)).toEqual({ width: 600, height: 300 });
    expect(expansionForGeometry(geometryFor("compact", 45, "top", 300), "top", 300)).toBe(0);
  });

  it("places the host along positive and negative-origin monitors", () => {
    const monitor = { x: -1920, y: -120, width: 1920, height: 1080 };
    const host = hostFor("edge", "top");
    expect(placementFor(monitor, host, "top", 0)).toEqual({ x: -1920, y: -120, width: 600, height: 286 });
    expect(placementFor(monitor, host, "top", 50)).toEqual({ x: -1260, y: -120, width: 600, height: 286 });
    expect(placementFor(monitor, host, "bottom", 100)).toEqual({ x: -600, y: 674, width: 600, height: 286 });
  });

  it("overdraws one physical pixel beyond every attached screen edge", () => {
    const shown = { x: 100, y: 200, width: 300, height: 160 };
    expect(overlapAttachedEdge(shown, "top")).toEqual({ ...shown, y: 199 });
    expect(overlapAttachedEdge(shown, "right")).toEqual({ ...shown, x: 101 });
    expect(overlapAttachedEdge(shown, "bottom")).toEqual({ ...shown, y: 201 });
    expect(overlapAttachedEdge(shown, "left")).toEqual({ ...shown, x: 99 });
  });

  it("builds fixed-size iPhone shoulder polygons in all four directions", () => {
    const geometry = geometryFor("compact");
    const floating = shapePolygonFor(geometry, "floating", "top", 8);
    for (const edge of ["top", "right", "bottom", "left"] as const) {
      const points = shapePolygonFor(geometryFor("compact", 45, edge), "edge", edge, 8);
      expect(points).toHaveLength(floating.length);
      expect(points.every(({ x, y }) => Number.isFinite(x) && Number.isFinite(y))).toBe(true);
    }
    expect(shapePolygonFor(geometry, "edge", "top", 99)).toEqual(shapePolygonFor(geometry, "edge", "top", 16));
  });

  it("uses the shoulder radius for both inward inset and curve depth without resizing the island", () => {
    const geometry = geometryFor("expanded");
    const manual = shapePolygonFor(geometry, "edge", "top", 32);
    expect(manual[8]).toEqual({ x: 32, y: 32 });
    expect(Math.max(...manual.map(({ x }) => x))).toBe(geometry.width);
    expect(Math.max(...manual.map(({ y }) => y))).toBe(geometry.height);
  });

  it("keeps the attached expanded shoulder outline mirrored across its centerline", () => {
    const geometry = geometryFor("expanded");
    for (const shoulderRadius of [0, 32]) {
      const points = shapePolygonFor(geometry, "edge", "top", shoulderRadius);
      for (const point of points) {
        const mirror = { x: geometry.width - point.x, y: point.y };
        expect(points.some((candidate) => Math.abs(candidate.x - mirror.x) < 0.02
          && Math.abs(candidate.y - mirror.y) < 0.02)).toBe(true);
      }
    }
  });

  it("interpolates silhouettes without changing their point topology", () => {
    const geometry = geometryFor("compact");
    const floating = shapePolygonFor(geometry, "floating", "top", 8);
    const edge = shapePolygonFor(geometry, "edge", "top", 8);
    expect(interpolatePolygon(floating, edge, 0)).toEqual(floating);
    expect(interpolatePolygon(floating, edge, 1)).toEqual(edge);
    expect(interpolatePolygon(floating, edge, .5)).toHaveLength(floating.length);
  });

  it("hides toward the selected edge while leaving a two-pixel reveal", () => {
    const shown = { x: 100, y: 200, width: 322, height: 160 };
    const compact = geometryFor("compact", 45, "left");
    const offset = surfaceOffsetFor({ width: 322, height: 160 }, compact, "floating", "left");
    expect(hiddenPlacementFor(shown, offset, compact, "left")).toEqual({ x: 52, y: 200, width: 322, height: 160 });
  });

  it("leaves exactly two pixels visible for both styles on every edge", () => {
    const shown = { x: 100, y: 200, width: 322, height: 182 };
    for (const style of ["floating", "edge"] as const) {
      for (const edge of ["top", "right", "bottom", "left"] as const) {
        const host = hostFor(style, edge);
        const compact = geometryFor("compact", 45, edge);
        const offset = surfaceOffsetFor(host, compact, style, edge);
        const hidden = hiddenPlacementFor({ ...shown, ...host }, offset, compact, edge);
        if (edge === "top") expect(hidden.y + offset.y + compact.height).toBe(shown.y + 2);
        if (edge === "right") expect(hidden.x + offset.x).toBe(shown.x + host.width - 2);
        if (edge === "bottom") expect(hidden.y + offset.y).toBe(shown.y + host.height - 2);
        if (edge === "left") expect(hidden.x + offset.x + compact.width).toBe(shown.x + 2);
      }
    }
  });

  it("clamps crossfade progress during rapid retargeting", () => {
    expect(expansionForWidth(80)).toBe(0);
    expect(expansionForWidth(90)).toBe(0);
    expect(expansionForWidth(345)).toBeCloseTo(0.5);
    expect(expansionForWidth(600)).toBe(1);
    expect(expansionForWidth(800)).toBe(1);
  });

  it("keeps the larger stable envelope until a transition settles", () => {
    const compact = geometryFor("compact");
    const hover = geometryFor("hover");
    const expanded = geometryFor("expanded", 38);

    expect(stableEnvelope(compact, hover)).toEqual(hover);
    expect(stableEnvelope(hover, compact)).toEqual(hover);
    expect(stableEnvelope(expanded, compact)).toEqual(expanded);
    expect(stableEnvelope(compact, expanded)).toEqual(expanded);
  });
});


describe("in-island navigation geometry", () => {
  it("keeps the music page width and caps the menu at five slots", () => {
    expect(navigationGeometry("music",5).width).toBe(600);
    expect(navigationGeometry("music",8)).toEqual(navigationGeometry("music",5));
    expect(navigationGeometry("weather",5).height).toBeGreaterThan(navigationGeometry("music",5).height);
  });
  it("fits every page and maximum shoulders inside the shared host", () => {
    for (const edge of ["top","right","bottom","left"] as const) {
      for (const style of ["floating","edge"] as const) {
        const host = navigationHostFor(style,edge,300);
        for (const page of ["music","clock","weather","timer","volume"] as const) {
          const geometry = navigationGeometry(page,8,80,style,edge,64);
          const offset = surfaceOffsetFor(host,geometry,style,edge);
          expect(offset.x).toBeGreaterThanOrEqual(0);
          expect(offset.y).toBeGreaterThanOrEqual(0);
          expect(offset.x + geometry.width).toBeLessThanOrEqual(host.width);
          expect(offset.y + geometry.height).toBeLessThanOrEqual(host.height);
        }
      }
    }
  });
});
