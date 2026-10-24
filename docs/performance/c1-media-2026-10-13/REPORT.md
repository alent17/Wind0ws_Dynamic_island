# 第2批：媒体能力与控制错误返回

批次启动：2026-10-13；交付：2026-10-14（北京时间）。任务依据为发布聊天转交的 TASK-02.md，执行工作树 m3-release-c。

**本批修复与声明能力路径验证通过；没有声明支持 seek 的现成会话，有效 seek 验收仍为 N/A。C1、A5保持 [ ]，不提交或推送。** 动画420/360/240ms及Spring延期不变，没有开展C2/UI/region架构改造。

## 核对与最小修复

- `services/media.rs::control_media` 原来等待了WinRT操作，但丢弃其bool。现统一处理toggle/next/prev的结果：true代表请求获受理；false变为结构化code2001拒绝错误；WinRT error保留失败信息。获受理不等于播放器已完成操作。
- `seek_media` **原本已经检查false**，本轮没有把此前现象归因为遗漏bool。原路径没有能力门禁：Apple Music声明seek=false时仍派发TryChangePlaybackPositionAsync；API曾返回true，但SMTC时间线未到目标。本轮对非网易云先读取实际选中会话的Controls能力，不支持就返回结构化“不支持调整播放进度”及原始session ID，阻止位置变更派发。seek结果的bool检查复用同一结果处理函数。
- 显示层和命令层共享seek支持判定。网易云保留原有兼容尝试，即未声明seek也允许尝试，不新增能力查询来阻断它；TryChange返回false或error仍必须报错。此兼容策略有单测，**本批未运行真实网易云有效seek，不能声称新增功能通过**。
- `src/App.svelte::handleMediaAction` 移除播放态乐观翻转以及失败后再翻转当前值的逻辑，播放状态完全由实际媒体快照更新。这样拒绝不会反向覆盖请求期间收到的权威状态。现有后台150ms异步确认刷新不变，按钮可能等待实际快照；没有增加主线程同步等待，没有调整动画。
- command层原有后台run_native及`?`传播保持，mediaApi控制/seek直接返回invoke Promise。主岛seek=false时本来就没有范围输入；此只读入口与新命令门禁一致。没有新增UI布局/组件/Toast或状态控制器。

业务diff仅两个文件：`src-tauri/src/services/media.rs`（修复及5项测试）、`src/App.svelte`（错误/快照连接）。旧C1/A5工作树其他修改保留。`batch02.diff`是这两份本批文件的完整diff，before/和source-after/保留修改前后内容。

## 测试与执行

| 命令/验证 | 退出码与结果 |
|---|---|
| cargo test --manifest-path src-tauri/Cargo.toml --lib services::media:: --offline | 0；15/15，含新增5项控制测试 |
| node node_modules/vitest/vitest.mjs run src/lib/mediaStore.test.ts src/lib/mediaClock.test.ts | 0；17/17 |
| node node_modules/svelte-check/bin/svelte-check --tsconfig ./tsconfig.json | 0；0错误/0警告 |
| npm run build | 0；前端生产构建通过 |
| node node_modules/@tauri-apps/cli/tauri.js build --no-bundle -- --offline | 0；实际使用的标准Tauri Release构建 |
| node scripts/accept-c1-media-errors.mjs 9241 …/no-session-verified.json --no-session | 0；实际无选中会话，control/seek均返回code2001 |
| node scripts/accept-c1-media-errors.mjs 9241 …/apple-new-build.json | 0；真实声明能力路径及单独UI故障注入通过 |
| node scripts/accept-c1-media-errors.mjs 9241 …/no-session-after.json --no-session | 0；关闭测试播放器后再次验证无会话拒绝 |

新增Rust测试覆盖：true、toggle/next/prev/seek的false拒绝及序列化、WinRT error、unsupported seek闭包调用次数0、声明支持/网易云兼容各派发1次并检查false、无会话。测试的bool/error输入属于确定性结果映射单测，**不是声称真实Apple Music产生了每一种WinRT拒绝**。真实command边界由unsupported/no-session验证补齐。

保留失败/工具限制：第一次测试构造器与windows-core0.52不匹配（E0599），已改为Error::new后15项通过；npm run check缺少Windows命令shim，未安装依赖，改用同一已安装包的Node入口通过。首次直接cargo release未启用打包协议，页面为chrome-error、RPC报Origin header无效，未用于功能验收；该产物存unbundled-attempt/，随后标准Tauri构建修正构建入口。Apple Music最初只暴露空会话，要求真实曲目/有效duration后才验收；第一次UI注入仅拦fetch而实际走postMessage，未产生拒绝，已保留失败证据并补覆盖实际传输。以上失败不计通过。

## 新构建及旧对照保护

新Release另存于 `dist/performance/c1-media-2026-10-13/new-release/`：

- EXE SHA-256：`EA80838E71C64A9253364885A60D9AFD7EFF3C8DC5445A4AE7351D6DADA92D2C`
- DLL SHA-256：`33B43C71DB2A218DD0DAFEE6EB4DD34FE59D5CC3FE78B8112B8DA0BD7CD0C292`

旧保存baseline EXE/DLL仍为`00AE8D28…`/`A2451332…`，candidate仍为`0F1D5746…`/`754349D1…`；完整实际字节校验见binary-hashes.json。旧文件没有改写。HEAD、既有修改清单和新源码哈希另存，未宣称新构建适用于旧性能配对。

## 真实Apple Music及错误传播证据

实际选中source为apple，raw session ID `AppleInc.AppleMusicWin_nzyj5cx40ttqa!App`；仅该真实会话可用。当前曲目“飛ぶ時”，时长252000ms，capabilities.seek=false，playPause=true。没有模拟SMTC会话。

| 路径 | 实际前后证据 |
|---|---|
| 不支持seek | 暂停21000ms；目标60000ms返回code2001、message明确“不支持调整播放进度”和Apple原始ID；之后仍暂停21000ms |
| 播放与时间线 | control请求返回受理；实际playing=true，22000→23000ms |
| 暂停稳定 | 实际paused，24000ms，后续采样仍24000ms |
| 既有UI入口 | 主岛真实曲目展开、进度区存在；范围输入数0，seek只读 |
| 无选中会话 | 播放器启动前和关闭后，control_media及seek_media均reject，code2001/message“未找到可控制的媒体会话” |

UI错误处理另作**故障注入诊断**：对一次真实主页面的control请求注入延迟的code2001拒绝，并在等待期间重新投递从实际SMTC读到的暂停快照。该测试没有伪造会话，且不将注入结果当成真实WinRT false。实际传输为postMessage；捕获1次注入，前后按钮标签都是“播放”，实际SMTC仍暂停，console记录“媒体控制失败”。旧的失败后反向翻转会覆盖此权威快照，本修复没有残留乐观状态。代理和事件注入均恢复/停止，不作为性能样本。

有效seek仍为N/A：当前仅Apple Music的seek=false会话可用，没有声称目标位置成功。网易云兼容的真实成功路径未重测；已有兼容准入和bool/error处理不被破坏。没有盲加长等待去伪造位置变化，也未新增同步等待到UI线程。

## 恢复、交付与后续

本轮原系统设备为AB13X USB Audio、音量13%，并非上一批的20%；按本轮实际读取值恢复13%，不套用旧值。测试创建的Apple Music会话已暂停，测试启动的播放器和新构建进程已关闭；选择设置仅在必要时临时更改并恢复，本次没有改选择。播放测试使进度21→24秒，SMTC不支持seek，无法可靠恢复原进度，此限制明确保留。没有重置用户缓存、设置或播放列表，没有修改动画参数。

原始JSON保留页面实际invoke UTC/performance.now、成功/错误、真实前后状态、会话ID/能力、UI记录与恢复；这里没有帧时/IPC/资源新成绩。完整diff、源码状态、测试日志、构建哈希及证据在执行目录和审阅包中。

当前交付只收口能力/错误返回：C1旧95/55ms、A5原生交互、匹配资源成本、region增长以及长期稳定性继续待后续；C1/A5保持[ ]。不开展C2/UI/架构/Spring，不提交或推送，停在可审阅状态。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
