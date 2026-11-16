# 功能 03：Mock Plan、公共对象与轻量执行准备合同 v1

日期：2026-11-15。状态：**PREPARATION-FROZEN（由功能03主协调审阅确认）；IMPLEMENTATION-BLOCKED；waiting-dependency=01**。

本版冻结来源已确定的语义、准备阶段所有权和独立验收门槛；不冻结仍需跨功能决定的字节格式、密码学接线或 ABI。`S` 表示来源规则／已经授权的 P0 mock，`P` 表示可审查候选，`D` 表示总协调须决议的接口。只有总协调给出 01 的准确已验收 SHA、02/03 共享接口决议，并授权实现后，才可修订为 Builder 使用的完整实现合同。本阶段不能把本文件、黄金向量自检或旧 fixture 的通过当成功能 03 完成。

## 1. 基线、来源和所有权

准备基线为 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，其代码基线为 `36eff1c76d5c4df5d0790133c0e14719c0c73ebd`。实际 worktree 为 `/home/jy/.codex/worktrees/1c43/megaroation_poc`，分支为 `codex/poc-mock-plan-execution`。没有把准备基线视为 01 已验收结果。

| 标识 | 权威与适用范围 |
| --- | --- |
| S-V2 | [PoC v2](../../../formalization_v10/poc_design_zh_v2.md) §§3、6.3、9、10.2、10.3；SHA-256 `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`。3×5 等权、静态排班、10-height 窗口、配置绑定 dep 与轻量执行是显式 mock。 |
| S-A | [v10 架构](../../../formalization_v10/00_arch_zh_v0_1.md) §§1.1–1.3、2.2、3.1–3.4：hash 身份、四类时间／位置的区别、Physical/Virtual/Anchor、Apply/Kept、VP1–VP3、Merge 和 origin。 |
| S-AUTH | [v10 Authority](../../../formalization_v10/01_l1_authority_zh_v0_1.md) §§3.2、3.12–3.14：BlockRef、Window、规则配置和 Plan 验证边界；P0 替换 stake/snapshot 选择，不声称实现这些机制。 |
| S-SP | [v10 Session Program](../../../formalization_v10/02_session_program_zh_v0_1.md) §§3.2、3.4–3.10、3.12、3.14：ProgramConfig/cid、来源材料、Build/Accept/CheckFINAL 与结果；03 提供纯对象和执行，04 实现准入和原生验签。 |
| S-GFR | [v10 GFR](../../../formalization_v10/09_finality_rules_zh_v0_1.md) §§3.2–3.4、4.3–4.7、5：Boundary/ReplayPosition/Lifecycle 的来源、公开顺序、独立 D 和精确恢复位置。 |
| S-CW | Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` 的 `DESIGN.md` §§1–3.1；Commonware **2026.10.0** 的 `utils/src/faults.rs`、`utils/src/ordered.rs`、`consensus/src/simplex/{types,scheme}.rs` 与 multisig 实现。使用 Simplex `N3f1`；不采用该参考仓库的 15 人 benchmark quorum 或 coded profile。 |
| P-REG | [regular 准备稿](regular-preparation-v1.md)：已经接受方向但仍待冻结的 identity、receipt、fixture 参数和 native 接线；不是已经通过的实现合同。 |
| D-H | [03 交接](../handoffs/03-mock-plan-execution-zh.md)、[共用约定](../handoffs/workflow-zh.md)与[总协调规则](../commander/README.md)：依赖门禁、单写者和交付阶段。 |

只读核对了源 checkout 已初始化的 `/mnt/reth/jy/megaroation_poc/regional_BFT_sim`，其 HEAD 等于上述 pin；未初始化、改写或提交本 worktree 的子模块。Commonware 本地缓存按版本查阅；锁定版本是权威，不跟随 `latest`。现有 `native_elector.rs` 只用于确认当前接线边界，不作为黄金期望来源。

| 路径／责任 | 本阶段唯一所有者 | 写权限 |
| --- | --- | --- |
| 本合同、`poc/docs/mock-plan-execution-preparation.md` | Docer `bft_spec` | 准备文档 |
| `poc/verification/mock-plan-execution/` | 独立 Verifier `bft_verifier` | 来源推导、固定输入、候选编码向量与短 checker |
| Git 分支、准备提交、合同最终核对 | 功能 03 主协调 | 只提交获分配准备产物 |
| `poc/crates/types/`、workspace/lockfile、实现源码、proto | 本阶段无 writer | 禁止修改 |
| `poc/docs/commander/`、源 checkout、Regional BFT 子模块 | 总协调／原所有者 | 本任务只读 |

后续建议由单 Builder 负责 `poc/crates/authorization/`、`poc/crates/core/src/{execution,metadata,merge}/` 和获分配 fixture 路径；公共 types、模块声明及 workspace/lockfile 由总协调明确指定一个 owner 后才能改。此建议不是当前写授权。

## 2. Registry、排班与 Plan

### 2.1 已确定语义 S-REG

Registry 恰有 15 个身份：`region i ∈ {0,1,2}`、`member j ∈ {0,1,2,3,4}`、`pid=5i+j`。每个 pid 唯一绑定身份公钥、独立 BLS vote 公钥和 `weight=1`；每地区是独立的 5 人委员会，`f=1`，最终证明要求至少 4 个不同合法 signer。离线、重启、早 Enter 都不修改 registry、权重或 leader 序列。公开配置绑定完整 registry、genesis、schedule 规则及版本、执行/dep fixture 引用和编码版本。

注册前验证每个 BLS vote key 的 PoP；重复 pid、重复身份键、重复投票键、缺项、错误 region/member、非等权或非 3×5 表均不得形成有效 registry。公钥必须用选定算法的规范解码与合法点检查，不能把任意同长度字节当合法键。PoP 算法、签名域、身份 key 对 vote key 的绑定证明及测试公钥表的具体字节属于 D-REG，不能由模拟 `pop_valid=true` 替代最终验证。

必须保存三套可逆映射：

```text
pid <-> (region, member)
(region, member) <-> participant identity key
(region, member) <-> Commonware Participant signer_index
```

Commonware signer index 按该委员会实际 `Scheme::participants()` 的排序身份公钥排列，不按 pid、不按 key 输入顺序，也不能未经验证改按 BLS vote key 排序。各 index 同时索引被绑定的 vote key；公钥排序无相等项。02 的 endpoint/transport 身份是另外一层认证；路由送达某 pid 不能证明该 payload 的 signer index、leader 或签名正确。

### 2.2 排班与输入域 S-SCHED

对非负 `beta,r` 和正 `view`：

```text
region = ((beta mod 3) + (r mod 3)) mod 3
member = (view - 1) mod 5
pid = 5 * region + member
```

上式先约简仅为避免有限整数加法溢出的等价计算，不改变数学规则。`view=0`、负数、超出最终编码类型的值均返回具名输入错误，不能 panic、截断或 wrap。计数器增加、窗口端点、timestamp 与 byte-limit 算术须检查上溢；“输入超出类型范围”和“两个合法输入相加超过类型范围但可用等价式算出 schedule”须分别测试。整数宽度与错误 discriminant 属 D-CODEC。

固定例子为 `(0,0,1)→(region=0,pid=0)`、`(0,0,2)→(0,1)`、`(0,1,1)→(1,5)`；`view=6` 回到该地区 member 0。Force 用激活时实际产生的新 `(beta,r)`，不预猜 r；只允许 `static-schedule-v1`，不允许控制记录任意改 leader。

### 2.3 Plan 和 Boundary 的语义骨架 S-PLAN

以下是字段职责，不是 Rust 定义或 canonical wire order：

```text
SessionId { beta, r }
Window { h_min, h_max }  // inclusive, h_max - h_min + 1 = 10
BlockRef { h, os, block_id }
RuleSpec { rule_id, version, params }
Boundary { cause, target_session, source_region?, reference_block?, anchor?, window? }
PlanRecord {
  session, region, committee, eligible_proposers, leader_rule, window,
  reference_block, config_ref, authorization_ref
}
AuthorizationView { committee, total_weight=5, fault_bound=1,
                    eligible_proposers, leader_rule, window, config_ref }
```

`PlanRecord` 为 v2 最小视图；其来源边界绑定、`authorization_ref` 内容、是否保留 v10 的 source_region/policy 等冗余字段须由 D-PLAN 决议。未实现 snapshot/staking 的字段不能伪造“已验证链上快照”。`Boundary` 可在准备阶段含未知分量；成功 Plan 所需选择输入、配置和来源必须完整。预计算 regular Plan 不应要求最终生产 anchor 已就绪。

`resolve(boundary, verified_public_inputs)` 返回：完整且可重算的 `PlanRecord`、缺配置/来源时 `WAIT(missing)`、明确规则下无合格安排时 `NONE`，或坏输入 `REJECT(reason)`。默认固定完整 registry 不因成员离线返回 NONE。尚未支持的 policy/rule version 不能被解释为 NONE 或静态默认值。验证收到的 Plan 时从公开来源重算、逐字段核对；“Plan 有效”“某 descriptor 已被 GFR 选中”是不同判断。

窗口为公共绝对区间：`h0 < h_min ≤ h_max` 且宽 10。创建 descriptor 时另查 `anchor.h < h_min`；不能把 `anchor.h+1..anchor.h+10` 当成所有后继的窗口。例如前驱在 h9 结束，后继仍为 `[11,20]`，h11 的后继可覆盖 padding h10。

`reference_block` 与生产 anchor 分开保存：regular 选区参考是**前驱 session 的初始 anchor**，新 descriptor 的生产 anchor 是 GFR 选出的末端；Force/RESUME 的参考取新分支实际 anchor。两个同高度、不同 `block_id` 的 anchor 不可相连或共享 cid。

## 3. 公共对象、身份与证书边界

### 3.1 最小对象 S-OBJ

| 对象 | 最小语义／不变量 | 负责扩展的下游 |
| --- | --- | --- |
| `ProgramConfig` | `chain, rule_version, verifier_version, config_ref, committee, fault_bound, eligible_proposers, leader_rule, window, byte_limit, metadata_window`；授权项逐项等于 Plan 的 AuthorizationView。 | 04 接 Simplex；03 不擅增改变 cid 的运行字段。 |
| `AnchorIdentity`／`Anchor` | 固定连接 `id`、height、实际 timestamp/origin；精确执行后状态由实际前缀验证。物理来源 `id=H(B)`；派生来源 `id=H(DERIVED,height,native_hash)`；genesis id 来自配置。 | 04 入口状态；12/13 派生 anchor。 |
| `Descriptor` | `{plan, anchor, cid}`，cid 重算通过后不可变；anchor 执行状态可晚于 Create 齐备，Enter 前必须验证。 | 04/05 生命周期，07 选择。 |
| `AnchorEvidence` | `{kind, source_ref, source_objects}`；GENESIS 指 config，REGIONAL 指前驱结果，FORCE 指公开 replay position，RESUME 指精确 L1 position；内容须指向同一 anchor。 | 04/06 验证来源；09/12/13 控制及 ResumeCut。 |
| `PhysicalBlock` | `(ph,h,ts,os,root,Txs)`；ph 为实际前一 physical 或 anchor 身份，root 为当前 Begin/DEP 前且 padding 后的状态，Txs 是原始普通交易。没有 view 或 DEP 字段。 | 04 用其 hash 作 Simplex payload；06 发布；07 Merge。 |
| `NativeProposalBinding` | `(cid,epoch,view,native_parent_view,payload_digest)` 与已验证 `PhysicalBlock.ph`、实际 parent 对照；view 与 height 不同。 | 04 独占真实 native subject 接线。 |
| `RegionalResult` | `{descriptor, block, final_proof, chain, anchor_evidence}`；chain 从同一 anchor 到 block 完整 hash 相连，末端严格相同。 | 06 完整发布；07 公开接纳。 |
| `ReplayPosition` | `{key,kind,index}`；kind 为 FORCE/RESUME/EXPIRE/SESSION_END/ESPRESSO，index 分别为精确 L1 position/height/session/Espresso height。 | 07 canonical 编码与比较；09/12 位置映射。 |
| `LifecycleEvent` | `{position,kind,body}`；Assigned{descriptor}、SessionClosed{session,reason}、BranchClosed{branch,reason}、Resumed{control_seq,branch,anchor,derived_prefix}。同 position 可有多个不同事件。 | 05/07 事件身份与去重；09/12/13 扩充具体控制体。 |

这里只复述 v10 的 ReplayPosition 语义顺序：先 key；同 key 时 L1 控制、EXPIRE、SESSION_END、ESPRESSO；Force 与 RESUME 混合按 `(block,tx,log)` 排序，不各自按 enum tag 排序。03 不在本阶段决定其字节、排序 key 存储表示、CloseReason discriminant、ResumeCut 或 ABI。v2 §4.1 已固定协议／仿真时间使用整数微秒，转换到 EVM 的 L1 timestamp 使用整数秒并显式检查取整边界；不得将这些单位作为待选方案。尚待 D-CODEC/D-L1 决定的是类型宽度、规范 wire representation、转换溢出及取整规则的具体编码。公开 key、虚拟参考时间、节点协议时钟、reader delivery 时间不得因整数表示相同而互换。

### 3.2 Context 和 hash 分层 S-ID

```text
cid = H(REGIONAL, session, id(anchor), ProgramConfig)
physical_digest = H(canonical PhysicalBlock)
native_hash[h] = H(actual native parent hash, actual metadata,
                   actual DEP, actual Kept, execution-after root)
derived_anchor_id = H(DERIVED, h, native_hash[h])
```

上述 H/括号是规范规定的语义 preimage，不是已经选定的拼接编码。每种域必须有一种版本化、无歧义编码，并有固定字节向量后才可实现。cid 不包含 mutable execution cache、reader ID、网络 incarnation、接收时间或墙钟；anchor 状态不作为 descriptor 身份字段。对 anchor 的 metadata/state 仍须根据其固定来源核对，不能以“不进 cid”为由接受替换。

Physical hash 承诺原始 Txs，即使一些交易最终被 Apply 跳过；native hash 承诺 Kept，且其 parent 是实际连续 native parent，不能使用 physical parent 或仅高度。相同 root/height 不足以判 G/D 一致；不同原始 Txs 经相同执行过滤后可能得到相同 native block。native hash 不包含 pid、view、cid、proof 或本地执行时间；这些不属于原生块实际内容，若未来拟加入须单列协议变更。

### 3.3 公共验签接口 S-FINAL

最小逻辑接口为 `FinalVerifier.check(descriptor, block, proof) -> bool`，可附加不改变真值的结构化诊断。03 冻结调用语义及向量，04 提供 Commonware 原生验签实现，不可在生产入口放 always-true verifier。

1. 用历史配置和 Plan 重建 ProgramConfig，核对 cid、物理 payload hash 和固定 verifier version。
2. 根据原生 proof 解码真实 subject：Commonware Proposal 含 round(epoch/view)、parent view、payload digest；物理 parent digest 通过已验证 payload 的 `ph` 绑定，不能虚构原生 Proposal 另有 parent-digest 字段。
3. chain/config/cid 由选定原生 namespace 加 native subject 全面绑定；kind 的独立 namespace/subject 不可互用。仅 storage partition 或网络 channel 名称不提供签名绑定。
4. 使用 5 人 registry 的历史身份↔投票键映射，验证规范 signer 集合/bitmap、合法范围、不同 signer 至少 4、实际 aggregate signature 与实际 final kind。3 个或重复凑 4 个均不够，notarize/nullify 不冒充 final。
5. 检查历史 FinalCert 不读取当前 observer clock、lease 或当前 Enter 状态；祖先完整性、Plan 公共来源、来源 anchor 与 G 接纳资格由相应消费者另查。证明先到正文后到时，proof observation 与 verified contiguous delivery 分别记录。

本阶段只提供结构/绑定负例；没有真实 PoP/BLS 执行，相关密码学结论记 INCONCLUSIVE。03 后续与 04 须补充同一真实 certificate 在错误 cid/config/chain/epoch/view/parent-view/kind/body 下的拒绝；另测正确证书但缺祖先只能取得证明观察、不能交付前缀。

## 4. Metadata、DEP、Apply 与 Merge

### 4.1 Metadata 和依赖 S-META

`ts*(h)=genesis_ts+(h-h0)*b_L2`。生产 origin 使用 v10 架构 §3.4：`sch(h)=max{ell: ts_L1(ell)+T_dep≤ts*(h)}`；`omega_A(h)=min(A.os+h-A.h,max(A.os,sch(h)))`，且在 A.h 取 A.os。参数/anchor 条件及 O1 连续、O2 不超前、O3 drift 均须验证；空 sch 集合和坏参数不得默认为 origin 0。

P0 的强制前缀由**本次实际 metadata/origin 对应的 attributes**与配置绑定的 `dep[h]` 构成；system/upgrade 的受限配置须显式写为不支持或固定空，不能静默丢弃真实来源的必需内容。`dep[h]` 按 height 给定是 v2 mock，不据此宣称真实 L1 deposit 按 origin 分配验证通过。缺 h 的 fixture 行与显式空列表是两回事。

`MetadataProvider`／`DepProvider` 最小输入为固定 config、具体 anchor/生产或派生模式、height、逐读者已验证 L1 视图；输出完整 metadata/DEP 或具名 WAIT/REJECT。G 使用生产 origin，D 使用 12 决定的实际派生 origin，两者共享 Apply/编码而不能共享“从 G 拷贝 origin”捷径。

### 4.2 轻量执行 S-EXEC 与候选 P-EXEC

S-EXEC 冻结：`Apply(state,meta,DEP,Txs)` 先 Begin 写入当前 block context、按顺序执行强制前缀，再逐笔检查普通交易的 Valid。格式正确但 nonce/余额不满足的普通交易跳过，不改变状态，也不进 Kept；通过 Valid 的交易执行并按原顺序进入 Kept，Kept 不含 DEP。非法编码在 BasicTx 阶段拒绝该 physical payload，不能当 invalid 交易跳过。若未来加入 revert，执行过但 revert 的交易仍在 Kept；本阶段无需实现 revert。

P-EXEC 建议只提供 Credit/Transfer/Attr 三种操作及小型账户表；但地址长度、nonce/余额整数宽度、未知账户、零金额、自转账、余额/nonce 上溢、强制 Credit 错误的终止类别、state-root 是否编码所有零账户等具体选择均为 D-EXEC。独立向量可以选一个具名实验 profile 演示完整行为，不能把这些示例悄悄提升为 S-EXEC。

执行 API 必须可纯函数重算。相同已验证 parent 状态和输入反复求值应给同一返回结果，不代表再次提交 DEP；缓存／提交身份必须绑定实际 parent、配置/执行版本、height 和输入 commitment，不能按 height 命中另一分支状态。atomic commit 与持久性如何接 01/02 由实现合同明确，纯对象计算不得绕过 controlled grant/Ack 发布协议输出。

### 4.3 Padding 和输出 S-MERGE

给定已验证 anchor 和完整 physical 链 `A,B1,…,Bn`，必须有严格增高的 h、正确 ph，以及每个 physical h 在固定 window 内。对连续高度从 `A.h+1` 到 `Bn.h` 执行：physical height 取其原 Txs，间隙取空 Txs；每一高度都计算实际 metadata/DEP、记录 Apply 前 root，再执行 Apply 得 post-state/Kept。physical root 应等于已执行所有前置 padding 后、本 height Begin/DEP 前的 root。

Merge 只展开被后继 physical 决定覆盖的区间，不因本地时钟、窗口末端或 SessionEnd 自行延长 G。无新 physical 的链只返回 anchor 处的已有前缀。缺 parent、祖先正文、metadata 或 DEP 时 WAIT，已知错误 parent/root/metadata/格式则 REJECT；不能把缺失数据替换成空块或空 DEP。

最小执行输出应区分 `virtual_block`（pre-root、原 Txs）、`kept`、`post_state/root`、`native_parent/hash` 和覆盖该 padding 的 physical 结果引用。返回这些对象不等于 Final、G 接纳、Inbox 接纳或 D 派生；下游分别产生这些事件。

## 5. 受限 finalized L1 fixture 和逐读者 gateway

S-L1：fixture 在公开 manifest 中固定 genesis、各 block 的 hash/parent、height/timestamp、finalization 条件、完整控制记录及来源 commitment。每个 reader 拥有独立 delivery/cursor；可见视图只含该 reader 实际收到且验证通过的**连续 finalized 前缀**。磁盘存在未来行不构成可见性；reader A 读到的前缀不能自动提供给 B。

语义骨架为 `L1BlockRef{height,hash}`、`L1Position{block,tx,log}`、`FinalizedPrefix{tip,control_coverage,provenance}` 与 `read/require(reader, needed)`。具体接口与 02 service envelope、09 Anvil reader 的相容定义属 D-L1。metadata 读取能力和完整控制覆盖能力分别表达；空 control 列表必须有完整性依据，不能由“查询没有返回日志”推出。

| 触发／前态 | guard | 必须结果 |
| --- | --- | --- |
| 查询已交付且完整验证的连续 finalized 行 | hash/parent/source/覆盖一致 | 返回具体 immutable 行和来源；相同前缀两读者的语义值相同。 |
| 查询范围内未来行或已 finalized 但未交付的行 | 当前 reader 前缀不够 | WAIT，不能越过 reader gateway 读底层文件。 |
| 收到更高行，中间行未交付 | 连续性未满足 | 暂存但不推进 visible prefix；后补中间行再验证推进。 |
| block metadata 已知但控制完整覆盖缺失 | GFR 请求控制覆盖 | WAIT(ControlCoverage)，不等价于零控制事件。 |
| 公开 manifest 承诺范围内行永久缺失或 source/hash/parent 损坏 | 确认 fixture 不完整／冲突 | 显式 FixtureError/REJECT；不伪装正常无限等待或空输入。 |
| 查询超过有界 manifest 上界 | 超出本试验声明范围 | FixtureExhausted；不可报告生产活性 FAIL 或自动延伸来源。 |
| 同一已验证位置重传相同内容／冲突内容 | immutable prefix | 前者幂等，后者拒绝并保留原前缀。 |

P-L1 的可审查 REG-A profile 来自 regular 准备稿：L1 heights 0–30、`ts_L1(ell)=2ell s`、finalized at timestamp+1.5s、通常 timestamp+2s 连续交付；L2 genesis h0=0/ts=10s、b_L2=1s、genesis origin=3、T_dep=5s、T_drift=15s。这些数值与具体 provenance 字节尚待 D-L1；本阶段不运行 15-worker 或 Anvil，也不声称此 fixture 提供真实 L1 finality 安全性。

## 6. 独立验收矩阵

Verifier 从以上来源先推导期望，再检查本准备合同；禁止调用候选实现生成 expected。独立短脚本只能核对固定输入、算术、编码与执行候选。以下测试族是后续完整实现必须覆盖的最小集合；真实密码学、进程和集成场景当前均未执行。

| 编号 | 触发／前态与边界 | 期望与证据 |
| --- | --- | --- |
| M01 | 3×5 registry 完整；打乱输入公钥顺序；三地区相同 member | 同一语义表；pid/member/signer 双向表可核对，leader 按 member 而非排序 index 轮转。 |
| M02 | 重复身份/投票 key/pid、错 PoP、缺成员、坏权重、错误 region | 拒绝配置；保留具体解析/PoP失败证据，不创建有效 committee。 |
| M03 | 小值、跨 region/r/branch、view1/5/6、view0、表示上界 | 合法例按公式；坏域拒绝；合法大输入不因 beta+r overflow 得错误 region。 |
| M04 | 固定 10-height window、h9 后 successor11–20、坏宽度或 overflow | 参考 block/实际 anchor 分离；坏 window 拒绝，候选换 anchor 不重发 10-height。 |
| M05 | 相同高度不同 anchor hash，改 chain/config/session/ProgramConfig | cid 或有效性必须变化；仅改变 cache/reader/current clock 不改变 cid。 |
| M06 | 真实 3/4 signer、重复 signer、跨区 signer、错 kind/cid/view/epoch/body/parent-view | 公共 verifier 按实际 subject/domain 拒绝负例；正确 final proof 和有效 ancestry 的判断分别记录。 |
| M07 | 固定原交易：成功转账、nonce错、余额不足、依前笔状态才有效 | 按顺序得到准确账户、Kept 和 post-root；BasicTx 格式坏则整个 payload 拒绝。 |
| M08 | DEP 会改变余额；h gap 中 padding 也有 DEP；metadata context 改变 | 每高度一次 Begin/DEP；当前 physical pre-root 在其 Begin 前，post-root/native hash 在其 Apply 后。 |
| M09 | 相同 height/root、不同 native parent 或 metadata/DEP/Kept | 比较 hash 绑定实际输入；raw invalid Txs 改变 physical hash但允许 native hash不变。 |
| M10 | ph错、重复／下降height、missing ancestor/metadata/dep、未覆盖尾部 | 错内容拒绝；缺输入 WAIT；不凭时钟创建 padding；重复纯求值相同且不重复提交 DEP。 |
| M11 | 两读者不同 delivery，future/on-disk、out-of-order、缺control coverage | 不越过各自 finalized prefix；同一公开前缀最终值相同，WAIT不修改原前缀。 |
| M12 | manifest 内缺行、范围外、坏hash/parent/source、冲突重传 | 明确区分 fixture错误、exhausted、无效输入和正常暂缺；禁止 default empty。 |
| M13 | 同 key Force/RESUME 混合位置、Expire/SessionEnd/Espresso、同position多lifecycle | 遵 v10语义顺序，typed index不混用；实际canonical replay codec交07，03不抢定。 |

准备向量的候选编码 profile 为 **`MPX-PREP-LE64-SHA256-v0`**，仅在 [独立候选编码说明](../../verification/mock-plan-execution/proposal-codec-v0.md) 定义并核对；[来源期望](../../verification/mock-plan-execution/source-expectations.md)、[语义输入](../../verification/mock-plan-execution/semantic-vectors.json) 与[黄金示例](../../verification/mock-plan-execution/golden-vectors.json)分别保留。其 LE64/SHA-256、长度前缀、域 tag、账户与交易格式均为 P-CODEC/P-EXEC，不能作为批准的 Commonware subject 或 Solidity ABI。来源确定的语义测试与采用此 profile 的黄金字节／摘要必须分别标注；真实 key/PoP/certificate、production L1 gateway 缺执行时不得写 PASS。

## 7. 跨功能接口决议清单

| 决议 | 涉及功能 | 主协调须给出的具体结论；未决期间边界 |
| --- | --- | --- |
| D-BASE | 01、02、03 | 01 独立验收和 same-SHA 测试结果、可继承 SHA、恢复/root-step/Ack 不变量；本阶段不改实现。 |
| D-REG | 02、03、04 | 身份/vote key 算法、公钥/PoP fixture、PoP namespace、registry hash、排序 index 映射交换格式、endpoint认证与 signer责任；合成key不算真实验证。 |
| D-PLAN | 03、05、07、09、10/13 | v2最小 Plan如何绑定 Boundary/config/authorization_ref，哪些v10字段保留及unsupported policy结果；reference与anchor保持分离。 |
| D-CODEC | 03、04、06、07、09、12 | 唯一规范整数/长度/tag/hash/domain/版本/limit，ProgramConfig和cid实际preimage，Physical/native/derived身份；黄金实验profile仅供评审。 |
| D-CERT | 03、04、06、07 | namespace到chain/config/cid绑定，epoch/profile、native subject、proof封装和真实3/4证书；04写实际验签，03只提供公共边界。 |
| D-ROUTE | 02、03、04、06 | pid/service/reader ID、run/config/cid、incarnation、logical message、受控输入引用、payload limits；transport源与certificate signer分开。 |
| D-EXEC | 03、04、07、12 | 账户/交易/state编码、所有算术和边界、DEP格式/commitment、缓存/提交身份、native genesis parent/hash及错误分类；必须补固定执行向量再授权Builder。 |
| D-L1 | 02、03、07、09、12 | 逐reader immutable finalized prefix、metadata/control coverage、source commitment、已定μs/EVM整数秒语义的wire宽度/转换溢出/取整编码及fixture错误类别；09真实receipt adapter接相同语义。 |
| D-REPLAY | 03、05、07、09、12/13 | ReplayPosition/Lifecycle完整typed编码及事件去重；07负责排序实现，09负责实际tx/log位置，12提供位置绑定ResumeCut。 |
| D-ABI | 09、11、12/13 | Control/Inbox/Cursor/DA/attestation签名域与ABI及历史切面；03仅预留typed引用，不定义假ResumeCut或“block最终D”。 |

## 8. 阶段门禁和完成定义

本阶段交付：合同与独立推导、版本化固定输入/黄金示例及轻量自检、主协调保存的准备提交。只可报告准备检查 PASS/FAIL；不具备真实实现证据的项为 INCONCLUSIVE。没有 Builder、Tester、Rust build、重型仿真，未变更公共 types、workspace/lockfile 或任何实现代码。

进入实现前须把 D 项的批准值合并到修订合同，标明决议来源和影响消费者，冻结一份可执行编码/参数表及独立 expected 的 SHA。后续一个 Builder 负责源码；候选先提交命名 SHA，Verifier 对该 SHA 独立核对，Tester 只对同一已验证 SHA 跑约定矩阵。所有 Rust build/重型测试使用 `flock /tmp/megaroation-poc-heavy.lock`、`CARGO_BUILD_JOBS=2`，独立 target/artifacts；实现改变后重验受影响范围。

总协调报告根分区仅余约11GB并已分配任务专属数据盘目录 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03`。首次Rust构建前须检查当前 `.build`，保留和安全迁移已有内容，再将本worktree `.build` 链接到该专属目录；大artifact也使用本任务专属目录。不得共用其他job target、覆盖既有结果或在本准备阶段提前操作链接。轻量verification材料留在worktree。此资源安排不改变全局锁、恢复约束或未授权实现的门禁。

功能03完整完成仍需真实registry/PoP、canonical编码/公共验签接线边界、轻量执行/Merge和L1 gateway实现及相同候选的测试证据；状态保持 `waiting-dependency=01`。
