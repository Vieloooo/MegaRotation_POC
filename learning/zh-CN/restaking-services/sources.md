# 资料索引：Restaking、EigenDA 与 Sequencer Rotation

[学习入口](README.md) · [AVS](01_avs_architecture.md) · [EigenDA](02_eigenda_workflow.md) · [共识与轮换](03_consensus_and_rotation.md)

检索与核对日期：**2026-09-22**。以项目官方文档、公开代码和原始论文为依据。代码引用尽量固定到具体提交；文档中的产品定位、计划、参数示例与部署状态分别处理。下列阅读重点是本次 review 的提炼。

## 1. 按问题找资料

| 想解决的问题 | 先读 | 再读 |
| --- | --- | --- |
| AVS 到底提供什么，节点为什么参加？ | E01、E02 | E03、E08 |
| 某个服务实际能罚掉多少 stake？ | E04、E06 | E05、E07 |
| EigenDA 每一步由谁执行？ | D02、D03 | D04、D05–D09 |
| EigenDA 的 consensus 是什么？ | D01 | D10、P01 |
| 为什么有 DA 证书还不能确定唯一 stream？ | 本组第 2、3 篇 | P02、P03、P04 |
| Leader 能否连续生产多个 mini-block？ | P04 的 §6 | 本组第 3 篇第 4 节 |
| 怎样判断一个节点完成了服务？ | A01、A02 | D09、E06 |
| 数据不可用可以如何追责？ | D11 | D12 的 observability 与经济安全部分 |
| Restaked sequencing 有什么相近项目？ | A04、A05 | SQUAD 需进一步核实实现，不从名称推定协议 |

## 2. EigenLayer：成员、质押与服务责任

| 编号 | 一手资料 | 阅读重点与适用范围 |
| --- | --- | --- |
| E01 | [EigenLayer Overview](https://docs.eigencloud.xyz/eigenlayer/concepts/eigenlayer-overview) | 角色总览；当前 AVS 用语；理解 shared security 的组织方式 |
| E02 | [Understanding AVSs on EigenLayer](https://docs.eigencloud.xyz/eigenlayer/concepts/understanding-avs-on-eigenlayer) | Operator 承诺、验证、奖励、处罚如何形成服务流程；不要把提供接口等同于已完成具体业务设计 |
| E03 | [Operator Sets Overview](https://docs.eigencloud.xyz/eigenlayer/concepts/operator-sets/operator-sets-concept) | 一个 AVS 可划分多个职责集合；适合对照同地集群、数据服务与备用节点分工 |
| E04 | [Unique Stake](https://docs.eigencloud.xyz/eigenlayer/concepts/slashing/unique-stake) | 分配到特定 operator set 的可罚没份额；避免把总 TVL 当成每项服务的保证金 |
| E05 | [Allocation and Deallocation](https://docs.eigencloud.xyz/eigenlayer/concepts/operator-sets/allocation-deallocation) | Stake 生效、解除分配、注销与历史责任的时间边界 |
| E06 | [Slashing Overview](https://docs.eigencloud.xyz/eigenlayer/concepts/slashing/slashing-concept)；[Design Slashing](https://docs.eigencloud.xyz/eigenlayer/developers/howto/build/slashing/slashing-veto-committee-design) | 核心接口与 AVS 裁决流程的分工；特别注意罚没不必全部基于可在链上客观证明的事实 |
| E07 | [Duration Vaults](https://docs.eigencloud.xyz/eigenlayer/concepts/duration-vaults) | 固定时间段内的质押承诺；可对照“提前一个月排班”，但不提供 leader 抽签或 uptime 保证 |
| E08 | [RewardsCoordinator 合约说明](https://github.com/Layr-Labs/eigenlayer-contracts/blob/main/docs/core/RewardsCoordinator.md) | AVS 提交奖励及 operator-directed 分配；这是持续更新的实现文档，本组未冻结其部署版本 |

建议先读 E02–E05，再回到我们自己的架构，画出“谁的 stake 为谁的哪项义务负责”。不必一开始就逐行阅读合约。

## 3. EigenDA：架构、接口与安全性

| 编号 | 一手资料 | 推荐阅读位置／原因 |
| --- | --- | --- |
| D01 | [EigenDA Spec：Introduction](https://layr-labs.github.io/eigenda/)；[System Architecture](https://layr-labs.github.io/eigenda/protocol/architecture.html) | 解释为什么 DA 不需要自建排序链；后者含旧版 disperser 与 L1 bridging 路径，阅读时与 D03 对照 |
| D02 | [EigenDA V2: Core Architecture](https://www.eigenlabs.org/blog/eigenda-v2-core-architecture/) | 先读 Protocol Actors、Blob Lifecycle、Control/Data Planes；是工程介绍，性能与未来计划不能直接当成通用保证 |
| D03 | [EigenDA V2 Integration Spec](https://layr-labs.github.io/eigenda/integration/spec.html) | Proxy、disperser、relay、DA nodes、L1 inbox、rollup validator 的交互；页面标注更新于 2025-11-14 |
| D04 | [Rollup Payload Lifecycle](https://layr-labs.github.io/eigenda/integration/spec/2-rollup-payload-lifecycle.html) | Payload → blob → certificate → L1 引用 → retrieval 的全过程；帮助区分 rollup 数据与 EigenDA 数据结构 |
| D05 | [Common V2 Proto](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/api/proto/common/v2/common_v2.proto) | `BlobHeader`、`BlobCertificate`、`BatchHeader`、参考 L1 区块与 relay keys；注意部分注释仍含旧路径／TODO |
| D06 | [Disperser V2 Proto](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/api/proto/disperser/v2/disperser_v2.proto) | `DisperseBlob`、`GetBlobStatus`、`SignedBatch`；特别看 `COMPLETE` 不保证签名比例的注释 |
| D07 | [Relay Proto](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/api/proto/relay/relay.proto) | `GetBlob`、`GetChunks`、`GetValidatorChunks`；理解访问路径与分片分配的关系 |
| D08 | [Validator Node V2 Proto](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/api/proto/validator/node_v2.proto) | `StoreChunks` 在取得并验证分片后返回 batch header 签名；另有直接读取节点分片的 `GetChunks` |
| D09 | [EigenDACertVerificationLib](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/contracts/src/integrations/cert/libraries/EigenDACertVerificationLib.sol) | `checkDACert` 连接包含证明、安全参数、签名门槛与所需 quorums；适合理解消费者的接受条件 |
| D10 | [Security Model](https://docs.eigencloud.xyz/eigenda/core-concepts/security/security-model) | 区分编码正确性、分片分配、恢复门槛和证书的 BFT 假设；先读定义，暂时跳过证明 |
| D11 | [Security FAQs](https://docs.eigencloud.xyz/eigenda/core-concepts/security/security-FAQs) | 阅读 DA slashing、EIGEN token forking 和链上观察的限制；页面中的金额／阈值为其配置示例，不作为本资料的当前测量 |
| D12 | [EigenDA: The Hyperscale Verifiable Data Availability Layer](https://docs.eigencloud.xyz/assets/files/EigenDA_Whitepaper-c917fb56c146ebd146abfba3c52648a1.pdf) | Eigen Labs 白皮书，所读 PDF 为 35 页；先读 §1.1–1.4 的 storage/access/observability，再读 §5–6 的分析与经济安全；协议设计不等同于全部功能已部署 |
| D13 | [Payments](https://docs.eigencloud.xyz/eigenda/core-concepts/payments) | 按需与预约容量、计量者与限流；理解付费、接单、确认是不同事件，不使用其中的示例价格作报价 |

**代码版本：**D05–D09 固定到 `bfa451e1aaa5eb53ec250758054bbc3267e66208`，提交时间 2026-06-03。本次只读核对了上述接口和验证库，没有将其编译、部署或与生产合约逐一比对。

## 4. 其他 AVS：从实际工作流程理解 consensus 的范围

| 编号 | 一手资料 | 读什么／版本边界 |
| --- | --- | --- |
| A01 | [Incredible Squaring README](https://github.com/Layr-Labs/incredible-squaring-avs/blob/d922e8cc0cd1be6b77787bb75fa4460acec2411d/README.md)；[TaskManager](https://github.com/Layr-Labs/incredible-squaring-avs/blob/d922e8cc0cd1be6b77787bb75fa4460acec2411d/contracts/src/IncredibleSquaringTaskManager.sol) | 任务生成、operator 签名、聚合、挑战与 slasher 调用。提交 `d922e8cc`（2025-04-01），明确为测试示例；README 存在不同阶段文字，具体行为对照代码 |
| A02 | [Hourglass README](https://github.com/Layr-Labs/hourglass-monorepo/blob/ca1c88e06f30a95500bfddc1506cabf8a95770fa/README.md)；[仓库首页](https://github.com/Layr-Labs/hourglass-monorepo) | TaskMailbox → Aggregator → Executor/Performer → 结果证书。提交 `ca1c88e0`（2026-02-27）；仓库于 2026-03-02 归档，标明测试用途 |
| A03 | [MACH M2 README](https://github.com/alt-research2/mach-avs/blob/3da201ff5c04b47021448dde68c09de00f2a17f0/README.md)；[MachServiceManager](https://github.com/alt-research2/mach-avs/blob/3da201ff5c04b47021448dde68c09de00f2a17f0/contracts/src/core/MachServiceManager.sol) | Verifier 发现异常、operator 签名、aggregator 汇总、`confirmAlert`；另有 ZK 模式。提交 `3da201ff`（2026-06-09），不代表所有 MACH 部署都采用相同配置 |
| A04 | [AltLayer Overview](https://docs.altlayer.io/) | VITAL / MACH / SQUAD 的服务划分；用于理解产品定位，不据此补写 SQUAD 的 consensus 算法 |
| A05 | [In-house Decentralized Sequencers](https://docs.altlayer.io/core-features-of-altlayers-in-house-rollup-stack/decentralized-sequencers/) | Beacon Layer、stake 与随机选择的架构描述；属于另一组 stack 文档，不能直接作为 SQUAD 主网实现证据 |

## 5. 原始论文：补足数据可用性与排序之间的部分

| 编号 | 文献 | 先读什么 | 与本项目的关系 |
| --- | --- | --- | --- |
| P01 | Kamilla Nazirkhanova、Joachim Neu、David Tse，[Information Dispersal with Provable Retrievability for Rollups](https://arxiv.org/abs/2111.12323)，AFT 2022；预印本始于 2021 | 问题定义、Semi-AVID-PR 的目标、dispersal 与 retrieval 接口 | 对消费者所需性质作有针对性的取舍；不能直接等同于我们的 weak liveness |
| P02 | Lei Yang 等，[DispersedLedger: High-Throughput Byzantine Consensus on Variable Bandwidth Networks](https://www.usenix.org/conference/nsdi22/presentation/yang)，NSDI 2022 | 引言与系统流程，随后读 §3–4；同页有论文和演讲 | 区分数据传输与 commitment 排序；给出两者组合的具体研究方案 |
| P03 | Jeb Bearer 等，[The Espresso Sequencing Network: HotShot Consensus, Tiramisu Data-Availability, and Builder-Exchange](https://eprint.iacr.org/2024/1189)，2024 年预印本 | 总体架构、DA 与共识接口、builder-exchange 的问题定义 | 2024 年共享排序研究的比较对象；不直接代表当前 Espresso 产品接口 |
| P04 | Maofan Yin、Dahlia Malkhi、Michael K. Reiter、Guy Golan Gueta、Ittai Abraham，[HotStuff: BFT Consensus in the Lens of Blockchain](https://arxiv.org/abs/1803.05069)，PODC 2019；所读 arXiv v6 | §4 的基础协议、§6 的事件驱动协议与 Pacemaker | 了解投票、锁定、已有证书与 leader 接替；stable leader 可以跨多个高度持续服务 |

P01 的会议归属也可在[作者主页](https://sites.google.com/stanford.edu/kamilla-nazirkhanova/home)核对。上述论文在本资料中按模型与机制学习，没有把论文里的实验数值用作我们方案的性能预期。

Espresso 的[当前架构定位](https://docs.espressosys.com/network/learn/espresso-in-the-modular-stack.md)已明确区分自身的 settlement/finality 与应用自己的排序。这也是为什么这里将 P03 明确标注为历史论文，而不混用当前产品描述。

## 6. 本次需要保留的版本差异

| 容易混淆的表述 | 本次采用的处理方式 |
| --- | --- |
| 旧 EigenDA 需要 L1 bridging，V2 又说不需要 | 分开解释 disperser 的旧桥接路径，以及 rollup 自己仍可能发布到 L1 的证书引用 |
| 集成文档写 `CERTIFIED`，代码写 `COMPLETE` | 按接口版本描述；消费者始终检查实际签名与安全参数 |
| BlobCertificate 听起来已经是最终证明 | 区分内部数据结构、batch attestation 和消费者需要的完整验证信息 |
| V2 博客、FAQ 与概览展示不同阈值 | 不拼成一套默认参数；实际门槛由所用版本、quorum 与 verifier 配置确定 |
| 文档介绍 permissionless disperser 或 DAS | 区分架构能力、路线图与实现状态；本组没有声称所有生产入口均已无许可或所有观察机制均已上线 |
| EigenLayer 有 slash 接口，所以网络故障能被自动客观裁决 | 分开介绍执行罚没的能力和 AVS 决定罚没的依据 |
| 公开文档仍推荐 Hourglass，但仓库已经归档 | 保留它作结构学习材料，不作为本次选择当前开发框架的结论 |
| 同一页面混有旧阶段说明和新代码链接 | 固定具体代码提交；例如 Incredible Squaring 的历史 placeholder 文字不覆盖已核对的 slasher 调用 |

## 7. 建议的两轮阅读

**第一轮：建立系统画面。** 读本组第 1、2 篇，再按 E02 → D02 → D03 → D11 的顺序看原资料。重点回答谁接单、谁持有数据、谁签名、谁能取回、谁决定处罚。

**第二轮：为本周架构作选择。** 读本组第 3 篇，再读 P01 的问题定义、P04 的 §6、P02 的系统流程和 P03 的总体架构。最后回到需求文档，确定生产节奏、确认节奏、leader 轮换节奏与交接对象。
