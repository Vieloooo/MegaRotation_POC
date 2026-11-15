# Sequencer Rotation PoC 进度报告

日期：2026-11-15。实现基线为 PoC `0f013e2`，Regional BFT 子模块固定到 `691cfc1`。各项历史验收使用的版本见下文。

## 我们想做什么

在一台机器上运行 15 个独立 replica 进程，分成三个地区，每区五个等权成员。每个地区使用 Commonware Simplex，系统通过受控时钟、网络、L1 和公共排序服务来观察跨区交接。最终需要三个可重放的完整场景：正常 regular 轮转、生产停滞后的 Force、批次服务故障造成链分歧后的 RESUME。Web 页面需要解释每次交接的原因和实际发生时间。第一阶段使用模拟 staking，不实现 staking/slashing，也不做 TLA 证明。[设计与验收](../../formalization_v10/poc_design_zh_v2.md)

## 现在做到哪里

项目处于运行基础和恢复阶段。已经有真实的双进程 Simplex 场景，可以按固定时间和消息顺序重复执行。完整 15 进程、三个地区轮转，以及 Force 和 RESUME 还没有端到端跑通。

| 部分 | 当前结果 | 结果覆盖的范围 |
| --- | --- | --- |
| I1a 时钟、消息和进程控制 | 通过有限验收 | 两个真实 worker 使用 SignedIntent fixture，8 个场景、3 个 seed 和两种执行条件共 48 次运行，24 对完整虚拟轨迹一致。[结果](i1a-gate-result.md) |
| 原生任务、时间和随机数 | 通过子项验收 | 27 个独立测试分别在两种墙钟暂停条件下通过。[结果](i1b-runtime-slice-result.md) |
| 原生存储和持久化见证 | 通过各自有限验收 | 覆盖受控存储 I/O、日志与见证、持久授权及相关异常。[存储](native-storage-result.md) · [见证](native-witness-result.md) |
| 原生 Simplex Fresh | 通过有限重放 | 两个真实 worker 加最多三个脚本签名者，五成员 quorum 为四。66 次运行经独立检查，33 对完整结果一致。健康场景两个真实 worker 各交付两块。[结果](native-engine-fresh-result.md) |
| 同一 cid 崩溃恢复 | 已实现并复跑一条场景 | seed 7 在 Verify 等待 body 时杀掉 worker，以新进程恢复相同工作和原认证截止时间。场景在恢复等待状态后结束，交付数为零。该场景的完整独立进程验收仍待完成。[实现和复跑](native-recovery-implementation.md) |
| L1 工具 | 环境 smoke 已通过 | Anvil 手动出块、部署和合约状态修改通过 12 个检查。PoC 的 L1 驱动、治理和 Inbox 尚待实现。[记录](setup/foundry-2026-11-15.md) |
| regular、Force、RESUME | 设计和部分准备已写 | 尚未完成 15 个真实 worker 的接入及端到端交接。[regular 准备](contracts/regular-preparation-v1.md) · [D 派生准备](contracts/derivation-preparation-v1.md) |

历史 Fresh 结果对应 `fec7ade`，I1a 对应 `35112ac`。这些数字来自有限固定场景，时间均为模拟虚拟时间，采集耗时是本机墙钟时间。Fresh 没有执行 resolver 修复。2026-11-15 对 `0f013e2` 复跑了 129 项原生测试和一次 seed 7 进程恢复。I1a 默认测试有 11 项 ignored，其中包括单独执行的 loopback 控制。详见各项结果文件。

## 还需要完成什么

1. 收口 I1b 恢复验收。当前 pending-Verify 只是 R1 的有限场景，还需要补齐不同状态和持久化切点、超时与重试、丢失 Ack、恶意输入、启动栅栏以及 R1–R18 的独立进程验收。实际 resolver 请求、响应丢失和重启后修复也需要跑通。[恢复合同](contracts/task-contract-native-recovery-v1.md)
2. 在验证后的版本上扩展到 15 个真实 worker，建立三个独立的五成员身份集合、时钟和 session 准入，接入轻量执行。
3. 接入 Manager、Publisher、模拟 Espresso 和 GFR，验证跨三个地区的 regular 交接，包括完整窗口和 SessionEnd 路径。
4. 接入 L1 治理控制、Force、批次和 DA 派生、历史 ResumeCut，再实现 RESUME 和对应故障场景。
5. 构建 Web Observer，展示 15 个节点、时钟、链与每次交接。完成多 seed 的时间边界、重放和故障验收。

具体实现前仍需冻结编码、接口和测试范围。regular 与 D 的准备稿列出了尚待决定的内容，例如输入准入与时序界、派生队列初始化和恢复切面。

## 怎么看现有材料

`formalization_v10/` 是当前协议文档和 PoC 设计。`poc/crates/` 是 I1a，`poc/prototypes/i1b-runtime/` 是原生 Simplex 原型。`poc/docs/contracts/` 记录合同，`poc/verification/` 存放检查代码和版本化结果。`poc/artifacts/` 存放本地原始轨迹和实验输出，不随普通 Git checkout 携带。运行命令见 [README](../../README.md)。

Regional BFT 子模块中的共识回放展示的是 15 成员委员会的 benchmark。此 PoC 的目标是 3 个各有 5 个成员的独立委员会。两者各有自己的配置和验收范围。
