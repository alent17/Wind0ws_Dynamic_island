import { writable } from "svelte/store";
import { artworkValue } from "./mediaArtwork";
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
let refreshInFlight: Promise<void> | undefined;
let refreshGeneration = 0;

function publish(next: MediaState) {
  updateVersion += 1;
  if (artworkValue({...next}) !== undefined) artworkVersion += 1;
  media.update((previous) => ({
    ...next,
    albumArt: artworkValue({...next}) ?? (
      previous.title === next.title && previous.artist === next.artist && previous.source === next.source
        ? previous.albumArt
        : ""
    ),
  }));
}

function refresh(epoch: number): Promise<void> {
  if (refreshInFlight && refreshGeneration === epoch) return refreshInFlight;

  const versionAtStart = updateVersion;
  const artworkAtStart = artworkVersion;
  const request = mediaApi.getMediaInfo();
  const pending = request.then((next) => {
    // A media event received during this request is newer than its snapshot.
    if (epoch !== generation || consumers === 0) return;
    if (versionAtStart === updateVersion) { publish(next); return; }
    // A clock-only delta must not discard a delayed full cover snapshot.
    // Explicit artwork updates/clears since the request remain newer.
    if (artworkAtStart === artworkVersion) {
      const cover = artworkValue({...next});
      if (cover !== undefined) media.update(previous =>
        previous.title === next.title && previous.artist === next.artist && previous.source === next.source
          ? {...previous,albumArt:cover} : previous);
    }
  }).catch(() => {
    // Browser Studio deliberately keeps deterministic preview data.
  }).finally(() => {
    if (refreshInFlight === pending) refreshInFlight = undefined;
  });
  refreshInFlight = pending;
  refreshGeneration = epoch;
  return pending;
}

export async function connectMedia(): Promise<() => void> {
  consumers++;
  if (consumers === 1) {
    const epoch = ++generation;
    void refresh(epoch);
    void listen<MediaState>("media-update", ({ payload }) => {
      if (epoch === generation && consumers) publish(payload);
    }).then((dispose) => {
      if (epoch !== generation || consumers === 0) dispose();
      else unlisten = dispose;
    }).catch(() => {
      if (epoch === generation && consumers) poll = setInterval(() => void refresh(epoch), 2500);
    });
  }
  let released = false;
  return () => {
    if (released) return;
    released = true;
    consumers -= 1;
    if (consumers > 0) return;
    generation++;
    unlisten?.();
    unlisten = undefined;
    if (poll) clearInterval(poll);
    poll = undefined;
  };
}
