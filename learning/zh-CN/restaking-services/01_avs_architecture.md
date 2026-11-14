# 01 — AVS：链上质押如何支持一个链下服务

[学习入口](README.md) · [下一篇：EigenDA](02_eigenda_workflow.md) · [资料索引](sources.md)

## 1. 先理解它提供的是什么

一个 restaking 服务可以这样理解：**一组运营者运行特定软件，为客户完成链下工作，并用分配给这项工作的质押承担经济责任。** 客户需要的可能是数据可用性、计算结果、rollup 验证或排序。EigenLayer 提供这些服务共用的经济与身份基础设施，具体工作规则由 AVS 定义。

旧资料通常把 AVS 展开为 *Actively Validated Service*；当前 EigenCloud 概览使用 *Autonomous Verifiable Service*。下文统一使用 AVS，避免把名称变化理解成所有服务都换了一套协议。[EigenLayer Overview](https://docs.eigencloud.xyz/eigenlayer/concepts/eigenlayer-overview)

**我们新架构与 AVS 最接近的地方，是“获得服务资格，同时承担持续交付义务”。** 但“作为 AVS 接入 EigenLayer”和“拥有一个能够容错运行的 sequencer 集群”仍是两项工程：前者解决质押与问责的连接，后者解决实际生产与协调。

## 2. 各个角色如何配合

| 角色 | 做什么 | 放到我们的场景中如何理解 |
| --- | --- | --- |
| Restaker | 提供被纳入服务经济安全安排的资产 | 为某运营者承担服务责任提供质押支持 |
| Operator | 运行 AVS 软件，接收任务并输出结果或签名 | 东京集群里的运营者 A、B、C 等 |
| AVS 开发者／管理合约 | 定义工作、参与条件、确认与奖惩规则 | 定义 mini-block stream 服务契约 |
| Operator set | AVS 内承担某类工作的成员集合 | 可以按地区或职责划分，但这是我们的设计选择 |
| 客户／消费者 | 请求服务并验证交付结果 | 提交流量的一侧，以及接收流的 P |
| Aggregator（如有） | 收集结果与签名，形成证书或提交链上 | 一种协调角色；不是所有 AVS 都必须由固定聚合器运行 |

EigenLayer 的 operator set 可以按工作类型、硬件条件、服务活性要求或资产构成划分。因此，一个 AVS 不必让所有节点承担相同任务，也不必只设一个委员会。[Understanding AVSs](https://docs.eigencloud.xyz/eigenlayer/concepts/understanding-avs-on-eigenlayer)、[Operator Sets](https://docs.eigencloud.xyz/eigenlayer/concepts/operator-sets/operator-sets-concept)

这里的 **operator set 是成员与责任边界，quorum certificate 是达到某种门槛的证明**。不同项目也会把资产加权的成员集合叫作 quorum。读文档时要判断它说的是“哪一组人”还是“收到了多少签名”，不能只按同一个英文单词类比。

## 3. 从加入网络到完成一次服务

把一个任务型 AVS 的生命周期拆开看，会比直接问“它使用哪个 consensus”更容易理解。

### 第一步：确定成员、密钥和可用质押

Operator 注册，加入适用的 operator set，并分配 stake。客户端和验证合约需要知道使用哪一份成员及权重记录，否则“有 70% 签名”没有确定的分母。任务还应绑定服务、输入和适用版本，避免把另一个任务的签名拿来使用。

EigenLayer 当前的 **Unique Stake** 是分配给特定 operator set、由相应 AVS 在该边界内可罚没的质押份额。不能把一个运营者的全部委托额，或整个 EigenLayer 的总质押额，都当成某个服务实际可以扣除的保证金。[Unique Stake](https://docs.eigencloud.xyz/eigenlayer/concepts/slashing/unique-stake)

### 第二步：产生任务并执行

任务可以由客户、链上事件或服务自己的调度器产生。节点按服务规则获取输入、执行工作、检查结果，再发出签名。例如，数据服务的检查是“我已取得并保存负责的分片”；计算服务的检查是“我对这个输入得到这个结果”。

任务型框架 Hourglass 将这些角色拆成 TaskMailbox、Aggregator、Executor 和执行具体逻辑的 Performer。它展示了一条完整路径：链上创建任务，聚合器发现执行者并分发输入，执行者运行程序并签名，聚合器提交达到门槛的结果。这是框架的设计流程；所查公开仓库已归档，并明确标为测试用途，适合学习结构。[Hourglass 快照](https://github.com/Layr-Labs/hourglass-monorepo/tree/ca1c88e06f30a95500bfddc1506cabf8a95770fa)、[仓库状态](https://github.com/Layr-Labs/hourglass-monorepo)

### 第三步：确认结果

验证者检查签名、成员资格、权重门槛，以及服务特有的证明或约束。**BLS 聚合签名压缩的是一组签名，并没有替服务定义“什么结果正确”或“出现分歧怎样处理”。** 对同一个确定性任务，节点可以独立计算再聚合；对相互竞争的历史，系统还需要选择与一致性规则。

例如，任务结果 `7² = 49` 可以直接复算；两个合法 mini-block 候选却可能都有效，只是交易顺序不同。后一种情形需要解决选择问题，不能期待计算正确性替我们选出唯一下一块。详见 [第 3 篇](03_consensus_and_rotation.md)。

### 第四步：奖励、挑战与退出

EigenLayer 的奖励基础设施接收 AVS 的奖励提交，支持按 operator 指定分配。它负责记录和分发奖励，工作是否完成及应分配多少，仍需要 AVS 的业务规则与依据。[RewardsCoordinator](https://github.com/Layr-Labs/eigenlayer-contracts/blob/main/docs/core/RewardsCoordinator.md)

退出也需要时间边界。分配 stake 的生效、解除分配及注销后的可罚没窗口，都有相应延迟；“不再接新任务”与“历史任务已不再承担责任”不是同一时刻。[Allocation and Deallocation](https://docs.eigencloud.xyz/eigenlayer/concepts/operator-sets/allocation-deallocation)

**对轮换的启示：**A 的任期结束时，可以将生产权移交给 B，但 A 对任期内已签承诺的责任，应保留到相应的检查／挑战窗口结束。这是对本项目的设计推断。

## 4. Slashing 到底由谁决定

这里需要区分两个层面：

| 层面 | 提供什么 |
| --- | --- |
| EigenLayer 核心协议 | 在权限、stake 分配与时间条件满足时，执行份额扣减、销毁或适用的再分配 |
| AVS 自己的规则 | 什么算违约、谁提交指控、谁判断、需要什么证据、是否有延迟或否决流程 |

当前官方 slashing 文档明确指出：核心罚没接口具有较大灵活性，AVS 的罚没理由不必都能以客观证明在链上裁决。因此，**“支持 slashing”不能直接理解为“任何人都能提交一份无争议的欺诈证明，并由合约自动裁决”。** 若依赖委员会、管理员或社会判断，应明确写出这些信任条件。[Slashing Overview](https://docs.eigencloud.xyz/eigenlayer/concepts/slashing/slashing-concept)

对我们而言，下列三种行为需要不同的依据：

- **冲突签名：**先定义什么叫同一位置、同一授权下不兼容的承诺，再验证签名。
- **没有按时交付：**先定义任务何时有效生效、输入条件、交付对象和时限，再判断超时。
- **某接收方说没有收到：**需要进一步检查数据路径和故障原因，不能只凭这一句陈述完成归责。

故障接替可以由服务恢复规则触发；是否扣钱由归责规则决定。这个区分使系统可以及时恢复，而不必在每次接替前先解决全部争议。

## 5. 一个月排班，需要固定什么

我们提出“按 L1 stake 和随机性，提前派生一个月的排班”。这可以作为 AVS 的调度规则，但普通的 operator set 注册本身不提供这样一张排班表。

至少需要固定：**成员及权重快照、排班随机源、适用时间段、故障接替顺序，以及该时段内实际可依赖的质押承诺。** 节点中途退出、stake 变化和委员会换届都需要自己的生效规则。

EigenLayer 的 Duration Vaults 提供了相关但范围不同的机制：存入阶段结束后，将资产在固定期限内锁定并分配给指定 operator set，期限结束再处理退出。它值得作为“固定服务期内的质押承诺”参考；它不会替我们完成 leader 抽签，也不会保证机器整个期间在线。[Duration Vaults](https://docs.eigencloud.xyz/eigenlayer/concepts/duration-vaults)

## 6. 对这周方案最直接的收获

可以把同地 sequencer 集群定义成一个持续服务型 AVS，逐项写出：**哪个 operator set 提供服务、当值权如何产生、交付物是什么、哪些成员确认、P 如何接受、发生故障如何恢复、质押责任何时结束。**

EigenLayer 提供可复用的经济与身份组件。我们的主要协议工作仍是：把这些组件接到 mini-block stream 的生产、确认和接替流程上，使“有质押的节点”真正成为“有明确服务义务的节点”。
