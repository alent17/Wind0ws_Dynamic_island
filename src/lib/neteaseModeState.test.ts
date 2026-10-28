import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { get } from "svelte/store";
vi.mock("./api/media", () => ({ mediaApi: { getNeteasePlaybackMode: vi.fn() } }));
import { mediaApi } from "./api/media";
import { clearNeteaseModes, neteaseModeState } from "./neteaseModeState";
const read = vi.mocked(mediaApi.getNeteasePlaybackMode);
const releases: (() => void)[] = [];
beforeEach(() => { read.mockReset(); clearNeteaseModes(); });
afterEach(() => { releases.splice(0).forEach(release => release()); clearNeteaseModes(); vi.useRealTimers(); });

it("retries a transient failure once for two active surfaces without duplicate probes", async () => {
  vi.useFakeTimers();
  read.mockRejectedValueOnce(new Error("busy")).mockResolvedValueOnce("repeat_one");
  const entry = neteaseModeState("epoch:retry");
  releases.push(entry.activate(), entry.activate());
  await entry.load();
  expect(read).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(500);
  expect(read).toHaveBeenCalledTimes(2);
  expect(get(entry)).toBe("repeat_one");
  await vi.advanceTimersByTimeAsync(60_000);
  expect(read).toHaveBeenCalledTimes(2);
});

it("bounds automatic retries for unknown modes and permits explicit recovery", async () => {
  vi.useFakeTimers();
  read.mockResolvedValue(null);
  const entry = neteaseModeState("epoch:unsupported");
  releases.push(entry.activate());
  await entry.load();
  await vi.advanceTimersByTimeAsync(60_000);
  expect(read).toHaveBeenCalledTimes(4);
  expect(get(entry.status)).toEqual({ loading: false, failed: true });
  read.mockResolvedValueOnce("shuffle");
  await entry.load(true);
  expect(get(entry)).toBe("shuffle");
});

it("stops automatic retry work after the last surface closes", async () => {
  vi.useFakeTimers();
  read.mockRejectedValue(new Error("starting"));
  const entry = neteaseModeState("epoch:closed");
  const release = entry.activate();
  await entry.load();
  release();
  await vi.advanceTimersByTimeAsync(60_000);
  expect(read).toHaveBeenCalledTimes(1);
});

it("an accepted mode fences a pending read and cannot leave a stuck request after clear", async () => {
  let finish!: (mode: "shuffle") => void;
  read.mockReturnValueOnce(new Promise(resolve => { finish = resolve; }));
  const entry = neteaseModeState("epoch:accept");
  const pending = entry.load();
  entry.accept("repeat_list");
  finish("shuffle");
  await pending;
  expect(get(entry)).toBe("repeat_list");
  entry.clear();
  read.mockResolvedValueOnce("repeat_one");
  await entry.load();
  expect(get(entry)).toBe("repeat_one");
});

it("coalesces mode probes and reuses confirmed state when a panel reopens", async () => {
  read.mockResolvedValue("shuffle");
  const entry = neteaseModeState("epoch:player");
  await Promise.all([entry.load(), entry.load()]);
  await neteaseModeState("epoch:player").load();
  expect(read).toHaveBeenCalledTimes(1);
  expect(get(entry)).toBe("shuffle");
});

it("a disconnection discards late mode reads and allows a fresh reconnect probe", async () => {
  let finish!: (value: "shuffle") => void;
  read.mockReturnValueOnce(new Promise(resolve => { finish = resolve; }));
  const entry = neteaseModeState("epoch:reconnect");
  const pending = entry.load();
  clearNeteaseModes();
  finish("shuffle");
  await pending;
  expect(get(entry)).toBeUndefined();
  read.mockResolvedValueOnce("repeat_one");
  await entry.load();
  expect(get(entry)).toBe("repeat_one");
});
