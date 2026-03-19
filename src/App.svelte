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
  let currentTheme = $state("original"); // 当前主题：original 或 ios26

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
    durationMs > 0
      ? (precisePosition() / durationMs) * 100
      : $progressSpring,
  );

  // --- 核心：手动计时器 (100ms 高频更新) ---
  // 已取消：现在进度完全由系统 GSMTC 控制

  // --- iOS 26 液态物理参数 ---
  // stiffness 低、damping 高 = 丝滑绸缎感
  const widthSpring = spring(160, { stiffness: 0.05, damping: 0.4 });
  const heightSpring = spring(37, { stiffness: 0.05, damping: 0.4 });
  
  // 内容透明度：收缩时必须极快消失 (stiffness 高)
  const contentOpacity = spring(0, { stiffness: 0.2, damping: 1 });

  const win = getCurrentWindow();

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

  // 核心同步函数：同步窗口的物理尺寸和位置，确保中心点对齐
  async function syncPhysicalDimensions(w: number, h: number) {
    const dpr = window.devicePixelRatio || 1;

    try {
      // 修正：直接调用导入的 currentMonitor 函数
      const monitor = await currentMonitor();
      if (!monitor) return;

      // 使用 Math.round 替代 Math.floor，减少亚像素渲染导致的边缘闪烁
      const physW = Math.round(w * dpr);
      const physH = Math.round(h * dpr);

      const screenWidth = monitor.size.width;
      const centerX = Math.round((screenWidth - physW) / 2);
      const targetY = Math.round(12 * dpr);

      // 执行同步
      await win.setSize(new PhysicalSize(physW, physH));
      await win.setPosition(new PhysicalPosition(centerX, targetY));
    } catch (e) {
      // 忽略同步中的微小冲突错误
    }
  }

  async function extractDominantColor(imgSrc: string) {
    if (!imgSrc || (imgSrc.startsWith('http') === false && imgSrc.startsWith('data:') === false)) return;
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
      canvas.width = 10; canvas.height = 10;
      ctx.drawImage(img, 0, 0, 10, 10);
      const data = ctx.getImageData(0, 0, 10, 10).data;
      let r = 0, g = 0, b = 0;
      for (let i = 0; i < data.length; i += 4) { r += data[i]; g += data[i + 1]; b += data[i + 2]; }
      accentColor = `rgb(${Math.max(Math.floor(r / 25), 100)},${Math.max(Math.floor(g / 25), 100)},${Math.max(Math.floor(b / 25), 100)})`;
    } catch (e) { console.warn("取色失败"); }
  }

  // 关键：监听状态变化，设置 Spring 目标值
  $effect(() => {
    const isExp = expanded;
    const isHov = hovering;

    // 1. 仅设置目标值，让 Spring 自动跑动画
    if (isExp) {
      widthSpring.set(400);
      heightSpring.set(210);
      contentOpacity.set(1);
    } else {
      // 收起状态：内容消失速度快，形状收缩慢（iOS 26 质感）
      widthSpring.set(isHov ? 172 : 160);
      heightSpring.set(isHov ? 40 : 37);
      contentOpacity.set(0);
    }
    
    // 注意：这里不再调用 syncPhysicalDimensions
  });

  // --- 核心修复：实时监听 Spring 动画值并同步窗口 ---
  $effect(() => {
    // 关键：这里引用 $widthSpring 和 $heightSpring
    // 每当 Spring 动画数值变动（每一帧），都会触发此处的同步
    const currentW = $widthSpring;
    const currentH = $heightSpring;

    // 只有当尺寸发生显著变化时（减少微小浮点数计算导致的频繁调用）再同步
    if (Math.abs(currentW - lastW) > 0.1 || Math.abs(currentH - lastH) > 0.1) {
      syncPhysicalDimensions(currentW, currentH).then(() => {
        lastW = currentW;
        lastH = currentH;
      });
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

    listen("media-update", (event: any) => {
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
        const newCover = data.album_art || data.thumbnail || data.cover_url || data.api_cover_url || data.image || "";
        
        if (newCover && newCover !== artworkUrl) {
          // 验证封面格式
          if (newCover.startsWith("data:image") || newCover.startsWith("http://") || newCover.startsWith("https://") || newCover.startsWith("file://")) {
            artworkUrl = newCover;
          } else if (newCover.includes(":\\") || newCover.includes(":/")) {
            // Windows 路径转换为 file:// 协议
            const fileUrl = "file:///" + newCover.replace(/\\/g, "/").replace(/^\/+/, "");
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
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      if (cleanup) cleanup();
      stopAutoClose();
      unlistenTheme();
    };
  });
</script>

<div
  class="fixed inset-0 flex items-start justify-center pointer-events-none"
  style="background: transparent;"
>
  <div
    class="pointer-events-auto relative"
    class:shadow-2xl={currentTheme === 'original'}
    style="
      width: {$widthSpring}px;
      height: {$heightSpring}px;
      background: {currentTheme === 'ios26' ? 'rgba(15, 15, 15, 0.6)' : '#000000'};
      backdrop-filter: {currentTheme === 'ios26' ? 'blur(30px) saturate(200%) brightness(80%)' : 'none'};
      -webkit-backdrop-filter: {currentTheme === 'ios26' ? 'blur(30px) saturate(200%) brightness(80%)' : 'none'};
      border: {currentTheme === 'ios26' ? '0.5px solid rgba(255, 255, 255, 0.2)' : 'none'};
      box-shadow: {currentTheme === 'ios26' 
        ? '0 20px 40px rgba(0, 0, 0, 0.4), inset 0 0 0 1px rgba(255, 255, 255, 0.05), inset 0 1px 2px rgba(255, 255, 255, 0.2)' 
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
        border 0.5s ease;
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
    {#if currentTheme === 'ios26'}
      <!-- 液态玻璃光泽层 -->
      <div class="glass-reflection"></div>
    {/if}
    <!-- 拖拽区域：只覆盖背景，不遮挡按钮 -->
    <div class="absolute inset-0 z-0" data-tauri-drag-region></div>

    <!-- 内容区域：z-index 高于拖拽区域，按钮可点击 -->
    <div class="w-full h-full relative z-10">
      {#if !expanded}
        <div
          class="h-full w-full flex items-center justify-between select-none"
          style="opacity: {1 - $contentOpacity}; transition: opacity 0.3s;"
        >
          <div class="w-6 h-6 rounded-md overflow-hidden flex-shrink-0 ml-[14px]! bg-white/10 select-none">
            {#if artworkUrl}
              <img 
                src={artworkUrl} 
                alt="" 
                class="w-full h-full object-cover" 
                onload={() => console.log("[图片加载] 成功加载封面")}
                onerror={(e) => {
                  console.error("[图片加载] 封面加载失败:", artworkUrl);
                  (e.currentTarget as HTMLImageElement).style.display = 'none';
                }}
              />
            {/if}
          </div>

          <div class="flex gap-[3px] items-center h-4 mr-[14px]!">
            {#each [0.6, 1.2, 0.9, 1.5, 0.7] as h, i}
              <div
                class="w-[2px] rounded-full"
                class:animate-island-wave={isPlaying}
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
          class="expanded-content"
          style="
            padding: 24px 28px 45px 28px;
            opacity: {$contentOpacity};
            transform: translateY({($contentOpacity - 1) * 20}px);
          "
        >
          {#if expanded}
            <!-- 已移除播放器徽章显示 -->
          {/if}

          <div style="margin-bottom: 20px;">
            <div class="flex items-center" style="gap: 16px;">
              <div
                class="w-[68px] h-[68px] rounded-2xl overflow-hidden shadow-2xl ring-1 ring-white/10 flex-shrink-0 cursor-pointer select-none"
                data-stop-toggle
              >
                {#if artworkUrl}
                  <img
                    src={artworkUrl}
                    alt="cover"
                    class="w-full h-full object-cover pointer-events-none"
                    onload={() => console.log("[图片加载] 成功加载封面 (展开状态)")}
                    onerror={(e) => {
                      console.error("[图片加载] 封面加载失败 (展开状态):", artworkUrl);
                      (e.currentTarget as HTMLImageElement).style.display = 'none';
                    }}
                  />
                {/if}
              </div>

              <div class="flex-1 min-w-0">
            <h2
              class="font-bold text-[19px] truncate text-white tracking-tight select-none"
            >
              {trackTitle}
            </h2>
            <p class="text-[14px] text-white/50 truncate font-medium select-none">
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

            <button
              class="w-6 h-6 flex items-center justify-center rounded-md border border-white/10 text-[9px] font-bold text-white/30 relative z-50 cursor-pointer bg-transparent hover:bg-white/5 transition-colors"
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

  /* iOS 26 液态玻璃光泽层 */
  .glass-reflection {
    position: absolute;
    top: -50%;
    left: -50%;
    width: 200%;
    height: 200%;
    background: radial-gradient(circle at 50% 50%, rgba(255,255,255,0.03) 0%, transparent 60%);
    pointer-events: none;
    z-index: 0;
  }

  /* 内容切换动画容器 */
  .expanded-content {
    transition: transform 0.6s cubic-bezier(0.32, 0.72, 0, 1);
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .pill-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 100%;
    padding: 0 14px;
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
