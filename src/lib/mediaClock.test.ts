import { describe, expect, it } from "vitest";
import { formatTime, progressRatio, projectedPosition } from "./mediaClock";
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
});
