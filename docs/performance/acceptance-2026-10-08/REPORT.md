# 2026-10-08 Release 基线及 M1/M2 验收

三轮 Release Idle 数据已取得，M1 的本轮资源/查询验证通过；M2 未通过，浮窗重开出现空封面。仅修正测量工具并新增验收脚本，没有修改业务代码，没有实施 M3、动画、UI 或架构阶段。

## 构建与条件

- 源提交：`7d75c9784d03fd658260cc68604b7e52cfafccde`，Isle 1.0.11。
- 新构建：`npm run build`、`cargo build --release --manifest-path src-tauri/Cargo.toml --features tauri/custom-protocol`，Release 构建成功；不使用旧 EXE。
- EXE SHA-256：`FC697AA1933F1095466A18CBCB1B177F8B0C30376ADFC12D1F5665839CB3213F`。
- 正式 Idle 使用同一个 Release 进程，三轮各预热 300 秒再采样 600 秒。项目 tauri dev/Vite/debug Isle 已关闭；未附加 DevTools，不运行构建任务。
- 网易云保持 Paused，前后均为同一首曲目。Compact 主窗口，浮窗关闭，Spectrum 设置为 realtime/开启但无播放；Capture 四项全关，有天气城市。保留原有用户设置。
- 两块 1920×1080 显示器，scaleFactor=1；监视器索引 1，工作区 1920×1032。详细记录在 [monitors-and-scene.json](../../../dist/performance/acceptance-2026-10-08/monitors-and-scene.json)。
- 堆/切歌验收为另一轮同一 Release 的 WebView2 远程诊断。该轮的 CPU/内存不可混入无 DevTools 的 Idle 数据。

## B0：有效三轮

CPU 为 12 个逻辑核的整机归一化进程组利用率，表中平均值是样本均值，累计 CPU 秒数复核分别约 0.0753%、0.0743%、0.0705%。每个 PID 与其 WebView2 后代分开记录。

| 轮次 | 实际采样时长 | 样本数 | 平均 CPU | CPU p95 | Private commit MiB 首→尾 | 线程首→尾 | 句柄首→尾 |
|---|---|---|---|---|---|---|---|
| release-idle-r1 | 600.80s | 407 | 0.0754% | 0.3541% | 256.29 → 256.61 | 213 → 211 | 3789 → 3787 |
| release-idle-r2 | 600.21s | 396 | 0.0743% | 0.2704% | 256.93 → 258.09 | 220 → 212 | 3800 → 3794 |
| release-idle-r3 | 600.87s | 395 | 0.0704% | 0.2628% | 259.69 → 260.46 | 211 → 210 | 3795 → 3793 |

三轮 CPU 均低于原计划候选平均 0.1%、p95 0.5% 预算。GPU 专用内存约 16.3 MiB、单轮范围约 16.08–16.39 MiB。GPU engine 计数器读数为 0，但该计数器使用整数百分比，因此不能据此声称精确 0 GPU 或没有呈现帧。

整个有效窗口的 private commit 约从 256.29 到 260.46 MiB，缓慢增加约 4.17 MiB。线程、句柄、GPU 内存有界；本轮不能证明 8 小时无泄漏。没有桌面空载控制轮，候选预算未进一步做机器噪声校准；IPC bytes/in-flight、FFT 精确计数、ETW 实际帧时仍为未采集。B0 **本次用户要求的三轮 OS 基线完成**，原计划更广的 profiling 条目尚未齐备，故完整 B0 任务暂不勾选。

数据：[r1](../../../dist/performance/acceptance-2026-10-08/release-idle-r1.json)、[r2](../../../dist/performance/acceptance-2026-10-08/release-idle-r2.json)、[r3](../../../dist/performance/acceptance-2026-10-08/release-idle-r3.json)、[汇总](summary.json)。

### CPU 计数器缺陷与无效轮

首次三轮发现所有 CPU 被记录为 0。根因是 Windows PowerShell 的 `[math]::Max(0, delta)` 选中了整数重载，15.625 ms 等小数增量被取整。已改为显式 double，并使用原始 `TotalProcessorTime.TotalSeconds` 与读取时间戳。

`powershell -NoProfile -File scripts/test-cpu-counter.ps1` 验证小数增量、12 核归一化及重置钳位通过；短时真实进程复核取得非零 CPU。随后完整重跑三轮，本报告只使用重跑数据。旧轮移至 `dist/performance/acceptance-2026-10-08/invalid-cpu-rounds/`，JSON 标明 `cpuDataValid=false`，不能用于 CPU 验收；其他字段仍可作诊断记录。旧 debug/旧脚本的 CPU 数字也须视为未经此修复验证，不能再引用其“接近 0”结论。

## M1：本轮验收通过

在真实 Release 主窗口的原生命令入口连续调用 1000 次 `list_audio_output_devices`，0 次错误，每 250 次保存进程组记录。此前已覆盖 PROPVARIANT 转换失败路径及 apartment 嵌套/已有 STA 的 Windows 单测。

| 查询数 | 原生 app Private commit MiB | app 句柄 | app 线程 |
|---|---|---|---|
| 前 | 32.750 | 777 | 45 |
| 250 | 32.668 | 770 | 43 |
| 500 | 32.715 | 775 | 44 |
| 750 | 32.668 | 772 | 43 |
| 1000 | 32.668 | 772 | 43 |

未见随查询次数增长的原生 commit、句柄或线程。该结论是资源归属/错误路径/同路径压力验证，不能用来量化修复前实际泄漏字节，也不替代 UMDH/ETW 原生分配归因或 8 小时验收。按纯正确性修复口径勾选 M1。

## M2：未通过

当前网易云播放列表完成 **100 次真实切歌、100 首不同曲目**。全部切歌结果的当前封面像素与当前图片 URL 一致；0/25/50/75/100 的 WebView2 堆快照均报告 Detached Canvas=0。窗口在每 25 首后关闭重开，因此这些 heap 点跨了不同浮窗上下文，不能当作单个连续 WebView 100 首的 retained-heap 斜率。

**失败现场：第 50 首后重开，曲名 Veranda、歌手 The Tron 正确，但无封面 Canvas。** 等待 15 秒仍未恢复；随后的诊断读取原生 `get_media_info_cmd`，同曲目的 `albumArt` 长度为 5510，DOM 中仍为占位 `<div>`，canvasCount=0。截图见 [reopen-50-failure.png](../../../dist/performance/acceptance-2026-10-08/artwork/reopen-50-failure.png)。失败被保留，随后从第 51 首继续，完成了全部 100 次；没有把失败当作通过重试掩盖。

从代码看，[FloatingWindow.svelte:747](../../../src/FloatingWindow.svelte) 只有曲目变化分支写入封面；同曲目分支没有吸收迟到的完整封面。监听注册先于初始全量查询（约 835–843 行）。**可能原因**是先到的无封面 delta 确立了曲目，随后全量快照被归入 same-track 分支而忽略封面。事件时序未做完整 trace，不能把该推断写成已证实根因。

真正退出网易云后，原生媒体源为空，浮窗 canvasCount=0/imageCount=0；重启网易云后，封面 Canvas 恢复为 1。无会话及恢复 post-GC 快照的 Detached Canvas 也为 0。

浮窗关闭、播放器暂停后静置采样 300.29 秒，193 个样本：进程组 private commit 296.52 → 280.55 MiB，末 30 个样本平均约 279.46 MiB；句柄 4010 → 3924，线程 236 → 214。只剩主窗口 Renderer，private commit 约 88.15 MiB，GPU 专用内存回到 16.01 MiB。回收趋势成立，但 Renderer/进程组未严格回到最初启动占用，不能将缓存填充后的平台误写为零增长或长期无泄漏。

**M2 保持未勾选：重开空白违反显示正确性门槛。** 后续需补同曲目封面的初始化/迟到快照处理并重验；本轮未修改该业务路径。

## 证据与脚本

原始数据和 `.heapsnapshot` 全部保存在项目的 `dist/performance/acceptance-2026-10-08/`（忽略目录，未推送）。

- [切歌记录](../../../dist/performance/acceptance-2026-10-08/artwork/artwork-acceptance.json)：100 次像素检查、失败、5 个堆点及截图。
- [资源与无会话记录](../../../dist/performance/acceptance-2026-10-08/recovery/recovery.json)：1000 次设备查询、真实播放器退出/恢复。
- [5 分钟回稳](../../../dist/performance/acceptance-2026-10-08/recovery/after-artwork-5m-recovery.json)。
- `scripts/run-release-idle-baseline.ps1`：固定根 PID、三轮串行并核验构建状态。
- `scripts/media-session.ps1`：Windows PowerShell WinRT 会话读写。
- `scripts/accept-native-artwork.mjs`：真实 Release WebView2 切歌/像素/heap 检查，支持失败后保留证据继续采满语料。
- `scripts/native-resource-recovery.mjs`：设备命令压力与资源记录。
- `scripts/summarize-native-baseline.py`、`scripts/test-cpu-counter.ps1`：汇总及计数器回归。

结束后关闭诊断进程，恢复普通 Release 和网易云播放，浮窗关闭；不遗留调试会话、构建或采样任务。验收结束时未提交/推送新测量脚本与报告；2026-10-08 项目所有者随后要求上传，测量工具与汇总报告独立提交。原始大文件保留在本地 dist 目录。
