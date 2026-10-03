# Tauri 性能采样

本目录只保留 Tauri/WebView 应用的前端采样记录。采样主要来自 Playwright/Chromium 页面，不等于完整 Windows 进程的 CPU、私有内存或工作集指标。

- `before.json`、`after.json`：页面资源释放改动前后的场景记录。
- `retained-memory.json`：生产构建下重复切换页面后的保留量。
- `live-spectrum-2026-09-30.json`、`spectrum-1.0.8-2026-09-30.json`：频谱页面渲染采样。

采样条件、指标定义和限制见各文件对应的项目变更记录。Rust/Win32 原生 UI 的性能与回归证据已随代码迁移至[原生 UI 仓库](https://github.com/alent17/Wind0ws_Dynamic_island-native)。
