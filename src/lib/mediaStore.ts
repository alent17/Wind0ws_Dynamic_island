import { writable } from "svelte/store";
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

async function refresh() {
  try {
    const next = await mediaApi.getMediaInfo();
    media.set(next);
  } catch {
    // Browser Studio deliberately keeps deterministic preview data.
  }
}

export async function connectMedia(): Promise<() => void> {
  consumers += 1;
  if (consumers === 1) {
    await refresh();
    try {
      unlisten = await listen<MediaState>("media-update", ({ payload }) => media.set(payload));
    } catch {
      poll = setInterval(refresh, 2500);
    }
  }
  return () => {
    consumers = Math.max(0, consumers - 1);
    if (!consumers) {
      unlisten?.();
      unlisten = undefined;
      if (poll) clearInterval(poll);
      poll = undefined;
    }
  };
}
