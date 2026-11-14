### 多签健康控制政策

对应英文 `sec:health-policy`。既有 honest, coordinated committee 执行以下签发算法。它限制 committee **以后签发哪些控制**；不能仅由“committee 诚实”推出这些限制。健康报告与运营报告可以请求干预，但下面的结论不要求它们准确，也不要求它们在有界时间内送达。包括 emergency 在内的任何报告都不能绕过有限额度。

算法不改动已签发控制的 target、activation time 或优先级。每条已提交 MSIG 仍在既有 first crossing 进入 Protocol 5：第一条匹配 entry branch version 的 due MSIG 优先于 AUTO，其余 due records 按原规则消费。普通 regional handoff 不使 target 过期。controller 不等待 quota 完成再执行匹配的 MSIG，不回滚 accepted DATA，也不选择更旧的 retained anchor。

#### 状态与额度

controller 单调推进一个已验证的 replay cursor；签发 journal 在重启和服务实例替换后继续保留。用 `κ` 标识其观察到的 accepted tip，绑定 chain、DATA height、block hash 和 provenance。

每个 `κ` 有三个初始为 0 的持久位：

| 状态 | 每个 key 的额度 |
|---|---|
| `f[κ]` | 至多一条 MSIG |
| `ℓ[κ]` | 至多一份针对单一区域的健康租期 |
| `c[κ]` | 至多一条可选 CLEAR |

令 `d_i` 为 controller 已重放的 G 中，由区域 i 生产并已接受的 DATA 数量。另设 `u_i[d_i]`，在该区域的每个 accepted-count 下至多允许一份健康租期。全部位初始为 0；扣款将它永久置为 1。

journal 将每次 reservation 绑定到精确操作，记录签名释放、实际提交、finalization、canonical processing，以及各区域尚未结束的租期。**所有签发路径先持久化扣款，再释放签名。** target 变 stale、提交失败或结果未知、阈值已过、操作被放弃，都不退款。重试复用同一精确记录与控制身份，不能新签一条替代记录。若原操作可能从未实际提交，重试还须满足首次提交的 advance-announcement 条件。

只有新观察到的 accepted tip 提供新的 `κ`；只有区域 i 的新 DATA 被接受才推进 `d_i`。Force、recovery、cooldown、Ready、raw publication 和 receipt formation 都不做这两件事。重新读取旧历史不能回退 cursor 或清空持久位。其他区域的 accepted DATA 尤其不能补充 `u_i`。缺少完整 control 或 namespace evidence 时，暂停新签发决策。

#### Controller 的三个动作

受健康管理的区域初始 governance flag 为 ON。本模块只发出下面三类健康控制，每次都遵守既有授权和 advance-announcement 规则。额度耗尽后，运营请求保持未签名状态，没有公共控制效力。

**H1：中断一个 branch。** 在已验证 cursor 上，若 `f[κ]=0`，先预留该额度，绑定此处观察到的 β，提交一条未来 MSIG，且 **attached configuration 为空**。activation timestamp 根据实际提交时刻选择。target 后来变 stale 时，既不 retarget，也不退款。需要排除某区域时，另走健康租期流程。

**H2：为 OFF 设置有限租期。** 对区域 i，要求 `ℓ[κ]=u_i[d_i]=0`，且 i 没有 outstanding lease。先预留这两个额度，然后：

1. 先提交独立的 `STATUS ON(i)`，其 activation threshold `θ_on` 为有限未来值；等待这条精确记录 finalized 且可获取。
2. 在准备实际提交 OFF 的时刻 `s_off`，选择满足以下条件的 `θ_off`：

   \[
   s_{\mathrm{off}}+T_{\mathrm{L1}}+\epsilon_E+\eta
   <\theta_{\mathrm{off}}<\theta_{\mathrm{on}}.
   \]

3. 如果已无这样的 future gap，在释放 OFF 签名前放弃 OFF；已排队 ON 保留，已用额度不退。否则，提交独立的 `STATUS OFF(i)`，将激活阈值设为 `θ_off`。同一租期不签发第二条 OFF。
4. 已签发记录全部 canonical processed、任何未签发 OFF 已确定放弃之前，不为 i 开始下一租期。结果不明的提交继续视作 pending。

**H3：可选地提前 clear cooldown。** 默认等待 AUTO cooldown 自然到期。如果要提前清除，要求 `c[κ]=0`，先扣款，再提交一条针对该区域、已授权的未来 CLEAR。恢复报告或 ON 不清除 cooldown，不补充额度，也不重开已关闭 branch。

租期 ON 表示撤销临时排除，**不表示已证明恢复或未来 synchrony**。它在 OFF 能提交之前已经被安排，因此恢复不依赖日后的 recovery report 或尚未签发的 controller 承诺。

提交顺序与激活顺序可以不同。如果一个 Espresso timestamp jump 同时跨过两个 threshold，两条独立 STATUS 在同一 prelude 中依次 OFF、ON，最终为 ON；该租期可能根本没有有效隔离区间。

新 MSIG 使用空 attached configuration 有实际作用。假设一条 MSIG 附带 OFF，另有 threshold 更晚的独立 ON；timestamp jump 可使二者第一次在同一 block 到期。Protocol 5 先处理所有独立 STATUS，之后 MSIG attached OFF 反而覆盖 ON。新政策避免生成这个组合，但不重新解释已有的旧记录。

#### 行政范围与已有系统的迁移

明确的永久行政移除与健康租期分开，并把该区域移出 managed-region 服务声明的范围。安排该永久 OFF 之前，先停止该区域的新租期并处理其 pending health controls；已签发 ON 仍保留原效力。若允许在模块之外无限签发 administrative 或 emergency 控制，就不能再使用本模块的有限干预保证，不能把旁路当成不计数的良性例外。

下面的恢复命题针对从 genesis 开始运行本政策、具有 clean journal 且 managed flags 初始 ON 的执行。若在已有系统中启用，须单独盘点有限 legacy backlog，包括 reserved、released 和 submitted 操作；每条已签发记录保持原语义。尚未耗尽的旧 MSIG 数量加入下面的次数上界。

旧 OFF 未必有恢复 ON；明确处理这些历史状态之前，不声称恢复命题成立。尤其对旧 MSIG attached OFF，纠正 ON 应在观察到其实际 canonical processing 后再提交，不能只给 ON 选一个更晚 threshold。这里不假定 legacy cleanup 或 controller catch-up 的有界完成时间。

#### 命题：固定 accepted prefix 下控制次数有限

对应英文 `prop:finite-controls`。设从时刻 `s_0` 起，真实 accepted prefix 固定为 `G*`。令 `h*=h(G*)`，controller 在 `s_0` 已观察到的 accepted height 为 `h_0≤h*`。此后每一类 MSIG、health lease、CLEAR 的新 reservation 均不超过

\[
h^*-h_0+1.
\]

如果已有 `N_0` 个先前预留的 MSIG 操作仍可能引起 Force，**包括尚未释放签名的预留操作**，则 `s_0` 之后 effective MSIG Force 的次数至多为

\[
N_0+h^*-h_0+1.
\]

**证明。** 单调 cursor 只能访问 G* 祖先链中高度 `h_0,…,h*` 的 accepted tips。每个 key 的每类额度各有一个。更旧 key 上的 reservation 已在先前 journal 中；其中仍未结束的 MSIG 计入 `N_0`。每条已签发 MSIG 在 Protocol 5 下至多生效一次；stale record 和 exact retry 不增加次数。

这个计数包含迟到观察已接受 DATA 的情形。如果 controller 在 `s_0` 已跟上 G*，最多还可预留一条新 MSIG，而不是每次 β 改变后都获得新额度。

#### 命题：有限健康排除之后恢复 ON

对应英文 `prop:health-restoration`。考虑一个从 genesis 起始终受本政策管理的区域，并使用既有 progress 条件中 Espresso timestamps 持续增长的要求。如果 G 最终停止增长，该区域的 governance flag 最终永久为 ON。即使其他区域继续推进 G，只要该区域自己的 accepted DATA 总数有限，同一结论仍成立。

**证明。** 在固定 G 的尾段，由前一命题及有限的先前 journal，lease reservation 总数有限。另一路证明是：若区域 i 的 accepted DATA 有限，其已观察 `d_i` 只取有限多个值；每个值只允许一份租期，与其他区域的进度无关。

每条已签发 OFF 之前，都已有一条 finalized 的独立 ON，且 ON 的有限 threshold 更晚。增长的 Espresso timestamps 最终跨过它。即使二者 first crossing 相同，执行结果仍为 ON。同一区域的下一份租期不与这次 processing 重叠；新 MSIG 又没有 attached status changes。因此有限份租期中的最后一条 OFF 最终被撤销。OFF 释放前被放弃的租期不妨碍恢复。如果从未签发 OFF，初始状态已经是 ON。

这两个命题只说明控制政策的性质。ON 仍须结合 cooldown 和 authority 检查才决定 eligibility。AUTO 仍可能持续打断工作；全局增长也可能无限补充 MSIG 额度。这里没有证明 timely receipt formation、每个区域获得 useful service，或请求最终被接受。它们为固定 G 的反证路径提供有限 committee interference，不把冻结的服务条件强化成 recurring successful intervals，也不假设准确的 failure oracle。
