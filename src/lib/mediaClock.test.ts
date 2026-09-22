import { describe, expect, it } from "vitest";
import { clampSeekPosition, formatTime, mediaTrackKey, progressRatio, projectedPosition, reconcileReportedPosition, shouldShowIdleClock } from "./mediaClock";
import type { MediaState } from "$lib/api/types";

const state = (overrides: Partial<MediaState> = {}): MediaState => ({
  title: "Test",
  artist: "Artist",
  albumArt: "",
  isPlaying: true,
  positionMs: 20_000,
  durationMs: 100_000,
  lastUpdatedTimestamp: 1_000,
  source: "test",
  sourceDisplay: "Test",
  ...overrides,
});

describe("shared media clock", () => {
  it("projects playing media from its backend timestamp", () => {
    expect(projectedPosition(state(), 6_000)).toBe(25_000);
  });

  it("freezes paused media", () => {
    expect(projectedPosition(state({ isPlaying: false }), 90_000)).toBe(20_000);
  });

  it("clamps position and handles missing duration", () => {
    expect(projectedPosition(state({ positionMs: 99_000 }), 9_000)).toBe(100_000);
    expect(progressRatio(state({ durationMs: 0 }), 9_000)).toBe(0);
  });

  it("formats stable tabular timestamps", () => {
    expect(formatTime(0)).toBe("0:00");
    expect(formatTime(244_000)).toBe("4:04");
  });

  it("normalizes track identity across punctuation and casing", () => {
    expect(mediaTrackKey("Midnight City", "M83")).toBe(mediaTrackKey("midnight-city", "m83"));
  });

  it("clamps seek previews and shows the clock only without a media session", () => {
    expect(clampSeekPosition(-100, 10_000)).toBe(0);
    expect(clampSeekPosition(12_000, 10_000)).toBe(10_000);
    expect(shouldShowIdleClock(false)).toBe(true);
    expect(shouldShowIdleClock(true)).toBe(false);
  });

  it("projects across repeated snapshots instead of resetting every poll", () => {
    expect(reconcileReportedPosition(43_500, 42_000, 180_000, true, false, 42_000)).toBe(43_500);
    expect(reconcileReportedPosition(44_500, 42_000, 180_000, true, false, 42_000)).toBe(44_500);
  });

  it("accepts backwards seeks while paused and a restart at zero", () => {
    expect(reconcileReportedPosition(42_500, 12_000, 180_000, false, false, 42_000)).toBe(12_000);
    expect(reconcileReportedPosition(42_500, 0, 180_000, true, false, 42_000)).toBe(0);
  });

  it("freezes repeated paused reports and resets for a different track", () => {
    expect(reconcileReportedPosition(42_500, 42_000, 180_000, false, false, 42_000)).toBe(42_500);
    expect(reconcileReportedPosition(42_500, 0, 180_000, false, true, 0)).toBe(0);
  });
});
