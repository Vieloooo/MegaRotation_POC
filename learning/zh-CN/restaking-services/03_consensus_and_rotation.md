# 03 — 从服务证书到连续历史：哪些机制适合我们的轮换架构

[学习入口](README.md) · [上一篇：EigenDA](02_eigenda_workflow.md) · [资料索引](sources.md)

本篇先比较具体服务的工作流程，再映射到 [本周需求](../../../meetup_record/meeting_0922_requirements.md)。机制分析与本项目的设计推断分开表述。

## 1. 任务型 AVS：用 Incredible Squaring 看完整流程

Incredible Squaring 是 EigenLayer 的教学项目。它的任务很简单：给一个整数，节点计算平方。简单的业务逻辑让我们能看清楚 AVS 的组织结构。

一次任务按下面的顺序完成：

1. **创建任务。** Task generator 在链上登记输入、所需 quorum 与门槛。
2. **执行任务。** 已加入的 operators 观察任务，独立计算结果，签名并交给 aggregator。
3. **汇总结果。** Aggregator 按相同结果收集 BLS 签名，把达到要求的响应提交给 TaskManager。
4. **检查响应。** 合约核对任务、时间窗口、签名以及任务创建时的 stake 门槛。
5. **挑战错误。** Challenger 可以复算结果；错误结果进入合约中的挑战与罚没路径。

这是测试示例的流程，不是已经投产的计算网络。固定快照中的 TaskManager 包含 `respondToTask`、`raiseAndResolveChallenge` 和调用 slasher 的路径；README 某些段落仍保留更早的 placeholder／veto 表述，理解实现时应按具体代码核对。[README 快照](https://github.com/Layr-Labs/incredible-squaring-avs/blob/d922e8cc0cd1be6b77787bb75fa4460acec2411d/README.md)、[TaskManager 快照](https://github.com/Layr-Labs/incredible-squaring-avs/blob/d922e8cc0cd1be6b77787bb75fa4460acec2411d/contracts/src/IncredibleSquaringTaskManager.sol)

**这里没有自动出现一条新的 BFT 链。** 任务的登记顺序由底层链确定，链下节点为特定结果提供签名。Aggregator 的存在也不说明它已经有容错选举或自动接替协议。

**与我们的问题比较：**如果任务是“检查这份 mini-block 是否满足共同规则”，可以借鉴这个结构；如果任务是“多个合法 mini-block 里选哪一个作为唯一后继”，独立复算不会得到唯一答案。交易打包策略不同，正是我们必须另外处理的选择空间。

## 2. MACH：验证 rollup，并对异常告警形成确认

AltLayer 将 restaked rollup 的服务分为 VITAL、MACH 和 SQUAD，分别面向状态验证、快速最终性和去中心化排序。这是服务分工，不表示三者运行相同的 consensus。[AltLayer Overview](https://docs.altlayer.io/)

为了避免只根据产品名推断机制，这里查看了公开的 MACH M2 仓库快照 `3da201ff`。它展示的一个具体路径是：

| 阶段 | 谁做什么 |
| --- | --- |
| 观察 | Mach verifier 从 L2 获取区块／状态信息，并运行相应检查 |
| 发现异常 | Verifier 产生异常告警，交给 operator 等处理路径 |
| 签名与聚合 | Operator 为告警签名，aggregator 收集支持 |
| 链上确认 | ServiceManager 检查告警、参考区块的 stake 与 quorum 门槛，记录已确认告警 |
| 应用使用 | 依赖方查询告警状态，例如作为 RPC 服务的熔断依据 |

同一仓库还描述 ZK proof 模式，使用证明支持告警，无需以聚合签名完成相同的确认步骤。[MACH README 快照](https://github.com/alt-research2/mach-avs/blob/3da201ff5c04b47021448dde68c09de00f2a17f0/README.md)

**这个实现路径是“验证与告警确认”，不能直接描述成每个 rollup 区块都由 MACH 的独立 HotStuff/Tendermint 网络重新排序。** 所查 `confirmAlert` 有专门的调用权限，门槛与 allowlist 也有配置接口；仓库列有部署信息，但本次没有审计某一主网实例当前的权限与参数。[MachServiceManager](https://github.com/alt-research2/mach-avs/blob/3da201ff5c04b47021448dde68c09de00f2a17f0/contracts/src/core/MachServiceManager.sol)

**对本项目的启示：**可以让一组成员观察 S 的服务并形成可验证报告。不过，确认“出现了某种异常”与确定“B 从哪个前缀开始接替 A”仍是两个协议动作。

SQUAD 的服务定位更接近我们要的 sequencer rotation，但这次取得的一手资料不足以核实其完整轮换、锁定与 view-change 算法。因此这里只将它列为进一步调研对象，不从宣传中的“去中心化排序”推断具体实现。AltLayer 另有 in-house stack 的 Beacon Layer、随机选 sequencer 等描述，也不能直接拼接成 SQUAD 当前部署的证明。[SQUAD 所在的服务概览](https://docs.altlayer.io/)、[In-house Decentralized Sequencers](https://docs.altlayer.io/core-features-of-altlayers-in-house-rollup-stack/decentralized-sequencers/)

## 3. 文献主线一：只保证数据能够取回，需要什么

### Semi-AVID-PR：对服务性质作有针对性的取舍

Nazirkhanova、Neu 和 Tse 的 *Information Dispersal with Provable Retrievability for Rollups*（AFT 2022）研究高效的链下数据分散和可取回性。它采用纠删码与承诺，并针对 rollup 的需要放松传统 information dispersal 的部分终止要求。[论文](https://arxiv.org/abs/2111.12323)

这与我们提出 weak liveness 的研究思路有关：**先明确消费者究竟需要哪种进展，再决定哪些性质必须由底层服务承担。** 但该论文对 termination 的取舍，与我们“至少某个成员按自己的策略持续产出 mini-block”的要求，不是同一个定义，不能直接援引其结果。

EigenDA 的新白皮书将该工作列为存储层 VID 的参考。建议读这篇时重点看 dispersal 的输出、retrieval 的输入，以及持有证书之后到底承诺什么；可以暂时跳过密码学实现细节。

### DispersedLedger：把数据传输与有序日志分开

DispersedLedger（NSDI 2022）进一步研究：节点可以先对小型 commitment 的有序日志达成一致，再按自己的网络能力下载对应的完整数据。它需要保证被确定的数据确实可取回，不能只对无法下载的哈希进行排序。[论文与演讲](https://www.usenix.org/conference/nsdi22/presentation/yang)

这与我们的小集群存在一个直接连接：**用于决定授权、前缀与接替的消息可以很小，而完整 mini-block 数据走独立的传输路径。** 这是可借鉴的分工；论文针对异步、带宽波动环境的完整协议，不等于我们已经得到同地低延迟方案。

## 4. 文献主线二：要维持同一条 stream，需要增加什么

### HotStuff：稳定 leader 与前缀约束

HotStuff 是 leader-based BFT replication 协议。它不仅收集签名，还规定提案、投票、证书、锁定和接替时的状态选择。论文中的 Pacemaker 负责帮助节点进入能够继续进展的轮次；新 leader 获取已有证书信息，再提出可被支持的后继。[HotStuff，§4–6](https://arxiv.org/abs/1803.05069)

其中 §6 的事件驱动版本明确讨论 **stable leader 跨多个高度连续提出内容**。因此，可以把下面三个节奏分开考虑：

- Mini-block 多久生成一次。
- 已产生的前缀多久形成一次可依赖的确认。
- Leader 多久正常轮换一次，以及什么故障触发提前接替。

**对我们而言，“不逐 mini-block 轮换 leader”是合理的架构约束，但它没有自动决定是否逐 mini-block 投票。** 如果减少确认频率，需要说明窗口内尾部的承诺，以及故障时如何处理。这是设计选择，不能只通过延长 leader 任期解决。

### Espresso 的 2024 论文：共识可以面向数据承诺，执行留给使用者

Espresso 的 *HotShot Consensus, Tiramisu Data-Availability, and Builder-Exchange* 将构建区块、对数据可用性证书的序列达成共识，以及 rollup 自己的执行分开。它还处理 builder 与共识 proposer 之间的信息和付款交换问题。[2024 论文](https://eprint.iacr.org/2024/1189)

我们可从中提取一个架构方向：**小集群共同确定可获取的 mini-block 前缀，P 根据自己的执行与发布规则继续处理。** 但将具体执行有效性留给 P，需要同时定义 P 拒绝输入时的处理，不能让确认前缀与可执行输入脱节。

这里特指 2024 年论文的共享排序设计。当前 Espresso 文档已经将产品表述为 settlement/finality 服务，并明确说明集成应用保有自己的排序规则；本文不将论文中的所有描述当成当前产品接口。[当前 How Espresso Fits](https://docs.espressosys.com/network/learn/espresso-in-the-modular-stack.md)

## 5. 一个最小例子：为什么有签名还不够

下面是我们自己的教学例子。

共同前缀为 H，A 产生 X，B 产生 Y；X、Y 都满足交易有效性规则，但都声称紧接 H，彼此不兼容。

| 成员签名时执行的规则 | 能得到什么 | 仍缺少什么 |
| --- | --- | --- |
| “我保存了这份数据” | X、Y 都可以有可用性证书 | 唯一后继的选择规则 |
| “我检查过这份交易包有效” | X、Y 都可以通过有效性检查 | 同一位置只能确定一个结果的规则 |
| “我只支持与已锁定历史兼容、且满足当前轮次规则的后继” | 为共同前缀提供约束基础 | 配套的门槛、锁定更新和接替规则 |

第三行也不是一份完整协议：必须规定消息何时达到确认、不同成员看到不同进度如何继续、换届时怎样继承状态。它只是说明，我们需要把签名的命题从“数据存在／有效”推进到“这段历史可以作为共同后继”。

同样，固定委员会内的诚实比例保证不能直接推出新旧委员会之间的安全交接。地理轮换或成员更新，需要带着可信的继承前缀转移授权。

## 6. 映射回我们的需求

下表是基于文献的设计推断，不是建议直接采用某个系统的完整协议。

| 本周需求 | 最值得借鉴的机制 | 我们仍要定义什么 |
| --- | --- | --- |
| 不同运营者形成同地小集群 | EigenLayer operator sets | 地理资格、故障相关性、成员与投票权重 |
| 以 mini-block 为交付单位 | EigenDA 的明确对象、数据持有与证书验证 | Mini-block 是否已执行，P 如何接受与拒绝 |
| Block-level weak liveness | 将服务进展条件与数据恢复条件分别定义 | 正常交付与接替的时限，以及容忍哪些不合作 |
| 一条连续 stream | HotStuff 的前缀约束和接替状态 | 哪一级确认对 P 生效，未确认尾部怎样处理 |
| 低延迟、长任期 | Stable leader；控制消息与数据传输分开 | 共识频率、数据复制方式和关键路径 |
| 按 stake 提前排班 | 权重快照、时间范围、stake commitment | 随机源、分配规则、撤销和故障替补 |
| 跨地理轮换 | 授权迁移和历史状态继承的接口 | 备用地区如何同步，地区失效时谁决定接替 |
| 奖励与罚没 | 显式任务、结果、证据和可罚没窗口 | 正常 leader、备用成员、数据服务各自如何计酬 |

**同地部署尤其需要注意故障相关性。** 五个独立运营者如果都依赖同一机房、电力或出口网络，仍可能同时失效。这里的推论是：operator 的组织独立性、BFT 权重分散程度与地理故障隔离，应分别描述；不能从“有五个运营者”直接得到地区容灾。

## 7. 下一轮讨论可以先确定的三件事

1. **我们要委员会确认哪一种东西？** 仅确认当值权，还是同时确认 mini-block 前缀和数据持有？选择不同，给 P 的保证也不同。
2. **我们如何保持低延迟？** 稳定 leader、缩小控制消息、异步复制与降低确认频率分别影响什么；先选择承诺边界，再谈优化。
3. **我们如何证明服务已经交付？** 以哪些数据和证书触发 P 的义务；故障时先恢复哪些功能，之后凭什么归责。

若本周先产出架构方案，优先把这三项写成清晰接口，随后再决定采用或改造哪一种 BFT 算法。EigenDA 提供的是很好的服务分工参考；连续历史与接替仍需要自己的协议定义。
