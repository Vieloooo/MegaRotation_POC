# 功能 02：独立验收准备

日期：2026-11-15。状态：**PREPARATION ONLY；waiting-dependency=01**。这里没有功能 02 的运行时通过结果，没有实现候选，没有仿真。

基线为 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`；worktree 为 `/home/jy/.codex/worktrees/0b8e/megaroation_poc`；分支为 `codex/poc-sim-communication`。Verifier 使用实际启动的 `bft_verifier` 命名角色；主协调读取本轮 turn context 后确认模型/effort 为 `gpt-6-astra` / `xhigh`。

按 `regional-bft-conformance` 执行的顺序：

1. 只读 v2、v10、已冻结 I1/接口附件/native recovery 合同、固定 Regional BFT DESIGN 和 Commonware Clock/p2p 源。没有先看功能 02 的新合同或 PoC 实现。
2. 向主协调发送独立规则与 E/A/L/O/C/T/S/R 向量框架，记录来源先于实现的边界。
3. 只读定位基线 native hub/config/network/driver/wire、I1a 和现有验证入口；定位结果不用于生成期望。
4. Docer 释放唯一 writer 后，将既已推导的 [来源映射](source-map.md)、[独立期望](expectations-v1.md)、[静态输入](vectors-v1.json) 和 [计划矩阵](planned-matrix.md) 落盘。
5. 之后才阅读 Docer 新合同并另记对照结果；对适配提案的条件化期望不追溯冒充来源规则。

`vectors-v1.json` 是语义输入表，包含手写数值、符号身份、触发条件与预期。它不是最终 protobuf 编码、签名 fixture、可执行 scenario 配置或已运行 trace。符号 `m1`、`cid-A` 等不能当作密码学哈希；03 的公共对象及 01 的恢复接口尚待协调决议。

所有 runtime/重型测试命令均为 **planned / NOT RUN**。本阶段只允许 JSON 解析、交叉引用、静态字面算术与文件范围检查。相应记录在 `static-check.json`；其中 PASS 仅指准备材料的一致性，不是功能通过。未来运行结果必须来自包含已接受 01 的同一命名候选 SHA。

未来原始证据目录由主协调指定为：

```text
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/{verification,simulation}/sim-communication/<candidate>/<case>/
```

本阶段未创建该目录、迁移缓存、运行 Rust 构建或启动 native worker。Builder/Tester 未启动；完整功能 02 及总体 I1 仍开放。
