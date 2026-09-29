import { beforeEach, expect, it, vi } from "vitest";
import type { MediaState } from "$lib/api/types";

const api = vi.hoisted(() => ({ listen: vi.fn(), getMediaInfo: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: api.listen }));
vi.mock("$lib/api/media", () => ({ mediaApi: { getMediaInfo: api.getMediaInfo } }));

const state = (title: string, albumArt = ""): MediaState => ({
  title,
  artist: "Artist",
  albumArt,
  isPlaying: true,
  positionMs: 0,
  durationMs: 1000,
  lastUpdatedTimestamp: 0,
  source: "test",
  sourceDisplay: "Test",
  capabilities: { previous: false, playPause: true, next: false, seek: false, shuffle: false, repeat: false },
  shuffleActive: false,
  repeatMode: "none",
});

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => { resolve = complete; });
  return { promise, resolve };
}

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
  api.getMediaInfo.mockResolvedValue(state("Initial"));
  api.listen.mockResolvedValue(vi.fn());
});

it("shares one listener and releases it after the final consumer", async () => {
  const dispose = vi.fn();
  api.listen.mockResolvedValue(dispose);
  const { connectMedia } = await import("./mediaStore");
  const first = await connectMedia();
  const second = await connectMedia();
  await Promise.resolve();

  expect(api.listen).toHaveBeenCalledTimes(1);
  first();
  first();
  expect(dispose).not.toHaveBeenCalled();
  second();
  expect(dispose).toHaveBeenCalledTimes(1);
});

it("disposes a listener that resolves after the consumer leaves", async () => {
  const listener = deferred<() => void>();
  api.listen.mockReturnValue(listener.promise);
  const { connectMedia } = await import("./mediaStore");
  const release = await connectMedia();
  release();

  const dispose = vi.fn();
  listener.resolve(dispose);
  await Promise.resolve();
  expect(dispose).toHaveBeenCalledTimes(1);
});

it("keeps a newer event when the initial snapshot completes later", async () => {
  const snapshot = deferred<MediaState>();
  api.getMediaInfo.mockReturnValue(snapshot.promise);
  let onUpdate!: (event: { payload: MediaState }) => void;
  api.listen.mockImplementation((_event, callback) => {
    onUpdate = callback;
    return Promise.resolve(vi.fn());
  });
  const { connectMedia, media } = await import("./mediaStore");
  let current!: MediaState;
  const unsubscribe = media.subscribe((value) => { current = value; });
  const release = await connectMedia();

  onUpdate({ payload: state("New", "cover.png") });
  snapshot.resolve(state("Old"));
  await snapshot.promise;
  await Promise.resolve();
  expect(current.title).toBe("New");
  release();
  unsubscribe();
});

it("ignores a snapshot from a previous connection after reconnecting", async () => {
  const oldSnapshot = deferred<MediaState>();
  api.getMediaInfo.mockReturnValueOnce(oldSnapshot.promise).mockResolvedValueOnce(state("New"));
  const { connectMedia, media } = await import("./mediaStore");
  let current!: MediaState;
  const unsubscribe = media.subscribe((value) => { current = value; });
  const first = await connectMedia();
  first();
  const second = await connectMedia();
  await Promise.resolve();
  expect(current.title).toBe("New");

  oldSnapshot.resolve(state("Old"));
  await oldSnapshot.promise;
  await Promise.resolve();
  expect(current.title).toBe("New");
  second();
  unsubscribe();
});
