# ️ Windows Dynamic Island - 灵动岛音乐播放器

一个受 macOS Dynamic Island 启发的 Windows 桌面音乐播放器，使用 Tauri + Svelte 5 构建。

![Windows Dynamic Island](https://img.shields.io/badge/Platform-Windows-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)
![Svelte](https://img.shields.io/badge/Svelte-5-ff3e00)
![Rust](https://img.shields.io/badge/Rust-latest-dea584)

## ✨ 特性

### 🎨 多主题支持

- **经典黑** - 原始黑色主题
- **液体玻璃** - 毛玻璃效果主题
- **复古卡通** - 60 年代波普艺术风格主题

### 🎵 音乐播放控制

- 支持所有主流音乐平台（网易云音乐、QQ 音乐、Spotify 等）
- 系统级媒体控制集成（SMTC）
- 实时进度同步
- 专辑封面显示
- MV 播放支持

### 🎯 灵动岛交互

- 智能收起/展开动画
- 鼠标悬停自动展开
- 顶部边缘手势唤醒
- 流畅的弹簧物理动画

### 📊 音频可视化

- 实时频谱分析
- 可调节的频谱条数量
- 多种频谱样式

### 🖼️ 悬浮窗模式

- 独立悬浮窗口
- 进度条显示
- 迷你播放控制

## 🚀 技术栈

- **前端**: Svelte 5 (Runes) + Vite
- **后端**: Rust + Tauri 2.0
- **媒体控制**: Windows SMTC API
- **网络请求**: reqwest (Rust)
- **样式**: Tailwind CSS

## 📦 安装与运行

### 环境要求

- Node.js 18+
- Rust 1.70+
- Windows 10/11

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建发布版

```bash
npm run tauri build
```

## 🎮 使用说明

### 基本操作

- **鼠标悬停顶部边缘** - 唤醒灵动岛
- **点击灵动岛** - 展开/收起
- **拖动灵动岛** - 调整位置（部分模式）
- **滚动** - 调节音量

### 主题切换

1. 点击设置按钮
2. 选择喜欢的主题
3. 自动保存偏好设置

### 播放器功能

- 自动检测系统媒体播放
- 支持网易云音乐时长获取
- 专辑封面高清获取
- MV 链接自动获取与播放

## 🛠️ 开发

### 项目结构

```
Wind0ws_Dynamic_island/
├── src/                    # 前端 Svelte 代码
│   ├── App.svelte         # 主界面组件
│   ├── FloatingWindow.svelte  # 悬浮窗组件
│   ├── Settings.svelte    # 设置界面
│   └── stores/            # 状态管理
├── src-tauri/             # 后端 Rust 代码
│   ├── src/
│   │   └── lib.rs        # 主要逻辑
│   └── Cargo.toml        # Rust 依赖
└── package.json          # 项目配置
```

<br />

## 📝 更新日志

### v1.0.0

- ✨ 添加复古卡通主题
- 🎨 优化专辑封面显示（52x52）
- 🐛 修复边框圆角渲染问题
- ⚡ 优化进度同步逻辑
- 🎯 改进悬浮窗进度条

<br />

## 🙏 致谢

- [Tauri](https://tauri.app/)
- [Svelte](https://svelte.dev/)

***

Made with ❤️ by Windows Dynamic Island Team
