<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { spring, tweened } from "svelte/motion";
  import { ChevronLeft, ChevronRight, Music2, Pause, Play, SkipBack, SkipForward } from "lucide-svelte";
  import MediaProgress from "$lib/MediaProgress.svelte";
  import Spectrum from "$lib/Spectrum.svelte";
  import FeatureRail from "$lib/FeatureRail.svelte";
  import { FEATURE_RAIL_BUTTON_SIZE, FEATURE_RAIL_COLLAPSED_WIDTH, FEATURE_RAIL_GAP, FEATURE_RAIL_WIDTH, featureRailHeight, type IslandTool } from "$lib/featureRail";
  import type { CountdownStatus } from "$lib/countdown";
  import { ISLAND_MOTION, islandMorphEasing, islandSettleEasing } from "$lib/islandMotion";
  import type { MediaState, SpectrumMode } from "$lib/api/types";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import {
    borderRadiusCss,
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

  type PreviewHighlight="shape"|"edge"|"position"|"compactLength"|"shoulder"|"radius"|"background"|"spectrum"|null;

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
    railBackground = background,
    border = "1px solid rgba(255,255,255,.1)",
    boxShadow = "none",
    spectrumTopColor = "#fff",
    spectrumBottomColor = "#888",
    previewSpectrum,
    previewHighlight = null,
    previewEdgePosition = 50,
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
    onTimerOpen,
    onTimerStart,
    onTimerPause,
    onTimerResume,
    onTimerAdjust,
    onTimerReset,
    timerStatus = "idle",
    timerRemainingMs = 0,
    clockText = "00:00",
    clockTimeZone = "system",
    enabledTools = ["floating", "timer"],
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
    railBackground?: string;
    border?: string;
    boxShadow?: string;
    spectrumTopColor?: string;
    spectrumBottomColor?: string;
    previewSpectrum?: number[];
    previewHighlight?: PreviewHighlight;
    previewEdgePosition?: number;
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
    onTimerOpen?: () => void;
    onTimerStart?: (durationMs: number) => void;
    onTimerPause?: () => void;
    onTimerResume?: () => void;
    onTimerAdjust?: (deltaMs: number) => void;
    onTimerReset?: () => void;
    timerStatus?: CountdownStatus;
    timerRemainingMs?: number;
    clockText?: string;
    clockTimeZone?: string;
    enabledTools?: IslandTool[];
  }>();
  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);

  const initial = untrack(() => geometryFor(mode, expandedRadius, edge, compactLength));
  const widthMotion = tweened(initial.width);
  const heightMotion = tweened(initial.height);
  const radiusMotion = tweened(initial.radius);
  const styleMorph = tweened(untrack(() => islandStyle === "edge" ? 1 : 0));
  const hideMorph = spring(untrack(() => simulateHidden && mode === "hidden" ? 1 : 0), { stiffness: 0.18, damping: 0.7, precision: 0.001 });
  let animatedWidth = $state(initial.width);
  let animatedHeight = $state(initial.height);
  let animatedRadius = $state(initial.radius);
  let styleProgress = $state(untrack(() => islandStyle === "edge" ? 1 : 0));
  let hideProgress = $state(untrack(() => simulateHidden && mode === "hidden" ? 1 : 0));
  let prefersReducedMotion = $state(false);
  let activeEnvelope: IslandGeometry = initial;
  let transitionRevision = 0;
  const unsubscribeWidth = widthMotion.subscribe((value) => animatedWidth = value);
  const unsubscribeHeight = heightMotion.subscribe((value) => animatedHeight = value);
  const unsubscribeRadius = radiusMotion.subscribe((value) => animatedRadius = value);
  const unsubscribeStyle = styleMorph.subscribe((value) => styleProgress = value);
  const unsubscribeHide = hideMorph.subscribe((value) => hideProgress = value);

  const size = $derived({ width: animatedWidth, height: animatedHeight, radius: animatedRadius });
  let activeTool = $state<IslandTool | null>(null);
  const expandedGeometry = $derived(geometryFor("expanded", expandedRadius, edge, compactLength));
  const railHeight = $derived(featureRailHeight(enabledTools.length));
  const railWidth = $derived(activeTool ? FEATURE_RAIL_WIDTH : FEATURE_RAIL_COLLAPSED_WIDTH);
  const railExtraRects = $derived.by(() => {
    if (mode !== "expanded" || enabledTools.length === 0) return [];
    return [{
      x: edge === "right" ? -railWidth - FEATURE_RAIL_GAP : expandedGeometry.width + FEATURE_RAIL_GAP,
      y: (expandedGeometry.height - railHeight) / 2,
      width: railWidth,
      height: railHeight,
      radius: FEATURE_RAIL_BUTTON_SIZE / 2,
    }];
  });

  onMount(() => {
    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updatePreference = () => prefersReducedMotion = mediaQuery.matches;
    updatePreference();
    mediaQuery.addEventListener("change", updatePreference);
    return () => mediaQuery.removeEventListener("change", updatePreference);
  });

  async function animateGeometry(target: IslandGeometry, hard: boolean, revision: number) {
    if (hard) {
      await Promise.all([
        widthMotion.set(target.width, { duration: 0 }),
        heightMotion.set(target.height, { duration: 0 }),
        radiusMotion.set(target.radius, { duration: 0 }),
      ]);
      return;
    }

    const horizontalEdge = edge === "top" || edge === "bottom";
    const crossAxis = horizontalEdge ? widthMotion : heightMotion;
    const outwardAxis = horizontalEdge ? heightMotion : widthMotion;
    const crossTarget = horizontalEdge ? target.width : target.height;
    const outwardTarget = horizontalEdge ? target.height : target.width;

    if (mode === "expanded") {
      const cross = crossAxis.set(crossTarget, {
        duration: ISLAND_MOTION.crossAxisDuration,
        easing: islandMorphEasing,
      });
      const radius = radiusMotion.set(target.radius, {
        duration: ISLAND_MOTION.radiusDuration,
        easing: islandMorphEasing,
      });
      const outward = outwardAxis.set(outwardTarget + ISLAND_MOTION.overshoot, {
        delay: ISLAND_MOTION.outwardDelay,
        duration: ISLAND_MOTION.outwardDuration,
        easing: islandMorphEasing,
      }).then(() => {
        if (revision !== transitionRevision) return;
        return outwardAxis.set(outwardTarget, {
          duration: ISLAND_MOTION.settleDuration,
          easing: islandSettleEasing,
        });
      });
      await Promise.all([cross, radius, outward]);
      return;
    }

    const duration = mode === "hover" ? ISLAND_MOTION.hoverDuration : ISLAND_MOTION.collapseDuration;
    await Promise.all([
      outwardAxis.set(outwardTarget, { duration, easing: islandSettleEasing }),
      crossAxis.set(crossTarget, {
        delay: mode === "hover" ? 0 : ISLAND_MOTION.outwardDelay,
        duration,
        easing: islandSettleEasing,
      }),
      radiusMotion.set(target.radius, { duration, easing: islandSettleEasing }),
    ]);
  }

  $effect(() => {
    const target = geometryFor(mode, expandedRadius, edge, compactLength);
    const hard = !enableAnimations || reduceAnimations || prefersReducedMotion;
    const revision = ++transitionRevision;
    activeEnvelope = stableEnvelope(activeEnvelope, target);
    onRegionChange?.({ geometry: activeEnvelope, radii: radiiFor(activeEnvelope, islandStyle, edge), polygon: shapePolygonFor(activeEnvelope, islandStyle, edge, edgeShoulderRadius), extraRects: untrack(() => railExtraRects), settled: false });
    animateGeometry(target, hard, revision).then(() => {
      if (revision !== transitionRevision) return;
      activeEnvelope = target;
      onRegionChange?.({ geometry: target, radii: radiiFor(target, islandStyle, edge), polygon: shapePolygonFor(target, islandStyle, edge, edgeShoulderRadius), extraRects: untrack(() => railExtraRects), settled: true });
    });
  });
  $effect(() => {
    if ((mode !== "expanded" || (activeTool !== null && !enabledTools.includes(activeTool))) && activeTool !== null) activeTool = null;
  });
  $effect(() => {
    const extraRects = railExtraRects;
    if (mode !== "expanded" || enabledTools.length === 0) return;
    onRegionChange?.({ geometry: activeEnvelope, radii: radiiFor(activeEnvelope, islandStyle, edge), polygon: shapePolygonFor(activeEnvelope, islandStyle, edge, edgeShoulderRadius), extraRects, settled: true });
  });
  $effect(() => {
    const hard = !enableAnimations || reduceAnimations || prefersReducedMotion;
    styleMorph.set(
      islandStyle === "edge" ? 1 : 0,
      hard ? { duration: 0 } : { duration: ISLAND_MOTION.styleDuration, easing: islandMorphEasing },
    );
  });
  $effect(() => {
    hideMorph.set(simulateHidden && mode === "hidden" ? 1 : 0, { hard: !enableAnimations || reduceAnimations });
  });
  $effect(() => () => {
    unsubscribeWidth();
    unsubscribeHeight();
    unsubscribeRadius();
    unsubscribeStyle();
    unsubscribeHide();
  });

  const currentRadii = $derived(radiiFor(size, islandStyle, edge));
  const currentPolygon = $derived(interpolatePolygon(
    shapePolygonFor(size, "floating", edge, edgeShoulderRadius),
    shapePolygonFor(size, "edge", edge, edgeShoulderRadius),
    styleProgress,
  ));
  const surfaceClipPath = $derived(polygonCss(currentPolygon));
  const resolvedClipPath = $derived(islandStyle === "edge" ? surfaceClipPath : "none");
  const contentShoulderInset = $derived(Math.min(16, Math.max(0, edgeShoulderRadius)) * styleProgress);
  const anchorTransform = $derived.by(() => {
    const gap = 22 * (1 - styleProgress);
    const mix = (shown: number, hidden: number) => shown + (hidden - shown) * hideProgress;
    if (edge === "top") return `translateX(-50%) translateY(${mix(gap, 2 - size.height)}px)`;
    if (edge === "right") return `translateY(-50%) translateX(${mix(-gap, size.width - 2)}px)`;
    if (edge === "bottom") return `translateX(-50%) translateY(${mix(-gap, size.height - 2)}px)`;
    return `translateY(-50%) translateX(${mix(gap, 2 - size.width)}px)`;
  });
  const outwardProgress = $derived.by(() => {
    const horizontalEdge = edge === "top" || edge === "bottom";
    const compact = geometryFor("hover", expandedRadius, edge, compactLength);
    const expanded = geometryFor("expanded", expandedRadius, edge, compactLength);
    const value = horizontalEdge ? size.height : size.width;
    const start = horizontalEdge ? compact.height : compact.width;
    const end = horizontalEdge ? expanded.height : expanded.width;
    return Math.min(1, Math.max(0, (value - start) / (end - start)));
  });
  const expandedOpacity = $derived(Math.min(1, Math.max(0, (outwardProgress - 0.18) / 0.42)));
  const secondaryOpacity = $derived(Math.min(1, Math.max(0, (outwardProgress - 0.42) / 0.38)));
  const compactOpacity = $derived(Math.min(1, Math.max(0, 1 - outwardProgress * 4)));
  const railVisible = $derived(mode === "expanded" && enabledTools.length > 0 && outwardProgress > 0.82);
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

  function handleAnchorFocusOut(event: FocusEvent) {
    const anchor = event.currentTarget;
    const nextTarget = event.relatedTarget;
    if (!(anchor instanceof HTMLElement) || !(nextTarget instanceof Node) || !anchor.contains(nextTarget)) {
      onHoverChange?.(false);
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
  <div
    class="surface-anchor"
    style:transform={anchorTransform}
    role="group"
    onmouseenter={() => onHoverChange?.(true)}
    onmouseleave={() => onHoverChange?.(false)}
    onfocusin={() => onHoverChange?.(true)}
    onfocusout={handleAnchorFocusOut}
  >
  <div
    class="island-surface"
    style:width={`${size.width}px`}
    style:height={`${size.height}px`}
    style:border-radius={borderRadiusCss(currentRadii)}
    style:clip-path={resolvedClipPath}
    style:background={surfaceBackground}
    style:border={surfaceBorder}
    style:box-shadow={boxShadow}
    role="button"
    aria-expanded={mode === "expanded"}
    tabindex={interactive ? 0 : undefined}
    aria-label={t("dynamicIsland")}
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
        <button class="cover compact-cover" type="button" aria-label={t("openPlayer")} onclick={(e) => { e.stopPropagation(); onOpenPlayer?.(); }}>
          {#if media.albumArt}
            {#key media.albumArt}
              <span class="compact-disc" class:spinning={mode === "compact" && media.isPlaying && enableAnimations && !reduceAnimations}>
                <img class="cover-image" src={media.albumArt} alt="" draggable="false" />
              </span>
            {/key}
          {:else}
            <Music2 size={12} />
          {/if}
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
      style={`opacity:${expandedOpacity};transform:translateY(${(1 - expandedOpacity) * -6}px);--secondary-opacity:${secondaryOpacity}`}
      aria-hidden={expandedOpacity <= .5}
    >
      {#if idle}
        <div class="idle-expanded">
          <small>{t("idleInfo")}</small>
          <strong title={idleTitle}>{idleTitle}</strong>
          <span title={idleSubtitle}>{idleSubtitle}</span>
          <div class="idle-controls">
            <button type="button" aria-label={t("previousItem")} onclick={(e) => { e.stopPropagation(); onIdleAction?.("prev"); }}><ChevronLeft size={18}/></button>
            <button type="button" aria-label={idlePaused ? t("resumeCarousel") : t("pauseCarousel")} aria-pressed={idlePaused} onclick={(e) => { e.stopPropagation(); onIdleAction?.("toggle"); }}>{#if idlePaused}<Play size={17}/>{:else}<Pause size={17}/>{/if}</button>
            <button type="button" aria-label={t("nextItem")} onclick={(e) => { e.stopPropagation(); onIdleAction?.("next"); }}><ChevronRight size={18}/></button>
          </div>
        </div>
      {:else}
      <div class="top-row">
        <button class="cover expanded-cover" type="button" aria-label={t("openPlayer")} onclick={(e) => { e.stopPropagation(); onOpenPlayer?.(); }}>
          {#if media.albumArt}{#key media.albumArt}<img class="cover-image" src={media.albumArt} alt="" draggable="false" />{/key}{:else}<Music2 size={30} />{/if}
        </button>
        <div class="metadata">
          <strong title={media.title}>{media.title || t("waitingPlayback")}</strong>
          <span title={media.artist}>{media.artist || t("unknownArtist")}</span>
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
          <button type="button" class="side" aria-label={t("previous")} onclick={(e) => { e.stopPropagation(); onMediaAction?.("prev"); }}><SkipBack size={22} fill="currentColor" /></button>
          <button type="button" class="play" aria-label={media.isPlaying ? t("pause") : t("play")} onclick={(e) => { e.stopPropagation(); onMediaAction?.("play_pause"); }}>
            {#if media.isPlaying}<Pause size={32} fill="currentColor" />{:else}<Play size={32} fill="currentColor" />{/if}
          </button>
          <button type="button" class="side" aria-label={t("next")} onclick={(e) => { e.stopPropagation(); onMediaAction?.("next"); }}><SkipForward size={22} fill="currentColor" /></button>
        </div>
        <span class="control-spacer"></span>
      </div>

      {/if}
    </div>

    {#if previewHighlight}
      <div
        class="studio-preview-highlight"
        class:highlight-shape={previewHighlight === "shape"}
        class:highlight-edge={previewHighlight === "edge" || previewHighlight === "position"}
        class:highlight-length={previewHighlight === "compactLength"}
        class:highlight-shoulder={previewHighlight === "shoulder"}
        class:highlight-radius={previewHighlight === "radius"}
        class:highlight-background={previewHighlight === "background"}
        class:highlight-spectrum={previewHighlight === "spectrum"}
        class:highlight-position={previewHighlight === "position"}
        class:preview-expanded={mode === "expanded"}
        class:length-vertical={edge === "left" || edge === "right"}
        style={`--preview-radius-tl:${Math.max(8, Math.min(45, currentRadii.topLeft))}px;--preview-radius-tr:${Math.max(8, Math.min(45, currentRadii.topRight))}px;--preview-radius-br:${Math.max(8, Math.min(45, currentRadii.bottomRight))}px;--preview-radius-bl:${Math.max(8, Math.min(45, currentRadii.bottomLeft))}px;--preview-shoulder:${Math.max(2, Math.min(16, edgeShoulderRadius))}px`}
        aria-hidden="true"
      >
        {#if previewHighlight === "radius"}
          {#if currentRadii.topLeft > 0}<i class="corner top-left"></i>{/if}
          {#if currentRadii.topRight > 0}<i class="corner top-right"></i>{/if}
          {#if currentRadii.bottomLeft > 0}<i class="corner bottom-left"></i>{/if}
          {#if currentRadii.bottomRight > 0}<i class="corner bottom-right"></i>{/if}
        {/if}
        {#if previewHighlight === "compactLength"}
          <i class="length-handle start"></i>
          <i class="length-handle end"></i>
        {/if}
        {#if previewHighlight === "shoulder"}
          <i class="shoulder-handle start"></i>
          <i class="shoulder-handle end"></i>
        {/if}
        {#if previewHighlight === "background"}
          <i class="background-wash"></i>
        {/if}
        {#if previewHighlight === "spectrum"}
          <i class="spectrum-focus"></i>
        {/if}
      </div>
    {/if}
  </div>

  <div
    class="feature-rail-anchor"
    style={`left:${edge === "right" ? -railWidth - FEATURE_RAIL_GAP : expandedGeometry.width + FEATURE_RAIL_GAP}px;top:${(expandedGeometry.height - railHeight) / 2}px`}
  >
    <FeatureRail
      visible={railVisible}
      {activeTool}
      {enabledTools}
      {railBackground}
      onTool={(tool) => activeTool = tool}
      onFloating={onToggleFloating}
      onTimerOpen={onTimerOpen}
    />
  </div>
  </div>
</div>

<style>
  .island-frame{position:relative;width:100%;height:100%;flex:none;overflow:visible;box-sizing:border-box}.island-frame.simulate-hidden{overflow:hidden}.surface-anchor{position:absolute;z-index:1}.edge-top .surface-anchor{top:0;left:50%}.edge-right .surface-anchor{top:50%;right:0}.edge-bottom .surface-anchor{bottom:0;left:50%}.edge-left .surface-anchor{top:50%;left:0}.feature-rail-anchor{position:absolute;z-index:4;pointer-events:none}
  .island-surface{position:relative;z-index:1;overflow:hidden;flex:none;box-sizing:border-box;color:#fff;font-family:var(--app-font);text-rendering:optimizeLegibility;font-synthesis:none;contain:layout style;transition:transform 140ms cubic-bezier(.23,1,.32,1),box-shadow 180ms cubic-bezier(.23,1,.32,1)}
  .floating-style .island-surface:active{transform:scale(.97) translateZ(0)}.compact-layer,.expanded-layer{position:absolute;z-index:1;box-sizing:border-box;pointer-events:none}
  .compact-layer{inset:0;display:flex;align-items:center;justify-content:space-between;padding:0 8px 0 4px}.compact-layer.edge-inset:not(.vertical){padding-left:calc(4px + var(--shoulder-inset));padding-right:calc(8px + var(--shoulder-inset))}.compact-layer.vertical{flex-direction:column;padding:4px 0 8px}.compact-layer.edge-inset.vertical{padding-top:calc(4px + var(--shoulder-inset));padding-bottom:calc(8px + var(--shoulder-inset))}.compact-layer button,.compact-layer :global(canvas){pointer-events:auto}.time-display{width:100%;text-align:center;color:rgba(255,255,255,.8);font:500 12px/1 var(--app-font);letter-spacing:.05em;font-variant-numeric:tabular-nums;user-select:none}.cover{display:grid;place-items:center;flex:none;padding:0;overflow:hidden;color:rgba(255,255,255,.3);background:rgba(255,255,255,.06);border:0;cursor:pointer;user-select:none}.cover img{width:100%;height:100%;display:block;object-fit:cover;-webkit-user-drag:none;user-select:none}.compact-cover{width:20px;height:20px;border-radius:50%}.expanded-cover{width:52px;height:52px;border-radius:12px;box-shadow:0 8px 22px rgba(0,0,0,.35);outline:1px solid rgba(255,255,255,.1)}.playing-dot{width:3px;height:3px;border-radius:50%}
  .compact-disc{display:block;width:100%;height:100%;transform-origin:center;animation:compact-disc-spin 8s linear infinite;animation-play-state:paused}.compact-disc.spinning{animation-play-state:running;will-change:transform}@keyframes compact-disc-spin{to{transform:rotate(1turn)}}
  .idle-compact{display:block;width:100%;padding:0 7px;overflow:hidden;text-align:center;text-overflow:ellipsis;white-space:nowrap;font:600 11px/1 var(--app-font);color:rgba(255,255,255,.9)}.idle-compact.vertical{writing-mode:vertical-rl;max-height:100%;padding:7px 0}.idle-expanded{position:absolute;inset:0;display:flex;align-items:center;justify-content:center;flex-direction:column;text-align:center;padding:18px 28px;box-sizing:border-box}.idle-expanded small{font-size:9px;letter-spacing:.12em;color:rgba(255,255,255,.45)}.idle-expanded strong{max-width:100%;margin-top:8px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:22px;line-height:1.15}.idle-expanded>span{max-width:100%;margin-top:5px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:11px;color:rgba(255,255,255,.62)}.idle-controls{display:flex;flex:none;gap:10px;margin-top:14px}.idle-controls button{display:grid;place-items:center;width:32px;height:30px;border:1px solid rgba(255,255,255,.12);border-radius:10px;color:#fff;background:rgba(255,255,255,.06);pointer-events:auto;cursor:pointer}
  .cover-image{animation:cover-flip-in 420ms cubic-bezier(.23,1,.32,1)}@keyframes cover-flip-in{from{opacity:0;transform:perspective(500px) rotateY(-70deg) scale(.9)}to{opacity:1;transform:perspective(500px) rotateY(0) scale(1)}}
  .debug-overlay{position:absolute;z-index:4;top:4px;left:50%;display:flex;gap:5px;max-width:calc(100% - 12px);padding:2px 6px;border-radius:5px;transform:translateX(-50%);overflow:hidden;color:#4ade80;background:rgba(0,0,0,.75);font:500 8px/1.3 ui-monospace,monospace;white-space:nowrap;pointer-events:none}.debug-overlay span{overflow:hidden;text-overflow:ellipsis}
  .expanded-layer{left:calc(50% - 150px);top:0;width:300px;height:160px;padding:16px 28px 20px;display:flex;flex-direction:column;transition:opacity 120ms linear}.expanded-layer[aria-hidden="true"]{pointer-events:none}.expanded-layer[aria-hidden="false"]{pointer-events:auto}.top-row{display:flex;align-items:center;gap:12px;margin-bottom:8px;min-height:52px}.metadata{min-width:0;flex:1;font-family:var(--app-font);user-select:none}.metadata strong,.metadata span{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.metadata strong{font-size:13px;line-height:1.2;font-weight:700;letter-spacing:-.02em;margin-bottom:4px}.metadata span{font-size:11px;line-height:1.2;font-weight:500;color:rgba(255,255,255,.68);letter-spacing:.005em}.progress-block,.control-row,.idle-controls{opacity:var(--secondary-opacity);transition:opacity 100ms linear}
  .progress-block{width:100%;margin-bottom:4px}
  .control-row{position:relative;display:grid;grid-template-columns:28px 1fr 28px;align-items:center;width:100%;height:40px}.control-spacer{width:28px}.controls{display:flex;align-items:center;justify-content:center;gap:20px}.controls button{display:grid;place-items:center;width:40px;height:40px;padding:0;color:rgba(255,255,255,.9);background:transparent;border:0;cursor:pointer;transition:transform 140ms cubic-bezier(.23,1,.32,1),color 140ms ease}.controls .side{width:32px;height:32px}.controls .play{color:#fff}.controls button:active,.cover:active{transform:scale(.94)}button:focus-visible{outline:2px solid #fff;outline-offset:2px}
  .studio-preview-highlight{position:absolute;z-index:20;inset:0;overflow:hidden;border-radius:inherit;pointer-events:none}.studio-preview-highlight::before,.studio-preview-highlight::after{content:"";position:absolute;pointer-events:none}.highlight-shape::after{inset:2px;border:2px solid rgba(167,139,250,.9);border-radius:inherit;box-shadow:0 0 12px rgba(167,139,250,.42),inset 0 0 7px rgba(167,139,250,.18)}.highlight-edge::before{background:#60a5fa;box-shadow:0 0 9px rgba(96,165,250,.8)}.edge-top .highlight-edge::before,.edge-bottom .highlight-edge::before{left:18px;right:18px;height:2px}.edge-top .highlight-edge::before{top:2px}.edge-bottom .highlight-edge::before{bottom:2px}.edge-left .highlight-edge::before,.edge-right .highlight-edge::before{top:18px;bottom:18px;width:2px}.edge-left .highlight-edge::before{left:2px}.edge-right .highlight-edge::before{right:2px}.highlight-position::after{width:8px;height:8px;border:2px solid #fff;border-radius:50%;background:#60a5fa;box-shadow:0 0 0 3px rgba(96,165,250,.2),0 0 10px rgba(96,165,250,.92)}.edge-top .highlight-position::after,.edge-bottom .highlight-position::after{left:calc(var(--preview-position) * 1% - 6px)}.edge-top .highlight-position::after{top:-3px}.edge-bottom .highlight-position::after{bottom:-3px}.edge-left .highlight-position::after,.edge-right .highlight-position::after{top:calc(var(--preview-position) * 1% - 6px)}.edge-left .highlight-position::after{left:-3px}.edge-right .highlight-position::after{right:-3px}.corner{position:absolute;width:calc(var(--preview-radius-tl) + 9px);height:calc(var(--preview-radius-tl) + 9px);max-width:48px;max-height:48px;opacity:.95}.corner.top-left{top:2px;left:2px;width:calc(var(--preview-radius-tl) + 9px);height:calc(var(--preview-radius-tl) + 9px);border:2px solid;border-color:#a78bfa transparent transparent #a78bfa;border-radius:var(--preview-radius-tl) 0 0 0}.corner.top-right{top:2px;right:2px;width:calc(var(--preview-radius-tr) + 9px);height:calc(var(--preview-radius-tr) + 9px);border:2px solid;border-color:#a78bfa #a78bfa transparent transparent;border-radius:0 var(--preview-radius-tr) 0 0}.corner.bottom-left{bottom:2px;left:2px;width:calc(var(--preview-radius-bl) + 9px);height:calc(var(--preview-radius-bl) + 9px);border:2px solid;border-color:transparent transparent #a78bfa #a78bfa;border-radius:0 0 0 var(--preview-radius-bl)}.corner.bottom-right{right:2px;bottom:2px;width:calc(var(--preview-radius-br) + 9px);height:calc(var(--preview-radius-br) + 9px);border:2px solid;border-color:transparent #a78bfa #a78bfa transparent;border-radius:0 0 var(--preview-radius-br) 0}.length-handle{position:absolute;width:3px;height:22px;border-radius:999px;background:#60a5fa;box-shadow:0 0 9px rgba(96,165,250,.8)}.length-handle.start{left:2px;top:50%;transform:translateY(-50%)}.length-handle.end{right:2px;top:50%;transform:translateY(-50%)}.length-vertical .length-handle{width:22px;height:3px;left:50%;transform:translateX(-50%)}.length-vertical .length-handle.start{top:2px}.length-vertical .length-handle.end{top:auto;right:auto;bottom:2px}.shoulder-handle{position:absolute;width:3px;height:18px;border-radius:999px;background:#a78bfa;box-shadow:0 0 8px rgba(167,139,250,.8)}.edge-top .shoulder-handle.start{top:2px;left:calc(var(--preview-shoulder) + 1px)}.edge-top .shoulder-handle.end{top:2px;right:calc(var(--preview-shoulder) + 1px)}.edge-bottom .shoulder-handle.start{bottom:2px;left:calc(var(--preview-shoulder) + 1px)}.edge-bottom .shoulder-handle.end{right:calc(var(--preview-shoulder) + 1px);bottom:2px}.edge-left .shoulder-handle,.edge-right .shoulder-handle{width:18px;height:3px;top:calc(var(--preview-shoulder) + 1px)}.edge-left .shoulder-handle.start{left:2px}.edge-left .shoulder-handle.end{bottom:calc(var(--preview-shoulder) + 1px);top:auto;left:2px}.edge-right .shoulder-handle.start{right:2px}.edge-right .shoulder-handle.end{right:2px;top:auto;bottom:calc(var(--preview-shoulder) + 1px)}.background-wash{position:absolute;inset:2px;border-radius:inherit;background:rgba(167,139,250,.14);box-shadow:inset 0 0 0 1px rgba(196,181,253,.72),0 0 16px rgba(167,139,250,.22)}.spectrum-focus{position:absolute;z-index:1;top:50%;right:5px;width:23px;height:21px;border:1px solid #60a5fa;border-radius:6px;background:rgba(96,165,250,.1);box-shadow:0 0 10px rgba(96,165,250,.55);transform:translateY(-50%)}.preview-expanded .spectrum-focus{top:14px;right:22px;width:47px;height:31px;border-radius:8px}
  @media (hover:hover) and (pointer:fine){.controls button:hover{transform:scale(1.06);color:#fff}.cover:hover{filter:brightness(1.08)}}
  @media (prefers-reduced-motion:reduce){.surface-anchor{transition:none!important}.island-surface,.expanded-layer,.controls button{transition-duration:120ms!important}.island-surface{transition-property:opacity,box-shadow!important}.expanded-layer{transform:none!important}.cover-image,.compact-disc{animation:none}.studio-preview-highlight .corner,.studio-preview-highlight::after,.studio-preview-highlight .length-handle,.studio-preview-highlight .shoulder-handle,.studio-preview-highlight .background-wash,.studio-preview-highlight .spectrum-focus{animation:none!important;opacity:.78}}
</style>
