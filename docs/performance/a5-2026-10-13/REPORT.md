# C1 / A5：全转换并发覆盖与 Apple Music 执行记录

日期：2026-10-13（北京时间）。执行工作树为任务单指定的 m3-release-c。**本批次全转换并发覆盖完成；Apple Music 有效 seek 未通过，C1、A5 保持 [ ]。** 没有修改/提交/推送业务候选，动画仍为420/360/240ms，Spring延期，C2/UI/架构阶段未开展。

## 范围、条件及构建

使用原保存 Release 文件，未重建：baseline EXE `00AE8D2837E18405515BCCB28EF1BA2479E68CCD2403D67B6822EB3E8B2FA336`、DLL `A245133252EAFF80D0141B892AC4D1A5CA019C26484CEA6BE11A5F7896AC0208`；candidate EXE `0F1D57468F384642D5C6A11530DBF8446AEB64DC6155A4B949B207BF2A1A9E4B`、DLL `754349D12F1CF194A920A8FCB00149AEE7B7A5F1132B3ADE8192DDD29E912FAC`。分析器重新读取实际字节核对两侧哈希。

真实播放器为 Apple Music，SMTC ID 为 AppleInc.AppleMusicWin_nzyj5cx40ttqa!App；当前测试曲目“飛ぶ時”/Vaundy，时长252秒。播放器本来未运行，启动后从现有播放列表A选择曲目，没有模拟会话。两版设置内容相同（映射键顺序无语义差异），floating/top，DPR=1，主 HWND 在 DISPLAY2。每轮初态/测量终态均暂停、33%；每轮清理恢复原音量20%及原暂停状态。初始进度baseline148秒、candidate149秒，由于seek未通过未伪称冻结同一进度。显示器/DPI/完整设置、初末状态均在矩阵JSON中。

本日 WebView2 为155.0.4283.45，两侧相同；与10月12日的154版本不同，不跨日比较收益。采集后QueryDisplayConfig观察两条活动路径200Hz，但GDI名称查询返回31，故未通过该工具强行关联VidPnSourceId与DISPLAY名称。DISPLAY2依据窗口探针，DPR/显示器信息依据页面和Tauri原始记录。帧分类使用各自主岛唯一时间相关swapchain，16/16样式窗口都有呈现；PresentMon没有直接HWND字段。

四个有效输入的探针窗口都是372×289，与主页面viewport一致；均采到初始化移至副屏原点的一次位置变化，尺寸保持一致。该观察不等于SetWindowPos调用计数，也不能排除两次窗口采样之间的短时变化。

## 执行与门禁

每次转换独立派发一条媒体toggle和一条音量写入，参数序列相同。控制对等待首个样式mutation后在同一页面派发，记录invoke派发/返回performance.now及UTC、命令参数、成功/失败、所属转换。下一次转换等待当前控制对返回；10秒超时停止准入，不追加排队。控制峰值在途为2。请求/响应包络不代表native worker在整个包络中执行。

门禁用同页面性能时钟要求每个转换都有成功媒体AND音量请求与first→last span有正长度相交，身份严格为1–16。不能用Node/CDP包络代替。helper单测5/5（含开始burst、失败、边界接触、零时长、错误身份和CDP误判）。成功或失败均保留原始文件；清理限定主页面，移除observer/事件处理器，等待控制请求结束，恢复播放和音量，并记录恢复结果。

成功采集命令均退出0：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-a5-animation-capture.ps1 -Build baseline -CdpPort 9231 -Seconds 40 -Suffix full-r1 -Mode combined -OutputRoot dist/performance/a5-full-2026-10-13 -MeasureResources
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-a5-animation-capture.ps1 -Build candidate -CdpPort 9232 -Seconds 40 -Suffix full-r1 -Mode combined -OutputRoot dist/performance/a5-full-2026-10-13 -MeasureResources
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-a5-animation-capture.ps1 -Build baseline -CdpPort 9231 -Seconds 30 -Suffix diag-r3 -Mode combined -OutputRoot dist/performance/a5-full-2026-10-13 -Instrument
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-a5-animation-capture.ps1 -Build candidate -CdpPort 9232 -Seconds 30 -Suffix diag-r3 -Mode combined -OutputRoot dist/performance/a5-full-2026-10-13 -Instrument
node --test scripts/a5-command-coverage.test.mjs
node scripts/analyze-a5-animation-capture.mjs combined full-r1 dist/performance/a5-full-2026-10-13
node scripts/analyze-a5-animation-capture.mjs combined diag-r3 dist/performance/a5-full-2026-10-13
node dist/performance/a5-full-2026-10-13/verify-replay.mjs
```

PresentMon的Process.ExitCode为N/A，不能写成0；完整start/stop日志、矩阵退出0、窗口探针及呈现记录越过最后confirmation共同确认未截断。QPC锚点、Present起点加MsUntilDisplayed、按显示区间交集保留跨界行，first/last/confirmation分开；原始CSV缺显示时间/间隔的行记录为不可分类，未补零。

### 全16次覆盖表（成功 / 与样式活动span相交毫秒）

| 转换 | baseline媒体 | baseline音量 | candidate媒体 | candidate音量 |
|---|---|---|---|---|
| 1 | ✓ / 215.80 | ✓ / 110.00 | ✓ / 224.60 | ✓ / 108.90 |
| 2 | ✓ / 174.20 | ✓ / 11.90 | ✓ / 172.60 | ✓ / 12.60 |
| 3 | ✓ / 173.50 | ✓ / 14.40 | ✓ / 183.10 | ✓ / 12.20 |
| 4 | ✓ / 172.20 | ✓ / 12.20 | ✓ / 171.10 | ✓ / 14.60 |
| 5 | ✓ / 171.10 | ✓ / 10.70 | ✓ / 172.60 | ✓ / 14.10 |
| 6 | ✓ / 169.80 | ✓ / 15.50 | ✓ / 171.80 | ✓ / 13.80 |
| 7 | ✓ / 175.50 | ✓ / 12.10 | ✓ / 168.70 | ✓ / 14.00 |
| 8 | ✓ / 172.70 | ✓ / 14.60 | ✓ / 171.20 | ✓ / 14.10 |
| 9 | ✓ / 173.20 | ✓ / 9.90 | ✓ / 171.60 | ✓ / 14.80 |
| 10 | ✓ / 170.60 | ✓ / 16.80 | ✓ / 171.60 | ✓ / 13.70 |
| 11 | ✓ / 168.50 | ✓ / 12.60 | ✓ / 173.30 | ✓ / 12.30 |
| 12 | ✓ / 172.80 | ✓ / 13.80 | ✓ / 173.20 | ✓ / 12.50 |
| 13 | ✓ / 174.50 | ✓ / 15.40 | ✓ / 174.90 | ✓ / 12.00 |
| 14 | ✓ / 172.00 | ✓ / 9.70 | ✓ / 170.40 | ✓ / 14.90 |
| 15 | ✓ / 171.70 | ✓ / 14.60 | ✓ / 169.10 | ✓ / 16.90 |
| 16 | ✓ / 173.50 | ✓ / 14.10 | ✓ / 173.50 | ✓ / 12.50 |

full-r1及有效diag-r3均16/16通过，各16条媒体/16条音量操作，控制失败0，结束暂停/33%，清理恢复暂停/20%，在途控制0。全IPC代理的峰值在途另记，不能与控制对的2混作同一指标。

## 无运行时IPC代理轮的呈现结果

full-r1有PresentMon/窗口探针/资源计数采集开销，但没有IPC传输/回调代理。

| 构建 | 完全在活动span的>50ms | 确认尾段>50ms | 跨边界>50ms | 最大活动重叠ms | 累计活动重叠ms | 完全在活动span P95 ms |
|---|---:|---:|---:|---:|---:|---:|
| baseline | 0 | 2 | 4 | 31.69 | 61.74 | 5.03 |
| candidate | 0 | 1 | 10 | 35.51 | 94.66 | 5.03 |

程序点击到首mutation的P50均0.40ms，最大baseline0.70ms、candidate1.10ms。不是物理鼠标输入到显示延迟。候选跨界原始间隔最差205.05ms，仍与活动span有重叠；不能因跨界就整条归为静止，也不能用0个完全包含长间隔宣称零卡顿/收益。style-mutation span是样式探针边界，不是CSS/合成器线程的完成契约。无ETW等待栈，未归因。

## IPC诊断轮：有代理，不能与性能轮混合

草案直接赋值invoke的代理未生效。diag-r1两侧IPC数据为空，排除IPC验收；加入硬门禁后的baseline diag-r2退出1并保留失败JSON。运行时descriptor证明Tauri invoke/bridge只读且不可配置。最终代理保持该只读API原样，通过可代理的custom-protocol fetch/WebView2传输及callback Map观察请求和完成，callback-ID去重重试；单请求smoke得到1条记录/0残留，再采有效diag-r3。

代理记录JSON参数及UTF-8字节，20,000条上限。diag-r3无丢失/无不支持payload/无残留在途；其额外JSON编码、Map及记录分配会干扰时间与内存，不能混入full-r1。只涵盖主页面发出的invoke，排除其他窗口、事件、响应、协议头和native worker时段。region参数字节不是全传输字节；传输开始戳不是invoke函数入口戳，控制覆盖仍以实际页面invoke性能戳为准。

| 转换 | B invoke数 | B region数 | B region参数字节 | B活动峰值在途 | C invoke数 | C region数 | C region参数字节 | C活动峰值在途 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 52 | 45 | 147941 | 14 | 87 | 81 | 266618 | 15 |
| 2 | 51 | 49 | 160739 | 4 | 75 | 73 | 239619 | 4 |
| 3 | 58 | 49 | 161030 | 6 | 91 | 85 | 279613 | 6 |
| 4 | 52 | 50 | 163964 | 4 | 78 | 73 | 239521 | 5 |
| 5 | 58 | 49 | 161213 | 6 | 91 | 85 | 279589 | 3 |
| 6 | 51 | 49 | 160719 | 3 | 75 | 73 | 239293 | 3 |
| 7 | 55 | 49 | 161141 | 5 | 91 | 85 | 279790 | 5 |
| 8 | 51 | 49 | 160594 | 3 | 77 | 73 | 239570 | 4 |
| 9 | 55 | 49 | 161255 | 5 | 91 | 85 | 279769 | 3 |
| 10 | 52 | 50 | 163995 | 3 | 75 | 73 | 239463 | 3 |
| 11 | 58 | 49 | 161227 | 5 | 91 | 85 | 279646 | 5 |
| 12 | 51 | 49 | 160706 | 3 | 77 | 73 | 239410 | 3 |
| 13 | 58 | 49 | 161227 | 5 | 92 | 86 | 283035 | 3 |
| 14 | 52 | 50 | 163957 | 3 | 75 | 73 | 239585 | 3 |
| 15 | 55 | 49 | 161085 | 5 | 91 | 85 | 279913 | 5 |
| 16 | 52 | 50 | 164136 | 3 | 77 | 73 | 239551 | 3 |

点击派发→确认窗口合计：region请求 784→1261，参数字节 2574929→4143985（约+60.94%）；全主页面invoke数861→1334。完整代理采集峰值在途14/15，结束0/0；此数不是native线程峰值。更长候选动画在本轮发送更多region，是需要保留的成本观测，不是优化收益。native SetWindowPos/resize实际调用次数N/A；窗口矩形采样变化不能替代调用计数。

## 资源可用范围

本日Windows 11 Insider Preview build26340、12逻辑核。full-r1资源各28个采样，探针本身有系统采集开销；基线启动晚，动画段0样本，候选仅5样本，**动画段CPU比较不可用**。下面是探针采样窗观测，不是真峰值或匹配动画成本验收，CPU/内存/GPU长期回稳仍未证实。

| 观测 | baseline | candidate |
|---|---:|---:|
| 采样窗 CPU 均值，机器总量% | 0.10 | 1.08 |
| 动画段 CPU 均值% | N/A：动画段0个样本 | 5.30：仅5个样本 |
| 进程树 Private Bytes 采样峰值 MiB | 343.99 | 363.52 |
| Renderer Private Bytes 采样峰值 MiB | 156.11 | 168.39 |
| GPU committed 采样峰值 MiB | 39.84 | 44.24 |
| 进程树线程采样峰值 | 256 | 254 |
| 进程树句柄采样峰值 | 3899 | 3866 |
| 末3个样本 Private Bytes 均值 MiB | 279.13 | 281.60 |

GPU计数器可读；单PID/engine利用率采样最大0%/1%，不能相加成整机GPU利用率，且可能错过短时动画。JS heap/Detached Canvas、实际SetWindowPos次数、native worker等待栈、长期GPU/内存恢复N/A。当前单轮成本数据不用于证明收益或勾选C1/A5。

## Apple Music真实功能证据

两次功能脚本均完成写证据（命令退出0），但结果accepted=false；退出0只表示脚本完成，不是功能验收通过。最终文件apple-functional-playing-seek.json：

- 播放状态true，时间线154→155秒；暂停后156秒稳定；恢复播放true，符合play/pause/时间线推进。
- 暂停时seek目标60秒返回成功，SMTC仍156秒。恢复播放后再seek60秒也返回成功，连续5秒采样157→161秒，未到目标。有效seek未通过；不把“返回成功”当作播放器功能成功。
- 非法action返回结构化code2001错误，前端invoke确实拒绝；错误传播通过此负例。
- 会话报告seek=false。不能由上述数据确定根因或断言真实播放器内部泄漏/阻塞；尚缺有效seek/错误反馈及完整其他播放器功能验收。

播放器测试前没有运行，测试新建的播放会话已暂停；每轮系统音量恢复原20%。实际SMTC进度无法可靠seek恢复，不伪称恢复原位置；测试播放器最后按标准关闭窗口，关闭状态另记environment-final.json。没有用户登录或购曲操作。

## 原始输入、独立复算与交付

所有原始文件在dist/performance/a5-full-2026-10-13，旧10月12日数据保留。分析JSON为combined-full-r1-analysis.json与combined-diag-r3-analysis.json；独立verify-replay.mjs不导入分析器/coverage helper，直接读取CSV、QPC/UTC锚点和命令/转换JSON，复算四侧区间计数、活动重叠、16/16同页覆盖及IPC参数字节/逐转换region数量。raw-recompute-audit.json记录4/4通过。

| 有效输入 | CSV原始行数 | SHA-256 |
|---|---:|---|
| full baseline | 1063 | `C2A726901DC85E910AEBE6AB59C8BD1810F63A7DCFF5521F565E2EAACF6C3C02` |
| full candidate | 1416 | `CD1CB4D9E3DC94DA7BD55648014CBCCA7C5814755588172EBE393353D4E029B6` |
| diag baseline | 1075 | `FBCB146EBD2984A9B4F325654358B14800240C8A7BECC23D411CF15E2BCA589D` |
| diag candidate | 1611 | `48E6AB504669977E25533B513FA48866EC38E00BD1BC51BE2E91A2001FF352D0` |

离线包含脚本、两份分析JSON、完整CSV/status/bounds/transition、资源JSON、Apple Music前后状态、错误及失败诊断记录。无应用二进制；回放明确使用录制的二进制哈希元数据，本机已另外校验实际保存EXE/DLL。Node即可回放，不安装依赖；实时采集才需要已有Playwright/PresentMon。

修改文件：scripts/benchmark-c1-matrix.mjs、a5-command-coverage.mjs、a5-command-coverage.test.mjs、run-a5-animation-capture.ps1、analyze-a5-animation-capture.mjs、a5-apple-music-validation.mjs；docs/OPTIMIZATION_PLAN.md及本目录报告/JSON。未改业务源文件。草案窗口选择、observer/listener cleanup、错误/在途恢复、页面时钟、唯一swapchain和完整尾段均加入本轮审阅/门禁。

本批次停在可审阅交付。全转换覆盖完成；有效seek、完整原生交互、稳定资源成本与长期目标继续待办，C1/A5均保持[ ]，不触发WPR管理员提权，不提交/推送业务候选。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
