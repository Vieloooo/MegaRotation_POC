# 准备合同独立对照

状态：**准备材料审阅；没有运行时候选审查；waiting-dependency=01**。

Verifier 首先向 lead 报告从固定来源形成的规则框架，随后只读定位基线实现；获 writer 后先落盘自己的来源/期望/60条静态向量，之后才读取 [SIM-COMMUNICATION v1](../../docs/contracts/task-contract-sim-communication-v1.md)。对照合同 SHA-256 为 `9242760a1dacd559e6a5ac629b0604d8626a0cdd7f3ed5345b76971596d3e1a5`。没有 Builder 或 Tester，没有实现 diff 可审查。

源先行初稿的内容 hash（只标识本次阅读顺序中的已落盘版本，不是已提交候选）：

| 初稿 | SHA-256 |
| --- | --- |
| `vectors-v1.json`（60条） | `593cedd02541af1e43c0d1654c6b0164b06f65cd6ee73d60542cbcde4b05ede7` |

对照后补充 O-09：sender 收到有效 receive ACK 后，在自身 durable confirmation/delete 前与后崩溃；预期分别保留可安全重试的 m1 与已确认空 outbox。该补充来自原 `V2-NET/RECOVERY` 保管规则，与合同 SC-08 对齐，标记为 post-contract coverage refinement。还显式展开 E-04 的 epoch/namespace/digest、A-01 的 timers/samples、O-03 原 first receipt 字面值及 T-05 stop±1μs；不从现有实现输出生成结果。

lead审阅后进一步补充 O-10（同ID不同payload/context/origin）、C-10（合法seq5先到、合法seq4迟到）和S-07（reader重启保留prefix/gap）。E-04的合法基线明确为region1内的replica5→replica6/native.vote，再逐字段变异。最终共64条向量；这些是来源规则的边界展开，不是候选测试结果。

| 合同 gate | 独立向量 | 对照结论 |
| --- | --- | --- |
| SC-01 | E-01–E-03 | 15地址和服务注册；pid/member分离；实际PID/engine覆盖另列。Q5仍待冻结。 |
| SC-02 | E-07–E-08、O-08、S-06、R-05 | 真正native四channel与resolver，证据/完整交付分离。 |
| SC-03 | E-04–E-06、E-08 | domain/source/destination/incarnation/长度/摘要；历史入口单列。 |
| SC-04 | A-01–A-08 | Hub全批原子性及受控重放；不承诺回滚此前授权native I/O。 |
| SC-05 | L-01–L-04、L-08 | 默认attempt FIFO、跨link/channel交错、显式同link故障重排。 |
| SC-06 | L-05–L-10、O-07 | 容量、分区端点、drain和attempt溢出；Q3算法仍条件化。 |
| SC-07 | O-01–O-02 | durable enqueue/发送后未ACK两类sender crash。 |
| SC-08 | O-03、O-09 | receiver持久dedup与丢ACK；sender确认/删除自身durable边界。 |
| SC-09 | O-04–O-06、O-10、E-05、R-05 | 错ACK、篡改同ID与终止后的历史运输，禁止重签/复活。 |
| SC-10 | C-01、C-09 | C1 offset/RPC分离与逐次实际读钟。 |
| SC-11 | C-02–C-04、C-07–C-08、C-10 | C2丢失/乱序/序号污染/continuity，grant不续租。 |
| SC-12 | C-05–C-06、T-05 | lease/stop±1μs、expiry跨恢复、永久终止。 |
| SC-13 | T-01–T-06、O-03、R-04 | 原timer/operation/first receipt/Enter和physical重绑定。 |
| SC-14 | S-01–S-02、S-07 | reader缺口前缀、public key与local time分开；reader restart保留prefix/gap。 |
| SC-15 | R-01 | 每seed四个wall/start组合；完整逻辑trace。 |
| SC-16 | S-03–S-05 | 脚本fault、越权admin和基础设施失败分开。 |
| SC-17 | R-02 | 同候选I1a/Fresh/01回归，01完整命令尚待交接。 |
| SC-18 | R-03 | 独立恶意trace拒绝，不调用被测状态迁移。 |

没有发现来源硬规则与新合同的冲突。以下保持未决，不作为本阶段PASS：Q1 accepted01及恢复/custody接口；Q2 03公共canonical与02外层wire；Q3完整link匹配、jitter/tie/online/drain；Q4可靠receipt/dedup/容量/GC及逻辑FIFO消费需求；Q5真实engine/service覆盖；Q6公共路径与准确回归命令。

默认transport-attempt FIFO及无额外end-to-end logical FIFO与独立期望一致；公开reader连续prefix由独立gate保证。receive ACK确认可恢复custody，不能替代proof/body/业务验收。此结论是文档对照，**功能02运行验收仍为INCONCLUSIVE / NOT RUN**。
