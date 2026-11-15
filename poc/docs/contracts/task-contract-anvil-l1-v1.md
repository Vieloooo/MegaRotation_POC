# 功能09：Anvil、ControlAuthority、BatchInbox 组件合同 v1

日期：2026-11-15。接口版本：`ANVIL-L1-ABI-v1`。合同修订：`r1`（2026-11-16，canonical calldata明确化，ABI/签名域不变）。状态：**FROZEN；09 lead 已落实独立 Verifier 的三项冻结条件，可交 Builder**。本合同授权范围为独立合约及显式输入的真实 L1 组件；组件通过不等于功能09完整 accepted。

基线 `0a951c482a9712678ab9de34765b781b9c3c9f42`，工作区 `/home/jy/.codex/worktrees/65ef/megaroation_poc`，分支 `codex/poc-anvil-l1`。本合同按[分层依赖v2](../commander/dependency-plan-v2.md)覆盖旧交接的整任务等待；不改变协议语义或完整验收。

## 1. 来源、边界和所有权

`S` 是来源规则；`P0` 是 v2 已授权 mock；`E` 是09在本合同冻结的工程选择。

| 来源 | 本合同使用的规则 |
| --- | --- |
| [v10 架构](../../../formalization_v10/00_arch_zh_v0_1.md) §§1.2、4.2 | `p=(block,transaction,transaction-local-log)`；请求 `(authenticated caller,rid)` 同内容重试返回原结果，异内容拒绝；历史缺失 WAIT。 |
| [Authority](../../../formalization_v10/01_l1_authority_zh_v0_1.md) §§3.1–3.2、3.9–3.11 | 共用严格递增序号；实际 inclusion timestamp 加 `M_a`；完整 policy；RESUME 与 Override 原子；origin 不超过 inclusion height；不在合约判断治理活性。 |
| [Inbox](../../../formalization_v10/07_batch_inbox_zh_v0_1.md) §§3.1–3.7 | 完整 cursor；source 与 written_at 分离；固定权限、完整签名和 DA 绑定；只有普通接纳与同交易 Override 改 cursor；历史读不表示 G′/D 已物化。 |
| [Certifier](../../../formalization_v10/06_certifier_zh_v0_1.md) §§3.2、3.4、4 | 签名绑定 chain/Inbox/prev/next/commitment/DA certificate hash/range/gpos；不签未来 written_at；恢复后的 stale prev 拒绝。 |
| [DA](../../../formalization_v10/08_da_zh_v0_1.md) §§3–4 | 公开确定的实际验签；commitment 打开到唯一 bytes；已接纳而未读到的数据是 WAIT；签名不证明已执行内容。 |
| [GFR/Derivation](../../../formalization_v10/09_finality_rules_zh_v0_1.md) §§3.3–3.4、4.6–4.7、5.1–5.5 | 精确位置、同交易 RESUME/CursorSet、每条 RESUME 独立历史切面；09不实现 ReplayPosition 排序或 ResumeCut。 |
| [v2](../../../formalization_v10/poc_design_zh_v2.md) §§2、7、8.5–8.6、9.3、10.3 | P0 无 staking；EIP-712 治理；固定 submitter/certifier；单完整 frame、每批1–2 heights；手动 Anvil、独立 finalize 和 reader delivery。 |
| [原生 Inbox](../../../megaeth_specs/specs/block-certification/batch-inbox.md) Emergency Cursor Override；[派生准备稿](derivation-preparation-v1.md) | 已物化快照 `a738f9b14c7880953d8cff8788db14e8e3e79124` 仅用于解释 override origin 上界和事件意义；不声称兼容原生六字段 cursor、EigenDA/TEE/activation ABI。 |
| [09](../handoffs/09-anvil-l1-zh.md)、[11](../handoffs/11-batch-services-zh.md)、[12](../handoffs/12-derivation-zh.md)、[13](../handoffs/13-resume-zh.md) | 当前消费者需求；09可先冻结本职接口，真实消费者随后按准确提交接线。 |

根 DESIGN、固定 Regional BFT `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` DESIGN 及 `regional-bft-conformance` 已核对。Commonware 固定 `2026.10.0`；本组件没有 BFT 引擎和 quorum 变更，不把 n=15 benchmark 规则用于本任务。

| 路径／职责 | owner |
| --- | --- |
| 本合同、`poc/docs/anvil-l1-preparation-v1.md` | Docer `bft_spec`，09 lead 冻结 |
| `poc/contracts/`、`poc/crates/l1-adapter/`（独立 workspace/lockfile） | 单 Builder `bft_builder` |
| `poc/verification/anvil-l1/` | 独立 Verifier；Tester 仅写 lead 指定报告/结果路径 |
| 公共 types、根 `poc/Cargo.toml`、`poc/Cargo.lock`、core 声明 | 03 独占；09只读，不重定义公共对象 |
| 真实 transport/Clock；native 恢复 | 02；01，各自准确提交后接线 |
| 源 checkout、Regional BFT 子模块、commander 登记 | 本任务只读 |

一个路径一个 writer；lead 负责接口/依赖/集成。所有 Rust build、Foundry批量编译测试和重型运行使用 `flock /tmp/megaroation-poc-heavy.lock <command...>`、`CARGO_BUILD_JOBS=2`，不得嵌套锁。`.build` 指向 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/09`，target/cargo/artifacts/PID 任务独占。仅监听本次分配的 loopback 端口，只结束自己的 child。

## 2. ABI 数据与配置（E）

Solidity `0.8.20`，Shanghai，ABI 标准编码，不用 `abi.encodePacked` 拼签名字段。r1显式要求所有写入口（含Override）的 `msg.data[4:]` 与 `abi.encode(decoded arguments)` 逐字节相同；拒绝额外尾字节、动态offset别名或其他非规范布局。这是09工程profile限制，不是Solidity默认decoder的保证。格式拒绝发生在任何请求缓存/状态/事件生效前；规范编码中的有效异签名重试仍按§4幂等规则处理。公开 interface 与编译产生的 ABI 一并发布。所有整数范围检查在窄化前完成；溢出拒绝，不能取模。所有动态 `bytes` 在 JSON 中是偶数位 `0x` hex；这只是传输表示，签名字节是其原始 bytes。

```solidity
struct Policy {
    uint8 enabledMask;
    uint8 order0;
    uint8 order1;
    uint8 order2;
    uint64 l1Floor;
}
struct Cursor {
    uint64 height;
    uint64 origin;
    bytes32 framesDigest;
    uint8 sourceKind;
    bytes source;
}
struct BatchSubmission {
    Cursor prev;
    Cursor next;
    bytes32 commitment;
    bytes daCertificate;
    uint64 firstHeight;
    uint64 lastHeight;
    bytes gpos;
    bytes certifierSignature;
}
```

P0 Policy 只接受 `Policy(7,0,1,2,l1Floor)`，即公开 `static-schedule-v1` 的固定三地区顺序；其他 mask 或 order 均以 `InvalidPolicy` 拒绝，不提供临时排除地区或重排的扩展。`l1Floor` 为 uint64，完整签名绑定并保留在历史中，不改变静态排班公式；不增加“必须不晚于当前 block”的来源外限制。

Cursor 的 source 规范形状：

| sourceKind | source 字节 | 意义 |
| --- | --- | --- |
| 0 | 长度0 | GENESIS；完整初始化内容由部署 manifest/configRef 指定。 |
| 1 | 长度1–1024 | GLOBAL；03/07 所有的 canonical ReplayPosition 编码的 opaque bytes。09不解释或另造 replay 类型。 |
| 2 | 恰32 bytes，等于 `abi.encode(uint64 controlSeq)` | RESUME；高24 bytes必须为0，唯一序号绑定 Authority 记录。 |

其他 tag、非规范空值和超长值拒绝。`gpos` 长度1–1024；`next.sourceKind=1` 且 `next.source` 与 `gpos` 字节完全相同。这里的范围校验不是 ReplayPosition 语义验证；来源 gpos 有效性由真实03/07对象和可信 Certifier 检查。组件向量只证明 opaque bytes 的绑定。

P0 每批完整闭合：`prev.framesDigest=next.framesDigest=0`，`firstHeight=prev.height+1`，`lastHeight=next.height`，范围恰1或2个连续 heights。`next.origin <= block.number`。不在 Inbox 检查 sequencing window、G/D head、payload 执行或恢复预算。Partial frame、EMPTY range、可变 certifier/submitter、替代 Batcher 权限、EigenDA/TEE、staking/upgrade 不在此 profile 中；不能默认为已支持。

部署配置包含 `chainId`、非零 `configRef:bytes32`、不同的治理/Certifier/DA signer 地址、固定非零 submitter、`M_a:uint64 > 0`、genesis height/origin 和 timestamp policy。地址不可修改。建议 Authority 构造函数创建其唯一 Inbox，避免可重绑 admin；若分开部署，必须采用在同次部署固定双方地址、无公开重绑的等价机制。Genesis cursor 为 `(genesisHeight, genesisOrigin, 0, 0, 0x)`，`genesisOrigin <= deployment block.number`。不是强制 origin=0；与03实际 genesis 的匹配是集成 gate。部署产生单独 `CursorInitialized`，不得用 `CursorAdvanced` 或 `CursorSet` 伪装 genesis。

## 3. EIP-712 与签名字节（E，满足S绑定要求）

Hash 全部为 Ethereum Keccak-256；不能换成 NIST SHA3-256。EIP-712 domain type 的精确 UTF-8 字符串：

```text
EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)
```

Authority name=`SequencerRotationControlAuthority`；Inbox name=`SequencerRotationBatchInbox`；两者 version=`1`，chainId 取实际 `block.chainid`，verifyingContract 为实际合约地址，salt=`configRef`。域分隔符为 `keccak256(abi.encode(domainTypeHash,keccak256(bytes(name)),keccak256(bytes("1")),chainId,address,salt))`；签名 digest 为 `keccak256(0x1901 || domainSeparator || structHash)`。不做 `personal_sign` 前缀。

以下是精确 type strings；字段顺序是规范的一部分：

```text
Policy(uint8 enabledMask,uint8 order0,uint8 order1,uint8 order2,uint64 l1Floor)
Cursor(uint64 height,uint64 origin,bytes32 framesDigest,uint8 sourceKind,bytes source)
Force(address caller,bytes32 requestId,uint64 controlSeq,uint64 targetBranch,bytes32 policyHash)
Resume(address caller,bytes32 requestId,uint64 controlSeq,bytes32 policyHash,uint64 targetHeight,uint64 origin)
DAReceipt(bytes32 commitment)
BatchAttestation(bytes32 prevHash,bytes32 nextHash,bytes32 commitment,bytes32 daCertificateHash,uint64 firstHeight,uint64 lastHeight,bytes32 gposHash)
```

每个 `structHash=keccak256(abi.encode(typeHash, field1,...))`。Policy 五字段直接 ABI 编码。Cursor 的动态 `source` 字段用 `keccak256(source)` 替代；其余直接编码。Force/Resume 的 `policyHash` 是上述 Policy structHash。Batch 的 `prevHash/nextHash` 是上述 Cursor structHash，`daCertificateHash=keccak256(daCertificate)`，`gposHash=keccak256(gpos)`。完整 cursor、完整 gpos 和完整 DA certificate 均被绑定；`written_at` 不在任何待签名对象中。

控制签名额外绑定实际 `msg.sender` 为 `caller` 和 `requestId`，允许 relayer，但只能使用授权该 relayer 地址的治理签名。治理 signer 与 caller 是不同概念。BatchAttestation 沿 v10 不含 requestId；固定 submitter 可给同证书选择请求编号，prev 检查仍阻止第二次接纳。

所有三类签名是实际 secp256k1 `r[32] || s[32] || v[1]`，长度65，`v∈{27,28}`，`r∈[1,n-1]`，`s∈[1,n/2]`，拒绝恢复到零地址或非配置 signer；不接受 high-s 别名、compact signature 或 boolean valid 桩。DA certificate 就是配置 DA signer 对 Inbox 域 `DAReceipt(commitment)` 的65字节签名；Certifier 签名为同域 BatchAttestation，typeHash 隔离两种用途。该固定 signer DA 是明确P0替代，不宣称 EigenDA 证明。

P0 DA commitment 为 `keccak256(payload)`。DA 服务只在 bytes 不可变持久化后签名，此持久性由11验收；09能验证 certificate/commitment 绑定但不能从签名证明磁盘持久。任何读取者取回 bytes 后须核对 commitment；未取回或 mismatch 不产生合法 DAObject/派生输入。

公开 digest helper：`hashPolicy(Policy)`、`hashCursor(Cursor)`、`forceDigest(caller,rid,seq,branch,policy)`、`resumeDigest(caller,rid,seq,policy,height,origin)`、`daDigest(commitment)`、`batchDigest(BatchSubmission)`。helpers 与执行使用相同 preimage；独立黄金向量不得由这些 helpers 生成 expected。

## 4. 写接口、去重、原子状态和事件

最小公开 ABI（Solidity getter 可额外暴露固定配置，不得新增状态修改通路）：

```solidity
// ControlAuthority
submitForce(bytes32 requestId, uint64 controlSeq, uint64 targetBranch,
            Policy policy, bytes signature)
    returns (uint64 acceptedSeq, uint64 includedBlock, uint64 act);
submitResume(bytes32 requestId, uint64 controlSeq, Policy policy,
             uint64 targetHeight, uint64 origin, bytes signature)
    returns (uint64 acceptedSeq, uint64 includedBlock, uint64 act);
// BatchInbox
submitBatch(bytes32 requestId, BatchSubmission submission)
    returns (bytes32 acceptedCursorHash, uint64 includedBlock);
overrideCursor(Cursor next); // Authority-only; no externally callable admin setter
currentCursor() view returns (Cursor);
```

请求键为各合约内 `(msg.sender, requestId)`。Authority 的 Force/Resume 共用请求表；不同 type 即不同内容。先检查身份/签名格式和签名，再处理请求表；请求存在且完整语义内容相同，返回存储的原结果，无状态变化/新协议日志；同键不同内容 `RequestConflict`。重新签署相同消息而产生另一份有效低s签名视为同内容：Authority content 是控制 EIP-712 structHash；Inbox content 是 BatchAttestation structHash（包含 DA certificate 原始 bytes 的hash，但不包含 Certifier signature）。坏签名即使 request 已存在仍拒绝。缓存命中必须先于当前 seq/prev/height 检查，否则原先成功请求在状态推进后无法幂等重试。失败请求不占用 rid，不消耗控制序号。

控制序号共用 `hasControl` sentinel：初始 false，第一条可为0；以后严格 `controlSeq > lastControlSeq`，允许跳号，不要求 `+1`。达到 uint64 上界后不能再用新序号控制，不溢出复用。无效目标 branch、HALT 等 GFR语义不是Authority拒绝条件。

事件精确参数和 indexing：

```solidity
event ForceRecorded(address indexed caller, bytes32 indexed requestId,
    uint64 indexed controlSeq, uint64 targetBranch, Policy policy, uint64 act);
event ResumeRecorded(address indexed caller, bytes32 indexed requestId,
    uint64 indexed controlSeq, Policy policy, uint64 targetHeight,
    uint64 origin, uint64 act);
event CursorInitialized(Cursor cursor);
event CursorAdvanced(address indexed caller, bytes32 indexed requestId,
    Cursor prev, Cursor next, bytes32 commitment);
event CursorSet(Cursor oldCursor, Cursor next);
```

事件 tuple field 顺序与§2完全相同。`act=block.timestamp+M_a`，经过 uint64/加法范围检查，从实际 inclusion header 重算核对；finalize 或 delivery 时刻不参与。事件本身不填假的 tx/log 位置。

| 触发与前态 | guard | 下一状态／输出 |
| --- | --- | --- |
| 新 Force | 合法签名/req/seq/policy，时间可表示 | 写控制序号/请求结果，恰1条 ForceRecorded。 |
| 新 SubmitBatch | 固定 submitter；规范cursor/source/range；完整prev等于当前；DA与Certifier真实验签；origin界 | 原子写next和请求结果，恰1条 CursorAdvanced；旧prev任何字段变化都不相等。 |
| 新 Resume | 合法签名/req/seq/policy；`targetHeight>cursor.height`；`origin<=block.number` | Authority构造 `(targetHeight,origin,0,2,abi.encode(seq))`，内部Override成功后发ResumeRecorded；seq/req/cursor/events共同提交。 |
| Override | `msg.sender==immutable authority`；target严格增加；frames=0；origin界；source为RESUME且与此次Authority调用参数一致 | 写cursor，恰1条CursorSet；没有通用外部可调用的转发入口。 |
| 任一授权/格式/签名/内部调用失败 | 已确定失败 | revert；receipt status=0；无该操作日志，cursor/seq/req/历史不变。 |

RESUME origin 是治理签名中的真实输入，合约不把它改成 block.number。它是 cursor scan 元数据，**不等于**12随后按实际D parent/origin计算的恢复anchor origin；09不制造 ResumeCut。

固定事件顺序：成功 Resume 的 Inbox `CursorSet` 在前，Authority `ResumeRecorded` 在后；Authority先进行所需检查/状态记账，再内部调用，最后emit。EVM revert仍回滚先前写入。唯一Authority只从SubmitResume调用Override，外部没有独立Override授权入口。为验证“内部已改cursor后后续revert”的EVM原子性可用独立test-only harness；它不得进入生产部署配置。

最小错误名均为无参 custom error：`Unauthorized()`、`InvalidSignature()`、`InvalidPolicy()`、`InvalidCursor()`、`InvalidRange()`、`InvalidSequence()`、`RequestConflict()`、`StalePrev()`、`InvalidOrigin()`、`InvalidTarget()`、`InvalidDA()`、`Overflow()`。Solidity ABI decode 的失败可为原生revert；独立拒绝测试每次只改变一个guard，不依赖多个同时失败时的错误优先级。

## 5. Receipt、历史与03公共类型映射

`p=(block,tx,log)` 的log是**该交易receipt内所有日志的0-based索引**。Ethereum RPC `logIndex` 是block全局索引，保留为 `rpcLogIndex` 证据，不能直接放进规范p。转换须有完整有序交易和完整receipts，验证 `transactionIndex`、blockHash、txHash、receipt顺序和日志索引连续性，再在receipt内枚举；不得先过滤其他合约日志后重新编号。

| ABI/RPC材料 | 规范对象／边界（03 owner） |
| --- | --- |
| 完整header + hash/parentHash/number/timestamp | L1 metadata；timestamp秒按 checked `*1_000_000` 转参考μs。 |
| receipt的blockNumber/transactionIndex/交易内log序号 | `L1Position{block,tx,log}`；真实txHash和RPC全局logIndex另存证据。 |
| Cursor的四项逻辑字段及source tagged bytes | 03 Cursor/source；GLOBAL bytes经03 canonical decoder消费；不能由09opaque roundtrip声称语义有效。 |
| 成功CursorAdvanced + 同tx完整提交calldata | AcceptedBatchInput；commitment/prev/next相等，保留DA certificate、range、gpos、签名、transaction_ref。 |
| CursorSet与同tx唯一对应ResumeRecorded | OverrideInput，`resume_position`取实际Resume事件p。 |
| 完整finalized block前缀及全部receipts（含失败/无日志tx） | `FinalizedPrefix{tip,control_coverage,provenance}`；缺日志材料不是“零controls”。 |
| 确定as_of范围内最后cursor写入及原始receipt | CursorRead `{cursor,written_at,as_of,read_evidence}`。 |

成功Resume匹配要求：同chain/blockHash/txHash/transactionIndex，配置Inbox/Authority地址，成功receipt，sourceKind=2、seq相等、height/origin相等、frames=0，CursorSet先于ResumeRecorded；不能仅按seq或block配对。最小生产路径每个Resume调用各一条Set/Resume；含多条pair的test harness必须逐pair核对，不能任选最后一条。两条不同transaction的RESUME即使同block/act也有各自p；不返回“block最终D”。缺同tx配对材料 WAIT；完整材料明确矛盾 REJECT，不发布半个控制。

历史读由adapter提供 `read_cursor(reader, as_of_height)` 和逐事件receipt查读，不要求EVM合约知道finality或未来tx/log。`as_of <= reader.visible_prefix`；使用该block tag的 `eth_call currentCursor()` 与重放至该block最后cursor事件的值相互核对。written_at是最后CursorInitialized/Advanced/Set的真实p，重试无新事件故不改变written_at。Genesis之前读取返回NotInitialized/WAIT（取决于请求是否缺已知历史），不能捏造 `(0,0,0)`。同block内精确事件位置的cursor历史由有序receipt重放得到；block-tag状态仅表示整block结束。

read_evidence含明确的部署配置/地址、header/tx/receipt及canonical chain联系。本P0信任自管Anvil RPC所给canonical执行，做完整性、一致性与合约签名核对，不声称验证Ethereum PoS或MPT inclusion proofs。此信任前提不能删去。

## 6. 显式输入 driver 与 reader（E）

生产组件是 `poc/crates/l1-adapter/` 的同一个Rust状态机及真实JSON-RPC backend；Python可编排/检查，但不得另有一套推进链再用未被调用的Rust toy driver充数。无timer/wall-clock自动mine/finalize/deliver。fake backend只供单元向量，不算真实EVM结果。

公开组件输入语义如下；Rust具体命名可遵循本表，不引入第二套canonical公共域对象。每个输入带唯一有序command id，输出带该id及原始RPC证据。相同id/内容重放不重复副作用，冲突id拒绝；不确定RPC提交结果进入待核对状态，不能盲目发第二个mine。生产Clock/transport grant/ack封装由02真实接线提供，本阶段脚本承担显式输入来源并记录此边界。

| 输入 | guard／可观察结果 |
| --- | --- |
| `Tick(reference_us)` | 非负单调零偏移虚拟参考时间；只改变可执行输入的时间条件，不自行出块或交付。 |
| `Submit(raw_signed_transaction)` | 明确提交输入才调用sendRawTransaction；缓存实际txHash；提交成功不表示已included/success/finalized。重复同raw tx按txHash去重。 |
| `Mine(slot)` | 顺序下一个slot，slot对应时间<=当前Tick，且无未完成mine；driver唯一调用setNextBlockTimestamp+mine；取回核对实际header和全部tx/receipts后发布included。 |
| `Finalize(through_height)` | 不超过已经完整获取的included prefix；每个block已到配置的虚拟finality delay；只能单调推进，不能只信Anvil `finalized` tag。 |
| `Deliver(reader, through_height)` | 不能超过finalized prefix；交付连续完整headers/receipts/交易；reader独立不可变prefix单调推进。 |
| `Read(reader, historical_query)` | 只消费该reader已交付prefix；未来/缺coverage WAIT；明确冲突REJECT。 |

Timestamp policy：genesis timestamp为manifest中的整数秒 `t0`，固定slot间隔 `slotSeconds>0`（默认2）；block n的timestamp=`t0+n*slotSeconds`。Mine n要求reference_us至少为 `n*slotSeconds*1_000_000`，所有乘加checked。启动Anvil固定genesis timestamp、chainId和gas配置，关闭auto/interval mining；部署交易也通过同一个driver的Submit/Mine。组件虚拟reference_us以genesis为0；绝对header秒与相对reference_us不可混用。slot编号等于实际连续block number，部署占用的blocks也计入；block n的相对虚拟生产tick是 `n*slotSeconds*1_000_000`。默认模拟finality delay=2,000,000μs；Finalize要求当前reference_us至少为该生产tick加delay，且必须由单独输入触发，不因Tick自动推进。若实际02公共epoch不同，由显式offset配置映射并向下游记录，不能偷偷读墙钟校正。

异步RPC到达、执行耗时和墙钟暂停只影响本机耗时，不影响命令顺序/slot/timestamp。实际RPC失败与待补receipt记录为基础设施结果，不能伪装protocol成功。并发提交的纳入顺序由脚本固定发送顺序、明确交易nonce及Anvil FIFO选项约束；保存实际tx顺序，不用预计nonce替代receipt。

driver持有的未来blocks不授权reader查看；reader A/B使用独立prefix，在相同prefix上值一致。尚未交付的历史即使底层RPC可查询也不得透过reader API访问。缺任一block、tx、receipt或控制pair时不推进完整control coverage。可缓存乱序材料，但只能发布连续已验证前缀。

## 7. 独立黄金输入与验收矩阵

Verifier必须先从来源与本合同推导expected，并在读实现前保存源码hash、type strings、ABI preimages、digest、真实签名、恢复地址、事件topic和语义向量。独立向量使用固定chainId `31337`、configRef=`0x11`重复32bytes、Authority=`0x1000000000000000000000000000000000000001`、Inbox=`0x2000000000000000000000000000000000000002`；这些离线地址不是部署地址，真实Anvil向量需按实际部署地址另生成。

最小固定语义向量：policy `(7,0,1,2,0)`；genesis cursor `(0,0,0,GENESIS,empty)`；opaque gpos `0x01020304`；首batch范围 `[1,2]`、next height2/origin0/sourceGLOBAL(gpos)，payload是UTF-8 `anvil-l1-v1-payload`；首Force seq0/targetBranch7；后Resume seq2/targetHeight8/origin0。请求编号分别为32bytes的1、2、3（big-endian）。黄金请求对应Force=1、Batch=2、Resume=3，caller=`0x3000000000000000000000000000000000000003`。固定私钥只能作为公开测试fixture标识，必须与真实/本机凭证区分，禁止复制用户凭证。黄金期望含签名逐字段变异，不依赖候选helper自证。

| ID | 前态／输入 | 必须结果与原始证据 |
| --- | --- | --- |
| A01 | 固定三域与数据，独立hash/signature/ABI vectors | 离线独立expected与合约digest/真实ecrecover一致；保留preimage和签名。 |
| A02 | 错chain、contract、configRef、control type、req caller/rid及Force/Resume每一实际输入变异 | 对应签名拒绝；无seq/cursor/req/events变化。 |
| A03 | first seq0；later seq2；重复/下降seq；max边界 | 0和跳号2成功；新rid的旧/相同seq拒绝；不overflow。 |
| A04 | 同caller/rid/内容有效重试；同rid不同内容；不同有效签名同内容；合法签名配trailing/alias ABI | 原结果/零新日志/原written_at；异内容拒绝；坏重试签名拒绝；失败不耗rid；非规范ABI以真实status=0收据验证。 |
| A05 | 固定policy及mask0/非法高bit、非固定排列/重复/越界、l1Floor变化 | 仅(7,0,1,2,l1Floor)接受；其他mask/order拒绝；所有policy字段被签名绑定。 |
| A06 | 真实两批1–2height接纳；错submitter、wrong prev任一字段、source tag/bytes、range、origin | 正例各恰1条Advanced；拒绝不动cursor；完整prev比较不只height。 |
| A07 | DA错误signer/commitment，certifier错误signer；逐字段变异；signature长度/v/r/s边界 | 实际cryptographic拒绝；high-s/错误域/type不通过；没有bool替代。 |
| A08 | Resume target等于/低于cursor、origin大于inclusion、直接Override | 拒绝、无Set/Resume，且seq/rid未消耗。 |
| A09 | 正确Resume；后续使用Override前签出的旧prev batch | 同tx Set→Resume，height8/frames0/sourceRESUME(seq)；旧prev拒绝。 |
| A10 | 内部Override失败；test-only后写入再revert | Authority/Inbox状态及两类日志全部回滚，成功/失败receipt均存。 |
| A11 | 同block多控制（Force/Resume混合，至少两Resume） | 同act但不同tx/local-log位置；Set/Resume逐pair相互绑定；不合并为block结果。 |
| A12 | 两笔含日志交易在同block，第二笔RPC logIndex不为0 | 规范local-log从该receipt0开始；保存全局RPC索引；过滤日志前枚举。 |
| A13 | 当前和多个历史block/tag、无新写block、重试block、同block逐event历史 | cursor/written_at准确；后续写不改旧读；genesis位置来自Initialized。 |
| A14 | 真实Anvil墙钟暂停至少250ms；只有Tick、Submit而无Mine | blockNumber不变；pending tx无receipt；显式Mine后准确slot/header时间。 |
| A15 | Mine后未Finalize、Finalize未Deliver、A/B不同Delivery | 三个frontier独立；读者不能查未交付未来；same-prefix结果相同。 |
| A16 | 缺block/receipt/控制pair、乱序delivery、冲突重复历史、失败receipt/错误emitter | 缺材料WAIT，冲突REJECT；不产生虚假control coverage/accepted input。 |
| A17 | 同脚本两次、一次不同墙钟暂停；重复command/id冲突、未知mine结果 | 规范虚拟trace一致；重放不再mine；不确定副作用必须核对；保存完整RPC请求响应。 |
| A18 | 02受控Clock/transport及03公共对象在准确上游SHA接线 | 本组件同API成为真实受控链路，grant/ack及reader gateway回归通过；本阶段保持INCONCLUSIVE。 |
| A19 | 11实际DA/Certifier/G来源、12历史ResumeCut、13恢复终点 | 各owner联调；09组件验收不能替代这些结果。 |

Anvil部署地址与nonce固定到run manifest；完整matrix使用准确candidate SHA、干净状态和固定输入脚本，无随机调度时seed写`none (explicit script)`，随机补充另列seed。测试包括实际status=0的已收录失败交易，不能只用eth_call revert替代。某输入若节点预验证拒绝，记录该层结果并另准备可被mined的负例。

## 8. 阶段gate与交付

冻结前：09 lead核对全部工程选择，独立Verifier完成source→expectation与黄金向量审查，保存合同/expected准确SHA。Builder只能在此后实施，接口修订回到合同并标明受影响消费者。

组件候选提交后：Verifier对准确源码SHA复核A01–A17，Builder修复后复核；Tester在同一已核查SHA运行约定矩阵，保留命令、工具hash、ABI、配置、原始RPC/receipts/部署、每项PASS/FAIL/INCONCLUSIVE与限界。Foundry参考版本 `1.0.0-stable/e144b82070619b6e10485c38734b4d4d45aebe04`，solc `0.8.20+commit.a1b79de6`；版本/hash来源见[setup](../setup/foundry-2026-11-15.md)。历史counter smoke不算本合同证据。

完整09仍需要：02真实受控调度/Clock与grant/ack、03同一公共对象/reader接口、真实reader完整前缀及历史查询在准确集成SHA的A18回归。A19由11/12/13消费，需明确数据/位置映射兼容但不在09伪造服务或切面。不得把接口发布、合约smoke、显式脚本组件通过写成feature accepted；不合main、不push。

未决项只限外部接线：03/07 canonical ReplayPosition bytes、最终公共Rust类型/traits与02命令-envelope、03实际genesis/config映射、11实际frame codec/DA持久性和12实际ResumeCut。以上不会阻塞本合同的opaque绑定、真实合约和driver组件实现，但禁止以fixture证明这些外部项。

冻结记录：09 lead已落实[独立合同审查](../../verification/anvil-l1/contract-review.md)的三项条件；[黄金向量](../../verification/anvil-l1/golden-vectors.json) SHA-256为 `c189ae0c36df64bb60ee565c45c0f85cd23bb2a87500e17110c37cadc68d24d5`。接口提交同时包含合同、来源预期、向量及独立准备结果；本冻结不是候选实现通过。

r1记录：lead与独立Verifier在候选前确认canonical-calldata限制与原独立decoder一致；不改函数selector、事件topic、EIP-712域或请求semantic identity。初始可消费接口提交为 `daad412e46eeecd64ef9b8af8175e64ec40edc53`，消费者继续使用规范ABI编码；候选包含本明确化和相应真实EVM拒绝向量。
