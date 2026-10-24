# 第9批：默认关闭的原生 region 静态实验适配器

本批交付默认关闭的 `experimental-native-region` Cargo feature、真实UI/Win32适配代码、离线故障测试及默认/实验两个标准Release构建。**不是产品首击修复，不批准默认接入，C1/A5仍 `[ ]`。** 所有原生调用仅编译，测试不创建HWND或调用SetWindowRgn；本批不运行任一产物，不启动浏览器/底板，不输入、不采集、不操作播放器、不提交推送。

实验被实际限制为独立窗口 `a5-region-static-lab` 的一次静态mask。它复制主岛已接受几何，但不运行产品页面，不克隆封面/阴影/动画。默认主岛仍用旧monitor，不在主岛上混合两种策略。第7批立即首击失败及4/3/1证据保持，不能把这个实验与原8次hover回归混作通过。

## 真实CSS/DOM与视觉边界

静态审查来源为未改的IslandSurface.svelte、App.svelte与islandMotion.ts；实际快照和SHA256在包内。

| 代码 | 实际约束与本实验不能证明的内容 |
|---|---|
| IslandSurface:66/512/750，boxShadow属性、transform/box-shadow transition | 属性默认none，本仓库未找到调用方显式传入boxShadow，不能虚构当前主岛已有某个外阴影。若使用外阴影，native硬region会截掉边界外绘制；不能扩region替它留阴影。 |
| :752 expanded-cover的0 8px 22px阴影及outline | 是真实CSS，但祖先surface已有overflow:hidden/clip-path。是否在具体状态碰到轮廓须按实际布局验证，不能从CSS断言全部都在region外或全部无损。 |
| :698/:746 moving-cover绝对定位+translate/scale，frame overflow:visible，surface overflow:hidden | 移动封面仍在surface内，不把它直接说成已确认的外部绘制；整数mask及延迟通知可能造成额外裁边/不同步，本实验没有封面，不能据此签署连续性。 |
| :765前后preview highlight的corner/handle、wash、spectrum阴影 | 属Studio预览条件装饰，不代表真实主岛一直显示。若延伸轮廓外会受硬裁剪；本实验不覆盖Studio，不删除这些视觉要求。 |
| :327、:430附近notifyRegionChange，App.applyIslandRegion | Svelte effect读取Tween当前size/radii/polygon/style并异步发IPC，extraRects当前通知为空；不是与浏览器呈现同一事务。没有native应用/paint ACK，不能把每帧SetWindowRgn直接接入产品。 |

本实验仅画冻结mask的Canvas轮廓，没有以上视觉内容，`visualVerified=false`、`productCompatible=false`。没有扩大命中范围，没有修改420/360/240ms Tween或Spring计划，也没有删除产品视觉代码来宣称兼容。严格保留外部视觉与原命中形状能否并存，仍需不同架构或独立设计，当前未知。

## 接口与隔离

`Cargo.toml`显式 `default=[]`；实验feature仅开启所需Windows绑定功能，不升级版本。默认构建保留两个IPC入口，但一律返回feature disabled错误，不能创建实验窗口。

实验构建：

- `experimental_region_control({action:"status"})`返回sourceAcceptedRevision及当前阶段。feature中accepted revision在与几何相同锁内更新，避免把提前fetch_max的desired revision当作几何身份。
- `experimental_region_open({revision,mode:"static-mask-lab"})`只接受当前非零accepted revision。复制不可变快照；已有未关闭实验时拒绝新请求，不更新可见窗口几何。主岛必须先隐藏，避免旧命中层与实验重叠干扰；创建和实际应用前各检查一次。代码不替用户隐藏/恢复主岛、不更改设置。
- 创建隐藏、不激活、不可调尺寸的独立about:blank WebView。该label不在现有capabilities列表中，页面不运行App、播放器、拖拽或动画代码；导航只允许about:blank。首次加载完成与物理定位/事件接线两道门都完成后，只排队一次UI工作。
- open回复只确认creation/enqueue，**不附一个可能已经过时的nativeApplied=false断言**。native-applied、ready和错误由status查询；status在UI线程复核可见性、静态frame/style、source仍隐藏以及输入门。没有poll timer或自动等待重试。
- `control`只支持status/hide/close。hide失效后不支持恢复，须安全关闭并重新创建新代次；创建失败且无窗口可由close重置，pending work不可被强行重用。UI排队的control也绑定代次，不能操作复用label的新窗口。

旧monitor/watchdog仅持有main WebviewWindow；实验label在install_island_cursor_passthrough处显式拒绝被旧机制接管。实验region/style的唯一owner是适配器，主岛旧策略不被替换。实验通过Tauri/tao自己的set_ignore_cursor_events(false)在UI线程同步更新缓存，再读native style拒绝TRANSPARENT/RTL；不直接SetWindowLong与tao缓存竞争。其他程序强行改style、绕过受支持接口操纵窗口，不属支持范围。

## 原生应用、所有权与线程

适配器的run_on_main_thread完成实际派发；执行时比较GetWindowThreadProcessId与GetCurrentThreadId。创建/窗口尺寸请求先排入UI，配置完成后才允许apply入队。enqueue成功不被当作native完成。

物理frame来自GetWindowRect、ClientToScreen与scale_factor：客户端DIP复用既有point_in_rounded_rect谓词，整数光标hotspot采样、右边界exclusive，包含原polygon和extra rect语义，不放大整数光标热区。第8批中心采样保留为参考，实验改用raster_hotspot：中心在分数边缘可能纳入旧整数热点判定之外的像素，新增分数边界回归及三个DPI逐点对照覆盖圆角/polygon/extra rect。与旧monitor在视口内的整数物理光标判定逐点一致；这不是产品AA/视觉等价证明。负屏幕坐标只用于取窗口位置，不嵌入局部mask。虽然转换计算了client→window offset，当前静态Canvas预览实际**拒绝非零offset和RTL**，不假装已支持所有宿主布局。

共享raster每draft最多100万像素、256 polygon点、64extra rect、16384行段；最坏约100万次包含判断，每判断最多遍历这些几何，GDI最多1+16384次Create和16384次Combine。它在隐藏状态的UI工作中执行，可能阻塞主UI，不是性能方案；没有测CPU/延迟收益，不能开展正式性能轮。

实际 `transfer(Api)` 负责创建destination、逐段创建/合并/删除row、最后SetWindowRgn；Win32与fake使用**同一策略函数**。Create/Combine/row删除失败均返回错误并尝试释放仍归本方destination；清理失败附带handle和unconfirmed错误。SetWindowRgn返回0时临时destination仍由本方删除；非零即系统接管，此后没有delete/query/复用该句柄。本批没实现回读，因而也没拿已接管句柄作GetWindowRgn目标。[Microsoft SetWindowRgn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowrgn)明确成功后的所有权及window坐标语义。

状态分开记录Enqueued、Applying、Applied、Invalid、Closed。真实转移成功后即使发生重入失效，也保留nativeApplied历史但拒绝ready；若窗口已销毁则不保留active applied标志。派发、分配、native应用、show失败不伪报就绪；不拿空region或全窗接收作为错误兜底。实验创建/应用失败可能保留隐藏窗口，错误可查，随后显式close处理。

## 生命周期与输入保护的实际接线

实验宿主安装局部SetWindowSubclass，UI线程内检查返回值；没有全局hook。WM_SHOWWINDOW隐藏使状态失效；WM_WINDOWPOSCHANGING对非Applied阻止SHOW，输入门忙时不执行HIDE；WM_ENTERSIZEMOVE/WM_CANCELMODE/宿主WM_CAPTURECHANGED/WM_DPICHANGED使状态失效但保留旧shape，不重放/ReleaseCapture；WM_NCDESTROY移除本subclass并关闭该代次，移除失败记录unconfirmed。Tauri窗口事件另接Destroyed、CloseRequested、Moved/Resized/ScaleFactorChanged；只有物理frame/DPI改变才失效，单纯局部shape不变的平移无需新mask。未在动画每帧hide/show。

首次show仅在SetWindowRgn成功、代次/阶段仍有效、frame复核后发生。JS eval只是Canvas绘制请求入队，没有browser paint ACK，**nativeReady不代表已呈现或视觉通过**。宿主事件/消息真实接线的存在由feature编译证明；顺序、消息flag的实际效果和subclass/GPU生命周期仍未现场验证。

[GetCapture](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcapture)只返回当前线程捕获，不当作全局无捕获。因此输入门保守要求foreground进程为本应用，检查五个物理鼠标键高位，并对宿主/子窗口线程及foreground使用[GetGUIThreadInfo](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getguithreadinfo)。查询失败、超过128个不同子线程、捕获或move/size均拒绝apply/hide/close；不丢弃已持有释放，要求释放后显式重试，无自动轮询。非本应用foreground、跨权限/其他桌面、触控/pen/复杂child capture或产品手势未获支持；不能据保守门签署完整输入生命周期。

宿主局部subclass并不拦截所有WebView内部子窗口消息；静态fixture没有可交互控件和拖拽路径，也不提供live reshape。重入的系统销毁、进程终止、窗口管理器行为、子窗口capture时序仍未知。[SetWindowSubclass](https://learn.microsoft.com/en-us/windows/win32/api/commctrl/nf-commctrl-setwindowsubclass)不允许跨线程安装；[WM_WINDOWPOSCHANGING](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-windowposchanging)允许改变部分flags，具体组合必须后续原生核验。主岛并发重新显示的契约由创建/apply/status检查拒绝，不宣称在任意外部改动间隙都有持续全局观察。

## 四类结果

| 类别 | 本批边界 |
|---|---|
| 已实现、已编译 | 默认关闭feature；真正HWND/UI派发、SetWindowRgn、成功移交/失败释放；lab与旧monitor分属不同HWND且拒绝旧接管；局部subclass及Tauri事件；静态/非零offset/restore等实际拒绝接口。 |
| 仅离线验证 | 共享ownership策略故障、stale工作、加载/定位门、输入保守决策、ready/关闭决策；GDI无窗口包含检查。无测试调用SetWindowRgn或创建窗口。 |
| 仍未知 | 原生首击、跨进程真实穿透、窗口/子窗口实际线程树、绘制呈现、API真实故障、OS移交生命周期/泄漏、WM flag效果、CPU/内存/延迟。 |
| 明确不支持 | 产品页面接入、hover/动画/封面、实时region变更、restore、拖拽/capture状态应用、非本应用foreground、非零offset/RTL、触控/pen、直接迁用原8次回归及正式性能验收。 |

## 检查、构建与回退

窗口相关默认Rust测试24/24，实验feature37/37（24共享几何/既有测试+13适配策略），媒体15/15。早期30项日志保留；最终以feature-tests-final-dpi.log/default-tests-hotspot.log为准。真实UI派发失败、SetWindowSubclass失败、Windows资源分配/删除失败没有现场注入；fake只验证同一策略如何响应这些API边界，不能证明OS路由或清理成功。

两种标准构建命令、退出码、binary/source/lock SHA256见authoritative-build-results.json、binary-hashes.json及日志。首轮experimental-release/在整数hotspot修正前完成，作为superseded产物保留但不选入审阅包；随后experimental-hotspot-release/亦在原生DPI保护补齐前编译，保留为superseded；最终实验以experimental-final-release/（整数hotspot＋WM_DPICHANGED失效）为准。默认构建之后只有feature/test-only两个子模块发生采样修正，它们在默认Release cfg中不编译；默认实际编译源码/配置未变。附文件字节中disabled分支/lab label的离线存在检查作为辅助，不能单靠字符串证明行为。新产物分别保存default-release/与experimental-final-release/，从未启动；默认版保留旧monitor，实验版也不会自动创建lab。未改前端，因此不启动Playwright、不用旧动画成绩替代本批。204项未改源码/配置/资源逐项与第8批快照比对；媒体和Tween代码、锁文件及旧8项保存产物保持。

回退仅移除本批Cargo feature、两个拒绝默认的IPC入口及lib.rs注册、feature-only accepted revision/label guard/模块声明、新adapter；prototype的共享可见性按增量还原；撤销计划本批文字。before/是实际进入时工作树，batch09.diff据此生成，不用git reset覆盖前面的C1/媒体/动画工作。保留本批审阅记录与旧ZIP，不提交推送。

## 后续拟议评估（没有本批现场授权）

先独立审阅静态实验设计和代码。若允许单独运行，先记录/人工安排source主岛隐藏及本应用foreground，冻结当前accepted几何；在静态lab核对native状态、真实canvas、HWND/PID/TID归属及可见样式。另行确定有限次同一静态点进入后立即down/up，以及轮廓外对独立进程的对称收件；不加停留或挪点伪装原首击通过。随后只核对已支持的hide拒绝恢复、DPI失效、关闭/代次和按下期间拒绝操作，所有原始失败保留、实际用户状态由获准现场方案恢复。

这不是原8次hover/产品测试的替代。若机制有效，仍须先解决外部视觉、动画提交同步、完整子窗口输入/生命周期和资源成本，才可能提出默认策略候选。C1/A5保持 `[ ]`，完成交付即停止等待独立验收，不自动进入现场。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
