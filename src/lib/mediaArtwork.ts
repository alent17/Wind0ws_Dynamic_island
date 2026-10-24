/** An absent field is a delta omission; an empty/null field explicitly clears. */
export function artworkValue(payload: Record<string, unknown>): string | undefined {
  for (const key of ["albumArt", "thumbnail", "coverUrl", "api_cover_url", "image"]) {
    if (!Object.hasOwn(payload, key)) continue;
    const value = payload[key];
    if (value === null) return "";
    if (typeof value === "string") return value;
  }
  return undefined;
}

export type ArtworkSnapshotToken = { events: number; artwork: number; trackGeneration: number; trackKey: string };
export type SnapshotDecision = "accept" | "artwork-only" | "discard";
export function snapshotDecision(start: ArtworkSnapshotToken, current: ArtworkSnapshotToken, sameTrack: boolean): SnapshotDecision {
  // A matching key is insufficient after an intervening track change (A→B→A).
  // The one exception is an initially empty player receiving its first track
  // while the command snapshot is pending; that same-track event may still let
  // the snapshot fill a missing cover.
  const initialTrackArrival = start.trackKey === "" &&
    current.trackGeneration === start.trackGeneration + 1 && sameTrack;
  if (start.trackGeneration !== current.trackGeneration && !initialTrackArrival) return "discard";
  if (start.events !== current.events) {
    if (!sameTrack || start.artwork !== current.artwork) return "discard";
    // The command snapshot can only fill a missing cover. Its playback clock and
    // capabilities are older than the event that arrived while it was pending.
    return "artwork-only";
  }
  return "accept";
}

export function withoutArtwork<T extends Record<string, unknown>>(payload: T) {
  const copy = { ...payload };
  for (const key of ["albumArt", "thumbnail", "coverUrl", "api_cover_url", "image"]) delete copy[key];
  return copy;
}

export function mergeProgressAnchorFields(
  current: { progressKnown?: boolean; neteaseProgressAvailable?: boolean },
  incoming: Record<string, unknown>,
) {
  return {
    progressKnown: typeof incoming.progressKnown === "boolean" ? incoming.progressKnown : current.progressKnown,
    neteaseProgressAvailable: typeof incoming.neteaseProgressAvailable === "boolean"
      ? incoming.neteaseProgressAvailable
      : current.neteaseProgressAvailable,
  };
}

export type NativeMediaStamp = { nativeEpoch: string; nativeRevision: number };
export type NativeStampDecision = "legacy" | "initial" | "newer" | "same" | "older" | "switch" | "retired" | "invalid";

export function readNativeMediaStamp(payload: Record<string, unknown>): NativeMediaStamp | undefined {
  const epoch = payload.nativeEpoch;
  const revision = payload.nativeRevision;
  if (typeof epoch !== "string" || !epoch || !Number.isSafeInteger(revision) || Number(revision) < 0) return undefined;
  return { nativeEpoch: epoch, nativeRevision: Number(revision) };
}

/** Epochs are opaque; only revisions within the current epoch are ordered. */
export function nativeStampDecision(
  current: NativeMediaStamp | undefined,
  stampedMode: boolean,
  retiredEpochs: ReadonlySet<string>,
  incoming: NativeMediaStamp | undefined,
  maySwitchEpoch: boolean,
): NativeStampDecision {
  if (!incoming) return stampedMode ? "invalid" : "legacy";
  if (retiredEpochs.has(incoming.nativeEpoch)) return "retired";
  if (!current) return "initial";
  if (incoming.nativeEpoch !== current.nativeEpoch) return maySwitchEpoch ? "switch" : "retired";
  if (incoming.nativeRevision < current.nativeRevision) return "older";
  if (incoming.nativeRevision === current.nativeRevision) return "same";
  return "newer";
}

export type AppOverlayDecision = "accept" | "wrong-session" | "retired-session" | "invalid-sequence" | "old-sequence" | "wrong-basis" | "wrong-track";
export function appOverlayDecision(args: {
  incomingSession: unknown;
  currentSession: string | undefined;
  retiredSessions: ReadonlySet<string>;
  incomingSequence: unknown;
  currentSequence: number;
  basisMatches: boolean;
  trackMatches: boolean;
}): AppOverlayDecision {
  if (typeof args.incomingSession !== "string" || args.retiredSessions.has(args.incomingSession)) return "retired-session";
  if (!args.currentSession || args.incomingSession !== args.currentSession) return "wrong-session";
  if (!Number.isSafeInteger(args.incomingSequence) || Number(args.incomingSequence) < 1) return "invalid-sequence";
  if (Number(args.incomingSequence) <= args.currentSequence) return "old-sequence";
  if (!args.basisMatches) return "wrong-basis";
  if (!args.trackMatches) return "wrong-track";
  return "accept";
}
