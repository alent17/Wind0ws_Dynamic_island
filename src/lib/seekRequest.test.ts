import { describe, expect, it, vi } from "vitest";
import { createSeekHandler, createSeekRequestGate, executeSeekRequest, formatSeekError, formatUserFacingError, seekSnapshotAllowsRollback } from "./seekRequest";

const confirmed = { trackId: "song-1", positionMs: 42_250, durationMs: 180_000 };

describe("seek request settlement shared by App and FloatingWindow", () => {
  it("applies the CDP-confirmed timeline after a successful request", async () => {
    let position = 90_000;
    let duration = 180_000;
    await executeSeekRequest(
      () => Promise.resolve(confirmed),
      () => true,
      () => true,
      (result) => { position = result?.positionMs ?? position; duration = result?.durationMs ?? duration; },
      () => { position = 90_000; },
    );
    expect(position).toBe(42_250);
    expect(duration).toBe(180_000);
  });

  it("rolls optimistic progress back when CDP rejects or cannot verify", async () => {
    let position = 90_000;
    let rejected: unknown;
    const rollback = vi.fn(() => { position = 35_000; });
    await expect(executeSeekRequest(
      () => Promise.reject(new Error("CDP read-back failed")),
      () => true,
      () => true,
      () => { position = confirmed.positionMs; },
      (error) => { rejected = error; rollback(); },
    )).rejects.toThrow("CDP read-back failed");
    expect(rollback).toHaveBeenCalledOnce();
    expect(position).toBe(35_000);
    expect(rejected).toBeInstanceOf(Error);
    expect(formatSeekError(rejected)).toBe("CDP read-back failed");
  });

  it.each([
    ["invoke string rejection", "播放器拒绝了跳转", "播放器拒绝了跳转"],
    ["invoke AppError rejection", { code: 4102, message: "CDP 未确认跳转" }, "[4102] CDP 未确认跳转"],
    ["invoke Error rejection", new Error("调用失败\n at secret stack"), "调用失败"],
    ["unknown payload", { stack: "secret", path: "C:\\private\\file" }, "播放进度跳转失败"],
  ])("formats %s into short inline feedback", (_label, rejected, expected) => {
    expect(formatSeekError(rejected)).toBe(expected);
  });

  it("uses an action-specific fallback and preserves native error detail", () => {
    expect(formatUserFacingError({ stack: "private" }, "网易云播放模式切换失败")).toBe("网易云播放模式切换失败");
    expect(formatUserFacingError(new Error("CDP 读回校验失败"), "网易云播放模式切换失败")).toBe("CDP 读回校验失败");
  });

  it("clears visible error on the next request and on confirmed success", async () => {
    let alert = "上次失败";
    const beginSeek = () => { alert = ""; };
    beginSeek();
    expect(alert).toBe("");
    await executeSeekRequest(
      () => Promise.resolve(confirmed), () => true, () => true,
      () => { alert = ""; }, () => {},
    );
    expect(alert).toBe("");
  });

  it("ignores late success and failure after the UI changes tracks", async () => {
    let currentTrack = "song-1";
    let position = 90_000;
    let rollbackCount = 0;
    let resolve!: (result: typeof confirmed) => void;
    const pending = new Promise<typeof confirmed>((done) => { resolve = done; });
    const request = executeSeekRequest(
      () => pending,
      () => currentTrack === "song-1",
      () => true,
      (result) => { position = result.positionMs; },
      () => { rollbackCount += 1; },
    );
    currentTrack = "song-2";
    resolve(confirmed);
    await request;
    expect(position).toBe(90_000);
    expect(rollbackCount).toBe(0);
  });

  it("does not let an older failed drag roll back the latest request", async () => {
    const gate = createSeekRequestGate();
    const oldRequestId = gate.begin();
    let position = 20_000;
    let rejectOld!: (error: Error) => void;
    const oldPending = new Promise<typeof confirmed>((_, reject) => { rejectOld = reject; });
    const oldRequest = executeSeekRequest(
      () => oldPending, () => true, () => gate.isCurrent(oldRequestId),
      () => { position = 10_000; }, () => { position = 0; },
    ).catch(() => {});
    gate.begin();
    position = 80_000;
    rejectOld(new Error("old seek failed"));
    await oldRequest;
    expect(position).toBe(80_000);
  });

  it("ignores success after a newer native snapshot invalidates the request", async () => {
    let nativeVersion = 1;
    let position = 15_000;
    let resolve!: (result: typeof confirmed) => void;
    const pending = new Promise<typeof confirmed>((done) => { resolve = done; });
    const request = executeSeekRequest(
      () => pending, () => true, () => nativeVersion === 1,
      (result) => { position = result.positionMs; }, () => { position = 30_000; },
    );
    nativeVersion = 2;
    position = 55_000;
    resolve(confirmed);
    await request;
    expect(position).toBe(55_000);
  });

  it("ignores late failure after the window is disposed", async () => {
    let mounted = true;
    let rollbackCount = 0;
    let reject!: (error: Error) => void;
    const pending = new Promise<typeof confirmed>((_, fail) => { reject = fail; });
    const request = executeSeekRequest(
      () => pending, () => true, () => mounted,
      () => {}, () => { rollbackCount++; },
    ).catch(() => {});
    mounted = false;
    reject(new Error("late"));
    await request;
    expect(rollbackCount).toBe(0);
  });

  it("keeps a seek valid across four newer native snapshots and exposes its late error", async () => {
    vi.useFakeTimers();
    try {
      const gate = createSeekRequestGate();
      const requestId = gate.begin();
      let nativeSnapshotRevision = 0;
      const requestSnapshotRevision = nativeSnapshotRevision;
      let latestSnapshotPosition = 0;
      let position = 80_000;
      let alert = "";
      let reject!: (error: unknown) => void;
      const pending = new Promise<typeof confirmed>((_, fail) => { reject = fail; });
      const request = executeSeekRequest(
        () => pending,
        () => true,
        () => gate.isCurrent(requestId),
        (result) => { position = result.positionMs; alert = ""; },
        (error) => {
          if (seekSnapshotAllowsRollback(requestSnapshotRevision, nativeSnapshotRevision)) {
            position = latestSnapshotPosition;
          }
          alert = formatSeekError(error);
        },
      ).catch(() => {});

      for (let second = 1; second <= 4; second++) {
        await vi.advanceTimersByTimeAsync(1_000);
        nativeSnapshotRevision++;
        latestSnapshotPosition = second * 1_000;
        position = latestSnapshotPosition + 250;
        expect(gate.isCurrent(requestId)).toBe(true);
      }
      reject({ code: 4204, message: "CDP read-back timed out" });
      await request;

      expect(alert).toBe("[4204] CDP read-back timed out");
      expect(position).toBe(4_250);
      expect(seekSnapshotAllowsRollback(requestSnapshotRevision, nativeSnapshotRevision)).toBe(false);
    } finally {
      vi.useRealTimers();
    }
  });

  it("applies a confirmed seek after newer native snapshots without letting them cancel the request", async () => {
    vi.useFakeTimers();
    try {
      const gate = createSeekRequestGate();
      const requestId = gate.begin();
      let nativeSnapshotRevision = 0;
      let position = 0;
      let resolve!: (result: typeof confirmed) => void;
      const pending = new Promise<typeof confirmed>((done) => { resolve = done; });
      const request = executeSeekRequest(
        () => pending, () => true, () => gate.isCurrent(requestId),
        (result) => { position = result.positionMs; }, () => {},
      );
      for (let second = 1; second <= 4; second++) {
        await vi.advanceTimersByTimeAsync(1_000);
        nativeSnapshotRevision++;
        expect(gate.isCurrent(requestId)).toBe(true);
      }
      resolve(confirmed);
      await request;
      expect(position).toBe(confirmed.positionMs);
      expect(seekSnapshotAllowsRollback(0, nativeSnapshotRevision)).toBe(false);
    } finally {
      vi.useRealTimers();
    }
  });

  it.each(["new seek", "track change", "dispose"])("invalidates the handler after %s, not after native snapshots", async (invalidation) => {
    const gate = createSeekRequestGate();
    let currentTrack = "song-1";
    let disposed = false;
    let alert = "";
    const rollback = vi.fn();
    const pendingRejectors: Array<(error: unknown) => void> = [];
    const handler = createSeekHandler({
      gate,
      createContext: (positionMs: number, requestId: number) => ({ positionMs, requestId, track: currentTrack }),
      isCurrentTrack: (context) => context.track === currentTrack,
      isDisposed: () => disposed,
      request: () => new Promise((_resolve, reject) => pendingRejectors.push(reject)),
      onBegin: () => { alert = ""; },
      onConfirmed: () => {},
      onFailed: () => { rollback(); },
      onError: (error) => { alert = formatSeekError(error); },
    });

    const stale = handler(10_000);
    let current: Promise<void> | undefined;
    if (invalidation === "new seek") current = handler(20_000);
    else if (invalidation === "track change") currentTrack = "song-2";
    else disposed = true;

    pendingRejectors[0](new Error("stale seek failed"));
    await stale;
    expect(alert).toBe("");
    expect(rollback).not.toHaveBeenCalled();

    if (current) {
      pendingRejectors[1](new Error("current seek failed"));
      await current;
      expect(alert).toBe("current seek failed");
      expect(rollback).toHaveBeenCalledOnce();
    }
  });
});
