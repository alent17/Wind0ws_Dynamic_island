# 第3批：资源探针就绪与匹配采集（2026-10-14）

本批测量链路及一轮原保存A5配对资源诊断通过；C1、A5保持 `[ ]`。未改业务、动画参数、C2/I1/UI/Spring，未提交或推送。本轮原构建不含第2批媒体修复，不能作为第2批新Release的性能验收；不与第1批diag-r3 IPC代理数据混合。

## 修正及门禁

`measure-native.ps1`保留旧调用参数，新增可选RunId/ReadyFile。识别显式RootPid进程树，保留初始CPU基准点；完成所有当轮存活进程有效CPU增量后，原子发布包含run身份、PID、UTC、样本索引/进程身份的ready。协调器最多等待90秒，校验本轮身份/启动时间；陈旧文件立即拒绝。资源采集先就绪，随后启动PresentMon/窗口采样与转换；原3秒等待只用于呈现采集启动，不再作为资源就绪证明。样本逐条JSONL落盘，失败时保留已经读取的证据。

CPU计数逐PID/start-time记录读取前后UTC、前次读取前后UTC及累积秒数。区间用保守的“前次读取开始→本次读取结束”，不是样本JSON写入时间。完整在序列内、跨边界、窗外分别汇总；新PID无前值时不可用。序列显式start/end标记来自页面performance.now及UTC，包含16次转换、确认、控制及间隙，不等于first→last活动span。跨界增量不拆分、不整段计为纯动画。内存/GPU计数为周期观测，读数采集范围也保留，不能保证峰值。

资源门禁自动要求至少3份整个进程树都有效且完全落在序列窗的CPU增量，并要求序列前、末尾后各有有效增量；基准点、陈旧身份、缺失/未来ready、0样本和截断尾段均拒绝。解析/门禁失败不能发布有效配对结论。保留原16/16媒体AND音量同页正长度覆盖、first/last/confirmation分离、QPC→UTC、Present起点加MsUntilDisplayed、跨尾边界行、唯一16次主岛swapchain和呈现尾段门禁。

## 失败与唯一接受配对

- baseline resources-r1：用户网易云也在运行，实际选中netease，预期apple门禁在转换前拒绝。保留原始记录，恢复实际16%音量，未操作网易云播放。
- baseline resources-r2：16次完成且序列内3样本，但ready UTC被PowerShell重新序列化为少几个尾随零，字符串比较误拒绝。已按相同时间值/身份/样本索引比较；原失败记录不覆盖，不补作成绩。
- baseline resources-r3：序列5.773秒，完整在窗内仅2份增量，真实覆盖门禁拒绝。未与candidate凑配对。
- 接受resources-r4：两侧统一15个250ms转换间隙；这是固定诊断调度，动画420/360/240ms未改。相同脚本哈希，无IPC代理，PresentMon各45秒。使用新suffix，不覆盖旧二进制/旧样本。

为保持真实播放器条件，协调器临时选择已有Apple真实会话，并恢复原selectedPlayerIds/playerOrderIds/monitorIndex。原网易云用户进程始终保留。相同曲目“飛ぶ時”/Vaundy，duration252秒、seek=false，不尝试seek。baseline初始2秒，candidate初始5秒，最终各5秒/10秒；进度无法精确匹配/恢复，列为限制，不伪造相同进度。

## 资源结果（单轮、有探针开销）

| 构建 | 整序列秒 | 完整窗内增量样本 | 窗内进程CPU秒 | 跨界进程CPU秒 | 全采样树Private Bytes MiB | Renderer MiB | GPU committed MiB | 线程 | 句柄 |
|---|---:|---:|---:|---:|---|---|---|---|---|
| baseline | 9.721 | 6 | 3.875 | 0.90625 | 253.82–368.79 | 81.60–171.15 | 18.65–40.98 | 253–272 | 3697–4011 |
| candidate | 12.143 | 8 | 5.484375 | 1.03125 | 246.99–367.08 | 81.31–165.89 | 18.00–39.89 | 238–270 | 3471–4016 |

窗内CPU秒仅为读取区间完全在序列内的各进程增量之和，不是整序列总CPU；两侧采样覆盖数、覆盖秒和动画持续时间不同，不按窗口总秒数或活动秒数冒充平均CPU，也不将此差异直接解释为优化收益/回归。逐样本、逐PID实际区间与分类见resource-gate JSON及独立复算。其余CPU：baseline窗前0.28125秒/窗后0.609375秒；candidate窗前0.90625秒/窗后0.734375秒。表中内存、线程、句柄/GPU范围含序列前后，不能当作序列内峰值或长期回稳；GPU是PID committed读数，单engine utilization原始字段可读但不求和当全GPU利用率。

两侧同日、相同WebView2 155.0.4283.45、相同设置/真实歌曲、DISPLAY2副屏1920×1080、DPR1、暂停/33%测量初态、250ms调度和匹配探针。`matching-conditions.json`独立核验这些条件及测量源哈希；系统信息仅在采集结束后读取，未当作同步刷新率映射证明。采集开销存在，进程退出可能漏尾CPU；探针只统计isle后代，不包含外部播放器成本。

## 呈现与覆盖

两侧16/16成功media AND volume覆盖，每转换一个媒体toggle及一个音量写入，串行准入、控制峰值在途2。唯一swapchain分别0x178B71BB280/0x216C931A3A0，每条覆盖16个活动span，主窗口在DISPLAY2；PM缺少HWND，只作唯一时间相关，不强连VidPnSourceId=0至显示设备。完整原始CSV保留；缺MsUntilDisplayed行baseline18/candidate9，未伪造显示时间。

完全在活动span内的>50ms间隔均0；确认尾段3/1，跨界13/13，窗口间9/6。250ms诊断间隙会产生跨界/静止长间隔，不能统称动画停顿，也不能用活动内0条断言无卡顿。原95/55ms口径、真实等待栈、完整交互、region请求成本、支持seek会话及长期内存/GPU继续待验收。

## 验证、恢复与交付

- 门禁9项+原命令覆盖5项，共14/14，退出0。JS语法/PowerShell解析、git diff --check通过。
- resources-r4两侧协调器、matrix、资源probe、PresentMon退出0，资源门禁均通过；分析器0；独立原始复算0（两侧资源+两侧呈现/命令）。独立脚本不import分析器/门禁，复算区间分类/CPU秒、QPC呈现与覆盖，并核验匹配条件。
- 本轮实际用户音量16%，REDMI电脑音箱2 Pro，静音false。两侧均恢复16%和原播放器选择/顺序/monitorIndex，真实Apple暂停。关闭本任务启动Apple PID [omitted]及测试isle，保留用户cloudmusic PID [omitted]/20024/22892。进度恢复N/A，无seek尝试。
- diff、before源码、当前测量源码、双方所有原始资源JSON/JSONL/ready/时间线/PM/bounds/status、失败输入、源/二进制哈希、独立复算、系统/恢复证据均在离线审阅包。旧EXE/DLL实际hash仍与第1批一致，包内保存这对构建；第2批新Release不参与本次采集。

离线命令（包根目录）：`node --test scripts/a5-resource-gate.test.mjs scripts/a5-command-coverage.test.mjs`；`node scripts/analyze-a5-animation-capture.mjs combined resources-r4 . --metadata-only-binaries`；`node verify-replay.mjs .`。manifest可独立逐文件校验。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
