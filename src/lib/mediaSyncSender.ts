import { withoutArtwork } from "./mediaArtwork";
import { isTransportDiagnosticsEnabled, recordTransportCounter, runTransportEvent } from "./transportDiagnostics";
import type { MediaState } from "$lib/api/types";

export type AppMediaOverlay = {
  nativeEpoch: string;
  nativeRevision: number;
  trackKey: string;
  title: string;
  artist: string;
  albumArt: string;
  artworkResolved: boolean;
  positionMs: number;
  lastUpdatedTimestamp: number;
};
export type MediaSyncSnapshot = MediaState & {
  artworkResolved?: boolean;
  appSessionId?: string;
  overlaySeq?: number;
  appOverlay?: AppMediaOverlay;
};
export type MediaSyncPayload = Omit<MediaSyncSnapshot, "albumArt"> & { albumArt?: string };

type ArtworkBaseline = { track: string; artwork: string };
type PendingSnapshot = { snapshot: MediaSyncSnapshot; generation: number };

export type MediaSyncSenderOptions = {
  dispatch: (payload: MediaSyncPayload) => Promise<unknown> | unknown;
  onError?: (error: unknown) => void;
  retryBaseDelayMs?: number;
  retryMaxAttempts?: number;
};

function trackKey(snapshot: MediaSyncSnapshot) {
  return `${snapshot.source}|${snapshot.title}|${snapshot.artist}`;
}

/** Serialize App-to-Floating syncs while retaining the latest complete local snapshot. */
export function createMediaSyncSender({
  dispatch,
  onError,
  retryBaseDelayMs = 250,
  retryMaxAttempts = 3,
}: MediaSyncSenderOptions) {
  let inFlight: PendingSnapshot | undefined;
  let pending: PendingSnapshot | undefined;
  let acknowledged: ArtworkBaseline | undefined;
  let generation = 0;
  let generationToken: string | undefined;
  let consumerActive = false;
  let retryTimer: ReturnType<typeof setTimeout> | undefined;
  let retryCount = 0;
  let disposed = false;

  function payloadFor(snapshot: MediaSyncSnapshot): MediaSyncPayload {
    if (snapshot.appOverlay) {
      // Stamped receivers use the session-scoped overlay and discard the
      // top-level cover. Keep the overlay cover for recovery after a rejected
      // out-of-order event, while avoiding two copies in every IPC payload.
      const { albumArt: _topLevelArtwork, ...payload } = snapshot;
      return payload;
    }
    if (acknowledged?.track === trackKey(snapshot) && acknowledged.artwork === snapshot.albumArt) {
      return withoutArtwork({ ...snapshot }) as MediaSyncPayload;
    }
    return { ...snapshot };
  }

  function report(error: unknown) {
    try {
      onError?.(error);
    } catch {
      // Error reporting must not interrupt delivery or retry state.
    }
  }

  function scheduleRetry() {
    if (disposed || retryTimer || !pending || retryCount >= retryMaxAttempts) return;
    if (isTransportDiagnosticsEnabled()) recordTransportCounter("mediaSync", "retries");
    const delay = Math.max(1, retryBaseDelayMs) * 2 ** retryCount;
    retryCount += 1;
    retryTimer = setTimeout(() => {
      retryTimer = undefined;
      pump();
    }, delay);
  }

  function completeSuccess(candidate: PendingSnapshot) {
    if (disposed || inFlight !== candidate) return;
    inFlight = undefined;
    if (candidate.generation === generation) {
      acknowledged = {
        track: trackKey(candidate.snapshot),
        artwork: candidate.snapshot.albumArt,
      };
      retryCount = 0;
    }
    pump();
  }

  function completeFailure(candidate: PendingSnapshot, error: unknown) {
    if (disposed || inFlight !== candidate) return;
    inFlight = undefined;
    if (candidate.generation !== generation) {
      pump();
      return;
    }
    if (candidate.generation === generation && !pending) pending = candidate;
    report(error);
    scheduleRetry();
  }

  function pump() {
    if (disposed || !consumerActive || inFlight || retryTimer || !pending) return;
    const candidate = pending;
    pending = undefined;
    inFlight = candidate;

    let result: Promise<unknown> | unknown;
    try {
      const payload = payloadFor(candidate.snapshot);
      result = isTransportDiagnosticsEnabled()
        ? runTransportEvent("mediaSync", () => dispatch(payload))
        : dispatch(payload);
    } catch (error) {
      completeFailure(candidate, error);
      return;
    }
    void Promise.resolve(result).then(
      () => completeSuccess(candidate),
      (error: unknown) => completeFailure(candidate, error),
    );
  }

  return {
    offer(snapshot: MediaSyncSnapshot) {
      if (disposed || !consumerActive) return;
      if (isTransportDiagnosticsEnabled()) {
        recordTransportCounter("mediaSync", "offers");
        if (pending) recordTransportCounter("mediaSync", "replaced");
      }
      // Keep the full shape here. Artwork omission is decided only at dispatch,
      // against the last candidate that Tauri successfully accepted.
      pending = {
        snapshot: {
          ...snapshot,
          capabilities: snapshot.capabilities ? { ...snapshot.capabilities } : snapshot.capabilities,
        },
        generation,
      };
      retryCount = 0;
      if (retryTimer) {
        clearTimeout(retryTimer);
        retryTimer = undefined;
      }
      pump();
    },
    resetForConsumer(token: string) {
      if (disposed || !token || (token === generationToken && consumerActive)) return false;
      generationToken = token;
      consumerActive = true;
      generation += 1;
      acknowledged = undefined;
      pending = undefined;
      retryCount = 0;
      if (retryTimer) {
        clearTimeout(retryTimer);
        retryTimer = undefined;
      }
      return true;
    },
    pauseForConsumer(token: string) {
      if (disposed || !token || token !== generationToken || !consumerActive) return false;
      consumerActive = false;
      generation += 1;
      acknowledged = undefined;
      pending = undefined;
      retryCount = 0;
      if (retryTimer) {
        clearTimeout(retryTimer);
        retryTimer = undefined;
      }
      return true;
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      pending = undefined;
      if (retryTimer) {
        clearTimeout(retryTimer);
        retryTimer = undefined;
      }
    },
  };
}
