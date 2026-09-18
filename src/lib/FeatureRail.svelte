<script lang="ts">
  import { GalleryHorizontalEnd, Settings, Timer, Volume2, VolumeX } from "lucide-svelte";
  import { formatCountdown, type CountdownStatus } from "$lib/countdown";
  import type { IslandTool } from "$lib/featureRail";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  let {
    visible = false,
    activeTool = null,
    enabledTools = ["settings", "floating", "volume", "timer"],
    railBackground = "#000",
    volume = 50,
    muted = false,
    timerStatus = "idle",
    timerRemainingMs = 0,
    timerFinished = false,
    onTool,
    onSettingsToggle,
    onFloating,
    onAudioOpen,
  } = $props<{
    visible?: boolean;
    activeTool?: IslandTool | null;
    enabledTools?: IslandTool[];
    railBackground?: string;
    volume?: number;
    muted?: boolean;
    timerStatus?: CountdownStatus;
    timerRemainingMs?: number;
    timerFinished?: boolean;
    onTool?: (tool: IslandTool | null) => void;
    onSettingsToggle?: () => void;
    onFloating?: () => void;
    onAudioOpen?: () => void | Promise<void>;
  }>();

  const t = (key: TranslationKey) => translate(key, {}, $locale);
  const timerText = $derived(formatCountdown(timerRemainingMs));
  const labels = $derived($locale.startsWith("en")
    ? { settings: "Settings", floating: "Layout", volume: "Volume", timer: "Timer" }
    : $locale.startsWith("ja")
      ? { settings: "設定", floating: "配置", volume: "音量", timer: "タイマー" }
      : { settings: "设置", floating: "布局", volume: "音量", timer: "倒计时" });

  function selectTool(tool: IslandTool) {
    if (tool === "settings") {
      onTool?.(null);
      onSettingsToggle?.();
      return;
    }
    if (tool === "floating") {
      onTool?.(null);
      onFloating?.();
      return;
    }

    const next = activeTool === tool ? null : tool;
    onTool?.(next);
    if (tool === "volume" && next === "volume") void onAudioOpen?.();
  }
</script>

<div
  class="feature-rail"
  class:visible
  aria-hidden={!visible}
  inert={!visible}
  style={`--rail-background:${railBackground};--tool-count:${Math.max(1, enabledTools.length)}`}
>
  <div class="feature-bar" data-stop-toggle>
    {#if enabledTools.includes("settings")}
      <button class="feature-segment" type="button" aria-label={t("settingsTool")} onclick={(event) => { event.stopPropagation(); selectTool("settings"); }}>
        <Settings size={14} strokeWidth={2.1} />
        <span>{labels.settings}</span>
      </button>
    {/if}

    {#if enabledTools.includes("floating")}
      <button class="feature-segment" type="button" aria-label={t("toggleFloating")} onclick={(event) => { event.stopPropagation(); selectTool("floating"); }}>
        <GalleryHorizontalEnd size={14} strokeWidth={2.1} />
        <span>{labels.floating}</span>
      </button>
    {/if}

    {#if enabledTools.includes("volume")}
      <div
        class="feature-segment volume-segment"
        class:active={activeTool === "volume"}
        role="button"
        tabindex="0"
        aria-label={t("volume")}
        aria-pressed={activeTool === "volume"}
        onclick={(event) => { event.stopPropagation(); selectTool("volume"); }}
        onkeydown={(event) => { if ((event.key === "Enter" || event.key === " ") && event.target === event.currentTarget) { event.preventDefault(); selectTool("volume"); } }}
      >
        {#if muted || volume === 0}<VolumeX size={14} strokeWidth={2.1} />{:else}<Volume2 size={14} strokeWidth={2.1} />{/if}
        <span>{labels.volume}</span>
      </div>
    {/if}

    {#if enabledTools.includes("timer")}
      <button class="feature-segment" class:active={activeTool === "timer"} class:timer-running={timerStatus === "running"} class:timer-paused={timerStatus === "paused"} type="button" aria-label={t("timerTool")} aria-pressed={activeTool === "timer"} onclick={(event) => { event.stopPropagation(); selectTool("timer"); }}>
        <Timer size={14} strokeWidth={2.1} />
        <span>{timerFinished ? labels.timer : timerStatus === "idle" ? labels.timer : timerText}</span>
      </button>
    {/if}
  </div>
</div>

<style>
  .feature-rail {
    position: relative;
    z-index: 4;
    width: 276px;
    height: 40px;
    opacity: 0;
    transform: translateY(-8px) scale(.96);
    transform-origin: center top;
    pointer-events: none;
    transition: opacity 160ms ease, transform 260ms cubic-bezier(.22, 1, .36, 1);
  }

  .feature-rail.visible {
    opacity: 1;
    transform: translateY(0) scale(1);
    pointer-events: auto;
  }

  .feature-bar {
    display: grid;
    grid-template-columns: repeat(var(--tool-count, 4), minmax(0, 1fr));
    width: 100%;
    height: 40px;
    padding: 3px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, .13);
    border-radius: 22px;
    background: rgba(18, 18, 20, .96);
    box-shadow: 0 12px 28px rgba(0, 0, 0, .32), inset 0 1px 0 rgba(255,255,255,.08);
    backdrop-filter: blur(16px) saturate(125%);
    -webkit-backdrop-filter: blur(16px) saturate(125%);
  }

  .feature-segment {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 0;
    gap: 5px;
    padding: 0 8px;
    border: 0;
    border-radius: 18px;
    color: rgba(255, 255, 255, .56);
    background: transparent;
    font: 600 10px/1 var(--app-font, "Segoe UI", sans-serif);
    letter-spacing: .01em;
    text-transform: none;
    white-space: nowrap;
    cursor: pointer;
    transition: color 150ms ease, background 180ms ease, transform 150ms cubic-bezier(.23, 1, .32, 1);
  }

  .feature-segment:hover { color: rgba(255,255,255,.9); background: rgba(255,255,255,.06); }
  .feature-segment:active { transform: scale(.96); }
  .feature-segment.active {
    color: #fff;
    background: #1e6ff0;
    box-shadow: inset 0 0 0 1px rgba(255,255,255,.18), 0 3px 10px rgba(30,111,240,.28);
  }

  .feature-segment.timer-running:not(.active) {
    color: #f5a052;
    background: rgba(242, 139, 49, .08);
  }

  .feature-segment.timer-paused:not(.active) {
    color: rgba(245, 160, 82, .58);
    background: rgba(242, 139, 49, .05);
  }

  button:focus-visible { outline: 2px solid #fff; outline-offset: 2px; }

  @media (max-width: 360px) {
    .feature-rail, .feature-bar { width: 252px; }
    .feature-segment { padding: 0 5px; gap: 3px; font-size: 9px; }
  }

  @media (prefers-reduced-motion: reduce) {
    .feature-rail, .feature-segment { transition: none; }
    .feature-rail { transform: none; }
  }
</style>
