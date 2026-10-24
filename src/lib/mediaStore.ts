import { writable } from "svelte/store";
import { artworkValue, nativeStampDecision, readNativeMediaStamp, type NativeMediaStamp } from "./mediaArtwork";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { mediaApi } from "$lib/api/media";
import type { MediaState } from "$lib/api/types";

const demoAlbumArt = new URL("../assets/icons/spotify.svg", import.meta.url).href;

export const DEMO_MEDIA: MediaState = {
  title: "Midnight City",
  artist: "M83 · Hurry Up, We’re Dreaming",
  albumArt: demoAlbumArt,
  isPlaying: true,
  positionMs: 112_000,
  durationMs: 244_000,
  lastUpdatedTimestamp: Date.now(),
  source: "spotify",
  sourceDisplay: "Spotify",
  capabilities: { previous: true, playPause: true, next: true, seek: false, shuffle: false, repeat: false },
  shuffleActive: false,
  repeatMode: "none",
};

export const media = writable<MediaState>(DEMO_MEDIA);
let consumers = 0;
let unlisten: UnlistenFn | undefined;
let poll: ReturnType<typeof setInterval> | undefined;
let generation = 0;
let updateVersion = 0;
let artworkVersion = 0;
let refreshInFlight: Promise<boolean> | undefined;
let refreshGeneration = 0;
let trackGeneration = 0;
let connectionTrackKey: string | undefined;
let connectionState: MediaState | undefined;
let connectionArtworkCleared = false;
let nativeStamp: NativeMediaStamp | undefined;
let nativeStampedMode = false;
let retiredNativeEpochs = new Set<string>();
let connectionReady = false;
const readyCallbacks = new Set<() => void>();

function trackKey(value: Pick<MediaState, "title" | "artist" | "source">) {
  return `${value.source || ""}|${value.title || ""}|${value.artist || ""}`;
}

function markReady(epoch: number) {
  if (epoch !== generation || consumers === 0) return;
  connectionReady = true;
  const callbacks = [...readyCallbacks];
  readyCallbacks.clear();
  for (const callback of callbacks) callback();
}

function publish(next: MediaState, epoch: number, acceptedDecision?: ReturnType<typeof nativeStampDecision>) {
  if (epoch !== generation || consumers === 0) return false;
  const stamp = readNativeMediaStamp({ ...next });
  const decision = acceptedDecision ?? nativeStampDecision(nativeStamp, nativeStampedMode, retiredNativeEpochs, stamp, true);
  if (["older", "retired", "invalid"].includes(decision)) return false;
  const current = connectionState;
  const nextTrackKey = trackKey(next);
  if (decision === "same") {
    if (!current || trackKey(current) !== nextTrackKey) return false;
    const cover = artworkValue({ ...next });
    if (!current.albumArt && !connectionArtworkCleared && cover) {
      connectionState = { ...current, albumArt: cover };
      connectionArtworkCleared = false;
      artworkVersion += 1;
      updateVersion += 1;
      media.update((previous) => trackKey(previous) === nextTrackKey ? { ...previous, albumArt: cover } : previous);
    }
    markReady(epoch);
    return true;
  }
  if (decision === "switch") {
    if (nativeStamp) retiredNativeEpochs.add(nativeStamp.nativeEpoch);
    nativeStamp = stamp;
    nativeStampedMode = true;
  } else if (decision === "initial" || decision === "newer") {
    if (stamp) {
      nativeStamp = stamp;
      nativeStampedMode = true;
    }
  }
  if (connectionTrackKey !== nextTrackKey) trackGeneration += 1;
  connectionTrackKey = nextTrackKey;
  updateVersion += 1;
  if (artworkValue({...next}) !== undefined) artworkVersion += 1;
  const sameTrack = current && trackKey(current) === nextTrackKey;
  const incomingArtwork = artworkValue({ ...next });
  const merged = { ...next, albumArt: incomingArtwork ?? (sameTrack ? current.albumArt : "") };
  if (!sameTrack || incomingArtwork !== undefined) connectionArtworkCleared = incomingArtwork === "";
  connectionState = merged;
  media.set(merged);
  // Track changes within this native connection are local ordering evidence
  // for command snapshots. The DEMO_MEDIA placeholder is not a live baseline.
  markReady(epoch);
  return true;
}

function fillSnapshotArtwork(next: MediaState, epoch: number, trackGenerationAtStart: number, artworkAtStart: number, trackKeyAtStart: string | undefined) {
  if (epoch !== generation || consumers === 0 || artworkAtStart !== artworkVersion) return false;
  const current = connectionState;
  const cover = artworkValue({ ...next });
  if (!current || !cover || current.albumArt || connectionArtworkCleared || trackKey(current) !== trackKey(next)) return false;
  // Preserve the initial empty→track exception while still rejecting A→B→A.
  const initialTrackArrival = trackKeyAtStart === undefined && connectionTrackKey === trackKey(next) && trackGeneration === trackGenerationAtStart + 1;
  if (trackGenerationAtStart !== trackGeneration && !initialTrackArrival) return false;
  if (trackKeyAtStart !== trackKey(next) && !initialTrackArrival) return false;
  connectionState = { ...current, albumArt: cover };
  connectionArtworkCleared = false;
  artworkVersion += 1;
  updateVersion += 1;
  media.update((previous) => trackKey(previous) === trackKey(next) ? { ...previous, albumArt: cover } : previous);
  return true;
}

function refresh(epoch: number): Promise<boolean> {
  if (refreshInFlight && refreshGeneration === epoch) return refreshInFlight;

  const versionAtStart = updateVersion;
  const artworkAtStart = artworkVersion;
  const trackGenerationAtStart = trackGeneration;
  const trackKeyAtStart = connectionTrackKey;
  const request = mediaApi.getMediaInfo();
  const pending = request.then((next) => {
    if (epoch !== generation || consumers === 0) return false;
    const stamp = readNativeMediaStamp({ ...next });
    const decision = nativeStampDecision(nativeStamp, nativeStampedMode, retiredNativeEpochs, stamp, true);
    if (["older", "retired", "invalid"].includes(decision)) return false;
    if (["initial", "newer", "switch"].includes(decision)) return publish(next, epoch, decision);
    if (decision === "same") {
      fillSnapshotArtwork(next, epoch, trackGenerationAtStart, artworkAtStart, trackKeyAtStart);
      markReady(epoch);
      return true;
    }
    if (versionAtStart === updateVersion) { publish(next, epoch, decision); return true; }
    // A clock-only delta must not discard a delayed full cover snapshot.
    // Explicit artwork updates/clears and track changes since the request
    // remain newer, including a return to the same track key (A→B→A).
    fillSnapshotArtwork(next, epoch, trackGenerationAtStart, artworkAtStart, trackKeyAtStart);
    markReady(epoch);
    return true;
  }).catch(() => {
    // Browser Studio deliberately keeps deterministic preview data.
    return false;
  }).finally(() => {
    if (refreshInFlight === pending) refreshInFlight = undefined;
  });
  refreshInFlight = pending;
  refreshGeneration = epoch;
  return pending;
}

function startFallbackPoll(epoch: number) {
  if (poll || epoch !== generation || consumers === 0) return;
  poll = setInterval(() => void refresh(epoch), 2500);
}

async function initializeConnection(epoch: number) {
  try {
    const dispose = await listen<MediaState>("media-update", ({ payload }) => {
      if (epoch === generation && consumers) publish(payload, epoch);
    });
    if (epoch !== generation || consumers === 0) {
      dispose();
      return;
    }
    unlisten = dispose;
  } catch {
    if (epoch !== generation || consumers === 0) return;
    startFallbackPoll(epoch);
  }
  // Register the listener first so updates during the command snapshot cannot
  // fall into a subscription gap. Do not make connectMedia wait on this work.
  if (epoch === generation && consumers) void refresh(epoch);
}

export function connectMedia(onReady?: () => void): () => void {
  const firstConsumer = consumers === 0;
  consumers++;
  if (firstConsumer) {
    ++generation;
    updateVersion = 0;
    artworkVersion = 0;
    trackGeneration = 0;
    connectionTrackKey = undefined;
    connectionState = undefined;
    connectionArtworkCleared = false;
    nativeStamp = undefined;
    nativeStampedMode = false;
    retiredNativeEpochs = new Set<string>();
    connectionReady = false;
    readyCallbacks.clear();
  }
  const epoch = generation;
  let readyCallback = onReady;
  const notifyReady = () => {
    if (!readyCallback) return;
    const callback = readyCallback;
    readyCallback = undefined;
    callback();
  };
  if (readyCallback && !connectionReady) readyCallbacks.add(notifyReady);
  else notifyReady();
  if (firstConsumer) void initializeConnection(epoch);

  let released = false;
  return () => {
    if (released) return;
    released = true;
    readyCallbacks.delete(notifyReady);
    consumers -= 1;
    if (consumers > 0) return;
    generation++;
    connectionReady = false;
    connectionState = undefined;
    connectionArtworkCleared = false;
    connectionTrackKey = undefined;
    nativeStamp = undefined;
    nativeStampedMode = false;
    retiredNativeEpochs = new Set<string>();
    updateVersion = 0;
    artworkVersion = 0;
    trackGeneration = 0;
    readyCallbacks.clear();
    unlisten?.();
    unlisten = undefined;
    if (poll) clearInterval(poll);
    poll = undefined;
  };
}
