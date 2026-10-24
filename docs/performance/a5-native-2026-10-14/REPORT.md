# 第5批：当前媒体修复Release原生交互矩阵（2026-10-14）

**完整原生矩阵未通过。** 使用第2批标准Tauri保存Release，保留成功、失败和未验项，不修改业务/动画。C1/A5仍 `[ ]`，未重建、开性能轮、提交或推送。用户主观手感仍待查看原始短片；工程状态检查不能替用户签署观感。

## 构建、输入与证据边界

- EXE SHA256 EA80838E71C64A9253364885A60D9AFD7EFF3C8DC5445A4AE7351D6DADA92D2C；同目录DLL33B43C71DB2A218DD0DAFEE6EB4DD34FE59D5CC3FE78B8112B8DA0BD7CD0C292。两次测试均先读文件验证，Node明确spawn该绝对路径，原生HWND/PID/rect保存。不是旧A5配对或unbundled-attempt。
- App、media服务、islandMotion源码hash与第2批保存来源一致，islandMotion仍420/360/240ms，IslandSurface当前hash另记录。未重建；没有用测量等待时间倒推动画参数。
- 测试当时没有保存GetProcessImage/loaded WebView2版本字段，不能补造为“已观测实际加载版本”。补充启动只读元数据的命令被自动审批审查拒绝，原因只返回blocked by policy；未换方式重试启动，运行时加载字段N/A。路径证据目前限spawn参数、匹配二进制hash及其PID/HWND。
- 专用WinForms测试底板覆盖真实副屏DISPLAY2（1920×1080、DPR1），位于isle下方，接收透明/误命中点击。使用Windows SetCursorPos、SendInput鼠标、keybd_event键盘注入：是实际OS/native输入路径，但不是人工硬件动作。CDP仅读DOM、调用设置和模拟本原生页面低动效，未用programmatic click或CDP鼠标假称原生命中。
- 保存原生CopyFromScreen整副屏PNG、12秒gdigrab 1920×540短片floating-motion.mp4、完整DOM/窗口/输入/底板点击时间线、可信page事件、恢复与失败记录。短片15fps仅供视觉审阅，不是实际呈现帧时、性能或60FPS验收。

## 有界场景表

| 场景 | 结果及范围 | 原始证据/下一步 |
|---|---|---|
| floating Compact→Hover未完成时展开 | 状态终态检查通过；trusted native点击，外壳展开 | 首轮floating-hover-interrupt。首段封面尚未就绪，不能借此勾封面连续性。 |
| floating收起中反向→展开→Compact | 状态检查通过，记录中途快照及native点击 | floating-reverse；观感/突然跳位仍需短片审核。 |
| 真实指针快速进出 | 未展开并回到Compact终态 | floating-rapid-pointer及截图；不是逐帧性能。 |
| edge top | 首次测试点点击未展开 | DOM安全候选x=-960,y=2；底板收到、页面无click。立即停止该项，展开/收起/反向未验完。 |
| edge right | 同上 | x=-4,y=517；同类原始失败保留，未换点反复试到通过。 |
| edge bottom | 同上 | x=-960,y=1008；同上。 |
| edge left | 同上 | x=-1898,y=517；同上。 |
| edge→floating切换后首次命中 | 首次点击未展开 | x=-960,y=15，底板收到、页面无click。稍后独立浮动场景能够展开，不能据此覆盖首次失败。 |
| 已有音量按钮实际命中 | 按钮进入音量页通过 | trusted native点击、page-back和系统音量界面出现。 |
| Native激活/Tab焦点 | 部分证据有效，焦点可见性未验收 | SetForeground请求、trusted Tab、activeElement为系统音量DIV滑块。首轮断言只统计button，漏滑块，故原fail保留；不能判为产品没有焦点。截图可审，但未采DIV的:focus-visible/outline，键盘操作完整验收仍未知。 |
| 首轮透明穿透项 | 测试前置失败，未执行到所选透明点 | 确保Compact的点击失败，所以不是“透明穿透坏了”。补一个独立透明点场景，未重试贴边失败。 |
| 独立原生透明点 | 实测通过 | 副屏点(-1143,180)，在native window内且DOM surface外。底板点击数+1，主页面click数不增，WindowFromPoint落底板PID。不是只查WM_NCHITTEST。 |
| 真实已就绪封面Compact/Hover/Expanded | 终态与中间样式证据有效，主观连续性待审阅 | 使用网易云真实“祈り、終われば”/中島美嘉；最多15秒等真实cover加载，未伪造媒体。补充65个DOM几何样本，7个含moving-cover in-motion及完整真实图片；原生终态截图都保留。 |
| 手动隐藏→10秒恢复 | 操作和恢复命中检查通过 | 已有Hide按钮native点击，hidden rect.Top=-40、恢复Top=0，恢复后native点击展开。窗口仍visible、只是移位/内容隐藏，不误称整个native rect移出了屏幕。 |
| 原生页面低动效 | 偏好模拟与布局终态检查通过 | CDP emulateMedia reduce作用于实际WebView2页面，展开终态、封面有效；已清除模拟。不是Windows全局低动效实测。 |
| 真跨DPI | N/A | 两个真实显示器scaleFactor都1，不用CSS缩放冒充跨DPI。 |
| 主观手感/整体性能 | 待用户/另轮 | 不由此短片、DOM轨迹或部分命中检查勾A5；不开展性能/IPC轮。 |

edge与style-switch的坐标是DOM认为可点击的表面候选，实际底板接收证明该输入没有送到主页面。Win32 WindowFromPoint是在输入后的40ms观察，部分已重新指向isle；不能用这个后采查询代替实际收件人。可能涉及hover/布局变化、native区域或测试点边界策略，本批没有栈/逐帧region证据，不指定根因，也不声称整个边都不能点击。待发布方独立复核后另开最小定位任务，不在本批修I1/动画。

## 封面与既有UI待办

首轮初次展开原生截图显示音乐占位，紧接着同曲目真实图片填入；初段DOM记录没有img，稍后为1200×1200完整图片，最终展开原生截图粉色封面可见。这说明首段没有满足“真实封面已就绪”的测试前置条件，不能以首段动画通过宣称封面连续性通过，也不能仅此认定M2新回归/泄漏。补充场景先等真实图ready再记录，未改变业务。

已查看floating-expanded、manual-restored、focus失败等关键原生图片；后段封面/文字/按钮终态可见，expanded底部深色背景等保留为既有U4–U11视觉待办边界，本批不修。65份DOM轨迹是采样几何，不是完整物理呈现序列，不能自动证实“完全无跳位”；短片有首段未就绪限制，手感与更多原生视觉矩阵继续待审。

## 工具失败与退出码

首次工具启动Main(HWND)使用不可封送的泛型delegate，导致setup失败。仅修测试helper为具名Win32 delegate，保留attempt-setup-failure；没有修改产品来凑通过。一次编辑编码失败后命令仍重复执行了同类setup失败，随后修正UTF-8及退出码检查。两次有效工具运行中，首轮5项runner pass/7项fail，补充2项pass；首轮进程退出0仅说明工具写完并恢复，**不代表12项全通过**。补充日志有FileHandle GC关闭警告，属于工具资源处理告警，不作为产品回归；最终工具补显式close，仅语法检查，未以再次运行重试失败场景。补充实际运行版本和最终工具版本分开保存。

纯Node离线核验退出0：实际保存的四edge底板收件/主页面无click、trusted事件、真实加载封面轨迹、独立透明点击、隐藏恢复、低动效及settings/audio/play-state/foreground恢复一致。输出evidenceVerified=true而productMatrixAccepted=false，不会把证据自洽当总体通过。

## 状态恢复与交付

本轮没有隔离配置目录接口，只临时改必要style/edge/monitor/工具/动画开关/自动隐藏字段并还原。首轮和补充两侧完整get_settings与各自原值逐字段相同（包括播放器选择/顺序/屏幕）；音量16%、REDMI电脑音箱2 Pro、静音false逐字段相同；网易云播放态保持暂停，同曲目位置0，无play/seek调用。前台HWND264128恢复，helper请求恢复原鼠标位置（未二次读取验证鼠标坐标）。本批isle/底板已关闭，用户网易云PID7760/20024/22892保留；未启动Apple。

包内含实际运行构建/源码hash、工具首轮版本与最终版本、逐动作原始JSON/场景审计、截图短片、恢复/失败记录及文档diff。测试时未采到loaded runtime/version明确保留N/A。C1/A5保持 `[ ]`；当前构建匹配性能/资源/region成本、有效seek支持会话、长期稳定仍是独立待办。

自动审批审查拒绝了补充启动Release读取运行时元数据的命令，返回原因仅为blocked by policy；未重试该动作，缺字段已明确标记。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
