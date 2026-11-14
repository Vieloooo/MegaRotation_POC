# Part III v1：协议与资金生命周期严格审查

审查对象：`formalization_v2/part3_staking/versions/part3_v1.tex`，当前 Version 1，2026-10-19。依赖核对：当前 `sync_hotstuff_rr_part2_rotation_v4_5.tex`；恢复排除所需处核对 `sync_hotstuff_rr_part1_core.tex`。本次仅作 verifier，不修改原稿，不替作者补证明。行号指本次读取的源文件。

## Summary

**Final Verdict：本次限定范围内未发现已被反例证实的 Critical Error；机制可行，但有两个需补齐的 Justification Gap：强制退役的退出记录，以及新 authority 中 quota/BFT 参数的公开唯一来源。补齐前，不能把资金生命周期与 §6 的完整确定性表述视为已核验完毕。**

不要求本文证明抗 grinding、公平抽签、无条件活性、无条件提款，或经济激励足以自动带来举报。这些均不是文稿所声称的结论。Part II 的 BFT 与外部信任假设在本次按其陈述使用，本报告不重新认证 Part II 的全部证明。

### List of Findings

1. **Justification Gap — 强制退役未明确生成 §5 所需的退出记录。**
   - Location：v1 行 185–189：“Positions below the minimum bond are omitted and retired from future snapshots; associated queue requests are consumed.” 对照行 172–173：“Selection records an exit epoch $e_x$”，及行 300–301：“supply its exit record and proof of an effective Force whose installed minimum is at least $e_x$.”
   - Issue：自愿出队明确记录 `e_x`，低于最低 bond 的强制退役只说明未来排除、消耗请求，没有明确记录首次永久排除 epoch，也没有另一条正余额提款入口。后文安全边界需要的状态并未在此分支产生。
   - ≤6 页稿是否必须补：**是。** 这是生命周期的必要状态转换，不是额外活性定理。
2. **Justification Gap — 新 authority 的 quota 与 BFT 参数没有明确的公共唯一来源。**
   - Location：v1 行 67–68：“Session authority & The plan, exact execution anchor $A_s$, handoff evidence, quota and regional BFT parameters.” 行 342–349：“Authorities are now keyed by $(\beta,r)$ and explicit configuration ID ... Quota and voting parameters remain fixed within the session.”
   - Issue：快照与 branch policy 已给委员会、权重、候选次序、最低 epoch，但没有给 `TB_s`、`F_i`、`Delta_i`、`K` 的确定来源或公共映射；“固定”不等于所有验证者获得同一个值。旧 Part II 对这些值的来源是 `AUTH_{r,0}`，这里替换了 authority 体系，不能无说明地继续假定旧记录解决问题。
   - ≤6 页稿是否必须补：**是。** 只需明确参数来自哪个已认证公共对象/固定规则；无需扩写 BFT 引擎。

## Detailed Verification Log

### 1. 快照固定、采用与余额分离

> 行 149–154：“Snapshot $e$ activates at $a_e=a_0+eM$ and has cutoff $c_e=a_e-P$ ... reads the complete L1 state strictly before block $c_e$ ... Late materialization cannot move the cutoff or reorder requests.”

固定 cutoff 与读取 cutoff 前的完整 L1 状态能使快照内容不受后来的构造时间影响。规定缺失快照则等待，没有授权降级到旧快照。行 156–159 区分请求受理、快照冻结、session 采用，也明确不能重写已固定的 plan。这部分与后面的历史权重核验相容。

> 行 166–167：“Once included in a frozen snapshot, its escrow cannot be refunded through the pending-request cancellation path.”
>
> 行 173–175：“It must still serve old assignments ... does not itself authorize withdrawal.”

已经被快照采用的入队不能借取消请求退款；自愿 exit、快照省略、单纯经过 epoch 都不能提前释放 escrow。这些限制堵住了最直接的“已取得责任但先退款”路径。

> 行 185–190：“At the cutoff, included positions receive their remaining positive balances as weights ... A later penalty reduces escrow, not a frozen denominator.”

`B_o` 是可扣款余额，`w_o` 是历史证书的固定验证权重。二者分离是必要的，也与 Part II 行 488–493 的 fixed-snapshot/accepted-lifetime 假设一致。快照冻结后扣款不改旧 QC 和 FINAL 的分母。本文没有证明剩余经济质押仍对应固定权重下的诚实性；行 199–204 与 386–387 已明确该限制。

### 2. 强制退役的状态转换：Justification Gap

> 行 186–187：“Positions below the minimum bond are omitted and retired from future snapshots; associated queue requests are consumed.”

该句可以确定“以后不再入选”，却不能确定“合约拥有 §5 要求的 exit record 与 `e_x`”。`e_x` 的唯一显式写入规则位于自愿 exit 被队列选中的段落。

最短问题轨迹：某 position 已在旧快照承担责任，余额从 100 被扣到 5，而最低 bond 为 10；它还有一个尚未达到 exit delay 的退出请求。在 epoch `e` 的 cutoff，它被强制退役，该请求也被消耗。按自愿出队条件，它尚未被选中，故未由该规则产生 `e_x`。即使随后存在所有外部服务、有效 Force 且其最低快照已经超过 `e`，§5 仍要求一条本文未规定怎样产生的 exit record。这个例子没有要求无条件提款，只展示一个正常状态分支缺少后续入口。

如果作者意图是强制退役也自动记录首次排除 epoch，则正文需要明确。不能在 verifier 模式下自行补入该状态转换。后续退出安全论证在“已存在有效 `e_x` 的位置”上仍可独立继续核对；对这一强制退役分支的完整性暂不验证通过。

### 3. 自愿队列的边界

> 行 178–183：“Select each FIFO's longest mature prefix fitting its budgets ... every admitted whole position to fit a fresh budget ... No regional stake floor may veto an exit indefinitely.”

在固定的请求顺序、成熟规则、预算与请求数上限作为公开参数时，队列选择是确定的；分开计入流入和流出避免净额抵消绕过变动限制。文稿没有把“positive budgets”直接提升成无条件的时间界，而在行 204 保留持续 L1 处理条件。

**可接受的明示限制：** 未证明有限墙钟时间内每个 exit 都完成，不构成缺陷。资金最终释放还依赖 multisig Force；行 381–383 已准确声明。

### 4. Slash predicates、扣款与费用前提

> 行 209–212：“All signed evidence binds ... historical position and configuration, region, session, and its initial execution anchor ... Signatures are checked against that historical authority, never the current balance.”

链/协议域、历史 position、配置、session 与初始 anchor 的绑定避免跨链、跨 branch 或不同 incoming context 拼接证据。必须按历史配置验证，不能因为被罚后余额降低而否认旧签名，也不能因后来退出而改写原证书。

行 216–225 的五种 offense 与其声称的单签限制一致：

- 同一完整 fresh slot 的 DATA/DRAIN leader 冲突，使用 Part II 的 admission/quota 证据；Part II 行 1432–1439 明确超 quota DATA 或非 drain generation 的 DRAIN 不计入该 quit predicate。
- 同一 fresh slot 的不同 block vote，DATA/DRAIN 共用一票限制；符合 Part II 行 906–908。
- 同一 recovery generation 的不同 NEWVIEW opening，祖先/后代也属于不同 opening；符合 Part II 行 1435–1444。
- 同一 generation 的不同 ANCHOR opening vote，违反单 ANCHOR vote 限制。
- 同一 session、同一 DATA height 的不同 FINAL hash；FINAL 不含 `g,t`，故跨 generation/term 不能逃避这一 conflict slot。

这里“不同 proposal”应继承 Part II 行 1099–1101 的 block identity：parent QC 的 signer set 或编码变化本身不是新的 proposal。本次把该已引用的 Part II 定义作为依赖使用，不另报一个新 defect。

> 行 228–230：“Timeouts, BLAME, OFF status and forwarding evidence are not these offenses ... Reusing a height on a different branch is not by itself a conflict.”

未把缺席、超时、被关闭的 speculative suffix 或单纯转发当作有罪证据。这一点与 Force 不保证 discarded suffix 区域一致性的限定相容。

> 行 235–240：“atomically records the offense, debits a published penalty fraction of the historical bonded weight capped by remaining escrow ... Deduplicate by position, offense type and conflict slot ... Total debits cannot exceed escrow.”

给定公开 penalty 参数及其实现数值约定，该规则使用历史 bonded weight 计算、以当前 escrow 截断，并原子记录 offense，足以防止同一 conflict slot 因更换证据对编码而被重复扣款。不同 offense type 各自处罚不违反本文的任何“每个行为只罚一次”承诺，因为本文没有该承诺。多项有效处罚按 L1 顺序执行，也不造成总额超过 escrow。

**可接受的明示限制：** 本文没有 reporter reward 或 gas 补贴模型，不应据此声称经济激励保证举报。`T_ev` 直接假定证据检查、收集和实际提交有界，`T_L1^stake` 再假定有效 slash 交易的服务有界；所以成功提交路径及所需费用资源必须属于这些服务前提。Part II 原服务假设同样明确需要工作中的提交路径与费用资源（行 562–567）。只要保持本文的条件化表述，无需在六页内新增博弈或激励分析。

### 5. REC 与 proposer exclusion

> 行 245–257：“complete quit proof ... at most one immutable record ... $J_g$ is exactly the banned-key set in the contract immediately before that transaction ... $J_g$ changes proposer eligibility only.”

REC 的唯一 key 为 session/generation，并且引用完整 authority、前一 generation 的有效 quit proof。这样不是凭裸计数器预占任意 future generation。L1 顺序唯一确定其 ban 集合，finality 后该 generation 不再受后来 slash 到达时间影响；相同记录的读者获得同一 proposer schedule。

“immediately before that transaction”也意味着 REC 先于某 slash 时不会包含该新 ban。文稿没有承诺当场重写已有 schedule，因此这不是错误。行 259–262 对初始 generation 使用 snapshot cutoff 时的 exclusions，同样是明确的时间边界。

> 行 264–267：“eventual exclusion if valid proofs reach L1 and later recovery records are created ... does not prove fairness ... separate assumptions.”

**可接受的明示限制：** 这里的 later records 必须是 slash 已被处理以后创建的记录。固定 session 的后续此类记录会排除已 ban 的 key；过去冻结的记录不被重写。没有 eligible proposer 时等待外部恢复，是已声明的停顿条件。此机制没有证明 Part I 的 schedule fairness，也不把 Part I 的永久身份集合假设扩展成跨新注册身份的全局女巫抵抗；本文没有声称这些结论。

### 6. 最低 epoch 与有效 Force 的退出边界

> 行 276–289：“minima never decrease ... A stale update changes none of them ... Every new authority, including a Force bootstrap, obeys $e_s\ge m_\beta$ ... The same Force closes all older running and prepared sessions.”

对已拥有明确 `e_x` 的 position，这个论证的两部分都必要且相容：有效 Force 关闭旧 branch，故旧的 running/prepared/speculative session 不能继续产生被该链接受的新权限；新的 branch 使用不低于 `m` 的 epoch，且 minimum 以后只升不降，故在所有 `e>=e_x` 都缺席的 position 不会再被未来 authority 选中。单有其中一项均不足，原稿同时提供了两项。

Part II 行 2555–2588 对 stale target、branch closure、零输出 reuse 与 speculative successor 的归档支持这一论证；行 2599–2604 还明确 closure 可以先于目标 authority 的可用性固定。因此新 branch 暂无可用目标本身不撤销已经生效的退休边界。

> 行 300–308：“verify the complete applicable L1 controls and Espresso history through that Force's activation, including the fact that its target was current ... not mere Espresso inclusion of a claimed result.”

不是只提交一条已上 Espresso 的断言，而是验证该控制确实在 canonical replay 中匹配当前 branch 并执行，能排除把 stale Force 当作提款依据。检查点需是 Part II 意义下可认证、可继续正确 replay 的状态；本段没有去除该继承前提。

**可接受的明示限制：** Force 可丢弃 `G` 之外的 speculative suffix；multisig 与外部服务不行动则不能得到有限退出边界。行 381–385 已说清这两项成本，无需把它们升级为协议错误。

### 7. 证据窗口的条件性与时间加总

> 行 311–324：“Start the holding period when L1 accepts that verified boundary, at time $t_c$ ... $T_{\rm hold}\ge 2T_E+T_{\rm pub}+T_{\rm ev}+T_{\rm L1}^{\rm stake}+\varepsilon_{\rm clock}$ ... The existing bounds alone do not supply $T_{\rm ev}$.”

接受已验证 activation 的时间 `t_c` 不早于该 activation 的现实发生时刻。从 activation 起计算的可见性余量，以 `t_c` 为起点计时只会更保守。对满足额外 evidence-service 假设的相关材料，closure 可见性 `T_E`、可用 complete result 的提交与可见性 `T_pub+T_E`、额外收集核验提交 `T_ev`、L1 slash 服务 `T_L1^stake` 串联相加，给出的窗口不漏这些已列环节。时钟换算另有部署 margin，且提款必须严格晚于 deadline。

这不是从 Part II 单独推出“所有 slash 证据都可在窗口内发现”。Part II 的 `T_pub` 从完整结果已经可用开始，不界定签名/证据何时首次形成或可收集；并且 discarded session 的 equivocation 也不必存在完整 FINAL receipt。正文通过单列 `T_ev` 和行 328–330 的限定承担这一缺口，未把它隐瞒为已有结论。

> 行 326–330：“after all valid proofs accepted by that deadline have been applied in L1 order ... No fixed window catches evidence hidden forever or unbounded offline investigation ... not a claim that every offense will eventually be paid.”

**可接受的明示限制：** 有界且可用的证据服务保证的是符合该服务前提的及时可执行索赔在 escrow 释放前得到处理，不覆盖无限隐藏或任意晚出现的旧 key 证据。这种限制与正文完全一致，不能要求作者再证明其已否认的无条件 accountability。

### 8. §6 确定性论证的参数来源：Justification Gap

> 行 342–349：“Authorities are now keyed by $(\beta,r)$ and explicit configuration ID, not by $r$ alone ... Quota and voting parameters remain fixed within the session.”
>
> 行 351–357：“Why the rules are deterministic ... None of these choices uses receipt arrival order or a reader's latest local head.”

快照选择、candidate order、positive-weight region、canonical weighted choice、REC 排除集合这几个已定义的选择步骤是确定的。但是 session authority 还包含 quota 与 BFT 参数。v1 行 61–68 的对象表没有规定这些参数由哪个公共规则/记录赋值；正文也只陈述其固定性。旧 Part II 行 335–344 的来源是按 `r` 索引的 L1 `AUTH_{r,0}`，它明确携带 `TB_r,F_i,Delta_i,K`。新的派生 authority 体系需要显式接上这一来源或其替代，而不能只声称“保持固定”。

最短不确定性例子：同一公开快照有 `W=100`，在保持 `F<W/2` 的情况下，一个读者取 `F=20`，另一个取 `F=40`，总权重 30 的同一 FINAL signature set 就会得到不同判断。类似地，两种固定 `TB_s` 会给出不同 terminal height，从而不同 handoff。这不是说作者实际允许读者任选，而是现有文本没有给出排除此二义性的公共赋值规则。

该缺口的下游影响是完整 authority、terminal boundary 与 FINAL checker 的一致性暂未验证通过；它不推翻前述最低 epoch 的排除逻辑，也不推翻在共同已固定参数前提下的条件退出论证。

### 9. §6 其余保证与成本

> 行 359–375：“Before the verified boundary, the contract keeps the position locked ... minimum and branch closure prevent an old key from reopening authority after withdrawal. Data needed for historical validation must still be retained or transferred under Part II.”

对已定义 `e_x`、共同参数和可靠 replay 的位置，锁仓至有效边界、再保留证据窗口、历史证书权重不变这条链条成立。这里关闭的是旧 session 的生产/投票权限；历史数据保存或可靠转交义务仍由 Part II 保留，不能把退出误读成可立即销毁所有验证数据。原稿最后一句已经显式保留了这一义务。

> 行 379–387：可预测 schedule、snapshot 重用、慢 session 延迟采用、multisig/external services 的条件退出、speculative suffix 丢弃、恢复时 L1 等待、slash 后仍用 frozen weight。

这些是可接受的显式 scope 与成本。没有发现需要为了六页论文而追加抗 grinding 公平性、全局无条件进展或新的社会 checkpoint 机制的理由。

## 修订必要性结论

本次范围内需在正文补齐的是 **强制退役如何产生退出记录** 与 **quota/BFT 参数的公共唯一来源**。其余所核对部分应保留现有条件和成本，不应为回应审查而不当地加强保证。本报告没有提供替代协议或修订文本，也没有修改被审稿件。
