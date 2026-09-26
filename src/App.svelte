<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { emit, listen } from "@tauri-apps/api/event";
  import { eventManager, onMediaUpdate } from "./utils/eventManager";
  import { Events } from "./utils/eventConstants";
  import { mediaApi } from "$lib/api/media";
  import { idleApi } from "$lib/api/idle";
  import { audioApi } from "$lib/api/audio";
  import { refreshSpectrumDevice } from "$lib/spectrumStore";
  import { windowApi } from "$lib/api/window";
  import { settingsApi } from "$lib/api/settings";
  import IslandSurface from "$lib/IslandSurface.svelte";
  import Spectrum from "$lib/Spectrum.svelte";
  import { extractSpectrumColorsFromImage } from "$lib/spectrumColors";
  import {

    clampExpandedRadius,
    clampShoulderRadius,
    geometryFor,
    hiddenPlacementFor,
    navigationHostFor as hostFor,
    overlapAttachedEdge,
    placementFor,
    surfaceOffsetFor,
    type IslandEdge,
    type IslandMode,
    type IslandPage,
    type IslandRegionChange,
    type IslandStyle,
  } from "$lib/islandGeometry";
  import type { AppSettings, AudioDeviceInfo, IdleSnapshot, MediaState, MonitorInfo, SystemAudioState } from "$lib/api/types";
  import { DEFAULT_SETTINGS } from "$lib/api/types";
  import type { IslandTool } from "$lib/featureRail";
  import { applyAppFont } from "$lib/font";
  import { locale, setLocale, translate, type TranslationKey } from "$lib/i18n";
  import {
    activeCaptureReasons,
    EMPTY_CAPTURE_SNAPSHOT,
    type CaptureSnapshot,
  } from "$lib/captureMode";
  import { clampSeekPosition, mediaTrackKey, projectedPosition, reconcileReportedPosition } from "$lib/mediaClock";
  import { adjustCountdown, completeCountdown, createCountdownState, formatClock, getRemainingMs, pauseCountdown, resetCountdown, resumeCountdown, startCountdown, type CountdownState } from "$lib/countdown";
  import { ISLAND_MOTION } from "$lib/islandMotion";
  import {
    getCurrentWindow,
    currentMonitor,
    availableMonitors,
  } from "@tauri-apps/api/window";
  const playerNames = {
    netease: "网易云音乐",
    spotify: "Spotify",
    bilibili: "Bilibili",
    qqmusic: "QQ 音乐",
    apple: "Apple Music",
    generic: "多媒体",
  };

  const isDev = import.meta.env?.DEV ?? false;
  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);

  const logger = {
    log: (...args: any[]) => isDev && console.log("[App]", ...args),
    error: (...args: any[]) => console.error("[App]", ...args),
    warn: (...args: any[]) => console.warn("[App]", ...args),
    debug: (...args: any[]) => isDev && console.debug("[App]", ...args),
  };

  // ========== 状态管理 ==========
  let expanded = $state(false);
  let activeIslandPage = $state<IslandPage>("music");
  let hovering = $state(false);
  let artworkUrl = $state<string>("");
  let rawCoverUrl = "";
  let trackTitle = $state<string>("");
  let artistName = $state<string>("");
  let isPlaying = $state<boolean>(false);
  let lastSongKey: string | null = null;
  let metadataRecovery = { pending: false, nextAttempt: 0, attempts: 0 };

  async function recoverMissingMetadata() {
    if (!hasMediaSession || !lastSongKey || !trackTitle || trackTitle === "未知曲目") return;
    if (durationMs > 0 && (artworkUrl || currentSource !== "netease")) return;
    const recovery = metadataRecovery;
    if (recovery.pending || Date.now() < recovery.nextAttempt) return;
    const key = lastSongKey;
    recovery.pending = true;
    try {
      const info = await mediaApi.getNeteaseSongInfo(trackTitle, artistName === "未知艺术家" ? "" : artistName);
      if (key !== lastSongKey || recovery !== metadataRecovery || !info) return;
      if (!durationMs && info.duration && info.duration > 0) {
        // Preserve the elapsed clock while duration was unavailable.
        currentTimeMs = projectedPosition(islandMedia, Date.now());
        mediaSnapshotAt = Date.now();
        durationMs = info.duration;
      }
      if (!artworkUrl && info.albumPic) {
        const image = new Image();
        image.onload = () => {
          if (key !== lastSongKey || recovery !== metadataRecovery) return;
          rawCoverUrl = info.albumPic!;
          artworkUrl = info.albumPic!;
          void syncFloatingMediaClock();
        };
        image.src = info.albumPic;
      }
      void syncFloatingMediaClock();
    } catch (error) {
      console.warn("[媒体信息] 补查失败，将自动重试", error);
    } finally {
      recovery.pending = false;
      recovery.attempts += 1;
      recovery.nextAttempt = Date.now() + Math.min(60_000, 5_000 * 2 ** Math.min(recovery.attempts - 1, 4));
    }
  }

  let lastReportedPosition: number | undefined;
  let pendingSeekUntil = 0;
  let spectrumTopColor = $state<string>("#ffffff");
  let spectrumBottomColor = $state<string>("#888888");

  let currentTimeMs = $state<number>(0);
  let mediaSnapshotAt = $state<number>(Date.now());
  let clockNow = $state<number>(Date.now());
  let pageVisible = $state(true);
  function handleVisibilityChange() {
    pageVisible = document.visibilityState === "visible";
  }
  let durationMs = $state<number>(0);
  let currentSource = $state<string>("generic");
  let hasMediaSession = $state(false);
  let mediaCapabilities = $state<MediaState["capabilities"]>();
  let islandMode = $derived<IslandMode>(expanded ? "expanded" : hovering ? "hover" : "compact");
  let islandMedia = $derived<MediaState>({
    title: hasMediaSession ? trackTitle : "",
    artist: hasMediaSession ? artistName : "",
    albumArt: artworkUrl,
    isPlaying,
    positionMs: currentTimeMs,
    durationMs,
    lastUpdatedTimestamp: mediaSnapshotAt,
    source: currentSource,
    sourceDisplay: playerNames[currentSource as keyof typeof playerNames] || "多媒体",
    capabilities: mediaCapabilities,
  });
  let displayedPosition = $derived(projectedPosition(islandMedia, clockNow));

  let lastSyncedArtwork = "";
  let lastSyncedTrack = "";
  function syncFloatingMediaClock() {
    const syncedAt = Date.now();
    const snapshot: MediaState = {
      ...islandMedia,
      positionMs: projectedPosition(islandMedia, syncedAt),
      lastUpdatedTimestamp: syncedAt,
    };
    const signature = `${snapshot.source}|${snapshot.title}|${snapshot.artist}`;
    if (signature === lastSyncedTrack && snapshot.albumArt === lastSyncedArtwork) snapshot.albumArt = "";
    else { lastSyncedTrack = signature; lastSyncedArtwork = snapshot.albumArt; }
    return emit(Events.ISLAND_MEDIA_SYNC, snapshot);
  }
  function normalizedStyle(value: string): IslandStyle {
    return value === "edge" ? "edge" : "floating";
  }

  function normalizedEdge(value: string): IslandEdge {
    return value === "right" || value === "bottom" || value === "left" ? value : "top";
  }

  function normalizedSettings(value: Partial<AppSettings>): AppSettings {
    const position = Number(value.islandEdgePosition ?? DEFAULT_SETTINGS.islandEdgePosition);
    const legacyAutoHide = (value as Partial<AppSettings> & { autoHide?: boolean }).autoHide;
    return {
      ...DEFAULT_SETTINGS,
      ...value,
      islandStyle: normalizedStyle(value.islandStyle ?? DEFAULT_SETTINGS.islandStyle),
      islandEdge: normalizedEdge(value.islandEdge ?? DEFAULT_SETTINGS.islandEdge),
      spectrumMode: value.spectrumMode === "random" ? "random" : "realtime",
      captureHideOnFullscreen: value.captureHideOnFullscreen ?? legacyAutoHide ?? true,
      islandEdgePosition: Math.min(100, Math.max(0, Number.isFinite(position) ? position : 50)),
      collapsedEdgeShoulderRadius: Math.min(16, clampShoulderRadius(value.collapsedEdgeShoulderRadius ?? (value as Partial<AppSettings> & { edgeShoulderRadius?: number }).edgeShoulderRadius ?? 8)),
      expandedEdgeShoulderRadius: clampShoulderRadius(value.expandedEdgeShoulderRadius ?? 32),
      expandedCornerRadius: clampExpandedRadius(value.expandedCornerRadius ?? 45),
      compactLength: Math.min(300, Math.max(80, Number(value.compactLength ?? 80))),
      idleRotationSeconds: Math.min(60, Math.max(2, Number(value.idleRotationSeconds ?? 5))),
      idleItems: Array.isArray(value.idleItems) ? value.idleItems : DEFAULT_SETTINGS.idleItems,
      clockTimeZone: typeof value.clockTimeZone === "string" && value.clockTimeZone.trim().length > 0
        ? value.clockTimeZone.trim()
        : DEFAULT_SETTINGS.clockTimeZone,
    };
  }

  let interactionRegionRevision = Date.now() * 1000;
  let lastInteractionRegionSignature = "";
  function applyIslandRegion({ geometry, radii, polygon, extraRects, anchorGap }: IslandRegionChange) {
    const host = hostFor(renderedIslandStyle, renderedIslandEdge, appSettings.compactLength);
    const offset = surfaceOffsetFor(host, geometry, renderedIslandStyle, renderedIslandEdge);
    if (anchorGap !== undefined) {
      const delta = anchorGap - (renderedIslandStyle === "floating" ? 22 : 0);
      if (renderedIslandEdge === "top") offset.y += delta;
      else if (renderedIslandEdge === "bottom") offset.y -= delta;
      else if (renderedIslandEdge === "left") offset.x += delta;
      else offset.x -= delta;
    }
    const translatedExtraRects = (extraRects ?? []).map((rect) => ({
      ...rect,
      x: offset.x + rect.x,
      y: offset.y + rect.y,
    }));
    const signature = JSON.stringify({
      x: offset.x,
      y: offset.y,
      width: geometry.width,
      height: geometry.height,
      radii,
      polygon,
      extraRects: translatedExtraRects,
    });
    if (signature === lastInteractionRegionSignature) return;
    lastInteractionRegionSignature = signature;
    windowApi.setIslandInteractionRegion({
      revision: ++interactionRegionRevision,
      x: offset.x,
      y: offset.y,
      width: geometry.width,
      height: geometry.height,
      radii,
      polygon,
      extraRects: translatedExtraRects,
    }).catch((error) => logger.warn("窗口区域更新失败", error));
  }

  let currentTime = $state("");
  let lastClockMinute = "";

  // Clock text only changes once per minute. Keep the high-resolution clock
  // for media/timer interpolation, but avoid constructing Intl formatters on
  // every 250 ms tick.
  function updateTimeDisplay() {
    const minuteKey = `${appSettings.clockTimeZone}|${$locale}|${Math.floor(Date.now() / 60_000)}`;
    if (minuteKey === lastClockMinute) return;
    lastClockMinute = minuteKey;
    currentTime = formatClock(appSettings.clockTimeZone, $locale, Date.now());
  }

  onMount(() => {
    lastPlayedMedia = loadRememberedTrack();
    const persistOnPageHide = () => rememberCurrentTrack(true);
    window.addEventListener("pagehide", persistOnPageHide);
    updateTimeDisplay();
    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      window.removeEventListener("pagehide", persistOnPageHide);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  });

  $effect(() => {
    const interval = pageVisible && expanded && activeIslandPage === "music" && isPlaying ? 250
      : countdown.status === "running" ? 1000 : 60_000;
    const tick = () => { clockNow = Date.now(); updateTimeDisplay(); };
    appSettings.clockTimeZone; $locale;
    untrack(tick);
    let timer: ReturnType<typeof setTimeout>;
    const schedule = () => { timer = setTimeout(() => { tick(); schedule(); }, interval - Date.now() % interval); };
    schedule();
    return () => clearTimeout(timer);
  });

  const playerApps: Record<string, string> = {
    netease: "NeteaseCloudMusic",
    spotify: "Spotify",
    bilibili: "Bilibili",
    qqmusic: "QQMusic",
    apple: "AppleMusic",
    generic: "",
  };

  const LAST_PLAYED_MEDIA_KEY = "isle:last-played-media:v1";
  type RememberedTrack = {
    title: string;
    artist: string;
    albumArt: string;
    source: string;
    sourceDisplay: string;
    positionMs: number;
    durationMs: number;
  };
  let lastPlayedMedia = $state<RememberedTrack | null>(null);
  let resumingLastTrack = $state(false);
  let lastPersistedTrackSignature = "";
  let lastPersistedMediaAt = 0;

  function loadRememberedTrack(): RememberedTrack | null {
    try {
      const value = JSON.parse(window.localStorage.getItem(LAST_PLAYED_MEDIA_KEY) || "null");
      if (
        !value || typeof value.title !== "string" || !value.title ||
        typeof value.artist !== "string" || typeof value.source !== "string"
      ) return null;
      return {
        title: value.title,
        artist: value.artist,
        albumArt: typeof value.albumArt === "string" ? value.albumArt : "",
        source: value.source,
        sourceDisplay: typeof value.sourceDisplay === "string" ? value.sourceDisplay : "",
        positionMs: Math.max(0, Number(value.positionMs) || 0),
        durationMs: Math.max(0, Number(value.durationMs) || 0),
      };
    } catch {
      return null;
    }
  }

  function rememberCurrentTrack(force = false) {
    if (!hasMediaSession || !currentSource || !trackTitle || trackTitle === "未知曲目") return;

    const now = Date.now();
    const signature = `${currentSource}|${mediaTrackKey(trackTitle, artistName)}|${isPlaying}`;
    const savedTrack: RememberedTrack = {
      title: trackTitle,
      artist: artistName,
      albumArt: artworkUrl.startsWith("data:image/") && artworkUrl.length <= 900_000
        ? artworkUrl
        : artworkUrl.startsWith("https://") ? artworkUrl : "",
      source: currentSource,
      sourceDisplay: playerNames[currentSource as keyof typeof playerNames] || "多媒体",
      positionMs: projectedPosition(islandMedia, now),
      durationMs,
    };
    lastPlayedMedia = savedTrack;

    // Media snapshots arrive frequently. Keep the in-memory position current,
    // but only write to local storage on track/state changes or every 10 sec.
    if (!force && signature === lastPersistedTrackSignature && now - lastPersistedMediaAt < 10_000) return;
    try {
      window.localStorage.setItem(LAST_PLAYED_MEDIA_KEY, JSON.stringify(savedTrack));
      lastPersistedTrackSignature = signature;
      lastPersistedMediaAt = now;
    } catch {
      // Remembering a track is best effort if local storage is unavailable/full.
    }
  }

  function sameRememberedTrack(media: MediaState, saved: RememberedTrack) {
    const savedArtist = saved.artist === "未知艺术家" ? "" : saved.artist;
    const artistMatches = !savedArtist || !media.artist ||
      mediaTrackKey(media.artist, "") === mediaTrackKey(savedArtist, "");
    return media.source === saved.source && artistMatches &&
      mediaTrackKey(media.title || "", "") === mediaTrackKey(saved.title, "");
  }

  async function resumeLastTrack() {
    const saved = lastPlayedMedia;
    const appName = saved ? playerApps[saved.source] : "";
    if (!saved || !appName || resumingLastTrack) return;

    resumingLastTrack = true;
    try {
      await windowApi.openApplication(appName);

      const activateIfReady = async (current: MediaState) => {
        if (!sameRememberedTrack(current, saved)) return false;
        if (!current.isPlaying) await mediaApi.controlMedia("play_pause");
        return true;
      };

      // The app may still have a paused session even though no update has
      // reached the island yet.
      const current = await mediaApi.getMediaInfo().catch(() => null);
      if (current && await activateIfReady(current)) return;

      const deadline = Date.now() + 12_000;
      while (Date.now() < deadline) {
        await new Promise((resolve) => setTimeout(resolve, 600));
        const sessions = await mediaApi.listMediaSessions().catch(() => []);
        if (!sessions.some((session) => session.source === saved.source)) continue;
        const reopened = await mediaApi.getMediaInfo().catch(() => null);
        if (reopened && await activateIfReady(reopened)) return;
      }
    } catch (error) {
      console.error("恢复上次播放失败:", error);
    } finally {
      resumingLastTrack = false;
    }
  }

  async function openCurrentPlayer() {
    try {
      const appName = playerApps[currentSource] || "";
      if (appName) {
        await windowApi.openApplication(appName);
        console.log(`[播放器] 已尝试打开 ${appName}`);
      } else {
        console.warn(`[播放器] 未找到 ${currentSource} 的应用映射`);
      }
    } catch (error) {
      console.error("[播放器] 打开失败:", error);
    }
  }

  // ===== 应用设置 =====
  let appSettings = $state<AppSettings>({
    ...DEFAULT_SETTINGS,
  });
  let enabledFeatureTools = $derived.by<IslandTool[]>(() => {
    const tools: IslandTool[] = [];
    if (appSettings.showSettingsTool) tools.push("settings");
    if (appSettings.showFloatingTool) tools.push("floating");
    if (appSettings.showVolumeTool) tools.push("volume");
    if (appSettings.showTimerTool) tools.push("timer");
    if (appSettings.showHideTool) tools.push("hide");
    if (appSettings.showClockTool) tools.push("clock");
    if (appSettings.showWeatherTool) tools.push("weather");
    return tools;
  });
  let lastSettingsSnapshot = "";
  function applySettingsIfChanged(value: AppSettings) {
    const next = normalizedSettings(value);
    const snapshot = JSON.stringify(next);
    if (snapshot === lastSettingsSnapshot) return;
    lastSettingsSnapshot = snapshot;
    appSettings = next;
  }
  function toggleIsland() {
    expanded = !expanded;
  }

  let systemAudio = $state<SystemAudioState | null>(null);
  let audioDevices = $state<AudioDeviceInfo[]>([]);
  let audioDeviceSwitching = $state(false);
  let audioStateRequest: Promise<SystemAudioState | null> | null = null;
  let pendingVolume: number | null = null;
  let volumeWriteRequest: Promise<void> | null = null;

  async function handleAudioOpen(): Promise<SystemAudioState | null> {
    if (!(window as any).__TAURI_INTERNALS__) return null;
    if (audioStateRequest) return audioStateRequest;

    const request = (async () => {
      try {
        const [nextState, nextDevices] = await Promise.all([
          audioApi.getState(),
          audioApi.listDevices().catch((error) => {
            logger.warn("读取音频输出设备失败", error);
            return [] as AudioDeviceInfo[];
          }),
        ]);
        systemAudio = nextState;
        audioDevices = nextDevices;
        return nextState;
      } catch (error) {
        logger.warn("读取系统音量失败", error);
        return null;
      }
    })().finally(() => {
      audioStateRequest = null;
    });

    audioStateRequest = request;
    return request;
  }

  async function handleAudioVolume(volumePercent: number) {
    const nextVolume = Math.max(0, Math.min(100, Math.round(volumePercent)));
    const currentAudio = systemAudio ?? await handleAudioOpen();
    if (!currentAudio) return;

    systemAudio = {
      ...currentAudio,
      volumePercent: nextVolume,
      muted: nextVolume === 0,
    };

    // Range inputs emit many events while dragging. Keep the latest value and
    // serialize Windows audio writes so an older request cannot win the race
    // and snap the thumb back to a stale volume.
    pendingVolume = nextVolume;
    if (volumeWriteRequest) return;

    volumeWriteRequest = (async () => {
      try {
        while (pendingVolume !== null) {
          const value = pendingVolume;
          pendingVolume = null;
          await audioApi.setVolume(value);
        }
      } catch (error) {
        pendingVolume = null;
        logger.error("设置系统音量失败", error);
        void handleAudioOpen();
      }
    })().finally(() => {
      volumeWriteRequest = null;
    });

    await volumeWriteRequest;
  }

  async function handleAudioDevice(deviceId: string) {
    if (audioDeviceSwitching || !deviceId || deviceId === systemAudio?.deviceId) return;
    audioDeviceSwitching = true;
    try {
      await audioApi.setDefaultDevice(deviceId);
      await refreshSpectrumDevice().catch((error) => logger.warn("重连音频频谱失败", error));
      const [nextState, nextDevices] = await Promise.all([
        audioApi.getState(),
        audioApi.listDevices(),
      ]);
      systemAudio = nextState;
      audioDevices = nextDevices;
    } catch (error) {
      logger.error("切换音频输出设备失败", error);
      await handleAudioOpen();
    } finally {
      audioDeviceSwitching = false;
    }
  }

  onDestroy(() => {
    pendingVolume = null;
    volumeWriteRequest = null;
    audioStateRequest = null;
  });

  let countdown = $state<CountdownState>(createCountdownState());
  let timerRemainingMs = $derived(getRemainingMs(countdown, clockNow));
  let timerFinished = $state(false);
  let timerFinishedTimeout: ReturnType<typeof setTimeout> | null = null;

  function clearTimerFinished() {
    timerFinished = false;
    if (timerFinishedTimeout !== null) {
      clearTimeout(timerFinishedTimeout);
      timerFinishedTimeout = null;
    }
  }

  function handleTimerStart(durationMs: number) {
    clearTimerFinished();
    countdown = startCountdown(countdown, durationMs, Date.now());
  }

  function handleTimerPause() {
    clearTimerFinished();
    countdown = pauseCountdown(countdown, Date.now());
  }

  function handleTimerResume() {
    clearTimerFinished();
    countdown = resumeCountdown(countdown, Date.now());
  }

  function handleTimerAdjust(deltaMs: number) {
    clearTimerFinished();
    countdown = adjustCountdown(countdown, deltaMs, Date.now());
  }

  function handleTimerReset() {
    clearTimerFinished();
    countdown = resetCountdown(countdown);
  }

  $effect(() => {
    if (countdown.status !== "running" || timerRemainingMs > 0) return;
    timerFinished = true;
    expanded = true;
    if (timerFinishedTimeout !== null) clearTimeout(timerFinishedTimeout);
    timerFinishedTimeout = setTimeout(() => {
      timerFinished = false;
      timerFinishedTimeout = null;
    }, 10_000);
    countdown = completeCountdown(countdown, clockNow);
  });

  function timerSnapshot() {
    return {
      status: countdown.status,
      durationMs: countdown.durationMs,
      remainingMs: timerRemainingMs,
      label: countdown.label,
    };
  }

  let lastTimerBroadcast = "";
  $effect(() => {
    const snapshot = timerSnapshot();
    if (!(window as any).__TAURI_INTERNALS__) return;

    // The timer window interpolates between snapshots locally. Broadcasting
    // every 250 ms made the main window do unnecessary IPC and rendering work;
    // one update per displayed second is enough while a timer is running.
    const broadcastKey = JSON.stringify({
      ...snapshot,
      remainingMs: snapshot.status === "running"
        ? Math.ceil(snapshot.remainingMs / 1000) * 1000
        : snapshot.remainingMs,
    });
    if (broadcastKey === lastTimerBroadcast) return;
    lastTimerBroadcast = broadcastKey;
    void emit(Events.TIMER_STATE_CHANGED, snapshot);
  });

  $effect(() => {
    applyAppFont(appSettings.fontId);
    setLocale(appSettings.language);
  });
  let idleSnapshot = $state<IdleSnapshot>({ cpuPercent: 0, memoryPercent: 0, uploadBytesPerSecond: 0, downloadBytesPerSecond: 0, batteryPercent: null, batteryCharging: null, weatherTemperature: null, weatherCode: null, weatherUpdatedAt: null, weatherForecast: [] });
  // Keep weather available for the expanded function panel even while media
  // is playing. Compact mode still only renders it in the idle state.
  let showIdle = $derived(!hasMediaSession);
  let weatherLoading = $state(false);
  let weatherFailed = $state(false);
  let lastWeatherLocation = "";
  $effect(() => {
    const locationKey = JSON.stringify(appSettings.weatherLocation);
    if (lastWeatherLocation !== locationKey) {
      lastWeatherLocation = locationKey;
      idleSnapshot = { ...idleSnapshot, weatherTemperature: null, weatherCode: null, weatherForecast: [], weatherUpdatedAt: null };
    }
    if (!pageVisible || (!showIdle && (!expanded || activeIslandPage !== "weather"))) return;
    let pending = false;
    let disposed = false;
    const refresh = () => {
      if (pending) return;
      pending = true;
      weatherLoading = true;
      void idleApi.getSnapshot(locationKey)
        .then((value) => { if (!disposed) { idleSnapshot = value; weatherFailed = value.weatherUpdatedAt === null || Date.now() - value.weatherUpdatedAt * 1000 >= 30 * 60_000; } })
        .catch(() => { if (!disposed) weatherFailed = true; })
        .finally(() => { pending = false; if (!disposed) weatherLoading = false; });
    };
    refresh();
    const timer = setInterval(refresh, 30_000);
    return () => { disposed = true; clearInterval(timer); };
  });

  let renderedIslandStyle = $state<IslandStyle>(DEFAULT_SETTINGS.islandStyle);
  let renderedIslandEdge = $state<IslandEdge>(DEFAULT_SETTINGS.islandEdge);
  let fixedHostElement: HTMLDivElement;
  let placementAnimation: Animation | null = null;
  let placementTransitionRevision = 0;
  let suppressPlacementEffect = false;
  let panelInteracting = $state(false);
  let currentHost = $derived(hostFor(
    renderedIslandStyle,
    renderedIslandEdge,
    appSettings.compactLength,
  ));

  let captureSnapshot = $state<CaptureSnapshot>({ ...EMPTY_CAPTURE_SNAPSHOT });
  let isFullscreenApp = $derived(captureSnapshot.fullscreen);
  let isMouseAtTop = $state(false);
  let isHidden = $state(false);
  let manualHideActive = $state(false);
  let manualHideTimeout: ReturnType<typeof setTimeout> | null = null;
  let manualHideSequence = 0;

  let showMonitorMenu = $state(false);
  let monitors: Array<{
    name: string;
    index: number;
    position: { x: number; y: number };
    size: { width: number; height: number };
  }> = $state([]);
  let currentMonitorIndex = $state(0);

  let isFloatingWindowOpen = $state(false);
  let settingsWindowRequest: Promise<void> | null = null;

  let fps = $state(0);
  let frameCount = 0;
  let lastFpsTime = 0;
  let debugRafId: number | null = null;

  let win: ReturnType<typeof getCurrentWindow>;

  let cachedScreenWidth = 0;
  let cachedScreenHeight = 0;

  let monitorAnchorX = 0;
  let monitorAnchorY = 0;
  let windowReady = $state(false);
  let windowPlacementRevision = 0;
  let lastAppliedBounds = "";
  let lastPlacementInput = "";
  let placementMonitors: MonitorInfo[] = [];
  let placementMonitorsRefreshedAt = 0;

  // Island geometry is animated inside a fixed native host. Resizing the
  // WebView for every spring frame was the main source of expansion jank.

  async function applyWindowPlacement(
    style = renderedIslandStyle,
    edge = renderedIslandEdge,
    monitorIndex = appSettings.monitorIndex,
    positionPercent = appSettings.islandEdgePosition,
    hidden = isHidden,
    animateBounds = true,
  ) {
    if (!windowReady) return;
    const placementInput = JSON.stringify({
      style,
      edge,
      monitorIndex,
      positionPercent,
      hidden,
      compactLength: appSettings.compactLength,
    });
    if (
      placementInput === lastPlacementInput
      && placementMonitors.length > 0
      && Date.now() - placementMonitorsRefreshedAt < 5_000
    ) return;
    lastPlacementInput = placementInput;
    const revision = ++windowPlacementRevision;
    const shouldRefreshMonitors =
      placementMonitors.length === 0 || Date.now() - placementMonitorsRefreshedAt >= 5000;
    const allMonitors = shouldRefreshMonitors
      ? await windowApi.getMonitors()
      : placementMonitors;
    if (!allMonitors.length) return;
    if (shouldRefreshMonitors) {
      placementMonitors = allMonitors;
      placementMonitorsRefreshedAt = Date.now();
    }
    const safeIndex = Math.min(Math.max(0, monitorIndex), allMonitors.length - 1);
    const monitor = allMonitors[safeIndex];
    const dpr = monitor.scaleFactor || window.devicePixelRatio || 1;
    const host = hostFor(style, edge, appSettings.compactLength);
    const physicalHost = { width: Math.round(host.width * dpr), height: Math.round(host.height * dpr) };
    const baseShown = placementFor(
      { x: monitor.workX, y: monitor.workY, width: monitor.workWidth, height: monitor.workHeight },
      physicalHost,
      edge,
      Math.min(100, Math.max(0, positionPercent)),
    );
    // Extend attached windows one physical pixel beyond the compositor edge,
    // so clip-path antialiasing cannot reveal a transparent seam.
    const shown = style === "edge" ? overlapAttachedEdge(baseShown, edge) : baseShown;
    const compact = geometryFor("compact", appSettings.expandedCornerRadius, edge, appSettings.compactLength);
    const offset = surfaceOffsetFor(host, compact, style, edge);
    const target = hidden
      ? hiddenPlacementFor(
          baseShown,
          { x: offset.x * dpr, y: offset.y * dpr },
          { ...compact, width: compact.width * dpr, height: compact.height * dpr },
          edge,
          2,
        )
      : shown;
    if (revision !== windowPlacementRevision) return;
    currentMonitorIndex = safeIndex;
    cachedScreenWidth = monitor.width;
    cachedScreenHeight = monitor.height;
    monitorAnchorX = monitor.x + monitor.width / 2;
    monitorAnchorY = monitor.y;
    const animate = animateBounds
      && appSettings.enableAnimations
      && !appSettings.reduceAnimations
      && !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const boundsKey = `${target.x}:${target.y}:${target.width}:${target.height}`;
    if (boundsKey === lastAppliedBounds) return;
    await windowApi.animateWindowBounds(target.width, target.height, target.x, target.y, animate);
    if (revision === windowPlacementRevision) lastAppliedBounds = boundsKey;
  }

  function edgeTransform(edge: IslandEdge) {
    if (edge === "top") return "translateY(-6px) scale(0.96)";
    if (edge === "right") return "translateX(6px) scale(0.96)";
    if (edge === "bottom") return "translateY(6px) scale(0.96)";
    return "translateX(-6px) scale(0.96)";
  }

  async function transitionPlacement(nextStyle: IslandStyle, nextEdge: IslandEdge) {
    if (nextEdge === renderedIslandEdge) {
      // The fixed host reserves the floating gap in both modes, so the surface
      // can stay visible while its anchor and silhouette morph in place. Only
      // the one-pixel edge overlap changes at the native level; apply it once
      // without starting the expensive SetWindowPos animation loop.
      placementTransitionRevision += 1;
      placementAnimation?.cancel();
      suppressPlacementEffect = true;
      renderedIslandStyle = nextStyle;
      await applyWindowPlacement(
        nextStyle,
        nextEdge,
        appSettings.monitorIndex,
        appSettings.islandEdgePosition,
        isHidden,
        false,
      ).catch((error) => logger.warn("布局切换定位失败", error));
      suppressPlacementEffect = false;
      return;
    }
    const revision = ++placementTransitionRevision;
    placementAnimation?.cancel();
    const animate = appSettings.enableAnimations && fixedHostElement;
    const reduced = appSettings.reduceAnimations || window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const exitTransform = reduced ? "translate3d(0,0,0) scale(1)" : edgeTransform(renderedIslandEdge);
    const enterTransform = reduced ? "translate3d(0,0,0) scale(1)" : edgeTransform(nextEdge);
    if (animate) {
      placementAnimation = fixedHostElement.animate(
        [{ opacity: 1, transform: "translate3d(0,0,0) scale(1)" }, { opacity: 0, transform: exitTransform }],
        { duration: reduced ? 120 : 140, easing: "cubic-bezier(0.23, 1, 0.32, 1)", fill: "forwards" },
      );
      await placementAnimation.finished.catch(() => undefined);
      if (revision !== placementTransitionRevision) return;
    }
    suppressPlacementEffect = true;
    renderedIslandStyle = nextStyle;
    renderedIslandEdge = nextEdge;
    // The host is fully transparent at this point. Reposition it once, then
    // fade the compositor layer in; animating native resize/move concurrently
    // with WebView layout is visibly janky on Windows.
    await applyWindowPlacement(
      nextStyle,
      nextEdge,
      appSettings.monitorIndex,
      appSettings.islandEdgePosition,
      isHidden,
      false,
    ).catch((error) => logger.warn("布局切换定位失败", error));
    if (revision !== placementTransitionRevision) { suppressPlacementEffect = false; return; }
    suppressPlacementEffect = false;
    if (!fixedHostElement) return;
    placementAnimation?.cancel();
    if (animate) {
      placementAnimation = fixedHostElement.animate(
        [{ opacity: 0, transform: enterTransform }, { opacity: 1, transform: "translate3d(0,0,0) scale(1)" }],
        { duration: reduced ? 120 : 180, easing: "cubic-bezier(0.23, 1, 0.32, 1)", fill: "forwards" },
      );
      await placementAnimation.finished.catch(() => undefined);
    } else {
      fixedHostElement.style.opacity = "1";
      fixedHostElement.style.transform = "translate3d(0,0,0)";
    }
  }

  $effect(() => {
    const style = normalizedStyle(appSettings.islandStyle);
    const edge = normalizedEdge(appSettings.islandEdge);
    if (style !== renderedIslandStyle || edge !== renderedIslandEdge) {
      void transitionPlacement(style, edge);
    }
  });

  $effect(() => {
    const ready = windowReady;
    const monitorIndex = appSettings.monitorIndex;
    const position = appSettings.islandEdgePosition;
    const style = renderedIslandStyle;
    const edge = renderedIslandEdge;
    const hidden = isHidden;
    if (ready && !suppressPlacementEffect) void applyWindowPlacement(style, edge, monitorIndex, position, hidden);
  });

  $effect(() => {
    const panelActive = panelInteracting;
    const pointerInside = hovering;
    const shouldAutoClose = expanded && !pointerInside && !panelActive && !timerFinished;
    if (!shouldAutoClose) return;

    const timeout = setTimeout(() => {
      if (expanded && !hovering && !panelInteracting && !timerFinished) {
        expanded = false;
        showMonitorMenu = false;
      }
    }, 2_000);

    return () => clearTimeout(timeout);
  });

  async function toggleFloatingWindow() {
    try {
      if (isFloatingWindowOpen) {
        await windowApi.closeFloatingWindow();
        isFloatingWindowOpen = false;
      } else {
        await windowApi.openFloatingWindow();
        isFloatingWindowOpen = true;
        void syncFloatingMediaClock();
      }
    } catch (error) {
      logger.error("切换悬浮窗失败:", error);
    }
  }

  async function toggleTimerWindow() {
    try {
      await windowApi.toggleTimerWindow();
    } catch (error) {
      logger.error("切换倒计时窗口失败:", error);
    }
  }

  async function showSettingsWindow() {
    if (settingsWindowRequest) {
      await settingsWindowRequest;
      return;
    }

    const request = (async () => {
      try {
        // Settings is an open/focus action. Using toggle here can hide an
        // existing window whose visibility state is stale after a close or
        // minimize, making the button appear to do nothing.
        await windowApi.showStudioWindow();
      } catch (error) {
        logger.error("打开设置窗口失败:", error);
      }
    })();

    settingsWindowRequest = request;
    try {
      await request;
    } finally {
      if (settingsWindowRequest === request) settingsWindowRequest = null;
    }
  }

  function formatRgb(r: number, g: number, b: number): string {
    return `rgb(${r},${g},${b})`;
  }

  async function extractSpectrumColors(imgSrc: string) {
    if (!imgSrc) return;
    try {
      const colors = await extractSpectrumColorsFromImage(imgSrc);
      if (!colors || artworkUrl !== imgSrc) return;
      spectrumTopColor = formatRgb(...colors.top);
      spectrumBottomColor = formatRgb(...colors.bottom);
    } catch (error) {
      console.warn("取色失败，将保留当前频谱颜色", error);
    }
  }

  $effect(() => {
    if (artworkUrl) extractSpectrumColors(artworkUrl);
  });

  async function handleMediaAction(action: string, e?: MouseEvent) {
    e?.stopPropagation();

    if (action === "play_pause") {
      isPlaying = !isPlaying;
    }

    try {
      await mediaApi.controlMedia(action as "play_pause" | "next" | "prev");
    } catch (err) {
      if (action === "play_pause") {
        isPlaying = !isPlaying;
      }
      console.error("媒体控制失败:", err);
    }
  }

  async function handleSeek(positionMs: number) {
    const next = clampSeekPosition(positionMs, durationMs);
    const previous = projectedPosition(islandMedia);
    const seekTrack = lastSongKey;
    pendingSeekUntil = Date.now() + 2000;
    currentTimeMs = next;
    mediaSnapshotAt = Date.now();
    void syncFloatingMediaClock();
    try {
      await mediaApi.seekMedia(next);
    } catch (error) {
      if (seekTrack === lastSongKey) {
        pendingSeekUntil = 0;
        currentTimeMs = previous;
        mediaSnapshotAt = Date.now();
        void syncFloatingMediaClock();
      }
      console.error("调整播放进度失败:", error);
    }
  }

  function formatTime(ms: number): string {
    if (ms <= 0) return "00:00";
    const s = Math.floor(ms / 1000);
    const min = Math.floor(s / 60);
    const sec = s % 60;
    return `${min.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`;
  }

  let hideTimeout: ReturnType<typeof setTimeout> | null = null;

  function syncCaptureVisibility(snapshot = captureSnapshot) {
    const reasons = activeCaptureReasons(snapshot, appSettings);
    const fullscreenPeek = isMouseAtTop && reasons.length === 1 && reasons[0] === "fullscreen";
    const shouldHide = manualHideActive || (reasons.length > 0 && !fullscreenPeek);
    if (shouldHide && !isHidden) void hideWindowToTop();
    else if (!shouldHide && isHidden) void showWindow();
  }

  async function hideForTenSeconds() {
    const sequence = ++manualHideSequence;
    if (manualHideTimeout !== null) {
      clearTimeout(manualHideTimeout);
      manualHideTimeout = null;
    }

    const wasExpanded = expanded;
    // Force the compact state before moving the native window away. This keeps
    // the hidden placement and the rendered island geometry in sync.
    expanded = false;
    hovering = false;

    if (wasExpanded && appSettings.enableAnimations && !appSettings.reduceAnimations && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      await new Promise((resolve) => setTimeout(resolve, ISLAND_MOTION.collapseDuration + ISLAND_MOTION.outwardDelay));
    }
    if (sequence !== manualHideSequence) return;

    manualHideActive = true;
    syncCaptureVisibility();
    manualHideTimeout = setTimeout(() => {
      manualHideTimeout = null;
      manualHideActive = false;
      syncCaptureVisibility();
    }, 10_000);
  }

  function handleCaptureModeChange(snapshot: CaptureSnapshot) {
    captureSnapshot = { ...EMPTY_CAPTURE_SNAPSHOT, ...snapshot };
    syncCaptureVisibility(captureSnapshot);
  }

  async function hideWindowToTop() {
    try {
      isHidden = true;
      await applyWindowPlacement(undefined, undefined, undefined, undefined, true);
      console.log(`[自动隐藏] 窗口已向${renderedIslandEdge}边收起，保留 2px 唤醒区域`);
    } catch (error) {
      console.error("[自动隐藏] 失败:", error);
    }
  }

  async function showWindow() {
    try {
      isHidden = false;
      await applyWindowPlacement(undefined, undefined, undefined, undefined, false);
      console.log(`[自动显示] 窗口已恢复到${renderedIslandEdge}边`);
    } catch (error) {
      console.error("[自动显示] 失败:", error);
    }
  }

  async function handleMouseMove(event: MouseEvent) {
    if (!appSettings.captureHideOnFullscreen || !isFullscreenApp || captureSnapshot.screenshot) return;

    const mouseX = event.clientX;
    const mouseY = event.clientY;
    const wasMouseAtTop = isMouseAtTop;
    if (renderedIslandEdge === "top") isMouseAtTop = mouseY > window.innerHeight - 100;
    else if (renderedIslandEdge === "right") isMouseAtTop = mouseX < 100;
    else if (renderedIslandEdge === "bottom") isMouseAtTop = mouseY < 100;
    else isMouseAtTop = mouseX > window.innerWidth - 100;

    if (isMouseAtTop !== wasMouseAtTop) {
      console.log("[鼠标检测] 鼠标在顶部:", isMouseAtTop);

      if (isMouseAtTop && isHidden && !manualHideActive) {
        showWindow();

        if (hideTimeout) clearTimeout(hideTimeout);
        hideTimeout = setTimeout(() => {
          if (!isMouseAtTop) {
            hideWindowToTop();
          }
        }, 5000);
      } else if (!isMouseAtTop && !isHidden) {
        if (hideTimeout) clearTimeout(hideTimeout);
        hideTimeout = setTimeout(() => {
          if (!isMouseAtTop) {
            hideWindowToTop();
          }
        }, 500);
      }
    }
  }

  async function switchMonitor(index: number) {
    try {
      const allMonitors = await availableMonitors();
      const targetMonitor = allMonitors[index];

      if (!targetMonitor) {
        console.error("[显示器] 未找到目标显示器");
        return;
      }

      await moveToMonitor(targetMonitor);

      currentMonitorIndex = index;
      showMonitorMenu = false;

      try {
        await settingsApi.updateSettings({ monitorIndex: index });
        console.log("[显示器] 已保存选择到设置，索引:", index);
      } catch (saveError) {
        console.error("[显示器] 保存设置失败:", saveError);
      }

      console.log(
        "[显示器] 已切换到:",
        targetMonitor.name,
        "锚点:",
        monitorAnchorX,
        monitorAnchorY,
      );
    } catch (error) {
      console.error("[显示器] 切换失败:", error);
    }
  }

  async function moveToMonitor(targetMonitor: any) {
    const index = typeof targetMonitor?.index === "number"
      ? targetMonitor.index
      : monitors.findIndex((monitor) => monitor.position.x === targetMonitor?.position?.x && monitor.position.y === targetMonitor?.position?.y);
    await applyWindowPlacement(undefined, undefined, index >= 0 ? index : currentMonitorIndex);
    console.log("[显示器] 已按当前边缘与沿边位置移动到:", targetMonitor?.name || "显示器");
  }

  let lastMonitorIndex = -1;
  $effect(() => {
    const idx = appSettings.monitorIndex;
    if (
      idx !== undefined &&
      win &&
      monitors.length > 0 &&
      idx !== lastMonitorIndex
    ) {
      lastMonitorIndex = idx;
      const targetMonitor = monitors[idx];
      if (targetMonitor) {
        moveToMonitor(targetMonitor).catch(console.error);
      }
    }
  });

  function toggleMonitorMenu() {
    showMonitorMenu = !showMonitorMenu;
  }

  function closeMonitorMenu() {
    showMonitorMenu = false;
  }

  function startDebugFps() {
    if (debugRafId) return;
    lastFpsTime = performance.now();
    frameCount = 0;
    function tick() {
      frameCount++;
      const now = performance.now();
      if (now - lastFpsTime >= 1000) {
        fps = frameCount;
        frameCount = 0;
        lastFpsTime = now;
      }
      debugRafId = requestAnimationFrame(tick);
    }
    debugRafId = requestAnimationFrame(tick);
  }

  function stopDebugFps() {
    if (debugRafId) {
      cancelAnimationFrame(debugRafId);
      debugRafId = null;
    }
  }

  $effect(() => {
    if (appSettings.showDebugInfo) {
      startDebugFps();
    } else {
      stopDebugFps();
    }
  });

  onMount(() => {
    let cleanups: Array<() => void> = [];

    (async () => {
      console.log("[App.svelte] onMount 开始监听事件");

      // Register the timer bridge before any settings/monitor IPC. The timer
      // window is preloaded at startup, so its initial state request must not
      // race with the slower bootstrap work below.
      const unlistenTimerRequest = await listen(
        Events.TIMER_REQUEST_STATE,
        () => void emit(Events.TIMER_STATE_CHANGED, timerSnapshot()),
      );
      cleanups.push(unlistenTimerRequest);

      const unlistenTimerAction = await listen(
        Events.TIMER_ACTION,
        (event) => {
          const payload = event.payload as { action?: string; durationMs?: number };
          if (payload?.action === "start" && Number(payload.durationMs) > 0) handleTimerStart(Number(payload.durationMs));
          else if (payload?.action === "pause") handleTimerPause();
          else if (payload?.action === "resume") handleTimerResume();
          else if (payload?.action === "adjust") handleTimerAdjust(Number(payload.durationMs) || 0);
          else if (payload?.action === "reset") handleTimerReset();
        },
      );
      cleanups.push(unlistenTimerAction);

      try {
        const loadedSettings = await settingsApi.getSettings();
        const appWindow = getCurrentWindow();
        await appWindow.setAlwaysOnTop(loadedSettings.alwaysOnTop ?? true);
        applySettingsIfChanged(loadedSettings);
        console.log("[设置] 已加载:", appSettings);
      } catch (error) {
        console.error("[设置] 读取失败:", error);
      }

      const unlistenSettings = await eventManager.on(
        Events.SETTINGS_UPDATED,
        (s: any) => {
          if (s) {
            applySettingsIfChanged(s);
            console.log("[设置] 实时更新:", appSettings);

            if (s.monitorIndex !== undefined) {
              currentMonitorIndex = s.monitorIndex;
            }

            syncCaptureVisibility();
          }
        },
      );
      cleanups.push(unlistenSettings);

      const unlistenSettingsChanged = await eventManager.on(
        Events.SETTINGS_CHANGED,
        (settingName: any) => {
          console.log("[设置] 单项变更:", settingName);

          if (settingName === "monitorIndex") {
            windowApi
              .getCurrentMonitorIndex()
              .then((idx: number) => {
                currentMonitorIndex = idx;
                if (monitors[idx]) {
                  moveToMonitor(monitors[idx]).catch(console.error);
                }
              })
              .catch(console.error);
          } else if (settingName === "alwaysOnTop") {
          } else {
            settingsApi
              .getSettings()
              .then((s) => {
                if (s) {
                  appSettings = { ...appSettings, ...s };
                }
              })
              .catch(console.error);
          }
        },
      );
      cleanups.push(unlistenSettingsChanged);

      try {
        const allMonitors = await availableMonitors();
        monitors = allMonitors.map((m, idx) => {
          let name = m.name || `显示器 ${idx + 1}`;
          name = name.replace(/^\\\\\.\\DISPLAY/, "");
          name = name.replace(/^DISPLAY/, "");
          name = name.replace(/\\Device\\Video.*$/, "");
          const parts = name.split(/[\\/]/);
          if (parts.length > 1) {
            name = parts[parts.length - 1];
          }
          if (name.length > 12) {
            name = name.substring(0, 12) + "...";
          }
          return {
            name: name || `显示器 ${idx + 1}`,
            index: idx,
            position: m.position,
            size: m.size,
          };
        });

        const savedMonitorIndex = appSettings.monitorIndex ?? 0;

        if (savedMonitorIndex >= 0 && savedMonitorIndex < allMonitors.length) {
          currentMonitorIndex = savedMonitorIndex;
          const savedMonitor = allMonitors[savedMonitorIndex];
          cachedScreenWidth = savedMonitor.size.width;
          cachedScreenHeight = savedMonitor.size.height;
          monitorAnchorX =
            savedMonitor.position.x + savedMonitor.size.width / 2;
          monitorAnchorY = savedMonitor.position.y;
          console.log(
            "[显示器] 从设置恢复上次选择:",
            monitors[currentMonitorIndex]?.name,
          );
        } else {
          const activeMonitor = await currentMonitor();
          currentMonitorIndex = activeMonitor
            ? allMonitors.findIndex((m) => m.name === activeMonitor.name)
            : 0;

          if (activeMonitor) {
            cachedScreenWidth = activeMonitor.size.width;
            cachedScreenHeight = activeMonitor.size.height;
            monitorAnchorX =
              activeMonitor.position.x + activeMonitor.size.width / 2;
            monitorAnchorY = activeMonitor.position.y;
          }
          console.log(
            "[显示器] 使用当前显示器:",
            monitors[currentMonitorIndex]?.name,
          );
        }
      } catch (error) {
        console.error("[显示器] 初始化失败:", error);
      }

      const unlistenFloatingWindowClosed = await eventManager.on(
        Events.FLOATING_WINDOW_CLOSED,
        () => {
          isFloatingWindowOpen = false;
          console.log("[悬浮窗] 已关闭，更新状态");
        },
      );
      cleanups.push(unlistenFloatingWindowClosed);

      const unlistenMediaUpdate = await onMediaUpdate((data: any) => {
        const receivedAt = Date.now();
        // A removed session is an authoritative idle transition, not a metadata gap.
        if (!data.source) {
          rememberCurrentTrack(true);
          hasMediaSession = false;
          isPlaying = false;
          currentSource = "";
          lastSongKey = null;
          metadataRecovery = { pending: false, nextAttempt: 0, attempts: 0 };
          lastReportedPosition = undefined;
          pendingSeekUntil = 0;
          trackTitle = "";
          artistName = "";
          artworkUrl = "";
          rawCoverUrl = "";
          currentTimeMs = 0;
          durationMs = 0;
          mediaSnapshotAt = receivedAt;
          mediaCapabilities = data.capabilities;
          void syncFloatingMediaClock();
          return;
        }
        // SMTC can briefly return an empty metadata snapshot while the same
        // session is refreshing. Do not turn that transient gap into a new
        // zero-position track.
        if (
          data.source && lastSongKey &&
          (!data.title || data.title === "等待播放...") &&
          trackTitle &&
          trackTitle !== "未知曲目"
        ) {
          return;
        }
        const previousPosition = projectedPosition(islandMedia, receivedAt);
        const nextPlaying = Boolean(data.isPlaying);
        hasMediaSession = Boolean(data.source);
        currentSource = data.source || "";
        mediaCapabilities = data.capabilities;

        const currentSongKey = `${data.source || ""}|${mediaTrackKey(data.title || "", data.artist || "")}`;
        const songChanged = lastSongKey !== currentSongKey;

        const incomingDuration = Number(data.durationMs) || 0;
        const reportedDuration = songChanged
          ? incomingDuration
          : incomingDuration || durationMs;
        currentTimeMs = !songChanged && receivedAt < pendingSeekUntil ? previousPosition : reconcileReportedPosition(
          previousPosition,
          Number(data.positionMs) || 0,
          reportedDuration,
          nextPlaying,
          songChanged,
          lastReportedPosition,
        );
        lastReportedPosition = Number(data.positionMs) || 0;
        durationMs = reportedDuration;
        mediaSnapshotAt = receivedAt;
        isPlaying = nextPlaying;

        if (songChanged) {
          pendingSeekUntil = 0;
          lastSongKey = currentSongKey;
          metadataRecovery = { pending: false, nextAttempt: 0, attempts: 0 };
        }

        const titleChanged = trackTitle !== data.title;
        const artistChanged = artistName !== data.artist;

        const newCover =
          data.albumArt ||
          data.thumbnail ||
          data.coverUrl ||
          data.api_cover_url ||
          data.image ||
          (songChanged ? "" : rawCoverUrl);

        const coverChanged = songChanged || newCover !== rawCoverUrl;

        if (titleChanged || artistChanged || coverChanged) {
          if (titleChanged) {
            trackTitle = data.title || "未知曲目";

          }
          if (artistChanged) {
            artistName = data.artist || "未知艺术家";
          }

          if (coverChanged) {
            rawCoverUrl = newCover;

            if (
              newCover &&
              (newCover.startsWith("data:image") ||
                newCover.startsWith("http://") ||
                newCover.startsWith("https://") ||
                newCover.startsWith("file://"))
            ) {
              artworkUrl = newCover;
            } else if (
              newCover &&
              (newCover.includes(":\\") || newCover.includes(":/"))
            ) {
              artworkUrl = convertFileSrc(newCover);
            } else {
              artworkUrl = "";
            }
          }

          if (
            songChanged &&
            appSettings.enableHdCover &&
            data.title
          ) {
            const requestedTrackKey = currentSongKey;
            void mediaApi
              .resolveHdCover(data.title, data.artist || "", data.source || currentSource)
              .then((resolved) => {
                if (!resolved || lastSongKey !== requestedTrackKey) return;
                const resolvedUrl = /^[a-z]:[\\/]/i.test(resolved.url)
                  ? convertFileSrc(resolved.url)
                  : resolved.url;
                const image = new Image();
                image.onload = () => {
                  if (lastSongKey !== requestedTrackKey) return;
                  artworkUrl = resolvedUrl;
                  void syncFloatingMediaClock();
                };
                image.src = resolvedUrl;
              })
              .catch(() => undefined);
          }
        }

        void recoverMissingMetadata();
        rememberCurrentTrack(songChanged || !lastPersistedTrackSignature.endsWith(`|${isPlaying}`));
        void syncFloatingMediaClock();
      });
      cleanups.push(unlistenMediaUpdate);
      const recoveryTimer = setInterval(() => void recoverMissingMetadata(), 5_000);
      cleanups.push(() => clearInterval(recoveryTimer));
    })();

    return () => {
      cleanups.forEach((fn) => fn && fn());
    };
  });

  onDestroy(() => {
    stopDebugFps();
    if (timerFinishedTimeout !== null) clearTimeout(timerFinishedTimeout);
    if (manualHideTimeout !== null) clearTimeout(manualHideTimeout);
  });

  function handleGlobalClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (showMonitorMenu && !target.closest(".relative")) {
      closeMonitorMenu();
    }
  }

  async function initializeFixedHost() {
    await applyWindowPlacement();
  }

  onMount(() => {
    win = getCurrentWindow();
    windowReady = true;
    console.log("[App.svelte] 窗口对象已初始化");
    initializeFixedHost().catch((error) => logger.error("固定宿主初始化失败", error));

    document.addEventListener("click", handleGlobalClick);
    return () => {
      document.removeEventListener("click", handleGlobalClick);
    };
  });

  onMount(() => {
    // 初始设置由主启动流程统一加载；这里仅监听设置页实时广播。
    const workAreaTimer = setInterval(() => {
      if (windowReady) {
        void applyWindowPlacement(
          undefined,
          undefined,
          undefined,
          undefined,
          undefined,
          false,
        ).catch(() => undefined);
      }
    }, 30_000);

    // 所有捕获场景统一从 Capture Mode 状态进入，不各自维护隐藏逻辑。
    const unlistenCaptureMode = eventManager.on(
      Events.CAPTURE_MODE_CHANGED,
      (snapshot) => handleCaptureModeChange(snapshot as CaptureSnapshot),
    );

    let mouseMoveTimeout: ReturnType<typeof setTimeout> | null = null;
    const handleMouseMoveThrottled = (e: MouseEvent) => {
      if (mouseMoveTimeout) {
        clearTimeout(mouseMoveTimeout);
      }
      mouseMoveTimeout = setTimeout(() => {
        handleMouseMove(e);
      }, 100);
    };

    document.addEventListener("mousemove", handleMouseMoveThrottled);

    return () => {
      unlistenCaptureMode.then((unlisten) => unlisten());

      if (hideTimeout) {
        clearTimeout(hideTimeout);
      }
      if (mouseMoveTimeout) {
        clearTimeout(mouseMoveTimeout);
      }
      document.removeEventListener("mousemove", handleMouseMoveThrottled);
      clearInterval(workAreaTimer);
    };
  });
</script>

<div
  bind:this={fixedHostElement}
  class="fixed-host"
  style={`width:${currentHost.width}px;height:${currentHost.height}px`}
>
  <IslandSurface
    media={islandMedia}
    mode={islandMode}
    islandStyle={renderedIslandStyle}
    edge={renderedIslandEdge}
    position={displayedPosition}
    expandedRadius={appSettings.expandedCornerRadius ?? 45}
    collapsedEdgeShoulderRadius={appSettings.collapsedEdgeShoulderRadius ?? 8}
    expandedEdgeShoulderRadius={appSettings.expandedEdgeShoulderRadius ?? 32}
    compactLength={appSettings.compactLength ?? 80}
    idle={showIdle}
    idleTime={currentTime}
    idleWeatherTemperature={idleSnapshot.weatherTemperature}
    idleWeatherCode={idleSnapshot.weatherCode}
    idleWeatherForecast={idleSnapshot.weatherForecast}
    {lastPlayedMedia}
    canResumeLastTrack={Boolean(lastPlayedMedia && playerApps[lastPlayedMedia.source])}
    {resumingLastTrack}
    showSpectrum={appSettings.showSpectrum}
    spectrumMode={appSettings.spectrumMode}
    enableAnimations={appSettings.enableAnimations}
    reduceAnimations={appSettings.reduceAnimations}
    {spectrumTopColor}
    {spectrumBottomColor}
    showTime={false}
    timeText={currentTime}
    showDebugInfo={appSettings.showDebugInfo}
    debugLines={[`${fps} FPS`, currentSource, `${Math.round(displayedPosition)} ms`, isHidden ? "hidden" : islandMode]}
    onToggle={toggleIsland}
    onOpenPlayer={openCurrentPlayer}
    onResumeLastTrack={resumeLastTrack}
    onMediaAction={(action) => handleMediaAction(action)}
    onSeek={handleSeek}
    onToggleFloating={toggleFloatingWindow}
    onHideForTenSeconds={hideForTenSeconds}
    onSettingsToggle={showSettingsWindow}
    enabledTools={enabledFeatureTools}
    showCustomFunctionPanel={appSettings.showCustomFunctionPanel}
    visible={pageVisible && !isHidden}
    onPageChange={(page) => activeIslandPage = page}
    weatherCity={appSettings.weatherLocation?.name ?? ""}
    weatherUpdatedAt={idleSnapshot.weatherUpdatedAt}
    {weatherLoading}
    {weatherFailed}
    onPanelActivity={(active) => panelInteracting = active}
    onHoverChange={(value) => hovering = value}
    onRegionChange={applyIslandRegion}
    {systemAudio}
    {audioDevices}
    {audioDeviceSwitching}
    onAudioOpen={() => { void handleAudioOpen(); }}
    onAudioVolume={handleAudioVolume}
    onAudioDevice={handleAudioDevice}
    timerStatus={countdown.status}
    {timerRemainingMs}
    timerDurationMs={countdown.durationMs}
    timerLabel={countdown.label}
    {timerFinished}
    onTimerFinishedDismiss={clearTimerFinished}
    clockText={currentTime}
    clockTimeZone={appSettings.clockTimeZone}
    onTimerStart={handleTimerStart}
    onTimerPause={handleTimerPause}
    onTimerResume={handleTimerResume}
    onTimerAdjust={handleTimerAdjust}
    onTimerReset={handleTimerReset}
  />
</div>


<style>
  :global(*) { box-sizing: border-box; }
  :global(html, body, #app) { margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; background: transparent; -webkit-font-smoothing: antialiased; }
  .fixed-host { display: flex; align-items: flex-start; justify-content: center; box-sizing: border-box; background: transparent; pointer-events: none; }
  .fixed-host :global(.island-surface) { pointer-events: auto; }
</style>
