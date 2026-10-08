# 2026-10-08 M3 实施与验证

M3 的字节、解码、并发、输出、缓存和错误输入边界已实施，工程回归及实际原生命令验证通过。其他优化阶段未实施；M2 的同曲目迟到封面/重开空白失败仍保留。完整同条件 Release Benchmark C 尚未重跑，因此按优化计划的严格性能门槛，M3 暂不勾选。

## 最终预算

| 层 | 预算与处理 |
|---|---|
| 图片输入 | 文件、Base64、HTTP、SMTC 均限制为 12 MiB；文件 metadata 先拒绝大输入，限量 reader 再防止读取中增长 |
| SMTC | u64 大小在分配和转换前验证；0、超过预算、转换溢出均拒绝；要求 LoadAsync 实际加载数匹配；有效图转为正确 PNG MIME |
| 下载 | 4 个在途下载名额；响应逐 chunk 写临时文件，写入前检查累计字节；有/无 Content-Length 均限量；超限、失败、取消通过 RAII 删除部分文件 |
| MV | 保留单文件 128 MiB 上限及现有磁盘配额；不再整包保留于内存 |
| 解码 | 边长各 ≤8192、总像素 ≤16M；image reader allocation 64 MiB；先检查 header，之后仅一次完整解码；第三方 decoder 内部 scratch 的支持依格式，不把此数冒充整个进程峰值 |
| 图像工作 | 2 个活跃处理、最多 4 个等待；超出返回可控繁忙错误；加载、解码、像素化、颜色处理使用同一预算入口 |
| 缩略图与输出 | 保留比例，最大边 1280（640 logical px ×2）；PNG 输出写入器本身限制 12 MiB；普通/像素化路径共用一次解码，不重复完整 decode |
| Canvas | 根据当前可见尺寸 × DPR 分配；最长边 ≤1280；不放大小原图；CSS 布局、动画及封面 owner 规则不变 |
| 处理缓存 | 12 项与 8 MiB 保守 UTF-16 字节双上限；SHA-256 短内容 key；单项过大不驻留；最多 2 个前端指纹/处理工作 |
| 高清解析缓存 | 保留 64 项/8 MiB 上限，将 key/provider 字符串也计入保留字节；磁盘 total/image/video 配额保持 512/128/384 MiB |
| 旧缓存 | 公共读取入口检查图片字节和尺寸，超大旧图先缩略后返回；缓存发布通过完成文件 rename，元数据记账与发布序列化；同路径超大图也能替换并刷新真实 size |
| 崩溃残留 | 命名 partial 文件在正常失败时删除；启动及专用回退临时目录检查其 owner PID，清理已退出进程遗留文件；不删除活跃写入者文件 |
| pixel_size | 读取图片前验证 1–1280；0、u32::MAX 返回业务错误 3006；块遍历裁到图内，平均求和使用 u64 |

## 验证

- `npm test`：73 项通过，包含 1×/1.5×/2×DPI、缓存 byte LRU、短 key。
- `npm run check`：0 errors、0 warnings。
- `npm run test:ui`：9 项通过，包括真实 Canvas 卸载重建及动画回归。
- `npm run build`：通过。
- `cargo test --offline --locked --manifest-path src-tauri/Cargo.toml --lib -j 4`：50 项通过，1 项硬件压力测试默认 ignored；之前的 M1 验收轮已执行过 1000 次真实设备命令。
- `cargo check --offline --locked --manifest-path src-tauri/Cargo.toml -j 4`：通过。
- 独立 debug/custom-protocol 原生构建启动成功；真实 IPC 入口验证 0/极大 pixel_size、损坏图、超过 12 MiB、超过边长、超过像素数均为可控错误。正常 2560×1440 输入普通/像素化均输出 1280×720 PNG，之后 backend 仍可调用。见 [命令记录](native-command-results.json)。该轮不是 Release 性能基准。
- 无 Content-Length 的 chunked 超限响应返回 3004，未留下 partial 文件；本地 HTTP fixture 禁用环境 proxy 并有超时，避免测试受用户网络设置影响。
- 截断 PNG、超大 BMP header、有效超边长 PNG、SMTC u64::MAX、稀疏超大文件、缓存同路径大图迁移及正确 byte 记账回归通过。

Rust 验证使用已安装的 Windows SDK rc.exe，不安装新 SDK；为避免其他 dev 构建的 artifact 锁，验证产物放在忽略目录 `dist/m3-validation`。隔离 WebView 使用专用数据目录和 9229 本地调试端口，验证后已停止；测试应用曾按现有 autoStart=true 配置改写启动目标，已恢复普通 Release 启动路径。

## 数据与剩余验收

之前的三轮无 DevTools Release Idle、100 首真实播放器与 M1/M2 验收发生在 M3 修改之前，详见 [基线与验收报告](../acceptance-2026-10-08/REPORT.md)。不能将那些数据当成 M3 优化后结果，也不能用 debug synthetic fixture 宣称 Renderer/GPU 的性能 Delta。

本轮已证明异常输入不会走零步长 panic/无上限分配、实际命令输出受约束。仍需同条件 Release、相同 100 首语料，补下载/解码峰值、Renderer/GPU 与正常封面质量的量化对照；8 小时稳定性未完成。M2 的第 50 首后浮窗重开空白未在本轮修复，不能机械地把它的验收变成通过。

本轮对既有 PlayerControls 的并行本地改动未作修改，也未将其当作 M3 实施内容。验证结束时新脚本、报告和业务修改未提交或推送；2026-10-08 项目所有者要求提交到 GitHub，本轮将这些内容独立提交，原始大文件继续保留在本地 dist 目录。
