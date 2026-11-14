# main_v1：区域 completion 的通信优化与文献

检索日期：2026-09-28。目标文件：[main_v1.tex](../main_v1.tex)，SHA-256 `f918927188fe07fd5962e0b2b65abde65565e727f1dbb811b2e49db8386cb13e`。本笔记不修改协议；文献定理、从当前源码得到的观察、建议的适配分别说明。它不是对新协议的完整证明认证或性能实测。

## 1. 当前成本来自哪里

令 `n=n_i`、`W=W_i`。当前同步区域模型要求 responsive honest weight `>2W/3`，没有要求 responsive honest **identity count** `>2n/3`。

- R4（源码 616–619 行）：在 `3Δ_R` 冻结 lock，所有成员，包括已 fast-decide 的成员，参加 completion。
- B1/B2（638–654 行）：运行 `n` 个并行的 authenticated origin broadcasts，得到共同向量，再按原始权重选择支持度 `>2W/3` 的非空值，否则选 bottom。
- Appendix A（1444–1496 行）：每个 origin 使用带轮次限制的 Dolev–Strong 风格签名链；每个节点每个 origin 最多转发两个语义值。
- 成本（1335–1374 行）：每个 origin 最坏 `O(n²)` 点对点消息，`n` 个 origin 得到 `O(n³)` 消息；控制位数为 `O(n³(L_R^c+nκ))`。其中 `L_R^c` 已包括语义证据；新签名链另计。
- 当前正常实例也承担该 completion。`τ_R≥(n+5)Δ_R` 是相邻依赖实例的间隔，`2δ_R` 只是内部 fast decision 的 good-case latency。
- bulk body caching 已经写入 1389–1400 行；它不能消除上面的 origin 数量、控制消息和签名链成本。

因此要分别优化：**输入向量重建、短值 agreement、证书字节、bulk data、轮数**。降低其中一项不自动降低其余各项。

## 2. 最值得先利用的结构：只有一个可能的非空候选

**当前源码的性质。** 两个不同 `Q_x` 的 signer 权重交集大于 `W/3`，且 honest 成员每实例只签一个 VOTE。因此，所有合法非空 lock 的语义值相同，最多有一个 `x`。证书的 signer 集可以不同；不要把不同的 `Q_x` 编码误当成不同候选值。

这说明区域 completion 所需的 agreement contract 可以通过以下方式研究实现：

1. 每个成员在原来的 `3Δ_R` 截止时间冻结 `b_S=1` 当且仅当其 lock 非空；否则 `b_S=0`。
2. 运行具有 honest-input unanimity validity 的 weighted binary BA。
3. BA 输出 0 时返回 bottom；输出 1 时，取得并验证唯一 `x` 的 `Q_x` 和完整 witness，再返回 `x`。

**适配逻辑。** 若曾有 responsive honest fast decider，则其转发使所有 responsive honest 成员在截止前持有同一个 `x`，所以全部输入 1。若全部输入 0，unanimity 要求输出 0。输出 1 则意味着至少一个 responsive honest 成员初始输入 1，因而至少有一个可靠 holder 持有合法完整对象；候选唯一性消除了取回不同 `x` 的风险。

这是一条 reduction 方案，不是完整实现：必须给出有通信上界和截止时间的 retrieval，保证每个输出 1 的成员取得完整 witness。若把多签证书在所有边上重复发送，仍会重新引入 cubic bits。

**语义边界。** 该方案保留 agreement、termination、unanimity、external validity 和 fast-decision compatibility；它不保证在混合 lock 输入时逐执行复现现有 B2 的 `>2W/3` 向量选择结果。若必须保留 B2 的确切选择函数，就应研究 parallel broadcast / interactive consistency，而不能直接用 binary BA 替代。此 reduction 也不能直接套用到具有多个不同合法报告的 global report vector。

## 3. 候选方案与可信度

| 路线 | 文献保证或明确的成本对象 | 对当前版本的适配 | 建议 |
|---|---|---|---|
| Weighted King + binary reduction | 核心 `O(αn²)` 常数大小消息，`3α` 个同步通信轮 | 直接对应 `<1/3` faulty weight；另加 witness、export | 最容易形成可审查的完整实现 |
| Weight reduction + Momose–Ren BA | `O(n²κ)` **短值 agreement 核心**，deterministic、PKI | 以容错到接近 `1/2` 的底层吸收权重离散化误差 | 最有价值的二次通信目标 |
| Multi-shot agreement | 文献中 linear amortized communication、constant amortized latency | 需要跨实例保留状态、threshold setup、权重适配 | 长期 streaming 设计的第二阶段 |
| Parallel broadcast | 文献中约 cubic / quadratic bits，分别对应不同 setup | 保留共同输入向量；额外 count-fraction、randomization 条件 | 仅在确实需要 origin vector 时优先考虑 |
| QC 聚合 / threshold signatures | 压缩证书，通常不减少逻辑消息数 | 需 signer/weight verification；threshold key setup 单列 | 与上述路线叠加 |
| Extension / dispersal / retrieval | 大值通信可接近 `O(nℓ)` 加短值 agreement 开销 | 保持 complete-object validity，单独计算 witness 和 body | 避免短值 BA 的收益被大证据抵消 |

### A. Weighted King：最直接的权重原生替换

Vijay K. Garg and John Bridgman, **The Weighted Byzantine Agreement Problem**, IPDPS 2011，尤其 Section IV / Figure 2 / Theorem 2。[作者全文](https://users.ece.utexas.edu/~garg/dist/ipdps11.pdf)，[DOI](https://doi.org/10.1109/IPDPS.2011.57)。

**文献结果。** Weighted King 在 faulty weight `<1/3` 下实现 binary agreement，每个 king 阶段含三轮。按权重排序选出的 king 集只要总权重大于允许的故障权重，就至少包含一个正确 king。每阶段两次 all-to-all 和一次 king fan-out；没有 Dolev–Strong 长签名链。

**对本稿的保守实例化。** 将 `V_i\H_i` 全部按 faulty/omission 处理；其权重 `<W/3`。固定一个按权重降序、累计权重 `>W/3` 的前缀，长度为 `α`。该前缀必含 responsive honest king；可使用其全部 `α` 个阶段。`α≤⌊n/3⌋+1`，权重集中时可以更小。

由消息流程推得核心 `O(αn²)` 消息；若每个小消息计 `κ` 位认证/标识开销，则为 `O(αn²κ)` 位。最坏仍 cubic messages，但去掉了现有 quartic signature-bit term。不要将其表述为一般的 quadratic-message BA。

加上 binary reduction 的 witness distribution 和 DECIDED export 后才是完整区域成本。即使取一个保守的每 holder 向所有人发送 `O(nκ)` 大小 QC 的方案，额外控制成本是 `O(n³κ)`，不会恢复签名链的 `O(n⁴κ)` 项；bulk body 则应继续独立计算。更有效的 retrieval 可进一步改进它。

### B. Momose–Ren + 权重离散化：更强的 asymptotic 目标

Atsuki Momose and Ling Ren, **Optimal Communication Complexity of Authenticated Byzantine Agreement**, DISC 2021，Theorems 1–2、Sections 2–4。[正式全文](https://drops.dagstuhl.de/storage/00lipics/lipics-vol209-disc2021/LIPIcs.DISC.2021.32/LIPIcs.DISC.2021.32.pdf)。

**文献结果。** 对 `O(κ)` 位输入，提供 worst-case `O(n²κ)` communication 的 synchronous BA：threshold-signature 版本容忍 `f<n/2`；standard-PKI 版本容忍 `f≤(1/2−ε)n`，其中 `ε>0` 为常数。具备 honest-input unanimity；不应直接当作携带任意大 witness 的 externally validated BA。

权重适配的依据是 Andrei Tonkikh and Luciano Freitas, **Swiper: a new paradigm for efficient weighted distributed protocols**, PODC 2024。阅读 Weight Restriction 和 black-box transformation；[作者版本 §2.1、§4.4](https://arxiv.org/html/2307.15561v2)。它把权重映射成受控数量的虚拟身份；底层容错比例与原始权重容错比例之间需要留出余量。

**一个可以直接核算的本稿组合方案。** 不必额外假设原始 Byzantine 节点数少于 `n/3`。设置

\[
t_S=\left\lceil 12n\frac{w_S}{W}\right\rceil,
\qquad T=\sum_{S\in V_i}t_S.
\]

因为每个 `w_S>0`，每个真实成员至少模拟一个身份，且 `12n≤T<13n`。令 `F=V_i\H_i`，则

\[
\sum_{S\in F}t_S
\leq12n\frac{\mathrm{wt}(F)}W+|F|
<5n\leq\frac5{12}T.
\]

因此底层选取 `ε=1/12` 的 PKI 版本即可覆盖该虚拟故障集合。每个 honest 成员的虚拟身份采用同一个冻结的 binary input；底层的 unanimity 随之继承。`T=O(n)`，所以短值 agreement 核心保持 `O(n²κ)` 位通信。

这段 ceiling construction 和它与 main_v1 的组合是**本笔记的推导**，不是两篇论文对本稿已有的定理。它说明不必为套用 count-based 文献而削弱原始 `<1/3` 权重容错；不是建议实际部署固定 12 倍身份。实际可用 Swiper 寻找更小映射。

必须补齐：配置共同确定虚拟成员及其 PKI / domain separation；非响应成员的处理；按语义值保持 validity；witness retrieval；真实成员的 DECIDED；轮次预算和签名验证负载。虚拟票只服务于 completion，不能据此更改原协议的 leader 权重、VOTE/DECIDED quorum 或奖励分配。

`O(n²κ)` 在这里仅指短值 agreement。未经相应设计，不能把它写成完整 in-region phase 的总成本，也不能承诺更短的 worst-case latency；该 BA 的轮数仍为线性量级，虚拟身份还会增加常数。

### C. 长期流：multi-shot / amortized agreement

Jun Wan, Atsuki Momose, Ling Ren, Elaine Shi, Zhuolun Xiang, **On the Amortized Communication Complexity of Byzantine Broadcast**, PODC 2023。[作者全文](https://eprint.iacr.org/2023/038.pdf)，Table 1。

文献的 honest-majority、threshold-signature 分支在 `f≤(1/2−ε)n` 下，`K` 次短值 broadcast 的总成本为 `O(κnK+κn³)`，因此长期摊销 `O(nκ)`；它没有声称每一次都是 linear。论文另有任意 dishonest-majority 的分支，成本不同，不能混用。

更近的 Pierre Civit et al., **Repeated Agreement is Cheap! On Weak Accountability and Multishot Byzantine Agreement**, PODC 2025，提供 synchronous、`t<n/2`、threshold-signature 模型下，兼有 strong unanimity 与 external validity 的 multi-shot agreement：`O(n(L_o+κ))` amortized communication、`O(1)` amortized latency，并限制慢实例数量。[全文](https://eprint.iacr.org/2025/826.pdf)，Figure 1、Section 3；[会议记录](https://www.podc.org/podc2025/schedule/)。

**适合本项目之处：** 配置内成员和故障集固定，且本来就是一串区域实例，跨实例积累信息有价值。**适配成本：** 持久争议/排除状态、内部委员会与原始 stake quorum 的关系、权重模拟、threshold setup、恢复或 region handoff 时是否保留摊销状态。2025 论文的弱 accountability 可以排除一个含故障者的争议对，不能把被排除者直接认定为可罚没的作恶者。

这些是改变 completion 架构的方案。现有 R2 all-to-all VOTE 和 R5 export 仍在时，即使 completion 摊销 linear，整个区域阶段也不会自动 linear。

### D. 如果必须保留输入向量：专门优化 parallel broadcast

Georgios Tsimos, Julian Loss, Charalampos Papamanthou, **Gossiping for Communication-Efficient Broadcast**, CRYPTO 2022。[会议全文](https://crypto.iacr.org/2022/papers/538805_1_En_15_Chapter_OnlinePDF.pdf)，Table 1、Sections 5–6。

其 binary Parallel Broadcast 专门避免黑盒执行 `n` 次单发送者广播：BulletinPBC 在普通 public-key 模型达到 `\widetilde O(n³κ²)` bits；TrustedPBC 在更强 trusted-PKI setup 下达到 `\widetilde O(n²κ⁴)` bits。轮数分别为 `O(n log n)` 和 `O(κ log n)`；包含概率性保证。

**不能直接替换的原因：** 两个结果要求 honest **identity** fraction 至少某个常数 `ε`，即 `t<(1−ε)n`；当前权重假设未提供这个条件。论文给出的主要协议是 binary，不能忽略把 proposal/QC/witness 编入协议的成本。对本区域任务，先移除不必要的向量重建通常更合适；这篇文献更适合必须保留 B2 或 global report vector 的后续研究。

### E. 证书与大数据优化应与 agreement 分开

**QC 聚合。** Boneh–Drijvers–Neven 的 [BLS Multi-Signatures With Public-Key Aggregation](https://crypto.stanford.edu/~dabo/pubs/papers/BLSmultisig) 给出了安全聚合共同消息签名的方法，并处理 rogue-key 风险。应用到 VOTE/DECIDED 时，采用已证明的安全变体或合适的 PoP 注册规则。

当前独立签名 QC 可有 `O(nκ)` bits。使用聚合签名加 signer bitmap 时，证书仍约为 `O(κ+n)` bits；权重总和必须由实际 signer set 验证。把这种证书 all-to-all 发送仍有 bitmap 项，不能称为总 `O(n²κ)` 位。真正 constant-size threshold certificate 需要额外的密钥生成、share 分配和权重阈值设计；accountability 所需的个别签名也不能随意丢失。

当前 CHAIN 的每一层签名绑定前一个 prefix。它与同一消息上的 QC 聚合不是同一 primitive；不能直接把普通 BLS aggregate 替换进去而省略有序 signer 信息、前缀绑定与轮次验证。即使密码学压缩成功，`n` 个 origin 的消息数量仍然存在。

**大值 extension。** Kartik Nayak et al., **Improved Extension Protocols for Byzantine Broadcast and Agreement**, DISC 2020，[正式全文](https://drops.dagstuhl.de/storage/00lipics/lipics-vol179-disc2020/LIPIcs.DISC.2020.28/LIPIcs.DISC.2020.28.pdf)，Table 1。其 authenticated synchronous BA、`t<n/2` 分支的通信为 `O(nℓ+A(κ)+n²κ)`，其中 `ℓ` 是大值位数、`A` 是短值 BA 成本。它优化 payload 系数；不会消除任意多次 BA 调用的成本。计入参数时也要包括论文所需的编码/承诺原语。

适配到本稿时，重点是让 proposal body、QC、history evidence 与短值控制协议分离，保持 reconstruction 和 external validity。所有 responsive honest 成员都必须获得 `b` 位新数据时，至少要给这些接收者传递相应信息；hash 不能代替 bytes。当前稿已经有 body cache，新的收益要与该 baseline 比较。

可进一步阅读 Civit et al., **DARE to Agree: Byzantine Agreement With Optimal Resilience and Adaptive Communication**, PODC 2024：[正式作者存档](https://infoscience.epfl.ch/server/api/core/bitstreams/d008e100-d4b1-4ad7-8a8f-60263deb1868/content)，[完整版](https://eprint.iacr.org/2024/403.pdf)。它把 Disperse–Agree–Retrieve 与 validity 一起处理；不同分支使用不同 threshold primitives，尤其不要把 MTSS 分支写成普通 signatures-only 方案。

## 4. 建议的优先顺序与验证边界

1. **先定义优化契约。** 保留 R1–R3 的 `2δ_R` fast decision 和冻结截止；用 agreement/unanimity/external validity 定义 completion 接口，并明确是否允许改变混合输入下的 B2 选择结果。
2. **实现易审查的基线。** Weighted King + binary reduction + 有界 witness distribution；完整计算 messages、control bits、body bits、verification、completion rounds。
3. **追求二次控制开销。** 用 weight reduction + Momose–Ren 替换短值 BA；同时设计 retrieval 与证书传播，否则核心优化会被大证据广播淹没。
4. **需要持续吞吐时再研究 multi-shot。** 明确有限配置长度下的总成本，而不是仅报无限序列的摊销极限。region handoff 和状态重置可能重新支付昂贵阶段。

必须保持的接口：一个合法非空候选；截止前冻结 input；每个输出者持有完整 witness；早决定成员继续完成规定工作；独立 DECIDED certificate；新实例只能在父结果和 completion 的新终止条件满足后启动。总时序应改成 `3Δ_R + T_completion + T_export` 并重新推导 `τ_R`，不能沿用 `(n+5)Δ_R` 或把它写成 `2δ_R`。

不要只因本地看到 fast QC 就跳过 completion：其余节点是否拥有同一继续执行状态需要单独证明。也不要用普通 gossip 或异步 RBC 直接替代这里的有界终止广播；它们的故障发送者 termination contract 可能不同。

**当前最可信的结论：** 现有 cubic-message / quartic-signature-bit completion 是一个保守实现的成本，并非区域任务必需的复杂度。Weighted King 是与现有权重模型最直接匹配的成熟替换；weight reduction + quadratic authenticated BA 是更强且有文献支撑的研究方向。完整区域阶段的 quadratic-bit bound 尚需证书、可用性和时序的组合设计，不能仅引用短值 BA 定理宣称已经实现。
