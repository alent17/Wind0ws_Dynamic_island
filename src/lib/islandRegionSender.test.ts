import { afterEach, describe, expect, it, vi } from "vitest";
import { createIslandRegionSender } from "./islandRegionSender";

type Region = { geometry: string; settled: boolean };
type Attempt = { value: Region; resolve: () => void; reject: (error: Error) => void };

function harness(options: { onError?: (error: unknown) => void; retryBaseDelayMs?: number; retryMaxDelayMs?: number } = {}) {
  const attempts: Attempt[] = [];
  let active = 0;
  let peakInFlight = 0;
  const sender = createIslandRegionSender<Region>({
    send: (value) => new Promise<void>((resolve, reject) => {
      active += 1;
      peakInFlight = Math.max(peakInFlight, active);
      attempts.push({
        value,
        resolve: () => { active -= 1; resolve(); },
        reject: (error) => { active -= 1; reject(error); },
      });
    }),
    ...options,
  });
  return { sender, attempts, getPeakInFlight: () => peakInFlight };
}

const offer = (sender: ReturnType<typeof createIslandRegionSender<Region>>, geometry: string, settled = false) => {
  const value = { geometry, settled };
  sender.offer(value, geometry);
};

async function flushMicrotasks() {
  await Promise.resolve();
  await Promise.resolve();
}

afterEach(() => vi.useRealTimers());

describe("island region sender", () => {
  it("keeps one request in flight and sends only the latest pending shape", async () => {
    const { sender, attempts, getPeakInFlight } = harness();
    offer(sender, "A");
    offer(sender, "B");
    offer(sender, "C");
    expect(attempts.map(({ value }) => value.geometry)).toEqual(["A"]);

    attempts[0].resolve();
    await flushMicrotasks();
    expect(attempts.map(({ value }) => value.geometry)).toEqual(["A", "C"]);
    expect(getPeakInFlight()).toBe(1);

    attempts[1].resolve();
    await flushMicrotasks();
    offer(sender, "C", true);
    expect(attempts).toHaveLength(2);
    sender.dispose();
  });

  it("preserves a new latest state even when it matches an older in-flight success", async () => {
    const { sender, attempts } = harness();
    offer(sender, "A");
    offer(sender, "B");
    attempts[0].resolve();
    await flushMicrotasks();
    expect(attempts.map(({ value }) => value.geometry)).toEqual(["A", "B"]);
    attempts[1].resolve();
    await flushMicrotasks();

    // A was acknowledged earlier, but B has since been sent. A is now a real
    // latest-state change and must be sent again instead of deduped by stale ack.
    offer(sender, "A", true);
    expect(attempts.map(({ value }) => value.geometry)).toEqual(["A", "B", "A"]);
    expect(attempts[2].value.settled).toBe(true);
    sender.dispose();
  });

  it("keeps a same-signature settled emission through failure and retries without acknowledging failure", async () => {
    vi.useFakeTimers();
    const { sender, attempts } = harness({ retryBaseDelayMs: 100, retryMaxDelayMs: 400 });
    offer(sender, "terminal", false);
    offer(sender, "terminal", true);
    attempts[0].reject(new Error("first send failed"));
    await flushMicrotasks();
    expect(attempts).toHaveLength(1);

    await vi.advanceTimersByTimeAsync(99);
    expect(attempts).toHaveLength(1);
    await vi.advanceTimersByTimeAsync(1);
    expect(attempts).toHaveLength(2);
    expect(attempts[1].value).toEqual({ geometry: "terminal", settled: true });
    attempts[1].resolve();
    await flushMicrotasks();
    offer(sender, "terminal", true);
    expect(attempts).toHaveLength(2);
    sender.dispose();
  });

  it("backs off, retries the newest pending request and reports one error per failure burst", async () => {
    vi.useFakeTimers();
    const onError = vi.fn();
    const { sender, attempts } = harness({ onError, retryBaseDelayMs: 100, retryMaxDelayMs: 200 });
    offer(sender, "A");
    attempts[0].reject(new Error("A failed"));
    await flushMicrotasks();
    offer(sender, "B");
    offer(sender, "C");

    await vi.advanceTimersByTimeAsync(100);
    expect(attempts.map(({ value }) => value.geometry)).toEqual(["A", "C"]);
    attempts[1].reject(new Error("C failed again"));
    await flushMicrotasks();
    expect(onError).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(199);
    expect(attempts).toHaveLength(2);
    await vi.advanceTimersByTimeAsync(1);
    expect(attempts).toHaveLength(3);
    attempts[2].resolve();
    await flushMicrotasks();
    offer(sender, "D");
    attempts[3].reject(new Error("new failure burst"));
    await flushMicrotasks();
    expect(onError).toHaveBeenCalledTimes(2);
    sender.dispose();
  });

  it("clears retry timers and ignores late in-flight completion after dispose", async () => {
    vi.useFakeTimers();
    const onError = vi.fn();
    const { sender, attempts } = harness({ onError, retryBaseDelayMs: 100 });
    offer(sender, "A");
    attempts[0].reject(new Error("retry pending"));
    await flushMicrotasks();
    expect(vi.getTimerCount()).toBe(1);
    sender.dispose();
    expect(vi.getTimerCount()).toBe(0);
    await vi.advanceTimersByTimeAsync(500);
    expect(attempts).toHaveLength(1);

    const active = harness({ onError });
    offer(active.sender, "B");
    offer(active.sender, "C");
    active.sender.dispose();
    active.attempts[0].resolve();
    await flushMicrotasks();
    expect(active.attempts).toHaveLength(1);
    expect(onError).toHaveBeenCalledTimes(1);
  });
});
