export type CountdownStatus = "idle" | "running" | "paused";

export type CountdownState = {
  status: CountdownStatus;
  durationMs: number;
  endsAt: number | null;
  pausedRemainingMs: number;
  label: string;
};

export const DEFAULT_COUNTDOWN: CountdownState = {
  status: "idle",
  durationMs: 0,
  endsAt: null,
  pausedRemainingMs: 0,
  label: "倒计时",
};

export function createCountdownState(label = "倒计时"): CountdownState {
  return { ...DEFAULT_COUNTDOWN, label };
}

export function getRemainingMs(state: CountdownState, now = Date.now()): number {
  if (state.status === "running" && state.endsAt !== null) {
    return Math.max(0, state.endsAt - now);
  }
  return Math.max(0, state.pausedRemainingMs);
}

export function startCountdown(
  state: CountdownState,
  durationMs: number,
  now = Date.now(),
): CountdownState {
  const duration = Math.max(1_000, Math.round(durationMs));
  return {
    ...state,
    status: "running",
    durationMs: duration,
    endsAt: now + duration,
    pausedRemainingMs: duration,
  };
}

export function pauseCountdown(state: CountdownState, now = Date.now()): CountdownState {
  const remaining = getRemainingMs(state, now);
  return {
    ...state,
    status: remaining > 0 ? "paused" : "idle",
    endsAt: null,
    pausedRemainingMs: remaining,
  };
}

export function resumeCountdown(state: CountdownState, now = Date.now()): CountdownState {
  const remaining = getRemainingMs(state, now);
  if (remaining <= 0) return createCountdownState(state.label);
  return {
    ...state,
    status: "running",
    endsAt: now + remaining,
    pausedRemainingMs: remaining,
  };
}

export function adjustCountdown(
  state: CountdownState,
  deltaMs: number,
  now = Date.now(),
): CountdownState {
  const remaining = Math.max(0, getRemainingMs(state, now) + Math.round(deltaMs));
  if (remaining <= 0) return createCountdownState(state.label);
  return state.status === "running"
    ? { ...state, durationMs: Math.max(state.durationMs, remaining), endsAt: now + remaining, pausedRemainingMs: remaining }
    : { ...state, durationMs: Math.max(state.durationMs, remaining), endsAt: null, pausedRemainingMs: remaining, status: "paused" };
}

export function resetCountdown(state: CountdownState): CountdownState {
  return createCountdownState(state.label);
}

/**
 * Collapse an elapsed running timer back to the idle state.
 * The timer is derived from an absolute end timestamp, so callers can run
 * this at any cadence without drifting or firing more than once.
 */
export function completeCountdown(state: CountdownState, now = Date.now()): CountdownState {
  if (state.status !== "running" || getRemainingMs(state, now) > 0) return state;
  return createCountdownState(state.label);
}

export function formatCountdown(ms: number): string {
  const totalSeconds = Math.max(0, Math.ceil(ms / 1_000));
  const hours = Math.floor(totalSeconds / 3_600);
  const minutes = Math.floor((totalSeconds % 3_600) / 60);
  const seconds = totalSeconds % 60;
  if (hours > 0) return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export function formatClock(timeZone: string, locale: string, now = Date.now()): string {
  return new Intl.DateTimeFormat(locale, {
    ...(timeZone && timeZone !== "system" ? { timeZone } : {}),
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(new Date(now));
}

export function clockZoneLabel(timeZone: string, locale: string): string {
  if (!timeZone || timeZone === "system") {
    return locale.startsWith("en") ? "System time" : locale.startsWith("ja") ? "システム時刻" : "系统时间";
  }
  return timeZone.replace("Asia/", "").replace("America/", "").replace("Europe/", "").replaceAll("_", " ");
}
