# 11 crash-only Batcher、Certifier、DA 与 G′

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以含 **08、09** 的集成 SHA 建 managed worktree `poc-batch-services`，分支 `codex/poc-batch-services`。完成真实 Inbox 提交链路、crash-only 服务和已接纳视图。与 09、12 冻结 Inbox/DA 接口后可与 10 并行实现。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 服务接口与所有权

1. **Batcher** 以实际 finalized cursor 和指定公共 gpos 的 virtual 段构造单完整 frame，每批 1–2 个连续 heights。使用有状态交易与 03 执行产生的 Kept，padding 保存空普通交易。编码、长度、连续性和首末位置向 12 提供可核对向量。
2. **Certifier** 维护独立公共重放及 L1 cursor，在验证对应 G 前缀、payload 和 DA 时等待缺失输入。用真实配置签名覆盖完整 BatchAttestation，包括 chain、Inbox、prev/next、DA、范围和 gpos。按 v10 处理尚未重放当前 cursor 对应 RESUME 和 stale prev。
3. **DA** 只在不可变 bytes 持久化后出具对应 commitment 的 receipt。带同 commitment 重试和 restart 可返回相同数据；unavailable 与 commitment mismatch 可观察，读取端验证。故障为独立服务的延迟/崩溃/恢复，已确认数据跨重启保留。
4. **Inbox reader / G′** 使用 09 的真实 finalized successful receipt 和配置好的合约地址。普通接纳通过具体 gpos 物化公开前缀。CursorSet 对接按 L1 位置的 ResumeCut，12 提供切面，13 接入恢复链路。缺来源时给 cursor 和实际可物化前缀，并报告待补材料。

Builder 拥有 `poc/crates/services/src/{batcher,certifier,da,accepted_view}/` 和独立 actor 接线。各角色有独立状态、收件箱、权限和故障控制，使用 02 调度。主会话同 09/12 确定编码、ABI 及 L1 位置信息，09 的合约和 12 的派生状态机由各自任务持有。Verifier 写 `poc/verification/batch-services/`。 [v10 Batcher](../../../formalization_v10/05_batcher_zh_v0_1.md) · [Certifier](../../../formalization_v10/06_certifier_zh_v0_1.md) · [DA](../../../formalization_v10/08_da_zh_v0_1.md) · [Inbox/G′](../../../formalization_v10/07_batch_inbox_zh_v0_1.md#37-读取者的接纳视图-g) · [派生准备稿](../contracts/derivation-preparation-v1.md)

## 完成条件

- 真实提交至少两个批次，覆盖会改状态的普通交易、无效 nonce/balance 过滤与连续段。每次以实际 Inbox finalized receipt 标记 Accepted，G′ 绑定指定 gpos。区分提交、合约接纳、数据能读及 D 执行。
- 从原始字节独立验证 frame、commitment、DA receipt、attestation 字段和签名。替换字段、错 signer、坏 commitment、stale prev、越界区间与重放均有对照。
- 在各角色关键持久化/发送边界 crash/restart。验证已持久步骤和身份可复用，丢回包与延迟重试不造成重复接纳。Override 后的旧 prev 已签批次无法入 Inbox。Certifier 自己的公共 prefix 落后时 WAIT。
- Tester 交付可复现的已接纳 L1 receipts、原始 DA bytes、身份与签名公开向量、角色故障轨迹及 G′ 查询方式。12 可消费同样本重放 D，13 和 14 可使用该服务与诊断接口。
