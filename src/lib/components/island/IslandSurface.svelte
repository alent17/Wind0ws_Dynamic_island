<script lang="ts">
  import { onMount, onDestroy, untrack, tick } from "svelte";
  import { spring, tweened } from "svelte/motion";
  import { slide } from "svelte/transition";
  import { FolderOpen, Activity, Airplay, ArrowLeft, Clock, CloudSun, EyeOff, GalleryHorizontalEnd, Music2, PanelsTopLeft, Pencil, Settings, Shuffle, SkipBack, SkipForward, Timer, Volume2 } from "lucide-svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { settingsStore } from "$lib/settingsStore";
  import { settingsApi } from "$lib/api/settings";
  import MediaProgress from "$lib/components/media/MediaProgress.svelte";
  import PlayPauseIcon from "$lib/components/media/PlayPauseIcon.svelte";
  import RollingNumber from "$lib/components/time/RollingNumber.svelte";
  import Spectrum from "$lib/components/media/Spectrum.svelte";
  import CoverArt from "$lib/components/media/CoverArt.svelte";
  import NeteaseConnectedModeControl from "$lib/components/media/NeteaseConnectedModeControl.svelte";
  import NeteaseLikeControl from "$lib/components/media/NeteaseLikeControl.svelte";
  import SmtcModeControls from "$lib/components/media/SmtcModeControls.svelte";
  import TimerPanel from "$lib/components/time/TimerPanel.svelte";
  import MarqueeTitle from "$lib/components/media/MarqueeTitle.svelte";
  import SystemMonitor from "$lib/components/island/SystemMonitor.svelte";
  import type { SystemSnapshot } from "$lib/systemMonitor";
  import CalendarWidget from "$lib/components/time/CalendarWidget.svelte";
  import CalendarExtension from "$lib/components/time/CalendarExtension.svelte";
  import WorldClockExtension from "$lib/components/time/WorldClockExtension.svelte";
  import WidgetLayout from "$lib/components/island/WidgetLayout.svelte";
  import FeatureStack from "$lib/components/island/FeatureStack.svelte";
  import WeatherExtension from "$lib/components/island/WeatherExtension.svelte";
  import { normalizeWidgets, type IslandWidgets, type IslandWidgetKind } from "$lib/widgetLayout";
  import InfoPanel from "$lib/components/island/InfoPanel.svelte";
  import FeatureMenu from "$lib/components/island/FeatureMenu.svelte";
  import VolumePanel from "$lib/components/volume/VolumePanel.svelte";
  import AppVolumeMixer from "$lib/components/volume/AppVolumeMixer.svelte";
  import WeatherIcon from "$lib/components/media/WeatherIcon.svelte";
  import type { IslandTool } from "$lib/featureRail";
  import { formatCountdown, type CountdownStatus } from "$lib/countdown";
  import { ISLAND_MOTION, widgetEditMotion, widgetLayoutMotion, islandMorphEasing, islandSettleEasing, islandShapeMotion, anchoredLayoutOffset } from "$lib/islandMotion";
  import type { AudioDeviceInfo, MediaState, SpectrumMode, SystemAudioState, WeatherForecastDay } from "$lib/api/types";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import {
    navigationGeometry,
    featureExtensionGeometry,
    worldClockExtensionGeometry,
    volumeExtensionGeometry,
    islandAnchorGap,
    type IslandPage,
    borderRadiusCss,
    geometryFor,
    interpolatePolygon,
    radiiFor,
    polygonCss,
    shapePolygonFor,
    scaleIslandGeometry,
    type IslandEdge,
    type IslandMode,
    type IslandRegionChange,
    type IslandStyle,
  } from "$lib/islandGeometry";

  function volumeWheel(node: HTMLElement) {
    const handleWheel = (event: WheelEvent) => {
      if (!wheelVolumeControl || !onWheelVolume || !interactive || !visible || mode === "hidden" || editing || featureExpanded || event.defaultPrevented || event.ctrlKey || event.deltaY === 0 || Math.abs(event.deltaX) > Math.abs(event.deltaY)) return;
      event.preventDefault();
      event.stopPropagation();
      if (volumeProtectionPrompt) return;
      onWheelVolume(event.deltaY < 0 ? 2 : -2);
    };
    node.addEventListener("wheel", handleWheel, { passive: false });
    return { destroy: () => node.removeEventListener("wheel", handleWheel) };
  }

  function animateSkip(event: MouseEvent, direction: -1 | 1) {
    if (!enableAnimations || reduceAnimations || prefersReducedMotion) return;
    const icon = (event.currentTarget as HTMLButtonElement).querySelector("svg");
    if (!icon) return;
    icon.getAnimations().forEach((animation) => animation.cancel());
    icon.animate([
      { transform: "translateX(0) scale(1)", opacity: 1, offset: 0 },
      { transform: `translateX(${direction * 14}px) scale(.86)`, opacity: 0, offset: .45 },
      { transform: `translateX(${-direction * 14}px) scale(.86)`, opacity: 0, offset: .46 },
      { transform: "translateX(0) scale(1)", opacity: 1, offset: 1 },
    ], { duration: 300, easing: "cubic-bezier(.22,.61,.36,1)" });
  }

  type PreviewHighlight="shape"|"edge"|"position"|"compactLength"|"shoulder"|"radius"|"background"|"spectrum"|null;

  let {
    media,
    monitorSnapshot,
    islandWidgets = { left: [], right: [] },
    widgetsEnabled = true,
    onWidgetsToggle,
    onWidgetsSave,
    onLayoutPreview,
    mode = "compact",
    islandStyle = "floating",
    edge = "top",
    position = 0,
    expandedRadius = 45,
    expandedLength = 600,
    collapsedEdgeShoulderRadius = 8,
    expandedEdgeShoulderRadius = 32,
    expandedScale = 1,
    compactLength = 80,
    idle = false,
    idleTime = "",
    idleWeatherTemperature = null,
    idleWeatherCode = null,
    idleWeatherForecast = [],
    lastPlayedMedia = null,
    canResumeLastTrack = false,
    resumingLastTrack = false,
    neteaseModeBusy = false,
    neteaseModeError = "",
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
    seekBusy = false,
    seekError = "",
    onToggleFloating,
    onHideForTenSeconds,
    keepIslandExpanded = false,
    onToggleKeepIslandExpanded,
    onHoverChange,
    onRegionChange,
    systemAudio,
    onSettingsToggle,
    onAudioOpen,
    onAudioVolume,
    wheelVolumeControl = false,
    onWheelVolume,
    wheelVolumeDisplay = null,
    volumeProtectionPrompt = false,
    volumeProtectionLimit = 80,
    onConfirmVolumeIncrease,
    onCancelVolumeIncrease,
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
    monitorSnapshot?: SystemSnapshot | null;
    islandWidgets?: IslandWidgets;
    widgetsEnabled?: boolean;
    onWidgetsToggle?: (enabled: boolean) => Promise<void>;
    onWidgetsSave?: (layout: IslandWidgets) => Promise<void>;
    onLayoutPreview?: (layout: IslandWidgets, editing: boolean) => void;
    mode?: IslandMode;
    islandStyle?: IslandStyle;
    edge?: IslandEdge;
    position?: number;
    expandedRadius?: number;
    expandedLength?: number;
    collapsedEdgeShoulderRadius?: number;
    expandedEdgeShoulderRadius?: number;
    expandedScale?: number;
    compactLength?: number;
    idle?: boolean;
    idleTime?: string;
    idleWeatherTemperature?: number | null;
    idleWeatherCode?: number | null;
    idleWeatherForecast?: WeatherForecastDay[];
    lastPlayedMedia?: { title: string; artist: string; albumArt: string; sourceDisplay: string } | null;
    canResumeLastTrack?: boolean;
    resumingLastTrack?: boolean;
    neteaseModeBusy?: boolean;
    neteaseModeError?: string;
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
    onMediaAction?: (action: "prev" | "play_pause" | "next" | "cycle_netease_mode") => void;
    onSeek?: (positionMs: number) => void | Promise<void>;
    seekBusy?: boolean;
    seekError?: string;
    onToggleFloating?: () => void;
    onHideForTenSeconds?: () => void;
    keepIslandExpanded?: boolean;
    onToggleKeepIslandExpanded?: () => void | Promise<void>;
    onSettingsToggle?: () => void;
    onHoverChange?: (hovering: boolean) => void;
    onRegionChange?: (change: IslandRegionChange) => void;
    systemAudio?: SystemAudioState | null;
    onAudioOpen?: () => void | Promise<void>;
    onAudioVolume?: (volumePercent: number) => void | Promise<void>;
    wheelVolumeControl?: boolean;
    onWheelVolume?: (delta: number) => void;
    wheelVolumeDisplay?: number | null;
    volumeProtectionPrompt?: boolean;
    volumeProtectionLimit?: number;
    onConfirmVolumeIncrease?: () => void;
    onCancelVolumeIncrease?: () => void;
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
  const motionOptions = { duration: ISLAND_MOTION.layoutDuration, easing: islandSettleEasing };
  const volumeHudMarks = Array.from({ length: 21 }, (_, index) => index * 5);
  let previousMotionMode = untrack(() => mode);
  let previousMotionStyle = untrack(() => islandStyle);
  let expandedLayoutMotion = $state(untrack(() => mode === "expanded"));
  const widthMotion = tweened(initial.width, motionOptions);
  const heightMotion = tweened(initial.height, motionOptions);
  const radiusMotion = tweened(initial.radius, motionOptions);
  const styleMorph = tweened(untrack(() => islandStyle === "edge" ? 1 : 0), motionOptions);
  const hideMorph = spring(untrack(() => simulateHidden && mode === "hidden" ? 1 : 0), { stiffness: 0.18, damping: 0.7, precision: 0.001 });
  let animatedWidth = $state(initial.width);
  let animatedHeight = $state(initial.height);
  let animatedRadius = $state(initial.radius);
  let styleProgress = $state(untrack(() => islandStyle === "edge" ? 1 : 0));
  let hideProgress = $state(untrack(() => simulateHidden && mode === "hidden" ? 1 : 0));
  let prefersReducedMotion = $state(false);
  const shoulderMotion = tweened(untrack(() => mode === "expanded" ? expandedEdgeShoulderRadius * expandedScale : collapsedEdgeShoulderRadius), motionOptions);
  let animatedShoulder = $state(untrack(() => mode === "expanded" ? expandedEdgeShoulderRadius * expandedScale : collapsedEdgeShoulderRadius));
  const unsubscribeShoulder = shoulderMotion.subscribe(value => animatedShoulder = value);
  let shapeSettled = $state(false);
  let compactShapeMotion = $state(untrack(() => mode === "compact" || mode === "hover"));
  let transitionRevision = 0;
  const unsubscribeWidth = widthMotion.subscribe((value) => animatedWidth = value);
  const unsubscribeHeight = heightMotion.subscribe((value) => animatedHeight = value);
  const unsubscribeRadius = radiusMotion.subscribe((value) => animatedRadius = value);
  const unsubscribeStyle = styleMorph.subscribe((value) => styleProgress = Math.max(0, Math.min(1, value)));
  const unsubscribeHide = hideMorph.subscribe((value) => hideProgress = value);

  const size = $derived({ width: Math.max(1, animatedWidth), height: Math.max(1, animatedHeight), radius: Math.max(0, Math.min(80, animatedRadius)) });
  let page = $state<IslandPage>("music");
  let pageNavigationRevision = 0;
  let wasTimerFinished = $state(false);
  let menuScroll = $state(0);
  let pageRoot = $state<HTMLDivElement>();
  let editing = $state(false);
  let previousEditing = untrack(() => editing);
  let editLayoutMotion = $state(false);
  let featureExpanded = $state(false);
  let featureKind = $state<"volume" | "calendar" | "clock" | "weather" | "stack">("volume");
  let stackPreparing = $state(false);
  let stackSelection = $state<IslandWidgetKind | undefined>();
  let stackTimer: ReturnType<typeof setTimeout> | undefined;
  function closeFeature() { clearTimeout(stackTimer); featureFromStack = false; featureExpanded = false; stackPreparing = false; }
  function openStack() {
    if (!interactive || !visible || mode !== "expanded" || editing || featureExpanded || stackPreparing) return;
    stackSelection = configuredLayout.left[0] ?? configuredLayout.right[0];
    stackPreparing = true;
    const delay = enableAnimations && !reduceAnimations && !prefersReducedMotion && widgetCount > 0 ? ISLAND_MOTION.collapseDuration : 0;
    stackTimer = setTimeout(() => { if (visible && mode === "expanded" && !editing) { featureKind = "stack"; featureExpanded = true; } else stackPreparing = false; }, delay);
  }
  onDestroy(() => clearTimeout(stackTimer));
  function musicStackGesture(node:HTMLElement) {
    let start:{id:number;x:number;y:number}|null = null;
    const wheel = (event:WheelEvent) => { if (editing || featureExpanded || stackPreparing || event.ctrlKey || event.deltaY <= 0 || Math.abs(event.deltaX) > Math.abs(event.deltaY) || (event.target as HTMLElement).closest("input,select,textarea,[role=slider]")) return; event.preventDefault(); event.stopPropagation(); openStack(); };
    const down = (event:PointerEvent) => { if (editing || featureExpanded || stackPreparing || !event.isPrimary || event.button !== 0 || (event.target as HTMLElement).closest("button,input,select,textarea,[role=slider]")) return; start={id:event.pointerId,x:event.clientX,y:event.clientY}; };
    const move = (event:PointerEvent) => { if (!start || start.id !== event.pointerId) return; const dx=event.clientX-start.x, dy=event.clientY-start.y; if (dy>28 && dy>Math.abs(dx)) {start=null;event.stopPropagation();openStack();} else if (Math.abs(dx)>28 || dy< -28) start=null; };
    const end = () => {start=null;};
    node.addEventListener("wheel",wheel,{passive:false});node.addEventListener("pointerdown",down);node.addEventListener("pointermove",move);node.addEventListener("pointerup",end);node.addEventListener("pointercancel",end);
    return {destroy(){node.removeEventListener("wheel",wheel);node.removeEventListener("pointerdown",down);node.removeEventListener("pointermove",move);node.removeEventListener("pointerup",end);node.removeEventListener("pointercancel",end);}};
  }
  let featureFromStack = false;
  let volumeFeatureMetrics = $state({rows:4,total:4});
  function backFromFeature() { if (featureFromStack) { featureFromStack = false; featureKind = "stack"; } else closeFeature(); }
  function openFeature(kind: "volume" | "calendar" | "clock" | "weather") {
    featureFromStack = featureExpanded && featureKind === "stack";
    if (kind === "volume") volumeFeatureMetrics = {rows:Math.min(5, volumeFeatureMetrics.total),total:volumeFeatureMetrics.total};
    featureKind = kind; featureExpanded = true;
  }
  function updateVolumeFeatureMetrics(rows:number, total:number) {
    if (volumeFeatureMetrics.rows !== rows || volumeFeatureMetrics.total !== total) volumeFeatureMetrics = {rows,total};
  }
  let previousFeatureExpanded = false;
  let previousStackPreparing = false;
  const featureOpacity = tweened(0);
  let featureOpacityValue = $state(0);
  const unsubscribeFeatureOpacity = featureOpacity.subscribe(value => featureOpacityValue = value);
  onDestroy(unsubscribeFeatureOpacity);
  let styleLayoutMotion = $state(false);
  let contextMenuOpen = $state(false);
  let contextMenuRoot = $state<HTMLDivElement>();
  let surfaceRoot = $state<HTMLDivElement>();
  let contextMenuRect = $state({x:0,y:0,width:224,height:216});
  let savingLayout = $state(false);
  let layoutError = $state(false);
  let draftLayout = $state<IslandWidgets>({left: [], right: []});
  const configuredLayout = $derived(normalizeWidgets(islandWidgets));
  const layoutActive = $derived(widgetsEnabled && configuredLayout.left.length + configuredLayout.right.length > 0);
  let togglingWidgets = $state(false);
  let widgetToggleError = $state(false);
  const widgetReveal = tweened(untrack(() => widgetsEnabled ? 1 : 0));
  let widgetRevealValue = $state(untrack(() => widgetsEnabled ? 1 : 0));
  const unsubscribeWidgetReveal = widgetReveal.subscribe(value => widgetRevealValue = value);
  onDestroy(unsubscribeWidgetReveal);
  let previousWidgetsEnabled = untrack(() => widgetsEnabled);
  const currentLayout = $derived(editing ? draftLayout : widgetsEnabled ? configuredLayout : {left:[],right:[]});
  async function toggleWidgets() {
    if (togglingWidgets || editing) return;
    widgetToggleError = false;
    if (!configuredLayout.left.length && !configuredLayout.right.length) {beginEditing();return;}
    const next = !widgetsEnabled;
    togglingWidgets = true;
    try {await onWidgetsToggle?.(next);widgetsEnabled = next;}
    catch {widgetToggleError = true;}
    finally {togglingWidgets = false;}
  }
  const widgetCount = $derived(currentLayout.left.length + currentLayout.right.length);
  let previousWidgetCount = untrack(() => widgetCount);
  let editRevision = 0;
  function beginEditing() {
    if (!interactive || !visible || editing) return;
    contextMenuOpen = false;
    editRevision++;
    draftLayout = normalizeWidgets(islandWidgets);
    layoutError = false;
    page = "music";
    editing = true;
    suppressBackgroundClick = true;
    if (mode !== "expanded") onToggle?.();
    void tick().then(() => pageRoot?.querySelector<HTMLButtonElement>(".catalog-item")?.focus({preventScroll: true}));
  }
  function cancelEditing() {
    if (savingLayout) return;
    editRevision++;
    editing = false; layoutError = false;
  }
  async function saveLayout() {
    if (savingLayout) return;
    const revision = editRevision;
    savingLayout = true; layoutError = false;
    try {
      await onWidgetsSave?.(normalizeWidgets(draftLayout));
      widgetsEnabled = true;
      if (revision === editRevision) editing = false;
    } catch {
      if (revision === editRevision) layoutError = true;
    } finally { savingLayout = false; }
  }
  $effect(() => {
    if (!visible || mode === "hidden") contextMenuOpen = false;
    if ((!visible || mode === "hidden") && editing) { editing = false; untrack(() => editRevision++); }
    if ((!visible || mode !== "expanded") && (featureExpanded || stackPreparing)) closeFeature();
  });
  $effect(() => { onLayoutPreview?.(currentLayout, editing); });
  const panelVisible = $derived(!editing && showCustomFunctionPanel);
  const toolOrder: IslandTool[] = ["files", "timer", "volume", "floating", "settings", "hide", "clock", "weather", "system"];
  const toolIcons = { files: FolderOpen, timer: Timer, volume: Volume2, floating: GalleryHorizontalEnd, settings: Settings, hide: EyeOff, clock: Clock, weather: CloudSun, system: Activity };
  const toolLabels: Record<IslandTool, TranslationKey> = { files: "fileSort", timer: "timer", volume: "volume", floating: "floatingTool", settings: "settingsTool", hide: "hideTool", clock: "clock", weather: "weather", system: "systemMonitor" };
  const shortLabels: Record<IslandTool, TranslationKey> = {files:"fileSort",timer:"timer",volume:"menuVolume",floating:"menuFloating",settings:"menuSettings",hide:"menuHide",clock:"clock",weather:"weather",system:"systemMonitor"};
  const menuItems = $derived(toolOrder.filter(tool => (tool === "system" || tool === "files") || enabledTools.includes(tool)).map(tool => ({ id: tool, label: t(toolLabels[tool]), shortLabel: t(shortLabels[tool]), icon: toolIcons[tool] })));
  const expandedBaseGeometry = $derived(navigationGeometry(page, panelVisible ? menuItems.length : 0, expandedRadius, islandStyle, edge, expandedEdgeShoulderRadius, expandedLength, stackPreparing ? 0 : widgetCount, editing));
  const expandedGeometry = $derived({
    ...scaleIslandGeometry(expandedBaseGeometry, expandedScale),
  });

  let compactRoot = $state<HTMLDivElement>();
  type SpectrumBox = { x: number; y: number; width: number; height: number };
  let spectrumLayouts = $state<{ compact: SpectrumBox; expanded: SpectrumBox } | null>(null);
  const compactMode = "compact" as const;
  let coverLayouts = $state<{ compact: { x: number; y: number; width: number }; expanded: { x: number; y: number; width: number } } | null>(null);
  const compactGeometry = $derived(geometryFor(compactMode, expandedRadius, edge, effectiveCompactLength));
  const compactInset = $derived(styleProgress * Math.min(collapsedEdgeShoulderRadius, Math.max(compactGeometry.width, compactGeometry.height) / 4, Math.min(compactGeometry.width, compactGeometry.height) / 2));
  const expandedInset = $derived(styleProgress * Math.min(expandedEdgeShoulderRadius, Math.max(expandedGeometry.width, expandedGeometry.height) / 4, Math.min(expandedGeometry.width, expandedGeometry.height) / 2));
  const featureBaseGeometry = $derived(featureKind === "volume"
    ? volumeExtensionGeometry(volumeFeatureMetrics.rows, volumeFeatureMetrics.total > 5, expandedRadius, edge === "left" || edge === "right" ? expandedInset : 0)
    : featureKind === "stack" ? {...featureExtensionGeometry(expandedRadius), width:expandedLength + 48 + ((edge === "top" || edge === "bottom") ? expandedInset * 2 : 0), height:537 + ((edge === "left" || edge === "right") ? expandedInset * 2 : 0)} : featureKind === "clock" ? worldClockExtensionGeometry(expandedRadius) : featureExtensionGeometry(expandedRadius));
  const featureGeometry = $derived(scaleIslandGeometry(featureBaseGeometry, expandedScale));
  const featureOffset = $derived(anchoredLayoutOffset(size, featureGeometry, edge));
  const compactLayout = $derived(compactShapeMotion ? size : {
    width: Math.min(size.width, compactGeometry.width),
    height: Math.min(size.height, compactGeometry.height),
  });
  const compactOffset = $derived(anchoredLayoutOffset(size, compactLayout, edge));
  const expandedLayoutGeometry = $derived(editLayoutMotion && mode === "expanded" ? {width:size.width,height:size.height} : expandedGeometry);
  const expandedContentGeometry = $derived(editLayoutMotion && mode === "expanded"
    ? {width:expandedLayoutGeometry.width / Math.max(.001, expandedScale),height:expandedLayoutGeometry.height / Math.max(.001, expandedScale)}
    : expandedBaseGeometry);
  const expandedOffset = $derived(anchoredLayoutOffset(size, expandedLayoutGeometry, edge));

  // Read the real layout boxes, including CSS zoom, toolbar and shoulder padding.
  // Translation cancels when subtracting the root, so measurements stay stable
  // while the island and the native window move.
  $effect(() => {
    const compact = compactRoot;
    const expanded = pageRoot;
    compactGeometry; expandedGeometry; compactInset; expandedInset; panelVisible; renderedPage; idle; showTime; timerActive;
    if (!compact || !expanded) { coverLayouts = null; return; }
    // A hover layout follows the shell. Its artwork does not fly, so wait
    // for the endpoint instead of measuring every ResizeObserver delivery.
    if (compactShapeMotion && !shapeSettled) return;
    let disposed = false;
    const measure = () => {
      if (disposed) return;
      const small = compact.querySelector<HTMLElement>(".compact-cover");
      const large = expanded.querySelector<HTMLElement>(".top-row .expanded-cover");
      if (!small || !large) { coverLayouts = null; return; }
      const a = small.getBoundingClientRect(), b = large.getBoundingClientRect();
      const c = compact.getBoundingClientRect(), d = expanded.getBoundingClientRect();
      // Rects use viewport pixels; the Studio preview can scale the entire
      // host. Convert back to surface coordinates without removing CSS zoom.
      const css = getComputedStyle(compact);
      const scaleX = c.width / parseFloat(css.width) || 1;
      const scaleY = c.height / parseFloat(css.height) || 1;
      const next = { compact: { x: (a.left - c.left) / scaleX, y: (a.top - c.top) / scaleY, width: a.width / scaleX }, expanded: { x: (b.left - d.left) / scaleX, y: (b.top - d.top) / scaleY, width: b.width / scaleX } };
      if (JSON.stringify(next) !== JSON.stringify(untrack(() => coverLayouts))) coverLayouts = next;
    };
    const observer = new ResizeObserver(measure);
    observer.observe(compact); observer.observe(expanded);
    void tick().then(measure);
    return () => { disposed = true; observer.disconnect(); };
  });

  // Match the artwork's shared-element path instead of fading two spectra.
  $effect(() => {
    const compact = compactRoot, expanded = pageRoot;
    compactGeometry; expandedGeometry; compactInset; expandedInset; panelVisible; renderedPage; idle; showTime; timerActive; showSpectrum;
    if (!compact || !expanded) { spectrumLayouts = null; return; }
    if (compactShapeMotion && !shapeSettled) return;
    let disposed = false;
    const measure = () => {
      if (disposed) return;
      const small = compact.querySelector<HTMLElement>(".spectrum-endpoint");
      const large = expanded.querySelector<HTMLElement>(".spectrum-endpoint");
      if (!small || !large) { spectrumLayouts = null; return; }
      const c = compact.getBoundingClientRect(), d = expanded.getBoundingClientRect();
      const css = getComputedStyle(compact);
      const sx = c.width / parseFloat(css.width) || 1, sy = c.height / parseFloat(css.height) || 1;
      const box = (element: HTMLElement, root: DOMRect): SpectrumBox => {
        const rect = element.getBoundingClientRect();
        return { x: (rect.left - root.left) / sx, y: (rect.top - root.top) / sy, width: rect.width / sx, height: rect.height / sy };
      };
      const next = { compact: box(small, c), expanded: box(large, d) };
      if (JSON.stringify(next) !== JSON.stringify(untrack(() => spectrumLayouts))) spectrumLayouts = next;
    };
    const observer = new ResizeObserver(measure);
    observer.observe(compact); observer.observe(expanded);
    void tick().then(measure);
    return () => { disposed = true; observer.disconnect(); };
  });

  let surfaceDisposed = false;
  onDestroy(() => { surfaceDisposed = true; });
  async function navigate(next: IslandPage) {
    const revision = ++pageNavigationRevision;
    page = next;
    await tick();
    if (surfaceDisposed || revision !== pageNavigationRevision || mode !== "expanded" || !visible || page !== next || renderedPage !== next) return;
    pageRoot?.querySelector<HTMLButtonElement>(next === "music" ? ".feature-menu button" : ".page-back")?.focus({ preventScroll: true });
  }
  function goBack() {
    if (pendingSort) { cancelPendingSort(); return; }
    if (page === "music") { onToggle?.(); return; }
    void navigate("music");
  }
  function selectTool(id: string) {
    if (id === "files" || id === "timer" || id === "volume" || id === "clock" || id === "weather" || id === "system") {
      void navigate(id);
      if (id === "volume") void onAudioOpen?.();
    } else if (id === "floating") onToggleFloating?.();
    else if (id === "settings") onSettingsToggle?.();
    else if (id === "hide") onHideForTenSeconds?.();
  }

  type SortResult = { source: string; status: "moved" | "skipped" | "failed"; destination: string | null; message: string };
  let sortResults = $state<SortResult[]>([]);
  let sortError = $state("");
  let sortBusy = $state(false);
  let sortProgress = $state({ completed: 0, total: 0 });
  let checkingSortInputs = $state(false);
  let pendingSort = $state<{ paths: string[]; folders: string[] } | null>(null);
  let fileDragActive = $state(false);
  let pendingFilePage = $state(false);
  $effect(() => { if (pendingFilePage && mode === "expanded") { pendingFilePage = false; void navigate("files"); } });
  function showFilePage() {
    if (mode === "expanded") void navigate("files");
    else { pendingFilePage = true; onToggle?.(); }
  }
  async function chooseSortDirectory() {
    try {
      const directory = await invoke<string | null>("choose_sort_directory");
      if (directory) await settingsApi.updateSettings({ fileSortDirectory: directory });
    } catch (error) { sortError = String(error); }
  }
  async function setSortDate(value: "none" | "year" | "month") {
    try { await settingsApi.updateSettings({ fileSortDate: value }); }
    catch (error) { sortError = String(error); }
  }
  async function runFileSort(paths: string[], confirmedFolders: string[]) {
    sortBusy = true; sortError = ""; sortResults = []; pendingSort = null;
    sortProgress = { completed: 0, total: 0 };
    const batchId = crypto.randomUUID();
    let stopProgress: (() => void) | undefined;
    try {
      try {
        stopProgress = await listen<{ batchId: string; completed: number; total: number }>("file-sort-progress", ({ payload: progress }) => {
          if (progress.batchId === batchId) sortProgress = { completed: progress.completed, total: progress.total };
        });
      } catch (error) { console.warn("Failed to subscribe to file sorting progress", error); }
      sortResults = await invoke<SortResult[]>("sort_dropped_files", { paths, batchId, confirmedFolders });
      sortProgress = { completed: sortResults.length, total: sortResults.length };
    } catch (error) { sortError = String(error); }
    finally { stopProgress?.(); sortBusy = false; }
  }
  function cancelPendingSort() { pendingSort = null; }
  function confirmPendingSort() {
    if (!pendingSort) return;
    const { paths, folders } = pendingSort;
    void runFileSort(paths, folders);
  }
  onMount(() => {
    if (!interactive || !isTauri() || getCurrentWindow().label !== "main") return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWindow().onDragDropEvent(async ({ payload }) => {
      if (payload.type === "enter") { fileDragActive = true; return; }
      if (payload.type === "leave") { fileDragActive = false; return; }
      if (payload.type !== "drop") return;
      fileDragActive = false;
      showFilePage();
      if (sortBusy || checkingSortInputs || pendingSort) { sortError = "请先完成当前操作，再拖入其他文件"; return; }
      if (!$settingsStore.fileSortDirectory) { sortError = "请先选择目标文件夹，然后重新拖入文件"; return; }
      checkingSortInputs = true; sortError = "";
      try {
        const folders = await invoke<string[]>("inspect_sort_inputs", { paths: payload.paths });
        if (disposed) return;
        if (folders.length) {
          pendingSort = { paths: payload.paths, folders };
          await tick();
          pageRoot?.querySelector<HTMLButtonElement>(".file-confirm-cancel")?.focus({ preventScroll: true });
        } else {
          void runFileSort(payload.paths, []);
        }
      } catch (error) { sortError = String(error); }
      finally { checkingSortInputs = false; }
    }).then(stop => { if (disposed) stop(); else unlisten = stop; }).catch(error => { sortError = String(error); });
    return () => { disposed = true; unlisten?.(); };
  });

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
    const target = mode === "expanded" ? (featureExpanded ? featureGeometry : expandedGeometry) : geometryFor(mode, expandedRadius, edge, effectiveCompactLength);
    const hard = !enableAnimations || reduceAnimations || prefersReducedMotion;
    compactShapeMotion = (previousMotionMode === "compact" || previousMotionMode === "hover")
      && (mode === "compact" || mode === "hover");
    const widgetToggle = previousWidgetsEnabled !== widgetsEnabled && previousMotionMode === "expanded" && mode === "expanded";
    const widgetChange = previousWidgetCount !== widgetCount && previousMotionMode === "expanded" && mode === "expanded";
    const editingChanged = previousEditing !== editing && previousMotionMode === "expanded" && mode === "expanded";
    const featureChanged = previousFeatureExpanded !== featureExpanded;
    const stackChanged = previousStackPreparing !== stackPreparing;
    const styleChanged = previousMotionStyle !== islandStyle;
    if (styleChanged) styleLayoutMotion = !hard;
    if ((styleChanged && mode === "expanded") || editingChanged || (widgetChange && editing)) editLayoutMotion = true;
    else if (mode !== "expanded") editLayoutMotion = false;
    expandedLayoutMotion = previousMotionMode === "expanded" && mode === "expanded";
    const motion = featureChanged ? widgetLayoutMotion(featureExpanded, hard) : stackChanged ? widgetLayoutMotion(!stackPreparing, hard) : styleChanged && !hard
      ? { duration: ISLAND_MOTION.outwardDuration, easing: islandMorphEasing }
      : editingChanged || (widgetChange && editing)
      ? widgetEditMotion(hard)
      : widgetToggle || widgetChange
        ? widgetLayoutMotion(widgetToggle ? widgetsEnabled : widgetCount > previousWidgetCount, hard)
        : islandShapeMotion(previousMotionMode, mode, hard);
    previousWidgetCount = widgetCount;
    previousWidgetsEnabled = widgetsEnabled;
    previousEditing = editing;
    previousFeatureExpanded = featureExpanded;
    previousStackPreparing = stackPreparing;
    previousMotionStyle = islandStyle;
    previousMotionMode = mode;
    const revision = ++transitionRevision;
    shapeSettled = false;
    void Promise.all([
      widgetReveal.set(stackPreparing ? 0 : editing || widgetsEnabled ? 1 : 0, editing || hard ? {duration:0} : stackPreparing ? widgetLayoutMotion(false, hard) : motion),
      featureOpacity.set(featureExpanded ? 1 : 0, motion),
      widthMotion.set(target.width, motion),
      heightMotion.set(target.height, motion),
      radiusMotion.set(target.radius, motion),
      shoulderMotion.set(mode === "expanded" ? expandedEdgeShoulderRadius * expandedScale : collapsedEdgeShoulderRadius, motion),
      styleMorph.set(islandStyle === "edge" ? 1 : 0, motion),
    ]).then(() => {
      if (revision === transitionRevision) {
        shapeSettled = true;
        editLayoutMotion = false;
        styleLayoutMotion = false;
        compactShapeMotion = mode === "compact" || mode === "hover";
      }
    });
  });
  $effect(() => {
    if (mode !== "expanded" || (!panelVisible && page !== "files")) { page = "music"; menuScroll = 0; }
    else if (page !== "music" && page !== "system" && page !== "files" && !enabledTools.includes(page)) page = "music";
  });
  $effect(() => {
    const finished = timerFinished;
    if (!editing && finished && !wasTimerFinished && panelVisible && enabledTools.includes("timer")) void navigate("timer");
    else if (!finished && wasTimerFinished && page === "timer") void navigate("music");
    wasTimerFinished = finished;
  });
  $effect(() => { onPanelActivity?.(contextMenuOpen || editing || featureExpanded || stackPreparing || (mode === "expanded" && page !== "music")); onPageChange?.(page); });
  let renderedPage = $state<IslandPage>("music");
  let pageExiting = $state(false);
  $effect(() => {
    const next = page;
    if (mode !== "expanded" || !visible || !enableAnimations || reduceAnimations || prefersReducedMotion) { renderedPage = next; pageExiting = false; return; }
    if (untrack(() => renderedPage) === next) { pageExiting = false; return; }
    pageExiting = true;
    const timeout = setTimeout(() => {
      renderedPage = next; pageExiting = false;
      void tick().then(() => { if (!surfaceDisposed && page === next && mode === "expanded") pageRoot?.querySelector<HTMLButtonElement>(next === "music" ? ".feature-menu button" : ".page-back")?.focus({preventScroll:true}); });
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
    const gap = islandAnchorGap(expandedScale, styleProgress);
    const mix = (shown: number, hidden: number) => shown + (hidden - shown) * hideProgress;
    if (edge === "top") return `translateX(-50%) translateY(${mix(gap, 2 - size.height)}px)`;
    if (edge === "right") return `translateY(-50%) translateX(${mix(-gap, size.width - 2)}px)`;
    if (edge === "bottom") return `translateX(-50%) translateY(${mix(-gap, size.height - 2)}px)`;
    return `translateY(-50%) translateX(${mix(gap, 2 - size.width)}px)`;
  });
  const outwardProgress = $derived.by(() => {
    if (expandedLayoutMotion && mode === "expanded") return 1;
    if (compactShapeMotion) return 0;
    const horizontalEdge = edge === "top" || edge === "bottom";
    const compact = compactGeometry;
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
    notifyRegionChange({ geometry: size, radii: currentRadii, polygon: currentPolygon, extraRects: contextMenuOpen ? [{...contextMenuRect,radius:20}] : [], anchorGap: islandAnchorGap(expandedScale, styleProgress), settled: shapeSettled });
  });
  const expandedOpacity = $derived(outwardProgress);
  const musicTitleActive = $derived(visible && mode === "expanded" && page === "music" && expandedOpacity > .5);
  const sharedCover = $derived.by(() => {
    if (!coverLayouts) return null;
    const p = outwardProgress;
    const a = coverLayouts.compact, b = coverLayouts.expanded;
    const from = { x: compactOffset.x + a.x, y: compactOffset.y + a.y };
    const to = { x: expandedOffset.x + b.x, y: expandedOffset.y + b.y };
    return { x: from.x + (to.x - from.x) * p, y: from.y + (to.y - from.y) * p, width: a.width + (b.width - a.width) * p,
      dx: (from.x - to.x) * (1 - p), dy: (from.y - to.y) * (1 - p), radius: 42 + (20 - 42) * p };
  });
  const movingCover = $derived(Boolean(sharedCover && !shapeSettled && outwardProgress > 0 && outwardProgress < 1 && renderedPage === "music" && !idle && !timerActive && !showTime && enableAnimations && !reduceAnimations && !prefersReducedMotion));
  const sharedSpectrum = $derived.by(() => {
    if (!spectrumLayouts) return null;
    const a = spectrumLayouts.compact, b = spectrumLayouts.expanded, p = outwardProgress;
    return {
      x: (compactOffset.x + a.x) * (1 - p) + (expandedOffset.x + b.x) * p,
      y: (compactOffset.y + a.y) * (1 - p) + (expandedOffset.y + b.y) * p,
      width: a.width + (b.width - a.width) * p,
      height: a.height + (b.height - a.height) * p,
    };
  });
  const movingSpectrum = $derived(Boolean(showSpectrum && sharedSpectrum && movingCover));
  const compactOpacity = $derived(Math.min(1, Math.max(0, (0.08 - outwardProgress) / 0.08)));
  const surfaceBackground = $derived(mode === "expanded" ? background : "#000");
  const surfaceBorder = $derived(mode === "expanded" ? border : "1px solid transparent");
  const controlSelector = 'button,input,select,textarea,a,[role="slider"],[role="listbox"],[data-stop-toggle]';
  let backgroundPressed = $state(false);
  let gestureX = 0;
  let gestureY = 0;
  let suppressBackgroundClick = false;
  function startSurfaceGesture(event: PointerEvent) {
    gestureX = event.clientX; gestureY = event.clientY;
    suppressBackgroundClick = Boolean((event.target as HTMLElement).closest(controlSelector));
    backgroundPressed = !suppressBackgroundClick && !editing && interactive && event.button === 0 && event.isPrimary;
  }
  function moveSurfaceGesture(event: PointerEvent) {
    if (Math.hypot(event.clientX - gestureX, event.clientY - gestureY) > 5) {
      backgroundPressed = false;
      if (event.buttons) suppressBackgroundClick = true;
    }
  }
  function toggleFromSurface(event: MouseEvent | KeyboardEvent) {
    const target = event.target as HTMLElement;
    if (editing || !interactive || target.closest(controlSelector) || (event.detail > 0 && suppressBackgroundClick)) return;
    if (featureExpanded || stackPreparing) { closeFeature(); return; }
    onToggle?.();
  }

  function keyToggle(event: KeyboardEvent) {
    if (featureExpanded) { if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); closeFeature(); } return; }
    if ((event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) && interactive) { event.preventDefault(); void openContextMenu(); return; }
    if (event.key === "Escape" && editing) { event.preventDefault(); event.stopPropagation(); cancelEditing(); return; }
    if (event.key === "F2" && interactive) { event.preventDefault(); beginEditing(); return; }
    if (editing) return;
    if (event.key === "Escape" && mode === "expanded" && !event.defaultPrevented) { event.preventDefault(); event.stopPropagation(); goBack(); return; }
    if ((event.key === "Enter" || event.key === " ") && event.target === event.currentTarget) {
      event.preventDefault();
      onToggle?.();
    }
  }

  function closeContextMenu(restoreFocus = false) {
    contextMenuOpen = false;
    if (restoreFocus) surfaceRoot?.focus({preventScroll:true});
  }
  async function openContextMenu(event?: MouseEvent) {
    event?.preventDefault(); event?.stopPropagation();
    if (!interactive || !visible || mode === "hidden") return;
    const surface = surfaceRoot;
    const frame = surface?.closest<HTMLElement>(".island-frame");
    if (!surface || !frame) return;
    const a = surface.getBoundingClientRect(), f = frame.getBoundingClientRect();
    const sx = a.width / surface.offsetWidth || 1, sy = a.height / surface.offsetHeight || 1;
    const width = Math.min(224,(f.width - 16) / sx), height = 216;
    const x = Math.max((f.left-a.left)/sx + 8, Math.min(event ? (event.clientX-a.left)/sx : (size.width-width)/2,(f.right-a.left)/sx-width-8));
    let y = size.height + 8;
    if (a.top + (y+height)*sy > f.bottom-4) y = -height-8;
    y = Math.max((f.top-a.top)/sy+4,Math.min(y,(f.bottom-a.top)/sy-height-4));
    contextMenuRect = {x,y,width,height};
    contextMenuOpen = true;
    suppressBackgroundClick = true;
    await tick();
    contextMenuRoot?.querySelector<HTMLButtonElement>("button")?.focus({preventScroll:true});
  }
  function contextMenuKey(event: KeyboardEvent) {
    if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); closeContextMenu(true); }
    if (event.key === "Tab") closeContextMenu();
    if (["ArrowDown","ArrowUp","Home","End"].includes(event.key)) {
      event.preventDefault();
      const items = Array.from(contextMenuRoot?.querySelectorAll<HTMLButtonElement>("button") ?? []);
      const index = items.indexOf(document.activeElement as HTMLButtonElement);
      items[event.key === "Home" ? 0 : event.key === "End" ? items.length-1 : (index + (event.key === "ArrowUp" ? -1 : 1) + items.length)%items.length]?.focus();
    }
  }
  onMount(() => {
    const dismiss = (event: PointerEvent) => {
      if (!contextMenuOpen || contextMenuRoot?.contains(event.target as Node) || event.button === 2) return;
      closeContextMenu();
      event.preventDefault(); event.stopPropagation();
    };
    document.addEventListener("pointerdown",dismiss,true);
    return () => document.removeEventListener("pointerdown",dismiss,true);
  });

  function handleAnchorFocusOut(event: FocusEvent) {
    const anchor = event.currentTarget;
    const nextTarget = event.relatedTarget;
    if (!(anchor instanceof HTMLElement) || !(nextTarget instanceof Node) || !anchor.contains(nextTarget)) {
      onHoverChange?.(false);
    }
  }
</script>

<svelte:window onpointerup={() => backgroundPressed = false} onpointercancel={() => backgroundPressed = false} onpointermove={moveSurfaceGesture} onblur={() => {backgroundPressed = false;closeContextMenu();}} onkeydown={(event) => { if (featureExpanded && event.key === "Escape") { event.preventDefault(); closeFeature(); } else if (editing && event.key === "Escape") { event.preventDefault(); cancelEditing(); } }} />

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
    onmouseleave={() => {if (!contextMenuOpen) onHoverChange?.(false)}}
    onfocusin={() => onHoverChange?.(true)}
    onfocusout={handleAnchorFocusOut}
  >
  <svg class="shadow-filter" aria-hidden="true" width="0" height="0">
    <filter id="island-upper-shadow" x="-50%" y="-100%" width="200%" height="300%" color-interpolation-filters="sRGB">
      <feGaussianBlur in="SourceAlpha" stdDeviation="1.5" result="blur" />
      <feOffset in="blur" dy="-0.5" result="offset" />
      <feFlood flood-color="#000" flood-opacity="0.85" result="color" />
      <feComposite in="color" in2="offset" operator="in" />
    </filter>
  </svg>
  <div class="island-shadow" class:background-press={backgroundPressed} class:editing-surface={editing} aria-hidden="true">
    <div class="island-shadow-shape" style:width={`${size.width}px`} style:height={`${size.height}px`} style:border-radius={borderRadiusCss(currentRadii)} style:clip-path={resolvedClipPath}></div>
  </div>
  <div
    class="island-surface"
    use:volumeWheel
    class:background-press={backgroundPressed}
    bind:this={surfaceRoot}
    class:editing-surface={editing}
    class:expanded-surface={mode === "expanded"}
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
    oncontextmenu={openContextMenu}
  >
    {#if fileDragActive}
      <div class="file-drop-feedback" aria-live="polite">{mode === "expanded" ? "松开以分类文件" : "分类文件"}</div>
    {/if}
    {#if wheelVolumeDisplay !== null && visible && mode !== "hidden"}
      <div class="volume-indicator" class:compact-mode={mode !== "expanded"} class:vertical={edge === "left" || edge === "right"} role="meter" aria-label={t("currentVolume", { value: wheelVolumeDisplay })} aria-valuemin="0" aria-valuemax="100" aria-valuenow={wheelVolumeDisplay}>
        <div class="volume-meter" aria-hidden="true">
          <span class="volume-meter-fill" style:width={edge === "left" || edge === "right" ? undefined : `${wheelVolumeDisplay}%`} style:height={edge === "left" || edge === "right" ? `${wheelVolumeDisplay}%` : undefined}></span>
          {#each volumeHudMarks as mark}
            <i class:passed={mark <= wheelVolumeDisplay} style:left={edge === "left" || edge === "right" ? "50%" : `${mark}%`} style:bottom={edge === "left" || edge === "right" ? `calc(${mark}% - 26px)` : undefined}></i>
          {/each}
        </div>
      </div>
    {/if}
    {#if volumeProtectionPrompt && visible && mode !== "hidden"}
      <div class="volume-protection-prompt" role="dialog" aria-modal="true" aria-label={t("volumeProtection")} tabindex="-1" onkeydown={(event)=>event.stopPropagation()} onpointerdown={(event)=>event.stopPropagation()} onclick={(event)=>event.stopPropagation()}>
        <strong>{t("volumeProtectionReached", { value: volumeProtectionLimit })}</strong>
        <div><button type="button" onclick={(event)=>{event.stopPropagation();onCancelVolumeIncrease?.()}}>{t("volumeProtectionStop")}</button><button type="button" onclick={(event)=>{event.stopPropagation();onConfirmVolumeIncrease?.()}}>{t("volumeProtectionContinue")}</button></div>
      </div>
    {/if}
    {#if showDebugInfo}
      <div class="debug-overlay" aria-hidden="true">
        {#each debugLines as line}<span>{line}</span>{/each}
      </div>
    {/if}

    {#if visible && mode !== "hidden"}
    <div class="compact-layer edge-inset" bind:this={compactRoot} class:vertical={edge === "left" || edge === "right"} style={`opacity:${compactOpacity};--shoulder-inset:${compactInset}px;width:${compactLayout.width}px;height:${compactLayout.height}px;left:${compactOffset.x}px;top:${compactOffset.y}px`} aria-hidden={mode === "expanded" || expandedOpacity > .5} inert={mode === "expanded"}>
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
        <button class="cover compact-cover" class:shared-cover-source={movingCover} type="button" aria-label={t("openPlayer")} onclick={(e) => { e.stopPropagation(); onOpenPlayer?.(); }}>
          {#if media.albumArt}
            <span class="compact-disc">
              <CoverArt src={media.albumArt} />
            </span>
          {:else}
            <Music2 size={12} />
          {/if}
        </button>
        {#if showSpectrum}
          <div class="spectrum-endpoint" class:shared-spectrum-source={movingSpectrum}>
            <Spectrum active={!movingSpectrum && compactOpacity > .05} playing={media.isPlaying} mode={spectrumMode} reduceMotion={reduceAnimations} topColor={spectrumTopColor} bottomColor={spectrumBottomColor} values={previewSpectrum} />
          </div>
        {:else if media.isPlaying}
          <i class="playing-dot" style:background={spectrumTopColor}></i>
        {/if}
      {/if}
    </div>

    {/if}
          {#snippet stackWidget(kind: IslandWidgetKind)}
            {#if kind === "timer"}
              <TimerPanel status={timerStatus} remainingMs={timerRemainingMs} durationMs={timerDurationMs} finished={timerFinished} reduceMotion={reduceAnimations || !enableAnimations} onFinishedDismiss={onTimerFinishedDismiss} onStart={onTimerStart} onPause={onTimerPause} onResume={onTimerResume} onReset={onTimerReset} />
            {:else if kind === "volume"}
              <VolumePanel volume={systemAudio?.volumePercent ?? 0} muted={systemAudio?.muted ?? false} deviceId={systemAudio?.deviceId ?? ""} devices={audioDevices} switchingDevice={audioDeviceSwitching} reduceMotion={reduceAnimations || !enableAnimations} onVolume={onAudioVolume} onDevice={onAudioDevice} onExpand={() => openFeature("volume")} />
            {:else if kind === "system"}
              <SystemMonitor active={visible && mode === "expanded" && !editing} snapshot={monitorSnapshot} />
            {:else if kind === "calendar"}
              <CalendarWidget timeZone={clockTimeZone} onExpand={() => openFeature("calendar")} />
            {:else}
              <InfoPanel widget page={kind} time={clockText} timeZone={clockTimeZone} city={weatherCity} temperature={idleWeatherTemperature} code={idleWeatherCode} forecast={idleWeatherForecast} updatedAt={weatherUpdatedAt} loading={weatherLoading} failed={weatherFailed} reduceMotion={reduceAnimations || !enableAnimations || prefersReducedMotion} onSettings={onSettingsToggle} onExpandClock={() => openFeature("clock")} onExpandWeather={() => openFeature("weather")} />
            {/if}
          {/snippet}

    {#if visible && mode !== "hidden"}
    <div
      class="expanded-layer" class:vertical={edge === "left" || edge === "right"}
      style:opacity={expandedOpacity * (1 - featureOpacityValue)}
      style:width={`${expandedLayoutGeometry.width}px`}
      style:height={`${expandedLayoutGeometry.height}px`}
      style:left={`${expandedOffset.x}px`}
      style:top={`${expandedOffset.y}px`}
      style:transform={sharedCover && !reduceAnimations && enableAnimations && !prefersReducedMotion ? `translate(${sharedCover.dx}px,${sharedCover.dy}px)` : "none"}
      aria-hidden={mode !== "expanded" || expandedOpacity <= .5}
      inert={mode !== "expanded" || featureExpanded}
    >
      <div class="expanded-shell" class:composed={widgetCount > 0 || editing} bind:this={pageRoot} class:reduce-page-motion={reduceAnimations || !enableAnimations || prefersReducedMotion} style={`--widget-vertical-inset:${(edge === "left" || edge === "right") && (widgetCount > 0 || editing) ? expandedInset : 0}px;--widget-edge-inset:${(edge === "top" || edge === "bottom") && widgetCount > 0 ? expandedInset : 0}px;--editor-edge-inset:${edge === "top" || edge === "bottom" ? expandedInset : 0}px;--page-width:${expandedContentGeometry.width}px;--page-height:${expandedContentGeometry.height}px;--content-scale:${expandedScale};--shoulder-content-inset:${expandedInset}px`}>
        {#if panelVisible}
          <div class="top-tools" transition:slide={{duration:enableAnimations && !reduceAnimations && !prefersReducedMotion ? ISLAND_MOTION.outwardDuration : 0,easing:islandMorphEasing}}><FeatureMenu items={menuItems} toolbar activeId={page} scrollPosition={menuScroll} onScroll={(value) => menuScroll = value} onSelect={selectTool} /></div>
        {/if}
        <div class="page-body" class:exiting={pageExiting} class:with-toolbar={panelVisible} inert={pageExiting}>
        {#if renderedPage === "music"}
        <WidgetLayout widget={stackWidget} layout={editing ? draftLayout : configuredLayout} enabled={widgetsEnabled} reveal={widgetRevealValue} animate={enableAnimations && !reduceAnimations && !prefersReducedMotion} styleMorphing={styleLayoutMotion} {editing} busy={savingLayout} error={layoutError} musicWidth={expandedLength} onChange={(layout) => draftLayout = layout} onSave={() => void saveLayout()} onCancel={cancelEditing}>
          <section class="music-pane" use:musicStackGesture>
            <div class="top-row">
            <button class="cover expanded-cover" class:shared-cover-source={movingCover} type="button" aria-label={t("openPlayer")} disabled={idle} onclick={(e) => { e.stopPropagation(); onOpenPlayer?.(); }}>
              {#if (idle ? lastPlayedMedia?.albumArt : media.albumArt)}<CoverArt src={(idle ? lastPlayedMedia?.albumArt : media.albumArt) || ""} />{:else}<Music2 size={30} />{/if}
            </button>
            <div class="metadata">
              <MarqueeTitle text={(idle ? lastPlayedMedia?.title : media.title) || t("waitingPlayback")} active={musicTitleActive} reduceMotion={reduceAnimations || !enableAnimations || prefersReducedMotion} />
              <span title={idle ? lastPlayedMedia?.artist : media.artist}>{(idle ? lastPlayedMedia?.artist || lastPlayedMedia?.sourceDisplay : media.artist) || t("unknownArtist")}</span>
            </div>
            {#if showSpectrum}
              <div class="spectrum-endpoint" class:shared-spectrum-source={movingSpectrum}>
                <Spectrum active={!movingSpectrum && expandedOpacity > .05} playing={!idle && media.isPlaying} mode={spectrumMode} reduceMotion={reduceAnimations} topColor={spectrumTopColor} bottomColor={spectrumBottomColor} scale={2.5} values={previewSpectrum} />
              </div>
            {/if}
            </div>

            <div class="progress-block">
              <MediaProgress position={idle ? 0 : position} duration={idle ? 0 : media.durationMs} progressKnown={media.progressKnown !== false} estimated={media.neteaseFeatures?.progress.source === "estimate"} seekable={!idle && Boolean(media.capabilities?.seek)} {seekBusy} {seekError} {onSeek} />
            </div>

            <div class="control-row" class:system-netease={media.source === "netease" && media.neteaseFeatures?.mode.source !== "cdp"}>
              <div class="utility-controls leading-controls" aria-label={t("playbackControls")}>
                {#if !idle && media.neteaseFeatures?.mode.source === "cdp"}
                  {#key `${media.nativeEpoch}:${media.sourceDisplay}`}<NeteaseConnectedModeControl {media} live={Boolean(onMediaAction)}/>{/key}
                {:else if media.capabilities?.shuffle || media.capabilities?.repeat}
                  {#key media.source}<SmtcModeControls {media} live={Boolean(onMediaAction)}/>{/key}
                {:else}
                  <span class="control-mark" aria-hidden="true"><Shuffle size={34} strokeWidth={2} /></span>
                {/if}
                {#if !idle && media.source === "netease"}
                  {#key `${media.nativeEpoch}:${media.nativeTrackGeneration}:${media.title}:${media.artist}`}
                    <NeteaseLikeControl title={media.title} artist={media.artist} cacheKey={`${media.nativeEpoch}:${media.nativeTrackGeneration}`} available={media.neteaseFeatures?.like.controllable === true} live={Boolean(onMediaAction)} />
                  {/key}
                {/if}
              </div>
              <div class="transport-controls" aria-label={t("playbackControls")}>
                <button type="button" aria-label={t("previous")} disabled={idle || media.capabilities?.previous === false} onclick={(e) => { e.stopPropagation(); animateSkip(e, -1); onMediaAction?.("prev"); }}><SkipBack size={36} fill="currentColor" /></button>
                <button type="button" class="play" aria-label={idle ? (resumingLastTrack ? t("openingPlayer") : t("resumeMusic")) : media.isPlaying ? t("pause") : t("play")} disabled={idle ? !lastPlayedMedia || !canResumeLastTrack || resumingLastTrack : media.capabilities?.playPause === false} onclick={(e) => { e.stopPropagation(); if (idle) onResumeLastTrack?.(); else onMediaAction?.("play_pause"); }}>
                  <PlayPauseIcon playing={!idle && media.isPlaying} size={50} reduceMotion={reduceAnimations || !enableAnimations} />
                </button>
                <button type="button" aria-label={t("next")} disabled={idle || media.capabilities?.next === false} onclick={(e) => { e.stopPropagation(); animateSkip(e, 1); onMediaAction?.("next"); }}><SkipForward size={36} fill="currentColor" /></button>
              </div>
              <div class="utility-controls trailing-controls">
                <button class="layout-toggle" class:layout-active={layoutActive} type="button" aria-label={t(layoutActive ? "disableWidgetLayout" : "enableWidgetLayout")} title={t(layoutActive ? "disableWidgetLayout" : "enableWidgetLayout")} aria-pressed={layoutActive} disabled={togglingWidgets || editing} onclick={(e) => {e.stopPropagation();void toggleWidgets();}}><PanelsTopLeft size={34} strokeWidth={1.8}/></button>
                {#if widgetToggleError}<span class="layout-toggle-error" role="status">{t("settingsSaveFailed")}</span>{/if}
                <button type="button" aria-label={t("toggleFloating")} onclick={(e) => { e.stopPropagation(); onToggleFloating?.(); }}><Airplay size={34} strokeWidth={1.8} /></button>
              </div>
            </div>
        </section>
        </WidgetLayout>

        {:else}
          <section class="navigation-page" class:file-sort-page={renderedPage === "files"}>
            <header class="page-header">
              <button type="button" class="page-back" aria-label={t("back")} onclick={goBack}><ArrowLeft size={17} /></button>
              <span>{t(toolLabels[renderedPage])}</span>
            </header>
            {#key renderedPage}
              <div class="page-content">
                {#if renderedPage === "files"}
                  <div class="file-sort-panel" role="group" aria-label={t("fileSort")}>
                    {#if pendingSort}
                      <div class="file-confirm" role="group" aria-label="确认分类文件夹">
                        <strong>确认分类文件夹？</strong>
                        <p>文件夹及其子文件夹中的文件将被移动到所选位置，同批拖入的文件也会一起分类。原文件夹会保留。</p>
                        <div class="file-confirm-paths" title={pendingSort.folders.join("\n")}>
                          {#each pendingSort.folders as folder}<div>{folder}</div>{/each}
                        </div>
                        <div class="file-confirm-target" title={$settingsStore.fileSortDirectory ?? ""}>保存到：{$settingsStore.fileSortDirectory}</div>
                        <div class="file-confirm-actions">
                          <button type="button" class="file-confirm-cancel" onclick={cancelPendingSort}>取消</button>
                          <button type="button" class="file-confirm-accept" onclick={confirmPendingSort}>确认分类</button>
                        </div>
                      </div>
                    {:else}
                    <div class="file-field">
                      <span class="file-field-label">保存位置</span>
                      <button class="file-directory" type="button" onclick={chooseSortDirectory} title={$settingsStore.fileSortDirectory ?? "选择目标文件夹"}>
                        <FolderOpen size={24}/><span>{$settingsStore.fileSortDirectory || "选择目标文件夹"}</span><span class="file-directory-action">更改</span>
                      </button>
                    </div>
                    <div class="file-field">
                      <span class="file-field-label">日期分类</span>
                      <div class="file-date" role="group" aria-label="日期分类">
                        {#each [["none", "不按日期"], ["year", "按年份"], ["month", "年 / 月"]] as [value, label]}
                          <button type="button" class:active={$settingsStore.fileSortDate === value} aria-pressed={$settingsStore.fileSortDate === value} onclick={() => setSortDate(value as "none" | "year" | "month")}>{label}</button>
                        {/each}
                      </div>
                    </div>
                    {#if sortBusy && sortProgress.total > 1}
                      <div class="file-progress">
                        <div class="file-progress-label"><span>正在分类</span><span>{sortProgress.completed} / {sortProgress.total}</span></div>
                        <div class="file-progress-track" role="progressbar" aria-label="文件分类进度" aria-valuemin="0" aria-valuemax={sortProgress.total} aria-valuenow={sortProgress.completed}>
                          <div class="file-progress-fill" style:width={`${sortProgress.completed / sortProgress.total * 100}%`}></div>
                        </div>
                      </div>
                    {/if}
                    <!-- svelte-ignore a11y_no_noninteractive_tabindex: scrollable results need keyboard access -->
                    <div class="file-results" role="region" aria-label="分类结果" aria-live="polite" tabindex="0">
                      {#if sortError}<span class="file-error">{sortError}</span>
                      {:else if checkingSortInputs}<span>正在检查拖入的项目…</span>
                      {:else if sortBusy}<span>{sortProgress.total === 0 ? "正在扫描文件夹…" : "正在分类文件…"}</span>
                      {:else if sortResults.length}
                        {#each sortResults as item}
                          <div class="file-result" title={`${item.source}${item.destination ? ` → ${item.destination}` : `：${item.message}`}`}>
                            <span class="file-result-status" class:failed={item.status === "failed"}>{item.status === "moved" ? "已移动" : item.status === "skipped" ? "已跳过" : "失败"}</span>
                            <span class="file-result-detail">{item.source.split(/[\\/]/).pop()}{item.message ? `：${item.message}` : ""}</span>
                          </div>
                        {/each}
                      {:else}<span>将文件或文件夹拖到灵动岛即可自动分类</span>{/if}
                    </div>
                    {/if}
                  </div>
                {:else if renderedPage === "timer"}
                  <TimerPanel status={timerStatus} remainingMs={timerRemainingMs} durationMs={timerDurationMs} finished={timerFinished} reduceMotion={reduceAnimations || !enableAnimations} onFinishedDismiss={onTimerFinishedDismiss} onStart={onTimerStart} onPause={onTimerPause} onResume={onTimerResume} onReset={onTimerReset} />
                {:else if renderedPage === "volume"}
                  <VolumePanel volume={systemAudio?.volumePercent ?? 0} muted={systemAudio?.muted ?? false} deviceId={systemAudio?.deviceId ?? ""} devices={audioDevices} switchingDevice={audioDeviceSwitching} reduceMotion={reduceAnimations || !enableAnimations} onVolume={onAudioVolume} onDevice={onAudioDevice} onExpand={() => openFeature("volume")} />
                {:else if renderedPage === "system"}
                  <SystemMonitor active={visible && mode === "expanded"} snapshot={monitorSnapshot} />
                {:else if renderedPage === "clock" || renderedPage === "weather"}
                  <InfoPanel page={renderedPage} time={clockText} timeZone={clockTimeZone} city={weatherCity} temperature={idleWeatherTemperature} code={idleWeatherCode} forecast={idleWeatherForecast} updatedAt={weatherUpdatedAt} loading={weatherLoading} failed={weatherFailed} reduceMotion={reduceAnimations || !enableAnimations || prefersReducedMotion} onSettings={onSettingsToggle} onExpandClock={() => openFeature("clock")} onExpandWeather={() => openFeature("weather")} />
                {/if}
              </div>
            {/key}
          </section>
        {/if}
        </div>
      </div>
    </div>
    {/if}

    {#if visible && mode === "expanded" && (featureExpanded || featureOpacityValue > 0)}
      <div class="feature-extension" style:opacity={featureOpacityValue} style:width={`${featureGeometry.width}px`} style:height={`${featureGeometry.height}px`} style:left={`${featureOffset.x}px`} style:top={`${featureOffset.y}px`} inert={!featureExpanded}>
        <div class="feature-extension-content" style={`zoom:${expandedScale};width:${featureBaseGeometry.width}px;height:${featureBaseGeometry.height}px;--extension-inline-inset:${edge === "top" || edge === "bottom" ? expandedInset : 0}px;--extension-block-inset:${edge === "left" || edge === "right" ? expandedInset : 0}px`}>
          {#if featureKind === "volume"}<AppVolumeMixer onBack={backFromFeature} onMetrics={updateVolumeFeatureMetrics} />{:else if featureKind === "calendar"}<CalendarExtension timeZone={clockTimeZone} reduceMotion={reduceAnimations || !enableAnimations || prefersReducedMotion} onBack={backFromFeature} />{:else if featureKind === "clock"}<WorldClockExtension timeZone={clockTimeZone} onBack={backFromFeature} />{:else if featureKind === "weather"}<WeatherExtension onBack={backFromFeature} />{:else}<FeatureStack widget={stackWidget} initial={stackSelection} onSelect={(kind) => stackSelection = kind} reduceMotion={reduceAnimations || !enableAnimations || prefersReducedMotion} onBack={closeFeature} />{/if}
        </div>
      </div>
    {/if}

    {#if visible && mode !== "hidden" && sharedCover}
      <div class="cover moving-cover" class:in-motion={movingCover} aria-hidden="true" style:transform={`translate(${sharedCover.x}px,${sharedCover.y}px) scale(${sharedCover.width / 84})`} style:border-radius={`${sharedCover.radius}px`}>
        {#if media.albumArt}<CoverArt src={media.albumArt} />{:else}<Music2 size={30} />{/if}
      </div>
    {/if}
    {#if visible && mode !== "hidden" && sharedSpectrum}
      <div class="moving-spectrum" class:in-motion={movingSpectrum} aria-hidden="true" style:transform={`translate(${sharedSpectrum.x}px,${sharedSpectrum.y}px) scale(${sharedSpectrum.width / 42},${sharedSpectrum.height / 45})`}>
        <Spectrum active={movingSpectrum} playing={media.isPlaying} mode={spectrumMode} reduceMotion={reduceAnimations} topColor={spectrumTopColor} bottomColor={spectrumBottomColor} scale={2.5} values={previewSpectrum} />
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

  {#if contextMenuOpen}
    <div class="island-context-menu" bind:this={contextMenuRoot} role="menu" aria-label={t("dynamicIsland")} tabindex="-1" data-stop-toggle style:left={`${contextMenuRect.x}px`} style:top={`${contextMenuRect.y}px`} style:width={`${contextMenuRect.width}px`} onkeydown={contextMenuKey} oncontextmenu={(event) => event.preventDefault()}>
      <button data-layout-edit-control role="menuitem" onclick={beginEditing}><Pencil size={19}/><span>{t("widgetEdit")}</span></button>
      <button data-layout-edit-control role="menuitem" onclick={() => {closeContextMenu();onSettingsToggle?.()}}><Settings size={19}/><span>{t("settingsTool")}</span></button>
      <button data-layout-edit-control role="menuitemcheckbox" aria-checked={keepIslandExpanded} onclick={() => {void onToggleKeepIslandExpanded?.()}}><span class="menu-check" aria-hidden="true">{keepIslandExpanded ? "✓" : ""}</span><span>{t("keepIslandExpanded")}</span></button>
      <button data-layout-edit-control role="menuitem" onclick={() => {closeContextMenu();onHideForTenSeconds?.()}}><EyeOff size={19}/><span>{t("hideTool")}</span></button>
    </div>
  {/if}
  </div>
</div>

<style>
  .layout-toggle{color:#747d89}.layout-toggle.layout-active{color:#f5f5f7}.layout-toggle-error{position:absolute;right:0;bottom:100%;padding:8px 12px;border-radius:12px;background:#161616;color:#eee;font:12px/1.4 var(--app-font);white-space:nowrap;z-index:5}
  .island-context-menu{pointer-events:auto}
  .island-context-menu{position:absolute;z-index:30;height:216px;box-sizing:border-box;padding:7px;background:#080808;color:#f5f5f7;border:1px solid #ffffff24;border-radius:20px;box-shadow:0 12px 32px #0006;font:500 14px/1.2 var(--app-font,system-ui);animation:menu-reveal 140ms ease-out;transform-origin:top center}
  .island-context-menu button{display:flex;align-items:center;gap:12px;width:100%;height:48px;box-sizing:border-box;padding:0 14px;background:transparent;border:0;border-radius:13px;color:inherit;font:inherit;text-align:left;cursor:pointer}.island-context-menu button:hover,.island-context-menu button:focus-visible{background:#ffffff14;outline:0}.island-context-menu button:active{background:#ffffff20}.island-context-menu button :global(svg){flex:none;color:#b5b5bd}
  .island-context-menu .menu-check{display:flex;align-items:center;justify-content:center;flex:none;width:19px;height:19px;color:#b5b5bd;font-size:16px}
  @keyframes menu-reveal{from{opacity:0;transform:translateY(-5px)}to{opacity:1;transform:translateY(0)}}
  @media(prefers-reduced-motion:reduce){.island-context-menu{animation:none}}
  .spectrum-endpoint{display:flex;align-items:center;flex:none}
  .shared-spectrum-source{visibility:hidden}
  .moving-spectrum{position:absolute;left:0;top:0;z-index:3;visibility:hidden;transform-origin:top left;pointer-events:none}
  .moving-spectrum.in-motion{visibility:visible;will-change:transform}
  .cover.moving-cover{position:absolute;left:0;top:0;z-index:3;width:84px;height:84px;visibility:hidden;transform-origin:top left;pointer-events:none}
  .cover.moving-cover.in-motion{visibility:visible;will-change:transform}
  .cover.shared-cover-source{visibility:hidden}
  .island-frame{position:relative;width:100%;height:100%;flex:none;overflow:visible;box-sizing:border-box}.island-frame.simulate-hidden{overflow:hidden}.surface-anchor{position:absolute;z-index:1}.edge-top .surface-anchor{top:0;left:50%}.edge-right .surface-anchor{top:50%;right:0}.edge-bottom .surface-anchor{bottom:0;left:50%}.edge-left .surface-anchor{top:50%;left:0}
  .file-drop-feedback{position:absolute;inset:0;z-index:20;display:grid;place-items:center;background:#000;color:#91caff;font:600 12px var(--app-font);pointer-events:none;outline:1px solid #91caff;outline-offset:-2px}
  .file-confirm{display:flex;flex-direction:column;gap:10px;min-width:0;min-height:0;height:100%;font-size:18px;line-height:1.35}
  .file-confirm strong{font-size:21px;color:#fff}.file-confirm p{margin:0;color:rgba(255,255,255,.78)}
  .file-confirm-paths{min-height:0;max-height:65px;overflow:auto;color:#fff;overflow-wrap:anywhere;scrollbar-width:thin;scrollbar-color:rgba(255,255,255,.32) transparent}
  .file-confirm-target{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:rgba(255,255,255,.65);font-size:16px}
  .file-confirm-actions{display:flex;gap:10px;margin-top:auto}.file-confirm-actions button{flex:1;min-height:42px;border-radius:12px;background:rgba(255,255,255,.1)}
  .file-sort-panel{display:flex;flex-direction:column;gap:16px;width:100%;height:100%;min-width:0;min-height:0;color:#fff;font:500 19px/1.35 var(--app-font,system-ui)}
  .file-field{display:flex;flex-direction:column;gap:7px;min-width:0;flex:none}
  .file-field-label{font-size:16px;color:rgba(255,255,255,.62)}
  .file-sort-panel button{font:inherit;color:inherit;border:0;cursor:pointer}
  .file-directory{display:flex;align-items:center;gap:12px;width:100%;min-width:0;min-height:52px;text-align:left;background:rgba(255,255,255,.08);border-radius:12px;padding:10px 14px}
  .file-directory :global(svg){flex:none;color:rgba(255,255,255,.8)}
  .file-directory span:not(.file-directory-action){overflow:hidden;text-overflow:ellipsis;white-space:nowrap;flex:1;min-width:0}
  .file-directory .file-directory-action{flex:none;color:#91caff;font-size:17px}
  .file-date{display:flex;gap:8px}
  .file-date button{flex:1;min-width:0;min-height:44px;border-radius:12px;padding:8px 6px;background:rgba(255,255,255,.07);white-space:nowrap}
  .file-date button.active,.file-date button.active:hover,.file-date button.active:focus-visible{color:#fff;background:hsl(223 70% 33%)}
  .file-sort-panel button:hover,.file-sort-panel button:focus-visible{background:rgba(255,255,255,.16)}
  .file-sort-panel button:focus-visible,.file-results:focus-visible{outline:2px solid #91caff;outline-offset:2px}
  .file-progress{flex:none;display:flex;flex-direction:column;gap:7px}
  .file-progress-label{display:flex;justify-content:space-between;color:rgba(255,255,255,.72);font-size:16px;font-variant-numeric:tabular-nums}
  .file-progress-track{height:7px;overflow:hidden;border-radius:999px;background:rgba(255,255,255,.19)}
  .file-progress-fill{height:100%;border-radius:inherit;background:#fff;transition:width 240ms cubic-bezier(.23,1,.32,1)}
  .reduce-page-motion .file-progress-fill{transition:none}
  .file-results{flex:1;min-height:44px;overflow:auto;color:rgba(255,255,255,.72);font-size:17px;line-height:1.45;overflow-wrap:anywhere;scrollbar-width:thin;scrollbar-color:rgba(255,255,255,.32) transparent}
  .file-results::-webkit-scrollbar{width:6px;height:6px}
  .file-results::-webkit-scrollbar-track{background:transparent}
  .file-results::-webkit-scrollbar-thumb{border-radius:999px;background:rgba(255,255,255,.32)}
  .file-results::-webkit-scrollbar-thumb:hover{background:rgba(255,255,255,.55)}
  .file-results::-webkit-scrollbar-button{display:none;width:0;height:0}
  .file-result{display:flex;align-items:flex-start;gap:10px;padding:3px 0}.file-result-status{flex:none;color:#91caff}.file-result-status.failed,.file-error{color:#ffb4ab}.file-result-detail{min-width:0;overflow-wrap:anywhere}
  .file-sort-page .page-header{height:36px;margin-bottom:16px;font-size:20px;font-weight:600;color:#fff}
  .file-sort-page .page-back{width:36px;height:36px}
  .file-sort-page .page-content{align-items:stretch}
  .island-surface{position:relative;z-index:1;overflow:hidden;flex:none;box-sizing:border-box;color:#fff;font-family:var(--app-font);text-rendering:optimizeLegibility;font-synthesis:none;contain:layout style;transition:transform 140ms cubic-bezier(.23,1,.32,1),box-shadow 180ms cubic-bezier(.23,1,.32,1)}
  .shadow-filter{position:absolute;pointer-events:none}
  .island-shadow{position:absolute;inset:0;z-index:0;filter:url(#island-upper-shadow);pointer-events:none;transition:transform 140ms cubic-bezier(.23,1,.32,1)}
  .island-shadow-shape{background:#000}
  .floating-style .surface-anchor:active .island-shadow.background-press:not(.editing-surface){transform:scale(.97) translateZ(0)}
  .island-surface:focus-visible{outline:none}
  .volume-indicator{position:absolute;z-index:40;top:12px;left:50%;display:flex;align-items:center;width:min(320px,calc(100% - 32px));height:48px;padding:0 12px;box-sizing:border-box;border:1px solid rgba(255,255,255,.1);border-radius:14px;background:rgba(14,15,18,.96);transform:translateX(-50%);pointer-events:none;box-shadow:0 6px 18px rgba(0,0,0,.28)}
  .volume-indicator.compact-mode{inset:0;display:grid;width:100%;height:100%;padding:3px 18px;border:0;border-radius:inherit;background:rgba(0,0,0,.97);transform:none;box-shadow:none}
  .volume-meter{position:relative;width:100%;height:24px;overflow:hidden;border-radius:2px}
  .volume-meter-fill{position:absolute;top:0;bottom:0;left:0;max-width:100%;background:#193a8e;transition:width 90ms linear,height 90ms linear}
  .volume-meter i{position:absolute;top:0;bottom:0;display:block;width:3px;transform:translateX(-50%);background:#fff}
  .volume-meter i::after{position:absolute;top:0;bottom:0;left:0;width:3px;border-radius:1px;background:#fff;content:""}
  .volume-meter i.passed{opacity:0}
  .volume-indicator.compact-mode .volume-meter{height:100%}
  .volume-indicator.vertical.compact-mode{padding:3px 0}
  .volume-indicator.vertical.compact-mode .volume-meter{width:56px;height:100%}
  .volume-indicator.vertical.compact-mode .volume-meter-fill{top:auto;right:0;bottom:0;left:0;width:100%;max-height:100%}
  .volume-indicator.vertical.compact-mode .volume-meter i{top:auto;right:0;bottom:0;left:50%;width:3px;height:52px;transform:translateX(-50%)}
  .volume-indicator.vertical.compact-mode .volume-meter i::after{top:0;right:0;bottom:0;left:0;width:3px;height:100%}
  .volume-protection-prompt{position:absolute;z-index:41;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;padding:16px;box-sizing:border-box;border-radius:inherit;background:rgba(18,19,22,.96);color:#fff;text-align:center;font:500 13px/1.4 var(--app-font)}
  .volume-protection-prompt>div{display:flex;gap:8px}
  .volume-protection-prompt button{padding:7px 12px;border:1px solid rgba(255,255,255,.2);border-radius:9px;background:#292b31;color:#fff;font:600 12px/1.2 var(--app-font);cursor:pointer}
  .volume-protection-prompt button:last-child{background:#f2f3f5;color:#111}

  .floating-style .island-surface.background-press:active{transform:scale(.97) translateZ(0)}.compact-layer,.expanded-layer{position:absolute;z-index:1;box-sizing:border-box;pointer-events:none}
  .compact-layer{inset:0;display:flex;align-items:center;justify-content:space-between;padding:0 8px 0 4px}.compact-layer.edge-inset:not(.vertical){padding-left:calc(4px + var(--shoulder-inset));padding-right:calc(8px + var(--shoulder-inset))}.compact-layer.vertical{flex-direction:column;padding:4px 0 8px}.compact-layer.edge-inset.vertical{padding-top:calc(4px + var(--shoulder-inset));padding-bottom:calc(8px + var(--shoulder-inset))}.compact-layer button,.compact-layer :global(canvas){pointer-events:auto}.time-display{width:100%;text-align:center;color:rgba(255,255,255,.8);font:500 12px/1 var(--app-font);letter-spacing:.05em;font-variant-numeric:tabular-nums;user-select:none}.cover{position:relative;display:grid;place-items:center;flex:none;padding:0;overflow:hidden;color:rgba(255,255,255,.3);background:rgba(255,255,255,.06);border:0;cursor:pointer;user-select:none}.compact-cover{width:20px;height:20px;border-radius:50%;top:-1px}.expanded-cover{width:84px;height:84px;border-radius:12px;box-shadow:0 8px 22px rgba(0,0,0,.35);outline:1px solid rgba(255,255,255,.1)}.playing-dot{width:3px;height:3px;border-radius:50%}
  .compact-disc{position:relative;display:block;width:100%;height:100%;transform-origin:center}
  .idle-compact-dashboard{display:flex;align-items:center;justify-content:space-between;gap:8px;width:100%;padding:0 7px;color:rgba(255,255,255,.92);user-select:none}.idle-clock{display:flex;align-items:center;height:1em;font:600 12px/1 var(--app-font);font-variant-numeric:tabular-nums;letter-spacing:.02em}.idle-weather{display:flex;align-items:center;gap:4px;font:600 12px/1 var(--app-font);font-variant-numeric:tabular-nums}.idle-weather strong{display:flex;align-items:center;height:1em;font:inherit}.idle-weather :global(svg){flex:none;color:rgba(255,255,255,.82)}.idle-compact-dashboard.vertical{flex-direction:column;padding:7px 0;gap:10px}.idle-compact-dashboard.vertical .idle-weather{flex-direction:column}
  .debug-overlay{position:absolute;z-index:4;top:4px;left:50%;display:flex;gap:5px;max-width:calc(100% - 12px);padding:2px 6px;border-radius:5px;transform:translateX(-50%);overflow:hidden;color:#4ade80;background:rgba(0,0,0,.75);font:500 8px/1.3 var(--app-font);white-space:nowrap;pointer-events:none}.debug-overlay span{overflow:hidden;text-overflow:ellipsis}
  .expanded-layer{inset:0;width:100%;height:100%;padding:0;transform-origin:left top;will-change:transform,opacity}.expanded-layer.vertical{transform-origin:center top}.expanded-layer[aria-hidden="true"]{pointer-events:none}.expanded-layer[aria-hidden="false"]{pointer-events:auto}.expanded-shell{display:grid;width:100%;height:var(--expanded-content-height,210px);grid-template-columns:var(--music-pane-width,600px)}.music-pane{position:relative;width:var(--music-pane-width,600px);height:var(--expanded-content-height,210px);display:flex;flex-direction:column;justify-content:space-between;padding:14px 32px 10px;box-sizing:border-box}.top-row{display:flex;align-items:center;gap:16px;margin-bottom:8px;min-height:84px}.metadata{min-width:0;flex:1;font-family:var(--app-font);user-select:none}.metadata span{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.metadata span{font-size:11px;line-height:1.2;font-weight:500;color:rgba(255,255,255,.68);letter-spacing:.005em}
  .progress-block{width:100%;margin-bottom:10px}
  .control-row{position:relative;display:grid;grid-template-columns:minmax(0,1fr) auto minmax(0,1fr);align-items:center;width:100%;height:58px}.cover:active{transform:scale(.94)}button:focus-visible{outline:2px solid #fff;outline-offset:2px}
  .compact-timer{width:100%;height:100%;display:flex;align-items:center;justify-content:center;gap:7px;padding:0 9px;border:0;color:#f28b31;background:transparent;cursor:pointer;font-family:var(--app-font);user-select:none}.compact-timer span{min-width:42px;font-size:12px;font-weight:600;line-height:1;letter-spacing:.015em;font-variant-numeric:tabular-nums;text-align:center;white-space:nowrap}.compact-timer.finished span{min-width:0;font-size:11px}.compact-timer.paused{color:rgba(242,139,49,.62)}.timer-pause{width:7px;height:8px;display:flex;align-items:center;justify-content:center;gap:2px}.timer-pause b{display:block;width:2px;height:7px;border-radius:1px;background:currentColor}.compact-timer.urgent{color:#ff765f}
  .compact-timer{position:absolute;inset:0;display:block;overflow:hidden;padding:0;border-radius:inherit;color:#f4f7ff;background:transparent;box-shadow:none;cursor:default!important}.compact-timer:hover,.compact-timer:focus-visible{background:transparent!important;box-shadow:none!important;outline:0!important}.compact-timer .timer-summary{position:absolute;inset:0 14px 4px;display:flex;align-items:center;justify-content:space-between;gap:12px;min-width:0}.compact-timer .timer-leading{display:flex;align-items:center;gap:7px;min-width:0;color:rgba(245,248,255,.9)}.compact-timer .timer-leading :global(svg){flex:none;color:#70aaff}.compact-timer .timer-time{min-width:0;overflow:hidden;color:#f5f7fb;font-size:13px;font-weight:600;line-height:1;font-variant-numeric:tabular-nums;letter-spacing:.01em;text-align:right;text-overflow:ellipsis;white-space:nowrap}.compact-timer .timer-progress-track{position:absolute;right:14px;bottom:5px;left:14px;height:2px;overflow:hidden;border-radius:999px;background:rgba(115,143,183,.2)}.compact-timer .timer-progress-fill{display:block;height:100%;max-width:100%;border-radius:inherit;background:#193a8e;box-shadow:0 0 7px rgba(25,58,142,.68);transition:width 350ms linear,background-color 180ms ease}.compact-timer .timer-progress-fill.paused{background:linear-gradient(90deg,#6d8fb9,#a7c5ed);box-shadow:none}.compact-timer .timer-progress-fill.finished{background:linear-gradient(90deg,#42d99a,#8af0c0)}.compact-timer.paused{color:#f4f7ff}.compact-timer.urgent{box-shadow:inset 1px 0 0 rgba(255,111,102,.58),inset -1px 0 0 rgba(255,111,102,.58),inset 0 0 12px rgba(255,74,60,.09)}.compact-timer.urgent .timer-leading :global(svg){color:#ff8c7c}.compact-timer .timer-pause{flex:none;width:7px;height:8px;gap:2px;color:#a8c5e9}.compact-timer .timer-pause b{width:2px;height:7px;border-radius:1px;background:currentColor}
  .compact-layer.vertical .compact-timer .timer-summary{inset:14px 4px;flex-direction:column;justify-content:center;gap:10px}.compact-layer.vertical .compact-timer .timer-leading{flex-direction:column;gap:5px}.compact-layer.vertical .compact-timer .timer-time{max-width:100%;font-size:11px;text-align:center}.compact-layer.vertical .compact-timer .timer-progress-track{top:14px;right:auto;bottom:14px;left:5px;width:2px;height:auto}.compact-layer.vertical .timer-progress-fill{width:100%!important;height:var(--timer-progress-height,0%)}
  .attached-style.edge-top .music-pane,.attached-style.edge-bottom .music-pane{--music-inline-inset:calc(40px + var(--shoulder-content-inset,0px))}
  .music-pane{padding-top:14px;padding-bottom:10px}
  .top-row{gap:16px;margin-bottom:8px}
  @media (hover:hover) and (pointer:fine){}
  @media (prefers-reduced-motion:reduce){}
  .studio-preview-highlight{position:absolute;z-index:20;inset:0;overflow:hidden;border-radius:inherit;pointer-events:none}.studio-preview-highlight::before,.studio-preview-highlight::after{content:"";position:absolute;pointer-events:none}.highlight-shape::after{inset:2px;border:2px solid rgba(167,139,250,.9);border-radius:inherit;box-shadow:0 0 12px rgba(167,139,250,.42),inset 0 0 7px rgba(167,139,250,.18)}.highlight-edge::before{background:#60a5fa;box-shadow:0 0 9px rgba(96,165,250,.8)}.edge-top .highlight-edge::before,.edge-bottom .highlight-edge::before{left:18px;right:18px;height:2px}.edge-top .highlight-edge::before{top:2px}.edge-bottom .highlight-edge::before{bottom:2px}.edge-left .highlight-edge::before,.edge-right .highlight-edge::before{top:18px;bottom:18px;width:2px}.edge-left .highlight-edge::before{left:2px}.edge-right .highlight-edge::before{right:2px}.highlight-position::after{width:8px;height:8px;border:2px solid #fff;border-radius:50%;background:#60a5fa;box-shadow:0 0 0 3px rgba(96,165,250,.2),0 0 10px rgba(96,165,250,.92)}.edge-top .highlight-position::after,.edge-bottom .highlight-position::after{left:calc(var(--preview-position) * 1% - 6px)}.edge-top .highlight-position::after{top:-3px}.edge-bottom .highlight-position::after{bottom:-3px}.edge-left .highlight-position::after,.edge-right .highlight-position::after{top:calc(var(--preview-position) * 1% - 6px)}.edge-left .highlight-position::after{left:-3px}.edge-right .highlight-position::after{right:-3px}.corner{position:absolute;width:calc(var(--preview-radius-tl) + 9px);height:calc(var(--preview-radius-tl) + 9px);max-width:48px;max-height:48px;opacity:.95}.corner.top-left{top:2px;left:2px;width:calc(var(--preview-radius-tl) + 9px);height:calc(var(--preview-radius-tl) + 9px);border:2px solid;border-color:#a78bfa transparent transparent #a78bfa;border-radius:var(--preview-radius-tl) 0 0 0}.corner.top-right{top:2px;right:2px;width:calc(var(--preview-radius-tr) + 9px);height:calc(var(--preview-radius-tr) + 9px);border:2px solid;border-color:#a78bfa #a78bfa transparent transparent;border-radius:0 var(--preview-radius-tr) 0 0}.corner.bottom-left{bottom:2px;left:2px;width:calc(var(--preview-radius-bl) + 9px);height:calc(var(--preview-radius-bl) + 9px);border:2px solid;border-color:transparent transparent #a78bfa #a78bfa;border-radius:0 0 0 var(--preview-radius-bl)}.corner.bottom-right{right:2px;bottom:2px;width:calc(var(--preview-radius-br) + 9px);height:calc(var(--preview-radius-br) + 9px);border:2px solid;border-color:transparent #a78bfa #a78bfa transparent;border-radius:0 0 var(--preview-radius-br) 0}.length-handle{position:absolute;width:3px;height:22px;border-radius:999px;background:#60a5fa;box-shadow:0 0 9px rgba(96,165,250,.8)}.length-handle.start{left:2px;top:50%;transform:translateY(-50%)}.length-handle.end{right:2px;top:50%;transform:translateY(-50%)}.length-vertical .length-handle{width:22px;height:3px;left:50%;transform:translateX(-50%)}.length-vertical .length-handle.start{top:2px}.length-vertical .length-handle.end{top:auto;right:auto;bottom:2px}.shoulder-handle{position:absolute;width:3px;height:18px;border-radius:999px;background:#a78bfa;box-shadow:0 0 8px rgba(167,139,250,.8)}.edge-top .shoulder-handle.start{top:2px;left:calc(var(--preview-shoulder) + 1px)}.edge-top .shoulder-handle.end{top:2px;right:calc(var(--preview-shoulder) + 1px)}.edge-bottom .shoulder-handle.start{bottom:2px;left:calc(var(--preview-shoulder) + 1px)}.edge-bottom .shoulder-handle.end{right:calc(var(--preview-shoulder) + 1px);bottom:2px}.edge-left .shoulder-handle,.edge-right .shoulder-handle{width:18px;height:3px;top:calc(var(--preview-shoulder) + 1px)}.edge-left .shoulder-handle.start{left:2px}.edge-left .shoulder-handle.end{bottom:calc(var(--preview-shoulder) + 1px);top:auto;left:2px}.edge-right .shoulder-handle.start{right:2px}.edge-right .shoulder-handle.end{right:2px;top:auto;bottom:calc(var(--preview-shoulder) + 1px)}.background-wash{position:absolute;inset:2px;border-radius:inherit;background:rgba(167,139,250,.14);box-shadow:inset 0 0 0 1px rgba(196,181,253,.72),0 0 16px rgba(167,139,250,.22)}.spectrum-focus{position:absolute;z-index:1;top:50%;right:5px;width:23px;height:21px;border:1px solid #60a5fa;border-radius:6px;background:rgba(96,165,250,.1);box-shadow:0 0 10px rgba(96,165,250,.55);transform:translateY(-50%)}.preview-expanded .spectrum-focus{top:14px;right:22px;width:47px;height:31px;border-radius:8px}
  @media (hover:hover) and (pointer:fine){.cover:hover{filter:brightness(1.08)}}
  @media (prefers-reduced-motion:reduce){.surface-anchor{transition:none!important}.island-surface,.expanded-layer,.control-row button{transition-duration:120ms!important}.island-surface,.island-shadow{transition-property:opacity,box-shadow!important}.expanded-layer{transform:none!important}.studio-preview-highlight .corner,.studio-preview-highlight::after,.studio-preview-highlight .length-handle,.studio-preview-highlight .shoulder-handle,.studio-preview-highlight .background-wash,.studio-preview-highlight .spectrum-focus{animation:none!important;opacity:.78}}

  .expanded-shell{display:block;width:var(--page-width);height:var(--page-height);zoom:var(--content-scale,1);transform-origin:top left}
  .feature-extension{position:absolute;z-index:3;overflow:hidden}
  .music-pane{width:100%;height:100%}






  .attached-style.edge-left .music-pane,.attached-style.edge-right .music-pane{padding-top:calc(16px + var(--shoulder-content-inset,0px));padding-bottom:calc(20px + var(--shoulder-content-inset,0px))}

  .navigation-page{display:flex;flex-direction:column;width:100%;height:100%;padding:16px 24px 24px;box-sizing:border-box}
  .attached-style.edge-top .navigation-page,.attached-style.edge-bottom .navigation-page{padding-left:calc(24px + var(--shoulder-content-inset,0px));padding-right:calc(24px + var(--shoulder-content-inset,0px))}
  .attached-style.edge-left .navigation-page,.attached-style.edge-right .navigation-page{padding-top:calc(16px + var(--shoulder-content-inset,0px));padding-bottom:calc(24px + var(--shoulder-content-inset,0px))}
  .page-header{display:flex;align-items:center;gap:9px;min-width:0;height:28px;flex:none;margin-bottom:12px;color:rgba(235,244,255,.78);font:500 11px/1 var(--app-font,system-ui)}
  .page-header span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .page-back{display:grid;place-items:center;width:28px;height:28px;flex:none;padding:0;border:0;border-radius:8px;color:#91caff;background:rgba(255,255,255,.08);cursor:pointer}
  .page-back:hover{background:rgba(255,255,255,.15)}

  .page-content{display:flex;flex:1;min-height:0;align-items:center;animation:page-enter 180ms ease both}
  .page-content>:global(*){width:100%}
  @keyframes page-enter{from{opacity:0;transform:translateY(4px)}to{opacity:1;transform:translateY(0)}}
  .reduce-page-motion .page-content,.reduce-page-motion .music-pane{animation:none}

  .expanded-shell{display:flex;flex-direction:column}
  .top-tools{flex:none;height:40px;box-sizing:border-box;padding:6px 32px;display:flex;align-items:center;justify-content:flex-end}
  .attached-style.edge-top .top-tools,.attached-style.edge-bottom .top-tools{padding-left:calc(28px + var(--shoulder-content-inset,0px));padding-right:calc(28px + var(--shoulder-content-inset,0px))}
  .attached-style.edge-left .top-tools,.attached-style.edge-right .top-tools{height:calc(40px + var(--shoulder-content-inset,0px));padding-top:calc(6px + var(--shoulder-content-inset,0px))}
  .page-body{flex:1;min-height:0;opacity:1;transition:opacity 100ms ease}.page-body.exiting{opacity:0;pointer-events:none}
  .attached-style.edge-left .page-body.with-toolbar .music-pane,.attached-style.edge-right .page-body.with-toolbar .music-pane{padding-top:16px}
  .attached-style.edge-left .page-body.with-toolbar .navigation-page,.attached-style.edge-right .page-body.with-toolbar .navigation-page{padding-top:16px}
  .reduce-page-motion .page-body{transition:none}
  .music-pane :global(.metadata .marquee-title){font-size:18px;line-height:1.2}.music-pane .metadata>span{font-size:15px}.music-pane :global(.progress-block .time){font-size:14px}.music-pane :global(.progress-block.with-times){column-gap:14px}.music-pane :global(.progress-block .track){height:8px}
  .expanded-cover{border-radius:20px}
  .top-row{gap:20px;margin-bottom:0}
  .music-pane :global(.metadata .marquee-title){font-size:24px;line-height:1.12;margin-bottom:5px}
  .music-pane .metadata>span{font-size:18px;line-height:1.15}
  .music-pane{padding:20px var(--music-inline-inset,40px) 14px}
  .music-pane :global(.progress-block .time){font-size:20px}
  .music-pane :global(.progress-block.with-times){grid-template-columns:max-content minmax(0,1fr) max-content;column-gap:12px}
  .music-pane :global(.progress-block .track){height:10px}
  .music-pane :global(.progress-block .elapsed){justify-content:flex-start}
  .music-pane :global(.progress-block .remaining){justify-content:flex-end}

  .control-row{display:grid;grid-template-columns:minmax(0,1fr) auto minmax(0,1fr);align-items:center;gap:0;width:100%;height:60px;margin-top:0;transform:translateY(-5px)}
  .utility-controls,.transport-controls{display:flex;align-items:center;flex:none}
  .utility-controls{gap:22px}
  .leading-controls{justify-self:start}.trailing-controls{justify-self:end}
  .transport-controls{gap:38px}
  .control-row button,.control-mark{display:grid;place-items:center;flex:none;width:48px;height:48px;padding:0;border:0;border-radius:9px;color:rgba(255,255,255,.95);background:transparent}
  .transport-controls button{width:58px;height:58px;border-radius:12px}
  .control-row button{position:relative;isolation:isolate;cursor:pointer;transition:transform 140ms cubic-bezier(.23,1,.32,1),color 140ms ease,background 150ms ease}
  .control-row button::before{content:"";position:absolute;inset:-2.5px;z-index:-1;border-radius:inherit;background:rgba(255,255,255,.1);opacity:0;pointer-events:none;transition:opacity 150ms ease}
  .control-row button:active:not(:disabled){transform:scale(.92)}
  .control-row button:disabled{opacity:.45;cursor:default;background:transparent}
  .control-row .play{color:#fff}
  .trailing-controls button:last-child{color:rgba(255,255,255,.95)}
  @media (hover:hover) and (pointer:fine){.control-row button:hover:not(:disabled){color:#fff}.control-row button:hover:not(:disabled)::before{opacity:1}}
  .attached-style.edge-left .composed .page-body .music-pane,.attached-style.edge-right .composed .page-body .music-pane{padding-top:20px;padding-bottom:14px}
  .floating-style .island-surface.editing-surface:active{transform:none}
  /* Respond to the music pane's usable width, including attached shoulders. */
  .system-netease .leading-controls :global(.smtc-modes){flex-direction:column;gap:0}
  .system-netease .leading-controls :global(.smtc-modes button){height:22px}
  .system-netease .leading-controls :global(.smtc-modes svg){width:20px;height:20px}
  .music-pane{container-type:inline-size;container-name:island-music}
  @container island-music (max-width:500px){
    .control-row{grid-template-columns:minmax(0,1fr) auto minmax(0,1fr);column-gap:12px}
    .utility-controls{gap:8px}
    .transport-controls{justify-self:center;justify-content:center;min-width:0;gap:8px}
    .control-row :global(.utility-controls button),.control-row .control-mark,
    .control-row :global(.like-wrap),.control-row :global(.mode-control-wrap){width:40px;height:40px}
    .system-netease .leading-controls :global(.smtc-modes button){height:20px}
    .transport-controls button{width:48px;height:48px}
    .control-row :global(svg){width:30px;height:30px}
    .control-row :global(.play-pause-icon),.transport-controls .play :global(svg){width:42px;height:42px}
  }
  @container island-music (max-width:330px){
    .control-row{column-gap:6px}
    .utility-controls{gap:4px}
    .control-row :global(.utility-controls button),.control-row .control-mark,
    .control-row :global(.like-wrap),.control-row :global(.mode-control-wrap){width:32px;height:32px}
    .transport-controls{gap:4px}
    .system-netease .leading-controls :global(.smtc-modes button){height:16px}
    .system-netease .leading-controls :global(.smtc-modes svg){width:16px;height:16px}
    .transport-controls button{width:36px;height:36px}
    .control-row :global(svg){width:26px;height:26px}
    .control-row :global(.play-pause-icon),.transport-controls .play :global(svg){width:34px;height:34px}
  }
</style>
