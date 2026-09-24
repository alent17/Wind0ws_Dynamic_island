<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { spring, tweened } from "svelte/motion";
  import { EyeOff, GalleryHorizontalEnd, Music2, Play, Settings, SkipBack, SkipForward, Timer, Volume2, VolumeX } from "lucide-svelte";
  import ComicClock from "$lib/ComicClock.svelte";
  import MediaProgress from "$lib/MediaProgress.svelte";
  import PlayPauseIcon from "$lib/PlayPauseIcon.svelte";
  import RollingNumber from "$lib/RollingNumber.svelte";
  import Spectrum from "$lib/Spectrum.svelte";
  import TimerPanel from "$lib/TimerPanel.svelte";
  import VolumePanel from "$lib/VolumePanel.svelte";
  import WeatherIcon from "$lib/WeatherIcon.svelte";
  import type { IslandTool } from "$lib/featureRail";
  import { formatCountdown, type CountdownStatus } from "$lib/countdown";
  import { ISLAND_MOTION, islandMorphEasing, islandSettleEasing } from "$lib/islandMotion";
  import type { AudioDeviceInfo, MediaState, SpectrumMode, SystemAudioState, WeatherForecastDay } from "$lib/api/types";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import {
    CUSTOM_PANEL_WIDTH,
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
  const clockLocale = $derived($locale === "zh-CN" ? "zh-CN" : $locale === "ja" ? "ja-JP" : "en-US");
  const idleForecast = $derived.by(() => {
    const localeId = clockLocale;
    return idleWeatherForecast.slice(1, 4).map((day: WeatherForecastDay) => {
      const parsedDate = new Date(`${day.date}T12:00:00`);
      const label = Number.isNaN(parsedDate.getTime())
        ? day.date.slice(5)
        : new Intl.DateTimeFormat(localeId, { weekday: "short" }).format(parsedDate);
      return { ...day, label };
    });
  });
  const clockDate = $derived.by(() => {
    void clockText;
    const options: Intl.DateTimeFormatOptions = {
      weekday: "long",
      month: "short",
      day: "numeric",
    };

    if (clockTimeZone && clockTimeZone !== "system") options.timeZone = clockTimeZone;
    return new Intl.DateTimeFormat(clockLocale, options).format(new Date());
  });
  const greeting = $derived.by(() => {
    void clockText;
    const options: Intl.DateTimeFormatOptions = { hour: "numeric", hour12: false };
    if (clockTimeZone && clockTimeZone !== "system") options.timeZone = clockTimeZone;

    const hour = Number(new Intl.DateTimeFormat("en-US", options).format(new Date()));
    if (hour < 5) return "good night";
    if (hour < 12) return "good morning";
    if (hour < 18) return "good afternoon";
    return "good evening";
  });

  const initial = untrack(() => geometryFor(
    mode,
    expandedRadius,
    edge,
    effectiveCompactLength,
    (showCustomFunctionPanel && enabledTools.length > 0 ? CUSTOM_PANEL_WIDTH : 0),
  ));
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
  let wasTimerFinished = $state(false);
  const panelVisible = $derived(showCustomFunctionPanel && enabledTools.length > 0);
  const panelExtraWidth = $derived(panelVisible ? CUSTOM_PANEL_WIDTH : 0);
  const musicPaneWidth = 300;
  const expandedExtraWidth = $derived(panelExtraWidth);

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

  function notifyRegionChange(change: IslandRegionChange) {
    // The App callback reads media state while calculating the host offset.
    // Do not let those reads make this geometry effect restart on every media
    // clock update while playback is active.
    untrack(() => onRegionChange?.(change));
  }

  $effect(() => {
    const target = geometryFor(mode, expandedRadius, edge, effectiveCompactLength, expandedExtraWidth);
    const hard = !enableAnimations || reduceAnimations || prefersReducedMotion;
    const revision = ++transitionRevision;
    activeEnvelope = stableEnvelope(activeEnvelope, target);
    notifyRegionChange({ geometry: activeEnvelope, radii: radiiFor(activeEnvelope, islandStyle, edge), polygon: shapePolygonFor(activeEnvelope, islandStyle, edge, mode === "expanded" ? expandedEdgeShoulderRadius : collapsedEdgeShoulderRadius), extraRects: [], settled: false });
    animateGeometry(target, hard, revision).then(() => {
      if (revision !== transitionRevision) return;
      activeEnvelope = target;
      notifyRegionChange({ geometry: target, radii: radiiFor(target, islandStyle, edge), polygon: shapePolygonFor(target, islandStyle, edge, mode === "expanded" ? expandedEdgeShoulderRadius : collapsedEdgeShoulderRadius), extraRects: [], settled: true });
    });
  });
  $effect(() => {
    if (!panelVisible || (mode !== "expanded" && activeTool === "volume") || (activeTool !== null && !enabledTools.includes(activeTool))) activeTool = null;
  });
  $effect(() => {
    const finished = timerFinished;
    if (finished && panelVisible && enabledTools.includes("timer")) activeTool = "timer";
    else if (!finished && wasTimerFinished && activeTool === "timer") activeTool = null;
    wasTimerFinished = finished;
  });
  $effect(() => {
    const panel = panelVisible;
    void panel;
    onPanelActivity?.(activeTool === "timer" || activeTool === "volume");
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
    const expanded = geometryFor("expanded", expandedRadius, edge, effectiveCompactLength, expandedExtraWidth);
    const value = horizontalEdge ? size.height : size.width;
    const start = horizontalEdge ? compact.height : compact.width;
    const end = horizontalEdge ? expanded.height : expanded.width;
    return Math.min(1, Math.max(0, (value - start) / (end - start)));
  });
  const currentShoulderRadius = $derived(
    collapsedEdgeShoulderRadius + (expandedEdgeShoulderRadius - collapsedEdgeShoulderRadius) * outwardProgress,
  );
  const shoulderMarkerOffset = $derived.by(() => {
    const verticalEdge = edge === "left" || edge === "right";
    const canonicalWidth = verticalEdge ? size.height : size.width;
    const canonicalHeight = verticalEdge ? size.width : size.height;
    return Math.min(currentShoulderRadius, canonicalWidth / 4, canonicalHeight / 2);
  });
  const contentShoulderInset = $derived(shoulderMarkerOffset * styleProgress);
  const currentRadii = $derived(radiiFor(size, islandStyle, edge));
  const currentPolygon = $derived(interpolatePolygon(
    shapePolygonFor(size, "floating", edge, 0),
    shapePolygonFor(size, "edge", edge, currentShoulderRadius),
    styleProgress,
  ));
  const surfaceClipPath = $derived(polygonCss(currentPolygon));
  const resolvedClipPath = $derived(islandStyle === "edge" ? surfaceClipPath : "none");
  const expandedOpacity = $derived(Math.min(1, Math.max(0, (outwardProgress - 0.18) / 0.42)));
  const secondaryOpacity = $derived(Math.min(1, Math.max(0, (outwardProgress - 0.42) / 0.38)));
  const compactOpacity = $derived(Math.min(1, Math.max(0, 1 - outwardProgress * 4)));
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
    onclick={toggleFromSurface}
    onkeydown={keyToggle}
  >
    {#if showDebugInfo}
      <div class="debug-overlay" aria-hidden="true">
        {#each debugLines as line}<span>{line}</span>{/each}
      </div>
    {/if}

    <div class="compact-layer" class:vertical={edge === "left" || edge === "right"} class:edge-inset={islandStyle === "edge"} style={`opacity:${compactOpacity};--shoulder-inset:${contentShoulderInset}px`} aria-hidden={expandedOpacity > .5}>
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
            activeTool = "timer";
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

    <div
      class="expanded-layer"
      style={`opacity:${expandedOpacity};transform:translateY(${(1 - expandedOpacity) * -6}px);--secondary-opacity:${secondaryOpacity}`}
      aria-hidden={expandedOpacity <= .5}
    >
      <div class="expanded-shell" class:with-panel={panelVisible} style={`--music-pane-width:${musicPaneWidth}px;--function-pane-width:${CUSTOM_PANEL_WIDTH}px;--shoulder-content-inset:${contentShoulderInset}px`}>
        <section class="music-pane">
          {#if !idle}
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
                  <strong title={lastPlayedMedia?.title}>{lastPlayedMedia?.title || t("mediaPlayer")}</strong>
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

        {#if panelVisible}
          <section class="function-pane">
            <div class="function-toolbar" role="toolbar" aria-label={t("featureTools")}>
              {#if enabledTools.includes("timer")}
                <button
                  class="function-icon"
                  class:active={activeTool === "timer"}
                  class:timer-running={timerStatus === "running"}
                  class:timer-paused={timerStatus === "paused"}
                  type="button"
                  aria-label={t("timerTool")}
                  aria-pressed={activeTool === "timer"}
                  onclick={(event) => { event.stopPropagation(); activeTool = activeTool === "timer" ? null : "timer"; }}
                >
                  <Timer size={16} />
                </button>
              {/if}
              {#if enabledTools.includes("volume")}
                <button
                  class="function-icon"
                  class:active={activeTool === "volume"}
                  type="button"
                  aria-label={t("volume")}
                  aria-pressed={activeTool === "volume"}
                  onclick={(event) => { event.stopPropagation(); const opening = activeTool !== "volume"; activeTool = opening ? "volume" : null; if (opening) void onAudioOpen?.(); }}
                >
                  {#if systemAudio?.muted || systemAudio?.volumePercent === 0}<VolumeX size={16} />{:else}<Volume2 size={16} />{/if}
                </button>
              {/if}
              {#if enabledTools.includes("floating")}
                <button
                  class="function-icon"
                  class:active={activeTool === "floating"}
                  type="button"
                  aria-label={t("floatingTool")}
                  aria-pressed={activeTool === "floating"}
                  onclick={(event) => { event.stopPropagation(); activeTool = activeTool === "floating" ? null : "floating"; onToggleFloating?.(); }}
                >
                  <GalleryHorizontalEnd size={16} />
                </button>
              {/if}
              {#if enabledTools.includes("settings")}
                <button
                  class="function-icon settings-icon"
                  type="button"
                  aria-label={t("settingsTool")}
                  onclick={(event) => { event.stopPropagation(); onSettingsToggle?.(); }}
                >
                  <Settings size={16} />
                </button>
              {/if}
              {#if enabledTools.includes("hide")}
                <button
                  class="function-icon"
                  type="button"
                  aria-label={t("hideTool")}
                  onclick={(event) => { event.stopPropagation(); onHideForTenSeconds?.(); }}
                >
                  <EyeOff size={16} />
                </button>
              {/if}
            </div>

            <div class="function-content">
              {#if activeTool === "timer"}
                <TimerPanel status={timerStatus} remainingMs={timerRemainingMs} finished={timerFinished} reduceMotion={reduceAnimations || !enableAnimations} onFinishedDismiss={onTimerFinishedDismiss} onStart={onTimerStart} onPause={onTimerPause} onResume={onTimerResume} onReset={onTimerReset} />
              {:else if activeTool === "volume"}
                <VolumePanel volume={systemAudio?.volumePercent ?? 0} muted={systemAudio?.muted ?? false} deviceId={systemAudio?.deviceId ?? ""} devices={audioDevices} switchingDevice={audioDeviceSwitching} reduceMotion={reduceAnimations || !enableAnimations} onVolume={onAudioVolume} onDevice={onAudioDevice} />
              {:else}
                <div class="clock-dashboard" class:with-forecast={idleForecast.length > 0}>
                  <ComicClock time={clockText} date={clockDate} {greeting} compact={idleForecast.length > 0} reduceMotion={reduceAnimations || !enableAnimations} />
                  {#if idleForecast.length > 0}
                    <div class="forecast-strip" aria-label={t("weatherForecast")}>
                      {#each idleForecast as forecast}
                        <div class="forecast-day" title={forecast.date}>
                          <span class="forecast-day-name">{forecast.label}</span>
                          <WeatherIcon code={forecast.weatherCode} size={14} />
                          <strong><RollingNumber value={`${Math.round(forecast.temperatureMax)}°`} reduceMotion={reduceAnimations || !enableAnimations} /></strong>
                          <small><RollingNumber value={`${Math.round(forecast.temperatureMin)}°`} reduceMotion={reduceAnimations || !enableAnimations} /></small>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          </section>
        {/if}
      </div>
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
  .expanded-layer{inset:0;width:100%;height:100%;padding:0;transition:opacity 120ms linear}.expanded-layer[aria-hidden="true"]{pointer-events:none}.expanded-layer[aria-hidden="false"]{pointer-events:auto}.expanded-shell{display:grid;width:100%;height:var(--expanded-content-height,160px);grid-template-columns:var(--music-pane-width,300px)}.expanded-shell.with-panel{grid-template-columns:var(--music-pane-width,300px) var(--function-pane-width,300px)}.music-pane{position:relative;width:var(--music-pane-width,300px);height:var(--expanded-content-height,160px);display:flex;flex-direction:column;padding:16px 28px 20px;box-sizing:border-box}.function-pane{position:relative;width:var(--function-pane-width,300px);height:var(--expanded-content-height,160px);display:flex;flex-direction:column;padding:12px 16px 14px 14px;box-sizing:border-box;border-left:1px solid rgba(255,255,255,.075)}.function-toolbar{display:flex;flex:none;align-items:center;justify-content:flex-end;align-self:flex-end;height:34px;gap:2px;padding:2px;border:1px solid rgba(255,255,255,.12);border-radius:999px;background:linear-gradient(180deg,rgba(255,255,255,.075),rgba(255,255,255,.025)),rgba(12,12,14,.34);box-shadow:inset 0 1px 0 rgba(255,255,255,.08),0 3px 10px rgba(0,0,0,.12);backdrop-filter:blur(18px) saturate(140%);-webkit-backdrop-filter:blur(18px) saturate(140%)}.function-icon{display:grid;place-items:center;width:28px;height:28px;padding:0;border:0;border-radius:999px;color:rgba(255,255,255,.62);background:transparent;cursor:pointer;transition:color 150ms ease,background 150ms ease,transform 140ms cubic-bezier(.23,1,.32,1),box-shadow 150ms ease}.function-icon.active{color:#f59a23}.function-icon.timer-running:not(.active){color:#f59a23}.function-icon.timer-paused:not(.active){color:rgba(245,154,35,.7)}.function-icon:active{transform:scale(.92)}.function-icon:focus-visible{outline:2px solid rgba(255,255,255,.92);outline-offset:2px;background:rgba(255,255,255,.09)}.function-content{flex:1;min-height:0;display:flex;align-items:center;justify-content:center;padding-top:8px}.clock-dashboard{width:100%;height:100%;display:flex;flex-direction:column;align-items:flex-start;justify-content:center;gap:18px;padding:0 11px 5px 10px;box-sizing:border-box;color:#fff;user-select:none}.clock-dashboard.with-forecast{flex-direction:row;align-items:center;justify-content:space-between;gap:10px}.forecast-strip{display:flex;align-items:stretch;gap:7px;margin-left:auto;padding-left:10px;border-left:1px solid rgba(255,255,255,.1)}.forecast-day{display:grid;grid-template-rows:14px 22px 14px 11px;justify-items:center;align-items:center;min-width:33px;color:rgba(255,255,255,.52);font-variant-numeric:tabular-nums}.forecast-day-name{font-size:9px;line-height:1}.forecast-day :global(svg){width:17px;height:17px;color:rgba(255,255,255,.78)}.forecast-day strong{color:rgba(255,255,255,.9);font-size:12px;line-height:1}.forecast-day small{color:rgba(255,255,255,.4);font-size:9px;line-height:1}.top-row{display:flex;align-items:center;gap:12px;margin-bottom:8px;min-height:52px}.metadata{min-width:0;flex:1;font-family:var(--app-font);user-select:none}.metadata strong,.metadata span{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.metadata strong{font-size:13px;line-height:1.2;font-weight:700;letter-spacing:-.02em;margin-bottom:4px}.metadata span{font-size:11px;line-height:1.2;font-weight:500;color:rgba(255,255,255,.68);letter-spacing:.005em}.progress-block,.control-row{opacity:var(--secondary-opacity);transition:opacity 100ms linear}
  .progress-block{width:100%;margin-bottom:6px}
  .control-row{position:relative;display:grid;grid-template-columns:minmax(0,1fr) auto minmax(0,1fr);align-items:center;width:100%;height:48px}.control-spacer{width:0}.controls{display:flex;align-items:center;justify-content:center;gap:8px}.controls button{flex:none;display:grid;place-items:center;width:48px;height:48px;padding:0;border:0;border-radius:12px;color:rgba(255,255,255,.9);background:transparent;cursor:pointer;transition:transform 140ms cubic-bezier(.23,1,.32,1),color 140ms ease,background 150ms ease}.controls .play{color:#fff}.controls button:active,.cover:active{transform:scale(.94)}.controls button:focus-visible:not(:disabled){background:rgba(255,255,255,.1)}button:focus-visible{outline:2px solid #fff;outline-offset:2px}
  .compact-timer{width:100%;height:100%;display:flex;align-items:center;justify-content:center;gap:7px;padding:0 9px;border:0;color:#f28b31;background:transparent;cursor:pointer;font-family:var(--app-font);user-select:none}.compact-timer span{min-width:42px;font-size:12px;font-weight:600;line-height:1;letter-spacing:.015em;font-variant-numeric:tabular-nums;text-align:center;white-space:nowrap}.compact-timer.finished span{min-width:0;font-size:11px}.compact-timer.paused{color:rgba(242,139,49,.62)}.timer-pause{width:7px;height:8px;display:flex;align-items:center;justify-content:center;gap:2px}.timer-pause b{display:block;width:2px;height:7px;border-radius:1px;background:currentColor}.compact-timer.urgent{color:#ff765f}
  .compact-timer{position:absolute;inset:0;display:block;overflow:hidden;padding:0;border-radius:inherit;color:#f4f7ff;background:transparent;box-shadow:inset 1px 0 0 rgba(82,151,255,.46),inset -1px 0 0 rgba(82,151,255,.46),inset 0 0 12px rgba(28,91,190,.08)}.compact-timer:hover{background:rgba(40,100,190,.055)}.compact-timer .timer-summary{position:absolute;inset:0 14px 4px;display:flex;align-items:center;justify-content:space-between;gap:12px;min-width:0}.compact-timer .timer-leading{display:flex;align-items:center;gap:7px;min-width:0;color:rgba(245,248,255,.9)}.compact-timer .timer-leading :global(svg){flex:none;color:#70aaff}.compact-timer .timer-time{min-width:0;overflow:hidden;color:#f5f7fb;font-size:13px;font-weight:600;line-height:1;font-variant-numeric:tabular-nums;letter-spacing:.01em;text-align:right;text-overflow:ellipsis;white-space:nowrap}.compact-timer .timer-progress-track{position:absolute;right:14px;bottom:5px;left:14px;height:2px;overflow:hidden;border-radius:999px;background:rgba(115,143,183,.2)}.compact-timer .timer-progress-fill{display:block;height:100%;max-width:100%;border-radius:inherit;background:linear-gradient(90deg,#2688ff,#60b5ff);box-shadow:0 0 7px rgba(57,145,255,.68);transition:width 350ms linear,background-color 180ms ease}.compact-timer .timer-progress-fill.paused{background:linear-gradient(90deg,#6d8fb9,#a7c5ed);box-shadow:none}.compact-timer .timer-progress-fill.finished{background:linear-gradient(90deg,#42d99a,#8af0c0)}.compact-timer.paused{color:#f4f7ff}.compact-timer.urgent{box-shadow:inset 1px 0 0 rgba(255,111,102,.58),inset -1px 0 0 rgba(255,111,102,.58),inset 0 0 12px rgba(255,74,60,.09)}.compact-timer.urgent .timer-leading :global(svg){color:#ff8c7c}.compact-timer .timer-pause{flex:none;width:7px;height:8px;gap:2px;color:#a8c5e9}.compact-timer .timer-pause b{width:2px;height:7px;border-radius:1px;background:currentColor}
  .compact-layer.vertical .compact-timer .timer-summary{inset:14px 4px;flex-direction:column;justify-content:center;gap:10px}.compact-layer.vertical .compact-timer .timer-leading{flex-direction:column;gap:5px}.compact-layer.vertical .compact-timer .timer-time{max-width:100%;font-size:11px;text-align:center}.compact-layer.vertical .compact-timer .timer-progress-track{top:14px;right:auto;bottom:14px;left:5px;width:2px;height:auto}.compact-layer.vertical .timer-progress-fill{width:100%!important;height:var(--timer-progress-height,0%)}
  .idle-player{display:flex;flex:1;min-height:0;flex-direction:column}.idle-cover{cursor:default;color:rgba(255,255,255,.58);background:linear-gradient(145deg,rgba(255,255,255,.12),rgba(255,255,255,.035));box-shadow:none}.idle-progress{height:3px;margin:0 0 7px;overflow:hidden;border-radius:999px;background:rgba(255,255,255,.13)}.idle-progress span{display:block;width:100%;height:100%;border-radius:inherit;background:linear-gradient(90deg,rgba(255,255,255,.38),rgba(255,255,255,.16))}.idle-controls{opacity:.36!important}.idle-controls.has-remembered-track{opacity:1!important}.idle-controls button:disabled{cursor:default}.idle-controls button:disabled:active{transform:none}
  .clock-dashboard{gap:11px}
  /* Keep the control area in the island's opaque black surface; the controls stay compact and quiet. */
  .function-toolbar{height:36px;box-sizing:border-box;gap:3px;padding:3px;border-color:rgba(255,255,255,.1);background:#151517;box-shadow:inset 0 1px 0 rgba(255,255,255,.045)}
  .function-icon{width:28px;height:28px;color:rgba(255,255,255,.62);transition:color 160ms ease,background 160ms ease,transform 180ms cubic-bezier(.2,.8,.2,1),box-shadow 160ms ease}
  .function-icon:hover{color:#fff}
  .function-icon.active{color:#ffd097}
  .function-icon.timer-running:not(.active){color:#ffbc6c}
  .function-icon.timer-paused:not(.active){color:rgba(255,188,108,.68)}
  .function-content{position:relative;overflow:visible;margin-top:8px;padding:0 4px}
  .function-content>*{animation:function-content-enter 220ms cubic-bezier(.2,.8,.2,1) both}
  @keyframes function-content-enter{from{opacity:0;transform:translateY(3px)}to{opacity:1;transform:translateY(0)}}
  .clock-dashboard{gap:11px;padding-right:7px;padding-left:5px}
  .clock-dashboard.with-forecast{gap:10px}
  .forecast-strip{gap:6px;padding-left:8px;border-color:rgba(255,255,255,.13)}
  .forecast-day{grid-template-rows:15px 22px 15px 11px;min-width:31px}
  .function-pane{padding:12px 16px 14px;background:#000;border-left-color:rgba(255,255,255,.09)}
  .function-pane{padding-right:max(16px,var(--shoulder-content-inset,16px))}
  .edge-top .music-pane,.edge-bottom .music-pane{padding-left:calc(28px + var(--shoulder-content-inset,0px));padding-right:calc(28px + var(--shoulder-content-inset,0px))}
  .attached-style.edge-top .function-pane,.attached-style.edge-bottom .function-pane{padding-right:calc(28px + var(--shoulder-content-inset,0px))}
  .function-toolbar{height:25px;gap:4px;padding:0;border:0;border-radius:0;background:transparent;box-shadow:none;backdrop-filter:none;-webkit-backdrop-filter:none}
  .function-icon{width:24px;height:24px;border-radius:7px;color:rgba(238,243,250,.58)}
  .function-icon:hover{background:transparent}
  .function-icon.active{color:#c3e2ff}
  .function-icon.timer-running:not(.active),.function-icon.timer-paused:not(.active){color:#75baff}
  .function-content{margin-top:5px;padding:0}
  .music-pane{padding-top:18px;padding-bottom:15px}
  .top-row{gap:10px;margin-bottom:10px}
  .function-pane :global(.device-trigger){border-color:rgba(255,255,255,.1);border-radius:7px;background:rgba(255,255,255,.045)}
  .function-pane :global(.device-trigger:hover:not(:disabled)),.function-pane :global(.device-trigger.open){border-color:rgba(111,176,255,.3);background:rgba(255,255,255,.075)}
  .function-pane :global(.device-menu button.active){background:rgba(77,153,247,.14)}
  .function-pane :global(.device-menu button svg){color:#75baff}
  .function-pane :global(.volume-slider){background:linear-gradient(90deg,#55a9ff 0%,#55a9ff var(--volume-progress),rgba(255,255,255,.15) var(--volume-progress),rgba(255,255,255,.15) 100%)}
  .function-pane :global(.volume-slider::-moz-range-progress){background:#55a9ff}
  .function-pane :global(.volume-slider::-webkit-slider-thumb){background:#55a9ff}
  .function-pane :global(.volume-slider::-moz-range-thumb){background:#55a9ff}
  .function-pane :global(.device-trigger:focus-visible),.function-pane :global(.device-menu button:focus-visible){outline-color:#8ac5ff}
  @media (hover:hover) and (pointer:fine){.function-icon:hover{background:rgba(255,255,255,.09);transform:scale(1.04)}.function-icon:not(.active):hover{color:#fff}}
  @media (prefers-reduced-motion:reduce){.function-icon,.function-content>*{transition:none;animation:none}}
  .studio-preview-highlight{position:absolute;z-index:20;inset:0;overflow:hidden;border-radius:inherit;pointer-events:none}.studio-preview-highlight::before,.studio-preview-highlight::after{content:"";position:absolute;pointer-events:none}.highlight-shape::after{inset:2px;border:2px solid rgba(167,139,250,.9);border-radius:inherit;box-shadow:0 0 12px rgba(167,139,250,.42),inset 0 0 7px rgba(167,139,250,.18)}.highlight-edge::before{background:#60a5fa;box-shadow:0 0 9px rgba(96,165,250,.8)}.edge-top .highlight-edge::before,.edge-bottom .highlight-edge::before{left:18px;right:18px;height:2px}.edge-top .highlight-edge::before{top:2px}.edge-bottom .highlight-edge::before{bottom:2px}.edge-left .highlight-edge::before,.edge-right .highlight-edge::before{top:18px;bottom:18px;width:2px}.edge-left .highlight-edge::before{left:2px}.edge-right .highlight-edge::before{right:2px}.highlight-position::after{width:8px;height:8px;border:2px solid #fff;border-radius:50%;background:#60a5fa;box-shadow:0 0 0 3px rgba(96,165,250,.2),0 0 10px rgba(96,165,250,.92)}.edge-top .highlight-position::after,.edge-bottom .highlight-position::after{left:calc(var(--preview-position) * 1% - 6px)}.edge-top .highlight-position::after{top:-3px}.edge-bottom .highlight-position::after{bottom:-3px}.edge-left .highlight-position::after,.edge-right .highlight-position::after{top:calc(var(--preview-position) * 1% - 6px)}.edge-left .highlight-position::after{left:-3px}.edge-right .highlight-position::after{right:-3px}.corner{position:absolute;width:calc(var(--preview-radius-tl) + 9px);height:calc(var(--preview-radius-tl) + 9px);max-width:48px;max-height:48px;opacity:.95}.corner.top-left{top:2px;left:2px;width:calc(var(--preview-radius-tl) + 9px);height:calc(var(--preview-radius-tl) + 9px);border:2px solid;border-color:#a78bfa transparent transparent #a78bfa;border-radius:var(--preview-radius-tl) 0 0 0}.corner.top-right{top:2px;right:2px;width:calc(var(--preview-radius-tr) + 9px);height:calc(var(--preview-radius-tr) + 9px);border:2px solid;border-color:#a78bfa #a78bfa transparent transparent;border-radius:0 var(--preview-radius-tr) 0 0}.corner.bottom-left{bottom:2px;left:2px;width:calc(var(--preview-radius-bl) + 9px);height:calc(var(--preview-radius-bl) + 9px);border:2px solid;border-color:transparent transparent #a78bfa #a78bfa;border-radius:0 0 0 var(--preview-radius-bl)}.corner.bottom-right{right:2px;bottom:2px;width:calc(var(--preview-radius-br) + 9px);height:calc(var(--preview-radius-br) + 9px);border:2px solid;border-color:transparent #a78bfa #a78bfa transparent;border-radius:0 0 var(--preview-radius-br) 0}.length-handle{position:absolute;width:3px;height:22px;border-radius:999px;background:#60a5fa;box-shadow:0 0 9px rgba(96,165,250,.8)}.length-handle.start{left:2px;top:50%;transform:translateY(-50%)}.length-handle.end{right:2px;top:50%;transform:translateY(-50%)}.length-vertical .length-handle{width:22px;height:3px;left:50%;transform:translateX(-50%)}.length-vertical .length-handle.start{top:2px}.length-vertical .length-handle.end{top:auto;right:auto;bottom:2px}.shoulder-handle{position:absolute;width:3px;height:18px;border-radius:999px;background:#a78bfa;box-shadow:0 0 8px rgba(167,139,250,.8)}.edge-top .shoulder-handle.start{top:2px;left:calc(var(--preview-shoulder) + 1px)}.edge-top .shoulder-handle.end{top:2px;right:calc(var(--preview-shoulder) + 1px)}.edge-bottom .shoulder-handle.start{bottom:2px;left:calc(var(--preview-shoulder) + 1px)}.edge-bottom .shoulder-handle.end{right:calc(var(--preview-shoulder) + 1px);bottom:2px}.edge-left .shoulder-handle,.edge-right .shoulder-handle{width:18px;height:3px;top:calc(var(--preview-shoulder) + 1px)}.edge-left .shoulder-handle.start{left:2px}.edge-left .shoulder-handle.end{bottom:calc(var(--preview-shoulder) + 1px);top:auto;left:2px}.edge-right .shoulder-handle.start{right:2px}.edge-right .shoulder-handle.end{right:2px;top:auto;bottom:calc(var(--preview-shoulder) + 1px)}.background-wash{position:absolute;inset:2px;border-radius:inherit;background:rgba(167,139,250,.14);box-shadow:inset 0 0 0 1px rgba(196,181,253,.72),0 0 16px rgba(167,139,250,.22)}.spectrum-focus{position:absolute;z-index:1;top:50%;right:5px;width:23px;height:21px;border:1px solid #60a5fa;border-radius:6px;background:rgba(96,165,250,.1);box-shadow:0 0 10px rgba(96,165,250,.55);transform:translateY(-50%)}.preview-expanded .spectrum-focus{top:14px;right:22px;width:47px;height:31px;border-radius:8px}
  @media (hover:hover) and (pointer:fine){.controls button:hover:not(:disabled){background:rgba(255,255,255,.1);transform:scale(1.06);color:#fff}.cover:hover{filter:brightness(1.08)}}
  @media (prefers-reduced-motion:reduce){.surface-anchor{transition:none!important}.island-surface,.expanded-layer,.controls button{transition-duration:120ms!important}.island-surface{transition-property:opacity,box-shadow!important}.expanded-layer{transform:none!important}.cover-image,.compact-disc{animation:none}.studio-preview-highlight .corner,.studio-preview-highlight::after,.studio-preview-highlight .length-handle,.studio-preview-highlight .shoulder-handle,.studio-preview-highlight .background-wash,.studio-preview-highlight .spectrum-focus{animation:none!important;opacity:.78}}
</style>
