import { describe, expect, it } from "vitest";
import { adjustCountdown, completeCountdown, getRemainingMs, pauseCountdown, resumeCountdown, startCountdown } from "./countdown";

describe("countdown", () => {
  it("derives remaining time from endsAt instead of decrementing ticks", () => {
    const state = startCountdown({ status: "idle", durationMs: 0, endsAt: null, pausedRemainingMs: 0, label: "test" }, 300_000, 1_000);
    expect(getRemainingMs(state, 4_000)).toBe(297_000);
    expect(getRemainingMs(state, 8_000)).toBe(293_000);
  });

  it("pauses and resumes from the measured remaining time", () => {
    const running = startCountdown({ status: "idle", durationMs: 0, endsAt: null, pausedRemainingMs: 0, label: "test" }, 60_000, 1_000);
    const paused = pauseCountdown(running, 15_500);
    expect(paused.status).toBe("paused");
    expect(paused.pausedRemainingMs).toBe(45_500);
    const resumed = resumeCountdown(paused, 20_000);
    expect(resumed.endsAt).toBe(65_500);
  });

  it("adjusts a running timer without resetting its elapsed time", () => {
    const running = startCountdown({ status: "idle", durationMs: 0, endsAt: null, pausedRemainingMs: 0, label: "test" }, 60_000, 1_000);
    const adjusted = adjustCountdown(running, 60_000, 31_000);
    expect(adjusted.endsAt).toBe(121_000);
    expect(getRemainingMs(adjusted, 31_000)).toBe(90_000);
  });

  it("returns an elapsed running timer to idle", () => {
    const running = startCountdown({ status: "idle", durationMs: 0, endsAt: null, pausedRemainingMs: 0, label: "test" }, 60_000, 1_000);
    const finished = completeCountdown(running, 61_000);
    expect(finished).toEqual({ status: "idle", durationMs: 0, endsAt: null, pausedRemainingMs: 0, label: "test" });
  });

  it("does not complete a timer before its end timestamp", () => {
    const running = startCountdown({ status: "idle", durationMs: 0, endsAt: null, pausedRemainingMs: 0, label: "test" }, 60_000, 1_000);
    expect(completeCountdown(running, 60_999)).toBe(running);
  });
});
