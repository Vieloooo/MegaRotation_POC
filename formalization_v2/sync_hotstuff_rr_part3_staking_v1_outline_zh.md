# Part III v1 详细大纲：Staking、加权选主与 Authority

日期：2026 年 10 月 19 日。

本文整理本次讨论确定的设计，作为 Part III 第一版的写作大纲。正文分成三部分：**A. 数据与接口；B. 权重管理；C. 轮转与 authority 派生。** 最后列出与 Part II 的衔接、完整例子，以及正式稿仍须补齐的规则。

主要依据是 [Part II v4.5](sync_hotstuff_rr_part2_rotation_v4_5.tex) 和 [Part I](sync_hotstuff_rr_part1_core.tex)。已有 [staking v2](../formalization/staking_rotation_module_v2.tex) 的队列和历史责任设计可以复用；其中“每次交接都经过 L1”的旧流程不沿用。本文是设计大纲，不是已经完成的协议实现或安全证明。

## 0. 用几句话说明整体设计

L1 按较长的 staking epoch 批量处理成员和权重变化。每份快照提前发布，并在指定的未来 L1 高度激活。用户可以随时申请进入或退出，但申请不会立即改变正在运行的 BFT。

BFT 在正常轮转时，提前为下一 session 确定成员、权重和 leader 排班。**新 session 使用其直接前驱 session 的初始 anchor，读取这块的 BFT 高度和 L1 引用高度。** L1 引用高度决定用哪份 staking 快照，BFT 高度作为选主 seed。

Region 的候选顺序由 multisig 提前规定的 `randList(beta, r)` 提供，再按该分支的 status map 及目标快照中的正权重资格选出实际 region。区内 leader 才按 stake 权重抽样。

资金退出分成两件事：先停止接到新职责，再完成已经接到的职责。职责结束后，仍要留下足够的争议和罚没时间，最后才能提走剩余资金。

### 0.1 本版已经确定的选择

| 项目 | 本版规则 |
|---|---|
| Stake 更新节奏 | L1 上较长、固定周期的 staking epoch |
| Snapshot 生效 | 提前发布，在指定的未来 L1 高度激活 |
| L1 是否跟踪 BFT session 编号 | 不需要；L1 发布配置，BFT 根据公开规则采用配置 |
| Regular 选主参考块 | 直接前驱 session 的初始 anchor |
| 权重来源 | 参考块所引用 L1 高度上，已经激活的最新快照 |
| 基础 seed | 参考块的 BFT DATA 高度 |
| 每个 view 的输入 | 基础 seed 加完整 `(beta, r, g, t)`，再做哈希 |
| Region 选择 | Multisig 规定的候选顺序，加 status map 和正权重检查 |
| Session 内权重 | 固定；term rotation 和 recovery 都不重取快照 |
| Regular handoff | 继续使用 Part II 的 terminal receipt 和 Ready，不增加一次交接时的 L1 确认 |
| 历史证书 | 始终按产生它的 session 的历史配置验证 |
| 退出依据 | 职责确实结束、旧配置不能再授权新职责、追责窗口完成 |

本版不加入独立随机信标，也不让节点各自提交 VRF 竞争 leader。这里的 `WeightedRand` 是一个公开、确定性的加权选择函数。它的安全边界在第 3 节说明。

### 0.2 先把三个编号和两个 anchor 分清楚

| 记号 | 含义 |
|---|---|
| `e` | L1 staking epoch / 快照编号 |
| `(beta, r)` | 一个 BFT session；`r` 不是物理 region 的编号 |
| `(beta, r, g, t)` | 一个 BFT view；`g` 是 recovery generation，`t` 是区内 leader term |
| `A_s` | Session `s` 实际从哪里继续执行，即它的初始 anchor |
| `E_s` | 用来给 session `s` 选配置和 leader 的参考块 |

对 regular successor `s=(beta,r)`，其直接前驱是 `p=(beta,r-1)`：

$$
E_s=A_p,\qquad A_s=T_p,
$$

其中 `T_p` 是前驱的 terminal DATA。

用户给出的例子应直接放进正式稿：

$$
(0,0)\longrightarrow(0,1)\longrightarrow(0,2).
$$

- `(0,2)` 的选主参考块是 `(0,1)` 的初始 anchor，即 `(0,0)` 的 terminal DATA。
- `(0,2)` 实际执行的 anchor 是 `(0,1)` 的 terminal DATA。
- 所以 `(0,1)` 开始时，就可以算出 `(0,2)` 的配置和基础 leader 排班。
- `(0,2)` 仍要等 `(0,1)` 的合法交接和自身 Ready，才能开始生产。

本文按这个确切关系展开，不再使用含混的“隔几代”计数。

## A. 数据结构与公共接口

## 1. Stake：资金、身份、权重和职责

### 1.1 一笔 stake 至少记录什么

以 stake position 为基本对象。一个 position 表示一笔有明确所有者、运营密钥和 region 归属的抵押。

| 字段 | 含义 |
|---|---|
| `positionId` | 永久、唯一的抵押记录编号；退出后也保留历史记录 |
| `owner` / `withdrawalAddress` | 谁有权申请资金操作，资金最终发给谁 |
| `operatorKey` | 实际参加 BFT 的签名公钥，注册时证明持有对应私钥 |
| `regionId` | 节点所属 region |
| `balance` | 合约中尚未被提走或罚没的实际余额 |
| `pendingRequests` | 尚未处理完成的加入、增仓、减仓或退出申请 |
| `exitEpoch` | 从哪份未来配置起，不再把它纳入新快照 |
| `slashRecords` | 已经受理、执行和仍待处理的罚没记录 |
| `releaseState` | 是否取得退休证明、何时结束追责等待、是否仍有资金被案件占用 |

第一版建议使用一种抵押资产和明确的 region 归属。迁移 region、换运营密钥都要经过公开登记和相应生效边界，不能用改字段来甩掉历史责任。委托、复杂份额凭证和多资产折价可以留到后续版本。

为简化验签和计权，第一版要求一个运营公钥不能同时绑定多份有效 position。历史 position ID 不复用，避免一份签名被重复计权，或把旧责任转嫁给后来登记的资金。

### 1.2 三种金额不能混在一起

1. **实际余额**：现在合约里还有多少钱。
2. **有效 stake**：已经完成入场条件、可以进入下一份快照的金额。
3. **快照权重**：某份已冻结配置中记录的金额，供该配置下的 BFT 验证使用。

第一版可按同一种资产的最小整数单位线性计权。新存款、退出申请和 slash 都可能改变未来有效 stake，但不会倒过来改写历史快照。

例如，一个节点的历史权重为 100，后来余额被罚到 70，旧证书仍按权重 100 验证。这里保证的是历史验证一致，不是声称还有 100 的抵押品可供再次罚没。

### 1.3 职责单独记录

一笔资金可能同时支持正在运行的 session 和已经提前安排的后继 session。资金状态与职责状态要分开，不能只用一个 `Active / Exited` 标志表达全部含义。

职责记录至少绑定：position、目标 session、快照、region、预分配依据，以及完成或取消的依据。预分配给某 region 时，该 region 的全部投票成员都有职责，不只是首任 leader。

这些职责首先由 BFT 的公开历史确定。不能假设 L1 合约天然知道所有职责；L1 释放资金所需的证明接口见第 8 节。

## 2. 周期快照：L1 发布什么，BFT 读取什么

### 2.1 Staking epoch 是 L1 自己的周期

建议第一版用固定的 L1 高度间隔定义 epoch：激活边界为 `a_e = a_0 + e * M_stake`。`M_stake` 是一个相对较大的协议参数，不跟随 BFT session 的快慢改变。

每个周期预先规定快照截止点和发布安排，并为未来激活留出公告时间。具体数值以后根据 L1 最终性、配置传播和节点准备时间确定。

- 用户申请可以随时提交；每个 epoch 按队列和预算统一处理。
- 同一个 epoch 内的普通入场、退出变化不连续重算权重。
- Slash 可以及时扣款和记录处罚，其对后续配置的影响在配置规则中落实。
- 一个 staking epoch 可以覆盖多个 BFT session。
- 一个很长的 session 也可以跨过多个 staking epoch，但它继续使用原来的快照。

### 2.2 快照对象

每份 `StakeSnapshot C_e` 包含：

| 字段 | 含义 |
|---|---|
| `epochId` | 快照编号 `e` |
| `snapshotCut` | 取 stake 状态的确切 L1 截止位置 |
| `publicationPosition` | 配置记录在 L1 的发布位置 |
| `activationHeight a_e` | 这份配置从哪个 L1 高度起可供新选择采用 |
| `membersAndWeights` | 每个 region 的 position、公钥和正权重 |
| `snapshotRoot` / `cfgId` | 绑定完整配置的承诺和标识 |
| `ruleVersion` | 计算快照所用的公开规则版本 |

快照截止位置可以用区块高度加日志位置表达；正式稿必须说明读取操作前还是操作后的状态。这样同一 L1 区块内的多笔增减仓不会留下歧义。

快照截止、发布、激活是三个事件。配置记录应在其未来激活高度之前发布。晚来的维护调用不能移动原定截止点，也不能挑选更有利的队列前缀。

### 2.3 `LatestStakingSnapshot(l)`

对一个已经验证的 finalized L1 引用高度 `l`，定义：

$$
\operatorname{LatestStakingSnapshot}(l)
=C_{e^*},\qquad e^*=\max\{e:a_e\le l\}.
$$

这个接口读取的是**截至 `l` 已激活的最新快照**。未来才激活的配置，即使已经公布，也不参与此次选择。

验证者必须能确认有关配置记录的完整性。只有某个旧快照的存在证明，不足以证明它是最新的。缺配置或缺权重数据时等待，不能默默退回更旧的快照。

快照在 `a_e` 激活，不表示所有正在运行的 session 当场换权重。它只是开始对符合读取规则的新排班可用。

## 3. 加权选择：给定权重和 seed，唯一选出 leader

### 3.1 纯函数接口

```text
WeightedRand(有序成员及整数权重, 输入值) -> 唯一成员，或 EMPTY
```

同样的输入必须得到同样的结果，函数内部不读取本地时钟、不重新查当前 stake，也不调用本地随机数。

一种简单实现是：按固定的 position ID 顺序排列成员；把总权重分成连续区间；从输入值确定一个总权重范围内的整数，看它落在哪个区间。

例如权重为 `(60,30,10)`，区间为 `[0,60)`、`[60,90)`、`[90,100)`。若抽样整数为 73，就选择第二个成员。整数抽样应采用明确的无偏转换，例如拒绝采样，避免随意取模带来的额外偏差。

总权重为零时返回 `EMPTY`。不能临时指定一个没有权重的 leader。

### 3.2 每个 view 的基础 leader 映射

对 session `s=(beta,r)`，设选主参考块的 BFT 高度为 `h_ref`，目标 region 的固定权重表为 `W_s`：

$$
u_{s,g,t}=H(\texttt{LEADER},\mathit{chainId},h_{\rm ref},\beta,r,g,t),
$$

$$
\operatorname{BaseLeader}(\beta,r,g,t)
=\operatorname{WeightedRand}(\mathcal W_s,u_{s,g,t}).
$$

不同 view 可以再次选中同一节点。唯一性的意思是“同一 view 只有一个计算结果”，不是“每个 view 必须换一个不同的人”。

固定 snapshot、参考块的完整身份和算法版本都放在 authority 中验证。本版基础 seed 仍是约定的 BFT 高度；不额外掺入可由 leader 调整的交易内容或区块完整哈希。

### 3.3 这项设计承诺到哪里

在理想均匀抽样输入下，单次选中概率与固定权重成比例。实际采用公开高度哈希时，输入可以提前计算，所以不能直接推出不可预测性、抗身份选择操纵或长期收益公平性。

正式稿先把它定义为**公开的确定性加权排班**。以后若要增强随机性，可以替换 seed 来源，保留本节的选择接口。第一版不为此增加新的随机信标协议。

本节给出正常情况下的基础排班。已证明双签者如何从未来 leader 资格中排除，单独在第 7.3 节处理，避免把“投票权重固定”和“以后永远不能禁任 leader”混为一谈。

## 4. Region rotation：候选顺序与资格分开

### 4.1 候选顺序接口

对当前 session `(beta,r)`：

$$
\Pi_{\beta,r}=\operatorname{randList}(\beta,r)=(i_1,i_2,\ldots,i_m).
$$

这是给下一个 regular session `(beta,r+1)` 使用的候选顺序。列表或其生成规则由 multisig 提前公布，绑定规则版本，并在对应 branch 内固定。

列表不含重复或未知 region。第一版可要求覆盖固定 region 集合。是否把当前 region 放在最后，由生成规则明确规定，不留给接收者临时决定。

`randList` 是接口名。本版不按各 region 的总 stake 随机挑选 region，也不在轮转时临时重新打乱列表。

### 4.2 实际选择接口

```text
SelectRegion(候选列表, branch 的 status map, 目标 session 的快照)
    -> 列表中第一个 ON 且总权重大于零的 region
    -> 若全部不合格，返回 PENDING
```

例如候选顺序是 A、B、C、D，A 为 OFF，B 为 ON 且有正权重，就选 B。

- 使用目标 session 的权重快照检查总权重。
- 没拿到 B 的配置材料时等待，不能把它当作零权重而跳过。
- B 没响应，不自动改变候选顺序或 status map。
- 选定 region 后，整个 session 的 recovery 都留在该 region。

Multisig 更新仍遵守 Part II 的公开生效边界。第一版建议把排班规则版本与 `S_beta` 一起绑定到 branch；新 branch 采用新规则，旧历史不重算。

## 5. 提前安排与正式 authority

### 5.1 先有排班，再有启动权限

为了说清楚提前准备，区分两个对象即可：

| 对象 | 已经固定什么 | 允许做什么 |
|---|---|---|
| `Plan_s` | 目标 session、参考块、快照、region、基础 seed 和选主算法 | 提前同步、准备运行环境、计算 leader |
| `Authority_s` | `Plan_s` 加实际交接 anchor、合法边界证据和 Part II 参数 | 经 Part II 的 Ready / Enter 后运行 BFT |

`Plan_s` 不能凭空生成 terminal receipt，也不授予提前投票或出块的权利。目标初始 anchor 尚未出现时，不把它伪装成已经完整的 authority。

### 5.2 Authority 的最小字段

Authority 至少绑定：链标识、目标 `(beta,r)`、配置 ID、选定 region、参考块的原身份与哈希、`h_ref`、`l_ref`、region 排班规则版本、leader 算法、实际初始 anchor，以及交接证明。

DATA quota、`K`、区域延迟界限与证书阈值沿用 Part II 的公开参数来源；不能因为 stake 改了，就在正在运行的 session 中重新设置 quota 或阈值。

`VerifyAuthority` 检查这些字段能否由同一份已认证历史唯一推出。同一 session 的已固定字段不能由后来的消息覆盖。

### 5.3 对外接口一览

| 接口 | 用途 |
|---|---|
| `LatestStakingSnapshot(l)` | 取得截至指定 L1 高度已激活的最新配置 |
| `RegionalWeights(cfgId, region)` | 读取固定成员和权重 |
| `WeightedRand(weights, input)` | 执行确定性的加权选择 |
| `randList(beta,r)` | 给出当前 session 的 regular 后继候选顺序 |
| `SelectRegion(list, status, snapshot)` | 选择实际服务 region |
| `PrepareNext(currentSession)` | 用当前 session 的初始 anchor，准备下一 session |
| `Leader(authority,g,t)` | 返回该授权上下文中的 leader |
| `VerifyAuthority(authority, evidence)` | 验证配置、参考块、region 与交接来源 |
| `Join / RequestExit / Slash / Withdraw` | 管理资金及其资格；具体步骤见 B 部分 |
| `VerifyRetirement(position, proof)` | 为资金释放验证职责终止；见第 8 节 |

接口需要区分“资料缺失，等待”“证据无效，拒绝”和“完整检查后无合格候选”。这三个结果不能都解释成可以换下一个人。

## B. 权重如何变化：进入、退出和 Slash

## 6. 周期队列与成员生效

### 6.1 从以太坊借鉴什么

借鉴三件事即可：申请排队、用变更额度控制进入和退出、退出资格与提款资格分开。以太坊的 pending deposit 处理还检查最终性和处理数量；退出通过队列确定 exit epoch，之后才到 withdrawable epoch。这里借鉴流程，不复制其金额和等待天数。[Fulu pending deposits](https://ethereum.github.io/consensus-specs/specs/fulu/beacon-chain/#modified-process_pending_deposits)、[Electra validator exit](https://ethereum.github.io/consensus-specs/specs/electra/beacon-chain/#modified-initiate_validator_exit)

### 6.2 进入流程

1. 用户存入资金，登记 region、运营公钥和提款地址，证明密钥持有与操作授权。
2. 申请按 L1 的确定顺序进入队列。排队阶段没有新的投票或 leader 权利。
3. 满足入场 timelock，并按 epoch 的处理预算选入未来快照。
4. 快照在预定 L1 高度激活。
5. 某个 session 的选主参考块读到这份配置后，节点才被纳入相应排班。
6. 目标 session 正式启动时，节点还要满足 Part II 的同步、状态和 Ready 条件。

因此，入场 timelock、快照激活等待、提前排班等待是不同阶段。已经被配置或排班使用的资金不能按“申请还没有实际出块”直接退款。

### 6.3 退出与减仓流程

1. 所有者提交退出或减仓申请，按 L1 顺序排队。
2. 满足退出 timelock 和 epoch 预算后，确定从哪份未来快照开始减少权重或删除成员。
3. 旧快照及其已经分配的职责继续有效；正在运行的 session 不换权重。
4. 证明该笔资金不再可能接到新职责，已有职责也已经结束或合法取消。
5. 开始剩余的追责等待；及时提出的未决案件继续占用相关资金。
6. 所有条件满足后，只向提款地址释放剩余可提余额。

第一版可以先支持整笔退出。若支持增仓、部分减仓，也走相同的周期处理和责任保留规则，不另设实时改权重的捷径。

### 6.4 更新额度

每个 staking epoch 分别设置流入和流出额度，必要时同时限制 position 数量。额度按 L1 epoch 计，不按 BFT session 计，否则 session 转得快就会绕过限速。

- 流入与流出分别计账，不能只限制净变化。
- 可设置全局和每个 region 的额度。
- 大额申请采用明确的分批处理或金额上限，避免队首永远放不下。
- 取消和已经完成的申请不能堵住队列。
- Slash 扣款和由此引起的强制移除单独计账，不把它们伪装成受到普通退出额度约束的自愿变化。

限速只能限制变化速度，不能保证每个 region 永远满足诚实权重条件。

## 7. Slash：先证明违规，再处理资金和未来资格

### 7.1 第一版关注能直接验证的签名冲突

以太坊把 slash 与退出安排、资金处罚联系起来。我们保留这种联系，但违规谓词必须按自己的 BFT 消息定义，不能照搬以太坊的投票格式。[Electra slashing](https://ethereum.github.io/consensus-specs/specs/electra/beacon-chain/#modified-slash_validator)

| 违规类型 | 比较范围 | 核心冲突 |
|---|---|---|
| Leader 重复提案 | 同一授权 session、`g,t,j` | 签署两个不同且满足历史准入条件的 DATA / DRAIN 提案 |
| Fresh vote 双签 | 同一授权 session、`g,t,j` | 对不同块投票；DATA / DRAIN 共同受一次 fresh vote 约束 |
| NEWVIEW 双签 | 同一 session、同一 recovery generation | 宣布两个不同 opening；即使两者有祖先关系也算不同 opening |
| ANCHOR-VOTE 双签 | 同一 session、同一 generation | 对两个不同 opening 投 ANCHOR 票 |
| FINAL 冲突 | 同一 session、同一 DATA 高度 | 为不同块签 FINAL；不能要求 `g,t` 相同，因为 FINAL 本来不绑定它们 |

Leader 提案和 NEWVIEW 的证据检查与 Part II 已有 equivocation 条件衔接，包含所需的历史授权、admission 和 quota 依据。其他谓词逐条从 Part II 的禁止行为定义，不笼统写成“同一高度签过两个哈希”。

证据绑定历史 position、公钥、链、配置、region、session 和其初始 anchor。Force 后在新 branch 重用某个 DATA 高度，不自动构成双签。

超时、BLAME、地区 OFF、正常恢复、转发别人的冲突证据，都不直接构成以上违规。同一 opening 的不同 QC 编码也不是 NEWVIEW 双签。

### 7.2 证据处理流程

```text
任何人提交完整证据
    -> 验证签名、历史授权和冲突条件
    -> 检查该违规是否已经处理
    -> 记录处罚并扣减对应 position 的余额
    -> 更新未来资格与提款限制
```

一份确定性的双签证明不需要再收集一个“同意罚没”的区域 quorum。若证据已经足够完成裁决，可直接执行；需要补材料或有未决争议的案件，保留其资金占用，直到处理完成。

去重以 position、违规类型和冲突位置为依据，不能只按证据字节哈希去重，否则同一次双签换一种包装就可能重复扣款。多次处罚总额不能超过剩余可罚余额。

罚没比例、是否全额罚没、举报奖励作为少量公开参数单独规定，不让举报者或当前 leader 决定。

### 7.3 扣款不等于重算当前委员会

Slash 有两个后果，应分别描述：

- **资金后果**：扣减当前实际余额，并在未来 staking 配置中反映。
- **选主资格后果**：已证明双签者应被排除出后续合适的 leader 授权。

第二项是 Part I / II 的进展条件所需要的。不能只扣钱，却让同一个已暴露的作恶者无限出现在后续 leader 中。

建议保留 `LeaderEligibility(session,generation,penaltyCheckpoint)` 作为窄接口：新 generation 固定一份共同的处罚截止记录，从历史成员中排除已禁任 leader 的身份，再按原权重数值执行加权选择。投票快照与历史证书阈值保持不变。

正式稿须补齐：处罚截止点如何唯一确定、排除从哪个尚未固定的授权起生效，以及没有剩余候选时如何等待或交给 Force。不能用各节点“目前知道的最新处罚”自行重排，也不能反悔修改已固定 view 的 leader。

因此，“提前知道排班”指在当时已固定的资格上下文下知道基础排班；未来依法生效的 leader 禁任按独立授权规则处理。若第一版暂不完成这一接口，就应保留 Part I 的排除假设，不能声称已经实现完整的进展保证。

## 8. 冻结和提款：按 Part II 的实际职责计算

### 8.1 冻结的三段时间

一笔资金的总锁定期包含：**排队与预留时间、实际服务时间、服务结束后的追责时间。** 提前排班主要延长前两段，不意味着必须照搬以太坊的固定退出天数。

例如 `(0,1)` 开始时，节点已被安排到 `(0,2)`。它随后提出退出，也必须支持 `(0,2)` 的既定职责。若同一快照还用于后续 session，责任会继续延伸，直到出现可信的“不会再接新任务”的边界。

### 8.2 什么算职责结束

本节严格使用 Part II v4.5 的停止条件：

| 情况 | 可使用的结束依据 | 资金释放还需要什么 |
|---|---|---|
| Regular 完成 | 完整 terminal FINAL receipt 触发 Completion | 将相应完成与后续边界纳入可验证的全局历史，处理其他已分配职责 |
| Force 关闭 branch | 已生效 Force 的公开 cutoff，按 Part II 终止旧版本程序 | 验证对应 L1 控制、Espresso 激活位置与全局重放，处理旧证据和未决责任 |
| 预排 session 从未启动 | 证明其所在 branch 已关闭或安排已按明确规则取消 | 证明不能在别处重启该安排；已经发生的其他职责仍然有效 |

仅有 terminal QC、某节点本地停止、region OFF、stake epoch 更新，都不是完整的提款依据。程序停止也不自动允许删除数据；保留义务需要完成或移交。

正常接棒仍可早于 global finality。**提款可以等全局证据，出块接棒不因此等待提款流程。**

### 8.3 退休证明接口

建议第一版集中使用一个接口：

```text
VerifyRetirement(position, proof) -> 退休已成立，或尚未成立
```

证明至少说明：

1. 该 position 已按 L1 队列规则从未来配置退出。
2. 存在协议上可执行的边界，之后不能再用旧快照为它分配新职责，包括 Force 路径。
3. 此前已运行和已预留的职责都已结束或合法取消。
4. 对应事实可由 finalized L1 和 Espresso 历史、Part II 的重放规则验证；需要的数据责任已履行或可靠移交。

L1 无须为每个普通 session 更新一笔 staking 状态。退出时，由公共验证者基于这些历史构造证明，L1 的验证接口检查相应结论。

这需要一种合约可以验证的证明或既有受验证的全局状态接口；**Espresso 的收录证明本身不等于退休结论正确。** 具体证明封装留给验证层，但上述四项条件不能省略，也不交由待退出者自行签字确认。

停止新增职责的规则仍需正式化。仅有 `LatestStakingSnapshot(l)` 不够：旧 L1 引用可能长期不推进，Force 也可能从较旧 anchor 重启。必须证明旧快照已经不能再授权这些资金，而不是假设换过一个 epoch 就自然满足。

### 8.4 留多长的争议和罚没时间

冷却的起点应是**完整退休结论已经被验证、足以调查相关职责的材料已经可获得**的事件，不是提交退出请求的时间，也不是预计最后一个 session 会结束的时间。

Part II 已有的时间界限可以帮助拆分预算：

| 时间 | Part II 已经保证的范围 | 在退出流程中的用途 |
|---|---|---|
| `T_pub` | 完整区域结果已经可用后，提交给 Espresso 的时间 | 估计公共材料交付尾部 |
| `T_E` | Espresso 收录、最终化和对持续在线读者的可验证交付 | 确认公共历史和相关材料已经可用 |
| `T_L1` | 已实际提交的有效控制操作的最终化与交付 | 若用于 slash / 退休交易，必须显式扩展相应服务假设 |

正式稿再定义一段统一的 `T_dispute`，覆盖尚未完成的材料交付、调查及构造证据、提交并使有效 slash 被 L1 受理，以及安全余量。若退休证据已经证明某段交付完成，就不重复等待那一段。

具体预算需要补上证据发现和提交服务的条件。Part II 的 `T_pub + T_E` 从“完整结果已经可用”才开始，不能覆盖任意长的停机、秘密保留的双签证据或无人提交的案件。

合约计时使用哪个 L1 时钟或高度单位也要写清楚。Part II 的网络时间界限不能未经换算就变成固定块数；正式参数应采用明确的时钟假设和保守余量，不用平均出块时间当保证。

追责截止前已有效受理的案件，不能因为冷却期结束就失效；其占用的余额要保留到处理完成。单纯提交无效材料不能无限冻结别人资金。

第一版在退休证明未成立时保持锁定，不凭“已经过了两个 session 的平均时长”放款。因此暂不承诺无条件的固定提款期限。

### 8.5 Espresso 最终性怎样简化这里的问题

本系统的区域结果经 Espresso 排序、最终化，再由 Part II 结合完整 L1 控制重放得到全局前缀 `G`。一旦进入 `G`，后续 Force 不会删除它。恢复和退出判断可以依赖这套外部认证历史，不让已经退出的区域节点重新投票决定过去发生了什么。

在既定 L1、Espresso 信任锚和历史可验证性假设下，Part III **不再额外引入一套由本层旧质押者或社会协调来选择区域历史的机制**。这避免了让同一组待退出资金同时证明“自己已经没有责任”的循环。

这里与以太坊 weak subjectivity 的比较仅限于本层的历史认证方式；以太坊的相关规则服务于其自身 PoS 历史同步。[Ethereum weak subjectivity](https://ethereum.github.io/consensus-specs/specs/phase0/weak-subjectivity/)

两个已有边界仍须保留：Espresso 收录一份原始材料不等于该材料已经通过 Part II 进入 `G`；已经固定的历史也不自动禁止旧密钥为尚未结束的旧授权补签新材料。后一个问题由退休边界、授权失效和历史责任窗口处理。已有外部信任锚不需要在 Part III 中重新发明，但也不能被省略。

## C. 轮转模块：Seed、下一任 Leader 与 Authority

## 9. Regular rotation 的完整派生过程

### 9.1 当前 session 开始时，准备下一 session

当前 session 为 `p=(beta,r)`，它的初始 anchor 为 `A_p`，目标为 `s=(beta,r+1)`。

按以下顺序执行 `PrepareNext(p)`：

1. 验证 `p` 的已固定初始 anchor 和所属交接历史。
2. 令 `E_s=A_p`，读取 `h_ref=h(E_s)`、`l_ref=lambda(E_s)`。
3. 取得并验证 `C_s=LatestStakingSnapshot(l_ref)`。
4. 读取 `randList(beta,r)` 和该分支固定的 `S_beta`。
5. 按候选顺序，使用 `C_s` 中的权重选择实际 region。
6. 固定 `Plan_s`；根据参考高度和目标 view 编号计算基础 leader 排班。
7. 目标成员开始跟随当前 session，准备未来接棒所需的状态。

对目标 `(beta,r+1)`，列表参数使用**当前** `(beta,r)`；leader 哈希中使用**目标** `(beta,r+1,g,t)`。这两个编号不能混用。

参考块本身保留原来的生产 session、hash 和 L1 引用。不能因为它被新 session 使用，就改写它的原身份。

### 9.2 真正接棒时，补齐实际执行 anchor

当前 session 获得 terminal DATA 的完整 receipt 后，下一 session 的实际初始 anchor 固定为 `T_p`。此时：

- 使用先前已经固定的 `Plan_s`，不按收到 receipt 时的最新 L1 状态重取配置。
- 检查目标 branch 仍然有效，排班没有被 Force 取消。
- 验证 `T_p`、交接证明、配置及其依赖。
- 准备并验证 `T_p` 对应的执行状态。
- 形成完整 authority，并按 Part II 的 Ready / Enter 启动。

目标第一次提案仍从 `T_p` 继续执行，不从较早的 `E_s` 继续执行。Regular handoff 不额外等待一次 L1 staking 更新或 Espresso 确认。

### 9.3 Session 内的 term rotation 和 recovery

进入 session 后，以下内容始终固定：选主参考块、`h_ref`、`l_ref`、stake 快照、region、投票成员与权重、DATA quota。

- `t` 增加时，从同一基础输入派生下一 term 的 leader。
- `g` 增加时，保留相同 session 配置，并使用该 generation 的合法 leader 授权。
- Recovery 选出的新执行 anchor 不替换 `E_s`。
- 因为 leader 先确定、再接收 STATUS 选择恢复 anchor，不会产生“先知道恢复 anchor 才能选恢复 leader”的循环。

## 10. Genesis、Force 和等待状态

### 10.1 Genesis

第一份 session 没有前驱，使用 genesis checkpoint 明确给出的配置、参考块和排班上下文。第一份 session 开始后，即可按普通规则用其初始 anchor 准备第二份 session。

初始两个 session 因此可以使用相同的参考高度和快照；完整 view 编号区分其选主输入。

### 10.2 Force

Force 保留 Part II 的语义：在规定的 Espresso 激活位置关闭旧 branch，从当时的 `G` 末尾重启；不能从节点本地最靠前的 speculative session 选择前驱。

- 旧 branch 中未启动的预排安排失效，已经运行的程序按 Part II 的 Force-close 停止。
- 取消未来安排不抹去过去已经发生的签名、发布或数据责任。
- 新 branch 首个 session 没有普通同分支前驱，需要专门的 bootstrap 上下文。

**建议的最小 bootstrap 方案，待正式稿确定：** 使用 Force 固定保留的 `G` 末块作为首个 session 的选主参考块；使用新 branch 的排班规则与 status map，通过单独的 Force 适配接口选择 region。首个新 session 建立后，后续 regular session 再恢复第 9 节规则。

Force 的目标 `r` 可能复用，候选列表如何对应新分支首个目标也需要明确；不能机械套用 regular 的 `r+1`。退休规则还必须防止 Force 重新授权已经获准提款的旧资金。

### 10.3 缺资料和无候选

缺 L1 历史、快照、签名证明或执行状态时等待补齐。候选列表经过完整检查后都不合格，才是 PENDING。故障超时不授权重新选快照、修改 seed 或自行改变 region 状态。

## 11. 与 Part II v4.5 的具体衔接

| Part II 当前接口 | Part III 提供的实现或需要明确的调整 |
|---|---|
| L1 直接提供 `AUTH_(r,g)` | L1 主要发布周期快照及控制；BFT 根据参考块、排班规则和交接历史派生 session authority |
| Authority 只按 `r` 索引，Force 复用 `r` 时复用原记录 | 本方案需要按 `(beta,r)` 绑定派生结果、保留显式 `cfgId`；Force 复用规则应同步修订 |
| 固定循环 `NextLive` | 用 `randList(beta,r)` 提供候选优先级，再检查 `S_beta` 和目标快照中的正权重 |
| Session 内固定成员、权重与 quota | 保留；本模块只在新 session 的授权中采用配置 |
| 签名绑定配置与初始 anchor | 保留，并把派生 authority 的来源验证纳入 admission |
| Regular 由 terminal receipt 交接 | 保留；提前排班只减少准备时间 |
| Force 的全局 cutoff 与 `G` 保留规则 | 保留；补足新的配置 bootstrap 和退休限制 |
| 恢复等待有效 authority | 明确派生 authority 和处罚截止记录的唯一来源及可用性，不再含糊依赖本机最新状态 |
| 历史 FINAL / QC 验证 | 使用原 session 的 authority 和快照，不能查询当前余额重算 |

正式接入时需要检查 Handoff、ReadyBundle、CheckFINAL 的配置上下文、L1 引用下界和 authority 验证接口。区域节点与全局重放必须采用同一套新的配置派生和选区规则。Part II 关于 `G` 只追加、Force 保留已有 `G` 的语义继续保留。

Part II 的区域故障假设仍按冻结权重计算：`F_i < W_i/2`，证书要求权重 `> F_i`。开放 staking、限速和罚没本身不证明每个 region 满足这个条件。跨 session 允许新身份进入后，也不能直接照搬 Part I 关于全局有限作恶身份集的全部进展论证。

## 12. 一个贯穿全文的例子

以下高度仅用于解释规则，不是部署参数建议。

1. L1 配置 `C_7` 在高度 2000 激活，下一份 `C_8` 在高度 3000 激活。
2. `(0,0)` 的 terminal DATA 是高度 100，它引用的 L1 高度为 2500。
3. `(0,1)` 从这块开始，因此 `A_(0,1)=T_(0,0)`。
4. 为 `(0,2)` 选主时，取 `E_(0,2)=A_(0,1)`，得到 `h_ref=100`、`l_ref=2500`。
5. `LatestStakingSnapshot(2500)=C_7`。读取 `randList(0,1)=(A,B,C,D)`；若 A 为 OFF、B 为 ON 且具有正权重，就为 `(0,2)` 选定 B。
6. 若 B 的三个成员权重为 `(60,30,10)`，则各 view 根据 `H(LEADER, chainId, 100, 0,2,g,t)` 在这个固定权重表中选主。
7. `(0,1)` 最终生产到 DATA 120。`(0,2)` 实际从 120 的执行状态开始，选主参考块仍然是 100。
8. 即使 120 引用的 L1 高度已超过 3000，`(0,2)` 仍用已经固定的 `C_7`。它开始后，为 `(0,3)` 选主时才读取 `A_(0,2)=120` 的引用，因而可能采用 `C_8`。

这个例子同时说明：快照可以被多个 session 共用；更新只在 session 边界被采用；权重选择比实际接棒提前一个 session；执行不会退回选主参考块。

若 B 的第二个成员在 `(0,1)` 运行期间申请退出，它仍须履行已经分配的 `(0,2)` 职责。只有旧快照不能再给它分配任务、已有职责结束、退休证明成立并完成追责等待后，剩余资金才可提走。

若 `(0,1)` 期间发生 Force，旧 branch 上预排的 `(0,2)` 失效。新 branch 即使复用编号 `r=2`，也不是原先那个 session。

## 13. 正式稿还须补齐的少量规则

| 待定项 | 为什么需要明确 | v1 的处理方向 |
|---|---|---|
| 长 epoch、快照截止与公告间隔 | 保证配置更新可重现，并留出准备时间 | 固定 L1 周期；不凭 BFT 速度临时改变 |
| L1 引用推进与旧快照退休 | 仅“不下降”可能让旧配置长期被采用，影响退出 | 给出可验证的退休边界，并覆盖 regular 和 Force；不能用本机看到的最新高度 |
| 退休证明的链上验证方式 | L1 不天然知道 BFT 职责是否已经结束 | 使用外部最终历史和 Part II 重放结论的可验证接口，不增加区域社会共识 |
| 争议窗口与证据服务 | 实际职责可能长于预计时长，材料交付也不是违规发现 | 从验证退休与材料可用后计时，给足调查和 L1 执行余量；保留及时未决案件 |
| 禁任 leader 的生效边界 | Slash 后需要排除未来 leader，但不能改历史投票快照 | 共同的处罚截止记录与尚未固定的 generation 授权 |
| Force 的首次选区及 seed 上下文 | 可能复用 `r`，也没有普通同分支前驱 | 单独 bootstrap，正常后继继续使用前驱初始 anchor |

正式稿应分别说明并检查：相同历史得到相同配置与 leader；提前排班不会授予提前出块权；同一 session 的权重不会中途变化；退出不会逃掉已分配责任；Force 不会重新使用已释放的资金；历史证书不会被后来的余额改变。

这些规则补齐之前，可以把主流程写清楚，但不把尚未证明的退休条件、随机公平性或无条件提款期限写成已成立的结论。

## 14. 写作顺序与篇幅安排

正式 Part III 按本文 A、B、C 三部分组织：

1. 先给出数据对象和五个核心接口，让读者看懂 stake、snapshot、加权 leader、region 顺序和 authority 分别是什么。
2. 再写周期队列与 slash，说明新权重怎样产生、旧权重为什么不能改、资金什么时候可以释放。
3. 最后用 `(0,0) -> (0,1) -> (0,2)` 的例子串起参考块、快照、seed、region 选择、提前排班和正式启动。

正文保留必要公式和状态表。具体签名编码、整数采样细节、罚没证据字段及时间预算放入附录。以太坊作为进入／退出流程的参考；Part II 和 Espresso 的实际接口决定本系统的停止、历史验证和追责规则。
