# 13 治理 RESUME 与历史切面恢复

> **2026-11-16普通运行路线修订：** 可先完成BFT worker持续运行的RESUME：独立D真实fallback/分歧或合规HALT/D推进，恢复输入后按真实L1位置对齐历史cut并启动新cid。无需01，但不能直接G=D；延迟/暂停场景不冒充原U1实际Batcher crash/restart，原完整矩阵另保留。 详见[不重启worker的交接路线](../commander/restart-free-handoff-track-20261116.md)。

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以含 **10、11、12** 的集成 SHA 建 managed worktree `poc-resume`，分支 `codex/poc-resume`。完成真实 L1 RESUME、按位置绑定的原子 G 对齐、Manager 接续及首个新结果进入 G。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 实现

1. 扩展 10 的治理状态机。M 自己核对共同高度实际 native hash 分歧（HASH_DIVERGENCE）或已经 HALT 且 D 按规程越过冻结前缀（HALTED_D_ADVANCED）。复查 Espresso watermark、L1、批次服务、DA、anchor/目标区域准备和一次至多一条未决控制。普通 lag 或共同高度仍一致的较高 D 作为对照。
2. Docer 写清目标高度求解和取整，基于已观察的 G/D/Inbox、收录前推进上界、预计收录 timestamp 区间和 v2 安全余量。选择最小满足 `h > H_guard` 与 `t_L,max + T_prep ≤ ts*(h) ≤ t_L,min + T_drift − ε` 的合法整数 h。检查 `T_prep ≥ M_a` 和 `M_a + T_ready + δ_G ≤ T_prep + b_L2`，区间无解时记录理由。收录超界仍按实际公共记录处理。
3. 通过 09 的真实交易收录并绑定 RESUME 与 CursorSet。GFR 等公共覆盖和 12 的 `DeriveAtResume(p_res)`，以该记录对应切面原子替换 G 到目标；D 保持独立前缀。绑定 anchor、真实 origin/state、新 `(β+1,0)` 和固定窗口。错过可启动条件时 G 已对齐且 Tail HALT；同键 Expire 与 SessionEnd 依全序处理。
4. 与 05/07 的事件合同核对 Manager §3.10 和 GFR §4.6 的相反书写顺序。若尚无冻结结论，由 Docer 明确同一重放事件的原子输出顺序。核对旧分支全部关闭、恢复材料在分配/准备时可用、Preparing 缺输入可重试。接入 G′ 的 CursorSet 历史来源和独立 A/B reader。

Builder 在 10 已集成基础上拥有合同列出的 `poc/crates/governance/`、GFR/Manager 的 RESUME 模块以及 11 G′/12 历史切面的接线。合约规则沿用 09，公共接口变更由主会话统一。Verifier 写 `poc/verification/resume/`。 [v2 §8.5–8.6、§9.3](../../../formalization_v10/poc_design_zh_v2.md) · [v10 RESUME](../../../formalization_v10/09_finality_rules_zh_v0_1.md#46-resume) · [Manager 事件接口](../../../formalization_v10/03_session_manager_zh_v0_1.md#310-applylifecycleevent)

## 实验与完成条件

- **U1** 注入 Batcher crash，让真实窗口回退造成共同高度 hash 分歧。M 由自己的观测决定，等所需服务恢复后提交。对照服务尚未恢复的 WAIT。
- **U2** 验证已 HALT 且 D 越过冻结前缀的应急路径，以及 D 仅落后或高但共同高度一致的对照。
- **U3** 覆盖区间有解、无解、实际收录超预算、低但合约可接纳的目标以及过期等号。旧 G 允许发生合规后缀变化，已完成对齐可伴随 HALT。
- **U4** 同一真实 L1 block 两条 RESUME 按不同位置保存历史切面，加一条前序已接纳 DA 尚不可读的对照，保持原位置 WAIT，读齐后原子转移。A/B 在相同位置比较。

正常成功需 **G 已对齐指定历史切面，且新分支首个合格结果实际进入 G**。Tester 记录 detect 到 first-global-accept 的各点、原始 L1 和 DA、旧/新 G、D、所用切面和观测前提。向 14/15 提交三种读者轨迹、恢复启动入口和已验证 SHA。
