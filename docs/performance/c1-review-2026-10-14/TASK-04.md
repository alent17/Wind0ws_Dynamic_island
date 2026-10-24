# 第 4 批：旧 C1 95/55 ms 证据复核

执行聊天：`01a11622-f9c1-7900-b656-d0d9b04739ce`。

工作树：`E:/CodexData/.codex/worktrees/m3-release-c/Wind0ws_Dynamic_island`。

第 3 批已独立验收通过：118 项 manifest、门禁14/14、回放4/4；验收方另从 JSON/JSONL 复算，双方分别245对PID/start-time CPU计数链接，窗内6/8份有效增量、3.875/5.484375秒、跨界0.90625/1.03125秒均一致。资源链路子任务通过，不代表性能收益或第2批新Release验收，C1/A5保持 `[ ]`。

## 本批目标与边界

只读复核旧 C1/M6 r4 原始证据，修正历史归属和验收清单。**不新采集、不重建、不改业务或动画、不重复资源轮、不开展C2/I1/UI/Spring，不提交推送。** 不再把旧样本笼统留成“需要进一步归因”，而要明确哪些事实能重算、哪些字段根本没有记录、旧分类是否成立及最小剩余验证。

发布方已确认原始输入存在于 `dist/performance/c1-2026-10-12/`：

- `matrix-analysis-user-r4.json`。
- `matrix-{m6,c1}-{animation-only,controls-only,combined}-user-r4.json` 及对应progress JSONL。
- `matrix-presentmon-{m6,c1}-user-r4.csv`、status、command、log。
- `matrix-bounds-{m6,c1}-user-r4.csv`。

重要事实：旧 PM CSV 有 `TimeInDateTime` / `CPUStartDateTime`，**没有逐帧 TimeInQPC**；窗口探针有 QPC/UTC。不能把“探针有QPC”写成“已对旧帧完成QPC对齐”。C1 CSV首行TimeInDateTime约21:18:51，而对应status及探针UTC约05:18:51；这是需要证明的时钟映射问题，不能未经来源论证就减16小时。不要用今天的新A5样本替代旧C1样本。

## 交付要求

1. 校验源文件哈希，沿原分析器定位旧95.00/54.99ms对应的**完整原始行、行号、PID/swapchain、动作序号**；重建旧分类为何称其在动画内。保留旧JSON和原始CSV，不原地改写历史输入。
2. 核对旧时间换算、MsUntilDisplayed、区间相交筛选，以及旧first/last/100ms确认的实际记录情况。只有真实保存的字段才可称精确边界；不能把confirmation当last mutation，不能推算一个不存在的精确last-change。
3. 根据确凿时钟锚点/捕获元数据决定能否无歧义映射旧显示时刻。若做候选映射或减去100ms等敏感性分析，明确它是条件分析，列假设与误差；数据不足则归属标“无法确认”，禁止通过硬编码偏移变成确证。
4. 分别给出95ms、55ms能够确定/不能确定的事实，保留显示间隔的真实数值，但区分“间隔存在”“与活动相交”“完全在实际活动内”“线程阻塞根因”。没有线程栈不能用MsCPUBusy/低GPU时间归因。
5. 修正 `docs/performance/c1-2026-10-12/REPORT.md` 中仍写 genuine presentation gap / fully inside actual animation 的**当前结论**，旧统计可以明确标为已被复核取代的历史结果。同步计划 C1 正文/摘要/状态表；不要只追加一段而让当前正文继续互相矛盾。已验收的Apple能力/错误路径和资源链路状态与“有效seek N/A”“新构建性能未验收”分开，避免仍称另一播放器完全未测。
6. 输出简短 C1/A5 剩余验收表：每项是确认缺陷、未知证据、真实不支持能力还是后续测试，给出最小下一步。不要把Apple本身seek=false当作必须修成支持的产品缺陷；不要未经用户授权触发WPR提权。C1/A5仍 `[ ]`，不自行降低整体门禁。
7. 交付脚本/复算JSON、两条样本精确源行及对应输入、修正文档diff、可离线复核包。纯Node或现有运行时即可复算，不新增网络依赖。完成后停在可审阅结果，我会读取并独立验收；无需为了此任务向其他聊天发送消息。

旧证据不足本身是本轮可交付结论，不是让本轮扩展为无限补采的理由。下一次必要的有界新采集或原生交互矩阵由发布任务聊天在验收后单独安排。
