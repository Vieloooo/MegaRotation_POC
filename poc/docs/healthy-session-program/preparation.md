# H1 健康 Session Program：准备记录

日期：2026-11-16。当前阶段：**正式M1分切片实施授权已到，优先交付API冻结稿；H1/NR-T/NR-A尚未通过。** `d55d39f73593676325bdc0fbb7ba3845ca93025f`撤销旧“仅准备/等待全部Q”门禁。M0已验收`35d382ac445cc8bc779f0d18fd23cb57e54680da`，已正常合入本worktree。Docer仍只写分配文档，不构建、不采样签名、不merge或commit；独立预期封存后立即由单Builder实现API切片，局部native决议不阻API。

交付：[任务合同v1](task-contract-h1-v1.md)、[接口语义v1](interfaces-h1-v1.md)、[API切片冻结稿](api-slice-v1.md)。当前writer为H1 Docer（命名`bft_spec`，Astra/xhigh）；本稿不代签独立Verifier结论。writer完成后释放四份文档，不commit。旧准备快照`f857675`与02 `e1a4357`完整历史保留。

## 1. 工作区与当前许可

| 项 | 读取结果/边界 |
| --- | --- |
| 显式workdir | `/home/jy/.codex/worktrees/0b8e/megaroation_poc`；全部shell命令显式使用它。 |
| 分支/起点 | 原`codex/poc-sim-communication`及e1a历史保留；本次指派开始HEAD=`4741489ba7a07d2a3f3f194dbd59ae3fd8d17bb9`，`git status --short`无输出，clean。 |
| 当前Docer writer路径 | `poc/docs/healthy-session-program/{task-contract-h1-v1,interfaces-h1-v1,preparation,api-slice-v1}.md`，必要api-v1附件获准但本次未新增。 |
| Docer只读 | 所有代码/proto/manifest/lock/verification及其他路径。H1 Builder正式所有权另见任务合同§6，不能将Docer只读限制误作全功能未授权。 |
| 准确实施基线 | M0=`35d382ac445cc8bc779f0d18fd23cb57e54680da`已合入当前4741489；不用浮动main，不由Docer重做集成。 |
| skill | 已读并使用 `.agents/skills/regional-bft-conformance/SKILL.md` 的来源先行、quorum/subject/domain、证据/交付分层和独立矩阵要求。已搜索 `protocol-spec-writer`，当前本机可见skill目录未发现；未声称使用不存在的skill。 |

已先读根AGENTS/DESIGN，`formalization_v10/poc_v2_xuelab_handoff_zh.md`、`poc_v2_xuelab_setup_zh.md`、`poc_design_zh_v2.md`；再读SP/SM/GFR、固定BFT DESIGN/上游Commonware；最后只读定位当前adapter。这保持来源推导先于实现解释。

## 2. 来源快照与核对

| 来源 | 准确版本/核对 |
| --- | --- |
| 最新计划 | `4a9a6b0a92a11e5778833988bb6524d929f4608e`；只读 `/mnt/reth/jy/megaroation_poc/poc/docs/milestones/{README,H1-healthy-session-program-zh,H2-session-manager-zh,H3-publication-gfr-zh,M0-foundation-merge-zh}.md`。五份实际文件均逐字节等于该提交git对象；H1文件SHA256为 `682d0ba04d4f5dcc1f51198b375d33a26f76e7ad10d1acc35d2a18a40b143714`。 |
| 正式派发 | `d55d39f73593676325bdc0fbb7ba3845ca93025f:poc/docs/milestones/M1-implementation-dispatch-20261116.md`，git对象SHA256=`4a211cd38a5db108dc3b8548a87c19e599cb3870e534e513b41e487b4c5cc8b6`。已完整只读源worktree文件，hash=`4f5a63a00ce2008057b995082f4ab570ff23a57d1ac5c27d56b80c9309593b83`；与git对象diff仅增加三方准备路径/027/c4提交一段，十项决议相同。规则仍引用准确d55。 |
| H2/H3收口输入 | 只读`git show 027ba8be9f8f64e35313f990f4f03c5d3c238a0c:poc/docs/contracts/h2-session-manager-healthy-preparation-v1.md`和`c4a944a754e6f9cfa50f57a2399b707a2555ce94:poc/docs/h3-publication-gfr-interface-proposal-v1.md`，未merge其实现；d55覆盖其旧门禁/整批等待提议。 |
| NR旧草案 | 提交 `8a5e3002dc579fedd3bc9f21cfac2c4fe90209a3`；`git show`内容SHA256 `8acf24c5529d2af2fc9d70813d72e8ea48583c516b19ec0d5c91028d63f34dd5`，匹配指派。 |
| BFT pin | 根gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`；本worktree子模块未初始化。只读 `/mnt/reth/jy/megaroation_poc/regional_BFT_sim`，其HEAD同pin，并用`git show PIN:DESIGN.md`读取；未初始化/修改子模块。 |
| Commonware | release `2026.10.0`，upstream `92a2f68a0e131c8b69de41e84aec37da6280f212`；读取 `/mnt/reth/jy/regional_BFT_sim/.tooling/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` 下对应consensus/utils release源码，核对Simplex protocol/certify/Finalization/N3f1；consensus `.cargo_vcs_info.json` 的sha1与上述upstream一致。Rust目标1.95.0，未启动编译/安装。 |
| 01安全封存 | 总协调给定checkpoint `0183b401f805e5508ae5fdb20d53457eb4149504`，WIP `f7bda20b1d52a5fc351c064c3d31bb738f6be5c3` **不得merge**。 |
| 01共享清单 | `/home/jy/.codex/worktrees/2123/megaroation_poc/poc/docs/native-recovery-checkpoint-shared-files-20261116.json`，实际SHA256 `2286ba8856e5f130d860e2968c3867ff223f3ea8dd77e4b957e4dc7057e70033`匹配指派。清单自己标明inventory only/no ownership transfer/no new validation。 |

01封存本身不证明恢复WIP通过，也不自动移交路径；**本次授权来自d55的明确移交**：scheme/witness/journal/driver/context最小normal扩展与聚焦测试、sim-transport增量admission/可靠邮箱同步，以及normal模块/二进制/proto/tests和原型声明文件。01 WIP、恢复状态机/patch/crash fixtures仍只读，不能merge。根types/根manifest及其他feature仍不可写。API切片只开其独立crate路径，不顺便改shared。

已交付M0的来源输入如下；本轮只读，不自行merge/cherry-pick：

| 组件 | 完整SHA | H1用途 |
| --- | --- | --- |
| 02 | `e1a435712ec492cae2bbff120a51ffd507eebf45` | 运输kernel当前基线。 |
| 03 | `e9c15e04bc83b335668f3ecd837367aa87344a89` | 唯一domain/registry/Plan/执行与fixture接口。 |
| 06 | `4b034dfbb6c3e1c7143208347b546daab8dfe298` | H3消费Publisher/SimEspresso/reader。 |
| 07 | `788291735ce6db3c3eb78b4594a28d730cce71cd` | H3的真实verifier接线、GFR-PUBLIC/LIFE。 |
| 16 | `5c34b400ba5245d7775a281566e21bf59708b147` | H3交付同一真实结果字节作官方本地发布/读回。 |
| 09 | `c443ae925ed549b046731c2d8ef3dd9ce0311d24` | M2备用控制/L1组件；M1不启动批次/DA/D或PoC控制链服务。 |

## 3. 从来源到当前adapter的差距

| 固定位置（当前e1a基线，除非另注） | 只读发现 | H1所需决议 |
| --- | --- | --- |
| `poc/prototypes/i1b-runtime/src/native_scheme.rs::GuardedScheme::sign` | scope/clock/sign-cache已有；round限制view1–8，超界panic | 显式normal profile/view/epoch/domain；保留旧入口及Fresh语义。 |
| `src/witness.rs::WitnessPolicy::new` | 固定旧PoP域 `I1b/storage/pop/v1`；profile/section绑定 | 03真实PoP域为 `PoC/Registry/PoP/v1`，需normal policy，不关闭验证。 |
| `src/native_step_journal.rs::StepJournal::create` | 创建绑定旧`Resolved`/worker | profile-neutral绑定+完整新Grant/Ack/root版本，保留旧recovery解析语义。 |
| `src/driver.rs::{Inner,configure_witness,spawn_session}`、`src/context.rs` | sessions map已有，但无完整candidate parking接口；witness/partition单例且first-grant前配置 | 单Clock/cid-scoped任务停车、多policy/ledger/partition、运行后Create；不是仅换new constructor。 |
| `src/native_hub.rs/native_config.rs/native_network.rs/native_application.rs` | 当前普通fixture为双worker/旧固定profile、脚本同伴与合成payload | 新normal模块接03与02，15进程/五真实engine；旧native恢复/fresh文件不被普通writer抢写。 |
| `poc/crates/sim-transport/src/config.rs/route.rs/reliable.rs` | contexts静态，Rebind仅incarnation，可靠邮箱也持有准入配置 | d55已选A：RouteKernel和每个相关ReliableMailbox同revision、同受控提交增量准入，保留旧custody/queue/counters；不预装未来cid或清空重建。 |
| 03 `domain::FinalVerifier` | trait，没有真实native默认实现 | H1实现真实历史配置/Finalization verifier，H3通过07 VerifierBinding消费。 |
| 07 `codec::PublicObject`、`api::VerificationProfile/VerifierBinding` | 已有Body/FinalProof分离与Native profile、basis能力 | H1不另造canonical；H3补真实依据、完整delivery与public-prefix接线。 |

NR旧稿已被覆盖：H1不是R0-only，原实例停车不是全留旧05，跨feature组合归merge owner；A已选且shared已由d55明确移交，普通字段由owner按切片冻结。恢复完整矩阵暂缓，不移除正常journal/sync/witness/terminal/签名守卫。

## 4. DOC01/02/03修正与API切片

DOC01：运输A不仅修改RouteKernel；所有相关ReliableMailbox准入revision、inbox/outbox/custody与receipt同受控提交同步，保留旧历史。DOC02：完整合法ReplayRecord接受为RootCommitted(actions_pending)，后继状态WAIT不延迟已知合法关闭；动作另有AppliedReceipt/ActionsApplied；durable terminal/ClockUnsafe后失败不得回滚Running。DOC03：h9→h11主例需真实来源到达后在原h11准入窗提前生产缓存；未提前生产可在合法晚slot如h14继续并验证padding，但这是单列案例，不冒充h11，二者窗口均11–20，不预知未来anchor/cid。

API切片独占`poc/crates/healthy-session-api/`，复用03对象/07 ReplayRecord及编码，无第二套公共编码/无runtime或always-true verifier。DTO和ports表达command/handle/generation、exact Ready绑定、opaque ManagerDelta/action意图、唯一H1持久owner与分层receipt、ProofObserved/完整结果/历史Native binding。只实现局部ID、shape guards、goldens与消费者编译例；Ready构造成功不证明真实执行或H1发行，H3没有Ready发行port。

Commonware uniform verify/certify与本地first-receive/time桥接经专项Spec和独立Verifier源码核对，已选择[Q-ADMIT-v1路线A](admission-bridge-v1.md)，无需新增vendor hook；真实native实现/重放仍NOT_RUN。不能把本地时间拒绝转false，也不能用sign(None)伪装可恢复暂停。API可立即实施。H3拥有必要分片/credits和小payload编码界：02 packet≤64KiB、06 message≤64KiB/block≤1MiB；H1给受控出口。Force当前暂不实施。

## 5. 执行计划与证据状态

| 阶段 | 操作 | 本轮状态 |
| --- | --- | --- |
| 来源/文档 | 显式workdir的git状态/固定git对象读取、技能/规范/adapter只读、NR/01清单hash核对、三文档准备 | 已完成读取与文档写入；不计协议验证。 |
| 独立准备 | Verifier从固定来源推导H1-00…23与M1联接预期，形成独立文件后才审实现 | 由独立Verifier负责；本稿不代签其结论。 |
| M0兼容 | 已交付35d382及其原报告 | 不因仅文档/API新增重复；本Docer未运行，历史Fresh不当15PID证据。 |
| API切片 | DTO/ports/local ID/shape/goldens/consumer编译例 | 冻结稿已写；独立预期→Builder→独立review→同SHA tests，**PLANNED / NOT_RUN**。 |
| shared/profile复验 | 最终候选SHA上的Fresh旧路径兼容与受影响独立检查；必要完整Fresh矩阵 | **PLANNED / NOT_RUN**。 |
| NR-T/NR-A | 15真实worker、五engine、边界/原实例/真实证书与受控输出 | **PLANNED / NOT_RUN**。 |
| 固定矩阵 | seeds7/42/99，正/反启动、0/2ms wall暂停；同命名候选先独立验证再simulation | **PLANNED / NOT_RUN**。 |
| M1 | Regular、SessionEnd两案例与官方同字节读回 | **PLANNED / NOT_RUN**，归后续集成；Force暂缓。 |

命令登记规则：已存在的 `python3 poc/verification/native-engine-fresh/run_verification.py`、`check.py RUN_DIR --output REPORT.json`、`verify_pairs.py PAIRS.json --output REPORT.json` 仅作为 **PLANNED / NOT_RUN 的历史runner候选**；先由lead核对新M0的参数/输出目录/资源与锁策略，再给出完整可运行命令。本轮未调用这些命令。normal runner/CLI尚不存在，本稿不编造命令或seed产物路径作为已执行事实。

未来build/heavy simulation需宿主 `flock /tmp/megaroation-poc-heavy.lock`、jobs=2、数据盘独立cargo target/artifact；完整shell使用显式workdir和准确候选SHA。一次一个重任务。原始证据至少含15PID/身份/engine清单、descriptor/Ready/Enter/view/Close六类时间、全部原生消息/证明/body/执行输入、完整grant/Ack/root/业务durability、每reader已交付前缀、签名/首收audit、命令/seeds/offsets/limits与失败未覆盖项；私钥不进入公开artifact。

准备/接口冻结不等于H1通过。第一轮三文档的路径/链接/格式检查曾通过；本轮新增API稿并修订原三文件，另做同范围文档检查。Docer只计算私有ID规范preimage/hash向量，未运行被测Rust、构建、真实签名、15PID或官方发布。API及新矩阵结果仍NOT_RUN。
