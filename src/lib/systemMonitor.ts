export interface SystemSnapshot {
  cpuPercent: number;
  memoryPercent: number;
  memoryUsedBytes: number;
  memoryTotalBytes: number;
  gpuPercent: number | null;
  uploadBytesPerSecond: number;
  downloadBytesPerSecond: number;
}
export function formatBytes(value: number, speed = false): string {
  if (!Number.isFinite(value) || value < 0) return "--";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(4, Math.floor(Math.log(Math.max(1,value)) / Math.log(1024)));
  const amount = value / 1024 ** index;
  return `${amount.toFixed(index === 0 ? 0 : 1)} ${units[index]}${speed ? "/s" : ""}`;
}
export function usagePercent(value: number | null | undefined): number | null {
  return value == null || !Number.isFinite(value) ? null : Math.round(Math.min(100, Math.max(0,value)));
}
