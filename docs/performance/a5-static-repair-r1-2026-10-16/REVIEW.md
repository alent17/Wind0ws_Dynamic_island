# 第 12 批独立审阅：退回三项最小修复

已读取执行聊天与实际分发源。ZIP `FA03078A33BD41C38B46A633A7F5F80BF8B237F05C13154B2D106348A41F5174`，70,983,155 bytes，解包前实际核验 344/344 长度/hash（含 manifest 共 345 成员），未见私有配置或 cleanup-control 条目。独立解包目录 Node 回归 34/34 通过，但没有覆盖下面实际接线路径的缺陷。**候选暂不通过，不安排新现场。C1/A5 `[ ]`。**

## [P1] 初始化持锁执行 HWND getter，可能使 UI 与 worker 互等

`src-tauri/src/commands/window_region_adapter.rs:626–631` 的构造 worker 持有 `gate.lock()` 时调用 `hwnd(&w)`。锁定 Tauri 的 hwnd→dispatcher.window_handle→get_raw_window_handle→window_getter/getter 会派发到 UI 线程并阻塞 `rx.recv()`。与此同时，UI 的导航/标题/page-load 回调会取同一 gate；若该回调先于 RawWindowHandle 请求执行，UI 等 gate、worker 等 UI，形成死锁。deadline 也需要这个锁，不能解开等待。

应先在不持有 gate/OWNER 的情况下取得 HWND/其他需 UI 派发的值，再取锁提交纯状态。检查新增路径是否还有同类跨 UI 调用。给实际配置策略加无 GUI 回归，使注入的 handle provider 可验证调用时 gate 未锁；不要仅测试手动设置几个 bool。此问题是新候选的代码缺陷，不是第 11 批旧 Enqueued 的已证明根因。

## [P1] 自启恢复只接受启动前值，无法恢复正常实验启动产生的变化

`scripts/a5-static-input.ps1:61` 在 Restore-Persistent 中要求当前两个 Run 值与启动前 `$runValues` 完全相同，否则在写回之前拒绝。实际应用 `src-tauri/src/lib.rs:575` 启动时调用 set_auto_start；`services/settings.rs:151` 起会在启用自启时把 Isle 值重写为当前 EXE 命令，并删除 legacy 值。实验 EXE 位于独立目录，因此正常启动就会产生这类变化，第 11 批已有对应原始证据。

不能简单忽略注册表差异；需把原始快照与根据锁定实现/本次实际受控启动可证明的期望状态分开，严格核对选定 EXE/启动参数/autoStart 与两 value 的存在性/类型。只接受可审计的本工具启动变化，未知值继续拒绝。恢复每项后记录实际 ack/读回，部分成功后续清理只能接受已记录的原值，不能重新吸收外部变化。通过真正共享的恢复决策用 fake 注册表验证原路径→实验路径→原路径，以及未知/部分恢复失败，不接触真实用户 Run。

## [P2] 清理服务 5 秒断开，与实际清理阶段预算不匹配

`scripts/a5-static-cleanup-control.mjs:12` 对连接设置 5 秒超时，接到合法请求后仍保持该计时。restore 后端允许多个 3–5 秒阶段，客户端却等待 60 秒。实际清理超过 5 秒时服务端会先销毁连接，即使随后完成恢复，客户端也拿不到确认。

发布方使用原生产 serveCleanupOnly/requestCleanupOnly，注入仅等待 5300ms 的无 GUI 清理回调，实际复现：清理成功，客户端 `SyntaxError: Unexpected end of JSON input`。证据为独立目录 `cleanup-timeout-reproduction.json` 和 `reproduce-cleanup-timeout.mjs`；无窗口、输入或用户配置操作，临时 token 文件已删除。

接到并验证合法请求后应将读请求超时与执行清理预算区分，按实际共享总预算返回明确结果或可审计 pending，不能静默断开导致“恢复已完成/调用方报失败”歧义。加共享路径回归覆盖超过旧 5 秒且仍在合法预算内的操作、真实断管与重复请求互斥。

## 返修范围

同第 12 批内仅修上述三项，保留首包、v2、初始日志和原始第 11 批失败。没有 GUI/新会话/输入/截图/播放器操作、业务或动画改动授权。Rust 变动后更新必要受影响检查及标准默认/feature 构建，二进制和工具 hash/scope/pending 记录保持匹配；新修订包独立命名。完成后交发布方审阅，C1/A5 不勾选。
