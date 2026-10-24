# 第11批仅恢复草案：现有用户状态，非第二次测试

本文件保存各版恢复方案的历史审阅范围，**实际结果以[最终报告](REPORT.md)及各版result为准**。初版送审时尚未执行；用户原范围已包含恢复，各新增方法先由发布方审代码。原实验evidence/journal字节不改，测试输入0次，原定10秒内lab持续Enqueued，后续Canvas/截图/选点/收件均未到达。

## 原会话和明确限制

- 输出目录：`E:/CodexData/.codex/worktrees/m3-release-c/Wind0ws_Dynamic_island/dist/performance/a5-static-lab-2026-10-15/run-01`。
- 原Node PID [omitted]、Isle PID [omitted]、helper/底板PID29784；精确Process.StartTime和映像路径冻结在`recovery-guard-draft.json`及`recovery-process-lifetimes-readonly.json`。Windows CIM与Process精度可能不同，执行门使用实际Process.StartTime。
- 主岛HWND788216，底板461556；lab与Studio已经正常关闭。原前台592458/PID [omitted]/TID10872，原物理光标1051,689。
- 原exec session36863调用未指定tty，为普通pipes。**没有可安全定向取消的TTY，不发送Ctrl+C/控制台广播或探测信号。** 原Node已停在无公开resumer的永不完成Promise，helper stdin由它持有；新方案不声称复用此通道，不做调试注入、远程句柄关闭或硬杀。
- Node与原helper只保留为transport pending；不能把用户状态恢复与它们的干净退出混为一谈。即使本方案成功，整体complete仍false。

## 只读差异证据

完整顶层键级语义比较只发现playerOrderIds新增Chrome.UserData.Profile2、无删除；playerWeights只是对象键顺序变化，语义相同。Studio的onMount自动refreshSessions→mergePlayerOrderIds→updatePreference→300ms persist及关闭persist与此相符，**来源仍为推论**：没有该内部IPC的调用者/会话列表原始轨迹，不能认定用户操作。协调器没有发update_settings，monitorIndex原已为1。

当前settings.json为98792C92…，.bak为原主文件5879A5A0…；两文件原始备份仍留本地。Run/Isle原现均String但值不同，旧Wind0wsDynamicIsland原现均缺失。所有当前字节hash及Run值类型/存在性/值hash锁定，原值全文不在审阅文件中。

脚本将当前两配置文件另存`run-01/private-recovery-before`，校验并保留新增ID审计，然后恢复原配置字节。这些私有文件和原`private-backup`均不进入可分发包。

## 具体代码与执行门

源码为仓库`scripts/a11-restore-session.ps1`和`scripts/a11-restore-native.cs`，均为新增，仅恢复。默认输出计划，在类型加载前返回；`-PolicyTests`仅fake语义门，`-CompileOnly`只编译类型、不调用原生方法。当前原生/业务/动画工具不改。

实际执行还须`-ExecuteRestore -GuardFile … -ExpectedGuardSHA256 … -ReviewReference …`，并且guard的approvedForExecution=true。草案guard仍false；发布方审阅后才能另存批准记录并提供实际审核引用，不重写草案制造旧授权。

1. 校验原0-input/released证据、journal和已审EXE的原字节，核对三进程PID/精确创建时间/路径、两窗口owner/title、lab/Studio未重新出现。没有window创建、测试move/down/up、播放器调用、status IPC、重开应用、业务修改或构建代码。
2. 重新读取当前文件与Run值，必须等于只读审核冻结的字节/hash。真实共享语义门重新确认唯一差异仍是预定playerOrderIds单项新增，原顺序/其他键不能改变；任何其他变化就停。先保存当前两文件私有副本并校验。每个后续变更前重新检查实际按钮/capture/move-size空闲，未知或busy就停。
3. 对已确认的本轮source恢复原可见性（ShowWindowAsync，无激活；有界等IsWindowVisible真实回读）。对同一个已验owner的main只发一次WM_CLOSE，5秒内等PID退出；不重发、不加WM_QUIT、不用Kill或另一退出路线。退出不成就pending。
4. 没有任何Isle写入者后，重核文件/Run冻结hash。在同目录写原始备份临时文件→hash→File.Replace两文件，各自即时回读原SHA。只处理之前存在的两文件，不递归删除/移动。两Run值按原类型/值/缺失状态恢复并即时回读，不输出全文；本次旧值缺失的项必须仍缺失，不删除未知值。
5. 再核对原helper精确身份，只向已知底板发一次WM_CLOSE并等真实HWND销毁；不结束helper进程或stdin，不创建新窗口。
6. 核对原前台HWND的原PID/TID仍有效；请求SetPhysicalCursorPos原坐标和SetForegroundWindow原HWND，即时回读实际foreground/cursor/physical idle。请求返回与实际状态分别记录，未匹配就pending。不要求关闭其他用户应用。

记录逐步骤成功、失败、时间、native只读状态、原始文件SHA与恢复回读到新的`recovery-result.json`，原失败文件不动。任何异常立即停止依赖步骤，不补试；保留私有备份及原工具进程。脚本不吞掉失败宣称恢复、也不在旧实验结果上打通过标记。

共享设置差异门5项离线测试和新native类型CompileOnly通过，默认无原生调用；早期规范化测试识别PowerShell包装字符串误作对象的问题，已修正并保留日志。它们不是已执行恢复或真实Win32行为证据。C1/A5仍[ ]。本草案交发布聊天审阅，在收到该新恢复方法审核结果之前不执行。

## 后续版本和已执行边界

上文保留为初版审阅范围，不把它当作全部步骤已经执行。初版经独立审阅后，在guard的ISO时间被PowerShell自动转DateTime的身份比较门以0步骤误拒。原recovery-result.json保留；v2明确-DateKind String并共享完整100ns字符串比较，真实JSON解析/一tick差异/拒绝DateTime共3项加原5项通过独立复核。

v2在13:29:45–46Z已确认唯一差异/全部frozen值/idle，保存当前配置私有副本、恢复原source可见性，并只发一次WM_CLOSE使Isle正常退出。首个File.Replace因PowerShell将$null绑定为空backup string而报path empty，两个目标文件只读hash仍是frozen值，Run未恢复；原结果SHA9A526533…和前三步全部保留。

v3只继续余下文件/Run→底板正常关闭→前台/光标，不再show/close原main。前置绑定v2完整hash及恰好三步证明、app PID/主窗已不存在/无writer；复核同一helper/Node和文件/Run冻结值。File.Replace使用[NullString]::Value，已在非用户文件实际重现旧失败并验证新调用替换成功，非仅模型测试。新结果固定recovery-result-v3.json；旧记录不覆盖。只清理经固定路径/hash/2659字节/创建时间匹配本轮v2的唯一失败temp，不递归。原Node/helper仍transport pending，不硬杀或伪造resume。此v3已送发布方审阅，本文更新时尚未执行。

## 最终执行补记（10月16日）

v3后来在旧数字PID门0步骤停止；当时身份没有记录，不能确认PID复用或原退出对象。v4改用原实例代次，独立13项策略/CompileOnly审阅通过，但续办时原工具已不在，未执行。它们的退出原因/最终native清理无记录。

10月16日仅持久恢复源码a11-restore-persistence.ps1经全文独立审阅，验证冻结文件/Run及原始和当前私有副本、NoWriter门后，一次恢复两配置原bytes和Run并即时/独立回读一致，清理固定失败temp。没有窗口、输入或旧前台/光标操作；历史完整UI恢复仍未证实。各版失败、未执行版本及审阅记录均在包内，不能把本历史方案当作可重复执行授权。
