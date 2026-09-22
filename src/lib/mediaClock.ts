import type { MediaState } from "$lib/api/types";

function normalizeTrackText(value: string): string {
  return value.normalize("NFKC").toLocaleLowerCase().replace(/[^\p{L}\p{N}]/gu, "");
}

export function mediaTrackKey(title: string, artist: string): string {
  return `${normalizeTrackText(title)}|${normalizeTrackText(artist)}`;
}

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

export function clampSeekPosition(positionMs: number, durationMs: number): number {
  if (!Number.isFinite(positionMs) || !Number.isFinite(durationMs) || durationMs <= 0) return 0;
  return Math.min(durationMs, Math.max(0, positionMs));
}

export function reconcileReportedPosition(
  previousPositionMs: number,
  reportedPositionMs: number,
  durationMs: number,
  isPlaying: boolean,
  trackChanged = false,
  previousReportedPositionMs?: number,
): number {
  const previous = Math.max(0, Number.isFinite(previousPositionMs) ? previousPositionMs : 0);
  const reported = Math.max(0, Number.isFinite(reportedPositionMs) ? reportedPositionMs : 0);
  const clamp = (value: number) => durationMs > 0 ? Math.min(durationMs, value) : value;
  // A provider can publish the same snapshot for many seconds. Re-anchoring
  // every poll freezes the projected clock. A changed report, including zero
  // or a paused seek, is authoritative.
  if (!trackChanged && reported === previousReportedPositionMs) return clamp(previous);
  return clamp(reported);
}

export function shouldShowIdleClock(hasMediaSession: boolean): boolean {
  return !hasMediaSession;
}
