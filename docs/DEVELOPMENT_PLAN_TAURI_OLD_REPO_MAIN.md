# Wind0ws Dynamic Island — 旧 Tauri 主仓库恢复与 Native 功能迁移计划

> 目标：重新以旧 Tauri 仓库 `alent17/Wind0ws_Dynamic_island` 作为主开发仓库，在其现有 Tauri / Svelte UI 基础上继续开发，并将 `alent17/Wind0ws_Dynamic_island-native` 中已经验证成熟的 Rust / Win32 / 播放器能力按模块迁回。  
> 原则：**旧 Tauri 仓库负责主工程和 UI，Native 仓库只作为功能移植来源；Rust 管功能，Svelte 管界面。**

---


---

## 0. 主仓库与迁移策略

### 主开发仓库

```text
https://github.com/alent17/Wind0ws_Dynamic_island
```

旧 Tauri 仓库重新作为最终产品和主开发仓库，继续保留：

- Tauri 工程结构
- Svelte 前端
- 灵动岛 UI
- 设置页面
- CSS / Web Animation
- 前端组件和交互
- Tauri 窗口体系
- 前后端通信
- 已经验证过的视觉方案

### Native 功能来源仓库

```text
https://github.com/alent17/Wind0ws_Dynamic_island-native
```

Native 仓库不再作为最终产品主仓库，只作为功能迁移来源，重点提取：

- Rust Core 改进
- Win32 系统能力
- SMTC
- 网易云音乐专用控制
- 其他播放器控制改进
- 系统音量
- 开机启动
- 截图隐藏
- DPI / 多显示器相关实现
- 其他已经验证成熟的系统逻辑

### 原则上不迁移

- Native UI
- Native 设置页面
- Native UI 动画
- Native UI 布局代码
- 与 Svelte UI 重复的视图层代码
- 仅为 Native UI 服务的窗口封装
- 已被旧 Tauri 版本更好实现的重复逻辑

### 最终关系

```text
旧 Tauri 仓库
Wind0ws_Dynamic_island
        │
        │ 继续作为主项目
        ↓
Tauri 2 + Svelte UI
        +
Rust Backend
        +
必要 Win32
        ↑
        │ 按模块迁移
        │
Native 仓库
Wind0ws_Dynamic_island-native
```

> **UI 留在旧 Tauri 仓库，Native 仓库只迁成熟功能，不迁 UI。**

---

## P-1 — 两仓库迁移审计

正式迁移前，Codex 必须同时读取：

- `alent17/Wind0ws_Dynamic_island`
- `alent17/Wind0ws_Dynamic_island-native`

并在旧 Tauri 主仓库生成：

```text
docs/MIGRATION_AUDIT.md
```

### 审计标记

| 标记 | 含义 |
|---|---|
| ✅ | 直接保留旧 Tauri 实现 |
| ⬆️ | 从 Native 仓库迁入 |
| 🔄 | 两边对比后合并 / 重构 |
| 🗑 | 删除 / 不迁移 |
| 🆕 | 重新实现 |

### 审计范围

- [x] Tauri 配置
- [x] Svelte 前端
- [x] 灵动岛 UI
- [x] 设置页面
- [x] 动画系统
- [x] 状态管理
- [x] Rust 后端
- [x] SMTC
- [x] 网易云控制
- [x] 其他播放器控制
- [x] 系统音量
- [x] 开机启动
- [x] 截图隐藏
- [x] 窗口控制
- [x] DPI / 多显示器
- [x] 配置持久化
- [x] Native UI
- [x] 重复依赖
- [x] 重复 Rust 模块
- [x] 可删除代码

### 审计必须回答

1. 旧 Tauri 仓库哪些部分可以原样保留？
2. Native 仓库哪些功能明显更新或更稳定？
3. 哪些 Rust 模块需要从 Native 仓库迁回？
4. 哪些模块两边都有实现，应该选哪一个？
5. 哪些 Native UI 代码完全不应迁移？
6. 是否存在 Cargo / npm / Tauri 依赖版本冲突？
7. 是否存在配置文件格式差异？
8. 哪些模块风险最高？
9. 第一批最安全的迁移模块是什么？
10. 哪些功能需要迁移后重新测试？

### 审计完成前禁止

- 不允许直接把 Native 仓库 merge 到旧 Tauri 仓库
- 不允许整仓复制 Native 源码
- 不允许覆盖旧 Tauri UI
- 不允许大规模删除旧 Tauri 代码
- 不允许在未对比实现前替换 Rust 模块
- 不允许直接修改 `main` 做实验性迁移

---

## P0 — Git 迁移分支策略

旧 Tauri 仓库继续作为主仓库。

```text
Wind0ws_Dynamic_island
│
├─ main
│
├─ backup/pre-native-return
│
└─ migration/native-features
```

### 操作原则

1. 为旧 Tauri 仓库当前 `main` 创建备份分支或 tag。
2. 从 `main` 创建 `migration/native-features`。
3. 所有 Native 功能迁移先在迁移分支完成。
4. 每迁移一个模块就独立测试。
5. 每个模块尽量独立 commit。
6. 不使用整仓 `git merge native/main`。
7. 迁移完成并通过验收后再合并回 `main`。

### 第一批建议迁移

- [ ] 网易云控制
- [x] SMTC
- [x] 其他播放器控制改进（保留通用 SMTC 能力；按迁移审计继续使用 Tauri 实现）
- [x] 系统音量
- [x] 开机启动
- [x] 截图隐藏
- [x] 多显示器 / DPI 相关能力（能力已迁移；跨缩放验收仍未完成）

### 暂时不要迁移

- [x] Native UI
- [x] Native 设置页
- [x] Native 动画系统
- [x] Native UI 状态管理

> 迁移边界清单中的“暂时不要迁移”项已确认保持在 Native 参考仓库，没有复制进 Tauri；方框勾选表示已遵循该迁移约束。


## 1. 总体架构

```text
┌─────────────────────────────┐
│        Tauri / Svelte       │
│                             │
│  灵动岛 UI                  │
│  设置页面                   │
│  音乐控制 UI                │
│  倒计时 / 音量 / 时钟       │
│  CSS / Web Animation        │
└──────────────┬──────────────┘
               │ invoke / event
               ↓
┌─────────────────────────────┐
│          Rust Core          │
│                             │
│  播放器状态                  │
│  网易云控制                  │
│  SMTC / Win32               │
│  系统音量                    │
│  开机启动                    │
│  截图隐藏                    │
└──────────────┬──────────────┘
               │
               ↓
┌─────────────────────────────┐
│       Windows Native        │
│ HWND / Win32 / COM / API    │
└─────────────────────────────┘
```

---

## 2. 开发计划表

| 阶段 | 任务 | 具体内容 | 验收标准 | 状态 |
|---|---|---|---|---|
| **P0** | 旧 Tauri 主仓库备份 | 给旧 Tauri `main` 打 tag / 建备份分支 | Tauri 基线可随时恢复 | ☑ |
| **P0** | 两仓库迁移审计 | 对比旧 Tauri 与 Native 仓库 | 生成 `docs/MIGRATION_AUDIT.md` | ☑ |
| **P0** | 创建迁移分支 | 创建 `migration/native-features` | Native 功能不直接修改 `main` | ☑ |
| **P1** | 验证旧 Tauri 基线 | 执行 Svelte 诊断、前端构建、Dev 启动及 Tauri Release NSIS 构建 | 0 诊断错误；Dev 窗口启动；安装包构建成功 | ☑ |
| **P1** | 运行时窗口验收 | 启动 Release EXE 并验证透明、无边框、置顶、任务栏和 DPI 行为 | 实机及缩放测试通过 | ☐ |
| **P1** | 无边框窗口 | 移除系统标题栏，自定义窗口外观 | 无系统边框、无多余白边 | ☑ |
| **P1** | 不透明灵动岛表面 | 胶囊表面使用实色；仅窗口宿主画布透明以隐藏矩形窗口边界 | 桌面不出现黑色矩形底框，岛体保持实色 | ☑ |
| **P1** | 置顶 | 灵动岛保持 Always On Top | 普通窗口无法盖住灵动岛 | ☑ |
| **P1** | 任务栏隐藏 | 灵动岛主窗口不进入任务栏 | Alt+Tab / 任务栏行为符合预期 | ☑ |
| **P1** | DPI 适配 | 100% / 125% / 150% / 多显示器测试 | 不错位、不模糊 | ☐ |
| **P1** | 展开态屏幕自适应 | 按所选显示器有效工作区与 DPI 等比缩放展开内容、交互轮廓和原生宿主 | 副屏不溢出，展开内容与点击区域对齐；折叠态尺寸不变 | ☑ |
| **P2** | 建立 Island Store | 创建统一状态管理 | UI 不直接读取后端杂乱状态 | ☑ |
| **P2** | 状态机 | 建立 `idle / media / expanded / timer / volume` 等状态 | 状态切换路径明确 | ☑ |
| **P2** | Rust → Svelte 通信 | 后端只通过 event / invoke 提供数据 | UI 与 Rust 解耦 | ☑ |
| **P2** | Settings Store | 设置统一存储 | 修改设置后即时生效并持久化 | ☑ |
| **P3** | 重做灵动岛基础 UI | 只完成黑色胶囊、尺寸、圆角、布局 | 静态 UI 达到设计要求 | ☑ |
| **P3** | 左侧媒体区域 | 专辑封面、歌曲、歌手 | 文本溢出、长歌名正常 | ☑ |
| **P3** | 右侧功能栏 | 倒计时 / 设置 / 音量 / 时钟 | 与岛体风格统一 | ☑ |
| **P3** | Hover 状态 | 鼠标进入 / 离开时展开收起 | 无跳变、无闪烁 | ☑ |
| **P3** | 点击区域 | 为不同控件定义独立点击区 | 不出现误触 | ☑ |
| **P4** | 灵动岛形变动画 | `width / height / radius / transform` 统一动画 | 收起、展开自然连续 | ☑ |
| **P4** | 左下角锚定动画 | 保留“向右上生长”逻辑 | 锚点视觉不漂移 | ☑ |
| **P4** | 内容进出动画 | 封面、文字、按钮分别控制 opacity / transform | 不抢动画、不突然出现 | ☑ |
| **P4** | 状态切换动画 | Media → Timer → Volume 等 | 状态间不闪、不重叠 | ☑ |
| **P4** | 动画中断处理 | 快速 Hover / 离开 / 切歌时处理动画打断 | 不出现尺寸卡死 | ☑ |
| **P5** | 恢复 SMTC | 获取 Windows 当前媒体信息 | 支持系统播放器 | ☑ |
| **P5** | 恢复网易云控制 | 使用已确定的网易云专用方案 | 网易云播放 / 暂停 / 上下曲正常 | ☐ |
| **P5** | 其他播放器 | 保持原有控制方式不变 | Spotify 等播放器正常 | ☑ |
| **P5** | 专辑封面更新 | 切歌实时更新封面 | 不残留上一首封面 | ☑ |
| **P5** | 播放状态同步 | Pause / Playing / Stopped | UI 状态实时一致 | ☑ |
| **P6** | 音量模块 | 获取和调整系统音量 | UI 与 Windows 音量双向同步 | ☑ |
| **P6** | 倒计时模块 | 新建 / 暂停 / 结束提醒 | 收起后仍正常计时 | ☑ |
| **P6** | 时钟模块 | 时间显示和格式设置 | 不造成持续高 CPU | ☑ |
| **P6** | 开机启动 | 恢复自动启动 | 开关可控、重启有效 | ☑ |
| **P6** | 截图隐藏 | 使用 Win32 原生方案 | 截图时符合设定 | ☑ |
| **P7** | 重做设置页面 | 设置窗口完全使用 Svelte UI | 不再出现 Native 设置页崩坏 | ☑ |
| **P7** | 设置页系统标题栏 | 使用 Windows 原生边框、标题和窗口控制按钮 | 标准窗口装饰可见 | ☑ |
| **P7** | 外观设置 | 圆角、位置、功能显示等 | 修改即时预览 | ☑ |
| **P7** | 模块开关 | 音乐 / 时钟 / 倒计时等可独立显示 | 配置正确保存 | ☑ |
| **P8** | 性能治理 | 检查事件监听、Timer、Store 更新频率 | 空闲 CPU 保持低占用 | ☐ |
| **P8** | 内存治理 | 避免封面图片、事件监听泄漏 | 长时间运行无明显增长 | ☐ |
| **P8** | EXE 测试 | Debug 和 Release 对比 | Release 不出现额外问题 | ☐ |
| **P8** | 多显示器测试 | 主副屏、缩放不同、分辨率不同 | 灵动岛位置正确 | ☐ |
| **P8** | 睡眠恢复 | Windows 睡眠 / 唤醒后测试 | 不丢播放器、不丢窗口 | ☑ |
| **P9** | 清理旧 Native UI | 删除确认不再使用的 Native UI 层 | 不留下双 UI 架构 | ☑ |
| **P9** | 重构目录 | 整理 `src` / `src-tauri` | 结构清晰 | ☑ |
| **P9** | 编写开发文档 | 架构、模块、事件、状态机说明 | 后续 Codex 能直接理解 | ☑ |
| **P9** | 发布候选版长期验收 | Release Candidate 长时间稳定运行与设备验收 | 可长期日常使用 | ☐ |

> P1 运行时窗口验收进度（2026-10-06）：已重建并启动 Release EXE。在本机两台 1920×1080、100% 缩放显示器上，透明无边框、置顶及任务栏 / Alt+Tab 隐藏已验收；125% / 150% 缩放尚未覆盖，因此该综合验收项保持未完成。

> Release 构建与启动冒烟（2026-10-06）：`pnpm tauri build --bundles nsis` 成功生成 Release EXE 和 NSIS 安装器；单独启动 Release EXE 后进程正常、桌面可见灵动岛，无空白窗口或矩形黑底。此项只证明本机 100% 缩放环境的启动冒烟，不替代播放器、混合 DPI、睡眠恢复或长时间运行验收。

> 空闲资源短采样（2026-10-06）：Debug 与 Release 两个实例同时空闲采样 20 秒，12 个逻辑处理器上合计进程平均 CPU 约 0.42%，私有内存合计约 49.7 MB。时间过短且不是单实例长跑，作为即时基线记录；P8 性能与内存治理仍需单实例和长时间验收。

> Debug 交互与稳定性采样（2026-10-06）：直接操作当前工作区 Debug 版，验证收起 → 展开媒体页 → 系统音量页 → 返回媒体页 → 收起；可访问树能读到歌曲元数据、播放进度、播放控件和音量滑块（当前值 24%，测试未改动系统音量），视觉上无空白窗口或矩形黑底。单实例约 3 分钟采样期间 CPU 累计增加 6.34 秒（12 逻辑处理器，约 0.29% 平均），私有内存从约 30.1 MB 到 31.7 MB；界面展开/加载天气时增长约 1.8 MB，随后最后 30 秒保持在 31.7–31.9 MB。该时长不足以完成 P8 长时间内存验收。

> WebView2 短时内存采样（2026-10-06）：按 `com.isle-app.isle\EBWebView` 进程组汇总，5 分钟内持续为 6 个子进程，私有内存从约 302.8 MB 到 307.5 MB；中间短暂达到约 332.6 MB 后回落，结束阶段在约 306–308 MB 间波动。此短采样没有看到持续单调增长，但不足以完成长时间验收。

> 网易云 CDP 验收前置（2026-10-06）：本机网易云进程版本为 3.1.41.205529，启动参数没有 `--remote-debugging-port`，`127.0.0.1:9223` 当前无监听；当前 SMTC 媒体信息和常规传输控件可见，无法在此会话做真实客户端联调。

> 目录整理与自动化验收（2026-10-06）：Svelte 组件按职责移入 `src/lib/components/{island,media,settings,time,volume}`；Rust 命令、模型、服务和共享状态继续按模块分层。目录迁移后 `pnpm check`（0 错误 / 0 警告）、`pnpm build`、`pnpm test`（13 个文件、63 项通过）和 `cargo test --no-default-features`（31 项通过）均成功。几何测试改为验证当前 600×210 展开尺寸；修正竖向屏幕边缘浮岛展开时原生宿主未给 22px 间距留位的越界。

> 展开态屏幕自适应（2026-10-06）：展开媒体页按参考布局重排为 600 × 249.1 基准尺寸，并以当前选中显示器工作区 / DPI 计算内容缩放；初始宿主定位、表面偏移与动画命中宿主现共用同一展开缩放，避免不同尺寸的空白宿主和交互区域。副屏启动、`pnpm check` 与 `pnpm build` 通过。125% / 150% 的完整视觉验收仍归入上方未完成的 DPI 适配项。

> 副屏 DPI 抽查（2026-10-06）：在 Windows“系统 → 显示”中确认副屏原为 100%，依次切换到 125% 和 150%；两档设置页截图中均仍可见 Isle 小胶囊，Isle 进程保持响应；随后将副屏恢复为 100%。切到 150% 后，UI 自动化窗口枚举不再返回 Isle 主窗口（仅设置窗与 Studio 窗仍可操作），因此未能核验展开态布局、命中区域与清晰度；P1 完整 DPI 验收及 Milestone 1 / 4 的 DPI 项保持未完成。

> P2/P6/P7/P9 进度（2026-10-06）：`src/lib/islandStore.ts` 集中主岛展开、悬停、页面和媒体会话状态，并派生 idle/media/expanded/timer/volume 等视图；`src/lib/settingsStore.ts` 负责 main 与 FloatingWindow 的设置读取、`settings-updated` 同步和初始读取防竞态。Studio 保留未保存的编辑草稿。设置页使用标准 Windows 窗口边框、标题栏与系统窗口控制按钮。截图监控新增 Windows AppCapture 视频捕获状态查询，并映射到录屏隐藏偏好；无法覆盖不使用 Windows AppCapture 的所有第三方捕获程序，屏幕共享检测仍未实现。仓库不含 Native UI 渲染层；新增 `docs/ARCHITECTURE.md` 与 `docs/STATE_MACHINE.md` 记录架构和状态边界。后端 invoke/event 与前端 API 封装已核对完成。`src/lib/components/` 现按 island/media/settings/time/volume 分组，Rust 继续按 commands/models/services/state 分层。`pnpm check` 与 `pnpm build` 均通过；快速 hover、计时器完成、标题栏交互、捕获事件和跨窗口行为仍需运行时验收。

> Studio 设置同步与切歌封面过渡（2026-10-06）：Studio 现订阅 `settings-updated` 并从 `settingsStore` 读取跨窗口设置快照；未保存的外观草稿字段优先保留，干净字段跟随外部更新。新增 `CoverArt.svelte`，在新封面加载成功后双图交叉淡化，并保留上一个有效封面直到切换开始；失败加载不会先清空当前图片。代码路径已实现，真实播放器切歌的视觉连续性仍待运行时确认。`pnpm check` 0 错误 / 0 警告，`pnpm test` 13 个文件、64 项通过，`cargo test --no-default-features` 31 项通过，`pnpm build` 通过。

> 网易云播放模式控制实现（2026-10-06）：在 `src-tauri/src/services/netease_cdp.rs` 增加单独的 loopback CDP 适配器，Tauri 暴露读取 / 循环播放模式命令，主岛仅在读取到受支持的网易云模式后显示模式按钮。网络操作在 blocking pool 执行，CDP endpoint、页面、大小、超时均有限制，写入后做状态读回验证；不覆盖 AI/FM/未知模式。该代码不重启播放器，本机 9223 无监听，因此真实网易云模式切换仍未验收，P5 和迁移候选框保持未完成。后端单测已新增（总数 36 项，通过情况见本轮验证记录）。

> 捕获隐私能力范围（2026-10-06）：设置页不再提供无效的“屏幕共享时隐藏”开关，并说明当前不支持检测屏幕共享。录屏说明限定为 Windows AppCapture 能报告的状态，不能保证发现第三方录屏软件；截图监控依赖 Print Screen 与 Win + Shift + S 快捷键检测。以上属于有限覆盖，未通过运行时验收前不扩展勾选范围。

> 恢复后布局刷新（2026-10-06）：主窗口在 WebView 恢复可见、重新获得焦点或 pageshow 时重新枚举显示器并无动画校正宿主边界，同时刷新时钟显示。此路径补充了休眠期间显示器 / 工作区变化后的恢复处理；单次睡眠唤醒实机结果见下方记录。

> 睡眠唤醒单次验收（2026-10-06）：Windows System 日志记录 S3 入睡（Kernel-Power 42）与恢复（Kernel-Power 107，Power-Troubleshooter 1）。恢复后 Debug 进程仍为 PID 28984、Responding=true；其 Isle 主窗口仍可见并位于副屏工作区（边界 x=-1222..-697），7 个应用关联 WebView2 进程仍在；原有 `cloudmusic.exe` 进程也仍运行。此单次循环足以勾选基本睡眠 / 唤醒测试，不代表长时间或重复循环稳定性已验收。

> Release 包重建（2026-10-06）：Studio 同步、封面过渡与网易云 CDP 功能合入后，`pnpm tauri build --bundles nsis` 成功生成最新 Release EXE 和 NSIS 安装器。产物位于 `src-tauri/target/release/isle.exe` 与 `src-tauri/target/release/bundle/nsis/Isle_1.0.11_x64-setup.exe`。Release Candidate“可长期日常使用”仍需长时间运行和设备验收，因此 P9 保持未完成。

> 实现盘点（2026-10-06）：P3–P7 中已勾选的行表示对应 Svelte / Rust 功能路径已实现；Hover 与独立点击区已在 Milestone 2 核对。完整视觉验收、播放器实机兼容、不同缩放下的 DPI、重启自启动和长时间运行等条件仍需 P1/P8 设备验收。网易云播放模式 CDP 路径已实现但真实客户端端点尚不可用；通用录屏兼容和屏幕共享检测仍未完成；Windows AppCapture 支持的系统视频捕获已有检测代码，仍需实机确认。

---

## 3. 当前目录结构

```text
Wind0ws_Dynamic_island/
│
├─ src/
│  ├─ App.svelte / FloatingWindow.svelte / Studio.svelte / TimerWindow.svelte
│  ├─ lib/
│  │  ├─ api/
│  │  ├─ components/
│  │  │  ├─ island/
│  │  │  ├─ media/
│  │  │  ├─ settings/
│  │  │  ├─ time/
│  │  │  └─ volume/
│  │  ├─ islandStore.ts / mediaStore.ts / settingsStore.ts / spectrumStore.ts
│  │  └─ geometry, motion, media, timer, spectrum domain modules
│  ├─ styles/
│  ├─ utils/
│  ├─ main.ts / studio.ts
│  └─ TimerWindow.svelte
│
├─ src-tauri/
│  └─ src/
│     ├─ commands/
│     ├─ models/
│     ├─ services/
│     ├─ state/
│     ├─ audio.rs / event_bus.rs / error.rs / utils.rs
│     └─ lib.rs / main.rs
│
└─ docs/
   ├─ ARCHITECTURE.md
   ├─ DEVELOPMENT_PLAN.md
   └─ STATE_MACHINE.md
```

---

## 4. 开发优先级

严格按照以下顺序推进，避免功能、UI、窗口逻辑同时修改：

```text
P0 备份 / 清点
        ↓
P1 Tauri 窗口
        ↓
P2 状态管理
        ↓
P3 静态 UI
        ↓
P4 动画
        ↓
P5 播放器
        ↓
P6 系统功能
        ↓
P7 设置页面
        ↓
P8 稳定性
        ↓
P9 清理 / 发布
```

### 约束

- **P3 完成之前，不开始复杂动画。**
- **P4 完成之前，不继续往灵动岛 UI 上堆新功能。**
- UI 不直接操作播放器或系统能力。
- Rust 后端不负责具体 UI 布局。
- 所有状态变化必须经过统一 Store / State Machine。
- Win32 只用于 Tauri / WebView 无法可靠完成的功能。

---

## 5. Milestone 1 — Tauri UI 基座

第一阶段只完成基础框架，不接入复杂业务。

- [x] 当前 Native 项目完整备份
- [x] 创建 Tauri 2 + Svelte 前端
- [x] 保留现有 Rust 后端
- [x] 灵动岛透明无边框窗口成功启动
- [x] Always On Top
- [x] 不进入任务栏和 Alt+Tab
- [x] 正确固定到屏幕位置
- [ ] DPI 正常
- [x] 完成最基础黑色灵动岛胶囊
- [x] 设置窗口能够正常打开
- [x] Debug / Release 启动冒烟

> Native 工作树备份（2026-10-06）：完整源码、`.git` 历史、暂存/未暂存更改及未跟踪素材已归档至 `E:\Wind0ws_Dynamic_island-native-backup-2026-10-06.tar.gz`；只排除可重建的 `target/` 编译目录。SHA-256：`46F62DA2763F07980FE9D0275C519AB64420CEF26F39E4A1D7D7AF26C1182BC8`。

### Milestone 1 完成后暂停

此阶段暂时不要：

- 接网易云
- 做复杂动画
- 做倒计时
- 重做所有功能
- 清理全部旧代码

先确认 **Tauri 窗口层 + Svelte UI 基础架构稳定**，再进入下一阶段。

---

## 6. Milestone 2 — 灵动岛 UI 与动画

- [x] 完成灵动岛完整静态布局
- [x] 完成媒体区域
- [x] 完成右侧功能栏
- [x] 建立 Island State Machine
- [x] 完成 Hover 展开 / 收起
- [x] 完成左下角锚定向右上生长
- [x] 完成内容淡入 / 位移动画
- [x] 完成快速 Hover 动画中断处理
- [x] 完成 Media / Timer / Volume 状态切换
- [x] 确认窗口尺寸变化与前端动画同步

---

## 7. Milestone 3 — 媒体系统

- [x] 恢复 Windows SMTC
- [ ] 恢复网易云音乐专用控制方案
- [x] 保持其他播放器原有逻辑
- [x] 播放 / 暂停
- [x] 上一首 / 下一首
- [x] 歌曲名称
- [x] 歌手
- [x] 专辑封面
- [x] 播放状态同步
- [x] 切歌时 UI 不闪烁

---

## 8. Milestone 4 — 系统功能

- [x] 系统音量
- [x] 倒计时
- [x] 时钟
- [x] 开机启动
- [x] 截图隐藏
- [x] 多显示器
- [ ] DPI
- [x] 睡眠 / 唤醒恢复（已通过一次 S3 循环；重复与长时间稳定性待观察）
- [x] 窗口位置持久化

---

## 9. Milestone 5 — 设置页面

- [x] 设置页完全迁移到 Svelte
- [x] 无边框窗口
- [x] 设置页原生系统标题栏与窗口边框
- [x] 最小化
- [x] 关闭
- [x] 拖动
- [x] 外观设置
- [x] 灵动岛位置设置
- [x] 功能模块开关
- [x] 动画相关设置
- [x] 设置即时预览
- [x] 配置持久化
- [ ] 设置页长时间运行不崩溃

---

## 10. Milestone 6 — 稳定性与发布

- [x] 检查事件监听泄漏
- [x] 检查 Timer 更新频率
- [x] 检查 Store 无意义重复更新
- [x] 检查封面资源释放
- [x] 检查 WebView 内存占用（已记录短时基线；长时间稳定性仍待验收）
- [x] Debug 启动冒烟
- [x] Release 启动冒烟
- [x] 多显示器基础测试
- [ ] 不同 DPI 测试
- [x] 睡眠唤醒测试（一次 S3 循环；详见本节验收记录）
- [ ] 长时间挂机测试
- [x] 清理旧 Native UI
- [x] 整理项目结构
- [x] 完善开发文档
- [x] 打包 Release Candidate（NSIS 安装包已生成；长期日常使用验收仍未完成）

---

## 11. 核心开发原则

### 前端负责

- UI 布局
- 动画
- 状态展示
- 用户交互
- 设置页面
- 灵动岛视觉表现

### Rust 负责

- Windows 系统 API
- 播放器控制
- 网易云控制
- SMTC
- 音量
- 开机启动
- 截图隐藏
- 多显示器与窗口能力
- 配置持久化底层能力

### 禁止

```text
Rust 直接控制大量 UI 细节
UI 直接操作 Win32
每个功能自己修改窗口尺寸
每个组件维护一套自己的展开状态
多个模块同时控制动画
播放器状态直接耦合 UI DOM
```

统一关系：

```text
Windows / Player
      ↓
   Rust Core
      ↓
 event / invoke
      ↓
     Store
      ↓
 State Machine
      ↓
 Svelte UI
      ↓
 Animation
```

---

## 12. 最终目标

最终架构应满足：

- UI 可以快速修改
- 动画可以独立调试
- Windows 原生能力完整保留
- 网易云拥有单独控制方案
- 其他播放器维持通用控制逻辑
- 设置页面稳定
- 灵动岛与右侧功能区域视觉统一
- 不因增加新功能破坏窗口系统
- 不因修改 UI 影响播放器后端
- 后续 Codex 能根据计划逐项完成并勾选状态

---

**当前推荐起点：先在旧 Tauri 仓库执行 `P-1 两仓库迁移审计 → P0 Git 分支准备 → 再逐模块迁移 Native 功能`。**

