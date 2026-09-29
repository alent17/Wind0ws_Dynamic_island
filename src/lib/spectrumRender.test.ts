import { describe, expect, it } from "vitest";
import { shouldAnimateSpectrum } from "./spectrumRender";

describe("spectrum rendering policy", () => {
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
