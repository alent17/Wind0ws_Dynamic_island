# Isle Optimization Plan

> 2026-10-09 更新：M2 浮窗重开空封面已记录真实事件时序、修复并通过封面正确性/Canvas 所有权验收：100 首真实切歌及四次重开全部通过，Detached Canvas=0。M2 勾选；GPU 长期回稳及 Release 性能收益未确认，不能据此判定内存/Idle 达标。M3 保持未勾选，后续 Release C 前后构建必须包含同一 M2 修复。见 [M2 验收报告](performance/m2-reopen-2026-10-08/ACCEPTANCE.md)。

> 2026-10-08 历史记录：三轮最新 Release OS 基线已采集；M1 正确性/资源压力验收通过。该轮 M2 第 50 首重开失败，详见 [原验收报告](performance/acceptance-2026-10-08/REPORT.md)。完整 B0 的 IPC/ETW/噪声校准仍待补。

> 原始审查基线为 a9a855e。2026-10-07 已按项目所有者的“执行”指令实施 B1、M1、M2 的代码与验证工作，并补充 B0 的采样工具及 Windows 冒烟记录。详细结果见 [本轮执行记录](performance/optimization-2026-10-07/EXECUTION.md)。完整性能验收尚未完成，未将短时诊断包装成长测结果。

## 1. 审查基线与证据口径

- 项目：<https://github.com/alent17/Wind0ws_Dynamic_island>
- 审查日期：2026-10-07。
- 审查提交：`a9a855ef62d2596775115c70b82f63af8a67229e`，提交时间 `2026-10-06T20:22:01+08:00`。
- 项目版本：`1.0.11`；`package-lock.json` 解析版本包括 Svelte `5.54.1`、Tauri JS API `2.10.1`、Vite `6.4.1`、TypeScript `5.6.3`、Vitest `5.0.0`。
- Rust 配置位于 `src-tauri/Cargo.toml`，仓库根目录没有独立 Cargo.toml。Tauri 使用 `version = "2"`，Windows 绑定使用 `0.52`，CPAL `0.15`，rustfft `6.1`，image `0.25`；运行基线仍须记录实际解析版本。
- 配置检查包括 `package.json`、两个 JS 锁文件、`src-tauri/tauri.conf.json`、`src-tauri/capabilities/default.json`、Vite/Playwright 配置及 Rust release profile。
- 本次是源码审查，环境为 Linux，未运行 Windows API、WASAPI 或 WebView2。文中的 Hz 是源码配置/理论上限，CPU/GPU 成本是定性判断，不是测量结果。Forced Reflow 时长、实际 FPS、泄漏增长率须由 Windows trace/heap snapshot 证实。

证据等级：**确定**表示调用链可以直接证明；**风险**表示存在保留、并发或性能触发条件，仍需复现；**待测**表示必须有运行数据才能下结论。文件行号以审查提交为准。

### 已检查的实现范围

| 范围 | 实际代码与审查内容 |
|---|---|
| 主窗口 | `src/App.svelte`：启动、媒体、音频、倒计时、天气、捕获、监视器、窗口放置、隐藏、调试与清理 |
| 表面与动画 | `src/lib/components/island/IslandSurface.svelte`、`src/lib/islandMotion.ts`、`src/lib/islandGeometry.ts`、`src/lib/islandStore.ts`；六个 spring、封面 overlay、页面切换与命中区域 |
| 媒体/封面 | `src/FloatingWindow.svelte`、`src/lib/mediaStore.ts`、`src/lib/mediaClock.ts`、`src/lib/spectrumColors.ts`、媒体子组件；Rust `commands/media.rs`、`services/media.rs`、`services/image.rs`、`services/color.rs`、`services/netease_cdp.rs`、`utils.rs` |
| Spectrum/音频 | `src/lib/components/media/Spectrum.svelte`、`src/lib/spectrumStore.ts`、`src/lib/spectrumRender.ts`；Rust `audio.rs`、`commands/audio.rs`、`services/system_audio.rs`、`lib.rs` 中消费者生命周期；占位 `services/spectrum.rs` 与真实链路分开判断 |
| Timer | `src/TimerWindow.svelte`、`src/lib/countdown.ts`、TimerPanel/RollingDigit/RollingNumber/ComicClock；主窗口事件桥与原生窗口创建/关闭 |
| Weather/系统指标 | `src/lib/api/idle.ts`、InfoPanel、App/Studio 调度；Rust `commands/idle.rs`、`services/idle.rs` 的 sysinfo、网络、天气缓存 |
| Settings | `src/Studio.svelte`、`src/lib/settingsStore.ts`、StudioSlider/StudioSelect；Rust `commands/settings.rs`、`services/settings.rs`、`models/settings.rs` |
| Window/Capture/Monitor | Rust `commands/window.rs`、`commands/monitor.rs`、`lib.rs`；`src/lib/captureMode.ts`、VisibleWindow、前端窗口 API；点击穿透、DPI、捕获检测、native bounds |
| IPC/缓存/基础设施 | `src/utils/eventManager.ts`、eventConstants、全部前端 API、Rust `event_bus.rs`、`services/cache.rs`、commands/cache、状态/模型/错误处理、入口与样式 |
| 验证设施 | `src/lib/*.test.ts`、Rust 模块测试、`ui-tests/`、`scripts/measure-*`、`scripts/build-performance.mjs`、`docs/performance/*.json`；检查测试覆盖与报告适用范围 |

### 应保留的已有优化

1. 展开/收起使用预留的 native host，轮廓与交互区域变化；`set_island_interaction_region` **仅更新 Rust 内存中的命中区域**，没有逐次 `SetWindowRgn` 或 resize。
2. `Spectrum.svelte` 已按 active/playing/realtime/reduceMotion 启停消费者；前端引用计数、350 ms 停止缓冲、Rust 按窗口标签保留消费者，窗口销毁会释放。不能把当前实现描述为“FFT 永久运行”。
3. 音频回调不直接发送 IPC；2048 点 FFT 使用预分配缓冲、全零输入跳过 FFT；发布者 50 ms 检查，变化阈值 0.01，稳定值约 1 Hz 心跳；前端绘制约 30 Hz 并跳过未变化像素。
4. 原生媒体轮询已省略未变化的封面大字段；Timer 广播按秒去重；浮窗处理结果已有 **12 项 LRU**，HD 封面缓存已有 **64 项/约 8 MiB URL 字符预算**，时间线缓存有 256 项上限，磁盘缓存有 512 MiB 总配额。不能称这些缓存无限增长，也不能把 URL/磁盘配额当作解码图片/GPU 内存上限。
5. `createAsyncCleanup`、mediaStore/settingsStore/spectrumStore 已处理多数迟到注册与释放；副窗口使用 VisibleWindow 控制挂载，关闭窗口会销毁 WebView。不能归因为“启动时预加载所有副窗口并永久保留”。
6. 天气后端已有 30 分钟 TTL 与共享请求锁；sysinfo 已移到 blocking pool。主入口懒加载浮窗/Timer，Studio 另一个入口。主灵动岛没有默认持续的 backdrop blur。

## 2. 优先级与最严重问题

| 标记 | 含义 | 使用原则 |
|---|---|---|
| 🔴 P0 Critical | 资源泄漏、可达崩溃、严重阻塞/CPU | 先建立可复现证据；不把“可能高”包装成实测严重 CPU |
| 🟠 P1 High | 高频 IPC、后台 CPU、动画架构/状态正确性 | 对主要常驻成本与交互可靠性优先处理 |
| 🟡 P2 Medium | UI、绘制、组件边界 | 数据与基础状态稳定后处理 |
| 🟢 P3 Low | 清理、代码质量、可选重构 | 只有明确收益才实施 |

| # | 问题 / 判断 | 证据 | 任务 |
|---|---|---|---|
| 1 | **确定：设备名称 PROPVARIANT 未释放，COM 初始化未配对。** 正常查询音频设备即可触发前者；后者的实际内存增长量仍待测。 | `services/system_audio.rs:33–49` 的 GetValue 没有 PropVariantClear；Windows 0.52 生成绑定没有 PROPVARIANT Drop；`services/media.rs:644–650,957–965` 重复 CoInitializeEx 无对应 CoUninitialize | M1 |
| 2 | **确定：同步媒体控制阻塞 native 主线程。** play/pause 明确 sleep 150 ms，另有 WinRT 阻塞等待；图片/音量同步命令也占主线程。 | `commands/media.rs:93–142`、`commands/audio.rs`、`services/media.rs:957–1041` | C1 |
| 3 | **确定：浮窗可能继续画到已脱离 DOM 的 Canvas。** cover→无封面/无会话→cover 后缓存引用没有更新；旧图回调也可能覆盖新图。是生命周期/显示 bug，不能据此直接断言每首歌泄漏一个 Canvas。 | `FloatingWindow.svelte:897–944,1008–1026,1066–1081,1532–1598` | M2 |
| 4 | **确定存在缺口：下载、解码及并发峰值缺少完整预算。** 部分下载整包读取后才判断大小，SMTC 未限制 stream size，Canvas 使用原图尺寸，pixel_size=0 可触发 panic；release 为 panic=abort。 | `services/cache.rs:287–337`、`services/media.rs:133–170,732–763`、`services/image.rs:19–37,47–77`、`src-tauri/Cargo.toml` | M3/M4 |
| 5 | **确定：Idle 仍有多个永久轮询。** 截图按键 20 Hz、媒体 1 Hz、全屏/录制 2 Hz、远处鼠标 5 Hz；Capture 选项全部关闭也未取消对应检测。实际 CPU 值待测。 | `lib.rs:175–315`、`commands/window.rs:62–70,410–522` | C2 |
| 6 | **确定：CSS 隐藏不是完整工作暂停。** 浮窗 capture-hidden 不进入 VisibleWindow 卸载分支，仍有进度计时、MV 与旋转；主岛原生移出屏幕也不等于 document.hidden。 | `FloatingWindow.svelte:76–79,848–855,1541–1575,2033,2136`、`App.svelte:292–304,639–659,1070–1088` | C3 |
| 7 | **确定：交互区域在 spring 每帧变化时发送 IPC。** 72 点 polygon 重算/JSON 序列化，fire-and-forget，无节流/背压，settled 被忽略；失败后 signature 仍被视为已发送。 | `IslandSurface.svelte:477–485`、`App.svelte:220–259`、`commands/window.rs:557–588` | I1 |
| 8 | **确定：封面 rAF 每帧 DOM measure + layout 属性写入。** 同时外壳 width/height/shoulder 改变，具备 Forced Reflow 条件；是否成为主要掉帧源须 trace。 | `IslandSurface.svelte:333–378,548–551,793` | A2 |
| 9 | **确定：动画完成标准与隐藏状态不统一。** 多路 spring、封面 250 ms、页面 100/180 ms、原生位置 spring 分离；收起时封面主动消失；手动隐藏等固定 460 ms，自动隐藏未先收起却按 Compact 位移。 | `IslandSurface.svelte:286–438,626–633`、`App.svelte:1038–1088`、`commands/window.rs:1128–1278` | A1/A3/A4 |
| 10 | **确定：视觉与命中区域的缩放公式不一致，UI 整体 zoom 也过小。** App scale=.625×layoutScale，即 .3875–.625；geometry helper 却把它下限截到 .62。封面/按钮/字号随页面整体缩小，工具条入口密集。 | `App.svelte:756–758`、`islandGeometry.ts:208,300–319`、`IslandSurface.svelte:253–258,815,846–869` | I2/U1/U2 |

此外：设置的跨窗口并发更新/非原子写入（M6）、重复封面解析与过期主色返回（M4）、EventManager 迟到注册缺口（M5）、自动化配置失效（B1）也必须进入计划。

## 3. 动画逐项审查

| 转换 | 当前行为与问题 | 建议方向 |
|---|---|---|
| Compact → Hover | 80×28/r14 → 90×30/r15；六路运动中尺寸相关 spring 同时更新。没有证据证明仅小幅 hover 就持续 native resize；有逐帧 region IPC。 | 小幅尺寸或 transform 均可，按 trace 选择；明确 Hovering 意图与取消规则，避免 hover/click/auto-hide 改不同状态。 |
| Hover → Expanded | width/height/radius/shoulder/style 各自 spring；Compact DOM 在 mode=expanded 时立即移除，compactOpacity 无法完成跨状态淡出；封面到移动中的目标每帧测量。 | 同一控制器协调外壳与内容；短暂保留两个内容端点并正确 inert；封面测量一次，动画位置从 geometry 推导。 |
| Expanded → Compact | expanded 层收起时 opacity 被设为 1；封面两端隐藏到 shapeSettled，再重新出现，没有逆向连续轨迹。 | 按反向进度淡出/移动，收起可被打断并从当前画面继续，不能先把封面清掉。 |
| Floating → Edge / Edge → Floating | styleMorph 同时改变 gap、四角 radii、72 点 clip-path；width/height/shoulder 也有目标重设。侧边 navigationHostFor 只为 Floating 额外加 22，切换会改变 native host 宽度。不同 edge 另用 140 ms fade-out + placement + 180 ms fade-in。 | 同 edge 保留两种形态共同 host；固定画面锚点，统一过渡 epoch。换显示器/edge 时阶段性移动一次，不能多个放置事务重叠。 |
| Page → Page | page 已改变目标尺寸，renderedPage 仍保留旧页 100 ms；之后内容 180 ms enter，外壳 spring 完成时间不固定。music 600 宽→工具 300 宽可能先裁掉旧页。Marquee 的 ResizeObserver 可因尺寸变化反复测量/重启动画。 | PageTransition 区分 exit、morph、enter；旧内容 exit 完成再换 owner，目标布局预计算，测量不追随每帧壳布局。 |
| Visible → Hidden | 主 App 立即 `visible=false` 卸载内容，但外壳 mode 仍可 expanded；原生 hiddenPlacementFor 使用 compact 尺寸。real App 未使用模拟预览的 hideMorph。 | 明确 Hiding→Hidden；普通隐藏等待受控收起/隐藏完成，截图/录制隐藏优先即时遮蔽全部外壳；保留 2 个物理像素唤醒区域要单独计算。 |
| Hidden → Visible | isHidden=false 后内容立即恢复，native 位置另一个 spring；隐藏中媒体/页面改变可能让恢复目标过时。 | Restoring 取最新意图，容器/interaction 初始化后再恢复内容；不会依赖恢复时 getBoundingClientRect 持续测量。 |

### 属性与调度判断

- **width/height**：真实 DOM layout 属性每帧变化；Svelte 是细粒度、批量更新，不能说 App 整棵树每帧重渲染。问题是几何、clip、区域 effect 的依赖扇出与子布局，不是 `$state` 本身有问题。
- **border-radius/clip-path**：radii 插值与肩部 polygon 每帧更新；可能触发 paint/mask，是否独立 GPU layer 不能仅凭 CSS 判断。Floating 的 clip-path=none 时仍生成 polygon 供命中区域，适合走 round-rect 快路径。
- **position/translate/scale**：CSS anchor transform、expanded scale 与原生 SetWindowPos 是不同坐标空间；pressed 表面 scale=.97 与命中 polygon 暂时不同，需容差而非再加逐帧 IPC。
- **spring**：六个 spring 分别负责不同属性，并非六个 writer 写同一属性；竞争在状态目标、内容挂载和完成时机。统一所有权比强制“只剩一个 spring”更重要。
- **requestAnimationFrame**：主要持续候选是开启 debug FPS；封面是 250 ms 期间连续 rAF；RollingDigit 双 rAF 有界但未显式取消。Spectrum 现有 30 Hz capped timer 并非每帧空转 rAF。
- **getBoundingClientRect**：封面目标每帧调用是重点；ArcVolumeControl 每个 pointermove 测量是较低优先级且需核实是否仍可达；VolumePanel 仅点击提交测量一次，不应一概标高频。
- **native resize**：普通展开/收起已没有逐帧 resize。`animate_window_bounds` 在隐藏/恢复/放置期间约每 16 ms 调 SetWindowPos，重复提交 w/h，没有只移动标志；同尺寸是否仍引起昂贵 composition 必须 ETW 确认。

### IslandAnimationController 的建议边界

**建议引入一个可打断的动画控制器，但先修内存、CPU、IPC，再落地动画。** 保留目前固定 native host。控制器产生一个 frame snapshot；尺寸、radii、shoulder、gap、contentProgress、coverProgress 从该 snapshot 推导。可用共享时间线或同一 owner 下的向量 spring，先小范围验证再选方案。不能对整棵内容用 scaleX/scaleY 拉伸文字。

| 状态 | 进入/退出规则与资源职责 |
|---|---|
| Compact | 无持续动画，展示单一主要状态；按需 Spectrum 与分钟时钟。 |
| Hovering | 小幅响应指针/焦点，离开回 Compact，展开请求进入 Expanding。 |
| Expanding | 固定 target/epoch，预计算目标几何，保持封面身份；collapse/hide 可从当前值打断。 |
| Expanded | 页面稳定后才进入；无形变 rAF；只让当前可见内容持有刷新需求。 |
| PageTransition | 一个 pendingPage，快速切换只保留最新意图；旧/新内容完成统一协调。 |
| Collapsing | 外壳与封面逆向过渡；完成后再卸载 expanded 层。 |
| Hiding | 区分普通隐藏与捕获紧急隐藏；禁用交互、停止可见工作；最终保证整个轮廓隐藏。 |
| Hidden | 无视觉 rAF、FFT、图片处理或进度渲染，仅保留必要的唤醒/业务 deadline。 |
| Restoring | 获取最新状态，同步最终容器/region 后淡入；被再次隐藏时取消旧恢复。 |

`islandStore` 保留用户意图（expanded/page 等）作为一个入口；动画控制器管理过渡状态，不再让 App、Surface、native animation 各自产生“已完成”。transition/placement revision 继续保留，但由统一 owner 发放，过期完成回调无副作用。

**视觉与 Windows region/bounds 分离：**

| 管线 | 候选预算 | 正确性条件 |
|---|---|---|
| 视觉 | 目标 60 FPS；静止时零动画循环 | CSS transform/opacity 及受控 geometry；动画结束取消临时层/任务。 |
| Interaction Region | 起始/结束立即同步；变化期间先试 20 Hz，最多一个请求在途、一个 latest pending | 不能只同步最终值导致展开控件点不到，也不能大矩形 envelope 长期吞掉桌面点击；优先同步指针附近和关键交互阶段，最后 flush 终态。 |
| Native bounds | 普通展开/Page 切换保持 host；仅显示器/DPI/edge/隐藏/用户位置必要变化 | 主线程不阻塞；最终尺寸/位置与坐标变换一致。主窗口实际每阶段 resize 次数需计数。 |

## 4. UI Review 与内容层级

当前 Expanded 已经是“一个主页面 + 工具入口”，并没有大量同时展示的 Card。应保留这个方向。问题是默认七入口、播放器装饰性伪控件、重复浮窗入口、全页 zoom 和过多不同运动，增加了控制面板感。

### Compact：保持单一状态

| 元素 | 当前代码 | 建议约束 |
|---|---|---|
| 轮廓 | 默认 80×28、r14；可设长轴 80–300；Hover 90×30 | 保留默认紧凑轮廓；按内容预算扩展，避免为了功能数量持续加宽。 |
| 音乐 | 20×20 圆形封面 + 六条 Spectrum/播放小点；Compact 本来没有歌名/歌手 | 保留“封面 + 一个播放指示”；若新增歌名只允许短标题替代次要元素，歌手留 Expanded。 |
| 时间/天气 | 无媒体时同时显示 HH:mm、天气图标、温度；内层还有 gap/padding | 80 宽下需测实际字体宽度；容不下则主时钟 + 简短温度或单一天气，避免额外信息强行溢出。 |
| Timer | 活跃时强制长轴至少 240；侧边横向数字放入 28 宽 | 用 Timer 临时接管主状态，移除重复标签，检查小时级数字与竖向布局。不能偷偷改用户宽度后不解释。 |
| 内边距/间距 | Compact 左 4 右 8；Idle 又叠加 7 的 padding | 基于最终可见逻辑像素统一留白，检查所有 edge、语言、无封面与长内容。 |
| 动态 | Compact 唱片只转一次 8 秒；Spectrum 按需 | 不改成永久旋转；暂停/隐藏静止。 |

### Expanded：Primary/Secondary/Tools 分工

| 层级 | 内容 | 表现与预算 |
|---|---|---|
| Primary Content | 当前音乐，或用户明确选择的 Timer/天气/音量/时钟 | 一次一个任务页面，保留一个连续外壳；单一视觉焦点。 |
| Secondary Content | 歌手、进度/剩余时长、设备名、天气更新状态 | 最多一行/一组低对比辅助信息；无意义“等待”装饰不抢空间。 |
| Tool Rail | 高频 Timer、音量、常用工具 | 候选 3–4 个直接入口，剩余进入更多；用户可调整，但不把所有工具常驻为卡片。 |
| Navigation | 页内返回与一致 Escape | 一层返回，当前页有 active 状态；动作按钮（设置/隐藏/浮窗）不冒充选中页面。 |
| Status | 播放/暂停、Timer 完成、天气失败 | 轻量内联状态，仅重要完成事件短暂突出，避免额外状态 Card 常驻。 |
| Controls | 上一首/播放/下一首、必要音量/Timer 操作 | 删除不可操作的 ListMusic/Star/Shuffle 装饰；未支持能力禁用/隐藏；去掉重复浮窗入口。 |

**尺寸核查：** 主 App 的 expandedScale 范围 .3875–.625，Expanded 使用 CSS `zoom`。因此 84 的封面约为 32.55–52.5 CSS px，24 的歌名字号约 9.3–15，18 的歌手约 6.98–11.25，38×48 的按钮约 14.73×18.6 到 23.75×30。工具页 300 的基础宽约 116.25–187.5。以上为源码公式推导，并非截图测量；屏幕物理像素还需乘 DPI。建议用最终逻辑尺寸做排版，正文/操作命中尺寸不能跟着工作区再乘一个 .625。

hover 只给轻微底色/亮度；pressed 用短 scale/opacity；active 用低对比底色或图标颜色；focus-visible 必须清楚且不被 clip 裁掉。文字、图标、工具入口不要全部套相同装饰背景。轻量 UI 不等于压小字号与点击目标。

## 5. 内存：固有开销、额外成本与泄漏分开

| 内存域 | 固有/正常用途 | 本项目额外候选 | 如何区分 |
|---|---|---|---|
| Browser Process | WebView2 环境、配置、网络/缓存与进程协调 | 窗口/环境创建，活跃下载 | 按 Isle 对应进程树与 user-data 环境归属采集；不能把系统其他应用 WebView2 算入。 |
| Renderer Process | JS VM、DOM、字体、图像解码与 Canvas | Base64、多份封面解码、原尺寸 Canvas、保留旧 Canvas、多个窗口 | Renderer private bytes 与 JS heap 分别采；小 JS heap 不排除大 image/native allocation。 |
| GPU Process | composition、纹理、视频 decode surface | 透明 host、封面两层、Canvas backing、MV、持续 CSS animation | 对照关闭 Spectrum/MV/浮窗的 GPU engine/memory；共享 GPU process 时不能简单按窗口分摊。 |
| JS Heap | 有界状态/store/closure | rawCoverUrl/artworkUrl/lastPlayedMedia、LRU 值与大 key、未完成 Image/Promise | 比较 post-GC retained heap、retainer path；重复字符串是否共享由快照判断，不能按变量数盲目乘内存。 |
| DOM / Detached DOM | 当前页节点与有限过渡层 | newCanvasRef/oldCanvasRef 指向移出的 Canvas | cover→空→cover 快照检查 Detached Canvas 和 isConnected，确认 retainer 归属。 |
| Image Cache | 浏览器按资源缓存已解码图 | HD 1200 封面，原图+处理 PNG+旧图并存 | 不保证 `img.src=""` 或 revoke 会立即让 Working Set 下降；应看保留对象和稳定平台期。 |
| GPU Surface / Composition Layer | 静态透明窗口和必要动画 | 长驻 will-change、两张 Canvas、video、不必要 mask | ETW/DevTools layer/paint 验证，不按 CSS 属性直接宣布创建多少 layer。 |
| Rust / COM | SMTC/CPAL/缓存/网络缓冲 | PROPVARIANT、COM 初始化、整包下载、重复 image decode | 原生堆/分配 trace 与线程/句柄；WebView2 解释不了 Rust 进程内增长。 |

一张 1200×1200 RGBA 缓冲理论上约 **5.49 MiB**，只是单个像素缓冲的计算；原图、两张 Canvas、处理中图和 GPU 纹理可能并存，但不能据此宣称实测用了固定倍数。

### 封面策略判断

- `artworkUrl/rawCoverUrl` 同时保留 SMTC/HD 不一定泄漏，但身份需要 source + track + artVersion，避免跨播放器同名歌曲命中 `title|artist` 旧缓存。
- `localStorage` 只有一条 last-played 记录，data URI 超过 900,000 字符不持久化；存在同步写入和大字符串保留成本，没有“100 首记录无限增长”证据。后续优先持久化 track ID/缓存文件 key。
- HD resolver 缓存的是 URL/data URL；URL 字符预算不包含磁盘文件或 decode/GPU。浮窗 12 项 LRU 只有数量上限，还需要总字节/图片像素预算及短 key。
- `new Image()` 不是天然泄漏；CoverArt 已清除回调，Spectrum 取色有 5 秒 timeout，但后者 timeout 后未完整清回调/取消源。App 的 HD/recovery preload 没有统一 teardown；Floating 的 pendingImages 有 destroy 清理但没有逐个超时/并发限制。
- 推荐首先用 **Rust 限尺寸后的缓存文件 + asset URL**；缩略图尺寸按显示逻辑尺寸×DPI 限制，HD 仅大视图需要时请求。评估 WebP 与处理成本，透明度/质量正确后再切格式。
- Object URL 适合 webview 内临时 Blob，不作为跨窗口共享封面身份；必须在旧 DOM/动画不再使用后 revoke。当前 CSP 没有 `blob:`，不能直接替换后破坏图片显示；asset URL、原生路径、data URI 的解析接口也需统一。
- Floating 的 Rust 取色/图片函数接受 data URI/路径，前端 HD 返回的是 convertFileSrc URL：需显式传 cache key/path，而非把 asset URL 当本地文件名。正常主色回调也要校验 track/art revision。

### 生命周期审查结果

| 对象 | 当前已清理 | 剩余缺口/判断 |
|---|---|---|
| App onMount/listen | createAsyncCleanup、interval/DOM listener、capture/mouse cleanup | recovery/HD Image 没有统一取消；resume 600 ms 重试、手动隐藏等待、placement WAAPI 缺统一 owner。 |
| IslandSurface 六路订阅 | 所有 subscribe 都有 unsubscribe；matchMedia listener 有 cleanup | cover rAF/await tick 没有 teardown epoch/cancel；取消订阅不能替代取消待完成动画。 |
| Spectrum store/组件 | refcount、代次、watchdog clear、timer/rAF clear、Rust Destroyed release | 减少动画/系统偏好 gate 传递不一致；恢复策略需 native 健康状态，不能只重试已 running 的 start。 |
| mediaStore/settingsStore | 迟到 listener、初次 snapshot 竞态、disconnect/generation | mediaStore 相同快照仍写新对象；多 webview 有多个独立 store，并非全应用单例。 |
| EventManager | 独立 listener throttle/debounce 状态、off/destroy 清理 | await listen 后无 isDestroyed 二次检查。当前全局 destroy 没有主调用证据，标为潜在封装缺陷，而非已证实循环泄漏。 |
| Floating | DOM listener、Tauri listener、timer、AbortController、pendingImages、refs onDestroy | cover 变空时 Canvas refs 不清；部分请求只丢弃结果不取消；图像绘制/取色缺少最后结果检查。 |
| Timer/Studio/VisibleWindow | interval 清理、late listener 清理、真正隐藏时卸载 | capture CSS hide 不触发卸载；Timer idle 条纹无限动画；预览 clock 1 Hz 对 HH:mm 过频。 |
| RollingDigit/Marquee | matchMedia cleanup、ResizeObserver disconnect、WAAPI cancel | RollingDigit 双 rAF 未取消但有界；Marquee 在持续布局变化下可能重启，应测并合并测量。 |
| 程序入口 contextmenu | 与 document 同生命周期 | 不是每次组件挂载累计的 listener，保持低优先级。 |

## 6. 所有周期性/动画任务清单

成本 L/M/H 是相对成本候选；H 不代表测到高 CPU。频率来自源码，按请求耗时/节流/可见性会降低。

| 模块 | 当前刷新频率/条件 | 是否必要 | CPU 成本 | IPC 成本 | UI 成本 | 优化建议 |
|---|---|---|---|---|---|---|
| Rust SMTC | 1 s，后台线程一直查询；无会话仅停止重复空事件 | 状态变化必要，Idle 逐秒全量查询不足以合理化 | M：RequestAsync、会话枚举、属性读取/字符串 | 有会话约 1 event/s，播放控制另发 | M：主/浮/Studio 更新快照 | 复用 manager，订阅变更；低频兜底，播放时间前端投影 |
| App 音乐 clock | 可见 Expanded/music/playing 250 ms；此处未判 isHidden | 仅当前可见进度 | L–M；同时 updateTimeDisplay/Intl | 0；另有媒体同步事件 | 4 Hz 进度；分钟时钟重复计算 | 区分进度与分钟时钟，capture/offscreen 停止视觉 clock |
| App idle clock | 通常 60 s，分钟对齐递归 timeout | 必要 | L | 0 | 仅分钟变化 | 保留，恢复时补一次；非可见时不绘制 |
| App Timer | running 时至少 1 s；若音乐 clock 4 Hz 则同 clock 更新 | deadline/完成必要 | L | Timer 状态按秒去重，约 1/s | 秒值和 compact progress | 事件传 deadline，隐藏只保留完成调度；不要停止真实倒计时 |
| TimerWindow clock | visible + running 时 250 ms | 秒数字一般不需 4 Hz | L–M | 0，本地投影 | 4 Hz 派生，数字实际按秒变化 | 秒边界更新；仅确需平滑线条时局部高频 |
| Floating clock | !pageVisible 2 s；playing 非 Compact 250 ms；否则 1 s | 即使无 hover/捕获隐藏仍更新，过频 | L–M | 0 | 进度只在相应 UI 可见才必要 | 使用 isCaptureHidden/控件可见性；空闲零进度 timer |
| App metadata recovery | interval 5 s；按 pending/backoff 最短 5→60 s | 仅缺 duration/art 时 | L 短路，网络失败时 M | 条件 get_netease_song_info | 结果改变时 | 缺失时才建立有截止期重试任务；正常媒体不驻留 timer |
| Resume last track | 最长 12 s、600 ms 轮询；用户触发 | 有界必要 | M：SMTC snapshot | ≤约 1.67 invoke/s，仅恢复中 | 最后结果 | 取消/销毁可中断；事件完成优先 |
| App idle/weather/system | interval 30 s + 首次请求；pageVisible 且 idle 或天气页 | 天气不需 30 s；隐藏不需 sysinfo | M：CPU/memory/network 刷新 | ≤1 invoke/30 s；前端另有 30 s 缓存 | 结果对象/加载标志变化 | 分离天气与系统指标，只取当前展示字段 |
| Studio idle/weather | 挂载后 30 s；VisibleWindow 隐藏会卸载 | 天气预览必要，系统指标不一定 | M | ≤1/30 s/窗口 | 预览刷新 | 天气共享后端 TTL，需求驱动 |
| Weather 网络 | 后端同坐标 TTL 1800 s、请求锁 | 必要 | L–M 网络/JSON | 通过 snapshot 返回 | 变化后 | 保留共享与 TTL；失败退避、地理查询加 timeout |
| Studio clock | 挂载时 1 s | 当前 HH:mm 不需每秒 | L，重复格式化 | 0 | 1 Hz 预览 | minute 对齐，实际秒展示才 1 Hz |
| Studio 媒体会话 | 10 s + 打开/用户动作 | 打开播放器选择区域时必要 | M：枚举/COM | ≤0.1 invoke/s | 列表对象更换 | 仅选项可见/会话变更重取；避免初次重复 snapshot |
| mediaStore fallback | 仅 listen 失败时 2500 ms | 兜底必要，不能称正常双重 polling | M | ≤0.4 invoke/s/消费者窗口 | 快照刷新 | 恢复 listener 后停止兜底；只发布有意义变化 |
| App work area | interval 30 s；monitor 数据请求缓存 5 s | 低频兜底可保留但优先系统事件 | M：显示器枚举 | ≤1 get_monitors/30 s，bounds 有签名去重 | 大多 0，可能重复请求派生 | 显示器/工作区/DPI 事件触发；可见恢复补采 |
| Rust screenshot keys | 50 ms，20 Hz，设置关闭也运行 | 设置开启时需检测 | L/次，持续唤醒 | 仅变化+捕获心跳 | 状态 effect | 关闭就停止；评估轻量原生事件/快捷键机制，覆盖 Win+Shift+S/PrintScreen |
| Rust fullscreen | 500 ms，2 Hz，无设置 gate | 设置开启且相关显示器时 | L–M WinAPI | 合并 Capture | 隐藏/恢复 | 前台/窗口事件 + 低频校验；多显示器定义明确 |
| Rust AppCapture | 500 ms，2 Hz，无设置 gate | 当前 view 支持时有用 | L–M，失败重复 | 合并 Capture | 捕获变化 | 事件式/能力探测后退避；GetForCurrentView 不是通用录屏检测 |
| Capture heartbeat | 2 s，重复相同 snapshot | 为晚加入窗口设计，长期重复可替代 | L：比较/锁/序列化 | 0.5 event/s 广播 | 新对象/回调 | 新消费者 initial snapshot，变更 emit；screenShare 当前始终 false |
| Rust cursor hit test | inside 33 ms/near 50 ms/far 200 ms；错误重试另有间隔 | 点击穿透必要 | L–M：WinAPI、72 点、每次 scale_factor | 无 JS invoke；native cursor flag 只变更时设置 | 0 JS；改变 hover/穿透 | 缓存 DPI，远处更低频/事件触发，保留近处响应 |
| Cursor watchdog | 每 1 s，3 s stale 重启 generation | 故障恢复有用 | L；主线程阻塞时可能触发重复代次 | 0 JS | 0 | 先排除主线程阻塞；恢复去重，线程数回稳 |
| App mouse wrapper | 每 DOM mousemove 清/建 100 ms timeout，是 debounce | 全屏唤醒需；关闭全屏偏好仍创建 timer | L–M 高频分配 | handle 内条件 fullscreen invoke | 无常驻 UI | gate 放在分配前；不要误称固定 10 Hz throttle |
| Shape springs/region | 仅 active transition，接近刷新率 | 视觉必要；每帧 IPC 不必要 | M：派生/72 点/JSON | 约视觉帧率 invoke，需实测合并程度 | layout/clip/opacity | 分离两条时钟，latest + settled flush |
| Cover motion | 展开 250 ms rAF，每帧 measure/layout write | 连续封面必要，实现可优化 | M–H，强制布局候选 | 0 直接，壳另外更新 region | layout/paint | FLIP/transform，固定目标推导，清理与反向轨迹 |
| Marquee | overflow 时 WAAPI 无限；ResizeObserver 可随 shell 变化 | 长标题且可见时必要 | L–M | 0 | transform 合成/测量 | transition 后再启用，隐藏暂停，reduced motion 静态 |
| Debug FPS | showDebugInfo 时连续 rAF | 仅诊断 | L 持续 | 0 | debug 文本 1 s | 默认关闭且隐藏暂停；rAF FPS 不是实际呈现 FPS |
| Spectrum capture | 仅消费者存在；audio sample rate 输入，2048 样本/FFT | playing+enabled+visible 时必要 | M：CPAL/WASAPI+FFT；全零跳过 FFT | 回调不 IPC | 0 | 保留按需；统一 reduce/visibility gate，失败恢复有限退避 |
| Spectrum publisher | 50 ms 检查，变化时≤20 Hz，稳定 1 Hz 心跳 | 活跃可视化必要 | L–M 锁/序列化 | app.emit 广播 spectrum-data | 30 Hz 绘制 | 目标窗口/消费者；不要盲目提高至 60 Hz |
| Spectrum render/random | capped ~30 Hz，静止像素不 draw；random retarget 125 ms | active playing 才必要 | L–M | random 模式 0 native capture | Canvas draw | 保留 cap/dirty；reduce 时静止值 |
| Spectrum watchdog/retry | active 1 s；stale>2.5 s 重附/start；listener retry 1 s | 仅故障时 | L；长期无设备重试可能持续 | start/relisten 条件触发 | 清零 | native health 与 generation 配合；无消费者零 watchdog |
| Native bounds animation | active 时线程约 16 ms SetWindowPos；副窗260 ms | 仅放置/隐藏等必要 | M–H 待 ETW | 仅起始 invoke，线程内调用不是每帧 JS IPC | native composition | 纯 move 不重复尺寸，修 dt/最终落点；不误记为 region resize |
| Floating circular art | playing 时 20 s CSS infinite；capture CSS hide 未暂停 | 视觉可选 | JS 低，GPU 持续候选 | 0 | transform 合成 | 隐藏停止，静态/有限旋转默认更省电 |
| Floating MV | 播放中 video decode/timeupdate/30 s preview loop | 仅用户启用且可见必要 | CPU/GPU M–H | 换歌网络/缓存命令 | video surface | 捕获/隐藏暂停；旧请求取消，明确下载的是完整文件 |
| Timer idle stripes | 未运行选择界面 `.stripe-layer` 820 ms CSS infinite | Idle 装饰没有业务必要 | paint/GPU 待测 | 0 | background-position/transform | 改静态高亮或仅交互期间短动画 |
| CoverArt crossfade/RollingDigit | 220 ms fade cleanup/transitionend 双 rAF | 有限必要 | L | 0 | opacity/transform | 保留有限动画，显式 cancellation 与低动效一致 |
| 设置/位置写入 | Studio debounce300 ms；浮窗 move/resize debounce500 ms | 事件驱动必要 | 文件 I/O M | 用户活动时 | 0/应用反馈 | 后端全局串行/原子提交；不作为永久轮询 |
| Cache metadata | 访问时最多约2 s持久化；不是驻留2 s interval | 有限必要 | 文件 I/O M | 缓存访问命令 | 0 | 合并写入，不把磁盘元数据刷新当后台轮询 |

## 7. IPC 审查清单

同一事件每秒发送一次，可能被多个 WebView 接收多次。必须分别统计 **native emit 次数、收件窗口数、bytes、每个 listener 回调次数**；不能仅看一个 EventBus 统计作为全部 IPC。

| IPC | 来源 | 当前频率 | 必要性 | 建议 |
|---|---|---|---|---|
| `media-update` | Rust 1 s listener；control_media play_pause 后另外 emit | 有会话约1/s+操作；空会话确认后抑制重复 | 元数据/播放/能力变化必要；暂停重复不是必要 | metadata/state/progress 分层；保留时间校准、完整初始 snapshot；不要每秒 clone 大 art 再 strip |
| `island-media-sync` | App syncFloatingMediaClock | 每次媒体处理与补全，即使没有浮窗也 emit | 主岛权威位置/HD补全可能需要 | 有消费者才发送；浮窗 late join snapshot；同歌 artwork delta 也应用；取消双权威重复解析 |
| `spectrum-data` | audio.rs publisher 的直接 `app.emit` | ≤20/s，稳定约1/s，仅 capture 运行 | 可见 Spectrum 必要 | 维持批量六 bars；目标消费者与字节统计；当前绕过 EventBus |
| `start/stop/restart_spectrum` | spectrumStore / Rust label consumers | 可见/播放状态变化；watchdog 故障重试 | 必要 | 幂等、在途有界、代次一致；不要形变每帧触发 |
| `set_island_interaction_region` | IslandSurface→App→window API | 动画中近帧率；静止签名去重 | 最终命中必要，逐帧通常可降 | 在途1+latest1，候选20Hz、量化/容差、终态必 flush、失败不提前确认 signature |
| `animate_window_bounds` | App placement / hide / restore | 状态/设置/30 s兜底，bounds签名相同时跳过 | native放置必要 | 事务去重；普通展开/Page不要resize；约16ms SetWindowPos是内部动作另计 |
| `get_monitors` | App placement / Studio启动等 | App缓存5s；工作区任务30s | 变化/启动/恢复必要 | 原生变化事件 + 兜底，统一 logical/physical 坐标 |
| `check_fullscreen_app` | App mouse唤醒路径 | 条件性100ms debounce之后，不是永久固定10Hz | 与Rust捕获监控重叠 | 共享原生快照；唤醒按当前显示器判断 |
| `capture-mode-changed` | Rust capture monitor | 变化+2s heartbeat | 变化必要 | 新窗口主动读取snapshot；关闭检测时零对应工作 |
| `get_idle_snapshot` | App/Studio idleApi | 每挂载消费者≤1/30s，JS同webview缓存/请求合并 | 天气必要，未显示系统指标不必要 | 分离weather/system，字段变化后才更新 UI |
| `timer-state-changed` | App倒计时事件桥 | running秒键去重约1/s，动作另发 | 可见Timer显示可本地投影 | 传deadline+status/revision，动作/完成必达；无需显示的秒广播停掉 |
| Timer request/action | TimerWindow→主 App | 打开/用户动作 | 必要 | 明确权威与revision，暂停/恢复不能丢失；重挂载读取当前 snapshot |
| `get_media_info_cmd` | 浮窗初始、mediaStore、resume等 | 初始/恢复；listener失败兜底2.5s | 初始必要 | 共享snapshot，避免重复COM查询；正确初始化worker COM |
| `resolve_hd_cover` / `get_netease_song_info_cmd` | App+Floating补全/高清/时长/MV | 每track/缺失重试，可能并发 | 按需 | 单航班、track token、metadata查询不强制下载图片 |
| `process_image` / `extract_dominant_color` | 浮窗 cover变化 | 每图/设置改变 | 仅启用效果/颜色需要 | 缓存key、限尺寸、blocking pool、有界请求、过期结果不绘制 |
| `get_system_audio_state/list_audio_output_devices` | 进入音量页/切设备 | 按需，不是永久poll | 必要 | 保留按需；COM资源清理、设备变更事件 |
| `set_system_volume` | pointer/keyboard，App已串行保留latest | 输入驱动，暂无Hz预算 | 必要 | 相同整数跳过，候选30Hz在途1+latest1，pointerup终值必送；move时不重复设备枚举/激活 |
| `save/update_settings` + settings事件 | Studio/浮窗/主 App | debounce300/500ms及用户动作；全量+legacy多个广播 | 必要 | 后端单提交队列，revision；只广播一次权威提交，检查legacy依赖再删除 |
| download/cache/window show/close | 用户动作、换歌、窗口生命周期 | 非周期 | 必要 | 在途预算、cancel与销毁对称；浮窗create缺少Studio/Timer已有的创建锁 |

### Idle 模式目标

Idle 定义：主岛 Compact、无播放/无运行 Timer、Spectrum 无消费者、副窗关闭、debug关闭、桌面无交互。额外分别测试“所有 Capture 关闭”和“默认 Capture 开启”，不能用关闭必要功能的单一场景宣称默认场景已达标。

目标是整个 Isle 进程组 CPU 接近0%、GPU接近桌面空闲基线、内存进入稳定平台。零动作时无视觉 rAF、FFT、图片处理、进度timer或region IPC。允许分钟时钟、天气到期刷新和低频容错；真实 Timer/捕获唤醒需求不能为零CPU而失效。鼠标近岛和实际播放不属于纯Idle。

Spectrum目标保持 `Music Playing && Spectrum Enabled && Spectrum Visible`；多个窗口按消费者OR聚合，无任何消费者后350ms宽限结束，capture/FFT/publisher停。`reduceAnimations`、全局动画开关和系统 reduced-motion 的产品语义须统一，当前传给Spectrum的 reduceMotion 只含 reduceAnimations。

## 8. App.svelte 架构判断

App（1699行）已是协调多个服务的 God Component，但 Svelte5 细粒度更新不会因它长就自动全量rerender。先以 effect 执行计数与 trace 证实依赖扇出，按下面边界逐个提取；不为了文件更短创造10个互相订阅的controller。

| 候选边界 | 实际收益/所有权 | 顺序 |
|---|---|---|
| IslandAnimationController | state/epoch/完成/中断统一，Surface仅消费frame | Phase4 必需，可先保持同文件接口，验证后提取 |
| PlacementController | host、DPI、monitor缓存、hide位移、native事务，取消plain suppressPlacementEffect竞争 | Phase3正确性→Phase4协同→Phase6提取 |
| MediaController | 一个track身份/clock/封面resolver、浮窗snapshot、在途预算 | Phase1/3修正确性，Phase6依据profile提取 |
| AudioController | 音量latest队列、设备refresh、COM后台actor接口 | Phase2/3；原有串行latest逻辑保留 |
| WeatherController | weather TTL 与system需求分离、城市request revision | Phase2；不再因30s天气触发CPU/network采集 |
| TimerController | 保留绝对deadline权威、完成事件、跨窗动作 | 只有更新范围/复用受益才Phase6拆，不能拆出两份倒计时 |
| WindowController/IslandController | 若仅转发API/订阅全部store没有收益 | 暂不创建；先确定与Placement/Animation的唯一边界 |
| PerformanceController | 低开销计数/诊断生命周期 | Phase0临时或单独profiling build；正式默认关闭 |

提取验收是 effect/重复计算或耦合可证明下降、profiling更清楚、测试不退化。行数下降本身不计为性能收益。

## 9. Current Baseline 与 Benchmark 规范

**当前 Windows Baseline：全部待采集。** 不能将仓库旧Chrome报告写成此提交的优化前数据。

| 项目 | 当前可确认 | 待采集内容 |
|---|---|---|
| `docs/performance/before.json`、`docs/performance/after.json` | 60s Chrome headless fixture、synthetic Spectrum；包含TaskDuration/heap/DOM/listener | 当前提交native的WS、private bytes、GPU/FFT/IPC、长时间增长；旧结果不是同条件改善证明（部分scene CPU反而上升） |
| `live-spectrum-2026-09-30.json` | 60s模拟场景约0.63% renderer TaskDuration CPU | 非native capture；不能等同整机CPU或默认应用CPU |
| `retained-memory.json` | 30次fixture循环后Nodes/listeners稳定，post-GC heap增量359,896 bytes（约351.5KiB） | 没有浮窗cover空态/100首/8小时/native对象；不足以排除本审查问题 |
| `scripts/measure-native.ps1` | 能区分app/browser/renderer等并采CPU/WS/privateWS/privateBytes | 限1–60s，进程列表仅开始采一次，无GPU/JSheap/threads/handles；长测不能直接使用原参数完成 |
| Playwright入口 | 配置请求 `npm run web:dev`，package.json不存在此script | 先恢复可启动入口；旧断言仍有300px/200px布局，需对照600宽导航语义核实 |
| 动画自动测试 | 全局 reducedMotion=reduce | 增加 no-preference 与中途打断/不同DPI native试验；静态布局pass不等于60FPS |
| devtools | tauri.conf主窗devtools=false | profiler专用release构建/允许的WebView2调试通道；正式发行默认配置不能混入调试负担 |

### 统一测量口径

| 指标 | 记录方式/口径 |
|---|---|
| Working Set | app与每个Isle WebView2 PID曲线；总WS仅作辅助，共享页不可盲目累计成唯一物理占用 |
| Private Working Set | 每PID及总私有驻留；与任务管理器字段对齐 |
| Commit Size | 每PID private commit/private bytes对应的已提交内存，注明工具口径；不要把WS当Commit |
| JS Heap | 每个webview used/total heap、post-GC retained heap与allocation/retainer snapshot；不在正式8h样本频繁强制GC |
| Renderer Memory | PID角色、privateWS/privateBytes、图像/DOM/Canvas快照；一个窗口不保证独占一个renderer |
| GPU Memory | PID/适用进程组 dedicated/shared GPU usage，GPU engine/video decode，DWM单独记录；无法分配到单窗时注明 |
| CPU % | CPU秒差/壁钟时间/逻辑核数为整机百分比；同时报单核等价CPU防止多核归一掩盖成本；平均/p95/峰值 |
| Thread / Handle Count | 每PID采样，设备重连/窗开关后回到稳定范围；不能仅看总内存 |
| FPS / Frame Time | 实际呈现/trace帧时，不只rAF计数；60Hz预算16.67ms，报告p50/p95/p99、missed-vsync率；120/144Hz注明预算 |
| Long Task | 每webview >50ms task数量、最长时长、分配/GC/layout/IPC归因，动画时间段独立统计 |
| IPC / second | 所有invoke、native emit、前端emit及接收次数/bytes/p95耗时/在途峰值；包含绕过EventBus的spectrum-data |
| 辅助指标 | Layout/RecalcStyle/Paint、decode次数、FFT次数、native SetWindowPos/resize次数、listener/DOM计数、请求并发/缓存字节 |

Windows release 构建；固定机器、Windows/WebView2版本、CPU/GPU、逻辑核、显示器与DPI、刷新率、电源模式、窗口状态、歌曲/封面语料、网络/缓存冷热。每次3轮，warm-up5分钟，先测桌面控制组，再测应用；Windows进程树采样1s，长测按块落盘且动态发现新PID，不能固定最初进程列表。Profiler挂接用于独立诊断轮，避免把开着DevTools的内存当正式基线。

| Benchmark | 可复现场景 | 重点与对照 |
|---|---|---|
| A | 启动后不操作10分钟，Compact、无音乐/Timer/副窗 | 默认Capture vs 全关；有天气城市 vs 无城市；无持续rAF/FFT/region IPC，CPU/GPU平台期 |
| B | 固定播放器与歌曲播放30分钟 | Compact/Expanded、Spectrum开关、浮窗有无、MV开关分轮；时间准确性、持续CPU与内存 |
| C | 连续切100首不同封面的歌，包含封面→空→封面、source变化与网络错误 | 每25首采Renderer/JSheap/GPU；完成后静置5分钟，另做post-GC诊断；缓存/在途数有界、无错图/Detached Canvas |
| D | 展开/收起500次 | 正常节奏350次、100次80–150ms打断、50次混合Page/style/隐藏；四edge/DPI分轮；frame/IPC/最终命中/闪烁 |
| E | 真实WASAPI Spectrum运行30分钟 | 音乐→暂停→隐藏→显示→切设备→无设备→恢复；capture/FFT/event启停，静音与非零输入；random单独对照 |
| F | 后台运行8小时 | 纯Idle为主，规定时点播放/切歌/Timer/副窗/锁屏恢复/热插拔；每小时快照；内存斜率、线程句柄、恢复正确性 |

每个Benchmark均记录上表全部核心指标；静止场景没有动画时 FPS 标 `N/A`，记录是否仍在产生帧；停止capture后FFT标0，不用无数据冒充0。结果保留原始时间序列、进程角色/样本环境与对照，不只截图任务管理器。

## 10. 执行顺序与阶段门禁

**Phase 0 Performance Baseline → Phase 1 Memory / Leak → Phase 2 CPU / Background Tasks → Phase 3 IPC → Phase 4 Animation Engine → Phase 5 UI → Phase 6 Architecture Cleanup → Phase 7 Long-running Stability Test。**

优先级表示严重程度，Phase表示风险控制顺序，两者不是同一编号。后续不能把UI、动画引擎和后台架构放进同一批重构。

每次执行：确认本阶段范围 → 修改一个可审核任务/一组直接依赖任务 → 运行相关测试 → 相同条件前后数据对比 → 检查回归 → 在本文件记录结果并将对应`[ ]`改`[x]` → 提交该阶段 → 停止，等待下一阶段确认。测试失败或没有基线时不勾选，当前审查任务不产生优化提交。

## 11. 可逐项执行的任务

### Phase 0 — Performance Baseline

- [ ] **B0 · 🟠 P1 — Record current Windows performance baseline**
  - **Problem:** 当前提交缺少native Idle CPU/内存、WebView2进程、GPU、动画帧与全部IPC的同条件基线；已有Chrome结果不覆盖这些开销。
  - **File:** `scripts/measure-native.ps1`、`scripts/measure-performance.mjs`、`scripts/measure-retained-memory.mjs`、`src/App.svelte:1191–1215`、`src-tauri/src/event_bus.rs`、`src-tauri/tauri.conf.json`、`docs/performance/`。
  - **Reason:** 无基线无法判断优化收益、工作集波动与真实泄漏，也无法验证“接近0%”和“60 FPS”。
  - **Proposed change:** 经确认后建立专用profiling构建与动态PID采集，补GPU/线程/句柄/IPC bytes/在途数；采第9节全部指标。先执行A、C、D的短复现与资源定位，再保留完整A–F的优化前运行；新指标默认不进入发行版热路径。
  - **Expected benefit:** 得到可比较的Idle、音乐、窗口/封面操作成本与内存归属；明确最先修的可达缺陷。
  - **Risk:** DevTools/trace/频繁snapshot改变结果；CPU归一、共享GPU/WS可能误计。需独立无profiler正式轮和带profiler诊断轮。
  - **Verification:** 3轮同环境、warm-up5分钟，记录所有PID/版本/DPI/歌曲语料。Idle CPU、Private WS、Commit、Renderer/JS heap、GPU和FPS条目均有数据或明确N/A；capture停止时FFT/IPC应可直接核实。当前任务尚未执行。

- [x] **B1 · 🟠 P1 — Restore a usable verification entry and audit existing assertions**
  - **Problem:** Playwright `web:dev`脚本不存在；默认reduced motion绕过真实动画；旧布局断言与当前导航结构可能不一致。
  - **File:** `package.json`、`playwright.config.ts:13–20`、`ui-tests/island-preview.pw.ts:325–382`、`ui-tests/island-navigation.pw.ts`、`ui-tests/island-fixture.svelte`、`scripts/build-performance.mjs`。
  - **Reason:** 后续“运行测试”必须真的能启动且检查所需行为，不能用失效入口或把新期望值机械替换来获得pass。
  - **Proposed change:** 经确认后统一真实Vite启动命令/fixture构建入口；核实300px/200px等断言原意，按产品布局契约修正；保留reduce测试，另加no-preference动画和打断场景；选定一个权威JS lockfile与安装方式。
  - **Expected benefit:** 每阶段有可重复的TS/Svelte/纯函数/UI回归检查入口。
  - **Risk:** 更新断言掩盖布局回归，浏览器fixture仍不含native命中/性能。
  - **Verification:** Windows执行`npm ci`（若选npm锁）、`npm run check`、`npm test`、`npm run test:ui`、`npm run build`；Rust执行适用的`cargo test`及`cargo check`。列明任何历史失败与覆盖盲区，不将fixture pass替代native Benchmark。本次未安装依赖/运行这些构建测试，避免新增业务/构建产物。

### Phase 1 — Memory / Leak / Stability

- [x] **M1 · 🔴 P0 — Release PROPVARIANT and balance COM ownership**
  - **Problem:** `device_name`返回后PROPVARIANT的字符串资源没有清理；媒体会话/控制调用重复CoInitializeEx且忽略返回值，没有配对释放。worker媒体读取路径的线程初始化策略也不一致。
  - **File:** `src-tauri/src/services/system_audio.rs:33–49`、`src-tauri/src/services/media.rs:644–650,957–965`、`src-tauri/src/commands/media.rs:12–58`、`src-tauri/src/lib.rs:179–189`。
  - **Reason:** windows 0.52 PROPVARIANT是原始生成结构，没有自动Drop；成功COM初始化包括S_FALSE也需配对。长寿命线程与短任务必须有各自明确owner。
  - **Proposed change:** 对GetValue成功后的所有return路径建立不复制原始所有权的清理guard；引入COM/WinRT线程初始化策略（固定媒体actor或任务RAII），检查初始化失败/模式冲突；先释放接口再释放线程apartment，避免直接套CoUninitialize影响Tauri既有apartment。
  - **Expected benefit:** 消除确定的设备属性资源泄漏和初始化不平衡，减少长期原生资源风险。
  - **Risk:** 双重PropVariantClear、Clone原始指针、提前CoUninitialize或STA/MTA错用会造成更严重问题。
  - **Verification:** 连续1000次设备名称/列表查询与打开关闭音量页，比较native retained allocations/private commit、句柄与线程；覆盖PropVariantToString错误路径。验证媒体控制/会话读取仍正常，COM成功/失败路径配对计数一致。记录前后数据，实际泄漏字节不能只靠WS波动判断。

- [x] **M2 · 🟠 P1 — Give Canvas and image rendering explicit DOM ownership**
  - **2026-10-09 验收状态：** 明确区分封面字段省略与清空，接收同曲目迟到/替换封面，保护较新事件不被旧快照覆盖。5 次延迟快照重开、100 首真实切歌及四次关闭重开、真实无会话恢复通过，原生堆快照 Detached Canvas=0；79 项前端、9 项 UI、51 项 Rust 通过（1 项忽略）。本次勾选限于封面正确性/Canvas 所有权；诊断 GPU 仍波动，Release 性能目标不据此勾选。见 [M2 验收报告](performance/m2-reopen-2026-10-08/ACCEPTANCE.md)。
  - **Problem:** 浮窗缓存的newCanvasRef/oldCanvasRef在条件DOM销毁后仍保留；后续仅在null时赋新ref；异步Image.onload可以把旧歌画到当前Canvas。
  - **File:** `src/FloatingWindow.svelte:886–945,995–1028,1066–1081,1313–1330,1532–1598`。
  - **Reason:** 会导致Detached Canvas保留与封面空白/错图；这是具体owner缺陷，不能仅清LRU解决。
  - **Proposed change:** 改用绑定实际Canvas的生命周期（如bind:this）并在端点退出后清/缩减backing store；每次图像绘制带track/art/render epoch、canvas identity/isConnected检查；cover变空立即取消当前绘制。旧图仅在有界crossfade期间保留。
  - **Expected benefit:** 新DOM正确显示封面，旧回调不能覆盖，Canvas backing资源可回收。
  - **Risk:** 过早释放旧Canvas导致crossfade闪烁；修改需保留cover→cover动画。
  - **Verification:** cover→空会话→cover、cover→无图曲目→cover循环100次；故意延迟旧Image.onload，最终封面始终最新。heap快照检索Detached Canvas及retainer，关闭窗口后listener/ref回基线；比较Renderer privateBytes/GPU与回稳时间。

- [ ] **M3 · 🔴 P0 — Bound input, decode and cache bytes; reject crash inputs**
  - **2026-10-08 实施状态：** 已落实图片/下载/解码/并发/缓存/Canvas 预算及错误输入校验，50 项 Rust、73 项前端、9 项 UI 及实际原生命令验证通过。完整同条件 Release C 对照未完成，暂不勾选。见 [M3 实施记录](performance/m3-2026-10-08/IMPLEMENTATION.md)。
  - **2026-10-09 Release C 对照：** 相同 M2 修复、固定 100 首、连续浮窗、前后各三轮正式采样及五分钟恢复已完成。三轮中位数：GPU 峰值 −54.88%，但进程树 Private Commit 峰值 +24.17%、Renderer 峰值 +54.55%、CPU 平均值 +43.54%；恢复末分钟 Renderer Commit +70.42%。下载缓冲/Canvas backing 下降不能代替整体内存收益验收，M3 保持 `[ ]`；GPU 长期回稳仍未证实。见 [完整对照报告](performance/m3-release-c-2026-10-09/REPORT.md)。
  - **Problem:** SMTC按stream size分配且u64转u32未验证；HTTP缺Content-Length时整包读取后才超限拒绝；原图尺寸解码/Canvas和数量LRU没有项目字节预算；pixel_size=0进入step_by(0)会panic，release abort。
  - **File:** `src-tauri/src/services/media.rs:133–170,732–763`、`src-tauri/src/services/cache.rs:287–337`、`src-tauri/src/services/image.rs:19–37,47–77,130–145`、`src-tauri/src/utils.rs`、`src/FloatingWindow.svelte:948–1028,1066–1081`、`src-tauri/Cargo.toml`。
  - **Reason:** 现有磁盘/条目上限不能控制下载在途、原始decode、Base64和Canvas峰值；一个可达command错误输入不应终止应用。
  - **Proposed change:** 分块下载至有上限临时文件并在读取中止损，限制SMTC字节/转换溢出、图片像素/尺寸与处理并发；按可见尺寸×DPI制作缓存缩略图。补总字节LRU、短key与处理结果预算；pixel_size校验范围并返回AppError，查大值求和溢出。避免process_image像素化分支重复完整decode。
  - **Expected benefit:** 低峰值、更少JS/GPU backing、错误输入不crash；100首场景可进入明确有界平台。
  - **Risk:** 太低上限损害封面清晰度/合法MV；WebP改变画质/处理时间；format检测与SMTC当前固定PNG MIME需一并核实。
  - **Verification:** chunked无Content-Length超限响应、截断图片、异常像素尺寸、SMTC超限、pixel_size=0/极大值均可控错误；正常1×/1.5×/2×DPI封面清晰。用C对比下载并发、decode内存峰值、Renderer/GPU；不能仅看磁盘配额。

- [ ] **M4 · 🟠 P1 — Coalesce artwork work and cancel superseded requests**
  - **Problem:** App/浮窗各自高清解析；duration/MV搜索也下载图片；快速切歌旧Rust下载仍完成，process_image请求Map只按同key合并、无总并发上限；浮窗主色无revision检查；同歌分支忽略新albumArt。
  - **File:** `src/App.svelte:87–121,965–968,1512–1534`、`src/FloatingWindow.svelte:169–238,329–349,782–814,948–994`、`src-tauri/src/services/media.rs:381–430,891–913`、`src-tauri/src/services/cache.rs:287–337`、`src-tauri/src/utils.rs:27–81`。
  - **Reason:** 迟到结果丢弃并不取消下载/decode，双窗口重复请求放大CPU、内存与网络；asset URL与Rust path接口不匹配导致重复失败。
  - **Proposed change:** 后端按source/track/artVersion做single-flight，元数据查询与图像下载分开，前端只订阅权威结果；请求带cancel token或显式过期检查/全局并发预算，旧歌/窗口关闭不继续昂贵处理。规范cache key→path/asset URL接口；主色和绘制结果检查revision；同歌允许artVersion更新。
  - **Expected benefit:** 快速100首切换时工作有界，不错图/错色，不为拿时长或MV ID重复下载封面。
  - **Risk:** 取消共有single-flight时误伤另一窗口；缓存key缺source会串歌；过度取消在弱网络下反复失败。
  - **Verification:** 双窗口同时请求同歌计数只一次；同名不同source不混；故意让第1首最后返回，第100首始终最终结果。关闭窗口后在途任务有界退出；C记录network/decode/IPC次数与private commit。正常HD升级无需重新切歌即可显示。

- [ ] **M5 · 🟠 P1 — Close late-listener, Image and animation teardown gaps**
  - **Problem:** EventManager await listen之后未复查destroy；Surface cover rAF/await tick无teardown cancellation；App若销毁，预加载Image/恢复等待/placement Animation缺统一取消；部分图像超时只reject未清handler。
  - **File:** `src/utils/eventManager.ts:30–64,104–115`、`src/lib/asyncCleanup.ts`、`src/lib/components/island/IslandSurface.svelte:333–438`、`src/App.svelte:87–121,387–419,802–865,1038–1064,1512–1534`、`src/lib/spectrumColors.ts:15–31`、`src/FloatingWindow.svelte:479–489`、`src/lib/components/time/RollingDigit.svelte:41–48`。
  - **Reason:** 多数cleanup已有实现，应补具体缺口。EventManager缺陷目前是潜在封装竞态；有界双rAF本身不证明长期泄漏。
  - **Proposed change:** async注册返回后若owner已销毁立即unlisten；统一dispose/epoch、取消rAF/WAAPI/timeout、图像handler与source，设置逐图超时与并发限制；复用createAsyncCleanup，不新增另一个全局资源注册器。
  - **Expected benefit:** 卸载后不会继续更新状态或保留DOM/closure，开关窗和中断动画可稳定回收。
  - **Risk:** 把document生命周期listener误当组件泄漏；销毁前强制取消共有任务使其他窗口失去状态。
  - **Verification:** mock延迟listen返回，在destroy之后unlisten调用一次且getListenerCount为0；动画中卸载后无额外frame写入；图像超时后pending集合归零；真实副窗100次开关比较DOM/listener/线程/句柄post-warmup基线。

- [ ] **M6 · 🟠 P1 — Serialize settings commits and make persistence recoverable**
  - **Problem:** update_settings读snapshot后跨await保存，多窗口可覆盖彼此patch；save先改内存再spawn_blocking写文件，写入顺序可反转；fs::write直接覆盖，不是原子提交；Studio吞掉保存错误。
  - **File:** `src-tauri/src/commands/settings.rs:59–113,209–230,421–466`、`src-tauri/src/services/settings.rs:58–72`、`src-tauri/src/commands/window.rs`中的设置写入、`src/Studio.svelte:231–247`。
  - **Reason:** 与动画/IPC优化同时发生时容易被误诊为“状态竞争”；配置丢失/损坏会造成长期稳定性问题。
  - **Proposed change:** 后端唯一写入队列将merge→validate→persist→publish变成带revision事务；使用Windows可替换的原子落盘方案，失败保留可恢复旧版本与明确UI错误。legacy setter与浮窗位置写入也进入同owner；避免锁持有期间做长Windows调用/磁盘I/O。
  - **Expected benefit:** 多窗口更新不丢失，崩溃/写入失败后配置可恢复，广播有权威版本。
  - **Risk:** 外部副作用（topmost/autostart）不天然原子，需要约定失败/回滚；Windows重命名覆盖语义不能照搬Unix。
  - **Verification:** 并发不同patch/位置保存100轮，最后所有字段与revision正确、disk与memory一致；模拟写权限/中途失败/副作用失败，UI报告错误且重启不回默认。已有设置迁移测试通过，前后I/O时间无明显回归。

### Phase 2 — CPU / Background Tasks

- [ ] **C1 · 🔴 P0 — Move blocking native work off the UI thread**
  - **Problem:** 同步control_media包含WinRT.get和150ms sleep；seek/shuffle/repeat、取色、pixelate、音量设备COM查询与部分文件操作也同步。process_image虽async，函数内同步decode/encode仍占Tokio worker。
  - **File:** `src-tauri/src/commands/media.rs:93–142`、`src-tauri/src/commands/audio.rs`、`src-tauri/src/services/image.rs:19–37`、`src-tauri/src/services/system_audio.rs`、`src-tauri/src/commands/cache.rs`、`src-tauri/src/commands/settings.rs`。
  - **Reason:** Tauri2同步command默认运行主线程；150ms已经远超16.67ms帧预算，也可能拖住cursor monitor的scale_factor请求和watchdog。不是单纯调整CSS可解决。
  - **Proposed change:** 对确定阻塞链路使用async command+受限blocking pool或专属COM actor；sleep改异步延迟/事件确认，保留操作完成与错误传播。CPU图片处理用有界blocking任务；所有worker遵守M1初始化。UI线程只处理必须在该线程的窗口操作。
  - **Expected benefit:** 点击播放/拖音量/封面处理期间主线程不长时间停顿，动画和点击穿透响应更稳。
  - **Risk:** 任意跨线程传COM对象不安全；仅加async关键字不足；返回过早导致假播放状态；blocking任务不可被强制取消需并发/过期预算。
  - **Verification:** 播放/暂停、seek、连续音量输入与同时展开trace，native UI线程不再执行150ms sleep/图片decode；操作失败能传到UI，状态以播放器结果为准。比较D的p95/p99帧时、命令延迟、watchdog重启与线程峰值；相关Rust/native测试通过。

- [ ] **C2 · 🟠 P1 — Replace permanent polling with demand and events**
  - **Problem:** media1Hz、screenshot20Hz、fullscreen/recording2Hz、cursor远处5Hz长期运行，与可见性/设置不完整关联；media每次重新获取manager/属性，Capture关闭也消耗检测。
  - **File:** `src-tauri/src/lib.rs:175–315`、`src-tauri/src/services/media.rs:554–643,697–765`、`src-tauri/src/commands/window.rs:410–522`、`src/App.svelte:1571–1608`。
  - **Reason:** 常驻程序大部分时间Idle，减少持续唤醒比动画期间微优化更影响后台CPU/电源。
  - **Proposed change:** SMTC manager由actor持有并订阅session/property/playback/timeline变化，低频兜底且只有必要字段查询；Capture检测按偏好开启，能力不支持时退避。鼠标维持自适应近处响应，缓存DPI、远处更低频/合适原生事件；工作区用变化事件+低频兜底。mouse debounce在分配timeout前先gate。
  - **Expected benefit:** 默认/关闭Capture两种Idle下减少线程唤醒、COM调用与事件量，接近0CPU目标。
  - **Risk:** SMTC事件漏发/播放器差异、鼠标穿透与唤醒延迟、截屏快捷键冲突；GetForCurrentView不支持任意录屏，不应通过加频掩盖能力限制。
  - **Verification:** A对照每项poll次数/CPU/线程唤醒，关闭功能后相应计数为0；播放/切歌/应用退出、Win+Shift+S/PrintScreen、fullscreen、4edge唤醒与多monitor不退化。所有低频兜底有上限，事件式listener也有cleanup；F验证锁屏/设备变化恢复。

- [ ] **C3 · 🟠 P1 — Make visibility a shared work gate**
  - **Problem:** pageVisible只反映document；主岛isHidden移出屏幕仍可能4Hzclock/30s快照；浮窗capture-hidden为CSS隐藏，MV/旋转/进度仍活跃；VisibleWindow已解决真正隐藏/关闭，不能忽略已有机制。
  - **File:** `src/App.svelte:292–304,639–659,1070–1088,1622–1660`、`src/FloatingWindow.svelte:76–79,848–855,1541–1575,2033,2136`、`src/lib/components/island/VisibleWindow.svelte`、`src/lib/components/island/IslandSurface.svelte:617–652`、`src/lib/components/media/Spectrum.svelte`。
  - **Reason:** “没有视觉变化就不要刷新UI”需要workVisible而不只CSS/DOM visibility；video与composition可在JS闲时仍耗GPU。
  - **Proposed change:** 定义document/native visibility、captureHidden、island state、当前page/hover控件共同决定的需求；停止不可见进度与Spectrum，暂停video/唱片/marquee，恢复时只补一次snapshot。保留Timer绝对deadline和必要capturing/唤醒；不因为后台music正在播放而一直启动不可见Spectrum。
  - **Expected benefit:** 隐藏真正进入低CPU/GPU；减少不显示的Svelte派生、decode与采集。
  - **Risk:** 隐藏期间Timer丢完成、重新显示进度跳错、暂停video后丢失用户意图；捕获隐藏不能恢复泄露内容。
  - **Verification:** playing+capture隐藏/主岛offscreen/minimize/副窗close四种路径分别检查：350ms宽限后无消费者则FFT/event为0、MV暂停、视觉timer停、无持续paint；恢复正确显示当前歌曲/Timer。B/E/F比较CPU/GPU；reduced-motion与showSpectrum关闭也检查。

- [ ] **C4 · 🟡 P2 — Separate clocks, weather and unseen system metrics**
  - **Problem:** 音乐250msclock每次updateTimeDisplay/Intl；天气30s快照同时刷新CPU/memory/network，即使Island只显示钟/温度；Studio1Hz HH:mm和会话10s不按区域需求；TimerWindow250ms更新秒数字。
  - **File:** `src/App.svelte:292–304,639–659`、`src/Studio.svelte:164–214`、`src/TimerWindow.svelte:70–88`、`src/lib/api/idle.ts`、`src-tauri/src/services/idle.rs:54–89,239–325`、`src/lib/countdown.ts:101–107`、`src/lib/components/island/InfoPanel.svelte:11–13`。
  - **Reason:** 全部可清理不代表频率必要；读了但未展示的系统指标仍有IPC/采集成本。
  - **Proposed change:** 分开minuteClock、visibleMediaProgress、Timer deadline；复用稳定Intl formatter；天气独立按TTL返回，system仅可见模块订阅所需字段。Studio会话区域可见/变化才取；秒边界更新数字。Spectrum颜色仅showSpectrum/颜色实际需求存在时解析，避免隐藏/关闭还decode封面。
  - **Expected benefit:** 少timer、少格式化/对象刷新，weather Idle没有不需要的sysinfo。
  - **Risk:** 首次CPU/network采样语义与共享collector elapsed会变化；Timer平滑线条可能需要局部动画，不能一刀切破坏观感。
  - **Verification:** A显示只有钟/天气时system采样计数0；分钟不变时clock文字derived不重算；paused/hidden媒体无进度timer；Timer到期准确且UI秒值正确。天气城市变更/失败/TTL仍正确；前后invoke/effect计数与CPU对比。

### Phase 3 — IPC / Geometry correctness

- [ ] **I1 · 🟠 P1 — Bound interaction-region synchronization**
  - **Problem:** 每次spring-derived polygon改变即发region command；App忽略settled，fire-and-forget，多请求可积压；signature在成功前记录，错误后同终态不重试。
  - **File:** `src/App.svelte:220–259`、`src/lib/components/island/IslandSurface.svelte:477–485`、`src/lib/api/window.ts:46–57`、`src-tauri/src/commands/window.rs:557–588`。
  - **Reason:** 72点JSON、Rust反序列化与锁占用参与动画热路径；Rust revision只能拒旧状态，不能省掉已发生的序列化/分派。
  - **Proposed change:** 引入1个在途+1个latest pending的sender，动画中先试20Hz并做合适logical像素量化/误差阈值；起始/最终/关键交互flush，消费settled；成功后才更新ack signature，失败可退避重试终态。Floating稳定round-rect走简单数据路径，edge保留必要shape精度。
  - **Expected benefit:** transition IPC/bytes下降、队列有界，同时可见控件仍可点到。
  - **Risk:** 只发终态或过宽envelope会丢点击/吞桌面；过度量化伤肩部点击穿透。20Hz是候选，必须实测后选择。
  - **Verification:** D记录发送/bytes/在途峰值与帧时；候选目标20Hz上限+少量阶段flush、在途≤1/pending≤1。逐帧位置取样点中控件/透明角/桌面，4edge与打断都正确；注入一次command失败后终态成功重发，不出现永久失效region。

- [ ] **I2 · 🟠 P1 — Unify logical/physical geometry before lowering update rate**
  - **Problem:** App expandedScale=.3875–.625，而surfaceOffsetFor/navigationHostFor最小.62；视觉gap与regiongap在scale<.62偏移；侧边style切换host宽不同；浮窗保存physical bounds而builder inner_size按logical值重开。
  - **File:** `src/App.svelte:220–234,750–783`、`src/lib/islandGeometry.ts:201–215,248–267,300–319`、`src/lib/components/island/IslandSurface.svelte:443–451`、`src/FloatingWindow.svelte:658–687`、`src-tauri/src/commands/window.rs:748–864`。
  - **Reason:** 降频之前必须保证同一帧geometry变换正确。scale=.3875时gap差为22×(.62−.3875)=5.115 CSS px；不是网络/IPC延迟造成。
  - **Proposed change:** 定义一种logical画面坐标，明确DPI转换点与anchor/workarea/host snapshot；消除不同clamp、为同edge两种style预留共同容器；bounds保存明确单位与DPI，重开按正确单位恢复。捕获hidden放置使用完成状态/实际轮廓；与A3协调，不提前改视觉时间线。
  - **Expected benefit:** 命中轮廓与可见轮廓一致、混合DPI恢复位置/大小稳定，同edge切换不额外resize。
  - **Risk:** 历史保存位置需迁移；舍入/屏幕负坐标/边缘1px overlap与2px唤醒区域有特殊约束。
  - **Verification:** geometry纯函数测试覆盖真实App最终scale .3875/.4/.55/.62/.625、4edge/style、DPI100/125/150/200%、负坐标/任务栏位置；实际点击边界偏差≤1 logical px，唤醒保留2 physical px。浮窗保存/关闭/跨DPI重开尺寸不倍增。D对照native resize计数。

- [ ] **I3 · 🟠 P1 — Publish one media snapshot authority and lossless deltas**
  - **Problem:** Rust media-update与App island-media-sync在浮窗竞争权威；无浮窗App也同步；同歌分支忽略albumArt；多个throttle采用leading drop没有trailing最新值，封面发送基线可能已前移却消息被丢弃；Timer每秒广播缺消费者gate。
  - **File:** `src-tauri/src/lib.rs:190–229`、`src-tauri/src/event_bus.rs:300–334`、`src/utils/eventManager.ts:139–177`、`src/utils/eventConstants.ts`、`src/App.svelte:177–188,610–635,1380–1534`、`src/FloatingWindow.svelte:718–846`、`src/lib/mediaStore.ts`。
  - **Reason:** 高频时序与大图数据应有明确初始snapshot/最新delta契约，不能依靠2.5s“相信主窗口”掩盖重复来源。
  - **Proposed change:** 原生或共享MediaController提供versioned snapshot，metadata/art变更可靠送达，进度仅低频校准前端投影；late join取完整最新cover。实时delta latest coalesce，不能丢pause/track/art最终值；只向有需求的窗口发送同步，Timer传deadline+状态/完成并依需求减少秒广播。后端缓存封面源带source，不先clone全base64再strip。
  - **Expected benefit:** 更低全应用IPC/bytes、更少重复HD工作、同歌补图正确，跨窗状态不漂移。
  - **Risk:** 旧消费者兼容、初始snapshot与event先后、seek重置和短暂SMTC空会话；改变throttle需逐事件区分可靠动作与可丢progress。
  - **Verification:** 无浮窗时island-media-sync=0；打开浮窗立刻收到最新HD与进度；快速play/pause/track/art变更最终状态不丢；Timer关闭/重开与主岛不可见仍按deadline完成。B/C对照IPC bytes、resolver次数、快照对象更新次数；mediaClock/mediaStore/capture/countdown测试与native场景通过。

- [ ] **I4 · 🟡 P2 — Bound action IPC and instrument the complete transport**
  - **Problem:** 音量pointer驱动暂无频率预算，虽然App已有串行latest队列；spectrum-data绕过EventBus，现有stats不代表所有IPC；全量settings与legacy事件重复处理；浮窗create缺单航班。
  - **File:** `src/App.svelte:460–545`、`src/lib/components/volume/VolumePanel.svelte:49–79`、`src/lib/components/volume/ArcVolumeControl.svelte:31–55`、`src-tauri/src/event_bus.rs:238–370,448–522`、`src-tauri/src/audio.rs:77–99`、`src-tauri/src/commands/settings.rs`、`src-tauri/src/commands/window.rs`。
  - **Reason:** 已优化的路径要继续收口而非重写；诊断不完整会把“总事件下降”当假收益。
  - **Proposed change:** 音量跳过相同整数、候选30Hz在途1+latest1，pointerup立即终值；复用设备endpoint避免每move激活。统计所有invoke/emit/receive与bytes，正式默认关闭；settings按revision一次提交发布，审计legacy subscriber后去重；窗口create同label单航班。Arc若可达，pointerdown缓存rect，resize/scroll失效更新。
  - **Expected benefit:** 动作IPC/COM调用有界，统计能解释实际帧与背景成本，避免连续点击创建竞态。
  - **Risk:** 音量最终值丢失、旧setting listener失效、instrumentation自身加成本、共享endpoint换设备后过时。
  - **Verification:** 连续拖动1分钟终值准确，command数不随鼠标采样率无限提高，切设备/系统外部音量变化一致；重复开浮窗只一个实例。stats与原始trace采样数对齐，含直接Spectrum事件；诊断关闭时额外CPU近于测量噪声。

### Phase 4 — Animation Engine

- [ ] **A1 · 🟠 P1 — Introduce an interruptible IslandAnimationController**
  - **Problem:** intent、shapeSettled、pageExiting、cover epoch、placement revision、hide flags分别控制同一过渡；固定duration并非实际spring settle，快速目标变化容易不同步。
  - **File:** `src/lib/islandStore.ts`、`src/lib/islandMotion.ts`、`src/lib/components/island/IslandSurface.svelte:208–258,389–438`、`src/App.svelte:802–891,1038–1088`。
  - **Reason:** 动画自然度来自协调的几何/内容/封面时间线与可打断性，单改stiffness不能解决挂载与完成标准。
  - **Proposed change:** 实现第3节九状态、单一transition epoch/frame snapshot、开始/settled/cancel回调；方向反转从当前位置/必要速度续接。先统一owner保留独立向量通道，profile后决定单进度或向量spring；删固定460ms等待，统一低动效与visibility策略。组件只消费snapshot、native sender独立。
  - **Expected benefit:** 目标重设不中断连续感，不再被迟到完成回调重置当前状态；动画结束无后台动画循环。
  - **Risk:** controller过度抽象反增依赖，所有属性使用单progress会损失自然性；被取消spring promise的完成语义必须明确。
  - **Verification:** 状态/epoch测试与D：expand↔collapse中途反转、hover快速进出、page切换后hide/restore、settings改变、Timer完成。只允许一个当前transition，旧完成无写入，静止无rAF；与Phase3相同IPC预算，60Hz实际帧时目标见第12节。

- [ ] **A2 · 🟠 P1 — Use one geometry measurement and transform for album motion**
  - **Problem:** 封面250ms每帧getBoundingClientRect并写left/top/width/height，目标同时随壳布局移动；收起主动取消overlay/隐藏两端，没有逆向连续封面。
  - **File:** `src/lib/components/island/IslandSurface.svelte:286–378,776–786,793,866–868`、`src/lib/components/media/CoverArt.svelte`、`src/lib/islandGeometry.ts`。
  - **Reason:** 测量→布局写入→spring布局变化→再测量形成Forced Reflow候选；封面消失打断Dynamic Island的物体连续性。
  - **Proposed change:** FLIP或由统一geometry快照给出的source/targetrect，最多转换开始/布局条件改变时测量，帧内transform translate/scale与适量radius；封面identity跨expand/collapse共用有界overlay，结束/取消释放临时will-change和src。不让overlay追踪DOM每帧measure。
  - **Expected benefit:** 降低layout cost、封面正反向连续，打断后位置不跳。
  - **Risk:** 文本/封面shape被不等比缩放，跨DPI/换屏坐标失真；Object URL过早revoke或目标图未decode造成白闪。
  - **Verification:** 记录每次转换DOM measure次数目标≤2（特殊布局变更另记），rAF内getBoundingClientRect为0；trace中该路径forced layout消失/明显下降。D包含音乐换封面中途展开/收起，旧/新图无闪烁、错位、空白；C确认overlay资源回稳。

- [ ] **A3 · 🟠 P1 — Separate visual hiding from native bounds transactions**
  - **Problem:** auto-hide保留Expanded mode，却用Compact尺寸移窗；内容visible立即变false；hideMorph只用于模拟预览；native线程16ms重复提交尺寸，dt最多16ms造成延迟帧时钟变慢，返回Ok不等于实际settled。
  - **File:** `src/App.svelte:718–865,1038–1088`、`src/lib/islandGeometry.ts:248–267,308–319`、`src/lib/components/island/IslandSurface.svelte:430–451,575–633`、`src-tauri/src/commands/window.rs:1128–1278`。
  - **Reason:** 当前实际隐藏和视觉状态机脱节，Expanded外壳可能仍露出；native moving不应成为每帧视觉owner。
  - **Proposed change:** 普通hide由controller进入Hiding，实际轮廓完成遮蔽后阶段性放置/终态sync；捕获场景立即使整个外壳不可见且暂停work，保证安全终态，不等长动画。保留2px唤醒的形态单独计算。native纯位置使用适合move的flags，不重复resize；若保留native动画用真实elapsed/有界settle、取消token/最终精确落点，发完成事件。
  - **Expected benefit:** 完整隐藏/恢复，没有突然resize、残留壳/错唤醒；native composition调用减少。
  - **Risk:** 唤醒区域丢失、点击穿透冻结、顶部边缘1px缝隙、捕获期间恢复时机错误；不能只“移动远一点”而不核实interaction。
  - **Verification:** Expanded/music/每工具页+4edge触发screenshot/fullscreen/recording/manual hide，只剩规定唤醒条，无正文/外壳残留；restore保留最新意图。D记录普通展开native resize为0，同edge style不resize；hide/placement SetWindowPos次数下降、实际完成准确，锁屏/热插拔后恢复正确。

- [ ] **A4 · 🟠 P1 — Coordinate content mount, page exit and shell morph**
  - **Problem:** Compact层立即unmount，expanded收起opacity=1，旧renderedPage在缩壳100ms时仍占旧尺寸；Page entry180ms与spring分离，Marquee/ResizeObserver可重复启动。
  - **File:** `src/lib/components/island/IslandSurface.svelte:408–429,488–493,575–733,815–845`、`src/lib/components/media/MarqueeTitle.svelte`、`src/lib/components/media/PlayPauseIcon.svelte`。
  - **Reason:** 看起来“不自然”不仅是帧率，还有元素身份消失、旧内容被裁、焦点与可点击状态提前切换。
  - **Proposed change:** 使用A1的统一progress决定有限双层挂载、opacity、inert、focus时机；Page exit→布局目标/壳变形→enter有明确阶段，快速切换latest page；未退出的旧页不被提前缩壳裁掉。过渡时暂停marquee测量，settled后一次重测；low-motion走同状态契约快速完成。
  - **Expected benefit:** Compact/Expanded和Page切换连续无闪烁，控件不会在透明阶段拦截输入。
  - **Risk:** 双层DOM/封面长期保留、focus转移打断输入；不能以常驻所有页面换动画流畅。
  - **Verification:** D录帧检查旧内容裁切、透明空档、focus/键盘Escape；转换完成只保留当前内容DOM。no-preference/reduce两套测试与100次page切换、mount/unmount后Nodes/listeners回稳；marquee测量计数不跟spring每帧增长。

### Phase 5 — UI / Rendering

- [ ] **U1 · 🟡 P2 — Keep Compact sparse and use final logical sizes**
  - **Problem:** Idle clock/weather叠加padding在80宽里空间紧；Timer强制240长轴且竖向文本受限；Expanded全页zoom导致字/目标过小，轻量感依赖缩小而非信息层级。
  - **File:** `src/lib/components/island/IslandSurface.svelte:193–202,575–618,795–805,815,846–869`、`src/App.svelte:750–758`、`src/lib/components/island/InfoPanel.svelte`、`src/lib/components/time/TimerPanel.svelte`。
  - **Reason:** 所有状态自然过渡的前提是终态排版能读/能点；Compact不应塞歌名、歌手、天气、时间、Spectrum和工具。
  - **Proposed change:** 按第4节Compact规则，一次主要音乐/Timer/Idle状态；明确可用宽度与overflow策略。Expanded以最终logical尺寸定typography、button hit target、padding/留白，responsive调整布局而非整体.625缩放；候选正文≥12px、图标按钮hit area≥28–32px，再按Windows实际显示审核。
  - **Expected benefit:** 紧凑但可读，窄显示器/DPI不会让工具页与操作缩成微型控件，减少layout溢出。
  - **Risk:** 调大目标后窗口占地增加；不能简单放大整个Island，需要内容减量；旧外观设置需兼容。
  - **Verification:** 4edge、DPI100–200%、窄逻辑workarea、长歌名/中文日文英语、无封面/温度负数/小时Timer均无溢出；实际logical字/按钮尺寸测量，与baseline截图比对；D帧/IPC预算不退化，Compact保持单一主状态。

- [ ] **U2 · 🟡 P2 — Simplify Expanded hierarchy and real interaction states**
  - **Problem:** 默认7个工具用5槽横滑，多个伪功能图标、重复浮窗入口；音乐prev/play/next未使用capabilities禁用，而另一个PlayerControls已有能力处理。
  - **File:** `src/lib/components/island/IslandSurface.svelte:247–272,640–710,846–869`、`src/lib/components/island/FeatureMenu.svelte`、`src/lib/components/media/PlayerControls.svelte`、`src/lib/components/volume/VolumePanel.svelte`、`src/lib/components/time/TimerPanel.svelte`。
  - **Reason:** 多入口/无行为装饰增加控制面板感，button语义、hover/active/focus与用户理解不一致。
  - **Proposed change:** 一主页面、少量Secondary；3–4个高频工具+更多入口候选，设置保留独立Studio；去掉假按钮装饰和重复入口，按能力启用；统一hover/pressed/active/focus/disabled视觉与短动作时长，保留工具顺序定制与键盘导航，减少Card/Border/Divider。
  - **Expected benefit:** Expanded仍像连续动态信息容器，功能增长不把外壳变成工具总览面板，入口和操作更清楚。
  - **Risk:** 功能发现与习惯入口位置变化；“更多”过深；改按钮顺序不应打破keyboard/aria契约。
  - **Verification:** 主播放/音量/Timer/天气/时钟/设置/浮窗/隐藏所有功能可达；不支持能力正确disabled，动作不会误展开/收起。滑动、wheel、键盘、Escape、focus-visible与原生命中正确；无新增常驻卡片/边框墙，页面DOM/帧时无回归。

- [ ] **U3 · 🟡 P2 — Remove idle decoration work and budget composition**
  - **Problem:** Timer未运行界面条纹无限820ms动画；浮窗旋转与MV在CSS隐藏时仍工作；永久will-change、translateZ与动态polygon可能增加layers/paint。主岛无默认blur，旧FeatureRail的blur可能已不在活跃UI。
  - **File:** `src/TimerWindow.svelte:884–907,1074–1078`、`src/FloatingWindow.svelte:2093–2195`、`src/lib/components/island/IslandSurface.svelte:477–483,792–813`、`src/lib/components/island/FeatureRail.svelte:156–158`、`src/Studio.svelte:441,468,473,561–562`、`src-tauri/src/lib.rs:391–410`。
  - **Reason:** GPU Idle要看实际paint/composition和video decode；静态shadow/opacity/transform不是天然持续耗GPU，不能依据关键词删除全部设计。
  - **Proposed change:** Timer Idle静态选中态；旋转按可见性与动效偏好暂停/有限运行（C3基础上）；临时动画才加will-change，settled释放；简化edge mask只在trace确认成本时实施。核实FeatureRail/native vibrancy活跃调用，没有可达链路则不作为P1优化；Studio blur限可见必要区域。
  - **Expected benefit:** 长期常驻少无意义合成帧，GPU接近桌面Idle，必要动画仍有层支持。
  - **Risk:** 去层导致动画期间重paint；clip-path替换破坏肩部精确轮廓；透明host改变可能伤点击穿透。
  - **Verification:** A与Timer选择界面静置/MV隐藏对照GPU engines、present/paint count、layer memory；Idle装饰无持续帧，静态shadow保留不造成无谓重绘。D保持60FPS目标，低动效/低透明偏好无无限动画，截图与region吻合。

### Phase 6 — Architecture Cleanup（仅有收益时）

- [ ] **R1 · 🟡 P2 — Extract owners only where update scope and coupling improve**
  - **Problem:** App集中多个服务与大量props/effects；Media、clock、settings变化可影响不相关derived/placement检查，但不能用行数推定整树rerender。
  - **File:** `src/App.svelte`、`src/lib/components/island/IslandSurface.svelte`、`src/lib/islandStore.ts`、`src/lib/mediaStore.ts`、`src/lib/settingsStore.ts`、`src/lib/api/`。
  - **Reason:** 先修生命周期/IPC/动画后再划分owner，能避免两份状态、重复订阅与控制器相互循环。
  - **Proposed change:** 依据第8节effect计数/trace逐个提取Animation、Placement、Media、Audio/Weather等确有职责边界的owner；Timer保留唯一deadline；将clock高频更新局限到进度组件，稳定props与derived。只转发调用且不会缩小更新范围的Window/IslandController不创建。
  - **Expected benefit:** 易profile、依赖清楚、低频/高频更新范围分离，可测重复计算减少。
  - **Risk:** 多store/多订阅增IPC，异步启动顺序改变，拆分后毁坏原有untrack/cleanup。
  - **Verification:** 拆前后effect/derived次数与B/D帧时对照；每owner启动/释放次数有界，listeners/commands不增；相同业务与native回归通过。不以文件行数或“更整洁”作为性能验收。

- [ ] **R2 · 🟢 P3 — Remove verified dead paths and align documentation/config**
  - **Problem:** 占位Spectrum service、FeatureRail旧UI、未使用easing/旧布局参数/legacy事件/宽泛API可能误导profiling；setup注释仍写启动Spectrum，Timer注释仍写pre-created，与实际按需行为不同。
  - **File:** `src-tauri/src/services/spectrum.rs`、`src/lib/components/island/FeatureRail.svelte`、`src/lib/islandMotion.ts`、`src/lib/islandGeometry.ts`、`src/utils/eventConstants.ts`、`src/lib/api/settings.ts`、`src-tauri/src/lib.rs`、`src/TimerWindow.svelte`、`src/styles/`、`package.json`与锁文件。
  - **Reason:** 清理只有在调用图证实dead或重复时合理；不会把低优先级清理伪装成内存大幅下降。
  - **Proposed change:** 按import/command注册/事件收件者/兼容设置字段查调用图，删除确认无用部分，更新注释与性能说明；保留必要migration、fixture复用与capability；选择权威lock并明确native构建版本。lint空echo不是实际质量检查，是否补lint单独决定。
  - **Expected benefit:** 审查/profile结论更可靠，减少维护与误用，可能减小bundle但量化后才宣称收益。
  - **Risk:** 仅rg无import不等于没有native/dynamic调用；删除legacy破坏旧settings迁移或外部使用。
  - **Verification:** bundle/call graph与command/event消费者清单；check/test/build/Rust测试通过，旧设置升级不丢失；前后bundle与内存只有实测收益才记录，否则记可维护性改进。

### Phase 7 — Long-running Stability Test

- [ ] **TA · 🟠 P1 — Benchmark A: 10-minute idle**
  - **Problem:** 没有当前native Idle稳定性证明，多个后台检测可能掩盖“JS空闲”。
  - **File:** `scripts/measure-native.ps1`、`docs/performance/`、`src-tauri/src/lib.rs`、`src/App.svelte`、`src-tauri/src/commands/window.rs`。
  - **Reason:** 常驻应用最常见场景必须成为发布门槛。
  - **Proposed change:** 按第9节A，默认/关闭Capture、有/无天气城市分轮；每轮完整10分钟，3次，收全部指标与辅助工作计数。
  - **Expected benefit:** 证明Idle CPU/GPU回到近空闲且内存稳定，明确默认功能的成本。
  - **Risk:** 用户桌面其他GPU/媒体活动、系统更新、DevTools会污染；控制组与应用组同条件。
  - **Verification:** 第12节Idle门槛；无持续视觉rAF/FFT/region更新/图片处理，使用相同优化前场景对照所有指标，结果完整才勾选。

- [ ] **TB · 🟠 P1 — Benchmark B: 30-minute music**
  - **Problem:** 真实音乐、SMTC、封面、浮窗/MV在一起的持续成本未知，Chrome random数据不覆盖。
  - **File:** `src/App.svelte`、`src/FloatingWindow.svelte`、`src-tauri/src/services/media.rs`、`src-tauri/src/audio.rs`、`docs/performance/`。
  - **Reason:** 优化不应牺牲时间校准、播放控制与歌曲元数据/封面体验。
  - **Proposed change:** 固定播放器/音乐语料，Compact/Expanded、Spectrum开关、浮窗/MV各做独立30分钟轮，持续采第9节全部指标。
  - **Expected benefit:** 分出媒体、Spectrum和MV边际成本，证明长时间播放无异常增长。
  - **Risk:** 网络与播放器自身更新影响数据；变化变量太多无法归因。
  - **Verification:** 与baseline同条件3轮；播放/暂停/seek/切歌正确，进度与权威状态一致、缓存/请求/线程有界；关闭可选功能时对应工作停止，所有核心指标曲线与最终结果归档。

- [ ] **TC · 🟠 P1 — Benchmark C: 100 track changes**
  - **Problem:** 原尺寸Canvas/Base64/过期Image与缓存峰值需要实际Renderer/JS/GPU增长证据。
  - **File:** `src/FloatingWindow.svelte`、`src/lib/components/media/CoverArt.svelte`、`src/lib/spectrumColors.ts`、`src-tauri/src/services/media.rs`、`src-tauri/src/services/image.rs`、`src-tauri/src/services/cache.rs`、`docs/performance/`。
  - **Reason:** 只查缓存size不能排除DOM/decoded image/native在途增长。
  - **Proposed change:** 第9节C语料包含100首不同图、无图/空会话、网络慢/失败、source同名；第0/25/50/75/100首采snapshot，最后静置5分钟，单独post-GC诊断轮。
  - **Expected benefit:** 证明封面owner正确、内存平台有界，定位额外内存而非笼统归WebView2。
  - **Risk:** 强制GC/缓存清空掩盖正式工作集平台；不同图尺寸/网络不可比。
  - **Verification:** 最终无错图/错色/空白，无累积Detached Canvas/Image/listener；decode/cache/在途字节符合设定预算；Renderer Memory、JS Heap、GPU Memory与app commit曲线均对比baseline，原始图/语料尺寸留记录。

- [ ] **TD · 🟠 P1 — Benchmark D: 500 expand/collapse cycles**
  - **Problem:** 单次动画观感或reduce测试不覆盖中断竞争、native区域错误与长期动画对象保留。
  - **File:** `src/lib/components/island/IslandSurface.svelte`、`src/lib/islandMotion.ts`、`src/lib/islandGeometry.ts`、`src/App.svelte`、`src-tauri/src/commands/window.rs`、`ui-tests/`、`docs/performance/`。
  - **Reason:** 要同时验证帧时、连续性、点击区域和资源回收。
  - **Proposed change:** 按第9节D分正常/快速反转/混合状态500次，四edge、DPI、不同刷新率分轮；保留真实动画no-preference，另跑reduce正确性。
  - **Expected benefit:** 证明动画控制器/区域降频/native分离产生可量化收益且无视觉回归。
  - **Risk:** 自动输入间隔改变用户真实行为；rAF FPS可能高而实际呈现仍掉帧。
  - **Verification:** 实际帧时/Long Task/IPC bytes/measure/native resize/DOM与监听对照；第12节动画与命中门槛满足，500次后稳定资源，录帧无封面消失、layout shift、闪烁、残留外壳和最终状态竞争。

- [ ] **TE · 🟠 P1 — Benchmark E: 30-minute real Spectrum**
  - **Problem:** gate已经存在，但多窗口/捕获隐藏/设备失败/代次恢复是否真的停capture与FFT仍须native验证。
  - **File:** `src-tauri/src/audio.rs`、`src-tauri/src/lib.rs`、`src/lib/spectrumStore.ts`、`src/lib/components/media/Spectrum.svelte`、`docs/performance/`。
  - **Reason:** 在语义正确前盲目降低FFT/事件频率会牺牲视觉却未解决隐藏音频工作。
  - **Proposed change:** 真实WASAPI音频30分钟，按固定时点暂停/隐藏/showSpectrum关闭/切设备/无设备/恢复，包含多个消费者、静音与非零；random独立轮确认不启native。
  - **Expected benefit:** 核实playing+enabled+visible gate及线程/设备资源退出，明确真实FFT成本。
  - **Risk:** 同名设备变更仅按name判断可能漏重启；硬件采样格式、失败loop会影响可比性。
  - **Verification:** 无消费者350ms宽限后capture/FFT/publisher零；活跃事件≤20Hz+已定义心跳，audio callback不IPC；设备重连线程峰值回稳无累计增长，30分钟CPU/heap/native/GPU与baseline对照，音频/系统播放不受干扰。

- [ ] **TF · 🟠 P1 — Benchmark F: 8-hour background stability**
  - **Problem:** 短时间heap/Nodes稳定不能证明原生资源、处理任务、GPU或COM八小时不增长。
  - **File:** `src/`与`src-tauri/src/`各资源owner、`scripts/measure-native.ps1`、`docs/performance/`、本计划结果记录。
  - **Reason:** 最终验收需覆盖常驻与休眠恢复，不以“运行了8小时没crash”代替数据。
  - **Proposed change:** 8小时Idle为主，固定时点音乐/100首切换/Timer完成/副窗开关/锁屏与睡眠恢复/热插拔；每秒轻量OS采样、每小时快照，结束后恢复纯Idle5分钟；至少baseline与优化后同条件对照，异常轮复测。
  - **Expected benefit:** 证明无持续资源增长且真实功能跨长期运行/恢复仍正确。
  - **Risk:** WebView2动态缓存/系统trim导致WS锯齿；系统后台活动影响CPU，不能按单个端点下结论。
  - **Verification:** 第12节长期门槛，分析warm-up后commit/privateWS/post-GC heap趋势、GPU、线程/句柄/IPC；明显斜率有retainer/原生分配归因。所有核心指标完整，功能恢复与最终Idle一致，数据对照及结论记录后才勾选。

## 12. 完成标准、阶段记录与前三项

下述数值是**候选验收预算**，不是当前实测结论。Phase0根据机器控制组噪声确定最终阈值，并在任何优化前冻结，不能优化后调整阈值来“通过”。

| 目标 | 验收方式 |
|---|---|
| Idle CPU接近0 | Isle进程组10分钟平均整机CPU候选≤0.1%，1s p95≤0.5%；同时公布逻辑核数/单核等价值与桌面控制组，不以多核归一掩盖单线程长期工作。默认Capture成本如超预算必须解释并优化。 |
| GPU接近Idle | 排除可见播放/MV/Spectrum后，不持续提交装饰帧或video decode；应用归属GPU activity接近控制组噪声，无周期性高负载；GPU memory进入平台，不要求透明窗口占用为0。 |
| 内存稳定 | warm-up与有界缓存填充后，不出现随歌曲数/循环数线性增长的retained对象、commit、线程或句柄。8h post-warmup private commit/retained heap增长候选≤max(10MiB,5%)且无可归因持续正斜率；缓存预算和新增窗口驻留变化另列，不强求WS每次回到启动数。 |
| Animation 60FPS | 60Hz环境实际frame time p95≤16.67ms，p99候选≤25ms，missed-vsync候选<1%；无>50ms由应用路径产生的动画Long Task；报告p50/p95/p99与最差片段。高刷新率使用对应预算，不用rAF次数充当呈现FPS。 |
| 无明显视觉/状态回归 | 无封面消失、突然native resize、layout shift、闪烁、旧page被提前裁切；快速打断/隐藏恢复最终状态正确，无迟到动画竞争。 |
| 命中/放置 | 可见按钮、肩部透明区域和桌面穿透正确；logical/physical转换一致；4edge唤醒条保持2 physical px，DPI/工作区变化后位置准确。 |
| UI轻量 | 一次一个Primary任务，Compact不堆信息，Expanded少量工具+清晰导航，不增加大量Card/Border/Divider；字号/命中尺寸可读可点。 |
| 资源与IPC | 无消费者后capture/FFT/Spectrum IPC停止；普通展开native resize为0；region在途/更新频率有界且终态必达；所有定时器、listener、订阅、Image与临时动画有owner/cleanup。 |

### 推荐最先执行的三个明确任务

1. **B0（先配合B1修复验证入口）：Windows基线与资源归属。** 先取得A/C/D可复现数据，避免只凭WebView2占用猜测；完整长测排程不阻塞确定缺陷定位。
2. **M1：原生资源释放与线程初始化。** 有直接代码证据、变更范围可控，先修确定泄漏/COM生命周期，再测长时间增长。
3. **M2：Canvas生命周期与旧图回调。** 直接覆盖用户100首场景的显示与保留问题，先稳定图像owner。随后按Phase1顺序处理M3–M6，再进入C1/C2等后台/主线程工作；不提前重做UI。

### 每任务的执行结果模板

| Task ID | Before commit / After commit | 环境/场景 | Before数据 | After数据 | Delta与归因 | Tests/回归结果 | 结论/风险 | 阶段提交 |
|---|---|---|---|---|---|---|---|---|
| B1 | 本地修改前快照 / 当前工作目录 | Windows 11；npm lock | 65 项单测中 1 项旧断言失败；旧性能脚本入口失效 | 70 项单测、9 项 UI、check/build、Rust 检查通过 | 修正旧断言与真实夹具入口；动画在帧循环内采样 | 见执行记录 | 验证入口完成；native 场景仍有覆盖盲区 | 未提交 |
| M1 | 本地修改前快照 / 当前工作目录 | Windows COM/WinRT、真实音频设备 | 原始属性无清理、线程初始化未配对 | RAII 守卫；错误转换路径与 1000 次设备查询通过 | 正确性修复；原生 retained allocations 收益未定量 | 39 项 Rust 单测通过，硬件 stress 单独通过 | 代码完成；长期原生分配验证待补，暂不勾选 | 未提交 |
| M2 | 本地修改前快照 / 当前工作目录 | Svelte action、真实 Chromium Canvas | DOM 查询缓存与迟到绘制缺口 | 5 项单测、2 项真实 DOM 回归通过 | DOM owner 退出清空 backing store、图片回调及节点引用 | 包含 100 次卸载重建、迟到后台结果 | 代码完成；真实 WebView2 heap/retainer 与 GPU 回稳待补，暂不勾选 | 未提交 |
| B0 | 本地修改前快照 / 当前工作目录 | debug native + Chrome production fixture | 短时进程树诊断 | 动态 PID、CPU、Private WS/Bytes、线程、句柄、GPU 原始记录 | 数据用于验证采集链路，无正式优化收益结论 | 全部脚本冒烟通过 | 完整基线与 A–F 长测未完成，暂不勾选 | 未提交 |

勾选`[x]`意味着相关测试、同条件数据对比、功能回归和结果记录已完成；“代码已改”不单独构成完成。纯正确性修复记录资源/错误路径验证；没有显著性能收益时如实记零收益，不造数字。每个阶段独立提交，失败/回归先修或回滚当前任务，不扩张成一次性全仓重构。

## 13. 技术依据

平台资料只用于解释机制，具体问题依据本仓库审查提交，不从README推断：

- [Tauri 2 Calling Rust — command线程语义](https://v2.tauri.app/develop/calling-rust/)：同步command默认主线程；重任务需正确异步/后台执行。
- [Svelte $effect — 依赖、批量更新与teardown](https://svelte.dev/docs/svelte/$effect)：同步读取跟踪、异步边界与cleanup；组件行数不等同整树rerender。
- [Microsoft WebView2 process model](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model)：browser/renderer/GPU进程组与共享环境，不能按“每窗口一个固定进程”相加。
- [Microsoft IPropertyStore::GetValue](https://learn.microsoft.com/en-us/windows/win32/api/propsys/nf-propsys-ipropertystore-getvalue)、[PropVariantClear](https://learn.microsoft.com/en-us/windows/win32/api/propidl/nf-propidl-propvariantclear)：属性返回结构与释放接口。
- [windows-rs 0.52.0 PROPVARIANT生成绑定](https://github.com/microsoft/windows-rs/blob/0.52.0/crates/libs/windows/src/Windows/Win32/System/Com/StructuredStorage/mod.rs#L2947-L2965)：检查了实际版本的定义/实现，没有自动Drop，不能套用较新版/其他crate的行为。
- [Microsoft CoInitializeEx](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coinitializeex)：每次成功初始化，包括S_FALSE，需对应CoUninitialize，并遵守线程apartment模型。
- [Chrome DevTools memory problems](https://developer.chrome.com/docs/devtools/memory-problems)：用retained对象、Detached DOM与heap snapshots区分bloat/leak；Chrome fixture不等同native WebView2总开销。

**当前执行状态：B1、M1 已完成；M2 封面正确性/Canvas 所有权验收完成并独立提交。B0 完整基线尚未完成；M3 预算实现已提交，本轮同条件 Release C 六轮对照已完成，但整体 Commit/CPU 回归，保持未勾选。GPU/内存长期稳定仍未证实。其他阶段不动，本轮限于 M3 Release C 记录与验收，不继续改业务实现。**
