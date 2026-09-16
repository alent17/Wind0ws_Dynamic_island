export function shouldAnimateSpectrum(
  active: boolean,
  mounted: boolean,
  playing: boolean,
  hasStaticValues: boolean,
  hasVisibleBars: boolean,
): boolean {
  return active && mounted && !hasStaticValues && (playing || hasVisibleBars);
}
