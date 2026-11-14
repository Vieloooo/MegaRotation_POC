# 委员会、事故轮换与 L1 验证：独立设计分析

日期：2026-09-28。本文是 v3 的**新提案**，不是当前协议的性质声明。只读依据为 `formalization/main_v2.tex`，SHA-256 `e61c6e5083979464cef09f86d101a62940d040bd8735673fcad03a705deeb919`。下文保留原稿的 `S_{i,j}`、`ω_{i,j}`、`proposer` 以及“排序认证与执行正确性分开”的边界。原稿 regional confirmation receipt 是 post-commit endorsement quorum，不能用 vote QC 替代。

## 1. 建议采用的决策

1. 正常路径按固定 **B 个 batch** 轮换，不让委员会逐次批准，也不等待当次 L1 交易。委员会是事故控制面的独立信任集合，不把每个区域包装为外层 CFT 参与者。
2. 正常交接允许新区域先产生绑定候选 terminal parent 的 speculative stream；新区域的对外不可撤销确认必须等待旧区域 terminal parent 的最终性证明。旧、新 signer 集合不相交没有问题，前提是新区域接受的是同一个经过验证的 terminal checkpoint，而非对旧区的一个无证明 tip 猜测。
3. 事故走 `Freeze → Close/Collect → Rotate → 等待 L1 finality → Activate`。正常路径的低延迟与事故路径的数十分钟恢复分开分析。
4. 主方案保持强 prefix safety：**所有已经允许对外作为 final 接受的旧前缀必须保留**，不只保留 L1 已发布前缀。无法证明安全 cut 或无法取得其数据时，停止恢复；不能由委员会投票把缺失历史删掉。
5. L1 逐 mini-block 验证其授权来源、regional finality 和内容/次序承诺；proposer 的执行后 state root 另外使用执行证明或已明确定义的争议机制。来源签名不证明 EVM 执行正确。

## 2. 最小对象与授权索引

区分 batch 序号 `k` 与 mini-block 高度 `h`。一个 batch 覆盖非空连续区间 `[h⁻(k),h⁺(k)]`；每个 mini-block 可包含零笔交易。固定 B 的 quota 作用于 batch，而非 mini-block，不能直接令 `region(h)=floor(h/B)`。

L1 安装控制配置：

```text
ControlConfig = (
  chainId, contract, controlGeneration g,
  scheduleVersion c, orderedRegionSchedule R_c,
  batchQuota B, firstBatch k0,
  perRegionMembershipRoots,
  committeeVersion u, committeeMembershipRoot, committeeThreshold
)
```

从 `k0` 开始，未被事故事件截断时：

```text
tenure(k) = floor((k-k0)/B)
owner(k) = R_c[tenure(k) mod |R_c|]
```

每个 tenure 使用全新的 session 标识：

```text
sid = H(chainId, contract, g, c, tenure, regionId,
        regionMembershipRoot, inheritedCheckpointDigest)
```

批次、mini-block、regional finality receipt 和 normal-transition proof 都绑定 `sid`。新签名集合不能只通过替换 regionId 继承旧证书。相同区域再次轮到生产、恢复上线或更换 stake snapshot 时，也不能复用旧 session。

若 normal successor 对 terminal candidate 做预执行，候选 session 用该 candidate digest 派生。候选变化形成另一 session/attempt，不能把已有签名搬到新 parent 上。global-final receipt 的验证必须解析 inherited checkpoint 的 finality proof；一个只绑定候选 parent 的 regional receipt 至多是条件性结果。

L1 保留**不可追溯改写的授权区间历史** `Auth(g,k)`。事故事件可截断旧 generation 的未来授权区间，并创建新 generation；不能通过覆盖“current region”这个单变量使旧区历史数据失去合法来源。恢复区加入新 schedule 的生效 batch 明确且只向未来生效。

## 3. SMR 黑盒仍须承诺什么

可以隐藏 Sync HotStuff 内部消息，但不能隐藏其安全前提。建议接口：

```text
VerifyFinal(sid, k, batchDigest, receipt, config) -> bool
FetchFinalPrefix(sid, frontier) -> ordered bytes + receipts
StartSession(sid, inheritedCheckpoint, authorizedBatchInterval)
```

`VerifyFinal` 验证 receipt 的实际 finality 语义，不能仅验证签名数。正常路径的 terminal quota 必须内嵌为 honest sequencer 的有效性规则：第 B 个 batch 之后旧 session 不再创建应用 batch。证明普通轮换时，只需要 terminal finality、完整前缀继承及 successor 的有效性约束，不需要旧新委员会 quorum 交集。

事故恢复还需要两个**不能从普通 SMR safety 自动推出**的能力：

* **Closure/fencing**：存在一个安全的最终边界，之后旧 authority 无法再产生任何可能被正确接收者接受为 global-final 的新后缀。
* **Recoverability/completeness**：恢复方能获得覆盖该边界以前所有有效 finalized 历史的一个完整、相容前缀及其数据/证明。

可以额外给黑盒提供 `Seal(sid) → ClosureCert(sid, frontier)`，但只有当底层协议真的实现了持久停止、锁继承和安全 frontier 选择，这个接口才有效。给任意 SMR 加一个同名函数不构成实现或证明。整个旧区不可达时，也不能假设该函数仍会返回。

## 4. 委员会阈值与签名域

委员会 `C` 的总权重为 `W_C`，故障权重上界 `f_C`；阈值 `q_C` 应满足

```text
2 q_C - W_C > f_C,        q_C <= W_C - f_C.
```

第一式保证两个证书的签名集合交集中至少有一个诚实成员；第二式允许诚实委员会独立形成证书。例如 `f_C < W_C/3` 配合 `q_C > 2W_C/3`。若用人数 multisig，则 `n=3f+1,t=2f+1` 是一个具体实例。`t` 个签名本身只说明这些 key 同意该 statement，不会使观察结论变成客观事实。

所有事故动作签署 canonical encoding，并使用不同 domain：`FREEZE`、`ROTATE`、`REACTIVATE`、`COMMITTEE_UPDATE`。共用字段至少包括：

```text
chainId, contractAddress, committeeVersion,
actionType, eventNonce, expectedControlStateHash,
controlGeneration, affectedSessionOrInterval,
payloadHash, expiryOrValidityRule
```

签名者必须逐个去重，查历史激活的 committeeVersion；过期配置、重复 signer、重放 nonce、错误 chain/contract、错误 expected state 一律拒绝。诚实成员持久记录已签的 `(committeeVersion, expectedControlStateHash, eventNonce, actionType)`，不签冲突 payload。L1 的状态机顺序化实际执行，委员会 quorum-intersection 另保护外部已收集的冲突证书。

成员变更走已授权的配置激活规则，不能由待加入成员自己批准。旧委员会签署的挂起事件在配置变更后的接受规则必须明确；建议默认失效，由新委员会重新对当前控制状态签名。

## 5. L1 状态与事件

建议状态：

```text
mode ∈ {RUNNING, FROZEN, ROTATION_INSTALLED}
controlStateHash, generation, committeeVersion, nextEventNonce
scheduleHistory, membershipHistory, sessionAuthorizationHistory
publishedFrontier = (batchK, miniHeightH, qH, executionRoot)
freezeRecord = optional frozen snapshot
rotationRecord = optional installed replacement and retained old prefix
```

`publishedFrontier` 指当前 L1 合约已接受的有序发布前缀，不等于旧区已经 global-final 的最高前缀，也不等于该 L1 交易已达到最终性。外部启动新 authority 只依赖 finalized L1 状态。

### 5.1 Freeze

`FreezeCert` 指定预期控制状态、要冻结的 generation/授权后缀、原因证据 hash 与唯一 nonce。合约原子执行：

1. 检查委员会证书及 CAS（compare-and-swap）条件。
2. 在这笔交易的执行位置读取当前 `publishedFrontier`，记录为 `F_L1`；之后暂时禁止该冻结范围继续推进发布 frontier。
3. 固定后续 `Rotate` 的 `freezeRecordHash`，将 mode 改为 FROZEN。

这里有两种安全竞态处理：让 FreezeCert 同时绑定精确发布 frontier，则抢先到来的合法 publish 导致 freeze 失败、重新签名；或者签名明确授权“冻结此控制 generation，并以本交易原子执行时的 frontier 为底线”，由合约产生不可更改的 `freezeRecordHash`。建议后一种避免正常数据发布使签名反复失效。Rotate 必须引用已经确定且已达到 L1 finality 的 freeze record。

**合约冻结发布并不等于旧区停止签名。** 隔离的旧区域和持有旧状态的客户端可能尚未看到 freeze；因此此时只能开始事故流程，不能凭这一笔交易声称 offchain-finality frontier 已经闭合。

若 normal rotation 恰好并发，冻结对象应覆盖该 generation 中尚未 L1-settled 的授权后缀及其所有 speculative descendants；收集必须包括已按旧 schedule 获得有效 global finality 的 successor session，不能只询问委员会最初认为的那个 active region。

### 5.2 Close/Collect

等待以下两项都成立：

**CLOSE：** 旧授权后缀已闭合，任何此后可被正确接收者接受的 global-final receipt 都落在待恢复前缀内。闭合证明可能来自旧 SMR 的安全 Seal，或事先配置的 authority lease 到期配合严格的时间/新鲜度规则；不能从“已等一小时”推出。

**COLLECT：** 取得完整继承前缀 `F*`，它延伸 `F_L1`，并覆盖 CLOSE 以前所有已有效 finalized 的后缀，包括尚未向 proposer/L1 发布的部分。委员会逐个检查来源、finality receipts、hash links 和原始数据可恢复性。

`F*` 必须是上述完整 finalized 前缀，不能只选择投票最多的“tip”，也不能从一条较长但未 final 的分支中挑一个高度。若发现冲突 finality receipts，则已经发生区域安全前提或签名安全违约；进入争议/停止状态，不以委员会多数票把它重新解释成原有 safety theorem 内的正常恢复。

“所有委员都没有见过更高 receipt”不等价于 COLLECT。“委员看到的最高 receipt”只给出一个下界。委员会可作为信任根证明它执行了额外观察/存储规则，但论文必须把这些规则及其完整性假设写出来。

### 5.3 Rotate

证书 payload 建议为：

```text
Rotate(
  freezeRecordHash,
  retainedPrefix = (cutBatch, cutMiniHeight, cutDigest),
  retainedPrefixProofRoot, recoveryDataCommitment,
  closureEvidenceHash,
  newGeneration, newRegion, newSession,
  newRegionMembershipRoot,
  newScheduleVersion, newBatchStart = cutBatch + 1,
  inheritedExecutionCheckpointReference,
  reasonEvidenceHash
)
```

合约检查：证书有效、冻结记录匹配、generation/nonce 单调、新 region 已授权、cut 延伸冻结底线、旧证明对应历史成员表、固定 batch/miniblock 边界正确。L1 能直接验证的 proof 与依赖委员会诚实观察的字段必须分别列出；不能把 `closureEvidenceHash` 的存在当成闭合证明本身。

若 cut 高于已发布 frontier，Rotate 安装的是**待补齐的旧 final 数据义务**和新授权边界。proposer 必须先按旧来源证明依次补齐这些 batch，执行到 cut 后，才可提交新区域数据。委员会不能虚构 cut 的 state root；从已验证执行 checkpoint 到 cut 的执行由 proposer/执行证明路径完成。

### 5.4 Finalized L1 → Activate

新区域只在 Rotate 交易所属 L1 状态达到协议规定的 finality 后激活新 session。`Activate` 可为 offchain 状态转换，无须再花一笔 L1 交易：检查 finalized-L1 proof、下载完整旧 prefix 及必要执行状态、验证本地 checkpoint、绑定新 sid，再开始创建可 global-final 的新块。

在此前预同步和预执行是允许的；不能对外宣称新 authority 已不可逆生效。若依赖“若干确认”而非 L1 finality，则跨区安全也只能相对于所选 L1 common-prefix/reorg 假设成立；几十分钟/一小时是运行目标，不是无需前提的最坏时间上界。

## 6. 事故分支选择的不可省略限制

### 6.1 未发布 finalized suffix

设 L1 只发布到 `q100`，旧区已经向一个正确用户交付 finalized `q101...q105`，但委员会只看到 `q102`。若委员会选择 `q102` 并授权新区域在高度 103 建造另一分支，则旧用户的 finality 被回滚；L1 合约延伸 q100 的检查完全发现不了这个问题。增加多签人数、延长等待本身都不消除该执行。

因此，以下三者不能无条件兼得：

1. 所有有效 regional-final receipts 都立即成为不可回滚的 global finality。
2. 整个区域以及这些 receipts 的全部副本可能永久不可达。
3. 事故必定在有限时间内从较早 checkpoint 恢复。

建议保留 1，明确放弃无条件的 3：CLOSE/COLLECT 不满足时 fail-closed。

### 6.2 回滚范围

恢复仅可丢弃从未达到 global finality 的 candidate/preview 分支。新 tip 必须延伸所有已经 global-final 的前缀。不能使用“委员会指定新 tip”作为任意切换到较短链或 sibling 的权限。如果业务确实要在数据永久丢失时强制回滚，那是独立的社会/治理恢复，不属于本协议的安全事故轮换。

### 6.3 预置 lease 可做什么

若希望旧区完全失联时仍有机会闭合，可考虑预先设定一段较长 authority lease：receipt 的有效性受 lease、严格签名规则、可靠时钟/有界时钟偏差及客户端新鲜度约束；事故至少等到 lease 到期并经过传播安全余量，再收集完整 prefix。lease 的更新可低频上 L1，不必每次正常轮换上 L1。

这是一项**新增机制和新增假设**。只在消息里加一个 timestamp 不足够；需要证明 honest signer 不会签署过期 receipt，陈旧客户端不会无限接受过期 authority，以及合法但延迟披露的旧 receipt 已被恢复规则覆盖。lease 解决停止继续扩张的问题，不单独解决未知/丢失历史问题。本文不把它默认加入正常热路径，也不把它当作已经完成的证明。

### 6.4 外部持久化选项与代价

另一条具体方向是在接收者允许称为 global-final 前，要求 receipt 及数据已存入与区域故障域独立的存储/DA quorum，并有可验证 durable ACK 或 availability certificate。恢复集合必须与 ACK 集合在正确数据持有者上相交，而且存储系统必须支持枚举完整 frontier、抵抗隐藏已 ACK 的对象，并在事故时可读取。仅存 hash 或仅由 proposer 单点保管都不满足此条件。

这会增加 normal **finality-release** 延迟及跨区域字节成本，尽管 proposal pipeline 仍可继续。它不是“免费观察”，也不是当前需求已经默认接受的机制。可以让观察委员会承担该存储职责，但逐批 durable ACK 便是新的热路径负担；推荐在 v3 中列为可选择加强项，先不偷偷写进常规延迟公式。

## 7. 恢复上线的区域

`ReactivateCert` 至少包含 regionId、全新的 membership/config root、恢复检查所依据的 finalized checkpoint、当前 control-state hash、首次可参加的未来 tenure/batch 及 nonce。L1 记录 region 从 DISABLED 转为 ELIGIBLE 的 finalized 事件。

ELIGIBLE 不等于立即成为 active。恢复区只有到新 schedule 明确分配给它的未来区间才可生产；不恢复旧 sid 的未完成 quota，不使旧签名重新有效。区域必须清理/隔离旧会话状态、同步最新 checkpoint，再进入新 session。若安排恢复区立刻接替当前区域，必须另走具备 cut 与继承检查的 Rotate，而不是滥用 Reactivate 跳过交接。

普通轮换中遇到 DISABLED region 的处理应由已生效 schedule 决定。不能由一个区域或 proposer 自行跳过；可在事故 Rotate 时直接安装排除故障区的新循环 schedule。

## 8. L1 对 proposer 的必要检查

proposer 提交从当前 `publishedFrontier` 的严格下一位置开始的有序区间，附原始数据/DA 引用、mini-block 清单、regional finality 证明、跨区边界证明和输出 state root 的证明路径。

L1 的逻辑验证条件：

1. **连续性**：batch 和 mini-block 区间无缺失、重叠或调序，首 predecessor 等于 L1 当前 q；尾 digest 唯一。
2. **逐高度授权**：每个 h 所属 batch k 的 `(g,sid,region,membershipRoot)` 等于历史 `Auth(g,k)`，并满足普通/事故轮换区间。不能只验证提交者“是一个已登记 sequencer”，也不能用最新成员表验证旧签名。
3. **来源与 finality**：每个 producer ORDER 签名有效，region receipt 根据对应 fixed stake snapshot 验证；receipt 覆盖该 mini-block 的 batch digest 与完整有序列表。vote QC 不能冒充 post-commit receipt。
4. **跨区 parent**：new session 的 inherited checkpoint 等于上一授权区间的最终 terminal digest；普通转换有 terminal finality proof，事故转换有 finalized L1 Rotate 和完整 retained prefix。普通流中的 speculative receipt 不能跳过此门槛。
5. **数据对应**：提交/DA 承诺的数据 canonical encoding 与已 final 的 mini-block tx commitment 完全一致，保留交易顺序、重数、长度和边界。Merkle inclusion 单独只证明“某叶存在”，还需证明全范围的完整性和次序，防止删项/拼接。
6. **DA 条件**：验证所选 DA 方案要求的承诺/可用性证据；一个任意 URI 或不可读取 hash 不算数据发布。具体 DA 安全性与保留期需另行定义。
7. **执行正确性**：从先前已接受 executionRoot 执行上述同一数据，导出所提交 state root。可用有效性证明或已明确的 optimistic dispute 流程；如果暂未设计此层，结论仅为“来源/排序承诺一致”，不得写成“L1 已保证正确 state root”。
8. **更新原子性**：所有条件成立后才共同更新 ordering frontier、DA commitment 与 execution commitment。失败不留下部分推进的授权或状态。

“逐高度检查”是逻辑性质，不强迫在 Solidity 中逐一循环。可先给出透明的逐项验证器，再让范围证明/聚合证明或专用证明电路压缩它；压缩需证明等价，不能以一个批量 proposer 签名替代。

### Proposer 的诚实性与权限

如果任何人都能携完整有效证明发布，proposer 可是 permissionless relay，L1 不需要信任其正确性；若指定 P，则额外验证其发布权限，但这不代替上述内容证明。恶意 P 可以拖延；在协议允许第三方补交且数据可恢复时，拖延不必破坏 liveness。是否允许补交是 v3 需显式定稿的选择。

## 9. 可主张的性质与仍需补足的前提

在下列前提下，建议证明“授权流 prefix safety”：区域 SMR safety 与 receipt soundness；签名/hash 安全；正常 handoff 的 terminal-parent gate；委员会阈值诚实与不冲突签名；L1 finality/common prefix；事故 CLOSE/COLLECT；新区域只延伸恢复 checkpoint。按正常 transition 或事故 transition 作归纳，每个新 final prefix 延伸先前 final prefix；L1 的发布验证器仅接受这个 prefix 上连续区间。

该论证没有把区域延迟假设失效自动转换成 crash fault。若所选同步 SMR 的 safety 本身依赖网络延迟上界，则区域 outage/partition 是否仍满足 safety 需单独设定；观察委员会的故障判断不能修复已发生的互相冲突 finality。将 SMR 改为黑盒允许另选满足目标故障模型的实现，但不使当前 Sync HotStuff 的前提自动加强。

条件事故恢复时间为：

```text
T_detect + T_freeze_L1final + T_close + T_collect
         + T_rotate_L1final + T_stateSync + T_firstFinal.
```

允许数十分钟到一小时使慢控制面具有实践余量；没有 CLOSE/COLLECT、可用 L1 和最终能够签名的诚实委员会，就没有有限时间保证。正常轮换不含这些项。

## 10. 给主稿提纲的接口建议

主稿定义顺序应是：finality 层次与授权对象 → SMR 黑盒 → 固定 quota 的正常 handoff → speculative-overlap 安全门槛 → 委员会及 L1 状态机 → 事故 CLOSE/COLLECT 假设 → proposer/DA/L1 验证器 → composition theorem → 分开的正常/事故代价。原有 regional Sync HotStuff 实例可放在后面或附录。

原稿 `sec:architecture` 的 future CFT wrapper、`sec:model` 的 outer-finality 空缺、`sec:receipt` 的授权域、`sec:delivery` 的早到数据语义和 `sec:complexity` 的无 cross-region bound 需要针对这些接口修订。不要仅把原文 “CFT” 替换成 “committee”：正常授权区间、继承条件、闭合与恢复完备性均是新实质内容。

## 11. 需要作者最终选择但不阻碍提案的项

* 黑盒 SMR safety 的故障/网络模型是否覆盖区域隔离；当前同步实例不能无条件推广。
* 事故 closure 用 native Seal、预置 lease，还是明确只在旧 final frontier 可安全封闭时恢复。
* recoverability 采用运行假设、外部 durable ACK/DA，还是组合；额外 ACK 对正常 finality 延迟的预算。
* L1 等待使用 consensus finality 还是较弱确认规则；时间预算依赖所选规则。
* DA 模式、执行证明模式、是否允许 permissionless publication fallback。
* normal quota 以 batch 为单位时，是否另要求区间边界与 EVM block 边界重合。若目标编码要求同一个 EVM block 的全部 mini-blocks 使用同一授权 signer，必须额外保证该对齐；不能在本文里默默假定当前稿的抽象 mini-block 等同于现有实现编码。

这些选择不改变主建议：常规流水线先优化 proposal overlap，finality 的继承门槛保持完整；事故允许慢，但不能以 multisig 掩盖未知 finalized 历史或旧 authority 未闭合。
