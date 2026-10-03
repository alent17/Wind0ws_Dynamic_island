# Isle — Windows 灵动岛（Tauri）

这个仓库维护基于 **Svelte、Tauri 2 和 WebView2** 的 Windows 桌面应用。独立 Rust/Win32 原生 UI 已迁移到另一个仓库：[Wind0ws_Dynamic_island-native](https://github.com/alent17/Wind0ws_Dynamic_island-native)。

## 开发

需要 Node.js、pnpm（或 npm）、Rust MSVC、Visual Studio C++ Build Tools 和 Windows SDK。

```powershell
pnpm install
pnpm dev
```

常用命令：

```powershell
pnpm check
pnpm test
pnpm test:ui
pnpm build
pnpm bundle:windows
```

`pnpm tauri dev` 可直接启动 Tauri 开发模式。Windows 安装包由 `pnpm bundle:windows` 生成。

## 项目结构

- `src/`：Svelte 界面
- `src-tauri/`：Tauri/Rust 后端与应用配置
- `ui-tests/`：Playwright UI 测试与迁移参考截图
- `docs/`：项目文档、发布记录和 Tauri 性能采样

安装和升级说明见 [INSTALL.md](INSTALL.md)。原生 UI 的源码和文档见[原生 UI 仓库](https://github.com/alent17/Wind0ws_Dynamic_island-native)。
