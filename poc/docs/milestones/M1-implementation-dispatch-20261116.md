# M1 实施授权与接口决议

2026-11-16。用户明确要求开始派活、快速实现M1并复用现有infra。M0已通过，准确基线 **35d382ac445cc8bc779f0d18fd23cb57e54680da**，真实main同值。此通知撤销H1/H2/H3旧消息中的“仅准备、不可启动Builder”限制；各lead在下列明确边界内修订并冻结本阶段合同、固定独立预期后，立即进入单Builder实施和复核，不必再请求用户确认。没有冻结的局部接口只阻塞相关实现，不阻塞整项工作。

## 当前目标与复用边界

15健康真实worker、3×5委员会/q4、10-height窗口；真实区域Final及正文经Publisher→SimEspresso→两个独立GFR，驱动Manager实现R0→R1→R2，R2首个正确结果进入G。另跑SessionEnd且不移窗，官方本地节点读回同一真实结果字节。M2 Force暂不实施；01恢复、批次/DA、D/RESUME、Web继续保留暂缓。

沿用03唯一对象/PoP/执行、02 kernel/ClockTracker/custody、现有ControlledDriver/Clock/storage/witness、06 Publisher/reader、07 codecs/规则和16官方runner。不得复制Commonware、建立另一套公共编码、引入always-true production verifier，或用直接网络输出绕过完整grant/Ack。当前旧原型只有2真实worker；扩成15必须有真实PID和五引擎证据。

## 所有权正式移交

01已干净封存于0183b401f805e5508ae5fdb20d53457eb4149504，全部子agent停止。下面仅从M0稳定基础扩展；不引入01恢复WIP。

| owner | 唯一可写的实施范围 |
| --- | --- |
| H1（原02 chat） | `poc/prototypes/i1b-runtime/src/normal_running/`、normal-hub/normal-worker、normal tests、`poc/proto/normal_running.proto`；原型manifest/lock/build/lib由H1 lead串行整合；最小共享扩展正式移交`native_scheme.rs`、`witness.rs`、`native_step_journal.rs`、`driver.rs`、`context.rs`及相应聚焦测试；02 `sim-transport`的版本化增量cid准入和可靠邮箱同步也由H1单一writer负责。 |
| H1接口owner | 可新增轻量独立crate `poc/crates/healthy-session-api/`，只放三方共用的typed命令/观察/证据引用/ports；全部公共域对象引用03，发布编码引用07，不承载第二套协议实现。先交可编译接口及独立向量，后实现runtime；H2/H3消费该接口而不反向让H1依赖Manager或bridge。 |
| H2 | `poc/crates/session-manager/`、`poc/verification/session-manager-healthy/`及专属合同/报告。只有Manager逻辑和私有计划/状态表示，不改H1 runtime、公共types或其他feature。 |
| H3 | `poc/crates/publication-bridge/`、已有`poc/crates/viewer-gfr/`必要复用/抽取和专属验证/文档。16当前idle，额外移交`poc/scenarios/espresso-compatibility/{capture.py,run-local.sh,README.md}`及`poc/verification/espresso-compatibility/check_capture.py`的最小artifact输入扩展；保留旧markers模式/来源检查/现有回放行为，不复制启动/进程管理。06源码暂不移交，消费现有trait。 |
| M1 merge chat | 跨feature提交整合、必要workspace/module接线、launcher和`poc/scenarios/m1-healthy-regular/`、独立系统checker/结果。只能整合各owner已交付的准确候选；实质性协议修改先返owner。 |

01恢复模块、原恢复patch/crash fixtures仍只读。H1若确需Commonware opt-in hook，先给具体文件、源码规则、guard/输出与独立对照；本移交不授权改quorum/lock/vote或全局回调语义。根types/manifest的其他变更交独立merge owner串行收口。

## 已决边界与有限剩余决策

1. **动态cid选方案A。** 采用H1提案的版本化增量context admission。固定base config，revision/hash单调；RouteKernel与ReliableMailbox必须在同一受控提交中同步准入事实，保留旧queues/counters/inbox/outbox/custody；不能清空重建kernel或静态预装未来cid。H1冻结字段与向量并实现，H2/H3只消费。
2. **每worker一个持久提交owner。** H2产生不可变ManagerDelta/action intents，H1拥有root/aux真实持久提交和engine guards。完整GFR ReplayRecord是生命周期批次，复用全部GFR-LIFE-1事件身份，不按position单独去重。采用`Committed(actions_pending)`与`ActionsApplied`分层：完整合法record预检后，关闭/tombstone、Selected+Preparing、cursor及action intents共同提交；缺后继执行材料不延迟已知合法关闭，也不允许提前Enter。结构/来源验证未通过的record不能冒充此状态。cursor表示整批义务已持久接受，不代表下游action都完成；H3的GFR pc不因Manager落后而回滚。H1/H2冻结具体receipt/hash/错误与幂等ID；不为M1引入另一套服务数据库事务。
3. **终止单调且失败关闭。** durable terminal/tombstone或ClockUnsafe守卫安装后，后续失败不能回滚为Running/重新签名。未完成assignment/cursor保持pending，必要时当前step/runtime停止并报告；重试不能清除安全守卫。提交前可拒绝未生效计划，提交后保留事实和待办，不伪造成功或做逻辑回滚。
4. **Pause保留原实例。** H1停车cid任务但保留engine/future、mailbox、原绝对deadline、callback和签名历史。H2只决定候选/调度；SelectBack不是第二次Enter。到期事件在原实例恢复后按受控顺序处理，不以now+timeout延期。当前无OS重启，故不要求为此实现恢复cut。
5. **精确ReadyState。** H1验证03 ExecutionAnchor及checkpoint/真实前缀来源、材料和执行，产生绑定worker/cid/anchor/验证版本的ReadyToken；H2不能传Ready bool或仅root。缺材料WAIT，已知错误REJECT。ClockUnsafe用私有local cause+持久guard，不能映射成公共Time或擅改03公共enum；已终止cid不复活。
6. **真实验证与本地准入分离。** H1是FinalVerifier/namespace/PoP/Accept映射owner；复用03 trait、07 VerifierBinding/Native profile。全球确定性payload合法性和node-local首收/clock/stop不是一个谓词；WAIT或本地过期不得直接转成全局certify(false)。具体Commonware callback/abstention/cancel映射必须由H1 Docer与独立Verifier针对锁定源码闭合后实施；不阻塞其余guard/实例/route基础工作，不能默认开门来跑happy path。
7. **发布与完整性。** H3复用GFR-PUBLIC-1，必要时从07抽出公共完整性helper供Publisher/GFR共用，不能引入第二套q_avail算法。完整结果、proof观察、公开完整与G接纳分开。跨层Stop依据来自已验证公开position，不能把H1本地TIME/Pause当公开停止。
8. **有界小payload优先。** M1使用会改变状态的小型交易工作量，H3冻结并核算完整object/result/attempt的真实编码上界，保持02每包≤64KiB、06每消息≤64KiB及块≤1MiB。必要的transport分片/重组和credits由H3独占实现，H1提供固定channel/endpoint/受控出口；不改变公共对象编码，不把完整publication拆成丢失原身份的新任务，不一次无界fanout15份占满队列。先完成M1所需有界profile，不建设通用大数据网络。
9. **SessionEnd两种证据分开。** h9→h11主例必须提前准备/生产合法h11并缓存，获选后真实接纳；若未提前生产，则允许后继在当时合法slot如h14继续，并覆盖真实padding，但不能宣称已验证h9→h11或倒填首收。两者窗口都保持11–20，BFT进程健康，publication延迟另记。
10. **官方发布入口归H3。** 以小型artifact-input适配复用16现有runner，传入主场景实际公共结果字节，读回校验payload/hash并复核区域证明；保留原marker测试兼容。官方时序Unknown/Unsupported、可信本地API与QC/VID未验证边界不变，不影响SimEspresso确定性主trace。

结构/语义职责以上述决定为准。序列化tag、错误枚举、具体Rust签名、队列界等由对应owner在版本化合同及独立向量中冻结，不需要总协调逐一批准普通实现细节。不得把前述局部决策扩大成重新设计整套系统。

## 立即执行顺序

- **H1现在推进**：保存准备成果、同步准确M0；修订独立审查发现的问题；首先发布三方可编译API和goldens，再并行于H2/H3完成普通runtime、15PID/五引擎、slot-aware/真实证书。每项源码切片先固定独立预期，单writer实施。既有M0已执行的兼容检查不因只新增文档重复，改变shared行为后再重跑适用回归。
- **H2现在推进**：在H1接口提案及本决定上冻结本地Manager计划/状态机合同，独立预期已齐备的部分立即实现；生产runtime接线只等待真实H1实现，不等待H1整项正式矩阵。H1 API未发布期间可实现不依赖该crate的内部状态转换与独立用例，禁止复制公共API。
- **H3现在推进**：实现已有03/06/07之上的codec/Publisher适配、公共完整性/stop和有界传输；用现有FinalVerifier依赖注入接口开发，真实H1验证器到位后接线。先补16 artifact-input小适配，不能拿test fixture判定当真实M1。接口未实现的局部保持显式gate，其余可编码。
- **M1 merge并行准备**：远端独立chat现在准备launcher/场景/checker与输入证据约定，并整合精确docs/API提交。功能代码只接已交付且复核过的SHA；持续增量集成，不等三个“大包”全部结束才发现不兼容。没有真实全部链路前不能报告M1 PASS。

各owner尽早在进展给出小接口提交SHA/消费例子/未覆盖项；总协调用消息交给消费者。H1/H2/H3在本次授权下可自行完成约定小切片freeze→实现→复核→测试；只有改变协议不变量或越过他人路径才回报总协调。无需因“没有整体freeze”停止所有模块。

## 基线接收与集成约束

保留已有worktree/分支/准备提交。可从准确M0创建自己的implementation分支并重放**仅自己拥有的准备文档/独立向量**；旧准备分支/提交保留，不reset、不丢修改。无冲突的M0基线同步不算重新集成功能；跨feature源码合并、冲突与共享模块接线必须由独立M1 merge chat负责。它交付明确integration SHA后，下游再同步。

角色仍为Astra/ultra主chat、Docer/Verifier Astra/xhigh、Builder/Tester Sol/high；最多5活跃chat。当前目标H1/H2/H3+M1 merge共4，保留一槽处理明确阻塞，不用为凑并发开启无关功能。所有重任务用宿主flock和数据盘独立target/artifact。保留原完整故障验收OPEN；成功标准仍是15真实worker的M1端到端结果，不缩为组件smoke。
