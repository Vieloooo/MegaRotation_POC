# sim-transport API v1

2026-11-15。**FROZEN by lead；实现/运行验收尚未完成**。与 [kernel合同](contracts/task-contract-sim-transport-kernel-v1.md) 一起由02 lead在独立预期审阅后冻结。以下名称、字段顺序、类型、方法和错误是Builder/Verifier/06/09共同消费的最小接口；不是新RPC/protobuf或03公共对象规范。

crate路径 `poc/crates/sim-transport/`，Rust导入名 `sim_transport`。文中所有pub value types可Clone/Debug/PartialEq/Eq；Endpoint、ChannelId、Id、StepStamp等key还实现Ord。私有kernel/plan不暴露可变内部字段；plan不Clone，commit按值消费。serde采用下文唯一编码，不另设feature切换编码。

## 1. 身份、消息与hash

```rust
pub type Id = [u8; 32];
pub type Tick = u64;
pub type LocalTime = i64;
pub enum Endpoint { Replica(u32), Service(u32), Reader(u32) }
pub enum EndpointKind { Replica, Service, Reader }
pub struct ChannelId(pub u16); // 0无效；由消费配置分配运输ID，不是原生wire enum
pub struct EndpointBinding { pub endpoint: Endpoint, pub incarnation: u64 }
pub struct StepStamp {
    pub logical_step_id: u64, pub tick_us: Tick,
    pub event_id: u64, pub causal_wave: u64,
}
pub struct Origin {
    pub step_id: u64, pub effect_index: u32,
    pub sent_tick_us: Tick, pub source_incarnation: u64,
}
pub struct Message {
    pub run: Id, pub config: Id, pub source: Endpoint, pub destination: Endpoint,
    pub channel: ChannelId, pub context: Option<Id>, pub codec: Id,
    pub origin: Origin, pub payload_hash: Id, pub logical_id: Id, pub payload: Vec<u8>,
}
pub struct Envelope {
    pub message: Message, pub attempt: u32,
    pub source_incarnation: u64, pub destination_incarnation: u64,
}
pub fn framed_hash(domain: &str, parts: &[&[u8]]) -> Id;
pub fn payload_hash(payload: &[u8]) -> Id;
pub fn message_id(message: &Message) -> Id; // 忽略自身logical_id，只取下面列明字段
impl Message { pub fn validate_hashes(&self) -> Result<(), Error>; }
```

`Id`除实际hash结果外的opaque输入不得全零；incarnation/step_id/event_id/attempt≥1，effect_index和tick可为0。Endpoint排序为tag1/2/3后id-u32。03负责public pid→region/member/signer、公钥、cid/Descriptor和业务字节；02只按显式注册/相等比较。caller可构造Message后设 `payload_hash` 和 `logical_id`，每个公共接纳入口都重验，不能信任已填字段。

H为合同的length-prefixed SHA-256；以下每个逗号项是独立part：

| 值 | domain与parts |
| --- | --- |
| payload_hash | `SIMT/payload/v1`, payload bytes |
| message_id | `SIMT/message/v1`, run,config,Endpoint(source),Endpoint(destination),channel-u16BE,Option(context),codec,origin.step_id-u64BE,origin.effect_index-u32BE,origin.sent_tick_us-u64BE,origin.source_incarnation-u64BE,payload_hash |
| kernel_config_hash | `SIMT/config/v1`, canonical normalized TransportConfig JSON bytes |
| jitter word | `SIMT/jitter/v1`, seed-u64BE,Endpoint(src),Endpoint(dst),channel-u16BE,message_id,attempt-u32BE；取digest[0..8]作u64BE，再 `% (jitter_us+1)` |
| packet tie | `SIMT/tie/v1`, seed-u64BE,packet_seq-u64BE；比较完整32bytes lexicographic |
| rate profile | `SIMT/rate/v1`, UTF8 `fixed-rate-one-v1` |
| custody transition | `SIMT/custody-transition/v1`, run,config,Endpoint(owner),base_revision-u64BE,canonical MailboxAction JSON bytes |
| custody record | `SIMT/custody-record/v1`, canonical PersistenceRecord JSON bytes |

Endpoint hash编码是tag-u8+id-u32BE；Option(context)是0或1+32bytes。current wrapper incarnation、attempt、network copy、接收时间不进入message ID。保持原始签名payload bytes，kernel不重新编码/签名它们。

## 2. 配置与RouteKernel

```rust
pub enum ContextPolicy { Forbidden, Required }
pub enum Reliability { BestEffort, Reliable }
pub struct ChannelSpec {
    pub id: ChannelId, pub context: ContextPolicy, pub reliability: Reliability,
}
pub struct ChannelPermit { pub channel: ChannelId, pub contexts: Vec<Id> }
pub struct EndpointRegistration {
    pub binding: EndpointBinding,
    pub send: Vec<ChannelPermit>, pub receive: Vec<ChannelPermit>,
}
pub enum EndpointSelector { Any, Kind(EndpointKind), Exact(Endpoint) }
pub enum ChannelSelector { Any, Exact(ChannelId) }
pub enum OrderMode { Fifo, Unordered }
pub enum OfflineMode { Hold, Drop }
pub struct OfflineInterval { pub start_us: Tick, pub end_us: Tick }
pub struct FaultEntry {
    pub send_ordinal: u64, pub drop: bool,
    pub duplicate_delay_us: Option<u64>, pub extra_delay_us: u64,
}
pub struct LinkProfile {
    pub latency_us: u64, pub jitter_us: u64, pub order: OrderMode,
    pub offline: Vec<OfflineInterval>, pub offline_mode: OfflineMode,
    pub capacity: u32, pub drain_bytes_per_second: u64, pub faults: Vec<FaultEntry>,
}
pub struct LinkRule {
    pub source: EndpointSelector, pub destination: EndpointSelector,
    pub channel: ChannelSelector, pub profile: LinkProfile,
}
pub struct Limits {
    pub endpoints: u32, pub channels: u32, pub rules: u32,
    pub payload_bytes: u64, pub link_copies: u32, pub queued_copies: u32,
    pub queued_payload_bytes: u64, pub tracked_messages: u32,
}
pub struct TransportConfig {
    pub schema: u32, pub run: Id, pub config: Id, pub seed: u64,
    pub limits: Limits, pub endpoints: Vec<EndpointRegistration>,
    pub channels: Vec<ChannelSpec>, pub rules: Vec<LinkRule>,
}
impl Default for Limits { /* 64,32,1024,65536,64,1024,4194304,8192 */ }
impl Default for LinkProfile { /* 合同§3默认值 */ }
impl TransportConfig {
    pub fn normalize(self) -> Result<Self, Error>;
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, Error>;
    pub fn hash(&self) -> Result<Id, Error>;
}
pub enum RouteCommand {
    Send(Envelope), Drain,
    Rebind { endpoint: Endpoint, expected_incarnation: u64, new_incarnation: u64 },
}
pub struct PacketRef {
    pub logical_id: Id, pub attempt: u32, pub copy_index: u8, pub packet_seq: u64,
}
pub enum DropReason {
    Fault, Offline, LinkCapacity, GlobalCapacity, PayloadBudget, StaleBinding,
}
pub struct Delivery {
    pub envelope: Envelope, pub packet: PacketRef,
    pub raw_due_us: Tick, pub eligible_tick_us: Tick, pub delivered_tick_us: Tick,
}
pub enum RouteEventKind {
    Queued { packet: PacketRef, raw_due_us: Tick },
    Dropped { packet: PacketRef, reason: DropReason, raw_due_us: Option<Tick> },
    Delivered(Delivery), Rebound(EndpointBinding),
}
pub struct RouteEvent {
    pub step: StepStamp, pub effect_index: u32, pub kind: RouteEventKind,
}
pub struct RouteKernel { /* 私有state与instance token */ }
pub struct RoutePlan { /* 私有、不可Clone、不可修改 */ }
impl RouteKernel {
    pub fn new(config: TransportConfig) -> Result<Self, Error>;
    pub fn plan(&self, step: StepStamp, commands: &[RouteCommand]) -> Result<RoutePlan, Error>;
    pub fn commit(&mut self, plan: RoutePlan) -> Result<Vec<RouteEvent>, Error>;
    pub fn next_wakeup_us(&self) -> Option<Tick>;
    pub fn revision(&self) -> u64;
    pub fn config_hash(&self) -> Id;
    pub fn snapshot_bytes(&self) -> Result<Vec<u8>, Error>;
    pub fn from_snapshot(config: TransportConfig, bytes: &[u8]) -> Result<Self, Error>;
}
impl Clone for RouteKernel { /* deep-copy state，新建私有instance token */ }
impl RoutePlan {
    pub fn base_revision(&self) -> u64;
    pub fn candidate_snapshot_bytes(&self) -> Result<Vec<u8>, Error>;
}
```

normalize排序：endpoints按Endpoint，channels/permits按ChannelId，contexts按Id，offline按start，faults按ordinal，rules按selector `(Any<Kind<Exact,kind/endpoint,channel Any<Exact)`；重复任何key拒绝。rule相同selector也拒绝。最高匹配优先级与有权限route展开按合同§3，不由rules排序决定胜者。config编码上限1MiB；全配置offline interval总数≤1024、fault entry总数≤1024、permit context引用总数≤4096；该三界固定不可增。所有Limits>0且≤v1 maxima，profile.capacity>0且≤limits.link_copies，rate>0，jitter<u64::MAX。

同logical ID的last_attempt表有限且不驱逐；Send要求attempt>last_seen，未知ID默认last_seen=0，满8192后新ID返回ResourceLimit整批无变化。packet_seq给原copy及duplicate各一个，按接受命令顺序checked分配；copy_index=0/1。fault/drop/overflow仍记录attempt和send ordinal，避免重试伪装首次发送；非法command则这些计数全部回滚。

`Drain`按合同每次选候选、释放/丢弃后重算。FIFO只把队首参与全局候选，seed tie永不逆转同linkFIFO。eligible_tick是该copy本次按raw_due/drain/offline计算的时间；实际delivery始终是显式step.tick，不能由wall决定。允许late Drain，记录eligible与delivered不同；`next_wakeup_us=max(last_committed_tick,min(candidate_eligible))`。step.logical_step_id须>最近提交step，event_id无需递增；同step可有多个Send/一个或多个Drain但仅一次plan commit。No-op commit也checked推进revision，以拒绝并发过期plan。

每plan commands≤4096、events≤8192，candidate snapshot与完整events canonical数组字节数之和≤32MiB；超界ResourceLimit，不改变原kernel，事件累积时即检查预算。effect_index是全部输出从0连续的ordinal。immediate Fault或Send时offline Drop只分配copy0并输出一条raw_due=None的Dropped，不生成duplicate；其他overflow、Drain offline与StaleBinding均保留Some(raw_due)。Unordered的tie使用同一SIMT/tie/v1 packet tie。初始send_ordinal=0、next_release_us=0、revision=0、next_packet_seq=1。

Hold模式late Drain时还须检查实际step.tick是否在线：原eligible在线但当前tick处于后续offline区间时继续hold，next_wakeup对max(last_committed_tick,eligible)再次应用online调整。origin.step_id仅源端root坐标，不与Route StepStamp.logical_step_id比较；只要求step_id≥1、sent_tick≤当前tick、origin.source_incarnation≤Envelope.source_incarnation。上述是v1接线澄清，不改变消息hash或公开字段。

## 3. ReliableMailbox

```rust
pub struct ReliableLimits {
    pub outbox_records: u32, pub inbox_records: u32, pub retained_payload_bytes: u64,
}
impl Default for ReliableLimits { /* 1024,1024,4194304 */ }
pub struct ReceiveAck {
    pub run: Id, pub config: Id, pub logical_id: Id,
    pub source: Endpoint, pub destination: Endpoint, pub channel: ChannelId,
    pub context: Option<Id>, pub codec: Id, pub origin: Origin, pub payload_hash: Id,
    pub receiver_incarnation: u64, pub custody_transition_id: Id, pub storage_receipt: Id,
}
pub enum MailboxAction {
    Enqueue { message: Message, now_us: Tick },
    Attempt { logical_id: Id, now_us: Tick, destination: EndpointBinding },
    Receive {
        envelope: Envelope, received_tick_us: Tick, received_local_us: Option<LocalTime>,
        authenticated_source: EndpointBinding,
    },
    MarkConsumed { logical_id: Id },
    Acknowledge { ack: ReceiveAck, authenticated_receiver: EndpointBinding },
    Rebind { expected_incarnation: u64, new_incarnation: u64 },
}
pub struct DurabilityReceipt {
    pub owner: Endpoint, pub base_revision: u64,
    pub transition_id: Id, pub candidate_record_hash: Id, pub storage_receipt: Id,
}
pub struct InboxItem {
    pub message: Message, pub first_receive_tick_us: Tick,
    pub first_receive_local_us: Option<LocalTime>, pub consumed: bool,
}
pub enum CustodyRelease {
    Enqueued { logical_id: Id, retry_due_us: Tick },
    Send(Envelope), ReceiveAck(ReceiveAck), InboxAvailable { logical_id: Id },
    Acknowledged { logical_id: Id }, Rebound(EndpointBinding),
}
pub struct CommittedCustody {
    pub transition_id: Id, pub revision: u64, pub releases: Vec<CustodyRelease>,
}
pub struct ReliableMailbox { /* 私有 */ }
pub struct CustodyPlan { /* 私有、不可Clone、不可修改 */ }
impl ReliableMailbox {
    pub fn new(config: TransportConfig, owner: Endpoint,
        limits: ReliableLimits, retry_interval_us: u64) -> Result<Self, Error>;
    pub fn plan(&self, action: MailboxAction) -> Result<CustodyPlan, Error>;
    pub fn confirm(&mut self, plan: CustodyPlan,
        receipt: DurabilityReceipt) -> Result<CommittedCustody, Error>;
    pub fn pending_inbox(&self) -> Vec<InboxItem>;
    pub fn retry_due_us(&self, logical_id: Id) -> Option<Tick>;
    pub fn revision(&self) -> u64;
    pub fn restore(config: TransportConfig, record: &[u8], receipt: DurabilityReceipt)
        -> Result<(Self, CommittedCustody), Error>;
}
impl Clone for ReliableMailbox { /* deep-copy，新建instance token */ }
impl CustodyPlan {
    pub fn owner(&self) -> Endpoint;
    pub fn base_revision(&self) -> u64;
    pub fn transition_id(&self) -> Id;
    pub fn candidate_record_hash(&self) -> Id;
    pub fn persistence_bytes(&self) -> &[u8];
}
```

Mailbox不推进全局时间，action.now/received_tick由controller给出；per-owner的last_action_tick只由Enqueue/Attempt的now推进且不得倒退。Receive保存原始收包位置，不以它推进/回退该watermark；该位置不晚于当前外层恢复位置由01 adapter约束，重复接收仍保留旧首次值。Enqueue的origin.sent_tick≤now；Attempt now≥retry_due，destination.endpoint必须为原message.destination、incarnation≥1；路由接线时还必须核对它的current registration。Receive要求目的current owner binding、authenticated_source与envelope source/current incarnation相同；Acknowledge要求authenticated_receiver对应原destination及ack.receiver_incarnation。两者“authenticated”参数是可信adapter声明，不是库自己进行认证；入站前必须经真实路由/认证/恢复授权。没有`durable=true`或`signature_valid=true`的生产证明开关。

可靠记录只接受ChannelSpec为Reliable、相应发送/接收权限与context合法。Receive新ID的两个release顺序固定为InboxAvailable、ReceiveAck；duplicate相同message只ReceiveAck。Enqueue/ACK遇到tombstone保持已确认状态，零新的业务发送；不同header同ID拒绝。MarkConsumed只改consumed，无release；再次MarkConsumed幂等。pending_inbox按logical_id排序，通知可以重放，库只保证一次插入，不声称任意业务副作用一次。

duplicate identical Enqueue与matching tombstone ACK均零release；合法confirm仍revision+1，Enqueue仍遵守last_action_tick。初始last_action_tick_us=0。

active outbox的Acknowledge须last_attempt>0且last_envelope存在，否则InvalidReceipt；authenticated_receiver须匹配ACK当前binding，并且receiver_incarnation≥last_envelope.destination_incarnation，允许receiver恢复后的更高incarnation。matching tombstone ACK仍按基础header/auth检查后零release幂等，不能因last_envelope已清除而拒绝。trusted adapter仍负责实际认证与当前registry绑定。

confirm核对私有instance、base revision、receipt全部绑定；不先安装state再检查receipt。record中release intent是Enqueued、SendEnvelope、AckFor、InboxAvailable、Acknowledged、Rebound；AckFor只含logical_id。confirm/restore按record内模板和**同一receipt**生成ReceiveAck，custody_transition_id=该record.transition_id，storage_receipt=外部receipt值；message/header/receiver incarnation均来自candidate，不接受caller另填。stable release身份为 `(transition_id,release_index)`，01 adapter必须去重其root提交。

从plan到persist到confirm再到root输出提交，同owner controller独占串行。已persist但confirm返回StalePlan/CrossInstance时fail closed，将原record/receipt交01恢复；不得删掉“失败计划”并忽略已持久义务。restore返回原释放意图用于受控重放；谁已真正释放、trusted floor及root Ack归01，不由这次纯函数调用认定。

## 4. Snapshot与序列化格式

只有一种canonical JSON：serde_json compact，字段严格按本附件struct顺序，整数十进制、bytes为0..255整数数组、无空白或floats、Option用null。所有enum统一serde adjacent tag `#[serde(tag="kind", content="value", rename_all="snake_case")]`；Endpoint为`{"kind":"replica","value":0}`，Send为`{"kind":"send","value":...}`，命名字段variant的value是按字段顺序的object，unit enum为`{"kind":"fifo"}`。ChannelId按其u16整数编码。所有objects拒绝未知字段。decode后reencode必须逐byte一致；canonical bytes不依赖HashMap/OS/内部token。

以下为私有snapshot schema的**字段顺序**，Builder不能暗加字段；如必须改须修订版本/独立向量。列表全部按注明key排序，不编码运行时map。

```text
RouteSnapshot {
  schema=1, kernel_config_hash, revision, last_step:Option<StepStamp>,
  next_packet_seq, bindings:[EndpointBinding sorted endpoint],
  attempts:[{logical_id,last_attempt} sorted logical_id],
  links:[{source,destination,channel,send_ordinal,next_release_us,
          queue:[{envelope,packet,raw_due_us} in insertion order]} sorted route]
}
MessageHeader {
  run,config,source,destination,channel,context,codec,origin,payload_hash,logical_id
}
MailboxSnapshot {
  schema=1,kernel_config_hash,owner,incarnation,revision,limits,retry_interval_us,
  last_action_tick_us,
  outbox:[{header,payload:Option<Vec<u8>>,last_attempt,retry_due_us,
           last_envelope:Option<Envelope>,acknowledged:bool} sorted logical_id],
  inbox:[InboxItem sorted message.logical_id]
}
PersistenceRecord {
  schema=1,owner,base_revision,transition_id,action:MailboxAction,
  candidate:MailboxSnapshot,release_intents:[ReleaseIntent in output order]
}
ReleaseIntent = Enqueued{logical_id,retry_due_us} | SendEnvelope{envelope}
  | AckFor{logical_id} | InboxAvailable{logical_id} | Acknowledged{logical_id}
  | Rebound{binding}
```

RouteSnapshot next_packet_seq是下一未使用编号，初始1。revision初始0；成功commit候选revision=base+1。links包含所有已展开可用route（空queue也保留），bindings必须与初始endpoint集合相同且incarnation≥初始值。attempts按tracked_messages限额；队列packet_seq唯一且<next_packet_seq，payload/hash/binding的原身份与当前binding独立校验（queued旧incarnation允许保存，Drain会drop），queue中attempt≤tracked last_attempt。

MailboxSnapshot acknowledged=true时payload=None、last_envelope=None，header永留；retry_due保留历史值但不再可调度。active必须payload=Some且hash正确，last_attempt=0时last_envelope=None，>0时last_envelope对应last_attempt及同message；inbox首收包/consumed保留。容量计算outbox active payload+inbox payload，总数含tombstone/consumed；恢复不能以删历史记录满足限额。PersistenceRecord candidate.revision=base_revision+1；owner/config/transition/action/release引用一致，未知record拒绝。

RouteSnapshot/ClockSnapshot/PersistenceRecord最大32MiB，先检查字节上限再deserialize，再核对schema/config/canonical/唯一性/排序/摘要/计数/容量。new/plan也须在返回成功前检查其candidate编码上限，不能允许commit成功后才发现无法保存snapshot。restore检查内部一致性和外部receipt字段匹配；完整历史真实性、old snapshot rollback/trusted floor和真实文件损坏策略归01。receipt没有放进自身所证明的record，避免循环hash。

## 5. Clock纯算法

```rust
pub struct ClockIdentity {
    pub run: Id, pub config: Id, pub endpoint: Endpoint, pub incarnation: u64,
    pub domain: Id, pub epsilon_us: i64,
}
pub struct ClockSample {
    pub run: Id, pub config: Id, pub endpoint: Endpoint, pub incarnation: u64,
    pub domain: Id, pub rate_profile: Id, pub request_id: u64, pub sequence: u64,
    pub as_of_tick_us: Tick, pub local_at_as_of_us: LocalTime,
    pub uncertainty_us: u64, pub lease_end_tick_us: Tick,
}
pub struct ClockView { /* 私有字段，只有ClockTracker生成 */ }
impl ClockView {
    pub fn reference_tick_us(&self) -> Tick;
    pub fn local_time_us(&self) -> LocalTime;
    pub fn lease_end_tick_us(&self) -> Tick;
    pub fn sample_sequence(&self) -> u64;
    pub fn local_deadline_tick_us(&self, absolute_local_us: LocalTime) -> Result<Tick, Error>;
}
pub struct ClockTracker { /* 私有 */ }
impl ClockTracker {
    pub fn new(identity: ClockIdentity) -> Result<Self, Error>;
    pub fn request(&mut self) -> Result<u64, Error>;
    pub fn install(&mut self, delivery_tick_us: Tick, sample: ClockSample) -> Result<(), Error>;
    pub fn view(&self, tick_us: Tick) -> Result<ClockView, Error>;
    pub fn rebind(&mut self, expected_incarnation: u64, new_incarnation: u64) -> Result<(), Error>;
    pub fn snapshot_bytes(&self) -> Result<Vec<u8>, Error>;
    pub fn from_snapshot(identity: ClockIdentity, bytes: &[u8]) -> Result<Self, Error>;
}
```

ClockIdentity epsilon在[-1000,1000]，投影用i128计算`tick+epsilon`后checked转i64。request从1起；最多64 outstanding requests，达到上限ResourceLimit且counter不变。install只接受outstanding且未接受的request、sequence>watermark、相同identity/domain/rate、as_of≤delivery<lease_end、uncertainty≥abs(epsilon)且≤1000、local_at_as_of=as_of+epsilon，并要求delivery不早于上次成功install位置；全检查后一次更新，invalid seq100不毒化valid seq3。view须已校准、tick≥sample delivery且tick<lease_end，投影本tick而非直接使用旧sample值；只读，不续租，不是admin advance。

local_deadline_tick_us计算`max(0,absolute_local_us-epsilon)`，i128中间值后checked转u64，不把deadline改成调用时刻+duration。超过lease的deadline仍可换算，但是否安全触发由外层timer/lease guard决定。rebind只old+1，保留request counter/accepted watermark/prior lease end，清sample和旧incarnation outstanding requests（不能以旧request接受新回复）；下个request ID继续递增。旧cid terminal不在本对象，新sample不能为它发恢复许可证。

ClockSnapshot字段顺序：`schema=1,identity,request_counter,outstanding:[u64 sorted],accepted_sequence,prior_lease_end_tick_us,sample:Option<ClockSample>,sample_delivery_tick_us:Option<u64>`；snapshot不证明源权限。ServiceGrant可携带可信服务的reference tick，BFT应用适配仍只暴露同节点local clock，不给协议算法全局fault脚本。

sample_delivery_tick_us同时是最后一次成功install的反倒退水位，rebind清sample时保留它；sample=None允许delivery=Some，sample=Some必须有delivery。新install不得早于此水位，view仍要求存在可用sample。restore只证明结构、receipt/action/release引用与局部不变量；缺少prior state不能证明完整历史transition，完整历史与trusted floor仍归01。

## 6. Service/reader共同消费面

```rust
pub enum ServiceInput {
    Message(Delivery),
    Timer { timer_id: Id, absolute_local_deadline_us: LocalTime },
    External { kind: u32, bytes: Vec<u8> },
}
pub struct ServiceGrant {
    pub target: EndpointBinding, pub step: StepStamp,
    pub clock: ClockView, pub input: ServiceInput,
}
pub enum ServiceEffect {
    Send(Message),
    ArmTimer { timer_id: Id, absolute_local_deadline_us: LocalTime },
    CancelTimer { timer_id: Id },
    Work { work_id: Id, duration_us: u64, input: Vec<u8> },
}
pub trait GrantConsumer {
    type Error;
    fn on_grant(&mut self, grant: ServiceGrant) -> Result<Vec<ServiceEffect>, Self::Error>;
}
```

06 Publisher/SimEspresso使用`Service(6)`示例，09 L1 driver使用`Service(9)`，两个读者使用`Reader(1/2)`；这些数值仅API使用示例，由运行配置注册，非保留全局ID。06在on_grant中解释opaque出版/日志bytes和自己的q/θ；09可用reference_tick_us及其冻结timestamp policy产生整数秒L1输入；02不替其取整、造finalized或执行EVM。Reader(1)收到Message不改变Reader(2)；reader解释公共position、gap/prefix和GFR由06/03/07实现。纯kernel不提供`reader_prefix()`。

adapter调用次序：从唯一controller获真实grant→核对target/step/ClockView→调用同GrantConsumer（fixture也相同）→将Send/Timer/Work意图纳入该root Ack→root确认后交RouteKernel或ReliableMailbox流程。ServiceEffect没有Advance/Step/AdminFault；consumer不能自行推进时间。函数直接返回effects只说明计算完毕，不证明已受控、持久或对网络发布。

## 7. 错误与结果分类

```rust
pub enum Error {
    InvalidConfig, UnsupportedSchema, DuplicateId, UnknownEndpoint, UnknownChannel,
    MissingProfile, AmbiguousProfile, PermissionDenied, ContextMismatch, BindingMismatch,
    HashMismatch, InvalidOrigin, InvalidAttempt, SelfRoute, PayloadTooLarge,
    ResourceLimit, ArithmeticOverflow, TimeRegression, StepRegression,
    CrossInstance, ConfigMismatch, StalePlan, InvalidReceipt, UnknownMessage,
    MessageConflict, RetryNotDue, NonCanonical, SnapshotTooLarge, CorruptSnapshot,
    Uncalibrated, InvalidSample, StaleSample, LeaseExpired,
}
```

错误优先级：语法/schema/配置→大小/算术→binding/权限/context→hash/身份→业务前态；同时多错的恶意输入只要求落在相应错误集合且原子拒绝，独立向量单字段变异检查精确variant。具体层级固定：wrongrun/config为ConfigMismatch，旧incarnation为BindingMismatch，sameID变异为HashMismatch或MessageConflict（先hash失败则HashMismatch），重复/倒退attempt为InvalidAttempt，已知ACK同ID但header不符为MessageConflict，wrongreceipt为InvalidReceipt，snapshot格式/局部不变量错误为CorruptSnapshot，非canonical合法JSON为NonCanonical。

Route overflow是成功plan的Dropped事件，不是Error；新tracked ID耗尽、mailbox历史容量耗尽是ResourceLimit整plan不提交。所有错误不释放protocol outputs。Clock view到lease equality返回LeaseExpired；install expired/future/profile错误为InvalidSample，低/重复accepted sequence为StaleSample；两者不修改校准/水位/request。实际协议失败分类、root durable失败处理、panic/watchdog分类由集成adapter承担，不能把组件Error自动宣称协议crash或安全反例。

本接口稿发布可供06/09开发同trait fixture；待独立kernel候选验证后发布可消费代码SHA。真实01/03/native接线、服务持久事务与完整02验收仍另记，组件API/测试不得代替它们。
