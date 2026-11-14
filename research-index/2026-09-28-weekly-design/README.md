# 过去一周的 sequencer rotation 设计思路：资料索引

整理日期：2026-09-28（Asia/Shanghai）。主体范围为 **9 月 22–28 日的项目 Codex 对话、提交与设计资料**；9 月 21 日的 OP Stack 学习对话作为本周起点补充。用途是为后续汇报准备材料，本次不制作 slides。

配套总结：[设计 tradeoffs：指标、运行场景与方案取舍](tradeoffs-zh.md)。

后续汇报材料：[架构与 tradeoffs 幻灯片及讲述安排（9/29 整理版，13 页）](../../presentations/2026-09-28-design-tradeoffs/README.md)。

本索引的 Git 基线为 `a095f61a0b3e818122453071fe50356e8c7443c4`（9 月 28 日 23:03，`add a new design`）。最新 v3 正文写作任务仍在进行中；本索引以该提交内的 status-entry 大纲为最新已提交方向，不将进行中的工作视为定稿。来源文件哈希见 [source-manifest.json](source-manifest.json)，对话入口见 [chat-index.md](chat-index.md)。

## 1. 先读这六份材料

| 顺序 | 资料 | 用它回答什么 |
|---|---|---|
| 1 | [9/22 架构愿景与需求](../../meetup_record/meeting_0922_requirements.md) | 为什么把 sequencer 与 proposer 分开？我们需要怎样的 live stream？ |
| 2 | [作者原始 outline](../../formalization/outline.md) | GeoBFT 与区域 LMD-GHOST／全局 FFG 的最初构想是什么？ |
| 3 | [LMD/FFG 改写记录](../../formalization/reviews/lmd-ffg-redesign/README.md) | 为什么允许本地 preview 重组，把持久保证交给全局层？ |
| 4 | [main_v1 跨区轮换与 virtual SMR](../../formalization/reviews/main-v1-cross-region-and-virtual-smr-2026-09-28.md) | 现有全局层有多重？把 region 变成外层 CFT 节点需要什么？ |
| 5 | [SMR 封装文献与接口](../../formalization/reviews/smr-protocol-wrapping-literature-2026-09-28.md) | 为什么“内部跑 BFT”不自动得到 crash-only 进程？ |
| 6 | [最新已提交 v3 大纲](../../formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md) | 如何改为区域 SMR、固定排班、状态委员会与 L1 派生验证？ |

你本次口述的 “zero BFT” 在原始 outline 和历史对话中对应 **GeoBFT**，本索引采用该名称。“global routing” 暂按跨区协调、轮换和交接的概括理解；历史材料没有一个已定名为 routing 的统一原语。

## 2. 本周设计主线

以下是对已有材料的整理归纳，不是新协议或新的安全证明。

最初的问题是：把交易排序从 proposer 中拆出来后，谁能保证获得排序权的 sequencer 持续提供可用、可验证的 stream，以及故障时由谁接替？随后出现两个相关但不同的设计轴：**区域内部如何确认 stream**，以及 **跨区域如何继承历史、转移权限和向 L1 证明来源**。

区域内部先后考虑了同步两跳 broadcast、LMD-GHOST 的可重组 preview、PBFT 和 Sync HotStuff；外层先考虑报告一致性加 FFG，再探索把区域封装成 crash-only 节点运行 CFT，最后转向 L1 预设排班与低频状态委员会。不能把区域原语的替换全部当作外层架构的替换。

| 设计路线 | 区域内负责什么 | 跨区层负责什么 | 主要考量 | 在资料中的状态 |
|---|---|---|---|---|
| A. GeoBFT 启发的分层 BFT／broadcast | 对 batch 或前缀形成区域确认 | 传播、共同采纳前缀、轮换与最终确认 | 利用低区域延迟；但快速确认后的 completion 和全局协调仍有成本 | 已有多个具体历史版本 |
| B. 区域 LMD-GHOST + 全局 FFG | 快速出流、HEAD 投票、选择可重组分支 | 先形成共同报告／目标，再用 FFG 确认 checkpoint 与权限切换 | 减轻区域逐 batch 决定；代价是局部头不等于不可回滚确认，全局层依然昂贵 | 当前 `main.tex` 对应此路线 |
| C. 区域 BFT SMR + 外层 CFT | 实现一个正确或停止的逻辑协议进程 | 由逻辑 region 运行 CFT，复制全局状态和转移领导权 | 需要复制外层程序的状态、输入、timer、输出与恢复义务，不能只复制业务 stream | 文献与接口已有；外层未完成 |
| D. 区域 SMR + L1 排班 + 状态委员会 | 提供区域 final receipt 和可取回 stream | 正常按 quota 与列表轮换；活动区域故障由状态更新触发恢复 | 减少常态全局协调；必须补足安全交接、委员会信任和 L1 验证关系 | 最新已提交 v3 提案，非完整证明 |

### A. 从 GeoBFT 出发：区域确认 + 跨区协调

**最初动机。** 把快通信放在同一个 region 内，批量确认 mini-block，减少跨地理通信进入每次快速生产的路径。

**重要材料：**

- [原始 outline](../../formalization/outline.md)：GeoBFT 启发、L1 权重、两层轮换、长任期、batch 确认。
- [GeoBFT／PBFT／FFG 来源与设计差异](../../formalization/reviews/consensus-sources.md)：GeoBFT 原方案由各 cluster 贡献请求；项目中的单 active region 是新的适配。
- [同步 broadcast 来源](../../formalization/reviews/synchronous-broadcast-sources.md)：`2δ` good-case 路径及其收尾条件。
- [main_v1.tex](../../formalization/main_v1.tex)：区域 fast path + completion、全局 report agreement + FFG。
- [区域 completion 优化研究](../../formalization/reviews/main-v1-completion-literature-2026-09-28.md)：当前实现为何出现 cubic 消息成本，以及 binary BA、权重适配、证书和数据传输的不同优化对象。
- [main_PBFT.tex](../../formalization/main_PBFT.tex) 与 [PBFT 修订记录](../../formalization/reviews/pbft-revision-2026-09-28/README.md)：区域使用 PBFT 的对照版本，仍保留全局控制层。

**汇报应保留的取舍。** `2δ` 是特定模型下内部快速决定的时间，不是完整 stream 服务成本，也不是外部 P 取得永久确认的端到端延迟。历史 main_v1 的正常实例也运行 completion；聚合签名压缩证书不等于消除 agreement 的消息交互。

### B. 改造 Ethereum 结构：区域 LMD-GHOST，全局 FFG

**动机。** 不要求区域内每个 slot 都形成共同 DECIDED／commit；先连续生产允许重组的分支，把不可回滚的前缀收敛放到较慢的全局 checkpoint。

**重要材料：**

- [main.tex](../../formalization/main.tex)：§3 服务契约、§5.2 区域 LMD、§5.4 报告与 checkpoint、§5.5 最终性与激活、§7 成本。
- [LMD/FFG 改写记录](../../formalization/reviews/lmd-ffg-redesign/README.md)：改写原因、已删除的逐 batch completion、分轮审查与修复。
- [Ethereum 对话上下文辨析](../../formalization/reviews/ethereum-chat-context.md)：head selection、FFG finality、授权和执行正确性的区别；原 outline 的 Ethereum 类比有哪些不准确之处。
- [FFG 组合分析](../../formalization/reviews/ffg-composition-advisory.md)：FFG 与已有 agreement 各承担什么。
- [9/27 liveness 审查](../../formalization/reviews/attack-audit-2026-09-27/attack-review.md)：数据 deadline、backpressure、leader 排班、任期与资源限制。

**关键发现。** 这里不是把 FFG 接上去就自动解决全局选择。该版本仍用全局 signed report agreement 形成共同输入和 checkpoint，再进行 FFG voting。局部 Preview、全局 adopted prefix、finalized Deliver 必须分开讲。

**审查边界。** 9/27 记录区分模型内允许的暂停、参数违反进展前提的饥饿，以及破坏持续同步／完整对象假设后的死锁。不能把这些全部称为“已证明该协议不安全”，也不能将旧审查转用于后来的 v2/v3。

### C. BFT region 作为一个逻辑进程，再运行 CFT

**动机。** 若每个 region 对外都只可能正确运行或停止，跨区协调能否使用比全局 Byzantine consensus 更轻的 CFT？进一步要求 leader 顺序服从 L1 排班，而不是自由竞选。

**重要材料：**

- [virtual-SMR/CFT 抽象](../../formalization/reviews/main-v1-cross-region-and-virtual-smr-2026-09-28.md)：旧协议的真实 global voters 是全部 sequencer，不是 region 逻辑节点；改成 CFT 是新的故障模型和协议。
- [通用 wrapper 文献](../../formalization/reviews/smr-protocol-wrapping-literature-2026-09-28.md)：Steward、Nysiad、SMR 输入／输出、持久 outbox、timer／随机性、fair input processing。
- [L1 指定 CFT 轮换](../../formalization/reviews/l1-scheduled-cft-rotation-2026-09-28.md)：VR Revisited、Vertical Paxos、Paxos、Mencius；区分 designation、activation、commit。
- [VRR 具体流程](../../formalization/reviews/vrr-walkthrough-for-regional-cft-2026-09-28.md)：预定新 primary 如何补日志和安全接班。
- [main_v2.tex](../../formalization/main_v2.tex)：区域 PoS SMR 采用 Sync HotStuff；明确把 cross-region CFT、rotation、global finality 留为未完成部分。
- [HotStuff family 与 P 的确认目标](../../formalization/reviews/hotstuff-family-proposer-finality-2026-09-28.md)：比较区域 commit、外部可验证确认与 pipeline 间隔。

**困难的准确说法。** 内层 BFT 需要约束外层程序的整个可接受行为，包括输入认证、状态转换、持久承诺、输出证据与恢复。只给业务 batch 多签，不能阻止一个恶意 representative 在外层乱投票或确认不存在的持久状态。内层同步假设失效也不自动变成 crash-only。

**为何继续寻找别的方案。** 外层一次逻辑交互可能附带内层复制／认证；CFT 仍需跨区数据恢复和安全交接。9/28 用户随后明确选择不再以“每个区域模拟 crash-only agent”作为下一版主线。

### D. 最新方向：状态委员会、固定排班与 L1 验证

**重要材料：**

- [status-entry 新大纲及修订计划](../../formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md)：当前已提交的主要阅读入口。
- [该提案状态说明](../../formalization/proposals/v3-status-entry-2026-09-28/README.md)：明确哪些旧选择已被替代、哪些仍未完成。
- [前一轮中文设计](../../formalization/proposals/v3-committee-rotation-2026-09-28/design-zh.md) 与 [修订历史](../../formalization/proposals/v3-committee-rotation-2026-09-28/README.md)：委员会范围收缩、finalized anchor、cutoff、L1 发布验证。

**已经澄清的方向：**

1. 委员会业务 entry 为 `(batch height, region → status)`；签名域、版本和防重放字段属于认证封装。
2. 正常轮换在固定数量的应用 batches 完成后，按预定列表选择下一个 live region，跳过 unavailable region。
3. 只有当前 active region 被标为 unavailable 才触发强制交接；非活动区恢复只改变候选资格，不重置当前任期。
4. 新 region 先验证旧区 finalized anchor，再开始排序。此前在未 final 候选上预启动新区的 conditional-session 方案已被撤回。
5. 委员会不逐批签 stream tip，也不签新区首 header；其高度声明仍须由区域 final receipt 和数据证明解析为具体历史。
6. L1／proof system 需要检查 sequencer stream 与 rotation 的派生关系；fraud proof 或 validity proof 的具体实现尚待选定。

**未闭合的问题。** 固定 quota 的 terminal Drain、共同生效的 L1/status snapshot、正常交接与事故事件竞态、事故 cutoff 是否覆盖全部已 final 输出、数据恢复、无其他 live region 时的策略，以及端到端 rollup 证明关系。减少委员会签署的业务字段，并不自动解决这些安全义务。

**同期工作。** [main-v3 control-design 审查](../../formalization/reviews/main-v3-2026-09-28/control-design.md) 是整理时工作区已有的未提交分析，单列参考，不用它覆盖上述已提交基线。

## 3. 横跨所有方案的四组问题

| 问题 | 本周应讲清的判断 | 主要资料 |
|---|---|---|
| 到底需要什么 safety？ | 排序来源可验证、同一 session 前缀一致、跨 region 保留已确认前缀、执行正确性、L1 finality 是不同保证。早期“safety 交给下游”的直觉后来收敛为必须明确接受层级。 | [需求 §4](../../meetup_record/meeting_0922_requirements.md)、[v1 服务契约](../../formalization/main_v1.tex)、[v3 跨区定义计划](../../formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md) |
| 全局 consensus 为什么 costly？ | WAN 传输之外，还包括共同报告、完整对象取回、证书验证、存储、quorum 与切换等待；只更换传播网络或聚合签名不能抹去共识义务。 | [virtual-SMR 分析 §§1–9](../../formalization/reviews/main-v1-cross-region-and-virtual-smr-2026-09-28.md)、[main.tex 成本](../../formalization/main.tex) |
| proposer 如何忠实 rollup？ | L1 需要核验来源、权限历史、final receipt、内容与顺序连续性，以及发布数据／承诺的绑定。执行正确性与 DA 可用性仍需独立机制。单验一个签名不等于完成全部检查。 | [9/22 责任划分](../../meetup_record/meeting_0916_response.md)、[v3 L1 派生关系](../../formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md) |
| 延迟究竟量什么？ | 分开报告 finality time、pipeline block time、区域内轮换、正常跨区轮换、事故跨区轮换；区分实际延迟 δ 与上界 Δ。 | [HotStuff 确认比较](../../formalization/reviews/hotstuff-family-proposer-finality-2026-09-28.md)、[v2 §6](../../formalization/main_v2.tex)、[最新大纲](../../formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md) |

用户在 9/28 明确把确认优化目标选为“外部 proposer P 收到可验证、不可回滚的确认”。正常轮换是日常路径、优先优化；事故轮换罕见，可以接受较长的 L1 等待。这是设计偏好，不是已测量的性能结果。

“类似跨链”的含义也应精确：L1 要验证另一个共识域输出的来源与最终确认。它不意味着本周已经构造了完整 bridge，也不意味着委员会 attestation 能替代执行证明或 DA。

## 4. 时间线与 Git 快照

提交是保存点；同一次提交内可能包含几天的讨论、替代稿和审查。按对话时间恢复设计演化更准确。

| 日期 | 主要讨论／决定 | Git 与资料依据 |
|---|---|---|
| 9/21–22 | OP Stack、fault proofs、Conductor、mini-block 与不同确认层级；拆分 sequencer／proposer 责任 | `1470f64`（OP notes）；[学习索引](../../learning/zh-CN/README.md) |
| 9/22 | L1 授权、两层地域结构、weak liveness、continuity、低延迟与责任边界 | `d6b191b`（reorg）、`1a15b61`（rotation intuition）；[会议需求](../../meetup_record/meeting_0922_requirements.md) |
| 9/24 | GeoBFT、多层 confirmation、batch 粒度、同步 `2δ`-BB；区域约 10 个节点、诚实权重阈值澄清 | [分析MegaETH sequencer rotation](codex://threads/01a0d2ac-bab6-7d60-b43c-02bbb43bf193)、[outline](../../formalization/outline.md) |
| 9/25 | 独立 live-stream 服务、正式模型与接口；两跳 fast path 后的安全收尾；轻量恢复补丁 | `bb9ff167f0505cf4e2c89cc1edd26ad09b6bb9be`；[补丁记录](../../formalization/reviews/lightweight-security-patch.md) |
| 9/26 | 追问 completion／DECIDED 是否必要、通信为什么 cubic；转向本地 LMD streaming + 全局 FFG | [详解区域轮换机制](codex://threads/01a0de08-7a1e-7001-a4ce-433121da1c7c)、[改写记录](../../formalization/reviews/lmd-ffg-redesign/README.md) |
| 9/27 | 检查 live-stream 停顿、deadline 死锁、任期与资源限制 | [冻结审查](../../formalization/reviews/attack-audit-2026-09-27/README.md) |
| 9/28 前段 | LMD 的安全作用、PBFT 对照、completion 优化、HotStuff family、virtual SMR 与外层 CFT；v2 先实现区域部分 | [chat-index](chat-index.md)、[v2 说明](../../formalization/README.md) |
| 9/28 后段 | 放弃通用 crash-only wrapper 作为主线；引入委员会，再将其收缩为 status entry；正常／事故交接与 L1 忠实派生 | `a095f61`；[status-entry 大纲](../../formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md) |

## 5. 文件版本地图：避免讲错版本

| 文件 | 应如何称呼 | 不能误称为什么 |
|---|---|---|
| [outline.md](../../formalization/outline.md) | 作者早期构想 | 已验证的完整算法；其中有历史术语混用 |
| [main.tex](../../formalization/main.tex) | LMD-GHOST／全局 FFG 路线 | 本周所有设计的唯一最新总稿 |
| [main_v1.tex](../../formalization/main_v1.tex) | 区域同步 broadcast/completion + 全局报告／FFG | main.tex 后的新一代 LMD 稿 |
| [main_PBFT.tex](../../formalization/main_PBFT.tex) | 区域 PBFT 对照分支 | 外层 CFT；或最新选定的唯一区域协议 |
| [main_v2.tex](../../formalization/main_v2.tex) | 区域 Sync HotStuff／PoS SMR，外层留空 | 已完成两层 CFT 组合与全局最终性 |
| [v3 committee 提案](../../formalization/proposals/v3-committee-rotation-2026-09-28/README.md) | 前一轮及其修订历史 | working 目录的 conditional 方案仍被采用 |
| [v3 status-entry 提案](../../formalization/proposals/v3-status-entry-2026-09-28/README.md) | 最新已提交的大纲与设计方向 | 已完成论文正文、实现或完整证明 |

## 6. 现成背景资料与图示

| 材料 | 汇报中可支持的主题 |
|---|---|
| [OP Stack 架构与数据往返](../../learning/zh-CN/06_op_stack_architecture_and_pipeline.md) | sequencer、执行、batching、DA、L1 derivation 的分工 |
| [桥接与故障处理](../../learning/zh-CN/08_op_stack_bridge_and_disputes.md) | 错误执行、拒绝发布、暂时停机不是同一问题 |
| [OP Conductor](../../learning/zh-CN/10_op_conductor_ha_trust_and_incentives.md) | CFT 高可用的原始参照与信任边界 |
| [OP Stack 与 MegaETH 对照](../../learning/zh-CN/11_op_stack_vs_megaeth_architecture.md) | 低延迟 stream、确认层级和复制保证 |
| [EigenDA／restaking 学习专题](../../learning/zh-CN/restaking-services/README.md) | 把 sequencing 视作有明确义务与证据的独立服务 |
| [v2 原生 TikZ 图索引](../../formalization/figures/v2-major/README.md) | 区域 pipeline、view change、数据取回、blame |
| [v3 两类轮换图 PDF](../../formalization/proposals/v3-status-entry-2026-09-28/figures/rotation-pipelines.pdf) | 正常轮换与 active-region 故障轮换；属于提案图 |

文献入口优先使用已有 [consensus sources](../../formalization/reviews/consensus-sources.md)、[同步 broadcast sources](../../formalization/reviews/synchronous-broadcast-sources.md)、[HotStuff 比较](../../formalization/reviews/hotstuff-family-proposer-finality-2026-09-28.md)、[wrapper 文献](../../formalization/reviews/smr-protocol-wrapping-literature-2026-09-28.md)、[CFT 排班文献](../../formalization/reviews/l1-scheduled-cft-rotation-2026-09-28.md)。本次整理没有重新做外部文献检索或核实部署参数；这些条目保留原笔记的检索日期与假设。

## 7. 后续汇报可采用的叙述顺序

1. 问题起点：拆分 sequencer 与 proposer，如何提供被 L1 授权的持续 stream？
2. 先定承诺：哪些输出允许重组，哪些跨 region 也必须保留？
3. GeoBFT 启发与区域快速确认：快路径之外还有什么成本？
4. LMD-GHOST／FFG 路线：把最终确认上移，为什么仍然需要昂贵的全局协调？
5. BFT 内层／CFT 外层路线：为什么逻辑进程封装有实质义务？
6. 最新委员会路线：哪些决策可以低频化，正常轮换如何保持简单？
7. 与 proposer／L1 的连接：证明来源、权限、finality 与忠实派生；列出尚未完成的接口和安全条件。

这是一份材料编排建议，没有生成 slides，也没有将未解决问题写成既定结论。
