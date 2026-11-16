# H1 健康 Session Program：准备记录

日期：2026-11-16。结论：**文档/接口/验收准备完成候选；H1 未实现，NR-T/NR-A 与所有新矩阵 NOT_RUN。** 当前三份文档待lead与独立Verifier复核，尚未冻结编码合同。不构建、不运行重型测试、不采样新签名、不merge其他功能、不创建新用户chat。

交付：[任务合同v1](task-contract-h1-v1.md)、[三组接口与Q决议v1](interfaces-h1-v1.md)。当前writer为H1 Docer（命名 `bft_spec`，Astra/xhigh）；独立Verifier只读准备，不把本稿写成其独立结论。writer完成后释放路径，不commit。

## 1. 工作区与当前许可

| 项 | 读取结果/边界 |
| --- | --- |
| 显式workdir | `/home/jy/.codex/worktrees/0b8e/megaroation_poc`；全部shell命令显式使用它。 |
| 分支/准备起点 | `codex/poc-sim-communication`；`e1a435712ec492cae2bbff120a51ffd507eebf45`。开始时`git status --short`无输出，clean。保留原02worktree及完整历史。 |
| 当前唯一writer路径 | `poc/docs/healthy-session-program/task-contract-h1-v1.md`、`interfaces-h1-v1.md`、`preparation.md`。 |
| 当前只读 | 生产代码/proto/shared shim/root manifest/lock、verification、其他文档、01 checkpoint、所有组件源、子模块。 |
| future实现基线 | **M0尚未交付准确验证SHA，不填值。** 正常合并M0由lead在获交付后进行；Docer不合并。 |
| skill | 已读并使用 `.agents/skills/regional-bft-conformance/SKILL.md` 的来源先行、quorum/subject/domain、证据/交付分层和独立矩阵要求。已搜索 `protocol-spec-writer`，当前本机可见skill目录未发现；未声称使用不存在的skill。 |

已先读根AGENTS/DESIGN，`formalization_v10/poc_v2_xuelab_handoff_zh.md`、`poc_v2_xuelab_setup_zh.md`、`poc_design_zh_v2.md`；再读SP/SM/GFR、固定BFT DESIGN/上游Commonware；最后只读定位当前adapter。这保持来源推导先于实现解释。

## 2. 来源快照与核对

| 来源 | 准确版本/核对 |
| --- | --- |
| 最新计划 | `4a9a6b0a92a11e5778833988bb6524d929f4608e`；只读 `/mnt/reth/jy/megaroation_poc/poc/docs/milestones/{README,H1-healthy-session-program-zh,H2-session-manager-zh,H3-publication-gfr-zh,M0-foundation-merge-zh}.md`。五份实际文件均逐字节等于该提交git对象；H1文件SHA256为 `682d0ba04d4f5dcc1f51198b375d33a26f76e7ad10d1acc35d2a18a40b143714`。 |
| NR旧草案 | 提交 `8a5e3002dc579fedd3bc9f21cfac2c4fe90209a3`；`git show`内容SHA256 `8acf24c5529d2af2fc9d70813d72e8ea48583c516b19ec0d5c91028d63f34dd5`，匹配指派。 |
| BFT pin | 根gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`；本worktree子模块未初始化。只读 `/mnt/reth/jy/megaroation_poc/regional_BFT_sim`，其HEAD同pin，并用`git show PIN:DESIGN.md`读取；未初始化/修改子模块。 |
| Commonware | release `2026.10.0`，upstream `92a2f68a0e131c8b69de41e84aec37da6280f212`；读取 `/mnt/reth/jy/regional_BFT_sim/.tooling/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` 下对应consensus/utils release源码，核对Simplex protocol/certify/Finalization/N3f1；consensus `.cargo_vcs_info.json` 的sha1与上述upstream一致。Rust目标1.95.0，未启动编译/安装。 |
| 01安全封存 | 总协调给定checkpoint `0183b401f805e5508ae5fdb20d53457eb4149504`，WIP `f7bda20b1d52a5fc351c064c3d31bb738f6be5c3` **不得merge**。 |
| 01共享清单 | `/home/jy/.codex/worktrees/2123/megaroation_poc/poc/docs/native-recovery-checkpoint-shared-files-20261116.json`，实际SHA256 `2286ba8856e5f130d860e2968c3867ff223f3ea8dd77e4b957e4dc7057e70033`匹配指派。清单自己标明inventory only/no ownership transfer/no new validation。 |

01“已安全停止/命名保存”说明不再并发写入与证据已保存；**不说明恢复WIP验收通过、不授权H1改shared、不把该checkpoint变成M0输入**。normal scheme/witness/journal/driver/context扩展仍须总协调明确移交精确路径；原恢复状态机/patch/crash fixture只读。清单的hash相同不能推出组合验证通过。

M0 正在组合的准确输入如下；本轮只读，不自行merge/cherry-pick：

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
| `poc/crates/sim-transport/src/config.rs/route.rs` | contexts静态，Rebind仅incarnation，snapshot/commit要求原config hash | 推荐`SIMT-CONTEXT-ADMISSION-1`增量准入；备选固定carrier+normal内层准入，必须明确选边界，禁止预装未来cid或空kernel替换。 |
| 03 `domain::FinalVerifier` | trait，没有真实native默认实现 | H1实现真实历史配置/Finalization verifier，H3通过07 VerifierBinding消费。 |
| 07 `codec::PublicObject`、`api::VerificationProfile/VerifierBinding` | 已有Body/FinalProof分离与Native profile、basis能力 | H1不另造canonical；H3补真实依据、完整delivery与public-prefix接线。 |

NR旧稿中需被最新计划覆盖的具体内容：单cid/R0-only并非H1最终接口；停车/原实例保留不能全留旧05；本feature不自行组合02/03实现；动态transport权限不能推迟成“另一个与H1无关的未来问题”；01已封存但shared移交依然具体、串行。恢复完整矩阵退出近期关键路径，不移除正常journal/sync/witness/terminal/签名守卫。

## 4. M0与下游必须交付的有限清单

M0交付准确集成SHA、dirty状态、组件source-tree及候选/测试绑定、冲突diff、Cargo/patch唯一来源、Fresh与旧基线差异、必要兼容检查结果/命令与未覆盖gate。历史 `fec7ade1675cb60dd16a16fb28ebe81e3f76f508` 的66次/33对Fresh仅为两个真实worker和脚本同伴，不是M0同源码或15健康节点证据。准确M0上必须先跑批准的Fresh兼容检查，shared改动后再复验受影响旧路径。

总协调交付每项shared扩展的owner/allowlist/顺序，包括scheme/witness/journal与若采用的driver/context/transport最小diff；无需等待整个01，但不能从封存推导默认转让。H2/H3应回复接口提案中的Q-LIFE/Q-PREP/Q-RUNTIME/Q-RESULT相关细节；真实proof/basis适配不新增公共trait。H3另确认StopDecision公开position来源与1MiB/64KiB分片/完整前缀边界。

H2需在冻结中确认SessionEnd变体：h10迟公开时，h9候选可在h11准入窗口提前准备/生产缓存；后继始终11–20。H1应支持真实候选停车/恢复与未来cid安装；不能等公开SessionEnd才创建所有后继状态。选择事实来自真实GFR/public provenance，Hub只调度输入，不充当选择oracle。

## 5. 执行计划与证据状态

| 阶段 | 操作 | 本轮状态 |
| --- | --- | --- |
| 来源/文档 | 显式workdir的git状态/固定git对象读取、技能/规范/adapter只读、NR/01清单hash核对、三文档准备 | 已完成读取与文档写入；不计协议验证。 |
| 独立准备 | Verifier从固定来源推导H1-00…23与M1联接预期，形成独立文件后才审实现 | 由独立Verifier负责；本稿不代签其结论。 |
| M0兼容 | 准确M0的Fresh代表性矩阵+runtime/storage/witness受影响检查 | **PLANNED / NOT_RUN**，M0 SHA待交付。 |
| shared/profile复验 | 最终候选SHA上的Fresh旧路径兼容与受影响独立检查；必要完整Fresh矩阵 | **PLANNED / NOT_RUN**。 |
| NR-T/NR-A | 15真实worker、五engine、边界/原实例/真实证书与受控输出 | **PLANNED / NOT_RUN**。 |
| 固定矩阵 | seeds7/42/99，正/反启动、0/2ms wall暂停；同命名候选先独立验证再simulation | **PLANNED / NOT_RUN**。 |
| M1/M2 | Regular与官方同字节读回；后续健康主动Force | **PLANNED / NOT_RUN**，归后续集成。 |

命令登记规则：已存在的 `python3 poc/verification/native-engine-fresh/run_verification.py`、`check.py RUN_DIR --output REPORT.json`、`verify_pairs.py PAIRS.json --output REPORT.json` 仅作为 **PLANNED / NOT_RUN 的历史runner候选**；先由lead核对新M0的参数/输出目录/资源与锁策略，再给出完整可运行命令。本轮未调用这些命令。normal runner/CLI尚不存在，本稿不编造命令或seed产物路径作为已执行事实。

未来build/heavy simulation需宿主 `flock /tmp/megaroation-poc-heavy.lock`、jobs=2、数据盘独立cargo target/artifact；完整shell使用显式workdir和准确候选SHA。一次一个重任务。原始证据至少含15PID/身份/engine清单、descriptor/Ready/Enter/view/Close六类时间、全部原生消息/证明/body/执行输入、完整grant/Ack/root/业务durability、每reader已交付前缀、签名/首收audit、命令/seeds/offsets/limits与失败未覆盖项；私钥不进入公开artifact。

准备完成不等于H1通过。本轮文档检查：三文件无尾随空白、Markdown fenced code成对、所有相对链接目标存在；`git diff --check`无报错（新增未跟踪文件另经逐行检查），`git status --short`只显示本目录新增文档，未修改已跟踪生产文件。所有实现测试、真实签名、15PID运行、官方发布和新完整矩阵仍未执行。
