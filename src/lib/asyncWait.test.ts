import { afterEach, expect, it, vi } from "vitest";
import { createAsyncCleanup } from "./asyncCleanup";
import { waitForOwner } from "./asyncWait";
afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals(); });
it("disposal cancels and settles timeout and frame waits without continuation writes", async () => {
  vi.useFakeTimers(); let callback!: () => void;
  const cancel = vi.fn(); vi.stubGlobal("requestAnimationFrame", (next: () => void) => { callback = next; return 42; });
  vi.stubGlobal("cancelAnimationFrame", cancel);
  const owner = createAsyncCleanup(), write = vi.fn();
  const waits = [waitForOwner(owner, 600), waitForOwner(owner)].map(async ready => { if (await ready) write(); });
  owner.dispose(); callback(); vi.runAllTimers(); await Promise.all(waits);
  expect(write).not.toHaveBeenCalled(); expect(cancel).toHaveBeenCalledWith(42); expect(vi.getTimerCount()).toBe(0);
});
it("normal and already disposed waits settle and remove their cleanup", async () => {
  vi.useFakeTimers(); const owner = createAsyncCleanup(); const result = waitForOwner(owner, 10);
  vi.advanceTimersByTime(10); expect(await result).toBe(true); owner.dispose();
  expect(await waitForOwner(owner, 10)).toBe(false); expect(vi.getTimerCount()).toBe(0);
});
