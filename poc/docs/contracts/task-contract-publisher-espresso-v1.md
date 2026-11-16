# PUBLISHER-ESPRESSO v1 — 独立组件合同

日期：2026-11-15。版本：`publisher-espresso-component-v1`。状态：**06 lead已冻结，独立Verifier合同审查PASS；本次只授权独立组件，不等于完整06 accepted。** 实现基线 `0a951c482a9712678ab9de34765b781b9c3c9f42`（起点 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，正常 fast-forward 合入管理政策）。接口和独立 expected 提交后才开始 Builder；候选源码先提交，Verifier复核该SHA后Tester运行同SHA矩阵。

冻结澄清：重复finalize为纯幂等，不推进事件tick前沿；reader接收tick必须不早于记录finalized_at，message先于header时延后到header到齐复核；只有已保存完整Offer验证结果及payload的任务才可转RECORDED，VERIFYING或未经验证的STOPPED不能仅靠receipt升级。独立期望与literal向量见 `poc/verification/publisher-espresso/`，先于任何Builder实现形成。

## 1. 权威、阶段和所有权

| 来源 | 本合同使用的规则 |
| --- | --- |
| [v2 §§4.1、4.5、5.1、5.3](../../../formalization_v10/poc_design_zh_v2.md) | 整数微秒，virtual/public/local/wall分开；独立TimestampPolicy；proposed/finalized/reader_received分开；所有协议输出受控。 |
| [v10架构 §1.4](../../../formalization_v10/00_arch_zh_v0_1.md) | E-S前缀、完整消息列表和非递减θ；E-TS相对proposal-time且有live条件；watermark只依赖E-S。 |
| [v10 Publisher §§3.1–3.5、PUB-1–3](../../../formalization_v10/04_publisher_zh_v0_1.md) | 完整结果验证、缺材料WAIT、key去重、持久发布、完整finalized确认、公开停止。 |
| [v10 GFR §§3.2–3.5、4.7](../../../formalization_v10/09_finality_rules_zh_v0_1.md) | 公开完整与本地缺失分开；对象原始公开位置；q_avail由07计算；同θ按q排序。 |
| [分层依赖v2](../commander/dependency-plan-v2.md) | 06可以先完成独立组件；真实02/03/04仍是集成gate。 |
| [06](../handoffs/06-publisher-espresso-zh.md)、[07](../handoffs/07-viewer-gfr-zh.md)、[08](../handoffs/08-regular-integration-zh.md)、[16](../handoffs/16-espresso-compatibility-zh.md) | 日志/reader消费边界、实际区域证书联调、官方来源适配。 |
| pinned Regional BFT `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，`DESIGN.md` §§1–2.1、3.1；Commonware `2026.10.0` | 证据/完整交付不同；native证书由04验证。这里不改变BFT、签名域或quorum，不采用15人benchmark profile。 |

上述v10/v2规则记为 **S**；本合同具体整数、codec、事务API、容量和重试动作属于 **A：06工程适配选择**。A不能重定义03 canonical对象或07的GFR。

| 路径 | 唯一writer |
| --- | --- |
| 本合同、`poc/docs/publisher-espresso-preparation-v1.md` | Docer，冻结后由06 lead管理 |
| `poc/crates/sim-espresso/`、`poc/crates/publisher/`（含各自独立workspace/lockfile及回归） | 单Builder |
| `poc/verification/publisher-espresso/` | Verifier；Tester仅写明确移交的run/report文件 |
| root workspace/lock、types/core声明、其他crate、native/proto、commander和子模块 | 只读，不在06权限内 |

本阶段不写EvidenceTask/L1处罚：P0无staking/slashing，06交接的本次范围为PublicationTask。保留将来独立中继职责，不借此删除区域publication。不调用真实网络，不启动实时timer，不自行读墙钟推进。无背景线程/网络输出；函数只消费显式输入，返回待02受控调度提交的effects。此组件API不声称自身提供01/02 root grant/Ack认证。

## 2. 最小可消费接口 PE-LOG-1

以下字段和语义冻结；Rust方法名可用snake_case，错误必须可区分类别，具体导出签名在首个接口提交补一份API映射。两个crate优先std-only；不能修改根manifest。

```text
TickUs(u64)                       // 仅调用方显式授予的virtual参考时间
PublicTimeUs(u64)                 // θ；不同语义，禁止由读取者墙钟构造
EspressoPosition { q:u64, message_index:u32 }
LogMessage { codec:u32, bytes:Vec<u8> }
ProposedBlock { q:u64, theta_us:u64, proposed_at_us:u64,
                live_at_proposal:bool, messages:Vec<LogMessage> }
FinalizedBlock { proposal:ProposedBlock, finalized_at_us:u64 }
ReaderReceipt { q:u64, reader_received_at_us:u64 }
```

`q`从0连续递增；空日志tip为None，不能把q=0当空。message_index从0开始，空列表是可验证的完整列表。`codec`标识上层payload编码；06不解析/重写03的cid、PhysicalBlock、RegionalResult或签名字节。codec=0为未指定，拒绝；组件fixture使用明示测试codec，不能称03生产编码。实例配置持有不透明`source_id: Vec<u8>`（1..256字节），只作日志来源隔离，不重新定义chain/config/cid。

public ordinal、θ和各观测tick不可互换。`proposed_at`与`finalized_at`是模型virtual时间，`reader_received_at`是该reader收到完整/片段数据的virtual时间；OS时间仅在外部诊断。finalize不得改写proposal字段。finalization tick不小于proposal tick；同源显式proposal/finalize事件tick不倒退。最终公开顺序按q，绝不按网络到达或θ重新排序。

### 2.1 TimestampPolicy

```text
TimestampPolicy = Offset { offset_us:i64, quantum_us:u64 }
                | Fixed  { theta_us:u64 }
TimestampAssessment = WithinBound
                    | OutsideBound { deviation_us:u64, bound_us:u64 }
                    | NotAssumedNonLive
TimestampError = InvalidQuantum | ArithmeticRange | Regressed
```

Offset先在足够宽的checked整数中算`base=proposed_at+offset`，要求0..u64::MAX，再取`theta=(base/quantum)*quantum`；quantum>0。不取max(last_theta,theta)，不裁剪负值。Fixed每次给同一θ。policy只在显式propose输入调用，可在下一提议前由场景显式换policy；历史proposal/finalized不可变。

若有前块，`theta<previous_theta`为`Regressed`/无效源，整个提议拒绝且q不消耗；`==`合法。E-TS判定为`abs(theta-proposed_at)<=delta_ts_us`，等号在界内；只有`live_at_proposal=true`时出Within/Outside。non-live时为NotAssumedNonLive，即使以后live时finalize也不追认。非递减但越界仍允许提议/finalize/读取，保留OutsideBound；不能当日志分叉，也不继续宣称时限前提成立。冻结θ而q增加是合法E-S输入，watermark永不越过冻结值。

### 2.2 日志驱动和错误原子性

语义操作：`propose(tick,live,messages)`、`finalize(q,tick)`、`finalized(q)`。propose分配下一个q并保存完整有序列表；finalize仅允许下一个尚未finalize的proposal（从0开始），不得跳过缺失q；未知/未提议q为错误。同一已finalize q的重复调用返回原记录，不改首次finalized_at；试图替换内容拒绝。消息顺序、重复消息和空消息列表均原样保留；去重属于Publisher任务/接收请求，不擅自消除公共日志中的重复publication。

每次操作先检查全部输入和算术，再整体安装；失败保留q、tick前沿、policy和所有已有记录。最大单message bytes=65536、单block messages=1024、最大encoded block bytes=1,048,576（包含header/长度），边界允许，超界拒绝。不得为读取缺数据返回空列表。component有界存储可返回具名Capacity，不能逐出已finalized前缀。计数耗尽具名错误，不能wrap。

### 2.3 PE-LOG-WIRE-1

本codec仅编码06日志envelope，内嵌payload bytes原样；不作为03 canonical codec或签名preimage。

```text
magic = ASCII "PELG" (4 bytes)
version:u16 LE = 1
source_id_len:u32 LE; source_id bytes
q:u64 LE; theta_us:u64 LE; proposed_at_us:u64 LE
live_at_proposal:u8 (0 or 1)
finalized_at_us:u64 LE
message_count:u32 LE
repeat message_count: codec:u32 LE; byte_len:u32 LE; exact bytes
```

仅FinalizedBlock可用本wire（未finalize proposal不冒充finalized）。严格消费全部字节，未知version、坏bool、尾随bytes、截断、非法长度/codec、时间逆序拒绝；分配前检查长度和总界。decoder只完成结构解码，不由“能decode”推出来源真实性。E-S连续性由reader适配器校验，官方16来源需要自己的finality验证依据。配置E-TS bound不进入wire，assessment可由保存的proposal字段和配置重算，不用reader到达时间。reader_received不进入公共wire，分别保存在reader状态/trace。

## 3. 独立reader与完整前缀 PE-READER-1

对07/16公开reader-neutral语义trait：

```text
FinalizedLogReader:
  source_id() -> &[u8]
  tip() -> Option<(q,theta)>
  read(q) -> Complete(FinalizedBlock) | Wait(MissingLog) | InvalidSource(reason)
  require_prefix_through(q) -> VerifiedPrefix | Wait{next_q,missing} | InvalidSource
  require_watermark(x) -> VerifiedPrefix | Wait{next_q,missing} | InvalidSource
```

`VerifiedPrefix`的普通构造入口私有，仅通过ReaderState连续性/完整性校验产生。官方16adapter先验证其来源finality与完整列表，再调用同ReaderState接收并委托实现trait；不要求复制SimEspresso全局存储。Rust trait本身不能阻止恶意实现，生产绑定/来源验证仍属接线可信边界；这里禁止在本组件默认入口接受一个裸bool作为verified依据。它提供source、连续完整tip及按q读取能力，不给07一个任意跳跃的max_received。这里的verified指完整来源日志/E-S核对，**不等于每个publication已过CheckFINAL、正文完整、G接纳**。

独立`ReaderState`绑定source及reader identity。输入`receive_header(q,theta,proposed,live,finalized,message_count,received_tick)`和`receive_message(q,index,message,received_tick)`；亦可提供`receive_block`方便操作，语义必须等价于整体原子完整交付。header对完整message_count作出来源承诺；在Sim模型中输入来自本日志已finalized record，不能从未finalize proposal生成。独立源适配器负责对header/messages做真实性验证，测试fixture不等于实际Espresso共识证明。

收到q>next或缺message时缓冲，不泄漏为可消费前缀。每个message index恰好覆盖`0..count`才完整；count=0立即完整。收到message但header缺失可缓冲，不能推断count。重复相同header/message幂等，保留首次接收tick；相同q/index内容冲突或source不符是InvalidSource，保留已验证前缀，不能覆盖。message index超header范围拒绝；缓冲后header揭示越界必须报告冲突，不能丢掉冲突后视为完整。

连续前缀只从q=0向后推进，要求每块完整、来源一致、theta非递减、proposal/finalized字段合法。q较高的完整块不能掩盖缺口。`read(q)`仅在q≤visible tip时返回Complete；否则Wait，包括本地已缓冲的更高块。`require_watermark(x)`在本reader完整前缀内有tip.theta≥x时成功；同θ后续blocks不影响已覆盖“严格小于x”的结论。未收到更晚block或θ冻结都不能由wall time补水位。

**两种缺失必须分开：** (1) 来源已公开message，但本reader尚未收到其bytes或完整header：日志prefix WAIT；(2) proof-only publication本身完整收到，而所需physical正文尚未在公共历史出现：该block仍推进日志prefix，具体result为NotPubliclyComplete，由07判定，不能阻塞日志。06只保留每个opaque对象原公开位置；缺引用正文不能用publisher本地缓存填成已公开。reader A前进不改变B；读取底层未来文件/SimEspresso全局对象补齐B禁止。reader snapshot/recover保留已验证前缀、pending fragments和首次receipt，重复交付不重置位置。

## 4. Publisher边界 PE-PUB-1

采用关联类型或泛型消费03对象：`Key`的语义为`(cid,H(B))`；cid/hash/result及其canonical bytes完全由03adapter提供，06不得定义另一套hash/PhysicalBlock。可以声明通用`PublicationKey<Cid,Hash>{cid,block_hash}`容器，不规定Cid/Hash位宽、hash算法或编码。

```text
PublicationTask<Key,Result> {
  key, result, phase, missing,
  verified_payload:Option<Vec<LogMessage>>,
  attempts:Vec<Attempt>, recorded_at:Option<RecordedAt>,
  stop_reason:Option<PublicStop>
}
phase = VERIFYING | QUEUED | SUBMITTED | RECORDED | STOPPED
RecordedAt { publication_position:EspressoPosition, complete_through_q:u64,
             source_id, verification_basis }
Attempt { stable_id, number, exact_payload, created_tick,
          state:PREPARED | DISPATCHED | SERVICE_ACKNOWLEDGED }
```

`verification_basis`是不透明但持久可追溯来源引用，不可只存`true`。记录源id、publication位置和所依赖的前缀tip/对象引用。`publication_position`指确认的那条publication，`complete_through_q`指本次核验完整性实际使用的完整前缀q上界，两者不得混用：proof位于q1而所需body位于q2时不能把q1当成完整收录依据；必须保存至少覆盖q2的complete_through_q。这是收录验证basis，不是06重新计算07的最早q_avail。Publisher core还要检查`publication_position.q <= complete_through_q <= reader.tip.q`、publication message_index在该block范围内，以及source/key精确匹配；任何不满足者拒绝且不填recorded_at，不能仅信opaque verification_basis。`Result`可保存候选组成部分，missing为具名且可序列化的列表。verified_payload在完整Offer通过后冻结，后续提交同字节；任务不因Program TIME/Close、Manager切换或重启删除。

### 4.1 验证trait

```text
PublicationVerifier:
  key(result) -> Result<Key,Reject>
  verify_offer(result,explicit_public_inputs)
     -> Verified{key,complete_result,publication_messages,validation_basis}
      | Wait{key,partial_result,missing}
      | Reject{reason}
  verify_recorded(task,reader_prefix,explicit_public_inputs)
     -> Recorded{key,publication_position,complete_through_q,verification_basis}
      | Wait{missing}
      | NotYetPublic
      | Reject{reason}
```

生产adapter必须执行CheckFINAL、exact anchor provenance、目标身份/正文hash、同cid且从anchor连接到目标的完整physical ancestry。已知wrong cid/body/proof、断链或错anchor REJECT；欠历史配置、来源或正文WAIT且保留任务。未知验证器/codec版本REJECT；不能默认通过。trait由真实03/04适配器实现；组件fixture只测试本trait的调用和状态传播，必须显式命名为non-cryptographic fixture，不提供生产always-true实现，不把其PASS称密码学PASS。

`verify_recorded`必须从**该Publisher reader已验证完整前缀**重读相应消息，验证key匹配及完整publication（包括只引用此前已公开正文的合法情况）；service submit ACK、全局存在但未交付的finalized块、proof-only、未finalize proposal都不能RECORDED。若多个合法位置可确认，选择当前完整前缀中最早可确认的publication位置并保存本次所用complete_through_q；首条已验证receipt以后不改写，重复确认不产生新task或Recorded effect。它不计算07的q_avail或G接纳；可选anchor材料与GFR完整性差异保持v10 §3.2。

### 4.2 Offer、补齐与去重

首次Offer先建立可恢复VERIFYING任务，再按verify_offer输出原子更新：Verified→QUEUED；Wait→VERIFYING+missing；Reject输出具名错误，不产生可提交任务。若在persist前验证失败，可以不新建无效任务；同key已有任务不得被坏重Offer破坏。

重复Offer按key而非raw result bytes去重。合法不同proof可以对应同一key；不能因其bytes不同自动判冲突。已完整验证的任务保留首次验证的result/payload及全部terminal/attempt状态，不因重复Offer回到QUEUED；重复不产生新任务。VERIFYING任务可用新候选完整替换待验证材料（再次运行全部验证），或显式`reverify(key,updated_result,inputs)`；替换必须仍导出同key，不做未经校验的拼接。一个无效补齐返回REJECT但保留旧待补材料。已RECORDED/STOPPED仍按key去重，不能复活。

### 4.3 提交与crash边界

`prepare_attempt(key,tick)`只对已验证、未recorded且未public-stop任务允许；首次或明确Retry输入创建下一个单调number和stable_id，把exact_payload及PREPARED意图持久化后返回待调度effect。同一个尚未确认dispatched的PREPARED意图重放返回原stable_id/bytes，不创建新attempt。无自动后台重试或wall deadline。stable_id由持久store namespace+key关联任务序号+attempt number确定，不能由进程PID/重启次数/墙钟生成。

PREPARED期间phase保持QUEUED（已有SUBMITTED任务的重试则保持SUBMITTED）；`mark_dispatched(stable_id,controlled_delivery_ref)`在调用方确认该effect已纳入受控输出提交后持久改为DISPATCHED，phase→SUBMITTED。外部提交成功而mark未落盘时crash，恢复重发同一个PREPARED stable_id；服务接入必须按stable_id幂等或容忍同publication重复。服务ACK另记SERVICE_ACKNOWLEDGED，仍SUBMITTED。ACK未知id、id匹配但payload/引用不匹配，均拒绝不改其他任务。输入确认顺序允许日志确认先于ACK：先RECORDED后迟到ACK不得降级。

正常retry在前一次DISPATCHED/ACK后由外部显式输入发起，新的attempt number持久递增，任务key与payload不变。同一待发effect有持久重放义务，crash重开不丢弃。返回effect不是网络发送；02集成必须把store提交和root grant/Ack custody相接后才能释放真实输出，当前file-store不能替代该gate。

### 4.4 公开停止与已收录确认

STOP输入由可信GFR公共重放adapter提供，含来源position和精确scope；06不从任意裸`other_cid`或墙钟构造停止理由：

- SessionClosed只作用匹配session的任务；BranchClosed只作用匹配branch。
- SessionAssigned(other cid)仅在**同一session候选选择范围**排除该任务cid时适用；其他session正常Assigned不会清空历史Publisher任务。
- 截止watermark须由该reader完整finalized前缀提供，且theta≥该任务已验证session截止键；等号停止。未完整reader/header-only水位不够。
- Program TIME/本地Close/Manager切换仅诊断，不改task phase或重试许可。

为了不重新定义03 lifecycle，组件消费`StopDecision<Key>{affected_keys,reason,public_basis}`或等价trait投影；生产adapter负责证明scope。测试必须包含未匹配key不变。PublicStop持久单调，一旦设置，后续新attempt永远禁止；已经持久PREPARED但未dispatched的effect不再释放，历史DISPATCHED消息可仍在途。

RECORDED优先于STOPPED：已有recorded_at时保存stop_reason但phase仍RECORDED；未recorded时phase→STOPPED，保留所有payload/attempt。STOPPED后收到已验证完整历史publication确认，可以填recorded_at并phase→RECORDED，同时保留stop_reason；不能因此产生新attempt。这样满足`phase==RECORDED iff recorded_at.is_some()`，公开关闭不会抹掉已经或后来核明的历史收录。Confirmed记录不等于GFR接纳，也不要求该publication符合GFR截止。

## 5. 实际持久化、恢复和受控边界

必须交付一个实际file-backed store，不能只有Clone/内存SnapshotStore测试。允许纯状态转换+通用snapshot trait，至少一个file adapter通过`StateCodec`编码/解码整个Publisher状态；generic Result/Key由调用方提供规范编码器。存储envelope有明确magic/version、长度界和完整性校验；错误版本、截断、损坏、解码后不变量破坏显式失败，不默认空状态。文件codec不是03对象codec，不改内嵌bytes。

A选择：同目录temp文件写入全部bytes→file.sync_all→rename覆盖主snapshot→parent directory.sync_all；或具有同等已证明crash边界的journal。只允许一个writer持有同一store。成功返回前完成耐久提交；提交失败无对外effect、无半个内存状态。rename前失败旧主文件仍权威；rename后directory sync失败属于**commit outcome unknown**，关闭当前store并要求重新打开核对，不能假设回滚继续写。恢复只能选完整主snapshot，遗留temp不自行提升为已提交。每次更新单调generation；恢复校验namespace/配置绑定，防止打开另一run的任务。

必须真实重开文件验证：enqueue前/后，prepare意图前/后，dispatch前/后，service ACK前/后，finalized但尚未reader交付，Recorded提交前/后，STOP提交前/后。同key、exact payload、stable attempt、终止标记和recorded_at恢复；不可对缺失/损坏主文件自动重新初始化已有任务。reader snapshot至少提供可编码/恢复的独立状态及相同不变量；若用同file-store模板，保持reader namespace与Publisher分开。

## 6. 独立验收矩阵与C3固定输入

Verifier先从S来源写expected，接口冻结前附独立向量；不能用Builder输出生成expected。以下均为本次组件必须执行的有限矩阵，具体测试函数/命令由接口提交附录映射。

| ID | 输入/触发 | 必须结果 |
| --- | --- | --- |
| PE01 | q0/q1/q2，θ=100/100/101；空block与重复message | q总序保留，空列表完整，重复不擅删。 |
| PE02 / C3 | delta=1000；(q,proposal,theta)=(0,10000,9000),(1,10000,11000),(2,11000,11000)；独立proposal10000/theta11001 | 前三条Within，q1/q2同键合法；独立超前条Outside by1但E-S合法。 |
| PE03 / C3 | delta=1000，Fixed θ=10000，proposal=10000/11000/11001，均live | q增加；Within/Within/Outside by1；watermark(10001)仍WAIT。 |
| PE04 / C3 | Offset0量化10，proposal=109→110 | θ100→110；以proposal判deviation9/0；quantum0和负/overflow拒绝。 |
| PE05 / C3 | θ10000后提议9999；non-live proposal10000 θ0后tick100000 live时finalize | 回退整步拒绝、q不消耗；后者NotAssumedNonLive不追认E-TS。 |
| PE06 | propose后未finalize；跳q finalize；重复finalize不同tick | reader无可见新块；跳号拒绝；重复保持原finalized记录。 |
| PE07 | codec往返；count=0；截断/尾随/坏bool/未知version/大小边界 | 精确原bytes和位置；错误原子拒绝，不能分配超界。 |
| PE08 | A收0,1,2；B先2再0，q1 header缺一个message，最后补齐 | B依次tip None/0/0/2；A状态不泄漏，B不提前watermark。 |
| PE09 | proof-only完整q0，body在q2；B本地缓存body但未读q2 | q0前缀可推进；缓存不成为公开body；q_avail留07，非06自定。 |
| PE10 | exact/conflicting duplicate、wrong source、message先header、越界index | exact幂等保留首次tick；冲突InvalidSource保留已验证前缀。 |
| PE11 | Offer missing ancestor/anchor/config；wrong cid/body/proof/ancestry | WAIT保留或具名REJECT，无submission；完整输入经trait验证后QUEUED。 |
| PE12 | 同key相同/不同合法proof重Offer；无效补齐；不同key | 一key一task，合法proof变体不按raw bytes冲突，旧verified状态不降级；不同key独立。 |
| PE13 | prepare→file重开→dispatch→service ACK→finalized未交付→完整reader确认 | 意图同ID重放；QUEUED→SUBMITTED→SUBMITTED→SUBMITTED→RECORDED。 |
| PE14 | TIME；无关Assigned；匹配公开排除；deadline-1/=deadline | TIME/无关scope不变；匹配/等号停止；不丢历史recorded；STOP后迟确认可RECORDED不重试。 |
| PE15 | storage fault各边界、损坏/错namespace、旧temp | 无提前effect/半state；真实重开保留任务/attempt/stop/recorded；损坏不当空库；unknown commit关闭重开。 |
| PE16 | 同逻辑输入和seeds 7/42/99，wall pause 0/2ms | 规范virtual trace、日志、reader前缀、publication状态与attempt完全相同；wall字段分开。 |

固定C3向量的单位均为μs，delta_ts=1000μs是该测试profile，不能与replica δ_G相互替代；量化算术为独立缩放用例。时间测试可纯确定性，无随机性时明确seed=N/A。PE16 seed仅控制外部输入/交付排列，不能参与q或时间policy随机采样。文件I/O与wall pause测试不量化协议latency。

命令都从本worktree明确运行；Rust build/重型test/simulation一律`flock /tmp/megaroation-poc-heavy.lock <command...>`，`CARGO_BUILD_JOBS=2`且不嵌套。Rust1.95.0；组件各自`--locked --offline`，数据盘专属 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/06`，独立cargo/target/artifacts。保留既有.build再安全迁移/链接。原始证据记录精确candidate SHA、dirty状态、命令/seed/profile、输入与trace hash、绝对路径、wall/virtual范围及PASS/FAIL/INCONCLUSIVE。

## 7. 精确依赖与完整06未过gate

| gate | 必需上游/消费者；当前状态 |
| --- | --- |
| D03 | 已接受03 public types/codec/ResultKey、AnchorEvidence/ancestry验证、明确验签调用边界和StateCodec绑定的准确SHA。只读参考 `183430e71e5d61533f54a76b2d7d22c5021f1704` 是PREPARATION草案，MPX-PREP-LE64-SHA256-v0不是已接受生产codec；当前不依赖其代码。 |
| D02 | 已接受02 endpoint/envelope、controlled grant/Ack effect持久custody、服务/逐reader投递、丢包/重试、真实Clock参考tick的准确SHA。参考 `125d8cf4e30b5affe3cd731fbb5d4a73d0e40a10` 为草案；其旧整任务等待被分层v2覆盖，不能拿它的fixture冒充实际接线。 |
| D04 | 原生最终证明/CheckFINAL，真实五成员四不同signer证书与wrong context/body负例；通过04/08实际区域结果联调。组件trait测试不覆盖crypto。 |
| D07 | 07独立计算q_avail、公共stop scope/lifecycle投影、GFR接纳；06不替07认证结果。消费PE-LOG-1/PE-READER-1接口与样本后核对。 |
| D16 | 官方来源的submit/finality/query/stream和timestamp证据适配PE-READER-1；Sim来源认证不是官方共识认证，兼容需16独立验收。 |
| D06-integrated | 真正经过02调度的A/B reader、Publisher持久输入/输出、grant/Ack与crash/restart，同一集成SHA原06矩阵；不能用显式函数输入组件PASS代替。 |

本合同的纯日志、TimestampPolicy、reader与持久Publisher完成后可报告**06组件PASS**，完整功能仍OPEN/待接线。PUB-4时限活性、官方dev-node、真实15-worker、G/Inbox/D均不由本矩阵证明。跨职责语义若有分歧，记录具体局部gate并继续无依赖组件，不改03/02/04所有权或自动升级依赖。
