# SIM-TRANSPORT-KERNEL v1

2026-11-15。状态：**FROZEN by lead；实现和运行验收尚未完成**。本合同按总协调 [dependency-plan-v2](../commander/dependency-plan-v2.md) 授权，选定独立生产运输内核的具体方案。lead已复核本版、API及[独立预期](../../verification/sim-communication/kernel-v1/expectations.md)，授权一个Builder开始独立组件，无需等待整个01；不能据组件通过宣称功能02已accepted。精确Rust接口以 [sim-transport-api-v1](../sim-transport-api-v1.md) 为同版附件，两者一起冻结。

来源硬规则继承 [完整02合同](task-contract-sim-communication-v1.md) S1–S7及F项。本合同选定其中Q3/Q4的纯算法；Q1真实root custody/恢复与03公共canonical仍是集成边界。正常FIFO、稳定消息身份、受控释放、RPC/时钟/墙钟分离、既有terminal/signature不复活等规则不变。

## 1. 交付与禁止范围

Builder只写新 `poc/crates/sim-transport/`，单包独立workspace，`Cargo.lock`也在该目录。建议文件为 `src/{lib,identity,route,reliable,clock,service,snapshot}.rs` 与 `tests/`；允许在该目录内拆分。package `sim-transport`，edition2024，rust-version1.95；依赖固定 `sha2=0.11.1`、`serde=1.0.229`（derive）、`serde_json=1.0.154`。无Commonware/Tokio/proto/网络/签名依赖，不复制已有runtime实现。既有native/I1a、根manifest/lock、types、proto、vendor、子模块均不改。

组件包含typed Endpoint、opaque Envelope、纯RouteKernel、独立ReliableMailbox、rate1 ClockTracker、service grant/delivery类型与trait。生产者和fixture消费同一API，不另建绕开它的测试实现。组件不包含RPC server、线程、磁盘fsync、真实root Ack、BFT、业务公共日志、reader前缀排序或链finality。

主基线是已正常合并调度授权的当前worktree `c647098`；原准备提交 `125d8cf4e30b5affe3cd731fbb5d4a73d0e40a10` 留作历史。Verifier独立kernel预期在读Builder前保存；Builder候选命名SHA后由Verifier复核、Tester在同SHA运行组件矩阵。native完整SC-01–18仍全部保留待集成。

## 2. 标识、编码与容量

`Endpoint = Replica(u32) | Service(u32) | Reader(u32)`，类型tag分别1/2/3；同数字不同类型不别名。Replica数值是03的public pid，kernel不从它求region/member/signer，也不实现registry/PoP。run/config/context/codec均为opaque32字节引用；kernel只校验相等/非零和显式允许集合。业务cid、Descriptor、ReplayPosition的canonical编码不在此crate。

运输hash `H(domain,parts)` 为SHA-256：domain及每个part分别先放u64BE长度再放原字节。整数parts按声明宽度BE；Endpoint编码为tag-u8+id-u32BE；Option<Id>为单字节0或1+32bytes。域固定在API附件；不改变I1/Fresh hash。payload可为空，最大65,536bytes。

logical message ID绑定run/config/src/dst/channel/context/codec、不可变origin和payload hash。origin为`{step_id:u64,effect_index:u32,sent_tick_us:u64,source_incarnation:u64}`；origin不含root receipt的业务结构。attempt和当前两端incarnation不纳入logical ID。所有计数/时间转换checked，不能wrap。旧message可用当前wrapper重试，但kernel验证wrapper与其已登记当前binding一致。

默认且本v1固定上限：64 endpoints、32 channels、1024 link rules、每link64 queued copies、全kernel1024 queued copies及4MiB queued payload、8192 tracked logical message IDs；每owner mailbox最多1024个outbox记录（active+tombstone合计）、1024个inbox记录，总保留payload4MiB。payload单条上限65,536bytes。配置可**降低**这些上限，不可超出v1 maxima。tracked last-attempt表不驱逐，满后新ID使整plan返回ResourceLimit；已知ID重试仍可在其余限额内处理，snapshot恢复核对同一上限。所有有界空间都包括duplicate副本/历史去重项；内存满明确返回或记录overflow，不确认后静默丢保管义务。确认后outbox去payload但保留不可变header tombstone；inbox有限run内不GC，包括已业务消费的项。

## 3. 固定配置与profile选择

配置用排序数组（API中Vec）表达注册endpoint、channel与rules，构造时按规范key排序，拒绝重复。EndpointRegistration固定incarnation≥1、可发送/接收channels及逐channel允许opaque contexts。Channel配置`context=Forbidden|Required`与`reliability=BestEffort|Reliable`；没有absent context wildcard。独立历史channel可允许旧context，live channel只列允许的live context；kernel不理解Close，也不能替adapter放行终止后的新签名。

LinkSelector分别为src/dst `Any|Kind|Exact` 和channel `Any|Exact`。匹配优先级为lexicographic `(Exact endpoint选择数, Kind endpoint选择数, exact-channel是否为1)`，较大者胜；最高优先级多条匹配即`AmbiguousProfile`，即使值相同也拒绝。所有注册且有权限的有向src≠dst/channel必须唯一匹配；无匹配`MissingProfile`。无region selector；03可把region规则展开为明确endpoints。展开表、endpoint/channel权限、seed、限额和profiles进入kernel_config_hash。

默认profile：latency20,000us，jitter0，FIFO，offline=[]，offline_mode=Hold，capacity64，drain1,000,000 payload-bytes/s，无loss/duplicate/extra delay；overflow唯一`DropNewest`。profile可覆盖latency/jitter/offline/drain/capacity/faults，但静态运行期间不更新。需新配置则创建新kernel，不能重解释已有snapshot。

## 4. 纯RouteKernel算法

`plan(step,commands)`只在私有state副本执行，`commit(plan)`成功一次才安装全部变化并返回events。plan私有且不Clone；含不可伪造的私有instance token、base revision、config hash和候选state。kernel Clone和snapshot restore均新建token；旧plan不能跨实例使用。token是进程内比较能力，**不进入snapshot/hash/trace**。任何command无效整plan失败，原state、counter、queues、bindings和events均不变。commit拒绝cross-instance/config/stale revision；成功revision checked+1，消费plan，无半次安装。

StepStamp为调用者受控输入 `{logical_step_id,tick_us,event_id,causal_wave}`：tick不倒退，logical_step_id严格增大但可跳号；event_id只是稳定输入身份，不要求按数值顺序处理（既有scheduler可按tick/wave/tie先处理后创建事件）。kernel不读取墙钟、不自行发grant。command顺序就是同root batch顺序。正常command为Send、Drain、Rebind。Rebind只允许当前incarnation恰好+1；外部恢复/管理员授权由adapter核验，此纯状态检查不是授权证明。

Send先核对run/config、logical ID/payload hash、src/dst登记、channel权限/context、current incarnations、origin不在未来、attempt≥1。每logical ID保存last admitted-attempt，要求新attempt严格大于它（允许因durable-before-release crash跳号）；同一attempt的root重放应由adapter幂等处理，不再次Send。网络duplicate由kernel内部copy产生，保留attempt。每link send ordinal从1递增，即使随后drop；每个新copy取得全局packet_seq。

每link可配置按send ordinal排序且唯一的FaultEntry：`drop:bool,duplicate_delay_us:Option<u64>,extra_delay_us:u64`。drop与duplicate同时设定拒绝配置；未列ordinal无fault。jitter为 `first_u64_BE(H("SIMT/jitter/v1",[seed-u64BE,src,dst,channel-u16BE,message_id,attempt-u32BE])) % (jitter_us+1)`；jitter_us不得为u64::MAX。raw_due=send_tick+latency+jitter+extra_delay，全程checked。duplicate若有则原copy之后入队，duplicate raw_due=原raw_due+delay；不是新logical message/attempt。

offline区间严格递增、不重叠且start<end，均`[start,end)`。Hold：离线也可入队，delivery候选tick向后跳到包含它的offline区间end，重复直到online。Drop：Send当时离线直接Drop；Drain当时离线则丢弃该次eligible copy，记录原因。正常driver在next_wakeup给的tick调用Drain；若adapter显式延迟Drain，则按实际受控Drain tick判断online，记录原raw_due与实际tick，不捏造过去交付。

FIFO每次只允许队首成为candidate，即使后续raw_due更早。Unordered profile按`(raw_due,packet tie,packet_seq)`选该link候选，其中packet tie就是下文`SIMT/tie/v1`，不另定义jitter tie；它是显式故障/实验模式，不计作正常FIFO通过。两个模式都只保证各自transport copies顺序，不提供跨丢失重试的end-to-end logical FIFO。跨link/channel自然交错。

drain是**whole-frame payload-byte pacing**，不是逐bit传输模型。某link候选eligible tick至少为`max(raw_due,next_release_us)`，再应用Hold。Drain只处理eligible≤step.tick的copy；成功交付后设 `next_release_us=step.tick+max(1,ceil(max(1,payload_len)*1_000_000 / bytes_per_second))`，用u128中间乘法/除法后checked转u64。rate必须>0；零payload按1byte、至少1us计费；drop/overflow不消费drain credit。没有初始burst credit。跳过很多tick后Drain仍不能一次免费释放同link backlog。跨link各有独立drain。

候选全序为`(eligible_tick,tie_rank,packet_seq)`，tie_rank=`H("SIMT/tie/v1",[seed-u64BE,packet_seq-u64BE])`；每轮释放/丢弃后重算候选。所有本步产生的事件带`step`和该步effect_index，按操作/候选顺序输出；同tick因果wave由外层调度器按既有规则衔接，kernel不伪造root event_id。`next_wakeup_us()`返回最早候选tick，无候选返回None。

copy入队前分别检查link/global条数与payload预算，超界只DropNewest该copy并记录Overflow，不驱逐旧copy、不删除可靠outbox。重复副本可因容量被丢而原copy保留；这是合法已定义overflow结果。与错误command导致整批拒绝严格区分。Drain交付时再核对目标current incarnation：过期物理wrapper丢弃为StaleBinding；可靠sender稍后使用新binding重试，不能改写已入队wrapper。

每plan最多4096 commands、8192 events，candidate snapshot canonical bytes加完整events canonical数组bytes最多32MiB；超过任一界返回ResourceLimit且原kernel不变，累积events时即检查预算。immediate Fault或Send当时offline Drop只分配原copy0并输出一条Dropped，raw_due=None，不生成duplicate；已计算raw_due之后的overflow、Drain offline及StaleBinding均为Some(raw_due)。RouteEvent.effect_index为该plan全部输出从0连续的ordinal，checked u32，不是command编号。初始send_ordinal、next_release_us、revision均为0，next_packet_seq为1。

## 5. ReliableMailbox与adapter持久确认

每个逻辑Endpoint一个mailbox，独立于route队列。MailboxConfig从同TransportConfig提取owner/权限；默认retry_interval20,000us（>0），无指数backoff/GC。只接Reliable channels；新native raw-vote保留责任不在此组件。

API是 `plan(action) -> CustodyPlan`、`confirm(plan,DurabilityReceipt) -> CommittedCustody`。plan不改变可观察state、不释放Send/ACK；receipt为可信adapter在实际持久事务完成后提供的typed声明，绑定owner、base revision、stable transition ID、candidate_record_hash和opaque storage_receipt。组件只检查字段与非零receipt，**不验证fsync、root receipt真实性或密码学认证**；fixture receipt只测试此边界，不能算真实durability证据。

transition ID=`H("SIMT/custody-transition/v1",[run,config,owner,base_revision-u64BE,canonical-action-bytes])`，不依赖当前incarnation wrapper之外的随机状态。persistence record含schema、owner/base_revision/transition ID、完整candidate snapshot及ordered release intents；record hash使用`SIMT/custody-record/v1`，不把自身hash或外部storage_receipt塞入record。ACK intent只存AckFor(message_id)模板；confirm与restore使用同一匹配的外部receipt填充storage_receipt及transition ID，不能落盘后改变收件消息/语义。这样adapter能恢复已持久但confirm/release未完成的义务，无自引用hash。

**串行前提：** 对同owner从plan→persist→confirm直到outputs纳入root Ack，controller独占串行。不可让另一个mutation改变revision。未持久的stale plan可丢弃；已persist后发现stale/cross-instance必须fail closed，并以stable transition ID交01恢复已持久record，不能当普通失败丢掉发送/保管义务。组件不替01实现这一durable协调。

| Action | candidate状态与confirm后允许输出 |
| --- | --- |
| Enqueue(message,now) | 新ID记录完整不可变message、last_attempt=0、retry_due=now。只返回Enqueued/RetryDue提示；还无network Send。相同ID+相同message无第二任务；相同ID改变内容拒绝。tombstone命中也是幂等已确认，禁止重插成新业务发送。 |
| Attempt(id,now,current_dst_binding) | 必须active、now≥原retry_due；last_attempt checked+1，retry_due=now+retry_interval。candidate持久确认后才返回Envelope。restore保留原retry_due；后续真正重试才从其实际now开新间隔。 |
| Receive(envelope,first_tick,first_local,authenticated_src_binding) | 目的owner/current incarnation、run/config/权限/context和src binding全部核对；新ID存不可变message与首次收包，pending消费标记。确认后返回InboxAvailable提示和ReceiveAck；已有相同message只重发ACK，首次收包不覆盖。变异同ID拒绝。 |
| MarkConsumed(id) | adapter须把业务消费状态与此candidate同事务保存。确认后标consumed；无新的业务输出。pending inbox是稳定任务，可查询/恢复；库不声称任意外部副作用exactly-once。 |
| Acknowledge(ack,authenticated_receiver_binding) | 核对原run/config/src/dst/channel/context/message/payload/origin、current receiver binding与非零custody receipt；将active outbox变成header tombstone并释放payload预算。确认后才删除active义务；重复matching ACK无新删除。 |
| Rebind(new incarnation) | 只owner old+1；无message/origin/first receipt/retry_due重写。重新握手真实性仍由01/adapter核验。 |

Inbox每个logical ID只插入一次。`InboxAvailable`是可恢复任务通知，不是原生proof/完整body/G/Inbox/D验收；重复数据或恢复重发ACK不产生第二个logical insert。消费者按stable ID幂等处理pending项，业务exactly-once需其状态与MarkConsumed原子持久。ACK仅在receiver确认持久保管后释放，sender删除也需自己的确认；没有ACK-of-ACK。历史context记录不因新的live context或ClockUnsafe自动删，真实终止/签名权限仍由01/04/05守卫。

重复相同Enqueue以及matching tombstone ACK均零release；合法confirm仍推进revision，Enqueue仍检查last_action_tick。Mailbox初始last_action_tick_us=0。ClockTracker跨rebind保留最后成功install的delivery时刻水位，具体snapshot约束见API；不能借清校准绕过时间倒退检查。

## 6. Snapshot、Clock和服务边界

配置、消息及snapshot采用固定struct字段顺序的compact serde_json；所有集合编码为按API key排序的Vec，bytes为JSON整数数组，无map/floats。枚举采用`kind`字符串tag与声明字段，无unknown fields；decode后重编码必须字节相等，否则`NonCanonical`。snapshot包含schema/config hash/revision和全部可观察state；内部token不编码。snapshot最大32MiB，decode前先检查大小，decode后核对排序、去重、容量、摘要、绑定和计数关系；不接受自洽但不同config。详细结构顺序由API附件冻结，不新增第二序列化模式。

Route snapshot恢复只恢复纯state并新建token。Mailbox恢复需要完整persistence record+匹配typed receipt，返回同transition ID的release intents，供外部root去重；调用本接口不证明真实durability或trusted floor，回滚保护与01源文件custody仍待接线。新确认后旧plan不能在restore/clone对象使用。

ClockTracker是单独的rate1纯状态，不藏在RouteKernel中：request编号checked递增；install验证domain/run/config/endpoint/incarnation/request/sequence/as_of≤delivery<lease_end、uncertainty覆盖abs(offset)且≤1000、local_at_as_of=as_of+offset；原子拒绝不污染序号/其他request。view(tick)只投影有效lease，不续租；deadline换算保留绝对值。rebind保留request/sequence/旧lease信息并清校准，需新sample。ClockTracker不创建cid、不设置Terminated、不签名；新sample只能恢复校准，不能授权恢复旧cid。terminal和stop实际签名gate仍在01/04/05。

ServiceGrant/GrantConsumer供06/09及fixture共同使用：显式StepStamp、EndpointBinding、ClockView和opaque Message/Timer/External输入，返回opaque send/absolute timer/work intents。06解释q/θ和reader prefix，09解释L1秒timestamp/tx/log/finalized，03拥有公共位置编码。02只交付各Reader endpoint真正收到的opaque bytes；一个reader的Delivery不会改变另一个reader。adapter必须在真实grant内调用trait并把返回effects放进完整root Ack；直接函数调用不是受控进程证据。

## 7. 组件gate与剩余集成gate

| Gate | 必测组件证据 |
| --- | --- |
| K-01 | 15 replica+service+reader地址、typed同数值不别名、opaque context/权限/64KiB边界、stable ID与attempt/current incarnation分离。 |
| K-02 | profile优先级/歧义/缺项、固定seed jitter literal、FIFO逆raw_due、跨link/channel与Unordered、half-open Hold/Drop、duplicate/overflow、空payload/rate边界/checked时间溢出。 |
| K-03 | clone/prevalidate/commit全批原子、late-invalid command、两个plan的stale、跨实例/clone/restore plan拒绝、revision/counter溢出；无私有token进canonical bytes。 |
| K-04 | enqueue/Attempt/Receive/ACK-delete各persist前后快照、错receipt不释放、restore保留obligation/原retry、sameID变异、ACK伪binding、tombstone再enqueue、全历史容量边界。 |
| K-05 | rate1 ±1000us、half-open lease、迟/乱/失效样本、seq poisoning、原deadline、grant view无续租、rebind清校准但不伪造cid恢复。 |
| K-06 | Service与Reader同trait fixture；reader A/B独立opaque交付；pure snapshots/消息hash重放稳定；无公共prefix或EVM业务规则。 |
| K-07 | seeds7/42/99 × 配置声明顺序forward/reverse（构造规范排序）× wall pause0/2ms，纯kernel同逻辑输入完整trace配对；恶意state/record/receipt/plan变异独立checker拒绝。不是worker启动顺序，active native engines=0。 |

Verifier本阶段可新增独立kernel静态/可执行案例，但不得重写既有64条完整集成期望为较弱完成条件。Builder在自己crate内提供小example/CLI，仅驱动同一生产API输出完整trace，不另实现预期模型。Tester记录每seed四组合的trace及实际active native engines=0。真实多进程native四channel、RPC request/sample/response、root fsync/recovery floors、原生签名gate、实际reader业务及01 R1–R18/Fresh/I1a回归全部是**剩余集成gate**，组件测试不能代替。

未来命令（**planned，本Docer阶段未执行**；cwd为实际worktree）：

```sh
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/target/sim-transport CARGO_HOME=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/cargo /mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path poc/crates/sim-transport/Cargo.toml --locked --offline
```

执行前由lead把同工具链bin加入本命令环境PATH，确保cargo能找到rustc；不改全局配置。所有build/重型验证/仿真使用同一flock且不嵌套，原始输出在02数据盘专属artifacts，命令/seed/SHA/dirty/限额记录齐全。当前冻结稿无新增真正阻塞项；只待lead接口冻结与独立预期落盘，不再等待whole-feature01。
