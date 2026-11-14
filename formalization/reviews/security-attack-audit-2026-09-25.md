# main.tex adversarial security audit

审查日期：2026-09-25。对象：`formalization/main.tex`，1350 行，SHA-256 `1fc6cbfa619412b506d0d4f6d9d8b911e89609c8c03cdf1f858e9db148ed549e`。本报告没有修改协议正文。

## Summary

**Final Verdict:** 在全文明确要求的同步、固定诚实 quorum、持久化签名历史、完整对象交付、minimum tenure，以及健康 successor 等条件全部成立时，本次审查没有找到打破 finalized `Deliver` 的 agreement 或 validity 的具体攻击。发现一项恢复能力限制，可反驳 Progress 定义较宽的读法；另有两项应澄清的接收端或配置条件。它们不能统一表述为“已推翻全文有条件安全定理”。

| Finding | 分类 | 影响 |
| --- | --- | --- |
| F1：区域恢复后仍缺少 predecessor RC | 恢复协议缺口；Progress 定义范围问题 | 没有其他可接管区域时，数据流可永久停止；不产生两个 finalized 分叉 |
| F2：跳过多次 handoff 后按高度清理 preview | 接收端状态规范缺口 | 可能留下与新 retained prefix 冲突的旧 preview；尚不构成 Deliver agreement 反例 |
| F3：Progress 没有直接绑定 tenure/admission 条件 | 定义与配置范围问题 | 若忽略正文后部的参数要求，可在全诚实执行中永久 rotate 而不 adopt |

## Detailed Verification Log

### F1. 暂时的区域延迟异常可留下永久无法结束的 instance

**相关位置。** [R3–R6](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:477) 允许 fast decider 保留原决定，completion 的其他节点采用其输出；没有前驱 `RC_u` 就不能进入下一 instance。[恢复规则](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:640) 禁止给已经过期的 instance 补投票。[T3](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:710) 只允许选择 `j != i`。相较之下，[Progress 定义](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:270) 的第一句较宽，而 [Service liveness theorem](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:994) 明确要求健康 eligible successor。

**具体执行。** 令总权重为 1；当前区域 `V1` 权重 `19/20`，其中四个等权成员为诚实的 A、B、C 和 Byzantine D。另一区域 `V2` 权重 `1/20`，其成员诚实但不可用。于是：

\[
\operatorname{wt}(H_G)=3(19/80)=57/80>2/3,
\qquad
\operatorname{wt}(B)+\operatorname{wt}(V_2)=23/80<1/3.
\]

局部 Byzantine 比例为 `1/4 < 1/3`。A、B、C 始终满足全局 backbone 时限。取 `Delta_G` 足够大，使下面几秒的区域延迟异常仍不违反全局时限。所有其他容量和 minimum-tenure 条件都可以满足；攻击不依赖过短的 geographic timeout。

1. 当前 instance `u` 的诚实 leader A 提出合法 `x`。A、B、C 收到 proposal 并各自发出唯一 VOTE。
2. 调度让 A 在 `2 Delta_R` 前收到三票，因此 fast-decide `x` 并签署 `DECIDED(x)`。把送往 B、C 的必要 VOTE 和完整 quorum 转发延迟到 `3 Delta_R` 之后。
3. 在 `3 Delta_R` 的 freeze 时刻，completion 输入分别为 `x, bottom, bottom`。此时恢复区域同步；D 在 completion 中广播 bottom，或不提供对 x 的支持。
4. completion 正常结束；x 的支持只有 A，不超过 `2 W1/3`，故输出 bottom。A 按 R4 保留 fast decision x；B、C 决定 bottom。
5. D 拒绝签署任何 DECIDED。x 只有一份诚实签名，bottom 只有两份，均无法形成所需的三签 quorum。因而 `RC_u` 永远不存在。
6. 从此 A、B、C 的网络持续健康，且有诚实 producer 和足够容量。然而 R6 禁止进入 `u+1`；持久化 sign-once 规则也不允许把两种决定重新合并成证书。
7. 全局 epochs 和 FFG 仍能正常推进。但 V2 没有 READY quorum，T3 又排除了当前 V1。于是只能不断 HOLD，数据 tip 永不增长。

**精确结论。** 这是“区域网络恢复即可恢复服务”的反例，也暴露了 Definition 3.1(iii) 第一项承诺应有的前提。初始失败 instance 不满足局部 whole-instance synchrony，因此它不反驳 Regional broadcast theorem；执行中没有另一健康 successor，因此也不反驳现有 Service liveness theorem。它不是 finalized-history fork。

**建议 patch：允许 finalized same-region generation reset。** 当 no-progress 已到阈值、没有其他 READY 区域、当前区域本身有针对 retained global parent 的 READY quorum 时，允许全局 ROTATE 选择当前区域，但必须：

- 分配新 generation `a+1`，保持全局 parent 的完整 `(h,q)`；
- 完成现有 ROTATE → CONFIRM → finality → 固定 activation boundary 流程；
- 从新的 `R0` 和 instance 1 开始，消息仍绑定新 `(c,a,u)`；
- 保留旧签名历史；仅丢弃未被全局保留的 preview。

不应在旧 generation 中直接清空 DECIDED 状态并重签。新 generation 才能避免复用已经发生冲突的签名域。这个补丁沿用现有保留全局 prefix 的安全结构，但新增选择分支及恢复 liveness 仍需独立证明。

若不增加协议能力，最小文字修补是：Progress 明确要求从共同 certified predecessor 开始的完整 live instance；一旦 instance 在失败期间无法取得 RC，恢复保证还要求另一个健康 eligible successor。

### F2. 跳过多代的 consumer catch-up 不能只比较 retained height

**相关位置。** [D6](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:582) 要求 handoff 扩展最新 finalized anchor，但清理 preview 的文字是 “Remove only preview entries beyond that retained tip”。[checkpoint 验证](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:396) 要求验证缺失 ancestry，却没有明确要求把其中每次 handoff 的本地 preview 清理操作逐次执行。

**具体状态序列。**

1. P 尚未获得新全局证据，持有 generation 0 的 preview `101..110`，其中 `M101=X`。
2. generation 1 的 rotation 只保留到 100；新区域产生另一合法分支 `101..105`，其中 `M101=Y != X`，并被全局 adopt。
3. generation 2 的 rotation 保留新分支到 105。
4. P 一次性追到 generation 2。若实现只对最终 retained height 105 执行 D6，则只删除旧 `106..110`，错误保留含 X 的旧 `101..105`。

**精确结论。** 这是允许出现的 preview 状态合并错误；逐次执行全部中间 handoff 的实现不会发生此问题。因此应归类为规范缺口，而不是所有合规实现必然被攻破。D5 独立校验 finalized prefix，仍会阻止把 X 当作新分支的 finalized entry；本例本身未打破 Definition 中 delivered-log agreement。

**建议 patch。** 明确要求按 ancestry 顺序应用每次 handoff；或把 D6 定义成基于 digest 的 prefix reconciliation：旧 preview 只有在其每个位置与新验证的 retained prefix 相容时才能保留，发生首个 digest 冲突即清掉其后缀。若缺少验证 bytes，先清空不确定 preview 并重新拉取，不能只凭高度保留。最终 `preview` 必须是当前已验证基点的连续延伸。

### F3. 必须把 minimum tenure 写入 admissible configuration

**相关位置。** [Progress](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:270)、[ROTATE 优先于采用当轮 extension](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:710)、[whole-outcome admission 和 tenure 要求](/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex:1067)。

若忽略后文的 minimum-tenure 要求，配置 `N_G=1`，所有区域都诚实且健康：新 generation 激活的第一 epoch 起点尚无新 outcome；在该 epoch 内生成并认证一批数据。下一 epoch 的 report 已携带它，但 planned rotation 已 due，T3 保留 parent tip 并丢弃本轮 extension。随后 CONFIRM，激活下一区域，重复。全局控制链持续 finality，数据 tip 却永不增长。

**精确结论。** 该配置违反正文已有的 tenure 不等式，因而不是满足全文条件的反例。问题是 Definition 对 admissibility 的引用不够直接，不能把附加条件省去后仍声称满足其 Progress。

**建议 patch。** 在 `Cfg`/`Theta` 的 admissibility 中显式纳入 `tau_R`、`T_G`、完整 outcome admission 和 minimum-tenure 条件，并让 Progress 引用它。涉及 `Q`、state transfer 等 workload bound 的部分应明确作为运行环境前提；不能声称静态配置检查本身即可验证未来网络或负载。若改成 planned rotation 至少等待一次 adoption，则必须另外保留有界故障切换机制，以免真正失败的区域永久阻塞换区。

### 已检查但未形成攻击的路径

- **同一 instance 两个 RC：** 两个大于 `2/3` 的 DECIDED quorum 必须共享诚实 signer；持久化 sign-once 阻止双证书。此处安全性并不要求故障区域成功结束内部 broadcast。
- **晚揭示 VOTE QC 冒充决定：** consumer 需要 DECIDED certificate，不能只凭原始 VOTE quorum 输出 Preview。
- **旧区域在 handoff 后继续生产：** 可向 stale consumer 提供旧 preview，但 effective-generation 检查阻止它进入新的 global adopted prefix。
- **仅保留已 finalized 数据而回滚已 adopted 数据：** ROTATE 保留 parent tip，因而也保留已 adopted、尚待 finalization 的部分。
- **替换 DATA supplier 或 transaction bytes：** supplier 与 producer 分离不会绕过原始 ORDER 签名、body commitment、parent continuity 或 RC 检查。
- **cap-two authenticated broadcast：** 在所要求的完整对象同步交付和 n 个 identity rounds 下，保留两个值后仍继续转发的规则足以传播歧义；不需要所有诚实节点保留同一对值。
- **不同 signer subset 导致 checkpoint hash 分叉：** 334–336 行已排除 certificate witness 编码进入 semantic identity，不能把这项当成已成立的攻击。实现必须递归落实这一规则。

交叉核对的原始来源：[2-delta Byzantine broadcast，Appendix C.1](https://arxiv.org/pdf/2102.07240v3)、[Casper FFG](https://arxiv.org/abs/1710.09437)。这里的结论以本稿具体算法为准，并没有直接套用 Gasper 或外部协议的整体安全性。

验证方式：分区域、全局、receiver 三条路径独立人工审查；显式构造消息调度；用精确有理数检查 F1 的所有权重和 quorum 不等式。未进行实现级复现或机器证明。交易 censorship/fairness、EVM execution correctness、动态 membership 和 historical-key compromise 明确不在当前定义范围，不能把缺少这些保证算成对本定义的攻击。
