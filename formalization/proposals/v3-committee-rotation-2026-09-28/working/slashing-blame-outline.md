# v3 独立分析：双签惩罚、blame 通信与文章结构

日期：2026-09-28。范围：针对当前 `formalization/main_v2.tex` 的设计分析；未修改稿件。下文的 v3 条款是建议，不是现有实现或已完成的安全证明。

## 1. 当前稿件已经规定的边界

- v2 第 252–255 行：proposal/vote 绑定 session、view、position、digest；诚实节点在同 view/position 最多投一票。
- 第 267–290 行：proposal 的转发和投票启动 `2Δ_R` commit timer；vote QC 可以启动下一批 proposal，不能代替 confirmed receipt。
- 第 328–391 行：blame/view-change 的四步流程及等待窗口。
- 第 405–427 行：确认后的 endorsement 才构成 confirmed receipt；ORDER 双签在同 session、生产 view、mini-block height 内比较。

这些规则区分三种对象：`ORDER` 证明 producer 签过特定 mini-block；`VOTE`/vote QC 证明区域成员支持过候选 block；`CONFIRM`/confirmed receipt 证明该区域 SMR 已完成确认。跨区设计必须继续保留这些区别。

## 2. “同一高度两个 head”如何成为可执行的 slashing 条件

### 2.1 域分离与内容标识

建议所有签名显式包含：

\[
 D=(\mathit{chainId},\mathit{protocolId},\mathit{configId},i,\mathit{sessionId},e),
\]

其中 `i` 是 region，`e` 是跨区生产任期（tenure），`sessionId` 是区域 SMR 的授权执行实例。视图 `v` 是该实例内部的 leader view，不能拿 `v` 代替跨区任期。对公钥身份和权重的检查使用签署时的已授权配置；惩罚不会临时修改当前 quorum 分母。

mini-block 内容标识建议为

\[
 q=H(\mathsf{MINIBLOCK},h,q_{h-1},\mathsf{txCommit},D,v_{\rm author},\ldots).
\]

签名编码、证书的排列顺序以及替代但有效的同内容签名不进入内容 identity。否则可以把同一 block 的两种编码误判为两个 head。

### 2.2 五类证据

| 类型 | 可比较的签名域 | 冲突条件 | 判罚对象 |
|---|---|---|---|
| `ORDER` | 同一 producer、公钥、`D`、原生产 `v_author`、mini-block 高度 `h` | 两个内容 digest 不同 | 双签 producer |
| `PROPOSE` | 同一 leader、公钥、`D`、view `v` | 同 batch position 的不同 digest；或该 view 内两个不相容的 proposal 链 | 双签 leader |
| `NEWVIEW` | 同一新 leader、公钥、`D`、view `v` | 选择两个不同 anchor；即使两个 anchor 是祖先与后代也违反“本 view 只宣布一个 anchor” | 双签 leader |
| `VOTE` | 同一 voter、公钥、`D`、view `v`、batch position `k` | 两个不同 block digest | 双签 voter |
| `CONFIRM` | 同一 endorser、公钥、`D`，允许不同 view | 声明了两个不相容的 confirmed prefixes | 双签 endorser |

其中两个 prefix 不相容，指没有一个是另一个的前缀；不同末端高度时必须附 ancestry 或承诺结构中可验证的冲突证明。`CONFIRM` 比较的是不可撤销的确认结果，所以不能仅按同 view 比较。

最直接满足用户所说 “leader propose two different heads at the same height” 的谓词是

\[
\begin{aligned}
\mathsf{SlashOrder}(pk;m,\sigma;m',\sigma')\iff{}&
\mathsf{Verify}_{pk}(m,\sigma)\land\mathsf{Verify}_{pk}(m',\sigma')\\
&\land \mathsf{AuthorizedProducer}(pk,D,v_{\rm author})\\
&\land m=(\mathsf{ORDER},D,v_{\rm author},h,q)\\
&\land m'=(\mathsf{ORDER},D,v_{\rm author},h,q')\land q\ne q'.
\end{aligned}
\]

若 `ORDER` 实际签完整 header，合约解析并计算这里的 `(D,v_author,h,q)`。`q` 必须绑定 parent 和交易序列承诺。

### 2.3 不能误罚的情况

1. 同一未确认位置在合法新 view 中重新提案，不自动构成 producer 双签。当前 v2 允许 view change 选择另一未确认分支。跨 view 的最终确认冲突由 `CONFIRM` 规则处理。
2. 新 view 对同一个旧 anchor 重新投票并生成新的 QC 是正常行为；mini-block 原作者签名和 `v_author` 不改写。
3. `NEWVIEW` 的旧 view QC 与新 view QC 都证明同一 block，不因为证书不同而构成不同 head。
4. relay 保留原签名；转发别人的两份双签证据不会让 relay 成为原始双签人。
5. 无回应、超时、被 blame、正常 region rotation、合法恢复不是双签证据。BLAME certificate 是本地 leader 更换授权，不是 leader 作恶的客观判罚证据。

因此，不能把“相同 global height 上有两份不同 ORDER”写成无条件判罚规则。其授权 session、view 和 provisional context 可能不同。

### 2.4 与正常跨区预提案的交叉风险

旧区的末 batch `T_e` 尚未 final 时，新区可以基于候选 tip `q(T_e)` 提前传播、执行或预提案，但新结果显式带该依赖。如果旧候选随后改变，新区必须能安全重启该预提案。

有两种可行的接口约束：

- 使用由协议决定、绑定旧候选 tip 的 provisional session/attempt；只有匹配最终旧区 receipt 的分支可被提升为正式结果。
- 复用同一实例时，必须经过底层 SMR 允许的 view-change/restart；不能要求 leader 在同一 `(session,view,height)` 对不同 parent 再次签 ORDER。

不能允许 producer 随意增加一个 nonce 就逃避双签检查；attempt 的派生规则、单一继承规则和 promotion 条件必须可验证。最容易审计的基线是：预提案可以发生，跨区 finality 的提升必须等待旧区末 batch 的 confirmed receipt。任意黑盒 SMR 的裸 QC 不能被假设为不可回退的 handover lock。

## 3. 当前 blame/view-change 的准确通信 pipeline

设旧 leader 为 `L=ℓ_i(v)`，新 leader 为 `L'=ℓ_i(v+1)`。

| 阶段 | 通信 | 状态变化与时间条件 |
|---|---|---|
| 进展检测 | 每个 `S` 广播 `BLAME(v)` | 在 `t_S(v)+(2p+4)Δ_R` 若少于 `p` 个 fresh proposals 触发投票；anchor 重认证和重复消息不计数 |
| quit proof | 收集到权重 `>2W_i/3` 的 BLAME 后广播 certificate；或直接广播 leader 双签证据 | 双签路径无需等待 BLAME quorum |
| quit | quit proof 向全体 relay | 停止旧 view 投票，取消所有尚未完成的 commit timers；已完成 commitments 保留 |
| 排空旧 view | 继续收集旧 view votes/certificates | 从各自 quit 时刻起等待 `Δ_R`，不是从收到第一条 BLAME 起计时 |
| STATUS / entry | `S → L' : STATUS(v+1,Q_S)` | 取已知最高 rank 的旧 view QC，固定 entry lock，再进入 `v+1` |
| leader 等待 | 新 leader 接收所有及时 STATUS | 从 `L'` 自身进入 `v+1` 起等待 `2Δ_R`；覆盖最多 `Δ_R` entry skew 与 STATUS delivery |
| NEWVIEW | `L' → all : NEWVIEW(v+1,Q*)` | 选已知最高 rank 旧 QC，只宣布一个 anchor |
| anchor 重认证 | `all ↔ all : relay NEWVIEW + VOTE(anchor,v+1)` | 检查 `rank(Q*)≥rank(entry lock)`，不启动 commit timer |
| 恢复 pipeline | `L' → all : PROPOSE(fresh child,Q_{v+1}(anchor))` | 获得 current-view anchor QC 后才发首个 fresh child；该 fresh proposal 才按正常规则启动 timer |

消息与 deadline 同时发生时先处理消息。晚到旧 QC 可供后续恢复使用，但不能修改当前 entry lock 或已宣布的 anchor。`p` 是进展 deadline 的索引，不是区域的固定 batch quota。

### 3.1 建议图形

新增原生 TikZ 通信图：`../figures/blame-view-change-pipeline.tex`；独立预览入口：`../figures/blame-view-change-preview.tex`。

图采用 time-down 通信泳道，显示代表性旧 leader、两个正确区域成员和新 leader。正确成员仅作通信角色示意，不表示四节点或等权假设。图分三段：trigger/quit proof、drain/STATUS、NEWVIEW/re-certify/resume。虚线等待框区分 `Δ_R` 和 `2Δ_R`；它们不是可以相加为所有节点共享的一条 wall-clock timer。两条触发路径合流，防止读者误认为 equivocation 也要等 BLAME quorum。

图明确只描述**区域内部**的 leader replacement；不能将 `BLAME` 箭头画向 L1 作为直接跨区 rotation 授权。跨区事故必须走委员会 multi-sig + L1 fence/activation。

## 4. 建议 v3 大纲

论文主轴应是“低开销的日常跨区交接 + 可较慢的 L1 事故控制”，区域 SMR 是可替换的实现层。

1. **Introduction and design objectives.** 明确正常 rotation 每固定 `B` 个 batches 发生；委员会只介入事故与恢复；日常 proposal pipeline 优先优化。
2. **System model and trust boundaries.** `S_{i,j}`, `ω_{i,j}`，region 配置、黑盒 SMR、observer committee、proposer、DA、L1；分别列区域 safety、委员会 threshold、L1 finality、数据可得性前提。
3. **Objects, authority and finality.** batch 与 mini-block 高度、tenure/session/view、deterministic schedule、候选/区域 finalized/global admissible/L1 accepted 各状态。
4. **Regional SMR interface.** `Propose`, `Finalized`, `VerifyFinality`, `Fetch`；说明 cross-region 正常流程只依赖接口，裸 QC 无统一语义。
5. **Normal scheduled rotation.** 固定 `B` quota；旧区末 batch 绑定唯一 successor；新区提前订阅数据、执行、预提案；dependency 与 promotion gates；不存在每次 handover 上 L1 的要求。
6. **Exceptional rotation and reinstatement.** committee observation → multi-sig intent → L1 记录 fence、选 tip、授权新区；恢复事件同样需 multi-sig；旧区重启不能沿旧权限自行恢复生产。
7. **L1 verification of publication.** proposer 提交连续区间；逐 tenure 验证每个高度的 region、原生产者配置、finalized receipt、跨区连接与 DA 内容承诺；独立说明 execution root 的 validity 证明义务。
8. **Accountability and slashing.** 正式签名域与上述 predicates；非法 region 权限声明单独列违规类型，不混淆为同高度双签；惩罚生效配置边界。
9. **Safety and liveness arguments.** 正常边界归纳、speculation 不进入 final 输出、事故 fence、恢复 fresh session；事故隐藏 finalized prefix 的发现/保留问题必须单独陈述。
10. **Latency and communication.** 正常路径分别测 proposal continuity 与 finality latency；事故路径允许多次 L1 confirmation；SMR 内部成本和 observer committee 成本分开。
11. **Regional instantiation: Sync HotStuff.** 保留 v2 的区域稳态和 view-change 核心，加入 blame pipeline；可主文压缩后将详细安全证明放附录。
12. **Limitations and open parameters.** committee threshold、L1 等待条件、DA 证明模式、攻击/同步模型、batch-size 和 quota、恢复 eligibility，区分本文决定与待定项。

## 5. 逐节修订计划

| 当前 v2 位置 | v3 操作 | 原因 |
|---|---|---|
| Abstract / Two-Layer Structure | 重写架构与 scope | 移除尚未完成的 cross-region CFT 核心叙事；引入 schedule + committee/L1 exceptions |
| Model and Service Definitions | 拆成对象/配置/授权、SMR 接口两部分 | 不能把区域内 confirmation 与全球不可回退前缀合为一个定义 |
| Regional PoS SMR | 后移为实例化章节 | 先说明跨区接口，再解释底层 Sync HotStuff 如何实现它 |
| Leader replacement | 保留区域协议并增加通信图 | local view change 不等于跨区 rotation |
| Confirmation receipts and slashing | receipt 保留接口位置，slashing 扩为独立节 | 提供可上链核验的明确比较域和证据 |
| Stream Delivery | 扩展为正常跨区 prefetch + proposer publication | 数据先到和 finalized receipt 后到的分离，支持 latency 优化 |
| Security of Regional SMR | 作为实例证明保留；新增外层安全不变量与条件定理 | 原证明仅覆盖固定 session，不能自动推广 signer-set change |
| Complexity Analysis | 拆成日常跨区、事故路径、区域内部三部分 | 体现设计优先级，避免用罕见事故成本遮蔽常见 handover latency |

## 6. 必须正面处理的三个安全问题

**新旧 signer sets 不相交。** 不能沿用单一区域 quorum intersection 证明跨区安全。正常边界必须绑定唯一最终旧 tip，并使新区正式结果只扩展它；事故边界必须绑定已 final 的 L1 授权记录及恢复 session。

**旧区死前可能存在未公开的 finalized prefix。** committee 仅观察到的 tip 不一定包含所有已经对外承诺不可回退的对象。若事故允许从更早 L1 checkpoint 重启，则必须明确较新的区域 finality 不是无条件的全球 finality；若要保留这些承诺，需可检索证据、见证/可得性条件或额外持久化门槛。等待几十分钟本身不能证明没有隐藏 finalized suffix。

**正确序列不等于正确 state root。** 正确 region + 正确 finalized mini-block + DA 承诺绑定，可以证明 proposer 发布了被授权的有序输入；仍不能单靠这些签名证明执行后 state root 正确。后者需独立 validity proof、fault proof，或明确新增执行验证信任假设。
