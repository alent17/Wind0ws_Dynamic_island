<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    Settings as SettingsIcon,
    X,
    Minus,
    Check,
    Sliders,
    Palette,
    Bell,
    Eye,
    Cpu,
    Gauge,
    Bug,
    FileText,
    MonitorOff,
    Zap,
    Sparkles,
    Music,
    ChevronRight,
    Circle,
    Moon,
    Sun,
    Stars,
    Waves,
    Trees,
    Image,
    Grid,
    Database,
    Folder,
    Trash2,
  } from "lucide-svelte";

  interface AppSettings {
    island_theme: string;
    auto_hide: boolean;
    show_spectrum: boolean;
    enable_animations: boolean;
    window_opacity: number;
    always_on_top: boolean;
    hardware_acceleration: boolean;
    reduce_animations: boolean;
    show_debug_info: boolean;
    log_level: string;
    player_weights: Record<string, number>;
    enable_mv_playback: boolean; // MV 播放功能
    lock_floating_window: boolean; // 锁定悬浮窗（禁止移动）
    enable_hd_cover: boolean; // 高清专辑封面获取
    enable_pixel_art: boolean; // 像素化专辑封面
  }

  let settings = $state<AppSettings>({
    island_theme: "original",
    auto_hide: true,
    show_spectrum: true,
    enable_animations: true,
    window_opacity: 255,
    always_on_top: true,
    hardware_acceleration: true,
    reduce_animations: false,
    show_debug_info: false,
    log_level: "Info",
    player_weights: {
      netease: 50,
      spotify: 50,
      bilibili: 50,
      qqmusic: 50,
      apple: 50,
      generic: 10,
    },
    enable_mv_playback: false, // 默认关闭 MV 播放
    lock_floating_window: false, // 默认不锁定悬浮窗
    enable_hd_cover: true, // 默认开启高清封面获取
    enable_pixel_art: false, // 默认关闭像素化
  });

  const appWindow = getCurrentWindow();
  let opacityValue = $state(255);
  let ready = $state(false);

  const logLevels = ["Debug", "Info", "Warn", "Error"];
  let showLogLevelMenu = $state(false);

  onMount(async () => {
    try {
      const saved = await invoke<AppSettings>("get_settings");
      settings = { ...settings, ...saved };
      opacityValue = saved.window_opacity ?? 255;

      if (saved.player_weights) {
        settings.player_weights = saved.player_weights;
      }
    } catch (e) {
      console.error("无法读取设置", e);
    }
    setTimeout(() => (ready = true), 50);
  });

  async function saveSettings(patch: Partial<AppSettings>) {
    settings = { ...settings, ...patch };
    try {
      await invoke("save_settings", { settings });
      if (patch.island_theme) {
        // 发送到主窗口应用主题（使用全局事件）
        await invoke("emit_event", {
          event: "theme-changed",
          payload: { islandTheme: settings.island_theme },
        });
        console.log("[设置] 主题已切换并保存:", settings.island_theme);
      }
      if (patch.enable_hd_cover !== undefined) {
        // 发送到悬浮窗应用高清封面设置
        await invoke("emit_event", {
          event: "hd-cover-changed",
          payload: { enableHDCover: settings.enable_hd_cover },
        });
        console.log(
          "[设置] 高清封面获取已切换并保存:",
          settings.enable_hd_cover,
        );
      }
      if (patch.enable_pixel_art !== undefined) {
        // 发送到悬浮窗应用像素化封面设置
        await invoke("emit_event", {
          event: "pixel-art-changed",
          payload: { enablePixelArt: settings.enable_pixel_art },
        });
        console.log(
          "[设置] 像素化封面已切换并保存:",
          settings.enable_pixel_art,
        );
      }
    } catch (e) {
      console.error("保存失败", e);
    }
  }

  async function updatePlayerWeight(player: string, weight: number) {
    try {
      await invoke("set_player_weight", { player, weight });
      settings.player_weights[player] = weight;
    } catch (e) {
      console.error("更新权重失败", e);
    }
  }

  $effect(() => {
    if (ready && opacityValue !== settings.window_opacity) {
      saveSettings({ window_opacity: opacityValue });
    }
  });

  function selectLogLevel(level: string) {
    saveSettings({ log_level: level });
    showLogLevelMenu = false;
  }

  function handleGlobalClick(e: MouseEvent) {
    if (
      showLogLevelMenu &&
      !(e.target as HTMLElement).closest(".log-level-selector")
    ) {
      showLogLevelMenu = false;
    }
  }

  onMount(() => {
    document.addEventListener("click", handleGlobalClick);
    loadCacheStats();
    loadCacheDirectory();
    return () => document.removeEventListener("click", handleGlobalClick);
  });

  const themes = [
    {
      id: "original",
      name: "极简经典",
      desc: "Apple 原始质感设计",
      gradient: "from-gray-700 to-gray-900",
      accent: "bg-gray-500",
      icon: "circle",
      color: "#6b7280",
    },
  ];

  const playerIcons: Record<string, string> = {
    netease: "/src/assets/icons/netease.svg",
    spotify: "/src/assets/icons/spotify.svg",
    bilibili: "/src/assets/icons/bilibili.svg",
    qqmusic: "/src/assets/icons/qqmusic.svg",
    apple: "/src/assets/icons/apple_music.svg",
    generic: "/src/assets/icons/default_music.svg",
  };

  const playerNames: Record<string, string> = {
    netease: "网易云音乐",
    spotify: "Spotify",
    bilibili: "Bilibili",
    qqmusic: "QQ 音乐",
    apple: "Apple Music",
    generic: "其他播放器",
  };

  // 缓存管理函数
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
        cachePathEl.title = cachePath; // 显示完整路径的 tooltip
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
      alert("选择缓存目录失败：" + e);
    }
  }

  async function clearCache() {
    if (!confirm("确定要清除所有缓存文件吗？此操作不可恢复。")) {
      return;
    }

    try {
      await invoke("clear_cache");
      await loadCacheStats();
      alert("缓存已清除！");
    } catch (e) {
      console.error("清除缓存失败", e);
      alert("清除缓存失败：" + e);
    }
  }
</script>

<div class="settings-container">
  <!-- 背景装饰 -->
  <div class="bg-decoration">
    <div class="gradient-orb gradient-orb-1"></div>
    <div class="gradient-orb gradient-orb-2"></div>
    <div class="gradient-orb gradient-orb-3"></div>
  </div>

  <!-- 主内容区 -->
  <div class="main-content">
    <!-- 头部 -->
    <header class="header">
      <div class="header-content">
        <div class="header-icon-wrapper">
          <SettingsIcon size={24} class="header-icon" />
        </div>
        <div class="header-text">
          <h1 class="title">设置</h1>
          <p class="subtitle">个性化您的灵动岛体验</p>
        </div>
      </div>
      <div class="window-controls">
        <button
          class="win-btn minimize-btn"
          onclick={async () => {
            try {
              await appWindow.minimize();
              console.log("[窗口] 已最小化");
            } catch (error) {
              console.error("[窗口] 最小化失败:", error);
            }
          }}
          title="最小化"
        >
          <Minus size={18} />
        </button>
        <button
          class="win-btn close-btn"
          onclick={async () => {
            try {
              await appWindow.close();
              console.log("[窗口] 已关闭");
            } catch (error) {
              console.error("[窗口] 关闭失败:", error);
            }
          }}
          title="关闭"
        >
          <X size={18} />
        </button>
      </div>
    </header>

    <!-- 设置内容 -->
    <div class="settings-scroll">
      <div class="settings-inner">
        <!-- 灵动岛主题 -->
        <section class="settings-section">
          <div class="section-header">
            <div class="header-icon-box amber">
              <Palette size={20} />
            </div>
            <div>
              <h2 class="section-title">灵动岛主题</h2>
              <p class="section-desc">选择你喜欢的外观风格</p>
            </div>
          </div>
          <div class="theme-grid">
            {#each themes as t, i}
              <button
                class="theme-card"
                class:active={settings.island_theme === t.id}
                style="animation-delay: {i * 80}ms"
                onclick={() => saveSettings({ island_theme: t.id })}
              >
                <div class="theme-preview">
                  <div class={`theme-gradient ${t.gradient}`}></div>
                  <div class="theme-icon-wrapper">
                    {#if t.icon === "circle"}
                      <Circle
                        size={28}
                        strokeWidth={2}
                        class="theme-icon"
                        style="color: {t.color}"
                      />
                    {:else if t.icon === "sparkles"}
                      <Sparkles
                        size={28}
                        strokeWidth={2}
                        class="theme-icon"
                        style="color: {t.color}"
                      />
                    {:else if t.icon === "moon"}
                      <Moon
                        size={28}
                        strokeWidth={2}
                        class="theme-icon"
                        style="color: {t.color}"
                      />
                    {:else if t.icon === "zap"}
                      <Zap
                        size={28}
                        strokeWidth={2}
                        class="theme-icon"
                        style="color: {t.color}"
                      />
                    {:else if t.icon === "stars"}
                      <Stars
                        size={28}
                        strokeWidth={2}
                        class="theme-icon"
                        style="color: {t.color}"
                      />
                    {:else if t.icon === "waves"}
                      <Waves
                        size={28}
                        strokeWidth={2}
                        class="theme-icon"
                        style="color: {t.color}"
                      />
                    {:else if t.icon === "sun"}
                      <Sun
                        size={28}
                        strokeWidth={2}
                        class="theme-icon"
                        style="color: {t.color}"
                      />
                    {:else if t.icon === "tree"}
                      <Trees
                        size={28}
                        strokeWidth={2}
                        class="theme-icon"
                        style="color: {t.color}"
                      />
                    {/if}
                  </div>
                  {#if settings.island_theme === t.id}
                    <div class="theme-check">
                      <Check size={16} strokeWidth={3} />
                    </div>
                  {/if}
                </div>
                <div class="theme-info">
                  <span class="theme-name">{t.name}</span>
                  <span class="theme-desc">{t.desc}</span>
                </div>
              </button>
            {/each}
          </div>
        </section>

        <!-- 外观与行为 -->
        <section class="settings-section">
          <div class="section-header">
            <div class="header-icon-box rose">
              <Sparkles size={20} />
            </div>
            <div>
              <h2 class="section-title">外观与行为</h2>
              <p class="section-desc">调整显示效果和交互体验</p>
            </div>
          </div>

          <!-- 显示设置子组 -->
          <div class="settings-subgroup">
            <h3 class="subgroup-title">显示设置</h3>
            <div class="settings-list">
              <div class="settings-list">
                <!-- 自动隐藏 -->
                <div
                  class="setting-item"
                  onclick={() =>
                    saveSettings({ auto_hide: !settings.auto_hide })}
                >
                  <div class="item-icon" class:active={settings.auto_hide}>
                    <MonitorOff size={18} />
                  </div>
                  <div class="item-content">
                    <div class="item-header">
                      <h3 class="item-title">自动隐藏</h3>
                      <span class="item-value"
                        >{settings.auto_hide ? "开启" : "关闭"}</span
                      >
                    </div>
                    <p class="item-desc">检测到全屏应用时自动隐藏灵动岛</p>
                  </div>
                  <div class="toggle" class:on={settings.auto_hide}>
                    <div class="toggle-knob"></div>
                  </div>
                </div>

                <!-- 显示频谱 -->
                <div
                  class="setting-item"
                  onclick={() =>
                    saveSettings({ show_spectrum: !settings.show_spectrum })}
                >
                  <div class="item-icon" class:active={settings.show_spectrum}>
                    <Sliders size={18} />
                  </div>
                  <div class="item-content">
                    <div class="item-header">
                      <h3 class="item-title">显示频谱</h3>
                      <span class="item-value"
                        >{settings.show_spectrum ? "开启" : "关闭"}</span
                      >
                    </div>
                    <p class="item-desc">显示音乐频谱动画效果</p>
                  </div>
                  <div class="toggle" class:on={settings.show_spectrum}>
                    <div class="toggle-knob"></div>
                  </div>
                </div>

                <!-- 启用动画 -->
                <div
                  class="setting-item"
                  onclick={() =>
                    saveSettings({
                      enable_animations: !settings.enable_animations,
                    })}
                >
                  <div
                    class="item-icon"
                    class:active={settings.enable_animations}
                  >
                    <Zap size={18} />
                  </div>
                  <div class="item-content">
                    <div class="item-header">
                      <h3 class="item-title">启用动画</h3>
                      <span class="item-value"
                        >{settings.enable_animations ? "开启" : "关闭"}</span
                      >
                    </div>
                    <p class="item-desc">启用流畅的过渡动画效果</p>
                  </div>
                  <div class="toggle" class:on={settings.enable_animations}>
                    <div class="toggle-knob"></div>
                  </div>
                </div>

                <!-- 窗口透明度 -->
                <div class="setting-item slider-item">
                  <div class="item-icon">
                    <Eye size={18} />
                  </div>
                  <div class="item-content">
                    <div class="item-header">
                      <h3 class="item-title">窗口透明度</h3>
                      <span class="item-value">{opacityValue}</span>
                    </div>
                    <p class="item-desc">调整灵动岛的背景透明度</p>
                    <div class="slider-container">
                      <input
                        type="range"
                        min="0"
                        max="255"
                        value={opacityValue}
                        oninput={(e) =>
                          (opacityValue = parseInt(
                            (e.target as HTMLInputElement).value,
                          ))}
                        class="slider"
                      />
                      <div
                        class="slider-fill"
                        style="width: {(opacityValue / 255) * 100}%"
                      ></div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- 高级设置 -->
        <section class="settings-section">
          <div class="section-header">
            <div class="header-icon-box violet">
              <Cpu size={20} />
            </div>
            <div>
              <h2 class="section-title">高级设置</h2>
              <p class="section-desc">进阶功能与性能优化</p>
            </div>
          </div>

          <div class="settings-list">
            <!-- 始终置顶 -->
            <div
              class="setting-item"
              onclick={() =>
                saveSettings({ always_on_top: !settings.always_on_top })}
            >
              <div class="item-icon" class:active={settings.always_on_top}>
                <Bell size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">始终置顶</h3>
                  <span class="item-value"
                    >{settings.always_on_top ? "开启" : "关闭"}</span
                  >
                </div>
                <p class="item-desc">保持灵动岛显示在所有窗口上方</p>
              </div>
              <div class="toggle" class:on={settings.always_on_top}>
                <div class="toggle-knob"></div>
              </div>
            </div>

            <!-- 硬件加速 -->
            <div
              class="setting-item"
              onclick={() =>
                saveSettings({
                  hardware_acceleration: !settings.hardware_acceleration,
                })}
            >
              <div
                class="item-icon"
                class:active={settings.hardware_acceleration}
              >
                <Cpu size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">硬件加速</h3>
                  <span class="item-value"
                    >{settings.hardware_acceleration ? "开启" : "关闭"}</span
                  >
                </div>
                <p class="item-desc">使用 GPU 加速渲染，提升性能</p>
              </div>
              <div class="toggle" class:on={settings.hardware_acceleration}>
                <div class="toggle-knob"></div>
              </div>
            </div>

            <!-- 减少动画 -->
            <div
              class="setting-item"
              onclick={() =>
                saveSettings({
                  reduce_animations: !settings.reduce_animations,
                })}
            >
              <div class="item-icon" class:active={settings.reduce_animations}>
                <Gauge size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">减少动画</h3>
                  <span class="item-value"
                    >{settings.reduce_animations ? "开启" : "关闭"}</span
                  >
                </div>
                <p class="item-desc">降低动画效果，提升响应速度</p>
              </div>
              <div class="toggle" class:on={settings.reduce_animations}>
                <div class="toggle-knob"></div>
              </div>
            </div>

            <!-- MV 播放 -->
            <div
              class="setting-item"
              onclick={async () => {
                const newValue = !settings.enable_mv_playback;
                await saveSettings({
                  enable_mv_playback: newValue,
                });
                // 刷新悬浮窗，让设置立即生效
                try {
                  await invoke("emit_event", {
                    event: "mv-playback-changed",
                    payload: { enable: newValue },
                  });
                  console.log(
                    "[设置] MV 播放已切换:",
                    newValue ? "开启" : "关闭",
                  );
                } catch (e) {
                  console.error("[设置] 发送事件失败:", e);
                }
              }}
            >
              <div class="item-icon" class:active={settings.enable_mv_playback}>
                <MonitorOff size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">MV 播放</h3>
                  <span class="item-value"
                    >{settings.enable_mv_playback ? "开启" : "关闭"}</span
                  >
                </div>
                <p class="item-desc">
                  在专辑封面处播放 Apple Music MV（需网络）
                </p>
              </div>
              <div class="toggle" class:on={settings.enable_mv_playback}>
                <div class="toggle-knob"></div>
              </div>
            </div>

            <!-- 高清封面获取 -->
            <div
              class="setting-item"
              onclick={async () => {
                const newValue = !settings.enable_hd_cover;
                await saveSettings({
                  enable_hd_cover: newValue,
                });
                console.log(
                  "[设置] 高清封面获取已切换:",
                  newValue ? "开启" : "关闭",
                );
              }}
            >
              <div class="item-icon" class:active={settings.enable_hd_cover}>
                <Image size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">高清封面获取</h3>
                  <span class="item-value"
                    >{settings.enable_hd_cover ? "开启" : "关闭"}</span
                  >
                </div>
                <p class="item-desc">从网络获取高清专辑封面（需网络）</p>
              </div>
              <div class="toggle" class:on={settings.enable_hd_cover}>
                <div class="toggle-knob"></div>
              </div>
            </div>

            <!-- 像素化封面 -->
            <div
              class="setting-item"
              onclick={async () => {
                const newValue = !settings.enable_pixel_art;
                await saveSettings({
                  enable_pixel_art: newValue,
                });
                console.log(
                  "[设置] 像素化封面已切换:",
                  newValue ? "开启" : "关闭",
                );
              }}
            >
              <div class="item-icon" class:active={settings.enable_pixel_art}>
                <Grid size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">像素化封面</h3>
                  <span class="item-value"
                    >{settings.enable_pixel_art ? "开启" : "关闭"}</span
                  >
                </div>
                <p class="item-desc">将专辑封面像素化显示（可搭配高清获取）</p>
              </div>
              <div class="toggle" class:on={settings.enable_pixel_art}>
                <div class="toggle-knob"></div>
              </div>
            </div>

            <!-- 锁定悬浮窗 -->
            <div
              class="setting-item"
              onclick={async () => {
                const newValue = !settings.lock_floating_window;
                await saveSettings({
                  lock_floating_window: newValue,
                });
                // 刷新悬浮窗，让设置立即生效
                try {
                  await invoke("emit_event", {
                    event: "lock-floating-window-changed",
                    payload: { lock: newValue },
                  });
                  console.log(
                    "[设置] 锁定悬浮窗已切换:",
                    newValue ? "锁定" : "解锁",
                  );
                } catch (e) {
                  console.error("[设置] 发送事件失败:", e);
                }
              }}
            >
              <div
                class="item-icon"
                class:active={settings.lock_floating_window}
              >
                <Zap size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">锁定悬浮窗</h3>
                  <span class="item-value"
                    >{settings.lock_floating_window ? "锁定" : "解锁"}</span
                  >
                </div>
                <p class="item-desc">锁定后悬浮窗不能移动，顶部栏不显示</p>
              </div>
              <div class="toggle" class:on={settings.lock_floating_window}>
                <div class="toggle-knob"></div>
              </div>
            </div>

            <!-- 缓存管理 -->
            <div class="setting-section-title" style="margin-top: 20px;">
              <h2>缓存管理</h2>
            </div>

            <!-- 缓存大小 -->
            <div class="setting-item">
              <div class="item-icon">
                <Database size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">缓存大小</h3>
                  <span class="item-value" id="cache-size">加载中...</span>
                </div>
                <p class="item-desc">当前缓存的文件大小和数量</p>
              </div>
            </div>

            <!-- 缓存目录 -->
            <div class="setting-item">
              <div class="item-icon">
                <Folder size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">缓存位置</h3>
                </div>
                <p class="item-desc" id="cache-path" title="加载中...">
                  加载中...
                </p>
              </div>
              <button class="action-btn" onclick={pickCacheDir}>
                <span>更改</span>
              </button>
            </div>

            <!-- 清除缓存 -->
            <div class="setting-item">
              <div class="item-icon" style="color: #ff4444;">
                <Trash2 size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">清除缓存</h3>
                  <span class="item-value" style="color: #ff4444;"
                    >删除所有缓存文件</span
                  >
                </div>
                <p class="item-desc">释放磁盘空间</p>
              </div>
              <button class="action-btn danger" onclick={clearCache}>
                <span>清除</span>
              </button>
            </div>

            <!-- 调试信息 -->
            <div
              class="setting-item"
              onclick={() =>
                saveSettings({ show_debug_info: !settings.show_debug_info })}
            >
              <div class="item-icon" class:active={settings.show_debug_info}>
                <Bug size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">调试信息</h3>
                  <span class="item-value"
                    >{settings.show_debug_info ? "开启" : "关闭"}</span
                  >
                </div>
                <p class="item-desc">显示帧率等调试数据</p>
              </div>
              <div class="toggle" class:on={settings.show_debug_info}>
                <div class="toggle-knob"></div>
              </div>
            </div>

            <!-- 日志级别 -->
            <div class="setting-item">
              <div class="item-icon">
                <FileText size={18} />
              </div>
              <div class="item-content">
                <div class="item-header">
                  <h3 class="item-title">日志级别</h3>
                  <span class="item-value">{settings.log_level}</span>
                </div>
                <p class="item-desc">设置日志输出详细程度</p>
                <div class="log-level-selector">
                  <button
                    class="log-level-btn"
                    onclick={() => (showLogLevelMenu = !showLogLevelMenu)}
                  >
                    <span>{settings.log_level}</span>
                    <ChevronRight
                      size={16}
                      class={showLogLevelMenu
                        ? "chevron chevron-open"
                        : "chevron"}
                    />
                  </button>
                  {#if showLogLevelMenu}
                    <div class="log-level-menu">
                      {#each logLevels as level}
                        <button
                          class="log-level-option"
                          class:active={settings.log_level === level}
                          onclick={() => selectLogLevel(level)}
                        >
                          {level}
                          {#if settings.log_level === level}
                            <Check size={14} strokeWidth={3} />
                          {/if}
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- 播放器权重 -->
        <section class="settings-section">
          <div class="section-header">
            <div class="header-icon-box blue">
              <Music size={20} />
            </div>
            <div>
              <h2 class="section-title">播放器优先级</h2>
              <p class="section-desc">
                设置不同播放器的检测权重（权重越高，优先级越高）
              </p>
            </div>
          </div>

          <div class="players-list">
            {#each Object.entries(playerNames) as [player, name]}
              <div class="player-item">
                <div class="player-content">
                  <div class="player-header">
                    <h3 class="player-name">{name}</h3>
                    <div class="player-value-container">
                      <span class="player-value"
                        >{settings.player_weights?.[player] ?? 50}</span
                      >
                      <span class="player-percent">%</span>
                    </div>
                  </div>
                  <div class="player-slider-container">
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={settings.player_weights?.[player] ?? 50}
                      oninput={(e) =>
                        updatePlayerWeight(
                          player,
                          parseInt((e.target as HTMLInputElement).value),
                        )}
                      class="player-slider"
                    />
                    <div
                      class="player-slider-fill"
                      style="width: {((settings.player_weights?.[player] ??
                        50) /
                        100) *
                        100}%"
                    ></div>
                    <div class="player-slider-labels">
                      <span class="slider-label">0</span>
                      <span class="slider-label">50</span>
                      <span class="slider-label">100</span>
                    </div>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </section>
      </div>
    </div>
  </div>
</div>

<style>
  /* ========== 基础变量 ========== */
  :root {
    --bg-primary: #0a0a0f;
    --bg-secondary: #12121a;
    --bg-tertiary: #1a1a25;
    --surface: rgba(255, 255, 255, 0.05);
    --surface-hover: rgba(255, 255, 255, 0.08);
    --surface-active: rgba(255, 255, 255, 0.1);
    --border: rgba(255, 255, 255, 0.08);
    --border-hover: rgba(255, 255, 255, 0.15);
    --text-primary: #ffffff;
    --text-secondary: rgba(255, 255, 255, 0.7);
    --text-tertiary: rgba(255, 255, 255, 0.5);
    --accent: #8b5cf6;
    --accent-hover: #7c3aed;
    --accent-glow: rgba(139, 92, 246, 0.3);
    --success: #10b981;
    --warning: #f59e0b;
    --danger: #ef4444;
  }

  /* ========== 容器与布局 ========== */
  .settings-container {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: linear-gradient(135deg, var(--bg-primary) 0%, #0f0f1a 100%);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
      "Helvetica Neue", Arial, sans-serif;
    color: var(--text-primary);
  }

  /* 背景装饰 */
  .bg-decoration {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
  }

  .gradient-orb {
    position: absolute;
    border-radius: 50%;
    filter: blur(80px);
    opacity: 0.3;
    animation: float 20s ease-in-out infinite;
  }

  .gradient-orb-1 {
    width: 400px;
    height: 400px;
    background: radial-gradient(circle, #8b5cf6 0%, transparent 70%);
    top: -100px;
    right: -100px;
    animation-delay: 0s;
  }

  .gradient-orb-2 {
    width: 300px;
    height: 300px;
    background: radial-gradient(circle, #ec4899 0%, transparent 70%);
    bottom: -50px;
    left: -50px;
    animation-delay: -7s;
  }

  .gradient-orb-3 {
    width: 350px;
    height: 350px;
    background: radial-gradient(circle, #3b82f6 0%, transparent 70%);
    top: 50%;
    left: 50%;
    animation-delay: -14s;
  }

  @keyframes float {
    0%,
    100% {
      transform: translate(0, 0) scale(1);
    }
    33% {
      transform: translate(30px, -30px) scale(1.1);
    }
    66% {
      transform: translate(-20px, 20px) scale(0.9);
    }
  }

  /* 主内容区 */
  .main-content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    height: 100vh;
    backdrop-filter: blur(20px);
  }

  /* ========== 头部 ========== */
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 24px 32px;
    border-bottom: 1px solid var(--border);
    background: rgba(10, 10, 15, 0.8);
    -webkit-app-region: drag;
  }

  .header-content {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .header-icon-wrapper {
    width: 48px;
    height: 48px;
    border-radius: 14px;
    background: linear-gradient(135deg, var(--accent), var(--accent-hover));
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 8px 24px var(--accent-glow);
  }

  .header-icon {
    color: white;
  }

  .header-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .title {
    font-size: 24px;
    font-weight: 600;
    letter-spacing: -0.02em;
    margin: 0;
    background: linear-gradient(135deg, #fff, rgba(255, 255, 255, 0.7));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .subtitle {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 0;
    font-weight: 400;
  }

  .window-controls {
    display: flex;
    gap: 8px;
    -webkit-app-region: no-drag;
  }

  .win-btn {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    border: none;
    background: var(--surface);
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
  }

  .win-btn::before {
    content: "";
    position: absolute;
    inset: 0;
    background: radial-gradient(
      circle at center,
      rgba(255, 255, 255, 0.1) 0%,
      transparent 70%
    );
    opacity: 0;
    transition: opacity 0.25s;
  }

  .win-btn:hover::before {
    opacity: 1;
  }

  .win-btn:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .win-btn:active {
    transform: translateY(0) scale(0.95);
  }

  .close-btn:hover {
    background: var(--danger);
    color: white;
    box-shadow: 0 4px 16px rgba(239, 68, 68, 0.4);
  }

  .close-btn:active {
    transform: translateY(0) scale(0.9);
  }

  :global(.win-btn svg) {
    transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }

  :global(.win-btn.minimize-btn:hover svg) {
    transform: translateY(1px);
  }

  :global(.win-btn.close-btn:hover svg) {
    transform: rotate(90deg) scale(1.1);
  }

  /* ========== 滚动区域 ========== */
  .settings-scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .settings-scroll::-webkit-scrollbar {
    width: 6px;
  }

  .settings-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .settings-scroll::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }

  .settings-scroll::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .settings-inner {
    max-width: 800px;
    margin: 0 auto;
    padding: 32px;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  /* ========== 设置区块 ========== */
  .settings-section {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 24px;
    backdrop-filter: blur(10px);
    animation: slideUp 0.5s ease both;
  }

  .settings-section:nth-child(1) {
    animation-delay: 0ms;
  }
  .settings-section:nth-child(2) {
    animation-delay: 100ms;
  }
  .settings-section:nth-child(3) {
    animation-delay: 200ms;
  }
  .settings-section:nth-child(4) {
    animation-delay: 300ms;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 24px;
  }

  .header-icon-box {
    width: 42px;
    height: 42px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .header-icon-box.amber {
    background: rgba(245, 158, 11, 0.15);
    color: #f59e0b;
  }

  .header-icon-box.rose {
    background: rgba(236, 72, 153, 0.15);
    color: #ec4899;
  }

  .header-icon-box.violet {
    background: rgba(139, 92, 246, 0.15);
    color: #8b5cf6;
  }

  .header-icon-box.blue {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
  }

  .section-title {
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 4px 0;
    color: var(--text-primary);
  }

  .section-desc {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 0;
  }

  /* ========== 主题卡片 ========== */
  .theme-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .theme-card {
    position: relative;
    background: var(--bg-tertiary);
    border: 2px solid var(--border);
    border-radius: 16px;
    padding: 0;
    cursor: pointer;
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    animation: fadeIn 0.6s ease both;
  }

  .theme-card:hover {
    border-color: var(--border-hover);
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  }

  .theme-card.active {
    border-color: var(--accent);
    box-shadow:
      0 0 0 3px var(--accent-glow),
      0 8px 24px rgba(0, 0, 0, 0.3);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .theme-preview {
    position: relative;
    height: 100px;
    overflow: hidden;
  }

  .theme-gradient {
    position: absolute;
    inset: 0;
    transition: all 0.3s;
  }

  .theme-icon-wrapper {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(10px);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .theme-icon {
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
  }

  .theme-check {
    position: absolute;
    top: 12px;
    right: 12px;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .theme-info {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .theme-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .theme-desc {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  /* ========== 设置子组 ========== */
  .settings-subgroup {
    margin-bottom: 24px;
  }

  .settings-subgroup:last-child {
    margin-bottom: 0;
  }

  .subgroup-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);
    margin: 0 0 12px 0;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* ========== 设置列表 ========== */
  .settings-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .setting-item {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 20px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 16px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  }

  .setting-item:hover {
    background: var(--surface-hover);
    border-color: var(--border-hover);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
    transform: translateY(-1px);
  }

  .setting-item.slider-item {
    cursor: default;
  }

  .setting-item.slider-item:hover {
    transform: none;
  }

  .item-icon {
    width: 44px;
    height: 44px;
    border-radius: 12px;
    background: var(--surface);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    flex-shrink: 0;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }

  .item-icon.active {
    background: var(--accent);
    color: white;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .item-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .item-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .item-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    line-height: 1.3;
  }

  .item-value {
    font-size: 13px;
    color: var(--text-secondary);
    font-weight: 500;
    background: var(--surface);
    padding: 2px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
  }

  .item-desc {
    font-size: 13px;
    color: var(--text-tertiary);
    margin: 0;
  }

  /* ========== 开关 ========== */
  .toggle {
    position: relative;
    width: 52px;
    height: 28px;
    background: var(--surface);
    border: 2px solid var(--border);
    border-radius: 16px;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
    flex-shrink: 0;
  }

  .toggle:hover {
    border-color: var(--border-hover);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.08);
  }

  .toggle.on {
    background: var(--accent);
    border-color: var(--accent);
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .toggle-knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 50%;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
  }

  .toggle.on .toggle-knob {
    left: calc(100% - 22px);
    transform: scale(1.05);
  }

  /* ========== 滑块 ========== */
  .slider-container {
    position: relative;
    height: 36px;
    display: flex;
    align-items: center;
    margin-top: 12px;
  }

  .slider {
    position: absolute;
    width: 100%;
    height: 8px;
    border-radius: 4px;
    background: linear-gradient(90deg, var(--accent) 0%, var(--surface) 0%);
    outline: none;
    -webkit-appearance: none;
    cursor: pointer;
    z-index: 2;
    transition: all 0.2s;
  }

  .slider:hover {
    height: 10px;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
    border: 2px solid white;
    box-shadow: 0 4px 12px var(--accent-glow);
    transition: all 0.2s;
  }

  .slider::-webkit-slider-thumb:hover {
    transform: scale(1.1);
  }

  .slider::-moz-range-thumb {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
    border: 2px solid white;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .slider::-moz-range-thumb:hover {
    transform: scale(1.1);
  }

  .slider-fill {
    position: absolute;
    height: 8px;
    border-radius: 4px;
    background: var(--accent);
    pointer-events: none;
    z-index: 1;
  }

  /* ========== 日志级别选择器 ========== */
  .log-level-selector {
    position: relative;
    margin-top: 8px;
  }

  .log-level-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text-secondary);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .log-level-btn:hover {
    background: var(--surface-hover);
    border-color: var(--border-hover);
  }

  .chevron {
    transition: transform 0.2s;
  }

  .chevron-open {
    transform: rotate(90deg);
  }

  .log-level-menu {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 8px;
    min-width: 140px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 100;
    animation: slideDown 0.2s ease;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .log-level-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 10px 12px;
    background: none;
    border: none;
    border-radius: 6px;
    color: var(--text-secondary);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .log-level-option:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .log-level-option.active {
    background: var(--accent);
    color: white;
  }

  /* ========== 播放器列表 ========== */
  .players-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .player-item {
    display: flex;
    align-items: center;
    padding: 16px 20px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 14px;
    transition: all 0.2s;
  }

  .player-item:hover {
    background: var(--surface-hover);
    border-color: var(--border-hover);
  }

  .player-content {
    flex: 1;
    min-width: 0;
  }

  .player-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }

  .player-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.01em;
  }

  .player-value-container {
    display: flex;
    align-items: baseline;
    gap: 2px;
  }

  .player-value {
    font-size: 16px;
    color: var(--accent);
    font-weight: 700;
    min-width: 32px;
    text-align: right;
    font-feature-settings: "tnum";
    font-variant-numeric: tabular-nums;
  }

  .player-percent {
    font-size: 11px;
    color: var(--text-tertiary);
    font-weight: 500;
  }

  .player-slider-container {
    position: relative;
    height: 48px;
    display: flex;
    align-items: center;
    flex-direction: column;
    gap: 8px;
  }

  .player-slider {
    position: absolute;
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--surface);
    outline: none;
    -webkit-appearance: none;
    cursor: pointer;
    z-index: 2;
    top: 0;
  }

  .player-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
    box-shadow:
      0 4px 12px var(--accent-glow),
      0 0 0 2px rgba(255, 255, 255, 0.1);
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    border: 2px solid white;
  }

  .player-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
    box-shadow:
      0 6px 16px var(--accent-glow),
      0 0 0 2px rgba(255, 255, 255, 0.2);
  }

  .player-slider-fill {
    position: absolute;
    height: 6px;
    border-radius: 3px;
    background: linear-gradient(90deg, var(--accent), var(--accent-hover));
    pointer-events: none;
    z-index: 1;
    top: 0;
    box-shadow: 0 0 8px var(--accent-glow);
  }

  .player-slider-labels {
    display: flex;
    justify-content: space-between;
    width: 100%;
    margin-top: 24px;
  }

  .slider-label {
    font-size: 11px;
    color: var(--text-tertiary);
    font-weight: 500;
  }

  .action-btn {
    padding: 8px 16px;
    background: var(--accent-color);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
  }

  .action-btn:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .action-btn.danger {
    background: #ff4444;
  }

  .action-btn.danger:hover {
    background: #ff6666;
  }
</style>
