# 项目架构重构指南

根据 CLAUDE.md 规范，项目需要进行以下架构重构。

## 当前状态

- **后端**：所有代码集中在 `lib.rs` 中（38 个 commands）
- **前端**：缺少 `$lib/` 目录结构，直接调用 `invoke`

## 目标架构

### 后端结构

```
src-tauri/src/
├── main.rs          # 入口：初始化 + plugin + window builder + handler 注册
├── lib.rs           # 模块声明
├── error.rs         # AppError（thiserror + Serialize）
├── commands/        # Tauri command（薄层，只做转发）
│   ├── mod.rs
│   ├── settings.rs  # 设置相关 commands
│   ├── media.rs     # 媒体相关 commands
│   ├── cache.rs     # 缓存相关 commands
│   ├── monitor.rs   # 显示器相关 commands
│   └── window.rs    # 窗口相关 commands
├── services/        # 业务逻辑
│   ├── mod.rs
│   ├── settings.rs  # 设置读写、开机启动
│   ├── media.rs     # GSMTC、网易云 API
│   ├── cache.rs     # 缓存管理
│   ├── spectrum.rs  # 音频频谱处理
│   └── color.rs     # 颜色提取
├── models/          # 数据结构
│   ├── mod.rs
│   ├── settings.rs  # AppSettings, CacheStats
│   ├── media.rs     # MediaState, SpectrumData, NeteaseSong
│   └── cache.rs     # CacheMetadata
└── state/           # AppState
    ├── mod.rs
    └── app_state.rs # AppState, MediaCache
```

### 前端结构

```
src/lib/
├── api/             # Tauri invoke 封装（唯一允许调用 invoke 的地方）
│   ├── index.ts     # cmd() 函数和 AppError 类
│   ├── settings.ts  # 设置 API
│   ├── media.ts     # 媒体 API
│   └── cache.ts     # 缓存 API
├── components/      # 通用 Svelte 组件
├── stores/          # Svelte store
├── types/           # TypeScript 类型定义
└── utils/           # 工具函数
```

## 迁移步骤

### Phase 1: 后端基础架构

1. ✅ 创建 `error.rs` - 统一错误处理
2. ✅ 创建 `models/` 目录 - 数据结构
3. ✅ 创建 `state/` 目录 - 状态管理
4. ✅ 创建 `services/` 目录 - 业务逻辑
5. ✅ 创建 `commands/` 目录 - Tauri commands

### Phase 2: 迁移 Commands

从 `lib.rs` 迁移 commands 到 `commands/` 目录：

```rust
// commands/settings.rs
use tauri::{AppHandle, State};
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;
use crate::state::AppState;
use crate::services;

#[tauri::command]
pub fn get_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<AppSettings> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    Ok(settings.clone())
}
```

### Phase 3: 迁移 Services

从 `lib.rs` 迁移业务逻辑到 `services/` 目录：

```rust
// services/settings.rs
use std::fs;
use tauri::{AppHandle, Manager};
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;

pub fn read_settings_file(app: &AppHandle) -> Option<AppSettings> {
    let config_dir = app.path().app_data_dir().ok()?;
    let config_path = config_dir.join("settings.json");
    let content = fs::read_to_string(config_path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_settings_file(app: &AppHandle, settings: &AppSettings) -> AppResult<()> {
    // ...
}
```

### Phase 4: 前端重构

1. 创建 `src/lib/api/index.ts`：

```typescript
import { invoke } from "@tauri-apps/api/core";

export class AppError extends Error {
  constructor(
    public readonly code: number,
    message: string,
  ) {
    super(message);
  }
}

export async function cmd<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    const e = err as { code: number; message: string };
    throw new AppError(e.code, e.message);
  }
}
```

2. 创建 `src/lib/api/settings.ts`：

```typescript
import { cmd } from "$lib/api";
import type { AppSettings } from "$lib/types";

export const getSettings = () => cmd<AppSettings>("get_settings");
export const saveSettings = (settings: AppSettings) => 
  cmd<void>("save_settings", { settings });
```

3. 更新 Svelte 组件：

```svelte
// 替换前
import { invoke } from "@tauri-apps/api/core";
const settings = await invoke<AppSettings>("get_settings");

// 替换后
import { getSettings } from "$lib/api/settings";
const settings = await getSettings();
```

## 已创建的架构文件

以下文件已创建，包含基础结构：

- `src-tauri/src/error.rs` - 统一错误处理
- `src-tauri/src/models/mod.rs` - 模型模块
- `src-tauri/src/models/settings.rs` - 设置模型
- `src-tauri/src/models/media.rs` - 媒体模型
- `src-tauri/src/models/cache.rs` - 缓存模型
- `src-tauri/src/state/mod.rs` - 状态模块
- `src-tauri/src/state/app_state.rs` - 应用状态
- `src-tauri/src/services/mod.rs` - 服务模块
- `src-tauri/src/services/settings.rs` - 设置服务
- `src-tauri/src/services/media.rs` - 媒体服务
- `src-tauri/src/services/cache.rs` - 缓存服务
- `src-tauri/src/services/spectrum.rs` - 频谱服务
- `src-tauri/src/services/color.rs` - 颜色服务
- `src-tauri/src/commands/mod.rs` - 命令模块
- `src-tauri/src/commands/settings.rs` - 设置命令
- `src-tauri/src/commands/media.rs` - 媒体命令
- `src-tauri/src/commands/cache.rs` - 缓存命令
- `src-tauri/src/commands/monitor.rs` - 显示器命令
- `src-tauri/src/commands/window.rs` - 窗口命令

## 下一步

1. 逐步将 `lib.rs` 中的代码迁移到新模块
2. 每迁移一个模块后运行 `cargo check` 验证
3. 更新 `main.rs` 中的 `generate_handler!` 注册
4. 创建前端 `$lib/api` 层

## 注意事项

- 保持最小改动，每次只迁移一个功能
- 迁移后确保 `cargo check` 和 `pnpm check` 都通过
- 遵循 CLAUDE.md 中的命名规范
- 禁止使用 `.unwrap()` 和 `.expect()`
