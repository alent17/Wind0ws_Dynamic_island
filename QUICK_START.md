# 快速开始指南

## 🚀 开发启动

### 前置要求
- Node.js 18+ 
- Rust 1.70+
- Tauri CLI

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
# 启动开发服务器（自动打开应用）
npm run tauri dev
```

### 构建生产版本

```bash
# 构建完整应用
npm run tauri build

# 仅构建前端
npm run build
```

## 📁 项目结构

```
Wind0ws_Dynamic_island/
├── src/                      # 前端源码
│   ├── components/           # 可复用组件
│   │   └── common/          # 通用组件
│   ├── stores/              # 状态管理
│   ├── styles/              # Spotify 设计系统样式
│   ├── utils/               # 工具函数
│   ├── routes/              # 路由页面
│   ├── App.svelte           # 灵动岛主组件
│   ├── FloatingWindow.svelte # 悬浮窗组件
│   └── Settings.svelte      # 设置页面
├── src-tauri/               # 后端源码
│   └── src/
│       ├── main.rs          # Rust 入口
│       └── lib.rs           # Tauri 命令
├── design.md                # Spotify 设计规范
├── ARCHITECTURE.md          # 架构文档
└── REFACTOR_SUMMARY.md      # 重构总结
```

## 🎨 Spotify 设计系统

本项目已完全实现 Spotify 设计系统，所有组件和样式都遵循 [design.md](./design.md) 规范。

### 核心特性

- **沉浸式暗色主题** - `#121212`, `#181818`, `#1f1f1f`
- **Spotify Green 强调** - `#1ed760`
- **Pill/Circle 几何** - 圆角按钮和圆形控件
- **重型阴影** - 深度和层次感
- **8px 间距系统** - 统一的布局节奏
- **SpotifyMixUI 字体** - 紧凑的字号系统

### 使用组件

```svelte
<script>
  import { Button, Card, Toggle, Input } from './components';
</script>

<!-- Pill 形状的主按钮 -->
<Button variant="primary" size="md">
  播放
</Button>

<!-- 深色卡片 -->
<Card variant="default" padding="md">
  <p>卡片内容</p>
</Card>

<!-- Pill 开关 -->
<Toggle checked={true} />

<!-- Pill 输入框 -->
<Input type="text" placeholder="搜索..." variant="pill" />
```

### 使用 CSS 变量

```css
.container {
  background: var(--base-dark);
  padding: var(--spacing-lg);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-medium);
}

.title {
  font-family: var(--font-family-title);
  font-size: var(--text-2xl);
  font-weight: var(--font-bold);
  color: var(--text-base);
}
```

## 🛠️ 状态管理

### Player Store

```svelte
<script>
  import { playerStore, playerActions } from './stores/player';
</script>

{$playerStore.isPlaying ? '播放中' : '已暂停'}

<button on:click={playerActions.toggle}>
  切换播放
</button>
```

### Settings Store

```svelte
<script>
  import { settingsStore, settingsActions } from './stores/settings';
</script>

{#if $settingsStore.auto_start}
  <p>开机启动已启用</p>
{/if}

<button on:click={() => settingsActions.setAutoStart(true)}>
  启用开机启动
</button>
```

## 🔧 Tauri API

### 窗口管理

```javascript
import { windowCommands } from './utils/tauri';

// 切换悬浮窗
await windowCommands.toggleFloatingWindow();

// 设置点击穿透
await windowCommands.setClickThrough(true);

// 设置始终置顶
await windowCommands.setAlwaysOnTop(true);
```

### 设置管理

```javascript
import { settingsCommands } from './utils/tauri';

// 设置开机启动
await settingsCommands.setAutoStart(true);

// 清除缓存
await settingsCommands.clearCache();
```

## 📝 开发规范

### 组件开发

1. **必须使用 CSS 变量** - 禁止硬编码颜色值
2. **支持无障碍访问** - 添加 ARIA 标签
3. **使用 Svelte 5 runes** - `$state`, `$derived`
4. **导出清晰接口** - 使用 TypeScript 或 JSDoc

### 样式规范

```css
/* ✅ 正确 - 使用 CSS 变量 */
.button {
  background: var(--accent-green);
  color: var(--text-base);
  padding: var(--spacing-sm) var(--spacing-lg);
  border-radius: var(--radius-pill);
}

/* ❌ 错误 - 硬编码值 */
.button {
  background: #1ed760;
  color: white;
  padding: 8px 16px;
  border-radius: 9999px;
}
```

### 命名规范

- **组件**: PascalCase - `Button.svelte`
- **样式**: kebab-case - `.btn-primary`
- **变量**: camelCase - `isPlaying`
- **CSS 变量**: kebab-case - `--accent-green`

## 🧪 测试

### 功能测试

1. **设置页面**
   - [ ] 主题切换正常
   - [ ] 开关功能正常
   - [ ] 拖拽排序正常
   - [ ] 清除缓存正常

2. **灵动岛**
   - [ ] 展开/收起动画流畅
   - [ ] 时间显示正常
   - [ ] 频谱动画正常
   - [ ] 播放器控制正常

3. **悬浮窗**
   - [ ] 专辑封面显示正常
   - [ ] 播放控制正常
   - [ ] 进度条更新正常
   - [ ] MV 播放正常

### 性能测试

```bash
# 构建分析
npm run build -- --stats

# 使用 Chrome DevTools 分析性能
```

## 📚 参考文档

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 完整架构说明
- [design.md](./design.md) - Spotify 设计规范
- [REFACTOR_SUMMARY.md](./REFACTOR_SUMMARY.md) - 重构总结
- [Svelte 5 文档](https://svelte.dev/docs/svelte/v5-migration-guide)
- [Tauri 文档](https://tauri.app/v1/guides/)

## 🔧 常见问题

### 构建失败

```bash
# 清理缓存
npm run clean
npm install

# 重新构建
npm run tauri build
```

### Rust 编译错误

```bash
# 更新 Rust
rustup update

# 清理 Cargo 缓存
cargo clean
```

### 样式不生效

检查是否正确导入 CSS 变量：

```svelte
<style>
  @import './styles/variables.css';
</style>
```

## 🎯 下一步

1. **推送代码到 GitHub**
   ```bash
   git push origin master
   ```

2. **继续组件拆分**
   - App.svelte → 可复用子组件
   - FloatingWindow.svelte → 播放器组件

3. **性能优化**
   - 组件懒加载
   - CSS 代码分割
   - 动画优化

---

*最后更新：2026-04-10*
