# H1 合同独立复核：文档候选

结论：**准备稿覆盖主要来源规则；三项需要澄清/补齐，未达到实现冻结条件。** 只读审阅新合同，未改 Docer 文件。没有 H1 代码候选、M0集成验收或实际运行结果。所有 runtime case 仍 NOT_RUN。

独立顺序证据：先从固定来源形成并落盘 source-map/expectations/75 recipes/planned matrix，independent-capture.json 已封存四文件 SHA256，随后才读取三份新 H1 文档。本报告的额外配方属于事后合同复核补充，单列 review-addenda.json，不覆盖原向量。

## 1. 发现

### H1-DOC-01：方案 A 的动态 cid 准入还必须覆盖 ReliableMailbox（P1，冻结前补齐）

- **位置：** `poc/docs/healthy-session-program/interfaces-h1-v1.md:136`、`:139`、`:148`、`:178`。当前只提出 RouteKernel 类准入plan/commit，虽要求保存可靠义务，但未指定每个 ReliableMailbox 如何看到同一已持久准入revision。
- **来源：** H1要求运行后真实多cid；SIMT合同§3的静态endpoint/channel/context校验、§5的可靠发送/收件与持久责任。新能力不能只让一个层认可权限。
- **静态定位：** 当前基线 `poc/crates/sim-transport/src/reliable.rs:269` 的构造器规范化并私持 `Arc<TransportConfig>`（:296）；`:307` plan使用该config；Enqueue `:494`、Receive `:617` 都重新调用 `config.check_message`。最终 `config.rs:453`–459按原send/receive contexts拒绝未知cid。RouteCommand当前只有Send/Drain/Rebind，`route.rs:27`。
- **影响：** 若只扩展路由层，未来合法cid的可靠final/body/state消息仍在邮箱入队或收件时报ContextMismatch，无法接通H2/H3；若临时绕过该检查，又会改变权限执行位置而没有方案B要求的完整审计。现有02遵守它原静态合同，本项不是既有02实现缺陷。
- **最小重放配方：** 创建含已知cid A的route和两端mailbox并保留A outbox/inbox；从真实来源得到B；只在route安装B；对B执行Enqueue→Receive。按现存邮箱校验应拒绝。修订后的方案A必须使相关route/mailbox共享或原子安装同一合法准入revision，并保留A记录、计数、原retry和在途ACK语义。
- **要求：** Q-TRANSPORT明确source/destination/reader/service哪些mailbox参与、receipt/snapshot/hash版本、同grant验证与安装边界、旧frame/ACK与未完成plan的解释。更新所有相关校验（含导入/恢复格式验证）或采用已声明方案B。实现及跨PID恢复仍需各自授权；本次不扩展恢复验收。
- **证据状态：** READ_ONLY_STATIC_SOURCE_TRACE；未来生产重放 NOT_RUN。补充配方 RV-TRANSPORT-MAILBOX。

### H1-DOC-02：生命周期“失败保留原状态”需限定在尚未持久阶段（P1，冻结前澄清）

- **位置：** `interfaces-h1-v1.md:47` 写batch失败保留原状态；同文件 `:41` 与任务合同 F-CLOSE/P-COMMIT要求terminal先持久，且承认native I/O不能回滚。
- **来源：** SP §3.6（:190–192）、SM §3.10（:230）要求不可撤销终止及先持久；V2 §5.1与SIMT §5区分可拒绝候选和真实持久事实。
- **影响：** 不区分失败阶段时，Builder可能在terminal已sync但完整root/Ack失败后恢复旧Running状态，或把已发生I/O和保管义务当成不存在；这会破坏Close之后零新签名。
- **最小重放配方：** 生命周期batch关闭A并切换B；A terminal先sync；root提交/适用receipt失败；旧A completion及新签名尝试随后到达。预期fail closed/场景中断，A不恢复、无新签/未提交输出释放；保留实际durable disposition。预校验阶段失败则允许零可见状态变化。
- **要求：** Q-LIFE/Q-WIRE明确验证拒绝、资源失败、已持久后基础设施失败的不同输出与处理；不要承诺回滚真实fsync/terminal。健康profile可中断，不要求本阶段实现跨PID恢复。
- **证据状态：** CONTRACT_CONSISTENCY_REVIEW；无故障注入执行。补充配方 RV-LIFECYCLE-DURABLE-FAILURE。

### H1-DOC-03：M1b 应区分提前 h11 physical 与晚合法 slot 两种条件（P2，范围澄清）

- **位置：** `task-contract-h1-v1.md:105`、`preparation.md:65` 把“不等SessionEnd才开始所有准备”写成单一验收要求。
- **来源：** SP §3.7以当前local slot构造；V2 §6.3允许迟到/失败产生physical gap；GFR §4.3只固定后继窗口与anchor，未要求每个height physical；H2 milestone要求支持合法提前候选能力。
- **影响：** 把提前生产能力要求扩成每条SessionEnd轨迹都必须提前h11，会把仍有预算的合法晚进入/padding轨迹误判失败，也可能推动预知未知anchor/cid。
- **重放/算术：** 条件参数genesis10s/slot1s，R0 stop21s/grace3s⇒SessionEnd24s；R1 h11及时界22.2s。因此要求h11 physical的额外场景确需提前已知合法候选。但若首次Build local24.4s，h14仍在11–20；h9 anchor之后virtual h10–13可由h14决定覆盖，所有新增height仍须满足公共截止与Expire。晚入场本身不证明有界活性违反。
- **要求：** 保留提前准备/缓存的能力用例，同时增加晚合法slot+padding用例；一般M1b硬要求是正确anchor、固定11–20、真实来源与G接纳，额外h11 physical目标单列前提。
- **证据状态：** STATIC_ARITHMETIC_PASS；生产运行 NOT_RUN。原独立向量 REGULAR-END（可选h11）与 REGULAR-END-LATE-SLOT 已覆盖。

## 2. 明确的未决工程决议

现稿已诚实列出Q，不需把每个待定命名当成缺陷。冻结前的实际依赖如下：

| 决议组 | 已具备内容 | 必须交付的实质结果 |
| --- | --- | --- |
| Q-ADMIT（首要安全/活性阻塞） | F-CALLBACK与接口:180识别一致性差异 | 精确native hook/签名出口状态机：全局确定性有效性与节点首收/stop准入分离；WAIT producer存活；本地永久拒绝的abstention/cancel及继续进展依据；propose返回后的verify/certify承诺；必要vendor路径另授权。当前不能选择直接false/drop来完成。 |
| Q-TRANSPORT | A/B两种边界，禁止预知cid/空kernel | 明确选A或B及单owner；若A，解决DOC-01；真实来源能力、完整版本/容量/原子提交和旧义务兼容。 |
| Q-LIFE/Q-PREP/Q-RUNTIME | 原handle、一次Enter、Ready证据、Pause保存work/timer、多policy缺口 | command/job代次/幂等、H2到H1有序batch、停车精确poll/wake/I/O行为、DOC-02失败语义、shared driver/context明确移交。 |
| Q-RESULT/Q-DOMAIN | 03 trait、07 PublicObject/VerifierBinding、proof/body分层 | 历史namespace/profile/version/epoch确定性构造与golden bytes，03 PoP映射，完整目标未到时只做subject证书观察的事件标签，缺basis WAIT。真实BLS向量后执行生成。 |
| Q-WIRE/Q-SHARED | 完整Grant/Ack、NotApplicable与多cidreceipt、旧构造器保留 | 字段号、canonical/hash/noncircular preimage、完整root记录版本和实际持久顺序；scheme/witness/journal准确owner/allowlist与同SHAFresh计划。 |
| Q-FRAGMENT/Q-INGRESS/Q-PARAM | 1MiB/64KiB冲突、配对首收、有限上界建议 | 有界分片重组和完整reader前缀；认证预算/等待结论；timer/section/work/queue资源界及前提失败分类；真实有界活性预算尚待后续校准。 |
| Q-M0（外部依赖） | 准确输入提交已列 | 总协调交付已验证完整M0 SHA、组件tree/patch/lock和共享路径授权。不能用浮动main或旧Fresh PASS替代。 |

Q-RESULT已有文字区分“仅subject的证书检查”和需完整block的CheckFINAL（接口:99）；冻结event schema时应保留这一差别，勿把所有proof observation字段统一标成完整CheckFINAL已执行。

## 3. 核对通过的准备项

来源与范围、03唯一canonical、三组接口方向、原实例A→B→A、normal持久边界、15 PID与五真实signer、NotApplicable witness、Accept等号/严格界、历史CheckFINAL、physical/native hash区分、完整证明/交付/公开/G分层、固定seed+启动/墙钟扰动、官方同字节读回的有限声明，均与独立期望一致。

此处“核对通过”仅指准备文档覆盖，不是对应运行时gate通过。没有review一个新的H1实现diff；只读定位的旧baseline代码用于准确界定改动需求。未知安全/活性假设已升级lead，未由verifier自行决定协议。

## 4. 重放范围与未覆盖

当前执行仅inline Python的JSON结构、来源/文档hash、链接/格式及条件算术；未调用生产API。没有BLS计算、OS worker、Simplex、root fsync fault注入、官方发布或确定性仿真。全部75基础向量及两条补充向量是PLANNED/NOT_RUN。

后续每个finding须在已冻结合同与命名实现SHA上执行真实最小复现，保留命令/seed/raw trace/配置/版本/进程及调用证据；当前静态文件行与配方不是已执行失败。完整I1、01恢复、04重启及M1/M2均未关闭。
