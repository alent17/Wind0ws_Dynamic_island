<script lang="ts">
  import { onMount } from "svelte";
  import { tweened } from "svelte/motion"; // 使用绝对平滑的 tweened 替代会抖动的 spring
  import { cubicOut } from "svelte/easing";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import {
    getCurrentWindow,
    currentMonitor,
    availableMonitors,
  } from "@tauri-apps/api/window";
  import {
    Music, Play, Pause, SkipBack, SkipForward, Heart, Monitor
  } from "lucide-svelte";

  const platformIcons = {
    netease: "/src/assets/icons/netease.png", spotify: "/src/assets/icons/spotify.png",
    bilibili: "/src/assets/icons/bilibili.png", qqmusic: "/src/assets/icons/default_music.png",
    apple: "/src/assets/icons/apple_music.png", generic: "/src/assets/icons/default_music.png",
  };
  const playerConfigs = {
    netease: { name: "网易云音乐", useProgressBar: false }, spotify: { name: "Spotify", useProgressBar: true },
    bilibili: { name: "Bilibili", useProgressBar: true }, qqmusic: { name: "QQ 音乐", useProgressBar: true },
    apple: { name: "Apple Music", useProgressBar: true }, generic: { name: "正在播放", useProgressBar: true },
  };
  const playerColors = {
    netease: "#ff2d55", spotify: "#1db954", bilibili: "#fb7299",
    qqmusic: "#31c27c", apple: "#fa243c", generic: "#ffffff",
  };

  let expanded = $state(false);
  let hovering = $state(false);
  let accentColor = $state<string>("#ffffff");

  let artworkUrl = $state<string>("");
  let trackTitle = $state<string>("");
  let artistName = $state<string>("");
  let isPlaying = $state<boolean>(false);
  let currentTimeMs = $state<number>(0);
  let durationMs = $state<number>(0);
  let currentSource = $state<string>("generic");
  let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;
  let currentTheme = $state("original");

  let isFullscreenApp = $state(false);
  let isMouseAtTop = $state(false);
  let isHidden = $state(false);
  let autoHideEnabled = $state(true);

  let showMonitorMenu = $state(false);
  let monitors: Array<{ name: string; index: number }> = $state([]);
  let currentMonitorIndex = $state(0);
  let isPressed = $state(false);

  const win = getCurrentWindow();

  // ========== 核心 1：数学级丝滑曲线 ==========
  // 持续 350ms 的 Apple 经典阻尼曲线
  const islandSize = tweened({ w: 140, h: 32 }, { duration: 350, easing: cubicOut });
  const contentOpacity = tweened(0, { duration: 300, easing: cubicOut });

  const currentColor = $derived(playerColors[currentSource as keyof typeof playerColors] || playerColors.generic);
  const currentConfig = $derived(playerConfigs[currentSource as keyof typeof playerConfigs] || playerConfigs.generic);
  const progressPercent = $derived(durationMs > 0 ? (currentTimeMs / durationMs) * 100 : 0);

  let monitorAnchorX = 0;
  let monitorAnchorY = 0;

  // ========== 核心 2：极速原生通讯 (无阻塞 Fire-and-Forget) ==========
  let frameScheduled = false;

  islandSize.subscribe(($size) => {
    if (!frameScheduled) {
      frameScheduled = true;
      requestAnimationFrame(() => {
        frameScheduled = false;
        
        const dpr = window.devicePixelRatio || 1;
        // 蓝框与黑岛 1:1 绝对相等，不再预留一丁点多余边界
        const physW = Math.round($size.w * dpr);
        const physH = Math.round($size.h * dpr);
        const centerX = Math.round(monitorAnchorX - physW / 2);
        const targetY = Math.round(monitorAnchorY + 12 * dpr);

        // 【最关键的一步】：调用原生加速接口，并且绝不加 await！
        // 让 JS 瞬间跑完继续渲染下一帧 DOM，从而榨干显示器的刷新率
        invoke("sync_window_bounds", { 
          width: physW, height: physH, x: centerX, y: targetY 
        }).catch(() => {});
      });
    }
  });

  $effect(() => {
    if (expanded) {
      islandSize.set({ w: 360, h: 180 });
      contentOpacity.set(1);
    } else {
      islandSize.set({ w: hovering ? 152 : 140, h: hovering ? 35 : 32 });
      contentOpacity.set(0);
    }
  });

  // ========== 交互与生命周期 ==========
  function startAutoClose() { stopAutoClose(); if (expanded && !hovering) { autoCloseTimer = setTimeout(() => { expanded = false; }, 5000); } }
  function stopAutoClose() { if (autoCloseTimer) { clearTimeout(autoCloseTimer); autoCloseTimer = null; } }
  $effect(() => { if (expanded) startAutoClose(); else { stopAutoClose(); showMonitorMenu = false; } });

  function handleMouseEnter() { hovering = true; stopAutoClose(); }
  function handleMouseLeave() { isPressed = false; hovering = false; if (expanded) startAutoClose(); }
  function handleRelease(e: MouseEvent) {
    isPressed = false;
    const target = e.target as HTMLElement;
    if (target.closest("button") || target.closest("[data-stop-toggle]")) return;
    expanded = !expanded;
  }

  async function handleMediaAction(action: string, e: MouseEvent) { e.stopPropagation(); try { await invoke("control_media", { action }); if (action === "play_pause") isPlaying = !isPlaying; } catch (err) {} }
  function formatTime(ms: number): string { if (ms <= 0) return "00:00"; const s = Math.floor(ms / 1000); const min = Math.floor(s / 60); const sec = s % 60; return `${min.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`; }

  async function extractDominantColor(imgSrc: string) {
    if (!imgSrc || (!imgSrc.startsWith("http") && !imgSrc.startsWith("data:"))) return;
    const img = new Image();
    try { img.crossOrigin = "Anonymous"; img.src = imgSrc; await new Promise((resolve) => { img.onload = resolve; img.onerror = resolve; }); const canvas = document.createElement("canvas"); const ctx = canvas.getContext("2d")!; canvas.width = 10; canvas.height = 10; ctx.drawImage(img, 0, 0, 10, 10); const data = ctx.getImageData(0, 0, 10, 10).data; let r = 0, g = 0, b = 0; for (let i = 0; i < data.length; i += 4) { r += data[i]; g += data[i + 1]; b += data[i + 2]; } accentColor = `rgb(${Math.max(Math.floor(r / 25), 100)},${Math.max(Math.floor(g / 25), 100)},${Math.max(Math.floor(b / 25), 100)})`; } catch (e) {} finally { img.src = ""; img.remove(); }
  }

  // 修改：由于废弃了手动 PhysicalPosition 类，我们改用底层加速的 invoke 进行纯位置位移隐藏
  async function hideWindowToTop() { 
    isHidden = true; 
    const dpr = window.devicePixelRatio || 1;
    invoke("sync_window_bounds", { width: Math.round($islandSize.w * dpr), height: Math.round($islandSize.h * dpr), x: Math.round(monitorAnchorX - ($islandSize.w * dpr) / 2), y: Math.round(-300 * dpr) }).catch(()=>{});
  }
  async function showWindow() { 
    isHidden = false; 
    const dpr = window.devicePixelRatio || 1;
    invoke("sync_window_bounds", { width: Math.round($islandSize.w * dpr), height: Math.round($islandSize.h * dpr), x: Math.round(monitorAnchorX - ($islandSize.w * dpr) / 2), y: Math.round(monitorAnchorY + 12 * dpr) }).catch(()=>{});
  }
  
  let hideTimeout: ReturnType<typeof setTimeout> | null = null;
  async function handleMouseMove(event: MouseEvent) { if (!autoHideEnabled || !isFullscreenApp) return; const mouseY = event.clientY; const wasMouseAtTop = isMouseAtTop; isMouseAtTop = mouseY < 100; if (isMouseAtTop !== wasMouseAtTop) { if (isMouseAtTop && isHidden) { showWindow(); if (hideTimeout) clearTimeout(hideTimeout); hideTimeout = setTimeout(() => { if (!isMouseAtTop) hideWindowToTop(); }, 5000); } else if (!isMouseAtTop && !isHidden) { if (hideTimeout) clearTimeout(hideTimeout); hideTimeout = setTimeout(() => { if (!isMouseAtTop) hideWindowToTop(); }, 500); } } }

  async function switchMonitor(index: number) {
    try {
      const allMonitors = await availableMonitors();
      const targetMonitor = allMonitors[index];
      if (!targetMonitor) return;
      monitorAnchorX = targetMonitor.position.x + targetMonitor.size.width / 2;
      monitorAnchorY = targetMonitor.position.y;
      currentMonitorIndex = index;
      showMonitorMenu = false;
      // 触发一次强同步
      islandSize.set({ w: $islandSize.w, h: $islandSize.h });
    } catch (error) {}
  }

  function toggleMonitorMenu() { showMonitorMenu = !showMonitorMenu; }
  function closeMonitorMenu() { showMonitorMenu = false; }
  function handleGlobalClick(event: MouseEvent) { const target = event.target as HTMLElement; if (showMonitorMenu && !target.closest(".relative")) closeMonitorMenu(); }

  let fullscreenCheckInterval: ReturnType<typeof setInterval> | null = null;
  async function checkFullscreenAndHide() { if (!autoHideEnabled) return; try { const isFullscreen = await invoke<boolean>("check_fullscreen_app"); if (isFullscreen !== isFullscreenApp) { isFullscreenApp = isFullscreen; if (isFullscreen) hideWindowToTop(); else showWindow(); } } catch (error) {} }

  onMount(() => {
    let cleanup: (() => void) | undefined;
    let unlistenTheme: (() => void) | undefined;

    (async () => {
      try {
        const allMonitors = await availableMonitors();
        monitors = allMonitors.map((m, idx) => ({ name: m.name || `显示器 ${idx + 1}`, index: idx }));
        const activeMonitor = await currentMonitor();
        currentMonitorIndex = activeMonitor ? allMonitors.findIndex((m) => m.name === activeMonitor.name) : 0;
        if (activeMonitor) {
          monitorAnchorX = activeMonitor.position.x + activeMonitor.size.width / 2;
          monitorAnchorY = activeMonitor.position.y;
          islandSize.set({ w: 140, h: 32 }); 
        }
      } catch (error) {}

      unlistenTheme = await listen("theme-changed", (event: any) => { currentTheme = event.payload.islandTheme || "original"; });
      const savedSettings = localStorage.getItem("dynamic-island-settings");
      if (savedSettings) { currentTheme = JSON.parse(savedSettings).islandTheme || "original"; }

      cleanup = await listen("media-update", (event: any) => {
        const data = event.payload; if (data.source) currentSource = data.source; isPlaying = data.is_playing || false; currentTimeMs = data.position_ms || 0; durationMs = data.duration_ms || 0;
        if (trackTitle !== data.title) {
          trackTitle = data.title || "未知曲目"; artistName = data.artist || "未知艺术家";
          const newCover = data.album_art || data.thumbnail || data.cover_url || data.image || "";
          if (newCover && newCover !== artworkUrl) { artworkUrl = (newCover.includes(":\\") || newCover.includes(":/")) ? "file:///" + newCover.replace(/\\/g, "/").replace(/^\/+/, "") : newCover; extractDominantColor(artworkUrl); } else if (!newCover) { artworkUrl = ""; }
        }
      });
      document.addEventListener("click", handleGlobalClick);
      document.addEventListener("mousemove", handleMouseMove);
      fullscreenCheckInterval = setInterval(checkFullscreenAndHide, 2000);
      checkFullscreenAndHide();
    })();

    return () => { if (cleanup) cleanup(); if (unlistenTheme) unlistenTheme(); stopAutoClose(); document.removeEventListener("click", handleGlobalClick); document.removeEventListener("mousemove", handleMouseMove); if (fullscreenCheckInterval) clearInterval(fullscreenCheckInterval); if (hideTimeout) clearTimeout(hideTimeout); };
  });
</script>

<div class="fixed inset-0 flex items-start justify-center pointer-events-none" style="background: transparent;">
  
  <div
    class="pointer-events-auto relative dynamic-island-container"
    class:island-hidden={isHidden && !isMouseAtTop}
    class:island-drop-animation={isMouseAtTop && isHidden}
    class:island-visible-edge={isHidden && isMouseAtTop}
    style="
      width: {$islandSize.w}px;
      height: {$islandSize.h}px;
      background: #000000;
      border-radius: {expanded ? '42px' : '22px'};
      transform: scale({isPressed ? 0.96 : 1});
    "
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
    onmousedown={() => (isPressed = true)}
    onmouseup={handleRelease}
    onkeydown={(e) => e.key === "Enter" && handleRelease(e as any)}
    role="button"
    tabindex="0"
    aria-label="Dynamic Island - Click to toggle"
  >
    <div class="absolute inset-0 z-0" data-tauri-drag-region></div>

    <div class="w-full h-full relative z-10 overflow-hidden">
      <div class="collapsed-content" class:is-hidden={expanded} style="opacity: {1 - $contentOpacity}; pointer-events: {expanded ? 'none' : 'auto'};">
        <div class="h-full w-full flex items-center justify-between select-none">
          {#if artworkUrl}
            <div class="w-6 h-6 rounded-md overflow-hidden flex-shrink-0 ml-[4px]! select-none">
              <img src={artworkUrl} alt="" class="w-full h-full object-cover" onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = "none"; }} />
            </div>
          {/if}
          <div class="flex gap-[2px] items-center h-4 mr-[4px]!">
            {#each [0.6, 1.2, 0.9, 1.5, 0.7] as h, i}
              <div class="w-[2px] rounded-full" class:animate-island-wave={isPlaying} style="background-color: {accentColor}; height: {h * 8}px; animation-delay: {i * 0.15}s;"></div>
            {/each}
          </div>
        </div>
      </div>

      <div class="expanded-content" class:is-visible={expanded} style="opacity: {$contentOpacity}; pointer-events: {expanded ? 'auto' : 'none'};">
        <div class="flex items-center" style="gap: 16px; margin-bottom: 20px;">
          {#if artworkUrl}
            <div class="w-[60px] h-[60px] rounded-[18px] overflow-hidden flex-shrink-0 cursor-pointer select-none" data-stop-toggle>
              <img src={artworkUrl} alt="cover" class="w-full h-full object-cover pointer-events-none" onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = "none"; }} />
            </div>
          {/if}
          <div class="flex-1 min-w-0" style="padding-top: 2px;">
            <h2 class="font-semibold text-[17px] truncate text-white/95 tracking-tight select-none">{trackTitle}</h2>
            <p class="text-[13.5px] text-white/50 truncate font-medium select-none mt-0.5">{artistName}</p>
          </div>
          
          <div class="relative">
            <button class="w-[32px] h-[32px] flex items-center justify-center rounded-full bg-[#1c1c1e] hover:bg-[#2c2c2e] transition-colors relative z-50 media-button" style="transform: translateZ(0); backface-visibility: hidden;" data-stop-toggle onclick={(e) => { e.stopPropagation(); toggleMonitorMenu(); }}>
              <Monitor size={15} class="text-white/70" style="transform: translateZ(0); backface-visibility: hidden;" />
            </button>
            
            {#if showMonitorMenu}
              <div class="absolute right-0 top-full mt-3 w-64 bg-[#1c1c1e] rounded-3xl border border-[#2c2c2e] overflow-hidden z-[100] monitor-menu" style="transform: translateZ(0);">
                <div class="p-3">
                  <div class="flex items-center justify-between px-2 py-1.5 mb-2">
                    <div class="flex items-center gap-2"><Monitor size={14} class="text-white/60" /><span class="text-xs font-semibold text-white/70 tracking-wide">显示器</span></div>
                    <div class="flex items-center gap-1.5"><div class="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"></div><span class="text-[10px] text-white/50 font-medium">{monitors.length} 个</span></div>
                  </div>
                  <div class="h-px bg-gradient-to-r from-transparent via-white/10 to-transparent mb-2"></div>
                  <div class="space-y-1">
                    {#each monitors as monitor, idx}
                      <button class="w-full text-left px-3 py-2.5 rounded-2xl transition-all duration-200 flex items-center gap-3 group relative overflow-hidden" class:selected={currentMonitorIndex === idx} class:hoverable={currentMonitorIndex !== idx} onclick={(e) => { e.stopPropagation(); switchMonitor(idx); }}>
                        {#if currentMonitorIndex === idx}<div class="absolute inset-0 bg-[#2c2c2e]"></div>{/if}
                        <div class="w-9 h-9 rounded-xl flex items-center justify-center flex-shrink-0 relative transition-colors duration-200" style="background-color: {currentMonitorIndex === idx ? '#3a3a3c' : '#2c2c2e'};">
                          <Monitor size={18} class="relative z-10 transition-colors duration-200" style="color: {currentMonitorIndex === idx ? '#ffffff' : 'rgba(255, 255, 255, 0.60)'};" />
                        </div>
                        <div class="flex-1 min-w-0 relative z-10">
                          <div class="flex items-center gap-2"><span class="text-sm font-medium truncate block transition-colors duration-200" style="color: {currentMonitorIndex === idx ? '#ffffff' : 'rgba(255, 255, 255, 0.70)'};">{monitor.name}</span></div>
                          {#if currentMonitorIndex === idx}<div class="text-[10px] text-white/40 mt-0.5">当前使用</div>{/if}
                        </div>
                      </button>
                    {/each}
                  </div>
                </div>
              </div>
            {/if}
          </div>
        </div>

        <div class="flex-1 flex items-center justify-between" style="margin-bottom: 22px;">
          <Heart size={18} class="text-white/30 hover:text-red-500 transition-colors relative z-50 cursor-pointer media-button" style="transform: translateZ(0); backface-visibility: hidden;" data-stop-toggle onclick={(e) => { e.stopPropagation(); }} />
          <div class="flex items-center justify-center" style="width: 170px; gap: 36px; will-change: auto; transform: translate3d(0, 0, 0); backface-visibility: hidden; perspective: 1000px; flex-shrink: 0;">
            <SkipBack size={24} fill="currentColor" class="text-white/90 hover:scale-105 active:scale-90 transition-transform relative z-50 cursor-pointer media-button" data-stop-toggle onclick={(e) => handleMediaAction("prev", e)} />
            {#if isPlaying}
              <Pause size={36} fill="currentColor" class="text-white hover:scale-105 active:scale-95 transition-transform relative z-50 cursor-pointer media-button" data-stop-toggle onclick={(e) => handleMediaAction("play_pause", e)} />
            {:else}
              <Play size={36} fill="currentColor" class="text-white hover:scale-105 active:scale-95 transition-transform relative z-50 cursor-pointer media-button" data-stop-toggle onclick={(e) => handleMediaAction("play_pause", e)} />
            {/if}
            <SkipForward size={24} fill="currentColor" class="text-white/90 hover:scale-105 active:scale-90 transition-transform relative z-50 cursor-pointer media-button" data-stop-toggle onclick={(e) => handleMediaAction("next", e)} />
          </div>
          <button class="w-[26px] h-[26px] flex items-center justify-center rounded-md text-[10px] font-bold text-white/60 relative z-50 cursor-pointer bg-[#1c1c1e] hover:bg-[#2c2c2e] transition-colors media-button" style="transform: translateZ(0); backface-visibility: hidden;" data-stop-toggle aria-label="Toggle lyrics" onclick={(e) => { e.stopPropagation(); }}>词</button>
        </div>

        <div class="mt-auto" style="margin-bottom: 2px;">
          {#if currentConfig.useProgressBar && durationMs > 0}
            <div class="relative w-full h-[4px] bg-[#2c2c2e] rounded-full overflow-hidden">
              <div class="absolute left-0 top-0 h-full rounded-full transition-all duration-300 ease-out" style="width: {progressPercent}%; background-color: {currentColor};"></div>
            </div>
            <div class="flex justify-between text-[10px] font-semibold mt-2 tracking-wide opacity-50">
              <span>{formatTime(currentTimeMs)}</span><span>{formatTime(durationMs)}</span>
            </div>
          {:else}
            <div class="flex flex-col items-center gap-2">
              <div class="breath-line w-full h-[3px] rounded-full" style="--accent-color: {currentColor}"></div>
              <div class="flex items-center gap-2 opacity-40 text-[9px] font-medium tracking-[0.2em] uppercase mt-1">
                {#if isPlaying}
                  <span class="animate-pulse">● Playing on {currentConfig.name}</span>
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
  .dynamic-island-container {
    overflow: hidden;
    display: flex;
    flex-direction: column;
    /* 取消全部会引起打架的 CSS Width/Height 过渡，完全忠实于 Tweened 数学曲线 */
    transition: transform 0.2s ease, border-radius 0.4s ease;
    will-change: border-radius, transform;
  }

  .expanded-content {
    position: absolute;
    top: 0;
    left: 50%;
    margin-left: -180px; 
    width: 360px;
    height: 180px;
    display: flex;
    flex-direction: column;
    padding: 18px 20px 22px 20px;
    transform: scale(0.96);
    filter: blur(2px);
    transition: transform 0.35s cubic-bezier(0.25, 1, 0.5, 1), filter 0.3s ease;
    pointer-events: none;
    will-change: transform, filter;
  }

  .expanded-content.is-visible {
    transform: scale(1);
    filter: blur(0);
    pointer-events: auto;
  }

  .collapsed-content {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 14px;
    transition: transform 0.35s ease;
  }

  .collapsed-content.is-hidden {
    transform: scale(0.95);
    pointer-events: none;
  }

  @keyframes island-wave { 0%, 100% { transform: scaleY(0.6); } 50% { transform: scaleY(1.8); } }
  .animate-island-wave { animation: island-wave 0.6s ease-in-out infinite; }
  
  .monitor-menu { animation: menu-slide-down 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards; transform-origin: top right; }
  @keyframes menu-slide-down { from { opacity: 0; transform: translateY(-10px) scale(0.95); } to { opacity: 1; transform: translateY(0) scale(1); } }
  
  button:active { transform: scale(0.92); }
  @keyframes breath { 0%, 100% { opacity: 0.2; transform: scaleX(0.95); } 50% { opacity: 0.8; transform: scaleX(1); } }
  .breath-line { width: 100%; height: 3px; border-radius: 99px; background: linear-gradient(90deg, transparent, var(--accent-color), transparent); animation: breath 2s infinite ease-in-out; }
</style>