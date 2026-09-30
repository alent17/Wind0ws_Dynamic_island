import { describe, expect, it } from "vitest";
import { shouldAnimateSpectrum, smoothSpectrumValue } from "./spectrumRender";

describe("spectrum rendering policy", () => {
  it("preserves smoothing speed across display frame rates", () => {
    const twoFrames = smoothSpectrumValue(smoothSpectrumValue(0, 1, 1000 / 60), 1, 1000 / 60);
    expect(twoFrames).toBeCloseTo(smoothSpectrumValue(0, 1, 1000 / 30), 10);
    expect(twoFrames).toBeLessThan(0.5);
  });

  it("releases smoothly and eventually settles at silence", () => {
    let value = 1;
    const first = smoothSpectrumValue(value, 0, 1000 / 30);
    expect(first).toBeGreaterThan(0.7);
    expect(first).toBeLessThan(1);
    for (let frame = 0; frame < 120; frame += 1) value = smoothSpectrumValue(value, 0, 1000 / 30);
    expect(value).toBe(0);
  });

  it("does not animate a static Studio preview", () => {
    expect(shouldAnimateSpectrum(true, true, true, true)).toBe(false);
  });

  it("animates while bars move and stops after they settle", () => {
    expect(shouldAnimateSpectrum(true, true, false, true)).toBe(true);
    expect(shouldAnimateSpectrum(true, true, false, false)).toBe(false);
  });

  it("does not animate an inactive or unmounted spectrum", () => {
    expect(shouldAnimateSpectrum(false, true, false, true)).toBe(false);
    expect(shouldAnimateSpectrum(true, false, false, true)).toBe(false);
  });
});
