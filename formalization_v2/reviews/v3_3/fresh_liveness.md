# v3.3 独立 liveness / Ready 评审

## Summary

**Final Verdict:** 本轮所查 ReadyBundle 抽象时序、bootstrap 条件界及严格引用/self-fallback 局部命题成立；但“至少一个可用 region 时持续全局进展、健康 region 的有效服务、正确请求最终进入 $G$”仍有 **JustificationGap**，不能通过完整目标验收。弱服务量词下有具体反例；不能把这些反例不加条件地称为“满足冻结全部进展前提”的反例，也不能因条件安全通过而宣布协议安全且活跃。

本报告为 reviewer-only：没有修改协议，没有读取先前/preflight/其他 reviewer 报告。只使用下列冻结源文件核对当前论证；环境默认记忆检索仅返回历史条目索引，未将历史评审结论作为证据。范围为 Ready/启动、选择器、持续进展和请求进展；不代替另一份完整的 regional-safety 复核。采用 proof-verifier 的逐步依赖检查，不以工具测试替代证明。

### 快照身份

以下相对路径均相对于 `formalization_v2/reviews/v3_3/snapshot/`，行号是文件原始行号。实际 `shasum -a 256` 与 `manifest.json` 的六项全部一致。

| 简称 | 路径 | SHA-256 |
|---|---|---|
| M | `sync_hotstuff_rr_part2_rotation_v3_3.tex` | `e69ba625229741039d704943d297830476242dadddbf473ae8cb3375f24b4f88` |
| R | `sections/rotation_v3_3_ready_bundle.tex` | `b5347497e6b925c15cd22bd78285c36fbf31ec1ff3d2f20fda3a770295f89858` |
| D | `sections/rotation_v3_3_retirement.tex` | `e024b736ad043c4912d8ae0d2f15072e3cd0cf2f0f1f42e35897971568d1bb58` |
| C | `sync_hotstuff_rr_part1_core.tex` | `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3` |
| V | `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` |
| A | `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |

额外按章节边界抽取比较：M 与 V 中从 `\section{Model and assumptions}` 到下一 section 的内容，均与 A **逐字节相等**，不是仅忽略空白相等。

### List of Findings

1. **J1 — JustificationGap：完整活跃性目标未获证明。** Location: M:2677–2679 “continuing global progress … useful service … request progress”；M:2752–2758 明言尚未修改 cutoff 或完成目标。Issue: 当前正向命题仅覆盖条件启动和条件选择；未建立每个未来时点之后仍有新 DATA 进入 $G$，更未建立正确工作或请求最终被接受。稿件已披露不足，因此这不是把稿件现有保留语句判为虚假；是完整交付目标未满足。
2. **J2 — JustificationGap：冻结进展前提的关键量词未形式化。** Location: A:303–309 / M:516–522 “accepted in time during some sustained service interval” 与 “enough uninterrupted service time”。Issue: 一次有限成功区间、任意未来时点之后存在成功区间、覆盖每个待处理正确结果，是不同前提。慢轨迹的全局停滞结论只在第一种等较弱解读下反驳全部所选前提；请求饥饿在仅要求重复成功、但不指定正确工作被服务的解读下仍成立。
3. **J3 — JustificationGap（具体实现符合性，非抽象推导错误）：native adapter 的合同尚需落实。** Location: R:108–140。Issue: 冻结模型的消息处理界能覆盖协议明确规定的“验证后直接 adopt 已可执行对象并 Enter”操作；它本身不证明任意数据库 restoration/conversion/native initialization 也有该界。稿件已正确保留这一限制。若实现还有未覆盖的 $u_v$，不能使用 $e+\Delta$ 及其下游界。
4. **J4 — JustificationGap：selection fairness、leader fairness 与 useful acceptance 之间缺少桥梁。** Location: M:2640–2645、2721–2739；A:297–312。Issue: 无限次真实正确 leader turn 和无限次 region selection 都允许其有用输出全部在已关闭 suffix 中。正确请求的 finite-local-predecessor/FIFO 不能独自消除此情况。

**CriticalError 状态：** 在本轮所查、且按原文精确条件解释的局部正向命题中，没有找到已成立的 CriticalError。以下三种若被用于后续“完整已证明”结论，则会成为 CriticalError：把外部安装时间直接算作零；把 W_start=6 轨迹无条件称为任何量词解释下的完整前提反例；从无限 $G$ 增长或正确 leader 真实轮转直接推出请求活跃性。

## Detailed Verification Log

### 1. 冻结前提和所需结论

> A:177–179: “The delay bound includes delivery and processing of complete validation dependencies.”

模型给出 region 内正确到正确完整消息的 delivery + processing 界 $\Delta_i$。它没有给首次收集任意历史、重建状态、得到下一 authority 的统一界（A:131–133、233–234、264–268、307–318）。Publish/Espresso 界始于完整 result，且显式不保证该 result 通过本链 acceptance（A:183–196、238–265）。因此不能把 $T_{\rm pub}+T_E$ 当作“从 public duty opening 到首次全局 DATA”的界。

> A:297–300: “Correct leaders receive repeated turns that are actually entered …”

这排除了只在从未到达的 schedule 后面列出 honest leader 的伪公平证明。下文反例安排每个正确身份实际成为 initializer、实际提案和提交无限次。没有发生 recovery 时，不能凭空要求无限次 recovery entry；若另行要求 recovery turn，必须将此量词写清。本文没有偷偷用 nominal schedule 代替实际进入。

完整目标至少需要区别以下性质：

- 全局增长：对任意未来时点 $t$，存在 $t'>t$ 使 $G(t')$ 严格扩展 $G(t)$。
- region 有效服务：指定健康、持续有资格的 region 的 DATA 实际反复进入 $G$，不是仅创建/进入本地程序。
- 请求进展：每个持续 admissible、已送达持续服务 region 的正确请求最终进入 $G$。

当前已证的条件启动界及 selection 命题均不足以单独推出这些性质。

### 2. ReadyBundle：协议合同可承担什么

> R:25–33: “full, already constructed application-state object” 及 native validation evidence。
>
> R:60–70: “constructs its complete bundle before its first Ready/Enter event … sends … at the event”。

该修补消除了旧 V:1578–1617 中 “complete bundle” 未明确包含可执行状态的接口空白。第一次正确 Enter 时已经有完整 $S_A,\Pi_A$、handoff、authority、history 及 transitive dependencies；发送不能推迟到 entry 后的后台任务。跨 region 获取、状态重建和形成 bundle 的耗时位于 $e$ 之前，不被本次证明赋予统一界。每个正确 entrant 都执行发送，无需识别谁是第一人。

> R:86–101: “validate and adopt … in this same message-processing event”；“does not first reconstruct a different database”。

在抽象状态机中，把已验证对象作为新程序的 executable state，是可以通过协议表示和 handler 语义来实现的操作。冻结 ass:network 对该完整协议消息的 processing 保证直接适用；无需再增加一个独立“state-transfer service”环境假设。**但成立的对象必须确实是这个 handler 可直接运行的对象，而非将待恢复快照改名为 executable object。** Soundness of $\Pi_A$ 仅证明内容对应锚点，不能证明 native adapter 的时间行为。

> R:104–118: first send at $e$，且 target 全程可激活，无 valid entry prohibition。

逐步推导如下：第一正确 entrant 在 $e$ 发出完整正确消息；每个正确收件人至多在 $e+\Delta$ 完成该 handler；guard 在该区间允许进入，最后一个原子步骤就是 Enter，或收件人已更早进入。因此

\[
\forall v\text{ correct},\qquad e_v\le e+\Delta.
\]

此结论以有限的 first correct Enter 存在为起点，不证明该 $e$ 一定存在，也不证明其相对 public opening 的距离。如果闭合、supersession、quit evidence 或其他有效 entry 禁止介入，相应上界的前提不成立，不能继续套用。

> R:120–140: 若外部安装仍有残留 $u_v$，没有统一界。

稿件的限定准确。对仍需单独 import/restore 的具体实现，只能在最终安装且仍有许可时写出依赖实际 $u_v$ 的式子，不能由“把操作叫作同一事件”推出 $u_v=0$。所以 **抽象协议义务可接受；无条件的具体 native 安装界未获证明**。这个区别是 J3 的全部范围，不应把已写明条件的正确命题误判为假命题。

### 3. Bootstrap 和 progress timers

> M:1982–1991: $e+\Delta,e+3\Delta,e+5\Delta,e+6\Delta$。

在 Ready 合同成立、handoff 全程可激活、正确 initializer、无 Quit/Completion/RequestClose 阻断的条件下：initializer 至迟 $e+\Delta$ 进入；其自己的 $2\Delta$ 等待至迟于 $e+3\Delta$ 完成；NEWVIEW 至迟 $e+4\Delta$ 到达正确成员；正确 ANCHOR votes 至迟 $e+5\Delta$ 回到 leader；正确权重 $W-F>F$ 足以构成 QC；首 DATA 至迟 $e+6\Delta$ 到达所有正确成员。各节点 first progress check 在 $e_v+6\Delta\ge e+6\Delta$，且消息先于同刻 timer（M:1060–1061、1182–1199）。因此正确启动不会因这些相等边界被错误 BLAME。

该局部推导无需额外等待 terminal DATA 的 Espresso publication；但 Ready 自身现已需要一个严格晚于 opening selection height 的 finalized header（M:1934–1938）。该等待移到了 $e$ 之前。Espresso growing timestamps 及可检索性可给最终存在后继 header，**不提供相对 duty opening 的固定等待界**。持续 AUTO 可使每次 header/Ready 准备完成时目标都过期。

在同一未关闭 session 且该首 proposal 的 timers 不被取消时，首 DATA 的全部本地 commit 还需至多另 $2\Delta$，即至迟 $e+8\Delta$；完整 FINAL 形成及 publication 又有后续步骤。M 的 $e+6\Delta$ 本来只声称 proposal 到达，不能用它支持长度 6 的全局接受窗口。

C:681–709 的 correct-leader progress 证明在入口 skew 已建立后适用。C:870–944 的无界 committed-height 和 request 证明还要求其工作不被外层闭合不断丢弃；不能直接迁移为 M 的全局请求 theorem。D 只保全已启动 timers/证据，并未承诺晚生成结果还能进入 $G$。

### 4. 严格 terminal reference 与 self-fallback

> M:2638–2668: “opening selection heights strictly increase”。

由 M:633–634 的 terminal strictness 及 M:1799–1806 的历史最小选择：

\[
q^{\rm open}_{\beta,r+1}=q_s
\ge \operatorname{height}(\xi(T_{\beta,r}))
>q^{\rm open}_{\beta,r}.
\]

连续 regular handoff 的 opening 高度是严格递增整数。若存在无限条这样的 handoff，必越过任何固定历史高度；若越过某高度后 eligibility 是固定非空有限集合，固定 cyclic search 确实轮流访问该集合每一成员。这个证明有效，且明确未证明无限 successful handoff 本身存在。它也没有把随时间变化的 eligibility 自动升级成公平性。

> M:1768–1782: 没有 alternative 时选择 eligible source 本身。

对一个未被新 Force supersede 的固定 pending boundary，如果历史搜索期间没有 eligible alternative，并且 source 最终 eligible，则最早使 selector 非空的历史位置可以选中 source。AUTO cooldown 正数有限且不因 replay 重置；growing Espresso time 可令其到期，但 OFF 仍阻断。新建的是新 pair，原 session 不被重启。局部命题成立。

**冻结前提的边界：** A:307–309 仍说 “A waiting boundary eventually has another eligible region”。若 `another` 严格指不同于 source 的 region，那么只有一个永久 eligible region 且 AUTO cooldown 使 boundary 等待的执行不满足此项完整前提。Self-fallback 是有效协议改动，但不能把该 sole-eligible 执行同时称为满足这项文字的模型内反例。`usable/live region` 与治理状态 `eligible` 也不是同一概念。要审查完整 one-live-region 命题，必须明确它在冻结前提下量化的执行域；不能暗改成必须两个 healthy region，也不能默默抹掉 `another`。

### 5. $W_{\rm start}=6$ 慢会话执行

> M:2683–2719: 两个 region，$W=3,F=1,\Delta=1$，正确 initializer 的结果在 cutoff 后形成。

这里权重 $W=3$，窗口是 $W_{\rm start}=6$，二者应避免混称。每个 region 两个正确 unit-weight 节点与一个沉默 Byzantine 节点。取 $0<\varepsilon<0.1$，duty 在实时时刻/时间戳 $a$ 打开。后继 Espresso header 在 $a+\varepsilon$ final 且立即可验证；两正确节点此时一起 Enter。以下具体延迟均满足 $\delta\le1$：

| 实时时刻 | 事件 |
|---|---|
| $e=a+\varepsilon$ | 完整 ReadyBundle、Enter、STATUS；严格 terminal-reference 依赖齐备 |
| $e+2$ | 正确 initializer 等满自己的 $2\Delta$，发送 NEWVIEW |
| $e+3$ | 另一正确成员收到 NEWVIEW 并发送 ANCHOR vote |
| $e+4$ | initializer 得到第二张 ANCHOR vote，提出 terminal DATA，自投票并启动 timer |
| $e+5$ | 另一正确成员收到 DATA、自投票并启动 timer |
| $a+6<e+6$ | Espresso deadline block 的 prelude 发现 accepted count=0，AUTO 闭合版本 |
| $e+6$ | initializer timer commit，发送 FINAL |
| $e+7$ | 另一正确成员 timer commit；可在此时取得两个正确 FINAL，形成首完整 receipt |

取 closure 即时可见或至多允许界内可见即可；两票在 closure 前已发。D:48–71 的 close gate 不取消这些 timers。选择两个 FINAL 以延迟 1 传播，即可使首 receipt 恰在 $e+7$ 形成；沉默 Byzantine 不提供更早的 FINAL。Complete-receipt multicast 不可能让 receipt 先于其必要签名形成。progress check 在 $e+6$，当时已投所需的一票，且 close gate 也禁止新增 BLAME；无 Quit 或 equivocation。

完整结果之后确实交给 Publish Layer，并可在 $T_{\rm pub}=T_E=1$ 的界内提交/finalize。M:2534–2535 和 2196–2200 的 cutoff 永久禁止它扩展新 $G$。Publication 成功与 global acceptance 失败并不矛盾。

下一 duty 在 $a+6$ 打开。两个 region 轮流被选，source cooldown 到 $a+6.5$，至其再次于 $a+12$ 被需要时已经到期。交替 initializer 身份使所有正确身份无限次真实 Enter、提案、投票、commit。每个下一 boundary 当时有另一 eligible region；authority/state 皆可预先备妥；timestamp=real time 满足 clock-lead，所有 controls 可为空；没有 committee 抢占或 equivocation。可在此无限尾部前放任意长有限成功 prefix。

**精确结论：** 核心执行是合法的 protocol/timing trace。若服务条件仅要求存在一个有限 sustained successful interval，则此前成功 prefix 满足它，随后 $G$ 永停，反驳该解读下的持续增长目标。如果条件要求每个未来时点后都存在足够成功服务，或某无限 successful tail，则这个纯停滞尾部不满足该强化解读，不能作为其反例。M:2713–2719 自身正确说明了这一点。`committee leaves … enough uninterrupted service time` 若只约束委员会行为，本例无 committee operation，已满足；若被解释为包括保证 AUTO 不抢占的端到端成功条件，则还必须承认它已经额外选定了成功窗口解释。

### 6. 请求饥饿：无限全局增长仍不足

> M:2721–2739: 正确请求 $x$ 首位排队；正确慢工作关闭；Byzantine 快工作空 DATA 被接受。

取持续 admissible 的单个 $x$，送到每个正确节点，排在其各自队首，没有旧请求。每个正确-led 慢 session 都按 M:2298–2309 把 $x$ 放入 terminal DATA；闭合后仍保留 pending。与上节慢 session 交错安排 Byzantine initializer 的快 session。Byzantine 仍等待 $2\Delta$ 并只签一个合法空 DATA；把其余实际消息延迟取足够小，至少一个正确 timer 在约 $a+4$ 到期，配合 Byzantine FINAL 或另一正确 FINAL 可形成 receipt；取允许范围内足够快 publication/finality，使接受发生于 $a+5<a+6$。无须所有延迟都取上界。

Empty DATA 本身有效且算作窗口进展（M:2113–2117）；正确 leader 的 FIFO 是对正确 proposer 的行为要求，并非 receiver 能从未知远端 queue 检验的每块有效性要求。Byzantine omission 不构成 equivocation，permanent equivocation exclusion 对其不适用。快 session 接受后可以通过 regular handoff 打开下一 duty；慢 session 则通过 AUTO 打开下一 duty。适当安排固定 authority 序列和初始 leader，可使两 region 都有无限快成功 session、所有正确身份都有无限慢真实工作。无跨 region transport 上界强迫在 closure 前启动额外 successor；可令 successor 从最终 public 状态取证。

于是同时成立：

- $G$ 无界增长，两 region 的 accepted empty DATA 都无界增长；
- 所有正确身份获得真实 DATA-producing turns、反复提出和本地提交 $x$；
- 每次完整 result 均按假设发表，request 的 pending/finite-predecessor 条件始终满足；
- 所有包含 $x$ 的 DATA 都属已闭合 suffix，$x\notin G$ 永久成立。

这反驳“真实 leader fairness + FIFO + 无界 global/region output”足以推出请求进展。若冻结 acceptance-service 前提只要求不断出现成功 EXPORT，而不要求成功 work 来自正确 leader/包含 $x$，上述 execution 仍被允许。若前提被解释为每个 pending 正确结果/请求都必须获得可接受服务，则该 trace 被排除，但此时已经把需要证明的关键服务保证放入前提，不能无说明地称其由 FIFO 推出。

此外，只让 region A 的快 Byzantine sessions 成功，让 region B 的全部正确 sessions 均采用慢执行，可得到 $G$ 无界增长但健康 B 从未提供 accepted DATA 的变体。它同样说明“selected repeatedly”不等于“usefully served repeatedly”。对该变体是否满足全部 service 前提，也必须逐项使用所选量词，不能跳过“enough time”的解释。

### 7. 剩余障碍及协议改动方向的审查边界

以下是对修补方案的约束，不是新定理或已完成的设计。

1. **只增加固定窗口不足。** A:307–318 对 preparation/authority/recovery 没有统一界；可以在新的 attempt 中让真实 Enter 晚到，使正确提案发生、但必要 receipt 仍越过 cutoff。单纯让有限 timeout 逐次变大也没有自动解决“每个新 attempt 可以有更大的有限准备延迟”的量词问题；需要持久保留进度或证明某个特定 attempt 最终获得足够服务。
2. **只把窗口起点移到一个本地 Ready 不够。** 公共解释器需要可验证的一致触发；并须防止永不 Ready 的 candidate 阻塞全局。可以研究用现有签名/传播服务构造可验证启动证据、在后台保留候选准备，但必须证明其 public ordering、活跃性与资源/权限规则；不能把“所有节点 Ready”作为新环境前提。
3. **只接受迟到完整 result 也不保证请求活跃性。** 即便允许旧 result 在锚点仍等于 $G$ 时进入，持续快 empty DATA 仍可能先推进 $G$，使每个含 $x$ 的迟到 result 失去 exact-base 条件。保护结果与保护请求是两个不同义务。
4. **可继续研究的协议级方向：** 将“换候选/准备替代”与“不可逆剥夺已验证工作的 append 资格”分开，保留足够的 attempt/state/evidence，使弱 eventual availability 能最终完成某份工作；同时给正确请求或正确生产机会一个可验证、持久的服务次序，禁止快空输出无限越过它。若把这落实为协议消息、验签/有效性条件和确定性公共仲裁，而非假设网络会优先服务某类 leader，才可能不增加外部 fairness。它会改变当前 Force cutoff/acceptance 语义，必须重新验证 regional/global safety、dead-candidate escape 和用户请求活跃性，不能称作已被本轮证明的“最小安全修补”。
5. **当前没有可据此直接签收的修补。** 不得把“healthy region 足够快”“某正确输出会被接受”“native import 至多常数”“客户端获得新的独立可靠公链 ingress”悄悄加入模型。新的 request/public-intent 消息若被采用，也要证明它使用冻结服务的既有适用域，而非替换服务保证。

### 8. 依赖图与最终传播状态

```text
冻结完整消息同步界 A:167–179
 + 真实 executable ReadyBundle / atomic adopt 协议合同 R:60–101
 + first Enter 存在、全程无 entry prohibition
    -> entry skew <= Δ                         [已核对，条件成立]
    -> correct initializer bootstrap 3Δ/5Δ/6Δ [已核对，条件成立]
    -> 本地 progress check 不误触发             [已核对，条件成立]
    -X-> public opening 到 Ready 的统一界       [没有该推论]
    -X-> receipt / EXPORT 在固定 cutoff 前接受 [J1/J2]

strict terminal reference + historical minimum selection
    -> regular opening heights 严格递增         [已核对]
 + 无限 regular handoff + 最终固定非空 eligibility
    -> 每个 eligible region 反复被选择          [已核对，条件成立]
    -X-> 无限 handoff 存在 / 有效服务             [J1/J4]

self-fallback + fixed pending boundary
 + source 最终 eligible、无 alternative / supersession
    -> source 可被重新选择                      [已核对，条件成立]
    -X-> startup 成功 / 持续全局进展              [J1/J2]

正确 leader 真正反复工作 + FIFO + pending 保留
    -> 正确 proposal 反复携带请求                [适用时已核对]
 + 请求所属正确工作最终进入 G                    [当前缺失]
    -> 请求最终进入 G                           [不能跳过缺失前提]

无界 G 增长 / 每 region 无界 accepted empty DATA
    -X-> 正确请求被包含                         [慢/快交错 trace 排除]
```

局部已核对命题可以保留其精确条件；所有依赖 J1/J2/J4 的完整进展主张继续标记未验证。本轮执行了哈希与冻结章节相等性检查、逐步时序/量词审查；未运行 model checker，也未证明完整 native implementation 的时间符合性。
