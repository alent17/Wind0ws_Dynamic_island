# M2 浮窗重开空封面修复与验收

本轮（北京时间 2026-10-08 至 10-09）仅修复 M2 的媒体封面接收与过期快照保护。100 首真实切歌、同曲目重开、迟到封面、封面替换、明确清空和真实无会话恢复均通过；M3 保持 `[ ]`，不开展其他阶段。

## 已确认的事件顺序

修复前代码基于 `74db42b`，已包含 M3 资源预算，仅加入有界事件诊断。对真实 `get_media_info_cmd` 返回后的交付增加 **1800 ms** 延迟、关闭 HD 封面，以可复现方式检查初始化竞争；没有伪造该轮播放器的媒体事件或封面。此前 Release 第 50 首重开失败仍保留在 [原验收报告](../acceptance-2026-10-08/REPORT.md)。

记录表明：

1. 原生增量事件和主岛同步先携带 `albumArt: ""`，确定当前曲目 `Itadaki`，浮窗显示封面为空。旧发送方实际把“封面未变化”的省略编码成了空字符串。
2. 相同曲目的初始完整快照携带 41998 字符封面，但落在主岛时钟同步的 2500 ms 保护窗口，被整体丢弃。
3. 后续同曲目的主岛同步携带 648514 字符封面；旧接收分支只更新播放信息，没有更新封面。最终 Canvas 数为 0。

因此根因包含发送语义混淆、时钟保护误抑制封面、同曲目分支遗漏三部分。1800 ms 是诊断注入，不能据此推算自然发生率；未加延迟的试跑并非每次都失败。完整相对时序和修复后的诊断见 [event-order.json](event-order.json)。默认运行不启用追踪或延迟；追踪最多 128 条，仅保存字段是否存在、长度和曲目标识，不保存图片内容。

## 修改

- 原生增量事件与主岛时钟同步在封面未变化时省略字段；完整快照和明确清空仍保留 `albumArt`。统一区分缺字段、空字符串/null 和有效封面；保留现有字段别名兼容。
- 浮窗在同曲目的时钟保护窗口内也接收封面。迟到或更新的同曲目封面启动现有渲染路径；明确清空取消在途封面请求并清空显示状态。
- 初始快照记录事件和封面版本。新曲目事件之后不允许旧快照回退曲目；同曲目新封面或明确清空之后，不允许旧快照覆盖/复活封面。共享媒体 store 同样允许被无封面增量抢先的同曲目完整快照补图。
- 保留现有 track/request epoch、Canvas DOM owner、异步绘制保护；M3 下载、解码、缓存字节及可见尺寸/DPI raster 预算未放宽。

## 验收

环境为 Windows 11 build 26340、实际 Tauri/WebView2、隔离的 **debug custom-protocol** 构建，前端采用最终 production build。DPI=1，浮窗 200×395，主岛 372×289。诊断端口 9230；这是 M2 正确性与资源恢复诊断，**不是 Release 性能对照或 Idle 基线**。

| 场景 | 结果 |
| --- | --- |
| 同曲目关闭重开，真实初始快照交付延迟 1800 ms，HD 关闭 | 5/5 封面及 Canvas 就绪 |
| 缺封面字段、明确空封面、迟到封面、同曲目连续替换 | 均通过；明确清空移除 Canvas，最后一次替换像素正确 |
| 完整初始快照晚于同曲目新封面/明确清空 | 旧封面被忽略，没有覆盖或复活 |
| 网易云当前播放列表真实切歌 | 100 次变化、100 首不同曲目、100/100 标题与 Canvas/当前图片像素检查通过 |
| 第 25、50、75、100 首后关闭重开浮窗，HD 开启 | 4/4 重开像素检查通过；旧 WebView target 关闭 |
| 实际退出网易云，再启动并恢复当前队列 | 无会话时 Canvas=0，恢复后 Canvas=1、标题对应当前曲目、图片就绪 |
| 原生堆快照中的 Detached Canvas | 切歌 0/25/50/75/100、同曲目更新、合成恢复与真实恢复均为 0；未强制 GC |
| 前端单测、UI 回归、Svelte 检查、production build | 79 项、9 项通过，0 errors/0 warnings，构建通过 |
| Rust 单测与原生构建 | 51 passed / 1 ignored；原生构建通过 |

真实切歌使用原生媒体控制；像素检查取当前图片与 Canvas 的三个对应点，不能仅以 DOM 中存在 Canvas 判定通过。页面错误为空；重开错误、无法完成像素检查或 Detached Canvas 会让脚本失败。每 25 首关闭重开会销毁对应 WebView，因此堆快照代表本轮指定操作序列，不是单个永不重建的 renderer 连续承载 100 首。

播放器退出时可能短暂返回查询错误。恢复脚本等待成功读取的空会话及空 Canvas，不把查询失败当作无会话通过。

歌曲语料、窗口/DPI、逐次重开结果和堆统计见 [results.json](results.json)。原始截图、`.heapsnapshot`、完整原生采样和逐首日志保留在本机 `dist/performance/m2-reopen-2026-10-08/`，不提交大型诊断文件。

真实会话恢复后暂停播放、保持浮窗打开，额外采集 300 秒（188 个样本，无强制 GC）。全部 renderer private bytes 前/后一分钟中位数为 244.23/208.24 MiB，GPU 进程 private bytes 为 97.98/127.03 MiB，GPU committed memory 为 55.85/72.72 MiB。Renderer 有回落，GPU 并未回到前段中位数；**不能声称 GPU 回稳、内存收益或 Idle 达标**。该构建含远程诊断，既不是修复前后匹配的 Release 对照，也不能把自然 GC 回落当作 M3 优化收益。详见 [resource-recovery.json](resource-recovery.json)。本次 M2 勾选表示封面状态、DOM owner 与过期回调正确性通过；性能目标仍由尚未完成的 B0/A–F 验收负责。

关闭浮窗后再采集 60 秒（35 个样本）：浮窗 WebView target 已消失，仅剩 1 个主岛 renderer。Renderer private bytes 中位数为 110.25 MiB，GPU committed memory 中位数为 57.18 MiB；关闭窗口释放了该浮窗 renderer，但这些诊断数据仍不足以证明长期 GPU 性能目标。

## 重跑与后续边界

启动带本地 WebView2 调试端口的待验收原生构建后，可运行：

```powershell
$env:ISLE_CDP_ENDPOINT='http://127.0.0.1:9230'
$env:ISLE_ROOT_PID='<待验收 Isle PID>'
$env:ISLE_RUNTIME_DESCRIPTION='<实际构建类型>'
node scripts/verify-m2-reception.mjs
node scripts/accept-native-artwork.mjs dist/performance/m2-reopen-repeat/real-tracks
$env:M2_PLAYER_EXE='E:\CloudMusic\cloudmusic.exe'
node scripts/verify-m2-session-recovery.mjs
```

这些脚本会操作已授权的当前播放列表与浮窗；真实无会话脚本只停止路径等于 `M2_PLAYER_EXE` 的播放器并在失败时重启。接收测试临时关闭 HD/像素化，退出时恢复设置和此前播放状态。

M2 完成后独立提交并停止。M3 的 Release Benchmark C 仍未完成；未来前后构建必须都包含本次 M2 修复，并固定相同歌曲语料、DPI、窗口操作、播放/HD/像素化设置，避免把封面显示差异计作内存收益。其他阶段状态不变。
