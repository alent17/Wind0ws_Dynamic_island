<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { eventManager, onMediaUpdate } from "./utils/eventManager";
  import { Events } from "./utils/eventConstants";
  import { mediaApi } from "$lib/api/media";
  import { windowApi } from "$lib/api/window";
  import { settingsApi } from "$lib/api/settings";
  import { applyAppFont } from "$lib/font";
  import { locale, setLocale, translate, type TranslationKey } from "$lib/i18n";
  import {
    activeCaptureReasons,
    EMPTY_CAPTURE_SNAPSHOT,
    type CaptureSnapshot,
  } from "$lib/captureMode";
  import MediaProgress from "$lib/MediaProgress.svelte";
  import { clampSeekPosition, mediaTrackKey, projectedPosition, reconcileReportedPosition } from "$lib/mediaClock";
  import { DEFAULT_SETTINGS, type MediaState, type AppSettings } from "$lib/api/types";
  import {
    Play,
    Pause,
    SkipBack,
    SkipForward,
    X,
    Pin,
    Minimize2,
    Lock,
    LockOpen,
  } from "lucide-svelte";

  interface WindowSize {
    width: number;
    height: number;
  }

  type ResizeDirection =
    | "NorthWest"
    | "NorthEast"
    | "SouthWest"
    | "SouthEast";

  type RGB = { r: number; g: number; b: number };
  type HSL = { h: number; s: number; l: number };

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

  let currentTrackKey = "";
  let lastReportedPosition: number | undefined;
  let pendingSeekUntil = 0;
  let displayCover = $state("");
  let previousCover = $state("");
  let isHovered = $state(false);
  let hoverLeaveTimeout: ReturnType<typeof setTimeout> | null = null;
  let captureSnapshot = $state<CaptureSnapshot>({ ...EMPTY_CAPTURE_SNAPSHOT });
  let capturePreferences = $state<AppSettings>({ ...DEFAULT_SETTINGS });
  let isCaptureHidden = $derived(
    activeCaptureReasons(captureSnapshot, capturePreferences).length > 0,
  );
  let slideDirection = $state<"left" | "right" | "">("");
  let isAnimating = $state(false); // 动画进行中标志
  // Artwork drives the album fill when enabled. The configured color remains
  // an independent fallback and is never mutated by extraction.
  let configuredFillColor = $state(DEFAULT_SETTINGS.floatingFillColor);
  let albumAccentColor = $state("rgb(40, 50, 60)");
  let albumAccentGradient = $state(createAlbumFill(40, 50, 60));
  let useAlbumColor = $state(DEFAULT_SETTINGS.floatingUseAlbumColor);
  let effectiveBackground = $derived(
    useAlbumColor ? albumAccentGradient : configuredFillColor,
  );
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
  let clockNow = $state(Date.now());
  let pageVisible = $state(true);
  let displayedPosition = $derived(projectedPosition(mediaState, clockNow));
  function handleVisibilityChange() {
    pageVisible = document.visibilityState === "visible";
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
  let circularAlbumArtSize = $derived(Math.max(50, Math.round(albumArtSize * 0.86)));
  let enablePixelArt = $state(false); // 像素化封面

  let unlisten: () => void;
  let unlistenIslandMediaSync: () => void;
  let lastIslandMediaSyncAt = 0;
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

  function preloadCover(url: string): Promise<boolean> {
    if (!url) return Promise.resolve(false);

    return new Promise((resolve) => {
      const image = pageImage();
      if (url.startsWith("http") && !url.includes("asset.localhost")) {
        image.crossOrigin = "Anonymous";
      }
      image.onload = () => resolve(true);
      image.onerror = () => resolve(false);
      image.src = url;
    });
  }

  async function resolveTrackCover(
    trackKey: string,
    title: string,
    artist: string,
    source: string,
    fallbackCover: string,
  ) {
    const requestId = ++coverRequestId;

    // Never leave the previous track's artwork visible while the HD lookup runs.
    transitionCover(fallbackCover, "left");
    if (!enableHDCover || !title) return;

    try {
      const resolved = await mediaApi.resolveHdCover(
        title,
        artist,
        source || "generic",
      );
      if (
        !resolved?.url ||
        requestId !== coverRequestId ||
        trackKey !== currentTrackKey ||
        !enableHDCover
      ) {
        return;
      }

      const resolvedUrl = /^[a-z]:[\\/]/i.test(resolved.url)
        ? convertFileSrc(resolved.url)
        : resolved.url;
      if (await preloadCover(resolvedUrl)) {
        if (requestId === coverRequestId && trackKey === currentTrackKey) {
          transitionCover(resolvedUrl, "left");
        }
      }
    } catch (error) {
      console.warn("[高清封面] 获取失败，继续使用系统封面:", error);
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

  function clamp01(value: number) {
    return Math.max(0, Math.min(1, value));
  }

  function rgbToHsl(r: number, g: number, b: number): HSL {
    r /= 255;
    g /= 255;
    b /= 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    const l = (max + min) / 2;
    const delta = max - min;
    let h = 0;
    let s = 0;

    if (delta !== 0) {
      s = delta / (1 - Math.abs(2 * l - 1));

      switch (max) {
        case r:
          h = ((g - b) / delta) % 6;
          break;
        case g:
          h = (b - r) / delta + 2;
          break;
        default:
          h = (r - g) / delta + 4;
          break;
      }

      h *= 60;
      if (h < 0) h += 360;
    }

    return { h, s, l };
  }

  function hslToRgb(h: number, s: number, l: number): RGB {
    const chroma = (1 - Math.abs(2 * l - 1)) * s;
    const x = chroma * (1 - Math.abs(((h / 60) % 2) - 1));
    const m = l - chroma / 2;
    let red = 0;
    let green = 0;
    let blue = 0;

    if (h < 60) {
      red = chroma;
      green = x;
    } else if (h < 120) {
      red = x;
      green = chroma;
    } else if (h < 180) {
      green = chroma;
      blue = x;
    } else if (h < 240) {
      green = x;
      blue = chroma;
    } else if (h < 300) {
      red = x;
      blue = chroma;
    } else {
      red = chroma;
      blue = x;
    }

    return {
      r: Math.round((red + m) * 255),
      g: Math.round((green + m) * 255),
      b: Math.round((blue + m) * 255),
    };
  }

  function createAlbumFill(r: number, g: number, b: number): string {
    const { h, s, l } = rgbToHsl(r, g, b);
    const strength = 0.07 * s;
    const top = hslToRgb(h, s, clamp01(l + strength));
    const bottom = hslToRgb(h, s, clamp01(l - strength));

    return `linear-gradient(180deg, rgb(${top.r}, ${top.g}, ${top.b}) 0%, rgb(${r}, ${g}, ${b}) 50%, rgb(${bottom.r}, ${bottom.g}, ${bottom.b}) 100%)`;
  }

  async function extractColors(imgSrc: string) {
    if (!useAlbumColor || !imgSrc) return;

    try {
      const [r, g, b] = await invoke<[number, number, number]>(
        "extract_dominant_color",
        { imagePath: imgSrc },
      );

      // Ignore an in-flight result if the user disabled album coloring while
      // the native color extraction was running.
      if (!useAlbumColor) return;

      albumAccentColor = `rgb(${r}, ${g}, ${b})`;
      albumAccentGradient = createAlbumFill(r, g, b);
    } catch (error) {
      console.error("[颜色提取] 失败:", error);
    }
  }

  // 封面切换函数（无动画）
  function transitionCover(
    newCover: string,
    direction: "left" | "right" = "left",
  ) {
    displayCover = newCover;
    if (newCover && useAlbumColor) void extractColors(newCover);
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
      if (mvUrl) return await cacheMV(mvUrl);
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
    if (video.currentTime >= 30 || video.ended) {
      video.currentTime = 0;
      if (mediaState.isPlaying) void video.play().catch(() => undefined);
    }
  }

  let disposed = false;
  const pageRequests = new AbortController();
  const pendingImages = new Set<HTMLImageElement>();
  function pageImage() {
    const image = new Image();
    pendingImages.add(image);
    const done = () => pendingImages.delete(image);
    image.addEventListener("load", done, {once:true});
    image.addEventListener("error", done, {once:true});
    return image;
  }
  function trackListener(dispose: () => void) {
    if (disposed) { dispose(); throw new DOMException("Page disposed", "AbortError"); }
    eventListeners.push(dispose);
  }
  onMount(() => {
    void initialize().catch(error => { if (!disposed) console.error(error); });
  });
  async function initialize() {
    document.addEventListener("visibilitychange", handleVisibilityChange);

    // 读取设置
    try {
      const settings = await invoke<AppSettings>("get_settings");
      capturePreferences = settings;
      configuredFillColor = settings.floatingFillColor ?? DEFAULT_SETTINGS.floatingFillColor;
      useAlbumColor = settings.floatingUseAlbumColor ?? DEFAULT_SETTINGS.floatingUseAlbumColor;
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
    eventListeners = [];

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
        enableHDCover = enabled;
        coverRequestId += 1;
        if (!currentTrackKey) return;

        if (enabled) {
          void resolveTrackCover(
            currentTrackKey,
            mediaState.title,
            mediaState.artist,
            mediaState.source,
            mediaState.albumArt,
          );
        } else {
          transitionCover(mediaState.albumArt, "left");
        }
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
        const nextUseAlbumColor = value.floatingUseAlbumColor ?? DEFAULT_SETTINGS.floatingUseAlbumColor;
        const shouldRefreshAlbumColor = nextUseAlbumColor && !useAlbumColor;
        capturePreferences = value;
        isMVPlaybackEnabled = value.enableMvPlayback ?? DEFAULT_SETTINGS.enableMvPlayback;
        setCircularAlbum(value.floatingCircularAlbum ?? DEFAULT_SETTINGS.floatingCircularAlbum);
        configuredFillColor = value.floatingFillColor ?? DEFAULT_SETTINGS.floatingFillColor;
        useAlbumColor = nextUseAlbumColor;
        if (shouldRefreshAlbumColor && displayCover) void extractColors(displayCover);
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
      },
    );
    trackListener(unlistenCaptureMode);

    const appWindow = getCurrentWindow();
    windowSize = { width: window.innerWidth, height: window.innerHeight };

    // 监听窗口大小变化
    unlistenResize = await appWindow.onResized(({ payload }) => {
      // 锁定时忽略大小变化
      if (isFloatingWindowLocked) {
        return;
      }

      // Tauri reports physical pixels here, while the 100px layout breakpoint
      // is expressed in CSS pixels. Read the viewport to stay correct on HiDPI.
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
    const handleMediaUpdate = (payload: any, authoritative = false) => {
      if (disposed) return;
      const receivedAt = Date.now();
      if (authoritative) {
        lastIslandMediaSyncAt = receivedAt;
      } else if (receivedAt - lastIslandMediaSyncAt < 2_500) {
        // The main island already normalized this system snapshot. Keep its
        // timeline authoritative instead of independently correcting it again.
        return;
      }

      const newTrackKey = `${payload.source || ""}|${mediaTrackKey(payload.title || "", payload.artist || "")}`;

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
        currentTrackKey = "";
        lastReportedPosition = undefined;
        pendingSeekUntil = 0;
        mvRequestId += 1;
        coverRequestId += 1;
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
        displayCover = "";
        isPlayingMV = false;
        mvUrl = "";
      } else if (newTrackKey !== currentTrackKey) {
        lastReportedPosition = Number(payload.positionMs) || 0;
        pendingSeekUntil = 0;
        currentTrackKey = newTrackKey;

        // 使用 SMTC 提供的图片作为基础
        const smtcCover =
          payload.albumArt || payload.thumbnail || payload.coverUrl || "";

        mediaState = {
          ...mediaState,
          ...payload,
          albumArt: smtcCover,
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
        void resolveTrackCover(
          newTrackKey,
          payload.title || "",
          payload.artist || "",
          payload.source || "generic",
          smtcCover,
        );
        void resolveMissingDuration(
          newTrackKey,
          payload.title || "",
          payload.artist || "",
        );

        if (isMVPlaybackEnabled && !floatingCircularAlbum) requestMVForCurrentTrack(newTrackKey, payload.title, payload.artist);
      } else {
        // 播放状态变化
        const previousPosition = projectedPosition(mediaState, receivedAt);
        const wasPlaying = mediaState.isPlaying;
        const isPlaying = Boolean(payload.isPlaying);
        const reportedDuration = Number(payload.durationMs) || mediaState.durationMs;

        mediaState = {
          ...mediaState,
          isPlaying,
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
        };
        lastReportedPosition = Number(payload.positionMs) || 0;

        // 根据播放状态控制 MV
        if (isPlayingMV && mvUrl) {
          const videoElement = document.querySelector(
            ".mv-player",
          ) as HTMLVideoElement;
          if (videoElement) {
            if (!isPlaying && wasPlaying) {
              // 歌曲暂停，MV 也暂停
              videoElement.pause();
            } else if (isPlaying && !wasPlaying) {
              // 歌曲从暂停恢复播放，MV 也恢复播放
              videoElement.play().catch((err) => {
                console.error("[MV] 恢复播放失败:", err);
              });
            }
          } else {
          }
        }
      }
    };
    unlistenIslandMediaSync = await eventManager.on(
      Events.ISLAND_MEDIA_SYNC,
      (payload) => handleMediaUpdate(payload, true),
    );
    trackListener(unlistenIslandMediaSync);
    unlisten = await onMediaUpdate(handleMediaUpdate);
    trackListener(unlisten);
    try {
      handleMediaUpdate(await mediaApi.getMediaInfo());
    } catch (error) {
      console.warn("[悬浮窗] 初始媒体状态读取失败:", error);
    }
  }

  $effect(() => {
    const intervalMs = !pageVisible
      ? 2_000
      : mediaState.isPlaying && !isCompactCover
        ? 250
        : 1_000;
    const interval = setInterval(() => clockNow = Date.now(), intervalMs);
    return () => clearInterval(interval);
  });

  onDestroy(() => {
    disposed = true;
    pageRequests.abort();
    for (const image of pendingImages) { image.onerror?.(new Event("error")); image.onload = null; image.onerror = null; image.src = ""; }
    pendingImages.clear();
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

    // 清理临时 Canvas
    tempCanvasCache = null;
    newCanvasRef = null;
    oldCanvasRef = null;

    if (savePositionTimeout) clearTimeout(savePositionTimeout);
  });

  let showControls = $derived(
    isHovered && windowSize.width > 100 && windowSize.height > 100,
  );

  // 缓存 Canvas 元素引用，避免重复查询
  let tempCanvasCache = $state<HTMLCanvasElement | null>(null);
  let newCanvasRef = $state<HTMLCanvasElement | null>(null);
  let oldCanvasRef = $state<HTMLCanvasElement | null>(null);

  // 事件监听器管理器
  let eventListeners = $state<(() => void)[]>([]);

  // 设置 Canvas 元素引用 - 监听 displayCover 变化确保 Canvas 元素已创建
  $effect(() => {
    if (displayCover) {
      const frame = requestAnimationFrame(() => {
        if (disposed) return;
        const newCanvas = document.querySelector(
          ".album-art-new",
        ) as HTMLCanvasElement;
        const oldCanvas = document.querySelector(
          ".album-art-old",
        ) as HTMLCanvasElement;

        if (newCanvas && !newCanvasRef) {
          newCanvasRef = newCanvas;
        }
        if (oldCanvas && !oldCanvasRef) {
          oldCanvasRef = oldCanvas;
        }
      });
      return () => cancelAnimationFrame(frame);
    }
  });

  // 监听 displayCover 和 enablePixelArt 变化，渲染到 Canvas
  $effect(() => {
    if (displayCover) {
      const renderFunction = enablePixelArt
        ? renderImageToCanvas
        : renderImageToCanvasNormal;

      const newCanvas =
        newCanvasRef ||
        (document.querySelector(".album-art-new") as HTMLCanvasElement);

      if (newCanvas) {
        if (!newCanvasRef) newCanvasRef = newCanvas;
        renderFunction(newCanvas, displayCover);
      }

      if (previousCover) {
        const oldCanvas =
          oldCanvasRef ||
          (document.querySelector(".album-art-old") as HTMLCanvasElement);

        if (oldCanvas) {
          if (!oldCanvasRef) oldCanvasRef = oldCanvas;
          renderFunction(oldCanvas, previousCover);
        }
      }
    }
  });

  // 缓存处理后的图片。浮动窗可能长时间切歌，使用有上限的 LRU，避免
  // 专辑封面 base64 无限留在 WebView 内存中。
  const MAX_PROCESSED_IMAGES = 12;
  const processedImageCache = new Map<string, string>();
  const processingPromises = new Map<string, Promise<string>>();

  // 使用后端 API 处理图片（支持像素化）
  async function processImageBackend(
    imageUrl: string,
    enablePixelArt: boolean,
  ): Promise<string> {
    const cacheKey = `${enablePixelArt ? "pixel" : "normal"}:${imageUrl}`;
    const cached = processedImageCache.get(cacheKey);
    if (cached) {
      // Map insertion order is the access order used by this small LRU.
      processedImageCache.delete(cacheKey);
      processedImageCache.set(cacheKey, cached);
      return cached;
    }
    const pending = processingPromises.get(cacheKey);
    if (pending) return pending;

    const processing = (async () => {
      try {
        const processedBase64 = await invoke<string>("process_image", {
          imagePath: imageUrl,
          enablePixelArt: enablePixelArt,
        });
        processedImageCache.set(cacheKey, processedBase64);
        while (processedImageCache.size > MAX_PROCESSED_IMAGES) {
          const oldestKey = processedImageCache.keys().next().value;
          if (oldestKey === undefined) break;
          processedImageCache.delete(oldestKey);
        }
        return processedBase64;
      } catch (error) {
        console.error("[图片处理] 后端处理失败:", error);
        return imageUrl;
      } finally {
        processingPromises.delete(cacheKey);
      }
    })();
    processingPromises.set(cacheKey, processing);
    return processing;
  }

  // 渲染图片到 Canvas（简化版，直接使用后端处理后的 base64）
  async function renderImageToCanvas(
    canvas: HTMLCanvasElement,
    imageUrl: string,
  ) {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    try {
      // 使用后端 API 处理图片
      const processedUrl = await processImageBackend(imageUrl, enablePixelArt);
      if (disposed) return;

      // 加载处理后的图片
      const img = pageImage();
      if (
        processedUrl.startsWith("http") &&
        !processedUrl.includes("asset.localhost")
      ) {
        img.crossOrigin = "Anonymous";
      }
      img.onload = () => {
        // 设置 Canvas 尺寸
        canvas.width = img.width;
        canvas.height = img.height;

        // 绘制图片
        ctx.drawImage(img, 0, 0);
      };
      img.onerror = () => {
        console.error("[Canvas] 图片加载失败，使用原图");
        renderImageToCanvasNormal(canvas, imageUrl);
      };
      img.src = processedUrl;
    } catch (error) {
      console.error("[图片处理] 处理错误:", error);
      renderImageToCanvasNormal(canvas, imageUrl);
    }
  }

  // 智能图片质量分析函数
  function analyzeImageQuality(img: HTMLImageElement): number {
    // 基于图片尺寸、清晰度和内容复杂度评估质量
    const width = img.width;
    const height = img.height;

    // 基础质量评分（0-1）
    let qualityScore = Math.min(1, width / 500); // 基于宽度评分

    // 如果图片太小，降低评分
    if (width < 100 || height < 100) {
      qualityScore *= 0.5;
    }

    // 如果图片太大但可能是低质量放大，适当调整
    if (width > 800 && qualityScore > 0.8) {
      qualityScore = 0.8 + (qualityScore - 0.8) * 0.5;
    }

    return Math.max(0.1, Math.min(1, qualityScore));
  }

  // 计算最优像素大小
  function calculateOptimalPixelSize(
    img: HTMLImageElement,
    qualityScore: number,
  ): number {
    // 统一使用固定的6px像素大小
    return 12;
  }

  // 高级像素化渲染（使用调色板量化 + Floyd-Steinberg 抖动）
  function renderCachedImage(
    canvas: HTMLCanvasElement,
    img: HTMLImageElement,
    pixelated: boolean,
  ) {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // 设置 Canvas 尺寸与图片一致
    canvas.width = img.width;
    canvas.height = img.height;

    if (pixelated) {
      // 智能像素化算法：根据图片质量和内容决定像素化程度
      const imgQuality = analyzeImageQuality(img);
      const pixelSize = calculateOptimalPixelSize(img, imgQuality);

      const scaledWidth = Math.ceil(img.width / pixelSize);
      const scaledHeight = Math.ceil(img.height / pixelSize);

      // 重用或创建临时 Canvas
      if (
        !tempCanvasCache ||
        tempCanvasCache.width !== scaledWidth ||
        tempCanvasCache.height !== scaledHeight
      ) {
        tempCanvasCache = document.createElement("canvas");
        tempCanvasCache.width = scaledWidth;
        tempCanvasCache.height = scaledHeight;
      }

      const tempCtx = tempCanvasCache.getContext("2d");
      if (!tempCtx) return;

      // 关闭图像平滑处理
      tempCtx.imageSmoothingEnabled = false;
      tempCtx.imageSmoothingQuality = "low";

      // 缩小图片
      tempCtx.drawImage(img, 0, 0, scaledWidth, scaledHeight);

      // 尝试获取像素数据（可能会因为跨域而失败）
      let imageData: ImageData;
      try {
        imageData = tempCtx.getImageData(0, 0, scaledWidth, scaledHeight);
      } catch (e) {
        // 跨域污染，使用简单像素化方案
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(
          tempCanvasCache,
          0,
          0,
          scaledWidth,
          scaledHeight,
          0,
          0,
          canvas.width,
          canvas.height,
        );
        return;
      }

      const data = imageData.data;

      // 提取全局调色板（使用 NeuQuant 简化版）
      const palette = extractOptimalPalette(data, 32); // 32 种颜色

      // 应用调色板量化 + Floyd-Steinberg 抖动
      applyPaletteWithDithering(data, scaledWidth, scaledHeight, palette);

      // 将处理后的数据放回临时 Canvas
      tempCtx.putImageData(imageData, 0, 0);

      // 清除主 Canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.imageSmoothingEnabled = false;

      // 将处理好的像素画放大绘制回主 Canvas
      ctx.drawImage(
        tempCanvasCache,
        0,
        0,
        scaledWidth,
        scaledHeight,
        0,
        0,
        canvas.width,
        canvas.height,
      );
    } else {
      // 正常渲染高清图
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(img, 0, 0);
    }
  }

  // 提取最优全局调色板（简化版 NeuQuant 算法）
  function extractOptimalPalette(
    data: Uint8ClampedArray,
    numColors: number,
  ): Uint8ClampedArray {
    // 统计颜色出现频率
    const colorCount = new Map<string, number>();
    for (let i = 0; i < data.length; i += 4) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const a = data[i + 3];
      if (a < 128) continue; // 跳过透明像素

      // 量化颜色以减少键数量
      const qr = Math.round(r / 8) * 8;
      const qg = Math.round(g / 8) * 8;
      const qb = Math.round(b / 8) * 8;
      const key = `${qr},${qg},${qb}`;

      colorCount.set(key, (colorCount.get(key) || 0) + 1);
    }

    // 按频率排序
    const sortedColors = Array.from(colorCount.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, numColors * 2); // 取前 2 倍数量用于后续优化

    // 使用简单的 k-means 聚类思想优化调色板
    const palette = new Uint8ClampedArray(numColors * 4);
    for (let i = 0; i < numColors; i++) {
      if (i < sortedColors.length) {
        const [r, g, b] = sortedColors[i][0].split(",").map(Number);
        palette[i * 4] = r;
        palette[i * 4 + 1] = g;
        palette[i * 4 + 2] = b;
        palette[i * 4 + 3] = 255;
      } else {
        // 填充剩余颜色
        palette[i * 4] = 0;
        palette[i * 4 + 1] = 0;
        palette[i * 4 + 2] = 0;
        palette[i * 4 + 3] = 255;
      }
    }

    return palette;
  }

  // 应用调色板量化 + Floyd-Steinberg 误差扩散抖动
  function applyPaletteWithDithering(
    data: Uint8ClampedArray,
    width: number,
    height: number,
    palette: Uint8ClampedArray,
  ) {
    // 创建误差缓冲区
    const errors = new Float32Array(data.length);

    // 查找最接近的调色板颜色
    function findClosestColor(
      r: number,
      g: number,
      b: number,
    ): [number, number, number] {
      let minDist = Infinity;
      let closestIdx = 0;

      for (let i = 0; i < palette.length / 4; i++) {
        const pr = palette[i * 4];
        const pg = palette[i * 4 + 1];
        const pb = palette[i * 4 + 2];

        // 使用加权欧几里得距离（考虑人眼对绿色更敏感）
        const dist = 2 * (r - pr) ** 2 + 4 * (g - pg) ** 2 + 3 * (b - pb) ** 2;

        if (dist < minDist) {
          minDist = dist;
          closestIdx = i;
        }
      }

      return [
        palette[closestIdx * 4],
        palette[closestIdx * 4 + 1],
        palette[closestIdx * 4 + 2],
      ];
    }

    // 逐像素处理
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const i = (y * width + x) * 4;

        // 加上累积的误差
        const r = Math.max(0, Math.min(255, data[i] + errors[i]));
        const g = Math.max(0, Math.min(255, data[i + 1] + errors[i + 1]));
        const b = Math.max(0, Math.min(255, data[i + 2] + errors[i + 2]));

        // 找到最接近的调色板颜色
        const [qr, qg, qb] = findClosestColor(r, g, b);

        // 更新像素
        data[i] = qr;
        data[i + 1] = qg;
        data[i + 2] = qb;
        // alpha 通道保持不变

        // 计算量化误差
        const errR = r - qr;
        const errG = g - qg;
        const errB = b - qb;

        // Floyd-Steinberg 误差扩散
        // 右：7/16
        if (x + 1 < width) {
          const ni = (y * width + (x + 1)) * 4;
          errors[ni] += errR * (7 / 16);
          errors[ni + 1] += errG * (7 / 16);
          errors[ni + 2] += errB * (7 / 16);
        }

        // 左下：3/16
        if (x > 0 && y + 1 < height) {
          const ni = ((y + 1) * width + (x - 1)) * 4;
          errors[ni] += errR * (3 / 16);
          errors[ni + 1] += errG * (3 / 16);
          errors[ni + 2] += errB * (3 / 16);
        }

        // 正下：5/16
        if (y + 1 < height) {
          const ni = ((y + 1) * width + x) * 4;
          errors[ni] += errR * (5 / 16);
          errors[ni + 1] += errG * (5 / 16);
          errors[ni + 2] += errB * (5 / 16);
        }

        // 右下：1/16
        if (x + 1 < width && y + 1 < height) {
          const ni = ((y + 1) * width + (x + 1)) * 4;
          errors[ni] += errR * (1 / 16);
          errors[ni + 1] += errG * (1 / 16);
          errors[ni + 2] += errB * (1 / 16);
        }
      }
    }
  }

  // 渲染图片到 Canvas（正常高清，不像素化）
  function renderImageToCanvasNormal(
    canvas: HTMLCanvasElement,
    imageUrl: string,
  ) {
    if (disposed) return;
    const img = pageImage();
    if (imageUrl.startsWith("http") && !imageUrl.includes("asset.localhost")) {
      img.crossOrigin = "Anonymous";
    }

    img.onload = () => {
      renderCachedImage(canvas, img, false);
    };

    img.onerror = () => {
      console.error("[Canvas] 图片加载失败:", imageUrl);
    };

    img.src = imageUrl;
  }

  async function togglePlay(e: MouseEvent) {
    e.stopPropagation();
    await mediaApi.controlMedia("play_pause");
    // 不手动更新状态，等待后端的 media-update 事件同步
  }

  async function seekTo(positionMs: number) {
    const next = clampSeekPosition(positionMs, mediaState.durationMs);
    const previous = projectedPosition(mediaState);
    const seekTrack = currentTrackKey;
    pendingSeekUntil = Date.now() + 2000;
    mediaState.positionMs = next;
    mediaState.lastUpdatedTimestamp = Date.now();
    await mediaApi.seekMedia(next).catch((error) => {
      if (seekTrack === currentTrackKey) {
        pendingSeekUntil = 0;
        mediaState.positionMs = previous;
        mediaState.lastUpdatedTimestamp = Date.now();
      }
      console.error("[进度] 调整失败:", error);
    });
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
      await windowApi.closeFloatingWindow();
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
    e: MouseEvent,
    direction: ResizeDirection,
  ) {
    if (isFloatingWindowLocked || e.button !== 0) return;

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
  class:locked={isFloatingWindowLocked}
  class:pixelated={enablePixelArt}
  class:circular-album={floatingCircularAlbum}
  class:playing={mediaState.isPlaying}
  class:compact-cover={isCompactCover}
  class:capture-hidden={isCaptureHidden}
  onpointerenter={handlePointerEnter}
  onpointerleave={handlePointerLeave}
  role="region"
  aria-label={t("mediaPlayer")}
  style={`--floating-background:${effectiveBackground}`}
>
  <div class="bg-solid"></div>

  {#if !isFloatingWindowLocked}
    <div
      class="corner-resize corner-resize-nw"
      onmousedown={(e) => handleCornerResize(e, "NorthWest")}
      aria-hidden="true"
    ></div>
    <div
      class="corner-resize corner-resize-ne"
      onmousedown={(e) => handleCornerResize(e, "NorthEast")}
      aria-hidden="true"
    ></div>
    <div
      class="corner-resize corner-resize-sw"
      onmousedown={(e) => handleCornerResize(e, "SouthWest")}
      aria-hidden="true"
    ></div>
    <div
      class="corner-resize corner-resize-se"
      onmousedown={(e) => handleCornerResize(e, "SouthEast")}
      aria-hidden="true"
    ></div>
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
        <img
          class="compact-cover-image"
          src={displayCover}
          alt=""
          draggable="false"
        />
        <!-- MV 视频播放 -->
        {#if isPlayingMV && mvUrl && !floatingCircularAlbum}
          <video
            class="mv-player"
            src={mvUrl}
            autoplay={mediaState.isPlaying}
            muted
            loop
            playsinline
            preload="auto"
            disablepictureinpicture
            poster=""
            onloadeddata={(e) => {
              const video = e.target as HTMLVideoElement;
              video.currentTime = 0;
              if (mediaState.isPlaying) video.play().catch(console.error);
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
        <!-- 旧图（如果有） -->
        {#if previousCover}
          <canvas
            class="album-art album-art-old"
            class:slide-out={slideDirection}
            draggable="false"
          ></canvas>
        {/if}
        <!-- 新图 -->
        <canvas
          class="album-art album-art-new"
          class:slide-in={slideDirection}
          draggable="false"
        ></canvas>
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
  </div>

  <div class="progress-layer">
    <div class="shared-progress">
      <MediaProgress
        position={displayedPosition}
        duration={mediaState.durationMs}
        seekable={Boolean(mediaState.capabilities?.seek)}
        onSeek={seekTo}
      />
    </div>
  </div>

  <!-- 控制按钮遮罩层 -->
  <div class="controls-overlay" class:visible={showControls}>
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
    border: 3px solid #121212;
    background: #121212;
    user-select: none;
    -webkit-user-select: none;
    box-sizing: border-box;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.26);
    isolation: isolate;
    transition: box-shadow 220ms cubic-bezier(.2,.8,.2,1);
  }

  .bg-solid {
    position: absolute;
    top: var(--toolbar-height);
    right: 0;
    bottom: 64px;
    left: 0;
    z-index: 1;
    background: var(--floating-background);
    transition:
      top 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      border-radius 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      background 0.3s cubic-bezier(0.4, 0, 0.2, 1),
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
    background: #121212;
    border: 0;
    border-radius: var(--floating-radius) var(--floating-radius) 0 0;
    box-shadow: none;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    pointer-events: auto; /* 确保可以接收鼠标事件 */
  }

  .player.hovered {
    --toolbar-height: 29px;
    box-shadow: 0 18px 52px rgba(0, 0, 0, 0.46), 0 5px 18px rgba(0, 0, 0, 0.28);
  }

  .player.hovered .bg-solid {
    border-radius: var(--floating-radius);
  }

  .player.hovered .drag-bar {
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

  .player.compact-cover .compact-cover-image {
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
    background: #121212;
    border-radius: 0 0 var(--floating-radius) var(--floating-radius);
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
    .player.circular-album.playing .album-wrapper {
      animation-duration: 60s;
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
    right: 16px;
    bottom: 8px;
    left: 16px;
    pointer-events: auto;
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
    width: 14px;
    height: 14px;
    pointer-events: auto;
  }

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
    cursor: nwse-resize;
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
