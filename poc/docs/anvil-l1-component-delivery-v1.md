# 功能09：独立组件交付 v1

2026-11-16（Asia/Shanghai）。**本次授权的独立组件阶段完成：A01–A17 PASS。完整功能09保持 INCONCLUSIVE，未计为 accepted。** 合约和显式输入 driver 已经独立复核，正式 Tester 在同一准确源码提交运行矩阵；未合入 main、未 push。

## 提交、工作区和所有权

| 项目 | 精确值 |
| --- | --- |
| 初始 checkout | `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b` |
| 正常合入的管理政策／实施基线 | `0a951c482a9712678ab9de34765b781b9c3c9f42` |
| 可独立消费的合同／黄金向量提交 | `daad412e46eeecd64ef9b8af8175e64ec40edc53` |
| 源码候选、独立复核、正式测试 SHA | `87daee8e9e77bbe27c337383d873a5eaf4a694ba` |
| 工作区 | `/home/jy/.codex/worktrees/65ef/megaroation_poc` |
| 分支 | `codex/poc-anvil-l1` |
| 合同 | [ANVIL-L1-ABI-v1 r1](contracts/task-contract-anvil-l1-v1.md) |

本交付报告提交只添加说明和独立验证证据，不改变上述已测试的合约、Rust 组件或场景源码。09源码仅在 `poc/contracts/`、`poc/crates/l1-adapter/`；后者使用自己的 workspace/lockfile。根 Cargo、公共 types/core 声明、01 native 文件、02 transport 和 commander 登记均未修改。源 checkout 未 reset。

Regional BFT gitlink 保持 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，只读使用源 checkout 已初始化副本；Commonware pin 为2026.10.0。本组件不实现 BFT，也不重定义公共 ReplayPosition 或 ResumeCut。

## 实际角色与实现

主 chat 为 GPT-6 Astra/ultra，由总协调只读检查实际 `turn_context` 核实（turn `01a1dfd2-46a9-7653-bbe5-bac657d8be7b`）。实际启用命名角色 `bft_spec`、`bft_verifier`（均 Astra/xhigh）、`bft_builder`、`bft_simulator`（均 GPT-6.1 Sol/high）。先保存 source→expectation 和黄金向量，再启动单 Builder；独立 Verifier 通过后才启动 Tester。无并行源码 writer，未静默降级角色。

- ControlAuthority／BatchInbox 实现真实 EIP-712、secp256k1、完整 cursor 与 DA/attestation 绑定、固定 static policy、首序号0及跳号、请求语义幂等、原子 RESUME/Override 和初始化事件。独立黄金向量覆盖4类真实签名、57项字段/域变异、40项非法签名、34项 Cast 交叉检查。
- 同一个 Rust driver 执行显式 Tick、raw Submit、Mine、Finalize、Deliver、Read；部署也走此接口。包含完整真实 transactions/receipts、交易内与区块全局日志索引、同交易 Set/Resume 配对、历史 block-hash 状态核对和 A/B reader 前缀隔离。
- 出块已执行但回包丢失时阻止下一次 mutation；重放原 command 只读核对已有结果，不重复挖矿。命令状态当前在内存中，跨进程持久恢复未验收。
- r1 显式限制所有写入口使用规范 ABI 参数，拒绝 trailing/alias 布局；此限制经 lead/Verifier 在候选前确认，selector、签名域和规范请求的幂等身份不变。

可消费 ABI 位于 [ControlAuthority](../contracts/abi/ControlAuthority.json)、[BatchInbox](../contracts/abi/BatchInbox.json)；运行入口见 [adapter README](../crates/l1-adapter/README.md)。公共 L1 对象映射保持合同§5的单一03 owner；opaque gpos仅证明字节绑定。

## 正式证据和复现

[独立审查报告](../verification/anvil-l1/reports/verification-87daee8.md)与[正式 Tester 报告](../verification/anvil-l1/reports/simulation-87daee8.md)分别列出准确命令、工具版本、逐项 A01–A19 结果和界限。可机器读取的[正式 manifest](../verification/anvil-l1/reports/simulation-87daee8-manifest.json)包含34份原始文件的 SHA-256、工具与可执行文件 hash。lead已再次核对34份原始文件和4份独立结果的 hash。

原始正式证据绝对目录：

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/09/artifacts/anvil-l1-simulator-87daee8-run1`

独立复核原始目录：

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/09/artifacts/anvil-l1-verifier-87daee8-run2`

两次正式运行分别插入300ms、650ms墙钟暂停，seed=`none (explicit script)`。每次区块范围0–7（含genesis共8 headers），7次实际 Mine RPC，195条command/reply，最终相对虚拟时间16,000,000μs；各145笔收据，其中129笔为预期回滚、16笔成功。独立重建11条协议事件、5次幂等请求、6次历史cursor读取和2次精确事件读取。虚拟轨迹、完整driver RPC轨迹、原始收据bundle跨暂停逐字节一致。

| 摘要 | SHA-256 |
| --- | --- |
| 虚拟轨迹 | `114bb15dc6c444da432ab5841dde21792ab8aa97032d0a7c42eb48a85b20b025` |
| 原始 RPC bundle | `c4eee3e00c96b524e31802d71ab883764aa6733beb0686f2ec8b86194eb5426c` |
| 正式 manifest | `10ce07d7922f0c53f579a9f820acf80b28d036666bcc14ae79b9c60ab186bdb9` |

正式 Tester 无异常失败或超时，锁内构建和矩阵本机耗时12.649秒；这不是虚拟时限或网络延迟测量。Verifier早期一次预检误选系统Python、缺coincurve，另有一次自动审批超时，均保留于其报告，不算候选行为失败。有效环境为 `/home/jy/miniconda3/bin/python3`（3.12.8／coincurve20.0.0），Rust1.95.0、Foundry1.0.0-stable、solc0.8.20。所有heavy使用 `flock /tmp/megaroation-poc-heavy.lock` 和 `CARGO_BUILD_JOBS=2`。

任务 `.build` 指向专属数据盘09目录，Cargo registry仅复制公开缓存，无凭证或全局配置；target、artifacts、进程输出独立。Anvil使用本次随机loopback端口并回收自己的child。保留的[临时结果映射](../verification/anvil-l1/reports/preserved-artifacts.json)使报告中的 `/tmp` 检查产物在仓库中也有逐字节相同副本。

## 剩余 gate

**A18 INCONCLUSIVE：** 等待02准确提交的真实Clock/transport/grant/ack和03唯一公共对象／reader接口，在准确集成SHA上回归完整前缀、历史读取及受控输出。新鲜私有Anvil、可信本地RPC、内存命令状态和显式脚本是本阶段前提；不声称跨进程恢复、Ethereum共识/MPT证明或公网测量。

**A19 INCONCLUSIVE：** 11实际frame/DA持久性/Certifier/G来源、12按具体位置的历史ResumeCut及13恢复终点由对应owner接线。本组件没有构造G/G′/D或假cut，也没有把合约接纳当作GFR激活或恢复成功。后续真实消费者使用本合同与ABI核对，并继续保留source、written_at、Set位置和Resume位置的区别。
