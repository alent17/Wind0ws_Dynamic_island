<script lang="ts">
  import { clearNeteaseModes } from "$lib/neteaseModeState";
  import { createArtworkOwner } from "$lib/artworkOwner";
  import { artworkValue, nativeStampDecision, readNativeMediaStamp, type NativeMediaStamp } from "$lib/mediaArtwork";
  import { onMount, onDestroy, untrack } from "svelte";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { emit, listen } from "@tauri-apps/api/event";
  import { eventManager, onMediaUpdate } from "./utils/eventManager";
  import { Events } from "./utils/eventConstants";
  import { mediaApi } from "$lib/api/media";
  import { islandMode, islandState, islandView, transitionIsland } from "$lib/islandStore";
  import { connectSettingsStore, refreshSettingsStore, settingsStore } from "$lib/settingsStore";
  import { LatestValueWriteQueue } from "$lib/latestValueWriteQueue";
  import { createAsyncCleanup } from "$lib/asyncCleanup";
import { createSeekHandler, createSeekRequestGate, formatSeekError, formatUserFacingError, seekSnapshotAllowsRollback } from "$lib/seekRequest";
  import { waitForOwner } from "$lib/asyncWait";
  import { createIslandRegionSender } from "$lib/islandRegionSender";
import { createMediaSyncSender, type MediaSyncSnapshot } from "$lib/mediaSyncSender";
  import { isTransportDiagnosticsEnabled, recordTransportCounter, runTimedTransportCommand, runTransportEvent } from "$lib/transportDiagnostics";
  import { loadImage } from "$lib/imageLoad";
  import { isHighResolutionSmtcCover } from "$lib/smtcArtworkQuality";
  const lifetime = createAsyncCleanup();
  const appMediaSessionId = crypto.randomUUID();
  let activeFloatingInstanceToken: string | undefined;
  const retiredFloatingInstanceTokens = new Set<string>();
  let appOverlaySequence = 0;
  const floatingMediaSender = createMediaSyncSender({
    dispatch: (payload) => emit(Events.ISLAND_MEDIA_SYNC, payload),
    onError: (error) => console.warn("[媒体同步] 浮窗状态发送失败，将保留最新快照重试", error),
  });
  let metadataImage: AbortController | undefined;
  onDestroy(() => {
    floatingMediaSender.dispose();
    lifetime.dispose(); metadataImage?.abort();
    ++windowPlacementRevision; ++placementTransitionRevision; ++manualHideSequence;
    placementAnimation?.cancel(); placementAnimation = null;
    windowReady = false;
  });
  import { idleApi } from "$lib/api/idle";
  import { audioApi } from "$lib/api/audio";
  import { refreshSpectrumDevice } from "$lib/spectrumStore";
  import { windowApi } from "$lib/api/window";
  import { settingsApi } from "$lib/api/settings";
  import { normalizeWidgets, type IslandWidgets } from "$lib/widgetLayout";
  import IslandSurface from "$lib/components/island/IslandSurface.svelte";
  import { extractSpectrumColorsFromImage } from "$lib/spectrumColors";
  import {

    clampExpandedRadius,
    clampShoulderRadius,
    geometryFor,
    hiddenPlacementFor,
    islandLayoutScaleForWorkArea,
    fitIslandViewport,
    interactionSurfaceOffsetFor,
    navigationHostFor as hostFor,
    overlapAttachedEdge,
    placementFor,
    physicalHostSizeFor,
    surfaceOffsetFor,
    type IslandEdge,
    type IslandRegionChange,
    type IslandStyle,
  } from "$lib/islandGeometry";
  import type { AppSettings, AudioDeviceInfo, MediaState, MonitorInfo, SystemAudioState, WeatherSnapshot } from "$lib/api/types";
  import { DEFAULT_SETTINGS } from "$lib/api/types";
  import { enabledBuiltinTools } from "$lib/featureRail";
  import { applyAppFont } from "$lib/font";
  import { locale, setLocale, translate, type TranslationKey } from "$lib/i18n";
  import {
    activeCaptureReasons,
    EMPTY_CAPTURE_SNAPSHOT,
    type CaptureSnapshot,
  } from "$lib/captureMode";
  import { clampSeekPosition, mediaTrackKey, projectedPosition, reconcileReportedPosition } from "$lib/mediaClock";
import { adjustCountdown, completeCountdown, createCountdownState, formatClock, getRemainingMs, pauseCountdown, resetCountdown, resumeCountdown, startCountdown, type CountdownState, type TimerSnapshot } from "$lib/countdown";
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
  let expanded = $derived($islandState.expanded);
  let activeIslandPage = $derived($islandState.activePage);
  let hovering = $derived($islandState.hovering);
  let artworkUrl = $state<string>("");
  let rawCoverUrl = "";
  let trackTitle = $state<string>("");
  let artistName = $state<string>("");
  let isPlaying = $state<boolean>(false);
  let neteasePlaybackMode = $state<MediaState["neteasePlaybackMode"]>();
  let neteaseModeBusy = $state(false);
  let neteaseModeError = $state("");
  let lastNeteaseModeProbeKey: string | null = null;
  let neteaseModeRevision = 0;
  let lastSongKey: string | null = null;
  let metadataRecovery = { pending: false, nextAttempt: 0, attempts: 0 };

  async function recoverMissingMetadata() {
    if (lifetime.disposed || !hasMediaSession || !lastSongKey || !trackTitle || trackTitle === "未知曲目") return;
    if (durationMs > 0 && (artworkUrl || currentSource !== "netease")) return;
    const recovery = metadataRecovery;
    if (recovery.pending || Date.now() < recovery.nextAttempt) return;
    const key = lastSongKey;
    recovery.pending = true;
    try {
      const info = await mediaApi.getNeteaseSongInfo(trackTitle, artistName === "未知艺术家" ? "" : artistName);
      if (lifetime.disposed || key !== lastSongKey || recovery !== metadataRecovery || !info) return;
      if (!durationMs && info.duration && info.duration > 0) {
        // Preserve the elapsed clock while duration was unavailable.
        currentTimeMs = projectedPosition(islandMedia, Date.now());
        mediaSnapshotAt = Date.now();
        durationMs = info.duration;
      }
      if (!artworkUrl && info.albumPic) {
        metadataImage?.abort();
        const controller = new AbortController(); metadataImage = controller;
        let image: HTMLImageElement | undefined;
        try {
          image = await loadImage(info.albumPic, controller.signal);
          if (lifetime.disposed || controller.signal.aborted || key !== lastSongKey || recovery !== metadataRecovery || artworkUrl) return;
          rawCoverUrl = info.albumPic;
          artworkUrl = info.albumPic;
          void syncFloatingMediaClock();
        } finally {
          image?.removeAttribute("src");
          if (metadataImage === controller) metadataImage = undefined;
        }
      }
      void syncFloatingMediaClock();
    } catch (error) {
      if (!lifetime.disposed && !(error instanceof DOMException && error.name === "AbortError")) console.warn("[媒体信息] 补查失败，将自动重试", error);
    } finally {
      if (lifetime.disposed) return;
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
  let resumePlacementTimer: ReturnType<typeof setTimeout> | undefined;
  function handleVisibilityChange() {
    pageVisible = document.visibilityState === "visible";
    if (pageVisible) {
      clockNow = Date.now();
      updateTimeDisplay(true);
      scheduleResumeRecovery();
    }
  }
  function scheduleResumeRecovery() {
    if (document.visibilityState !== "visible") return;
    if (resumePlacementTimer !== undefined) clearTimeout(resumePlacementTimer);
    resumePlacementTimer = setTimeout(() => {
      resumePlacementTimer = undefined;
      clockNow = Date.now();
      updateTimeDisplay(true);
      if (!windowReady) return;

      // Re-enumerate displays after resume/focus so a monitor or work-area
      // change made while suspended cannot leave the native host at stale bounds.
      placementMonitors = [];
      placementMonitorsRefreshedAt = 0;
      lastPlacementInput = "";
      lastAppliedBounds = "";
      clockNow = Date.now();
      updateTimeDisplay();
      void applyWindowPlacement(undefined, undefined, undefined, undefined, undefined, false)
        .catch((error) => logger.warn("恢复后刷新窗口布局失败", error));
    }, 350);
  }
  let durationMs = $state<number>(0);
  let currentSource = $state<string>("generic");
  let hasMediaSession = $state(false);
  let mediaCapabilities = $state<MediaState["capabilities"]>();
  let shuffleActive = $state<MediaState["shuffleActive"]>();
  let repeatMode = $state<MediaState["repeatMode"]>();
  // Remembered/default/demo state is not authoritative enough to replay to a
  // newly ready Floating consumer. This resets with each App mount.
  let appNativeMediaReady = false;
  let appNativeStamp = $state<NativeMediaStamp | undefined>(undefined);
  let appNativeTrackGeneration = 0;
  const seekRequestGate = createSeekRequestGate();
  let nativeSeekSnapshotRevision = 0;
  let seekErrorMessage = $state("");
  let seekBusy = $state(false);
  let progressKnown = $state(true);
  let neteaseProgressAvailable = $state<boolean | undefined>(undefined);
  let neteaseFeatures = $state<MediaState["neteaseFeatures"]>();
  let mediaSourceDisplay = $state("");
  let activeSeekRequests = 0;
  let latestNativeSeekSnapshot: { epoch: string; trackGeneration: number; title: string; artist: string; positionMs: number; durationMs: number } | undefined;
  let appNativeStampedMode = false;
  const appRetiredNativeEpochs = new Set<string>();
  let islandMedia = $derived<MediaState>({
    title: hasMediaSession ? trackTitle : "",
    artist: hasMediaSession ? artistName : "",
    albumArt: artworkUrl,
    isPlaying,
    positionMs: currentTimeMs,
    durationMs,
    lastUpdatedTimestamp: mediaSnapshotAt,
    source: currentSource,
    sourceDisplay: mediaSourceDisplay,
    capabilities: mediaCapabilities,
    shuffleActive,
    repeatMode,
    progressKnown,
    neteaseProgressAvailable,
    neteaseFeatures,
    nativeTrackGeneration: appNativeTrackGeneration,
    neteasePlaybackMode,
    ...(appNativeStamp ?? {}),
  });
  let displayedPosition = $derived(projectedPosition(islandMedia, clockNow));

  const mainCoverOwner = createArtworkOwner("cover");
  let hdArtworkResolved = $state(false);
  let mainCoverLookupTrackKey: string | null = null;
  let smtcCoverProbe: AbortController | undefined;
  let pendingHdImage: AbortController | undefined;
  function cancelMainCover() {
    mainCoverOwner.cancel();
    mainCoverLookupTrackKey = null;
    smtcCoverProbe?.abort(); smtcCoverProbe = undefined;
    pendingHdImage?.abort(); pendingHdImage = undefined;
  }
  $effect(() => { if (!$appSettings.enableHdCover) cancelMainCover(); });
  function syncFloatingMediaClock() {
    if (lifetime.disposed) return;
    if (!appNativeMediaReady || !activeFloatingInstanceToken) {
      if (isTransportDiagnosticsEnabled()) recordTransportCounter("mediaSync", "noConsumer");
      return;
    }
    const syncedAt = Date.now();
    if (appOverlaySequence >= Number.MAX_SAFE_INTEGER) {
      logger.warn("浮窗媒体 overlay 序号已耗尽，本次挂载停止同步");
      return;
    }
    const overlaySeq = ++appOverlaySequence;
    const snapshot: MediaSyncSnapshot = {
      ...islandMedia,
      artworkResolved: hdArtworkResolved,
      positionMs: projectedPosition(islandMedia, syncedAt),
      lastUpdatedTimestamp: syncedAt,
      appSessionId: appMediaSessionId,
      overlaySeq,
      ...(appNativeStamp ? { appOverlay: {
        nativeEpoch: appNativeStamp.nativeEpoch,
        nativeRevision: appNativeStamp.nativeRevision,
        trackKey: `${islandMedia.source}|${mediaTrackKey(islandMedia.title, islandMedia.artist)}`,
        title: islandMedia.title,
        artist: islandMedia.artist,
        albumArt: islandMedia.albumArt ?? "",
        artworkResolved: hdArtworkResolved,
        positionMs: projectedPosition(islandMedia, syncedAt),
        lastUpdatedTimestamp: syncedAt,
      } } : {}),
    };
    floatingMediaSender.offer(snapshot);
  }

  function pauseFloatingConsumer(instanceToken: string | undefined, retire: boolean) {
    if (!instanceToken || activeFloatingInstanceToken !== instanceToken) return false;
    floatingMediaSender.pauseForConsumer(`${appMediaSessionId}:${instanceToken}`);
    activeFloatingInstanceToken = undefined;
    if (retire) retiredFloatingInstanceTokens.add(instanceToken);
    return true;
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
    const legacyAppearance = value as Partial<AppSettings> & { compactLength?: number; expandedCornerRadius?: number };
    return {
      ...DEFAULT_SETTINGS,
      ...value,
      islandWidgets: normalizeWidgets(value.islandWidgets),
      floatingExpandedLength: normalizeExpandedLength(value.floatingExpandedLength),
      edgeExpandedLength: normalizeExpandedLength(value.edgeExpandedLength),
      islandStyle: normalizedStyle(value.islandStyle ?? DEFAULT_SETTINGS.islandStyle),
      islandEdge: normalizedEdge(value.islandEdge ?? DEFAULT_SETTINGS.islandEdge),
      spectrumMode: value.spectrumMode === "random" ? "random" : "realtime",
      captureHideOnFullscreen: value.captureHideOnFullscreen ?? legacyAutoHide ?? true,
      islandEdgePosition: Math.min(100, Math.max(0, Number.isFinite(position) ? position : 50)),
      collapsedEdgeShoulderRadius: Math.min(16, clampShoulderRadius(value.collapsedEdgeShoulderRadius ?? (value as Partial<AppSettings> & { edgeShoulderRadius?: number }).edgeShoulderRadius ?? 8)),
      expandedEdgeShoulderRadius: clampShoulderRadius(value.expandedEdgeShoulderRadius ?? 32),
      floatingExpandedCornerRadius: clampExpandedRadius(value.floatingExpandedCornerRadius ?? legacyAppearance.expandedCornerRadius ?? 45),
      edgeExpandedCornerRadius: clampExpandedRadius(value.edgeExpandedCornerRadius ?? legacyAppearance.expandedCornerRadius ?? 45),
      floatingCompactLength: Math.min(300, Math.max(80, Number(value.floatingCompactLength ?? legacyAppearance.compactLength ?? 80))),
      edgeCompactLength: Math.min(300, Math.max(80, Number(value.edgeCompactLength ?? legacyAppearance.compactLength ?? 80))),
      idleRotationSeconds: Math.min(60, Math.max(2, Number(value.idleRotationSeconds ?? 5))),
      idleItems: Array.isArray(value.idleItems) ? value.idleItems : DEFAULT_SETTINGS.idleItems,
      clockTimeZone: typeof value.clockTimeZone === "string" && value.clockTimeZone.trim().length > 0
        ? value.clockTimeZone.trim()
        : DEFAULT_SETTINGS.clockTimeZone,
    };
  }

  function normalizeExpandedLength(value: number | undefined) {
    return Math.min(900, Math.max(480, Number.isFinite(value) ? Math.round(value!) : 600));
  }

  function expandedLengthFor(style: IslandStyle, settings: AppSettings) {
    return style === "edge" ? settings.edgeExpandedLength : settings.floatingExpandedLength;
  }

  function compactLengthFor(style: IslandStyle, settings: AppSettings) {
    return style === "edge" ? settings.edgeCompactLength : settings.floatingCompactLength;
  }

  function expandedRadiusFor(style: IslandStyle, settings: AppSettings) {
    return style === "edge" ? settings.edgeExpandedCornerRadius : settings.floatingExpandedCornerRadius;
  }

  let interactionRegionRevision = Date.now() * 1000;
  let expandedScale = $state(1);
  type InteractionRegionPayload = Omit<Parameters<typeof windowApi.setIslandInteractionRegion>[0], "revision">;
  const interactionRegionSender = createIslandRegionSender<InteractionRegionPayload>({
    send: (region) => isTransportDiagnosticsEnabled()
      ? runTimedTransportCommand("region", () => windowApi.setIslandInteractionRegion({ ...region, revision: ++interactionRegionRevision }))
      : windowApi.setIslandInteractionRegion({ ...region, revision: ++interactionRegionRevision }),
    onError: (error) => logger.warn("窗口区域更新失败，稍后重试", error),
  });
  onDestroy(() => interactionRegionSender.dispose());

  let lastIslandRegion: IslandRegionChange | null = null;
  function applyIslandRegion(change: IslandRegionChange) {
    lastIslandRegion = change;
    const { geometry, radii, polygon, extraRects, anchorGap } = change;
    const host = renderedViewport.host;
    const offset = interactionSurfaceOffsetFor(host, geometry, renderedIslandStyle, renderedIslandEdge, renderedViewport.scale, anchorGap);
    const translatedExtraRects = (extraRects ?? []).map((rect) => ({
      ...rect,
      x: offset.x + rect.x,
      y: offset.y + rect.y,
    }));
    const region = {
      x: offset.x,
      y: offset.y,
      width: geometry.width,
      height: geometry.height,
      radii,
      polygon,
      extraRects: translatedExtraRects,
    } satisfies InteractionRegionPayload;
    interactionRegionSender.offer(region, JSON.stringify(region));
  }
  $effect(() => {
    // Host resizing also moves the surface even when its compact shape is unchanged.
    renderedViewport; renderedIslandStyle; renderedIslandEdge;
    if (lastIslandRegion) untrack(() => applyIslandRegion(lastIslandRegion!));
  });

  let currentTime = $state("");
  let lastClockMinute = "";

  // Clock text only changes once per minute. Keep the high-resolution clock
  // for media/timer interpolation, but avoid constructing Intl formatters on
  // every 250 ms tick.
  function updateTimeDisplay(force = false) {
    const minuteKey = `${$appSettings.clockTimeZone}|${$locale}|${Math.floor(Date.now() / 60_000)}`;
    if (!force && minuteKey === lastClockMinute) return;
    lastClockMinute = minuteKey;
    currentTime = formatClock($appSettings.clockTimeZone, $locale, Date.now());
  }

  onMount(() => {
    lastPlayedMedia = loadRememberedTrack();
    const persistOnPageHide = () => rememberCurrentTrack(true);
    window.addEventListener("pagehide", persistOnPageHide);
    updateTimeDisplay();
    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("focus", scheduleResumeRecovery);
    window.addEventListener("pageshow", scheduleResumeRecovery);

    return () => {
      window.removeEventListener("pagehide", persistOnPageHide);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      window.removeEventListener("focus", scheduleResumeRecovery);
      window.removeEventListener("pageshow", scheduleResumeRecovery);
      if (resumePlacementTimer !== undefined) clearTimeout(resumePlacementTimer);
    };
  });

  $effect(() => {
    $appSettings.clockTimeZone; $locale;
    updateTimeDisplay();
    let timer: ReturnType<typeof setTimeout>;
    const schedule = () => {
      timer = setTimeout(() => {
        updateTimeDisplay();
        schedule();
      }, 60_000 - Date.now() % 60_000);
    };
    schedule();
    return () => clearTimeout(timer);
  });

  $effect(() => {
    const mediaClockNeeded = pageVisible && !isHidden && expanded && activeIslandPage === "music" && isPlaying;
    const countdownRunning = countdown.status === "running";
    const countdownDeadline = countdown.endsAt;
    const hidden = !pageVisible || isHidden;
    return untrack(() => {
      clockNow = Date.now();
      let timer: ReturnType<typeof setTimeout>;
      const tick = () => { clockNow = Date.now(); };
      const schedule = () => {
        const now = Date.now();
        if (mediaClockNeeded) {
          timer = setTimeout(() => { tick(); schedule(); }, 250 - now % 250);
          return;
        }
        if (!countdownRunning || countdownDeadline === null) return;
        const remaining = countdownDeadline - now;
        if (remaining <= 0) { tick(); return; }
        const delay = hidden ? remaining : Math.min(remaining, remaining % 1_000 || 1_000);
        timer = setTimeout(() => { tick(); schedule(); }, Math.max(1, delay));
      };
      schedule();
      return () => clearTimeout(timer);
    });
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
    if (lifetime.disposed || !saved || !appName || resumingLastTrack) return;

    resumingLastTrack = true;
    try {
      await windowApi.openApplication(appName);
      if (lifetime.disposed) return;

      const activateIfReady = async (current: MediaState) => {
        if (lifetime.disposed || !sameRememberedTrack(current, saved)) return false;
        if (!current.isPlaying) await mediaApi.controlMedia("play_pause");
        return true;
      };

      // The app may still have a paused session even though no update has
      // reached the island yet.
      const current = await mediaApi.getMediaInfo().catch(() => null);
      if (current && await activateIfReady(current)) return;

      const deadline = Date.now() + 12_000;
      while (Date.now() < deadline) {
        if (!await waitForOwner(lifetime, 600)) return;
        const sessions = await mediaApi.listMediaSessions().catch(() => []);
        if (lifetime.disposed) return;
        if (!sessions.some((session) => session.source === saved.source)) continue;
        const reopened = await mediaApi.getMediaInfo().catch(() => null);
        if (reopened && await activateIfReady(reopened)) return;
      }
    } catch (error) {
      console.error("恢复上次播放失败:", error);
    } finally {
      if (!lifetime.disposed) resumingLastTrack = false;
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
  const appSettings = settingsStore;
  type WindowsNotification = { id: number; app: string; title: string; body: string };
  let notificationCurrent = $state<WindowsNotification | null>(null);
  let notificationQueue: WindowsNotification[] = [];
  let notificationTimer: ReturnType<typeof setTimeout> | undefined;
  let notificationExpanded = false;
  function clearNotifications() {
    clearTimeout(notificationTimer); notificationQueue = []; notificationCurrent = null;
    if (notificationExpanded && !$appSettings.keepIslandExpanded && !$islandState.hovering) transitionIsland({type:"collapse"});
    notificationExpanded = false;
  }
  function showNextNotification() {
    notificationCurrent = notificationQueue.shift() ?? null;
    if (!notificationCurrent) { clearNotifications(); return; }
    if (!$islandState.expanded) { notificationExpanded = true; transitionIsland({type:"expand"}); }
    notificationTimer = setTimeout(showNextNotification, 5000);
  }
  onMount(() => {
    if (!(window as any).__TAURI_INTERNALS__) return;
    let disposed = false; let revision = 0; let enabled = false;
    const seen = new Set<number>();
    let unlisten: (() => void) | undefined;
    const ready = listen<WindowsNotification>("windows-notification", ({payload}) => {
      if (!enabled || disposed || seen.has(payload.id)) return;
      seen.add(payload.id); if (seen.size > 512) seen.delete(seen.values().next().value!);
      if (notificationQueue.length >= 50) notificationQueue.shift();
      notificationQueue.push(payload); if (!notificationCurrent) showNextNotification();
    }).then(fn => { if (disposed) fn(); else unlisten = fn; });
    const unsubscribe = appSettings.subscribe(value => {
      if (value.notificationsEnabled === enabled) return;
      enabled = value.notificationsEnabled; const current = ++revision;
      if (!enabled) { seen.clear(); clearNotifications(); void invoke("stop_notification_listener"); return; }
      void ready.then(async () => {
        if (disposed || current !== revision) return;
        try { await invoke("start_notification_listener"); }
        catch { if (!disposed && current === revision) await settingsApi.updateSettings({notificationsEnabled:false}); }
      });
    });
    const timer = setInterval(() => {
      if (!enabled) return;
      void invoke<string>("notification_status").then(status => {
        if (!disposed && enabled && status !== "allowed") { clearNotifications(); void invoke("stop_notification_listener"); void settingsApi.updateSettings({notificationsEnabled:false}); }
      }).catch(() => { if (!disposed && enabled) { clearNotifications(); void invoke("stop_notification_listener"); void settingsApi.updateSettings({notificationsEnabled:false}); } });
    }, 10000);
    return () => { disposed = true; ++revision; unsubscribe(); clearInterval(timer); unlisten?.(); clearNotifications(); void invoke("stop_notification_listener"); };
  });

  let layoutPreview = $state<IslandWidgets | null>(null);
  let layoutEditing = $state(false);
  const shownWidgets = $derived(layoutPreview ?? ($appSettings.islandWidgetsEnabled ? normalizeWidgets($appSettings.islandWidgets) : {left:[],right:[]}));
  // Reserve the editor's full capacity before a card starts animating. Growing
  // the native WebView at the same time as the first card shifts its anchor.
  const requestedHostWidgetCount = $derived(Math.max(layoutEditing ? 2 : 0, shownWidgets.left.length + shownWidgets.right.length, normalizeWidgets($appSettings.islandWidgets).left.length + normalizeWidgets($appSettings.islandWidgets).right.length));
  let retainedHostWidgetCount = $state(0);
  const hostWidgetCount = $derived(Math.max(requestedHostWidgetCount, retainedHostWidgetCount));
  $effect(() => {
    const requested = requestedHostWidgetCount;
    if (expanded || layoutEditing) {
      if (requested > untrack(() => retainedHostWidgetCount)) retainedHostWidgetCount = requested;
      return;
    }
    // Keep the host steady until the island has finished collapsing; changing
    // native bounds during editor confirmation can move the visible content.
    const timer = setTimeout(() => { retainedHostWidgetCount = requested; }, ISLAND_MOTION.collapseDuration + 50);
    return () => clearTimeout(timer);
  });
  const weatherWidgetVisible = $derived(expanded && activeIslandPage === "music" && [...shownWidgets.left, ...shownWidgets.right].includes("weather"));
  const volumeWidgetVisible = $derived(expanded && activeIslandPage === "music" && [...shownWidgets.left, ...shownWidgets.right].includes("volume"));
  async function saveWidgets(layout: IslandWidgets) {
    await settingsApi.updateSettings({islandWidgets: normalizeWidgets(layout),islandWidgetsEnabled:true});
    appSettings.update(settings => ({...settings, islandWidgets: normalizeWidgets(layout),islandWidgetsEnabled:true}));
  }

  let enabledFeatureTools = $derived(enabledBuiltinTools($appSettings));
  function toggleIsland() {
    transitionIsland({ type: "toggle" });
  }

  let systemAudio = $state<SystemAudioState | null>(null);
  let audioDevices = $state<AudioDeviceInfo[]>([]);
  let audioDeviceSwitching = $state(false);
  let audioChangeRevision = 0;
  let audioStateRequest: Promise<SystemAudioState | null> | null = null;
  const volumeWriteQueue = new LatestValueWriteQueue<number>(
    (value) => isTransportDiagnosticsEnabled()
      ? runTimedTransportCommand("volume", () => audioApi.setVolume(value))
      : audioApi.setVolume(value),
    (error, pendingWillRetry) => {
      logger.error("设置系统音量失败", error);
      if (!pendingWillRetry && !audioDeviceSwitching && !lifetime.disposed) void handleAudioOpen();
    },
  );

  async function handleAudioOpen(): Promise<SystemAudioState | null> {
    if (!(window as any).__TAURI_INTERNALS__) return null;
    if (audioStateRequest) return audioStateRequest;

    const revision = audioChangeRevision;
    const request = (async () => {
      try {
        const [nextState, nextDevices] = await Promise.all([
          audioApi.getState(),
          audioApi.listDevices().catch((error) => {
            logger.warn("读取音频输出设备失败", error);
            return [] as AudioDeviceInfo[];
          }),
        ]);
        if (lifetime.disposed || revision !== audioChangeRevision || volumeWriteQueue.busy || audioDeviceSwitching) return systemAudio;
        volumeWriteQueue.invalidate();
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

  $effect(() => {
    if (!pageVisible || isHidden || !volumeWidgetVisible || layoutEditing) return;
    void handleAudioOpen();
    const refresh = setInterval(() => { if (!volumeWriteQueue.busy && !audioDeviceSwitching) void handleAudioOpen(); }, 5000);
    return () => clearInterval(refresh);
  });

  let wheelVolumeDisplay = $state<number | null>(null);
  let volumeProtectionPrompt = $state(false);
  let volumeProtectionBypassUntil = 0;
  let volumeHudTimer: ReturnType<typeof setTimeout> | undefined;
  let wheelVolumePending = 0;
  let wheelVolumeRunning = false;
  let lastVolumeWheelAt = 0;
  async function handleWheelVolume(delta: number) {
    if (!$appSettings.wheelVolumeControl || audioDeviceSwitching || lifetime.disposed || volumeProtectionPrompt) return;
    wheelVolumePending += delta;
    if (wheelVolumeRunning) return;
    wheelVolumeRunning = true;
    try {
      // Refresh after an idle pause so external volume changes are respected.
      if (Date.now() - lastVolumeWheelAt > 1000 && !volumeWriteQueue.busy) await handleAudioOpen();
      while (wheelVolumePending && $appSettings.wheelVolumeControl && !audioDeviceSwitching && !lifetime.disposed) {
        const change = wheelVolumePending;
        wheelVolumePending = 0;
        if (!systemAudio) break;
        lastVolumeWheelAt = Date.now();
        const current = systemAudio.volumePercent;
        let target = Math.max(0, Math.min(100, current + change));
        let requestConfirmation = false;
        if ($appSettings.volumeProtectionEnabled && Date.now() >= volumeProtectionBypassUntil && change > 0) {
          const limit = $appSettings.volumeProtectionLimit;
          if (current >= limit) {
            wheelVolumePending = 0;
            transitionIsland({ type: "expand" });
            volumeProtectionPrompt = true;
            showWheelVolume(current);
            break;
          }
          if (target >= limit) {
            target = limit;
            requestConfirmation = true;
          }
        }
        await handleAudioVolume(target);
        showWheelVolume(target);
        if (requestConfirmation) {
          wheelVolumePending = 0;
          transitionIsland({ type: "expand" });
          volumeProtectionPrompt = true;
          break;
        }
      }
    } catch (error) {
      logger.warn("滚轮调节音量失败", error);
    } finally {
      wheelVolumePending = 0;
      wheelVolumeRunning = false;
    }
  }

  function showWheelVolume(value: number) {
    wheelVolumeDisplay = value;
    if (volumeHudTimer) clearTimeout(volumeHudTimer);
    volumeHudTimer = setTimeout(() => wheelVolumeDisplay = null, 1300);
  }

  function confirmVolumeIncrease() {
    volumeProtectionBypassUntil = Date.now() + 10_000;
    volumeProtectionPrompt = false;
  }

  function cancelVolumeIncrease() {
    volumeProtectionPrompt = false;
    volumeProtectionBypassUntil = 0;
    wheelVolumePending = 0;
  }

  async function handleAudioVolume(volumePercent: number) {
    if (audioDeviceSwitching || lifetime.disposed) return;
    const nextVolume = Math.max(0, Math.min(100, Math.round(volumePercent)));
    const currentAudio = systemAudio ?? await handleAudioOpen();
    if (!currentAudio || audioDeviceSwitching || lifetime.disposed) return;

    audioChangeRevision++;
    systemAudio = {
      ...currentAudio,
      volumePercent: nextVolume,
      muted: nextVolume === 0,
    };

    // Keep one native write active, coalesce pending moves to the latest
    // integer, and ignore repeats for the same device/value pair.
    await volumeWriteQueue.submit(nextVolume, `${currentAudio.deviceId}\0${nextVolume}`);
  }

  async function handleAudioDevice(deviceId: string) {
    if (audioDeviceSwitching || !deviceId || deviceId === systemAudio?.deviceId) return;
    audioChangeRevision++;
    audioDeviceSwitching = true;
    volumeWriteQueue.cancelPending();
    try {
      await audioApi.setDefaultDevice(deviceId);
      await refreshSpectrumDevice().catch((error) => logger.warn("重连音频频谱失败", error));
      const [nextState, nextDevices] = await Promise.all([
        audioApi.getState(),
        audioApi.listDevices(),
      ]);
      volumeWriteQueue.invalidate();
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
    seekRequestGate.invalidate();
    volumeWriteQueue.cancelPending();
    audioStateRequest = null;
  });

  let countdown = $state<CountdownState>(createCountdownState());
  const timerAppSessionId = crypto.randomUUID();
  let timerStateRevision = 0;
  let lastTimerStateSignature = "";
  let lastTimerBroadcastRevision = 0;
  let timerSessionAnnounced = false;
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
    transitionIsland({ type: "expand" });
    if (timerFinishedTimeout !== null) clearTimeout(timerFinishedTimeout);
    timerFinishedTimeout = setTimeout(() => {
      timerFinished = false;
      timerFinishedTimeout = null;
    }, 10_000);
    countdown = completeCountdown(countdown, clockNow);
  });

  function timerSnapshot(requestId?: string): TimerSnapshot {
    const signature = JSON.stringify([
      countdown.status,
      countdown.durationMs,
      countdown.endsAt,
      countdown.pausedRemainingMs,
      countdown.label,
    ]);
    if (signature !== lastTimerStateSignature) {
      lastTimerStateSignature = signature;
      timerStateRevision += 1;
    }
    return {
      status: countdown.status,
      durationMs: countdown.durationMs,
      endsAt: countdown.status === "running" ? countdown.endsAt : null,
      remainingMs: countdown.status === "running" && countdown.endsAt !== null
        ? Math.max(0, countdown.endsAt - Date.now())
        : countdown.pausedRemainingMs,
      label: countdown.label,
      appSessionId: timerAppSessionId,
      stateRevision: timerStateRevision,
      sessionStart: !timerSessionAnnounced,
      ...(requestId ? { requestId } : {}),
    };
  }

  async function dispatchTimerSnapshot(requestId?: string, force = false) {
    if (!(window as any).__TAURI_INTERNALS__) return;
    const snapshot = timerSnapshot(requestId);
    const revision = snapshot.stateRevision ?? 0;
    if (!force && !requestId && revision <= lastTimerBroadcastRevision) return;
    try {
      if (isTransportDiagnosticsEnabled()) {
        await runTransportEvent("timerState", () => emit(Events.TIMER_STATE_CHANGED, snapshot), Boolean(requestId));
      } else {
        await emit(Events.TIMER_STATE_CHANGED, snapshot);
      }
      timerSessionAnnounced = true;
      if (revision >= lastTimerBroadcastRevision) {
        lastTimerBroadcastRevision = revision;
      }
    } catch (error) {
      logger.warn("广播倒计时状态失败；后续状态变更或窗口查询会补发", error);
    }
  }

  $effect(() => {
    countdown.status;
    countdown.durationMs;
    countdown.endsAt;
    countdown.pausedRemainingMs;
    countdown.label;
    void untrack(() => dispatchTimerSnapshot());
  });

  $effect(() => {
    applyAppFont($appSettings.fontId);
    setLocale($appSettings.language);
  });
  let weatherSnapshot = $state<WeatherSnapshot>({ weatherTemperature: null, weatherCode: null, weatherUpdatedAt: null, weatherForecast: [] });
  // Keep weather available for the expanded function panel even while media
  // is playing. Compact mode still only renders it in the idle state.
  let showIdle = $derived(!hasMediaSession);
  let weatherLoading = $state(false);
  let weatherFailed = $state(false);
  let lastWeatherLocation = "";
  $effect(() => {
    const locationKey = JSON.stringify($appSettings.weatherLocation);
    if (lastWeatherLocation !== locationKey) {
      lastWeatherLocation = locationKey;
      weatherSnapshot = { weatherTemperature: null, weatherCode: null, weatherForecast: [], weatherUpdatedAt: null };
    }
    if (!pageVisible || isHidden || (!showIdle && (!expanded || (activeIslandPage !== "weather" && !weatherWidgetVisible)))) return;
    let pending = false;
    let disposed = false;
    const refresh = () => {
      if (pending) return;
      pending = true;
      weatherLoading = true;
      void idleApi.getWeatherSnapshot(locationKey)
        .then((value) => { if (!disposed) { weatherSnapshot = value; weatherFailed = value.weatherUpdatedAt === null || Date.now() - value.weatherUpdatedAt * 1000 >= 30 * 60_000; } })
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
  // Both styles share an envelope so a style change cannot resize or shift
  // the native window while the surface is morphing inside it.
  function sharedStyleHost(edge: IslandEdge, scale: number, count: number) {
    const floating = hostFor("floating", edge, compactLengthFor("floating", $appSettings), scale, expandedLengthFor("floating", $appSettings), count);
    const attached = hostFor("edge", edge, compactLengthFor("edge", $appSettings), scale, expandedLengthFor("edge", $appSettings), count);
    return { width: Math.max(floating.width, attached.width), height: Math.max(floating.height, attached.height) };
  }
  let currentHost = $derived(sharedStyleHost(renderedIslandEdge, expandedScale, hostWidgetCount));

  let viewportWidth = $state(window.innerWidth);
  let viewportHeight = $state(window.innerHeight);
  const renderedViewport = $derived(fitIslandViewport(currentHost, expandedScale, {width:viewportWidth,height:viewportHeight}));
  // The native window uses integer physical pixels; rounding at non-integer
  // DPI can leave the WebView a fraction of a logical pixel below its host.
  // Only a real size shortage should suppress motion while placement catches up.
  const viewportClipped = $derived(viewportWidth < currentHost.width - 2 || viewportHeight < currentHost.height - 2);
  onMount(() => {
    const resize = () => { viewportWidth = window.innerWidth; viewportHeight = window.innerHeight; };
    resize();
    window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  });
  $effect(() => {
    if (!windowReady || !viewportClipped) return;
    // Wait for normal native resize animation first; recover a stuck/superseded request once.
    const timer = setTimeout(scheduleResumeRecovery, 600);
    return () => clearTimeout(timer);
  });

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
  let previousLayoutEditingForPlacement = false;
  let previousHostWidgetCountForPlacement = 0;
  let placementMonitors: MonitorInfo[] = [];
  let placementMonitorsRefreshedAt = 0;

  // Island geometry is animated inside a fixed native host. Resizing the
  // WebView for every spring frame was the main source of expansion jank.

  async function applyWindowPlacement(
    style = renderedIslandStyle,
    edge = renderedIslandEdge,
    monitorIndex = $appSettings.monitorIndex,
    positionPercent = $appSettings.islandEdgePosition,
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
      compactLength: compactLengthFor(style, $appSettings),
      floatingCompactLength: compactLengthFor("floating", $appSettings),
      edgeCompactLength: compactLengthFor("edge", $appSettings),
      floatingExpandedLength: expandedLengthFor("floating", $appSettings),
      edgeExpandedLength: expandedLengthFor("edge", $appSettings),
      expandedRadius: expandedRadiusFor(style, $appSettings),
      expandedLength: expandedLengthFor(style, $appSettings),
      hostWidgetCount,
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
    if (lifetime.disposed || revision !== windowPlacementRevision || !allMonitors.length) return;
    if (shouldRefreshMonitors) {
      placementMonitors = allMonitors;
      placementMonitorsRefreshedAt = Date.now();
    }
    const safeIndex = Math.min(Math.max(0, monitorIndex), allMonitors.length - 1);
    const monitor = allMonitors[safeIndex];
    const dpr = monitor.scaleFactor || window.devicePixelRatio || 1;
    expandedScale = Math.min(...(["floating", "edge"] as const).map(candidate =>
      islandLayoutScaleForWorkArea(monitor.workWidth, monitor.workHeight, dpr,
        candidate, edge, expandedLengthFor(candidate, $appSettings), hostWidgetCount)));
    // Monitor work area and native window bounds are physical pixels. The
    // IslandSurface, host and interaction region remain CSS-logical pixels.
    const compactLength = compactLengthFor(style, $appSettings);
    const host = sharedStyleHost(edge, expandedScale, hostWidgetCount);
    const physicalHost = physicalHostSizeFor(host, dpr);
    const baseShown = placementFor(
      { x: monitor.workX, y: monitor.workY, width: monitor.workWidth, height: monitor.workHeight },
      physicalHost,
      edge,
      Math.min(100, Math.max(0, positionPercent)),
    );
    // Extend attached windows one physical pixel beyond the compositor edge,
    // so clip-path antialiasing cannot reveal a transparent seam.
    const shown = style === "edge" ? overlapAttachedEdge(baseShown, edge) : baseShown;
    const compact = geometryFor("compact", expandedRadiusFor(style, $appSettings), edge, compactLength);
    const offset = surfaceOffsetFor(host, compact, style, edge, expandedScale);
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
      && $appSettings.enableAnimations
      && !$appSettings.reduceAnimations
      && !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const boundsKey = `${target.x}:${target.y}:${target.width}:${target.height}`;
    if (boundsKey === lastAppliedBounds) return;
    await windowApi.animateWindowBounds(target.width, target.height, target.x, target.y, animate);
    if (!lifetime.disposed && revision === windowPlacementRevision) lastAppliedBounds = boundsKey;
  }

  function edgeTransform(edge: IslandEdge) {
    if (edge === "top") return "translateY(-6px) scale(0.96)";
    if (edge === "right") return "translateX(6px) scale(0.96)";
    if (edge === "bottom") return "translateY(6px) scale(0.96)";
    return "translateX(-6px) scale(0.96)";
  }

  async function transitionPlacement(nextStyle: IslandStyle, nextEdge: IslandEdge) {
    if (lifetime.disposed) return;
    if (nextEdge === renderedIslandEdge) {
      // The fixed host reserves the floating gap in both modes, so the surface
      // can stay visible while its anchor and silhouette morph in place. Only
      // the one-pixel edge overlap changes at the native level; apply it once
      // without starting the expensive SetWindowPos animation loop.
      const revision = ++placementTransitionRevision;
      placementAnimation?.cancel();
      suppressPlacementEffect = true;
      renderedIslandStyle = nextStyle;
      await applyWindowPlacement(
        nextStyle,
        nextEdge,
        $appSettings.monitorIndex,
        $appSettings.islandEdgePosition,
        isHidden,
        false,
      ).catch((error) => logger.warn("布局切换定位失败", error));
      if (lifetime.disposed || revision !== placementTransitionRevision) return;
      suppressPlacementEffect = false;
      return;
    }
    const revision = ++placementTransitionRevision;
    placementAnimation?.cancel();
    const animate = $appSettings.enableAnimations && fixedHostElement;
    const reduced = $appSettings.reduceAnimations || window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const exitTransform = reduced ? "translate3d(0,0,0) scale(1)" : edgeTransform(renderedIslandEdge);
    const enterTransform = reduced ? "translate3d(0,0,0) scale(1)" : edgeTransform(nextEdge);
    if (animate) {
      placementAnimation = fixedHostElement.animate(
        [{ opacity: 1, transform: "translate3d(0,0,0) scale(1)" }, { opacity: 0, transform: exitTransform }],
        { duration: reduced ? 120 : 140, easing: "cubic-bezier(0.23, 1, 0.32, 1)", fill: "forwards" },
      );
      await placementAnimation.finished.catch(() => undefined);
      if (lifetime.disposed || revision !== placementTransitionRevision) return;
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
      $appSettings.monitorIndex,
      $appSettings.islandEdgePosition,
      isHidden,
      false,
    ).catch((error) => logger.warn("布局切换定位失败", error));
    if (lifetime.disposed || revision !== placementTransitionRevision) return;
    suppressPlacementEffect = false;
    if (!fixedHostElement) return;
    placementAnimation?.cancel();
    if (animate) {
      placementAnimation = fixedHostElement.animate(
        [{ opacity: 0, transform: enterTransform }, { opacity: 1, transform: "translate3d(0,0,0) scale(1)" }],
        { duration: reduced ? 120 : 180, easing: "cubic-bezier(0.23, 1, 0.32, 1)", fill: "forwards" },
      );
      await placementAnimation.finished.catch(() => undefined);
      if (lifetime.disposed || revision !== placementTransitionRevision) return;
      placementAnimation?.cancel(); placementAnimation = null;
    } else {
      fixedHostElement.style.opacity = "1";
      fixedHostElement.style.transform = "translate3d(0,0,0)";
    }
  }

  $effect(() => {
    const style = normalizedStyle($appSettings.islandStyle);
    const edge = normalizedEdge($appSettings.islandEdge);
    if (style !== renderedIslandStyle || edge !== renderedIslandEdge) {
      void transitionPlacement(style, edge);
    }
  });

  $effect(() => {
    const ready = windowReady;
    const monitorIndex = $appSettings.monitorIndex;
    const position = $appSettings.islandEdgePosition;
    const style = renderedIslandStyle;
    const edge = renderedIslandEdge;
    const hidden = isHidden;
    const editingNow = layoutEditing;
    const editTransition = editingNow !== previousLayoutEditingForPlacement;
    const count = hostWidgetCount;
    const editingHostChange = (editingNow || !expanded) && count !== previousHostWidgetCountForPlacement;
    previousLayoutEditingForPlacement = editingNow;
    previousHostWidgetCountForPlacement = count;
    if (ready && !suppressPlacementEffect) void applyWindowPlacement(style, edge, monitorIndex, position, hidden, !editTransition && !editingHostChange);
  });

  $effect(() => {
    const panelActive = panelInteracting;
    const pointerInside = hovering;
    const shouldAutoClose = expanded && !$appSettings.keepIslandExpanded && !pointerInside && !panelActive && !timerFinished && !notificationCurrent;
    if (!shouldAutoClose) return;

    const timeout = setTimeout(() => {
      const pointerStillInside = document.querySelector(".island-frame:hover") !== null;
      if (pointerStillInside && !hovering) {
        transitionIsland({ type: "hover-enter" });
        return;
      }
      if (expanded && !hovering && !pointerStillInside && !panelInteracting && !timerFinished && !notificationCurrent) {
        transitionIsland({ type: "collapse" });
        showMonitorMenu = false;
      }
    }, 2_000);

    return () => clearTimeout(timeout);
  });

  async function toggleFloatingWindow() {
    try {
      if (isFloatingWindowOpen) {
        await windowApi.closeFloatingWindow(activeFloatingInstanceToken);
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

  async function extractSpectrumColors(imgSrc: string, signal: AbortSignal) {
    if (!imgSrc) return;
    try {
      const colors = await extractSpectrumColorsFromImage(imgSrc, signal);
      const compactSurfaceVisible = $islandMode === "compact" || $islandMode === "hover";
      const hasCompactColorConsumer = compactSurfaceVisible && ($appSettings.showSpectrum || isPlaying);
      const hasExpandedSpectrumConsumer = $islandMode === "expanded" && activeIslandPage === "music" && $appSettings.showSpectrum;
      const hasCurrentColorConsumer = pageVisible && !isHidden && (hasCompactColorConsumer || hasExpandedSpectrumConsumer);
      if (lifetime.disposed || signal.aborted || !colors || artworkUrl !== imgSrc || !hasCurrentColorConsumer) return;
      spectrumTopColor = formatRgb(...colors.top);
      spectrumBottomColor = formatRgb(...colors.bottom);
    } catch (error) {
      if (!signal.aborted) console.warn("取色失败，将保留当前频谱颜色", error);
    }
  }

  $effect(() => {
    const compactSurfaceVisible = $islandMode === "compact" || $islandMode === "hover";
    const compactColorConsumer = compactSurfaceVisible && ($appSettings.showSpectrum || isPlaying);
    const expandedSpectrumConsumer = $islandMode === "expanded" && activeIslandPage === "music" && $appSettings.showSpectrum;
    const shouldExtractSpectrumColors = pageVisible && !isHidden && (compactColorConsumer || expandedSpectrumConsumer);
    const cover = artworkUrl;
    const controller = new AbortController();
    if (shouldExtractSpectrumColors && cover) void extractSpectrumColors(cover, controller.signal);
    return () => controller.abort();
  });

  async function handleMediaAction(action: string, e?: MouseEvent) {
    e?.stopPropagation();

    if (action === "cycle_netease_mode") {
      if (currentSource !== "netease" || neteaseModeBusy) return;
      neteaseModeRevision++;
      const requestedOnTrack = lastSongKey;
      neteaseModeError = "";
      neteaseModeBusy = true;
      try {
        const confirmedMode = await mediaApi.cycleNeteasePlaybackMode(islandMedia);
        if (!lifetime.disposed && currentSource === "netease") {
          // Playback mode is a player-wide setting, so a verified result remains
          // authoritative if the track changed while the command was in flight.
          neteasePlaybackMode = confirmedMode;
        }
      } catch (err) {
        console.error("网易云播放模式切换失败:", err);
        if (!lifetime.disposed && currentSource === "netease" && lastSongKey === requestedOnTrack) {
          neteaseModeError = formatUserFacingError(err, t("neteaseModeFailed"));
        }
      } finally {
        if (!lifetime.disposed) neteaseModeBusy = false;
      }
      return;
    }

    // Playback comes from media snapshots. A rejected request must not invert
    // a newer snapshot or leave an optimistic state behind.
    try {
      await mediaApi.controlMedia(action as "play_pause" | "next" | "prev");
    } catch (err) {
      console.error("媒体控制失败:", err);
    }
  }

  const appSeekHandler = createSeekHandler({
    gate: seekRequestGate,
    createContext: (positionMs: number, requestId: number) => ({
      next: clampSeekPosition(positionMs, durationMs),
      track: lastSongKey,
      requestId,
      snapshotRevision: nativeSeekSnapshotRevision,
      epoch: appNativeStamp?.nativeEpoch ?? "",
      trackGeneration: appNativeTrackGeneration,
      title: trackTitle,
      artist: artistName,
    }),
    isCurrentTrack: (context) => context.track === lastSongKey,
    isDisposed: () => lifetime.disposed,
    request: (context) => mediaApi.seekMedia(context.next, {
      nativeEpoch: context.epoch,
      nativeTrackGeneration: context.trackGeneration,
      title: context.title,
      artist: context.artist,
    }),
    onBegin: (context) => {
      seekErrorMessage = "";
      pendingSeekUntil = Date.now() + 2000;
      currentTimeMs = context.next;
      mediaSnapshotAt = Date.now();
      void syncFloatingMediaClock();
    },
    onConfirmed: (verified) => {
      pendingSeekUntil = 0;
      seekErrorMessage = "";
      if (!verified) return;
      currentTimeMs = verified.positionMs;
      durationMs = verified.durationMs;
      mediaSnapshotAt = Date.now();
      void syncFloatingMediaClock();
    },
    onFailed: async (_error, context) => {
      pendingSeekUntil = 0;
      // New native samples already reconciled the UI while CDP was pending.
      // Keep that newer state; the request remains current for its error alert.
      if (!seekSnapshotAllowsRollback(context.snapshotRevision, nativeSeekSnapshotRevision)) {
        void syncFloatingMediaClock();
        return;
      }
      const latest = latestNativeSeekSnapshot;
      if (latest && latest.epoch === context.epoch && latest.trackGeneration === context.trackGeneration
        && latest.title === context.title && latest.artist === context.artist) {
        currentTimeMs = latest.positionMs;
        durationMs = latest.durationMs || durationMs;
      } else {
        try {
          const snapshot = await mediaApi.getMediaInfo();
          if (!seekRequestGate.isCurrent(context.requestId) || lifetime.disposed || context.track !== lastSongKey
            || !seekSnapshotAllowsRollback(context.snapshotRevision, nativeSeekSnapshotRevision)) return;
          if (snapshot.nativeEpoch === context.epoch && snapshot.nativeTrackGeneration === context.trackGeneration
            && snapshot.title === context.title && snapshot.artist === context.artist) {
            currentTimeMs = Number(snapshot.positionMs) || 0;
            durationMs = Number(snapshot.durationMs) || durationMs;
          }
        } catch (refreshError) {
          logger.warn("seek 失败后刷新媒体位置失败，等待下一次原生快照", refreshError);
        }
      }
      mediaSnapshotAt = Date.now();
      void syncFloatingMediaClock();
    },
    onError: (error) => {
      console.error("调整播放进度失败:", error);
      seekErrorMessage = formatSeekError(error);
    },
  });

  async function handleSeek(positionMs: number) {
    activeSeekRequests++;
    seekBusy = true;
    try {
      await appSeekHandler(positionMs);
    } finally {
      activeSeekRequests = Math.max(0, activeSeekRequests - 1);
      if (!lifetime.disposed) seekBusy = activeSeekRequests > 0;
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
    const reasons = activeCaptureReasons(snapshot, $appSettings);
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
    transitionIsland({ type: "collapse" });
    transitionIsland({ type: "hover-leave" });

    if (wasExpanded && $appSettings.enableAnimations && !$appSettings.reduceAnimations && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      if (!await waitForOwner(lifetime, ISLAND_MOTION.collapseDuration + ISLAND_MOTION.outwardDelay)) return;
    }
    if (lifetime.disposed || sequence !== manualHideSequence) return;
    // The user may expand the island again while the collapse animation is
    // finishing. Do not let this older hide request take over that newer state.
    if (expanded) return;

    manualHideActive = true;
    syncCaptureVisibility();
    manualHideTimeout = setTimeout(() => {
      manualHideTimeout = null;
      manualHideActive = false;
      syncCaptureVisibility();
    }, 10_000);
  }

  function handleCaptureModeChange(snapshot: CaptureSnapshot) {
    if (lifetime.disposed) return;
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
    if (!$appSettings.captureHideOnFullscreen || !isFullscreenApp || captureSnapshot.screenshot) return;

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
    const idx = $appSettings.monitorIndex;
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
    if ($appSettings.showDebugInfo) {
      startDebugFps();
    } else {
      stopDebugFps();
    }
  });

  onMount(() => {
    const cleanups = lifetime;

    (async () => {
      console.log("[App.svelte] onMount 开始监听事件");

      // Register the timer bridge before any settings/monitor IPC. The timer
      // window is preloaded at startup, so its initial state request must not
      // race with the slower bootstrap work below.
      const unlistenTimerRequest = await listen(
        Events.TIMER_REQUEST_STATE,
        (event) => {
          if (lifetime.disposed) return;
          const requestId = (event.payload as { requestId?: unknown } | undefined)?.requestId;
          void dispatchTimerSnapshot(typeof requestId === "string" ? requestId : undefined, true);
        },
      );
      cleanups.add(unlistenTimerRequest);
      if (cleanups.disposed) return;

      const unlistenTimerAction = await listen(
        Events.TIMER_ACTION,
        (event) => {
          if (lifetime.disposed) return;
          const payload = event.payload as { action?: string; durationMs?: number };
          if (payload?.action === "start" && Number(payload.durationMs) > 0) handleTimerStart(Number(payload.durationMs));
          else if (payload?.action === "pause") handleTimerPause();
          else if (payload?.action === "resume") handleTimerResume();
          else if (payload?.action === "adjust") handleTimerAdjust(Number(payload.durationMs) || 0);
          else if (payload?.action === "reset") handleTimerReset();
        },
      );
      cleanups.add(unlistenTimerAction);
      if (cleanups.disposed) return;

      const settingsConnection = await connectSettingsStore(normalizedSettings);
      cleanups.add(settingsConnection.disconnect);
      if (cleanups.disposed) return;
      const loadedSettings = settingsConnection.settings;
      const appWindow = getCurrentWindow();
      await appWindow.setAlwaysOnTop(loadedSettings.alwaysOnTop ?? true);
      console.log("[设置] 已加载:", loadedSettings);
      if (cleanups.disposed) return;

      const unlistenSettings = await eventManager.on(
        Events.SETTINGS_UPDATED,
        (s: any) => {
          if (lifetime.disposed) return;
          if (s) {
            console.log("[设置] 实时更新:", $appSettings);

            if (s.monitorIndex !== undefined) {
              currentMonitorIndex = s.monitorIndex;
            }

            syncCaptureVisibility();
          }
        },
      );
      cleanups.add(unlistenSettings);
      if (cleanups.disposed) return;

      const unlistenSettingsChanged = await eventManager.on(
        Events.SETTINGS_CHANGED,
        (settingName: any) => {
          if (lifetime.disposed) return;
          console.log("[设置] 单项变更:", settingName);

          if (settingName === "monitorIndex") {
            windowApi
              .getCurrentMonitorIndex()
              .then((idx: number) => {
                if (lifetime.disposed) return;
                currentMonitorIndex = idx;
                if (monitors[idx]) {
                  moveToMonitor(monitors[idx]).catch(console.error);
                }
              })
              .catch(console.error);
          } else if (settingName === "monitor_index") {
            // Built-in settings commits emit this legacy marker immediately
            // before the full settings-updated snapshot. That snapshot is the
            // source of truth and already drives monitor placement.
            return;
          } else if (settingName !== "alwaysOnTop") {
            void refreshSettingsStore(normalizedSettings, () => !lifetime.disposed);
          }
        },
      );
      cleanups.add(unlistenSettingsChanged);
      if (cleanups.disposed) return;

      try {
        const allMonitors = await availableMonitors();
        if (lifetime.disposed) return;
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

        const savedMonitorIndex = $appSettings.monitorIndex ?? 0;

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
          if (lifetime.disposed) return;
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
      if (cleanups.disposed) return;

      // Keep the transparent native host hidden until the WebView has mounted,
      // the saved display has been resolved, and its responsive bounds are in place.
      if (!await waitForOwner(lifetime)) return;
      renderedIslandStyle = normalizedStyle($appSettings.islandStyle);
      renderedIslandEdge = normalizedEdge($appSettings.islandEdge);
      await applyWindowPlacement(
        renderedIslandStyle,
        renderedIslandEdge,
        $appSettings.monitorIndex,
        $appSettings.islandEdgePosition,
        isHidden,
        false,
      ).catch((error) => logger.warn("启动窗口定位失败", error));
      if (lifetime.disposed) return;
      await windowApi.showMainWindow().catch((error) => logger.error("启动时显示灵动岛失败", error));

      if (lifetime.disposed) return;
      const unlistenFloatingWindowClosed = await eventManager.on(
        Events.FLOATING_WINDOW_CLOSED,
        ({ instanceToken }: { instanceToken?: string } = {}) => {
          if (lifetime.disposed) return;
          if (pauseFloatingConsumer(instanceToken, true)) {
            isFloatingWindowOpen = false;
            console.log("[悬浮窗] 已关闭，更新状态");
          }
        },
      );
      cleanups.add(unlistenFloatingWindowClosed);
      if (cleanups.disposed) return;

      const unlistenFloatingMediaReady = await eventManager.on(
        Events.FLOATING_MEDIA_READY,
        ({ instanceToken, appSessionId }: { instanceToken?: string; appSessionId?: string }) => {
          if (lifetime.disposed || !instanceToken) return;
          if (appSessionId !== appMediaSessionId) {
            void emit(Events.FLOATING_MEDIA_READY_QUERY, { appSessionId: appMediaSessionId, instanceToken }).catch((error) => {
              if (!lifetime.disposed) logger.warn("向浮窗声明 App 媒体会话失败", error);
            });
            return;
          }
          if (retiredFloatingInstanceTokens.has(instanceToken)) return;
          if (activeFloatingInstanceToken && activeFloatingInstanceToken !== instanceToken) {
            retiredFloatingInstanceTokens.add(activeFloatingInstanceToken);
          }
          activeFloatingInstanceToken = instanceToken;
          if (floatingMediaSender.resetForConsumer(`${appMediaSessionId}:${instanceToken}`) && appNativeMediaReady) syncFloatingMediaClock();
        },
      );
      cleanups.add(unlistenFloatingMediaReady);
      if (cleanups.disposed) return;

      const unlistenFloatingMediaNotReady = await eventManager.on(
        Events.FLOATING_MEDIA_NOT_READY,
        ({ instanceToken, appSessionId, reason }: { instanceToken?: string; appSessionId?: string; reason?: "hidden" | "unmounted" } = {}) => {
          if (lifetime.disposed || !instanceToken || (appSessionId !== undefined && appSessionId !== appMediaSessionId)) return;
          if (pauseFloatingConsumer(instanceToken, reason !== "hidden")) {
            if (reason !== "hidden") isFloatingWindowOpen = false;
          }
        },
      );
      cleanups.add(unlistenFloatingMediaNotReady);
      if (cleanups.disposed) return;

      const unlistenMediaUpdate = await onMediaUpdate((data: any) => {
        if (lifetime.disposed) return;
        const incomingStamp = readNativeMediaStamp(data);
        const stampDecision = nativeStampDecision(appNativeStamp, appNativeStampedMode, appRetiredNativeEpochs, incomingStamp, true);
        if (["older", "retired", "invalid"].includes(stampDecision)) return;
        if (Number.isSafeInteger(data.nativeTrackGeneration) && data.nativeTrackGeneration >= 0) {
          appNativeTrackGeneration = data.nativeTrackGeneration;
        }
        if (stampDecision === "switch") {
          if (appNativeStamp) appRetiredNativeEpochs.add(appNativeStamp.nativeEpoch);
          appNativeStamp = incomingStamp;
          appNativeStampedMode = true;
        } else if (stampDecision === "initial" || stampDecision === "newer" || (stampDecision === "same" && incomingStamp)) {
          if (incomingStamp) {
            appNativeStamp = incomingStamp;
            appNativeStampedMode = true;
          }
        }
        appNativeMediaReady = true;
        const receivedAt = Date.now();
        if (data.source && appNativeStamp) {
          nativeSeekSnapshotRevision++;
          latestNativeSeekSnapshot = {
            epoch: appNativeStamp.nativeEpoch,
            trackGeneration: appNativeTrackGeneration,
            title: String(data.title || ""),
            artist: String(data.artist || ""),
            positionMs: Number(data.positionMs) || 0,
            durationMs: Number(data.durationMs) || 0,
          };
          if (pendingSeekUntil) {
            // A newer native position can refresh optimistic UI, but it does not
            // cancel the still-pending user seek or suppress its eventual error.
            pendingSeekUntil = 0;
          }
        } else if (!data.source) {
          latestNativeSeekSnapshot = undefined;
        }
        // A removed session is an authoritative idle transition, not a metadata gap.
        if (!data.source) {
          cancelMainCover(); hdArtworkResolved=false;
          rememberCurrentTrack(true);
          hasMediaSession = false;
          transitionIsland({ type: "media-session", active: false });
          isPlaying = false;
          currentSource = "";
          lastSongKey = null;
          seekErrorMessage = "";
          metadataImage?.abort(); metadataImage = undefined;
          metadataRecovery = { pending: false, nextAttempt: 0, attempts: 0 };
          lastReportedPosition = undefined;
          pendingSeekUntil = 0;
          seekRequestGate.invalidate();
          trackTitle = "";
          artistName = "";
          artworkUrl = "";
          rawCoverUrl = "";
          currentTimeMs = 0;
          durationMs = 0;
          mediaSnapshotAt = receivedAt;
          mediaCapabilities = data.capabilities;
          shuffleActive = undefined;
          repeatMode = undefined;
          progressKnown = true;
          neteaseProgressAvailable = undefined;
          clearNeteaseModes();
          neteaseFeatures = undefined;
          mediaSourceDisplay = "";
          neteasePlaybackMode = undefined;
          neteaseModeError = "";
          lastNeteaseModeProbeKey = null;
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
          transitionIsland({ type: "media-session", active: hasMediaSession });
        currentSource = data.source || "";
        mediaSourceDisplay = data.sourceDisplay || "";
        if (neteaseFeatures?.mode.source === "cdp" && data.neteaseFeatures?.mode.source !== "cdp") clearNeteaseModes();
        neteaseFeatures = data.neteaseFeatures;
        progressKnown = data.progressKnown !== false;
        neteaseProgressAvailable = currentSource === "netease" && typeof data.neteaseProgressAvailable === "boolean"
          ? data.neteaseProgressAvailable
          : undefined;
        if (currentSource !== "netease" || data.neteaseFeatures?.mode.source !== "cdp") {
          neteasePlaybackMode = undefined;
          neteaseModeError = "";
          lastNeteaseModeProbeKey = null;
        }
        mediaCapabilities = data.capabilities;
        shuffleActive = data.shuffleActive;
        repeatMode = data.repeatMode;

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
          seekRequestGate.invalidate();
          seekErrorMessage = "";
          neteaseModeError = "";
          pendingSeekUntil = 0;
          lastSongKey = currentSongKey;
          metadataImage?.abort(); metadataImage = undefined;
          metadataRecovery = { pending: false, nextAttempt: 0, attempts: 0 };
        }

        const titleChanged = trackTitle !== data.title;
        const artistChanged = artistName !== data.artist;

        const newCover = artworkValue(data) ?? (songChanged ? "" : rawCoverUrl);

        const coverChanged = songChanged || newCover !== rawCoverUrl;
        const coverAlreadyHandledForTrack = !songChanged && mainCoverLookupTrackKey === currentSongKey;

        if (titleChanged || artistChanged || coverChanged) {
          if (titleChanged) {
            trackTitle = data.title || "未知曲目";

          }
          if (artistChanged) {
            artistName = data.artist || "未知艺术家";
          }

          if (coverChanged && !coverAlreadyHandledForTrack) {
            metadataImage?.abort(); metadataImage = undefined;
            cancelMainCover(); hdArtworkResolved=false;
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
            coverChanged &&
            !coverAlreadyHandledForTrack &&
            (newCover || songChanged) &&
            $appSettings.enableHdCover &&
            data.title
          ) {
            const requestedTrackKey = currentSongKey;
            const requestedCover = newCover;
            mainCoverLookupTrackKey = requestedTrackKey;
            const probe = new AbortController();
            smtcCoverProbe = probe;
            void (async () => {
              let resolvedForTrack = false;
              try {
                const smtcAlreadyHighResolution = await isHighResolutionSmtcCover(requestedCover, probe.signal);
                if (lifetime.disposed || probe.signal.aborted || lastSongKey !== requestedTrackKey || !$appSettings.enableHdCover) return;
                if (smtcAlreadyHighResolution) {
                  hdArtworkResolved = true;
                  resolvedForTrack = true;
                  void syncFloatingMediaClock();
                  return;
                }

                const resolved = await mainCoverOwner
                  .request<import("$lib/api/types").ResolvedCover | null>({ title:data.title, artist:data.artist || "", source:data.source || currentSource },requestedCover);
                if (lifetime.disposed || probe.signal.aborted || !resolved || lastSongKey !== requestedTrackKey || !$appSettings.enableHdCover) return;
                const resolvedUrl = /^[a-z]:[\\/]/i.test(resolved.url)
                  ? convertFileSrc(resolved.url)
                  : resolved.url;
                pendingHdImage?.abort();
                const controller = new AbortController(); pendingHdImage = controller;
                let image: HTMLImageElement | undefined;
                try {
                  image = await loadImage(resolvedUrl, controller.signal);
                  if (lifetime.disposed || controller.signal.aborted || probe.signal.aborted || lastSongKey !== requestedTrackKey || !$appSettings.enableHdCover) return;
                  hdArtworkResolved = true;
                  resolvedForTrack = true;
                  artworkUrl = resolvedUrl;
                  void syncFloatingMediaClock();
                } finally {
                  image?.removeAttribute("src");
                  if (pendingHdImage === controller) pendingHdImage = undefined;
                }
              } catch {
                // The SMTC cover remains visible if quality probing or lookup fails.
              } finally {
                if (smtcCoverProbe === probe) {
                  smtcCoverProbe = undefined;
                  if (!resolvedForTrack && mainCoverLookupTrackKey === requestedTrackKey) mainCoverLookupTrackKey = null;
                }
              }
            })();
          }
        }

        void recoverMissingMetadata();
        rememberCurrentTrack(songChanged || !lastPersistedTrackSignature.endsWith(`|${isPlaying}`));
        void syncFloatingMediaClock();
      });
      cleanups.add(unlistenMediaUpdate);
      if (cleanups.disposed) return;
      // An already-mounted Floating window may have missed its ready event
      // while this App WebView was being reloaded. Its query listener is
      // installed before it announces ready, so this request is safe here.
      void emit(Events.FLOATING_MEDIA_READY_QUERY, { appSessionId: appMediaSessionId }).catch((error) => {
        if (!lifetime.disposed) logger.warn("查询浮窗媒体监听器状态失败", error);
      });
      const recoveryTimer = setInterval(() => void recoverMissingMetadata(), 5_000);
      cleanups.add(() => clearInterval(recoveryTimer));
    })().catch(error => { if (!lifetime.disposed) logger.error("初始化失败", error); });

    return () => {
      cleanups.dispose();
    };
  });

  onDestroy(() => {
    stopDebugFps();
    cancelMainCover();
    if (timerFinishedTimeout !== null) clearTimeout(timerFinishedTimeout);
    if (manualHideTimeout !== null) clearTimeout(manualHideTimeout);
  });

  function handleGlobalClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (showMonitorMenu && !target.closest(".relative")) {
      closeMonitorMenu();
    }
  }

  onMount(() => {
    win = getCurrentWindow();
    windowReady = true;
    console.log("[App.svelte] 窗口对象已初始化");

    document.addEventListener("click", handleGlobalClick);
    return () => {
      document.removeEventListener("click", handleGlobalClick);
    };
  });

  onMount(() => {
    // 初始设置由主启动流程统一加载；这里仅监听设置页实时广播。
    // Focus, pageshow and visibility recovery above handle normal display and
    // taskbar changes promptly. Keep only a low-frequency fallback for work
    // area changes that Windows does not surface to this WebView.
    let workAreaListenerDisposed = false;
    let unlistenWorkArea: (() => void) | undefined;
    void listen("isle-work-area-changed", scheduleResumeRecovery)
      .then((unlisten) => {
        if (workAreaListenerDisposed) unlisten();
        else unlistenWorkArea = unlisten;
      })
      .catch((error) => logger.warn("注册工作区变化监听失败，保留5分钟布局兜底", error));

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
    }, 5 * 60_000);

    // 所有捕获场景统一从 Capture Mode 状态进入，不各自维护隐藏逻辑。
    const unlistenCaptureMode = eventManager.on(
      Events.CAPTURE_MODE_CHANGED,
      (snapshot) => handleCaptureModeChange(snapshot as CaptureSnapshot),
    );

    let mouseMoveTimeout: ReturnType<typeof setTimeout> | null = null;
    const handleMouseMoveThrottled = (e: MouseEvent) => {
      // Fullscreen reveal is the only consumer of this document-wide mouse
      // stream. Gate before allocating a timer so the normal desktop path
      // does no debounce work at all.
      if (!$appSettings.captureHideOnFullscreen || !isFullscreenApp || captureSnapshot.screenshot) {
        if (mouseMoveTimeout) clearTimeout(mouseMoveTimeout);
        mouseMoveTimeout = null;
        return;
      }
      if (mouseMoveTimeout) {
        clearTimeout(mouseMoveTimeout);
      }
      mouseMoveTimeout = setTimeout(() => {
        mouseMoveTimeout = null;
        handleMouseMove(e);
      }, 100);
    };

    document.addEventListener("mousemove", handleMouseMoveThrottled);

    return () => {
      workAreaListenerDisposed = true;
      unlistenWorkArea?.();
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
  style={`width:${renderedViewport.host.width}px;height:${renderedViewport.host.height}px`}
>
  <IslandSurface
    media={islandMedia}
    {neteaseModeBusy}
    neteaseModeError={neteaseModeError}
    mode={$islandMode}
    islandStyle={renderedIslandStyle}
    edge={renderedIslandEdge}
    position={displayedPosition}
    expandedLength={expandedLengthFor(renderedIslandStyle, $appSettings)}
    expandedRadius={expandedRadiusFor(renderedIslandStyle, $appSettings)}
    collapsedEdgeShoulderRadius={$appSettings.collapsedEdgeShoulderRadius ?? 8}
    expandedEdgeShoulderRadius={$appSettings.expandedEdgeShoulderRadius ?? 32}
    expandedScale={renderedViewport.scale}
    compactLength={compactLengthFor(renderedIslandStyle, $appSettings)}
    idle={showIdle}
    idleTime={currentTime}
    idleWeatherTemperature={weatherSnapshot.weatherTemperature}
    idleWeatherCode={weatherSnapshot.weatherCode}
    idleWeatherForecast={weatherSnapshot.weatherForecast}
    widgetsEnabled={$appSettings.islandWidgetsEnabled}
    onWidgetsToggle={async enabled => {await settingsApi.updateSettings({islandWidgetsEnabled:enabled});appSettings.update(settings=>({...settings,islandWidgetsEnabled:enabled}));}}
    islandWidgets={$appSettings.islandWidgets}
    onWidgetsSave={saveWidgets}
    onLayoutPreview={(layout, editing) => { layoutPreview = layout; layoutEditing = editing; }}
    {lastPlayedMedia}
    canResumeLastTrack={Boolean(lastPlayedMedia && playerApps[lastPlayedMedia.source])}
    {resumingLastTrack}
    showSpectrum={$appSettings.showSpectrum}
    spectrumMode={$appSettings.spectrumMode}
    enableAnimations={$appSettings.enableAnimations}
    reduceAnimations={$appSettings.reduceAnimations || viewportClipped}
    {spectrumTopColor}
    {spectrumBottomColor}
    showTime={false}
    timeText={currentTime}
    showDebugInfo={$appSettings.showDebugInfo}
    debugLines={[`${fps} FPS`, currentSource, `${Math.round(displayedPosition)} ms`, isHidden ? "hidden" : `${$islandMode} / ${$islandView.type}`]}
    onToggle={toggleIsland}
    onOpenPlayer={openCurrentPlayer}
    onResumeLastTrack={resumeLastTrack}
    onMediaAction={(action) => handleMediaAction(action)}
    onSeek={handleSeek}
    {seekBusy}
    seekError={seekErrorMessage}
    onToggleFloating={toggleFloatingWindow}
    onHideForTenSeconds={hideForTenSeconds}
    keepIslandExpanded={$appSettings.keepIslandExpanded}
    onToggleKeepIslandExpanded={async () => {
      const keepIslandExpanded = !$appSettings.keepIslandExpanded;
      await settingsApi.updateSettings({ keepIslandExpanded });
      appSettings.update(settings => ({ ...settings, keepIslandExpanded }));
    }}
    onSettingsToggle={showSettingsWindow}
    enabledTools={enabledFeatureTools}
    showCustomFunctionPanel={$appSettings.showCustomFunctionPanel}
    visible={pageVisible && !isHidden}
    onPageChange={(page) => transitionIsland({ type: "select-page", page })}
    weatherCity={$appSettings.weatherLocation?.name ?? ""}
    weatherUpdatedAt={weatherSnapshot.weatherUpdatedAt}
    {weatherLoading}
    {weatherFailed}
    onPanelActivity={(active) => panelInteracting = active}
    onHoverChange={(value) => transitionIsland({ type: value ? "hover-enter" : "hover-leave" })}
    onRegionChange={applyIslandRegion}
    {systemAudio}
    {audioDevices}
    {audioDeviceSwitching}
    onAudioOpen={() => { void handleAudioOpen(); }}
    wheelVolumeControl={$appSettings.wheelVolumeControl}
    wheelVolumeDisplay={wheelVolumeDisplay}
    volumeProtectionPrompt={volumeProtectionPrompt}
    volumeProtectionLimit={$appSettings.volumeProtectionLimit}
    onConfirmVolumeIncrease={confirmVolumeIncrease}
    onCancelVolumeIncrease={cancelVolumeIncrease}
    onWheelVolume={(delta) => { void handleWheelVolume(delta); }}
    onAudioVolume={handleAudioVolume}
    onAudioDevice={handleAudioDevice}
    timerStatus={countdown.status}
    {timerRemainingMs}
    timerDurationMs={countdown.durationMs}
    timerLabel={countdown.label}
    {timerFinished}
    onTimerFinishedDismiss={clearTimerFinished}
    clockText={currentTime}
    clockTimeZone={$appSettings.clockTimeZone}
    onTimerStart={handleTimerStart}
    onTimerPause={handleTimerPause}
    onTimerResume={handleTimerResume}
    onTimerAdjust={handleTimerAdjust}
    onTimerReset={handleTimerReset}
  />
  {#if notificationCurrent && $islandMode === "expanded" && pageVisible && !isHidden}
    <div class="notification-overlay" role="status" aria-live="polite">
      <strong>{notificationCurrent.app}</strong>
      {#if notificationCurrent.title}<span>{notificationCurrent.title}</span>{/if}
      {#if notificationCurrent.body}<small>{notificationCurrent.body}</small>{/if}
    </div>
  {/if}
</div>


<style>
  :global(*) { box-sizing: border-box; }
  :global(html, body, #app) { margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; background: transparent; -webkit-font-smoothing: antialiased; }
  .notification-overlay { position: absolute; z-index: 20; top: 12px; width: min(80%, 480px); min-height: 96px; padding: 12px 18px; border-radius: 20px; background: #151515; color: white; display: flex; flex-direction: column; gap: 5px; pointer-events: none; overflow: hidden; }
  .notification-overlay strong, .notification-overlay span, .notification-overlay small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .notification-overlay small { color: #ccc; }
  .fixed-host { display: flex; align-items: flex-start; justify-content: center; box-sizing: border-box; background: transparent; pointer-events: none; }
  .fixed-host :global(.island-surface) { pointer-events: auto; }
</style>
