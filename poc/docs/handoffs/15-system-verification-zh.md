# 15 15-worker 全系统验收与可重放证据

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以含 **13、14** 的集成 SHA 建 managed worktree `poc-system-verification`，分支 `codex/poc-system-verification`。完成 v2 全系统矩阵、启动和离线 checker 入口、可重放证据包。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 范围

Docer 把 v2 §12 的每个编号映射到命令、配置、假设、注入点、输出和成功界。明确区分 native recovery R1–R18 与 v2 regular R1/R2。在同一集成代码上执行 I1/I2、C1–C3、S1/S2、R1/R2、F1–F4、U1–U4、G1/G2、W1。对顺序敏感场景启用固定 seeds、墙钟暂停/启动顺序扰动和公共输入到达差异。核对参数不等式与活性适用区间。独立兼容的真实 Espresso dev-node 由 16 验收。 [v2 §12–13](../../../formalization_v10/poc_design_zh_v2.md#12-参数场景与验收)

Builder 拥有 `poc/scenarios/system/`、任务分配的 supervisor/场景 runner 和证据采集入口。提供一条命令启动本机系统、一条运行命名场景、一条离线检查。Verifier 拥有 `poc/verification/system/`，从原始日志、签名、body、交易/DA、状态和交付重建独立判断。组件问题由总协调安排相应所有者修复、回归、重集成并更新实际候选 SHA。

## 完成条件

1. 生产三个完整的 15-worker 展示。regular 以正确后继首个合格结果进入 G 结束；Force 以真实控制激活并新分支首结果进入 G 结束；RESUME 以 position-bound 对齐并新分支首结果进入 G 结束。至少含一条涉及有状态交易、DA 读取和实际链分歧的恢复。
2. 记录 detect、submit、include、finalize、activate、quorum-ready、first-regional-final、first-global-accept。证明观察、区域 body 验证交付、公开完整、G 接纳、Inbox 接纳和 D 派生分别记录。A/B 以相同 replay position 比较，独立 checker 对证书域、签名 quorum、路由、时间、前缀、等待位置、窗口、hash 和实际交易顺序核对。
3. 对记录做 mutation，至少改变公共顺序、q_avail、原 deadline、异常前缀、重复发布、具体 RESUME 位置或 native hash，验证 checker 可拒绝相应错误。
4. Tester 提供完整矩阵、每个场景 PASS/FAIL/INCONCLUSIVE 和实际前提；缺持续参与、E-L/E-TS、数据或时序预算时按 v2 的适用范围归类。用例数、seed、运行命令、集成 SHA、Rust/Commonware/Anvil 版本与二进制哈希和受限 scope 写入结果。G-S2 仅用于无 RESUME 区间，G-S3 仅用于从 genesis 无 RESUME 的执行。
5. 交付 Web 可读的三条演示轨迹、原始公开证据、经过验证的 manifest、失败复现和复跑说明。最终报告分清虚拟时延、墙钟耗时和未测项。签名测试私钥留在本机任务目录。
