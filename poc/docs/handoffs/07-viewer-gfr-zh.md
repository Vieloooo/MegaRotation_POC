# 07 Viewer GFR 与独立公共重放

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以 **03 的集成 SHA 和 06 已冻结的公共日志接口**建 managed worktree `poc-viewer-gfr`，分支 `codex/poc-viewer-gfr`。实现纯 viewer GFR、regular/Expire 和独立 A/B reader，实际证书验签接在 04 的接口提交上。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 规则与本次交付

实现 `Init / AdvanceGlobal / Chain / Responsibility`。输入为已验证的 finalized Espresso/L1 前缀、公开对象、结构化材料与状态。返回新 ReplayState、已完成记录、LifecycleEvent、Tail 和具体处理位置。可用有限的真实可核对 L1 fixture 做 regular；为实际控制事件预留已冻结入口，Force 和 RESUME 分别在 10 和 13 扩展。确保不完整的前缀和本阶段尚未实现的控制被显式报告。 [v10 GFR §3–4](../../../formalization_v10/09_finality_rules_zh_v0_1.md) · [v10 G-S1](../../../formalization_v10/11_global_chain_safety_liveness_zh_v0_1.md)

1. Docer 固定按公开时间键、`L1 control → Expire → SessionEnd → Espresso`、同类索引的总序、ReplayPosition 编码和原子提交点。对即将执行的事件检查 Espresso watermark、完整 L1 控制覆盖和实际验证输入。缺材料时返回 `WAIT{next_position,missing}`，保持原事件位置。协议 timer 不由 viewer 的本地时钟触发。
2. 按已验证的最终证明和从选定 anchor 后到目标的完整 physical 正文计算最早 `q_avail`。公开历史未齐的候选暂时不合格；已公开但读者缺材料时 WAIT。对缓存候选保留早先 q_avail。截止用公开键，整段 Expire 用当前接纳键逐 height 检查，包含 padding。支持由后代最终证明覆盖同链祖先，仍核对每个实际 physical 正文。
3. 实现满窗口 immediate handoff、SessionEnd fallback、同 Espresso block 的重复接纳、Expire/HALT 与历史 replay/checkpoint。正确定义最后已接纳 physical anchor、reference block 和后继固定窗口。在同一 replay position 比较 G／来源／Tail／lifecycle／q_avail。明确多份合法 proof 的 canonical 比较投影。
4. 用 03 的 shared types、Plan 与 Merge，接 04 的公共原生 CheckFINAL 路径。给两个 reader 各自独立的 cursor、ReplayState 和下载调度，输出可供 08 和 14 查看。Verifier 用公开原始对象和从规范写出的期望检查每个转移。

Builder 拥有 `poc/crates/core/src/gfr/` 及合同列出的纯重放 reader 模块。主会话集成公共事件、共享模块与证书 adapter。Verifier 写 `poc/verification/viewer-gfr/`。07 完成后把 GFR 模块的所有权按 10、13 的顺序交接。

## 完成条件

- 逐项用缺 Espresso watermark、缺 L1 完整覆盖、已有公开但本地缺数据、公共历史未公开、无效 proof 和不延续前缀测试。校验 WAIT 原位置和保持已完成前缀，补齐后给出同一转移。
- 真实证书和正文向量覆盖 proof 先公开、body 后公开、缺 physical ancestor、多证明异序、候选先公开后选中，以及截止等号与任一新增 height 已 Expire 的拒绝。
- 测 h10 满窗、h9 后 SessionEnd、无新 physical 时沿用初始 anchor、同一 Espresso block 中符合条件的连续接纳。后继依固定窗口推进。Expiry/HALT 保留已接纳 G。
- Tester 执行 G1/G2，两个 reader 用不同下载顺序、独立状态和原始输入重放。相同位置输出一致，checkpoint 后能继续验证。交付可离线重放 fixtures、可独立审查的规则输出及 reader API。
