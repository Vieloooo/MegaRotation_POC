# Summary

**Final Verdict:** 本快照的接口修复与条件性 retirement 证明具有可行的抽象组成关系，但不能判为完整协议目标已达成：`Common interpretation` 关于完整 interpreter state 相等的子命题存在一个字面反例（**Critical Error，范围过强**）；原生 Ready/Enter 时间界仍是明示的接口义务；至少一个可用区域下的持续全局进展、有用服务与请求进展仍未证明（**Justification Gaps，稿件已承认**）。未在本轮检查中构造出满足完整抽象 Ready contract 的 regional committed-log 分叉，亦未构造出 global DATA prefix 分叉。

**List of Findings:**

- **I1 — 完整 interpreter state 相等的量词过强。** Location: 主文件 2559–2560，`Replaying the same complete Espresso prefix yields the same interpreter state.`；1458–1460 明确把 `pending controls` 和 `source-verification checkpoints` 放入该状态。Issue: 同一 Espresso 前缀不固定观察者已经读取的未来 L1 控制和 L1 源游标。下文给出满足源模型的反例。分类：**Critical Error** 于该完整状态相等子句；属于外部接口范围错误，**不是 global prefix safety 反例**。只比较已生效的公共转移状态，或同时固定完整 L1 输入，才是当前归纳实际支持的范围；本文不修改证明。
- **I2 — Ready 时间与依赖它的区域安全证明仍有原生接口义务。** Location: Ready fragment 107–146，尤其 `If validation, adoption and the atomic Enter transition are ... covered by ... ass:network`（112–113）；retirement fragment 240–242。Issue: 冻结模型的“complete validation dependencies”界不自动蕴含任意原生存储、状态安装或 runtime 初始化也在同一个 `Delta` 内。稿件明确保留该条件，因而其条件命题不由此判假；但把它提升为“仅由未改变的 Section 2 假设已完成实际区域安全证明”仍是 **Justification Gap**。这是明示的实现/模型接口义务，不能用给操作改名来消除。
- **I3 — 完整活性目标仍缺失，选择公平性不能替代服务公平性。** Location: 主文件 2702–2785，特别是 2704–2708、2740–2746、2779–2785。Issue: 文字保留固定公开 cutoff、无统一界的启动/恢复等待，并未证明某个未来正确产出必然赶在截止前进入 `G`。`some sustained service interval` 的量词也没有固定；`another eligible region` 仍在冻结的条件中。分类：**Justification Gap / 未完成目标**，不是稿件暗中声称已证明的错误定理。有限服务区间读法下，稿件慢会话轨迹给出持续全局进展的反例；更强无限尾部服务读法则会排除它，二者不可混用。

## 审阅对象与完整性

本报告只使用 `formalization_v2/reviews/v3_4/snapshot/` 中 manifest 列出的六个源文件，以及本地 `proof-verifier` skill。未读取其他版本的评审报告，未修改协议源文件。行号均为本快照物理源文件行号；下文以 **M** 表示主文件，**R** 表示 retirement fragment，**B** 表示 ReadyBundle fragment，**P1** 表示 Part I，**F** 表示冻结模型文件，**V31** 表示冻结 v3.1 主文件。

| 文件 | 实测 SHA-256 | 与 manifest |
|---|---|---|
| `sync_hotstuff_rr_part2_rotation_v3_4.tex` | `3f614539cb58be30acc44742163ec76e871107fa723bd1f4a766057ad4627bf0` | 一致 |
| `sections/rotation_v3_4_retirement.tex` | `aca904ad6cf8303a541a9265e2793a9f3b2764381b5ae8f6c1f5708d1899110a` | 一致 |
| `sections/rotation_v3_4_ready_bundle.tex` | `facc64cfdb30cdc9b7166c4a7332120dfb9f5ba92365602244e9930ac238597b` | 一致 |
| `sync_hotstuff_rr_part1_core.tex` | `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3` | 一致 |
| `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` | 一致 |
| `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` | 一致 |

从 M 中提取 `\section{Model and assumptions}` 到下一 section 的文本，与 F **逐字节相等**，亦与 V31 对应 section 逐字节相等；其 SHA-256 为表中 F 的值。此项只验证假设文本未改，不把文本相同当成派生证明语义已完成的依据。没有运行编译、模型检查或可执行安全测试；以下结论来自逐项手工检查。

# Detailed Verification Log

## 1. 假设与依赖图

| 前提/接口 | 位置 | 支持的结论与不可推出的结论 |
|---|---|---|
| 固定会话 snapshot，Byzantine 权重最多 `F < W/2`，证书门槛 `>F` | M 345–363；F 135–153 | 每证书有正确 signer，所有正确权重可形成证书；不保证关闭后所有正确者继续签名 |
| 区域正确到正确消息在 `Delta` 内送达并处理完整 validation dependencies | M 377–389；F 167–179 | 完整 proposal、close bundle、receipt 的实际发送可调用区域界；没有跨区首次 state acquisition 界 |
| L1 控制为共同认证前缀、future-dated、提前可见 | M 369–375、409–425、482–493 | 生效事件不能由遗漏控制或本地到达时间决定；不要求不同观察者具有完全相同的未来 pending-control 缓存 |
| Espresso 兼容前缀、完整 namespace、历史可恢复、时间非减 | M 428–464 | 共同外部顺序和可验证 absence；不等于 native execution 或 state installation 已完成 |
| Publish Layer 从完整 regional result 可用时开始计时 | M 391–406、466–479 | `s_submit <= s+T_pub`、visibility bound；不保证 receipt 形成、EXPORT 接受或赶上固定窗口 |
| ReadyBundle 是完整可执行 state object，adoption 与 Enter 被 bounded handler 覆盖 | B 11–58、60–105、107–146 | 在继续可进入的条件下，`e_v <= e+Delta`；这是具体 native adapter 仍须满足的接口合同 |
| 历史 handoff validation 与当前 CanEnter 分离 | M 1852–1928 | 已关闭版本的 receipt/anchor 仍可验证，不能由此复活 engine |
| manager 持久 Closed、完整 closure bytes、历史 snapshot 与 send-on-context 义务 | R 3–22、43–120；M 1621–1630 | 关闭不能只存在于已运行 worker，也不能在拒绝 Enter 时静默丢失 |
| 公共 startup clock 独立于 Ready | M 2011–2021、2126–2151 | 重启、late Ready、重复 handoff 不延后截止；这是剩余活性障碍的一部分 |

证明依赖顺序如下：

1. 历史 admission/authority 验证 + 完整 bytes + 持久 control owner -> `observable-close`（R 171–194）。
2. 首次完整 `RequestClose` 的 multicast + 区域消息界 -> 全体关闭不晚于 `c+Delta` -> 最后 timer 不晚于 `c+3Delta` -> 其 FINAL 不晚于 `c+4Delta`（R 196–236）。
3. 条件性 Ready entry 界 + 完整 Quit/terminal receipt 转发 + 消息先于 timer -> 存活 direct timer 的早期投票窗口无 Quit/Completion。
4. `observable-close` 把 Closed suppression 纳入 `c`：早窗口 `s+Delta<c` 得到 protecting QC；晚窗口 `s+Delta>=c` 在任何后继恢复 entry 前关闭所有 gate。
5. 早窗口 QC/lock rank 与晚窗口禁止恢复组成区域安全论证；receipt 经正确 FINAL signer 追溯到较早 direct timer（R 238–368）。
6. 外部公共转移对 `G` 只追加或保留，独立于 local lifecycle；它的 prefix preservation 归纳无需先证明完整 native Ready 时间（M 2554–2592）。不过其中**完整 S 相等**部分存在 I1。
7. 严格 selection height + cyclic selector 只给条件性选择序列性质；不提供步骤 6 的无限次成功 append（M 2663–2700）。

## 2. 原有 Part I 论证的适用边界

> P1 155–156: correct replicas initially enter within `Delta` of one another.
>
> P1 177–179: authority has one common simultaneous availability time `d(g)`.
>
> P1 751–755: every correct replica knows the protecting QC by a surviving timer's expiry.

这些不能原封不动搬到本稿。v3.4 的初始入场必须经 B 的条件合同；恢复 authority 在 M 1212–1227 仍需取得，但没有重新加入 P1 的共同 `d(g)` 环境假设。R 276–292 改为利用 correct sender 的完整 quit/admission/proposal dependencies，给所需 proposal window 的局部 entry 论证。

该论证的时序检查可成立：若 `a^-` 是前一 generation 的首次正确 Quit，首个后继 entry `e>=a^-+Delta`；完整 quit relay 不晚于 `a^-+Delta` 到其他 manager，接收方强制等待不晚于 `a^-+2Delta<=e+Delta<=s+Delta` 结束。早期 generation 的 Quit 依其已发送的完整 admission 证据逐级处理，不能把第一次在 `s` 看到的一串空 hash 当成完成此义务。M 915–924 的 complete bundle 规定和 R 286–292 是这个步骤的实质前提。

退休以后 universal protecting-QC 命题不成立；R 370–381 明示仅一个正确成员晚投票、另一个已关闭、Byzantine 拒签的例子。R 238–247 也明确撤回 universal-QC / complete-receipt-formation 承诺。此处没有把错误的 universal lemma 当作安全结论的捷径。

## 3. 历史验证、活跃许可与 manager-first 路由

> M 1854–1859: `VerifyHandoffHistory` ... `does not require its version to remain open now and grants no entry permission`.
>
> M 1915–1917: `Closed` blocks entry even before a complete per-session `c_v`; context resolution and closure relay still run before refusal.

这是有效的接口分离。Force 保留旧 DATA 的 original receipt 和 producing pair，故新 anchor 的历史验证必须允许其原版本已关闭。M 1867–1873 对 REGULAR 祖先递归使用 historical mode；M 1885–1900 对 Force 绑定 canonical pre-payload `G`，不重标 producing pair。M 1921–1928 又明确 receipt/ancestry 只走历史验证，进入新 engine 另走 CanEnter。未发现这两种谓词互相调用形成循环。

M 1044–1057 的旧 core 消息顺序先路由 engine；它必须放在后文更强的 manager dispatcher 前置检查之后理解。M 1621–1630 与 R 73–83 明确规定在 worker 不存在、Terminated、重复消息、restart 场景，先处理历史 context 和 closure relay，再拒绝/归档。这里存在多处描述，但目前较具体的 phase-independent 前置规则足以消除“必须先存在 worker 才知道关闭”的旧路径；不能把旧 core 段落当成绕过 manager 的第二个入口。

## 4. 完整 snapshot bytes 是否足以触发可观察关闭

> R 6–9: Closed retains complete source evidence, not only a hash, and the control owner keeps those bytes available.
>
> R 58–62: a complete close bundle contains immutable authority, membership and historical admission context, without requiring prior worker, live-entry permission or an unprovided cache.
>
> M 922–924: references are allowed only if the receiver can resolve them within the regional delay bound.

R 171–194 的关键组合不是“任何 proposal 都能神奇发现远端成员”，而是：正确投票者在 `s` 转发的有效完整 proposal，按 M 909–924 需携带/在界内解析该 session 的 boundary、authority 和 admission witness；这批历史依赖标识固定 recipient snapshot。接收者的 Closed 已保证完整 closure proof 在本地 control owner 手中。两者组合即可发送 complete close bundle，无须执行 state restoration 或等待 CanEnter。

因此在 R 命题的明确前提下，如果这次处理因 Closed 压制 vote/entry，则 handler 已具有所需两类材料并必须先 relay，得到 `c<=s+Delta`。R 69–83 则正确区分单独 Closed observation 与完整 close event：未掌握 snapshot 时没有 `c_v`，也不能据此提前启动 `4Delta` 退出时钟。这条区别对证明成立必要。

不被本命题覆盖的情形：仅有未认证成员列表、无法在界内解析的 hash、远端 closure URL、没有完整 historical admission context 的 Byzantine partial packet、或暂停 control owner 等待存储拷贝。前四者不满足完整包前提，最后一种违反 R 8–9 与 149–153 的 owner 交接规则；它们不是当前抽象协议内的反例。另须承认，本稿未给任意大历史/状态消息的物理吞吐实现证明，不能从抽象 `Delta` 宣称部署性能已获验证。

## 5. `4Delta` 与 early/late safety 组成

> R 198–205: first complete per-session close time `c`; everyone closes by `c+Delta`; timers expire by `c+3Delta`; generated FINAL reaches correct members by `c+4Delta`.

算术正确。令 `c=min c_v`，首个正确成员的 complete relay 得 `c<=c_v<=c+Delta`。每个真正开始过的 timer 对应 fresh vote `s<=c_v`，原 deadline 为 `s+2Delta<=c+3Delta`；再送达 FINAL 用一个 `Delta`。本地阈值 `c_v+4Delta` 不早于全局的 `c+4Delta`。R 117–119 的 message-first 顺序覆盖边界相等。

该界不是 receipt 完成或退出的统一 deadline。R 130–156 仍要求 timer 处理完、persistent owner 接受材料和路由、完整结果已 offer 等条件；R 232–236 明确把 receipt relay、Byzantine delayed release、存储、publication 和 acceptance 排除在 `4Delta` 内。没有发现以 elapsed grace 单独销毁最后证据副本的许可。

R 262–274 通过真实 Quit/Completion 消息取消选定 timer，推导其在 `s+Delta` 前不可能发生，未先假定 receipt-based commits 已安全。R 295–326 对 Closed 的分支也完整：

- 若 `s+Delta<c`，observable-close 排除任何 Closed suppression；正确权重 `W-F>F` 可形成 protecting QC。
- 若 `s+Delta>=c`，closure 到全体不晚于 `c+Delta<=s+2Delta`。后继 generation entry 必须满足 `e_next>=a+Delta>s+2Delta`，故发生 entry 时 gate 已关闭。

R 353–361 的 FINAL provenance 避免 receipt 相互认证：只由 timer commit 发 FINAL，receipt commit 不再发 FINAL，因此每个 receipt 有一个更早直接 timer 作为因果来源。独立的 same-generation equivocation / ranked-QC descent 部分与 P1 735–815 对接；它依赖前述完整消息与 entry 条件，不应外推到不满足 B 的 native adapter。

## 6. ReadyBundle 与严格 `xi` 的依赖

> B 14–15: every required object is present as bytes; URL/state root/unresolved hash is insufficient.
>
> B 25–32: full already-constructed executable `S_A` and native validation evidence `Pi_A`.
>
> B 112–113: timing follows **if** validation, adoption and Enter are the bounded complete message-processing operation.

完整 state 和验证证据、authorities、replay checkpoint 的授权依赖、原 anchor commitment evidence、严格晚于 opening 的 Espresso header 均列入 B 19–47。发送时间不是稍后异步任务：B 60–70 要求 first Ready/Enter 事件已构造完 bundle 并实际发送。接收侧 B 90–105 要求同一 handler 验证、adopt、atomic Enter，不能只排队等待另一个状态加载任务。

因此在声明的抽象 handler 语义下，首个正确 entry 时刻 `e` 的完整 send 确实推出 `e_v<=e+Delta`。但 M 的 frozen network assumption 只写 complete validation dependencies；B 124–146 正确指出额外 native import 不是命名成“一个事件”就变成零耗时。若残余为 `u_v`，只能得 `e_v<=e+Delta+u_v`，且 frozen model 不给统一 `u_v` 上界。这正是 I2 的未完成义务，不是可以省略的注脚。它影响 M 2000–2009 的 bootstrap 时间链和 R 255–260、276–279 对初始 entry 的使用。

严格 selection reference 没有形成自我证明循环：M 630–631 要求 terminal `height(xi)>q_open`；Ready 在 M 1953–1955、B 42–43 预先取得 qualifying finalized header；EXPORT 又要求每个 reference 严格早于 containing Espresso block（M 1428–1432）。完整 package 可先在外部 history 已存在后构造。代价是首次 Ready 可等待更久；该等待没有 public-window reset（M 2011–2021），不能把“DATA 在 Ready 后不再等 header”解释成“从 public opening 到生产无额外等待”。

## 7. I1：共同解释的确切反例与保留的结论

> M 1458–1460: `S_u` contains `pending controls` and `source-verification checkpoints`.
>
> M 256–259: ingestion retains pending as well as applied records, and ingestion is not activation.
>
> M 2559–2560: same complete Espresso prefix yields the same interpreter state.

取一个共同可信 checkpoint `C`，一个有限完整 Espresso 前缀 `E[0..q]`，其最大 timestamp 为 10；其中所有适用控制均已认证。

1. 正确观察者 `u` 在真实时间 19 刷新其完整已最终确定 L1 前缀并 replay 到 `q`，保存状态。此时没有下一条控制 `c`。
2. 诚实 committee 在真实时间 20 提交一条 future-dated STATUS `c`，例如 activation threshold 100。令 `T_L1=10, epsilon_E=0, eta=1`；`100>20+10+0+1`，满足 advance announcement。该记录可于时间 21 final 并可见，不违反任何 upper bound。
3. 正确观察者 `v` 在真实时间 22 从同一个 `C` 刷新 complete finalized L1 view（其中含 `c`），replay 同一个 `E[0..q]`。`c` 在 timestamp 10 尚未 due，进入 pending queue。

两者具有完全相同的 Espresso 前缀和适用控制，`G`、branch version、历史 eligibility、已生效窗口等派生值都相同，但 `v` 的 pending-control queue 含 `c`，`u` 的不含；L1 source cursor/checkpoint 亦可不同。`u` 早已完成那次合法 replay，不需要知道未来真实时间才提交的控制。源历史、future dating、完整 applicable controls、同步与 honest committee 均未被违反。

因此该 counterexample 只否定完整 `S` 相等子句。M 2567–2573 的“same next input”归纳仅证明相同**适用**输入触发相同**已生效**转移；不能覆盖未来 pending records 与源游标。它也在 V31 2071–2079 的同类文字中已存在，不是本次 retirement 新创造的问题。

`G` 的 prefix comparability 和 irrevocability 仍有独立的正确归纳：预处理控制不删除 DATA，Force 保留 pre-payload 全 `G`，有效 EXPORT 原子追加连续后缀，rejection 不变。历史 receipt 只供应证据，不绕过 open-version 或 expected-base 检查（M 1416–1454、2583–2591）。I1 不应被升级为尚未展示的 global fork。

## 8. Same-region renewal、无限目标与反例边界

> M 2667–2672: strictly advancing selection heights; fair repeated selection **if** infinitely many regular handoffs and eventually fixed nonempty eligibility.

该有限条件命题的推导正确：`q_open(next)=q_s>=height(xi(T))>q_open(current)`；自然数严格增长终会超过任何固定历史高度。随后 cyclic search 在固定 eligible set 中轮转；若集合只剩 source，fallback 返回自身。它不要求使用旧 engine，新 pair/authority/anchor 隔离仍成立。AUTO source 还受 finite cooldown/OFF 限制（M 1782–1796）。

但 F 307–309 / M 517–519 仍写 `A waiting boundary eventually has another eligible region`。因此“只剩一个 eligible source 且无限等待”的旧 selector 反例不能当作满足**全部冻结 progress 条件**的反例。M 2774–2777 已正确说明这个边界。同一 source 的新 fallback 是协议规则扩展，不自动证明 one-live-region liveness。

M 2710–2746 的慢 session 轨迹则覆盖 actual entry fairness：correct initializers 实际进入、提案、投票、commit，而 receipt 最早在 startup cutoff 之后形成。取 `Delta=1`、quota/target=1、startup window=6，Ready 于 `a+epsilon`；最晚消息发送使 terminal proposal 在 `a+4+epsilon`，第二位 correct voter 于 `a+5+epsilon` 投票，完整 FINAL receipt 不早于 `a+7+epsilon`。AUTO 于 `a+6` 保留旧 `G` 并关闭版本；后续完整结果仍可符合 `T_pub,T_E`，但无法接受旧版本 DATA。这不依赖取消既有 timer，因而 retirement 修复不消除此轨迹。

其模型地位取决于冻结 service prose 的未固定量词：在“存在某个有限 sustained service interval”的字面读法中，先加任意长成功前缀即可满足该存在条件，后续无限慢 session 仍能让 `G` 停止；若前提被定义为每个未来 duty 或无限尾部均有足够接受服务，该轨迹在该更强前提外。报告不替作者选择更强解释。

M 2748–2766 的 request 例子也区分了三种性质：全局 `G` 可以由 Byzantine 但规则兼容的 empty DATA 无限增长，每个区域也可被反复选中并交付 DATA；与此同时正确领导者带请求 `x` 的慢产出全部落在 closed suffix，故 `x` 永不进入 `G`。没有 equivocation，永久排除规则不能去掉这些快速空产出的 leader。不能由 selection fairness、区域安全或 pending queue retention 推得 request progress。

本稿 M 118–119、2638–2661、2697–2708、2779–2785 已明示这些限制；本轮未发现它在相邻最终保证中又声称完整目标已完成。故用户要求的至少一个 live region 下持续 global progress/useful service 仍是开放目标，不能由本文条件 safety 检查或一次成功编译代替。

## 9. 验证结论的使用边界

可据此报告：phase-independent closure relay、历史验证分离与完整 close context 已提供所需抽象接口；`4Delta` 算术和 early/late 分支在所列条件下没有本轮找到的反例。不可据此报告：全协议无条件安全认证、原生 state import 时间已界定、原版 Part I universal QC lemma 仍成立、每个 late local commit 必然形成 receipt、或 one-live-region 无限服务目标已达成。

I1 的完整状态相等子句需要消歧或收窄；I2、I3 属于仍须完成的具体义务。除此之外，本轮未把违反冻结服务/同步/完整证据前提的场景计作协议内安全攻击。
