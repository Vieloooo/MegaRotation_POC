# 01 原生同 cid 崩溃恢复与 I1 收口

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 的 `36eff1c76d5c4df5d0790133c0e14719c0c73ebd` 上创建 managed worktree `poc-native-recovery`，分支 `codex/poc-native-recovery`。以新进程恢复同一 cid，完成现有 R1–R18 合同和 I1 验收。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。按[共用工作约定](workflow-zh.md)返回提交与证据。

## 当前基础与规则来源

`recover-pending-verify` 已在 seed 7 恢复同一 Verify 工作与原认证截止时间，场景结束时交付数为零。基线复跑 129 项 native 测试通过。已有崩溃遗留 head scratch 的修复和独立用例。冻结的恢复合同 §2 定义了允许修改的精确路径，§12 定义 R1–R18，后续修订仍适用。Docer 从这些条款列出缺口和实现切点，Verifier 以现有独立期望为起点补齐真实进程用例。 [恢复合同](../contracts/task-contract-native-recovery-v1.md) · [独立期望](../../verification/native-recovery/expectations-v1.md) · [实现与复跑](../native-recovery-implementation.md)

## 实现与所有权

1. 先复核并修完 R1。确认新 PID、incarnation、真实 Archive producer、原 WorkBinding/receipt、原 certification deadline、整个 engine 的 startup fence。保留当前对照场景与 Fresh 兼容结果。
2. 恢复 proposal mailbox、pending build/Verify、OFFERED/CONSUMED、Close/取消和 Waiting/Closed actor phase。原 timer、retry、latch 和 floor 保持可追溯，检查恢复前、恰好、恢复后已到期的行为。涵盖 R2–R10 与 R17。
3. 完成中断 root transaction 与丢失 Ack 的授权和对账。Docer 明确合同允许的处置路径与输出去重边界。覆盖 root/aux/witness floors、prepared cache、原签名 bytes、精确 cached Ack 和未决 outbox。R11/R15 使用真实持久化切点，R12–R14 对照损坏与回滚输入、startup fence、Close、stop 和 lease expiry。
4. 在 R16 丢弃真实 certificate resolver response，重启 requester，执行新的 native 请求并消费实际匹配的响应。核对请求编号、generation、RNG、原 view deadline 与可恢复的上下文。真实 RESOLVER 流量和效果单独记录。

Builder 只修改恢复合同 §2 及已批准修订列出的原型、vendored hook、patch 和测试路径。共享 proto/类型和依赖由主会话集成。Verifier 写 `poc/verification/native-recovery/` 的独立用例和报告。02 的路由推广接续本任务的集成提交。

## 验收与交接

```sh
./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline
./poc/scripts/bootstrap.sh build --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline --bin native-hub --bin native-worker
run_dir=$(mktemp -d)
.build/target/debug/native-hub --scenario recover-pending-verify --seed 7 --output "$run_dir/replay"
```

Tester 在独立审查通过的集成 SHA 上完成 R1–R18，真实重启用新进程。对顺序敏感的场景用 seeds 7/42/99、0/2 ms 墙钟停顿、启动次序对照。用 mutation 检查错误 provenance、提前发布、错误 deadline 和重复签名。复查 Fresh/I1a 受影响的既有行为。报告保留未完成场景与失败证据。所有冻结的恢复和回归验收通过后，由主会话关闭 I1，并向 02、03 交付集成 SHA、恢复范围和矩阵。 [完成定义](../contracts/task-contract-native-recovery-v1.md#13-completion-and-limits)
