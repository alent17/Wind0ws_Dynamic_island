<script lang="ts">
  import { untrack } from "svelte";
  import { spring } from "svelte/motion";
  import {
    GalleryHorizontalEnd,
    Music2,
    Pause,
    Play,
    SkipBack,
    SkipForward,
  } from "lucide-svelte";
  import type { MediaState } from "$lib/api/types";

  export type LegacyIslandMode = "compact" | "hover" | "expanded" | "hidden";

  let {
    media,
    mode = "expanded",
    position = 0,
    onPreviewPlayPause,
  } = $props<{
    media: MediaState;
    mode?: LegacyIslandMode;
    position?: number;
    onPreviewPlayPause?: () => void;
  }>();

  const geometryFor = (value: LegacyIslandMode) => {
    if (value === "expanded") return { width: 300, height: 160, radius: 45 };
    if (value === "hover") return { width: 90, height: 30, radius: 24 };
    if (value === "hidden") return { width: 80, height: 6, radius: 3 };
    return { width: 80, height: 28, radius: 24 };
  };

  const initialGeometry = untrack(() => geometryFor(mode));
  const geometry = spring(initialGeometry, {
    stiffness: 0.14,
    damping: 0.72,
    precision: 0.05,
  });
  let size = $state(initialGeometry);
  const unsubscribe = geometry.subscribe((value) => (size = value));

  $effect(() => {
    geometry.set(geometryFor(mode));
  });
  $effect(() => () => unsubscribe());

  let progress = $derived(
    media.durationMs > 0
      ? Math.min(100, Math.max(0, (position / media.durationMs) * 100))
      : 0,
  );

  function formatTime(ms: number) {
    const seconds = Math.max(0, Math.floor(ms / 1000));
    return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
  }
</script>

<div
  class="legacy-island"
  class:expanded={mode === "expanded"}
  class:hidden={mode === "hidden"}
  style={`width:${size.width}px;height:${size.height}px;border-radius:${size.radius}px`}
  aria-label={`旧版灵动岛 ${mode} 预览`}
>
  {#if mode === "expanded"}
    <div class="expanded-content">
      <div class="top-row">
        <div class="cover cover-large">
          {#if media.albumArt}
            <img src={media.albumArt} alt="" />
          {:else}
            <Music2 size={27} />
          {/if}
        </div>
        <div class="metadata">
          <strong title={media.title}>{media.title || "等待播放..."}</strong>
          <span title={media.artist}>{media.artist || "未知艺术家"}</span>
        </div>
        <div class="spectrum spectrum-large" class:paused={!media.isPlaying} aria-label="频谱预览">
          {#each [0.45, 0.78, 0.58, 0.96, 0.7, 0.38] as level, index}
            <i style={`--level:${level};--index:${index}`}></i>
          {/each}
        </div>
      </div>

      <div class="progress">
        <div class="track"><i style={`width:${progress}%`}></i></div>
        <div class="times">
          <span>{formatTime(position)}</span>
          <span>-{formatTime(Math.max(0, media.durationMs - position))}</span>
        </div>
      </div>

      <div class="control-row" aria-label="播放控制预览">
        <span class="control-spacer"></span>
        <div class="controls">
          <button type="button" aria-label="上一首预览"><SkipBack size={22} fill="currentColor" /></button>
          <button type="button" class="play" aria-label={media.isPlaying ? "暂停预览" : "播放预览"} onclick={onPreviewPlayPause}>
            {#if media.isPlaying}<Pause size={29} fill="currentColor" />{:else}<Play size={29} fill="currentColor" />{/if}
          </button>
          <button type="button" aria-label="下一首预览"><SkipForward size={22} fill="currentColor" /></button>
        </div>
        <button type="button" class="floating" aria-label="悬浮窗按钮预览"><GalleryHorizontalEnd size={17} /></button>
      </div>
    </div>
  {:else if mode !== "hidden"}
    <div class="compact-content">
      <div class="cover">
        {#if media.albumArt}<img src={media.albumArt} alt="" />{:else}<Music2 size={12} />{/if}
      </div>
      <div class="spectrum" class:paused={!media.isPlaying} aria-label="频谱预览">
        {#each [0.45, 0.78, 0.58, 0.96, 0.7, 0.38] as level, index}
          <i style={`--level:${level};--index:${index}`}></i>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .legacy-island {
    position: relative;
    z-index: 1;
    overflow: hidden;
    flex: none;
    box-sizing: border-box;
    color: #fff;
    background: #000;
    border: 1px solid rgba(255, 255, 255, 0.1);
    transform: translateZ(0);
    will-change: width, height, border-radius;
  }

  .legacy-island.hidden {
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.22);
  }

  .compact-content {
    width: 100%;
    height: 100%;
    padding: 0 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    box-sizing: border-box;
  }

  .cover {
    width: 20px;
    height: 20px;
    display: grid;
    place-items: center;
    overflow: hidden;
    flex: none;
    border-radius: 4px;
    color: rgba(255, 255, 255, 0.24);
    background: rgba(255, 255, 255, 0.05);
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-large {
    width: 52px;
    height: 52px;
    border-radius: 12px;
    box-shadow: 0 10px 24px rgba(0, 0, 0, 0.44);
  }

  .spectrum {
    width: 20px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1.5px;
  }

  .spectrum i {
    width: 2px;
    height: 14px;
    border-radius: 1px;
    background: linear-gradient(to top, #888, #fff);
    transform: scaleY(var(--level));
    transform-origin: center;
    animation: meter 720ms calc(var(--index) * -93ms) ease-in-out infinite alternate;
  }

  .spectrum-large {
    width: 30px;
    height: 40px;
    gap: 2px;
    flex: none;
  }

  .spectrum-large i {
    width: 3px;
    height: 21px;
    border-radius: 1.5px;
  }

  .spectrum.paused i {
    animation-play-state: paused;
    transform: scaleY(0.18);
    opacity: 0.6;
  }

  .expanded-content {
    width: 300px;
    height: 160px;
    padding: 20px 28px 16px;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
  }

  .top-row {
    display: grid;
    grid-template-columns: 52px minmax(0, 1fr) 30px;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }

  .metadata {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .metadata strong,
  .metadata span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .metadata strong {
    color: #fff;
    font-size: 15px;
    font-weight: 700;
    line-height: 1.1;
    letter-spacing: -0.03em;
  }

  .metadata span {
    color: rgba(255, 255, 255, 0.8);
    font-size: 11px;
    font-weight: 500;
  }

  .progress {
    margin-bottom: 4px;
  }

  .track {
    width: 100%;
    height: 3px;
    overflow: hidden;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.12);
  }

  .track i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.4), rgba(255, 255, 255, 0.8));
  }

  .times {
    display: flex;
    justify-content: space-between;
    margin-top: 4px;
    color: rgba(255, 255, 255, 0.6);
    font-size: 9px;
    font-variant-numeric: tabular-nums;
  }

  .control-row {
    display: grid;
    grid-template-columns: 28px 1fr 28px;
    align-items: center;
    margin-top: auto;
  }

  .controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 20px;
  }

  button {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: 50%;
    color: rgba(255, 255, 255, 0.9);
    background: transparent;
    cursor: pointer;
    transition: transform 160ms ease-out, background 160ms ease;
  }

  button:hover {
    transform: scale(1.1);
  }

  button:active {
    transform: scale(0.9);
  }

  button:focus-visible {
    outline: 2px solid #fff;
    outline-offset: 1px;
  }

  .play {
    width: 36px;
    height: 36px;
    color: #fff;
  }

  .floating {
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  @keyframes meter {
    to { transform: scaleY(0.24); }
  }

  @media (prefers-reduced-motion: reduce) {
    .legacy-island { will-change: auto; }
    .spectrum i { animation: none; }
  }
</style>
