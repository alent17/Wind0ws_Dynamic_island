<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { X, Minus, Monitor, ChevronUp, ChevronDown } from "lucide-svelte";

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
    { id: "original", name: "经典黑", color: "#333" },
    { id: "liquid_glass", name: "液体玻璃", color: "#1a1a2e" },
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
  let cacheSize = $state("0 MB");

  let monitors = $state<Array<{ name: string; id: number }>>([]);
  let currentMonitorIndex = $state(0);

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
      await loadMonitors();
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
      cacheSize = `${stats.total_size_mb.toFixed(2)} MB`;
    } catch (e) {
      console.error("加载缓存统计失败", e);
    }
  }

  async function loadMonitors() {
    try {
      const allMonitors = await invoke<any[]>("get_available_monitors");
      monitors = allMonitors.map((m, idx) => ({
        name: m.name || `显示器 ${idx + 1}`,
        id: idx,
      }));
      currentMonitorIndex = await invoke<number>("get_current_monitor_index");
    } catch (e) {
      console.error("加载显示器列表失败", e);
    }
  }

  async function switchMonitor(index: number) {
    try {
      await invoke("set_current_monitor_index", { index });
      currentMonitorIndex = index;
    } catch (e) {
      console.error("切换显示器失败", e);
    }
  }

  async function pickCacheDir() {
    try {
      await invoke("pick_cache_directory");
    } catch (e) {
      console.error("选择缓存目录失败", e);
    }
  }

  async function clearCache() {
    if (!confirm("确定要清除所有缓存吗？")) return;
    try {
      await invoke("clear_cache");
      await loadCacheStats();
    } catch (e) {
      console.error("清除缓存失败", e);
    }
  }

  function movePlayer(index: number, direction: -1 | 1) {
    const target = index + direction;
    if (target < 0 || target >= playerOrder.length) return;
    const o = [...playerOrder];
    [o[index], o[target]] = [o[target], o[index]];
    playerOrder = o;
    const w: Record<string, number> = {};
    o.forEach((p) => (w[p] = settings.player_weights[p] ?? 50));
    saveSettings({ player_weights: w });
  }

  function togglePlayer(player: string) {
    const w = settings.player_weights[player] ?? 50;
    const nw = w === 0 ? 50 : 0;
    saveSettings({
      player_weights: { ...settings.player_weights, [player]: nw },
    });
  }
</script>

<div class="app">
  <div class="wrap">
    <!-- Header -->
    <header class="hdr" data-tauri-drag-region>
      <div class="hdr-left">
        <span class="hdr-tag">◆</span>
        <span class="hdr-title">ISLAND CONTROL</span>
      </div>
      <div class="hdr-right">
        <button class="win-btn" onclick={() => appWindow.minimize()}
          ><Minus size={12} /></button
        >
        <button class="win-btn close" onclick={() => appWindow.close()}
          ><X size={12} /></button
        >
      </div>
    </header>

    <div class="grid">
      <!-- ══════ 外观 ══════ -->
      <div class="card card-rose">
        <div class="card-head"><span>01</span> 外观</div>
        <div class="card-body">
          <div class="section">
            <span class="section-label">主题</span>
            <div class="theme-row">
              {#each themes as t}
                <label class="theme-chip">
                  <input
                    type="radio"
                    name="theme"
                    checked={settings.island_theme === t.id}
                    onchange={() => saveSettings({ island_theme: t.id })}
                  />
                  <span class="theme-chip-inner">
                    <span class="theme-dot" style="background:{t.color}"></span>
                    {t.name}
                  </span>
                </label>
              {/each}
            </div>
          </div>

          <div class="section">
            <span class="section-label">视觉效果</span>
            <div class="toggle-grid">
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.enable_hd_cover}
                  onchange={() =>
                    saveSettings({
                      enable_hd_cover: !settings.enable_hd_cover,
                    })}
                />
                <span class="tg-label">高清封面</span>
              </label>
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.enable_pixel_art}
                  onchange={() =>
                    saveSettings({
                      enable_pixel_art: !settings.enable_pixel_art,
                    })}
                />
                <span class="tg-label">像素风格</span>
              </label>
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.enable_halftone}
                  onchange={() =>
                    saveSettings({
                      enable_halftone: !settings.enable_halftone,
                    })}
                />
                <span class="tg-label">网点效果</span>
              </label>
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.enable_mv_playback}
                  onchange={() =>
                    saveSettings({
                      enable_mv_playback: !settings.enable_mv_playback,
                    })}
                />
                <span class="tg-label">MV 引擎</span>
              </label>
            </div>
          </div>

          <div class="section">
            <span class="section-label">灵动岛</span>
            <div class="slider-row">
              <span class="slider-text">展开圆角</span>
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
              <span class="slider-num">{settings.expanded_corner_radius}px</span
              >
            </div>
          </div>
        </div>
      </div>

      <!-- ══════ 窗口 ══════ -->
      <div class="card card-sage">
        <div class="card-head"><span>02</span> 窗口</div>
        <div class="card-body">
          <div class="section">
            <span class="section-label">显示器</span>
            <div class="monitor-grid">
              {#each monitors as monitor, idx}
                <button
                  class="monitor-btn"
                  class:active={currentMonitorIndex === idx}
                  onclick={() => switchMonitor(idx)}
                >
                  <Monitor size={14} />
                  <span class="monitor-name">{monitor.name}</span>
                  {#if currentMonitorIndex === idx}
                    <span class="monitor-dot"></span>
                  {/if}
                </button>
              {/each}
            </div>
          </div>

          <div class="section">
            <span class="section-label">悬浮窗</span>
            <div class="toggle-grid">
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.always_on_top}
                  onchange={() =>
                    saveSettings({ always_on_top: !settings.always_on_top })}
                />
                <span class="tg-label">窗口置顶</span>
              </label>
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.lock_floating_window}
                  onchange={() =>
                    saveSettings({
                      lock_floating_window: !settings.lock_floating_window,
                    })}
                />
                <span class="tg-label">锁定位置</span>
              </label>
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.always_show_top_bar}
                  onchange={() =>
                    saveSettings({
                      always_show_top_bar: !settings.always_show_top_bar,
                    })}
                />
                <span class="tg-label">固定顶栏</span>
              </label>
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.hide_floating_window}
                  onchange={() =>
                    saveSettings({
                      hide_floating_window: !settings.hide_floating_window,
                    })}
                />
                <span class="tg-label">隐藏窗口</span>
              </label>
            </div>
          </div>

          <div class="section">
            <span class="section-label">系统</span>
            <div class="toggle-grid">
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.auto_start}
                  onchange={toggleAutoStart}
                />
                <span class="tg-label">开机自启</span>
              </label>
              <label class="tg">
                <input
                  type="checkbox"
                  checked={settings.auto_hide}
                  onchange={() =>
                    saveSettings({ auto_hide: !settings.auto_hide })}
                />
                <span class="tg-label">自动隐藏</span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <!-- ══════ 播放器 ══════ -->
      <div class="card card-sky">
        <div class="card-head">
          <span>03</span> 播放器
          <span class="card-badge"
            >{playerOrder.filter((p) => (settings.player_weights[p] ?? 50) > 0)
              .length}/{playerOrder.length}</span
          >
        </div>
        <div class="card-body">
          {#each playerOrder as player, index}
            <div
              class="player-row"
              class:disabled={(settings.player_weights[player] ?? 50) === 0}
            >
              <span class="player-idx"
                >{String(index + 1).padStart(2, "0")}</span
              >
              <span class="player-name">{playerNames[player] || player}</span>
              <div class="player-ctrl">
                <button
                  class="pm-btn"
                  class:pm-off={(settings.player_weights[player] ?? 50) === 0}
                  onclick={() => togglePlayer(player)}
                >
                  {(settings.player_weights[player] ?? 50) === 0 ? "OFF" : "ON"}
                </button>
                <div class="arrow-group">
                  <button
                    class="arrow-btn"
                    disabled={index === 0}
                    onclick={() => movePlayer(index, -1)}
                  >
                    <ChevronUp size={12} />
                  </button>
                  <button
                    class="arrow-btn"
                    disabled={index === playerOrder.length - 1}
                    onclick={() => movePlayer(index, 1)}
                  >
                    <ChevronDown size={12} />
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- ══════ 存储 ══════ -->
      <div class="card card-amber card-compact">
        <div class="card-head"><span>04</span> 存储</div>
        <div class="card-body compact-body">
          <span class="cache-size">{cacheSize}</span>
          <div class="cache-btns">
            <button class="act-btn" onclick={pickCacheDir}>映射路径</button>
            <button class="act-btn danger" onclick={clearCache}>清空缓存</button
            >
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  @import url("https://fonts.googleapis.com/css2?family=Montserrat:wght@400;600;700;800;900&display=swap");

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
    font-family: "Montserrat", sans-serif;
    background: #f0e6d6;
  }

  :root {
    --warm-bg: #f0e6d6;
    --warm-white: #faf3e8;
    --warm-surface: #f5edd8;
    --ink: #2c2418;
  }

  .app {
    width: 100vw;
    height: 100vh;
    padding: 12px;
    background: var(--warm-bg);
  }

  .wrap {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  /* ─── Header ─── */
  .hdr {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: var(--warm-white);
    border: 3px solid #000;
    box-shadow: 6px 6px 0 #000;
    flex-shrink: 0;
    transition: all 0.3s ease;
  }
  .hdr:hover {
    translate: -2px -2px;
    box-shadow: 8px 8px 0 #000;
  }

  .hdr-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .hdr-tag {
    font-size: 18px;
    color: #d4785c;
  }
  .hdr-title {
    font-size: 15px;
    font-weight: 900;
    letter-spacing: 3px;
    color: var(--ink);
  }
  .hdr-right {
    display: flex;
    gap: 4px;
    -webkit-app-region: no-drag;
  }
  .win-btn {
    width: 28px;
    height: 28px;
    background: var(--warm-surface);
    border: 2px solid #000;
    color: var(--ink);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }
  .win-btn:hover {
    background: var(--ink);
    color: var(--warm-white);
    translate: 1px 1px;
  }
  .win-btn.close:hover {
    background: #d4785c;
    color: #000;
  }

  /* ─── Grid ─── */
  .grid {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr auto;
    gap: 10px;
    overflow-y: auto;
    scrollbar-width: none;
  }
  .grid::-webkit-scrollbar {
    display: none;
  }

  /* ─── Card ─── */
  .card {
    border: 3px solid #000;
    box-shadow: 8px 8px 0 #000;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: all 0.3s ease;
    animation: slideIn 0.4s ease both;
  }
  .card:hover {
    translate: -3px -3px;
    box-shadow: 11px 11px 0 #000;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      translate: 0 12px;
    }
    to {
      opacity: 1;
      translate: 0 0;
    }
  }

  .card-rose {
    background: #d4785c;
  }
  .card-sage {
    background: #7bab8a;
  }
  .card-sky {
    background: #7ca3c4;
  }
  .card-amber {
    background: #c9a84c;
  }

  .card-head {
    font-size: 13px;
    font-weight: 900;
    width: 100%;
    background: var(--warm-white);
    padding: 7px 14px;
    color: var(--ink);
    border-bottom: 3px solid #000;
    display: flex;
    align-items: center;
    gap: 8px;
    text-transform: uppercase;
    letter-spacing: 1px;
    flex-shrink: 0;
  }
  .card-head span:first-child {
    font-size: 18px;
    font-weight: 900;
    min-width: 24px;
  }
  .card-badge {
    margin-left: auto;
    background: #000;
    color: var(--warm-white);
    padding: 0 8px;
    font-size: 11px;
    font-weight: 800;
  }

  .card-body {
    padding: 10px 14px;
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    scrollbar-width: none;
  }
  .card-body::-webkit-scrollbar {
    display: none;
  }

  .card-compact .card-body {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }

  /* ─── Section ─── */
  .section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-top: 8px;
    border-top: 2px solid rgba(0, 0, 0, 0.12);
  }
  .section:first-child {
    padding-top: 0;
    border-top: none;
  }

  .section-label {
    font-size: 9px;
    font-weight: 900;
    text-transform: uppercase;
    letter-spacing: 2px;
    color: rgba(0, 0, 0, 0.4);
  }

  /* ─── Theme Chips ─── */
  .theme-row {
    display: flex;
    gap: 6px;
  }
  .theme-chip {
    cursor: pointer;
  }
  .theme-chip input {
    display: none;
  }
  .theme-chip-inner {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    background: rgba(250, 243, 232, 0.55);
    border: 2px solid #000;
    font-size: 11px;
    font-weight: 800;
    color: var(--ink);
    box-shadow: 3px 3px 0 #000;
    transition: all 0.2s;
  }
  .theme-chip-inner:hover {
    translate: -1px -1px;
    box-shadow: 4px 4px 0 #000;
  }
  .theme-chip input:checked + .theme-chip-inner {
    background: var(--ink);
    color: var(--warm-white);
    box-shadow: 3px 3px 0 rgba(0, 0, 0, 0.3);
  }
  .theme-dot {
    width: 10px;
    height: 10px;
    border-radius: 2px;
    border: 1px solid rgba(0, 0, 0, 0.3);
    flex-shrink: 0;
  }
  .theme-chip input:checked + .theme-chip-inner .theme-dot {
    border-color: rgba(255, 255, 255, 0.3);
  }

  /* ─── Toggles ─── */
  .toggle-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }
  .tg {
    cursor: pointer;
  }
  .tg input {
    display: none;
  }
  .tg-label {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px 12px;
    min-width: 72px;
    height: 30px;
    background: rgba(250, 243, 232, 0.5);
    border: 2px solid #000;
    font-size: 11px;
    font-weight: 800;
    color: var(--ink);
    box-shadow: 3px 3px 0 #000;
    transition: all 0.2s;
    user-select: none;
  }
  .tg-label:hover {
    translate: -1px -1px;
    box-shadow: 4px 4px 0 #000;
    background: rgba(250, 243, 232, 0.75);
  }
  .tg input:checked + .tg-label {
    background: var(--ink);
    color: var(--warm-white);
    box-shadow: 3px 3px 0 rgba(0, 0, 0, 0.3);
    translate: 2px 2px;
  }
  .tg input:checked + .tg-label:hover {
    translate: 1px 1px;
    box-shadow: 4px 4px 0 rgba(0, 0, 0, 0.3);
  }

  /* ─── Slider ─── */
  .slider-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 0;
  }
  .slider-text {
    font-size: 10px;
    font-weight: 800;
    color: rgba(0, 0, 0, 0.5);
    min-width: 52px;
  }
  .slider-row input[type="range"] {
    flex: 1;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: rgba(0, 0, 0, 0.15);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .slider-row input::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    background: var(--ink);
    border: 2px solid var(--warm-white);
    border-radius: 0;
    cursor: pointer;
    box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.2);
    transition: all 0.15s;
  }
  .slider-row input::-webkit-slider-thumb:hover {
    translate: -1px -1px;
    box-shadow: 3px 3px 0 rgba(0, 0, 0, 0.2);
  }
  .slider-num {
    font-size: 12px;
    font-weight: 900;
    color: var(--ink);
    min-width: 36px;
    text-align: right;
  }

  /* ─── Monitor ─── */
  .monitor-grid {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .monitor-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 12px;
    background: transparent;
    border: 2px solid rgba(0, 0, 0, 0.1);
    cursor: pointer;
    transition: all 0.2s;
    font-family: inherit;
    font-size: 11px;
    font-weight: 700;
    color: var(--ink);
  }
  .monitor-btn:hover {
    background: rgba(250, 243, 232, 0.75);
    border-color: rgba(0, 0, 0, 0.2);
    translate: -1px -1px;
    box-shadow: 3px 3px 0 rgba(0, 0, 0, 0.1);
  }
  .monitor-btn.active {
    background: var(--ink);
    color: var(--warm-white);
    border-color: var(--ink);
    box-shadow: 3px 3px 0 rgba(0, 0, 0, 0.2);
    translate: 2px 2px;
  }
  .monitor-btn.active:hover {
    translate: 1px 1px;
    box-shadow: 4px 4px 0 rgba(0, 0, 0, 0.2);
  }
  .monitor-name {
    flex: 1;
    text-align: left;
  }
  .monitor-dot {
    width: 6px;
    height: 6px;
    background: #7bab8a;
    border-radius: 50%;
  }

  /* ─── Player List ─── */
  .player-row {
    display: flex;
    align-items: center;
    padding: 5px 8px;
    background: rgba(250, 243, 232, 0.35);
    border: 2px solid rgba(0, 0, 0, 0.1);
    transition: all 0.15s;
    gap: 8px;
  }
  .player-row:hover {
    background: rgba(250, 243, 232, 0.6);
    border-color: rgba(0, 0, 0, 0.25);
  }
  .player-row.disabled {
    opacity: 0.45;
  }
  .player-row.disabled .player-name {
    text-decoration: line-through;
  }
  .player-idx {
    font-size: 9px;
    font-weight: 900;
    color: rgba(0, 0, 0, 0.35);
    min-width: 18px;
  }
  .player-name {
    font-size: 11px;
    font-weight: 700;
    color: var(--ink);
    flex: 1;
  }
  .player-ctrl {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .pm-btn {
    padding: 2px 10px;
    height: 24px;
    background: rgba(250, 243, 232, 0.5);
    border: 2px solid #000;
    color: var(--ink);
    font-size: 10px;
    font-weight: 900;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: inherit;
    box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.2);
    transition: all 0.15s;
    min-width: 38px;
  }
  .pm-btn:hover {
    background: var(--ink);
    color: var(--warm-white);
    translate: -1px -1px;
    box-shadow: 3px 3px 0 rgba(0, 0, 0, 0.2);
  }
  .pm-btn:active {
    translate: 1px 1px;
    box-shadow: 0 0 0;
  }
  .pm-btn.pm-off {
    background: rgba(0, 0, 0, 0.12);
    color: rgba(0, 0, 0, 0.35);
  }
  .pm-btn.pm-off:hover {
    background: #d4785c;
    color: #000;
  }

  .arrow-group {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .arrow-btn {
    width: 22px;
    height: 13px;
    background: rgba(250, 243, 232, 0.5);
    border: 1.5px solid rgba(0, 0, 0, 0.25);
    color: var(--ink);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: inherit;
    transition: all 0.12s;
    padding: 0;
  }
  .arrow-btn:first-child {
    border-radius: 3px 3px 0 0;
  }
  .arrow-btn:last-child {
    border-radius: 0 0 3px 3px;
  }
  .arrow-btn:hover:not(:disabled) {
    background: var(--ink);
    color: var(--warm-white);
    border-color: var(--ink);
  }
  .arrow-btn:active:not(:disabled) {
    translate: 0 1px;
  }
  .arrow-btn:disabled {
    opacity: 0.25;
    cursor: not-allowed;
  }

  /* ─── Storage ─── */
  .cache-size {
    font-size: 16px;
    font-weight: 900;
    color: var(--ink);
  }
  .cache-btns {
    display: flex;
    gap: 6px;
  }
  .act-btn {
    background: var(--warm-white);
    font-family: inherit;
    padding: 5px 14px;
    font-weight: 800;
    font-size: 12px;
    border: 3px solid #000;
    box-shadow: 4px 4px 0 #000;
    cursor: pointer;
    color: var(--ink);
    transition: all 0.2s;
  }
  .act-btn:hover {
    translate: -1.5px -1.5px;
    box-shadow: 5.5px 5.5px 0 #000;
    background: var(--ink);
    color: var(--warm-white);
  }
  .act-btn:active {
    translate: 3px 3px;
    box-shadow: 0 0 0 #000;
  }
  .act-btn.danger {
    background: #d4785c;
    color: #000;
  }
  .act-btn.danger:hover {
    background: #c06040;
    color: #000;
  }

  /* ─── Stagger ─── */
  .card:nth-child(1) {
    animation-delay: 0.05s;
  }
  .card:nth-child(2) {
    animation-delay: 0.1s;
  }
  .card:nth-child(3) {
    animation-delay: 0.15s;
  }
  .card:nth-child(4) {
    animation-delay: 0.2s;
  }
</style>
