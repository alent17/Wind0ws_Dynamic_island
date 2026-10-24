import { describe, expect, it } from "vitest";
import { artworkFillFromPixels } from "./artworkFill";

describe("artwork letterbox color", () => {
  it("preserves the actual solid color rather than generating a companion gradient", () => {
    for (const color of [[255, 255, 255], [0, 0, 0], [24, 56, 180]]) {
      expect(artworkFillFromPixels(new Uint8ClampedArray([...color, 255])))
        .toBe(`rgb(${color.join(", ")})`);
    }
  });
  it("uses the dominant area instead of a small saturated accent or transparent pixels", () => {
    expect(artworkFillFromPixels(new Uint8ClampedArray([
      200, 208, 216, 255, 202, 210, 218, 255, 201, 209, 217, 255,
      255, 0, 0, 255, 0, 255, 0, 0,
    ]))).toBe("rgb(201, 209, 217)");
  });
  it("returns no color when the image has no opaque pixels", () => {
    expect(artworkFillFromPixels(new Uint8ClampedArray([255, 0, 0, 0]))).toBeNull();
  });
});
