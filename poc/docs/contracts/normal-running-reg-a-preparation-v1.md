# NORMAL-RUNNING / REG-A v1：普通运行接线与健康准入草案

日期：2026-11-16，Asia/Shanghai。状态：**DRAFT / NOT FROZEN / NOT IMPLEMENTATION AUTHORIZATION**。本文件只准备下一空闲功能槽的02普通运行接线→04健康REG-A任务；没有启动新chat、构建、测试或改变任何验收结果。冻结前，Verifier必须从来源独立给出预期，lead须批准本稿末尾的具体接口决议及路径移交。

用户已授权BFT worker持续运行时先验证regular、合法主动Force和RESUME。[普通运行路线](../commander/restart-free-handoff-track-20261116.md)（`0c15aacf0461a0d1ae2bf4718c18a51aaaaa805b`）与[04修订](../handoffs/04-session-program-zh.md)（`32a3eb6b31ff8c1dfe2d05dbc2ed26c0c031bb77`）覆盖旧[regular准备稿](regular-preparation-v1.md)中“所有native接线均等待完整01”的管理门禁。它们不改变v10的签名、时钟、输出持久化和终止规则。

## 1. 来源、准确基线与证据分层

权威为v2 §§3–6、10.2、12及v10 Session Program §§3.4–3.10、4.3–4.9，03已接受的公共对象/执行合同，以及02运输内核合同。Regional BFT gitlink保持`691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`；Commonware固定2026.10.0，上游`92a2f68a0e131c8b69de41e84aec37da6280f212`，Rust1.95.0。

| 层级 | 准确版本 | 可以据此主张什么 |
| --- | --- | --- |
| **已验收的普通原生运行参照** | `fec7ade1675cb60dd16a16fb28ebe81e3f76f508` | [Fresh结果](../native-engine-fresh-result.md)：66次、33对完整重放；两个真实worker和最多三个脚本signer，q=4。不是五真实引擎、slot-aware或15-worker验收；实际resolver包为零。 |
| 稳定的工程组合起点 | `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`；代码祖先`36eff1c76d5c4df5d0790133c0e14719c0c73ebd` | 现有文件布局与受控runtime的固定快照，包含后续有界恢复代码。**不能把fec7ade的完整Fresh结果直接移植到此SHA。** 普通入口仍调用`Engine::new`，恢复入口独立调用`Engine::new_with_recovery`。 |
| 02可消费运输内核 | 测试候选`4fd6f115bd969b0a8f252d7933edd21c27f00a8f`；交付`e1a435712ec492cae2bbff120a51ffd507eebf45` | `poc/crates/sim-transport`生产tree均为`3d2ff54ca9231d2611e424599b0db73170faa9a2`；46独立用例、12组件重放通过，真实native worker数为零。 |
| 03已接受组件 | `e9c15e04bc83b335668f3ecd837367aa87344a89` | `poc-types::domain`、`poc-authorization`、`poc-core`：真实registry/PoP、Plan/Descriptor/编码、Apply/Merge、独立L1 fixture。`FinalVerifier`是接口，没有默认true实现。 |

只读Git核对：上述ada、02交付e1a、03交付e9及本稿读取时源HEAD `04e0536127a67b82a955031b050cd2f0c39542d4`的`poc/prototypes/i1b-runtime` tree都为 **`e572b30e7c1af6123a6ba968cd20dd03d90689eb`**。所以02/03尚未把01的最新恢复WIP混入原型；这只是源码相同性，不是新的运行PASS。fec7ade之后已有实质性原型与vendored hook变化，不能称两者相同。

下一任务从总协调给定的干净managed worktree正常合并03交付、02交付与最新调度文档，产生一个**新的、实际记录的组合SHA**；本稿不预填其值。合并必须保留当前commander登记和各来源历史，不整体恢复旧目录，也不纳入01工作分支的未验收变化。以fec7ade作为普通运行行为参照，在组合SHA先通过§8的兼容gate，再进入新profile实现。若兼容失败，定位准确文件/接口并修复，不能通过调用01恢复入口或复制旧Commonware绕过。

I1a `35112ac2188287269d6faefb567466687beefeed`、runtime `3ab4549e48bf6e3e8fe49beed828430c974dba5e`、storage `4f889afeb6901294a8f941a009cae3f759388485`、witness `050c59b9e446e0aaf295f24e3d3a15357b00c443`各自只证明已有结果文件声明的有限子项。新组合需要复验受影响部分；它们均不关闭整体I1。

## 2. 本阶段交付与明确不覆盖的故障

拟命名profile：`normal-running-reg-a-v1`，名字和字节常量待冻结。15个真实OS worker；一个选中地区的五个成员运行五个真实Simplex引擎，其余十个worker注册、取钟、接受各自已交付输入，但不持有所选cid的签名权限。健康主例选`(β,r)=(0,0)`、R0、窗口1–10，真实私钥只交给其所属worker；Hub不提供脚本票、不代签。

每个worker incarnation固定为1；禁止自动重启、同cid销毁重建、加载恢复cut、`RecoverEngine`和`RecoveredAck`。意外退出使该场景FAIL或按明确基础设施条件INCONCLUSIVE；不能创建空journal继续。可以注入消息延迟/丢包、Clock回复故障、Close、永久stop及不重启的网络隔离。

十个未进入R0的worker仍需自己的root-step持久记录，但不能为取得一个非零witness receipt而伪造R0实例或提前签名。新Ack对“没有native artifact的Clock/reader step”和“带native受见证输出的step”作显式区分：后者必须有适用policy/真实receipt，前者仍需完整root提交。此字段/guard区分须在新wire与独立用例中冻结，不能照搬旧双worker一律已配置witness的假设。

**不重启不豁免持久化。** 签名、首收/Accept依据、terminal状态、native artifact witness、root-step和已提交输出仍须保留原持久边界。只把跨PID恢复与崩溃切点验收留OPEN，不能删除正常路径的sync、日志或Ack提交。Manager的完整多cid停车与选回归05；本稿只覆盖最小Create/Ready/Enter/Close及独立候选分裂诊断。

交付分两项可消费里程碑：

1. **NR-T：02普通接线。** 真实worker的Clock RPC、timer、消息、受控root/Ack与02内核接通；版本化公开输入能驱动03逐reader fixture。此里程碑不计完整02 accepted。
2. **NR-A：04健康REG-A。** NR-T上五真实引擎、slot-aware准入、真实公开验签、十高度健康生产和错时Enter等通过。报告为“04无BFT进程重启profile PASS”，不能据此关闭04的重启条目或完整I1。

## 3. 组件如何接入：保持唯一语义来源

### 3.1 身份、配置和签名域

- 直接消费03的`ValidatedRegistry`、`StaticScheduleProvider`、`Descriptor`和canonical BE v1。`Pid(0..14)`、`MemberId(0..4)`、实际Commonware `SignerIndex(0..4)`分别记录，禁止强转代替映射。03公开fixture的三地区signer顺序为`[4,3,1,0,2]`、`[7,6,5,9,8]`、`[13,11,14,10,12]`；运行时仍按真实identity公钥排序重算并核对。
- transport `Endpoint::Replica(u32::from(pid.get()))`负责进程身份；native消息中的局部member只在已验证descriptor所属region内解释。成员身份、公钥、source endpoint、目标endpoint和cid必须同时匹配。所有normal入站拒绝`ScriptedProvenance`，不能保留旧“member≥2就是脚本”的分支。
- 公开`ConfigRef`、02 `kernel_config_hash`与runtime run-manifest hash是三个不同对象。冻结`RunBinding {run_id, public_config_ref, registry_hash, transport_config_hash, adapter_profile_hash, descriptor_hashes, codec_ids}`；run id最后由非循环的manifest preimage生成，原始manifest中不包含自己的最终hash。
- 03的PoP域是 **`PoC/Registry/PoP/v1`**。旧`WitnessPolicy::new`固定用`I1b/storage/pop/v1`，不能把03合法key表直接交给它而宣称已兼容。需§6的profile扩展，保持旧构造器和旧fixture域不变。
- 新normal签名namespace建议由公开`config_ref + cid + epoch + verifier-profile-version`唯一导出；具体domain和黄金向量须先冻结，且engine signer、witness verifier与公共`FinalVerifier`同源。run/transport身份在外层隔离，不能让历史CheckFINAL依赖观察者本地wall time或秘密配置。新增verifier版本必须进入ProgramConfig/Descriptor；不能在同cid内改变既有签名域。

### 3.2 02内核到真实root提交

消费已有`RouteKernel::plan/commit`、`ClockTracker`、`ReliableMailbox::plan/confirm`及`ServiceGrant/GrantConsumer`类型；不再实现一套排序/丢包/FIFO算法。02内核的`DurabilityReceipt`是adapter提供的持久事实，不是它自行验证过fsync的证明。

一次step先完整验证target/run/config/incarnation、grant/hash与所有effects；对队列/计数/reader/outbox变化形成候选，持久root和必要业务/aux记录完成后才安装并释放。任何effect拒绝时不能出现半份Hub投递或半份收件消费。这个拒绝原子性不声称回滚此前已获授权的native I/O。相同grant重送返回原完整Ack；修改后重送或重复release不得再执行逻辑输出。

持久`CustodyPlan.persistence_bytes()`须与owner、transition id、candidate record hash、root receipt绑定；只能在真实持久成功后`confirm`。业务消费与`MarkConsumed`需同一提交边界。已经持久的释放义务不能因plan stale被悄悄丢弃。normal阶段不调用恢复API，但照样保存将来审计所需的完整记录。

当前02 kernel的permits静态、Rebind只改incarnation。REG-A只安装本次公开已知的单一cid及fixture服务权限，无需等待动态权限扩展。不要把“为未来所有session预装权限/清空重建kernel”作为05/08的后续解决方案；动态权限更新是单独版本化接口里程碑。

### 3.3 唯一Clock与公开输入

worker内部继续由`ControlledDriver`及同一个NodeClock驱动Manager/Build/engine；02 ClockTracker若用于Hub校验/服务ClockView，只是同一真实Sample流程的投影，不成为第二个取时权威。RPC request/sample/response分别排队；只有实际收到且验证通过的Sample续租。grant不免费校准，clock误差±1000µs与RPC延迟独立，sleep注册绝对local deadline，OS暂停不推进虚拟时间。

03 `CheckedSetup`、`FixtureFinalizer`、`FixtureReader`的`finalize`、`deliver_header`、`deliver_empty_control`三个动作分别进入受控step。每个worker有独立reader，不共享已变更的reader状态；完整fixture文件不是读取者的未来可见前缀。消息可乱序到达，缺control覆盖或metadata仍WAIT。

## 4. 从旧fixture到健康REG-A的最小行为变化

| 旧限制/位置（e572原型tree） | normal需要的变化 | 应保留的边界 |
| --- | --- | --- |
| `native_hub.rs`的双worker数组、旧固定route | 新adapter管理15个`Pid→Process/Binding`，由02内核统一路由与wakeups | 一次仅授予一个可审查step；OS到达次序不变成协议顺序 |
| `native_config.rs::Resolved`的2 offset、2高度、脚本key和固定body | 新`NormalProfile`引用03对象、15 clock profiles、10-height窗口和可配置view限额 | 不改旧Fresh profile含义；height和view不是同一个计数 |
| `native_network.rs::validate_envelope`中local source<2、脚本source≥2 | normal网络adapter使用03身份/02endpoint绑定，所有五名sender均是真实worker | 真实vote/certificate/resolver/body四channel字节和校验不改；无脚本旁路 |
| `native_application.rs`的合成Block、固定Archive/work | 新应用adapter消费03PhysicalBlock/执行；按真实native parent/view/digest调用Build/Accept | WAIT保留活producer；Final proof观察与完整body/ancestry验证交付分开 |
| `GuardedScheme::sign`硬限view1–8 | 同一守卫逻辑增加显式normal profile范围 | 原subject/cache、同坐标冲突拒绝、Clock/lease/terminal/stop检查不弱化 |
| `WitnessPolicy::new`的旧PoP/namespace/section限制 | 验证normal registry与显式有界journal profile | 所有native artifact仍经实际存储sync和同一witness授权；不能关掉见证来产第9/10块 |
| `StepJournal::create(root,&Resolved,worker)` | 提供profile-neutral创建绑定和normal记录扩展 | 使用同一持久append/head/完整Ack机制，保留旧恢复解析及旧记录意义 |

健康主例使用03的配置/fixture，建议沿regular准备稿：genesis timestamp10s、slot1s、window1–10，start11s/stop21s，协议虚拟运行从10s开始。各地区offset为−1000/−500/0/+500/+1000µs。healthy Enter skew可用0/20/40/60/80ms；另测规定的0/20/100/500/1500ms。跨worker真实共同参与区间必须由trace计算。

初始预算沿已提出的1.2s leader timeout、1.5s certification timeout、100ms retry、区域内20ms、RPC单向250µs、lease100ms/refresh20ms、T_win200ms；这些是**待真实五引擎校准的参数提案**，不提前承诺满足时限。native view上限与journal section上限建议一起取128、retention128且profile内不prune，确保十个height不是十个view；freeze时需核对实际Commonware journal映射与资源界。超界明确失败/前提不足，不循环计数或截断历史。

## 5. Build、Accept与公开证书合同

Create验证03 Plan的公开reference和descriptor，固定生产anchor身份。Enter验证exact anchor的来源、执行后状态、reader准备和健康Clock；同一cid只Enter一次，重复返回原结果，拒绝已Terminated或非成员。十个非成员进程的活跃时钟/网络不能被当成R0 quorum。

Build依实际native parent选当前local slot，执行padding后构造03 `PhysicalBlock`；不按view编号伪造height。NotBefore为`max(start,ts*(parent.h+1))`，到末高度后为无穷；等待下一slot不由worker自行快进。应用合法性和BFT的parent/lock安全规则都必须成立。

冻结首收key至少绑定`(cid,epoch,view,native_parent_view,physical_digest)`及对应原生proposal身份。完整收包时刻为合格header与完整body两者原始ingress较晚者；保留最早适用配对，不用完成认证或重试时刻覆盖。直接leader需认证transport及真实leader/participant绑定；转发header需实际leader签名或合格原生证明。无权header不能抢占首收。LOCAL_BUILD在完整B形成时记录，不要求先有自己投票。

v10 Accept5a–5d不变：`τ_recv≥ts*(h)−2000µs`；`C_m≥ts*(h)`；`τ_recv<ts*(h+1)+T_win`；`τ_recv<stop`。5a/5c/5d失败永久拒绝；其余成立而5b未到为WAIT(time)。WAIT(missing/time)保留同一工作/首收标识和活回调。延迟认证时先解决已admitted、可能建立更早首收的输入，再给出永久时间结论。来源认证预算/队列上限要冻结，不能无限等待早先恶意输入。

在返回propose、verify(true)、certify(true)及实际新签名出口前，检查该步授权、local Clock/lease、entered/terminated/stop和已经持久的适用Accept依据。永久本地时间拒绝采用明确的abstention处理，不把WAIT当成native false，也不把丢弃producer误当成功取消。

实现03的`FinalVerifier::check(&Descriptor,&PhysicalBlock,&[u8])->bool`：实际解码锁定Simplex Finalization，重建历史五成员scheme与normal namespace，检查epoch/view/subject/block digest及四个不同合法signer；仅有Notarization不得当Final。不能用脚本布尔oracle。它不读当前clock重跑Accept，也不替代Plan来源、ancestry或GFR选中检查。实际native payload digest绑定03的**physical digest**，G/D比较仍用03的**native hash**，不能互换。

## 6. 新路径所有权与必须先串行的公共变化

建议正常功能唯一Builder独占新文件：

```text
poc/prototypes/i1b-runtime/src/normal_running/
  mod.rs, config.rs, hub.rs, worker.rs, network.rs,
  application.rs, admission.rs, public_verifier.rs, wire.rs
poc/prototypes/i1b-runtime/src/bin/normal-hub.rs
poc/prototypes/i1b-runtime/src/bin/normal-worker.rs
poc/prototypes/i1b-runtime/tests/normal_running_*.rs
poc/proto/normal_running.proto
poc/scenarios/normal-running-reg-a/
```

同crate新module可复用现有`pub(crate)`的受控能力；不复制`ControlledDriver`、Clock、Blob、Commonware engine或整套witness/root journal。Verifier独占`poc/verification/normal-running-reg-a/`，Tester的原始数据放本任务的数据盘专属目录。上述只是待lead冻结的allowlist，不能凭本稿开始写。

以下是**具体共享接口 gate**；总协调指定一次只由一个owner提交，并把相同提交交给01及普通路线，不等待01整项验收，也不能在两条分支各自改出不兼容版本：

| 精确已有路径/能力 | 最小提案及责任 |
| --- | --- |
| 原型`src/lib.rs`、`Cargo.toml`/`Cargo.lock`、`build.rs` | lead登记`normal_running`模块/二进制、new proto，path依赖02独立workspace和03三个crate；维持现有Commonware两个patch的唯一来源。不得复制新的vendor或让Cargo选出另一版本的runtime/consensus。 |
| `src/native_scheme.rs` | 01当前所有者或获串行移交者增加显式`SigningProfile {epoch,min_view,max_view,namespace/profile binding}`构造入口；旧`new`保持Fresh1–8。normal复用同一受控守卫/缓存而非裸`Scheme::sign`。 |
| `src/witness.rs` | 独立review的normal policy构造入口，字段覆盖PoP domain、verified region registry、namespace、cid、worker/member、partition与有界sections。不能接受任意namespace/skip-verification布尔开关；profile映射必须校验。旧构造器与旧seal编码保持兼容，扩展需要新版本及黄金向量。 |
| `src/native_step_journal.rs` | 将创建所需的`run/config/worker/profile`绑定从旧`Resolved`中显式传入，原入口作兼容包装；normal公开输入/Accept/custody记录需与完整Ack同一持久边界。采用版本化扩展或共用append原语，不能复制第二套不受审查的journal实现。旧recovery读取不应猜测新格式。 |
| 必要时`src/driver.rs`/`src/context.rs` | 先用现有controlled IO/aux能力；仅缺最小admission/terminal辅助持久提交或scope读取接口时，逐个提出签名及guard再串行增加。不能暴露能跳过authority、直接更新时钟或无grant释放输出的接口。 |

**普通路线不直接写**既有`native_recovery*.rs`、01的crash fixtures或恢复hooks；也不改现有`native_replica.rs/native_hub.rs/native_config.rs/native_application.rs/native_network.rs`来抢占01写集合。对通用逻辑的必要提取必须列明小型diff与旧调用者兼容测试，再由相应owner交接。

新RPC建议用独立`rotation.normal.v1` package，冻结以下字段而不重解释旧`native.proto`：

- `RegisterBinding`：schema/profile、run、runtime_config、public_config、transport_config、registry hash、pid/region/member/signer index、incarnation=1及clock domain。
- `NormalGrant`：binding、step/event/tick/causal-wave、完整input与其hash；input需能表达Sample、Timer、Create/Enter/Close、真实transport delivery、原work completion、L1 header/control delivery。每种opaque bytes要有codec id和明确长度界。
- `NormalAck`：原grant/hash、status、全部effects、root sequence/hash、witness receipt、适用的admission/terminal/custody记录绑定；effect包含受控send、timer/work、sign audit、proof observation、verified delivery及lifecycle。
- 原型root journal的normal扩展必须持久覆盖**实际NormalGrant/NormalAck字节**，不能只记录一个省略公开输入的旧Grant投影；proto字段号、canonical/hash、max frame及记录版本由lead先冻结并交独立Verifier。

## 7. 当前可以先派发的顺序

**P0—组合与接口。** 下一个空闲槽领取本稿及来源提交，lead整合02/03、冻结profile/API小提交；Verifier先导出独立身份/namespace/首收/时序/提交预期。先做准确组合SHA的Fresh兼容gate，明确它与fec参照的差异。

**P1—NR-T。** 在新adapter接通15个真实进程的注册、RPC Sample、timers和grant/Ack；五个地区内endpoint使用真实native消息，其他worker保持独立Clock/reader。先验证route/Clock/完整提交，再启用生产。不得以纯kernel例子的0 native worker结果替代。

**P2—NR-A。** 接03 Build/Accept、五引擎和公开FinalVerifier，完成§8。其中3/2候选分裂用两份经过有效来源验证的不同descriptor；不能用伪造anchor或同cid篡改配置制造分裂，也不需要提前完成05的全候选parking。

每阶段保存命名候选SHA、独立复核结果和同SHA矩阵，再向下一阶段发出接口。01继续其恢复工作；普通路线只向01提供新增normal profile/持久记录的准确版本，后续重启兼容另开验收。

## 8. 验收矩阵草案（均为PLANNED / NOT RUN）

| ID | 本profile可判定的结果 |
| --- | --- |
| NR0 | 组合前后核对来源tree、patch/lock与普通`Engine::new`路径；至少运行既有Fresh healthy、two-real-alone、proof-before-body、close/lease与密码学负例和runtime/storage/witness受影响独立测试。最终共享scheme/witness/journal改变后对同集成SHA重跑完整Fresh矩阵；不把旧66次当本次结果。 |
| NR1 | 15个独立PID、固定incarnation、三地区真实registry；R0五个真实engine，其余十个不能为该cid签名。假pid/member/signer、重复注册、run/config/cid/epoch/codec错绑定均原子拒绝。无脚本private key或Hub代签。 |
| NR2 | 逐次Clock read/sign误差≤1ms；sample回复延迟/乱序/过期、lease等号、原timer和重复grant。Manager/Build/engine同钟，wall pause不推进τ，失败样本不刷新lease。 |
| NR3 | vote/certificate/resolver/body各channel走真实adapter；签名bytes与native主体不改。对未实际触发resolver的正常场景如实报告零；另用不重启、body/cert延迟触发一次真实request/response，证明该channel接通，但不声称R16跨重启修复通过。 |
| NR4 | 健康主例在固定1–10窗口产出十个physical block，五真实worker的proof、完整verified delivery、pre/post state与03执行一致；用真实转账与无效nonce/balance过滤，独立计算physical/native hashes。 |
| NR5 | 3/4不同合法signer、重复signer、错域/cid/epoch/view/subject/parent/body、Notarization冒充Final；使用实际密码学验证，不以结构fixture替代。公开CheckFINAL不依赖当前clock。 |
| NR6 | Accept5a–5d在等号与±1µs测试；NotBefore、parent/body/L1缺输入WAIT、原header/body ingress、认证延迟、同进程重复回调/重复投递保持首收；永久时间拒绝与WAIT的native callback行为分别核对。 |
| NR7 | 规定错时Enter、三早两晚、late join较高view、3/2合法候选分裂；无共同quorum时不宣布有界进展。晚进入不重置全体view或窗口，不能通过全员屏障消除时差。非健康时序例不强求仍有十个physical blocks。 |
| NR8 | Close/stop/ClockUnsafe与propose/verify/certify交错，持久Terminated先于后续签名出口，重复Enter不能复活；此前已形成的证明/body/合法历史输出保留。检验原签名缓存与相同坐标冲突拒绝。 |
| NR9 | 全Ack候选中单一非法effect、未持久receipt、stale plan、重复release和篡改grant；不出现提前Send/Final/consumer推进，已有持久义务可追溯。不以“未重启”省略root/aux/witness落盘。 |
| NR10 | seeds7/42/99，正/反启动顺序及0/2ms墙钟扰动；固定逻辑输入下完整虚拟trace和native消息帧逐字节比较。PID/wall耗时另存诊断，不能从协议trace删事件取得相等。 |

完整进程恢复R1–R18（包括新PID、incarnation变化、恢复首收/签名/原deadline、未完成事务和丢Ack恢复、R16 resolver重启修复）、normal持久格式的实际重启读回、05完整多cidparking、完整04重启条目及I1仍标**OPEN**。08/10/13三类handoff的成功终点也不由此REG-A结果替代。

运行命令在freeze后按实际二进制/runner确定；本稿不编造已存在的CLI。Rust重编译/重型仿真沿全宿主`flock /tmp/megaroation-poc-heavy.lock`、jobs=2；专属target/artifacts落数据盘。最终manifest固定候选SHA、二进制与配置hash、15个PID/身份、实际engine/channel数量、每个Enter/Close、全部公开输入和签名/证书/body、完整Ack与日志证据、seeds和观测范围。

## 9. 冻结前必须定案的有限清单

1. 总协调给出实际组合SHA及四个共享扩展的单一owner/提交次序；不再以“等待整个01”为代名词。
2. normal verifier版本、namespace与03 PoP映射，签名/见证共用配置和黄金字节；单独核对旧Fresh不变。
3. `NormalGrant/Ack`与root记录版本、admission/terminal/custody字段和持久次序；明确root内IO与外部释放的不同边界。
4. 新profile view/section/retention/byte/queue界、十五Clock和Enter schedule；真实五引擎时限校准方法与不满足前提时的分类。
5. 输入认证/首次完整收包规则及预算，WAIT/永久拒绝的真实native callback接线；公共FinalVerifier实际证书解码范围。
6. Verifier独立预期文件、真实进程矩阵和mutation输入先于Builder；lead确认NR-T/NR-A的有限PASS标签及仍OPEN列表。

上述清单冻结并通过来源复核后，可直接派发单Writer的P0→P1→P2工作；不存在必须先通过完整01才可实现这个profile的协议依赖。
