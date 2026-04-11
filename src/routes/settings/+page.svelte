<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    X,
    Minus,
    Check,
    Palette,
    Music,
    Database,
    GripVertical,
    ArrowUp,
    ArrowDown,
    Moon,
    Sun,
    Monitor,
    ChevronRight,
  } from "lucide-svelte";
  
  import Button from '../../components/common/Button.svelte';
  import Card from '../../components/common/Card.svelte';
  import Toggle from '../../components/common/Toggle.svelte';
  import { settingsStore, settingsActions } from '../../stores/settings';
  import { windowCommands, settingsCommands } from '../../utils/tauri';

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
      name: "极简经典",
      icon: Sun,
      color: "#1a1a1a",
    },
    {
      id: "glassmorphism",
      name: "毛玻璃",
      icon: Moon,
      color: "#1a1a2e",
    },
  ];

  const playerNames: Record<string, string> = {
    netease: "网易云音乐",
    spotify: "Spotify",
    bilibili: "Bilibili",
    qqmusic: "QQ 音乐",
    apple: "Apple Music",
    generic: "其他播放器",
  };

  let playerOrder = $state<string[]>([]);
  let dragIndex = $state<number | null>(null);
  let overIndex = $state<number | null>(null);
  let isDragging = $state(false);

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
    } catch (e) {
      console.error("无法读取设置", e);
    }

    playerOrder = Object.keys(settings.player_weights);
  });

  async function updateSetting(key: keyof AppSettings, value: any) {
    settings[key as keyof AppSettings] = value;
    try {
      if (key === "auto_start") {
        await settingsCommands.setAutoStart(value);
      } else {
        await invoke("update_settings", {
          settings: { [key]: value },
        });
      }
    } catch (e) {
      console.error("保存设置失败", e);
    }
  }

  async function handlePlayerDragStart(index: number) {
    dragIndex = index;
    overIndex = index;
    isDragging = true;
  }

  async function handlePlayerDrop() {
    if (dragIndex === null || overIndex === null) return;

    const newOrder = [...playerOrder];
    const draggedItem = newOrder[dragIndex];
    newOrder.splice(dragIndex, 1);
    newOrder.splice(overIndex, 0, draggedItem);

    playerOrder = newOrder;
    dragIndex = null;
    overIndex = null;
    isDragging = false;

    const newWeights: Record<string, number> = {};
    playerOrder.forEach((player, index) => {
      newWeights[player] = playerOrder.length - index;
    });

    settings.player_weights = newWeights;
    await updateSetting("player_weights", newWeights);
  }

  async function clearCache() {
    try {
      await settingsCommands.clearCache();
      settingsActions.clearCache();
      alert("缓存已清除");
    } catch (e) {
      console.error("清除缓存失败", e);
      alert("清除缓存失败：" + e);
    }
  }
</script>

<div class="settings-container">
  <header class="header">
    <div class="header-content">
      <h1 class="title">设置</h1>
      <div class="window-controls">
        <button class="icon-btn" aria-label="最小化">
          <Minus size={20} />
        </button>
        <button class="icon-btn close" aria-label="关闭" on:click={() => appWindow.close()}>
          <X size={20} />
        </button>
      </div>
    </div>
  </header>

  <div class="content">
    <section class="preview-section">
      <h2 class="preview-title">预览</h2>
      <div class="preview-container">
        <div class="island-preview collapsed">
          <div class="island-content">
            <div class="top-section">
              <div class="album-art-preview">
                <div class="album-art-inner"></div>
              </div>
              <div class="track-info-preview">
                <div class="track-name">歌曲名称</div>
                <div class="artist-name">艺术家</div>
              </div>
            </div>
            <div class="playback-controls">
              <button class="control-btn-preview" aria-label="播放">
                <div class="play-icon"></div>
              </button>
            </div>
          </div>
        </div>

        <div class="island-preview expanded">
          <div class="island-content expanded">
            <div class="top-section">
              <div class="album-art-preview expanded">
                <div class="album-art-inner"></div>
              </div>
              <div class="track-info-preview">
                <div class="track-name">歌曲名称</div>
                <div class="artist-name">艺术家</div>
              </div>
            </div>
            <div class="progress-bar-preview">
              <div class="progress-fill-preview"></div>
            </div>
            <div class="playback-controls-expanded">
              <button class="control-btn-preview" aria-label="上一首">
                <div class="skip-back-icon"></div>
              </button>
              <button class="control-btn-preview" aria-label="播放">
                <div class="play-icon"></div>
              </button>
              <button class="control-btn-preview" aria-label="下一首">
                <div class="skip-forward-icon"></div>
              </button>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h2 class="section-title">常规设置</h2>
      
      <Card variant="default" padding="none" class="setting-card">
        <div class="setting-row">
          <div class="setting-icon">
            <Palette size={20} />
          </div>
          <div class="setting-info">
            <div class="setting-label">主题</div>
            <div class="setting-hint">选择灵动岛的主题风格</div>
          </div>
          <div class="theme-selector">
            {#each themes as theme}
              <button
                class="theme-option {settings.island_theme === theme.id ? 'active' : ''}"
                on:click={() => updateSetting("island_theme", theme.id)}
              >
                <div class="theme-icon-wrapper">
                  <svelte:component this={theme.icon} size={20} />
                </div>
                <span class="theme-label">{theme.name}</span>
              </button>
            {/each}
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-icon">
            <Monitor size={20} />
          </div>
          <div class="setting-info">
            <div class="setting-label">自动隐藏</div>
            <div class="setting-hint">鼠标移开时自动隐藏灵动岛</div>
          </div>
          <Toggle 
            checked={settings.auto_hide}
            on:change={(e) => updateSetting("auto_hide", e.detail.checked)}
          />
        </div>

        <div class="setting-row">
          <div class="setting-icon">
            <Music size={20} />
          </div>
          <div class="setting-info">
            <div class="setting-label">显示频谱</div>
            <div class="setting-hint">显示音乐频谱动画效果</div>
          </div>
          <Toggle 
            checked={settings.show_spectrum}
            on:change={(e) => updateSetting("show_spectrum", e.detail.checked)}
          />
        </div>

        <div class="setting-row">
          <div class="setting-icon">
            <Check size={20} />
          </div>
          <div class="setting-info">
            <div class="setting-label">始终置顶</div>
            <div class="setting-hint">保持灵动岛在所有窗口之上</div>
          </div>
          <Toggle 
            checked={settings.always_on_top}
            on:change={(e) => updateSetting("always_on_top", e.detail.checked)}
          />
        </div>
      </Card>
    </section>

    <section class="settings-section">
      <h2 class="section-title">播放器管理</h2>
      
      <Card variant="default" padding="none" class="setting-card">
        <div class="setting-row">
          <div class="setting-info">
            <div class="setting-label">播放器优先级</div>
            <div class="setting-hint">拖拽调整播放器检测优先级</div>
          </div>
        </div>
        
        <div class="player-list">
          {#each playerOrder as player, index}
            <div
              class="player-item {isDragging && index === dragIndex ? 'dragging' : ''}"
              draggable="true"
              on:dragstart={() => handlePlayerDragStart(index)}
              on:dragend={handlePlayerDrop}
              on:dragover|preventDefault
              on:dragenter={() => overIndex = index}
            >
              <div class="drag-handle">
                <GripVertical size={20} />
              </div>
              <div class="player-icon">
                {#if player === 'netease'}
                  <Music size={20} />
                {:else if player === 'spotify'}
                  <Music size={20} />
                {:else if player === 'bilibili'}
                  <Music size={20} />
                {:else if player === 'qqmusic'}
                  <Music size={20} />
                {:else if player === 'apple'}
                  <Music size={20} />
                {:else}
                  <Music size={20} />
                {/if}
              </div>
              <span class="player-name">{playerNames[player]}</span>
              {#if index === 0}
                <span class="priority-badge">最高</span>
              {/if}
            </div>
          {/each}
        </div>
      </Card>
    </section>

    <section class="settings-section">
      <h2 class="section-title">缓存管理</h2>
      
      <Card variant="default" padding="none" class="setting-card">
        <div class="setting-row danger">
          <div class="setting-icon danger">
            <Database size={20} />
          </div>
          <div class="setting-info">
            <div class="setting-label">清除缓存</div>
            <div class="setting-hint">清除所有本地缓存数据</div>
          </div>
          <Button variant="danger" size="sm" on:click={clearCache}>
            清除
          </Button>
        </div>
      </Card>
    </section>
  </div>
</div>

<style>
  @import '../../styles/variables.css';
  
  .settings-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--base-dark);
    color: var(--text-base);
    font-family: var(--font-family-ui);
  }
  
  .header {
    background: var(--base-dark);
    padding: var(--spacing-md) var(--spacing-xl);
    -webkit-app-region: drag;
    border-bottom: 1px solid var(--border-gray);
  }
  
  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .title {
    font-size: var(--text-2xl);
    font-weight: var(--font-bold);
    color: var(--text-base);
    font-family: var(--font-family-title);
    margin: 0;
  }
  
  .window-controls {
    display: flex;
    gap: var(--spacing-sm);
    -webkit-app-region: no-drag;
  }
  
  .icon-btn {
    width: 32px;
    height: 32px;
    border: none;
    background: var(--base-mid-gray);
    border-radius: var(--radius-circle);
    color: var(--text-base);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-base);
  }
  
  .icon-btn:hover {
    background: var(--base-card);
    transform: scale(1.05);
  }
  
  .icon-btn.close:hover {
    background: var(--text-negative);
  }
  
  .content {
    flex: 1;
    overflow-y: hidden;
    padding: 0;
    background: var(--base-dark);
    scrollbar-width: none;
    -ms-overflow-style: none;
  }
  
  .content::-webkit-scrollbar {
    display: none;
  }
  
  .preview-section {
    padding: var(--spacing-xxl);
    background: var(--base-dark-gray);
    border-bottom: 1px solid var(--border-gray);
  }
  
  .preview-title {
    font-size: var(--text-md);
    font-weight: var(--font-bold);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    margin-bottom: var(--spacing-lg);
    font-family: var(--font-family-ui);
  }
  
  .preview-container {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xxl);
    margin-bottom: var(--spacing-xl);
    width: 100%;
    align-items: flex-start;
  }
  
  .island-preview {
    width: 340px;
    border-radius: var(--radius-xl);
    background: var(--base-black);
    padding: var(--spacing-md);
    position: relative;
    overflow: hidden;
    transition: all var(--transition-slow);
    box-shadow: var(--shadow-heavy);
    flex-shrink: 0;
  }
  
  .island-preview.collapsed {
    height: 84px;
  }
  
  .island-preview.expanded {
    height: 220px;
    padding: var(--spacing-lg);
  }
  
  .island-content {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    height: 100%;
  }
  
  .island-content.expanded {
    flex-direction: column;
    gap: var(--spacing-sm);
  }
  
  .top-section {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    width: 100%;
  }
  
  .album-art-preview {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-card);
    background: linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 100%);
    flex-shrink: 0;
    position: relative;
    overflow: hidden;
    box-shadow: var(--shadow-medium);
  }
  
  .album-art-preview.expanded {
    width: 50px;
    height: 50px;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-medium);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .album-art-inner {
    width: 100%;
    height: 100%;
    background: linear-gradient(
      135deg,
      rgba(255, 255, 255, 0.1) 0%,
      transparent 50%,
      rgba(0, 0, 0, 0.2) 100%
    );
    position: relative;
  }
  
  .track-info-preview {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    flex: 1;
  }
  
  .track-name {
    font-size: var(--text-md);
    font-weight: var(--font-bold);
    color: var(--text-base);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .artist-name {
    font-size: var(--text-sm);
    font-weight: var(--font-normal);
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .playback-controls {
    display: flex;
    gap: var(--spacing-sm);
  }
  
  .playback-controls-expanded {
    display: flex;
    gap: var(--spacing-md);
    width: 100%;
    justify-content: center;
  }
  
  .control-btn-preview {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-circle);
    background: var(--base-mid-gray);
    border: none;
    color: var(--text-base);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-base);
  }
  
  .control-btn-preview:hover {
    background: var(--base-card);
    transform: scale(1.05);
  }
  
  .progress-bar-preview {
    width: 100%;
    height: 3px;
    background: var(--base-mid-gray);
    border-radius: var(--radius-pill);
    overflow: hidden;
  }
  
  .progress-fill-preview {
    width: 30%;
    height: 100%;
    background: var(--accent-green);
    border-radius: var(--radius-pill);
  }
  
  .settings-section {
    padding: var(--spacing-xl);
    background: var(--base-dark);
  }
  
  .section-title {
    font-size: var(--text-md);
    font-weight: var(--font-bold);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    margin-bottom: var(--spacing-lg);
    font-family: var(--font-family-ui);
  }
  
  .setting-card {
    margin-bottom: var(--spacing-xl);
  }
  
  .setting-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-lg);
    border-bottom: 1px solid var(--border-gray);
    transition: background var(--transition-base);
  }
  
  .setting-row:last-child {
    border-bottom: none;
  }
  
  .setting-row:hover {
    background: var(--base-mid-gray);
  }
  
  .setting-row.danger:hover {
    background: rgba(243, 114, 127, 0.1);
  }
  
  .setting-icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--base-mid-gray);
    border-radius: var(--radius-circle);
    color: var(--text-secondary);
    flex-shrink: 0;
  }
  
  .setting-icon.danger {
    background: rgba(243, 114, 127, 0.1);
    color: var(--text-negative);
  }
  
  .setting-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    min-width: 0;
  }
  
  .setting-label {
    font-size: var(--text-lg);
    font-weight: var(--font-bold);
    color: var(--text-base);
    font-family: var(--font-family-ui);
  }
  
  .setting-hint {
    font-size: var(--text-md);
    color: var(--text-secondary);
  }
  
  .theme-selector {
    display: flex;
    gap: var(--spacing-sm);
  }
  
  .theme-option {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-lg);
    border: none;
    border-radius: var(--radius-pill);
    background: transparent;
    cursor: pointer;
    transition: all var(--transition-base);
    position: relative;
    font-family: var(--font-family-ui);
    font-size: var(--text-md);
    font-weight: var(--font-bold);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    color: var(--text-secondary);
  }
  
  .theme-option:hover {
    background: var(--base-mid-gray);
  }
  
  .theme-option.active {
    background: var(--accent-green);
    color: var(--base-dark);
  }
  
  .theme-icon-wrapper {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-base);
  }
  
  .theme-option.active .theme-icon-wrapper {
    color: var(--base-dark);
  }
  
  .theme-label {
    font-size: var(--text-md);
    font-weight: var(--font-bold);
    color: var(--text-base);
  }
  
  .theme-option.active .theme-label {
    color: var(--base-dark);
  }
  
  .player-list {
    display: flex;
    flex-direction: column;
    gap: 0;
    background: var(--base-dark-gray);
    border-radius: var(--radius-card);
    overflow: hidden;
    box-shadow: var(--shadow-medium);
  }
  
  .player-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-lg);
    background: transparent;
    border-bottom: 1px solid var(--border-gray);
    cursor: grab;
    transition: all var(--transition-base);
  }
  
  .player-item:last-child {
    border-bottom: none;
  }
  
  .player-item:hover {
    background: var(--base-mid-gray);
  }
  
  .player-item.dragging {
    opacity: 0.5;
    background: var(--base-card);
  }
  
  .drag-handle {
    color: var(--text-secondary);
    cursor: grab;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .player-icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--base-mid-gray);
    border-radius: var(--radius-circle);
    color: var(--text-base);
  }
  
  .player-name {
    flex: 1;
    font-size: var(--text-md);
    font-weight: var(--font-bold);
    color: var(--text-base);
  }
  
  .priority-badge {
    font-size: var(--text-xs);
    font-weight: var(--font-bold);
    color: var(--accent-green);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    padding: var(--spacing-xs) var(--spacing-sm);
    background: rgba(30, 215, 96, 0.1);
    border-radius: var(--radius-pill);
  }
  
  .danger-text {
    color: var(--text-negative);
  }
</style>
