# H1 API slice v1：三方可编译DTO与ports冻结稿

日期：2026-11-16。合同ID：`H1-API-1`。状态：**FROZEN BY LEAD；独立预期及下述两项shape澄清已复核，可交单Builder**。本切片不依赖Q-ADMIT的native回调决议，不实现runtime、Manager reducer、Publisher/GFR规则、真实FinalVerifier或durability。正式授权 `d55d39f73593676325bdc0fbb7ba3845ca93025f`，准确M0 `35d382ac445cc8bc779f0d18fd23cb57e54680da`；当前H1基线 `4741489ba7a07d2a3f3f194dbd59ae3fd8d17bb9`。文档中所有API测试均 **PLANNED / NOT_RUN**。

来源：SP §§3.1–3.12、SM §§3.2、3.7–3.10、4.1–4.3、GF §§4.3、4.7–4.8，见[总合同来源表](task-contract-h1-v1.md#1-权威与版本)；03 `e9c15e04bc83b335668f3ecd837367aa87344a89`；07 `788291735ce6db3c3eb78b4594a28d730cce71cd`；H2准备 `027ba8be9f8f64e35313f990f4f03c5d3c238a0c` 的 `poc/docs/contracts/h2-session-manager-healthy-preparation-v1.md` §§5–7；H3准备 `c4a944a754e6f9cfa50f57a2399b707a2555ce94` 的 `poc/docs/h3-publication-gfr-interface-proposal-v1.md` §§1–2。d55决定优先于旧H2/H3准备门禁与“缺状态则整批pending”提案。

## 1. 允许路径、依赖与本次完成边界

Builder唯一可写 `poc/crates/healthy-session-api/{Cargo.toml,Cargo.lock,src/**,tests/**,examples/**,README.md}`。这是独立workspace，package=`poc-healthy-session-api`、lib=`poc_healthy_session_api`、version=`0.1.0`、edition2024、rust-version1.95.0、publish=false、resolver2。仅直接依赖 `poc-types={path="../types"}`、`poc-viewer-gfr={path="../viewer-gfr"}`；不登记根workspace，不增加runtime/Commonware/serde依赖，不改03/07代码。允许标准库。源码可拆模块，但下述名字全部在crate root公开。

实现仅含DTO、无默认实现的traits、本地ID函数、shape检查、消费编译例和独立goldens。**没有网络、Clock推进、文件/fsync、签名、验签、执行、选择、public replay或输出释放实现。** 所有DTO只表示声称的输入/记录；构造或`validate_shape()`成功不认证来源，不产生Ready、durability或finality事实。H1 runtime才可实际发行/接受ReadyToken、提交root及安装guard；H2/H3不能通过手工构造DTO绕过其ledger校验。

API v1传递typed对象，**不提供Serde/完整DTO wire/ReplayRecord codec**。`ReplayRecord`完整typed携带，Source EvidenceRef绑定的实际源记录由H3已有持久层编码；opaque Manager数据由H2私有codec定义。未来wire必须另切片冻结并复用03/07编码，不能从此crate派生“新的canonical RegionalResult”。私有控制ID的hash算法仅用于adapter身份，见§3。

## 2. 公共类型复用与精确DTO

以下Rust声明给出确切字段、variant和方法签名；实现体由Builder按§3–§5填写。除`LocalId`内字段外，DTO字段公开；定义的DTO/enum统一派生`Clone, Debug, PartialEq, Eq`，能Copy的scope/id/ref小结构可另派生Copy。只对`LocalId/IdKind/WorkerScope`要求`PartialOrd, Ord, Hash`，不要求已有复杂对象实现Ord。ports及`VerifierBinding`不要求Clone/Debug。

```rust
pub use poc_types::domain::{
    Anchor, AnchorEvidence, Canonical, CanonicalObject, Cid, CloseReason,
    ConfigRef, Descriptor, DomainError, ExecutionAnchor, FinalVerifier, Hash32,
    InputMissing, InputStatus, PhysicalBlock, Pid, PublicConfig, RegionId,
    RegionalResult, ReplayPosition, ResultKey, SessionId, VirtualBlock,
};
pub use poc_viewer_gfr::{
    GfrError, ReplayRecord, VerificationBasis, VerificationProfile, VerifierBinding,
    encode_position, lifecycle_id, lifecycle_projection,
};

pub const API_VERSION: u16 = 1;
pub const MAX_OPAQUE_BYTES: usize = 65_536;
pub const MAX_ACTIONS: usize = 128;
pub const MAX_GUARDS: usize = 128;
pub const MAX_REFS: usize = 128;
pub const MAX_PREP_BLOCKS: usize = 128;
pub const MAX_REPLAY_EVENTS: usize = 128;
pub const MAX_REPLAY_EFFECTS: usize = 128;

pub struct LocalId(Hash32);
impl LocalId {
    pub fn from_hash(hash: Hash32) -> Result<Self, ApiError>;
    pub fn as_hash(&self) -> &Hash32;
}
#[repr(u8)]
pub enum IdKind {
    Operation = 1, Program = 2, Preparation = 3, Ready = 4,
    Action = 5, ManagerDelta = 6, ReplayBatch = 7, Delivery = 8,
}
pub struct WorkerScope {
    pub run: Hash32,
    pub config: ConfigRef,
    pub worker: Pid,
    pub incarnation: u64,
}
pub struct EvidenceRef { pub source: Hash32, pub object: Hash32 }
pub struct RootRef { pub sequence: u64, pub hash: Hash32 }
pub struct GrantRef {
    pub scope: WorkerScope,
    pub sequence: u64,
    pub hash: Hash32,
    pub tick_us: u64,
}
pub struct ControlledOrigin {
    pub grant: GrantRef,
    pub root: RootRef,
    pub effect_index: u32,
}
pub struct ProgramHandle {
    pub scope: WorkerScope,
    pub cid: Cid,
    pub instance_id: LocalId,
    pub generation: u64,
}
pub struct PreparationJob {
    pub program: ProgramHandle,
    pub id: LocalId,
    pub generation: u64,
}
pub struct StateMaterial {
    pub checkpoint: Option<ExecutionAnchor>,
    pub physical_prefix: Vec<PhysicalBlock>,
    pub virtual_prefix: Vec<VirtualBlock>,
    pub input_refs: Vec<EvidenceRef>,
}
pub struct PreparationRequest {
    pub generation: u64,
    pub anchor: Anchor,
    pub anchor_evidence: AnchorEvidence,
    pub material: StateMaterial,
    pub provenance: Vec<EvidenceRef>,
    pub validation_profile: Hash32,
}
pub struct ReadyToken {
    pub id: LocalId,
    pub job: PreparationJob,
    pub anchor: Anchor,
    pub state_root: Hash32,
    pub native_hash: Hash32,
    pub validation_profile: Hash32,
    pub execution_basis: EvidenceRef,
    pub origin: ControlledOrigin,
}
pub struct ReadyState {
    pub token: ReadyToken,
    pub execution_anchor: ExecutionAnchor,
    pub ordered_input_refs: Vec<EvidenceRef>,
}
pub enum ProgramPhase { Assigned, Preparing, Ready, Running, Paused, Terminated }
pub struct ProgramObservation {
    pub program: ProgramHandle,
    pub phase: ProgramPhase,
    pub original_entry_us: Option<i64>,
    pub view: Option<u64>,
    pub terminal: Option<CloseCause>,
    pub history: Option<EvidenceRef>,
    pub timers: Option<EvidenceRef>,
    pub pending_work: Option<EvidenceRef>,
    pub origin: ControlledOrigin,
}
pub enum CancelCause { Superseded, Explicit, Closed }
pub enum CloseCause {
    Program(CloseReason),
    ClockUnsafe { evidence: EvidenceRef },
}
pub enum CloseScope { Cid(Cid), Session(SessionId), Branch(u64) }
pub enum ProgramCommand {
    Create { descriptor: Descriptor, plan_basis: Vec<EvidenceRef> },
    Prepare { program: ProgramHandle, request: PreparationRequest },
    Supply { job: PreparationJob, material: StateMaterial,
             provenance: Vec<EvidenceRef> },
    CancelPrepare { job: PreparationJob, cause: CancelCause },
    Enter { program: ProgramHandle, ready: ReadyToken },
    Pause { program: ProgramHandle, decision: EvidenceRef },
    SelectBack { program: ProgramHandle, decision: EvidenceRef },
    Close { scope: CloseScope, cause: CloseCause, basis: Vec<EvidenceRef> },
}
```

Create尚无原生handle，H1按`(worker,cid)`返回唯一原handle；其余操作显式带原handle/job generation，Close可覆盖尚未Create的session/branch并写tombstone。`ProgramHandle.generation=1`为本健康profile；Pause/SelectBack不增generation，不能伪装同cid重建。Preparation generation允许单调增加的不同job；旧completion不得作用于新job。`CloseCause::ClockUnsafe`是私有原因，绝不变更03公共`CloseReason`或映射成`Time`。

```rust
pub struct ReplayCursor {
    pub source: Hash32,
    pub sequence: u64,
    pub batch_id: LocalId,
    pub position: ReplayPosition,
}
pub struct ReplayBatch {
    pub id: LocalId,
    pub source: Hash32,
    pub sequence: u64,
    pub previous: Option<LocalId>,
    pub source_record: EvidenceRef,
    pub record: ReplayRecord,
    pub lifecycle_ids: Vec<Hash32>,
    pub origin: ControlledOrigin,
}
pub enum ManagerCause {
    Replay(ReplayBatch),
    Regional { key: ResultKey, evidence: EvidenceRef },
    Local { operation: LocalId, input: EvidenceRef },
}
pub struct OpaqueManagerDelta { pub codec: u32, pub bytes: Vec<u8> }
pub enum GuardIntent {
    Terminate { scope: CloseScope, cause: CloseCause, basis: Vec<EvidenceRef> },
    Schedule { session: SessionId, running: Option<ProgramHandle> },
    BlockNewEnter { clock_evidence: EvidenceRef },
}
pub enum ActionBody {
    Program(ProgramCommand),
    OfferRegional(RegionalDelivery),
    ForwardRegional { delivery: RegionalDelivery, to: RegionId },
}
pub struct ActionIntent { pub id: LocalId, pub body: ActionBody }
pub struct ManagerDelta {
    pub id: LocalId,
    pub scope: WorkerScope,
    pub base_revision: u64,
    pub next_revision: u64,
    pub cause: ManagerCause,
    pub opaque: OpaqueManagerDelta,
    pub guards: Vec<GuardIntent>,
    pub actions: Vec<ActionIntent>,
}
pub struct RootCommitted {
    pub scope: WorkerScope,
    pub delta_id: LocalId,
    pub manager_revision: u64,
    pub root: RootRef,
    pub replay_cursor: Option<ReplayCursor>,
    pub actions_pending: Vec<LocalId>,
}
pub struct ActionExecution {
    pub committed: RootCommitted,
    pub action_id: LocalId,
    pub grant: GrantRef,
}
pub enum ProgramReply {
    Assigned(ProgramHandle),
    Preparing { job: PreparationJob, missing: Vec<WaitReason> },
    Ready(ReadyState),
    Entered { program: ProgramHandle, original_entry_us: i64 },
    Paused(ProgramHandle),
    Running(ProgramHandle),
    Terminated { scope: CloseScope, cause: CloseCause, durable_guard: EvidenceRef },
    PrepareCancelled(PreparationJob),
}
pub enum ActionReply {
    Program(ProgramReply),
    PublicationAccepted { key: ResultKey, task: EvidenceRef },
    ForwardAccepted { key: ResultKey, custody: EvidenceRef },
    SuppressedByGuard { guard: EvidenceRef },
}
pub struct AppliedReceipt {
    pub delta_id: LocalId,
    pub action_id: LocalId,
    pub committed_root: RootRef,
    pub origin: ControlledOrigin,
    pub reply: ActionReply,
}
pub struct ActionsApplied {
    pub committed: RootCommitted,
    pub receipts: Vec<AppliedReceipt>,
    pub completion_origin: ControlledOrigin,
}
pub enum FailureDisposition { GuardsRetained, CommittedPending, RuntimeStopped }
pub struct FailClosed {
    pub scope: WorkerScope,
    pub delta_id: LocalId,
    pub reason: RejectReason,
    pub disposition: FailureDisposition,
    pub committed: Option<RootCommitted>,
    pub durable_guards: Vec<EvidenceRef>,
    pub actions_pending: Vec<LocalId>,
}
pub enum CommitOutcome { Rejected(RejectReason), Committed(RootCommitted), FailClosed(FailClosed) }
pub enum ActionProgress {
    Applied(AppliedReceipt),
    Pending { action_id: LocalId, missing: Vec<WaitReason> },
    FailClosed(FailClosed),
}
pub enum CompletionOutcome { Pending(RootCommitted), Applied(ActionsApplied), FailClosed(FailClosed) }
```

`RootCommitted`只确认整批guard/状态/cursor/义务已持久接受，不能当作Enter/Close/Offer动作完成。`AppliedReceipt`也只完成其具体命令：Preparing回执说明准备job登记成功，不表示Ready。过时Enter被已持久guard抑制时返回`SuppressedByGuard`，不能报Entered。`ActionsApplied`保存原commit及每项回执，不改写原`actions_pending`作为历史。旧guard生效后失败只能`FailClosed`；`Rejected`仅用于尚未产生任何durable safety/commit事实的预检失败。

```rust
pub enum ProofCheck {
    CertificateSubject { basis: Vec<EvidenceRef> },
    TargetBound { block: PhysicalBlock, basis: Vec<EvidenceRef> },
}
pub struct ProofObserved {
    pub key: ResultKey,
    pub descriptor: Descriptor,
    pub native_finalization: Vec<u8>,
    pub check: ProofCheck,
    pub native_profile: VerificationProfile,
    pub origin: ControlledOrigin,
}
pub struct RegionalDelivery {
    pub id: LocalId,
    pub result: RegionalResult,
    pub validation_basis: Vec<EvidenceRef>,
    pub native_profile: VerificationProfile,
    pub origin: ControlledOrigin,
}
pub enum WaitReason {
    Input(InputMissing),
    Evidence(EvidenceRef),
    Body(Hash32),
    Finality(ResultKey),
    NativeVerifierBasis,
    NotBefore { local_deadline_us: i64 },
    Work(PreparationJob),
    Action(LocalId),
}
pub enum ApiError {
    Domain(DomainError),
    Gfr(GfrError),
    InvalidField(&'static str),
    BindingMismatch(&'static str),
    LimitExceeded { field: &'static str, actual: usize, max: usize },
    DuplicateId(LocalId),
    Overflow,
    NonNativeProfile,
}
pub enum RejectReason {
    Shape(ApiError), InvalidSource, InvalidProof, InvalidBody, InvalidAncestry,
    InvalidState, UnsupportedProfile, Conflict, StaleRevision, Terminated,
    ClockUnsafe, Capacity, PersistenceFailure,
}
pub enum PortResult<T> { Ready(T), Wait(Vec<WaitReason>), Reject(RejectReason) }

pub trait ManagerCommitPort {
    fn commit(&mut self, delta: &ManagerDelta, grant: &GrantRef) -> CommitOutcome;
    fn complete(&mut self, committed: &RootCommitted, receipts: &[AppliedReceipt],
                grant: &GrantRef) -> CompletionOutcome;
}
pub trait ProgramPort {
    fn execute(&mut self, execution: &ActionExecution,
               command: &ProgramCommand) -> ActionProgress;
    fn observe(&self, program: &ProgramHandle) -> PortResult<ProgramObservation>;
    fn preparation(&self, job: &PreparationJob) -> PortResult<ReadyState>;
}
pub trait RegionalResultPort {
    fn offer(&mut self, execution: &ActionExecution,
             delivery: &RegionalDelivery) -> ActionProgress;
}
pub trait ReplayFeedPort {
    fn next(&self, after: Option<&ReplayCursor>) -> PortResult<ReplayBatch>;
}
pub trait NativeVerifierPort {
    fn binding(&self) -> PortResult<VerifierBinding<'_>>;
}

pub fn derive_local_id(scope: &WorkerScope, kind: IdKind,
                       ordinal: u64, subject: Hash32) -> Result<LocalId, ApiError>;
pub fn validate_native_binding(binding: &VerifierBinding<'_>, config: &PublicConfig)
    -> Result<(), ApiError>;
```

没有默认port实现，没有`issue_ready(bool)`、没有`impl FinalVerifier`。H1实现`ProgramPort/ManagerCommitPort/NativeVerifierPort`，H3实现`RegionalResultPort/ReplayFeedPort`；H2生成ManagerDelta并消费回执。`observe/preparation`只读取已经受控提交的观察，不推进work/时钟或发行新token；Prepare命令已回Preparing后，真实work在后续grant完成并保存Ready，H2随后读取该ReadyState（其token携带实际completion origin）。取消/终止后不得靠读取生成新Ready；历史token也不能绕过Enter最新guard。H3仅消费真实材料与既有FinalVerifier/VerificationBasis，不发行ReadyToken。API库不会调用`FinalVerifier::check`来“证明”运行通过；`validate_native_binding`仅检Native profile和版本形状。

Progress（notarization/certification/nullification）不能构造成`ProofObserved`；它留在runtime审计端。ProofObserved必须携带真实Finalization bytes，但本API只检查非空和bounds。`CertificateSubject`说明目标body尚缺，不能调用需要完整block的03 CheckFINAL；`TargetBound`才有目标block，但不代表祖先/执行已完整。RegionalDelivery具有完整RegionalResult形状，H3仍须调用真实验证器和完整性helper；DTO名称不代表已验真或已公开/G接纳。

## 3. 私有控制ID与字节向量

`LocalId`是H1本地控制身份，**不是03 cid/ResultKey，也不是07 lifecycle id/public编码**。`LocalId::from_hash`拒绝全零；其他Hash32不因本API变更03语义。`derive_local_id`先验证WorkerScope和ordinal>0，再调用既有`poc_types::hash_parts`，不另实现hash/framing：

```text
hash_parts("H1/API/ControlId/v1", [
  u16BE(API_VERSION), u8(IdKind), raw32(scope.run), raw32(scope.config),
  Pid::encode(scope.worker), u64BE(scope.incarnation), u64BE(ordinal), raw32(subject)
])
```

每个part以及domain由既有helper加`u64BE(length)`；Pid调用03 Canonical，因此是`0001 || pid`，不能改成裸u8。返回32字节转Hash32/LocalId。此处没有把整个DTO编码为第二套协议对象。

固定用法：ProgramHandle.instance_id由H1分配Program kind；prepare/ready各自kind、真实job generation/发行序号；ManagerDelta.id由其原因引用与本地操作序号确定；ActionIntent第i项（从0）严格为`derive_local_id(&delta.scope,Action,i+1,*delta.id.as_hash())`。重试保留原id和完整DTO；同id不同完整内容由持久owner判Conflict，不能仅因hash/position相等吞掉新内容。ReplayBatch.id严格为`derive_local_id(&origin.grant.scope,ReplayBatch,sequence,source)`：它标识来源流位置，**不自称整个record的内容hash**；内容由source_record.object及完整typed record保留并在重复时比较。shape检查重算该id；完整record的bytes/hash由H3源持久层负责，API不编码或认证它。

golden固定scope：run=`11`×32、config=`22`×32、Pid7、incarnation1，subject=`33`×32。每个preimage长209B：

| kind/ordinal | 期望LocalId raw32 hex |
| --- | --- |
| Action(5)/9 | `378abd8372b975ca5ec0a7a5d926c1bf9dfe89451ac0969b11ad8cf38500e851` |
| ManagerDelta(6)/9 | `9a3aff0ea2122313497a3f08d601a7c95f4fbfea04778b19b31679447ebdc88f` |
| Action(5)/10 | `e19c244383676f7a97329d26ada91933adad80e74651d6c986da51cee3ab96c7` |

第一向量完整preimage：

```text
000000000000001348312f4150492f436f6e74726f6c49642f76310000000000000002000100000000000000010500000000000000201111111111111111111111111111111111111111111111111111111111111111000000000000002022222222222222222222222222222222222222222222222222222222222222220000000000000003000107000000000000000800000000000000010000000000000008000000000000000900000000000000203333333333333333333333333333333333333333333333333333333333333333
```

这些为Docer用Python hashlib从冻结preimage独立计算的规范向量，未运行被测Rust实现；Verifier应独立重算并封存。07 lifecycle向量直接使用已有`lifecycle_projection/lifecycle_id`接口，API只与既有输出比较；不得再实现其tag/framing。

## 4. 可执行shape guards（不作协议验证）

以下类型必须公开`validate_shape(&self) -> Result<(),ApiError>`：`WorkerScope/EvidenceRef/RootRef/GrantRef/ControlledOrigin/ProgramHandle/PreparationJob/StateMaterial/PreparationRequest/ReadyToken/ReadyState/ProgramObservation/ReplayBatch/ManagerDelta/RootCommitted/ActionExecution/AppliedReceipt/ActionsApplied/FailClosed/ProofObserved/RegionalDelivery`。内部辅助检查可私有；不要求每个enum都有public方法。检查顺序为表自上而下，容器按字段声明顺序递归，先长度后内容，首次错误返回。字符串field名使用Rust字段名（嵌套错误保留原名），错误Display不冻结。

| 对象/检查 | 精确拒绝条件和错误 |
| --- | --- |
| 基本绑定 | WorkerScope.run全零=`InvalidField("run")`；incarnation≠1=`InvalidField("incarnation")`。Root/Grant sequence=0或hash全零对应InvalidField。EvidenceRef.source/object全零同理。local id零由constructor拒绝`InvalidField("local_id")`。不新增公共Cid/ConfigRef合法性规则。 |
| handle/job | program.scope通过；program.generation≠1拒绝；job.generation=0拒绝。原instance真实性/唯一性、generation单调由H1 ledger检查，API不假装能证明。 |
| 资源 | opaque.bytes长度1..=65536且codec≠0；actions≤128、guards≤128、refs≤128、prep physical/virtual各≤128、replay.events/effects各≤128；AnchorEvidence.source_objects≤128且每项≤65536；超过用LimitExceeded，要求非空的空bytes/codec用InvalidField。Native proof长度1..=07 MAX_PROOF_BYTES(32768)，沿用公开上界；每个03对象调用自身validate或encode_checked。 |
| StateMaterial/Request | optional checkpoint只调用ExecutionAnchor.validate，physical按03 PhysicalBlock.validate，virtual的metadata/tx/dep格式用03既有helpers；不执行前缀、不核对hash连通/来源。Request.generation>0、AnchorEvidence.validate_structure、validation_profile非零；provenance允许空以表达WAIT。 |
| ReadyToken/State | token.job有效；token.origin.grant.scope必须等于job.program.scope；anchor用03validate；validation_profile非零。ReadyState.execution_anchor.validate；其anchor与token.anchor精确相等，state.root()与token.state_root相等，native_hash与token.native_hash相等；ordered_input_refs≤128且非空。只检查字段自洽，不能推出checkpoint真实、执行正确或token已被H1发行。 |
| ProgramObservation | program和origin scope相同；Terminated当且仅当terminal=Some；Running/Paused必须original_entry_us=Some且view=Some(v>0)；Assigned/Preparing/Ready时entry/view均None；Terminated可未Enter，若view=Some仍须v>0及entry=Some。history/timers/work引用只检查shape，不证明native原实例。 |
| ReplayBatch | sequence>0；sequence1要求previous=None，sequence>1要求previous等于同scope/source在sequence−1的派生id；source非零且source_record.source==source；source_record/origin结构合法；id按§3重算。record.position.validate且每个event.position等于record.position；lifecycle_ids长度等于events长度，逐项必须等于07 lifecycle_id(event)，重复ID拒绝`InvalidField("lifecycle_ids")`。**允许events=[]**；完整input_ref/effects仍保留，不因空事件省略record。来源、实际消费cursor连续性、完整记录真伪由生产consumer校验。 |
| ManagerDelta | scope有效；next_revision严格=base_revision.checked_add(1)，溢出Overflow，错值BindingMismatch("next_revision")。Replay cause.origin.scope与delta.scope相等；每个handle/ready/job的scope都匹配，Create descriptor.plan.config_ref匹配scope.config；Enter ready.job.program==program。Offer/Forward可携带其他worker的真实origin，但run/config必须一致，不能要求来源worker等于本地worker。Action IDs按§3及顺序重算且唯一；重名DuplicateId。其他opaque字节不解析；guard/Close不能用shape检查替代合法作用域验证。 |
| RootCommitted/ActionExecution | root有效、revision>0、pending唯一≤128；cursor position有效且seq>0。ActionExecution.grant.scope==committed.scope，action_id必须在原actions_pending中，否则BindingMismatch("action_id")；实际grant/root真实性由runtime检查。 |
| AppliedReceipt/ActionsApplied | origin/root结构合法；ProgramReply中的handle/job/ready本地scope与receipt.origin.grant.scope相等；ActionsApplied中receipt.delta_id、committed_root、origin.grant.scope逐项匹配commit；receipt action IDs按原actions_pending顺序恰好覆盖、无重复（允许双方空）；completion_origin.scope一致。receipt的reply只作递归字段自洽检查，不能把合成receipt当durability。 |
| FailClosed | scope、现有commit/guard refs有效；GuardsRetained要求durable_guards非空；CommittedPending要求committed=Some且scope/delta一致；RuntimeStopped允许无root但不构造成功receipt。commit存在时actions_pending只能是commit原pending的有序子集；不得把FailClosed转换成Rejected/Running。 |
| ProofObserved | descriptor结构/编码有效；key.cid==descriptor.cid；native_profile必须Native、protocol_version/verifier_version均非零、source_revision为40个小写hex字符（仅SHA形状，不证明提交存在）；proof在界内，origin scope.config==descriptor.plan.config_ref。TargetBound的block.digest()==key.block_hash；CertificateSubject没有body字段，不能伪造dummy block填充。 |
| RegionalDelivery | native profile与origin/config同上；result.chain非空且≤10，末项精确等于result.block，首parent==descriptor.anchor.block.block_id，逐项parent==前项physical digest、height严格增加且在固定window；每项03结构合法；final_proof非空且≤32768；anchor_evidence结构合法；validation_basis非空≤128。不执行Apply/Merge或验签，不推出final/provenance/G资格。 |
| Native binding | `validate_native_binding`先拒绝NonCryptographicFixture为NonNativeProfile；Native版本必须非零、source_revision为40个小写hex字符，再调用既有profile.validate(config)映射ApiError::Gfr；不调用verifier/basis，不把成功当密码学有效或提交存在证明。 |

Lead冻结澄清：ManagerDelta先检查所有action ID重复，再按ordinal比较派生ID；重复明确返回DuplicateId。FailClosed的任何committed=Some都必须与外层scope/delta_id一致，不限于CommittedPending；没有commit时actions_pending同样须≤128且唯一。上述是shape自洽检查，不认证外部持久事实。

ReplayBatch按其专门条款检查：完整保留input_ref/effects，检查effects数量和明确列出的位置、事件、LIFE-ID及外层绑定，不递归验证Duty/Tail内的域对象，也不解析InputRef::L1的source_bytes。资源行“每个03对象”指本表明确要求检查的对象，不构成另一套public replay validator。实际producer/consumer必须验证完整记录合法性；本slice的计数限制不声称整个嵌套ReplayRecord已通过字节容量或语义检查，shape成功不能授权root提交/动作释放。此澄清经独立来源预期核对，未改变公共对象或编码。

字段名已只读核对：03 `PlanRecord`有`config_ref/window`，**没有rule_version/verifier_version**；版本来自`PublicConfig`与其生成的`ProgramConfig`。因此ProofObserved/RegionalDelivery的无配置shape入口只能检查Native形状，实际配置匹配由`validate_native_binding(binding, &PublicConfig)`及真实消费者验证；不得增加影子Plan字段或在没有配置时声称已校验版本兼容。

## 5. 持久owner和fail-closed语义

H2不持有提交数据库/engine锁；它计算不可变ManagerDelta。H1`ManagerCommitPort::commit`实际实现必须先预检完整合法record，再在同一root/aux边界接受opaque state/cursor/全部action义务及安全guard。已知合法旧Close不等待后继Ready。Selected+Preparing可和旧terminal一起提交；缺普通执行材料只对应Preparation WAIT。结构/来源未知或错误不可提前声称RootCommitted。

RootCommitted允许后续ActionsPending。实际动作在`ActionExecution`的受控grant中逐项完成，不能因已commit cursor就返回ActionsApplied。每个action重试保留原ID/内容；旧Enter/SelectBack执行前仍检查最新terminal/Clock/selection。FailClosed保留已提交事实和guard，不能回滚为Running，不能撤回H3已提交GFR pc。缺receipt继续待办；同worker只有H1安装持久提交，H3服务custody/Publisher Recorded不被root receipt替代。

`ControlledOrigin`是完成提交后附上的来源引用：producing root覆盖原effect payload和slot，外层观察引用该root；不能把含自身root hash的wrapper递归放回自己的hash preimage。后续consumer提交可保存完整wrapper。当前slice不定义完整root/wire编码，但明确不允许自引用hash或“为了填origin先释放未提交输出”。同一个source sequence/batch id的重复，生产consumer须比较完整ReplayRecord及来源引用；不能只比较position、event IDs或此处的局部hash就忽略input_ref/effects变化。

本slice不实现上述行为；它只保证消费者能表达这些状态，编译例必须展示分层。真实guard/receipt与完整grant/Ack绑定是后续runtime切片的验收，不能用DTO单元测试关闭。

## 6. 独立验收向量与消费例（全部 PLANNED / NOT_RUN）

| ID | 输入/断言 | 允许结论 |
| --- | --- | --- |
| API01 | 三份固定ID golden+209B preimage；pid裸字节、端序、kind/ordinal/config/worker改变负例 | 私有ID算法一致，无公共编码复制。 |
| API02 | zero run/id、incarnation0/2、ordinal0、revision溢出；合法pid0/14 | 本地shape拒绝；不声称身份认证。 |
| API03 | Create→原handle DTO；Prepare/Supply/Enter/Pause/SelectBack/Close消费例；错worker/cid/job generation | shape绑定与consumer编译；不声称真实engine/一次Enter已实现。 |
| API04 | Ready完整exact anchor/state root/native hash；改anchor metadata、state、worker；root-only不能构成ReadyState | 自洽性检查；编译例明确token由ProgramPort返回，H3无发行port；不称真实执行证明。 |
| API05 | 同pc Close+Assign完整ReplayRecord；互换/少一lifecycle id，空events记录 | 逐事件07身份与完整record保持；不按position去重；空events合法且effects/input保留。 |
| API06 | successor material缺失的ManagerDelta含旧Terminate、新Preparing opaque状态和Prepare action | RootCommitted(actions_pending) DTO可表达关闭已接受，不能伪造Ready/ActionsApplied；不运行reducer。 |
| API07 | ActionsApplied缺/重复/错action/root/scope回执；正确全覆盖和空集合 | receipt绑定/shape检查，不是假fsync证明。 |
| API08 | 已durable guard后失败、root提交后动作pending、runtime停止 | FailClosed类型能保留guard/commit/原ID；生产port未实现，无回滚Running的API helper。 |
| API09 | proof-only CertificateSubject与TargetBound、完整RegionalDelivery；错body/chain/key；Notarization不提供转换器 | 证据层级与shape，真实3/4签与错域检查仍属native测试。 |
| API10 | Native/fixture profile，历史公开配置版本匹配/不匹配；trait object直接消费03 FinalVerifier和07 VerifierBinding | 无默认true实现、无另一个验签trait/codec。 |
| API11 | actions/refs/proof/opaque精确上界与+1；十block合法、11拒绝 | API有界shape；不声称运输/Publisher字节总量已验收。 |
| API12 | `examples/h2_consumer.rs`、`examples/h3_consumer.rs`编译 | H2只计划/提交/分层消费；H3只消费结果/Native binding和提供完整ReplayBatch，不发行Ready。样例不实现假的production verifier。 |
| API13 | 手工构造但字段自洽的Ready/receipt可以通过shape；伪引用不能得到本slice未实现的runtime验证 | 明确shape≠来源/执行/durability；不得把测试DTO的通过计作H1发行、fsync或真实final。 |

测试仅测试所写local helper/shape guard与接口可消费性，不能写一套模拟runtime来证明自己。独立预期由Verifier存专属路径，再交Builder；消费示例可用只读函数/泛型port参数，避免实现会被误当生产的stub。fixture标识清楚。

执行命令由lead在实际manifest存在后锁定；预期使用独立manifest的`cargo check --locked --all-targets`及API局部tests（**PLANNED / NOT_RUN**）。Docer不build；不因仅API新增重复M0 heavy矩阵。发布精确API候选SHA、合同版本、独立向量/检查结果与消费者例子，H2/H3正常消费该SHA。总体H1/M1仍OPEN，Q-ADMIT另由专项source分析收口后才启用native voting路径。
