# 05 Session Manager 与多 cid 生命周期

> **2026-11-16普通运行路线修订：** 无BFT进程重启的Manager阶段可先接入04健康profile：内存内A→B→A必须保留原engine/view/签名历史和timer；不以重建空实例代替Pause/恢复。OS重启后持久终止/同cid恢复另列01依赖，不能阻塞其余生命周期接线。 详见[不重启worker的交接路线](../commander/restart-free-handoff-track-20261116.md)。

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以 **04 的集成 SHA** 建 managed worktree `poc-session-manager`，分支 `codex/poc-session-manager`。交付每个 worker 的 Session Manager 和真实 native 候选生命周期。可提前写状态机合同和独立样例。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 来源与实现边界

v10 定义了 Manager 的入口、准备、调度、公开选定和终止。原型只提供基础 stop/retire 和固定 cid 适配。该任务以可验证的公共 lifecycle/record 与 regional result 向量接入真实进程。08 接上真正的 GFR 和 Publisher，13 接上真实的 ResumeCut。 [v10 Manager §3–4](../../../formalization_v10/03_session_manager_zh_v0_1.md) · [regular 多 cid 迁移](../contracts/regular-preparation-v1.md)

## Docer 要明确的行为

- 按 cid 保存唯一 descriptor。生命周期为 Assigned、Preparing、Ready、Running、Paused、Terminated，选择状态为 Candidate、Selected、Excluded。Ready 验证 exact anchor、来源和值状态；单给同高度或 root 不足以 Ready。
- 允许准备同一已选前驱的合法后继候选。同一逻辑 session 在每个 worker 上最多调度一个候选。较高候选未 Ready 时原候选可继续。切换到新候选时停车可获选的原 cid，选回时恢复原 view、共识状态、签名历史、原 deadline 和待完成工作。重复 Enter 只重放原结果。
- 消费 `SessionAssigned`、`SessionClosed`、`BranchClosed`、ClockEvent 和 IncompatibilityFact。公开选定须精确匹配连接身份。关闭先持久化 Terminated 再 Close，覆盖尚未 Enter、Preparing 和 Paused。已形成的 Final、Evidence 和发布任务按协议继续保留。
- Manager 使用共享 NodeClock 做本地 stop。GFR 的公开定时事件仍取决于公开重放。D 判死核对共同高度的实际 native hash 并传递给相应后继。真实 D 接线在 13。
- 与 07/13 明确生命周期的有序事件和原子消费边界，特别是 Manager §3.10 写 `Resumed` 保存材料后再分配，GFR §4.6 的文字次序相反。冻结接口时明确该边界。真实 RESUME 转移和全流程测试归 13。

## 所有权与通过标准

Builder 拥有 `poc/crates/core/src/manager/` 以及由 04 交出的 native driver/replica 的生命周期接线。worker 保持一个 Clock、executor、grant sequence 和 root journal。各 cid 独立持有 engine、signer cache、storage/witness policy、archive 与 pending jobs/timers。合同注明共享 I/O 和同一 Ack 中多 cid 的提交边界。主会话拥有公共事件、模块声明和 wire；Verifier 拥有 `poc/verification/session-manager/`。

Tester 验收真实 worker 的 A→B→A 切换、重复 Create/Enter、选中未就绪候选、迟到结果、关闭 Preparing/Paused 后收到材料、Clock 到 stop、崩溃后恢复原候选或保持终止、D 共同高度冲突及单纯 ahead/lag。核对原工作和计时标识、持久日志、新旧签名来源与历史结果保留。至少一条用例经过真正的 native parking/恢复。向 08 交付 Manager 生命周期驱动示例和发布输出接口。
