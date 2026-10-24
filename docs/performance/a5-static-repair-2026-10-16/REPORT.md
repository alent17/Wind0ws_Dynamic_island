# 第12批：就绪门与恢复工具离线返修候选

状态：**候选待独立审阅；静态实验、产品首击、C1/A5均未验收通过。** 本批仅代码、离线验证及Release构建；没有启动Isle、浏览器或底板，没有输入、截图、新采集、播放器操作、提交或推送。第11批现场允许已消耗，没有第二次现场。

## 依据与初始化修正

第11批161次Enqueued、nativeApplied/nativeReady=false，测试move/down/up均0；这不能归因为OS输入、SetWindowRgn或已确认的WebView缺陷。锁定wry 0.54.4的实际`attach_handlers`调用约466先于首次Navigate约531；642–665是函数定义。此前“注册晚于导航”候选撤回，原根因仍未确定。

只修改feature-only `window_region_adapter.rs`。窗口先隐藏构造、注册回调和设置位置/尺寸，再绑定实际HWND并显式导航一次至本PID/generation/revision独有的完整静态data文档。该文档无产品代码、外部资源或额外能力；其自身load回调设置唯一标题回执。接收时同时核对ticket、HWND、精确文档URL、允许的首次导航和回执；普通PageLoad Started/Finished只计数，不凭其当前URI推定文档完成。bootstrap空文档、旧身份和重复回执不会重新排队；重载/未知导航失效。

状态公开loaded/configured/queued、导航、HWND、URL/回执、页面事件计数、当前阶段、首次失败及失败所在阶段。loaded与configured都成立才能claim一次，仍须原生事务成功才nativeReady。10秒deadline保留原预算并使无回执的Enqueued显式失效，不伪设loaded，不扩大等待。协调器按status中的精确documentUrl定位页面，不再找任意about:blank。

依赖依据：锁定Tauri 2.10.3的`WebviewWindowBuilder::on_document_title_changed`、`navigate`，wry NavigationStarting/DocumentTitleChanged与attach_handlers实际调用；源码副本和hash随包。官方[ICoreWebView2 Navigate](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2#navigate)描述完整顶层导航，标题事件读取当前文档标题；不能把它当作Canvas已呈现。data文档、CSP、实际回调顺序及UI派发行为本批只编译/协议测试，**未经真实WebView运行验证**；导航拒绝/丢回执会显式失败/超时，不能称现场已修复。

## 真实恢复接线

- 实际PowerShell helper点源`a5-static-persistence.ps1`，通过`Language.NullString.Value`传入真正的null备份名。真实独立文件验证Replace成功消费临时源、保留备份、目标hash、目标原不存在Move分支，以及真实sharing violation保留目标和失败临时源；测试文件均为人工字节，不含用户配置。
- 实际JSON读取使用`ConvertFrom-Json -DateKind String`，7位小数/100ns保留为字符串；进程start ticks在原生快照中也是字符串。实例门比较PID与start ticks，拒绝数字PID复用；真实C# SameIdentity被AppAlive/SameApp和仅编译策略检查共同使用，PS共享比较也接入本轮helper操作门。原owner/gate原字节及失败释放锁存保留。
- 启动前两配置字节和Run备份仍不可变。新Studio创建前安装实际IPC fetch观察器，记录请求参数、成功/失败响应、开始/结束顺序及文档身份；使用锁定Tauri Windows custom-protocol路径，不尝试覆写不可写的invoke。原状态、基于真实session结果的预期playerOrderIds合并/工具monitor补丁、当前get_settings分别保存。未知键/顺序、旧接口写入、缺初始get_settings、重载、重叠写入、IPC fallback或当前快照不符即停止。完整设置审计仅进本机private-backup，不进分发包。
- Studio关闭期间的审计通过binding逐条保存，只有正常persist/原生窗口消失后才恢复语义设置。成功的回滚ack单独保留，部分失败后不会再把自己的回滚当成未知用户修改。磁盘按退出后检查点逐文件校验，已完成文件保持原hash才跳过；不吸收其他文件的新变化。Run若出现未经审计的变化则停止，不能假定为本工具的正常写入。两原文件/Run真正恢复后仍逐项即时回读；本批没有接触真实用户配置或Run。
- `createCleanupResumer`用于实际初始恢复和pending续清理，已完成六阶段及各次已发关闭/回滚ack保留。公开入口为`node scripts/run-a5-static-lab.mjs --cleanup-only <同会话cleanup-control.json>`，只连当前协调器受token/session约束的本机命名管道，不载入live/Playwright，不重开实验、不改点、不重置4次预算、无任意命令。释放必须已确认且held=false，续做前重新检查物理idle、绑定进程实例与设置；不会再次SendInput。
- 正常收尾等待实际helper ChildProcess close/stdio关闭；CLI父监督器在实际协调器close后写coordinator-exit.json。离线子进程测试覆盖这些共享实际路径，不能代替将来GUI会话退出证据。EOF/断管没有显式stop就不恢复历史前台/光标；已退出的传输拒绝冷启动原生恢复。原前台还必须PID/TID/start ticks和30分钟内身份一致，否则保留pending并明确只持久恢复需另审。历史第11批三个恢复脚本、失败包及私有备份不改。

## 验证与构建

最终命令、完整日志、退出码、源/锁文件/二进制SHA-256见包内audit与verification.json。Node工具回归34/34；owner9/9、日志/锁存5/5；实际文件/100ns/实例测试通过；C# CompileOnly通过且nativeMethodsInvoked=false/windowCreated=false。受影响feature窗口37项通过；完整feature 96通过/1项要求Windows音频服务的压力测试保持ignore。

完整默认Rust首次82通过/1失败/1 ignore：原`http_budget::rejects_chunked_overflow_and_removes_partial_file`未匹配预期3004错误。单项复跑通过，整套串行83通过/1 ignore；保留首次失败与复跑日志，原因未定位，不修改本轮外的HTTP业务代码，也不把复跑通过写成从未失败。

标准Tauri Release流程均执行beforeBuildCommand npm run build，默认和experimental-native-region分别保存全新目录。本机cargo符号链接失效、tauri npm命令shim缺失的首次入口失败保留；使用已安装stable真实bin目录及`node node_modules/@tauri-apps/cli/tauri.js build --no-bundle [--features experimental-native-region]`继续，无依赖升级/锁变更。默认版未安装首击修复，产品旧monitor仍在；实验版仅显式feature lab。二进制均未启动。

| 构建 | 文件 | 字节 | SHA-256 |
|---|---|---:|---|
| default-release | isle.exe | 23367168 | `CE207C625AFA7A79D9E96554394595389894C0FD050CB28BFF3AFD86A6CDB469` |
| default-release | isle_lib.dll | 137728 | `4B3F8268303176443B13CC5AC1C65320737781D1BD90D88C1DB25BAE2351809A` |
| experimental-final-release | isle.exe | 23420416 | `18FD7F1311A7D83E63342A9EBD27B278ACC2C4CED1926F951DCC5F5D25736670` |
| experimental-final-release | isle_lib.dll | 137728 | `385B89D51483B165AE0D393B9588D4524BE26263603591F2AA9967C075513827` |

首个实验构建保存在experimental-release，因随后补充迟到bootstrap导航门而被替代，不作为源匹配交付；其原二进制与日志仍保留。最终选择experimental-final-release，之后没有再修改Rust。默认构建后只改feature排除模块，默认编译代码与237项原源基线中其余236项一致。工具选择scope升为v2-candidate并绑定新EXE/DLL，旧v1现场记录不能重用；PENDING_APPROVAL保持false。

首次审阅包解压后，Node回归33通过/1失败，原因是旧测试引用的第10批未批准样例未分发，而非测试进入live。原包、manifest及失败日志保留。修订包`a5-static-repair-review-v2.zip`补入原字节、approved=false的公开夹具；不改旧样例或测试。最终选择v2，交付前再次在其解压source目录执行全部34项。

分发`audit/analyze-run.mjs`现在读同包`raw/`，从解包目录运行可复算原始161次Enqueued/0输入。原始evidence/journal及旧脚本/ZIP哈希校验，旧run-01路径版本不覆盖。审阅包只显式拷贝公开材料，无private-backup/private-recovery-before或控制token。

## 限制与停止点

没有本轮GUI就绪/Canvas/OS路由/真实Studio持久化恢复/实际GUI进程退出验收，也没有性能数据。IPC观察器依赖锁定传输与新文档覆盖，任何回退/缺记录会拒绝；不能保证未来环境一定能自动清理。启动项若有未审变化也会pending，不提供宽泛忽略。持续Enqueued原根因、旧首击投递、95/55ms旧口径、长期GPU/内存、第6首超时均未由本批解决。

C1/A5保持[ ]；所有40条原勾选行逐字保持，M1–M6不改。Tween420/360/240、媒体修复、C2/I1/UI/Spring均不推进。包为离线返修候选，发回“任务发布”后停止，等待独立审阅；没有未来现场允许。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
