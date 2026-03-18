<script lang="ts">
  import { onMount } from 'svelte';
  import { spring } from 'svelte/motion';
  import { getCurrentWindow, PhysicalSize, PhysicalPosition, currentMonitor } from '@tauri-apps/api/window';
  import { Music, Play, Pause, SkipBack, SkipForward, Heart, Cast } from 'lucide-svelte';

  let expanded = $state(false);
  let hovering = $state(false);
  let accentColor = $state<string>('#fe2c55');
  let artworkUrl = $state<string>('https://picsum.photos/400/400?random=1');
  let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;

  // iOS 26 液态金属感 - 刚度略高保证响应速度，阻尼较低产生细微回弹
  const widthSpring = spring(160, { stiffness: 0.12, damping: 0.28 });
  const heightSpring = spring(37, { stiffness: 0.1, damping: 0.25 });
  const contentOpacity = spring(0, { stiffness: 0.15, damping: 0.8 });

  const win = getCurrentWindow();

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

      // 使用 Math.floor 或 Math.round 确保没有小数像素，防止高分屏锯齿
      const physW = Math.round(w * dpr);
      const physH = Math.round(h * dpr);
      
      // 计算居中 X
      const screenWidth = monitor.size.width;
      const centerX = Math.round((screenWidth - physW) / 2);
      
      // iOS 26 风格：药丸距离顶部稍微远一点点，显得更有悬浮感
      const targetY = Math.round(12 * dpr);

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
      img.crossOrigin = 'Anonymous';
      img.src = imgSrc;
      
      await new Promise<void>((resolve, reject) => {
        img.onload = () => resolve();
        img.onerror = () => reject();
      });
      
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d')!;
      canvas.width = 10;
      canvas.height = 10;
      ctx.drawImage(img, 0, 0, 10, 10);
      const data = ctx.getImageData(0, 0, 10, 10).data;
      let r = 0, g = 0, b = 0;
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
    
    const targetW = isExp ? 370 : (isHov ? 172 : 160);
    const targetH = isExp ? 152 : (isHov ? 40 : 37);

    // 1. 设置 Spring 目标
    widthSpring.set(targetW);
    heightSpring.set(targetH);

    // 2. 窗口同步（只在目标改变时执行一次，不放进帧循环）
    syncPhysicalDimensions(targetW, targetH);
    
    // 3. 处理透明度
    contentOpacity.set(isExp ? 1 : 0);
  });

  let isPressed = $state(false);

  function handlePress() {
    isPressed = true;
  }

  function handleRelease() {
    isPressed = false;
    toggle(); // 触发展开
  }

  function toggle() {
    expanded = !expanded;
    if (expanded) {
      startAutoCloseTimer(); // 点击展开后开启计时
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
</script>

<div class="fixed inset-0 flex items-start justify-center overflow-hidden pointer-events-none">
  <div
    class="pointer-events-auto overflow-hidden relative bg-black shadow-2xl transition-all duration-500"
    style="
      width: {$widthSpring}px;
      height: {$heightSpring}px;
      /* 核心修改：大幅增加圆角，展开状态设为高度的 1/3 到 1/4 */
      border-radius: {expanded ? '48px' : '22px'};
      box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
      /* 点击缩放：让药丸看起来像肉冻一样 Q 弹 */
      transform: scale({isPressed ? 0.94 : 1});
      transition: border-radius 0.6s cubic-bezier(0.32, 0.72, 0, 1), transform 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    "
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
    onmousedown={() => isPressed = true}
    onmouseup={() => { isPressed = false; toggle(); }}
    onkeydown={(e) => e.key === 'Enter' && toggle()}
    role="button"
    tabindex="0"
    data-tauri-drag-region
  >
    <div class="w-full h-full rounded-[inherit] overflow-hidden relative">

    {#if !expanded}
      <div class="h-[37px] w-full flex items-center justify-between px-3 absolute top-0">
        <div class="w-6 h-6 rounded-md overflow-hidden flex-shrink-0">
           <img src={artworkUrl} alt="" class="w-full h-full object-cover" />
        </div>
        
        <div class="flex gap-[2px] items-center h-4 px-1">
          {#each [0.6, 1.2, 0.9, 1.5, 0.7] as h, i}
            <div 
              class="w-[2px] rounded-full animate-island-wave"
              style="background-color: {accentColor}; height: {h * 8}px; animation-delay: {i * 0.15}s;"
            ></div>
          {/each}
        </div>
      </div>
    {:else}
      <div 
        class="flex flex-col h-full p-6 text-white select-none"
        style="
          opacity: {$contentOpacity};
          transform: translateY({expanded ? '0px' : '-10px'});
          transition: transform 0.6s cubic-bezier(0.23, 1, 0.32, 1);
        "
      >
        <!-- 顶部：封面 + 信息 -->
        <div class="flex items-center gap-4 mb-5">
          <div class="w-14 h-14 rounded-2xl overflow-hidden shadow-lg ring-1 ring-white/10 flex-shrink-0">
            <img src={artworkUrl} alt="cover" class="w-full h-full object-cover" />
          </div>

          <div class="flex-1 min-w-0">
            <h2 class="font-medium text-[18px] truncate text-white tracking-tight">BOY MEETS GIRL</h2>
            <p class="text-[14px] text-white/60 truncate">WurtS</p>
          </div>

          <button class="w-9 h-9 flex items-center justify-center rounded-full bg-white/5 hover:bg-white/10 transition-colors flex-shrink-0">
            <Cast size={18} class="text-white/80" />
          </button>
        </div>

        <!-- 中部：控制按钮 -->
        <div class="flex items-center justify-between px-2 mb-5">
          <Heart size={22} class="text-white/40 hover:text-red-500 transition-colors" />
          <div class="flex items-center gap-10">
            <SkipBack size={26} fill="currentColor" class="text-white/90" />
            <Pause size={34} fill="currentColor" class="text-white" />
            <SkipForward size={26} fill="currentColor" class="text-white/90" />
          </div>
          <div class="w-6 h-6 flex items-center justify-center rounded-md border border-white/20 text-[10px] font-black text-white/40">
            词
          </div>
        </div>

        <!-- 底部：进度条 -->
        <div class="mt-auto">
          <div class="relative w-full h-[6px] bg-white/15 rounded-full overflow-hidden">
            <div 
              class="absolute left-0 top-0 h-full rounded-full transition-all duration-300 ease-out" 
              style="width: 45%; background-color: {accentColor}"
            ></div>
          </div>
          <div class="flex justify-between text-[11px] font-bold text-white/30 mt-1.5 tracking-tighter">
            <span>01:20</span>
            <span>02:59</span>
          </div>
        </div>
      </div>
    {/if}
    </div>
  </div>
</div>

<style>
  @keyframes island-wave {
    0%, 100% { transform: scaleY(0.6); }
    50% { transform: scaleY(1.8); }
  }
  
  .animate-island-wave {
    animation: island-wave 0.6s ease-in-out infinite;
  }

  :global(html, body) {
    background: transparent !important;
    margin: 0;
    padding: 0;
    width: 100vw;
    height: 100vh;
    /* 必须禁用 pointer-events，由内部药丸开启 */
    pointer-events: none;
    overflow: hidden;
    /* 确保 Tauri 窗口透明背景正确渲染 */
    -webkit-app-region: no-drag;
    backdrop-filter: none;
  }
</style>
