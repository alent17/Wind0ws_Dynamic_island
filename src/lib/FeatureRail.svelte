<script lang="ts">
  import { GalleryHorizontalEnd, Timer, Volume2, VolumeX } from "lucide-svelte";
  import TimerPanel from "$lib/TimerPanel.svelte";
  import type { CountdownStatus } from "$lib/countdown";
  import type { IslandTool } from "$lib/featureRail";
  import { locale, translate, type TranslationKey } from "$lib/i18n";


  let {
    visible = false,
    activeTool = null,
    volume = 50,
    muted = false,
    timerStatus = "idle",
    timerRemainingMs = 0,
    clockText = "00:00",
    timeZone = "system",
    onTool,
    onFloating,
    onAudioOpen,
    onVolume,
    onTimerStart,
    onTimerPause,
    onTimerResume,
    onTimerAdjust,
    onTimerReset,
  } = $props<{
    visible?: boolean;
    activeTool?: IslandTool | null;
    volume?: number;
    muted?: boolean;
    timerStatus?: CountdownStatus;
    timerRemainingMs?: number;
    clockText?: string;
    timeZone?: string;
    onTool?: (tool: IslandTool | null) => void;
    onFloating?: () => void;
    onAudioOpen?: () => void;
    onVolume?: (volumePercent: number) => void | Promise<void>;
    onTimerStart?: (durationMs: number) => void;
    onTimerPause?: () => void;
    onTimerResume?: () => void;
    onTimerAdjust?: (deltaMs: number) => void;
    onTimerReset?: () => void;
  }>();

  const t = (key: TranslationKey) => translate(key, {}, $locale);

  function toggleTool(tool: IslandTool) {
    const next = activeTool === tool ? null : tool;
    onTool?.(next);
    if (tool === "volume" && next === "volume") onAudioOpen?.();
  }
</script>

<div class="feature-rail" class:visible aria-hidden={!visible}>
  <button class="tool-circle" type="button" aria-label={t("toggleFloating")} onclick={(event) => { event.stopPropagation(); onFloating?.(); }}>
    <GalleryHorizontalEnd size={18} />
  </button>

  <div class="tool-item" class:expanded={activeTool === "volume"}>
    <button class="tool-circle" type="button" aria-label={t("volume")} aria-expanded={activeTool === "volume"} onclick={(event) => { event.stopPropagation(); toggleTool("volume"); }}>
      {#if muted || volume === 0}<VolumeX size={18} />{:else}<Volume2 size={18} />{/if}
    </button>
    <div class="tool-content volume-content" data-stop-toggle>
      <input aria-label={t("volume")} type="range" min="0" max="100" value={volume} oninput={(event) => onVolume?.(Number(event.currentTarget.value))} />
      <span>{muted ? t("muted") : `${volume}%`}</span>
    </div>
  </div>

  <div class="tool-item timer-item" class:expanded={activeTool === "timer"}>
    <button class="tool-circle" type="button" aria-label={t("timer")} aria-expanded={activeTool === "timer"} onclick={(event) => { event.stopPropagation(); toggleTool("timer"); }}>
      <Timer size={18} />
    </button>
    <div class="tool-content timer-content" data-stop-toggle>
      <TimerPanel
        status={timerStatus}
        remainingMs={timerRemainingMs}
        {clockText}
        {timeZone}
        onStart={onTimerStart}
        onPause={onTimerPause}
        onResume={onTimerResume}
        onAdjust={onTimerAdjust}
        onReset={onTimerReset}
      />
    </div>
  </div>
</div>

<style>
  .feature-rail{position:relative;z-index:4;display:flex;flex-direction:column;gap:8px;width:38px;opacity:0;transform:translateX(-10px) scale(.92);transform-origin:left center;pointer-events:none;transition:opacity 150ms ease,transform 260ms cubic-bezier(.22,1,.36,1)}.feature-rail.visible{opacity:1;transform:translateX(0) scale(1);pointer-events:auto}.tool-circle{position:relative;z-index:2;display:grid;place-items:center;flex:none;width:38px;height:38px;padding:0;color:rgba(255,255,255,.8);border:1px solid rgba(255,255,255,.1);border-radius:19px;background:#111;box-shadow:0 8px 18px rgba(0,0,0,.22);cursor:pointer;transition:color 140ms ease,background 140ms ease,transform 140ms cubic-bezier(.23,1,.32,1)}.tool-circle:hover{color:#fff;background:#1b1b1b;transform:scale(1.05)}.tool-circle:active{transform:scale(.94)}.tool-item{position:relative;width:38px;height:38px;flex:none;overflow:hidden;border:1px solid rgba(255,255,255,.1);border-radius:19px;background:#111;box-shadow:0 8px 18px rgba(0,0,0,.22);transform-origin:left center;transition:width 320ms cubic-bezier(.22,1,.36,1),border-radius 260ms cubic-bezier(.22,1,.36,1),background 160ms ease}.tool-item.expanded{width:168px;border-radius:19px;background:#151515}.tool-item .tool-circle{border:0;box-shadow:none}.tool-content{position:absolute;left:44px;right:10px;top:50%;display:flex;align-items:center;min-width:0;opacity:0;transform:translateY(-50%) translateX(-8px);pointer-events:none;transition:opacity 120ms 100ms ease,transform 180ms 80ms ease}.tool-item.expanded .tool-content{opacity:1;transform:translateY(-50%) translateX(0);pointer-events:auto}.volume-content{gap:7px}.volume-content input{width:82px;height:4px;padding:0;border:0;border-radius:4px;accent-color:#fff;background:rgba(255,255,255,.18);cursor:pointer}.volume-content span{min-width:27px;color:rgba(255,255,255,.82);font-size:10px;font-variant-numeric:tabular-nums;text-align:right}.timer-content{right:8px}.timer-content :global(.timer-panel){width:100%}button:focus-visible,input:focus-visible{outline:2px solid #fff;outline-offset:2px}@media (prefers-reduced-motion:reduce){.feature-rail,.tool-item,.tool-content,.tool-circle{transition-duration:120ms!important;transform:none}.feature-rail{opacity:0}.feature-rail.visible{opacity:1}.tool-item.expanded .tool-content{transform:translateY(-50%)}}
</style>
