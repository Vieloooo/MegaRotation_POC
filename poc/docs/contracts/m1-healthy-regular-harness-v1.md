# M1 healthy regular harness task contract v1

2026-11-16，Asia/Shanghai。合同标识 **M1-HARNESS-1**。状态：**局部机械 harness 合同 FROZEN**；Lead 已认可来源语义、独立 Verifier 的四项澄清及准备输入约定，授权单 Builder 实现 CLI/preflight/证据完整性 checker 外壳。**M1 尚未运行**；真实生产 ports、精确运行 schedule、协议 oracle 仍 NotReady，H1/H2/H3 的未知 API 不在本合同中代为冻结。Docer 仅写本文件，冻结后停止写入。

实际 checkout：`/home/jy/.codex/worktrees/5d7a/megaroation_poc`。读取基线 **`35d382ac445cc8bc779f0d18fd23cb57e54680da`**（M0 已通过）；实施决议按 **`d55d39f73593676325bdc0fbb7ba3845ca93025f:poc/docs/milestones/M1-implementation-dispatch-20261116.md`** 读取。决议中的授权覆盖旧准备稿的「不得实施」，但不把未知候选 SHA、待冻结 API 或组件测试视为已交付系统能力。

## 1. 来源与适用规则

已先读根 `AGENTS.md`、`DESIGN.md`、v2 handoff/setup/design，再读以下源规则和当前 adapter。使用 [regional-bft-conformance](../../../.agents/skills/regional-bft-conformance/SKILL.md) 的来源先行、证据/交付分离方法；本次技能目录及本机技能文件搜索没有找到 `protocol-spec-writer`，不声称使用。本文是 harness 合同，不修改形式化协议。

| 来源及版本 | 本合同采用的规则 |
| --- | --- |
| [v2 design](../../../formalization_v10/poc_design_zh_v2.md) §§3.2–3.3、4–6、9.1、10.1–10.2、12–13；[handoff](../../../formalization_v10/poc_v2_xuelab_handoff_zh.md)、[setup](../../../formalization_v10/poc_v2_xuelab_setup_zh.md) | 公开 schedule、3×5/q4、整数微秒、误差≤1000µs、独立 RPC 延迟、错时进入、受控 grant/Ack、固定十高度窗口、有状态轻量执行、来源/运行证据。 |
| [v10 02](../../../formalization_v10/02_session_program_zh_v0_1.md)，2026-11-09 v0.1，§§3.2、3.4–3.12、4.1–4.9 | descriptor/cid 不变；精确 Ready/Enter；首收不覆写；Build 用当前 slot；历史 FINAL 验证不重用观察者当前时间；Close/stop 单调；真实最终证明与完整链分别验证；活性从共同参与区间起算。 |
| [v10 03](../../../formalization_v10/03_session_manager_zh_v0_1.md)，v0.1，§§3.2、3.4–3.10、4.1–4.3 | Ready 必须有完整状态及可验证来源；每 worker/session 最多调度一个候选；Pause/选回保留原实例，不再次 Enter；关闭影响作用域内全部候选。 |
| [v10 04](../../../formalization_v10/04_publisher_zh_v0_1.md)，2026-11-10 v0.1，§§3.2–3.5、PUB-1–4 | 完整结果验证、稳定 result key、提交与真实收录分开；本地 Close/Pause 不停止 Publisher，Stop 需公开依据。 |
| [v10 09](../../../formalization_v10/09_finality_rules_zh_v0_1.md)，v0.1，§§3.1–3.5、4.1–4.3、4.7–4.8 | 公共全序及双日志覆盖；最早完整公开 `q_avail`；缺本地输入 WAIT；全段逐高度 Expire；SessionEnd 不补块、不移窗；同 position 状态一致。 |
| Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，`DESIGN.md` §§2–2.1、3.1、4–5 | 原生 Simplex 边界、最终证明/连续交付分离、signer/域核验及独立负例。其 n=15 benchmark quorum 不用于本任务的三个 n=5 委员会。 |
| Commonware **2026.10.0**，上游 `92a2f68a0e131c8b69de41e84aec37da6280f212`；`consensus/src/simplex/mod.rs` 的 Protocol Description / Certification / Deviations；`utils/src/faults.rs::Faults::quorum`、`N3f1::max_faults` | n=5 时 f=1、q=4；notarization/nullification 是进展证据；finalization 的真实 subject/namespace/signers 验证与正文交付分开。certify 结果必须是全体 honest 一致的 payload 判定，不能把局部 WAIT/到期直接映射为全局 false。 |

本 checkout 的子模块未初始化；只读 `/mnt/reth/jy/megaroation_poc/regional_BFT_sim` 已初始化副本，确认其 HEAD 为上述 gitlink 且 clean。`git -C` 在空子模块目录向上找到父仓库不构成子模块版本证据。读取的 BFT DESIGN SHA-256 为 `0749ff8bef9ad18b3794788d9f72c887126f4a2368dcf6adcb8587853eee3eae`；本机锁定 registry 源 `simplex/mod.rs` 为 `f2745799e26a93b55fd1aa8178d1252e3a10717eac28a1fcc1f25335bcae829d`、`faults.rs` 为 `f443c640e027d71263cb908fbb8bfd32ccf09d26bad80585f10495179fc2c1f9`。不初始化、覆盖或更改子模块。

**来源规则与适配选择。** 上表规定协议判定；M0 接口、实施决议规定已有模块边界。下文 JSON 清单、文件布局、命令、资源上限、seed/启动排列和诊断状态是 M1 harness 的执行选择，不成为线上协议编码。H1 的签名 namespace、callback 映射、ReadyToken、receipt，H2 的 ManagerDelta，H3 的公共结果组包/分片及官方 artifact 输入仍由各 owner 的版本化合同定义。本文只索引它们的原始字节和证据。

## 2. 成功终点、所有权与明确边界

M1 的主终点为真实 worker 生产的结果经 Publisher→SimEspresso→独立 GFR A/B，驱动 Manager/Program 完成 R0→R1→R2，**R2 首个合格结果进入 G**。完整路径窗口分别为 1–10、11–20、21–30；主例在 h21 入 G 后记录成功，不把 FINAL、Submitted、Recorded 或 SessionAssigned 代替终点。另有 SessionEnd 固定窗口及同一真实结果的官方发布/读回门禁。三项全部通过才可提出 M1 PASS。

所有签名来自 15 个持续存活的真实 OS worker；全局 `pid=5*region+member`、region∈0..2、member∈0..4。三个独立五成员委员会，五份独立 BLS key/PoP 和明确 pid↔排序后 signer-index 映射；实际聚合证明至少四个合法不同 signer。不是一个十五人委员会，也不是两真实 worker 加脚本票。这里的 `pid` 是协议身份，OS 进程号另命名 `os_pid`。依 v2§2.2/v10 03，每worker拥有独立Manager、本地公共reader/GFR及其pc/输入/候选状态；不能共享可变ReplayState或将未经本地验证的hub/远端选择当作本地重放。A/B可以指定其中两个独立实例或另设独立观察者，不要求新增OS进程，但必须记录其实际归属。

| owner | 允许写入/消费范围 |
| --- | --- |
| 本 Docer | 仅 `poc/docs/contracts/m1-healthy-regular-harness-v1.md`，不写代码、不 commit。 |
| Lead | 精确提交集成、必要 workspace/module 接线、版本/依赖绑定、逐路径 writer 移交和最终报告；保持各 feature 归属。 |
| 单 Builder（Lead 冻结后） | `poc/scenarios/m1-healthy-regular/` 的 launcher、输入 schema、raw evidence exporter 接线；组件 defect 返回 owner，不能在此复制协议实现。 |
| 独立 Verifier | `poc/verification/m1-healthy-regular/` 的来源预期、独立 checker、审查记录；若由 Builder 先做机械文件检查外壳，须明确逐文件移交后才能写。 |
| Tester | 新 run artifact 目录、获准的 `poc/docs/milestones/results/M1-*` 报告；只运行已复核命名集成 SHA。 |

没有授权改 commander 状态、恢复 WIP、regional_BFT_sim、其他 worktree 或服务配置。M1 不做 OS 重启、同 cid 重建、Force、RESUME、DA/D、批次、Web、staking 或 TLA。保留既有 terminal/signed/custody 持久化边界；不重启不等于可以绕过这些守卫。官方 dev-node 自带的隔离 Anvil 是16的运行依赖，不是启动 M1 PoC 控制链。

## 3. 已存在的可复用接口与待接线 ports

下表以 M0 SHA 的实际源码为准，不把旧草稿中的建议参数当作现存 CLI。

| 已存在接口 | 已读位置/调用 | 复用与缺口 |
| --- | --- | --- |
| M0 验证 runner | `poc/verification/m0-foundation/run_validation.py --candidate SHA --output NEW_DIR`；逐命令保存 argv/exit/raw stdout/stderr/hash；heavy 子命令自行套 flock | 固定 M0 数据根及7组兼容测试；不是 M1 launcher。复用证据/守卫模式，不为新文档重复整套 M0。其 native 命令是 `native-hub --scenario healthy --seed 7 --output DIR --wall-pause-ms 0|2`，仍为两真实 worker+脚本票，不改名当 M1。 |
| 03 对象/授权/执行 | `poc-types::domain`、`ValidatedRegistry`、`StaticScheduleProvider`、`FinalVerifier::check(descriptor,block,proof)`；`poc-core` 的 Apply/Merge、metadata/DEP、FixtureFinalizer/Reader | 公共对象/PoP/执行的唯一编码与实现依赖。FinalVerifier trait 本身不提供真实证书实现。M1 可使用03明确的有限 L1 fixture，必须保存真实 fixture bytes/coverage，不能复制 G 作为输入。 |
| 02 运输/原型 | `sim-transport` Endpoint/Envelope、RouteKernel/ReliableMailbox/ClockTracker；`poc/prototypes/i1b-runtime/` ControlledDriver/Clock/storage/witness | H1 提供普通十五 worker、动态 cid 准入、实际受控出口。已有内核和原型不足以凭配置标记为十五引擎。 |
| 06 发布/日志 | `Publisher::{offer,prepare_attempt,mark_dispatched,mark_service_acknowledged,confirm_recorded,apply_public_stop,apply_watermark}`；`PublicationVerifier/StateCodec`；`SimEspresso::{propose,finalize}`；`wire::{encode_finalized,decode_finalized}`；`ReaderState` | 保留 PE-LOG-WIRE-1/PE-READER-1/PE-PUB-1。prepare、dispatch、service ACK、Recorded 分别记录；每消息64KiB、每块1MiB。 |
| 07 重放/编码 | `init/advance_global/chain/responsibility`、`ReplayReads/ReplayServices`、`VerifierBinding{verifier,basis,profile}`、`VerificationProfile::Native`、`GFR-POS-1/LIFE-1/PUBLIC-1` | 真实 verifier 及 basis 由H1接入；Native 标签不证明签名通过。A/B 各自独立 ReaderState/ReplayState；完整 ReplayRecord 交 Manager，不拆成只按 position 去重的单个事件。 |
| 16 官方 wrapper | `poc/scenarios/espresso-compatibility/run-local.sh RUN_ID EXPECTED_CANDIDATE_SHA`；`capture.py --image --run --run-id --candidate --replay-binary` | 当前只提交两个硬编码 MARKERS，没有真实 M1 artifact 输入参数。H3 独占最小扩展，M1 不给现有脚本传猜测的参数，不复制启动器。wrapper 内部持 flock，不能外层再持同锁。 |
| 16 离线回放 | `espresso-compatibility replay BUNDLE READER NEW_OUTPUT [--pause-ms N]`、`compare BUNDLE READER REQUERY_READER`；`check_capture.py` | 保存 EC16-CAPTURE-1、EC16-AUDIT-1、原始 HTTP 和 first/requery；不会验证区域 FINAL，M1 独立追加真实 proof/正文核验。 |

**待接线 ports 是清单标识，不是新 Rust trait 或 RPC。** harness 将它们绑定到 owner 已交付的具体入口/类型/证据 schema，不能用空实现置 Ready。

| port_id | owner/所需材料 | 当前基线能力 |
| --- | --- | --- |
| `h1.runtime` | 正常 hub/worker 可执行文件、准确 argv、15 PID/5 engines、grant/Ack/clock/raw 输出、动态 cid、channel/endpoint、健康停止方法 | `NotReady`：M0 没有普通十五 worker 入口。 |
| `h1.session` | healthy-session-api 准确交付、真实 FinalVerifier/namespace/Accept 映射、Ready/Enter/Pause/SelectBack/terminal、持久 receipt 原始编码 | `NotReady`：不能用现有 FinalVerifier trait 或准备稿替代。 |
| `h2.manager` | 准确 Manager 接口/交付，完整 record→不可变 delta/intents→H1提交→actions applied、幂等身份及原实例引用 | `NotReady`。 |
| `h3.bridge` | 完整 Res→06、GFR公共完整性/stop、实际 bounded payload/分片、两个独立 reader、raw log/export 与 owner schema | `NotReady`。 |
| `h3.official_artifact` | 16扩展的准确 CLI/schema，实际结果字节清单/原来源绑定和官方读回映射，保留marker兼容 | `NotReady`；旧marker运行不能满足。 |
| `oracle.protocol` | 独立 Verifier 的源规则预期、原始 proof/执行/公共重放 checker、输入版本及反例；准确源码/可执行文件 hash | `NotReady`；不能以待测 GFR 的输出生成 expected。 |

接收一个小接口提交即可更新相应绑定并实现其接线，无需等三项完整矩阵。接口提交未提供实际运行能力时，该 port 仍不能标 Ready。owner 依赖未齐只阻塞相关阶段，允许继续 schema/证据校验和其他已冻结部分。

## 4. Harness CLI 与状态语义

以下是 **本合同授权新增** 的 CLI，当前基线尚不存在。实现应提供帮助与参数错误退出，不将命令示例当作运行记录。

```text
python3 poc/scenarios/m1-healthy-regular/run.py preflight --candidate FULL_SHA --input INPUT_JSON --output NEW_DIR
python3 poc/scenarios/m1-healthy-regular/run.py run       --candidate FULL_SHA --input INPUT_JSON --output NEW_DIR
python3 poc/verification/m1-healthy-regular/check.py --bundle RUN_DIR --expected EXPECTATIONS_JSON --output NEW_REPORT_JSON
python3 poc/verification/m1-healthy-regular/check.py --compare LEFT_RUN RIGHT_RUN --expected EXPECTATIONS_JSON --output NEW_REPORT_JSON
```

每次 `run` 执行 input 指定的一项 scenario/seed/order/pause，不默认隐式跑矩阵。`preflight` 只检查版本、schema、路径、能力和环境，不启动 worker/官方节点，不构建，不显示 M1 PASS。`check` 离线只读；网络、worker 启动和签名都不属于 checker 的能力。compare 比较两次独立通过的同输入运行；它不是协议检查的替代。

退出码：0 为该请求操作完成（preflight Ready 或检查/运行全部必需门禁 PASS）；1 为执行后的 FAIL；2 为无效输入、版本不符或证据损坏；3 为 **NOT_RUN / NotReady**（准确列出未接线 port/前置能力）；4 为 **INCONCLUSIVE / Infrastructure**（实际已启动后发生环境失败，保留部分证据）。被测 worker 意外退出、同cid重建、绕过Ack、或协议反例是 FAIL；启动前已知依赖缺失不得谎称运行失败，运行后的失败也不得改回 NOT_RUN。wall watchdog 截断是 INCONCLUSIVE，明确没有证明成功；虚拟/event预算内未达冻结终点是 FAIL，除非证据确认基础设施故障。

每个 gate 为 `{id,status,reason,evidence_refs}`；`status` 仅 `PASS|FAIL|NOT_RUN|INCONCLUSIVE`。`reason` 是带 `code` 的对象；`NotReady` 携 `missing_ports` 与 `missing_inputs` 数组，分别列具体port-id和JSON字段路径（可为空，至少一项非空）。`preflight.readiness` 为 `Ready|NotReady|Reject`，与协议中的 ReadyState 和 `ReplayStatus::Ready` 不同。缺项不能用 SKIP/PASS 消隐。

在尚无运行能力时合法产物示例：

```json
{"schema":"M1-RUN-1","operation":"preflight","status":"NOT_RUN","readiness":"NotReady","reason":{"code":"NotReady","missing_ports":["h1.runtime","h1.session","h2.manager","h3.bridge","h3.official_artifact","oracle.protocol"],"missing_inputs":["profile","fixtures.schedule","expectations"]},"gates":[],"artifacts":[]}
```

正式 manifest 还必须含 §5 的必需公共字段；该示例仅示意状态。`preflight` 即使 Ready，也只表示可以尝试执行，不能产生 `m1_pass=true`。准备阶段对于任何非null port binding，机械实现只核其文件/path/hash/size，不猜测尚未冻结的内部JSON schema或执行其中argv；无已审查且已实现的adapter消费该binding时，仍为NotReady。不得把任意非null文件或自报Ready当作运行能力。

## 5. 输入、版本绑定和原始证据格式

### 5.1 INPUT：M1-INPUT-1

JSON 使用 UTF-8，无重复 key；未知字段/版本拒绝。非负整数不用浮点，时间统一整数µs（wall单列ns或ms）；SHA为小写 hex。所有 artifact 为 run/input 根内的相对普通文件，拒绝绝对路径、`..`、symlink/越界与同一inventory内重复索引；不同事实可以引用同一已索引文件。`FileRef={path,sha256,bytes}`，hash指实际文件 bytes，不能指重新编码后的对象。原协议 hash 与文件 SHA-256 分开。调用者可先准备自包含input目录，将实际合同、独立预期及已交付原始inputs逐byte复制其中并计算FileRef；这不是新CLI，也不授权生成假fixture或将摘要改称原始对象。只复制公开/任务所需材料，不将私钥、凭证并入可审查证据包。

| 必需字段 | 类型/含义 |
| --- | --- |
| `schema`,`contract` | `M1-INPUT-1`；本合同的 `FileRef`（不自包含自己的hash）。 |
| `candidate`,`m0`,`dispatch` | 实际待跑40位SHA；M0=`35d382ac445cc8bc779f0d18fd23cb57e54680da`；决议SHA=`d55d39f73593676325bdc0fbb7ba3845ca93025f`。 |
| `components` | H1/H2/H3各 `{owner,delivery_sha,review:FileRef,contract:FileRef}`；尚未交付取null并产生NotReady，不填浮动HEAD。 |
| `scenario`,`seed`,`start_order`,`wall_pause_ms` | scenario枚举见§7；seed=7/42/99；order=`forward|reverse`；pause=0/2。 |
| `fixtures` | `config,registry,genesis,dep,workload,l1,schedule` 各 `FileRef|null`；准备阶段未冻结项用null，正式运行均须非null。至少一笔有效普通交易改变state，不能只有空块或永远被过滤的交易。 |
| `profile` | `FileRef|null`；owner冻结的完整运行/网络/clock/处理预算/engine配置及编码版本，不能只有参数摘要；准备阶段可null。 |
| `ports` | §3每项 `{port_id,owner,binding:FileRef|null}`；binding保存准确交付/接口版本、实际binary及sha、argv数组、输入/输出schema、lock_owner。没有绑定时只做preflight。 |
| `expectations` | Verifier预先冻结的 `FileRef|null`，来源规则/矩阵/独立oracle版本；准备阶段可null，禁止从本次待测GFR输出生成。 |
| `source_run` | 非H09为null；H09为 `{manifest:FileRef,checker_report:FileRef,public_messages:[FileRef],regional_results:[FileRef]}`，指向实际H01 seed7/forward/pause0的证据副本。保留原source bundle相对布局及文件bytes，核对来源manifest/实际结果/公共message关联；有报告不免除原始核验，不能只引用一张PASS摘要。 |
| `limits` | 本v1为 `{max_virtual_tick_us,max_events:1000000,wall_timeout_s:1800,max_raw_bytes:1073741824}`；虚拟cut取 `ts*(31)+W_exp`，由公开config精确算出并在preflight核对；config/profile未齐时`max_virtual_tick_us`为null。 |

所有字段名都必须存在；上述明确允许的null表示尚未交付的事实，preflight应完成可做的机械检查并返回NotReady及逐字段`missing_inputs`，不能把合法准备清单当格式错误，也不能补入默认/占位运行fixture。`run`必须在启动任何进程前要求components、所有fixtures/profile/expectations、limits的cut和本操作必需ports均已实证绑定且非null；不满足则NOT_RUN。H01/H02/H03运行需要`h1.runtime,h1.session,h2.manager,h3.bridge,oracle.protocol`，H09另需`h3.official_artifact`及非null source_run（准备H09时也可null并明确NotReady）；H09不再次启动source-run的workers。是否有公开source expectations与是否已有独立可执行oracle分别检查，前者存在不自动使`oracle.protocol` Ready。当前Builder仅做CLI/preflight/机械checker时，实际生产port和oracle绑定继续为空。

`schedule` 固定外部消息/reader交付与工作完成事件及稳定tie-break，记录seed到 schedule 的生成算法版本；不能让 OS 启动先后决定随机流。`profile`必须给 `T_sync/T_fin/T_next/T_vc` 的实际配置和来源，未校准时只报告有界功能进展、不宣称满足其理论上界。其余可采用v2§12.1初值，但实际采用值必须已显式冻结，禁止靠未写出的默认值。fixture必须覆盖虚拟cut所需的L1/metadata/DEP；有限fixture耗尽不能当作合法SessionEnd。

**M1-EXPECTATIONS-1** 是`--expected`及input.expectations所指JSON的最小封装，只有字段 `{schema:"M1-EXPECTATIONS-1",contract:FileRef,source_expectations:FileRef,oracle_binding:FileRef|null,required_gates:[string]}`。`source_expectations`为独立Verifier从来源预先写成的文档原字节；`oracle_binding`初始为null，非null时仍须由已审查adapter消费，不能执行猜测的内部schema。它不新增公共协议编码或伪造具体expected向量。调用者可用Python把真实合同/独立预期逐byte复制到自包含目录并写此封装，不需新增CLI。

`required_gates`使用§6固定ID：H01/H02/H03必须为 `BundleIntegrity,ProcessAndCustody,RegionalFinalAndDelivery,PublicReplayAndLifecycle`；H09另含 `OfficialSameBytes`，其前四项检查source-run原始证据。拒绝重复、未知或删减必需ID。矩阵整体M1放行还须§7规定的必跑case/对照全部完成；单run PASS不能代表M1 PASS。

### 5.2 RUN：M1-RUN-1

所有操作输出全新目录；进程启动前保存初始 manifest，失败/超时也增量保存，不覆盖原尝试。必需字段为 `schema,operation,contract_sha256,input:FileRef,candidate,revision_before,revision_after,worktree,components,source_files,tool_versions,commands,ports,scenario,seed,start_order,wall_pause_ms,limits,status,readiness,reason,gates,artifacts,cutoff,pending`。readiness供preflight使用，其他操作为null。未开始阶段/未知after值可为null，须说明；null不表示零。

`commands[]` 逐条记录 `{argv,cwd,binary_sha256,started_wall_ns,elapsed_wall_ns,exit_code,timeout,lock_owner,stdout:FileRef,stderr:FileRef}`；argv为数组、不经shell拼接。`source_files[]` 覆盖所有被测source/lock/proto、harness、oracle、contract和fixtures。`artifacts[]` 为带 `role,producer,schema,FileRef` 的索引；manifest不索引自身hash，完成后另生成 `manifest.sha256`。结论不可只给一个 trace digest。

`cutoff` 同时给最后虚拟tick、event-id、A/B各pc、完成/截断原因；`pending`列出未交付proof/body/结果、未完成Publisher attempt、Manager actions、reader缺项。endpoint和内部Rust对象不能以未经定义的 `Debug` 文本代替原始可解析证据。

checker新报告为 **M1-CHECK-1**，字段 `{schema,operation,bundles,expected,checker,commands,gates,status,reason}`；operation为`check|compare`，bundles为所查manifest的FileRef数组，expected为M1-EXPECTATIONS-1的FileRef，checker记录实际源码/可执行文件hash，commands与gates沿用本合同结构。跨根只读输入在报告中另记录其已解析根，不能以报告FileRef反向授权任意文件读取。每gate的reason/evidence_refs指明实际执行的oracle、原始输入与derived expected/observed产物；命名可解析文件不能代替执行该oracle。

### 5.3 原始证据最小集合

下表的文件角色是harness索引名称；实际owner原始文件名/codec保留在binding中。JSON摘要只帮助定位，事实以对应原始数据及独立复核为准。

| role | 原始来源及必须可恢复的事实 |
| --- | --- |
| `processes` | supervisor实际spawn/reap记录：protocol pid、region/member、os_pid、`/proc` start ticks、parent PID、exe hash、argv、incarnation=1、启动/退出时间、持续存活检查。15进程全程无重启；清理退出发生在观察cut之后。 |
| `controlled_execution` | H1 hub的全部Grant/Ack frames、root/aux提交与custody/witness文件；event-id、worker、固定tick、输入refs、完整输出refs及持久receipt。每个网络/计时/签名/服务输出必须回溯已提交Ack，孤立raw日志不证明受控输出。 |
| `clock` | 实际Sample请求/交付、sequence/lease、ClockView、timer/work-start/work-complete；每次读钟/签名的τ与C_m、租约判据及RPC延迟。wall耗时不进入τ、C_m或θ。 |
| `instances` | 每worker/cid的Create/Ready/Enter/view-start/Pause/SelectBack/terminal原始观察，唯一实例身份、Ready来源/完整状态、原绝对deadline、签名历史引用及关闭持久事实。process start不算Enter。 |
| `votes_and_proofs` | worker实际签名subject/签名字节、域/namespace、view/round、委员会映射；原生最终证书完整bytes及其来源worker/输出receipt。不能只保存signer数量或`verified=true`。 |
| `regional_results` | descriptor、目标physical block、从anchor到目标的完整bodies、FINAL、AnchorEvidence和状态/执行输入的规范bytes；result key、文件sha、协议hash、raw-output来源。目标及祖先都可重算；progress证书不替代FINAL。 |
| `publisher` | 原始完整Offer/verification结果、持久task/attempt、prepare/dispatch/service-ack/Recorded、Stop依据；绑定key、真实message bytes、q/index和受控出口。 |
| `public_logs` | SimEspresso每个q的PE-LOG-WIRE-1 finalized bytes（含完整列表及空块）；proposed/finalized tick、θ和source-id；L1 fixture原始headers/完整空control覆盖/metadata/DEP输入及其真实交付。 |
| `readers` | 每worker及A/B的独立reader_id/归属、header/message接收记录、原始输入bytes、首次received tick和连续前缀；不是拷贝A快照给B。 |
| `gfr_observed` | 每worker本地GFR及A/B各pc的实际输出：完整G及src/Tail/selected/closed/Rec/lifecycle，按07现存编码保存；关联该worker输入→record→Manager receipt。这些是待比对actual，不能作为预期来源或未经验证的远端选择。 |
| `manager_commits` | 完整ReplayRecord输入、每事件GFR-LIFE-1身份、H2 delta/intents、H1持久receipt和后续action结果；区分Committed(actions_pending)与ActionsApplied；cursor不冒充动作全部完成。 |
| `official` | 单列16来源身份、OCI/binary、launch配置、原始提交request/ACK/读回block/leaf/payload/transaction与first/requery包；指回本run实际public message bytes和result/proof集合。禁止用marker或重新生成等价fixture顶替。 |

可另产 `events.jsonl` 方便查找，行字段为 `{schema:"M1-EVENT-1",seq,producer,kind,virtual_tick_us,local_time_us,public_position,worker_pid,cid,refs}`，不适用坐标为null。`refs`必须落到上表raw证据；kind沿用绑定owner词汇，不作为本合同新建协议enum。排序`seq`仅是采集索引，不代替公共ReplayPosition。

## 6. 启动守卫和独立 checker

运行前核查clean命名candidate、精确components祖先关系/源码hash、M0和决议来源、BFT pin、Rust1.95.0/Commonware2026.10.0、独立预期先于测试实现冻结的证据。新输出目录不得存在；拒绝越界路径/超额输入。保存原始Git状态，不清理其他人文件以满足clean。非正式dirty诊断不能标正式M1 PASS。

15个worker各只获得自身测试私钥；launcher可以准备预置key文件，Hub/脚本不能签票。不用全员Ready barrier才统一Enter。协议run内所有组件输出经过H1受控端口；官方16进程另跑、另计墙钟，绝不注入SimEspresso主trace。

重构建/仿真只由一个owner持 `/tmp/megaroation-poc-heavy.lock`，`CARGO_BUILD_JOBS=2`，Rust构建`--locked --offline`。wrapper自行持锁时外层不持锁。data/target/artifacts在本任务数据盘独立目录，record其解析后的绝对路径；环境/依赖不足如实NotReady，不复制凭证或启动未知服务。supervisor仅清理本次child PID及start-ticks匹配的进程，禁用广泛kill。信号/异常也保留manifest/stdout/stderr/已有raw文件。

独立checker按如下顺序判定，各层独立报告：

1. **BundleIntegrity**：schema、hash/size/path、候选/工具/输入/依赖身份、完整索引、原始文件存在、cut和未完事项。通过这一层仅代表证据包完整。
2. **ProcessAndCustody**：真实15进程、每所属worker真实引擎与签名出口、incarnation=1、无脚本签名/同cid重建、每个可观察输出的Grant→durableAck→custody；lease/clock/首收/stop证据。不能信任worker自报`real=true`。
3. **RegionalFinalAndDelivery**：从真实registry/PoP、config/descriptor和独立expected profile重建签名域，固定Commonware源码验证原生certificate，验证≥4不同所属signer；完整body/ancestry/slot/metadata/执行另核。proof observation、应用验证/连续交付分别计时和计数，pending proof不计delivery。
4. **PublicReplayAndLifecycle**：从原始L1/PE日志、config/DEP/workload独立推导全部可完成positions和G/src/Tail/lifecycle，对比被测A/B；核对最早`q_avail`、公共缺正文与本地缺消息、截止严格不等号、全段Expire及固定window。完整生命周期批次和Manager持久动作另核。
5. **OfficialSameBytes**：原主场景结果+GFR-PUBLIC-1实际消息bytes与官方request/readback payload逐byte/hash相等；同result的真实区域proof/链重新验证。官方source/API可信profile单独结论。

Verifier可复用已核对的03 canonical/密码学原语，但不能调用被测GFR `advance_global` 再将其输出写成expected；独立执行expected亦不得导入被测GFR/Merge转移计算。仅A/B运行同一buggy代码且彼此相等也不足。独立参考规则/向量必须在候选审查前确定。代码重用范围和独立性限制明确写入报告；缺独立crypto/执行/replay核验入口则对应gate NotReady，不以`verified`标签补齐。

**汇总规则固定。** 输入manifest中的status/gates、ports Ready及checker_report都是待核验声明；checker只有在对应已审查oracle实际执行、返回可核对结果且raw支持时，才能自产该gate PASS。完整目录+自报PASS/verified/Ready不能通过协议gate。未实现oracle时，即使BundleIntegrity PASS，协议层与总结果仍NOT_RUN/NotReady；已具备oracle却缺少该层必需raw为无效证据（exit2），不能降级为摘要检查。任何真实FAIL或损坏证据优先报告，不得被其他NotReady掩盖；只有全部适用required_gates均实际PASS时本次check才PASS。compare也先确认两侧协议checks实际完成，否则NOT_RUN，不能只比两个相同假摘要。

compare只规范化明确非协议的字段：os_pid/start-ticks、wall时间、绝对输出路径、run-id命名包装。不得去掉clock/Grant顺序、message身份/bytes、signed bytes、公开position、G、生命周期、持久义务或尚未完成项来得到相等。同scenario/seed/start_order的pause0/2比较完整规范虚拟trace和关键raw字节/状态；正反进程启动顺序仅改变OS启动次序，协议登记/虚拟计划按pid确定，因此须有同样的规范投影。若改了虚拟Enter顺序，那是另一份schedule，只比较共同pc的来源规则结果，不能称同计划重放。

## 7. 冻结验收 cases 与有界矩阵

下表全部为 **PLANNED / NOT_RUN**。每例均由触发、前置/守卫和输出构成；没有运行日志不能改为PASS。外部Publication延迟是场景输入，不改BFT健康性，也不冒称满足正常及时发布的PUB-4前提。

| ID | 触发、前置与守卫 | 必需输出/判定 |
| --- | --- | --- |
| H01 `full-window` | 15存活；三个真实五引擎委员会随职责工作；每节点独立clock/Ready/Enter；真实小交易；R0的h10、R1的h20均在公共截止前完整公开并接纳 | R0→R1→R2；窗口1–10→11–20→21–30；anchor分别genesis/h10/h20；R2真实h21进入G；A/B同pc完整公共状态相同。每region至少两个不同Enter时刻，某节点已开始view时同cid另一成员尚未Enter；不能用barrier掩盖。 |
| H02 `session-end-preproduced` | R0 h9先合格入G；R1基于h9准备候选A，在合法原slot真实产出h11并取得proof/body，公共对象可在获选前缓存；将R0合法h10的跨区转发延后到A的h11产出之后，使更高anchor10候选B Ready并被本地调度；h10的完整公开延后到前驱截止或之后；完整双日志覆盖截止 | 真SessionEnd关闭R0并选择anchor9的A，此事件后G仍止于h9；A原实例由Paused选回、Enter总数1、保留deadline/签名历史；B被关闭/排除。h11原`q_avail`不改写，只有之后的合法Espresso事件才接纳padding10和physical11（可同键、排在SessionEnd之后），当前接纳键对两个新增height均须未Expire；后继window始终11–20。继续到R2首合格结果入G。h11未真实提前产出或未实际选回则本例FAIL，不能用H03改名替代。 |
| H03 `session-end-late-slot` | 同样使h9为前驱最后接纳末端，但后继没有缓存h11；在公共选择/Ready后按当时合法slot（如h14）真实生产 | 只追加来源规则允许的padding10..13与physical14；window仍11–20；没有倒填h11或first_received。单列补充例，不代替H02；h14仅为示例，实际height由冻结schedule与slot函数独立推导。 |
| H04 生命周期/时间 | 复用H02候选切换；重复完整ReplayRecord/Enter请求；选择时暂缺state material | 已知合法close/Selected+Preparing/cursor/intents持久接受；缺state不提前Enter；重复无新实例；actions_pending不伪装全部完成；终止或ClockUnsafe不回滚。对尚未发生的故障不宣称覆盖。 |
| H05 quorum/域负例 | 从真实proof结构派生离线反例：3 distinct、重复signer伪装4、外区key、wrong cid/view/kind、subject或目标body变异、malformed/truncated；恰好4与5 distinct合法proof作正向边界对照 | 独立验证拒绝全部不合法proof；notarization/nullification不得作为FINAL。反例生成不要求从截断aggregate伪造一个有效3签名；可用owner冻结的独立正/负向向量，须明示不属于在线worker实际产物。 |
| H06 完整性/重放负例 | FINAL先于正文；仅hash引用/本地cache，public日志缺physical ancestor；已有public object但B延迟读取；缺L1覆盖/验证材料；截止等号及同键SessionEnd/Expire | proof观察不代表delivery或q_avail；public缺正文暂不合格；本地缺输入在原pc WAIT无部分提交；coverage前不交接；等号不接纳；按控制→Expire→SessionEnd→Espresso排序；整段任何height到期则拒绝整段。 |
| H07 checker拒绝 | 对已验证包复制后分别篡改proof/body、registry/domain、G输出/生命周期、reader位置、少一worker、未授权outbound、expected来源声明、raw hash/路径、官方payload；准备阶段另构造hash自洽但仅填`PASS/verified/Ready`的机械假PASS包 | 对应gate必须FAIL/invalid evidence，不接受删掉失败行后的摘要；原始包保持不变。机械假PASS包在缺协议oracle时必须总NOT_RUN，证明布尔claim不形成PASS；未取得真包前只可测试机械schema/hash/能力拒绝，不能称协议负例通过。 |
| H08 replay | 同输入/seed/order的wall pause0与2ms；相同协议schedule的forward/reverse OS启动 | 规范虚拟trace和受控/公共/持久事实配对一致；实际worker进程号和wall诊断单列。不同seed不强求逐byte相等，但每个run都独立验证规则，A/B同pc一致。 |
| H09 `official-same-bytes` | 选H01 seed7/forward/pause0真实run中证明R2首结果入G所需的实际GFR-PUBLIC-1消息bytes集合（含其必需目标proof/body消息），交H3的16 artifact入口 | 真实官方submit及完整读回每条相同bytes/hash，保留原结果key/来源链；first/requery一致，区域proof重验。仅marker、仅HTTP ACK、仅transaction hash或重新编码的替代payload不通过。 |

**必跑系统矩阵：H01、H02各 seeds7/42/99 × forward/reverse × pause0/2ms = 24 runs / 12 pause pairs**，另比较相同seed/pause的正反启动顺序。H02至少一个明确worker的A→B→A有完整原实例证据；其他成员的调度不强迫同步，但最终产出必须符合每cid quorum与准备条件。**H03明确为非必需补充诊断**；若运行，固定seed7/forward/pause0一例，单列结果，不计入M1必需gate、不替代H02。H04取系统trace及owner独立向量对应覆盖；H05–H07离线反例各一套，H08为必需重放对照，H09一次官方运行即可，不要求对24个run重复官方发布。

共同有限边界：最多1,000,000受控events、虚拟cut=`ts*(31)+W_exp`、wall watchdog1800s/run、raw bundle≤1GiB；触界保留原因和部分证据，不扩界后覆盖同一次run。H01以h21成功为观察终点；H02以R2首个合格结果入G并完成选回/关闭义务为终点。成功后受控drain到证据完整；不为清队列推进到新的未声明协议区间。最终检查明确cut时pending及不再需要执行的旧任务，不能丢弃使完整性看似通过。

官方单列沿用16固定source `fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171` / release20260918 / protocol0.3 / APIv1 / namespace160016；image与二进制pin按16现存verify记录核查。原wrapper最大250s、capture height<128、raw bundle≤256MiB及readback界限保留，H3若调整必须显式版本化，不静默扩界。官方proposal/finalization virtual tick与live_at_proposal继续 **Unknown**；PE timing/export继续 **Unsupported**；QC/VID不在可信本地API验收范围。M1新增区域proof核验不升级这些官方保证。

## 8. 未冻结项、增量落地与最终放行

| 未闭合项 | owner / 解除门禁所需交付 |
| --- | --- |
| H1 API、真实15 worker启动/终止CLI及输出schema；动态cid、ReadyToken/receipt、namespace/本地Accept到Commonware callback映射 | H1准确接口/实现SHA、合同及独立向量/审查。尤其WAIT/local stop不能默认为certify(true)或(false)来跑happy path。 |
| ManagerDelta/action持久提交与完整record幂等、Paused原实例绑定 | H1+H2各自边界的可消费SHA/消费例子及失败语义，M1仅接线。 |
| 完整public object/result/attempt大小、跨层transport分片/credits、实际raw exporter、public Stop共同helper | H3合同/精确编码大小和上界、实现SHA及独立边界测试；02≤64KiB/06≤64KiB消息和≤1MiB块保持。 |
| 16 artifact输入参数/清单、同bytes回查 | H3最小扩展的准确SHA/CLI/schema与审查；本合同不发明旧runner不支持的参数。 |
| H01/H02完整虚拟schedule、微秒预算、真实workload/key fixture、有限L1覆盖、迟到后继实际slot | 集成Lead基于已交付ports填M1-INPUT-1；Verifier在执行前校验可行性和独立预期。h11缓存证据由真实运行产生，不可在schedule注入伪造block。 |
| 独立proof/execution/GFR可执行oracle | Verifier已独立形成来源预期；具体oracle绑定待其版本化交付，null不影响局部机械外壳实施。本文局部合同已获Lead冻结，最终hash由交付消息记录。 |

本次冻结足以先实现：CLI/schema/版本守卫、只读preflight、NotReady传播、完整commands/raw索引、fresh-output/进程清理框架、离线机械检查与反例入口。生产binding内部schema及实际执行仍等对应owner，当前Builder不实现猜测的binding解析/argv执行。禁止为使这些外壳返回PASS而制造十五worker/finality/publication/G。接线ports未到时交付物可以是 **harness preparation complete / M1 NOT_RUN**，不是M1完成。

只有同一准确集成SHA已独立复核、必跑系统矩阵及所有必需gates通过、官方相同bytes通过、证据hash封存，才能报告M1 PASS；每个实际执行SHA与报告/交付SHA分别注明。H3后续仅官方adapter版本变化时按影响复核，不重写历史运行身份。合同后续变动采用新hash和明确变更记录，不用运行结果回填或修改旧expected。

最终交付包含完整candidate/组件/合同/期望hash、准确命令与工具、24次系统run及配对清单、原始15PID/签名/证书/body/公共日志/manager证据、离线checker结果、官方包、失败尝试及限制。I1完整范围、原恢复/故障矩阵、官方PE时序、M2 Force及DA/D/RESUME保持原状态，不因本合同或M1有限健康矩阵自动关闭。
