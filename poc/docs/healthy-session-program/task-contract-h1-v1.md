# H1 v1：健康 Session Program 任务合同候选

日期：2026-11-16，Asia/Shanghai。修订：**M1实施授权后的分切片合同**。正式授权为 `d55d39f73593676325bdc0fbb7ba3845ca93025f` 的 `poc/docs/milestones/M1-implementation-dispatch-20261116.md`。M0已交付 `35d382ac445cc8bc779f0d18fd23cb57e54680da`，当前H1基线 `4741489ba7a07d2a3f3f194dbd59ae3fd8d17bb9` 已正常合入。H1、NR-T、NR-A尚未通过；[API切片合同](api-slice-v1.md)先冻结、形成独立预期并交Builder，其余局部Q只阻塞对应实现，不阻塞全部工作。

配套：[接口提案](interfaces-h1-v1.md)、[准备与版本记录](preparation.md)。独立 Verifier 的预期先于实现形成；本稿不是其独立验收报告。

## 1. 权威与版本

优先级为最新 milestone 执行范围 → 根 `AGENTS.md`/`DESIGN.md` 与 v2 明示 mock → v10 协议语义 → 固定 Commonware 实现映射。最新 milestone 改变近期工作顺序，不削弱持久化、原实例、终止、签名或公开接纳规则。

| 引用 | 固定版本与节 | 本合同采用的规则 |
| --- | --- | --- |
| M | 基础计划 `4a9a6b0a92a11e5778833988bb6524d929f4608e` 的五份milestone文档；最新派发 `d55d39f73593676325bdc0fbb7ba3845ca93025f` 的 `M1-implementation-dispatch-20261116.md` 优先 | M0已交付、M1立即分切片实施、正式所有权移交及十项接口决议；Force暂不实施。只读权威源位于 `/mnt/reth/jy/megaroation_poc/`。 |
| V2 | [v2](../../../formalization_v10/poc_design_zh_v2.md) §§3.1–3.3、4、5、6、9.1、10.2、12 | 三个五成员委员会，q=4，10-height 窗口，误差与 RPC 延迟分开，单时间权威和完整 grant/Ack。 |
| SP | [v10 Session Program v0.1](../../../formalization_v10/02_session_program_zh_v0_1.md) §§3.1–3.12、4.2–4.9 | cid 独立状态，Create/Enter/Close，Build/Accept5a–5d/NotBefore，CheckFINAL，停止签名与有条件进展。 |
| SM | [v10 Manager v0.1](../../../formalization_v10/03_session_manager_zh_v0_1.md) §§3.1–3.10、4.1–4.3 | Ready 的精确状态依据，候选表、一次 Enter、Pause/选回原实例、全部候选的单调关闭。 |
| GF | [v10 GFR v0.1](../../../formalization_v10/09_finality_rules_zh_v0_1.md) §§3.2–3.4、4.2–4.3、4.7 | proof/body 完整性、公共顺序、满窗与 SessionEnd、原子生命周期；本地 stop 不替代公开截止。 |
| BFT | gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` 的 `DESIGN.md` §§1、2、2.1、3.1、4、5 | 上游协议与 adapter 区分、证据与交付分离、WAIT callback、验证前 parent 使用、写后发送。 |
| CW | Commonware `2026.10.0` / upstream `92a2f68a0e131c8b69de41e84aec37da6280f212`；`consensus/src/simplex/mod.rs` 的 Protocol Description / Certification / Deviations，`types.rs::Finalization::verify`，`consensus/src/lib.rs` 的 Automaton/CertifiableAutomaton，`utils/src/faults.rs::N3f1` | Native notarize/nullify/finalize subject、最终证明、原生 parent/skip/签名守卫、单次 callback 与 certify 的确定性要求。源码已从本机固定 release cache 只读读取；Simplex 论文为其说明来源，实际 quorum 按该 release。 |
| 03 | `e9c15e04bc83b335668f3ecd837367aa87344a89`，`mock-plan-public-interface-v1.md` §§2–4、`task-contract-mock-plan-execution-v2.md` §§2–6 | 唯一 domain 类型、canonical BE v1、真实 registry/PoP、Plan/cid、PhysicalBlock、Apply/Merge、逐 reader L1 fixture、FinalVerifier trait。 |
| 02 | `e1a435712ec492cae2bbff120a51ffd507eebf45`，`task-contract-sim-transport-kernel-v1.md` 和 `poc/crates/sim-transport/` | 路由/Clock/可靠 custody/服务 grant 组件；动态 cid 准入尚不是现有 API。 |
| 06/07 | `4b034dfbb6c3e1c7143208347b546daab8dfe298` / `788291735ce6db3c3eb78b4594a28d730cce71cd`，PE-PUB-1/PE-LOG-1/PE-READER-1，GFR-PUBLIC-1/GFR-LIFE-1 | H3 消费边界；组件 fixture 结果不代替 H1 密码学验证。 |
| NR | `8a5e3002dc579fedd3bc9f21cfac2c4fe90209a3` 的 `normal-running-reg-a-preparation-v1.md` | 保留 NR-T/NR-A、签名/首收/持久化建议。其单 cid/R0-only、停车全归旧05、feature 自行合并组件提议被 M 覆盖。 |

本 checkout 的 BFT 子模块未初始化；只读 `/mnt/reth/jy/megaroation_poc/regional_BFT_sim` 中准确 pin，未初始化/写入任何子模块。根 `DESIGN.md` 是指向上述 v2 和 pin 的权威索引。详见准备记录中的 source provenance。

## 2. 本次范围与通过标签

**P-SCOPE。** H1 交付真实运行能力：15 个健康 OS worker，固定 incarnation=1；每个地区五个独立身份，选中地区至少五个真实 Simplex engine；其余十个进程持续有自己的 Clock、公开 reader、root-step。它们不能为不属于自己的 cid 签名，也不能为满足 Ack receipt 格式而伪造实例/witness。后继准备或候选调度可以随后使这些进程进入合法实例。

H1 必须支持 R0→R1→R2 所需的动态 Create/Prepare/Enter/Pause/Close、按 cid 隔离 engine/签名历史/timer/storage、同一进程 A→B→A 恢复原实例。H2 拥有 Manager 的候选选择纯逻辑，H1 拥有真实 runtime 停车、调度及签名出口；H3 拥有 publication/GFR。H1 不自行合并其他功能实现，不将全部停车能力推给 H2。

| Gate | H1 可交付的证据 | 不能由该 gate 代替的终点 |
| --- | --- | --- |
| NR-T | 15 真实进程注册、独立 Clock/reader、真实消息、timer、完整 root/Ack、可审查 cid 准入和 lifecycle 接线 | 五引擎十高度、公开 FinalVerifier、M1 G 接纳。 |
| NR-A | 五真实引擎健康 REG-A、slot-aware Build/Accept、真实证明/完整交付、边界矩阵、同进程停车与选回 | 整体 I1、全部恢复范围、M1/M2。 |
| H1 交付 | 冻结接口 + NR-T/NR-A + 可被 H2/H3 消费的版本化实现/独立报告/同 SHA 证据 | 仅 R0 样例不够；完整 M1 由集成任务验证。 |
| M1 集成依赖 | 15 进程持续运行，R0→R1→R2；真实 final/body 经 Publisher/SimEspresso/独立 GFR，R2 首个合格结果进入 G；另走 SessionEnd；同一真实结果字节经官方本地 Espresso 提交读回 | regional final、publication submitted、SessionAssigned 均不是 M1 完成。 |

当前只实施M1；Force暂缓，H1保留既有branch-close的typed入口但不实现Force策略或激活。跨 PID 恢复、D/RESUME、批次/DA/Web 已退出近期关键路径；相关旧故障 gate、01 R1–R18、04 重启范围、整体 I1 仍 **OPEN**。异常退出使场景 FAIL 或明确的基础设施 INCONCLUSIVE，不能空 journal 重启继续。

## 3. F：来源硬规则及触发合同

| ID / 来源 | 触发与前态 | Guard | 状态或输出 |
| --- | --- | --- | --- |
| F-ID / V2 §3.3、03 §§2–4 | 收到 descriptor、registry、消息或证书 | 重算 Plan/cid；15 pid 与 region/member/按 identity key 排序的 signer index 分开；真实 PoP；历史配置/epoch/view/kind/subject 一致 | 未知/冲突来源 REJECT，未交付所需来源 WAIT；每区四个不同合法 signer 才是 quorum。 |
| F-CREATE / SP §3.4 | Create 固定 Plan 和 anchor | anchor.h < window.h_min；配置与身份有效；同 cid 固定 descriptor | 首次 Assigned，不启动共识；重复返回原 handle/状态；已 Terminated 不复活。 |
| F-READY / SM §§3.2、3.7 | 取得部分或全部 anchor 材料 | exact anchor 身份/height/metadata/来源；从已验证 checkpoint 重放连续原生前缀并核对完整 ExecutionState | 缺 parent/body/DEP/L1/来源为 Preparing+WAIT；root 相等、URL 或全局磁盘存在均不构成 Ready。 |
| F-ENTER / SP §3.5、SM §4.2 | 已 Ready 且本地调度此 cid | 成员、来源/状态、Clock 安全、未 excluded/terminated，C_m < stop | 首次 Enter(view1)，记录原 entry_time；重复返回原结果，不 reset；提前 Enter 后仍等 NotBefore。 |
| F-PARK / SM §§3.9、4.2–4.3 | 同 session 从 A 选 B，再选 A | 最多一个候选得到共识调度；A 尚可选，返回前重查 terminal/Clock/stop | A 保留 handle/view/锁/签名/timer/pending 工作；选回只恢复原调度，不第二次 Enter，不重新构造 Engine。 |
| F-CLOSE / SP §§3.6、4.9；SM §3.10 | Close、C_m≥stop、ClockUnsafe 或公开关闭 | 包括 Assigned/Preparing/Ready/Running/Paused 全部作用域；持久 terminal 先于后续新签名 | 不可逆 Terminated；迟到准备/回调不能复活；保留 pre-stop 签名、blocks、proof、输出和 publication 义务。 |
| F-BUILD / SP §§3.7、3.9 | native 选择 parent P 后 propose | 到 NotBefore，slot(C_m) 在固定 window，P.h<h；parent/ancestry/L1/状态完整；发送前再次检查 stop | 执行 padding/DEP/普通交易，返回 03 PhysicalBlock；缺输入/time WAIT；窗口不按 Enter 或 view 移动。 |
| F-ACCEPT / SP §3.8 | 首次完整合格 proposal 或同请求重试 | 连接/metadata/执行前 root/BasicTx；5a–5d 使用持久原首收 | ACCEPT/WAIT(missing)/WAIT(time)/永久 REJECT；ACCEPT 不绕过 native parent/lock/nullify/签名守卫。 |
| F-CALLBACK / CW Automaton/CertifiableAutomaton、BFT §2.1 | propose/verify/certify 尚待输入 | response 单次终结；WAIT 不 resolve false、不 drop producer；request abandon 与暂停不同 | 同请求保持 pending；确已取消才处置并禁止迟到 completion 恢复；节点本地准入不得冒充各节点一致的 certify 谓词。确切 abstention 接线是 Q-ADMIT。 |
| F-FINAL / CW Finalization::verify、SP §§3.10、3.12 | 接收原生证明或 Reporter activity | 公开历史 scheme 验证 Finalize subject、四 signer、精确 physical digest/cid；native activity 先验并不一定 complete | notarization/certification/nullification 是进展证据；Finalization 是 finality 证据；完整经执行验证的 ancestry/body 才可交付 RegionalResult。 |
| F-PUBLIC / GF §§3.2–4.3 | H3 收集公开 proof/body 与日志前缀 | 逐 reader 完整前缀、q_avail/接纳键、watermark 和顺序 | H1 不决定 G 接纳；本地 stop/Clock 不能触发 SessionEnd，Close 不删除历史 publication。 |

Accept 时间字面边界固定：`δ_G=1000µs`，`Δ_G=2000µs`。5a `recv ≥ ts*(h)−2000µs`；5b `now ≥ ts*(h)`；5c `recv < ts*(h+1)+T_win`；5d `recv < stop`。5a/5c/5d 任一失败即永久拒绝；三项成立而 5b 未到才 WAIT(time)。公开 `CheckFINAL` 不读当前 Clock 重跑 Accept。

## 4. P：工程守卫与输出提交

**P-CLOCK。** 每 worker 同一 NodeClock 驱动 Manager/Build/engine；SimHub 唯一推进虚拟 τ。RPC request/sample/response 独立调度，只有实际交付且验证的 Sample 可续租；grant 不校准。clock error、RPC latency、本地协议时间、Espresso θ、wall elapsed 分列。绝对 local timer deadline 在 Pause/选回与续租后保持原值。

**P-COMMIT。** input/grant 校验 → 有界确定性工作与候选 effects → native journal/sync/适用 witness + 首收/准入/terminal/reader/custody 辅助记录 → 实际完整 NormalGrant/NormalAck root 提交 → Hub 安装/释放。具体格式见接口提案 Q-WIRE。所有实际 Send、Final/Delivery、timer/work、consumer advance 均归本步 Ack，后台不旁路。无 native artifact 的 step 仍有真实 root，witness 项为显式 NotApplicable；有 artifact 却无匹配真实 receipt 必须拒绝。

相同 grant 重放原完整 Ack，不再执行；篡改 grant、单个非法 effect、stale revision、缺 durable receipt 不产生部分外部释放。02 的 DurabilityReceipt 是 adapter 提供的事实，须对应真实 sync/记录，不当成组件自己证明 fsync。已持久释放义务不得因 stale plan 静默遗失；不声称能回滚此前已授权 native I/O。每worker仅H1拥有持久提交：完整合法ReplayRecord预检后，Manager opaque delta、关闭/tombstone、Selected+Preparing、cursor及action intents同root提交，返回`RootCommitted(actions_pending)`；另有`ActionsApplied`回执。缺后继执行材料不延迟已知合法Close。durable terminal或ClockUnsafe guard生效后的任何失败均fail-closed，不回滚Running；未完成动作继续pending，必要时停止runtime并报告。

**P-INGRESS。** 原始 header/body 收包时刻与认证完成时刻分离；保留 `(cid,epoch,view,native_parent_view,physical_digest)` 的合格首收证据，直接 leader 绑定真实 transport 身份，转发 header 要有真实 leader 签名或已验证 native 证明。首收是最早适用合格 header/body 配对的较晚原 ingress；无权 header 不能占位。LOCAL_BUILD 在完整 B 形成时记录，不要求预先有自己的签名。认证延迟/乱序须先处理可能改变首收的已 admitted 输入，再永久时间裁决；队列/认证预算冻结后可验证，不能无限等待恶意垃圾。重复投递或 WAIT 重试不改写首收。

**P-BOUNDS。** 提案 profile `healthy-session-program-v1`；old Fresh 构造器/字节不改。初始参数仅供冻结：genesis 10s、slot 1s、R0 window1–10/start11s/stop21s，后继固定11–20、21–30；各地区 offsets −1000/−500/0/+500/+1000µs；intra 20ms/cross 100ms/RPC 单向250µs；lease100ms/refresh20ms；T_win200ms；leader/certification timeout1.2s/1.5s、retry100ms。view/section/retention128 提案须核对 native journal 映射，profile 内不 prune；十高度不等于十 view。超资源界显式失败，绝不截断签名历史。真实五引擎尚未校准，不提前主张时限成立。

有界进展只对同 cid 持续调度且拥有完整输入的固定集合 U、|U|≥4、τ_end≤stop−δ_G 声明。τ_lc 从共同就绪/参与区间起算，通常 τ_sync=max(τ_lc+T_sync,start)；不从第一 Enter 起算。3/2 候选分裂不能合并算 quorum，暂停回切须重查区间。

## 5. 验收矩阵（全部 PLANNED / NOT_RUN）

每例须交付触发事件/前态/输入字节/独立预期/实际 trace；源码位置由 verifier 在独立预期后定位。密码学用真实 native 编码和历史配置；fixture 只供纯接口准备并明示，不计真实 gate。以下均未运行。

| ID | 最小事件序列或边界 | 必须断言 / 证据 | 对应 |
| --- | --- | --- | --- |
| H1-00 | 消费已交付准确M0的兼容记录；shared行为改变后在最终候选复验适用路径 | 不因仅文档/API新增重复M0 heavy检查；普通Engine::new、旧constructor/PoP/namespace/record兼容及受影响healthy/two-real-alone/proof-before-body/close/lease/crypto/runtime/storage/witness另记命令；不得把历史66次移植为新PASS | P-SCOPE/P-SHARED |
| H1-01 | 启动15 worker，三地区各五身份；R0 先运行 | 15 个不同真实 PID、incarnation=1，5 个实际 engine 的创建/运行证据、五真实私钥独占；10个独立 Clock/reader/root，无脚本 signer、无 Hub 代签 | NR-T/F-ID |
| H1-02 | 假 pid/member/signer、错 run/config/cid/epoch/codec、重复注册 | 精确类型映射与拒绝原子性；实际 participant 顺序来自 03 registry，不按 pid cast | F-ID |
| H1-03 | Sample 乱序/过期/延迟、lease 等号、ClockUnsafe；同虚拟输入不同 wall pause | 不免费续租、读钟/新签误差≤1ms、RPC latency 单列、同节点同钟；原 timer deadline 不改，暂停不能延长 lease | P-CLOCK |
| H1-04 | vote/certificate/resolver/body 分别经过 adapter；额外缺 body/cert 后补 | 真实原生字节和四 channel 计数；healthy 未触发 resolver 如实为0；另例真实 request/response，不称跨重启修复 | NR-T/F-FINAL |
| H1-05 | 五引擎健康固定1–10，每 height 生产 | 十个真实 physical blocks、独立重算 physical/native hashes、完整 ancestry、pre/post state/DEP/交易过滤；每个 Final 对应真实四签证明 | NR-A/F-BUILD |
| H1-06 | 3/4 distinct signer；重复、未知、跨地区 signer；位翻转 | 3拒绝、4合法通过；重复不充数，错误公钥/PoP/subject/bitmap均拒绝；不得由手工 bool 预期代替密码运算 | F-ID/F-FINAL |
| H1-07 | 错 namespace/cid/config/epoch/view/kind/payload/parent/body；Notarization 冒充 Final | CheckFINAL false；正确证书在任意观察 Clock 均可验证；body hash/执行失败独立拒绝，不把 notarization 记 final | F-FINAL |
| H1-08 | 5a/5b/5c/5d 分别等号及±1µs，其他条件固定成立 | 5a等号可过；5b等号解除 WAIT；5c/5d等号拒绝；5a/5c/5d失败不被5b等待掩盖；首收不更新 | F-ACCEPT |
| H1-09 | NotBefore−1/等号/+1；parent=h_max；缺 parent/body/L1/DEP 后到 | 原请求 pending，无提前 propose/new sign；+∞不提新payload但 stop 前允许合法旧消息；输入齐后同首收重试 | F-BUILD/F-CALLBACK |
| H1-10 | header先/body后与反序；认证延迟；早伪造header；转发header；LOCAL_BUILD | 首收为最早合格完整配对的原 ingress，非认证/重试时间；伪造不能抢位；审计原始hash/身份/时刻 | P-INGRESS |
| H1-11 | 同 cid Enter 为0/20/100/500/1500ms；3早2晚；高view late join | 无全员屏障；保留六类时间；从实际共同区间推导进展，late Enter不重置别人的view/窗口，不强求异常时序仍十physical块 | SP §4.6/V2 §6 |
| H1-12 | 两个合法来源 anchor 候选，五成员3/2分裂 | 不拼quorum，不声称共同有界进展；native签名域、缓存、timer/storage不串cid | F-ID/F-PARK |
| H1-13 | Create两次/Enter两次，先缺状态、补齐 exact Ready，再 Enter | 同一handle/entry_time；缺材料WAIT，不用root-only Ready；错anchor/state/metadata或越界prefix拒绝 | F-CREATE/F-READY/F-ENTER |
| H1-14 | A Running→B Running→A 被 GFR 选中；另例B未Ready | A同一engine/view/锁/签名记录、原timer/pending工作；无第二次Enter；B未Ready可继续A，公开排除后不得继续A | F-PARK；H2纯规则+H1真实engine |
| H1-15 | Pause期间timer到期、input/work完成、Close/stop；选回 | 原deadline保留，事件在受控队列保留且有界；Paused不新签；resume先检查Clock/terminal；stop后的A拒绝恢复 | F-PARK/F-CLOSE/Q-RUNTIME |
| H1-16 | Preparing/Ready/Running/Paused各相位Close；Close与propose/verify/certify completion交错 | terminal先持久；新签名数停止；迟到 Ready/completion不能复活；历史proof/body/output继续可取 | F-CLOSE |
| H1-17 | 同一签名坐标重复/冲突；pre-stop旧签名stop后拼证 | 原bytes缓存/冲突拒绝；stop后无CREATED；合法历史证书可观察但公开接纳仍由H3截止决定 | SP §4.9/CW |
| H1-18 | proof先到、body/祖先/执行输入后到 | ProgressObserved、FinalProofObserved、Delivered分开；缺body/祖先WAIT且无可消费Res；补齐后连续 verified prefix 一次交付 | F-FINAL |
| H1-19 | 一个非法effect、缺receipt、stale计划、重送grant/重复release、无artifact step | 无半份外部输出；实际完整Ack/root可重算；无artifact仍root且不伪造witness；有artifact必须适用真实见证 | P-COMMIT |
| H1-20 | 运行后真实来源产生未来cid，动态准入后发送；未知cid/伪来源先发送 | 已选择A：RouteKernel与所有受影响ReliableMailbox在同一受控提交、同admission revision安装；新cid只来自真实已验证descriptor/Prepare；旧队列、FIFO、ordinal、inbox/outbox/custody、timer、历史hash不变；不得空kernel替换 | P-TRANSPORT-A |
| H1-21 | 新cid准入重复/冲突、失效revision、Close后历史body/proof仍在途 | 幂等安装或原子拒绝；共识新生产关闭与历史data允许交付分开；不撤销持久义务，不改变旧frame解释 | Q-TRANSPORT/F-CLOSE |
| H1-22 | Prepare取消/Close后旧job迟到；同cid补材料、不同cid同height状态 | token/anchor/配置/来源绑定；旧completion不产生Ready/签名；允许缓存已验证材料但不能按height跨链复用 | H1-SPREP-1 |
| H1-23 | seeds7/42/99 × 正/反启动顺序 × wall暂停0/2ms，固定外部输入 | 同组完整虚拟trace/native帧逐字节相同；PID/wall诊断另存；不删协议事件制造相等 | V2 §5 |
| H1-M1a | 接H2/H3，R0满窗→R1满窗→R2首结果入G | 15 PID持续、真实lifecycle从独立GFR到Manager/Program；记录每区proof/完整结果/publication/G接纳；R2入G才是集成终点 | M；集成任务 PLANNED |
| H1-M1b | h10未及时公开，h9成为regular anchor；提前收到真实h9来源后准备/生产合法h11并缓存 | 这是指定h9→h11主例：必须在原h11准入窗内生产/首收，不能预知未来anchor/cid或倒填时间；窗口11–20，本地stop不触发public SessionEnd；获选后原历史结果真实入G | M派发§9/H2/GF §4.3 |
| H1-M1b-late | 未提前生产h11；SessionEnd后在当时合法slot（例如h14）生产 | 同样固定11–20；完整验证真实h10–h13 padding与h14，不要求十个physical；不报h9→h11通过，不补造h11首收或挪窗 | M派发§9；与主例分开 |
| H1-M1c | 主场景实际结果字节交16官方本地节点并读回 | hash/来源与SimEspresso结果相同，q/index/namespace/payload核对；wall/Unknown证据独立，不声称QC/VID或完整PE时序通过 | M/H3/16 |

测试证据必须包含负向 oracle：故意冲突 final trace、application-invalid body、未认证首收、脚本 signer 或 Hub 代签标记应被 checker 拒绝；不能只证明 checker 接受正常输出。有限矩阵只证明所列边界与调度，不是协议证明。

## 6. 路径、冻结门禁与交付

当前Docer唯一writer只拥有本目录的前三份文档及`api-slice-v1.md`、必要的`api-v1.md`；Docer不写代码、不build、不commit。正式H1实施所有权已由d55派发授予，不能再用旧“仅准备”门禁阻止Builder。

H1正式拥有轻量独立workspace `poc/crates/healthy-session-api/`，原型`src/normal_running/`、normal-hub/normal-worker、normal tests、`poc/proto/normal_running.proto`；原型module/build/manifest/lock由lead串行安装；`native_scheme.rs/witness.rs/native_step_journal.rs/driver.rs/context.rs`最小normal扩展与相应聚焦测试、`poc/crates/sim-transport/`增量admission/可靠邮箱同步正式移交H1单writer。实际每切片只开放其allowlist；API切片不触碰runtime。根types/根manifest、01恢复模块/patch/crash fixtures、其他feature和子模块仍只读。Verifier持有独立verification路径。

按切片冻结精确签名/guards/outputs/向量与单owner，由独立Verifier先保存预期，再Builder、独立复核与同SHA测试。API切片只需本地DTO/ID/shape约定。Q-ADMIT已通过两份独立源码审阅并选择[路线A](admission-bridge-v1.md)：本地Accept限制notarize准入，certify保持确定性V与完整持久数据，仍不代表native实现或活性矩阵通过。其余native相关Q在对应切片实现前闭合。M0已执行的兼容检查不因文档/API新增重复；shared行为改变后按影响复验Fresh。报告准确候选/验证SHA、dirty、命令、seed、trace、限制及OPEN项；API编译接口通过不能报H1或M1通过。
