# H2 Session Manager healthy preparation v1

日期：2026-11-16，Asia/Shanghai。合同 ID：`H2-SM-HEALTHY-v1-draft2`。状态：**DRAFT / NOT FROZEN / NOT IMPLEMENTATION AUTHORIZATION**。本轮仅产出合同与独立验收准备；没有实现、构建、测试、模拟或验收 PASS。

准备基线为 `4a9a6b0a92a11e5778833988bb6524d929f4608e`，工作目录为 `/home/jy/.codex/worktrees/a6cf/megaroation_poc`。这只是计划文档基线，**不是 M0 集成交付 SHA**。M0 准确验收 SHA、H1/H3 冻结接口提交和本合同独立复核齐备后，lead 才可收口合同；启动 Builder 还须总协调的后续实现授权。本轮准备派发不因依赖到齐自动变为实现授权。未知 SHA 保持未填写，不以浮动 HEAD 代替。

本稿使用 [regional-bft-conformance](../../../.agents/skills/regional-bft-conformance/SKILL.md) 按触发、前态、守卫、输出组织验收。已检查可见技能目录，未找到可选的 `protocol-spec-writer`，没有安装或声称使用它。

## 1. 近期目标、范围与路径所有权

[近期总计划](../milestones/README.md) 第 7–13、34–39、58、70–72 行及 [H2 handoff](../milestones/H2-session-manager-zh.md) 第 3–17 行确定本轮范围。下文“源规则 S”来自 v10/锁定上游；“适配选择 A”是本稿待冻结的工程契约，不反向修改来源。

**M1 终点**：15 个持续运行的真实健康 worker，三个独立五成员委员会，quorum 为四个不同合法 signer，固定 10-height 窗口；完成 R0→R1→R2 regular，真实 regional final 与完整结果经 Publisher→SimEspresso→独立 GFR 重放，直到 R2 首个合格结果进入 G。另测 SessionEnd 路径。相同真实结果字节还需经官方本地 Espresso 提交并读回；这是 H3/M1 的独立发布证据，官方 wall clock/Unknown 时间证据不混入确定性 trace，也不推导 QC/VID 或完整 PE 时序验证。

**H2 责任**：每个 worker 的候选表、来源/anchor 精确状态准备、Ready/一次 Enter、Selected/Excluded、本地 Pause/选回原实例、全部作用域关闭、生命周期原子消费、历史结果任务保留。H2 提供状态机与消费者接口，H1 提供实际 Program/runtime 能力，H3 提供发布与本地 GFR 的已验证重放结果。Hub 的故障脚本只控制合法输入交付，不能指定“选中这个 cid”。

M2 才加入健康主动 Force。H2 当前预留并验证通用 `BranchClosed` 和新 assignment 的消费入口，不实现治理、Force 交易/激活策略或新窗口计算。OS 重启、D、不相容派生、RESUME、批次/DA、Web、staking/TLA 不在近期实现范围。无 OS 重启仅缩小运行矩阵；普通持久化、签名历史、首收记录、terminal 守卫、完整 grant/Ack 和真实验证仍保留。

| 路径 | 本轮/后续所有权 |
| --- | --- |
| `poc/docs/contracts/h2-session-manager-healthy-preparation-v1.md` | 本轮先由 Docer 唯一写入，已交接 lead 串行修订/统一保存；Verifier 只读审稿。 |
| `poc/crates/session-manager/` | freeze 后 H2 单 Builder；独立 workspace/manifest/lockfile，不登记根 workspace。 |
| `poc/verification/session-manager-healthy/` | 本轮 lead 保存已封存的独立预期/向量/审阅；freeze 后再分配检查器文件，与 Builder 路径互斥。 |
| H2 专属报告路径 | lead 命名后分配；本稿不授权编辑其他文档。 |
| `poc/crates/types/`、根 manifest/lock、H1 `normal_running`/原型/vendor/proto、H3 bridge/viewer-gfr、commander | H2 不可写。需要的公共改动回到相应 owner 串行冻结，不能复制编码或抢写 shim。 |
| `regional_BFT_sim/` 及其 Git 历史 | 只读，保持 gitlink、工作树和历史原状。 |

未来构建与 artifact 根为 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager`，统一重任务锁 `/tmp/megaroation-poc-heavy.lock`，`CARGO_BUILD_JOBS=2`，一次一个 build/重型 simulation。本轮未执行这些任务。实现命令待真实 manifest/runner 冻结后填写，不预造 CLI。

## 2. 来源 pins 与可核对边界

根 [DESIGN.md](../../../DESIGN.md) 第 3–5 行规定 v10+v2 mock 的权威顺序。交接/搭建说明已读，尤其 handoff §2、§7 和 setup §7。下面的 v10 行号固定于准备基线；Git 对象中的行号固定于表内提交。

| 来源 | 版本、章节/行号 | 本合同使用的规则 |
| --- | --- | --- |
| [v10 02](../../../formalization_v10/02_session_program_zh_v0_1.md) | 工作稿 v0.1；§3.1–3.2 行 52–135；§3.4–3.10 行 170–246 | 唯一 descriptor/cid；实际 anchor state；一次 Enter；Close 不可撤销；first-received 不重写；历史 CheckFINAL 独立于本地当前时间。 |
| [v10 03](../../../formalization_v10/03_session_manager_zh_v0_1.md) | 工作稿 v0.1；§3.1–3.2 行 52–118；§3.5–3.10 行 174–230；§4.1–4.4 行 236–283 | 两维状态、候选准备/调度、逐作用域关闭、终止先持久再 Close、历史交付不删除、G 选中与本地偏好不同。 |
| [v10 04](../../../formalization_v10/04_publisher_zh_v0_1.md) | 工作稿 v0.1；§3.1–3.5 行 43–109；PUB-1…3 行 131–139 | `(cid,H(B))` 唯一任务；完整验证后 Offer；本地 Close 不清队列；公开排除/截止才可 STOPPED。 |
| [v10 09](../../../formalization_v10/09_finality_rules_zh_v0_1.md) | 工作稿 v0.1；§3.2–3.4 行 90–148；§4.2–4.3 行 221–263；§4.7–4.8 行 354–422 | proof/body 公开完整性、原始 q_avail、公共事件顺序、regular 固定窗口、只预备已选前驱下一代、LifecycleEvent 身份与原子重放。 |
| [PoC v2](../../../formalization_v10/poc_design_zh_v2.md) | 2026-11-14；§3.2–3.3 行 121–150；§4.3–4.4 行 188–218；§5–6 行 237–322；§9.1/10.1/12.3 行 475–485/509–517/623–631 | 3×5/q4/static schedule；同节点同钟；ClockUnsafe；全部输出受控；错时 Enter；regular 的全局成功终点。 |
| Regional BFT DESIGN | gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`；§2/2.1、§3.1、§5（§5 行 96–111） | Commonware adapter、progress/finality/delivery 分开、暂停/stop 的签名边界和独立验证。 |
| Regional BFT README / AGENTS / Cargo.toml | 同一 `691cfc14…`；README “Codex workflow”、AGENTS “Workflow”、Cargo dependencies | Doc→Implement→Verify→Simulation；路径唯一 owner；Commonware `=2026.10.0`，Rust 1.95.0。旧 n=15 benchmark 不作为本任务的委员会 profile。 |
| Commonware upstream | release `v2026.10.0`，commit `92a2f68a0e131c8b69de41e84aec37da6280f212`；`utils/src/faults.rs` 行 53–95；`consensus/src/simplex/mod.rs` 行 23–115、117–155 | `N3f1: f=(n−1)/3, q=n−f`，所以 n=5/q=4；Notarization/Nullification/Finalization 分开；certification 与 body delivery 不等同。 |

本 checkout 的 `regional_BFT_sim/` 未物化，主仓对象库中也没有其 commit；未执行 submodule init/update。已通过 `git -C /mnt/reth/jy/regional_BFT_sim show 691cfc14…:<path>` 只读取得准确 pinned 文档与 manifest。`handoff_provenance/regional-bft-local-*` 具有不同来源，不替代上述 pin。Commonware 源码从 `/mnt/reth/jy/regional_BFT_sim/.tooling/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` 的 `*-2026.10.0` 读取，`.cargo_vcs_info.json` 确认完整 upstream SHA；本合同采用 pinned upstream 路径，不从原论文的阈值记法推导 q=3。

来源内容 SHA-256：

| 文件 | SHA-256 |
| --- | --- |
| `02_session_program_zh_v0_1.md` | `a8cafc846acffda6b2d08915b9ccc251825c33cf84e776dae94d67611db47eb2` |
| `03_session_manager_zh_v0_1.md` | `dbcc467b6f3f52d7832112217c3caf27de1e09bdda11b9745393aea326fd5954` |
| `04_publisher_zh_v0_1.md` | `80999009464571ed8f4b1e129b83c67638121829286ebefbf6f3f6c127f8da41` |
| `09_finality_rules_zh_v0_1.md` | `6922b77e1de0c8eaddde9f08d08d6734ae63844d406f0e6e32438e33ba0dabcc` |
| `poc_design_zh_v2.md` | `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab` |

## 3. 已有公共对象与 adapter 实际边界

以下为**已只读核对的来源对象**，不表示它们已经合入本 checkout 或通过 H2/M1 联调。历史组件文档中的“M1/M2”是各组件自己的阶段名，不等于本轮 M1 Regular/M2 Force。

| pin | 已核对路径与行号 | 消费要求 |
| --- | --- | --- |
| 03 `e9c15e04bc83b335668f3ecd837367aa87344a89` | `poc/crates/types/src/domain/objects.rs` 行 177–205、616–630、917–975、977–1095 | 直接导入 `Descriptor/Anchor/ExecutionAnchor/RegionalResult/ReplayPosition/LifecycleEvent`；结构 decode 不等于来源验证。 |
| 同一 03 | `poc/crates/authorization/src/schedule.rs` 行 70–120、136–174 | 使用 `validate_plan`、`descriptor`/`validate_descriptor` 及 `StaticScheduleProvider`；后者重算 cid，前者另核对公开 reference 来源。禁止只调用 `Descriptor::validate()` 就当身份验证完成。 |
| 同一 03 | `poc/crates/core/src/{execution,merge,metadata,fixture}/mod.rs` | 状态重建复用 03 Apply/Merge、DEP/L1 fixture 的版本；不新造按 height 取 state 的捷径。 |
| 07 `788291735ce6db3c3eb78b4594a28d730cce71cd` | `poc/crates/viewer-gfr/src/api.rs` 行 109–176、191–254；`src/lib.rs` 行 11–27 | `ReplayRecord{position,input_ref,effects,events}` 是完整来源记录；`ChainView/ResponsibilityView` 必须绑定同一处理位置；`VerifierBinding` 区分输入不足和真实证明失败。 |
| 同一 07 | `poc/docs/contracts/task-contract-viewer-gfr-v1.md` §4 行 102–140；`src/codec.rs` 行 124–191 | 复用 GFR-POS-1、GFR-LIFE-1 的 `compare_positions`、`lifecycle_projection`、`lifecycle_id`。没有 `decode_lifecycle` 导出；不得假设已有反序列化函数或另写公共 lifecycle codec。 |

核对后的公共 shape（只是已有类型的摘要，不是 H2 新协议）：

```text
Descriptor { plan: PlanRecord, anchor: Anchor, cid: Cid }
Anchor { block: BlockRef { height, origin, block_id }, timestamp_us }
ExecutionAnchor { anchor, state: ExecutionState, native_hash }
AnchorEvidence { source: Genesis(ConfigRef) | Regional(ResultKey)
                         | Force(ReplayPosition) | Resume(L1Position),
                 source_objects: Vec<Vec<u8>> }
RegionalResult { descriptor, block, final_proof, chain, anchor_evidence }
ReplayPosition { key_us, kind, index }
LifecycleEvent { position, body: LifecycleBody } // kind() 从 body 导出
LifecycleBody = SessionAssigned { descriptor }
              | SessionClosed { session, reason }
              | BranchClosed { branch, reason }
              | Resumed { control_seq, branch, anchor, derived_prefix }
FinalVerifier::check(&Descriptor, &PhysicalBlock, &[u8]) -> bool
```

GFR-LIFE-1 的事件身份覆盖 position、body 种类和完整 canonical body；同一 pc 的 `SessionClosed`、`SessionAssigned` 不能互相去重。同一 Espresso ReplayRecord 可能连续完成多轮 regular，必须保留整个有序 events 列表。实际 API 为 `lifecycle_id(&LifecycleEvent)`，内部调用 `lifecycle_projection`，再用 `PoC/GfrLifecycle/v1` 与既有 hash framing 计算身份；完整 event/projection 同时保留，hash 不能替代来源材料。

当前普通运行接口仍是 [NR 草案](normal-running-reg-a-preparation-v1.md)，不是 H1 已冻结 API。现有 `poc/prototypes/i1b-runtime/src/native_replica.rs` 行 524–587 仅显示旧受控 WorkCompleted/Close 与 effect 汇集边界，`native_application.rs` 行 468–472 的 close 会终止 pending 工作，`native_scheme.rs` 行 150–194 仍有旧 fixture view 1–8 守卫。它们**不能充当候选 Pause/原实例恢复 API**，也不能由 H2 复制或改写。H1 负责 normal profile 的 Create/Prepare/Enter/Pause/Close、cid 隔离的 engine/sign/timer/storage 以及真实 FinalVerifier；H2 当前不得把这些能力标为已有。

## 4. 状态、不变量与证据分层

**S1 — 每 cid 唯一实例。** `Programs[cid]` 保存固定 descriptor、原 `program_handle`、anchor evidence、准备进度/ReadyState、`entered/entry_time`、phase、selection、terminal record、regional chain 与已有任务引用。固定 descriptor 同一 cid 下发生任何不一致即 REJECT；重复相同输入只能推进原条目。状态缓存按完整 anchor/parent 链身份关联，不能只按 height 或 root 复用。

**S2 — 两维状态。** phase 为 Assigned/Preparing/Ready/Running/Paused/Terminated；selection 为 Candidate/Selected/Excluded。Ready 要求完整执行后 state 与可信 checkpoint→anchor 的逐步执行依据或已批准的原生状态验证依据。仅 root、地址、调用者 `ready=true` 或 `ExecutionAnchor::validate()` 的 metadata 检查都不足。entered 只由 false→true；Paused 必须 entered；Terminated 永不返回可运行状态。TIME 关闭不自动把 Candidate/Selected 改成 Excluded。

**S3 — 调度与公开选择。** 同一逻辑 session 本节点至多调度一个 cid；未公开选定时，只在同一已选前驱决定链上选择最高合法、Ready、未终止候选。更高候选未 Ready 不必停掉当前候选。获选后只允许匹配完整 descriptor/anchor 身份的 cid；所选未 Ready 则等待，不能运行其他已 Excluded 候选。更高 regional final 本身不改变 Selected；更高 physical block 已进入 G 则可使较低候选 EXCLUDED。候选只延伸已选前驱的一代，不递归运行尚未选候选的后继（09 §4.3 行 261）。

**S4 — 终止覆盖全部阶段。** SessionClosed/BranchClosed 作用于范围内 Assigned、Preparing、Ready、Running、Paused 全部条目；先持久 Terminated 再 Close。A 提前关闭后到达的 state、WorkCompleted、proposal、TimerFired 或旧 Enter 请求不得复活它。原 terminal 原因/时刻保留，后续关闭事实可附加留痕，不覆盖首次终止记录。

**A1 — 关闭范围 tombstone。** 除现有条目 terminal，还持久保存 closed session/branch 集合和来源位置，阻止同一公开关闭之后迟到的 descriptor 创建可运行实例。完全未知 cid 可登记历史材料，但不能越过 scope guard。此存储结构是适配选择；“全部有关实例不得再次运行”是源规则。

**S5 — 同进程 A→B→A。** A/B 为同一 session 的两个合法候选。切到 B 时保留 A 原 engine handle、view/锁、签名缓存、首收时刻、绝对 timer deadline、pending 验证/认证/提议工作及输出。GFR 后来选 A 时，将 B 置 Excluded+Terminated 并关闭；若 A 未终止，则恢复 A 调度；不 Create 第二个 A、不 Enter 第二次、不重新设 `now+timeout`。若原 deadline 已过，恢复后在受控步中处理到期条件，不能延期。暂停期间如何登记 timer 到期和缓存 callback 由 H1 冻结，不能通过销毁 future 伪装暂停。

**S6 — 新生产与历史交付分开。** Close/ClockUnsafe/stop 阻止新签名；已形成签名可按固定 verifier 组装/核对历史证明。合法历史 Final/完整结果可继续组装、去重和转交。每项发布任务以既有 ResultKey 标识，Manager 切换/Close 不删除它；H3/Publisher 根据自己的已验证公开关闭/排除/截止决定停止新重试，保留任务与收录证据。迟到结果无效则 REJECT，材料不齐则 WAIT，已关闭 cid 的完整合法历史结果不能仅因当前时间重新跑 Accept 而判假。

| 证据层 | 可观察事件/依据 | 不可从该层直接推出 |
| --- | --- | --- |
| Progress | native notarization/certification/nullification、view advance | Final、完整正文、G 接纳。 |
| Finality | 固定历史配置的真实 Finalization，q4 不同合法 signer、正确 subject/domain | body/ancestry 已齐、ReadyState、公开完整。 |
| Local delivery | 真实 finality 覆盖的连续 validated physical prefix 与执行依据 | 已发布、已获 GFR 选择。 |
| Publication | Publisher Offer/Submitted、验证过的 Espresso 收录位置、q_avail | Submitted 不等于 Recorded；Recorded 不等于 G accepted。 |
| Global admission | 本地 GFR 的 APPEND/source/position，完整区间过期检查 | 其他读者已到同一位置或后续 session 已产生结果。 |

## 5. H2 建议接口：待 H1/H3 与 lead 冻结

下面仅冻结讨论对象及语义，**不是可实现的 Rust trait 签名或 wire codec**。公共对象全部直接复用 03/07；H2 私有 action/receipt/journal 的格式、上限及 hash domain 尚未决议。

| 输入/操作 | 守卫及返回/副作用 | owner / gate |
| --- | --- | --- |
| `InitFromVerifiedGenesis(config, registry, initial Responsibility)` | 核对初始化来源、固定初始 Plan/descriptor；Genesis `pc=None`，不得伪造 SessionAssigned ReplayPosition。本地区条目进入准备；全局 duty 供后继 Plan 使用。 | 03/07 的既有来源；H2 初始化 wrapper 待冻结。 |
| `ObserveRegional(RegionalResult, verified_input_refs)` | 真实 FinalVerifier + Plan 来源 + 完整 physical chain/Merge 检查；WAIT 保留原结果；成功后提交 Offer/跨区转发意图；只有来自已选前驱的合法决定才加入下一代候选。 | H1 FinalVerifier/chain；H3 Offer receipt；H2 调度。 |
| `PrepareCandidate(plan, anchor, evidence, material)` | 重算 cid 和来源，复用唯一 handle；构造匹配 anchor 的精确 ReadyState；不足为 Preparing(missing)，冲突为 REJECT。 | 03 对象/执行；H1 Create/Prepare；H2 state evidence。 |
| `ScheduleCandidate(cid)` | 先执行 §4 的调度与 local clock/terminal 守卫；受控 Pause 旧候选，再对 Ready 新候选一次 Enter，或恢复 Paused 的原实例。无合格候选返回 WAIT/REJECT。 | H1 Pause/Resume 原句柄语义；H2 选择。 |
| `ConsumeReplayRecord(source_binding, record, same_position_views)` | 仅消费本地 GFR 已完整提交的记录，按 §6 原子更新全部生命周期；不能把孤立远端 lifecycle 当权威。 | H3 record 边界/来源；H1 root commit/guard；H2 reduction。 |
| `ClockStep/ClockUnsafe` | 与 H1 同一 NodeClock；每一进入/恢复/实际签名再次检查 lease 与 stop；stop 等号拒绝。ClockUnsafe 的封存范围和 local reason 映射见 O3。 | H1 时钟与 terminal 接口。 |
| `ProgramOutput(cid, finality/delivery/evidence refs)` | 保留 progress/finality/delivery 分类；Final 缺 chain 返回 WAIT；历史完整 Res 去重提交独立交付意图，不因关闭删除。 | H1 原始输出/证明/body；H2 组装；H3 发布。 |
| `ActionReceipt(action_id, outcome, durable_ref)` | 仅确认原意图；重复 receipt 幂等；失败保留同一 action_id 重试；迟到 receipt 不撤销更高优先级 terminal。 | H1/H3 实际 receipt shape 待冻结。 |

建议 H2 返回 `ManagerDelta{changed_entries, preparation_waits, durable_action_intents, audit}`，不直接发网络/arm timer/调用后台签名。所有 effect 经 H1 当前受控 grant 的完整 Ack 释放，未完成动作留待后续授权步。物理 I/O 执行可异步，协议可见状态与输出必须有可核对的提交边界。

M1 的 state preparation 选用**已验证 genesis/native checkpoint + 03 的真实执行前缀**作为起点，再接公开 G 与所需 regional chain 的 Virt；这是 v10 03 §3.7 使用 D checkpoint 的近期替代输入选择，必须明确标为 A，不能伪造 `D=G` 或声称通过 D/RESUME。每项 Ready evidence 保留 checkpoint 身份、anchor block_id/height/origin、完整执行输入顺序、DEP/L1 fixture refs、pre/post state root 与 native hash；精确 schema/大小上限见 O4。

## 6. 生命周期原子消费与重试契约（A，待冻结）

源要求是：GFR 一个事件不发布部分转移（09 行 418）；Manager 不能把同 pc 的不同事件合并（09 行 379）；终止先持久再调用原实例 Close（03 行 230、240）。**建议实现边界为一个完整已提交 ReplayRecord**，包含可能在同一 Espresso pc 连续产生的多个 close/assignment。07 的 GFR-LIFE-1 只定义每事件身份；下述 record 消费事务与 H1 root-Ack 集成是新增 adapter 选择。

1. **验证来源与连续性。** 绑定 run、worker、public config、GFR 实例/来源、前一 source cursor；核对 record 的完整位置、input_ref、有序 effects/events 和同位置 Chain/Responsibility。不能跳过未知记录，不能把 `pc` 或单个 event hash 当整个 batch 的身份。相同 cursor/position 的完整重复返回既有提交结果；同位置内容冲突拒绝。准确 record 比较/摘要格式由 H3/H2 freeze，复用现有 position/lifecycle 投影，不创造第二套公共编码。
2. **完整预检。** 按来源顺序预演全部事件与作用域，解析公共 descriptor、Plan 和所需来源依据。非法/冲突/缺乏核对依据的整个 record 不推进消费者 cursor、不暴露部分选择或关闭。普通 assignment 的执行状态尚未下载完整是明确 Preparing，并不阻止已经可验证的公开关闭提交；不得等 ReadyState 才记录已知 SessionClosed。出现 RESUME 相关输入本 profile 报 Unsupported 并保持整条 record 未消费，不能跳过后继续。
3. **形成一个提交候选。** 同时包含：source cursor/record 完整身份及同位置公共视图引用、全部 lifecycle IDs 和原 projections、selected/scheduled 变化、closed-session/branch tombstone、全部受影响条目的 terminal/selection、待创建/准备/关闭/恢复意图、已有 publication/evidence task 引用。G 的已验证 APPEND 若使较低 anchor 不可能获选，相应 EXCLUDED 也纳入本次提交。`events=[]` 的已验证 record 仍正常推进消费者 cursor，不跳过其 effects/公共视图。当前记录中先 assigned 后又 closed 的 cid 最终不可运行；不向 engine 暴露中间短暂可签名状态。
4. **与 H1 守卫共同持久提交。** 对受影响 cid 的 terminal/暂停调度 guard、H2 状态及下游 action outbox 必须落入一个可审查的提交边界，再允许输出。不能出现“cursor 已提交但关闭记录丢失”或“Manager Terminated 但 engine 仍可新签名”。H1 若不能在同一 root/aux 提交中表达这些状态，gate 未过；不能自行声称多个独立 fsync 等同原子事务。
5. **提交后动作。** 对既有实例先生效禁止新签名/调度的 guard，再执行幂等 Close/Pause；准备、Create、Enter/Resume 仅在相应来源、Ready 和时钟守卫通过后执行。A→B 切换不得有同 session 双 Running 的观察点。worker 成功处理语义需区分 `Committed(actions_pending)` 与 `ActionsApplied`，不得因 cursor 前移把后者伪报完成。
6. **下游重试。** 每个 action 绑定原 record 身份、事件/动作序号和 cid，持久化后按相同 ID 重试。可核对的下游 receipt 才移除/完成该意图。重投 record 不再创建实例、第二次 Enter 或刷新 deadline；未完成 Close/Offer 仍可继续。过时 Enter/Resume 意图在执行前重查最新 terminal/selection，不能覆盖后来的 Close。尚未收到 receipt 的失败不伪造成功，也不创建新任务身份。
7. **源确认与 Ack。** consumer cursor 代表已持久接受整条 record 并保存其全部待执行义务，不代表全部下游动作完成。对 source 的确认必须晚于该提交；网络发送、timer/work、生命周期审计、publication 意图及确认均进入同一授权步的完整 Ack。严格区分 durable acceptance、外部 effect release、action receipt。没有 Ack 后后台偷偷送出的 Close/Final/Offer。
8. **本轮失败边界。** 独立 fixture 可在同进程模拟提交前拒绝、提交成功但 action 尚未执行、同一 action/record 重投、关闭动作暂未获 receipt，并检查全有/全无及幂等。实际断电、OS crash、新 PID 恢复/丢 Ack 恢复另属 OPEN；这不允许删除正常路径的持久记录。

SessionClosed/BranchClosed 即使当前没有匹配 Programs 也保存 scope tombstone；随后到达的新候选先受它限制。SessionAssigned 在同一 record 选择 A 时，将同 session 不匹配候选置 Excluded 并持久关闭；若 A 已 TIME-terminated，保留 Selected+Terminated 供历史结果判定，不再次进入。普通公开关闭无需统一把历史 Selected 改为 Excluded；scope 关闭事实和发布资格由来源事件独立表达。

## 7. Regular 与 SessionEnd 的必需时序

**接满路径 S。** 当前窗口 `[1,10]`，GFR 接纳 h10 的 physical block 后，以该块为生产 anchor；后继 `(β,r+1)` 的窗口严格为 `[11,20]`。Plan 的 reference_block 仍为前驱初始 anchor，不能改成 h10；R1 知道前驱选中后即可算后继 Plan，逐候选准备。R1→R2 同理，最终 R2 的真实首结果须由 GFR APPEND 到 G。区域完成、Publisher submitted、SessionAssigned 都不足以代替此终点。

**SessionEnd h9→h11 路径 S+A。** 测试须使用健康 worker 与显式受控的结果/发布延迟，让 R0 最后及时进入 G 的 physical block 为 h9，h10 在前驱截止前不满足公开接纳。推荐先保持 h9 候选为 R1 可调度的最高 Ready 候选；如另包含更高候选回切，须显式安排其到达时刻，不能偷偷跳过最高 Ready 调度规则。

| 阶段 | 必须存在的实际事件与断言 |
| --- | --- |
| 前驱尚未公开关闭 | 本地 GFR 已选 R0；后继 Plan 固定为 `[11,20]`；R1 收到经过真实 FinalVerifier/chain 校验的 h9 regional result，并从可信 checkpoint 实际执行到 h9，建立候选 A 的 ReadyState。这里不要求 h9 已成为最终 handoff anchor。 |
| 原 h11 生产/准入时段 | 各 R1 worker 独立 Enter/Schedule A，真实 engine 在固定窗口 Build h11；proposal 首收满足 `ts*(11)−2δ_G ≤ firstSeen < ts*(12)+T_win` 且 `firstSeen < stop_A`，实际投票还须当前 clock/lease/stop 与 BFT 守卫。真实 q4 final、完整 body/ancestry 和执行证据被保留。没有 GFR 选中也可以生产合法候选，但不能把它加入 G。 |
| 提前缓存 | 本地结果可提前 Offer，H3 可将公共 proof/body 按原位置缓存；是否已公开与是否仅本地缓存必须分别记录。保存原 firstSeen、view、timer deadline、结果键和实际 publication 位置；不在后续选择时重签或重写时间。 |
| 公共 SessionEnd | 只有 Espresso/L1 覆盖截止且前序事件处理完，本地 GFR 才输出 R0 SessionClosed(Complete) 及以 h9 为 anchor 的 SessionAssigned(A)；R1 窗口仍 `[11,20]`。本地 clock 到点只导致 TIME，不制造这条公共记录。 |
| 后继接纳 | 若 A 尚未终止且曾 Paused，选回原实例；历史缓存结果无需再生产。GFR 在合格 Espresso 事件中按原 q_avail 与实际接纳键 x 检查全部新增 virtual heights，接纳由 h11 决定覆盖的 h10 padding 与 h11。仅当该已验证链确无 physical h10 时才可展开 padding；已存在但缺正文的 physical ancestor 不能被当 padding。 |

选择一个具有足够 h11 生产/验证/发布余量且接纳时 h10、h11 均未 Expire 的精确输入 schedule，冻结 `T_win`、grace、链接/准备预算；不从旧 benchmark 延迟臆测可行。若实际前提不足，保留失败/INCONCLUSIVE 证据，不能把窗口改为 `[12,21]`、`[14,23]` 或迟到时重新给十个高度。负例要求：**只等 SessionEnd 后才开始准备/生产 h11，不能倒填 firstSeen 或固定旧 height 来冒充正例**。

待冻结的数值示例：`ts*(h)=10s+h×1s`、`T_win=200ms`、`T_grace=3s`、`δ_G=1ms` 时，R0 stop 为 21s、SessionEnd 公共键为 24s；R1 仍为窗口 `[11,20]`、start=21s、stop=31s。h11 的首收区间为 `[20.998s,22.2s)`，当前本地时刻至少为 21s 才可通过不超前检查，实际新签名还须 clock 健康且小于 31s。因此到 SessionEnd=24s 才开始制造 h11 无法满足原准入窗；应在早先窗口产生并缓存 A9-h11，之后按真实公开位置接纳。此示例不禁止 A9 获选/恢复后在当时合法 slot 产生 h14 等新结果，只要求用于证明 h9→h11 的证据确实来自原 h11 窗口；具体 profile 与 expiry 预算仍须 O7 冻结。

## 8. 独立验收索引（全部 PLANNED / NOT RUN）

Verifier 先从 §2 的来源与以下 guard 写独立预期，再读实现；不调用被测 reducer 生成 expected。各 fixture 标明真实性：纯状态机 fixture 可以验证控制顺序，只有实际 H1 engine/签名/提交证据可关闭 native gate。每条记录输入、前态、预期输出/禁止输出、source refs 和最小 mutation。

| ID | 触发与前态 | 核心守卫及预期 |
| --- | --- | --- |
| H2-A01 | 同一合法 descriptor 两次 Prepare/Create；同 cid 篡改 anchor/plan/config | 相同输入仅一 entry/handle；篡改 REJECT，无额外实例。终止后重复 Create 保持 terminal。S1；02 §3.4。 |
| H2-A02 | 已知 anchor，只有 root/下载地址/部分 checkpoint；随后完整输入到达 | 前者 Preparing/WAIT，不能 Ready/Enter；完整执行与 anchor 身份一致才 Ready。错误 state、同 height 异 hash、异 parent cache 拒绝。03 行 58、81、196–198。 |
| H2-A03 | 候选 Ready；分别非成员、未调度、Excluded、Terminated、clock 不安全、`C_m=stop` | 非成员/失格/到 stop 不 Enter；未 Ready/未调度 WAIT；成功一次 Enter，重复回报原 entry_time/view，不再初始化。02 行 182–184；03 行 246–255。 |
| H2-A04 | A Running，B 更高合法候选但 Preparing；B 后来 Ready | B 未 Ready 时 A 不被销毁；B Ready 后按规则 Pause A、Enter B；同 session 不双 Running，原 A 保留。03 §4.3。 |
| H2-A05 | A→B；GFR 选回 A | A 原 handle/view/锁/签名/firstSeen/pending work/deadline 逐项可核对；只恢复调度，Enter(A)计数仍 1。B Excluded/Terminated；不重置超时。S5。 |
| H2-A06 | A Paused 时原 deadline 到达；恢复 A；另一个变体已到 stop | 原 deadline 到期按受控事件处理，不以恢复时刻延期；到 stop 的 A 永久终止，无新签名。v2 行 195；03 行 257–259。 |
| H2-A07 | SessionAssigned 选中尚未 Ready 的 A，B 已 Running | 原子排除 B；A Selected+Preparing；等待真实状态，不能因 B 现成可用继续运行它。03 行 224、263。 |
| H2-A08 | 同高度不同 anchor；更高 regional result；更高 physical 已进入 G 三个输入分别比较 | 严格按身份区分；regional final 只改变候选偏好，不能选中/判死；已验证 G 进展才触发适用 EXCLUDED。03 行 180、275–276。 |
| H2-A09 | SessionClosed 涵盖 Assigned/Preparing/Ready/Running/Paused；BranchClosed 包含多个 session 与提前准备后继 | 一次提交覆盖全部条目和 scope tombstone；先持久 terminal/guard，Close 幂等；不只处理 Running。03 行 225、230、242。 |
| H2-A10 | A09 后迟到 material、descriptor、Enter、TimerFired、WorkCompleted；关闭时尚无条目 | 没有复活/第二实例/新签名；closed scope 阻止迟到新登记变可运行。历史材料可留存。S4/A1。 |
| H2-A11 | 一个 record 内同 pc 的 Close→Assign；以及同 Espresso pc 多轮 regular | 全列表按来源顺序消费，各 lifecycle ID 独立；没有只按 position 去重或中间 cid 短暂运行。GFR-LIFE-1/§6。 |
| H2-A12 | 完整重复 record、同位置异 body、缺 event 边界、未来 cursor 越过未知记录 | 完整重放幂等；冲突/缺边界/跳跃拒绝或等待，不推进 cursor；hash 相同仍保存 full projection。§6.1。 |
| H2-A13 | record 最后一个 descriptor/source 无效或必需依据缺失；普通新 anchor state 仅缺下载 | 前者整个 batch 无可见半更新；后者可原子提交关闭及 Selected+Preparing，禁止 Enter。不能把普通状态下载 WAIT 变成延迟已知关闭。§6.2。 |
| H2-A14 | 提交前失败；提交后 Close/Offer 暂无 receipt；重投相同 record/action；迟到 Enter receipt | 提交前全无；提交后 cursor、terminal、intents 全在；同 ID 重试不丢关闭/任务；迟到回执不覆盖新 terminal。报告 Committed 与 ActionsApplied 分别状态。§6.3–8。 |
| H2-A15 | A 有待发布完整结果，切换或本地 TIME；另一路输入公开关闭/排除 | 前者任务保留并可继续合法重试；后者交 H3 依据公共事实 STOPPED/已 RECORDED，原结果/证据不删除。04 PUB-3。 |
| H2-A16 | 真实 final proof 先到，body/ancestor 或状态依据后到；无效 body/错误 ancestry 对照 | finality observation 独立保留；未完整 delivery/Ready/Offer；补齐并验证后才对应推进，假数据 REJECT。02 §3.10–3.12；Regional BFT §5。 |
| H2-A17 | 同结果重复输出/Offer；迟到合法 historical Final；Notarization 冒充 Final | ResultKey 去重；历史 Final 不重跑当前 Accept；progress proof 不走 Final 路径。03 §3.5–3.6；04 §3.4。 |
| H2-A18 | Close/stop/ClockUnsafe 与 Build/verify/certify completion、sign 请求交错 | 每次实际新签名出口仍读当前 guard，终止前形成的材料可交付；ClockUnsafe 范围/原因按 O3 冻结后验收，不能临时等同 Time。v2 行 216、252。 |
| H2-A19 | 从已选前驱收到若干合法后继候选，随后收到未选候选的后继结果 | 仅已选前驱的一代可提前准备/调度；不递归启动未选下一代，不从全局脚本指定 Selected。09 行 261。 |
| H2-A20 | Genesis `pc=None`、不属于本地区的 assignment、unsupported Resumed | Genesis 通过验证后的 duty 初始化，不伪造公开位置；非成员可记录公开选择但不建可签名实例；RESUME 整记录 Unsupported，无部分关闭/分支切换。§5–6。 |
| H2-A21 | 仅 fixture 输入 BranchClosed(Force/Expired) 与独立的新 branch assignment | 旧 branch 所有阶段与未来迟到候选均受关闭约束；新 descriptor 来源独立核对。只证明消费接口，不声称 Force 激活或 M2 PASS。 |
| H2-A22 | manager delta 内任意非法 effect、未持久 action/terminal、提前发送或 Ack 后后台输出 | 整步按 H1 commit 契约拒绝/保留未释放义务；不能部分释放合法项，不能绕过完整 Ack。v2 §5.1，H1 native gate。 |

独立集成项按 H2/H3/H1 联合证据逐个关闭；H2 纯规则完成不能自行宣布 M1：

| ID | 输入与成功条件 | 依赖/结论限制 |
| --- | --- | --- |
| H2-N01 | 至少一条真实 engine A→B→A trace，包含原签名/绝对 deadline/pending 回调与一次 Enter；另一条实际 Close/stop 负例 | H1 native Pause/Resume/Close 接通后执行；fixture 通过不替代。 |
| H2-N02 | 实际公开 CheckFINAL 对 3/4 个不同合法 signer、重复 signer、错 cid/domain/view/target、Notarization 逐一判定；Manager 再验证 chain/state | H1 真实 verifier；H2 不实现第二套验签。证明、delivery、Ready 分别记录。 |
| H2-I01 | 15 PID、3×5/quorum4、固定窗口、R0→R1→R2；每个 worker 的 descriptor/Ready/Enter/Close 有轨迹；R2 首结果真实进入 G | M1 merge 的系统终点，H1/H2/H3 同一准确 integrated SHA。没有脚本 signer、同 cid 重建或全员启动屏障。 |
| H2-I02 | §7 SessionEnd h9→h11，生产和首收在原 h11 准入窗；cached 结果选中后以原 q_avail 接纳，h10 padding/h11 都检查 Expire | H1 真实生产+H2 提前准备+H3 正确 public replay；另测截止同键结果被拒绝和等待至 SessionEnd 才准备的负例。 |
| H2-I03 | 同 I01/I02 实际 RegionalResult/publication 字节提交官方本地 Espresso 并读回，逐字节/hash 对比 | H3/16 独立官方兼容证据；不改变 SimEspresso 主 trace 或时间结论。 |
| H2-I04 | 固定外部输入，错时 Ready/Enter、3/2 候选分裂、不同 wall pause/reader 延迟 | same-cid 四节点共同参与区间才可声称有界进展；wall 不进入协议时间；相同 replay position 的公共选择一致。seed/延迟组合 freeze 后由 Tester 记录。 |

## 9. 依赖 gates、待决项与交付标签

| gate | 关闭所需材料 | 本轮状态 |
| --- | --- | --- |
| G0 来源/准备 | §2 pins、03/07 对象与当前 adapter 边界可审查；独立预期准备完成 | 本稿完成来源记录；独立复核待 lead 汇总。无实现授权。 |
| G1 M0 | M0 新的完整交付 SHA、合入的 02/03/06/07/16 与 M2 备用 09 来源清单、兼容复验结果 | **OPEN**。`4a9a6b0…` 不是 M0。 |
| G2 H1/H3 接口 freeze | O1–O9 的明确决议及精确签名/codec/receipt/ownership、H1 最小接口提交、H3 已提交 record 来源接口、独立期望 | **OPEN**。准备文档不是接口发布。 |
| G3 H2 组件 | 单 Builder 实现于获准路径、独立 A01–A22 核对、同候选 SHA、无未解释的必需失败 | **NOT STARTED**；可称有限状态机组件 PASS，不关闭 native/M1。 |
| G4 H2 native | H1 实际运行接口与 O9 准入/certification 映射已接受，N01/N02 和受控输出/terminal 路径通过独立检查 | **OPEN**；至少真实停车/选回与停止不可由 fixture 替代。 |
| G5 M1 | M1 merge 的同 integrated SHA 上 I01/I02、官方独立 I03 与批准的有界矩阵；原始 artifact 与 checker | **OPEN**。全局成功终点未达前不报 M1 PASS。 |

冻结前必须逐项定案：

| ID | 有限未决问题 | owner / 需要的决议 |
| --- | --- | --- |
| O1 | H1 `Create/Prepare/Enter/Pause/Resume/Close` 的实际调用/返回签名、handle 生命周期、五成员 cid 隔离、instance/command 身份 | H1+lead；发布准确接口 SHA。H2 不能销毁 engine 或直接操作内部锁/签名/cache。 |
| O2 | Pause 时 timer 到期、挂起 verify/certify/propose callback、入站缓存、任务唤醒与恢复顺序；哪些可观察字段证明原实例与原 deadline | H1+独立 verifier；须能完成 N01，不只是内存 phase 修改。原 deadline 不平移是源约束，具体事件表示是 adapter 选择。 |
| O3 | `ClockUnsafe` 本地终止原因与作用范围：当前 Running、Paused、未 Enter 候选如何封存，clock 恢复后哪些合法后续 cid 可进入 | v2 要求终止当前 cid/禁止新 Enter；03 canonical `CloseReason` 没有 ClockUnsafe。H1+lead 显式冻结 local cause/持久守卫及候选范围；**不擅自映射成 Time，不新增公共 enum**。已 Terminated cid 一律不复活。 |
| O4 | Ready evidence schema、可信起点、03 Apply/Merge 的调用接口、state/work completion receipt、输入容量及验证开销 schedule | H1/H2+03 来源核对；M1 不实现 D 的替代输入明确标注。缺数据与无效数据必须不同。 |
| O5 | H3 完整 record/source cursor/来源承诺的输入形状；多轮 regular 的同 pc 批次边界；record 比较与重复/冲突策略 | H3+H2+lead；GFR-LIFE-1 原样复用。缺 `decode_lifecycle` 时先确定 typed 进程内传递或 owner 提供版本化 wire，不由 H2复制。 |
| O6 | H2 状态、H1 terminal/调度 guard、source cursor 与 action intents 如何在同 root/aux 事务持久；提交/释放/下游 receipt 的失败与重试边界 | H1/H2/H3+lead；确定 action ID、journal/receipt 版本与 byte bounds、full Ack 所有字段。原子声明须有真实边界证据。 |
| O7 | h9→h11 的精确 virtual schedule、`T_win`/grace/expiry/准备预算、h10 regional 与公开延迟；A→B→A 是否单列 trace | H1/H2/H3+Tester；必须在原 h11 窗口生产，不能为适配结果修改 protocol window/firstSeen。 |
| O8 | 同高度冲突候选、来源不相容或缓存上限的行为与清理策略 | H2+verifier；有效同 cid 决定链受 BFT 安全性约束，冲突报错/隔离，不自行 hash tie-break 选择一条“合法链”；不能回收仍可选的 Paused 实例来制造第二个实例。 |
| O9 | H1 如何把节点本地 firstSeen/clock/terminal 准入映射到 Commonware 要求的全 context 确定性 verify/certify；WAIT、永久本地拒绝、parent 选择和决定有效性的边界 | H1+独立 verifier；按 pinned Regional BFT DESIGN §2.1 行 47 与 Commonware simplex/mod.rs 行 108–115 提供已接受的映射和 BFT-2 parent/decision 证据。不能把本地超时直接当全局 payload 无效、把 WAIT 当 false，或用可变本地时刻替代确定性认证谓词；H2 保留 pending work 本身不关闭该 gate。 |

交付时保存：合同版本/hash，M0/H1/H3/03/07 pin，准确候选/独立验证/集成 SHA 与 dirty 状态，工具/lockfile hash，精确命令、seed/固定输入、完整原始 trace、逐 cid 生命周期与实际 handle/guard/timer/签名证据、Ready 执行依据、source record/cursor、action/receipt/完整 Ack、真实 final/body/ancestry、公共 q_avail 与 G 接纳 position、失败与观察截止界。诊断 wall/PID 和虚拟协议 trace 分开。

本合同允许准备工作在依赖实现前推进；实际编码等待 G1/G2、合同 freeze 及总协调的后续实现授权。有限 fixture/原生/系统证据分层报告 `PASS/FAIL/INCONCLUSIVE`，不把准备稿、组件通过、发布提交或区域 final 写成 M1/M2 完成；普通持久化通过也不构成 OS 重启验收。

独立审阅记录及 oracle 封存信息见 [H2 准备报告](../h2-session-manager-healthy-preparation-report-v1.md)；draft2 修正事件 API 描述和公开排除动作，并新增 O9，不改变已封存的独立预期。
