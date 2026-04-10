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
    initPlayerOrder();
    loadCacheStats();
    loadCacheDirectory();
  });

  async function saveSettings(patch: Partial<AppSettings>) {
    settings = { ...settings, ...patch };
    try {
      await invoke("save_settings", { settings });
      if (patch.island_theme) {
        await invoke("emit_event", {
          event: "theme-changed",
          payload: { islandTheme: settings.island_theme },
        });
      }
      if (patch.enable_hd_cover !== undefined) {
        await invoke("emit_event", {
          event: "hd-cover-changed",
          payload: { enableHDCover: settings.enable_hd_cover },
        });
      }
      if (patch.enable_pixel_art !== undefined) {
        await invoke("emit_event", {
          event: "pixel-art-changed",
          payload: { enablePixelArt: settings.enable_pixel_art },
        });
      }
    } catch (e) {
      console.error("保存失败", e);
    }
  }

  function initPlayerOrder() {
    const weights = settings.player_weights || {};
    playerOrder = Object.entries(weights)
      .sort((a, b) => b[1] - a[1])
      .map(([key]) => key);
    Object.keys(playerNames).forEach((key) => {
      if (!playerOrder.includes(key)) {
        playerOrder.push(key);
      }
    });
  }

  async function movePlayer(from: number, to: number) {
    if (from === to) return;
    const newOrder = [...playerOrder];
    const item = newOrder.splice(from, 1)[0];
    newOrder.splice(to, 0, item);
    playerOrder = newOrder;

    const total = playerOrder.length;
    const newWeights: Record<string, number> = {};
    playerOrder.forEach((player, index) => {
      newWeights[player] = Math.round(((total - index) / total) * 100);
    });

    settings.player_weights = newWeights;
    try {
      await invoke("save_settings", { settings });
      for (const [player, weight] of Object.entries(newWeights)) {
        await invoke("set_player_weight", { player, weight });
      }
    } catch (e) {
      console.error("保存排序失败", e);
    }
  }

  function handleDragStart(index: number) {
    dragIndex = index;
    isDragging = true;
  }

  function handleDragOver(index: number) {
    if (isDragging) {
      overIndex = index;
    }
  }

  function handleDragEnd() {
    if (dragIndex !== null && overIndex !== null && dragIndex !== overIndex) {
      movePlayer(dragIndex, overIndex);
    }
    dragIndex = null;
    overIndex = null;
    isDragging = false;
  }

  function moveUp(index: number) {
    if (index > 0) {
      movePlayer(index, index - 1);
    }
  }

  function moveDown(index: number) {
    if (index < playerOrder.length - 1) {
      movePlayer(index, index + 1);
    }
  }

  async function loadCacheStats() {
    try {
      const stats: any = await invoke("get_cache_stats");
      const cacheSizeEl = document.getElementById("cache-size");
      if (cacheSizeEl) {
        cacheSizeEl.textContent = `${stats.total_size_mb.toFixed(2)} MB (${stats.total_files} 个文件)`;
      }
    } catch (e) {
      console.error("加载缓存统计失败", e);
    }
  }

  async function loadCacheDirectory() {
    try {
      const cachePath: string = await invoke("get_cache_directory");
      const cachePathEl = document.getElementById("cache-path");
      if (cachePathEl) {
        cachePathEl.textContent = cachePath;
        cachePathEl.title = cachePath;
      }
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
</script>

<div class="settings-app">
  <div class="header">
    <div class="header-content">
      <h1 class="title">Settings</h1>
      <div class="window-controls">
        <button
          class="icon-btn"
          onclick={async () => await appWindow.minimize()}
          title="最小化"
        >
          <Minus size={18} />
        </button>
        <button
          class="icon-btn close"
          onclick={async () => await appWindow.close()}
          title="关闭"
        >
          <X size={18} />
        </button>
      </div>
    </div>
  </div>

  <div class="content">
    <div class="preview-section">
      <h2 class="preview-title">LIVE PREVIEW</h2>
      <div class="preview-container">
        <!-- 收起状态 -->
        <div
          class="island-preview"
          class:theme-original={settings.island_theme === "original"}
          class:theme-glassmorphism={settings.island_theme === "glassmorphism"}
        >
          <div class="island-content">
            <div class="album-art-preview">
              <div class="album-art-inner"></div>
            </div>
            <div class="track-info-preview">
              <div class="track-title-preview">Goodbye Henry. (feat. ...</div>
              <div class="artist-preview">RAVE</div>
            </div>
            {#if settings.show_spectrum}
              <div class="spectrum-preview">
                <div class="spectrum-bar"></div>
                <div class="spectrum-bar"></div>
                <div class="spectrum-bar"></div>
                <div class="spectrum-bar"></div>
                <div class="spectrum-bar"></div>
                <div class="spectrum-bar"></div>
              </div>
            {/if}
          </div>
        </div>

        <!-- 展开状态 -->
        <div
          class="island-preview expanded"
          class:theme-original={settings.island_theme === "original"}
          class:theme-glassmorphism={settings.island_theme === "glassmorphism"}
        >
          <div class="island-content expanded">
            <!-- 顶部区域：封面 + 标题 -->
            <div class="top-section">
              <div class="album-art-preview expanded">
                <div class="album-art-inner"></div>
              </div>
              <div class="track-info-preview expanded">
                <div class="track-title-preview">
                  Goodbye Henry. (feat. RAVE)
                </div>
                <div class="artist-preview">RAVE</div>
              </div>
            </div>

            <!-- 中部区域：播放控制按钮 -->
            <div class="controls-preview">
              <button class="control-btn-preview settings" aria-label="设置">
                <svg
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <circle cx="12" cy="12" r="3"></circle>
                  <path
                    d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
                  ></path>
                </svg>
              </button>

              <div class="playback-controls">
                <button class="control-btn-preview" aria-label="上一首">
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <polygon points="19 20 9 12 19 4 19 20"></polygon>
                    <line
                      x1="5"
                      y1="19"
                      x2="5"
                      y2="5"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    ></line>
                  </svg>
                </button>
                <button class="control-btn-preview play" aria-label="播放/暂停">
                  <svg
                    width="32"
                    height="32"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <polygon points="5 3 19 12 5 21 5 3"></polygon>
                  </svg>
                </button>
                <button class="control-btn-preview" aria-label="下一首">
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <polygon points="5 4 15 12 5 20 5 4"></polygon>
                    <line
                      x1="19"
                      y1="5"
                      x2="19"
                      y2="19"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    ></line>
                  </svg>
                </button>
              </div>

              <button class="control-btn-preview floating" aria-label="悬浮窗">
                <svg
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"
                  ></rect>
                  <line x1="8" y1="12" x2="16" y2="12"></line>
                  <line x1="12" y1="8" x2="12" y2="16"></line>
                </svg>
              </button>
            </div>

            <!-- 底部区域：频谱 -->
            {#if settings.show_spectrum}
              <div class="spectrum-preview-expanded">
                <div class="spectrum-bar-expanded"></div>
                <div class="spectrum-bar-expanded"></div>
                <div class="spectrum-bar-expanded"></div>
                <div class="spectrum-bar-expanded"></div>
                <div class="spectrum-bar-expanded"></div>
                <div class="spectrum-bar-expanded"></div>
                <div class="spectrum-bar-expanded"></div>
                <div class="spectrum-bar-expanded"></div>
                <div class="spectrum-bar-expanded"></div>
                <div class="spectrum-bar-expanded"></div>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
    <div class="section">
      <h2 class="section-title">Theme</h2>
      <div class="theme-selector">
        {#each themes as theme}
          <button
            class="theme-option"
            class:active={settings.island_theme === theme.id}
            onclick={() => saveSettings({ island_theme: theme.id })}
          >
            <div class="theme-icon-wrapper">
              <svelte:component this={theme.icon} size={20} />
            </div>
            <span class="theme-label">{theme.name}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="section">
      <h2 class="section-title">Appearance</h2>

      <div class="setting-card">
        <div class="setting-row">
          <div class="setting-icon">
            <Palette size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">显示频谱</span>
            <span class="setting-hint">显示音乐频谱动画效果</span>
          </div>
          <button
            class="toggle"
            class:active={settings.show_spectrum}
            onclick={() =>
              saveSettings({ show_spectrum: !settings.show_spectrum })}
          >
            <span class="toggle-knob"></span>
          </button>
        </div>
      </div>
    </div>

    <div class="section">
      <h2 class="section-title">General</h2>

      <div class="setting-card">
        <div class="setting-row">
          <div class="setting-icon">
            <Music size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">自动隐藏</span>
            <span class="setting-hint">检测到全屏应用时自动隐藏灵动岛</span>
          </div>
          <button
            class="toggle"
            class:active={settings.auto_hide}
            onclick={() => saveSettings({ auto_hide: !settings.auto_hide })}
          >
            <span class="toggle-knob"></span>
          </button>
        </div>

        <div class="setting-row">
          <div class="setting-icon">
            <Music size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">始终置顶</span>
            <span class="setting-hint">保持灵动岛显示在所有窗口上方</span>
          </div>
          <button
            class="toggle"
            class:active={settings.always_on_top}
            onclick={() =>
              saveSettings({ always_on_top: !settings.always_on_top })}
          >
            <span class="toggle-knob"></span>
          </button>
        </div>

        <div class="setting-row">
          <div class="setting-icon">
            <Music size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">开机启动</span>
            <span class="setting-hint">系统启动时自动启动灵动岛</span>
          </div>
          <button
            class="toggle"
            class:active={settings.auto_start}
            onclick={() => saveSettings({ auto_start: !settings.auto_start })}
          >
            <span class="toggle-knob"></span>
          </button>
        </div>
      </div>
    </div>

    <div class="section">
      <h2 class="section-title">Features</h2>

      <div class="setting-card">
        <div class="setting-row">
          <div class="setting-icon">
            <Music size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">MV 播放</span>
            <span class="setting-hint">在专辑封面处播放 Apple Music MV</span>
          </div>
          <button
            class="toggle"
            class:active={settings.enable_mv_playback}
            onclick={async () => {
              const newValue = !settings.enable_mv_playback;
              await saveSettings({ enable_mv_playback: newValue });
              try {
                await invoke("emit_event", {
                  event: "mv-playback-changed",
                  payload: { enable: newValue },
                });
              } catch (e) {
                console.error("[设置] 发送事件失败:", e);
              }
            }}
          >
            <span class="toggle-knob"></span>
          </button>
        </div>

        <div class="setting-row">
          <div class="setting-icon">
            <Music size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">高清封面获取</span>
            <span class="setting-hint">从网络获取高清专辑封面</span>
          </div>
          <button
            class="toggle"
            class:active={settings.enable_hd_cover}
            onclick={async () => {
              const newValue = !settings.enable_hd_cover;
              await saveSettings({ enable_hd_cover: newValue });
            }}
          >
            <span class="toggle-knob"></span>
          </button>
        </div>

        <div class="setting-row">
          <div class="setting-icon">
            <Music size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">像素化封面</span>
            <span class="setting-hint">将专辑封面像素化显示</span>
          </div>
          <button
            class="toggle"
            class:active={settings.enable_pixel_art}
            onclick={async () => {
              const newValue = !settings.enable_pixel_art;
              await saveSettings({ enable_pixel_art: newValue });
            }}
          >
            <span class="toggle-knob"></span>
          </button>
        </div>

        <div class="setting-row">
          <div class="setting-icon">
            <Music size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">锁定悬浮窗</span>
            <span class="setting-hint">锁定后悬浮窗不能移动</span>
          </div>
          <button
            class="toggle"
            class:active={settings.lock_floating_window}
            onclick={async () => {
              const newValue = !settings.lock_floating_window;
              await saveSettings({ lock_floating_window: newValue });
              try {
                await invoke("emit_event", {
                  event: "lock-floating-window-changed",
                  payload: { lock: newValue },
                });
              } catch (e) {
                console.error("[设置] 发送事件失败:", e);
              }
            }}
          >
            <span class="toggle-knob"></span>
          </button>
        </div>
      </div>
    </div>

    <div class="section">
      <h2 class="section-title">Players</h2>
      <p class="section-hint">拖拽排序，排名越靠前优先级越高</p>

      <div
        class="player-list"
        role="list"
        ondragover={(e) => e.preventDefault()}
      >
        {#each playerOrder as player, index (player)}
          <div
            class="player-item"
            class:dragging={dragIndex === index}
            class:over={overIndex === index && dragIndex !== index}
            onmousedown={() => handleDragStart(index)}
            onmouseover={() => handleDragOver(index)}
            onmouseup={handleDragEnd}
          >
            <div class="player-grip">
              <GripVertical size={16} />
            </div>
            <div class="player-info">
              <span class="player-name">{playerNames[player]}</span>
              <span class="player-weight">
                {settings.player_weights?.[player] ?? 50}%
              </span>
            </div>
            <div class="player-actions">
              <button
                class="move-btn"
                class:disabled={index === 0}
                onclick={() => moveUp(index)}
                disabled={index === 0}
              >
                <ArrowUp size={14} />
              </button>
              <button
                class="move-btn"
                class:disabled={index === playerOrder.length - 1}
                onclick={() => moveDown(index)}
                disabled={index === playerOrder.length - 1}
              >
                <ArrowDown size={14} />
              </button>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <div class="section">
      <h2 class="section-title">Storage</h2>

      <div class="setting-card">
        <div class="setting-row">
          <div class="setting-icon">
            <Database size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">缓存大小</span>
            <span class="setting-hint" id="cache-size">加载中...</span>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-icon">
            <Database size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label">缓存位置</span>
            <span class="setting-hint" id="cache-path" title="加载中...">
              加载中...
            </span>
          </div>
          <button class="link-btn" onclick={pickCacheDir}>
            更改
            <ChevronRight size={16} />
          </button>
        </div>

        <div class="setting-row danger">
          <div class="setting-icon danger">
            <Database size={18} />
          </div>
          <div class="setting-info">
            <span class="setting-label danger-text">清除缓存</span>
            <span class="setting-hint">删除所有缓存文件，释放磁盘空间</span>
          </div>
          <button class="danger-btn" onclick={clearCache}> 清除 </button>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  :root {
    --bg-app: #000000;
    --bg-card: #1c1c1e;
    --bg-card-secondary: #2c2c2e;
    --bg-hover: #3a3a3c;
    --border: #38383a;
    --text: #ffffff;
    --text-secondary: #8e8e93;
    --text-hint: #636366;
    --accent: #30d158;
    --accent-light: rgba(48, 209, 88, 0.15);
    --success: #30d158;
    --danger: #ff453a;
    --radius: 10px;
    --radius-sm: 8px;
    --radius-lg: 20px;
  }

  * {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  .settings-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-app);
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI",
      Roboto, sans-serif;
    color: var(--text);
    overflow: hidden;
  }

  /* ========== 顶部栏 ========== */
  .header {
    background: var(--bg-app);
    padding: 12px 20px;
    -webkit-app-region: drag;
    border-bottom: 0.5px solid var(--border);
  }

  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .title {
    font-size: 28px;
    font-weight: 700;
    color: var(--text);
  }

  .window-controls {
    display: flex;
    gap: 8px;
    -webkit-app-region: no-drag;
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    border: none;
    background: var(--bg-card-secondary);
    border-radius: 50%;
    color: var(--text);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .icon-btn:hover {
    background: var(--bg-hover);
  }

  .icon-btn.close:hover {
    background: var(--danger);
  }

  /* ========== 内容区 ========== */
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }

  .content::-webkit-scrollbar {
    width: 1px;
  }

  .content::-webkit-scrollbar-thumb {
    background: transparent;
  }

  /* ========== 预览窗口 ========== */
  .preview-section {
    padding: 20px;
    background: var(--bg-app);
  }

  .preview-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 16px;
  }

  .preview-container {
    display: flex;
    flex-direction: column;
    gap: 20px;
    margin-bottom: 20px;
    width: 100%;
    align-items: flex-start;
    padding: 0;
    background: transparent;
    border-radius: 0;
    border: none;
  }

  .island-preview {
    width: 340px;
    height: 84px;
    border-radius: 42px;
    background: #000000;
    padding: 10px 14px;
    position: relative;
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
    margin-bottom: 20px;
    flex-shrink: 0;
  }

  .island-preview:last-child {
    margin-bottom: 0;
  }

  .island-preview.theme-glassmorphism {
    background: var(--bg-card-secondary);
    backdrop-filter: blur(10px);
  }

  .island-preview.expanded {
    height: 220px;
    padding: 16px;
  }

  .island-content {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    height: 100%;
  }

  .island-content.expanded {
    flex-direction: column;
    gap: 8px;
    height: 100%;
  }

  .top-section {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
  }

  .album-art-preview {
    width: 64px;
    height: 64px;
    border-radius: 8px;
    background: linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 100%);
    flex-shrink: 0;
    position: relative;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .album-art-preview.expanded {
    width: 50px;
    height: 50px;
    border-radius: 12px;
    flex-shrink: 0;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
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

  .album-art-inner::before {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    width: 20px;
    height: 20px;
    background: radial-gradient(
      circle,
      rgba(255, 255, 255, 0.15) 0%,
      transparent 70%
    );
    border-radius: 50%;
    transform: translate(-50%, -50%);
  }

  .track-info-preview {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
    min-width: 0;
    margin-left: 8px;
    justify-content: center;
  }

  .track-info-preview.expanded {
    flex: 1;
    min-width: 0;
    gap: 4px;
    margin-left: 0;
  }

  .track-title-preview {
    font-size: 14px;
    font-weight: 700;
    color: #ffffff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    letter-spacing: -0.03em;
    line-height: 1.2;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
  }

  .artist-preview {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.6);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
    letter-spacing: -0.01em;
    line-height: 1.2;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      sans-serif;
  }

  .spectrum-preview {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 20px;
    padding: 0 4px;
    margin-right: 4px;
    margin-left: auto;
  }

  .spectrum-bar {
    flex: 1;
    background: linear-gradient(to top, #30d158, #34aada);
    border-radius: 1px;
    animation: spectrum 0.8s ease-in-out infinite;
    min-width: 2px;
    opacity: 0.9;
  }

  .spectrum-bar:nth-child(1) {
    animation-delay: 0s;
    height: 50%;
  }
  .spectrum-bar:nth-child(2) {
    animation-delay: 0.1s;
    height: 70%;
  }
  .spectrum-bar:nth-child(3) {
    animation-delay: 0.2s;
    height: 100%;
  }
  .spectrum-bar:nth-child(4) {
    animation-delay: 0.3s;
    height: 80%;
  }
  .spectrum-bar:nth-child(5) {
    animation-delay: 0.4s;
    height: 60%;
  }
  .spectrum-bar:nth-child(6) {
    animation-delay: 0.5s;
    height: 40%;
  }

  @keyframes spectrum {
    0%,
    100% {
      transform: scaleY(0.5);
      opacity: 0.6;
    }
    50% {
      transform: scaleY(1);
      opacity: 1;
    }
  }

  .controls-preview {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    flex: 1;
  }

  .control-btn-preview {
    width: 32px;
    height: 32px;
    border: none;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    color: rgba(255, 255, 255, 0.9);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .control-btn-preview:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: scale(1.1);
  }

  .control-btn-preview:active {
    transform: scale(0.95);
  }

  .control-btn-preview.play {
    width: 40px;
    height: 40px;
    background: transparent;
    color: #ffffff;
  }

  .control-btn-preview.play:hover {
    transform: scale(1.1);
  }

  .control-btn-preview.settings {
    color: rgba(255, 255, 255, 0.4);
    margin-left: 8px;
  }

  .control-btn-preview.settings:hover {
    color: rgba(255, 255, 255, 0.9);
  }

  .control-btn-preview.floating {
    width: 28px;
    height: 28px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: transparent;
    margin-right: 8px;
  }

  .control-btn-preview.floating:hover {
    border-color: rgba(255, 255, 255, 0.2);
    transform: scale(1.1);
  }

  .playback-controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 20px;
    width: 130px;
    flex-shrink: 0;
  }

  .spectrum-preview-expanded {
    display: flex;
    align-items: flex-end;
    justify-content: center;
    gap: 2px;
    height: 24px;
    width: 100%;
    margin-top: auto;
    margin-bottom: 6px;
  }

  .spectrum-bar-expanded {
    flex: 1;
    min-width: 2px;
    border-radius: 2px;
    background: linear-gradient(to top, #30d158, #34aada);
    animation: spectrum-expanded 1.5s ease-in-out infinite;
  }

  .spectrum-bar-expanded:nth-child(1) {
    animation: sp-e1 1.8s ease-in-out infinite;
  }
  .spectrum-bar-expanded:nth-child(2) {
    animation: sp-e2 1.6s ease-in-out infinite;
  }
  .spectrum-bar-expanded:nth-child(3) {
    animation: sp-e3 1.9s ease-in-out infinite;
  }
  .spectrum-bar-expanded:nth-child(4) {
    animation: sp-e4 1.5s ease-in-out infinite;
  }
  .spectrum-bar-expanded:nth-child(5) {
    animation: sp-e5 1.3s ease-in-out infinite;
  }
  .spectrum-bar-expanded:nth-child(6) {
    animation: sp-e6 1.7s ease-in-out infinite;
  }
  .spectrum-bar-expanded:nth-child(7) {
    animation: sp-e7 1.4s ease-in-out infinite;
  }
  .spectrum-bar-expanded:nth-child(8) {
    animation: sp-e8 1.6s ease-in-out infinite;
  }
  .spectrum-bar-expanded:nth-child(9) {
    animation: sp-e9 1.2s ease-in-out infinite;
  }
  .spectrum-bar-expanded:nth-child(10) {
    animation: sp-e10 1.5s ease-in-out infinite;
  }

  @keyframes sp-e1 {
    0%,
    100% {
      height: 0.5px;
    }
    15% {
      height: 18px;
    }
    35% {
      height: 6px;
    }
    55% {
      height: 22px;
    }
    75% {
      height: 3px;
    }
  }
  @keyframes sp-e2 {
    0%,
    100% {
      height: 1px;
    }
    20% {
      height: 16px;
    }
    40% {
      height: 8px;
    }
    60% {
      height: 20px;
    }
    80% {
      height: 4px;
    }
  }
  @keyframes sp-e3 {
    0%,
    100% {
      height: 0.5px;
    }
    25% {
      height: 14px;
    }
    50% {
      height: 5px;
    }
    75% {
      height: 18px;
    }
  }
  @keyframes sp-e4 {
    0%,
    100% {
      height: 1px;
    }
    30% {
      height: 15px;
    }
    60% {
      height: 7px;
    }
    90% {
      height: 19px;
    }
  }
  @keyframes sp-e5 {
    0%,
    100% {
      height: 0.5px;
    }
    20% {
      height: 13px;
    }
    40% {
      height: 9px;
    }
    60% {
      height: 17px;
    }
    80% {
      height: 3px;
    }
  }
  @keyframes sp-e6 {
    0%,
    100% {
      height: 1px;
    }
    25% {
      height: 12px;
    }
    50% {
      height: 6px;
    }
    75% {
      height: 16px;
    }
  }
  @keyframes sp-e7 {
    0%,
    100% {
      height: 0.5px;
    }
    30% {
      height: 14px;
    }
    60% {
      height: 8px;
    }
    90% {
      height: 18px;
    }
  }
  @keyframes sp-e8 {
    0%,
    100% {
      height: 1px;
    }
    20% {
      height: 11px;
    }
    40% {
      height: 5px;
    }
    60% {
      height: 15px;
    }
    80% {
      height: 2px;
    }
  }
  @keyframes sp-e9 {
    0%,
    100% {
      height: 0.5px;
    }
    25% {
      height: 13px;
    }
    50% {
      height: 7px;
    }
    75% {
      height: 17px;
    }
  }
  @keyframes sp-e10 {
    0%,
    100% {
      height: 1px;
    }
    30% {
      height: 12px;
    }
    60% {
      height: 6px;
    }
    90% {
      height: 16px;
    }
  }

  /* ========== 区块 ========== */
  .section {
    margin-bottom: 0;
    padding: 0 20px 20px;
  }

  .section-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 12px;
    padding: 0 4px;
  }

  .section-hint {
    font-size: 12px;
    color: var(--text-hint);
    margin-bottom: 12px;
  }

  /* ========== 主题选择器 ========== */
  .theme-selector {
    display: flex;
    gap: 10px;
    background: var(--bg-card);
    padding: 6px;
    border-radius: var(--radius-lg);
    margin-bottom: 20px;
  }

  .theme-option {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px 16px;
    border: none;
    border-radius: var(--radius);
    background: transparent;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
  }

  .theme-option:hover {
    background: var(--bg-hover);
  }

  .theme-option.active {
    background: var(--accent);
  }

  .theme-icon-wrapper {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text);
  }

  .theme-option.active .theme-icon-wrapper {
    color: white;
  }

  .theme-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
  }

  .theme-option.active .theme-label {
    color: white;
  }

  /* ========== 卡片 ========== */
  .setting-card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    overflow: hidden;
    margin-bottom: 20px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border-bottom: 0.5px solid var(--border);
    transition: background 0.15s ease;
  }

  .setting-row:last-child {
    border-bottom: none;
  }

  .setting-row:hover {
    background: var(--bg-hover);
  }

  .setting-row.danger:hover {
    background: rgba(255, 59, 48, 0.1);
  }

  .setting-icon {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card-secondary);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .setting-icon.danger {
    background: rgba(255, 59, 48, 0.1);
    color: var(--danger);
  }

  .setting-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .setting-label {
    font-size: 15px;
    font-weight: 500;
    color: var(--text);
  }

  .setting-hint {
    font-size: 12px;
    color: var(--text-hint);
  }

  .danger-text {
    color: var(--danger);
  }

  /* ========== 开关 ========== */
  .toggle {
    position: relative;
    width: 51px;
    height: 31px;
    border: none;
    background: var(--bg-card-secondary);
    border-radius: 16px;
    cursor: pointer;
    transition: background 0.2s ease;
    flex-shrink: 0;
  }

  .toggle.active {
    background: var(--accent);
  }

  .toggle-knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 25px;
    height: 25px;
    background: white;
    border-radius: 50%;
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.15);
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .toggle.active .toggle-knob {
    transform: translateX(20px);
  }

  /* ========== 链接按钮 ========== */
  .link-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--accent);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .link-btn:hover {
    background: var(--accent-light);
  }

  /* ========== 危险按钮 ========== */
  .danger-btn {
    padding: 8px 16px;
    background: var(--danger);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .danger-btn:hover {
    opacity: 0.9;
  }

  /* ========== 播放器列表 ========== */
  .player-list {
    display: flex;
    flex-direction: column;
    gap: 0;
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .player-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px;
    background: transparent;
    border-bottom: 0.5px solid var(--border);
    border-radius: 0;
    cursor: grab;
    transition: all 0.15s ease;
  }

  .player-item:last-child {
    border-bottom: none;
  }

  .player-item:hover {
    background: var(--bg-hover);
  }

  .player-item.dragging {
    opacity: 0.5;
    transform: scale(0.98);
    border-color: var(--accent);
  }

  .player-item.over {
    border-color: var(--accent);
    background: var(--accent-light);
  }

  .player-grip {
    color: var(--text-hint);
    cursor: grab;
    display: flex;
    align-items: center;
  }

  .player-info {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-width: 0;
  }

  .player-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
  }

  .player-weight {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
  }

  .player-actions {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .move-btn {
    width: 26px;
    height: 22px;
    border: none;
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.1s ease;
  }

  .move-btn:hover:not(.disabled) {
    background: var(--accent);
    color: white;
  }

  .move-btn.disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
