import { afterEach, describe, expect, it, vi } from "vitest";

const native = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn(), dispose: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: native.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: native.listen }));

afterEach(() => { vi.useRealTimers(); vi.resetModules(); vi.resetAllMocks(); });

describe("spectrum capture lifecycle", () => {
  it("retries a lost stream and releases its watchdog with the last consumer", async () => {
    vi.useFakeTimers();
    native.listen.mockResolvedValue(native.dispose);
    native.invoke.mockResolvedValue(undefined);
    const { retainSpectrum } = await import("./spectrumStore");
    const release = retainSpectrum();
    await vi.advanceTimersByTimeAsync(3100);
    expect(native.invoke.mock.calls.filter(([command]) => command === "start_spectrum")).toHaveLength(2);
    release();
    await vi.advanceTimersByTimeAsync(5000);
    // The watchdog replaces the listener once, then release disposes the replacement.
    expect(native.dispose).toHaveBeenCalledTimes(2);
    expect(native.invoke).toHaveBeenLastCalledWith("stop_spectrum");
    expect(vi.getTimerCount()).toBe(0);
  });

  it("does not stop native capture when compact hands off to expanded", async () => {
    vi.useFakeTimers();
    native.listen.mockResolvedValue(native.dispose);
    native.invoke.mockResolvedValue(undefined);
    const { retainSpectrum } = await import("./spectrumStore");
    const releaseCompact = retainSpectrum();
    await vi.advanceTimersByTimeAsync(0);
    releaseCompact();
    const releaseExpanded = retainSpectrum();
    await vi.advanceTimersByTimeAsync(400);
    expect(native.invoke.mock.calls.some(([command]) => command === "stop_spectrum")).toBe(false);
    releaseExpanded();
    await vi.advanceTimersByTimeAsync(400);
    expect(native.invoke).toHaveBeenLastCalledWith("stop_spectrum");
  });

  it("restarts active capture after the output device changes", async () => {
    vi.useFakeTimers();
    native.listen.mockResolvedValue(native.dispose);
    native.invoke.mockResolvedValue(undefined);
    const { retainSpectrum, refreshSpectrumDevice } = await import("./spectrumStore");
    const release = retainSpectrum();
    await vi.advanceTimersByTimeAsync(0);
    await refreshSpectrumDevice();
    expect(native.invoke).toHaveBeenLastCalledWith("restart_spectrum");
    release();
    await vi.advanceTimersByTimeAsync(400);
  });
});
