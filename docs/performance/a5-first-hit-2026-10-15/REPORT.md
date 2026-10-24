# 第8批：首击机制审查与隔离原型（2026-10-15）

本批交付是**窗口形状方案的设计与纯离线原型，候选待审**，没有把首击修复接入默认路径。任务单明确允许在需要较大窗口形状变更时先交付设计与隔离原型。本轮沿用该分支：当前命中切换确实依赖周期检查和UI派发；但安全消除这一机制涉及绘制裁剪及WebView子窗口，不能用未经验证的WM_NCHITTEST替代。生产路径仍有原问题，C1/A5继续 `[ ]`。

第7批的4 page / 3 board / 1 unobserved以及bit0x20关联继续有效；它们只支持时序候选，不证明唯一因果。spawn PID [omitted]、底板26164、点下窗口owner9440三种来源不混同。本批没有新现场数据，不启动应用、浏览器或底板，不输入、不控制播放器、不重试独立元数据动作，不提交推送。

## 当前调用链：源码能证明什么

锁文件及本地实际源码：Tauri2.10.3、tauri-runtime-wry2.10.1、tao0.34.6、wry0.54.4、webview2-com0.38.2。实际审阅文件及SHA256放入dependency-source/和dependency-source-hashes.json，未升级依赖。

1. `src-tauri/src/commands/window.rs::spawn_cursor_monitor`创建独立std线程，用GetCursorPos/GetWindowRect及scale_factor换算客户端DIP，读取共享几何，再决定ignore。检查后睡33/50/200ms；远近模式不是输入事件通知。即使派发完全即时，光标刚进入检查间隔仍存在恢复接收的等待窗口。此为源码机制判断，不是第7批真实投递因果的完整证明。
2. Tauri WebviewWindow委托Window；runtime-wry `src/lib.rs:234`的send_user_message在主线程直接处理，在其他线程通过event proxy排队。`2326`发送SetIgnoreCursorEvents，`3461`处理时调用tao并丢弃返回值。monitor调用成功只能证明这一层接受派发，不能作OS状态确认。
3. tao `windows/window.rs:566`通过thread_executor设置WindowFlags；同UI线程直接执行，否则PostMessage（event_loop.rs:522）。WindowState的IGNORE_CURSOR_EVENT映射到WS_EX_TRANSPARENT | WS_EX_LAYERED（window_state.rs:285）；apply_diff内部SetWindowLongW和部分SetWindowPos结果也没有形成可靠上层确认。当前last_ignored缓存的是请求状态；错误分支的fallback结果同样被忽略。没有在本批悄悄把该缓存说成已生效状态。
4. wry的`webview2/mod.rs:181`创建WRY_WEBVIEW容器，使用WS_CHILD | WS_CLIPCHILDREN，parent来自宿主HWND；`:375–410`创建windowed CoreWebView2Controller，而非composition controller。runtime-wry `:5031`附近选择build/build_as_child，两条Windows路径均有这个容器。具体WebView内部子窗口、线程/进程身份必须以后按HWND实测；不能把第7批9440映射为某具体模块。
5. 以上是锁定实现的线程/窗口关系审查，不是本次运行时TID或父子窗口枚举；本批无GUI权限，不补采这些字段。

## 几何与现有失效边界

单一几何来源是IslandSurface回传的geometry/radii/polygon/extraRects，经App.applyIslandRegion按host/surface offset平移，revision递增，再由set_island_interaction_region规范化保存。主polygon相对base，extra rect使用客户端绝对DIP；point_in_rounded_rect先判base/polygon，再并入圆角extra rect。原型直接复用此谓词，没有另造扩大热区的形状。原IPC成功目前表示内存接受，不代表native shape应用。

当前region为进程内OnceLock，enabled在规范化后固定true；重复几何跳过，旧revision拒绝，直至新IPC替换。没有把主窗口隐藏、DPI/布局版本、销毁/重建等做成统一的已应用几何身份。坐标由monitor当时读到的窗口位置/scale解释，布局和这些读数不是事务。App先缓存签名再异步发送，失败只记录warn。这些是若接入native region必须共同处理的相邻边界；本批未趁机重构或修正它们。

## API语义与方案取舍

- [Microsoft WM_NCHITTEST](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-nchittest)：HTTRANSPARENT继续寻找的是同线程底层窗口，并非任意其他应用；负屏幕坐标必须按有符号值解读。因此只在宿主返回HTTRANSPARENT不能保证跨进程底板收到原点击，且子窗口路径尚未验证。不安装这个补丁。
- [Microsoft layered windows](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#layered-windows)：带WS_EX_TRANSPARENT的layered窗口忽略其形状，把鼠标事件交给下面窗口。这与第7批样式关联相容，但不能证明当时具体路由。去掉周期切换前必须为透明区提供真实穿透机制，不能永久整窗接收。
- [Microsoft SetWindowRgn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowrgn)：region约束系统绘制，坐标是window而非client，RTL另有方向；成功后HRGN所有权转给系统，不能再删除。它不是独立于视觉的输入mask。基于持久形状实现光标无关命中是本批选择研究的机制，跨进程输入行为、WebView组合/阴影效果仍待原生验证，不由这份绘制API文档推导为已验证。
- [Microsoft WebView2 hosting](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/windowed-vs-visual-hosting)：windowed与visual hosting的输入职责不同，composition方案需宿主负责空间输入路由。迁移为此模式超出最小修复，不采用。
- 无窗口原型按[CreateRectRgn](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createrectrgn)检查空句柄；[CombineRgn](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-combinergn)允许目标与源相同，RGN_OR合并并检查ERROR；仍归本方的对象用[DeleteObject](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-deleteobject)删除，失败输出清理未确认。未测试分配/删除失败，不宣称资源验收完成。

直接原生改style也仍要发现光标进入，不能解决周期窗口，还会与tao缓存竞争。WM_MOUSEMOVE在穿透时不能保证回到本窗口；高频轮询只是缩短等待。全局低级鼠标hook需要另行证明回调开销、超时、线程和销毁、跨完整性级别及实际命中前的执行顺序；本批不安装或作为默认兜底。吞掉/重放点击、延长停留、放大region均不采用。

## 持久窗口形状：可审阅的接入设计，尚未实现

目标是在展示已接受几何时保持接收开关关闭ignore，用同一几何形成持久window region，让正常命中不等待光标监控。没有新轮询或hook。需要依次解决以下事务，不能把它们当成本批已覆盖：

1. 每个主岛宿主绑定generation；UI线程维护desired和native-applied两个独立快照，包含revision、DPI/client offset/physical bounds epoch。将几何、revision和空间信息绑定为不可变draft。窗口纯屏幕平移不写进局部mask；DPI/尺寸/client offset和样式改变必须重新准备。四边/floating都使用同一源几何。
2. 在UI线程由draft生成HRGN，使用CreateRectRgn/CombineRgn等构造独立临时对象，出错删除所有仍归本方的GDI对象；SetWindowRgn返回非零才移交HRGN且更新applied。返回零保留/核实旧applied，报失败并保持新版本未就绪，不能回传成功。读取GetWindowRgn作诊断有助复核，但不等于输入路径验收。
3. 检查并按唯一机制管理透明style：安装region与移除ignore必须由同一UI所有者协调，停止旧monitor和watchdog，不能让它们继续覆盖样式。tao缓存同步、SetWindowLong错误和失败回滚都需证明，不能一面直接改style一面保留Tauri旧缓存。原型没有实现这一步。
4. 首次显示/重建/恢复必须在有效region及style确认后揭示；隐藏、销毁使排队draft失效。动画中不能简单每帧hide/show等待ACK：现有CSS先变化后异步通知，需要确定视觉提交与region的对应策略，避免裁剪和闪烁。遮罩会裁掉阴影、抗锯齿边缘、动效装饰、特征条等，即使透明区点击正确也可能视觉回归；不扩大热区去保留阴影，需另作视觉/输入架构设计或放弃此机制。
5. 拖拽的capture/pointercancel/失焦/取消和销毁以宿主事件管理：不得在已持有按下过程中抛弃释放；DPI切换和布局变化使旧draft失效。边界在1×/1.5×/2×的像素中心采样与OS整数点语义可能差半像素，需要明确政策后再谈严格等价。RTL不能误用左上原点。模型的ready=false只是状态标志，没有真正隐藏或阻断输入。

接入以上内容属于窗口形状及视觉提交变更。本批明确停在隔离原型，不把“形状已应用所以应可点”当作现场事实。

## 本批原型代码与离线验证

`window_region_prototype.rs`由window.rs的`#[cfg(test)]`模块引入：生产Release完全不包含它，无新增feature、依赖或自动开关。原型实现客户端DIP→物理window行span（右边界exclusive），复用生产包含判定；每draft最大100万像素、16384段、256顶点/64extra rect，超过即返回错误而不截断/扩大。它是有限成本的参考实现，不是每帧性能候选。无窗口GDI构造用CreateRectRgn/CombineRgn形成真实HRGN，PtInRegion检查三个DPI的整数点及空region；OwnedRegion始终由本方持有并RAII删除，未调用SetWindowRgn，也未创建HWND。没有真实分配/Combine失败故障注入或泄漏统计，错误清理仅经源码审查。

Session以generation/epoch/revision拒绝旧draft，后端返回成功才更新last_applied；失败保留最后成功身份并ready=false，允许新revision重试。Backend只有fake实现，**仅有无窗口GDI构造/RAII删除与PtInRegion检查；无SetWindowRgn适配器、无向系统移交HRGN所有权或UI线程验证**。测试中的DPI/hide/drag名称仅表示调用invalidate的模型事件，不表示已接宿主生命周期回调。真实透明区穿透、renderer行为、拖拽释放和实际错误补偿尚未验证。

最终窗口相关Rust测试22/22通过（已有9项＋原型13项）；先期19项日志保留，最终以rust-window-final.log为准。标准Tauri `node node_modules/@tauri-apps/cli/tauri.js build --no-bundle -- --offline`退出0（Vite生产构建完成，Rust Release 3m48s）。原型只在cfg(test)编译，标准构建没有将测试代码接入默认路径。媒体Rust测试15/15通过，未调用真实播放器；受保护前端/媒体源码和Cargo/package锁文件SHA256与本批前一致。详细检查与源码保护结果见logs与verification.json。没有前端改动，因此不启动Playwright或浏览器，不用旧10/10充当本轮原生证据。保存Release只是标准构建检查，默认行为仍是旧monitor，不可拿它直接做“已安装首击候选”的性能/功能对照。

## 后续现场拟议清单（本批不执行）

先独立评审此设计，决定是否继续形状方案；若不能保留视觉/生命周期正确性，明确换设计，不能带模型包直接启现场。接入候选获准后，另申请固定次数的原生回归：保留第7批8个相同点/预算及实际QPC，不改点不加等待；双方记录对称down/up/click及按下前后style、native-applied geometry revision、父子HWND/TID/PID和实际映像来源；透明区用独立进程收件验证，双侧/无侧都保留。然后单独设计四边、隐藏/恢复、拖拽取消、1×/1.5×/2×跨屏及动画中首击，明确停止条件和状态恢复，不把它自动加到旧8次授权。

原生正确性通过后才做相同C1/媒体实现的匹配Release性能、IPC/region资源成本及长期回稳。Tween420/360/240及第2批媒体行为保持，Spring继续延期，C2/I1和其他UI待办不推进。

## 审阅与回退

审阅包包含本批前快照、增量diff、测试原型、当前源码快照、依赖源码/锁定hash、标准构建日志、新目录二进制及manifest。before中的window.rs与计划是本轮进入时的实际工作树，不能用git reset回退此前未提交工作。撤销本批时只撤销window.rs的5行test模块、移除本批原型并按增量撤销计划新增文字；不要恢复其他业务文件。已有保存Release、现场原始数据与旧ZIP未覆盖。整体验收不通过，本批完成即停止等独立审阅。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
