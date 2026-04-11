# 项目架构文档

## 📁 目录结构

### 前端结构 (src/)

```
src/
├── components/          # 可复用组件
│   ├── common/         # 通用组件
│   │   ├── Button.svelte    # Spotify 风格按钮
│   │   ├── Card.svelte      # 卡片容器
│   │   ├── Toggle.svelte    # 开关切换
│   │   └── Input.svelte     # 输入框
│   ├── layout/         # 布局组件（预留）
│   ├── player/         # 播放器组件（预留）
│   └── settings/       # 设置组件（预留）
├── styles/             # 样式系统
│   ├── variables.css   # CSS 变量定义（Spotify 设计系统）
│   ├── base.css        # 基础样式重置
│   └── components.css  # 组件样式
├── stores/             # Svelte 状态管理
│   ├── player.js       # 播放器状态
│   ├── settings.js     # 设置状态
│   ├── theme.js        # 主题状态
│   └── index.js        # 统一导出
├── hooks/              # 自定义 hooks（预留）
├── utils/              # 工具函数
│   └── tauri.js        # Tauri API 封装
├── routes/             # 路由页面
│   ├── +layout.svelte
│   ├── +page.svelte
│   └── settings/
│       └── +page.svelte
├── assets/             # 静态资源
│   ├── fonts/
│   └── icons/
├── App.svelte          # 主应用组件（灵动岛）
├── FloatingWindow.svelte # 悬浮窗组件
├── Settings.svelte     # 设置页面（旧版，保留兼容）
└── main.ts             # 应用入口
```

### 后端结构 (src-tauri/)

```
src-tauri/
├── src/
│   ├── main.rs         # Rust 应用入口
│   ├── lib.rs          # Tauri 命令定义
│   ├── commands/       # 命令模块（预留）
│   ├── models/         # 数据模型（预留）
│   └── utils/          # 工具函数（预留）
├── Cargo.toml          # Rust 依赖配置
└── tauri.conf.json     # Tauri 配置
```

## 🎨 Spotify 设计系统

### 颜色系统

```css
/* Base Colors */
--base-dark: #121212;        /* 主背景 */
--base-dark-gray: #181818;   /* 卡片背景 */
--base-mid-gray: #1f1f1f;    /* 按钮背景 */

/* Text Colors */
--text-base: #ffffff;        /* 主文本 */
--text-secondary: #b3b3b3;   /* 次要文本 */
--text-subdued: #cbcbcb;     /* 柔和文本 */

/* Accent Colors */
--accent-green: #1ed760;     /* Spotify Green */

/* Semantic Colors */
--text-negative: #f3727f;    /* 错误 */
--text-warning: #ffa42b;     /* 警告 */
--text-announcement: #539df5; /* 信息 */
```

### 组件规范

#### Button 组件
- **Pill 形状**: `border-radius: 9999px`
- **Circular 形状**: `border-radius: 50%`
- **Uppercase 标签**: `text-transform: uppercase; letter-spacing: 1.4px`
- **变体**: primary, secondary, outlined, danger, circular

#### Card 组件
- **背景**: `#181818`
- **圆角**: `8px`
- **阴影**: `rgba(0,0,0,0.3) 0px 8px 8px`

#### Toggle 组件
- **Pill 形状**: `border-radius: 9999px`
- **激活色**: `#1ed760`
- **内阴影边框**: `inset-border`

### 间距系统

基于 8px 基础单位：
- `--spacing-xs: 4px`
- `--spacing-sm: 8px`
- `--spacing-md: 12px`
- `--spacing-lg: 16px`
- `--spacing-xl: 20px`
- `--spacing-xxl: 24px`

### 字体系统

```css
--font-family-title: 'SpotifyMixUITitle', ...;
--font-family-ui: 'SpotifyMixUI', ...;

/* 字号 */
--text-xs: 10px;   /* Micro */
--text-sm: 12px;   /* Small */
--text-md: 14px;   /* Caption/Button */
--text-lg: 16px;   /* Body */
--text-xl: 18px;   /* Feature Heading */
--text-2xl: 24px;  /* Section Title */
```

## 🔧 使用方法

### 组件导入

```svelte
<script>
  import { Button, Card, Toggle, Input } from '$lib/components';
</script>

<Button variant="primary" size="md">
  播放
</Button>

<Card variant="default" padding="md">
  <p>卡片内容</p>
</Card>

<Toggle checked={true} />

<Input type="text" placeholder="搜索..." variant="pill" />
```

### 状态管理

```svelte
<script>
  import { playerStore, playerActions } from '$lib/stores';
  import { settingsStore, settingsActions } from '$lib/stores';
</script>

{$playerStore.isPlaying ? '播放中' : '已暂停'}

<button on:click={playerActions.toggle}>
  切换播放
</button>
```

### Tauri API

```javascript
import { windowCommands, settingsCommands } from '$lib/utils/tauri';

// 切换窗口显示
await windowCommands.toggleFloatingWindow();

// 设置开机启动
await settingsCommands.setAutoStart(true);
```

## 📝 开发规范

### 组件开发
1. 所有组件必须使用 Spotify 设计系统的 CSS 变量
2. 组件必须支持无障碍访问（ARIA）
3. 使用 Svelte 5 的 runes 语法（`$state`, `$derived`）
4. 导出清晰的 props 接口

### 样式规范
1. 禁止使用硬编码的颜色值，必须使用 CSS 变量
2. 间距必须使用 spacing 系统
3. 圆角必须使用 radius 系统
4. 过渡动画使用 transition 变量

### 命名规范
- 组件：PascalCase（如 `Button.svelte`）
- 样式：kebab-case（如 `.btn-primary`）
- 变量：camelCase（如 `isPlaying`）
- CSS 变量：kebab-case（如 `--accent-green`）

## 🚀 构建命令

```bash
# 开发模式
npm run tauri dev

# 构建生产版本
npm run tauri build

# 仅构建前端
npm run build

# 代码检查
npm run lint
```

## 📄 相关文件

- [design.md](./design.md) - Spotify 设计规范完整文档
- [.plan.md](./.plan.md) - 项目开发计划
- [README.md](./README.md) - 项目说明

## 🔄 迁移指南

### 从旧版迁移

旧版 Settings.svelte 仍然保留，但建议使用新的组件化版本：

```svelte
<!-- 旧版 -->
<Settings />

<!-- 新版 -->
<SettingsPage />
```

所有样式已自动迁移到 Spotify 设计系统，无需手动调整。

## 🎯 待完成工作

- [ ] 拆分 App.svelte 为可复用组件
- [ ] 拆分 FloatingWindow.svelte 为可复用组件
- [ ] 实现 Player 组件系列
- [ ] 完善 Layout 组件
- [ ] 添加自定义 hooks
- [ ] 后端 Rust 代码模块化
- [ ] 添加单元测试

---

*最后更新：2026-04-10*
