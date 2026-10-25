# Tauri 性能采样

本目录保留 Tauri/WebView 应用的前端及 Windows 进程采样记录。历史采样主要来自 Playwright/Chromium 页面，不等于完整 Windows 进程的 CPU、私有内存或工作集指标。

2026-10-24 新增完整 Windows 进程树诊断：见 [整体计划](../PERFORMANCE_PLAN_2026-10-24.md)、[测试报告](../PERFORMANCE_TEST_REPORT_2026-10-24.md) 和 [原始证据目录](audit-2026-10-24/)。该次诊断尚未达到 100 MB 目标；原始 JSON 明确区分原生进程树与浏览器夹具数据。

后续图片分配优化及可复现对照见 [第二轮报告](../PERFORMANCE_ROUND2_2026-10-24.md)；[数据目录](round2-2026-10-24/)中的请求堆分配量不代表 Windows 工作集。

- `before.json`、`after.json`：页面资源释放改动前后的场景记录。
- `retained-memory.json`：生产构建下重复切换页面后的保留量。
- `live-spectrum-2026-09-30.json`、`spectrum-1.0.8-2026-09-30.json`：频谱页面渲染采样。

采样条件、指标定义和限制见各文件对应的项目变更记录。Rust/Win32 原生 UI 的性能与回归证据已随代码迁移至[原生 UI 仓库](https://github.com/alent17/Wind0ws_Dynamic_island-native)。
