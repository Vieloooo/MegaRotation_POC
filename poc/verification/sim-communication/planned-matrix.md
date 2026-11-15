# 未来执行矩阵（全部 planned / NOT RUN）

本阶段只准备静态向量。以下命令未运行；`run_matrix.py`、`check_trace.py` 在本目录 **尚未实现**，其 CLI 是待冻结提案，不是现有工具。01 accepted SHA 与跨任务接口决议到达后，先确认候选包含该 SHA；Verifier 与 Tester 使用同一命名候选并记录 dirty 状态、源码/配置/二进制/hash、真实 PID、active engine、channel、结果与原始证据。

未来所有 Rust build、重型检查、仿真都在外层使用同一 flock，`CARGO_BUILD_JOBS=2`，不嵌套加锁。示例中的 `<candidate>`、`<case>`、`<new-output>` 是必须替换的占位符。

| Scenario ID | 静态向量 | 未来实际证据与执行方式 |
| --- | --- | --- |
| endpoint-domain | E-01–E-08 | 15 replica 注册和附加服务/reader；不同 region 同 member；真实 vote/certificate/resolver/body 通过 native route；真实 PID/active engine/脚本贡献分列。planned runner。 |
| ack-atomicity | A-01–A-08 | 全批有效/非法末尾、duplicate/unsolicited/recovered Ack、完整 recovery frame bound；Hub adoption 全观测集及 root journal 边界。planned runner。 |
| link-order-faults | L-01–L-10 | FIFO、跨 link/channel、same tick、64/65 queue、分区边界、恢复 drain、重排模式、匹配冲突与计数器边界。planned runner。 |
| reliable-custody | O-01–O-10 | 三 crash、ACK 丢失、sender确认/删除持久边界、receipt/同ID mutation、closed cid、m1/m2 retry顺序、overflow保管、native repair区别。planned runner；真实 restart/new PID。 |
| clock-c1-c2 | C-01–C-10 | offset两端、request/sample/response分段、序号污染/有效样本乱序、half-open lease、新 incarnation校准、固定ClockView。planned runner。 |
| original-timers | T-01–T-06 | 原绝对 deadline、renew/cancel/rearm/restart、stop时工作完成、原operation due。planned runner并回归01 R3/R7–R10/R14。 |
| services-readers | S-01–S-07 | reader独立前缀/gap重启、public time、actor crash、admin与诊断隔离、wall watchdog、ACK custody≠应用接纳。planned runner；服务fixture明确标签。 |
| replay-regression | R-01–R-05 | 每seed启动/墙钟四组合、既有I1a/Fresh/01、负trace、错时Enter、历史证明。planned runner及下列现有入口。 |

拟定功能 02 runner 命令（**planned；文件和 CLI 尚不存在**）：

```sh
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin python3 poc/verification/sim-communication/run_matrix.py --candidate <candidate> --vectors poc/verification/sim-communication/vectors-v1.json --seeds 7,42,99 --wall-pauses-ms 0,2 --start-orders forward,reverse --output /mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/verification/sim-communication/<candidate>/<new-output>
```

独立 checker 命令（**planned；文件和 CLI 尚不存在**）：

```sh
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 python3 poc/verification/sim-communication/check_trace.py --vectors poc/verification/sim-communication/vectors-v1.json --run <new-output> --mutations --output <new-output>/checker-report.json
```

现有回归入口的计划调用（入口存在，但本阶段 **NOT RUN**；原有 seed7/旧目录不复用为新结果）：

```sh
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin ./poc/scripts/bootstrap.sh test --locked --offline
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin ./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline -- --test-threads=1
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 .build/target/debug/sim-hub --scenario healthy --seed 7 --output <new-i1a-output> --wall-pause-ms 0
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 python3 poc/verification/i1/check_trace.py <new-i1a-output> --mutations --output <new-i1a-output>/independent-report.json
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 .build/target/debug/native-hub --scenario healthy --seed 7 --output <new-fresh-output> --wall-pause-ms 0
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 python3 poc/verification/native-engine-fresh/check.py <new-fresh-output> --output <new-fresh-output>/independent-report.json
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 .build/target/debug/native-hub --scenario recover-pending-verify --seed 7 --output <new-recovery-output> --wall-pause-ms 0
```

未来 target 位于 02 专属数据盘目录时，以该已记录绝对 binary 路径替代上述 `.build/target/debug/`；不得误用其他 job 的二进制。完整 01 的 R1–R18 回归命令由接受交接提供，以上单个 `recover-pending-verify` 仅是已有 bounded case，不能替代全部恢复验收。现有 Fresh `run_verification.py` 含旧候选 artifact 位置，不能直接把它的成功当作新候选回归。

矩阵至少 seeds={7,42,99}×wall={0,2}×start={forward,reverse}，每seed内部按相同逻辑配置比较；纯负例为具名静态 fixture，不借seed掩盖未覆盖分支。多seed不是一个跨seed字节相等要求。单个向量不能保证全部恢复分支；最终报告逐项记录 PASS/FAIL/INCONCLUSIVE、前提、观测截止、缺失输入、命令和真实证据。

所有命令的 cwd 必须为 `/home/jy/.codex/worktrees/0b8e/megaroation_poc`。准备阶段未授权这些执行；需要先接收01接受的集成SHA和总协调完整实现授权。
