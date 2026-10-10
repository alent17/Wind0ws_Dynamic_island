# M3 封面处理回归修正：对照协议

M2 保持 `[x]`；协议制定时 M3 保持 `[ ]`，2026-10-10 完整验收通过后更新为 `[x]`，边界见 REPORT.md。本轮只修改 M3 的图片准备、缓存发布 MIME 和普通 Canvas 栅格/采样策略，M4 及后续业务阶段不动。原有 PlayerControls 本地修改不纳入本轮。

## 定位与最小修正

旧 M3 对无需缩放的封面仍完整解码再编码 PNG。使用当前 Release image 库、同一 101 份冻结输入的独立原生诊断，三轮处理时间中位数为 7.400 秒；完整校验后保留原字节为 1.625 秒。输出合计 170,855,819 → 74,714,453 bytes。101 份均借用原字节，无新编码副本。这证明不必要编码的原生成本，不证明 Renderer retained-image 或泄漏归因；具体整应用变化以正式轮为准。

合规静态 JPEG/PNG 且不超过 1280 edge：保留原编码、ICC/元数据与 MIME；仍执行一次完整有界解码，防止只检查 header 放过损坏像素数据。超尺寸、其他格式与 APNG 沿用受限单帧 PNG 输出。下载字节/并发、解码尺寸/分配、图片工作队列、磁盘及 JS 缓存、pixel_size 错误保护保留。

普通 Canvas 采用 2× visible×DPR 的有界 backing（上限 1280，不放大小图）及 high smoothing；像素模式保持原 1× 栅格和 nearest-neighbor 语义。画质以实际输出对照确认，不能只凭公式宣称改善。

上述栅格方案为候选 A。其六轮结果虽已消除大部分峰值回归，但恢复 Renderer +13.56%、Tree +5.46%，未通过警戒值，M3 不能勾选，见 CANDIDATE_A.md。候选 B 对普通封面保留已受 1280 edge 限制的自然栅格，去除第二次缩小，像素模式继续 visible×DPR。1280 上限、不放大小图、原格式完整验证及所有字节/工作/缓存限制不变。B 另跑全新前后各三轮，不混用 A 的控制数据；正式 B 只采用 `dist/performance/m3-correction-b-2026-10-09/runs-2026-10-10/`。10 月 10 日主机重启中断旧批恢复段后，旧 `runs/` 整批排除，六轮全部重新采集。

## 匹配构建与正式条件

参考侧保留相同 M2、撤回 M3（对应 2340164 服务）；候选侧只加入本次 M3 修正。两侧同 Cargo/Cargo.lock、生产前端、release opt-level=s/LTO/codegen-units=1/panic=abort 及测量适配器。新增 codec 调用/耗时计数；参考下载身份探针改读发布前的完整响应 bytes，避免旧报告里的共享缓存短读混淆。所有哈希与差异文件另存清单。

复用上一轮冻结语料、101 个回放 URL 键（100 首加一首预热），64 KiB/16 ms chunk，DPI=1、200×395、HD 开、pixel/MV 关、Spectrum 设置不变。Body 仍是原 108×108 SMTC 缩略图，HD 不可用。回放原源 URL 98 个、SHA-256 94 个，绝对值不外推普通在线缓存命中。

每轮 fresh exe/cache/WebView profile，预热 60 秒；连续浮窗 100 首，每首至少 5 秒并等待绘制/HD；暂停播放器保持浮窗 300 秒。顺序为前1、后1、后2、前2、前3、后3。正式轮不附加 CDP/HeapProfiler/强制 GC；构建与 UI 诊断已结束。

首轮未锁定窗口在第 49 首首次绘制时宽度为 392（随后回到 200），已由尺寸断言中止，48 首数据整体作废，保留于 `runs/`。两侧统一设置 `lockFloatingWindow=true` 防止交互改变窗口，200×395/DPI1 不变；正式六轮重新从第 1 轮开始，仅取 `runs-locked/`。锁定是共享测量条件调整，用户常规应用设置不受影响。

预先采用如下回归警戒值：进程树/Renderer 连续峰值与恢复末分钟中位数不高于参考三轮中位数 5%；平均 CPU 不高于参考值 `max(10%, 0.2 个百分点)`；观察 P95、曲线与各轮波动。该值用于识别明显回归，不能把小幅波动解释为收益，也不替代画质和非法输入验收。若超出则不勾选 M3。

## 独立画质与生命周期诊断

使用原生 WebView2 的 renderer DPR emulation=1、1.5、2，CSS viewport=200×395；每个 DPR 前后各跑同样 100 首。记录实际 devicePixelRatio、Canvas backing、CSS 尺寸和实际显示截图，指针固定在视口外。保留浏览器跨源安全限制；参考图使用同样的 smoothing 配置。源字节比较与显示截图差异分别统计，检查细字/细线、错图/空白以及 Detached Canvas。每 25 首关闭重开的回归另列。

这覆盖 M3 的 `devicePixelRatio` 栅格逻辑及 WebView2 像素呈现，不等同于修改 Windows 显示器缩放；Native window scale-factor、跨屏命中与 Windows 全链路 DPI 不在本轮判定内。GPU 长期回稳仍不得由五分钟观察宣称。

## 工程验证与结果

当前：53 Rust 测试通过、1 硬件压力用例 ignored；新增合法 APNG 转单帧的实际数据回归通过；80 前端测试通过；Svelte 0 errors/0 warnings；生产前端与两侧 Release 构建成功；9 UI 回归通过。UI 初始导航受系统代理干扰，已证明本地直连 105 ms 并限定测试浏览器 `--no-proxy-server`，没有放宽断言或改变用户代理设置。

完整正式轮和独立诊断已完成，结果见 REPORT.md。候选 A 原始数据保存在 `dist/performance/m3-correction-2026-10-09/`，候选 B 在 `dist/performance/m3-correction-b-2026-10-09/`；旧轮数据保留，禁止混合批次。本阶段独立提交并停止。

诊断工具修正：Playwright 截图准备会复位外部设置的 DPR，旧 quality-dpr15/quality-dpr2 记录实际 DPR=1，整体排除。改为持有 CDP session 的 Page.captureScreenshot，每次截图前后断言 DPR；仅采纳 quality-fixed-dpr15/quality-fixed-dpr2。quality 汇总也断言 100 份实际 DPR 与请求值一致。DPR 1 原记录本就为 1，仍有效。

有效 DPR 2 的首个参考诊断在第 31 首 mocha 出现 imageReady=false，保留 quality-fixed-dpr2，整体不计为通过。缓存 328247 bytes，独立 sharp 完整解码成功，不能据此确定时序根因；保持原断言，全新 quality-fixed2-dpr2 重跑前后两侧。
