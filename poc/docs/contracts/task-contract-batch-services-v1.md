# 功能11：Batcher、Certifier、DA 独立组件合同 v1

日期：2026-11-16。版本：`BS-COMPONENT-v1 / r2`。状态：**FROZEN：独立 Verifier 来源先行审查 PASS，11 lead 批准按本合同实现。** r2只澄清固定gpos区间投影与最新RESUME覆盖条件，BS-FRAME-v1/BS-CRYPTO-v1字节和M1接口保持不变。基线为按顺序正常合并03 `e9c15e04bc83b335668f3ecd837367aa87344a89` 和09 `c443ae925ed549b046731c2d8ef3dd9ce0311d24` 后的 `c9805f7a818bc23d30018ce4dc6f1ee45f92e3ac`。工作目录 `/home/jy/.codex/worktrees/f025/megaroation_poc`，分支 `codex/poc-batch-services`。

本合同只冻结可独立消费的服务组件。真实 DA 持久化、真实 secp256k1 签名和真实 Anvil 接纳可在本阶段验收；公共 G/GFR 来源仍为显式受限 fixture。组件 PASS 不代表完整11、G′、D、RESUME 或受控系统接线 accepted。

## 1. 来源、工程选择与所有权

`S` 为来源规则，`P0` 为 v2 mock，`E` 为本合同的工程选择。

| 来源 | 采用的规则 |
| --- | --- |
| [v10 Batcher](../../../formalization_v10/05_batcher_zh_v0_1.md) §§3.2–3.5、BAT-1/2/3 | 记录高度连续、epoch为来源origin、transactions为同一来源位置执行的Kept；padding有空普通交易记录；job按prev/y区分；DA→认证→finalized接纳；不拥有cursor修改权。 |
| [v10 Certifier](../../../formalization_v10/06_certifier_zh_v0_1.md) §§3.1–3.4、CERT-1/2/3 | 独立L1/G来源；当前prev、最近相关RESUME已重放、DA打开及证书、来源Kept、next/range全部检查后签名；缓存不跳过这些检查。 |
| [v10 Inbox](../../../formalization_v10/07_batch_inbox_zh_v0_1.md) §§3.2–3.7 | source与written_at分开；accepted来源为成功finalized事件；G′来源是具体gpos或具体ResumeCut，cursor高度不等于已物化高度。 |
| [v10 DA](../../../formalization_v10/08_da_zh_v0_1.md) §§3–4 | 唯一不可变payload；公开确定的CheckDA；未知/暂不可取回不等于空数据；已确认数据恢复后保留。 |
| [v2](../../../formalization_v10/poc_design_zh_v2.md) §§5.1、7、10.2–10.3 | crash-only角色、每批单完整frame/1–2连续heights、真实签名/commitment、03轻量执行；显式调度输入，墙钟不参与公共规则。 |
| [03合同v2](task-contract-mock-plan-execution-v2.md) §§2–6 | 使用现有Hash32/L1Position/Metadata/ExecutionState/VirtualBlock/Transfer和Apply；不复制交易、state或native hash编码。ReplayPosition只有typed骨架，其完整codec归07。 |
| [09合同](task-contract-anvil-l1-v1.md) §§2–6，`ANVIL-L1-ABI-v1/r1` | 原样消费cursor/ABI、Keccak commitment、DAReceipt/BatchAttestation EIP-712、成功receipt、交易内log位置、实际driver与独立reader gateway。 |
| [11 handoff](../handoffs/11-batch-services-zh.md)、[派生准备稿](derivation-preparation-v1.md) | 下游12独立从L1/DA取字节；11唯一拥有frame编码；accepted缺DA时WAIT；恢复切面归12/13。 |

根 DESIGN 指向固定 Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`；Commonware固定 `2026.10.0`。本组件不改BFT协议、quorum、native恢复或其签名门禁。适用 `regional-bft-conformance` 的证据、交付、执行分层和crash边界要求；没有本任务的额外BFT论文规则。MegaETH快照为 `a738f9b14c7880953d8cff8788db14e8e3e79124`，本P0 codec不声称与MegaETH/OP原生wire兼容。

| 路径 | 唯一writer/职责 |
| --- | --- |
| 本合同 | Docer `bft_spec`；lead审查后冻结 |
| `poc/crates/batch-services/`，独立workspace及自己的Cargo.lock | 单Builder；内部 `batcher`、`certifier`、`da`、`accepted_view`，可另有唯一公共codec/crypto/persistence模块 |
| `poc/verification/batch-services/` | 独立Verifier；Tester仅写lead移交的报告子路径 |
| 本任务报告 | lead分配的11专属报告路径 |
| 03 types/core、根workspace/lockfile；09合约/ABI/driver；其他功能；`poc/docs/commander/`；Regional BFT子模块 | 只读；需要扩展时提出局部变更，由总协调指定owner |

`.build` 使用 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11`，本次新的target/artifact/PID目录，不覆盖旧证据。所有重型build/test/simulation经 `flock /tmp/megaroation-poc-heavy.lock`、`CARGO_BUILD_JOBS=2`，不嵌套锁。主协调ultra，Docer/Verifier为实际命名Astra/xhigh角色，Builder/Tester为Sol/high；运行时无法核验的信息如实注明。

## 2. 优先交付：唯一frame/payload编码 `BS-FRAME-v1`（E）

这是v2允许的单完整frame、闭合channel P0原生profile，不采用压缩、partial frame、多个frame或原生channel接手规则。完整格式在本版本中唯一；12必须消费本crate的同一个decode/commitment接口，不在另一个crate重写生产codec。独立checker另写解码用于验证是允许的。

全部整数big-endian、无符号、固定宽度。下面`||`是串接，`raw`是原bytes，不是hex文本。字段不能隐式补零、截短、取模或忽略尾随字节。

```text
SubmissionPayload y =
    magic[4]                   // ASCII "P0B1"，hex 50304231
 || derivation_version:u16     // 恰为1
 || channel_id[16]
 || frame_number:u16           // 恰为0
 || frame_data_len:u32
 || frame_data[frame_data_len]
 || is_last:u8                 // 恰为1

frame_data = record_count:u16 || BlockRecord[record_count]

BlockRecord =
    block_number:u64
 || epoch_num:u64
 || transaction_count:u32
 || (transaction_len:u32 || transaction_raw[transaction_len])[*]

channel_id = first_16_bytes(Keccak256(frame_data))
d = Keccak256(y)
```

`record_count`恰1或2。记录高度严格连续且checked加法；不因wrap把`u64::MAX`后视为0。单个记录height可以为`u64::MAX`，能否接到prev由§5检查。origin是每条记录的`G_gpos[h].metadata.origin`，解码器不依赖当前L1或墙钟来判断它；来源匹配由Certifier核对，派生中的origin有效性由12判断。

每record普通交易数不超过03的 `MAX_TRANSACTIONS=1024`，raw总bytes不超过03的 `MAX_TRANSACTION_BYTES=65536`，实际Apply还服从绑定PublicConfig的byte_limit。每条交易必须调用03 `Transfer::decode_raw` / `validate_transactions`，在本版本中恰19B，格式为03既有`01|from:u8|to:u8|nonce:u64BE|amount:u64BE`。**此处只是引用03格式；11不得维护第二份交易decoder。** Credit/Attr/DEP、未知opcode、截断或尾字节整体拒绝。Kept允许零额和self-transfer；不能根据内容在codec里再执行或过滤它。

由这些固定上限，合法单record最多`20+1024*(4+19)=23572`B，frame_data最多`2+2*23572=47146`B，完整payload最多`47175`B。在分配长度或做加法前检查上限；最短合法payload是1条空记录，共51B。长度必须恰等实际数据；frame_data内部消耗必须恰好结束，channel_id必须匹配计算值。magic/version/number/is_last/record count不符、trailing bytes、溢出或超限均Reject；无部分成功。

外层payload中没有DEP、parent hash、state root、G对象、gpos或L1 receipt。由03元数据及实际D parent执行是12的责任；G/gpos和cursor由签名绑定。`channel_id`不授予来源权威，仅给闭合channel确定身份。相同records编码完全相同，无随机channel id；重建不允许改字节却复用旧commitment。

最小公开API语义（Rust名称可统一，但不得提供另一种格式）：

```text
BlockRecord { block_number:u64, epoch_num:u64, transactions:Vec<Vec<u8>> }
ClosedFrame { channel_id:[u8;16], records:Vec<BlockRecord> }
encode_closed(records) -> Result<Vec<u8>, CodecError>
decode_closed(payload) -> Result<ClosedFrame, CodecError>
commitment(payload) -> Hash32                     // Keccak256，非03 SHA256对象hash
verify_payload(commitment, payload) -> Result<ClosedFrame, CodecError>
```

`verify_payload`要求hash相等且decode成功。`commitment`只是哈希，不表示格式、持久性、来源或接纳有效。错误至少区分UnsupportedVersion、Malformed、LimitExceeded、Overflow、CommitmentMismatch；Display文本不作协议。

## 3. 09签名与对象复用 `BS-CRYPTO-v1`

生产ABI完全使用09 `BatchWire/CursorWire`、既有合约ABI和其语义。可在11中用规范ABI库产生签名preimage/calldata，不另创09字段/width/type string。已有09 public decoder/gateway负责验证真实receipt。wire JSON只是传输表示，不能拿JSON文本签名或当公共对象canonical编码。

Domain与09完全相同：

```text
EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)
name="SequencerRotationBatchInbox", version="1"
chainId=实际配置的Anvil chain id
verifyingContract=实际部署Inbox地址, salt=configRef

Cursor(uint64 height,uint64 origin,bytes32 framesDigest,uint8 sourceKind,bytes source)
DAReceipt(bytes32 commitment)
BatchAttestation(bytes32 prevHash,bytes32 nextHash,bytes32 commitment,bytes32 daCertificateHash,uint64 firstHeight,uint64 lastHeight,bytes32 gposHash)
```

`digest=Keccak256(0x1901||domainSeparator||structHash)`；Cursor动态source先Keccak，Batch的prevHash/nextHash为完整Cursor structHash、daCertificateHash为证书原65B的Keccak、gposHash为原gpos的Keccak。所有ABI标量按09标准32B word；禁止`personal_sign`、NIST SHA3或自定义packed替代。`written_at`和未来L1位置不在签名中。

DA signer和Certifier signer必须与部署manifest绑定；身份相互分离并与治理身份不同。签名精确`r[32]||s[32]||v[1]`65B、`v=27/28`、`1<=r<n`、`1<=s<=n/2`，恢复地址必须等于配置。使用真实secp256k1签名/恢复；任何bool或预置valid标记只算拒绝性的fixture，不算密码学路径。保存公开地址、digest、签名、domain与输入向量；不track私钥，只记录公开测试seed/recipe，私钥仅在本机run目录。

公开检查返回值只表示当前检查内容：`check_da(d,receipt,domain,expected_signer)`成功不是DA可取回证据，`verify_batch`成功不是L1接纳或G来源证明。

## 4. 来源能力与G fixture的明确边界

03 `ReplayPosition`可作typed引用，结构validate只校验kind/index匹配；03没有冻结它的canonical字节，完整codec/排序属于07。11 **不实现ReplayPosition编码、不依赖enum声明顺序比较位置**。本阶段`gpos_bytes`是1–1024B opaque来源句柄，必须从显式的`FixtureSourceView`取得并绑定一份不可变source snapshot；报告、trace和manifest写`source_mode=fixture`、fixture身份/摘要及其typed position。不得把任意opaque bytes标为公共GFR位置已验证。

源码中必须存在无默认“全部通过”实现的只读来源能力：给定完整gpos来源引用、区间和所需恢复边界，返回`Ready(source snapshot)`、`Wait(missing)`或`Reject(conflict)`。未来07 adapter在同一边界交付真实replay来源；不能引入第二条绕过fixture验证的生产通路。未知gpos、缺anchor/state/metadata/DEP或未处理相关RESUME为Wait；已知相矛盾的来源/height/origin/执行为Reject。

Batcher与Certifier各有独立source snapshot/replay readiness、L1 reader id、状态目录和mailbox；Certifier不得把Batcher送来的Kept或`source_valid=true`当其独立重算。fixture可复制同一不可变公开输入，但每角色独立执行/核对。该独立性只证明组件行为，不能声明07公共重放或04证明已接线。

source snapshot至少绑定：来源身份和gpos bytes/typed引用、配置/规则版本、区间之前的实际03ExecutionState和native parent、连续VirtualBlock raw输入、实际metadata、配置绑定DEP来源，以及已处理RESUME引用。Certifier从同一父状态逐height调用03 Apply，核对每个pre_state_root、得到Kept/post-state；不得仅比较Batcher提交的记录与自己复制来的记录。padding由来源的空raw普通交易表现，仍照常Apply DEP，编码其空Kept。

最新相关RESUME由Certifier自己的完整finalized Inbox历史确定：取不晚于当前cursor written_at的最后CursorSet所配对RESUME。source须已处理该**具体位置/序号**；只比较branch高度、gpos时间戳或sourceKind不够。即使current cursor已经是GLOBAL且接在RESUME之后，也仍需检查最近RESUME。缺位置完整性或处理证明则Wait；不能猜测“不存在RESUME”。fixture必须显式给出它可覆盖的控制历史和处理集合。

**r2来源解释（独立审查PASS）。** 固定gpos的快照可以覆盖比本批更长的前缀：例如已有G[1..4]，可依次编码[1,2]与[3,4]而不等新gpos。来源从快照的实际parent逐height执行并验证连续性/pre-state root，直到当前prev再投影请求区间；不得把第二批直接从genesis重新执行，也不能要求快照parent恰好是本批prev。当前完整09历史负责证明最近相关RESUME存在或不存在。不存在时，普通cursor推进不要求source控制coverage推进到新receipt written_at；存在时，source必须精确处理其配对Resume位置/序号，且declared coverage覆盖该Resume。09的Set日志在Resume之前，按Set<=written_at选择后仍须检查后面的paired Resume，GLOBAL后继也不清除此条件。独立预期见[source-projection-review](../../verification/batch-services/source-projection-review.md)的SP01–SP06。11 lead接受该来源解释并冻结r2。

## 5. Batcher与Certifier状态转换

### 5.1 Build与任务身份

`Build(prev,J,source)`要求prev是Batcher自己的09 reader当前finalized已交付cursor；`framesDigest=0`，`J.first=prev.height+1`，长度1或2，算术checked，区间存在于同一source snapshot且未被该次恢复切面覆盖。来源不足Wait，已知不符Reject。按03执行得到Kept，再唯一编码y。构建不改变Inbox cursor。

job identity采用无歧义固定preimage：`Keccak256(ASCII("PoC/BatchJob/v1") || chainId_u256ABIword || inbox_raw20 || configRef_raw32 || prevHash_raw32 || commitment_raw32)`。这里prevHash来自09，commitment来自§2。job与不可变完整prev/payload/source_ref共同保存；哈希命中仍比对完整内容。相同身份异内容为Conflict，不覆盖。一个job固定requestId，建议直接为job_id；不能在重试时随机换requestId。

```text
BUILT -> DA_READY -> CERTIFIED -> SUBMITTED -> ACCEPTED
              WAIT保留阶段；StalePrev/明确失败保留可审计终态，不删除历史
```

BUILT保存完整prev/y/source；DA_READY保存d和receipt；CERTIFIED保存完整BatchSubmission/签名；SUBMITTED保存实际signed raw transaction、txHash、nonce和请求身份；ACCEPTED仅保存真实original CursorAdvanced及其finalized位置。状态推进与其结果必须durable后才对外可见。

raw transaction须在发送前持久化，断在发送或收据返回前恢复时查该txHash/原requestId的实际结果，未知时保留WAIT/SubmittedUnknown。允许原样重发同一raw tx，不得改nonce/requestId盲重交。成功幂等重试receipt没有新Advanced时，查原接纳事件并关联原结果，不创建第二accepted。确定stale prev后保留旧任务和签名，不覆盖或将它改签成新prev；按新cursor创建新job且不再覆盖已接纳/RESUME跳过高度。

### 5.2 Certify

按以下顺序检查语义（多个同时坏字段的错误优先级不作要求）：

1. 自己的09 reader已取得完整当前CursorRead与写入位置；请求prev逐字段/bytes完全相等。只比较height不够。已知stale prev Reject；历史未齐Wait。
2. 自己source已满足§4最新相关RESUME处理条件；未处理Wait。
3. `decode_closed`、commitment打开和真实DA receipt验证通过；取回或核对的bytes必须与请求完全一致。已知坏签名/坏hash Reject；DA暂时不可取回Wait。
4. prev.framesDigest为0；records从prev.height+1开始、恰1或2且连续。逐height独立Apply并比较全部record metadata origin和Kept。
5. 计算next：height为末条height，origin为末条epoch，framesDigest=0，sourceKind=1，source精确等本次source gpos bytes；first/last来自records。next不是请求者自由指定的答案。
6. 持久化完整attestation、采用的source/cursor证据和签名后返回Certified。输出不能绕过显式actor处理/提交边界。

Certifier缓存按完整请求（prev、payload、d、receipt）与实际采用的gpos/domain绑定，命中仍执行当前prev和恢复检查；不能照搬Inbox的“命中成功rid先返回”规则。同一请求/来源有效重试返回原签名bytes。换identity/domain/config或来源冲突时禁止复用旧journal。Certifier可以在签名后L1才发生变化：旧签名保留，Inbox当前prev检查负责拒绝其接纳，不承诺签名与纳入之间没有竞态。

## 6. DA与持久化边界

DA只保管不可变`d -> y`及对应domain/身份下的receipt，P0本阶段不GC任何已确认对象。可配置unavailable/delayed，但不能在重启时遗忘已确认bytes。保管者/保留期后的完整DA-RET活性仍是系统假设，有限测试不证明无限保管。

`Disperse(y)`先做准入大小检查并计算d。未存在时在本任务专属目录写临时bytes，完整写入、`file.sync_all`，以不会覆盖已存在不同内容的原子方式发布d文件，再`parent_directory.sync_all`；初始化目录链本身也需落盘。只有此持久边界完成后才能签DAReceipt。随后将receipt和绑定身份/domain的元数据写入durable记录并同步文件/父目录，**完成后才返回receipt**。签名故障不能留下“已认证但没有payload”；bytes已持久化而receipt未持久化时，重启可在验证原bytes后重新生成并保存相同有效receipt，先前未返回签名不算已确认结果。

已存在同d时逐字节/hash核对y并复用保存的receipt；不存在的数据返回Wait，不用空bytes。不同bytes、存储损坏或不匹配domain/identity不能覆盖原对象，报告明确corruption/conflict并不发新receipt。`Retrieve(d)`对实际读取bytes重新计算commitment并验证保存receipt；临时不可用Wait，mismatch报告`Wait(DataIntegrityUnavailable)`并附损坏诊断，不返回可用DAObject。下游不得把其归类为没有batch而fallback。

三角色的persistent root均有版本和身份manifest，至少绑定chain、Inbox、configRef、角色、signer/submitter、codec版本及来源fixture配置。重启须逐项匹配；换key/配置不能静默创建同身份的新journal。进程崩溃允许未同步暂存写丢失，不允许已返回/已签且发送的状态回退。签名/提交outbox先持久化后输出；处理结果与命令去重记录必须原子提交或可从已持久状态无歧义恢复。

显式崩溃边界至少包括：DA bytes写前、bytes sync之后receipt前、receipt sync之后回包前；Certifier签名保存前、保存后回包前；Batcher任务/DA结果/认证结果保存后，raw tx保存后send前、send后receipt前、接纳保存后回包前。保存signed state和terminal state，拒绝重启重新从空状态运行。

## 7. 实际Inbox reader与下游 `BS-ACCEPTED-v1`

消费09同一真实 `l1-adapter::Driver` 的独立reader gateway。`Read`只有在对应reader已`Deliver`的finalized prefix内才Ready，不能直接访问latest绕过gateway，不能因为底层Anvil有receipt便称finalized。读取缺失历史/receipt/配对控制Wait，完整矛盾Reject。实际Anvil RPC canonical执行及09配置/driver为P0信任边界；不宣称验证Ethereum PoS/MPT。

`AcceptedRecord`应为由gateway适配器产出的不可随意标成trusted的对象；公共JSON/任意EventRecord可导入成候选，但必须通过同样reader/receipt/来源检查才产出Ready。fixture导入路径必须携带fixture证据标签，不与真实路径混淆。

最小可消费记录语义：

```text
AcceptedRecord {
  chain_id, inbox, config_ref,
  position: poc_types::domain::L1Position,    // block/tx/交易receipt内log
  block_hash, transaction_hash, rpc_log_index,
  request_id, submission: l1_adapter::BatchWire,
  inclusion_timestamp_seconds,
  finalized_through, reader_visible_through,
  evidence_ref                             // 原始完整header/tx/receipt+driver命令结果
}
AcceptedPayload { accepted: AcceptedRecord, da_object, frame: ClosedFrame }
```

`position.log`按完整receipt全部日志从0计，过滤前枚举；RPC block-global logIndex单列。acceptance identity为chain/Inbox/blockHash/txHash/localLog，重复回放同内容幂等；同identity异内容Reject。只接纳配置Inbox的成功status=1且含原始CursorAdvanced的事件，核对其prev/next/d与完整tx calldata相等、caller/requestId、签名及domain一致；range/gpos从完整calldata保留。失败receipt、错地址、尚未finalize/deliver、只有submitted calldata、成功但没有Advanced的幂等retry均不创建新记录。

accepted记录存在不要求DA此刻可取回；下游加载`AcceptedPayload`时调用唯一`verify_payload`、CheckDA和attestation验证，核对record首末height/末origin与submission next/range一致。缺DA/损坏保留准确输入位置WAIT，不能跳过后处理dependent CursorSet。DA延迟不改变实际inclusion位置或sequencing-window判断。

CursorRead的当前cursor及written_at来自09实际历史。`AcceptedView`分别输出cursor_read、已物化连续virtual_prefix、missing_sources；普通接纳只在具体gpos来源可用且核对通过时追加。G来源未知仍可返回已验证accepted record，但不得按cursor高度制造virtual blocks。CursorSet保留具体Set与同tx Resume位置，并等待12/13的该位置ResumeCut；未取得时标记materialization pending，旧分支prefix只能作为历史缓存，不能宣称它仍是新cursor的连续当前G′。不以最新D head或整个block结束状态替代该切面。完整G′的来源接线保持外部gate。

## 8. 显式actor输入与时间

每角色只响应显式有序命令/mailbox输入，命令身份和完整内容去重；同id异内容Reject，无后台timer自动出签、提交或推进接纳。服务delay/unavailable/crash脚本只改变输入/回复何时交付，不能成为Certifier/G来源判断的全知输入。等待只返回缺材料/服务理由，不能读wall clock触发协议推进。

本阶段driver由脚本显式Tick/Submit/Mine/Finalize/Deliver；11 actor把durable effects和outbox结果作为当前命令的一次输出交给host，host显式送下一服务或09。完整02 grant/ack封装尚未接线，标INCONCLUSIVE；不能把组件脚本称为受控15-worker运行。已有签名即使在延迟中到达仍按实际Inbox状态判断，clock/transport不会抹除它。

## 9. 独立向量和组件矩阵

Verifier必须先从本合同和来源生成expectations/独立encoder、Keccak/EIP712与恢复地址期望，再读候选。Checker不得调用生产codec/hash helper产生expected。公开最小向量：genesis parent metadata `(h=0,origin=0,timestamp_us=0)`，账户1为nonce0/balance10，账户2不存在；每高度DEP仅03匹配metadata的Attr，没有Credit；h1/h2时间分别1,000,000和2,000,000μs、origin0。h1 raw普通交易顺序：Transfer(1,2,nonce0,amount3)、Transfer(1,2,nonce0,amount1)、Transfer(1,2,nonce1,amount100)；Kept仅第一条，结果账户1=(nonce1,balance7)，账户2=(nonce0,balance3)。h2可用Transfer(1,2,nonce1,amount2)，Kept该条，结果账户1=(nonce2,balance5)，账户2=(nonce0,balance5)；独立padding向量h2 raw为空，Kept为空且沿用h1余额/nonce。

向量至少含：单h1、h1/h2、h1/空h2完整payload hex、channel_data/长度/channel_id/d、03交易原bytes、真实DA/Certifier签名和每一digest preimage。crypto离线domain可沿09固定chain31337/configRef=`11`重复32B/Inbox=`0x2000000000000000000000000000000000000002`；真实Anvil样本必须改用真实部署域重新签。fixture gpos建议UTF-8 `bs11-fixture-gpos-1`，仅为opaque句柄，另列typed位置与source_mode，不声称该字符串是公共ReplayPosition编码。

| ID | 输入/边界 | 必须结果 |
| --- | --- | --- |
| B01 | 上述1/2height/空padding黄金向量 | 生产bytes与独立expected逐字节一致；Kept过滤和03Apply后状态一致；DEP未进入payload。 |
| B02 | 错magic/version/channel_id/number/is_last/count/len、尾字节、截断、非连续、u64 wrap、超count/bytes、Credit/Attr | 明确Reject，不部分decode/执行。含1条max height编码结构向量与接prev溢出拒绝。 |
| B03 | 正确DA/Certifier签名；chain/Inbox/configRef/type/prev每字段/next/d/receipt/range/gpos逐字段替换，wrong signer/high-s/v/长度 | 真实密码学验证正例通过、单字段负例拒绝，保留preimage与恢复地址。 |
| B04 | 同y重复Disperse/重启；不可用/读取缺失/已存bytes损坏 | 相同d/原receipt/原bytes；不可用和损坏不产有效DAObject，不以空代替。 |
| B05 | DA规定三个crash边界及回复丢失 | 未确认状态可恢复，任何已返回receipt对应bytes均持久；重启不改身份或已确认对象。 |
| B06 | Certifier来源缺height/DEP/state/gpos、内容错、source snapshot不同、已知stale prev | 缺材料Wait，已知错内容或stale Reject；不会签名；真实独立Apply验证。 |
| B07 | Certifier缓存后cursor改变，RESUME未处理，RESUME后GLOBAL cursor | 缓存不能绕current prev/最近相关RESUME；落后Wait；已知旧prev Reject。 |
| B08 | Certifier签名前/保存后回包前crash；identity/config变更 | 原有效请求恢复原结果；异身份journal拒绝；持久签名不丢失。 |
| B09 | Batcher各持久阶段crash/restart、重复命令、同id异内容 | 原job/prev/y/d/receipt/签名/requestId保留；冲突拒绝，不倒退终态。 |
| B10 | raw tx保存/send/receipt间crash或回复丢失 | 原txHash和raw tx恢复；先核对未知结果；不盲造新nonce/rid；只一次实际接纳。 |
| B11 | 本机09真实Anvil至少两批，含状态改变、invalid nonce/balance过滤、连续段 | 实际配置Inbox的status=1/finalized/delivered Advanced；payload、原始receipt/tx、公开签名可独立验证；标明G fixture。 |
| B12 | submit/included/finalized/delivered分开，错误地址/status0/无Advanced retry/错误metadata | 只有成功且reader已交付真实接纳创建record；精确local log与rpcLogIndex保留；重复不再接纳。 |
| B13 | 真实Override后使用事前已签旧prev提交 | 实际已mined status0/StalePrev；Certifier新请求未处理该RESUME则Wait；不声称完成恢复切面。 |
| B14 | 真实accepted记录后DA unavailable或损坏，再恢复 | accepted记录保留；AcceptedPayload在原输入Wait；恢复后同bytes/d，inclusion位置不变。 |
| B15 | 接纳来源gpos缺失；CursorSet无ResumeCut | cursor可报告，物化prefix明确不足/Wait；不制造完整G′或复制D。 |
| B16 | 不同墙钟暂停的相同显式脚本；三个角色独立fault开关 | 同虚拟/协议输出与状态摘要；wall耗时另记；故障不泄漏为公共规则输入。 |
| B17 | 12消费同crate frame接口/真实DA与accepted样本 | 可复用接口与原始样本交付；12实际D/恢复派生验收由其owner完成，当前不自称PASS。 |
| B18 | 07/08真实G来源和gpos codec；02真实受控actor接线；12/13位置恢复切面 | 完整集成gate，本阶段INCONCLUSIVE；组件成功不增加完整accepted功能计数。 |

真实negative必须保留已mined status0收据，不能只用eth_call revert代替B13。crate单元测试覆盖拒绝guard，即使模型只注入crash不注入恶意角色；这些负例是验证，不改变角色威胁模型。B05/B08/B09/B10包含真实服务进程停止/重开持久root的样本，纯对象drop/reopen另标fixture，不当作全部crash边界证据。精确I/O边界可用受控故障钩子产生单进程的exit/重启，必须记录实际边界和落盘文件。

## 10. 交付顺序、冻结与保留gate

先发布小接口提交：本合同冻结版本、独立source expectations/向量、`BS-FRAME-v1` codec与commitment/accepted-record形状及检查入口；明确此时真实收据样本尚缺（如尚未运行）。然后持续完成DA、Certifier、Batcher和真实Anvil接纳，尽快交付至少两批真实receipt/DA样本，供总协调向12分发。接口consumer不必等main或01。

源码候选由lead提交后，Verifier和Tester检查同一准确SHA。修改产生新候选，复验受影响矩阵。每个交付包含baseline/interface/candidate/verified/tested SHA、状态、模型/角色可见信息、worktree/branch、独立报告、精确命令/seed（显式脚本写`none (explicit script)`）、工具版本、原始RPC/receipt/payload/signature/journal/fault traces绝对路径及SHA256摘要。real Anvil evidence和source fixture分别标记。

保留gate：07/08来源和公共gpos；02完整受控grant/ack与故障运输；12真正D和位置绑定ResumeCut；13恢复链路；01/I1及15-worker整体；无限期DA-RET/A-DELIVER/CERT-5预算。09既有合约通过不能替代11持久性，11组件通过不能替代完整功能。不得改09 ABI/driver以迁就测试，不合main、不push，不向其他用户chat发消息。

冻结栏：11 lead 于2026-11-16接受独立[合同审查](../../verification/batch-services/contract-review.md)的“PASS for interface freeze”，冻结r1。来源先行expectations SHA256为 `badea23abf7751499692f992530e233073de196197b5196d9526681a86e543e4`；frame黄金向量为 `947b3b1c5fba237d0f72d5ad3e574bb661d58f0314928f0986ab3559d392910b`（6合法、19非法、1最大边界）；签名黄金向量为 `9e9a41ba7801dd307199a787e23d6e2646e699f802e9800eaf3a4ec826eb08f2`（2真实离线签名、22字段变异、6malformed）。这些是实现前独立向量，不是候选通过或Anvil接纳证据。接口源码SHA将在M1交付报告中记录。任何上述字节、身份或guard变更均先修订合同并由Verifier复审；不得复用v1名义悄悄改变bytes。
