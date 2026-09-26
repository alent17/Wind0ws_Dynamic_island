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

let generation = 0;
let pending: Promise<MediaState> | undefined;
function publish(next: MediaState) {
  media.update(previous => ({...next, albumArt: next.albumArt || (previous.title === next.title && previous.artist === next.artist && previous.source === next.source ? previous.albumArt : "")}));
}
async function refresh(epoch: number) {
  try {
    const request = pending ??= mediaApi.getMediaInfo();
    try { const next = await request; if (epoch === generation && consumers) publish(next); }
    finally { if (pending === request) pending = undefined; }
  } catch { /* Browser previews keep demo data. */ }
}
export async function connectMedia(): Promise<() => void> {
  consumers++;
  if (consumers === 1) {
    const epoch = ++generation;
    void refresh(epoch);
    void listen<MediaState>("media-update", ({payload}) => {
      if (epoch === generation && consumers) publish(payload);
    }).then(dispose => {
      if (epoch !== generation || !consumers) dispose(); else unlisten = dispose;
    }).catch(() => {
      if (epoch === generation && consumers) poll = setInterval(() => void refresh(epoch), 2500);
    });
  }
  let released = false;
  return () => {
    if (released) return;
    released = true;
    if (--consumers) return;
    generation++;
    unlisten?.(); unlisten = undefined;
    if (poll) clearInterval(poll);
    poll = undefined;
  };
}
