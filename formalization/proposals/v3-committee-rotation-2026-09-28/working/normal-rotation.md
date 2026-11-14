# 正常跨区域轮换：提前流水线与不降级的最终性

本文为独立设计提案，不修改 `formalization/main_v2.tex`。范围是用户优先要求的日常固定 batch 配额轮换；事故轮换只给出必须遵守的接口约束。所有算法和性能公式均为提案或条件推导，不代表现有实现已经具备这些行为。

## 1. 推荐结论

正常轮换不经过委员会，也不等待新一轮 L1 确认。L1 已确定配置、区域顺序和配额；旧区域第 `B` 个应用 batch 确定唯一最终边界，新区域根据这个边界接续。为了降低停顿，新区域可在旧末 batch 尚未 finalized 时，对一个经过认证的候选边界启动**条件会话**，提前完成提议、传播、投票和区域内部确认。只有旧边界的最终证明与新区域的本地最终证明同时成立，才产生跨区域 `GlobalDeliver`。

因此重叠的是工作和等待，不是对已最终前缀的可撤销承诺。旧、新区域的 validator sets 无须相交；安全连接来自旧区域对确切前驱的最终证明及新区域对其确切后继的最终证明。普通 vote QC 只用作提前启动的性能提示。

应首先规范两种路径：

| 路径 | 何时启动新区域 | 何时对外最终确认 | 地位 |
|---|---|---|---|
| 保守路径 | 获得旧末 batch 的最终证明后 | 新区域自己的最终证明成立后 | 必须始终可用的正确性基线 |
| 提前路径 | 已获得旧末 batch 的认证候选及所需数据时 | 两边最终证明都成立后 | 推荐的日常性能优化 |
| 不推荐的捷径 | 看到旧 QC 即把新区域的确认视为全局最终 | 旧区域尚未最终确认时 | 从通用 SMR 黑盒性质无法证明安全 |

## 2. 当前正文能提供什么

`main_v2.tex:100–112` 将 membership、keys、weights 固定在一个经认证的 regional session 中；`132–174` 区分原始数据与确认 receipt，并明确现有安全定义只覆盖单区域授权会话。`234–255` 的 batch positions、原始 authorship view 和 certificate view 也不同。`264–282` 允许认证链与 commit timer 重叠。`405–411` 更明确指出 raw vote QC 不能用作 confirmed-stream receipt。

所以新设计应扩展 session / authority 的含义和跨区 receipt verifier，不能直接把旧区域 QC 当作新区域的无条件授权。本文也不声称已找到当前 Sync HotStuff 实例中任意具体 QC 都会被撤销的执行；准确结论是：现有黑盒接口没有赋予普通 QC 跨会话不可撤销含义。若未来希望凭 QC 直接转移最终权限，需要新增并证明更强的接口契约。

## 3. 编号与权限上下文

采用四个不同编号，避免把批次、mini-block、view 混为一谈：

- `g`：由已稳定的 L1 事故/配置事件递增的 generation。普通轮换不递增它。
- `e`：此 generation 内的正常区域任期序号，owner 为确定性的 `sched(C,g,e)`；不从本地时钟推定 owner。
- `r∈{1,…,B}`：该任期的应用 batch 位置。重传、替换候选和跨 view 重认证均不消耗额外配额。
- `h`：全局 mini-block 高度。batch 大小可变，因此 `h` 不能从 `eB+r` 直接推导。

另可保留全局应用 batch 序号 `k`。每次事故切换后记录显式起点 `k_start`，不要假定所有 generation 都有完整的 `eB` 个旧 batches。

一个候选边界描述符为

```text
d = (parent_authority, e, owner_old, r=B,
     terminal_batch_digest, last_miniblock_height, last_miniblock_hash,
     segment_commitment)
```

其中 `segment_commitment` 及其证明必须认证从该任期起点至位置 `B` 的连续批次和 mini-block 序列，不能只签一个声称“这是第 B 个”的末尾 hash。下一会话上下文确定为

```text
sid_next(d) = H("REGION_SESSION", chain_id, config_hash, g, e+1,
               owner_next, H(parent_authority), H(d))
```

`sid_next(d)` 不是 producer 可随意挑选的 nonce；它是经过验证的旧候选边界的确定性函数。每个给定有效最终边界只有一个正式 successor session。`ORDER`、proposal、vote、local-final endorsement 都绑定 chain、configuration、generation、epoch、region、session、对应位置和原始作者 view。跨 view 重认证保留原始 producer 的 bytes、签名和 authorship context。

正常区域顺序由已稳定的配置预先决定。冻结/恢复区域造成的 schedule 修改只能由已生效的 L1 事件决定；未确认事件和节点各自的故障猜测不能修改 `sched`。

## 4. 黑盒接口：本地不可撤销与全局不可撤销分别命名

区域 SMR 的最小接口应显式包含：

```text
OpenSession(sid, owner_set, anchor, quota)
Candidate(sid, r, batch, candidate_evidence)
LocalFinalize(sid, prefix, local_final_proof)
VerifyLocalFinal(sid, prefix, local_final_proof)
```

`LocalFinalize` 表示该**确切 session 内**的不可撤销决定。session 自身可能仍依赖未被上游最终确认的 anchor；因而不能把这个通知直接发给应用当作已最终的全局日志。

跨区 adapter 另定义

```text
GlobalDeliver(prefix, boundary_chain, local_final_proof)
```

它先验证该 session 的所有边界依赖都成立，再验证本地最终证明，并强制 hash-linked 连续扩展此前全局已交付的前缀。候选数据可提前用 `TentativeStream` 传输；名称、API 与 UI 不能将其与 `GlobalDeliver` 混用。

旧区域的正常结束证明可以直接复用其 `r=B` 的 local-final receipt，加上完整任期上下文与批次证明；不必增加一个委员会签名或新共识 round。称它为 `TerminalFinal(d)` 是一种验证语义，并不必然要求新的密码学证书格式。

通用 SMR 黑盒还必须声明“末尾可排空”性质：停止接收新的应用 batch 后，最后一个 batch 仍能完成最终确认。当前 Sync HotStuff timer 路径适合这种模式。若换用需要后继区块才能最终确认祖先的 chained engine，应允许不消耗应用配额的控制区块，或提供专门的 close 操作；不能在 batch `B` 后冻结一切消息再无条件声称最后一批会 finalized。

## 5. 正常提前轮换协议

1. **预热。** 在旧区域仍正常生产时，下一预定区域已获知配置、owner set、下一个 epoch 和配额，并持续接收已最终前缀及候选数据。准备数据和应用上下文无需等待任期结束。黑盒自己的 session bootstrap 开销仍须如实计入，不能直接宣称为零。
2. **旧区域达配额。** 只允许应用位置 `1,…,B`；旧区域不能通过延迟最后一个 batch 的最终确认而继续占据 `B+1`。旧 view-change 和排空控制消息仍可继续。
3. **认证候选边界。** 旧区域为 `r=B` 提供候选 `d`、候选证据和必要数据。证据可以是本实例的 QC；它仍不是 `TerminalFinal(d)`。下一区域验证父权限、固定配额、数据与祖先后，才认可这个有界的条件会话。
4. **提前新流。** 下一区域在 `sid_next(d)` 下提出首批，首 mini-block 的前驱严格等于 `d.last_miniblock_hash`，高度严格为 `d.last_miniblock_height+1`。新组可执行本地共识，产生标明条件 session 的 `LocalFinalize`；这些结果暂存。
5. **兑现边界。** 若收到 `TerminalFinal(d)`，该条件 session 成为唯一正式 successor。已有且验证通过的本地结果可立即转换为全局可交付结果，不再额外重做一次跨区投票。边界证明可以缓存，并由后续 proofs 引用。
6. **候选未兑现。** 若旧区域最终确认了另一合法边界 `d'`，保留已经全局确认的前缀，仅废弃 `sid_next(d)` 的条件输出；按照唯一的 `sid_next(d')` 继续。已经在废弃 session 内本地决定的对象仍是那个 session 的历史，不声称撤销了该 session 内的 SMR commitment。
7. **旧区域失活。** 不能靠超时把条件新流升级为全局最终。走 L1 委员会事故路径；该路径仍须遵守全局已最终前缀保存规则。

建议第一版只允许**一个未解决的跨区域边界**：其父任期的起始权限必须已经全局有效；同一下一 epoch 同时最多维护一个活跃候选会话，并限制 speculative batches、bytes、运行时间、缓存和请求速率。切换候选须有协议规定的旧区恢复证据或新的最终边界，不能接受 proposer 任意创建的 hash。到达资源上限时只暂停进一步条件工作，不把已缓存结果误作最终。这些限制只影响性能，不放松最终性门槛。

## 6. 时序与收益

令 `t_Q` 为旧末 batch 的候选证据就绪时刻，`t_F` 为其最终证明就绪时刻；`d_X` 为该轮实际跨区完整消息延迟，`L_new` 为新区域从获得候选数据至其首批本地最终证明就绪所需的时间（包括未隐藏的 bootstrap）。忽略两条证明到消费者的最后一跳差异时：

```text
串行：T_serial   ≈ t_F + d_X + L_new
重叠：T_overlap  ≈ max(t_F + d_X, t_Q + d_X + L_new)
```

理想收益约为 `min(t_F−t_Q, L_new)`。真实界限应分别加入旧最终证明、新本地证明及数据到达同一 verifier 的路径。公式展示的是把两段等待并行，不是删除旧区域最终性等待。

```text
时间 ↓     旧区域组                  新区域组                 proposer / L1 verifier
           提议末 batch B
           候选 QC + 数据 ──────────→ 打开条件 session
           commit timer              提议首 batch 1 ─────────→ 可预取、可预执行
           继续运行                  vote / timer              暂不最终接受
           TerminalFinal(d) ────────→ 兑现唯一父边界 ──────────→ 验证父边界
                                      LocalFinalize ─────────→ 验证新组最终证明
                                                               GlobalDeliver
```

两条最后证明的到达顺序可以交换。论文图应画出两条流水线重叠与 `AND` 验证门，不应把“旧 QC 到达”画成最终权限已经无条件切换。

## 7. 安全论证应证明的内容

在密码学有效、每个正式或条件 session 满足其黑盒安全、L1 authority history 一致、初始前缀一致的前提下，可对正式 epoch 归纳证明正常跨区 prefix safety：

1. 旧正式 session 的安全性保证至多一个有效 `TerminalFinal(d)`。
2. 因此确定性的 `sid_next(d)` 和新 owner 唯一。
3. 新区域本地安全保证该 session 内至多一条最终前缀；其首对象必须延伸 `d`。
4. `GlobalDeliver` 只接受上述前缀，故不可能与旧已交付前缀冲突。非兑现条件 session 对这个归纳不产生任何全局输出。

这个论证不依赖旧、新 quorum 在成员上相交。其代价是全局接受仍然等待旧边界最终；新区域先完成多少工作不改变这一门槛。

在仅给定通用 SMR 安全/活性的黑盒条件下，不能证明“任意候选 QC 被新组确认后立即成为全局最终”。一个允许最终决定前替换候选的黑盒，完全可在之后最终确认另一候选；不同 signer sets 的 quorums 没有交集能自动排除这种行为。这是接口层反例，不是对当前 Sync HotStuff 具体执行的未经证明的攻击声称。

若要在旧 batch 尚未 `LocalFinalize` 时就产生不可撤销的全球后继，必须额外提供 `HandoffLock(d)`：它保证旧正式 session 的所有未来最终输出均与 `d` 相容、该边界永不换成冲突值，并能安全结束旧权限。该性质实质上已经为边界提供最终性意义，通常需要 engine 专属规则及新的证明。第一版不建议以普通 QC 冒充它。

## 8. 条件会话与 equivocation / slash

推荐维护两个明确的验证域：

- 一个有效 session 内，同一 producer、原始 authorship view、mini-block height 的不同 `ORDER` headers 是正面的 equivocation 证据；session 还绑定唯一候选边界。
- 不同合法条件 session 的竞争后继，不应仅因 numeric height 相同而自动 slash；协议允许一个候选失效后基于新候选重启，必须避免惩罚这个合法行为。

这个例外不能由 signer 任意发明 session ID：候选必须来自经验证的父权限和父区域证据，ID 由上文公式唯一决定，并遵守有界候选切换规则。同一已最终边界下，不允许通过增加 nonce 创建第二个正式 session。若 L1 不希望处理全部条件证据，条件 `ORDER` 可保留在单独的签名类型下，待父边界兑现后再取得正式 endorsements；具体编码必须与最终 verifier 一起定稿。

更强的“同高度、跨 view、跨所有候选域都不能双签”规则会把合法恢复也判为错误，除非相应协议本来就禁止这些行为。原始 producer view、重认证 view 与最终 session 的关系必须写清。

## 9. L1 与 native block 边界

L1 应证明每个已提交 mini-block 属于某个有效 batch，batch 属于指定 `(g,e,r)`，而这个 epoch 的 owner、signer snapshot、anchor 和最终证明均正确。只验证最后一个 root 或一个当前 region ID 不足以覆盖跨边界数据。正常边界证明可随 proposer 的常规提交批量上链，不要求每次普通轮换单独发一笔 L1 rotation 交易。

若一个 native EVM block / proposer 提交覆盖两个区域的 mini-blocks，有两个明确选项：

1. 允许跨区覆盖，验证器对其中两个分段分别验证区域最终证明和正确的边界连接；外层一个 signer/header 不能替代这些逐段证据。
2. 要求每个 native block 完全落在一个区域任期内，使普通轮换对齐 native block 边界；其额外等待与空尾开销应计入性能。

建议先选择允许分段验证的抽象接口，除非 native block 规则明确要求一个 sequencer signer 独占完整 block。无论哪种选择，`proposer` 提交者签名、sequencer 的原始 `ORDER` 签名、区域最终确认 quorum、执行正确性证明是不同证据。正确的区域序列并不会自动证明 `stateRoot` 是正确执行这些交易的结果。

## 10. 与事故路径的必要连接

事故发生时可能同时存在旧末尾与新条件流。L1 事故事件必须明确保留的全局前缀和新 generation。未兑现条件流不自动获得最终性；已兑现并全局交付的新流不能被委员会当作“仍属旧区域候选”回滚。

由于本接口允许尚未写入 L1 的全局最终日志，事故时“保留当前 L1 tip”本身不足以保证其余已交付前缀。若要承诺强最终性，就需要 closed-prefix completeness 及相应数据可恢复条件；无法证明时停止恢复比选择任意旧 tip 安全。事故等待更长时间有助于收集证据，但经过几十分钟本身并不证明证据完整。
