/** Session-only aggregate counters for a small allowlist of optimized routes. */
export type TransportRoute = "mediaSync" | "timerState" | "region" | "volume";
export type TransportCounter =
  | "offers"
  | "replaced"
  | "deduped"
  | "noConsumer"
  | "retries"
  | "dispatchAttempts"
  | "dispatchSuccess"
  | "dispatchErrors"
  | "received"
  | "requestReplies";
export type TimedTransportRoute = "region" | "volume";

type RouteStats = Partial<Record<TransportCounter, number>> & {
  durationCount: number;
  durationSumMs: number;
  durationMaxMs: number;
};
type RouteStatsMap = Record<TransportRoute, RouteStats>;
type TimingToken = { generation: number; startedAt: number };

const ROUTES: readonly TransportRoute[] = ["mediaSync", "timerState", "region", "volume"];
const COUNTERS: readonly TransportCounter[] = [
  "offers", "replaced", "deduped", "noConsumer", "retries", "dispatchAttempts",
  "dispatchSuccess", "dispatchErrors", "received", "requestReplies",
];
let enabled = false;
let generation = 0;
let stats = createEmptyStats();

function emptyRouteStats(): RouteStats {
  return { durationCount: 0, durationSumMs: 0, durationMaxMs: 0 };
}

function createEmptyStats(): RouteStatsMap {
  return { mediaSync: emptyRouteStats(), timerState: emptyRouteStats(), region: emptyRouteStats(), volume: emptyRouteStats() };
}

function recordForCurrentGeneration(route: TransportRoute, counter: TransportCounter, token: number) {
  if (!enabled || generation !== token) return;
  const row = stats[route];
  row[counter] = (row[counter] ?? 0) + 1;
}

export function isTransportDiagnosticsEnabled(): boolean {
  return enabled;
}

/** Explicit opt-in. State is local to this WebView and is never persisted. */
export function enableTransportDiagnostics(): void {
  generation += 1;
  enabled = true;
}

export function disableTransportDiagnostics(): void {
  generation += 1;
  enabled = false;
}

export function resetTransportDiagnostics(): void {
  generation += 1;
  stats = createEmptyStats();
}

export function snapshotTransportDiagnostics() {
  return {
    scope: "this-webview" as const,
    enabled,
    routes: Object.fromEntries(ROUTES.map(route => [route, { ...stats[route] }])) as RouteStatsMap,
  };
}

export function recordTransportCounter(route: TransportRoute, counter: TransportCounter): void {
  if (!enabled) return;
  recordForCurrentGeneration(route, counter, generation);
}

/** Records a command call only while enabled; the disabled path invokes action directly. */
export function runTimedTransportCommand<T>(route: TimedTransportRoute, action: () => T): T {
  if (!enabled) return action();

  const token: TimingToken = { generation, startedAt: performance.now() };
  recordForCurrentGeneration(route, "dispatchAttempts", token.generation);
  const finish = (success: boolean) => {
    if (!enabled || generation !== token.generation) return;
    recordForCurrentGeneration(route, success ? "dispatchSuccess" : "dispatchErrors", token.generation);
    const elapsed = Math.max(0, performance.now() - token.startedAt);
    const row = stats[route];
    row.durationCount += 1;
    row.durationSumMs += elapsed;
    row.durationMaxMs = Math.max(row.durationMaxMs, elapsed);
  };

  let result: T;
  try {
    result = action();
  } catch (error) {
    finish(false);
    throw error;
  }

  if (result && typeof (result as { then?: unknown }).then === "function") {
    return Promise.resolve(result).then(
      value => { finish(true); return value; },
      error => { finish(false); throw error; },
    ) as T;
  }
  finish(true);
  return result;
}

/** Counts app-level event dispatch outcomes without inspecting or serializing payloads. */
export function runTransportEvent<T>(
  route: "mediaSync" | "timerState",
  action: () => T,
  requestReply = false,
): T {
  if (!enabled) return action();
  const token = generation;
  recordForCurrentGeneration(route, "dispatchAttempts", token);
  if (route === "timerState" && requestReply) recordForCurrentGeneration(route, "requestReplies", token);
  let result: T;
  try {
    result = action();
  } catch (error) {
    recordForCurrentGeneration(route, "dispatchErrors", token);
    throw error;
  }
  if (result && typeof (result as { then?: unknown }).then === "function") {
    return Promise.resolve(result).then(
      value => { recordForCurrentGeneration(route, "dispatchSuccess", token); return value; },
      error => { recordForCurrentGeneration(route, "dispatchErrors", token); throw error; },
    ) as T;
  }
  recordForCurrentGeneration(route, "dispatchSuccess", token);
  return result;
}

export const transportDiagnosticsSchema = { routes: ROUTES, counters: COUNTERS } as const;
