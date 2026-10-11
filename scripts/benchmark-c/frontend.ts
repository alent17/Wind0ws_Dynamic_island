import { invoke } from '@tauri-apps/api/core';
let enabled: boolean | undefined;
export async function reportBenchmarkC(value: Record<string,unknown>) {
  if (enabled === undefined) {
    try { enabled = await invoke<boolean>('benchmark_c_enabled'); }
    catch { return; }
  }
  if (enabled) await invoke('benchmark_c_report',{value});
}
