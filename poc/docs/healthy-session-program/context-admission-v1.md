# SIMT-CONTEXT-ADMISSION-1：增量 cid 准入切片

2026-11-16。状态：**REVIEWABLE FREEZE DRAFT；实现/测试 NOT_RUN**。这是可独立实现的纯运输组件切片，不包含 normal RPC wire、跨进程投影、真实 fsync 或 BFT engine。局部字段、编码和 guards 在本稿固定；独立 Verifier 先形成预期，lead 冻结后交单 Builder。API 切片和本合同可以分别交付，不等待 Q-ADMIT。

文档 worktree：`/home/jy/.codex/worktrees/h1-context-admission-docs/megaroation_poc`，由 root 的 managed 工具创建；基线 `2ea612a16fb4620e5faf7191b7b9c4f32dd7b07b`，分支 `codex/h1-context-admission-docs`。Docer 只写本文件、不 build、不改代码；保留主 worktree 的 API Builder 独占写入。此文档合入前不 archive 该 worktree。

## 1. 权威、来源规则与工程决定

正式授权为 `d55d39f73593676325bdc0fbb7ba3845ca93025f` 的 `poc/docs/milestones/M1-implementation-dispatch-20261116.md`：已选增量方案 A，H1 单独拥有 sim-transport；RouteKernel 与所有相关 ReliableMailbox 同一受控提交安装同一 admission revision。M0 为 `35d382ac445cc8bc779f0d18fd23cb57e54680da`。root 本次进一步确认中央 transport/custody authority 与 worker 本地投影分层，不能声称跨十五个磁盘发生一次原子事务。

| 标签 | 固定来源与节 | 本切片采用的约束 |
| --- | --- | --- |
| F-CONTEXT | v10 [Session Program](../../../formalization_v10/02_session_program_zh_v0_1.md) §§3.1–3.6；[Manager](../../../formalization_v10/03_session_manager_zh_v0_1.md) §§3.7–3.10、4.1–4.3 | cid 由固定 descriptor/anchor 决定；Create/Prepare 的真实来源可先于 Ready；Enter/签名/永久终止独立；同 cid 原历史保留。 |
| F-CONTROL | [v2](../../../formalization_v10/poc_design_zh_v2.md) §§4、5.1–5.3、6 | 单时间权威；全部可观察输出经 grant/Ack；RPC、local、virtual、public、wall 分开；不重启/暂停不清持久义务。 |
| F-CUSTODY | [SIM-TRANSPORT-KERNEL v1](../contracts/task-contract-sim-transport-kernel-v1.md) §§2–6；[API v1](../sim-transport-api-v1.md) | 真实保管后才 ACK，业务消费与 MarkConsumed 的明确边界；私有 instance token、revision、计划/提交分离；旧稳定 message/attempt/receipt 与规范 JSON。 |
| F-EVIDENCE | Regional BFT pin `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，DESIGN §§2.1、3.1、5；Commonware `2026.10.0` / `92a2f68a0e131c8b69de41e84aec37da6280f212` | 进展证明、finality 与完整交付不同；运输准入和 custody ACK 都不是 BFT 证据。此次不改 quorum/subject/vote/lock 或 Commonware。 |
| P-A | d55 已决项1–5及本次 root 接线指派 | fixed base config；add-only context catalog；中央 authority 本地原子 batch；worker 各自 root 投影；没有全员 Ready 屏障。 |
| P-PURE | 本稿 §§3–8 | 下列 API/JSON/schema/limits 是 adapter 选择，不是论文新增规则。kernel 只比较 opaque Id；03 继续唯一拥有公共 cid/Descriptor 编码、registry/PoP。 |

沿用 `regional-bft-conformance` 的触发/前态/guard/output 与负向边界框架。此前已按根 AGENTS 读取 handoff/setup/v2、SP/SM/GFR、固定 BFT DESIGN 和上游；本 worktree 再核对 AGENTS/DESIGN/skill 后只读具体代码。`protocol-spec-writer` 当前不可用。本切片不研究 staking/TLA/Force/恢复，所有旧完整故障 gate 仍 OPEN。

## 2. 实际旧边界与最小改动

固定基线的 `config.rs::ChannelPermit.contexts` 静态；`RouteCommand` 只有 Send/Drain/Rebind，Rebind 只改 incarnation。`RouteKernel` 与 `ReliableMailbox` 各自持有配置并调用其 check_header/check_message；只更新 kernel 会让可靠 mailbox 拒绝新 cid。`normalize_and_expand` 按现有 endpoint/channel 权限生成 route topology，不依赖 contexts 内容，因此本切片只增加已有 endpoint/channel 的 context 集合，不新增 links 或重算 link ordinal。

`route.rs::RoutePlan` 和 `reliable.rs::CustodyPlan` 已捕获私有 Instance 与 component revision。`RouteSnapshot` 保存 queues/FIFO/drain/attempts/bindings；`MailboxSnapshot` 保存 outbox/inbox、first receive、consumed、retry 与 tombstone；未释放的 `CommittedCustody.releases` 还由外层持久 owner 保存，不能只比较 mailbox snapshot 就宣布没有待办。

实现 allowlist 为 `poc/crates/sim-transport/` 内新增 `src/admission.rs`、必要的 `config.rs/route.rs/reliable.rs/snapshot.rs/lib.rs` 私有接线和该 crate 的聚焦测试；复用现有 hash/codec/route/custody 算法，不改根 manifest、types、proto、normal runtime、API crate、vendor、子模块或其他功能。现有静态 v1 构造器、调用语义、输入/消息/ACK 字节、goldens 必须完全不变。只有显式调用本稿准入 API 并成功 confirm 后，对应对象才进入 opt-in admitted 模式。

## 3. 真实进程责任与能力安装

### 3.1 一个中央 custody authority，多个独立 worker root

中央 transport/custody authority 的一个 service root 持有 canonical RouteKernel、全部登记逻辑 owner 的 canonical ReliableMailbox、准入 catalog 和持久 release ledger。它不是某个 replica 的 BFT 实例；它不持有 replica 私钥、不代签。库的原子 confirm 只作用于此 authority 进程内的这些对象。内部可共享不可变 Arc，但 Arc 不是跨进程实现或持久证据。

每个 worker 有自己的 root owner，保存本地输入/输出义务、cid/engine/签名历史、已实际安装的 admission 投影。worker 不再放第二份竞争的 canonical ReliableMailbox；服务端 canonical custody 与 worker 的业务输入/输出任务是不同层。纯库切片不实现这些 worker 任务和 wire，但下一切片必须采用本节的责任与提交顺序。

| 阶段 | 谁持久、谁可输出 | 明确不代表什么 |
| --- | --- | --- |
| 验证准入来源 | H1 调用端用03重算 descriptor/cid，核对实际 Create/Prepare 的 anchor/provenance，保存验证依据并形成 authorization_ref | kernel 不验 Plan/PoP，不接收 bool Ready 当授权；Hub 不预装未来 anchor/cid。 |
| authority 安装 | authority service root 持久完整 admission record，再原子 confirm kernel+全部 canonical mailbox；生成有原 transition ID 的 ProjectionIntent | 不表示任何 worker 已安装或 Ready，也不触发新签名。 |
| worker 投影 | 相应 worker 在受控 grant 验证 authority 记录/连续 catalog，持久投影与原生调度许可后，用本 worker 完整 Ack 给 ProjectionApplied 证据 | 仅数据能力，不表示 execution Ready/Enter/Selected；不能用服务 root 冒充 worker root。 |
| 新 cid 发送/协议交付 | authority 已 commit，参与此动作的 worker 投影已 root 确认；再允许对应新 context 的协议调度/发送或向 worker 交付 | 无关 worker 不构成屏障；state-fetch 不等 Ready，避免准备死锁；签名仍独立受 Enter/Clock/lease/terminal/native guard。 |
| 旧 context | 继续使用其原准入记录；投影只需覆盖该 context 的 installed_revision，不要求等于最新 global revision | 新 context 更新不暂停旧队列或要求十五 worker 全部追平。 |

新 context 的 receive **custody** 可以先在 authority 的 canonical inbox 保管；在目标 worker 投影完成前不能据此调度其协议 callback。其 ReceiveAck 仅证明 authority 为该逻辑 endpoint 持久保管，不证明 worker 已收到/执行/Ready。该 ACK 也必须纳入 authority 的受控 root/Ack。worker application receipt 是独立后续事实。

worker 出站 root 先保存原消息/输出义务；authority 接受并持久 canonical sender outbox 后给 transfer receipt，worker 才能把本地义务记为已转交。authority 的 MarkConsumed 必须与本 service root 内的持久 `WorkerDeliveryTask`/转交义务共同提交；该任务只有取得实际 worker root 输入接收回执才完成。它不把远端应用执行与中央 MarkConsumed 说成一个磁盘事务，也不宣称任意业务 exactly-once。首次 authority custody 时刻是运输事实；SP Accept 的 first_received 仍由实际 worker ingress/本地 Clock 保存，不能复制 service 时间冒充。

### 3.2 投影格式责任（下一切片，不在本次实现）

ProjectionIntent 提供 endpoint binding、目标 stamp/context、稳定 projection_id；实际 normal wire 还必须绑定 run/base-config、完整 catalog 增量前缀、authority transition/record/root receipt、receiver incarnation 和 controlled origin。worker 返回的 ProjectionApplied 至少绑定同一 projection_id/transition/stamp/endpoint 和其真实 root receipt。authority 保存每 endpoint 已确认的安装位置，未知或 gap 保持 WAIT，不靠共享内存更新。

一个 worker 可以在一个 grant 安装已验证的多条历史 catalog 前缀，但不得跳过验证或预知未来 cid。丢失/重复投影按原 projection_id 重放，既有 worker terminal/签名历史不改变。endpoint 当前 incarnation 与投影不匹配拒绝；本健康profile保持 incarnation=1。首次正式 wire/真实 fsync 接线须独立冻结并验证，本纯组件不能关闭它们的 gate。

## 4. 精确值类型与 API

除包含私有plan的AdmissionPlanResult外，全部新公开 value types 派生 Clone/Debug/PartialEq/Eq/Serialize/Deserialize；struct `deny_unknown_fields`，字段顺序如下，Endpoint/ChannelId/Id/StepStamp/既有 receipt 类型直接复用。ContextAdmissionPlan和AdmissionPlanResult只需Debug，不Clone、不Serialize/Deserialize，不暴露可变 candidate/token。无需新增依赖。

```rust
pub struct ContextGrant {
    pub endpoint: Endpoint,
    pub send: Vec<ChannelId>,
    pub receive: Vec<ChannelId>,
}
pub struct ContextAdmission {
    pub context: Id,
    pub authorization_ref: Id,
    pub grants: Vec<ContextGrant>,
}
pub struct AdmissionStamp { pub revision: u64, pub state_hash: Id }
pub struct AdmissionEntry {
    pub revision: u64,
    pub transition_id: Id,
    pub request: ContextAdmission,
}
pub struct AdmissionCatalog {
    pub schema: u32,                 // 1, new catalog's own schema
    pub authority: Id,
    pub base_config_hash: Id,
    pub revision: u64,
    pub entries: Vec<AdmissionEntry>,
}
pub struct PreparedCustodyRef { pub transition_id: Id, pub record_hash: Id }
pub struct PendingReleaseRef {
    pub transition_id: Id,
    pub record_hash: Id,
    pub storage_receipt: Id,
    pub releases_hash: Id,
    pub remaining_indices: Vec<u32>,
}
pub struct OwnerCustodyCut {
    pub owner: Endpoint,
    pub revision: u64,
    pub prepared: Vec<PreparedCustodyRef>,
    pub unreleased: Vec<PendingReleaseRef>,
}
pub struct AdmissionCustodyCut {
    pub schema: u32,                 // 1
    pub authority: Id,
    pub service_root: Id,            // actual durable parent root, not self hash
    pub owners: Vec<OwnerCustodyCut>,
}
pub struct MailboxRevision { pub owner: Endpoint, pub revision: u64 }
pub struct ProjectionIntent {
    pub endpoint: EndpointBinding,
    pub stamp: AdmissionStamp,
    pub context: Id,
    pub projection_id: Id,
}
pub struct AdmissionDurabilityReceipt {
    pub authority: Id,
    pub base_stamp: AdmissionStamp,
    pub transition_id: Id,
    pub candidate_record_hash: Id,
    pub parent_service_root: Id,
    pub storage_receipt: Id,
}
pub struct AdmissionCommitted {
    pub receipt: AdmissionDurabilityReceipt,
    pub stamp: AdmissionStamp,
    pub route_revision: u64,
    pub mailbox_revisions: Vec<MailboxRevision>,
    pub projection_intents: Vec<ProjectionIntent>,
    pub carried_custody_cut: AdmissionCustodyCut,
}
pub enum AdmissionPlanResult {
    AlreadyAdmitted { installed_revision: u64, transition_id: Id,
                      current: AdmissionStamp },
    Planned(ContextAdmissionPlan),
}
pub enum AdmissionError {
    Transport(Error),
    InvalidAuthority,
    InvalidGrantSet,
    StaticContext,
    ContextConflict,
    OwnerSetMismatch,
    CustodyBusy(Endpoint),
    CustodyCutMismatch,
    AdmissionMismatch,
}

impl ContextAdmission {
    pub fn normalize(self) -> Result<Self, AdmissionError>;
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, AdmissionError>;
}
impl AdmissionCatalog {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, AdmissionError>;
    pub fn stamp(&self) -> Result<AdmissionStamp, AdmissionError>;
}
impl AdmissionCustodyCut {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, AdmissionError>;
    pub fn hash(&self) -> Result<Id, AdmissionError>;
}
impl ContextAdmissionPlan {
    pub fn transition_id(&self) -> Id;
    pub fn candidate_record_hash(&self) -> Id;
    pub fn persistence_bytes(&self) -> &[u8];
    pub fn next_stamp(&self) -> AdmissionStamp;
}
impl RouteKernel {
    pub fn admission_stamp(&self) -> Option<AdmissionStamp>;
    pub fn from_admitted_snapshot(config: TransportConfig, bytes: &[u8])
        -> Result<Self, AdmissionError>;
}
impl ReliableMailbox {
    pub fn owner(&self) -> Endpoint;
    pub fn admission_stamp(&self) -> Option<AdmissionStamp>;
    pub fn restore_admitted(config: TransportConfig, record: &[u8],
                            receipt: DurabilityReceipt)
        -> Result<(Self, CommittedCustody), AdmissionError>;
}
pub fn initial_admission_stamp(authority: Id, config: &TransportConfig)
    -> Result<AdmissionStamp, AdmissionError>;
pub fn plan_context_admission(
    authority: Id, kernel: &RouteKernel, mailboxes: &[&ReliableMailbox],
    step: StepStamp, expected: AdmissionStamp, request: ContextAdmission,
    custody_cut: AdmissionCustodyCut,
) -> Result<AdmissionPlanResult, AdmissionError>;
pub fn confirm_context_admission(
    kernel: &mut RouteKernel, mailboxes: &mut [&mut ReliableMailbox],
    plan: ContextAdmissionPlan, current_parent_cut: &AdmissionCustodyCut,
    receipt: AdmissionDurabilityReceipt,
) -> Result<AdmissionCommitted, AdmissionError>;
pub fn restore_context_admission(
    config: TransportConfig, record: &[u8], receipt: AdmissionDurabilityReceipt,
) -> Result<(RouteKernel, Vec<ReliableMailbox>, AdmissionCommitted), AdmissionError>;
```

AdmissionError implements Display/Error/From<Error>；可编码enum沿既有`kind/value`、snake_case。Catalog/Cut自身的canonical_bytes只校验内在结构/排序/bounds；与base权限、真实component revision或实际持久来源的对应由plan/confirm及真实adapter各自验证，不能由编码成功推出授权。其余错误复用 Transport(Error::CrossInstance/StalePlan/InvalidReceipt/ResourceLimit/ArithmeticOverflow/NonCanonical/CorruptSnapshot/ConfigMismatch/BindingMismatch/StepRegression/TimeRegression)。不更改原Error判定或静态v1输出。

## 5. Plan/confirm guards 与状态转移

### 5.1 固定配置、完整 owner 集合、add-only

base config是原normalize结果，`kernel.config_hash()`始终为原`SIMT/config/v1` hash；run/config/seed/limits/rules/endpoint binding初值/channel配置均固定。有效 context权限是base集合与已提交catalog的并集；可缓存这份有效视图，但不得把其hash改名为base hash。仅在已有send/receive channel permit上添加非零opaque context，channel必须Required；Forbidden channel、未知endpoint/channel、新增channel权限/links一律拒绝。

ContextAdmission.normalize按endpoint排序，每行send/receive按channel排序；重复行/重复channel拒绝，不静默去重；grants非空，每行至少一个方向非空，context/authorization_ref均非零。context已出现在base任一静态permit时返回StaticContext；本版本不扩张一个既有静态cid。动态同cid的完整request不可变：相同规范request返回AlreadyAdmitted；不同grant或authorization_ref返回ContextConflict。调用端对合法proof变体应复用首次已验证的稳定授权依据引用，而不是让kernel猜其等价。

canonical mailbox owner集合精确等于base中有任一Reliable send/receive channel permit的endpoints，与其目前contexts是否为空无关。入参mailboxes必须按Endpoint严格排序、无遗漏/重复/额外owner；全部base配置一致，owner当前incarnation须等于kernel对应binding。即使本次grant不涉及某个owner，也一起安装stamp并保留其完整state，以免随后被误当旧权限副本。BestEffort-only endpoint无需mailbox。

初始全static对象的admission_stamp为None；本函数把它们的预期视作authority+base绑定的revision0空catalog。成功confirm才进入admitted模式。已有admitted对象必须全体catalog字节/stamp一致，混合None/Some、不同authority/hash或不同catalog原子拒绝。ordinal/revision溢出拒绝。

### 5.2 创建纯候选

先验证base/owner/catalog/request/cut结构，再核对expected属于当前catalog的正确历史前缀：future revision或错误prefix hash拒绝。若完全相同request已在catalog，返回AlreadyAdmitted，不改任何revision/step，不生成新projection；重发工作从原transition的持久记录恢复。否则expected必须等于current stamp。

CustodyCut必须包含精确owner集合，authority一致、service_root非零、每row.revision等于对应mailbox现revision；pending引用非零且规范排序。任何row.prepared非空返回CustodyBusy(owner)：已持久但尚未confirm的旧计划必须先完成或走明确恢复，不允许在它前面安装新准入。unreleased允许非空，完整原引用/receipt/剩余release索引进入候选记录，不可改写为新receipt或重算原ACK。

step满足原RouteKernel规则，并且tick不小于任何mailbox.last_action_tick。纯候选通过现有空route plan的等价逻辑把route.revision+1和last_step=step；**不Drain、不Send**。所有mailbox.revision各+1，last_action_tick、first receive、retry_due和全部业务state不变。新增一条catalog entry（revision=current+1），overlay权限更新到所有组件。旧links、FIFO顺序/drain credit/send ordinal、packet_seq、last attempt、bindings、outbox/inbox/payload/consumed/tombstone逐字段保留；仅声明过的revision/last_step/admission字段变化。

ContextAdmissionPlan捕获kernel及每mailbox的私有Instance token、base component revisions、完整旧catalog、custody_cut及预生成候选/序列化record/release intents。它不是可外部构造的serde值。plan不修改任何组件、不释放网络/ACK/投影；所有可能失败的候选分配、编码与容量检查在plan或confirm预检完成。

### 5.3 全量预检后一次安装

authority实际持久完整record后，提供AdmissionDurabilityReceipt。确认依次检查：同一个kernel/每个owner的instance；精确owner集合/base配置/catalog；全部component revisions仍等于捕获值；current_parent_cut与plan.cut字节相同；receipt authority/base_stamp/transition_id/record_hash/parent_service_root精确匹配且storage_receipt非零。任何错误发生在所有state替换之前，候选全部不安装。

检查全部成功后，预先准备好AdmissionCommitted输出，再用不可失败的字段move安装kernel与全部mailboxes；中间不回调、不I/O、不产Send/ACK。结果里的ProjectionIntent仍只是受控输出意图，必须由authority完整root/Ack释放。发生不可恢复panic/基础设施错误则停止authority，不允许某些组件继续旧权限运行。

该库只检查typed receipt的绑定，不验证fsync/root认证；测试receipt只证明接口与纯状态原子性。真实authority必须串行独占从plan→持久→confirm→输出纳入root的区间。`current_parent_cut`是这次提交的原parent事务视图，新的storage_receipt在其上追加；不是把包含自身的root hash写回record形成循环。

### 5.4 旧plan与旧持久义务

成功准入使所有component revision增加，所以此前未持久RoutePlan/CustodyPlan正常返回StalePlan；caller可丢弃这些**未持久**候选，再用原业务意图重plan。clone/restore生成新Instance，旧plan返回CrossInstance。若某计划已经持久，不能凭同样错误把它当无义务失败；controller保存其原record/receipt并fail-closed。真实集成必须证明cut.prepared如实反映此状态，不能为了通过传空表。

已经confirm但尚未释放的Send/ReceiveAck/InboxAvailable等由service root的release ledger保留。PendingReleaseRef绑定原完整PersistenceRecord、原storage receipt、原CommittedCustody.releases hash及剩余索引；admission只携带引用，先前原字节必须仍从cut.service_root可达。confirm/restore返回原carried cut，不能生成新message ID、attempt、首收或ACK。完成某项release改变cut，若与待confirm admission交错，必须在持久前重新plan；持久后出现这种交错是controller违约，fail-closed而非丢义务。

库无法认证caller提供的cut是否是磁盘真相，正如旧DurabilityReceipt不证明fsync。这一信任边界和后续真实root检查分别记录，不能把纯库成功说成生产事务通过。

## 6. 编码、hash、容量和snapshot兼容

### 6.1 唯一控制编码

复用现有compact serde_json：struct声明顺序、bytes为整数数组、Endpoint/enum沿v1 tag；无map/float/unknown fields，decode后重编码字节必须一致。normalize只用于新request/config构造；持久decode拒绝未规范排序而不是暗中修复。

Catalog.schema=1；entries按revision连续1..N，revision=N，context唯一；grant规范化。空catalog revision0。CustodyCut.schema=1，owners按Endpoint排序，prepared/unreleased各按transition_id严格排序，remaining_indices非空且严格递增。PendingReleaseRef.releases_hash定义为既有framed_hash(`SIMT/custody-releases/v1`, `[canonical Vec<CustodyRelease> bytes]`)；库验证引用结构，真实服务核对可达原列表与索引范围。历史服务已采用另一release-ledger hash时，由adapter新增此审计投影，不改变原记录/receipt。

所有H仍为既有`framed_hash`；逗号分隔项是独立length-framed part：

| 值 | domain与parts |
| --- | --- |
| base config hash | 原`SIMT/config/v1`, normalized base JSON |
| catalog state hash | `SIMT/admission-state/v1`,完整Catalog JSON；hash不在该JSON内 |
| request审计hash | `SIMT/admission-request/v1`,规范ContextAdmission JSON |
| custody cut hash | `SIMT/admission-custody-cut/v1`,完整CustodyCut JSON |
| admission transition | `SIMT/admission-transition/v1`, authority,base_config_hash,base_admission_revision-u64BE,base_state_hash,route_base_revision-u64BE,canonical StepStamp JSON,canonical request JSON,custody_cut_hash |
| candidate record hash | `SIMT/admission-record/v1`,下述完整AdmissionRecord JSON |
| projection_id | `SIMT/admission-projection/v1`,transition_id,原Endpoint(tag-u8+id-u32BE) |

next catalog entry包含新transition_id，但transition preimage只依赖旧catalog/输入/cut，所以无循环。new stamp由next catalog hash计算；ProjectionIntent随后计算；record含这些已知值但不含自身record hash或外部storage_receipt。

AdmissionRecord为私有serde结构，字段顺序精确：`schema(2), authority, base_config_hash, step, base_stamp, transition_id, request, custody_cut, next_catalog, route_base_revision, mailbox_bases, route_candidate, mailbox_candidates, projection_intents`。mailbox_bases是sorted `MailboxRevision`；route_candidate/mailbox_candidates分别是原v1 RouteSnapshot/MailboxSnapshot的完整字段结构（schema仍1），它们在outer schema2与next_catalog下解释，不可单独当静态snapshot恢复。projection_intents按request.grants的endpoint排序，仅列这些endpoint，使用其当前binding；不会等待无关endpoint。

### 6.2 明确opt-in v2，静态v1 exact不变

- 旧`RouteKernel::new`、`ReliableMailbox::new`初始状态、旧静态plan/confirm/snapshot/restore及全部goldens不变；原SCHEMA=1不全局改成2。旧restore入口仍拒绝schema2。
- admitted kernel的`RouteKernel::snapshot_bytes`与`RoutePlan::candidate_snapshot_bytes`输出新outer `{schema:2,catalog:AdmissionCatalog,route:RouteSnapshotV1}`；`from_admitted_snapshot`只读此格式，验证base hash、catalog和effective权限后恢复。静态时仍输出原v1字节。
- admitted mailbox普通操作继续使用原MailboxAction/Message/ReceiveAck和原`SIMT/custody-transition/v1`公式；其`CustodyPlan.persistence_bytes`输出 `{schema:2,catalog:AdmissionCatalog,record:PersistenceRecordV1}`，candidate_record_hash用`SIMT/custody-record/v2`。原DurabilityReceipt字段不变但绑定此实际外层hash；`restore_admitted`只读这个明确格式。静态record继续v1格式/domain。
- base snapshot中的kernel_config_hash始终固定base hash；effective context视图只用于权限验证，不拿effective-config hash替换它。验证旧message/ACK时使用add-only并集，不改任何原签名payload/message ID/attempt/origin/custody bytes。
- `restore_context_admission`校验完整record、receipt、旧catalog前缀、最新transition重算、候选revision与完整owner集合、权限/容量/排序；返回新Instance的kernel/mailboxes、相同projection IDs及carried cut。它不执行文件操作、不自动释放，也不证明trusted floor/跨PID恢复。
- 不提供把admitted状态剥成v1的降级导出，不接受“重建空kernel/mailbox再导入权限”。Close不删catalog或旧context权限；停止新签名由worker terminal/Clock守卫执行。权限撤销/拓扑改变不是本切片。

### 6.3 有界资源

沿用v1 endpoint≤64、channel≤32及全部queue/payload/mailbox上限；同一context grant只引用已登记channel。新catalog最多128 entries；base+dynamic总`(endpoint,direction,channel,context)` membership cells≤4096（按两方向分别计数），与base normalize现有上界一致。每个cut owner的prepared+unreleased引用≤1024，全cut≤8192，每个remaining_indices≤8192。所有cut/catalog/单component snapshot/整个admission record均≤32MiB，plan确认前检查；没有截断/GC/逐出旧待办来给准入腾空间。达到界限返回ResourceLimit，旧权限/义务不变。M1小数据实际容量预算在normal接线另核，不用此组件边界宣称15路fanout已通过。

## 7. 完整独立golden输入

本节是Docer按规范JSON/hash编写的独立输入生成式，未调用Rust被测实现、未运行build。Verifier应独立重算封存。以下完整Python只构造规范值；J输出就是golden输入bytes，可写入专属verification路径，**不要导入sim_transport实现来产生expected**。

```python
import json, hashlib
from struct import pack
J=lambda x:json.dumps(x,separators=(',',':'),ensure_ascii=False).encode()
def H(d,*p):
    return list(hashlib.sha256(b''.join(pack('>Q',len(z))+z
                for z in [d.encode(),*p])).digest())
B=lambda x:bytes(x)
E=lambda n:{'kind':'replica','value':n}
U=lambda n:{'endpoint':E(n),'incarnation':1}
I=lambda x:[x]*32
lim={'endpoints':64,'channels':32,'rules':1024,'payload_bytes':65536,
     'link_copies':64,'queued_copies':1024,'queued_payload_bytes':4194304,
     'tracked_messages':8192}
profile={'latency_us':20000,'jitter_us':0,'order':{'kind':'fifo'},
         'offline':[],'offline_mode':{'kind':'hold'},'capacity':64,
         'drain_bytes_per_second':1000000,'faults':[]}
permit={'channel':7,'contexts':[I(0x33)]}
base={'schema':1,'run':I(0x11),'config':I(0x22),'seed':7,'limits':lim,
      'endpoints':[{'binding':U(n),'send':[permit],'receive':[permit]}
                   for n in [0,1]],
      'channels':[{'id':7,'context':{'kind':'required'},
                   'reliability':{'kind':'reliable'}}],
      'rules':[{'source':{'kind':'any'},'destination':{'kind':'any'},
                'channel':{'kind':'exact','value':7},'profile':profile}]}
bh=H('SIMT/config/v1',J(base))
cat0={'schema':1,'authority':I(0xaa),'base_config_hash':bh,'revision':0,'entries':[]}
s0={'revision':0,'state_hash':H('SIMT/admission-state/v1',J(cat0))}
req={'context':I(0x55),'authorization_ref':I(0xcc),
     'grants':[{'endpoint':E(n),'send':[7],'receive':[7]} for n in [0,1]]}
cut={'schema':1,'authority':I(0xaa),'service_root':I(0xbb),
     'owners':[{'owner':E(n),'revision':0,'prepared':[],'unreleased':[]}
                for n in [0,1]]}
ch=H('SIMT/admission-custody-cut/v1',J(cut))
step={'logical_step_id':1,'tick_us':1000000,'event_id':10,'causal_wave':0}
t=H('SIMT/admission-transition/v1',B(I(0xaa)),B(bh),pack('>Q',0),
    B(s0['state_hash']),pack('>Q',0),J(step),J(req),B(ch))
cat1={**cat0,'revision':1,'entries':[{'revision':1,'transition_id':t,'request':req}]}
s1={'revision':1,'state_hash':H('SIMT/admission-state/v1',J(cat1))}
route={'schema':1,'kernel_config_hash':bh,'revision':1,'last_step':step,
       'next_packet_seq':1,'bindings':[U(0),U(1)],'attempts':[],
       'links':[{'source':E(a),'destination':E(b),'channel':7,'send_ordinal':0,
                 'next_release_us':0,'queue':[]} for a,b in [(0,1),(1,0)]]}
rl={'outbox_records':1024,'inbox_records':1024,'retained_payload_bytes':4194304}
mb=[{'schema':1,'kernel_config_hash':bh,'owner':E(n),'incarnation':1,'revision':1,
     'limits':rl,'retry_interval_us':20000,'last_action_tick_us':0,
     'outbox':[],'inbox':[]} for n in [0,1]]
proj=[{'endpoint':U(n),'stamp':s1,'context':I(0x55),
       'projection_id':H('SIMT/admission-projection/v1',B(t),bytes([1])+pack('>I',n))}
       for n in [0,1]]
record={'schema':2,'authority':I(0xaa),'base_config_hash':bh,'step':step,
        'base_stamp':s0,'transition_id':t,'request':req,'custody_cut':cut,
        'next_catalog':cat1,'route_base_revision':0,
        'mailbox_bases':[{'owner':E(n),'revision':0} for n in [0,1]],
        'route_candidate':route,'mailbox_candidates':mb,'projection_intents':proj}
receipt={'authority':I(0xaa),'base_stamp':s0,'transition_id':t,
         'candidate_record_hash':H('SIMT/admission-record/v1',J(record)),
         'parent_service_root':I(0xbb),'storage_receipt':I(0xdd)}
```

0xaa/0xbb/0xcc/0xdd是**组件fixture opaque refs**，不冒充来源认证或磁盘receipt。基线两个logical endpoint只是纯组件向量，不是15 PID或BFT签名证据。

| 对象 | 精确JSON bytes长度 | 对应domain的hash hex |
| --- | --- | --- |
| J(base) | 1456 | `e1a6bd692ca773e424f0941a7f48ffe999edb8d212a24349c7fba7e63e1f60bb` |
| J(cat0) | 319 | `6161a6ef286f1bf2009bd75621314e44d935adfde0f2b18031f28a4f988588a3` |
| J(req) | 404 | `be60dbb9f3c0737745ccb25ef92d89ffdce51149cae9af2dc49eb86e029f5b6e` |
| J(cut) | 474 | `430b55e43b325862d20b49ecc216d39c8e8f8e8b9ca3ad2700786b3a00bfbfb2` |
| J(cat1) | 883 | `1c5fffcba807cc304b083cf97c915d79f3bf2b5e4156a0ba0a802eb67840f0cd` |
| J(record) | 5070 | `8a37e4399833a7a8dac7a8cb1995f601b396c6b337629a8dd6d30fe1dae0d5eb` |
| transition t | raw32 | `7bfdbaba6b4b0876ba6421c98e0191cb6214923c1fb7f22b70c6c0c1c88ff7f2` |
| projection Replica0 | raw32 | `967c74f5d4e7e5cf2460b311b46cfd90ab3d59092e7e9994f844eaa71590771f` |
| projection Replica1 | raw32 | `543bcd28973427ccca5372b8cc10828157adef0a979a9d78bbcdb645c59fcbed` |

## 8. 验收矩阵（PLANNED / NOT_RUN）

每例以独立输入/预期和完整前后snapshot/record检查；保存准确候选SHA、命令、seed或上述literal fixture和限制。纯组件receipt、proof-like payload均标fixture，不计真实root/验签/交付。继承v1已有全部goldens与独立harness，不重写其期望以适配新行为。

| ID | 触发/前态 | guard与输出断言 |
| --- | --- | --- |
| CA01 | 原静态v1构造/route/custody/clock/service/restore矩阵 | 全部原bytes/domain/error/golden不变；不调用新准入API时无schema2字段。 |
| CA02 | §7全套输入，unknown0x55先发送；plan后未confirm；正确confirm后发送 | 前两次ContextMismatch无变更；plan无输出；confirm后kernel+两mailboxes同stamp/revision，base hash不变；实际旧算法路由/可靠保管可用。 |
| CA03 | 部分owner缺失/重复/乱序、different base、incarnation错绑定、混合static/admitted或不同catalog | 全量拒绝，不部分安装；核对所有component前后bytes。 |
| CA04 | 错instance（clone/restore其中一个）、单一component mutation后旧plan、坏receipt/cut/root/recordhash | 全量预检失败；其他组件也无变化。无已persist事实时可重plan；fixture明确标persist后则记录fail-closed义务，不把它算安全丢弃。 |
| CA05 | 两个并行admission plan，从相同base确认一个，再确认另一个 | 第二个StalePlan/AdmissionMismatch，旧权限完整，新request需重plan；相同request重送AlreadyAdmitted不增rev/不重复projection。 |
| CA06 | 同cid不同grants/authref、static已有cid、zero IDs、Forbidden/unknown channel、endpoint越界、重复permission | 指定错误，base/queues/custody不变；kernel不自行验descriptor或接受wildcard。 |
| CA07 | 旧0x33已有多link queued帧/FIFO drain credit/fault ordinal/attempt；安装0x55 | 除revision/last_step/catalog外逐字段相等；下一次Drain/重试行为与独立旧模型一致，不重排/重分配packet ID。 |
| CA08 | 旧outbox active+ACK tombstone、inbox consumed+pending、原first-local/first-tick/retry_due；安装新cid | 全部原数据不变；旧matching ACK幂等，old retry按原deadline/attempt执行；不清重建mailbox。 |
| CA09 | cut.prepared非空；以及unreleased含旧Send/ReceiveAck/InboxAvailable索引 | prepared→CustodyBusy；unreleased允许且record/restore原样携带，原receipt/ACK/attempt不改。索引重复/空、zero receipt、超过界拒绝。 |
| CA10 | 某旧未持久RoutePlan/CustodyPlan早于admission；另例已持久尚未confirm | 前者安装后StalePlan可丢候选；后者先Busy/fail-closed并保留原record；不得用相同错误忽略持久义务。 |
| CA11 | admitted普通mailbox record、route snapshot、完整admission record roundtrip；旧restore读schema2；混合v2catalog/原record | 新入口同state/原release IDs，新Instance；旧入口拒绝2；wrong config/catalog/hash/revision/unknown/trailing/noncanonical拒绝。无跨PID恢复PASS主张。 |
| CA12 | 128/129 entries、4096/4097 membership cells、32MiB等号/+1、pending限额；u64 revision/step overflow | 精确界内可计划（其余约束合法），超界原子拒绝；没有GC旧custody或wrap计数。 |
| CA13 | 新context只授权部分endpoint；旧context继续，另一worker尚未投影 | 纯库只输出相关ProjectionIntent且不动旧权限；真实worker投影/调度验收留下一slice，不能用共享Arc宣称通过。 |
| CA14 | proof-like bytes经旧/新context传输，ACK先于worker application receipt | 原payload不变；运输准入/保管ACK不产生Final/Delivered/G/Ready结论。真正crypto/完整body验证属H1/H3。 |
| CA15 | golden request输入排序置换、snapshot canonical字段/array顺序mutation；同输入重放 | 构造normalize一致，重复拒绝；持久decode非规范字节拒绝；所有hash及projection ID匹配独立golden。 |

CA07/08的额外固定序列：沿v1 fixture构造source0→destination1、context0x33、codec0x44、Origin(step7,effect2,tick1_000_000,inc1)、payload `abc`；依次Enqueue/Attempt/Receive/MarkConsumed/Acknowledge选择前后不同cut保存，另保留一条未ACK message（effect3、payload `def`）以及一条未Drain网络copy。各旧动作使用v1真实纯函数和独立既有规则；admission step取大于其最后logical_step、tick不早于全部watermark。expected是所有原字段不变及同截止之后的原行为，不用新实现计算期望。

## 9. 交付、局部Q与仍OPEN

Docer交付仅本文小提交。实现由H1单writer在上述crate路径执行；独立Verifier先冻结CA01–15预期，再审候选；同准确SHA跑局部组件/旧v1回归。Docer没有运行任何Rust build/测试；golden计算只是规范输入计算。原始失败和旧worktree保留。

本纯切片没有待外部批准的普通字段：base/add-only/all-owner本地原子batch/v2/limits/API均可冻结后实现。后续局部Q：normal projection wire与worker/service receipt认证、实际service-root持久/custody-cut来源与release ledger、worker输入/输出转交任务和MarkConsumed边界、真实fsync/完整grant-Ack、资源实际预算。它们只阻真实进程接线的对应结论，不阻纯kernel API实现。Q-ADMIT、真实FinalVerifier、15 PID/五engine、M1整体端到端及所有旧完整恢复gate保持OPEN。
