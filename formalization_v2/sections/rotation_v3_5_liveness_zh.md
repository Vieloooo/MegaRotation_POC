### 9.6 进展前提与稳定重试

以下结果使用冻结模型，以及本版明确规定的 retry、publication 和输入 handler 规则。证明全局接受时，不把“下一个必要 EXPORT 会在某个成功 service interval 内被接受”再次当作前提。初始 native state 恢复和 authority 可用性仍只有 eventual 保证，没有新增统一延迟上界。

v3.1 的进展段落明确说：**“The regional engine keeps its original fair-schedule, exclusion and uninterrupted-commit-window conditions.”** 因此保留 Part I 的精确条件：

\[
\forall g_0\ \exists g\ge g_0:\ L(g,0)\text{ 正确},\qquad
\forall g,t_0\ \exists t\ge t_0:\ L(g,t)\text{ 正确}.
\]

每个已暴露 equivocator \(p\) 都有有限 generation \(G_p\)，从此所有 \(g\ge G_p\) 的 leader positions 都不再分配给它。这里是 **generation indices**；排除改变未来 authority，不改写已 finalized 的 assignment。冻结 §2 对正确轮次必须 **actually entered** 的要求也保留。schedule 中出现一个正确名字，不等于该轮次已进入，更不等于其 DATA 已全局接受。

对于已安装的 slot \(r\)，不可变 schedule 写成

\[
L(\beta,r,g,t)=\ell_{r,g}(i_{\beta,r},t).
\]

改变 \(\beta\) 不重新生成 schedule。零输出 retry 读取相同的 \(\mathsf{AUTH}_{r,g}\)，但为新 pair 创建全新的 locks、votes、entry times 和 timers，初次进入仍为 \(g=t=0\)。后续 recovery 只能由这个新 pair 内的 Quit proof admission；旧 beta 的 Quit 不是新 beta 的入场证明。

为了反证，设 \(G\) 最终固定为 \(G^*\)。至多再有一次 Force 关闭“此前已经向这个 frontier 增加过正 DATA”的 duty。此后 zero-output 规则固定 \(r=r^*\)，相应 configuration、quota 和 base-window 参数也固定。持久 preparation 保存精确 anchor 的可执行状态、provenance 和验证依赖；authority fetching 保存缓存与未完成请求，不因 beta 更换而撤销。后台 authority 任务继续获取 \(r^*\) 的后续 indices，不在每次 Force 时退回零。

下文只对该已安装配置和既有有限 region cycle 取有限集合，不假设所有未来 slots 的成员并集有限，也不假设未来所有配置共享一个 \(\Delta\) 上界。

### 9.7 准备完成后的有限区域响应

**命题：有限 authority 前缀与区域响应。** 固定 \(r^*\) 及其中 region \(i\) 的 snapshot。在 fixed-\(G^*\) 尾部，存在有限 recovery index \(g_i^*\) 和有限准备完成时间，此后直到 \(g_i^*\) 的全部 authority 与精确 opening state 都可从持久缓存取得。对于之后启动且在所考虑区间内没有被外部 closure 打断的新 retry，在完整 ReadyBundle handler 契约下，从第一次正确 Enter 起，有限 \(R_i\) 内会形成并提供含有正 session DATA 的完整结果。

**证明。** 已安装区域 roster 有限。令 \(E_i\) 为此固定 authority namespace 中最终曾被揭露 admissible equivocation 的身份集合。每个 \(p\in E_i\) 的第一次暴露给出有限 exclusion generation \(G_{i,p}\)。所有 beta retries 使用相同 schedule，故该高代排除对每次 retry 都适用。定义

\[
\bar g_i=\max\bigl(\{1\}\cup\{G_{i,p}:p\in E_i\}\bigr),\qquad
g_i^*=\min\{g\ge\bar g_i:\ell_{r^*,g}(i,0)\text{ 正确}\}.
\]

有限 roster 使前者有限，继承的 initializer fairness 使后者有限。在任何 \(g\ge\bar g_i\) 中，不可能再揭露 admissible leader equivocation：否则签名者属于 \(E_i\)，却占据其 exclusion tail 已禁止的 leader position。较低 generation 或其他 pair 的 proof 不会冻结本 generation。低代 faulty assignments 可以在新 beta 重现，但 \(g_i^*\) 之前的 generation 数固定。

集合 \(\{\mathsf{AUTH}_{r^*,g}:0\le g\le g_i^*\}\) 有限。eventual availability 与持久 fetch 使它们在有限时间后全部到齐；固定 anchor 的 native preparation 和验证依赖也一样。对这些有限完成时间取最大值，不等于给 authority 或 state restoration 新增统一时限。缓存保存的是可直接附接到 fresh continuation 的可执行 anchor state；若每个 retry 仍需无界数据库 import，就不满足这条准备规则。

准备完成后，可采用以下保守区域界。设 \(Q=\mathrm{TB}_{r^*}\)、\(\Delta=\Delta_i\)，令

\[
D_i=(Q+1)(2K+12)\Delta_i,\qquad
R_i=(g_i^*+1)D_i+2g_i^*\Delta_i+9\Delta_i.
\]

初次正确 Enter 的 skew 至多 \(\Delta\)；authority 已缓存的 recovery 也一样。Part I 的 terminal-QC handoff 给普通 term-entry skew 至多 \(2\Delta\)。这些都是当前 pair 的 entry events，不继承旧时钟。

设 \(u\) 是某 term 的第一次正确 entry。除非此前已 Quit、Completion 或推进，其最后 progress check 不晚于 \(u+(2K+6)\Delta\)。opening 未知时第一次 check 仍执行，故缺 opening 不能无限抑制 BLAME。若某正确成员在此前投了 terminal proposal，完整 forwarding 与正确 votes 在再 \(2\Delta\) 内形成 QC，除非先发生 Quit：低于 endpoint 时推进 term；在 endpoint 时，再留 \(4\Delta\) 足以完成全部正确 commits 和 FINAL 聚合。若没有正确 terminal vote，就不可能有正确成员靠它的 QC 前进；所有正确成员末次 checks 产生匹配 BLAME，再 \(2\Delta\) 内全部冻结。equivocation 只能更早触发 Quit。因此每 term 的推进、冻结或完整结果三者之一在 \((2K+12)\Delta\) 内发生。

DATA generation 至多 \(Q\) 个 terms，DRAIN generation 只有一个。因此 \(D_i\) 界定一个 generation 到完整结果或首次正确 Quit 的时间。首次正确 Quit 后，proof 在 \(\Delta\) 内传播；下一 authority 已缓存时，全部正确成员在首次 Quit 后 \(2\Delta\) 内完成原 recovery wait 并进入下一代。对有限前缀逐代应用，便在 \(R_i\) 的预算内实际进入 \(g_i^*\)，除非早已形成正完整结果。这里是**证明 actual entry**，而不是把 schedule fairness 当作已入场的替代品。

在 \(g_i^*\)，initializer 正确且没有 admissible equivocation interruption。Part I 的 protected initial commit window 给出：从该 generation 第一次 entry \(e\) 起，first fresh proposal 最迟 \(e+5\Delta\)，全部正确 commitments 最迟 \(e+8\Delta\)，完整 FINAL receipt 最迟 \(e+9\Delta\)。若 fresh block 是 DRAIN，它完成的是同 session 的 terminal DATA receipt。因 \(Q\ge1\)，这仍覆盖 retry anchor 之后的正 DATA。Completion 只会更早满足完整结果结论。证毕。

### 9.8 从 opening block 到全局进展

**命题：准备完成的 public retry 响应。** 在 fixed-\(G^*\) 尾部，有限 preparation 和最后一次有效 health-control change 之后，设某 retry 的 opening block 首次 finalized 的实时为 \(a\)。对于模型覆盖的持续在线、跟随历史的参与者，存在有限

\[
L=\max_{i\in\mathcal I^*}\{2T_E+2\Delta_i+R_i+T_{\rm pub}\},
\]

使得最迟 \(a+L\)，要么 \(G\) 已增长，要么此前已有 effective Force 关闭该 retry。\(\mathcal I^*\) 是已安装 snapshot 中可能被选中的有限 region 集。

**证明。** 对上一命题的 preparation 完成时间取有限最大值，保留 verified replay checkpoint 和直接可执行 anchor 对象。opening block 及此前所有已 finalized predecessors 的完整 source evidence 最迟 \(a+T_E\) 可用；适用 controls 已在 crossing 前公告，稳定前缀依赖已缓存。规定的立即区域 SOURCESTEP relay 和有序完整 handler，在依赖齐全的同一事件中完成输入验证、P6 和 Manager，故第一次 Ready/Enter 最迟 \(a+T_E+\Delta_i\)，除非更新的已验证 closure 已禁止它。ReadyBundle 再用至多 \(\Delta_i\) 使全部正确成员进入。Ready 不再等待 opening 之后的空 Espresso header；这里没有无界 native installation 或任意离线 catch-up。

该步骤使用持续维护的 predecessor cursor 和完整 message-processing 契约，不给每个历史 height 额外串联一轮网络等待。完整 namespace 中缺必要材料的 candidate 由 record-completeness gate 拒绝，不能通过要求 arbitrary future evidence 暂停整段 replay；真正的 source history 缺失仍必须等待。

区域响应命题提供完整 next result；Publish Layer 和 Espresso 再增加 \(T_{\rm pub}+T_E\)。提交时携带精确 base \(G^*\) 及完整连续依赖。若另一 EXPORT 已改变 base，则 \(G\) 已增长；否则 base 仍正确，除非 effective Force 已关闭 branch，branch、authority、quota 和 reference 检查均通过。proposal 的 Espresso reference 早于其 EXPORT 所在 block，新的 forced version 也严格在 opening block 之后 exports。公式保守包含两次区域 relay allowance。若 Force 恰在结果所在 block 生效，它先于 payload 执行，属于 closure 分支；没有 cutoff 例外。证毕。

**命题：持续全局 DATA 进展。** 在冻结的 regional、source、delivery 前提及本版 retry、source-handler、health-controller 规则下，只要 continuing eligibility 条件保留一个非空、能在这些前提下服务的已安装 region 集，全局 DATA height 就无界增长。不要求同时有两个 eligible regions；也不给 initial restoration 或 authority wait 一个统一 failover 时限。

**证明。** 反设 \(G\) 最终固定为 \(G^*\)。至多一次 positive-duty Force 后，stable-slot 规则固定 \(r^*\)。health-controller policy 在不变 frontier 只允许有限次后续 matching MSIG 和 health-status changes；已提交 controls 仍按原 target、priority、first crossing 生效。越过这些有限 controls 并完成有限 preparation。

固定 governance 和 authority eligibility 后，早期 cooldown 在有限位置随增长的 Espresso timestamps 到期。有限 owed 集也稳定；若旧 round 已结束，下一个 boundary 从 governance-eligible regions 开新集合。非空 continuing eligibility 提供候选。条件 AUTO cooldown 不移除 owed 集中最后一个 eligible 候选：只有另一这样的候选存在才冷却 source。若 source 不在 owed 集，则已经存在 eligible owed 候选。因此这段有限前缀后，每次 AUTO 都立即赋予下一 duty；不会产生可反复补充任意实时 clock lag 的 AUTO 自造 unassigned interval。此论证不清除已有 cooldown，也不绕过 OFF。

将连续 retry openings 编号为 \(n=0,1,\ldots\)，其首次 finalization 实时和 timestamps 为 \(a_n,\theta_n\)。若某 retry 此后不再关闭，上一个响应命题使 \(G\) 在 \(L\) 内增长，矛盾。否则每次关闭都是 AUTO，同时就是下一 opening。所有 attempts 均为零 accepted DATA，所以一直在 startup phase。令固定 tolerance \(M_n=1+\beta_n\)，对应原 startup base-window 长度为 \(W_n\)。有限 installed 参数集给出 \(W_{\min}=\min_nW_n>0\)。结算已 crossing 的 base windows 得

\[
\theta_{n+1}-\theta_n\ge M_nW_n\ge M_nW_{\min},\qquad
a_{n+1}-a_n\le L.
\]

第二式由无增长反设下的 public-response 命题得到，**立即 reassignment 是必要的一步**。timestamp 跳跃可结算多个 misses，但只产生一个 Force，新 duty 从实际 triggering timestamp 开始。每个 effective Force 将 beta 加一，故 \(M_n=1+\beta_0+n\)，于是

\[
\theta_0+W_{\min}\sum_{n<N}(1+\beta_0+n)
\le\theta_N\le a_N+\epsilon_E\le a_0+NL+\epsilon_E.
\]

左边随 \(N\) 二次增长，右边线性增长，矛盾。只用了冻结 upper-lead bound；任何有限初始 timestamp lag 都进入常数 \(a_0-\theta_0\)，没有新增 lower-lag 或空块生成速率假设。因此 \(G\) 没有最后一个 accepted DATA height。证毕。

结论适用于满足既定 regional model、拥有必要正确 voting weight 和通信服务的 regions。一个物理健康却永久 OFF 的 region 不是 eligible candidate；本命题也不扩展到其他 assigned snapshots 永久失去正确 quorum 的模型。有限最大值是固定 frontier 反证中的 proof witnesses，不是所有未来配置共享的运营 SLA。

### 9.9 持续 eligible region 的有用服务

公共 round ledger 保存 owed 集 \(\mathcal O\)。round opening 时加入 governance-ON 且 designated-authority weight 为正的 region IDs；暂时 AUTO cooldown 不影响 debt membership。某 region 的 DATA 进入 \(G\)，或生效的 governance OFF／失去 authority eligibility 将其 prune 时，它离开当前集合。round opening 后才 ON 的 region 加入下一 round。只有旧 debts 全部清偿或 prune，并到达后续 duty boundary／selection 时才开新 round；一个有限 duty 内不能凭空产生无限 rounds。

**只有 Force selection 受 \(\mathcal O\) 过滤。** REGULAR 保留对全部 eligible regions 的历史 NextLive 选择。其新搜索下界是

\[
q_0^{\rm regular}=\max\{\operatorname{height}(\xi(T_{\beta,r})),\ q^{\rm open}_{\beta,r}+1\}.
\]

所以连续 accepted regular sessions 的 opening heights 严格增长。后来的 OFF 不改写已经固定的 regular witness。ledger 只限制下一 Force 选择，不撤销这些 witnesses，也不追溯修改已经打开 session 的 quota。

**命题：重复的 accepted service。** 假设全局进展命题的条件。从某个位置开始始终 governance-ON、具有正 designated-authority weight，且持续满足 regional serving 前提的 region \(C\)，会反复向 \(G\) 贡献 DATA。暂时 AUTO cooldown 不清偿这项义务。

**证明。** 先固定一个包含 \(C\) debt 的 round，反设 \(C\) 永不贡献。它的 debt 不能 prune。有限集合中的其他成员至多各清偿或 prune 一次；新 ON 不能加入这个未完成 round。

若 effective Forces 无限次，考察每个有 global output 的 Force-created version 的首个 accepted DATA。它必来自该 version 初选的 owed region：strict cutoff 拒绝旧版本输出，连续接受也不能跳过 forced session。这块首个 DATA 要么清偿该成员，要么该成员已在 selection 后被 prune。\(C\) 未服务时，这类成员移除只能有限次，因此不能有无限多个 productive Force-created versions。反之，若无限 Forces 中只有有限版本产生 DATA，则之后的 tail 没有全局增长，违反全局进展命题。因此无限 Force 情形必服务 \(C\)。

若 Force 只有有限次，最后一个 version 包含无界 accepted DATA。每个 session quota 有限，所以 accepted regular sessions 无限。其严格增长的 selection heights 最终越过 \(C\) 的 permanent-eligibility 位置和早期 cooldown expiry。cyclic NextLive 此后不能永远越过 \(C\)：扫描达到它时，它就是 eligible；其他 region 的 eligibility 变化不能使固定环跳过一个 eligible \(C\)。连续接受要越过其 selected session，必须先接受它的 DATA，仍矛盾。

故每个含此 debt 的 round 都会服务 \(C\)。对于刚变成 permanent-ON 的 \(C\)，考虑当时尚未完成的旧 round。如果其集合一直非空，有限 removals 后会留下稳定非空 residual set；无限 Force 分支会迫使其中再有成员清偿，有限 Force 分支则由 regular cyclic 选择到达一个 residual member。independent pruning 也减少集合，不会延长它。因此旧 round 结束。无界 DATA 与有限 session quotas 继续产生 duty boundaries，下一 round opening 纳入 \(C\)。每次贡献后重复论证，得重复服务。证毕。

这证明的是 accepted regional DATA service。empty DATA 也能清偿 region debt，所以不是交易 inclusion 证明。证明明确允许 owed 集外的 regular sessions；把所有 selections 都过滤成 owed-only 是另一个协议。

### 9.10 仍未解决的请求 liveness

全局增长与重复 region credits 仍不足以证明每个 admissible correct-client request 都进入 \(G\)。以下执行在 stable-slot 修复后仍成立。取 quota 至少二。在每个新 slot，Byzantine initializer 先产生一块合法 empty DATA 并使其全局接受，随后停产，由正常 BLAME 进入 recovery。把该新 slot 所需 \(\mathsf{AUTH}_{r,1}\) 延至 running phase 最后一个容忍失败窗口的 cutoff 前才提供。正确 recovery initializer 真正进入，并提出队首请求 \(x\)，但合法区域延迟使 DATA votes 发生在 crossing 前，完整 FINAL receipt 形成在 crossing 后。

例如 \(\Delta=1\)，recovery entry 在 \(d\)，两名必要正确成员的 DATA votes 在 \(d+4,d+5\)，commit timers 在 \(d+6,d+7\)。在 \(d+5.5\) crossing 就会先于完整 receipt 关闭版本。每个 authority 都在有限时间可用，模型却不给必须更早到达的上界。对任意有限 \(M=1+\beta\)，都可把该次 recovery entry 选在足够晚的位置。

该 duty 已接受前面的 empty DATA，所以 Force 合法推进到 \(r+1\)。下一 slot 带来全新的 authority wait，不是已经缓存的 slot 的 zero-output retry。让 correct initializer identities 和 selected regions 得到其要求的轮次，并反复执行。高代 schedules 仍满足 Part I fairness，不需要 equivocation；完整 late results 满足 publication bounds，\(x\) 始终保留在 pending queue。accepted empty DATA 持续增加 \(G\) 并清偿 region debts，而包含 \(x\) 的结果始终被 strict cutoff 拒绝。

因此完整 request target 仍开放。需要另行证明 accepted honest proposals，或提出有完整论证的 canonical request ingress/inclusion 规则。新的 request 或 READY 消息不会自动继承 complete-result publication guarantee。本节不宣称完整 request liveness，也不宣称整个 goal 已完成。

### 9.11 实现与验证范围

以上是规定的抽象协议论证。native adapter 必须保留可直接执行的 anchor state，SOURCESTEP／ReadyBundle handlers 必须满足既有完整 message-processing 契约；它们没有认证任意数据库 import、archive scan、离线 catch-up 或具体 Espresso profile 的性能。新的 retry、window、cooldown、health-policy 与 debt transitions 仍须结合不变 strict cutoff 和 retirement 规则接受独立整体验证。上述区域与全局进展结果明确保留 request obligation。
