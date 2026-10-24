# 已完成优化任务

2026-10-24：从[优化计划](OPTIMIZATION_PLAN.md)第11节移出的已勾选任务。保留原任务与验收边界；完成不表示所有性能或原生验收目标均已达标。

- [x] **B1 · 🟠 P1 — Restore a usable verification entry and audit existing assertions**
  - **Problem:** Playwright `web:dev`脚本不存在；默认reduced motion绕过真实动画；旧布局断言与当前导航结构可能不一致。
  - **File:** `package.json`、`playwright.config.ts:13–20`、`ui-tests/island-preview.pw.ts:325–382`、`ui-tests/island-navigation.pw.ts`、`ui-tests/island-fixture.svelte`、`scripts/build-performance.mjs`。
  - **Reason:** 后续“运行测试”必须真的能启动且检查所需行为，不能用失效入口或把新期望值机械替换来获得pass。
  - **Proposed change:** 经确认后统一真实Vite启动命令/fixture构建入口；核实300px/200px等断言原意，按产品布局契约修正；保留reduce测试，另加no-preference动画和打断场景；选定一个权威JS lockfile与安装方式。
  - **Expected benefit:** 每阶段有可重复的TS/Svelte/纯函数/UI回归检查入口。
  - **Risk:** 更新断言掩盖布局回归，浏览器fixture仍不含native命中/性能。
  - **Verification:** Windows执行`npm ci`（若选npm锁）、`npm run check`、`npm test`、`npm run test:ui`、`npm run build`；Rust执行适用的`cargo test`及`cargo check`。列明任何历史失败与覆盖盲区，不将fixture pass替代native Benchmark。本次未安装依赖/运行这些构建测试，避免新增业务/构建产物。

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

- [x] **M3 · 🔴 P0 — Bound input, decode and cache bytes; reject crash inputs**
  - **2026-10-10 回归修正验收：** 合规静态 JPEG/PNG 完整有界校验后保留原编码；普通 Canvas 保留受 1280 edge 限制的源栅格，像素模式及全部资源/非法输入保护保留。全新匹配 Release C 前后各三轮：Tree/Renderer 峰值 −1.10%/−1.57%，恢复末分钟 +2.07%/+3.39%，CPU +8.79%（+0.168 个百分点），均在预设回归门槛内；下载缓冲峰值 −98.04%。三档 renderer DPR 1/1.5/2 前后各 100 首画质及每 25 首重开回归通过，Detached Canvas=0。M3 按上述范围勾选，不宣称整体内存收益；GPU 长期回稳和 Windows 实际跨屏 DPI 仍待验证。见 [修正验收报告](performance/m3-correction-2026-10-09/REPORT.md)。
  - **2026-10-08 实施状态：** 已落实图片/下载/解码/并发/缓存/Canvas 预算及错误输入校验，50 项 Rust、73 项前端、9 项 UI 及实际原生命令验证通过。完整同条件 Release C 对照未完成，暂不勾选。见 [M3 实施记录](performance/m3-2026-10-08/IMPLEMENTATION.md)。
  - **2026-10-09 Release C 对照：** 相同 M2 修复、固定 100 首、连续浮窗、前后各三轮正式采样及五分钟恢复已完成。三轮中位数：GPU 峰值 −54.88%，但进程树 Private Commit 峰值 +24.17%、Renderer 峰值 +54.55%、CPU 平均值 +43.54%；恢复末分钟 Renderer Commit +70.42%。下载缓冲/Canvas backing 下降不能代替整体内存收益验收，M3 保持 `[ ]`；GPU 长期回稳仍未证实。见 [完整对照报告](performance/m3-release-c-2026-10-09/REPORT.md)。
  - **Problem:** SMTC按stream size分配且u64转u32未验证；HTTP缺Content-Length时整包读取后才超限拒绝；原图尺寸解码/Canvas和数量LRU没有项目字节预算；pixel_size=0进入step_by(0)会panic，release abort。
  - **File:** `src-tauri/src/services/media.rs:133–170,732–763`、`src-tauri/src/services/cache.rs:287–337`、`src-tauri/src/services/image.rs:19–37,47–77,130–145`、`src-tauri/src/utils.rs`、`src/FloatingWindow.svelte:948–1028,1066–1081`、`src-tauri/Cargo.toml`。
  - **Reason:** 现有磁盘/条目上限不能控制下载在途、原始decode、Base64和Canvas峰值；一个可达command错误输入不应终止应用。
  - **Proposed change:** 分块下载至有上限临时文件并在读取中止损，限制SMTC字节/转换溢出、图片像素/尺寸与处理并发；按可见尺寸×DPI制作缓存缩略图。补总字节LRU、短key与处理结果预算；pixel_size校验范围并返回AppError，查大值求和溢出。避免process_image像素化分支重复完整decode。
  - **Expected benefit:** 低峰值、更少JS/GPU backing、错误输入不crash；100首场景可进入明确有界平台。
  - **Risk:** 太低上限损害封面清晰度/合法MV；WebP改变画质/处理时间；format检测与SMTC当前固定PNG MIME需一并核实。
  - **Verification:** chunked无Content-Length超限响应、截断图片、异常像素尺寸、SMTC超限、pixel_size=0/极大值均可控错误；正常1×/1.5×/2×DPI封面清晰。用C对比下载并发、decode内存峰值、Renderer/GPU；不能仅看磁盘配额。

- [x] **M4 · 🟠 P1 — Coalesce artwork work and cancel superseded requests**
  - **2026-10-12 验收：** 最终源码 Release C 前后三轮、每轮 100 首连续浮窗及五分钟恢复完成；native decode 中位数 605→527（−12.9%），download-input 403→315（−21.8%），CPU 平均 −9.47%，first-paint −25.38%，HD-paint −14.80%。进程树峰值 +0.32%、Renderer 峰值 +3.72%，GPU 峰值 +0.93%；恢复末分钟进程树/Renderer Commit −2.22%/−2.92%。最终 Release 真实播放器复验 100 首和四次重开通过，Canvas 连通且像素一致 100/100，五个快照 Detached Canvas=0。首次独立诊断在第 6 首等待图片完成时失败；用同一最终 Release、新目录重跑后该曲及完整序列通过，故保留首次失败记录并不归因于代码根因。84 项前端、56 项 Rust 通过，Svelte 检查零错误/警告。M4 勾选并独立提交；峰值内存轻微上升，不宣称整体内存峰值收益。M5 及后续阶段不动。见 [M4 验收报告](performance/m4-2026-10-11/REPORT.md)。
  - **Problem:** App/浮窗各自高清解析；duration/MV搜索也下载图片；快速切歌旧Rust下载仍完成，process_image请求Map只按同key合并、无总并发上限；浮窗主色无revision检查；同歌分支忽略新albumArt。
  - **File:** `src/App.svelte:87–121,965–968,1512–1534`、`src/FloatingWindow.svelte:169–238,329–349,782–814,948–994`、`src-tauri/src/services/media.rs:381–430,891–913`、`src-tauri/src/services/cache.rs:287–337`、`src-tauri/src/utils.rs:27–81`。
  - **Reason:** 迟到结果丢弃并不取消下载/decode，双窗口重复请求放大CPU、内存与网络；asset URL与Rust path接口不匹配导致重复失败。
  - **Proposed change:** 后端按source/track/artVersion做single-flight，元数据查询与图像下载分开，前端只订阅权威结果；请求带cancel token或显式过期检查/全局并发预算，旧歌/窗口关闭不继续昂贵处理。规范cache key→path/asset URL接口；主色和绘制结果检查revision；同歌允许artVersion更新。
  - **Expected benefit:** 快速100首切换时工作有界，不错图/错色，不为拿时长或MV ID重复下载封面。
  - **Risk:** 取消共有single-flight时误伤另一窗口；缓存key缺source会串歌；过度取消在弱网络下反复失败。
  - **Verification:** 双窗口同时请求同歌计数只一次；同名不同source不混；故意让第1首最后返回，第100首始终最终结果。关闭窗口后在途任务有界退出；C记录network/decode/IPC次数与private commit。正常HD升级无需重新切歌即可显示。

- [x] **M5 · 🟠 P1 — Close late-listener, Image and animation teardown gaps**
  - **Problem:** 历史审查发现 EventManager await listen 后未复查 destroy；IslandSurface 旧封面 rAF 循环现已移除，剩余 await tick 后焦点 DOM 写入需检查销毁状态；App 预加载 Image、恢复等待和 placement Animation 需统一取消；图片超时必须清 handler 与 source。
  - **File:** `src/utils/eventManager.ts:30–64,104–115`、`src/lib/asyncCleanup.ts`、`src/lib/components/island/IslandSurface.svelte:333–438`、`src/App.svelte:87–121,387–419,802–865,1038–1064,1512–1534`、`src/lib/spectrumColors.ts:15–31`、`src/FloatingWindow.svelte:479–489`、`src/lib/components/time/RollingDigit.svelte:41–48`。
  - **Reason:** 多数cleanup已有实现，应补具体缺口。EventManager缺陷目前是潜在封装竞态；有界双rAF本身不证明长期泄漏。
  - **Proposed change:** async注册返回后若owner已销毁立即unlisten；用dispose/epoch取消rAF/WAAPI/timeout、图像handler与source；图片请求有逐图超时；复用createAsyncCleanup，不新增另一个全局资源注册器。
  - **Expected benefit:** 卸载后不会继续更新状态或保留DOM/closure，开关窗和中断动画可稳定回收。
  - **Risk:** 把document生命周期listener误当组件泄漏；销毁前强制取消共有任务使其他窗口失去状态。
  - **Verification:** mock延迟listen返回，在destroy之后unlisten调用一次且getListenerCount为0；动画中卸载后无额外frame写入；图像超时后pending集合归零；匹配 Release 各进行10次预热与100次副窗开关，DOM/监听器无净增长；句柄净增由基线+612降至+69，增长集中在前10次测量开关后平台。进程树私有提交在10秒恢复后仍各增约30.36 MiB/25.32 MiB，前后差异不足以证明长期内存回稳；保持该限制，不归因为泄漏。没有活动媒体会话，Release image request计数为0；图片错误/超时/Abort由单元测试覆盖。M5 已勾选，详见 [M5 验收报告](performance/m5-2026-10-12/REPORT.md)。第6首超时继续作为未定位现象记录，复现时补图片错误和请求时序。

- [x] **M6 · 🟠 P1 — Serialize settings commits and make persistence recoverable**
  - **Problem:** update_settings读snapshot后跨await保存，多窗口可覆盖彼此patch；save先改内存再spawn_blocking写文件，写入顺序可反转；fs::write直接覆盖，不是原子提交；Studio吞掉保存错误。已完成唯一串行事务、临时文件替换/备份恢复和可重试错误提示；按用户 2026-10-12 最终报告通过正确性与原生 Studio 验收。
  - **File:** `src-tauri/src/commands/settings.rs:59–113,209–230,421–466`、`src-tauri/src/services/settings.rs:58–72`、`src-tauri/src/commands/window.rs`中的设置写入、`src/Studio.svelte:231–247`。
  - **Reason:** 与动画/IPC优化同时发生时容易被误诊为“状态竞争”；配置丢失/损坏会造成长期稳定性问题。
  - **Proposed change:** 后端唯一写入队列将merge→validate→persist→publish变成带revision事务；使用Windows可替换的原子落盘方案，失败保留可恢复旧版本与明确UI错误。legacy setter与浮窗位置写入也进入同owner；避免锁持有期间做长Windows调用/磁盘I/O。
  - **Expected benefit:** 多窗口更新不丢失，崩溃/写入失败后配置可恢复，广播有权威版本。
  - **Risk:** 外部副作用（topmost/autostart）不天然原子，需要约定失败/回滚；Windows重命名覆盖语义不能照搬Unix。
  - **Verification:** 按用户提供的 2026-10-12 最终报告，61 Rust tests passed / 0 failed / 1 ignored，包含100个并发 patch/位置事务、写入失败补偿、外部操作失败和备份恢复；95 项 Vitest、Svelte 零错误/警告、production/Release 构建通过。已确认测试EXE缺少Common Controls v6 manifest；补测试宿主资源链接后可启动，Windows只读备份handle的flush错误已修正。真实隔离Studio的保存失败、保留修改、关闭拦截、重试及重启备份恢复通过。匹配Release四样本保存延迟中位数27→39.5 ms（+12.5 ms）；此回归继续保留，未事先设定预算，不宣称保存性能改善。原子写入合成测试约7.62 ms/次，与Studio时延分开记录。M6按正确性验收勾选；用户报告的 Windows 基线提交为 `48530f7`（本会话未解析为远端完整SHA）。相关完整结果依据用户最新报告，见 [M6 实施与验收记录](performance/m6-2026-10-12/REPORT.md)。

- [x] **C1 · 🔴 P0 — Move blocking native work off the UI thread**
  - **Problem:** 原始审查中同步control_media包含WinRT.get和150ms sleep；seek/shuffle/repeat、取色、pixelate、音量设备COM查询与部分文件操作也同步。process_image虽async，函数内同步decode/encode仍占Tokio worker。
  - **File:** `src-tauri/src/commands/media.rs:93–142`、`src-tauri/src/commands/audio.rs`、`src-tauri/src/services/image.rs:19–37`、`src-tauri/src/services/system_audio.rs`、`src-tauri/src/commands/cache.rs`、`src-tauri/src/commands/settings.rs`。
  - **Reason:** Tauri2同步command默认运行主线程；150ms已经远超16.67ms帧预算，也可能拖住cursor monitor的scale_factor请求和watchdog。不是单纯调整CSS可解决。
  - **Proposed change:** 对确定阻塞链路使用async command+受限blocking pool或专属COM actor；sleep改异步延迟/事件确认，保留操作完成与错误传播。CPU图片处理用有界blocking任务；所有worker遵守M1初始化。后台迁移已实现，当前只补验收和剩余长尾诊断。UI线程只处理必须在该线程的窗口操作。
  - **Expected benefit:** 点击播放/拖音量/封面处理期间主线程不长时间停顿，动画和点击穿透响应更稳。
  - **Risk:** 任意跨线程传COM对象不安全；仅加async关键字不足；返回过早导致假播放状态；blocking任务不可被强制取消需并发/过期预算。
  - **Verification:** bounded worker迁移和既有工程检查已完成；第2批独立验收了WinRT false/error传播、真实Apple播放/暂停/时间线、seek=false不支持及只读UI、无会话和明确标记的UI故障注入。真实声明支持seek的会话不可用，有效seek为N/A；不是Apple必须支持seek的缺陷。第3批资源就绪/真实CPU读取区间及旧保存A5诊断配对通过独立验收，不代表第2批新Release性能。第4批已取得并复算旧r4原始输入：95.0015/54.9938ms源行成立，但frame无QPC、16小时偏移只有推算、last mutation未保存，旧“实际动画内”分类撤回，归属无法确认。WPR失败且无线程栈，不能按CPU busy/低GPU归因；本轮不采集/提权。C1保持 `[ ]`，A5仍 `[ ]`；新构建匹配性能、原生交互/请求成本与必要时真实停顿栈继续待办，详见第22节及[C1报告](performance/c1-2026-10-12/REPORT.md)。 第7批固定8次及PID来源修订仅证据验收通过（4 page/3 board/1 unobserved），首击仍未通过。第8批锁定依赖语义审查及持久window region纯离线原型22项窗口测试/15项媒体测试、标准Release构建通过；仅cfg(test)候选待审/待现场验证，默认生产路径未安装修复，不能用该Release直接验收首击。详见第25–26节。 第9批默认关闭的独立静态region lab适配器仅待审；真实UI/所有权/宿主失效路径已编译，产品首击/视觉/OS路由和完整生命周期仍未知，不替代原8次hover回归，见第27节。 第10批只准备该构建的4次静态lab协调工具，默认dry-run，28项JS/9项owner/5项日志与释放故障测试及CompileOnly通过；第9批编译交付已独立验收，本批工具待审、现场未允许，真实投递/no-activate/恢复未知，见第28节。 第11批单次已授权静态实验以0次测试输入停止，原定10秒nativeReady未确认；16日持久配置/Run恢复独立回读通过，历史完整UI恢复未证实，C1/A5仍[ ]，见第29节。

- [x] **A5 · 🟠 P1 — 放慢展开/收起并柔化鼠标 Hover 的扩大**
  - **Problem:** 用户反馈展开/收起过快、鼠标进入时扩大僵硬。当前 GitHub `6ef0855` 的展开/收起均为240 ms，形变统一使用 cubic-bezier(0.23,1,0.32,1)，起步较陡；hoverDuration=140虽定义但未使用，Compact↔Hover实际也走240 ms与相同曲线。compact层提前使用目标width/height与offset，使内容先跳位；Hover离开时轻微height差还会被当成向外展开进度，可能错误启用移动封面。
  - **File:** `src/lib/islandMotion.ts`、`src/lib/components/island/IslandSurface.svelte`、`src/App.svelte`的手动隐藏等待调用、`ui-tests/IslandFixture.svelte`、`ui-tests/island-motion.pw.ts`。
  - **Reason:** 仅增加时长仍可能在最初几帧完成大部分变化；需区分小幅Hover与展开/收起节奏，让尺寸、圆角和肩部使用同一次选定的时间与曲线。此任务针对交互手感，不能替代C1呈现长尾诊断。
  - **Proposed change:** 独立候选：展开420 ms、收起360 ms，使用 cubic-bezier(0.25,0.1,0.25,1)；Compact↔Hover使用240 ms及起始斜率为0的 cubic-bezier(0.3,0,0.2,1)。用前后目标mode选择动作，每次重设从当前tween值继续。纯Compact/Hover时内层采用当前外壳尺寸、outwardProgress保持0；过渡到展开时内层尺寸不超出当前外壳，避免尚未完成Hover就点击展开时跳位。Hover形变期间断开封面端点ResizeObserver，settled后再测；展开/收起保持既有共享封面路径。保留transition revision、低动效瞬时切换和固定native host。普通Page/设置/style布局改变保留240 ms既有曲线；不引入新spring或全局控制器。App手动隐藏等待继续读取同一collapseDuration，避免仍只等待旧240 ms；该固定等待仍不等于实际settled，后续由A1/A3协调完成状态。
  - **Expected benefit:** 让展开/收起有可感知的推进和收尾，小幅Hover柔和起步，快速进出可连续反向；不宣称解决95/55 ms长尾、提升FPS或降低CPU/内存。
  - **Risk:** 延长动画可能增加现有逐帧Region IPC的总次数、持续layout/clip成本及手动隐藏的等待；Hover抖动、动作反向和Timer/页面变化可能重设动画。仅改时长无法修复内容布局瞬跳或原生线程等待；不以降低动作速度掩盖掉帧，不擅自增加弹跳/过冲。
  - **Verification:** 候选最初在隔离的Linux工作树实现，基于GitHub `6ef0855`；补丁已应用到当前Windows工作树，Windows 10/10动画回归通过，并完成保存Release二进制的动画独立配对诊断。修改前后95项单测均通过，Svelte零错误/警告；候选production build通过；2026-10-13补齐无代理及有代理配对的16/16同页并发覆盖，资源/IPC成本和功能缺口见第19节。修改前7项原动画UI回归通过；最终候选10项动画UI回归通过，含4edge、封面正反向、scaled preview、reduced-motion、较缓展开、真实鼠标Hover进入/离开无初始跳位及未完成Hover时点击展开。完整样式采样见本节后文。后续真实Release在相同C1代码上只比较A5差异，固定DISPLAY2/200 Hz、暂停33%、设置/DPI和场景，覆盖4edge与floating/edge、鼠标快速进出/点击、动画中反向、封面连续性、键盘焦点、手动隐藏和reduced-motion。把PresentMon间隔两端与实际动画起止对齐，保留95/55 ms及全部跨边界原始样本分类；记录每次动作时长、实际呈现、Frame Time/Long Task、IPC/second及每动作总量、native resize次数、CPU/GPU和终态命中。完整匹配Release性能、原生交互/IPC/资源成本及用户手感验收完成前保持 `[ ]`，不提交或推送候选；C1仍使用原参数对照。 第5批当前媒体修复Release原生取证完成：四edge及切回floating首次命中未通过；部分floating/已就绪真实封面终态、独立透明穿透、隐藏恢复和原生页低动效有证据，焦点可见性/跨DPI/用户手感未验收，loaded运行时字段N/A。完整矩阵保持未通过（第23节）。 第7批固定8次及PID来源修订仅证据验收通过（4 page/3 board/1 unobserved），首击仍未通过。第8批锁定依赖语义审查及持久window region纯离线原型22项窗口测试/15项媒体测试、标准Release构建通过；仅cfg(test)候选待审/待现场验证，默认生产路径未安装修复，不能用该Release直接验收首击。详见第25–26节。 第9批默认关闭的独立静态region lab适配器仅待审；真实UI/所有权/宿主失效路径已编译，产品首击/视觉/OS路由和完整生命周期仍未知，不替代原8次hover回归，见第27节。 第10批只准备该构建的4次静态lab协调工具，默认dry-run，28项JS/9项owner/5项日志与释放故障测试及CompileOnly通过；第9批编译交付已独立验收，本批工具待审、现场未允许，真实投递/no-activate/恢复未知，见第28节。 第11批单次已授权静态实验以0次测试输入停止，原定10秒nativeReady未确认；16日持久配置/Run恢复独立回读通过，历史完整UI恢复未证实，C1/A5仍[ ]，见第29节。

- [x] **R2 · 🟢 P3 — Remove verified dead paths and align documentation/config**
  - **Problem:** 占位Spectrum service、FeatureRail旧UI、未使用easing/旧布局参数/legacy事件/宽泛API可能误导profiling；setup注释仍写启动Spectrum，Timer注释仍写pre-created，与实际按需行为不同。
  - **File:** `src-tauri/src/services/spectrum.rs`、`src/lib/components/island/FeatureRail.svelte`、`src/lib/islandMotion.ts`、`src/lib/islandGeometry.ts`、`src/utils/eventConstants.ts`、`src/lib/api/settings.ts`、`src-tauri/src/lib.rs`、`src/TimerWindow.svelte`、`src/styles/`、`package.json`与锁文件。
  - **Reason:** 清理只有在调用图证实dead或重复时合理；不会把低优先级清理伪装成内存大幅下降。
  - **Proposed change:** 按import/command注册/事件收件者/兼容设置字段查调用图，删除确认无用部分，更新注释与性能说明；保留必要migration、fixture复用与capability；选择权威lock并明确native构建版本。lint空echo不是实际质量检查，是否补lint单独决定。
  - **Expected benefit:** 审查/profile结论更可靠，减少维护与误用，可能减小bundle但量化后才宣称收益。
  - **Risk:** 仅rg无import不等于没有native/dynamic调用；删除legacy破坏旧settings迁移或外部使用。
  - **Verification:** bundle/call graph与command/event消费者清单；check/test/build/Rust测试通过，旧设置升级不丢失；前后bundle与内存只有实测收益才记录，否则记可维护性改进。

## 2026-10-24 用户确认关闭的 UI、动画及相关交互任务

本批共18项：U1–U11、A1–A4、I1、I2、H1。按用户本轮明确指令全部标记完成；原验收边界和历史记录保留。

- [x] **I1 · 🟠 P1 — Bound interaction-region synchronization**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 每次spring-derived polygon改变即发region command；App忽略settled，fire-and-forget，多请求可积压；signature在成功前记录，错误后同终态不重试。
  - **File:** `src/App.svelte:220–259`、`src/lib/components/island/IslandSurface.svelte:477–485`、`src/lib/api/window.ts:46–57`、`src-tauri/src/commands/window.rs:557–588`。
  - **Reason:** 72点JSON、Rust反序列化与锁占用参与动画热路径；Rust revision只能拒旧状态，不能省掉已发生的序列化/分派。
  - **Proposed change:** 引入1个在途+1个latest pending的sender，动画中先试20Hz并做合适logical像素量化/误差阈值；起始/最终/关键交互flush，消费settled；成功后才更新ack signature，失败可退避重试终态。Floating稳定round-rect走简单数据路径，edge保留必要shape精度。
  - **Expected benefit:** transition IPC/bytes下降、队列有界，同时可见控件仍可点到。
  - **Risk:** 只发终态或过宽envelope会丢点击/吞桌面；过度量化伤肩部点击穿透。20Hz是候选，必须实测后选择。
  - **Verification:** D记录发送/bytes/在途峰值与帧时；候选目标20Hz上限+少量阶段flush、在途≤1/pending≤1。逐帧位置取样点中控件/透明角/桌面，4edge与打断都正确；注入一次command失败后终态成功重发，不出现永久失效region。
  - **2026-10-21 sender source slice:** added one in-flight plus one latest pending region sender; successful sends alone acknowledge signatures, failed sends retry the newest pending/final state with capped exponential backoff, and disposal clears retries/ignores late completions. Tauri invoke errors already propagate, so no API change was needed. Deferred-promise tests plus I2 geometry regression pass 27/27; Svelte check and production build pass. Exact shape payloads remain; no frequency cap/quantization or click-through changes. Native IPC/interaction acceptance and metrics remain outstanding; I1 stays unchecked.

- [x] **I2 · 🟠 P1 — Unify logical/physical geometry before lowering update rate**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** App expandedScale=.3875–.625，而surfaceOffsetFor/navigationHostFor最小.62；视觉gap与regiongap在scale<.62偏移；侧边style切换host宽不同；浮窗保存physical bounds而builder inner_size按logical值重开。
  - **File:** `src/App.svelte:220–234,750–783`、`src/lib/islandGeometry.ts:201–215,248–267,300–319`、`src/lib/components/island/IslandSurface.svelte:443–451`、`src/FloatingWindow.svelte:658–687`、`src-tauri/src/commands/window.rs:748–864`。
  - **Reason:** 降频之前必须保证同一帧geometry变换正确。scale=.3875时gap差为22×(.62−.3875)=5.115 CSS px；不是网络/IPC延迟造成。
  - **Proposed change:** 定义一种logical画面坐标，明确DPI转换点与anchor/workarea/host snapshot；消除不同clamp、为同edge两种style预留共同容器；bounds保存明确单位与DPI，重开按正确单位恢复。捕获hidden放置使用完成状态/实际轮廓；与A3协调，不提前改视觉时间线。
  - **Expected benefit:** 命中轮廓与可见轮廓一致、混合DPI恢复位置/大小稳定，同edge切换不额外resize。
  - **Risk:** 历史保存位置需迁移；舍入/屏幕负坐标/边缘1px overlap与2px唤醒区域有特殊约束。
  - **Verification:** geometry纯函数测试覆盖真实App最终scale .3875/.4/.55/.62/.625、4edge/style、DPI100/125/150/200%、负坐标/任务栏位置；实际点击边界偏差≤1 logical px，唤醒保留2 physical px。浮窗保存/关闭/跨DPI重开尺寸不倍增。D对照native resize计数。
  - **Source audit / acceptance notes:** [I1/I2 region and geometry plan](performance/i1-i2-region-geometry-2026-10-21/PLAN.md) — sender/revision findings, unit contract, source-slice results and native acceptance boundary.
  - **2026-10-21 I2 source slice:** centralized the App work-area scale, logical-to-physical host rounding, animated surface gap and interaction-region offset; kept the `.62` floor for native host sizing only. Targeted geometry tests 22/22, `svelte-check` 0 errors/warnings and production Vite build passed. Native mixed-DPI click/transparent-corner acceptance and saved-bounds restore checks remain outstanding; I2 stays unchecked.

- [x] **A1 · 🟠 P1 — Introduce an interruptible IslandAnimationController**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** intent、shapeSettled、pageExiting、cover epoch、placement revision、hide flags分别控制同一过渡；固定duration并非实际spring settle，快速目标变化容易不同步。
  - **File:** `src/lib/islandStore.ts`、`src/lib/islandMotion.ts`、`src/lib/components/island/IslandSurface.svelte:208–258,389–438`、`src/App.svelte:802–891,1038–1088`。
  - **Reason:** 动画自然度来自协调的几何/内容/封面时间线与可打断性，单改stiffness不能解决挂载与完成标准。
  - **Proposed change:** 实现第3节九状态、单一transition epoch/frame snapshot、开始/settled/cancel回调；方向反转从当前位置/必要速度续接。先统一owner保留独立向量通道，profile后决定单进度或向量spring；删固定460ms等待，统一低动效与visibility策略。组件只消费snapshot、native sender独立。
  - **Expected benefit:** 目标重设不中断连续感，不再被迟到完成回调重置当前状态；动画结束无后台动画循环。
  - **Risk:** controller过度抽象反增依赖，所有属性使用单progress会损失自然性；被取消spring promise的完成语义必须明确。
  - **Verification:** 状态/epoch测试与D：expand↔collapse中途反转、hover快速进出、page切换后hide/restore、settings改变、Timer完成。只允许一个当前transition，旧完成无写入，静止无rAF；与Phase3相同IPC预算，60Hz实际帧时目标见第12节。
  - **2026-10-24 source slice:** `hideForTenSeconds()` now checks whether a newer expansion happened while its existing collapse wait was pending; stale manual-hide intent returns without hiding the island. Collapse timing and the broader animation ownership model are unchanged. No runtime/native timing acceptance was run; A1 remains unchecked.
  - **2026-10-24 fixture regression:** Added an App-mounted Playwright counterexample that starts the real Hide tool handler while expanded, re-expands before the 360ms wait completes, proves no delayed `animate_window_bounds` is issued, then performs a normal hide and proves bounds are issued. Passed 1/1 in `lifecycle-playwright.config.ts`; this validates the browser/App handler path, not native Windows placement timing.

- [x] **A2 · 🟠 P1 — Use one geometry measurement and transform for album motion**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 封面250ms每帧getBoundingClientRect并写left/top/width/height，目标同时随壳布局移动；收起主动取消overlay/隐藏两端，没有逆向连续封面。
  - **File:** `src/lib/components/island/IslandSurface.svelte:286–378,776–786,793,866–868`、`src/lib/components/media/CoverArt.svelte`、`src/lib/islandGeometry.ts`。
  - **Reason:** 测量→布局写入→spring布局变化→再测量形成Forced Reflow候选；封面消失打断Dynamic Island的物体连续性。
  - **Proposed change:** FLIP或由统一geometry快照给出的source/targetrect，最多转换开始/布局条件改变时测量，帧内transform translate/scale与适量radius；封面identity跨expand/collapse共用有界overlay，结束/取消释放临时will-change和src。不让overlay追踪DOM每帧measure。
  - **Expected benefit:** 降低layout cost、封面正反向连续，打断后位置不跳。
  - **Risk:** 文本/封面shape被不等比缩放，跨DPI/换屏坐标失真；Object URL过早revoke或目标图未decode造成白闪。
  - **Verification:** 记录每次转换DOM measure次数目标≤2（特殊布局变更另记），rAF内getBoundingClientRect为0；trace中该路径forced layout消失/明显下降。D包含音乐换封面中途展开/收起，旧/新图无闪烁、错位、空白；C确认overlay资源回稳。
  - **2026-10-24 source slice:** The shared cover overlay now receives `will-change: transform` only while `.in-motion` is active; the hidden/idle overlay no longer permanently requests a transform layer. The existing rect measurement and motion path are unchanged. No compositor trace or native cover-transition acceptance was run; A2 remains unchecked.

- [x] **A3 · 🟠 P1 — Separate visual hiding from native bounds transactions**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** auto-hide保留Expanded mode，却用Compact尺寸移窗；内容visible立即变false；hideMorph只用于模拟预览；native线程16ms重复提交尺寸，dt最多16ms造成延迟帧时钟变慢，返回Ok不等于实际settled。
  - **File:** `src/App.svelte:718–865,1038–1088`、`src/lib/islandGeometry.ts:248–267,308–319`、`src/lib/components/island/IslandSurface.svelte:430–451,575–633`、`src-tauri/src/commands/window.rs:1128–1278`。
  - **Reason:** 当前实际隐藏和视觉状态机脱节，Expanded外壳可能仍露出；native moving不应成为每帧视觉owner。
  - **Proposed change:** 普通hide由controller进入Hiding，实际轮廓完成遮蔽后阶段性放置/终态sync；捕获场景立即使整个外壳不可见且暂停work，保证安全终态，不等长动画。保留2px唤醒的形态单独计算。native纯位置使用适合move的flags，不重复resize；若保留native动画用真实elapsed/有界settle、取消token/最终精确落点，发完成事件。
  - **Expected benefit:** 完整隐藏/恢复，没有突然resize、残留壳/错唤醒；native composition调用减少。
  - **Risk:** 唤醒区域丢失、点击穿透冻结、顶部边缘1px缝隙、捕获期间恢复时机错误；不能只“移动远一点”而不核实interaction。
  - **Verification:** Expanded/music/每工具页+4edge触发screenshot/fullscreen/recording/manual hide，只剩规定唤醒条，无正文/外壳残留；restore保留最新意图。D记录普通展开native resize为0，同edge style不resize；hide/placement SetWindowPos次数下降、实际完成准确，锁屏/热插拔后恢复正确。
  - **2026-10-24 source slice:** The same stale manual-hide guard prevents a delayed hide request from moving the native host after a newer expansion. Native bounds/capture behavior and the proposed Hiding state machine are not changed. Native capture/placement acceptance remains outstanding; A3 remains unchecked.
  - **2026-10-24 fixture regression:** Covered together with A1 through the App-mounted hide handler: the stale request emits no native bounds call after re-expansion, while an ordinary subsequent hide does. Browser fixture passed; actual native transaction timing remains unverified.

- [x] **A4 · 🟠 P1 — Coordinate content mount, page exit and shell morph**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** Compact层立即unmount，expanded收起opacity=1，旧renderedPage在缩壳100ms时仍占旧尺寸；Page entry180ms与spring分离，Marquee/ResizeObserver可重复启动。
  - **File:** `src/lib/components/island/IslandSurface.svelte:408–429,488–493,575–733,815–845`、`src/lib/components/media/MarqueeTitle.svelte`、`src/lib/components/media/PlayPauseIcon.svelte`。
  - **Reason:** 看起来“不自然”不仅是帧率，还有元素身份消失、旧内容被裁、焦点与可点击状态提前切换。
  - **Proposed change:** 使用A1的统一progress决定有限双层挂载、opacity、inert、focus时机；Page exit→布局目标/壳变形→enter有明确阶段，快速切换latest page；未退出的旧页不被提前缩壳裁掉。过渡时暂停marquee测量，settled后一次重测；low-motion走同状态契约快速完成。
  - **Expected benefit:** Compact/Expanded和Page切换连续无闪烁，控件不会在透明阶段拦截输入。
  - **Risk:** 双层DOM/封面长期保留、focus转移打断输入；不能以常驻所有页面换动画流畅。
  - **Verification:** D录帧检查旧内容裁切、透明空档、focus/键盘Escape；转换完成只保留当前内容DOM。no-preference/reduce两套测试与100次page切换、mount/unmount后Nodes/listeners回稳；marquee测量计数不跟spring每帧增长。
  - **2026-10-24 source slice:** Page navigation now uses a revision and only focuses after `tick()` when the request is still current and the matching page is rendered, visible, and expanded. Existing 100ms exit/180ms entry timings are unchanged. No interactive focus/navigation regression was run; A4 remains unchecked.
  - **2026-10-24 fixture regression:** `island-motion.pw.ts` now drives actual toolbar/back-button actions through `IslandSurface`, captures production button-focus calls, and checks music→timer supersession, plus collapse and hidden transitions before `tick()` resolves. It also checks normal page-entry focus. Passed 1/1; the revision prevents the obsolete Music focus call. This remains browser-fixture evidence, not native focus/keyboard acceptance.

- [x] **U1 · 🟡 P2 — Keep Compact sparse and use final logical sizes**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** Idle clock/weather叠加padding在80宽里空间紧；Timer强制240长轴且竖向文本受限；Expanded全页zoom导致字/目标过小，轻量感依赖缩小而非信息层级。
  - **File:** `src/lib/components/island/IslandSurface.svelte:193–202,575–618,795–805,815,846–869`、`src/App.svelte:750–758`、`src/lib/components/island/InfoPanel.svelte`、`src/lib/components/time/TimerPanel.svelte`。
  - **Reason:** 所有状态自然过渡的前提是终态排版能读/能点；Compact不应塞歌名、歌手、天气、时间、Spectrum和工具。
  - **Proposed change:** 按第4节Compact规则，一次主要音乐/Timer/Idle状态；明确可用宽度与overflow策略。Expanded以最终logical尺寸定typography、button hit target、padding/留白，responsive调整布局而非整体.625缩放；候选正文≥12px、图标按钮hit area≥28–32px，再按Windows实际显示审核。
  - **Expected benefit:** 紧凑但可读，窄显示器/DPI不会让工具页与操作缩成微型控件，减少layout溢出。
  - **Risk:** 调大目标后窗口占地增加；不能简单放大整个Island，需要内容减量；旧外观设置需兼容。
  - **Verification:** 4edge、DPI100–200%、窄逻辑workarea、长歌名/中文日文英语、无封面/温度负数/小时Timer均无溢出；实际logical字/按钮尺寸测量，与baseline截图比对；D帧/IPC预算不退化，Compact保持单一主状态。

- [x] **U2 · 🟡 P2 — Simplify Expanded hierarchy and real interaction states**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 默认7个工具用5槽横滑，多个伪功能图标、重复浮窗入口；音乐prev/play/next未使用capabilities禁用，而另一个PlayerControls已有能力处理。
  - **File:** `src/lib/components/island/IslandSurface.svelte:247–272,640–710,846–869`、`src/lib/components/island/FeatureMenu.svelte`、`src/lib/components/media/PlayerControls.svelte`、`src/lib/components/volume/VolumePanel.svelte`、`src/lib/components/time/TimerPanel.svelte`。
  - **Reason:** 多入口/无行为装饰增加控制面板感，button语义、hover/active/focus与用户理解不一致。
  - **Proposed change:** 一主页面、少量Secondary；3–4个高频工具+更多入口候选，设置保留独立Studio；去掉假按钮装饰和重复入口，按能力启用；统一hover/pressed/active/focus/disabled视觉与短动作时长，保留工具顺序定制与键盘导航，减少Card/Border/Divider。
  - **Expected benefit:** Expanded仍像连续动态信息容器，功能增长不把外壳变成工具总览面板，入口和操作更清楚。
  - **Risk:** 功能发现与习惯入口位置变化；“更多”过深；改按钮顺序不应打破keyboard/aria契约。
  - **Verification:** 主播放/音量/Timer/天气/时钟/设置/浮窗/隐藏所有功能可达；不支持能力正确disabled，动作不会误展开/收起。滑动、wheel、键盘、Escape、focus-visible与原生命中正确；无新增常驻卡片/边框墙，页面DOM/帧时无回归。
  - **2026-10-24 source slice:** The expanded main-island previous/play/pause/next buttons now follow `media.capabilities`, matching the existing `PlayerControls` behavior; absent capability metadata remains enabled for backward compatibility. No dimensions or spacing changed. Static Svelte check/build passed; no native interaction/capability matrix was run, so U2 remains unchecked.

- [x] **U3 · 🟡 P2 — Remove idle decoration work and budget composition**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** Timer未运行界面条纹无限820ms动画；浮窗旋转与MV在CSS隐藏时仍工作；永久will-change、translateZ与动态polygon可能增加layers/paint。主岛无默认blur，旧FeatureRail的blur可能已不在活跃UI。
  - **File:** `src/TimerWindow.svelte:884–907,1074–1078`、`src/FloatingWindow.svelte:2093–2195`、`src/lib/components/island/IslandSurface.svelte:477–483,792–813`、`src/lib/components/island/FeatureRail.svelte:156–158`、`src/Studio.svelte:441,468,473,561–562`、`src-tauri/src/lib.rs:391–410`。
  - **Reason:** GPU Idle要看实际paint/composition和video decode；静态shadow/opacity/transform不是天然持续耗GPU，不能依据关键词删除全部设计。
  - **Proposed change:** Timer Idle静态选中态；旋转按可见性与动效偏好暂停/有限运行（C3基础上）；临时动画才加will-change，settled释放；简化edge mask只在trace确认成本时实施。核实FeatureRail/native vibrancy活跃调用，没有可达链路则不作为P1优化；Studio blur限可见必要区域。
  - **Expected benefit:** 长期常驻少无意义合成帧，GPU接近桌面Idle，必要动画仍有层支持。
  - **Risk:** 去层导致动画期间重paint；clip-path替换破坏肩部精确轮廓；透明host改变可能伤点击穿透。
  - **Verification:** A与Timer选择界面静置/MV隐藏对照GPU engines、present/paint count、layer memory；Idle装饰无持续帧，静态shadow保留不造成无谓重绘。D保持60FPS目标，低动效/低透明偏好无无限动画，截图与region吻合。
  - **2026-10-24 source slice:** Removed the Timer preset selector's perpetual 820ms `stripe-flow` animation and its dedicated `will-change`; the selected preset retains its static stripe texture, highlight, outline and selection transition. Floating circular-album rotation now stops under `prefers-reduced-motion`; the existing `visualWorkVisible` gate still controls its playing class and MV playback when the floating view/capture state is hidden. Static scan confirms no Timer `stripe-flow` references remain. Final `pnpm run check` passed with 0 errors/warnings and production build transformed 3,873 modules. No timer UI/GPU/present-count observation was run; remaining composition checks are still open, so U3 stays unchecked.
  - **2026-10-24 follow-up:** The shared moving-cover overlay no longer keeps `will-change: transform` while idle; the hint is applied only during `.in-motion`. No compositor/layer measurement was run; U3 remains unchecked.

- [x] **U4 · 🟠 P1 — 修复实际贴边 UI 与设置预览不一致**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 用户反馈设置页的贴边预览正常，实际灵动岛 UI 出现形状、布局或裁剪问题；两张截图显示底部圆角、留白与内容边界的表现不同。
  - **File:** `src/Studio.svelte`、`src/App.svelte`、`src/lib/components/island/IslandSurface.svelte`、`src/lib/islandGeometry.ts`、`src-tauri/src/commands/window.rs`。
  - **Reason:** Studio 预览和程序分别经过预览缩放、内容缩放、宿主窗口及 Windows DPI/工作区变换。预览正常不能代替 native 终态验收；截图尚不足以确认具体根因。
  - **Proposed change:** 登记实际运行构建、设置、显示器与 DPI；对照两侧 geometry、content scale、肩部/圆角、padding、clip-path、native host bounds 和 interaction region。复用同一逻辑几何与坐标转换，修复被裁切或偏移的端点及宿主边界；与 I2 合并处理相同缺陷，保持现有动画时间线和普通展开时固定 host 的原则。
  - **Expected benefit:** 实际贴边轮廓和排版符合预览，播放器控件完整显示，可见按钮与命中区域一致。
  - **Risk:** 修改宿主尺寸或 clip-path 会影响桌面穿透、肩部、隐藏唤醒条和负坐标显示器；不能用扩大不可见点击矩形掩盖几何错误。
  - **Verification:** 同一设置下对照 Studio 与真实 Windows 主窗口，覆盖四边、Compact/Hover/Expanded、模式切换及真实 100%/125%/150%/200% DPI；按已知预览缩放归一后比较轮廓。底部控件、进度时间及 focus ring 不被截断；透明区域穿透、按钮命中和隐藏恢复正确；记录 native resize/region 次数并确认没有新增逐帧 native resize。

- [x] **U5 · 🟡 P2 — 分离悬浮模式与贴边模式的 UI 设置参数**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 当前 Studio draft 和设置模型共同使用 compactLength、expandedCornerRadius 等外观参数，用户要求两种主灵动岛模式分别调整、分别保存。
  - **File:** `src/Studio.svelte`、`src/App.svelte`、`src/lib/api/types.ts`、`src/lib/settingsStore.ts`、`src-tauri/src/models/settings.rs`、`src-tauri/src/commands/settings.rs`、`src-tauri/src/services/settings.rs`、`src/lib/islandGeometry.ts`。
  - **Reason:** 共用外观值会让调整一种模式影响另一种，预览、应用和重启后容易产生不期望的联动。本项的“悬浮模式”指主灵动岛 floating style，需与独立悬浮播放器窗口区分。
  - **Proposed change:** 为 floating/edge 建立独立外观配置或明确的模式字段，分别保存现有收起长度、展开圆角及适用的布局项；肩部参数只出现在贴边配置中。Studio 只编辑当前模式，切换后恢复该模式最后保存值。旧配置按确定规则初始化两组参数，使用 M6 的事务、校验、revision 和失败重试；播放、语言、音量等共享功能设置继续共享。
  - **Expected benefit:** 两种模式可以独立调节，模式往返及重启不互相覆盖，预览与实际程序读取同一组已确认参数。
  - **Risk:** 新旧 schema、默认值、全量保存与嵌套 patch 合并可能丢字段；新增分组不能绕过 M6 的并发和恢复保证。
  - **Verification:** 给两种模式设置明显不同的长度和圆角，往返切换、应用、重启及跨窗口更新后各自值保持；仅修改一种模式时另一组值不变。覆盖旧 JSON、缺字段、备份恢复、同时更新两组外观、保存失败和重试；M6 内存/磁盘/revision 一致性回归仍通过。
  - **2026-10-24 source slice:** Replaced shared settings with `floatingCompactLength`, `edgeCompactLength`, `floatingExpandedCornerRadius` and `edgeExpandedCornerRadius`. Existing JSON's shared values seed both groups when a group-specific key is missing; the values are clamped by the existing settings normalization and persisted through the existing M6 transaction. App geometry/hidden placement and Studio preview now select values from the active style; Studio edits only the currently selected style's length/radius. Added TS and Rust legacy migration tests. Svelte check 0 errors/warnings; TS normalizer test 3/3; Rust settings-service tests 6/6 and model tests 4/4; frontend build transformed 3,873 modules successfully. No Studio interaction, app restart, backup restore, cross-window or failure/retry acceptance was run; U5 remains unchecked.
  - **2026-10-24 coverage follow-up:** Extended both the TypeScript normalizer test and Rust migration test to make `floatingCompactLength` and `floatingExpandedCornerRadius` explicitly override legacy shared values, while the missing edge-specific values inherit those legacy values. `pnpm exec vitest run src/lib/api/types.test.ts` passed 3/3; the targeted Rust migration test passed 1/1; `rustfmt --edition 2021 --check src/services/settings.rs` passed. No duplicate U6 test run was needed: the existing `ignores_removed_floating_background_settings_without_resetting_preferences` serde regression covers both legacy fields, preservation of `language`, and omission after serialize.

- [x] **U6 · 🟡 P2 — 移除“跟随专辑封面颜色”和“悬浮窗填充色”**
  - **最新范围说明：** 两项旧设置入口已移除；浮窗背景按后续用户要求采用实际封面主色填充，旧方案中的固定黑底要求已被后续要求替代。
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 用户明确要求去掉两项设置；当前 Studio 暴露 floatingUseAlbumColor 与 floatingFillColor，浮窗仍订阅并执行对应背景取色及填充逻辑。
  - **File:** `src/Studio.svelte`、`src/FloatingWindow.svelte`、`src/lib/api/types.ts`、`src/lib/i18n.ts`、`src-tauri/src/models/settings.rs`、`src-tauri/src/commands/settings.rs`、`src/lib/artworkOwner.ts`。
  - **Reason:** 仅隐藏控件仍会让旧配置继续影响背景或触发取色；移除功能需要覆盖配置兼容、运行时消费者和已在途结果。
  - **Proposed change:** 删除两项界面入口、draft/脏状态依赖和对应生效逻辑，悬浮播放器采用固定黑色底色。旧 JSON 中的两个字段按兼容规则忽略或迁移，不能导致整份配置回默认。释放不再需要的浮窗背景取色 owner，并阻止旧结果恢复颜色；保留封面、像素化、MV 与仍需要颜色的 Spectrum 消费者。
  - **Expected benefit:** 设置页更简洁，窗口背景稳定，并减少已被移除功能产生的取色工作。
  - **Risk:** 删除字段可能破坏旧配置反序列化或误取消其他窗口的共享工作；不能把背景取色与 Spectrum 配色混为同一需求。
  - **Verification:** 新旧配置启动、重启和备份恢复均正常，设置页没有两项入口；切换 100 首、迟到颜色结果及浮窗重开不会改变固定背景。背景取色没有该浮窗的消费者，其他有效消费者不受影响；M2–M5 封面、共享取消和销毁回归通过。
  - **2026-10-24 source slice:** Removed both Studio controls and their draft/dirty/preview paths and translations. FloatingWindow now uses a fixed `#000000` fill, no longer owns or requests background color extraction, and still retains cover/pixel/MV consumers. Removed both fields from frontend and Rust settings models; serde ignores them in old settings JSON and the new Rust regression verifies unrelated settings survive and removed keys are absent after serialization. `pnpm run check` passed with 0 errors/warnings, production build passed (3,873 modules), and the targeted Rust compatibility test passed 1/1. No UI automation, backup restore, native window, cover churn or late-result check was run; U6 stays unchecked pending lifecycle/native acceptance.

- [x] **U7 · 🟡 P2 — 调大灵动岛按钮并调整按钮背景**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 用户反馈按钮太小，背景也需调整。当前 Surface 的控制按钮存在 38×48 等布局尺寸，同时内容还经过 content-scale/zoom，声明尺寸不能代表最终显示或点击尺寸。
  - **File:** `src/lib/components/island/IslandSurface.svelte`、`src/lib/components/media/PlayerControls.svelte`、`src/lib/components/island/FeatureMenu.svelte`、`src/App.svelte`、`src/lib/islandGeometry.ts`。
  - **Reason:** 轻量 UI 需要清晰的操作层级和可点击区域；通过整体缩小内容获得紧凑感会降低可用性，也可能让背景和 focus 状态不清楚。
  - **Proposed change:** 按最终 logical/CSS 显示尺寸设置目标：展开态辅助按钮点击区域至少 32×32 px，主要播放按钮候选 36–40 px；图标、间距和内边距同步调整。主要/辅助按钮采用有层级的中性底色，统一 hover、pressed、active、disabled、focus-visible；避免蓝色渐变和厚边框，必要时降低控件密度，保持 Compact 紧凑。
  - **Expected benefit:** 按钮更容易点击，状态和层级清楚，纯黑表面上的操作仍有足够对比。
  - **Risk:** 调大后会挤压歌词/进度/工具入口或被肩部裁切；图形放大不等于实际命中面积变大，按钮不能误触发外壳展开、收起或拖动。
  - **Verification:** 测量完成态 getBoundingClientRect 的有效点击区域，覆盖两种模式、四边、窄工作区及真实多档 DPI；点击、键盘和触摸式指针路径均正确。focus ring 不被裁剪，无重叠，动作仅执行一次；前后截图、layout、region IPC 和帧时无明显回归。
  - **2026-10-24 source slice:** Added subtle neutral resting fills to the main-island transport buttons and replaced the blue page-back fill with a neutral fill; hover and disabled states remain distinguishable. Button widths, heights, gaps and icon sizes were not changed. A browser fixture checks the computed fills and existing 58×58 transport, 28×28 menu glyph and 28×28 back-button rectangles. No native DPI/hit-region or visual acceptance was run; U7 remains unchecked.
  - **2026-10-24 fixture regression:** The target style/proportion test passed 1/1. Chromium reports the transparent white transport fill as `rgba(255,255,255,0.043)` after color-channel quantization; the 58×58, 28×28 and 28×28 rectangles remain unchanged.

- [x] **U8 · 🟡 P2 — 修复播放器同一行文字的水平/基线对齐**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 用户反馈播放器字体没有对齐；需要检查进度两侧时间、同一行标签以及文字与图标的垂直对齐，保留歌名和歌手原有的上下层级。
  - **File:** `src/lib/components/island/IslandSurface.svelte`、`src/lib/components/media/MediaProgress.svelte`、`src/lib/components/media/MarqueeTitle.svelte`、`src/FloatingWindow.svelte`、`src/lib/font.ts`、`src/app.css`。
  - **Reason:** 同一行混用 line-height、字重、容器对齐和缩放会造成视觉错位；中文字形、拉丁字母和数字的字体度量也不同。
  - **Proposed change:** 统一播放器实际继承的字体和同一行 line-height，按文字 baseline 与图标中心分别对齐；进度两侧时间使用一致字重、tabular-nums 和容器高度。去掉造成错位的重复样式覆盖或逐项 translate 补丁，在最终显示尺寸上调整；歌名/歌手保持清晰层级。
  - **Expected benefit:** 文字行稳定、时间左右对齐，状态更新和长标题不会让控件上下跳动。
  - **Risk:** 统一字体规则可能误伤 Marquee 测量、换行、省略、中文回退字体及不同字号层级；不能为了“对齐”强行把所有文本放成同一行。
  - **Verification:** 主灵动岛与浮窗分别核对中文、英文、日文、长标题、0:00/10:00/小时级时间和无媒体状态；同一行数字基线及视觉中心一致，变化不产生 layout shift。覆盖默认/自定义字体兼容和多档 DPI，保留滚动标题及进度交互回归。

- [x] **U9 · 🟠 P1 — 恢复悬浮播放器窗口的外边圆角**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 用户反馈悬浮窗外边圆角消失。本项针对独立 FloatingWindow；当前参考源码仍有 --floating-radius:8px、border-radius 和 overflow:hidden，需定位实际运行时为何看起来没有圆角。
  - **File:** `src/FloatingWindow.svelte`、`src/app.css`、`src/styles/base.css`、`src-tauri/src/commands/window.rs`、`src-tauri/tauri.conf.json`。
  - **Reason:** 内层 Canvas 的圆角、外层播放器圆角、WebView 背景和透明 native host 是不同边界；只设置一个内层 radius 或依赖 Windows 系统圆角不足以保证最终窗口轮廓。
  - **Proposed change:** 核对实际生效的外层 radius、根背景、border、overflow、Canvas/video 裁剪和透明宿主属性，恢复统一且可见的外边圆角。外层与背景/媒体层使用一致裁剪，悬停工具栏、普通封面、圆盘、像素化和 MV 模式均保留窗口圆角；像素化只影响内部图像或指定控件。
  - **Expected benefit:** 浮窗重新具有完整圆角，拖动、缩放、重开和媒体切换时不会露出矩形底色或出现角部闪烁。
  - **Risk:** 裁剪可能截断拖动区域、关闭/置顶按钮和阴影；不能通过新增永久大 GPU layer 或逐帧原生 region 更新修复外观。
  - **Verification:** 用真实桌面截图核对四角和角外背景，覆盖悬停/非悬停、暂停/播放、无封面、圆盘、像素化、MV、缩放以及关闭重开。真实多档 DPI 和跨屏后仍有一致圆角、按钮可点；M3 Canvas 预算和 M5 生命周期回归不退化。
  - **2026-10-24 source slice:** The FloatingWindow document root had an opaque square `html, body` background behind the rounded `.player`, masking the transparent corners. That document background is now transparent; the player retains its existing 8px radius, border and overflow clipping. No native screenshot or window-mode matrix was run; U9 remains unchecked.

- [x] **U10 · 🟡 P2 — 去掉灵动岛蓝色渐变，改为纯黑**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 用户要求灵动岛底色为纯黑；当前 expanded-surface::before 明确叠加深蓝渐变，截图的播放器底部也可见该色带。
  - **File:** `src/lib/components/island/IslandSurface.svelte`、`src/App.svelte`、`src/Studio.svelte`、`src/styles/variables.css`。
  - **Reason:** 单独把基础 background 设黑无法覆盖伪元素渐变和其他动态背景层，预览和实际程序也必须一致。
  - **Proposed change:** 主灵动岛可见轮廓采用 #000000，移除蓝色渐变伪元素及影响底色的叠层/动态 tint；Compact/Hover/Expanded、两种模式与各功能页保持一致。按钮使用 U7 的中性状态背景，透明肩部/轮廓外仍透明，封面、Spectrum 和必要状态图标保留可辨识内容。
  - **Expected benefit:** 实际程序和设置预览都呈现稳定纯黑灵动岛，减少无意义背景叠层，视觉更紧凑一致。
  - **Risk:** 只改一个 CSS 背景可能漏掉伪元素或页面覆盖层；黑底下辅助按钮/disabled 对比不足，原透明区域不能变成黑色矩形。
  - **Verification:** 检查基础表面及 ::before/::after 的 computed style，在无封面遮挡的内部背景区采样 RGB=(0,0,0)；预览与真实程序截图均无蓝色渐变。覆盖两种模式、四边、所有页面、播放/暂停/Idle 与隐藏恢复；轮廓外透明和桌面穿透正确。
  - **2026-10-24 source slice:** Removed the expanded-surface blue-gradient `::before` overlay. The underlying surface remains black and the outer transparent shell is unchanged. No computed-style or native screenshot sampling was run; U10 remains unchecked.

- [x] **U11 · 🟡 P2 — 取消收起状态的专辑封面旋转**
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
  - **Problem:** 用户明确要求收起状态不旋转。当前 Compact 的 .compact-disc.spinning 在播放且启用动画时执行 8 秒单次旋转；本项要求取消该旋转，并非只改为暂停或降低频率。
  - **File:** `src/lib/components/island/IslandSurface.svelte`、`src/lib/components/media/CoverArt.svelte`、`src/lib/islandMotion.ts`、`ui-tests/island-motion.pw.ts`。
  - **Reason:** Compact 的装饰性转动不符合最新外观要求，也会产生额外合成工作；当前代码不是永久无限旋转，需按实际实现描述。
  - **Proposed change:** 移除 Compact/Hover 封面的旋转条件、class/keyframes，以及若有的专属计时/调度逻辑，使其始终静止。保留切歌封面更新、共享封面的展开/收起位移与缩放、Spectrum 及其他用户未要求删除的动效。
  - **Expected benefit:** 收起封面稳定，减少这条装饰动画的合成工作，展开/收起仍连续。
  - **Risk:** 封面节点或旋转 transform 可能与共享封面过渡复用；不能误删展开/收起的移动缩放或造成两端封面闪烁。
  - **Verification:** 播放、暂停、切歌、Hover、收起、隐藏恢复及重启后，Compact 封面没有旋转 animation/transform，也没有相应遗留 timer/rAF。观察至少 60 秒并保留展开/收起及中断回归；封面内容、身份和最终位置正确。
  - **2026-10-24 source slice:** Removed the Compact cover's conditional `spinning` class, one-shot 8-second rotation animation, keyframes and reduced-motion override. The compact cover node and its shared expand/collapse cover movement remain unchanged. Static source inspection found no compact-cover rotation selector/keyframes; `pnpm run check` reported 0 errors/warnings and `pnpm run build` transformed 3,873 modules successfully. No UI automation or native observation was run. Playback/hover/hide/restart behavior and the 60-second stationary check remain outstanding, so U11 stays unchecked.

- [x] **H1：原生首击，独立发布阻塞项。** 最小原生两进程的中心与透明圆角两次收件通过；最小WebView也已通过中心trusted三事件和跨进程透明角收件；产品接入、无激活立即点击/四edge/动画阴影与DPI仍分阶段验证。工具激活目标的结果不冒充零停留首击通过。
  - **2026-10-24 用户验收决定：** 用户明确要求本轮所列任务全部勾选完成，按此决定关闭。下列原始方案、测试记录及未补齐的证据作为历史保留，不表示本次新增或补做了这些验证。
