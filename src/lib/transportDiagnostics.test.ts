import { afterEach, describe, expect, it, vi } from "vitest";
import {
  disableTransportDiagnostics,
  enableTransportDiagnostics,
  recordTransportCounter,
  resetTransportDiagnostics,
  runTimedTransportCommand,
  runTransportEvent,
  snapshotTransportDiagnostics,
  transportDiagnosticsSchema,
} from "./transportDiagnostics";

afterEach(() => {
  disableTransportDiagnostics();
  resetTransportDiagnostics();
  vi.restoreAllMocks();
});

describe("transport diagnostics", () => {
  it("is default-off, allowlisted, and does not read the clock or schedule timers while disabled", () => {
    const clock = vi.spyOn(performance, "now");
    const timer = vi.spyOn(globalThis, "setTimeout");
    let calls = 0;

    expect(snapshotTransportDiagnostics()).toMatchObject({ scope: "this-webview", enabled: false });
    expect(runTimedTransportCommand("volume", () => { calls++; return 7; })).toBe(7);
    expect(runTransportEvent("mediaSync", () => { calls++; return 9; })).toBe(9);
    recordTransportCounter("mediaSync", "received");

    const snapshot = snapshotTransportDiagnostics();
    expect(snapshot.enabled).toBe(false);
    expect(Object.keys(snapshot.routes)).toEqual(["mediaSync", "timerState", "region", "volume"]);
    expect(snapshot.routes.volume.dispatchAttempts).toBeUndefined();
    expect(snapshot.routes.mediaSync.received).toBeUndefined();
    expect(calls).toBe(2);
    expect(clock).not.toHaveBeenCalled();
    expect(timer).not.toHaveBeenCalled();
    expect(transportDiagnosticsSchema.routes).toEqual(["mediaSync", "timerState", "region", "volume"]);
  });

  it("records allowlisted event/command outcomes, duration aggregates, and reset", async () => {
    vi.spyOn(performance, "now").mockReturnValueOnce(10).mockReturnValueOnce(14);
    enableTransportDiagnostics();
    await runTimedTransportCommand("region", () => Promise.resolve("ok"));
    await runTransportEvent("timerState", () => Promise.resolve("ok"), true);
    recordTransportCounter("timerState", "received");

    expect(snapshotTransportDiagnostics()).toMatchObject({
      enabled: true,
      scope: "this-webview",
      routes: {
        region: { dispatchAttempts: 1, dispatchSuccess: 1, durationCount: 1, durationSumMs: 4, durationMaxMs: 4 },
        timerState: { dispatchAttempts: 1, dispatchSuccess: 1, requestReplies: 1, received: 1 },
      },
    });
    resetTransportDiagnostics();
    expect(snapshotTransportDiagnostics().routes.region.dispatchAttempts).toBeUndefined();
    expect(snapshotTransportDiagnostics().routes.timerState.received).toBeUndefined();
  });

  it("does not let a pending command completion contaminate a later enable generation", async () => {
    let resolve!: (value: string) => void;
    enableTransportDiagnostics();
    const pending = runTimedTransportCommand("volume", () => new Promise<string>(done => { resolve = done; }));
    disableTransportDiagnostics();
    enableTransportDiagnostics();
    resolve("done");
    await pending;

    const volume = snapshotTransportDiagnostics().routes.volume;
    expect(volume.dispatchAttempts).toBe(1);
    expect(volume.dispatchSuccess).toBeUndefined();
    expect(volume.durationCount).toBe(0);
  });
});
