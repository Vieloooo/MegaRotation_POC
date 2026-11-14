# v3.2 独立安全审查

## Summary

**Final Verdict:** 在冻结模型、基线 regional soundness 与外部验证接口成立的条件下，未找到破坏全局 DATA prefix 的模型内执行；公开解释器与本地实例隔离的主论证可成立，但退休安全证明仍有一项实质性 **Justification Gap**，历史 handoff 验证接口还有一项规范层面的 **Justification Gap**，因此不能把整套 v3.2 安全证明标记为已通过。

审查身份为独立 verifier；使用 `proof-verifier` 工作流，仅阅读所列冻结快照及该工作流，不读取作者、预检或其他审查报告，不修改协议。以下 `M:L` 指主文件行号，`R:L` 指退休附录行号，`P1:L` 指 Part I 行号。行号均属于本次快照。

### List of Findings

1. **S1 — Justification Gap，退休安全证明中的 Completion 分支未闭合。**
   - **Location:** `R:158–161`：“If $s+\Delta<c$ ... The baseline protecting-QC and recovery-lock argument therefore applies ... with its unchanged treatment of genuine Completion.”
   - **Issue:** $s+\Delta<c$ 只排除了 retirement 对传播窗口内投票的抑制，没有排除原有 Completion 对投票的抑制。Part I 的 protecting-QC lemma 要求所有正确副本投票；所给基线 soundness premise 不等于这个更强结论。下文给出满足基线安全、但 early-case DRAIN 无 protecting QC 的执行。需要核实 Completion 对应的独立证明分支，不能仅引用 Part I。若将该句理解为“每个 early-case timer 都有其自身的 protecting QC”，该子断言直接为假；本审查不据此声称退休安全命题本身为假。
   - **Affected:** `prop:retirement-safety` 的 early case，以及其 `R:178–184` 对跨 generation/ancestor/receipt 的汇总结论尚未完全验证。`prop:retirement-drain` 不受影响；全局 prefix 命题的状态机保持部分也不依赖这个 QC 存在结论。
2. **S2 — Justification Gap，历史 admission 证明与当前 Enter 授权的验证语境未明确分离。**
   - **Location:** `M:1820–1829,1847–1861`：`VerifyHandoff` 重放至 “currently processed public prefix”，并要求目标版本 “still open”；同时 `M:829–840,897–901,1854–1855` 要求验证 retained anchor 的原始 receipt、ancestry 与 boundary admission witness。
   - **Issue:** Force 必须使用刚关闭版本的原始 anchor/receipt。历史 boundary 的证明有效性不能由“该版本在当前 prefix 仍开放”判定。当前文字没有给出历史验证与 live activation 的不同调用契约；按唯一给出的 `VerifyHandoff` 过程递归验证旧 boundary，会拒绝一个真实的 retained anchor。正文明确要求旧 receipt 仍有效，故这里应视为接口未闭合，而非已证明的安全攻击。
   - **Affected:** 新节点或 cache miss 时对 Force packet、旧 receipt 与历史 admission 的可执行验证，以及 `prop:program-isolation` 依赖的统一 handoff 判定。尚未构造两个正确观察者得到冲突 $G$ 的执行。

**Critical Error 数量：0 个已确认的命题级安全反例。** S1 包含一个可被否定的强读法；本报告按原文尚可能意指某个未给出的 Completion 分支，将整体问题列为 Justification Gap。没有把未提供的证明补进作者论证。

## 冻结材料与模型过滤

实际重新计算的 SHA-256 均与 `snapshot/manifest.json` 一致：

| 文件（相对 snapshot） | SHA-256 |
| --- | --- |
| `sync_hotstuff_rr_part2_rotation_v3_2.tex` | `956b733d9c30bb6aaed3a2b5a7e67f3628e620392b958ed7471455243108dc1d` |
| `sections/rotation_v3_2_retirement.tex` | `6174fc0dcaeadf6ced021b33e3f8f4390a73d0b477306d0486718074414279e5` |
| `sync_hotstuff_rr_part1_core.tex` | `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3` |
| `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` |
| `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |

从 v3.2 主文件提取整个 `Model and assumptions` section，直至下一 section 前，与冻结 `model_and_assumptions.tex` 逐字节相等（两者均 16,456 字符）。因此没有通过改写 Section 2 增强模型。

本审查保留以下冻结 premises：固定 voting snapshot 与 Byzantine weight $\le F<W/2$；正确节点间完整验证依赖在 $\Delta$ 内送达并处理；完整且不可变的 L1 control history；兼容且可认证的 Espresso history；advance announcement；原有 regional receipt/application verification soundness。未使用漏控制、超时消息、伪造签名、旧密钥超额妥协或在模型外丢失唯一历史副本作为反例。

`R:136–146` 的 “baseline soundness premise for DATA/DRAIN, receipts and Completion” 只能当作已有基线的条件化接口，不能被解释成退休修改已经安全，也不能自动推出基线没有陈述的 universal-QC 性质。S1 正是检查这一依赖边界。

## Detailed Verification Log

### 1. 公开 Force、regular 竞争与 accepted prefix

**引用：** `M:2124–2156` 依次执行 STATUS/CLEAR、首个匹配 entry version 的 MSIG，否则 AUTO，然后保留 $G$、增加 $\beta$、选择 successor。

检查结果：在相同已认证输入下，这些步骤具有确定顺序。多个 due MSIG 不能连续增加版本；附属 configuration 只有选中且 target 匹配者适用。AUTO 被匹配 MSIG 抑制。时间跳跃及相同 timestamp 不引入 payload 插入点。Force 的 retained anchor 是触发区块 payload 前的完整 $G$，不是旧程序最新 speculative tip（`M:2042–2043,2145–2149`）。

**引用：** `M:1405–1417` 要求 exact base、open version、连续 DATA、原始 regional evidence、严格早于 containing block 的 selection references，并禁止 Force 新版本在其触发区块输出。

检查结果：旧 regular successor 可以比全局确认领先，但不能凭旧 receipt 在关闭后追加旧版本 DATA。触发区块中的旧 EXPORT 先遇到关闭检查；新版本 EXPORT 则遇到 same-block 禁令。因此“terminal receipt 与 Force 谁先到本地”影响旧 speculative worker 是否曾启动，不影响 canonical $G$。附加 ancestry 中的旧 DATA 只是验证材料，不应误算为新 append。

**引用：** `M:2487–2513` 的全局 induction。

检查结果：在冻结 sound verification interface 下，对共同 finalized Espresso prefix 逐 prelude/record 归纳是合适的。每次转移只保持或追加 $G$；Force 不重写已接受身份。尚未发现循环依赖于本地 Ready、Completion 或 retirement phase 的 global acceptance 条件。该结论是 source/validation 条件化的输入 prefix finality，不能升级成 native execution 或 withdrawal finality。

### 2. Typed handoff、immutable pair 与异步 callback

**引用：** `M:1758–1786,1841–1861,2527–2536`。

检查结果：REGULAR 保持 $\beta$ 并增加 $r$；Force 增加 $\beta$，根据被捕获 export duty 或存储的 pending source context 选择 $r_b+1$。未 assignment 前重复 Force 保留 predecessor index，但使用新 $\beta$，不会与已经打开的 pair 混同。最早 eligible 历史位置与 immutable authority 决定 selected region。preparing plan 的可变性不会修改 installed fixed handoff。

**引用：** `M:1598–1600,1654–1664,1876–1899,1907–1914,1984–1991`。

检查结果：callback 绑定 target pair 与 anchor producer/height/hash；恢复到错误 anchor 的 state 无法通过 Ready。Ready 与 Enter 在同一 serialized event，durable entered flag 先于签名 dispatch，重复消息和 crash resume 不授予第二次初始 Enter。旧程序与新程序的 key 包含 $\beta$，故复用 $r$ 不混合 locks/votes/timers。

**S2 的具体触发路径：**

- 版本 7 的 $(7,20)$ 经真实 REGULAR boundary 打开，当前 $G$ 已接受其 DATA 108。
- 在 $J_{q_F}$，合法 Force 关闭版本 7，建立新目标 $(8,21)$，保留原生产身份为 $(7,20)$ 的 anchor 108。
- 新节点从 authorized checkpoint 重放到 $q_F$，没有旧 boundary 的已验证 cache，但具有全部必要 history；这不违反任何 availability premise。
- 它验证新 Force packet 的原始 anchor receipt。`def:receipt` 的 ancestry validation 需要旧 session 的 admission witness。
- 若这里再次调用本文唯一具体给出的 `VerifyHandoff` 并使用 current processed prefix，旧 REGULAR boundary 在 `M:1828–1829` 因版本 7 已关闭而失败；旧 Force boundary 则遇到 `M:1847–1848` 同类问题。

这不是说历史 witness 实际无效。相反，`M:179–184,1312–1314,1457–1460` 明确保留它的证明效力。缺口是调用语境未定义：historical authority/boundary validity 与当前 activation eligibility 不是同一谓词。不得以“以前 cache 过，所以通过”作为 soundness 的额外假设。完成这项接口定义后，才能严格核实 cache hit/miss 得到同一判断。

### 3. RequestClose、4Δ drain 与资源转移

**引用：** `R:14–57`：关闭必须由完整 finalized prelude 证据触发；全部 fresh-action gates 关闭；旧 timers 保持原 deadline；真实 Quit 与 Completion 保持原取消语义；消息先于同时刻 timer/retirement。

检查结果：pending MSIG、OFF、裸 timestamp 均不能单独关闭。initial Enter、term/recovery entry、STATUS/NEWVIEW、buffered proposal 与 authority callback 均受 gate 覆盖。既存 commit timer、证据传播及 FINAL 未因 Force 静默取消。没有观察到一条被 gate 遗漏的明确 fresh-signing 路径。

**引用：** `R:98–117` 的 drain proposition。

对拥有既存 timer 的正确实例，设第一份完整 closure relay 在 $c$ 发出，则 close gate 至迟 $c+\Delta$ 关闭。既存 timer 的 vote time 满足 $s\le c_v\le c+\Delta$，故 expiry $s+2\Delta\le c+3\Delta$。真实 Quit/Completion 只可能取消，不会延期；实际生成的 regional FINAL 至迟 $c+4\Delta$ 送达。每个 drain deadline $d_v=c_v+4\Delta\ge c+4\Delta$。这个计算通过，且 equal-time message priority 处理了等号。

未进入实例可以通过 definitive cancellation 关闭 entry gate，无需声称它拥有一个并不存在的 timer；已 Completed 实例的提前退出也不遗留该实例的 active timer。`R:66–85` 的 durable owner、late-message routing 与 outstanding publication/data duties 是 FinishClose 的条件，故 $4\Delta$ 没有被错误当作 storage/publication 全部完成的上界。该 proposition 不保证 receipt 形成，也不保证最后一个 Byzantine-held message 在期限内出现。

### 4. 退休安全：晚窗口与同 generation 部分

**引用：** `R:147–167`。

对于在 $s+2\Delta$ 真正存活的 correct direct timer，若 generation $g$ 的首个正确 Quit 为 $a\le s+\Delta$，其完整 proof 会在 expiry 前到达并取消 timer，矛盾。因此任何此类 Quit 必须满足 $a>s+\Delta$。任何正确 $g+1$ entry 至少为 $a_v+\Delta\ge a+\Delta>s+2\Delta$。

若 $s+\Delta\ge c$，则 $s+2\Delta\ge c+\Delta$；全体 entry gates 已经关闭时，任何后继 generation entry 仍必须更晚。因此不存在正确的后继 generation vote，也无法形成需要正确 signer 的 QC。这里不需要一个实际存在的 $B$ protecting QC；晚窗口分析的关键时间不等式通过。

**引用：** `R:169–177` 与 `P1:735–746`。

在同一 generation 中，完整 signed histories 的首处分叉暴露原有 proposal/NEWVIEW equivocation。若另一 correct voter 在收到首个 history 前投了冲突票，其 forward 最迟能在首个 timer 的 $2\Delta$ expiry 前揭示 proof；若已先收到，则 evidence-first processing 阻止该冲突投票。关闭后继续保留、比较与传播证据是这个论证所需的动作，并非要求已关闭副本重新投票。该局部论证没有发现新的 retirement 破口。

### 5. S1：early window 不能直接继承 universal protecting-QC

**引用：** `P1:751–766` 明确由“every correct replica has already cast or now casts the same DATA vote”推出 protecting QC；`M:1087–1088,1131–1133` 的 Completed phase 则停止投票。`R:158–161` 仅检查 close time，没有排除 Completion。

以下是证明子步骤的具体反例；它不是冲突 DATA 攻击。令 $\Delta=1$，有两个正确成员 $H_0,H_1$ 与一个 Byzantine leader $B$，均为 unit weight，$F=1$。选取现有 publication/visibility 常数足够大，使下述有限区间内，发布到 Espresso 的 terminal receipt 尚未送达 $H_1$；这没有违反其 bounded delivery deadline。

| 时间 | 合法事件 |
| --- | --- |
| $0$ | $H_0$ 为 generation 0 的 terminal DATA $T$ 投票并转发；timer expiry 为 2。 |
| $1$ | $H_1$ 收到完整 $T$ history、投票，timer expiry 为 3；两人的票至迟时间 2 构成 $T$ 的 proposal QC。 |
| $2$ | $H_0$ timer commits $T$，发送 FINAL。Byzantine 的同内容 FINAL 只交给 $H_0$，故 $H_0$ 获得 terminal receipt 并 Completion。它会依法 offer complete result，但无需在此刻把完整 receipt 交给 $H_1$。 |
| $2.2$ | $B$ 向 $H_1$ 暴露同一 admitted terminal position 的另一份合法签名 proposal；$H_1$ 获得真实 equivocation proof、Quit，取消 expiry 3 的 timer。此 proof 在 $H_0$ 已提交后到达，不撤销 $T$。 |
| $3.2$ 以后 | $H_1$ 正常恢复到 generation 1，以已知 $T$ QC 作为 opening justification。$H_1+B$ 的 ANCHOR votes 可构成合法 ANCHOR QC；$H_0$ 已 Completed，不参加。 |
| $s=6.2$ | $B$ 给 $H_1$ 一个合法 DRAIN $D$，parent 为 $T$；$H_1$ 投票并转发，timer expiry 为 8.2。$H_0$ 收到后因 Completed 不投票；$B$ withholding DRAIN vote。 |
| $c=7.7$ | 第一份正确、完整 closure 被验证；因此 $s+\Delta=7.2<c$，确实属于作者的 early case。关闭不取消 $H_1$ 的既存 timer。 |
| $8.2$ | $H_1$ 的 $D$ timer commits $D$ 及 ancestor $T$，发送原规则要求的 $T$ FINAL。至今只有 $H_1$ 的 DRAIN vote，权重 1 不大于 $F$，故不存在 $D$ proposal QC。 |

真实 quit 的具体时刻、recovery authority 与 leader wait 可按表给出的余量满足；所有 correct-to-correct 延迟均不超过 1。$H_0$ 的 committed log 为到 $T$ 的链，$H_1$ 的为该链加 $D$，完全 prefix-comparable。因此不能通过引用“基线安全”排除这个例子。

结论是：在 early case，某个 Completed member 确实可能不产生所引用 Part I lemma 所要求的 vote。现有报告里没有一个可被直接调用的 Completion-aware lemma 来替代这一点。“with its unchanged treatment”没有给出该 lemma 的精确结论与适用前提。由于证明还要求覆盖 DATA/DRAIN、ancestor 与 receipt-based commitment，这一遗漏不能用“全局只导出 DATA”消除。这里需要作者补齐证明依赖或收窄命题；本审查不代写该分支。

按 proof-verifier 的依赖处理原则，暂时假定作者能建立所需 early-case continuation invariant 后，后续 recovery-rank induction 的结构是合理的；但该暂定前提不计入已验证结果。

## 实际完成的验证与限制

- 完成：五个快照文件 SHA-256 校验；完整模型 section 的精确比对；规范与证明的 source-grounded 人工审查；Force/regular、关闭后接受、同时间事件、固定 pair、Ready/Enter、callback、timer drain 和 early/late retirement 的局部推理检查。
- 没有完成：可执行协议实现测试、全状态 model checking、机械化证明、对 baseline DATA/DRAIN/Completion 和 native application validity 的独立重新证明、完整 liveness 或公平性认证。
- 结果边界：没有确认模型内 global DATA fork 或 retained-prefix rollback；也没有证明所有可能攻击均不存在。S1 在消除前，退休扩展的完整安全证明仍为未完成；S2 在澄清前，不能宣称历史验证与 cache 状态完全无关的接口已被具体指定。
