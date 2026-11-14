# v3.2 独立形式证明审阅

## Summary

**Final Verdict:** 当前论证可成立但尚不完整（**viable but incomplete due to a Justification Gap**）：`4Δ` drain、retirement 的条件安全论证及固定 handoff／局部隔离论证通过本次逐步检查；首次 Ready／Enter 后所有正确成员在 `e+Δ` 内入场的推导尚未由冻结模型充分支持。没有据此确认区域或全局 prefix safety 的反例，也没有将条件性的 baseline soundness 前提误判为循环论证。

**List of Findings:**

- **F1 — Justification Gap；首次入场偏差及其 bootstrap 时间界。** **Location:** 主文 1929–1940：“the complete-bundle relay and existing regional communication bound make all correct members enter by `e+Δ`”。**Issue:** Ready 同时要求可继续执行的 native state 已恢复并验证（1882–1883）；现有区域同步界明示覆盖消息及其完整验证依赖（376–388、903–912），却没有明确建立“一个成员 Ready 后，它的 relay 足以使其他成员完成 native state 恢复／安装，并且这一工作也在同一个 `Δ` 内”的接口蕴含。冻结模型明确不为 state restoration 给出时间界（模型 262–265、307–315）。因此不能仅由接收完整证明材料推到每个成员已经 Ready。`e+3Δ`、`e+5Δ`、`e+6Δ` 依赖这一未证步骤，随之未获验证。此表述已见 v3.1 的 1607–1617，是继承缺口，不是 v3.2 新增的环境假设文字。

**需要完成的工作（不在本审阅中修复）：** 明确并证明 Ready relay 对接收方 executable-state readiness 的充分性及其时限适用范围，或者收窄当前时间界使其不依赖尚未得到的 native-state 恢复上界。不能把“恢复时间计入 `Δ`”作为未声明的新假设来让现有证明通过。随后重新核对需要 initial-entry skew 的 Part I 引用。其他通过的结论仍保持下文列出的条件范围。

### 审阅对象、隔离与验证方式

只读取本目录 `snapshot/` 内指定的主文、retirement、冻结 v3.1 模型／主文、Part I，以及 proof-verifier 技能；未读取其他评审、preflight 或作者工作历史。未修改协议源码。此报告是源文本数学审阅，不是模型检查、执行测试或编译验证。

以下缩写的行号均指本次不可变快照：

| 缩写 | 文件 | SHA-256（已实际计算） |
|---|---|---|
| M | `snapshot/sync_hotstuff_rr_part2_rotation_v3_2.tex` | `956b733d9c30bb6aaed3a2b5a7e67f3628e620392b958ed7471455243108dc1d` |
| R | `snapshot/sections/rotation_v3_2_retirement.tex` | `6174fc0dcaeadf6ced021b33e3f8f4390a73d0b477306d0486718074414279e5` |
| I | `snapshot/sync_hotstuff_rr_part1_core.tex` | `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3` |
| V | `snapshot/revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` |
| A | `snapshot/revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |

M 的整个 `Model and assumptions` 段（210–530）与 A **逐字节相同**；其提取后的 SHA-256 也是 `1309fa5f...`。这确认文字冻结；语义适用性另作以下检查，不能由字节一致替代。

## Detailed Verification Log

### 1. 依赖图与验证边界

| 结论 | 实际依赖 | 本次结果 |
|---|---|---|
| `retirement-drain`，R 96–118 | 完整 closure 证据；正确成员区域 relay 的 `Δ`；投票时启动且期限不变的 `2Δ` timer；gate 禁止新 timer；FINAL 区域广播；消息优先 | 通过 |
| `retirement-safety` 的同 generation 分支，R 169–177 | 完整 signed histories 的 fork-or-proof 性质；正确投票者转发；保留证据；真正 Quit 仍执行；消息优先 | 在所述 baseline 区域接口下通过 |
| `retirement-safety` 的 early 分支，R 158–161、178–184 | baseline protecting-QC／recovery-lock／unique-extension 论证及 baseline Completion 处理；close 前传播窗口不受 gate 影响 | 条件性通过；不把 baseline 接口视为本文已重新证明 |
| `retirement-safety` 的 late 分支，R 162–167 | closure 在 `c+Δ` 传播；真正 Quit 后至少等待 `Δ` 才能 entry；每份 QC 有正确 signer；close 禁止 entry 和 vote | 通过；无需对 late timer 声称 protecting QC |
| Common interpretation／global prefix，M 2476–2514 | 共同 checkpoint、兼容完整源历史、确定性解释规则、sound regional/native validity 接口 | 条件性通过 |
| Fixed entry／isolation，M 2516–2550 | predecessor 的唯一有效 terminal DATA；不可变 authority；确定性 selection／Force；精确 Ready gate；durable at-most-once entry | 条件性通过 |
| `e+Δ` 入场、`e+3Δ/5Δ/6Δ` bootstrap，M 1929–1940 | 首位 Ready 的完整 bundle 能在 `Δ` 内使所有成员满足全部 Ready 条件 | F1：未建立这一蕴含；后续时间界未验证 |

Part I 的原始链条是：initial-entry skew（I 153–156）与共同 authority availability time（I 173–181）→ entry/freeze lemma（I 602–628）→ timer protecting QC（I 749–776）→ recovery lock（I 778–793）→ unique extensibility（I 795–815）→ prefix safety（I 817–830）。Part II 另有 stake 权重、DATA/DRAIN、FINAL 和 Completion。R 134–147 明说其对这些基线接口作条件依赖；此处没有把 Part I 的原文自动当作 Part II 全部修改的无条件证明。

### 2. 冻结假设的语义审计

> “The delay bound includes delivery and processing of complete validation dependencies.”（M 386–387；A 177–178）

它足以用于完整 closure 证明包的区域传播：R 14–24 指定完整 triggering header、控制及 namespace 历史、checkpoint 和 closure context；R 108–110 的发送者已经完成验证并发送完整证明包。没有从 Espresso finality 本身推断即时可读性，也没有从首次签署控制、Espresso timestamp 或开始历史追赶计时（R 120–126）。此处没有新加跨区域输送服务。

> “The bounds ... do not bound ... state restoration”（A 314–315；对应 M 523–525）

这不能被 M 1893–1899 的 “existing validation interface” 自动替换成 executable-state recovery 的时限承诺。正确性／soundness 与完成时间是不同性质。模型只要求必要 state 最终 available（A 307–309），且任意历史追赶没有常数时限（A 229–234）。因此 F1 是语义检查结果，尽管模型文字未改。

其他没有发现暗中加强的项目：retirement 不改变固定 voting snapshot、`F<W/2`、certificate `>F`、故障界或 authority；没有把诚实 multisig 的健康判断提升为新的 synchrony 证明（M 1577–1578）；不为 storage ownership transfer、receipt formation、publication retry 增添期限（R 128–132）；不把“至少一健康 region”当作现有 NextLive 的充分活性条件（M 2570–2582）。

### 3. `4Δ` drain 的逐步核验

> “every correct member has closed by `c+Δ`”（R 98–100）

取首个正确 RequestClose 时间 `c`。其完整区域 relay 经现有同步界在 `c+Δ` 前送达并处理；每个接收方重新验证而不是信任裸通知。对已经运行的正确实例，有 `c≤c_v≤c+Δ`。未入场目标则必须同样受 closed-version／entry gate 的持久记录阻止，不会在消息到达后再启动旧版本。

> “A correct DATA/DRAIN vote at time `s=τ_v(P)` starts its only timer”（R 110–113）

由 M 1093–1100，duplicate 不重启 timer，QC 不启动 timer（M 1107–1109）。由 R 29–36，close 禁止新 vote、timer、term/generation entry，并覆盖 callback 和 buffered processing。因此每个幸存 timer 的启动时刻 `s≤c_v≤c+Δ`，到期 `s+2Δ≤c+3Δ`。Quit／Completion 只能取消，不能延期（R 37–51）。算术与量词正确。

> “FINAL messages ... reach every correct member by `c+4Δ`”（R 101–103）

真正到期的 timer 发送 FINAL 给固定区域成员及 publication collector（M 1103–1107；R 42–46），再用一次区域 `Δ` 得到 `c+4Δ`。对任何本地 `d_v=c_v+4Δ`，因 `c_v≥c` 而 `d_v≥c+4Δ`。消息优先规则覆盖到期／retirement 同时发生的等号（R 55–57、117）。

这个界只覆盖诚实 timer 事件及其直接 FINAL 区域送达。它不保证 FINAL 权重达到 `>F`，不保证 receipt relay 完成，也不保证 Byzantine message release、持久存储转交或 EXPORT 接受。R 64–85 的额外 guards 是必要条件，不能被“等到 `d_v`”替代。文本明确保留这些 guards，因此没有误把保守 drain threshold 当作整个关闭过程的截止时间。

### 4. Retirement 条件安全证明

> “under ... the baseline soundness premise for DATA/DRAIN, receipts and Completion”（R 136–137）

这是对**未加 retirement gate 的基线区域接口**的明确条件依赖，不是把“retirement 安全”本身作为前提。因而不能仅因它依赖 baseline soundness 就判为循环证明。本文也没有由此重新证明 Part II 基线的所有修改。尤其 Part I 的纯 DATA 证明没有 FINAL／terminal Completion 规则，原文不可未经检查直接套用。

#### 4.1 幸存 timer 与真正 Quit 的时间关系

> “If the first correct genuine Quit ... were at `a≤s+Δ`, its proof would reach the committer by expiry and cancel this timer.”（R 147–151）

正确 Quit 广播证明（M 1184–1192）；retirement 保留这一处理（R 37–41）。故 `a+Δ≤s+2Δ` 时 timer 不可能幸存，等号由消息优先处理解决。于是有后续 Quit 时必为 `a>s+Δ`，而每个正确下一 generation entry 满足 `e_v≥a_v+Δ≥a+Δ>s+2Δ`。此处只需 entry 的**下界**；没有使用所有成员同步入场的上界。

#### 4.2 Early case

> “If `s+Δ<c`, the retirement gate suppresses no vote during ... propagation window.”（R 158–161）

首个 close 在 `c`，所以到 `s+Δ` 为止没有成员因本次 gate 被禁止原本应投的 matching vote。完整 proposal／signed history 传播、existing timer 的处理和 QC 收集保留。若 baseline protecting-QC 与 Completion 安全接口适用，这个 case 没有破坏其所需的投票窗口；随后到达的 QC 不因 close 丢弃，existing lock 不被改写。

这段正确依赖 baseline lemma 的原有适用条件。F1 意味着不能仅凭 M 1929–1932 把所有启动实例都认证为满足 Part I 的 initial-entry-skew 条件；不能把条件通过改写成“仅由冻结环境假设已独立完成区域全证明”。

#### 4.3 Late case

> “If `s+Δ≥c`, its expiry is at least `c+Δ`.”（R 162–167）

`s+2Δ≥c+Δ`；在该到期时刻所有正确成员都已关闭 fresh-action gate。第 4.1 步又说明任何下一 generation entry 必须严格晚于 timer 到期，所以这样的正确 entry 已被 gate 禁止。任何更高 generation 的 QC 必须有正确 signer（M 346–352），因此不能通过另一 recovery 利用缺失的 matching votes。既有更高 generation 的正确 vote 也不能逃过论证：其 entry 本来就会与第 4.1 步的严格时间下界冲突。此分支不依赖 late timer 形成 protecting QC。

#### 4.4 同 generation 的 fork 与跨 generation 的比较

> “complete signed histories either extend one another or expose ... equivocation proof”（R 169–177）

固定 opening 下，quota／position 规则为每个 fresh position 指定唯一 leader；不同 opening 由两个有效 NEWVIEW 构成证据。DATA 与 recovery-only DRAIN 各遵循其 admitted 类型及 position 规则（M 663–720、1251–1268）；不能把无效超 quota proposal 当作有效 fork proof。正确 voter 转发完整历史并保留证据；若另一正确 signer 已先看见不兼容历史则不能投票，否则它的冲突历史最迟在第一份 proposal 的传播窗口内发出，至迟在首个 timer 到期前暴露证据。retirement 仍做 evidence-first Quit，因此这一反证未借用关闭后新的 matching votes。

跨 generation：若存在更高 generation 的正确 vote，较低 generation 的幸存直接 commitment 不能是 late case；early case 通过 baseline protecting-QC／lock／unique-extensibility 关系得到 extension。祖先 commitment 随直接 commitment 继承可比性；receipt-based commitment 由 `>F` FINAL 中的正确 signer 追溯到真实 commitment（M 807–855），沿用 baseline Completion 的安全接口。没有 retirement 步骤撤销任何 commitment。

R 192–203 的三成员例子正确说明：关闭 H0 后，H1 可合法投票并由其 timer commit，而 Byzantine 成员不贡献 vote／FINAL；此时 H1 的单份 FINAL 权重不超过 `F`，既无 proposal QC，也未必有完整 receipt。故原 universal protecting-QC 结论确实不可全盘复制；本文已经明确排除了这一错误结论。Publish Layer 的完整结果前提也不能补出签名。

### 5. First-Ready／Enter 时间界：F1 的具体位置与后果

> “Ready ... native execution state ... has been recovered and verified”（M 1876–1889）

Ready 并非“已收到可验证 handoff”。它还要求 executable state、本地 entry 记录、当前 processed public history 等条件。M 1796–1808 的 typed handoff 列出 anchor、authority、history 等材料；M 1897–1899 要求 gate 通过后 relay 完整 validated bundle，但没有定义该 bundle 怎样消除接收方尚未完成的 executable-state recovery，也没有将 state installation 明确纳入 M 903–912 的 complete-validation processing。

冻结模型没有排除如下时间安排：H0 在 `e` 时已经恢复 state 并 Enter；H1 在 `e+Δ` 前收到、认证全部 source／handoff 材料，但其独立的 native state materialization／recovery 直到更晚才完成；期间 fixed handoff 仍有效且没有 retirement。H1 按 Ready 定义继续 Waiting。这里不是主张任何特定 native adapter 必定如此，而是当前抽象接口未给出排除它的条件。若 intended bundle 本身含可直接使用的完整 state，仍需证明其接收、验证和变为可执行状态确实全部属于现有 `Δ` 的适用范围，不能仅从“bundle complete”字样省略这一环。

原子 Ready/Enter（M 1654–1659、1907–1914）消除了**同一个成员**通过 Ready 后的排队延迟，却不能限制不同成员**达到 Ready 之前**的差异。把计时原点从开始 transfer 改为首位 Ready 也只消除了首位成员的恢复时长，不能自动消除其他成员尚未完成的工作。

因此本审阅将 F1 判为 **Justification Gap**，而不是声称已证明整个安全 theorem 为假。假定入场偏差结论暂时成立，后面的算术是正确的：initializer 至迟 `e+Δ` 入场，再等 `2Δ`，NEWVIEW 至迟 `e+3Δ`；下发加回传至迟 `e+5Δ` 得 ANCHOR QC，再下发至迟 `e+6Δ`。但是这些界目前都依赖 F1，不能独立宣称验证通过。

### 6. 固定 pair、handoff validator 与局部隔离

> “each opened pair ... has one fixed handoff”（M 2518–2536）

REGULAR 分支需要的是**同一 predecessor 的 terminal DATA 唯一性**，不能仅由“terminal height 相同”推出相同 hash。所述 regional-validity／baseline safety 前提提供这一点；`ξ` 是 DATA hash／签名绑定的一部分（M 624–639），所以同一 terminal DATA 也固定历史 selection 起点。最小 `q_s` 和固定 cyclic order 固定目标 region；不可变 authority 固定配置。不同 receipt signer set／encoding 不构成不同 DATA 或不同 semantic handoff。

Force 分支由公共 replay 唯一确定 old/new version、pre-payload `G`、捕获的 source context 和 selection 起点（M 1841–1848、2145–2156）。正常 handoff 保持 `β` 并增加 `r`；Force 增加 `β`；未分配期间重复 Force 保持 predecessor index 但仍更换 `β`（M 1758–1764）。因此不会和已打开的同一 pair 重用。局部“拒绝不同 fixed fields”只防本地替换，不能单独证明全局唯一性；此处证明先给出了共同来源及 induction，再使用拒绝替换，顺序正确。

对 `VerifyHandoff`（M 1812–1862）的规范级完整性检查如下：

| 检查域 | 已覆盖的条件 |
|---|---|
| 共同输入 | chain/domain/type、授权 checkpoint、完整相容 history、完整 applicable control prefix；缺证据为 needs-evidence |
| REGULAR | predecessor 固定边界、authority/quota、精确 terminal height、terminal FINAL、ancestry、signed `ξ`、同版本与 `r+1`、版本仍开 |
| MSIG | 授权 finalized L1 record、sequence、first crossing、整个 prelude 的首个 matching Force、stale／同 prelude 其他 instruction 排除 |
| AUTO | accepted DATA count、fixed window、first deadline crossing、无高优先 MSIG、realized timestamp cooldown |
| Force 边界 | 唯一 replay transition、`β+1`、stored source context、精确 pre-payload anchor/provenance、intervening Force supersession |
| selection／anchor | 最早 eligible `q_s`、不可变 authority、两种 reference lower bounds、原 producing pair、direct／covering evidence、完整 ancestry |
| 本地安装 | 与 packet 和已有 fixed record 比较、equal duplicate、禁止 replacement、Enter 前按当前 processed prefix 再检查 |

未发现一个被上述形式证明必须使用、却完全遗漏的 mode-specific 检查。它是“不独立修改公共状态”的 validator；第 (vii) 项还读取 local fixed record／current processed prefix，所以严格实现为只接受 `(C,H,P)` 的纯函数时，必须显式传入这些验证上下文或将安装检查留在 manager。现有文本没有宣称它可忽略该上下文；此处不是新的 safety finding。Native-state readiness 属于独立 Ready gate，不能误称 pure handoff 验证成功已证明 state 可执行或其恢复有界。

at-most-once Enter 由 durable flag、serialized Ready/Enter 与签名前持久化支持（M 1654–1664、1907–1914）；full pair／anchor context 防止旧 callback 替换固定 anchor（M 1726–1741、1984–1991）。Waiting 不签名、不启动 timer；已终止记录保留 tombstone。因而局部隔离与 at-most-once 结论不依赖 F1 的所有成员同步入场时间界。

### 7. 外部公共 prefix 证明与 retirement 的依赖方向

> “Induct over ... its prelude and ordered EXPORT records”（M 2487–2503）

在共同 authenticated checkpoint、完整 applicable source history 和共同 deterministic validation predicate 的接口范围内，prelude 的 STATUS/CLEAR 顺序、first matching MSIG、AUTO、historical selection 均确定。EXPORT 在临时状态整体检查，成功只 append 精确 next continuous range，失败不改 `G`（M 1389–1419、2225–2229）。缺失 evidence 是暂停，不是零 count 或 malformed record（M 1425–1439、2215–2219）。故对同一验证输入的 induction 成立。

> “Force retains all of `G` before payload”（M 2505–2513）

各事件均不删除 `G`；Force 不重命名旧 DATA 的 producing pair，closed version 不再有 append 权限。不同 Espresso 前缀的观察者处于同一有序状态序列的不同位置。retirement 不选择 retained anchor、不授权 EXPORT、也不写 `G`（R 186–189），故不会为这个外部 induction 增添分支。

该结论仍以 regional/native validity 的 soundness 为前提；公共顺序不能替代区域协议的独立安全证明。反过来，F1 的启动时间缺口也不会直接推翻这个条件性的外部 interpreter 证明。本文没有从 `4Δ`、局部 Terminated 或一次合法 receipt 推导已经完成全局 publication／application execution。

### 8. 本次核验结论的精确范围

已逐步验证 `4Δ` 的算术、时间起点、等号和消息优先、Quit 的严格 entry 下界、early/late case 覆盖、late case 无需 protecting QC 的理由、同 generation retained-evidence 机制、固定 pair／anchor/provenance、at-most-once entry，以及公共 prefix 的条件 induction。

唯一列为本次 finding 的缺口是 F1。未修复它，也未假定 native recovery 有界。Retirement 通过的是其**明示 baseline 接口前提下**的增量安全论证；未将“baseline 已声称 sound”当成对全部 Part II 基线和 application adapter 的重新认证。没有运行形式模型或声称可执行验证通过。
