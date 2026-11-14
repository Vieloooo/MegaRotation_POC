# 在 BFT SMR 中运行协议 f：虚拟进程、故障模型转换与文献

日期：2026-09-28。问题：能否系统地把协议进程 f 放到一个 SMR group 中，使其向其他进程表现为一个 non-Byzantine、允许 crash/recovery 的实体？本文给出直接文献、建议的接口与需要证明的边界。未实现或认证一个通用编译器，未修改 main_v1。

## 1. 这是已有研究方向

最贴切的检索词：**fault-model transformation / protocol transformation / Byzantine-tolerant protocol compiler / replicated process / hierarchical Byzantine replication**。

两个尤其直接的先例：

- **Nysiad: Practical Protocol Transformation to Tolerate Byzantine Failures**，Ho、van Renesse、Bickford、Dolev，NSDI 2008。[会议全文](https://www.usenix.org/legacy/event/nsdi08/tech/full_papers/ho/ho_html/nysiad_html.html)。它把 crash-tolerant 协议的进程复制到 guards，验证输入和输出，并以 credits 限制选择性处理。目标是正确、公平地处理输入，或停止产生输出。normal path 使用 ordered reliable broadcast，不需要为每步执行完整 consensus；它允许被保护进程停止。guard/monitor 图和 epoch service 是具体构造条件，不能将它直接视为任意不重叠区域的现成插件。
- **Steward: Scaling Byzantine Fault-Tolerant Replication to Wide Area Networks**，Amir et al.，DSN 2006 / TDSC 2010。[作者项目页与正式版本链接](https://jhu-dsn.github.io/steward.html)，[作者保存的详细技术报告](https://cnitarot.github.io/papers/scalable_byz.pdf)。它采用站点内 BFT、站点间 Paxos-like benign-fault-tolerant 协议，外发消息由站点级 threshold signature 认证。其每站点故障界和多数站点可用条件，与“全网仅一个 Byzantine 总量上限”不同。它还区分需要本地 ordering 的操作与可以更轻量认证的消息。

Steward 是本项目“region BFT → region virtual process → cross-region CFT”最直接的系统先例；Nysiad 是“给定协议 f，系统地改变其容错模型”最直接的阅读入口。这些是研究和设计依据，不是对历史代码当前可部署性的判断。

## 2. 输入对象应是一个进程的状态机

令协议为 `F=(f_1,...,f_m)`，其中 f_i 是第 i 个逻辑进程的程序。每个区域 G_i 复制 f_i，模拟**同一个外层身份 i**。不能让每个物理副本都作为一个独立的外层 Raft voter，否则得到的是另一个成员模型。

推荐先把 f_i 写成确定性 transition：

\[
(s_{k+1},\mathcal O_k,\mathcal T_k)=f_i(s_k,e_k).
\]

- s：全部协议状态。
- e：一个显式输入事件。
- O：本次状态转换产生的逻辑 output commands。
- T：设置/取消定时器等受 wrapper 管理的指令。

一种示意接口：

```text
Event = Receive(sender, message_id, payload, evidence)
      | TimerExpired(timer_id, generation, evidence)
      | RandomChoice(label, value, evidence)
      | ClientRequest(client_id, request_id, payload)

Command = Send(destination, payload)
        | Reply(client_id, request_id, result)
        | ArmTimer(timer_id, generation, duration)
        | CancelTimer(timer_id, generation)

step(state, event) -> (new_state, commands)
```

`step` 不直接读 wall clock、不直接采样、不直接发网络包、不执行未封装外部副作用。SMR 排序的是足以确定状态演化的事件，不必是每条 CPU 指令。代码版本、配置与初始状态也是共同输入的一部分。

## 3. 一个保守、易审查的 wrapper

下面是建议的通用基线设计，不声称是任何一篇论文的逐字算法或现成定理。

```mermaid
flowchart LR
    A["消息 / timer / client input"] --> B["验证事件来源与合法性"]
    B --> C["区域 BFT SMR：排列并提交事件"]
    C --> D["所有正确副本执行同一 step"]
    D --> E["持久状态与 outbox"]
    E --> F["为正确的输出生成区域证明"]
    F --> G["重传 / relay / cluster-sending"]
    G --> H["对端验签、检查身份并去重"]
    H --> I["交付给对端逻辑进程"]
```

### 3.1 输入：一致性之外还要合法性

对 Receive，需要验证原逻辑发送者、发送目标、消息编号、配置、消息内容和原发送 group 的证据。Byzantine ingress 不能凭空构造一个 VoteGranted，然后仅凭本区域对该输入达成一致就当作真实外部消息。

对重复消息，可以在 receiver 的复制状态中维护去重信息。只有去重记录也随 crash/recovery 恢复，才能避免重启后重复触发非幂等状态转换。若 f 原本允许重复，则 wrapper 可以保留这种 channel 语义；不要无说明地改变它。

### 3.2 执行：只能对已承诺的事件产生可接受的外部效果

所有正确副本按相同日志顺序执行 f_i。输入已最终确定、必要数据已持久保留后，才能释放对应的外部效果。状态和 outbox 应可从同一个 committed event prefix 恢复；不必每步都写完整 snapshot，但不能先 ACK、后丢失 ACK 所承诺的状态。

**对输入顺序的 QC 不自动证明任意声称的 output 是 f 的正确结果。** 可以让副本在执行之后为 output 签名，或采用能证明执行关系的其他方案。正确副本只认证从已提交状态转换推导出的具体输出。

### 3.3 输出：把协议身份与物理转发者分离

给每个逻辑输出分配唯一标识，例如：

\[
\mathsf{oid}=(\mathsf{configuration},i,k,j),
\]

其中 k 是已提交事件的位置，j 是该事件产生的第 j 个 output。认证语句还绑定程序版本、目标 destination、message type 与 payload digest。

任何物理节点都可 relay 一个合法 output；receiver 接受的是 group 证明，而不是信任 relay 的个人身份。leader 隐瞒消息时，其他 holder 应能接续发送。

在 n_i=3t_i+1 的常见模型下，要求 2t_i+1 份 post-execution endorsements 是一种保守基线；这不是“执行结果认证的最小阈值”定理。若底层已证明唯一最终状态且 honest 仅在 final execution 后认证，某些系统采用 t_i+1 matching replies。确认语义、durability 和重配置要求决定阈值，不应把 output attestation、ordering quorum 和 availability quorum 混为一谈。

### 3.4 传输：持久 outbox 与对端去重

若事件已提交但 sender 在发消息前 crash，恢复后从 outbox 重发。若发出后、记录 ACK 前 crash，也可能重发。因此应基于重复传输设计逻辑去重；不能只靠 sender 端承诺 exactly-once。

一个逻辑输出也可能面向多个 recipient。不同 recipient 可按 f 的要求收到不同消息；需要保证这些差异都是同一次合法执行的输出，而不是要求所有外发消息字节都相同。

外部系统若不能检查证据、去重或参与相应事务，SMR 无法自动让不可逆副作用 exactly-once。例如发送邮件、物理 actuator、第三方支付调用，需要各自的 adapter/幂等键或事务契约。

### 3.5 进展：不能只认证每条已经发出的消息

允许的网络延迟、暂时不可达与持续选择性丢弃是不同性质。一个 relay 若一直发送 heartbeat，却永久丢弃合法写请求，可能破坏上层 CFT 所依赖的公平处理和可靠通信假设。

需要 fair input admission、合理的 timer admission、可替换 relay 和输出重传，或者采用一种能迫使不公平进程停止可接受输出的专门转换机制。SMR 的 agreement 属性本身不等于每个合法请求最终被包含。

## 4. 两个容易漏掉的非确定性来源

### Timer

物理 timer 只能产生候选事件。接收进程必须检查 timer 是否确实已 armed、generation 是否当前、是否已被取消，以及采用的 timing/failure-detector 契约。将 TimerExpired 放进日志只解决“一起看见”，不证明 timer 没有过早触发。

可用虚拟 tick、经过验证的 timeout 事件或其他与目标协议相容的方法。安全和活性需要分开分析：Raft 的计时主要影响选举进度；有 lease/real-time deadline 的 f 则要求更严格的时间语义。批处理和区域共识带来的排队也会改变外层有效 timeout。

### Randomness

各物理副本独立随机采样会让 f 的状态分叉。应把 random choice 变成共同事件，或使用受控、可重放的虚拟进程私有随机源。

“大家使用同一个数”只保证一致，不保证该数无偏、不可预测或不同逻辑进程之间独立。若这些性质用于安全或活性，就需要另外的 randomness construction。秘密密钥也不能直接作为公开复制的普通状态；需要按所用密码学模型单独处理。

相关专门文献：Cachin、Schubert、Vukolić，**Non-determinism in Byzantine Fault-Tolerant Replication**，[作者论文](https://arxiv.org/abs/1603.07351)，重点区分普通非确定选择与密码学随机性/秘密。

## 5. 实际上如何把 Raft 放进去

可参考 [etcd-io/raft](https://github.com/etcd-io/raft) 的 core/IO 分离接口：网络消息和 tick 驱动状态机，状态机产出待发送消息、日志与状态变化，由外部集成层负责持久化和 transport。

在本项目的候选实现中：

```text
Region A 的所有 BFT 副本
    复制同一个 outer Raft node A
    相同 outer identity、状态、配置与事件序列

收到 RequestVote(B, term=7)
    -> 验证 B 的区域消息证明
    -> A 的 BFT SMR 提交输入事件
    -> 所有正确副本执行同一个 Raft transition
    -> 持久化 term/votedFor 与输出意图
    -> 认证 VoteGranted(A,B,7)
    -> 任意可用 relay 将消息发往 Region B
```

后一条 RequestVote(C, term=7) 若与前面的 grant 冲突，会基于同一个 votedFor 状态被拒绝；Byzantine relay 不能独自制造合法的第二份区域 grant。

**代码层发现。** 查询时 etcd/raft 的 [`resetRandomizedElectionTimeout`](https://raw.githubusercontent.com/etcd-io/raft/main/raft.go) 调用 `globalRand.Intn(...)`。因此，不能仅凭 README 的 deterministic core 描述，就在各副本里独立调用整个未经适配的库。该 random source 以及相关 hidden state 需要显式封装。选择 RawNode 风格的同步 step 接口有利于控制事件循环，但还不是现成的 BFT compiler。

代码链接跟随 main，可能变化；真正实现时应锁定 commit 并审计调用路径。本文仅验证了这个随机入口，没有审计整个 etcd/raft。

## 6. crash-only 的精确目标

对上层而言，应讨论**经过验证后被接受的行为**。底层恶意机器仍可发送垃圾包；receiver 把它们过滤掉，不代表物理机器真的不 Byzantine。

建议把目标定义为 non-Byzantine **crash-recovery virtual process**，并明确它允许暂停、重启、消息重复和重排中的哪些行为。不要把它写成附带完美故障检测的 fail-stop 节点。

安全侧的形式化目标可以写为外部 trace refinement：

\[
\pi_{\rm ext}(\mathrm{Exec}(\mathrm{Wrap}(f_i)))
\subseteq \mathrm{Traces}(f_i,\mathcal F_{\rm target}).
\]

其中 π 隐藏内部复制动作并应用约定的网络语义；目标故障类 F_target 必须明确。这是建议证明的契约，不是本文已经证明的通用结论。活性还需单独加入内层 SMR 的 progress、fair delivery/input processing、状态可恢复性和外层协议的环境前提。

以下情况不能自动变成 crash-only：

- 底层仅为 CFT SMR，但物理成员实际 Byzantine；
- BFT corruption threshold 被突破，攻击者能够生成区域证书；
- 同步型 SMR 的安全前提被违反；
- 已回复的持久状态在整体重启后丢失；
- 所有副本执行同一个错误版本的 f（复制不会修复共同的软件 bug）；
- 收件方仍信任未经过 group 验证的个人签名或无证据的输出。

## 7. 两条不同的工程路线

| 路线 | 内层目标 | 外层看到什么 | 合适场景 |
|---|---|---|---|
| 完整 BFT SMR 虚拟进程 | 在部分物理节点故障时仍持续执行 f | 正确的逻辑进程；组不可用时暂停/恢复 | 区域内部有冗余且希望区域尽量可用 |
| fault-transformation / Nysiad 风格 | 保证行为正确，允许整个受保护进程停下 | 正确处理或停止输出，交给已有 crash-tolerant 协议恢复 | 希望复用既有 CFT，允许更积极地牺牲单个逻辑进程的可用性 |

后者能够减少 normal-path agreement 工作，但有其独立 guard topology 和 fairness 机制；不是前者未经证明的性能优化。

还须核算外层 crash budget：若每个区域都允许存在一个 Byzantine primary，而转换器允许 primary 作恶时整个受保护进程停下，则可能有很多虚拟区域同时不可用。内层每区域的 Byzantine 上限，不自动给出“小于一半虚拟区域停止”的外层活性前提。

## 8. 本项目的建议

1. 先定义一个协议无关的 VirtualProcess 接口：Step、认证输入、提交/恢复、输出证明、可靠投递和 timer/randomness；不从具体 HotStuff 消息开始定义接口。
2. 内层只需适配这个契约。HotStuff、PBFT、Sync HotStuff 等只有满足选定安全与活性前提时才可替换，不能因都叫 SMR 就认为完全互换。
3. 外层先写一个普通 crash-recovery Raft/Paxos 状态机，用区域虚拟进程实例化其节点。区域 local leader 仅是可更换的执行/转发角色，不持有绕过区域证明的外层权限。
4. 原型先采用清楚的 commit-before-output 和可恢复 outbox；之后才根据 Steward 等工作合并认证、批量输入并减少不必要的 local ordering。优化必须保留先前接口语义。
5. 区分本地虚拟进程的一步已提交与外层 stream entry 已全局提交。前者不能直接给 P 跨区域永不回滚的承诺。

Generic wrapper 的额外代价包括本地输入排序、输出认证和可靠 inter-cluster delivery。外层一次网络 hop 不能直接按一条物理消息的耗时计算。

## 9. 建议阅读顺序

| 文献 | 最值得读的部分 | 回答什么 |
|---|---|---|
| [Steward，DSN 2006 / TDSC 2010](https://jhu-dsn.github.io/steward.html) | 站点抽象、local protocols、threshold-signed WAN messages | region BFT + outer CFT 是否有直接先例 |
| [Nysiad，NSDI 2008](https://www.usenix.org/legacy/event/nsdi08/tech/full_papers/ho/ho_html/nysiad_html.html) | §3 model、§4 replication/attestation/credits/epochs | 通用协议转换如何约束输入输出与公平性 |
| [Schneider，ACM Computing Surveys 1990](https://www.cs.cornell.edu/courses/cs614/2003sp/papers/Sch90.pdf) | 状态机输入与 §4 output handling | SMR 如何执行服务及合成可信输出 |
| [Coan，IEEE Transactions on Computers 1988](https://groups.csail.mit.edu/tds/papers/Coan/IEEETransactions88.pdf) | source/object protocol 模型与 translation 条件 | fault-tolerant protocol compiler 的理论谱系；非任意程序无条件编译 |
| [Cachin–Schubert–Vukolić](https://arxiv.org/abs/1603.07351) | nondeterminism / randomness / secrets | timer、随机数与外部操作如何影响复制 |
| [Lynch–Tuttle，I/O Automata](https://groups.csail.mit.edu/tds/papers/Lynch/CWI89.pdf) | input/output/internal actions、组合语义 | 如何表述 wrapper 的抽象与 refinement |
| [Fault-Tolerant Cluster-Sending，FoIKS 2022](https://expolab.org/papers/CSP%20-%20FoIKs%202022.pdf) | cluster-sending specification | 两个 Byzantine groups 如何实现可靠逻辑消息 |
| [etcd/raft](https://github.com/etcd-io/raft) | Step/Tick、Ready、storage/transport integration | 如何组织 outer f 的实际代码 |

Cluster-sending 是 transport 构件，不能单独证明 sender 执行 f 正确。可继续阅读其 [2023 expected-cost 后续工作](https://www.jsys.org/bibliography/2023-06-12-hellingsSolution.html)。可用的 BFT 应用库例子是 [BFT-SMaRt](https://github.com/bft-smart/library)；库提供 replicated application 的基础，不自动包含本文全部外部协议封装。

与项目已有说明的关系：[main_v1 的跨区域轮换与 virtual-SMR 抽象](main-v1-cross-region-and-virtual-smr-2026-09-28.md)。本次检索补充了前文尚未列出的直接架构与编译器先例。
