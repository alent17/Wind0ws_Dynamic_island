<script lang="ts">
  import { isTauri, invoke } from "@tauri-apps/api/core";
  import { Cpu, MemoryStick, CircuitBoard, ArrowDown, ArrowUp, Network } from "lucide-svelte";
  import { locale, translate } from "$lib/i18n";
  import { formatBytes, usagePercent, type SystemSnapshot } from "$lib/systemMonitor";
  let { active = true, snapshot } = $props<{active?: boolean; snapshot?: SystemSnapshot | null}>();
  let live = $state<SystemSnapshot | null>(null);
  let failed = $state(false);
  const data = $derived(snapshot === undefined ? live : snapshot);
  const t = (key: Parameters<typeof translate>[0]) => translate(key,{},$locale);
  const metrics = $derived([
    {label:"CPU",icon:Cpu,value:usagePercent(data?.cpuPercent)},
    {label:"GPU",icon:CircuitBoard,value:usagePercent(data?.gpuPercent)},
    {label:t("memory"),icon:MemoryStick,value:usagePercent(data?.memoryPercent)},
  ]);
  $effect(() => {
    if (!active || snapshot !== undefined || !isTauri()) return;
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    const refresh = async () => {
      try {
        const value = await invoke<SystemSnapshot>("get_system_snapshot");
        if (!disposed) {live = value;failed = false;}
      } catch {if (!disposed) failed = true;}
      if (!disposed) timer = setTimeout(refresh,1000);
    };
    void refresh();
    return () => {disposed = true;clearTimeout(timer);};
  });
</script>

<div class="system-monitor" aria-label={t("systemMonitor")}>
  <div class="system-grid">
    {#each metrics as metric, index}
      <div class="system-metric" class:unavailable={metric.value === null}>
        <div class="metric-label"><metric.icon size={17}/><span>{metric.label}</span></div>
        <div class="metric-number">{failed ? "--" : metric.value ?? "--"}<small>{!failed && metric.value !== null ? "%" : ""}</small></div>
        {#if index === 2}<span class="metric-detail">{data && !failed ? `${formatBytes(data.memoryUsedBytes)} / ${formatBytes(data.memoryTotalBytes)}` : t("systemUnavailable")}</span>
        {:else if metric.value === null || failed}<span class="metric-detail">{t("systemUnavailable")}</span>
        {:else}<div class="usage-track" aria-hidden="true"><span style:width={`${metric.value}%`}></span></div>{/if}
      </div>
    {/each}
    <div class="system-metric network-metric">
      <div class="metric-label"><Network size={17}/><span>{t("network")}</span></div>
      <div class="network-rate"><ArrowDown size={16}/><strong>{data && !failed ? formatBytes(data.downloadBytesPerSecond,true) : "--"}</strong></div>
      <div class="network-rate upload"><ArrowUp size={16}/><strong>{data && !failed ? formatBytes(data.uploadBytesPerSecond,true) : "--"}</strong></div>
    </div>
  </div>
  <span class="system-status" role="status"><i class:online={!!data && !failed}></i>{failed ? t("systemReadFailed") : data ? t("systemLive") : t("systemWaiting")}</span>
</div>

<style>
  .system-monitor{container-type:inline-size;display:flex;flex-direction:column;gap:10px;width:100%;height:100%;min-height:0;color:#f5f5f7;font-family:var(--app-font,system-ui);user-select:none}
  .system-grid{flex:1;min-height:0;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));grid-template-rows:repeat(2,minmax(0,1fr));gap:12px 20px}
  .system-metric{display:flex;flex-direction:column;justify-content:space-between;gap:5px;min-width:0;min-height:0}.metric-label{display:flex;align-items:center;gap:7px;font-size:14px;color:#ffffff8c;white-space:nowrap}.metric-label :global(svg){flex:none}
  .metric-number{font-size:clamp(28px,9cqw,40px);font-weight:650;line-height:1;font-variant-numeric:tabular-nums;letter-spacing:-.04em}.metric-number small{font-size:.4em;font-weight:500;color:#ffffff80;margin-left:3px;letter-spacing:0}
  .metric-detail{font-size:12px;line-height:1.2;color:#ffffff73;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.usage-track{height:3px;background:#ffffff1a;border-radius:4px;overflow:hidden;margin:4px 0}.usage-track span{display:block;height:100%;border-radius:inherit;background:#dedee3}
  .network-rate{display:flex;align-items:center;gap:6px;min-width:0;color:#f5f5f7}.network-rate strong{font-size:clamp(14px,4.6cqw,20px);font-weight:600;line-height:1.15;white-space:nowrap;font-variant-numeric:tabular-nums}.network-rate :global(svg){flex:none;color:#9dbbdb}.network-rate.upload{color:#ffffff8c}.network-rate.upload :global(svg){color:#ffffff73}
  .system-status{display:flex;align-items:center;gap:6px;flex:none;font-size:11px;line-height:14px;color:#ffffff66}.system-status i{width:4px;height:4px;border-radius:50%;background:#666}.system-status i.online{background:#8fb59c}
</style>
