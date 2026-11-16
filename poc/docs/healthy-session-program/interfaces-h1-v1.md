# H1 v1：生命周期、精确状态准备与真实区域结果接口提案

日期：2026-11-16。状态：**M1分切片接口合同；API切片精确定义见[api-slice-v1.md](api-slice-v1.md)**。正式派发`d55d39f73593676325bdc0fbb7ba3845ca93025f`已选择运输方案A、提交分层与所有权；M0=`35d382ac445cc8bc779f0d18fd23cb57e54680da`已合入。本文伪接口是runtime语义说明，精确可编译DTO/ports以API切片为准，不声称已有实现。未决native问题只阻对应切片，Q-ADMIT不阻API开发。

## 1. 唯一公共对象与现有消费边界

公共对象只引用 03 `e9c15e04bc83b335668f3ecd837367aa87344a89` 的 `poc_types::domain`：`Pid/RegionId/MemberId/SignerIndex/Hash32/Cid/ConfigRef/ResultKey`、`Descriptor/PlanRecord/Anchor/ExecutionAnchor/ExecutionState/PhysicalBlock/RegionalResult/AnchorEvidence`、`InputStatus/InputMissing/InputError`、`SessionId/CloseReason/ReplayPosition/LifecycleEvent`、`FinalVerifier`。身份使用 `Canonical`，已提供 codec 的复合对象使用 `CanonicalObject::encode_checked`；03 不提供 RegionalResult/AnchorEvidence 的完整 canonical codec，本稿也不发明第二套。

`ResultKey{cid,block_hash}` 使用 **physical digest**；03 `native_hash` 用于执行后原生链身份/G-D 比较，不能互换。`ConfigRef`、02 `kernel_config_hash`、runtime profile/run manifest hash 分别记录。签名 cid 不包含 PID/运行 id/到达时间/Clock/transport 权限 revision。

H3 继续使用 07 `788291735ce6db3c3eb78b4594a28d730cce71cd` 的 `GFR-POS-1/GFR-LIFE-1/GFR-PUBLIC-1`。`viewer_gfr::codec::PublicObject` 是 `Body(PhysicalBlock)` 或 `FinalProof{descriptor,target,proof}`，`PUBLICATION_CODEC=0x47465201`；其 body/proof 编码、lifecycle projection/id 归 07/H3，H1 不另定义发布 canonical。06 负责 PE-PUB/LOG/READER，H1 不改其存储协议。

H1 transport/RPC 的操作 id、job token、runtime handle、receipt 等只是 adapter 控制元数据，必须有新版本及明确 hash preimage。它们不能替代 03 对象/证明，也不能改变公共 cid。真正 serialized wire、长度界、字段号、hash 与错误码由 Q-WIRE 冻结；此处用 typed 伪接口表达语义。

## 2. H1↔H2 lifecycle：`H1-LIFE-1`

### 2.1 能力与输入输出

一个 worker 持有一个 NodeClock/受控 driver，以及 `Programs[Cid]` 的原生实例能力。H2 保管逻辑候选表和下列 opaque handle，不读取/写入 native lock、vote 或 timer 内部结构。H1 不自行从 Hub 场景脚本选择 cid。

```text
ProgramHandle = opaque worker-local identity bound to (run, pid, cid, instance_generation=1)
CommandContext = (operation_id, actual_grant_ref, input_ref, expected_local_revision)

Create(ctx, descriptor, verified_plan_basis) -> Assigned(handle, observation) | Wait | Reject
Enter(ctx, handle, ReadyToken)             -> Entered(original_entry_time, observation) | Wait | Reject
Pause(ctx, handle, schedule_decision_ref)  -> Paused(observation) | Reject
SelectBack(ctx, handle, schedule_decision_ref) -> Running(observation) | Wait | Reject
Close(ctx, handle, CloseReason, close_basis) -> Terminated(original_close_record) | Reject
Observe(handle) -> immutable ProgramObservation
```

`ProgramObservation` 至少含 cid/handle、created/entered/paused/terminated、原 entry/close 记录、当前 view、签名历史摘要与持久引用、timer/deferred-work 摘要、last committed root/operation。摘要用于审计，不可代替原始签名字节/完整状态。Prepare 能力见 §3；Ready 是完成精确状态验证的结果，不能由外部传一个 bool。所有 mutating 调用在受控 grant 内执行并进入完整 Ack/root。

| 操作 | H2 的责任/来源 | H1 guard 与结果 |
| --- | --- | --- |
| Create | 验证/请求 Plan 和合法 anchor 来源，按 cid 去重 | 重算 descriptor/cid/本地注册信息；创建固定独立记录，不 spawn 可签名 engine；同 cid 返回原 handle，包括已终止状态。 |
| Enter | 条目 Ready、委员会成员、本地 scheduled 指向该 cid，未 Excluded | ReadyToken 精确绑定原 handle/anchor/验证版本；Clock/lease/stop/terminal；首次启动原 engine view1，重复返回原 entry；不要求五节点同步屏障。 |
| Pause | 更高合法候选被调度，旧候选仍可选；同 session 最多一个 Running | 在当前 grant 静止点停车全部 cid 共识任务/签名能力；保存原实例与 pending callbacks；不 abort/drop/restart engine，不把临时暂停持久成 Terminated。 |
| SelectBack | 合法调度决定，或 GFR SessionAssigned 选回原候选 | 校验仍为同 handle 且未 terminal/excluded、Clock/stop；若 Paused，只恢复原调度；不得第二次 Enter；若 never-entered，走原 Enter 流程。 |
| Close | 已验证 lifecycle 的作用范围，或本地 TIME/ClockUnsafe | 先提交不可撤销 terminal，再停止新签名；重复返回原关闭记录；覆盖尚未 Enter 的候选；历史输出/证书/持久 publication 不删除。 |

“selected/excluded” 属于 H2/GFR 逻辑，不等同于 entered/paused/terminated。TIME 关闭不自动 Excluded；已关闭候选的历史结果仍可能在公开截止前被接纳。

### 2.2 生命周期原子性与 parking

H3输出完整`ReplayRecord`的有序`LifecycleEvent`，H2保留每个GFR-LIFE-1身份，不能只按position去重。每worker仅H1拥有持久提交。完整记录的结构/来源预检成功后，H2生成不可变opaque ManagerDelta/action intents；H1把关闭/tombstone、Selected+Preparing、整批cursor与待办义务同root提交，返回`RootCommitted(actions_pending)`；实际动作完成另给`AppliedReceipt/ActionsApplied`。后继状态缺失只使准备WAIT，不延迟已知合法Close；结构或来源未通过时不能冒充已提交。

cursor表示整批义务已持久接受，不等于所有动作完成；H3自己的GFR pc不随Manager滞后回滚。提交前未生效计划可拒绝；durable terminal/tombstone或ClockUnsafe guard安装后失败必须保留事实和待办，不能回滚Running；必要时停当前step/runtime并fail-closed报告。既有guard与root提交失败的精确receipt/错误由API切片表达，真实durability仍由后续runtime证明。不得引入第二套服务数据库事务。

**推荐 P-RUNTIME：** 在已有单 worker ControlledDriver 内新增 cid-scoped 调度状态，按 `(pid,cid)` 保存 engine、签名 cache、witness policy/ledger、storage partition、task/timer/work 归属。manager/reader/Clock/root 任务不依附某个选中 engine 的生命周期。Pause 只停止该 cid 的共识任务 poll/新签名；保持同一 engine future、mailbox、原 timer deadline、原 callback producer 与已授权 I/O 身份。incoming/due timer/work completion 可在受控有界队列记录，不能在 Pause 期间触发协议新签名。SelectBack 先检查 Clock/stop/terminal，再按原有确定性队列交付已到期事件；不能“从现在重新计时”。

若 Pause 时已有 native I/O，必须先到可记录的静止边界，保留其 pending/completed disposition，不能取消已发生 fsync 的事实。Close 使其后继生产 continuation 无效，但历史数据/receipt/发布义务仍记录。达到队列界是显式资源失败，不丢锁、签名或 deadline。本提案禁止每 cid 启动一个自行快进的 Runner，也不把销毁 Engine 后从空状态构造称为 Pause。

当前 `driver.rs` 虽有 `sessions: BTreeMap<Id,Session>`，`Session` 仅 stop/retired；`witness: Option<WitnessRuntime>`、`Inner.witness_partition: Option<String>` 和 `configure_witness` 的 first-grant 前限制不支持运行后多 cid witness。Q-RUNTIME 必须覆盖这个真实缺口；只放宽 scheme view 上限不足以交付 H1。

## 3. H1↔H2 exact state preparation：`H1-SPREP-1`

### 3.1 精确材料与责任

H2 决定准备哪个候选、保存进度并控制取消；H1 执行受控取数/验证/执行工作，返回可审计 Ready/WAIT。状态算法复用 03 `poc_core` 的 `check_setup/Apply/Merge` 与 metadata/DEP provider，不能另做脚本状态根。近期 M1 采用已验证 genesis checkpoint 和真实 regional/G 的已验证前缀；不以“未启动 D”为理由虚构 D 状态。Force/Resume 来源 typed variant 保留，但没有相应 verifier 时返回明确 Unsupported/Wait，绝不默认真。

```text
Prepare(ctx, handle, anchor_evidence: AnchorEvidence,
        material: StateMaterial, provenance: PreparationBasis)
  -> Preparing(job, Missing[]) | Ready(ReadyToken, ReadyState) | Reject
Supply(ctx, job, additional_material, provenance_delta)
  -> Preparing(same_job, Missing[]) | Ready(...) | Reject
CancelPrepare(ctx, job, cancellation_basis) -> Cancelled(job, durable_record)

ReadyState = {
  exact_execution_anchor: ExecutionAnchor,
  validated_prefix_refs, execution_input_refs, source_basis_refs,
  validation_profile_ref, completion_grant_ref, durable_record_ref
}
```

`StateMaterial/PreparationBasis/ReadyToken` 是版本化 adapter 容器提案，不是新增公共 canonical 对象。其内部身份/blocks/state/source 引用复用 03；已可表示的 missing 使用 03 `InputMissing::{Reference,L1Header,ControlCoverage,Parent,State,Dep}`。外层缺 final proof/anchor provenance 可用具名 H1 missing tag，须冻结且保留原输入引用，不能泛化成不可核对字符串。

`PreparationBasis` 明确区分：公开验证配置和 registry、checkpoint 的完整 state/metadata/native hash、从该 checkpoint 到 exact anchor 的有序原生/physical 输入与来源、已验证 regional result 的 ResultKey+proof/ancestry、各 reader 实际已交付的 L1 header/control coverage/DEP。`AnchorEvidence.source_objects` 只是待解析字节，绝不是认证布尔值。

### 3.2 Guard、WAIT、取消

1. 检查 `(run,pid,cid,handle,job_generation)`、fixed descriptor/anchor、public config/verifier version、计划来源及请求 revision。ReadyToken 不能跨 worker/cid/anchor 使用；同 height 不能复用其他链 state。
2. 身份正确但来源/输入未到：WAIT 并持久 missing 和原进度；未来 fixture 行在磁盘、未投递 finalized 数据、只给 root/URL 均仍 WAIT。已知冲突/坏 codec/无效执行是 REJECT，不伪装成空材料。
3. 完成完整状态执行验证后，核对 exact anchor id/height/metadata 与 `ExecutionAnchor`；ReadyToken 绑定本次执行依据及持久记录。Ready 不能越过尚未验证的 prefix gap。
4. `Supply` 为同 job 的幂等增量；输入相同不重复执行已提交 effects，矛盾输入原子拒绝。相同 cid 再 Prepare 复用原实例/进度，不造第二 engine。
5. `CancelPrepare` 使该 job generation 失效并记录原因；Close/Excluded 额外使 cid terminal。迟到 completion 只能记 discarded/缓存可验证材料，不能 Ready/Enter/签名。取消 job 本身不重定义 v10 Terminated；是否可再次准备由 H2 原候选状态决定，用新 job generation 且原 handle。
6. Pause 不是取消；保持原 work producer 和结果归属。恢复时重查 terminal/Clock/stop，且旧 generation 的 completion 永不更新新请求。确切 cancellation/event 序列在 Q-PREP/Q-RUNTIME 冻结。

## 4. H1→H3 真实 final/body：`H1-RESULT-1`

### 4.1 三层输出

| 层 | 能力/记录 | 允许的消费者结论 |
| --- | --- | --- |
| 进展 | `ProgressObserved{cid,epoch,view,kind,native_bytes_ref,verified_basis}`，kind 为 Notarization/Certification/Nullification 等 | 仅对应 native 进展，不能给 Publisher 一个合格 Final。 |
| 最终证明 | `FinalProofObserved{ResultKey,descriptor,native_finalization_bytes,proof_basis,observation_grant}` | 已过真实 CheckFINAL 的证据；缺完整body/ancestry/状态仍 pending。仅digest证明观察与完整 `FinalEvent` 分开记录。 |
| 可消费交付 | `RegionalDelivery{result: RegionalResult, validation_basis, delivery_id, durable_ref}` | 目标 body、anchor来源、完整 physical ancestry、执行有效且连续 prefix 可用。H2按SP/SM组装，H1提供真实proof/body/验证能力；H3重新用公开依据验证并 Offer。 |

proof 先到时可先记录 native subject/digest 的证书检查；目标 PhysicalBlock 还未取得时，不能声称已经调用要求完整 block 的 03 `FinalVerifier::check`。补齐目标后做完整 CheckFINAL；缺祖先时继续 pending。阶段记录必须能区分“证书主体密码学成立”与“绑定完整目标并验证 delivery”。

最终证明格式采用固定 Simplex `Finalization<bls12381_multisig::Scheme<PublicKey,MinSig>,Digest>` 的原生 bytes；payload digest 精确绑定 03 physical digest。检查完整 consumption/codec bounds/epoch/view/历史域/参与者 bitmap。Native Reporter 收到一条 activity 不是默认验证通过。不得用 Notarization、脚本 Final flag、Hub 聚签或 always-true verifier 补齐。

### 4.2 公开 verifier 与复用方式

直接实现 03 已有能力：

```rust
// Existing 03 trait; H1 supplies its real implementation.
fn check(&self, descriptor: &Descriptor, block: &PhysicalBlock, proof: &[u8]) -> bool;
```

构造 `NativeFinalVerifier`（提案名）只接受公开 immutable PublicConfig、ValidatedRegistry、明确 verifier profile/历史公钥。核对 descriptor/cid、窗口/目标 hash、native epoch/view/Finalize subject、真实四 signer、namespace。它不读当前本地 Clock，不靠运行时私有状态，不在 check 内作 RPC；Plan 的实际来源、ancestry、执行和 GFR 选中资格由外层检查。若历史配置缺失，外层 typed WAIT；未知格式/版本或已知错误 false/REJECT，不能降级 fixture。

07 现有 `VerifierBinding{verifier,basis,profile}` 可以直接持有该 trait；`VerificationProfile::Native{protocol_version,verifier_version,source_revision}` 必须与 03 PublicConfig 匹配；`VerificationBasis::require` 提供真实历史依据，不能恒真。H1/H3 共用同一真实 verifier 实现/版本，fixture profile 与 native profile 明确隔离。

新 namespace 必须由公开固定字段导出并进入 verifier/profile/config/cid 选择；候选为 `config_ref + cid + epoch + verifier_profile_version` 的域分离编码。具体 tag/preimage/黄金 bytes 是 Q-DOMAIN，不能把运行 seed/秘密配置混入公开验签。03 PoP 域 `PoC/Registry/PoP/v1` 与旧 witness `I1b/storage/pop/v1` 的差异由受审 normal policy 解决，禁止关闭 PoP。

H3 的 `PublicationVerifier` 对 `RegionalResult` 再做 exact anchor provenance、完整 hash 相连 ancestry、body/执行、CheckFINAL，随后生成现有 GFR-PUBLIC-1 对象。合法不同 proof 可对应同 ResultKey；以 block identity 去重 delivery/任务，不用 raw proof bytes 作为冲突判断。完整结果与验证依据持久保留；Close 不删除。Publisher StopDecision 来自 H3 已验证公开 position，不从 H1 本地 TIME/Pause推导。

### 4.3 大小、公开前缀与官方读回

06单条message上限64KiB、整block上限1MiB；02每packet payload≤64KiB，wrapper也占字节。03普通交易上限64KiB不保证完整encoded object≤64KiB。M1先冻结会改变状态的小交易工作量及完整object/result/attempt真实编码上界；必要分片/reassembly/credits由H3唯一实现，H1提供固定channel/endpoint与受控出口。不能静默截断/提高旧limit/将片段当public对象，也不能把一个不可容纳的完整attempt偷偷分成不同任务。重组保持原06/07字节/hash与原AttemptId，十五reader不能无界fanout挤满02预算。

H3按已移交职责冻结必要的bounded carrier/fragment/credits格式；每片走H1受控出口，完整对象的hash/长度/连续覆盖成立后才交reader，缺片WAIT/冲突REJECT/重复幂等。H1不另实现分片codec。真实proof/body沿现有GFR public object边界，`q_avail`由H3复用07完整性算法按真实前缀重算，不能用H1 delivery time/submit ACK替代。

同一主场景真实 publication bytes 的 artifact 写明 ResultKey、来源 grant/root、对象hash/native证明；H3交16官方本地 dev-node提交/读回 q/index/namespace/payload。官方 wall-clock、PE时间 Unknown/Unsupported 与确定性 SimEspresso trace分开；HTTP读回不能声称 QC/VID 或完整PE时序通过。

## 5. 运行后未知 cid：已决定 `P-TRANSPORT-A`

### 5.1 已核对的当前边界

02 `config.rs::{TransportConfig,EndpointRegistration,ChannelPermit}` 将 contexts 固定在 normalized config；`route.rs::RouteCommand` 只有 Send/Drain/Rebind，Rebind只更新 incarnation。`RouteKernel::commit` 与 `from_snapshot` 要求原 config hash 一致。未来 cid 包含真实尚未产生的 anchor，不能运行前预装所有 cid；用新空 kernel替换会丢旧 queue/FIFO/send ordinal/dedupe/custody/history。H1也不能把完整范围缩回 R0。

### 5.2 已选方案 A：kernel与mailbox同revision增量准入

总协调已移交最小sim-transport扩展。`SIMT-CONTEXT-ADMISSION-1`保持immutable base config/hash，增加单调`admission_revision/admission_state_hash`及新snapshot/schema；旧v1语义不重解释。**RouteKernel与所有受影响ReliableMailbox在同一受控root提交内安装相同revision/准入事实**；任何一方preflight失败都不得释放部分权限。提交后的故障保留durable事实/待办并fail-closed，不回退为不同revision继续发送。复用既有plan/commit/FIFO/custody算法，精确字段和向量在transport切片冻结。

```text
ContextAdmissionPlan = plan_admit_context(
    StepStamp, expected_admission_revision,
    cid: Cid, descriptor_ref, verified_preparation_basis_ref,
    exact endpoint/channel send/receive grants)
commit_admission(plan, actual_durable_receipt) -> admission_revision, events
```

上述函数名仍是transport切片待冻结API。kernel不替Manager验Plan，不接受Hub脚本的allow布尔值；adapter先校验真实Create/Prepare来源及descriptor再准入。votes/certs/resolver限定五成员；跨区final/body/state能力按角色区分，不给下一区域旧cid签名能力。真实已验证descriptor可在Prepare阶段安装数据能力；签名仍独立受Enter/Running/Clock/terminal守卫。未知未来anchor/cid不得预装。

安装原子且幂等：相同cid/相同规则重复无新权利；错descriptor/config、相同cid不同权限、未来revision或stale base原子拒绝。未知cid的native frame不得先发再靠晚安装追认；保留有界等待/重新发送原义务。更新保留所有旧route queues、FIFOfloors、packet/message/attempt counters、fault ordinals、**每个可靠mailbox的outbox/inbox/consume/custody及未释放receipt**、scheduled timers与root链；不能用新mailbox或新kernel清空历史，旧frames不改context。

Close停止生产但不删历史cid的数据权限/持久义务。若需撤权，另分“新共识发送”与“已持久历史proof/body交付”，独立审计；M1可采用仅增准入+本地终止守卫，不能以清空权限取消已承诺交付。测试覆盖H1-20/21和有旧pending队列时安装新cid的字节/ordinal不变。

### 5.3 方案 B（历史备选，已不采用）

现有kernel使用 `ContextPolicy::Forbidden` 的固定正常载体channel，公开cid完整放在 normal typed frame；H1在每个受控发送/接收出口维护持久 `NormalContextRegistry` 并验证以上准入，kernel继续只验证固定 endpoint/channel。无 wildcard native cid、无 future cid预装、无kernel重建。

该历史备选改变cid权限执行位置；正式派发已选A，Builder不得改走B或混用。此段只保留讨论来源，不是当前实现选项。

## 6. wire、共享能力和决议清单

`H1-WIRE-1` 提案以独立 `rotation.normal.v1` package承载：

| 结构 | 必须绑定的内容 |
| --- | --- |
| Register | schema/profile/run/public_config/registry/transport_config；pid/region/member/signer映射，incarnation=1，clock domain；不预绑定未来所有descriptor |
| Grant | 完整input bytes+hash，step/event/tick/causal-wave，worker/expected runtime revision；Sample/Timer、Create/Prepare/Supply/Cancel/Enter/Pause/SelectBack/Close、transport、work completion、public reader/lifecycle inputs |
| Ack | 原grant/hash、全部effects、实际root sequence/hash、reader/lifecycle/preparation/admission/terminal/custody记录引用；`NativeEvidence=NotApplicable | Receipts(per-cid policy/artifact/ledger refs)`，不得用零或伪造receipt表示适用见证 |
| Effects | 完整send/frame、timer/work更新、签名audit、proof observation、verified delivery、lifecycle/Ready观察、transport准入候选；全部在root提交后释放 |

root journal必须保存实际新Grant/Ack完整字节，不能仅投影成旧native.proto遗漏Prepare/Pause/reader/public事件。hash canonical属于控制wire并需独立冻结；不替代03公共codec。一次Ack全验证/持久后安装；重复原grant精确返回原Ack，变化重放拒绝。

| Q | 需冻结的具体决定、owner 与门禁 |
| --- | --- |
| P-M0 | 已交付`35d382ac445cc8bc779f0d18fd23cb57e54680da`并合入H1基线4741489；现有组合检查不因仅文档/API新增重复，shared行为改变后复验适用Fresh。 |
| Q-LIFE | H1/H2确认CommandContext/ProgramHandle及operation幂等、LifecycleBatch原子顺序/持久receipt；H2纯Manager，H1runtime执行，07事件codec不变。 |
| Q-PREP | H1/H2确认StateMaterial/ReadyToken/job generation、exact provenance/WAIT/cancel语义、03输入引用及资源界；禁止Ready bool/仅root。 |
| Q-RESULT | H1/H3确认§4的原生证明profile/公开FinalVerifier构造与basis、目标未到时的proof observation、完整RegionalResult及消费Ack；03 trait和07 publication codec保持唯一，明确历史配置缺失WAIT与错误证明REJECT。 |
| Q-RUNTIME | `driver.rs/context.rs`最小扩展已移交H1；实现前按切片列cid停车、单Clock、多witness/partition、late安装的精确guard/diff/独立预期与适用Fresh回归，不复制driver/runtime。 |
| P-TRANSPORT-A | A已选、sim-transport扩展正式移交H1；kernel与mailbox同revision原子准入。transport切片再冻snapshot/receipt/字段/向量，API切片不改其实现。 |
| Q-DOMAIN | 正常verifier版本/namespace/epoch及黄金bytes；03 PoP匹配，engine/witness/公开verifier共用来源；不得换同cid verifier。 |
| Q-ADMIT | 已按[Q-ADMIT-v1路线A](admission-bridge-v1.md)冻结来源映射：notarize-only本地准入，certify执行确定性V+完整持久数据；producer、真实取消、提议承诺、parent/subject绑定及条件活性有独立来源依据。无新增vendor hook，实际native实现/重放仍待独立验收，不能用API测试代替。 |
| Q-INGRESS | 首收身份/key、header/body配对、forwarded证明与LOCAL_BUILD、认证队列/预算和永久结论顺序；独立黄金时序先于代码。 |
| Q-WIRE | schema、proto字段、完整Grant/Ack/root记录版本/hash、NotApplicable/多cidreceipt、errors和bounds；原型build/lib/manifest/lock由lead串行安装。 |
| P-SHARED | d55已正式移交scheme/witness/journal/driver/context最小normal扩展与聚焦测试，单writer按切片修改，旧Fresh语义保留；授权不含01恢复WIP/quorum/lock/vote或全局callback变更。 |
| Q-FRAGMENT | owner已定H3；H3冻结64KiB message/packet、1MiB block下真实M1编码界与必要carrier/credits，H1给受控出口，07公开字节唯一。 |
| Q-PARAM | view/section/retention、timer/WAIT/队列/认证/处理预算、逐worker offset/Enter schedule、未知资源与前提失败的分类；五引擎时限实测后才能作进展声明。 |

三组最小接口release需精确API提交、contract版本、03/07兼容向量与独立review。结构/语义按d55已决项执行，普通字段/错误/局部ID由本owner在API切片冻结；未决native映射不阻API发布。H3有界小payload优先：02每包≤64KiB、06每message≤64KiB、block≤1MiB；必要分片/重组/credits由H3独占，H1提供受控出口，完整attempt真实编码上界需核算。
