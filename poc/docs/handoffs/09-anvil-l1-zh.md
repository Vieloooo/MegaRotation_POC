# 09 Anvil L1 driver、ControlAuthority 与 BatchInbox

> **2026-11-16普通运行路线修订：** 受控L1普通接线可用02正常运行路径+03对象推进，无须等01。主动Force、RESUME合约操作可用授权M和真实本地EVM测试；不能把合约接纳等同GFR激活，也不能把服务driver自身重启能力与BFT恢复混为一项。 详见[不重启worker的交接路线](../commander/restart-free-handoff-track-20261116.md)。

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以含 **02、03** 的集成 SHA 建 managed worktree `poc-anvil-l1`，分支 `codex/poc-anvil-l1`。交付受虚拟时钟调度的真实 Anvil 链、两个 P0 合约以及 finalized L1 reader。提前与 11、12 核对 ABI 和数据绑定。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 已有基础和本次交付

现有 Foundry/Anvil 环境仅通过 counter smoke，已记录版本和校验值。该任务在真实 EVM 执行并读回交易和日志；公共控制生效的判断由 10/13 的 GFR 处理。 [Foundry 记录](../setup/foundry-2026-11-15.md) · [v2 §7](../../../formalization_v10/poc_design_zh_v2.md#7-第一阶段-l1保留控制不实现-staking) · [Authority §3.9–3.10](../../../formalization_v10/01_l1_authority_zh_v0_1.md#39-submitforceu-sig) · [Inbox §3](../../../formalization_v10/07_batch_inbox_zh_v0_1.md#3-接口)

1. L1 driver 是唯一手动出块者，按 SimHub 参考时钟和确定性 slot 设置整数秒 timestamp。区分 submit、include、EVM success、模拟 finalize 和 reader delivery。读者只看自己的完整 finalized prefix，包含真实 receipts 的 `(block,tx,log)`，缺历史材料时 WAIT。时间与日志接入 02，读者边界与 03 的 fixture 接口兼容。
2. ControlAuthority 用固定治理 signer、control_seq、request 去重和静态 policy。签名绑定 chain ID、合约地址、控制类型和实际输入。Force 控制记录的 act 从实际收录 timestamp 加 `M_a` 计算。SubmitResume 在同一笔交易中执行 Inbox.Override 与记录 RESUME，目标 height 高于 cursor、frames_digest 清零、origin 来源可核对。
3. BatchInbox 验证固定 submitter 权限、Certifier 签名、DA 绑定、完整 prev cursor 和来源位置。实现 `CursorAdvanced`、`CursorSet` 和历史 cursor 读接口。Cursor 的 source 与收录后取得的 written_at/receipts 保持分开。
4. Docer 与 11/12 冻结 PoC 的 Solidity ABI、签名域、cursor/DA/attestation 字节、config/policy、genesis cursor、历史读取和逐 event 的验证依据。将同交易的 RESUME 记录与 CursorSet 位置绑定。合约处理授权和原子状态，治理方处理活性预算。

Builder 拥有 `poc/contracts/` 与 `poc/crates/chain-adapters/src/l1/`，锁定实际 Foundry、solc 和依赖。主会话集成 Rust/EVM 接线、Cargo 和公共 L1 对象。分别预留 10 使用控制、11 使用 Inbox、12 使用 finalized events 的接入范例。Verifier 写 `poc/verification/anvil-l1/`。

## 通过标准

- 真实 Anvil 可以在墙钟暂停期间保持未出块，在授予虚拟事件后得到确定性 header 时间与日志顺序。验证成功/失败 receipt、完整 block coverage、读者不同 delivery 前缀及历史 cursor。
- 对错链、错合约、错类型、签名逐字段篡改、无权限、重复请求、错误序号、stale prev、错误 DA/Certifier 和低目标运行拒绝用例。
- SubmitResume 的 Override 和控制日志一起成功，任一检查失败时两者都回滚。真实同一 L1 block 的多个控制按 tx/log 有稳定位置与实际 act。同交易绑定和 rollback 均从真实状态与 receipts 检查。
- Tester 交付部署/测试命令、genesis/ABI/签名向量、固定版本和 hashes、编排与读者 replay 证据。由合约提交侧、finalized reader 和独立检查器分别确认读取边界。
