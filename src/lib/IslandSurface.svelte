<script lang="ts">
  import { onMount, untrack, tick } from "svelte";
  import { spring } from "svelte/motion";
  import { ArrowLeft, Clock, CloudSun, EyeOff, GalleryHorizontalEnd, Music2, Play, Settings, SkipBack, SkipForward, Timer, Volume2, VolumeX } from "lucide-svelte";
  import MediaProgress from "$lib/MediaProgress.svelte";
  import PlayPauseIcon from "$lib/PlayPauseIcon.svelte";
  import RollingNumber from "$lib/RollingNumber.svelte";
  import Spectrum from "$lib/Spectrum.svelte";
  import TimerPanel from "$lib/TimerPanel.svelte";
  import MarqueeTitle from "$lib/MarqueeTitle.svelte";
  import InfoPanel from "$lib/InfoPanel.svelte";
  import FeatureMenu from "$lib/FeatureMenu.svelte";
  import VolumePanel from "$lib/VolumePanel.svelte";
  import WeatherIcon from "$lib/WeatherIcon.svelte";
  import type { IslandTool } from "$lib/featureRail";
  import { formatCountdown, type CountdownStatus } from "$lib/countdown";
  import { ISLAND_SPRING } from "$lib/islandMotion";
  import type { AudioDeviceInfo, MediaState, SpectrumMode, SystemAudioState, WeatherForecastDay } from "$lib/api/types";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import {
    navigationGeometry,
    type IslandPage,
    borderRadiusCss,
    geometryFor,
    interpolatePolygon,
    radiiFor,
    polygonCss,
    shapePolygonFor,
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
    collapsedEdgeShoulderRadius = 8,
    expandedEdgeShoulderRadius = 32,
    compactLength = 80,
    idle = false,
    idleTime = "",
    idleWeatherTemperature = null,
    idleWeatherCode = null,
    idleWeatherForecast = [],
    lastPlayedMedia = null,
    canResumeLastTrack = false,
    resumingLastTrack = false,
    showSpectrum = true,
    spectrumMode = "realtime",
    enableAnimations = true,
    reduceAnimations = false,
    background = "#000",
    border = "1px solid rgba(255,255,255,.1)",
    boxShadow = "none",
    showCustomFunctionPanel = false,
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
    onResumeLastTrack,
    onMediaAction,
    onSeek,
    onToggleFloating,
    onHideForTenSeconds,
    onHoverChange,
    onRegionChange,
    systemAudio,
    onSettingsToggle,
    onAudioOpen,
    onAudioVolume,
    audioDevices = [],
    audioDeviceSwitching = false,
    onAudioDevice,
    onTimerStart,
    onTimerPause,
    onTimerResume,
    onTimerAdjust,
    onTimerReset,
    onTimerFinishedDismiss,
    onPanelActivity,
    onPageChange,
    visible = true,
    weatherCity = "",
    weatherUpdatedAt = null,
    weatherLoading = false,
    weatherFailed = false,
    timerStatus = "idle",
    timerRemainingMs = 0,
    timerDurationMs = 0,
    timerLabel = "倒计时",
    timerFinished = false,
    clockText = "00:00",
    clockTimeZone = "system",
    enabledTools = ["settings", "floating", "volume", "timer", "hide"],
  } = $props<{
    media: MediaState;
    mode?: IslandMode;
    islandStyle?: IslandStyle;
    edge?: IslandEdge;
    position?: number;
    expandedRadius?: number;
    collapsedEdgeShoulderRadius?: number;
    expandedEdgeShoulderRadius?: number;
    compactLength?: number;
    idle?: boolean;
    idleTime?: string;
    idleWeatherTemperature?: number | null;
    idleWeatherCode?: number | null;
    idleWeatherForecast?: WeatherForecastDay[];
    lastPlayedMedia?: { title: string; artist: string; albumArt: string; sourceDisplay: string } | null;
    canResumeLastTrack?: boolean;
    resumingLastTrack?: boolean;
    showSpectrum?: boolean;
    spectrumMode?: SpectrumMode;
    enableAnimations?: boolean;
    reduceAnimations?: boolean;
    background?: string;
    border?: string;
    boxShadow?: string;
    showCustomFunctionPanel?: boolean;
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
    onResumeLastTrack?: () => void;
    onMediaAction?: (action: "prev" | "play_pause" | "next") => void;
    onSeek?: (positionMs: number) => void | Promise<void>;
    onToggleFloating?: () => void;
    onHideForTenSeconds?: () => void;
    onSettingsToggle?: () => void;
    onHoverChange?: (hovering: boolean) => void;
    onRegionChange?: (change: IslandRegionChange) => void;
    systemAudio?: SystemAudioState | null;
    onAudioOpen?: () => void | Promise<void>;
    onAudioVolume?: (volumePercent: number) => void | Promise<void>;
    audioDevices?: AudioDeviceInfo[];
    audioDeviceSwitching?: boolean;
    onAudioDevice?: (deviceId: string) => void | Promise<void>;
    onTimerStart?: (durationMs: number) => void;
    onTimerPause?: () => void;
    onTimerResume?: () => void;
    onTimerAdjust?: (deltaMs: number) => void;
    onTimerReset?: () => void;
    onTimerFinishedDismiss?: () => void;
    onPanelActivity?: (active: boolean) => void;
    onPageChange?: (page: IslandPage) => void;
    visible?: boolean;
    weatherCity?: string;
    weatherUpdatedAt?: number | null;
    weatherLoading?: boolean;
    weatherFailed?: boolean;
    timerStatus?: CountdownStatus;
    timerRemainingMs?: number;
    timerDurationMs?: number;
    timerLabel?: string;
    timerFinished?: boolean;
    clockText?: string;
    clockTimeZone?: string;
    enabledTools?: IslandTool[];
  }>();
  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);

  const timerActive = $derived(timerStatus !== "idle" || timerFinished);
  const timerText = $derived(formatCountdown(timerRemainingMs));
  const timerUrgent = $derived(timerStatus === "running" && timerRemainingMs > 0 && timerRemainingMs <= 10_000);
  const timerProgress = $derived(timerFinished ? 1 : timerDurationMs > 0 ? Math.min(1, Math.max(0, timerRemainingMs / timerDurationMs)) : 0);
  const timerFinishedText = $derived(t("timerComplete"));
  const effectiveCompactLength = $derived(timerActive ? Math.max(compactLength, 240) : compactLength);
  const initial = untrack(() => geometryFor(
    mode,
    expandedRadius,
    edge,
    effectiveCompactLength,
    0,
  ));
  const widthMotion = spring(initial.width, ISLAND_SPRING);
  const heightMotion = spring(initial.height, ISLAND_SPRING);
  const radiusMotion = spring(initial.radius, ISLAND_SPRING);
  const styleMorph = spring(untrack(() => islandStyle === "edge" ? 1 : 0), ISLAND_SPRING);
  const hideMorph = spring(untrack(() => simulateHidden && mode === "hidden" ? 1 : 0), { stiffness: 0.18, damping: 0.7, precision: 0.001 });
  let animatedWidth = $state(initial.width);
  let animatedHeight = $state(initial.height);
  let animatedRadius = $state(initial.radius);
  let styleProgress = $state(untrack(() => islandStyle === "edge" ? 1 : 0));
  let hideProgress = $state(untrack(() => simulateHidden && mode === "hidden" ? 1 : 0));
  let prefersReducedMotion = $state(false);
  const shoulderMotion = spring(untrack(() => mode === "expanded" ? expandedEdgeShoulderRadius : collapsedEdgeShoulderRadius), ISLAND_SPRING);
  let animatedShoulder = $state(untrack(() => mode === "expanded" ? expandedEdgeShoulderRadius : collapsedEdgeShoulderRadius));
  const unsubscribeShoulder = shoulderMotion.subscribe(value => animatedShoulder = value);
  let shapeSettled = $state(false);
  let transitionRevision = 0;
  const unsubscribeWidth = widthMotion.subscribe((value) => animatedWidth = value);
  const unsubscribeHeight = heightMotion.subscribe((value) => animatedHeight = value);
  const unsubscribeRadius = radiusMotion.subscribe((value) => animatedRadius = value);
  const unsubscribeStyle = styleMorph.subscribe((value) => styleProgress = Math.max(0, Math.min(1, value)));
  const unsubscribeHide = hideMorph.subscribe((value) => hideProgress = value);

  const size = $derived({ width: Math.max(1, animatedWidth), height: Math.max(1, animatedHeight), radius: Math.max(0, Math.min(80, animatedRadius)) });
  let page = $state<IslandPage>("music");
  let wasTimerFinished = $state(false);
  let menuScroll = $state(0);
  let pageRoot = $state<HTMLDivElement>();
  const panelVisible = $derived(showCustomFunctionPanel && enabledTools.length > 0);
  const toolOrder: IslandTool[] = ["timer", "volume", "floating", "settings", "hide", "clock", "weather"];
  const toolIcons = { timer: Timer, volume: Volume2, floating: GalleryHorizontalEnd, settings: Settings, hide: EyeOff, clock: Clock, weather: CloudSun };
  const toolLabels: Record<IslandTool, TranslationKey> = { timer: "timer", volume: "volume", floating: "floatingTool", settings: "settingsTool", hide: "hideTool", clock: "clock", weather: "weather" };
  const shortLabels: Record<IslandTool, TranslationKey> = {timer:"timer",volume:"menuVolume",floating:"menuFloating",settings:"menuSettings",hide:"menuHide",clock:"clock",weather:"weather"};
  const menuItems = $derived(toolOrder.filter(tool => enabledTools.includes(tool)).map(tool => ({ id: tool, label: t(toolLabels[tool]), shortLabel: t(shortLabels[tool]), icon: toolIcons[tool] })));
  const expandedGeometry = $derived(navigationGeometry(page, panelVisible ? menuItems.length : 0, expandedRadius, islandStyle, edge, expandedEdgeShoulderRadius));

  async function navigate(next: IslandPage) {
    page = next;
    await tick();
    pageRoot?.querySelector<HTMLButtonElement>(next === "music" ? ".feature-menu button" : ".page-back")?.focus({ preventScroll: true });
  }
  function goBack() {
    if (page === "music") { onToggle?.(); return; }
    void navigate("music");
  }
  function selectTool(id: string) {
    if (id === "timer" || id === "volume" || id === "clock" || id === "weather") {
      void navigate(id);
      if (id === "volume") void onAudioOpen?.();
    } else if (id === "floating") onToggleFloating?.();
    else if (id === "settings") onSettingsToggle?.();
    else if (id === "hide") onHideForTenSeconds?.();
  }

  onMount(() => {
    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updatePreference = () => prefersReducedMotion = mediaQuery.matches;
    updatePreference();
    mediaQuery.addEventListener("change", updatePreference);
    return () => mediaQuery.removeEventListener("change", updatePreference);
  });

  function notifyRegionChange(change: IslandRegionChange) {
    // The App callback reads media state while calculating the host offset.
    // Do not let those reads make this geometry effect restart on every media
    // clock update while playback is active.
    untrack(() => onRegionChange?.(change));
  }

  $effect(() => {
    const target = mode === "expanded" ? expandedGeometry : geometryFor(mode, expandedRadius, edge, effectiveCompactLength);
    const hard = !enableAnimations || reduceAnimations || prefersReducedMotion;
    const revision = ++transitionRevision;
    shapeSettled = false;
    void Promise.all([
      widthMotion.set(target.width, { hard }),
      heightMotion.set(target.height, { hard }),
      radiusMotion.set(target.radius, { hard }),
      shoulderMotion.set(mode === "expanded" ? expandedEdgeShoulderRadius : collapsedEdgeShoulderRadius, { hard }),
      styleMorph.set(islandStyle === "edge" ? 1 : 0, { hard }),
    ]).then(() => { if (revision === transitionRevision) shapeSettled = true; });
  });
  $effect(() => {
    if (mode !== "expanded" || !panelVisible) { page = "music"; menuScroll = 0; }
    else if (page !== "music" && !enabledTools.includes(page)) page = "music";
  });
  $effect(() => {
    const finished = timerFinished;
    if (finished && !wasTimerFinished && panelVisible && enabledTools.includes("timer")) void navigate("timer");
    else if (!finished && wasTimerFinished && page === "timer") void navigate("music");
    wasTimerFinished = finished;
  });
  $effect(() => { onPanelActivity?.(mode === "expanded" && page !== "music"); onPageChange?.(page); });
  let renderedPage = $state<IslandPage>("music");
  let pageExiting = $state(false);
  $effect(() => {
    const next = page;
    if (mode !== "expanded" || !visible || !enableAnimations || reduceAnimations || prefersReducedMotion) { renderedPage = next; pageExiting = false; return; }
    if (untrack(() => renderedPage) === next) { pageExiting = false; return; }
    pageExiting = true;
    const timeout = setTimeout(() => {
      renderedPage = next; pageExiting = false;
      void tick().then(() => { if (page === next && mode === "expanded") pageRoot?.querySelector<HTMLButtonElement>(next === "music" ? ".feature-menu button" : ".page-back")?.focus({preventScroll:true}); });
    }, 100);
    return () => clearTimeout(timeout);
  });
  $effect(() => {
    hideMorph.set(simulateHidden && mode === "hidden" ? 1 : 0, { hard: !enableAnimations || reduceAnimations || prefersReducedMotion });
  });
  $effect(() => () => {
    ++transitionRevision;
    unsubscribeShoulder();
    unsubscribeWidth();
    unsubscribeHeight();
    unsubscribeRadius();
    unsubscribeStyle();
    unsubscribeHide();
  });

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
    const compact = geometryFor("hover", expandedRadius, edge, effectiveCompactLength);
    const expanded = expandedGeometry;
    const value = horizontalEdge ? size.height : size.width;
    const start = horizontalEdge ? compact.height : compact.width;
    const end = horizontalEdge ? expanded.height : expanded.width;
    return Math.min(1, Math.max(0, (value - start) / (end - start)));
  });
  const currentShoulderRadius = $derived(Math.max(0, Math.min(64, animatedShoulder)));
  const shoulderMarkerOffset = $derived.by(() => {
    const verticalEdge = edge === "left" || edge === "right";
    const canonicalWidth = verticalEdge ? size.height : size.width;
    const canonicalHeight = verticalEdge ? size.width : size.height;
    return Math.min(currentShoulderRadius, canonicalWidth / 4, canonicalHeight / 2);
  });
  const contentShoulderInset = $derived(shoulderMarkerOffset * styleProgress);
  const currentRadii = $derived.by(() => {
    const floating = radiiFor(size, "floating", edge);
    const attached = radiiFor(size, "edge", edge);
    return { topLeft: floating.topLeft + (attached.topLeft - floating.topLeft) * styleProgress,
      topRight: floating.topRight + (attached.topRight - floating.topRight) * styleProgress,
      bottomLeft: floating.bottomLeft + (attached.bottomLeft - floating.bottomLeft) * styleProgress,
      bottomRight: floating.bottomRight + (attached.bottomRight - floating.bottomRight) * styleProgress };
  });
  const currentPolygon = $derived(interpolatePolygon(
    shapePolygonFor(size, "floating", edge, 0),
    shapePolygonFor(size, "edge", edge, currentShoulderRadius),
    styleProgress,
  ));
  const surfaceClipPath = $derived(polygonCss(currentPolygon));
  const resolvedClipPath = $derived(styleProgress > 0 || islandStyle === "edge" ? surfaceClipPath : "none");
  $effect(() => {
    notifyRegionChange({ geometry: size, radii: currentRadii, polygon: currentPolygon, extraRects: [], anchorGap: 22 * (1 - styleProgress), settled: shapeSettled });
  });
  const expandedOpacity = $derived(Math.min(1, Math.max(0, (outwardProgress - 0.18) / 0.42)));
  const secondaryOpacity = $derived(Math.min(1, Math.max(0, (outwardProgress - 0.42) / 0.38)));
  const compactOpacity = $derived(Math.min(1, Math.max(0, 1 - outwardProgress * 4)));
  const surfaceBackground = $derived(mode === "expanded" ? background : "#000");
  const surfaceBorder = $derived(mode === "expanded" ? border : "1px solid transparent");
  const controlSelector = 'button,input,select,textarea,a,[role="slider"],[role="listbox"],[data-stop-toggle]';
  let gestureX = 0;
  let gestureY = 0;
  let suppressBackgroundClick = false;
  function startSurfaceGesture(event: PointerEvent) {
    gestureX = event.clientX; gestureY = event.clientY;
    suppressBackgroundClick = Boolean((event.target as HTMLElement).closest(controlSelector));
  }
  function moveSurfaceGesture(event: PointerEvent) {
    if (event.buttons && Math.hypot(event.clientX - gestureX, event.clientY - gestureY) > 5) suppressBackgroundClick = true;
  }
  function toggleFromSurface(event: MouseEvent | KeyboardEvent) {
    const target = event.target as HTMLElement;
    if (!interactive || target.closest(controlSelector) || (event.detail > 0 && suppressBackgroundClick)) return;
    onToggle?.();
  }

  function keyToggle(event: KeyboardEvent) {
    if (event.key === "Escape" && mode === "expanded" && !event.defaultPrevented) { event.preventDefault(); event.stopPropagation(); goBack(); return; }
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
  class:attached-style={islandStyle === "edge"}
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
    onpointerdowncapture={startSurfaceGesture}
    onpointermovecapture={moveSurfaceGesture}
    onpointercancelcapture={() => suppressBackgroundClick = true}
    onclick={toggleFromSurface}
    onkeydown={keyToggle}
  >
    {#if showDebugInfo}
      <div class="debug-overlay" aria-hidden="true">
        {#each debugLines as line}<span>{line}</span>{/each}
      </div>
    {/if}

    {#if visible && mode !== "expanded" && mode !== "hidden"}
    <div class="compact-layer" class:vertical={edge === "left" || edge === "right"} class:edge-inset={islandStyle === "edge"} style={`opacity:${compactOpacity};--shoulder-inset:${contentShoulderInset}px`} aria-hidden={expandedOpacity > .5} inert={mode === "expanded"}>
      {#if timerActive}
        <button
          class="compact-timer"
          class:paused={timerStatus === "paused"}
          class:urgent={timerUrgent}
          class:finished={timerFinished}
          type="button"
          aria-label={timerFinished ? timerFinishedText : `${timerLabel} ${timerText}`}
          onclick={(event) => {
            event.stopPropagation();
            page = "timer";
            if (mode !== "expanded") onToggle?.();
          }}
        >
          <span class="timer-summary">
            <span class="timer-leading"><Timer size={14} strokeWidth={1.8} />
              {#if !timerFinished && timerStatus === "paused"}
                <i class="timer-pause" aria-hidden="true"><b></b><b></b></i>
              {/if}
            </span>
            <strong class="timer-time"><RollingNumber value={timerFinished ? timerFinishedText : timerText} reduceMotion={reduceAnimations || !enableAnimations} /></strong>
          </span>
          <span class="timer-progress-track" aria-hidden="true">
            <span class="timer-progress-fill" class:paused={timerStatus === "paused"} class:finished={timerFinished} style:width={`${timerProgress * 100}%`} style:height={edge === "left" || edge === "right" ? `${timerProgress * 100}%` : "100%"}></span>
          </span>
        </button>
      {:else if showTime}
        <span class="time-display"><RollingNumber value={timeText} reduceMotion={reduceAnimations || !enableAnimations} /></span>
      {:else if idle}
        <div class="idle-compact-dashboard" class:vertical={edge === "left" || edge === "right"}>
          <span class="idle-clock"><RollingNumber value={idleTime} reduceMotion={reduceAnimations || !enableAnimations} /></span>
          <span class="idle-weather"><WeatherIcon code={idleWeatherCode} size={14} /><strong><RollingNumber value={idleWeatherTemperature === null ? "--°" : `${Math.round(idleWeatherTemperature)}°`} reduceMotion={reduceAnimations || !enableAnimations} /></strong></span>
        </div>
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

    {/if}
    {#if visible && mode !== "hidden" && (mode === "expanded" || (!shapeSettled && expandedOpacity > .05))}
    <div
      class="expanded-layer"
      style={`opacity:${expandedOpacity};transform:translateY(${(1 - expandedOpacity) * -6}px);--secondary-opacity:${secondaryOpacity}`}
      aria-hidden={expandedOpacity <= .5}
      inert={mode !== "expanded"}
    >
      <div class="expanded-shell" bind:this={pageRoot} class:reduce-page-motion={reduceAnimations || !enableAnimations || prefersReducedMotion} style={`--page-width:${expandedGeometry.width}px;--page-height:${expandedGeometry.height}px;--shoulder-content-inset:${contentShoulderInset}px`}>
        {#if panelVisible}
          <div class="top-tools"><FeatureMenu items={menuItems} toolbar activeId={page} scrollPosition={menuScroll} onScroll={(value) => menuScroll = value} onSelect={selectTool} /></div>
        {/if}
        <div class="page-body" class:exiting={pageExiting} class:with-toolbar={panelVisible} inert={pageExiting}>
        {#if renderedPage === "music"}
        <section class="music-pane">
          {#if !idle}
            <div class="top-row">
            <button class="cover expanded-cover" type="button" aria-label={t("openPlayer")} onclick={(e) => { e.stopPropagation(); onOpenPlayer?.(); }}>
              {#if media.albumArt}{#key media.albumArt}<img class="cover-image" src={media.albumArt} alt="" draggable="false" />{/key}{:else}<Music2 size={30} />{/if}
            </button>
            <div class="metadata">
              <MarqueeTitle text={media.title || t("waitingPlayback")} reduceMotion={reduceAnimations || !enableAnimations || prefersReducedMotion} />
              <span title={media.artist}>{media.artist || t("unknownArtist")}</span>
            </div>
            {#if showSpectrum}
              <Spectrum active={expandedOpacity > .05} playing={media.isPlaying} mode={spectrumMode} reduceMotion={reduceAnimations} topColor={spectrumTopColor} bottomColor={spectrumBottomColor} scale={1.5} values={previewSpectrum} />
            {/if}
            </div>

            <div class="progress-block">
              <MediaProgress position={position} duration={media.durationMs} seekable={Boolean(media.capabilities?.seek)} reduceMotion={reduceAnimations || !enableAnimations} {onSeek} />
            </div>

            <div class="control-row">
              <span class="control-spacer"></span>
              <div class="controls">
                <button type="button" class="side" aria-label={t("previous")} onclick={(e) => { e.stopPropagation(); onMediaAction?.("prev"); }}><SkipBack size={20} fill="currentColor" /></button>
                <button type="button" class="play" aria-label={media.isPlaying ? t("pause") : t("play")} onclick={(e) => { e.stopPropagation(); onMediaAction?.("play_pause"); }}>
                  <PlayPauseIcon playing={media.isPlaying} size={20} reduceMotion={reduceAnimations || !enableAnimations} />
                </button>
                <button type="button" class="side" aria-label={t("next")} onclick={(e) => { e.stopPropagation(); onMediaAction?.("next"); }}><SkipForward size={20} fill="currentColor" /></button>
              </div>
              <span class="control-spacer"></span>
            </div>
          {:else}
            <div class="idle-player" role="group" aria-label={t("mediaPlayer")}>
              <div class="top-row">
                <div class="cover expanded-cover idle-cover" aria-hidden="true">
                  {#if lastPlayedMedia?.albumArt}<img class="cover-image" src={lastPlayedMedia.albumArt} alt="" draggable="false" />{:else}<Music2 size={27} />{/if}
                </div>
                <div class="metadata">
                  <MarqueeTitle text={lastPlayedMedia?.title || t("mediaPlayer")} reduceMotion={reduceAnimations || !enableAnimations || prefersReducedMotion} />
                  <span title={lastPlayedMedia?.artist}>{lastPlayedMedia?.artist || lastPlayedMedia?.sourceDisplay || t("waitingPlayback")}</span>
                </div>
              </div>
              <div class="idle-progress" aria-hidden="true"><span></span></div>
              <div class="control-row idle-controls" class:has-remembered-track={Boolean(lastPlayedMedia)} role="group" aria-label={t("playbackControls")}>
                <span class="control-spacer"></span>
                <div class="controls">
                  <button type="button" class="side" aria-label={t("previous")} disabled><SkipBack size={22} fill="currentColor" /></button>
                  <button
                    type="button"
                    class="play"
                    aria-label={resumingLastTrack ? t("openingPlayer") : t("resumeMusic")}
                    title={resumingLastTrack ? t("openingPlayer") : t("resumeMusic")}
                    disabled={!lastPlayedMedia || !canResumeLastTrack || resumingLastTrack}
                    onclick={(event) => { event.stopPropagation(); onResumeLastTrack?.(); }}
                  ><Play size={32} fill="currentColor" /></button>
                  <button type="button" class="side" aria-label={t("next")} disabled><SkipForward size={22} fill="currentColor" /></button>
                </div>
                <span class="control-spacer"></span>
              </div>
            </div>
          {/if}
        </section>

        {:else}
          <section class="navigation-page">
            <header class="page-header">
              <button type="button" class="page-back" aria-label={t("back")} onclick={goBack}><ArrowLeft size={17} /></button>
              <span>{t(toolLabels[renderedPage])}</span>
            </header>
            {#key renderedPage}
              <div class="page-content">
                {#if renderedPage === "timer"}
                  <TimerPanel status={timerStatus} remainingMs={timerRemainingMs} finished={timerFinished} reduceMotion={reduceAnimations || !enableAnimations} onFinishedDismiss={onTimerFinishedDismiss} onStart={onTimerStart} onPause={onTimerPause} onResume={onTimerResume} onReset={onTimerReset} />
                {:else if renderedPage === "volume"}
                  <VolumePanel volume={systemAudio?.volumePercent ?? 0} muted={systemAudio?.muted ?? false} deviceId={systemAudio?.deviceId ?? ""} devices={audioDevices} switchingDevice={audioDeviceSwitching} reduceMotion={reduceAnimations || !enableAnimations} onVolume={onAudioVolume} onDevice={onAudioDevice} />
                {:else if renderedPage === "clock" || renderedPage === "weather"}
                  <InfoPanel page={renderedPage} time={clockText} timeZone={clockTimeZone} city={weatherCity} temperature={idleWeatherTemperature} code={idleWeatherCode} forecast={idleWeatherForecast} updatedAt={weatherUpdatedAt} loading={weatherLoading} failed={weatherFailed} reduceMotion={reduceAnimations || !enableAnimations || prefersReducedMotion} onSettings={onSettingsToggle} />
                {/if}
              </div>
            {/key}
          </section>
        {/if}
        </div>
      </div>
    </div>
    {/if}

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
        style={`--preview-radius-tl:${Math.max(8, Math.min(45, currentRadii.topLeft))}px;--preview-radius-tr:${Math.max(8, Math.min(45, currentRadii.topRight))}px;--preview-radius-br:${Math.max(8, Math.min(45, currentRadii.bottomRight))}px;--preview-radius-bl:${Math.max(8, Math.min(45, currentRadii.bottomLeft))}px;--preview-shoulder:${shoulderMarkerOffset}px`}
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

  </div>
</div>

<style>
  .island-frame{position:relative;width:100%;height:100%;flex:none;overflow:visible;box-sizing:border-box}.island-frame.simulate-hidden{overflow:hidden}.surface-anchor{position:absolute;z-index:1}.edge-top .surface-anchor{top:0;left:50%}.edge-right .surface-anchor{top:50%;right:0}.edge-bottom .surface-anchor{bottom:0;left:50%}.edge-left .surface-anchor{top:50%;left:0}
  .island-surface{position:relative;z-index:1;overflow:hidden;flex:none;box-sizing:border-box;color:#fff;font-family:var(--app-font);text-rendering:optimizeLegibility;font-synthesis:none;contain:layout style;transition:transform 140ms cubic-bezier(.23,1,.32,1),box-shadow 180ms cubic-bezier(.23,1,.32,1)}
  .floating-style .island-surface:active{transform:scale(.97) translateZ(0)}.compact-layer,.expanded-layer{position:absolute;z-index:1;box-sizing:border-box;pointer-events:none}
  .compact-layer{inset:0;display:flex;align-items:center;justify-content:space-between;padding:0 8px 0 4px}.compact-layer.edge-inset:not(.vertical){padding-left:calc(4px + var(--shoulder-inset));padding-right:calc(8px + var(--shoulder-inset))}.compact-layer.vertical{flex-direction:column;padding:4px 0 8px}.compact-layer.edge-inset.vertical{padding-top:calc(4px + var(--shoulder-inset));padding-bottom:calc(8px + var(--shoulder-inset))}.compact-layer button,.compact-layer :global(canvas){pointer-events:auto}.time-display{width:100%;text-align:center;color:rgba(255,255,255,.8);font:500 12px/1 var(--app-font);letter-spacing:.05em;font-variant-numeric:tabular-nums;user-select:none}.cover{display:grid;place-items:center;flex:none;padding:0;overflow:hidden;color:rgba(255,255,255,.3);background:rgba(255,255,255,.06);border:0;cursor:pointer;user-select:none}.cover img{width:100%;height:100%;display:block;object-fit:cover;-webkit-user-drag:none;user-select:none}.compact-cover{width:20px;height:20px;border-radius:50%}.expanded-cover{width:52px;height:52px;border-radius:12px;box-shadow:0 8px 22px rgba(0,0,0,.35);outline:1px solid rgba(255,255,255,.1)}.playing-dot{width:3px;height:3px;border-radius:50%}
  .compact-disc{display:block;width:100%;height:100%;transform-origin:center;animation:compact-disc-spin 8s linear infinite;animation-play-state:paused}.compact-disc.spinning{animation-play-state:running;will-change:transform}@keyframes compact-disc-spin{to{transform:rotate(1turn)}}
  .idle-compact-dashboard{display:flex;align-items:center;justify-content:space-between;gap:8px;width:100%;padding:0 7px;color:rgba(255,255,255,.92);user-select:none}.idle-clock{font:600 12px/1 var(--app-font);font-variant-numeric:tabular-nums;letter-spacing:.02em}.idle-weather{display:flex;align-items:center;gap:4px;font:500 11px/1 var(--app-font);font-variant-numeric:tabular-nums}.idle-weather :global(svg){flex:none;color:rgba(255,255,255,.82)}.idle-compact-dashboard.vertical{flex-direction:column;padding:7px 0;gap:10px}.idle-compact-dashboard.vertical .idle-weather{flex-direction:column}
  .cover-image{animation:cover-flip-in 420ms cubic-bezier(.23,1,.32,1)}@keyframes cover-flip-in{from{opacity:0;transform:perspective(500px) rotateY(-70deg) scale(.9)}to{opacity:1;transform:perspective(500px) rotateY(0) scale(1)}}
  .debug-overlay{position:absolute;z-index:4;top:4px;left:50%;display:flex;gap:5px;max-width:calc(100% - 12px);padding:2px 6px;border-radius:5px;transform:translateX(-50%);overflow:hidden;color:#4ade80;background:rgba(0,0,0,.75);font:500 8px/1.3 var(--app-font);white-space:nowrap;pointer-events:none}.debug-overlay span{overflow:hidden;text-overflow:ellipsis}
  .expanded-layer{inset:0;width:100%;height:100%;padding:0;transition:opacity 120ms linear}.expanded-layer[aria-hidden="true"]{pointer-events:none}.expanded-layer[aria-hidden="false"]{pointer-events:auto}.expanded-shell{display:grid;width:100%;height:var(--expanded-content-height,160px);grid-template-columns:var(--music-pane-width,300px)}.music-pane{position:relative;width:var(--music-pane-width,300px);height:var(--expanded-content-height,160px);display:flex;flex-direction:column;padding:16px 28px 20px;box-sizing:border-box}.top-row{display:flex;align-items:center;gap:12px;margin-bottom:8px;min-height:52px}.metadata{min-width:0;flex:1;font-family:var(--app-font);user-select:none}.metadata span{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.metadata span{font-size:11px;line-height:1.2;font-weight:500;color:rgba(255,255,255,.68);letter-spacing:.005em}.progress-block,.control-row{opacity:var(--secondary-opacity);transition:opacity 100ms linear}
  .progress-block{width:100%;margin-bottom:6px}
  .control-row{position:relative;display:grid;grid-template-columns:minmax(0,1fr) auto minmax(0,1fr);align-items:center;width:100%;height:48px}.control-spacer{width:0}.controls{display:flex;align-items:center;justify-content:center;gap:8px}.controls button{flex:none;display:grid;place-items:center;width:48px;height:48px;padding:0;border:0;border-radius:12px;color:rgba(255,255,255,.9);background:transparent;cursor:pointer;transition:transform 140ms cubic-bezier(.23,1,.32,1),color 140ms ease,background 150ms ease}.controls .play{color:#fff}.controls button:active,.cover:active{transform:scale(.94)}.controls button:focus-visible:not(:disabled){background:rgba(255,255,255,.1)}button:focus-visible{outline:2px solid #fff;outline-offset:2px}
  .compact-timer{width:100%;height:100%;display:flex;align-items:center;justify-content:center;gap:7px;padding:0 9px;border:0;color:#f28b31;background:transparent;cursor:pointer;font-family:var(--app-font);user-select:none}.compact-timer span{min-width:42px;font-size:12px;font-weight:600;line-height:1;letter-spacing:.015em;font-variant-numeric:tabular-nums;text-align:center;white-space:nowrap}.compact-timer.finished span{min-width:0;font-size:11px}.compact-timer.paused{color:rgba(242,139,49,.62)}.timer-pause{width:7px;height:8px;display:flex;align-items:center;justify-content:center;gap:2px}.timer-pause b{display:block;width:2px;height:7px;border-radius:1px;background:currentColor}.compact-timer.urgent{color:#ff765f}
  .compact-timer{position:absolute;inset:0;display:block;overflow:hidden;padding:0;border-radius:inherit;color:#f4f7ff;background:transparent;box-shadow:inset 1px 0 0 rgba(82,151,255,.46),inset -1px 0 0 rgba(82,151,255,.46),inset 0 0 12px rgba(28,91,190,.08)}.compact-timer:hover{background:rgba(40,100,190,.055)}.compact-timer .timer-summary{position:absolute;inset:0 14px 4px;display:flex;align-items:center;justify-content:space-between;gap:12px;min-width:0}.compact-timer .timer-leading{display:flex;align-items:center;gap:7px;min-width:0;color:rgba(245,248,255,.9)}.compact-timer .timer-leading :global(svg){flex:none;color:#70aaff}.compact-timer .timer-time{min-width:0;overflow:hidden;color:#f5f7fb;font-size:13px;font-weight:600;line-height:1;font-variant-numeric:tabular-nums;letter-spacing:.01em;text-align:right;text-overflow:ellipsis;white-space:nowrap}.compact-timer .timer-progress-track{position:absolute;right:14px;bottom:5px;left:14px;height:2px;overflow:hidden;border-radius:999px;background:rgba(115,143,183,.2)}.compact-timer .timer-progress-fill{display:block;height:100%;max-width:100%;border-radius:inherit;background:linear-gradient(90deg,#2688ff,#60b5ff);box-shadow:0 0 7px rgba(57,145,255,.68);transition:width 350ms linear,background-color 180ms ease}.compact-timer .timer-progress-fill.paused{background:linear-gradient(90deg,#6d8fb9,#a7c5ed);box-shadow:none}.compact-timer .timer-progress-fill.finished{background:linear-gradient(90deg,#42d99a,#8af0c0)}.compact-timer.paused{color:#f4f7ff}.compact-timer.urgent{box-shadow:inset 1px 0 0 rgba(255,111,102,.58),inset -1px 0 0 rgba(255,111,102,.58),inset 0 0 12px rgba(255,74,60,.09)}.compact-timer.urgent .timer-leading :global(svg){color:#ff8c7c}.compact-timer .timer-pause{flex:none;width:7px;height:8px;gap:2px;color:#a8c5e9}.compact-timer .timer-pause b{width:2px;height:7px;border-radius:1px;background:currentColor}
  .compact-layer.vertical .compact-timer .timer-summary{inset:14px 4px;flex-direction:column;justify-content:center;gap:10px}.compact-layer.vertical .compact-timer .timer-leading{flex-direction:column;gap:5px}.compact-layer.vertical .compact-timer .timer-time{max-width:100%;font-size:11px;text-align:center}.compact-layer.vertical .compact-timer .timer-progress-track{top:14px;right:auto;bottom:14px;left:5px;width:2px;height:auto}.compact-layer.vertical .timer-progress-fill{width:100%!important;height:var(--timer-progress-height,0%)}
  .idle-player{display:flex;flex:1;min-height:0;flex-direction:column}.idle-cover{cursor:default;color:rgba(255,255,255,.58);background:linear-gradient(145deg,rgba(255,255,255,.12),rgba(255,255,255,.035));box-shadow:none}.idle-progress{height:3px;margin:0 0 7px;overflow:hidden;border-radius:999px;background:rgba(255,255,255,.13)}.idle-progress span{display:block;width:100%;height:100%;border-radius:inherit;background:linear-gradient(90deg,rgba(255,255,255,.38),rgba(255,255,255,.16))}.idle-controls{opacity:.36!important}.idle-controls.has-remembered-track{opacity:1!important}.idle-controls button:disabled{cursor:default}.idle-controls button:disabled:active{transform:none}
  .edge-top .music-pane,.edge-bottom .music-pane{padding-left:calc(28px + var(--shoulder-content-inset,0px));padding-right:calc(28px + var(--shoulder-content-inset,0px))}
  .music-pane{padding-top:18px;padding-bottom:15px}
  .top-row{gap:10px;margin-bottom:10px}
  @media (hover:hover) and (pointer:fine){}
  @media (prefers-reduced-motion:reduce){}
  .studio-preview-highlight{position:absolute;z-index:20;inset:0;overflow:hidden;border-radius:inherit;pointer-events:none}.studio-preview-highlight::before,.studio-preview-highlight::after{content:"";position:absolute;pointer-events:none}.highlight-shape::after{inset:2px;border:2px solid rgba(167,139,250,.9);border-radius:inherit;box-shadow:0 0 12px rgba(167,139,250,.42),inset 0 0 7px rgba(167,139,250,.18)}.highlight-edge::before{background:#60a5fa;box-shadow:0 0 9px rgba(96,165,250,.8)}.edge-top .highlight-edge::before,.edge-bottom .highlight-edge::before{left:18px;right:18px;height:2px}.edge-top .highlight-edge::before{top:2px}.edge-bottom .highlight-edge::before{bottom:2px}.edge-left .highlight-edge::before,.edge-right .highlight-edge::before{top:18px;bottom:18px;width:2px}.edge-left .highlight-edge::before{left:2px}.edge-right .highlight-edge::before{right:2px}.highlight-position::after{width:8px;height:8px;border:2px solid #fff;border-radius:50%;background:#60a5fa;box-shadow:0 0 0 3px rgba(96,165,250,.2),0 0 10px rgba(96,165,250,.92)}.edge-top .highlight-position::after,.edge-bottom .highlight-position::after{left:calc(var(--preview-position) * 1% - 6px)}.edge-top .highlight-position::after{top:-3px}.edge-bottom .highlight-position::after{bottom:-3px}.edge-left .highlight-position::after,.edge-right .highlight-position::after{top:calc(var(--preview-position) * 1% - 6px)}.edge-left .highlight-position::after{left:-3px}.edge-right .highlight-position::after{right:-3px}.corner{position:absolute;width:calc(var(--preview-radius-tl) + 9px);height:calc(var(--preview-radius-tl) + 9px);max-width:48px;max-height:48px;opacity:.95}.corner.top-left{top:2px;left:2px;width:calc(var(--preview-radius-tl) + 9px);height:calc(var(--preview-radius-tl) + 9px);border:2px solid;border-color:#a78bfa transparent transparent #a78bfa;border-radius:var(--preview-radius-tl) 0 0 0}.corner.top-right{top:2px;right:2px;width:calc(var(--preview-radius-tr) + 9px);height:calc(var(--preview-radius-tr) + 9px);border:2px solid;border-color:#a78bfa #a78bfa transparent transparent;border-radius:0 var(--preview-radius-tr) 0 0}.corner.bottom-left{bottom:2px;left:2px;width:calc(var(--preview-radius-bl) + 9px);height:calc(var(--preview-radius-bl) + 9px);border:2px solid;border-color:transparent transparent #a78bfa #a78bfa;border-radius:0 0 0 var(--preview-radius-bl)}.corner.bottom-right{right:2px;bottom:2px;width:calc(var(--preview-radius-br) + 9px);height:calc(var(--preview-radius-br) + 9px);border:2px solid;border-color:transparent #a78bfa #a78bfa transparent;border-radius:0 0 var(--preview-radius-br) 0}.length-handle{position:absolute;width:3px;height:22px;border-radius:999px;background:#60a5fa;box-shadow:0 0 9px rgba(96,165,250,.8)}.length-handle.start{left:2px;top:50%;transform:translateY(-50%)}.length-handle.end{right:2px;top:50%;transform:translateY(-50%)}.length-vertical .length-handle{width:22px;height:3px;left:50%;transform:translateX(-50%)}.length-vertical .length-handle.start{top:2px}.length-vertical .length-handle.end{top:auto;right:auto;bottom:2px}.shoulder-handle{position:absolute;width:3px;height:18px;border-radius:999px;background:#a78bfa;box-shadow:0 0 8px rgba(167,139,250,.8)}.edge-top .shoulder-handle.start{top:2px;left:calc(var(--preview-shoulder) + 1px)}.edge-top .shoulder-handle.end{top:2px;right:calc(var(--preview-shoulder) + 1px)}.edge-bottom .shoulder-handle.start{bottom:2px;left:calc(var(--preview-shoulder) + 1px)}.edge-bottom .shoulder-handle.end{right:calc(var(--preview-shoulder) + 1px);bottom:2px}.edge-left .shoulder-handle,.edge-right .shoulder-handle{width:18px;height:3px;top:calc(var(--preview-shoulder) + 1px)}.edge-left .shoulder-handle.start{left:2px}.edge-left .shoulder-handle.end{bottom:calc(var(--preview-shoulder) + 1px);top:auto;left:2px}.edge-right .shoulder-handle.start{right:2px}.edge-right .shoulder-handle.end{right:2px;top:auto;bottom:calc(var(--preview-shoulder) + 1px)}.background-wash{position:absolute;inset:2px;border-radius:inherit;background:rgba(167,139,250,.14);box-shadow:inset 0 0 0 1px rgba(196,181,253,.72),0 0 16px rgba(167,139,250,.22)}.spectrum-focus{position:absolute;z-index:1;top:50%;right:5px;width:23px;height:21px;border:1px solid #60a5fa;border-radius:6px;background:rgba(96,165,250,.1);box-shadow:0 0 10px rgba(96,165,250,.55);transform:translateY(-50%)}.preview-expanded .spectrum-focus{top:14px;right:22px;width:47px;height:31px;border-radius:8px}
  @media (hover:hover) and (pointer:fine){.controls button:hover:not(:disabled){background:rgba(255,255,255,.1);transform:scale(1.06);color:#fff}.cover:hover{filter:brightness(1.08)}}
  @media (prefers-reduced-motion:reduce){.surface-anchor{transition:none!important}.island-surface,.expanded-layer,.controls button{transition-duration:120ms!important}.island-surface{transition-property:opacity,box-shadow!important}.expanded-layer{transform:none!important}.cover-image,.compact-disc{animation:none}.studio-preview-highlight .corner,.studio-preview-highlight::after,.studio-preview-highlight .length-handle,.studio-preview-highlight .shoulder-handle,.studio-preview-highlight .background-wash,.studio-preview-highlight .spectrum-focus{animation:none!important;opacity:.78}}

  .expanded-shell{display:block;width:var(--page-width);height:var(--page-height)}
  .music-pane{width:100%;height:100%;animation:page-enter 180ms ease both}






  .attached-style.edge-left .music-pane,.attached-style.edge-right .music-pane{padding-top:calc(16px + var(--shoulder-content-inset,0px));padding-bottom:calc(20px + var(--shoulder-content-inset,0px))}

  .navigation-page{display:flex;flex-direction:column;width:100%;height:100%;padding:16px 24px 24px;box-sizing:border-box}
  .attached-style.edge-top .navigation-page,.attached-style.edge-bottom .navigation-page{padding-left:calc(24px + var(--shoulder-content-inset,0px));padding-right:calc(24px + var(--shoulder-content-inset,0px))}
  .attached-style.edge-left .navigation-page,.attached-style.edge-right .navigation-page{padding-top:calc(16px + var(--shoulder-content-inset,0px));padding-bottom:calc(24px + var(--shoulder-content-inset,0px))}
  .page-header{display:flex;align-items:center;gap:9px;min-width:0;height:28px;flex:none;margin-bottom:12px;color:rgba(235,244,255,.78);font:500 11px/1 var(--app-font,system-ui)}
  .page-header span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .page-back{display:grid;place-items:center;width:28px;height:28px;flex:none;padding:0;border:0;border-radius:8px;color:#91caff;background:rgba(65,145,235,.14);cursor:pointer}
  .page-back:hover{background:rgba(65,145,235,.25)}

  .page-content{display:flex;flex:1;min-height:0;align-items:center;animation:page-enter 180ms ease both}
  .page-content>:global(*){width:100%}
  @keyframes page-enter{from{opacity:0;transform:translateY(4px)}to{opacity:1;transform:translateY(0)}}
  .reduce-page-motion .page-content,.reduce-page-motion .music-pane{animation:none}

  .expanded-shell{display:flex;flex-direction:column}
  .top-tools{flex:none;height:40px;box-sizing:border-box;padding:6px 28px;display:flex;align-items:center}
  .attached-style.edge-top .top-tools,.attached-style.edge-bottom .top-tools{padding-left:calc(28px + var(--shoulder-content-inset,0px));padding-right:calc(28px + var(--shoulder-content-inset,0px))}
  .attached-style.edge-left .top-tools,.attached-style.edge-right .top-tools{height:calc(40px + var(--shoulder-content-inset,0px));padding-top:calc(6px + var(--shoulder-content-inset,0px))}
  .page-body{flex:1;min-height:0;opacity:1;transition:opacity 100ms ease}.page-body.exiting{opacity:0;pointer-events:none}
  .attached-style.edge-left .page-body.with-toolbar .music-pane,.attached-style.edge-right .page-body.with-toolbar .music-pane{padding-top:16px}
  .attached-style.edge-left .page-body.with-toolbar .navigation-page,.attached-style.edge-right .page-body.with-toolbar .navigation-page{padding-top:16px}
  .reduce-page-motion .page-body{transition:none}
</style>
