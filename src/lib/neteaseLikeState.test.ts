import { beforeEach, expect, it, vi } from "vitest";
import { get } from "svelte/store";
vi.mock("./api/media", () => ({ mediaApi: { neteaseSongLike: vi.fn() } }));
import { mediaApi } from "./api/media";
import { neteaseLikeState, acceptNeteaseLikeState } from "./neteaseLikeState";
const read = vi.mocked(mediaApi.neteaseSongLike);
const song = { trackId: "1", playId: "play-1", title: "Song", artist: "Artist", liked: false };
beforeEach(() => { vi.clearAllMocks(); });

it("reuses confirmed state and coalesces reads across remounted consumers", async () => {
  read.mockResolvedValue(song);
  const first = neteaseLikeState("epoch:1", "Song", "Artist");
  await Promise.all([first.load(), first.load()]);
  const reopened = neteaseLikeState("epoch:1", "Song", "Artist");
  await reopened.load();
  expect(reopened).toBe(first);
  expect(get(reopened).value).toEqual(song);
  expect(read).toHaveBeenCalledTimes(1);
  await neteaseLikeState("epoch:2", "Next", "Artist").load();
  expect(read).toHaveBeenCalledTimes(2);
});

it("keeps confirmed writes across reopening and retains prior state on failure", async () => {
  read.mockResolvedValueOnce(song).mockResolvedValueOnce(song).mockResolvedValueOnce({ ...song, liked: true }).mockRejectedValueOnce(new Error("offline"));
  const state = neteaseLikeState("write:1", "Song", "Artist");
  await state.load();
  await state.toggle();
  await neteaseLikeState("write:1", "Song", "Artist").load();
  expect(get(state).value?.liked).toBe(true);
  expect(read).toHaveBeenCalledTimes(3);
  await state.toggle();
  expect(get(state).value?.liked).toBe(true);
  expect(get(state).failed).toBe(true);
});

it("an old track request updates only its own cache entry", async () => {
  let resolve!: (value: typeof song) => void;
  read.mockImplementationOnce(() => new Promise(done => { resolve = done; })).mockResolvedValueOnce({ ...song, trackId: "2", liked: true });
  const old = neteaseLikeState("race:1", "Old", "Artist");
  const pending = old.load();
  const current = neteaseLikeState("race:2", "New", "Artist");
  await current.load();
  resolve(song);
  await pending;
  expect(get(current).value?.trackId).toBe("2");
  expect(get(current).value?.liked).toBe(true);
});

it("shares confirmed favorite changes without another provider read", async () => {
  read.mockResolvedValue(song);
  const state = neteaseLikeState("event:1", "Song", "Artist");
  await state.load();
  acceptNeteaseLikeState({ ...song, liked: true });
  await state.load();
  expect(get(state).value?.liked).toBe(true);
  expect(read).toHaveBeenCalledTimes(1);
});

it("an explicit click recovers from a failed initial read", async () => {
  read.mockRejectedValueOnce(new Error("starting")).mockResolvedValueOnce(song).mockResolvedValueOnce({ ...song, liked: true });
  const state = neteaseLikeState("recovery:1", "Song", "Artist");
  await state.load();
  expect(get(state).value).toBeNull();
  await state.toggle();
  expect(get(state).value?.liked).toBe(true);
  expect(get(state).failed).toBe(false);
});

it("uses a fresh play instance when the same song has restarted", async () => {
  const fresh = { ...song, playId: "new-play", liked: true };
  read.mockResolvedValueOnce(song).mockResolvedValueOnce(fresh).mockResolvedValueOnce({ ...fresh, liked: false });
  const state = neteaseLikeState("restarted:1", "Song", "Artist");
  await state.load();
  await state.toggle();
  expect(read).toHaveBeenLastCalledWith("Song", "Artist", fresh, false);
  expect(get(state).value?.liked).toBe(false);
});

it("automatically recovers a transient initial read and stops reading after success", async () => {
  vi.useFakeTimers();
  try {
    read.mockRejectedValueOnce(new Error("player initializing")).mockResolvedValueOnce(song);
    const state = neteaseLikeState("retry:1", "Song", "Artist");
    await state.load();
    expect(get(state).error).toBe("player initializing");
    await vi.advanceTimersByTimeAsync(500);
    expect(get(state).value).toEqual(song);
    expect(get(state).failed).toBe(false);
    await vi.advanceTimersByTimeAsync(10000);
    await state.load();
    expect(read).toHaveBeenCalledTimes(2);
  } finally { vi.useRealTimers(); }
});

it("stops after three failed reads and exposes the real error instead of loading forever", async () => {
  vi.useFakeTimers();
  try {
    read.mockRejectedValue(new Error("CDP unavailable"));
    const state = neteaseLikeState("retry:2", "Song", "Artist");
    await state.load();
    await vi.advanceTimersByTimeAsync(10000);
    await state.load();
    expect(read).toHaveBeenCalledTimes(3);
    expect(get(state).loading).toBe(false);
    expect(get(state).error).toBe("CDP unavailable");
    expect(get(state).failed).toBe(true);
  } finally { vi.useRealTimers(); }
});
