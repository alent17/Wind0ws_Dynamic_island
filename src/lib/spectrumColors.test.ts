import { describe, expect, it } from "vitest";
import {
  createSpectrumPalette,
  extractSpectrumColorsFromPixels,
  parseSpectrumColor,
} from "./spectrumColors";

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

  it("keeps dark album colors visible against the black island", () => {
    const palette = createSpectrumPalette("#101a42", "#050914", 6);
    const luminance = (color: string) => {
      const [r, g, b] = parseSpectrumColor(color).map((channel) => channel / 255);
      const linear = (channel: number) => channel <= 0.03928
        ? channel / 12.92
        : ((channel + 0.055) / 1.055) ** 2.4;
      return 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);
    };

    expect(Math.min(...palette.map(luminance))).toBeGreaterThanOrEqual(0.22);
  });

  it("prefers the artwork palette over black borders and white lettering", () => {
    const pixels = new Uint8ClampedArray(10 * 10 * 4);
    for (let pixel = 0; pixel < 100; pixel += 1) {
      const offset = pixel * 4;
      const x = pixel % 10;
      const edge = x === 0 || x === 9;
      const lettering = pixel === 44 || pixel === 45 || pixel === 54 || pixel === 55;
      const color = edge ? [0, 0, 0] : lettering ? [255, 255, 255]
        : pixel % 3 === 0 ? [210, 52, 82] : [126, 24, 58];
      pixels.set([...color, 255], offset);
    }

    const colors = extractSpectrumColorsFromPixels(pixels, 10, 10);
    expect(colors).not.toBeNull();
    expect(colors!.top[0]).toBeGreaterThan(colors!.top[1] * 2);
    expect(colors!.bottom[0]).toBeGreaterThan(colors!.bottom[2] * 1.5);
  });
});
