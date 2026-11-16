# 功能 11 正式组件测试

2026-11-16。**合同 B01–B16 PASS。** 在独立 Verifier 放行的源码 `e5f77d27edde91a61f7400d1a61961e4cbe09c9c` 上重跑实际 Anvil、进程恢复和离线独立检查。两轮各取得 2 次原始 Inbox 接纳、3 笔链上回滚，覆盖 11 个角色进程退出点。B17 的 12 派生消费与 B18 的整合验收为 **INCONCLUSIVE**。

| 项 | 值 |
| --- | --- |
| worktree / 分支 | `/home/jy/.codex/worktrees/f025/megaroation_poc` / `codex/poc-batch-services` |
| 集成基线 | `c9805f7a818bc23d30018ce4dc6f1ee45f92e3ac` |
| 03 / 09 | `e9c15e04bc83b335668f3ecd837367aa87344a89` / `c443ae925ed549b046731c2d8ef3dd9ce0311d24` |
| M1 对外接口 | `75b574d40930b7c9c748a4c6b6e381117c937537` |
| 合同 | `BS-COMPONENT-v1 / r2` at `b9b8b1595ffd2a95fbdd779a843a4119ed59bba8` |
| Tester | 命名角色 `bft_simulator`，实际 GPT-6.1 Sol，角色配置 high |
| 运行起点状态 | HEAD 与放行 SHA 相同。生产及依赖路径与 HEAD 一致。未跟踪路径为并行编写的独立验证器及其报告，原始状态见 `worktree-status.txt`。 |
| 场景种子 | `none (explicit script)`。两轮使用同一脚本、本地测试签名身份及配置。 |

## 两轮结果

300 ms 与 650 ms 两组均在私有 loopback 上启动真实手动出块 Anvil。每组从 genesis 到 L1 高度 6，产生 7 个完整区块头、8 份真实收据。两个原始 `CursorAdvanced` 绑定高度 1–2 和 3。一次同请求成功重试没有新增接纳事件。Override 写入 `(5,0,0)`，同交易配对的 RESUME 位于 `(5,0,1)`、序号 0。旧 prev、错误 Certifier signer 和替换 DA commitment 三笔交易实际进入区块 6，收据状态均为 0。

独立检查器从收据恢复出正确的 signer、commitment、Calldata 与 DA records，并比较 3 个已接纳高度的 Kept 及 4 个 fixture 高度的生产 poststate/native hash。高度 1 保留交易 0 与 3，并过滤错误 nonce 和余额不足。高度 2 保留空普通交易记录。高度 3 复用同一 gpos，经前序高度的实际执行状态得到后续交易。高度 4 仅用于来源执行和旧 prev 负例。

每组有 119 条显式 L1 driver 命令、20 次与命令相符的修改 RPC（8 次广播、6 次设定块时间、6 次挖矿）、8 笔独立验签的原始 EIP-155 交易。查询在尚未收录、尚未 finalized、尚未交付给 reader 三个位置分别返回 WAIT。终点虚拟参考为 14,000,000 μs，included、finalized 与三个 reader 的可见前缀均为 L1 高度 6。L1 header 时间从 genesis 1700000000 秒按 2 秒 slot 步进，脚本采用 2,000,000 μs finality delay。300/650 ms 是实际墙钟注入参数，整轮命令墙钟含等待共享锁为 6.32 s。两轮均在各自的 90 s runner 超时内结束。RPC 为 same-host loopback，未采集独立网络时延。

每组注入 3 个 DA、2 个 Certifier、5 个 Batcher 持久阶段 `_exit(91)`，以及 1 个真实广播后的 Batcher `SIGKILL`。所有退出后的文件哈希与同 durable root 重开后的一致。掉回包的交易保留原始 raw bytes、nonce 1 和 tx hash `0xafefb53b872022136c81f62e0b01a5ff65db66a6e332c0e5532c46f3a5fad92b`，先以 `SubmittedUnknown` 等待，随后以原 finalized 接纳事件完成 reconcile。两笔 Accepted 和一笔 Reverted 均留在持久 journal 中。两个 Anvil 端口 34821 与 33599 已无 listener，进程表无该运行的 Anvil 或组件进程。

脚本规定的 `protocol-trace`、`driver-trace`、`raw-rpc-bundle`、`accepted-records` 和 `execution-source` 五份 JSON 两轮逐字节相同。另核对独立 RPC 轨迹、fixture 快照、DA 索引、Override 后接纳视图也逐字节相同。故障轨迹仅在比较时排除操作系统分配的 `pid` 和 `reopen_pid`，其他字段完全一致，规范化 SHA-256 为 `be9cd6c55980f124328adc7841dd976a34bb84caa5745bcffb7346432bdc1aa4`。原始收据包 SHA-256 为 `98854d8360645b9d451e6181173fe69a73d9cb62ddbaf5842e68e72037b14f2b`。

## 合同核对

| ID | 状态 | 证据 |
| --- | --- | --- |
| B01 | PASS | 冻结 frame 向量、真实 DA frame 和独立 Kept/poststate 比较，含 padding。 |
| B02 | PASS | 截断、尾随字节、格式、连续性、范围、计数与长度的冻结负例。 |
| B03 | PASS | 真实 EIP-712/secp256k1 向量及字段变异，附真实 Anvil 的错误 signer 和 commitment 负例。 |
| B04 | PASS | 同内容重试与重开保持字节和签名，缺失、不可用或损坏返回 WAIT，写入拒绝覆盖损坏对象。 |
| B05 | PASS | DA 写 bytes 前、bytes 持久化后、receipt 持久化后的三个真实退出点。 |
| B06 | PASS | 独立角色读取并执行 03 fixture，缺材料 WAIT、矛盾 pre-state/内容与 stale prev 拒绝。 |
| B07 | PASS | 真实缓存后 cursor 推进与 Override 检查，未处理精确 RESUME 返回 WAIT。后来 GLOBAL cursor 仍关联前一 RESUME 的用例使用历史 fixture。 |
| B08 | PASS | Certifier 签名持久化前后两个真实退出点，加身份及配置不匹配拒绝。 |
| B09 | PASS | Batcher 五个持久阶段真实退出并重开，覆盖重复和冲突命令、原始任务与终态。 |
| B10 | PASS | raw 持久化后和真实广播后分别注入退出，原始 nonce、hash 与字节保留，未知状态等待并恢复原接纳。 |
| B11 | PASS | 真实 BatchInbox 接纳两个连续批次，逐一独立打开 immutable DA bytes，G 来源标记 fixture。 |
| B12 | PASS | 实际收录、finality、交付相互独立。相同请求的成功重试、失败收据均不产生额外接纳。另用实际收据重放 fixture 验证 4 个缺失 WAIT、5 个篡改 REJECT，修复后仅推进一次。 |
| B13 | PASS | Override 前生成的旧 prev 提交在 Override 后链上回滚。Certifier 对旧 prev 拒绝、对未重放精确 RESUME 等待。 |
| B14 | PASS | 真实接纳后损坏或暂不可用 DA 返回 WAIT，复原后保持 commitment、字节和位置。 |
| B15 | PASS | 缺 gpos 时只获知可物化部分。缺位置绑定 ResumeCut 时旧前缀只作历史，当前视图报告缺少 `(5,0,1)` 对应的 cut。 |
| B16 | PASS | 三角色独立故障边界，两组墙钟参数的协议输出、收据和持久证据一致。 |
| B17 | INCONCLUSIVE | 已提供单一生产接口、实际 L1 收据与 DA。12 的独立 D 执行／恢复消费仍待其验收。 |
| B18 | INCONCLUSIVE | 02 对接、07/08 实际 G/gpos、完整 G′、12/13 的位置绑定 cut 和 15 worker 端到端仍待集成。 |

运行覆盖 Linux 同机进程退出和同一文件系统上的 durable root 重开；断电、内核崩溃、任意硬件故障、完整 BFT worker 重启，以及 A-DELIVER/CERT-5/DA-RET 的活性与时间预算均在此证据范围之外。固定四高度 G fixture 的 typed 位置是 `Espresso(2)`、key `5000000` μs，`gpos` 字节为 `poc11/fixtures/gpos/1`。

## 复现与样本

运行路径为 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1`。Rust/Cargo 1.95.0，Foundry 1.0.0-stable `e144b82070619b6e10485c38734b4d4d45aebe04`，solc 0.8.20 `a1b79de6`，Python 3.12.8 与 coincurve 20.0.0。工具、组件 runner、锁文件和原始证据哈希见 [manifest.json](manifest.json)。完整命令从 worktree 根执行，复跑时传一个新的输出路径。

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/batch-services"
export CARGO_BUILD_JOBS=2
export L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0
/usr/bin/time -f 'elapsed_wall_seconds=%e exit_status=%x' flock /tmp/megaroation-poc-heavy.lock \
  bash poc/crates/batch-services/tests/validate.sh "$PWD/.build/artifacts/m2-simulator-e5f77d2-run1"

export PYTHONDONTWRITEBYTECODE=1
verify="$PWD/poc/verification/batch-services"
out="$verify/reports/simulator-e5f77d2"
for pause in 300 650; do
    run="/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-$pause/run"
    /home/jy/miniconda3/bin/python3 "$verify/check_accepted_evidence.py" \
      --bundle "$run/raw-rpc-bundle.json" --da-index "$run/da-index.json" > "$out/accepted-$pause.json"
    /home/jy/miniconda3/bin/python3 "$verify/check_execution_evidence.py" \
      --source "$run/execution-source.json" --accepted-result "$out/accepted-$pause.json" > "$out/execution-$pause.json"
    /home/jy/miniconda3/bin/python3 "$verify/check_m2_driver_and_crash.py" \
      "$run" > "$out/driver-crash-$pause.json"
done

export L1_ADAPTER_RPC_BUNDLE="$PWD/.build/artifacts/m2-simulator-e5f77d2-run1/pause-300/run/raw-rpc-bundle.json"
/usr/bin/time -f 'elapsed_wall_seconds=%e exit_status=%x' flock /tmp/megaroation-poc-heavy.lock \
  cargo test --locked --offline --manifest-path poc/verification/batch-services/consumer-m2/Cargo.toml \
  --test reader_faults -- --ignored 2>&1 | tee "$out/reader-faults.log"
cp "$out/reader-faults.log" "$out/reader-faults.txt"
```

给 12 的 [真实 L1 收据包](/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run/raw-rpc-bundle.json) 和 [DA 索引](/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run/da-index.json) 均由同一已验证修订生成。

| L2 高度 / L1 位置 | Keccak commitment | DA bytes / SHA-256 |
| --- | --- | --- |
| 1–2 / `(2,0,0)` | `0x3719037b49943e3c4dfdb7488fd54eb94cbfa8b849e4e8339ea5f03cc7ca3ef2` | 117 / `f84fd2986e31829e19bbad01784c5d4701d43febea0d42ed8fd3877073437e5c` |
| 3 / `(3,0,0)` | `0x5629543d719715cc831f03dd53c111bd0112681dd05eee06ae084f92f1e9448d` | 74 / `dc09763e46982ed64c6f28e94af3163e6cbad48fefc4a8b6413f16e732aa7b93` |
