import { describe, expect, it, vi } from "vitest";
import { LatestValueWriteQueue } from "./latestValueWriteQueue";

function deferred<T = void>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((done, fail) => { resolve = done; reject = fail; });
  return { promise, resolve, reject };
}

describe("LatestValueWriteQueue", () => {
  it("deduplicates the current value while a write is active", async () => {
    const first = deferred<void>();
    const write = vi.fn(() => first.promise);
    const queue = new LatestValueWriteQueue(write, vi.fn());

    const request = queue.submit(40, "speaker:40");
    await Promise.resolve();
    const duplicate = queue.submit(40, "speaker:40");
    expect(write).toHaveBeenCalledTimes(1);
    first.resolve();
    await Promise.all([request, duplicate]);
  });

  it("keeps only the latest pending value", async () => {
    const first = deferred<void>();
    const calls: number[] = [];
    const queue = new LatestValueWriteQueue<number>(async value => {
      calls.push(value);
      if (calls.length === 1) await first.promise;
    }, vi.fn());

    const request = queue.submit(10, "speaker:10");
    await Promise.resolve();
    queue.submit(20, "speaker:20");
    queue.submit(30, "speaker:30");
    first.resolve();
    await request;
    expect(calls).toEqual([10, 30]);
  });

  it("does not let an older in-flight A suppress a newer A after pending B", async () => {
    const first = deferred<void>();
    const calls: number[] = [];
    const queue = new LatestValueWriteQueue<number>(async value => {
      calls.push(value);
      if (calls.length === 1) await first.promise;
    }, vi.fn());

    const request = queue.submit(10, "speaker:10");
    await Promise.resolve();
    queue.submit(20, "speaker:20");
    queue.submit(10, "speaker:10");
    first.resolve();
    await request;
    expect(calls).toEqual([10, 10]);
  });

  it("cancels only the pending write when the owner is destroyed", async () => {
    const first = deferred<void>();
    const calls: number[] = [];
    const queue = new LatestValueWriteQueue<number>(async value => {
      calls.push(value);
      if (calls.length === 1) await first.promise;
    }, vi.fn());

    const request = queue.submit(10, "speaker:10");
    await Promise.resolve();
    queue.submit(20, "speaker:20");
    queue.cancelPending();
    first.resolve();
    await request;
    expect(calls).toEqual([10]);
  });

  it("allows retry after failure and re-sends after external/device invalidation", async () => {
    const failed = deferred<void>();
    const errors = vi.fn();
    const write = vi.fn()
      .mockImplementationOnce(() => failed.promise)
      .mockResolvedValue(undefined);
    const queue = new LatestValueWriteQueue(write, errors);

    const request = queue.submit(55, "speaker-a:55");
    await Promise.resolve();
    failed.reject(new Error("native write failed"));
    await request;
    expect(errors).toHaveBeenCalledTimes(1);
    await queue.submit(55, "speaker-a:55");
    expect(write).toHaveBeenCalledTimes(2);

    queue.invalidate();
    await queue.submit(55, "speaker-a:55");
    expect(write).toHaveBeenCalledTimes(3);
    queue.invalidate();
    await queue.submit(55, "speaker-b:55");
    await queue.submit(55, "speaker-b:55");
    expect(write).toHaveBeenCalledTimes(4);
  });

  it("dispatches the newest pending value after an older in-flight write fails", async () => {
    const failed = deferred<void>();
    const calls: number[] = [];
    const errorState: boolean[] = [];
    const queue = new LatestValueWriteQueue<number>(async value => {
      calls.push(value);
      if (calls.length === 1) await failed.promise;
    }, (_error, pendingWillRetry) => errorState.push(pendingWillRetry));

    const request = queue.submit(40, "speaker:40");
    await Promise.resolve();
    queue.submit(55, "speaker:55");
    queue.submit(70, "speaker:70");
    failed.reject(new Error("first native write failed"));
    await request;

    expect(calls).toEqual([40, 70]);
    expect(errorState).toEqual([true]);
    await queue.submit(70, "speaker:70");
    expect(calls).toEqual([40, 70]);
  });
});
