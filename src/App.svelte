<script lang="ts">
  import { onMount } from "svelte";
  import { spring } from "svelte/motion";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
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
  } from "lucide-svelte";

  // 播放器图标映射（使用本地图片资源）
  const platformIcons = {
    netease: "/src/assets/icons/netease.png",
    spotify: "/src/assets/icons/spotify.png",
    bilibili: "/src/assets/icons/bilibili.png",
    qqmusic: "/src/assets/icons/default_music.png", // 使用默认图标作为 QQ 音乐的备选
    apple: "/src/assets/icons/apple_music.png",
    generic: "/src/assets/icons/default_music.png",
  };

  // 播放器名称映射
  const playerNames = {
    netease: "网易云音乐",
    spotify: "Spotify",
    bilibili: "Bilibili",
    qqmusic: "QQ 音乐",
    apple: "Apple Music",
    generic: "多媒体",
  };

  // 播放器颜色配置
  const playerColors = {
    netease: "#ff2d55",
    spotify: "#1db954",
    bilibili: "#fb7299",
    qqmusic: "#31c27c",
    apple: "#fa243c",
    generic: "#ffffff",
  };

  // 播放器配置（包含是否显示进度条）
  const playerConfigs = {
    netease: {
      name: "网易云音乐",
      color: "#ff2d55",
      icon: "/src/assets/icons/netease.png",
      useProgressBar: false, // 关键：强制网易云不显示进度条
    },
    spotify: {
      name: "Spotify",
      color: "#1db954",
      icon: "/src/assets/icons/spotify.png",
      useProgressBar: true,
    },
    bilibili: {
      name: "Bilibili",
      color: "#fb7299",
      icon: "/src/assets/icons/bilibili.png",
      useProgressBar: true,
    },
    qqmusic: {
      name: "QQ 音乐",
      color: "#31c27c",
      icon: "/src/assets/icons/default_music.png",
      useProgressBar: true,
    },
    apple: {
      name: "Apple Music",
      color: "#fa243c",
      icon: "/src/assets/icons/apple_music.png",
      useProgressBar: true,
    },
    generic: {
      name: "正在播放",
      color: "#ffffff",
      icon: "/src/assets/icons/default_music.png",
      useProgressBar: true,
    },
  };

  // ========== 状态管理 ==========
  let expanded = $state(false);
  let hovering = $state(false);
  let accentColor = $state<string>("#fe2c55");
  let artworkUrl = $state<string>("");
  let trackTitle = $state<string>("");
  let artistName = $state<string>("");
  let isPlaying = $state<boolean>(false);
  let currentTimeMs = $state<number>(0);
  let durationMs = $state<number>(0);
  let currentSource = $state<string>("generic");
  let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;
  let currentTheme = $state("original");

  // 全屏检测和自动隐藏相关状态
  let isFullscreenApp = $state(false);
  let isMouseAtTop = $state(false);
  let isHidden = $state(false);
  let autoHideEnabled = $state(true);

  // 显示器选择相关状态
  let showMonitorMenu = $state(false);
  let monitors: Array<{ name: string; index: number }> = $state([]);
  let currentMonitorIndex = $state(0);

  // 当前显示的图标路径（根据 source 动态计算）
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

  // 判定直播逻辑 (只要时长的 0 就是直播)
  let isLive = $derived(durationMs === 0);

  // --- 模拟进度核心 (Spring) ---
  // stiffness 控制追赶速度，damping 控制回弹
  let progressSpring = spring(0, { stiffness: 0.15, damping: 0.8 });

  // --- 灵动岛的"液态进度"计算 ---
  // 已简化：直接使用系统给的进度，不再计算物理时差
  const precisePosition = $derived(() => {
    return currentTimeMs;
  });

  const progressPercent = $derived(
    durationMs > 0 ? (precisePosition() / durationMs) * 100 : $progressSpring,
  );

  // --- 核心：手动计时器 (100ms 高频更新) ---
  // 已取消：现在进度完全由系统 GSMTC 控制

  // ========== 优化的 Spring 参数 ==========
  // 绸缎感动画：形状快速响应，内容优雅浮现
  // 优化：提高 stiffness 加快响应，降低 damping 减少回弹，使动画更流畅
  const widthSpring = spring(160, { stiffness: 0.25, damping: 0.7 });
  const heightSpring = spring(37, { stiffness: 0.25, damping: 0.7 });
  // 关键：contentOpacity 使用更快的参数，减少等待感
  const contentOpacity = spring(0, { stiffness: 0.2, damping: 0.75 });

  const win = getCurrentWindow();

  // ========== 窗口同步优化 ==========
  // 同步队列和缓存机制
  let cachedScreenWidth = 0;
  let cachedScreenHeight = 0;
  let isSyncing = false;
  let pendingW = 0;
  let pendingH = 0;
  let hasPendingSync = false;

  // 显示器锚点：记录用户选择的显示器位置
  let monitorAnchorX = 0;
  let monitorAnchorY = 0;

  // 处理同步队列：批量更新窗口尺寸和位置
  async function processSyncQueue() {
    if (isSyncing || !hasPendingSync) return;

    isSyncing = true;
    hasPendingSync = false;

    const w = pendingW;
    const h = pendingH;
    const dpr = window.devicePixelRatio || 1;

    try {
      // 使用缓存的显示器尺寸
      if (!cachedScreenWidth) {
        const monitor = await currentMonitor();
        if (monitor) {
          cachedScreenWidth = monitor.size.width;
          cachedScreenHeight = monitor.size.height;
          // 初始化锚点
          monitorAnchorX = monitor.position.x + monitor.size.width / 2;
          monitorAnchorY = monitor.position.y;
        }
      }

      const physW = Math.round(w * dpr);
      const physH = Math.round(h * dpr);
      // 使用固定的显示器锚点计算居中位置
      const centerX = Math.round(monitorAnchorX - physW / 2);
      const targetY = Math.round(monitorAnchorY + 12 * dpr);

      // 并发执行窗口操作，减少 IPC 调用次数
      await Promise.all([
        win.setSize(new PhysicalSize(physW, physH)),
        win.setPosition(new PhysicalPosition(centerX, targetY)),
      ]);
    } catch (err) {
      console.error("窗口同步失败:", err);
    } finally {
      isSyncing = false;
      // 处理积压的同步请求
      if (hasPendingSync) {
        requestAnimationFrame(processSyncQueue);
      }
    }
  }

  // 记录上次的尺寸，只有真正变化时才同步窗口（防止卡顿）
  let lastW = 0;
  let lastH = 0;

  /**
   * 核心逻辑：自动收起管理
   */
  function startAutoClose() {
    stopAutoClose(); // 先清理旧的
    // 只有在【已展开】且【鼠标不在界面上】时才启动计时
    if (expanded && !hovering) {
      autoCloseTimer = setTimeout(() => {
        expanded = false;
      }, 5000);
    }
  }

  function stopAutoClose() {
    if (autoCloseTimer) {
      clearTimeout(autoCloseTimer);
      autoCloseTimer = null;
    }
  }

  // 监听展开状态变化：当点击展开时，尝试启动倒计时
  $effect(() => {
    if (expanded) {
      startAutoClose();
    } else {
      stopAutoClose();
      // 收起时关闭显示器选择菜单
      showMonitorMenu = false;
    }
  });

  /**
   * 鼠标交互处理
   */
  function handleMouseEnter() {
    hovering = true;
    stopAutoClose(); // 鼠标进来，立刻停止收起计时
  }

  function handleMouseLeave() {
    hovering = false;
    if (expanded) {
      startAutoClose(); // 鼠标离开，如果是展开状态，重新开始 5s 计时
    }
  }

  async function extractDominantColor(imgSrc: string) {
    if (
      !imgSrc ||
      (imgSrc.startsWith("http") === false &&
        imgSrc.startsWith("data:") === false)
    )
      return;
    try {
      const img = new Image();
      img.crossOrigin = "Anonymous";
      img.src = imgSrc;
      await new Promise((resolve) => {
        img.onload = resolve;
        img.onerror = resolve; // 即使失败也释放，防止堵塞
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
      accentColor = `rgb(${Math.max(Math.floor(r / 25), 100)},${Math.max(Math.floor(g / 25), 100)},${Math.max(Math.floor(b / 25), 100)})`;
    } catch (e) {
      console.warn("取色失败");
    }
  }

  // 关键：监听状态变化，设置 Spring 目标值（绸缎感优化 + 固定尺寸）
  $effect(() => {
    const isExp = expanded;
    const isHov = hovering;

    // 绸缎感核心：容器先展开，内容稍后浮现
    if (isExp) {
      // 展开态固定尺寸：360x180
      widthSpring.set(360);
      heightSpring.set(180);
      // 让透明度稍微滞后于形状变化，产生"容器先开，内容后到"的层级感
      setTimeout(() => contentOpacity.set(1), 50);
    } else {
      // 收起状态固定尺寸
      widthSpring.set(isHov ? 152 : 140);
      heightSpring.set(isHov ? 35 : 32);
      contentOpacity.set(0);
    }
  });

  // --- 核心修复：实时监听 Spring 动画值并同步窗口（优化版）---
  $effect(() => {
    const currentW = $widthSpring;
    const currentH = $heightSpring;

    // 核心优化：将阈值从 0.1 提高到 0.8
    // DOM 元素（内部 UI）依然会呈现 0.1 级别的细腻过渡，
    // 但物理透明窗口只需要精确到 0.8 即可，这能大幅减少操作系统层面的重绘压力
    if (Math.abs(currentW - lastW) > 0.8 || Math.abs(currentH - lastH) > 0.8) {
      pendingW = currentW;
      pendingH = currentH;
      hasPendingSync = true;

      // 如果当前没有在同步，则触发下一帧同步
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

    // 如果点击的是按钮或者按钮里的图标，就不执行 toggle
    const target = e.target as HTMLElement;
    if (target.closest("button") || target.closest("[data-stop-toggle]")) {
      return;
    }

    expanded = !expanded; // 触发展开
  }

  // ========== 媒体控制指令 ==========
  async function handleMediaAction(action: string, e: MouseEvent) {
    e.stopPropagation();
    try {
      await invoke("control_media", { action });
      if (action === "play_pause") {
        isPlaying = !isPlaying;
      }
    } catch (err) {
      console.error("媒体控制失败:", err);
    }
  }

  // ========== 格式化时间为 MM:SS ==========
  function formatTime(ms: number): string {
    if (ms <= 0) return "00:00";
    const s = Math.floor(ms / 1000);
    const min = Math.floor(s / 60);
    const sec = s % 60;
    return `${min.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`;
  }

  // ========== 全屏检测和自动隐藏 ==========
  let fullscreenCheckInterval: ReturnType<typeof setInterval> | null = null;
  let hideTimeout: ReturnType<typeof setTimeout> | null = null;

  async function checkFullscreenAndHide() {
    if (!autoHideEnabled) return;

    try {
      const isFullscreen = await invoke<boolean>("check_fullscreen_app");

      if (isFullscreen !== isFullscreenApp) {
        isFullscreenApp = isFullscreen;
        console.log(
          "[全屏检测] 状态变化:",
          isFullscreen ? "检测到全屏应用" : "全屏应用已关闭",
        );

        if (isFullscreen) {
          // 检测到全屏应用，隐藏窗口到顶部
          hideWindowToTop();
        } else {
          // 全屏应用关闭，显示窗口
          showWindow();
        }
      }
    } catch (error) {
      console.error("[全屏检测] 失败:", error);
    }
  }

  async function hideWindowToTop() {
    try {
      const appWindow = getCurrentWindow();
      const currentSize = await appWindow.innerSize();

      // 获取所有显示器
      const allMonitors = await availableMonitors();

      if (allMonitors.length > 0 && currentMonitorIndex < allMonitors.length) {
        // 使用用户选择的显示器（或当前显示器）
        const targetMonitor = allMonitors[currentMonitorIndex];

        // 计算屏幕中心 X 坐标
        const screenCenterX =
          targetMonitor.position.x + targetMonitor.size.width / 2;
        // 计算窗口中心 X 坐标（使窗口居中）
        const windowCenterX = screenCenterX - currentSize.width / 2;

        // 计算隐藏位置（窗口上边缘移到屏幕外，只留 2px 可见边）
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
        // 兜底：使用默认居中（X=0）
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

      // 获取所有显示器
      const allMonitors = await availableMonitors();

      if (allMonitors.length > 0 && currentMonitorIndex < allMonitors.length) {
        // 使用用户选择的显示器
        const targetMonitor = allMonitors[currentMonitorIndex];

        // 计算屏幕中心 X 坐标
        const screenCenterX =
          targetMonitor.position.x + targetMonitor.size.width / 2;
        // 计算窗口中心 X 坐标（使窗口居中）
        const windowCenterX = screenCenterX - currentSize.width / 2;
        // 恢复到正常位置（距离顶部 12px）
        const targetY = Math.round(12 * dpr);

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
        // 兜底：使用默认居中
        const targetY = Math.round(12 * dpr);
        await appWindow.setPosition(new PhysicalPosition(0, targetY));
        isHidden = false;
        console.log("[自动显示] 未找到显示器，使用默认位置");
      }
    } catch (error) {
      console.error("[自动显示] 失败:", error);
    }
  }

  async function handleMouseMove(event: MouseEvent) {
    if (!autoHideEnabled || !isFullscreenApp) return;

    // 检测鼠标是否在屏幕顶部（y < 100）
    const mouseY = event.clientY;
    const wasMouseAtTop = isMouseAtTop;
    isMouseAtTop = mouseY < 100;

    if (isMouseAtTop !== wasMouseAtTop) {
      console.log("[鼠标检测] 鼠标在顶部:", isMouseAtTop);

      if (isMouseAtTop && isHidden) {
        // 鼠标移动到顶部，显示窗口
        showWindow();

        // 设置自动隐藏定时器（5 秒后如果没有鼠标移动，再次隐藏）
        if (hideTimeout) clearTimeout(hideTimeout);
        hideTimeout = setTimeout(() => {
          if (!isMouseAtTop) {
            hideWindowToTop();
          }
        }, 5000);
      } else if (!isMouseAtTop && !isHidden) {
        // 鼠标离开顶部，延迟隐藏窗口
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

      // 更新显示器锚点
      monitorAnchorX = targetMonitor.position.x + targetMonitor.size.width / 2;
      monitorAnchorY = targetMonitor.position.y;
      cachedScreenWidth = targetMonitor.size.width;
      cachedScreenHeight = targetMonitor.size.height;

      const appWindow = getCurrentWindow();
      const currentSize = await appWindow.innerSize();

      // 设置窗口位置到目标显示器中心
      const targetX = Math.round(monitorAnchorX - currentSize.width / 2);
      const targetY = Math.round(monitorAnchorY + 12); // 距离顶部 12px

      await appWindow.setPosition(new PhysicalPosition(targetX, targetY));
      currentMonitorIndex = index;
      showMonitorMenu = false;

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

  // 点击其他地方关闭菜单
  function closeMonitorMenu() {
    showMonitorMenu = false;
  }

  // 监听媒体变化（事件推送方式 - 不阻塞主线程）
  onMount(async () => {
    console.log("[App.svelte] onMount 开始监听事件");

    // 初始化显示器列表
    try {
      const allMonitors = await availableMonitors();
      monitors = allMonitors.map((m, idx) => ({
        name: m.name || `显示器 ${idx + 1}`,
        index: idx,
      }));

      const activeMonitor = await currentMonitor();
      currentMonitorIndex = activeMonitor
        ? allMonitors.findIndex((m) => m.name === activeMonitor.name)
        : 0;

      // 初始化显示器锚点
      if (activeMonitor) {
        cachedScreenWidth = activeMonitor.size.width;
        cachedScreenHeight = activeMonitor.size.height;
        // 计算初始锚点（显示器中心）
        monitorAnchorX =
          activeMonitor.position.x + activeMonitor.size.width / 2;
        monitorAnchorY = activeMonitor.position.y;
      }

      console.log(
        "[显示器] 初始化完成，当前显示器:",
        monitors[currentMonitorIndex]?.name,
      );
    } catch (error) {
      console.error("[显示器] 初始化失败:", error);
    }

    // 监听导航到设置页面的事件
    const unlistenNavigate = await listen("navigate-to-settings", async () => {
      console.log("[App.svelte] 收到 navigate-to-settings 事件");

      // 使用 Tauri 的 WebviewWindow API 创建设置窗口
      try {
        const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
        console.log("[App.svelte] 导入 WebviewWindow 成功");

        console.log("[App.svelte] 创建设置窗口");
        // 创建新窗口
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

        // 监听窗口加载完成
        webview.once("tauri://created", () => {
          console.log("[设置窗口] 窗口创建成功");
        });

        // 监听窗口错误
        webview.once("tauri://error", (e) => {
          console.error("[设置窗口] 创建失败:", e);
        });
      } catch (error) {
        console.error("[App.svelte] 打开设置窗口失败:", error);
      }
    });

    console.log("[App.svelte] 事件监听器已注册");

    // 监听主题变化
    const unlistenTheme = await listen("theme-changed", (event: any) => {
      const { islandTheme } = event.payload;
      currentTheme = islandTheme || "original";
      console.log("[主题切换] 切换到:", currentTheme);
    });

    // 加载保存的主题
    const savedSettings = localStorage.getItem("dynamic-island-settings");
    if (savedSettings) {
      const settings = JSON.parse(savedSettings);
      currentTheme = settings.islandTheme || "original";
    }

    // 监听 SMTC 推送 (去掉了所有网易云 API 耦合)
    let cleanup: (() => void) | undefined;

    const unlistenMediaUpdate = await listen("media-update", (event: any) => {
      const data = event.payload;

      // 更新基础信息
      if (data.source) currentSource = data.source;
      isPlaying = data.is_playing || false;
      currentTimeMs = data.position_ms || 0;
      durationMs = data.duration_ms || 0;

      // 封面处理：尝试所有可能的字段
      if (trackTitle !== data.title) {
        trackTitle = data.title || "未知曲目";
        artistName = data.artist || "未知艺术家";

        // 尝试获取缩略图 (按优先级尝试所有可能的字段)
        // 注意：后端发送的是 album_art 字段！
        const newCover =
          data.album_art ||
          data.thumbnail ||
          data.cover_url ||
          data.api_cover_url ||
          data.image ||
          "";

        if (newCover && newCover !== artworkUrl) {
          // 验证封面格式
          if (
            newCover.startsWith("data:image") ||
            newCover.startsWith("http://") ||
            newCover.startsWith("https://") ||
            newCover.startsWith("file://")
          ) {
            artworkUrl = newCover;
          } else if (newCover.includes(":\\") || newCover.includes(":/")) {
            // Windows 路径转换为 file:// 协议
            const fileUrl =
              "file:///" + newCover.replace(/\\/g, "/").replace(/^\/+/, "");
            artworkUrl = fileUrl;
          } else {
            artworkUrl = "";
          }
        } else if (!newCover) {
          artworkUrl = "";
        }

        progressSpring.set(0, { soft: true });
      }

      // 更新进度
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
    };
  });

  // 全局点击事件：关闭显示器菜单
  function handleGlobalClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (showMonitorMenu && !target.closest(".relative")) {
      closeMonitorMenu();
    }
  }

  // 注册全局点击事件
  onMount(() => {
    document.addEventListener("click", handleGlobalClick);
    return () => {
      document.removeEventListener("click", handleGlobalClick);
    };
  });

  // 全屏检测和鼠标移动监听
  onMount(async () => {
    // 启动全屏检测定时器（每 2 秒检测一次）
    fullscreenCheckInterval = setInterval(checkFullscreenAndHide, 2000);

    // 立即检测一次
    checkFullscreenAndHide();

    // 监听全局鼠标移动
    document.addEventListener("mousemove", handleMouseMove);

    return () => {
      if (fullscreenCheckInterval) {
        clearInterval(fullscreenCheckInterval);
      }
      if (hideTimeout) {
        clearTimeout(hideTimeout);
      }
      document.removeEventListener("mousemove", handleMouseMove);
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
      background: {currentTheme === 'ios26'
      ? 'rgba(15, 15, 15, 0.6)'
      : '#000000'};
      backdrop-filter: {currentTheme === 'ios26'
      ? 'blur(30px) saturate(200%) brightness(80%)'
      : 'none'};
      -webkit-backdrop-filter: {currentTheme === 'ios26'
      ? 'blur(30px) saturate(200%) brightness(80%)'
      : 'none'};
      border: {currentTheme === 'ios26'
      ? '0.5px solid rgba(255, 255, 255, 0.2)'
      : 'none'};
      box-shadow: {currentTheme === 'ios26'
      ? '0 20px 40px rgba(0, 0, 0, 0.4), inset 0 0 0 1px rgba(255, 255, 255, 0.05), inset 0 1px 2px rgba(255, 255, 255, 0.2)'
      : isHidden
        ? '0 2px 10px rgba(255, 255, 255, 0.1)'
        : '0 20px 50px rgba(0,0,0,0.6)'};
      border-radius: {expanded ? '42px' : '22px'};
      overflow: hidden;
      display: flex;
      flex-direction: column;
      transform: scale({isPressed ? 0.96 : 1});
      transition: 
        border-radius 0.8s cubic-bezier(0.32, 0.72, 0, 1),
        transform 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275),
        background 0.5s ease,
        backdrop-filter 0.5s ease,
        border 0.5s ease,
        box-shadow 0.3s ease,
        opacity 0.4s ease,
        filter 0.4s ease;
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
    {#if currentTheme === "ios26"}
      <!-- 液态玻璃光泽层 -->
      <div class="glass-reflection"></div>
    {/if}
    <!-- 拖拽区域：只覆盖背景，不遮挡按钮 -->
    <div class="absolute inset-0 z-0" data-tauri-drag-region></div>

    <!-- 内容区域：z-index 高于拖拽区域，按钮可点击 -->
    <div class="w-full h-full relative z-10 overflow-hidden">
      <!-- 收起态内容：使用 CSS 类控制显隐，支持退出动画 -->
      <div
        class="collapsed-content"
        class:is-hidden={expanded}
        style="opacity: {1 - $contentOpacity};"
      >
        <div
          class="h-full w-full flex items-center justify-between select-none"
        >
          {#if artworkUrl}
            <div
              class="w-6 h-6 rounded-md overflow-hidden flex-shrink-0 ml-[4px]! select-none"
            >
              <img
                src={artworkUrl}
                alt=""
                class="w-full h-full object-cover"
                onload={() => console.log("[图片加载] 成功加载封面")}
                onerror={(e) => {
                  console.error("[图片加载] 封面加载失败:", artworkUrl);
                  (e.currentTarget as HTMLImageElement).style.display = "none";
                }}
              />
            </div>
          {/if}

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
        </div>
      </div>

      <!-- 展开态内容：使用 CSS 类控制显隐，支持绸缎感动画 -->
      <div
        class="expanded-content"
        class:is-visible={expanded}
        style="opacity: {$contentOpacity};"
      >
        <!-- 顶部区域：封面 + 标题 + Cast 按钮 -->
        <div class="flex items-center" style="gap: 14px; margin-bottom: 16px;">
          {#if artworkUrl}
            <div
              class="w-[56px] h-[56px] rounded-2xl overflow-hidden shadow-2xl ring-1 ring-white/10 flex-shrink-0 cursor-pointer select-none"
              data-stop-toggle
            >
              <img
                src={artworkUrl}
                alt="cover"
                class="w-full h-full object-cover pointer-events-none"
                onload={() => console.log("[图片加载] 成功加载封面 (展开状态)")}
                onerror={(e) => {
                  console.error(
                    "[图片加载] 封面加载失败 (展开状态):",
                    artworkUrl,
                  );
                  (e.currentTarget as HTMLImageElement).style.display = "none";
                }}
              />
            </div>
          {/if}

          <div class="flex-1 min-w-0">
            <h2
              class="font-bold text-[17px] truncate text-white tracking-tight select-none"
            >
              {trackTitle}
            </h2>
            <p
              class="text-[13px] text-white/50 truncate font-medium select-none"
            >
              {artistName}
            </p>
          </div>

          <!-- 显示器选择按钮 -->
          <div class="relative">
            <button
              class="w-8 h-8 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 relative z-50 media-button"
              style="transform: translateZ(0); backface-visibility: hidden;"
              data-stop-toggle
              onclick={(e) => {
                e.stopPropagation();
                toggleMonitorMenu();
              }}
            >
              <Monitor
                size={16}
                class="text-white/80"
                style="transform: translateZ(0); backface-visibility: hidden;"
              />
            </button>

            <!-- 显示器选择菜单 -->
            {#if showMonitorMenu}
              <div
                class="absolute right-0 top-full mt-3 w-64 bg-black/80 backdrop-blur-2xl rounded-3xl shadow-2xl border border-white/10 overflow-hidden z-[100] monitor-menu"
                style="transform: translateZ(0);"
              >
                <div class="p-3">
                  <!-- 标题 -->
                  <div
                    class="flex items-center justify-between px-2 py-1.5 mb-2"
                  >
                    <div class="flex items-center gap-2">
                      <Monitor size={14} class="text-white/60" />
                      <span
                        class="text-xs font-semibold text-white/70 tracking-wide"
                      >
                        显示器
                      </span>
                    </div>
                    <div class="flex items-center gap-1.5">
                      <div
                        class="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse"
                      ></div>
                      <span class="text-[10px] text-white/50 font-medium">
                        {monitors.length} 个显示器
                      </span>
                    </div>
                  </div>

                  <!-- 分隔线 -->
                  <div
                    class="h-px bg-gradient-to-r from-transparent via-white/10 to-transparent mb-2"
                  ></div>

                  <!-- 显示器列表 -->
                  <div class="space-y-1">
                    {#each monitors as monitor, idx}
                      <button
                        class="w-full text-left px-3 py-2.5 rounded-2xl transition-all duration-200 flex items-center gap-3 group relative overflow-hidden"
                        class:selected={currentMonitorIndex === idx}
                        class:hoverable={currentMonitorIndex !== idx}
                        onclick={(e) => {
                          e.stopPropagation();
                          switchMonitor(idx);
                        }}
                      >
                        <!-- 选中状态的背景光晕 -->
                        {#if currentMonitorIndex === idx}
                          <div
                            class="absolute inset-0 bg-gradient-to-br from-white/5 to-transparent"
                          ></div>
                        {/if}

                        <!-- 显示器图标 -->
                        <div
                          class="w-9 h-9 rounded-xl flex items-center justify-center flex-shrink-0 relative transition-colors duration-200"
                          style="background-color: {currentMonitorIndex === idx
                            ? 'rgba(255, 255, 255, 0.10)'
                            : 'rgba(255, 255, 255, 0.05)'};"
                        >
                          <Monitor
                            size={18}
                            class="relative z-10 transition-colors duration-200"
                            style="color: {currentMonitorIndex === idx
                              ? '#ffffff'
                              : 'rgba(255, 255, 255, 0.60)'};"
                          />
                        </div>

                        <!-- 显示器信息 -->
                        <div class="flex-1 min-w-0 relative z-10">
                          <div class="flex items-center gap-2">
                            <span
                              class="text-sm font-medium truncate block transition-colors duration-200"
                              style="color: {currentMonitorIndex === idx
                                ? '#ffffff'
                                : 'rgba(255, 255, 255, 0.70)'};"
                            >
                              {monitor.name}
                            </span>
                          </div>
                          {#if currentMonitorIndex === idx}
                            <div class="text-[10px] text-white/40 mt-0.5">
                              当前使用
                            </div>
                          {/if}
                        </div>

                        <!-- 选中指示器 -->
                        {#if currentMonitorIndex === idx}
                          <div class="flex items-center gap-1.5 relative z-10">
                            <div
                              class="w-5 h-5 rounded-full bg-green-500/20 flex items-center justify-center"
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
                                  stroke="#4ade80"
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
          style="margin-bottom: 20px;"
        >
          <Heart
            size={18}
            class="text-white/30 hover:text-red-500 transition-colors relative z-50 cursor-pointer media-button"
            style="transform: translateZ(0); backface-visibility: hidden;"
            data-stop-toggle
            onclick={(e) => {
              e.stopPropagation();
              console.log("点击了喜欢按钮");
            }}
          />

          <!-- 关键修复：固定按钮容器宽度，防止布局重计算导致的抖动 -->
          <div
            class="flex items-center justify-center"
            style="
                /* 固定宽度，避免父容器宽度变化时重新计算布局 */
                width: 170px;
                gap: 36px;
                /* GPU 加速：隔离合成层，彻底解决抖动 */
                will-change: auto;
                transform: translate3d(0, 0, 0);
                backface-visibility: hidden;
                perspective: 1000px;
                /* 关键：禁用 flex 的自动收缩 */
                flex-shrink: 0;
              "
          >
            <SkipBack
              size={24}
              fill="currentColor"
              class="text-white/90 hover:scale-105 active:scale-90 transition-transform relative z-50 cursor-pointer media-button"
              data-stop-toggle
              onclick={(e) => handleMediaAction("prev", e)}
            />
            {#if isPlaying}
              <Pause
                size={36}
                fill="currentColor"
                class="text-white hover:scale-105 active:scale-95 transition-transform relative z-50 cursor-pointer media-button"
                data-stop-toggle
                onclick={(e) => handleMediaAction("play_pause", e)}
              />
            {:else}
              <Play
                size={36}
                fill="currentColor"
                class="text-white hover:scale-105 active:scale-95 transition-transform relative z-50 cursor-pointer media-button"
                data-stop-toggle
                onclick={(e) => handleMediaAction("play_pause", e)}
              />
            {/if}
            <SkipForward
              size={24}
              fill="currentColor"
              class="text-white/90 hover:scale-105 active:scale-90 transition-transform relative z-50 cursor-pointer media-button"
              data-stop-toggle
              onclick={(e) => handleMediaAction("next", e)}
            />
          </div>

          <button
            class="w-6 h-6 flex items-center justify-center rounded-md border border-white/10 text-[9px] font-bold text-white/30 relative z-50 cursor-pointer bg-transparent hover:bg-white/5 transition-colors media-button"
            style="transform: translateZ(0); backface-visibility: hidden;"
            data-stop-toggle
            aria-label="Toggle lyrics"
            onclick={(e) => {
              e.stopPropagation();
              console.log("点击了歌词按钮");
            }}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.stopPropagation();
                console.log("点击了歌词按钮");
              }
            }}
          >
            词
          </button>
        </div>

        <!-- 底部区域：进度条 -->
        <div class="mt-auto" style="margin-bottom: 4px;">
          {#if currentConfig.useProgressBar && durationMs > 0}
            <!-- 普通进度条模式 -->
            <div
              class="relative w-full h-[3px] bg-white/10 rounded-full overflow-hidden"
            >
              <div
                class="absolute left-0 top-0 h-full rounded-full transition-all duration-300"
                style="width: {progressPercent}%; background-color: {currentColor};"
              ></div>
            </div>

            <div
              class="flex justify-between text-[10px] font-bold mt-2 tracking-tighter"
            >
              <span class="text-white/20">{formatTime(currentTimeMs)}</span>
              <span class="text-white/20">{formatTime(durationMs)}</span>
            </div>
          {:else}
            <!-- 呼吸灯模式（网易云专用或其他无进度情况） -->
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
    animation: island-wave 0.6s ease-in-out infinite;
  }

  /* iOS 26 液态玻璃光泽层 */
  .glass-reflection {
    position: absolute;
    top: -50%;
    left: -50%;
    width: 200%;
    height: 200%;
    background: radial-gradient(
      circle at 50% 50%,
      rgba(255, 255, 255, 0.03) 0%,
      transparent 60%
    );
    pointer-events: none;
    z-index: 0;
  }

  /* 显示器选择菜单选中状态 */
  button.selected {
    background-color: rgba(255, 255, 255, 0.15);
  }

  /* 显示器选择菜单 hover 状态 */
  button.hoverable:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  /* 显示器选择菜单动画 */
  .monitor-menu {
    animation: menu-slide-down 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94)
      forwards;
    transform-origin: top right;
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

  /* ========== 水滴状自动显示动画 ========== */
  /* 当窗口从顶部滑下时，使用水滴状变形效果 */
  .island-drop-animation {
    animation: island-water-drop 0.5s cubic-bezier(0.68, -0.55, 0.265, 1.55)
      forwards;
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
      transform 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94),
      opacity 0.3s ease;
    transform: translateY(-100%);
    opacity: 0;
    pointer-events: none;
  }

  /* 隐藏时可见的顶部边缘 - 添加发光效果 */
  .island-visible-edge {
    box-shadow:
      0 2px 15px rgba(255, 255, 255, 0.3),
      0 0 20px rgba(255, 255, 255, 0.1),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
  }

  /* 隐藏时顶部边缘的白色边框提示 */
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

  /* ========== 绸缎感动画核心 ========== */
  /* 展开内容容器：使用 translateY 位移差产生"浮现"效果 */
  .expanded-content {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    /* 优化：调整内边距以适应更小的尺寸 */
    padding: 20px 24px 36px 24px;

    /* 关键：优化动画曲线和时间，更流畅自然 */
    transform: translateY(30px) scale(0.92);
    transition:
      transform 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94),
      filter 0.35s ease,
      opacity 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);

    filter: blur(8px);
    opacity: 0;
    pointer-events: none;
    will-change: transform, opacity, filter;
  }

  /* 展开状态：内容浮现 */
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

    /* 关键：使用更流畅的动画曲线 */
    transition:
      transform 0.35s cubic-bezier(0.25, 0.46, 0.45, 0.94),
      opacity 0.35s ease;

    will-change: transform, opacity;
  }

  /* 收起态隐藏：向上位移淡出 */
  .collapsed-content.is-hidden {
    transform: translateY(-10px);
    opacity: 0;
    pointer-events: none;
  }

  /* ========== 按钮入场动画 ========== */
  /* 播放控制按钮容器：从上往下放大出现 (优化时长) */
  .expanded-content.is-visible .flex-1 {
    animation: button-drop-in 0.35s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
    opacity: 0;
    transform: translateY(-30px) scale(0.85);
  }

  /* 底部进度条：最后出现 (优化时长) */
  .expanded-content.is-visible .mt-auto {
    animation: progress-fade-in 0.3s ease-out 0.1s forwards;
    opacity: 0;
    transform: translateY(-15px);
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
    animation: breath 2s infinite ease-in-out;
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

  /* 强制内容 div 覆盖 */
  .pointer-events-auto {
    -webkit-font-smoothing: antialiased;
    /* 增加这一行，强制开启 3D 渲染，减少闪烁 */
    transform: translate3d(0, 0, 0) !important;
    will-change: width, height, border-radius;
    mask-image: radial-gradient(white, black);
  }

  /* ========== 全局 GPU 加速规则 ========== */
  /* 强制所有按钮使用 GPU 加速，彻底消除抖动 */
  button,
  [data-stop-toggle],
  .flex-1,
  .expanded-content button,
  .expanded-content [data-stop-toggle] {
    /* 强制开启独立合成层，防止受父级重绘干扰 */
    transform: translate3d(0, 0, 0) !important;
    backface-visibility: hidden !important;
    -webkit-font-smoothing: subpixel-antialiased;
    will-change: auto;
    perspective: 1000px;
    /* 关键：布局隔离，防止父容器宽度变化影响 */
    contain: layout style;
  }

  /* 按钮按压反馈：更灵敏的操作响应 */
  button:active {
    transform: scale(0.92) !important;
    transition: transform 0.1s ease !important;
  }

  /* 播放控制按钮：独立的下落动画 (60fps 优化) */
  .expanded-content.is-visible .media-button {
    animation: button-icon-bounce 0.5s cubic-bezier(0.34, 1.56, 0.64, 1)
      forwards;
    opacity: 0;
    transform: translateY(-25px) scale(0.8);
    /* 确保 GPU 加速 */
    backface-visibility: hidden;
    will-change: transform, opacity;
  }

  /* 逐个延迟，产生级联效果 */
  .expanded-content.is-visible .media-button:nth-child(1) {
    animation-delay: 0.15s;
  }

  .expanded-content.is-visible .media-button:nth-child(2) {
    animation-delay: 0.2s;
  }

  .expanded-content.is-visible .media-button:nth-child(3) {
    animation-delay: 0.25s;
  }

  .expanded-content.is-visible .media-button:nth-child(4) {
    animation-delay: 0.3s;
  }

  @keyframes button-icon-bounce {
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
