# main_v3 拟议大纲与 main_v2 修订计划

日期：2026-09-28；与 [当前设计](design-zh.md) 同步。
本轮决定：正常轮换等待旧区末批 finalized 并送达后启动新区；委员会事故事件只声明截止 batch 高度及区域状态。
删除条件会话与单独的 StreamFinal 阶段，保留有效授权、区域 final receipt 和 L1 finality 的区别。
不创建 `main_v3.tex`，不编辑 [main_v2.tex](../../main_v2.tex)。

## 1. 论文主线

建议标题：

> An Authenticated Sequencer Stream with Finalized Regional Handoffs

建议副标题：

> Fixed-Quota Rotation and Committee-Authorized Cutoff Heights

区域 SMR 作为黑盒，旧区唯一 finalized terminal batch 为新区提供确定锚点。
委员会只管理故障/恢复及事故截止位置，不认证正常 stream tip，不决定同一高度的分支，也不签新区首 header。
L1 根据委员会权限事件和独立区域 finality witness 解析继承历史，验证 proposer 发布与执行的正确输入。
正文以模型、对象、算法和条件证明组织；原来的 outer CFT / crash-only regional wrapper 不再是目标架构。

## 2. 拟议目录

### 1. Introduction and Design Overview

定义日常固定配额轮换、区域事故恢复和忠实发布三个问题。
主图展示 active region、预热 successor、proposer、DA、L1 及只处理控制事件的委员会。
说明接受 $2\Delta_G+\delta_X$ 的理想边界等待，因而采用先确认后启动的结构。
区分边界可用时间、新区首 proposal 和新区首 finalized batch 的时间。

### 2. System Model and Service Specification

#### 2.1 Participants and assumptions

定义 $S_{i,j}$、$\omega_{i,j}$、region、委员会、proposer、DA 与 L1。
区域 safety/liveness、委员会阈值、网络、数据保留和 L1 最终性独立陈述。
说明同步 SMR 的网络假设不能被委员会的故障标签替代。

#### 2.2 Heights and authority intervals

明确 g、e、v、全局 batch 高度 b、tenure 内 batch 位置 k、全局 mini-block 高度 h、物理 log position。
给每 g 的首应用高度 $b_g$、固定 B 与 deterministic schedule。
事故签 b*，不能把它当 mini-block 高度；b 在新 generation 中不重置。

#### 2.3 Finality semantics

定义 Pending、有效 session 内的区域 Finalized、L1Accepted、L1Finalized。
不再定义 ConditionalCommit 或独立的 StreamFinal 状态。
“session 有效”要求启动前已验证继承锚点和权限；final receipt 是共识凭证，不是执行 receipt。

#### 2.4 Properties

授权有效性、前缀安全、正常进展、事故不回滚、按高度唯一解析和忠实发布。
执行正确性、DA 可用性、交易公平性分别给范围。

### 3. Data Structures and the SMR Interface

#### 3.1 Cryptographic conventions

规范编码、domain separation、哈希、签名、成员快照、去重与 weighted receipts。
内容身份不由签名编码决定。

#### 3.2 Batches and final receipts

batch 绑定 g/e/region/session/b/k/mini-block range/parent/ordered bytes/terminal flag。
区分原始 authorship view 与 recertification view。

#### 3.3 Finalized-anchor sessions

正常 session 由已 final 的 terminal anchor 和已安装 schedule 唯一导出。
genesis 与事故首 session 有显式基例；同 session 重启还原防双签状态。

#### 3.4 Black-box contract

`StartSession / Propose / Finalize / VerifyFinal / Fetch / Drain`。
StartSession 只接受 finalized anchor；不需要条件会话隔离或候选分支晋升接口。
末尾排空仍是黑盒义务，物理控制条目不消耗应用 quota。

### 4. Normal Cross-Region Rotation

#### 4.1 Fixed quota and terminal boundary

恰好 B 个应用 batches，重传/重认证不计数；末批须能安全完成。
固定配额与 native EVM block signer 边界的构造前提。

#### 4.2 Finalize, transfer, verify, start

完整算法 `FinalizeThenHandoff`：预热 → 旧末批 final → 跨区发送 → 验凭证/父链/owner → 安装新 session → 新区提案。
预取未确认 bytes 只服务传输准备，不能授权提前新区排序。
正常流程不需新委员会签名或新 L1 交易。

#### 4.3 Timing and failures

$T_{boundary}=T_{old-final}+\delta_X+\varepsilon_{verify}$。
约 $2\Delta_G+\delta_X$ 仅在 final receipt 就绪与数据预热等计量前提下使用。
写清未隐藏 bootstrap 与新区自己 finalization 的额外开销。
QC-only、丢失数据或 successor 不可用时，不绕过 finality/权限门槛。

**图 2。** [正常轮换图](figures/normal-handoff.pdf)：已改成先确认后交接，不再有条件会话或双证明晋升阶段。

### 5. Committee Control and Height-Based Incident Recovery

#### 5.1 Authority limits

委员会可标记区域状态并签 FREEZE、ROTATE_HEIGHT、READMIT。
禁止认证常规 stream、挑选冲突 block hash、签新区首 header 或任意更换 producer。
下一 owner 来自既定调度规则；状态更新影响资格，不等于任意调度权。

#### 5.2 Height statement and regional witness

定义委员会签署事件 E_rot 和独立见证 W*。
E_rot 携带 b*、控制上下文和区域状态；W* 携带区域 final receipt、batch 与 ancestry。
verifier 通过 `ResolveFinalizedBatch` 导出具体 anchor；L1 仍保存派生 hash，但其内容不是委员会裁定。

#### 5.3 Unique resolution

在区域 safety 和有效授权历史下证明同 `(g,b)` 的 final value 唯一。
两个 witness 可以有不同签名编码，但若都有效，其解析内容应相同。
不存在 final evidence 则等待；出现冲突 final evidence 则停止，不让委员会选择其中一个。

#### 5.4 Complete cutoff and fencing

用“委员会看到 100，而实际 final 到 105”的反例说明：只给高度仍需要完整截止点。
closed-cut 必须覆盖整个 g 的所有合法 final histories，停止未来旧权限导出并保证数据恢复。
区分见到一个合法 receipt 与证明没有更高 final suffix。

#### 5.5 On-chain state machine

`Observe / Freeze / CloseCollect / DeclareHeight / Resolve / Install / Activate`。
控制 head 与 publication cursor 分离；FREEZE 原子捕获执行时 frontier。
规定控制 nonce、deadline 和只推进过期轮次的 AdvanceExpiredRound。
安装新 g 不覆盖旧 suffix 的历史补发布义务；外部节点等 L1 finality 再启动新区。

#### 5.6 Readmission

恢复为 eligible 不等于立即 active。
旧 session 不复活；恢复资格进入新 schedule 的生效边界须事先定义。
若要不等下一事故就重返日常轮换，补全普通配置激活规则。

**图 3。** 后续拟画事故状态图：委员会只给 b*，区域 witness 提供内容，L1/新区独立解析。

### 6. Data Retrieval and L1 Publication

#### 6.1 Push and repair

保留区域 push、peer poll-fetch 和 successor 预热；timeout 不授予权限。

#### 6.2 Source authentication

L1 继续验证 regional final receipts / 等价聚合证明和完整授权历史。
仅事故截止高度的 multisig 不能替代正常来源证明。
一笔提交可跨多个 region，按连续区间核验。

#### 6.3 Exact data and execution

对齐 stream commitment、DA 编码与完整交易次序；不是仅证明几个叶子存在。
单独定义 VerifyDA 与 VerifyExecution；pending execution 和已结算状态分离。

#### 6.4 Historical publication

事故新 g 生效后，允许旧 suffix 依据历史配置补交到解析出的 anchor，然后接纳新区数据。
合约接受当前链上的事件与外部等待 L1 最终性分开。

### 7. Accountability and Slashing

保留当前稿的 ORDER 双头比较域、adjudication 去重、VOTE / PROPOSE / CONFIRM 证据。
移除已删除的条件 session 相关特例；仍不能把不同合法 local views 的同高度 proposal 一律误罚。
委员会控制消息有独立 domain 和 nonce equivocation 规则。
错误 cutoff、主观故障判断或不可得数据不自动等价于可上链证明的双签。

### 8. Regional Instantiation and Terminal Drain

保留当前 Sync HotStuff 普通 steady-state 与本地 V1–V4 恢复流程。
引用已有的 [blame 通信图](figures/blame-pipeline.pdf)，主稿已另有其区域图输入。

固定 B 的适配仍是新工作：无限 p-progress deadline 不得要求 B+1；末批 NEWVIEW anchor 重认证没有 commit timer，须另给 completion 机制。
控制后继/close 的编码、有效性和安全进展证明必须完成，不能凭接口名称假设已有实现。

### 9. Safety and Progress

建议结果顺序：

1. **Unique finalized anchor.** 正常 terminal batch 的 final value 唯一。
2. **Normal handoff safety.** 会话启动前校验 final anchor，归纳保持 prefix safety。
3. **Height-only authorization soundness.** 相同合法 `(g,b*)` witness 唯一解析为同一内容；委员会消息不能指定另一个内容。
4. **Emergency prefix preservation.** 明确依赖完整截止点、fencing、恢复数据和 L1。
5. **Faithful publication.** 发布区间来自正确区域的完整 finalized bytes；执行正确性另行组合。
6. **Normal progress.** Drain、健康 successor、传输、bootstrap、服务与 EVM 边界前提。
7. **Conditional recovery progress.** 事故关闭/收集、委员会协调与 L1 进展。
8. **Evidence soundness.** 合法 view/session 转移不会误罚。

这些是拟证明目标，不是本轮完成的正式证明。

### 10. Costs and Remaining Design Choices

分别计区域稳态、正常交接、L1 来源验证与慢事故成本。
当前不以委员会 checkpoint 取代区域证明；若减少证书体积，另研究 prefix/aggregate/recursive proofs。
不再计条件会话缓存或 speculative branch cancellation。
明确 closed-cut 实现、恢复资格激活、terminal completion、原生适配和 DA/execution verifier 尚需定稿。

### Appendices

- A. 区域证明与 terminal adapter。
- B. 消息编码、区域 receipt verifier 与 ResolveFinalizedBatch。
- C. 错误 cutoff、迟到 suffix、冻结竞态等反例。
- D. Native MegaETH adapter 与来源快照。

## 3. 当前 main_v2 的修订映射

以下定位基于本轮起始读取的 694 行版本，SHA-256 `d201da53bceb0df51f8be46784f9f7b576f8ce1f8bb512526c15e23d30f08e35`。
这些是未来修订位置，本轮不修改正文。
正文在本轮期间另有变化，因此行号仅对应上述快照，不声称对应当前最新磁盘版本。

| 当前位置 | v3 修订方向 |
|---|---|
| title/abstract，约 17–66 | 从 unfinished outer CFT 改成 finalized handoff 与高度授权恢复 |
| §1，69–96 | 去掉 crash-only regional wrapper，明示委员会权限范围 |
| §2.1，99–130 | 加入委员会、generation、全局 batch b 和截止高度定义 |
| §2.2，132–174 | 有效 session 启动前验证 finalized anchor，无中间确认状态 |
| §2.3，176–216 | 给黑盒与 Drain，区分物理/应用位置 |
| §3.1–3.2，219–312 | 保留区域流水线，加固定 quota 和 terminal 约束 |
| §3.3，314–411 | 保留 blame 图/等待规则，修改末批完成与越界进度要求 |
| §3.4，413–492 | 已有详细 slashing；沿用并扩展新 g/b/session 上下文 |
| §4，494–532 | 加预热 successor、height witness 和 L1 忠实发布 |
| §5，534–626 | 原区域结果保留前提，另证正常交接与截止高度唯一解析 |
| §6，628–673 | 加明确交接测量点，不把交接准备等同新区首批 final |
| bibliography，约 675–694 | 只增实际依赖的来源，不把新定理归给既有原语 |

## 4. 实际修订顺序

1. 固定 b/h/g、quota、Finalized 和 committee 权限对象。
2. 完成 FinalizeThenHandoff 的伪码、延迟口径和归纳证明。
3. 给 Sync HotStuff 实例的 Drain 及 quota deadline 完整适配。
4. 定义 E_rot / W* 分离，证明 ResolveFinalizedBatch 的唯一性。
5. 具体选择 closed-cut/fencing/恢复机制，再完成事故和 readmission 算法。
6. 写 L1 verifier、历史补发布、DA/执行边界和 costs。
7. 冻结新版本进行独立安全审查与排版验证。

## 5. 定稿前检查

- raw QC 不会授权新区启动；final receipt 验证后才能启动。
- 同一 b 的不同 witness 在安全模型内不能解析为不同内容。
- b*=100 不会覆盖已 final 到 105 的历史；故障观察较晚、正常 region 已切换也须涵盖。
- 委员会消息不含选择某个 block/首 header 的内容字段；解析结果来自区域证明。
- 空 generation 的 cutoff 复用已认证起始 checkpoint。
- fixed B 末批和跨 view Drain 都可完成，不要求新应用 batch B+1。
- FREEZE/publication 竞态、过期 nonce、历史补交与恢复资格生效边界无歧义。
- batch 高度和 mini-block 高度、原始 signer view 和重认证 view、L1 接受和最终性都不混用。
