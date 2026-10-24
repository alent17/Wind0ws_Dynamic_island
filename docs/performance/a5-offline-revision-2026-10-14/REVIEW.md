# 第 6 批独立审阅：工具返修

日期：2026-10-14（北京时间）。

**离线证据、默认dry-run与原有测试通过；现场工具准备暂不通过，退回两项修正。C1/A5仍 `[ ]`，未授权现场执行。**

## 已通过复核

- ZIP 257,424字节，SHA256 `42C4D5298786FB9EC0D7E117DEAFC3C0F3EAAD28C4D23F46090F21D158ECCE61`；26项manifest文件长度/哈希全部一致。
- 独立解压重跑7/7测试及分析器，退出0；审阅默认guard后，仅运行不带EnableNativeInput的helper，返回guiCreated=false、inputInjected=false。未走live分支。
- 另写不导入交付分析器的审计，从原始first.json用整数纳秒算术核对五条handler至后查差45.8281/50.4479/54.6019/54.3352/53.4941ms，7项输入hash一致。
- 0/50/250ms方案共8次，均标记未执行；已正确说明原floating成功不是严格同点停留对照、后查询不能覆盖实际收件、后端accepted-region和loaded runtime未知。

## 必须修正后再审

1. **press-down成功后退出的清理路径缺少释放。** `scripts/a5-native-input-v2.ps1:64`将down拆为独立命令；`:84`的finally只关底板、恢复cursor/foreground。成功down后若stdin EOF、stop或协调器异常导致没有正常up，就没有对应释放。这里是静态可达路径，并未声称已现场出现按钮卡住。请跟踪本工具成功注入且尚未匹配释放的按下，在异常/EOF/stop清理中先释放再恢复cursor，且不误释放工具没有持有的用户输入。释放失败应明确记录，不能伪称已清理。
2. **冲突收件被静默选成底板。** `scripts/a5-native-plan.mjs:22` 对plateDelta=1、pageClicks=1返回bottom-plate。两侧均有收件可能是时窗归属、残留事件或观测冲突，不能作为唯一收件结论。分别保留原始计数，标ambiguous/conflict并阻止此条用于唯一归属通过。只有一侧有收件时正常分类，无收件仍unobserved；后查PID不得决定收件人。

以上返修仅限测试工具和相应离线测试；保留原包和首轮版本，生成修订版diff/包。测试应覆盖真实清理策略的down→EOF/stop/错误、正常down/up不重复释放、释放失败和未持有不释放，以及双收件/单收件/无收件；不要只增加源码字符串断言冒充故障路径验证。

继续禁止GUI/应用启动、真实输入、采集、重建或业务修改。原自动审批拒绝的补启动动作不重试。现场C#仍未编译运行，修订工具不等于原生bug已复现或修复。

独立审计：`E:/Wind0ws_Dynamic_island/dist/performance/a5-offline-independent-review-2026-10-14/acceptance-audit.json`。
