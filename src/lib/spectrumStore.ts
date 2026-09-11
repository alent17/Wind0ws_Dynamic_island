import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";

const NUM_BARS = 6;
const values = writable<Float32Array>(new Float32Array(NUM_BARS));

export const spectrumValues = { subscribe: values.subscribe };

let consumers = 0;
let generation = 0;
let unlisten: UnlistenFn | undefined;

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
    await invoke("start_spectrum");
  } catch (error) {
    if (currentGeneration === generation) {
      unlisten?.();
      unlisten = undefined;
    }
    console.warn("[Spectrum] 启动失败", error);
  }
}

export function retainSpectrum(): () => void {
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
    void invoke("stop_spectrum").catch(() => {});
  };
}
