# 12 独立 D 派生与历史 ResumeCut

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以含 **03、09** 以及 **11 已冻结的 Inbox/DA 数据接口**的集成 SHA 建 managed worktree `poc-derivation`，分支 `codex/poc-derivation`。实现独立派生 D 与位置绑定的 ResumeCut。可先做规则与输入向量，联调使用 11 的实际提交。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 输入、状态与来源

Derive 依赖 finalized L1、真实 Inbox 接纳、可验证 DA、固定 `dep[h]`、metadata 和派生配置，从实际 D parent 计算执行和 native hash。Docer 核对仓库所含 MegaETH 快照 `a738f9b14c7880953d8cff8788db14e8e3e79124`、准备稿列出的固定 OP 源、v10 Derivation 和 v2 轻量化规则，先冻结 checkpoint/genesis 初始化、epoch 前进、epoch_has_output 依据和完整 input frontier 的可达向量。现有准备稿记录这些未决项。 [D 准备稿](../contracts/derivation-preparation-v1.md) · [v10 Derivation §5](../../../formalization_v10/09_finality_rules_zh_v0_1.md#5-derivation-rule) · [固定原生派生](../../../megaeth_specs/specs/rollup/derivation.md)

- 实现批次按实际 inclusion 的有效窗口、按实际 parent 执行、完整输入覆盖后的 deposit-only fallback，以及规定的 same-height substitution。多个连续 record 必须在更新的 parent 上逐条判断。已接纳 DA 未读或 mismatch 时在该位置等待。
- CursorSet 按实际规范位置顺序处理。等此前接受输入齐备后 reset，实际 D 未达目标时按规则 fill，D 已达到时保持原前缀和 head。保存每条 RESUME 的不可变切面并实现 `DeriveAtResume(p_res)`。切面绑定同交易的实际 RESUME 位置、恢复 anchor、prefix、state 和验证依据。
- 保持独立 D 数据接口与可读的 input/checkpoint 状态。给 13 提供 exact-cut API 和按共同高度比较实际 native hash 的输入，治理策略与 GFR 原子替换归 13。给 G′ reader 提供切面读取边界。

Builder 拥有 `poc/crates/core/src/derivation/` 和独立派生 runner。复用 03 的执行和 09/11 的真实数据对象。主会话处理公共模块和接口扩展，Verifier 写 `poc/verification/derivation/`。

## 完成条件

- origin 10、window 40 时，inclusion 49/50 过 lateness 谓词，51 被判迟到。改变 reader 下载延迟得到同一 inclusion 结论。完整窗口、缺 next-origin、epoch 尚无输出、已接纳但 DA 缺失分别有用例。
- 覆盖正常执行、有状态交易、fallback 改变 parent 后仍可采纳合格内容、规则给出的同高度替代与第二条记录重验。区分坏输入、未来记录、晚到和替代条件。
- CursorSet 覆盖 D 落后时按包含 L1 范围与真实 parent/origin fill，D 已更高时保持 head，例如 cursor=5、D=10、target=8。待前序 DA 读齐后再做 reset/fill。
- 同一 L1 block 中的两条 RESUME 各有切面，后续增长与 checkpoint/restart 都保持原历史输出。缺精确位置和缺已接纳数据的场景停在相应位置。
- Tester 用 11 的真实接纳 receipts 和 DA 做输入矩阵，把实际 D head、执行/状态向量、每条切面和失效用例交给 13。独立 checker 从公开 bytes、实际 D parent 与状态重算输出。
