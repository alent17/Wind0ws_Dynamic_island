import { navigationGeometry, navigationHostFor, navigationScaleForWorkArea } from "./islandGeometry";
import { describe, expect, it } from "vitest";
import {
  geometryFor,
  hiddenPlacementFor,
  hostFor,
  interactionSurfaceOffsetFor,
  islandAnchorGap,
  islandContentScaleForWorkArea,
  interpolatePolygon,
  ISLAND_HOST,
  overlapAttachedEdge,
  placementFor,
  physicalHostSizeFor,
  radiiFor,
  scaleIslandGeometry,
  shapePolygonFor,
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
  });

  it("places the host along positive and negative-origin monitors", () => {
    const monitor = { x: -1920, y: -120, width: 1920, height: 1080 };
    const host = physicalHostSizeFor(hostFor("edge", "top"), 1);
    expect(placementFor(monitor, host, "top", 0)).toEqual({ x: -1920, y: -120, width: 600, height: 286 });
    expect(placementFor(monitor, host, "top", 50)).toEqual({ x: -1260, y: -120, width: 600, height: 286 });
    expect(placementFor(monitor, host, "bottom", 100)).toEqual({ x: -600, y: 674, width: 600, height: 286 });
  });

  it("overdraws exactly one physical pixel beyond every attached screen edge at each DPR", () => {
    for (const dpr of [1, 1.25, 1.5, 2]) {
      const shown = { x: -100, y: 200, width: Math.round(300 * dpr), height: Math.round(160 * dpr) };
      expect(overlapAttachedEdge(shown, "top").y).toBe(shown.y - 1);
      expect(overlapAttachedEdge(shown, "right").x).toBe(shown.x + 1);
      expect(overlapAttachedEdge(shown, "bottom").y).toBe(shown.y + 1);
      expect(overlapAttachedEdge(shown, "left").x).toBe(shown.x - 1);
    }
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

});


describe("in-island navigation geometry", () => {
  it("keeps the music page width and bounds additional menu slots", () => {
    expect(navigationGeometry("music",5).width).toBe(600);
    expect(navigationGeometry("music",8)).toEqual(navigationGeometry("music",5));
    // Weather has a compact forecast layout; it is taller than the clock,
    // not necessarily the music page with its playback controls.
    expect(navigationGeometry("weather",5).height).toBeGreaterThan(navigationGeometry("clock",5).height);
    for (const page of ["music", "weather", "clock", "timer", "volume"] as const) {
      expect(navigationGeometry(page,5).height - navigationGeometry(page,0).height).toBeCloseTo(40);
    }
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

  it("scales expanded UI against logical work area and DPI", () => {
    expect(navigationScaleForWorkArea(2100, 1180, 1)).toBe(1);
    expect(navigationScaleForWorkArea(1680, 1050, 1.25)).toBeCloseTo(0.64);
    expect(navigationScaleForWorkArea(1920, 1080, 1.5)).toBe(0.62);
    expect(navigationScaleForWorkArea(1920, 1080, 0)).toBeCloseTo(0.9142857);

    for (const scale of [0.62, 0.8, 1]) {
      for (const edge of ["top", "right", "bottom", "left"] as const) {
        for (const style of ["floating", "edge"] as const) {
          const host = navigationHostFor(style, edge, 300, scale);
          for (const page of ["music", "clock", "weather", "timer", "volume"] as const) {
            const base = navigationGeometry(page, 8, 80, style, edge, 64);
            const geometry = { ...base, width: base.width * scale, height: base.height * scale, radius: base.radius * scale };
            const offset = surfaceOffsetFor(host, geometry, style, edge, scale);
            expect(offset.x).toBeGreaterThanOrEqual(-0.001);
            expect(offset.y).toBeGreaterThanOrEqual(-0.001);
            expect(offset.x + geometry.width).toBeLessThanOrEqual(host.width + 0.001);
            expect(offset.y + geometry.height).toBeLessThanOrEqual(host.height + 0.001);
          }
        }
      }
    }
  });

  it("keeps the real content scale, animated visual anchor, hit region and physical cursor coordinates aligned", () => {
    const edges = ["top", "right", "bottom", "left"] as const;
    const styles = ["floating", "edge"] as const;
    const dprs = [1, 1.25, 1.5, 2] as const;
    const expectedScales = [0.3875, 0.4, 0.55, 0.62, 0.625];

    // Exercise the production App scale calculation using Win32-style physical
    // work-area dimensions (including a taskbar-reduced area) and each DPR.
    for (const dpr of dprs) {
      const fullAreaScale = islandContentScaleForWorkArea(2100 * dpr, 1180 * dpr, dpr);
      expect(fullAreaScale).toBeCloseTo(0.625);
      const minimumScale = islandContentScaleForWorkArea(2100 * 0.62 * dpr, 1180 * 0.62 * dpr, dpr);
      expect(minimumScale).toBeCloseTo(0.3875);
    }
    expect(islandContentScaleForWorkArea(2100 * 0.64 * 1.25, 1180 * 0.64 * 1.25, 1.25)).toBeCloseTo(0.4);
    expect(islandContentScaleForWorkArea(2100 * 0.88 * 1.5, 1180 * 0.88 * 1.5, 1.5)).toBeCloseTo(0.55);
    expect(islandContentScaleForWorkArea(2100 * 0.992, 1180 * 0.992, 1)).toBeCloseTo(0.62);

    for (const scale of expectedScales) {
      for (const dpr of dprs) {
        const baseGeometry = navigationGeometry("music", 0, 45);
        const visualGeometry = scaleIslandGeometry(baseGeometry, scale);
        const progress = 0.37;
        const visibleGap = islandAnchorGap(scale, progress);
        expect(visibleGap).toBeCloseTo(22 * scale * (1 - progress));

        for (const style of styles) {
          for (const edge of edges) {
            // These are the same production helpers used by App placement,
            // region submission, and IslandSurface rendering.
            const host = navigationHostFor(style, edge, 80, scale);
            const regionOrigin = interactionSurfaceOffsetFor(
              host, visualGeometry, style, edge, scale, visibleGap,
            );
            if (edge === "top") {
              expect(regionOrigin.y).toBeCloseTo(visibleGap);
              expect(regionOrigin.x).toBeCloseTo((host.width - visualGeometry.width) / 2);
            } else if (edge === "bottom") {
              expect(regionOrigin.y).toBeCloseTo(host.height - visibleGap - visualGeometry.height);
              expect(regionOrigin.x).toBeCloseTo((host.width - visualGeometry.width) / 2);
            } else if (edge === "left") {
              expect(regionOrigin.x).toBeCloseTo(visibleGap);
              expect(regionOrigin.y).toBeCloseTo((host.height - visualGeometry.height) / 2);
            } else {
              expect(regionOrigin.x).toBeCloseTo(host.width - visibleGap - visualGeometry.width);
              expect(regionOrigin.y).toBeCloseTo((host.height - visualGeometry.height) / 2);
            }

            const physicalHost = physicalHostSizeFor(host, dpr);
            const workArea = { x: -1920, y: -90, width: 1920, height: 1020 };
            const windowBounds = placementFor(workArea, physicalHost, edge, 37);
            expect(windowBounds.x).toBeGreaterThanOrEqual(workArea.x);
            expect(windowBounds.y).toBeGreaterThanOrEqual(workArea.y);
            expect(windowBounds.x + windowBounds.width).toBeLessThanOrEqual(workArea.x + workArea.width);
            expect(windowBounds.y + windowBounds.height).toBeLessThanOrEqual(workArea.y + workArea.height);

            // Native cursor code computes (screen physical - window physical
            // origin) / scaleFactor. Rounding a CSS point to a physical screen
            // pixel must therefore return to the same hit point within 1 CSS px.
            const localPoint = {
              x: regionOrigin.x + visualGeometry.width * 0.43,
              y: regionOrigin.y + visualGeometry.height * 0.61,
            };
            const cursorPhysical = {
              x: windowBounds.x + Math.round(localPoint.x * dpr),
              y: windowBounds.y + Math.round(localPoint.y * dpr),
            };
            const nativeLocal = {
              x: (cursorPhysical.x - windowBounds.x) / dpr,
              y: (cursorPhysical.y - windowBounds.y) / dpr,
            };
            expect(Math.abs(nativeLocal.x - localPoint.x)).toBeLessThanOrEqual(1);
            expect(Math.abs(nativeLocal.y - localPoint.y)).toBeLessThanOrEqual(1);
          }
        }
      }
    }
  });

  it("preserves the two physical-pixel reveal after logical-to-physical placement at fractional scales", () => {
    const dpr = 1.5;
    const styles = ["floating", "edge"] as const;
    for (const scale of [0.3875, 0.4, 0.55, 0.62, 0.625]) {
      for (const style of styles) {
        for (const edge of ["top", "right", "bottom", "left"] as const) {
          const host = navigationHostFor(style, edge, 80, scale);
          const physicalHost = physicalHostSizeFor(host, dpr);
          const compact = geometryFor("compact", 45, edge);
          const logicalOffset = interactionSurfaceOffsetFor(host, compact, style, edge, scale);
          const physicalOffset = { x: logicalOffset.x * dpr, y: logicalOffset.y * dpr };
          const physicalCompact = { ...compact, width: compact.width * dpr, height: compact.height * dpr };
          const shown = placementFor({ x: -1500, y: 35, width: 1500, height: 900 }, physicalHost, edge, 50);
          const hidden = hiddenPlacementFor(shown, physicalOffset, physicalCompact, edge, 2);
          if (edge === "top") expect(Math.abs(hidden.y + physicalOffset.y + physicalCompact.height - (shown.y + 2))).toBeLessThanOrEqual(0.5);
          if (edge === "right") expect(Math.abs(hidden.x + physicalOffset.x - (shown.x + physicalHost.width - 2))).toBeLessThanOrEqual(0.5);
          if (edge === "bottom") expect(Math.abs(hidden.y + physicalOffset.y - (shown.y + physicalHost.height - 2))).toBeLessThanOrEqual(0.5);
          if (edge === "left") expect(Math.abs(hidden.x + physicalOffset.x + physicalCompact.width - (shown.x + 2))).toBeLessThanOrEqual(0.5);
        }
      }
    }
  });
});
