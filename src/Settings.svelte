<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";

  let activeTab = "general";
  let islandTheme = $state("original");
  let autoHide = $state(false);
  let showSpectrum = $state(true);
  let enableAnimations = $state(true);
  let windowOpacity = $state(100);
  let alwaysOnTop = $state(true);

  const tabs = [
    { id: "general", label: "常规", icon: "⚙️" },
    { id: "appearance", label: "外观", icon: "🎨" },
    { id: "behavior", label: "行为", icon: "⚡" },
    { id: "about", label: "关于", icon: "ℹ️" },
  ];

  const themes = [
    { id: "original", name: "经典黑", color: "#1a1a1a" },
    { id: "white", name: "纯净白", color: "#f5f5f5" },
    { id: "blue", name: "深海蓝", color: "#1e3a5f" },
    { id: "purple", name: "梦幻紫", color: "#4a1a5f" },
    { id: "green", name: "自然绿", color: "#1a5f3a" },
    { id: "red", name: "热情红", color: "#5f1a1a" },
  ];

  onMount(async () => {
    // 加载保存的设置
    try {
      const settings = await invoke("get_settings");
      islandTheme = settings.island_theme || "original";
      autoHide = settings.auto_hide || false;
      showSpectrum = settings.show_spectrum || true;
      enableAnimations = settings.enable_animations || true;
      windowOpacity = settings.window_opacity || 100;
      alwaysOnTop = settings.always_on_top || true;
    } catch (e) {
      console.error("加载设置失败:", e);
    }
  });

  async function saveSettings() {
    try {
      await invoke("save_settings", {
        settings: {
          island_theme: islandTheme,
          auto_hide: autoHide,
          show_spectrum: showSpectrum,
          enable_animations: enableAnimations,
          window_opacity: windowOpacity,
          always_on_top: alwaysOnTop,
        },
      });
      console.log("设置已保存");
    } catch (e) {
      console.error("保存设置失败:", e);
    }
  }

  async function closeSettings() {
    const win = getCurrentWindow();
    await win.close();
  }

  function handleTabChange(tabId: string) {
    activeTab = tabId;
  }
</script>

<div class="settings-container">
  <!-- 标题栏 -->
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-left">
      <h1 class="title">⚙️ 设置</h1>
    </div>
    <button class="close-btn" onclick={closeSettings}>✕</button>
  </div>

  <div class="settings-content">
    <!-- 侧边栏 -->
    <div class="sidebar">
      {#each tabs as tab}
        <button
          class="tab-button {activeTab === tab.id ? 'active' : ''}"
          onclick={() => handleTabChange(tab.id)}
        >
          <span class="tab-icon">{tab.icon}</span>
          <span class="tab-label">{tab.label}</span>
        </button>
      {/each}
    </div>

    <!-- 内容区域 -->
    <div class="content">
      {#if activeTab === "general"}
        <div class="tab-content">
          <h2 class="section-title">常规设置</h2>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>窗口置顶</h3>
              <p>保持灵动岛窗口始终在最上层</p>
            </div>
            <label class="toggle">
              <input type="checkbox" bind:checked={alwaysOnTop} onchange={saveSettings} />
              <span class="toggle-slider"></span>
            </label>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>窗口透明度</h3>
              <p>调整灵动岛窗口的透明程度</p>
            </div>
            <div class="slider-container">
              <input
                type="range"
                min="50"
                max="100"
                bind:value={windowOpacity}
                oninput={saveSettings}
              />
              <span class="slider-value">{windowOpacity}%</span>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>自动隐藏</h3>
              <p>鼠标离开后自动隐藏灵动岛</p>
            </div>
            <label class="toggle">
              <input type="checkbox" bind:checked={autoHide} onchange={saveSettings} />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
      {/if}

      {#if activeTab === "appearance"}
        <div class="tab-content">
          <h2 class="section-title">外观设置</h2>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>灵动岛主题</h3>
              <p>选择灵动岛的颜色主题</p>
            </div>
          </div>

          <div class="theme-grid">
            {#each themes as theme}
              <button
                class="theme-button {islandTheme === theme.id ? 'active' : ''}"
                onclick={() => {
                  islandTheme = theme.id;
                  saveSettings();
                }}
              >
                <div
                  class="theme-preview"
                  style="background-color: {theme.color}"
                ></div>
                <span class="theme-name">{theme.name}</span>
              </button>
            {/each}
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>显示频谱</h3>
              <p>在收起状态显示音乐频谱动画</p>
            </div>
            <label class="toggle">
              <input type="checkbox" bind:checked={showSpectrum} onchange={saveSettings} />
              <span class="toggle-slider"></span>
            </label>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>启用动画</h3>
              <p>开启流畅的展开/收起动画效果</p>
            </div>
            <label class="toggle">
              <input type="checkbox" bind:checked={enableAnimations} onchange={saveSettings} />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
      {/if}

      {#if activeTab === "behavior"}
        <div class="tab-content">
          <h2 class="section-title">行为设置</h2>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>展开宽度</h3>
              <p>设置灵动岛展开时的宽度</p>
            </div>
            <select class="select-input">
              <option value="320">320px (紧凑)</option>
              <option value="360" selected>360px (标准)</option>
              <option value="400">400px (宽敞)</option>
            </select>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>展开高度</h3>
              <p>设置灵动岛展开时的高度</p>
            </div>
            <select class="select-input">
              <option value="160">160px (紧凑)</option>
              <option value="180" selected>180px (标准)</option>
              <option value="200">200px (宽敞)</option>
            </select>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>点击区域</h3>
              <p>设置灵动岛的交互区域</p>
            </div>
            <select class="select-input">
              <option value="island">仅灵动岛</option>
              <option value="screen" selected>整个屏幕</option>
            </select>
          </div>
        </div>
      {/if}

      {#if activeTab === "about"}
        <div class="tab-content">
          <h2 class="section-title">关于</h2>
          
          <div class="about-content">
            <div class="app-icon">️</div>
            <h3 class="app-name">Windows Dynamic Island</h3>
            <p class="app-version">版本 1.0.0</p>
            
            <div class="app-description">
              <p>Windows 平台上的灵动岛体验</p>
              <p>支持全局媒体控制和实时频谱显示</p>
            </div>

            <div class="tech-stack">
              <div class="tech-item">
                <span class="tech-icon">🦀</span>
                <span>Tauri 2.0</span>
              </div>
              <div class="tech-item">
                <span class="tech-icon">⚡</span>
                <span>Svelte 5</span>
              </div>
              <div class="tech-item">
                <span class="tech-icon">🎨</span>
                <span>TypeScript</span>
              </div>
            </div>

            <div class="copyright">
              <p>Made with ❤️ for Windows</p>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    background: transparent;
    overflow: hidden;
  }

  .settings-container {
    width: 100vw;
    height: 100vh;
    background: linear-gradient(135deg, rgba(30, 30, 30, 0.95) 0%, rgba(20, 20, 20, 0.98) 100%);
    backdrop-filter: blur(20px);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .titlebar {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    user-select: none;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .title {
    font-size: 18px;
    font-weight: 600;
    color: #ffffff;
    letter-spacing: 0.3px;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: none;
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
    font-size: 20px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .close-btn:hover {
    background: rgba(255, 77, 77, 0.3);
  }

  .settings-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .sidebar {
    width: 220px;
    background: rgba(255, 255, 255, 0.03);
    border-right: 1px solid rgba(255, 255, 255, 0.08);
    padding: 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .tab-button {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 10px;
    border: none;
    background: transparent;
    color: rgba(255, 255, 255, 0.6);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
  }

  .tab-button:hover {
    background: rgba(255, 255, 255, 0.06);
    color: #ffffff;
  }

  .tab-button.active {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
  }

  .tab-icon {
    font-size: 18px;
  }

  .tab-label {
    flex: 1;
  }

  .content {
    flex: 1;
    padding: 32px;
    overflow-y: auto;
  }

  .section-title {
    font-size: 24px;
    font-weight: 700;
    color: #ffffff;
    margin-bottom: 32px;
    letter-spacing: 0.5px;
  }

  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .setting-item:last-child {
    border-bottom: none;
  }

  .setting-info {
    flex: 1;
  }

  .setting-info h3 {
    font-size: 15px;
    font-weight: 600;
    color: #ffffff;
    margin-bottom: 6px;
  }

  .setting-info p {
    font-size: 13px;
    color: rgba(255, 255, 255, 0.4);
    line-height: 1.5;
  }

  /* Toggle Switch */
  .toggle {
    position: relative;
    display: inline-block;
    width: 50px;
    height: 28px;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(255, 255, 255, 0.2);
    transition: 0.3s;
    border-radius: 28px;
  }

  .toggle-slider:before {
    position: absolute;
    content: "";
    height: 22px;
    width: 22px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  input:checked + .toggle-slider {
    background-color: #007AFF;
  }

  input:checked + .toggle-slider:before {
    transform: translateX(22px);
  }

  /* Slider */
  .slider-container {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  input[type="range"] {
    width: 200px;
    height: 6px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.1);
    outline: none;
    -webkit-appearance: none;
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #ffffff;
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .slider-value {
    font-size: 14px;
    font-weight: 600;
    color: #ffffff;
    min-width: 45px;
    text-align: right;
  }

  /* Select */
  .select-input {
    padding: 10px 16px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.05);
    color: #ffffff;
    font-size: 14px;
    cursor: pointer;
    outline: none;
  }

  .select-input:focus {
    border-color: rgba(255, 255, 255, 0.2);
  }

  /* Theme Grid */
  .theme-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
    margin-bottom: 24px;
  }

  .theme-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 16px;
    border-radius: 12px;
    border: 2px solid transparent;
    background: rgba(255, 255, 255, 0.03);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .theme-button:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .theme-button.active {
    border-color: #007AFF;
    background: rgba(0, 122, 255, 0.1);
  }

  .theme-preview {
    width: 48px;
    height: 48px;
    border-radius: 10px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .theme-name {
    font-size: 13px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.8);
  }

  /* About */
  .about-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 40px 0;
  }

  .app-icon {
    font-size: 80px;
    margin-bottom: 16px;
  }

  .app-name {
    font-size: 28px;
    font-weight: 700;
    color: #ffffff;
    margin-bottom: 8px;
  }

  .app-version {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.4);
    margin-bottom: 24px;
  }

  .app-description {
    margin-bottom: 32px;
    line-height: 1.8;
  }

  .app-description p {
    font-size: 15px;
    color: rgba(255, 255, 255, 0.6);
  }

  .tech-stack {
    display: flex;
    gap: 24px;
    margin-bottom: 32px;
  }

  .tech-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .tech-icon {
    font-size: 32px;
  }

  .tech-item span {
    font-size: 13px;
    color: rgba(255, 255, 255, 0.6);
  }

  .copyright {
    margin-top: 24px;
  }

  .copyright p {
    font-size: 13px;
    color: rgba(255, 255, 255, 0.3);
  }

  /* Scrollbar */
  .content::-webkit-scrollbar {
    width: 8px;
  }

  .content::-webkit-scrollbar-track {
    background: transparent;
  }

  .content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
  }

  .content::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }
</style>
