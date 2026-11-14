## 7. Force handler

Protocol 5 关闭外部 branch；Protocol 3 消费已验证的 closure，再按 retirement 接口收尾旧程序。解释器从 accepted delivery 推导 AUTO，或执行已提前公告的 MSIG Force。两者都保留触发 Espresso block 的 payload 之前的完整全球前缀 $G$。有效 cutoff 以及 MSIG 优先于 AUTO 的顺序不变。

### 7.1 Export duty 与缺块窗口

Export duty 属于负责 $G$ 之后下一块 DATA 的 session。其初始 authority 固定 $W_{\rm start}$、$W_{\rm run}$、$m_{\rm start}$、$q_{\min}$ 和 $C_{\rm auto}$；这些基础窗口参数和 session quota 都不重写。

在版本 $\beta$ 中开启一份 duty 时，固定其连续失败容忍数

$$
M=1+\beta,
$$

并令 $\mathit{misses}=0$。$M$ 在这份 duty 内不变，不引入额外 retry counter。常规交接保持 $\beta$；有效 Force 先增加 $\beta$，再开启后继 duty。

令 $R$ 为窗口开启时的剩余 DATA quota。完成会先移除 duty，不会在 $R=0$ 时再开窗口，因此每个目标都为正。公共 opening timestamp 为 $\theta_0$ 时，开启 startup 窗口：

$$
[\theta_0,\theta_0+W_{\rm start}),\qquad
m=\min\{m_{\rm start},R\}.
$$

Startup 首次成功后，改用长度 $W_{\rm run}$、目标 $m=\min\{q_{\min},R\}$ 的 running 窗口。每个窗口保存 phase、start、deadline、固定 target 和 accepted count。Duty 另保存 $M$、$\mathit{misses}$，以及用于 retry-slot 规则的本 duty 累计 accepted DATA 数 $a$。

只有新接受进 $G$ 的 DATA 增加这些计数。同一 DATA 只在原生产 session 中计算一次。Empty DATA 按同样 quota 规则计数；DRAIN、heartbeat、proposal QC、重复记录以及前方仍有缺口的 future-session DATA 都不计数。新窗口的 count 从零开始，target 按该时点剩余 quota 截断。多余输出不能结转成后续窗口的信用。

**结算到期窗口。** 在首个 timestamp 达到 deadline 的 Espresso block 中，先比较已有 count 和该窗口固定 target，再读取 payload。

- 成功：令 $\mathit{misses}=0$。Startup 成功后开启 running；running 成功后开启下一个 running。
- 失败：将 $\mathit{misses}$ 加一。若仍小于 $M$，在**同一 phase** 开启下一个基础窗口；startup 失败后仍是 startup。若达到 $M$，产生一个 AUTO，并停止结算这份 duty。

每个后继基础窗口都从前一 deadline 开始，长度取 authority 为该 phase 固定的基础长度；不能改用发现 crossing 的 block timestamp。因此，成功重置连续失败计数；AUTO 需要本 duty 连续 $M$ 个基础窗口失败。

若一个 timestamp 跨过多个 deadline，就按上述规则依次结算，直到下一个 deadline 大于该 timestamp，或已经产生 AUTO。中间不插入 payload。版本仍开放时接受 terminal DATA，会取消该 session 的所有剩余检查及 misses，即使当前窗口尚未到期。

窗口仍是半开区间。Timestamp 恰好等于 deadline 的 DATA，不能修复 prelude 已经结算的窗口；只有 prelude 后版本仍开放时，它才可能计入下一窗口。

**零 DATA 窗口的批量结算。** 不必执行 $M$ 个后台 callback。设当前窗口 count 为零、deadline 为 $d\le\theta$、phase 长度为 $W$、misses 为 $b<M$。在没有中间 payload 时，随后跨过的同 phase 窗口也都为零，剩余 quota 不变。令

$$
N=1+\left\lfloor\frac{\theta-d}{W}\right\rfloor,
\qquad j=\min\{N,M-b\}.
$$

若 $j=M-b$，所需的最后一次失败具有 deadline $d+(j-1)W$，并在当前 block 产生 AUTO。否则令 $\mathit{misses}=b+j$，开启 start 为 $d+(j-1)W$、deadline 为 $d+jW$、count 为零的新窗口。有非零 count 的窗口先正常结算；之后连续的零 count 窗口可以采用这个算式。它与逐个结算完全相同，不另造时钟，也不改变基础窗口长度。

**一次 miss 说明什么。** 失败只说明规定速率下 accepted DATA 不足；一次 miss 不再必然关闭 branch。Receipt 形成、状态恢复、发布和 Espresso 服务都可能影响计数。违反某个服务上界意味着相应活性前提不成立，但公开 miss 本身不识别责任方。Miss 或最终 AUTO 都不自动构成可 slash 的 equivocation 或 censorship 证明。

### 7.2 Owed service 与 Force 选区

解释器维护当前服务 round 的有限 owed 集 $\mathcal O$。只有 accepted DATA 清偿服务义务；selection、Ready、区域 commitment 和 raw publication 都不清偿。某块 DATA 进入 $G$ 时，payload 处理从 $\mathcal O$ 中移除其生产 region（若该 region 仍在集合中）。已经分配给它的 duty 可以继续完成固定的剩余 quota。该 duty 继续期间，即使 owed 集为空也不立即补充。

在新的**公共** duty boundary 或 pending-boundary selection，针对指定 target slot $r_*$ 执行 $\mathsf{PrepareOwed}(r_*)$：

1. 取得不可改写的 $\mathsf{AUTH}_{r_*,0}$ 及完整验证证据。缺证据时在该规范转换处等待，不能将它解释为零权重或空集合。
2. 移除 governance 为 OFF，或在指定 target authority 下 total weight 非正的成员。实际生效的 governance OFF 也在应用控制时移除相应 debt，包括匹配的旧 MSIG 所附带的 OFF。Stale MSIG 的附带配置没有效果。
3. 若此时 $\mathcal O$ 为空，开启新 round，加入 governance-ON 且在指定 target authority 下 total weight 为正的全部 regions。构造 debt 集时忽略临时 AUTO cooldown。若没有这样的 region，集合保持为空，选区等待。

除新 round 初始化外，ON 不把新 region 插入尚未结束的 round；它可加入下一 round。Cooldown 从不移除 debt，CLEAR 和新 branch version 也不重置 owed 集。

只有规范公共解释器调用该过程。投机的 REGULAR 交接可以准备后继，但不修改 $\mathcal O$。公共解释器分配 regular successor 时会调用 $\mathsf{PrepareOwed}$，但其选中 region 仍由历史 $\mathsf{NextLive}$ 决定：**REGULAR 不按 owed 集过滤。**

对于 Force，定义 $\mathsf{NextOwed}(i,X,O)$：从 source $i$ 后面按固定环顺序先搜索其他 regions，只考虑属于 $O$、且在 snapshot $X$ 与指定 target authority 下 eligible 的成员。若没有这样的其他 region，而 $i$ 自己 eligible 且属于 $O$，则返回 $i$；否则返回 $\bot$。Synthetic source 为 $\bot$ 时，从列表第一项开始搜索。Governance ON、cooldown 已到期和指定 authority 下正权重仍须同时满足。

对每个 Espresso block，在 prelude 后、payload 前保存 $\chi_E(q)$ 和 $\mathcal O_E(q)$。Pending forced boundary 使用捕获的 source context 和指定 target authority，从触发位置的 selection lower bound 开始，寻找首个满足

$$
\mathsf{NextOwed}(i_b,\chi_E(q),\mathcal O_E(q))\ne\bot
$$

的已认证历史位置。这个最早位置固定 successor。之后 payload 中的 credit 或新 round 不重写已经保存的 pre-payload snapshots。REGULAR witness 仍执行其规定的历史 $\mathsf{NextLive}$ 搜索，即使选出的 region 不在当前 owed 集。普通 OFF 不撤销已经固定的 handoff。

**有条件的 AUTO cooldown。** 应用新的 AUTO cooldown 前，先按 retry-slot 规则确定 target slot，执行 $\mathsf{PrepareOwed}$，再在**尚未加入这次新 cooldown** 的状态下检查该 target authority 的 eligibility。只有 $\mathcal O$ 中还存在另一个当前 eligible 的 region，才把 source 的 automatic cooldown 设为 $\theta+C_{\rm auto}$；否则保持其 cooldown 记录不变。

这不清除旧 cooldown，不覆盖 OFF，也不隐含 CLEAR。它避免这次 AUTO 隔离 owed 集中最后一个 eligible source。如果既有约束使所有 owed candidates 都不 eligible，boundary 仍需等待。

### 7.3 Scheduled control 与两类 Force

MSIG Force 绑定 chain、目标 branch version $\beta$、activation timestamp $\theta_a$、control sequence 和已授权的 attached configuration。L1 sequence 标识该指令；指令不提供 retained DATA height。委员会按既有诚实控制与 L1 服务前提提交，并提供 finalized evidence。仅签名不启动 submission bound。

L1 finalization 使指令被知晓；它只在 $J_{q(\theta_a)}$ 生效。此前正常生产、交接和合法 EXPORT 都可继续。激活时，目标匹配进入 prelude 时的 branch version 才执行；stale target 被消费，不关闭当前版本，也不应用附带配置。Regular handoff 不改变 $\beta$，因此不会取消该指令。Independent STATUS 保留自己的独立效果。

新 HealthPolicy 签发的 MSIG 附带配置为空，但解释器仍执行匹配、有效的旧记录所附带的授权配置。不提前应用它，不应用 stale 记录的配置，也不重新解释 independent status 与 MSIG 的既有执行顺序。

AUTO 绑定 chain、旧 branch version、export-session pair、固定 $M$、由 replay 确立的连续 misses、第 $M$ 个失败窗口的身份和 finalized triggering header。Cause deadline 是最后这个失败窗口的 deadline；realized time 是触发 block 的 timestamp。它不需要交易来创建；可选 audit record 也不能让它再次执行。

**Protocol 5：处理 prelude 并关闭 branch。** 对已认证、finalized 的 $J_q$，在读取 payload 前令 $\theta=\theta_q$，并捕获 entry branch version。

1. **应用独立控制。** 从完整 control prefix 取出尚未应用、已经到期的 independent STATUS 和 CLEAR，按 $(\theta_a,c)$ 执行并标记 applied。实际生效的 OFF 移除相应 debt。新 ON 不加入未完成 round；继续中的 duty 也不因 owed 集为空而立即开新 round。同 timestamp 的后续 block 不重复应用。
2. **应用匹配 MSIG。** 按同样顺序检查 due MSIG，只应用第一个 target 匹配 entry version 的记录。若已有 duty，捕获其 source context、accepted count $a$ 和 index；否则沿用保存的 context 与 target index。应用该记录授权的 attached configuration 及相应 debt pruning，并关闭 entry version。其他 due MSIG 全部按 stale 消费，不应用附带变更。若发生匹配 MSIG closure，取消旧窗口并跳过第 3 步。
3. **结算交付窗口。** 若有 duty，按第 7.1 节结算 deadline 不大于 $\theta$ 的窗口。连续 misses 尚未达到 $M$ 时，保持 branch 开放并保留结算后的窗口状态。第 $M$ 次 miss 时，捕获 source context、index 和累计 accepted count $a$，并在该位置关闭 branch。后继选区前确定 target slot、准备 owed 集并按上述条件施加 cooldown。缺 namespace 或 control evidence 不能证明窗口失败。缺 target authority 证据时暂停 debt、cooldown 和 selection 的计算；不能把它解释为候选集为空，也不改变 cutoff。
4. **保留前缀并确定 retry target。** 任一种 closure 都完整保留 $G$ 及原 provenance，归档旧版本的未确认 suffix 和普通后继。记录 cause、triggering header、retained anchor、捕获的 context 和 selection lower bound $q$。按 retry-slot 规则：已分配 duty 且 $a=0$ 时保留当前 $r$；$a>0$ 时 target 为 $r+1$；未分配 duty 时保留已保存的 target slot。$\beta$ 恰增加一次。新 engine 真正 Enter 时使用全新的 core state，且 $g=t=0$。触发 block $J_q$ 本身不能接受新版本 DATA。
5. **准备、保存 snapshot、选区。** 对新 boundary 或尚未解决的公共 boundary，在指定 target authority 下执行 $\mathsf{PrepareOwed}$。全部 prelude 效果完成后保存 $\chi_E(q)$ 和 $\mathcal O_E(q)$。Forced boundary 用最早历史 $\mathsf{NextOwed}$ 位置；regular boundary 用其规定的 $\mathsf{NextLive}$；genesis 使用其 synthetic selector。Duty 开启时，从规范公共 opening timestamp 开启 startup，固定 $M=1+\beta$，并令 $\mathit{misses}=0,a=0$。没有候选则保留 anchor，不运行 DATA 窗口。若 duty 只是继续，仍保存 pre-payload snapshots，但不重置窗口、容忍数、计数或 round。之后才处理 payload。

所需选区证据可以稍后取得，但缺证据不会移动 control cutoff，也不会产生新的 opening timestamp。Replay 在同一规范转换处恢复；已记录的效果不重复应用。Force 的 cause、新版本和 retained anchor 可以已经确定，而 target selection 尚未解决。

若 timestamp 一次跨过多个 threshold，先处理 independent controls，再处理匹配 MSIG；MSIG 优先于 entry duty 的所有 AUTO 检查。窗口结算可以累计多次 miss，但同一 prelude 最多执行一次有效 Force。新 duty 从实际 selection timestamp 开始，而不是从旧的 missed deadline 开始，也不会在同一 prelude 中再次被结算。正的首个窗口长度阻止同 block 的连锁 Force。之后 payload 可以清偿 accepted DATA 对应的 debt，并按公共 boundary 规则分配 regular duty。

**转换确定性。** 相同的完整来源历史和前一 replay state，固定了 due-record 顺序、target matching、窗口计数、$M$、miss 转换、target slot 和 debt 更新。Eligibility 使用同一指定 authority；历史 $\mathsf{NextOwed}$ 使用已保存 snapshots，而非本地接收时间。批量算式执行同样的零 count 窗口转换。因此 prelude 只有一个下一 replay state；缺证据只暂停计算，不改选其他结果。这一转换性质不另行证明全球活性。

### 7.4 控制处理与完备性

Control ingestion 验证完整 L1 prefix，并保留 pending activation queue。Sequence number 本身不表示所有已取得记录都已激活。解释器在各自 target context 中对每条控制只处理一次。ON、重复 Force 和晚到区域 receipt 都不能重新开放已关闭的 branch version。

每个正确 observer 在确认 Espresso 区间前刷新完整 finalized control view。Advance-announcement 前提保证该区间适用的控制已经可用。缺证据就暂停 replay。新 observer 重建同样历史，不能只因自己晚得知某条指令而拒绝它。

Certifier 的 pure core 绑定所用 L1 checkpoint、control sequence 和 history commitment。L1 acceptance interface 拒绝遗漏适用控制的 attestation。保守实现要求所声明的完整 control version 与 accepting contract 要求的版本一致；stale version 刷新后再重试。更细的检查必须提供等价完备性证明。一条事件的 inclusion proof 不能证明没有其他中间控制。

如果后来发现某条遗漏控制本应影响已经确认的历史，这违反 completeness 或 announcement 前提。它不授权回滚 $G$，不授权在任意更晚位置重新执行 Force，也不允许按本地到达顺序处理。本协议的条件性保证不规定这种前提违例之后的恢复过程。
