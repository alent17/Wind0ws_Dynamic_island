# 待批准现场单：a5-static-lab-four-v1

**当前未批准，不执行。** 先由发布聊天审阅本批工具，再请求人类明确允许以下单次范围。`PENDING_APPROVAL.json`中的approved=false必须保留到真实允许到来；记录文件或`--live`参数本身不是权限来源。

## 构建及对象

使用第9批 `experimental-final-release`，不重建、不切版本：

- EXE SHA256：`4919DD50A41B47E183E2AEE3EEAFE29C73C052A4F6711D7FD425E72630FBC168`
- DLL SHA256：`1C455A8E14D83534787D6C06AB9D798D8948654118CCEA6E91E789CE75FE266E`

两文件在 `dist/performance/a5-region-adapter-2026-10-15/experimental-final-release/`。现场前重新核对字节hash与审阅工具manifest。

要求无已运行的Isle进程、当前输入/捕获空闲、至少两屏且index1为副屏；不满足就停，不接管或关闭用户已有Isle。用户期间不操作鼠标/触控/其他Isle设置，避免混入真实输入或并发写入。播放器不启动、不暂停、不切歌、不seek或调音量。

本轮新建一个测试Isle进程、其主岛、Studio、静态lab，以及独立辅助进程的一个no-activate底板。用户既有应用仅保存并最终恢复其前台身份，不移动/关闭。主岛与lab不并发显示。Studio在主屏工作区左上+32物理px，保持创建尺寸，不合适就停；不隐含已有前台控制窗口。

## 将来允许的前置动作

1. 辅助进程先记录原前台HWND和物理光标；在本地私有备份保存实际Roaming `com.isle-app.isle/settings.json`及`.bak`、HKCU Run的`Isle`和`Wind0wsDynamicIsland`两个值及类型/存在性。应用启动会同步自启值，这是必须提前保存的原因。不同目录/未知类型不猜测或改路径。
2. 启动且只启动上述已验hash的Isle，使用本次正常会话的本地CDP 9250；端口被占用就停。只连接这个WebView，不启动Chrome/另一个播放器或独立元数据进程。
3. 经现有show_studio_window接口创建Studio（该接口含正常set_focus），将它置于主屏上述位置，并实际读回rect、前台HWND/root/PID/TID和输入状态。Windows若不允许前台，前置失败就停；不按Alt/额外点击或反复抢焦点来绕门。
4. 读取实际设置；仅在需要时临时把monitorIndex改为1，不改动画/edge/hover/音频设置。观察主岛实际位于副屏；对已观察rect发一次原有sync_window_bounds相同值请求并核对实际结果。隐藏主岛并读回，随后获取非零accepted revision。
5. 创建仅覆盖预定宿主rect的专用底板，再请求一次静态lab。底板有WS_EX_NOACTIVATE/TOOLWINDOW、ShowWithoutActivation、WM_MOUSEACTIVATE返回MA_NOACTIVATE而不是ANDEAT，并禁用managed selectable。专用STA消息泵持续处理消息，不靠显式pump才投递。创建前/后的前台须一致；后续收件也记录当时foreground。
6. 原生查询确认lab实际rect等于预定请求、DPI与Canvas DPR一致、style不含穿透、source仍隐藏、控制前台仍属于本轮Studio，且状态为Applied/nativeReady。等待仅用于有上限的启动/前置就绪；一旦Invalid立即停，不重开lab凑通过。
7. 读取真实Canvas位图、viewport/style和alpha：位图须与实际物理frame逐维匹配，填色须为#15151c。两次rAF只提供渲染机会，不当作已呈现；另保存一次宿主区域的桌面合成截图，核对两点各5×5真实像素：内点为lab色，外点为底板#1d2e41。缺图/不一致/遮挡立即停，不反复截到成功。它不是显示器光子或帧时证明。

## 坐标一次冻结与输入上限

当前没有运行时frame，实际坐标记N/A，**不沿用旧-960等坐标**。选择规则现在固定：

| 项目 | 固定规则 |
|---|---|
| 内点 | 从Canvas整数物理像素中选5×5全不透明块，中心最接近宿主矩形中心；距离相同按y再x取第一个。 |
| 外点 | 在宿主rect内、边缘至少3px，按y/x扫描取首个5×5全透明块；不能改geometry制造空白。 |
| 起点 | `(宿主right+32, 宿主bottom+32)`；必须处于副屏工作区且不落控制窗口；不可用就停，不换方向。 |
| 顺序 | 内1 → 外1 → 内2 → 外2。始终复用同两点与同起点。 |
| 预算 | 最多4次成功左键down及对应up；每次先移起点，再移目标，分开发down/up。目标和起点均无故意停留预算。 |

图像、实际frame/DPI/HWND身份、请求尺寸/位置、revision、两点和起点在任何光标移动/down前写入frozen-plan.json并冻结到辅助进程。无合法点就停，不重选。每次down前再检查实际光标、frame/DPI/style/source隐藏、Studio前台、目标root与输入空闲；目标不是lab/底板就拒绝向其他窗口注入。

0预算不等于原合并立即click。记录SetPhysicalCursorPos结束→SendInput down开始、down→up的实际QPC间隔，以及命令/查询边界与原始Int64行；它包含协议和检查成本，不是OS投递时间。up后的250ms仅用于收齐观测，不在下一次down前加等待来修复命中。系统若将重复单击识别为双击，保留诊断并停，不改双击设置或补点击。

## 收件与status

两侧各保存down/up/click完整序列。原生侧来自WM_LBUTTONDOWN/UP与派生WinForms MouseClick，按共同family规范记录，**不谎称必然产生WM_POINTER消息或具有浏览器isTrusted**；可选WM_POINTER、WM_MOUSEACTIVATE、双击单列原始诊断。保存HWND/PID/TID、QPC/UTC、GetMessageTime32与extraInfo；本轮SendInput的down/up tag需匹配。

浏览器记录pointerdown/up、mousedown/up、click/dblclick，保留isTrusted、pointerType、client/screen坐标、event.timeStamp、timeOrigin和处理时刻。主判定用mouse pointerdown/up/click；双方要求按序且各恰一次，实际坐标匹配。原生trusted记unknown。两侧有主收件即冲突；缺失、拆分、重复、错侧都不能用后查WindowFromPoint/PID补成成功。

status会主动失效，不是纯观察。每对输入前后，先只读检查无按下/捕获且Studio前台成立，才查询；目标move到up之间不查询status。查询前后phase/error、查询原因和当时native状态独立保存。因status guard失效的记录不能归为唯一OS路由根因。就绪与发送之间仍存在外部变化竞态，不宣称原子观察。

## 立即停止与恢复

任一前置、遮挡、漂移、双收件/缺失/错侧、tag或激活异常、status invalid、日志不可用、API失败都会结束输入，不补齐剩余次数、不加DPI/拖拽/按住关闭/性能/旧hover矩阵。

仅释放本工具确实仍持有的按下。正常up后不重复up，未持有不发up；已观察到释放失败后不自动追加尝试。EOF仍由共用finally策略处理。若释放或物理空闲未确认，不移动光标/前台、不关闭窗口或硬杀；保留进程、输出和私有备份，报告cleanupPending，需另外处理，不能宣称恢复成功。输出管道/日志失败也不能跳过这个保留分支。

安全释放后按固定次序：应用前台安排用于lab正常close；确认lab销毁 → 关闭Studio并等其persist/destroy → 检查未混入其他设置变更，恢复本轮monitor设置和原主岛可见性并即时读回 → 对本轮owned PID窗口发WM_CLOSE并确认其退出（不硬杀） → 在没有Isle写入者时原样恢复两配置文件/两个自启值并回读 → 关闭底板且确认HWND消失，恢复原前台和光标并即时回读。正常关闭/前台被系统拒绝或并发设置变化时停止依赖恢复，保留未确认项，不用用户进程强退兜底。只回滚上述配置项，不承诺撤销应用正常生成的全部缓存/日志。

结束后报告实际执行数（可少于4）、每侧全部事件、所有失败/观测副作用和恢复结果。即使4次都完整，也只是静态lab证据，C1/A5与产品首击、视觉、长期内存仍未通过。本单须先独立审阅，再取得具体允许；本批到此仍未授权现场。
