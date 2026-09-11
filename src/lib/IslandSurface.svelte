<script lang="ts">
  import { untrack } from "svelte";
  import { spring } from "svelte/motion";
  import { ChevronLeft, ChevronRight, Music2, Pause, Play, SkipBack, SkipForward, GalleryHorizontalEnd } from "lucide-svelte";
  import MediaProgress from "$lib/MediaProgress.svelte";
  import Spectrum from "$lib/Spectrum.svelte";
  import type { MediaState, SpectrumMode } from "$lib/api/types";
  import {
    borderRadiusCss,
    expansionForGeometry,
    geometryFor,
    interpolatePolygon,
    radiiFor,
    polygonCss,
    shapePolygonFor,
    stableEnvelope,
    type IslandGeometry,
    type IslandEdge,
    type IslandMode,
    type IslandRegionChange,
    type IslandStyle,
  } from "$lib/islandGeometry";

  let {
    media,
    mode = "compact",
    islandStyle = "floating",
    edge = "top",
    position = 0,
    expandedRadius = 45,
    edgeShoulderRadius = 8,
    compactLength = 80,
    idle = false,
    idleTitle = "",
    idleSubtitle = "",
    idlePaused = false,
    showSpectrum = true,
    spectrumMode = "realtime",
    enableAnimations = true,
    reduceAnimations = false,
    background = "#000",
    border = "1px solid rgba(255,255,255,.1)",
    boxShadow = "none",
    spectrumTopColor = "#fff",
    spectrumBottomColor = "#888",
    previewSpectrum,
    showTime = false,
    timeText = "",
    showDebugInfo = false,
    debugLines = [],
    interactive = true,
    simulateHidden = false,
    onToggle,
    onOpenPlayer,
    onMediaAction,
    onSeek,
    onToggleFloating,
    onHoverChange,
    onRegionChange,
    onIdleAction,
  } = $props<{
    media: MediaState;
    mode?: IslandMode;
    islandStyle?: IslandStyle;
    edge?: IslandEdge;
    position?: number;
    expandedRadius?: number;
    edgeShoulderRadius?: number;
    compactLength?: number;
    idle?: boolean;
    idleTitle?: string;
    idleSubtitle?: string;
    idlePaused?: boolean;
    showSpectrum?: boolean;
    spectrumMode?: SpectrumMode;
    enableAnimations?: boolean;
    reduceAnimations?: boolean;
    background?: string;
    border?: string;
    boxShadow?: string;
    spectrumTopColor?: string;
    spectrumBottomColor?: string;
    previewSpectrum?: number[];
    showTime?: boolean;
    timeText?: string;
    showDebugInfo?: boolean;
    debugLines?: string[];
    interactive?: boolean;
    simulateHidden?: boolean;
    onToggle?: () => void;
    onOpenPlayer?: () => void;
    onMediaAction?: (action: "prev" | "play_pause" | "next") => void;
    onSeek?: (positionMs: number) => void | Promise<void>;
    onToggleFloating?: () => void;
    onHoverChange?: (hovering: boolean) => void;
    onRegionChange?: (change: IslandRegionChange) => void;
    onIdleAction?: (action: "prev" | "toggle" | "next") => void;
  }>();

  const initial = untrack(() => geometryFor(mode, expandedRadius, edge, compactLength));
  const geometry = spring(initial, { stiffness: 0.18, damping: 0.7, precision: 0.1 });
  const styleMorph = spring(untrack(() => islandStyle === "edge" ? 1 : 0), { stiffness: 0.18, damping: 0.7, precision: 0.001 });
  const hideMorph = spring(untrack(() => simulateHidden && mode === "hidden" ? 1 : 0), { stiffness: 0.18, damping: 0.7, precision: 0.001 });
  let size = $state(initial);
  let styleProgress = $state(untrack(() => islandStyle === "edge" ? 1 : 0));
  let hideProgress = $state(untrack(() => simulateHidden && mode === "hidden" ? 1 : 0));
  let activeEnvelope: IslandGeometry = initial;
  let transitionRevision = 0;
  const unsubscribe = geometry.subscribe((value) => {
    size = value;
  });
  const unsubscribeStyle = styleMorph.subscribe((value) => styleProgress = value);
  const unsubscribeHide = hideMorph.subscribe((value) => hideProgress = value);

  $effect(() => {
    const target = geometryFor(mode, expandedRadius, edge, compactLength);
    const hard = !enableAnimations || reduceAnimations;
    const revision = ++transitionRevision;
    activeEnvelope = stableEnvelope(activeEnvelope, target);
    onRegionChange?.({ geometry: activeEnvelope, radii: radiiFor(activeEnvelope, islandStyle, edge), polygon: shapePolygonFor(activeEnvelope, islandStyle, edge, edgeShoulderRadius), settled: false });
    geometry.set(target, { hard }).then(() => {
      if (revision !== transitionRevision) return;
      activeEnvelope = target;
      onRegionChange?.({ geometry: target, radii: radiiFor(target, islandStyle, edge), polygon: shapePolygonFor(target, islandStyle, edge, edgeShoulderRadius), settled: true });
    });
  });
  $effect(() => {
    styleMorph.set(islandStyle === "edge" ? 1 : 0, { hard: !enableAnimations || reduceAnimations });
  });
  $effect(() => {
    hideMorph.set(simulateHidden && mode === "hidden" ? 1 : 0, { hard: !enableAnimations || reduceAnimations });
  });
  $effect(() => () => { unsubscribe(); unsubscribeStyle(); unsubscribeHide(); });

  const expansion = $derived(expansionForGeometry(size, edge, compactLength));
  const currentRadii = $derived(radiiFor(size, islandStyle, edge));
  const currentPolygon = $derived(interpolatePolygon(
    shapePolygonFor(size, "floating", edge, edgeShoulderRadius),
    shapePolygonFor(size, "edge", edge, edgeShoulderRadius),
    styleProgress,
  ));
  const surfaceClipPath = $derived(polygonCss(currentPolygon));
  const contentShoulderInset = $derived(Math.min(16, Math.max(0, edgeShoulderRadius)) * styleProgress);
  const anchorTransform = $derived.by(() => {
    const gap = 22 * (1 - styleProgress);
    const mix = (shown: number, hidden: number) => shown + (hidden - shown) * hideProgress;
    if (edge === "top") return `translateX(-50%) translateY(${mix(gap, 2 - size.height)}px)`;
    if (edge === "right") return `translateY(-50%) translateX(${mix(-gap, size.width - 2)}px)`;
    if (edge === "bottom") return `translateX(-50%) translateY(${mix(-gap, size.height - 2)}px)`;
    return `translateY(-50%) translateX(${mix(gap, 2 - size.width)}px)`;
  });
  const expandedOpacity = $derived(Math.min(1, Math.max(0, (expansion - 0.1) / 0.5)));
  const compactOpacity = $derived(Math.min(1, Math.max(0, 1 - expansion * 3)));
  const surfaceBackground = $derived(mode === "expanded" ? background : "#000");
  const surfaceBorder = $derived(mode === "expanded" ? border : "1px solid transparent");
  function toggleFromSurface(event: MouseEvent | KeyboardEvent) {
    const target = event.target as HTMLElement;
    if (!interactive || target.closest("button,[data-stop-toggle]")) return;
    onToggle?.();
  }

  function keyToggle(event: KeyboardEvent) {
    if ((event.key === "Enter" || event.key === " ") && event.target === event.currentTarget) {
      event.preventDefault();
      onToggle?.();
    }
  }
</script>

<div
  class="island-frame"
  class:floating-style={islandStyle === "floating"}
  class:edge-top={edge === "top"}
  class:edge-right={edge === "right"}
  class:edge-bottom={edge === "bottom"}
  class:edge-left={edge === "left"}
  class:simulate-hidden={simulateHidden && mode === "hidden"}
>
  <div class="surface-anchor" style:transform={anchorTransform}>
  <div
    class="island-surface"
    style:width={`${size.width}px`}
    style:height={`${size.height}px`}
    style:border-radius={borderRadiusCss(currentRadii)}
    style:clip-path={surfaceClipPath}
    style:background={surfaceBackground}
    style:border={surfaceBorder}
    style:box-shadow={boxShadow}
    role="button"
    aria-expanded={mode === "expanded"}
    tabindex={interactive ? 0 : undefined}
    aria-label="灵动岛，点击切换展开状态"
    onmouseenter={() => onHoverChange?.(true)}
    onmouseleave={() => onHoverChange?.(false)}
    onfocusin={() => onHoverChange?.(true)}
    onfocusout={() => onHoverChange?.(false)}
    onclick={toggleFromSurface}
    onkeydown={keyToggle}
  >
    {#if showDebugInfo}
      <div class="debug-overlay" aria-hidden="true">
        {#each debugLines as line}<span>{line}</span>{/each}
      </div>
    {/if}

    <div class="compact-layer" class:vertical={edge === "left" || edge === "right"} class:edge-inset={islandStyle === "edge"} style={`opacity:${compactOpacity};--shoulder-inset:${contentShoulderInset}px`} aria-hidden={expandedOpacity > .5}>
      {#if showTime}
        <span class="time-display">{timeText}</span>
      {:else if idle}
        <span class="idle-compact" class:vertical={edge === "left" || edge === "right"} title={idleTitle}>{idleTitle}</span>
      {:else}
        <button class="cover compact-cover" type="button" aria-label="打开当前播放器" onclick={(e) => { e.stopPropagation(); onOpenPlayer?.(); }}>
          {#if media.albumArt}{#key media.albumArt}<img class="cover-image" src={media.albumArt} alt="" draggable="false" />{/key}{:else}<Music2 size={12} />{/if}
        </button>
        {#if showSpectrum}
          <Spectrum active={compactOpacity > .05} playing={media.isPlaying} mode={spectrumMode} reduceMotion={reduceAnimations} topColor={spectrumTopColor} bottomColor={spectrumBottomColor} values={previewSpectrum} />
        {:else if media.isPlaying}
          <i class="playing-dot" style:background={spectrumTopColor}></i>
        {/if}
      {/if}
    </div>

    <div
      class="expanded-layer"
      style:opacity={expandedOpacity}
      style:transform={`translateX(-50%) translateY(${(1 - expandedOpacity) * 5}px)`}
      aria-hidden={expandedOpacity <= .5}
    >
      {#if idle}
        <div class="idle-expanded">
          <small>空闲信息</small>
          <strong title={idleTitle}>{idleTitle}</strong>
          <span title={idleSubtitle}>{idleSubtitle}</span>
          <div class="idle-controls">
            <button type="button" aria-label="上一项" onclick={(e) => { e.stopPropagation(); onIdleAction?.("prev"); }}><ChevronLeft size={18}/></button>
            <button type="button" aria-label={idlePaused ? "继续轮播" : "暂停轮播"} aria-pressed={idlePaused} onclick={(e) => { e.stopPropagation(); onIdleAction?.("toggle"); }}>{#if idlePaused}<Play size={17}/>{:else}<Pause size={17}/>{/if}</button>
            <button type="button" aria-label="下一项" onclick={(e) => { e.stopPropagation(); onIdleAction?.("next"); }}><ChevronRight size={18}/></button>
          </div>
        </div>
      {:else}
      <div class="top-row">
        <button class="cover expanded-cover" type="button" aria-label="打开当前播放器" onclick={(e) => { e.stopPropagation(); onOpenPlayer?.(); }}>
          {#if media.albumArt}{#key media.albumArt}<img class="cover-image" src={media.albumArt} alt="" draggable="false" />{/key}{:else}<Music2 size={30} />{/if}
        </button>
        <div class="metadata">
          <strong title={media.title}>{media.title || "等待播放..."}</strong>
          <span title={media.artist}>{media.artist || "未知艺术家"}</span>
        </div>
        {#if showSpectrum}
          <Spectrum active={expandedOpacity > .05} playing={media.isPlaying} mode={spectrumMode} reduceMotion={reduceAnimations} topColor={spectrumTopColor} bottomColor={spectrumBottomColor} scale={1.5} values={previewSpectrum} />
        {/if}
      </div>

      <div class="progress-block">
        <MediaProgress position={position} duration={media.durationMs} seekable={Boolean(media.capabilities?.seek)} {onSeek} />
      </div>

      <div class="control-row">
        <span class="control-spacer"></span>
        <div class="controls">
          <button type="button" aria-label="上一首" onclick={(e) => { e.stopPropagation(); onMediaAction?.("prev"); }}><SkipBack size={22} fill="currentColor" /></button>
          <button type="button" class="play" aria-label={media.isPlaying ? "暂停" : "播放"} onclick={(e) => { e.stopPropagation(); onMediaAction?.("play_pause"); }}>
            {#if media.isPlaying}<Pause size={32} fill="currentColor" />{:else}<Play size={32} fill="currentColor" />{/if}
          </button>
          <button type="button" aria-label="下一首" onclick={(e) => { e.stopPropagation(); onMediaAction?.("next"); }}><SkipForward size={22} fill="currentColor" /></button>
        </div>
        <button type="button" class="floating" aria-label="切换悬浮窗" onclick={(e) => { e.stopPropagation(); onToggleFloating?.(); }}><GalleryHorizontalEnd size={18} /></button>
      </div>
      {/if}
    </div>
  </div>
  </div>
</div>

<style>
  .island-frame{position:relative;width:100%;height:100%;flex:none;overflow:visible;box-sizing:border-box}.island-frame.simulate-hidden{overflow:hidden}.surface-anchor{position:absolute;z-index:1;will-change:transform}.edge-top .surface-anchor{top:0;left:50%}.edge-right .surface-anchor{top:50%;right:0}.edge-bottom .surface-anchor{bottom:0;left:50%}.edge-left .surface-anchor{top:50%;left:0}
  .island-surface{position:relative;z-index:1;overflow:hidden;flex:none;box-sizing:border-box;color:#fff;contain:layout paint style;transform:translateZ(0);transition:transform 140ms cubic-bezier(.23,1,.32,1),box-shadow 180ms cubic-bezier(.23,1,.32,1);will-change:clip-path}
  .floating-style .island-surface:active{transform:scale(.97) translateZ(0)}.compact-layer,.expanded-layer{position:absolute;z-index:1;box-sizing:border-box;pointer-events:none}
  .compact-layer{inset:0;display:flex;align-items:center;justify-content:space-between;padding:0 8px 0 4px}.compact-layer.edge-inset:not(.vertical){padding-left:calc(4px + var(--shoulder-inset));padding-right:calc(8px + var(--shoulder-inset))}.compact-layer.vertical{flex-direction:column;padding:4px 0 8px}.compact-layer.edge-inset.vertical{padding-top:calc(4px + var(--shoulder-inset));padding-bottom:calc(8px + var(--shoulder-inset))}.compact-layer button,.compact-layer :global(canvas){pointer-events:auto}.time-display{width:100%;text-align:center;color:rgba(255,255,255,.8);font:500 12px/1 var(--app-font);letter-spacing:.05em;font-variant-numeric:tabular-nums;user-select:none}.cover{display:grid;place-items:center;flex:none;padding:0;overflow:hidden;color:rgba(255,255,255,.3);background:rgba(255,255,255,.06);border:0;cursor:pointer;user-select:none}.cover img{width:100%;height:100%;display:block;object-fit:cover;-webkit-user-drag:none;user-select:none}.compact-cover{width:20px;height:20px;border-radius:50%}.expanded-cover{width:52px;height:52px;border-radius:12px;box-shadow:0 8px 22px rgba(0,0,0,.35);outline:1px solid rgba(255,255,255,.1)}.playing-dot{width:3px;height:3px;border-radius:50%}
  .idle-compact{display:block;width:100%;padding:0 7px;overflow:hidden;text-align:center;text-overflow:ellipsis;white-space:nowrap;font:600 11px/1 var(--app-font);color:rgba(255,255,255,.9)}.idle-compact.vertical{writing-mode:vertical-rl;max-height:100%;padding:7px 0}.idle-expanded{width:100%;height:100%;display:flex;align-items:center;justify-content:center;flex-direction:column;text-align:center;padding:18px 28px}.idle-expanded small{font-size:9px;letter-spacing:.12em;color:rgba(255,255,255,.45)}.idle-expanded strong{max-width:100%;margin-top:8px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:22px;line-height:1.15}.idle-expanded>span{max-width:100%;margin-top:5px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:11px;color:rgba(255,255,255,.62)}.idle-controls{display:flex;gap:10px;margin-top:16px}.idle-controls button{display:grid;place-items:center;width:32px;height:30px;border:1px solid rgba(255,255,255,.12);border-radius:10px;color:#fff;background:rgba(255,255,255,.06);pointer-events:auto;cursor:pointer}
  .cover-image{animation:cover-flip-in 420ms cubic-bezier(.23,1,.32,1)}@keyframes cover-flip-in{from{opacity:0;transform:perspective(500px) rotateY(-70deg) scale(.9)}to{opacity:1;transform:perspective(500px) rotateY(0) scale(1)}}
  .debug-overlay{position:absolute;z-index:4;top:4px;left:50%;display:flex;gap:5px;max-width:calc(100% - 12px);padding:2px 6px;border-radius:5px;transform:translateX(-50%);overflow:hidden;color:#4ade80;background:rgba(0,0,0,.75);font:500 8px/1.3 ui-monospace,monospace;white-space:nowrap;pointer-events:none}.debug-overlay span{overflow:hidden;text-overflow:ellipsis}
  .expanded-layer{left:50%;top:0;width:300px;height:160px;padding:20px 28px 16px;display:flex;flex-direction:column;transition:opacity 120ms linear;will-change:transform,opacity}.expanded-layer[aria-hidden="true"]{pointer-events:none}.expanded-layer[aria-hidden="false"]{pointer-events:auto}.top-row{display:flex;align-items:center;gap:12px;margin-bottom:12px;min-height:52px}.metadata{min-width:0;flex:1;font-family:var(--app-font);user-select:none}.metadata strong,.metadata span{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.metadata strong{font-size:13px;line-height:1.15;font-weight:700;letter-spacing:-.03em;margin-bottom:4px}.metadata span{font-size:11px;line-height:1.15;font-weight:500;color:rgba(255,255,255,.65)}
  .progress-block{width:100%;margin-bottom:8px}
  .control-row{position:relative;display:grid;grid-template-columns:28px 1fr 28px;align-items:center;width:100%;height:40px}.control-spacer{width:28px}.controls{display:flex;align-items:center;justify-content:center;gap:20px}.controls button,.floating{display:grid;place-items:center;padding:0;color:rgba(255,255,255,.9);background:transparent;border:0;cursor:pointer;transition:transform 140ms cubic-bezier(.23,1,.32,1),color 140ms ease}.controls button{width:32px;height:32px}.controls .play{width:40px;height:40px;color:#fff}.floating{width:28px;height:28px;border-radius:12px;border:1px solid rgba(255,255,255,.1)}.controls button:active,.floating:active,.cover:active{transform:scale(.94)}button:focus-visible{outline:2px solid #fff;outline-offset:2px}
  @media (hover:hover) and (pointer:fine){.controls button:hover,.floating:hover{transform:scale(1.06);color:#fff}.cover:hover{filter:brightness(1.08)}}
  @media (prefers-reduced-motion:reduce){.surface-anchor{transition:none!important}.island-surface,.expanded-layer,.controls button,.floating{transition-duration:120ms!important}.island-surface{transition-property:opacity,box-shadow!important}.expanded-layer{transform:translateX(-50%)!important}.cover-image{animation:none}}
</style>
