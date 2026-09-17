<script lang="ts">
  import { Pause, Play, Plus, X } from "lucide-svelte";
  import { clockZoneLabel, formatCountdown, type CountdownStatus } from "$lib/countdown";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  let {
    status = "idle",
    remainingMs = 0,
    clockText = "00:00",
    timeZone = "system",
    onStart,
    onPause,
    onResume,
    onAdjust,
    onReset,
  } = $props<{
    status?: CountdownStatus;
    remainingMs?: number;
    clockText?: string;
    timeZone?: string;
    onStart?: (durationMs: number) => void;
    onPause?: () => void;
    onResume?: () => void;
    onAdjust?: (deltaMs: number) => void;
    onReset?: () => void;
  }>();

  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);
  const active = $derived((status === "running" || status === "paused") && remainingMs > 0);

  function stop(event: MouseEvent) {
    event.stopPropagation();
  }
</script>

<div class="timer-panel" data-stop-toggle>
  {#if active}
    <div class="timer-readout">
      <strong>{formatCountdown(remainingMs)}</strong>
      <small>{t("remainingTime")}</small>
    </div>
    <button class="timer-close" type="button" aria-label={t("cancelTimer")} onclick={(event) => { stop(event); onReset?.(); }}>
      <X size={14} />
    </button>
    <div class="timer-actions">
      <button type="button" aria-label={t("addMinute")} onclick={(event) => { stop(event); onAdjust?.(60_000); }}><Plus size={13} />1m</button>
      <button type="button" aria-label={t("addFiveMinutes")} onclick={(event) => { stop(event); onAdjust?.(300_000); }}><Plus size={13} />5m</button>
      <button type="button" class:running={status === "running"} aria-label={status === "running" ? t("pauseTimer") : t("resumeTimer")} onclick={(event) => { stop(event); status === "running" ? onPause?.() : onResume?.(); }}>
        {#if status === "running"}<Pause size={13} />{:else}<Play size={13} />{/if}
      </button>
    </div>
  {:else}
    <div class="clock-readout">
      <strong>{clockText}</strong>
      <small>{clockZoneLabel(timeZone, $locale)}</small>
    </div>
    <div class="timer-presets">
      {#each [5, 10, 25, 60] as minutes}
        <button type="button" aria-label={t("startTimer")} onclick={(event) => { stop(event); onStart?.(minutes * 60_000); }}>{minutes}m</button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .timer-panel{position:relative;width:100%;height:38px;display:flex;align-items:center;color:#fff;font-variant-numeric:tabular-nums}.timer-readout,.clock-readout{min-width:0;display:flex;flex-direction:column;gap:1px}.timer-readout strong,.clock-readout strong{font-size:15px;line-height:1;font-weight:700;letter-spacing:.01em}.timer-readout small,.clock-readout small{max-width:70px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:rgba(255,255,255,.48);font-size:8px;line-height:1.1}.timer-close{display:grid;place-items:center;width:22px;height:22px;margin-left:auto;color:rgba(255,255,255,.55);border:0;border-radius:8px;background:transparent;cursor:pointer}.timer-close:hover{color:#fff;background:rgba(255,255,255,.1)}.timer-actions{display:flex;align-items:center;gap:4px;margin-left:auto}.timer-actions button,.timer-presets button{display:inline-flex;align-items:center;justify-content:center;gap:2px;height:22px;padding:0 5px;border:1px solid rgba(255,255,255,.12);border-radius:7px;color:rgba(255,255,255,.76);background:rgba(255,255,255,.08);font-size:9px;line-height:1;cursor:pointer}.timer-actions button:hover,.timer-presets button:hover{color:#fff;background:rgba(255,255,255,.17)}.timer-actions button.running{color:#fff;background:rgba(255,255,255,.16)}.timer-presets{display:flex;gap:4px;margin-left:auto}.timer-presets button{min-width:25px}.clock-readout strong{font-size:16px}.clock-readout small{max-width:76px}
  @media (prefers-reduced-motion:reduce){.timer-actions button,.timer-presets button,.timer-close{transition:none}}
</style>
