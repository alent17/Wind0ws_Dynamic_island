import { describe, expect, it } from "vitest";
import { shouldAnimateSpectrum } from "./spectrumRender";

describe("spectrum rendering policy", () => {
  it("does not animate a static Studio preview", () => {
    expect(shouldAnimateSpectrum(true, true, true, true, true)).toBe(false);
  });

  it("animates live playback and the final fade-out frames", () => {
    expect(shouldAnimateSpectrum(true, true, true, false, false)).toBe(true);
    expect(shouldAnimateSpectrum(true, true, false, false, true)).toBe(true);
  });

  it("does not animate an inactive or unmounted spectrum", () => {
    expect(shouldAnimateSpectrum(false, true, true, false, true)).toBe(false);
    expect(shouldAnimateSpectrum(true, false, true, false, true)).toBe(false);
  });
});
