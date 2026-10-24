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
  const release = connectMedia();
  release();

  const dispose = vi.fn();
  listener.resolve(dispose);
  await Promise.resolve();
  expect(dispose).toHaveBeenCalledTimes(1);
});

it("disposes a stale listener that resolves after a new connection has started", async () => {
  const staleListener = deferred<() => void>();
  const staleDispose = vi.fn();
  const currentDispose = vi.fn();
  api.listen.mockReturnValueOnce(staleListener.promise).mockResolvedValueOnce(currentDispose);
  const { connectMedia } = await import("./mediaStore");
  const releaseOld = connectMedia();
  releaseOld();
  const releaseCurrent = connectMedia();
  await vi.waitFor(() => expect(api.getMediaInfo).toHaveBeenCalledTimes(1));
  staleListener.resolve(staleDispose);
  await vi.waitFor(() => expect(staleDispose).toHaveBeenCalledTimes(1));
  expect(api.getMediaInfo).toHaveBeenCalledTimes(1);
  releaseCurrent();
  expect(currentDispose).toHaveBeenCalledTimes(1);
});

it("registers the listener before its initial snapshot and keeps events received during it", async () => {
  const snapshot = deferred<MediaState>();
  const listener = deferred<() => void>();
  const order: string[] = [];
  api.getMediaInfo.mockReturnValue(snapshot.promise);
  let onUpdate!: (event: { payload: MediaState }) => void;
  api.listen.mockImplementation((_event, callback) => {
    order.push("listen");
    onUpdate = callback;
    return listener.promise;
  });
  api.getMediaInfo.mockImplementation(() => { order.push("snapshot"); return snapshot.promise; });
  const { connectMedia, media } = await import("./mediaStore");
  let current!: MediaState;
  const unsubscribe = media.subscribe((value) => { current = value; });
  const release = await connectMedia();

  expect(order).toEqual(["listen"]);
  listener.resolve(vi.fn());
  await vi.waitFor(() => expect(order).toEqual(["listen", "snapshot"]));
  onUpdate({ payload: state("New", "cover.png") });
  snapshot.resolve(state("Old"));
  await snapshot.promise;
  await Promise.resolve();
  expect(current.title).toBe("New");
  release();
  unsubscribe();
});

it("shares one listener and one initial snapshot across concurrent consumers", async () => {
  const listener = deferred<() => void>();
  api.listen.mockReturnValue(listener.promise);
  const readyA = vi.fn();
  const readyB = vi.fn();
  const { connectMedia } = await import("./mediaStore");
  const releaseA = connectMedia(readyA);
  const releaseB = connectMedia(readyB);
  expect(api.listen).toHaveBeenCalledTimes(1);
  expect(api.getMediaInfo).not.toHaveBeenCalled();
  listener.resolve(vi.fn());
  await vi.waitFor(() => expect(api.getMediaInfo).toHaveBeenCalledTimes(1));
  await vi.waitFor(() => { expect(readyA).toHaveBeenCalledTimes(1); expect(readyB).toHaveBeenCalledTimes(1); });
  releaseA();
  releaseB();
});

it("takes the first snapshot and retains the 2.5 second polling fallback when listen fails", async () => {
  vi.useFakeTimers();
  api.listen.mockRejectedValue(new Error("listener unavailable"));
  api.getMediaInfo.mockResolvedValue(state("Fallback snapshot"));
  const ready = vi.fn();
  const { connectMedia, media } = await import("./mediaStore");
  let current!: MediaState;
  const unsubscribe = media.subscribe(value => { current = value; });
  const release = connectMedia(ready);
  await vi.waitFor(() => expect(api.getMediaInfo).toHaveBeenCalledTimes(1));
  await vi.waitFor(() => expect(current.title).toBe("Fallback snapshot"));
  expect(ready).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(2500);
  expect(api.getMediaInfo).toHaveBeenCalledTimes(2);
  release();
  await vi.advanceTimersByTimeAsync(5000);
  expect(api.getMediaInfo).toHaveBeenCalledTimes(2);
  unsubscribe();
  vi.useRealTimers();
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

it("receives delayed same-track artwork and treats omissions separately from clears", async () => {
  let update!: (event: {payload:MediaState}) => void;
  api.listen.mockImplementation((_event,callback)=>{update=callback;return Promise.resolve(vi.fn());});
  const {connectMedia,media}=await import("./mediaStore");
  let current!:MediaState;const unsubscribe=media.subscribe(value=>current=value);
  const release=await connectMedia();await Promise.resolve();
  update({payload:state("Same","new-cover")});
  const delta={...state("Same")};delete (delta as Partial<MediaState>).albumArt;
  update({payload:delta});expect(current.albumArt).toBe("new-cover");
  update({payload:state("Same","")});expect(current.albumArt).toBe("");
  update({payload:state("Same","late-cover")});expect(current.albumArt).toBe("late-cover");
  release();unsubscribe();
});

it("keeps delayed initial artwork after an omission but not after an explicit clear", async () => {
  const snapshot=deferred<MediaState>();api.getMediaInfo.mockReturnValue(snapshot.promise);
  let update!:(event:{payload:MediaState})=>void;
  api.listen.mockImplementation((_event,callback)=>{update=callback;return Promise.resolve(vi.fn());});
  const {connectMedia,media}=await import("./mediaStore");let current!:MediaState;
  const unsubscribe=media.subscribe(value=>current=value);const release=await connectMedia();
  const delta={...state("Same")};delete(delta as Partial<MediaState>).albumArt;update({payload:delta});
  snapshot.resolve(state("Same","full-cover"));await snapshot.promise;await Promise.resolve();
  expect(current.albumArt).toBe("full-cover");release();unsubscribe();
});
it("does not revive artwork cleared while the initial snapshot was pending", async () => {
  const snapshot=deferred<MediaState>();api.getMediaInfo.mockReturnValue(snapshot.promise);
  let update!:(event:{payload:MediaState})=>void;
  api.listen.mockImplementation((_event,callback)=>{update=callback;return Promise.resolve(vi.fn());});
  const {connectMedia,media}=await import("./mediaStore");let current!:MediaState;
  const unsubscribe=media.subscribe(value=>current=value);const release=await connectMedia();
  update({payload:state("Same","")});snapshot.resolve(state("Same","old-cover"));
  await snapshot.promise;await Promise.resolve();expect(current.albumArt).toBe("");release();unsubscribe();
});

it("does not fill artwork from an A snapshot after A-to-B-to-A listener events", async () => {
  const snapshot=deferred<MediaState>();api.getMediaInfo.mockReturnValue(snapshot.promise);
  let update!:(event:{payload:MediaState})=>void;
  api.listen.mockImplementation((_event,callback)=>{update=callback;return Promise.resolve(vi.fn());});
  const {connectMedia,media}=await import("./mediaStore");let current!:MediaState;
  const unsubscribe=media.subscribe(value=>current=value);const release=connectMedia();
  await vi.waitFor(()=>expect(api.getMediaInfo).toHaveBeenCalledTimes(1));
  const a={...state("Track A")};delete (a as Partial<MediaState>).albumArt;
  const b={...state("Track B")};delete (b as Partial<MediaState>).albumArt;
  update({payload:a});update({payload:b});update({payload:a});
  snapshot.resolve(state("Track A","stale-cover"));await snapshot.promise;await Promise.resolve();
  expect(current.title).toBe("Track A");expect(current.albumArt).toBe("");
  release();unsubscribe();
});

it("rejects lower and unstamped events after entering native stamp mode", async () => {
  let update!:(event:{payload:MediaState})=>void;
  api.listen.mockImplementation((_event,callback)=>{update=callback;return Promise.resolve(vi.fn());});
  const {connectMedia,media}=await import("./mediaStore");let current!:MediaState;
  const unsubscribe=media.subscribe(value=>current=value);const release=connectMedia();await vi.waitFor(()=>expect(api.getMediaInfo).toHaveBeenCalled());
  update({payload:{...state("Revision 2","cover-2"),nativeEpoch:"epoch-a",nativeRevision:2}});
  update({payload:{...state("Revision 1","stale-cover"),nativeEpoch:"epoch-a",nativeRevision:1}});
  update({payload:state("Legacy rollback","legacy-cover")});
  expect(current.title).toBe("Revision 2");expect(current.albumArt).toBe("cover-2");
  release();unsubscribe();
});

it("accepts trusted epoch switches, rejects retired epochs, and resets epoch history per connection", async () => {
  const updates:Array<(event:{payload:MediaState})=>void>=[];
  api.listen.mockImplementation((_event,callback)=>{updates.push(callback);return Promise.resolve(vi.fn());});
  const {connectMedia,media}=await import("./mediaStore");let current!:MediaState;
  const unsubscribe=media.subscribe(value=>current=value);
  const first=connectMedia();await vi.waitFor(()=>expect(updates).toHaveLength(1));
  updates[0]({payload:{...state("Epoch A"),nativeEpoch:"epoch-a",nativeRevision:4}});
  updates[0]({payload:{...state("Epoch B"),nativeEpoch:"epoch-b",nativeRevision:0}});
  updates[0]({payload:{...state("Retired A"),nativeEpoch:"epoch-a",nativeRevision:99}});
  expect(current.title).toBe("Epoch B");
  first();
  const second=connectMedia();await vi.waitFor(()=>expect(updates).toHaveLength(2));
  updates[1]({payload:{...state("Epoch A after reconnect"),nativeEpoch:"epoch-a",nativeRevision:0}});
  expect(current.title).toBe("Epoch A after reconnect");
  second();unsubscribe();
});

it("lets a same-revision command fill missing art without reverting event playback", async () => {
  const snapshot=deferred<MediaState>();api.getMediaInfo.mockReturnValue(snapshot.promise);
  let update!:(event:{payload:MediaState})=>void;
  api.listen.mockImplementation((_event,callback)=>{update=callback;return Promise.resolve(vi.fn());});
  const {connectMedia,media}=await import("./mediaStore");let current!:MediaState;
  const unsubscribe=media.subscribe(value=>current=value);const release=connectMedia();
  await vi.waitFor(()=>expect(api.getMediaInfo).toHaveBeenCalledTimes(1));
  const event={...state("Same basis"),nativeEpoch:"snapshot-epoch",nativeRevision:7,positionMs:77_000,isPlaying:false};
  delete (event as Partial<MediaState>).albumArt;
  update({payload:event});
  snapshot.resolve({...event,albumArt:"command-cover",positionMs:1_000,isPlaying:true});
  await snapshot.promise;await Promise.resolve();
  expect(current).toMatchObject({title:"Same basis",positionMs:77_000,isPlaying:false,albumArt:"command-cover"});
  release();unsubscribe();
});

it("rejects an older stamped command snapshot even when it carries artwork", async () => {
  const snapshot=deferred<MediaState>();api.getMediaInfo.mockReturnValue(snapshot.promise);
  let update!:(event:{payload:MediaState})=>void;
  api.listen.mockImplementation((_event,callback)=>{update=callback;return Promise.resolve(vi.fn());});
  const {connectMedia,media}=await import("./mediaStore");let current!:MediaState;
  const unsubscribe=media.subscribe(value=>current=value);const release=connectMedia();
  await vi.waitFor(()=>expect(api.getMediaInfo).toHaveBeenCalledTimes(1));
  update({payload:{...state("Current", "current-cover"),nativeEpoch:"snapshot-epoch",nativeRevision:8,positionMs:80_000}});
  snapshot.resolve({...state("Old", "stale-cover"),nativeEpoch:"snapshot-epoch",nativeRevision:7,positionMs:7_000});
  await snapshot.promise;await Promise.resolve();
  expect(current).toMatchObject({title:"Current",positionMs:80_000,albumArt:"current-cover"});
  release();unsubscribe();
});

it("accepts an actor command snapshot epoch switch and retires the prior event epoch", async () => {
  const snapshot=deferred<MediaState>();api.getMediaInfo.mockReturnValue(snapshot.promise);
  let update!:(event:{payload:MediaState})=>void;
  api.listen.mockImplementation((_event,callback)=>{update=callback;return Promise.resolve(vi.fn());});
  const {connectMedia,media}=await import("./mediaStore");let current!:MediaState;
  const unsubscribe=media.subscribe(value=>current=value);const release=connectMedia();
  await vi.waitFor(()=>expect(api.getMediaInfo).toHaveBeenCalledTimes(1));
  update({payload:{...state("Old epoch"),nativeEpoch:"epoch-old",nativeRevision:5}});
  snapshot.resolve({...state("Command established epoch"),nativeEpoch:"epoch-new",nativeRevision:0});
  await snapshot.promise;await Promise.resolve();
  expect(current.title).toBe("Command established epoch");
  update({payload:{...state("Retired event"),nativeEpoch:"epoch-old",nativeRevision:99}});
  expect(current.title).toBe("Command established epoch");
  release();unsubscribe();
});

it("does not treat a same-revision explicit clear as missing artwork on a later snapshot", async () => {
  vi.useFakeTimers();
  api.listen.mockRejectedValue(new Error("use actor snapshot fallback"));
  api.getMediaInfo
    .mockResolvedValueOnce({...state("Cleared", ""),nativeEpoch:"clear-epoch",nativeRevision:3})
    .mockResolvedValueOnce({...state("Cleared", "old-cover"),nativeEpoch:"clear-epoch",nativeRevision:3});
  const {connectMedia,media}=await import("./mediaStore");let current!:MediaState;
  const unsubscribe=media.subscribe(value=>current=value);const release=connectMedia();
  await vi.waitFor(()=>expect(api.getMediaInfo).toHaveBeenCalledTimes(1));
  expect(current.albumArt).toBe("");
  await vi.advanceTimersByTimeAsync(2500);
  expect(api.getMediaInfo).toHaveBeenCalledTimes(2);
  expect(current.albumArt).toBe("");
  release();unsubscribe();vi.useRealTimers();
});
