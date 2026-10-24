<script lang="ts">
  import IslandSurface from "../src/lib/components/island/IslandSurface.svelte";
  import type { MediaState } from "../src/lib/api/types";
  import { clampSeekPosition } from "../src/lib/mediaClock";
  import { createSeekHandler, createSeekRequestGate, formatSeekError, seekSnapshotAllowsRollback } from "../src/lib/seekRequest";

  let seekError = $state("");
  let positionMs = $state(12_000);
  let durationMs = $state(180_000);
  let nativeRevision = 0;
  let latestSnapshotPosition = 0;
  let currentTrackKey = "fixture-track";
  let disposed = false;
  let resolvePending: ((result: { trackId: string; positionMs: number; durationMs: number }) => void) | undefined;
  let rejectPending: ((error: unknown) => void) | undefined;
  const seekGate = createSeekRequestGate();

  const media: MediaState = {
    title: "Seek test track",
    artist: "Seek test artist",
    albumArt: "",
    isPlaying: true,
    positionMs: 12_000,
    durationMs: 180_000,
    lastUpdatedTimestamp: Date.now(),
    source: "netease",
    sourceDisplay: "NetEase",
    capabilities: { seek: true },
  };

  const handleSeek = createSeekHandler({
    gate: seekGate,
    createContext: (requestedPosition: number, requestId: number) => ({
      next: clampSeekPosition(requestedPosition, durationMs),
      track: currentTrackKey,
      requestId,
      snapshotRevision: nativeRevision,
    }),
    isCurrentTrack: (context) => context.track === currentTrackKey,
    isDisposed: () => disposed,
    request: (context) => new Promise((resolve, reject) => {
      window.__seekMediaInvokes.push({ command: "seek_media", positionMs: context.next });
      resolvePending = resolve;
      rejectPending = reject;
    }),
    onBegin: (context) => {
      seekError = "";
      positionMs = context.next;
    },
    onConfirmed: (result) => {
      seekError = "";
      if (!result) return;
      positionMs = result.positionMs;
      durationMs = result.durationMs;
    },
    onFailed: (_error, context) => {
      if (seekSnapshotAllowsRollback(context.snapshotRevision, nativeRevision)) {
        positionMs = latestSnapshotPosition;
      }
    },
    onError: (error) => seekError = formatSeekError(error),
  });

  window.__setSeekError = (message) => seekError = message;
  window.__pushNativeSnapshot = (nextPosition) => {
    nativeRevision++;
    latestSnapshotPosition = nextPosition;
    positionMs = nextPosition;
  };
  window.__setDisplayedPosition = (nextPosition) => positionMs = nextPosition;
  window.__resolveSeek = (result) => resolvePending?.(result);
  window.__rejectSeek = (error) => rejectPending?.(error);
  window.__seekState = () => ({ positionMs, error: seekError, nativeRevision });
</script>

<IslandSurface
  {media}
  mode="expanded"
  position={positionMs}
  enableAnimations={false}
  reduceAnimations={true}
  showSpectrum={false}
  onSeek={handleSeek}
  {seekError}
/>
