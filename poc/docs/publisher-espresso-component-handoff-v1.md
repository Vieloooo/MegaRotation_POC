# 06 Publisher / SimEspresso 独立组件交付 v1

2026-11-16，Asia/Shanghai。**本次授权组件阶段完成：PE01–PE16 PASS；完整功能06仍 OPEN / INCONCLUSIVE，未计 accepted。** 确定性日志、独立 reader、TimestampPolicy、持久 Publisher 已实施并在同一准确源码提交上完成独立复核和矩阵。未合入 main、未推送远端。

## 提交、角色与所有权

| 项目 | 准确记录 |
| --- | --- |
| Worktree / branch | `/home/jy/.codex/worktrees/1fe1/megaroation_poc` / `codex/poc-publisher-espresso` |
| 初始基线 | clean detached `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b` |
| 政策合入 | 正常 fast-forward 到 `0a951c482a9712678ab9de34765b781b9c3c9f42`，未 reset 源 checkout |
| 合同与独立预期 | `4334f4f52bb1c349acf0abde56aafc12f6dc32d7` |
| 07/16 可消费日志/reader接口 | `9d3dc6cdc056f52e4cfcfbe48aeadb9b6a9eb976` |
| 完整组件候选、独立复核、Tester实际HEAD | **`8ba78f1ace3b383caa1c4360401c645d25907326`** |
| BFT pin | `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`；只读源已初始化子模块，本 worktree gitlink不变 |
| 工具 | Rust/cargo 1.95.0；参考 Commonware 2026.10.0；两个组件本身 std-only 加本地 path dependency |
| 数据目录 | `.build`指向 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/06`，独立cargo、target和artifacts |

实际启动顺序为命名 `bft_spec`（Astra/xhigh）→ `bft_verifier`（Astra/xhigh，先来源独立预期）→ `bft_builder`（GPT-6.1 Sol/high）→ 同Verifier复核 → `bft_simulator`（GPT-6.1 Sol/high）。主chat Astra/ultra由总协调读取runtime turn_context核验，turn ID为`01a1dfd1-e7a6-7191-aafb-31527a5cce90`。子角色按工具实际命名角色启动，未降级；各自报告区分角色配置与独立runtime attestation。

源码只写 `poc/crates/sim-espresso/`、`poc/crates/publisher/`，各有独立workspace/lockfile。文档只写本任务专属合同、准备/交付说明及 `poc/verification/publisher-espresso/`。各阶段串行移交writer；读者复核可并行且不改源码。根 `poc/Cargo.toml`、`poc/Cargo.lock`、types/core、其他功能源码、commander和子模块与政策基线无差异。

## 可消费行为

- **PE-LOG-1 / PE-LOG-WIRE-1**：显式propose/finalize产生连续q、非递减θ、完整有序消息列表；严格PELG envelope codec保留opaque payload。等θ/冻结θ合法，非递减超界只使时限前提失效，倒退拒绝且状态不变。E-TS相对proposal tick且只在proposal时live；旧non-live proposal不因后来finalize获得保证。
- **PE-READER-1**：独立source/reader绑定、分片交付和checkpoint；只暴露连续完整前缀及其watermark。读者未收到公开消息与公共历史尚缺结果正文分别表达。保留首次接收时间、q/index出处，q_avail归07。
- **PE-PUB-1**：generic Key/Result及结构化验证trait；WAIT持久保留，同key合法proof变体保留首份完整payload。PREPARED意图先落盘，再返回待受控提交的effect；dispatch、服务ACK、完整reader收录分别记录。稳定attempt身份、已验证截止、公开stop和recorded出处跨文件重开保留。
- **持久边界**：单writer文件锁，temp写入/sync→rename→目录sync；不确定提交poison当前句柄，连旧PREPARED重放也禁止，必须重开。STOP不删除历史收录；未验证的STOPPED必须先完成Offer验证，才允许后来核明历史收录。

准确Rust API和边界见 [SimEspresso README](../crates/sim-espresso/README.md)、[Publisher README](../crates/publisher/README.md)；规则版本见[冻结合同](contracts/task-contract-publisher-espresso-v1.md)。接口提交之后只增加共用消息列表大小检查，原日志/reader接口保留。

## 运行结果与原始证据

| 阶段 | 实际结果 |
| --- | --- |
| Builder | 两crate编译/格式检查；7日志/reader + 8 Publisher测试PASS；seed42、2ms smoke PASS |
| 独立Verifier | 自写外部harness 6/6 PASS；独立审查并执行恢复故障4/4 PASS；无阻塞finding |
| Tester PE01–PE15 | 同一`8ba78f1`运行15项测试全部PASS |
| Tester PE16 | seeds **7/42/99 × wall pause 0/2ms**：6/6回放、3/3配对PASS；每次另按独立预期检查完整JSONL/PELG/checkpoint字节 |
| lead封存核对 | 两份证据清单共49+121项哈希全部一致；组件源码与验证候选无差异 |

种子只选择三个固定交付计划：B的前三块顺序分别为`2,0,1`、`2,1,0`、`0,2,1`。每次覆盖virtual ticks 100–146、四个q、θ=`104,104,104,105`；不代表穷举调度。首次public proof在q0，材料补齐到q2；Recorded保存publication q0/index0和complete-through q2，不冒充07的q_avail计算。每对比较完整虚拟JSONL、4个PELG、3个reader checkpoint、PREPARED及最终Publisher snapshot共10个文件。stderr/wall成本单列，lock文件无协议状态。

| 原始证据目录（绝对路径） | SHA256SUMS摘要 |
| --- | --- |
| `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/06/artifacts/verification/8ba78f1ace3b383caa1c4360401c645d25907326/independent-run1` | `881b45c9920eed281b0d83a730a690b6b30328bd1e5df65b61b8706df5dc6df0`（49文件） |
| `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/06/artifacts/simulation/8ba78f1ace3b383caa1c4360401c645d25907326/matrix-run1` | `ef61f46f11461df871771526d8e71b6ffd123591a8f8c54bbbab0fa6f19f4b7e`（121文件） |

[独立复核报告](../verification/publisher-espresso/candidate-review.md)记录原始命令、harness和日志hash；[Tester报告](../verification/publisher-espresso/simulation-report.md)与[机器结果](../verification/publisher-espresso/simulation-results.json)记录8条串行flock命令、exit、输入hash、逐seed轨迹hash和wall诊断。所有Rust build/test/仿真使用 `/tmp/megaroation-poc-heavy.lock`、`CARGO_BUILD_JOBS=2`、Rust1.95.0、`--locked --offline`；未并发重型任务。

交付提交新增文档/验证工具，组件源码仍等于候选。可从源码相同的交付HEAD复跑，必须换新输出目录；脚本记录实际HEAD并拒绝组件源码变化。正式已执行矩阵使用`--exact-head`且实际HEAD等于候选。下列命令省略该选项以允许交付HEAD：

```sh
python3 poc/verification/publisher-espresso/run-matrix.py \
  /mnt/reth/jy/megaroation_poc/.build/commander-jobs/06/artifacts/simulation/8ba78f1ace3b383caa1c4360401c645d25907326/matrix-run2
bash poc/verification/publisher-espresso/run-independent.sh \
  /mnt/reth/jy/megaroation_poc/.build/commander-jobs/06/artifacts/verification/8ba78f1ace3b383caa1c4360401c645d25907326/independent-run2
```

## 准确依赖与未过gate

| Gate | 后续需要的准确接口/接线 | 当前结论 |
| --- | --- | --- |
| D03-M1 | 已只读核对`0e72dc247de22afdc8d6681713b3d15f4aa05262`，合同`dae686b7152a4b8dabcc670fa08e9abd88bf276d`。可绑定`Key=poc_types::domain::ResultKey`及`Canonical`；66-byte BE v1编码，不重算opaque ID。未导入该分支。 | 与06 generic边界兼容；不代表03全功能验收 |
| D03-M2 / D04 | RegionalResult/Descriptor/PhysicalBlock/AnchorEvidence、真实canonical payload codec标识、完整ancestry/anchor验证、原生CheckFINAL和真实四不同signer证书负例的接受SHA。 | INCONCLUSIVE / OPEN |
| D02 | endpoint/channel、授权reference tick、effect→完整Ack持久custody、dispatch回执和稳定ID重放、逐reader实际交付。06 block上限1MiB/消息64KiB，02草案payload64KiB，需要准确分片规则及接线SHA。 | INCONCLUSIVE / OPEN |
| D07 | 带公开position及精确session/branch作用域的StopDecision和截止来源；迟到Offer须重新投影历史公开排除；07独立计算q_avail/GFR。 | INCONCLUSIVE / OPEN |
| D16 | 官方来源身份、q/θ/完整列表映射和finality验证；确认是否具有可核验proposed_at/live_at_proposal，缺失时不得填造。 | INCONCLUSIVE / OPEN |
| D06-integrated | 02真实调度下Publisher/独立reader的受控grant/Ack、恢复及原06矩阵，同一接受集成SHA；由03 owner后续收口根workspace。 | INCONCLUSIVE / OPEN |

本次fixture走相同trait且明确为non-cryptographic；没有把验证桩当BLS通过，没有真实网络/15-worker/官方Espresso或G接纳证据。真实文件重开和注入I/O失败是有限组件证据，不是物理断电实验或01/02恢复custody证明。PUB-4时限活性、I1和完整功能06均未因本交付关闭。
