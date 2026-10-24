# 真实Compact组件的原生形状隔离接入

日期：2026-10-16。用户“下一步”后执行，工作树为 m3-release-c。本轮不是产品首击修复完成。

## 为什么没有直接替换主岛

主岛的鼠标进入事件立即改变islandStore，IslandSurface的effect随即启动Hover tween。固定Compact掩码会裁剪增大的画面；当前cursor monitor与watchdog也会异步写穿透状态。因此直接把上一轮固定胶囊移到main，既不能保持动画，也不能保证只有一个原生状态owner。

本轮把真实IslandSurface放入独立宿主，锁定Compact、不接Hover/Toggle、不启动产品setup；以此先验证真实组件和共享几何，下一阶段再做动画前解除掩码的同步屏障与单owner切换。没有修改主岛事件或动画代码。

## 已完成实现

- Cargo feature `compact-native-probe` 默认关闭；依赖已有experimental-native-region。新example使用独立identifier/profile，唯一窗口label为compact-native-probe，唯一IPC为apply_compact；正常产品handler不注册该命令。
- `ui-tests/CompactNativeFixture.svelte`直接复用IslandSurface，使用暂停的演示媒体、无封面、无Spectrum、无动画；固定Compact、80长度、scale1，style/edge启动时固定。没有接真实播放器或Studio。
- 收到settled后只读取一次实际surface DOM位置，把当前组件的radii/polygon交给原生；不复制App的offset公式，不用手写胶囊代替真实组件。
- 原生复用normalized_interaction_region、raster_hotspot与gdi_region。只允许独立label、稳定Compact和严格schema；拒绝主岛、Hover/Expanded/Hidden、extraRects。单次claim拒绝后续重试/变更。
- 原生检查零client offset、borderless尺寸，使用实际scale与物理frame。安装成功才移交HRGN；失败沿用RAII清理。命令经UI队列执行，返回的是SetWindowRgn实际结果；成功后才显示宿主。
- 安装后位置/尺寸/DPI事件请求隐藏并记录失效；90秒请求退出。上述真实生命周期路径尚未现场验证，不能由代码接线声称通过。

## 验证结果

- `cargo test --manifest-path src-tauri/Cargo.toml --features compact-native-probe commands::window:: --lib`：41/41通过，含新增3项schema/host/状态边界测试，以及已有共享几何、三档DPI、headless GDI与旧lab策略测试。旧lab的fake transfer测试不等于新installer的真实SetWindowRgn验收。
- `cargo build --manifest-path src-tauri/Cargo.toml --features compact-native-probe --example compact_native_probe`：首次完整构建成功，有一个unused import警告；随后移除该import，最终检查结果补充在下方。
- `node node_modules/vite/bin/vite.js build --config ui-tests/compact-native-vite.config.ts`：成功，独立输出dist/performance/compact-native。主产品build目录不被此命令覆盖。
- 默认feature隔离与Svelte检查结果见文末。

补充：`cargo check --manifest-path src-tauri/Cargo.toml --no-default-features --lib`通过；`node node_modules/svelte-check/bin/svelte-check --tsconfig ./tsconfig.json`零错误、零警告。新实验模块/导出在默认编译中不存在，默认产品仍运行原monitor；Svelte命令按仓库tsconfig范围检查，独立fixture另有Vite编译结果。

## 现场没有执行

原定floating/top、edge/left各一次内部和一次外部点击，最多4次，无失败补点。独立Vite服务启动成功；随后“读取127.0.0.1页面并启动底板及Compact宿主”的exec命令被自动审批整体拒绝，仅返回`blocked by policy`，没有更具体理由。没有拆分、重试、换入口启动或以浏览器测试绕过拒绝。

因此本轮测试宿主启动0、测试点击0，nativeApplied/真实呈现/收件/生命周期全为未测；上一轮最小手写胶囊的4次点击结果不迁移为真实组件通过。服务已按记录PID23404与精确Vite配置命令行确认身份后关闭；进程查询未发现compact_native_probe或first-hit-minimal。

## 待验收的具体候选

源码入口为src-tauri/examples/compact_native_probe.rs。独立Vite配置ui-tests/compact-native-vite.config.ts使用127.0.0.1:1426。运行example需要全新输出目录，后跟固定style/edge；拒绝覆盖旧输出。测试前需独立底板就绪、页面ready回执、SetWindowRgn原生成功与实际画面一致，才选择该次截图中的内部/透明角坐标，不照抄上一轮320/160。

本轮候选不支持动态mode/style/edge、设备DPI变化、拖动、外阴影或真实封面；即使未来这4次通过，也只能证明稳定Compact的真实组件静态命中。工具会激活目标，不证明无激活/立即move-down首击。

下一阶段产品接入必须先完成两件事：在任何形变开始前等待原生清除掩码的实际确认；原生形状owner与旧monitor/watchdog互斥，迟到旧写入不能越过切换。不能仅加开关就把固定掩码用于生产动画。

未修改产品默认运行路径、业务设置、媒体状态、自启、Spring/I1/C2，未提交或推送。C1-P、A5-F、H1仍未验收。

## 用户“继续”后的单次再尝试

2026-10-16，收到用户直接“继续”后，在全新live-continued-01目录准备运行，未重建候选。启动独立Vite服务PID8928后，再次提交相同范围的完整命令（读取本地测试页、启动底板及floating/top候选）；自动审批再次整体拒绝，只返回blocked by policy，没有更具体理由。没有拆分命令、改启动入口或重放点击。

本次宿主启动0、点击0，未产生floating-top运行目录或收件日志。按PID及精确Vite命令行验证身份后关闭临时服务；cleanup.json记录serverStopped=true、testProcesses=0。默认产品路径和现有候选保持；没有新增工具或重复离线测试来冒充现场进展。当前阻断是自动审批，不是需要再次调试实验代码的运行失败。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
