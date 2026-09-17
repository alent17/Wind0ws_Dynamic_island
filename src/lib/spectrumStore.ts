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

// The compact and expanded surfaces can swap visibility during the same
// transition. Keep the native capture alive for a short grace period so a
// hand-off does not issue a stop/start pair on every expansion.
const CAPTURE_STOP_GRACE_MS = 350;

function queueCaptureCommand(command: "start_spectrum" | "stop_spectrum") {
  // Start and stop can cross during a quick collapse, pause, or settings
  // change. Keep their native calls in order so capture cannot outlive users.
  captureCommands = captureCommands.catch(() => undefined).then(() => invoke(command));
  return captureCommands;
}

async function connect(currentGeneration: number) {
  const dispose = await listen<number[]>("spectrum-data", ({ payload }) => {
    values.set(Float32Array.from({ length: NUM_BARS }, (_, index) => payload[index] ?? 0));
  });

  if (currentGeneration !== generation || consumers === 0) {
    dispose();
    return;
  }

  unlisten = dispose;
  try {
    await queueCaptureCommand("start_spectrum");
  } catch (error) {
    if (currentGeneration === generation) {
      unlisten?.();
      unlisten = undefined;
    }
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
