# 补充讨论：回执、交易根，以及 MegaETH Mini-block、Flashblocks 与 OP Stack Subblocks

初稿日期：2026 年 9 月 17 日；更新日期：2026 年 9 月 21 日。
本文保留此前关于回执与交易根的讨论，并补充当前 OP Stack Subblocks 与另外两种设计的比较。
MegaETH 部分依据已核对的本地规格提交 `59797b45b1f59b7c2c147e6f99be39399a1f2b44`，包括其中按 Rex5 定义的行为；它不等于对主网升级状态的核验。OP Stack 与 Unichain 部分区分规范、架构说明、接口示例和迁移公告。

**三者共享“在完整 EVM 区块封块前，逐步发布交易执行进展”的思路。核心区别在于：谁生产这些增量，消费者通过重执行还是应用状态增量来跟随，以及消息究竟承诺了什么。** 当前 OP Stack **Subblocks 是 Flashblocks 的兼容后继方案**，不只是泛称，也不是与 Flashblocks 无关的新共识机制。

整体的 L2 → L1 发布、派生与确认流程见 [OP Stack 架构笔记](06_op_stack_architecture_and_pipeline.md)。

## 1. 回执属于每一笔交易

每笔被包含并执行的交易都有一份回执（receipt），包括执行后回滚的交易。
一个 mini-block 中的交易数必须等于回执数；其 `receipt_root` 是对这些回执构成的有序列表的承诺，并不是“整个 mini-block 的一份回执”。
见 [mini-block 组装规则](../../megaeth_specs/specs/protocol/state-sync.md#mini-block-assembly)。

回执记录执行成功或失败、累计 gas、日志和日志 bloom 等信息。
它不是完整状态变化清单，也不是执行正确性的密码学证明。
见 [Ethereum 回执结构](https://ethereum.org/developers/docs/data-structures-and-encoding/patricia-merkle-trie/#receipts-trie)。

假设同一 EVM 区块此前已消耗 `100,000` gas，后续交易如下：

| mini-block 索引 | 交易 | 执行结果 | 本笔消耗 gas | 回执中的累计 gas |
| --- | --- | --- | ---: | ---: |
| 4 | A | 成功 | 21,000 | 121,000 |
| 4 | B | 成功 | 30,000 | 151,000 |
| 5 | C | 回滚 | 45,000 | 196,000 |

索引 4 的 mini-block 包含 `[rA,rB]`，索引 5 的包含 `[rC]`。
累计 gas 的范围是整个 EVM 区块，进入下一个 mini-block 时不会归零。
见 [Receipt Fragment](../../megaeth_specs/specs/protocol/state-sync.md#receipt-fragment)。

## 2. tx_root 的计算规则

设当前 mini-block 的交易列表为 `[tx_0, ..., tx_(n-1)]`。
对每个位置 `i`，构造一个键值对：

```text
key   = RLP(i)
value = EncodeTx(tx_i)

tx_root = MPT_Root({ key -> value })
```

MPT 是 Merkle Patricia Trie。
这里的索引是当前 mini-block 内的索引，从零开始；不是交易哈希，也不是账户地址。

| 列表位置 | 键 | 值 |
| ---: | --- | --- |
| 0 | `0x80`，即整数 0 的 RLP 编码 | 第一笔交易的完整编码 |
| 1 | `0x01` | 第二笔交易的完整编码 |
| 2 | `0x02` | 第三笔交易的完整编码 |

Legacy 交易使用包括签名字段的完整 RLP 编码。
Typed transaction 使用类型字节与该类型交易载荷的拼接。
最后按以太坊 MPT 的节点编码与哈希规则计算 32 字节根；不是简单拼接所有交易后取哈希，也不是任意一棵以交易哈希为叶子的二叉树。
见 [MegaETH Mini-Block Body](../../megaeth_specs/specs/protocol/blocks.md#mini-block-body)及 [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718#transactions)。

因此，`[A,B]` 与 `[B,A]` 改变了位置到交易的对应关系，其根不同。
计算交易根只需要交易字节与顺序，不需要执行交易；计算正确的回执根则需要执行结果。
这使仅负责排序的运营者可以自行生成交易顺序承诺，但不能仅凭这个根证明执行正确。

下面省略编码步骤，仅展示索引范围：

```text
mini-block 0 = [A,B]  -> Root({0:A, 1:B})
mini-block 1 = [C,D]  -> Root({0:C, 1:D})
完整 EVM 区块 [A,B,C,D] -> Root({0:A, 1:B, 2:C, 3:D})
```

每个 mini-block 的索引重新从零开始。
完整 EVM 区块的交易根从拼接后的完整列表重新计算，不能直接合并各个 mini-block 的交易根得到。
见 [EVM Block](../../megaeth_specs/specs/protocol/blocks.md#evm-block)。

## 3. 谁决定顺序，又由什么固定顺序

普通交易的顺序由排序器选择，受 nonce、余额等交易有效性条件约束。
例如，同一账户当前需要 nonce 7 时，不能把 nonce 8 的交易放在 nonce 7 之前执行。
第一个 mini-block 还需将该区块所需的存款与 L1 属性交易放在普通交易之前。
现有规格没有要求普通交易必须先到先得或按手续费排序。
见 [Transaction Admission and Ordering](../../megaeth_specs/specs/protocol/sequencing.md#transaction-admission-and-ordering)。

顺序一旦确定，交易根绑定位置与内容；mini-block 头签名再绑定该根。
转发者不能保持原签名不变，却替换交易或改变交易顺序。
但根不能证明排序公平，也不能阻止持有签名密钥的一方另行签署冲突承诺。

mini-block 之间的顺序由获得认证的 `index` 和全链 `number` 确定，而不是网络包先到先得，也不是依据 mini-block 时间戳排序。
例如，节点已经处理完索引 3，却先收到索引 5，可以缓存 5、补取 4，再依次处理；不能把 5 当作 4 使用。
见 [Payload Delivery](../../megaeth_specs/specs/protocol/state-sync.md#payload-delivery)与 [Mini-Block Timestamps](../../megaeth_specs/specs/protocol/blocks.md#mini-block-timestamps)。

当前 mini-block 头没有前一个 mini-block 的哈希，也没有累计交易前缀根。
第二个 mini-block 的局部交易根只承诺自己的交易，不自动承诺第一个 mini-block 的具体交易内容。
回执中的累计 gas、执行状态对前序交易的依赖，也不能替代对前序交易字节的密码学承诺。
见 [Mini-Block Header](../../megaeth_specs/specs/protocol/blocks.md#mini-block-header)。

## 4. 三个名称对应什么，共同思路是什么

- **MegaETH mini-block**：MegaETH 本地规格中的原生细粒度单位，也是 state-sync 的最小同步单位。
- **Flashblocks**：Flashbots 的部分区块预确认方案，Unichain 使用这一名称；其公开设计通过 Rollup-Boost 与外部构建者集成，最终仍输出普通 OP Stack L2 区块。
- **OP Stack Subblocks**：当前有独立规范和架构说明的后继方案，保留 Flashblocks 的消费者接口，但改变了生产架构和若干字段语义。

见 [MegaETH 区块层级](../../megaeth_specs/specs/protocol/blocks.md#block-hierarchy)、[Flashblocks 架构](https://specs.optimism.io/protocol/flashblocks.html#system-architecture)、[Subblocks 兼容性](https://specs.optimism.io/protocol/subblocks.html#backwards-compatibility)。

共同的思路是把完整区块的构建窗口分成较小的发布间隔，让应用提前观察已经执行的交易及结果。它们都保留外层 EVM 区块，**更快看到执行进展不等于更快获得 L1 最终性**。Subblocks 也不是 batch/channel/frame 那类发布到 L1 的序列化分片；其边界用于链下流式预确认，没有因此新增一个 L1 派生或结算单位。

OP Labs 的[架构迁移公告](https://docs.optimism.io/notices/preconfirmation-architecture-transition)列明 Subblocks 已于 2026 年 8 月 31 日在 OP Mainnet 上线，同时说明 Flashbots 继续维护 Flashblocks。不能把 OP Labs 的支持迁移理解为所有 OP Stack 链、尤其 Unichain，都已换成同一实现。

## 5. 核心对照：生产、传输与消费

| 比较项 | MegaETH mini-block（本地规格） | Flashblocks（公开设计／Unichain 文档） | OP Stack Subblocks（当前官方规范与架构） |
| --- | --- | --- | --- |
| 协议位置 | MegaETH 区块层级与 state-sync 的组成部分 | 预确认扩展，保留 OP Stack 基础规则 | 执行客户端原生发布部分区块，但不改变 OP Stack 有效性／最终性规则 |
| 生产组件 | 规格规定输出和签名权限，不规定 sequencer 内部如何拆分 | `op-node → Rollup-Boost → 外部 builder`，另有 fallback 执行客户端 | `op-node → 执行客户端`；后者边执行边发布，CL/EL 之间不放 Rollup-Boost sidecar |
| 消息标识 | EVM 区块号、块内 `index`、全链 mini-block `number` | 通常按 `payload_id + index` 标识一次构建中的更新 | 保留 `payload_id + index`，并与区块号关联；第一份携带 `base` |
| 增量内容 | 本份交易、回执，以及单独传输的 SALT 状态增量 | 新增交易和执行相关 metadata；字段依设计、客户端与部署核对 | 新增交易；metadata 含相关回执与更新后的余额 |
| 接收端状态 | 快速 state-sync 路径应用状态增量，不靠重执行跟随 | 文档描述预确认缓存／overlay；验证及重执行方式需按具体客户端区分 | Subblock-aware RPC 节点执行流中的交易，生成 pending 状态 |
| 回执根 | 只覆盖当前 mini-block | 累计到当前更新的执行前缀 | 同样覆盖累计执行前缀 |
| 交易根 | 每个 mini-block 有自己的 `tx_root` | 不能从版本不一的块哈希字段推定统一的逐份交易承诺 | 所列消息结构没有独立 `tx_root`；按流顺序拼接交易 |
| 状态根 | Mini-block 头无此字段；封块时核对 EVM header 的状态根 | 设计要求、消息类型和公开样例并不总一致，见第 7 节 | 流中明确置零；封块后从普通区块传播取得真实值 |
| 认证 | Rex5 起有 mini-block 头签名；流 payload 另有内容与位置认证 | 哈希链、签名或可选 TEE 不是所有部署的统一保证 | 所列流结构不含 MegaETH 式逐份签名；预确认仍依赖 sequencer |
| 节奏 | 约 10 ms 是描述性的生产节奏，数量和封块时机不是固定有效性规则 | Unichain 文档描述约 200 ms | 通常以 200 ms 为目标，可配置，不是每份必达的延迟保证 |

生产拓扑依据 [Subblocks 官方 Architecture](https://docs.optimism.io/op-stack/features/subblocks#architecture)与 [Flashblocks System Architecture](https://specs.optimism.io/protocol/flashblocks.html#system-architecture)。Subblocks 仍可使用 WebSocket proxy 转发，`op-conductor` 可管理多 sequencer：省去 CL/EL 间的 sidecar 不表示整个系统只剩一个进程。这里描述官方给出的架构，不推定任意具体链的全部进程配置。

MegaETH 的边界见 [Sequencing](../../megaeth_specs/specs/protocol/sequencing.md#block-production-mechanics)与 [State Sync](../../megaeth_specs/specs/protocol/state-sync.md)。Flashblocks 消费端见 [Unichain 集成文档](https://developers.uniswap.org/docs/unichain/technical-information/flashblocks#how-it-works)；Subblocks 消费行为见[规范](https://specs.optimism.io/protocol/subblocks.html#payload)与[应用集成指南](https://docs.optimism.io/app-developers/guides/transactions/integrating-subblocks)。

## 6. 状态同步与独立执行是最重要的语义差别

设消费者已经拥有前缀状态 `S`，收到下一小份数据。

```text
MegaETH 的快速 state-sync 路径：
    接收交易、回执、状态变化 fragments
    → 验证 sequencer 身份、内容与位置
    → ApplyDelta(S, state_delta) 得到下一状态
    → 封块后与签名 EVM header 的状态根核对

OP Stack Subblocks-aware RPC 路径：
    接收下一份新增交易
    → 在本地前缀状态上执行交易
    → Execute(S, new_transactions) 得到下一状态
    → 将结果用于 pending 查询；随后核对普通封块结果
```

MegaETH 的 state-change fragment 携带 SALT deltas、新字节码等信息；交易与回执分别走另外的流，封块 header 再走独立流。一个含 `n` 个 mini-block 的 EVM 区块对应三组逐份 fragments 和一份封块 header。见 [Streams and Payloads](../../megaeth_specs/specs/protocol/state-sync.md#streams-and-payloads)。

这条快速路径区分三种检查：

1. **来源认证**：数据是否来自该区块授权的 sequencer，而非由转发者替换。
2. **承诺一致性**：应用 deltas 得到的状态是否与 sequencer 签署的 EVM header 一致。
3. **执行正确性**：这些状态变化是否确实是按 EVM 规则执行那些交易的结果。

前两项不能代替第三项。MegaETH 的 [Trust Model](../../megaeth_specs/specs/protocol/state-sync.md#trust-model)明确将独立执行验证放在其他机制中；本地 [Stateless Validation](../../megaeth_specs/specs/protocol/stateless-validation.md)和 [Fault Proofs](../../megaeth_specs/specs/rollup/fault-proofs.md)仍是占位页，不能声称已经从这些页面验证了完整实现或部署。

Subblocks 的余额 metadata 也不能等同于 MegaETH 的完整状态同步协议。官方的 Subblocks-aware 节点通过执行交易提供 pending 状态，不是仅凭余额 metadata 重建全部合约存储。见 [Subblocks 说明](https://docs.optimism.io/op-stack/features/subblocks#what-a-subblock-payload-carries)。

这种独立执行以正确的父状态和协议规则为前提；它验证候选前缀的计算结果，不保证该前缀不会被 sequencer 放弃。

## 7. 局部根、累计根，以及必须按版本解释的字段

只看四笔交易 `[A,B,C,D]`，将其分两次发布；省略系统交易和升级特有字段：

| 第二次发布时观察的内容 | MegaETH mini-block | Flashblocks 累计载荷设计／Subblocks |
| --- | --- | --- |
| 本次新增交易 | `[C,D]` | `[C,D]` |
| 本次附带的交易回执 | `[rC,rD]` | 对应新增交易的回执 |
| 回执根覆盖范围 | `[rC,rD]` | `[rA,rB,rC,rD]` |
| 接收方已累计的交易 | `[A,B,C,D]` | `[A,B,C,D]` |

“消息是增量”不表示每个字段都只描述本次增量。MegaETH 的根范围见 [Mini-Block Body](../../megaeth_specs/specs/protocol/blocks.md#mini-block-body)；OP 字段范围见 [Subblock payload](https://specs.optimism.io/protocol/subblocks.html#payload)。

**当前 Subblocks 对四个兼容字段有明确规定：** `state_root`、`block_hash`、`withdrawals_root` 都是零哈希，`withdrawals` 是空列表。消费者应将它们视为不可用，不能用来证明状态或区块身份；但 `receipts_root`、`gas_used`、`logs_bloom` 仍有真实累计值。因此，既不能把零根当成最终区块的根，也不能说整条消息完全没有承诺相关信息。见 [字段迁移公告](https://docs.optimism.io/notices/subblocks)。

这里的 `withdrawals` 是保留的 Engine API 字段，不是 OP Stack 的 L2 → L1 提款消息列表；空列表不表示这条链不能提款。见 [字段定义](https://specs.optimism.io/protocol/subblocks.html#payload)。

Flashblocks 则必须分清资料层次：

- **公开设计**：[Flashblocks 规范](https://specs.optimism.io/protocol/flashblocks.html#data-structures)描述 `parent_flash_hash`，以及状态根等字段的用途。
- **当前公开样例**：[Unichain WebSocket 示例](https://developers.uniswap.org/docs/unichain/technical-information/flashblocks#websocket-api)没有展示 `parent_flash_hash`，且 `state_root` 为零，同时有非零 `block_hash` 和 `withdrawals_root`。这个样例不是对当前主网全量消息的实测，也不能套用 Subblocks 的四字段置零规则解释它。
- **历史核查记录**：2026 年 9 月 17 日曾核对的公开 `FlashblocksPayloadV1` 类型没有 `parent_flash_hash`。保留此记录用于解释旧笔记，不能将其表述为所有当前版本都没有该字段。

Subblocks 保留旧格式以兼容消费者，但“解码器还能解析”不表示所有字段保留原来的承诺语义。其 `base.parent_hash` 指向父 **EVM 区块**，也不能当成上一份 subblock 的哈希。见 [Subblocks 字段定义](https://specs.optimism.io/protocol/subblocks.html#payload)。

累计状态根、累计回执根与累计交易根也不同。不同交易历史可能产生相同状态或回执内容，因此前两者不能自动替代对完整交易字节及顺序的承诺。Flashblocks 可结合 TEE，但这是额外的执行与信任机制，不是增量发布或 Merkle 根本身提供的性质。见 [Flashblocks Abstract](https://specs.optimism.io/protocol/flashblocks.html#abstract)。

## 8. 在确认流程和 sequencer 轮换中应怎样使用这些区别

对于 OP Stack，细粒度预确认出现在此前执行流程图的 **C1 之前**：

```text
Subblock / Flashblock 的 pending 结果
    → 完整区块封块、传播并被节点验证（C1 / unsafe）
    → L1 输入可派生（C2 / safe）
    → 相关 L1 历史 finalized（C3 / finalized）
```

这是正常推进的路径，不能据此承诺每条早期消息必然进入最终历史。Subblocks 保证同一次构建内按序追加，但整个构建可以被放弃；新构建从 index 0 开始。流本身不提供“这个构建已封块”的最终判据，仍要观察普通区块传播。见 [Subblocks 消费规则](https://specs.optimism.io/protocol/subblocks.html#consumer-guarantees)。

在 RPC 层，Subblocks-aware 节点通过 `pending` 查询暴露执行前缀，也可能提前返回交易回执；因此，“已经有 receipt”不能作为 safe 或 finalized 的判据。见 [应用集成指南](https://docs.optimism.io/app-developers/guides/transactions/integrating-subblocks)。

结合这些机制，对 sequencer 轮换有以下设计含义，而非已经具备的统一保证：

- **前缀继承要额外定义。** 同一构建中的 append-only，不自动约束换领导者后必须保留哪些预确认交易。
- **交接内容依消费方式不同。** 重执行路径需要确定的交易前缀及父状态；状态同步路径还需要对应 deltas、应用进度和认证信息。双方都需处理数据缺失与冲突。
- **MegaETH 有块内签名权限边界。** 本地规格要求一个 EVM 区块内所有 mini-block 使用同一授权 key；不能因采用细粒度发布，就推导出可在块中途更换签名者。见 [Sequencer Authority](../../megaeth_specs/specs/protocol/sequencing.md#sequencer-authority)。
- **可归责与可处罚不同。** 签名可以提供关于已发布内容的证据，但不会自动带来罚没、不可撤销的预确认或 L1 最终性。承诺、证据、恢复和处罚规则需分别设计。

[返回中文阅读索引](README.md)
