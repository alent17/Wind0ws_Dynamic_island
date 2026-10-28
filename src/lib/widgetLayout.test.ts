import { describe, expect, it } from "vitest";
import { normalizeWidgets, placeWidget, widgetExtraWidth, widgetPanelWidth, widgetPanelOrder, widgetsFromPanelOrder, moveLayoutPanel } from "./widgetLayout";
import { featureExtensionGeometry, volumeExtensionGeometry, navigationGeometry, navigationHostFor, islandLayoutScaleForWorkArea, physicalHostSizeFor, placementFor, fitIslandViewport } from "./islandGeometry";

describe("widget layout", () => {
  it("sizes the volume extension for the visible page, including page controls and edge insets", () => {
    expect(volumeExtensionGeometry(4, false).height).toBe(430);
    expect(volumeExtensionGeometry(1, true).height).toBe(267);
    expect(volumeExtensionGeometry(5, true).height).toBeLessThan(featureExtensionGeometry().height);
    expect(volumeExtensionGeometry(4, false, 45, 12).height).toBe(454);
  });
  it("fits a vertical feature extension in a high-DPI work area even without widgets", () => {
    const scale = islandLayoutScaleForWorkArea(1366, 768, 2, "edge", "top", 600, 0);
    const host = navigationHostFor("edge", "top", 80, scale, 600, 0);
    expect(host.height * 2).toBeLessThanOrEqual(768);
    expect(host.height).toBeGreaterThanOrEqual(featureExtensionGeometry().height * scale);
  });
  it("repartitions side panels when music itself moves",()=>{
    const order=widgetPanelOrder({left:["clock"],right:["system"]});
    expect(widgetsFromPanelOrder(moveLayoutPanel(order,"music",2))).toEqual({left:["clock","system"],right:[]});
    expect(widgetsFromPanelOrder(moveLayoutPanel(order,"music",0))).toEqual({left:[],right:["clock","system"]});
  });
  it("fits the real WebView while preserving its edge anchor and restores full size after resize", () => {
    const host={width:949,height:237};
    const cramped=fitIslandViewport(host,0.508,{width:600,height:200});
    expect(cramped.host).toEqual({width:600,height:200});
    expect(1776*cramped.scale).toBeLessThanOrEqual(600);
    expect(439.1*cramped.scale).toBeLessThanOrEqual(200);
    expect(fitIslandViewport(host,0.508,{width:950,height:238})).toEqual({host,scale:0.508});
  });
  it("keeps the entire native envelope on screen at high DPI and edge positions", () => {
    for (const dpr of [1, 1.25, 1.5, 2, 2.5]) {
      for (const edge of ["top", "left", "right", "bottom"] as const) {
        for (const length of [480, 900]) {
          const scale = islandLayoutScaleForWorkArea(1920, 1080, dpr, "edge", edge, length, 2);
          const host = navigationHostFor("edge", edge, 80, scale, length, 2);
          const physical = physicalHostSizeFor(host, dpr);
          expect(physical.width).toBeLessThanOrEqual(1920);
          expect(physical.height).toBeLessThanOrEqual(1080);
          const expanded = navigationGeometry("music", 0, 45, "edge", edge, 64, length, 2, true);
          expect(expanded.width * scale).toBeLessThanOrEqual(host.width);
          expect(expanded.height * scale).toBeLessThanOrEqual(host.height);
          for (const position of [0, 50, 100]) {
            const bounds = placementFor({x:-1920,y:30,width:1920,height:1080}, physical, edge, position);
            expect(bounds.x).toBeGreaterThanOrEqual(-1920);
            expect(bounds.x + bounds.width).toBeLessThanOrEqual(0);
            expect(bounds.y).toBeGreaterThanOrEqual(30);
            expect(bounds.y + bounds.height).toBeLessThanOrEqual(1110);
          }
        }
      }
    }
  });
  it("keeps function panels at eighty percent of music width", () => {
    for (const length of [480, 600, 750, 900]) {
      expect(widgetPanelWidth(length)).toBeCloseTo(length * 0.8);
      const composed = navigationGeometry("music", 0, 45, "floating", "top", 32, length, 2);
      expect(composed.width).toBe(length + 2 * (widgetPanelWidth(length) + 12));
      expect(navigationHostFor("floating", "top", 80, 1, length, 2).width).toBeGreaterThanOrEqual(composed.width);
    }
  });
  it("loads old and malformed settings safely and caps unique widgets at two", () => {
    expect(normalizeWidgets(undefined)).toEqual({left: [], right: []});
    expect(normalizeWidgets({left: ["unknown", "timer", "timer"], right: ["volume", "clock"]})).toEqual({left: ["timer"], right: ["volume"]});
    expect(normalizeWidgets({left: "clock", right: ["weather"]})).toEqual({left: [], right: ["weather"]});
    expect(normalizeWidgets({left: ["calendar"], right: ["clock"]})).toEqual({left: ["calendar"], right: ["clock"]});
  });
  it("allows two on one side, reorders them and moves across music without duplicates", () => {
    let layout = placeWidget({left: [], right: []}, "timer", "left");
    layout = placeWidget(layout, "weather", "left", 0);
    expect(layout.left).toEqual(["weather", "timer"]);
    layout = placeWidget(layout, "weather", "left", 2);
    expect(layout.left).toEqual(["timer", "weather"]);
    layout = placeWidget(layout, "timer", "right");
    expect(layout).toEqual({left: ["weather"], right: ["timer"]});
    expect(placeWidget(layout, "clock", "left")).toEqual(layout);
  });
  it("adds side widths without squeezing music or changing standalone pages", () => {
    const base = navigationGeometry("music", 0);
    const composed = navigationGeometry("music", 0, 45, "floating", "top", 32, 600, 2, true);
    expect(composed.width).toBe(base.width + widgetExtraWidth(2));
    expect(composed.height).toBeGreaterThan(base.height);
    expect(navigationGeometry("weather", 0, 45, "floating", "top", 32, 600, 2)).toEqual(navigationGeometry("weather", 0));
    for (const edge of ["top", "left", "right", "bottom"] as const) {
      const host = navigationHostFor("floating", edge, 80, 1, 600, 2);
      expect(host.width).toBeGreaterThanOrEqual(composed.width);
      expect(host.height).toBeGreaterThanOrEqual(composed.height);
      const attached = navigationGeometry("music", 0, 45, "edge", edge, 64, 600, 2, true);
      const attachedHost = navigationHostFor("edge", edge, 80, 1, 600, 2);
      expect(attachedHost.width).toBeGreaterThanOrEqual(attached.width);
      expect(attachedHost.height).toBeGreaterThanOrEqual(attached.height);
    }
  });
});
