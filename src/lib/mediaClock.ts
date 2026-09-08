import type { MediaState } from "$lib/api/types";

export function projectedPosition(media: MediaState, now = Date.now()): number {
  if (!media.durationMs) return 0;
  const elapsed = media.isPlaying ? Math.max(0, now - media.lastUpdatedTimestamp) : 0;
  return Math.min(media.durationMs, Math.max(0, media.positionMs + elapsed));
}

export function progressRatio(media: MediaState, now = Date.now()): number {
  return media.durationMs ? projectedPosition(media, now) / media.durationMs : 0;
}

export function formatTime(ms: number): string {
  if (!Number.isFinite(ms) || ms < 0) return "0:00";
  const seconds = Math.floor(ms / 1000);
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}
