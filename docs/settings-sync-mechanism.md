# 统一设置更新机制

## 概述

本文档描述了灵动岛应用中实现的统一设置更新机制，实现了后端、设置页面和主窗口之间的实时同步。

## 架构设计

```
┌─────────────┐    save_settings     ┌─────────────┐
│ 设置页面    │ ───────────────────► │   Rust 后端  │
│ Settings    │                      │   lib.rs    │
└─────────────┘                      └──────┬──────┘
                                            │
                                   emit("settings-updated")
                                            │
                                            ▼
                                   ┌────────────────┐
                                   │  Tauri Event    │
                                   │  Bus            │
                                   └──────┬──────────┘
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    │                     │                     │
                    ▼                     ▼                     ▼
           ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
           │   主窗口      │      │  悬浮窗口     │      │ 其他监听器   │
           │   App.svelte │      │              │      │              │
           └──────────────┘      └──────────────┘      └──────────────┘
```

## 核心组件

### 1. Rust 后端 (`src-tauri/src/lib.rs`)

#### save_settings 命令

```rust
#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    // 1. 更新内存状态
    let state = app.state::<AppState>();
    {
        let mut state_settings = state.settings.lock().map_err(|_| "Failed to lock settings")?;
        *state_settings = settings.clone();
    }

    // 2. 持久化到文件
    write_settings_file(&app, &settings)?;

    // 3. 应用置顶设置
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(settings.always_on_top);
    }

    // 4. 更新开机启动状态
    if settings.auto_start {
        set_auto_start(app.clone(), true)?;
    } else {
        let _ = set_auto_start(app.clone(), false);
    }

    // 5. 广播完整设置更新事件
    let _ = app.emit("settings-updated", settings);

    Ok(())
}
```

**关键点：**
- 第 1098 行广播 `settings-updated` 事件
- 传递完整的 `settings` 对象作为 payload
- 其他单项变更事件（如 `halftone-changed`）仍然保留用于特殊处理

### 2. 主窗口 (`src/App.svelte`)

#### 事件监听

```typescript
// ===== 监听设置变更事件 =====
const unlistenSettings = await listen(
  "settings-updated",
  (event: any) => {
    const s = event.payload;
    if (s) {
      // 核心：直接用完整对象覆盖，Svelte 的响应式会自动触发 UI 更新
      appSettings = s;
      console.log("[设置] 实时更新:", appSettings);

      // auto_hide 关闭时立即显示窗口
      if (!appSettings.auto_hide && isHidden) {
        showWindow();
      }
    }
  },
);
```

#### appSettings 默认值

```typescript
let appSettings = $state({
  auto_hide: true,
  show_spectrum: true,
  enable_animations: true,
  reduce_animations: false,
  show_debug_info: false,
  window_opacity: 255,
  hide_settings_button: false,
  hide_monitor_selector: false,
  hide_floating_window: false,
  expanded_corner_radius: 16,
  always_show_top_bar: true,
  always_on_top: true,
  // 播放器权重
  player_weights: {
    netease: 50,
    spotify: 50,
    bilibili: 50,
    qqmusic: 50,
    apple: 50,
    generic: 10,
  },
  // 显示器设置
  monitor_index: 0,
  // 专辑封面设置
  enable_hd_cover: true,
  enable_pixel_art: false,
  enable_halftone: false,
  // MV 播放
  enable_mv_playback: true,
  // 悬浮窗锁定
  lock_floating_window: false,
  // 开机启动
  auto_start: false,
});
```

### 3. 设置页面 (`src/Settings.svelte`)

#### 即时保存函数

```typescript
async function saveSettings(newSettings: Partial<AppSettings>) {
  try {
    settings = { ...settings, ...newSettings };
    await invoke("save_settings", { settings });
  } catch (e) {
    console.error("保存设置失败", e);
  }
}
```

#### UI 组件使用示例

```svelte
<!-- Toggle 开关 -->
<label class="tg">
  <input
    type="checkbox"
    checked={settings.show_spectrum}
    onchange={() => saveSettings({ show_spectrum: !settings.show_spectrum })}
  />
  <span class="tg-label">显示频谱</span>
</label>

<!-- Slider 滑块 -->
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
```

## 工作流程

### 设置变更流程

1. 用户在设置页面修改某个选项
2. `onchange` 或 `oninput` 事件触发 `saveSettings()`
3. `saveSettings()` 调用后端 `save_settings` 命令
4. 后端执行以下操作：
   - 更新内存中的设置
   - 持久化到 JSON 配置文件
   - 应用相关副作用（如置顶、开机启动）
   - 广播 `settings-updated` 事件
5. 主窗口监听器收到事件
6. Svelte 响应式系统自动更新所有相关 UI

### 初始加载流程

1. 主窗口 `onMount` 执行
2. 调用 `invoke("get_settings")` 获取保存的设置
3. 用获取的完整对象覆盖 `appSettings`
4. UI 自动渲染最新设置

## 优势

1. **统一性**：所有设置变更通过单一事件 `settings-updated` 传播
2. **简洁性**：前端无需为每个设置编写单独的监听器
3. **可扩展性**：新增设置字段时，前后端自动同步，无需额外代码
4. **即时性**：用户修改设置后立即保存和同步，无需手动点击保存
5. **一致性**：后端保存后立即广播，确保所有客户端状态一致

## 事件列表

| 事件名 | 方向 | 用途 |
|--------|------|------|
| `settings-updated` | 后端 → 前端 | 完整设置对象更新 |
| `halftone-changed` | 后端 → 前端 | 网点效果变更（保留用于特殊处理） |
| `pixel-art-changed` | 后端 → 前端 | 像素风格变更 |
| `hd-cover-changed` | 后端 → 前端 | 高清封面变更 |
| `mv-playback-changed` | 后端 → 前端 | MV 播放变更 |
| `corner-radius-changed` | 后端 → 前端 | 圆角变更 |
| `lock-floating-window-changed` | 后端 → 前端 | 悬浮窗锁定变更 |
| `always-on-top-changed` | 后端 → 前端 | 置顶设置变更 |
| `settings-changed` | 后端 → 前端 | 单项设置变更（通用） |

## 配置文件

设置持久化到：`%APPDATA%\isle\settings.json`

```json
{
  "island_theme": "original",
  "auto_hide": true,
  "show_spectrum": true,
  "enable_animations": true,
  "window_opacity": 255,
  "always_on_top": true,
  "hardware_acceleration": true,
  "reduce_animations": false,
  "show_debug_info": false,
  "log_level": "Info",
  "monitor_index": 0,
  "player_weights": {
    "netease": 50,
    "spotify": 50,
    "bilibili": 50,
    "qqmusic": 50,
    "apple": 50,
    "generic": 10
  },
  "enable_mv_playback": true,
  "lock_floating_window": false,
  "enable_hd_cover": true,
  "enable_pixel_art": false,
  "enable_halftone": false,
  "auto_start": false,
  "hide_settings_button": false,
  "hide_monitor_selector": false,
  "hide_floating_window": false,
  "expanded_corner_radius": 16,
  "always_show_top_bar": true
}
```
