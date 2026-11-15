# 独立期望 v1

先从 [source-map](source-map.md) 的来源推导，再只读定位实现。本文件初稿及静态向量在读取 Docer 新合同之前落盘。`vectors-v1.json` 的 expected 是未来 checker 的独立依据，不能由候选输出反推。数值单位统一为整数微秒；所有 runtime 结果当前均为 **NOT RUN / INCONCLUSIVE**。

## E：端点、命名域与实际 native channel

`V2-ID/H02/RBFT` 要求 public pid、region/member、native signer index、公钥和 OS PID 分开。`pid={0,5,10}` 分别为 `(region,member)={(0,0),(1,0),(2,0)}`，不能因为 member 都是 0 而合并路由/签名身份。配置必须表示 15 个 replica 和附加 service/reader；真实启动 PID、active engine、脚本 peer 和每类 channel 的实际覆盖分列。功能 04 的完整 slot-aware Session Program 不由端点注册替代。

每个 live-envelope 的 run/config/source ownership/destination/current physical incarnation/cid/channel/长度/hash 都须验证；历史 final/proof/body/state 使用其原 cid/context 验证，不能仅因 receiver 已进入新 cid 丢弃历史结果，也不能送入错误 live engine。服务无 cid 的条件是明确 channel/context 策略，不能把 absent cid 当 wildcard。最终字段编码和 signer 映射由 03/总协调决议；本文件不自建 canonical 公共 types。

transport 接受 65,536-byte payload 边界不代表任意 payload 是有效 native 消息。65,537-byte payload 必须拒绝。`VOTE/CERTIFICATE/RESOLVER/BODY` 每类必须有真实 native 产生、受控 Ack 提交、路由、接收/消费证据；静态字节或 BODY-only repair 不替代 resolver 执行。

## A：root-step Ack 原子性

`I1-SCHED/RECOVERY` 的拒绝观测集为：Hub scheduler/event allocator、queues/attempt counters、outbox、receiver dedup/inbox、reader prefix、accepted-Ack index、trusted floors、worker logical progress 与协议成功 trace。先放有效 ClockRequest/Send，再放非法末尾 effect，整份 Ack 拒绝时这些集合与水位必须与提交前相同；允许单独记录拒绝/基础设施诊断，禁止虚假的 AckCommitted/Delivered 等成功事件。

这是 **Hub effect-adoption 原子性**。此前在授权 grant 内已经完成的 native/auxiliary I/O 可以留下可审计 durable evidence；不承诺回滚它们。worker 的 root journal/head/cache precommit bound 与 Hub adoption 两个边界必须各有证据，不能用一个“事务失败”字样合并。

完全相同的重复 Ack 为零新效果；篡改/未授权/错误 incarnation 的 Ack 失败关闭。RecoveredAck 先核对原 canonical Ack、原 grant、root receipt/cut 和新物理 binding，然后按原逻辑 `(owner,step,effect-index)` 去重。旧 Send 字节、message id、attempt 保留；旧 ClockRequest/Timer/Work 不能作为当前物理任务直接排程。未写出 root record 的 OUTPUT_INTENTS 恢复不是 cached Ack。1 MiB frame 必须检查完整 prospective recovery wrapper；最终 byte boundary 输入需待 01 的确定编码提供。

## L/O：有序链路、可靠保管与重试

`V2-NET/CW-P2P` 给出默认 link 有序及 accepted≠received。**适配提案条件**：默认 FIFO 作用于每有向 endpoint/channel 已入队的 transport attempts；不额外推出丢失/重试之后 end-to-end logical FIFO。两个 attempt 的 raw due 为 1,020,000 与 1,001,001 时，默认链路必须先交付第一个；精确第二个 due/tie 依冻结队列算法。另一 link 或 channel 可以先到；同链路任意重排必须明确选择 adversarial 模式。

可靠 outbox 的语义至少为 at-least-once 尝试 + durable receiver 去重保管。receive ACK 只能在接收方持久保管 message 与去重身份后发出；仅内存收到不足以让 sender 删除唯一 durable 副本。这是满足 `V2-NET/H02` 崩溃后恢复送达的适配义务，不是 native `Feedback::accepted` 自带保证。ACK 不证明签名有效、正文完整、应用接纳、G/Inbox/D 进展，也不承诺任意外部副作用 exactly-once。

三个必测 crash 点：sender durable enqueue 后未发送；发送后未收到 ACK；receiver 已持久接收且 ACK 丢失。恢复后逻辑 message id、原发送位置、payload、签名字节不变，合法新尝试的 attempt 增长，接收方 durable inbox 插入一次。ACK 删除边界本身也要持久；重放 ACK 不重复删除别的任务。拥塞/分区导致一次 attempt 丢弃时，可靠 outbox 仍有 custody，恢复后在已声明在线、有限延迟、公平调度、正 drain 速率与足够保留容量条件下检查送达。

对照：m1 的首 attempt 丢失，m2 先到，m1 重试后到。在 transport-attempt FIFO 提案下这合法，持久插入顺序可为 `[m2,m1]`，各一次；公开 reader 的 prefix 必须另设缺口 buffer/gate，不能依赖链路 FIFO。可靠 ACK 控制消息无需递归 ACK-of-ACK；ACK 丢失由数据重发触发再次 ACK。

raw vote/certificate/resolver/native-body 的修复沿既有 native profile；跨区 final/proof/body/state 是可靠 outbox 的独立渠道。不能承诺 raw vote 永久保留，也不能把 native resolver 清掉后宣称跨区 outbox 已替它修复。

## C/T：Clock、RPC、lease、原 timer

`V2-CLOCK/I1-CLOCK/I1-ANNEX/CW-CLOCK`：

```text
C_m(delivery) = sample.node_time_at_as_of + delivery - sample.as_of
              = delivery + epsilon
valid lease  <=> delivery < lease_end
timer first eligible reference tick = local_deadline - epsilon  (rate=1)
```

例如请求于 1,000,000，request transit=250，response transit=10,000：sample as-of=1,000,250，delivery=1,010,250，sample age=10,000。ε=+1000/-1000 时当前 local 分别为 1,011,250/1,009,250；两者差 2,000，不把 10ms 响应延迟当时钟误差。未来服务采样处理时间如非零也必须单列。

每次 Manager/BFT/governor/签名读钟取相同步内 ClockView；墙钟暂停不改变 τ。样本验证包含 run/endpoint/incarnation/domain/profile、outstanding request、sequence 水位、as-of≤delivery<lease_end、uncertainty≤1000、固定 profile 连续性。拒绝 sequence=100/uncertainty=1001 后，合法 sequence=3 仍能由 watermark=2 前进到3，其他 outstanding request 不被消耗。

lease_end=1,100,000 时，在 1,099,999 签名可能合法（其余 guard 成立），1,100,000/1,100,001 必须禁止新签名，即使 ClockUnsafe 通知尚未运行。普通 grant 不延长 lease；已经 entered 的 cid 在 unsafe 后 durable Terminated，新样本不能复活。恢复保留请求/序号水位和旧 expiry，current incarnation 必须重新校准。

deadline=1,025,000，ε=+1000/-1000 的 timer 最早参考 tick 为1,024,000/1,026,000；续租、重试、恢复不改该绝对 deadline。取消 generation 的旧投递不唤醒；恢复到已经 due 的 timer 经安全恢复/startup fence 后立即可用，不能 recovery-now+duration。terminal cid 的 timer 显式取消。WorkCompleted 在原 due 之后恢复使用原 operation 身份与认证 RecoveryWorkCompleted，并在实际签名出口再次检查 lease/stop/Terminated。

## S/R：服务读者、故障分类与重放

`V2-READERS`：reader A 物理先收 q2 时，其可用连续 prefix 仍为0；再收 q1 后为2。reader B 只收 q1 时仍为1。不同 reader 不共享消费 cursor 或可变协议状态；L1/Espresso finalized、public key、reader receipt 和应用处理位置分别记录。transport ACK 可确认 q2 已持久保管，但不能把它冒充已处理 prefix2。DA 已接纳而不可读仍是 WAIT，transport 不将其改为空输入。

服务使用相同 grant/Clock/event 边界并有独立 actor 状态和故障开关。replica、治理、Web 无 Advance/SetFault 权限；全知故障诊断不能流入协议观察。脚本 crash、未脚本 panic/断联、wall watchdog 记录分开；watchdog 只使实验 INFRASTRUCTURE_FAILURE，不推进 τ 或伪造 protocol crash。

`V2-EVIDENCE/I1-ANNEX/H02`：对 seed 7/42/99 分别比较启动顺序正/反与墙钟0/2ms的完整逻辑 trace。只排除明确 wall/PID/port 诊断；不能删真实逻辑事件强行相等。还要运行 I1a loopback、Fresh 与 01 受影响恢复矩阵；固定同一接受候选。Checker 对提前 lease renewal、半份 Ack、重复逻辑插入、改签名/原 timer、跨 reader 越前缀等恶意 trace 必须拒绝。真实运行覆盖、seed、命令和原始文件未取得时保持 INCONCLUSIVE。

## 需协调冻结的适配选择

1. 01：accepted integration SHA、root/Ack/OUTPUT_INTENTS custody API、current incarnation 重绑定及新 timer/work 授权；保持全部已接受恢复用例。
2. 03：public pid/key/member/signer map、config_ref/cid/epoch/context canonical；02 不写公共协议 types。
3. 02/总协调：endpoint 与 channel 数值/编码、可靠 ACK receipt/dedup 持久记录、匹配优先级、分区/drain/tie 算法、attempt 上限；未来命令/schema 也需冻结。
4. 下游服务：reader prefix 与 custody ACK 的区别、服务重启/幂等接口；完整服务语义由相应功能验收。
5. 功能 02 的真实 PID/active-engine 最小覆盖需合同明确报告；配置15端点或两 worker 原生回归本身均不能声称完整15节点slot-aware功能通过。

提案条件化的向量使用 `conditional_adapter_proposal` 标记。来源硬规则不因实现难度或现有 fixture 限制而降低。
