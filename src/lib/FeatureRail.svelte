<script lang="ts">
  import { GalleryHorizontalEnd, Timer, Volume2, VolumeX } from "lucide-svelte";
  import type { IslandTool } from "$lib/featureRail";
  import { locale, translate, type TranslationKey } from "$lib/i18n";


  let {
    visible = false,
    activeTool = null,
    enabledTools = ["floating", "volume", "timer"],
    railBackground = "#000",
    volume = 50,
    muted = false,
    onTool,
    onFloating,
    onTimerOpen,
    onAudioOpen,
    onVolume,
  } = $props<{
    visible?: boolean;
    activeTool?: IslandTool | null;
    enabledTools?: IslandTool[];
    railBackground?: string;
    volume?: number;
    muted?: boolean;
    onTool?: (tool: IslandTool | null) => void;
    onFloating?: () => void;
    onTimerOpen?: () => void;
    onAudioOpen?: () => void;
    onVolume?: (volumePercent: number) => void | Promise<void>;
  }>();

  const t = (key: TranslationKey) => translate(key, {}, $locale);

  function toggleTool(tool: IslandTool) {
    const next = activeTool === tool ? null : tool;
    onTool?.(next);
    if (tool === "volume" && next === "volume") onAudioOpen?.();
  }

  function openTimer() {
    onTool?.(null);
    onTimerOpen?.();
  }
</script>

<div class="feature-rail" class:visible aria-hidden={!visible} inert={!visible} style={`--rail-background:${railBackground}`}>
  {#if enabledTools.includes("floating")}
    <button class="tool-circle" type="button" aria-label={t("toggleFloating")} onclick={(event) => { event.stopPropagation(); onFloating?.(); }}>
      <GalleryHorizontalEnd size={16} />
    </button>
  {/if}

  {#if enabledTools.includes("volume")}
    <div class="tool-item" class:expanded={activeTool === "volume"}>
      <button class="tool-circle" type="button" aria-label={t("volume")} aria-expanded={activeTool === "volume"} onclick={(event) => { event.stopPropagation(); toggleTool("volume"); }}>
        {#if muted || volume === 0}<VolumeX size={16} />{:else}<Volume2 size={16} />{/if}
      </button>
      <div class="tool-content volume-content" data-stop-toggle>
        <input aria-label={t("volume")} type="range" min="0" max="100" value={volume} oninput={(event) => onVolume?.(Number(event.currentTarget.value))} />
        <span>{muted ? t("muted") : `${volume}%`}</span>
      </div>
    </div>
  {/if}

  {#if enabledTools.includes("timer")}
    <div class="tool-item timer-item">
      <button class="tool-circle" type="button" aria-label={t("timer")} onclick={(event) => { event.stopPropagation(); openTimer(); }}>
        <Timer size={16} />
      </button>
    </div>
  {/if}
</div>

<style>
  .feature-rail{position:relative;z-index:4;display:flex;flex-direction:column;gap:6px;width:32px;opacity:0;transform:translateX(-8px) scale(.94);transform-origin:left center;pointer-events:none;transition:opacity 150ms ease,transform 240ms cubic-bezier(.22,1,.36,1)}.feature-rail.visible{opacity:1;transform:translateX(0) scale(1);pointer-events:auto}.tool-circle{position:relative;z-index:2;display:grid;place-items:center;flex:none;width:32px;height:32px;padding:0;color:rgba(255,255,255,.78);border:1px solid rgba(255,255,255,.12);border-radius:16px;background:var(--rail-background,#000);box-shadow:0 6px 14px rgba(0,0,0,.2);cursor:pointer;transition:color 140ms ease,filter 140ms ease,transform 140ms cubic-bezier(.23,1,.32,1)}.tool-circle:hover{color:#fff;filter:brightness(1.18);transform:scale(1.04)}.tool-circle:active{transform:scale(.94)}.tool-item{position:relative;width:168px;height:32px;flex:none;overflow:hidden;clip-path:inset(0 136px 0 0 round 16px);border:1px solid rgba(255,255,255,.12);border-radius:16px;background:var(--rail-background,#000);box-shadow:0 6px 14px rgba(0,0,0,.2);transform-origin:left center;transition:clip-path 280ms cubic-bezier(.22,1,.36,1),filter 140ms ease}.tool-item.expanded{clip-path:inset(0 round 16px);filter:brightness(1.06)}.tool-item .tool-circle{border:0;box-shadow:none}.tool-content{position:absolute;left:39px;right:10px;top:50%;display:flex;align-items:center;min-width:0;opacity:0;transform:translateY(-50%) translateX(-8px);pointer-events:none;transition:opacity 120ms 80ms ease,transform 180ms 60ms ease}.tool-item.expanded .tool-content{opacity:1;transform:translateY(-50%) translateX(0);pointer-events:auto}.volume-content{gap:7px}.volume-content input{width:82px;height:4px;padding:0;border:0;border-radius:4px;accent-color:#fff;background:rgba(255,255,255,.2);cursor:pointer}.volume-content span{min-width:27px;color:rgba(255,255,255,.84);font-size:10px;font-variant-numeric:tabular-nums;text-align:right}.tool-item input{touch-action:none}button:focus-visible,input:focus-visible{outline:2px solid #fff;outline-offset:2px}@media (prefers-reduced-motion:reduce){.feature-rail,.tool-item,.tool-content,.tool-circle{transition-duration:120ms!important;transform:none}.feature-rail{opacity:0}.feature-rail.visible{opacity:1}.tool-item.expanded .tool-content{transform:translateY(-50%)}}
</style>
