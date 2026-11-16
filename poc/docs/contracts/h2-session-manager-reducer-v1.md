# H2 Manager reducer v1：M1 纯组件实施合同

2026-11-16。合同 ID：`H2-REDUCER-1`。状态：**FROZEN — H2 lead 与独立 bft_verifier 于 2026-11-16 确认本纯组件切片，立即交单 Builder 实施**。本合同只冻结纯内存 reducer/不可变 delta 与动作意图，不等待尚未发布的 `healthy-session-api`，也不以组件通过代替真实 Manager/H1 接线或 M1 验收。

授权为 [`M1-implementation-dispatch-20261116.md`](</mnt/reth/jy/megaroation_poc/poc/docs/milestones/M1-implementation-dispatch-20261116.md>)，准确提交 `d55d39f73593676325bdc0fbb7ba3845ca93025f`，已完整只读核对。M0 为 **`35d382ac445cc8bc779f0d18fd23cb57e54680da`**；当前实施分支 `codex/h2-session-manager-implementation`，自身准备文档重放后基线 `dd3110f77724a273a4e7982c05d6c4b36dea4b9d`。旧准备分支、`027ba8b`、封存 oracle 与旧审查 hash 保留；本合同不改写其历史结论。

## 1. 来源与本切片边界

继续使用 [regional-bft-conformance](../../../.agents/skills/regional-bft-conformance/SKILL.md)，来源 pins/内容 hash 见 [历史准备稿 §2](h2-session-manager-healthy-preparation-v1.md#2-来源-pins-与可核对边界)。本轮已复核 M0 的实际 03/07 类型和 H1 尚未冻结的接口提案；具体约束如下。

| 权威/现有对象 | 固定版本与段落 | 本切片规则 |
| --- | --- | --- |
| v10 02 | `02_session_program_zh_v0_1.md` §3.1–3.2 行 52–135；§3.4–3.6 行 170–192 | descriptor 不变、一次 Enter、终止不复活、原签名/首收/实例历史保留。 |
| v10 03 | `03_session_manager_zh_v0_1.md` §3.1–3.2 行 52–118；§3.7–3.10 行 190–230；§4 行 236–283 | 准备与选择分开；最高 Ready 候选；全部阶段关闭；先持久 terminal 再 Close；已形成任务不因切换删除。 |
| v10 04/09 | 04 §3.4/PUB-3 行 95–109/137；09 §4.3 行 246–263、§4.7–4.8 行 354–422 | 公共关闭与本地 TIME 不同；完整 ReplayRecord；同 pc 多 lifecycle；只预备已选前驱的一代；fixed window。 |
| v2 | §3.3、§4.4、§5.1、§6；行 143–150、202–218、237–256、285–322 | 3×5/q4、同 NodeClock、ClockUnsafe、所有输出经 grant/Ack、错时 Enter。 |
| 03/07 | 来源 `e9c15e04bc83b335668f3ecd837367aa87344a89` / `788291735ce6db3c3eb78b4594a28d730cce71cd`，现已在 M0 | 直接导入 domain 对象、`ProgramConfig::from_plan/regional_cid`、03 `metadata::l2_timestamp_us`、07 `ReplayRecord/compare_positions/lifecycle_projection/lifecycle_id`。 |
| Regional BFT/Commonware | gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，DESIGN §2.1/3.1/5；Commonware 2026.10.0，`92a2f68a0e131c8b69de41e84aec37da6280f212` | progress、finality、validated delivery 分别记录；纯 reducer 不生成或验证 native final proof。 |
| 最新实施派发 | `d55d39f…`“已决边界”2–5、9 | H1 是唯一持久提交 owner；完整 record 与 intents 共同接受；durable guard 之后失败关闭；Pause 原实例；早 h11 与晚合法 slot 分开。 |

H1 提案 `/home/jy/.codex/worktrees/0b8e/megaroation_poc/poc/docs/healthy-session-program/interfaces-h1-v1.md` 仅作未来接口方向参考。其 §2.2“失败保留原状态”受到最新派发的单调 durable guard 规则约束，不能用来回滚已落盘 terminal。本合同的内部类型不复制提案中的公共 `ReadyToken/ProgramHandle/CommandContext`。

唯一实现路径为 `poc/crates/session-manager/`，独立 `[workspace]`；package `poc-session-manager`、lib `poc_session_manager`。只依赖 M0 的 `poc-types`、`poc-core`、`poc-viewer-gfr`；如确需额外纯值依赖由 H2 lead 在本 crate manifest/lock 内登记，不改根 manifest/types。Verifier 独占 `poc/verification/session-manager-healthy/` 的本切片新 harness/预期路径，不覆盖封存准备文件。本轮 Docer 仅写本合同及历史稿顶部说明；不改代码、不构建、不 commit。

本切片不创建 engine，不做 RPC/网络/文件/数据库、后台线程、计时器、签名或执行状态验证；不消费未来 API crate，不实现 dispatch/launcher/Force/RESUME。H2 的纯状态、计划、receipt 账本是本 worker 的逻辑组件；H1 负责真实 root/aux、guard、原实例和完整 Ack，H3 负责真实 public replay 与发布。

## 2. 明确信任边界

本组件接受**同进程可信 adapter 已验证的输入**。其生产调用方须先验证配置/registry/Plan 来源、regional proof/完整 ancestry、精确执行状态和公共日志来源；H2 只重做下面列出的结构与归属检查。所有名称中的 `Verified`/`PreparationCompleted`/`CommitRef` 是前置条件，不是由名字、hash、Rust constructor 或 `Eq` 产生的认证。

| 输入 | H2 实际检查 | 必须由真实 adapter 提供的事实 |
| --- | --- | --- |
| 已验证 genesis/descriptor/candidate | config reference、03 结构、重算 cid、固定 anchor/plan、worker 委员会归属、后继关系与窗口、相同 cid 唯一 descriptor | 真实 PoP/公开 reference/Plan 来源；候选 anchor 确来自已选前驱同一有效决定链。 |
| 完整已验证 ReplayRecord | 固定 source/连续私有 seq、完整 typed Eq、position/index/input_ref 对应、每个 event.position 相等、lifecycle 投影/ID、有序列表、支持的 M1 事件 | 07 实际重放已经原子提交整条记录，没有省略 events/effects；source seq 没漏记录。H2 不重新运行 GFR。 |
| PreparationCompleted | 原 Prepare action、worker/cid、完整 Anchor、config、instance reference、验证版本、证据引用完全匹配 | H1 已执行 03 验证并持久保存完整 ReadyState。不能只给 root/地址；本 slice 的 fixture 只声明该输入事实，不证明它。 |
| ActionReceipt/CommitRef/Clock 输入 | 原 action/delta、revision、worker/instance/事件种类、重复/冲突、单调时间与代次 | 真正运行时执行/落盘、NodeClock/lease 和实际 output release 事实。 |

不提供 `ready: bool` 或生产 `always_true` verifier。PreparationCompleted 是绑定原 Prepare 意图的一条内部观察，**不是第二个公共 ReadyToken**。未来 bridge 将 H1 ReadyToken/ReadyState 验证后映射成此观察；此时才可关闭真实 Ready gate。恶意调用方仍能伪造这些“已验证”事实；本库不防调用方伪造验证结果，不把内部 evidence hash 当密码学证明、完整数据或持久证明。测试报告必须带 `trusted-input fixture` 标签。

## 3. 最小 Rust 接口与内部值

以下是本 crate 的 Rust 级接口；无 wire/serde/canonical 协议承诺，公共域对象仍直接使用 03/07。Builder 可拆小型模块，但不得引入泛型事务框架或重新实现公共编码。除有穷枚举和值引用外，Manager/ManagerDelta 内部字段私有；view/getter 只读，不提供任意状态 restore/set_phase/set_ready。

```rust
impl Manager {
    pub fn new(binding: ManagerBinding, config: PublicConfig,
               genesis: VerifiedGenesis) -> Result<Self, ManagerError>;
    pub fn plan(&self, input: ManagerInput, local_time_us: u64)
        -> Result<PlanOutcome, ManagerError>;
    pub fn accept_commit(&mut self, delta: ManagerDelta, commit: CommitRef)
        -> Result<CommitStatus, ManagerError>;
    pub fn latch_commit_failure(&mut self, delta: &ManagerDelta,
                                guard_ref: Hash32) -> Result<(), ManagerError>;
    pub fn view(&self) -> ManagerView<'_>;
    pub fn dispatchable(&self, local_time_us: u64) -> Vec<&ActionIntent>;
}
enum PlanOutcome { Unchanged(CommitStatus), Wait(WaitReason), Planned(ManagerDelta) }
enum CommitStatus { Committed { actions_pending: usize }, ActionsApplied, Faulted }
```

`plan` 和 `dispatchable` 无 I/O/签名/时钟读取，不改 Manager；`accept_commit` 只是依据可信 commit attestation 安装内存状态。它本身不 fsync。`local_time_us` 来自 H1 同一 NodeClock，要求不低于已接受的本地时刻；不能用 wall 或 public `position.key_us` 代替。返回 Unchanged/Wait/Error 不安装原计划；已生效的安全 guard 不因这些返回撤销。

| 内部值 | v1 最小字段/约束 |
| --- | --- |
| `ManagerBinding` | `manager_id:Hash32, worker:Pid, public_config:ConfigRef, gfr_source:Hash32, preparation_version:u16`；version 固定 1；manager_id 区分 run/worker 实例，由可信启动层绑定。 |
| `VerifiedGenesis` | `descriptor:Descriptor, source_ref:Hash32`；固定初始选中职责，`pc=None`、source seq 尚无值；不造 SessionAssigned 位置。初始不具备 clock-ready 或 Prepared 事实。 |
| `VerifiedCandidate` | `descriptor:Descriptor, predecessor:Cid, provenance_ref:Hash32`；仅已选前驱的一代，Handoff Plan/reference/window 符合 §5。 |
| `VerifiedReplayInput` | `source:Hash32, seq:u64, record:ReplayRecord, validation_ref:Hash32`；首次 seq=0，此后严格 +1。验证引用保留，但重复 record 的语义身份不依赖观察者新选择了哪份等效依据。 |
| `PreparedRef` | `prepare_action:ActionId, worker:Pid, cid:Cid, instance_ref:Hash32, anchor:Anchor, config_ref:ConfigRef, validation_version:u16, execution_ref:Hash32, provenance_ref:Hash32`。不携带或导出公共 ReadyToken，不把 root 当执行依据。 |
| `ActionId` | `manager_id:Hash32, worker:Pid, origin_revision:u64, ordinal:u16`；普通 Eq/Ord 结构，不进入 cid 或公共协议。origin_revision 是产生该意图的决策目标 revision，receipt/retry 不改它。 |
| `CommitRef` | 绑定 manager/worker、base/target revision、**本 delta 的完整 cause=(ManagerInput,local_time_us)** 和 `durable_ref:Hash32`；不得只按 revision 匹配另一候选 delta。可提供 `CommitRef::for_delta(&delta,durable_ref)` 供可信调用方/test 构造，名称/构造不证明落盘。 |
| `ManagerDelta` | base/target revision、完整 cause、私有 next state、按稳定顺序的 action intents、要与 root 同步安装的 terminal/scope/schedule guard 变化、审计变化；只读访问，不可任意构造/修改。 |

input 的有限集合：

```text
ManagerInput = RegisterCandidate(VerifiedCandidate)
             | ConsumeRecord(VerifiedReplayInput)
             | PreparationCompleted(PreparedRef)
             | Receipt(ActionReceipt)
             | Schedule(SessionId)
             | ObserveTime
             | ClockReady { generation:u64, basis_ref:Hash32 }
             | ClockUnsafe { basis_ref:Hash32 }
             | RetainResult { key:ResultKey, verified_result_ref:Hash32 }
```

`RetainResult` 只保存由可信生产路径确认的完整历史结果引用及 Publish 意图，不携带 `Final=true`、不生成证明，不推导 G 接纳。结果必须属于已登记 descriptor 的 cid；相同 ResultKey 重复相同引用幂等，等效证据引用可留作审计但不造第二任务；有意义的内容冲突由上游真实验证器拒绝，H2 不能只凭不同 ref hash 宣告同 key 区块冲突。

action 的有限集合：`Create{descriptor,provenance_ref}`、`Prepare{cid,instance_ref,anchor,provenance_ref,validation_version}`、`Enter{cid,instance_ref,prepared_ref}`、`Pause{cid,instance_ref}`、`SelectBack{cid,instance_ref}`、`Close{cid,instance_ref:Option<Hash32>,cause}`、`Publish{key,verified_result_ref}`。ActionIntent 还含 `id`、已知前置 action IDs；所有引用属于本 manager。Create receipt 才给出真实 opaque instance_ref；尚未 Create 就关闭可用 `None` 的 guard-only Close 意图，不能为了 Close 新造 engine。

## 4. 状态、动作与 receipt 规则

状态保存 revision、本地时刻/clock generation/enter gate、唯一 Programs、每 session 的公开 selected 与本地 scheduled、closed session/branch tombstone、source cursor 与完整记录历史、action ledger、结果任务引用、Faulted 标记。SessionId/ReplayPosition 在 03 中没有 Ord；内部按 `(beta,r)` 建 session map，位置排序调用 07 `compare_positions`，不擅自为公共类型另定义顺序。

每个 Program 保存 descriptor、immutable provenance、instance_ref、prepared observation、`entered/entry_time`、`enter_action:Option<ActionId>`、selection、phase、首个 terminal cause、原动作引用。phase 使用来源状态 Assigned/Preparing/Ready/Running/Paused/Terminated，另有适配过渡态 Entering/Pausing/Resuming，用来明确动作尚未被真实 receipt 确认。selection 为 Candidate/Selected/Excluded。**产生 Enter intent 只进入 Entering，不把 entered 置 true，不固定 entry_time，不声称 Running。**

1. 新本地区 descriptor 只产生一次 Create；CreateApplied receipt 绑定 worker/cid，保存原 instance_ref，进入 Preparing 并产生一次 Prepare。同 cid 后续请求、public assignment、重试都复用原实例，矛盾 instance_ref 拒绝。非本地区的 assignment 只维护公共选择，不给它创建可签名实例。
2. PreparationCompleted 只能完成该 cid 已发出的原 Prepare action；全部绑定相等才标记其 Applied 并使未终止/未排除条目 Ready。缺材料没有这个观察，保持 Preparing；本组件不制造成功完成。完整相同 completion 重复幂等；同 action 改 anchor/worker/version/instance 为冲突。终止后匹配旧 completion 仅保留迟到审计，不 Ready/Enter。
3. Schedule 先按 §5 选唯一合法 Ready/已 entered 原候选并检查 clock/stop。切走 entered A 或已有尚未完成/确认 Enter 意图的 A 时，安装禁调度 guard 并产生或复用原 instance 的 Pause(A)/适用 Close(A)；entered A 为 Pausing；新 B 的 Enter/SelectBack 依赖 A 的 PauseApplied 或 CloseApplied，以及 B 的 Create/Prepare 完成。旧 schedule guard 禁用与新选择在 H1 同一受控提交安装，未完成依赖前 `dispatchable` 不返回 B 的运行命令。不能把 entered=false 当成引擎从未进入的证明；对真实尚未 Enter 的 A，PauseApplied 只确认 parked guard，不伪置 entered=true/Paused，原 Enter ID 继续保留。迟到 Entered 只更新历史，并保持最新 park/terminal guard。
4. 每 cid 至多分配一个 Enter action ID；未确认 Enter 在候选失去本地调度时留为 held pending，不重新分配 ID；重新可调度时仍重试原 ID。匹配 Entered receipt 才单调设 entered=true 并保存首次 entry_time。若当前已终止/排除或另一个候选被调度，只记录已发生的历史 Enter，不恢复 Running；需要关闭/停车的意图仍保留。不能把“未收到 Entered”当作确认 engine 从未进入。
5. A→B→A：已 entered 的 A 恢复只发 SelectBack，带原 instance_ref；它依赖仍未完成的 Pause/Close 安全义务。不得重新 Create/Prepare/Enter、改 deadline 或清签名历史。实际 engine/view/callback/deadline 的保持由 H1 gate 证明；纯组件只证明命令与引用未换。
6. `ActionReceipt` 成功类别为 `Created{instance_ref}`、`Entered{instance_ref,entry_time_us}`、`Paused{instance_ref}`、`SelectedBack{instance_ref}`、`Closed{instance_ref:Option<Hash32>}`、`Published{key}`，均带原 ActionId 与 `receipt_ref:Hash32`。Prepare 完成仅用 PreparedRef。类别、cid/worker/instance/ResultKey 与原动作不符拒绝；未知 ID 拒绝；匹配完整重复幂等；同 ID 的成功观察冲突拒绝。首次真实成功字段不能由后续重复回执覆盖。
7. `Retryable{failure_ref}` receipt 保留原动作 Pending 与同一 ID，不重置实例/准备/进入计数。公共关闭或新选择导致旧 Enter/SelectBack 不再合格时，ledger 保留它及审计，`dispatchable` 屏蔽它；不得绕过 terminal 继续执行。对永久取消的生产动作记 Cancelled；其后到达的匹配成功 receipt 只能更新历史事实，不能解开 guard。Pause/Close 与 Publish 义务不因生产动作取消被删除。
8. 应用回执自身产生下一次状态 revision，必要时创建新的后续 Prepare/Pause/Close/SelectBack 意图；原 action 的 origin_revision 不变。一个未接受 delta 丢弃后，在**相同 state/input/time** 上重新 plan 得到同样 IDs/顺序；已接受 record 或 action 重复不分配新 IDs。所有排序按来源事件顺序；同作用域多个 cid 使用 Cid 字节顺序；每个决策按 Create→Prepare→Pause/Close→Enter/SelectBack→Publish 的依赖规则稳定编号，安全依赖优先于编号。
9. `Committed{actions_pending}` 表示逻辑义务已由可信 commit attestation 接受。仅该已接受决策及其后续义务全部 Applied/按明确终止规则 Cancelled 后，状态报告 ActionsApplied；Retryable/held 动作仍计 pending。这不宣称 publication Recorded、G accepted 或 runtime 已通过。View 应同时列出 observed phase、selection、待办与原 action ID。

## 5. 候选、选择、关闭与本地时钟

**固定 descriptor。** 用 03 `Descriptor::validate`、`ProgramConfig::from_plan` 和 `regional_cid` 检查结构/config/cid；重算不能替代来源验证。stop 使用 03 `l2_timestamp_us(config,h_max+1)`，checked arithmetic，不接受调用方重新给出的窗口/stop。重复完全相同 descriptor 复用，任一字段不一致同 cid 拒绝。

**提前准备边界。** RegisterCandidate 必须引用已有 selected 前驱 cid，target session 是相同 branch 的 `r+1`；window 为前驱 `[h_max+1,h_max+10]`；reference_block 为前驱初始 anchor.block，source_region 为前驱 Plan.region。anchor 必须在前驱合法决定链范围内；“在同一已验证链”由可信 adapter 保证，H2 检查引用和高度边界。没有 selected 的前驱不能递归准备下一代。候选可在未 Selected 时 Ready/Enter；局部调度选择最高合法 Ready anchor。相同 height 不同 anchor 的所谓同链候选拒绝为 CandidateConflict，不能 hash tie-break 自选真链。

**公开选择。** SessionAssigned 保留 selected descriptor，即使该 worker 不属其地区。对本地区，匹配条目 Selected；其他同 session 候选 Excluded+Terminated，包括 Preparing/Paused；不存在则登记 Assigned/Create 意图。所选缺 ReadyState 只处于 Assigned/Preparing，关闭/cursor 照常接受，Enter 不可派发。旧更高 regional result 不能改 public selected；普通 `Schedule` 不能改它。公开 APPEND 判死使用已验证 07 `ReplayEffect::Append{last,source:Regional(ResultKey)}`：按既有 07 语义，last 是该完整结果的 physical target 高度，source.cid 必须属于已验证的 selected 前驱历史。按 effects 原顺序维护该前驱已接纳 physical_tip 高度的单调事实；同 predecessor 且 anchor.height<last 的后继候选原子置 Excluded+Terminated，包括 Preparing/Paused/Entering。随后 RegisterCandidate 同样检查此 tip 屏障，阻止迟到低 anchor 重开。此语义由 trusted GFR adapter 承诺，不是从任意未验证高度推断来源；未知 source 不猜测，M1 不支持 DER。如果未来 bridge 不能保证该现有 effect 语义，必须先提供绑定原 record 的具名 verified tip 接口。

**scope tombstone。** SessionClosed(Complete/Expired) 和 BranchClosed(Expired) 覆盖所有阶段和未知未来 cid；持久化逻辑 tombstone 后，迟到 Register/Create/PreparationCompleted/Enter 不得开放该范围。关闭原 terminal reason 不覆写，后续 public facts 可另存。TIME/ClockUnsafe 不改变 selection 为 Excluded；历史 Selected+Terminated 是合法状态。所有已有结果/Publish 任务保留，公开停止新发布重试由 H3 判断。

**M1 unsupported。** Force/Resume 的 ReplayKind/InputRef、Resumed、Force/Resume/不相容恢复理由或 Force/Resume cause 的新 assignment 返回 Unsupported，整 record 不消费。BranchClosed 的通用类型保留，当前只接受真实 Expire 来源的 Expired；不得直接给 reducer 喂模拟 Force 激活当 M2 证据。普通 session 完成与全局 Expire 支持；不引入 D。

**Time。** ObserveTime 为所有 `now >= stop` 的未终止条目安装私有 `LocalCause::Time` terminal 与 Close 意图，覆盖 Preparing/Paused。Schedule/dispatchable 在所有路径重查 `now < stop`；到 stop 即使尚未提交 ObserveTime 也不能返回 Enter/SelectBack。H1 实际签名出口仍独立检查同 NodeClock/lease/terminal；纯 now 值不是租约证明。

**ClockUnsafe 的冻结保守选择 A。** 私有 `TerminalCause = Public(CloseReason) | LocalTime | LocalClockUnsafe{generation,basis_ref}`，不扩展 03 enum、不把 Unsafe 映射成 Time。Unsafe 关闭**本 worker 当前已登记的所有未终止 cid**，包括 entered Running/Paused 以及 Assigned/Preparing/Ready/Entering；保留历史结果。enter gate 立即置 Disabled，并记录 `unsafe_session_fence` 为已观察 selected/Programs 的最大 `(beta,r)`，没有 session 时为空。该选择比“至少当前 entered”保守，固定用于此 slice。

ClockReady 仅在初始或 Unsafe 后接受更大的 generation 和当前可信依据；重复相同代次/依据幂等，回退或同代不同依据拒绝。恢复只打开**fence 之后的后续 session**的进入资格；fence 内 session 的新 anchor 也不能绕过禁入。旧 cid terminal 永不撤销，已提前准备但被 Unsafe 关闭的后继也不复活。新 session 仍需合法公开选中或已选前驱一代候选来源、重新真实准备和正常时钟；这不是 OS restart 或 RESUME。本地时间不允许倒退；ClockUnsafe 可使用最后已接受的 local_time_us 表达关门事件，不要求先取得新的安全 clock sample。public record/结果接收在 worker 禁入期间仍可处理；不能把 clock 不安全当作改变 GFR pc 的理由。

## 6. 完整 record 原子计划与提交失败

07 的 `ReplayRecord{position,input_ref,effects,events}` 有完整 typed Eq，没有整 record codec 或公开验证函数。消费时保存原 record，不引入“第二套 record hash 编码”。复用 position 编码/比较和每个 lifecycle projection/ID；比较全部 input_ref/effects/events，不能只比 event IDs 或 position。seq 是此组件的私有 source 接受序号，不能改变 GFR 的 public ReplayPosition。

1. `source==binding.gfr_source`；seq 小于 next 时必须命中保存的同 seq 完整 record 并且 typed Eq，否则 SourceConflict；相同记录返回原接受状态，不重做动作。seq 大于 next 返回 Wait(SourceGap)。next 的 position 须严格大于已接受 pc（07 compare），每个 event.position 必须等于它；InputRef 的 variant、session/height/q 与 ReplayKind/ReplayIndex 精确一致。M1 仅允许 Espresso/SessionEnd/Expire。
2. 保留 effects/events 各自顺序，不能自行给两份列表编造共同 ordinal。检查内嵌 descriptor/position/支持的 reason、投影长度、完整容量后才形成 delta。一个 pc 可 Close→Assign→Close→Assign；逐事件计算最终 selected/terminal，但**只给最终仍可运行的实例产生 Create/Prepare/Enter 类意图**。中间已关闭 assignment 只留来源/tombstone，不启动 engine。`events=[]` 仍接受完整 record 和 effects、推进 source cursor。
3. 不能仅以 Eq/投影/Hash32 证明 record 来自 GFR；validated-input 信任前提见 §2。结构矛盾或缺验证来源的 record 返回 Error/Wait，无半 cursor/selection/tombstone 安装。后继执行材料缺失不属于结构/来源错误：合法关闭、Selected+Preparing、cursor 和全部 action intents 同属一次 delta。
4. `accept_commit` 验证 base/target revision、完整 ManagerBinding 及 cause=(ManagerInput,local_time_us) 与 CommitRef，成功后一次安装 next state、cursor、terminal/tombstone、schedule guards 与 pending ledger，revision +1。相同 delta/receipt 重投返回原状态；另一 input 同 revision 的 receipt 不能通用。没有两个不同 delta 被同一 revision 接受。原 record 重放与 commit 重放分别去重。
5. H1 真实提交必须覆盖 delta 对应的 root/aux 与安全 guard，之后才调 `accept_commit`；收到可信 CommitRef 前本库不声称 durable。H1 pc 与 H2 source cursor 分开，H2 等待/失败不能回滚 H3 GFR。此 slice 的 fixture CommitRef 只测试内存语义。
6. **提交前拒绝**可以丢弃未生效 delta，原状态不变。**一旦任何 terminal/tombstone/Unsafe guard 已落盘而后续失败**，调用 `latch_commit_failure`：先核对 delta 属于同 manager/worker/base 上下文，拒绝外国或错误基线 delta；任何拒绝不能清除已持久的安全事实。仅合入该 delta 已知的单调 terminal/tombstone 与禁入安全投影，保留原 cursor/assignment 未接受的事实和 failed delta/cause/guard_ref，状态转 Faulted。生产动作全部禁止 dispatch；不能回滚到 Running、清 guard、复用旧 delta 开门。此 slice 无 Faulted 恢复接口；该 run 停止并报告，关闭未完成义务仍可审计。H1 更早安装的任何 guard 本身也必须 sticky，不能等待库调用才生效。
7. Faulted 可以只读；plan/accept 新普通动作拒绝 RuntimeFaulted。重复同一 failure 输入幂等，冲突 failure 不弱化已有 guard。此政策经 lead 同意，用于最小 M1 组件，不建设数据库恢复事务。OS crash/restart/丢 Ack 恢复依然 OPEN。

## 7. 错误、等待与容量

`ManagerError` 最小 variants：`InvalidBinding`、`Malformed`、`DescriptorConflict`、`CidMismatch`、`WrongCommittee`、`CandidateConflict`、`WrongPredecessor`、`PreparationBinding`、`SourceConflict`、`InvalidRecord`、`ReceiptConflict`、`UnknownAction`、`StaleDelta`、`CommitBinding`、`ClockRegression`、`ClockGeneration`、`Unsupported`、`Capacity{kind,limit}`、`Overflow`、`RuntimeFaulted`。可附受限结构化 id/数值，不要求字符串相同。`WaitReason` 为 `SourceGap{expected,received}`、`Preparation{cid}`、`ClockDisabled`、`NoRunnableCandidate`、`PendingAction{action}`；无输入缺失机制时不伪造 Ready。

v1 固定 bounds：每 worker 64 Programs、128 selected/closed sessions、16 branch tombstones、2048 完整 source records、4096 action ledger entries（含已完成以保证去重）、256 ResultKey 任务。每 record 最多 64 lifecycle events、128 effects；03 encoded descriptor ≤16KiB、每 record 全部 lifecycle projections 合计 ≤1MiB；每 action 最多 64 dependencies，receipt/引用为定长 Hash32/typed value。只保存来源/执行对象引用，不复制 payload/state 全文。计数与长度使用 checked arithmetic，先检查再扩容；达到界限显式 Capacity，不清除 terminal、旧 instance、action 去重或原始来源以继续运行。

限额是有界 M1 slice 选择，不是协议全局上限；不静默增加。已知合法关闭遇到资源/提交故障时，生产 bridge 必须停止 worker 的新生产并报告，不能因“delta未接受”继续签名；H1 的失败关闭 guard 仍是必要 gate。纯组件检查容量拒绝无半安装，不能据此声称实际故障已被安全处理。

## 8. 两种 SessionEnd 证据与独立验收

早 h11 与晚合法 slot 是两项独立 integration 案例。参数示例 `ts*(h)=10s+h×1s, T_win=200ms, grace=3s, δ_G=1ms`：前驱窗口1–10，stop21s，SessionEnd24s；后继始终11–20、start21s、stop31s。**早 h11** 在 `[20.998s,22.2s)` 首收且合法当前时刻投票，提前准备 A9、真实产生 h11 并缓存，到24s public assignment 后用原证明/firstSeen/q_avail 接纳。**晚合法 slot** 未提前产出 h11，24s 之后按当前合法 h14 等生产并真实覆盖 padding，不补造 h11 首收；仍不移窗。纯 reducer 只测试两条选择/准备/任务轨迹，真正时序、proof/body 和 G APPEND 由 H1/H3/M1 gate 核对。

Verifier 先封存输入与期望，再读实现；全部以下案例当前 **NOT RUN**。既有准备 oracle 不重写，本 slice 新建独立文件并引用旧 hash。

| ID | 输入/边界 | 纯组件必须观察到的输出/禁止行为 |
| --- | --- | --- |
| R01 | 相同/冲突 descriptor、错误 cid/config/member | 唯一 entry/Create；冲突原子拒绝；非本地区 assignment 不建 engine。 |
| R02 | Create→Preparing；匹配/跨 worker/cid/anchor/version/instance Prepare completion；关闭后完成 | 只有原 action 完整匹配可 Ready；终止后不复活；root/URL/ready bool 没有可用入口。 |
| R03 | Ready Schedule 与 Create/Prepare/Pause 尚未完成 | Enter pending 不等于 entered/Running；依赖未满足不可 dispatch；真正 Entered receipt 固定首次时间。 |
| R04 | A Ready，B 更高未 Ready→Ready；A→B→A | 先保留 A，后 Pause A/Enter B，选回 A 只 SelectBack 原 instance；Enter action 唯一；无同 session 双可运行出口。 |
| R05 | Pausing/Entering 期间再次选择、关闭或迟到 receipt | 依赖和最新 guard 生效；迟到成功只留历史，不恢复终止实例；未完成原动作用同 ID。 |
| R06 | 选中未 Ready A、B Running；scope Close 各 phase/未知未来 cid | 关闭/Selected+Preparing/cursor 一起接受；迟到候选受 tombstone 限制，不因 state WAIT 留 B 运行。 |
| R07 | 一个 record 同 pc 多 lifecycle/多轮 regular、空 events | 全 typed record 保留；不会 position 去重丢 assignment；中间已关闭 assignment 无启动意图；空记录 cursor 仍前进。 |
| R08 | source seq gap/repeat/conflict；position/input_ref/index/event.position 变异；仅 effects 变化 | Gap WAIT；完整同 record 幂等；其余冲突拒绝且无半状态；不只比 lifecycle hash。 |
| R09 | record 尾部坏 descriptor、超限、unsupported Force/Resume | 全条无半 cursor/assignment/close；明确 Unsupported/Capacity；无模拟控制激活。 |
| R10 | 相同 state/input/time 重计划、丢弃未 commit delta、receipt 推进 revision 后 record 重放 | action IDs 逐字相同/不重号；origin revision 不漂移；已接受义务不重复分配。 |
| R11 | wrong commit cause/base/worker、重复 commit、Retryable/重复/冲突 action receipt | 严格拒绝错绑定，原ID待重试；完整匹配幂等；Committed 与 ActionsApplied 分开。 |
| R12 | commit 前丢弃；durable guard 后故障；再次 Schedule/ClockReady | 前者无变化；后者保留 terminal/tombstone、cursor 未伪推进且 Faulted；没有 rollback/recover/新生产出口。 |
| R13 | stop−1/stop/stop+1，Preparing/Paused；Unsafe 当前 entered/未进入全部；恢复旧/新 session | stop 等号禁止；Unsafe 全登记关闭且 worker gate 禁入；大 generation 恢复仅 fence 后新 session，旧 cid 永不复活。 |
| R14 | 未选前驱后继、再下一代、same-height 异 anchor、较高未 Ready | 拒绝错误来源/冲突；不递归提前启动；仍按最高 Ready 而非最大已知高度调度。 |
| R15 | 原结果 RetainResult、重复任务、Pause/Time/Unsafe/public Close | 任务引用和稳定 Publish ID 保留；本地停止不伪造公开 STOPPED/Recorded/G accepted。 |
| R16 | 早 h11 与晚 h14 两条 fixture 轨迹 | 均 fixed11–20；早例复用已准备/结果引用；晚例不伪造 h11/firstSeen；不声称真实 engine/proof 已验收。 |
| R17 | 每类容量恰好界限/界限+1、u64/u16计数溢出 | 界内合法；超界明确拒绝无半安装，不为腾空间丢历史/terminal。 |

组件交付保存精确 candidate/review SHA、合同/新 oracle hashes、命令/输入/结果和 trusted-input 限制。未来命令使用独立 manifest、Rust1.95.0、`--locked`，重任务使用 `flock /tmp/megaroation-poc-heavy.lock`、jobs=2；cargo/target/artifacts 位于 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager`。本合同不列虚构的已运行命令。

## 9. freeze 与后续消费 gates

本 slice 的结构/语义已由 H2 lead + Verifier 收口冻结，不再请求用户确认，不等待 H1 全部测试。单 Builder 仅实现 `poc/crates/session-manager/`（含自有 manifest/lock、组件 tests/examples）；独立验收材料由 Verifier 在单独临时目录编写、lead 串行收录到 `poc/verification/session-manager-healthy/reducer-v1/`。直接实现 R01–R17 及独立矩阵适用断言，只报告“纯 Manager reducer 组件”结论。H1 API 发布后另做薄 bridge，不回头复制其公共类型。

仍必须单独关闭的生产 gates：H1 的真实 ReadyToken/精确执行、同实例 Pause/SelectBack/callback/绝对 deadline、真实签名/FinalVerifier、typed delta 到实际 root/aux/guard 的原子提交与 fail-closed、全部 effect 进入完整 Ack、H3 本地 verified replay 到连续 source seq 的真实绑定与出版任务接收。它们不阻塞本合同纯逻辑实现；组件 fixture 不关闭它们。

M1 最终仍要求15真实健康 worker、R0→R1→R2、R2 首结果进入 G、SessionEnd 与官方同字节读回。progress proof、final proof、完整 delivery、提交、Recorded、G 接纳继续分开；OS 重启、D/RESUME、Force/M2、批次/DA、Web 保持原未覆盖状态。

冻结依据：增量来源预期 SHA-256 `9ad71186b41e85a5e30100a7fd63d9a009435e12e4459046e3056ad8cd7965e5`，独立矩阵 `4d78b0c12f65f7be5f0c5a2da0048b7815f3ef9e027290e942960ac39cd87395`。Verifier 审阅后，lead 补入 APPEND physical-tip 屏障、未确认 Enter 的停车依赖、包含 local_time 的 commit cause，以及 failure latch 外来 delta 拒绝；独立 Verifier 确认这些最小修正后无纯组件 freeze hardblock。真实 H1/H3 gates 保持上述 OPEN。
