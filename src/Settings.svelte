<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    X,
    Minus,
    Maximize2,
    Palette,
    Music,
    Monitor,
    Pin,
    Image,
    Zap,
    Power,
    Database,
    Trash2,
    FolderOpen,
    Check,
  } from "lucide-svelte";

  interface AppSettings {
    island_theme: string;
    auto_hide: boolean;
    show_spectrum: boolean;
    always_on_top: boolean;
    player_weights: Record<string, number>;
    enable_mv_playback: boolean;
    lock_floating_window: boolean;
    enable_hd_cover: boolean;
    enable_pixel_art: boolean;
    auto_start: boolean;
  }

  let settings = $state<AppSettings>({
    island_theme: "original",
    auto_hide: true,
    show_spectrum: true,
    always_on_top: true,
    player_weights: {
      netease: 50,
      spotify: 50,
      bilibili: 50,
      qqmusic: 50,
      apple: 50,
      generic: 10,
    },
    enable_mv_playback: false,
    lock_floating_window: false,
    enable_hd_cover: true,
    enable_pixel_art: false,
    auto_start: false,
  });

  const appWindow = getCurrentWindow();

  const themes = [
    {
      id: "original",
      name: "经典黑",
      color: "#1a1a1a",
      gradient: "linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 100%)",
    },
    {
      id: "glassmorphism",
      name: "毛玻璃",
      color: "#1a1a2e",
      gradient: "linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)",
    },
  ];

  const playerNames: Record<string, string> = {
    netease: "网易云音乐",
    spotify: "Spotify",
    bilibili: "Bilibili",
    qqmusic: "QQ 音乐",
    apple: "Apple Music",
    generic: "其他",
  };

  let playerOrder = $state<string[]>([]);
  let dragIndex = $state<number | null>(null);
  let overIndex = $state<number | null>(null);
  let isDragging = $state(false);

  let cacheSize = $state("0 MB");
  let cachePath = $state("");

  onMount(async () => {
    try {
      const saved = await invoke<AppSettings>("get_settings");
      settings = { ...settings, ...saved };
      if (saved.player_weights) {
        settings.player_weights = saved.player_weights;
      }

      try {
        const autoStart = await invoke<boolean>("get_auto_start");
        settings.auto_start = autoStart;
      } catch (e) {
        console.error("加载开机启动状态失败", e);
      }

      playerOrder = Object.keys(settings.player_weights);
      loadCacheStats();
      loadCacheDirectory();
    } catch (e) {
      console.error("无法读取设置", e);
    }
  });

  async function saveSettings(newSettings: Partial<AppSettings>) {
    try {
      settings = { ...settings, ...newSettings };
      await invoke("save_settings", { settings });
      console.log("[设置] 已保存:", Object.keys(newSettings)[0]);
    } catch (e) {
      console.error("保存设置失败", e);
    }
  }

  async function toggleAutoStart() {
    try {
      const newState = !settings.auto_start;
      await invoke("set_auto_start", { enable: newState });
      settings.auto_start = newState;
      console.log("[开机启动] 已", newState ? "开启" : "关闭");
    } catch (e) {
      console.error("切换开机启动失败", e);
    }
  }

  async function loadCacheStats() {
    try {
      const stats: any = await invoke("get_cache_stats");
      cacheSize = `${stats.total_size_mb.toFixed(2)} MB (${stats.total_files} 个文件)`;
    } catch (e) {
      console.error("加载缓存统计失败", e);
    }
  }

  async function loadCacheDirectory() {
    try {
      cachePath = await invoke("get_cache_directory");
    } catch (e) {
      console.error("加载缓存目录失败", e);
    }
  }

  async function pickCacheDir() {
    try {
      const result: string | null = await invoke("pick_cache_directory");
      if (result) {
        await loadCacheDirectory();
      }
    } catch (e) {
      console.error("选择缓存目录失败", e);
    }
  }

  async function clearCache() {
    if (!confirm("确定要清除所有缓存文件吗？此操作不可恢复。")) {
      return;
    }
    try {
      await invoke("clear_cache");
      await loadCacheStats();
    } catch (e) {
      console.error("清除缓存失败", e);
    }
  }

  function handleDragStart(index: number) {
    dragIndex = index;
    isDragging = true;
  }

  function handleDragOver(index: number) {
    overIndex = index;
  }

  function handleDragLeave() {
    overIndex = null;
  }

  function handleDrop() {
    if (dragIndex === null || overIndex === null) return;

    const newOrder = [...playerOrder];
    const draggedItem = newOrder.splice(dragIndex, 1)[0];
    newOrder.splice(overIndex, 0, draggedItem);
    playerOrder = newOrder;

    const newWeights: Record<string, number> = {};
    newOrder.forEach((player, index) => {
      newWeights[player] = settings.player_weights[player] ?? 50;
    });

    saveSettings({ player_weights: newWeights });

    dragIndex = null;
    overIndex = null;
    isDragging = false;
  }

  async function updatePlayerWeight(player: string, delta: number) {
    const currentWeight = settings.player_weights[player] ?? 50;
    const newWeight = Math.max(0, Math.min(100, currentWeight + delta));
    const newWeights = { ...settings.player_weights, [player]: newWeight };
    await saveSettings({ player_weights: newWeights });
  }

  // 循环切换主题的辅助函数
  function cycleTheme() {
    const currentIndex = themes.findIndex(t => t.id === settings.island_theme);
    const nextIndex = (currentIndex + 1) % themes.length;
    saveSettings({ island_theme: themes[nextIndex].id });
  }
</script>

<div class="theradyme-app">
  <div class="bento-container">
    
    <header class="t-panel header-panel">
      <div class="brand">ISLAND_CTRL&reg;</div>
      <nav class="nav-links">
        <span>CAPABILITIES</span>
        <span>AUDIT SHIELD</span>
        <span>PRICING</span>
        <span class="nav-active">LOGIN</span>
      </nav>
      <div class="window-controls">
        <button onclick={async () => await appWindow.minimize()} title="最小化"><Minus size={14} color="#000" strokeWidth={3} /></button>
        <button onclick={async () => await appWindow.toggleMaximize()} title="最大化"><Maximize2 size={14} color="#000" strokeWidth={3} /></button>
        <button onclick={async () => await appWindow.close()} title="关闭"><X size={14} color="#000" strokeWidth={3} /></button>
      </div>
    </header>

    <section class="t-panel hero-panel bg-orange">
      <div class="panel-top-bar">
        <span>SYSTEM STATUS: ACTIVE</span>
        <span>UA 670-B</span>
      </div>
      
      <div class="hero-content">
        <h1 class="giant-text">AUTONOMOUS<br>ISLAND DEFENSE</h1>
        <p class="hero-subtext">
          Advanced algorithmic layout for desktop interfaces.<br>
          Minimize distraction. Maximize retention.<br>
          Zero human latency.
        </p>
      </div>

      <div class="hero-bottom-action">
        <span class="action-label">INITIALIZE THEME SEQUENCE</span>
        <div class="action-input-group">
          <div class="theme-readout">
            <span class="muted">CURRENT_THEME_ID:</span> 
            {themes.find(t => t.id === settings.island_theme)?.name || 'UNKNOWN'}
          </div>
          <button class="action-btn" onclick={cycleTheme}>ENGAGE &rarr;</button>
        </div>
        <div class="secure-line">
          <div class="barcode"></div>
          <span>SECURE CONNECTION ESTABLISHED</span>
        </div>
      </div>
    </section>

    <section class="t-panel risk-panel bg-gray">
      <span class="panel-label">PLAYER PRIORITY ASSESSMENT</span>
      <div class="risk-stat">
        <span class="risk-value">OPT.</span>
        <span class="risk-sub">Historical conflict avg: 2.4%</span>
      </div>
      <div class="risk-box">
        <span class="rb-label">ACTIVE SOURCES</span>
        <span class="rb-badge">{playerOrder.length}</span>
      </div>
      
      <div class="priority-list">
        <span class="list-title">WEIGHT ALLOCATION</span>
        {#each playerOrder as player, index}
          <div
            class="p-item"
            class:dragging={dragIndex === index}
            draggable="true"
            ondragstart={() => handleDragStart(index)}
            ondragover={(e) => { e.preventDefault(); handleDragOver(index); }}
            ondragleave={handleDragLeave}
            ondrop={handleDrop}
          >
            <div class="p-name">
              <span class="p-idx">[{index + 1}]</span> 
              {playerNames[player] || player}
            </div>
            <div class="p-controls">
              <button onclick={() => updatePlayerWeight(player, -10)}>-</button>
              <span class="p-weight">{settings.player_weights[player] ?? 50}</span>
              <button onclick={() => updatePlayerWeight(player, 10)}>+</button>
            </div>
          </div>
        {/each}
      </div>
    </section>

    <section class="t-panel live-panel bg-red">
      <div class="live-inner">
        <span class="live-label">CACHE PROCESSING</span>
        <span class="live-status">{cacheSize}</span>
        <div class="cache-actions">
          <button onclick={pickCacheDir}>DIR</button>
          <button onclick={clearCache}>PURGE</button>
        </div>
      </div>
    </section>

    <section class="t-panel ts26-panel bg-green">
      <div class="ts-left">
        <h2 class="giant-ts">DISP</h2>
        <div class="ts-line">MODULE_4</div>
      </div>
      <div class="ts-right toggles-container">
        <button class="retro-toggle" onclick={() => saveSettings({ show_spectrum: !settings.show_spectrum })}>
          <span>SPECTRUM</span>
          <div class="retro-checkbox" class:checked={settings.show_spectrum}></div>
        </button>
        <button class="retro-toggle" onclick={() => saveSettings({ auto_hide: !settings.auto_hide })}>
          <span>AUTO-HIDE</span>
          <div class="retro-checkbox" class:checked={settings.auto_hide}></div>
        </button>
        <button class="retro-toggle" onclick={() => saveSettings({ always_on_top: !settings.always_on_top })}>
          <span>TOPMOST</span>
          <div class="retro-checkbox" class:checked={settings.always_on_top}></div>
        </button>
      </div>
    </section>

    <section class="t-panel comp-panel bg-gray">
      <span class="panel-label">COVER ENGINE</span>
      <div class="comp-features">
        <div class="comp-toggles">
          <button class="retro-toggle" onclick={() => saveSettings({ enable_hd_cover: !settings.enable_hd_cover })}>
            <span>HD SYNC</span>
            <div class="retro-checkbox" class:checked={settings.enable_hd_cover}></div>
          </button>
          <button class="retro-toggle" onclick={() => saveSettings({ enable_pixel_art: !settings.enable_pixel_art })}>
            <span>PIXEL ART</span>
            <div class="retro-checkbox" class:checked={settings.enable_pixel_art}></div>
          </button>
          <button class="retro-toggle" onclick={() => saveSettings({ enable_mv_playback: !settings.enable_mv_playback })}>
            <span>MV STREAM</span>
            <div class="retro-checkbox" class:checked={settings.enable_mv_playback}></div>
          </button>
        </div>
        <div class="comp-icon">
          <div class="diamond">
            <span class="arrow">&#x21B1;</span>
          </div>
        </div>
      </div>
    </section>

    <section class="t-panel data-panel bg-yellow">
      <span class="panel-label">SYSTEM CONFIG</span>
      <div class="data-box">
        <span>SYS-V1.0</span>
      </div>
      <div class="toggles-container" style="margin-top: auto;">
        <button class="retro-toggle" onclick={() => saveSettings({ lock_floating_window: !settings.lock_floating_window })}>
          <span>LOCK WIN</span>
          <div class="retro-checkbox" class:checked={settings.lock_floating_window}></div>
        </button>
        <button class="retro-toggle" onclick={toggleAutoStart}>
          <span>AUTO BOOT</span>
          <div class="retro-checkbox" class:checked={settings.auto_start}></div>
        </button>
      </div>
    </section>

  </div>
</div>

<style>
  /* 基础重置 & 变量 */
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    /* 尽量匹配复古扩展字体和等宽字体 */
    font-family: 'Space Grotesk', 'Microgramma', 'Helvetica Neue', Helvetica, Arial, sans-serif;
    background: linear-gradient(135deg, #d3cbbd 0%, #a29b92 100%);
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    overflow: hidden;
  }

  :global(html) {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .theradyme-app {
    width: 100%;
    height: 100%;
    padding: 16px;
    display: flex;
    box-sizing: border-box;
    overflow: hidden;
  }

  /* 核心配色板 */
  .bg-orange { background-color: #efb574; }
  .bg-gray   { background-color: #c7c9cc; }
  .bg-red    { background-color: #c56d5e; }
  .bg-green  { background-color: #8bab93; }
  .bg-yellow { background-color: #f1b979; }

  /* 粗犷主义网格布局 - 4:3 比例 */
  .bento-container {
    width: 100%;
    height: 100%;
    background: #0f0f0f;
    border-radius: 20px;
    padding: 8px;
    display: grid;
    grid-template-columns: 4fr 3fr !important;
    grid-template-rows: auto 1fr auto !important;
    gap: 6px;
    box-shadow: 0 20px 50px rgba(0,0,0,0.5);
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .t-panel {
    border-radius: 12px;
    color: #111;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  /* 通用标签样式 */
  .panel-label {
    font-size: 9px;
    font-family: 'SF Mono', 'Courier New', monospace;
    text-transform: uppercase;
    font-weight: bold;
    letter-spacing: 0.5px;
    color: #333;
  }

  /* ==================== 区域分配 ==================== */
  .header-panel {
    grid-column: 1 / -1;
    grid-row: 1 / 2;
    background: #e6e8eb;
    border-radius: 18px 18px 14px 14px;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    padding: 12px 24px;
    overflow: hidden;
    min-height: 0;
  }

  .hero-panel {
    grid-column: 1 / 2;
    grid-row: 1 / 4;
    padding: 24px;
    overflow: hidden;
    min-height: 0;
  }

  .risk-panel {
    grid-column: 2 / 3;
    grid-row: 1 / 2;
    padding: 16px;
    overflow: hidden;
    min-height: 0;
  }

  .live-panel {
    grid-column: 3 / 4;
    grid-row: 3 / 4;
    padding: 20px;
    overflow: hidden;
    min-height: 0;
  }

  .ts26-panel {
    grid-column: 1 / 2;
    grid-row: 3 / 4;
    padding: 16px;
    flex-direction: row;
    gap: 16px;
    overflow: hidden;
    min-height: 0;
  }

  .comp-panel {
    grid-column: 2 / 3;
    grid-row: 3 / 4;
    padding: 16px;
    overflow: hidden;
    min-height: 0;
  }

  .data-panel {
    grid-column: 2 / 3;
    grid-row: 4 / 5;
    padding: 16px;
    overflow: hidden;
    min-height: 0;
  }

  /* ==================== 顶部导航 ==================== */
  .brand {
    font-weight: 800;
    font-size: clamp(12px, 1.5vw, 16px);
    letter-spacing: 1px;
  }

  .nav-links {
    display: flex;
    gap: clamp(16px, 3vw, 32px);
    font-family: 'SF Mono', 'Courier New', monospace;
    font-size: clamp(8px, 1vw, 10px);
    font-weight: bold;
  }

  .nav-active {
    border-bottom: 2px solid #000;
    padding-bottom: 2px;
  }

  .window-controls {
    display: flex;
    gap: clamp(8px, 1vw, 12px);
  }

  .window-controls button {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
  }

  /* ==================== HERO 左侧橙色区域 ==================== */
  .panel-top-bar {
    display: flex;
    justify-content: space-between;
    font-family: 'SF Mono', 'Courier New', monospace;
    font-size: clamp(7px, 0.9vw, 9px);
    font-weight: bold;
    border-bottom: 1px solid rgba(0,0,0,0.2);
    padding-bottom: clamp(4px, 0.5vw, 6px);
    margin-bottom: clamp(12px, 2vw, 24px);
  }

  .hero-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-height: 0;
    overflow: hidden;
  }

  .giant-text {
    font-size: clamp(1.2rem, 4vw, 3.8rem);
    font-weight: 600;
    line-height: 1.05;
    letter-spacing: -1px;
    margin-bottom: clamp(8px, 1.5vw, 16px);
    text-transform: uppercase;
  }

  .hero-subtext {
    font-family: 'SF Mono', 'Courier New', monospace;
    font-size: clamp(9px, 1.1vw, 11px);
    line-height: 1.7;
    border-left: 2px solid #111;
    padding-left: clamp(8px, 1.2vw, 12px);
    max-width: 80%;
    font-weight: 600;
  }

  .hero-bottom-action {
    margin-top: auto;
  }

  .action-label {
    font-family: 'SF Mono', 'Courier New', monospace;
    font-size: clamp(7px, 0.9vw, 9px);
    font-weight: bold;
    display: block;
    margin-bottom: clamp(4px, 0.5vw, 6px);
  }

  .action-input-group {
    display: flex;
    border: 1.5px solid #111;
    height: clamp(36px, 5vw, 48px);
    margin-bottom: clamp(4px, 0.8vw, 8px);
  }

  .theme-readout {
    flex: 1;
    display: flex;
    align-items: center;
    padding: 0 clamp(8px, 1.5vw, 16px);
    font-family: 'SF Mono', 'Courier New', monospace;
    font-size: clamp(10px, 1.2vw, 12px);
    font-weight: bold;
  }

  .theme-readout .muted {
    opacity: 0.5;
    margin-right: clamp(4px, 0.8vw, 8px);
    font-size: clamp(8px, 1vw, 10px);
  }

  .action-btn {
    background: #111;
    color: #efb574;
    border: none;
    padding: 0 clamp(12px, 2.5vw, 24px);
    font-family: 'SF Mono', 'Courier New', monospace;
    font-weight: bold;
    font-size: clamp(10px, 1.2vw, 12px);
    cursor: pointer;
  }

  .secure-line {
    display: flex;
    align-items: center;
    gap: clamp(8px, 1.2vw, 12px);
    font-family: 'SF Mono', 'Courier New', monospace;
    font-size: clamp(6px, 0.8vw, 8px);
    font-weight: bold;
  }

  .barcode {
    height: clamp(8px, 1.2vw, 12px);
    width: clamp(40px, 6vw, 60px);
    background: repeating-linear-gradient(
      90deg,
      #111,
      #111 2px,
      transparent 2px,
      transparent 4px
    );
  }

  /* ==================== RISK / PRIORITY 右上灰色 ==================== */
  .risk-stat {
    margin: clamp(8px, 1.5vw, 16px) 0;
  }
  .risk-value {
    font-size: clamp(20px, 3vw, 28px);
    font-weight: 600;
    display: block;
  }
  .risk-sub {
    font-family: 'SF Mono', 'Courier New', monospace;
    font-size: clamp(8px, 1vw, 10px);
    opacity: 0.7;
  }
  
  .risk-box {
    border: 1.5px solid #111;
    padding: 10px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
  }
  
  .rb-label { font-family: monospace; font-size: 10px; font-weight: bold;}
  .rb-badge { border: 1.5px solid #111; padding: 2px 8px; font-weight: 800; font-size: 12px; }

  .priority-list {
    margin-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
    overflow-y: auto;
  }

  .list-title {
    font-family: monospace;
    font-size: 9px;
    font-weight: bold;
    border-bottom: 1px solid #111;
    padding-bottom: 4px;
    margin-bottom: 8px;
  }

  .p-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: clamp(4px, 0.8vw, 8px) clamp(6px, 1vw, 10px);
    border-bottom: 1px dotted rgba(0,0,0,0.2);
    font-size: clamp(9px, 1.1vw, 11px);
  }
  .p-item.dragging { opacity: 0.4; }
  .p-idx { opacity: 0.5; margin-right: clamp(2px, 0.4vw, 4px); }

  .p-controls {
    display: flex;
    align-items: center;
    gap: clamp(4px, 0.8vw, 8px);
  }
  .p-controls button {
    width: clamp(18px, 2.2vw, 22px);
    height: clamp(18px, 2.2vw, 22px);
    border: 1px solid #111;
    background: transparent;
    font-size: clamp(12px, 1.4vw, 14px);
    cursor: pointer;
  }
  .p-weight {
    font-weight: 600;
    font-size: clamp(10px, 1.2vw, 12px);
    min-width: clamp(24px, 2.8vw, 28px);
    text-align: center;
  }

  /* ==================== LIVE PROCESSING 右中红色 ==================== */
  .live-panel {
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
  }
  .live-inner {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: clamp(8px, 1.2vw, 12px);
  }
  .live-label {
    font-size: clamp(7px, 0.9vw, 9px);
    font-weight: 600;
    letter-spacing: 1.5px;
  }
  .live-status {
    font-size: clamp(20px, 3vw, 28px);
    font-weight: 700;
    line-height: 1.2;
  }
  .cache-actions {
    display: flex;
    gap: clamp(4px, 0.8vw, 8px);
    margin-top: clamp(4px, 0.8vw, 8px);
  }
  .cache-actions button {
    flex: 1;
    padding: clamp(4px, 0.8vw, 8px) clamp(8px, 1.2vw, 12px);
    border: 1.5px solid #111;
    background: transparent;
    font-family: monospace;
    font-size: clamp(8px, 1vw, 10px);
    font-weight: 600;
    cursor: pointer;
    transition: 0.2s;
  }
  .cache-actions button:hover {
    background: #111;
    color: #fff;
  }

  /* ==================== TS26 左下绿色 ==================== */
  .ts-left {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    min-height: 0;
    overflow: hidden;
  }

  .giant-ts {
    font-size: clamp(24px, 3.5vw, 36px);
    font-weight: 600;
    line-height: 1;
    letter-spacing: -1px;
    margin-bottom: clamp(4px, 0.6vw, 6px);
  }

  .ts-line {
    font-family: monospace;
    font-size: clamp(7px, 0.9vw, 9px);
    font-weight: bold;
    border-bottom: 1.5px solid #111;
    padding-bottom: 4px;
  }

  .ts-right {
    flex: 1.5;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    padding-left: clamp(8px, 1.5vw, 16px);
    min-height: 0;
    overflow: hidden;
  }

  .toggles-container {
    display: flex;
    flex-direction: column;
    gap: clamp(4px, 0.6vw, 6px);
  }

  .retro-toggle {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: transparent;
    border: none;
    font-family: monospace;
    font-size: clamp(8px, 1vw, 10px);
    font-weight: bold;
    color: #111;
    cursor: pointer;
    padding: clamp(2px, 0.4vw, 4px) 0;
    border-bottom: 1px dotted rgba(0,0,0,0.3);
    width: 100%;
  }

  .retro-checkbox {
    width: clamp(10px, 1.2vw, 12px);
    height: clamp(10px, 1.2vw, 12px);
    border: 1.5px solid #111;
    background: transparent;
    transition: 0.2s;
  }

  .retro-checkbox.checked {
    background: #111;
    box-shadow: inset 0 0 0 2px var(--bg-color, transparent);
  }

  .ts26-panel .retro-checkbox.checked {
    box-shadow: inset 0 0 0 2px #8bab93;
  }
</style>