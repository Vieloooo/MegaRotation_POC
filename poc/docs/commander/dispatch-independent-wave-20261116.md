# 01恢复期间的独立工作派发

2026-11-16，总协调授权。用户要求继续收口01，并并发推进不依赖未完成01的功能。活跃用户chat上限5：新01 + 07 + 11 + 12 + 16。旧01被新01接替；02/03/06/09当前均idle，不为此波重新启动。

## 共用启动段

你是本功能主协调，使用 GPT-6 Astra / ultra，继续既定 Docer（bft_spec，Astra/xhigh）→ Verifier（bft_verifier，Astra/xhigh，先来源独立预期）→ 单 Builder（bft_builder，Sol/high）→ Verifier复核 → Tester（bft_simulator，Sol/high）。使用实际可用角色，不静默降级。子agent不能另外创建用户chat扩张槽位。

源checkout `/mnt/reth/jy/megaroation_poc`，项目 `ccb826e2-d47a-4623-9b20-2632ac96d8e3`，已连接宿主 `xuelab_jump`。使用本chat自动创建的managed worktree，不创建第二个。核对干净起点，在本worktree正常合并下面列明的准确依赖提交；禁止reset/重写原历史。源checkout仅作只读资料来源。先读根AGENTS/DESIGN、v2交接/搭建/设计、任务handoff、五chat分层依赖v2及本文。若新worktree没有本文，从源checkout的绝对路径读取。

先冻结当前阶段合同和独立预期，再实现。以可用接口与组件提交推进，不等01全部完成，也不等待main批次合入。测试fixture和真实运行证据分别标注；测试桩不能作为密码学验证，公共规则不能读取全知调度器状态或墙钟。所有项目使用用户自己的本地PoC服务/进程/数据；外部只读取官方资料或固定公开依赖，服务运行使用本机独立端口。

你不是独自工作：一个路径一个writer，禁止修改其他功能目录、native恢复源码、Regional BFT子模块及 `poc/docs/commander/`。新功能使用下列独立crate/workspace和自己的lockfile，避免同时编辑根Cargo/types/core声明。消费03已有canonical对象，不重复定义其编码；扩展共享对象时提出精确局部变更，由总协调指定owner。主lead可依据已固定来源和独立Verifier冻结本职接口，不必等待尚未启动的每个消费者；争议只阻塞受影响部分。

根分区空间有限，各功能worktree的 `.build` 指向 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/<编号>`，安全保留现有内容；公开下载缓存可复用，自己的target/artifacts/PID。全宿主重型构建、测试、仿真仍经 `flock /tmp/megaroation-poc-heavy.lock`，默认两个build jobs，不嵌套同锁。Rust1.95.0可用 `/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin`，Commonware2026.10.0、Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` 保持。旧证据只读，新的运行使用新目录。

先发布小而完整的接口SHA和独立向量，然后持续完成本阶段实现/复核/测试。源码候选先提交，Verifier/Tester检查同一SHA；失败或修改后复验受影响部分。回传基线/接口/候选/测试SHA、实际模型/角色、worktree/分支、准确命令/seeds、证据绝对路径与摘要、PASS/FAIL/INCONCLUSIVE和剩余gate。不要把组件完成记为整个功能完成，不自行合入main或push。总协调主动读取进展，不要求你向其他chat发消息。

## 可消费依赖

| 功能来源 | 合并到自己worktree的交付SHA | 核对范围 |
| --- | --- | --- |
| 03 公共对象/排班/执行 | `e9c15e04bc83b335668f3ecd837367aa87344a89` | 最终Verifier/Tester报告PASS；总协调独立审计79项核对通过，已ACCEPT完整03授权范围。该SHA可消费；main尚未合入。 |
| 06 公共日志/Publisher | `4b034dfbb6c3e1c7143208347b546daab8dfe298` | 源码与被测试的`8ba78f1ace3b383caa1c4360401c645d25907326`一致；总协调核对49+121份证据哈希。PE-LOG-1/PE-LOG-WIRE-1/PE-READER-1可独立消费。 |
| 09 合约/L1组件 | `c443ae925ed549b046731c2d8ef3dd9ce0311d24` | 源码与`87daee8e9e77bbe27c337383d873a5eaf4a694ba`一致；总协调核对34份原始文件及4份独立结果哈希。A01–A17组件通过，A18/A19集成未通过。 |

06的官方来源验证属于16，07规则属于07，09后续G/DA/ResumeCut联调属于11/12/13；不能把这些下游责任倒置成消费现有接口的前置条件。原功能完整验收保持开放，届时补实际系统接线。

## 07：Viewer GFR纯规则与独立reader

标题 `PoC 07 Viewer GFR 与独立重放`，分支 `codex/poc-viewer-gfr`。按顺序正常合并03、06交付。读取 `poc/docs/handoffs/07-viewer-gfr-zh.md`。

独占 `poc/crates/viewer-gfr/`（独立workspace）、`poc/verification/viewer-gfr/`、本任务合同和报告。该路径替代原建议的core/gfr目录，避免与12共同编辑core声明。复用03的Plan/Merge/域对象和06的真实组件reader接口。

现在完成纯Init/AdvanceGlobal/Chain/Responsibility、公共排序、watermark/完整L1控制覆盖、WAIT原位置、q_avail与接纳位置分离、逐height Expire/padding、regular满窗/SessionEnd、HALT、checkpoint和独立A/B重放。先冻结ReplayPosition和lifecycle输出比较/去重边界，与03typed骨架兼容；Force/RESUME尚未实现的输入明确报告，不能跳过。

冻结验证能力接口和真实证书接线边界。04真实native FinalVerifier尚未交付，可以用明确标注的输入向量测试纯规则，但不能把bool桩当作真证书验收；该gate保留。生命周期在未来RESUME时材料先可用再分配的原子边界需列出决议，当前不擅改v10。该纯规则阶段不依赖01。

## 11：Batcher/Certifier/DA独立服务组件

标题 `PoC 11 批次与 DA 服务组件`，分支 `codex/poc-batch-services`。按顺序正常合并03、09交付。读取 `poc/docs/handoffs/11-batch-services-zh.md`。

独占 `poc/crates/batch-services/`（独立workspace）、`poc/verification/batch-services/`及本任务合同/报告，内部按batcher/certifier/da/accepted_view分模块。不修改09的合约/ABI/driver或根workspace。先核对09已冻结的签名/commitment/receipt语义，**优先提交供12消费的frame/DA payload编码、commitment验证和accepted-record接口**，只保留一套编码。

实现不可变DA字节持久化、真实receipt与Certifier签名、1–2个连续height的frame、crash-only持久任务/去重/恢复、prev cursor和实际Inbox successful finalized receipt读取。尽可能使用09真实本地Anvil做组件接纳测试，不能把构造的收据算真实接纳。G来源和gpos语义暂用明确受限的fixture接口，缺必要来源时WAIT；07/08实际G和12/13恢复切面仍需后续联调。

本阶段可实现/测试持久性、真实签名与合约接纳；不等待01，也不提前声称完整G′或完整11通过。尽快发布供12调用的准确接口提交及实际DA/receipt样本。

## 12：独立D派生与位置绑定切面

标题 `PoC 12 独立 D 派生与 ResumeCut`，分支 `codex/poc-derivation`。按顺序正常合并03、09交付。读取 `poc/docs/handoffs/12-derivation-zh.md` 和固定derivation来源准备稿。

独占 `poc/crates/derivation/`（独立workspace）、`poc/verification/derivation/`及本任务合同/报告，避免与07共同编辑core声明。复用03执行和09实际(block,tx,log)/CursorSet事件。

先从固定MegaETH/OP来源独立推导并冻结checkpoint/genesis、epoch/窗口/fallback、实际parent上的执行、完整input frontier、CursorSet顺序和DeriveAtResume规则。可先实现不依赖frame字节的状态机、可用输入/缺数据WAIT、窗口边界、deposit-only和精确位置切面；**11拥有batch frame和DA编码，禁止自己创造第二种**。11接口发布前显式记录该局部依赖，先完成来源/合同/无frame路径和独立期望；如果这些工作全部完成而接口仍未到，再释放活跃槽等续派。

只用实际D parent，禁止读取G或复制G结果作为D。需要更多L1高度时用09真实driver生成本地输入，不能越过03有限fixture范围编造已验证数据。真实11 payload/DA/receipts到位后再完成实际批次与fallback/同区块不同RESUME切面联调。本阶段无需01，但真实11依赖不得掩盖成已通过。

## 16：官方Espresso本地兼容

标题 `PoC 16 Espresso 本地兼容验证`，分支 `codex/poc-espresso-compatibility`。正常合并06交付。读取 `poc/docs/handoffs/16-espresso-compatibility-zh.md`。

独占 `poc/crates/espresso-compatibility/`（独立workspace）、`poc/scenarios/espresso-compatibility/`、`poc/verification/espresso-compatibility/`及本任务合同/报告。固定官方release/commit或镜像digest，查询官方API与源代码后冻结来源身份、q/theta、完整列表、submit/finality/query与06公共日志的映射。

在用户本机启动任务专属官方dev-node并做publication、读取/录制、重复查询和离线回放；只管理自己的容器/PID/端口，不连接或压测公共生产网络。缺少Docker/运行条件时先调查受支持的本地方式，记录准确环境缺项，不改系统daemon/代理或伪造官方节点结果。官方wall-clock运行与录制后的确定性重放分别报告；无法核验proposed_at/live属性时保留缺失，不伪造06的时序保证。

16只依赖06已发布的接口，不等待01/04/08/15。完成原16范围并独立复核/测试后，可提交整个16验收申请。

## 批次规则

每三个**完整accepted功能**触发独立merge-to-main chat；这四路中的组件里程碑不增加计数。接口/组件可先用于下游，无须等main。用户chat总数按活跃工作计，等待且所有子agent停止时释放槽位；新的合并chat也占一槽。当前未授权第六个活跃功能chat。
