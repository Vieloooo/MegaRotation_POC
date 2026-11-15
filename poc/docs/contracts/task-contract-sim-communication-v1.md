# SIM-COMMUNICATION v1 — 全局虚拟时钟与仿真通信准备合同

日期：2026-11-15；状态更新于2026-11-16。阶段：**COMPONENT VERIFIED；native-integration-waiting-dependency=01**。[分层依赖 v2](../commander/dependency-plan-v2.md) 已解除整个02实现必须等待01完成的门禁。独立 `poc/crates/sim-transport/` 已按[kernel合同](task-contract-sim-transport-kernel-v1.md)和[API v1](../sim-transport-api-v1.md)完成有限组件验收，证据见[交付记录](../sim-transport-kernel-results.md)。native/root/RPC真实接线仍等待01准确提交与文件移交；公共身份/域对象消费03。本文保留完整功能硬规则和原验收，完整02尚未accepted。

原准备基线：`ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，其中代码基线为 `36eff1c76d5c4df5d0790133c0e14719c0c73ebd`；准备提交为 `125d8cf4e30b5affe3cd731fbb5d4a73d0e40a10`。本阶段在正常合并授权提交 `0a951c4` 的本worktree revision `c647098` 上冻结独立组件。只有后续native接线/完整功能验收必须包含获接受的01准确集成SHA。

## 1. 规则来源与冻结层级

| 标识 | 来源与适用内容 |
| --- | --- |
| S1 | [v2 §3–6、§11–13](../../../formalization_v10/poc_design_zh_v2.md)：3×5 身份、一个虚拟时间权威、RPC/ClockView、受控输出、通信、独立读者与验收。 |
| S2 | [v10 Session Program §3.1–3.12、§4.6–4.9](../../../formalization_v10/02_session_program_zh_v0_1.md)：首次收包、持久签名与终止、历史最终证明和新投票的不同入口。 |
| S3 | [I1 v1 §6–10](task-contract-i1-v1.md) 与 [I1 接口附件](i1-interface-annex-v1.md)：完整 Ack、半开 lease、原 timer、拒绝原子性、样本投影、Fresh/I1a 兼容。 |
| S4 | [NATIVE-RECOVERY v1 §3–10、§12](task-contract-native-recovery-v1.md)：GrantIntent、RecoveryCut/root receipt、原输入与期限、恢复栅栏、RecoveredAck、R1–R18。01 的已接受实现 SHA 尚未交付。 |
| S5 | 固定子模块 `regional_BFT_sim` 的 `DESIGN.md` §1–2.1、§3.1、§5，pin `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`：签名 context、证据/交付分离、显式 repair 和有限证据。 |
| S6 | Commonware `2026.10.0`，上游 release commit `92a2f68a0e131c8b69de41e84aec37da6280f212`：`runtime/src/lib.rs::Clock`；`p2p/src/lib.rs::{UnlimitedSender,LimitedSender,Sender,Receiver}`；`p2p/src/simulated/mod.rs` 的有序链路说明及 `test_message_ordering`。 |
| S7 | [02 交接](../handoffs/02-sim-communication-zh.md)、[03 交接](../handoffs/03-mock-plan-execution-zh.md)、[工作约定](../handoffs/workflow-zh.md)：阶段边界、所有权、下游和真实网络证据。 |

下文 **F** 表示由以上已接受来源约束的硬规则；**P** 表示本任务推荐的适配选择，尚待 lead/总协调决议；**Q** 表示实现前必须明确的跨任务问题。P 不反向修改 v10，也不因在草案中出现而获得写公共路径的授权。改变 F 或移除验收项须版本化修订；不得由实现自行弱化。

P0 固定三地区、每区五个等权身份，正常 quorum 为四个不同合法 signer，窗口为十个 height。运输层不改变 quorum、签名 namespace、原生 vote/certificate 编码、Plan/Descriptor 或公钥排序。Kudzu/Minimmit 的 benchmark 门槛、coded delivery 与 staking 均不属于本任务。

## 2. 范围与所有权

**F：** 本任务把可配置时钟、endpoint、链路与可靠结果运输接到真实 native 路径。保留 I1a、Fresh 及 01 恢复 gate。15 个 replica 地址可配置、注册、按 cid 交付是本任务验收；04 负责 15 个真实复制节点上的 slot-aware Session Program。报告必须分别列实际 OS PID、注册 endpoint、活动 native engine 数量及各 channel 的真实覆盖，不能用 15 个静态地址冒充 15 个已运行引擎。

首轮准备仅写文档/独立静态输入，已保存于 `125d8cf`。当前Docer独占本文件、preparation及新kernel合同/API文档；随后Verifier保存独立kernel预期。lead冻结后一个Builder只写新 `poc/crates/sim-transport/`，该目录使用独立workspace及自己的Cargo.lock；不修改已有native/I1a、根manifest/lock、types、proto或commander。一个writer写入时其他角色只读。

新独立crate的写入范围以kernel合同为准；以下既有native集成路径仍为 **P：后续路径提案，不是当前写入授权**。本节原提议在native目录新增运输模块已由独立crate方案替代；未来直接接入该生产组件，不复制第二套实现。

| 路径/责任 | 未来所有者与边界 |
| --- | --- |
| `poc/prototypes/i1b-runtime/src/{native_hub,native_config,native_network}.rs` | 01 验收交出后由 02 单一 Builder 拥有；注册、路由、配置、native transport 接线。 |
| 同目录 `driver.rs` | 仅 route/transport 接口由 02 在 01 交出后修改；恢复、I/O custody、终止与签名 gate 保持 01 合同。共享同文件必须串行移交。 |
| `poc/crates/sim-transport/` | 当前获授权的独立生产kernel；精确API、算法和组件gate见新合同，独立workspace/lock由02拥有。 |
| `poc/crates/sim-hub/src/`、`poc/crates/sim-runtime/src/` | 只复用已验收调度/队列语义；若要抽取共同逻辑，另列精确文件交总协调批准，不能顺带重构 I1a。 |
| `poc/proto/native.proto`、native `native_wire_support.rs` | 02 lead 集成新 transport profile；字段号、版本、hash 向量须实现前冻结。I1a `i1.proto` 与 Fresh 编码保留。 |
| `poc/crates/types/` | 03 独占公共域对象、registry/config、Descriptor/cid、ReplayPosition 的 canonical 编码；02 引用，不再造一份。 |
| workspace/manifest/lockfiles/模块声明 | 总协调指定单一集成者；新依赖在批准前不引入。 |
| `poc/verification/sim-communication/` | Verifier 独立预期、静态向量、未来 checker 与复核记录；Tester 仅在已验证候选跑矩阵。 |

## 3. Endpoint、身份与 envelope

### 3.1 身份分层

**F（S1、S2、S5）：** public pid、地区局部 member、Commonware 排序 signer index、OS PID、runtime incarnation 是不同坐标。`pid=5*region+member`，例如 `pid=0/5/10` 都是各自地区的 `member=0`，不能互相路由或计成同一 signer。服务与 reader 没有伪造的委员会成员身份。

**P：** 采用类型化的逻辑 endpoint，独立于运行它的进程：

```text
EndpointId = Replica(public_pid) | Service(service_id) | Reader(reader_id)
EndpointRegistration = {
  endpoint_id, runtime_id, incarnation, run_id, transport_config_hash,
  endpoint_kind, registry_ref?, public_pid?, region?, local_member?,
  allowed_channels, allowed_context_refs, clock_domain, profile_version
}
ContextBinding = { cid, descriptor_ref, epoch, registry_ref, namespace_ref }
```

`runtime_id` 可承载多个独立 actor；actor 的 inbox、故障、durable state、权限必须分开。Replica 的 Manager/BFT/Builder 共用一个 NodeClock。服务可用固定零偏移参考域，不能因此读取未来场景。注册表在运行开始前固定；重连只替换被授权的 incarnation，不改变 endpoint、committee 或 config。

注册先验证 run/config/类型/权限与 03 的身份映射，再绑定受控流。冲突 endpoint、冲突 public key/member 映射、未知 runtime、旧/跳跃 incarnation、无效 clock domain 或未支持版本失败时不得部分登记。incarnation 递增及恢复授权采用 01 的 checked 规则，禁止 wrap。

### 3.2 运输记录

**F：** 所有协议输出必须来自完整已提交 Ack。每条消息保存原始字节与 payload 摘要、稳定逻辑 ID、原发送位置及 attempt；网络副本/重发不能产生新签名。运输认证不等于原生证书验签。

**P：** 新 transport profile 使用外层 envelope，保留现有 native envelope/原生签名字节原样：

```text
TransportEnvelope = {
  version, run_id, transport_config_hash, src_endpoint, dst_endpoint,
  channel, context_ref?, logical_message_id,
  origin = { source_root_step_ref, effect_index, sent_tick_us, source_incarnation },
  attempt, current_sender_incarnation, destination_incarnation,
  payload_codec_ref, payload_hash, payload_bytes, provenance_ref?
}
```

原始 `origin` 不随重试、转发或恢复改写；当前 attempt/transport incarnation 不是原生签名域。只有被授权的当前 transport wrapper 才能承载旧逻辑消息/字节/原发送位置，旧 incarnation 的 wire frame 本身仍拒绝。逻辑 ID 应绑定 run/config、src/dst/channel、context、原发送位置和 payload 摘要；不能仅按 payload 合并不同业务发送，也不能把 attempt 或当前 transport incarnation 纳入逻辑 ID 使重启绕过去重。不可变 origin（含最初 source_incarnation）的精确字段编码、最终 hash domain、宽度、字段顺序与字节向量交 Q2 冻结。多播按目的 endpoint 分成可独立确认的运输任务，保留共同源 effect 引用。

**F：** 解码前有 frame/payload 上限，解码后核对长度、摘要、channel、来源权限、目的 endpoint、run/config/context 和当前物理绑定。整数使用 checked arithmetic；未知枚举/版本、无效 optional 组合和计数耗尽明确拒绝。不可因 prefix/HashMap 遍历顺序选择 endpoint。

**P：** 保留 Fresh 的 1 MiB gRPC frame、64 KiB payload、每 directed-link/channel 64 个在途 envelope 作为新 profile 的初始值，单独配置 durable outbox/inbox 容量与总字节上限；后两者在实现附件冻结。payload 为 64 KiB 与 64 KiB+1、队列第 64/65 条均有独立向量。大状态只通过未来明确 chunk protocol 传输，本任务不偷偷放宽 frame 上限。

### 3.3 活 context 与历史结果

**F（S2 §3.6、§3.10、S1 §5.3）：** 新 vote/engine 输入只能去匹配 cid/epoch/namespace 的接收实例；外层路由不能修补或重标记内层错误 context。Close/ClockUnsafe 后旧 cid 永久停止新签名。历史 Final/proof/body/state 与已有发布任务保留，可继续向有权消费者转发；历史证明验证无需观察者同步时钟，也不能用当前 cid 的新投票入口验证旧结果。

**P：** `LIVE_ENGINE` 与 `HISTORICAL_RESULT` 是显式能力/目的入口。后者要求已提交来源和原始 context，经过 transport/service 的受控 root-step，不能借此重新唤醒 terminated engine 或调用签名。跨区历史转发仍核对历史 descriptor/registry；缺历史材料返回 WAIT，不改成目的端当前 cid。未知 context 的 live 包原子拒绝，已关闭 context 的历史包依保存的来源继续运输。

## 4. Channel 与可靠性责任

| channel 类别（P 名称） | 冻结行为与适配提案 |
| --- | --- |
| `native.vote`、`native.certificate`、`native.resolver` | **F：** 保持 Commonware 原生修复、限流、反馈和认证语义；发送 accepted 不代表交付。raw vote 不承诺无限期 durable outbox。真实 resolver repair 要执行，不能用脚本重发代替。 |
| `native.body` | **F：** 保留真实 body request/reply、内容校验与 Archive 的独立重试/持久化；证明先到不能当作完整 body 交付。不要把 receiver transport ACK 当 Verify/Certify 成功。 |
| `result.final/proof/body/state` | **F：** 跨区结果采用 durable outbox + receive ACK + 重试；已关闭 cid 的任务不删除。**P：** 与 native body channel 分开，避免可靠性责任混淆。 |
| `publisher.publication`、`log.espresso`、`log.l1`、`da.request/response`、`governance.transaction` | **F：** 服务输出和各 reader 实际可见输入走同一调度/提交边界。**P：** durable transfer 可复用可靠通道；业务幂等键、公共位置、事务接纳结果由对应模块定义。transport ACK 不代表发布/finalized/Inbox 接纳。 |
| `clock.request/response` | **F：** 请求、采样、响应独立受控事件；允许延迟、丢失、乱序，不能隐式可靠重发以掩盖时钟故障。新的 refresh 是新受控 request。 |
| `transport.receive_ack` | **P：** 反向显式通道，支持延迟/丢失；不再对 ACK 递归生成 ACK。重复数据触发重发已持久化的 ACK 记录。 |
| admin / diagnostic | **F：** admin 独立授权，操作入 trace；诊断可读全局场景但不喂给协议。浏览器不获取 Advance/Step 权限。 |

具体 enum 数值和 channel 到能力/codec/reliability 的一一映射属 Q2/Q4，禁止用任意未知字符串绕过校验。单独服务业务（Espresso、Anvil、DA、GFR）由后继任务实现；02 至少提供可执行的独立 endpoint fixture 和接口消费例子。

## 5. LinkProfile、FIFO、故障与 drain

**F（S1 §5、S3、S6）：** SimHub 是唯一调度器；默认 directed link/channel 保序，跨 link 或 channel 可交错。同 seed、逻辑输入与故障序列产生同一虚拟 trace；OS 到达顺序/启动顺序/墙钟暂停不分配逻辑事件序号。随机 jitter 本身不能声称覆盖同链路任意重排。

**P：** 配置包含 `src_selector,dst_selector,channel,latency_us,jitter_us,online_intervals,queue_bound,overflow,loss,duplicate,reorder,retry,drain`。先展开 endpoint/channel 匹配成唯一完整表；默认表补齐未覆盖项，同 specificity 的重叠匹配直接报配置错误，不按配置文件偶然顺序覆盖。显式 directed endpoint+channel 规则优先于地区/角色规则，地区/角色优先于默认；最终表参与 transport config hash。

正常 link 使用 FIFO 队列；样本 latency/jitter 必须非负，算出 eligible tick 后还受队首、online 和 drain 条件约束。新入队 attempt 不能越过同队列旧 attempt；已丢弃 attempt 之后的重试是稍后入队的运输尝试，来源并未额外保证跨逻辑消息的端到端有序交付。同 tick 事件用 `(tick, causal_wave, seeded_tie_rank, event_id)` 全序；同 tick 衍生事件进入后续 causal wave。tie/hash 算法沿用获接受 native profile 或版本化冻结，不能更改 Fresh 旧 trace 身份。

**P：** online 区间为 `[start,end)`，空/重叠/倒置或溢出配置拒绝。场景明确是 send-attempt drop、in-flight drop 还是 `hold`；不得含糊地将分区等同于缓存。`drop-newest` 只丢 bounded transport queue 的新 attempt，输出可审计 Drop 原因；可靠 outbox 中的逻辑义务仍在。outbox 容量不足须在 durable enqueue 前明确 backpressure/rejection，不能确认后静默驱逐。

恢复 drain 建议按每 link `bytes_per_second`、初始 credit、frame-accounting 固定服务规则；使用整数虚拟微秒和 checked ceiling division。heal 时不能凭墙钟瞬时清空 backlog，不能赠送未配置 burst。最小向量可用固定 `drain_interval_us` 的消息限速 profile；字节预算与消息预算不可混称。最终算法和默认量见 Q3。

同 link 的故意重排是显式 `reorder` 故障 profile，记录被打破的 FIFO 前提及具体选择，不混入正常 FIFO 结果。重复保持逻辑 ID/原发送位置，增加可辨认的副本/attempt 坐标，接收去重保留首次接收时间。丢包、排队、实际 delivery、receiver consume 分别记录。

## 6. 可靠 outbox、receive ACK 与单次逻辑交付

**F：** 网络允许 at-least-once attempts。跨 crash 的单次逻辑交付依赖 durable receiver dedup；不是声称网络实现 exactly-once。送达活性只在两端最终恢复、持久存储完好、正 drain、有限 backlog/公平重试、往返通道最终可用时断言；无限分区或 ACK 永远丢失只得 pending。

**P：** plain records 包含 `OutboxEntry(logical_id,exact_envelope,origin,logical_dst,next_attempt,retry_deadline,state)`、`InboxReceipt(logical_id,src,dst,context,payload_hash,first_receive_tick,first_local_us?,receiver_root_receipt)`、`AckReceipt(logical_id,payload_hash,receiver_root_receipt)`。dedup key 绑定 run/config/src/dst/channel/context，不能只按 payload/hash 或当前 incarnation。保留期/GC 是 Q4；准备基线为有限 run 内不 prune。

| 触发与前态 | guard | 下一状态/可见输出 |
| --- | --- | --- |
| 创建可靠发送任务 | 来源 effect 合法；容量和 payload/context 校验全部通过 | 先把完整 OutboxEntry 纳入 sender root durable commit；随后同一完整 Ack 才可释放首次 attempt。 |
| durable enqueue 后、首发前崩溃 | 01 的 source/root floor 验证通过 | 重启仍有同一逻辑任务；origin/payload 不变，无新签名；按原 retry 义务调度。 |
| 收到首份数据 | 完整 envelope、目的能力和 context 合法；inbox 可持久化 | 受控 receiver grant 记录 first receipt、dedup 与交付义务；receiver root durable commit 完成后，完整 Ack 才能释放 receive ACK。socket 读到 bytes 不足以 ACK。 |
| 收到完全相同重试 | 已有匹配 durable InboxReceipt | 不再次生成逻辑交付或覆盖首次收包；可在新受控步重发原 receive ACK。 |
| 相同 logical ID 但 payload/context/origin 不同 | 不匹配已有记录 | 原子拒绝，无 dedup 覆写、无成功 ACK、无新消费。 |
| receiver 已持久接收但 ACK 丢失/receiver 重启 | receipt/dedup 及业务交付义务仍可恢复 | sender 重发；receiver 重发 ACK；全 run 单次逻辑交付，业务完成与 transport 收件另计。 |
| sender 发送后、收到 ACK 前崩溃 | outbox 未获有效确认 | 恢复原条目、单调 attempt 和原 retry deadline，不凭发送成功删除。 |
| sender 收到 receive ACK | ACK 绑定对应 logical ID、src/dst、payload 和可信 receiver commit；匹配已发送条目 | 将确认/删除纳入 sender 受控 root durable commit；完成前崩溃仍允许安全重发。 |
| 伪造、跨 run/config/dst、未知 ID 或旧物理绑定 ACK | 不满足确认关系 | 拒绝原子性；不能删除其他条目或推进确认 frontier。授权恢复 ACK 走 01 明确例外。 |
| cid Close/lease 到期 | 终止 marker 持久存在 | 保留历史 outbox；仅已提交历史数据运输可继续，新 engine 签名不可能。 |

`receiver_root_receipt` 不在它所覆盖的记录内自引用；采用已提交 payload/receipt 引用的两阶段编码方式，最终安排由 01 的 root format 决定。业务副作用要么与 inbox 接收/consume marker 同步持久，要么以 durable pending task 幂等完成；不得先向业务交付后才落 dedup。transport ACK 只说明可恢复收件，不表示原生 proof 验证、完整 ancestry/body、G 接纳、Inbox 接纳或 D 派生。

重试必须由 SimHub timer/grant 触发；原 deadline 保留。**P：** 基线只承诺同 directed link/channel 的已入队 transport attempts FIFO，以及可靠层 at-least-once + durable dedup，不增加跨逻辑消息的端到端 FIFO。丢失的第一条消息重试时，第二条可能已先被持久接收。reader 的公共日志有独立连续 prefix gate/缓冲，不能借网络FIFO推导连续前缀。若消费方确需更强逻辑顺序，须在 Q4 明确新stream序号、backpressure和恢复规则并单列向量。

## 7. Clock/RPC/lease 与 timer

**F（S1 §4、S3、S4）：** 协议仅看本节点 `C_m`，SimHub/审计看 `τ`，公开日志用自己的 `θ/position`，墙钟只诊断/watchdog。所有协议时间是整数微秒；跨 API 精度变换须精确或拒绝，不能无声取整。rate=1 基线为 `C_m(τ)=τ+ε_m`，每节点 `|ε_m|≤1000us`，两节点差可达 2000us；Enter 时差不受这个界限制。

请求提交、request transit、服务采样、response transit/交付分开记录。样本是带 `as_of` 的历史值，交付时计算 `node_time_at_as_of + delivery_tick - as_of_tick`。Commonware `Clock::current()` 同步且不阻塞；一个 grant 内 ClockView 固定，Tokio/CPU 墙钟变慢不让它变化。

样本只在 response-delivery grant 中完整检查后安装：run/config 对应域、endpoint/incarnation、outstanding request、严格递增 accepted sequence、`as_of≤delivery<lease_end`、rate/profile、连续性与 uncertainty 均有效。拒绝不消费别的 request、不推进 sequence、不续租；invalid sequence 100 后 valid 3 仍可接受。正常 grant、timer 或 transport ACK 不能续租。

每次真实 `Clock::current`、Manager 读钟、governor 读钟、Enter 与实际签名出口都检查 lease/terminal/stop。lease equality 和 local stop equality 都禁止新签名，即使 ClockUnsafe/Close 通知尚未先执行。旧 lease 在 downtime 中到期，恢复按 01 保守策略永久终止旧 cid；新样本不能复活它。偏移/RPC/误差监视不得用 `max(last,new_sample)` 掩盖后跳。非零 rate/slew 不属本基线，需另定 `u+ρa` 预算。

**F：** timer 保存原绝对 local deadline、逻辑 owner/cid、generation、取消/触发状态。续租和恢复不重新从“现在”计 timeout。已经 due 的 timer 在安全恢复栅栏后以原 deadline 变为 eligible；terminal cid 取消可见，不恢复 engine。01 可换物理 timer ID/incarnation，但原逻辑义务和期限不变；旧物理 timer/work delivery 拒绝。

原 leader/certification/nullify retry deadline、latch 和 application work due 都来自 01 recovery snapshot；02 不重算 native view 时间。deadline 与 sample delivery、Close、work completion 同 tick 的合法排列都覆盖，不能靠固定“先刷新”排序绕过 equality guard。

## 8. Root-step Ack 原子性与恢复接口

**F：** 一次最多一个全局 outstanding grant；worker 运行到受控静止点，全部 timer/work/network/sign/proof/durable effects 属于一个完整 Ack。先按 01 保存 GrantIntent、native/witness/auxiliary frontiers、RecoveryCut 与 root Step/Ack，再暴露输出。后台 I/O 线程只做已有 custody 内工作，不能独立驱动协议/新签名/发送。

**P：** hub 沿既有 `AckPlan` 方式，在副本中验证并计划全部变更，再统一安装。任何 effect 无效时，Hub提交状态中的worker receipt frontier、trusted recovery floor、event counter、queues、outbox/inbox/dedup提交索引、attempt、timer、samples 和 accepted-Ack 表保持前态；只允许诊断拒绝记录。此原子性观测集是Hub提交状态，不承诺回滚本grant期间此前已授权的native持久I/O；worker已经保存的证据按01失败/恢复规则保留和处理。不得先入队第一条 send，再因第二条错误返回失败。合法且已冻结的 overflow/drop 是该 batch 的明确结果，与 malformed effect 的整体拒绝分开。

exact duplicate Ack 幂等且不重新投递；altered duplicate、wrong event/worker/incarnation、unsolicited Ack、伪恢复授权 fail closed。`RecoveredAck` 只携当前 transport binding 与原 canonical Ack bytes/grant hash/root receipt/cut hash；只接受 hub 已知 outstanding/committed step。不能重签、重写原 Ack 或借新 incarnation 重执行逻辑步。Hub crash recovery 本阶段不默认承诺，是否纳入可靠状态故障域见 Q1。

需要 01 提供：已接受 SHA、root/aux/witness receipt API 与 schema、恢复授权校验、原 timer/work/RNG/quota checkpoint、source custody、terminal replay 入口、R1–R18 准确命令和受影响子集。02 只能扩展 root effect 内容，不能降低 01 的签名前/同步前/发布前持久化顺序或 startup fence。

## 9. 服务、reader 与公开前缀接口

**F（S1 §2/§7/§10/§11）：** Publisher/Espresso、L1、DA、治理与独立 reader 使用同一受控交付接口，各 actor 独立状态/故障/inbox。实际 Anvil、Espresso 与业务验证由后继模块负责，02 不制造已经 finalized 的事实。公共来源产生、finalize、网络到达、reader 验证/消费的位置分别记录。

**P（更新）：** 02提供受控grant/ClockView/opaque Delivery与可靠保管组件的精确类型，见API v1；不提供公共 `Poll/DeliverReaderPrefix` 业务实现。06拥有公共日志/reader prefix顺序，公共ReplayPosition等canonical对象由03收口。09拥有L1 position/finalized映射。02只按独立Reader endpoint交付opaque bytes，不能从最大收到height猜前缀。

不同 reader 只获得各自实际交付且连续可验证的前缀。A 已收到 position 5 不授予 B 任何输入；B 先收到 5 而缺 4 时保留 gap/pending，不能对协议暴露包含缺口的 finalized prefix。读者自己的 receipt/cursor 跨 restart 保留，重复不跳位置。相同公共 position 的消费可以比较；不同 position 不能直接判链分歧。L1 block/tx/log 与 Espresso q/θ 不互换为本地 tick。

admin 故障动作包含作用 endpoint、虚拟位置与命名 checkpoint；protocol crash/restart、网络 partition、服务暂停、未脚本 panic/disconnect 和 wall watchdog 分别记录。未脚本失败/30s watchdog 是 `INFRASTRUCTURE_FAILURE`，不得当作正常协议 crash，也不推进虚拟时间。监督器只管理本 run 的实际 child PID。Observer 展示背压不改变协议进度。

## 10. 验收矩阵与证据

以下是**未来验收范围，全部 PLANNED/NOT RUN**。Verifier 先独立按来源形成预期和静态输入，再对获接受实现基线/候选执行；不从实现计算期望。种子为 `7,42,99`；具体静态向量固定 fixture ID。进程启动顺序正/逆，墙钟 pause `0/2ms` 配对，逻辑 run/config/seed/input 不变。

| Gate | 前态/触发 | 必须观察 |
| --- | --- | --- |
| SC-01 | 注册 15 replica + Publisher/Espresso/L1/DA + A/B readers；pid 0/5/10 同 member0 | 唯一地址与合法权限、各 cid 精确路由；列实际 PID/engine/channel 覆盖；不混 signer index。 |
| SC-02 | native vote/certificate/resolver/body 真实发送与修复 | 全部从受控 Ack 路由；保存原生签名字节、receipt/provenance；proof 先于 body 时无提前完整交付。 |
| SC-03 | wrong run/config/cid/epoch/namespace/destination/source、旧 incarnation、payload 超界/摘要错 | 原子拒绝；跨 context 不重标记；边界大小合法，越界明确失败。 |
| SC-04 | unsolicited/altered/duplicate Ack，batch 先合法后非法 effect | 无半份提交、计数污染或消息投递；exact duplicate 只一次，授权 RecoveredAck 原样重放。 |
| SC-05 | 单 directed link/channel 两消息 jitter 使后发更早 eligible | FIFO 保持；跨 link/channel 可反序；显式 reorder 故障单列，seed/tie 可重放。 |
| SC-06 | capacity 边界/第65条、区间端点、partition/heal/backlog | admission/drop/delivery 分开；可见 drop-newest；恢复 drain 遵守预算，持久 outbox 不被丢弃。 |
| SC-07 | reliable enqueue durable 后首发前 crash；发送后 ACK 前 crash | 同逻辑 ID/字节/origin，attempt 单调，原 retry；恢复后有条件最终送达。 |
| SC-08 | receiver durable 接收后 ACK 丢失与 receiver restart；sender收到ACK删除前后crash | durable dedup，单次逻辑交付，ACK 不早于 receiver commit，sender只在自身 durable确认后删除。 |
| SC-09 | 错 ACK、篡改 duplicate、closed-cid 历史结果 | 不误删；旧结果继续受控转发；旧 engine 不复活且新签名数不增加。 |
| SC-10 / C1 | offsets −1000/+1000us；RPC 单向 50/250us、1000/10000us | 每次真实读钟误差界与 Manager/BFT 同钟；响应年龄独立，投影正确，2ms 两两差合法。 |
| SC-11 / C2 | 乱序/丢回复/旧运行/未来as-of/expired；invalid seq100 后 valid3 | rejection 原子性；不使用旧样本冒充当前；grant 不刷新 lease。 |
| SC-12 / C2 | lease/stop 前1us、等于、后1us；原lease跨crash过期 | equality/later 无新签名，终止持久，新样本不复活旧cid。 |
| SC-13 | 续租、取消/rearm、restart、already-due、旧物理 timer/work | 原 deadline、首次收包/首次 Enter 保留；世代去重与栅栏正确；不得 recovery-now+duration。 |
| SC-14 | A/B 不同交付顺序、gap、reader restart；公开位置同键 | reader 只消费实际连续前缀，互不泄漏；公开时间键与协议钟分开。 |
| SC-15 | 同逻辑输入/seed、启动顺序与0/2ms墙钟暂停变化 | 完整 canonical virtual trace 一致，仅 PID/port/wall 诊断排除；输入/队列/ACK/签名不能从比较删掉。 |
| SC-16 | explicit fault 与 watchdog/unscripted disconnect 对照 | 明确 fault/infra 分类；无墙钟转协议时间；只有本run PID被管理。 |
| SC-17 | I1a loopback、Fresh 与01全部受影响恢复案例 | 在同一候选 SHA 回归；无未说明语义或 encoding 改变；R1–R18 覆盖清单逐项保留。 |
| SC-18 | 人为污染 trace：提前ACK、假续租、丢失outbox、第二次逻辑交付、越权历史重签 | 独立 checker 拒绝；checker 不调用被测状态迁移函数。 |

每次运行保留 candidate SHA/dirty状态、来源/合同/配置/patch/二进制 hash、命令、种子、完整 resolved endpoints/links/clock/fault schedule、原始 wire frames、native bytes/签名、root/aux/witness/outbox/inbox证据、process history、virtual trace、wall diagnostics 和独立 checker 结果。每个 channel 记录实际 terminal 覆盖和未覆盖项。`ProofObserved`、可验证完整区域交付、G/Inbox/D 事件分列；本任务不把 routing PASS 升格为后续业务 PASS。

原始输出逻辑位置（**planned**）：`poc/artifacts/verification/sim-communication/<candidate-sha>/<case>/` 和 `poc/artifacts/simulation/sim-communication/<candidate-sha>/<case>/`。大文件实际目标为02独占数据盘目录 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/`，manifest记录逻辑路径与实际绝对路径。构建前按总协调安排安全迁移本worktree现有 `.build` 并链接至该02目录，target也放其中；本准备阶段不迁移。全部 Rust build/重型测试/仿真使用 `flock /tmp/megaroation-poc-heavy.lock <command...>`，`CARGO_BUILD_JOBS=2`；不嵌套锁。准确运行入口在实现附件冻结，当前不存在的新 CLI 必须保持 `planned`。

## 11. 待总协调/01/03 决议

| ID | 必须确认的接口/问题 | 推荐与依赖 |
| --- | --- | --- |
| Q1 | 01 接受 SHA、route/driver 文件移交、root receipt/effect扩展、RecoveredAck/外部floor、outbox/inbox持久化故障域、terminal历史运输权限 | 继承01 root custody；hub只调度，durable义务属source/receiver；hub crash recovery不默认纳入。由01/总协调确认。 |
| Q2 | 03 registry/config_ref、pid/member/signer、Descriptor/cid/context、ReplayPosition canonical格式；新运输schema/hash/channel enum | 03独占域对象；02仅外层transport编码；明确 public config_ref 与 transport_config_hash 的组合关系，避免循环hash。 |
| Q3 | 纯运输算法 | 由kernel合同/API v1选定具体匹配、jitter/tie、FIFO/故障、容量与payload-byte drain；lead冻结后无需等待01。Fresh旧profile不变。 |
| Q4 | 可靠组件与真实持久接线分层 | kernel冻结outbox/inbox/ACK状态、可信adapter确认、容量/保留/重试；真实root custody/ACK认证仍Q1。无额外端到端逻辑FIFO，reader业务prefix归06。 |
| Q5 | 15 endpoint与真实native活动覆盖如何分场景、service/reader fixture入口、04/06/09消费接口 | 至少真实native四channel闭环及15地址/多cid对照；逐项报告真实PID/engine，04才作15节点slot-aware结论。 |
| Q6 | 精确新文件、模块声明/manifest所有者、01回归命令与预算 | 总协调统一公共路径；在同一集成SHA冻结再实现。 |

原准备SHA及完整SC-01–18保留。当前先冻结独立kernel合同/API和独立输入，按Builder→Verifier→Tester完成组件里程碑；组件PASS不计功能accepted。native接线保持 `native-integration-waiting-dependency=01`，未来在真实集成SHA运行完整原矩阵。03/06/09未交付的业务canonical对象保持opaque，不阻塞纯运输算法。
