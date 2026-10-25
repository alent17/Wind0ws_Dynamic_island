# 第二轮图片链路内存优化

本轮继续保留现有功能和动画，未使用 computer use。工作区仍有其他任务的修改，以下只归因于本轮图片处理变更。

## 已实施

1. **像素化复用 RGBA 缓冲区。** 原实现保留解码图，再复制一张输出图。新实现消费解码图、转换为可复用的 RGBA 缓冲区，分块求平均后原地更新 RGB，保持每个像素的 alpha。RGBA 输入在 1280×1280 时少分配 6,553,600 字节。
2. **Base64 直接追加到最终 data URI。** 按编码长度一次预留容量，避免先分配完整 Base64 字符串再复制到 `format!` 的结果。该路径覆盖像素化输出、高清封面下载回退和 SMTC 缩略图；没有改变 MIME、编码和 IPC 返回格式。
3. **图片读取进入限流后再执行。** 图片效果和取色请求先获得单个重图片工作名额，再读取文件或解码 Base64。等待中的请求不再额外持有一份已读入的二进制图片；入场失败的请求不会先做文件 IO。入场锁只获取一次，避免递归获取导致死锁。
4. **按已知文件长度预分配读取缓冲区。** 本地封面、缓存封面和下载后的临时文件按元数据长度预留容量，避免流式读取缓冲区几何增长造成额外分配。仍限制实际读取字节数；文件在元数据查询后增长、缩小或者容量提示失效时都保留原来的限额检查。

原始 Base64 IPC 字符串仍可能存在于排队请求中；本轮没有声称消除了全部排队内存。整程序 WebView2/GPU 常驻成本也不包含在下述图片库收益里。

## 可复现的分配对照

新增 `scripts/benchmark-image-memory.rs` 与 PowerShell 运行器。探针直接编译本仓库图片、预算和输入加载模块，链接现有 Release 图片库。旧路径保留旧的复制与编码流程作为参考。自定义分配计数器记录 Rust 请求的活动堆峰值与累计分配流量。

同一张确定性随机 1280×1280 RGBA PNG，普通处理与像素化各比较三轮，每次新进程；交替运行顺序。两种算法的输出长度和摘要逐轮一致，像素正确性另由单元测试验证。

| 场景 | 指标 | 旧路径 | 新路径 | 变化 |
| --- | --- | ---: | ---: | ---: |
| 普通图片 | 请求堆峰值 | 39.35 MB | 37.52 MB | 减少 1.83 MB，约 4.7% |
| 普通图片 | 累计分配量 | 109.84 MB | 90.88 MB | 减少 18.96 MB，约 17.3% |
| 像素化 | 请求堆峰值 | 35.44 MB | 27.05 MB | 减少 8.39 MB，约 23.7% |
| 像素化 | 累计分配量 | 64.16 MB | 42.73 MB | 减少 21.44 MB，约 33.4% |

没有通过减少图片尺寸、改变色彩、移除透明度或减少动画来获得这些结果。记录了每次耗时，但同机编译负载导致波动明显，本轮不把耗时差异作为 CPU 改善结论。

这些是**独立图片流程的 Rust 请求分配量**，不是任务管理器工作集，也不包含系统分配器内部开销、WebView2、GPU 或整个应用的常驻内存。累计分配量不是同时占用量。单个合成输入不能代表所有封面格式与网络条件。耗时是在存在其他工作负载的环境下得到的诊断结果，不能承诺同等整机 CPU 改善。

原始证据：[image-allocation.json](performance/round2-2026-10-24/image-allocation.json)、[汇总](performance/round2-2026-10-24/image-allocation-summary.json)。JSON 保存探针和生产源码哈希以及输入摘要。探针二进制与合成大图放在忽略的 dist 目录，不作为源码提交。

复现：

```powershell
./scripts/benchmark-image-memory.ps1
cargo test --lib --manifest-path src-tauri/Cargo.toml
npm run test:ui -- ui-tests/canvas-artwork.pw.ts ui-tests/cover-lifecycle.pw.ts ui-tests/lifecycle.pw.ts --output dist/performance/round2-ui-results
```

## 回归结果

- Rust 最终串行全量回归：153 项通过、1 项忽略、0 失败（`-- --test-threads=1`）。
- 图片、浮窗、媒体同步、隐藏生命周期等选定的 Playwright 回归：57/57 通过。
- 新增测试验证 1/12/1280 像素块、非整块边界、RGB/RGBA/灰度/灰度 alpha/16 位 RGBA 的逐像素一致性；验证 RGBA 内存地址复用、PNG IPC 往返和 Base64 补位/MIME 一致性。
- 原有任务限流、取消和损坏图片处理继续保留。
- 增加读取容量提示的精确容量检查，以及提示过小、过大和输入无限增长时的字节上限检查。
- 并行全量测试曾有一次既有 `services::apartment::tests::preserves_existing_sta_without_uninitializing_its_owner` 在清理 COM 后断言未初始化失败；单独复测和串行全量复测通过。记录为测试稳定性问题，本轮未修改 COM 生产代码，也未将并行运行描述为全部通过。并行运行中所有图片新增测试均通过。
- 最终 Release 构建成功，仍有既有的未使用进度方法及字段警告。
- 内存门禁扩展为 5 项测试，要求每个采样快照同时包含 WebView2 browser 与 renderer；前端未就绪的纯 Rust 进程不能以低占用“达标”。证据无效时会覆盖旧的通过摘要，防止读取到过期成功结果。

## 100 MB 目标

本轮收益集中于图片处理瞬时分配。不能将减少的请求分配量直接从上一轮约 181 MB 的整程序常驻工作集中相减。Release 构建和进程树补充采样的结果在下方记录；未完成所有场景正式验收前，不宣称达标。

本轮第一次 Release 进程树短测（RootPid 37076，10 秒预热、30 秒采样）仅有 Rust 主进程，未出现 WebView2 browser/renderer。启动日志还出现 `the underlying handle is not available`。该次约 6.5 MB 的结果**无效**，不是整程序内存达标。已补强门禁并将 [该次摘要](performance/round2-2026-10-24/release-memory-summary.json)明确标记 `valid:false, passed:false`。保留 [原始采样](performance/round2-2026-10-24/round2-release.json)供定位，不把此启动异常归因于图片优化，也不将当前 Release 运行时功能描述为已经验收通过。

启动异常与此前 COM 并行测试问题均需要进一步排查；WebView2 未正常存活时不能进行内存和动画的正式验收。本轮图片库的优化收益与该启动异常分开记录。

延长预热后重试（RootPid 34868，45 秒预热、15 秒采样）仍未出现 browser/renderer，日志重复窗口句柄不可用。该次 [原始采样](performance/round2-2026-10-24/round2-retry.json)和 [门禁结果](performance/round2-2026-10-24/retry-memory-summary.json)也保留为无效证据。两个测试实例均在采样后退出；没有停止其他程序。启动日志摘要见 [startup-diagnostics.json](performance/round2-2026-10-24/startup-diagnostics.json)。

新门禁重新检查上一轮有效进程树仍得到 `valid:true, passed:false`：平均 180.64 MB、峰值 184.15 MB。因此目前没有证据说明整程序已达到 100 MB。
