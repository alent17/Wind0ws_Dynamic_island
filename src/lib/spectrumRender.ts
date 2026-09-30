export function smoothSpectrumValue(current: number, target: number, frameMs: number, band = 0): number {
  const timeConstant = (target > current ? 68 : 180) * (1 + (band % 3 - 1) * 0.08);
  const next = current + (target - current) * -Math.expm1(-Math.max(0, frameMs) / timeConstant);
  return Math.abs(target - next) <= 0.002 ? target : next;
}

export function shouldAnimateSpectrum(
  active: boolean,
  mounted: boolean,
  hasStaticValues: boolean,
  hasMovingBars: boolean,
): boolean {
  return active && mounted && !hasStaticValues && hasMovingBars;
}
