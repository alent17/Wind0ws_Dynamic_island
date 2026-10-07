# 2026-10-07 B0/B1、M1、M2 执行记录

本轮完成了验证入口修复及两项正确性修复。Windows 短时数据只证明采样工具可以工作，不能证明 Idle、内存或 FPS 达标。首次验证结束时未创建提交、PR 或推送。2026-10-08 用户要求上传到 GitHub，发布范围包括本轮修复与通过回归的现有动画/夹具变更，不包含原本缺失的安装包、静态资源或编辑器配置的删除。

## 范围与基线

用户批准执行推荐前三项。审查提交为 `a9a855ef62d2596775115c70b82f63af8a67229e`。原目录没有 `.git`，本轮恢复了该提交的 Git 元数据与 `origin/main` 跟踪；未覆盖本地源文件。Git 使用浅层、按需 blob 获取；旧 release/static 文件原本缺失，仍缺失。

本地已有动画实现及 UI 夹具变更，因此不能把本轮数据称为原始 a9a855e 的纯净性能基线。已保留现有变更。详见 [source-baseline.json](source-baseline.json)。修改前文件与原生源码快照在项目的 `dist/optimization-before/`；修改前原生程序为该快照重新编译的 debug/custom-protocol 构建。

## 实施结果

| 项目 | 已实施与验证 | 尚未验收 |
|---|---|---|
| B0 | 动态枚举 Isle 后代进程；按 PID+启动时间处理复用；记录每个实际采样时间、CPU 单核/整机口径、Private WS/Bytes、线程、句柄、GPU engines/memory；支持桌面控制组、指定根 PID、最长 8 小时及预热；保存 Windows 与 Chrome 冒烟 JSON | 三轮正式 release 基线、5 分钟预热、冻结机器噪声预算；真实播放器/语料；WebView2 JS heap/DOM/retainers；ETW 呈现帧时；IPC bytes/in-flight 与 FFT 计数；DPI |
| B1 | npm/package-lock.json 为本轮权威安装入口；独立 Vitest 配置仅发现 src 测试；修正过时天气高度断言；补齐 fixture 播放/随机频谱/卸载控制；修复性能脚本旧选择器与 fixture build；保留 reduced/no-preference 与反转测试；动画短暂层在同一浏览器帧循环内观察；UI 并发上限为 2 | 现有浏览器夹具不能证明 native 命中、真实媒体或完整产品功能覆盖；旧导航测试在本地已缺失，本轮未恢复 |
| M1 | move-only PROPVARIANT owner 在成功或转换失败时释放；线程绑定、不可 Send/Sync 的 WinRT apartment 守卫配对成功初始化，保留已有 STA；用于媒体读取/列表/控制/seek/shuffle/repeat、系统音频，以及后台媒体/捕获线程；接口先于 apartment 释放 | native retained allocation 归因、长时 private commit/句柄/线程趋势，以及真实播放器控制回归 |
| M2 | Canvas 由 Svelte action 绑定实际 DOM；每次请求推进 epoch，覆盖曲目、URL 与像素模式变化；过期 backend/image 结果不能绘制；空图/卸载取消图片回调与 src，清零 backing store 并释放节点引用；关闭后不再填充处理缓存 | 真实 WebView2 跨 100 首语料的 heap/Detached Canvas/Image 与 GPU 回稳对照；不是完整下载/解码预算（M3/M4） |

M1/M2 的代码修复已完成；按原计划严格完成定义，未完成原生性能验收之前暂不勾选。B1 完成，B0 部分完成。

## 验证记录

| 命令/场景 | 结果 |
|---|---|
| `npm ci --prefer-offline` | 通过；首次因并发构建占用原生模块失败，等待构建结束后重装成功 |
| `npm test` | 14 个文件、70 项通过 |
| `npm run check` | 0 errors、0 warnings |
| `npm run test:ui` | 9 项通过：7 项动画回归，2 项真实 Canvas DOM/异步回归；包含 100 次封面卸载重建 |
| `npm run build` | 通过 |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib` | 39 项通过；设备压力测试默认 ignored |
| `cargo test --manifest-path src-tauri/Cargo.toml queries_device_properties_1000_times -- --ignored --nocapture` | 通过，1000 次真实设备列表/名称查询，约 15 秒 |
| `cargo check --manifest-path src-tauri/Cargo.toml` | 通过 |
| `cargo build --manifest-path src-tauri/Cargo.toml --features tauri/custom-protocol` | 通过；debug 原生启动及 WebView2 进程树采样成功 |
| Chrome production fixture | 五个场景各 3 秒冒烟；另做 5 次预热 + 30 次页切换/展开收起的 forced-GC 诊断 |

第一次并发 UI 复测有两个测试在自动化往返后错过短暂 moving cover。修正采样位置并限制 worker 数，未删除封面运动/终态/坐标检查，最终 9 项全部通过。

Chrome forced-GC 诊断在本轮记录中 Nodes=945、listeners=59，前后不变；JS heap 从 2,984,536 到 3,190,164 字节，增加 205,628 字节，低于原脚本 1 MiB 的诊断预算。这是主岛 fixture 的结果，不能代替浮窗或 native 长测。

## 数据与适用限制

- [environment.json](environment.json)：Windows 11 build 26340、Ryzen 5 5600/12 logical cores、RTX 5060 Ti、运行时及依赖版本；DPI 明确 N/A。
- [desktop-control-smoke.json](desktop-control-smoke.json)：3 个桌面控制组样本，采样时存在构建活动，不用于冻结 Idle 阈值。
- [before-native-idle-smoke.json](before-native-idle-smoke.json)：修改前快照的首次进程采样。早期采样器按样本数循环，实际时长大于请求时长；实际时长已记录。随后改为按 wall-clock 时长停止。
- [before-native-smoke-final.json](before-native-smoke-final.json)、[after-native-smoke-final.json](after-native-smoke-final.json)：使用相同最终采样器的修改前/后启动诊断。debug、仅 8 秒启动等待与约 10 秒采样，未做正式预热、场景控制或三轮重复；有同步构建/诊断负载，故不计算性能改善 Delta。
- [fixture-smoke.json](fixture-smoke.json)、[fixture-smoke-retained.json](fixture-smoke-retained.json)：Chrome fixture 入口与 forced-GC 诊断；单页导航场景的 DOM 总量可能包含未 GC 的上一文档。末次脚本为每个场景新建独立 browser context，避免跨场景累积文档；“卸载”改为真正的 Svelte unmount。普通轮不强制 GC，单场景计数仍可能含未回收对象。见 [fixture-final-smoke.json](fixture-final-smoke.json)。

GPU engine 百分比按实例保存，不相加冒充 GPU 总利用率。新出生 PID 首个 CPU 样本为不可用；进程退出可少一个末端样本。CIM/GPU 采集可能耗时超过 1 秒，分析须使用实际时间戳，不把样本数当作严格 1 Hz。工作集变化不等于泄漏字节。

## 可复现入口

使用 npm 的 package-lock.json；pnpm-lock.yaml 本轮保留，未作为本轮安装验证依据。

```powershell
npm ci
npm test
npm run check
npm run test:ui
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml queries_device_properties_1000_times -- --ignored
cargo check --manifest-path src-tauri/Cargo.toml
npm run perf:build -- trial
$env:PERF_SAMPLE_SECONDS = '60'
npm run perf:fixture -- trial
npm run perf:retained -- trial
# 对已启动且已确认场景的 release Isle：
powershell -NoProfile -File scripts/measure-native.ps1 -Seconds 600 -WarmupSeconds 300 -Label release-idle-r1 -IncludeGpu
```

A–F 长测均未完成。短时原生 CPU/内存数据不作性能收益声明。完整基线仍需安静的桌面控制组、正式 release、真实音乐语料及 WebView2/ETW 诊断，才能冻结验收预算并勾选 B0/M1/M2。
