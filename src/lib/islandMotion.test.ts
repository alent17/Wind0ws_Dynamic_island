import { describe, expect, it } from "vitest";
import { anchoredLayoutOffset, cubicBezierEasing, islandMorphEasing } from "./islandMotion";

describe("island motion", () => {
  it("keeps target layouts on their attachment edge as surface dimensions change", () => {
    const surface = { width: 420, height: 180 };
    const compact = { width: 170, height: 28 };
    expect(anchoredLayoutOffset(surface, compact, "top")).toEqual({ x: 125, y: 0 });
    expect(anchoredLayoutOffset(surface, compact, "bottom")).toEqual({ x: 125, y: 152 });
    const vertical = { width: 28, height: 170 };
    expect(anchoredLayoutOffset(surface, vertical, "left")).toEqual({ x: 0, y: 5 });
    expect(anchoredLayoutOffset(surface, vertical, "right")).toEqual({ x: 392, y: 5 });
    expect(anchoredLayoutOffset({ width: 170, height: 28 }, compact, "bottom")).toEqual({ x: 0, y: 0 });
  });
  it("keeps cubic bezier endpoints exact and progresses monotonically", () => {
    expect(islandMorphEasing(0)).toBe(0);
    expect(islandMorphEasing(1)).toBe(1);
    const samples = Array.from({ length: 21 }, (_, index) => islandMorphEasing(index / 20));
    expect(samples.every((value, index) => index === 0 || value >= samples[index - 1])).toBe(true);
  });

  it("clamps easing input to its valid interval", () => {
    const easing = cubicBezierEasing(0.22, 0.8, 0.2, 1);
    expect(easing(-1)).toBe(0);
    expect(easing(2)).toBe(1);
  });
});
