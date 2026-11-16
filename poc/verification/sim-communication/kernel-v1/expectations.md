# Kernel v1 独立预期与冻结前审阅

先读固定来源与原64向量，落盘 [source-first](source-first.md) 后才读取组件合同/API。现在的算法期望来自 [kernel合同](../../../docs/contracts/task-contract-sim-transport-kernel-v1.md) 与 [API](../../../docs/sim-transport-api-v1.md)，没有读任何Builder源码。精确字面值在 [golden-vectors.json](golden-vectors.json)，输入边界在 [vectors.json](vectors.json)。本阶段 **NOT RUN**；harness在Builder释放writer后补写，独立预期已先形成。

## 冻结前审阅结论与lead决议

API §4现稿采用唯一serde adjacent tag：unit=`{"kind":"fifo"}`，tuple=`{"kind":"replica","value":0}`，named variant也为`{"kind":"rebind","value":{...}}`。这与serde指定属性一致、可以实现；不能误用named字段平铺在kind旁的另一种编码。私有snapshot、MessageHeader与ReleaseIntent也必须遵守相同规则。

已向lead报告并由lead明确以下决议，待其同步合同冻结；本Verifier不改Docer文件：

1. Unordered的“jitter-tie”就是packet tie：`H(SIMT/tie/v1,seed-u64BE,packet_seq-u64BE)`，不另定hash。
2. immediate Fault或Send当时offline Drop只分配原copy0、发一条Dropped，`raw_due=None`，不生成duplicate。正常raw_due算出后的overflow、Drain offline和StaleBinding均为`Some(raw_due)`。
3. `RouteEvent.effect_index`是本次全部输出从0连续的ordinal，checked u32；不是command编号。
4. duplicate identical Enqueue和matching tombstone ACK均零release；合法confirm仍revision+1，且Enqueue遵守last_action_tick。
5. ClockSnapshot.sample_delivery_tick_us保留最后成功install时刻，跨rebind不清；sample=None时允许delivery=Some，新install不得早于该水位。sample=Some必须有delivery。view先检查sample校准。
6. restore只核结构、receipt/action/release引用与局部不变量。无prior state不能证明完整历史transition；历史真实性/trusted floor归01。
7. Route plan最多4096 commands、8192 events，candidate snapshot canonical bytes加完整events canonical数组bytes≤32MiB。超界ResourceLimit且原state不变；累积events时就检查预算，不能先构造无界中间结果。
8. 初始send_ordinal=0、next_release_us=0、last_action_tick_us=0、revision=0、next_packet_seq=1。
9. 保持event_id≥1。来源只读核对：I1a `sim-runtime/src/lib.rs:68,93`明确要求event_id>0，I1a及native `event_key`均由0先checked+1后分配；现有fixture首ID为1。本项只记录旧接口事实，没有读取Builder。

上述澄清纳入后，没有剩余阻止组件Builder启动的合同/API问题；真实持久化与native接线仍不在本组件证明范围。

## 独立hash与数值

hash只用Python标准库hashlib和手写按API排序的JSON对象推导，未调用本crate或既有实现helper。H对domain和每个part分别加u64BE长度；没有把整个拼接作为一个part。

| 对象 | 独立期望 |
| --- | --- |
| payload `abc` | `a486d65d31be102f081390036c0c8436009c0ede7ff4dcdf879d75b2e8af1bda` |
| golden Message（run=0x11×32/config=0x22×32/R0→R1/channel7/context=0x33×32/codec=0x44×32/origin=(7,2,1000000,1)） | `7423e6af569d272d5d2a80634d61b4a84e615a8d5e95ce2c7e32771702206b73` |
| normalized TransportConfig | `c71654ba27624d989e8ac44469b5b055c061980eecf693f67bfb30424359438c` |
| jitter max1000，seed7/42/99，attempt1 | `[4,996,436] us` |
| 相同消息/seed，attempt4 | `[301,48,856] us` |
| packet seq1/2/3相同eligible时，seed7 tie顺序 | `[3,2,1]` |
| 同上seed42/99 | `[2,3,1]` / `[2,3,1]` |

完整enqueue→Attempt与另一receiver Receive→sender Acknowledge四份PersistenceRecord、canonical JSON、transition ID、record hash、ReceiveAck保存在golden文件。新Enqueue确认只发Enqueued提示；首次network Send还需要单独Attempt确认。这比完整02的抽象“enqueue后可以发送”细化为两个显式durable边界，不改变来源规则。

drain按whole-frame payload pacing：rate=1,000,000 bytes/s，3-byte payload每次至少隔3us；零payload按1byte最少1us；late Drain不会补发积攒的免费credit。举例raw_due=1020，首次实际Drain=1100，第二条同link earliest=1103，而非1023。不同link各有自己的next_release。

## 主要测试结构

| 独立组 | 核心断言 |
| --- | --- |
| K01 | typed namespace；opaque run/config/context/codec及权限；原payload/逻辑ID；wrapper attempt/current incarnation不入ID；64KiB边界。 |
| K02 | normalize声明顺序无关；lexicographic selector优先级；歧义/缺项；固定jitter/tie；FIFO/Unordered；Hold/Drop半开端点；duplicate/copy预算；全局条数/字节/历史attempt限额；checked整数。 |
| K03 | plan完全只读；合法overflow与非法末尾全拒绝区别；stale/cross-instance/clone/restore；No-op也增revision；snapshot canonical与局部不变量；新command/event/encoded-size预算。plan不可Clone/不可由caller构造，外部harness只使用public API。 |
| K04 | 四份完整golden记录；每种wrong receipt保持零release和原revision；恢复原retry/first receipt；duplicate receive只ACK；tombstone不能重新enqueue；consumed/inbox/tombstone计入保留容量；持久后stale是恢复前提失败而非义务撤销。 |
| K05 | rate1 ±1000、as-of投影、lease等号、seq poisoning与有效seq乱序；64 outstanding边界；rebind保留counter/sequence/delivery水位且清校准；deadline clamp与溢出。只证明ClockTracker纯状态。 |
| K06 | Service和Reader通过同GrantConsumer fixture；A的opaque交付不改B；无reader_prefix/Advance/公共日志排序。trusted ClockView来自ClockTracker，外层真实grant认证仍是adapter责任。 |
| K07 | seeds7/42/99×配置声明顺序正反×墙钟0/2ms共12个组件run；同seed完整逻辑trace相等，active native engines=0；独立负变异只断言可检出的局部不变量。 |

可靠性边界：typed DurabilityReceipt是可信adapter声明，不是fsync或密码学证据。每owner独占plan→persist→confirm→root提交，禁止并发mutation。restore返回同`(transition_id,release_index)`意图供01 root去重；调用restore不会自动证明已发布/未发布。测试用明确mock receipt，不命名为真实root receipt。

初始/变异snapshot只能证明纯state结构，不能证明从genesis可达或外部trusted floor。测试不以重算record hash+自造receipt声称攻破存储认证；这类真实性归01。外部harness不得调用Builder状态迁移来计算expected。

后续harness计划：独立Cargo workspace `kernel-v1/Cargo.toml`，path dependency `../../../crates/sim-transport`，从public API检查黄金hash/完整canonical record与状态变化；测试文件仍只在本Verifier路径。当前未创建harness或lockfile、未build；接口冻结提交不必等待测试代码完成。
