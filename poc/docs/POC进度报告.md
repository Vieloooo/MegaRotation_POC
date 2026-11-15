# Sequencer Rotation PoC 进度报告

2026-11-15 核对了 PoC 提交 `36eff1c`。Regional BFT 子模块固定到 `691cfc1`，Rust 和 Commonware 分别固定到 1.95.0 和 2026.10.0。历史验收各自绑定结果文档所列的版本。

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
| 同一 cid 崩溃恢复 | 已实现并复跑一条场景 | seed 7 在 Verify 等 body 时杀掉 worker，以新进程恢复相同工作和原认证截止时间。场景在恢复等待处结束，交付数为零。R1 的独立进程验收与完整 I1 仍开放。[恢复实现](native-recovery-implementation.md) · [恢复合同](contracts/task-contract-native-recovery-v1.md#12-executable-acceptance-r1r18) |
| Mock Plan 与轻量执行 | 选区和 leader 规则已写，原型有区内选举 | `(β+r)%3` 决定地区，`(v−1)%5` 决定区内 leader。原型实现五成员内部的轮转，三地区 `StaticScheduleProvider`、Plan/descriptor 绑定、固定 `dep[h]` 和交易执行待实现与验收。[v2 设计 §3](../../formalization_v10/poc_design_zh_v2.md#3-模块化把授权调度与共识分离) · [原型 elector](../prototypes/i1b-runtime/src/native_elector.rs) |
| 公共日志与外部服务 | 有服务设计和 Anvil 环境 smoke | Anvil 手动出块、部署和状态修改通过 12 个检查。PoC 的 L1 driver、ControlAuthority/Inbox、Publisher、SimEspresso、治理 M、Batcher/Certifier 和 DA 待实现。[v2 设计](../../formalization_v10/poc_design_zh_v2.md) · [Anvil 记录](setup/foundry-2026-11-15.md) |
| Session Manager | v10 有接口、状态、候选调度及关闭规则 | 当前 worker 有基础 stop/retire，完整的每 cid 候选管理、Pause 后恢复原实例、公开选定后的排除及持久终止尚待接入。缺独立状态转换和多进程测试。[Manager 规格](../../formalization_v10/03_session_manager_zh_v0_1.md) · [regular 准备](contracts/regular-preparation-v1.md) |
| Viewer GFR、D 与 Observer | v10 有重放接口和三种交接规则，D 有来源准备稿 | 未实现读取公共日志产生 G 的 viewer GFR，也未实现独立 D 派生、两个 reader 或 PoC Web。当前尚无 regular、Force、RESUME 的端到端测试。[GFR 规格](../../formalization_v10/09_finality_rules_zh_v0_1.md) · [D 派生准备](contracts/derivation-preparation-v1.md) |

## 本次复现

在 `36eff1c` 上，I1a 默认套件 20 项通过、11 项默认跳过。启用 loopback 后单独运行的 10 项 socket 测试全部通过。原生原型 129 项通过，两个工作区的格式检查和 `native-hub`、`native-worker` 锁定构建均通过。历史 Fresh 矩阵对应 `fec7ade`，I1a 矩阵对应 `35112ac`。Fresh 矩阵中的实际 resolver 消息数为零。

从仓库根目录运行，工具链配置见 [README](../../README.md#工作区与复现)。

```sh
./poc/scripts/bootstrap.sh
./poc/scripts/bootstrap.sh test --locked --offline -p sim-runtime --test i1_acceptance -- --ignored socket_ --test-threads=1
./poc/scripts/bootstrap.sh fmt --manifest-path prototypes/i1b-runtime/Cargo.toml --all --check
./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline
```

重新构建后复跑了两次 `recover-pending-verify --seed 7`。第二次附加 `--wall-pause-ms 2 --reverse-start`。两次都恢复了原认证截止时间 `1,419,500 µs`（节点时钟），在虚拟时间 `1,046,000 µs` 结束，交付数均为 `[0,0]`。场景配置、54 行虚拟 trace、消息帧均逐字节一致。trace SHA-256 为 `97e3d7e1d8035d68b9673f94ef2853936e6cd825fd64008192ff954b6c419568`。命令见[恢复运行说明](native-recovery-implementation.md#bounded-process-scenario)。

## 实现前仍需冻结的合同

- **Schedule 与 Session Program。** 固定公开 registry、全局 pid／区内编号／Commonware signer index 的映射、Plan/descriptor/cid 与证书域，以及 physical block、状态和 native hash 的编码向量。冻结首次完整收包、来源认证和时间窗口的 Accept 条件。逐节点验证相对虚拟参考时钟的误差 ≤1 ms，独立记录 RPC 延迟、Enter 时差和实际共同参与区间。[regular 准备](contracts/regular-preparation-v1.md)
- **Manager。** 定义从公共 lifecycle 事件到本地动作的输出、持久化和幂等边界。一 worker 共用 Clock、executor 和根 journal，每个 cid 持有独立的 engine、签名状态、存储和待完成工作。用重复 Enter、切换后选回原候选、关闭 Preparing/Paused、重启后的终止守卫覆盖生命周期。[v10 Manager §3–4](../../formalization_v10/03_session_manager_zh_v0_1.md)
- **Viewer GFR。** 冻结 canonical replay position、输出比较字段及验收向量。同键按 L1 control → Expire → SessionEnd → Espresso 处理，缺公共覆盖或本地必需材料时在原位置 WAIT。测试 proof 先到、body 后到、公开时间和接纳时间分离、包含 padding 的逐高度 Expire，以及两个 reader 在不同下载顺序下重放到同一位置的 G、Tail 和事件。[v10 GFR §3–4](../../formalization_v10/09_finality_rules_zh_v0_1.md)
- **RESUME 接口。** Manager §3.10 预期先收到 `Resumed` 保存材料，再消费 `SessionAssigned`。GFR §4.6 的书写顺序相反。需要明确同一次原子转移返回的事件顺序并测试。D 还需冻结队列初始状态、epoch 推进及按具体 L1 位置持久化的 ResumeCut。[Manager §3.10](../../formalization_v10/03_session_manager_zh_v0_1.md#310-applylifecycleevent) · [GFR §4.6](../../formalization_v10/09_finality_rules_zh_v0_1.md#46-resume) · [D 派生准备](contracts/derivation-preparation-v1.md)

## 实现顺序与完成条件

1. **收口 I1。** 先独立验收已有 pending-Verify 进程场景，再补不同恢复状态与持久化切点、原 deadline、丢 Ack、outbox 和启动栅栏。完成真实 resolver 请求、丢响应、重启后修复，并在同一集成提交上完成 R1–R18 与固定 seed 重放。规则合同可同时准备。[恢复合同](contracts/task-contract-native-recovery-v1.md#12-executable-acceptance-r1r18)
2. **15 worker 与首个区域 session。** 三个独立的五成员 registry，接入静态 Plan、轻量执行、受控 finalized L1 fixture、slot-aware Build/Accept。在 15 个真实进程中先运行一个地区的五个真实引擎。验收错时 Enter、三 ready 两 late、3/2 候选分裂及四个不同合法 signer。[REG-A 准备](contracts/regular-preparation-v1.md#two-increments)
3. **三个地区 regular。** 接入 Manager、持久 Publisher、SimEspresso 和纯 GFR，再让两个 reader 独立下载与重放。分别跑接满 h10 和 SessionEnd 只到 h9 的路径，后继窗口均为 11–20，后一条由合格的 h11 结果覆盖 padding10。三区场景在 R2 第一个合格结果进入 G 时完成。Web Observer 可从这一阶段开始接入只读轨迹。[REG-B 准备](contracts/regular-preparation-v1.md#next-acceptance-scope)
4. **Force。** 用 Anvil 与 L1 driver 跑真实控制交易，治理 M 按公开进度触发。覆盖观察者落后、Espresso 停滞、在途 regular、同键控制和 HALT。以新分支首个合格结果进入 G 作为成功终点。[v2 设计 §7–9](../../formalization_v10/poc_design_zh_v2.md)
5. **D 与 RESUME。** 接入 Inbox、crash-only Batcher/Certifier 与持久 DA，让 D 仅用 finalized L1、接纳记录和可验证数据独立派生。覆盖 DA WAIT、共同高度 hash 分歧与单纯 lag、同区块不同位置的两条 RESUME。以 G 对齐历史切面且新分支首个合格结果进入 G 作为成功终点。[D 派生准备](contracts/derivation-preparation-v1.md) · [v2 验收矩阵](../../formalization_v10/poc_design_zh_v2.md#122-第一阶段验收矩阵)

各阶段保留独立预期、原始输入和重放轨迹。综合验收覆盖多个 seed、墙钟扰动和不同输入到达顺序，分别记录 regional final、G accepted、Inbox accepted 和 D derived。全阶段需在单机 15 worker 上跑通三个交接场景，并可通过 Web 追溯对应的公共位置和节点实际 Enter。

## 怎么看现有材料

`formalization_v10/` 是当前协议文档和 PoC 设计。`poc/crates/` 是 I1a，`poc/prototypes/i1b-runtime/` 是原生 Simplex 原型。`poc/docs/contracts/` 下的 regular 与 derivation 准备稿列出了尚待冻结的工程选择，I1 和恢复合同定义了可执行验收。`poc/verification/` 存放检查代码和版本化结果。`poc/artifacts/` 存放本地原始轨迹和实验输出。部分 v10 历史链接指向整理前的资料，可从 `0f013e2` 查阅。运行命令见 [README](../../README.md)。

Regional BFT 子模块中的共识回放展示的是 15 成员委员会的 benchmark。此 PoC 的目标是 3 个各有 5 个成员的独立委员会。两者各有自己的配置和验收范围。
