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
    enable_halftone: boolean;
    auto_start: boolean;
    hide_settings_button: boolean;
    hide_monitor_selector: boolean;
    hide_floating_window: boolean;
    expanded_corner_radius: number;
    always_show_top_bar: boolean;
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
    enable_halftone: false,
    auto_start: false,
    hide_settings_button: false,
    hide_monitor_selector: false,
    hide_floating_window: false,
    expanded_corner_radius: 16,
    always_show_top_bar: true,
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
      id: "liquid_glass",
      name: "液体玻璃",
      color: "#0f0f0f",
      gradient: "linear-gradient(135deg, #0f0f0f 0%, #1a1a1a 100%)",
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

  function cycleTheme() {
    const currentIndex = themes.findIndex(
      (t) => t.id === settings.island_theme,
    );
    const nextIndex = (currentIndex + 1) % themes.length;
    saveSettings({ island_theme: themes[nextIndex].id });
  }
</script>

<div class="theradyme-app">
  <div class="bento-container">
    <header class="t-panel header-panel" data-tauri-drag-region>
      <div class="brand">ISLAND_CTRL&reg;</div>
      <div class="window-controls">
        <button onclick={async () => await appWindow.minimize()} title="最小化"
          ><Minus size={14} color="#000" strokeWidth={3} /></button
        >
        <button
          onclick={async () => await appWindow.toggleMaximize()}
          title="最大化"
          ><Maximize2 size={14} color="#000" strokeWidth={3} /></button
        >
        <button onclick={async () => await appWindow.close()} title="关闭"
          ><X size={14} color="#000" strokeWidth={3} /></button
        >
      </div>
    </header>

    <section class="t-panel hero-panel bg-orange">
      <div class="panel-top-bar">
        <span>系统状态：运行中</span>
        <span>UA 670-B</span>
      </div>

      <div class="hero-content">
        <h1 class="giant-text">灵动岛<br />控制中心</h1>
        <p class="hero-subtext">
          智能桌面媒体交互体验。<br />
          实时音乐播放监控。<br />
          多播放器智能切换。
        </p>
      </div>

      <div class="hero-bottom-action">
        <span class="action-label">主题切换</span>
        <div class="action-input-group">
          <div class="theme-readout">
            <span class="muted">当前主题:</span>
            {themes.find((t) => t.id === settings.island_theme)?.name || "未知"}
          </div>
          <button class="action-btn" onclick={cycleTheme}>切换 &rarr;</button>
        </div>
        <div class="secure-line">
          <div class="barcode"></div>
          <span>系统运行正常</span>
        </div>
      </div>
    </section>

    <section class="t-panel risk-panel bg-gray">
      <span class="panel-label">播放器管理</span>

      <div class="risk-stat">
        <span class="risk-value">优先</span>
        <span class="risk-sub">智能排序引擎</span>
      </div>
      <div class="risk-box">
        <span class="rb-label">已配置项</span>
        <span class="rb-badge">{playerOrder.length}</span>
      </div>

      <div class="priority-list">
        <span class="list-title">优先级列表</span>
        {#each playerOrder as player, index}
          <div
            class="p-item"
            class:dragging={dragIndex === index}
            draggable="true"
            ondragstart={() => handleDragStart(index)}
            ondragover={(e) => {
              e.preventDefault();
              handleDragOver(index);
            }}
            ondragleave={handleDragLeave}
            ondrop={handleDrop}
          >
            <div class="p-name">
              <span class="p-idx">[{index + 1}]</span>
              {playerNames[player] || player}
            </div>
            <div class="p-controls">
              <button onclick={() => updatePlayerWeight(player, -10)}>-</button>
              <span class="p-weight"
                >{settings.player_weights[player] ?? 50}</span
              >
              <button onclick={() => updatePlayerWeight(player, 10)}>+</button>
            </div>
          </div>
        {/each}
      </div>
    </section>

    <section class="t-panel ts26-panel bg-green">
      <div class="ts-left">
        <h2 class="giant-ts" style="font-size: 24px;">系统</h2>
        <div class="ts-line">OS 级别交互</div>
      </div>
      <div class="toggles-container" style="margin-top: auto;">
        <button class="retro-toggle" onclick={toggleAutoStart}>
          <span>开机自启</span>
          <div class="retro-checkbox" class:checked={settings.auto_start}></div>
        </button>
        <button
          class="retro-toggle"
          onclick={() => saveSettings({ auto_hide: !settings.auto_hide })}
        >
          <span>自动隐藏</span>
          <div class="retro-checkbox" class:checked={settings.auto_hide}></div>
        </button>
      </div>
    </section>

    <section class="t-panel comp-panel bg-gray">
      <span class="panel-label">媒体视觉</span>
      <div class="comp-features">
        <div class="comp-toggles">
          <button
            class="retro-toggle"
            onclick={() =>
              saveSettings({ enable_hd_cover: !settings.enable_hd_cover })}
          >
            <span>高清封面</span>
            <div
              class="retro-checkbox"
              class:checked={settings.enable_hd_cover}
            ></div>
          </button>
          <button
            class="retro-toggle"
            onclick={() =>
              saveSettings({ enable_pixel_art: !settings.enable_pixel_art })}
          >
            <span>像素风格</span>
            <div
              class="retro-checkbox"
              class:checked={settings.enable_pixel_art}
            ></div>
          </button>
          <button
            class="retro-toggle"
            onclick={() =>
              saveSettings({ enable_halftone: !settings.enable_halftone })}
          >
            <span>网点效果</span>
            <div
              class="retro-checkbox"
              class:checked={settings.enable_halftone}
            ></div>
          </button>
          <button
            class="retro-toggle"
            onclick={() =>
              saveSettings({
                enable_mv_playback: !settings.enable_mv_playback,
              })}
          >
            <span>MV 引擎</span>
            <div
              class="retro-checkbox"
              class:checked={settings.enable_mv_playback}
            ></div>
          </button>
        </div>
      </div>
    </section>

    <section class="t-panel window-panel bg-yellow">
      <span class="panel-label">悬浮窗功能</span>
      <div class="wide-toggles">
        <button
          class="retro-toggle"
          onclick={() =>
            saveSettings({ always_on_top: !settings.always_on_top })}
        >
          <span>窗口置顶</span>
          <div
            class="retro-checkbox"
            class:checked={settings.always_on_top}
          ></div>
        </button>
        <button
          class="retro-toggle"
          onclick={() =>
            saveSettings({
              lock_floating_window: !settings.lock_floating_window,
            })}
        >
          <span>锁定悬浮窗</span>
          <div
            class="retro-checkbox"
            class:checked={settings.lock_floating_window}
          ></div>
        </button>
        <button
          class="retro-toggle"
          onclick={() =>
            saveSettings({
              always_show_top_bar: !settings.always_show_top_bar,
            })}
        >
          <span>固定顶部栏</span>
          <div
            class="retro-checkbox"
            class:checked={settings.always_show_top_bar}
          ></div>
        </button>
        <button
          class="retro-toggle"
          onclick={() =>
            saveSettings({
              hide_floating_window: !settings.hide_floating_window,
            })}
        >
          <span>隐藏悬浮窗</span>
          <div
            class="retro-checkbox"
            class:checked={settings.hide_floating_window}
          ></div>
        </button>
      </div>
    </section>

    <section
      class="t-panel island-panel bg-orange"
      style="background-color: #e2a878;"
    >
      <span class="panel-label">灵动岛功能</span>
      <div class="wide-toggles">
        <button
          class="retro-toggle"
          onclick={() =>
            saveSettings({
              hide_settings_button: !settings.hide_settings_button,
            })}
        >
          <span>隐藏设置按钮</span>
          <div
            class="retro-checkbox"
            class:checked={settings.hide_settings_button}
          ></div>
        </button>
        <button
          class="retro-toggle"
          onclick={() =>
            saveSettings({
              hide_monitor_selector: !settings.hide_monitor_selector,
            })}
        >
          <span>隐藏显示器按钮</span>
          <div
            class="retro-checkbox"
            class:checked={settings.hide_monitor_selector}
          ></div>
        </button>

        <div class="corner-radius-control">
          <span class="corner-label">展开圆角</span>
          <input
            type="range"
            min="0"
            max="32"
            value={settings.expanded_corner_radius}
            oninput={(e) => {
              const value = parseInt(e.currentTarget.value);
              settings.expanded_corner_radius = value;
              invoke("set_expanded_corner_radius", { radius: value });
            }}
            class="corner-slider"
          />
          <span class="corner-value">{settings.expanded_corner_radius}px</span>
        </div>
      </div>
    </section>

    <section class="t-panel live-panel bg-red">
      <div class="live-inner">
        <span class="live-label">系统缓存</span>
        <span class="live-status">{cacheSize}</span>
        <div class="cache-actions">
          <button onclick={pickCacheDir}>映射路径</button>
          <button onclick={clearCache}>清空冗余</button>
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  /* 全局重置与背景 */
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body),
  :global(html) {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
  }

  .theradyme-app {
    width: 100vw;
    height: 100vh;
    display: flex;
    background: transparent;
  }

  /* 核心配色板保持不变 */
  .bg-orange {
    background-color: #efb574;
  }
  .bg-gray {
    background-color: #c7c9cc;
  }
  .bg-red {
    background-color: #c56d5e;
  }
  .bg-green {
    background-color: #8bab93;
  }
  .bg-yellow {
    background-color: #f1b979;
  }

  /* * [修改重点]：彻底重构底层网格，实现内容决定的参差排版
   * 使用 4列 基础网格搭配 grid-auto-flow: dense;
   */
  .bento-container {
    width: 100%;
    height: 100%;
    background: #0f0f0f;
    border-radius: 12px;
    padding: 10px;
    display: grid;
    grid-template-columns: repeat(4, 1fr); /* 4列等分底座 */
    grid-auto-rows: minmax(min-content, max-content);
    grid-auto-flow: dense; /* 核心：允许错落排列填补空隙 */
    gap: 10px;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
    overflow-y: auto; /* 允许滚动 */
    overflow-x: hidden;
  }

  /* 面板基础样式保留 */
  .t-panel {
    color: #111;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  /* * [修改重点]：利用 grid-column/row 的 span 跨度重塑排版
   * 制造极其规整又参差错落的拼图感
   */
  .header-panel {
    grid-column: span 4;
    background: #e6e8eb;
    border-radius: 18px 18px 14px 14px;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
  }

  .hero-panel {
    grid-column: span 2;
    grid-row: span 2; /* 巨无霸方块 */
    padding: 22px;
    border-radius: 16px 12px 14px 18px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  }

  .risk-panel {
    grid-column: span 1;
    grid-row: span 2; /* 高挑细长方块 */
    padding: 14px;
    border-radius: 14px 16px 12px 14px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.12);
  }

  .ts26-panel {
    grid-column: span 1;
    grid-row: span 1; /* 小正块 */
    padding: 14px;
    flex-direction: column;
    border-radius: 12px 14px 16px 12px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  }

  .comp-panel {
    grid-column: span 1;
    grid-row: span 1; /* 小正块 */
    padding: 14px;
    border-radius: 15px 13px 15px 13px;
    box-shadow: 0 7px 22px rgba(0, 0, 0, 0.13);
  }

  .window-panel {
    grid-column: span 2; /* 宽横条 */
    padding: 14px;
    border-radius: 13px 15px 11px 16px;
    box-shadow: 0 5px 18px rgba(0, 0, 0, 0.11);
  }

  .island-panel {
    grid-column: span 2; /* 宽横条 */
    padding: 14px;
    border-radius: 16px 11px 14px 12px;
    box-shadow: 0 6px 15px rgba(0, 0, 0, 0.1);
  }

  .live-panel {
    grid-column: span 4; /* 底座横栏 */
    padding: 8px 12px;
    border-radius: 10px 10px 12px 12px;
    box-shadow: 0 3px 12px rgba(0, 0, 0, 0.08);
  }

  /* ==================== 局部样式微调与兼容 ==================== */
  .brand {
    font-weight: 800;
    font-size: clamp(12px, 1.5vw, 16px);
    letter-spacing: 1px;
  }

  .window-controls {
    display: flex;
    gap: clamp(8px, 1vw, 12px);
    -webkit-app-region: no-drag;
  }

  .window-controls button {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    transition: opacity 0.2s;
  }
  .window-controls button:hover {
    opacity: 0.5;
  }

  .panel-top-bar {
    display: flex;
    justify-content: space-between;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: clamp(7px, 0.9vw, 9px);
    font-weight: bold;
    border-bottom: 1px solid rgba(0, 0, 0, 0.2);
    padding-bottom: clamp(4px, 0.5vw, 6px);
    margin-bottom: clamp(12px, 2vw, 24px);
  }

  .hero-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-height: 0;
  }

  .giant-text {
    font-size: clamp(1.3rem, 3vw, 3.2rem);
    font-weight: 700;
    line-height: 1.05;
    letter-spacing: -1px;
    margin-bottom: 12px;
    text-transform: uppercase;
  }

  .hero-subtext {
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: clamp(10px, 1.1vw, 12px);
    line-height: 1.7;
    border-left: 2px solid #111;
    padding-left: clamp(8px, 1.2vw, 12px);
    max-width: 85%;
    font-weight: 600;
  }

  .hero-bottom-action {
    margin-top: auto;
  }

  .action-label {
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
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
    border-radius: 4px;
    overflow: hidden;
  }

  .theme-readout {
    flex: 1;
    display: flex;
    align-items: center;
    padding: 0 clamp(8px, 1.5vw, 16px);
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
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
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-weight: bold;
    font-size: clamp(10px, 1.2vw, 12px);
    cursor: pointer;
    transition: filter 0.2s;
  }
  .action-btn:hover {
    filter: brightness(1.2);
  }

  .secure-line {
    display: flex;
    align-items: center;
    gap: clamp(8px, 1.2vw, 12px);
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
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

  /* 面板内部细节保留 */
  .panel-label {
    font-size: 8px;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    text-transform: uppercase;
    font-weight: bold;
    letter-spacing: 0.5px;
    color: #333;
    margin-bottom: clamp(4px, 0.6vw, 6px);
  }

  .risk-stat {
    margin: clamp(4px, 1vw, 8px) 0;
  }
  .risk-value {
    font-size: clamp(18px, 2.5vw, 24px);
    font-weight: 700;
    display: block;
  }
  .risk-sub {
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: clamp(8px, 1vw, 10px);
    opacity: 0.7;
  }
  .risk-box {
    border: 1.5px solid #111;
    padding: 6px 10px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  .rb-label {
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: 10px;
    font-weight: bold;
  }
  .rb-badge {
    border: 1.5px solid #111;
    padding: 2px 8px;
    font-weight: 800;
    font-size: 12px;
  }

  .priority-list {
    margin-top: 4px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    overflow-y: auto;
    padding-right: 4px;
  }
  .priority-list::-webkit-scrollbar {
    width: 4px;
  }
  .priority-list::-webkit-scrollbar-track {
    background: transparent;
  }
  .priority-list::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 4px;
  }

  .list-title {
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: 9px;
    font-weight: bold;
    border-bottom: 1px solid #111;
    padding-bottom: 4px;
    margin-bottom: 4px;
  }

  .p-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: clamp(3px, 0.6vw, 6px) clamp(5px, 0.8vw, 8px);
    border-bottom: 1px dotted rgba(0, 0, 0, 0.2);
    font-size: clamp(9px, 1vw, 11px);
    transition: background 0.2s;
  }
  .p-item:hover {
    background: rgba(0, 0, 0, 0.05);
  }
  .p-item.dragging {
    opacity: 0.4;
  }
  .p-idx {
    opacity: 0.5;
    margin-right: clamp(2px, 0.4vw, 4px);
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
  }
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
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .p-controls button:active {
    background: #111;
    color: #fff;
  }
  .p-weight {
    font-weight: 700;
    font-size: clamp(10px, 1.2vw, 12px);
    min-width: clamp(24px, 2.8vw, 28px);
    text-align: center;
  }

  /* --- 修改重点：为新划分的宽体区域实现密集的二级排列 --- */
  .toggles-container,
  .comp-toggles {
    display: flex;
    flex-direction: column;
    gap: clamp(3px, 0.6vw, 6px);
    margin-top: auto;
  }

  .toggles-container .retro-toggle span {
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: clamp(10px, 1.2vw, 12px);
    font-weight: bold;
  }

  .wide-toggles {
    display: grid;
    grid-template-columns: 1fr 1fr; /* 两列排布按钮，让横向的 Panel 显得信息完整充实 */
    gap: 4px 16px;
    margin-top: auto;
  }

  .retro-toggle {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: transparent;
    border: none;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: clamp(9px, 1.1vw, 11px);
    font-weight: bold;
    color: #111;
    cursor: pointer;
    padding: clamp(3px, 0.5vw, 6px) 0;
    border-bottom: 1px dotted rgba(0, 0, 0, 0.3);
    width: 100%;
    transition: opacity 0.2s;
  }
  .retro-toggle:hover {
    opacity: 0.6;
  }

  .retro-checkbox {
    width: clamp(12px, 1.4vw, 14px);
    height: clamp(12px, 1.4vw, 14px);
    border: 1.5px solid #111;
    background: transparent;
    transition: 0.2s;
  }
  .retro-checkbox.checked {
    background: #111;
    box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.8);
  }

  /* 基础系统区调整 */
  .ts-left {
    flex: none;
    margin-bottom: 8px;
  }
  .giant-ts {
    font-size: clamp(20px, 3vw, 26px);
    font-weight: 700;
    line-height: 1;
    letter-spacing: -1px;
    margin-bottom: 4px;
  }
  .ts-line {
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: clamp(7px, 0.9vw, 9px);
    font-weight: bold;
    border-bottom: 1.5px solid #111;
    padding-bottom: 4px;
  }

  /* 滑动条保持不变 */
  .corner-radius-control {
    display: flex;
    align-items: center;
    gap: clamp(5px, 0.8vw, 8px);
    padding: clamp(3px, 0.5vw, 6px) 0;
    border-bottom: 1px dotted rgba(0, 0, 0, 0.3);
    width: 100%;
  }
  .corner-label {
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: clamp(9px, 1.1vw, 11px);
    font-weight: bold;
    color: #111;
    min-width: 55px;
  }
  .corner-slider {
    flex: 1;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: rgba(17, 17, 17, 0.2);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .corner-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: clamp(12px, 1.5vw, 16px);
    height: clamp(12px, 1.5vw, 16px);
    background: #111;
    border-radius: 50%;
    cursor: pointer;
    transition: transform 0.15s ease;
  }
  .corner-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }
  .corner-value {
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: clamp(9px, 1.1vw, 11px);
    font-weight: bold;
    color: #111;
    min-width: 40px;
    text-align: right;
  }

  .live-inner {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    gap: clamp(8px, 1.2vw, 16px);
  }
  .live-label {
    font-size: clamp(7px, 0.9vw, 9px);
    font-weight: 700;
    letter-spacing: 1.5px;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
  }
  .live-status {
    font-size: clamp(12px, 1.5vw, 18px);
    font-weight: 700;
    line-height: 1.2;
  }
  .cache-actions {
    display: flex;
    gap: clamp(5px, 0.7vw, 8px);
    width: auto;
  }
  .cache-actions button {
    padding: clamp(3px, 0.6vw, 6px) 12px;
    border: 1.5px solid #111;
    background: transparent;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
    font-size: clamp(10px, 1.2vw, 12px);
    font-weight: 700;
    cursor: pointer;
    transition: 0.2s;
  }
  .cache-actions button:hover {
    background: #111;
    color: #fff;
  }
</style>
