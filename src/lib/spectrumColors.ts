export type Rgb = [number, number, number];

type Hsl = {
  h: number;
  s: number;
  l: number;
};

export type SpectrumColorPair = {
  top: Rgb;
  bottom: Rgb;
};

/** Read an album cover with the same sampling used by every island surface. */
export async function extractSpectrumColorsFromImage(src: string): Promise<SpectrumColorPair | null> {
  if (!src) return null;
  const image = new Image();
  if (!src.startsWith("file://") && !src.startsWith("data:")) image.crossOrigin = "Anonymous";
  await new Promise<void>((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error("Artwork load timed out")), 5_000);
    image.onload = () => { clearTimeout(timeout); resolve(); };
    image.onerror = () => { clearTimeout(timeout); reject(new Error("Artwork load failed")); };
    image.src = src;
  });
  const canvas = document.createElement("canvas");
  canvas.width = 24;
  canvas.height = 24;
  const context = canvas.getContext("2d");
  if (!context) return null;
  context.drawImage(image, 0, 0, 24, 24);
  return extractSpectrumColorsFromPixels(context.getImageData(0, 0, 24, 24).data, 24, 24);
}

const clamp = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, value));

export function parseSpectrumColor(color: string): Rgb {
  const rgb = color.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/i);
  if (rgb) return [Number(rgb[1]), Number(rgb[2]), Number(rgb[3])];

  const hex = color.replace("#", "");
  const normalized = hex.length === 3
    ? hex.split("").map((value) => value + value).join("")
    : hex.padEnd(6, "f").slice(0, 6);

  return [
    Number.parseInt(normalized.slice(0, 2), 16),
    Number.parseInt(normalized.slice(2, 4), 16),
    Number.parseInt(normalized.slice(4, 6), 16),
  ].map((value) => Number.isNaN(value) ? 255 : value) as Rgb;
}

function rgbToHsl([r, g, b]: Rgb): Hsl {
  const red = r / 255;
  const green = g / 255;
  const blue = b / 255;
  const max = Math.max(red, green, blue);
  const min = Math.min(red, green, blue);
  const lightness = (max + min) / 2;

  if (max === min) return { h: 0, s: 0, l: lightness };

  const delta = max - min;
  const saturation = lightness > 0.5
    ? delta / (2 - max - min)
    : delta / (max + min);
  let hue = 0;

  if (max === red) hue = (green - blue) / delta + (green < blue ? 6 : 0);
  else if (max === green) hue = (blue - red) / delta + 2;
  else hue = (red - green) / delta + 4;

  return { h: hue * 60, s: saturation, l: lightness };
}

function hslToRgb({ h, s, l }: Hsl): Rgb {
  if (s === 0) {
    const value = Math.round(l * 255);
    return [value, value, value];
  }

  const hue = ((h % 360) + 360) % 360 / 360;
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;
  const hueToRgb = (t: number) => {
    let channel = t;
    if (channel < 0) channel += 1;
    if (channel > 1) channel -= 1;
    if (channel < 1 / 6) return p + (q - p) * 6 * channel;
    if (channel < 1 / 2) return q;
    if (channel < 2 / 3) return p + (q - p) * (2 / 3 - channel) * 6;
    return p;
  };

  return [hueToRgb(hue + 1 / 3), hueToRgb(hue), hueToRgb(hue - 1 / 3)]
    .map((value) => Math.round(value * 255)) as Rgb;
}

function interpolateHue(from: number, to: number, amount: number): number {
  let delta = ((to - from + 540) % 360) - 180;
  if (Math.abs(delta) < 1) delta = 0;
  return from + delta * amount;
}

function toCss([r, g, b]: Rgb): string {
  return `rgb(${r},${g},${b})`;
}

function hueDistance(from: number, to: number): number {
  return Math.abs(((to - from + 540) % 360) - 180) / 180;
}

/**
 * Finds two representative artwork colors without letting black borders,
 * white typography, or a tiny saturated detail take over the spectrum.
 */
export function extractSpectrumColorsFromPixels(
  pixels: Uint8ClampedArray,
  width: number,
  height: number,
): SpectrumColorPair | null {
  const bins = new Map<string, { rgb: Rgb; weight: number }>();
  const safeWidth = Math.max(1, width);
  const safeHeight = Math.max(1, height);

  for (let index = 0; index < pixels.length; index += 4) {
    if (pixels[index + 3] < 128) continue;
    const pixelIndex = index / 4;
    const x = pixelIndex % safeWidth;
    const y = Math.floor(pixelIndex / safeWidth);
    const nx = (x + 0.5) / safeWidth - 0.5;
    const ny = (y + 0.5) / safeHeight - 0.5;
    const centerWeight = 1 + Math.max(0, 0.3 - Math.hypot(nx, ny) * 0.42);
    const rgb: Rgb = [
      Math.round(pixels[index] / 24) * 24,
      Math.round(pixels[index + 1] / 24) * 24,
      Math.round(pixels[index + 2] / 24) * 24,
    ].map((channel) => clamp(channel, 0, 255)) as Rgb;
    const key = rgb.join(",");
    const existing = bins.get(key);
    if (existing) existing.weight += centerWeight;
    else bins.set(key, { rgb, weight: centerWeight });
  }

  const candidates = [...bins.values()];
  if (!candidates.length) return null;

  const scored = candidates.map((candidate) => {
    const hsl = rgbToHsl(candidate.rgb);
    const midtone = clamp(1 - Math.abs(hsl.l - 0.5) * 1.35, 0.22, 1);
    return {
      ...candidate,
      hsl,
      score: candidate.weight * (0.32 + hsl.s * 1.18) * midtone,
    };
  }).sort((a, b) => b.score - a.score);

  const primary = scored[0];
  let secondary = scored.slice(1).map((candidate) => {
    const distance = hueDistance(primary.hsl.h, candidate.hsl.h) * 0.58
      + Math.abs(primary.hsl.l - candidate.hsl.l) * 0.42;
    return { candidate, score: candidate.score * (0.42 + distance) };
  }).sort((a, b) => b.score - a.score)[0]?.candidate;

  const separation = secondary
    ? hueDistance(primary.hsl.h, secondary.hsl.h) * 0.55
      + Math.abs(primary.hsl.l - secondary.hsl.l) * 0.45
    : 0;
  if (!secondary || separation < 0.1) {
    const companionLightness = primary.hsl.l > 0.58
      ? primary.hsl.l - 0.2
      : primary.hsl.l + 0.2;
    secondary = {
      ...primary,
      rgb: hslToRgb({ ...primary.hsl, l: clamp(companionLightness, 0.18, 0.82) }),
    };
  }

  return primary.hsl.l >= secondary.hsl.l
    ? { top: primary.rgb, bottom: secondary.rgb }
    : { top: secondary.rgb, bottom: primary.rgb };
}

function relativeLuminance([r, g, b]: Rgb): number {
  const toLinear = (channel: number) => {
    const normalized = channel / 255;
    return normalized <= 0.03928
      ? normalized / 12.92
      : ((normalized + 0.055) / 1.055) ** 2.4;
  };

  return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
}

/**
 * Raises only the lightness of a color until it has enough contrast against
 * the island's black surface. Keeping hue and saturation stable preserves the
 * album-art color without allowing dark navy, burgundy, or forest tones to
 * disappear into the background.
 */
function ensureReadableColor({ h, s, l }: Hsl): Rgb {
  const minimumLuminance = 0.22;
  let candidate = hslToRgb({ h, s, l });
  if (relativeLuminance(candidate) >= minimumLuminance) return candidate;

  let low = l;
  let high = 1;
  for (let step = 0; step < 8; step += 1) {
    const midpoint = (low + high) / 2;
    const midpointColor = hslToRgb({ h, s, l: midpoint });
    if (relativeLuminance(midpointColor) >= minimumLuminance) {
      high = midpoint;
      candidate = midpointColor;
    } else {
      low = midpoint;
    }
  }

  return candidate;
}

/**
 * Creates a deliberately readable ramp for six tiny canvas bars.
 * Album colors remain the endpoints, while close or muted endpoints get
 * enough lightness and saturation separation to survive the small UI size.
 */
export function createSpectrumPalette(
  topColor: string,
  bottomColor: string,
  count: number,
): string[] {
  const top = rgbToHsl(parseSpectrumColor(topColor));
  const bottom = rgbToHsl(parseSpectrumColor(bottomColor));
  const hasColor = Math.max(top.s, bottom.s) >= 0.08;
  const lightnessGap = Math.abs(top.l - bottom.l);
  const hue = top.s >= bottom.s ? top.h : bottom.h;
  const topHue = top.s >= 0.08 ? top.h : hue;
  const bottomHue = bottom.s >= 0.08 ? bottom.h : hue;
  const minimumSaturation = hasColor ? Math.max(Math.min(top.s, bottom.s), 0.16) : 0;
  const lightMin = Math.min(top.l, bottom.l);
  const lightMax = Math.max(top.l, bottom.l);
  const gapBoost = lightnessGap < 0.18 ? 0.1 : 0;
  const readableBottom = clamp(lightMin - gapBoost * 0.35, hasColor ? 0.3 : 0.48, 0.72);
  const readableTop = clamp(lightMax + gapBoost, hasColor ? 0.48 : 0.58, 0.9);

  return Array.from({ length: Math.max(1, count) }, (_, index) => {
    const amount = count <= 1 ? 1 : index / (count - 1);
    const interpolatedSaturation = bottom.s + (top.s - bottom.s) * amount;
    return toCss(ensureReadableColor({
      h: hasColor ? interpolateHue(bottomHue, topHue, amount) : 0,
      // A very small chroma lift keeps adjacent two-pixel bars distinct after
      // dark colors are raised to the same readable luminance.
      s: hasColor ? clamp(interpolatedSaturation + amount * 0.08, minimumSaturation, 0.92) : 0,
      l: readableBottom + (readableTop - readableBottom) * amount,
    }));
  });
}
