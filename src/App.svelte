<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { spring } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { eventManager, onMediaUpdate } from "./utils/eventManager";
  import { Events } from "./utils/eventConstants";
  import { mediaApi } from "$lib/api/media";
  import { windowApi } from "$lib/api/window";
  import { settingsApi } from "$lib/api/settings";
  import Spectrum from "$lib/Spectrum.svelte";
  import type { AppSettings } from "$lib/api/types";
  import { DEFAULT_SETTINGS } from "$lib/api/types";
  import {
    getCurrentWindow,
    PhysicalSize,
    PhysicalPosition,
    currentMonitor,
    availableMonitors,
  } from "@tauri-apps/api/window";
  import {
    Music,
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Heart,
    Monitor,
    GalleryHorizontalEnd,
  } from "lucide-svelte";

  const platformIcons = {
    netease: "/src/assets/icons/netease.svg",
    spotify: "/src/assets/icons/spotify.svg",
    bilibili: "/src/assets/icons/bilibili.svg",
    qqmusic: "/src/assets/icons/qqmusic.svg",
    apple: "/src/assets/icons/apple_music.svg",
    generic: "/src/assets/icons/default_music.svg",
  };

  const playerNames = {
    netease: "网易云音乐",
    spotify: "Spotify",
    bilibili: "Bilibili",
    qqmusic: "QQ 音乐",
    apple: "Apple Music",
    generic: "多媒体",
  };

  const playerColors = {
    netease: "#ff2d55",
    spotify: "#1db954",
    bilibili: "#fb7299",
    qqmusic: "#31c27c",
    apple: "#fa243c",
    generic: "#ffffff",
  };

  const isDev = import.meta.env?.DEV ?? false;

  const logger = {
    log: (...args: any[]) => isDev && console.log("[App]", ...args),
    error: (...args: any[]) => console.error("[App]", ...args),
    warn: (...args: any[]) => console.warn("[App]", ...args),
    debug: (...args: any[]) => isDev && console.debug("[App]", ...args),
  };

  const throttle = (fn: Function, delay: number) => {
    let lastCall = 0;
    return (...args: any[]) => {
      const now = Date.now();
      if (now - lastCall >= delay) {
        lastCall = now;
        fn(...args);
      }
    };
  };

  const debounce = (fn: Function, delay: number) => {
    let timeoutId: ReturnType<typeof setTimeout>;
    return (...args: any[]) => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(() => fn(...args), delay);
    };
  };

  const playerConfigs = {
    netease: {
      name: "网易云音乐",
      color: "#ff2d55",
      icon: "/src/assets/icons/netease.svg",
      useProgressBar: false,
    },
    spotify: {
      name: "Spotify",
      color: "#1db954",
      icon: "/src/assets/icons/spotify.svg",
      useProgressBar: true,
    },
    bilibili: {
      name: "Bilibili",
      color: "#fb7299",
      icon: "/src/assets/icons/bilibili.svg",
      useProgressBar: true,
    },
    qqmusic: {
      name: "QQ 音乐",
      color: "#31c27c",
      icon: "/src/assets/icons/qqmusic.svg",
      useProgressBar: true,
    },
    apple: {
      name: "Apple Music",
      color: "#fa243c",
      icon: "/src/assets/icons/apple_music.svg",
      useProgressBar: true,
    },
    generic: {
      name: "正在播放",
      color: "#ffffff",
      icon: "/src/assets/icons/default_music.svg",
      useProgressBar: true,
    },
  };

  // ========== 状态管理 ==========
  let expanded = $state(false);
  let hovering = $state(false);
  let isAnimating = $state(false);
  let accentColor = $state<string>("#fe2c55");
  let secondaryColor = $state<string>("#fe2c55");
  let artworkUrl = $state<string>("");
  let rawCoverUrl = "";
  let flipKey = $state(0);
  let trackTitle = $state<string>("");
  let artistName = $state<string>("");
  let isPlaying = $state<boolean>(false);
  let lastSongKey: string | null = null;
  let currentTheme = $state<string>("original");

  let spectrumTopColor = $state<string>("#ffffff");
  let spectrumBottomColor = $state<string>("#888888");

  let currentTimeMs = $state<number>(0);
  let durationMs = $state<number>(0);
  let currentSource = $state<string>("generic");
  let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;

  let maxBackendPosition = 0;

  let showTimeDisplay = $state(false);
  let currentTime = $state("");
  let pausedStartTime = $state<number>(0);

  // 优化：缓存时间格式化结果，减少字符串操作
  function updateTimeDisplay() {
    const now = new Date();
    const hours = now.getHours();
    const minutes = now.getMinutes();
    // 使用模板字符串比 padStart 更高效
    currentTime = `${hours < 10 ? "0" : ""}${hours}:${minutes < 10 ? "0" : ""}${minutes}`;
  }

  $effect(() => {
    if (!isPlaying) {
      pausedStartTime = Date.now();
      showTimeDisplay = false;
    } else {
      showTimeDisplay = false;
    }
  });

  onMount(() => {
    updateTimeDisplay();
    const checkInterval = setInterval(() => {
      updateTimeDisplay();
      if (!isPlaying && pausedStartTime > 0) {
        const elapsed = Date.now() - pausedStartTime;
        if (elapsed >= 2 * 60 * 1000 && !showTimeDisplay) {
          showTimeDisplay = true;
        }
      }
    }, 1000);

    return () => clearInterval(checkInterval);
  });

  const playerApps: Record<string, string> = {
    netease: "NeteaseCloudMusic",
    spotify: "Spotify",
    bilibili: "Bilibili",
    qqmusic: "QQMusic",
    apple: "AppleMusic",
    generic: "",
  };

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

  // ===== 主题样式辅助函数 =====
  function getThemeBackground(theme: string): string {
    // 使用 CSS 变量，让 CSS 来控制主题颜色
    return "var(--island-bg)";
  }

  function getThemeBackgroundSize(theme: string): string {
    return "100% 100%";
  }

  function getThemeBackgroundPosition(theme: string): string {
    return "0% 0%";
  }

  function getThemeBackdropFilter(theme: string): string {
    // 所有主题都不使用毛玻璃效果
    return "none";
  }

  function getThemeBorder(theme: string): string {
    // 使用 CSS 变量
    return "1px solid var(--island-border)";
  }

  function getDynamicBorderRadius(currentHeight: number): string {
    const minHeight = 28;
    const maxHeight = 160;
    const minRadius = 24;
    const maxRadius = appSettings.expandedCornerRadius || 45;

    const clampedHeight = Math.max(
      minHeight,
      Math.min(maxHeight, currentHeight),
    );

    const progress = (clampedHeight - minHeight) / (maxHeight - minHeight);
    const radius = minRadius + (maxRadius - minRadius) * progress;

    return `${radius}px`;
  }

  function getThemeBoxShadow(
    theme: string,
    isHidden: boolean,
    expanded: boolean,
  ): string {
    // 所有状态都没有阴影
    return "none";
  }

  // ===== 新增：应用设置 =====
  let appSettings = $state<AppSettings>({
    ...DEFAULT_SETTINGS,
  });

  // ========== 性能检测和自适应系统 ==========
  type PerformanceLevel = "high" | "medium" | "low";
  let performanceLevel = $state<PerformanceLevel>("high");
  let currentFps = $state(60);
  let fpsHistory: number[] = [];
  let performanceCheckInterval: number | null = null;
  let displayRefreshRate = $state(60);
  let highFrameRateMode = $state(false);

  async function detectDisplayRefreshRate(): Promise<number> {
    return new Promise((resolve) => {
      const frames: number[] = [];
      let lastTime = performance.now();
      let frameCount = 0;

      function measureFrame(currentTime: number) {
        frameCount++;
        frames.push(currentTime);

        if (frameCount < 60) {
          requestAnimationFrame(measureFrame);
        } else {
          const intervals = [];
          for (let i = 1; i < frames.length; i++) {
            intervals.push(frames[i] - frames[i - 1]);
          }

          const avgInterval =
            intervals.reduce((a, b) => a + b, 0) / intervals.length;
          const refreshRate = Math.round(1000 / avgInterval);

          console.log(`[性能] 显示器刷新率: ${refreshRate}Hz`);
          resolve(refreshRate);
        }
      }

      requestAnimationFrame(measureFrame);
    });
  }

  function detectPerformanceLevel(): PerformanceLevel {
    const cores = navigator.hardwareConcurrency || 4;
    const memory = (navigator as any).deviceMemory || 8;
    const isMobile =
      /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
        navigator.userAgent,
      );
    const hasHardwareAcceleration = checkHardwareAcceleration();

    let score = 0;
    if (cores >= 8) score += 3;
    else if (cores >= 4) score += 2;
    else score += 1;

    if (memory >= 8) score += 3;
    else if (memory >= 4) score += 2;
    else score += 1;

    if (hasHardwareAcceleration) score += 2;
    else score += 0;

    if (isMobile) score -= 2;

    if (score >= 7) return "high";
    else if (score >= 4) return "medium";
    else return "low";
  }

  function checkHardwareAcceleration(): boolean {
    try {
      const canvas = document.createElement("canvas");
      const gl =
        canvas.getContext("webgl") || canvas.getContext("experimental-webgl");
      if (gl && gl instanceof WebGLRenderingContext) {
        const debugInfo = gl.getExtension("WEBGL_debug_renderer_info");
        if (debugInfo) {
          const renderer = gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL);
          return (
            !renderer.toLowerCase().includes("swiftshader") &&
            !renderer.toLowerCase().includes("llvmpipe")
          );
        }
      }
      return true;
    } catch (e) {
      return false;
    }
  }

  function getOptimizedSpringParams(
    level: PerformanceLevel,
    refreshRate: number = 60,
  ) {
    const frameMultiplier = refreshRate >= 120 ? 1.2 : 1.0;

    switch (level) {
      case "high":
        if (refreshRate >= 120) {
          return {
            stiffness: 0.15,
            damping: 0.7,
            precision: 0.1,
          };
        } else {
          return {
            stiffness: 0.18,
            damping: 0.7,
            precision: 0.1,
          };
        }
      case "medium":
        return {
          stiffness: 0.2,
          damping: 0.75,
          precision: 0.1,
        };
      case "low":
        return {
          stiffness: 0.25,
          damping: 0.85,
          precision: 0.1,
        };
    }
  }

  function startFpsMonitoring() {
    let lastTime = performance.now();
    let frames = 0;

    function measureFps() {
      frames++;
      const currentTime = performance.now();

      if (currentTime - lastTime >= 1000) {
        currentFps = Math.round((frames * 1000) / (currentTime - lastTime));

        fpsHistory.push(currentFps);
        if (fpsHistory.length > 10) {
          fpsHistory.shift();
        }

        adjustPerformanceBasedOnFps();

        frames = 0;
        lastTime = currentTime;
      }

      requestAnimationFrame(measureFps);
    }

    requestAnimationFrame(measureFps);
  }

  function adjustPerformanceBasedOnFps() {
    if (fpsHistory.length < 5) return;

    const avgFps = fpsHistory.reduce((a, b) => a + b, 0) / fpsHistory.length;

    if (avgFps < 30 && performanceLevel !== "low") {
      console.log("[性能] 帧率过低，降低性能等级");
      performanceLevel = "low";
      updateSpringParams();
    } else if (avgFps < 45 && avgFps >= 30 && performanceLevel === "high") {
      console.log("[性能] 帧率中等，调整为中等性能");
      performanceLevel = "medium";
      updateSpringParams();
    } else if (avgFps >= 55 && performanceLevel !== "high") {
      console.log("[性能] 帧率良好，提升性能等级");
      performanceLevel = "high";
      updateSpringParams();
    }
  }

  function updateSpringParams() {
    const params = getOptimizedSpringParams(
      performanceLevel,
      displayRefreshRate,
    );

    const currentWidth = $widthSpring;
    const currentHeight = $heightSpring;
    const currentOpacity = $contentOpacity;

    Object.assign(widthSpring, {
      stiffness: params.stiffness,
      damping: params.damping,
      precision: params.precision,
    });

    Object.assign(heightSpring, {
      stiffness: params.stiffness,
      damping: params.damping,
      precision: params.precision,
    });

    Object.assign(contentOpacity, {
      stiffness: params.stiffness * 1.2,
      damping: params.damping * 1.2,
      precision: params.precision,
    });

    highFrameRateMode = displayRefreshRate >= 120;

    console.log(
      `[性能] 已更新 Spring 参数: ${performanceLevel}, 刷新率: ${displayRefreshRate}Hz, 高帧率模式: ${highFrameRateMode}`,
      params,
    );
  }

  let isFullscreenApp = $state(false);
  let isMouseAtTop = $state(false);
  let isHidden = $state(false);
  let autoHideEnabled = $state(true);

  let showMonitorMenu = $state(false);
  let monitors: Array<{
    name: string;
    index: number;
    position: { x: number; y: number };
    size: { width: number; height: number };
  }> = $state([]);
  let currentMonitorIndex = $state(0);

  let isFloatingWindowOpen = $state(false);

  let fps = $state(0);
  let frameCount = 0;
  let lastFpsTime = 0;
  let debugRafId: number | null = null;

  const currentIcon = $derived(
    platformIcons[currentSource as keyof typeof platformIcons] ||
      platformIcons.generic,
  );

  const currentColor = $derived(
    playerColors[currentSource as keyof typeof playerColors] ||
      playerColors.generic,
  );

  const currentConfig = $derived(
    playerConfigs[currentSource as keyof typeof playerConfigs] ||
      playerConfigs.generic,
  );

  let isLive = $derived(durationMs === 0);

  let progressSpring = spring(0, {
    stiffness: 0.15,
    damping: 0.8,
    precision: 0.5,
  });

  const precisePosition = $derived(() => {
    return currentTimeMs;
  });

  const progressPercent = $derived(
    durationMs > 0 ? (precisePosition() / durationMs) * 100 : 0,
  );

  let widthSpring = spring(80, {
    stiffness: 0.2,
    damping: 0.85,
    precision: 0.1,
  });
  let heightSpring = spring(28, {
    stiffness: 0.2,
    damping: 0.85,
    precision: 0.1,
  });
  let contentOpacity = spring(0, {
    stiffness: 0.15,
    damping: 0.8,
    precision: 0.01,
  });

  let win: ReturnType<typeof getCurrentWindow>;

  let cachedScreenWidth = 0;
  let cachedScreenHeight = 0;
  let isSyncing = false;
  let pendingW = 0;
  let pendingH = 0;
  let hasPendingSync = false;

  let monitorAnchorX = 0;
  let monitorAnchorY = 0;

  let lastSyncTime = 0;
  const SYNC_COOLDOWN_MS = 16;

  async function processSyncQueue() {
    if (isSyncing || !hasPendingSync) return;

    isSyncing = true;
    hasPendingSync = false;
    lastSyncTime = performance.now();

    const w = pendingW;
    const h = pendingH;
    const dpr = window.devicePixelRatio || 1;

    try {
      if (!cachedScreenWidth) {
        const monitor = await currentMonitor();
        if (monitor) {
          cachedScreenWidth = monitor.size.width;
          cachedScreenHeight = monitor.size.height;
          monitorAnchorX = monitor.position.x + monitor.size.width / 2;
          monitorAnchorY = monitor.position.y;
        }
      }

      const physW = Math.round(w * dpr);
      const physH = Math.round(h * dpr);
      const centerX = Math.round(monitorAnchorX - physW / 2);
      const targetY = Math.round(monitorAnchorY + 22 * dpr);

      await Promise.all([
        win.setSize(new PhysicalSize(physW, physH)),
        win.setPosition(new PhysicalPosition(centerX, targetY)),
      ]);
    } catch (err) {
      logger.error("窗口同步失败:", err);
    } finally {
      isSyncing = false;
      if (hasPendingSync) {
        const elapsed = performance.now() - lastSyncTime;
        if (elapsed < SYNC_COOLDOWN_MS) {
          setTimeout(processSyncQueue, SYNC_COOLDOWN_MS - elapsed);
        } else {
          requestAnimationFrame(processSyncQueue);
        }
      }
    }
  }

  let lastW = 0;
  let lastH = 0;

  function isNearTarget(current: number, ...targets: number[]): boolean {
    return targets.some((t) => Math.abs(current - t) < 2);
  }

  $effect(() => {
    const currentW = $widthSpring;
    const currentH = $heightSpring;

    const nearTarget =
      isNearTarget(currentW, 80, 90, 300) &&
      isNearTarget(currentH, 28, 30, 160);
    const syncThreshold = nearTarget ? 0.5 : 1.5;

    if (
      Math.abs(currentW - lastW) > syncThreshold ||
      Math.abs(currentH - lastH) > syncThreshold
    ) {
      pendingW = currentW;
      pendingH = currentH;
      hasPendingSync = true;

      if (!isSyncing) {
        requestAnimationFrame(processSyncQueue);
      }

      lastW = currentW;
      lastH = currentH;
    }
  });

  function startAutoClose() {
    stopAutoClose();
    if (expanded && !hovering) {
      const delay = appSettings.enableAnimations ? 5000 : 3000;
      logger.log(`开始自动收起计时器: ${delay}ms`);
      autoCloseTimer = setTimeout(() => {
        logger.log("自动收起计时器触发");
        expanded = false;
        autoCloseTimer = null;
      }, delay);
    }
  }

  function stopAutoClose() {
    if (autoCloseTimer) {
      clearTimeout(autoCloseTimer);
      autoCloseTimer = null;
    }
  }

  $effect(() => {
    if (expanded) {
      startAutoClose();
    } else {
      stopAutoClose();
      showMonitorMenu = false;
    }
  });

  function handleMouseEnter() {
    hovering = true;
    stopAutoClose();
  }

  function handleMouseLeave() {
    hovering = false;
    logger.log("鼠标离开，开始自动收起计时器");
    if (expanded) {
      startAutoClose();
    }
  }

  async function toggleFloatingWindow() {
    try {
      if (isFloatingWindowOpen) {
        await windowApi.closeFloatingWindow();
        isFloatingWindowOpen = false;
      } else {
        await windowApi.openFloatingWindow();
        isFloatingWindowOpen = true;
      }
    } catch (error) {
      logger.error("切换悬浮窗失败:", error);
    }
  }

  // 颜色提取辅助函数
  function parseColorKey(key: string): [number, number, number] {
    return key.split(",").map(Number) as [number, number, number];
  }

  function calculateBrightness(r: number, g: number, b: number): number {
    return (r + g + b) / 3;
  }

  function clampColorValue(v: number): number {
    return Math.max(80, Math.min(255, v));
  }

  function formatRgb(r: number, g: number, b: number): string {
    return `rgb(${r},${g},${b})`;
  }

  async function extractDominantColor(imgSrc: string) {
    if (!imgSrc) {
      accentColor = currentColor;
      secondaryColor = currentColor;
      return;
    }

    try {
      const img = new Image();

      // 对于本地文件和 data URL，不设置 crossOrigin
      if (!imgSrc.startsWith("file://") && !imgSrc.startsWith("data:")) {
        img.crossOrigin = "Anonymous";
      }
      img.src = imgSrc;

      // 添加超时处理
      const loadPromise = new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(
          () => reject(new Error("图片加载超时")),
          5000,
        );
        img.onload = () => {
          clearTimeout(timeout);
          resolve();
        };
        img.onerror = () => {
          clearTimeout(timeout);
          reject(new Error("图片加载失败"));
        };
      });

      await loadPromise;

      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        throw new Error("无法获取 canvas 上下文");
      }
      canvas.width = 24;
      canvas.height = 24;
      ctx.drawImage(img, 0, 0, 24, 24);

      const data = ctx.getImageData(0, 0, 24, 24).data;

      const colorMap: Map<string, number> = new Map();

      // 统计所有颜色的出现频率
      for (let i = 0; i < data.length; i += 4) {
        const r = Math.floor(data[i] / 16) * 16;
        const g = Math.floor(data[i + 1] / 16) * 16;
        const b = Math.floor(data[i + 2] / 16) * 16;
        const a = data[i + 3];

        // 跳过透明像素
        if (a < 128) continue;

        const key = `${r},${g},${b}`;
        colorMap.set(key, (colorMap.get(key) || 0) + 1);
      }

      const sortedColors = [...colorMap.entries()].sort((a, b) => b[1] - a[1]);

      if (sortedColors.length >= 2) {
        const [bottomKey] = sortedColors[0];
        const [rBottom, gBottom, bBottom] = parseColorKey(bottomKey);

        const bottomBrightness = calculateBrightness(rBottom, gBottom, bBottom);
        let topColor = sortedColors[1];
        let maxContrast = 0;

        for (let i = 1; i < sortedColors.length; i++) {
          const [colorKey] = sortedColors[i];
          const [r, g, b] = parseColorKey(colorKey);
          const brightness = calculateBrightness(r, g, b);
          const contrast = Math.abs(brightness - bottomBrightness);

          if (contrast > maxContrast) {
            maxContrast = contrast;
            topColor = sortedColors[i];
          }
        }

        const [topKey] = topColor;
        const [rTop, gTop, bTop] = parseColorKey(topKey);

        if (topKey === bottomKey && sortedColors.length >= 3) {
          const [secondKey] = sortedColors[1];
          const [r2, g2, b2] = parseColorKey(secondKey);
          accentColor = formatRgb(
            clampColorValue(r2),
            clampColorValue(g2),
            clampColorValue(b2),
          );
        } else {
          accentColor = formatRgb(
            clampColorValue(rBottom),
            clampColorValue(gBottom),
            clampColorValue(bBottom),
          );
        }

        secondaryColor = formatRgb(
          clampColorValue(rTop),
          clampColorValue(gTop),
          clampColorValue(bTop),
        );

        spectrumBottomColor = formatRgb(
          clampColorValue(rBottom),
          clampColorValue(gBottom),
          clampColorValue(bBottom),
        );
        spectrumTopColor = formatRgb(
          clampColorValue(rTop),
          clampColorValue(gTop),
          clampColorValue(bTop),
        );
      } else if (sortedColors.length === 1) {
        const [mainKey] = sortedColors[0];
        const [r, g, b] = parseColorKey(mainKey);
        accentColor = formatRgb(r, g, b);
        secondaryColor = accentColor;
      } else {
        accentColor = currentColor;
        secondaryColor = currentColor;
      }
    } catch (e) {
      console.warn("取色失败，将使用默认颜色", e);
      accentColor = currentColor;
      secondaryColor = currentColor;
    }
  }

  $effect(() => {
    if (artworkUrl) {
      extractDominantColor(artworkUrl);
    } else {
      accentColor = currentColor;
      secondaryColor = currentColor;
    }
  });

  $effect(() => {
    const isExp = expanded;
    const isHov = hovering;
    const reduced = appSettings.reduceAnimations;
    const animEnabled = appSettings.enableAnimations;

    requestAnimationFrame(() => {
      if (isExp) {
        widthSpring.set(300);
        heightSpring.set(160);

        if (!animEnabled) {
          contentOpacity.set(1);
        } else if (reduced) {
          contentOpacity.set(1);
        } else {
          setTimeout(() => contentOpacity.set(1), 80);
        }
      } else {
        contentOpacity.set(0);

        setTimeout(() => {
          widthSpring.set(isHov ? 90 : 80);
          heightSpring.set(isHov ? 30 : 28);
        }, 60);
      }
    });
  });

  let animatingTimer: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    const _ = expanded;
    if (animatingTimer) clearTimeout(animatingTimer);
    isAnimating = true;
    animatingTimer = setTimeout(() => {
      isAnimating = false;
      animatingTimer = null;
    }, 300);
  });

  let isPressed = $state(false);

  function handlePress() {
    isPressed = true;
  }

  function handleRelease(e: MouseEvent) {
    isPressed = false;

    const target = e.target as HTMLElement;
    if (target.closest("button") || target.closest("[data-stop-toggle]")) {
      return;
    }

    expanded = !expanded;
  }

  async function handleMediaAction(action: string, e: MouseEvent) {
    e.stopPropagation();

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

  function formatTime(ms: number): string {
    if (ms <= 0) return "00:00";
    const s = Math.floor(ms / 1000);
    const min = Math.floor(s / 60);
    const sec = s % 60;
    return `${min.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`;
  }

  let fullscreenCheckInterval: ReturnType<typeof setInterval> | null = null;
  let hideTimeout: ReturnType<typeof setTimeout> | null = null;

  // 处理全屏状态变化
  function handleFullscreenChange(isFullscreen: boolean) {
    if (!autoHideEnabled || !appSettings.autoHide) return;

    if (isFullscreen !== isFullscreenApp) {
      isFullscreenApp = isFullscreen;
      console.log(
        "[全屏检测] 状态变化:",
        isFullscreen ? "检测到全屏应用" : "全屏应用已关闭",
      );

      if (isFullscreen) {
        hideWindowToTop();
      } else {
        showWindow();
      }
    }
  }

  async function hideWindowToTop() {
    if (!appSettings.autoHide) return;

    try {
      const appWindow = getCurrentWindow();
      const currentSize = await appWindow.innerSize();

      const allMonitors = await availableMonitors();

      if (allMonitors.length > 0 && currentMonitorIndex < allMonitors.length) {
        const targetMonitor = allMonitors[currentMonitorIndex];

        const screenCenterX =
          targetMonitor.position.x + targetMonitor.size.width / 2;
        const windowCenterX = screenCenterX - currentSize.width / 2;
        const targetY = Math.round(-currentSize.height + 2);

        await appWindow.setPosition(
          new PhysicalPosition(Math.round(windowCenterX), targetY),
        );
        isHidden = true;
        console.log("[自动隐藏] 窗口已隐藏到顶部中间，留 2px 可见边");
      } else {
        const targetY = Math.round(-currentSize.height + 2);
        await appWindow.setPosition(new PhysicalPosition(0, targetY));
        isHidden = true;
        console.log("[自动隐藏] 未找到显示器，使用默认位置");
      }
    } catch (error) {
      console.error("[自动隐藏] 失败:", error);
    }
  }

  async function showWindow() {
    try {
      const appWindow = getCurrentWindow();
      const currentSize = await appWindow.innerSize();
      const dpr = window.devicePixelRatio || 1;

      const allMonitors = await availableMonitors();

      if (allMonitors.length > 0 && currentMonitorIndex < allMonitors.length) {
        const targetMonitor = allMonitors[currentMonitorIndex];

        const screenCenterX =
          targetMonitor.position.x + targetMonitor.size.width / 2;
        const windowCenterX = screenCenterX - currentSize.width / 2;
        const targetY = Math.round(22 * dpr);

        await appWindow.setPosition(
          new PhysicalPosition(Math.round(windowCenterX), targetY),
        );
        isHidden = false;
        console.log("[自动显示] 窗口已显示在顶部中间");
      } else {
        const targetY = Math.round(22 * dpr);
        await appWindow.setPosition(new PhysicalPosition(0, targetY));
        isHidden = false;
        console.log("[自动显示] 未找到显示器，使用默认位置");
      }
    } catch (error) {
      console.error("[自动显示] 失败:", error);
    }
  }

  async function handleMouseMove(event: MouseEvent) {
    if (!autoHideEnabled || !appSettings.autoHide || !isFullscreenApp) return;

    const mouseY = event.clientY;
    const wasMouseAtTop = isMouseAtTop;
    isMouseAtTop = mouseY < 100;

    if (isMouseAtTop !== wasMouseAtTop) {
      console.log("[鼠标检测] 鼠标在顶部:", isMouseAtTop);

      if (isMouseAtTop && isHidden) {
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
        const savedSettings = await settingsApi.getSettings();
        await settingsApi.saveSettings({
          ...savedSettings,
          monitorIndex: index,
        });
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
    monitorAnchorX = targetMonitor.position.x + targetMonitor.size.width / 2;
    monitorAnchorY = targetMonitor.position.y;
    cachedScreenWidth = targetMonitor.size.width;
    cachedScreenHeight = targetMonitor.size.height;

    const appWindow = getCurrentWindow();
    const currentSize = await appWindow.innerSize();

    const targetX = Math.round(monitorAnchorX - currentSize.width / 2);
    const targetY = Math.round(monitorAnchorY + 22);

    await appWindow.setPosition(new PhysicalPosition(targetX, targetY));

    console.log(
      "[显示器] 已移动到:",
      targetMonitor.name || `显示器`,
      "位置:",
      targetX,
      targetY,
    );
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

      try {
        const savedSettings = await settingsApi.getSettings();
        const appWindow = getCurrentWindow();
        await appWindow.setAlwaysOnTop(savedSettings.alwaysOnTop ?? true);
        console.log("[置顶设置] 已应用:", savedSettings.alwaysOnTop);
      } catch (error) {
        console.error("[置顶设置] 读取失败:", error);
      }

      try {
        const loadedSettings = await settingsApi.getSettings();
        appSettings = { ...DEFAULT_SETTINGS, ...loadedSettings };
        console.log("[设置] 已加载:", appSettings);
      } catch (error) {
        console.error("[设置] 读取失败:", error);
      }

      try {
        displayRefreshRate = await detectDisplayRefreshRate();
        console.log(`[性能] 显示器刷新率: ${displayRefreshRate}Hz`);

        performanceLevel = detectPerformanceLevel();
        console.log(`[性能] 设备性能等级: ${performanceLevel}`);

        updateSpringParams();

        startFpsMonitoring();
        console.log("[性能] 帧率监控已启动");

        if (highFrameRateMode) {
          console.log(`[性能] 🚀 高帧率模式已启用 (${displayRefreshRate}Hz)`);
        }
      } catch (error) {
        console.error("[性能] 初始化失败:", error);
        performanceLevel = "high";
        displayRefreshRate = 60;
      }

      const unlistenSettings = await eventManager.on(
        Events.SETTINGS_UPDATED,
        (s: any) => {
          if (s) {
            appSettings = s;
            console.log("[设置] 实时更新:", appSettings);

            if (s.islandTheme) {
              currentTheme = s.islandTheme;
            }

            if (s.monitorIndex !== undefined) {
              currentMonitorIndex = s.monitorIndex;
            }

            if (!appSettings.autoHide && isHidden) {
              showWindow();
            }
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
          } else if (settingName === "islandTheme") {
            currentTheme = appSettings.islandTheme;
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

      const unlistenCornerRadiusChanged = await eventManager.on(
        Events.CORNER_RADIUS_CHANGED,
        (radius: any) => {
          console.log("[设置] 圆角变更:", radius);
          appSettings.expandedCornerRadius = radius;
        },
      );
      cleanups.push(unlistenCornerRadiusChanged);

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

        const savedSettings = await settingsApi.getSettings();
        const savedMonitorIndex = savedSettings.monitorIndex ?? 0;

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

      const unlistenTheme = await eventManager.on(
        Events.THEME_CHANGED,
        ({ islandTheme }: any) => {
          currentTheme = islandTheme || "original";
          console.log("[主题切换] 切换到:", currentTheme);
        },
      );
      cleanups.push(unlistenTheme);

      try {
        const savedSettings = await settingsApi.getSettings();
        currentTheme = savedSettings.islandTheme || "original";
        console.log("[主题加载] 从设置加载主题:", currentTheme);
      } catch (e) {
        console.error("[主题加载] 失败:", e);
      }

      const unlistenMediaUpdate = await onMediaUpdate((data: any) => {
        if (data.source) currentSource = data.source;
        isPlaying = data.isPlaying || false;

        const currentSongKey = `${data.title || ""}-${data.artist || ""}`;
        const songChanged = lastSongKey !== currentSongKey;

        if (songChanged) {
          maxBackendPosition = 0;
          lastSongKey = currentSongKey;
        }

        const newPosition = data.positionMs || 0;

        maxBackendPosition = Math.max(maxBackendPosition, newPosition);

        const isBackendStuck =
          newPosition < 1000 &&
          currentTimeMs > 3000 &&
          maxBackendPosition < 1000 &&
          !songChanged;

        if (isBackendStuck) {
          // 后端完全拿不到进度，忽略覆盖，全靠前端自己计时
        } else {
          if (
            Math.abs(currentTimeMs - newPosition) > 3000 ||
            songChanged ||
            !isPlaying
          ) {
            currentTimeMs = newPosition;
          }
        }

        if (songChanged) {
          console.log("[歌曲变更] 检测到新歌:", data.title, "-", data.artist);
          lastSongKey = currentSongKey;

          if (data.durationMs && data.durationMs > 1000) {
            durationMs = data.durationMs;
            console.log("[时长] ✓ 使用 SMTC 提供的有效时长:", durationMs, "ms");
          } else {
            const songName = data.title || trackTitle;
            const resolvedArtist = data.artist || artistName;

            if (
              songName &&
              songName !== "未知曲目" &&
              resolvedArtist &&
              resolvedArtist !== "未知艺术家"
            ) {
              mediaApi
                .getNeteaseSongInfo(songName, resolvedArtist)
                .then((songInfo) => {
                  if (songInfo) {
                    if (songInfo.duration && songInfo.duration > 0) {
                      durationMs = songInfo.duration;
                      console.log(
                        "[网易云 API] ✓ 获取时长成功:",
                        songInfo.duration,
                        "ms",
                      );
                    }
                    if (
                      songInfo.albumPic &&
                      (!rawCoverUrl || rawCoverUrl === "")
                    ) {
                      const highQualityPic = songInfo.albumPic.replace(
                        /(\d+)x(\d+)\.jpg/,
                        "1024y1024.jpg",
                      );
                      console.log(
                        "[网易云 API] ✓ 获取专辑图片:",
                        highQualityPic,
                      );
                    }
                    if (songInfo.mvId && songInfo.mvId > 0) {
                      console.log("[网易云 API] ✓ 发现 MV，ID:", songInfo.mvId);
                      if (songInfo.mvUrl) {
                        console.log(
                          "[网易云 API] ✓ MV 播放链接:",
                          songInfo.mvUrl,
                        );
                      }
                    }
                  } else {
                    console.warn("[网易云 API] ✗ 未找到歌曲信息");
                  }
                })
                .catch((err) => {
                  console.error("[网易云 API] ✗ 获取歌曲信息失败:", err);
                });
            }
          }
        }

        const titleChanged = trackTitle !== data.title;
        const artistChanged = artistName !== data.artist;

        const newCover =
          data.albumArt ||
          data.thumbnail ||
          data.coverUrl ||
          data.api_cover_url ||
          data.image ||
          "";

        const coverChanged = newCover !== rawCoverUrl;

        if (titleChanged || artistChanged || coverChanged) {
          if (titleChanged) {
            trackTitle = data.title || "未知曲目";
            setTimeout(() => {
              const titleEl = document.querySelector(
                ".marquee-text",
              ) as HTMLElement;
              const wrapperEl = document.querySelector(
                ".marquee-wrapper",
              ) as HTMLElement;
              if (titleEl && wrapperEl) {
                titleEl.classList.remove("marquee-active");
                titleEl.style.transform = "";

                requestAnimationFrame(() => {
                  if (titleEl.scrollWidth > titleEl.clientWidth) {
                    titleEl.classList.add("marquee-active");
                  }
                });
              }
            }, 100);
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
              flipKey += 1;
            } else if (
              newCover &&
              (newCover.includes(":\\") || newCover.includes(":/"))
            ) {
              artworkUrl = convertFileSrc(newCover);
              flipKey += 1;
            } else {
              artworkUrl = "";
            }
          }

          progressSpring.set(0, { soft: true });
        }

        if (durationMs > 0) {
          progressSpring.set((currentTimeMs / durationMs) * 100);
        }
      });
      cleanups.push(unlistenMediaUpdate);
    })();

    return () => {
      cleanups.forEach((fn) => fn && fn());
    };
  });

  onDestroy(() => {
    stopAutoClose();
    stopDebugFps();
  });

  function handleGlobalClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (showMonitorMenu && !target.closest(".relative")) {
      closeMonitorMenu();
    }
  }

  onMount(() => {
    win = getCurrentWindow();
    console.log("[App.svelte] 窗口对象已初始化");

    let progressInterval: ReturnType<typeof setInterval> | null = null;

    const startProgressUpdate = () => {
      if (progressInterval) return;

      progressInterval = setInterval(() => {
        if (isPlaying && durationMs > 0 && currentTimeMs < durationMs) {
          currentTimeMs += 100;
          if (currentTimeMs > durationMs) {
            currentTimeMs = durationMs;
          }
        }
      }, 100);
    };

    const stopProgressUpdate = () => {
      if (progressInterval) {
        clearInterval(progressInterval);
        progressInterval = null;
      }
    };

    $effect(() => {
      if (isPlaying) {
        startProgressUpdate();
      } else {
        stopProgressUpdate();
      }
    });

    document.addEventListener("click", handleGlobalClick);
    return () => {
      document.removeEventListener("click", handleGlobalClick);
      stopProgressUpdate();
    };
  });

  onMount(() => {
    // 1. 初始加载设置里的主题
    (async () => {
      try {
        const savedSettings = await settingsApi.getSettings();
        currentTheme = savedSettings.islandTheme || "original";
        console.log("[主题加载] 初始主题:", currentTheme);
      } catch (e) {
        console.error("[主题加载] 失败:", e);
      }
    })();

    // 2. 监听来自设置页面的实时切换广播
    const unlistenTheme = listen("theme-changed", (event) => {
      currentTheme = event.payload as string;
      console.log("[主题切换] 主题已切换为:", currentTheme);
    });

    // 监听后端推送的全屏状态变化事件
    const unlistenFullscreen = eventManager.on(
      "fullscreen-changed",
      (event) => {
        handleFullscreenChange(event.payload);
      },
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
      // 清理主题监听
      unlistenTheme.then((unlisten) => unlisten());
      // 清理全屏监听
      unlistenFullscreen.then((unlisten) => unlisten());

      if (hideTimeout) {
        clearTimeout(hideTimeout);
      }
      if (mouseMoveTimeout) {
        clearTimeout(mouseMoveTimeout);
      }
      document.removeEventListener("mousemove", handleMouseMoveThrottled);
    };
  });
</script>

<div
  class="fixed inset-0 flex items-start justify-center pointer-events-none"
  style="background: transparent;"
>
  <div
    class="pointer-events-auto relative"
    class:theme-original={currentTheme === "original"}
    class:island-hidden={isHidden && !isMouseAtTop}
    class:island-drop-animation={isMouseAtTop && isHidden}
    class:island-visible-edge={isHidden && isMouseAtTop}
    style="
      width: {$widthSpring}px;
      height: {$heightSpring}px;
      background: {getThemeBackground(currentTheme)};
      background-size: {getThemeBackgroundSize(currentTheme)};
      background-position: {getThemeBackgroundPosition(currentTheme)};
      backdrop-filter: {getThemeBackdropFilter(currentTheme)};
      -webkit-backdrop-filter: {getThemeBackdropFilter(currentTheme)};
      border: {getThemeBorder(currentTheme)};
      box-shadow: {getThemeBoxShadow(currentTheme, isHidden, expanded)};
      border-radius: {getDynamicBorderRadius($heightSpring)};
      overflow: hidden;
      display: flex;
      flex-direction: column;
      transform: scale({isPressed ? 0.96 : 1}) translateZ(0);
      {isAnimating ? 'will-change: transform, width, height;' : ''}
      transition:
        transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1),
        box-shadow 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
    "
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
    onmousedown={() => (isPressed = true)}
    onmouseup={handleRelease}
    onkeydown={(e) => e.key === "Enter" && (expanded = !expanded)}
    role="button"
    tabindex="0"
    aria-label="Dynamic Island - Click to toggle"
  >
    <!-- 调试信息覆盖层 -->
    {#if appSettings.showDebugInfo}
      <div class="debug-overlay">
        <span>FPS: {currentFps}</span>
        <span>刷新率: {displayRefreshRate}Hz</span>
        <span>高帧率: {highFrameRateMode ? "✓" : "✗"}</span>
        <span
          >性能: {performanceLevel === "high"
            ? "高性能"
            : performanceLevel === "medium"
              ? "中等"
              : "低性能"}</span
        >
        <span>Theme: {currentTheme}</span>
        <span>Src: {currentSource}</span>
        <span>Pos: {currentTimeMs}ms</span>
        <span>Hidden: {isHidden}</span>
      </div>
    {/if}

    <div class="absolute inset-0 z-0" data-tauri-drag-region></div>

    <div class="w-full h-full relative z-10 overflow-hidden">
      <!-- 收起态内容 -->
      <div
        class="collapsed-content"
        class:is-hidden={expanded}
        style="opacity: {1 - $contentOpacity};"
      >
        {#if showTimeDisplay}
          <div
            class="h-full w-full flex items-center justify-center select-none"
          >
            <div class="time-display">
              <span>{currentTime}</span>
            </div>
          </div>
        {:else}
          <div
            class="h-full w-full flex items-center justify-between select-none"
          >
            <div
              class="w-5 h-5 rounded overflow-hidden flex-shrink-0 select-none cursor-pointer"
              style="background-color: rgba(255, 255, 255, 0.05);"
              role="button"
              tabindex="0"
              data-stop-toggle
              onclick={(e) => {
                e.stopPropagation();
                openCurrentPlayer();
              }}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.stopPropagation();
                  openCurrentPlayer();
                }
              }}
            >
              {#if artworkUrl}
                {#key flipKey}
                  <img
                    src={artworkUrl}
                    alt=""
                    class="w-full h-full object-cover flip-enter"
                    onload={() => console.log("[图片加载] 成功加载封面")}
                    onerror={(e) => {
                      console.error("[图片加载] 封面加载失败:", artworkUrl);
                      (e.currentTarget as HTMLImageElement).style.display =
                        "none";
                    }}
                  />
                {/key}
              {:else}
                <div class="w-full h-full flex items-center justify-center">
                  <svg
                    class="w-3 h-3 text-white/20"
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

            {#if appSettings.showSpectrum}
              <div class="spectrum-wrapper">
                <Spectrum topColor={spectrumBottomColor} bottomColor={spectrumTopColor} />
              </div>
            {:else}
              <div class="flex items-center h-4 gap-[3px]">
                {#if isPlaying}
                  <div
                    class="w-[3px] h-[3px] rounded-full animate-pulse"
                    style="background-color: {accentColor};"
                  ></div>
                {/if}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- 展开态内容 -->
      <div
        class="expanded-content"
        class:is-visible={expanded}
        style="opacity: {$contentOpacity};"
      >
        <div class="ui-content-layer">
          <!-- 顶部区域：封面 + 标题 + 频谱 -->
          <div
            class="flex items-center justify-between"
            style="gap: 12px; margin-bottom: 12px;"
          >
            <div
              class="w-[52px] h-[52px] rounded-[12px] overflow-hidden shadow-2xl ring-1 ring-white/10 flex-shrink-0 cursor-pointer select-none transition-all duration-300 hover:scale-105 hover:shadow-xl"
              style="background-color: rgba(255, 255, 255, 0.05);"
              role="button"
              tabindex="0"
              data-stop-toggle
              onclick={(e) => {
                e.stopPropagation();
                openCurrentPlayer();
              }}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.stopPropagation();
                  openCurrentPlayer();
                }
              }}
            >
              {#if artworkUrl}
                {#key flipKey}
                  <img
                    src={artworkUrl}
                    alt="cover"
                    class="w-full h-full object-cover pointer-events-none flip-enter"
                    onload={() =>
                      console.log("[图片加载] 成功加载封面 (展开状态)")}
                    onerror={(e) => {
                      console.error(
                        "[图片加载] 封面加载失败 (展开状态):",
                        artworkUrl,
                      );
                      (e.currentTarget as HTMLImageElement).style.display =
                        "none";
                    }}
                  />
                {/key}
              {:else}
                <div class="w-full h-full flex items-center justify-center">
                  <svg
                    class="w-8 h-8 text-white/20"
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

            <div class="flex-1 min-w-0">
              <div class="marquee-wrapper relative overflow-hidden">
                <h2
                  class="marquee-text dynamic-glass-text select-none leading-tight mb-1 whitespace-nowrap"
                  style="font-size: clamp(12px, 4vw, 18px); font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Inter', sans-serif; font-weight: 700; letter-spacing: -0.03em;"
                  data-full-title={trackTitle}
                >
                  {trackTitle}
                </h2>
              </div>
              <p
                class="truncate dynamic-glass-text-secondary select-none leading-tight"
                style="font-size: clamp(10px, 3vw, 14px); font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Inter', sans-serif; font-weight: 500; letter-spacing: -0.01em;"
              >
                {artistName}
              </p>
            </div>

            {#if appSettings.showSpectrum}
              <div class="spectrum-wrapper-expanded">
                <Spectrum topColor={spectrumBottomColor} bottomColor={spectrumTopColor} scale={1.5} />
              </div>
            {/if}
          </div>

          <!-- 中部区域：进度条 -->
          <div
            class="relative flex items-center justify-center"
            style="margin-bottom: 10px; width: 100%;"
          >
            <div class="w-full">
              <div class="progress-bar">
                <div
                  class="progress-fill"
                  style="width: {durationMs > 0
                    ? (currentTimeMs / durationMs) * 100
                    : 0}%"
                ></div>
              </div>
              <div class="flex justify-between mt-1">
                <span class="text-[10px] text-white/60"
                  >{formatTime(currentTimeMs)}</span
                >
                <span class="text-[10px] text-white/60"
                  >-{formatTime(durationMs - currentTimeMs)}</span
                >
              </div>
            </div>
          </div>

          <!-- 中部区域：播放控制按钮 -->
          <div
            class="relative flex items-center justify-center"
            style="margin-bottom: 10px; width: 100%;"
          >
            <div
              class="flex items-center justify-center"
              style="
                gap: 20px;
                will-change: auto;
                transform: translate3d(0, 0, 0);
                backface-visibility: hidden;
                perspective: 1000px;
              "
            >
              <button
                class="flex items-center justify-center text-white/90 hover:scale-110 active:scale-90 transition-all duration-300 relative z-50 cursor-pointer media-button bg-transparent border-none p-0"
                style="width: 32px; height: 32px;"
                data-stop-toggle
                onclick={(e) => handleMediaAction("prev", e)}
              >
                <SkipBack size={22} fill="currentColor" />
              </button>

              <button
                class="flex items-center justify-center text-white hover:scale-110 active:scale-95 transition-all duration-300 relative z-50 cursor-pointer media-button bg-transparent border-none p-0"
                style="width: 40px; height: 40px;"
                data-stop-toggle
                onclick={(e) => handleMediaAction("play_pause", e)}
              >
                {#if isPlaying}
                  <Pause size={32} fill="currentColor" />
                {:else}
                  <Play size={32} fill="currentColor" />
                {/if}
              </button>

              <button
                class="flex items-center justify-center text-white/90 hover:scale-110 active:scale-90 transition-all duration-300 relative z-50 cursor-pointer media-button bg-transparent border-none p-0"
                style="width: 32px; height: 32px;"
                data-stop-toggle
                onclick={(e) => handleMediaAction("next", e)}
              >
                <SkipForward size={22} fill="currentColor" />
              </button>
            </div>

            <!-- 悬浮窗按钮 -->
            <div class="absolute right-0" style="transform: translateZ(0);">
              <button
                class="w-7 h-7 flex items-center justify-center rounded-xl border border-white/10 text-white/90 hover:scale-110 active:scale-90 transition-all duration-300 relative z-50 cursor-pointer media-button hover:border-white/20"
                style="transform: translateZ(0); backface-visibility: hidden;"
                data-stop-toggle
                aria-label={isFloatingWindowOpen ? "关闭悬浮窗" : "打开悬浮窗"}
                onclick={(e) => {
                  e.stopPropagation();
                  toggleFloatingWindow();
                }}
                onkeydown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.stopPropagation();
                    toggleFloatingWindow();
                  }
                }}
              >
                <GalleryHorizontalEnd size={18} />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  /* ========== 全局基础样式 ========== */
  :global(*) {
    box-sizing: border-box;
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  /* ========== 时间显示 ========== */
  .time-display {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    animation: time-fade-in 0.5s ease-out forwards;
  }

  .time-display span {
    font-size: 12px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.8);
    letter-spacing: 0.05em;
    font-variant-numeric: tabular-nums;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
  }

  @keyframes time-fade-in {
    from {
      opacity: 0;
      transform: scale(0.9);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  /* ========== 频谱 Canvas ========== */
  .spectrum-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 28px;
  }

  .collapsed-content .spectrum-wrapper {
    height: 18px;
    overflow: hidden;
  }

  .spectrum-wrapper-expanded {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    height: 40px;
    flex-shrink: 0;
  }

  /* ========== 进度条 ========== */
  .progress-bar {
    width: 100%;
    height: 3px;
    background: rgba(255, 255, 255, 0.12);
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.4), rgba(255, 255, 255, 0.8));
    border-radius: 999px;
    transition: width 0.15s linear;
  }

  /* ===== 通用文本样式 ===== */
  .dynamic-glass-text {
    color: #ffffff;
    text-shadow:
      0 0 1px rgba(0, 0, 0, 0.4),
      0 1px 4px rgba(0, 0, 0, 0.3);
    -webkit-font-smoothing: antialiased;
  }

  .dynamic-glass-text-secondary {
    color: rgba(255, 255, 255, 0.8);
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    -webkit-font-smoothing: antialiased;
  }

  .ui-content-layer {
    position: relative;
    z-index: 2;
  }

  /* ===== 调试信息覆盖层 ===== */
  .debug-overlay {
    position: absolute;
    top: 4px;
    left: 8px;
    z-index: 200;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    pointer-events: none;
    font-family: "JetBrains Mono", monospace;
    font-size: 9px;
    font-weight: 500;
    color: #4ade80;
    background: rgba(0, 0, 0, 0.7);
    padding: 3px 8px;
    border-radius: 6px;
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    letter-spacing: 0.02em;
    line-height: 1.4;
  }

  .debug-overlay span {
    white-space: nowrap;
  }

  .debug-overlay span {
    animation: island-water-drop 0.25s cubic-bezier(0.32, 0.72, 0, 1) forwards;
    will-change: transform, opacity;
  }

  @keyframes island-water-drop {
    0% {
      opacity: 0;
      transform: translateY(-100%) scale(0.8, 0.6);
      border-radius: 50% 50% 30% 30%;
    }
    40% {
      transform: translateY(10%) scale(1.05, 0.95);
      border-radius: 45% 45% 35% 35%;
    }
    70% {
      transform: translateY(-5%) scale(0.98, 1.02);
      border-radius: 45% 45% 40% 40%;
    }
    100% {
      opacity: 1;
      transform: translateY(0) scale(1, 1);
      border-radius: 45px;
    }
  }

  .island-hidden {
    transition:
      transform 0.35s cubic-bezier(0.32, 0.72, 0, 1),
      opacity 0.3s cubic-bezier(0.32, 0.72, 0, 1);
    transform: translateY(-100%);
    opacity: 0;
    pointer-events: none;
    will-change: transform, opacity;
  }

  .island-visible-edge {
    box-shadow:
      0 2px 15px rgba(255, 255, 255, 0.3),
      0 0 20px rgba(255, 255, 255, 0.1),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
  }

  .island-visible-edge::before {
    content: "";
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 40px;
    height: 2px;
    background: linear-gradient(
      to bottom,
      rgba(255, 255, 255, 0.8),
      rgba(255, 255, 255, 0.3)
    );
    border-radius: 0 0 2px 2px;
    pointer-events: none;
  }

  /* 绸缎感动画核心 */
  .expanded-content {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    padding: 20px 28px 16px 28px;

    transform: translateY(30px) scale(0.92) translateZ(0);
    will-change: transform, filter, opacity;
    transition:
      transform 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94),
      filter 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94),
      opacity 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);

    filter: blur(8px);
    opacity: 0;
    pointer-events: none;
    will-change: transform, opacity, filter;
    transform: translate3d(0, 0, 0);
  }

  .expanded-content.is-visible {
    transform: translateY(0) scale(1) translateZ(0);
    filter: blur(0);
    opacity: 1;
    pointer-events: auto;
  }

  .collapsed-content {
    position: absolute;
    inset: 0;
    height: 100%;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;

    transition:
      transform 0.55s cubic-bezier(0.32, 0.72, 0, 1),
      opacity 0.55s cubic-bezier(0.32, 0.72, 0, 1);

    will-change: transform, opacity;
    transform: translate3d(0, 0, 0);
  }

  .collapsed-content.is-hidden {
    transform: translateY(-10px);
    opacity: 0;
    pointer-events: none;
  }

  .expanded-content.is-visible .flex-1 {
    animation: button-drop-in 0.4s cubic-bezier(0.32, 0.72, 0, 1) forwards;
    opacity: 0;
    transform: translateY(-30px) scale(0.85);
    will-change: transform, opacity;
  }

  .expanded-content.is-visible .flex-1 {
    animation: button-drop-in 0.4s cubic-bezier(0.32, 0.72, 0, 1) forwards;
    opacity: 0;
    transform: translateY(-30px) scale(0.85);
    will-change: transform, opacity;
  }

  @keyframes button-drop-in {
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .flip-enter {
    animation: flip-enter 0.8s cubic-bezier(0.25, 0.46, 0.45, 0.94);
    transform-origin: center;
    will-change: transform, opacity;
  }

  @keyframes cover-fade-in {
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes flip-enter {
    0% {
      transform: perspective(1000px) rotateY(-180deg) scale(0.8);
      opacity: 0;
    }
    20% {
      transform: perspective(1000px) rotateY(-140deg) scale(0.88);
      opacity: 0.2;
    }
    40% {
      transform: perspective(1000px) rotateY(-90deg) scale(0.93);
      opacity: 0.4;
    }
    60% {
      transform: perspective(1000px) rotateY(-50deg) scale(0.96);
      opacity: 0.65;
    }
    80% {
      transform: perspective(1000px) rotateY(-15deg) scale(0.98);
      opacity: 0.85;
    }
    100% {
      transform: perspective(1000px) rotateY(0deg) scale(1);
      opacity: 1;
    }
  }

  :global(html, body) {
    background: transparent !important;
    background-color: transparent !important;
    border: none !important;
    outline: none !important;
    margin: 0;
    padding: 0;
    width: 100vw;
    height: 100vh;
    pointer-events: auto;
    overflow: hidden;
    -webkit-app-region: no-drag;
    -webkit-backface-visibility: hidden;
    backface-visibility: hidden;
  }

  :global(#app),
  :global(main) {
    background: transparent !important;
  }

  .pointer-events-auto {
    -webkit-font-smoothing: antialiased;
    transform: translate3d(0, 0, 0) !important;
    will-change: transform;
    backface-visibility: hidden;
    perspective: 1000px;
    contain: layout style;
  }

  :global(*:focus) {
    outline: none !important;
    box-shadow: none !important;
    border: none !important;
  }

  :global(*:focus-visible) {
    outline: none !important;
    box-shadow: none !important;
    border: none !important;
  }

  button,
  [data-stop-toggle],
  .media-button {
    transform: translate3d(0, 0, 0) !important;
    backface-visibility: hidden !important;
    -webkit-font-smoothing: subpixel-antialiased;
    will-change: auto;
    perspective: 1000px;
    contain: layout style;
    background: transparent !important;
    border: none !important;
    box-shadow: none !important;
  }

  button:active {
    transform: scale(0.92) translateZ(0) !important;
    transition: transform 0.1s ease !important;
  }

  .expanded-content.is-visible .media-button {
    animation: button-icon-bounce 0.2s cubic-bezier(0.32, 0.72, 0, 1) forwards;
    opacity: 0;
    transform: translateY(-25px) scale(0.8);
    backface-visibility: hidden;
    will-change: transform, opacity;
  }

  .expanded-content.is-visible .media-button:nth-child(1) {
    animation-delay: 0.03s;
  }

  .expanded-content.is-visible .media-button:nth-child(2) {
    animation-delay: 0.06s;
  }

  .expanded-content.is-visible .media-button:nth-child(3) {
    animation-delay: 0.09s;
  }

  .expanded-content.is-visible .media-button:nth-child(4) {
    animation-delay: 0.12s;
  }

  @keyframes button-icon-bounce {
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  /* ─── Marquee 滚动效果 ─── */
  .marquee-wrapper {
    position: relative;
  }
  .marquee-text {
    display: inline-block;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  @keyframes marquee-scroll {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(-50%);
    }
  }
  .marquee-text::after {
    content: attr(data-full-title);
    position: absolute;
    left: 100%;
    white-space: nowrap;
  }
</style>
