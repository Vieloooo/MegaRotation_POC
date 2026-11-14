# v3.4.1 proof-scope erratum 独立核验

## Summary

**Final Verdict:** 在文中既定的历史、完整适用 L1 控制、advance announcement、不可变 authority 及验证接口前提下，本次将“完整 interpreter state 相同”收窄为“public replay result 相同”的 erratum 有效；本核验范围内未发现 `Critical Error` 或 `Justification Gap`。

**List of Findings:** 无。这里的通过仅针对本次定义和 Proposition `prop:global-prefix` 的证明范围修正，以及修正未改变运行规则这一声明。它不是完整协议安全性、原生状态安装时限或 liveness 的重新认证。固定模型前提下的完整 liveness 目标仍未解决；主文 2730–2813 行保留了这一限制。

核验方式：独立阅读冻结源文件、逐字段检查定义和归纳依赖、搜索残留强相等表述、重算主文 diff、逐字比较全部 6 个 protocol boxes 和所含两个运行片段。不读取 v3.4 或其他版本的 reviewer reports；不修改协议源文件。本次未运行实现、model checker、测试或 LaTeX 编译。

### 冻结对象与 SHA-256

所有行号均指本目录 `snapshot/` 中的文件。主文简称为 `main`。

| 对象 | 实际 SHA-256 |
| --- | --- |
| `sync_hotstuff_rr_part2_rotation_v3_4_1.tex` | `67cf324bcda71a8ee2bad74772ef618a4c2000b002eef31390c2fcb468e3b133` |
| `sections/rotation_v3_4_retirement.tex` | `aca904ad6cf8303a541a9265e2793a9f3b2764381b5ae8f6c1f5708d1899110a` |
| `sections/rotation_v3_4_ready_bundle.tex` | `facc64cfdb30cdc9b7166c4a7332120dfb9f5ba92365602244e9930ac238597b` |
| `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |
| `manifest.json` | `64dea4a8bc45faa72c3ce0f4fbe16c5b0a410ea09b446f753bcf76d7d423add8` |
| 比较基线 `../v3_4/snapshot/sync_hotstuff_rr_part2_rotation_v3_4.tex` | `3f614539cb58be30acc44742163ec76e871107fa723bd1f4a766057ad4627bf0` |

manifest 中四项均与实测匹配。主文内嵌的整个 Model and assumptions 与冻结模型文件相同。

## Detailed Verification Log

### 1. 原有完整状态和新结论对象

**Location:** main 1466–1483：

> “It contains ... pending controls, and source-verification checkpoints.”
>
> “Its public replay result at a completed Espresso position is ...”
>
> “It excludes future controls not yet due, extra source-reading checkpoints, validation caches and locally prepared speculative programs.”

**Verification:** 原先的完整 `S_u` 明确含有 future-control queue 和 source-verification checkpoints；相同 Espresso 位置并不固定它们。新定义保留完整状态供实现和证明输入使用，另行指定由该已完成位置产生的公开结果，因而没有把完整状态的字段偷偷删除。

具体而言，两个观察者从同一 checkpoint 出发，其中一个额外取得一条 `theta_a > theta_q` 的已最终确认控制，另一个尚未取得它：两者的本地 L1 cursor、queue 可以不同，公开结果仍相同。反之，若缺少的控制已适用于该 Espresso 前缀，main 2277–2282、2311–2313 要求等待，不能将其作为第二个正确且已完成的结果。这恰好消除了原强相等主张的反例。

### 2. 公开结果逐字段的依赖

**Location:** main 1471–1477。

**Verification:** 所列字段都可以由已处理的 canonical history 固定；没有字段需要由观察者任意选择的本地准备进度决定。

| 公开字段 | 固定它的输入和规则 |
| --- | --- |
| `G`、内容顺序与 producing identity | 相同 checkpoint、规范 EXPORT 次序、精确 expected base、原子接受规则；main 1403–1462。 |
| 当前 branch version、closed-version set | Protocol 5 的 due MSIG/AUTO 唯一转换与版本递增；main 2214–2252。这里是该 replay 位置产生的关闭集合，不能替换成某个 manager 已额外收到的未来 closure 证据集合。两者的分离来自 main 1722–1727 的单一 public mutation path，以及 retirement 47–56 行“consumes an established closure”。 |
| assigned export duty、窗口和计数 | 固定初始 authority、已接受 DATA 数、规范 opening timestamp 和 deadline；main 2033–2043、2141–2173。Ready、下载、重启和重复 handoff 不重置窗口。 |
| activated eligibility、历史 eligibility snapshot | 到期 STATUS/CLEAR、有效 Force 附带变化、由 AUTO 派生的 cooldown、不可变指定 authority；main 248–302、2218–2247。未到期控制不提前生效。 |
| applied-control effects | 完整适用控制集合、`(theta_a,c)` 次序、entry-version target match 以及一次处理规则；main 264–277、2218–2230、2271–2282。较长 L1 前缀中的未到期记录不改变这个集合或排序。 |
| public boundary/source-context records、历史 selection snapshots | 已接受 terminal DATA 或规范 Force 的 cause/anchor/source context，以及最早合格 `q_s`；main 1820–1848、1884–1927、2240–2249。本地 speculative regular handoffs 和预取缓存不因存在就进入 public interpreter。 |

authority 不是一个隐含的本地选择输入：同一 `(r,g)` 最多有一个不可变 authority；缺失证据必须等待，而不能当作零权重或空候选集，见 main 248–262、1448–1455。公开历史记录中的 authority、anchor reference 与 publication-height bounds 因此不会因不同读取时刻而分叉。

### 3. 归纳基及控制输入完整性

**Location:** main 2578–2600：

> “start from the same authenticated interpreter checkpoint”
>
> “Complete applicable L1 controls, immutable authorities, Espresso headers and namespace contents authenticate the same next input.”

**Verification:** 相同初始完整 checkpoint 蕴含相同初始公开结果。之后无需保持完整 checkpoint 相同：每一步只比较适用控制及该 Espresso 步骤使用的共同认证输入。

“all applicable L1 controls verified”没有引入新环境假设：main 379–382 已要求完整控制前缀，main 490–501 已规定 advance announcement，main 2277–2282、2311–2313 已要求确认前刷新适用控制并在证据不足时暂停。首次 crossing 时由 `theta_q >= theta_a > s+T_L1+epsilon_E+eta` 及 `theta_q <= t_E(q)+epsilon_E` 得到 `t_E(q) > s+T_L1+eta`，与已有可见性界共同提供证明使用的先行可用性。离线或落后的观察者仍须完成补读；这里没有由 elapsed time 自动推断“已经完整”。

控制 sequence order 与 activation order 不同也不破坏归纳。所有已到期记录都必须属于共同适用输入；只读到更短且遗漏已到期记录的 L1 前缀违反前提。对于合法的不同较长 L1 前缀，其额外未到期记录只留在辅助 queue 中。

### 4. Prelude 与历史边界

**Location:** main 2596–2600：

> “The status order, first matching MSIG, alternative AUTO check, fixed windows and first eligible historical selection are deterministic.”

**Verification:** Protocol 5 先处理独立 STATUS/CLEAR，再只应用匹配 entry version 的第一条 due MSIG；其余 due Force 按既有规则消耗。仅没有匹配 MSIG 时才检查旧 duty 的 AUTO。旧窗口、count 和 retained `G` 已由归纳假设固定，故选择、版本、cooldown 与新边界也固定。等时间戳不会重放已处理控制，timestamp jump 按同一个首 crossing 处理。

Regular boundary 的 `q_0` 来自 terminal DATA 签名 reference，Force 的 `q_0` 来自 triggering height；`q_s` 取完整历史中的最小合格位置。已接受 EXPORT 的 selection reference 必须严格早于其所在 Espresso block（main 1436–1440），所以不能通过额外预读未来 Espresso blocks 改写当前已完成结果。缺历史或 authority 时等待并回到同一个 canonical position；已知没有候选时才保持 unassigned。上述分支未使用本地接收时间。

### 5. EXPORT 步及 prefix-comparability

**Location:** main 2602–2609：

> “Acceptance atomically appends exactly the next continuous range; rejection appends nothing.”
>
> “This gives identical public replay results after each record.”

**Verification:** 输入 record 和 prelude 公开状态相同，expected DATA base 相同；公开接受规则检查相同 branch、连续高度、provenance、regional/application validity、quota 及历史 normal boundaries。这里仍使用文中已声明的验证接口前提，不是重新证明这些接口的 soundness。完整 namespace 缺失会暂停，不能当作非法 record；历史 cache miss 也必须执行相同验证，而不能得到不同公开判定。

临时状态整笔接受或整笔拒绝确保计数、正常边界转换与 DATA 追加一致。支持性证据可以另行缓存，但它们已被排除在比较对象外。因此共同 Espresso 前缀的每个公开步骤一致；两个已验证前缀长度不同的观察者位于同一 append-only DATA 序列的有序步骤，其 DATA 输出 prefix-comparable。归纳没有再次要求 future queues 或读取游标相同。

### 6. Preservation 与本地 lifecycle

**Location:** main 2611–2619：

> “Force retains all of G before payload ... EXPORT only extends it.”
>
> “Local Handler phases and retirement do not mutate G.”

**Verification:** STATUS、CLEAR、window settlement 和 unassigned selection 不删除 DATA；Force 保留 pre-payload `G` 和原 producing identity；合法 EXPORT 仅作连续追加。closed version 不能重新开放，后续旧版本 EXPORT 无权追加。这个分支不依赖两个观察者拥有相同本地 programs、receipt caches、Closed relay duties 或原生执行状态。新定义也未声称这些本地对象相同。

### 7. 全 checkpoint 绑定与残留强表述

**Location:** main 1482–1483、2284–2291、2338–2343：

> “A certifier transition still binds its actual full previous/next checkpoints and complete source inputs”

**Verification:** Section 8 确实规定 attestation 绑定完整前后 replay checkpoints，Section 7 还要求绑定 L1 checkpoint、control sequence 和 history commitment。投影相同并不允许用 `G` 或 public replay result 替换 certifier 的完整绑定对象。不同辅助输入可以形成不同完整 checkpoint；这与其公开结果相同没有矛盾。

已搜索主文及两个运行片段中的 same/identical/equal/agree/common state、checkpoint、snapshot、boundary 等相关表述。原先“same interpreter state”“identical states”“same prelude state”“states agree”不再残留为跨观察者结论。main 2072–2073 保留的“same verified public prefix fixes the same public state and Force boundary”位于 speculative programs 的区分语境，且由 1471–1483 的定义和 1722–1727 的 public mutation path 限定为公开结果；它没有重新声称整个 `S_u`、所有 source cursors 或本地 manager 状态相等。

### 8. 无运行规则改动及审查边界

**Location:** main 105–111：

> “This is a proof-scope correction, with no protocol transition change.”

**Verification:** 由两个冻结 main 文件重新生成的 unified diff 与 `v3_4_to_v3_4_1.diff` 完全一致。差异限于版本/title/header 元数据、版本说明、public replay result 定义及 `prop:global-prefix` 的陈述和证明措辞。全部 6 个 protocol boxes 逐字一致；两个 `sections/rotation_v3_4_*.tex` 运行片段逐字一致；冻结模型和主文整个模型节逐字一致。因此该声明通过。

本次未重新证明 retirement safety、ReadyBundle 的完整消息安装条件、Part I 区域 soundness 或整个系统 liveness。尤其不能由本 erratum 的有效性推导“固定前提下完整目标已实现”；main 2730–2813 明确保留 slow-session/request-progress 障碍和 service quantifier 问题。
