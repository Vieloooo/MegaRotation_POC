# 10 治理 Force 与生产分支切换

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以含 **08、09** 的集成 SHA 建 managed worktree `poc-force`，分支 `codex/poc-force`。实现可信治理 M 的公开停滞策略，真实 L1 Force 激活并切换生产，直到新分支首个结果进入 G。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 实现规则

- M 通过自己的公共 reader、健康 Clock 与显式服务观察执行 PublicationStallPolicy。用当前被选中 cid 的最近 G 接纳进度、完整公共输入和正常推进的 q 与 θ 计算可观察停滞。记录发送前 branch/Tail 复查、依据位置和过期/准备预算。治理输入来自公开观测和授权探针。
- 建立 durable 在途控制状态机与去重，一次至多一条未决控制。区分 submit、included、finalized、GFR applied 和 recovery observed。明确交易失败、处理后的 no-op 和已激活但 HALT 如何释放在途槽，恢复监视与在途槽各自有状态。
- GFR 在实际 act 和公共全序的位置处理 Force。激活时 branch 仍匹配且 Tail 允许才替换。当前 branch 中间发生 regular 后重新按实际 Tail 判断。使用激活前 G head 为 anchor 并保留旧 G，关闭旧 branch 全部候选。按原职责是否已贡献计算新 r。新窗口首高为 `max(A.h+1, slot(act+T_init)+1)`，末高为首高加 `TB−1`，再按实际 `(β,r)` 调用 03 的静态排班。用 v10 规则处理 PENDING，默认停滞策略针对 NEXT。
- 复用 05 的 BranchClosed 和 SessionAssigned，触发真实新 session 准备、错时 Enter、生产与发布。正常场景的排班所选区域满足可用条件。 [v2 §8.1–8.4](../../../formalization_v10/poc_design_zh_v2.md#8-governance-agent可信组织的可执行策略) · [v10 Force](../../../formalization_v10/09_finality_rules_zh_v0_1.md#44-force)

Builder 拥有 `poc/crates/governance/` 和合同列出的 GFR Force 与 Manager 接线。共享公共接口由主会话集成。沿用 09 的真实交易/日志，11 保持批次服务路径独立。Verifier 写 `poc/verification/force/`。

## 验收

| 用例 | 独立断言 |
| --- | --- |
| v2 F1，地区停机或 Publisher 停止 | Espresso 继续且公开停滞达到阈值，真实控制收录后按 act 激活。用两类停滞各自的观察证据 |
| v2 F2，M 落后、材料未齐、q 前进但 θ 冻结 | 公共前提不足时 WAIT，重复、无效或未选中结果不会刷新进度 |
| v2 F3，Force 与旧结果或 SessionEnd 同键 | 控制先执行，activation anchor 来自此前完整重放，旧 G 保留，旧分支全部 cid 关闭 |
| v2 F4，Force 在途时 regular 或 Expire 已发生 | 同 branch regular 后仍按当前职责处理；过期 HALT 或 stale branch 的控制被记录为无效 |
| 预算与状态 | 贡献/未贡献的 r、插入一条激活前 G 新结果、PENDING 用例、失败交易、去重和 M 恢复；保留不满足可用性/时序前提的轨迹 |

正常成功用例必须有新分支首个合法结果进入 G。Tester 保存 detect、submit、include、finalize、activate、quorum-ready、first-regional-final、first-global-accept。向 13 交接治理控制状态机、消费位置以及实际 L1 → GFR → Manager 的运行入口。
