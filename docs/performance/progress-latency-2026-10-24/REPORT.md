# 网易云进度读取延迟修正（2026-10-24）

## 改动

- 成功读取后的采样间隔：播放中 500 → 200 ms，暂停中 2,000 → 500 ms。
- 失败不再固定等待 30 秒：首次 250 ms，连续失败依次等待 500/1,000/2,000/4,000 ms，最高 5 秒；成功或明确重置后清除失败计数。
- 周期进度读取单次总期限为 1 秒；独立设置检查和 seek 的既有期限保留。
- 本机 TCP 连接设置 `TCP_NODELAY`，避免小型 HTTP/WebSocket 写入等待前一包确认。
- 仍为单请求在途，保留曲目/播放实例、native revision 和 seek generation 校验；计时估算模式不启动周期读取。
- 播放/暂停、上一首、下一首的 SMTC 路由保留。

## 验证

- `cargo test --locked --lib -- --test-threads=1`：145 项通过，1 项忽略。
- 新测试覆盖渐进退避、成功后的恢复、调度期限，以及本机 socket 的 nodelay 配置。
- 现有只读 SMTC/CDP 识别工具连续检测耗时约 123/50/50/51 ms，包含进程启动和 SMTC 枚举，不能视为纯 CDP 请求耗时或优化前后对照。
- 使用扩展的只读 C2 生产监听器探针：`c2-media-probe.exe --seconds=8 --use-saved-settings`。读取本机保存的会话映射，但在探针内禁用连接自动恢复，不重启播放器、不操作播放、不写设置。
- 首轮 8 秒未枚举到媒体会话，成功采样数为零，排除该轮，不用其证明采样频率。
- 随后独立识别工具确认实际网易云会话及进度可读，再运行稳定播放的 8 秒观察：`elapsed_ms=8002 progress_started=37 progress_read_ok=37 progress_published=37 progress_read_timeout=0 progress_read_identity_mismatch=0 progress_read_no_page=0 progress_read_other_error=0 progress_inflight_busy=0 snapshot_ok=8`。约每秒 4.6 次成功采样发布，普通 SMTC 快照仍约每秒一次。
- 当前运行的开发程序构建时间晚于上述源文件修改时间，进程响应正常。

这是一轮短时后端采样观察，没有测量点击或外部拖拽到屏幕显示的端到端时延，也没有完成 CPU/内存/功耗对照。200/500 ms 是调度间隔，需另加实际播放器响应和显示刷新时间；更高读取频率不代表资源消耗下降。
