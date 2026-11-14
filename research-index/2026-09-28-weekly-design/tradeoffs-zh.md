# 设计 tradeoffs：指标、运行场景与方案取舍

日期：2026-09-28。依据过去一周的 [资料索引](README.md)、[Codex 对话记录索引](chat-index.md)、区域协议和最新已提交的 status-entry 大纲。本文归纳设计目标与选择，不修改协议，不将候选方案的理论分析写成性能实测或已完成的组合证明。

## 1. 最核心的优化目标

**在承诺清楚的安全与授权边界内，让正常 stream 尽可能快、日常轮换尽可能轻；把较重的协调和等待留给低频事故，同时控制 proposer 在 L1 证明源流与轮换合法性的复杂度。**

9/24 的优先级表达是：低延迟／低通信开销与低 rotation cost 优先，其次是 weak liveness，再其次是 censorship。9/28 又把确认指标明确为“外部 proposer P 收到可验证、不可回滚的确认”，并把正常跨区轮换列为日常重点，允许罕见整区故障恢复等待多个 L1 确认阶段。

这些记录可归纳为下表；其中“最低安全契约作为约束”是本次整理的表达，不是把早期讨论改写成早已确定了全系统 finality。

| 地位 | 关注点 | 具体含义 |
|---|---|---|
| 先明确的约束 | 授权与接受层级 | 谁可生产；什么只算 preview；什么一旦确认就必须被后继继承 |
| 首要性能 | 正常路径延迟、持续出流能力、常态通信成本 | 高频生产不能为低频事故逐批支付过重的全局协调成本 |
| 同样优先 | 区域内接替与正常跨区交班 | 正常轮换是服务的一部分，应尽量短暂停顿、少量补状态 |
| 必须提供，但可采用较慢路径 | 整区失效后的恢复 | 允许更长检测、委员会处理与 L1 等待；仍须有数据、继承和进展条件 |
| 新近突出的系统成本 | 可供 L1 验证的证据复杂度、委员会职责 | 共识越复杂，忠实 rollup 的证明接口也可能越复杂；这是需要单列评估的设计成本 |
| 当前后置 | 逐交易纳入、抗审查、排序公平、精细激励 | 本周 weak liveness 是 stream 继续增长，不自动保证每笔交易被纳入 |

来源：[9/22 需求 §4–5](../../meetup_record/meeting_0922_requirements.md)、[对话中的设计转折](chat-index.md)、[P 的 finality 目标](../../formalization/reviews/hotstuff-family-proposer-finality-2026-09-28.md)。

## 2. 我们真正需要比较的指标

| 指标 | 测量起点与终点 | 为什么单独计量 |
|---|---|---|
| P 的 confirmation / finality latency | 完整 proposal 发出 → P 获得完整数据及可验证的指定层级确认 | 区域内部有人 commit，不代表外部 P 已拿到证据；区域 finality 也不自动跨区永久有效 |
| 用户实际等待 | 交易进入系统 → 获得所承诺的确认 | 包含 batch accumulation、排队等开销，不能直接用 proposal-ready 的协议数字替代 |
| Pipeline block / batch interval | 稳态相邻新 proposal 或相邻 final receipts 的间隔，明确选哪一种 | 多个确认 timer 可以重叠；低间隔不等于每批都同样快地 final |
| Sustainable throughput | 可持续处理的应用数据量／时间 | 依赖 batch 大小、带宽、验证能力和负载；block time 只是其中一项 |
| Rotation interruption | 约定交接／故障事件 → 新 leader 或新区首个可接受输出 | 区分首次 proposal、首次 regional receipt、首次全系统永久输出 |
| Communication | 区域／跨区消息数、字节数、往返／串行跳数 | 证书聚合可降字节；不必然减少消息数或关键路径轮数 |
| State-transfer / retention cost | 接班所需缺失数据、证据、持久状态及长期保存量 | 已有 tip hash 不等于数据已经可恢复 |
| L1 / proof-system cost | 提交与验证 authority、finality、rotation、ordered data 所需成本 | 分开记证据大小、DA 字节、证明生成和 L1 验证；尚无具体系统时不能声称常数 gas |
| Trust / model complexity | 区域故障界、同步假设、委员会与数据持有义务 | 减少协议消息有时来自更强的假设或更窄的保证 |

例如当前 [v2 §6](../../formalization/main_v2.tex) 把区域内部 good-case commitment 写为 `δ_R + 2Δ_R`，而 ready batches 的 pipeline interval 约为 `2δ_R`。这里 δ_R 是实际区域消息延迟，Δ_R 是模型使用的延迟上界。两者并不矛盾，因为 timer 等待可以重叠。

P 的时间还要计入数据与 final receipt 的取得：直接收 endorsements 与先由 collector 聚合再发送是不同路径。[同期复杂度分析](../../formalization/reviews/main-v3-2026-09-28/complexity-assumptions.md) 专门指出了这个区别；它是未提交的分析材料，不作为新版本已完成的定理。

## 3. 优先分析的四类运行场景

| 场景 | 我们最关心的性能 | 愿意承担的成本／约束 | 需要避免的混淆 |
|---|---|---|---|
| A. Ideal in-region sequencing | P 的确认延迟、pipeline interval、每批通信和证据量 | 在已授权的区域内持续生产，允许同步模型和流水线 | 不把 mini-block cadence、QC 到达、regional finality、L1 finality 当成同一事件 |
| B. In-region leader rotation | 换人停顿、锁／前缀继承、首个新确认的时间 | 本地协调、view-change 证据；故障情形另有检测与失败后继 | 计划任期结束和故障触发应分别计；不能从 quit proof 起算却称为从宕机起算 |
| C. Regular cross-region rotation | 旧区末批到新区首批的间隔、证据与状态传输、摊销吞吐损失 | 已接受“旧 anchor finalized 后再启动新区”；正常切换使用预设授权，不逐次等新 L1 事件 | 旧末批尚未 final 和 final receipt 已就绪是两个起算点；新区 bootstrap 和首批确认要另计 |
| D. Accident cross-region rotation | 从整区失效到恢复已承诺级别的 stream，以及能否保留确认历史 | 可等较长的委员会／L1 流程，优先保证 cutoff、旧权限失效和数据恢复 | “可接受几十分钟到一小时”是容忍度，不是已有恢复上界；等待本身不恢复丢失的数据 |

其中 B 可展开为正常 local term end、沉默 leader、equivocation／blame；C 是正常 quota 到期，不能把“没有进展的 timeout”也含混称为同一种正常轮换。

非活动 region 上线／下线属于状态维护成本。最新决定是：它不打断当前 active region，不重置 quota；已有交接若在等 successor，则恢复资格可能使其继续。[status-entry 大纲](../../formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md)

## 4. Safety 的取舍发生在“承诺什么、何时承诺”

早期曾讨论是否可把 safety 交给下游 L2。后续设计逐渐明确：可以缩小 sequencing service 承诺的范围，但只要对外称某段 stream 已不可回滚，换 leader／换 region 就必须保留它。

| 保证 | 在本系统中的地位 | 不能由什么代替 |
|---|---|---|
| Authorization / provenance | 必须能验证指定位置的 producer／region 合法 | 某个有效签名不自动证明该身份此时有权生产 |
| Regional prefix safety | 区域 session 内已确认历史不冲突 | 仅有 latest head 或 raw vote QC 不必然等于 final receipt |
| Cross-region prefix inheritance | 若给 P 全局永久承诺，就必须跨区保留 | L1 指定新 owner 不自动证明它继承了全部已确认前缀 |
| Faithful rollup | P 发布的有序数据与源 stream、轮换历史一致 | 只验证几笔交易 membership，不能排除删改、重排和拼接 |
| Execution correctness / DA / L1 settlement | 下游独立接口与保证 | 排序委员会多签不能自动证明执行正确或数据可用 |

因此一条低成本路线可以选择“快 preview、慢 finality”；但评价时要承认它交付了较弱的早期承诺。Slashing 是对违规证据的处置，不能把已经允许回滚的确认变成技术上不可回滚。

## 5. 四条整体架构路线

### A. GeoBFT 启发：区域 BFT，再做跨区协调

**获得什么：** 地理局部性，区域确认与区域内接替有明确基础；批量处理可摊薄确认开销。项目探索的单 active region 是从 GeoBFT 获得启发后的适配。

**付出什么：** 内外两层的确认、数据保存和交接义务都必须落实。项目历史稿的全局层运行 report agreement 和 FFG，正常时期也有成本。区域 fast path 之后还有 completion，不能只报两跳。

**核心 tradeoff：** 比较清楚的分层确认结构与故障处理，交换为重复的协调、证明和数据流转工作。

来源：[最初 outline](../../formalization/outline.md)、[来源差异](../../formalization/reviews/consensus-sources.md)、[v1 的跨区规则](../../formalization/reviews/main-v1-cross-region-and-virtual-smr-2026-09-28.md)。

### B. 区域 LMD-GHOST + 全局 FFG

**获得什么：** 区域本地可持续出流、选 head，不必逐 batch 完成共同 DECIDED。允许暂定 suffix 变化，可以减轻本地串行确认约束。

**付出什么：** P 更早看到的 stream 是可重组 preview；强保证要等全局层。实际设计仍先对 reports 达成一致再执行 FFG，因此全局 agreement、数据与 quorum 成本依然存在。全局 checkpoint 越低频，控制开销越容易摊薄，但最终确认等待和暂定尾部也更大。

**核心 tradeoff：** 更早的可用 preview、较轻的局部决定流程，交换为较晚的持久承诺和更重的全局依赖。

来源：[main.tex](../../formalization/main.tex)、[LMD/FFG 改写](../../formalization/reviews/lmd-ffg-redesign/README.md)。

### C. 区域 BFT SMR → crash-only 逻辑进程 → 外层 CFT

**获得什么：** 外层以 region 为逻辑成员，在封装成立时可使用 CFT；把区域内部 Byzantine 问题和跨区 crash/recovery 分开。L1 排班可与 VR／Paxos 风格的指定 leader 结合，不必绑定 Raft 的竞选流程。

**付出什么：** 需要封装外层程序的输入、状态、timer、外发承诺、持久恢复和公平处理。只对交易流排序，并不能保证外层节点正确执行。外层一个逻辑 hop 可能包含内层共识／认证；跨区复制与恢复也仍存在。

**核心 tradeoff：** 通用的组合接口和较弱的外层故障模型，交换为封装复杂度、内外层交互成本，以及新的组合证明义务。

**本周选择：** 不是认为这条路线不可能；已找到直接研究先例。但用户随后认为其通用封装成本偏重，下一版转向委员会路径。

来源：[wrapper 研究](../../formalization/reviews/smr-protocol-wrapping-literature-2026-09-28.md)、[L1-scheduled CFT](../../formalization/reviews/l1-scheduled-cft-rotation-2026-09-28.md)。

### D. 区域 SMR + L1 排班 + 低频状态委员会

**获得什么：** 正常生产与常规轮换可依据已生效配置和区域证据运行；无需每批运行新增的全局 agreement，也无需把每个 region 编译成通用 CFT 节点。委员会业务收缩为 `(batch height, region→status)`。

**付出什么：** 对委员会状态历史、事故 cutoff、L1 生效次序与数据保留增加明确依赖。需要处理正常交接与事故更新的并发、共同 status snapshot、末批 Drain、旧 authority 失效，以及已确认但未上 L1 的后缀。不能再直接套用外层 CFT 的容错定理。

**核心 tradeoff：** 轻量常态路径和有限的委员会业务，交换为对低频控制机制的信任、较慢事故恢复与专门的跨区继承证明。

**当前状态：** 本周最后选定的提案方向，完整协议与端到端保证仍在形成中。低频委员会仍需一致地认证其 entries；multisig 这个名字本身不提供无成本的 agreement 或 liveness。

来源：[最新已提交大纲](../../formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md)。

## 6. 区域原语的比较，是另一条设计轴

| 区域方案／研究方向 | 想优化什么 | 代价／限制 | 本周角色 |
|---|---|---|---|
| 同步 `2δ`-BB + completion | 内部 good-case 快速决定 | 同步上界、锁传播与 completion；历史实现正常路径也运行较重的收尾 | 早期具体基线 |
| 完整区域 PBFT | 稳定的 commit／view-change 结构 | 更多阶段与 all-to-all 通信；区域确认仍需外层继承才能全局永久 | 对照分支 `main_PBFT` |
| HotStuff family | 阶段数、证书路径、通信与 handover；P 作为外部 learner | 少 phase 不等于少单程 hop；不同 variants 的 finality 与恢复假设不同 | 比较研究 |
| Sync HotStuff | 连续流水线、同步区域内的明确提交与恢复规则 | 提交等待依赖 Δ；v2 的具体实例仍为 all-to-all，不应宣称线性通信 | v2 采用的区域基线 |
| Binary BA／weighted BA 替代 completion | 避免为了决定一个结果重建完整输入向量 | 需权重适配、完整 witness retrieval；不必逐执行复现旧选择函数 | 尚未集成为最终协议的优化候选 |

历史 v1 的 completion 成本为 `O(n³)` 消息；这来自其具体 authenticated-broadcast 组合，不是“所有 BFT 都 cubic”。v2 中 `O(n²)` 消息也不等于 `O(n²)` 字节：全转发 `O(n)` 大小的证书列表仍可产生 cubic control bits。[completion 笔记](../../formalization/reviews/main-v1-completion-literature-2026-09-28.md)、[v2 成本](../../formalization/main_v2.tex)

所以区域方案的选取至少要同时看：**最早安全确认、持续实例间隔、正常通信、失败后恢复、可向 P 导出的证据**。只按一个 good-case 常数排名会遗漏本周最在意的部分。

## 7. 若干更具体、已经讨论过的取舍

| 选择 | 一端的收益 | 另一端的成本 | 本周倾向／状态 |
|---|---|---|---|
| 每个 mini-block 确认 vs batch 确认 | batch 摊薄投票、签名和协调 | 累积等待、更粗的确认粒度、更多在途数据 | 倾向批量；“每 10 个”是讨论例子，不是最终常数 |
| 长任期 vs 高频轮换 | 长任期减少交接摊销成本 | 更久集中生产权；服务机会与故障规则仍须设计 | 同一 leader 连续服务，正常到期与故障接替分开 |
| 新 region 提前流水 vs 等旧区 final | 提前启动可隐藏交接等待 | 跨 session 条件依赖、回滚与外部确认规则更复杂 | 最新决定接受等待，删除 conditional session |
| 委员会认证 stream tip vs 仅管状态 | 前者可提供更集中、可能更紧凑的 L1 来源认证接口 | 增加委员会对内容／finality 的权力和责任；仍需证明发布数据与 tip 绑定 | 曾考虑前者；最新保留状态＋高度，区域 witness 认证内容 |
| 直接验证区域证据 vs 以证明系统封装 | 封装可改变 L1 verifier／证据接口 | 证明生成、数据见证与实现复杂度转移到别处 | 先写派生关系，fraud／validity 机制待定 |
| 全广播 vs leader push + pull repair | 后者减少正常情况下冗余数据分发 | 缺数据时补拉、检测与额外等待；需要可靠持有者 | 采用 push/pull 的数据传输方向，不据此取消共识消息 |
| 更早永久确认 vs 更少跨区复制 | 早确认改善 P 体验；少复制降低常态 WAN 成本 | 若唯一持有区消失，已确认数据可能无法恢复 | 必须明确复制／DA／持久性前提，不能同时默认都免费 |
| 更短故障 timeout vs 减少误判 | 较短 timeout 可更早恢复 | 抖动导致误判、无效切换与额外协调 | 故障检测与 slashing 分开；同步假设下才解释 deadline |
| 委员会签高度 vs 签分支 root | 只签高度缩小业务范围 | 高度本身不足以识别内容或证明不存在更高 final suffix | 用独立区域证据解析 anchor；closed-cut 问题仍需解决 |

这里“委员会使 L1 认证更紧凑”只是研究动机；未经具体 scheme 和 witness 设计，不能推导“不再需要保存区域证据”或“gas 已降低”。

## 8. 用于后续方案评估的统一表

比较任何两条路线时，先固定接受层级、区域／全局故障模型、网络前提和数据持久性，再填以下项目。数值未知时标“待定”，不以论文宣传数字混填。

| 每条方案都要填写 | 正常出流 | 区域内轮换 | 正常跨区轮换 | 事故跨区轮换 |
|---|---|---|---|---|
| 起算点与首次可接受输出 | proposal-ready → P | 计划边界／故障／quit proof 分开 | 旧末批 proposal 或旧 receipt 二选一 | 整区失效 → 新区首个指定层级确认 |
| 关键路径等待 | proposal、投票、timer、证据到达 | 检测、view change、继承、首批确认 | 末批 final、传输、bootstrap、首批确认 | 检测、safe cutoff、委员会、L1、数据恢复、首批确认 |
| 常态／单次通信量 | 区域、跨区、向 P 分列 | 控制证据与缺失数据分列 | 交接证据与缺失前缀分列 | 同上，另计委员会与 L1 |
| 对持续吞吐的影响 | 稳态 batch interval 与负载容量 | 停顿如何摊销 | quota 多大、多久交接一次 | 频率假设与尾部中断长度 |
| 安全与信任前提 | 哪些确认不可回滚 | 哪些锁与签名记录持久保留 | 旧确认如何被新区继承 | 谁决定截止、谁仍持有数据、旧区如何被隔离 |

本周逐步形成的选择可以具体表达为：**优先保住快速持续生产与低成本日常交班，接受显式的同步／持久性条件和较慢的事故恢复，限制委员会业务范围，并把跨区安全与 L1 忠实派生写成必须完成的独立义务。**
