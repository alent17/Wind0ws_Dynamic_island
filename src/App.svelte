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
  } from "@tauri-apps/api/window";
  import {
    Music,
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Heart,
    Cast,
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

  let expanded = $state(false);
  let hovering = $state(false);
  let accentColor = $state<string>("#fe2c55");
  let artworkUrl = $state<string>("https://picsum.photos/400/400?random=1");
  let trackTitle = $state<string>("");
  let artistName = $state<string>("");
  let isPlaying = $state<boolean>(false);
  let currentTimeMs = $state<number>(0);
  let durationMs = $state<number>(0);
  let currentTrackId = $state<string>(""); // 用来判断是否换歌了
  let lastSyncTimestamp = $state<number>(Date.now()); // 系统给位置的时间戳
  let currentSource = $state<string>("generic"); // 播放器来源
  let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;

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
    manualDuration > 0
      ? (precisePosition() / manualDuration) * 100
      : durationMs > 0
        ? (precisePosition() / durationMs) * 100
        : $progressSpring,
  );

  // --- 核心：手动计时器 (100ms 高频更新) ---
  // 已取消：现在进度完全由系统 GSMTC 控制
  // $effect(() => {
  //   let interval: ReturnType<typeof setInterval>;
  //
  //   if (isPlaying) {
  //     interval = setInterval(() => {
  //       // 每 100ms 自己加 100，提供流畅的进度感
  //       currentTimeMs += 100;
  //
  //       // 如果超过了 API 时长，停止自增
  //       const limit = manualDuration > 0 ? manualDuration : durationMs;
  //       if (limit > 0 && currentTimeMs >= limit) {
  //         currentTimeMs = limit;
  //       }
  //     }, 100);
  //   }
  //
  //   return () => clearInterval(interval);
  // });

  // iOS 26 液态金属感 - 刚度略高保证响应速度，阻尼较低产生细微回弹
  const widthSpring = spring(160, { stiffness: 0.08, damping: 0.32 });
  const heightSpring = spring(37, { stiffness: 0.08, damping: 0.32 });
  const contentOpacity = spring(0, { stiffness: 0.15, damping: 0.9 });

  const win = getCurrentWindow();

  // 记录上次的尺寸，只有真正变化时才同步窗口（防止卡顿）
  let lastW = 0;
  let lastH = 0;

  // 自动收起逻辑
  function startAutoCloseTimer() {
    stopAutoCloseTimer();
    if (expanded) {
      autoCloseTimer = setTimeout(() => {
        expanded = false;
      }, 5000);
    }
  }

  function stopAutoCloseTimer() {
    if (autoCloseTimer) {
      clearTimeout(autoCloseTimer);
      autoCloseTimer = null;
    }
  }

  // 核心同步函数：同步窗口的物理尺寸和位置，确保中心点对齐
  async function syncPhysicalDimensions(w: number, h: number) {
    const dpr = window.devicePixelRatio || 1;

    try {
      // 修正：直接调用导入的 currentMonitor 函数
      const monitor = await currentMonitor();
      if (!monitor) return;

      // 关键：强制向下取整，确保内容容器永远完全覆盖窗口物理像素
      const physW = Math.floor(w * dpr);
      const physH = Math.floor(h * dpr);

      // 计算居中 X
      const screenWidth = monitor.size.width;
      const centerX = Math.floor((screenWidth - physW) / 2);

      // iOS 26 风格：药丸距离顶部稍微远一点点，显得更有悬浮感
      const targetY = Math.floor(12 * dpr);

      // 原子化操作：先改大小，再定位，防止窗口在收缩时因坐标超出范围而"消失"
      await win.setSize(new PhysicalSize(physW, physH));
      await win.setPosition(new PhysicalPosition(centerX, targetY));
    } catch (e) {
      console.error("同步失败", e);
    }
  }

  async function extractDominantColor(imgSrc: string) {
    try {
      const img = new Image();
      img.crossOrigin = "Anonymous";
      img.src = imgSrc;

      await new Promise<void>((resolve, reject) => {
        img.onload = () => resolve();
        img.onerror = () => reject();
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
      console.error("提取颜色失败:", e);
    }
  }

  // 关键：监听状态变化，同步窗口和 Spring
  $effect(() => {
    // 监听这三个核心状态
    const isExp = expanded;
    const isHov = hovering;

    if (artworkUrl) {
      extractDominantColor(artworkUrl);
    }

    const targetW = isExp ? 400 : isHov ? 172 : 160;
    const targetH = isExp ? 210 : isHov ? 40 : 37;

    // 关键：内容透明度的响应速度要快于形变
    if (!isExp) {
      // 收起逻辑：内容先行消失
      contentOpacity.set(0);
      // 内容完全消失后再触发大幅度形变，会显得非常丝滑
      widthSpring.set(targetW);
      heightSpring.set(targetH);
    } else {
      // 展开逻辑：先拉伸形状，再浮现内容
      widthSpring.set(targetW);
      heightSpring.set(targetH);
      // 形状拉伸到一半左右时再开始浮现内容，效果最好
      setTimeout(() => contentOpacity.set(1), 100);
    }

    // 只有尺寸真正变化时才同步窗口（防止卡顿）
    if (targetW !== lastW || targetH !== lastH) {
      syncPhysicalDimensions(targetW, targetH);
      lastW = targetW;
      lastH = targetH;
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

    toggle(); // 触发展开
  }

  function toggle() {
    expanded = !expanded;
    if (expanded) {
      startAutoCloseTimer(); // 点击展开后开启计时
    }
  }

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

  function handleMouseEnter() {
    hovering = true;
    stopAutoCloseTimer(); // 鼠标进入，停止倒计时
  }

  function handleMouseLeave() {
    hovering = false;
    if (expanded) startAutoCloseTimer(); // 鼠标离开且是展开态，开始倒计时
  }

  // 格式化时间为 MM:SS（毫秒输入）
  function formatTime(ms: number): string {
    if (ms <= 0) return "00:00";
    const s = Math.floor(ms / 1000);
    const min = Math.floor(s / 60);
    const sec = s % 60;
    return `${min.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`;
  }

  // 监听媒体变化（事件推送方式 - 不阻塞主线程）
  onMount(async () => {
    const unlisten = await listen("media-update", (event: any) => {
      const data = event.payload;

      // 更新播放器来源
      if (data.source) {
        currentSource = data.source;
      }

      // 1. 处理切歌逻辑
      if (trackTitle !== data.title) {
        trackTitle = data.title || "未在播放";
        artistName = data.artist || "";
        currentSource = data.source || "generic";

        // 使用系统返回的封面（如果有）
        if (data.thumbnail) {
          artworkUrl = data.thumbnail;
        }

        // 重置进度和时长
        currentTimeMs = data.position_ms || 0;
        lastSyncTimestamp = data.last_updated_timestamp || Date.now();
        durationMs = data.duration_ms || 0;

        progressSpring.set(0, { soft: true });
      }

      isPlaying = data.is_playing || false;

      // 2. 进度同步策略（统一使用系统数据）
      currentTimeMs = data.position_ms || 0;
      durationMs = data.duration_ms || 0;

      // 更新 Spring（使用系统时长）
      if (durationMs > 0 && data.position_ms > 0) {
        let percent = (data.position_ms / durationMs) * 100;
        progressSpring.set(percent);
      } else if (durationMs === 0) {
        // 直播模式
        progressSpring.set(0);
      }

      // 提取封面主色（仅当封面是 Base64 时）
      if (artworkUrl && artworkUrl.startsWith("data:")) {
        extractDominantColor(artworkUrl);
      }
    });

    return () => {
      unlisten(); // 组件销毁时取消监听
    };
  });
</script>

<div
  class="fixed inset-0 flex items-start justify-center pointer-events-none"
  style="background: transparent;"
>
  <div
    class="pointer-events-auto relative shadow-2xl"
    style="
      width: {$widthSpring}px;
      height: {$heightSpring}px;
      background-color: #000000;
      border-radius: {expanded ? '54px' : '22px'};
      overflow: hidden;
      display: flex;
      flex-direction: column;
      box-shadow: 0 20px 50px rgba(0,0,0,0.6);
      transform: scale({isPressed ? 0.96 : 1});
      transition: 
        border-radius 0.8s cubic-bezier(0.32, 0.72, 0, 1),
        transform 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    "
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
    onmousedown={() => (isPressed = true)}
    onmouseup={handleRelease}
    onkeydown={(e) => e.key === "Enter" && toggle()}
    role="button"
    tabindex="0"
  >
    <!-- 拖拽区域：只覆盖背景，不遮挡按钮 -->
    <div class="absolute inset-0 z-0" data-tauri-drag-region></div>

    <!-- 内容区域：z-index 高于拖拽区域，按钮可点击 -->
    <div class="w-full h-full relative z-10">
      {#if !expanded}
        <div
          class="h-full w-full flex items-center justify-between"
          style="opacity: {1 - $contentOpacity}; transition: opacity 0.3s;"
        >
          <div
            class="w-6 h-6 rounded-md overflow-hidden flex-shrink-0 ml-[14px]!"
          >
            <img src={artworkUrl} alt="" class="w-full h-full object-cover" />
          </div>

          <div class="flex gap-[3px] items-center h-4 mr-[14px]!">
            {#each [0.6, 1.2, 0.9, 1.5, 0.7] as h, i}
              <div
                class="w-[2px] rounded-full animate-island-wave"
                style="
                  background-color: {accentColor}; 
                  height: {h * 8}px; 
                  animation-delay: {i * 0.15}s;
                  opacity: {1 - $contentOpacity};
                "
              ></div>
            {/each}
          </div>
        </div>
      {:else}
        <div
          class="flex flex-col h-full text-white select-none"
          style="
            padding: 24px 28px 45px 28px;
            box-sizing: border-box;
            opacity: {$contentOpacity};
            transform: translateY({(1 - $contentOpacity) * -15}px) scale({0.95 +
            $contentOpacity * 0.05});
          "
        >
          {#if expanded}
            <!-- 已移除播放器徽章显示 -->
          {/if}

          <div style="margin-bottom: 20px;">
            <div class="flex items-center" style="gap: 16px;">
              <div
                class="w-[68px] h-[68px] rounded-2xl overflow-hidden shadow-2xl ring-1 ring-white/10 flex-shrink-0 cursor-pointer"
                data-stop-toggle
              >
                <img
                  src={artworkUrl}
                  alt="cover"
                  class="w-full h-full object-cover pointer-events-none"
                />
              </div>

              <div class="flex-1 min-w-0">
                <h2
                  class="font-bold text-[19px] truncate text-white tracking-tight"
                >
                  {trackTitle}
                </h2>
                <p class="text-[14px] text-white/50 truncate font-medium">
                  {artistName}
                </p>
              </div>

              <button
                class="w-9 h-9 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 relative z-50"
                data-stop-toggle
                onclick={(e) => {
                  e.stopPropagation(); // 防止触发岛的 toggle
                  console.log("点击了 Cast 按钮");
                }}
              >
                <Cast size={18} class="text-white/80" />
              </button>
            </div>
          </div>

          <div
            class="flex-1 flex items-center justify-between"
            style="margin-bottom: 20px;"
          >
            <Heart
              size={22}
              class="text-white/30 hover:text-red-500 transition-colors relative z-50 cursor-pointer"
              data-stop-toggle
              onclick={(e) => {
                e.stopPropagation();
                console.log("点击了喜欢按钮");
              }}
            />

            <div class="flex items-center" style="gap: 44px;">
              <SkipBack
                size={26}
                fill="currentColor"
                class="text-white/90 active:scale-90 relative z-50 cursor-pointer"
                data-stop-toggle
                onclick={(e) => handleMediaAction("prev", e)}
              />
              {#if isPlaying}
                <Pause
                  size={38}
                  fill="currentColor"
                  class="text-white active:scale-95 relative z-50 cursor-pointer"
                  data-stop-toggle
                  onclick={(e) => handleMediaAction("play_pause", e)}
                />
              {:else}
                <Play
                  size={38}
                  fill="currentColor"
                  class="text-white active:scale-95 relative z-50 cursor-pointer"
                  data-stop-toggle
                  onclick={(e) => handleMediaAction("play_pause", e)}
                />
              {/if}
              <SkipForward
                size={26}
                fill="currentColor"
                class="text-white/90 active:scale-90 relative z-50 cursor-pointer"
                data-stop-toggle
                onclick={(e) => handleMediaAction("next", e)}
              />
            </div>

            <div
              class="w-6 h-6 flex items-center justify-center rounded-md border border-white/10 text-[9px] font-bold text-white/30 relative z-50 cursor-pointer"
              data-stop-toggle
              onclick={(e) => {
                e.stopPropagation();
                console.log("点击了歌词按钮");
              }}
            >
              词
            </div>
          </div>

          <div class="mt-auto" style="margin-bottom: 5px;">
            {#if currentConfig.useProgressBar && durationMs > 0}
              <!-- 普通进度条模式 -->
              <div
                class="relative w-full h-[4px] bg-white/10 rounded-full overflow-hidden"
              >
                <div
                  class="absolute left-0 top-0 h-full rounded-full transition-all duration-300"
                  style="width: {progressPercent}%; background-color: {currentColor};"
                ></div>
              </div>

              <div
                class="flex justify-between text-[11px] font-bold mt-2.5 tracking-tighter"
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
                  class="flex items-center gap-2 opacity-30 text-[9px] tracking-[0.2em] uppercase"
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
      {/if}
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
    transform: translateZ(0);
    will-change: width, height, border-radius;
    mask-image: radial-gradient(white, black);
  }
</style>
