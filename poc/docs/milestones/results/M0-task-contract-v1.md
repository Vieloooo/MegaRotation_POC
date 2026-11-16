# M0 foundation integration contract v1

2026-11-16，Asia/Shanghai。Docer：命名 `bft_spec`（角色配置 GPT-6 Astra / xhigh）。状态：**FROZEN，lead 于 2026-11-16 批准；尚未验收 M0**。本合同只规定六个已交付组件与管理计划的历史保留合并、源码身份核对及兼容复验，不新增协议规则，不将组件通过升级为 M1/M2 或完整 PoC 通过。

## 1. 权威、读取顺序与范围

先读取本 checkout 的 `AGENTS.md`、`DESIGN.md`，源 checkout `/mnt/reth/jy/megaroation_poc` 的 v2 handoff/setup/design、milestones `README.md` 与 `M0-foundation-merge-zh.md`，再读取各准确组件提交中的合同、接口交付及独立报告。使用 [regional-bft-conformance](../../../../.agents/skills/regional-bft-conformance/SKILL.md) 的来源先行、证据与交付分离方法；本会话未提供 `protocol-spec-writer`，不声称使用。Docer 未读取候选实现来计算预期，未运行构建或仿真。

协议来源继续为 [v2](../../../../formalization_v10/poc_design_zh_v2.md) §§3–5、10、12–13，v10 Session Program/Manager/GFR，固定 BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` 的 DESIGN 和 Commonware 2026.10.0。源 checkout 已初始化 BFT 副本仅只读。本任务没有 Kudzu/Minimmit 规则或上游引擎变更，不能把其 n=15 benchmark 参数应用于三个独立五成员委员会。

M0 可复用基础面向后续 M1：15 个健康真实 worker、3×5、每证书至少四个不同合法 signer、10-height 固定窗口、真实区域结果经 SimEspresso/GFR 接纳，另用 16 官方本地节点发布读回同一结果。**这些是后续验收终点，不是 M0 通过条件或已有结果。** M2 才启用健康节点主动 Force。M0 不启动 PoC L1 控制链、Batcher/DA/D，不合入 01 恢复 WIP、11 或 12 的分支成果。

## 2. 精确输入与 Git 守卫

实际工作 checkout 为 `/home/jy/.codex/worktrees/440b/megaroation_poc`，分支 `codex/poc-m1-foundation`。旧 main 必须为 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`。已只读核对全部七项都是旧 main 的后代；它们之间的祖先关系如下，未列出的输入之间不存在祖先关系。

| 合并次序 | 输入 | 精确 SHA | 已包含的其他输入 |
| --- | --- | --- | --- |
| 1 | 03 | `e9c15e04bc83b335668f3ecd837367aa87344a89` | 无 |
| 2 | 02 | `e1a435712ec492cae2bbff120a51ffd507eebf45` | 无 |
| 3 | 06 | `4b034dfbb6c3e1c7143208347b546daab8dfe298` | 无 |
| 4 | 07 | `788291735ce6db3c3eb78b4594a28d730cce71cd` | 03、06 |
| 5 | 16 | `5c34b400ba5245d7775a281566e21bf59708b147` | 06 |
| 6 | 09 | `c443ae925ed549b046731c2d8ef3dd9ce0311d24` | 无 |
| 7 | 最新总协调文档 | `d5cbecb24836af80f8cd5414052043a17ca2ae17` | 含 primary plan `4a9a6b0a92a11e5778833988bb6524d929f4608e`；不含其他六项输入 |

只做正常 Git merge，保留真实父提交关系；已有祖先不重复 cherry-pick。分支起点、每次 merge 的父提交、冲突路径和选定内容须进入集成报告。最终所有七个输入都须是集成提交祖先。源 checkout 和其他 worktree 的未提交内容、分支及历史不变；不在源 checkout 编辑 commander 动态登记。

初始 primary plan 为 `4a9a6b0a92a11e5778833988bb6524d929f4608e`。冻结前总协调追加准确文档输入 `d5cbecb24836af80f8cd5414052043a17ca2ae17`，lead 明确批准升级第7项；已核对其为 primary plan 后代，差异仅 `poc/docs/commander/state.json`，没有源码变化。管理文件身份以 **d5cbecb** 为准，旧组件携带的旧 commander 状态不得覆盖它。仅对这次精确输入负责，不猜测以后浮动 HEAD。合入这些输入原本共有的旧基线不等于引入被排除分支的新 WIP；验证应比较输入闭包和当前分支尖端，而不是禁止共享祖先存在。

## 3. 所有权与允许改动

| 责任 | 所有权 |
| --- | --- |
| Lead | 接口/依赖决议、Git 合并、必要公共 manifest/lock/module 声明、集成 SHA、结果报告、最终 main CAS |
| Docer | 仅本合同文件；冻结后无源码写入 |
| 单 Builder | Lead 按冲突清单逐项交付的跨功能合并冲突；不得趁合并重写组件 |
| Verifier | 单独的 `poc/verification/m0-foundation/` 验证预期/审查报告；候选源码只读 |
| Tester | 同一已复核集成 SHA 上的获准运行；独立数据目录和指定 M0 测试报告 |

各组件保留独立 workspace。新实现若确有必要只能位于 `poc/`，但 M0 的默认目标是零功能改写。不得复制 Commonware、改变 BFT pin、替换已批准编码或设置默认通过的 `FinalVerifier`。行为差异不能隐藏在冲突解决中：保存最小差异、影响的来源规则与原 owner，修订合同/独立预期后才可实现并扩大复验范围。

每路径一个 writer。本 checkout 内只有一个源码 writer；如果确需并行源码 writer，先建立 Git baseline 并使用 managed worktrees。主与子角色均保留实际可见模型证据；配置值不冒称独立 runtime attestation。

## 4. 已交付源码与历史测试绑定

下表的历史测试 SHA 来自组件独立报告。Docer 用 `git diff TESTED DELIVERY -- PATHS` 对选定完整目录实际只读核对，均为空；03 的已验收和交付 SHA 相同。该检查仅证明历史 tested→delivery 源码身份，**集成后还须由 Verifier 重核 delivery→M0 并执行本合同 §6**。

| 组件 | 已测试 SHA | 必核完整源码路径 | 历史结论与不继承项 |
| --- | --- | --- | --- |
| 03 | `e9c15e04bc83b335668f3ecd837367aa87344a89` | `poc/crates/{types,authorization,core}`、`poc/Cargo.toml`、`poc/Cargo.lock` | 授权组件 ACCEPT；独立 32 tests，根 workspace 52 PASS / 11 ignored。ignored 不能计 PASS。PoP/registry 不等于最终证书四 signer 验收。 |
| 02 | `4fd6f115bd969b0a8f252d7933edd21c27f00a8f` | `poc/crates/sim-transport`、`poc/verification/sim-communication/kernel-v1` | 独立 46 tests，seeds 7/42/99 × forward/reverse × pause 0/2ms 共12组；native engines=0。真实 root/fsync/RPC/恢复和完整 SC 矩阵未验收。 |
| 06 | `8ba78f1ace3b383caa1c4360401c645d25907326` | `poc/crates/{sim-espresso,publisher}` | PE01–PE16 组件范围；7+8 crate tests、独立6+恢复4，6次/3对重放。真实区域证书/受控服务接线未验收。 |
| 07 | `8907ad64d882673d6ef2142defdf6bff2e208a71` | `poc/crates/viewer-gfr`、`poc/verification/viewer-gfr/independent-harness` | M1接口167、M2规则117；G1/G2 × 7/42/99 × 0/2ms 共12次/6对。真 FinalVerifier V22 和 RESUME material staging VG39 仍 OPEN。 |
| 16 | `6248072b3fd0eee1ae468f0961a4d3589291cba6` | `poc/crates/espresso-compatibility`、`poc/scenarios/espresso-compatibility` | 固定本地 API 的 G-API/G-REPLAY/G-ENV；crate8、独立Rust7、离线30。完整 PE 时序 Unsupported，QC/VID/Regional FINAL 未验证。 |
| 09 | `87daee8e9e77bbe27c337383d873a5eaf4a694ba` | `poc/contracts`、`poc/crates/l1-adapter`（场景在该 crate 中） | A01–A17 组件通过；真实受控 transport/public mapping A18、下游 A19 未验收。M0 仅保留为 M2 备用源码。 |

Docer 核对的 delivery Git tree 对象：

| 路径 | tree/blob |
| --- | --- |
| `poc/crates/types` | `acfae286745ae361f4f28d2f21f23bad2e65fbb4` |
| `poc/crates/authorization` | `56c58d4a37c1d3edbc1ef47968360f3ccf1ab8c5` |
| `poc/crates/core` | `88b5a33d7fb5fa49c2351cd8f1c179cab7e1f99c` |
| `poc/Cargo.toml` | `5bbb60489196b18cc6e0ed458f000499ccad27c6` |
| `poc/Cargo.lock` | `e1f6fbaa45f97dd4f6e6ab5266179af8cbec59ee` |
| `poc/crates/sim-transport` | `3d2ff54ca9231d2611e424599b0db73170faa9a2` |
| `poc/verification/sim-communication/kernel-v1` | `d73f7cd3e8e7ec29c91281ee2e37f5b9d6e25a9b` |
| `poc/crates/sim-espresso` | `fe55120f6d27e1995127594d9e62f507b1727a4f` |
| `poc/crates/publisher` | `51e8f394815ccbc375489917c8794eb9a57f88b4` |
| `poc/crates/viewer-gfr` | `daac3b284c3a637d34567d6c03f20558a139ff31` |
| `poc/verification/viewer-gfr/independent-harness` | `94f7462f254ae11c4d4f7ff9e7c6e7b32955ad12` |
| `poc/crates/espresso-compatibility` | `e433331648cb2742e66af425a31facca1e66673a` |
| `poc/scenarios/espresso-compatibility` | `115654c1bffc87443210355d685b765401fdcf2b` |
| `poc/contracts` | `0ede2a795e9d1dd4487ec99b0b991d7929d77d57` |
| `poc/crates/l1-adapter` | `5143a1b93fbe883bd35f62e09664d94053042fcf` |

历史报告来源按各自准确 delivery SHA 读取：`mock-plan-execution-result.md`、commander 的 `acceptance/feature03-20261116/report.md`（在计划 SHA）；`sim-transport-kernel-results.md`；`publisher-espresso-component-handoff-v1.md`；`viewer-gfr-component-handoff-v1.md`；`espresso-compatibility-handoff-v1.md`；`anvil-l1-component-delivery-v1.md`。它们链接的原始 manifests、源码/输入/日志 hash 须由 M0 证据索引保留原身份，不改标为 M0 新跑结果。

## 5. 必须保留的接口和状态边界

| 组件/下游 | 可消费接口 | 合并守卫和后续义务 |
| --- | --- | --- |
| 03 → H1/H2/H3 | `poc-types::domain`；`ValidatedRegistry`、`StaticScheduleProvider`；Apply/metadata/DEP/Merge、`FixtureFinalizer`/`FixtureReader` | 唯一公共编码/registry/PoP；pid、member、signer-index 分开。decode/结构有效不自动认证 public reference、anchor 或 FINAL；既有 I1 exports/hash/proto 兼容。 |
| 02 → H1 | `sim_transport` 的 Endpoint/Message/Envelope、RouteKernel、ReliableMailbox、ClockTracker、GrantConsumer | 纯算法不伪称实际受控执行。grant 不续 lease；terminal/signed 不复活；真实 adapter 必须完成 root Ack/custody 才释放输出。运输 ACK 不等于 publication/finality/G。 |
| 06 → H3 | PE-LOG-1 / PE-LOG-WIRE-1、PE-READER-1、PE-PUB-1 | q/θ/完整列表与各 reader 可见前缀分开；PREPARED/dispatch/ACK/Recorded 分开；q_avail 由07推导。06 block1MiB/message64KiB 与02 opaque64KiB 的真实分片接线仍待 H1/H3，不在 M0 发明编码。 |
| 07 → H2/H3 | `init`、`advance_global`、chain/responsibility；GFR-POS-1、GFR-LIFE-1、GFR-PUBLIC-1 | 控制同类按 L1 position，再 Expire、SessionEnd、Espresso；完整公共 coverage 决定推进。公开未齐与本地缺输入不同；WAIT 保留原位置且无部分提交。固定 successor window、逐 virtual height Expire。Force/RESUME 仍显式 Unsupported；Native 标签不是密码学证据。 |
| 16 → H3/M1 | EC16-NATIVE-1 / EC16-AUDIT-1；复用06 q/θ/opaque message对象 | 只承诺固定官方来源的本地可信 API。proposed_at/finalized_at virtual tick/live_at_proposal 为 Unknown；不得填造 PELG/VerifiedPrefix 或把墙钟当 E-TS/T_E/1ms 证据。M1未来提交其真实结果，本次不声称已有该结果。 |
| 09 → H4/M2 | ControlAuthority/BatchInbox ABI；显式 Tick/Submit/Mine/Finalize/Deliver/Read 与独立 L1 reader | M0 保留已验证组件；不启动 M1 控制链或批次服务。实际 receipt inclusion/finality/delivery/act 分开；typed 03 position 映射、真实受控调用仍需后续接线。 |

06/07 的 non-cryptographic fixture 必须继续明确标记；不得用 fixture通过、注册PoP或 hash一致性代替真实四 signer FINAL。16 固定来源为 release `20260918` / source `fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171`、协议0.3、API/v1；M0 不升级该来源，也不访问浮动 latest。

## 6. 冻结的 M0 复验矩阵

所有项目先由独立 Verifier 从来源冻结预期，再只读审查命名集成 SHA。Tester 只运行同一 SHA。实际命令/环境/锁/输入/output hash 都入 manifest；下表为 **PLANNED / NOT RUN**，不能作为已通过结果。

| ID | 动作/输入 | 必要期望与归类 |
| --- | --- | --- |
| M0-I01 | 七输入 ancestry；逐组件 tree/hash、根 manifest/lock、BFT pin；新旧管理文件 | 保留全部白名单历史及primary plan 4a9a6b0；排除01/11/12新WIP；管理状态来自d5cbecb；任何源码差异逐项解释并定向复验。 |
| M0-T03 | 根 `poc` workspace `cargo test --workspace --locked --offline`；03独立 M1/M2/auth/core/recipe，四个Python固定reference checker | 所有运行用例通过，忽略项独立列出；独立字节/root/Kept/reader/4096边界不变。根suite含03时不重复计数。 |
| M0-T02 | `cargo test --manifest-path poc/verification/sim-communication/kernel-v1/Cargo.toml --locked --offline -- --test-threads=1` | 独立46例及原snapshot负例/near-u64控制通过；这是运输内核，不是15真实引擎。 |
| M0-T06 | `sim-espresso`、`publisher` 两crate锁定tests；`publisher-espresso/independent-harness` 锁定tests | 日志/reader与Publisher状态、恢复故障、同q/θ、完整前缀、原子拒绝保持；fixture证据分类不变。 |
| M0-T07 | `viewer-gfr/independent-harness` 的接口checker及 `--bin m2`；G1/G2 各 seed7，pause0/2ms 四次代表性 replay | 独立规则预期通过；每场景两次完整trace逐byte相同，并经既有独立trace checker。G1有限q0–5/h20、G2 q0–7/h11；真实V22不计通过。未重跑7/42/99全矩阵时只按源码身份引用历史证据。 |
| M0-T16 | `espresso-compatibility` crate和独立harness锁定tests、离线checks；对已保存官方run02包以M0 binary重放 first/paused/requery 并运行 `check_capture.py` | 固定离线正/负输入和原始记录检查通过；三份audit/projection一致；Unknown/Unsupported保持。历史官方运行保留6248072身份；本次离线回放不能称新官方发布。 |
| M0-T09 | `l1-adapter`锁定check/tests；合约/ABI/接口身份和独立黄金向量检查 | 保留原ABI/签名域和driver接口。默认不重跑全部A01–A17现场矩阵；如果组件行为/依赖受合并影响，则重跑受影响真实场景。 |
| M0-TN | 当前集成普通原型的适用锁定编译；native healthy，seed7 × wall pause0/2ms，两次普通运行兼容对照；使用现有 `poc/verification/native-engine-fresh/check.py` 的适用检查 | 保留完整虚拟trace配对与两height/受控输出等既有断言；明确只有两个真实worker加脚本同伴。当前源与 `fec7ade1675cb60dd16a16fb28ebe81e3f76f508` 历史Fresh不同，不移植其PASS；不强行跑01完整恢复矩阵。若checker与当前源有适用差异，记录具体失败/界限，不改oracle隐藏差异。 |

07接口checker原M1含167检查（1,511截断prefix及大容量用例）；若标准调用分开 capacity，记录各命令及实际覆盖，不重复算case。现有脚本可能绑定原 job目录、候选SHA或内部锁，运行前只读核对CLI：优先使用可指定 M0 output/target 的入口，或等价底层命令并保存准确argv，禁止覆盖旧证据，禁止嵌套同一个flock。

若所有组件代码、依赖和harness与历史测试源均相同，完整02/06/07/09/16历史矩阵可作为明确范围的既有证据引用，本次仍完成上述必选兼容复验。若变更影响行为，历史结论不能覆盖该差异；补充独立预期和受影响测试后才能放行。

## 7. 运行资源、证据及最终放行

独立数据根为 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m0-foundation`，target、artifacts 和输出均在其中。仅复用公开 Cargo 下载缓存，不复制凭证，不修改全局 daemon/SSH。Rust 1.95.0 使用 `REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin`，Commonware2026.10.0，`CARGO_BUILD_JOBS=2`。每项 build/heavy simulation 单层获取 `/tmp/megaroation-poc-heavy.lock`；一时只跑一个重任务。

每次保存真实 SHA、dirty状态、命令/exit/timeout、工具版本、锁及输入hash、seed或N/A、raw stdout/stderr、trace、输出hash和范围；虚拟时间、公开q/θ、读者实际交付、本地墙钟诊断分别记录。原失败和环境不足记录不覆盖。只有准确集成提交上的独立复核加获准复验通过，才可标记 **M0 foundation integration PASS**。

报告写 `poc/docs/milestones/results/M0-*` 及 `poc/verification/m0-foundation/`。可记录一次命名代码集成SHA及后续报告提交，但必须标明哪些SHA实际运行；报告提交若改变任何被测输入/源码/依赖，重新核验相关范围，不能假称同SHA测试。

最后重新检查：main 仍等于旧值且没有被任何其他 worktree检出。守卫满足后使用旧值比较的 CAS 更新本地 main 至已验证交付提交；守卫不满足或验证失败则保留集成分支、main不动，报告实际值和原因，不force、不push、不部署。最终回传精确旧/新main、集成/测试SHA、worktree、冲突、命令/seeds、证据hash、可复用接口和未覆盖gate。

## 8. 冻结决议

现有用户范围足以决定本合同，无需新增协议选择或用户确认。Lead已在编码/冲突解决前明确批准 v1，并批准第7项升级为d5cbecb；独立 Verifier自行确认矩阵与来源边界以及现有wrapper的输出/锁约束。Lead已只读核定 native healthy 的 pause上限为2ms，因此M0-TN固定0/2ms。它们是执行选择，不授权引入01恢复WIP或缩小既定必选兼容范围。后续任何实质性修复须单列并重新冻结相应条款。
