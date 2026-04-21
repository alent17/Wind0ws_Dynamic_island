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
    { id: "original", name: "经典黑", color: "#1a1a1a" },
    { id: "liquid_glass", name: "液体玻璃", color: "#0f0f0f" },
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
      if (saved.player_weights) settings.player_weights = saved.player_weights;
      try {
        settings.auto_start = await invoke<boolean>("get_auto_start");
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
    } catch (e) {
      console.error("保存设置失败", e);
    }
  }

  async function toggleAutoStart() {
    try {
      const s = !settings.auto_start;
      await invoke("set_auto_start", { enable: s });
      settings.auto_start = s;
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
      const r: string | null = await invoke("pick_cache_directory");
      if (r) await loadCacheDirectory();
    } catch (e) {
      console.error("选择缓存目录失败", e);
    }
  }

  async function clearCache() {
    if (!confirm("确定要清除所有缓存文件吗？此操作不可恢复。")) return;
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
    const o = [...playerOrder];
    const item = o.splice(dragIndex, 1)[0];
    o.splice(overIndex, 0, item);
    playerOrder = o;
    const w: Record<string, number> = {};
    o.forEach((p) => (w[p] = settings.player_weights[p] ?? 50));
    saveSettings({ player_weights: w });
    dragIndex = null;
    overIndex = null;
    isDragging = false;
  }

  async function updatePlayerWeight(player: string, delta: number) {
    const c = settings.player_weights[player] ?? 50;
    const n = Math.max(0, Math.min(100, c + delta));
    await saveSettings({
      player_weights: { ...settings.player_weights, [player]: n },
    });
  }

  function cycleTheme() {
    const i = themes.findIndex((t) => t.id === settings.island_theme);
    saveSettings({ island_theme: themes[(i + 1) % themes.length].id });
  }
</script>

<div class="app">
  <div class="grid">
    <!-- ═══════════ Header ═══════════ -->
    <header class="panel hdr" data-tauri-drag-region>
      <div class="hdr-brand">
        <span class="hdr-mark">◉</span> ISLAND_CTRL
      </div>
      <div class="hdr-btns">
        <button onclick={() => appWindow.minimize()} title="最小化">
          <Minus size={13} />
        </button>
        <button onclick={() => appWindow.toggleMaximize()} title="最大化">
          <Maximize2 size={13} />
        </button>
        <button class="close" onclick={() => appWindow.close()} title="关闭">
          <X size={13} />
        </button>
      </div>
    </header>

    <!-- ═══════════ Hero ═══════════ -->
    <section class="panel hero">
      <div class="hero-status">
        <span class="status-dot"></span>
        <span>系统运行中</span>
        <span class="sep">·</span>
        <span>UA 670-B</span>
      </div>
      <div class="hero-body">
        <h1>灵动岛<br />控制中心</h1>
        <p>
          智能桌面媒体交互体验<br />
          实时音乐播放监控 · 多播放器智能切换
        </p>
      </div>
      <div class="hero-foot">
        <span class="label">主题选择</span>
        <div class="theme-opts">
          {#each themes as t}
            <button
              class="theme-card"
              class:active={settings.island_theme === t.id}
              onclick={() => saveSettings({ island_theme: t.id })}
            >
              <div class="theme-swatch" style="background:{t.color}"></div>
              <span>{t.name}</span>
              {#if settings.island_theme === t.id}
                <Check size={11} />
              {/if}
            </button>
          {/each}
        </div>
      </div>
    </section>

    <!-- ═══════════ Player Manager ═══════════ -->
    <section class="panel pm">
      <div class="pm-head">
        <div>
          <span class="label">播放器管理</span>
          <span class="pm-num">{playerOrder.length}</span>
        </div>
        <span class="pm-sub">拖拽排序 · 权重调节</span>
      </div>
      <div class="pm-list">
        {#each playerOrder as player, index}
          <div
            class="pm-row"
            class:dragging={dragIndex === index}
            class:over={overIndex === index}
            draggable="true"
            ondragstart={() => handleDragStart(index)}
            ondragover={(e) => {
              e.preventDefault();
              handleDragOver(index);
            }}
            ondragleave={handleDragLeave}
            ondrop={handleDrop}
          >
            <div class="pm-info">
              <span class="pm-idx">{String(index + 1).padStart(2, "0")}</span>
              <span class="pm-name">{playerNames[player] || player}</span>
            </div>
            <div class="pm-ctrl">
              <button onclick={() => updatePlayerWeight(player, -10)}>−</button>
              <span class="pm-w">{settings.player_weights[player] ?? 50}</span>
              <button onclick={() => updatePlayerWeight(player, 10)}>+</button>
            </div>
          </div>
        {/each}
      </div>
    </section>

    <!-- ═══════════ System ═══════════ -->
    <section class="panel sys">
      <span class="label">系统</span>
      <div class="toggles">
        <button class="tg" onclick={toggleAutoStart}>
          <span>开机自启</span>
          <div class="sw" class:on={settings.auto_start}></div>
        </button>
        <button
          class="tg"
          onclick={() => saveSettings({ auto_hide: !settings.auto_hide })}
        >
          <span>自动隐藏</span>
          <div class="sw" class:on={settings.auto_hide}></div>
        </button>
      </div>
    </section>

    <!-- ═══════════ Visual Effects ═══════════ -->
    <section class="panel vis">
      <span class="label">媒体视觉</span>
      <div class="toggles">
        <button
          class="tg"
          onclick={() =>
            saveSettings({ enable_hd_cover: !settings.enable_hd_cover })}
        >
          <span>高清封面</span>
          <div class="sw" class:on={settings.enable_hd_cover}></div>
        </button>
        <button
          class="tg"
          onclick={() =>
            saveSettings({ enable_pixel_art: !settings.enable_pixel_art })}
        >
          <span>像素风格</span>
          <div class="sw" class:on={settings.enable_pixel_art}></div>
        </button>
        <button
          class="tg"
          onclick={() =>
            saveSettings({ enable_halftone: !settings.enable_halftone })}
        >
          <span>网点效果</span>
          <div class="sw" class:on={settings.enable_halftone}></div>
        </button>
        <button
          class="tg"
          onclick={() =>
            saveSettings({ enable_mv_playback: !settings.enable_mv_playback })}
        >
          <span>MV 引擎</span>
          <div class="sw" class:on={settings.enable_mv_playback}></div>
        </button>
      </div>
    </section>

    <!-- ═══════════ Floating Window ═══════════ -->
    <section class="panel flt">
      <span class="label">悬浮窗</span>
      <div class="toggles">
        <button
          class="tg"
          onclick={() =>
            saveSettings({ always_on_top: !settings.always_on_top })}
        >
          <span>窗口置顶</span>
          <div class="sw" class:on={settings.always_on_top}></div>
        </button>
        <button
          class="tg"
          onclick={() =>
            saveSettings({
              lock_floating_window: !settings.lock_floating_window,
            })}
        >
          <span>锁定位置</span>
          <div class="sw" class:on={settings.lock_floating_window}></div>
        </button>
        <button
          class="tg"
          onclick={() =>
            saveSettings({
              always_show_top_bar: !settings.always_show_top_bar,
            })}
        >
          <span>固定顶栏</span>
          <div class="sw" class:on={settings.always_show_top_bar}></div>
        </button>
        <button
          class="tg"
          onclick={() =>
            saveSettings({
              hide_floating_window: !settings.hide_floating_window,
            })}
        >
          <span>隐藏窗口</span>
          <div class="sw" class:on={settings.hide_floating_window}></div>
        </button>
      </div>
    </section>

    <!-- ═══════════ Island ═══════════ -->
    <section class="panel isl">
      <span class="label">灵动岛</span>
      <div class="toggles">
        <button
          class="tg"
          onclick={() =>
            saveSettings({
              hide_settings_button: !settings.hide_settings_button,
            })}
        >
          <span>隐藏设置按钮</span>
          <div class="sw" class:on={settings.hide_settings_button}></div>
        </button>
        <button
          class="tg"
          onclick={() =>
            saveSettings({
              hide_monitor_selector: !settings.hide_monitor_selector,
            })}
        >
          <span>隐藏显示器按钮</span>
          <div class="sw" class:on={settings.hide_monitor_selector}></div>
        </button>
        <div class="slider-row">
          <span>展开圆角</span>
          <input
            type="range"
            min="0"
            max="32"
            value={settings.expanded_corner_radius}
            oninput={(e) => {
              const v = parseInt(e.currentTarget.value);
              settings.expanded_corner_radius = v;
              invoke("set_expanded_corner_radius", { radius: v });
            }}
          />
          <span class="sv">{settings.expanded_corner_radius}px</span>
        </div>
      </div>
    </section>

    <!-- ═══════════ Cache ═══════════ -->
    <section class="panel cache">
      <div class="cache-in">
        <div>
          <span class="label">系统缓存</span>
          <span class="cache-sz">{cacheSize}</span>
        </div>
        <div class="cache-btns">
          <button onclick={pickCacheDir}>映射路径</button>
          <button class="warn" onclick={clearCache}>清空缓存</button>
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  @import url("https://fonts.googleapis.com/css2?family=DM+Mono:wght@300;400;500&family=Instrument+Serif:ital@0;1&display=swap");

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
    font-family: "DM Mono", monospace;
    background: #09090b;
  }

  :root {
    --bg: #09090b;
    --surf: #141416;
    --surf2: #1c1c1f;
    --surf3: #242427;
    --amber: #d4a057;
    --amber-d: #a07830;
    --amber-g: rgba(212, 160, 87, 0.12);
    --txt: #e8e4de;
    --txt2: #8a8580;
    --txt3: #4a4744;
    --red: #c06050;
    --green: #5a9a6a;
    --brd: rgba(255, 255, 255, 0.05);
    --brd2: rgba(255, 255, 255, 0.08);
  }

  /* ─── App ─── */
  .app {
    width: 100vw;
    height: 100vh;
    padding: 12px;
    background: radial-gradient(
        ellipse at 15% 10%,
        rgba(212, 160, 87, 0.04) 0%,
        transparent 50%
      ),
      radial-gradient(
        ellipse at 85% 90%,
        rgba(90, 100, 120, 0.03) 0%,
        transparent 50%
      ),
      var(--bg);
  }

  .grid {
    width: 100%;
    height: 100%;
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    grid-auto-rows: min-content;
    gap: 10px;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: var(--surf3) transparent;
  }

  /* ─── Panel Base ─── */
  .panel {
    background: var(--surf);
    border: 1px solid var(--brd);
    border-radius: 14px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    transition: border-color 0.3s ease;
    animation: rise 0.5s ease both;
  }
  .panel:hover {
    border-color: var(--brd2);
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .label {
    font-size: 10px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 2px;
    color: var(--txt3);
    display: block;
    margin-bottom: 14px;
  }

  /* ─── Header ─── */
  .hdr {
    grid-column: 1 / -1;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    border-radius: 12px;
    animation-delay: 0s;
  }
  .hdr-brand {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 3px;
    color: var(--txt2);
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .hdr-mark {
    color: var(--amber);
    font-size: 14px;
  }
  .hdr-btns {
    display: flex;
    gap: 2px;
    -webkit-app-region: no-drag;
  }
  .hdr-btns button {
    background: none;
    border: none;
    color: var(--txt3);
    cursor: pointer;
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    transition: all 0.15s;
  }
  .hdr-btns button:hover {
    background: var(--surf2);
    color: var(--txt);
  }
  .hdr-btns .close:hover {
    background: rgba(192, 96, 80, 0.15);
    color: var(--red);
  }

  /* ─── Hero ─── */
  .hero {
    background: radial-gradient(
        ellipse at 20% 80%,
        rgba(212, 160, 87, 0.08) 0%,
        transparent 50%
      ),
      var(--surf);
    padding: 24px;
    animation-delay: 0.05s;
  }
  .hero-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 10px;
    color: var(--txt3);
    letter-spacing: 0.5px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--brd);
    margin-bottom: 20px;
  }
  .status-dot {
    width: 6px;
    height: 6px;
    background: var(--green);
    border-radius: 50%;
    animation: pulse 2s ease infinite;
  }
  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }
  .sep {
    opacity: 0.3;
  }

  .hero-body {
    flex: 1;
  }
  .hero-body h1 {
    font-family: "Instrument Serif", serif;
    font-size: clamp(28px, 4vw, 42px);
    font-weight: 400;
    line-height: 1.1;
    letter-spacing: -1px;
    color: var(--txt);
    margin-bottom: 12px;
  }
  .hero-body p {
    font-size: 12px;
    line-height: 1.8;
    color: var(--txt2);
    max-width: 320px;
  }

  .hero-foot {
    margin-top: 24px;
  }
  .hero-foot .label {
    margin-bottom: 10px;
  }
  .theme-opts {
    display: flex;
    gap: 8px;
  }
  .theme-card {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--surf2);
    border: 1px solid var(--brd);
    border-radius: 10px;
    color: var(--txt2);
    font-family: inherit;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.2s;
  }
  .theme-card:hover {
    border-color: rgba(255, 255, 255, 0.1);
    color: var(--txt);
  }
  .theme-card.active {
    border-color: var(--amber);
    background: var(--amber-g);
    color: var(--amber);
  }
  .theme-swatch {
    width: 14px;
    height: 14px;
    border-radius: 4px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    flex-shrink: 0;
  }

  /* ─── Player Manager ─── */
  .pm {
    grid-row: span 2;
    padding: 20px;
    animation-delay: 0.1s;
  }
  .pm-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: 16px;
  }
  .pm-head > div:first-child {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .pm-head .label {
    margin-bottom: 0;
  }
  .pm-num {
    font-family: "Instrument Serif", serif;
    font-size: 28px;
    color: var(--amber);
    line-height: 1;
  }
  .pm-sub {
    font-size: 10px;
    color: var(--txt3);
  }
  .pm-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    scrollbar-width: thin;
    scrollbar-color: var(--surf3) transparent;
  }
  .pm-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    border-radius: 8px;
    cursor: grab;
    transition: all 0.15s;
    border: 1px solid transparent;
  }
  .pm-row:hover {
    background: var(--surf2);
  }
  .pm-row.dragging {
    opacity: 0.3;
  }
  .pm-row.over {
    border-color: var(--amber);
    background: var(--amber-g);
  }
  .pm-info {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .pm-idx {
    font-size: 10px;
    color: var(--txt3);
    min-width: 18px;
  }
  .pm-name {
    font-size: 12px;
    font-weight: 400;
    color: var(--txt);
  }
  .pm-ctrl {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .pm-ctrl button {
    width: 24px;
    height: 24px;
    background: var(--surf2);
    border: 1px solid var(--brd2);
    border-radius: 6px;
    color: var(--txt2);
    font-size: 14px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
    font-family: inherit;
  }
  .pm-ctrl button:hover {
    background: var(--surf3);
    color: var(--txt);
    border-color: rgba(255, 255, 255, 0.12);
  }
  .pm-ctrl button:active {
    transform: scale(0.92);
  }
  .pm-w {
    font-size: 13px;
    font-weight: 500;
    color: var(--amber);
    min-width: 28px;
    text-align: center;
  }

  /* ─── Toggle System ─── */
  .toggles {
    display: flex;
    flex-direction: column;
    flex: 1;
  }
  .tg {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 11px 0;
    background: none;
    border: none;
    border-bottom: 1px solid var(--brd);
    color: var(--txt2);
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: color 0.15s;
    width: 100%;
    text-align: left;
  }
  .tg:last-child {
    border-bottom: none;
  }
  .tg:hover {
    color: var(--txt);
  }

  .sw {
    width: 32px;
    height: 18px;
    background: var(--surf3);
    border-radius: 9px;
    position: relative;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    flex-shrink: 0;
  }
  .sw::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 12px;
    height: 12px;
    background: var(--txt3);
    border-radius: 50%;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }
  .sw.on {
    background: var(--amber);
    box-shadow: 0 0 12px rgba(212, 160, 87, 0.25);
  }
  .sw.on::after {
    left: 17px;
    background: var(--bg);
  }

  /* ─── Slider ─── */
  .slider-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 0;
    color: var(--txt2);
    font-size: 12px;
  }
  .slider-row input[type="range"] {
    flex: 1;
    height: 3px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--surf3);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .slider-row input::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    background: var(--amber);
    border-radius: 50%;
    cursor: pointer;
    box-shadow: 0 0 8px rgba(212, 160, 87, 0.3);
    transition: transform 0.15s;
  }
  .slider-row input::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }
  .sv {
    font-size: 11px;
    color: var(--amber);
    min-width: 36px;
    text-align: right;
    font-weight: 500;
  }

  /* ─── Cache ─── */
  .cache {
    grid-column: 1 / -1;
    padding: 14px 20px;
    animation-delay: 0.4s;
  }
  .cache-in {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
  }
  .cache-in > div:first-child {
    display: flex;
    align-items: baseline;
    gap: 14px;
  }
  .cache-in .label {
    margin-bottom: 0;
  }
  .cache-sz {
    font-size: 14px;
    font-weight: 500;
    color: var(--txt);
  }
  .cache-btns {
    display: flex;
    gap: 8px;
  }
  .cache-btns button {
    padding: 7px 16px;
    background: var(--surf2);
    border: 1px solid var(--brd2);
    border-radius: 8px;
    color: var(--txt2);
    font-family: inherit;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .cache-btns button:hover {
    background: var(--surf3);
    border-color: rgba(255, 255, 255, 0.1);
    color: var(--txt);
  }
  .cache-btns .warn:hover {
    background: rgba(192, 96, 80, 0.12);
    border-color: rgba(192, 96, 80, 0.3);
    color: var(--red);
  }

  /* ─── Animation Delays ─── */
  .sys {
    animation-delay: 0.15s;
  }
  .vis {
    animation-delay: 0.2s;
  }
  .flt {
    animation-delay: 0.25s;
  }
  .isl {
    animation-delay: 0.3s;
  }
</style>
