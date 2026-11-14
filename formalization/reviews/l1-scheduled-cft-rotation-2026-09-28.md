# L1 指定轮换规则的跨区域 CFT：文献与设计判断

日期：2026-09-28。接受用户前提：每个区域已经被封装成一个 crash-only／可安全恢复的逻辑进程。本笔记不重新证明这一封装，专门讨论外层 CFT 的预定 leader rotation。它是研究建议，不是对当前 `main_v1.tex` 的修改或新协议的完整证明。

相关背景：[已有 virtual-region 分层说明](main-v1-cross-region-and-virtual-smr-2026-09-28.md)；[P 不可回滚确认目标](hotstuff-family-proposer-finality-2026-09-28.md)。

## 1. 结论

**可以预先指定 CFT 的下一任 leader。CFT 安全性不要求通过 Raft 式竞选来决定 leader 身份。** 最直接的文献是 *Viewstamped Replication Revisited*，最贴近外部 L1 管理配置的文献是 *Vertical Paxos and Primary-Backup Replication*。

需要分清三个决定：

1. **Designation**：谁拥有某个 view/epoch 的领导权。L1 可完全规定。
2. **Activation**：何时已有足够证据允许该 leader 开始新的写入。需要 fence 旧 view 并恢复状态。
3. **Commit**：哪些 stream entries 已经不可回滚。由复制协议的提交条件决定。

确定性 leader 映射不能替代后两项。恢复 quorum 的作用是确定可继承历史与限制旧 view 的新接受行为，而不是在候选人之间自由挑选赢家。

## 2. 文献地图

| 文献 | 原文机制 | 与需求的匹配 |
|---|---|---|
| Liskov–Cowling, *Viewstamped Replication Revisited*, 2012 | primary 由 view number 和配置按 round-robin 决定；view change 给指定 primary 恢复日志 | 固定 region 集合、按任期轮换：最直接 |
| Lamport–Malkhi–Zhou, *Vertical Paxos and Primary-Backup Replication*, 2009，修正版 2019 | 外部 configuration master 指定每个配置的 acceptors 和 leader | L1 直接管理 authority／成员集合：架构最接近 |
| Lamport, *Paxos Made Simple*, 2001 | leader 选择与 acceptor 的安全规则可分离；新 leader 用 Phase 1 恢复已有选择 | 固定成员下，可设计 L1-authorized ballot owners |
| Mao–Junqueira–Marzullo, *Mencius*, OSDI 2008 | 按 instance number 静态分配 coordinator；skip/revoke 处理慢节点和故障 | 若轮换单位是 log slot／block，而非一个 region 独占整段任期 |
| Ongaro, *Consensus: Bridging Theory and Practice*, 2014，§11.2.2 | 直接比较 Raft、Paxos、VRR 的选主与日志恢复 | 解释为什么指定一个落后节点接班仍可安全 |

### 2.1 Viewstamped Replication Revisited

[MIT 记录与原始入口](https://dspace.mit.edu/entities/publication/80846d94-fcd3-40e6-87fb-8d91fe99a5d1)，[论文全文镜像](https://codewisdom.io/static/papers/viewstamped-replication.pdf)。重点读 §4 开头、§4.2 View Changes、§4.3 Recovery、§8.1。

论文明确把 primary 作为 view number 与 configuration 的确定性函数。正常配置有 `2g+1` 个 replicas；view change 由指定的新 primary 收集 `g+1` 个 DOVIEWCHANGE（含自己），恢复安全日志并发送 STARTVIEW。选择日志时先比较最后一次 normal view，若相同再比较 op-number；不能简化成无条件选择最长日志。

这与 Raft 的重要区别是：**预定 successor 可以暂时落后，协议负责把日志补给它。** 论文原生支持 deterministic primary assignment；把 view-change trigger 改成 L1 授权的计划轮换，是本项目的适配，不是原文已经实现的 blockchain 功能。

工程阅读参考：[TigerBeetle VSR internals](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/vsr.md)。该实现还包含日志修复、落后 primary 主动放弃当前 view 等规则；它不是 L1-scheduled 的现成实现。

### 2.2 Vertical Paxos

[Lamport 的修正版全文](https://lamport.org/pubs/vertical-paxos.pdf)，[Microsoft Research 页面](https://www.microsoft.com/en-us/research/publication/vertical-paxos-and-primary-backup-replication/)。重点读 §2、§4.2–4.3。

外部 master 决定每个 configuration 的 acceptor 集合及 leader；数据复制协议负责从旧配置取得正确状态。Vertical Paxos II 将指定新配置与激活新配置分开：新 leader 完成必要 state transfer 后通知 master，再由 master 激活。Vertical Paxos I 允许状态迁移与新配置工作重叠，但可能需要向多个旧配置取回状态。

把 L1 映射到这个 master 是很自然的**架构类比**，不是文献对 L1 finality、链重组和异步链观察已有的组合定理。需采用共同的 canonical/finalized L1 授权视图，并规定旧数据取回与 activation 的条件。

如果只换 leader、不换外层 acceptor 集合，没必要为每次轮换引入完整 reconfiguration。固定成员的 VR／Multi-Paxos 更简单；Vertical Paxos 用于未来成员变更或更明确的外部 master 分工。

### 2.3 Paxos 的 leader 可以由外部指定

[Paxos Made Simple](https://lamport.azurewebsites.net/pubs/paxos-simple.pdf)，§2.4、§3。

Paxos 的安全来自 ballot 单调性、acceptor promises、quorum intersection 与恢复时继承可能已 chosen 的值。leader 选择主要影响 progress。固定配置下，可给 ballot 预分配 owner，并要求只有 L1 规则指定的 owner 才能启动该 ballot。

新 leader 仍需 Phase 1（或具有同等证明的预执行／交接），检查之前 accepted 的值。不能只读取显式标注为 committed 的条目：某值可能已被 quorum 接受，而提交通知尚未送达。

Paxos 的“proposer”是协议角色；本文的外部消费端 `P` 是 learner/client。两者不要因为名称相同而混淆。

### 2.4 Mencius：按 slot 轮换的直接先例

[OSDI 2008 会议页](https://www.usenix.org/conference/osdi-08/mencius-building-efficient-replicated-state-machines-wans)，[原文 HTML](https://static.usenix.org/events/osdi08/tech/full_papers/mao/mao_html/index.html)，重点 §4.2–4.5。

三个节点时，节点 0 负责 slots `0,3,6,...`，节点 1 负责 `1,4,7,...`，节点 2 负责 `2,5,8,...`。原文要求全体知道 coordinator assignment；正常 coordinator 有值就提出，没有值可 skip。故障时通过 revoke／更高 Paxos round 解决未完成实例。

这比“给 Raft 调整选举优先级”更接近按预定规则分配 proposal rights。但 Mencius 是多个节点拥有交错实例、可并行提出的设计；不是默认仅有一个 region 在整个时间窗口拥有所有新排序权。若所有后续 slot 都必须等某个失效 region 的 slot，还需要它原文的 skip/revoke 来解除阻塞。故障恢复的执行者选择也须按本项目的 L1 规则适配。

### 2.5 Raft 作者自己的比较

[Ongaro 博士论文](https://ongardie.net/var/blurbs/pubs/dissertation.pdf)，§11.2.2、Table 11.1（印刷页 151–153）。

该节明确列出 VRR 的 `leader(v)=v mod n`，并对比两类实现：Raft 在选举时要求获胜者拥有足够新的日志；Paxos／VRR 可先确定新 leader，再传输必要日志。预定 leader 不可用或状态落后会增加接班延迟，但不意味着必须换成竞选机制。

Raft 可以做 leadership transfer，所以不能声称它绝对不允许指定 successor。只是“L1 的规定顺序决定下一任，无需候选人竞争”这个研究入口，VRR 更直接。

## 3. 适合本项目的模型

令 `E_i` 表示 region `i` 的 crash-only virtual entity，外层成员固定为

\[
\mathcal R=\{E_0,\ldots,E_{m-1}\},\qquad m=2g+1.
\]

普通等权 CFT 使用 `g+1` 的读／写多数 quorum，容忍最多 `g` 个 region 不可用。这里 `g` 是外层区域故障数，与内部 BFT 的 Byzantine fault bound 不同。若外层也采用权重或 flexible quorums，需要另给交集和 availability 条件。

L1 安装配置和一个共同确定的函数

\[
\ell(c,e)=\mathsf{Schedule}_{\mathrm{L1}}(c,e),
\]

其中 `c` 是配置编号，`e` 是逻辑轮换 view。可以是循环顺序、指定列表或提前确定的权重调度。若状态会影响选择，相关输入必须是共同认证状态，不能各节点按自己的在线观察得到不同 owner。

region 自己的 local leader 不等于外层 primary。内层换 local leader 仍代表同一个 `E_i`；只有外层 view 前进才改变 active region。

## 4. 一个协议适配轮廓

以下不是完整算法，而是需要由选定 VR／Paxos 原语实现的接口。

1. **授权。** 节点验证 L1 配置与 `owner(c,e)`；只有该 owner 能在此 view 提出新外层日志条目。
2. **进入新 view。** 计划任期到期、L1 显式命令或规则授权的 timeout/skip 触发 `e→e+1`。各实体单调记录 view，并停止接受旧 view 的新写入。状态须跨区域内部换 leader／恢复保留。
3. **恢复。** 指定 successor 从合法 quorum 收集外层协议状态；采用 VR 的日志选择规则或 Paxos 的最高 accepted-ballot 规则，而不是自行取最大高度／最大 commitIndex。
4. **补数据。** 取得所选前缀与必要 witness，补齐未决条目、缺口和去重状态。恢复旧 chosen 值与生成新的业务提案要区分。
5. **激活。** 通过所选协议的 STARTVIEW／Phase-1-and-recovery 完成新 view 的安装，之后开始正常复制；对外导出绑定配置、view、位置和内容的 commit 证据。

旧 epoch fencing 的含义是阻止旧 leader 继续取得新的冲突接受；它不撤销旧 epoch 已 chosen 的结果。旧 commit 证明可能晚到，继承规则必须覆盖这种执行。

### 具体例子

三个逻辑区域 `A,B,C`，外层容忍一个区域 crash，quorum 为两个。L1 规定下个 primary 是 `C`。

- `A` 任期内，entry `x` 已在 `A,B` 上被外层协议稳定接受／chosen，并对 P 确认；C 还未收到 x。
- A 随后故障。
- C 收集 `B,C` 的 view-change 状态。B 提供必须保留的 x；C 先补齐它，再开始新条目。

整个过程中不需要在 B 和 C 之间竞选。即使 B 的日志更完整，L1 仍可规定 C 为 primary；协议做的是把必要状态交给 C。

如果 C 也故障，则必须执行事先规定的失败策略：等待 C 恢复、按规则进入下一 view，或等 L1 再授权。若规则要求每个被指定者都必须实际服务才能轮到下一位，指定者永久 crash 就会永久阻塞；任何 CFT 都无法消除这个直接限制。

## 5. 两种 L1 集成方式

### L1 只安装规则

L1 最终确认配置、轮换函数、任期／slot 规则及故障跳过条件。外层 CFT 根据这些规则执行有证据的 view transitions。每次正常 rotation 无需新的 L1 交易。

这与当前 `main_v1.tex` 120–140 行的“L1 安装 cyclic schedule 并委托带证据的后续转换”在授权架构上接近，但外层协议会从现有 report-AB＋FFG 改成 region-level CFT，是新的设计。

### L1 逐次指定 epoch

L1 每次发布或最终确认 `(configuration, epoch, owner, membership)`。外层只接受这次授权，随后完成恢复和激活。这个方案更接近 Vertical Paxos master，但若每次交接都等 L1 finality，L1 延迟就进入交接关键路径。

L1 先公布“下一任是谁”并不等于它在公布瞬间就已具备安全接管的状态。提前预热 successor、复制历史和缓存配置，可以减少交接延迟；取消安全恢复不在这个优化范围内。

## 6. 与 P 的最终性目标的关系

只有 active region 内部决定了 `x`，不代表外层 CFT 已选择 `x`。若要求活动 region 永久不可用后其他 region 仍能继续，并保留 P 的所有不可回滚输出，那么在给 P 这类确认之前，必须存在可由后继恢复的区域外状态/数据依据。

采用 VR／Multi-Paxos 的普通实现时，这是跨区域复制 quorum 的工作；若另用 L1/DA 或专门备份路径，必须明确它们提供什么持久性与恢复保证。L1 的 authority mapping 本身不会保存丢失的 stream bytes。

因此此路线的收益是：外层可以使用 CFT 的多数 quorum 和恢复协议，leader 身份严格服从 L1 规则；它不会自动消除 WAN 复制或状态交接成本。内层 SMR 确认每个外层 promise／accept 消息的延迟也需要计入。

## 7. 当前选择建议

- **固定 region 集合、一个 active region 服务一段任期：** 首选 VR Revisited 的确定性 primary 模型，外接 L1 schedule 和授权触发规则。
- **L1 还负责变更成员集合、备份配置和激活：** 重点采用 Vertical Paxos 的 configuration master 模型。
- **每个 block/slot 按预分配 region 轮换且允许并行：** 研究 Mencius；不要把它直接当成单一 primary 的按时轮换。

需要在下一版设计中明确的主要选择是：按时间还是按已提交 slot 推进；故障能否跳过；L1 是只发布规则还是逐次参与；P 的全局确认究竟发生在外层哪一个 commit 条件。一个只写 `leader(e)=schedule(e)` 的版本尚未定义完整的 handoff protocol。
