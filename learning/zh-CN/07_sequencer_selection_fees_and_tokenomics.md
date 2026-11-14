# 排序器如何被选择、如何赚钱：OP Stack、OP Mainnet、Base 与 MegaETH

**资料核对日期：2026 年 9 月 21 日；2026 年 9 月 22 日补充 Fault Proof 激励。** 本文统一讨论排序器选择、收入分配、代币经济，以及 Challenger 的参与、争议报酬与抗攻击约束。

[返回中文索引](README.md) · [先读：OP Stack 架构与确认流程](06_op_stack_architecture_and_pipeline.md) · [相关系统与激励设计](04_related_systems_and_incentives.md)

本文区分四种证据：**协议规则**、**官方公布的现行安排**、**历史安排／实验**、**计划与尚未确认的细节**。“官方称已上线”以公告及升级记录为依据；本次没有逐笔审计链上费用流、回购成交、全部角色权限或私下商业合同。本地 MegaETH 规格仍固定在 `59797b45b1f59b7c2c147e6f99be39399a1f2b44`，不能用该快照替代当前部署证据。

快速阅读：[通用授权与费用](#2-op-stack排序器资格与费用规则) → [三链对照](#3-三条链的对照) → [OP Mainnet](#4-op-mainnet-与-op服务收入collective-财政和代币治理)／[Base](#5-base运营业务与独立技术栈不能沿用旧分成)／[MegaETH](#6-megaeth当前单活授权未来将运营权与-mega-连接) → [Challenger 与 Fault Proof](#7-challenger-与-fault-proof-系统的激励) → [激励分析](#8-这些收入与-token-机制是否足以保证诚实) → [轮换设计问题](#9-对-megaeth-sequencer-rotation-研究最有用的待定义项)。

## 1. 先把五种权利分开

| 问题 | 实际在分配什么 | 不应混同的概念 |
| --- | --- | --- |
| 谁能成为 sequencer？ | 运营者资格、授权密钥 | 交易用户提高 tip，不等于取得出块权 |
| 谁在这一时刻出块？ | 已授权运营者／机器之间的活跃身份 | HA 集群选主，不等于公共 PoS 选举 |
| 用户付什么钱？ | 执行、数据发布以及可能的运营附加费 | 支付 gas 的币，不一定是项目代币 |
| 收入归谁？ | Vault 收款权、运营合同、财政分配 | 收入不是净利润，vault 名称不是最终受益人 |
| Token holder 获得什么？ | 治理权、奖励资格、访问权，或回购带来的间接联系 | 锁币、出块权、现金分红和可罚没保证金不是同一件事 |

**OP Stack 是一套可配置的技术栈，没有为所有采用者规定统一的 sequencer 选举或统一的代币经济。** OP Mainnet 是具体链；Optimism Collective 是治理与经济组织；OP 是代币；Superchain 的成员关系和收入协议又是另一层。使用 OP Stack 代码，不自动产生向 OP 持币者分配交易费的义务。[OP Stack 配置与角色](https://specs.optimism.io/protocol/configurability.html)、[Superchain 收入规则](https://optimism.io/blog/how-%28and-why%29-the-superchain-drives-fees-to-the-optimism-collective)

尤其需要更新两个常见印象：**Base 在 2026 年已进入独立升级的 Base 技术栈阶段；MegaETH 官网仍把质押驱动的 sequencer rotation 标为 Planned。** 所以本文比较的是三个系统截至核对日各自公布的安排，而非三个完全相同的 OP Stack 部署。[Base 升级记录](https://docs.base.org/upgrades/overview)、[MEGA 官网](https://www.megaeth.com/token)

## 2. OP Stack：排序器资格与费用规则

### 2.1 “选择排序器”通常首先是授权配置

OP Stack 的配置中有两个不同身份：

- **Unsafe block signer**：用于认证链下传播的 unsafe 区块。
- **Batch submitter**：其 L1 交易能够被识别为合法的 batch 输入。

这些配置由相应的 `SystemConfig` 管理权限控制。第三方节点验证身份与数据；公开运行一个节点、持有 OP 或为 dispute game 押保证金，不会自动获得这两个权限。[System Config Owner](https://specs.optimism.io/protocol/configurability.html#system-config-owner)、[Batcher 身份认证](https://specs.optimism.io/protocol/derivation.html#batcher-transaction-format)

必须分别询问：**谁有权指定运营者、谁有权修改 key、哪个物理实例当前活跃？** 一个获授权的运营方可以配置多个备用实例。`op-conductor` 在这样的集群中用 Raft 协调 leader 和 unsafe head；官方明确它不提供 Byzantine fault tolerance。它处理可靠性，不定义一个开放的 stake／auction 市场。[OP Conductor](https://docs.optimism.io/chain-operators/tools/op-conductor)

Permissionless fault proofs 开放的是提议／争议的参与路径，不是公共排序权限。改变状态证明机制，也不必然改变 sequencer 的选择方法。

### 2.2 用户付出的费用：先看组成，再看最终去向

对启用相关升级的普通 sequenced transaction，可以用下式理解费用：

$$
F_{\mathrm{tx}}=g(b+p)+F_{\mathrm{L1data}}+F_{\mathrm{operator}}.
$$

这里 $g$ 是计费用的 gas，$b$ 是 L2 base fee，$p$ 是实际支付的 priority fee，后两项分别是 L1 数据费和可配置的运营费。它不是用户填写的 gas limit 乘以所有 fee caps；实际收费、退款和 gas 规则以激活版本为准。Deposit 交易另有从 L1 购买资源的处理，不能照搬普通交易公式。[交易费用](https://docs.optimism.io/op-stack/transactions/fees)

| 组成 | 直接进入的 L2 vault | 要理解的机制 |
| --- | --- | --- |
| $g b$ | `BaseFeeVault` | OP Stack 的 L2 base fee 归集到 vault；不像 Ethereum L1 base fee 那样直接销毁 |
| $g p$ | `SequencerFeeVault` | 主要收集 tips；这个特定 vault 不代表全部“排序收入” |
| $F_{\mathrm{L1data}}$ | `L1FeeVault` | 向用户收取估计的数据发布成本，不是每笔交易都与 batcher 的真实 L1 支出一一相等 |
| $F_{\mathrm{operator}}$ | `OperatorFeeVault` | 可用于额外运营定价；是否非零及参数值取决于该链 |

Vault 的 `recipient`、提现到 L1 还是 L2，以及后续是否经财政／分账合约，决定实际资金路径。任何人可以触发某些 vault 的提现，不意味着调用者取得这笔收入：钱仍发给配置的 recipient。[Fee vaults](https://docs.optimism.io/op-stack/transactions/fee-vaults)、[Execution Engine 的费用规则](https://specs.optimism.io/protocol/exec-engine.html#fees)

Operator fee 也有版本差异：Isthmus 引入它，Jovian 改变了 scalar 的计量公式。概念上可记作 $c+\alpha g$，但不要把旧代码中的 scalar 数值直接当作新版本里的同一单位。存在该模块也不表示 OP Mainnet 或每条标准链当前都收取非零 operator fee。[Isthmus](https://specs.optimism.io/protocol/isthmus/exec-engine.html)、[Jovian](https://specs.optimism.io/protocol/jovian/exec-engine.html)

### 2.3 L1 data fee 不直接等于 Ethereum 验证者拿到的钱

用户在 L2 支付 data fee；batcher 另行在 L1 支付发布交易所需的费用。两者之间隔着压缩效率、batch 大小、报价 scalar、提交时点和 L1 价格变化。

因此需要分别记账：

$$
R_{\mathrm{user}}=\sum_{\mathrm{tx}}F_{\mathrm{tx}},
\qquad
M_{\mathrm{after\ L1}}=R_{\mathrm{user}}-C_{\mathrm{L1}}.
$$

$C_{\mathrm{L1}}$ 是实际的 L1 发布等支出；$M_{\mathrm{after\ L1}}$ 只是扣除这一项后的差额。运营者还可能承担服务器、带宽、证明、监控、资金占用、商业分成等成本，所以它仍不是最终利润。[L1 数据费估计](https://docs.optimism.io/op-stack/transactions/fees#l1-data-fee)

下面的式子是**分析用账本，不是某条链的合约公式**：

$$
\Pi=R_{\mathrm{user}}+R_{\mathrm{other}}
-C_{\mathrm{L1}}-C_{\mathrm{infra}}-C_{\mathrm{proof/monitor}}
-P_{\mathrm{share}}-C_{\mathrm{capital}}.
$$

$R_{\mathrm{other}}$ 可以包括商业服务、补贴、被运营者实际获得的 MEV 或其他收入，但需要逐项证据；不能把链上所有可见 MEV 都算成 sequencer 收入。这个区分也是阅读“低 gas”“按成本运营”“净收入分成”时的基础。

## 3. 三条链的对照

表内是官方材料能够支持的描述；具体证据和时间边界见第 4—6 节。

| 维度 | OP Mainnet | Base | MegaETH |
| --- | --- | --- | --- |
| 与 OP Stack 的关系 | Optimism 的具体主网部署 | 源自 OP Stack；2026 年已开始独立升级 | 采用并改造相关 L2 架构；不能照搬标准 OP 的全部规则 |
| 当前排序资格 | Optimism 管理结构下的配置授权 | Base／Coinbase 运营体系，配置授权，单一活跃排序器 | 项目方授权的单活模型；MegaLabs 运营者身份来自 2026-01 官方披露 |
| 是否已核实公开 stake-to-sequence | 没有；OP 持仓不自动授予排序权 | 没有；不能由开放节点／公开证明推出开放排序 | 官网仍将 stake + performance rotation 标为计划 |
| Gas 币 | ETH | ETH | ETH；MEGA 不是 gas 币 |
| 用户费经济模式 | 执行与数据费等归集，再进入适用的 Collective 经济安排 | 排序服务收入；实际 L1 与运营支出另计 | ETH 费用支持运营；另以 USDm 收益与生态增长为经济来源 |
| 项目 token | OP | 仅核实到探索网络 token 的官方公告，未核实正式发行 | MEGA，2026-04-30 上线 |
| Token 与收入的连接 | 治理及预算；2026 年获批、已开始执行的 OP 回购 | 没有已核实的官方网络 token 分账模型 | Foundation 收到的 USDm rewards 用于买入并积累 MEGA；另有 KPI commitment 奖励 |
| 仍属计划／不能假定的机制 | 买回 OP 的未来安全质押等用途 | 正式网络 token、公开 token 选主 | Proximity Markets、Sequencer Rotation 的完整实施规则 |

这里“当前排序资格”没有声称本次审计了所有管理员签名人或运营合同。**授权接口可核验，不等于选择背后的商业安排全部公开。**

## 4. OP Mainnet 与 OP：服务收入、Collective 财政和代币治理

### 4.1 谁选 sequencer，为什么愿意运营

OP Mainnet 通过相应管理结构指定服务角色，并由配置中的 signer／batcher 身份执行认证。这个模型没有一个普遍规则要求候选人先质押 OP，再按持币权重抽签出块。官方角色文档中的 Optimism 管理体系与实际服务地址，应与任何未来去中心化选举提案区分。[SystemConfig](https://specs.optimism.io/protocol/system-config.html#unsafe-block-signer)、[权限配置](https://specs.optimism.io/protocol/configurability.html#service-roles)

从经济分析角度，现有运营者更接近一个长期经营基础设施的服务提供方：稳定服务维持交易需求、收入和生态价值。运营公司的成本及报酬，与 Collective 如何处分链收入，不必是同一个账户。不能把所有财政收入直接当作运行某台 sequencer 的个人奖励。

### 4.2 Superchain 分成：共享经济安排，不是开源许可费

官方公布的标准 Superchain 分成模型，对适用的成员链取以下两者较大值：

$$
P_{\mathrm{Collective}}
=\max\{0.025R,\;0.15(R-C_{\mathrm{L1}})\}.
$$

$R$ 是链的费用收入，$R-C_{\mathrm{L1}}$ 是这里特定定义的 **net onchain profit**。这是扣除 L1 gas 成本后的口径，并非公司净利润。适用范围取决于加入的安排；免费使用或 fork 开源 OP Stack 本身，不强制采用该分成。[官方分成说明](https://optimism.io/blog/how-%28and-why%29-the-superchain-drives-fees-to-the-optimism-collective)

**教学例子，不是真实财务数据：**假设收入为 100 个计价单位，实际 L1 支出为 40，那么两项分别是 2.5 和 9，分成为 9。剩余 51 还要承担其他成本。这个例子也不能直接用于现行 Base，原因见第 5 节。

OP Mainnet 在官方 Collective 收入指标中是特殊项：贡献 **100% 的 net onchain profit**。不能将这句话改成“100% 的用户交易费都没有成本地发给 OP 持币者”。[官方收入指标定义](https://optimism.io/blog/metrics-to-be-optimistic-about-in-2025)

### 4.3 OP 的供给与分配，和 gas 收入分别看

OP 是治理与生态预算资产，OP Mainnet 的 gas 仍以 ETH 支付。2026-08-06 官方预算报告沿用以下原始分配框架，合计 **4,294,967,296 OP**：

| 原始分配类别 | 比例 |
| --- | ---: |
| Ecosystem Fund，合并包含 Governance Fund | 25% |
| 用户 Airdrops | 19% |
| Retro Funding | 20% |
| Early Core Contributors | 18.9% |
| Investors | 17.1% |

这不是截至今日各池余额，也不是每期奖励率。已分配、已承诺、已归属解锁、实际流通是不同概念；预算用途还能受后续治理影响。本文不把 8 月流通量快照冒充 9 月实时数据，也不把上述总量基数当作未经限定的永久供应上限。[2026-08-06 预算报告](https://gov.optimism.io/t/collective-year-4-budget-update-and-year-5-budget-outlook/10796)

### 4.4 2026 年 OP 回购：已通过并已有执行披露

2026-01-28，相关治理提案获通过，授权一个为期 12 个月的计划，将纳入范围的 Superchain 收入的 50% 用于购买 OP。这个百分比位于 **Collective 收入层**，不是直接截取每条链用户手续费的一半。[获批确认](https://gov.optimism.io/t/proposal-to-align-the-op-token-with-superchain-success/10527/38)、[计划原文](https://gov.optimism.io/t/proposal-to-align-the-op-token-with-superchain-success/10527)

官方执行帖 3 月 5 日披露首期使用 95.8 ETH 购买 1,574,817 OP；8 月 7 日披露第二、三期后，累计公布购买 **9,451,924 OP**。这是截至读取时该帖披露的累计数，不能推断到 9 月每一期都已按时完成。买回的币进入指定财政账户；不能自动推导为 burn、按持仓分红或每个持有人享有固定赎回权。[回购执行记录](https://gov.optimism.io/t/buyback-communication-thread/10588)

### 4.5 一个容易混淆的历史实验：stake-to-prioritize

OP Mainnet 于 2026-05-26 开始最长四周的 stake-based priority ordering 实验。用户质押 OP，为自己或指定 beneficiary 争取交易排序优先；官方归档明确规定实验窗口结束后回到标准 priority-gas-auction（PGA）排序，而且优先处理不保证纳入、绝对次序或 finality。

**这里是交易需求方质押以争取优先，不是基础设施供给方质押以取得 sequencer 权限。** 因此，“OP 只在任何时期都用于治理”太绝对，“OP 已成为排序器 PoS 质押币”也不成立。[归档实验说明](https://docs.optimism.io/notices/archive/stake-based-priority-ordering)

## 5. Base：运营业务与独立技术栈，不能沿用旧分成

### 5.1 当前资格仍是服务授权，HA 选主是另一层

Base 当前规范描述单一活跃 sequencer；`SystemConfig Owner` 管理 P2P signer 和 batch submitter。Coinbase 的公开披露将 Base 排序服务作为其运营及收入业务的一部分。本次未发现向公众开放的“质押 Base／OP 代币后竞选排序器”准入规则。[Base protocol overview](https://docs.base.org/specifications/base-protocol/overview)、[Base 配置](https://docs.base.org/specifications/reference/configuration)、[Coinbase 收入披露](https://investor.coinbase.com/files/doc_financials/2024/Shareholder-Letter-Q1-2024.pdf)

2025 年 Base 工程文章披露过由五个 HA 实例通过 Conductor／Raft 协调的部署。这说明多个实例可以支持一个逻辑排序服务，不代表五个公众验证者分享 token staking 收益，也不证明 2026 年实例数仍相同。[Flashblocks Deep Dive](https://blog.base.dev/flashblocks-deep-dive)

### 5.2 2026 年发生了什么变化

Base 宣布采用统一的 Base-operated stack，并从自己的发行渠道演进。官方升级表确认 Azul 于 **2026-05-28** 主网上线，是首次独立升级；Beryl 于 **2026-06-25** 上线。因此“Base 始终与最新 OP Stack 完全相同”已不是合适描述。代码继承、升级治理、证明系统和经济协议都需要分别检查。[统一技术栈公告](https://blog.base.dev/next-chapter-for-base-chain-1)、[升级记录](https://docs.base.org/upgrades/overview)

### 5.3 费用流与 token 状态

Base 费用仍以 ETH 计，公开说明的主分类为 L2 execution fee 与 L1 data/security fee。当前执行规格把 base fee、tip、L1-cost fee 分别归集到对应 vault，**L2 base fee 不销毁**。Operator fee 机制存在，但非零收费不能从模块存在推出。本次没有逐块查询其参数或最终 recipient，不能给出未经验证的固定分账比例。[Base Network Fees](https://docs.base.org/specifications/transactions/network-fees)、[Base execution fees](https://docs.base.org/specifications/base-protocol/execution/l2-execution-engine#fees)

Base 的 token 声明也需要时间边界：2025 年 BaseCamp 官方宣布正在探索网络 token，未给出确定的设计和时间。截至本次检索，**未找到正式发行官方 Base network token 及确定 tokenomics 的一手公告**。因此不应将同名市场代币当作 Base 排序权益，也不应把早期“目前没有发行计划”的表述当作当前完整立场。[BaseCamp 官方声明](https://blog.base.org/the-state-of-base-at-basecamp-2025)

### 5.4 与 Optimism 的旧分成已经不能当作现行条款

| 时间／状态 | 可以确认的安排 |
| --- | --- |
| 2023 年历史协议 | Base 承诺按收入 2.5% 与扣除 L1 数据成本后的利润 15% 二者较高值贡献；可附条件赚取最多 OP 供应的 2.75%，另有 9% 投票上限 |
| 2026-02-26 官方澄清 | 随向 Base-operated stack 转换，原 Superchain revenue share 不再继续；未归属的 OP grant 不再有权取得；9% 投票上限仍适用 |
| 新商业关系 | Base 继续作为 OP Enterprise 客户；该商业服务的精确付款金额／比例未在上述材料披露 |

来源：[2023 年承诺](https://blog.base.org/bases-commitment-to-decentralization-with-the-superchain)、[2026 年官方澄清](https://gov.optimism.io/t/base-the-superchain-and-governance-questions-that-merit-answers/10612/4)。

2026-03-03 的进一步说明指出，Enterprise 收入流向 Optimism Foundation，但不在此前批准的 Superchain revenue-share 回购计划范围内。因此，既不能写“Base 继续按旧公式分成”，也不能写“Base 与 Optimism 之间已经没有任何经济往来”。[Enterprise 收入口径](https://gov.optimism.io/t/base-the-superchain-and-governance-questions-that-merit-answers/10612/6)

### 5.5 排序政策与经济中立性

当前 Base 文档说明其交易排序会考虑 priority fee 与到达时机；晚到的高费交易不能倒插已经提交的 Flashblock 前缀。Coinbase 另有不优待自有业务、不滥用未公开交易信息的中立性原则。这些政策或运营承诺不能自动等同于可由合约检验的全局公平排序证明，也没有据此确认公共 MEV 分红。[Transaction Ordering](https://docs.base.org/specifications/transactions/transaction-ordering)、[Neutrality Principles](https://www.coinbase.com/blog/coinbases-neutrality-principles-for-base)

## 6. MegaETH：当前单活授权，未来将运营权与 MEGA 连接

### 6.1 当前知道什么，仍不知道什么

2026-01-25 的官方 MiCA 文件 H.4—H.6 描述单活 sequencer，并披露由 **MegaLabs** 运营；这是运营者身份的较早披露。本次没有找到更新的官方替换公告，但也没有据此审计当前全部运营主体和授权管理人。[MiCA 白皮书](https://static.megaeth.com/MiCA%20Crypto-Asset%20White%20Paper%20-%20MEGA%20Token.pdf)

本地固定提交的 [Sequencing](../../megaeth_specs/specs/protocol/sequencing.md#sequencer-authority) 定义 `sequencerAt(block_number)`，并要求同一 EVM 区块内所有 mini-block 使用同一授权 key。对应的 [SequencerRegistry 页](../../megaeth_specs/specs/system-contracts/sequencer-registry.md)仍是占位页。公开开发文档也描述了 registry，但**存在“当前 key 的查询接口”不能证明“公开 stake election 已经上线”**。[公开 System Contracts](https://docs.megaeth.com/developer-docs/overview-3/system-contracts#sequencer-registry)

当前主网连接文档明确 gas 币为 **ETH**，并说明 EIP-1559 base-fee 自动调整实际上被关闭。MiCA 中较早的动态费市描述不能覆盖这个更新的开发文档。费用的最终 vault recipient、基金会／运营者分账比例，本次没有核实；不应补造按 MEGA stake 自动分手续费的规则。[当前主网参数](https://docs.megaeth.com/user-guide/connect)

### 6.2 MEGA 已发行，但不同用途有不同进度

官方公告确认 **2026-04-30 MEGA 上线**。它介绍了 Flux 中的 KPI commitment 奖励，同时将 proximity markets 与后续 rotation 描述为未来层次。公开发行代币、允许锁币参与奖励，以及允许锁币成为排序器，是三件独立的事。[MEGA is Live](https://www.megaeth.com/blog-news/mega-is-live)

| 机制 | 参与者付出什么 | 获得什么 | 核对到的状态 |
| --- | --- | --- | --- |
| Flux / KPI commitment | 持币者锁定 MEGA，积累 maturity | 网络达到经认定的 KPI 后，获得预留代币奖励资格 | 已有公开产品及公告 |
| Proximity Markets | 交易方竞拍席位、锁定 MEGA，或两者结合 | 靠近 sequencer 的低延迟入口与数据服务 | 官网标为 Planned |
| Sequencer Rotation | 运营者质押 MEGA，并满足性能要求 | 被选中承担排序职责；拟支持 delegation 与 slashing | 官网标为 Planned |

官网所述 rotation 基于 stake 和 performance，proximity 则销售低延迟访问；**购买更近的交易入口，不等于买下 sequencer 的出块资格。** 本次未找到已公布的完整选主函数、最低 stake、绩效测量规则和 slash 证据规范。[Token 页面](https://www.megaeth.com/token)

### 6.3 USDm 与 MEGA 的资金联系

2025-09 的 USDm 设计说明提出以储备收益支持 sequencer OPEX，从而支持较低、按成本定价的 gas。2026-04 的代币发布公告和当前 token 页面进一步说明：**Foundation 收到的 USDm rewards 用于买回并积累 MEGA。**[USDm 设计说明](https://www.megaeth.com/blog-news/megaeth-introduces-usdm)、[MEGA 发布公告](https://www.megaeth.com/blog-news/mega-is-live)

需要保留会计边界：储备总收益、项目方实际收到的 rewards、运营支出和回购预算不必是同一个量。上述资料没有给出可以逐项对账的固定分账瀑布；不能直接拼成“全部利息同时 100% 付 OPEX、又 100% 买币”。同样，**buy back and accumulate 不等于 burn，也不自动创设固定现金分红。**

对激励研究，这引入了不同于单纯提高 gas margin 的收益来源：运营方希望应用使用与 USDm 规模增长。低交易费、稳定币余额增长、延迟服务销售和运营成本之间是否能持续平衡，仍需数量模型；项目公布这一方向，不等于已经证明任何规模下都可自给自足。

### 6.4 供给、KPI 奖励与治理边界

MiCA 披露总量 **100 亿 MEGA**。当前官网以约数展示以下分配，不能当作实时流通量：

| 分配 | 官网约数 |
| --- | ---: |
| KPI Rewards | 53% |
| Community Allocation | 15% |
| VC Allocation | 15% |
| Team and Advisors | 10% |
| Foundation / Eco Reserve | 7% |

官网说明整数有舍入和实际分配调整；不应把它与较早文件的小数精度硬拼成同一快照。[Token 分配](https://www.megaeth.com/token)、[MiCA 供给披露](https://static.megaeth.com/MiCA%20Crypto-Asset%20White%20Paper%20-%20MEGA%20Token.pdf)

KPI commitment 的奖励对象是 committed holders；不是按每个区块付给实际出块者。官网当前阶段由 Foundation 定义、认定 KPI，后续社区治理与 oracle 验证是路线图。奖励条款也保留相应认定与实施裁量，所以不能把它当作完全无治理信任的逐块付费合约。[KPI 机制](https://www.megaeth.com/token)、[Rewards 条款](https://www.megaeth.com/terms-of-use)

## 7. Challenger 与 Fault Proof 系统的激励

**本节的中心问题是：谁愿意持续检查，谁支付被制造出来的争议成本，以及攻击者能否靠增加争议拖垮诚实方？** “有 fault proof”和“有诚实且有资源的参与者持续运行”是两个不同条件。

范围是带保证金、棋钟和交互式反驳的 permissionless `FaultDisputeGame`／`SuperFaultDisputeGame` 家族。Super FDG 改变了上层声明与状态推进的对象，但延续相关争议结构；不能把这里的收益与时间规则直接套到所有 permissioned 或 ZK game。合约链接用于核对参考实现，不代表本次已审计某条链的实际 game type、bond 或 delay 参数。[FDG](https://specs.optimism.io/fault-proof/stage-one/fault-dispute-game.html)、[Super FDG](https://specs.optimism.io/fault-proof/stage-one/super-fault-dispute-game.html)

### 7.1 Challenger 并不天然是诚实方，也不只负责攻击

在一个 game 中，Challenger 与 Defender 表示反对或支持 root claim 的立场，不是好人与坏人的固定身份。同一诚实运营者需要挑战错误 root，也需要维护受到恶意反驳的正确 root。

| 职责 | 诚实参与者实际做什么 | 成本与资金需求 |
| --- | --- | --- |
| 监控 | 读取新 game 与 claims，核对声明 | 常驻服务、同步节点、数据获取与保存 |
| 计算 | 确定分歧，生成所需执行 trace／证明 | CPU、内存、存储与运行时间 |
| 回应 | 提交 attack／defend，必要时提供 preimages、执行 step | L1 gas；新增 claim 时缴纳 bond |
| 结算 | 自底向上 resolve claims，再 resolve game、领取资金 | 仍需发送交易、支付 gas，并等待资金可提取 |

正确 root 没有遭到错误 counterclaim 时，诚实 agent 可以不出手，但仍继续监控。收到一个“Challenge”动作并不自动获得奖励；也不应看到新 claim 就机械攻击。诚实策略根据 claim 的位置、祖先关系和计算结果选择必要回应，并避免重复已经存在的相同回应。[OP-Challenger](https://docs.optimism.io/op-stack/fault-proofs/challenger#fault-detection-responses)、[Honest Challenger 策略](https://specs.optimism.io/fault-proof/stage-one/honest-challenger-fdg.html)

正确 root 受到恶意挑战时，仍需有人及时维护。合约按已经提交的证据和时限裁决，不会替缺席的诚实方自动完成整段重算；所以“我知道自己的 root 正确”不是忽略挑战的理由。

### 7.2 钱从哪里来：重新分配 bonds，而不是按挑战次数发币

Factory 创建 game 时收取按 game type 配置的初始化 bond；随后 `attack`／`defend` 新增 claim 时，按目标位置要求追加 ETH bond。`step`、`resolve` 等动作也需要 gas，但不是每个动作都新增一份 claim bond。[Factory 接口](https://specs.optimism.io/fault-proof/stage-one/dispute-game-interface.html#disputegamefactory-interface)、[参考实现](https://github.com/ethereum-optimism/optimism/blob/develop/packages/contracts-bedrock/src/dispute/SuperFaultDisputeGame.sol)

正常结算按**局部 subgame**分配，不是整个 game 的总赢家拿走所有人的押金：

- 一条 claim 没有被成功反驳，其 bond 返还该 claim 的出资者。
- 被成功反驳的 claim，其 bond 归于结算后成立的、位置最靠左的反驳子 claim 的提交者。
- 在终端通过 `step` 成功反驳时，由相应的成功调用者取得被反驳 claim 的 bond。

**Leftmost 指 trace／tree 中的位置，不是最早发出交易的时间；每笔奖励也不是在所有旁观者之间平均分配。** 具体分配还依赖下文的 freeloader 处理。[Bond 分配](https://specs.optimism.io/fault-proof/stage-one/bond-incentives.html#subgame-resolution)

**局部教学例子，金额不是部署参数：**攻击者 A 的一条错误 claim 押了 0.4 ETH；H 为有效反驳押了 0.6 ETH。假设该局部争议最终按规则由 H 获奖，H 的本金完整返回，其他成本合计 0.1 ETH，那么 H 收回 1.0 ETH，但净收益是 $0.4-0.1=0.3$ ETH，不能把自己的 0.6 ETH 本金再算成收入。实际 game 可能有多条 claims，须逐条核算。

对选定的一段运营期间，可以使用以下**分析用账本**：

$$
\Pi_H^{\mathrm{FDG}}
=R_{\mathrm{external\ bonds}}-L_{\mathrm{own\ bonds}}
-C_{\mathrm{gas}}-C_{\mathrm{compute}}-C_{\mathrm{monitor}}-C_{\mathrm{capital}}.
$$

前两项分别是从其他经济主体获得的押金，以及自己最终未收回的押金；四项成本分别对应链上操作、计算、监控与锁资。取回自己的本金不算收入。同一控制者的地址间转账也不算外部奖励。这是核算框架，不是协议保证右侧必为正的定理。

### 7.3 为什么有人长期监控：一次获奖与持续参与不同

如果一段时间内全是正确提议，也没有恶意反驳，那么该机制不会因为某人“认真检查了一天”自动支付固定薪水。诚实 proposer 收回自己押金也不等于得到区块奖励；本节没有一条将 sequencer 交易费自动按比例分给所有 challenger 的规则。

因此，监控者可能还需要保护自有资产／业务的动机、运营方预算或安全服务合同。这里是在说明可能的资金来源，不是声称所有 OP 链都已有这些安排。[Bond 规范的 Other Incentives](https://specs.optimism.io/fault-proof/stage-one/bond-incentives.html#other-incentives)明确承认运行 agent 和资本机会成本，并未为它们单列奖励，而是假设总体 bond 收益足以覆盖。

这是一个参与问题：**没有现成争议时谁承担常驻成本，以及有争议时谁能赢得奖励，必须一起分析。** 多个监控者也未必都拿到同一条 claim 的奖励；已存在的有效回应可以使重复出手没有必要。因此不能用“每次正确挑战都有潜在奖励”直接推出足够多的独立监控者会持续存在。

### 7.4 “Serial／Sybil／Spam”分别攻击什么

这里把 Serial 理解为串行拖延；如果所指是 Sybil，多身份攻击则是下表另一行。

| 攻击 | 攻击者试图获得什么 | 已有约束 | 仍须考虑什么 |
| --- | --- | --- | --- |
| **Serial delay** | 反复回应或挑战，让同一争议不断获得完整新窗口 | 继承式棋钟、最大深度、受限 extension | 到期后的 resolve 仍需有人实际执行 |
| **Sybil** | 换大量地址，绕开去重、重置时钟或重复领钱 | 去重与路径时钟不按钱包身份计算；每个有效新增对象仍有成本 | 不同 claims／不同 games 仍然可以增加工作量 |
| **Branch spam** | 在一个 game 中放入许多不同反驳，消耗诚实方资金与计算 | 每条新增 claim 的 bond、gas、重复项拒绝 | 宽度不受最大深度直接限制 |
| **Game spam** | 连续创建许多不同假 root／game | 初始化 bond、gas、game 去重与有效性检查 | 新 game 有独立资源需求，不共享一个全局时钟 |
| **Self-challenge／freeloader** | 用自控地址回收本应支付给诚实回应者的 bond | 局部分配、leftmost 规则与完整诚实回应策略 | 押金不是每创建一次就永久烧掉 |

这些机制分别约束时间、对象重复和经济成本，不能合并成“OP 已经消除了 spam”的一个结论。后面逐项解释。

### 7.5 串行拖延：棋钟沿路径继承，换身份不会清零

新增 claim 的累计用时沿路径从其祖父 claim 继承，并计入等待回应的时间。这里的交替阵营由反驳关系决定，不按真实控制者或诚实身份分组。攻击者不能在每次回应时重新获得一整段初始预算，换地址也不会清除已有路径上的消耗。

**只说明原理的例子：**若某方沿该路径已经用掉 100 单位预算中的 80，后续不因提交新 move 就恢复到 100。真实规则还包含 clock extension：在剩余时间过短时保留必要回应时间，执行 trace 或 preimage 工作还可能适用特别扩展。扩展与有限深度及配置有关，不能描述成完全不加时。[Game Clock](https://specs.optimism.io/fault-proof/stage-one/fault-dispute-game.html#game-clock)

这个预算属于 **claim 路径的累计结构**：它既不是按攻击者真实身份计费，也不是其所有 sibling branches、所有 games 共同扣减的一只全局秒表。

此外，最大深度限制垂直反驳长度，终端由 VM step 处理；但时钟到期只使相应结算具备条件，不会让所有合约自动执行。仍需逐步处理子争议并发送 `resolveClaim`／`resolve` 交易。大量 children 可以分页结算，分页解决单笔交易可执行性，并不消除总工作量。[Honest Challenger 的 Resolution](https://specs.optimism.io/fault-proof/stage-one/honest-challenger-fdg.html#resolution)、[FDG 结算实现](https://github.com/ethereum-optimism/optimism/blob/develop/packages/contracts-bedrock/src/dispute/FaultDisputeGame.sol)

因此，本节不将“有限交互深度与棋钟”提升为对任意攻击规模都成立的固定墙钟完成上界。

### 7.6 Sybil 与 spam：哪些重复会被拒绝

Factory 用 `(gameType, rootClaim, extraData)` 识别一个 game，不把 creator 地址作为该唯一性的一部分。不同地址不能重复创建完全相同的这组参数。初始化对 extraData 等还有格式约束，不能假定可任意追加无意义字节绕过去重。[DisputeGameFactory](https://github.com/ethereum-optimism/optimism/blob/develop/packages/contracts-bedrock/src/dispute/DisputeGameFactory.sol)

同一 game 内，claim 的去重涉及 **claim 值、position、parent/challenge index**，而不是提交者地址。换钱包重发相同对象没有用；改变 claim 值或 parent，则可能是一个不同对象。**所以“同一高度只能有一个 game”“同一位置只能有一个 claim”都不是正确概括。**[Super FDG 的 initialize／move](https://github.com/ethereum-optimism/optimism/blob/develop/packages/contracts-bedrock/src/dispute/SuperFaultDisputeGame.sol)

经济阻力在于：有效创建的新 game／claim 要支付相应 bond 与 gas，资金还会被锁定。公开 bond 定价参考按深度增长，用固定的 gas 价格假设覆盖更深层回应工作；它是成本预算，不是按真实账单实时报销。[Bond 定价规范](https://specs.optimism.io/fault-proof/stage-one/bond-incentives.html#fault-proof-mainnet-incentives)

参数阅读应保留边界：该参考使用 200 gwei，并把 gas 成本预算从 400,000 扩展到最大深度处的 300,000,000；上端包括可能很重的 preimage 工作，不是说单个 move 必须消耗 3 亿 gas。规范中深度 73 的示例不代表所有部署；root 的初始化 bond 还要读取 Factory 配置。本文不把 0.08 ETH、某个总资金数字或某个固定天数写成所有链的通用要求。

### 7.7 连续开假 game，会不会不断重置提款计时

**仅仅新增一个不同 game，不会直接修改已有 game 的时钟或裁决结果。** Game 的起始 anchor 在创建时固定；后来的 anchor 更新也不改写正在进行的 game。要反驳一份已有正确声明，攻击者需在那个 game 自身允许的窗口内行动。[FDG Anchor State](https://specs.optimism.io/fault-proof/stage-one/fault-dispute-game.html#anchor-state)

提款也不是引用 Factory 中“最新的任意 game”。Portal 检查该提款证明绑定的具体 game，要求它满足系统认可、正确方向胜出及等待等条件；不要求所有其他 games 都先结束。[Portal 检查](https://specs.optimism.io/fault-proof/stage-one/optimism-portal.html#checkwithdrawal)、[Valid Claim](https://specs.optimism.io/fault-proof/stage-one/anchor-state-registry.html#valid-claim)

当前 Portal 参考实现还按 `(withdrawalHash, proofSubmitter)` 分别保存证明。第三人从另一地址再提交一份证明，不覆盖用户原有的好证明；finalize 可以指定使用哪个 submitter 的证明。用户自己重新 prove 则会更新自己的记录与计时。因此，“别人不断重 prove 就把你的好证明永远归零”不符合这套存储安排。[OptimismPortal2](https://github.com/ethereum-optimism/optimism/blob/develop/packages/contracts-bedrock/src/L1/OptimismPortal2.sol)

以上只排除了直接重置这类攻击。大量新争议仍可能占用链下资源、资本和 L1 blockspace，间接阻止必要回应；治理暂停等操作也属于另一层影响。

### 7.8 自己制造错误、自己挑战，能否刷奖励

**若所有 bonds 都来自同一个经济主体，且不计任何外部补贴或攻击收益，地址之间搬运 bonds 不会凭空创造利润。** 在正常资金守恒的合约路径下，控制者收回的合计资金至多来自自己原本的投入；再扣 gas、计算和锁资成本，不能获得正的纯自循环收益。这是按控制者联合账本计算的推论，不是把每个地址当成不同受害者。

但这不代表所有 self-challenge 都无害。规范专门定义了 **freeloader claims**：攻击者可能反驳自己的 claim，争取把相关 bond 收回，而不是让诚实方拿到。忽略某些这类动作，可以不改变整个 root 的正确裁决，却改变奖金归属。诚实策略因此还需要处理相关 freeloader claims。[Freeloader Claims](https://specs.optimism.io/fault-proof/stage-one/fault-dispute-game.html#freeloader-claims)

**“已做足够工作让假 root 失败”与“已做足够工作确保获得应得的局部 bond”不总是同一件事。** Claim 的语义也不只是它携带的哈希值，还包括其位置与所反驳的命题；不能仅以某个中间状态哈希恰好正确，就断言该 move 是诚实的。

### 7.9 诚实方的瓶颈：峰值资本，而不只是最终盈亏

即使预期获奖足以覆盖总成本，H 仍需先支付回应的 gas 和 bonds。未来尚未到账的奖金不能直接用于今天的下一步。可以用下式记录其峰值押金占用：

$$
K_H^{\mathrm{locked}}
=\max_t\sum_{c\in\mathcal L_H(t)}b_c,
$$

其中 $\mathcal L_H(t)$ 是时刻 $t$ 由 H 出资、其本金仍被锁定或等待提取的 claims，$b_c$ 是各自押金；已确定且不可追回的押金损失不再计入该集合，而计入损失项。这个式子只度量锁资，实际资金安排还需覆盖 gas、运营储备和已实现损失的补充。多个分支／games 可以同时放大资金需求。

官方 Challenger 运行说明明确承认：攻击者可以通过大量 claims 让诚实方资金不足以继续回应。因此，安全需要足够资金、数据和计算能力的诚实参与者及时行动，不能把这个前提改写为“有 bond，所以这些参与者一定自然出现”。文档里的单路径金额例子也不是全系统资源上界。[Challenger 资金要求](https://docs.optimism.io/op-stack/fault-proofs/challenger#how-much-eth-do-you-need-to-challenge-in-a-game)

资金退出还存在额外步骤：game resolve 后达到 finalization 条件，确定分配并启动 `DelayedWETH` 解锁，再满足其 delay 后提取。规范建议值与某条链实际值需要分开；这段延迟让应急干预有时间，也增加双方的资本占用。[DelayedWETH](https://github.com/ethereum-optimism/optimism/blob/develop/packages/contracts-bedrock/src/dispute/DelayedWETH.sol)

正常模式按局部结果分配，特定失效情形下可能转为退回原出资的 Refund 模式。暂停本身不等于已经自动退款，当前参考实现会阻止相应 close 操作。黑名单、retirement、暂停和管理权限是应急机制，不能当作攻击者无论如何必被罚没的数学保证。[closeGame／claimCredit](https://github.com/ethereum-optimism/optimism/blob/develop/packages/contracts-bedrock/src/dispute/SuperFaultDisputeGame.sol)、[Registry 条件](https://specs.optimism.io/fault-proof/stage-one/anchor-state-registry.html)

### 7.10 能够得到的激励结论与尚需的假设

该设计给出三个互补约束：**正确计算能够用于反驳；时钟限制沿路径拖延；bonds 为必要回应提供有条件的经济来源。** 但以下三个问题没有因此自动消失：

1. **监控供给：**平静时期由谁承担固定成本，未来奖励能否覆盖进入和持续运行？
2. **资源不对称：**攻击者新增的工作量、诚实方需要押的资本和可获得奖励是否匹配，且能否在时限内完成？
3. **外部收益：**攻击者可能为了错误提款、市场头寸、贿赂或损害竞争者而愿意付出 gas 与押金损失。

对选定的攻击策略，在上述参与者账本中加入外部收益即可得到一个**分析用分解**：

$$
\Pi_A
=V_A^{\mathrm{external}}+R_A^{\mathrm{external\ bonds}}
-L_A^{\mathrm{own\ bonds}}-C_A^{\mathrm{gas}}
-C_A^{\mathrm{compute}}-C_A^{\mathrm{capital}}.
$$

这里不能把“累计提交的全部 bond”直接代入 $L_A^{\mathrm{own\ bonds}}$：部分本金可能返还或在自控地址间回流。要断言某类 spam 对风险中性的攻击者不划算，需要在明确的信息、资源和对手行为假设下证明其期望增量收益不为正；要断言系统可持续，还要检查诚实方参与与流动性条件。本文没有声称已经证明这些全局结论。

对于你的研究，建议把 **单个 game 内的拖延控制**、**跨 game／多分支的资源消耗**、**奖励归属攻击**和**长期监控的参与条件**分别建模。这些属于 fault-proof 服务的激励；与本篇前文中 sequencer 的交易费和 token 收益相联系，但不能用排序器收入替代 challenger 的独立收益与成本。

## 8. 这些收入与 token 机制是否足以保证诚实？

### 8.1 先区分支付、验证和惩罚

OP Stack 中 claim／move 的 ETH bond 用于状态争议的成本与激励。它不是所有 sequencer 共同缴纳的 OP 质押，也没有因此为审查、短暂停机或所有预确认违约提供统一罚没。即使同一运营方既排序又提交状态声明，也只能按具体 game 的规则扣其相关 bond。[Bond Incentives](https://specs.optimism.io/fault-proof/stage-one/bond-incentives.html)

| 机制 | 直接改善什么 | 不能单凭它推出什么 |
| --- | --- | --- |
| Fee 收入 | 有人愿意提供服务 | 对每笔交易都公平、及时服务 |
| 管理员授权与替换 | 控制服务资格、移除不合适运营者 | 每次偏离都有客观证明和自动罚款 |
| Fault proof / validity proof | 约束所编码的执行／派生命题 | 全部排序、审查、offline 都被覆盖 |
| 质押 | 形成参与门槛或可锁定资本 | 未定义证据和执行权限时能够有效 slash |
| 回购 | 财政资金与 token 购买需求建立联系 | 持币者取得出块权、现金分红或必然获利 |
| KPI 奖励 | 激励参与者关注网络指标与持币承诺 | 单个运营者为每项指标失败承担可归责处罚 |

### 8.2 一个研究用的偏离收益模型

以下是**候选分析框架，不是对三条链已证明的均衡结论**。给定运营者 $i$ 的某个偏离 $d$，可比较相对于履职的增量收益：

$$
\Delta\Pi_i(d)
=G_i(d)+K_i(d)
-q_i(d)L_i(d)
-\Delta V_i(d).
$$

- $G_i(d)$：偏离带来的额外收入，包括可归于该运营者的外部收益。
- $K_i(d)$：偷懒或不发布数据等行为节约的当期成本。
- $q_i(d)L_i(d)$：该类偏离可被识别、归责并实际处罚所形成的预期损失。
- $\Delta V_i(d)$：失去未来费用、服务资格、业务或相关资产价值的预期损失，已按一致口径折现。

要由此论证某项行为有激励相容性，需要明确可行偏离、信息结构、真实收入归属、处罚证据、执行概率和未来价值。仅说“有 token、有质押、有回购”没有给出这些条件。

例如，某种链下审查没有被编码为可证明、可罚没的违规，那么就不能把它的 $q_i(d)$ 偷设成 1。提高 stake 也不能单独解决这个缺口。可先建立签名接收凭证、纳入期限和状态依赖的免责条件，再讨论具体罚则；这些都属于新增设计，不应误写成现成 OP Stack 保证。

这里还要区分 **应急替换** 与 **罚没**：在无法辨别故障和恶意时，超时可以先触发服务恢复；是否没收抵押可以要求更强证据。两者不必共用同一个门槛。

## 9. 对 MegaETH sequencer rotation 研究最有用的待定义项

官网提供了 stake、performance、delegation、slashing 的方向，但下面这些仍需要完整规格才能分析：

| 待定义项 | 必须回答的问题 |
| --- | --- |
| 准入与选择 | 质押只决定候选资格，还是直接影响选中概率？性能由谁测量？是否有地区、硬件或委托上限？ |
| 被轮换的职责 | 只轮换 ordering，还是 execution、签名、DA 发布、状态提议一起轮换？ |
| 收款对象 | 用户 gas、proximity 收入、USDm 收益和代币奖励分别给哪一类主体？ |
| 正常与备用报酬 | 只给当前 active operator 付费，还是为 standby 的同步与就绪付费？ |
| 可归责义务 | 应保证纳入、顺序、执行结果、发布时间，还是交接时保留已承诺前缀？ |
| 处罚 | 哪种证据允许 slash？短时 offline 是立即罚没、少付服务费，还是只替换？谁裁决与申诉？ |
| 退出与延迟责任 | 解质押是否早于争议结束？跨轮换的迟到数据和错误声明由谁承担？ |
| 经济抗攻击 | 收入是否足够支持监控与备用节点？token 下跌、低利用率或外部贿赂时是否仍有履职动力？ |

如果采用 [第 03 篇](03_formal_model_and_handoff.md) 的排序与执行分离模型，应把两方分别作为有策略、成本和收入的主体，而非用一个总的“sequencer reward”掩盖责任边界。比如只奖励签名排序数，可能奖励无法执行或未发布的订单；只奖励最终执行数，又可能让正常 orderer 为 executor 故障承担全部收入损失。这些是需要机制处理的权衡。

## 10. 阅读与更新时的检查表

1. **先看当前资格，再看 token。** 当前 authorized signer／batcher 由谁配置？持币是否真的改变其选择？
2. **先看钱的路径，再看“价值捕获”。** 用户支付 → vault → recipient → 成本／分成 → treasury／回购；不要跳过中间账户。
3. **给事件加日期。** Base 2023 分成、OP 2026 四周排序实验、MegaETH Planned rotation，不应拼成一个现行系统。
4. **给每种 staking 写出获得的权利。** KPI 奖励、交易优先、低延迟入口、出块资格和挑战保证金分别建模。
5. **不把治理文件当部署审计。** 精确 recipient、运营合同、实时 supply、回购完成期次和 slashing 执行记录要另行核验。

推荐核验顺序：先读 [OP fee vaults](https://docs.optimism.io/op-stack/transactions/fee-vaults) 和 [SystemConfig](https://specs.optimism.io/protocol/system-config.html)，再读 [Base 升级记录](https://docs.base.org/upgrades/overview) 与 [2026 分成澄清](https://gov.optimism.io/t/base-the-superchain-and-governance-questions-that-merit-answers/10612/4)，最后对照 [MEGA token 页面](https://www.megaeth.com/token) 中仍标 Planned 的功能与本地固定版本的 [Sequencing 规格](../../megaeth_specs/specs/protocol/sequencing.md)。
