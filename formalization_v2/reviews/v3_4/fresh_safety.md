# v3.4 冻结快照：独立安全审查

## Summary

**Final Verdict:** 快照中的完整结论并非逐字成立：`prop:global-prefix` 的“相同 Espresso 前缀产生相同完整 interpreter state”子结论存在一个 **Critical Error**；但这不是 DATA 分叉反例，独立检查未发现 phase-independent closure、退休期间同一 session 的区域提交安全、或全局 DATA prefix preservation 在其明确条件下的反例。

**List of Findings:**

- **F1 — Critical Error，范围限于完整状态相等。** Location: 主文件 2559–2560 行：“Replaying the same complete Espresso prefix yields the same interpreter state.” 主文件 1458–1460 行将 pending controls 和 source-verification checkpoints 纳入该 state；2567–2573 行只证明相同 applicable controls 产生相同 prelude，未证明未来尚未生效的 L1 输入队列或获取游标相同。下面给出遵守冻结假设的直接反例。此错误不推翻 DATA prefix-comparability、已接受 DATA 的保存或 closed-version cutoff。

未另行发现需要归类为 **Justification Gap** 的退休证明缺口。此判定严格保留文中已明示的 baseline regional validation / DATA-DRAIN soundness，以及 ReadyBundle 的完整消息处理/原生状态采用接口条件；这些不是本次审查已经证明的具体实现事实。审查没有把局部安全推成 liveness。

### 快照与范围

仅阅读指定 snapshot 内六个 `.tex` 文件及 manifest，另读取 proof-verifier skill；未读取其他 reviewer 或 preflight。未改动协议文件。所有 manifest SHA-256 均逐项重算一致：

| 文件（相对 snapshot） | SHA-256 |
| --- | --- |
| `sync_hotstuff_rr_part2_rotation_v3_4.tex` | `3f614539cb58be30acc44742163ec76e871107fa723bd1f4a766057ad4627bf0` |
| `sections/rotation_v3_4_retirement.tex` | `aca904ad6cf8303a541a9265e2793a9f3b2764381b5ae8f6c1f5708d1899110a` |
| `sections/rotation_v3_4_ready_bundle.tex` | `facc64cfdb30cdc9b7166c4a7332120dfb9f5ba92365602244e9930ac238597b` |
| `sync_hotstuff_rr_part1_core.tex` | `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3` |
| `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` |
| `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |

主文件从 `\section{Model and assumptions}` 到下一 section 之前的完整字符串，与冻结的 `model_and_assumptions.tex` **完全相等**，不是只做空白归一化比较。下文简称主文件为 **M**、retirement 为 **R**、ReadyBundle 为 **B**、Part I 为 **I**；行号均指冻结快照。

### 依赖图

1. 冻结同步/权重模型（M 347–363、377–389）及完整历史验证（M 888–943）
   → 正确完整提案可供每个成员按历史 snapshot 验证，正确证书至少含一个正确签名者。
2. Closed 的持久存储、完整源证据和全阶段控制 owner（R 1–22、43–120）
   → suppression 时完整 close relay（R 171–194）
   → `c <= s + Delta`，或 early window 内无关闭抑制。
3. 完整 close relay + 原计时器期限 + fresh-action gate
   → `c_v <= c + Delta`、最后 timer `<= c + 3 Delta`、该 timer 的 FINAL `<= c + 4 Delta`（R 196–236）。
4. Ready/Enter 条件接口（B 60–146）+ 完整 quit proof 转发 + 原 recovery wait
   → 未被合法关闭/完成的成员可处理给定 proposal 的 vote window（R 255–292）。
5. 存活 direct timer + Quit/Completion 及时广播
   → 首个相关 Quit/Completion 严格晚于 `s + Delta`（R 262–274）。
6. 2、4、5
   → early case 的 protecting QC；或 late case 的关闭先于任何后续 recovery entry（R 294–326）。
7. 完整签名历史 + evidence-first + 6 + baseline rank/parent rules
   → direct commitments 可比较；FINAL 的 timer provenance
   → receipt/Completion commitments 继承同一受保护前缀（R 328–362）。
8. 认证公共历史 + 完整 applicable controls + 确定性 prelude + 原子连续 EXPORT
   → 全局 `G` 可比较且不回退；所有旧版本新 DATA 被拒绝（M 2554–2592）。
   **该支不能再推出不同节点的未来 L1 pending 队列或来源游标完全相等，见 F1。**

## Detailed Verification Log

### 1. 模型、签名门限与完整历史

> M 349–353：“`F_i < W_i/2` … every regional certificate needs … weight greater than `F_i`.”

门限推理成立：Byzantine 总权重至多 `F`，故任何 `>F` 证书包含正确签名者；正确总权重至少 `W-F>F`，故 all-correct matching votes 足以形成 QC。整个 session 固定 snapshot，后续 OFF、slashing 或 AUTH 的变化不能重算旧证书权重（M 355–360）。

> M 919–924：“Every relevant ancestor includes its signed proposal and block data … Correct senders send complete validation bundles.”

该规则覆盖真实转发，而非只约束最初 leader：Vote 明确在实际投票时 forward `P and its bundle`（M 1108–1113）。Admission 验证包含初始 boundary、recovery entry-proof chain、早期 terminal QCs（M 888–918）；另有 pair 的授权 boundary 检查（M 791–792）。因此关闭证明需要的 fixed membership/history 不是靠旧 worker 的隐含 cache。引用只有在 receiver 能于原延迟界内解析时才允许；R 73–83 又明确要求正确提案携带或及时解析历史 snapshot context。这个消息规则是结论的重要协议前提；只有 block/QC hash 的实现不满足它。

Evidence-first 在丢弃重复、无效或不完整 wrapper 之前比较所有已保留签名材料（M 926–943、1044–1059）。因而历史中已投票过、已间接提交或早期 term 的 proposal 不能以 dedup 为由隐藏 equivocation。

### 2. Closed、完整 close 时间及所有 lifecycle 路径

> R 10–13：“This branch-wide record immediately prohibits … It is separate … from the later per-session complete-close event.”

`Closed[beta]` 立即关入口和 fresh-action gate；`c_v` 只在该 session 的完整 RequestClose 处理与 multicast 时产生（R 57–72）。未知 snapshot 时不捏造 `c_v`，更不允许从较早 Closed 时间启动释放倒计时（R 123–130）。这一分离没有把已停票成员排除在证明之外，因为未知上下文路径保留后续补全责任。

| 被攻击路径 | 快照中的防线 | 判定 |
| --- | --- | --- |
| Waiting 已知 snapshot，Force 先到 | R 57–68：立即完整 RequestClose 和 multicast | 不存在静默退出路径 |
| Waiting 尚未知 snapshot，Force 先到 | R 69–83：保存完整 closure bytes 和 pending relay，Ready/proposal 到来先补 context 并回复 | 不靠 worker 或未来 fetch promise |
| Ready 与 Enter 同时遇到 closure | M 1977–1985、R 117–119：序列化再查门，message/evidence 优先 | 不能抢在同时 closure 后进入 |
| Completed | R 15–21、103–120、158–167：保留 manager control record、receipt 和 closure relay | Completion 只停 core，不停控制传播 |
| Terminated / archive / 无 worker | R 73–83、139–156；M 1621–1630：先控制回复再 tombstone/duplicate 返回 | late packet 不会被静默丢弃 |
| 重建 worker、restart、延迟 callback | R 84–91、115–119：先恢复 Closed/close time/pending owner；M 1763–1767 | 不能重置 entry 或绕过门 |

R 7–9 和 149–153 特别要求接收 owner 已持有所需字节和 receive route 后才能释放旧 owner；尚未完成的远程下载不算 ownership transfer。这排除了“逻辑上有人负责、实际上证据暂时不可发送”的空窗。

### 3. No-silent-closure 命题

> R 173–180：“suppressed … by `s+Delta` … implies `c <= s+Delta`.”

核验分两种真实事件顺序。若 Closed 先于 proposal，完整 proposal 在 `s+Delta` 内带来历史 snapshot，接收处理先补全 close bundle 并 multicast；若 proposal 先于 Closed，snapshot 已知，closure 自己的处理立即 multicast。两种情况都得到某正确成员的 `c_v <= s+Delta`。不需要这个成员曾 Enter，也不需要恢复可执行应用状态才能传播公共关闭证据。

拒收 ReadyBundle、重复 packet、Terminated/tombstone、restart 均被 R 73–83 明确纳入同一个先行处理器。故这不是“仅 Active 成员最早关闭”的错误量词。尚未识别 fixed snapshot 的早期 Closed 不能单独启动 drain clock，但只要它实际抑制该 vote window，完整转发便给出相应 `c` 上界。

### 4. Drain deadline 与 release

> R 208–220：“`c <= c_v <= c+Delta` … expiry … at most `c+3Delta`.”

首个完整 bundle 覆盖 snapshot 的所有正确成员，包括其 manager/archive；因此每个成员至迟 `c+Delta` 完成 close。所有正确新 vote/timer 被门阻断，既有 timer 的 vote time 满足 `s<=c_v<=c+Delta`，原 expiry `s+2Delta<=c+3Delta`。实际产生的 FINAL 再加一个区域消息界，至迟 `c+4Delta`。Quit/Completion 只能取消，不能延期或重启 timer。等时事件由 evidence-first 覆盖。

局部阈值 `d_v=c_v+4Delta>=c+4Delta` 保守，但不保证后续 receipt relay、Byzantine signatures、存储转移或 publication 已完成。R 130–156 要求计时器全部执行或合法取消、材料和 owner 真正接收、late routing 已安装，才准 FinishClose；不能仅凭四个 Delta 释放。Waiting/Ready 无计时器可免等，仍必须保留全部控制职责。Completed 若由 Active/Frozen close 流程产生，原 deadline 仍适用。

### 5. Ready/Enter 及 recovery 可处理性

> B 112–121：“If validation, adoption and the atomic Enter transition are the complete message-processing operation covered by … [network] … `e_v <= e+Delta`.”

这是明确的条件陈述。B 14–47 要求完整可执行 state 及全部验证输入；B 60–70 在首个实际 Enter 同时发送；B 90–105 将已验证 state 对象直接采用为执行对象，不允许仍有独立恢复工作时称 Ready。因此在这一抽象接口确实成立时，原 complete-message bound 能覆盖接收者的最终 Enter。B 124–146 同时正确声明：若实际还需要 `u_v` 的 native import，不能由命名为一个事件得到 `u_v=0`。本审查没有把该实现义务当作已经验证。

> R 280–291：“For `g>0`, let `a^-` be the first correct Quit of `g-1` … Applying the same complete-proof argument inductively …”

需要检查的不是 Part I 的统一 authority availability time；Part II 没有该共同到达时间。这里可用的是首次正确 Quit 的完整 proof relay。由初始 Ready 窗口开始，首个正确 Quit 携带其 authority/entry history；一个尚在前一 recovery 的未关闭成员会收到这些完整材料并处理 buffered proof。每次本地退出后的原 `Delta` wait 保留。对指定 generation，首个 entry `e>=a^-+Delta`；每个未被 Closure/Completion 阻断的成员在 `a^-+Delta` 前收到完整上一代 quit context，故其等待至迟在 `a^-+2Delta<=e+Delta<=s+Delta` 结束。当前完整 proposal 再供给当前 AUTH 和剩余 admission history。逐代使用同一逻辑，没有把长历史 catch-up 无条件变成常数时间，也没有借用 Part I 616–623 的共同 `d(g+1)`。

### 6. 存活 timer 的 early/late 分支

> R 263–274：“If any correct member first quits … `a<=s+Delta` … cancels the timer”; 对 Completion 同理。

直接 timer 在 `s+2Delta` 真正到期意味着没有相应正确 Quit 或 Completion 在 `s+Delta` 或之前发生。Quit 的完整 proof 与 terminal receipt 的 complete multicast 都会于到期前（含等时）触发取消。terminal receipt 与 generation 无关，Frozen/closing 仍执行 Completion；它先提交 terminal DATA 及祖先，再取消包括 DRAIN 在内的未到期 timer（M 1141–1153、R 35–38）。证明不需要先假设 receipt-based commits 已安全。

初始未进入的 receipt-cache 边界也被覆盖：首个 own-session receipt 至少需要一个正确 FINAL，它来自不早于 `e_0+2Delta` 的 timer；支持的 Ready contract 已在 `e_0+Delta` 前完成所有未关闭成员的初始进入。已关闭且不进入的成员不能成为后续目标 timer 的 committer。

> R 298–307：“if `s+Delta<c`, no closure suppresses that vote or entry … Every correct member … matching … vote.”

early case 的 all-correct matching votes 有真实前提：无 Quit、无 Completion，no-silent-closure 排除 Closed 所致缺席，完整提案和 admission 可处理。更早 term、已间接提交都不能免 Vote；冲突历史会在该窗口产生真实 quit proof 并反证目标 timer 存活。因此 `W-F>F` matching votes 在 `s+2Delta` 内形成 protecting QC。

> R 312–326：“`e_v(g+1)>=a_v+Delta>=a+Delta>s+2Delta` … late case … closed every … gate.”

early case 中每个可能再次 recovery 的成员在 lock snapshot 前已经获得 QC；窗口后 Completed 的成员没有后续 entry，故无需给它虚构未使用的 lock。late case `s+Delta>=c` 给出 timer expiry `s+2Delta>=c+Delta`，而所有正确 entry gate 已在 `c+Delta` 前关闭；任何后续 generation entry 又严格晚于该 expiry。因此即使目标 timer 没有 protecting QC，也不存在能签 higher-generation QC 的正确成员。不得把 late case 的局部 commitment 升格成 receipt 或完整 publication result，R 370–381 已正确给出门限不足例子。

### 7. 同代、跨代以及 receipt provenance

> R 329–338：“complete signed histories either extend one another or expose … equivocation.”

这一步有实际签名历史支撑（M 919–920）。同代不同 opening 暴露两个 NEWVIEW；相同 opening 的首个分叉位置暴露同一授权 leader 的两个签名 proposal。正确冲突签名者若先处理 B，就无法再合法签冲突；否则其已转发冲突历史不晚于 `s+Delta`，committer 到期前见证并 Quit。ANCHOR signer 同样转发 NEWVIEW（M 1245–1255）。因此后来才拼出 QC 不会避开已发生的正确转发。

跨代分支只对拥有 early-case protecting QC 的下代直接提交使用 rank descent；late case 已独立排除任何后代正确签名。每个 ANCHOR QC 的正确 signer 检查其固定 entry lock，fresh DATA/DRAIN 的 parent justification 严格低 rank（M 794–813、974–983）。这满足 I 795–815 的下降论证前提；没有沿用已失效的 universal protecting-QC lemma。

> R 353–361：“committing from another receipt emits no FINAL … traces to a causally earlier direct timer commitment.”

有效 receipt 至少含一个正确 FINAL；M 816–867、1116–1122 限定其来自 direct timer 或该 timer 的 ancestor commitment。receipt-based commit 不再产生 FINAL，故不存在靠 receipt 环状证明 receipt 的闭环。DRAIN timer 的 FINAL 只给 terminal DATA，而 Completion 取消未到期 DRAIN 后不虚构它已经提交。由此 receipt/Completion 只添加已有直接 timer 所保护的 DATA 前缀。

### 8. 历史验证、实时许可、全局前缀

> M 1854–1859：“VerifyHandoffHistory … does not require its version to remain open now and grants no entry permission.”

历史证明与 live activation 已分开。旧 receipt、Force retained anchor 和跨 session ancestor 递归只做历史验证；当前 Enter 还要 current/open、fixed boundary、所属 region、无 entry/close/tombstone/Closed（M 1908–1928）。因此关闭旧版本不会破坏新 Force 锚点的原始证据，也不会由晚到旧证据重开旧 engine。covering receipt 的 justified block 明确仍为选定 anchor，不是 receipt 中更高的被丢弃块（M 805–813、870–884）。

正常交接的 terminal height、DATA/xi 以及 historical selection 在 regional soundness 下唯一；Force 的 source context、pre-payload G 和新版本由公共 replay 唯一确定。反复 Force 在 unassigned 状态复用 predecessor index 而递增 beta（M 1798–1804、1885–1892）。固定 record 的冲突字段拒绝替换，初始 Enter 原子且持久（M 1901–1905、1977–1985），故 pair 隔离证明成立于所述条件。

全局 DATA 部分的归纳成立：MSIG/AUTO prelude 先于 payload；最多一次匹配 Force；完整 G/provenance 保留；EXPORT 对 exact current base 做原子连续追加，并检查 open version、每个 session 证据与 reference 的前置位置（M 1416–1454、2192–2229、2285–2313）。Missing history 停 replay，不能被当成零输出或空候选。关闭延迟传播影响本地生产，不能改变 canonical public cutoff。

### 9. F1：相同 Espresso prefix 不蕴含完整 interpreter state 相等

> M 1458–1460：“It contains … pending controls, and source-verification checkpoints.”
>
> M 2559–2560：“Replaying the same complete Espresso prefix yields the same interpreter state.”

**Classification: Critical Error（可直接反驳的过强子结论）。**

取 `T_L1=1`、`epsilon_E=0`、`eta=0.1`。两位正确 observer 从同一个 checkpoint 开始，下一 Espresso block `J_q` 的 timestamp 和 finalization time 均为 `10`；没有 DATA 或 due control 改变该例中的 `G`。

1. 正确 committee 在 real time `10` 提交合法 STATUS `x`，其激活阈值 `theta_a=20`。严格满足 `20 > 10+1+0+0.1`。
2. `x` 在 `10.2` finalized。B 在 `10.1` 刷新当时完整 finalized L1 prefix 并 replay `J_q`；该时刻 `x` 尚未 finalized，因此其 pending-control queue 不含 `x`。
3. A 在 `10.3` 获得 `x`，按 M 256–269 与 2249–2253 验证并保留尚未激活记录，在 `10.5` replay 同一个 `J_q`。
4. B 在 `10.8` 才取得 `x`；该延迟仍早于 `10+T_L1=11`。在 `10.6`，两者正确地完成同一 Espresso prefix 的 replay，A 的 pending queue 包含 `x`、B 的不包含；来源控制游标也可不同。两者均未漏掉任何在 `J_q` applicable 的 control，故没有违反 complete-applicable-controls 或 advance announcement。

因此完整 `S_A != S_B`，尽管两者 `G`、已生效 eligibility、Force boundary 等输出一致。即使把 comparison 理解为各自 replay 的完成瞬间，步骤 2 和 3 仍直接给出不同结果。原证明 2567–2573 只能从相同**已适用**的输入得到相同 prelude；不能推出未适用队列与获取 checkpoint 相同。

此 finding 的后果限于逐字的 state-equality 子结论及复述它的表述（例如 M 2050–2051 若“public state”也包含未来输入队列）。本报告不将它传播为 DATA prefix 分叉：同一 canonical Espresso position 所需的 applicable controls 仍共同；未来记录在激活前不改 G，激活时由原公告/完整性假设确保共同处理。报告仅指出必须区分两种状态对象，不在 verifier 阶段修改定理或协议。

### 10. 明确保留的条件及未认证事项

- 本次是手工对抗审查与源文件/哈希核验，未运行 model checker，也未认证可执行实现。
- 主文件冻结的 regional receipt/application soundness（M 495–501）是条件接口；本报告没有把它当成已独立实现证明。
- ReadyBundle 的可执行 native-state adoption/timing 条件仍需具体 adapter 证明。若存在非零且无统一上界的 `u_v`，B 129–144 已明确不成立的 `e+Delta` 推论不能用于生产实现。条件性 safety verdict 不覆盖这样的实现。
- 本次通过的是指定 session 的区域 retirement 安全与 canonical G preservation 的条件论证；不同 branch 的 speculative local DATA 可以不同，不能把它们统称成同一全局已确认 log。
- `c+4Delta` 不证明 complete receipt 形成、storage transfer 完成、publication acceptance 或 request progress。主文件 2638–2785 的 liveness 限制仍在；本次判定不撤销它们。
