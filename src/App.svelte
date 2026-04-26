<script lang="ts">
  import { onMount } from "svelte";
  import { spring } from "svelte/motion";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import {
    eventManager,
    onMediaUpdate,
    onAudioSpectrum,
  } from "./utils/eventManager";
  import { Events } from "./utils/eventConstants";
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

  // iOS风格频谱动画系统
  let spectrumPhase = $state(0);
  let targetSpectrumHeights = $state([0.5, 0.5, 0.5, 0.5, 0.5]);
  let targetSpectrumHeightsExpanded = $state(Array(40).fill(0.5));
  let spectrumTimer: number | null = null;

  const workArray5 = new Float32Array(5);
  const workArray40 = new Float32Array(40);

  let collapsedBars: HTMLDivElement[] = [];
  let expandedBars: HTMLDivElement[] = [];

  const baseCollapsedHeight = 20;
  const iOSCollapsedEnvelope = [0.5, 0.85, 1.0, 0.85, 0.5];
  const collapsedPhases = [0, 0.5, 1.0, 1.5, 0.8];

  const baseExpandedHeight = 30;

  const iOSExpandedEnvelope = [
    0.3, 0.4, 0.6, 0.8, 1.0, 1.15, 1.1, 1.0, 0.85, 0.75, 0.65, 0.6, 0.65, 0.75,
    0.85, 0.95, 1.0, 1.05, 1.05, 1.0, 1.0, 1.05, 1.05, 1.0, 0.95, 0.85, 0.75,
    0.65, 0.6, 0.65, 0.75, 0.85, 1.0, 1.1, 1.15, 1.0, 0.8, 0.6, 0.4, 0.3,
  ];

  const expandedPhases = [
    0, 0.3, 0.6, 0.9, 1.2, 1.5, 1.8, 2.1, 2.4, 2.7, 0.15, 0.45, 0.75, 1.05,
    1.35, 1.65, 1.95, 2.25, 2.55, 2.85,
  ];

  function startSpectrumAnimation() {
    if (spectrumTimer || !appSettings.show_spectrum) return;

    function animate() {
      if (!isPlaying || !appSettings.show_spectrum) {
        spectrumTimer = null;
        return;
      }

      spectrumTimer = requestAnimationFrame(animate);

      if (isPlaying) {
        for (let i = 0; i < 5; i++) {
          const target = targetSpectrumHeights[i] || 2;
          const current = workArray5[i] || 0.5;
          const diff = target - current;
          const tracking = diff > 0 ? 0.82 : 0.045;
          workArray5[i] = current + diff * tracking;
        }

        for (let i = 0; i < 5; i++) {
          const left = i > 0 ? workArray5[i - 1] : workArray5[i];
          const right = i < 4 ? workArray5[i + 1] : workArray5[i];
          workArray5[i] = workArray5[i] * 0.8 + left * 0.1 + right * 0.1;
        }

        for (let i = 0; i < 40; i++) {
          const target = targetSpectrumHeightsExpanded[i] || 2;
          const current = workArray40[i] || 0.5;
          const diff = target - current;
          const tracking = diff > 0 ? 0.9 : 0.035;
          workArray40[i] = current + diff * tracking;
        }

        for (let i = 0; i < 40; i++) {
          const left = i > 0 ? workArray40[i - 1] : workArray40[i];
          const right = i < 39 ? workArray40[i + 1] : workArray40[i];
          workArray40[i] = workArray40[i] * 0.92 + left * 0.04 + right * 0.04;
        }
      } else {
        for (let i = 0; i < 5; i++) {
          const current = workArray5[i] || 0.5;
          const diff = 2 - current;
          workArray5[i] = current + diff * 0.08;
        }
        for (let i = 0; i < 40; i++) {
          const current = workArray40[i] || 0.5;
          const diff = 2 - current;
          workArray40[i] = current + diff * 0.08;
        }
      }

      for (let i = 0; i < 5; i++) {
        if (collapsedBars[i]) {
          collapsedBars[i].style.transform =
            `scaleY(${workArray5[i]}) translateZ(0)`;
        }
      }

      for (let i = 0; i < 40; i++) {
        if (expandedBars[i]) {
          expandedBars[i].style.transform =
            `scaleY(${workArray40[i]}) translateZ(0)`;
        }
      }
    }

    spectrumTimer = requestAnimationFrame(animate);
  }

  function stopSpectrumAnimation() {
    if (spectrumTimer !== null) {
      cancelAnimationFrame(spectrumTimer);
      spectrumTimer = null;
    }
  }

  $effect(() => {
    if (isPlaying && appSettings.show_spectrum) {
      requestAnimationFrame(() => {
        startSpectrumAnimation();
      });
    }
  });
  let currentTimeMs = $state<number>(0);
  let durationMs = $state<number>(0);
  let currentSource = $state<string>("generic");
  let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;
  let currentTheme = $state("original");

  let maxBackendPosition = 0;

  let showTimeDisplay = $state(false);
  let currentTime = $state("");
  let pausedStartTime = $state<number>(0);

  function updateTimeDisplay() {
    const now = new Date();
    currentTime = `${now.getHours().toString().padStart(2, "0")}:${now.getMinutes().toString().padStart(2, "0")}`;
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
        await invoke("open_application", { name: appName });
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
    const backgrounds: Record<string, string> = {
      original: "#000000",
      ios26:
        "linear-gradient(90deg, #2d3748, #2a3345, #252d3d, #2a3345, #2d3748)",
      dark: "linear-gradient(90deg, #2d2d3a, #282835, #222230, #282835, #2d2d3a)",
      neon: "linear-gradient(90deg, #2d1f3d, #352545, #3d2045, #352545, #2d1f3d)",
      aurora:
        "linear-gradient(90deg, #050a0f, #081018, #0a1419, #0f231e, #143228, #1e4632, #143228, #0f231e, #0a1419, #081018, #050a0f)",
      ocean:
        "linear-gradient(90deg, #0f2a3a, #123040, #1a3a4a, #123540, #0f2a3a)",
      sunset:
        "linear-gradient(90deg, #3a1f2a, #3a2530, #3a2a3a, #302535, #3a1f2a)",
      forest:
        "linear-gradient(90deg, #1a2a25, #1c3028, #1f3a2a, #1c3530, #1a2a25)",
      liquid_glass: "rgba(255, 255, 255, 0.07)",
      retro: "#FFE5EC",
    };
    return backgrounds[theme] || backgrounds.original;
  }

  function getThemeBackgroundSize(theme: string): string {
    if (theme === "original") return "100% 100%";
    return "400% 100%";
  }

  function getThemeBackgroundPosition(theme: string): string {
    if (theme === "original") return "0% 0%";
    return "0% 50%";
  }

  function getThemeBackdropFilter(theme: string): string {
    const filters: Record<string, string> = {
      original: "none",
      ios26: "none",
      dark: "none",
      neon: "none",
      aurora: "none",
      ocean: "none",
      sunset: "none",
      forest: "none",
      liquid_glass: "blur(20px) saturate(150%) contrast(110%)",
      retro: "none",
    };
    return filters[theme] || filters.original;
  }

  function getThemeBorder(theme: string): string {
    const borders: Record<string, string> = {
      original: "none",
      ios26: "none",
      dark: "none",
      neon: "none",
      aurora: "none",
      ocean: "none",
      sunset: "none",
      forest: "none",
      liquid_glass: "none",
      retro: "3.5px solid #1a1a1a",
    };
    return borders[theme] || borders.original;
  }

  function getDynamicBorderRadius(currentHeight: number): string {
    const minHeight = 28;
    const maxHeight = 160;
    const minRadius = 24;
    const maxRadius =
      currentTheme === "liquid_glass"
        ? 24
        : appSettings.expanded_corner_radius || 42;

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
    if (isHidden) {
      return "none";
    }

    const shadows: Record<string, string> = {
      original: "none",
      ios26: "none",
      dark: "none",
      neon: "none",
      aurora: "none",
      ocean: "none",
      sunset: "none",
      forest: "none",
      liquid_glass: "none",
      retro: expanded ? "6px 6px 0 #1a1a1a" : "none",
    };
    return shadows[theme] || shadows.original;
  }

  // ===== 新增：应用设置 =====
  let appSettings = $state({
    auto_hide: true,
    show_spectrum: true,
    enable_animations: true,
    reduce_animations: false,
    show_debug_info: false,
    window_opacity: 255,
    hide_settings_button: false,
    hide_monitor_selector: false,
    hide_floating_window: false,
    expanded_corner_radius: 16,
    always_show_top_bar: true,
    always_on_top: true,
    player_weights: {
      netease: 50,
      spotify: 50,
      bilibili: 50,
      qqmusic: 50,
      apple: 50,
      generic: 10,
    },
    monitor_index: 0,
    enable_hd_cover: true,
    enable_pixel_art: false,
    enable_halftone: false,
    enable_mv_playback: true,
    lock_floating_window: false,
    auto_start: false,
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

    widthSpring = spring(currentWidth, {
      stiffness: params.stiffness,
      damping: params.damping,
      precision: params.precision,
    });

    heightSpring = spring(currentHeight, {
      stiffness: params.stiffness,
      damping: params.damping,
      precision: params.precision,
    });

    contentOpacity = spring(currentOpacity, {
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
  let monitors: Array<{ name: string; index: number }> = $state([]);
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

  async function processSyncQueue() {
    if (isSyncing || !hasPendingSync) return;

    isSyncing = true;
    hasPendingSync = false;

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
        requestAnimationFrame(processSyncQueue);
      }
    }
  }

  let lastW = 0;
  let lastH = 0;

  $effect(() => {
    const currentW = $widthSpring;
    const currentH = $heightSpring;

    const syncThreshold = highFrameRateMode ? 0.5 : 0.8;

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
      const delay = appSettings.enable_animations ? 5000 : 3000;
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
        await invoke("close_floating_window");
        isFloatingWindowOpen = false;
      } else {
        await invoke("open_floating_window");
        isFloatingWindowOpen = true;
      }
    } catch (error) {
      logger.error("切换悬浮窗失败:", error);
    }
  }

  async function extractDominantColor(imgSrc: string) {
    if (!imgSrc) {
      accentColor = currentColor;
      secondaryColor = currentColor;
      return;
    }

    try {
      const img = new Image();
      img.crossOrigin = "Anonymous";
      img.src = imgSrc;

      await new Promise((resolve, reject) => {
        img.onload = resolve;
        img.onerror = reject;
      });

      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d")!;
      canvas.width = 24;
      canvas.height = 24;
      ctx.drawImage(img, 0, 0, 24, 24);

      const data = ctx.getImageData(0, 0, 24, 24).data;

      const colorMap: Map<string, number> = new Map();

      for (let i = 0; i < data.length; i += 16) {
        const r = Math.floor(data[i] / 16) * 16;
        const g = Math.floor(data[i + 1] / 16) * 16;
        const b = Math.floor(data[i + 2] / 16) * 16;
        const brightness = (r + g + b) / 3;

        if (brightness > 30 && brightness < 240) {
          const key = `${r},${g},${b}`;
          colorMap.set(key, (colorMap.get(key) || 0) + 1);
        }
      }

      const sortedColors = [...colorMap.entries()].sort((a, b) => b[1] - a[1]);

      if (sortedColors.length >= 2) {
        const [mainKey] = sortedColors[0];
        const [secondKey] = sortedColors[1];
        const [r1, g1, b1] = mainKey.split(",").map(Number);
        const [r2, g2, b2] = secondKey.split(",").map(Number);

        const clamp = (v: number) => Math.max(80, Math.min(255, v));
        accentColor = `rgb(${clamp(r1)},${clamp(g1)},${clamp(b1)})`;
        secondaryColor = `rgb(${clamp(r2)},${clamp(g2)},${clamp(b2)})`;
      } else if (sortedColors.length === 1) {
        const [mainKey] = sortedColors[0];
        const [r, g, b] = mainKey.split(",").map(Number);
        const clamp = (v: number) => Math.max(80, Math.min(255, v));
        accentColor = `rgb(${clamp(r)},${clamp(g)},${clamp(b)})`;
        secondaryColor = accentColor;
      } else {
        accentColor = currentColor;
        secondaryColor = currentColor;
      }
    } catch (e) {
      console.warn("取色失败，将使用播放器默认颜色", e);
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
    const reduced = appSettings.reduce_animations;
    const animEnabled = appSettings.enable_animations;

    requestAnimationFrame(() => {
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
    try {
      await invoke("control_media", { action });
    } catch (err) {
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

  async function checkFullscreenAndHide() {
    if (!autoHideEnabled || !appSettings.auto_hide) return;

    try {
      const allMonitors = await availableMonitors();

      let monitorX = 0;
      let monitorY = 0;
      let monitorWidth = 0;
      let monitorHeight = 0;

      if (allMonitors.length > 0 && currentMonitorIndex < allMonitors.length) {
        const targetMonitor = allMonitors[currentMonitorIndex];
        monitorX = targetMonitor.position.x;
        monitorY = targetMonitor.position.y;
        monitorWidth = targetMonitor.size.width;
        monitorHeight = targetMonitor.size.height;
      } else if (allMonitors.length > 0) {
        const targetMonitor = allMonitors[0];
        monitorX = targetMonitor.position.x;
        monitorY = targetMonitor.position.y;
        monitorWidth = targetMonitor.size.width;
        monitorHeight = targetMonitor.size.height;
      }

      const isFullscreen = await invoke<boolean>("check_fullscreen_app", {
        monitorX,
        monitorY,
        monitorWidth,
        monitorHeight,
      });

      if (isFullscreen !== isFullscreenApp) {
        isFullscreenApp = isFullscreen;
        console.log(
          "[全屏检测] 状态变化:",
          isFullscreen ? "检测到全屏应用" : "全屏应用已关闭",
          "显示器:",
          `X:${monitorX}, Y:${monitorY}, W:${monitorWidth}, H:${monitorHeight}`,
        );

        if (isFullscreen) {
          hideWindowToTop();
        } else {
          showWindow();
        }
      }
    } catch (error) {
      console.error("[全屏检测] 失败:", error);
    }
  }

  async function hideWindowToTop() {
    if (!appSettings.auto_hide) return;

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
    if (!autoHideEnabled || !appSettings.auto_hide || !isFullscreenApp) return;

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
        const savedSettings = await invoke<any>("get_settings");
        await invoke("save_settings", {
          settings: {
            ...savedSettings,
            monitor_index: index,
          },
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
    const idx = appSettings.monitor_index;
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
    if (appSettings.show_debug_info) {
      startDebugFps();
    } else {
      stopDebugFps();
    }
  });

  onMount(() => {
    (async () => {
      console.log("[App.svelte] onMount 开始监听事件");

      try {
        const savedSettings = await invoke<any>("get_settings");
        const appWindow = getCurrentWindow();
        await appWindow.setAlwaysOnTop(savedSettings.always_on_top ?? true);
        console.log("[置顶设置] 已应用:", savedSettings.always_on_top);
      } catch (error) {
        console.error("[置顶设置] 读取失败:", error);
      }

      try {
        const loadedSettings = await invoke<any>("get_settings");
        appSettings = {
          auto_hide: loadedSettings.auto_hide ?? true,
          show_spectrum: loadedSettings.show_spectrum ?? true,
          enable_animations: loadedSettings.enable_animations ?? true,
          reduce_animations: loadedSettings.reduce_animations ?? false,
          show_debug_info: loadedSettings.show_debug_info ?? false,
          window_opacity: loadedSettings.window_opacity ?? 255,
          hide_settings_button: loadedSettings.hide_settings_button ?? false,
          hide_monitor_selector: loadedSettings.hide_monitor_selector ?? false,
          hide_floating_window: loadedSettings.hide_floating_window ?? false,
          expanded_corner_radius: loadedSettings.expanded_corner_radius ?? 16,
          always_show_top_bar: loadedSettings.always_show_top_bar ?? true,
        };
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

            if (s.island_theme) {
              currentTheme = s.island_theme;
            }

            if (s.monitor_index !== undefined) {
              currentMonitorIndex = s.monitor_index;
            }

            if (!appSettings.auto_hide && isHidden) {
              showWindow();
            }
          }
        },
      );

      const unlistenSettingsChanged = await eventManager.on(
        Events.SETTINGS_CHANGED,
        (settingName: any) => {
          console.log("[设置] 单项变更:", settingName);

          if (settingName === "monitor_index") {
            // 先从后端获取最新的显示器索引
            invoke<number>("get_current_monitor_index")
              .then((idx: number) => {
                currentMonitorIndex = idx;
                if (monitors[idx]) {
                  moveToMonitor(monitors[idx]).catch(console.error);
                }
              })
              .catch(console.error);
          } else if (settingName === "island_theme") {
            currentTheme = appSettings.island_theme;
          } else if (settingName === "always_on_top") {
            // 由后端处理置顶
          } else {
            invoke("get_settings")
              .then((s: any) => {
                if (s) {
                  appSettings = { ...appSettings, ...s };
                }
              })
              .catch(console.error);
          }
        },
      );

      const unlistenCornerRadiusChanged = await eventManager.on(
        Events.CORNER_RADIUS_CHANGED,
        (radius: any) => {
          console.log("[设置] 圆角变更:", radius);
          appSettings.expanded_corner_radius = radius;
        },
      );

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
          };
        });

        const savedSettings = await invoke<any>("get_settings");
        const savedMonitorIndex = savedSettings.monitor_index ?? 0;

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

      const unlistenTheme = await eventManager.on(
        Events.THEME_CHANGED,
        ({ islandTheme }: any) => {
          currentTheme = islandTheme || "original";
          console.log("[主题切换] 切换到:", currentTheme);
        },
      );

      try {
        const savedSettings = await invoke<any>("get_settings");
        currentTheme = savedSettings.island_theme || "original";
        console.log("[主题加载] 从设置加载主题:", currentTheme);
      } catch (e) {
        console.error("[主题加载] 失败:", e);
      }

      let cleanup: (() => void) | undefined;
      let lastSongKey: string | null = null;

      const unlistenMediaUpdate = await onMediaUpdate((data: any) => {
        if (data.source) currentSource = data.source;
        isPlaying = data.is_playing || false;

        const currentSongKey = `${data.title || ""}-${data.artist || ""}`;
        const songChanged = lastSongKey !== currentSongKey;

        if (songChanged) {
          maxBackendPosition = 0;
          lastSongKey = currentSongKey;
        }

        const newPosition = data.position_ms || 0;

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

          if (data.duration_ms && data.duration_ms > 1000) {
            durationMs = data.duration_ms;
            console.log("[时长] ✓ 使用 SMTC 提供的有效时长:", durationMs, "ms");
          } else {
            const songName = data.title || trackTitle;
            const artistName = data.artist || artistName;

            if (
              songName &&
              songName !== "未知曲目" &&
              artistName &&
              artistName !== "未知艺术家"
            ) {
              invoke("get_netease_song_info_cmd", {
                songName,
                artist: artistName,
              })
                .then((songInfo: any) => {
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
                      songInfo.album_pic &&
                      (!rawCoverUrl || rawCoverUrl === "")
                    ) {
                      const highQualityPic = songInfo.album_pic.replace(
                        /(\d+)x(\d+)\.jpg/,
                        "1024y1024.jpg",
                      );
                      console.log(
                        "[网易云 API] ✓ 获取专辑图片:",
                        highQualityPic,
                      );
                    }
                    if (songInfo.mv_id && songInfo.mv_id > 0) {
                      console.log(
                        "[网易云 API] ✓ 发现 MV，ID:",
                        songInfo.mv_id,
                      );
                      if (songInfo.mv_url) {
                        console.log(
                          "[网易云 API] ✓ MV 播放链接:",
                          songInfo.mv_url,
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
          data.album_art ||
          data.thumbnail ||
          data.cover_url ||
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

      const unlistenAudioSpectrum = await onAudioSpectrum(
        ({ bands, bands_expanded }: any) => {
          if (!isPlaying) return;

          const sensitivity = 1.2;

          targetSpectrumHeights = bands.map((val: number, i: number) => {
            let compressedVal = Math.min(1.0, val * sensitivity * 1.0);
            let mapped =
              compressedVal * baseCollapsedHeight * iOSCollapsedEnvelope[i];
            return Math.max(2, mapped);
          });

          targetSpectrumHeightsExpanded = bands_expanded.map(
            (val: number, i: number) => {
              let compressedVal = Math.min(1.0, val * sensitivity);
              let mapped =
                compressedVal * baseExpandedHeight * iOSExpandedEnvelope[i];
              return Math.max(2, mapped);
            },
          );

          if (!spectrumTimer && isPlaying && appSettings.show_spectrum) {
            setTimeout(() => startSpectrumAnimation(), 50);
          }
        },
      );

      cleanup = unlistenMediaUpdate;

      return () => {
        if (cleanup) cleanup();
        stopAutoClose();
        unlistenTheme();
        unlistenSettings();
        unlistenSettingsChanged();
        unlistenCornerRadiusChanged();
        unlistenFloatingWindowClosed();
        unlistenAudioSpectrum();
        stopDebugFps();
      };
    })();
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
    fullscreenCheckInterval = setInterval(checkFullscreenAndHide, 3000);
    checkFullscreenAndHide();

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
      if (fullscreenCheckInterval) {
        clearInterval(fullscreenCheckInterval);
      }
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
    class:shadow-2xl={currentTheme === "original"}
    class:island-hidden={isHidden && !isMouseAtTop}
    class:island-drop-animation={isMouseAtTop && isHidden}
    class:island-visible-edge={isHidden && isMouseAtTop}
    class:theme-liquid_glass={currentTheme === "liquid_glass"}
    class:theme-retro={currentTheme === "retro"}
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
    {#if appSettings.show_debug_info}
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
            {#if artworkUrl}
              {#key flipKey}
                <div
                  class="w-5 h-5 rounded overflow-hidden flex-shrink-0 select-none cursor-pointer"
                  data-stop-toggle
                  onclick={(e) => {
                    e.stopPropagation();
                    openCurrentPlayer();
                  }}
                >
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
                </div>
              {/key}
            {:else}
              <div
                class="w-5 h-5 rounded overflow-hidden flex-shrink-0 select-none"
              >
                <img
                  src={currentIcon}
                  alt=""
                  class="w-full h-full object-cover"
                />
              </div>
            {/if}

            {#if appSettings.show_spectrum}
              <div
                class="spectrum-container"
                style="--accent-color: {accentColor}; --secondary-color: {secondaryColor};"
              >
                {#each Array.from({ length: 5 }) as _, i}
                  <div class="spectrum-bar" bind:this={collapsedBars[i]}></div>
                {/each}
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
            {#if artworkUrl}
              {#key flipKey}
                <div
                  class="w-[52px] h-[52px] rounded-[12px] overflow-hidden shadow-2xl ring-1 ring-white/10 flex-shrink-0 cursor-pointer select-none transition-all duration-300 hover:scale-105 hover:shadow-xl"
                  data-stop-toggle
                  onclick={(e) => {
                    e.stopPropagation();
                    openCurrentPlayer();
                  }}
                >
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
                </div>
              {/key}
            {/if}

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

            {#if appSettings.show_spectrum}
              <div
                class="spectrum-container-expanded"
                style="--accent-color: {accentColor}; --secondary-color: {secondaryColor};"
              >
                {#each Array.from({ length: 40 }) as _, i}
                  <div
                    class="spectrum-bar-expanded"
                    bind:this={expandedBars[i]}
                  ></div>
                {/each}
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
              transform: translate3d(0, -2px, 0);
              backface-visibility: hidden;
              perspective: 1000px;
            "
            >
              <SkipBack
                size={22}
                fill="currentColor"
                class="text-white/90 hover:scale-110 active:scale-90 transition-all duration-300 relative z-50 cursor-pointer media-button"
                data-stop-toggle
                onclick={(e) => handleMediaAction("prev", e)}
              />
              {#if isPlaying}
                <Pause
                  size={32}
                  fill="currentColor"
                  class="text-white hover:scale-110 active:scale-95 transition-all duration-300 relative z-50 cursor-pointer media-button"
                  data-stop-toggle
                  onclick={(e) => handleMediaAction("play_pause", e)}
                />
              {:else}
                <Play
                  size={32}
                  fill="currentColor"
                  class="text-white hover:scale-110 active:scale-95 transition-all duration-300 relative z-50 cursor-pointer media-button"
                  data-stop-toggle
                  onclick={(e) => handleMediaAction("play_pause", e)}
                />
              {/if}
              <SkipForward
                size={22}
                fill="currentColor"
                class="text-white/90 hover:scale-110 active:scale-90 transition-all duration-300 relative z-50 cursor-pointer media-button"
                data-stop-toggle
                onclick={(e) => handleMediaAction("next", e)}
              />
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

  /* ========== iOS灵动岛风格频谱 ========== */
  .spectrum-container {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1px;
    height: 20px;
  }

  .collapsed-content .spectrum-container {
    height: 22px;
    overflow: hidden;
  }

  .spectrum-bar {
    width: 2px;
    height: 1px;
    border-radius: 1px;
    background: linear-gradient(
      to top,
      var(--accent-color, #fff),
      var(--secondary-color, #888)
    );
    background-clip: content-box;
    opacity: 0.95;
    will-change: transform;
    transform: scaleY(0.5) translateZ(0);
    transform-origin: center;
    backface-visibility: hidden;
  }

  .collapsed-content .spectrum-bar {
    transform-origin: center;
  }

  .spectrum-wrapper {
    margin-top: auto;
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: flex-end;
    height: 30px;
    overflow: visible;
  }

  .spectrum-container-expanded {
    display: flex;
    align-items: center !important;
    justify-content: flex-end;
    gap: 1px;
    height: 24px;
    width: auto;
    flex-shrink: 0;
  }

  .spectrum-bar-expanded {
    width: 2px;
    height: 1px;
    border-radius: 2px;
    background: linear-gradient(
      to top,
      var(--secondary-color, #888),
      var(--accent-color, #fff)
    );
    background-clip: content-box;
    opacity: 0.95;
    will-change: transform;
    transform: scaleY(0.5) translateZ(0);
    transform-origin: center;
    backface-visibility: hidden;
  }

  /* 显示器选择菜单动画 */
  .monitor-menu.menu-open {
    animation: menuSlideIn 0.3s cubic-bezier(0.4, 0, 0.2, 1) forwards;
    will-change: transform, opacity;
    transform: translateZ(0);
  }

  @keyframes menuSlideIn {
    0% {
      opacity: 0;
      transform: translateY(-8px) scale(0.95);
    }
    100% {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .monitor-menu button:hover {
    transform: translateX(4px) translateZ(0);
    transition: transform 0.15s ease-out;
  }

  .monitor-menu button:active {
    transform: scale(0.98) translateZ(0);
    transition: transform 0.1s ease-out;
  }

  .monitor-menu:not(.menu-open) {
    animation: menuSlideOut 0.2s cubic-bezier(0.4, 0, 0.2, 1) forwards;
    will-change: transform, opacity;
  }

  @keyframes menuSlideOut {
    0% {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
    100% {
      opacity: 0;
      transform: translateY(-8px) scale(0.95);
    }
  }

  /* ================================================================
 * Liquid Glass — 修复圆角阴影 + 增强玻璃质感
 * ================================================================
 *
 * 关键修复：
 *   1. 所有阴影改用 filter: drop-shadow()（跟随 border-radius）
 *   2. ::before 从 position: fixed 改为 absolute（可被 overflow: hidden 裁切）
 *   3. 删除原始 display: none !important 的冲突
 * ================================================================ */

  /* ── 容器：去除 box-shadow，改用 drop-shadow ── */
  .theme-liquid_glass {
    position: relative;
    background: rgba(200, 210, 230, 0.1) !important;
    border: 0.5px solid rgba(255, 255, 255, 0.18) !important;
    overflow: hidden !important;
    mix-blend-mode: normal !important;
    mask-image: none !important;
    -webkit-mask-image: none !important;
    contain: none !important;

    /* ★ 核心修复：drop-shadow 跟随圆角轮廓 */
    filter: drop-shadow(0 8px 24px rgba(0, 0, 0, 0.45))
      drop-shadow(0 2px 6px rgba(0, 0, 0, 0.2)) !important;

    transition:
      filter 0.3s ease,
      background 0.3s ease,
      border-color 0.3s ease,
      transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1) !important;
  }

  .theme-liquid_glass:hover {
    background: rgba(210, 220, 240, 0.16) !important;
    border-color: rgba(255, 255, 255, 0.28) !important;
    filter: drop-shadow(0 12px 32px rgba(0, 0, 0, 0.5))
      drop-shadow(0 4px 10px rgba(0, 0, 0, 0.25)) !important;
  }

  /* ── ::after：内侧高光层 ── */
  .theme-liquid_glass::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: linear-gradient(
      135deg,
      rgba(255, 255, 255, 0.2) 0%,
      rgba(255, 255, 255, 0.05) 30%,
      transparent 55%,
      rgba(0, 0, 0, 0.03) 100%
    );
    pointer-events: none;
    z-index: 0;
  }

  /* ── ::before：内侧明暗 + 噪点纹理 ── */
  /* 注意：这里必须用 !important 覆盖原始的 display: none !important */
  .theme-liquid_glass::before {
    content: "" !important;
    display: block !important;
    position: absolute !important;
    inset: 0 !important;
    border-radius: inherit !important;
    background: linear-gradient(
        180deg,
        rgba(255, 255, 255, 0.12) 0%,
        transparent 35%,
        transparent 65%,
        rgba(0, 0, 0, 0.08) 100%
      ),
      url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.035'/%3E%3C/svg%3E") !important;
    background-size:
      100% 100%,
      150px 150px !important;
    pointer-events: none !important;
    z-index: 1 !important;
    mix-blend-mode: overlay !important;
    opacity: 0.8 !important;
  }

  /* ── 文字：微发光 ── */
  .theme-liquid_glass :global(.dynamic-glass-text) {
    color: rgba(255, 255, 255, 0.95) !important;
    text-shadow:
      0 0 20px rgba(255, 255, 255, 0.12),
      0 1px 2px rgba(0, 0, 0, 0.2) !important;
  }

  .theme-liquid_glass :global(.dynamic-glass-text-secondary) {
    color: rgba(255, 255, 255, 0.6) !important;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2) !important;
  }

  .theme-liquid_glass :global(.time-display span) {
    color: rgba(255, 255, 255, 0.85) !important;
    text-shadow: 0 0 12px rgba(255, 255, 255, 0.08) !important;
  }

  /* ── 频谱条：使用经典黑主题默认样式 ── */

  /* ── 进度条：半透明白光 ── */
  .theme-liquid_glass :global(.progress-bar) {
    background: rgba(255, 255, 255, 0.07) !important;
    border: 0.5px solid rgba(255, 255, 255, 0.08) !important;
    border-radius: 999px !important;
    height: 3px !important;
  }

  .theme-liquid_glass :global(.progress-fill) {
    background: linear-gradient(
      90deg,
      rgba(255, 255, 255, 0.35),
      rgba(255, 255, 255, 0.75)
    ) !important;
    border-radius: 999px !important;
    box-shadow: 0 0 6px rgba(255, 255, 255, 0.15) !important;
  }

  .theme-liquid_glass :global(.text-white\/60) {
    color: rgba(255, 255, 255, 0.4) !important;
  }

  /* ── 按钮：毛玻璃胶囊 ── */
  .theme-liquid_glass :global(.media-button) {
    color: rgba(255, 255, 255, 0.8) !important;
    transition: all 0.25s ease !important;
  }

  .theme-liquid_glass :global(.media-button:hover) {
    color: rgba(255, 255, 255, 1) !important;
    filter: drop-shadow(0 0 6px rgba(255, 255, 255, 0.25)) !important;
  }

  .theme-liquid_glass :global(button) {
    background: rgba(255, 255, 255, 0.07) !important;
    border: 0.5px solid rgba(255, 255, 255, 0.12) !important;
    color: rgba(255, 255, 255, 0.85) !important;
    border-radius: 10px !important;
    backdrop-filter: blur(12px) !important;
    -webkit-backdrop-filter: blur(12px) !important;
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.1),
      0 2px 6px rgba(0, 0, 0, 0.15) !important;
    transition: all 0.2s ease !important;
  }

  .theme-liquid_glass :global(button:hover) {
    background: rgba(255, 255, 255, 0.14) !important;
    border-color: rgba(255, 255, 255, 0.22) !important;
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.15),
      0 4px 12px rgba(0, 0, 0, 0.2) !important;
  }

  .theme-liquid_glass :global(button:active) {
    background: rgba(255, 255, 255, 0.04) !important;
    box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.15) !important;
  }

  /* ── 专辑封面 ── */
  .theme-liquid_glass :global(.w-16) {
    border: 0.5px solid rgba(255, 255, 255, 0.12) !important;
    border-radius: 14px !important;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3) !important;
  }

  .theme-liquid_glass :global(.w-16:hover) {
    box-shadow:
      0 6px 24px rgba(0, 0, 0, 0.35),
      0 0 20px rgba(255, 255, 255, 0.04) !important;
  }

  .theme-liquid_glass :global(.collapsed-content .w-5) {
    border-radius: 6px !important;
  }

  /* ── 调试面板 ── */
  .theme-liquid_glass :global(.debug-overlay) {
    background: rgba(0, 0, 0, 0.45) !important;
    border: 0.5px solid rgba(255, 255, 255, 0.1) !important;
    border-radius: 10px !important;
    backdrop-filter: blur(16px) !important;
    -webkit-backdrop-filter: blur(16px) !important;
    color: rgba(255, 255, 255, 0.75) !important;
  }

  .theme-liquid_glass::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: linear-gradient(
      135deg,
      rgba(255, 255, 255, 0.2) 0%,
      transparent 40%,
      transparent 60%,
      rgba(0, 0, 0, 0.05) 100%
    );
    pointer-events: none;
    z-index: 0;
  }

  .theme-liquid_glass::before {
    display: none !important;
  }

  /* ================================================================
   * ===== RETRO POP ART — 复古波普卡通风格（全面改造） =====
   * ================================================================
   *
   * 灵感来源：60年代波普艺术 × 90年代漫画书 × 复古卡通
   *
   * 特色：
   *   - 暖米色背景 + 半色调网点纹理
   *   - 粗黑描边 + 偏移投影（漫画书风格）
   *   - 彩虹渐变频谱条
   *   - 波普风格进度条
   *   - 弹跳式展开/收起动画
   *   - 按钮悬浮波普高光
   * ================================================================ */

  /* ── 容器主体 ── */
  .theme-retro {
    position: relative;
    background: #ffe5ec !important;
    border: 3px solid #1a1a1a !important;
    border-radius: 18px !important;
    overflow: hidden !important;
    transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1) !important;
    /* 修复边框圆角渲染问题 */
    -webkit-mask-image: -webkit-radial-gradient(white, black);
    mask-image: -webkit-radial-gradient(white, black);
  }

  /* ── 半色调网点纹理叠加层 ── */
  .theme-retro::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background-image: radial-gradient(circle, #d4a0b0 1px, transparent 1px);
    background-size: 5px 5px;
    opacity: 0.13;
    pointer-events: none;
    z-index: 1;
  }

  /* ── 收起态文字 ── */
  .theme-retro :global(.dynamic-glass-text) {
    color: #1a1a1a !important;
    text-shadow: none !important;
    font-weight: 900 !important;
    letter-spacing: 0.02em !important;
  }

  .theme-retro :global(.dynamic-glass-text-secondary) {
    color: #555 !important;
    text-shadow: none !important;
    font-weight: 600 !important;
  }

  /* ── 时间显示 ── */
  .theme-retro :global(.time-display span) {
    color: #1a1a1a !important;
    font-weight: 900 !important;
    font-size: 14px !important;
    letter-spacing: 0.08em !important;
  }

  /* ── 收起态频谱条：使用经典黑主题默认样式 ── */

  /* ── 展开态频谱条：使用经典黑主题默认样式 ── */

  /* ── 进度条：波普漫画风格 ── */
  .theme-retro :global(.progress-bar) {
    background: #fff !important;
    border: 2.5px solid #1a1a1a !important;
    border-radius: 999px !important;
    height: 7px !important;
    overflow: hidden !important;
    box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.15) !important;
  }

  .theme-retro :global(.progress-fill) {
    background: linear-gradient(
      90deg,
      #ffd700,
      #ff6b6b,
      #ff1493,
      #5c7cfa
    ) !important;
    border-radius: 999px !important;
    transition: width 0.15s linear !important;
  }

  /* ── 进度条时间文字 ── */
  .theme-retro :global(.text-white\/60) {
    color: #777 !important;
    font-weight: 800 !important;
    font-size: 10px !important;
    letter-spacing: 0.04em !important;
  }

  /* ── 播放控制按钮 ── */
  .theme-retro :global(.media-button) {
    color: #1a1a1a !important;
    transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1) !important;
    filter: none !important;
  }

  .theme-retro :global(.media-button:hover) {
    color: #ff1493 !important;
    transform: scale(1.2) translateZ(0) !important;
    filter: drop-shadow(2px 2px 0 rgba(0, 0, 0, 0.2)) !important;
  }

  .theme-retro :global(.media-button:active) {
    transform: scale(0.88) translateZ(0) !important;
  }

  /* ── 悬浮窗按钮 ── */
  .theme-retro :global(button) {
    background: #ffd700 !important;
    border: 2.5px solid #1a1a1a !important;
    color: #1a1a1a !important;
    border-radius: 10px !important;
    box-shadow: 2px 2px 0 #1a1a1a !important;
    transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1) !important;
    font-weight: 800 !important;
  }

  .theme-retro :global(button:hover) {
    background: #4ecdc4 !important;
    transform: translate(-1px, -1px) translateZ(0) !important;
    box-shadow: 3px 3px 0 #1a1a1a !important;
  }

  .theme-retro :global(button:active) {
    transform: translate(2px, 2px) translateZ(0) !important;
    box-shadow: 0 0 0 #1a1a1a !important;
  }

  /* ── 展开态内容面板 ── */
  .theme-retro :global(.expanded-content) {
    padding: 16px 20px !important;
    border-radius: 18px !important;
  }

  /* ── 专辑封面：波普风格描边 ── */
  .theme-retro :global(.w-16) {
    width: 52px !important;
    height: 52px !important;
    border: 3px solid #1a1a1a !important;
    border-radius: 14px !important;
    overflow: hidden !important;
  }

  /* 展开态专辑封面阴影 - 已移除 */

  .theme-retro :global(.w-16:hover) {
    transform: scale(1.06) rotate(-2deg) translateZ(0) !important;
  }

  /* ── 收起态封面 ── */
  .theme-retro :global(.collapsed-content .w-5) {
    border: 2px solid #1a1a1a !important;
    border-radius: 4px !important;
    box-shadow: none !important;
  }

  /* ── 展开态弹跳入场动画 ── */
  .theme-retro :global(.expanded-content) {
    transition: none !important;
  }

  .theme-retro :global(.expanded-content.is-visible) {
    animation: retro-pop-in 0.45s cubic-bezier(0.34, 1.56, 0.64, 1) forwards !important;
  }

  @keyframes retro-pop-in {
    0% {
      opacity: 0;
      transform: scale(0.5) translateY(20px) translateZ(0);
      filter: blur(4px);
    }
    50% {
      opacity: 0.85;
      transform: scale(1.04) translateY(-4px) translateZ(0);
      filter: blur(0);
    }
    70% {
      transform: scale(0.98) translateY(2px) translateZ(0);
    }
    100% {
      opacity: 1;
      transform: scale(1) translateY(0) translateZ(0);
      filter: blur(0);
    }
  }

  /* ── 收起态弹跳退出动画 ── */
  .theme-retro :global(.collapsed-content.is-hidden) {
    animation: retro-pop-out 0.3s cubic-bezier(0.55, 0, 1, 0.45) forwards !important;
  }

  @keyframes retro-pop-out {
    0% {
      opacity: 1;
      transform: scale(1) translateZ(0);
    }
    100% {
      opacity: 0;
      transform: scale(0.7) translateY(-15px) translateZ(0);
    }
  }

  /* ── 收起态内容入场 ── */
  .theme-retro :global(.collapsed-content) {
    transition: none !important;
    box-shadow: none !important;
  }

  .theme-retro :global(.collapsed-content:not(.is-hidden)) {
    animation: retro-collapsed-in 0.4s cubic-bezier(0.34, 1.56, 0.64, 1)
      forwards !important;
    box-shadow: none !important;
  }

  /* 收起态所有元素无阴影 */
  .theme-retro :global(.collapsed-content *) {
    box-shadow: none !important;
  }

  @keyframes retro-collapsed-in {
    0% {
      opacity: 0;
      transform: scale(0.85) translateZ(0);
    }
    60% {
      transform: scale(1.05) translateZ(0);
    }
    100% {
      opacity: 1;
      transform: scale(1) translateZ(0);
    }
  }

  /* ── 按钮波普弹跳入场 ── */
  .theme-retro :global(.expanded-content.is-visible .media-button) {
    animation: retro-btn-bounce 0.4s cubic-bezier(0.34, 1.56, 0.64, 1) forwards !important;
  }

  .theme-retro
    :global(.expanded-content.is-visible .media-button:nth-child(1)) {
    animation-delay: 0.1s !important;
  }

  .theme-retro
    :global(.expanded-content.is-visible .media-button:nth-child(2)) {
    animation-delay: 0.18s !important;
  }

  .theme-retro
    :global(.expanded-content.is-visible .media-button:nth-child(3)) {
    animation-delay: 0.26s !important;
  }

  @keyframes retro-btn-bounce {
    0% {
      opacity: 0;
      transform: translateY(-30px) scale(0.3) rotate(-15deg) translateZ(0);
    }
    60% {
      opacity: 1;
      transform: translateY(4px) scale(1.1) rotate(3deg) translateZ(0);
    }
    80% {
      transform: translateY(-2px) scale(0.97) rotate(-1deg) translateZ(0);
    }
    100% {
      opacity: 1;
      transform: translateY(0) scale(1) rotate(0deg) translateZ(0);
    }
  }

  /* ── 封面翻转进入：复古旋转 ── */
  .theme-retro :global(.flip-enter) {
    animation: retro-flip 0.6s cubic-bezier(0.34, 1.56, 0.64, 1) !important;
  }

  @keyframes retro-flip {
    0% {
      transform: perspective(600px) rotateY(-120deg) scale(0.7);
      opacity: 0;
    }
    50% {
      transform: perspective(600px) rotateY(10deg) scale(1.05);
      opacity: 0.8;
    }
    75% {
      transform: perspective(600px) rotateY(-5deg) scale(0.98);
    }
    100% {
      transform: perspective(600px) rotateY(0deg) scale(1);
      opacity: 1;
    }
  }

  /* ── 调试面板适配 ── */
  .theme-retro :global(.debug-overlay) {
    background: rgba(0, 0, 0, 0.85) !important;
    border: 2px solid #ffd700 !important;
    border-radius: 8px !important;
    color: #ffd700 !important;
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

  button.selected {
    background-color: rgba(255, 255, 255, 0.15);
  }

  .monitor-menu {
    animation: menu-slide-down 0.15s cubic-bezier(0.32, 0.72, 0, 1) forwards;
    transform-origin: top right;
    will-change: transform, opacity;
  }

  @keyframes menu-slide-down {
    from {
      opacity: 0;
      transform: translateY(-10px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  /* ========== 展开态内容布局 ========== */
  .expanded-content .media-buttons {
    gap: 6px;
    justify-content: space-between;
    align-items: center;
  }

  .expanded-content .media-button {
    width: 24px;
    height: 24px;
    font-size: 11px;
    flex-shrink: 0;
  }

  .expanded-content .media-button-group {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 20px;
    width: 130px;
    flex-shrink: 0;
  }

  .expanded-content .media-button:first-child {
    margin-left: 8px;
  }

  .expanded-content .media-button:last-child {
    margin-right: 8px;
  }

  .expanded-content {
    padding: 16px 20px !important;
    padding-top: 16px !important;
    height: 160px !important;
    gap: 12px !important;
  }

  .expanded-content .title {
    font-size: 13px;
    line-height: 1.2;
  }

  .expanded-content .artist {
    font-size: 11px;
    line-height: 1.1;
  }

  .expanded-content .progress-container {
    margin-top: 4px;
  }

  .expanded-content .progress-bar {
    height: 4px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 2px;
    overflow: hidden;
    position: relative;
  }

  .expanded-content .progress-fill {
    height: 100%;
    background: linear-gradient(
      to right,
      var(--accent-color, #fff),
      var(--secondary-color, #fff)
    );
    border-radius: 2px;
    transition: width 0.1s linear;
  }

  /* 水滴状自动显示动画 */
  .island-drop-animation {
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
      border-radius: 42% 42% 38% 38%;
    }
    100% {
      opacity: 1;
      transform: translateY(0) scale(1, 1);
      border-radius: 42px;
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

  .expanded-content.is-visible .mt-auto {
    animation: progress-fade-in 0.35s cubic-bezier(0.32, 0.72, 0, 1) 0.05s
      forwards;
    opacity: 0;
    transform: translateY(-15px);
    will-change: transform, opacity;
  }

  @keyframes button-drop-in {
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes cover-fade-in {
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

  @keyframes progress-fade-in {
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes breath {
    0%,
    100% {
      opacity: 0.2;
      transform: scaleX(0.95);
    }
    50% {
      opacity: 0.8;
      transform: scaleX(1);
    }
  }

  .breath-line {
    width: 100%;
    height: 2px;
    border-radius: 99px;
    background: linear-gradient(
      90deg,
      transparent,
      var(--accent-color),
      transparent
    );
    animation: breath 0.8s cubic-bezier(0.32, 0.72, 0, 1) infinite;
    will-change: opacity, transform;
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
  }

  .media-button-wrapper.hidden {
    visibility: hidden;
    pointer-events: none;
  }

  .media-button-wrapper.hidden > * {
    visibility: hidden;
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
  .marquee-text.marquee-active {
    animation: marquee-scroll 8s linear infinite;
    padding-right: 50px;
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
