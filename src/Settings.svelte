<script lang="ts">
  import { onMount } from "svelte";
  import { settingsApi } from "$lib/api/settings";
  import { cacheApi } from "$lib/api/cache";
  import { windowApi } from "$lib/api/window";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { X, Minus } from "lucide-svelte";
  import type { AppSettings } from "$lib/api/types";
  import { DEFAULT_SETTINGS } from "$lib/api/types";
  import Toggle from "./components/common/Toggle.svelte";
  import Button from "./components/common/Button.svelte";

  let settings = $state<AppSettings>({ ...DEFAULT_SETTINGS });
  const appWindow = getCurrentWindow();

  const playerNames: Record<string, string> = {
    netease: "网易云音乐",
    spotify: "Spotify",
    bilibili: "Bilibili",
    qqmusic: "QQ 音乐",
    apple: "Apple Music",
    generic: "其他播放器",
  };

  let playerOrder = $state<string[]>([]);
  let cacheSize = $state("0 MB");
  let monitors = $state<string[]>([]);
  let currentMonitorIndex = $state(0);

  let dragIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  onMount(async () => {
    try {
      const saved = await settingsApi.getSettings();
      settings = { ...settings, ...saved };
      if (saved.playerWeights) settings.playerWeights = saved.playerWeights;

      try {
        settings.autoStart = await settingsApi.getAutoStart();
      } catch { /* ignore */ }

      playerOrder = Object.keys(settings.playerWeights);
      loadCacheStats();
      await loadMonitors();
    } catch { /* ignore */ }
  });

  async function saveSettings(partial: Partial<AppSettings>) {
    settings = { ...settings, ...partial };
    await settingsApi.saveSettings($state.snapshot(settings));
  }

  async function toggleAutoStart() {
    const next = !settings.autoStart;
    await settingsApi.setAutoStart(next);
    settings.autoStart = next;
  }

  async function loadCacheStats() {
    try {
      const stats = await cacheApi.getCacheStats();
      if (stats && typeof stats.totalSizeMb === "number" && !isNaN(stats.totalSizeMb)) {
        cacheSize = `${stats.totalSizeMb.toFixed(2)} MB`;
      } else {
        cacheSize = "0.00 MB";
      }
    } catch {
      cacheSize = "0.00 MB";
    }
  }

  async function loadMonitors() {
    try {
      monitors = await windowApi.getAvailableMonitors();
      currentMonitorIndex = await windowApi.getCurrentMonitorIndex();
    } catch { /* ignore */ }
  }

  async function switchMonitor(index: number) {
    await windowApi.setCurrentMonitorIndex(index);
    currentMonitorIndex = index;
  }

  async function clearCache() {
    try {
      await cacheApi.clearCache();
      await loadCacheStats();
    } catch { /* ignore */ }
  }

  function onDragStart(index: number, e: DragEvent) {
    dragIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", String(index));
    }
  }

  function onDragOver(index: number, e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOverIndex = index;
  }

  function onDragLeave() {
    dragOverIndex = null;
  }

  function onDrop(targetIndex: number, e: DragEvent) {
    e.preventDefault();
    const fromIndex = dragIndex;
    if (fromIndex === null || fromIndex === targetIndex) {
      dragIndex = null;
      dragOverIndex = null;
      return;
    }
    const next = [...playerOrder];
    const [moved] = next.splice(fromIndex, 1);
    next.splice(targetIndex, 0, moved);
    playerOrder = next;
    const w: Record<string, number> = {};
    next.forEach((p) => (w[p] = settings.playerWeights[p] ?? 50));
    saveSettings({ playerWeights: w });
    dragIndex = null;
    dragOverIndex = null;
  }

  function onDragEnd() {
    dragIndex = null;
    dragOverIndex = null;
  }
</script>

<div class="settings">
  <header class="settings-header" data-tauri-drag-region>
    <span class="settings-title">设置</span>
    <div class="settings-header-actions">
      <button class="settings-win-btn" onclick={() => appWindow.minimize()}>
        <Minus size={12} />
      </button>
      <button class="settings-win-btn settings-win-btn--close" onclick={() => appWindow.close()}>
        <X size={12} />
      </button>
    </div>
  </header>

  <div class="settings-body">
    <!-- 常规 -->
    <section class="settings-section">
      <h3 class="settings-section-title">常规</h3>
      <div class="settings-row">
        <span class="settings-label">自动隐藏</span>
        <Toggle
          checked={settings.autoHide}
          size="sm"
          onChange={() => saveSettings({ autoHide: !settings.autoHide })}
        />
      </div>
      <div class="settings-row">
        <span class="settings-label">显示频谱</span>
        <Toggle
          checked={settings.showSpectrum}
          size="sm"
          onChange={() => saveSettings({ showSpectrum: !settings.showSpectrum })}
        />
      </div>
      <div class="settings-row">
        <span class="settings-label">始终置顶</span>
        <Toggle
          checked={settings.alwaysOnTop}
          size="sm"
          onChange={() => saveSettings({ alwaysOnTop: !settings.alwaysOnTop })}
        />
      </div>
      <div class="settings-row">
        <span class="settings-label">开机自启</span>
        <Toggle
          checked={settings.autoStart}
          size="sm"
          onChange={toggleAutoStart}
        />
      </div>
    </section>

    <!-- 播放器优先级 -->
    <section class="settings-section">
      <h3 class="settings-section-title">播放器优先级</h3>
      <p class="settings-section-desc">拖拽排序，编号越小优先级越高</p>
      <div class="player-list">
        {#each playerOrder as player, index (player)}
          <div
            class="player-item"
            class:player-item--dragging={dragIndex === index}
            class:player-item--over={dragOverIndex === index}
            draggable="true"
            ondragstart={(e) => onDragStart(index, e)}
            ondragover={(e) => onDragOver(index, e)}
            ondragleave={onDragLeave}
            ondrop={(e) => onDrop(index, e)}
            ondragend={onDragEnd}
          >
            <span class="player-grip">⠿</span>
            <span class="player-rank">{index + 1}</span>
            <span class="player-name">{playerNames[player] || player}</span>
          </div>
        {/each}
      </div>
    </section>

    <!-- 显示器 -->
    <section class="settings-section">
      <h3 class="settings-section-title">显示器</h3>
      <div class="settings-row">
        <span class="settings-label">当前显示器</span>
        <select
          class="settings-select"
          value={currentMonitorIndex}
          onchange={(e) => switchMonitor(Number((e.target as HTMLSelectElement).value))}
        >
          {#each monitors as monitor, idx (idx)}
            <option value={idx}>
              {monitor || `显示器 ${idx + 1}`}
            </option>
          {/each}
        </select>
      </div>
    </section>

    <!-- 存储 -->
    <section class="settings-section">
      <h3 class="settings-section-title">存储</h3>
      <div class="settings-row">
        <span class="settings-label">缓存大小</span>
        <span class="settings-value">{cacheSize}</span>
      </div>
      <div class="settings-row">
        <Button variant="secondary" size="sm" onclick={clearCache}>
          清除缓存
        </Button>
      </div>
    </section>
  </div>
</div>

<style>
  @import "./styles/variables.css";

  .settings {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--base-dark);
    color: var(--text-base);
    font-family: var(--font-family-ui);
    overflow: hidden;
  }

  /* ─── Header ─── */
  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--spacing-lg);
    height: 48px;
    background: var(--base-dark-gray);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
    -webkit-user-select: none;
  }

  .settings-title {
    font-size: var(--text-lg);
    font-weight: var(--font-bold);
    color: var(--text-base);
  }

  .settings-header-actions {
    display: flex;
    gap: var(--spacing-xs);
    -webkit-app-region: no-drag;
  }

  .settings-win-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background var(--transition-base), color var(--transition-base);
  }

  .settings-win-btn:hover {
    background: var(--base-mid-gray);
    color: var(--text-base);
  }

  .settings-win-btn--close:hover {
    background: var(--text-negative);
    color: white;
  }

  /* ─── Body ─── */
  .settings-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xl);
    scrollbar-width: thin;
    scrollbar-color: var(--base-mid-gray) transparent;
  }

  .settings-body::-webkit-scrollbar {
    width: 6px;
  }

  .settings-body::-webkit-scrollbar-track {
    background: transparent;
  }

  .settings-body::-webkit-scrollbar-thumb {
    background: var(--base-mid-gray);
    border-radius: 3px;
  }

  /* ─── Section ─── */
  .settings-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .settings-section-title {
    font-size: var(--text-sm);
    font-weight: var(--font-bold);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    margin: 0 0 var(--spacing-xs);
    padding-bottom: var(--spacing-xs);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .settings-section-desc {
    font-size: var(--text-xs);
    color: var(--text-hint);
    margin: 0 0 var(--spacing-xs);
  }

  /* ─── Row ─── */
  .settings-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--base-dark-gray);
    border-radius: var(--radius-card);
    min-height: 44px;
  }

  .settings-label {
    font-size: var(--text-md);
    font-weight: var(--font-normal);
    color: var(--text-base);
  }

  .settings-value {
    font-size: var(--text-md);
    font-weight: var(--font-medium);
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  /* ─── Select ─── */
  .settings-select {
    appearance: none;
    -webkit-appearance: none;
    background: var(--base-mid-gray);
    color: var(--text-base);
    border: none;
    border-radius: var(--radius-input);
    padding: var(--spacing-xs) var(--spacing-lg) var(--spacing-xs) var(--spacing-md);
    font-size: var(--text-sm);
    font-family: var(--font-family-ui);
    cursor: pointer;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23b3b3b3' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    transition: background var(--transition-base);
  }

  .settings-select:hover {
    background-color: var(--base-card);
  }

  .settings-select:focus {
    outline: none;
    box-shadow: 0 0 0 2px var(--accent-green);
  }

  /* ─── Player List ─── */
  .player-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    background: var(--base-dark-gray);
    border-radius: var(--radius-card);
    overflow: hidden;
  }

  .player-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--base-dark-gray);
    cursor: grab;
    user-select: none;
    -webkit-user-select: none;
    transition: background var(--transition-base), transform var(--transition-fast);
  }

  .player-item:active {
    cursor: grabbing;
  }

  .player-item:hover {
    background: var(--base-mid-gray);
  }

  .player-item--dragging {
    opacity: 0.4;
    background: var(--base-mid-gray);
  }

  .player-item--over {
    box-shadow: inset 0 2px 0 0 var(--accent-green);
  }

  .player-grip {
    font-size: var(--text-lg);
    color: var(--text-hint);
    line-height: 1;
    flex-shrink: 0;
    letter-spacing: -2px;
  }

  .player-rank {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-circle);
    background: var(--base-card);
    font-size: var(--text-xs);
    font-weight: var(--font-bold);
    color: var(--text-secondary);
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  .player-name {
    font-size: var(--text-md);
    color: var(--text-base);
    flex: 1;
  }
</style>
