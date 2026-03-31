<script lang="ts">
  import { onMount } from "svelte";
  import { spring } from "svelte/motion";
  import { listen } from "@tauri-apps/api/event";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
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

  // 生产环境禁用调试日志
  const isDev = import.meta.env.DEV;

  // 优化的日志系统
  const logger = {
    log: (...args: any[]) => isDev && console.log("[App]", ...args),
    error: (...args: any[]) => console.error("[App]", ...args),
    warn: (...args: any[]) => console.warn("[App]", ...args),
    debug: (...args: any[]) => isDev && console.debug("[App]", ...args),
  };

  // 性能优化的节流函数
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

  // 防抖函数
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
  let accentColor = $state<string>("#fe2c55");
  let artworkUrl = $state<string>("");
  let flipKey = $state(0);
  let trackTitle = $state<string>("");
  let artistName = $state<string>("");
  let isPlaying = $state<boolean>(false);
  let currentTimeMs = $state<number>(0);
  let durationMs = $state<number>(0);
  let currentSource = $state<string>("generic");
  let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;
  let currentTheme = $state("original");

  // ===== 主题样式辅助函数 =====
  function getThemeBackground(theme: string): string {
    const backgrounds: Record<string, string> = {
      original: "#0a0a0a",
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
    };
    return borders[theme] || borders.original;
  }

  function getThemeBoxShadow(
    theme: string,
    isHidden: boolean,
    expanded: boolean,
  ): string {
    if (isHidden) {
      return "0 2px 10px rgba(255, 255, 255, 0.1)";
    }

    const shadows: Record<string, string> = {
      original: "0 20px 60px rgba(0,0,0,0.7)",
      ios26: "0 20px 60px rgba(0, 0, 0, 0.6)",
      dark: "0 20px 60px rgba(0, 0, 0, 0.7)",
      neon: "0 20px 60px rgba(0, 0, 0, 0.7)",
      aurora: "0 20px 60px rgba(0, 0, 0, 0.8)",
      ocean: "0 20px 60px rgba(0, 0, 0, 0.7)",
      sunset: "0 20px 60px rgba(0, 0, 0, 0.7)",
      forest: "0 20px 60px rgba(0, 0, 0, 0.7)",
    };
    return shadows[theme] || shadows.original;
  }

  // ===== 新增：应用设置（从设置页面同步） =====
  let appSettings = $state({
    auto_hide: true,
    show_spectrum: true,
    enable_animations: true,
    reduce_animations: false,
    show_debug_info: false,
    window_opacity: 255,
  });

  // 全屏检测和自动隐藏相关状态
  let isFullscreenApp = $state(false);
  let isMouseAtTop = $state(false);
  let isHidden = $state(false);
  let autoHideEnabled = $state(true);

  // 显示器选择相关状态
  let showMonitorMenu = $state(false);
  let monitors: Array<{ name: string; index: number }> = $state([]);
  let currentMonitorIndex = $state(0);

  // 悬浮窗状态
  let isFloatingWindowOpen = $state(false);

  // 调试信息状态
  let fps = $state(0);
  let frameCount = 0;
  let lastFpsTime = 0;
  let debugRafId: number | null = null;

  // 当前显示的图标路径
  const currentIcon = $derived(
    platformIcons[currentSource as keyof typeof platformIcons] ||
      platformIcons.generic,
  );

  // 当前播放器颜色
  const currentColor = $derived(
    playerColors[currentSource as keyof typeof playerColors] ||
      playerColors.generic,
  );

  // 获取当前播放器配置
  const currentConfig = $derived(
    playerConfigs[currentSource as keyof typeof playerConfigs] ||
      playerConfigs.generic,
  );

  // 判定直播逻辑
  let isLive = $derived(durationMs === 0);

  // --- 模拟进度核心 (Spring) ---
  let progressSpring = spring(0, { stiffness: 0.15, damping: 0.8 });

  const precisePosition = $derived(() => {
    return currentTimeMs;
  });

  const progressPercent = $derived(
    durationMs > 0 ? (precisePosition() / durationMs) * 100 : $progressSpring,
  );

  // ========== Spring 参数 ==========
  const widthSpring = spring(160, { stiffness: 0.4, damping: 0.75 });
  const heightSpring = spring(37, { stiffness: 0.4, damping: 0.75 });
  const contentOpacity = spring(0, { stiffness: 0.3, damping: 0.8 });

  const win = getCurrentWindow();

  // ========== 窗口同步优化 ==========
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
      const targetY = Math.round(monitorAnchorY + 17 * dpr); // 向下移动 5px

      await Promise.all([
        win.setSize(new PhysicalSize(physW, physH)),
        win.setPosition(new PhysicalPosition(centerX, targetY)),
      ]);
    } catch (err) {
      error("窗口同步失败:", err);
    } finally {
      isSyncing = false;
      if (hasPendingSync) {
        requestAnimationFrame(processSyncQueue);
      }
    }
  }

  let lastW = 0;
  let lastH = 0;

  // ===== 自动收起管理（响应动画开关） =====
  function startAutoClose() {
    stopAutoClose();
    if (expanded && !hovering) {
      const delay = appSettings.enable_animations ? 5000 : 3000;
      autoCloseTimer = setTimeout(() => {
        expanded = false;
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
    if (expanded) {
      startAutoClose();
    }
  }

  async function toggleFloatingWindow() {
    try {
      if (isFloatingWindowOpen) {
        // 关闭悬浮窗
        await invoke("close_floating_window");
        isFloatingWindowOpen = false;
      } else {
        // 打开悬浮窗
        await invoke("open_floating_window");
        isFloatingWindowOpen = true;
      }
    } catch (error) {
      error("切换悬浮窗失败:", error);
    }
  }

  // 替换原本的 extractDominantColor 函数
  async function extractDominantColor(imgSrc: string) {
    if (!imgSrc) {
      accentColor = currentColor; // 如果没有图片，回退到播放器默认颜色
      return;
    }

    try {
      const img = new Image();
      // 允许跨域图片取色
      img.crossOrigin = "Anonymous";
      img.src = imgSrc;

      await new Promise((resolve, reject) => {
        img.onload = resolve;
        img.onerror = reject;
      });

      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d")!;
      canvas.width = 10;
      canvas.height = 10;
      ctx.drawImage(img, 0, 0, 10, 10);

      const data = ctx.getImageData(0, 0, 10, 10).data;
      let r = 0,
        g = 0,
        b = 0;

      for (let i = 0; i < data.length; i += 4) {
        r += data[i];
        g += data[i + 1];
        b += data[i + 2];
      }

      // 修复原本算法的 Bug：10x10画布共 100 个像素点，应除以 100 取平均值，而不是原先的 25
      const pixelCount = 100;

      // 保留你的最小亮度限制 (100)，防止专辑封面太黑导致频谱看不清
      accentColor = `rgb(${Math.max(Math.floor(r / pixelCount), 100)},${Math.max(Math.floor(g / pixelCount), 100)},${Math.max(Math.floor(b / pixelCount), 100)})`;
    } catch (e) {
      console.warn("取色失败，将使用播放器默认颜色", e);
      accentColor = currentColor; // 取色因为跨域或协议拦截失败时，使用兜底颜色
    }
  }

  // ===== 新增：监听封面变化自动取色 =====
  $effect(() => {
    if (artworkUrl) {
      // 每次 artworkUrl 更新时，自动触发取色
      extractDominantColor(artworkUrl);
    } else {
      // 如果没有封面，同步恢复为当前播放器的颜色
      accentColor = currentColor;
    }
  });

  // ===== 展开/收起动画（响应 reduce_animations + enable_animations） =====
  $effect(() => {
    const isExp = expanded;
    const isHov = hovering;
    const reduced = appSettings.reduce_animations;
    const animEnabled = appSettings.enable_animations;

    if (isExp) {
      widthSpring.set(360);
      heightSpring.set(180);
      if (!animEnabled) {
        contentOpacity.set(1);
      } else if (reduced) {
        contentOpacity.set(1);
      } else {
        setTimeout(() => contentOpacity.set(1), 50);
      }
    } else {
      widthSpring.set(isHov ? 152 : 140);
      heightSpring.set(isHov ? 35 : 32);
      contentOpacity.set(0);
    }
  });

  // --- 实时监听 Spring 动画值并同步窗口 ---
  $effect(() => {
    const currentW = $widthSpring;
    const currentH = $heightSpring;

    if (Math.abs(currentW - lastW) > 0.8 || Math.abs(currentH - lastH) > 0.8) {
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

  // ========== 媒体控制指令 ==========
  async function handleMediaAction(action: string, e: MouseEvent) {
    e.stopPropagation();
    try {
      await invoke("control_media", { action });
      // 不手动更新状态，等待后端的 media-update 事件同步
    } catch (err) {
      console.error("媒体控制失败:", err);
    }
  }

  // ========== 格式化时间 ==========
  function formatTime(ms: number): string {
    if (ms <= 0) return "00:00";
    const s = Math.floor(ms / 1000);
    const min = Math.floor(s / 60);
    const sec = s % 60;
    return `${min.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`;
  }

  // ========== 全屏检测和自动隐藏（响应 auto_hide 开关） ==========
  let fullscreenCheckInterval: ReturnType<typeof setInterval> | null = null;
  let hideTimeout: ReturnType<typeof setTimeout> | null = null;

  async function checkFullscreenAndHide() {
    // ===== 检查 auto_hide 开关 =====
    if (!autoHideEnabled || !appSettings.auto_hide) return;

    try {
      // 获取当前显示器信息
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
        // 如果索引无效，使用第一个显示器
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
    // ===== 检查 auto_hide 开关 =====
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
        console.log(
          "[自动隐藏] 显示器:",
          targetMonitor.name,
          "中心 X:",
          windowCenterX,
        );
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
        const targetY = Math.round(17 * dpr); // 向下移动 5px

        await appWindow.setPosition(
          new PhysicalPosition(Math.round(windowCenterX), targetY),
        );
        isHidden = false;
        console.log("[自动显示] 窗口已显示在顶部中间");
        console.log(
          "[自动显示] 显示器:",
          targetMonitor.name,
          "中心 X:",
          windowCenterX,
        );
      } else {
        const targetY = Math.round(17 * dpr); // 向下移动 5px
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

  // ========== 显示器选择功能 ==========
  async function switchMonitor(index: number) {
    try {
      const allMonitors = await availableMonitors();
      const targetMonitor = allMonitors[index];

      if (!targetMonitor) {
        console.error("[显示器] 未找到目标显示器");
        return;
      }

      monitorAnchorX = targetMonitor.position.x + targetMonitor.size.width / 2;
      monitorAnchorY = targetMonitor.position.y;
      cachedScreenWidth = targetMonitor.size.width;
      cachedScreenHeight = targetMonitor.size.height;

      const appWindow = getCurrentWindow();
      const currentSize = await appWindow.innerSize();

      const targetX = Math.round(monitorAnchorX - currentSize.width / 2);
      const targetY = Math.round(monitorAnchorY + 17); // 向下移动 5px

      await appWindow.setPosition(new PhysicalPosition(targetX, targetY));
      currentMonitorIndex = index;
      showMonitorMenu = false;

      // 保存显示器索引到设置
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

  function toggleMonitorMenu() {
    showMonitorMenu = !showMonitorMenu;
  }

  function closeMonitorMenu() {
    showMonitorMenu = false;
  }

  // ===== 调试信息 FPS 计算 =====
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

  // ========== onMount ==========
  onMount(() => {
    (async () => {
      console.log("[App.svelte] onMount 开始监听事件");

      // 读取设置并应用置顶状态
      try {
        const savedSettings = await invoke<any>("get_settings");
        const appWindow = getCurrentWindow();
        await appWindow.setAlwaysOnTop(savedSettings.always_on_top ?? true);
        console.log("[置顶设置] 已应用:", savedSettings.always_on_top);
      } catch (error) {
        console.error("[置顶设置] 读取失败:", error);
      }

      // ===== 加载应用设置 =====
      try {
        const loadedSettings = await invoke<any>("get_settings");
        appSettings = {
          auto_hide: loadedSettings.auto_hide ?? true,
          show_spectrum: loadedSettings.show_spectrum ?? true,
          enable_animations: loadedSettings.enable_animations ?? true,
          reduce_animations: loadedSettings.reduce_animations ?? false,
          show_debug_info: loadedSettings.show_debug_info ?? false,
          window_opacity: loadedSettings.window_opacity ?? 255,
        };
        console.log("[设置] 已加载:", appSettings);
      } catch (error) {
        console.error("[设置] 读取失败:", error);
      }

      // ===== 监听设置变更事件 =====
      const unlistenSettings = await listen(
        "settings-updated",
        (event: any) => {
          const s = event.payload;
          if (s) {
            appSettings = {
              auto_hide: s.auto_hide ?? appSettings.auto_hide,
              show_spectrum: s.show_spectrum ?? appSettings.show_spectrum,
              enable_animations:
                s.enable_animations ?? appSettings.enable_animations,
              reduce_animations:
                s.reduce_animations ?? appSettings.reduce_animations,
              show_debug_info: s.show_debug_info ?? appSettings.show_debug_info,
              window_opacity: s.window_opacity ?? appSettings.window_opacity,
            };
            console.log("[设置] 实时更新:", appSettings);

            // auto_hide 关闭时立即显示窗口
            if (!appSettings.auto_hide && isHidden) {
              showWindow();
            }
          }
        },
      );

      // 初始化显示器列表
      try {
        const allMonitors = await availableMonitors();
        monitors = allMonitors.map((m, idx) => ({
          name: m.name || `显示器 ${idx + 1}`,
          index: idx,
        }));

        // 从设置中读取上次选择的显示器索引
        const savedSettings = await invoke<any>("get_settings");
        const savedMonitorIndex = savedSettings.monitor_index ?? 0;

        // 验证索引是否有效，如果无效则使用当前显示器
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
          // 如果保存的索引无效，使用当前显示器
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

      // 监听悬浮窗关闭事件
      const unlistenFloatingWindowClosed = await listen(
        "floating-window-closed",
        () => {
          isFloatingWindowOpen = false;
          console.log("[悬浮窗] 已关闭，更新状态");
        },
      );

      // 监听导航到设置页面的事件
      const unlistenNavigate = await listen(
        "navigate-to-settings",
        async () => {
          console.log("[App.svelte] 收到 navigate-to-settings 事件");

          try {
            const { WebviewWindow } = await import(
              "@tauri-apps/api/webviewWindow"
            );
            console.log("[App.svelte] 导入 WebviewWindow 成功");

            console.log("[App.svelte] 创建设置窗口");
            const webview = new WebviewWindow("settings-window", {
              url: "/settings.html",
              title: "设置",
              width: 800,
              height: 600,
              minWidth: 600,
              minHeight: 500,
              resizable: true,
              decorations: false,
              transparent: true,
              alwaysOnTop: false,
              center: true,
            });

            webview.once("tauri://created", () => {
              console.log("[设置窗口] 窗口创建成功");
            });

            webview.once("tauri://error", (e) => {
              console.error("[设置窗口] 创建失败:", e);
            });
          } catch (error) {
            console.error("[App.svelte] 打开设置窗口失败:", error);
          }
        },
      );

      console.log("[App.svelte] 事件监听器已注册");

      // 监听主题变化
      const unlistenTheme = await listen("theme-changed", (event: any) => {
        const { islandTheme } = event.payload;
        currentTheme = islandTheme || "original";
        console.log("[主题切换] 切换到:", currentTheme);
      });

      // 从设置中加载保存的主题
      try {
        const savedSettings = await invoke<any>("get_settings");
        currentTheme = savedSettings.island_theme || "original";
        console.log("[主题加载] 从设置加载主题:", currentTheme);
      } catch (e) {
        console.error("[主题加载] 失败:", e);
      }

      // 监听 SMTC 推送
      let cleanup: (() => void) | undefined;

      const unlistenMediaUpdate = await listen("media-update", (event: any) => {
        const data = event.payload;

        if (data.source) currentSource = data.source;
        isPlaying = data.is_playing || false;
        currentTimeMs = data.position_ms || 0;
        durationMs = data.duration_ms || 0;

        if (trackTitle !== data.title) {
          trackTitle = data.title || "未知曲目";
          artistName = data.artist || "未知艺术家";

          const newCover =
            data.album_art ||
            data.thumbnail ||
            data.cover_url ||
            data.api_cover_url ||
            data.image ||
            "";

          if (newCover && newCover !== artworkUrl) {
            if (
              newCover.startsWith("data:image") ||
              newCover.startsWith("http://") ||
              newCover.startsWith("https://") ||
              newCover.startsWith("file://")
            ) {
              flipKey += 1; // 触发翻转动画
              artworkUrl = newCover;
            } else if (newCover.includes(":\\") || newCover.includes(":/")) {
              // 本地文件路径，使用 convertFileSrc 转换
              artworkUrl = convertFileSrc(newCover);
              flipKey += 1; // 触发翻转动画
            } else {
              artworkUrl = "";
            }
          } else if (!newCover) {
            artworkUrl = "";
          }

          progressSpring.set(0, { soft: true });
        }

        if (durationMs > 0) {
          progressSpring.set((currentTimeMs / durationMs) * 100);
        }
      });

      cleanup = unlistenMediaUpdate;

      return () => {
        if (cleanup) cleanup();
        stopAutoClose();
        unlistenTheme();
        unlistenNavigate();
        unlistenSettings();
        unlistenFloatingWindowClosed();
        stopDebugFps();
      };
    })();
  });

  // 全局点击事件：关闭显示器菜单
  function handleGlobalClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (showMonitorMenu && !target.closest(".relative")) {
      closeMonitorMenu();
    }
  }

  onMount(() => {
    document.addEventListener("click", handleGlobalClick);
    return () => {
      document.removeEventListener("click", handleGlobalClick);
    };
  });

  // 全屏检测和鼠标移动监听
  onMount(() => {
    // 降低全屏检测频率，从 2000ms 改为 5000ms
    fullscreenCheckInterval = setInterval(checkFullscreenAndHide, 5000);
    checkFullscreenAndHide();

    // 使用节流优化鼠标移动监听
    let mouseMoveTimeout: ReturnType<typeof setTimeout> | null = null;
    const handleMouseMoveThrottled = (e: MouseEvent) => {
      if (mouseMoveTimeout) {
        clearTimeout(mouseMoveTimeout);
      }
      mouseMoveTimeout = setTimeout(() => {
        handleMouseMove(e);
      }, 100); // 100ms 节流
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
      border-radius: {expanded ? '42px' : '22px'};
      overflow: hidden;
      display: flex;
      flex-direction: column;
      transform: scale({isPressed ? 0.96 : 1});
      transition:
        border-radius 0.6s cubic-bezier(0.25, 0.1, 0.25, 1),
        transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1),
        background 0.4s cubic-bezier(0.25, 0.1, 0.25, 1),
        backdrop-filter 0.4s cubic-bezier(0.25, 0.1, 0.25, 1),
        border 0.4s cubic-bezier(0.25, 0.1, 0.25, 1),
        box-shadow 0.3s cubic-bezier(0.25, 0.1, 0.25, 1),
        opacity 0.35s cubic-bezier(0.25, 0.1, 0.25, 1),
        filter 0.35s cubic-bezier(0.25, 0.1, 0.25, 1);
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
    <!-- 星空背景层（除默认主题外） -->
    {#if currentTheme !== "original"}
      <div class="stars-bg"></div>
    {/if}

    <!-- 调试信息覆盖层 -->
    {#if appSettings.show_debug_info}
      <div class="debug-overlay">
        <span>FPS: {fps}</span>
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
        <div
          class="h-full w-full flex items-center justify-between select-none"
        >
          {#if artworkUrl}
            {#key flipKey}
              <div
                class="w-6 h-6 rounded-md overflow-hidden flex-shrink-0 ml-[4px]! select-none"
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
              class="w-6 h-6 rounded-md overflow-hidden flex-shrink-0 ml-[4px]! select-none"
            >
              <img
                src={currentIcon}
                alt=""
                class="w-full h-full object-cover"
              />
            </div>
          {/if}

          <!-- ===== 频谱：根据 show_spectrum 开关条件渲染 ===== -->
          {#if appSettings.show_spectrum}
            <div class="flex gap-[2px] items-center h-4 mr-[4px]!">
              {#each [0.6, 1.2, 0.9, 1.5, 0.7] as h, i}
                <div
                  class="w-[2px] rounded-full"
                  class:animate-island-wave={isPlaying}
                  style="
                    background-color: {accentColor};
                    height: {h * 8}px;
                    animation-delay: {i * 0.15}s;
                  "
                ></div>
              {/each}
            </div>
          {:else}
            <div class="flex items-center h-4 mr-[4px]! gap-[3px]">
              {#if isPlaying}
                <div
                  class="w-[5px] h-[5px] rounded-full animate-pulse"
                  style="background-color: {accentColor};"
                ></div>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <!-- 展开态内容 -->
      <div
        class="expanded-content"
        class:is-visible={expanded}
        style="opacity: {$contentOpacity};"
      >
        <!-- 顶部区域：封面 + 标题 + 显示器按钮 -->
        <div class="flex items-center" style="gap: 16px; margin-bottom: 20px;">
          {#if artworkUrl}
            {#key flipKey}
              <div
                class="w-[60px] h-[60px] rounded-2xl overflow-hidden shadow-2xl ring-1 ring-white/10 flex-shrink-0 cursor-pointer select-none transition-all duration-300 hover:scale-105 hover:shadow-xl"
                data-stop-toggle
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
            <h2
              class="font-bold text-[18px] truncate text-white tracking-tight select-none leading-tight mb-1"
            >
              {trackTitle}
            </h2>
            <p
              class="text-[14px] text-white/60 truncate font-medium select-none leading-tight"
            >
              {artistName}
            </p>
          </div>

          <!-- 显示器选择按钮 -->
          <div class="relative">
            <button
              class="w-10 h-10 flex items-center justify-center rounded-2xl bg-white/10 hover:bg-white/20 relative z-50 media-button transition-all duration-300 hover:scale-110"
              style="transform: translateZ(0); backface-visibility: hidden;"
              data-stop-toggle
              onclick={(e) => {
                e.stopPropagation();
                toggleMonitorMenu();
              }}
            >
              <Monitor
                size={18}
                class="text-white/80"
                style="transform: translateZ(0); backface-visibility: hidden;"
              />
            </button>

            <!-- 显示器选择菜单 -->
            {#if showMonitorMenu}
              <div
                class="absolute right-0 top-full mt-3 w-64 bg-black/90 backdrop-blur-3xl rounded-2xl shadow-2xl border border-white/20 overflow-hidden z-[100] monitor-menu transition-all duration-300 scale-95 opacity-0"
                class:menu-open={showMonitorMenu}
                style="transform: translateZ(0);"
              >
                <div class="p-3">
                  <div
                    class="flex items-center justify-between px-2 py-1.5 mb-2"
                  >
                    <div class="flex items-center gap-2">
                      <div
                        class="w-7 h-7 rounded-lg bg-white/10 flex items-center justify-center"
                      >
                        <Monitor size={14} class="text-white/80" />
                      </div>
                      <div>
                        <span
                          class="text-sm font-semibold text-white/95 tracking-wide subpixel-antialiased"
                          style="font-smooth: always; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale;"
                        >
                          显示器选择
                        </span>
                        <div
                          class="text-xs text-white/60 mt-0.5 subpixel-antialiased"
                          style="font-smooth: always; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale;"
                        >
                          选择要显示的显示器
                        </div>
                      </div>
                    </div>
                    <div class="flex items-center gap-1.5">
                      <div
                        class="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse"
                      ></div>
                      <span
                        class="text-xs text-white/70 font-medium subpixel-antialiased"
                        style="font-smooth: always; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale;"
                      >
                        {monitors.length} 个可用
                      </span>
                    </div>
                  </div>

                  <div
                    class="h-px bg-gradient-to-r from-transparent via-white/15 to-transparent mb-3"
                  ></div>

                  <div class="space-y-1">
                    {#each monitors as monitor, idx}
                      <button
                        class="w-full text-left px-3 py-2 rounded-xl transition-all duration-300 flex items-center gap-3 group relative overflow-hidden hover:bg-white/5 active:scale-95"
                        class:selected={currentMonitorIndex === idx}
                        onclick={(e) => {
                          e.stopPropagation();
                          switchMonitor(idx);
                        }}
                      >
                        {#if currentMonitorIndex === idx}
                          <div
                            class="absolute inset-0 bg-gradient-to-br from-white/8 to-white/2"
                          ></div>
                          <div
                            class="absolute left-0 top-0 bottom-0 w-1 bg-gradient-to-b from-green-400 to-blue-400"
                          ></div>
                        {/if}

                        <div
                          class="w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 relative transition-all duration-300 group-hover:scale-105"
                          style="background: {currentMonitorIndex === idx
                            ? 'linear-gradient(135deg, rgba(255, 255, 255, 0.15), rgba(255, 255, 255, 0.05))'
                            : 'rgba(255, 255, 255, 0.08)'};"
                        >
                          <Monitor
                            size={16}
                            class="relative z-10 transition-all duration-300"
                            style="color: {currentMonitorIndex === idx
                              ? '#ffffff'
                              : 'rgba(255, 255, 255, 0.70)'};"
                          />
                        </div>

                        <div class="flex-1 min-w-0 relative z-10">
                          <div class="flex items-center gap-2">
                            <span
                              class="text-sm font-semibold truncate block transition-all duration-300 subpixel-antialiased"
                              style="color: {currentMonitorIndex === idx
                                ? '#ffffff'
                                : 'rgba(255, 255, 255, 0.95)'};
                                font-smooth: always; 
                                -webkit-font-smoothing: antialiased; 
                                -moz-osx-font-smoothing: grayscale;"
                            >
                              {monitor.name}
                            </span>
                          </div>
                          {#if currentMonitorIndex === idx}
                            <div
                              class="text-xs text-green-400/90 font-semibold mt-0.5 flex items-center gap-1 subpixel-antialiased"
                              style="font-smooth: always; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale;"
                            >
                              <div
                                class="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse"
                              ></div>
                              当前使用中
                            </div>
                          {:else}
                            <div
                              class="text-xs text-white/60 mt-0.5 subpixel-antialiased"
                              style="font-smooth: always; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale;"
                            >
                              点击切换
                            </div>
                          {/if}
                        </div>

                        {#if currentMonitorIndex === idx}
                          <div class="flex items-center gap-1 relative z-10">
                            <div
                              class="w-5 h-5 rounded-full bg-gradient-to-r from-green-400 to-blue-400 flex items-center justify-center shadow"
                            >
                              <svg
                                width="10"
                                height="8"
                                viewBox="0 0 10 8"
                                fill="none"
                                class="relative z-10"
                              >
                                <path
                                  d="M1 4L3.5 6.5L9 1"
                                  stroke="white"
                                  stroke-width="1.5"
                                  stroke-linecap="round"
                                  stroke-linejoin="round"
                                />
                              </svg>
                            </div>
                          </div>
                        {/if}
                      </button>
                    {/each}
                  </div>
                </div>
              </div>
            {/if}
          </div>
        </div>

        <!-- 中部区域：播放控制按钮 -->
        <div
          class="flex-1 flex items-center justify-between"
          style="margin-bottom: 24px;"
        >
          <Heart
            size={20}
            class="text-white/40 hover:text-red-500 transition-all duration-300 hover:scale-110 relative z-50 cursor-pointer media-button"
            style="transform: translateZ(0); backface-visibility: hidden;"
            data-stop-toggle
            onclick={(e) => {
              e.stopPropagation();
              console.log("点击了喜欢按钮");
            }}
          />

          <div
            class="flex items-center justify-center"
            style="
                width: 180px;
                gap: 40px;
                will-change: auto;
                transform: translate3d(0, 0, 0);
                backface-visibility: hidden;
                perspective: 1000px;
                flex-shrink: 0;
              "
          >
            <SkipBack
              size={26}
              fill="currentColor"
              class="text-white/90 hover:scale-110 active:scale-90 transition-all duration-300 relative z-50 cursor-pointer media-button"
              data-stop-toggle
              onclick={(e) => handleMediaAction("prev", e)}
            />
            {#if isPlaying}
              <Pause
                size={40}
                fill="currentColor"
                class="text-white hover:scale-110 active:scale-95 transition-all duration-300 relative z-50 cursor-pointer media-button"
                data-stop-toggle
                onclick={(e) => handleMediaAction("play_pause", e)}
              />
            {:else}
              <Play
                size={40}
                fill="currentColor"
                class="text-white hover:scale-110 active:scale-95 transition-all duration-300 relative z-50 cursor-pointer media-button"
                data-stop-toggle
                onclick={(e) => handleMediaAction("play_pause", e)}
              />
            {/if}
            <SkipForward
              size={26}
              fill="currentColor"
              class="text-white/90 hover:scale-110 active:scale-90 transition-all duration-300 relative z-50 cursor-pointer media-button"
              data-stop-toggle
              onclick={(e) => handleMediaAction("next", e)}
            />
          </div>

          <button
            class="w-8 h-8 flex items-center justify-center rounded-xl border border-white/10 text-white/90 hover:scale-110 active:scale-90 transition-all duration-300 relative z-50 cursor-pointer media-button hover:border-white/20"
            style="transform: translateZ(0); backface-visibility: hidden;"
            data-stop-toggle
            aria-label={isFloatingWindowOpen
              ? "Close floating window"
              : "Open floating window"}
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

        <!-- 底部区域：进度条 -->
        <div class="mt-auto" style="margin-bottom: 8px;">
          {#if currentConfig.useProgressBar && durationMs > 0}
            <div
              class="relative w-full h-[4px] bg-white/10 rounded-full overflow-hidden"
            >
              <div
                class="absolute left-0 top-0 h-full rounded-full transition-all duration-300 ease-out"
                style="width: {progressPercent}%; background-color: {currentColor};"
              ></div>
            </div>

            <div
              class="flex justify-between text-[11px] font-semibold mt-3 tracking-tight"
            >
              <span class="text-white/30">{formatTime(currentTimeMs)}</span>
              <span class="text-white/30">{formatTime(durationMs)}</span>
            </div>
          {:else}
            <div class="flex flex-col items-center gap-2">
              <div
                class="breath-line w-full h-[2px] rounded-full"
                style="--accent-color: {currentColor}"
              ></div>

              <div
                class="flex items-center gap-2 opacity-30 text-[8px] tracking-[0.2em] uppercase"
              >
                {#if isPlaying}
                  <span class="animate-pulse"
                    >● Playing on {currentConfig.name}</span
                  >
                {:else}
                  <span>Paused</span>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  /* 性能优化的 CSS 变量和基础样式 */
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

  /* 优化的动画关键帧 - 使用 GPU 加速 */
  @keyframes island-wave {
    0%,
    100% {
      transform: scaleY(0.6);
    }
    50% {
      transform: scaleY(1.8);
    }
  }

  .animate-island-wave {
    animation: island-wave 0.35s cubic-bezier(0.32, 0.72, 0, 1) infinite;
    will-change: transform;
    transform: translateZ(0); /* GPU 加速 */
  }

  /* 优化的显示器选择菜单动画 */
  .monitor-menu.menu-open {
    animation: menuSlideIn 0.3s cubic-bezier(0.4, 0, 0.2, 1) forwards;
    will-change: transform, opacity;
    transform: translateZ(0); /* GPU 加速 */
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

  /* 优化的显示器选项悬停效果 */
  .monitor-menu button:hover {
    transform: translateX(4px) translateZ(0);
    transition: transform 0.15s ease-out;
  }

  .monitor-menu button:active {
    transform: scale(0.98) translateZ(0);
    transition: transform 0.1s ease-out;
  }

  /* 优化的显示器选择菜单关闭动画 */
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

  /* 星空背景 */
  .stars-bg {
    position: absolute;
    inset: 0;
    background-image: radial-gradient(
        1.5px 1.5px at 20px 30px,
        rgba(255, 255, 255, 0.9),
        transparent
      ),
      radial-gradient(
        1px 1px at 40px 70px,
        rgba(255, 255, 255, 0.7),
        transparent
      ),
      radial-gradient(
        1.5px 1.5px at 90px 40px,
        rgba(255, 255, 255, 0.8),
        transparent
      ),
      radial-gradient(
        1px 1px at 160px 90px,
        rgba(255, 255, 255, 0.6),
        transparent
      ),
      radial-gradient(
        1.5px 1.5px at 230px 50px,
        rgba(255, 255, 255, 0.7),
        transparent
      ),
      radial-gradient(
        1px 1px at 280px 80px,
        rgba(255, 255, 255, 0.8),
        transparent
      ),
      radial-gradient(
        1px 1px at 50px 120px,
        rgba(255, 255, 255, 0.6),
        transparent
      ),
      radial-gradient(
        1.5px 1.5px at 120px 150px,
        rgba(255, 255, 255, 0.7),
        transparent
      ),
      radial-gradient(
        1px 1px at 200px 20px,
        rgba(255, 255, 255, 0.9),
        transparent
      ),
      radial-gradient(
        1px 1px at 320px 30px,
        rgba(255, 255, 255, 0.6),
        transparent
      ),
      radial-gradient(
        1.5px 1.5px at 80px 10px,
        rgba(255, 255, 255, 0.7),
        transparent
      ),
      radial-gradient(
        1px 1px at 250px 140px,
        rgba(255, 255, 255, 0.8),
        transparent
      ),
      radial-gradient(
        1px 1px at 180px 60px,
        rgba(255, 255, 255, 0.5),
        transparent
      ),
      radial-gradient(
        1.5px 1.5px at 300px 100px,
        rgba(255, 255, 255, 0.7),
        transparent
      );
    background-repeat: repeat-x;
    background-size: 380px 180px;
    opacity: 0.35;
    pointer-events: none;
    z-index: 1;
    animation: stars-twinkle 4s ease-in-out infinite;
  }

  @keyframes stars-twinkle {
    0%,
    100% {
      opacity: 0.25;
      transform: translateY(0);
    }
    50% {
      opacity: 0.45;
      transform: translateY(-1px);
    }
  }

  /* 背景流动动画 */
  @keyframes bg-flow {
    0% {
      background-position: 0% 50%;
    }
    100% {
      background-position: 400% 50%;
    }
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

  /* 显示器选择菜单选中状态 */
  button.selected {
    background-color: rgba(255, 255, 255, 0.15);
  }

  button.hoverable:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  /* 显示器选择菜单动画 */
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

  /* 隐藏状态的窗口 */
  .island-hidden {
    transition:
      transform 0.35s cubic-bezier(0.32, 0.72, 0, 1),
      opacity 0.3s cubic-bezier(0.32, 0.72, 0, 1);
    transform: translateY(-100%);
    opacity: 0;
    pointer-events: none;
    will-change: transform, opacity;
  }

  /* 隐藏时可见的顶部边缘 */
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
    padding: 20px 24px 36px 24px;

    transform: translateY(30px) scale(0.92);
    transition:
      transform 0.35s cubic-bezier(0.32, 0.72, 0, 1),
      filter 0.35s cubic-bezier(0.32, 0.72, 0, 1),
      opacity 0.35s cubic-bezier(0.32, 0.72, 0, 1);

    filter: blur(8px);
    opacity: 0;
    pointer-events: none;
    will-change: transform, opacity, filter;
    transform: translate3d(0, 0, 0);
  }

  .expanded-content.is-visible {
    transform: translateY(0) scale(1);
    filter: blur(0);
    opacity: 1;
    pointer-events: auto;
  }

  /* 收起态内容容器 */
  .collapsed-content {
    position: absolute;
    inset: 0;
    height: 100%;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 14px;

    transition:
      transform 0.35s cubic-bezier(0.32, 0.72, 0, 1),
      opacity 0.35s cubic-bezier(0.32, 0.72, 0, 1);

    will-change: transform, opacity;
    transform: translate3d(0, 0, 0);
  }

  .collapsed-content.is-hidden {
    transform: translateY(-10px);
    opacity: 0;
    pointer-events: none;
  }

  /* 按钮入场动画 */
  .expanded-content.is-visible .flex-1 {
    animation: button-drop-in 0.2s cubic-bezier(0.32, 0.72, 0, 1) forwards;
    opacity: 0;
    transform: translateY(-30px) scale(0.85);
    will-change: transform, opacity;
  }

  .expanded-content.is-visible .mt-auto {
    animation: progress-fade-in 0.15s cubic-bezier(0.32, 0.72, 0, 1) 0.05s
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

  /* 翻转进入动画 */
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

  /* 呼吸横线动画 */
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
    pointer-events: none;
    overflow: hidden;
    -webkit-app-region: no-drag;
    backdrop-filter: none;
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
    will-change: width, height, border-radius;
    mask-image: radial-gradient(white, black);
  }

  /* 全局 GPU 加速规则 */
  button,
  [data-stop-toggle],
  .flex-1,
  .expanded-content button,
  .expanded-content [data-stop-toggle] {
    transform: translate3d(0, 0, 0) !important;
    backface-visibility: hidden !important;
    -webkit-font-smoothing: subpixel-antialiased;
    will-change: auto;
    perspective: 1000px;
    contain: layout style;
  }

  button:active {
    transform: scale(0.92) !important;
    transition: transform 0.1s ease !important;
  }

  /* 播放控制按钮：独立的下落动画 */
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
</style>
