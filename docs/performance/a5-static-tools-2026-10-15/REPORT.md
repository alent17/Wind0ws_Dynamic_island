# 第10批：静态lab四次协调工具，纯离线交付

**完成工具准备，现场未执行、尚未允许。C1/A5保持 `[ ]`。** 本轮只新增工具/测试/文档；未改业务或动画、未重建Isle、未启动应用/浏览器/底板、未原生输入或采集。只执行纯fake测试、默认路径与拒绝路径、PowerShell解析及C#类型编译；CompileOnly返回于StartPlate/Snapshot/Press/Move之前。

具体待批准范围见[LIVE_PLAN.md](LIVE_PLAN.md)，审批样例[PENDING_APPROVAL.json](PENDING_APPROVAL.json)保持false。代码中的审批记录只是审计辅助，不能代替人类实际允许；完成后交发布聊天独立审工具，再由其请求允许，不在本聊天自行进入live。

## 实现与共享策略

| 文件 | 本轮交付 |
|---|---|
| run-a5-static-lab.mjs | 默认仅打印dry-run JSON。--live缺记录/未批准/范围或二进制不符时，在导入live/Playwright/进程工具前拒绝。 |
| a5-static-live.mjs | 未来正常单次Isle/CDP的实际I/O接线：新Studio前置、source隐藏、尺寸请求与actual frame、截图/Canvas、带副作用status、收件与恢复。现在仅语法检查，没有执行该模块。 |
| a5-static-coordinator.mjs | 实际live和fake共用四次流程、一次选点/冻结、限次、完整收件/冲突判定、API QPC计算、停止及不安全释放保留。 |
| a5-static-restore.mjs | 实际live/fake共用恢复次序，防止Studio关闭晚保存覆盖恢复值；失败停止依赖步骤，报告pending。 |
| a5-static-input.ps1 / a5-static-native.cs | 默认无GUI；未来独立STA消息泵no-activate底板、native身份/物理frame/DPI、分离move/down/up、tag/QPC/原始事件、持久配置备份/回读与安全停止。 |
| a5-owned-press.ps1 | 原已验收策略原字节复用；成功down取得所有权，正常up清除，EOF/error/finally仅释放仍持有按下。 |
| a5-static-journal.ps1 | 实际辅助进程与fake共享的非抛出输出、保留及释放失败锁存策略，防止stdout/磁盘失败绕过保留，或catch/finally重复发送已失败的up。 |

前置使用现有show_studio_window与Studio控件窗口，不假设已存在前台。拒绝任何原有Isle进程；新建的所有Isle窗口均绑定spawn PID/进程开始时间，Point HWND的PID/TID单独记录，不能把WebView子窗口PID当spawn PID。Studio只有一次正常show/set_focus前置，拒绝就停；不会Alt绕锁或追加点击。恢复阶段另有一次明确的应用前台请求，不能用来重做失败试次。

source monitorIndex仅必要时暂改1；读取源码确认MonitorInfo camelCase工作区/DPI、AppData目录枚举14、窗口show/hide IPC形状、Studio关闭persist，以及启动对两个Run值的同步。因此在正常应用启动前保存配置/自启值，在Studio销毁后才恢复设置，在所有Isle写入者退出后才恢复原始文件与Run值。替换文件用同目录临时文件/hash后原子替换或move，原不存在文件仅删除明确两文件，不递归清理用户目录。未知类型、其他Isle写入者或恢复失败不假报成功。

## 收件与观察限制

本工具不再沿用旧MouseDown对click的非对称判定：原生WM_LBUTTONDOWN/UP及派生MouseClick与浏览器mouse pointerdown/up/click规范为共同down/up/click，raw源类型分别保留，不能认为是同一层的OS信号。浏览器另记compat mousedown/up及dblclick；native可选WM_POINTER、MA和双击为诊断。原生isTrusted无此语义，记unknown；browser需isTrusted且pointerType=mouse。Native down/up tag和各family坐标需匹配，重复/缺失/拆分/错侧或双侧均保留，后查询不能覆盖。

两侧实际HWND/PID/TID与查询时刻的point root分别记录。原生侧是回调当时的HWND/PID/TID；浏览器收件附上预检查询得到的lab宿主HWND/PID/TID，并明确不是浏览器事件未暴露的OS投递HWND或WebView子窗口PID/TID。原生事件时间是WndProc/.NET处理的QPC/UTC，GetMessageTime为独立32位message时钟；browser保留performance/event/UTC。没有把这些不同钟强行当同一投递时刻。动作额外记录PInvoke边界QPC及协议边界，保留原始JSON行避免Int64被JS重建后丢精度。零预算只有“不故意等待”的含义，不等于旧合并立即click，也不是性能样本。

no-activate有三道实际代码措施：[WS_EX_NOACTIVATE](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)、[ShowWithoutActivation](https://learn.microsoft.com/en-us/dotnet/api/system.windows.forms.form.showwithoutactivation?view=windowsdesktop-10.0)、[WM_MOUSEACTIVATE返回MA_NOACTIVATE](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-mouseactivate)，并禁用managed selectable。MA_NOACTIVATE不丢消息，与ANDEAT不同。是否在真实WebView/WinForms/系统环境达到预期，尚未验证；前后台样本及每次board收件的foreground字段需支持后续判断，不能仅靠styles签署通过。

先核对真实frame/DPI/位图，再用桌面合成截图的两个5×5块确认已画出；两次rAF、eval接受或configured=true都不能替代截图。截图不是光子/PresentMon证据；HDR/遮挡/缩放导致严格颜色不匹配时停，不关闭系统特性或放宽阈值。当前没有原生截图，实际坐标N/A，只有固定选点规则。

status仅在每对输入前/后且原生只读门确认Studio前台与无按下后查询；move目标→up之间没有status。其返回Invalid单列“在status处观察到，可能由guard查询造成”，保留prior phase/error/guard，不据此认定OS路由。发送前辅助进程再次检查实际frame/DPI/style/source隐藏、光标与目标root；仍不承诺跨线程系统状态与发送原子一致。任何外部交互或不支持变化都只能失败停止，不能迁用旧8次hover。

## 已运行检查

- JS共享协调/恢复策略：28/28，通过。包含冻结顺序/0预算无等待、frame/Canvas/DPI/前台失败在down前停、双侧/缺侧/错侧/坏tag/激活/双击/坐标不符、query失效单列、释放失败/未知阻断恢复，以及Studio晚保存/退出前写入/即时光标回读失败。
- 共用owned-press：9/9，pure fake，未加载native类型。
- 共用journal/保留/释放锁存故障策略：5/5，pure fake；磁盘与stdout同时失败不会跳过release/retention，未知physical idle不恢复。正常up返回0或抛错后，catch及EOF/finally均不能再次调用原生发送。
- 默认Node/PowerShell及未批准--live拒绝分支通过；C#CompileOnly与Node语法检查通过，未创建HWND或调用工具定义的PInvoke。没有Rust/Svelte/应用构建或GUI回归。

权威最终日志为tools-tests-delivery-final.log、journal-release-gate-tests-delivery.json、owned-press-tests-delivery.json及helper-compile-delivery-final.json，汇总见verification.json。早期工具检查错误保留：C#缺Concurrent、Forms.Primitives与私有GDI+引用，补齐当前PowerShell ref/forwarded程序集后通过；两个native-only故障fake最初误预期首条lab就触发，改为实际第二条board才有该信息，未放宽真实判定。最后源码复核另发现正常up失败后catch可能再次释放，已在原共用owner之外增加失败锁存并以两项实际共用策略fake验证；原owner字节未改。它们都不是现场失败或通过。

## 未证明、恢复与交付

辅助进程的完整未来native路径仅编译，未端到端运行；实际窗口消息顺序、前台权限、截图、CDP附着、原始收件与持久恢复都未知。Fake提供共享决策的故障证据，不证明Win32行为、no-activate、屏幕呈现或系统资源生命周期。真实配置/注册表路径与并发文件编辑不支持情形必须按前置/恢复错误停止，私有备份保留，不继续找替代路径。

正常结束只在完整恢复/即时读回确认后记complete。已知release失败不追加自动尝试；unsafe或未知物理状态保留进程/transport，不移动前台/光标、不销毁仍可能持有释放的目标，不以硬杀兜底。EOF即使stdout已断也先尽力记录到本地journal并执行共用finally；释放失败仍保留。若未能完成正常关闭、配置恢复或即时回读，只报告cleanupPending及未确认步骤，不能宣称环境已恢复；没有把保留的辅助进程计为“已干净关闭”。

第9批237项源/配置/资源及4个选定二进制字节一致；本轮新文件和计划增量另列，保留旧ZIP/原始数据。包包含工具、共用策略、diff、测试/编译日志、dry-run/待批准单、hash/manifest。回退仅移除本批新增工具/文档及计划本批增量，不还原其他业务修改。

C1/A5仍 `[ ]`。现将可审阅的工具与单次4-click范围交发布聊天，等待独立审工具与真实人类允许；本批结束，不自行启动live。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
