# Isle Tauri 版安装与构建

本仓库发布基于 Tauri 2、Svelte 和 WebView2 的 Windows 桌面应用。Rust/Win32 原生 UI 使用独立仓库：[Wind0ws_Dynamic_island-native](https://github.com/alent17/Wind0ws_Dynamic_island-native)。

## 安装

从 [GitHub Releases](https://github.com/alent17/Wind0ws_Dynamic_island/releases) 下载最新 Windows 安装包并运行。Tauri 版使用 WebView2；Windows 10/11 通常已提供运行时，若系统提示缺少 WebView2，请按安装提示安装 Microsoft WebView2 Runtime。

## 从源码开发

需要 Node.js、pnpm（或 npm）、Rust MSVC、Visual Studio C++ Build Tools 和 Windows SDK。

```powershell
pnpm install
pnpm tauri dev
```

## 构建

```powershell
pnpm check
pnpm test
pnpm build
pnpm bundle:windows
```

Playwright UI 测试使用 `pnpm test:ui`。构建产物位于 `src-tauri/target/release/bundle/`。
