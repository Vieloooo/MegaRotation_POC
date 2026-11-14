# 两轮强最终性 BFT 文献综述：理论保证、通信复杂度、实验与选型

**调研日期：2026-11-02。用途：通用选型，按多个应用场景分别讨论。**

研究范围为 FaB / SBFT 到 Simplex、Banyan、Kudzu、Alpenglow、Hydrangea / Hydrangea++、Minimmit 和 ChonkyBFT 的低延迟 BFT 路线。重点是“一轮提议加一轮投票能否形成强提交”，以及如何与较小 quorum 的慢路径安全地并存。Simplex / DispersedSimplex 和 Commonware Simplex 作为三轮基线。

本文是基于原论文、作者资料和实现仓库的文献评议；“论文给出了证明”“有研究实现”“已部署”分别记录。本次没有重新证明各协议、复现实验或审计实现。表中缺少同口径数据时保留空缺。

按问题阅读：[安全与活性](#2-研究路线与核心理论保证) · [通信复杂度](#3-理论通信复杂度) · [实验](#4-实验结果与证据强度) · [实现状态](#5-公开实现及后续发展) · [多场景选型](#6-按多个场景给出选型建议)。

## 阅读结论

- **安全模型优先于轮数。** “$2\delta$”通常是诚实 leader、及时可用的有效提议及良好通信条件下的提交时间；安全性则需要覆盖模型允许的异步、恶意 leader 和证书隐藏执行。
- **两轮强提交已经有理论与实现依据。** Kudzu / Banyan / Hydrangea 的快路径是直接提交；Commonware Simplex 文档中的两跳 optimistic finality 使用额外条件，不能与这些快提交的证据保证直接等同。
- **三轮不必在真实网络中更慢。** 等待较大 quorum 的一轮投票，可能慢于等待较小 quorum 的两轮投票。Banyan 的实网实验和 Minimmit 的模拟都支持这一判断。
- **Kudzu 的优势主要是组合设计。** 它结合并发快慢路径、有限的每 slot 投票分支和均衡的数据分发。本文确认了公开模拟代码，尚未确认原作者维护的完整独立网络实现。
- **Minimmit / ChonkyBFT 用简化路径换取较低的 Byzantine 容忍比例。** 常用的 $n\ge5f+1$ 配置中，提交依赖约 80% 的票；快速推进 view 并不等于低票数即可最终提交。
- **混合 crash 模型有实际意义，但必须写清故障类别。** Hydrangea 的额外 crash 容忍是独立参数；Alpenglow 的扩展活性涉及不同网络假设。二者不能只按“20+20”这个口号对齐。
- **可用代码的成熟度影响选型。** Commonware Simplex 有工程化接口；Solana Votor 有生产集成；Banyan / Hydrangea 公开代码主要是研究原型；ChonkyBFT 的公开仓库现已归档。强两轮理论能力不自动对应可直接集成的成熟库。

## 1. 统一比较口径

### 1.1 参数和延迟

| 符号 | 本文含义 |
|---|---|
| $n$ | 同一委员会中的节点数；stake 加权协议另行注明 |
| $f$ | 配置允许的 Byzantine 节点上限，不等于每次执行的实际故障数 |
| $p$ | 配置的快路径故障预算；各论文对其定义和范围可能不同 |
| $c$ | 在混合故障模型中，除 Byzantine 节点外额外允许的 crash 节点数 |
| $\Delta$ | 良好通信阶段的假定延迟上界，通常用于设置 timeout |
| $\delta$ | 实际消息传播延迟；异质网络中应用 quorum 到达时间表达，而非假定所有链路相同 |
| $\beta$ | 一个区块 payload 的 bit 长度 |
| $\lambda$ | 本文统一用作签名份额 / 压缩签名证书的 bit 长度 |
| $\kappa$ | 本文统一用作哈希输出长度 |

Shoup 的 DispersedSimplex 论文对 $\lambda,\kappa$ 的命名与上表相反；下文公式已换成上表记号，不改变其含义。

必须区分以下时间：

1. **Proposal-to-finality**：从有效区块开始提议到指定观察者决定。
2. **Proposal interval / view latency**：相邻区块或 view 的推进间隔。
3. **Transaction latency**：提交交易、等待打包、传播、执行、达成最终性及通知客户端的总时间。
4. **Fault recovery latency**：故障发现、timeout、收集切换证据、获取祖先 / payload 和恢复后的时间。

两轮提交不表示“每 $2\delta$ 才能出一块”；三轮最终性也不表示出块间隔必须是 $3\delta$。流水线和提前构建会影响后者。把“平均 block interval 的倒数”当成单笔交易 finality 会产生错误结论。

下文的 $2\delta/3\delta$ 均是忽略或另外计入本地处理的良好路径口径。它们不保证每次 GST 后出现诚实 leader 的任意 view 都从第一个节点进入时恰好两 / 三跳完成；进入时间差、证据到达和 timeout 仍属于活性分析的一部分。

### 1.2 四种容易混淆的保证

| 保证 | 需要验证什么 | 本综述如何处理 |
|---|---|---|
| **强最终性 / strong commit** | 决定有协议认可的证据；在配置的故障与密码学模型内，后续合法执行不能决定冲突值 | 快提交与慢提交均可提供 |
| **良好情况下的快速终止** | 诚实 leader、足够配合的参与者、通信 / 数据条件满足时多久决定 | 是时间保证，不能代替安全证明 |
| **Optimistic / speculative finality** | 某种额外的无故障、无 timeout 或无覆盖 skip 条件持续成立时，该块会保留 | 需要把额外条件与普通故障模型并列列出 |
| **数据 / 执行 / 结算保证** | payload 可恢复、状态转移有效、客户端可获得证明、L1 接受该结果 | 与 BFT 排序最终性分别确认 |

“optimistic”本身不代表弱安全：论文所称 optimistic two-round commit，可以是在良好条件下较快地产生**强提交证据**。Commonware 的 optimistic finality 则在文档中明确称为较弱的 speculative finality。这两个术语不能仅凭名称合并。[S08][S15]

### 1.3 基本假设和理论边界

经典部分同步允许 GST 前消息任意延迟，GST 后诚实节点间消息及时到达。安全性通常覆盖 GST 前的调度；终止性依赖后来存在足够长的良好通信时期。可靠交付、处理公平性、有效输入、可获得的祖先和委员会配置也需要满足。签名投票的状态在重启后不能被遗忘，否则可能签出论文模型不允许的冲突票。

若论文只证明固定委员会、静态腐化或某种 leader 选择方式，就不能自动推广到动态 stake、跨 epoch 换密钥、移动腐化和任意链外惩罚机制。本文记录的是作者的证明范围，不把部署名气视为额外定理。

FaB 使用 $5f+1$ 的经典构造开启两步快速共识；其关于精确最优节点数的早期结论后来被修正。2021 年工作在相应 authenticated partial-synchrony 模型下得到 $5f-1$ 的紧界，以及更一般的快路径故障预算结果。[S01][S03][S04]

因此：

- $n\ge3f+2p+1$ 是本文主要并发构造的一个简洁充分条件，**不是所有算法的精确下界**。
- Banyan 使用 $n\ge\max(3f+2p-1,3f+1)$ 的更紧参数；不能把它的阈值与简化帖子逐项替换。[S07]
- “两轮一定只能容忍低于 20% Byzantine”只是常用 $5f+1$ 配置的近似描述。$5f-1$ 的有限规模边界不同，例如 $f=1,n=4$。
- 在 $3f+1$ 个节点上提供“两轮良好路径、三轮一般路径”，并不等于在所有最多 $f$ 个故障的良好执行中均两轮终止：快路径可以要求更少实际故障。必须把 $p$ 写出来。

## 2. 研究路线与核心理论保证

### 2.1 路线关系

| 工作 | 主要研究问题 | 与本路线的关系 |
|---|---|---|
| FaB / Parameterized FaB，2005–2006 | 用更大 quorum / 更多节点实现快速共识 | 快路径与故障预算参数的历史起点 |
| SBFT，2019 | 用 collectors 和 threshold signatures 扩展复制规模 | 快慢路径及通信优化的工程前驱；其 replica commit 需要额外 collector 回传 |
| Simplex，2023；DispersedSimplex，2024 | 简化 view 推进及跨 view 安全；均衡分发大块 | 三轮强最终性与数据分发基线 |
| Banyan，2024 | 在 ICC 风格链上并发执行两轮快路径与三轮慢路径 | 更紧参数与公开网络研究原型 |
| Kudzu，2025 | 同时处理低延迟、均衡带宽、恶意分叉下的有限通信 | 与本次双路径帖子最直接对应的正式论文之一 |
| Alpenglow，2025 起 | 将相近投票思想用于 Solana PoS 和数据传播 | 与 Kudzu 同期、共享作者，非原协议的逐字实现 |
| Hydrangea，2025–2026；Hydrangea++ 公开草稿 | 加入独立 crash 预算，再优化提议间隔 | 更一般故障模型与流水线方向 |
| Minimmit，2025；ChonkyBFT，2025 | 保留一轮投票强提交，简化回退与切换 | 常用 $5f+1$ 配置下的另一种取舍 |

此表表示算法思想和问题的关系，不表示后一项目必然复用了前一项目的代码。Banyan 早于 Kudzu；Alpenglow 与 Kudzu 同期发展。

### 2.2 主协议比较表

表中的强最终性始终以该行列出的模型为条件。符号为统一展示；原始消息名称见来源。

| 协议 / 版本 | 理论安全与活性范围 | 强提交证据 | 良好路径 |
|---|---|---|---|
| **Simplex / DispersedSimplex** [S05][S06] | $f<n/3$，authenticated partial synchrony；终止依赖良好通信与诚实 leader | notarization 加后续 finalize quorum；规范版本的消息绑定方式不同 | 强最终性 $3\delta$，基本相邻提议间隔 $2\delta$ |
| **并发 2/3-round Simplex 风格帖子** [S08] | $n\ge3f+2p+1,\ 0\le p\le f$；作者目标为最多 $f$ Byzantine 下安全与活性 | $n-p$ 个 Vote 快提交；$n-f-p$ 个 Final 慢提交 | 至多 $p$ 故障的良好执行为两轮；一般至多 $f$ 故障为三轮 |
| **Banyan v3** [S07] | $n\ge\max(3f+2p-1,3f+1)$；最多 $f$ Byzantine 的部分同步模型 | rank-0 提议的 $n-p$ 个 fast votes；或 $\lceil(n+f+1)/2\rceil$ 个 slow finalization votes | 快路径两步；慢路径保持 ICC 型三步，不等待快路径 timeout 才启动 |
| **Kudzu v2** [S09] | $n\ge3f+2p+1$；最多 $f$ Byzantine；正式分析采用静态腐化 | complete block tree 中的块，加 FirstCert（$n-p$）或 FinalCert（$n-f-p$） | 最多 $p$ 不配合者时两轮；慢路径三轮 |
| **Alpenglow / Votor** [S10][S11] | stake 加权；Byzantine stake 严格低于 20% 的安全分析；额外 offline 容忍的活性条件见 2.5 | ≥80% notarize stake 的 fast-finalization；或约 60% 的 notarization 与 finalization 证书 | 在区块分发后，投票时间表达为 $\min(\delta_{80\%},2\delta_{60\%})$ |
| **Hydrangea** [S12] | $n=3f+2c+k+1$，最多 $f$ Byzantine 与额外 $c$ crash；$k$ 有范围限制 | 参考实现：$n-p$ 个普通 vote 快提交；$2f+c+1$ 个 commit 慢提交；前置区块证书为 $\lceil(n+f+1)/2\rceil$ | 摘要和参考代码采用 $p=\lfloor(c+k)/2\rfloor$：至多 $p$ 总故障时两轮，完整模型良好执行三轮 |
| **Hydrangea++ 当前公开草稿** [S13] | 延续混合故障模型，加入 optimistic / normal / fallback 不同票型；存在参数口径待澄清项 | 同票型 $n-p$ 个 vote 快提交，另有慢 commit 路径 | 声称保持两 / 三轮提交，并把连续诚实提议间隔降到 $\delta$ |
| **Minimmit v5** [S14] | $n\ge5f+1$，最多 $f$ Byzantine，partial synchrony；三轮慢提交不再单独运行 | $n-f$ 个普通 notarize votes，即 L-notarization | 诚实 leader 的良好执行两轮；$2f+1$ 的 M-notarization 可较早推进 view |
| **ChonkyBFT v1** [S16] | $n\ge5f+1$，authenticated partial synchrony | $n-f$ 个 CommitVote 的 CommitQC；超时报告保护可能隐藏的 CommitQC | 一次提议加一轮投票；view-change / 重提议并非另一条三轮提交路径 |

**历史对照：SBFT。** 其 $n=3f+2c+1$ 的参数与后来的记号相似，但 common mode 中的 $c$ 指 crash / slow 节点。副本从 pre-prepare、sign-share 到收到 collector 的 full-commit-proof 需要三次传播；只说“有 fast path”不能把它计为本文口径的两轮 replica finality。SBFT 同时讨论客户端执行回执，不能与副本决定时间混算。[S02]

### 2.3 双路径为何能安全组合

以下是对简化并发构造的计数展开，不是对所有协议的统一证明。[S08]

设快提交 quorum 为 $Q_F=n-p$。其中至少 $n-p-f$ 个是诚实投票者。后来收集 $n-f$ 个不同节点投票的观察者最多漏掉 $f$ 个节点，因此仍能看到至少

$$
C_F=n-p-2f
$$

个诚实支持者。任何冲突普通值至多由其他 $f+p$ 个节点支持。要求

$$
C_F>f+p
\quad\Longleftrightarrow\quad
n\ge3f+2p+1
$$

就得到这套简化快证书的门槛。

慢 quorum $Q_S=n-f-p$ 满足

$$
2Q_S-n=n-2f-2p\ge f+1,
$$

因而任意两个慢 quorum 至少相交一个诚实节点。这个交集只能在明确禁止相应冲突签名时用于安全证明。

合并还需要两个机制：

1. **同一 view 内按 Fast < Slow 排序。** 慢提交 $x$ 后，较小的冲突快证书仍可能存在。后续若引用该快证书，必须证明同 view 的慢层级可被跳过，而已经慢提交恰好排除了该证明。
2. **完成 view 时兼顾两层证据。** Fast-bottom 只排除快提交；Slow-bottom 只排除慢提交。只拿到其中一个，不能认为该 view 的所有提交可能性均已被排除。

普通值证书与提交证据也必须区分：Slow-Cert 由 Vote 形成，慢提交由 Final 形成，即使两者票数相同。

补投 bottom 的规则另需特别处理。诚实节点可在见到充分“无法快提交”的证据后先投普通值再补投 bottom，不能在证明中笼统地使用“诚实节点每 view 只投一票”。正确的排除论证要考察某个可能快提交值的诚实投票者中**第一个补投者**：在它补投前，它收集的 quorum 已包含 $C_F$ 个该值支持者，从而与“尚无快证书”的触发条件矛盾。

补投的活性作用是解除 split vote：恶意 leader 可能让诚实节点投向不同值，各节点收到了 $n-f$ 票却没有任何值证书；补投 bottom 才能形成跳过该层级的证据，使系统继续推进。已经看到某个值证书时不满足这条触发条件。[S08][S25]

这些计数尚不替代跨 view 的完整归纳、数据有效性和 timeout 时序分析。帖子是解释性证明草图，不能直接当成已经检查完整的生产状态机规格。

对应最初纯两轮帖子，令 $p=f$，则快提交为 $n-f$，残留证书为 $n-3f$。这个第三个 $f$ 来自**另一个观察者收票时可能遗漏的节点**：提交票中的诚实支持至少有 $n-2f$，观察者的 $n-f$ 人集合还可能漏掉其中 $f$ 人。两次扣减分别处理 Byzantine 票和观察者未收到的票，不能合并为一次。[S25]

### 2.4 Kudzu：强最终性与数据完成条件

Kudzu 的 FirstCert 证明 $n-p$ 个 first votes，FinalCert 证明 $n-f-p$ 个 final votes。其本地显式 finalization 还要求区块进入 complete block tree：包括祖先、notarization、从至少 $f+p+1$ 个认证分片恢复 payload，并通过有效性检查。[S09]（§4.3–4.4）

这使“证书可证明不会出现冲突决定”与“本地已拥有完整可执行数据”衔接起来。仅实现 quorum 计数、忽略祖先和数据恢复，并不是完整实现该协议。

其较特殊的投票逻辑允许在第一票之后重新审视某些候选，处理恶意 leader 的分叉；论文证明每 slot 的相关候选与诚实投票次数有常数界。这也是它相对原始 Banyan / ICC 风格多候选设计的重要区别。其 adaptive corruption 安全性在文中是附条件的推广预期，不应取代已明确使用的静态腐化证明设置。

### 2.5 Alpenglow：把安全性与额外 crash 活性分开

Alpenglow 作者把它与 Kudzu 描述为密切相关的设计：前者面向 PoS、leader windows 与 Solana 数据传播，后者提供较简洁的 permissioned 模型。[S09][S10]

当前 Anza 的条件说明称：Byzantine stake 低于 20% 时，即使完全异步和任意多 offline stake，仍有安全保证；这允许系统停顿，并不保证此时继续出块。额外 crash 容忍的活性依赖更强的分发 / non-equivocation 条件；在 equivocating leader 下，该说明仅承诺恶意加 offline 总 stake 低于 20% 时的相应活性。[S11]

Hydrangea 论文 §7 对“经典部分同步下无条件 20+20”的表述提出了质疑，而 Anza 明确区分了安全性与扩展活性。[S12] 本文不据此宣布已证明生产实现存在安全漏洞，也不把营销摘要当作消除了这些条件。**选型时必须采用对应版本的完整假设和 theorem，分别记录安全故障上限、活性故障上限及传播层假设。**

Votor 的 $\min(\delta_{80\%},2\delta_{60\%})$ 是区块分发后的投票时间。把它直接当成从交易提交开始的总延迟，或省略分片恢复和执行开销，都会低估实际成本。

### 2.6 Hydrangea / ++：版本与参数不一致

当前 USENIX PDF 的摘要、Theorem 1、技术概述及 Claim 6 使用

$$
p_{\rm conservative}=\left\lfloor\frac{c+k}{2}\right\rfloor.
$$

但 §3 又出现 $\lfloor(c+k+2)/2\rfloor$。本文已核对该 PDF 的页面图像，差异存在于排版正文，并非单纯文本提取错误。Hydrangea++ 当前公开 PDF 也有类似的摘要 / §2 定义差异。[S12][S13]

参考实现的配置代码采用前一种较保守定义，并设置：

$$
Q_{\rm block}=\left\lceil\frac{n+f+1}{2}\right\rceil,\quad
Q_{\rm fast}=n-p,\quad
Q_{\rm slow}=2f+c+1,\quad
Q_{\rm viewchange}=n-f-c.
$$

代码依据见 [Hydrangea 配置][S19]。本文比较表采用摘要与参考代码一致的保守口径；不把“+2”的另一种写法当作已额外获得一个快路径故障预算的结论。复用证明或参数前，应向作者确认正确版本。

Theorem 1 还要求：$c$ 为偶数时 $0\le k\le2f+c-4$，$c$ 为奇数时 $0\le k\le2f+c-3$。所以不能把所有满足节点数等式的 $f,c,k$ 都直接视为其 theorem 覆盖范围。Hydrangea++ 对应地使用参数 $m$。

### 2.7 Commonware Simplex 的两跳到底保证什么

当前文档的 ordinary full finalization 是：

~~~
提议及 notarize 传播 → 收齐 notarize、应用认证 → finalize 传播
         第 1 跳                 第 2 跳               第 3 跳
~~~

两跳处的 notarization 可以提供条件性的 speculative finality。条件涉及该 term 的起点到当前 view 均无 nullify trigger；单独看到 notarization 并不能证明这个条件已经永久满足。文档甚至明确说明，Byzantine leader 可以通过自己的 nullify 触发诚实方 timeout，排除自己已经 notarized 的提议。[S15]（Optimistic Finality）

“Commonware 有 $2\delta$ fast finality”若指文档的 optimistic finality，确有对应能力；若指与 Kudzu FirstCert 一样的强两轮提交证据，则不是同一保证。

此外，stable leaders、optimistic validation 和 pipelined handoff 可以提前构建和投票；相应条件下 view interval 可从两跳变为一跳。这优化的是流水线，不是删除 full-finalization 所需的最后一轮票。比较时应分别填写 full finality、speculative finality 和 proposal interval 三列。

### 2.8 活性 theorem 与证明证据应如何阅读

这些协议的部分同步活性首先是“网络后来稳定、timeout 足够长、领导者选择让诚实提议获得机会时，系统不会永久停住”。它不等于 GST 后每个 view 都成功，也不等于任意具体交易都有已证明的抗审查时限。

- **Simplex：** 原始版本使用共同随机 leader 的设置。其终止 / 吞吐分析带有该随机选择假设；替换为工程中的 scheduled / stable leader 后，需核对相应活性条件。[S05]
- **Kudzu：** Liveness I 分析及时进入的诚实 slot；Liveness II 证明坏 slot 最终可退出。后者含 timeout 项，不能用前者的两轮时间替换。静态腐化是正式分析的范围之一。[S09]（Lemma 5.9–5.10）
- **Minimmit：** 分别证明 view progression、诚实 leader 的提交和 optimistic responsiveness。M-quorum 的作用是推进；即使看到了约 40% 的票，仍需最终获得 L-quorum 才能强提交。[S14]（§5）
- **ChonkyBFT：** 活性明确假定周期性重传最新投票，并有额外 block-sync 服务在 $\Delta_{\rm sync}$ 内补齐区块；充分长 timeout 取决于该时间。只实现 commit votes 而不提供 block sync，不能直接继承完整活性结论。[S16]（§4.2）
- **Hydrangea / ++：** 独立 crash 预算参与慢路径的可达 quorum；++ 的 Lemma 7 从首个节点进入 view 计时给出含 $\Delta$ 的界，与同步进入时的良好两轮口径不同。[S12][S13]

论文中的归纳证明、有限状态 model checking 和完整机器检查证明也应分开。ChonkyBFT 提供 Quint 规格、模拟与 bounded model checking，加上论文的安全性论证；本文没有把它重述成任意节点数的全状态机器检查证明。[S16]（§5） 两篇 Simplex 风格帖子则主要是解释性规格与 proof sketch，证据层级不同。

## 3. 理论通信复杂度

### 3.1 先说明计量单位

本节把四个不同对象分开：

- **Multicast complexity**：一次发送给全体可能计为一个 multicast。
- **Point-to-point message complexity**：一次全体 multicast 展开后约为 $n$ 次发送。
- **Bit complexity**：计入 payload、签名、signer bitmap、Merkle path 和证书列表的总 bit 数。
- **Critical-node egress**：最忙的 leader / collector 发出多少字节；总量相同也可能有不同瓶颈。

例如，$n$ 个节点各广播一条小票是 $O(n)$ multicasts、$O(n^2)$ point-to-point messages。若每条包含 $O(n)$ 个签名，字节量可达 $O(n^3\lambda)$。所以“二次消息复杂度”不能直接写成“二次字节复杂度”。

下表中 **[P]** 表示来源给出的边界，**[D]** 表示在注明条件下由消息结构直接展开的分析。通常计入的是诚实节点按协议发送的消息；攻击者自行注入任意多无效字节不可能由纯协议渐近式约束。

### 3.2 可比较的理论边界

| 协议 / 数据模式 | 控制消息与通信量 | Payload / leader 负担 | 范围与依据 |
|---|---|---|---|
| **Simplex 基本投票结构** | [D] 固定数量的全体投票 / 转发阶段展开为 $O(n^2)$ 小消息；原论文的 multicast 口径需另看 | 原始全链提议、缓存与恢复策略影响字节量，不能仅由轮数给统一历史无关界 | 稳态控制结构；不要把每次转发整条历史忽略 [S05] |
| **DispersedSimplex** | [P] $3n\beta+O(n^2(\lambda+\kappa\log n))$ bit / slot；本表已统一原文符号 | 大块时 payload 项占主导；leader 和其他节点负担相差常数，可进一步平衡 | 论文 §3.4；含分片、Merkle paths、签名份额 / 证书 [S06] |
| **并发 2/3-round 教学协议** | [D] 固定全体 Vote / Final 阶段为 $O(n^2)$ 次发送；完整 bit 界还取决于证书压缩与历史证明 | 未单独构造与验证完整的数据分发层 | 教学伪代码不等于完整复杂度 theorem [S08] |
| **Banyan** | 良好单候选执行可展开为 $O(n^2)$ 控制发送；原始设计在恶意 equivocation 下可能出现每 slot 无界通信 | 公开实验使用完整区块传播，未据此获得 Kudzu 式均衡大块界 | 无界问题是后续 Kudzu 的明确比较结论，不能无条件填写“最坏 $O(n^2)$” [S07][S09] |
| **Kudzu** | [P] $O(n^2)$ messages；$O(n\beta+n^2\kappa\log n+n^2\lambda)$ bit / slot | 各节点含 leader 的发送量在常数因子内均衡；单节点存储 $O(\beta+n\kappa\log n+n\lambda)$ | §5.3–5.4 的有限候选 / 投票次数证明；参数还用到 $n<3(f+p+1)$ 的复杂度规范化条件 [S09] |
| **Minimmit** | [P] §6.4 在有界区块与阈值签名优化下为 $O(n^2)$ 通信 / view；裸多签列表转发更大 | [D] leader 发完整块、其他节点仅发 digest 时，payload 至少有 $O(n\beta)$ 项；若每轮票重复携带完整块则可有 $O(n^2\beta)$ 项 | 不能把“区块常数有界”的分析直接当作任意 $\beta$ 的 bit 界 [S14] |
| **Hydrangea** | [P] 使用 threshold signatures 时 $O(n^2)$ communication / view；TC 含 $O(n)$ 个报告，leader 广播 TC 本身已有二次成本 | 全块提议还应计 $O(n\beta)$ 数据量；论文实验主要排序 PoA 占位数据 | §5.2；具体 multi-signature / bitmap 编码需重新计 bit 数 [S12] |
| **Hydrangea++** | [D] 保留的全体投票结构及常数种票型不消除二次控制发送；提前多 view 的缓存 / 恢复需额外计入 | 优化 proposal interval，并不自动给出更低的 payload 渐近界 | 当前草稿未提供足够完整的统一 bit / 恢复成本 theorem，本文不补造 [S13] |
| **ChonkyBFT** | [P] quadratic message 设计；QC / TimeoutQC 的 signer 列表和嵌套报告必须计入具体大小 | leader 给每个副本发完整 proposal，负担随 $n\beta$ 增长；作者实验确认它是瓶颈 | 不能只凭“quadratic”摘要推断所有大块和 view-change 字节量均二次 [S16] |
| **Alpenglow / Votor + Rotor** | [D] 直接全体广播、固定票型时控制发送为二次级；stake、证书 signer 信息、repair 与重传另计 | Rotor 目标是按 stake 利用各节点带宽并消除 leader 瓶颈；不在此赋予与 Kudzu 完全相同的 bit 公式 | 作者理论模型与当前 Solana 部署的数据传播阶段需区分 [S10][S17] |
| **SBFT** | [P] collector 模式控制量 $O(cn)$；只有 $c=O(1)$ 时才能简称 $O(n)$；view-change 报告另计 | leader 仍发区块；collector 聚合 / 回传增加串行阶段 | 原文 §V，固定 small $c$ 是“linear”解释的一部分 [S02] |

### 3.3 签名聚合解决什么

Threshold signature 可以把相同消息的 quorum 签名压缩为常数大小证书，但需要 DKG / resharing、key epoch 和正确的签名份额管理。支持聚合的普通 BLS multisignature 仍可能携带 signer bitmap；不同签名者分别报告不同 high-vote 的 TC，也不能仅靠一条同消息阈值签名表达其全部语义。

一个便于预算的推导：若所有 $n$ 个节点都给其余节点转发一个含 $q=\Theta(n)$ 个独立签名的证书，发送数为 $n(n-1)$，仅签名部分即有 $\Theta(n^3\lambda)$ bit。换成常数证书后，这一项才降为 $\Theta(n^2\lambda)$。这是条件化的消息展开，不是断言所有论文都使用该低效转发方式。

Minimmit 还讨论聚合连续 nullification；它可以减少证据传输大小，但不表示验证大量不同 view 的历史必然是常数计算。动态委员会或不同 threshold keys 也需要单独处理。[S14]（§6.5）

### 3.4 不能从每 view 上界直接得到系统总开销

1. 恢复时可能需要补齐 $H$ 个历史 view 的 skip / ancestry 证据。
2. packet loss 下重复广播次数取决于网络和重试策略。
3. 向落后节点提供 payload / checkpoint 的流量可能超过当期控制票。
4. DKG、resharing、membership changes、证书外部验证和日志落盘常被稳态模型摊销或排除。
5. 大块编码后的总 payload 量为 $O(n\beta)$，仍有 $n^2$ 个携带 Merkle / 签名的发送；“均衡带宽”与“线性消息”不是同一个结论。

因此，实际比较应同时报告：每 finalized byte 的总网络字节、leader / collector 峰值 egress、每节点验签 / 编解码 CPU，以及恶意 / 恢复执行中的证据增长。

## 4. 实验结果与证据强度

### 4.1 哪些结果可以直接放在一起

只有同一实验框架、相同负载、相同观测事件和明确的故障预算，才适合计算性能差异。跨论文的毫秒数仅能说明各自环境中的表现。

| 工作 | 证据类型 | 可以支持什么 | 不能直接支持什么 |
|---|---|---|---|
| Banyan | Go 原型在 AWS WAN 上运行 | 指定规模与地理分布下，较大 fast quorum 的收益和代价 | 完整生产栈的 P99、恶意 equivocation 下的资源上界 |
| Minimmit 中的三协议比较 | 确定性网络模拟，使用实测地区延迟参数 | 相同网络模型下阈值、出块间隔、编码参数的影响 | 三套真实引擎的 CPU、磁盘和运行稳定性排名 |
| Hydrangea | Rust 原型在 GCP WAN 上运行 | PoA 排序层的延迟和吞吐量，及 withholding 时的慢路径表现 | 包含真实交易分发、验证、执行的端到端 TPS |
| ChonkyBFT | Rust 实现在全球 GCP 节点运行 | 无故障运行的吞吐、CPU 和传播瓶颈 | 故障恢复的 P99 或一般生产 SLO |
| DispersedSimplex / Kudzu 理论部分 | 成本公式与模型分析 | 数据通信渐近量与 leader 负担 | 当前硬件上的实际吞吐量 |
| Alpenglow | 原作者模拟、参考程序，以及后来的集成 / 部署资料 | 理论路径、模拟目标、部署状态分别可核对 | 用目标 150 ms 代替当前主网实测分布 |
| Hydrangea++ 当前 PDF | 摘要含性能声明，但缺少对应实验展开 | 记录作者声明及待验证事项 | 独立评估“11% / 2×”的可复现性 |

### 4.2 Banyan：真实 WAN 中的 fast-quorum 代价

论文使用 19 个 AWS t3.large 节点，每次运行 120 秒；实现基于 Bamboo，并在实验中以 round-robin leader 替代随机 beacon。下面摘录其同一组实验中的平均区块最终性；单位 ms。[S07]（§8）[S18]

| 部署 / 负载 | ICC | Banyan $f=6,p=1$ | Banyan $f=4,p=4$ |
|---|---:|---:|---:|
| 4 个全球数据中心，400 KB | 239 | 216 | 179 |
| 19 个全球地区，1 MB | 384 | 362 | 324 |

第一种 Banyan 配置保留 $f=6$ 的安全预算，fast quorum 为 18/19；第二种把 Byzantine 安全预算降为 4，fast quorum 为 15/19。因而 179 ms 的结果包含**不同安全预算**的影响，不能说在同一故障保证下所有配置都得到约 25% 的改善。

论文还测试了 crash 情形，并在该设置下观察到退回 ICC 行为没有额外惩罚。它没有由此测出 Byzantine leader 任意分叉、隐藏证书和持续 equivocation 时的完整资源成本。

### 4.3 Minimmit：同一模拟中的 Simplex / Kudzu / Minimmit

这是当前资料中较适合观察机制差别的一组对照。50 个模拟节点均匀分布于 10 个虚拟 AWS 地区；延迟均值使用地区实测 p50，jitter 标准差使用 p90−p50；每节点对称 1 Gbps，带宽分配采用 max-min fairness。区块生成被视为瞬时，性能执行不注入故障。[S14]（§7）

**32 KB 完整块、digest 投票、不使用 erasure coding：**

| 协议 | View latency，ms | Block finality，ms | 论文定义的 transaction latency，ms |
|---|---:|---:|---:|
| Simplex | $194.61\pm30.34$ | $299.34\pm25.98$ | $493.95\pm7.50$ |
| Kudzu | $189.94\pm29.32$ | $220.31\pm28.58$ | $410.25\pm7.61$ |
| Minimmit | $146.07\pm21.33$ | $220.30\pm28.57$ | $366.37\pm7.06$ |

这里 ± 是论文报告的标准差，**不是置信区间或 P99**。Minimmit 相对 Kudzu 的 block finality 几乎相同，收益主要来自较早推进 view。该文 transaction latency 按等待一个 view 加后续区块最终性计算；它不是任意客户端到达过程、排队和执行共同测出的平均交易体验。

**1 MB 区块、开启编码、仍为 1 Gbps：**

| 协议 | View latency，ms | Block finality，ms | Transaction latency，ms |
|---|---:|---:|---:|
| Simplex | $230.62\pm30.33$ | $335.36\pm25.97$ | $565.98\pm7.50$ |
| Kudzu | $220.94\pm29.33$ | $251.29\pm28.63$ | $472.23\pm7.61$ |
| Minimmit | $216.00\pm21.46$ | $290.32\pm28.52$ | $506.28\pm7.07$ |

此时 Kudzu 的 transaction latency 更低。模拟的 Minimmit 编码从 $f+1$ 个分片恢复，Kudzu 在该配置从 $2f+1$ 个分片恢复，因此后者每片更小。论文把改善 Minimmit 编码参数留作后续工作。提高带宽到 10 Gbps 后，论文又观察到 Minimmit 的优势，说明结论受数据分发参数影响。

比较还有两项限制：Simplex 与另外两个协议采用不同的 Byzantine 容忍比例；公开 estimator 的协议脚本主要描述性能路径，不能替代完整 Byzantine 状态机或真实 RS 编解码、验签、WAL 开销。Kudzu 脚本的存在证明“有可运行性能模型”，不证明“已发布完整 Kudzu 库”。固定代码快照见 [S23]。

### 4.4 Hydrangea：有真实网络实验，但负载是排序层

论文在 10 个 GCP 地区部署 10 / 50 / 100 个节点，使用 e2-standard-4、4 vCPU、16 GB 内存，运行 180 秒。它移除了 Narwhal mempool 和模拟客户端，由 leader 直接产生每项 190 B 的 PoA 占位数据；每块最多 800 项，约 153 KB。延迟定义为区块生成到 $2f+c+1$ 个节点提交的平均时间，吞吐量为 committed bytes/s。[S12]（§6）

作者报告：100 节点设置中，相对 Jolteon 延迟约降 20%，相对 ByzCrash 约降 10%，吞吐量有较小改善。**与 ByzCrash 的对照更适合隔离 fast path 的贡献**；与 Jolteon 的差异还包含故障模型和 quorum 的变化。

还有一个参数核对点：论文 10 节点实验写为 $f=1,c=2,k=2$，超过其 Theorem 1 所列的 $k\le2f+c-4=0$ 范围。该实测点不能直接被引用为该 theorem 覆盖配置的性能证据；是否有更一般的正确性论证适用于它，需作者澄清。本文保留原实验描述，不据此宣布协议不安全。

50 节点故障实验中的 9 个故障节点不发 vote，关闭了快路径；Hydrangea 与 ByzCrash 的性能趋于一致。这支持并发慢路径可接管该 withholding 场景，不构成任意 Byzantine 攻击的实验覆盖。

该结果应被理解为“对可用数据证明进行排序”的成本，不能直接套用到每个节点重新下载、验证、执行全部交易的链。

### 4.5 ChonkyBFT：真实引擎结果与 leader 瓶颈

论文在全球 GCP 网络使用 e2-highcpu-8、1 Gbps 链路，每次运行约 1–1.5 小时，报告无故障结果。摘录其表中数据：[S16]（§7）

| 节点数 | 区块大小 | 典型 RTT | Blocks/s | 单节点平均 CPU，核 |
|---|---:|---:|---:|---:|
| 6 | 1 MB | 120 ms | 3.7 | 0.5 |
| 25 | 1 MB | 130 ms | 3.3 | 1.0 |
| 100 | 1 MB | 140 ms | 2.1 | 2.2 |
| 100 | 0.1 MB | 140 ms | 3.0 | 2.2 |

论文指出完整 proposal 从 leader 发给全体会成为瓶颈，TCP 空闲后的 slow start 也影响结果。表中的 blocks/s 不能倒数换算为交易 finality；论文没有提供与上表一一对应的 P95 / P99 最终性。落后节点的 block sync 也不在协议核心分析范围内。

### 4.6 其他实验声明如何使用

- **Alpenglow：** 2025 年原作者的约 150 ms 估计来自模拟，并把执行计算等另行处理。2026 年官方页面继续使用约 150 ms 的目标，同时确认 Votor 已上线；这两类证据不能合并为本文自行验证的主网 P99。Rotor 尚未随 Votor 一同部署。[S17][S24][S28]
- **Hydrangea++：** 当前取得的 12 页 PDF 摘要声称最高 11% 更低延迟、近 2× 吞吐量；但该版本没有给出对应实验章节、完整测试设置与结果表。应把它记为作者摘要声明，暂不据此排名。[S13]
- **SBFT：** 209 副本的研究实验及相对优化 PBFT 的改善具有历史参考价值，但硬件、客户端回执、batching 与现代协议不同，不能横向拼接成统一 TPS 榜单。[S02]

**现有实证最有把握的结论是：quorum 大小、提议间隔和数据分发方式共同决定延迟。** 这些实验尚不足以证明某个协议在所有委员会规模、故障率和负载下同时胜过其他协议。

## 5. 公开实现及后续发展

### 5.1 截止调研日可确认的状态

| 项目 | 可确认的公开产物 | 采用时的含义 |
|---|---|---|
| **Commonware Simplex** | 已发布 Rust crate；本次读取文档版本 2026.10.0；有持久化、修复、认证及应用接口 [S15][S27] | 适合作为工程比较基线；文档的普通强最终性为三跳，不能重标为两跳 |
| **Banyan** | 作者公开 Go 研究原型，基于 Bamboo [S18] | 可用于复现实验与研究；需要另做 adversarial resource bounds 和生产工程 |
| **Kudzu** | DISC 2025 论文、后续版本；Commonware estimator 中有公开性能脚本 [S09][S23] | 尚未核实原作者独立完整网络库、稳定 API、生产部署；“未查到”不等于证明不存在 |
| **Alpenglow** | 作者参考仓库；Agave 中有 Votor 实现；Solana 官方确认 2026-10-21、epoch 1064 主网激活 [S17][S24][S29] | 有真实生产集成，主要面向 Solana；Rotor 仍待后续上线，不能称全套新数据层均已部署 |
| **Hydrangea** | 作者 Rust reference implementation，README 明确称研究用途、非生产就绪 [S20] | 是混合故障模型研究的直接起点，参数版本须先统一 |
| **Hydrangea++** | 当前公开论文草稿 [S13] | 本次未确认对应完整实现与可复现实验包 |
| **Minimmit** | 论文模拟器；Commonware 实验分支和 PR #2884 [S22][S23] | 不能把实验分支当成已发布稳定模块；本次核对的 main 模块导出仍是 Simplex [S27] |
| **ChonkyBFT** | Matter Labs 的 era-consensus Rust 实现；仓库于 2026-06-23 归档 [S21] | 可研究 / fork，但需要自行承担后续维护；原 L2 的共识决定还应与 L1 结算区分 |

Minimmit PR 的页面缓存仍曾显示 Open；本次直接页面核对显示其于 2026-07-15 关闭，维护者表示以后再考虑。以实时状态为准，不能把旧搜索缓存视为已合并或当前排期。PR 中的初步实网测试也不能与论文模拟表格混为一组数据。[S22]

### 5.2 Kudzu 提出后，作者与社区做了什么

可从公开证据重建的路径是：

1. **正式化和发表。** 2025 年 5 月初稿，9 月 v2；作者记录其获 DISC 2025 接收。[S09]
2. **与 Alpenglow 同期推进。** 二者共享研究人员和双路径思想；Alpenglow 加入 PoS、数据分发和 Solana 集成。它是相近路线的工程发展，不等于一个通用 Kudzu package。[S10][S24]
3. **其他作者扩展模型。** Hydrangea 加入独立 crash 容忍；Minimmit 改变 view 推进与提交的关系。这些是研究后续，而非 Kudzu 作者已实现全部变体的证据。[S12][S14]
4. **公开可跑性能模型。** Commonware estimator 明确比较 Kudzu 等投票阈值；它可帮助重做通信时间实验，但没有覆盖完整恢复和恶意执行。[S23]

Kudzu 文中提到更紧的节点参数及 stable-leader 方向。本文未把这些“后续工作”当作已经发布、已有完整证明和工程实现的稳定版本。

### 5.3 论文状态机、benchmark 配置与生产实现的边界

Hydrangea 配置代码的 benchmark 辅助函数按 is_honest 属性筛选部分测试参与者；这类受控故障注入元数据不应进入生产 peer selection。它本身不是协议安全漏洞的证据，却说明“能启动参考实验”与“完成面向未知恶意节点的产品集成”之间还有工作。[S19]

Commonware 的应用 certification hook 默认成功，也不替应用证明 payload 的外部可用性或执行有效性。接入者需要提供与实际数据层匹配的认证条件。[S15]

本次实现检查止于公开文档、关键配置 / 提交流程和仓库状态，未覆盖完整执行路径、持久化与 crash-recovery 审计。

## 6. 按多个场景给出选型建议

以下是基于前述证据的判断，不是对未复现实验的性能承诺。

### 6.1 先固定不能交换的条件

1. **安全预算：** 必须承受接近 $n/3$ 的 Byzantine，还是允许降至约 $n/5$？是否另需容忍独立 crash？
2. **业务最终性：** 客户端能否使用有撤销条件的确认？跨系统转账和不可逆外部动作应等待对应模型中的强证据。
3. **主要瓶颈：** 小块 WAN 往返、大块带宽、执行 CPU，还是故障切换尾延迟？
4. **工程投入：** 使用现成库，还是承担协议实现、恢复、持久化、membership 和审计？

### 6.2 场景建议

| 场景 | 优先评估 | 原因与采用条件 |
|---|---|---|
| 通用固定 / 分 epoch 委员会，要求接近 1/3 Byzantine，尽快获得可集成实现 | **Commonware Simplex** 作为基线 | 普通强最终性三跳，接口和恢复工程较完整；按应用接好 certification、数据与持久化 |
| 同样要求接近 1/3 Byzantine，但强两轮必须在无故障 / 极少故障时可用 | **Banyan 式小 $p$**；或 **Kudzu 小 $p$** 自研 | Banyan 紧参数可在某些 $3f+1$ 配置取 $p=1$；简化 Kudzu 同规模通常只可取 $p=0$，需等待全部票。必须评估尾节点拖延 |
| 可以接受约 1/5 Byzantine，希望减少分支并让普通良好 view 两轮强提交 | **Minimmit / ChonkyBFT** | 算法路径较直接；Minimmit 当前分支状态与 Chonky 仓库归档意味着集成团队需承担维护，不能当作现成稳定模块 |
| 大块、高带宽利用率、希望快慢路径自动竞争 | **Kudzu**，配合 **DispersedSimplex** 对照 | 数据均衡和有限投票分支有明确理论价值；主要成本是补齐完整实现和实测，而非再写一份 quorum 脚本 |
| 业务确实存在“恶意节点 + 额外非恶意宕机”两个预算 | **Hydrangea** 研究实现 | 模型直接表达该需求；先解决 $p$ 的版本不一致，再验证恢复和混合故障执行。++ 暂作为流水线研究备选 |
| 已在 Solana 体系内，或愿意深度采用其 PoS / 客户端栈 | **Votor / Alpenglow** | 有生产集成证据；采用对应传播层与 stake 假设，单独核对 Rotor 尚未部署的边界 |
| 大委员会，控制消息带宽比少一跳更重要 | 同时比较 **collector 架构**与**分片分发** | SBFT 在固定少量 collectors 下减少控制发送，但增加聚合回程和 collector 风险；分片改善数据量，不自动把全体投票变线性 |
| 教学、证明与参数边界研究 | **双路径帖子 → Banyan / Kudzu → Hydrangea / Minimmit** | 先掌握隐藏提交与跨 view 证据，再研究更紧参数、数据层和流水线；不直接把教学代码视为生产协议 |

对“已有成熟通用库，且默认提供强两轮、接近 1/3 Byzantine、无需额外 fast-case 假设”的组合，本文没有找到可支持这种推荐的证据。故障预算与两轮 good-case 的组合需要满足第 1.3 节的理论约束。

### 6.3 固定节点数，看看实际在等待多少票

以下仅为**阈值算例**；不代表实测，也不表示各行安全预算相同。所有行固定 $n=100$：

| 配置 | Byzantine 安全预算 | 两轮强提交所需普通票 | 慢提交票 / 其他说明 |
|---|---:|---:|---|
| Simplex / Commonware 普通配置 | $f=33$ | 无此强路径；67 票的 notarization 仍需后续 finalize | 再收 67 个 finalize，普通强提交三轮 |
| 简化 Kudzu 参数 $f=33,p=0$ | 33 | 100 | 慢 final quorum 67 |
| Banyan 参数 $f=33,p=1$ | 33 | 99 个 rank-0 fast votes | 慢 final quorum 67 |
| Kudzu 参数 $f=25,p=12$ | 25 | 88 | 慢 final quorum 63 |
| Minimmit 参数 $f=19$ | 19 | 81 | 39 票的 M-notarization 仅供提前推进，不是提交 |

这解释了“多收一次票”有时反而更快：第 99 个回复若远迟于第 67 个回复，等待两轮 67 票可能先于一轮 99 票完成。反过来，网络均匀、额外回复几乎免费时，两轮更占优。

固定故障数则有另一种代价。例如 $f=3$，Simplex 可用 $n=10$；简化 Kudzu 的 $p=1$ 需 $n=12$，$p=3$ 需 $n=16$。一次全体投票展开分别有 90、132、240 次 point-to-point 发送。该计算只计发送数量，未计 payload、回退阶段和密码学；它说明比较“相同安全故障数”时，增加节点不是免费的。

### 6.4 一个更实用的延迟比较方法

设 $P$ 为提议和必要数据到达的时间，$V(q)$ 为相关 quorum 的普通票到达耗时，$F(q)$ 为 final votes 到达耗时。简化双路径成本可写成：

$$
T_{\rm commit}\approx P+
\min\{V(n-p),\ V(n-f-p)+F(n-f-p)\}.
$$

这是性能预算表达式，不是独立安全定理；真实网络中各节点收票时间不同，阶段可重叠，数据恢复 / 本地验证也可能处于关键路径。

选型时应把实测 quorum 到达分布代入，而不是直接令所有 $V,F$ 都等于同一个 $\delta$。同时用交易到达和 batching 模型计算等待出块时间；否则会漏掉 Minimmit 和 Hydrangea++ 所针对的 proposal interval 收益。

## 7. 如果要做下一轮可复现评估

现有资料足以筛掉不匹配的模型，还不足以替代本地工作负载实验。建议按以下顺序建立共同 benchmark：

1. **分两组公平性。** 一组固定 $n$，明确允许的 $f,c,p$；另一组固定所需故障预算，允许 $n$ 变化。不要用不同安全预算得到的一个数字宣布全局胜负。
2. **同一数据与密码学配置。** 统一 block size、batching、编码恢复门槛、签名 / bitmap、真实 RS 与验签、CPU 和网络上限。分别测仅排序 PoA 和完整交易数据。
3. **分别计时。** 记录 view interval、有效 proposal 到各节点决定、客户端提交到可验证强证据的 p50 / p95 / p99；speculative confirmation 单列。
4. **量化资源。** 报告总 bytes / finalized byte、最忙节点 egress、验签 / 编码 CPU、WAL 和落后节点追赶开销。吞吐应在满足尾延迟约束下测，而非仅报最大平均值。
5. **故障执行。** 覆盖少量慢节点、恰好跨过 fast budget、Byzantine leader equivocation、只向部分节点隐藏 QC、部分发送后宕机、timeout 与晚到证书竞争，以及 GST 前分区后恢复。
6. **生命周期。** 测试重启后的防双签、key epoch 切换、委员会变更和 checkpoint / block sync。论文中的固定委员会安全性不能自行补齐这些接口。

第 5 项尤其重要：两轮强最终性的核心不是良好情况下少发一轮，而是已经发生但未被所有节点看到的提交，仍会约束所有后续合法提议。

## 8. 尚未解决的证据缺口与阅读顺序

### 8.1 采用前需要补齐的具体事项

- **Kudzu：** 完整实现及故障恢复的公开可复现实验；目前最强证据仍偏理论与性能模型。
- **Banyan：** 恶意多候选行为的通信 / 内存控制，以及生产所需的状态与恢复边界。
- **Hydrangea / ++：** 统一快路径 $p$ 定义；++ 补齐摘要性能数字对应的实验版本。
- **Minimmit：** 稳定实现的发布 / 维护状态、真实引擎同条件比较，以及编码方案调整后的完整论证。
- **ChonkyBFT：** 归档后的维护路线，及故障 / 落后恢复和尾延迟数据。
- **Alpenglow：** 对具体版本分别核对安全 theorem、扩展活性与传播假设；使用当前生产数据时注明仅 Votor 或包含哪一版数据层。
- **Commonware：** 应用所依赖的是 full finalization 还是 speculative feedback，certification hook 实际检查了什么。

### 8.2 建议阅读顺序

先读 [Simplex 作者协议说明][S05]，建立 notarize / finalize / nullify 的区别；再读 [纯两轮][S25] 与 [并发两 / 三轮帖子][S08]，理解残留证据与 Fast < Slow。随后读 [Banyan][S07] 的紧参数和网络实验、[Kudzu][S09] 的完整数据与通信分析。最后按需要进入 [Hydrangea][S12] 的混合故障模型、[Minimmit][S14] 的 view 推进，以及 [ChonkyBFT][S16] / [Alpenglow][S10] 的实现路线。

精确下界用 [Kuznetsov–Tonkikh–Zhang][S03] 和 [Abraham–Nayak–Ren–Xiang][S04] 核对；大块成本用 [DispersedSimplex][S06] 核对。不要只依靠后续论文的 Related Work 转述。

### 8.3 版本记录

论文比较采用 Banyan arXiv v3、Kudzu v2、Minimmit v5、ChonkyBFT v1，及 DISC 2024 DispersedSimplex 会议版。Commonware latest 文档本次标示 2026.10.0；latest / main 链接会变化，部署状态只对调研日期负责。

两份存在版本核对问题的 PDF，2026-11-02 取得文件的 SHA-256 如下，便于复核是否是同一份材料：

~~~
Hydrangea, USENIX PDF
12bdc6b328418e885fd56d80628d1c7f19677bd62f444dbbed340b640ca377a4

Hydrangea++, Supra PDF, 12 pages
25ad66cd640052852eb6e61ba1d7434fdc84330ef02fbca7684d54916b6ed81f
~~~

## 9. 原始文献与实现来源

以下均为论文、作者 / 项目官方材料或原实现仓库。正文的 [P] / [D] 区分论文边界与本文展开；未复现实验的数值均来自相应作者。

- **S01.** Martin, Alvisi. *Fast Byzantine Consensus*，FaB / Parameterized FaB。[论文][S01]；[作者项目页](https://www.cs.utexas.edu/~dahlin/projects/bft/)。
- **S02.** Gueta 等. *SBFT: a Scalable and Decentralized Trust Infrastructure*，DSN 2019。[论文 v3][S02]，重点 §IV–VI 的路径、collectors 和实验。
- **S03.** Kuznetsov, Tonkikh, Zhang. *Revisiting Optimal Resilience of Fast Byzantine Consensus*，2021，扩展版 v3。[论文][S03]。
- **S04.** Abraham, Nayak, Ren, Xiang. *Good-case Latency of Byzantine Broadcast: A Complete Categorization*，2021 / 修订 2022。[论文][S04]。
- **S05.** Chan, Pass. *Simplex Consensus: A Simple and Fast Consensus Protocol*，TCC 2023。[作者协议说明][S05]；[论文 ePrint 2023/463](https://eprint.iacr.org/2023/463)。本文详细机制主要核对作者说明，而非声称取得并审计了当前全部 ePrint 版本。
- **S06.** Shoup. *Sing a Song of Simplex*，DISC 2024，包含 DispersedSimplex。[会议版全文][S06]；[出版页](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.DISC.2024.37)。
- **S07.** Vonlanthen, Sliwinski, Albarello, Wattenhofer. *Banyan: Fast Rotating Leader BFT*，Middleware 2024，arXiv v3。[全文][S07]。
- **S08.** Decentralized Thoughts. *Combining 2-round and 3-round Simplex*，2025-07-29。[原帖][S08]。
- **S09.** Shoup, Sliwinski, Vonlanthen. *Kudzu: Fast and Simple High-Throughput BFT*，DISC 2025，arXiv v2。[全文][S09]；[版本 / 发表说明](https://arxiv.org/abs/2505.08771v2)。
- **S10.** Kniep, Sliwinski, Wattenhofer. *Solana Alpenglow Consensus*，2025。[作者论文页][S10]。本文未取得当前完整白皮书的可读版本，具体扩展模型同时核对 S11，未补造完整 bit 公式。
- **S11.** Anza. *Known Non-Issues*。[安全 / 活性条件说明][S11]，重点 equivocation、network synchrony、20+20 条件。
- **S12.** Shrestha, Kate, Nayak. *Hydrangea: Optimistic Two-Round Partial Synchrony with Improved Fault Resilience*，USENIX Security 2026。[正式 PDF][S12]。
- **S13.** Shrestha, Kate. *Hydrangea++: Enhancing Hydrangea with Optimistic Proposals*。[当前公开 PDF][S13]，草稿版本需结合 §8.3 核对。
- **S14.** Chou, Lewis-Pye, O'Grady. *Minimmit: Fast Finality with Even Faster Blocks*，arXiv 2508.10862 v5。[全文][S14]，重点 §6–7 的通信优化与模拟口径。
- **S15.** Commonware. *simplex module documentation*。[官方 Rust 文档][S15]，重点 Protocol Description、Optimistic Finality、Pipelined Handoff。
- **S16.** França, Kolegov, Konnov, Prusak. *ChonkyBFT: Consensus Protocol of ZKsync*，arXiv 2503.15380 v1。[全文][S16]，重点 quorum / view-change 和 Evaluation。
- **S17.** Solana Foundation. *Alpenglow upgrade status*，页面更新于 2026 年 10 月。[官方升级页][S17]。
- **S18.** Banyan 作者实现。[kobi-lizard/banyan][S18]。
- **S19.** Hydrangea 参数与 benchmark 配置。[config/src/lib.rs][S19]。
- **S20.** Hydrangea 作者 reference implementation。[nibeshrestha/hydrangea][S20]；[提交证据处理](https://github.com/nibeshrestha/hydrangea/blob/main/hydrangea/src/core.rs)。
- **S21.** Matter Labs. *era-consensus*。[归档仓库][S21]；BFT 引擎在 node/components/bft。
- **S22.** Commonware. *[consensus] Minimmit*, PR #2884。[PR][S22]；[实验分支](https://github.com/commonwarexyz/monorepo/tree/cl/minimmit/consensus/src/minimmit)。
- **S23.** Commonware. *estimator*，Minimmit 论文所指向的固定提交快照。[模拟器与协议脚本][S23]。
- **S24.** Alpenglow 作者参考程序，支持真实 UDP 集群与模拟。[qkniep/alpenglow][S24]。
- **S25.** Decentralized Thoughts. *2-round BFT in the Simplex style*，2025-07-18。[原帖][S25]；[更紧 $5f-1$ 的后续说明](https://decentralizedthoughts.github.io/2025-08-06-5fminus1-simplex/)。
- **S26.** Solana SIMD-0326。[Alpenglow 协议提案][S26]。
- **S27.** Commonware. [consensus/src/lib.rs][S27]；用于核对模块与应用接口，main 会变化。
- **S28.** Anza. *Alpenglow: A New Consensus for Solana*，2025 年发布说明。[作者模拟与目标说明][S28]。
- **S29.** Agave v4.3.2。[Votor 源码目录][S29]，用于定位生产集成代码，非本文完成代码审计的声明。

[S01]: https://www.cs.cornell.edu/lorenzo/papers/fab.pdf
[S02]: https://arxiv.org/html/1804.01626v3
[S03]: https://arxiv.org/abs/2102.12825v3
[S04]: https://arxiv.org/abs/2102.07240v3
[S05]: https://simplex.blog/protocol/
[S06]: https://drops.dagstuhl.de/storage/00lipics/lipics-vol319-disc2024/LIPIcs.DISC.2024.37/LIPIcs.DISC.2024.37.pdf
[S07]: https://arxiv.org/html/2312.05869v3
[S08]: https://decentralizedthoughts.github.io/2025-07-29-2-round-3-round-simplex/
[S09]: https://arxiv.org/html/2505.08771v2
[S10]: https://quentinkniep.com/research/solana-alpenglow-consensus.html
[S11]: https://github.com/anza-xyz/alpenglow/blob/master/KNOWN_NON_ISSUES.md
[S12]: https://www.usenix.org/system/files/usenixsecurity26-shrestha.pdf
[S13]: https://supra.com/documents/hydrangea-plus-plus.pdf
[S14]: https://arxiv.org/html/2508.10862v5
[S15]: https://docs.rs/commonware-consensus/latest/commonware_consensus/simplex/index.html
[S16]: https://arxiv.org/html/2503.15380v1
[S17]: https://solana.com/upgrades/alpenglow
[S18]: https://github.com/kobi-lizard/banyan
[S19]: https://github.com/nibeshrestha/hydrangea/blob/main/config/src/lib.rs
[S20]: https://github.com/nibeshrestha/hydrangea
[S21]: https://github.com/matter-labs/era-consensus
[S22]: https://github.com/commonwarexyz/monorepo/pull/2884
[S23]: https://github.com/commonwarexyz/monorepo/tree/19f19d32760daf1d497295726ec92a1e6b84959f/examples/estimator
[S24]: https://github.com/qkniep/alpenglow
[S25]: https://decentralizedthoughts.github.io/2025-07-18-two-round-Simplex/
[S26]: https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0326-alpenglow.md
[S27]: https://github.com/commonwarexyz/monorepo/blob/main/consensus/src/lib.rs
[S28]: https://www.anza.xyz/blog/alpenglow-a-new-consensus-for-solana
[S29]: https://github.com/anza-xyz/agave/tree/v4.3.2/votor/src

