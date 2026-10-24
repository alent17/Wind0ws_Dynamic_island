# 第11批：单次静态实验未通过，持久配置已恢复

**实际测试move/down/up均0；C1/A5保持[ ]。** 10月15日只启动一次已审实验版本，lab在原定10秒内持续Enqueued，按规则停止，没有重开或补点击。10月16日08:16北京时间，两配置文件原始字节及两个自启项恢复并即时回读一致，发布方也独立核验通过。历史前台/光标及原工具最终退出过程未证实，不能把持久恢复当完整环境恢复。

## 授权与版本

原授权为发布聊天01a13577-f007-77d1-aaee-227deac91734用户消息01a13faa-7db7-75c0-acc8-3f101e87d947，对request_user_input_async/call_bc9e5968f1164cdc8948c4337acbd21b第0项直接回答“允许执行这一次”；read_thread读取实际userMessage核对。10月16日执行聊天用户直接“继续”后完成遗留恢复。各新恢复方法先由发布方独立审具体源码/hash，再执行，审批JSON只是审计。

执行前17项已审文件及指定LIVE_PLAN/第10批ZIP/实验EXE-DLL实际字节一致。使用EXE4919DD50…/DLL1C455A8E…，未重建。原PENDING仍false、旧证据/ZIP保留。正常同次CDP记录Edg155.0.4283.45，无独立元数据启动、新Chrome、播放/暂停/seek或音量控制；业务/动画及420/360/240ms、C2/I1/Spring未改，无提交推送。

## 单次前置与限次核算

原Isle28944、helper/底板29784、协调器Node26488。Studio在主屏固定位置，main原在副屏monitorIndex1，协调器没有发update_settings。对实际rect(-1146,0)–(-774,289)同步相同bounds并核对后隐藏main，acceptedRevision1792069675781051。底板HWND461556，创建前后foreground均199444，只支持创建时未激活，未证明点击时no-activate。

open返回仅creation/enqueue。162次status含1次absent及161次Enqueued/nativeApplied=false/nativeReady=false；Enqueued首末约13:07:57.164Z–13:08:07.111Z。原定约10秒到期停止，未延长等待。随后explicit close返回Invalid属于明确关闭，不是此前OS路由失败证据。缺加载回调/UI执行时序或线程栈，持续Enqueued根因未确认。

| 项目 | 实际结果 |
|---|---|
| 单次会话/最大down预算 | 1次/4次 |
| 测试move/down/up及nativeApiCalls | 全部0 |
| 冻结坐标、Canvas读取、桌面截图 | 未到达，N/A |
| 浏览器监听、底板receipts导出 | 未到达，N/A |
| 完整输入路由/视觉/性能 | 未验收 |

完整已采evidence/native JSONL原样交付。没有伪造空事件证明OS未收件，没有用迟后WindowFromPoint补全收件；0次预算不授权第二次尝试。

## 恢复过程与保留的失败

原release-owned为ok=true/held=false/attempted=false，没有多发up；13:08:07.177Z物理idle=true。lab及Studio正常关闭并原生回读消失。设置恢复前deepEqual发现playerOrderIds新增Chrome.UserData.Profile2，停止依赖恢复。逐键语义比较唯一新增该ID，无删除；playerWeights仅对象键序不同。Studio onMount自动refreshSessions→merge/updatePreference→300ms保存及关闭persist与此相符，但缺内部caller/会话列表轨迹，来源仍为推论，不称用户改动。

| 恢复版本 | 实际边界 |
|---|---|
| 初版 | ISO被PowerShell转DateTime，身份比较误拒；0步骤。 |
| v2 | DateKind String及100ns修正后，私有保存当前配置、恢复source可见性、一次WM_CLOSE正常退出Isle；首个File.Replace因$null绑定为空backup路径失败。两目标文件/Run未替换。 |
| v3 | 旧数字PID存在门0步骤停止；当时具体进程未记录，后查已不存在，复用只是候选。 |
| v4 | 改为原实例代次判断，13项纯策略/CompileOnly独立通过；续办时旧helper/Node已不在，未执行，没有v4结果。 |
| 10月16日仅持久恢复 | 相同文件/Run frozen、无Isle writer，已审5097C7C3脚本一次完成6步骤及即时回读。没有窗口/输入方法。 |

$null绑定错误在非用户文件夹具实际复现target不变，改用[NullString]::Value实际替换通过，非仅fake。各版源码/diff、初始失败及审阅引用保留，不重写旧结果伪装一次成功。无硬杀、Ctrl+C广播、调试或远程句柄注入；原exec非TTY、无公开resumer。

## 持久恢复的实际回读

recovery-result-persistence.json：persistenceRestored=true，00:16:46.816Z–00:16:47.229Z；00:17:23Z另作只读核对一致，发布方独立实物核验通过。

| 项目 | 原状态与当前回读 |
|---|---|
| settings.json SHA256 | 5879A5A0A18BEF5759D48B6987A3DEF4A0233CD4AFE463D4B459BF7C26856BFF |
| settings.json.bak SHA256 | CA5C6D7BB89DBC00CA35C52805C47C42E41314D361FA6C6A2268AB192D675645 |
| Run/Isle | 原存在、String、原值回读相同，全文不输出。 |
| Run/Wind0wsDynamicIsland | 原缺失，回读仍缺失。 |
| 固定失败temp | 路径/hash/2659字节/创建窗匹配后单文件清理，回读不存在，非递归。 |
| Isle writer | 0 |

原三个进程现均不在，退出时间/原因和最终native清理未记录，journal无EOF/finally追加。未操作昨日过期HWND/前台/光标。最后历史native样本13:08:07.314Z前台/光标自然与原值相同，不是显式恢复证明，不能回溯补成完整通过。

原evidence SHA256：13BE98886FB994DBCD50D724A98E546A976539E010831D02263B2C1EE1A29082；journal：A651563E23EBEAEF7F8AD103AD77B4ECF10341351D04A58BEF0363D1D9D8519F，未改。原始配置和新增顺序项对应的当前两文件分别留本地private-backup/private-recovery-before，全文及私有Run值不入分发包。

## 交付与停止

包含公共原始证据、离线分析、恢复版本/失败/审阅/日志、当前源码、计划增量及manifest，不含私有配置或二进制副本。计划checkbox不变。静态机制和原产品首击仍未通过，C1/A5[ ]；视觉/成本/长期内存/GPU无新通过结论。报告与包发“任务发布”后停止，不自动推进下一阶段。

## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
