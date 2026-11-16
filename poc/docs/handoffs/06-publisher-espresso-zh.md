# 06 Publisher 与确定性 SimEspresso

> **2026-11-16普通运行路线修订：** Publisher/日志/reader的普通运行接线不等待完整01；消费02普通调度和03/04接口即可。BFT节点崩溃恢复与Publisher自身持久恢复分别记录，公开关闭/截止/WAIT规则和受控输出不变。 详见[不重启worker的交接路线](../commander/restart-free-handoff-track-20261116.md)。

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以含 **02、03** 的集成 SHA 建 managed worktree `poc-publisher-espresso`，分支 `codex/poc-publisher-espresso`。用受控通信交付持久 Publisher、确定性 Espresso 公共日志和逐读者 finalized 前缀。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 规则与输出接口

Publisher 按 `(cid,H(B))` 持久保存验证、待提交、已提交、已收录和停止状态。Offer 核对最终证明、exact anchor provenance、physical ancestry 和正文；缺来源时保留任务等待。已提交在核到完整 finalized publication 后才变为 RECORDED。公开关闭/排除或截止 watermark 使后续新重试停止。Program 的 TIME 停止保留待发布任务。复用 03 的编码与验签接口，08 对实际区域结果使用 04 的真证书校验。 [v10 Publisher §3](../../../formalization_v10/04_publisher_zh_v0_1.md)

SimEspresso 用 SimHub 的参考时钟和独立 TimestampPolicy 产生 `(q, θ, messages)`，记录 proposed、finalized 和逐 reader received。公开对象可以拆成 proof/body，多次 publication 可引用早先公开的正文。提供已验证完整 finalized 前缀与读者本地可见位置。独立读者的延迟、丢包和重试经过 02 的调度。06 输出公共日志接口供 07、08、16 使用。 [v2 §4.5、§5.3](../../../formalization_v10/poc_design_zh_v2.md) · [v10 公开输入](../../../formalization_v10/09_finality_rules_zh_v0_1.md#32-事件)

## 所有权与验收

建议 Builder 仅拥有 `poc/crates/services/src/publisher/`、`poc/crates/chain-adapters/src/sim_espresso/` 和对应回归。由主会话安排这两个模块的公共声明与链 adapter 类型，避免与 09 或 11 共改路径。Verifier 写 `poc/verification/publisher-espresso/`。

| 独立案例 | 预期记录 |
| --- | --- |
| 缺祖先正文、wrong cid/body/proof 或断链 | WAIT 或可核对的 REJECT，正常 Offer 以受校验的来源入队 |
| crash/restart 贯穿 enqueue、submit、finalize 与回包 | 原任务和去重键恢复；验证日志前保持 SUBMITTED，重复确认沿用 finalized 位置 |
| TIME、公开 SessionClosed、选定另一 cid、截止 watermark | 只有公开排除或截止路径停止新重试，保留历史收录 |
| 同 θ 多 q、冻结 θ 仍增加 q、lead/lag、超界、倒退 | 公共排序、watermark、时序假设失效和无效源分别可复现 |
| 同 proof/body 公共对象经不同 reader 延迟/丢失 | 各 reader 的覆盖位置独立，具备补齐后的一致公共日志；不泄漏未来条目 |

Tester 固定输入和 seed 做墙钟扰动对照，比较日志与 publication 状态、已交付前缀和实际重试。交付 C3 输入向量、公开日志 codec、已验证日志示例、待发布记录及 reader API。GFR 的 q_avail 与接纳结论由 07 独立推导。
