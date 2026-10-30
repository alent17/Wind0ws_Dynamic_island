<script lang="ts">
  import { clearNeteaseModes } from "$lib/neteaseModeState";
  import { onMount, onDestroy } from "svelte";
  import { loadImage } from "$lib/imageLoad";
  import { isHighResolutionSmtcCover } from "$lib/smtcArtworkQuality";
  import { artworkFillFromImage } from "$lib/artworkFill";
  import { appOverlayDecision, artworkValue, mergeProgressAnchorFields, nativeStampDecision, readNativeMediaStamp, snapshotDecision, withoutArtwork, type ArtworkSnapshotToken, type NativeMediaStamp } from "$lib/mediaArtwork";
  import { createCanvasArtworkRenderer, type CanvasArtworkRequest } from "$lib/canvasArtwork";
  import { createArtworkOwner } from "$lib/artworkOwner";
  import { ArtworkResultCache, artworkCacheKey, artworkCanvasSize } from "$lib/artworkBudget";
  import { createSeekHandler, createSeekRequestGate, formatSeekError, seekSnapshotAllowsRollback } from "$lib/seekRequest";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { eventManager, onMediaUpdate } from "./utils/eventManager";
  import { Events } from "./utils/eventConstants";
  import { isTransportDiagnosticsEnabled, recordTransportCounter } from "$lib/transportDiagnostics";
  import { mediaApi } from "$lib/api/media";
  import { windowApi } from "$lib/api/window";
  import { settingsApi } from "$lib/api/settings";
  import { connectSettingsStore, settingsStore } from "$lib/settingsStore";
  import { applyAppFont } from "$lib/font";
  import { locale, setLocale, translate, type TranslationKey } from "$lib/i18n";
  import {
    activeCaptureReasons,
    EMPTY_CAPTURE_SNAPSHOT,
    type CaptureSnapshot,
  } from "$lib/captureMode";
  import MediaProgress from "$lib/components/media/MediaProgress.svelte";
  import NeteaseConnectedModeControl from "$lib/components/media/NeteaseConnectedModeControl.svelte";
  import SmtcModeControls from "$lib/components/media/SmtcModeControls.svelte";
  import NeteaseLikeControl from "$lib/components/media/NeteaseLikeControl.svelte";
  import { clampSeekPosition, mediaTrackKey, projectedPosition, reconcileReportedPosition } from "$lib/mediaClock";
  import { DEFAULT_SETTINGS, type MediaState, type AppSettings } from "$lib/api/types";
  import {
    Play,
    Pause,
    SkipBack,
    SkipForward,
    X,
    Pin,
    Lock,
    LockOpen,
  } from "lucide-svelte";

  interface WindowSize {
    width: number;
    height: number;
  }

  type ResizeDirection =
    | "East"
    | "North"
    | "South"
    | "West"
    | "NorthWest"
    | "NorthEast"
    | "SouthWest"
    | "SouthEast";


  const INFO_HEIGHT = 64;
  const ART_MAX_SIZE = 640;
  const ART_SIDE_GUTTER = 16;
  const ART_VERTICAL_GUTTER = 8;

  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);
  let PLACEHOLDER_TITLE = t("waitingPlayback");
  let PLACEHOLDER_ARTIST = t("unknownArtist");
  let mediaState = $state<MediaState>({
    title: PLACEHOLDER_TITLE,
    artist: PLACEHOLDER_ARTIST,
    albumArt: "",
    isPlaying: false,
    positionMs: 0,
    durationMs: 0,
    lastUpdatedTimestamp: 0,
    source: "",
    sourceDisplay: "",
  });

  let currentTrackKey = $state("");
  let lastReportedPosition: number | undefined;
  let pendingSeekUntil = 0;
  const seekRequestGate = createSeekRequestGate();
  let nativeSeekSnapshotRevision = 0;
  let seekError = $state("");
  let latestNativeSeekSnapshot: { epoch: string; trackGeneration: number; title: string; artist: string; positionMs: number; durationMs: number } | undefined;
  let displayCover = $state("");
  let previousCover = $state("");
  let systemCover = "";
  let resolvedTrackCover = "";
  let resolvedTrackKey = "";
  let pendingResolvedCover = "";
  let albumFill = $state("#000000");
  let isHovered = $state(false);
  let isWindowFullscreen = $state(false);
  let hoverLeaveTimeout: ReturnType<typeof setTimeout> | null = null;
  let captureSnapshot = $state<CaptureSnapshot>({ ...EMPTY_CAPTURE_SNAPSHOT });
  const capturePreferences = settingsStore;
  let isCaptureHidden = $derived(
    activeCaptureReasons(captureSnapshot, $capturePreferences).length > 0,
  );
  let disposed = $state(false);
  const mediaInstanceToken = crypto.randomUUID();
  let mediaReady = false;
  let slideDirection = $state<"left" | "right" | "">("");
  let isAnimating = $state(false); // 动画进行中标志
  let windowSize = $state<WindowSize>({ width: 0, height: 0 });
  let albumArtSize = $derived(
    Math.max(
      50,
      Math.min(
        ART_MAX_SIZE,
        windowSize.width - ART_SIDE_GUTTER,
        windowSize.height - INFO_HEIGHT - ART_VERTICAL_GUTTER,
      ),
    ),
  );
  let isCompactCover = $derived(
    windowSize.width <= 200.5 && windowSize.height <= 200.5,
  );
  let retainCompactCover = $state(false);
  let clockNow = $state(Date.now());
  let pageVisible = $state(true);
  let visualWorkVisible = $derived(pageVisible && !isCaptureHidden);
  let displayedPosition = $derived(projectedPosition(mediaState, clockNow));
  function handleVisibilityChange() {
    pageVisible = document.visibilityState === "visible";
    if (!mediaReady || disposed) return;
    if (pageVisible) announceMediaReady();
    else announceMediaNotReady("hidden");
  }

  // MV 播放相关
  let isMVPlaybackEnabled = $state(false); // MV 播放功能是否启用
  let mvUrl = $state(""); // MV 视频链接
  let isPlayingMV = $state(false); // 是否正在播放 MV

  // 半色调网点效果
  let halftoneOverlayVisible = $state(false);

  // 置顶状态
  let isAlwaysOnTop = $state(false);

  // 锁定悬浮窗（禁止移动）
  let isFloatingWindowLocked = $state(false); // 初始值，会在 onMount 中从设置加载

  // 专辑封面设置
  let enableHDCover = $state(true); // 高清封面获取
  let floatingCircularAlbum = $state(DEFAULT_SETTINGS.floatingCircularAlbum);
  let shouldPlayMv = $derived(
    !disposed && visualWorkVisible && isMVPlaybackEnabled && isPlayingMV && Boolean(mvUrl)
      && !floatingCircularAlbum && mediaState.isPlaying,
  );
  let mvVideo = $state<HTMLVideoElement>();
  let mvPlaybackRevision = 0;
  let mvPendingPlayRevision: number | undefined;
  let circularAlbumArtSize = $derived(Math.max(50, Math.round(albumArtSize * 0.86)));
  let enablePixelArt = $state(false); // 像素化封面

  let unlisten: () => void;
  let unlistenIslandMediaSync: () => void;
  let currentNativeStamp: NativeMediaStamp | undefined;
  let nativeStampedMode = false;
  const retiredNativeEpochs = new Set<string>();
  let currentAppSessionId: string | undefined;
  let appSessionMode = false;
  const retiredAppSessions = new Set<string>();
  let lastAppOverlaySequence = 0;
  let appliedOverlayBasis: string | undefined;
  let mediaEventRevision = 0;
  let artworkRevision = 0;
  let mediaTrackGeneration = 0;
  let unlistenResize: () => void;
  let savePositionTimeout: ReturnType<typeof setTimeout> | null = null;
  let coverRequestId = 0;
  let durationRequestId = 0;
  let mvRequestId = 0;

  function setCircularAlbum(enabled: boolean) {
    const wasCircular = floatingCircularAlbum;
    floatingCircularAlbum = enabled;
    if (enabled) {
      mvRequestId += 1;
      isPlayingMV = false;
      mvUrl = "";
    } else if (wasCircular && isMVPlaybackEnabled && currentTrackKey) {
      requestMVForCurrentTrack(currentTrackKey, mediaState.title, mediaState.artist);
    }
    syncMvPlayback();
  }

  function handlePointerEnter() {
    if (hoverLeaveTimeout) {
      clearTimeout(hoverLeaveTimeout);
      hoverLeaveTimeout = null;
    }
    isHovered = true;
  }

  function handlePointerLeave() {
    if (hoverLeaveTimeout) clearTimeout(hoverLeaveTimeout);
    hoverLeaveTimeout = setTimeout(() => {
      isHovered = false;
      hoverLeaveTimeout = null;
    }, 80);
  }

  async function refreshFullscreenState(appWindow = getCurrentWindow()) {
    try {
      isWindowFullscreen = await appWindow.isFullscreen();
    } catch (error) {
      console.error("[悬浮窗] 读取全屏状态失败:", error);
      isWindowFullscreen = false;
    }
  }

  let coverPreload: AbortController | undefined;
  let smtcCoverProbe: AbortController | undefined;
  let coverLookupTrackKey = "";
  function cancelCoverPreload() { coverPreload?.abort(); coverPreload = undefined; pendingResolvedCover = ""; }
  async function preloadCover(url: string): Promise<boolean> {
    cancelCoverPreload();
    if (!url || disposed) return false;
    const controller = new AbortController(); coverPreload = controller;
    try {
      const image = await loadImage(url, controller.signal);
      image.removeAttribute("src");
      return !disposed && !controller.signal.aborted;
    } catch { return false; }
    finally { if (coverPreload === controller) coverPreload = undefined; }
  }

  async function resolveTrackCover(
    trackKey: string,
    title: string,
    artist: string,
    source: string,
    fallbackCover: string,
    alreadyResolved = false,
  ) {
    if (!alreadyResolved && resolvedTrackKey === trackKey && resolvedTrackCover) {
      systemCover = fallbackCover || systemCover;
      transitionCover(resolvedTrackCover, "left");
      return;
    }
    if (!alreadyResolved && smtcCoverProbe && coverLookupTrackKey === trackKey) return;

    smtcCoverProbe?.abort(); smtcCoverProbe = undefined;
    coverLookupTrackKey = "";
    coverOwner.cancel(); cancelCoverPreload();
    const requestId = ++coverRequestId;

    if (alreadyResolved) {
      if (enableHDCover && fallbackCover) await acceptResolvedCover(trackKey, fallbackCover, requestId);
      else transitionCover(systemCover, "left");
      return;
    }
    const changedBase = Boolean(systemCover) && systemCover !== fallbackCover;
    systemCover = fallbackCover;
    if (changedBase) { resolvedTrackCover = ""; resolvedTrackKey = ""; }
    // Re-enabling HD reuses the ready image without briefly downgrading it.
    if (enableHDCover && resolvedTrackKey === trackKey && resolvedTrackCover) {
      transitionCover(resolvedTrackCover, "left");
      return;
    }
    transitionCover(systemCover, "left");
    if (!enableHDCover || !title) return;

    const probe = new AbortController();
    smtcCoverProbe = probe;
    coverLookupTrackKey = trackKey;
    try {
      const smtcAlreadyHighResolution = await isHighResolutionSmtcCover(fallbackCover, probe.signal);
      if (
        disposed ||
        probe.signal.aborted ||
        requestId !== coverRequestId ||
        trackKey !== currentTrackKey ||
        !enableHDCover
      ) {
        return;
      }
      if (smtcAlreadyHighResolution) {
        resolvedTrackCover = fallbackCover;
        resolvedTrackKey = trackKey;
        return;
      }

      const resolved = await coverOwner.request<import("$lib/api/types").ResolvedCover | null>({ title, artist, source: source || "generic" }, fallbackCover);
      if (
        !resolved?.url ||
        probe.signal.aborted ||
        requestId !== coverRequestId ||
        trackKey !== currentTrackKey ||
        !enableHDCover
      ) {
        return;
      }

      const resolvedUrl = /^[a-z]:[\\/]/i.test(resolved.url)
        ? convertFileSrc(resolved.url)
        : resolved.url;
      await acceptResolvedCover(trackKey, resolvedUrl, requestId);
    } catch (error) {
      if (!(error instanceof DOMException && error.name === "AbortError") && !disposed && requestId === coverRequestId && trackKey === currentTrackKey) {
        console.warn("[高清封面] 获取失败，继续使用系统封面:", error);
      }
    } finally {
      if (smtcCoverProbe === probe) {
        smtcCoverProbe = undefined;
        if (resolvedTrackKey !== trackKey && coverLookupTrackKey === trackKey) coverLookupTrackKey = "";
      }
    }
  }

  async function acceptResolvedCover(trackKey: string, cover: string, requestId = ++coverRequestId) {
    if (!enableHDCover || disposed) return;
    if (cover === displayCover && resolvedTrackKey === trackKey) return;
    const loading = preloadCover(cover);
    pendingResolvedCover = cover;
    try {
      if (await loading) {
        if (!disposed && enableHDCover && requestId === coverRequestId && trackKey === currentTrackKey) {
          resolvedTrackCover = cover;
          resolvedTrackKey = trackKey;
          transitionCover(cover, "left");
        }
      }
    } finally {
      if (requestId === coverRequestId && pendingResolvedCover === cover) pendingResolvedCover = "";
    }
  }

  function setHighDefinitionCover(enabled: boolean) {
    if (enableHDCover === enabled) return;
    enableHDCover = enabled;
    smtcCoverProbe?.abort(); smtcCoverProbe = undefined;
    coverLookupTrackKey = "";
    coverRequestId++; coverOwner.cancel(); cancelCoverPreload();
    if (!currentTrackKey) return;
    if (enabled) {
      void resolveTrackCover(currentTrackKey, mediaState.title, mediaState.artist, mediaState.source, systemCover);
    } else {
      transitionCover(systemCover, "left");
    }
  }

  async function resolveMissingDuration(
    trackKey: string,
    title: string,
    artist: string,
  ) {
    if (!title || mediaState.durationMs > 0) return;
    const requestId = ++durationRequestId;
    try {
      const songInfo = await mediaApi.getNeteaseSongInfo(title, artist || "");
      if (
        requestId === durationRequestId &&
        trackKey === currentTrackKey &&
        songInfo?.duration &&
        songInfo.duration > 0 &&
        mediaState.durationMs === 0
      ) {
        mediaState.durationMs = songInfo.duration;
        mediaState.lastUpdatedTimestamp = Date.now();
      }
    } catch (error) {
      console.warn("[进度] 无法补全歌曲时长，继续等待播放器上报:", error);
    }
  }

  const coverOwner = createArtworkOwner("cover");
  const pixelOwner = createArtworkOwner("pixel");

  // 封面切换函数（无动画）
  function transitionCover(
    newCover: string,
    direction: "left" | "right" = "left",
  ) {
    displayCover = newCover;
    if (!newCover) albumFill = "#000000";
    previousCover = "";
    slideDirection = "";
    isAnimating = false;
  }

  // 从 Apple Music 获取 MV 链接（使用本地缓存）
  async function fetchMVFromAppleMusic(title: string, artist: string) {
    if (!isMVPlaybackEnabled || floatingCircularAlbum) return null;

    try {
      const query = encodeURIComponent(`${title} ${artist}`);
      const res = await fetch(
        `https://itunes.apple.com/search?term=${query}&limit=1&media=musicVideo`,
        {
          signal: pageRequests.signal,
          headers: {
            Referer: "https://music.apple.com",
            "User-Agent":
              "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
          },
        },
      );

      const contentType = res.headers.get("content-type");
      const isJSON =
        contentType &&
        (contentType.includes("application/json") ||
          contentType.includes("text/javascript"));

      if (!isJSON) {
        return null;
      }

      // 使用 text() 方法读取内容，然后解析为 JSON
      // 这样可以处理非标准的 Content-Type
      const text = await res.text();
      const data = JSON.parse(text);

      if (data.results?.length > 0) {
        const mvData = data.results[0];
        const previewUrl = mvData.previewUrl; // Apple Music 提供的 MV 预览链接
        if (!previewUrl) return null;

        // 先检查缓存
        try {
          const cachedPath = await invoke<string | null>("get_cached_media", {
            url: previewUrl,
          });
          if (cachedPath) {
            // 使用 convertFileSrc 转换为可访问的 URL
            return convertFileSrc(cachedPath);
          }
        } catch (cacheError) {
        }

        // 下载并缓存 MV
        try {
          const cachedPath = await invoke<string>("download_and_cache", {
            url: previewUrl,
            contentType: "video/mp4",
          });
          const safeUrl = convertFileSrc(cachedPath);
          return safeUrl;
        } catch (cacheError) {
          console.error("[MV] Apple Music 本地缓存失败:", cacheError);
          return null;
        }
      }
      return null;
    } catch (error) {
      console.error("[MV] 获取失败:", error);
      return null;
    }
  }

  async function cacheMV(url: string): Promise<string | null> {
    if (!url) return null;
    try {
      const cachedPath = await invoke<string>("download_and_cache", {
        url,
        contentType: "video/mp4",
      });
      return convertFileSrc(cachedPath);
    } catch (error) {
      console.error("[MV] 网易云 MV 本地缓存失败:", error);
      return null;
    }
  }

  async function fetchMVPreview(title: string, artist: string) {
    if (!isMVPlaybackEnabled || floatingCircularAlbum || !title) return null;
    try {
      // Restore the project's original NetEase MV path first. It works for
      // full MVs as well; the player below intentionally loops only 30 s.
      const song = await mediaApi.getNeteaseSongInfo(title, artist);
      const mvUrl = song?.mvUrl || (song?.mvId && song.mvId > 0
        ? await mediaApi.getNeteaseMvUrl(song.mvId) : null);
      if (mvUrl) {
        const cached = await cacheMV(mvUrl);
        if (cached) return cached;
      }
    } catch (error) {
      console.warn("[MV] 网易云匹配失败，尝试 Apple Music:", error);
    }
    return fetchMVFromAppleMusic(title, artist);
  }

  function requestMVForCurrentTrack(trackKey: string, title: string, artist: string) {
    if (floatingCircularAlbum || !isMVPlaybackEnabled) return;
    const requestId = ++mvRequestId;
    void fetchMVPreview(title, artist)
      .then((mvLink) => {
        if (!mvLink || requestId !== mvRequestId || currentTrackKey !== trackKey || !isMVPlaybackEnabled || floatingCircularAlbum) return;
        mvUrl = mvLink;
        isPlayingMV = true;
      })
      .catch((error) => console.error("[MV] 请求失败:", error));
  }

  function keepMVInPreview(video: HTMLVideoElement) {
    if (video !== mvVideo) return;
    if (video.currentTime >= 30 || video.ended) {
      video.currentTime = 0;
      syncMvPlayback(video);
    }
  }

  // Releasing the source when this element leaves the DOM lets the WebView
  // discard its decoder and buffered frames after a track or mode change.
  function releaseVideo(video: HTMLVideoElement) {
    return { destroy() { video.pause(); video.removeAttribute("src"); video.load(); } };
  }

  function syncMvPlayback(video = mvVideo) {
    if (disposed || !shouldPlayMv) {
      mvPlaybackRevision += 1;
      mvPendingPlayRevision = undefined;
      video?.pause();
      return;
    }
    if (!video) {
      mvPlaybackRevision += 1;
      mvPendingPlayRevision = undefined;
      return;
    }
    if (mvPendingPlayRevision !== undefined || !video.paused) return;

    const revision = ++mvPlaybackRevision;
    mvPendingPlayRevision = revision;
    void video.play().then(() => {
      // A play request can settle after capture mode, document visibility, or
      // the selected track has changed. Recheck the current owner before it
      // is allowed to keep decoding in the hidden window.
      if (mvPendingPlayRevision === revision) mvPendingPlayRevision = undefined;
      if (disposed || video !== mvVideo || !shouldPlayMv) {
        video.pause();
      }
    }).catch((error) => {
      if (mvPendingPlayRevision === revision) mvPendingPlayRevision = undefined;
      if (!disposed && revision === mvPlaybackRevision && shouldPlayMv) {
        console.error("[MV] 恢复播放失败:", error);
      }
    });
  }

  const pageRequests = new AbortController();
  function trackListener(dispose: () => void) {
    if (disposed) { dispose(); throw new DOMException("Page disposed", "AbortError"); }
    eventListeners.push(dispose);
  }
  onMount(() => {
    void initialize().catch(error => { if (!disposed) console.error(error); });
  });
  async function initialize() {
    pageVisible = document.visibilityState === "visible";
    document.addEventListener("visibilitychange", handleVisibilityChange);
    eventListeners = [];

    // 读取设置
    try {
      const settingsConnection = await connectSettingsStore((value) => ({
        ...DEFAULT_SETTINGS,
        ...value,
      }));
      trackListener(settingsConnection.disconnect);
      const settings = settingsConnection.settings;
      applyAppFont(settings.fontId);
      setLocale(settings.language);
      PLACEHOLDER_TITLE = t("waitingPlayback");
      PLACEHOLDER_ARTIST = t("unknownArtist");
      isMVPlaybackEnabled = settings.enableMvPlayback ?? false;

      // 加载置顶设置
      isAlwaysOnTop = settings.floatingWindowAlwaysOnTop ?? DEFAULT_SETTINGS.floatingWindowAlwaysOnTop;

      // 加载锁定悬浮窗设置
      isFloatingWindowLocked = settings.lockFloatingWindow ?? false;

      // 加载专辑封面设置
      enableHDCover = settings.enableHdCover ?? true;
      setCircularAlbum(settings.floatingCircularAlbum ?? DEFAULT_SETTINGS.floatingCircularAlbum);
      enablePixelArt = settings.enablePixelArt ?? false;
      halftoneOverlayVisible = settings.enableHalftone ?? false;

      // 设置窗口是否可调整大小
      await windowApi.setFloatingWindowResizable(!isFloatingWindowLocked);
    } catch (error) {
      console.error("[设置] 读取失败:", error);
      isMVPlaybackEnabled = true;
      isAlwaysOnTop = DEFAULT_SETTINGS.floatingWindowAlwaysOnTop;
    }

    if (disposed) return;
    // 初始化事件监听器管理器

    // 监听 MV 播放设置变化事件
    const unlistenMVChange = await eventManager.on(
      Events.MV_PLAYBACK_CHANGED,
      ({ enable }: any) => {
        isMVPlaybackEnabled = enable;
        // 如果关闭了 MV 播放，停止当前播放
        if (!isMVPlaybackEnabled || floatingCircularAlbum) {
          mvRequestId += 1;
          isPlayingMV = false;
          mvUrl = "";
        } else if (currentTrackKey) {
          requestMVForCurrentTrack(currentTrackKey, mediaState.title, mediaState.artist);
        }
        syncMvPlayback();
      },
    );
    trackListener(unlistenMVChange);

    // 监听锁定悬浮窗设置变化事件
    const unlistenLockChange = await eventManager.on(
      Events.LOCK_FLOATING_WINDOW_CHANGED,
      ({ lock }: any) => {
        isFloatingWindowLocked = lock;

        // 同时设置窗口是否可调整大小
        windowApi
          .setFloatingWindowResizable(!isFloatingWindowLocked)
          .catch((err) => {
            console.error("[锁定] 设置窗口可调整大小失败:", err);
          });
      },
    );
    trackListener(unlistenLockChange);

    // 监听高清封面获取设置变化事件
    const unlistenHDCoverChange = await eventManager.on(
      Events.HD_COVER_CHANGED,
      ({ enableHDCover: enabled }: any) => {
        setHighDefinitionCover(Boolean(enabled));
      },
    );
    trackListener(unlistenHDCoverChange);

    // 监听像素化封面设置变化事件
    const unlistenPixelArtChange = await eventManager.on(
      Events.PIXEL_ART_CHANGED,
      ({ enablePixelArt: enabled }: any) => {
        enablePixelArt = enabled;
      },
    );
    trackListener(unlistenPixelArtChange);

    // 监听网点效果设置变化事件
    const unlistenHalftoneChange = await eventManager.on(
      Events.HALFTONE_CHANGED,
      ({ enableHalftone: enabled }: any) => {
        halftoneOverlayVisible = enabled;
      },
    );
    trackListener(unlistenHalftoneChange);
    const unlistenSettingsChange = await eventManager.on(Events.SETTINGS_UPDATED, (value: AppSettings) => {
      if (value?.fontId) applyAppFont(value.fontId);
      if (value?.language) setLocale(value.language);
      if (value) {
        if (typeof value.enableHdCover === "boolean") setHighDefinitionCover(value.enableHdCover);
        isMVPlaybackEnabled = value.enableMvPlayback ?? DEFAULT_SETTINGS.enableMvPlayback;
        setCircularAlbum(value.floatingCircularAlbum ?? DEFAULT_SETTINGS.floatingCircularAlbum);
      }
    });
    trackListener(unlistenSettingsChange);
    const unlistenAlwaysOnTopChange = await eventManager.on(
      Events.FLOATING_WINDOW_ALWAYS_ON_TOP_CHANGED,
      ({ isAlwaysOnTop: enabled }: any) => {
        isAlwaysOnTop = Boolean(enabled);
      },
    );
    trackListener(unlistenAlwaysOnTopChange);
    const unlistenCaptureMode = await eventManager.on(
      Events.CAPTURE_MODE_CHANGED,
      (snapshot: CaptureSnapshot) => {
        captureSnapshot = { ...EMPTY_CAPTURE_SNAPSHOT, ...snapshot };
        syncMvPlayback();
      },
    );
    trackListener(unlistenCaptureMode);

    const appWindow = getCurrentWindow();
    await refreshFullscreenState(appWindow);
    windowSize = { width: window.innerWidth, height: window.innerHeight };

    // 监听窗口大小变化
    unlistenResize = await appWindow.onResized(({ payload }) => {
      void refreshFullscreenState(appWindow);
      // 锁定时忽略大小变化
      if (isFloatingWindowLocked) {
        return;
      }

      // Tauri reports physical pixels here, while the 100px layout breakpoint
      // is expressed in CSS pixels. Read the viewport to stay correct on HiDPI.
      if (isCompactCover && (window.innerWidth > 200.5 || window.innerHeight > 200.5)) {
        retainCompactCover = true;
      } else if (window.innerWidth <= 200.5 && window.innerHeight <= 200.5) {
        retainCompactCover = false;
      }
      windowSize = { width: window.innerWidth, height: window.innerHeight };

      // 防抖保存位置和大小
      if (savePositionTimeout) clearTimeout(savePositionTimeout);
      savePositionTimeout = setTimeout(async () => {
        try {
          const position = await appWindow.outerPosition();
          const currentSize = await appWindow.innerSize();
          await windowApi.saveFloatingWindowPosition(
            Math.round(position.x),
            Math.round(position.y),
            currentSize.width,
            currentSize.height,
          );
        } catch (error) {
          console.error("[悬浮窗] 保存位置失败:", error);
        }
      }, 500); // 500ms 防抖
    });

    trackListener(unlistenResize);
    // 监听窗口位置变化
    const unlistenMoved = await appWindow.onMoved(({ payload }) => {
      // 防抖保存位置和大小
      if (savePositionTimeout) clearTimeout(savePositionTimeout);
      savePositionTimeout = setTimeout(async () => {
        try {
          const position = await appWindow.outerPosition();
          const currentSize = await appWindow.innerSize();
          await windowApi.saveFloatingWindowPosition(
            Math.round(position.x),
            Math.round(position.y),
            currentSize.width,
            currentSize.height,
          );
        } catch (error) {
          console.error("[悬浮窗] 保存位置失败:", error);
        }
      }, 500); // 500ms 防抖
    });

    trackListener(unlistenMoved);
    window.addEventListener("blur", handlePointerLeave);
    document.addEventListener("mouseleave", handlePointerLeave);



    // 监听媒体更新事件（已内置节流）
    const traceMedia = (origin: string, outcome: string, payload: any) => {
      const trace = (window as any).__ISLE_MEDIA_TRACE__;
      if (!Array.isArray(trace)) return;
      trace.push({ at: Date.now(), origin, outcome,
        title: String(payload.title || "").slice(0, 256), source: payload.source,
        coverPresent: Object.hasOwn(payload, "albumArt"), coverChars: payload.albumArt?.length || 0,
        currentTrack: currentTrackKey.slice(0, 512), displayCoverChars: displayCover.length });
      if (trace.length > 128) trace.splice(0, trace.length - 128);
    };
    const beginAppSession = (sessionId: unknown) => {
      if (typeof sessionId !== "string" || !sessionId || retiredAppSessions.has(sessionId)) return false;
      if (sessionId !== currentAppSessionId) {
        if (currentAppSessionId) retiredAppSessions.add(currentAppSessionId);
        currentAppSessionId = sessionId;
        appSessionMode = true;
        lastAppOverlaySequence = 0;
      }
      return true;
    };
    const nativeBasisKey = (stamp: NativeMediaStamp | undefined) => stamp
      ? `${stamp.nativeEpoch}\u0000${stamp.nativeRevision}`
      : undefined;
    const applyAppOverlay = (payload: any, expectedTrackKey: string) => {
      const overlay = payload.appOverlay;
      const sequence = payload.overlaySeq;
      const decision = appOverlayDecision({
        incomingSession: payload.appSessionId,
        currentSession: appSessionMode ? currentAppSessionId : undefined,
        retiredSessions: retiredAppSessions,
        incomingSequence: sequence,
        currentSequence: lastAppOverlaySequence,
        basisMatches: Boolean(currentNativeStamp && overlay &&
          overlay.nativeEpoch === currentNativeStamp.nativeEpoch &&
          overlay.nativeRevision === currentNativeStamp.nativeRevision),
        trackMatches: Boolean(overlay && overlay.trackKey === expectedTrackKey && expectedTrackKey === currentTrackKey),
      });
      if (decision !== "accept") {
        traceMedia("island-sync", `rejected-app-overlay-${decision}`, payload);
        return false;
      }
      lastAppOverlaySequence = Number(sequence);
      mediaState = {
        ...mediaState,
        title: typeof overlay.title === "string" ? overlay.title : mediaState.title,
        artist: typeof overlay.artist === "string" ? overlay.artist : mediaState.artist,
        positionMs: Date.now() < pendingSeekUntil
          ? projectedPosition(mediaState, Date.now())
          : Number(overlay.positionMs) || 0,
        lastUpdatedTimestamp: Number(overlay.lastUpdatedTimestamp) || Date.now(),
      };
      receiveSameTrackArtwork(overlay);
      appliedOverlayBasis = nativeBasisKey(currentNativeStamp);
      traceMedia("island-sync", "applied-app-overlay", payload);
      return true;
    };
    const receiveSameTrackArtwork = (payload: any) => {
      const cover = artworkValue(payload);
      if (cover === undefined) return;
      if (payload.artworkResolved && cover) {
        if (!enableHDCover) return;
        if (pendingResolvedCover === cover) return;
        smtcCoverProbe?.abort(); smtcCoverProbe = undefined;
        coverLookupTrackKey = "";
        if (displayCover === cover) {
          coverOwner.cancel(); coverRequestId++;
          resolvedTrackCover = cover; resolvedTrackKey = currentTrackKey;
          return;
        }
        coverOwner.cancel(); cancelCoverPreload(); coverRequestId++;
        if (displayCover !== cover) void acceptResolvedCover(currentTrackKey, cover, coverRequestId);
        return;
      }
      if (cover && cover === mediaState.albumArt && displayCover) return;
      artworkRevision++;
      mediaState.albumArt = cover;
      if (!cover) {
        systemCover = ""; resolvedTrackCover = ""; resolvedTrackKey = "";
        smtcCoverProbe?.abort(); smtcCoverProbe = undefined;
        coverLookupTrackKey = "";
        coverRequestId++; coverOwner.cancel(); cancelCoverPreload(); pixelOwner.cancel();
        transitionCover("");
      } else {
        void resolveTrackCover(currentTrackKey, mediaState.title, mediaState.artist, mediaState.source, cover);
      }
    };
    const handleMediaUpdate = (payload: any, authoritative = false, origin = "native-event", snapshot?: ArtworkSnapshotToken) => {
      if (disposed) return;
      traceMedia(origin, "received", payload);
      if (origin === "island-sync") {
        if (appSessionMode) {
          if (payload.appSessionId !== currentAppSessionId) {
            traceMedia(origin, "dropped-retired-app-session", payload);
            return;
          }
        } else if (payload.appSessionId || nativeStampedMode) {
          // A relay cannot declare its own session. It must follow the explicit
          // ready-query handshake from the current App mount.
          traceMedia(origin, "dropped-unannounced-app-session", payload);
          return;
        }
        // Preserve the pre-stamp compatibility path; stamped relays must use
        // the session/basis-scoped artwork in appOverlay, never top-level art.
        if (appSessionMode && readNativeMediaStamp(payload)) payload = withoutArtwork(payload);
      }
      const receivedAt = Date.now();
      const newTrackKey = `${payload.source || ""}|${mediaTrackKey(payload.title || "", payload.artist || "")}`;
      const incomingStamp = readNativeMediaStamp(payload);
      const stampDecision = nativeStampDecision(
        currentNativeStamp,
        nativeStampedMode,
        retiredNativeEpochs,
        incomingStamp,
        origin === "native-event" || origin === "initial-snapshot",
      );
      if (["older", "retired", "invalid"].includes(stampDecision)) {
        traceMedia(origin, `dropped-${stampDecision}-native-stamp`, payload);
        return;
      }
      if (snapshot) {
        const decision = snapshotDecision(snapshot, {events:mediaEventRevision,artwork:artworkRevision,trackGeneration:mediaTrackGeneration,trackKey:currentTrackKey}, newTrackKey === currentTrackKey);
        if (decision === "discard") { traceMedia(origin, "dropped-stale-snapshot", payload); return; }
        if (decision === "artwork-only") {
          const cover = artworkValue(payload);
          if (cover && !mediaState.albumArt && !displayCover) {
            receiveSameTrackArtwork({ ...withoutArtwork(payload), albumArt: cover });
            traceMedia(origin, "filled-missing-snapshot-artwork", payload);
          } else {
            traceMedia(origin, "ignored-stale-snapshot", payload);
          }
          return;
        }
      } else { mediaEventRevision++; }
      if ((stampDecision === "initial" || stampDecision === "newer" || stampDecision === "switch") && incomingStamp && payload.source) {
        if (Number.isSafeInteger(payload.nativeTrackGeneration) && payload.nativeTrackGeneration >= 0) {
          nativeSeekSnapshotRevision++;
          latestNativeSeekSnapshot = {
            epoch: incomingStamp.nativeEpoch,
            trackGeneration: payload.nativeTrackGeneration,
            title: String(payload.title || ""),
            artist: String(payload.artist || ""),
            positionMs: Number(payload.positionMs) || 0,
            durationMs: Number(payload.durationMs) || 0,
          };
          if (pendingSeekUntil) {
            // Position snapshots do not cancel a still-pending user seek.
            pendingSeekUntil = 0;
          }
        }
      } else if (!payload.source) {
        latestNativeSeekSnapshot = undefined;
      }
      const previousBasis = nativeBasisKey(currentNativeStamp);
      if (stampDecision === "switch") {
        if (currentNativeStamp) retiredNativeEpochs.add(currentNativeStamp.nativeEpoch);
        currentNativeStamp = incomingStamp;
        nativeStampedMode = true;
      } else if (stampDecision === "initial" || stampDecision === "newer") {
        if (incomingStamp) {
          currentNativeStamp = incomingStamp;
          nativeStampedMode = true;
        }
      }
      if (nativeBasisKey(currentNativeStamp) !== previousBasis) appliedOverlayBasis = undefined;
      if (stampDecision === "same") {
        if (newTrackKey !== currentTrackKey) {
          traceMedia(origin, "ignored-same-stamp-track-mismatch", payload);
          return;
        }
        if (origin === "island-sync") {
          applyAppOverlay(payload, newTrackKey);
        } else if (origin === "initial-snapshot") {
          const cover = artworkValue(payload);
          if (cover && !mediaState.albumArt && !displayCover) receiveSameTrackArtwork({albumArt:cover});
        } else if (appliedOverlayBasis !== nativeBasisKey(currentNativeStamp)) {
          // A duplicate native event must not replace HD art accepted for this
          // exact basis; native artwork can still fill/refresh before overlay.
          receiveSameTrackArtwork(payload);
        }
        traceMedia(origin, "same-native-stamp-enrichment", payload);
        return;
      }

      if (mediaState.neteaseFeatures?.mode.source === "cdp" && payload.neteaseFeatures?.mode.source !== "cdp") clearNeteaseModes();
      // 检查是否是空状态（播放器关闭或无媒体）
      const isEmptyState =
        !payload.title ||
        payload.title === "" ||
        payload.title === "等待播放...";

      if (isEmptyState) {
        // Metadata can be empty for one polling cycle while SMTC refreshes.
        // Keep the current track instead of resetting its progress to zero.
        if (currentTrackKey && payload.source) return;
        // 播放器退出，重置为等待状态
        if (currentTrackKey) mediaTrackGeneration++;
        seekRequestGate.invalidate();
        currentTrackKey = "";
        seekError = "";
        artworkRevision++;
        lastReportedPosition = undefined;
        pendingSeekUntil = 0;
        mvRequestId += 1;
        smtcCoverProbe?.abort(); smtcCoverProbe = undefined;
        coverLookupTrackKey = "";
        coverRequestId += 1; coverOwner.cancel(); cancelCoverPreload(); pixelOwner.cancel();
        durationRequestId += 1;
        mediaState = {
          title: PLACEHOLDER_TITLE,
          artist: PLACEHOLDER_ARTIST,
          albumArt: "",
          isPlaying: false,
          positionMs: 0,
          durationMs: 0,
          lastUpdatedTimestamp: 0,
          source: "",
          sourceDisplay: "",
        };
        systemCover = ""; resolvedTrackCover = ""; resolvedTrackKey = "";
        transitionCover("");
        isPlayingMV = false;
        mvUrl = "";
        syncMvPlayback();
      } else if (newTrackKey !== currentTrackKey) {
        seekRequestGate.invalidate();
        lastReportedPosition = Number(payload.positionMs) || 0;
        pendingSeekUntil = 0;
        currentTrackKey = newTrackKey;
        systemCover = ""; resolvedTrackCover = ""; resolvedTrackKey = "";
        transitionCover("");
        seekError = "";
        mediaTrackGeneration++;

        // 使用 SMTC 提供的图片作为基础
        const incomingCover = artworkValue(payload);
        const smtcCover = incomingCover ?? "";
        if (incomingCover !== undefined) artworkRevision++;

        mediaState = {
          ...mediaState,
          ...payload,
          albumArt: payload.artworkResolved ? "" : smtcCover,
          lastUpdatedTimestamp:
            authoritative && Number(payload.lastUpdatedTimestamp) > 0
              ? Number(payload.lastUpdatedTimestamp)
              : receivedAt,
        };

        // Stop the previous MV and show the new SMTC cover immediately. The
        // resolver validates title/artist matches before upgrading any source,
        // including browser-based players, to high-resolution artwork.
        isPlayingMV = false;
        mvRequestId += 1;
        mvUrl = "";
        syncMvPlayback();
        void resolveTrackCover(
          newTrackKey,
          payload.title || "",
          payload.artist || "",
          payload.source || "generic",
          smtcCover,
          Boolean(payload.artworkResolved),
        );
        void resolveMissingDuration(
          newTrackKey,
          payload.title || "",
          payload.artist || "",
        );

        if (isMVPlaybackEnabled && !floatingCircularAlbum) requestMVForCurrentTrack(newTrackKey, payload.title, payload.artist);
      } else {
        receiveSameTrackArtwork(payload);
        // 播放状态变化
        const previousPosition = projectedPosition(mediaState, receivedAt);
        const wasPlaying = mediaState.isPlaying;
        const isPlaying = Boolean(payload.isPlaying);
        const reportedDuration = Number(payload.durationMs) || mediaState.durationMs;

        mediaState = {
          ...mediaState,
          isPlaying,
          nativeEpoch: incomingStamp?.nativeEpoch ?? mediaState.nativeEpoch,
          nativeRevision: incomingStamp?.nativeRevision ?? mediaState.nativeRevision,
          nativeTrackGeneration: Number.isSafeInteger(payload.nativeTrackGeneration) ? payload.nativeTrackGeneration : mediaState.nativeTrackGeneration,
          ...mergeProgressAnchorFields(mediaState, payload),
          positionMs: receivedAt < pendingSeekUntil ? previousPosition : authoritative
            ? Math.max(0, Number(payload.positionMs) || 0)
            : reconcileReportedPosition(
                previousPosition,
                Number(payload.positionMs) || 0,
                reportedDuration,
                isPlaying,
                false,
                lastReportedPosition,
              ),
          durationMs: reportedDuration,
          lastUpdatedTimestamp:
            authoritative && Number(payload.lastUpdatedTimestamp) > 0
              ? Number(payload.lastUpdatedTimestamp)
              : receivedAt,
          capabilities: payload.capabilities,
          neteaseFeatures: payload.neteaseFeatures,
          shuffleActive: payload.shuffleActive,
          repeatMode: payload.repeatMode,
          sourceDisplay: payload.sourceDisplay ?? mediaState.sourceDisplay,
        };
        lastReportedPosition = Number(payload.positionMs) || 0;

        // 根据播放状态控制 MV
        if (isPlaying !== wasPlaying) syncMvPlayback();
      }
      if (origin === "island-sync" && appSessionMode && incomingStamp) {
        applyAppOverlay(payload, newTrackKey);
      }
      traceMedia(origin, "applied", payload);
    };
    unlistenIslandMediaSync = await eventManager.on(
      Events.ISLAND_MEDIA_SYNC,
      (payload) => {
        if (isTransportDiagnosticsEnabled()) recordTransportCounter("mediaSync", "received");
        handleMediaUpdate(payload, true, "island-sync");
      },
    );
    trackListener(unlistenIslandMediaSync);
    unlisten = await onMediaUpdate(handleMediaUpdate);
    trackListener(unlisten);
    const unlistenReadyQuery = await eventManager.on(Events.FLOATING_MEDIA_READY_QUERY, ({ appSessionId }: { appSessionId?: string } = {}) => {
      if (disposed || !beginAppSession(appSessionId)) return;
      if (mediaReady && pageVisible) announceMediaReady();
      else if (mediaReady) announceMediaNotReady("hidden");
    });
    trackListener(unlistenReadyQuery);
    try {
      const snapshotToken = {events:mediaEventRevision,artwork:artworkRevision,trackGeneration:mediaTrackGeneration,trackKey:currentTrackKey};
      const snapshot = await mediaApi.getMediaInfo();
      const diagnosticDelay = (window as any).__ISLE_MEDIA_SNAPSHOT_DELAY_MS__;
      if (typeof diagnosticDelay === "number" && diagnosticDelay > 0) {
        await new Promise(resolve => setTimeout(resolve, Math.min(3000, diagnosticDelay)));
      }
      handleMediaUpdate(snapshot, false, "initial-snapshot", snapshotToken);
    } catch (error) {
      console.warn("[悬浮窗] 初始媒体状态读取失败:", error);
    }
    if (!disposed) {
      mediaReady = true;
      if (pageVisible) announceMediaReady();
      else announceMediaNotReady("hidden");
    }
  }

  function announceMediaReady() {
    if (disposed) return;
    void emit(Events.FLOATING_MEDIA_READY, { instanceToken: mediaInstanceToken, appSessionId: currentAppSessionId }).catch((error) => {
      if (!disposed) console.warn("[悬浮窗] 通知主岛媒体监听就绪失败:", error);
    });
  }

  function announceMediaNotReady(reason: "hidden" | "unmounted") {
    void emit(Events.FLOATING_MEDIA_NOT_READY, {
      instanceToken: mediaInstanceToken,
      appSessionId: currentAppSessionId,
      reason,
    }).catch((error) => {
      if (!disposed) console.warn("[悬浮窗] 通知主岛媒体监听暂停失败:", error);
    });
  }

  $effect(() => {
    const shouldTickProgress = visualWorkVisible && mediaState.isPlaying && !isCompactCover;
    if (!shouldTickProgress) {
      clockNow = Date.now();
      return;
    }

    const tick = () => { clockNow = Date.now(); };
    tick();
    const interval = setInterval(tick, 250);
    return () => clearInterval(interval);
  });

  $effect(() => {
    shouldPlayMv;
    syncMvPlayback();
  });

  onDestroy(() => {
    announceMediaNotReady("unmounted");
    disposed = true;
    seekRequestGate.invalidate();
    mvPlaybackRevision += 1;
    mvPendingPlayRevision = undefined;
    mvVideo?.pause();
    smtcCoverProbe?.abort(); smtcCoverProbe = undefined;
    coverLookupTrackKey = "";
    coverOwner.cancel(); cancelCoverPreload(); pixelOwner.cancel();
    pageRequests.abort();
    coverRequestId++; mvRequestId++; durationRequestId++;

    window.removeEventListener("blur", handlePointerLeave);
    document.removeEventListener("mouseleave", handlePointerLeave);
    document.removeEventListener("visibilitychange", handleVisibilityChange);
    if (hoverLeaveTimeout) clearTimeout(hoverLeaveTimeout);

    // 清理所有事件监听器
    if (eventListeners) {
      eventListeners.forEach((unlisten: () => void) => unlisten());
      eventListeners.length = 0;
    }

    processedImageCache.clear();

    if (savePositionTimeout) clearTimeout(savePositionTimeout);
  });

  let showControls = $derived(
    isHovered && windowSize.width > 100 && windowSize.height > 100,
  );

  let eventListeners = $state<(() => void)[]>([]);

  // Svelte actions bind the renderer to the actual DOM node and destroy it
  // when the cover branch exits. No document queries or detached refs remain.
  function coverCanvas(canvas: HTMLCanvasElement, request: CanvasArtworkRequest) {
    const renderer = createCanvasArtworkRenderer(canvas, {
      process: (url, signal) => processImageBackend(url, true, signal),
      paint: (target, image) => {
        const ctx = target.getContext("2d");
        if (!ctx) return;
        const raster = artworkCanvasSize(image.width, image.height, Math.max(target.clientWidth, target.clientHeight, 1), window.devicePixelRatio || 1);
        target.width = raster.width;
        target.height = raster.height;
        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = "high";
        ctx.drawImage(image, 0, 0, target.width, target.height);
        if (!disposed && target.classList.contains("album-art-new")) {
          albumFill = artworkFillFromImage(target) ?? albumFill;
          retainCompactCover = false;
        }
      },
      onError: (error) => console.error("[Canvas] 图片加载失败:", error),
    });
    let currentRequest = request;
    let lastSize = `${canvas.clientWidth}:${canvas.clientHeight}:${window.devicePixelRatio}`;
    let resizeTimer: ReturnType<typeof setTimeout> | undefined;
    const scheduleResize = () => {
      const nextSize = `${canvas.clientWidth}:${canvas.clientHeight}:${window.devicePixelRatio}`;
      if (nextSize === lastSize || !canvas.clientWidth || !canvas.clientHeight) return;
      lastSize = nextSize;
      if (resizeTimer) clearTimeout(resizeTimer);
      // Avoid reloading the cover on every intermediate frame of a resize.
      resizeTimer = setTimeout(() => renderer.update(currentRequest), 150);
    };
    const observer = typeof ResizeObserver === "undefined" ? undefined : new ResizeObserver(scheduleResize);
    observer?.observe(canvas);
    window.addEventListener("resize", scheduleResize);
    renderer.update(request);
    return {
      update(next: CanvasArtworkRequest) {
        if (resizeTimer) { clearTimeout(resizeTimer); resizeTimer = undefined; }
        currentRequest = next;
        renderer.update(next);
      },
      destroy() { if (resizeTimer) clearTimeout(resizeTimer); window.removeEventListener("resize", scheduleResize); observer?.disconnect(); renderer.destroy(); },
    };
  }

  // 缓存处理后的图片。浮动窗可能长时间切歌，使用有上限的 LRU，避免
  // 专辑封面 base64 无限留在 WebView 内存中。
  const processedImageCache = new ArtworkResultCache();
  // Pixel-art output is only used while this effect is enabled. Drop it when
  // the user disables the effect instead of retaining up to the LRU budget.
  $effect(() => { if (!enablePixelArt) processedImageCache.clear(); });
  async function processImageBackend(imageUrl: string, enablePixelArt: boolean, signal?: AbortSignal): Promise<string> {
    if (disposed || signal?.aborted) throw new DOMException("Artwork disposed", "AbortError");
    // Only data URLs are immutable by their bytes. File paths may be rewritten.
    const key = imageUrl.startsWith("data:") ? await artworkCacheKey(imageUrl, enablePixelArt) : undefined;
    if (signal?.aborted || disposed) throw new DOMException("Artwork superseded", "AbortError");
    const cached = key ? processedImageCache.get(key) : undefined;
    if (cached) return cached;
    const result = await pixelOwner.request<string>({imagePath:imageUrl,enablePixelArt},undefined,signal);
    if (disposed || signal?.aborted) throw new DOMException("Artwork superseded", "AbortError");
    if (key) processedImageCache.set(key,result);
    return result;
  }

  async function togglePlay(e: MouseEvent) {
    e.stopPropagation();
    await mediaApi.controlMedia("play_pause");
    // 不手动更新状态，等待后端的 media-update 事件同步
  }

  const floatingSeekHandler = createSeekHandler({
    gate: seekRequestGate,
    createContext: (positionMs: number, requestId: number) => ({
      next: clampSeekPosition(positionMs, mediaState.durationMs),
      track: currentTrackKey,
      requestId,
      snapshotRevision: nativeSeekSnapshotRevision,
      epoch: mediaState.nativeEpoch ?? "",
      trackGeneration: mediaState.nativeTrackGeneration ?? 0,
      title: mediaState.title,
      artist: mediaState.artist,
    }),
    isCurrentTrack: (context) => context.track === currentTrackKey,
    isDisposed: () => disposed,
    request: (context) => mediaApi.seekMedia(context.next, {
      nativeEpoch: context.epoch,
      nativeTrackGeneration: context.trackGeneration,
      title: context.title,
      artist: context.artist,
    }),
    onBegin: (context) => {
      seekError = "";
      pendingSeekUntil = Date.now() + 2000;
      mediaState.positionMs = context.next;
      mediaState.lastUpdatedTimestamp = Date.now();
    },
    onConfirmed: (verified) => {
      pendingSeekUntil = 0;
      seekError = "";
      if (!verified) return;
      mediaState.positionMs = verified.positionMs;
      mediaState.durationMs = verified.durationMs;
      mediaState.lastUpdatedTimestamp = Date.now();
    },
    onFailed: async (_error, context) => {
      pendingSeekUntil = 0;
      // A newer native sample already reconciled the window while CDP was pending.
      // Keep it, while allowing this current request to show its failure.
      if (!seekSnapshotAllowsRollback(context.snapshotRevision, nativeSeekSnapshotRevision)) return;
      const latest = latestNativeSeekSnapshot;
      if (latest && latest.epoch === context.epoch && latest.trackGeneration === context.trackGeneration
        && latest.title === context.title && latest.artist === context.artist) {
        mediaState.positionMs = latest.positionMs;
        mediaState.durationMs = latest.durationMs || mediaState.durationMs;
      } else {
        try {
          const snapshot = await mediaApi.getMediaInfo();
          if (!seekRequestGate.isCurrent(context.requestId) || disposed || context.track !== currentTrackKey
            || !seekSnapshotAllowsRollback(context.snapshotRevision, nativeSeekSnapshotRevision)) return;
          if (snapshot.nativeEpoch === context.epoch && snapshot.nativeTrackGeneration === context.trackGeneration
            && snapshot.title === context.title && snapshot.artist === context.artist) {
            mediaState.positionMs = Number(snapshot.positionMs) || 0;
            mediaState.durationMs = Number(snapshot.durationMs) || mediaState.durationMs;
          }
        } catch (refreshError) {
          console.warn("[进度] seek 失败后刷新媒体位置失败，等待下一次原生快照", refreshError);
        }
      }
      mediaState.lastUpdatedTimestamp = Date.now();
    },
    onError: (error) => {
      console.error("[进度] 调整失败:", error);
      seekError = formatSeekError(error);
    },
  });

  async function seekTo(positionMs: number) {
    await floatingSeekHandler(positionMs);
  }

  async function toggleAlwaysOnTop(e: MouseEvent) {
    e.stopPropagation();
    const next = !isAlwaysOnTop;
    try {
      await settingsApi.setFloatingWindowAlwaysOnTop(next);
      isAlwaysOnTop = next;
    } catch (error) {
      console.error("[置顶] 应用设置失败:", error);
    }
  }

  async function toggleFloatingLock(e: MouseEvent) {
    e.stopPropagation();
    const nextLocked = !isFloatingWindowLocked;
    isFloatingWindowLocked = nextLocked;
    try {
      await settingsApi.updateSettings({ lockFloatingWindow: nextLocked });
    } catch (error) {
      isFloatingWindowLocked = !nextLocked;
      console.error("[锁定] 保存悬浮窗位置锁定状态失败:", error);
    }
  }

  async function closeWindow(e: MouseEvent) {
    e.stopPropagation();
    try {
      await windowApi.closeFloatingWindow(mediaInstanceToken);
    } catch (error) {
      console.error("[悬浮窗] 关闭失败:", error);
    }
  }

  // 用于拖拽的标题栏区域 - 排除关闭按钮和置顶按钮
  function handleDragBarMousedown(e: MouseEvent) {
    // 如果悬浮窗已锁定，禁止拖拽
    if (isFloatingWindowLocked) {
      return;
    }

    const target = e.target as HTMLElement;
    if (
      target.closest(".close-btn-topbar") ||
      target.closest(".pin-btn-topbar") ||
      target.closest(".lock-btn-topbar")
    ) {
      return; // 如果点击的是关闭按钮或置顶按钮，不拖拽
    }
    getCurrentWindow().startDragging();
  }

  function handleCornerResize(
    e: PointerEvent,
    direction: ResizeDirection,
  ) {
    if (isFloatingWindowLocked || e.button !== 0 || e.isPrimary === false) return;

    e.preventDefault();
    e.stopPropagation();
    void getCurrentWindow().startResizeDragging(direction).catch((error) => {
      console.error("[悬浮窗] 启动对角缩放失败:", error);
    });
  }
</script>

<div
  class="player"
  class:hovered={isHovered}
  class:fullscreen={isWindowFullscreen}
  class:locked={isFloatingWindowLocked}
  class:pixelated={enablePixelArt}
  class:circular-album={floatingCircularAlbum}
  class:playing={mediaState.isPlaying && visualWorkVisible}
  class:compact-cover={isCompactCover}
  class:capture-hidden={isCaptureHidden}
  onpointerenter={handlePointerEnter}
  onpointerleave={handlePointerLeave}
  role="region"
  aria-label={t("mediaPlayer")}
>
  <div class="bg-solid" style:background={albumFill}></div>

  {#if !isFloatingWindowLocked}
    <div class="edge-resize edge-resize-n" onpointerdown={(e) => handleCornerResize(e, "North")} aria-hidden="true"></div>
    <div class="edge-resize edge-resize-e" onpointerdown={(e) => handleCornerResize(e, "East")} aria-hidden="true"></div>
    <div class="edge-resize edge-resize-s" onpointerdown={(e) => handleCornerResize(e, "South")} aria-hidden="true"></div>
    <div class="edge-resize edge-resize-w" onpointerdown={(e) => handleCornerResize(e, "West")} aria-hidden="true"></div>
    <div
      class="corner-resize corner-resize-nw"
      onpointerdown={(e) => handleCornerResize(e, "NorthWest")}
      aria-hidden="true"
    ></div>
    <div
      class="corner-resize corner-resize-ne"
      onpointerdown={(e) => handleCornerResize(e, "NorthEast")}
      aria-hidden="true"
    ></div>
    <div
      class="corner-resize corner-resize-sw"
      onpointerdown={(e) => handleCornerResize(e, "SouthWest")}
      aria-hidden="true"
    ></div>
    <div
      class="corner-resize corner-resize-se"
      onpointerdown={(e) => handleCornerResize(e, "SouthEast")}
      aria-hidden="true"
    >
      <svg class="resize-grip" width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
        <path d="M4 10 10 4M8 10l2-2" />
      </svg>
    </div>
  {/if}

  {#if isCompactCover && isHovered}
    <div class="compact-controls">
      <button
        class="compact-play"
        type="button"
        onclick={togglePlay}
        aria-label={mediaState.isPlaying ? t("pause") : t("play")}
      >
        {#if mediaState.isPlaying}
          <Pause size={20} fill="currentColor" />
        {:else}
          <Play size={20} fill="currentColor" />
        {/if}
      </button>
    </div>
    <div
      class="compact-drag-zone"
      onmousedown={handleDragBarMousedown}
      role="button"
      aria-label={t("dragWindow")}
      tabindex="0"
    ></div>
  {/if}

  <!-- 可拖拽的顶部栏 - 鼠标悬停时滑下（锁定时固定显示） -->
  {#if !isFloatingWindowLocked || isFloatingWindowLocked}
    <div
      class="drag-bar"
      class:locked={isFloatingWindowLocked}
      onmousedown={handleDragBarMousedown}
      role="button"
      aria-label={t("dragWindow")}
      tabindex="0"
    >
      <button
        class="lock-btn-topbar"
        onclick={toggleFloatingLock}
        aria-label={isFloatingWindowLocked ? t("unfixPosition") : t("fixPosition")}
        aria-pressed={isFloatingWindowLocked}
        title={isFloatingWindowLocked ? t("unfixPosition") : t("fixPosition")}
      >
        {#if isFloatingWindowLocked}<Lock size={15} strokeWidth={2} />{:else}<LockOpen size={15} strokeWidth={2} />{/if}
      </button>
      <button
        class="pin-btn-topbar"
        onclick={toggleAlwaysOnTop}
        aria-label={isAlwaysOnTop ? t("unpin") : t("pin")}
        aria-pressed={isAlwaysOnTop}
        class:pinned={isAlwaysOnTop}
      >
        <Pin size={16} strokeWidth={2} />
      </button>
      <div class="drag-handle">
        <div class="drag-dots">
          <div class="drag-dot"></div>
          <div class="drag-dot"></div>
          <div class="drag-dot"></div>
        </div>
      </div>
      <button class="close-btn-topbar" onclick={closeWindow} aria-label={t("close")}>
        <X size={16} strokeWidth={2} />
      </button>
    </div>
  {/if}

  <div class="media-stage">
    <div
      class="album-wrapper"
      style:width={isCompactCover ? "100%" : `${floatingCircularAlbum ? circularAlbumArtSize : albumArtSize}px`}
      style:height={isCompactCover ? "100%" : `${floatingCircularAlbum ? circularAlbumArtSize : albumArtSize}px`}
    >
      {#if displayCover}
        {#if isCompactCover || retainCompactCover}
          <img
            class="compact-cover-image"
            class:compact-cover-fallback={retainCompactCover}
            src={displayCover}
            alt=""
            draggable="false"
          />
        {/if}
        <!-- MV 视频播放 -->
        {#if isPlayingMV && mvUrl && !floatingCircularAlbum}
          <video
            use:releaseVideo
            class="mv-player"
            src={mvUrl}
            bind:this={mvVideo}
            autoplay={shouldPlayMv}
            muted
            loop
            playsinline
            preload="auto"
            disablepictureinpicture
            poster=""
            onloadeddata={(e) => {
              const video = e.target as HTMLVideoElement;
              if (video !== mvVideo) return;
              video.currentTime = 0;
              syncMvPlayback(video);
            }}
            onloadedmetadata={() => {
              console.log("[MV] metadata loaded", mvUrl);
            }}
            onerror={(e) => {
              const video = e.currentTarget as HTMLVideoElement;
              console.error(
                "[MV] playback error",
                video.error?.code,
                video.error?.message,
                mvUrl,
              );
            }}
            ontimeupdate={(e) => keepMVInPreview(e.currentTarget)}
            onended={(e) => keepMVInPreview(e.currentTarget)}
          ></video>
        {/if}
        {#if !isCompactCover}
          <!-- 旧图（如果有） -->
          {#if previousCover}
            <canvas
              use:coverCanvas={{ url: previousCover, track: currentTrackKey, pixelated: enablePixelArt }}
              class="album-art album-art-old"
              class:slide-out={slideDirection}
              draggable="false"
            ></canvas>
          {/if}
          <!-- 新图 -->
          <canvas
            use:coverCanvas={{ url: displayCover, track: currentTrackKey, pixelated: enablePixelArt }}
            class="album-art album-art-new"
            class:slide-in={slideDirection}
            draggable="false"
          ></canvas>
        {/if}
        {#if halftoneOverlayVisible}
          <div class="halftone-overlay"></div>
        {/if}
      {:else}
        <div
          class="album-art album-art-new"
          style="display: flex; align-items: center; justify-content: center; background-color: rgba(255, 255, 255, 0.05);"
        >
          <svg
            class="w-16 h-16 text-white/10"
            fill="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
            />
          </svg>
        </div>
      {/if}
    </div>
  </div>

  <!-- 歌曲信息层 - 贴在渐变背景上 -->
  <div class="track-info-layer">
    <div class="track-title" title={mediaState.title}>
      {mediaState.title}
    </div>
    <div class="track-artist" title={mediaState.artist}>
      {mediaState.artist}
    </div>
    {#if mediaState.source === "netease"}
      {#key `${mediaState.nativeEpoch}:${mediaState.nativeTrackGeneration}:${mediaState.title}:${mediaState.artist}`}
        <div class="floating-like-control">
          <NeteaseLikeControl title={mediaState.title} artist={mediaState.artist} cacheKey={`${mediaState.nativeEpoch}:${mediaState.nativeTrackGeneration}`} available={mediaState.neteaseFeatures?.like.controllable === true} live={Boolean(mediaState.title)} compact />
        </div>
      {/key}
    {/if}
  </div>

  <div class="progress-layer">
    <div class="shared-progress">
      <MediaProgress
        position={displayedPosition}
        duration={mediaState.durationMs}
        progressKnown={mediaState.progressKnown !== false}
        estimated={mediaState.neteaseFeatures?.progress.source === "estimate"}
        seekable={Boolean(mediaState.capabilities?.seek)}
        {seekError}
        onSeek={seekTo}
      />
    </div>
  </div>

  <!-- 控制按钮遮罩层 -->
  <div class="controls-overlay" class:visible={showControls}>
    {#if mediaState.neteaseFeatures?.mode.source === "cdp"}
      <div class="floating-modes">{#key `${mediaState.nativeEpoch}:${mediaState.sourceDisplay}`}<NeteaseConnectedModeControl media={mediaState} compact/>{/key}</div>
    {:else if mediaState.capabilities?.shuffle || mediaState.capabilities?.repeat}
      <div class="floating-modes">{#key mediaState.source}<SmtcModeControls media={mediaState} compact/>{/key}</div>
    {/if}
    <div class="controls">
      <button
        class="ctrl-btn"
        onclick={(e) => {
          e.stopPropagation();
          mediaApi.controlMedia("prev");
        }}
        aria-label={t("previous")}
      >
        <SkipBack size={20} fill="currentColor" />
      </button>

      <button
        class="play-btn"
        onclick={togglePlay}
        aria-label={mediaState.isPlaying ? t("pause") : t("play")}
      >
        {#if mediaState.isPlaying}
          <Pause size={20} fill="currentColor" />
        {:else}
          <Play size={20} fill="currentColor" />
        {/if}
      </button>

      <button
        class="ctrl-btn"
        onclick={(e) => {
          e.stopPropagation();
          mediaApi.controlMedia("next");
        }}
        aria-label={t("next")}
      >
        <SkipForward size={20} fill="currentColor" />
      </button>
    </div>
  </div>
</div>

<style>
  :global(.player), :global(.player button), :global(.player input) { font-family: var(--app-font) !important; }
  :global(body, html) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: transparent;
    font-family: var(--app-font);
    text-rendering: optimizeLegibility;
    font-synthesis: none;
    -webkit-font-smoothing: antialiased;
  }

  .player {
    /* Small card corners from the reference player, not the island's pill radius. */
    --floating-radius: 8px;
    --toolbar-height: 0px;
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    border-radius: var(--floating-radius);
    /* Clip composited child layers and shadows to the transparent window edge. */
    clip-path: inset(0 round var(--floating-radius));
    border: 3px solid #000000;
    background: #000000;
    user-select: none;
    -webkit-user-select: none;
    box-sizing: border-box;
    box-shadow: none;
    isolation: isolate;
  }

  .bg-solid {
    position: absolute;
    top: var(--toolbar-height);
    right: 0;
    bottom: 64px;
    left: 0;
    z-index: 1;
    background: #000000;
    transition:
      top 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      border-radius 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      opacity 0.3s ease;

    border-radius: var(--floating-radius);
  }

  /* 可拖拽的顶部栏 - 鼠标悬停时滑下 */
  .drag-bar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 29px;
    z-index: 300; /* 最高层级，确保不被遮罩层盖住 */
    display: flex;
    align-items: center;
    justify-content: space-between; /* 两端对齐 */
    padding: 3px;
    box-sizing: border-box;
    visibility: hidden; /* 完全隐藏 */
    transform: translateY(-100%);
    transition:
      visibility 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    background: #000000;
    border: 0;
    border-radius: var(--floating-radius) var(--floating-radius) 0 0;
    box-shadow: none;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    pointer-events: auto; /* 确保可以接收鼠标事件 */
  }

  .player.hovered {
    --toolbar-height: 29px;
  }

  .player.fullscreen {
    --toolbar-height: 29px;
  }

  .player.hovered .bg-solid {
    border-radius: var(--floating-radius);
  }

  .player.hovered .drag-bar {
    visibility: visible;
    transform: translateY(0);
  }

  .player.fullscreen .drag-bar {
    visibility: visible;
    transform: translateY(0);
  }

  .drag-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 12px;
    flex: 1; /* 占据中间空间 */
  }

  .drag-dots {
    position: absolute;
    top: 50%;
    left: 50%;
    display: flex;
    gap: 3px;
    padding: 0;
    transform: translate(-50%, -50%);
    pointer-events: none;
  }

  .drag-dot {
    width: 3px;
    height: 3px;
    background: rgba(255, 255, 255, 0.6);
    border-radius: 50%;
    transition: background 0.2s ease;
  }

  .drag-bar:hover .drag-dot {
    background: rgba(255, 255, 255, 0.8);
  }

  .lock-btn-topbar {
    width: 28px;
    height: 23px;
    padding: 0;
    border: 0;
    outline: none;
    background: transparent;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.58);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s ease, transform 0.15s ease;
    flex-shrink: 0;
  }

  .lock-btn-topbar:hover { color: rgba(255, 255, 255, 0.92); transform: scale(1.08); }
  .lock-btn-topbar:active { transform: scale(0.94); }
  .lock-btn-topbar:focus-visible { outline: 2px solid rgba(255, 255, 255, 0.76); outline-offset: 1px; }
  .lock-btn-topbar[aria-pressed="true"] { color: #fff; }

  /* 顶部栏置顶按钮 */
  .pin-btn-topbar {
    width: 28px;
    height: 23px;
    padding: 0;
    border: 0;
    outline: none;
    background: transparent;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.58);
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      color 0.15s ease,
      transform 0.15s ease,
      background 0.15s ease,
      border-color 0.15s ease;
    flex-shrink: 0; /* 不被压缩 */
  }

  .pin-btn-topbar:hover {
    color: rgba(255, 255, 255, 0.92);
    background: transparent;
    transform: scale(1.08);
  }

  .pin-btn-topbar:active {
    transform: scale(0.94);
    background: transparent;
  }

  .pin-btn-topbar:focus-visible {
    outline: 2px solid rgba(255, 255, 255, 0.76);
    outline-offset: 1px;
  }

  .pin-btn-topbar.pinned {
    color: #fff;
    background: transparent;
    box-shadow: none;
  }

  .pin-btn-topbar.pinned:hover {
    color: #fff;
    background: transparent;
  }

  .pin-btn-topbar :global(svg) {
    transition: transform 0.18s ease;
  }

  .pin-btn-topbar.pinned :global(svg) {
    transform: rotate(45deg);
  }

  /* 顶部栏关闭按钮 */
  .close-btn-topbar {
    background: none;
    border: none;
    outline: none;
    width: 28px;
    height: 23px;
    padding: 0;
    cursor: pointer;
    color: #dfdfdf;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    transition:
      color 0.15s ease,
      transform 0.15s ease,
      background 0.15s ease;
    flex-shrink: 0; /* 不被压缩 */
  }

  .close-btn-topbar:hover {
    color: #fff;
    transform: scale(1.1);
    background: transparent;
  }

  .close-btn-topbar:active {
    transform: scale(0.9);
    background: transparent;
  }

  /* ==================== 专辑封面 ==================== */
  .media-stage {
    position: absolute;
    inset: 0 0 64px 0;
    z-index: 2;
    display: grid;
    place-items: center;
    overflow: hidden;
    perspective: 1200px; /* 3D 透视效果 */
  }

  .album-wrapper {
    aspect-ratio: 1 / 1;
    flex: none;
    position: relative;
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.3),
      0 2px 12px rgba(0, 0, 0, 0.15);
    border-radius: 10px;
    overflow: hidden;
    min-width: 50px;
    min-height: 50px;
    /* 3D 变换容器 */
    transform-style: preserve-3d;
    transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  /* MV 播放器 */
  .mv-player {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 10;
    border-radius: 10px;
  }

  /* 半色调网点效果样式 */
  .halftone-overlay {
    position: absolute;
    inset: 0;
    z-index: 15;
    pointer-events: none;
    background-image: radial-gradient(
      rgba(0, 0, 0, 0.45) 1.5px,
      transparent 1.5px
    );
    background-size: 4px 4px;
    border-radius: 10px;
    mix-blend-mode: overlay;
    opacity: 0.8;
  }

  .player.pixelated .halftone-overlay {
    mix-blend-mode: hard-light;
    background-image: radial-gradient(
      rgba(0, 0, 0, 0.6) 1.5px,
      transparent 1px
    );
    background-size: 3px 3px;
  }

  .album-art {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    aspect-ratio: 1 / 1;
    object-fit: cover;
    display: block;
    border-radius: 10px;
    pointer-events: none; /* 让鼠标事件穿透，不阻挡按钮点击 */
    user-select: none;
    -webkit-user-drag: none;
    /* 优化的像素化效果 */
    image-rendering: -webkit-optimize-contrast;
    image-rendering: -moz-crisp-edges;
    image-rendering: crisp-edges;
    image-rendering: pixelated;
    -ms-interpolation-mode: nearest-neighbor;
  }

  .compact-cover-image {
    display: none;
  }

  /* At the 200 x 200 minimum size the floating player becomes a cover tile. */
  .player.compact-cover .media-stage {
    inset: 0;
    padding: 0;
  }

  .player.compact-cover .album-wrapper {
    width: 100%;
    height: 100%;
    max-width: none;
    max-height: none;
    min-width: 0;
    min-height: 0;
    border-radius: 0;
    box-shadow: none;
  }

  .player.compact-cover .album-art,
  .player.compact-cover .mv-player,
  .player.compact-cover .halftone-overlay {
    border-radius: 0;
  }

  .player.compact-cover {
    border: 0;
  }

  .player.compact-cover .compact-cover-image,
  .compact-cover-image.compact-cover-fallback {
    position: absolute;
    inset: 0;
    z-index: 4;
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
    pointer-events: none;
    user-select: none;
    -webkit-user-drag: none;
  }

  .player.compact-cover .album-art {
    display: none;
  }

  .player.capture-hidden {
    visibility: hidden;
    pointer-events: none;
  }

  .player.compact-cover.pixelated .compact-cover-image {
    image-rendering: pixelated;
  }

  .player.compact-cover .bg-solid,
  .player.compact-cover .drag-bar,
  .player.compact-cover .track-info-layer,
  .player.compact-cover .progress-layer,
  .player.compact-cover .controls-overlay {
    display: none;
  }

  .player.compact-cover.fullscreen .drag-bar {
    display: flex;
  }

  .compact-controls {
    position: absolute;
    inset: 0;
    z-index: 250;
    display: grid;
    place-items: center;
    background: transparent;
    pointer-events: none;
  }

  .compact-play {
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    padding: 0;
    border: 0;
    border-radius: 12px;
    color: rgba(255, 255, 255, .9);
    background: transparent;
    cursor: pointer;
    pointer-events: auto;
    transition: color 150ms ease, background 150ms ease, transform 140ms cubic-bezier(.23,1,.32,1);
  }

  .compact-play:focus-visible { outline: 2px solid #fff; outline-offset: 2px; background: rgba(255,255,255,.1); }

  .compact-drag-zone {
    position: absolute;
    inset: 0 0 auto;
    z-index: 300;
    height: 28px;
    cursor: grab;
  }

  /* 像素模式仍保留图像效果，文字统一使用 MiSans。 */
  :global(.pixel-font) {
    font-family: var(--app-font);
    font-weight: 700;
  }

  /* 旧图在上层，新图在下层 */
  .album-art-old {
    z-index: 2;
    will-change: transform, opacity;
    backface-visibility: hidden;
  }

  .album-art-new {
    z-index: 1;
    will-change: transform, opacity;
    backface-visibility: hidden;
  }

  /* 旧图向左滑出动画 */
  .album-art-old.slide-out {
    animation: slide-out-left 0.4s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  }

  /* 新图从右滑入动画 */
  .album-art-new.slide-in {
    animation: slide-in-from-right 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  }

  /* ==================== 歌曲信息层 ==================== */
  .track-info-layer {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    height: 64px;
    z-index: 5;
    box-sizing: border-box;
    padding: 8px 8px 7px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 3px;
    text-align: left;
    pointer-events: auto;
    overflow: hidden;
    background: #000000;
    /* The player owns the outer corners; avoid a second inset curved edge. */
    border-radius: 0;
  }

  .floating-like-control {
    position: absolute;
    top: 8px;
    right: 10px;
    z-index: 2;
    width: 48px;
    height: 48px;
    display: grid;
    place-items: center;
  }

  .track-info-layer:has(.floating-like-control) .track-title,
  .track-info-layer:has(.floating-like-control) .track-artist {
    padding-right: 58px;
  }

  .player.circular-album .album-wrapper {
    border-radius: 50%;
    animation: album-spin 20s linear infinite;
    animation-play-state: paused;
  }

  .player.circular-album.playing .album-wrapper {
    animation-play-state: running;
  }

  @keyframes album-spin {
    to { transform: rotate(360deg); }
  }

  @media (prefers-reduced-motion: reduce) {
    .player.circular-album .album-wrapper {
      animation: none;
    }
  }

  .track-title {
    color: #dfdfdf; /* 调整字体颜色 */
    font-size: clamp(14px, 5vw, 20px);
    font-weight: 600;
    letter-spacing: 0.01em;
    line-height: 1.3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    user-select: none;
    font-family: var(--app-font);
    text-shadow: 0 1px 4px rgba(0, 0, 0, 0.24);
    transition: all 0.3s ease;
  }

  .track-artist {
    color: rgba(255, 255, 255, 0.7);
    font-size: clamp(10px, 3vw, 12px);
    font-weight: 500;
    letter-spacing: 0.02em;
    line-height: 1.3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    user-select: none;
    text-shadow: 0 1px 4px rgba(0, 0, 0, 0.22);
    transition: all 0.3s ease;
  }

  .player.pixelated .track-title,
  .player.pixelated .track-artist {
    font-family: var(--app-font);
    font-weight: 700;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
        /* 优化GPU加速 */
    transform: translateZ(0);
    backface-visibility: hidden;
    perspective: 1000px;
  }

  /* 像素化圆角效果 - 仅移除专辑封面和控制按钮的圆角，保留悬浮窗整体圆角 */
  .player.pixelated .album-art {
    border-radius: 0 !important;
  }

  .player.pixelated .close-btn-topbar,
  .player.pixelated .pin-btn-topbar {
    border-radius: 0 !important;
  }

  /* ==================== 进度条 ==================== */
  .progress-layer {
    position: absolute;
    top: 0;
    bottom: 64px;
    left: 0;
    right: 0;
    z-index: 251;
    display: flex;
    flex-direction: column;
    opacity: 0;
    transition: opacity 0.3s ease;
    pointer-events: none;
  }

  .player.hovered .progress-layer {
    opacity: 1;
  }

  .shared-progress {
    position: absolute;
    /* Absolute offsets start inside the player's 3px border: 12 + 3 = 15px. */
    right: 12px;
    bottom: 8px;
    left: 12px;
    pointer-events: auto;
  }

  .shared-progress :global(.progress-block.with-times) {
    grid-template-columns: var(--time-width) minmax(0, 1fr) var(--time-width);
    column-gap: 6px;
    align-items: center;
  }

  .shared-progress :global(.time) {
    font-variant-numeric: tabular-nums;
  }

  /* 控制按钮遮罩层 */
  .controls-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 64px; /* 到歌曲信息层上方结束 */
    background: linear-gradient(
      to bottom,
      rgba(0, 0, 0, 0),
      rgba(0, 0, 0, 0.6)
    );
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 250; /* 低于顶部栏的 z-index: 300 */
    pointer-events: auto;
    opacity: 0;
    visibility: hidden;
    transition:
      opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      visibility 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .floating-modes{position:absolute;bottom:12px;left:50%;transform:translateX(-50%)}

  .controls-overlay.visible {
    opacity: 1;
    visibility: visible;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  /* 无边框透明窗口的系统边缘可直接缩放；四角需要显式启动对角缩放。 */
  .corner-resize {
    position: absolute;
    z-index: 400;
    width: 8px;
    height: 8px;
    pointer-events: auto;
  }

  /* Keep edge hit areas narrow so they don't intercept nearby controls. */
  .edge-resize {
    position: absolute;
    z-index: 399;
    pointer-events: auto;
    touch-action: none;
    user-select: none;
  }

  .edge-resize-n,
  .edge-resize-s {
    right: 40px;
    left: 40px;
    height: 8px;
    cursor: ns-resize;
  }

  .edge-resize-n { top: 0; }
  .edge-resize-s { bottom: 0; }

  .edge-resize-e,
  .edge-resize-w {
    top: 40px;
    bottom: 40px;
    width: 8px;
    cursor: ew-resize;
  }

  .edge-resize-e { right: 0; }
  .edge-resize-w { left: 0; }

  .corner-resize-nw {
    top: 0;
    left: 0;
    cursor: nwse-resize;
  }

  .corner-resize-ne {
    top: 0;
    right: 0;
    cursor: nesw-resize;
  }

  .corner-resize-sw {
    bottom: 0;
    left: 0;
    cursor: nesw-resize;
  }

  .corner-resize-se {
    right: 0;
    bottom: 0;
    width: 8px;
    height: 8px;
    cursor: nwse-resize;
    touch-action: none;
    user-select: none;
  }

  .resize-grip {
    position: absolute;
    right: 1px;
    bottom: 1px;
    overflow: visible;
    fill: none;
    stroke: rgba(255, 255, 255, .66);
    stroke-width: 1.5;
    stroke-linecap: round;
    pointer-events: none;
    transition: stroke 150ms ease;
  }

  .corner-resize-se:hover .resize-grip {
    stroke: rgba(255, 255, 255, .95);
  }

  .ctrl-btn {
    flex: none;
    width: 48px;
    height: 48px;
    background: transparent;
    border: 0;
    padding: 0;
    cursor: pointer;
    color: rgba(255,255,255,.9);
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 12px;
    transition:
      color 0.15s ease,
      transform 0.15s ease,
      background 0.15s ease;
  }

  .play-btn {
    flex: none;
    width: 48px;
    height: 48px;
    border: none;
    border-radius: 12px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    color: rgba(255,255,255,.9);
    transition:
      background 150ms ease,
      color 150ms ease,
      transform 140ms cubic-bezier(.23,1,.32,1);
  }

  @media (hover: hover) and (pointer: fine) {
    .compact-play:hover,
    .ctrl-btn:hover,
    .play-btn:hover {
      color: #fff;
      background: rgba(255,255,255,.1);
      transform: scale(1.06);
    }
  }

  .compact-play:active { transform: scale(.94); }
  .ctrl-btn:active { transform: scale(.9); }
  .play-btn:active { transform: scale(.95); }

  .ctrl-btn:focus-visible,
  .play-btn:focus-visible { outline: 2px solid #fff; outline-offset: 2px; background: rgba(255,255,255,.1); }

  /* 专辑图片淡入动画 */
  @keyframes fade-enter {
    0% {
      opacity: 0;
    }
    100% {
      opacity: 1;
    }
  }

  /* 向左滑出（下一首） */
  @keyframes slide-out-left {
    0% {
      transform: translateX(0) scale(1);
      opacity: 1;
    }
    100% {
      transform: translateX(-100%) scale(0.9);
      opacity: 0;
    }
  }

  /* 从右滑入（下一首） */
  @keyframes slide-in-from-right {
    0% {
      transform: translateX(100%) scale(0.9);
      opacity: 0;
    }
    100% {
      transform: translateX(0) scale(1);
      opacity: 1;
    }
  }

  /* 向右滑出（上一首） */
  @keyframes slide-out-right {
    0% {
      transform: translateX(0) scale(1);
      opacity: 1;
    }
    100% {
      transform: translateX(100%) scale(0.9);
      opacity: 0;
    }
  }

  /* 从左滑入（上一首） */
  @keyframes slide-in-from-left {
    0% {
      transform: translateX(-100%) scale(0.9);
      opacity: 0;
    }
    100% {
      transform: translateX(0) scale(1);
      opacity: 1;
    }
  }

  /* 默认进入动画 */
  @keyframes slide-enter {
    0% {
      transform: scale(0.95);
      opacity: 0;
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }

  @keyframes pulse-glow {
    0% {
      box-shadow:
        0 4px 20px rgba(0, 0, 0, 0.35),
        0 1px 6px rgba(0, 0, 0, 0.2);
    }
    50% {
      box-shadow:
        0 8px 32px rgba(0, 0, 0, 0.5),
        0 2px 12px rgba(255, 255, 255, 0.1),
        0 0 24px rgba(255, 255, 255, 0.15);
    }
    100% {
      box-shadow:
        0 4px 20px rgba(0, 0, 0, 0.35),
        0 1px 6px rgba(0, 0, 0, 0.2);
    }
  }
</style>
