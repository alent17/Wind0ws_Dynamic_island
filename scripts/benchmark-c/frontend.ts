import { invoke } from '@tauri-apps/api/core';
const enabled=invoke<boolean>('benchmark_c_enabled').catch(()=>false);
export async function reportBenchmarkC(value: Record<string,unknown>) {
  if(await enabled) await invoke('benchmark_c_report',{value});
}
