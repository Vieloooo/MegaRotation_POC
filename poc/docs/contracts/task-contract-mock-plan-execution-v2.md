# 功能03实现合同 v2：公共接口、Mock Plan 与轻量执行

日期：2026-11-15。状态：**M1/M2 CONTRACT FROZEN；IMPLEMENTATION AUTHORIZED**。功能03 lead已批准本合同§2–6字段、工程选择、bounds与职责边界，独立Verifier先固定本BE profile期望再编码。[总协调dependency-plan-v2](../commander/dependency-plan-v2.md) 删除对01完整验收的前置要求；本合同取代[v1准备合同](task-contract-mock-plan-execution-v1.md)的统一等待gate，不改v1历史或黄金LE输入。整体03仍须独立复核和相同候选SHA测试，组件接口通过不计功能accepted。

## 1. 权威、阶段和所有权

语义继续采用v1所列S-V2/S-A/S-AUTH/S-SP/S-GFR/S-CW来源及精确版本：v2 SHA-256 `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`，Regional BFT pin `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，Commonware 2026.10.0。准备提交 `183430e71e5d61533f54a76b2d7d22c5021f1704` 与新依赖计划的正常merge保留在历史；当前工作的精确基线和候选由lead交付记录给出。

| 阶段 | 冻结与实现范围 | 验收／消费者 |
| --- | --- | --- |
| M1 | [公共接口v1](mock-plan-public-interface-v1.md)：独立domain模块、身份、Hash32/Cid/ConfigRef、L1位置/引用与ResultKey；旧I1保持兼容。 | 独立固定bytes/errors/映射与hash_parts兼容；先发布准确接口SHA供02/06/09。 |
| M2-A | 本合同§2–4的对象、registry/PoP、Plan/leader、descriptor/身份。 | 真实公钥/PoP、独立生产codec与cid向量；无假最终证明验证。 |
| M2-B | §5–6的执行、metadata/DEP、Merge、native hash与受限L1 gateway。 | 固定转账/过滤/padding/缺输入/双reader向量；D与G共享纯执行，不共享链状态。 |
| M2集成 | 同候选的组件组合、错误原子性、所有适用回归。 | Verifier通过后Tester按同SHA运行；不声称04/15-worker、01恢复或09真实Anvil完成。 |

03 lead唯一写 `poc/crates/types/`、`poc/Cargo.toml`、`poc/Cargo.lock`、core模块声明与集成。一个Builder独占分配的 `poc/crates/authorization/`、`poc/crates/core/src/{execution,metadata,merge}/` 和具体L1 fixture模块；lead先完成必要types和manifest再移交源码写权，不能同时有两个生产源码writer。Docer只写本合同、公共接口文档和准备报告；Verifier只写独立verification路径。02/06/09的目录、01 native恢复文件、Regional BFT子模块和总协调登记均不由本任务修改。

## 2. 公共数据结构与验证层

M1的 `Canonical` 不修改。M2具有公开字段的结构使用另一trait：

```rust
pub trait CanonicalObject: Sized {
    fn encode_checked(&self) -> Result<Vec<u8>, DomainError>;
    fn decode(bytes: &[u8]) -> Result<Self, DomainError>;
}
```

每种对象都有结构合法性检查；`encode_checked`拒绝无效表示，`decode`拒绝未知version/tag、超界count/length、重复项、非规范顺序、截断及尾随bytes。所有公开digest计算必须先调用该对象的checked encoding，不能提供对非法对象静默hash的捷径。涉及外部配置、来源、BLS的验证必须显式另外调用 `validate_with` 或进入validated类型；codec通过不冒充外部证明通过。

下表字段顺序同时是M2 raw字段的规范顺序。Hash/Cid/ConfigRef、u8身份嵌套为M1 raw，嵌套结构不重复version。生产全为BE；`timestamp_us`为协议／仿真整数微秒，L1 header的`timestamp_s`为EVM整数秒。

| 类型 | 字段（依顺序）与约束 |
| --- | --- |
| `SessionId` | `beta:u64, r:u64`。 |
| `Window` | `h_min:u64, h_max:u64`；checked `h_max-h_min+1=10`，与配置核对时另查`h_min>genesis.height`。 |
| `Metadata` | `height:u64, timestamp_us:u64, origin:u64`。 |
| `BlockRef` | `height:u64, origin:u64, block_id:Hash32`。 |
| `Anchor` | `block:BlockRef, timestamp_us:u64`；不含执行状态或可变证明。 |
| `Account` | `nonce:u64, balance:u64`。 |
| `Transfer` | `from:u8, to:u8, nonce:u64, amount:u64`；`encode_raw()->Vec<u8>`及严格`decode_raw(&[u8])->Result<Self,DomainError>`使用§5.1的19B格式，不实现带version的CanonicalObject。 |
| `Credit` | `to:u8, amount:u64`；同上raw API，使用§5.1的10B格式。 |
| `Attributes` | `metadata:Metadata`；同上raw API，使用§5.1的25B格式。 |
| `ExecutionState` | `metadata:Metadata, accounts:sorted map<u8,Account>`；最多256项，id严格升序；只保存nonce/balance至少一个非零的项。 |
| `ExecutionAnchor` | `anchor:Anchor, state:ExecutionState, native_hash:Hash32`；metadata必须等于anchor实际height/time/origin。它是执行材料，不进Descriptor身份。 |
| `RegistryEntry` | `pid:Pid, identity_key:[u8;32], vote_key:[u8;96], pop:[u8;48]`；weight固定1，region/member从pid推出。 |
| `RegistryRecord` | `entries:Vec<RegistryEntry>`；恰15项，canonical按pid0..14。导入允许先对乱序输入排序，但规范decode不得接受乱序bytes。 |
| `CommitteeEntry` | `pid:Pid, identity_key:[u8;32], vote_key:[u8;96], weight:u64`；weight恰1，不复制PoP，PoP由已验证Registry保证。 |
| `RuleSpec` | `rule_id:u16, version:u16, params:Vec<u8>`；本版只接受`(1,1,[])`，对应`static-schedule-v1`，不静默接受其他policy。 |
| `PublicConfig` | `chain_id:u64, rule_version:u16, verifier_version:u16, registry_hash:Hash32, dep_fixture_hash:Hash32, l1_fixture_hash:Hash32, genesis:Anchor, genesis_state_root:Hash32, genesis_native_hash:Hash32, l2_slot_us:u64, l1_genesis_timestamp_us:u64, l1_block_us:u64, t_dep_us:u64, t_drift_us:u64, metadata_window_us:u64, byte_limit:u32`。 |
| `PlanInput` | `cause:BoundaryCause, session:SessionId, source_region:Option<RegionId>, window:Window, reference_block:BlockRef`；不含production anchor，允许regular提前Plan。 |
| `Boundary` | `input:PlanInput, anchor:Option<Anchor>`；这是可用输入骨架，不替代GFR未来未确定分量的完整状态机。 |
| `PlanRecord` | `session:SessionId, cause:BoundaryCause, source_region:Option<RegionId>, region:RegionId, committee:Vec<CommitteeEntry>, total_weight:u64, fault_bound:u64, eligible_proposers:Vec<Pid>, leader_rule:RuleSpec, window:Window, reference_block:BlockRef, config_ref:ConfigRef, authorization_ref:Hash32`。 |
| `ProgramConfig` | `chain_id:u64, rule_version:u16, verifier_version:u16, config_ref:ConfigRef, committee:Vec<CommitteeEntry>, fault_bound:u64, eligible_proposers:Vec<Pid>, leader_rule:RuleSpec, window:Window, byte_limit:u32, metadata_window_us:u64`。 |
| `Descriptor` | `plan:PlanRecord, anchor:Anchor, cid:Cid`；anchor height小于window.h_min；必须经authorization层`Provider.validate_descriptor`重算Plan/ProgramConfig/cid。types不依赖authorization。 |
| `PhysicalBlock` | `parent:Hash32, metadata:Metadata, pre_state_root:Hash32, transactions:Vec<Vec<u8>>`；parent是实际physical/anchor id，raw普通交易保留原顺序。 |
| `NativeBlock` | `parent:Hash32, metadata:Metadata, dep:Vec<Vec<u8>>, kept:Vec<Vec<u8>>, post_state_root:Hash32`；parent是实际连续native parent，不能换成physical parent。 |

`BoundaryCause`的wire tag固定为GENESIS=0、HANDOFF=1、FORCE=2、RESUME=3。本tag不用于公共事件的排序；尤其不把Force/RESUME按tag分组。Plan的committee恰为selected region的5项，pid升序；total_weight=5，fault_bound=1；eligible_proposers恰这5个pid且升序。ProgramConfig的对应字段必须逐项等于Plan授权。`PublicConfig`本版rule/verifier version均1；它只是03标识，04未接原生checker前不能据此声称“verifier已实现”。

Config校验须保证slot和L1间隔非零、`l2_slot_us≤l1_block_us`、`t_dep_us+l1_block_us≤t_drift_us`、`byte_limit∈1..=65536`；所有求和/乘法checked。`t_dep`是否覆盖实际L1读者延迟为场景活性前提而非密码学事实。`StaticScheduleProvider::new(config,registry)`只核对config结构与实际已验证registry hash，不可能核对未传入的state/fixture。core的独立 `check_setup(config,ExecutionAnchor,DepFixture,L1Fixture)` 必须核对完整genesis anchor、实际state root、genesis native hash和两fixture hash后才允许执行／fixture实例进入Ready，不能仅检查长度。未知JSON字段拒绝，包括试图在RegistryEntry注入weight；keys为固定长度arrays，可用内部Serde helper保持无额外依赖，不能放宽长度。

### 2.1 Codec与bounds

每个顶层M2 canonical object为 `u16BE(1)||raw_fields`。raw整数按字段宽度BE；Option为`u8(0)`或`u8(1)||raw(value)`；Vec为`u32BE(count)`，固定长元素直接raw串接，变长字节元素逐个`u32BE(byte_len)||bytes`；嵌套变长struct按其已知schema顺序解析，不额外叠长度。稀疏accounts编码为`u32BE(count)`后重复`u8(id)||u64BE(nonce)||u64BE(balance)`。

具体上界：registry15、committee5、eligible5、accounts256、普通交易1024条且raw bytes总和≤`min(config.byte_limit,65536)`、每height credits≤1024（加首个Attr后DEP≤1025项）、规则params本版长度0。后续fixture/chain bounds见§6。解码必须在分配前检查count/length、用checked累加总bytes；不仅检查单个交易长度。

### 2.2 下游typed骨架（本版不提供Canonical）

以下对象保持v10字段含义，提供同一组公共Rust值类型以避免02/06/09各自定义。它们**没有本版CanonicalObject、hash或最终验证实现**；完整证书域/编码归04，重放/lifecycle codec归05/07，ABI和历史派生切面归09/12/13。Rust enum变体名不是线上tag，也不能被Serde输出冒充canonical bytes。

| 类型 | 精确typed字段／变体 |
| --- | --- |
| `ReplayKind` | `Force, Resume, Expire, SessionEnd, Espresso`。 |
| `ReplayIndex` | `L1(L1Position), Height(u64), Session(SessionId), Espresso(u64)`；前两控制kind对应L1、Expire对应Height、SessionEnd对应Session、Espresso对应Espresso，错配拒绝。 |
| `ReplayPosition` | `key_us:u64, kind:ReplayKind, index:ReplayIndex`；禁止按enum声明次序派生协议Ord。07按v10同key控制混排→Expire→SessionEnd→Espresso实现排序。 |
| `AnchorEvidenceSource` | `Genesis(ConfigRef), Regional(ResultKey), Force(ReplayPosition), Resume(L1Position)`；Force来源position.kind须Force，Resume精确到tx/log。 |
| `AnchorEvidence` | `source:AnchorEvidenceSource, source_objects:Vec<Vec<u8>>`；source_objects是尚待相应verifier解析的原始公共对象，不是已验证标记；`kind()`从source派生，不能另传不一致kind。 |
| `RegionalResult` | `descriptor:Descriptor, block:PhysicalBlock, final_proof:Vec<u8>, chain:Vec<PhysicalBlock>, anchor_evidence:AnchorEvidence`；chain为anchor之后到目标的完整physical序列，首ph指anchor、末端等于block；无proof verifier或缺材料不能标valid。 |
| `VirtualBlock` | `metadata:Metadata, dep:Vec<Vec<u8>>, pre_state_root:Hash32, transactions:Vec<Vec<u8>>`；transactions保留raw输入，不是Kept。 |
| `CloseReason` | `Time, Complete, Force, Resume, Expired, Excluded, Incompatible`，逐项对应v10 Manager §3.2，不冻结数值tag。 |
| `LifecycleBody` | `SessionAssigned{descriptor:Descriptor}`、`SessionClosed{session:SessionId,reason:CloseReason}`、`BranchClosed{branch:u64,reason:CloseReason}`、`Resumed{control_seq:u64,branch:u64,anchor:Anchor,derived_prefix:Vec<VirtualBlock>}`。 |
| `LifecycleEvent` | `position:ReplayPosition, body:LifecycleBody`；`kind()`由body派生，等价于v10 position/kind/body记录，避免冗余kind错配；同position不同body保持不同事件。 |

RegionalResult/AnchorEvidence的公开内容按消费者已冻结的大小限制接收，03不为这些未编码对象发明新的公共签名格式。外层材料不足用typed WAIT；本版bool FinalVerifier不把“尚无适配器”变成true。上表是接口骨架，不代表可绕过04/07/09验证接纳来源。

## 3. hash/domain与上下文

生产hash统一复用旧 `hash_parts`：对ASCII域名和唯一canonical bytes分别加u64BE长度后SHA-256。它不修改I1行为。返回新Hash32时校验最终长度32。下列大小写是协议的一部分：

| 值 | 域与唯一preimage |
| --- | --- |
| registry hash | `PoC/Registry/v1`，RegistryRecord canonical bytes（经真实验证的registry导出）。 |
| ConfigRef | `PoC/PublicConfig/v1`，PublicConfig canonical bytes。 |
| state root | `PoC/ExecutionState/v1`，ExecutionState canonical bytes。 |
| physical digest | `PoC/PhysicalBlock/v1`，PhysicalBlock canonical bytes。 |
| native hash | `PoC/NativeBlock/v1`，NativeBlock canonical bytes。 |
| dep fixture hash | `PoC/DepFixture/v1`，§6 DepFixture canonical bytes。 |
| L1 fixture hash | `PoC/L1Fixture/v1`，§6 L1Fixture canonical bytes。 |
| authorization_ref | `PoC/Authorization/v1`，`u16BE(1)||raw(config_ref)||raw(registry_hash)||raw(PlanInput)`。 |
| cid | `PoC/Regional/v1`，`u16BE(1)||raw(session)||raw(anchor.block.block_id)||raw(ProgramConfig)`。 |
| derived anchor id | `PoC/DerivedAnchor/v1`，`u16BE(1)||u64BE(height)||raw(native_hash)`。 |

`PoC/Regional/v1`是v10 `H(REGIONAL,session,id(A),ProgramConfig)`的本PoC版本化映射，`PoC/DerivedAnchor/v1`对应v10 DERIVED。cid不额外hash整个Descriptor/Plan、不含Anchor state、arrival/reader/route/incarnation/clock；外部验证仍须检查anchor metadata/state。authorization_ref不含production anchor，且不能作为来源证明本身。Registry/fixture/config的hash依赖无环：registry、dep和L1 fixture不嵌config_ref；config引用它们，Plan引用config。

physical hash承诺raw Txs，native hash承诺实际DEP/Kept/post-root和实际parent。不同raw invalid交易可以产生不同physical hash但相同native hash；view/cid/proof不加入native块。旧LE proposal完全保留为历史，不作为生产bytes或黄金期望；Verifier新写本BE profile的固定字节/摘要。

## 4. Registry、Plan和公共证明边界

### 4.1 真实Registry/PoP

identity用Commonware `ed25519::PublicKey`，另加本任务批准的严格工程gate：固定 `curve25519-dalek=5.0.0`，`CompressedEdwardsY(raw).decompress()`成功、重新compress等于raw、`!is_small_order()`且`is_torsion_free()`，随后才调用Commonware PublicKey decode。这个严格gate不是Commonware自带行为：上游Ed25519采用ZIP215兼容decode，只做可解压检查，不能把其decode成功误称为canonical/无小阶/无混阶保证。地区内排序仍按最终decoded identity的原规范公钥bytes。

vote用BLS12-381 MinSig的G2 public 96B、PoP用G1 signature48B；固定版本原生BLS decode检查群/子群并拒绝infinity，真实调用 `ops::verify_proof_of_possession::<MinSig>(public,b"PoC/Registry/PoP/v1",pop)`。不能只验证长度。错误namespace/key/proof、重复identity/vote key/pid、缺成员或非法pid均拒绝，失败不返回部分ValidatedRegistry。

静态公开genesis绑定 `(pid,identity_key,vote_key,pop)`；本版不另造identity cross-signature要求。PoP只证明BLS私钥占有，不冒充transport认证或Plan来源。各地区signer index按decoded identity公钥排序，保持pid/member↔identity↔signer双向表；BLS vote公钥不决定index。Commonware multisig构造器本身不检查PoP，因此只有ValidatedRegistry可向04导出participant map。

确定性测试key允许固定seed生成，track只放public keys、PoP和生成recipe/seed，不能存私钥字节。用真实PoP阳性和错key/namespace/bit mutation负例验证密码路径。没有把旧脚本signer或bool PoP flag计为本任务通过。

### 4.2 Plan/provider接口

```text
ValidatedRegistry::load(RegistryRecord) -> Result<ValidatedRegistry, RegistryError>
StaticScheduleProvider::new(PublicConfig, ValidatedRegistry) -> Result<Provider, ConfigError>
Provider.resolve(PlanInput, PublicReferenceView) -> PlanOutcome
Provider.validate_plan(PlanRecord, PublicReferenceView) -> PlanOutcome
Provider.leader(PlanRecord, view:u64) -> Result<Pid, PlanError>
Provider.descriptor(PlanRecord, Anchor) -> Result<Descriptor, PlanError>
Provider.validate_descriptor(Descriptor) -> Result<(), PlanError>
```

`PublicReferenceView`是只读能力：按完整BlockRef查询已验证来源，返回`Ready(Anchor)`、`Wait(MissingReference)`或`Reject(InvalidSource)`；Ready时完整height/origin/id必须相等。实现可以用固定来源fixture或09/07未来adapter，不使用无上下文`reference_valid=true`当生产验证。PlanOutcome为`Ready(PlanRecord)|Wait(PlanMissing)|None|Reject(PlanError)`；默认完整静态registry不会None，离线/启动顺序不参与Plan。未知rule/config、坏window/算术或来源冲突Reject；缺来源Wait，不伪造空Plan。

Schedule按`((beta%3)+(r%3))%3`及`(view-1)%5`，view0拒绝；合法u64 beta/r不因直接相加overflow出错。Force使用GFR激活时实际新session，provider不决定其r。Window宽10、不因anchor和Enter改变；regular reference仍是前驱初始anchor，production anchor可为h9但后继window为11–20。

公共 `FinalVerifier` 可冻结为无默认实现的能力：`check(&Descriptor,&PhysicalBlock,proof:&[u8])->bool`。实际native subject/namespace/epoch/proof解码属于04；本任务不提供always-true或把样例bool计为验签。`ResultKey`供06索引；完整 `AnchorEvidence`、`RegionalResult`、`ReplayPosition`、`LifecycleEvent`沿v1 typed骨架保留消费职责，03本轮不冻结其完整codec/ABI。需要发布结果的组件必须经04验证能力与完整来源/链检查；03的纯对象hash不替代这些检查。

Provider持有已经核对的PublicConfig/ValidatedRegistry；`validate_descriptor`置于authorization或其extension trait，不能使types反向依赖authorization而产生crate循环。无外部config/registry的Descriptor decode只验证结构，不声称cid或Plan来源已经验证。

## 5. 轻量执行、metadata和Merge

### 5.1 已批准交易与账户规则

普通Transfer raw为`01||u8(from)||u8(to)||u64BE(nonce)||u64BE(amount)`，精确19B；Credit raw为`02||u8(to)||u64BE(amount)`，精确10B；Attr raw为`03||Metadata.raw`，精确25B。这些交易raw不是独立CanonicalObject，不重复两字节version；版本由包含它们的执行/块配置绑定。

账户id为u8全集，缺账户等同nonce0/balance0。State codec只保留非全零项，重复/乱序/显式全零项拒绝。`Apply(parent_state,metadata,DEP,raw_Txs)`预先严格解码全部普通交易及DEP；任何坏opcode、长度、尾随bytes或数量/bytes超限都整体Reject，不能执行部分然后忽略坏尾部。普通列表只允许Transfer，DEP恰为匹配本次metadata的首个Attr、之后零或多个Credit；重复/晚置Attr、错metadata、DEP内Transfer或普通列表内Credit/Attr都Reject。

Begin先设置实际metadata，再顺序credit；credit0合法、不产生多余全零账户。Credit溢出是输入Reject，纯函数不给出可提交的部分状态。普通Transfer在轮到它时检查sender nonce相等、nonce+1可表示、sender.balance≥amount，以及非self的receiver.balance+amount可表示；不满足任一项则跳过、状态不变、不入Kept。Self-transfer余额不变，仍要求余额≥amount且nonce有效，并只将nonce加1，不做虚假的receiver加款溢出检查。零额Transfer合法并递增nonce；未知接收方按需创建，未知发送方按同一规则判定。成功Transfer进入Kept，顺序不变，DEP不在Kept内。本版无revert opcode。

接口结果至少为 `ApplyOutput{state:ExecutionState,kept:Vec<Vec<u8>>}`。重复在同一不可变parent与输入上求值结果相同；对post-state再次执行是不同输入，不得当作幂等性测试。缓存必须绑定实际parent、配置/版本和输入，不能只按height；协议commit和grant/Ack接线仍归01/02，Apply本身不发布事件。

### 5.2 Metadata/DEP

`ts*(h)=genesis.timestamp_us+(h-genesis.height)*l2_slot_us`，所有减乘加checked。L1 row秒timestamp转换us检查乘法溢出，不混用reader arrival。scheduled origin是满足`ts_L1(ell)+t_dep_us≤ts*(h)`的最大ell；空集合不默认为0。生产origin使用v10公式`min(A.origin+h-A.height,max(A.origin,sch(h)))`，通过O1/O2/O3和anchor条件，origin输入未可见时Wait。D使用12提供的实际origin而不强制套生产schedule。

`DepProvider::at(metadata)`取配置绑定dep fixture相同height行，组成匹配实际metadata的Attr及该行Credit。显式空credits合法；缺行不等于空。真实L1按origin分配deposits不在P0验收范围。system/upgrade本版不支持，不能把未知system raw当普通无效交易跳过。

### 5.3 Merge

最小纯接口为 `merge(execution_anchor, physical_chain, window, metadata_provider, dep_provider, config) -> Ready(MergeOutput)|Wait(MissingInput)|Reject(MergeError)`。Window显式传入并按合同验证，不能从没有window字段的PublicConfig猜取；最多10个physical blocks，virtual展开最多4096 heights，超界显式LimitExceeded而不部分输出。physical chain必须完整hash相连、严格升高、各physical h在固定window，首parent等于anchor id。每physical前执行必要padding；padding的raw普通列表为空但照常Begin/DEP。physical.pre_state_root等于当前Begin之前状态的root；不是anchor root、不是post-root，也不是提前加了当前DEP的root。

输出按连续height记录metadata、pre-root、raw Txs、DEP、Kept、post-state/root、native parent/hash；每个native parent接实际上一native hash，初值取ExecutionAnchor.native_hash。只有被给定physical后继覆盖的区间可补padding，anchor-only返回空增量，不能凭时钟补到window末端。缺body/ancestor/metadata/DEP等待，已知错parent/root/metadata/格式拒绝；失败时无部分可提交MergeOutput。可保留纯计算的诊断trace，但不能把它当已交付前缀。

## 6. Dep fixture与L1 gateway

以下为03受限fixture语义，不定义09合约ABI。fixture/source本身在run manifest绑定，PublicConfig再绑定其hash；两reader只有各自实际交付的verified finalized prefix。

`DepFixture{first_height:u64, rows:Vec<Vec<Credit>>}`：每row连续对应`first_height+index`，显式空row保留，rows非空且最多4096，单row最多1024 credits，height算术checked。canonical raw为first_height、u32 row count，每row u32 credit count后串接固定10B Credit raw。未知height若超过声明范围为FixtureExhausted；来源宣称存在但丢失/损坏是FixtureError，不能生产空DEP。

L1最小共用类型／能力：

```text
L1Header { height:u64, hash:Hash32, parent_hash:Hash32, timestamp_s:u64 }
L1FixtureRow { header:L1Header, finalized_at_us:u64, empty_control_commitment:Hash32 }
L1Fixture { rows:Vec<L1FixtureRow> }
FinalizedL1Reader::header(height:u64) -> InputStatus<L1Header>
FinalizedL1Reader::require_control_coverage(through_height:u64) -> InputStatus<()>
FinalizedL1Reader::finalized_tip() -> Option<L1BlockRef>
```

共用只读结果类型为 `InputStatus<T> = Ready(T) | Wait(InputMissing) | Reject(InputError)`。`InputMissing`为`Reference(BlockRef)`、`L1Header(u64)`、`ControlCoverage(u64)`、`Parent(Hash32)`、`State(Hash32)`或`Dep(u64)`；State绑定所缺anchor/parent身份，L1 reader只使用适用的header/coverage分支，metadata缺失用具体L1Header或Dep表达，不设无约束字符串。`InputError`为`InvalidSource`、`Conflict`、`FixtureCorrupt`、`FixtureExhausted{requested:u64,first:u64,last:u64}`或`Malformed(DomainError)`。这些是Rust类型化语义，不冻结网络错误数字或错误文本；详细diagnostic可另附，不改变结果类别。来源缺失只能Wait，已确认来源冲突、codec坏输入或来源损坏只能Reject，调用者不得将Reject改写为空数据。

本L1 fixture只承诺完整**空**控制日志；非空控制行为归09，接口通过额外typed能力扩展，不能在03虚构Force/Resume/Inbox ABI。`empty_control_commitment=hash_parts("PoC/EmptyL1Control/v1",[u16BE(1)||u32BE(0)])`；这是mock来源承诺，不是真实EVM receipts证明。L1Fixture canonical为version、u32 count和各row raw；row raw按上表字段顺序，header raw是height/hash/parent/timestamp_s。默认31行heights0..30，最多4096，必须连续、parent hash相连且timestamp严格增加；秒→us和finalized_at计算checked，不能提前于header timestamp。genesis parent为固定零Hash32，公开fixture中block hashes与完整行一起受fixture hash绑定，不声称是Ethereum原生header哈希。

Fixture验证先检查source manifest/hash与全部范围，再产生不可变ValidatedFixture。header及完整control commitment是公开内容；row的finalized_at_us是本fixture声明的源调度时间，不能因为读者看到当前时间超过它就自行把block标finalized。必须先处理显式Finalize输入，验证该row/source与予定finalization条件，再产生可交付的finalized材料；独立Deliver输入携带该材料及实际delivered_at_us，不能合并两个事件。delivery时间和reader ID不进入header、control commitment、fixture hash或ConfigRef。

每个 `FixtureReader` 独立持有可见prefix和控制覆盖；共享只读source并不共享可见状态，不引入第二套ReaderId（02/06自行映射其reader对象）。只有显式delivered输入才能推进，缺Finalize材料时拒绝本次delivery而不推进前缀，delivered_at_us必须不早于该row实际finalized_at_us；out-of-order高行可暂存，直到前缀补齐才可见。重复相同行幂等，冲突行拒绝并保持原状态；finality或delivery位置不可回退。

header已交付但empty-control完整性材料未交付时，header查询可Ready，coverage查询必须Wait。磁盘未来row、已finalized未delivery、前缀gap正常Wait；known source hash/parent/row冲突为Reject/FixtureError；超出有限范围为FixtureExhausted。所有失败事件对reader状态原子；相同可见前缀两reader返回相同值。

默认REG-A fixture参数选regular准备方向：L1 timestamp=`2*ell`秒，finalized=`timestamp+1.5s`，常规delivery=`timestamp+2s`；L2 genesis h0=0/10s，origin3，slot1s，T_dep5s，T_drift15s，metadata_window200ms。这些是本组件固定输入，不构成15-worker准入时限的实测。不同reader delivery可延迟，不能改变L1 timestamp/finality/source值。

## 7. 独立验收、完成证据与未覆盖责任

沿用v1 M01–M13的语义案例，并补充本v2批准工程选择的生产BE字节及执行边缘值。M1独立expected已先行；M2每模块编码前须有独立固定输入/expected，不能用生产implementation输出生成golden。所有组件候选保存为准确提交，Verifier核对该SHA后Tester只跑同一SHA；修复后重验影响范围。

| 范围 | 必须真实检查 |
| --- | --- |
| 兼容性 | M1旧wire导出及I1 hash_parts/grant/Ack结果不变，新模块不shadow；既有适用轻量回归。 |
| Registry | 15真实identity/vote key及PoP，全部pid映射、各地区sorted signer，乱序导入canonical化、重复／坏点／错PoP/namespace拒绝。 |
| Plan/codec | 三地区排班、u64上界/view0、窗口边界、reference/anchor分离；固定BE bytes、域、config/registry/dep/L1 commitments、same-height不同anchor导致cid不同。 |
| 执行 | 成功转账、invalid nonce/balance、顺序依赖、zero/self/unknown、nonce/receiver/credit溢出、raw编码拒绝原子性、DEP顺序/metadata、state稀疏规范化。 |
| Merge/native | gap/padding pre-root、实际native parent、missing-input与错输入区分、同输入重算、不同raw invalid Txs同native结果、无后继不补尾。 |
| L1 | 双reader独立delivery、future/on-disk、finality前delivery拒绝、out-of-order、metadata可用但control覆盖缺、source损坏、范围外、重复/冲突原子性。 |

04仍负责真实Simplex FinalVerifier、证书subject/domain和3/4 signer final proof接线；03可以验证真实PoP，但不能据此报告04证书矩阵PASS。07负责ReplayPosition/Lifecycle完整编码和重放，09负责真实receipt/ABI，12/13负责位置绑定ResumeCut与D/RESUME。完整03的组件验收不声称这些任务或runtime持久化集成已完成。

Rust build、重型测试、仿真全部 `flock /tmp/megaroation-poc-heavy.lock` 串行、`CARGO_BUILD_JOBS=2`，不嵌套锁。任务 `.build` 与大artifact使用总协调分配的 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03`，迁移时保留已有内容并核对独占，不共用其他job target。Docer不运行Rust。最终交付列实际模型/角色、基线/接口/候选/验证/测试SHA、命令和seed、原始证据绝对路径/摘要、PASS/FAIL/INCONCLUSIVE与下游未覆盖责任；不合main、不推远端。
