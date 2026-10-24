# 第13批：新版单次静态实验现场单

发布方负责验收，聊天“任务实现”（01a11622-f9c1-7900-b656-d0d9b04739ce）执行。当前状态：**只准离线就绪核对，未批准现场**。不得把第11批单次许可或本任务单视为新许可。

## 固定对象

工作树：`E:/CodexData/.codex/worktrees/m3-release-c/Wind0ws_Dynamic_island`。

仅使用 `dist/performance/a5-static-repair-r1-2026-10-16/experimental-release/`，不重建：

- EXE SHA256：`38102A0F66B93A4370155CCFD6F420BC8A3831646216B7076C67DB493F506E7A`
- DLL SHA256：`CBD7290142D7BA3140814F74387EF0D4F7AA356A1BB22D2764526EE1264B737D`
- 工具 scope：`a5-static-lab-four-v3-candidate`。两版 Release 的旧件及所有失败证据保留。

已审源以 R1 包的 source-hashes 为准，现场前核对工具、业务源、锁文件和选定二进制。只允许新增现场文档及独立许可记录，不允许静默修改已审工具、重建或替换版本。

## 先完成的离线就绪核对

执行方可复制本单、出匹配 v3 的 LIVE_PLAN、核对 hash、运行默认 dry-run，返回具体命令及全新输出目录。`PENDING_APPROVAL.json` 仍为 false。不得运行 --live、启动应用/helper/浏览器、读取现场用户状态、操作鼠标/前台、截图、播放器或真实配置/注册表。

实际允许来自人类的新回复；发布方核对回复后再派发执行指令。新记录须绑定 scope、4 次上限、两个 hash 及该人类消息引用。

## 将来单次现场范围

以已审 `scripts/run-a5-static-lab.mjs` 为唯一入口；正常外层 supervisor 启动同一 coordinator，不能直接 --session-worker 绕开退出证明。新建唯一输出目录，例如 `dist/performance/a5-static-lab-v3-2026-10-16/run-01`；存在即停。**最多一次应用启动尝试**，即使 0 点击失败也不自动重开。

临时启动本轮 Isle、Studio、不可变静态 lab 和 no-activate 底板；会改变本轮窗口位置/前台、移动鼠标并保存一次宿主桌面截图。不启动或控制播放器。CDP 仅限本次正常 WebView 的本机 9250；不另启元数据探测进程。进程映像由已审 helper 的 MainModule 在绑定实例中确认；权限拒绝即停，不改用调试器或句柄技巧。

前置条件：无既有 Isle、双屏与固定副屏可用、输入/捕获空闲、端口空闲、控制窗口及实际几何/DPI均确认。helper 启动前私有备份 settings.json/.bak 和 Run 的 Isle/legacy 类型/存在性/原值。应用会按真实 autoStart 写入本轮 EXE 自启路径或删除两值；原始备份不可变，受控期望须按 R1 接线证明。Studio 自动播放器顺序写入由 IPC 审计约束，仅允许已证明的自动变化及本轮 monitorIndex 变化；任何未知写入均停止并保持 pending。

主岛放副屏并隐藏，Studio 放主屏工作区左上+32。创建底板、一次 lab；要求真实 Applied/nativeReady、版本/HWND/URL/文档回执一致、实际 frame/DPI/style/Canvas 匹配。一次截图核对内外各5×5像素；不可用不换位置、换点或重复截图凑成功。两次 rAF 不等于显示器呈现证明。

坐标在任何测试移动/down 前一次冻结：内点取最靠宿主中心的全不透明5×5块（同距按y/x），外点按y/x取首个全透明5×5块且边距至少3px，起点为right+32/bottom+32且须落副屏工作区。复用同两点及起点，顺序 **内→外→内→外**；最多4次左键 down 及对应 up，无故意停留。记录实际 QPC 间隔，不能称为零延迟。

每次输入前核对前台、输入、几何、DPI、style、隐藏源与目标 root。目标 move 到 up 之间不查 status。两侧保存原始 down/up/click、身份、时间及 tag；缺失、双侧、错序、重复、错侧均停，后查 WindowFromPoint 不可替代收件。status 查询可能主动失效，单独记录，不武断归因。up 后250ms仅收齐观测，不改变首次结果。

## 停止和恢复

任一前置/日志/输入/收件/几何/激活异常即停，不补剩余点击，不加 hover、DPI、拖拽、性能或媒体矩阵。

只释放确实仍持有且策略允许释放的本工具 down；已释放不重复，已观察释放失败不自动追加。释放或物理空闲不明时保留进程、原连接和证据，禁止窗口/前台/光标变更与硬杀。

安全释放后固定顺序：lab正常close并确认销毁 → Studio正常close并等待持久化/销毁 → 审计并恢复本轮设置语义与源可见性、即时读回 → 对绑定本轮窗口 WM_CLOSE 并确认正常进程退出 → 无写入者时精确恢复两设置文件及两 Run 原值，逐项读回/ack → 关闭底板并恢复原前台/光标、即时读回 → helper 正常退出回执 → supervisor 记录 coordinator 实际正常退出。

若恢复 pending，仅允许使用本次仍存活 closure 的 `--cleanup-only <本次cleanup-control.json>` 续做未完成步骤。它不重开实验、不刷新4次预算、不发送新测试输入。执行超时/断管先查持久结果，不并发重试或冷用旧 HWND/光标；原连接/实例丢失则报告 pending，另做可审阅持久恢复，不自行扩大范围。

## 交付与验收

保持原始证据不可变；失败同样交付。公开包应含新原始数据、输入/文档/原生回执、只读状态轨迹、截图及冻结计划、恢复各阶段读回、helper/coordinator退出证明、工具/二进制hash、可独立运行的离线分析和manifest。排除私有设置/Run备份、Studio私有审计内容及cleanup token；不能用路径或口头结论替代恢复证明。

成功只说明该静态实验的4次收件与本次恢复通过。C1、A5 仍 `[ ]`，动画420/360/240、媒体、其他阶段与Spring不动。不得提交或推送。执行方交付后停止，由发布方独立复算验收。
