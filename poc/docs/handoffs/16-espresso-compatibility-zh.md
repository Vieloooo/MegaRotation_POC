# 16 官方 Espresso dev-node 独立兼容

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以含 **06 冻结公共日志接口**的集成 SHA 建 managed worktree `poc-espresso-compatibility`，分支 `codex/poc-espresso-compatibility`。验证选定官方 Espresso dev-node 的 API、public log adapter、录制和离线重放。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 范围与来源

v2 主测试使用确定性 SimEspresso，另列真实 dev-node 兼容。Docer 先从固定官方源选择能在目标机使用的 release/commit 或镜像 digest，写明实际支持的 submit/finality/query/stream、public position 及 timestamp 语义。确定如何将真实来源映射到 06 的接口，同时报告缺失的 API 或可信输入假设。 [v2 §13.1](../../../formalization_v10/poc_design_zh_v2.md#131-顺序)

Builder 拥有 `poc/crates/chain-adapters/src/espresso/` 和 `poc/scenarios/espresso-compatibility/`。使用该任务的本地进程与独立端口做兼容测试。公共 adapter 接口由主会话登记，06 的确定性 runner 仍供主线使用。Verifier 写 `poc/verification/espresso-compatibility/`。

## 完成条件

- 发出可辨识 publication，录制真实 submit 响应、finalized blocks、读取位置和 payload。区分提议、确认和 reader 会话位置，核对 q、时间键/来源以及引用的正文。记录实际网络、时钟、认证和环境行为。
- 验证读者第二次查询、缺数据、重复/连续前缀及离线 replay。两次读取同一录制文件时比较公共对象与可复核的输出摘要。
- Tester 返回固定版本、启动/调用命令、原始 transcript、转换向量、可重放记录和兼容矩阵。对外部时钟控制与实时性质只报告实际观测；录制后的离线可重放与 dev-node 墙钟运行分别报告。
