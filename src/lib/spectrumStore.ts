import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";

const NUM_BARS = 6;
const values = writable<Float32Array>(new Float32Array(NUM_BARS));

export const spectrumValues = { subscribe: values.subscribe };

let consumers = 0;
let generation = 0;
let unlisten: UnlistenFn | undefined;
let captureCommands: Promise<unknown> = Promise.resolve();
let stopTimer: ReturnType<typeof setTimeout> | undefined;
let watchdog: ReturnType<typeof setInterval> | undefined;
let lastFrameAt = 0;

// The compact and expanded surfaces can swap visibility during the same
// transition. Keep the native capture alive for a short grace period so a
// hand-off does not issue a stop/start pair on every expansion.
const CAPTURE_STOP_GRACE_MS = 350;
const LISTENER_RETRY_MS = 1_000;

function queueCaptureCommand(command: "start_spectrum" | "stop_spectrum" | "restart_spectrum") {
  // Start and stop can cross during a quick collapse, pause, or settings
  // change. Keep their native calls in order so capture cannot outlive users.
  captureCommands = captureCommands.catch(() => undefined).then(() => invoke(command));
  return captureCommands;
}

export async function refreshSpectrumDevice(): Promise<void> {
  if (consumers > 0) await queueCaptureCommand("restart_spectrum");
}

function isCurrent(currentGeneration: number) {
  return currentGeneration === generation && consumers > 0;
}

async function attachListener(currentGeneration: number): Promise<boolean> {
  try {
    const dispose = await listen<number[]>("spectrum-data", ({ payload }) => {
      if (!isCurrent(currentGeneration)) return;
      lastFrameAt = Date.now();
      values.set(Float32Array.from({ length: NUM_BARS }, (_, index) => payload[index] ?? 0));
    });

    if (!isCurrent(currentGeneration)) {
      dispose();
      return false;
    }

    const previous = unlisten;
    unlisten = dispose;
    previous?.();
    lastFrameAt = Date.now();
    return true;
  } catch (error) {
    console.warn("[Spectrum] 监听连接失败", error);
    return false;
  }
}

async function connect(currentGeneration: number) {
  while (isCurrent(currentGeneration)) {
    if (await attachListener(currentGeneration)) break;
    await new Promise<void>((resolve) => {
      setTimeout(() => resolve(), LISTENER_RETRY_MS);
    });
  }

  if (!isCurrent(currentGeneration)) return;
  lastFrameAt = Date.now();
  if (watchdog) clearInterval(watchdog);
  watchdog = setInterval(() => {
    if (!isCurrent(currentGeneration)) return;
    if (Date.now() - lastFrameAt > 2500) {
      values.set(new Float32Array(NUM_BARS));
      lastFrameAt = Date.now();
      // Recover both sides: a native device change can end capture, while a
      // WebView listener can also become detached independently.
      void (async () => {
        if (!(await attachListener(currentGeneration))) return;
        await queueCaptureCommand("start_spectrum").catch(() => {});
      })();
    }
  }, 1000);
  try {
    await queueCaptureCommand("start_spectrum");
  } catch (error) {
    // Keep the listener: the watchdog retries after a device becomes available.
    console.warn("[Spectrum] 启动失败", error);
  }
}

export function retainSpectrum(): () => void {
  if (stopTimer) {
    clearTimeout(stopTimer);
    stopTimer = undefined;
  }
  consumers += 1;
  if (consumers === 1) {
    const currentGeneration = ++generation;
    void connect(currentGeneration).catch((error) => {
      console.warn("[Spectrum] 连接失败", error);
    });
  }

  let released = false;
  return () => {
    if (released) return;
    released = true;
    consumers = Math.max(0, consumers - 1);
    if (consumers !== 0) return;

    generation += 1;
    if (watchdog) clearInterval(watchdog);
    watchdog = undefined;
    unlisten?.();
    unlisten = undefined;
    values.set(new Float32Array(NUM_BARS));
    const releaseGeneration = generation;
    stopTimer = setTimeout(() => {
      stopTimer = undefined;
      if (consumers === 0 && releaseGeneration === generation) {
        void queueCaptureCommand("stop_spectrum").catch(() => {});
      }
    }, CAPTURE_STOP_GRACE_MS);
  };
}
