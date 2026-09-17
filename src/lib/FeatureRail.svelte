<script lang="ts">
  import { GalleryHorizontalEnd, Timer } from "lucide-svelte";
  import type { IslandTool } from "$lib/featureRail";
  import { locale, translate, type TranslationKey } from "$lib/i18n";


  let {
    visible = false,
    activeTool = null,
    enabledTools = ["floating", "timer"],
    railBackground = "#000",
    onTool,
    onFloating,
    onTimerOpen,
  } = $props<{
    visible?: boolean;
    activeTool?: IslandTool | null;
    enabledTools?: IslandTool[];
    railBackground?: string;
    onTool?: (tool: IslandTool | null) => void;
    onFloating?: () => void;
    onTimerOpen?: () => void;
  }>();

  const t = (key: TranslationKey) => translate(key, {}, $locale);

  function toggleTool(tool: IslandTool) {
    const next = activeTool === tool ? null : tool;
    onTool?.(next);
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

  {#if enabledTools.includes("timer")}
    <div class="tool-item timer-item">
      <button class="tool-circle" type="button" aria-label={t("timer")} onclick={(event) => { event.stopPropagation(); openTimer(); }}>
        <Timer size={16} />
      </button>
    </div>
  {/if}
</div>

<style>
  .feature-rail{position:relative;z-index:4;display:flex;flex-direction:column;gap:6px;width:32px;opacity:0;transform:translateX(-8px) scale(.94);transform-origin:left center;pointer-events:none;transition:opacity 150ms ease,transform 240ms cubic-bezier(.22,1,.36,1)}.feature-rail.visible{opacity:1;transform:translateX(0) scale(1);pointer-events:auto}.tool-circle{position:relative;z-index:2;display:grid;place-items:center;flex:none;width:32px;height:32px;padding:0;color:rgba(255,255,255,.78);border:1px solid rgba(255,255,255,.12);border-radius:16px;background:var(--rail-background,#000);box-shadow:0 6px 14px rgba(0,0,0,.2);cursor:pointer;transition:color 140ms ease,filter 140ms ease,transform 140ms cubic-bezier(.23,1,.32,1)}.tool-circle:hover{color:#fff;filter:brightness(1.18);transform:scale(1.04)}.tool-circle:active{transform:scale(.94)}.tool-item{position:relative;width:32px;height:32px;flex:none}.tool-item .tool-circle{border:1px solid rgba(255,255,255,.12);box-shadow:0 6px 14px rgba(0,0,0,.2)}button:focus-visible{outline:2px solid #fff;outline-offset:2px}@media (prefers-reduced-motion:reduce){.feature-rail,.tool-item,.tool-circle{transition-duration:120ms!important;transform:none}.feature-rail{opacity:0}.feature-rail.visible{opacity:1}}
</style>
