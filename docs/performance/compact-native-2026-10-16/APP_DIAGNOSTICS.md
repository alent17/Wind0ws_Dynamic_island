# 启动阻断与空审批列表：本地诊断

2026-10-16，用户确认 /approve 列表为空。未再次尝试启动被拒动作，未更改权限/安全设置或删除应用数据。

已确认：
- Windows安装包OpenAI.Codex版本26.1013.3656.0。
- 当前会话提供的权限为danger-full-access / never；本地项目记录为trusted。
- 检查的两个用户Codex目录无rules目录、无requirements.toml；这不排除组织、设备、云端或应用内策略。
- 最近应用主日志中有23条Local Work executor rendezvous unavailable、20条app_server_connection.reconnect_failed；含WebSocket连接未建立即关闭及超时。该日志未发现blocked by policy或Auto-review的文字记录。
- 日志还提示features.guardianv2.thread_context已弃用且忽略。这不证明它是阻断原因，因此没有修改配置。
- 上述连接故障与命令拒绝/审批列表空的因果关系尚未建立；不能把现象升级为根因。
- 临时测试服务已关闭，当前没有待执行点击。

建议恢复顺序：结束其他活动任务后正常退出并重开应用，回到原聊天，先检查普通只读命令及本地执行器连接是否恢复；不要通过放宽策略、切换工具或间接执行重放被拒启动。若审批入口仍空或命令仍被拒，通过/feedback反馈应用问题，由用户选择是否附带会话/日志。不要自动发送日志。

反馈摘要：Windows Codex 26.1013.3656.0；已授权本地测试命令在创建进程前返回blocked by policy；/approve命令存在但拒绝列表为空；读文件/编译等命令可运行；同日应用日志反复出现本地执行器WebSocket/rendezvous连接失败。请求确认拒绝来源及桌面单次授权入口是否工作。没有原始日志、令牌、配置全文或私有备份嵌入本文。

官方资料：
- https://learn.chatgpt.com/docs/reference/slash-commands
- https://learn.chatgpt.com/docs/sandboxing/auto-review
- https://learn.chatgpt.com/docs/reference/troubleshooting

本报告是诊断与恢复建议，不代表问题已经修复。

## 重启后的复查

2026-10-16 22:56（Asia/Shanghai）Codex 已重新启动。普通终端只读命令可以执行，但新会话日志仍连续出现 `app_server_connection.websocket_error`、`1006/open_timeout` 和 `Local Work executor rendezvous unavailable`，因此重启没有修复连接。

进一步确认：
- OpenAI 官方状态页当时显示服务正常。
- WinHTTP 和进程环境变量没有配置代理，但 Windows 当前用户代理已启用。
- Codex 后端实际连接本机 `127.0.0.1:7890`；监听进程为 `<local-path>
- FlClash 活动配置使用 `mixed-port: 7890`、`mode: global`。Codex 经代理后的部分请求能够返回，但 rendezvous/WebSocket 仍反复超时。
- 这些证据把故障范围缩小到当前代理节点或代理后的 WebSocket 路径；尚未证明是哪一个远端域名或具体规则失败。

下一项有意义的恢复动作是切换一个支持长连接/WebSocket 的 FlClash 节点，或重启 FlClash 核心后重新验证。该动作会影响整机网络，未在本次只读诊断中自动执行。
