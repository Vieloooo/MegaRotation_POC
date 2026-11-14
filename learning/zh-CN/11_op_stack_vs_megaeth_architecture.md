# OP Stack 与 MegaETH：架构、实时确认与高可用设计对比

整理与来源核对日期：2026 年 9 月 22 日。

[返回中文阅读索引](README.md) · [OP Stack 完整流程](06_op_stack_architecture_and_pipeline.md) · [Mini-block、Flashblocks 与 Subblocks](05_miniblocks_and_flashblocks.md) · [OP Conductor 高可用](10_op_conductor_ha_trust_and_incentives.md)

**MegaETH 延续了 OP Stack 的 rollup 框架，但将实时服务路径组织为“sequencer 持续执行 → mini-block → 状态与结果流 → read replicas”。**
OP Stack 的标准节点路径则围绕 rollup node、Execution Engine 和可独立派生、执行的完整区块展开；当前的 Subblocks 又允许它在封块前暴露执行进展。
因此，比较的重点是组件边界、谁执行、谁验证、传播什么，以及早期结果如何在故障切换后保留，不能只比较两个出块间隔。

## 阅读范围与证据

本文中的 **OP Stack 基线**指标准 rollup node / Engine API 架构，以 Ethereum DA 和交互式 fault proof 为比较实例；**Subblocks** 和 **OP Conductor**分别作为可选的预确认与 HA 扩展单独说明。
OP Stack 是可配置的软件栈，不能将这些模块写成每条 OP Stack 链都采用的唯一部署。

**MegaETH 当前设计**指本次查阅的官方公开文档，以及本地 `megaeth_specs` 固定提交 `59797b45b1f59b7c2c147e6f99be39399a1f2b44` 所规定的行为；区块、sequencing 和 state-sync 页标记为 `Rex5`。
这两种证据描述公开架构与规范，不等同于对主网全部进程、配置和故障恢复行为的实测。
OP Conductor 的代码比较固定在 `ad1c8e12f43c7e9158bb5da127b0153c3e28d4ad`，与第 10 篇一致；本次重新核对了完整 payload 提交、Raft Apply 和可选 Flashblocks 转发路径。

阅读顺序：

- [1. 架构总览](#overview)
- [2. Sequencer 与 CL/EL 边界](#boundaries)
- [3. 一笔交易的两条实时路径](#paths)
- [4. 节点分工与状态同步](#replicas)
- [5. 区块、签名与确认的含义](#commitments)
- [6. 10ms 到底来自哪里](#latency)
- [7. Raft 与故障切换：已知和未知](#ha)
- [8. DA、派生与 L1 结算](#settlement)
- [9. 对 sequencer rotation 的直接影响](#rotation)

<a id="overview"></a>

## 1. 架构总览

| 比较维度 | OP Stack 基线及相关扩展 | MegaETH 公开设计 / 本地规范 |
| --- | --- | --- |
| 正常出块权 | 通常一个 active sequencer；Conductor 可协调多套备用实例 | 一个被授权的 sequencer 角色；公开文档说明有 HA standby |
| 节点内部协议边界 | Rollup node 与 EL 通过 Engine API 交互 | 不规定 sequencer 内部的 CL/EL 协议边界 |
| 完整区块 | 标准 L2 EVM 区块；间隔按链配置 | 保留标准格式 EVM 区块，文档描述约 1s |
| 更细的实时输出 | 可启用 Subblocks，在构建期间流式发布；文档描述约 200ms 目标 | Mini-block 是原生 state-sync 单位，文档描述约 10ms |
| 快速跟随方式 | 普通验证节点执行完整块；Subblocks-aware 节点执行增量交易 | Read replica 验证来源后应用状态增量；full node 另做重执行 |
| 状态表示 | 标准执行状态与相应 trie / root 规则 | SALT 状态表示及其增量、承诺和 witness 格式 |
| HA 的已知发布条件 | 正常 Conductor 完整块路径：Raft commit 成功后才 gossip | 公开资料未交代每个 mini-block 发布前的 standby / quorum ACK 条件 |
| DA 比较实例 | Ethereum blobs / calldata；其他 DA 配置需另看 | 官方描述 EigenDA，证书经 batcher 提交 L1 |
| 争议比较实例 | 交互式 fault dispute game / Cannon 等相应实现 | 官方描述 Kailua / RISC Zero 的 ZK fault proof；本地细节页仍是 stub |

依据：[OP 组件与模块][op-components]、[节点组件规范][op-overview]、[Subblocks 架构][subblocks-doc]、[MegaETH 架构][mega-architecture]、[本地协议范围][mega-protocol]、[SALT][salt]。
表中的间隔是各资料描述的运行目标，不是所有交易、所有网络位置都能达到的端到端 SLA。

<a id="boundaries"></a>

## 2. Sequencer 与 CL/EL 边界

### 2.1 OP Stack：sequencer 是一套角色与组件，不只是一个排序函数

典型 OP Stack sequencer 包含 rollup node（例如 `op-node`）和 Execution Engine。
Rollup node 处理 L1 输入、派生与构建控制，EL 处理交易池、交易执行、区块构建和执行状态；二者通过 Engine API 交换 payload attributes、payload 和 forkchoice 等信息。
具体交易选择和排列可以发生在 EL / builder 中，不能直接将 `op-node` 等同于“只排序的服务”。
第三方验证节点同样具有 rollup node 与 EL，只是不承担活跃出块职责。[OP 节点组件][op-overview]、[执行引擎规范][op-engine]

这里的 consensus layer / rollup node 并不意味着全部 L2 节点逐块运行多数投票。
普通节点依据给定的 L1 历史、配置与状态独立派生和执行；Conductor 的 Raft 则是另一套运营集群内的协调机制。[派生规范][op-derivation]、[Conductor 第 10 篇](10_op_conductor_ha_trust_and_incentives.md#scope)

### 2.2 MegaETH：对外规定 sequencer 输出，内部拆分留给实现

本地规范将 sequencer 视为产生合法签名 mini-block 和 EVM 区块的角色。
它规定签名者、块结构、必要交易的位置和接收端检查，但不规定内部必须部署几个进程、是否使用 Engine API，或排序与执行之间如何调用。
**“不规定 CL/EL 边界”不能推出“实现里绝无 rollup node”，也不能推出“排序和执行已经由不同运营者承担”。**[Sequencing][mega-sequencing]

当前 mini-block 包含执行后的回执承诺和 gas 使用量，因此是带执行结果的输出。
如果我们的轮换方案另设一个只决定交易顺序、在执行前签名的 orderer，那是新的角色和接口：还需要定义执行方如何接受该顺序，以及最终结果如何对应它。[Mini-block 签名][mega-blocks]、[拟议模型第 03 篇](03_formal_model_and_handoff.md)

<a id="paths"></a>

## 3. 一笔交易的两条实时路径

### 3.1 OP Stack：完整块传播与可选 Subblocks 流

按是否启用扩展，用户可能通过不同路径先看到结果：

| 阶段 | 完整区块路径 | 启用 Subblocks 后的较早路径 |
| --- | --- | --- |
| 选择与执行 | EL / builder 在 rollup node 驱动下构建区块 | 同一次构建中逐步产生已执行的交易前缀 |
| 发布关口 | 封块；启用 Conductor 时先提交完整 payload 到 Raft | 执行客户端在封块前发布 WebSocket 增量流 |
| 网络消费 | 接收节点处理完整 payload，独立执行验证 | Subblocks-aware RPC 节点执行流中的交易，更新 pending 视图 |
| 应用读到什么 | 完整 unsafe 区块及其结果 | 未封块构建的结果；构建被放弃时可以撤销 |

当前 Subblocks 由执行客户端原生发布，`op-node` 仍经 Engine API 驱动构建；proxy 可转发流，Conductor 可管理 leader。
这与旧 Flashblocks 通过 Rollup-Boost / 外部 builder 集成的拓扑应分别解释。[Subblocks 架构][subblocks-doc]、[消费者规则][subblocks-spec]

**完整块的 Raft 提交发生在封块后，所以仅凭这个关口不能保护此前已经发出的全部 Subblocks。**
它们是否另有更早的复制机制，必须看该流的实现与恢复协议；第 7 节进一步区分这两种保证。

### 3.2 MegaETH：执行结果按 mini-block 流式输出

官方描述的正常流程为：

1. RPC 入口接收交易并转发给 sequencer。
2. Sequencer 持续执行，每约 10ms 将最近执行的交易封装为 mini-block。
3. 交易、回执和状态变化流向 RPC / replica 节点，接收端完成相应检查与应用后对外提供结果。
4. 约每 1s 封装一个包含这些 mini-block 的完整 EVM 区块；后续再进入 DA 与 L1 发布、结算流程。

上述顺序描述客户端可观察的数据路径；官方架构图没有展开的内部 HA 复制步骤不能据此当作不存在。[MegaETH 架构][mega-architecture]

两者都可以在完整区块之前返回执行结果。
更有辨识度的差异在接收侧：OP Subblocks-aware RPC 通过执行交易得到 pending 状态；MegaETH read replica 通过应用 sequencer 提供的状态变化更新状态。[Subblocks JSON-RPC][subblocks-spec]、[State Sync 信任模型][state-sync]

<a id="replicas"></a>

## 4. 节点分工与状态同步

### 4.1 “Replica”不能按名字一一对应

| 角色 | 主要接收 / 保存什么 | 是否独立执行验证 | 能否据此推定可接管出块 |
| --- | --- | --- | --- |
| OP 普通验证节点 | 区块、L1 派生输入、执行状态 | 是 | 不能；还需要出块授权和运营配置 |
| OP Conductor replica | Raft 日志、最新 unsafe payload 与协调状态 | Raft 本身不认证 EVM 执行正确 | 不能；配套 EL 还要同步到正确 head |
| MegaETH read replica | 已认证的交易、回执、状态变化 | 快速 state-sync 路径不重执行 | 不能；服务读取与接管出块是不同能力 |
| MegaETH full node / prover | 用于重执行或证明的输入 | 承担相应验证任务 | 不能仅凭能验证就推定有签名权 |
| MegaETH HA standby | 公开文档确认存在备用接管角色 | 内部配置与同步条件未完整公开 | 是否已能接管取决于这些未公开条件 |

OP 来源见[节点规范][op-overview]和[Conductor Raft 实现][op-raft]；MegaETH 的公开角色划分见[架构文档][mega-architecture]。
**地理分布很广的 read replicas，不等于一个对每次出块投票的 HA quorum。**

### 4.2 MegaETH state-sync 是一套独立的协议

本地规范定义四条经过认证的流：每个 mini-block 对应 `Transaction`、`Receipt`、`StateChange` 三类 fragment，每个完整 EVM 区块另有 `Header`。
State-change fragment 带 SALT 增量等内容；协议还处理缺口补取、reorg 和状态快照引导。
它不是“把标准交易广播改快一些”，也不是只发一个回执就完成状态同步。[Streams and Payloads][state-sync]

SALT 用于 MegaETH 的账户与存储状态表示，影响状态增量、state root 和 witness 的构造。
它与 mini-block 的交易根、回执根不是同一类数据结构：后两者仍按其各自规定的有序 MPT 构造。[SALT 概览][salt]、[Mini-block Body][mega-blocks]

State-sync 验证的是数据确实来自授权 sequencer，以及消息满足相应结构与连续性规则；这些检查本身不证明 sequencer 执行正确。
独立执行验证与证明属于另外的路径。
这种分工降低了实时 RPC 副本的执行负担，也使“迅速跟随状态”与“独立确信状态正确”成为不同时间点。[Trust Model][state-sync]

<a id="commitments"></a>

## 5. 区块、签名与确认的含义

### 5.1 同样是部分区块，承诺对象也可能不同

| 对象 | 表达什么 | 不能单独说明什么 |
| --- | --- | --- |
| 完整 OP unsafe payload | 已构建的完整区块及执行输出 | 尚未因此成为 L1 派生的 safe / finalized 历史 |
| 当前 OP Subblock | 一个构建中的交易增量与累计执行进展 | 流中的零 `state_root`、零 `block_hash` 不能作为状态或最终区块承诺 |
| MegaETH mini-block header | 位置、时间、gas、该 mini-block 的交易根与回执根 | 没有 state root，也没有前一个 mini-block 的哈希链接 |
| MegaETH mini-block signature | Rex5 起，授权 sequencer 对 header 内容的可验证签名承诺 | 不证明 quorum 持久化、独立执行正确、L1 最终性或自动赔付 |

MegaETH 的完整 EVM 区块按 mini-block 顺序拼接交易，并针对完整列表计算区块级交易根和回执根。
**标准区块格式兼容，不代表内部状态结构、实时传播协议和确认条件与 OP Stack 完全相同。**[Blocks and Mini-Blocks][mega-blocks]、[Subblocks payload][subblocks-spec]

### 5.2 “确认”至少要拆成四个判断

| 判断 | 需要的证据 / 条件 |
| --- | --- |
| Sequencer 已执行并声明某个结果 | 对应交易的结果，以及适用的认证或签名 |
| 换机器后仍能保留这个结果 | 接管协议要求继承的前缀，以及可恢复的字节与状态 |
| 结果符合执行规则 | 独立重执行或相应有效证明 |
| 历史和结算达到所需最终性 | DA、派生、L1 最终性以及相应桥接 / 争议条件 |

官方将 MegaETH mini-block 结果称为 preconfirmation，并说明其具有与该链完整 EVM 区块相同的 sequencer 预确认保证。[Realtime API][mega-realtime]
可以把它放在 unsafe 层级理解，但 mini-block 不是标准 OP 节点里一个已经完整封装的 unsafe EVM 区块。
签名让冲突承诺可以被归责；它本身不会使所有副本都持有数据，也不会自动执行罚没。[Mini-block 签名及安全说明][mega-blocks]

<a id="latency"></a>

## 6. 10ms 到底来自哪里

MegaETH 的实时架构将持续执行、细粒度封装与结果传播衔接起来，read replica 又省去了实时跟随中的交易重执行。
Mini-block 无须等待完整 EVM header 和区块级 state root 对外可用，就能提供这一层结果；这并不表示系统永远不计算完整区块承诺。[区块格式][mega-blocks]、[状态同步][state-sync]

分析一笔交易的用户端延迟时，可以按关键路径粗略分解：

```text
用户看到结果的时间
≈ 上行网络 + 排队与执行 + 等待本次封装
  + 发布前必须完成的内部步骤 + 下行传播与 RPC 处理
```

这是分析框架，不是声称这些实现步骤完全串行。
公开文档给出的约 10ms cadence / 结果输出描述，没有逐项给出这份延迟预算，尤其没有披露其中 HA 等待项的完成条件。[Mini-block 说明][mega-mini-doc]、[Realtime API][mega-realtime]

因此需要分别测量：**产出间隔、sequencer 处理时间、用户端预确认延迟、quorum 持久化延迟、故障恢复时间**。
仅每 10ms 产出一项，可以通过流水线实现；即使某项的复制耗时为 100ms，也不妨碍吞吐达到每秒 100 项，但它不能据此获得 10ms 的多数复制确认。
这个例子说明指标的区别，不是对 MegaETH 实测延迟的陈述。

反过来，Raft 本身也不与 10ms 必然矛盾：若 quorum 的网络与持久化足够快，它可能适合该预算；若发布前必须等待的 quorum 往返已经超过预算，就需要改变部署、发布条件或目标。
“只有一个 active sequencer”无法单独回答这个问题，因为 OP Conductor 的 Raft 同样只有一个 active leader。[Raft Apply 的完成条件][raft-apply]

<a id="ha"></a>

## 7. Raft 与故障切换：已知和未知

### 7.1 OP Conductor：完整 payload 的先提交后广播

在启用 Conductor、未使用灾备 bypass 的正常完整块路径中，固定版本代码的顺序是：

1. 构建并 seal，得到完整 `ExecutionPayloadEnvelope`。
2. `CommitUnsafePayload` 序列化 envelope，调用 Raft `Apply` 并等待完成。
3. 成功后才调用异步 gossip；失败则提前返回。

Raft 完成需要 quorum 持久化该日志并完成相应 leader FSM 应用。
三 voter 配置需要 leader 加至少一个 follower；不要求多数配套 EL 已执行该区块，也不等于整个状态数据库已被 Raft 复制。[Sequencer 固定源码][op-sequencer]、[Raft 固定源码][op-raft]、[Apply 语义][raft-apply]

故障接任还需要新实例的执行 head 与继承目标相符。
Conductor 在其 crash-fault 假设内保护完整块的接续；它不因此提供 Byzantine consensus、L1 最终性，或对所有更早信号的永久保留保证。[第 10 篇：故障切换](10_op_conductor_ha_trust_and_incentives.md#failover)

### 7.2 OP 的部分区块也要单独分析

当前 Subblocks 规范允许放弃进行中的构建，新构建从 index 0 开始；消费者不能把旧构建的部分结果当成 safe / final 数据。[Consumer guarantees][subblocks-spec]
本次核对的可选旧 Flashblocks WebSocket handler 在转发处检查 leader 后广播，没有为每条消息调用 `CommitUnsafePayload`。
这只证明该 handler 的行为，不能替代对任意当前 Subblocks 部署的完整审计。[固定转发源码][op-flash-handler]

因此，“OP 开启 Conductor”也不足以推出“用户看到的每一份预确认都已经 quorum commit”。
必须先说明用户收到的是完整块，还是构建中的增量消息。

### 7.3 MegaETH：存在 HA，但无法从现有材料补出其复制屏障

公开架构页描述 standby 可在 active sequencer 故障后接管；本地规范则明确将 HA 机制、内部部署与传播拓扑留给实现。[官方架构][mega-architecture]、[Normativity Boundary][mega-protocol]

| 需要核实的事项 | 当前证据能支持的结论 |
| --- | --- |
| 是否有 HA standby | 官方文档明确描述存在 |
| 内部是否使用原版 Conductor、定制 Raft 或其他机制 | 本次材料未建立结论 |
| 每个 mini-block 发出前是否等待副本持久化 / quorum ACK | 未公开明确完成条件 |
| 复制粒度是交易、mini-block、执行状态还是完整 EVM 块 | 未公开完整恢复协议 |
| 接任者如何获得最后一段已签名但尚未封块的前缀 | 规范没有给出可据此实现的 HA 算法 |
| 如何阻止失联旧实例继续签发输出 | 签名授权规则可查；物理实例的隔离机制未完整公开 |

这里的未知范围是 **HA 实现**，不影响我们比较已经写明的 mini-block 格式和 state-sync 行为。
也不能将缺少资料改写成“MegaETH 不做复制”或“已确认 mini-block 必然在切换时丢失”。

<a id="settlement"></a>

## 8. DA、派生与 L1 结算

两者都将快速执行结果与后续可重建、可结算的历史分开。
在本文选取的 OP Ethereum-DA 基线中，batcher 编码并压缩交易输入，将数据放入 L1 blobs / calldata；节点读取这些输入，结合 deposits、L1 attributes 和配置派生 L2 历史。
Batch 输入不是 sequencer 签发回执的简单汇总，节点仍需按执行规则计算或核对输出。[Batcher][op-batcher]、[Derivation][op-derivation]

MegaETH 官方架构描述使用 EigenDA 保存相应数据，再由 batcher 将 DA 证书提交到 L1。
这把数据检索与可用性依赖扩展到 EigenDA；有 L1 上的证书记录不能直接等同于全部原始数据存储在 Ethereum 上。
精确证书校验、不可用时的处理和派生差异，本地 [DA][mega-da]、[Derivation][mega-derivation] 与 [Batch Submission][mega-batching] 页仍未给出完整规范，本文不补造这些规则。[公开架构][mega-architecture]

证明路径也应单独比较：第 09 篇介绍的 OP 交互式争议通过细化分歧直到相应 VM 裁决；MegaETH 官方资料描述 Kailua 使用 ZK fault proof 处理争议。
**在争议系统中使用 ZK proof，不等于每次 10ms 预确认都等待 ZK proof，也不自动将该链改成每个状态更新都必须先交 validity proof 的结算模式。**
Kailua 自身也区分 hybrid / ZK fraud proof 与 validity 两类模式；不能仅凭使用 Kailua 的名称推断具体部署模式。[OP 争议背景](09_fault_proof_bisection_and_cannon_vm.md)、[MegaETH 公开说明][mega-architecture]、[Kailua 模式说明][kailua]、[本地 Fault Proofs 页][mega-faults]

还要保持两个时间点的区别：L1 输入及派生历史的最终性，与桥接使用某个状态声明所需的 dispute / withdrawal 条件。
不能因为退出需要等待相应争议条件，就把整段等待都算作普通交易的首次 confirmation time。[第 08 篇](08_op_stack_bridge_and_disputes.md)

<a id="rotation"></a>

## 9. 对 sequencer rotation 的直接影响

架构比较落实到轮换设计时，需要先确定三种不同边界：

| 轮换的东西 | 需要解决的事 | 已有事实提供了什么 |
| --- | --- | --- |
| 同一授权角色内部换物理实例 | 数据与执行状态接续、隔离旧实例、路由恢复 | OP Conductor 给出完整块参考；MegaETH 内部 HA 尚需核实 |
| 改变被授权的签名密钥 / 运营者 | 新权限何时生效、旧权限何时终止 | MegaETH 规范要求同一 EVM 区块内使用同一授权 key |
| 新增独立 orderer 并轮换它 | 执行前顺序承诺、无效输入处理、执行结果映射 | 当前 mini-block 签名在执行后作出，不能直接替代新接口 |

同一 EVM 区块内的签名 key 不变，不意味着整个区块必须由同一台物理机器完成；如果备用机器使用同一授权 key，仍须另外证明它继承了正确状态和已发布前缀。
若接任者改用另一把 key，则必须满足区块边界上的授权变更规则。[Sequencer Authority][mega-sequencing]

可以用一个具体故障检验方案：完整 EVM 区块 `B` 还未封块，mini-block `m0…m7` 已发给用户，此时 active sequencer 崩溃。
一套声称保留这些预确认的恢复协议必须明确：

1. **保留哪段前缀：**仅复制完成的部分，还是所有已对外签名的部分；发布规则必须与这个边界一致。
2. **谁拥有数据：**接任者从哪里取得交易、回执、签名和足以继续执行的状态；只知道最后一个完整块 hash 不够。
3. **谁继续签名：**同 key 换机器还是新 key 接任，以及旧实例恢复后如何失去输出能力。
4. **何时回复用户：**若承诺覆盖单机崩溃，需要在回复前完成哪些持久化或复制步骤，并将其计入延迟预算。
5. **谁接续发布：**未封块内容完成后，DA 和 batcher 的待发布任务如何保留，避免实时恢复却长期无法结算。

这些是拟议方案应回答的设计要求，不是对 MegaETH 已部署恢复算法的陈述。
可以沿用 OP Conductor“先确定可继承记录，再对外发布”的思路，但若要保护 10ms mini-block 预确认，就必须将记录粒度、发布条件与中途恢复规则明确到 mini-block 或相应交易前缀层面。

## 验证范围

- 核对了本地规范快照、官方架构与实时接口文档，以及上文固定版本的三处 OP 源码路径。
- 本地 rollup、stateless validation 等未完成页面只用于标明资料缺口，不作为某项机制已经实现的证据。
- 本篇没有部署集群、测量主网 10ms 延迟、抓取真实 HA ACK，或执行故障注入；公开性能描述与源码顺序不替代这些验证。

[返回中文阅读索引](README.md)

[op-components]: https://docs.optimism.io/op-stack/protocol/components
[op-overview]: https://specs.optimism.io/protocol/overview.html#l2-node-components
[op-engine]: https://specs.optimism.io/protocol/exec-engine.html
[op-derivation]: https://specs.optimism.io/protocol/derivation.html
[op-batcher]: https://specs.optimism.io/protocol/batcher.html
[subblocks-doc]: https://docs.optimism.io/op-stack/features/subblocks
[subblocks-spec]: https://specs.optimism.io/protocol/subblocks.html
[op-sequencer]: https://github.com/ethereum-optimism/optimism/blob/ad1c8e12f43c7e9158bb5da127b0153c3e28d4ad/op-node/rollup/sequencing/sequencer.go#L475-L492
[op-raft]: https://github.com/ethereum-optimism/optimism/blob/ad1c8e12f43c7e9158bb5da127b0153c3e28d4ad/op-conductor/consensus/raft.go#L305-L320
[op-flash-handler]: https://github.com/ethereum-optimism/optimism/blob/ad1c8e12f43c7e9158bb5da127b0153c3e28d4ad/op-conductor/rpc/ws/flashblocks_handler.go#L263-L273
[raft-apply]: https://github.com/hashicorp/raft/blob/main/docs/apply.md
[kailua]: https://github.com/boundless-xyz/kailua#how-does-op-kailua-work
[mega-architecture]: https://docs.megaeth.com/architecture
[mega-mini-doc]: https://docs.megaeth.com/mini-block
[mega-realtime]: https://docs.megaeth.com/developer-docs/overview-2/realtime-api
[mega-protocol]: ../../megaeth_specs/specs/protocol/overview.md#normativity-boundary
[mega-sequencing]: ../../megaeth_specs/specs/protocol/sequencing.md
[mega-blocks]: ../../megaeth_specs/specs/protocol/blocks.md
[state-sync]: ../../megaeth_specs/specs/protocol/state-sync.md
[salt]: ../../megaeth_specs/specs/protocol/salt/overview.md
[mega-da]: ../../megaeth_specs/specs/rollup/data-availability.md
[mega-derivation]: ../../megaeth_specs/specs/rollup/derivation.md
[mega-batching]: ../../megaeth_specs/specs/rollup/batch-submission.md
[mega-faults]: ../../megaeth_specs/specs/rollup/fault-proofs.md
