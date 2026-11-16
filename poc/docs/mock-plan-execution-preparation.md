# 功能 03 准备交付：Mock Plan、公共对象与轻量执行

日期：2026-11-15。当前状态：**IMPLEMENTATION-AUTHORIZED；M1接口先行；功能未完成**。

[dependency-plan-v2](commander/dependency-plan-v2.md)已正常合入，明确删除03等待整个01验收的统一门禁。现按[实现合同v2](contracts/task-contract-mock-plan-execution-v2.md)和[公共接口v1/M1](contracts/mock-plan-public-interface-v1.md)推进：M1先独立冻结bytes/期望并发布接口SHA，M2各模块合同与独立预期先行后编码。旧[准备合同v1](contracts/task-contract-mock-plan-execution-v1.md)、LE proposal和原始准备expected作为历史保留；它们的 `waiting-dependency=01` / `IMPLEMENTATION-BLOCKED` 已被新授权覆盖，不再阻止03独立组件实施。实际runtime/证书/ABI接线仍由各自任务验收。

M1接口提交 `0e72dc247de22afdc8d6681713b3d15f4aa05262` 已在相同SHA通过7项独立测试，状态为 **M1 INTERFACE PASS，可供02/06/09消费**。它包含 `poc_types::domain`、[冻结接口](contracts/mock-plan-public-interface-v1.md)与对应独立向量；[正式独立复核](../verification/mock-plan-execution/implementation-v1/m1-review-0e72dc2.md)记录准确命令与范围。原始run manifest位于 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/verification/m1/0e72dc247de22afdc8d6681713b3d15f4aa05262/run-manifest.json`。M1通过只释放消费者接口，不计完整03验收；M2公共类型、registry/PoP、执行与L1 fixture按新合同推进。

下文是原准备交付的历史记录，保留当时的范围和证据；“本阶段未启动Builder/Tester”等描述只指准备提交，不是当前实施授权限制。

## 工作环境与授权

本次在已领取的 managed worktree `/home/jy/.codex/worktrees/1c43/megaroation_poc`、分支 `codex/poc-mock-plan-execution` 工作。起始 HEAD 为 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，初始 tracked/untracked 状态干净；无需从其他 checkout 合并基线。代码祖先为 `36eff1c76d5c4df5d0790133c0e14719c0c73ebd`，并非已验收的01实现SHA。工作宿主为已连接的 xuelab_jump，不使用 direct 别名。

主协调模型要求和运行时状态为 `gpt-6-astra / ultra`；总协调通过本次 session 的实际 `turn_context` 核实并告知本 chat（不是仅从配置文件推断）。Docer 使用实际命名角色 `bft_spec / gpt-6-astra / xhigh`，Verifier 使用实际命名角色 `bft_verifier / gpt-6-astra / xhigh`。当前只启动这两个准备角色；没有 Builder/Tester，也没有替换指定模型。

本次采用 [regional-bft-conformance](../../.agents/skills/regional-bft-conformance/SKILL.md) 的来源→guard/output→独立预期流程。已读根 AGENTS/DESIGN、v2交接/搭建/设计、workflow、commander README及03交接。Regional BFT 只读来源 `/mnt/reth/jy/megaroation_poc/regional_BFT_sim` 的 HEAD 与 gitlink 均为 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。Commonware 使用锁定2026.10.0；v2 SHA-256为 `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`。

## 交付路径和冻结范围

| 所有者 | 产物 | 本阶段含义 |
| --- | --- | --- |
| Docer | [准备合同 v1](contracts/task-contract-mock-plan-execution-v1.md)、本文 | 冻结来源语义/准备验收，列清跨功能决议；不冻结未决canonical编码或ABI。 |
| Verifier | [独立推导与向量](../verification/mock-plan-execution/) | 先从规格推导，固定输入、语义边界及候选黄金字节/执行值；不导入实现制造expected。 |
| 功能03主协调 | 准备提交与最终交付SHA | 核对范围并保存文档/独立输入；不改 `poc/docs/commander/`。 |

未分配任何实现路径；`poc/crates/types/`、workspace/lockfiles、proto、生产Rust和子模块均不在本阶段写集合。后续实现需总协调重新给出精确路径owner。

已有依据足以固定：3×5等权、q4、registry和PoP验证要求、排班公式与pid/member/排序signer映射职责；reference_block和生产anchor分离；v10的cid语义preimage；PhysicalBlock/RegionalResult/ReplayPosition/LifecycleEvent最小语义；Begin/DEP/按序Valid/Kept、padding后的pre-root与实际native parent/post-root；逐reader finalized prefix及缺输入边界。

未定字节和算法选择单列：身份/PoP具体测试公钥、registry/config的canonical编码、hash/namespace、Plan授权来源编码、整数类型宽度和bounds、轻量账户/交易完整规则、L1来源commitment与09接口、07 replay/Lifecycle codec，以及09/12 ABI/ResumeCut。v2 §4.1 已固定协议／仿真时间为整数微秒、EVM/L1 timestamp为整数秒且显式检查取整边界；待定的是规范wire representation、转换溢出和取整规则的编码，不是改变时间单位。准备材料给出可比较候选，不让Builder默认选择。

## 下游最小接口清单

| 接口 | 消费者 | 本次固定什么／等待什么 |
| --- | --- | --- |
| Registry + `pid ↔ region/member ↔ participant signer_index` | 02路由、04签名 | 固定三层身份区别和可逆绑定；等待实际key/PoP与交换编码。 |
| `AuthorizationProvider.resolve`、`Leader`、Plan/Window/BlockRef | 04/05/07 | 固定静态公式和WAIT/NONE/REJECT；等待Plan字段与授权来源binding决议。 |
| ProgramConfig、Descriptor/cid、AnchorEvidence | 04/06/07/12 | 固定v10输入与不可变身份；等待唯一canonical bytes及09/12来源对象。 |
| PhysicalBlock、RegionalResult、`FinalVerifier.check` | 04/06/07 | 固定实际parent、完整链、历史验签无当前时钟；04负责真实proof/namespace。 |
| Metadata/DEP、Apply/Merge、native hash | 04/07/12 | 固定前后root、Kept与实际parent；等待账户/交易/hash profile和原子提交绑定。 |
| `L1Position`、逐reader finalized gateway、control coverage | 02/07/09/12 | 固定可见前缀与精确位置；等待实际receipt/fixture source和错误类型编码。 |
| ReplayPosition、LifecycleEvent | 05/07/09/12/13 | 固定typed骨架与v10语义顺序；07/09/12负责codec、真实位置和历史切面扩展。 |

## 独立向量的使用边界

Verifier的 [MPX-PREP-LE64-SHA256-v0](../verification/mock-plan-execution/proposal-codec-v0.md) 是待评审的实验编码：固定little-endian整数、长度前缀和SHA-256域，用小型Credit/Transfer/Attr执行向量展示raw Txs、Kept、pre/post roots、physical/native hashes的区别。[来源期望](../verification/mock-plan-execution/source-expectations.md)、[语义向量](../verification/mock-plan-execution/semantic-vectors.json)和[黄金字节/执行示例](../verification/mock-plan-execution/golden-vectors.json)分开保存。其bytes/hash期望不调用Rust实现；它不是批准的协议格式、Commonware实际签名bytes或Solidity ABI。编码被总协调修改时，应显式修订profile与黄金向量，而不是为匹配Builder输出重写expected。

来源语义向量覆盖schedule、window、乱序映射、同高异anchor、实际parent、状态转账/过滤、gap padding、metadata/DEP、重复纯求值、reader前缀及缺control覆盖。真实BLS/PoP与证书的3/4 quorum、错域/subject、production gateway、真实15-worker和Anvil均未执行；结构向量只能列出应有拒绝，不能称密码学通过。Verifier的[复现与结果报告](../verification/mock-plan-execution/README.md)和[独立checker](../verification/mock-plan-execution/check_vectors.py)给出实际运行命令、计数与摘要；最终准备SHA由主协调提交后在交付回复记录，避免在被提交内容内自引用自身SHA。

## 原准备阶段启动条件与限制（已被新依赖计划覆盖）

总协调须先提供01已验收且同SHA完成复核/仿真的依赖提交，再批准02/03公共接口与合同D项。然后修订为完整实现合同、固定独立expected并授权单Builder；候选提交后独立Verifier复核，同一候选再交Tester。当前不启动这两个角色。

本阶段只做文档/短脚本核对，不运行Rust构建。未来重型命令必须用宿主锁 `flock /tmp/megaroation-poc-heavy.lock`，`CARGO_BUILD_JOBS=2`，自己的target/artifacts和PID。总协调报告根分区约余11GB，已分配 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03`；首次Rust前检查并安全迁移本worktree已有 `.build` 内容，再链接到该任务专属目录，大artifact同样落此处，保留已有结果，不共用其他job target。当前不创建链接，轻量证据仍留在worktree。恢复持久状态、四类时间、controlled grant/Ack仍由01/02约束；03纯计算不能向协议输出开旁路。

准备材料完整不计accepted功能，整体I1和03实现仍开放。没有对main合并或远端推送，没有修改源checkout、Regional BFT worktree/history或总协调登记。
