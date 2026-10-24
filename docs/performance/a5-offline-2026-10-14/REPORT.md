# 第6批：首击失败离线定位与测试工具准备（2026-10-14）

**离线交付完成，首击根因未确认。** 不启动GUI/isle/浏览器/播放器/底板，不注入输入、不采集、构建或改业务；未重试/换方式执行被拒绝的补启动元数据。C1/A5保持 `[ ]`，不提交推送。以下方案和拟议工具均未现场执行。

## 现有时间线

完整逐阶段源记录/UTC/坐标/rect/page/底板引用在offline-timeline.json；inputs保留首轮矩阵、实际首轮helper和runner及只读产品路径源码，input-hashes.json冻结hash。设置记录的utc是响应后的记录时刻，不是派发起点；native记录utc是40ms消息泵及查询之后；底板MouseDown UTC是DoEvents时handler执行，并非OS原始down时间。浏览器UTC是事件回调时刻。精确纳秒算术只保留字段精度，不把不同采样来源当成完美同步或物理注入时刻。

| 原场景 | 实际底板回调 | 主页面click | 后查UTC减底板回调UTC |
|---|---:|---:|---:|
| edge top | 1 | 0 | 45.8281ms |
| edge right | 1 | 0 | 50.4479ms |
| edge bottom | 1 | 0 | 54.6019ms |
| edge left | 1 | 0 | 54.3352ms |
| style-switch首次floating | 1 | 0 | 53.4941ms |

五条后查pointPID都30228，实际底板PID30404；这个后查观察不能覆盖已发生的底板收件。记录能重建最后设置响应、最后离开/最近native状态、最近DOM快照、合并click请求、底板/page回调及后查，不能重建独立move/down/up执行戳、按下前实际穿透状态/region或准确DOM选点戳。

floating-hover-interrupt先有显式move和预定60ms等待/DOM快照，再click，确有trusted page click；它此前move点与click点不同（y23→y15），不是“同一坐标精确停留60ms”。floating reverse三个click都有page收件，后查减最近同坐标page回调约52.7437/42.3849/40.4402ms；已有移入/前次click/状态操作不能当成对应每次press的独立时间戳。五失败与floating成功不构成严格配对停留对照。

## 坐标和产品路径

旧runner以Win32 native rect.Left/Top加DOM viewport CSS点并round至整数；本轮两屏DPR/scaleFactor1。保存compact surface为floating x128.5/y12.0254/115×28，top x128.5/y0，right x344/y87/28×115，bottom x128.5/y261，left x0/y87。四edge native rect还含-1/+1贴边偏移。原top点距保存surface顶部仅3px，侧边选点也靠轮廓；hover、anchorGap、native placement及CSS active transform可能在移入后改变。保存快照及PNG只是其采样时刻，不能当按下时的几何/alpha或已接受polygon。

只读代码引用（包内完整源码）：

- commands/window.rs:61–63、spawn_cursor_monitor约410–481：读取cursor/rect/scale→换算local→按后端存储region判断inside→仅在ignored变化时set_ignore_cursor_events→sleep。inside33ms/near50ms/far200ms是检查后的休眠，不是保证在33ms内响应新移入。失败时恢复ignore=false；没有在每次region写入中同步更新ignore。
- App.svelte:251–286：hostFor/surfaceOffsetFor、anchorGap与extraRects平移，签名去重；先更新last signature、递增revision再异步invoke，失败仅warning。存在已登记的签名/重发待办，但旧日志没有失败命令证据，不能认定本次就是该缺陷。
- IslandSurface约324–331通过onRegionChange提交形变几何；width/height/anchor等随动画变化。window.rs:556–584先normalized、fetch_max revision，再锁region，旧revision被拒绝，新值仅写存储。没有对前端暴露last accepted polygon/revision getter。

因此“异步ignore迟到”“坐标/hover形变改变”“前后端区域不匹配/消息未接受”都是候选，不是已定位根因。WindowFromPoint即使在输入前读到App，也不是实际down收件；extended-style bit只是只读观察，不代表已接受region。

## 最小预定对照（未执行）

proposed-plan.json预先固定8次输入，不自适应换点或循环到通过：edge-top内侧(-960,15)与floating内侧(-960,26)，各0/50/250ms停留一次；原失败top边界(-960,2)另保留0/250ms两次参考。坐标来自保存快照，是拟议值；后续获授权现场仅验证该预定点在可见Compact/Hover交集，若不满足则记前置无效并停止，不能偷偷重选。

每次固定外部reset+明确300ms预算、保存原状态→冻结DOM候选/可见边界→move→显式0/50/250ms预算→DOM/native前查→press-down→press-up→立即后查→明确pump/40ms后查→分别收集底板和trusted page实际收件。0预算不得替换；独立命令/IPC和前查必有耗时，必须用实际QPC/UTC记录move/press差。计划不是“原合并立即Click的完全等效复制”：前查可能改变时序，旧立即失败继续独立保留，必要时后续评审单个进程内分阶段序列以限制roundtrip影响，不在此批实现/执行额外试验。

| 候选 | 支持性结果（仍非单轮确证） | 反证/分辨点 |
|---|---|---|
| ignore更新迟到 | 同一冻结点，0失败而250成功，前后style/lookup变化与实际收件相关 | 若在稳定内侧、pre-down条件一致且clear仍进底板，不能仅归poll；一条50ms失败可能仍在far200ms休眠，非反证。 |
| hover/坐标变化 | 固定物理点因native rect/DOM轮廓变化离开可见内侧 | 冻结可见内侧且前后几何稳定仍失败，削弱此解释；按下精确几何仍可能缺失。 |
| region不匹配/迟到 | 稳定点跨停留仍失败、前端候选与native输入分离 | 后端已接受region本工具不能读取，始终N/A；无该字段不能宣称polygon错误或revision根因。 |

结果以“立即首击表现”和“稳定停留表现”分开，不通过加sleep把首击失败从验收中删除；一次输入失败照录，不重试直到成功。

## 拟议工具与焦点

实际首轮helper单独冻结在inputs/first-helper.ps1。新a5-native-input-v2.ps1默认dry-run，在任何Add-Type/WinForms/PInvoke前return；未来显式EnableNativeInput才有native路径。本批只运行默认guard，未传该开关。press-down/up只注入，不移cursor/不sleep；state和明确pump分开，删除旧隐含40ms等待。每个动作记录before/after cursor、native rect/HWND/point HWND/PID/foreground、extended style及bit20、UTC/QPC读取起止及注入起止；后端接受region明确null/N/A。底板收件时刻标明handler处理语义，不能假称OS down timestamp。

a5-native-plan.mjs是纯数据计划生成器，没有启动/执行live coordinator。focus采集读取真实activeElement的tag/role/label/tabIndex、:focus-visible、outline/box-shadow/border、glyph style/aria值与rect，识别DIV[role=slider]，不只查button。无样式数据或native截图确认时不签署可见焦点。既有VolumePanel支持箭头±1及Home/End等；拟议仅Tab后确认实际滑块，再ArrowRight/ArrowLeft并按实际值恢复，避免发明Enter/Esc语义或Home/End跳到100%。未来测试必须另记录原生前台/截图及恢复，不修改焦点样式。

## 验证和边界

7/7纯数据/静态测试，退出0：无执行默认计划、press无坐标/移动/等待、0ms保持0、同点三个停留、后查不能覆盖实际收件、DIV焦点支持以及guard位于GUI初始化前。PowerShell默认dry-run输出guiCreated=false/inputInjected=false/implicitWaitMs=0；静态Parser及JS语法检查通过。未编译/执行live C#、未现场跑0/50/250，方案不当作复现或修复。所有GUI/用户状态都没有被本批改变；未请求提权或重试此前拒绝命令。

后续现场恢复设计保留第5批原设置/播放器选择/音量/foreground/cursor及进程所有权，仅关闭新建测试窗口；箭头操作后的音量按实际原值恢复。当前loaded runtime字段仍N/A，不以离线源码或安装版本补造。

工具diff、精确时间线、原始输入/hash、预定方案、dry-run/测试/语法日志及离线包齐全。C1/A5 `[ ]`，原生矩阵仍未通过、首击原因未确认。等待独立审阅后再另行安排现场任务。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
