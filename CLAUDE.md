# CLAUDE.md

## 项目概述

Tauri 2 桌面应用。Rust 后端 + Svelte 5 前端，通过 Tauri IPC 通信。包管理器 pnpm，构建工具 Vite。
视觉与交互基调：Neo-Brutalism 与 Tech-wear 风格，追求极致的 120Hz 丝滑动画体验与物理弹簧反馈。

## 常用命令

```bash
# 开发
pnpm tauri dev

# 构建
pnpm tauri build

# 前端类型检查
pnpm check

# 前端 lint + 格式化
pnpm lint
pnpm format

# Rust 检查（在 src-tauri/ 目录下执行）
cargo check
cargo clippy -- -D warnings
cargo fmt --check
cargo test

# 全量检查（CI 等效）
pnpm check && cd src-tauri && cargo clippy -- -D warnings && cargo test
```

> **注意**：修改任何代码后，必须确保对应层级的检查命令通过。

## 项目结构

```text
src-tauri/
├── src/
│   ├── main.rs          # 入口：初始化 + plugin + window builder + handler 注册
│   ├── lib.rs           # 模块声明
│   ├── commands/        # Tauri command（薄层，只做转发）
│   ├── services/        # 业务逻辑
│   ├── models/          # 数据结构（Serialize + Deserialize）
│   ├── state/           # AppState（Arc<RwLock<T>>）
│   ├── error.rs         # AppError（thiserror + Serialize）
│   └── utils.rs
├── Cargo.toml
├── tauri.conf.json
└── capabilities/        # Tauri 权限声明

src/
├── lib/
│   ├── components/      # 通用 Svelte 组件
│   ├── stores/          # Svelte store
│   ├── api/             # Tauri invoke 封装（唯一允许调用 invoke 的地方）
│   ├── types/           # TypeScript 类型定义
│   └── utils/           # 工具函数
├── routes/              # 页面（SvelteKit 约定式路由，支持多窗口路由）
├── app.css              # 全局 CSS 变量
└── app.html
```

## 架构分层

Svelte 组件 → `$lib/api/*.ts` → Tauri invoke → Rust command → Rust service → Rust model

- **command 层**：接收参数、注入 State、调用 service、返回 Result。不写业务逻辑。
- **service 层**：所有实际业务逻辑。返回 `Result<T, AppError>`。
- **model 层**：纯数据结构，`#[derive(Serialize, Deserialize)]` + `#[serde(rename_all = "camelCase")]`。
- **api 层**：TypeScript 对 Rust command 的一一映射封装，统一错误处理。

## Rust 规范

### 命名

| 类型            | 风格                | 示例              |
| :------------ | :---------------- | :-------------- |
| 文件 / 模块       | snake\_case       | `file_ops.rs`   |
| 结构体 / 枚举      | PascalCase        | `FileEntry`     |
| 函数 / 方法       | snake\_case       | `list_files`    |
| 常量            | SCREAMING\_SNAKE  | `MAX_PATH_LEN`  |
| Tauri command | snake\_case, 动词开头 | `get_file_list` |

### Tauri Command

```rust
#[tauri::command]
async fn get_file_list(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<Vec<FileEntry>, AppError> {
    let svc = FileService::new(&state);
    svc.list_files(&path).await
}
```

- 返回类型固定 `Result<T, AppError>`
- State 注入标注生命周期 `State<'_, AppState>`
- 注册时按模块分组：

```rust
.invoke_handler(tauri::generate_handler![
    // file
    commands::file_ops::get_file_list,
    commands::file_ops::create_directory,
    // settings
    commands::settings::get_settings,
    commands::settings::update_settings,
])
```

### 错误处理

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{message}")]
    Business { code: u32, message: String },
}

impl serde::Serialize for AppError { /* 产出 { code, message } */ }
```

- 前端收到的错误结构固定 `{ code: number, message: string }`
- 禁止 `.unwrap()` / `.expect()` 出现在 command 和 service 层
- 使用 `?` 传播，最终统一为 `AppError`

### 数据模型

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
    pub modified_at: String,
}
```

- 跨 IPC 的结构体必须 `#[serde(rename_all = "camelCase")]`
- 有默认值的字段加 `#[serde(default)]`
- 枚举用 `#[serde(rename_all = "lowercase")]`

### 状态管理

```rust
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub db: Arc<Database>,
}
```

- 读多写少用 `RwLock`，写频繁用 `Mutex`
- 通过 `.manage(state)` 注册

### 日志

使用 `tracing` + `tracing-subscriber`。禁止 `println!` / `eprintln!`。

### Cargo 依赖

最小化依赖，不加不需要的 feature。Release profile：

```toml
[profile.release]
strip = true
lto = true
codegen-units = 1
opt-level = "s"
panic = "abort"
```

## Svelte / TypeScript 规范

### 命名

| 类型      | 风格                | 示例                |
| :------ | :---------------- | :---------------- |
| 组件文件    | PascalCase.svelte | `FileList.svelte` |
| TS 文件   | camelCase.ts      | `fileStore.ts`    |
| 接口      | PascalCase        | `FileEntry`       |
| 变量 / 函数 | camelCase         | `getFileList`     |
| 常量      | SCREAMING\_SNAKE  | `DEFAULT_THEME`   |
| CSS 类名  | kebab-case        | `file-list__item` |
| CSS 变量  | kebab-case        | `--color-primary` |

### Svelte 5 Runes（强制使用）

```svelte
<script lang="ts">
  let { path = '/' }: { path?: string } = $props();
  let files = $state<FileEntry[]>([]);
  let loading = $state(false);
  let sorted = $derived([...files].sort((a, b) => a.name.localeCompare(b.name)));
  $effect(() => { loadFiles(path); });
</script>
```

- 禁止 `export let`、`$:`、`createEventDispatcher`（Svelte 4 遗留语法）
- `{#each}` 必须带 key：`{#each items as item (item.id)}`
- 条件渲染顺序：loading → error → empty → content

### IPC 封装

```typescript
// $lib/api/index.ts
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

```typescript
// $lib/api/file.ts
import { cmd } from "$lib/api";
import type { FileEntry } from "$lib/types";

export const getFileList = (path: string) =>
  cmd<FileEntry[]>("get_file_list", { path });
```

- 禁止在 `.svelte` 中直接 `import { invoke }`
- 所有调用走 `$lib/api/*.ts`
- API 文件与 Rust commands 模块一一对应
- 函数必须有显式返回类型

### Store

```typescript
function createSettingsStore() {
  const { subscribe, set, update } = writable<AppSettings | null>(null);
  return {
    subscribe,
    async load() {
      set(await getSettings());
    },
    async patch(partial: Partial<AppSettings>) {
      /* ... */
    },
  };
}
export const settings = createSettingsStore();
```

- 工厂函数 `createXxxStore()` 模式
- 隐藏原始 `set`/`update`，通过语义化方法暴露

### 类型与样式

- 与 Rust model 对应的接口字段名必须一致（camelCase），统一从 `$lib/types/index.ts` 导出
- 颜色 / 间距 / 圆角全部使用 `app.css` 中的 CSS 变量，禁止硬编码
- 组件样式 scoped，全局样式只放 `app.css`
- BEM-like 命名：`block__element--modifier`

### 多窗口设计与 UI/UX 细节约束

- **控制层分离**：诸如“关闭按钮”等操作性交互，必须放置在独立的浮动窗口 (Floating Window) 中，**严禁**将其整合进 Dynamic Island 面板。
- **高保真动画优先**：重构或优化代码时，120Hz 动画的流畅度优先级高于代码精简。禁止提交会让动画变得跳跃、卡顿的布局或逻辑变更。优先使用 `svelte/motion` 的弹簧 (spring) 物理曲线和 CSS transform。
- **圆角规范**：UI 组件统一使用标准的圆角 (Standard Rounded Corner)，**严禁**在任何未明确说明的地方使用内凹（倒角/Inverted）圆角。
- **音频频谱可视化**：实现音频频谱组件时，必须保持 6 根柱子，上下对称，且严格放置在歌曲信息的右侧。

### 事件机制

- 事件名 kebab-case：`file-changed`、`settings-updated`
- 监听必须在 `onDestroy` 中调用 `unlisten()` 取消

| Rust command (snake\_case) | TS 函数 (camelCase)  | 事件 (kebab-case)    |
| :------------------------- | :----------------- | :----------------- |
| `get_file_list`            | `getFileList()`    | —                  |
| `update_settings`          | `updateSettings()` | `settings-updated` |

## 安全

- 文件系统访问必须配置 Tauri scope，禁止全局开放
- 用户输入路径必须做遍历检查（拒绝 `..`）
- 敏感数据仅在 Rust 端处理，不在前端持久化
- `tauri.conf.json` 权限遵循最小权限原则
- 修改 capabilities 必须在 commit message 中说明原因

## Git 规范

### 提交格式

`<type>(<scope>): <简短描述>`

| type     | 含义      | scope  | 含义        |
| :------- | :------ | :----- | :-------- |
| feat     | 新功能     | rust   | Rust 后端   |
| fix      | 修复      | ui     | Svelte 前端 |
| refactor | 重构      | ipc    | IPC 接口    |
| style    | 样式 / 格式 | build  | 构建配置      |
| docs     | 文档      | deps   | 依赖        |
| test     | 测试      | <br /> | <br />    |
| chore    | 构建 / 依赖 | <br /> | <br />    |
| perf     | 性能      | <br /> | <br />    |

_示例：`feat(rust): 添加文件监听 service`、`fix(ui): 修复深色主题滚动条不可见`_

### 分支管理

```text
main ← 发布
└── develop ← 集成
    ├── feat/xxx
    ├── fix/xxx
    └── refactor/xxx
```

## 测试与 Lint 配置

- **测试**：Rust 使用 `#[cfg(test)] mod tests {}`，文件系统测试用 `tempfile`；Svelte 使用 Vitest + Testing Library。命名约定：`test_<函数>_<场景>_<预期>`。核心 service 层覆盖率目标 > 80%。
- **Rust Lint**：

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
tab_spaces = 4

# Cargo.toml
[lints.clippy]
unwrap_used = "warn"
expect_used = "warn"
panic = "deny"
```

- **TS/Svelte Lint**：

```json
{
  "semi": true,
  "singleQuote": true,
  "trailingComma": "all",
  "printWidth": 100,
  "tabWidth": 2
}
```

## AI 开发约束

### 必须做

1. 修改 Rust 后确保 `cargo check` + `cargo clippy` 通过。
2. 修改 Svelte 后确保 `pnpm check` 通过。
3. 新增 Tauri command 时同步完成四步：
   - `commands/` 添加 command 函数
   - `main.rs` 注册到 `generate_handler!`
   - `$lib/api/` 添加封装函数
   - `$lib/types/` 更新类型
4. 修改数据模型时 Rust 和 TypeScript 同步修改，字段名保持一致。
5. 保持最小改动，只改与任务相关的代码。

### 禁止做

1. 在 `.svelte` 中直接 `import { invoke }`。
2. 在 command / service 层使用 `.unwrap()` / `.expect()`。
3. 使用 Svelte 4 语法（`export let`、`$:`、`createEventDispatcher`）。
4. 硬编码颜色 / 间距 / 圆角。
5. 擅自使用内凹/倒角圆角。
6. 将控制类按钮（如关闭）塞入 Dynamic Island 中。
7. 为了精简代码而牺牲 UI 动画渲染性能（导致卡顿/掉帧）。
8. 未说明理由就修改 `tauri.conf.json` 权限或 `Cargo.toml` 依赖。
9. 做超出任务范围的架构变更。
10. 使用 `println!` 代替 `tracing`。

### 不确定时

- 先提出方案和取舍理由，确认后再实现。
- IPC 接口变更时同时列出 Rust 和 TypeScript 两侧改动。
- 权限变更时说明安全影响。

