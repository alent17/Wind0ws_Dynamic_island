export function rollingDigitDelta(from: number, to: number) {
  const current = Math.max(0, Math.min(9, Math.trunc(from)));
  const next = Math.max(0, Math.min(9, Math.trunc(to)));
  return ((next - current + 15) % 10) - 5;
}
