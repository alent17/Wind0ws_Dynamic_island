export type Rgb = [number, number, number];

type Hsl = {
  h: number;
  s: number;
  l: number;
};

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
  const minimumSaturation = hasColor ? Math.max(top.s, bottom.s, 0.52) : 0;
  const lightMin = Math.min(top.l, bottom.l);
  const lightMax = Math.max(top.l, bottom.l);
  const gapBoost = lightnessGap < 0.24 ? 0.14 : 0;
  const readableBottom = clamp(lightMin - gapBoost * 0.55, hasColor ? 0.34 : 0.48, 0.68);
  const readableTop = clamp(lightMax + gapBoost, hasColor ? 0.58 : 0.58, 0.94);

  return Array.from({ length: Math.max(1, count) }, (_, index) => {
    const amount = count <= 1 ? 1 : index / (count - 1);
    return toCss(ensureReadableColor({
      h: hasColor ? interpolateHue(bottomHue, topHue, amount) : 0,
      s: hasColor ? clamp(bottom.s + (top.s - bottom.s) * amount, minimumSaturation, 0.92) : 0,
      l: readableBottom + (readableTop - readableBottom) * amount,
    }));
  });
}
