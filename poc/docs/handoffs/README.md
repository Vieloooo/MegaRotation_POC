# Sequencer Rotation PoC v2 分任务交接

2026-11-15。每项工作对应一份中文启动文档。交给新会话时，附上目标任务的 Markdown 和[共用工作约定](workflow-zh.md)，并在启动消息中填入该任务的集成基线 SHA。可直接复制任务开头的启动段。新会话使用自己的 Codex managed worktree。讨论和验收可以按任务分开，主线集成由总协调收口。

用户现已授权[总协调批次调度](../commander/README.md)：最多五个并发 job，每完成三个功能创建独立合并到 main 的 chat。实际派发基线以总协调消息和登记为准；下文 `36eff1c` 是实现基线，其上的文档提交可作为启动基线。

## 按依赖派发

最新[不重启BFT worker路线](../commander/restart-free-handoff-track-20261116.md)允许02普通接线、04健康准入及三类handoff场景独立于完整01推进。只把同cid进程重启、恢复Ack/签名状态等验收留作01依赖。当前五个活跃功能先接收通知；下一空闲槽优先给02普通接线/04健康准入，不能为通知idle chat超过并发上限。

**调度修订：** [分层依赖v2](../commander/dependency-plan-v2.md)覆盖旧的整任务等待规则。下表保留完整接线所需的依赖；独立组件可在接口冻结后先实现，03不再依赖01。表中的编号用于指向任务文档，提交时使用完整 SHA 以及对应的合同和验收报告。多条前置工作合并后，从同时包含它们的一个集成 SHA 创建下游 worktree。

| 任务 | 已有基础与本次交付 | 实现基线 |
| --- | --- | --- |
| [01 原生恢复与 I1](01-native-recovery-zh.md) | 从一个 pending-Verify 进程场景扩展到 R1–R18，关闭 I1 | 当前 `36eff1c` |
| [02 全局时钟与仿真通信](02-sim-communication-zh.md) | 泛化现有真实 native 路由，接通多 endpoint、按 cid 分发和按 channel 注入故障 | 01 |
| [03 Mock Plan 与轻量执行](03-mock-plan-execution-zh.md) | 固定 3×5 registry、静态排班、公共编码、Merge 和执行向量 | 无01实现依赖；先冻结本功能合同 |
| [04 15-worker Session Program](04-session-program-zh.md) | REG-A，15 个真实进程中先运行一个地区的五个真实引擎，完成 slot-aware 准入 | 02 + 03 |
| [05 Session Manager](05-session-manager-zh.md) | 每 worker 的候选管理，多 cid Pause/恢复和持久 Close | 04 |
| [06 Publisher 与 SimEspresso](06-publisher-espresso-zh.md) | 持久发布任务、确定性 finalized 公共日志和独立 reader 交付 | 02 + 03 |
| [07 Viewer GFR](07-viewer-gfr-zh.md) | 纯 GFR 重放、regular/Expire、独立 A/B reader | 03，使用 06 的冻结日志接口；接真证书需 04 |
| [08 三区 regular 集成](08-regular-integration-zh.md) | REG-B，满窗口与 SessionEnd 两条 15-worker 交接路径 | 04 + 05 + 06 + 07 |
| [09 Anvil L1 与合约](09-anvil-l1-zh.md) | 受控 L1 driver、ControlAuthority、BatchInbox、finalized 日志读取 | 02 + 03，先与 11/12 核对 ABI 和数据承诺 |
| [10 Force](10-force-zh.md) | 由公开停滞触发真实交易，GFR 切换 branch 并恢复生产 | 08 + 09 |
| [11 批次、Certifier、DA](11-batch-services-zh.md) | crash-only 服务，真实 Inbox 接纳收据和 G′ 读取视图 | 08 + 09 |
| [12 独立 D 与 ResumeCut](12-derivation-zh.md) | 原生派生子集、窗口回退与按 L1 位置冻结切面 | 03 + 09，先用已冻结的 11 接口开发；实际数据联调需 11 |
| [13 RESUME](13-resume-zh.md) | 治理恢复、GFR 原子对齐、Manager 接续及首个全局接纳 | 10 + 11 + 12 |
| [14 Web Observer](14-web-observer-zh.md) | 只读快照和 SSE、独立 reader 诊断、故障与交接回放 | 08 可启动；四层链和恢复验证需 13 |
| [15 全系统验收](15-system-verification-zh.md) | 同一集成提交上的完整 v2 矩阵、可重放证据包和三条演示 | 13 + 14 |
| [16 官方 Espresso 兼容](16-espresso-compatibility-zh.md) | 真实 dev-node 的独立 API、录制和离线回放 | 06 的公共日志接口已冻结 |

当前同时推进 **01/02/03/06/09**。01恢复独占native接线；02先做独立transport；03先做公共对象与执行；06先做公共日志；09先做真实合约。接口/组件提交可提前供下游消费，完整功能仍按原矩阵验收。之后按就绪队列安排04/05/07/11/12/14/16的独立组件或集成工作，不等固定三任务批次全部结束。

## 共享路径与集成顺序

- **01 → 02 → 04 → 05 → 08。** 依次交接 native hub、driver、worker 和共用 wire 定义。使用上一任务验收后的集成提交。02 的路由接口经冻结后，06 和 09 在各自模块使用它。公共接线由总协调安排。
- **03 → 04/06/07/09/12。** 03 提供权威的 Plan、descriptor、执行和公共对象接口。02 拥有通信 wire，03 拥有 `poc/crates/types/` 的公共协议域对象。多个任务都需要修改 workspace manifest、lockfile、`lib.rs` 或同一 proto 时，总协调先提交共用声明，或在合入时串行负责这些文件。各 Builder 同时只写其专属模块。
- **07 → 10 → 13。** GFR 从 regular/Expire 扩展至 Force，再扩展至 RESUME。10 负责治理控制状态机初版，13 在其集成提交上增加恢复规则。
- **09 → 11/12/13。** 合约、ABI 和 L1 driver 的所有权在 09。11/12/13 消费同一签名域和 finalized 日志映射。需要改 ABI 时由总协调指定一个修改提交，随后重跑受影响任务。
- 每个任务回传候选 Git SHA、基线 SHA、合同、独立审查和测试矩阵。总协调核对实际集成 SHA，合并改变相关行为时在该 SHA 重跑受影响验证。原始轨迹保存在相应工作区的 `poc/artifacts/`。

历史数值和本次复跑见[进度报告](../POC进度报告.md)。PoC 目标和原始验收编号见 [v2 §12–13](../../../formalization_v10/poc_design_zh_v2.md#12-参数场景与验收)。
