# 第 2 批：媒体能力与控制错误返回

执行聊天：`01a11622-f9c1-7900-b656-d0d9b04739ce`。

执行工作树：`E:/CodexData/.codex/worktrees/m3-release-c/Wind0ws_Dynamic_island`。

上一批独立验收完成：交付 ZIP 的 92 文件哈希、离线回放 4/4、门禁单测 5/5，验收方另写原始输入复算也通过。全转换并发覆盖子任务通过；region 请求 784→1261、参数字节 +60.94% 的成本记录保留。CPU 配对尚不足，C1/A5 仍 `[ ]`。

## 本批次范围

收口 C1 媒体功能/错误传播缺口：Apple Music 声明 seek=false，位置变更请求却成功返回且时间线未到目标。先确认会话身份、能力与命令结果，再作必要的最小修复。允许修改本项所需媒体 Rust 代码、对应测试以及必要的既有错误处理连接；不重构 C2/UI/架构，不调动画，不开展 I1，不作 Spring 迁移。保留既有 C1/A5 未提交改动和保存的 Release 对照。

## 已核对的代码事实

- `services/media.rs::seek_media` 已检验 TryChangePlaybackPositionAsync 的 bool 为 false 时返回 AppError；不要再次宣称它遗漏 bool，也不要盲加长等待。
- `control_media` 的 TryTogglePlayPauseAsync / TrySkipNextAsync / TrySkipPreviousAsync 结果为 bool，目前只传播 WinRT error，却丢弃 bool。拒绝操作不能被报告为成功。
- MediaState / 会话已有能力字段。显示层为网易云保留未声明 seek 时的兼容尝试，不要无证据破坏该现有行为。

## 执行与验收要求

1. 沿实际选中会话路径核对 source、原始 session ID、控件能力及命令结果，说明 Apple Music seek=false 情形为何仍可报告成功。对于不支持的能力，应用必须诚实地禁用已有操作入口或返回结构化的“不支持”错误；不得把不支持的平台能力伪装成有效 seek 通过。
2. 对明确缺陷作最小修复：媒体控制 WinRT bool=false 必须传播拒绝错误；对于 seek 能力与入口/命令不一致，在不破坏已证实网易云兼容行为的前提下收口。不要为了确认状态把新的同步等待塞回 UI 线程。WinRT 请求获受理仍不等于实际位置变化，报告必须区分。
3. 测试覆盖实际错误路径：bool=true、bool=false、WinRT error、不支持 seek、无会话；检查 command 层/前端错误处理不会把拒绝变成成功或残留乐观状态。不要靠仅镜像实现的测试宣称真实播放器行为通过。
4. 在新构建上用真实 Apple Music 验证 play/pause 和不支持 seek 的诚实结果、时间线状态及错误传播。对于声明支持 seek 的可用会话，再验证有效目标位置；如果当前仅 Apple Music 的 seek=false 会话可用，把“不支持路径正确”与“有效 seek 尚无支持会话验收”分开，不反复对同一不支持会话盲测。
5. 复用上一批可靠采集工具和原始时钟口径，只跑与本项相关的回归/原生命令验证。旧保存 baseline/candidate 不改写；新业务构建、源码状态和哈希另存。恢复用户原播放/音量/设置，无法恢复的进度明确记录。
6. 交付最终 diff、相关测试/命令退出码、真实功能前后证据和限制、报告/计划更新。保持 C1/A5 `[ ]`，不提交或推送，在可审阅状态停止，由发布任务聊天独立验收后再安排下一批。

## 保留的后续缺口

旧 C1 95/55 ms 的修正口径复核、A5 原生交互矩阵、资源采样同步/匹配成本、region 更长动画带来的请求增长和长期稳定性仍待后续；本批次不同时改这些路径。不要自动触发此前用户取消过的 WPR 管理员提权。
