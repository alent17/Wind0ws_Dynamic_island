# 第6批工具返修（2026-10-14）

两项返修已完成可审阅交付，仍待发布方独立验收；C1/A5 `[ ]`，首击原因未知。全程纯离线：未启EnableNativeInput、GUI/应用/播放器/底板、原生输入、采集、重建或业务修改；没有重试被自动审批拒绝的补启动命令。原26项审阅包与首轮版本保留，未覆盖。

## 按下所有权与清理

新增纯PowerShell策略模块a5-owned-press.ps1。实际helper调用同一模块的down/up/session-finally，不是测试另写一套清理：

- 仅后端返回1（确认本工具成功注入）才置Held=true；重复down拒绝。
- 正常up须持有所有权，成功即清Held；未持有不调用后端释放，不干预非本工具输入。
- EOF/stop/会话异常统一session finally，若仍Held则先尝试up。正常down/up后的finally不重复发送up。
- 命令级异常也调用同一Close策略并把releaseOnError写入错误响应。释放失败保留Held=true、ok=false、released=false和错误文本，不伪称清理成功。
- 释放审计在关底板/恢复前输出独立owned-input-cleanup JSON。只有release.ok时执行显式cursor/foreground恢复；释放失败跳过这两项，避免仍按下时移动鼠标造成拖动。仍结束底板生命周期，不宣称OS自身关闭窗口时前台绝不会变化。

9项fake-backend故障验证执行生产同一策略/会话finally：down→EOF、stop、抛异常，正常成对、释放返回失败、释放抛异常、未持有、down失败、未持有up。成功路径trace均inject:2→inject:4→restore-cursor→restore-foreground；未持有无inject:4，正常up只一次；失败释放trace保留持有且restore-skipped-owned-input，既记录失败又不恢复导航。见ownership-tests.json。这里是静态可达清理路径修正，未声称已现场发生卡键；live C#未编译运行，无法保证硬杀进程绕过finally时的系统输入清理。

## 双收件冲突

recipientVerdict保存plateDelta与pageClicks原始整数计数。两侧均>0返回ambiguous/conflict、conflict=true、uniqueRecipient=false；无侧unobserved且false；仅一侧分别bottom-plate/page且true。latePointPid保留为查询字段，永远不选择或覆盖实际收件，冲突不能作为唯一归属通过。异常负数/非整数计数拒绝，不归零。

JS共9项测试退出0（原7项+冲突/单侧/无侧）；策略故障9项退出0。默认helper无GUI dry-run仍提前返回；PS Parser/JS语法/diff检查通过。recipient-cases.json保留双侧、底板、页面和无侧样例。首次版本、返修前版本、修订版diff与包均可离线比较。

## 状态与边界

原第6批的8次单点0/50/250ms方案未执行，立即首击失败和后查时间限制保持，不能由本返修称已复现/修复原生bug。loaded runtime、accepted region保持N/A；现场协调器仍未实现、仍需独立审阅后另安排授权任务。焦点DIV方案及实际读数/视觉确认要求保持，不改产品焦点样式。

原包SHA256 42C4D5298786FB9EC0D7E117DEAFC3C0F3EAAD28C4D23F46090F21D158ECCE61仍一致。本批没有GUI/输入状态需要恢复，业务四源与两二进制字节保持。C1/A5不勾选，不提交推送，已停止在可审阅工具返修产物。

离线命令：node --test scripts/a5-native-plan.test.mjs；pwsh -NoProfile -File scripts/test-a5-owned-press.ps1；pwsh -NoProfile -File scripts/a5-native-input-v2.ps1（不得带EnableNativeInput）。


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
