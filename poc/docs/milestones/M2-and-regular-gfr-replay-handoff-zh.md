# M2 与 M1 Regular GFR 回放交接

两个独立的 GPT-6.1 Sol / ultra chat 从包含已验收 M1 smoke 的本地 `main` 开始，各用自己的工作树。用户已授权以该 smoke 启动两项工作。第一轮记录基线完整 SHA 和实际工作目录，遵循 [AGENTS.md](../../../AGENTS.md) 和[当前模型策略](../commander/model-policy-20261116.md)。在编码前冻结合同，由 Verifier 先从规则推导独立预期，沿用单 writer 和重型构建调度。[M1 smoke 证据](results/M1-healthy-smoke-result-zh.md)

## A · M2 健康节点主动 Force

- 复用 M1 的 15-worker 链路和已有 `l1-adapter`/ControlAuthority。主动构造合法单签 Force，经真实 Anvil 手动出块、finalize、公开输入覆盖和 GFR `act` 处理。激活时保留旧 `G`，关闭该分支全部实例，按激活前的 anchor、实际分支/序号和固定窗口创建正确 session。
- 通过条件为正确新 `cid` 在约定准备预算内就绪，且新分支首个合格结果实际进入 `G`。同时测旧实例终止后的签名守卫、迟到旧结果、同键顺序、同分支在途 regular、stale branch/HALT 及去重。复跑健康 regular，封存真实交易、时间点、公开轨迹、独立检查结果和确切候选 SHA。
- 合同与工作路径以 [H4](H4-healthy-force-zh.md)、[M2 计划](M2-force-merge-zh.md) 和 [M1–M5 路线](M1-M5-implementation-plan-zh.md) 为依据。优先实现并验证健康主动 Force，自动故障检测和治理多签属于 M3。

## B · 从 M1 轨迹回放 Regular Handoff 和 GFR

在 `poc/web/` 与专属验证路径实现只读离线回放。读入 M1 封存原始轨迹与必要的解码投影，按虚拟时间和公开重放位置联动下列四项。负责解释从区域最终结果到全局接纳及职责切换，区域投票和共识消息动画在本任务范围外。

| 视图 | 输入与内容 |
| --- | --- |
| L1 log | `raw/l1.bin` 的 finalized L1 fixture。显示 header、timestamp、finalized 时间和控制列表。将规则所需覆盖位置标为根据 `raw/profile.json` 与候选配置推导，逐 header 的 reader 收取时刻按缺证据显示。 |
| Espresso log | `raw/espresso/*.bin` 和 `PublisherRecorded`。显示 finalized `q`、时间键 `θ`、实际 finalized 时间、正文/证明位置、`q_avail`。 |
| 区域最终结果 | `raw/results/*.bin` 和 `RegionalDelivered`。按 `(cid, physical_id)` 关联记录，展示区域、height、父块和最终结果的证据位置。 |
| 全局 `G` 与 GFR 规则 | `GAccepted`、`GfrReplayRecord`、`Handoff` 及独立 `public-replay.json`。显示当前 `G`、接纳键、anchor 和后继窗口，解释 L1/Espresso 覆盖、首次公开完整、上下文/父链检查、满窗后完成旧职责并分配下一职责。 |

页面支持播放、暂停、单步、筛选和跳到两次接棒。验收核对 h10 对应 q9、h20 对应 q19 及 h21 对应 q20，逐项可回到对应输入记录。早停 `SessionEnd` 可以用规则说明，画面中的实际事件来自该次满窗轨迹。使用固定的证据输入生成回放数据，按 [v10 GFR §3–4](../../../formalization_v10/09_finality_rules_zh_v0_1.md#3-gfr事件与排序) 写规则说明并独立对照快照。交付可在本机浏览器打开的页面、生成命令和检查结果。[Regional BFT 回放交互参考](../../../regional_BFT_sim/visualization/README.md)
