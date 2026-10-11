import { afterEach, describe, expect, it, vi } from "vitest";
const bridge = vi.hoisted(() => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => bridge);
import { EventManager } from "./eventManager";
afterEach(() => { vi.useRealTimers(); bridge.listen.mockReset(); });
describe("EventManager teardown", () => {
  it("releases registration arriving after repeated destroy exactly once", async () => {
    let complete!: (value: () => void) => void;
    bridge.listen.mockImplementation(() => new Promise(resolve => complete = resolve));
    const manager = new EventManager(), handler = vi.fn(), unlisten = vi.fn();
    const pending = manager.on("late", handler, { debounce: 20 });
    manager.destroy(); manager.destroy();
    bridge.listen.mock.calls[0][1]({ payload: "late" });
    complete(unlisten);
    const cleanup = await pending; cleanup(); cleanup(); manager.destroy();
    expect(unlisten).toHaveBeenCalledTimes(1);
    expect(manager.getListenerCount()).toBe(0);
    expect(handler).not.toHaveBeenCalled();
  });
  it("cancels pending debounce and releases active listeners once", async () => {
    vi.useFakeTimers(); const unlisten = vi.fn(); bridge.listen.mockResolvedValue(unlisten);
    const manager = new EventManager(), handler = vi.fn();
    const cleanup = await manager.on("pending", handler, { debounce: 20 });
    bridge.listen.mock.calls[0][1]({ payload: 1 });
    manager.destroy(); cleanup(); vi.runAllTimers();
    expect(handler).not.toHaveBeenCalled(); expect(unlisten).toHaveBeenCalledTimes(1);
  });
});
