# 功能11独立服务组件交付

2026-11-16。**本阶段组件 B01–B16：独立 Verifier 与正式 Tester 均 PASS。** 已交付唯一 frame 编码、真实签名、不可变 DA、持久 Batcher/Certifier、实际 Inbox reader 和本地 Anvil 样本。完整功能11仍开放；B17的实际12派生消费、B18的系统接线保持 INCONCLUSIVE，不增加完整 accepted 功能计数。

按最新用户调度，本任务在当前证据封存后**暂缓**，近期工作转向15健康节点regular/Espresso/GFR和健康节点Force。本次不新增实现阶段或矩阵；代码与原始artifact保留。命名checkpoint为 `codex/poc-batch-services-checkpoint-20261116`，所有子agent已完成，本次服务已退出。

## 准确版本与工作区

| 项目 | SHA / 路径 |
| --- | --- |
| 干净起点 | `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b` |
| 正常合并03 | `e9c15e04bc83b335668f3ecd837367aa87344a89` |
| 随后正常合并09 | `c443ae925ed549b046731c2d8ef3dd9ce0311d24` |
| 集成基线 | `c9805f7a818bc23d30018ce4dc6f1ee45f92e3ac` |
| r1合同／实现前独立向量 | `8fb46c296808dbe97c95df7a2761678412b95f62` |
| M1 frame/crypto/reader接口源码 | `75b574d40930b7c9c748a4c6b6e381117c937537` |
| M1独立审查与接口交付报告 | `72a08cc463f63740c32603b9e7630820760c9b1c` |
| r2来源投影澄清／独立执行检查 | `b9b8b1595ffd2a95fbdd779a843a4119ed59bba8` |
| **M2候选＝Verifier源码＝Tester源码** | **`e5f77d27edde91a61f7400d1a61961e4cbe09c9c`** |
| worktree / 分支 | `/home/jy/.codex/worktrees/f025/megaroation_poc` / `codex/poc-batch-services` |

本交付报告及验证材料提交不改变上述已验证源码。未合入 main、未 push。没有合入无关06；没有直接修改共享 types/core/根 Cargo、09、native恢复或 commander 文件。Regional BFT gitlink保持 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，Commonware保持2026.10.0。源 checkout只作资料来源，共享Git元数据仅用于授权分支上的正常合并与提交。

主协调实际 turn_context 已核实为 `gpt-6-astra / ultra`。按命名角色依次使用 `bft_spec`、`bft_verifier`（Astra/xhigh），单 `bft_builder`（GPT-6.1 Sol/high），Verifier复核，再由 `bft_simulator`（GPT-6.1 Sol/high）正式复跑。独立预期先于源码，每个路径一个writer，未另建用户chat或第二个worktree。

## 实现与消费接口

独立 workspace 位于 `poc/crates/batch-services/`，有自己的 Cargo.lock。下游可直接依赖 `batch-services = { path = "../batch-services" }`；[README](../crates/batch-services/README.md)列出精确API，[冻结r2合同](contracts/task-contract-batch-services-v1.md)与[M1接口交付](batch-services-interface-release-v1.md)给出边界。

- BS-FRAME-v1只含一个完整frame及1–2个连续height；直接使用03交易codec，保存Apply的Kept及空普通交易记录。Keccak commitment与09域下的真实secp256k1 DA/Certifier签名分别验证，gpos仍为受限fixture引用。
- DA先同步不可变bytes及目录，再签名并持久化receipt。角色目录绑定配置、身份和reader，采用单writer锁、原子journal、完整命令去重。Batcher在发送前保存raw transaction/nonce/hash；未知结果保留SubmittedUnknown，恢复时定位原finalized事件或确定失败，保留终态。
- Certifier使用独立source/reader，重新执行03 Apply；缓存命中仍检查当前prev、最近配对RESUME和DA。固定gpos可以从快照实际parent重放后投影连续小批次，无须为普通cursor推进等待新gpos。
- InboxGateway消费09真实driver的finalized、已交付前缀，闭合构造AcceptedRecord，保留实际block/tx/local-log、RPC logIndex和完整证据。缺DA在原输入WAIT；缺gpos只报告已物化部分；缺ResumeCut时保留历史前缀并报告具体缺失位置。

## 正式验证结果

[独立复核报告](../verification/batch-services/reports/m2-review-e5f77d2.md)与[正式Tester报告](../verification/batch-services/reports/simulator-e5f77d2/report.md)逐项列出B01–B18、命令、工具、独立输出和限制。每次运行均使用全宿主 `flock /tmp/megaroation-poc-heavy.lock`、两个build jobs、锁定离线依赖和新输出目录。

| 每个300/650ms运行的观测 | 结果 |
| --- | --- |
| 实际Anvil范围 | genesis至L1高度6，7个完整headers、8份receipts |
| 接纳／拒绝 | 2个原始CursorAdvanced；1个成功无新事件重试；3笔mined status0（旧prev、错误Certifier signer、错DA commitment） |
| 显式driver与raw tx | 119条命令，20次受控修改RPC，8笔原始EIP-155交易独立验签 |
| 实际进程故障 | 3个DA、2个Certifier、5个Batcher持久点 `_exit(91)`，及广播后回复前1次SIGKILL |
| 恢复终态 | 原raw bytes/hash/nonce保留；两个Accepted、一个Reverted持久保留 |
| 执行 | 3个已接纳height的Kept与4个source height的poststate/native hash独立吻合 |
| 时间 | 终点虚拟参考14,000,000μs；L1 header按2秒slot；300/650ms为墙钟注入 |

四个Rust测试及两轮场景均通过。五类原始／协议JSON逐字节相同，crash比较只排除OS分配的PID。独立reader故障重放另覆盖4个缺失WAIT、5个篡改REJECT和修复后单次推进。这些是实际收据上的故障fixture；后续GLOBAL cursor保留之前RESUME的用例也是明确标注的历史fixture。直接Set `(5,0,0)`→Resume `(5,0,1)`、seq0及旧prev回滚使用真实Anvil。

场景seed=`none (explicit script)`；M1随机frame seed=`1112753713`。正式Tester命令含锁墙钟6.32秒，不是协议延迟或活性界。Rust/Cargo1.95.0、Foundry1.0.0-stable、solc0.8.20、独立Python3.12.8/coincurve20.0.0；准确二进制hash见各manifest。自己的Anvil/角色子进程均已回收。

Verifier原始根目录：`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-verifier-e5f77d2-run1`。

**正式Tester原始根目录：** `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1`。

Lead的最终hash核对记录见[交付审计](batch-services-component-audit-v1.json)，对应[Verifier manifest](../verification/batch-services/reports/m2-manifest-e5f77d2.json)与[Tester manifest](../verification/batch-services/reports/simulator-e5f77d2/manifest.json)。开发smoke另存，正式结论只使用上述准确候选的独立运行。

## 供12消费的实际样本

使用正式目录 `pause-300/run/` 或逐字节相同的 `pause-650/run/`：

- [原始L1收据包](/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run/raw-rpc-bundle.json)：完整headers、transactions、receipts、实际配置。
- [DA索引](/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run/da-index.json)：commitment到相对不可变 `.bin` 文件的映射；同目录保留真实DA receipts。
- [接纳记录](/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run/accepted-records.json)、[driver轨迹](/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run/driver-trace.json)、[故障轨迹](/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run/crash-trace.json)用于核对位置、交付与恢复。
- `fixture-snapshot.json`保存本次公开配置、DEP和初始anchor；`execution-source.json`用于独立执行对照。12应从配置/genesis/DEP与L1/DA派生，不能复制其中G的后续状态作为D。

| L2 heights / L1位置 | Commitment（Keccak） | 字节数 / payload SHA-256 |
| --- | --- | --- |
| 1–2 / `(2,0,0)` | `3719037b49943e3c4dfdb7488fd54eb94cbfa8b849e4e8339ea5f03cc7ca3ef2` | 117 / `f84fd2986e31829e19bbad01784c5d4701d43febea0d42ed8fd3877073437e5c` |
| 3 / `(3,0,0)` | `5629543d719715cc831f03dd53c111bd0112681dd05eee06ae084f92f1e9448d` | 74 / `dc09763e46982ed64c6f28e94af3163e6cbad48fefc4a8b6413f16e732aa7b93` |

原始收据包SHA-256：`98854d8360645b9d451e6181173fe69a73d9cb62ddbaf5842e68e72037b14f2b`；driver轨迹：`d69af263251bf6fe1939f294efd4b4df91db6415862aaacc59214555ca788cfd`；规范化crash轨迹：`be9cd6c55980f124328adc7841dd976a34bb84caa5745bcffb7346432bdc1aa4`。

## 剩余gate与使用限制

本轮是独立服务与Anvil组件，G/gpos明确为四高度fixture；未建立真实07/08公共G、完整G′、12/13历史ResumeCut或02 grant/Ack接线。B17实际12执行和B18全系统验收须由对应owner继续。暂不可读的已接纳DA始终WAIT，不能用于制造fallback空块；独立D须按真实窗口及其实际parent执行。

服务进程退出与BFT同cid恢复分别报告。最新持续运行BFT profile不以完整01恢复为统一前置，但本次也未执行15-worker handoff或验证BFT恢复。Linux进程退出/fsync证据不覆盖断电、内核/硬件故障、拜占庭服务或无限保留，也不证明A-DELIVER/CERT-5/DA-RET预算。无staking、TLA或生产网络实验。
