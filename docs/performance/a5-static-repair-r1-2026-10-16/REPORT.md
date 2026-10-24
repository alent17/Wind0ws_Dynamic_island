# 第12批同批返修 R1：三项独立审阅问题

**返修候选待独立审阅，C1/A5仍[ ]。** 仅修复独立REVIEW.md的三项实际接线缺陷及必要验证/构建。没有应用/浏览器/底板启动、输入、截图、新现场、播放器操作、提交或推送。此前所有ZIP、失败日志、原始数据与私有备份保留。

## P1：持 gate 的同步 UI getter

旧候选构造worker在gate锁内调用hwnd，锁定Tauri的getter可能派发UI并等待，而UI的导航/标题/加载回调也需要gate，存在互等风险。改为实际共享`configure_document(gate, handle_provider)`先在无锁区调用provider取得HWND，再只持锁提交纯状态；导航和schedule仍在锁外。provider失败或期间回调使协议失效时不会配置/排队。

无GUI回归直接调用生产配置函数，provider通过try_lock取得同一gate并模拟初始回调；也覆盖provider失败和provider期间回调失效。其余新增路径逐处核对：schedule先hwnd再gate；标题回调先hwnd/url再gate；窗口事件先frame再OWNER；apply/control在所有UI getter/setter前释放OWNER；status与invalidate虽可能OWNER→gate嵌套，但锁内只有纯状态/JSON，不调UI。deadline纯状态读取后再失效。没有新增反向gate→OWNER锁次序。这是新候选风险修复，不解释第11批旧Enqueued根因。

## P1：受控启动产生的 Run 变化

撤回上一候选“Run只能仍等原值”的不完整策略。锁定lib.rs启动调用set_auto_start，settings.rs根据autoStart写`REG_SZ Isle = "当前EXE" --startup`并清理legacy，或关闭时删除两值。现在不可变原始快照与本次受控期望分开。

真实Node绑定传入已选定EXE/DLL hash、绝对路径、空启动参数；实际helper复核文件hash、绑定PID/start ticks和实际进程MainModule路径。取得同一应用首次get_settings的真实autoStart后，再发送startup-observed。helper检查锁定lib.rs/settings.rs源hash并构建共享纯策略计划，私有备份另存startup-plan.json；公开证据只含身份、hash和autoStart，不带原Run值。

共享`a5-static-startup.ps1`严格校验autoStart布尔、实例、EXE路径、所选文件hash、固定--startup格式、存在性、String/ExpandString类型与原快照hash。恢复前只接受精确原值或此已证明启动实现能产生的受控状态；Rust允许部分启动操作失败，因此精确原值/已知单值变化也可安全识别，任意未知命令/路径/类型仍拒绝。恢复每个值后实际回读，按name/hash产生持久journal ack，才标该值Completed。后续续做对Completed只接受已ack原值，不能重新接受实验路径；写成功但ack失败时，下一次读回原值只补ack，不重复写。未知状态在配置文件覆盖前也先拒绝。此恢复仍需原同会话身份、释放和idle门，不提供冷启动原生恢复。

fake注册表使用实际同一New/Assert/Restore函数，覆盖自启开启原路径→实验路径→精确原路径、关闭删除、未知类型/参数/legacy、选定hash/策略hash/路径/autoStart/参数错误、第二项写失败后续做、ack失败后续做和成功API但错误读回，共14组；没有读写真实用户Run。MainModule/实际GUI启动期望确认本批未运行；若证明缺失/权限拒绝则保持pending。

## P2：请求读取与清理执行预算

生产serveCleanupOnly接到并验证合法请求后关闭5秒读socket计时器。共享预算为读请求5秒、执行60秒、客户端70秒；不是把所有阶段继续绑在旧5秒闲置计时上。执行预算到期返回明确pending/cleanupStillRunning，既不取消后端也不放开busy，重复请求不得并行运行清理；后端真正settle后才释放互斥。实际后端原有日志/阶段ack仍保存。

真实断管或不完整回复转为A5_CLEANUP_OUTCOME_UNKNOWN，明确“清理可能仍在执行，检查同会话持久证据，不重放输入”，不再出现空JSON SyntaxError或误报恢复一定失败。最终结果若在断管后才完成，需要看持久证据/监督器退出记录；没有通过旧句柄冷恢复。

新增生产serve/request共享路径回归：真实等待5300ms完整取得成功结果；真实销毁socket后客户端得到unknown、第二合法请求被busy挡住；缩短测试执行预算时明确pending且原任务settle前仍互斥。未调整动画/现场采集超时。

## 验证与分发

Node37/37通过（含原34项与上述三项传输用例）；默认窗口Rust24/24、feature窗口38/38通过（新增真实配置provider无GUI回归）；共享fake注册表14组通过；C# CompileOnly通过，windowCreated/nativeMethodsInvoked=false。未扩大到UI/播放器或重复整套业务测试。旧默认HTTP测试偶发失败原因仍未定位，既有失败日志保留。

标准Tauri流程分别执行beforeBuildCommand npm run build及Release，默认与feature新目录均保存、未启动；已安装CLI由node直接运行，工具链使用实际stable bin，锁文件不改。新工具scope为v3-candidate，绑定本次实验EXE/DLL，PENDING_APPROVAL=false，旧v1/v2记录不能复用。

| 构建 | 文件 | 字节 | SHA-256 |
|---|---|---:|---|
| default-release | isle.exe | 23367168 | `2ADF6029772DBDCD0F71B5FC7F30E8A949E065731CEEB13682A7C2F3E71B1CBF` |
| default-release | isle_lib.dll | 137728 | `DA62AD3703B13E2E9F711ACD344A1BE7CF637409D3253E5D93E47C2D9F60380F` |
| experimental-release | isle.exe | 23420928 | `38102A0F66B93A4370155CCFD6F420BC8A3831646216B7076C67DB493F506E7A` |
| experimental-release | isle_lib.dll | 137728 | `CBD7290142D7BA3140814F74387EF0D4F7AA356A1BB22D2764526EE1264B737D` |

当前源、锁文件、依赖摘录、R1精确增量diff、返修前before、独立REVIEW、必要日志和新二进制hash均入修订包。分发分析器仍使用raw/，原161次Enqueued/0输入结果不变；旧包及失败保留。交付前实际解压全部manifest核验并在解压source运行Node37/37与共享注册表测试。没有私有配置/Run备份或cleanup token入包。

仍未现场验证WebView/data文档/CSP/加载顺序、真实自启应用/恢复、Canvas与OS投递或性能；旧Enqueued根因仍未知。默认仍是产品旧monitor，feature仅隔离静态lab，不强接主岛。Tween420/360/240、媒体、C2/I1/UI/Spring均不动，所有40条勾选行保持；发回发布聊天后停止，本报告不授权新现场。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
