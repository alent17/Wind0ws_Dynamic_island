# Isle Optimization Plan

> **2026-10-24 用户验收决定：** U1–U11、A1–A4、I1、I2、H1共18项，按用户本轮明确指令全部勾选完成并关闭。任务条目已归入[已完成优化任务](OPTIMIZATION_COMPLETED.md)，原测试记录和证据缺口保留为历史；本次仅更新状态，不补写验证结果。

> **2026-10-16 用户验收决定：** C1、A5按“实现完成”口径关闭并勾选。C1当前Release性能收益仍为未知，A5完整原生体验矩阵仍未补测；这些证据缺口保留为非阻塞记录，不再阻止两项关闭。H1继续作为独立首击问题，当前执行转入C2。

> **2026-10-16 下一步进展：** 默认关闭的compact-native-probe已接真实IslandSurface与共享几何，限独立宿主稳定Compact，未替换主岛。窗口离线41/41、独立前端构建、Svelte零错误/警告和默认feature后端检查通过。现场启动被执行策略整体拒绝（仅返回blocked by policy，拒绝来源未进一步披露），未绕过，宿主启动/测试点击均0；不能记成产品首击通过。见[候选报告](performance/compact-native-2026-10-16/REPORT.md)及第32节。

> **2026-10-16 当前执行口径调整（用户要求换办法并“开始”）：** C1-I阻塞迁移/契约与C1-P当前Release性能分开；A5-I动画实现与A5-F主观手感分开；H1原生首击作为独立发布阻塞项。长期资源另列R1，不再把首击/lab恢复/长期资源串成C1的完成条件。冻结旧静态lab工具扩建，改用不运行产品setup的最小原生→WebView实验。历史完整验收未补齐，不伪勾总项；本轮实际状态与后续停止线见第31节。

> 2026-10-15/16 第11批：单次静态lab前置10秒持续Enqueued，实际测试move/down/up均0，未重开或补点击。恢复失败保留，16日两配置原字节及Run原值恢复回读一致；原工具已退出但历史前台/光标完整恢复未证实。C1/A5 `[ ]`，见第29节和[报告](performance/a5-static-lab-2026-10-15/REPORT.md)。

> 2026-10-15 第10批：静态lab协调器、对称收件/no-activate底板与恢复工具已完成纯离线准备；28项JS、9项owner、5项日志/释放故障测试及C# CompileOnly通过。第9批代码/构建已独立验收；本批工具待审、最多4次静态点击的具体现场单待人类允许，未启动GUI/输入/采集或重建业务。C1/A5 `[ ]`，见第28节和[报告](performance/a5-static-tools-2026-10-15/REPORT.md)。

> 2026-10-15 第9批：默认关闭experimental-native-region真实适配器仅限独立静态lab；非产品接入。主岛仍旧monitor，实验label拒绝旧接管，实际UI/HRGN移交、宿主生命周期和故障策略已编译/离线验证；OS路由、视觉及资源仍未知。C1/A5 `[ ]`，见第27节和[报告](performance/a5-region-adapter-2026-10-15/REPORT.md)。

> 2026-10-15 第8批：首击机制审查完成，持久窗口形状方案仅交付纯离线隔离原型，候选待审/待现场验证；默认生产路径未接入修复。原型复用共享几何，覆盖过期版本/生命周期失效和失败状态；标准Release构建不证明首击。C1/A5 `[ ]`，Tween420/360/240及媒体修复保留，见第26节和[报告](performance/a5-first-hit-2026-10-15/REPORT.md)。

> 2026-10-15 第7批交付（14日晚执行）：用户“继续”后完成固定8次副屏停留对照并停止。页面click4、底板MouseDown3、无观测1；0预算实际3.8–8.8ms，等待结果不覆盖立即首击失败。按下前bit20与收件相关，支持时序候选但未确定单一根因，accepted-region/完整down-up路由仍缺。实际8%音量和原正在播放状态、设置/前台已恢复，cursor恢复请求未即时回读。C1/A5 `[ ]`；见第25节和[报告](performance/a5-dwell-2026-10-14/REPORT.md)。

> 2026-10-14 第6批工具返修：按下成功后的EOF/stop/错误清理及双收件冲突判定已修订，PS fake-backend 9/9、JS 9/9离线通过，待独立验收。原包保留，全程无GUI/输入；首击原因仍未知，C1/A5 `[ ]`。见第24节返修记录和[报告](performance/a5-offline-revision-2026-10-14/REPORT.md)。

> 2026-10-14 第6批：仅离线复核首击失败与准备工具，根因未确认。旧helper合并move/立即click，五条底板回调至后查45.8–54.6ms，成功floating先移入但并非同点精确停留。轮询ignore迟到、hover/坐标变化与区域不匹配均是候选，缺pre-down状态/已接受region不能确证。拟议move/press分离、0/50/250ms单点方案与DIV焦点工具仅dry-run，7/7通过；没有GUI/输入/采集。C1/A5 `[ ]`；见第24节。

> 2026-10-14 第5批：使用第2批保存标准Tauri Release完成有界原生交互取证，完整矩阵未通过。四edge首次DOM候选点击和切回floating首次点击实际落底板、页面无click；未重试凑通过或改业务。浮动状态部分操作、已就绪真实封面终态、独立原生透明穿透、手动隐藏恢复及原生页面低动效模拟有有效证据；焦点可见性、真实跨DPI/主观手感未验收，loaded运行时版本未记录。完整恢复16%音量/设置/播放态/前台，保留网易云。C1/A5 `[ ]`；见第23节和[报告](performance/a5-native-2026-10-14/REPORT.md)。

> 2026-10-14 当前状态：第4批旧C1原始复核完成。95.0015/54.9938ms源行存在，但旧分类把100ms稳定确认当精确终点、漏MsUntilDisplayed且16小时映射未证实；撤回实际动画内停顿结论，归属无法确认。第2批Apple能力/错误与第3批资源链路已独立验收，仅有效seek为N/A，新Release性能未验收。C1/A5 `[ ]`；最小剩余项见第22节和[复核报告](performance/c1-review-2026-10-14/REPORT.md)。

> 2026-10-14：第2批媒体能力/错误返回修复完成可审阅交付。toggle/skip的WinRT false现在传播拒绝；非网易云seek=false在派发前返回不支持，网易云兼容尝试保留；主页面播放态只按实际快照更新。新Release上Apple Music播放/暂停、时间线、不支持seek、只读入口、无会话错误及单独UI拒绝故障注入通过。真实声明支持seek的会话当前不可用，有效seek验收仍N/A；C1/A5 `[ ]`，未提交推送，详见第20节和[报告](performance/c1-media-2026-10-13/REPORT.md)。

> 2026-10-13：本批次无IPC代理配对与独立IPC诊断配对均完成16/16转换媒体AND音量覆盖，同页性能时钟门禁和四侧原始独立复算通过。Apple Music真实SMTC播放/暂停、时间线和错误负例已记录；seek返回成功但未观察到目标，仍未通过。候选region请求/参数字节约增61%，资源采样存在覆盖缺口，不宣称收益。C1、A5保持 `[ ]`，参数420/360/240ms不变、Spring延期；见第19节和[本日报告](performance/a5-2026-10-13/REPORT.md)。下列10月12日记录保留为历史。

> 2026-10-12 统计复核：分析器把“最后样式变化后稳定100 ms”的确认快照用作动画终点，旧7/5计数不能证明动画内卡顿；候选区间位于近似确认尾段或跨越不确定尾段。现有数据缺最后样式变化时间，需小范围配对补采后再判断。C1、A5保持 `[ ]`，Spring迁移延期至计划阶段完成后单独评估。分析文件及脚本已上传，详见第16节。

> 2026-10-12 13:30（北京时间）同步：C1 的 M6/候选匹配诊断矩阵已完成；此前 615–630 ms 间隔不能再直接称为动画内停顿，旧矩阵约95/55ms历史复核已完成、实际活动归属无法确认（第22节），C1 保持 `[ ]`。本轮按用户要求单独准备 A5 展开/收起与 Hover 手感微调候选；U4–U11 保留待办。当前验收状态及证据边界见第15节。

> 2026-10-12 UI 追加：用户新增8项明确要求，已登记为 Phase 5 的 U4–U11，全部待办；需求映射与截图/执行顺序见第14节。M6按用户最终报告同步为正确性验收通过，保存延迟+12.5 ms继续记录。本次仅更新计划。


> 2026-10-12 更新：M4 封面请求合并及过期任务取消已通过最终候选三轮 Release C 对照、100 首连续切换/四次浮窗重开，以及 owner 共享、取消、显式清空、同曲目换封面和失败重试测试。解码、下载输入、CPU 与封面延迟改善；进程树峰值基本持平、Renderer 峰值略增 3.72%，恢复读数下降。M4 勾选并独立提交；不宣称整体内存峰值收益。见 [M4 验收报告](performance/m4-2026-10-11/REPORT.md)。
> 2026-10-12 M5 更新：迟到 listener、图片超时/销毁、动画及等待任务回归通过；匹配 Release 的 100 次副窗开关对照显示，DOM/监听器保持稳定，句柄增长从基线 +612 降至 +69（增长主要集中在前 10 次测量开关，随后基本平台）。进程树私有提交仍增长约 25–30 MiB，前后相近，故不将其解读为内存回稳；第 6 首图片超时仍未定位。M5 勾选并独立提交。见 [M5 验收报告](performance/m5-2026-10-12/REPORT.md)。

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

此外：设置的跨窗口并发更新/非原子写入（M6）、自动化配置失效（B1）也必须进入计划。EventManager 迟到注册缺口已纳入并通过 M5 验收。

## 3. 动画逐项审查

以下为原始审查基线的发现；当前GitHub动画已采用tweened，最新实际参数、Hover布局问题及A5候选见第15节，不将历史spring描述当作当前源码结论。

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
- **requestAnimationFrame**：主要持续候选是开启 debug FPS；IslandSurface 已无旧封面 250 ms 连续 rAF 循环；RollingDigit 双 rAF 有界，M5 为其增加显式取消。Spectrum 现有 30 Hz capped timer 并非每帧空转 rAF。
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
- `new Image()` 不是天然泄漏；CoverArt 已清除回调。M5 工作区现以共享 `loadImage` 统一处理取色、HD/recovery 与浮窗预加载的成功、失败、超时和 Abort 清理；是否通过仍须以真实 100 次副窗开关和增长对照验收。
- 推荐首先用 **Rust 限尺寸后的缓存文件 + asset URL**；缩略图尺寸按显示逻辑尺寸×DPI 限制，HD 仅大视图需要时请求。评估 WebP 与处理成本，透明度/质量正确后再切格式。
- Object URL 适合 webview 内临时 Blob，不作为跨窗口共享封面身份；必须在旧 DOM/动画不再使用后 revoke。当前 CSP 没有 `blob:`，不能直接替换后破坏图片显示；asset URL、原生路径、data URI 的解析接口也需统一。
- Floating 的 Rust 取色/图片函数接受 data URI/路径，前端 HD 返回的是 convertFileSrc URL：需显式传 cache key/path，而非把 asset URL 当本地文件名。正常主色回调也要校验 track/art revision。

### 生命周期审查结果

| 对象 | 当前已清理 | 剩余缺口/判断 |
|---|---|---|
| App onMount/listen | createAsyncCleanup、interval/DOM listener、capture/mouse cleanup | M5 工作区已把 recovery/HD Image、恢复等待和 placement WAAPI 接入销毁/代次保护；待真实开关对照验收。 |
| IslandSurface 六路订阅 | 所有 subscribe 都有 unsubscribe；matchMedia listener 有 cleanup | 旧 cover rAF 循环已不存在；await tick 后的焦点 DOM 写入增加销毁检查，取消订阅仍不能替代取消待完成动画。 |
| Spectrum store/组件 | refcount、代次、watchdog clear、timer/rAF clear、Rust Destroyed release | 减少动画/系统偏好 gate 传递不一致；恢复策略需 native 健康状态，不能只重试已 running 的 start。 |
| mediaStore/settingsStore | 迟到 listener、初次 snapshot 竞态、disconnect/generation | mediaStore 相同快照仍写新对象；多 webview 有多个独立 store，并非全应用单例。 |
| EventManager | 独立 listener throttle/debounce 状态、off/destroy 清理 | M5 工作区在 await listen 返回后复查销毁状态并立即释放迟到 listener；待延迟注册回归和整体验收。 |
| Floating | DOM listener、Tauri listener、timer、AbortController、pendingImages、refs onDestroy | cover 变空时 Canvas refs 不清；部分请求只丢弃结果不取消；图像绘制/取色缺少最后结果检查。 |
| Timer/Studio/VisibleWindow | interval 清理、late listener 清理、真正隐藏时卸载 | capture CSS hide 不触发卸载；Timer idle 条纹无限动画；预览 clock 1 Hz 对 HH:mm 过频。 |
| RollingDigit/Marquee | matchMedia cleanup、ResizeObserver disconnect、WAAPI cancel | RollingDigit 双 rAF 现记录并取消待执行 frame，写状态前检查卸载；Marquee 在持续布局变化下可能重启，应测并合并测量。 |
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

已完成任务已移至[已完成优化任务](OPTIMIZATION_COMPLETED.md)，本节仅保留待办。历史阶段记录保留原样。

### Phase 0 — Performance Baseline

- [ ] **B0 · 🟠 P1 — Record current Windows performance baseline**
  - **Problem:** 当前提交缺少native Idle CPU/内存、WebView2进程、GPU、动画帧与全部IPC的同条件基线；已有Chrome结果不覆盖这些开销。
  - **File:** `scripts/measure-native.ps1`、`scripts/measure-performance.mjs`、`scripts/measure-retained-memory.mjs`、`src/App.svelte:1191–1215`、`src-tauri/src/event_bus.rs`、`src-tauri/tauri.conf.json`、`docs/performance/`。
  - **Reason:** 无基线无法判断优化收益、工作集波动与真实泄漏，也无法验证“接近0%”和“60 FPS”。
  - **Proposed change:** 经确认后建立专用profiling构建与动态PID采集，补GPU/线程/句柄/IPC bytes/在途数；采第9节全部指标。先执行A、C、D的短复现与资源定位，再保留完整A–F的优化前运行；新指标默认不进入发行版热路径。
  - **Expected benefit:** 得到可比较的Idle、音乐、窗口/封面操作成本与内存归属；明确最先修的可达缺陷。
  - **Risk:** DevTools/trace/频繁snapshot改变结果；CPU归一、共享GPU/WS可能误计。需独立无profiler正式轮和带profiler诊断轮。
  - **Verification:** 3轮同环境、warm-up5分钟，记录所有PID/版本/DPI/歌曲语料。Idle CPU、Private WS、Commit、Renderer/JS heap、GPU和FPS条目均有数据或明确N/A；capture停止时FFT/IPC应可直接核实。当前任务尚未执行。

### Phase 1 — Memory / Leak / Stability

### Phase 2 — CPU / Background Tasks

- [ ] **C2 · 🟠 P1 — Replace permanent polling with demand and events**
  - **Problem:** media1Hz、screenshot20Hz、fullscreen/recording2Hz、cursor远处5Hz长期运行，与可见性/设置不完整关联；media每次重新获取manager/属性，Capture关闭也消耗检测。
  - **File:** `src-tauri/src/lib.rs:175–315`、`src-tauri/src/services/media.rs:554–643,697–765`、`src-tauri/src/commands/window.rs:410–522`、`src/App.svelte:1571–1608`。
  - **Reason:** 常驻程序大部分时间Idle，减少持续唤醒比动画期间微优化更影响后台CPU/电源。
  - **Proposed change:** SMTC manager由actor持有并订阅session/property/playback/timeline变化，低频兜底且只有必要字段查询；Capture检测按偏好开启，能力不支持时退避。鼠标维持自适应近处响应，缓存DPI、远处更低频/合适原生事件；工作区用变化事件+低频兜底。mouse debounce在分配timeout前先gate。
  - **Expected benefit:** 默认/关闭Capture两种Idle下减少线程唤醒、COM调用与事件量，接近0CPU目标。
  - **Risk:** SMTC事件漏发/播放器差异、鼠标穿透与唤醒延迟、截屏快捷键冲突；GetForCurrentView不支持任意录屏，不应通过加频掩盖能力限制。
  - **Verification:** A对照每项poll次数/CPU/线程唤醒，关闭功能后相应计数为0；播放/切歌/应用退出、Win+Shift+S/PrintScreen、fullscreen、4edge唤醒与多monitor不退化。所有低频兜底有上限，事件式listener也有cleanup；F验证锁屏/设备变化恢复。2026-10-16已完成第一批：Capture按三个可检测偏好动态启停，截图关闭后50→500ms、全部原生检测关闭后1000ms且不调用检测API，AppCapture不可用30秒退避；媒体播放/暂停/无会话采用1/2/5秒节奏并保留首次1秒移除确认；普通桌面mousemove在分配debounce前gate；工作区30秒兜底改为5分钟并沿用focus/pageshow/visibility恢复。Rust库87 passed/1 ignored，production前端构建通过；离线缓存缺Playwright，Svelte Check/Vitest未运行。2026-10-20工作区变化通知已接入主窗口独立Win32 subclass：只筛选`WM_DISPLAYCHANGE`与`SPI_SETWORKAREA`对应的`WM_SETTINGCHANGE`，异步合并发给前端复用350ms布局恢复；仍保留5分钟兜底。默认Rust库/既有probe example离线检查、15项既有窗口测试及前端生产构建通过；尚无原生消息投递、多显示器工作区切换或CPU/功耗验证。C2保持`[ ]`：SMTC完整事件/元数据变化覆盖、原生工作区消息与恢复实测、真实唤醒/CPU计数和Capture回归仍待完成；cursor远处200ms不因本批变慢，避免扩大H1首击延迟。

- [ ] **C3 · 🟠 P1 — Make visibility a shared work gate**
  - **Problem:** pageVisible只反映document；主岛isHidden移出屏幕仍可能4Hzclock/30s快照；浮窗capture-hidden为CSS隐藏，MV/旋转/进度仍活跃；VisibleWindow已解决真正隐藏/关闭，不能忽略已有机制。
  - **File:** `src/App.svelte:292–304,639–659,1070–1088,1622–1660`、`src/FloatingWindow.svelte:76–79,848–855,1541–1575,2033,2136`、`src/lib/components/island/VisibleWindow.svelte`、`src/lib/components/island/IslandSurface.svelte:617–652`、`src/lib/components/media/Spectrum.svelte`。
  - **Reason:** “没有视觉变化就不要刷新UI”需要workVisible而不只CSS/DOM visibility；video与composition可在JS闲时仍耗GPU。
  - **Proposed change:** 定义document/native visibility、captureHidden、island state、当前page/hover控件共同决定的需求；停止不可见进度与Spectrum，暂停video/唱片/marquee，恢复时只补一次snapshot。保留Timer绝对deadline和必要capturing/唤醒；不因为后台music正在播放而一直启动不可见Spectrum。
  - **Expected benefit:** 隐藏真正进入低CPU/GPU；减少不显示的Svelte派生、decode与采集。
  - **Risk:** 隐藏期间Timer丢完成、重新显示进度跳错、暂停video后丢失用户意图；捕获隐藏不能恢复泄露内容。
  - **Verification:** playing+capture隐藏/主岛offscreen/minimize/副窗close四种路径分别检查：350ms宽限后无消费者则FFT/event为0、MV暂停、视觉timer停、无持续paint；恢复正确显示当前歌曲/Timer。B/E/F比较CPU/GPU；reduced-motion与showSpectrum关闭也检查。2026-10-20首批仅修FloatingWindow：统一`visualWorkVisible=pageVisible && !isCaptureHidden`门控静音MV、圆盘旋转与可见进度时钟；隐藏时暂停MV并保留节点/时刻，隐藏期间媒体/loadeddata回调不能重启；恢复按当前媒体播放态恢复，play Promise过期后重核当前可见性/节点/销毁状态；隐藏、暂停和compact无高频进度timer，恢复立即按Date.now校准。没有发送媒体暂停命令或改SMTC音频；主岛与Spectrum路径、countdown绝对deadline未改。组件生命周期夹具新增MV竞态和圆盘CSS动画用例，相关浏览器套件6/6、四组Vitest 15/15、Svelte检查0错误/警告、production前端构建3868模块通过。第二批增加`MarqueeTitle active`（默认true），仅当前音乐页在可见展开区间内运行动画，收起/隐藏时取消，恢复重新测量；主岛天气快照效果增加`!isHidden`门控，隐藏时清30秒interval并忽略旧结果，恢复立即调用既有idleApi并沿用30秒缓存。当前/记忆长标题及天气隐藏/缓存命中/过期各有既有IslandFixture/生命周期用例覆盖；生命周期7/7、标题2/2、fixture构建3854模块、Svelte检查0错误/警告、前端生产构建3868模块通过。唯一一次UI detector仅指出未修改IslandSurface CSS的现存样式/布局建议，exit 1；本批未改样式。天气以捕获事件/受控时钟mock验证，不代表原生窗口/网络行为。合并默认feature离线Release已从dirty共享工作树构建：EXE `90A49423B24C569119D6DEC22BA6BE50F5132E280F148538222140C4881405BB`（8,515,072 bytes），DLL `0B72FD68BBED4B58DF48CF0FBDBF1EED8987DB6E437631D21896F70458238465`（116,736 bytes）；前版先归档并核验，构建前/后未发现运行中的`isle.exe`。产物不是干净HEAD构建，未启动/安装。原生Capture provider、真实WebView隐藏/恢复时序及CPU/GPU收益仍待测；C3保持`[ ]`。细节及前端资源哈希见[记录](performance/c3-visibility-2026-10-20/REPORT.md)。
  - **Verification:** playing+capture隐藏/主岛offscreen/minimize/副窗close四种路径分别检查：350ms宽限后无消费者则FFT/event为0、MV暂停、视觉timer停、无持续paint；恢复正确显示当前歌曲/Timer。B/E/F比较CPU/GPU；reduced-motion与showSpectrum关闭也检查。2026-10-20首批仅修FloatingWindow：统一`visualWorkVisible=pageVisible && !isCaptureHidden`门控静音MV、圆盘旋转与可见进度时钟；隐藏时暂停MV并保留节点/时刻，隐藏期间媒体/loadeddata回调不能重启；恢复按当前媒体播放态恢复，play Promise过期后重核当前可见性/节点/销毁状态；隐藏、暂停和compact无高频进度timer，恢复立即按Date.now校准。没有发送媒体暂停命令或改SMTC音频；主岛与Spectrum路径、countdown绝对deadline未改。组件生命周期夹具新增MV竞态和圆盘CSS动画用例，相关浏览器套件6/6、四组Vitest 15/15、Svelte检查0错误/警告、production前端构建3868模块通过。第二批增加`MarqueeTitle active`（默认true），仅当前音乐页在可见展开区间内运行动画，收起/隐藏时取消，恢复重新测量；主岛天气快照效果增加`!isHidden`门控，隐藏时清30秒interval并忽略旧结果，恢复立即调用既有idleApi并沿用30秒缓存。当前/记忆长标题及天气隐藏/缓存命中/过期各有既有IslandFixture/生命周期用例覆盖；生命周期7/7、标题2/2、fixture构建3854模块、Svelte检查0错误/警告、前端生产构建3868模块通过。唯一一次UI detector仅指出未修改IslandSurface CSS的现存样式/布局建议，exit 1；本批未改样式。天气以捕获事件/受控时钟mock验证，不代表原生窗口/网络行为。合并默认feature离线Release已从dirty共享工作树构建：EXE `90A49423B24C569119D6DEC22BA6BE50F5132E280F148538222140C4881405BB`（8,515,072 bytes），DLL `0B72FD68BBED4B58DF48CF0FBDBF1EED8987DB6E437631D21896F70458238465`（116,736 bytes）；前版先归档并核验，构建前/后未发现运行中的`isle.exe`。产物不是干净HEAD构建，未启动/安装。原生Capture provider、真实WebView隐藏/恢复时序及CPU/GPU收益仍待测；C3保持`[ ]`。随后的有限启动检查最终单实例身份确为上述候选（PID [omitted]，`2026-10-20T19:06:34.358991+08:00`）；第一次工具启动实际命中旧 `<local-path> Use后按PID复查已无该进程。物理Escape后未再做任何UI观察或输入，故不称启动UI验收。初始与事后主设置/Run摘要有变化，备份哈希和已知7项布尔设置不变；未手动还原，Run摘要/原因和设置文件内容差异的详细边界见[C3记录](performance/c3-visibility-2026-10-20/REPORT.md)。

- [ ] **C4 · 🟡 P2 — Separate clocks, weather and unseen system metrics**
  - **Problem:** 音乐250msclock每次updateTimeDisplay/Intl；天气30s快照同时刷新CPU/memory/network，即使Island只显示钟/温度；Studio1Hz HH:mm和会话10s不按区域需求；TimerWindow250ms更新秒数字。
  - **File:** `src/App.svelte:292–304,639–659`、`src/Studio.svelte:164–214`、`src/TimerWindow.svelte:70–88`、`src/lib/api/idle.ts`、`src-tauri/src/services/idle.rs:54–89,239–325`、`src/lib/countdown.ts:101–107`、`src/lib/components/island/InfoPanel.svelte:11–13`。
  - **Reason:** 全部可清理不代表频率必要；读了但未展示的系统指标仍有IPC/采集成本。
  - **Proposed change:** 分开minuteClock、visibleMediaProgress、Timer absolute deadline；分钟文字按Date.now()边界刷新，locale/timezone变化及恢复时立即校准；不做全局Intl formatter缓存。只有可见且播放中的music页按250ms更新媒体进度；隐藏倒计时在绝对deadline单次唤醒。天气独立按TTL返回，Studio会话区域可见才刷新；秒边界更新数字。Spectrum颜色仅实际可见颜色消费者需要时解析，避免无消费者时decode封面。
  - **Expected benefit:** 少timer、少格式化/对象刷新，weather Idle没有不需要的sysinfo。
  - **Risk:** 首次CPU/network采样语义与共享collector elapsed会变化；Timer平滑线条可能需要局部动画，不能一刀切破坏观感。
  - **Verification:** A显示只有钟/天气时system采样计数0；分钟不变时clock文字derived不重算；paused/hidden媒体无进度timer；Timer到期准确且UI秒值正确。天气城市变更/失败/TTL仍正确；前后invoke/effect计数与CPU对比。
  - **2026-10-20 第一批实现:** App与Studio天气读取改用独立`get_weather_snapshot` command/`WeatherSnapshot`；服务只读选中地点并复用已有`weather_for`、30分钟天气TTL、请求单飞、按坐标匹配的错误缓存回退，不经过`system_snapshot`或`SYSTEM_COLLECTOR`。旧`get_idle_snapshot`仍返回完整系统与天气快照，前端`getSnapshot`缓存/pending语义未动。新weather API有独立的30秒last-value cache与按地点key共享的pending map（最多4个进行中的不同key）；App保留C3隐藏gate/30秒刷新，Studio城市变化会忽略旧地点结果。Vitest 4/4、Rust idle/weather tests 5/5、Rust offline/locked lib check与现有feature example check、生命周期7/7（fixture确认weather命令被调用且legacy系统命令为0）、fixture构建3854模块、Svelte检查0错误/警告、前端production构建3868模块通过；单次UI detector无发现。未访问实时天气网络、未重建Release。现在运行的PID2396仍是之前构建的C2/C3候选SHA `90A49423B24C569119D6DEC22BA6BE50F5132E280F148538222140C4881405BB`，不含本批C4源码。尚无原生采集/CPU功耗测量；本批不涵盖C4时钟、Studio会话列表、Timer或Spectrum颜色工作，C4保持`[ ]`。详见[天气独立读取记录](performance/c4-weather-snapshot-2026-10-20/REPORT.md)与[C3启动记录](performance/c3-visibility-2026-10-20/REPORT.md)。
  - **2026-10-21 第二批实现:** Studio preview clock移除1秒interval，挂载立即采样后按`Date.now()`计算下一个整分钟timeout，每次回调重对齐并在卸载时清理；原`previewClock`派生仍同时依赖timezone、locale和时间，因此设置变化立即呈现，天气/会话轮询未动。TimerWindow无连续进度环等250ms视觉消费者，改为按绝对剩余时间计算下一个显示秒边界timeout（5500ms约500ms后，5000ms约1000ms后）；暂停、idle、hidden、零时不刷新，快照/可见性变化立即同步，超时按`Date.now()`重算。deadline、完成逻辑与CSS滚动效果不变。生命周期fixture增加timeout记录和Studio/Timer挂载能力；受控时钟Playwright 2/2覆盖分钟边界/卸载、5.5秒相位、暂停/隐藏过期后恢复及卸载无迟到回调。fixture构建3866模块、Svelte检查0错误/警告、前端production构建3868模块通过。未做原生UI/性能测量或Release构建；C2/C3仍开放，C4仍`[ ]`，其余C4验收范围尚未完成。详见[第二批时钟记录](performance/c4-weather-snapshot-2026-10-20/REPORT.md#second-slice-studio-and-timerwindow-clocks)。
  - **2026-10-21 第三批实现:** App取色effect覆盖所有实际可见颜色消费者：compact/hover Spectrum、expanded music Spectrum，以及`showSpectrum=false`但播放中的compact playing-dot；document或主岛隐藏时均不取色。Studio预览同样覆盖可见Spectrum与关闭谱图时播放中的compact playing-dot，只在expanded非music页/hidden模式等无颜色消费者时跳过。复用`extractSpectrumColorsFromImage`现有AbortSignal/`loadImage`路径；门控变化和封面切换会abort，异步结果提交前再核对可见条件与当前封面。两入口均经`VisibleWindow`：该组件在document/native窗口隐藏时卸载内容，恢复后重新挂载；因此Studio分钟时钟依赖onMount立即采样恢复，无独立visibility listener。Studio谱图效果也随卸载取消并在重新挂载时重新评估。未新增取色缓存/解码机制，未改调色算法、Spectrum/FFT订阅或FloatingWindow专用颜色。`showSpectrum=false`时compact playing-dot仍需封面色，故继续提取当前封面；expanded非music页且谱图关闭时无可见颜色消费者，跳过提取。fixture记录取色image-loader超时、canvas和drawImage，排除正常CoverArt展示Image；本批两项Spectrum定向Playwright通过；含第二批时钟回归的合并定向套件4/4，覆盖关闭/隐藏不启动解码、旧请求取消/忽略、恢复后用当前封面采样；fixture构建3867模块、Svelte检查0错误/警告、前端production构建3868模块通过。未做原生UI、CPU/功耗测量或Release构建。C4仍`[ ]`，其余范围未验收。见[第三批频谱取色记录](performance/c4-weather-snapshot-2026-10-20/REPORT.md#third-slice-gate-spectrum-cover-color-extraction)。
  - **2026-10-21 第四批实现:** Studio `.players-section` 以`IntersectionObserver`门控会话列表查询。设置store初始化（含原catch后继续的回退）完成且区域进入视口后立即刷新，并启用原10秒fallback；离开视口清理interval并使在途request generation过期，重新进入立即取当前列表。若浏览器无`IntersectionObserver`，保持此前一直可轮询的兼容行为。刷新请求串行；10秒tick或显式刷新在请求中只排一个后续请求，当前请求结束后仅当区域仍可见且Studio仍挂载才执行；迟到结果不覆盖列表。原选择/离线排序、保存语义、手动按钮及样式不变。fixture支持挂起并完成媒体会话IPC；受控时钟Playwright 1/1覆盖初始区域外无调用、进入立即调用、10秒请求不重叠、离开停止、迟到结果丢弃、重新进入立即刷新、手动刷新及卸载时interval/observer清理。Svelte检查0错误/警告通过。生产构建与包含前两批回归的合并套件见[第四批会话列表记录](performance/c4-weather-snapshot-2026-10-20/REPORT.md#fourth-slice-gate-studio-media-session-polling-by-players-visibility)。未做原生UI、Release或进程操作；C4仍`[ ]`。
  - **2026-10-21 第五批实现:** App分钟文字从媒体/倒计时`clockNow`更新中拆分，独立按`Date.now()`下一个整分钟timeout刷新；设置timezone/locale变化立即重新格式化并重排边界，页面恢复立即强制校准system timezone。分钟不变时由minute key避免重复Intl构造，不新增全局formatter缓存。媒体`clockNow`仅在document可见、主岛未capture隐藏、expanded music页且播放中按250ms刷新；暂停或不可见时取消高频tick。可见倒计时按下一个ceil秒显示边界刷新，hidden/capture-hidden仅在绝对`endsAt`单次唤醒；既有完成effect/`completeCountdown`继续折叠完成状态，timerFinished提示仍按10秒关闭。受控时钟fixture覆盖分钟边界且同分钟formatter调用不增、timezone变化即时呈现、playing媒体250ms启停、capture/document隐藏后恢复当前进度、倒计时隐藏截止及完成一次、卸载清理。合并七项Studio/Timer/App生命周期Playwright 7/7、fixture构建3867模块、Svelte检查0错误/警告、前端production构建3868模块通过。未做原生UI或CPU/GPU/IPC性能对照；不新增Release/进程/配置操作。C4保持`[ ]`。详见[第五批App时钟与倒计时记录](performance/c4-weather-snapshot-2026-10-20/REPORT.md#fifth-slice-split-app-minute-clock-media-progress-and-countdown-deadline)。

### Phase 3 — IPC / Geometry correctness

- [ ] **I3 · 🟠 P1 — Publish one media snapshot authority and lossless deltas**
  - **Problem:** 媒体 actor、App relay、Floating 与 Studio 必须对齐同一原生 snapshot 顺序；原生运行与跨窗口性能尚未验收。App→Floating ready-consumer gate 与 Timer deadline fanout 已完成源码切片，仍需原生场景和性能对照。
  - **File:** `src-tauri/src/state/media_actor.rs`、`src-tauri/src/commands/media.rs`、`src-tauri/src/event_bus.rs`、`src/App.svelte`、`src/FloatingWindow.svelte`、`src/lib/mediaStore.ts`、`src/lib/mediaArtwork.ts`。
  - **Reason:** 以 actor 的 `{nativeEpoch,nativeRevision}` 为原生顺序依据，覆盖每个实际 consumer，并保留完整快照、封面 delta 与 App 本地 overlay 的边界。
  - **Proposed change / status:** 已完成 actor-owned snapshot/stamp、App stamp relay 与 mount-session/overlay sequence fence、Floating raw/command/relay revision gate、Studio `mediaStore` 每连接 revision gate、初始 snapshot/封面 delta 保护、ready 实例 gate 与关闭/隐藏/卸载暂停及 full replay；Timer 改为变化时发送状态和绝对 deadline，Timer 窗口本地按秒边界显示、隐藏暂停并在恢复/重开时请求当前状态。剩余为原生 SMTC/多窗口验收与 IPC/封面性能对照。协议边界是 Tauri emit 不代表 listener 已消费，且已 dispatch 的旧 relay 可在新 App session 声明到达前先应用。I3 保持未勾选，详细分片与证据见专项审计计划。
  - **Expected benefit:** 更低全应用IPC/bytes、更少重复HD工作、同歌补图正确，跨窗状态不漂移。
  - **Risk:** App session 声明到达前的旧 relay 窗口、旧版 unstamped 初始兼容，以及原生采样/事件节流与真实接收时序的差异；不能把 emit 接受当成消息送达确认。
  - **Verification:** 前端 mediaStore/mediaArtwork/countdown 单测、Studio 与 Timer 生命周期 fixture、Svelte 检查和前端构建通过。原生快速切歌/空会话/多窗口行为及 IPC bytes、高清封面 resolver 对照仍待验收。
  - **Source audit / acceptance notes:** [I3 media snapshot audit](performance/i3-media-snapshot-audit-2026-10-21/PLAN.md) — 已完成方案、实现切片与测试证据；剩余项为原生/性能验收及上述 delivery 边界。Rust 500ms leading-drop 仍可能把媒体重试延后到 actor poll。I3 remains unchecked.

- [ ] **I3-NCM2 · 🟡 P2 — 网易云进度来源设置与性能对照（2026-10-21 用户新增）**
  - **需求：** 设置页提供“真实进度（同步更准确）”与“计时估算（减少后台读取）”，让用户依据实际资源消耗和同步准确度选择。首个实现切片已在工作树接入 Rust/TS 设置、Studio 选择、actor 调度切换和可用性/未知锚点提示；默认保持 `real` 仅为旧行为兼容，不代表性能推荐，发布验收仍未通过。
  - **当前事实：** 当前候选已接入网易云 CDP 真实进度采样（播放约1秒、暂停约5秒、失败退避），采样间仍需本地插值；不能把它描述成完全不读播放器的纯计时器。主岛滑条拖动已由用户实机确认；此前临时±15秒快捷控件已按用户要求移除，不属于当前验收项。
  - **模式契约：** 真实模式以播放器实际位置/时长为依据，纠正外部拖动、暂停继续和切歌；插值仅使显示平滑，不冒充真实采样。计时估算模式关闭周期 CDP 进度读取，按最后确认位置、播放状态和经过时间估算，明确外部跳转可能不同步；用户主动 seek 的控制与必要确认读取不受此开关禁用。
  - **设置与兼容：** 首片已复用 Studio 的设置 patch、持久化和 `settings-updated` 全量广播；actor 读取原子 latest desired mode，有限队列消息只用于唤醒，因此队列满/快速连切不丢最后已保存选项。Estimate 停止周期 CDP 进度读并失效旧结果但不提前释放 in-flight gate；Real 重入清退避并在下一 actor refresh 发起一次读取。真实接口失败时状态会广播为 unavailable 并给用户手动回退提示；新曲无真实锚点时明确标注未知。Studio patch/persistence/broadcast fixture passed. Native actor switching and true multi-WebView consistency remain pending. 默认选项待性能/准确度对照后决定。
  - **性能对照（待测）：** 使用同机器/客户端版本、歌曲、播放状态、窗口可见性、频谱与封面配置，分别测试播放、暂停、隐藏/浮窗关闭及外部 seek；每组预热30秒、采集至少60秒并重复3次，记录环境和测量方法。比较 Isle 全进程树 CPU（单核与整机口径分列）、工作集/私有内存、CDP 读取/连接次数、invoke/event 数量，以及同步延迟与位置误差；网易云开启调试接口的额外开销单独对照，不混入 Isle 收益。未测项标“待测”，不预填省电百分比或零开销结论。
  - **用户选择：** 设置页用准确度与资源取舍的简短说明，附已测条件/版本和结果摘要；不放内部模块或协议细节，不常驻运行性能测试。只有数据支持后才标注哪种模式更省资源。
  - **验收：** 两模式切换、持久化、多窗口一致、暂停继续、滑条 seek、切曲、关闭重开与接口不可用均正确；估算模式无周期 CDP 读取，真实模式保持单在途、切曲丢弃旧结果及失败退避；发布可复核性能记录后再勾选。关联 [I3 专项记录](performance/i3-media-snapshot-audit-2026-10-21/PLAN.md)。

- [ ] **I4 · 🟡 P2 — Bound action IPC and instrument the complete transport**
  - **Problem:** 音量写入现有串行 latest 队列已补相同设备/整数去重，活动 `VolumePanel` 已在 pointerup 提交终值；仍无实际频率上限。App 已修复内置 `settings-changed("monitor_index")` 大小写不匹配导致的重复 `get_settings`；camelCase 和未知字段兼容保留。其余 full/field settings consumers 经调用链核对多为不同副作用。`spectrum-data` 直发绕过 EventBus stats。浮窗创建路径已加单航班，真实 Tauri 并发窗口行为仍未验收。
  - **File:** `src/App.svelte:580–613`、`src/lib/components/volume/VolumePanel.svelte:49–80`、未发现调用点的 `src/lib/components/volume/ArcVolumeControl.svelte`、`src-tauri/src/event_bus.rs`、`src-tauri/src/audio.rs:77–99`、`src-tauri/src/commands/settings.rs`、`src-tauri/src/commands/window.rs:1351–1435`。
  - **Reason:** 先保留已有串行队列并补齐活动输入路径；诊断不完整会把局部计数下降误当成全传输收益。
  - **Proposed change / status:** 音量首片已完成：`LatestValueWriteQueue` 保留 single-inflight/latest pending、按设备与整数去重，失败时保留并发送更新的 pending 值，设备切换开始时丢弃旧设备 pending 并暂停拖动输入；pointerup 使用最后坐标，cancel 不提交伪终值。原始队列/组件验证及本次新增的失败后 pending 回归均通过，诊断 fixture 的重复整数 queue 只触发一次模拟命令。浮窗现由 Tokio async mutex 串行化 label 检查/build 与既有 show/focus/动画，build 失败传播并释放锁；seam测试3/3，指定工具链 offline/locked lib test、lib check、probe example check均通过。该代码未做真实 Tauri 窗口验收。Settings 最小修复已完成：内置 snake `monitor_index` 不再触发重复查询；保留 camelCase 与未知 legacy fallback，后者复用 disposed publish guard。两项 App Playwright 覆盖 producer 顺序、store/placement、兼容分支和卸载/迟到响应；`pnpm check` 0 errors/warnings。该修复依赖内置 commit 随后发 full snapshot，不承诺第三方 snake-only 事件。累计集成复核还为旧 App session 的 `FLOATING_MEDIA_NOT_READY` 增加 session fence，并避免音量失败恢复读取与设备切换交叉；本次 `pnpm check` 0 errors/warnings。Studio 启动三次 font/lang DOM 写入低优先级未动。音量仍没有固定频率上限。现新增默认关闭、session-only 的四条前端 route 聚合计数：media-sync producer/consumer 与无消费者门控、Timer 状态发送/接收、region queue 与命令耗时、volume queue 去重与命令耗时；只统计调用数/结果/耗时，不统计 bytes。没有产品 UI、持久化、跨窗诊断事件或 Spectrum/Rust/native/Tauri 内部覆盖。诊断单元与 3 项生命周期 fixture、`pnpm check`（0 errors/warnings）及 3,872 模块前端 build 通过；disabled-path 开销未量化，不声称总 IPC 或性能收益。COM endpoint 复用暂缓，Arc 控件未发现可达调用点。I4 仍未勾选，剩余边界与证据见 [`i4-action-ipc-audit-2026-10-21/PLAN.md`](performance/i4-action-ipc-audit-2026-10-21/PLAN.md)。
  - **Expected benefit:** 减少重复整数对应的音量命令、拖动终值遗漏和每次显示器偏好保存的冗余设置查询，并为后续完整传输测量划清口径。
  - **Risk:** 错误去重可能丢掉返回在途值；pointercancel 不应被当作成功终值；设备 endpoint 缓存需处理 COM 所在线程和切设备失效。
  - **Verification:** 音量定向测试、Svelte和前端build通过；浮窗 seam/Rust checks通过。Settings修复的2项 App Playwright、Svelte和前端build通过；未做真实原生显示器/窗口验收。原生窗口/音量行为及含`spectrum-data`直发的完整统计仍未验收。I4保持未勾选。

### Phase 4 — Animation Engine

本阶段任务已按2026-10-24用户验收决定全部关闭，详见[已完成优化任务](OPTIMIZATION_COMPLETED.md)。

### Phase 5 — UI / Rendering

U1–U11已按2026-10-24用户验收决定全部关闭，详见[已完成优化任务](OPTIMIZATION_COMPLETED.md)；以下需求说明保留为历史。

#### 用户追加 UI 修复（U4–U11，2026-10-12）

以下八项来自本轮用户明确要求，验收状态按各条记录维护。它们与 U1–U3、I2 协调执行；发生冲突时以本轮要求为准。源码切片可先记录，但只有完成各条要求的测试与同条件验收才勾选完成。

### Phase 6 — Architecture Cleanup（仅有收益时）

- [ ] **R1 · 🟡 P2 — Extract owners only where update scope and coupling improve**
  - **Problem:** App集中多个服务与大量props/effects；Media、clock、settings变化可影响不相关derived/placement检查，但不能用行数推定整树rerender。
  - **File:** `src/App.svelte`、`src/lib/components/island/IslandSurface.svelte`、`src/lib/islandStore.ts`、`src/lib/mediaStore.ts`、`src/lib/settingsStore.ts`、`src/lib/api/`。
  - **Reason:** 先修生命周期/IPC/动画后再划分owner，能避免两份状态、重复订阅与控制器相互循环。
  - **Proposed change:** 依据第8节effect计数/trace逐个提取Animation、Placement、Media、Audio/Weather等确有职责边界的owner；Timer保留唯一deadline；将clock高频更新局限到进度组件，稳定props与derived。只转发调用且不会缩小更新范围的Window/IslandController不创建。
  - **Expected benefit:** 易profile、依赖清楚、低频/高频更新范围分离，可测重复计算减少。
  - **Risk:** 多store/多订阅增IPC，异步启动顺序改变，拆分后毁坏原有untrack/cleanup。
  - **Verification:** 拆前后effect/derived次数与B/D帧时对照；每owner启动/释放次数有界，listeners/commands不增；相同业务与native回归通过。不以文件行数或“更整洁”作为性能验收。

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

**当前执行状态：B1、M1–M6已完成对应验收，M6保存延迟+12.5ms仍保留。C1 worker实现、Apple媒体能力/控制错误子任务及资源测量链路已验收；第4批旧95/55ms原始复核完成，实际活动归属无法确认，撤回动画内停顿结论。声明支持会话有效seek为N/A、当前媒体修复Release匹配性能未验收，C1 `[ ]`。A5已应用Windows工作树、10/10回归与旧保存Release诊断完成，第5批原生交互取证发现四edge/切换首次命中未通过，焦点可见性/手感及成本/整体性能仍未验收，A5 `[ ]`。U2已补展开页播放能力禁用，U3已移除Timer预设条纹常驻动画、尊重减少动态偏好并临时启用移动封面的will-change，U5已实现分模式外观设置及旧值迁移，U6已移除两项浮窗背景设置，U11已移除Compact封面旋转；A1/A3加入迟到手动隐藏保护，A2改为运动期间才申请封面合成提示，A4加入过期页面焦点保护，U7增加中性按钮底色且保持尺寸比例，U9修复浮窗文档根透明背景，U10移除展开态蓝色渐变。A1/A3真实App handler竞态、A4页面焦点反例、U7样式/矩形、U5显式新值与缺失值继承均有浏览器或定向测试覆盖；U6既有serde兼容测试覆盖旧字段忽略。这些源码切片仍需各自原生/完整生命周期或性能验收，未勾选。U1/U4/U8缺少最终排版/截图/DPI样本；C2现场媒体会话测试依用户选择暂缓。C2、真实SMTC/网易云切曲与模式、原生首击H1、TA–TF长测和DPI/原生region验收需要现场或受控机器数据，保持待办。2026-10-24最终离线locked custom-protocol Release构建成功，前端28资源全部出现在当前EXE依赖文件且资源manifest SHA-256为`41654E2BFC82F4F1369EE8FB03FB30E303C5B2D70E464B7425FE3634A65746DD`；EXE SHA-256 `506DE69715876C60A72CE8DCF900FB5EDC7A7ED864F608DCAE3AED994215055A`（23,468,544 bytes），DLL SHA-256 `483D0C5DBDC7649021E92125EE3395FB58427B114F92303E559DB39129F41BC5`（116,736 bytes）。未启动、安装或部署。详情见[最终构建记录](performance/m3-final-build-2026-10-24/REPORT.md)。B0完整基线、跨屏DPI、长期GPU/内存、第6首超时和约25MiB增长仍待验证。第9批代码/构建已独立验收，第10批静态lab工具仅离线准备、工具待审/现场待允许，首击仍未知。最新边界见第28节。**

## 14. 2026-10-12 用户追加 UI 需求与执行边界

本节登记八项新要求；任务定义在 Phase 5 的 U4–U11。它们均保持待办，未实施、未验收。原始审查中的候选旋转、背景取色和共享模式外观方案如与本节冲突，以用户最新要求为准。

### 需求映射

| 用户序号 | Task ID | 优先级 | 验收目标 |
|---|---|---|---|
| 1 | U4 | 🟠 P1 | 实际贴边轮廓、裁剪、内容布局与设置预览一致，原生命中正确。 |
| 2 | U5 | 🟡 P2 | 主灵动岛 floating/edge 外观参数独立保存，切换及重启不互相覆盖。 |
| 3 | U6 | 🟡 P2 | 两个颜色设置入口及其生效逻辑移除，旧配置兼容，浮窗固定黑色底色。 |
| 4 | U7 | 🟡 P2 | 按最终显示尺寸增大按钮命中面积，统一中性背景和交互状态。 |
| 5 | U8 | 🟡 P2 | 播放器同一行文字、时间与图标对齐，保留歌名/歌手层级。 |
| 6 | U9 | 🟠 P1 | 独立悬浮播放器外边圆角在真实 Windows 环境恢复。 |
| 7 | U10 | 🟡 P2 | 灵动岛表面 #000000，预览与程序均无蓝色渐变。 |
| 8 | U11 | 🟡 P2 | Compact/Hover 专辑封面始终静止，保留展开/收起位移与缩放。 |

### 截图与源码依据

- 已查看用户附件 image(1).png（385×457）：Studio 贴边音乐预览，状态文字显示“展开 · 上 · 600×249.1 · r45”，播放器底部可见深蓝背景层。
- 已查看用户附件 image(2).png（340×145）：真实桌面中的贴边播放器。与预览的圆角、边界及底部留白表现不同；两张图缩放/裁切和显示内容不同，不能直接以像素差确认具体根因。
- U4–U11的截图/文件定位最初依据 `2d915d5`；2026-10-12本轮核对GitHub main为 `6ef0855cdb187ea8de4780119f89df12a640f88f`，提交标题为“perf: offload blocking native work (C1 partial)”。这只是部分实现上传，不是C1验收提交；执行前复核实际运行构建及未提交改动。
- 悬浮窗圆角问题来自用户报告；本轮两张附件主要展示主灵动岛，不冒充已取得浮窗角部失败截图。
- 每项执行记录补充 native 构建 SHA、设置快照、模式/状态、显示器、Windows scaleFactor、renderer DPR、实际截图和点击区域。DPR emulation 不能替代真实 Windows 跨屏 DPI 验收。

### 建议执行顺序

默认继续保留 Phase 0–7 的总顺序，每次只处理一个确认的任务/明确阶段。U4 与 I2 使用同一几何修复，U5 依赖 M6 的设置事务，不重复重构已经完成的资源所有权工作。

若用户选择先修现有显示问题，建议首先 U4（实际贴边 UI），随后 U9（浮窗圆角），再 U5（模式参数分离），每项单独验收和提交。其余顺序建议为 U6（移除颜色选项）、U10（纯黑底色）、U7（按钮）、U8（文字对齐）、U11（取消收起旋转）。

| 小阶段 | 包含任务 | 范围与依赖 |
|---|---|---|
| 显示正确性 | U4、U9，逐项执行 | 先修实际窗口表现，再调整视觉；不同时更换动画引擎。 |
| 外观参数 | U5，随后 U6 | 模式独立与旧配置迁移复用 M6；颜色入口删除完成后再确认最终背景。 |
| 静态视觉 | U10、U7、U8，逐项执行 | 在统一底色与正确几何上校正按钮和字体，保持轻量层级。 |
| Compact 装饰动效 | U11 | 只取消收起封面旋转，保留既有共享封面过渡。 |

上述顺序是UI阶段后续建议；U4–U11本轮仍只登记。用户本轮另行要求的A5动画微调是独立候选，不与UI重做或C1补测混为一轮。代码优化继续遵守“修改 → 相关测试 → 同条件前后数据 → 回归确认 → 勾选 → 独立提交 → 停止”的原门禁。

### 本次计划同步记录

M6 状态按用户在 2026-10-12 提供的最终验收报告同步：61 Rust 测试通过、0 failed、1 ignored，95 前端测试通过，Svelte 零错误/警告；并发事务、失败补偿、备份恢复及真实 Studio 保存失败/重试/关闭拦截验收通过。保存延迟四样本中位数 27→39.5 ms（+12.5 ms）继续记录，不能把低于 40 ms 当作事先设定门槛；100 次相同 payload 写入的原子写入约 7.62 ms/次属于独立合成测试。

此前UI计划追加时未收到M6独立提交SHA；本轮按后续C1报告记录Windows基线 `48530f7`，不填造远端完整SHA。U4–U11仍没有代码改动、测试成绩或性能收益声明。B0 完整基线、约 25 MiB 私有提交增长、GPU/内存长期稳定、真实跨屏 DPI 及第 6 首超时继续保持原验收边界。


## 15. 2026-10-12 C1 诊断复核与 A5 动画微调候选

### 当前版本与证据

- 本轮核对GitHub main：[6ef0855](https://github.com/alent17/Wind0ws_Dynamic_island/commit/6ef0855cdb187ea8de4780119f89df12a640f88f)，标题“perf: offload blocking native work (C1 partial)”。其已提交的C1报告仍是早期615–630 ms矩阵，不包含用户本轮提供的新对齐结果。状态始终是部分验收，不因上传而勾选。
- M6 Windows基线 `48530f7` 由用户和C1报告提供；本会话GitHub按此短SHA查询未解析成功，因此不补造完整SHA或将其当作远端main。
- 旧r4矩阵在第4批已取得本机完整CSV/status/bounds/matrix/progress并校验、复算；初始用户文字报告被原始证据复核取代。暂停/33%和窗口DISPLAY2有记录，但实际活动末端未保存，CSV无逐帧QPC/date转换来源，不能称精确动画对齐；详见第22节。
- [PresentMon官方字段说明](https://github.com/GameTechDev/PresentMon/blob/main/README-ConsoleApplication.md)：MsBetweenDisplayChange是前帧的显示持续时间，长间隔可以跨过动作起止/静止区间；它不等于某个线程的阻塞耗时。JS rAF节拍和draw-complete也不是compositor实际呈现。200 Hz刷新周期为5 ms；产品60 FPS目标仍对应约16.67 ms，测量报告须同时保留实际刷新率。

### C1 最新结果：保留 `[ ]`

| 场景/现象 | 最新记录 | 对验收的含义 |
|---|---|---|
| 历史r4矩阵 | 本机原始CSV/status/bounds/matrix已取得、哈希与源行复核完成 | 无逐帧QPC或确证date→UTC映射，不能写成完成了帧QPC对齐。 |
| 动画样式快照 | 约343–374ms为保存起点→100ms稳定确认快照跨度 | 无精确last mutation，不能当纯活动span。 |
| 旧第15次615–630ms | 旧宽窗数值保留，连续活动归属未成立 | 无新栈/精确边界证据，不声称已修复。 |
| C1动画独立95ms | 源CSV第669行/动作13，95.0015ms，PID [omitted] | 旧分类可复算但归属无法确认；条件分析落近似确认尾段，不是确证。 |
| C1并发55ms | 源CSV第1687行/动作14，54.9938ms，同PID/swapchain | 同上；不能称actual animation内停顿。 |
| ETW栈 | 历史WPR失败，缺相关线程栈 | 仅当后续正确对齐确有活动停顿才最小采栈，不自动提权。 |
| Apple播放器 | 第2批能力/错误、播放/暂停/时间线和只读入口独立验收通过 | seek=false为真实不支持能力；声明支持会话上的有效seek为N/A，不盲测Apple。 |
| 资源链路 | 第3批就绪/CPU区间及旧A5配对独立验收通过 | 新媒体修复Release性能仍未验收；单轮不同覆盖不推断收益。 |

旧95/55ms只读复核已经完成，以“证据不足，活动归属无法确认”关闭本批复核，不将它无限留成待归因。最小下一步由发布聊天另行安排：当前实现的匹配新构建、分离首/末/确认并有确证帧时钟锚点的有界性能轮；如确有真实活动停顿，再对应线程栈。另补声明支持seek的真实会话及A5原生交互/成本门禁，清单见第22节。原150ms同步等待迁移已完成，C2保持待办。

### A5 本轮范围与参数

用户本轮明确要求调整交互手感，因此独立准备小范围候选；不是提前完成A1控制器重构，也不改变C1已采集的基线。此候选最初基于GitHub `6ef0855`的独立工作树。2026-10-12已将附件补丁应用到当前Windows工作树，并完成同C1实现、仅A5不同的Release配对。业务差异仅在 `islandMotion.ts` 和 `IslandSurface.svelte`；两个UI夹具/回归文件用于鼠标路径与连续性验证；补上Hover内容随壳布局、避免误开移动封面以及Hover期间暂停端点测量。Rust/native、图片预算及颜色/字体设置不在此候选范围。

| 动作 | GitHub原实现 | A5候选 | 曲线/说明 |
|---|---:|---:|---|
| Compact/Hover → Expanded | 240 ms | 420 ms | cubic-bezier(0.25,0.1,0.25,1)，较缓起步和较长推进。 |
| Expanded → Compact/Hover | 240 ms | 360 ms | 同上，收起略快于展开，但仍慢于原实现。 |
| Compact ↔ Hover | 实际240 ms；定义140未使用 | 240 ms | 独立cubic-bezier(0.3,0,0.2,1)，起始斜率0，避免借用陡峭settle曲线。 |
| Page/设置/style布局变化 | 240 ms | 240 ms | 保留原settle曲线，避免顺带放慢所有动作。 |
| 动画关闭/reduced-motion | 直接终态 | 直接终态 | 不强制用户等待新的420/360 ms。 |

以上是源码目标duration，不保证实际端到端时长或物理呈现节奏。App手动隐藏本来从collapseDuration读取等待，候选同步延长该等待；固定等待不等于实际settled，此限制由A1/A3后续处理。捕获紧急隐藏仍按原路径处理。

### A5 工程验证与验收边界

隔离候选环境中修改前后均为95项Vitest通过、Svelte零错误/警告；候选production build通过。原实现7项动画UI回归通过；最终候选10项通过，包含原7项及较缓展开、真实指针Hover和Hover未结束时展开的3项新增回归。补丁应用到当前Windows工作树后，production build通过；使用包入口及工作树临时目录复跑，Vitest 95项通过、Svelte Check 0错误/警告。Windows Playwright 本地服务已 ready、Chrome 可独立启动，但受限执行环境连接本机端口时报 `EACCES`；在开始菜单打开的普通非管理员 PowerShell 中运行同一测试命令后10/10通过。此结果确认当前 Windows 工程回归可运行，但没有确定执行环境拒绝本地 socket 的具体策略。浏览器夹具验证布局/连续性/动作时长，不含Windows/WebView2/native interaction region，也不作为60 FPS或CPU/GPU验收。


### A5 匹配浏览器样式采样（非原生性能验收）

Linux / Chromium Headless Shell 134.0.6998.35 / 相同锁文件和Vite开发夹具，viewport 1280×760、no-preference、edge/top、length240、scale0.8、toolbar开启、播放暂停、Spectrum关闭。两侧各5个连续循环，先原实现后最终候选，每循环 Hover进入→离开→展开→收起；不是正式三轮Release性能比较。采样只读inline style和settled属性，不在帧循环调用getBoundingClientRect。`约60 ms进度`取第一个不早于60 ms的样本，常落在约66 ms；不是严格同一毫秒的呈现测量。

| 动作 | 原实现command→settled中位数/ms | 最终候选/ms | 原实现约60 ms归一化进度 | 候选约60 ms归一化进度 |
|---|---:|---:|---:|---:|
| Hover进入 | 266.7 | 266.6 | 69.5% | 23.8% |
| Hover离开 | 266.5 | 266.7 | 69.5% | 24.0% |
| 展开 | 266.6 | 449.9 | 69.6% | 12.5% |
| 收起 | 266.7 | 383.3 | 69.6% | 16.1% |

该数据确认候选动作变慢、Hover早期进度更缓；不证明更高FPS、更低CPU/GPU或原生呈现长尾已解决。command→settled含首次调度和完成回写，不能与420/360/240配置值或Windows343–374 ms新探针直接混比。原始样式样本在隔离工作树的 `dist/animation-ui/before.json`、`after-final.json`；早期仅曲线候选的 `after.json` 不作为最终候选数据。

首次测试工具准备中，默认浏览器下载不完整；另行取得的Chromium155在加载页面前崩溃，两侧都无法启动。正式工程回归和上述 Linux 样式采样统一使用可正常运行的Chromium134，前端锁文件与项目依赖未改变；上述环境启动失败不计作产品回归通过，也不隐去。随后 Windows 普通用户终端的 Playwright 回归 10/10 通过，具体本机端口访问限制原因仍未知。

A5仍保持 `[ ]`，当前改动未提交、未推送。Windows 普通非管理员终端的10项Playwright回归通过；同C1实现、仅A5不同的三轮Release配对已完成。先前报告的7/5条 >50 ms“完全落在样式动画内”未经正确边界确认：结束时间实际是100 ms稳定等待后的确认快照，现有分类不能证明这些属于动画卡顿。复核将旧区间拆为确认前、近似确认尾段与跨越该尾段后，候选动画独立7条全部跨越不确定尾段；候选并发5条中1条落在近似确认尾段、4条跨越该尾段，没有一条能据此确认发生在实际动画过程中。旧轮未采集完整最后样式变化时刻；分离时间戳的有界补采及并发复核见第17–18节。PresentMon窗口P95相近，但不能据此排除长尾；缺少ETW等待栈和逐次交互区域IPC计数，也未完成完整原生视觉/交互矩阵。详见 [A5 Windows Release验收报告](performance/a5-2026-10-12/REPORT.md)、[汇总 analysis.json](performance/a5-2026-10-12/analysis.json) 与 [analyze.mjs](performance/a5-2026-10-12/analyze.mjs)。更长动画可能增加现有逐帧region发送总量，需补IPC和CPU数据；4edge、floating/edge、快速进出与反向、手动隐藏、封面/焦点及低动效仍待原生验证。C1继续保持 `[ ]`；95/55ms旧间隔已复核为归属无法确认；Apple能力/错误路径已通过，支持会话的有效seek为N/A（第20–22节）。

本轮未实现U4–U11、C2或其他阶段；~25 MiB增长、GPU长期回稳、实际跨屏DPI和第6首图片超时均未解决。


## 16. 2026-10-12 A5 统计边界复核与后续

本轮检查了 A5 分析器、joined JSON、每轮矩阵记录及本机保留的 PresentMon CSV。没有修改动画业务代码或勾选任务；C1、A5仍为 `[ ]`。已将 `analysis.json` 与 `analyze.mjs` 纳入 GitHub 文档目录，以便审阅当时的输出和统计实现。完整输入 CSV 暂未上传。

### 统计口径核查

矩阵记录器先以第一次 inline-style mutation 作为起点，然后每10 ms左右轮询页面；只有在目标 expanded 状态成立且 `now - lastChange >= 100 ms` 时，才把当时 snapshot 记为 `animationEnd`。因此 `animationEnd.at` 是“稳定100 ms后的确认时刻”，不是最后一次样式变化，也不是实际动画终点。旧分析器使用 `animationStart.at` 到 `animationEnd.at` 建立动画区间，包含至少100 ms的确认等待与轮询延迟；旧的7/5统计不能解释为已确认的动画卡顿。

以下为对旧“完全在区间内”的 >50 ms PresentMon 项目按 `confirmation time - 100 ms` 做的保守边界拆分。该拆分是快照近似，只确定记录相对于确认尾段的大致位置，不证明尾段之前仍在物理动画中；最后样式变化及通过条件时刻没有保存。

| 场景 | 构建 | 确认尾段前 | 完全落入100 ms确认尾段* | 与该尾段重叠 |
|---|---|---:|---:|---:|
| Animation only | A5-off | 0 | 0 | 0 |
| Animation only | Candidate | 0 | 0 | 7 |
| Combined | A5-off | 0 | 3 | 1 |
| Combined | Candidate | 0 | 1 | 4 |

`*` 确认尾段按快照时间减100 ms近似；记录器没有保留精确的 lastChange 与通过条件时刻。故候选 animation-only 的7项均跨越边界不确定尾段；combined 的5项中1项完全落在近似确认尾段，4项跨越该尾段。它们都不能计为已证实的动画内卡顿。之前的区间总数也不是失败动作数。P95相近不能替代端点核验。

原始旧轮 PresentMon CSV、状态文件、窗口采样和每轮 Playwright JSON 仍在本机 `dist/performance/c1-2026-10-12/`。旧 joined `analysis.json` 未保留全部逐帧行或 last mutation，故该旧轮只能按上表作历史近似拆分；不能从中重建精确动画端点。文件与新补采细节见 [A5 Windows Release验收报告](performance/a5-2026-10-12/REPORT.md)。

### 下一步（已完成有界补采）

已用同一保存的 baseline/candidate Release 二进制各完成一轮、每轮16次 animation-only 转换；没有重建或修改动画业务代码。首轮发现 PresentMon `TimeInDateTime` 与同轮UTC戳表面相差16小时，按记录的UTC+8换算后仍相差8小时，原因未定，故该轮作废并改用 `--qpc_time`。PresentMon QPC 以窗口探针请求8ms间隔采集的 Stopwatch-QPC/UTC 锚点对齐；时间区间将 `MsUntilDisplayed` 加到 Present 起点，再由 `MsBetweenDisplayChange` 回推区间起点，并按区间相交筛选边界帧。脚本、完整原始 CSV、状态、窗口采样及逐次 first/last mutation 和确认时刻均保留在 `dist/performance/a5-boundary-2026-10-12/`。

两版均唯一映射到主岛 HWND 对应的 temporal swapchain：baseline `0x2764A1A3230`，candidate `0x24B251BB1F0`，每版16/16个转换均有present；HWND在 `\\.\DISPLAY2`。在first→last style-mutation活动区间，baseline/candidate没有完全落入区间的 `>50 ms` 显示间隔；两版分别有16和17条 `>50 ms` 间隔跨越动画/确认边界，baseline另有1条落在确认尾段。这个单轮采集使旧7/5条不再能被表述为确认的动画内卡顿，但样本不足以排除偶发停顿或比较性能收益。PresentMon缺少显示时刻/区间的行分别为39/2和25/2，不能纳入区间分类。PresentMon不提供HWND，此swapchain归属是依据采集窗口时间内的唯一呈现相关性，不能写成直接字段映射。

C1旧95/55ms历史复核结论见第22节；Apple能力/错误验收已通过，支持会话有效seek及必要时真实活动停顿栈继续待验证；不得单凭MsCPUBusy/GPU time归因。C1、A5均保持 `[ ]`，本轮不勾选、不提交、不推送。

### Spring迁移延期

按用户本轮提供的计划，展开/收起/Hover的Spring迁移暂缓，待现有计划阶段完成后再独立评估。A1如需工作，只统一现有动画所有权、状态和完成契约，不借此提前迁移Spring；其余隐藏或原生窗口spring只按已确认的具体缺陷处理。所有既有勾选状态保持不变。

## 17. 2026-10-12 A5 PresentMon 显示时刻修正与配对复采

本节更新并取代第16节“下一步待补采”的状态。C1、A5仍保持 `[ ]`；本轮只改测量脚本/验收文档，没有改动画业务代码、重建或提交二进制。所用保存构建哈希仍为 baseline `00AE8D28…` / DLL `A2451332…`，candidate `0F1D5746…` / DLL `754349D1…`。

修正后的脚本用 PresentMon `--qpc_time` 记录 Present 起点，并将每个 `TimeInQPC` 依据窗口探针同步采集的 QPC/UTC 锚点（请求8ms间隔，实际锚点间隔另记）映射到基准时钟。显示时刻按 `PresentStartUtc + MsUntilDisplayed` 推导，显示区间起点按 `displayedAtUtc - MsBetweenDisplayChange` 推导；样本选择使用区间与[first mutation,last mutation/confirmation]窗口相交，因此包含显示终点落在窗口之外的跨界间隔。首轮使用 `TimeInDateTime` 的尝试与脚本/浏览器UTC戳表面相差16小时，按UTC+8换算后仍相差8小时，原因未定，已作废；正式统计不用它。依据PresentMon源码，`TimeInDateTime`来自Present起点且必须加`MsUntilDisplayed`才是显示时刻；本机时间换算异常后，QPC加同源采样锚点更适合本轮对齐。完整 PresentMon CSV及SHA、状态、QPC/HWND/显示器采样、动作逐次 JSON均保留于本地 `dist/performance/a5-boundary-2026-10-12/`，分析器输出 `paired-analysis.json`。

| 一轮匹配采集 | 主岛 swapchain | 区间样本 | `>50 ms` | 完全在样式变化区间 | 完全在确认尾段 | 跨动画/确认边界 |
|---|---|---:|---:|---:|---:|---:|
| baseline | `0x2764A1A3230` | 780 | 17 | 0 | 1 | 16 |
| candidate | `0x24B251BB1F0` | 1,272 | 17 | 0 | 0 | 17 |

两侧各记录16次转换；每个所选swapchain都在16/16个样式变化区间中唯一匹配主岛窗口，`VidPnSourceId=0`、`LayerIndex=0`，窗口在 `\\.\DISPLAY2`。PresentMon不提供HWND，故归属是唯一时间相关性，不是直接窗口字段。完全落入 first→last style-mutation区间的最大 `MsBetweenDisplayChange` 为baseline 10.05 ms、candidate 15.02 ms；两侧均无完全落入该区间的>50 ms记录。确认尾段有1条baseline 75.02 ms；其余>50 ms样本跨过动画/确认边界。每轮另有39/25条缺`MsUntilDisplayed`及2/2条缺显示间隔的PresentMon行，无法按显示间隔分类。结果不证明没有偶发长间隔或候选收益，仍需更多轮次和C1单独复核。

三轮中的`528.0 ms`仍是candidate第一样式变化至100ms稳定确认的跨度，`522.4 ms`是第二轮单值；两者都不是动画时长。旧7/5条不再表述为确认的动画内卡顿。95/55 ms旧C1区间仍待在修正口径下复核；另一播放器、ETW等待栈、长期性能与A5完整原生视觉/交互验收仍未完成。详见 [A5验收报告](performance/a5-2026-10-12/REPORT.md)。

## 18. 测量脚本审阅、并发补采与计划正文同步

已复核两份新脚本与`paired-analysis.json`，在不导入分析器的独立复算路径中核对两场景四侧原始CSV、分类计数、活动重叠长度及点击派发差值。旧动画独立轮复算的分类计数保持一致；新脚本记录逐条活动重叠与各次动作ID、输入文件哈希和二进制核验状态，遇到主岛swapchain不唯一时拒绝发布所选链汇总。可移植审阅包包含脚本、两份分析JSON及完整原始输入；包内回放显式注明二进制仅来自采集元数据，本机另外核验了保存EXE/DLL实际字节。

原`clickAt`在准备阶段记录，不能用于真实点击响应跨度；新并发轮在页面内`element.click()`前记录派发时间，以页面`performance.now()`计算到首次mutation的延迟。确认也使用页面时间，消除Node/page时间配对混用；该程序点击测量不代表鼠标物理输入延迟，旧轮没有派发戳因而该字段保留为空。

同一保存Release二进制、DISPLAY2、暂停/33%初态各补一轮combined，16次转换。baseline/candidate完全在样式变化区间的>50ms仍为0；确认尾段为1/0条，跨边界14/17条。跨界长间隔与活动区间最大重叠37.6/38.2ms，累计重叠151.1/192.8ms；最大重叠两侧都在第1次转换，是跨动画起点的约130ms显示间隔。不能因跨界就整条剔为静止，也不能把重叠毫秒当成线程阻塞耗时或已定位原因。点击派发至首mutation P50为0.4/0.4ms，最大1.2/1.1ms。

沿用一次控制突发与动画序列同时开始的脚本，每版5条媒体/查询、6条音量调用返回成功，结束暂停/33%；命令时间范围实际与baseline第1–4次、candidate第1–3次样式区间重叠，后续转换没有持续命令压力。因此本轮是有界诊断，不是所有16次转换均承受相同并发负载的压力验收。网易云返回进度/时长为0，seek成功返回不能替代真实进度变化及其他播放器功能检查。

A5 Verification正文已同步“已应用Windows工作树、10/10回归及匹配Release诊断”，该轮当时把旧95/55ms写为“待修正口径复核”；现已在第22节完成原始复核、归属无法确认。动画参数不变，Spring迁移延期；C1、A5仍为`[ ]`。原生交互、逐动作IPC/native resize、CPU/GPU/资源成本与长期验收待后续完成，不由本轮样式测量勾选。详情见 [报告](performance/a5-2026-10-12/REPORT.md)。

## 19. 2026-10-13 全转换并发覆盖与 Apple Music 批次

按用户转交TASK.md，在既有工作树仅修测量脚本、采有界配对数据和更新报告。原保存baseline/candidate的EXE/DLL哈希重新校验，未重建/改业务源文件或提交/推送。每次转换等待首mutation后在页面内派发1个媒体toggle和1个音量写入，记录所属转换及页面实际派发/返回performance.now/UTC。每对请求完成后才准入下一转换，控制峰值在途2；超时停止准入并写失败/恢复状态。门禁严格要求1–16各有成功media AND volume与first→last span正长度相交，排除旧Node/CDP外包络。

无IPC代理full-r1两侧及有代理diag-r3两侧均16/16；helper单测5/5，独立原始复算4/4。主岛swapchain唯一时间相关且每个活动span都有呈现，完整PM/窗口采样覆盖最后confirmation；QPC/UTC、Present起点加MsUntilDisplayed、区间交集和first/last/confirmation分离保持。DISPLAY2、DPR1、同设置、同Apple Music曲目；开始/测量结束暂停33%，清理恢复原暂停20%。本日WebView2均155，不和昨日154跨日比较。PM自身ExitCode=N/A，未冒充0；脚本/矩阵退出码及start/stop/尾段证据保存。

full-r1活动span完全包含的>50ms两侧0，但确认尾段2/1条、跨界4/10条；活动最大重叠31.69/35.51ms，累计61.74/94.66ms。程序点击→首mutation P50均0.4ms，不是物理输入延迟；不能由0个完全包含长间隔宣称无卡顿或优化收益。该轮当时待复核的旧C1 95/55ms现已在第22节复核为归属无法确认，未归因。

草案直接替换只读invoke未生效；失败diag-r1、diag-r2保留并排除IPC验收。最终在传输/回调路径做代理，JSON参数UTF-8字节、请求时序/结束及峰值在途都有原始记录；diag-r3无丢失、无未支持payload、结束无残留。在逐转换点击→确认窗，region请求784→1261，参数字节2,574,929→4,143,985（约+61%），主页面invoke861→1334；完整代理在途峰值14/15。代理有开销，与性能轮分开；不是全协议字节、其他窗口/事件IPC或native worker数，也未得到SetWindowPos调用次数。更长动画的请求总量成本如实保留，不开展C2优化。

full-r1有CPU/GPU committed/Private Bytes/线程/句柄原始采样，但资源探针启动晚：baseline活动段0样本、candidate仅5，匹配活动CPU成本N/A。私有提交、GPU峰值是采样窗值，不是真峰值/长期回稳证明。GPU计数器可读，实际display名称查询失败保留原错误；不强行将VidPnSourceId直连DISPLAY2。跨屏DPI、长期内存/GPU、第6首超时与约25MiB增长继续待验收。

Apple Music真实SMTC会话完成play/pause、时间线推进/暂停稳定及结构化错误负例；有效seek未通过。暂停和播放两次60秒seek均成功返回，但时间线停在156秒或继续157→161秒，未到目标；不把成功返回当作功能通过，也不据此指定根因。最终恢复暂停/音量20%，关闭本任务启动的Apple Music；无法可靠seek恢复进度的缺口保留。播放器不需要用户登录/购曲或模拟会话。

本日报告含16次覆盖表、逐转换IPC、资源可用范围、命令/退出码、失败及恢复、原始哈希和离线回放包。全转换并发覆盖已交付；完整原生交互、有效seek、资源成本和长期验收未关闭，C1、A5仍 `[ ]`，Spring延期；停在可审阅交付，后续批次另安排。见[执行报告](performance/a5-2026-10-13/REPORT.md)。

## 20. 第2批：媒体能力与控制错误返回（10月13日启动、14日交付）

仅改`services/media.rs`及`App.svelte`的媒体错误/快照连接，保留既有C1/A5未提交修改及保存Release对照。control_media原来等待WinRT但丢bool，现true=受理、false=结构化拒绝、WinRT error=失败。seek_media原来已经检查false，本批没有错误归因：增加实际选中会话能力门禁，非网易云seek=false直接不支持、避免TryChange派发；与显示层共享判定。网易云未声明/未发布能力的既有兼容尝试保持，API false/error仍报错。没有增加主线程同步等待。

主页面取消播放状态的乐观翻转和失败后反向翻转，以真实媒体快照为准，避免拒绝覆盖请求期间的新快照。已有150ms后台异步刷新不变，按钮等待实际快照；UI布局/动画未改。seek只读入口原本已隐藏范围输入，现与命令拒绝一致。网易云成功seek未真实重测，仅保留并测试兼容准入及结果处理。

Rust相关15/15（新增5项：true/false/error、不支持seek不派发、兼容/声明支持派发一次、无会话）、前端媒体store/clock 17/17、Svelte零错误/警告、前端production及标准Tauri Release通过。初次直接cargo产物错误页及空SMTC会话/仅fetch注入未命中的失败记录保留，未计通过；正式使用标准Tauri构建单独保存的EXE/DLL，旧baseline/candidate实际字节哈希未变。

新构建实际Apple Music raw ID为AppleInc.AppleMusicWin_nzyj5cx40ttqa!App，曲目“飛ぶ時”/252秒/seek=false。暂停21秒请求60秒，立即reject code2001“不支持调整播放进度”，随后仍21秒；播放时22→23秒，暂停24秒稳定。主页面已展开且真实进度区存在，seek范围输入为0。播放器启动前及关闭后，control/seek无会话均reject code2001。

UI拒绝另以明确标记的诊断故障注入测试：阻断一次control传输、延迟错误并投递实际SMTC暂停快照，前后按钮都“播放”，实际状态未变、错误记录可见。不是模拟SMTC会话，也不是宣称真实WinRT false已由播放器重现。当前仅有Apple seek=false会话，声明支持的有效seek真实验收N/A；不能将不支持路径正确当成有效seek通过。

本批恢复本次实际设备AB13X USB Audio及音量13%/暂停，关闭任务启动的播放器和应用；不能可靠恢复原进度的限制保留。第1批region增长/参数字节约61%、CPU配对不足，旧95/55ms修正复核、A5原生交互、资源成本及长期稳定性继续待后续；C1/A5仍 `[ ]`，Spring延期、不做C2/UI/region架构、不提交推送，停在可审阅状态。详见[报告及本批diff/证据](performance/c1-media-2026-10-13/REPORT.md)。


## 21. 第3批：资源探针就绪与匹配采集（2026-10-14）

仅改测量/门禁/文档。probe完成进程树初始基准和有效CPU增量后发布本轮run ID/PID/UTC ready，协调器校验后才准入序列；CPU按逐PID实际读取区间分类，跨界增量单列。自动拒绝陈旧ready、0样本、基准点冒充和尾段截断，14/14测试通过。失败r1播放器选择不符、r2时间字符串格式误拒绝、r3仅2样本全部保留排除。

原保存A5配对resources-r4（两侧固定250ms转换间隙、无IPC代理）门禁/独立复算通过：16/16并发媒体AND音量覆盖，序列9.721/12.143秒，完整窗内CPU样本6/8；窗内所采增量CPU3.875/5.484375秒，跨界0.90625/1.03125秒独立保留，不按整序列分母冒充纯动画成本/收益。内存/GPU/线程/句柄为全采样窗范围，非真实峰值或长期证明。完全活动内>50ms均0，但确认尾段3/1、跨界13/13，固定间隙造成的静止跨度不能称动画卡顿。

DISPLAY2/DPR1、同WebView2/设置/真实Apple歌曲/暂停33%测量初态；真实进度2/5秒不同，不seek伪造匹配。本轮原用户16% REDMI音箱和播放器选择/顺序/屏幕均恢复，测试Apple/isle关闭、网易云保留。此轮旧构建不代表第2批新Release性能验收。C1/A5保持 `[ ]`，旧95/55ms、原生交互、region成本增长、有效seek及长期内存/GPU继续待办；无业务/动画/C2/I1/UI/Spring修改，无提交推送。见[第3批报告](performance/c1-resources-2026-10-14/REPORT.md)。


## 22. 第4批：旧C1 95/55ms只读证据复核（2026-10-14）

只读核验27份输入哈希并复算：C1 CSV第669行95.0015ms/动画独立动作13，第1687行54.9938ms/并发动作14，PID [omitted]、swapchain0x21E4A377F30。旧分析器直接用TimeInDateTime、漏MsUntilDisplayed、按结束点筛行并把animationEnd稳定确认快照当纯活动终点；16小时偏移由first CPUStart与启动UTC按整小时推算，不是成对帧时钟锚点。帧无TimeInQPC，窗口QPC不能完成这些旧帧的QPC对齐；没有精确last mutation。源间隔真实、旧snapshot-envelope分类可重建，但实际活动相交/完整包含与线程阻塞根因无法确认，撤回旧C1报告genuine/fully-inside actual animation结论。

条件分析明确假设exact16h及>=100ms稳定契约：加MsUntilDisplayed后，95ms区间05:18:57.261037600–57.356039100Z，55ms区间05:19:05.355903500–05.410897300Z，均落近似确认尾段；距确认减100ms仅2.0376/1.9035ms，几毫秒映射残差即可改变类别。不是精确last-change复原，不用条件结果确认根因。C1正文/摘要/状态表已直接改正，不仅追加脚注。

| 剩余项 | 类型/现状 | 最小下一步（本批不执行） |
|---|---|---|
| 旧95/55ms实际活动归属 | 未知证据，历史复核已完成、无法确认 | 新轮保存逐帧QPC或成对时钟锚点与精确首/末/确认；不硬减偏移凑确证。 |
| 当前媒体修复构建的C1性能 | 后续测试，旧保存A5不能替代 | 匹配当前媒体修复/工具/语料条件的有界前后Release并发、延迟/资源/帧时。 |
| WinRT拒绝与UI快照正确性 | 原确认缺陷已在第2批修复并独立验收 | 无需对同一不支持Apple会话重复测试。 |
| Apple seek=false | 真实不支持能力，拒绝/只读通过 | 不要求修成支持；使用声明支持的真实会话单独验有效seek，当前N/A。 |
| 线程阻塞根因 | 未知，无相关栈 | 仅确认真实连续活动停顿后另安排最小ETW/WPA栈；不自动提权。 |
| A5原生交互 | 后续测试，Windows夹具10/10不是原生验收 | 有界覆盖4edge/style、反向/hover/隐藏/封面/焦点/低动效，核对视觉与命中。 |
| region/IPC成本 | 确认测得原候选请求/参数约+61%，优化归因未定 | 匹配当前构建记录请求/字节/在途及native更新成本；不在此批实施C2/I1。 |
| 长期内存/GPU | 未知，短探针不证回稳 | 后续独立长期轮；不扩大本批。 |

C1/A5 `[ ]`不变，Spring延期。本轮无新采集/重建/业务/动画/C2/I1/UI修改、无提交推送。脚本/原行/原输入/hash/diff/离线包及详细报告见[第4批交付](performance/c1-review-2026-10-14/REPORT.md)。


## 23. 第5批：当前媒体修复Release A5原生交互取证（2026-10-14）

标准Tauri保存新Release EXE EA80838E…/DLL33B43C71…实际hash验证，无重建，参数420/360/240源hash一致。测试工具以OS注入实际鼠标/键盘和副屏专用底板，记录trusted页面事件与底板收件，不以DOM点击/WM_NCHITTEST代替native命中。完整原生矩阵未通过：四edge首次候选坐标及edge→floating首次点击落底板、主页面无click，各项停止；展开/收起/反向因此未完整验完，根因未确定，不在此批改I1/C2/动画。

floating Hover中展开、收起中反向和快速进出状态终态有证据；已有音量按钮native进入页面、trusted Tab至DIV滑块有证据，但旧按钮-only断言遗漏滑块，焦点可见性仍未知。首轮透明项因compact前置失败未到目标；仅补独立透明点(-1143,180)，底板+1/主页面click不增确认穿透。首段封面未ready，补真实网易云图ready后的65份几何样本（7份moving-cover图已加载）及native终态截图，不自动签署全程无跳位/主观手感。已有隐藏按钮10秒恢复后命中通过；本原生页面CDP低动效模拟及终态通过，不是Windows全局设置验证。

两屏DPR1，真跨DPI N/A；12秒15fps桌面短片仅视觉证据，不是性能。测试当时未记录loaded WebView2版本/实际image查询字段；补启动元数据命令被自动审批拒绝（blocked by policy），未重试，字段N/A。工具setup delegate失败、按钮-only焦点/前置失败原始记录均保留，不包装全通过。

设置、播放器选择/顺序/monitor、16% REDMI音箱/静音false、暂停状态与前台HWND均恢复；仅关闭任务isle/底板，保留网易云。C1/A5 `[ ]`，U4–U11待办边界保持，业务/动画/C2/I1/UI/Spring未改，无性能轮/提交推送。完整原生命中/视觉、焦点、当前构建匹配成本/性能、支持seek会话与长期稳定继续单独验收。详见[场景、原始证据和恢复报告](performance/a5-native-2026-10-14/REPORT.md)。


## 24. 第6批：首击失败离线定位与工具准备（2026-10-14）

五条失败各一次底板收件、page无click，后查与回调差45.8281/50.4479/54.6019/54.3352/53.4941ms；后pointPID不代表按下收件。旧helper移动后立即合并SendInput，成功floating先有move/等待但不同点，不能称严格停留对照。当前cursor monitor检查后按33/50/200ms休眠，再依存储region更新ignore；App host/offset/anchorGap、签名去重/异步revision路径及后端revision拒旧已只读追踪。缺按下前状态/独立执行戳/accepted-region，根因仍未知。

预定8次单点方案：top内侧和floating内侧各0/50/250ms，原top边界0/250参考；方案未执行，点无效则记前置失败，不换点凑通过。新helper默认在GUI/PInvoke前dry-run返回，move/state/down/up/pump分开，press不移动/隐含等待，UTC/QPC、前后cursor/styles/lookup与实际收件分开；后端接受regionN/A。支持DIV滑块active/focus-visible/outline/值及合法箭头、恢复设计。7/7离线测试与默认无GUI dry-run通过；未运行live路径/新采集/重建/业务/动画/UI/C2/I1/Spring修改或提交推送，未重试被拒元数据动作。C1/A5 `[ ]`，原生矩阵不通过/首击原因未确认，方案不等于复现或修复。详见[离线报告](performance/a5-offline-2026-10-14/REPORT.md)。


### 第24节同批工具返修（2026-10-14）

独立审阅认可原离线证据，工具准备退回两项；本返修只改测试工具与文档。成功注入down才取得所有权，正常up清除；EOF/stop/异常finally只释放本工具仍持有的按下，正常up后不重复释放、未持有不释放。释放失败保持held并记录错误、跳过显式cursor/foreground恢复。实际helper共享策略模块，9项fake-backend故障路径离线验证通过。双侧收件保留两个计数、标ambiguous/conflict且uniqueRecipient=false，单侧/无侧分开，late PID不定收件；JS9项通过。原包/首轮版本保留，修订工具仍待独立验收。无GUI/原生输入/现场方案/采集/构建/业务修改、无提交推送，未重试被拒动作；C1/A5 `[ ]`，根因未知。


## 25. 第7批：固定8次首击停留对照（10月14日执行、15日交付）

用户在等待明确现场允许后“继续”，按原8点/停留方案执行，前置点检有效，無换点重试。top内侧0ms无两侧收件、50ms底板、250ms页面；floating内侧0ms底板、50/250ms页面；原top边界0ms底板、250ms页面。8次down/up各返回1，最终无持有输入。操作QPC间隔0预算实际3.8–8.8ms、50约58–60ms、250约261–263ms，含查询/往返，不等效原合并立即click。

页面4次trusted click都在pre-down bit20=false，其余4次pre-down true；只支持异步穿透/时序候选，不确定region/几何/完整投递根因。first top0未保存完整pointerdown/up，明确unobserved，不能靠后pointPID当通过。正常同次会话CDP得Edg155.0.4283.45，没有另启被拒元数据命令；actual image getter/accepted-region仍N/A。

协调器fake4+计划9共13/13、所有权fake9/9、默认无GUI与离线证据核验通过。用户实际8% REDMI音箱/静音false和原网易云正在播放、settings/monitor/player选择及前台恢复；cursor仅请求恢复未即时回读。测试进程关闭、用户网易云保留。当前首击仍未通过，C1/A5 `[ ]`，无业务/动画/C2/I1/UI/Spring修改，无重建/性能轮/提交推送，8次即停；另由发布方独立验收。详见[报告](performance/a5-dwell-2026-10-14/REPORT.md)。

### 第7批PID来源表述修订（2026-10-15）

原始appPid=25612仅为Node spawn PID，helperReady.boardPid=26164为底板PID，WindowFromPoint记录9440仅为点下窗口owner PID。缺少进程映像/父子关系记录，不把9440称为测试isle进程。仅离线修订文档并重新出包，原始证据与旧ZIP不改；4 page / 3 board / 1 unobserved及bit关联结论不变，根因未知，C1/A5 [ ]。

## 26. 第8批：原生首击机制设计与隔离原型（2026-10-15）

后台monitor读取光标后经Tauri UI派发/tao样式切换，33/50/200ms检查本身形成等待窗口；上层成功只代表派发，last_ignored不是原生确认。HTTRANSPARENT只保证同线程下层搜索，不能直接保证其他应用收到；windowed WebView另有子窗口。持久window region会改变绘制裁剪/阴影与动画、DPI、拖拽生命周期，任务单允许先交付设计和隔离原型，因此本批不强接生产。

window_region_prototype.rs仅cfg(test)，无GUI/输入/窗口调用，复用point_in_rounded_rect形成有预算的物理行span；generation/epoch/revision与fake后端验证迟到拒绝、失败不伪报成功。另以无窗口GDI构造/RAII与PtInRegion检查掩码；尚无SetWindowRgn适配器/HRGN向系统移交/宿主回调接线，ready只是模型状态。当前Release保留旧monitor，不能作为已安装候选验收。首击未通过，根因未知，C1/A5 [ ]；本批候选待审/待现场验证。

无应用/浏览器/底板启动、输入、播放器控制、现场采集、提交推送；不重试被拒动作，不覆盖旧证据或二进制。详细API依据、接入事务和后续同点拟议清单见第8批报告；交付后停止。

## 27. 第9批：默认关闭的原生region静态实验适配器

显式Cargo feature experimental-native-region、default=[]；仅复制已接受几何到独立a5-region-static-lab，不运行产品hover/封面/动画，不替换main旧monitor；实验label拒绝旧机制接管。enqueue/desired/native-applied分离，同UI线程SetWindowRgn成功才移交HRGN，失败释放并报错；本地subclass及窗口事件接线。主岛须先隐藏，输入/捕获/非本应用foreground/非零client offset/RTL/live mutation/restore等不支持状态实际拒绝或失效，释放不重放。

共享raster有像素/段/顶点/extra rect预算，实验按整数hotspot采样避免中心采样的分数边缘扩张，三档DPI逐点对照通过；只供机制实验，可能阻塞UI；nativeReady不代表Canvas已呈现。产品外阴影、移动封面和装饰不能用扩大region保留，本实验的轮廓画面不证明产品视觉兼容。默认窗口测试24、实验37、媒体15通过；标准默认/实验Release结果及hash见第9批报告与审阅包，均未启动。旧立即首击失败保留，不以本lab替代旧8次hover验收。

实验适配器待审，未批准默认接入、未获现场采集权限；OS输入/视觉/生命周期/资源未知，C1/A5 [ ]，Tween420/360/240与媒体修复保持，C2/I1和Spring不推进。无GUI启动、输入、播放器操作、现场、提交推送；交付后停止。

## 28. 第10批：四次静态lab协调工具，纯离线交付

复用第9批保存experimental-final-release，EXE 4919DD50… / DLL 1C455A8E…，未重建/启动。新增默认dry-run的实际协调I/O与共享fake策略：新Studio正常前台、主岛副屏且隐藏、no-activate独立STA底板、已接受revision/请求与实际frame/DPI/Canvas及桌面两块像素确认。实际坐标N/A；先固定5×5不透明最近中心/透明y-x首点及right+32,bottom+32起点规则，未来任何输入前一次冻结，inside/outside各2次，不换点/不重试。

两侧down/up/click对称判定，原始事件/不同钟/身份来源保留，浏览器宿主native身份为预检查询、不是OS投递HWND；双侧/缺失/拆分/错侧/坏tag/坐标/双击或激活异常立即停止。status只在安全foreground/无按下门后查询，其主动失效单列，不冒充唯一OS路由根因；0预算仍含协议/检查成本，不等价旧合并立即click。真实no-activate、呈现、投递、恢复和资源未验证。

共享实际策略JS 28/28、原字节owner 9/9、日志/释放锁存故障5/5，默认/未批准live拒绝路径与C# CompileOnly通过；没有GUI/PInvoke输入/现场采集。正常up失败后锁存，catch/EOF/finally不再原生发送；stdout/日志失败不跳过保留。安全释放后按lab关闭→Studio晚persist结束→设置/可见性恢复→本轮应用正常退出→原始文件/Run值回读→底板关闭及前台/光标即时回读；不确认即pending，不硬杀。

本轮只新增工具/测试/文档及本计划增量，237项第9批源配置资源、4个选定二进制和原owner字节保持；旧证据保留。具体最多4次现场单与审批样例保持未批准，由发布聊天独立审阅工具后再请求人类允许。本批无应用/浏览器/底板启动、播放器操作、业务/动画/C2/I1/Spring推进、提交或推送。C1/A5 [ ]，完整首击/性能/长期验收仍待办；发回报告后停止。详见[报告](performance/a5-static-tools-2026-10-15/REPORT.md)和[待批准现场单](performance/a5-static-tools-2026-10-15/LIVE_PLAN.md)。
## 29. 第11批：单次静态lab前置失败及遗留恢复（10月15–16日）

按直接人类允许使用第9批指定实验构建，仅一个正常会话，未重建。162次status含1次absent、161次Enqueued/nativeApplied=false/nativeReady=false，原定10秒预算到期停止；测试move/down/up均0，没有冻结坐标、Canvas/截图或两侧收件阶段，N/A。底板创建前后foreground一致只证明创建时未激活，真实路由/点击no-activate/视觉/性能仍未知。explicit close的Invalid不是此前路由失败证据，持续Enqueued根因未定；没有重开/改点/补输入。

lab/Studio正常关闭。恢复前唯一语义差异为playerOrderIds新增Chrome.UserData.Profile2，与Studio自动发现路径相符但caller未记录。时间解析、File.Replace空参数及数字PID门失败原样保留，各新方法经独立审阅；v2已恢复source可见性并正常退出Isle，v4未执行。16日原工具已不在、无Isle writer且文件/Run仍frozen，已审仅持久恢复一次完成两配置原字节/Run原值即时回读并独立核验；一个已确认失败temp精确清理。私有备份不进分发包。

历史foreground/cursor及原工具最终退出清理无法回溯确认，不把配置恢复当完整环境恢复。原evidence/journal字节保留，checkbox不改；无业务/动画/C2/I1/Spring更改、无提交推送，C1/A5[ ]。发回[报告](performance/a5-static-lab-2026-10-15/REPORT.md)和包后停止，首击/完整性能和长期验收仍待办。

## 30. 第12批：初始化就绪门与恢复工具离线返修候选（2026-10-16）

仅修 feature-only 静态 lab 初始化及其协调/恢复工具。锁定 wry 实际先 attach_handlers（约466）后 Navigate（约531）；642–665是函数定义位置，此前“注册晚于导航”候选撤回。第11批持续 Enqueued 根因仍未知。本代次显式完整文档导航，文档 load 后带唯一身份的标题回执与 HWND/代次/URL共同校验，普通Finished只记录；loaded/configured/queued、有限阶段与错误公开，10秒期限不增加，缺事件也显式失效。离线协议测试及构建不证明真实 WebView事件、呈现或OS输入。

实际 helper 共享 NullString 原子替换，真实临时文件测成功消费/读回及分享锁失败保留；JSON保留7位tick字符串，实例门拒绝PID复用。不可变启动前备份、实际Studio IPC请求/响应审计、严格受控发现顺序及工具monitor补丁、当前状态分开；未知键/顺序/写入与传输回退拒绝。恢复有同会话cleanup-only入口，只继续未完成步骤，不重开实验/重置4次预算/重发已失败释放；新idle、实例及设置门通过才续清理。EOF/断管/旧实例明确仅允许另审持久恢复边界，不操作跨日旧前台/光标。正常helper/父协调器退出由实际子进程close确认代码接线，真实GUI恢复仍未现场验证。

默认及feature Release分别另存，默认仍保留产品旧monitor、未安装首击修复；Tween420/360/240、媒体、C2/I1/UI/Spring保持。分发分析器改读raw而非run-01，离线复算仍为161次Enqueued/0测试输入。旧证据/失败/私有备份/原恢复脚本和二进制保持；无GUI/浏览器/底板启动、输入、播放器操作、新采集、提交推送。结果以[本批报告](performance/a5-static-repair-2026-10-16/REPORT.md)与manifest为准，候选待独立审阅，C1/A5[ ]，不授权第二次现场；发回发布聊天后停止。


### 第12批同批返修 R1：独立审阅三项退回（2026-10-16）

仅修三项实际接线：配置策略在无gate/OWNER锁时获取HWND，再持锁提交纯状态，真实provider回归检查可取锁；Run分开不可变原快照和经选定EXE/DLL hash、实际实例/映像、首次autoStart及锁定实现hash证明的受控启动期望，逐值读回ack后才完成，部分恢复续做拒绝已ack项回退到实验路径；清理服务分开5秒读请求、60秒执行和70秒客户端预算，合法请求不再被5秒断开，超执行预算明确pending且不取消/并发重跑，断管标outcome unknown而非空JSON。

共享实际配置/恢复/传输离线回归及标准两版构建另存，真实5300ms与断管互斥回归通过；fake注册表不碰用户Run。旧候选和审阅失败保留。新候选仍待独立审阅，旧Enqueued根因不归因为本次修复，无新现场/GUI/输入/播放器/提交推送。C1/A5[ ]，默认旧monitor、Tween/媒体/C2/I1/UI/Spring不改。详见[返修报告](performance/a5-static-repair-r1-2026-10-16/REPORT.md)，发回后停止。

## 31. 换路线执行：拆分收尾与最小机制验证（2026-10-16）

本节为用户当前“开始”后的执行口径；第1–30节保持历史证据。2026-10-16用户随后确认按实现完成口径关闭C1/A5；未补齐的性能和体验数据继续如实保留，但不再作为总项关闭门槛。

- [x] **C1-I：阻塞迁移与返回契约。** 当前媒体/音量设备/图像/缓存与设置相关调用链核对完成；后台容量/线程2项、媒体控制错误契约5项本轮通过。实现结论独立成立，不依赖A5或旧95/55ms归因。未声称实际性能收益。
- [x] **C1-P：当前Release性能关闭决定。** 用户确认不再等待新一轮Release性能采集；当前收益保持未知，不以旧样本/最小工具补写结论。若未来确证实际活动停顿，再独立采栈。
- [x] **A5-I：动画候选实现。** 保留展开420/收起360/Hover240ms；沿用既有实现回归，本轮参数单测3/3。不给新候选叠加Spring。
- [x] **A5-F：短人工体验决定。** 用户确认保留当前420/360/240ms候选并关闭A5；首击失败仍不归因于调参，完整产品交互矩阵未补测的事实保留。
- [x] **H1：原生首击，独立发布阻塞项。** 最小原生两进程的中心与透明圆角两次收件通过；最小WebView也已通过中心trusted三事件和跨进程透明角收件；产品接入、无激活立即点击/四edge/动画阴影与DPI仍分阶段验证。工具激活目标的结果不冒充零停留首击通过。
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
- [ ] **R1：长期资源。** 内存/GPU长期行为单独验证，不再阻塞C1-I。

最小实验源码为 scripts/first-hit-minimal.cs 和 src-tauri/examples/first_hit_webview.rs；不调用产品run/setup，不启动Studio/播放器，不读写用户设置或Run注册表。旧实验、故障证据、恢复工具冻结保留。只增加独立example，不接入默认产品路径、不改现有动画或媒体业务。

原始数据、构建/运行限制、失败保留及当前结果见[本轮报告](performance/first-hit-minimal-2026-10-16/REPORT.md)。机制实验预算到期没有明确结果就停止该路线，不继续扩建协调器；后续产品接入不能仅凭静态胶囊成功放行。

## 32. H1 下一阶段：真实Compact组件隔离接入（2026-10-16）

用户“下一步”后，先核对真实产品接入风险：Hover立即启动tween，固定Compact掩码会裁剪动画，旧monitor/watchdog又会异步修改穿透状态。因此先把真实IslandSurface与共享raster/GDI接到独立宿主，而非直接改main。

compact-native-probe feature默认关闭，独立example仅注册apply_compact；原生拒绝main、非Compact、未settled和重复claim。实际SetWindowRgn成功才显示；位置/尺寸/DPI变化请求隐藏。前端固定style/edge/Compact，使用演示暂停媒体，无Spectrum/真实封面，不运行产品setup/配置/媒体服务。未修改产品动画和事件代码。

41项窗口相关离线测试、独立前端构建、Svelte零错误/警告、默认feature后端check通过，完整候选构建通过。原定floating/top与edge/left各内部/外部共最多4次，启动命令被执行策略整体拒绝，只返回blocked by policy，未提供可确认的审批来源；未重试或绕过，测试宿主启动0、点击0，Vite服务已关闭。上轮手写胶囊通过结果不算本轮真实组件通过。

状态：H1仍[ ]，真实组件呈现/路由/生命周期待现场；随后才做动画前清除掩码的确认屏障及与旧线程互斥。不新增现场恢复平台，不迁移Spring/I1/C2，不把静态实验冒充产品首击修复。详见[候选报告](performance/compact-native-2026-10-16/REPORT.md)。

## 33. C2 媒体监听事件驱动候选（2026-10-17）

媒体监听线程复用一个SMTC manager，订阅CurrentSessionChanged、SessionsChanged，以及各会话的TimelinePropertiesChanged、PlaybackInfoChanged和MediaPropertiesChanged；容量为1的同步事件队列合并重复通知后立即读取当前快照。会话订阅以 `windows-core 0.52.0` 的 COM 身份比较（canonical `IUnknown`）匹配具体对象，不以应用ID代替对象身份；对象移除、manager重新获取和线程退出时注销对应token。播放/暂停/无会话仍有1/2/5秒低频兜底，首次空快照后直接以1秒间隔确认；第二次连续空快照只发送一次空状态事件。媒体快照连续两次查询失败时丢弃旧manager及会话订阅，并立即尝试重新获取，RequestAsync失败则每5秒重试。单次查询失败不会清空当前媒体状态，且会打断连续空快照计数；普通媒体命令仍按调用方独立获取manager。

本批按协调窗口审阅，修正了重复SourceAppUserModelId下的会话重建/多对象订阅身份问题，以及确认空会话后每个低频轮询周期重复广播的问题。身份比较语义来自锁定crate源码：`IUnknown::PartialEq` 会 QueryInterface 双方到 canonical `IUnknown` 再比较指针。以上仅为静态实现和编译验证；不代表WinRT服务断线可在所有Windows版本及播放器上可靠触发查询错误。无会话属于有效空快照，不应触发manager重获。Windows运行时仍需确认：管理器首次可用/不可用、选中会话切换、切歌、播放暂停与进度事件、会话退出、短暂SMTC空快照，以及查询失败后的重新订阅恢复；每项记录事件时间、快照更新时间、恢复时延、token数量和应用日志，测试播放器需为专用会话，不操作用户正在使用的播放器或设置。

### 离线Release与后续采样步骤

1. 先记录工作树/提交、`src-tauri/Cargo.lock`和最终Release二进制SHA-256；使用Rustup真实工具链和已缓存依赖，不联网、不安装或升级工具。2026-10-17使用 `<local-user-path>`（cargo 1.94.1）并仅在构建进程设置 `RUSTC=<local-user-path>`（rustc 1.94.1）；从 `src-tauri` 执行 `cargo check --offline --lib` 和 `cargo build --offline --release`。输出使用独立 `CARGO_TARGET_DIR=src-tauri/target/c2-offline-2026-10-17`，避免覆盖已保存的 `src-tauri/target/release`。如缓存缺失即停止并记具体依赖，不改用在线解析。Cargo build输出留在新的带日期目录并计算SHA-256；不要覆盖发布包或用户安装。
2. 离线功能验证在专用Windows测试配置中运行新构建，先记录监听线程/manager订阅初值，再逐项触发上列SMTC事件和可控断线/恢复，检查更新及时性、重复事件合并、订阅清理、单次空快照保护及恢复后事件仍可用。保留ETW/应用日志和过程步骤；不可控断线应标N/A，不以模拟数据替代。
3. Release资源对照须使用同一台机器、同一电源模式/电量、显示器与WebView状态、播放器/会话和应用设置，按baseline/candidate交替顺序各至少3轮；每轮启动后预热5分钟，记录10分钟idle、5分钟暂停会话、5分钟播放会话。保存每秒的Isle及子进程CPU时间、工作集/私有字节、线程数和系统电源状态，计算CPU毫秒/分钟及每轮分布；应用退出恢复5分钟后记录。系统级功耗用Windows Performance Recorder可用的Power/CPU ETW profile与WPA能量视图；先用 `wpr -profiles` 确认profile存在，无法取得受控能量计量时明确记为“功耗未测”，不拿电池百分比短时变化作功耗结论。保存原始ETL/CSV、profile、命令、构建hash和环境清单，不只报均值。

2026-10-17真实离线构建（上一候选，不对应本次修订源码）：PATH中的 `<local-user-path>` shim确为0字节，但Rustup真实工具链有效；此前“本机无法重新编译”的判断已纠正。工作树HEAD `48530f7343e84129aef34113dbef4dd18f7431ef`，工作区含既有未提交改动。首轮 `cargo check --offline --lib` exit 0（3分13秒）；首轮 `cargo build --offline --release` exit 0（10分42秒），边界审阅修复前EXE已保留，SHA-256 `4EE631AA7C8E4D4E028DA7A4F6F93CD4310DE1E1B76CA0357806B429C8DEA07C`。最终源码再次 `cargo check --offline --lib` exit 0（12.06秒），`cargo build --offline --release` exit 0（7分22秒）。其EXE曾为 `src-tauri/target/c2-offline-2026-10-17/release/isle.exe`，SHA-256 `384E96E5C8581D263E61BEA012F08EAF9338FD639D2E8286A8A6233599B90689`；同版 DLL SHA-256 `745FF8FE25D3F10577EC58E93360DCC7E7A1924DB9F63704910E74C869AB2483`。本次返修前已将这对文件分别保留为 `isle-c2-previous-candidate-384E96E5-2026-10-19.exe` 和 `isle-lib-previous-candidate-2026-10-19.dll`，哈希保持不变。原有 `src-tauri/target/release/isle.exe` 未改写，SHA-256仍为 `38102A0F66B93A4370155CCFD6F420BC8A3831646216B7076C67DB493F506E7A`。根目录 `build/` 在构建前已存在且被 `tauri.conf.json` 指定为frontendDist；直接Cargo构建未调用 `beforeBuildCommand`、未运行前端构建或Tauri bundler，因此结果是本地Release EXE/DLL，不是NSIS/标准Tauri安装包。

2026-10-19 C2 idle cadence 返修：审阅发现首次缺失后虽按1秒确认，空状态已确认后的第三次及以后空快照仍被无条件重置为1秒。现改为把 `empty_state_emitted` 传入 `media_poll_interval(false, false, ...)`；逐步核对四次连续空快照为1/5/5/5秒，第二次确认只发送一次空状态；查询错误将未确认缺失计数清零，活动会话恢复后首次缺失仍为1秒。最终源码 `cargo check --offline --lib` exit 0（29.61秒）；同一隔离 `CARGO_TARGET_DIR=src-tauri/target/c2-offline-2026-10-17` 下 `cargo build --offline --release` exit 0（4分30秒）。当前EXE `src-tauri/target/c2-offline-2026-10-17/release/isle.exe` 为8,490,496 bytes，SHA-256 `6FF00C4C1D2304EDA70551F85A0C356FB98E3812A8FD3862505F69F8CF0B3036`；DLL `isle_lib.dll` 为116,736 bytes，SHA-256 `2A5D20778EEBC7AC5F3A2F34C2B39F705200913B51F17626945CD8D4C2F51345`。之前的384E候选保持在独立归档名，未将其哈希冒充本次源码产物。`git diff --check`通过。未运行EXE、测试、GUI或播放器，未改用户设置，仍无Windows运行时/CPU/功耗证据；C2保持 `[ ]`。

2026-10-20 Windows运行时隔离审查：未启动候选（运行0分钟），因此本轮没有实际观察监听初始化、manager/当前会话读取、SMTC事件触发更新、空会话确认/idle或进程退出后的订阅清理；这些仍全部待测。具体阻塞是产品启动没有隔离开机启动副作用：`src-tauri/src/lib.rs` setup 在读取设置后无条件调用 `set_auto_start(initial_settings.auto_start)`（约920行），`src-tauri/src/services/settings.rs` 会按设置写入或删除 HKCU `Software\\Microsoft\\Windows\\CurrentVersion\\Run` 的 `Isle`/旧版值（约151–193行）；只读检查确认当前HKCU Run下已有`Isle`值。配置使用固定Tauri identifier `com.isle-app.isle`（`src-tauri/tauri.conf.json`），设置读写走同一 `app_data_dir()/settings.json`（`settings.rs` 24–25、64–74行），没有候选实例的数据目录覆盖；因此启动可能改写当前启动项并共用真实设置，未运行以避免影响用户配置。缓存也可能共用：初始化先写EXE旁 `cache/media`，失败后回退同一 `app_cache_dir()`（`services/cache.rs` 143–154行）。

额外观测边界：`lib.rs` 约905行用默认 fmt subscriber、INFO级别初始化日志，但成功获取manager/快照和发送`media-update`均无日志；`event_bus.rs`虽维护计数，未暴露本应用诊断接口。最短下一步是在测试构建中加入明确的隔离数据/缓存目录并跳过Run同步的实例开关，同时提供不含媒体隐私的监听阶段/快照时间/订阅数诊断输出，再于专用Windows配置中启动候选；本轮没有新增诊断实现。候选/现有Isle及播放器进程均未启动或操作，设置和Run未改写，未做测试/性能采样，无CPU/功耗结论；C2保持`[ ]`。

2026-10-20 C2独立媒体宿主候选（未运行）：新增默认关闭的 `c2-media-probe` feature 与 `c2-media-probe` example。入口为 `src-tauri/examples/c2_media_probe.rs`；只在内存中注册 `AppState::default()`，将生成 `Context` 的 identifier 改为 `com.isle-app.c2-media-probe` 并在 `Builder::build` 前清空 `app.windows`，之后调用 `isle_lib::start_c2_media_probe_listener(AppHandle)`。未调用产品 `run()` / setup、未注册插件/命令、未建窗口/WebView/托盘，也未初始化Spectrum、捕获监控、设置或缓存。

复用生产路径：共享 `start_media_listener` 工作循环及COM apartment、SMTC manager与会话订阅、`services::media::get_media_info_from_manager` 快照读取、`EVENT_BUS.initialize` 和 `emit_media_update`。feature-only停止句柄通过同一容量为1的消息队列送入Stop；工作线程退出循环后走既有manager/session Remove* token注销，然后join。example在60秒观察截止后请求停止、join，再调用 `App::cleanup_before_exit`。同步WinRT `RequestAsync().get()` /媒体属性读取本身不可取消，因此若系统WinRT调用卡住，正常join及进程墙钟时长理论上可能越过60秒；这不是硬杀式上限，真实Stop响应/注销结果仍待运行观察，异常退出或永久挂起下清理不作保证。

仅probe路径输出一条汇总行：manager获取成功/失败、首次manager就绪/事件/兜底刷新数、非空/空/错误快照数、成功注册/成功注销token数与注销错误数，以及运行相对时长和快照均值/最大耗时。无曲名、艺术家、封面、会话AppID、媒体路径或事件payload。没有WebView，所以只能确认真实listener快照和EventBus发送调用路径被执行；不能观察前端收件/渲染、具体媒体内容或用户可见更新。无自然SMTC事件时事件刷新为0；没有会话时空快照可见。短运行不提供CPU/功耗证据。

落盘/注册表调用图审阅：example与复用的listener/get_media_info_from_manager/EventBus路径不调用 `app_data_dir/app_cache_dir`、设置读写、`init_cache_system`、`download_and_cache`、`current_exe`或Winreg/`set_auto_start`；封面仅从SMTC stream读入内存，处理后放入AppState内存缓存。因而产品的`settings.json`、HKCU Run `Isle`、exe旁`cache/media`及`app_cache_dir()/media_cache`都不在此入口的可达路径；Tauri identifier独立且无WebView user-data目录。example仅向stdout写一条汇总，不写日志文件。此为静态调用图审阅，宿主本批未运行。

离线检查使用Rustup cargo/rustc 1.94.1与既有隔离target `src-tauri/target/c2-offline-2026-10-17`：默认 `cargo check --offline --lib` 通过；`cargo check --offline --example c2-media-probe --features c2-media-probe` 通过；没有重建Release或改写6FF00C候选。本候选的预期运行命令（仅供协调窗口代码审阅后另行安排，本批未执行）：从`src-tauri`运行`$env:RUSTC='<local-user-path>'; $env:CARGO_TARGET_DIR='.\src-tauri\target\c2-offline-2026-10-17'; cargo run --offline --example c2-media-probe --features c2-media-probe`。本机后续运行只能被动观察现存SMTC会话，不操作播放器或Isle实例。C2状态为“宿主候选，未运行”，尚未验收实际manager、事件更新、idle/空状态和退出token注销；不宣称运行时通过或优化收益。

2026-10-20 实际独立宿主烟测：在同一隔离target下 `cargo build --offline --example c2-media-probe --features c2-media-probe` exit 0；debug宿主SHA-256为`6F5752C0AD298509817B8B9A44D2F44B4D46E819F3A7C9E2119480BE0C8ABBA0`。被动运行一次，60.25秒后exit 0，无强制终止。汇总：manager 1/0成功/失败，manager-ready/event/fallback刷新1/0/29，snapshot非空/空/error为30/0/0，订阅新增/成功注销/注销错误5/5/0，观察elapsed 60001ms，快照均值/最大耗时3235/19042µs。由于event刷新为0，不称观察到事件驱动；空会话、精确1/5秒节奏、单次空广播、断线恢复及WebView收件/呈现均未测。运行前后`settings.json`及`.bak`哈希不变，HKCU Run相关值仅内存比较且不变；PID [omitted]正常退出后无残留Isle/probe进程。原release EXE/DLL SHA-256保持6FF00C…/2A5D20…。stdout原始汇总与空stderr及命令、哈希、未测边界见[烟测报告](performance/c2-media-probe-2026-10-20/REPORT.md)。这是一次功能烟测，无CPU/功耗结论；C2仍`[ ]`。

2026-10-20 第二轮真实事件覆盖尝试：复用既有debug宿主，hash `6F5752C0AD298509817B8B9A44D2F44B4D46E819F3A7C9E2119480BE0C8ABBA0`，未改源码/重建；PID [omitted] 于08:55:56.6936731Z启动，观察60.13秒、exit 0、无强制终止。实际汇总 manager成功/失败1/0，manager-ready/event/fallback刷新1/0/59，snapshot非空/空/error为60/0/0，token新增/成功注销/错误8/8/0，elapsed 60002ms，快照均值/最大7220/29021µs。event仍为0，故本轮仍未观察到事件驱动刷新；不推断用户操作是否触发或具体类型通过。stop/join正常返回；设置/备份hash与Run值内存比较均未变；产品Release hash不变。原始日志、启动命令与未测边界见[第二轮观察记录](performance/c2-media-probe-2026-10-20/REPORT.md)。无空会话/精确cadence/断线恢复/WebView/CPU/功耗证据，C2保持`[ ]`。

2026-10-20 event=0诊断源码批：旧回调忽略`sync_channel(1)`的`try_send`结果，`refresh_event`只计worker消费数，因此旧运行无法区分回调未进入、队列满/断开或事件未消费。默认关闭的`c2-media-probe`诊断现在聚合统计5类回调进入次数、入队成功/队列满/断开、成功快照中的播放状态变化与选中会话变化次数；选中会话只保留内存hash用于比较，不记录媒体/AppID/fingerprint。会话登记完成后打印一次相对时间和成功token数的`media_probe_ready`，最终仅打印一行汇总；无逐事件日志。production未创建probe observer。另修复媒体属性token注销计数将失败也算成功的问题。

宿主事件循环审阅：锁定本地Tauri 2.10.3源码显示`Builder::build`构造runtime/App，`App::run_return`进入`runtime.run_return`，`Builder::run`才调用`build(...).run(...)`；probe只build后休眠。监听worker通过`RoInitialize(RO_INIT_MULTITHREADED)`进入MTA，本地apartment测试断言MTA；Microsoft说明MTA用于无需消息循环的后台工作（[官方文档](https://learn.microsoft.com/en-us/windows/win32/com/processes--threads--and-apartments)）。这不能证明具体SMTC provider/WinRT事件路径不依赖Tauri宿主loop，但也没有找到依赖它的源码证据；未改宿主，事件循环仍是未决假设，event=0不归因于产品实现故障。

最终源码通过`cargo check --offline --lib`及`cargo check --offline --example c2-media-probe --features c2-media-probe`（Rust 1.94.1、既有隔离target），`git diff --check`通过；没有重建或运行probe，没有改旧日志、Release产物或用户状态。后续观察需等ready标记后再提示用户操作。新计数可以区分回调沉默与入队结果，但零回调仍无法区分宿主/provider派发限制与操作没有触发已订阅事件；也不证明前端收件/呈现、idle/空会话、恢复、CPU或功耗。完整审阅与边界见[诊断报告](performance/c2-media-probe-2026-10-20/REPORT.md#source-audit-and-bounded-diagnostic-update-not-run)。C2保持`[ ]`。

2026-10-20 诊断版构建（未运行）：旧debug宿主原路径及归档副本hash均为`6F5752C0AD298509817B8B9A44D2F44B4D46E819F3A7C9E2119480BE0C8ABBA0`，副本保存在`performance/c2-media-probe-2026-10-20/archive/c2-media-probe-pre-diagnostics-6F5752C0AD298509.exe`。Rust/Cargo 1.94.1、既有隔离target下执行`cargo build --offline --example c2-media-probe --features c2-media-probe` exit 0；诊断debug EXE `src-tauri/target/c2-offline-2026-10-17/debug/examples/c2-media-probe.exe`，28,863,488 bytes，SHA-256 `6BBD9B28888B1C4FAE59CD79E5B7D76C9E3BACD1C5DC7CEF323189790AD99CB5`。构建输入Cargo.toml/Cargo.lock/src/lib.rs/example哈希及完整命令见[诊断报告](performance/c2-media-probe-2026-10-20/REPORT.md#diagnostic-host-build-built-not-run)；未重建产品Release，未运行诊断EXE，未改两轮旧日志。

术语限定：`selected_session_changes`比较快照`source_display`（AppID/来源标识）的内存指纹，只能说明该来源标识变化，不能识别同一AppID下不同COM会话对象的切换。`media_probe_ready`只说明成功枚举后本轮manager/session注册尝试已经结束，并打印成功token数量；若total_tokens为0，不称订阅就绪，也不应据此提示用户操作。格式为`media_probe_ready elapsed_ms=<ms> manager_tokens=<n> session_tokens=<n> total_tokens=<n>`。后续观察命令与限制见上述报告：计时从listener task启动后开始；同步WinRT `.get()`不可取消，实际墙钟可能超过60秒；无WebView，不能验收前端投递/呈现。第三轮必须等待用户新的开始时机，本批无播放器/设置操作，C2保持`[ ]`。

2026-10-20 第三轮诊断宿主被动观察：使用已构建EXE `6BBD9B28888B1C4FAE59CD79E5B7D76C9E3BACD1C5DC7CEF323189790AD99CB5`，PID [omitted]于09:14:53.3234109Z启动。ready为`media_probe_ready elapsed_ms=30 manager_tokens=2 session_tokens=6 total_tokens=8`，在09:15:16Z回传协调窗口，约剩37秒；用户随后回复“已操作”，未提供动作类型/精确时刻，不作推断。摘要：callback current/sessions/timeline/playback/media=0/0/0/2/6；入队success/full/disconnected=8/0/0；manager-ready/event/fallback刷新=1/8/55；playback/source标识变化=2/0；snapshot非空/空/error=64/0/0；订阅add/remove/remove-error=8/8/0；elapsed=60002ms，快照均值/最大=7610/29682µs。观察到本宿主回调→入队→消费与正常token注销路径；本轮没有验证未出现的3类回调、UI前端交付、idle/空会话、恢复、CPU/功耗。

stdout/stderr分别为新文件`host-diagnostic-2026-10-20.stdout.txt`与`host-diagnostic-2026-10-20.stderr.txt`，stderr 0 bytes。首次PowerShell监控读取被Start-Process锁住的stdout文件时异常退出；probe仍运行，随后独立监督器核验同一PID的路径/启动时间并观察其约60秒后自然结束，无强制终止。该监督器未保留子进程句柄的退出码，故exit code未知，不从summary或PID消失推断。监控异常也使本轮捕获的设置/Run前态未能持久化；结束后设置/备份hash与前两轮记录相同，Run名称存在状态也匹配先前记录，但这不是本轮同进程内存前后比较，不能据此证明本轮期间值不变。原始Run值未输出。完整stdout和测量限制见[第三轮诊断报告](performance/c2-media-probe-2026-10-20/REPORT.md#diagnostic-event-observation-third-passive-run)。C2保持`[ ]`。

2026-10-20 无选中会话/空allowlist idle验证：新增example参数`--no-selected-player`只在宿主内存的`AppState`设置`selected_player_ids=Some(Vec::new())`，生产`selected_session`真实路径因此返回None；不代表Windows全局无SMTC会话/真实断线。旧`6BBD9B...` EXE已归档核验。默认library与feature example离线检查通过，之后一次debug构建exit 0，新EXE SHA-256 `04B91E53325727039C5F16BFBD7C557358415C605027D6E5A07655D39DCE3A56`；产品Release未重建。PID [omitted]仅运行此模式一次，60.37s自然结束，持有同一Start-Process句柄实际读取exit code 0，无强制终止。记录manager-ready t=26ms、empty、next=1000ms；fallback t=1028ms再次empty、next=5000ms；后续fallback快照t=6029至56040ms，均empty且next=5000ms，相邻实际观察间隔约5000–5003ms。共13次empty/0 error，空状态EventBus调用1、成功返回1、错误/序列化失败0；不代表前端收到（无WebView）。manager/event/fallback刷新1/0/12，五类回调均0，队列结果0/0/0，订阅8/8移除成功/0错误。保存的原始日志和源/EXE哈希见[空allowlist报告](performance/c2-media-probe-2026-10-20/REPORT.md#empty-allowlist-idle-cadence-run)。同一PowerShell进程内settings主/备份hash前后相同、整份HKCU Run名称/kind/data快照在内存比较相同；原始Run值未输出。验证仅覆盖空allowlist空选择：真实系统无session、服务断线/恢复、事件打断idle、WebView交付、CPU/功耗及产品集成仍未测，C2保持`[ ]`。

2026-10-20 有界监听器元数据复用（实现与离线验证）：生产媒体监听器actor现在只缓存单个当前选中COM session的title/artist/artwork；每次snapshot仍刷新timeline、playback、capabilities。具体`IUnknown`身份变化、首个选中、MediaPropertiesChanged、manager当前/会话变更以及5秒TTL阈值任一触发时重读属性和封面流。5秒是缓存过期阈值而非严格墙钟刷新上界：只在后续snapshot检查；播放时fallback为1秒，暂停时为2秒，因此暂停时通常约6秒重读，另加同步WinRT查询耗时。该TTL不延迟timeline/playback snapshot。dirty generation在容量1队列`try_send`之前递增，故Full合并不丢失失效；WinRT接口/缓存局限于MTA监听器线程。空选择和manager重建清缓存；两次连续SMTC查询错误后的既有manager重建保护保留。直接`get_media_info`命令继续每次完整读取。Probe仅新增metadata_reads/metadata_reuses两项。默认library和既有feature example离线check均通过，TTL/session/dirty策略两条targeted unit test通过，`git diff --check`通过；具体结果见[报告](performance/c2-media-probe-2026-10-20/REPORT.md#bounded-listener-metadata-cache-implemented-not-run)。之后probe运行与实际计数见本节后续记录；不推断CPU/功耗，C2保持`[ ]`。

2026-10-20 metadata复用被动观察：重建前将旧debug probe `04B91E53325727039C5F16BFBD7C557358415C605027D6E5A07655D39DCE3A56`归档并逐字节hash核对；feature example使用已有隔离target离线构建成功，新EXE SHA-256 `5D44DA0BBC34CA9F22D04CFC75DA197A98C17DAAFE92B459991E26135016C68B`。默认选择模式仅运行一次：PID [omitted]，启动09:44:32.9297825Z，60.32秒自然结束，实际进程句柄ExitCode=0，无超时终止。63次selected snapshot全成功、0空/错误；metadata_reads/reuses=14/49，显示该会话本轮49次复用而非进入属性/封面读取路径。manager/event/fallback=1/5/57；callback current/sessions/timeline/playback/media=1/0/2/3/0；refresh入队/full/disconnected=5/1/0；订阅新增/移除/移除错误=8/8/0。该观察未收到MediaPropertiesChanged，不能验收媒体事件切封面正确；不证明CPU/功耗、前端或产品集成。settings主/备份hash同轮前后相同，整份HKCU Run名称/kind/data仅内存比较相同；完整限界见[报告](performance/c2-media-probe-2026-10-20/REPORT.md#metadata-reuse-passive-observation-one-default-mode-run)及原始stdout/stderr。C2保持`[ ]`。

2026-10-20 Capture静态审查与错误退避修复：截图/全屏/录制按独立偏好门控；各自检测周期为50/500/500ms，三项关闭后worker每1秒只检查revision并在状态变化时发一次快照，不调用`GetAsyncKeyState`、全屏检查或AppCapture检测方法（worker仍在启动时初始化一次MTA）。审查发现`IsCapturingVideo`失败时原对象会被保留并每500ms重调；现在释放该对象并与创建失败共用30秒退避。关闭录制偏好也释放对象，false→true会立刻清除旧退避并查询；截图快捷键/全屏互不依赖。前端收到状态变化后按当前偏好恢复窗口，独立手动隐藏仍保留。AppCapture查询错误仍按既有fail-open行为把录制状态设false；若真实捕获仍在持续，退避期间可能恢复显示，隐私策略取舍记录为待审边界。锁定`windows 0.52`的AppCapture API只表示当前view且不是通用第三方录屏检测；UI提示已有此边界。离线`cargo check --offline --lib`和既有Capture间隔测试3/3通过，feature example check与`git diff --check`通过。未验证live检测调用数/WinRT成功、录屏覆盖、产品隐藏恢复、锁屏/恢复或优雅退出；审查记录见[Capture审查](performance/c2-media-probe-2026-10-20/CAPTURE_REVIEW.md)。C2仍开放，无Release重建/用户设置更改/提交推送。

2026-10-20 包含媒体metadata缓存与Capture退避修复的产品Release构建：旧隔离target Release `isle.exe` (`6FF00C4C1D2304EDA70551F85A0C356FB98E3812A8FD3862505F69F8CF0B3036`, 8,490,496 bytes)和`isle_lib.dll` (`2A5D20778EEBC7AC5F3A2F34C2B39F705200913B51F17626945CD8D4C2F51345`, 116,736 bytes)先分别归档到`performance/c2-media-probe-2026-10-20/archive/`，副本SHA逐一匹配；此前探针EXE和日志保留。前端使用现有node_modules运行项目脚本`npm run build`（npm 11.9.0/Vite 6.4.1，exit 0，3868 modules，35.66s），本次生成build目录28个文件、合计24,126,810 bytes；`package-lock.json`/`pnpm-lock.yaml` SHA保持`F775FE6985E01B3892AB9A265C1C4DA732E97AED3E042FD592069375D026F208`/`1D46CF1E46B93D6B36A32BB74F8BE4F9B181E24AA00790E6784685732AE00439`。然后使用rustc/cargo 1.94.1、现有隔离`CARGO_TARGET_DIR=src-tauri/target/c2-offline-2026-10-17`执行`cargo build --offline --locked --release`（默认features，exit 0，5m03s）。这不是`cargo tauri build`：未执行完整Tauri bundler、安装器或setup。

最终`src-tauri/target/c2-offline-2026-10-17/release/isle.exe`为8,500,224 bytes，SHA-256 `542851A3757B406C611DCE1C52DDECD88FD8D91D5F50D06F8DA06CD6D5E16FAB`；`isle_lib.dll`为116,736 bytes，SHA-256 `187C4B3AA9AD7026C370ECA41DD6B665A1D5EB4499E2078E68E883DC928CE03C`。HEAD为`48530f7343e84129aef34113dbef4dd18f7431ef`且未移动；该worktree在构建前已包含C1/A5/C2相关dirty改动，故产物来自当前dirty working tree，不代表干净HEAD构建。Cargo与前端lock哈希构建前后未变。没有启动产品或probe、没有改设置/Run、没有重建/删除旧日志/探针/命名候选、无提交推送；新产物仅是前端资源和默认Cargo Release。当前构建不提供Windows运行时、功能交互、性能/CPU/功耗证据，C2仍保持`[ ]`。

2026-10-20 有界新版Release进程树采样（一次）：启动前重新核对PID 29324路径、启动时间`2026-10-20T10:04:53.5926771Z`及映像SHA-256 `542851A3757B406C611DCE1C52DDECD88FD8D91D5F50D06F8DA06CD6D5E16FAB`；启动后该PID仍存活。同机另有PID 39872运行同一SHA的Release，未启动第三份、未停止/控制任何进程。只对29324执行`measure-native.ps1 -RootPid 29324 -WarmupSeconds 30 -Seconds 60`一次；60个采样记录、采样循环实际60.687秒，端点CPU计数窗口约60.94秒。目标树在所有样本中均为7个稳定进程：Isle根进程与6个WebView2子进程；未见子PID加入/退出/重启。按各进程原始`TotalProcessorTime`累计端点差复算，根进程从1.468750s到1.671875s，增加0.203125 CPU秒；6个子进程端点差均为0，合计目标树同为0.203125 CPU秒，约0.3333%单逻辑核或12逻辑核机器容量的0.0278%。子进程的0差值表示本次端点计数未增加，不代表任何更细粒度CPU活动必为零。根进程线程47–50（均值48.33）；整树线程211–215（中位数212）。脚本另写了系统汇总CPU字段（60样本中位数13%、范围2–61%），它受第二个Isle实例及主机其他活动影响，不作为产品归因或比较结果；没有同条件旧版基线，不声称性能提升。配置、播放状态、窗口/显示器/DPI状态均未可靠观测，故本样本不称“idle”或受控场景；Label中的`idle`只是文件标识。进程树由PID父子关系动态枚举，跨采样间短命子进程可能未被观察；目标树包括WebView2，但不含其外部broker/系统服务。采样期间未设置profile、未发送输入、未读写设置/Run；仅在新证据目录写JSON/JSONL。脚本会枚举全机进程以判定子树，并记录一项系统总CPU计数，但不记录其他应用的逐进程CPU/内存/线程数据。原始记录与复算边界见[有界Release运行记录](performance/c2-media-probe-2026-10-20/runtime-idle-PID [omitted]/REPORT.md)；JSON SHA-256 `F4E6407EDF8AE73F5D457BAC51AA188F2BAF61A8C54AA06C572B9840AE7C4709`，JSONL SHA-256 `03EAD9EB04D13EB05F714231D0A5AEEA3E52CB0E4CAD574CA9DA21DD863082BE`。C2仍开放；未改源码/用户配置、未启动或关闭产品、未提交推送。

2026-10-20 工作区变化通知实现与离线验证：主窗口使用独立Win32 `SetWindowSubclass` 观察 `WM_DISPLAYCHANGE` 及仅在 `wParam == SPI_SETWORKAREA` 时接受的 `WM_SETTINGCHANGE`；WndProc不枚举显示器、不直接操作窗口，只以原子pending合并后异步发送 `isle-work-area-changed`。前端复用既有350ms恢复/布局重算路径，原5分钟低频兜底保留。安装在主WebView加载完成后经主线程投递，按HWND幂等；销毁代次阻止迟到安装，`WM_NCDESTROY`采用一次性清理保护，在`DefSubclassProc`返回后释放回调状态。默认Rust库与既有`c2-media-probe` feature example的offline/locked检查、既有窗口测试15/15及`npm run build`（Vite 6.4.1，3868模块）通过；未构建/运行新的Release或探针。Release文件未修改，其预期SHA仍为EXE `542851A3757B406C611DCE1C52DDECD88FD8D91D5F50D06F8DA06CD6D5E16FAB`、DLL `187C4B3AA9AD7026C370ECA41DD6B665A1D5EB4499E2078E68E883DC928CE03C`。原生工作区/显示器消息投递、多显示器布局恢复、应用销毁/重建现场路径和CPU/功耗仍未测；`RemoveWindowSubclass`失败仅记录错误，清理由Windows销毁回调及默认处理返回后的终止窗口生命周期兜底。完整范围见[工作区通知记录](performance/c2-media-probe-2026-10-20/WORK_AREA_NOTIFIER.md)。C2保持`[ ]`。

2026-10-20 cursor DPI查询优化（实现与离线验证）：锁定依赖源码确认`WebviewWindow::scale_factor()`经Tauri dispatcher调用Wry `window_getter!`，同步发送`WindowMessage::ScaleFactor`并等待事件循环回复；tao Windows后端读`WindowState.scale_factor`互斥状态。旧cursor worker每轮同步查询，频率随原33/50/200ms监视节奏变化。现在由现有全局`.on_window_event`一次性接收每窗口`ScaleFactorChanged`并更新label专属缓存；worker启动时查询一次，平稳期每1秒查询兜底，事件可即时生效，cursor采样及原33/50/200ms自适应间隔、坐标换算和ignore输入策略保持。事件revision使并发事件优先于在途初始化/兜底查询；scale必须为有限正数；无效事件或查询错误使缓存失效并沿现有fail-open路径重试，避免永久沿用旧DPI。Destroyed移除该label缓存并使活跃监视generation失效；重启初次查询刷新缓存，并校验传入窗口HWND仍与当前同label窗口一致，因此同label新窗口不继承旧值。重启未注册额外listener。纯逻辑测试覆盖事件/初始化竞争、非法值、查询失败不覆盖新事件、同label销毁重建/窗口缓存隔离。默认lib check、既有`c2-media-probe` feature example check通过；窗口测试模块13/13通过，`git diff --check`通过。未运行产品、未重建Release；没有本机混合DPI行为实测或CPU归因，仍需跨不同缩放显示器移动/系统缩放变化下验证ScaleFactorChanged到达、透明交互区坐标/穿透一致、同label重建及原间隔。实现和验证记录见[cursor DPI记录](performance/c2-media-probe-2026-10-20/CURSOR_DPI_CACHE.md)。C2保持`[ ]`。

## 35. 网易云模式按钮与进度跳转接入（2026-10-21）

- [ ] **M-Netease：模式按钮与切曲读回验收。** 主岛四态模式按钮及 `cycle_netease_playback_mode` → `LocalCdpClient` 已接入；后端仅在读回确认 requested/observed 一致且 `verified=true` 后返回新模式。CLI 模式循环和完整 App fixture 的 query/cycle/readback/error/busy/切曲路径已通过；真实网易云中的模式按钮结果及切歌后状态恢复仍待实机验收。用户已确认进度滑条拖动能控制网易云，±15 秒快捷控件按用户要求移除，不属于当前验收范围；另确认截图显示问题已解决，但该反馈未验证模式或性能。详见[本轮证据记录](performance/netease-mode-seek-2026-10-21/REPORT.md)。

## 36. 网易云进度来源设置与性能对照（2026-10-21）

本需求统一跟踪于 Phase 3 的 **I3-NCM2**，原 NPS1 为同一需求的别名，不另设重复任务。范围、设置行为、性能对照和验收均以 I3-NCM2 为准。首个设置/actor实现切片已进入工作树，legacy 默认仍为 `real` 以兼容旧行为；该值不代表性能推荐，原生/多 WebView 验收与性能准确度对照仍待完成。

网易云接口与 seek 实测证据见[本轮记录](performance/netease-mode-seek-2026-10-21/REPORT.md)。

2026-10-22 C4被动运行时采样收口：使用既有`measure-native.ps1`对已运行候选做一次只读动态进程树采样，10秒warmup+60秒采样，共42条/60.971秒；7个进程身份全程稳定。树CPU中位数1.2527%机器容量（12逻辑核；15.0325%单核等效），范围0–5.2715%；GPU 3D引擎中位数0%、最大1%（不代表全GPU利用率）。实际App可见/页面/捕获/播放状态未可靠记录，不能称Idle或clock/weather-only；无collector/IPC/effect计数、无匹配前测。采样脚本的外层`$LASTEXITCODE`判断错误，漏掉原计划的即时后置核验；2026-10-22复核时PID10580已不存在，settings与Run摘要当前匹配前置值，但跨日检查不能证明采样结束时状态。详见[C4采样报告](performance/c4-weather-snapshot-2026-10-20/REPORT.md#passive-dynamic-process-tree-sample-2026-10-21-uncontrolled-state)及原始JSON/JSONL。C4保持`[ ]`，无性能收益结论。

## 37. R2 verified dead-path cleanup slice (2026-10-22)

静态调用图确认 `src-tauri/src/services/spectrum.rs` 的占位函数 `process_spectrum_data` 无调用者，`services` 模块也不对 crate 外暴露；删除该 service 文件及模块声明，并同步移除只为它服务、未在 crate 顶层导出的 `SpectrumData` 模型。全仓搜索确认 `FeatureRail.svelte` 没有导入者或动态引用，删除该旧 UI 组件。修正 `lib.rs` setup 清单中“启动音频可视化器”为“创建频谱捕获状态（按可见消费者按需启停）”，与当前按需消费者生命周期一致。`pnpm run check`（0 errors/warnings）、离线 locked Rust `cargo check`及`git diff --check`通过；唯一 Rust warning 为已有 `NeteaseTimelineSample.track_id/play_id`未读字段。未做Release构建/进程切换、未运行播放器或UI自动化。该切片只清理两条已证实死路径与一条陈旧注释，不完成整个R2；其余dead-path审计以及bundle/Release验证仍待进行。源码、锁文件和既有dirty均保留在m3 worktree，没有提交或推送。

后续排除审计：`featureRail.ts` 仍被 App、Studio、IslandSurface 和 `islandGeometry.ts` 使用；`islandShapeMotion`/motion easings 有生产调用或单测用途；legacy `autoHide`、`edgeShoulderRadius`、Timer unversioned response、settings snapshot 与运行时事件均有兼容读取/派发路径，未删除。仓库范围搜索未发现 `AGENTS.md`。当前没有足够证据支持再删一个候选；R2其余清理应在逐项调用图与旧配置迁移验证后进行，不因“看起来未使用”扩大删除范围。

2026-10-23 command/event/config 调用图补审：按实际 `generate_handler!` 块与前端静态 `invoke` 字符串比对，找到 5 个无调用的旧包装（`get_netease_duration_cmd`、`set_hide_settings_button`、`set_hide_monitor_selector`、`set_hide_floating_window`、`set_expanded_corner_radius`）：均未注册为 Tauri command、Rust 端无对应公开函数，且前端全仓没有使用点；已移除这些本地 API 包装。其余 58 个不同的前端字面量调用均与注册命令匹配。10 个注册但无同名字面量调用的命令中，频谱启停通过队列动态传入命令名；其余保留，因为该静态审计不能排除间接/兼容调用，不以字面量搜索单独删除。事件链交叉核对确认 `media-update` 由 Rust event bus 到 `onMediaUpdate`；`spectrum-data` 由音频发布器到 `spectrumStore`；`settings-updated` 由设置快照写入方到各 WebView settings store；`isle-work-area-changed` 及 `isle-window-visible` 分别有窗口事件监听。`audio-spectrum` 旧常量当前无生产调用，实际频谱通道是 `spectrum-data`；`settings-committed` 在仓库内只有 Rust 发射点、未发现前端监听。本轮保留这两个旧事件入口，静态仓库搜索无法排除外部/兼容订阅。旧配置保留 `autoHide` 与 `edgeShoulderRadius` 的归一/serde alias；Timer 窗口保留未版本响应兼容，并以版本化响应阻止旧载荷回退；设置 store 仍先订阅再获取快照且以事件 revision 防止旧快照覆盖新事件。静态调用图核对和包装清理完成；最终有界dead-path/config审计及验证见2026-10-23 closeout。

2026-10-23 bundle/Release验证：`pnpm run check`（0 errors/warnings）和 `pnpm run build` 通过；Vite 产物28个文件、24,155,403 bytes。使用Rust/Cargo 1.94.1执行 `--offline --locked --release --features custom-protocol` 成功，Tauri build output确认 `cargo:dev=false` 与 `cargo:rustc-cfg=custom_protocol`。生成的 `src-tauri/target/release/isle.exe` 为23,470,592 bytes，SHA-256 `724C5F6B287F497FF1D1FB94D9DC1A993299B7561FF57CB9CCD325711214E2EA`；配套 `isle_lib.dll` 为116,736 bytes，SHA-256 `7F019FFB41EA178E05976C9A68050DB75C7290882610568D7227DB0048C908BF`。release dep-info包含当前前端全部16个JS及7个CSS资源；本轮未启动/替换应用，未做网易云或UI操作。

2026-10-23 bounded dead-path/config closeout: `stableEnvelope`、`expansionForWidth`、`expansionForGeometry` were referenced only by obsolete geometry tests, not production callsites; removed those helpers and tests plus unreferenced `FEATURE_RAIL_RESERVE` and `ISLAND_SPRING`. Removed the three files under `src/styles/` after confirming no app/import-graph references (their imports only linked the orphan stylesheets to one another). Corrected the Timer local-storage comment: that window is created on demand. Preserved old event names and legacy settings/Timer response paths where repository search cannot prove external/indirect compatibility use.

Lock/toolchain decision: `pnpm-lock.yaml` is authoritative for project/release work because the project has pnpm-only `onlyBuiltDependencies` and verified scripts use pnpm 10.33.2. The tracked `package-lock.json` remains a secondary compatibility lock; release commands use the pnpm lock. Native release toolchain is Rust/Cargo 1.94.1. Extended the legacy settings migration test to assert both `autoHide` and `edgeShoulderRadius` aliases.

Current-source validation: `pnpm run check` passed (0 errors/warnings); `pnpm exec vitest run src/lib/islandGeometry.test.ts` passed (20/20); Vite build passed (28 files, 24,155,204 bytes). `cargo test --offline --locked --lib migrates_legacy_auto_hide_to_fullscreen_capture_setting` passed (1/1). `cargo build --offline --locked --release --features custom-protocol` passed with Rust/Cargo 1.94.1; existing warning remains limited to unused `NeteaseTimelineSample.track_id/play_id`. Current `isle.exe` is 23,467,520 bytes (SHA-256 `8F6F8D509DC6B6BE02BE172556D3CAA32D753B81F9578C662D6147C9A433A0DE`), paired `isle_lib.dll` 116,736 bytes (SHA-256 `DADAE48FB95C605976FBE133682389F4841F4BA4647E289952490387E07B6762`). `git diff --check` passes; no Isle process was running, and no app/UI/player operation was performed. R2's bounded repository audit is complete. Remaining plan items stay under their existing runtime, real-player, native-interaction, and controlled-measurement acceptance; this pass found no further offline R2 cleanup to extend as a general scan.
