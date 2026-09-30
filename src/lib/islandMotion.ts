import { ISLAND_OVERSHOOT } from "./islandGeometry";

export const ISLAND_MOTION = {
  crossAxisDuration: 280,
  outwardDelay: 40,
  outwardDuration: 300,
  settleDuration: 60,
  radiusDuration: 320,
  collapseDuration: 420,
  hoverDuration: 140,
  styleDuration: 180,
  overshoot: ISLAND_OVERSHOOT,
} as const;

function coordinate(t: number, a: number, b: number) {
  const inverse = 1 - t;
  return 3 * inverse * inverse * t * a + 3 * inverse * t * t * b + t * t * t;
}

function slope(t: number, a: number, b: number) {
  return 3 * (1 - t) * (1 - t) * a
    + 6 * (1 - t) * t * (b - a)
    + 3 * t * t * (1 - b);
}

export function cubicBezierEasing(x1: number, y1: number, x2: number, y2: number) {
  return (progress: number) => {
    const x = Math.min(1, Math.max(0, progress));
    if (x === 0 || x === 1) return x;

    let parameter = x;
    for (let iteration = 0; iteration < 8; iteration += 1) {
      const error = coordinate(parameter, x1, x2) - x;
      const velocity = slope(parameter, x1, x2);
      if (Math.abs(error) < 1e-7 || Math.abs(velocity) < 1e-7) break;
      parameter -= error / velocity;
    }

    // Newton's method is fast for the normal case; bisection keeps extreme
    // control points stable and guarantees a value inside the curve.
    if (parameter < 0 || parameter > 1) {
      let lower = 0;
      let upper = 1;
      parameter = x;
      for (let iteration = 0; iteration < 16; iteration += 1) {
        const sample = coordinate(parameter, x1, x2);
        if (sample < x) lower = parameter;
        else upper = parameter;
        parameter = (lower + upper) / 2;
      }
    }

    return coordinate(parameter, y1, y2);
  };
}

export const islandMorphEasing = cubicBezierEasing(0.22, 0.8, 0.2, 1);
export const islandSettleEasing = cubicBezierEasing(0.23, 1, 0.32, 1);

export const ISLAND_SPRING = { stiffness: 0.18, damping: 0.8, precision: 0.01 } as const;
