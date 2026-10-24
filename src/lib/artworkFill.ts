/** Actual dominant image color for letterboxing; no spectrum hue/lightness synthesis. */
export function artworkFillFromPixels(pixels: Uint8ClampedArray): string | null {
  const bins = new Map<number, { count: number; r: number; g: number; b: number }>();
  for (let i = 0; i + 3 < pixels.length; i += 4) {
    if (pixels[i + 3] < 128) continue;
    const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
    const key = (r >> 4) * 256 + (g >> 4) * 16 + (b >> 4);
    const bin = bins.get(key) ?? { count: 0, r: 0, g: 0, b: 0 };
    bin.count++; bin.r += r; bin.g += g; bin.b += b;
    bins.set(key, bin);
  }
  const dominant = [...bins.values()].sort((a, b) => b.count - a.count)[0];
  return dominant ? `rgb(${Math.round(dominant.r / dominant.count)}, ${Math.round(dominant.g / dominant.count)}, ${Math.round(dominant.b / dominant.count)})` : null;
}

export function artworkFillFromImage(image: CanvasImageSource): string | null {
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 24;
  try {
    const context = canvas.getContext("2d");
    if (!context) return null;
    context.drawImage(image, 0, 0, 24, 24);
    return artworkFillFromPixels(context.getImageData(0, 0, 24, 24).data);
  } catch { return null; }
  finally { canvas.width = canvas.height = 0; }
}
