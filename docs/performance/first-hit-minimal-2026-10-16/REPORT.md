# C1 / A5 收尾与最小首击实验（2026-10-16）

用户直接要求换办法并“开始”。本次在用户指定计划所属的 m3-release-c 工作树执行；E:/Wind0ws_Dynamic_island 是较早实现，未把两者混为同一版本。保留既有脏文件和历史实验，不启动第12批协调器。

## 任务边界

| 子项 | 当前结论 | 剩余工作 / 完成条件 |
|---|---|---|
| C1-I 阻塞迁移与返回契约 | 实现及本轮定向检查通过 | 不再依赖 A5、首击、长期资源、旧95/55ms归因 |
| C1-P 当前 Release 性能 | 待验证，不宣称收益 | 一轮固定当前版本/基线的并发媒体与音量对照；只在出现已确认真实活动停顿时采栈 |
| A5-I 动画实现 | 保留420/360/240ms候选，已有实现回归证据 | 本轮参数单测3/3；原生用户手感不由单测代替 |
| A5-F 动画手感 | 待短人工体验决定 | 保留或回退候选，不把原生首击缺陷归因于参数 |
| H1 原生首击缺陷 | 产品未修复，独立发布阻塞项 | 最小原生→最小WebView→产品接入，每阶段先确认再扩大 |
| R1 长期资源 | 独立待办 | 独立长期内存/GPU轮，不阻塞C1-I实现结论 |

这不是把原来的完整验收伪报完成；C1/A5总项仍未全通过。历史报告中的停止/待审批属于历史批次记录，本次授权来自用户当前消息。

## C1 本轮检查

读取当前 commands/media.rs、audio.rs、cache.rs、settings.rs 与 native_task.rs：媒体/seek/shuffle/repeat及音量设备调用经 run_native，取色/像素化/同步图像和缓存文件经 run_background；媒体确认150ms为异步sleep。native池容量4、background容量2，满载直接结构化拒绝，不排无限队列。设置文件/自启相关同步工作已有spawn_blocking。

- `cargo test --manifest-path src-tauri/Cargo.toml native_task --lib -- --nocapture`：2/2，验证离开调用线程、容量拒绝和释放。
- `cargo test --manifest-path src-tauri/Cargo.toml services::media::control_tests --lib`：5/5，验证WinRT false/error、无会话、不支持seek不派发、声明支持/兼容派发。
- `node node_modules/vitest/vitest.mjs run src/lib/islandMotion.test.ts`：3/3。
- cargo shim本机不能启动，使用已安装stable/bin/cargo.exe及相应PATH后成功；第一次media::tests过滤为0项，不计入通过数量。

上述是实现/契约检查，不是当前产品真实并发帧时测量，也没有测试实际支持seek的播放器。

## 第一阶段：两个原生进程

`scripts/first-hit-minimal.cs` 为单文件WinForms程序：board与island使用两个进程和相同640×360 host；island用一次SetWindowRgn设固定胶囊；成功后HRGN归系统，失败释放。无鼠标轮询、穿透样式切换、产品代码/配置、媒体、注册表或输入注入。三分钟自动关闭，Esc正常关闭。坐标未跨DPI验证。

Computer Use（@oai/sky）基于实际截图操作，仅两次点击：

| 点 | island down/up/click | board down/up/click | 结果 |
|---|---|---|---|
| host中心(320,160) | 1/1/1 | 0/0/0 | 胶囊完整收件 |
| 胶囊透明圆角(165,105) | 0/0/0新增 | 1/1/1 | 独立进程底板完整收件 |

原始记录：island.tsv / board.tsv，含UTC、PID、HWND、局部坐标及Stopwatch ticks。WinForms的OnMouseClick回调先于OnMouseUp日志出现，不把记录顺序解释为OS先click后up。两进程均有closed记录。

限制：工具会激活目标；没有控制move→down间隔、没有no-activate或物理零停留结论；不是产品hover四edge回归。首次无参数启动因缺参数正常返回2、未创建窗口；首次csc路径格式失败后用绝对反斜杠路径编译成功。

## 第二阶段：最小Tauri/WebView

`src-tauri/examples/first_hit_webview.rs` 是独立example入口，不调用isle_lib::run或产品setup；清空配置中的自动窗口，使用独立identifier与本轮独立WebView profile。最终使用about:blank，build后eval固定文档，无网络资源，固定胶囊region，标题保存页面累计trusted pointerdown/up/click样本。三分钟自动退出。它复用当前依赖但不代表标准Release性能构建。

构建首次发现Tauri与项目windows绑定版本不同，HWND显式转换后继续构建；不改依赖或锁文件。

运行中暴露并修正的最小工具问题：example未自动带Common Controls v6资源，出现TaskDialogIndirect入口错误；沿用本仓库unit-test的resource静态链接方式解决。关闭错误对话框后进程仍存活，核对PID9048与精确映像路径后仅结束该失败实验进程，重新构建。run-01/02在进入setup时因未开启webview-data-url拒绝data URL（不是旧lab根因证据）；run-03的about:blank未执行初始化脚本，记录region-applied但未呈现，未点击，正常关闭。run-04改为build后显式eval固定文档，出现ready与真实画面。

**run-04 实际结果（DPI=1，仅2次点击）：**

| 点 | WebView新增事件 | 独立底板新增事件 | 结果 |
|---|---|---|---|
| host中心(320,160) | trusted pointerdown/up/click各1 | 无 | 进入WebView |
| 胶囊透明圆角(165,105) | 无，累计仍3 | down/up/click各1 | 到达底板 |

原始记录为webview-run-04/webview.tsv与board-webview-final.tsv；标题回执保存累计事件数组，不能把三个累计快照相加当6个事件。窗口标题仍显示loading不妨碍页面title回调，ready与点击取证来自实际回调及截图。页面原始事件time为performance.now，只与同页比较，不与底板Stopwatch直接相减。profile为本次隔离运行文件，gitignore排除，不是产品配置备份。

**机制结论：** 在这两个受控固定点、会激活目标的工具操作下，固定SetWindowRgn与当前Tauri/WebView可以共存并正确路由到跨进程下层；未证明无激活、立即move/down、动态轮廓、阴影、产品首击或跨DPI。四次实际测试点击总计：原生2次，WebView2次。没有重试同点凑通过。

## 下一步与停止线

本轮结束前已请求正常关闭WebView和底板，进程查询确认first_hit_webview/first-hit-minimal均不在运行；底板有closed记录。Tauri run退出没有执行尾部exited日志，不以该缺失日志声称有完整退出回调。没有保存或恢复用户原前台/光标位置，不宣称完整桌面状态恢复；未控制播放器或修改其状态。

`node scripts/verify-first-hit-minimal.mjs` 已对原始日志复核通过：原生/底板各恰好一组同点down/up/click、均closed、进程不同；WebView最终累计恰好三个trusted中心事件、前缀一致且没有外侧新增；其底板恰好一组透明角事件。实际测试点击总数4，不把失败启动计成点击。

可复现构建：Framework64 v4.0.30319 csc显式引用System.Windows.Forms/System.Drawing编译first-hit-minimal.cs；`cargo build --manifest-path src-tauri/Cargo.toml --example first_hit_webview`。运行时native程序参数为`board|island absolute-log-path`，WebView example参数为全新输出目录，拒绝覆盖已有目录。native程序需分别启动board/island，WebView阶段只启动board/example；静态坐标适用于本轮DPI1与主屏工作区原点，其他布局不得照抄本轮坐标。

本轮新增仅2个实验源码、1个日志验证器、报告/原始证据和计划增量；没有产品修复、发布、提交或推送。后续接入应复用已有共享几何，先以默认关闭开关验证稳定compact命中，再验证展开/收起，不把固定掩码直接套在动画上。

不继续扩建旧协调器，不做Spring/I1/C2，不增加旧数据追溯。如果最小WebView不能正常呈现或收件，直接记录这一最小差异；不重新引入产品配置恢复、播放器或完整场景。即使最小WebView通过，产品动画轮廓/阴影、DPI、宿主生命周期与不激活立即首击仍需单独解决。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
