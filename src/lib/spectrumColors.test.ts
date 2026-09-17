import { describe, expect, it } from "vitest";
import { createSpectrumPalette, parseSpectrumColor } from "./spectrumColors";

describe("spectrum colors", () => {
  it("keeps zero-valued RGB channels instead of turning them white", () => {
    expect(parseSpectrumColor("#001020")).toEqual([0, 16, 32]);
    expect(parseSpectrumColor("rgb(0, 12, 240)")).toEqual([0, 12, 240]);
  });

  it("creates distinct colors for tiny bars", () => {
    const palette = createSpectrumPalette("rgb(78,90,104)", "rgb(60,70,84)", 6);

    expect(palette).toHaveLength(6);
    expect(new Set(palette).size).toBeGreaterThan(3);
    expect(palette[0]).not.toBe(palette.at(-1));
  });
});
