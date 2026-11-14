# v3.2 冻结快照：独立对抗性活性审查

## Summary

**Final Verdict:** 不能将该快照判定为完成“至少一个区域存活时持续全球进展，并公平服务所有健康区域”的方案。正文没有给出这个完整活性定理，而且明确承认没有建立它。按 frozen progress 中 “some sustained service interval” 的存在量词读法，可以给出所有正确领导者反复实际生产、所有基础服务界都满足而 \(G\) 最终停止增长的完整轨迹；也可以给出 \(G\) 无界增长、正确客户端请求却永久不进入 \(G\) 的轨迹。若把相邻的非形式化 prose 解释为更强的无限服务保证，这些轨迹会被该额外读法排除，但本文没有精确定义该量词，审查不能替作者静默选择它。

这里的 **Critical Error** 指相关目标命题或推导若被声称成立时存在实际反例；不是把本文未声称的活性定理伪装成一个已经写出的错误定理。**Justification Gap** 指现有文本不能无歧义地支持该量词或推论。下述活性反例不反驳两个限定范围的安全/本地隔离 proposition。

**List of Findings:**

- **L1 — 条件性 Critical Error：完整的持续全球进展目标不成立。** Location: 主文 1915–1923、1942–1952、2054–2078；frozen Model 303–309、314–319。实际进入并生产 DATA 不保证结果在 AUTO 前被接受。下文 T1 让所有正确身份无限反复生产、完整结果随后正常形成并被提交，但最后一个有限成功区间之后 \(G\) 不再增长。成立范围是 progress 的明确存在区间读法；更强读法下的结论是量词 **Justification Gap**。
- **L2 — 条件性 Critical Error：全球吞吐与领导者轮次公平仍不足以推出请求活性。** Location: 主文 2251–2258、2566–2574；frozen Model 297–312。T2 让正确领导者每次包含队首请求，唯独其结果在 AUTO 后发布；其他轮次的 Byzantine leader 空 DATA 持续进入 \(G\)。每个正确领导者、每个区域都反复进入生产，客户端请求仍永不成为 global。
- **L3 — Justification Gap：历史引用饥饿反例的模型内地位依赖尚未写清的 schedule-fairness 量词。** Location: 主文 2614–2623；frozen Model 297–300。固定 \(q_0\) 的 A/B 循环严格满足 selection/reference 规则；但若 repeated entered turns 量化到 C 的所有正确领导者，则该循环违反此 frozen 前提。不能无条件称它是满足全部 progress 条件的反例，也不能用这一排除方式宣称协议已经实现区域公平。
- **L4 — 完整用户目标上的 Critical Error，且不是 frozen-progress 模型内反例：singleton 无法续任。** Location: 主文 1750–1756、2625–2631；frozen Model 307–309。唯一 eligible 区域完成 quota 后，NextLive 永远返回 \(\bot\)；AUTO 不再运行，反复 MSIG 也不改变这个选择事实。原稿关于该反例违反“eventually another eligible region”的限定是正确的。
- **L5 — Justification Gap：没有可用于普遍 end-to-end 活性证明的 startup/clock/Ready 时间关系。** Location: 主文 434–443、475–478、1874–1952；frozen Model 225–234、264–269、314–319。只限制 Espresso clock lead，不能把 \(W_{\rm start}\) 直接理解成真实时间 startup budget；恢复状态没有统一时限；窗口不等 Ready。正文已承认这些限制，因此本项不是指责其作了未出现的有界 failover 声称。

## 审查对象与证据完整性

本报告只读取 formalization_v2/reviews/v3_2/snapshot/ 内的协议、冻结 Model、Part I、retirement fragment 和 manifest，以及审查工作流的 proof-verifier/SKILL.md。未读取 preflight、作者报告或其他 reviewer 报告，未修改协议源文件。以下“主文”均指 snapshot 中的 sync_hotstuff_rr_part2_rotation_v3_2.tex；行号是冻结文件行号。

已重新计算并逐项匹配 manifest：

| 文件 | SHA256 |
|---|---|
| sync_hotstuff_rr_part2_rotation_v3_2.tex | 956b733d9c30bb6aaed3a2b5a7e67f3628e620392b958ed7471455243108dc1d |
| sections/rotation_v3_2_retirement.tex | 6174fc0dcaeadf6ced021b33e3f8f4390a73d0b477306d0486718074414279e5 |
| sync_hotstuff_rr_part1_core.tex | 00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3 |
| revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex | 483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034 |
| revisions/v3_1_frozen/model_and_assumptions.tex | 1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387 |

另作字符串逐字比较：v3.2 和 frozen v3.1 的整个 Model and assumptions section 均等于冻结片段。该检查包括 Conditions for progress 及其末尾相邻 prose，不是只比较编号 assumption。

## Detailed Verification Log

### 1. 冻结前提及量词

以下归纳保持 frozen Model 原范围。涉及活性时不能只引用几个 numbered assumptions 而遗漏其余内容。

| 冻结位置 | 前提及严格范围 |
|---|---|
| 5–7、30–44、46–59 | L1 authority/control prefix 完整、认证、不可变；每个 \((r,g)\) 至多一份 authority；初始 authority 固定每区域 membership/weights/\(F_i,\Delta_i,K\)、quota、正窗口/target/cooldown；recovery 只改 leader schedule。authority 按 \(r\) 索引，跨 branch 重用不得重新绑定。 |
| 61–84 | Espresso height/record order 给全序；threshold 首 crossing 在 payload 前生效；STATUS/CLEAR、matching MSIG、AUTO 依次处理。eligible 要求 ON、cooldown 到期、指定 authority 正总权重。OFF 不撤销历史 REGULAR witness；ON 不清 cooldown。 |
| 86–104 | Genesis/checkpoint 固定初始链、cyclic order、来源信任及 replay state；初始/后续 boundary 固定 reference bounds。 |
| 108–133 | 完整 regional result 包含 DATA、direct/covering FINAL 与全部验证依赖；正确 holder 立即 offer。跨区直接传输可用但无独立有界 cross-region transport 假设；否则从 Espresso 取。执行状态恢复属于 Ready，Espresso inclusion 不自动完成状态恢复。 |
| 135–153 | \(F_i<W_i/2\)，Byzantine weight 不超过 \(F_i\)，certificate weight \(>F_i\)，正确成员单独足够成证书；session 包括 recovery 的 snapshot 不变；正确副本保留状态/committed blocks；不可伪造/哈希绑定。旧 key compromise 也受明确 fault-counting 范围限制。 |
| 159–179 | 完整 finalized L1 history 共同可认证；区域内 correct-to-correct delivery 与 complete-dependency processing 受 \(\Delta_i\) 上界；共享全局时钟。没有跨区同步上界。 |
| 181–196 | 从“完整结果可用”开始 \(T_{\rm pub}\) 内，服务把 Espresso-admissible 完整记录送达 admission interface；有工作路径、fee、输入/负载界。已录入 Espresso 即满足交付，接受重试是另一任务。 |
| 199–215、272–283 | 委员会诚实协调，只提交授权、不变、future-dated 控制；没有私有 protected prefix 或选择旧 retained anchor 的权力。真正提交后 \(T_{\rm L1}\) 内所有正确在线参与者可最终验证。activation 满足 \(\theta_a>s+T_{\rm L1}+\epsilon_E+\eta\)。 |
| 218–234 | Espresso finalized prefixes 兼容、完整 namespace/真正 absence 可认证、历史可检索、global DATA 可恢复；timestamps 非递减且 \(\theta_q\le t_E(q)+\epsilon_E\)。没有 timestamp lag 或速率的双边界；任意长历史的 catch-up 没有常数时间界。 |
| 236–269 | admissible 完整记录真正提交后 \(T_E\) 内 inclusion/finality/完整可见；对象限定在线且 keeps up 者。每个 finalized block，包括空 namespace，finalization 后 \(T_E\) 内可见。明确不保证链内 acceptance。两个服务界不涵盖结果形成、state restore 或 rejected EXPORT retries；timestamp windows 另需 clock profile/startup/regional progress allowances。 |
| 285–291 | regional receipt/application 检查 sound 是 premise；certifier 认证两个来源及完整控制版本，publication 验证 execution/DA correspondence。 |

Conditions for progress（frozen 293–320）必须整体保留：

1. “Correct leaders receive repeated turns that are actually entered, including recovery initializers and DATA-producing turns across sessions.” 已暴露 equivocator 最终永久排除。
2. Espresso 持续产生可检索历史、growing timestamps，并 include 有效 exports。
3. 完整结果可供 Publish Layer；submission/Espresso 满足界；“Retried EXPORTs for the next required DATA are accepted in time during **some sustained service interval**.”
4. 所需 authority/recoverable native state 最终可用；waiting boundary 最终有“**another eligible region**”；委员会给予 working regions 足够不中断的 service time。
5. 正确客户端 admissible requests 到达继续服务的区域；适合 batch；每个 local queue 中较老请求有限。

相邻 prose 又明确：“The bounds ... do not bound regional recovery, state restoration, or repeated rejected EXPORTs”；窗口成功仍需 stated serving interval 和 compatible Espresso clock profile；否则 repeated AUTO 可以阻止 progress；Espresso failure 停止外部时间/全球进展，切换 global finality service 不在协议内。

这里至少有两个不可忽略的未形式化量词：

- “some sustained service interval” 是存在一个足够长但有限的成功区间，还是存在无限尾区间，还是每个未来 duty 都有成功区间？正文没有给出数学定义。三个命题严格不同。
- “Correct leaders receive repeated turns” 是对已经获得服务的区域中的 leader schedule 公平，还是对所有持续健康/eligible 区域内每个正确身份的全局实际进入公平？后者已经把相当一部分区域服务结果写成了 premise。

后文分别标记这两个解释。不能以一个任意强的读法排除反例，再说协议从较弱的环境条件证明了用户完整目标。

### 2. 目标与依赖图

需要分别判断：

- **持续全球进展**：对每个高度 \(h\)，最终 \(h(G)\ge h\)，不是曾经接受若干 DATA。
- **区域服务公平**：每个持续健康、满足 eligibility 的区域，在任意足够晚时点之后仍获得有效服务机会。选择、Enter、提出 DATA、其 DATA 被 global 接受是四个不同层次。
- **请求活性**：每个持续 admissible、按前提送达的正确请求最终属于 \(G\)。local commit 或反复被 proposal 包含不够。
- **观察者 catch-up**：每个固定历史前缀可重建，和一个落后 reader 最终追上动态增长的最新头部，也不是同一命题。

现稿依赖链是：canonical selection → 精确 handoff → exact-state Ready → initial Enter → 2Δ/ANCHOR bootstrap → DATA vote/timer → 完整 FINAL → bounded publication/inclusion → 在同一个仍开放 version 下及时 acceptance → \(G\) 增长。每个箭头都有独立条件。最后两个服务界不能倒推前面所有箭头有界，也不能把 inclusion 改称 acceptance。

### 3. T1：实际正确生产无限发生，但全球输出最终停止

**所反驳命题及范围。** 在 progress 的存在有限成功区间读法下，基础服务界、实际正确领导者轮次公平和最终材料可用不能推出 \(G\) 无界。该轨迹还修正了 Section 9 的原 W=1 例子可能“根本没有 DATA-producing turn”的歧义。

**配置。** 两个区域 A、B，cyclic order A→B→A，均一直 ON、健康；每区域三名单位权重成员，两正确一 Byzantine，\(W=3,F=1,\Delta=1\)。正确 leader 按无限 authority 序列交替分配给本区域两个正确身份；每个身份无限次真正 Enter、提出 DATA 并本地 commit。Byzantine 成员在这些慢轮次保持沉默，不 equivocate。所有所需 authority、exact anchor/state 都在各次 opening 前准备好。后续每个 duty 的 quota 为 1，\(m_{\rm start}=1,W_{\rm start}=6,C_{\rm auto}=1/2\)。所有 parameters 自其 authority 发布起不可变。无 MSIG/STATUS/CLEAR 干预。Espresso 持续 finalization，\(\theta=t_E\)，\(\epsilon_E=0\)；取 \(T_{\rm pub}=T_E=1\)，通过足够频繁的块兑现两个界。

先安排任意长但有限的成功前缀，其 authority 使用足够长 startup windows，所有 next-required EXPORT 必要时重试并及时接受。之后到达某个正常打开的 duty，从其真实与公有 opening 时间 \(a\) 起执行以下慢时间表。后续 AUTO successor 同样在其 opening 立即进入。

| 时间 | 事件 |
|---|---|
| \(a\) | 两个正确成员 \(H_0,H_1\) 通过 exact Ready，进入同一 \((\beta,r,0,0)\)；\(H_0\) 是正确 initial leader。 |
| \(a+2\) | \(H_0\) 按规定发唯一 NEWVIEW、投自身 ANCHOR vote。 |
| \(a+3\) | \(H_1\) 收到完整 NEWVIEW 并投 ANCHOR vote。 |
| \(a+4\) | \(H_0\) 收到 \(H_1\) vote，形成 weight 2 的 ANCHOR QC，提出唯一 quota DATA 并自己投票；其 commit deadline 为 \(a+6\)。 |
| \(a+5\) | \(H_1\) 收到完整 DATA、投票；其 commit deadline 为 \(a+7\)。 |
| \(a+6\) | Espresso 块首 crossing startup deadline；已有 accepted count=0，AUTO 先于 payload 关闭旧 version，保留全部旧 \(G\)，给源区域 cooldown 至 \(a+6.5\)，选择另一区域并打开新 duty。closure evidence 可立即可见。即使 RequestClose 先处理，旧 DATA timer 仍按 retirement 合同执行；\(H_0\) commit 并发 FINAL。 |
| \(a+7\) | \(H_1\) 的旧 timer commit；它收到 \(H_0\) FINAL，并有自身 FINAL，首次可形成完整 receipt。此前只有 \(H_0\) 一个 FINAL，weight 1 不大于 F。 |
| 最迟 \(a+9\) | 完整结果按 \(T_{\rm pub}+T_E\) 提交、包含、可验证。它的 version 已关闭，因此不能扩展 \(G\)。更早送达也不救援：最早完整结果已严格晚于 \(a+6\)。 |

所有 correct-to-correct 消息取 1 或更短延迟。首个 progress BLAME check 在 entry+6；双方已各投 DATA，不发 BLAME。没有 Quit、equivocation、timer 取消。新的程序无需等旧程序 FinishClose。下一个 duty opening 为 \(a+6\)，第三个为 \(a+12\)：前一个源区域的 cooldown 已在 \(a+6.5\) 结束，故 A、B 永久交替。DATA 的新 reference 可引用本次 opening header，EXPORT 位于其后，L1/reference/quota 全部满足。

因此每个后续 session 都有实际正确 DATA production、正确 local commitments、完整 FINAL、按界 Espresso inclusion；但每个结果都在 closure 后才完整，故 \(G\) 恒等于有限成功前缀。窗口目标不是 publication count，而是 \(G\) append count，这一点决定了结论。

**逐项假设审计。**

| 前提 | T1 是否满足 |
|---|---|
| stake/authentication/fixed authority/immutable controls | 是；两个正确签名可成证书，无超额 Byzantine weight、无伪造/重绑。 |
| authenticated synchrony/global clock | 是；所有正确链路 ≤1；真实时间统一。 |
| bounded L1/control announcement/honest committee | 是；后段无控制操作，authority 预先可用，委员会无任何打断行为。 |
| compatible Espresso/retrievability/growing timestamps | 是；\(\theta=t_E\) 严格持续增加，全部 payload 可检索，旧 \(G\) 可恢复。 |
| bounded complete-result submission/inclusion/visibility | 是；完整结果从 \(a+7\) 起才触发 service bound，按界 inclusion。closed-version acceptance 从来不是这些界的承诺。 |
| repeated entered correct DATA turns | 是；每个正确身份无限次实际提出 DATA，非仅名义排班。无 recovery 需要，不能把不存在的 recovery failure 当作反例的遗漏。 |
| eventual exclusion | 是；没有 exposed equivocator。 |
| eventual authorities/state/another eligible region | 是；材料立即可用；每次选择均有另一 ON 且 cooldown 已到期区域。 |
| correct client dissemination/finite predecessors | 可取没有请求而真；也可将一个请求送达全部正确节点，令各慢 DATA 反复包含它。 |
| some sustained acceptance interval | 存在量词下满足：前面可有任意长有限成功区间；若意指无限成功尾区间或每个未来 duty 成功，则不满足那个更强解释。 |
| enough uninterrupted service / compatible clock prose | 委员会从不打断，clock 完全正常；区域本地 timers 都完成。如果该措辞被重新解释为“包括 AUTO 在内必须给每个未来结果足够全球 acceptance 时间”，则它预先排除了此轨迹；这是必须明确的额外量词解释。 |

T1 证明的不是“所有 frozen 读法下定理均假”，而是明确存在读法下目标为假、强读法尚未被精确定义。原 Section 9 对量词的提醒是必要的，不能用 transport bounds 补出它。

### 4. T2：\(G\) 无界、各正确 leader 无限实际生产，但请求永久饥饿

沿用 T1 的两个区域、权重、窗口、公开时钟和慢 session。把正确客户端请求 \(x\) 在进入循环前送达所有正确成员：它持续 admissible、适合一个 DATA batch、各 local queue 中没有更老请求。正确 leader 的每个慢 DATA 都包含 \(x\)。所有 discarded suffix 上的 \(x\) 保持 pending，完全遵守主文 2256–2258。

加入**快 session**：本次 leader 为本区域 Byzantine 成员，但它不 equivocate，甚至也等待 2Δ 再发合法 NEWVIEW。取实际区域消息延迟充分小的 \(\varepsilon>0\)。它得到 ANCHOR QC 后提出合法空 DATA；正确成员正常投票、等待 2Δ commit，约在 opening+4+O(\varepsilon) 形成完整 receipt，立即 submit，安排 EXPORT 在 opening+5 前接受，严格早于 deadline opening+6。quota=1，其 terminal 取消本 duty 的剩余窗口。空 batch 不违反应用有效性；oldest-first 是正确 proposer 的行为义务，不能使 Byzantine omission 自动成为可验证 equivocation。主文 2064 和 2573–2574 明确允许 empty DATA count 且不据此推出请求 inclusion。

例如按 \(r\bmod6\) 重复下表（A/B 正常交替）：

| r mod 6 | 区域/leader | 类型 | global 结果 |
|---|---|---|---|
| 0 | A / Byzantine | 快 | 空 DATA 被接受 |
| 1 | B / 正确 \(H_{B0}\) | 慢 | 含 \(x\) DATA 在 AUTO 后才完整，不能被接受 |
| 2 | A / 正确 \(H_{A0}\) | 慢 | 同上 |
| 3 | B / Byzantine | 快 | 空 DATA 被接受 |
| 4 | A / 正确 \(H_{A1}\) | 慢 | 含 \(x\) DATA 被关闭 |
| 5 | B / 正确 \(H_{B1}\) | 慢 | 同上 |

每次 Force 捕获当前 export duty，令 \(r\) 增一；快 session 的 REGULAR 也令 \(r\) 增一，所以这个 authority-index schedule 可实现。正常 handoff 可选择不走 direct transfer，改走 Espresso 的允许路径，使下个慢 session 在其 public opening 才 Enter；不存在必须提前启动的同步跨区服务假设。取小的有界获取延迟只会让慢结果更晚，不会救活它。新 reference 使用足够新的已认证 header，避免历史公平性问题。所有初始 authority 按 r 提前发布，所有参数不改。

**结论。** 每六个 duty 接受两个空 DATA，\(G\) 无界增长；A/B 都无限次成功 global 服务；四个正确身份都无限次真正 Enter、生产、commit 含 \(x\) 的 DATA；所有完整结果按界进入 Espresso，但 \(x\notin G\) 永远成立。

**假设审计。** T1 的所有基础前提继续满足；请求前提现在非空且完整满足；永久 exclusion 无需触发；每个区域的正确 leader 实际轮次公平也满足，故此例不依赖 L3 的区域量词争议。成功接受区间无限次出现，初始还可以加任意长有限服务区间。因此即使把 some 加强为“无限多次出现成功区间”，仍不足以推出请求活性；需要的关键关系是正确请求所在结果最终进入可保留的 accepted prefix，而现有成功区间没有这个量词。若直接假设每个 next-required result 最终成功且不能被提前关闭，T2 被排除，但那是更强的服务前提，不能从 raw bounds 推出。

Part I 的 request theorem（930–944）不能直接迁移：其归纳每次成功正确 proposal 最终进入同一不可撤回 committed chain，消耗有限 predecessor 集合；这里正确 proposal 在 regional chain 上 commit 后仍可永远不进入 global chain。有限 predecessor 本身不提供消耗进度。

### 5. Section 9 三个已有反例的独立核验

#### 5.1 原 W_start=1 startup 轨迹（2593–2612）

原文：“Each new program enters promptly, but Protocol 4 waits \(2\Delta\) before sending NEWVIEW.” 当 \(\Delta=1\)、\(W_{\rm start}=1\)、clock=real time 时，选择后 1 秒 AUTO、2 秒才允许 NEWVIEW，轨迹规则正确；旧 gate 会阻止其后来补生产；cooldown<1 使两区域轮换可行。

但是它仅保证无限次 initial Enter，不保证这些轮次实际“DATA-producing”；若第一项 progress 要求无限次真正 DATA production，则原例违反该读法。其 some sustained interval 与 enough-service 问题同 T1。故该段不能无条件作为所有 frozen progress 解释下的定理反例。T1 是独立加强：每个正确轮次在 AUTO 前实际提出 DATA，其 local timers/完整 receipt 也完成，仍无 global progress。

#### 5.2 固定历史 \(q_0\) 排除恢复 C（2614–2623）

设 cyclic order A,B,C；\(q_0\) 的 A/B eligible、C OFF；之后在 \(q_1>q_0\) 合法 future-dated ON(C) 生效，C 永久健康且 eligible。初始 \(q^{\rm open}=q_0\)。A terminal \(\xi=q_0\) 时，NextLive(A,chi(q0))=B，所以 B 的 opening bound 仍为 \(q_0\)；B terminal 同样引用 \(q_0\)，NextLive(B,chi(q0))=A。归纳得到所有后续 opening bound/reference 都等于 \(q_0\)。新 authority 的 L1 publication height 可以增长；\(\lambda\) lower bound 与 Espresso q lower bound 分开，前者不迫使 \(\xi\) 变新。每个 EXPORT 放在 \(q>q_1\) 的新 block，严格 antecedence 也满足。

所有 active A/B sessions 使用足够大窗口、完整状态、及时 acceptance，\(G\) 无界；无 Force，C 的新 ON 不撤销旧 minimum。所有正确 verifier 都得到相同历史选择；这不是不同观察者视图分叉，而是可验证的永久旧 snapshot 选择。

假设审计：基础服务、stake、committee/announcement、state/authority、另一区域存在、A/B 实际 leader fairness、acceptance 成功均可满足。**唯一必须显式处理的冲突是 frozen 第一项的范围**：若它已经要求 C 的正确身份无限次实际进入 DATA turns，则此轨迹违反该前提；若它仅约束获得 session 的区域内 leader schedules，则全部前提满足并给出区域公平目标的 Critical Error。当前文本没有写出 \(\forall i\) 与选中/进入条件，故“模型内反例”的无条件断言存在 Justification Gap。

#### 5.3 singleton（2625–2631）

唯一 eligible A 完成 quota，terminal receipt 与 EXPORT 全部成功，\(G\) 到达其 terminal。正常 boundary source=A，所有未来历史 snapshot 只有 A eligible，NextLive(A,{A})=bot。因此 successor 永久 unassigned，窗口永久不存在，AUTO 无从触发。即使委员会发未来 MSIG，其无 assigned duty 分支继续保留 source=A 和 unopened next index；新 version 的 NextLive 仍为 bot。状态可恢复、clock/所有 raw services 正常也不能改变这个选择结果。

这确实反驳“至少一个 eligible 健康区域便能永久续产”的现行协议能力，但 frozen progress 明确要求 waiting boundary 最终存在“another eligible region”，所以它不是满足全部 progress premises 的反例。若用户目标包括唯一存活区域，不能仅复述这一前提便宣告目标完成；它把关键场景排除在定理域之外。该区分与 Section 9 的原限定一致。

### 6. Clock、Ready 与恢复的边界

主文只给 \(\theta_q\le t_E(q)+\epsilon_E\)，没有 \(\theta_q\ge t_E(q)-L\)。例如 \(\epsilon_E=0\)、opening block 在真实时间 100 最终确认而 timestamp=0，\(W_{\rm start}=10\)；下一 block 真实时间 100.1、timestamp=10。两 timestamp 均满足上界与非递减，10 秒的 timestamp window 只留 0.1 秒真实时间。新程序尚未完成 2Δ 等待便被 AUTO 关闭。这是一个合法单次 startup 失败，可置于其他轮次成功的执行中；它反驳“窗口长 W 自动给予 W 秒墙钟 startup”的推导，不单独声称构造了任意大固定 W 下的无限饥饿。

该区别重要：不能用一次 arbitrary lag jump 冒充一个永久持续、仍满足 lead bound 的快跳序列；永久轨迹仍要检查累计真实时间与 timestamp 的约束。T1/T2 刻意使用 \(\theta=t_E\)，完全不依赖这个问题。

Ready 定义要求 exact anchor DATA/receipt/native state/authority/所有依赖齐备，并在当前 processed prefix 中未闭合；candidate 下载、旧 anchor state 或未来 Force 公告不够。这个 gate 对授权是清楚的。entry-based bounds \(e+3\Delta,e+5\Delta,e+6\Delta\) 的前提明确包括未被 retirement 打断；它们不从 public opening 或 state transfer 起算，因此不能代入一个没有 restoration bound 的固定公共 deadline。区域内 complete-bundle relay 可以在首个 Ready 后把其余正确成员拉入，但不能生成首个成员尚无的 exact execution state。

在完整 progress 前提明确要求所需 state 最终可用时，不能把“exact state 永远不可得”当作模型内反例。允许每个 exact state 在有限但很晚的时刻可用，会造成个别 selected program 在其 Ready 前关闭；这种轨迹若还使所有正确 leader 永远没有实际进入轮次，也会违反强 schedule-fairness 读法，所以本报告未靠它证明无界停滞。

### 7. AUTO/MSIG races、retirement 与 publication

主文 2120–2173 的 prelude 顺序可以确定同一位置唯一 closure：独立 STATUS/CLEAR 先执行；按 \((\theta_a,c)\) 的首个 entry-version matching MSIG 执行后跳过 AUTO；此前 AUTO 已改变 version 的后到 MSIG 只作为 stale 消耗，其附带配置不应用；独立 STATUS 仍按自己的规则生效。REGULAR 只改 r，不使针对同一 beta 的 MSIG stale。这些规则没有给本次审查找到一个额外的活性矛盾。

不能用“诚实委员会无限频繁地合法 Force，永远不给任何 working region 服务机会”无条件作 frozen-progress 反例：这会直接触碰 enough-uninterrupted-service 前提。相反 T1/T2 完全没有委员会 Force，停顿来自固定窗口的 AUTO。

相同 timestamp 下第一个 crossing block 的 prelude 先于 payload，故恰好 deadline 才包含的有效 EXPORT 已太晚；把它算进旧窗口会改变协议。新 successor 窗口从 realized selection timestamp 起算，单个大 timestamp jump 不会递归 timeout 同 block 内所有 successor。这两点在反例和肯定判断中均使用原规则。

retirement fragment 96–132 的 4Δ 结论限定为 closure relay、旧 honest timers、FINAL drain；不保证 receipt aggregation、Byzantine 释放、storage transfer、Tpub、TE 或 acceptance 在 4Δ 内完成。它不等待 successor Ready，且新 session 不等 FinishClose，故不能把旧 program 的持久化退休未完成直接当作新 session 的协议级阻塞。fragment 192–203 也明确承认晚 vote 可永远没有足够 FINAL：在这种情况下 raw publication premise 根本未触发。T1/T2 更强，两个正确 vote 都在 close 前完成，完整 result 确实最终形成。

主文 2699–2718 描述 retries 的实现义务，但仍明确没有 bounded sequence of stale-base retries 保证。仅仅“record 在 Espresso 已 finalized”不证明它推动 \(G\)；T1/T2 使用 closed version 而非缺数据或虚假的 send attempt，因此不会违反这两个服务承诺。

### 8. 已有 theorem/proof 的准确范围

| 结果 | 本次审查判断 |
|---|---|
| 主文 2476–2514 Common interpretation and preservation of global DATA | 是确定性 replay、prefix preservation、closed-version rejection 的条件性安全命题。其归纳不产生任何未来 export 或 leader turn；T1/T2 全部满足其结论。活性缺口不能被传播为此命题的反例。 |
| 主文 2516–2558 Fixed entry and isolation of local programs | 是每 pair 固定 handoff、正确节点 Enter 至多一次、异步 callback 不越权、状态隔离；“at most once”绝不推出“eventually once”。本报告无由活性单独反驳它的证据。 |
| retirement 96–132 closure/drain proposition | 在完整 closure bundle 与区域同步前提下，\(c_v\le c+\Delta\)、existing timer ≤\(c+3\Delta\)、实际生成 FINAL ≤\(c+4\Delta\) 的时间推导与本文 T1 相容；不推出 G acceptance。 |
| retirement 134–203 conditional regional safety | 本报告不独立认证所有 baseline regional safety；只核对其显式不保证 complete-receipt/old-session progress 的范围，未用它推导活性。 |
| Part I 896–944 height/request liveness | 是其固定 regional core、authority/exclusion/fairness 下的 committed chain 命题。Part II 的 finite quota、Force retirement、global acceptance filter 使不能直接移植无界高度或 request 结论。 |
| 主文 2584–2637 | 是未完成目标与候选失败模式说明，不是全活性证明。其量词提醒正确；历史公平性反例还需要补做上述 schedule-fairness 范围审计。 |

### 9. 对“原始假设不变”的最终边界判断

当前规则下，完整目标未实现；不能把它替换为“全球前缀不会回滚”或“偶尔存在成功区间”。至少有三个直接的协议约束阻挡原目标：只选 **other** 的 selector、可永久复用历史 \(\xi\) 的 REGULAR 选择、以及在 Ready/receipt/acceptance 之前运行且可永久撤销正确输出资格的固定 AUTO budget。请求公平还需要把实际正确生产机会与最终 retained global acceptance 联系起来；目前没有这一推论。

本审查没有证明“任何协议都无法在相同环境下实现活性”，也没有提出替代协议/修复证明。必须先说明目标保留哪些外部条件：即使一个区域存活，若没有 Espresso 持续 finalization、所需 authority/state 最终可用或请求送达，原环境就没有足以保证目标的资源。完整 frozen progress 本身又排除了永远没有 another eligible region 的 singleton 场景。保持原假设文字不变并允许协议变化，不等于可以静默用这些条件排除用户明确要求覆盖的场景；需要在命题域和证明中明确处理。

只将 service-interval 和 repeated-turn prose 解释得更强，可以排除部分反例，但不能当作协议已建立对应结果。此次验证结论是：**基础及局部安全接口的若干范围清楚；完整持续/区域公平/请求活性没有通过独立审查。**

验证方式：源文本逐步数学/状态机审查、完整反例轨迹与逐项 premise audit、SHA256/Model 字符串一致性核验。未执行模型检查、代码仿真或 end-to-end 系统测试；不作已完成 secure-protocol 认证。
