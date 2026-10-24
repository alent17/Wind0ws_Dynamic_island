# 第 6 批：原生命中失败离线定位及诊断方案

执行聊天：`01a11622-f9c1-7900-b656-d0d9b04739ce`。

工作树：`E:/CodexData/.codex/worktrees/m3-release-c/Wind0ws_Dynamic_island`。

第5批证据链独立复核通过，但完整原生矩阵不通过。49项manifest、原始输入/底板收件及恢复复核通过；四edge和首次style-switch各一次点击进底板，主页面无click，后约40ms point PID均30228而底板PID30404。C1/A5仍 `[ ]`。

## 强制范围

**本批只离线分析、修改/准备测试工具并做静态或纯数据验证；不启动isle/浏览器/播放器/底板，不创建GUI，不实际注入鼠标键盘，不采集、不重建、不修改业务/动画/原生区域，不提交推送。** 不重试或换方式执行此前被自动审批拒绝的“补充启动Release读取运行时元数据”。后续现场诊断由发布聊天在审阅工具后另行安排。

## 已发现的关键线索

- 实际运行的first-run-tools helper的click分支是SetCursorPos后立即SendInput；没有移动后停留。
- 产品cursor monitor按inside33ms/near50ms/far200ms轮询，再set_ignore_cursor_events。命中测试的后采WindowFromPoint在约40ms之后。
- floating-hover-interrupt在按下前已经有move/hover等待，edge从外部移入后立即按下；这可能混入不同输入时序。请从保存的动作记录精确核对，不仅照搬此推测。
- 事件和图片仅证明各自采样时刻。不要把未记录的pre-down穿透状态/原生region补造成事实；不能据此确定纯几何错误，也不能简单说工具坏了或加大热区。

## 离线分析

1. 整理5条失败逐阶段时间线：设置切换/最后离开、DOM选点/最近快照、native移动与按下（当前哪些合并且没有独立戳）、底板MouseDown、page事件、40ms后查询。明确记录字段与推断字段，列缺失项。
2. 对比有真实page click的floating路径，检查原生/视口/DPR转换、选点是否靠边、hover后几何是否可能变化、底板/foreground及输入target断言。图片中的候选位置只用于输入后几何证据，不冒充按下瞬间。
3. 只读追踪cursor monitor、set_ignore_cursor_events、applyIslandRegion及region revision路径，给出能区分“异步穿透状态迟到”“坐标/形变变化”“区域本身不匹配”的最小对照方案。root cause仍未确定时明确写出。
4. 补焦点判定方案：支持实际DIV[role=slider]等已有可聚焦目标，记录focus-visible/outline以及合法键盘行为，不以只查button误报产品缺陷。本批不改产品焦点样式。

## 准备受控工具（本批禁止现场运行）

把移动、状态查询、按下/抬起拆分；press操作不得偷偷移动坐标。后续每次要记录输入前后实际cursor、native HWND/point HWND/PID、目标原生扩展样式（只读）、DOM候选/可见边界和高精度时间；有能力读取的字段才记，无法观测native最后接受region就明确N/A。WindowFromPoint或style查询不能替代实际底板/page收件。

提出预先固定的单点/停留对照，例如同一可见内侧坐标的0/50/250ms停留，各一次，并有floating对照；这是可审阅方案，不能本批执行。禁止换点/改热区/重复到成功后隐藏原失败。保持立即点击失败与稳定悬停结果两个独立指标，不用增加sleep把首击问题从验收中抹掉。

工具提供真正无副作用的dry-run/计划生成路径，可用纯数据检查schema/命令顺序/是否意外启动GUI；现场路径默认不执行。将实际首轮版本、拟议版本及diff分开。若编写离线测试，测试重点是press不移动、0ms不被悄悄替换、输入前后状态与收件人分开、失败不被后查询覆盖。

## 交付

交付离线时间线/证据引用、候选根因及反证条件、最小现场对照方案、工具diff和无副作用验证结果、权限限制与恢复设计。同步计划为“原生矩阵未通过、首击原因未确认”，不要勾C1/A5或将方案当已复现/修复。未保存的实际loaded runtime字段保持N/A。

完成后停在可审阅状态，发布聊天会独立验收；本批的完成不授权现场重试被拒动作。
