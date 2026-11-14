# 固定 authority slot 的 retry：可证明部分与剩余时钟阻碍

2026-10-18。独立设计分析，不是最终安全审查；只写本报告。有效 Force 仍在既定 first crossing 生效、检查 old beta、立即关闭旧版本；不接受 cutoff 后的旧版本新增 DATA。没有修改 frozen model、协议、AUTH 或已有证书。

## 1. 结论先行

**按完整 frozen source 的继承关系，固定 G 时保留同一 r、缓存同一 `(r,g)` 的未来 authority、每个新 beta 仍从本地 `g=t=0` 重启，足以消除“不断换 r，所以总有一个全新 AUTH 在 deadline 前才出现”的 authority 障碍。无需为了这一点引入跨 beta 的 recovery lock、旧 Quit admission、新的 leader-screen 或可变 AUTH。**

这个结论依赖现有的 Part I generation-tail schedule fairness 与 exclusion，而不是仅凭 Model 节的简写。它导出的是：固定配置、状态与有限 authority 前缀准备好以后，**不被外部 closure 打断的**新 attempt 在某个有限、执行相关的时间内形成首个完整正 DATA result。它不单独证明 current v3.4.1 的端到端 liveness。

尤其是：现有 Ready 的后继 Espresso header 要求仍需修复；而此前 proposed cumulative AUTO-budget 证明还漏了 **unassigned cooldown 间隔**。仅有上界 `theta <= t_final + epsilon_E` 时，长期无 duty 的等待可以反复补充时钟滞后，使任意大的 AUTO timestamp 预算仍在下一 attempt 的 receipt 形成前用一个跳跃耗尽。第 8 节给出具体构造。

## 2. 本报告采用的源文与继承关系

记：

- `M` = `revisions/v3_1_frozen/model_and_assumptions.tex`；
- `F` = `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex`；
- `I` = `revisions/v3_1_frozen/sync_hotstuff_rr_part1_core.tex`；
- `V` = 当前 `sync_hotstuff_rr_part2_rotation_v3_4_1.tex`。

关键原文不是只有 `M:297–300` 的短句。

1. `F:60–71`：每个 region 运行 Part I engine；regional DATA/DRAIN engine、timers、recovery rules 保留。
2. `F:957–993`：Protocols 1、2 是 Part I 的同名协议加逐项列出的 changes；这份列表没有删除 schedule fairness 或 exclusion。
3. `F:2104–2105` 明文：**“The regional engine keeps its original fair-schedule, exclusion and uninterrupted-commit-window conditions.”** 当前 `V:2667–2668` 保留同一句。
4. `I:210–211` 的原始量词是

   \[
   \forall g_0\ \exists g\ge g_0:\ L(g,0)\text{ correct},\qquad
   \forall g,t_0\ \exists t\ge t_0:\ L(g,t)\text{ correct}.
   \]

5. `I:187–201` 给每个已暴露 identity 一个有限 exclusion generation `G_p`，满足 `L(g,t) != p` 对所有 `g >= G_p,t >= 0`；并明确 `g_bad,G_p` 是 **generation indices**，future authorities 生效且不改写 finalized assignment。
6. `F:963–970`、`M:30–44` 明确 authority 只按数字 `(r,g)` 索引；另一个 beta 复用 r 时读取同一不可变 authority。session engine 由 `(beta,r)` 隔离。

因此，完整 source 的自然严格读法许可在固定 r、固定 region i 上继承上述 generation-tail 条件。Model 节强调“actually entered”并不删除上述更精确的 schedule 条件；前者防止把名义上的正确 leader 当作已取得服务，后者提供可达的未来正确 initializer。证明必须同时使用二者的正确角色。

**不应另外发明更强的排除语义：**“p 曾在某个 beta 暴露，所以以后任何 beta 的任何旧 AUTH 都不得再指派 p”不是 `I:199–201` 的 generation-index 规则。重复执行低 g 的不可变旧 schedule 与“所有足够高 g 永久排除 p”可以同时成立。高 g exclusion 不重写旧 assignment。

需要在新稿正文显式写出一个已有语义的展开式，避免实现误读：

\[
L(\beta,r,g,t)=\ell_{r,g}(i_{\beta,r},t).
\]

这里同一个 AUTH 的 schedule 不能暗中依 beta 选择全新的一套 leader。若允许 arbitrary beta-dependent schedule，缓存相同字节并不能给出固定恢复前缀；那会偏离“same immutable authority/leader schedule”的通常含义。

## 3. 可直接具体化的 retry 状态与 admission

有效 Force 的控制检查、事件顺序、保留 G、beta 加一和旧版本拒绝全部保持。只修改选择下一个数字 slot 的规则：

\[
r' =
\begin{cases}
r,&\text{assigned duty 的 accepted\_in\_duty}=0,\\
r+1,&\text{assigned duty 已有正 accepted DATA},\\
r_{\rm pending},&\text{当前 unassigned}.
\end{cases}
\]

REGULAR 仍使用 `r+1`。Force anchor 永远是触发 block payload 之前的精确 G。source region/cause/source context 与 target slot 是不同字段：保留 r 不等于把 source region 留在原地；仍可按现有 cyclic selector 选择其他 region。重复 unassigned Force 保留已确定的 pending target slot。

handoff witness 增加可重算的 zero-output/new-slot 分支说明。`VerifyHandoffHistory` 必须按该分支核对 target r，不能继续无条件要求 `r'=r_b+1`（当前 `V:1907–1914`）。源 context 不得为了得到相同 r 而伪造为 `r_b-1`。

每个新 `(beta',r')` 的 admission 保持现有类型：

\[
\operatorname{Admit}(\beta',r',0,0)
\iff \operatorname{Ready}(\text{exact new handoff})\land\operatorname{CanEnter}.
\]

- 安装同一个 `AUTH[r',0]`、固定 cfg/quota；anchor 等于新 handoff 的 G。
- 新建 `Q^0_{beta',r'}`，它表示继承 anchor 的**原始 commitment evidence**；不是旧引擎的 entry lock。
- 新建全部投票、锁、entry flag、timers、Quit/NEWVIEW/STATUS 状态；`g=t=0`。
- `g>0` 仍只由**这个新 pair 内**的 `g-1` Quit proof 与 `AUTH[r',g]` admission；新 NEWVIEW 的 lock collection 也只读该 pair 的 QCs。
- 旧 pair 的 equivocation/authority/state 材料可保留为证据或缓存；旧 Quit 不能充当新 pair 的 recovery admission，旧投票不能形成新 QC。

这与 `V:899–921,1011–1013,1220–1235,1999–2016` 的隔离结构相容。它是明确的 target-index 改动，不是单纯缓存优化。因为 `TB_r` 不变，零输出重试在同一 G 上得到同一个 endpoint `h(G)+TB_r`；没有在同一 session 内重置 quota。

## 4. 为何固定 r 有有限可用 authority 前缀

在反证用的 fixed-G infinite tail 上，至多经过一次“当前 duty 曾有正输出”的换 slot，此后 r 固定。

对固定 r 和 region i，使用该安装配置的有限 roster `V_i(cfg_r)`。这来自 Part I 的有限 committee 及 Part II 的固定 session snapshot；**不需要所有未来 r 的成员并集有限**。同一 r 的重试不引入新 roster，既定 fault bound 也覆盖该 snapshot 在证书接受期被攻破的 key。

令 `E_i` 是这条 tail 上、同一固定 schedule namespace 中最终曾被暴露的 Byzantine identities。`E_i` 有限。对每个 `p in E_i`，第一次暴露所在 engine 的 inherited exclusion 给出有限 `G_{i,p}`。因为所有 beta 使用同一 `ell_{r,g}(i,t)`，这个 schedule 的高 g exclusion 也适用于其他 beta。

取

\[
\bar g_i=\max\bigl(\{1\}\cup\{G_{i,p}:p\in E_i\}\bigr),\qquad
g_i^*=\min\{g\ge\bar g_i:\ell_{r,g}(i,0)\text{ correct}\}.
\]

第一项有限是有限 roster + eventual exclusion 的结果；第二项有限来自 inherited `forall g_0 exists correct initializer >= g_0`。不宣称任何公开已知或统一上界。

在任意 beta 的 `g >= bar g_i`，不能再出现会被正确节点揭露的 admissible leader equivocation：否则其 signer 属于 `E_i`，却被安排在已禁止它的 generation，矛盾。旧 generation 的 proof 只归档，旧 beta proof 也不能冻结当前 pair（`V:1293–1298`）。因此 `g_i^*` 是一个正确 initializer 且不存在有效 equivocation interruption 的 generation。

这一论证允许每次重试先重放有限个低 g 的 Byzantine initializer。静默导致新的本 session BLAME/QUIT；equivocation 也产生新的本 session Quit。它们耗费有限前缀的时间，但不会制造无穷多个新的 authority 名称。

有限集合

\[
\mathcal A_i^*=\{\operatorname{AUTH}_{r,g}:0\le g\le g_i^*\}
\]

的每个成员都有有限 availability time。`I:175–180` 还明确 authority 服务独立于 L2 commitment；只需最终到达，不需 bounded authority availability。其最大可用时间是有限的。这不是把“每个等待有限”错误升级成“所有未来等待有统一上界”：我们只取一个已经证明有限的前缀。

实现上需要 `AuthCache[(r,g)]` 与尚未完成的 fetch 属于 Session Manager/共享缓存，不随某个 beta 的取消而删除。可以保留一个**仅用于取证、不是 engine schedule**的顺序预取 cursor：获取/验证 r 的下一个 AUTH，缓存后推进 cursor；固定 r 的后台 worker 不因 Force 重启。每个有限前缀会在有限时间内完成。每次新 Enter 可把已有前缀一并放入完整 validation bundle，或在区域内提前传播。不能因缓存当前 g 的 authority 而丢弃已知未来 authority。

对于多 region，固定 cfg_r 和已有有限 region list 只产生有限多个这样的前缀。若证明目标适用这些 region 的完整 frozen regional premises，可取有限并集/有限最大值。不能借此声称“任意未来配置”有同一个 Delta，也不能把永久失去正确 quorum 的 region 当成仍满足这个 frozen model。

## 5. 从有限前缀到实际 Enter 与首个完整结果

下面的时间界从正确节点的**实际 Ready/Enter**开始；不是从 public assignment 或 candidate creation 开始。假设目标 handoff 在区间内始终可进入，现有 ReadyBundle 的完整 handler contract 成立，且没有外部 retirement。则正确成员的 initial entry skew 不超过 `Delta_i`（当前 ready-bundle fragment 与 `V:2022–2031`）。

选一个晚到足以使固定状态、所需历史及 `A_i^*` 已准备好的 attempt。由于所有所需 authority 已缓存，recoveries 不再包含不受界的 L1 等待；仍执行原来的 Delta recovery wait，不能省掉它。

设 `Q=TB_r`、`K>=1`、`Delta=Delta_i`。一个 generation 的 DATA terms 不超过 `ceil(Q/K)`，保守地不超过 Q；DRAIN generation 只有一个 term。对当前 admitted term 的第 p 个位置，原始检查仍在

\[
e_v+(2p+4)\Delta.
\]

有限 term 进展的证明分支必须写成：

1. 如果某正确成员已投 terminal proposal，完整 forwarding 与正确投票在有界时间内使终端 QC 可得；进入下一 term，或在 quota endpoint 等待 receipt。
2. 如果到最后检查仍没有 terminal proposal 的正确投票，所有尚停留此 term 的正确成员发 BLAME；传播使匹配的 correct weight 形成 Quit。已离开该 term 的成员则通过它持有的 terminal-QC history 使其他成员推进，不能无限隐藏这个推进。
3. 任意 admissible equivocation 可更早引起 Quit；它不能把这一“推进、Quit、或 complete result”三选一延长到无限。

一个可用但须在新稿中逐事件核对的宽松常数是

\[
D_i=(Q+1)(2K+12)\Delta_i.
\]

它为一个 generation 中至多 Q 个 term 的 progress checks、entry skew、QC/BLAME forwarding、终端 commit/FINAL 留出裕量。每次 Quit 后、authority 已缓存时，所有正确成员的下一 generation entry 在首次正确 Quit 后 `2 Delta_i` 内完成：proof 传播最多 Delta，再执行各自 Delta wait。这里没有靠“Byzantine leader 很快行动”来取得界。

因而可采用一个待逐项验证的显式上界

\[
R_i=(g_i^*+1)D_i+2g_i^*\Delta_i+9\Delta_i.
\]

从 first Enter 起，若此前还没有完整正结果，attempt 在这个有限时间内到达 `g_i^*` 并形成完整结果。最后 `9 Delta_i` 来自正确 initializer 的 first fresh proposal 最迟 `e+5 Delta_i`、全部正确 commit 最迟 `e+8 Delta_i`、FINAL 再传播一次。若是 DRAIN，则它完成同 session 的 terminal DATA receipt；`TB_r>=1` 保证这仍是相对于固定 anchor 的正 DATA result。

**对 actual fairness 的正确使用：**仅说 schedule 中有 `g_i^*` 不够。有限前缀推进 lemma 加上足够长且持续授权的执行区间，才证明这个 initializer 实际 Enter。随后使用 Part I 的 protected commit-window lemma，而不是把“actually entered correct turns”本身当作能赢过 AUTO 的结论。

`R_i` 的存在性有清楚的证明路线，但上述宽松系数不是已完成的机器检查结果。尤其要核对 generation/term entry-skew lemma、late terminal-QC handling 和 ReadyBundle 的 native adoption 前提。不能把 native import、任意历史追赶或 required authority 等待塞进 Delta。

若 next positive result 的 EXPORT 用精确固定 G 为 base、依赖完整且没有另一 accepted append，则 raw publication 不会因为 stale base 被拒；形成完整 result 后再加 `T_pub+T_E`。于是从 first Enter 到可被当前 open beta 接受的候选界为 `R_i+T_pub+T_E`，前提仍是截止前 beta 未关闭及 reference/admission 规则满足。不存在一个无条件从 public opening 到 acceptance 的已证常数。

## 6. 保持不可变窗口参数的 AUTO 具体候选

可沿用 `strict_cutoff_next_design.md` 的 failed-base-window 容忍计数：AUTH 中的 `W_start,W_run,m_start,q_min` 不改写；公开 replay 另保留不随 DATA/REGULAR 重置的 effective-Force counter nu。

\[
M(\nu)=2^\nu,\qquad
\text{AUTO 当且仅当当前 phase 连续失败 base windows 数达到 }M(\nu).
\]

每个失败 base window 仍在原 first crossing 结算；仅改变何时失败次数授权 AUTO。真正触发 AUTO 后仍立即 cutoff，不能把已提交 MSIG 变成排队等待。固定 G、固定 r 的 fresh startup duty 若始终没有 accepted DATA，累计 timestamp allowance 为

\[
B_n=M(\nu_n)W_{\rm start}>0.
\]

这不是当前 v3.4.1 已有的规则；它是明确的检测策略变更，也不等同于修改固定 AUTH 的窗口长度。有限 authority 前缀只解决它所需要的 regional-response 子引理，不自动完成时钟论证。

## 7. 已撤回的弱模型反例及正确分类

为了说明不能只读 Model 节简写，考虑固定 `W=5,F=2` 的 roster `{H0,H1,H2,B_eq,B_silent}`，`K=1,TB=4`。g=0 的 initializer 静默；g=1,2,3 分别由 H0,H1,H2 初始化，每次仅增加一个 certified DATA，然后快速轮到 B_eq，它在 commit 前 equivocate；g>=4 一律 B_silent。每个 beta 都重复这条前缀，全部 authority 已缓存。

三个 honest initializer 可借 Byzantine votes 快速形成自己 DATA 的 QC，再由 Byzantine successor 的完整双签证据在 `2 Delta` timer 前冻结；三次 recovery 后锁只到 DATA 3，剩余 quota 1，所有更高 generation 静默。每个 honest identity 实际担任 initializer/produces DATA 无限次，B_eq 从 generation 4 起也被排除，但没有 FINAL。

**该 trace 不是 full-source 模型内反例。**它违反 `I:210` 的高 generation initializer fairness；若其固定 generation 的全部后续 terms 也静默，还违反 `I:211`。它只能反驳把 inherited assumptions 错删、仅剩跨 beta 实际公平性的弱化版本。此前向主任务发出的“固定 r 缓存仍不足”早期结论应据此收回。不能为了阻止此 trace 增加新的 leader-screen、cyclic AUTH 或 skip admission，因为完整现有前提已排除它。

同理，“后来任一 beta 不得重放低 g 已暴露 leader”的强 chronology 解读与 Part I 明确的 generation-index 说明不一致；它至多是术语展开需要澄清，不能被当成另一条已经冻结的更强规则。

## 8. 真正仍需修复的时钟问题：无 duty 的 cooldown 间隔

有限 authority 前缀不能让下述一步自动成立：

\[
\text{failure of attempt }n\Longrightarrow t_{n+1}^{\rm open}
\le t_n^{\rm open}+L.
\]

即使从进入到 FC/publication 有统一 L，这最多约束当前 attempt 的 **closure time**；AUTO 之后可能 unassigned 等待 cooldown，下一 opening 的 real time 没有由 L 约束。Espresso 对已 final blocks 的 visibility 有界，不等于它会在固定实时时间内生成一个超过 cooldown timestamp 的新 block。

以当前 selector 明确支持的单一 eligible-region renewal 为例。所有 authority、fixed-G native state 都已缓存；committee 不再干预。记 attempt n 的 opening timestamp 为 `u_n`，AUTO allowance 为任意有限 `B_n`，cutoff timestamp 为 `c_n=u_n+B_n`，cooldown 为固定 `C>0`。

1. 在 previous cutoff 后，没有 assigned duty。可以立即按服务界 raw-publish 已关闭 session 的 late FC，同时让这些 block 的 timestamp 保持 `c_(n-1)`。这些记录不能增加 G，也不会提前结束 cooldown。
2. 等任意长但有限的 real interval，再 finalized 一个 timestamp 为 `u_n=c_(n-1)+C` 的 block。它是 cooldown 首次 crossing，self-fallback 赋予新 `(beta,r)` duty。选择其 real finalization time `a_n` 足够大，使 `a_n >= u_n+B_n+100 Delta`。
3. 当前 attempt 有实际正确 initializer、正常 votes 与 surviving timers。安排下一个 block 的时间戳跃到 `c_n`，real finalization time落在正确 DATA votes 已发生而完整 FC 尚未形成的区间。因 `a_n` 已足够大，`c_n <= t_final+epsilon_E` 成立。严格 AUTO cutoff 在 payload 前生效，old beta 的稍晚 FC 只能 raw-publish。
4. 重复以上步骤。每个 cooldown 等待有限，timestamps 不减且增长；bounded visibility/publication 没有被破坏。任意增长的 `B_n` 都能由下一段 unassigned real wait 预先补充足够 clock lag。

因此把 `theta_(n+1)>=theta_n+B_n` 与 `t_(n+1)<=t_n+L` 直接求和的证明，在存在这类 gap 时不成立。问题不是新的 authority 等待，而是 **opening 之间的 real-time slack 没有被收费**。

若要同时满足非常严格的“每个 correct identity 都反复成为 recovery initializer”，可以在固定 schedule 的低 generations 放置若干正确 initializer、紧随其后的可暴露 Byzantine successor，并在较高 clean generation 让最后一个正确 initializer形成将被 cutoff 的 result；这些低 generations 的旧 assignments 可重复，真正高 generation schedules 仍满足 Part I fairness/exclusion。第 4 节已说明低 g 重放不违反 generation-tail exclusion。这只增加固定 regional 准备时长，不妨碍上述任意等待后 timestamp 跳跃。

本构造采用之前报告已经明确采用的 finite-successful-prefix 阅读，满足“some sustained service interval”但不把它加强为每个未来 attempt 必须成功。若冻结文本被解释为必须持续提供足够的实际成功接受区间，则该条件直接排除此类 trace；应明确那一量词，不能暗中使用它来证明目标自身。字面“another eligible region”与单一 region fallback 的关系，也应沿用当前设计既有说明，而不是用它偷偷否定用户明确要求的 one-region renewal 场景。

这不是 strict-cutoff 的一般不可能性。可能的修复方向包括消除这种无结果也无计时义务的等待、让 retry exposure 使用可证明的实时锚点，或给出能覆盖 unassigned gaps 的新摊销不变量；每个方向都需要真正的公共 replay 规则。不能添加 Espresso lower-lag bound、把 local Ready 时间当 canonical timestamp，或自动给新 READY 类型套用 `T_pub`。

## 9. 建议下一步与状态

1. 在新 retry 设计中采用第 3 节的 r-preservation 和持久 cache；把第 2 节的 inherited schedule/exclusion 量词及 beta-independent schedule 解读写清楚。没有必要先改 regional admission。
2. 单独完成第 5 节 finite-prefix response lemma 的事件级证明，尤其是 term/entry skew 和 FC 聚合；将结论明确限定为充分准备后的持续授权区间。
3. 继续处理 current Ready 所要求的 `q>q_open` header 前置依赖；已有报告把 strict advancement 移到 regular successor search 的方案与本 authority 结论独立。
4. 在声称 adaptive AUTO 保证 progress 前，先解决第 8 节 unassigned cooldown gap。不能沿用未经修补的 opening-to-opening telescope。
5. 区域服务公平、请求 ingress/acceptance、hard-control submission policy 仍是独立义务；本报告的有限 authority 前缀不证明它们。

验证范围：阅读 live 与 frozen sources、检查继承量词、展开有限前缀论证和显式时序。未运行 model checker、形式证明器或新协议测试。没有最终 secure/liveness claim。
