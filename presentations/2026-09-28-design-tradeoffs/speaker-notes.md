# 源稿备注

以下内容按用户 v1 各页备注原样导出。

## Slide 1

建议用时：0.5 分钟
这次只讨论整体服务与架构选择。核心问题是怎样在可验证的授权和前缀承诺下，以低延迟持续向 proposer 提供流，并以合理成本完成轮换。协议中的投票、锁、阈值和算法会另行讲解。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/research-index/2026-09-28-weekly-design/tradeoffs-zh.md

## Slide 2

建议用时：2 分钟
先把 sequencing 视作独立服务。输入是交易和已生效的 L1 授权，输出是有序 mini-block 数据以及接收方可以独立检查的来源、连续性和确认依据。proposer 消费该流后负责执行、封装、DA/L1 发布。Sequencer 可为自己的排序目的执行，但该服务不强制其承担执行证明。架构图中的节点数量只示意角色，不表示委员会大小或容错阈值。图中的 M 是排序服务的抽象输出，不声称完全等同于当前原生 MegaETH header。跨 region 仍不可回滚是理想接口目标，不是所有历史方案均已提供的保证。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/meetup_record/meeting_0922_requirements.md
/Users/jin/projects/megaeth_sequenser_rotation/formalization/outline.md
/Users/jin/projects/megaeth_sequenser_rotation/research-index/2026-09-28-weekly-design/tradeoffs-zh.md

## Slide 3

建议用时：1.5 分钟
9 月 24 日的明确偏好是 latency、communication 与 rotation cost 优先，其次 weak liveness，再其次 censorship。后来进一步明确外部 P 的不可回滚确认目标。安全承诺应先说清楚：预览可变，已承诺不可回滚的前缀必须继承。这里的 weak liveness 关注流增长，不保证每个用户交易在固定时间内进入流。故障恢复与惩罚归责也应分别处理。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/meetup_record/meeting_0922_requirements.md
/Users/jin/projects/megaeth_sequenser_rotation/research-index/2026-09-28-weekly-design/chat-index.md

## Slide 4



## Slide 5

建议用时：2 分钟
左上是正常出流，右上是同一区域内换生产者。下方分别是常规地域交班与事故恢复。区域内换人需要区分计划任期结束和故障检测后接替。正常跨区在最新方案里由固定 batch quota 触发，下一地区先接收已 finalized 的锚点。右下委员会/L1 是最新候选架构的事故控制路径，图仅表示依赖关系，不表示故障区域必须在线授权退出。事故恢复还需要独立的已确认数据与安全截止条件。非活动区恢复只影响未来资格，不打断当前任期。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/research-index/2026-09-28-weekly-design/tradeoffs-zh.md
/Users/jin/projects/megaeth_sequenser_rotation/formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md

## Slide 6

建议用时：1.5 分钟
这张图只讲测量口径，没有按比例绘制协议时延。三批可同时处于确认流程，单批次的等待和连续出块间隔因此不同。P 的确认还要包括数据与证据到达。比较两条方案前必须固定同一确认层级，不能拿可撤回 QC 对比不可回滚 receipt。通信同时记消息数与字节数，另记跨区域数据与 L1 证明成本。Batch accumulation 与队列时间也不能藏在 proposal-ready 的计时起点之外。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/formalization/reviews/hotstuff-family-proposer-finality-2026-09-28.md
/Users/jin/projects/megaeth_sequenser_rotation/research-index/2026-09-28-weekly-design/tradeoffs-zh.md

## Slide 7

建议用时：2 分钟
最初从 GeoBFT 获得启发：将 Byzantine coordination 放在局部小集群，再进行跨区传播、共同采纳和轮换。本图画的是本项目的单 active region 适配，不是 GeoBFT 原论文逐项结构。历史具体实现包含区域 fast path/completion 和全局报告一致性加 FFG。需要承认每一层都承担真实的确认、数据保存和恢复义务。区域原语之后比较了同步 broadcast、PBFT 和 HotStuff family，这里只保留架构层。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/formalization/outline.md
/Users/jin/projects/megaeth_sequenser_rotation/formalization/reviews/consensus-sources.md
/Users/jin/projects/megaeth_sequenser_rotation/formalization/reviews/main-v1-cross-region-and-virtual-smr-2026-09-28.md

## Slide 8

建议用时：2 分钟
LMD-GHOST 负责选择本地暂定头，FFG 负责较慢的 checkpoint 最终确认。实际草稿还需要全局报告一致性先构造共同目标，因此 FFG 并没有消除全局 agreement。更低频的全局层可摊薄控制开销，同时延长最终确认等待，并留下更长的暂定尾部。该设计允许 preview 重组，不能拿 preview latency 与其他方案的永久确认直接比较。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/formalization/main.tex
/Users/jin/projects/megaeth_sequenser_rotation/formalization/reviews/lmd-ffg-redesign/README.md
/Users/jin/projects/megaeth_sequenser_rotation/formalization/reviews/ffg-composition-advisory.md

## Slide 9

建议用时：2 分钟
需要复制的是整个外层进程的合法行为，不仅是业务交易流：输入认证、协议状态、计时器、持久承诺、外发消息、重传和恢复。一个恶意 regional representative 不能绕过内部状态独自对外发承诺。内层一次共识不能简单视为一条便宜物理消息。Steward 和 Nysiad 是过去检索到的相关先例，说明路线可行而非没有研究。L1 指定谁接班也不替代日志恢复与激活。我们后来选择先避免通用 wrapper 的成本。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/formalization/reviews/smr-protocol-wrapping-literature-2026-09-28.md
/Users/jin/projects/megaeth_sequenser_rotation/formalization/reviews/l1-scheduled-cft-rotation-2026-09-28.md

## Slide 10

建议用时：2 分钟
这是本周最后选定的架构方向。正常轮换在固定数量的应用 batches 完成后沿预定义列表选择下一 live region，旧 finalized anchor 是启动新区的前提。委员会维护业务 entry：batch height 与 region status map。只有当前 active region 被标为 unavailable 才触发强制轮换。非活动区恢复只改变资格。L1 虚线表示已生效的授权/状态依据，不表示正常每次交接都必须新增一笔 L1 交易。委员会仍须一致地认证 entries，并保证事故 cutoff 与已确认输出兼容。完整组合安全与恢复仍有实质工作。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md
/Users/jin/projects/megaeth_sequenser_rotation/formalization/proposals/v3-committee-rotation-2026-09-28/README.md

## Slide 11



## Slide 12

建议用时：1.5 分钟
这是源共识域到 L1 的可验证输出接口，因此类似跨链验证问题。P 需要证明自己发布的数据忠实对应源 stream 及其 authority/rotation history。曾考虑让委员会进一步认证 stream tip，可能使来源认证更集中，但也增大了内容与 finality 的信任依赖。最新选择将委员会限制为状态与高度，内容由区域 final receipt 支持。高度本身不能证明不存在更高的已确认后缀，事故截止完整性仍需机制。Fraud proof 或 validity proof 是候选实现接口，当前没有 gas 或 prover 性能结论。执行正确性与 DA 可用性不由排序签名自动推出。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/meetup_record/meeting_0916_response.md
/Users/jin/projects/megaeth_sequenser_rotation/formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md

## Slide 13

建议用时：1.5 分钟
本表是对过去一周设计讨论的定性归纳，不是测量排名。必须在相同接受层级和可比假设下比较。A 的区域确认在旧实现中仍可能属于跨区可截断 preview；B 明确把强保证放到全局层；C 要先证明区域封装契约；D 以专门状态机制替代通用外层共识，但其控制信任、cutoff 和数据持久性义务都需要写明。不能仅凭多签名称宣称去掉了共识成本。
资料来源（本地设计记录）：
/Users/jin/projects/megaeth_sequenser_rotation/research-index/2026-09-28-weekly-design/tradeoffs-zh.md
