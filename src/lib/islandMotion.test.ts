import { describe, expect, it } from "vitest";
import { cubicBezierEasing, islandMorphEasing } from "./islandMotion";

describe("island motion", () => {
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
