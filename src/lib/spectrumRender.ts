export function shouldAnimateSpectrum(
  active: boolean,
  mounted: boolean,
  hasStaticValues: boolean,
  hasMovingBars: boolean,
): boolean {
  return active && mounted && !hasStaticValues && hasMovingBars;
}
