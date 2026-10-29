import { afterEach, describe, expect, it, vi } from "vitest";
import { createMediaSyncSender, type MediaSyncPayload, type MediaSyncSnapshot } from "./mediaSyncSender";

function snapshot(
  title: string,
  albumArt: string,
  patch: Partial<MediaSyncSnapshot> = {},
): MediaSyncSnapshot {
  return {
    title,
    artist: "Artist",
    albumArt,
    isPlaying: true,
    positionMs: 100,
    durationMs: 1_000,
    lastUpdatedTimestamp: 10,
    source: "player",
    sourceDisplay: "Player",
    ...patch,
  };
}

const flush = async () => {
  await Promise.resolve();
  await Promise.resolve();
};

afterEach(() => vi.useRealTimers());

describe("media sync sender", () => {
  it("acks artwork replacement before omitting it, and sends explicit clear on change", async () => {
    const sent: MediaSyncPayload[] = [];
    const sender = createMediaSyncSender({ dispatch: (payload) => { sent.push(payload); } });

    sender.resetForConsumer("floating");
    sender.offer(snapshot("Track", "cover-1"));
    await flush();
    sender.offer(snapshot("Track", "cover-1", { positionMs: 200 }));
    await flush();
    sender.offer(snapshot("Track", ""));
    await flush();
    sender.offer(snapshot("Track", ""));
    await flush();

    expect(sent.map((payload) => Object.hasOwn(payload, "albumArt") ? payload.albumArt : "<omitted>")).toEqual([
      "cover-1",
      "<omitted>",
      "",
      "<omitted>",
    ]);
    sender.dispose();
  });

  it("keeps stamped artwork in the overlay without a duplicate top-level copy", async () => {
    const sent: MediaSyncPayload[] = [];
    const sender = createMediaSyncSender({ dispatch: payload => { sent.push(payload); } });
    const stamped = (art: string, generation = 1): MediaSyncSnapshot => snapshot("Track", art, {
      nativeEpoch: "epoch", nativeRevision: 1, nativeTrackGeneration: generation, artworkResolved: true,
      appOverlay: { nativeEpoch: "epoch", nativeRevision: 1, trackKey: "player|track|artist", title: "Track", artist: "Artist", albumArt: art, artworkResolved: true, positionMs: 100, lastUpdatedTimestamp: 10 },
    });
    sender.resetForConsumer("floating");
    sender.offer(stamped("cover")); await flush();
    sender.offer(stamped("cover")); await flush();
    sender.offer(stamped("cover", 2)); await flush();
    sender.offer(stamped("", 2)); await flush();
    expect(sent.every(payload => !Object.hasOwn(payload, "albumArt"))).toBe(true);
    expect(sent.map(payload => Object.hasOwn(payload.appOverlay!, "albumArt") ? payload.appOverlay!.albumArt : "<omitted>"))
      .toEqual(["cover", "cover", "cover", ""]);
    sender.dispose();
  });

  it("retries stamped artwork after an emission failure", async () => {
    const sent: MediaSyncPayload[] = [];
    const sender = createMediaSyncSender({
      retryBaseDelayMs: 60_000,
      dispatch: payload => { sent.push(payload); return sent.length === 1 ? Promise.reject(new Error("failed")) : undefined; },
    });
    const stamped = snapshot("Track", "cover", {
      nativeEpoch: "epoch", nativeRevision: 1, nativeTrackGeneration: 1,
      appOverlay: { nativeEpoch: "epoch", nativeRevision: 1, trackKey: "player|track|artist", title: "Track", artist: "Artist", albumArt: "cover", artworkResolved: false, positionMs: 100, lastUpdatedTimestamp: 10 },
    });
    sender.resetForConsumer("floating");
    sender.offer(stamped); await flush();
    sender.offer(stamped); await flush();
    expect(sent).toHaveLength(2);
    expect(sent.map(payload => payload.appOverlay?.albumArt)).toEqual(["cover", "cover"]);
    sender.dispose();
  });

  it("keeps artwork unacknowledged after failure and retries it with the next same-track sync", async () => {
    let rejectFirst!: (error: Error) => void;
    const first = new Promise<void>((_resolve, reject) => { rejectFirst = reject; });
    const sent: MediaSyncPayload[] = [];
    const sender = createMediaSyncSender({
      retryBaseDelayMs: 60_000,
      dispatch: (payload) => {
        sent.push(payload);
        return sent.length === 1 ? first : undefined;
      },
    });

    sender.resetForConsumer("floating");
    sender.offer(snapshot("Track", "cover-1"));
    await flush();
    rejectFirst(new Error("emit failed"));
    await flush();
    sender.offer(snapshot("Track", "cover-1", { positionMs: 250 }));
    await flush();

    expect(sent).toHaveLength(2);
    expect(sent[1].albumArt).toBe("cover-1");
    sender.dispose();
  });

  it("coalesces in-flight B into the latest C snapshot without letting B ack C", async () => {
    let resolveB!: () => void;
    const deferredB = new Promise<void>((resolve) => { resolveB = resolve; });
    const sent: MediaSyncPayload[] = [];
    const sender = createMediaSyncSender({
      dispatch: (payload) => {
        sent.push(payload);
        return payload.title === "B" ? deferredB : undefined;
      },
    });

    sender.resetForConsumer("floating");
    sender.offer(snapshot("A", "art-A"));
    await flush();
    sender.offer(snapshot("B", "art-B"));
    await flush();
    expect(sent).toHaveLength(2);
    sender.offer(snapshot("C", "art-C"));
    expect(sent).toHaveLength(2);
    resolveB();
    await flush();

    expect(sent).toHaveLength(3);
    expect(sent[2]).toMatchObject({ title: "C", albumArt: "art-C" });
    sender.dispose();
  });

  it("keeps one bounded retry and clears retry/pending work on dispose", async () => {
    vi.useFakeTimers();
    const dispatch = vi.fn(() => Promise.reject(new Error("emit failed")));
    const sender = createMediaSyncSender({ dispatch, retryBaseDelayMs: 100, retryMaxAttempts: 2 });

    sender.resetForConsumer("floating");
    sender.offer(snapshot("Track", "clear"));
    await flush();
    expect(dispatch).toHaveBeenCalledTimes(1);
    sender.dispose();
    await vi.advanceTimersByTimeAsync(1_000);
    expect(dispatch).toHaveBeenCalledTimes(1);
  });

  it("drops a newer pending snapshot and ignores in-flight completion after dispose", async () => {
    let resolve!: () => void;
    const deferred = new Promise<void>((done) => { resolve = done; });
    const dispatch = vi.fn(() => deferred);
    const sender = createMediaSyncSender({ dispatch });

    sender.resetForConsumer("floating");
    sender.offer(snapshot("A", "art-A"));
    await flush();
    sender.offer(snapshot("B", "art-B"));
    sender.dispose();
    resolve();
    await flush();

    expect(dispatch).toHaveBeenCalledTimes(1);
  });

  it("sends full artwork for each new consumer and treats duplicate ready tokens idempotently", async () => {
    const sent: MediaSyncPayload[] = [];
    const sender = createMediaSyncSender({ dispatch: (payload) => { sent.push(payload); } });
    const sameTrack = snapshot("Track", "cover");

    expect(sender.resetForConsumer("floating-1")).toBe(true);
    sender.offer(sameTrack);
    await flush();
    expect(sender.resetForConsumer("floating-1")).toBe(false);
    sender.offer(sameTrack);
    await flush();
    expect(sender.resetForConsumer("floating-2")).toBe(true);
    sender.offer(sameTrack);
    await flush();

    expect(sent.map((payload) => Object.hasOwn(payload, "albumArt"))).toEqual([true, false, true]);
    sender.dispose();
  });

  it("does not let an old consumer's in-flight success acknowledge a new consumer", async () => {
    let resolveOld!: () => void;
    const oldEmit = new Promise<void>((resolve) => { resolveOld = resolve; });
    const sent: MediaSyncPayload[] = [];
    const sender = createMediaSyncSender({
      dispatch: (payload) => {
        sent.push(payload);
        return sent.length === 1 ? oldEmit : undefined;
      },
    });

    sender.resetForConsumer("floating-old");
    sender.resetForConsumer("floating");
    sender.offer(snapshot("Track", "cover"));
    await flush();
    sender.resetForConsumer("floating-new");
    sender.offer(snapshot("Track", "cover", { positionMs: 200 }));
    resolveOld();
    await flush();

    expect(sent).toHaveLength(2);
    expect(sent[1]).toMatchObject({ title: "Track", positionMs: 200, albumArt: "cover" });
    sender.dispose();
  });

  it("retries at a finite backoff limit until a later sync replaces the pending full snapshot", async () => {
    vi.useFakeTimers();
    const sent: MediaSyncPayload[] = [];
    let shouldFail = true;
    const sender = createMediaSyncSender({
      retryBaseDelayMs: 10,
      retryMaxAttempts: 2,
      dispatch: (payload) => {
        sent.push(payload);
        if (shouldFail) return Promise.reject(new Error("emit failed"));
      },
    });

    sender.resetForConsumer("floating");
    sender.offer(snapshot("Track", "cover"));
    await flush();
    await vi.advanceTimersByTimeAsync(10);
    await flush();
    await vi.advanceTimersByTimeAsync(20);
    await flush();
    expect(sent).toHaveLength(3);
    shouldFail = false;
    sender.offer(snapshot("Track", "cover", { positionMs: 500 }));
    await flush();
    expect(sent).toHaveLength(4);
    expect(sent[3].albumArt).toBe("cover");
    sender.dispose();
  });

  it("does not dispatch while closed, drops queued state, and requires a full replay after re-ready", async () => {
    let resolveOld!: () => void;
    const oldEmit = new Promise<void>((resolve) => { resolveOld = resolve; });
    const sent: MediaSyncPayload[] = [];
    const sender = createMediaSyncSender({ dispatch: (payload) => {
      sent.push(payload);
      return sent.length === 1 ? oldEmit : undefined;
    } });

    sender.resetForConsumer("floating-old");
    sender.offer(snapshot("Track", "cover"));
    await flush();
    sender.offer(snapshot("Queued", "queued-cover"));
    expect(sender.pauseForConsumer("stale-instance")).toBe(false);
    expect(sender.pauseForConsumer("floating-old")).toBe(true);
    sender.offer(snapshot("Closed", "closed-cover"));
    expect(sent).toHaveLength(1);
    sender.resetForConsumer("floating-new");
    sender.offer(snapshot("Track", "cover", { positionMs: 500 }));
    resolveOld();
    await flush();

    expect(sent).toHaveLength(2);
    expect(sent[1]).toMatchObject({ title: "Track", albumArt: "cover", positionMs: 500 });
    expect(sender.pauseForConsumer("floating-old")).toBe(false);
    sender.dispose();
  });
});
