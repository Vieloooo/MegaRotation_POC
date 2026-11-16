# Sequencer Rotation PoC v2 实现与验收计划（M1–M5）

2026-11-16 · 计划稿

在单机启动三个区域的 15 个真实 replica worker，依次验证正常接棒、L1 主动 Force、地区故障后的自动 Force、从派生链 `D` 恢复，以及 Espresso 停滞解除后的定时恢复。每个里程碑都以正确新 session 的首个合格区块被全局最终性规则（GFR）接纳进 `G` 为端到端终点。每次运行产出可独立检查和浏览器回放的确定性证据。[PoC v2 设计](../../../formalization_v10/poc_design_zh_v2.md) · [M1/M2 既有计划](README.md)

## 1. 实施顺序与交付条件

| 里程碑 | 需要实现 | 主验收场景与通过条件 | 实际接线依赖 |
| --- | --- | --- | --- |
| **M1 · 15 个健康节点轮转** | 接通 Session Program、Session Manager、Publisher、SimEspresso 和 GFR。 | 固定 seed，从 R0 换到 R1 再到 R2。分别确认 h10、h20 驱动接棒，R2 的 h21 进入 `G`。 | M0、H1/H2/H3、已有 finalized L1 fixture。 |
| **M2 · 健康节点主动 Force** | 接通真实 L1 控制交易、公开激活、旧分支关闭及新 session 创建。 | 15 个 worker 存活时主动提交 Force。激活时保留既有 `G`、关闭旧分支的所有 `cid`，正确的新分支按时准备并有首块进入 `G`。 | M1 健康运行链路、H4 和功能 09 的 Anvil/ControlAuthority。 |
| **M3 · 地区故障与自动 Force** | 实现公开停滞检测、治理多签，以及读取已生效 L1 区域 policy 的 Plan。 | 当前负责地区停止生产，Espresso 的 finalized 日志和水位继续前进。多签委员会观测到超时后自动提交 Force，由合格地区接棒并让首块进入 `G`。 | M2 的 L1 和 Force 路径。 |
| **M4 · RESUME 语义与 D** | 接入独立 `D`、位置绑定的 `ResumeCut`、RESUME 的 `G` 对齐和 Manager 恢复接线。 | 用预设流程提交合法 RESUME。`G` 对齐到该记录指定的 `D(h_res)`，新 session 用对应的 origin 和执行状态生产 `h_res+1`，最终由 GFR 接纳。 | M3 的治理授权，以及 09 的 Inbox、11 的批次/DA 和 12 的派生组件。 |
| **M5 · Espresso 停滞与自动 RESUME** | 接入停滞/恢复观测、多签自动 RESUME、目标高度求解和恢复计时。 | 停滞时 `L1` 与 `D` 继续推进，GFR 在缺水位的位置等待。恢复后满足治理条件时，多签在预算内提交。GFR 按时对齐正确 anchor，恢复区块进入 `G`。 | M3 的委员会和区域 policy、M4 的真实 RESUME 路径。 |

- **已有基础。** M0 集成基线为 `35d382ac445cc8bc779f0d18fd23cb57e54680da`。M1 的 `9154785732d803c1f2f69aee3b2af67220a12f58` 已有 seed 42 满窗口记录，Tester 核对了 15 个 PID、21 个公开区块及两次交接。完整 M1 仍按既有矩阵验收。功能 11 的批次/DA 组件已有真实 Inbox 样本，交付点为 `aab8a5180801ed7f97f7e437d147c97f972b4dd8`。功能 12 已验证无批次回退及历史切面，交付点为 `1f6d1b3b4076ff16908dc59f272a06686b1d1ea5`。接入功能 11 的真实批次输入是 M4 的集成工作。[本机 M1 Tester 证据](../../../.build/commander-jobs/m1-regular/artifacts/healthy-smoke-91547857-seed42-run01-tester/tester.json) · [M1 完整验收](M1-regular-merge-zh.md) · [批次/DA 交接](../handoffs/11-batch-services-zh.md) · [D 交接](../handoffs/12-derivation-zh.md)
- **共同配置。** 3 区 × 5 个等权 replica，固定 10-height window，Commonware Simplex 的区域 Final 要求 4 个不同的合法 BLS signer。沿用 mock staking 与轻量执行，场景包含改变状态的交易。所有 replica 独立 Ready/Enter，正常时钟相对虚拟参考 `τ` 的误差 ≤ 1 ms，Clock RPC 延迟单独配置。输出以成功的受控 grant/Ack 和持久提交为边界，终止与签名记录跨崩溃保留。轮转场景使用持续运行的 worker，M3 注入旧区离线。完整 I1 恢复验收按原合同追踪。[PoC v2 配置](../../../formalization_v10/poc_design_zh_v2.md) · [Session Program](../../../formalization_v10/02_session_program_zh_v0_1.md) · [持续运行交接路线](../commander/restart-free-handoff-track-20261116.md)
- **执行依赖。** 每个集成场景使用上游已验收的确切提交。M3 的签名与 Plan 向量、M4 的 11→12 输入适配、回放的日志格式和界面可在各接口冻结后独立准备。主协调负责公共类型、依赖与集成。先冻结每项任务的触发、输入、时限、输出及验收向量，再由单一 writer 实现。Verifier 从规格独立导出预期，Tester 在同一已复核候选上复跑。写入限 `poc/`，沿用现有的重型构建与仿真调度。[依赖计划](../commander/dependency-plan-v2.md) · [角色配置](../commander/model-policy-20261116.md)

接线位置沿用现有模块。M1/M2 扩展场景 runner、`viewer-gfr`、`session-manager` 和 `l1-adapter`。M3 在 `poc/contracts/` 扩展授权与 policy 校验，区域选择由 `authorization` 提供，停滞与签名收集归治理服务。M4 接合 `batch-services` 与 `derivation`，在同一 GFR/Manager 实现上增加恢复事件。M5 扩展治理服务和确定性故障场景。场景、独立检查和回放分别放在 `poc/scenarios/`、`poc/verification/` 和 `poc/web/`。[现有组件与路径](README.md) · [Observer 交接](../handoffs/14-web-observer-zh.md)

## 2. M1 · 15 个健康节点的 Regular Smoke

| 工作项 | 实现 | 测试与通过条件 |
| --- | --- | --- |
| M1.1 节点与准入 | 复用受控 runtime、5 节点 Simplex、静态 schedule 和真实 `FinalVerifier`。每个 worker 独立持有签名身份、状态和时钟。 | 独立核对 15 个真实 PID 和 3 份不重合委员会。四个合法 signer 的证书通过，三票、重复 signer、跨 `cid` 或不匹配正文的证据被拒绝。节点错时进入，按同 `cid` 具备共同参与条件的 quorum 计活性。 |
| M1.2 区域结果到 `G` | 由真实 Final 和完整正文产生区域结果，经 Publisher、SimEspresso 公共日志和 GFR 驱动 Manager。用实际 anchor/state 算 descriptor 和 `cid`。 | 满窗 seed 42 路径的窗口依次是 `[1,10]`、`[11,20]`、`[21,30]`。h10、h20 的 G 接纳驱动两次 COMPLETE/Assign，h21 连到 h20 并进入 `G`。逐块核对证书、正文、父链和执行结果，记录旧实例 Close 及截止签名。 |
| M1.3 既有完整 M1 回归 | 用同一运行入口覆盖 SessionEnd、错时候选和公共 reader。保留实际区域结果的原始字节。 | h10 未在公开截止前完整出现时，SessionEnd 取 h9 为 anchor，后继窗口仍是 `[11,20]`，合格 h11 决定 padding10。测暂停/选回的原实例状态、proof 先到/body 后到、两个 reader 同位置一致。沿用完整 M1 的 seed 7/42/99 与墙钟暂停对照。官方 Espresso 本地节点另测同字节发布和读回。 |

本路线的 M1 交付是健康满窗 smoke。从现有 M1 分支的 `poc/scenarios/m1-healthy-regular/run.py smoke` 复跑，以实际 SHA、运行命令、15 个进程及 h10/h20/h21 证据为准。M2 从已核验的健康路径开始接线。SessionEnd、完整重放矩阵与官方读回按既有完整 M1 计划收口。[M1 smoke 交接](M1-healthy-smoke-handoff-zh.md) · [完整矩阵](M1-regular-merge-zh.md)

## 3. M2 · 从 L1 主动触发 Force

| 工作项 | 实现 | 测试与通过条件 |
| --- | --- | --- |
| M2.1 L1 提交与激活 | 沿用 M2 单签授权。经真实 Anvil 交易、手动出块、模拟 finality 与 reader 交付记录 `ForceRecorded`。以实际收录时间戳计算 `act = ts_L1 + M_a`。GFR 等待 Espresso 水位覆盖 `act`、完整 L1 控制日志覆盖 `act − M_a` 及所需验证材料。 | 对照 submit、include、finalized、reader 可读、GFR apply 五个时点。任一公共输入缺失时 GFR 在原位置 `WAIT`。同键按 L1 控制 → Expire → SessionEnd → Espresso 重放。错误签名、过期序号和冲突请求被拒绝，重复合法输入幂等。 |
| M2.2 新旧职责切换 | 激活时按实际 Tail 取 `G` 的 head 为 anchor，`β` 加一。旧 `NEXT` 已贡献 physical block 则 `r` 加一，否则沿用 `r`。`PENDING` 使用原 Boundary 的序号和来源区域。新窗口首高为 `max(A.h+1, slot(act+T_init)+1)`，末高为首高加 9。用实际新 `(β,r)` 求 Plan。 | 分别测已贡献/未贡献，签名后、激活前新增 G 结果，以及在途发生同分支 regular。检查 `BranchClosed` 覆盖 Running、Preparing 和 Paused，旧节点仍存活且在本地终止提交后停止新签名。迟到旧分支结果被 GFR 排除。目标 branch 已旧或 Tail 已 HALT 时记录无效控制。 |
| M2.3 从新 anchor 出块 | Manager 按实际 Plan 绑定 anchor/state，为目标区域 Prepare/Ready/Enter。按公开窗口和本地 `NotBefore` 产块。 | GFR 在正确位置激活时保留整个旧 `G`。声明的同 `cid` quorum 在准备预算内就绪，新分支首个合格结果通过发布取得 `GAccepted`。独立核对 anchor、窗口、状态和追加的连续执行结果，包括需要的 padding。回归 M1 两条 regular 路径。 |

验收合同预先固定控制传播、重放、状态准备及发布预算。正常 Espresso 时序下，`T_init` 须覆盖水位等待、重放、状态准备和时钟误差。健康正例以 `ts*(h_min) − δ_G` 为准备截止，核对从激活到本地创建/关闭的实际耗时，以及首个结果能否赶在下一待填高度的公开 Expire 前被接纳。[H4 交接](H4-healthy-force-zh.md) · [v10 Force 规则](../../../formalization_v10/09_finality_rules_zh_v0_1.md#44-force)

## 4. M3 · 治理多签与地区状态驱动的 L1 Plan

**本计划采用的 PoC 配置**是 3 名治理签名人、2 签通过，可在场景配置中修改。治理使用独立的 EIP-712/secp256k1 身份。现有 `Policy` 的 `enabledMask` 和循环顺序随多签 L1 控制记录公开。`enabledMask` 表示治理确认的地区资格，GFR 激活时将其固化为新分支 policy。恢复上线的地区通过后续授权 policy 重新获得资格。M1/M2 沿用静态 schedule 配置。[v10 权限与 Plan](../../../formalization_v10/01_l1_authority_zh_v0_1.md) · [v2 停滞策略](../../../formalization_v10/poc_design_zh_v2.md#83-何时-force公开结果停滞策略)

| 工作项 | 实现 | 测试与通过条件 |
| --- | --- | --- |
| M3.1 识别公开生产停滞 | 治理 signer 用各自已验证的 L1/Espresso/GFR 视图观察当前 `NEXT` 职责。从窗口开始与分配键中的较晚者起跟踪进度，已有贡献时使用最近的合格 `GAccepted` 键。完整读到的 Espresso 键与进度键之差达到 `T_stall`，且 finalized `q` 与水位正常推进时，复查目标 branch、Tail 和剩余预算。 | 用受控调度使当前区域的 5 个 worker 离线，保持 Espresso 正常出块。测阈值以下及恰好达到阈值。治理 reader 落后或未读齐已公开材料时 `WAIT`。未公开完整、无效、重复或未选中的结果不刷新进度。健康对照、Espresso 停滞或仅 `q` 增长的对照都不触发地区 Force。签名依据可从治理日志独立复算。 |
| M3.2 多签、L1 校验与去重 | 将 ControlAuthority 的授权检查扩为固定成员与门槛，沿用请求、序号和签名域，冻结签名包编码。收集对同一 payload 的不同成员签名，绑定请求、目标分支、policy 和配置。在途交易、确定失败和已处理控制各有持久状态。 | `k−1/k` 阈值、重复 signer、非成员、跨链/合约/控制类型重放和篡改 policy。少于阈值的请求被拒绝，达到阈值的有效请求真实落链一次。广播结果未知时恢复原请求，重复轮询及读者重放验证去重。 |
| M3.3 地区资格与历史 Plan | 扩展当前仅接收全开启静态 policy 的 L1 校验和 Plan provider，校验 mask 和地区循环排列。使用 `ℓ_s = max(Ref.os, L1 floor)` 对应的已验证历史。从 `sourceRegion` 下一项按公开循环选择第一个合格启用区，无来源时从首项开始，并冻结委员会、leader、窗口及授权来源。 | R0 停机后，Force policy 禁用 R0，来源为 R0 时按循环选 R1。对同一选择输入测试额外禁用 R1 时选 R2，恢复后下一次授权 policy 可重新选回。验证收录至激活前后的 policy 版本，以及历史 `cid` 的固定 Plan。缺 L1 历史返回 `WAIT`，无合格地区时进入 `PENDING`。读者 A/B 在同一位置核对生效 policy 和 Plan。 |
| M3.4 故障接棒 | 用自动生成的多签控制走 M2 的激活、Close 和新职责路径。 | 签名、提交与 L1 激活分别满足冻结的预算。GFR 拒绝旧区迟到结果，合格地区的首块在 Expire 前进入 `G`。从 trace 可按触发证据 → 签名 → L1 policy → Plan → 新 `GAccepted` 逐步追溯。 |

## 5. M4 · 模拟 D 并从 `D(h_res)` 继续生产

| 工作项 | 实现 | 测试与通过条件 |
| --- | --- | --- |
| M4.1 独立派生 | 复用功能 12 的回退、CursorSet 和切面，接入功能 11 已验的 frame/DA 输入。最小场景使用固定 `dep[h]`、轻量执行、实际 finalized Anvil 收据及确定性 DA。`D` 在自己的原生父状态上执行，输入为 L1、Inbox、DA 和配置。 | 从实际 D parent 派生至少一个及时接纳且含普通交易的批次。缺批次的高度在完整原生窗口结束后生成 `deposit-only`。已接纳但缺 DA 时 `WAIT`，补齐后在原位置推进，错 commitment 被拒绝。用同一高度的实际 native hash 验证预期分歧。 |
| M4.2 位置绑定的切面 | 用预设的合法请求经治理授权提交 RESUME，`ResumeRecorded` 与 Inbox 的 `CursorSet` 同交易原子写入，目标严格高于 Inbox cursor。D 先处理此前接纳输入，再按规则补齐，生成按 `(block, tx, log)` 索引的 `ResumeCut`，包含目标前缀、anchor、origin 和执行后状态。 | 同一 L1 block 内两条 RESUME 有各自历史切面。D 后续增高时重读切面哈希一致。目标已落在 D 的历史内时从指定高度取 anchor，并继续保留 D 的完整派生链。CursorSet 失败时整笔交易回滚。 |
| M4.3 GFR 对齐 | 等所需公共覆盖和对应切面，原子关闭旧分支并把 `G` 对齐到指定派生前缀。保留共同前缀，按切面替换旧 `G` 超出共同前缀的后缀。新 window 为 `[h_res+1,h_res+10]`，新 session 为 `(β+1,0)`，Plan 的 Ref 和 L1 floor 取自恢复 anchor。 | 缺切面或 DA 时，reader 保留当前事件前的重放状态。补齐后在同位置原子提交 `G` 更新和生命周期事件。核对首个不同高度、对齐后的 native hash/state root、派生出处和 A/B 同位置结果。目标过低时覆盖「完成对齐后 HALT」和 Expire 等于激活键的全序分支。 |
| M4.4 Session Manager 接续 | 冻结一次 RESUME 的事件批次。拟采用 Manager §3.10 的消费顺序 `BranchClosed → Resumed → SessionAssigned`，确保旧实例已终止、切面材料已验证并可用于新实例准备。 | 重复事件及迟到的旧 Prepare/Enter 无法复活已终止 `cid`。新 `cid` 按节点错时 Ready/Enter，首块 `h_res+1` 的 parent 连接至 `D(h_res)`，metadata 和执行根匹配实际派生状态，最终经真实 Final、Publisher、Espresso 进入 `G`。 |

M4 主场景让含普通交易的区块进入 `G`，让对应批次错过 L1 原生窗口，产生可核对的 D/G 分歧。固定一条合法 RESUME 及恢复输入，直接观察从冲突后缀替换到 `h_res+1` 进入 `G`。M5 复用相同交易和派生接口，由治理动态求目标并提交。[D 派生合同准备](../contracts/derivation-preparation-v1.md) · [RESUME 交接](../handoffs/13-resume-zh.md) · [GFR §4.6、§5](../../../formalization_v10/09_finality_rules_zh_v0_1.md#46-resume) · [Manager §3.10](../../../formalization_v10/03_session_manager_zh_v0_1.md#310-applylifecycleevent)

## 6. M5 · Espresso 停滞、自动 RESUME 与 Session Timer

| 工作项 | 实现 | 测试与通过条件 |
| --- | --- | --- |
| M5.1 识别排序服务停滞 | 分别配置 Espresso 停止 finalized 和 `q` 仍增长但 `θ` 水位冻结。L1 与 Clock RPC 继续运行，D 由 L1/DA 独立推进。治理记录停滞证据，GFR 在缺公共覆盖处 `WAIT`。 | 停滞期间旧实例可以在本地 stop 到期时 Close，全局定时事件及待激活控制仍须等公共水位。排序服务中断时治理保持观察。测试短停恢复但无恢复事由，以及水位持续不足直到观测截止的对照。 |
| M5.2 多签自动 RESUME | Espresso 恢复到可持续提供所需水位后，各 signer 从自己的公共视图确认 `HASH_DIVERGENCE`，或已公开 HALT 且 `D` 按规程越过冻结前缀。复查 L1、批次/DA、目标地区、未决控制和可行目标高度，再多签提交。 | 主场景让停滞跨过旧高度 Expire，L1/D 继续推进。水位追上后 GFR 重放 Expire，委员会确认 HALT 与 D 推进并自动提交。另测真实共同高度 hash 分歧、普通 D lag、D 较高但共同前缀一致、恢复输入未就绪和目标无解。 |
| M5.3 目标求解和定时恢复 | 以观察到的 G、D、Inbox cursor 和收录前最大推进量计算 `H_guard`，根据收录时间区间解整数高度，使用实际收录的 L1 时间戳算 `act`。记录 `act` 覆盖、对齐、每节点 Ready/Enter、首提议、首 Final 和首 `GAccepted`。 | 测收录区间两端、实际收录超预算、错时 Enter、时钟偏差 ±1 ms、RPC 延迟单独升高和已过 stop。正例按时对齐到该交易的 `ResumeCut`，正确窗口和 parent 从 `D(h_res)` 推导，恢复首块进入 `G`。晚收录按实际公共规则处理，并保留相应 WAIT/HALT/超时轨迹。 |

**如何判定「及时」。** v2 规程要求 Espresso 已恢复 live 才发自动 RESUME。停滞期间的验收是准确观测和保留重放位置。以 `t_eligible` 记录足够 signer 可独立验证恢复条件、必需输入齐备且目标可行的首个虚拟时刻。从此时起测轮询、签名和广播。`act` 是公开排序键，实际处理耗时由虚拟 tick 度量。[v2 §8.5–8.6](../../../formalization_v10/poc_design_zh_v2.md#85-何时-resume分歧或过期后的恢复) · [GFR 公共输入规则](../../../formalization_v10/09_finality_rules_zh_v0_1.md#34-可重放的输入)

- **提交。** 断言 `t_submit − t_eligible ≤ T_poll + T_sign + T_broadcast`，并单独核对 L1 实际收录落在预计区间。选最小合法整数 `h > H_guard`，满足 `t_L,max + T_prep ≤ ts*(h) ≤ t_L,min + T_drift − ε`。无整数解时报告等待原因，到输入更新后重算。
- **对齐和准备。** 检查 `T_prep ≥ M_a` 与 `M_a + T_ready + δ_G ≤ T_prep + b_L2`。`T_ready` 覆盖公共日志、水位、派生/DA、GFR 重放和状态准备。正例核对 `G` 对齐及声明参与进展的至少四个同 `cid` 副本完成准备，截止时刻为 `ts*(h_res+1) − δ_G`。
- **开窗、停止和接纳。** `NotBefore` 由每个节点的本地协议时钟检查，`C_m ≥ ts*(h_max+1)` 后停止为该 `cid` 生成新签名。活性从同一 `cid` 的共同参与区间计时。用场景固定的链路、执行、发布和 GFR 预算核对首个区块的适用公开截止。预算以 v2 §12 的 `M_a=6 s`、`T_prep=10 s`、`T_ready=3 s`、`T_drift=15 s` 等为初值，结合实际组件配置核验后随场景冻结。[v2 参数与成功终点](../../../formalization_v10/poc_design_zh_v2.md#12-参数场景与验收)

## 7. 从确定性 benchmark 日志回放跨区轮转

参照 Regional BFT 的生成、检查和打包流程，交付可离线打开的只读 HTML 回放。M1 先提供两次 regular 接棒。M2/M3 增加 L1 控制和多签证据，M4/M5 加入 D、G 后缀替换与停滞恢复。页面直接消费运行产物，支持播放、暂停、单步、拖动时间轴及按区域、`cid`、height 和控制交易定位。[Regional BFT 回放](../../../regional_BFT_sim/visualization/README.md) · [Web Observer 交接](../handoffs/14-web-observer-zh.md)

| 工作项 | 交付与验收 |
| --- | --- |
| 原始日志与投影 | 版本化的 manifest、原始 trace 和浏览器投影。先独立检查运行证据，再从原始 trace 生成回放数据。用单调 `seq` 保留同一虚拟 tick 内的事件顺序，保留因果引用、节点/区域/`cid`/height、公共 replay position、Espresso `(q,θ)`、L1 `(block,tx,log)` 和原始证据定位。`submit`、`included`、`finalized`、`applied` 和 `GAccepted` 各有可定位的事件。 |
| 节点与链时间线 | 三区各展示五个 replica 的在线状态、Ready/Enter、view、leader、Close。显示窗口、旧/新 anchor 和 R/G/G′/D 的 head 与 hash，标注 Regional Final、公开完整、G 接纳、Inbox 接纳与 D 派生。并列显示虚拟时间、公共键和节点时钟偏移，展示 Espresso 水位缺口及治理的观察位置。 |
| 操作与解释 | 点击 handoff 能看到为什么发生、关闭了哪些 `cid`、新区域何时准备好、首块何时进入 `G`。点击 RESUME 能查看切面位置、分歧高度、替换区间及实际 parent。点击 `WAIT` 能看缺失的水位或数据。A/B 可切到同一公共位置核对。阶段耗时按虚拟 tick 计算，并显示场景的时限与适用前提。 |
| 确定性与验收 | 用同 SHA、seed 和配置复跑，注入不同墙钟暂停并比较虚拟 trace。对 PID、运行目录和墙钟诊断使用预先声明的规范化规则并保留原始文件。比较回放关键帧与独立检查结果，逐步和跳转落到同一事件得到同一画面状态。每个里程碑保存成功轨迹，另收录停滞、缺 DA 或迟到控制的等待/拒绝轨迹。 |

## 8. 每次验收的证据

1. **复现输入**包括候选 SHA、源码和工具/二进制版本、BFT gitlink、seed、genesis/registry/policy、交易、故障脚本、时钟/网络配置、截止预算及实际运行命令。每次用新输出目录。
2. **原始证据**包括进程与 grant/Ack/journal、区域证书和完整正文、真实 L1 交易和收据、Espresso 前缀、治理签名与触发证据、DA 内容、D 切面及 GFR/Manager 事件。每个里程碑提供一条运行命令、一条离线检查命令及一个回放入口。
3. **独立检查**从上述输入重算签名门槛、Plan 和窗口、公开顺序、anchor/链执行及生命周期。A/B 同位置一致性和独立规则检查都入报告。逐条记录预期、实测、适用前提及 `PASS/FAIL/INCONCLUSIVE`。缺必要输入的负例通过条件是出现预期 `WAIT` 等行为。时限测试在前提满足而超时时判失败，前提不足时记录可观察范围。[v2 证据包与时限说明](../../../formalization_v10/poc_design_zh_v2.md#123-三类成功终点与报告)
