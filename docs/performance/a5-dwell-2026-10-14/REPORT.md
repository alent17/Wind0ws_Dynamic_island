# 第7批：固定8次首击停留对照（10月14日执行、15日交付）

现场于北京时间10月14日23:47执行，10月15日完成文档交付；目录沿用任务日期。用户在明确停止等待现场允许后发送“继续”，本次按TASK-07固定范围执行。8次结束即停止，无换点、重试、扩矩阵、性能轮或业务/动画修正；C1/A5 `[ ]`，首击根因仍未确认。先完成协调器fake/默认无GUI验证，再启正常测试会话；没有重试此前被拒的独立元数据启动。

## 构建与前置

第2批标准Tauri保存EXE EA80838E71C64A9253364885A60D9AFD7EFF3C8DC5445A4AE7351D6DADA92D2C / DLL33B43C71DB2A218DD0DAFEE6EB4DD34FE59D5CC3FE78B8112B8DA0BD7CD0C292，启动前实际字节hash核对。正常会话CDP返回Edg/155.0.4283.45，记录于normalSessionRuntime，不为补字段另启应用；actual image API/loaded DLL查询仍未采到，仅有绝对spawn路径、二进制hash和PID/HWND。

原始证据的appPid=25612是Node spawn返回的PID；helperReady.boardPid=26164是底板PID；成功页面试次WindowFromPoint点下窗口owner PID为9440。三者角色分别记录：没有进程映像或父子关系证据，不能把9440称为测试isle进程或断言具体模块/进程角色。真实副屏x=-1920/1920×1080，双屏scaleFactor1，页面DPR1。8次冻结Compact及进入后的DOM点检均有效，坐标与方案完全一致。DOM elementFromPoint/rect只证明浏览器候选可见/非控件，不能代表native accepted polygon；该字段仍N/A。

固定top内侧(-960,15)、floating内侧(-960,26)各0/50/250ms，原top边界(-960,2)0/250参考。显式移外+300ms预算；每次configure预算700ms、必要的程序派发collapse只作前置重置，trusted=false不计原生试次。实际OS down/up恰各8次，无keypress、播放器控制/seek或新的播放器。

## 收件和时序表

| 序号 | 场景 | 预定停留ms | move操作结束→down操作开始ms | 底板MouseDown / 页面trusted click | 收件结论 | 按下前→末次查询bit0x20 |
|---|---|---:|---:|---|---|---|
| 1 | edge-top | 0 | 8.7674 | 0 / 0 | unobserved | True → False |
| 2 | edge-top | 50 | 58.2524 | 1 / 0 | bottom-plate | True → True |
| 3 | edge-top | 250 | 260.8336 | 0 / 1 | page | False → False |
| 4 | floating | 0 | 5.5972 | 1 / 0 | bottom-plate | True → True |
| 5 | floating | 50 | 59.559 | 0 / 1 | page | False → False |
| 6 | floating | 250 | 260.5195 | 0 / 1 | page | False → False |
| 7 | edge-top-original-border | 0 | 3.768 | 1 / 0 | bottom-plate | True → True |
| 8 | edge-top-original-border | 250 | 263.0884 | 0 / 1 | page | False → False |

8份完整QPC/UTC前后查询、cursor、rect、extended style、point HWND/PID、DOM候选/终态、底板和页面事件、动作参数在dwell-evidence.json。0预算实际约3.8–8.8ms，50预算约58–60ms，250预算约261–263ms；这含IPC/DOM/native查询/策略调用开销，不等于原合并立即click。QPC戳是操作边界，不是OS实际投递戳，底板UTC为显式pump时handler处理，浏览器为回调时刻。

第一条top内侧0预算：down/up接口均返回1，但没有记录到底板MouseDown或页面click。按下前bit20=true、末查false，末pointPID9440，不能把后查询当成该点击已送到页面。当前只记录底板MouseDown与页面click，未保存页面pointerdown/up及全量窗消息，无法确认是分阶段路由、激活吞事件、延迟/遗漏观测或其他原因；明确unobserved，不签署通过。

没有双收件冲突。4条页面click均trusted，3条底板回调分别独立归属，另1条无观测；observed仅意为观测到收件，底板收件不是A5命中通过。等待后成功也不能覆盖原立即首击失败。

## 能支持的解释与限制

本轮4条成功页面click，按下前bit20均false、pointPID9440；其余4条按下前bit20=true、pointPID26164（其中1条无收件）。固定点的接收结果随预定停留与当时扩展样式变化，支持异步穿透状态/时序这一候选。原边界点250ms有页面click，削弱“该点始终完全不可命中”的解释；不能排除hover几何/region异步更新后才匹配。没有最后接受polygon/revision、ignore实际调用戳、完整down/up路由或线程栈，单次/每预算一次也不能区分轮询相位与全部因果，不能定为纯几何问题或已定位唯一根因。

50ms top仍落底板，而floating50ms到页面，不等于固定50ms可用门槛；源cursor monitor检查后按33/50/200ms休眠，调度相位与区域更新均可能影响。此轮没有性能/CPU/PresentMon采样，不用本表推帧时、延迟分布或收益，不修改轮询频率/热区/动画/C2/I1/UI/Spring。

## 工具与验证

新增协调器共享executeDwellTrial，默认run-a5-dwell只输出无GUI计划，动态native/Playwright导入仅显式live。fake验证不初始化GUI：固定8schema、0预算保持、press无坐标、无效点止于down之前、释放失败停止后续动作、双收件保冲突。协调器4项+原计划9项共13/13；共享所有权fake9/9；默认无GUI计划及JS语法通过。现场使用第6批已验所有权/finally策略，8次up都返回1，最终cleanup无Held、未重复up；没有释放失败。

纯Node独立式离线核验（不import协调器/判定）：重算8个QPC间隔、底板增量、trusted click数、坐标顺序、各8次down/up、恢复和冲突标志通过，退出0。它是保存证据自洽检查，不是现场再次执行或独立验收，供发布方另行复算。tools-executed冻结实际本轮工具，diff及包可审。

## 用户状态与剩余项

本轮实际原音量为8%、REDMI电脑音箱2 Pro、静音false；网易云原本正在播放，恢复后仍在播放（位置自然推进，无seek/播放命令）。settings含显示器/播放器选择逐字段、audio逐字段、play-state与原值一致。关闭任务isle/底板，用户网易云7760/20024/22892保留，未启动Apple。foreground HWND658260恢复且读回一致；原cursor保存并调用恢复，但没有即时cursor回读，只能记恢复请求，不能宣称全部坐标已核实。

8次后不补点、不补额外现场字段。仍缺native accepted region和完整输入路由；首击表现仍未通过、根因未确认，整体C1/A5验收、其他edge矩阵/焦点/跨DPI/手感、当前构建匹配资源/region/性能及长期验证继续单独待办。无业务修改、重建、提交推送，已停止，等待独立审阅。

## 第7批来源表述修订（2026-10-15）

更正此前将点下窗口owner PID [omitted]称为测试isle PID的错误；spawn PID [omitted]与窗口owner PID [omitted]不得混同。修订仅依据原始dwell-evidence.json离线核对，原始证据和旧ZIP保持原字节，未启动、重采或输入。此更正不影响4 page / 3 board / 1 unobserved计数、bit0x20关联及根因未确认结论。修订包保留原工具，新增独立PID角色核验；C1/A5继续 [ ]。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
