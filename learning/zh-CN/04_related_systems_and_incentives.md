# 相关排序系统：它们约束什么，又为哪些服务付费

[English original / 英文原文](../04_related_systems_and_incentives.md)

**资料查阅日期：2026 年 9 月 16 日。** 本文回答[会议记录](../../meetup_record/260916.md)中关于相关项目与激励的问题。公开文档均在该日期查阅。“文档称已上线”表示项目方将某项功能描述为已经上线；本次整理没有独立检查已部署合约的存储、验证者成员名单或交易执行记录。研究提案会单独标明。

最重要的一点是：**选择下一任排序器、就交易顺序达成一致、检查执行结果、保证数据可用，以及向运营者付费，是不同的机制**。一个项目可能只将其中一项去中心化，而在另一项上仍依赖受信任的运营者。MegaETH 的轮换设计需要逐项给出明确答案。

## 1. 先明确要将哪一种服务去中心化

在下面的比较中，我们把工作分为五类：

| 工作 | 产出 | 需要回答的问题 |
| --- | --- | --- |
| 排序 | 按顺序排列的交易字节序列 | 谁决定顺序，哪些参与者必须同意？ |
| 执行 | 状态变化和交易回执 | 谁计算结果，谁检查结果？ |
| 发布与数据可用性（DA） | 可以恢复的输入数据 | 其他节点能否重建这条链，并继续运行？ |
| 结算验证 | 被接受的状态承诺 | 什么机制能阻止错误结果成为权威结果？ |
| 权限交接 | 新的获授权出块者及其起始位置 | 如何避免同一出块区间同时由两个主体掌控？ |

这些是概念上的角色划分，并不要求运行五个独立程序。一个运营者可以承担多项工作。

[本地 MegaETH 排序规格](../../megaeth_specs/specs/protocol/sequencing.md)为每个 EVM 区块授权一个密钥；该 EVM 区块内的所有 mini-block 都使用这个密钥。规格要求存款交易和 L1-attributes 先于普通交易出现，但除此之外，并未约束普通交易的相对顺序。因此，如果提案要强制按到达顺序排序、公平排序，或遵循某种拍卖策略，就相当于加入了新要求。不能用“现有执行验证已经检查了这些”来为这类要求提供依据。

同一份本地文档也有意不规定排序器内部如何拆分。引入类似 OP 的内部接口或共享排序器架构，是一项工程设计提议，不能据此声称 MegaETH 当前的规范架构就是如此。

## 2. 总览比较

各项判断的详细来源及其适用边界见后文。

| 系统或机制 | 交接或达成一致的对象 | 约束机制 | 激励机制 | 证据状态 |
| --- | --- | --- | --- | --- |
| OP Stack 加 OP Conductor | 受信任的高可用集群中的活跃出块者 | 通过 Raft 协调；rollup 派生规则和执行规则仍是独立部分 | 取决于运营者的经济安排；Conductor 本身不定义无许可的奖励市场 | 有工具文档；文档明确表示不具备拜占庭容错能力 |
| Arbitrum BoLD | 对状态断言进行争议裁决 | 带保证金的挑战和链上争议解决 | 状态断言／挑战保证金 | 文档称已上线；不负责选举排序器 |
| Arbitrum Timeboost | 临时的快速通道使用权 | 拍卖合约，加上受信任排序器执行的排序策略 | 为优先权支付拍卖款项 | 文档称已上线；去中心化版本属于另一份独立规格 |
| Espresso / HotShot | 就有序数据承诺达成一致 | BFT 证书、DA 机制及 rollup 集成 | 文档描述了基于质押的参与机制，以及提议者／委托者奖励 | 文档描述了生产网络；具体版本和罚没机制需要谨慎区分 |
| Astria | 经过排序、但排序层不解释其 rollup 执行语义的数据 | CometBFT 共识、rollup 派生及向 Celestia 发布数据 | 排序费用；本次整理未确认生产环境中验证者经济机制的具体安排 | 已核对架构文档；未审计当前验证者和部署状态 |
| Taiko | L1 上的提议排序，以及链下的预确认轮值 | L1 合约、证明、快速路径授权和强制包含 | 交易费和证明报酬；计划引入由质押担保的预确认 | 当前文档将预确认描述为白名单机制；无许可质押阶段仍处于计划中 |

## 3. OP Stack：区分抗审查与出块者故障切换

### 3.1 Rollup 规则约束什么

在 OP 文档描述的交易流程中，排序、批次发布和状态承诺发布是不同职责。批次发布者（batcher）压缩交易数据并将其发布到 L1；状态提议者（proposer）发布 L2 状态的承诺。因此，在讨论“轮换排序器”是否也要轮换 batcher 和 proposer 时，必须保留这一区别。前者不会自动意味着后两者也随之轮换。参见 [Transaction flow](https://docs.optimism.io/op-stack/transactions/transaction-flow)。

OP 的派生过程将排序器批次与从 L1 获取的信息结合起来，其中包括存款。当排序窗口结束、仍没有可接受的批次时，派生过程可以构造一个空的排序器批次；但这并不会取消强制处理存款的义务。其抗审查路径是：用户通过 L1 提交交易，再由派生规则处理，而不是承诺每一笔通过私有 RPC 提交的交易都必须被包含。参见 [Derivation specification](https://specs.optimism.io/protocol/derivation.html) 和 [Rollup protocol overview](https://docs.optimism.io/op-stack/protocol/overview)。

**对 MegaETH 设计的启示：**需要定义两个服务层级。快速路径可以承诺，在活跃排序器正常工作的情况下，以较低延迟返回回执。备用路径则可以承诺，通过其他获授权主体或从 L1 派生的输入，最终将交易纳入。两者应分别规定延迟上界、费用和前提假设。即使备用路径能够保障用户最终获得服务，若它需要数分钟甚至数小时，也无法满足毫秒级交易服务的要求。

### 3.2 Conductor 提供什么

[OP Conductor](https://docs.optimism.io/chain-operators/tools/op-conductor)使用 Raft 进行领导者选举和 unsafe 区块协调，监控排序器健康状况，并控制出块。其文档明确假设参与者诚实，且不提供拜占庭容错能力。实现文档描述的流程是：先通过 Conductor 提交 unsafe payload，再通过 gossip 传播。这为防止多个出块者相互冲突提供了有用的运维保护。参见 [Conductor implementation documentation](https://pkg.go.dev/github.com/ethereum-optimism/optimism/op-conductor)。

**对设计的启示：**Conductor 为运维接管提供了有用参考：跟踪精确的链头、让接替节点完成同步，并在过渡期间控制发布。但在运营者彼此竞争、且可能作恶的场景中，其信任假设决定了它不足以单独承担全部约束职责。恶意运营者可以忽略本地的停止指令。因此，MegaETH 需要由接收方和规范派生规则共同执行权限边界，而不能只依赖编排软件。

### 3.3 OP 社区已经讨论过 epoch 轮换

2022 年的 Optimism 论坛帖子 [Pragmatic first step towards decentralizing sequencer](https://gov.optimism.io/t/pragmatic-first-step-towards-decentralizing-sequencer/2767)提出了治理白名单、按 epoch 轮换、备用出块者、可能的保证金安排，以及后续引入拍卖的设想。技术回复强调，应共享待处理交易，避免交易在切换期间丢失。这是一项历史提案，不能用作已部署的证据。

其中最相关的经验十分具体：保住链头，只完成了交接的一半。待处理请求、已经作出的承诺，以及尚未完成的数据发布工作，也都需要明确的责任主体。一笔刚好在轮换边界前被接受的交易，不应因为没有被复制给接替者而消失。

## 4. Arbitrum：不能混为一谈的三种机制

### 4.1 排序器 feed 与强制包含

Arbitrum 的排序器先发布快速 feed，之后再发布批次。Feed 提供暂定的排序信息；将数据发布到父链，以及结算由这些数据产生的状态，是另外两个不同的里程碑。延迟收件箱（delayed inbox）和强制包含机制，为审查或服务中断提供了绕过排序器的路径。当前文档还描述了会改变强制包含延迟的审查超时缓冲机制。具体的延迟参数必须从相关部署中读取。参见 [The Sequencer and Censorship Resistance](https://docs.arbitrum.io/how-arbitrum-works/deep-dives/sequencer)。

对 MegaETH 而言，可借鉴的模式是：建立一条规范输入通道，在首选出块者不再配合时仍然可用。尚待设计的问题是，这条通道应如何与未完成的轮换交互：它能否触发接管，能否为下一任出块者增加必须处理的输入，还是两者都可以？

### 4.2 BoLD 验证状态，而不是排序器的业务策略

[Overview of BoLD](https://docs.arbitrum.io/how-arbitrum-works/bold/gentle-introduction)于 2026 年 9 月 11 日更新，文档称 BoLD 已在 One、Nova 和 Sepolia 上启用。提供保证金的验证者提交状态主张，并对错误主张发起挑战。BoLD 处理拖延争议解决的攻击，并支持无许可验证。论文 [BoLD: Fast and Cheap Dispute Resolution](https://arxiv.org/abs/2404.10491)阐述了这一争议机制。

这并不意味着排序器也是无许可的，也不意味着 BoLD 会检查排序器的全部排序决策。例如，排序器收到 Alice 的交易后将其排除，却正确执行了 Bob 的交易。最终状态仍然可能满足状态转移规则。若要针对 Alice 的交易被遗漏提出挑战，协议还需要额外规定相应规则，并提供证据证明：Alice 的交易原本有权被纳入。

同样，缴纳保证金的状态断言者，与缴纳保证金的排序器，也未必是同一主体。对前者的虚假状态主张进行罚没，并不会自动惩罚后者未履行的回执承诺，除非系统明确把这两种责任联系起来。

### 4.3 Timeboost 出售的是优先权，而不是排序器本身

[How Timeboost works](https://docs.arbitrum.io/how-arbitrum-works/timeboost/gentle-introduction)同样于 2026 年 9 月 11 日更新，文档称 Timeboost 已在 One 和 Nova 上线。它通过密封投标的第二价格拍卖，分配一个轮次内的快速通道；文档中的默认参数是每轮 60 秒，普通提交有 200 毫秒的延迟。控制者得到的是时间优势，而不是排序器的所有权，也不是对全部交易排序的无限控制权。文档明确表示，系统信任排序器会执行规定的排序策略。

Offchain Labs 另外发布了一份 [Decentralized time boost sequencer specification](https://github.com/OffchainLabs/decentralized-timeboost-spec)。它描述了满足诚实成员阈值假设的委员会、证书、时钟假设，以及可选的阈值加密。应将其视为独立的设计文档；这份规格的存在，并不能证明已上线的链正在使用它。

**对设计的启示：**需要明确 MegaETH 拍卖的究竟是出块责任、某种交易排序特权，还是两者兼有。这些是不同的产品，风险也不同。只负责提交优先交易的获胜者，不一定需要维护完整执行状态；接管当前排序器完整职责的获胜者，则需要具备执行、同步、发布和交接能力。如果采用草图中“仅负责排序”的方案，它需要掌握已经达成一致的排序历史，并承担分配给它的发布职责，而执行准备情况仍由执行器负责。无论哪一种角色，拍卖收入本身都无法证明其已经做好履职准备。

## 5. Espresso：共享确认不等于共享执行

论文 [The Espresso Sequencing Network: HotShot Consensus, Tiramisu Data-Availability, and Builder-Exchange](https://eprint.iacr.org/2024/1189.pdf)区分了区块构建、对有序承诺达成一致，以及数据可用性。其共享排序层并不执行每个接入 rollup 的状态机。其中的 builder-exchange 问题，讨论的是相互不信任的参与者之间如何交换区块信息和支付承诺。

[Espresso 节点文档](https://docs.espressosys.com/network/concepts/the-espresso-network/internal-functionality/sequencer-node)同样将其描述为与具体 L2 无关的服务：有序数据由各 rollup 自己的执行器和证明器解释。因此，仅仅集成 Espresso，并不会自动验证 MegaETH 的执行结果、维持 MegaETH 的状态增量的连续性，或提供已经完成同步的接替执行服务器。

### 5.1 区分三种保证

1. **一致性：**在共识的故障假设下，达到法定数量的参与者已经对某个顺序作出承诺。
2. **可用性：**在 DA 机制的假设下，可以取回被承诺的数据。
3. **执行有效性：**从约定的先前状态出发，应用这些数据，确实会得到所声称的下一状态。

[Espresso 的 DA 流程](https://docs.espressosys.com/network/concepts/the-espresso-network/properties-of-hotshot/espresso-data-availability-layer/how-it-works)结合了委员会数据可用性、可验证信息分散和数据可取回性证书。文档还说明，一条链可以使用 Espresso 的确认，同时选择 Ethereum 或其他提供方来承担 DA。[轻客户端合约](https://docs.espressosys.com/network/concepts/the-espresso-network/internal-functionality/light-client)在 L1 上跟踪 HotShot 承诺；其共识证明不能被当作某个具体 rollup 的执行证明。

**对设计的启示：**如果 MegaETH 仅使用委员会为轮换检查点出具证书，就必须明确成员究竟在证明什么。“我看到了一个哈希”“我持有输入数据”和“我验证了得到的状态”，分别意味着不同的职责和成本。为其中一项出具的证书，不能在没有说明的情况下替代另外两项。

### 5.2 激励与版本边界

[Mainnet 0 发布页面](https://docs.espressosys.com/network/releases/mainnet-0)记录了 2024 年 10 月上线、采用固定运营者集合的生产版本。后续的 [Mainnet 1 节点说明](https://docs.espressosys.com/network/releases/mainnet-1/running-a-mainnet-1-node)描述了质押注册和由前 100 名组成的活跃集合。[奖励文档](https://docs.espressosys.com/network/concepts/the-espresso-network/internal-functionality/stake-table/staking-rewards-calculation)描述了提议者与委托者分享奖励、佣金，以及质押延迟生效等机制。

不能把各个版本的功能拼接起来，再声称这是某一个实际运行中的配置。尤其是，自动生成的 [StakeTable 合约文档](https://espresso-network.docs.espressosys.com/contracts/src/StakeTable.sol/contract.StakeTable.html)说明，其轻客户端引用目前尚未使用，是为未来的罚没机制预留的。因此，本次整理并未确认其存在已经上线的罚没机制。基于质押的投票、奖励分配和可实际执行的惩罚，是三个需要分别确认的事实。

对 MegaETH 而言，这启发了一种有用的拆分：为活跃出块者交付的服务付费，为备用基础设施的就绪能力付费，并且在定义好可验证证据之后，再定义惩罚。“要求质押”本身并不是一份完整的约束机制规格。

## 6. Astria：不执行 rollup 交易的排序服务实例

[The Astria Sequencing Layer](https://docs.astria.org/overview/components/the-astria-sequencing-layer)描述了采用轮换提议者的 CometBFT 共识。Astria 对 rollup 数据达成一致，并执行自身的原生操作，例如价值转移和验证者集合更新；它并不执行每一笔 rollup 交易。验证者在共识过程中收到数据，再由中继者（relayer）将数据发布到 Celestia。

[交易流程文档](https://docs.astria.org/overview/transaction-flow)将流程拆分为：提交 rollup 数据的 Composer、排序器网络、Relayer，以及向 rollup 执行节点提供数据的 Conductor。文档区分了排序器承诺与之后得到 Celestia 支持、供派生流程使用的承诺。其中的数值时序是开发配置示例，不应当被解读为当前生产服务保证。

[Composer 文档](https://docs.astria.org/overview/components/composer)解释了运营者如何代普通 rollup 用户支付排序费。更熟悉系统的用户可以直接提交，并表达对 bundle 的偏好。Composer 在本地按照到达顺序打包批次，并不等于提供了全局公平性保证。本次整理没有确认当前生产环境的验证者准入、质押分布、罚没规则或奖励分成；这些在本文中仍属未知。

**对设计的启示：**Astria 展示了这样一种可能：分布式排序服务可以承担比 MegaETH 活跃执行排序器更少的职责。这类服务可以告诉下一任运营者应执行什么，却不能替代该运营者执行时所需的状态。对 MegaETH，应分别测量获取有序日志所需的时间、重建状态所需的时间，以及开始提供有实际用途的回执所需的时间。领导者选举很快，并不意味着执行接管也很快。

还需要区分名称：Astria 的 Conductor 将排序／DA 与 rollup 引擎连接起来；OP Conductor 则管理多个排序器实例之间的高可用切换。虽然名称相同，它们解决的是不同问题。

## 7. Taiko 与 based rollup：可借鉴的轮换思路，以及部署阶段限制

最初的研究文章 [Based rollups—superpowers from L1 sequencing](https://ethresear.ch/t/based-rollups-superpowers-from-l1-sequencing/15016)描述了由 L1 提议者及其 builder／searcher 生态驱动的排序。这是一类架构，并不意味着所有已经部署、被称为“based”的系统，都具有相同的准入或预确认规则。

Taiko 当前的[协议概览](https://docs.taiko.xyz/protocol/overview)将 Unzen 列为当前分叉版本，并说明其使用 Shasta 的提议／检查点架构。文档描述了面向预确认白名单参与者的快速提议路径、强制包含，以及至少包含一个 ZK 证明的组合执行证明。Ethereum 对提议交易排序，而证明推动协议状态最终确定。提议被 L1 包含，与状态获得证明支持并最终确定，仍然是不同事件。

[预确认页面](https://docs.taiko.xyz/protocol/preconfirmations)描述了按 epoch 轮值的出块者：它先构建并通过 gossip 传播区块，再将提议提交到 L1。文档将白名单预确认标为当前机制，将无许可、由质押担保的参与机制标为计划中的机制。后面的一些生命周期描述讨论的是这一计划阶段，因此不能把它们全部当作当前部署事实。本文依据文档明确标注的部署阶段，而不假定路线图中的目标已经实现。

[经济机制页面](https://docs.taiko.xyz/protocol/economics)描述了提议者从交易费中获得收入，并支付 L1 发布和证明成本。它还指出，已上线的 Unzen 活性保证金参数为零。这提醒我们：设计中存在保证金机制，并不意味着生产环境中已经存在非零的经济威慑。

研究文章 [Based preconfirmations](https://ethresear.ch/t/based-preconfirmations/17353)讨论了签名承诺、不同的安全性与活性故障，以及有抵押品支持的惩罚；后续讨论还考虑了如何让承诺以 L1 状态为条件，以应对重组。这些是研究思路，不会自动成为 Taiko 或 Ethereum 的共识规则。

**对设计的启示：**MegaETH 的每份回执都应说明其承诺范围：所承诺的交易、执行结果（如有）、父区块／检查点、出块者所属 epoch、截止时间，以及对 L1 重组的处理方式。还需要确定，继任者是否必须履行前任尚未兑现的承诺。如果必须履行，继任者在接管前就必须接收相应数据与责任；否则，承诺应明确在一个定义清楚的边界处终止。

## 8. 为什么乐观争议机制和有效性证明都不是完整的约束体系

执行证明回答的是一个条件性问题：给定这些输入和这一先前状态，所得到的状态是否正确？乐观争议机制通过挑战和裁决回答同一类问题。它们的具体约束范围，取决于被验证的计算中包含了哪些规则。

这两类机制都不会自动回答以下问题：请求是否按时收到、运营者是否故意拖延、接任运营者是否已经就绪，以及普通交易是否被公平排序。出块者完全可以正确执行一个空区块，或者一个经过选择性填充的区块。它也可以完全停止出块，从而不留下任何可供挑战的错误状态转移。

这并不是说包含规则或排序规则无法验证。如果协议提供公开输入，并定义可检查的规则，这些性质就可以变得可验证，例如通过 L1 收件箱、签名的包含承诺、经过认证的回执，或明确规定的排序过程。相应的经济成本和通信成本，也就成为设计的一部分。

Kelkar、Zhang、Goldfeder 和 Juels 的论文 [Order-Fairness for Byzantine Consensus](https://eprint.iacr.org/2020/269.pdf)明确将排序公平性视为一致性和活性之外的额外性质。因此，“所有验证者都同意了这个区块”并不能证明领导者选择了公平的顺序。对 MegaETH，应先选择具体的排序策略，再尝试对违反该策略的行为施加惩罚。

## 9. MegaETH 轮换的激励设计检查表

以下建议来自上述比较，并不是在陈述 MegaETH 已有的机制。

### 9.1 为不同义务分别付费

一个承担全部职责的活跃出块者，需要支付执行、带宽、发布和人员成本。在所提议的职责拆分方案中，应将每项成本分配给负责的排序器、执行器或发布者，而不是全部算在排序器头上。备用执行器可能承担大量同步成本，却拿不到活跃出块期间的交易费；仅负责排序的备用节点，则具有不同的成本结构。如果只为活跃期间付费，运营者就可能出于理性选择，减少对自身就绪职责的投入，从而造成很长的接管延迟。

在具备证据的情况下，应分别为已交付区块、成功的 DA 发布、就绪能力和完成交接支付报酬。避免仅根据运营者自报的在线时长付费。就绪奖励需要依赖可观察的测试，并且测试必须对应运营者实际需要提供服务的状态；同时也要考虑维持尚未投入使用的备用容量所需的成本。

### 9.2 先规定证据，再选择保证金

| 被指控的违规行为 | 可能的证据 | 主要困难 |
| --- | --- | --- |
| 双重签名／作出相互矛盾的声明 | 同一已定义位置上的冲突签名 | 明确位置、epoch 及允许重组的条件 |
| 违反包含承诺 | 签名回执，以及截止时间之后的规范历史 | 明确交易准入条件、截止时间和例外 |
| 未经授权出块 | 签名，以及具有权威性的出块者排班表 | 保证所有验证方使用同一版本的排班表 |
| 未履行发布义务 | 公开的发布义务，以及已经过期的截止时间 | 区分运营者遗漏与外部基础设施不可用 |
| 就绪能力不足 | 未通过协议定义的挑战，或接管失败 | 防止挑战者人为制造失败 |
| 不公平排序 | 所选排序规则要求的证据 | 私有的本地到达时间本身无法确立全局顺序 |

如果一种偏离行为无法被证明，提高保证金也收效甚微。反过来，过于宽泛的故障规则可能在网络分区时惩罚诚实运营者，并迫使其退出市场。因此，应分别定义恢复条件与归责条件：从一个无法联系到的领导者切换出去，并不意味着已经立刻证明它应该被罚没。

### 9.3 分别分析盈利能力与偏离激励

可以从下面这条基本收支恒等式出发：

\[
\text{运营利润} = \text{费用和奖励} + \text{允许获取的 MEV}
 - \text{执行和就绪成本} - \text{DA/L1 成本}
 - \text{拍卖支付} - \text{资金成本}.
\]

诚实运营时利润为正，是参与条件。激励相容还要求：任何可行的偏离都不能提高预期效用。对于一个已经明确的偏离，应比较它带来的额外收入或节省的成本，与预期可执行惩罚以及失去的未来收益。这只是建模检查表，不是均衡定理：能否发现违规取决于证据渠道，能否实际收取惩罚取决于保证金和退出延迟，未来收益则取决于重新进入和身份规则。

拍卖支付用于分配一项权利；保证金用于担保一项义务。应将两者分开。还需要规定：如果前任出块者已经收取交易费，而继任者必须继续完成发布或证明，相关费用由谁承担？剩余成本若没有明确归属，就会在每次交接边界上产生策略性行为的空间。

## 10. 有针对性的阅读路线

可以按以下顺序阅读，以回答会议中的具体问题。标为“提案”或“论文”的条目提供的是设计思路与论证，并不是部署证据。

| 优先级 | 一手来源 | 类型 | 为 MegaETH 提取的重点 |
| --- | --- | --- | --- |
| 1 | [OP Derivation](https://specs.optimism.io/protocol/derivation.html) | 协议规格 | 规范输入规则与备用路径行为 |
| 2 | [OP Conductor](https://docs.optimism.io/chain-operators/tools/op-conductor) | 运维文档 | 链头交接、监控，以及诚实节点假设 |
| 3 | [Pragmatic first step towards decentralizing sequencer](https://gov.optimism.io/t/pragmatic-first-step-towards-decentralizing-sequencer/2767) | Optimism 论坛提案，2022 年 | epoch 轮换、备用出块者、待处理交易的连续性 |
| 4 | [BoLD: Fast and Cheap Dispute Resolution](https://arxiv.org/abs/2404.10491) | 研究论文，2024 年 | 状态争议的活性与所承担的经济风险 |
| 5 | [Decentralized time boost sequencer](https://github.com/OffchainLabs/decentralized-timeboost-spec) | 设计规格 | 委员会假设，以及优先权与排序职责的区别 |
| 6 | [Espresso sequencing-network paper](https://eprint.iacr.org/2024/1189.pdf) | 研究论文，2024 年 | 一致性、DA、执行与 builder exchange 的拆分 |
| 7 | [Astria Transaction Flow](https://docs.astria.org/overview/transaction-flow) | 架构文档 | 排序承诺与由 DA 支撑的派生之间的区别 |
| 8 | [Taiko Preconfirmations](https://docs.taiko.xyz/protocol/preconfirmations) | 标明部署阶段的文档 | 出块者轮值与提前发出的执行回执 |
| 9 | [Based preconfirmations](https://ethresear.ch/t/based-preconfirmations/17353) | Ethereum Research 提案，始于 2023 年 | 承诺范围、安全性／活性故障、L1 条件 |
| 10 | [Order-Fairness for Byzantine Consensus](https://eprint.iacr.org/2020/269.pdf) | CRYPTO 2020 论文 | 为什么排序公平性需要独立的定义与机制 |

在实现任何借鉴而来的机制之前，先为它写一页接口说明：输入、输出、签名方、验证方、截止时间、故障假设、备用路径、支付方式，以及违约证据。这个过程会帮助明确：哪些现有机制可以复用，MegaETH 轮换的哪些部分仍然需要新的协议设计。
