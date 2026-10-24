# 第4批：旧C1 95/55ms证据复核（2026-10-14）

**复核完成，旧“genuine presentation gap during active animation / fully inside actual animation”结论撤回。** 95.0015/54.9938ms显示间隔确实存在，但实际活动相交/完整包含以及线程阻塞根因均无法确认。C1/A5保持 `[ ]`。本轮只读取旧文件、编写离线复核脚本并修正文档；无采集、构建、业务/动画/C2/I1/UI/Spring修改，无提交推送。

## 精确原始来源

行号从原CSV首行表头计为1，没有截切后的重新编号。

| 项目 | 95ms | 55ms |
|---|---|---|
| 原文件 | matrix-presentmon-c1-user-r4.csv | 同左 |
| 原行号 | 669 | 1687 |
| MsBetweenDisplayChange | 95.00150000000001 | 54.99380000000000 |
| PID / swapchain | 6052 / 0x21E4A377F30 | 同左 |
| VidPnSourceId / LayerIndex | 0 / 0 | 0 / 0 |
| 场景、旧归属动作 | animation-only，第13次展开 | combined，第14次收起 |
| 原TimeInDateTime（无时区标签） | 2026-10-12 21:18:57.353099200 | 2026-10-12 21:19:05.406288100 |
| 原CPUStartDateTime | 21:18:57.255342900 | 21:19:05.344404800 |
| MsUntilDisplayed | 2.9399 | 4.6092 |
| MsCPUBusy / MsGPUTime | 97.7563 / 0.1783 | 61.8833 / 0.1837 |
| 保存animationStart UTC | 05:18:57.009Z | 05:19:05.099Z |
| 保存animationEnd快照UTC | 05:18:57.359Z | 05:19:05.454Z |
| 保存start→end performance跨度 | 349.6ms | 355.4ms |

完整原始行及全部字段保存在source-rows.csv和historical-review.json，完整源CSV/原分析JSON/各场景矩阵/对应progress/status/bounds/命令/日志全部附包，旧输入未改写。27项输入逐字节SHA-256校验：25份旧r4原始证据及2份当前可用分析/协调器源码。当前源码可重建旧分类算法，但没有当时矩阵harness源码hash/快照，不能宣称确认了历史每行执行实现。

## 旧分类为何成立、为何不能沿用

可用旧分析器先把first CPUStartDateTime加Z解析，减status.startedUtc，按最接近整小时取offsetHours；M6/C1均得到16。对TimeInDateTime直接减该偏移，漏加MsUntilDisplayed；显示区间起点再减MsBetweenDisplayChange。它先只保留结束点落在窗口内的行，遗漏可能结束于窗口之后却跨窗口尾端的间隔。活动窗口用保存animationStart→animationEnd，animationEnd其实是transition-settled快照。

此算法复算得到旧95ms端点05:18:57.257Z–57.353Z（JSON显示截至毫秒）和55ms端点05:19:05.351Z–05.406Z。二者落在旧snapshot envelope内，因此原inside-animation标签可复现；标签所指的窗口不等于实际style-active span。

原矩阵只保存animationStart/animationEnd快照及observedDurationMs；progress的transition-settled逐字段与其一致。没有lastChange、精确lastStyleMutation、独立确认时刻或每帧样式序列。报告描述的100ms稳定契约不能据此恢复一个精确末次mutation，更不能把确认快照直接作为活动终点。所有旧343–374ms表述现改为起点至确认跨度。

## 时钟证据及明确限制

C1 status.startedUtc为05:18:50.0349812Z，首CPUStartDateTime为21:18:50.310277200；窗口首UTC为05:18:51.3179005Z，带Qpc/Frequency。旧CSV只有TimeInDateTime和CPUStartDateTime、无TimeInQPC，命令使用--date_time而非--qpc_time。没有保存PM日期转换来源/时区标记、成对帧timestamp与UTC锚点、时钟残差上界。不能用窗口QPC给这些没有QPC的帧建立精确对齐。

减16小时能把整份CSV大致放入捕获范围，只证明该候选偏移合理，无法精确校准。若额外假设所有显示行都位于协调器launch/finish UTC内，C1允许的常数偏移约57597.1904753–57601.2861652秒、M6约57597.5261606–57601.7483634秒；这是粗包络的条件约束（秒级），不是逐帧校准或确定误差区间。不能凭“China Standard Time通常UTC+8”推断此旧CSV为何相差16小时，更不能硬编码偏移升格为确证。

## 条件敏感性分析（不作最终归属）

额外假设：①exact16小时映射正确；②TimeInDateTime作为Present起点，MsUntilDisplayed适用于该行；③历史记录器确按文档在至少100ms无样式变化后保存animationEnd。保留原9位小数，不把Date.parse截到毫秒的损失当精度。

| 区间 | 条件显示起点→终点UTC | 确认减100ms | 起点晚于近似尾段边界 |
|---|---|---|---:|
| 95ms | 05:18:57.261037600→57.356039100Z | 05:18:57.259Z | 2.0376ms |
| 55ms | 05:19:05.355903500→05.410897300Z | 05:19:05.354Z | 1.9035ms |

在上述假设下二者落在近似确认尾段。若多减5ms，起点就提前到边界前2.9624/3.0965ms，分类变化；旧证据没有小于2ms的映射误差保证。这既不能确定“实际动画内卡顿”，也不能确定“没有卡顿”。确认减100ms不是精确last mutation；没有线程等待/运行栈，MsCPUBusy和低GPU时间不能定位COM、锁、DWM或具体阻塞线程。

## 当前状态与最小剩余验收

C1历史REPORT正文、Decision与计划摘要/C1 Verification/状态表已直接修正，不只追加尾注。旧统计表清晰标作被复核取代的历史算法输出。第2批真实Apple播放/暂停/时间线、能力/错误、只读入口及独立标记UI故障注入已独立验收；第3批资源链路/旧A5诊断通过，与第2批新Release性能验收分开。

| 剩余项 | 类型/现状 | 最小下一步（本批不执行） |
|---|---|---|
| 旧95/55ms活动归属 | 未知证据，本轮历史复核已完成 | 必要新轮保存精确首/末/确认和确证frame时钟锚点；旧证据无法补成确证。 |
| 当前媒体修复构建C1性能 | 后续测试，未验收 | 匹配当前实现/工具/语料的有界Release前后对照；旧A5不能代替。 |
| 控制拒绝与快照覆盖 | 已确认缺陷，修复并独立验收 | 保留第2批证据，无需盲测已通过路径。 |
| Apple seek=false | 真实不支持能力，拒绝/只读正确 | 不是必须修成支持的缺陷；另找声明支持真实会话验有效seek，当前N/A。 |
| 活动停顿根因 | 未知，无相关线程栈 | 仅对新轮确认的真实连续活动停顿安排最小ETW/WPA，勿自动提权。 |
| A5原生视觉/命中 | 后续测试，夹具10/10不足 | 有界覆盖4edge/style、反向/hover/隐藏/封面/焦点/低动效。 |
| region/IPC成本 | 已测约+61%请求/参数字节，影响尚未关门禁 | 匹配当前构建核对字节/在途/native更新成本，不实施C2/I1。 |
| 长期内存/GPU | 未知，短轮不证回稳 | 后续单独长期验证，不扩大本批。 |

## 交付与离线重算

纯Node运行`node scripts/review-c1-historical-gaps.mjs <审阅包根目录>`：校验27项源hash、精确定位2行、核验PID/chain/动作、重建旧分类和progress快照，输出带假设/缺失字段的复算JSON。退出0；JS语法检查及git diff --check通过。原r4源文件在脚本前后再次校验未变；本批没有启动播放器/应用、调整音量/屏幕或采集会话，无用户状态需要恢复。

交付batch04.diff、原输入/hash、两条精确原行、脚本/复算JSON、before/after文档和离线审阅包。C1/A5仍 `[ ]`，Spring延期，已停在可独立复核结果。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
