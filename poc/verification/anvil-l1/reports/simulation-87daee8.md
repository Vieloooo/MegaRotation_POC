# 功能09正式 Tester 结果

2026-11-16（Asia/Shanghai）。正式 Tester 使用命名角色 `bft_simulator`（GPT-6.1 Sol / high）。候选 `87daee8e9e77bbe27c337383d873a5eaf4a694ba` 的 A01–A17 在独立组件范围内 **PASS**。完整功能09仍为 **INCONCLUSIVE**，A18 和 A19 等待上游真实接线。

分支 `codex/poc-anvil-l1`，工作树 `/home/jy/.codex/worktrees/65ef/megaroation_poc`，测试前后 HEAD 相同且工作树干净。使用接口与独立预期提交 `daad412e46eeecd64ef9b8af8175e64ec40edc53` 和候选内的合同 `ANVIL-L1-ABI-v1 r1`。正式 Tester 在独立 Verifier 判定该源码候选可进入实跑后启动。

## 复现命令

在工作树根目录使用以下环境。输出目录要求是新目录。运行使用一个宿主全局锁和两个构建任务，读取已授权的任务专属 Cargo 缓存与数据盘目标目录。

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:/home/jy/miniconda3/bin:/usr/local/bin:/usr/bin:/bin"
export L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/l1-adapter"
export CARGO_BUILD_JOBS=2
out="$PWD/.build/artifacts/anvil-l1-simulator-87daee8-run1"
time flock /tmp/megaroation-poc-heavy.lock bash poc/crates/l1-adapter/tests/validate.sh "$out"
```

脚本使用固定 solc 编译 5 个 Solidity 文件，两份发布 ABI 与编译结果匹配。Cargo 构建及 A16 测试使用 `--offline --locked`。每个墙钟停顿值启动一个新鲜私有 Anvil，显式脚本 seed 为 `none (explicit script)`。复核命令从工作树根执行。

```sh
pyt=/home/jy/miniconda3/bin/python3
out="$PWD/.build/artifacts/anvil-l1-simulator-87daee8-run1"
for pause in 300 650; do
  "$pyt" poc/verification/anvil-l1/receipt_checker.py \
    --bundle "$out/pause-$pause/raw-rpc-bundle.json" \
    > "/tmp/anvil-l1-simulator-87daee8-receipts-$pause.json"
done
"$pyt" poc/verification/anvil-l1/golden_vectors.py check \
  --cast /mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0/cast \
  > /tmp/anvil-l1-simulator-87daee8-golden.json
for file in virtual-trace driver-trace raw-rpc-bundle; do
  cmp "$out/pause-300/$file.json" "$out/pause-650/$file.json"
done
cmp /tmp/anvil-l1-simulator-87daee8-receipts-{300,650}.json
"$pyt" /tmp/anvil-l1-simulator-87daee8-driver-check.py \
  > /tmp/anvil-l1-simulator-87daee8-driver-check.json
```

最后一条命令沿用独立 Verifier 的命令/RPC 检查脚本，仅替换证据目录。编译和运行参数及工具版本存于原始目录的 `validation.log` 和 `toolchain.txt`。实际 Python 为 `/home/jy/miniconda3/bin/python3`（3.12.8 / coincurve 20.0.0），Rust/Cargo 1.95.0，Foundry 1.0.0-stable（`e144b82070619b6e10485c38734b4d4d45aebe04`），solc 0.8.20（`a1b79de6`）。

## 观测结果

两次 Anvil 均为 Shanghai、chain ID 31337、FIFO、`--no-mining`，端口分别由各自运行分配为 52201、37455，仅限本机 loopback。Authority 位于 `0x28d5cad17734d0ac631273dcaae7d810b7332266`，Inbox 位于 `0x6dca7cc6c56bea1bc0aad0d10d8c4706f2fd54ed`。Anvil 命令、genesis、driver 配置、签名者与收据分别存于各运行目录。Slot 2 s，虚拟 finality delay 2,000,000 µs，公告窗口 6 s。两次均包含 block 0–7，Unix timestamp 1700000000–1700000014，最终虚拟参考时刻 16,000,000 µs，included/finalized/A/B 最终前缀均为 7。

| 实际项目 | 300 ms | 650 ms |
| --- | ---: | ---: |
| Header / 已执行挖矿 RPC | 8 / 7 | 8 / 7 |
| 命令与回复 | 195 | 195 |
| 收据总数 / status 0 / status 1 | 145 / 129 / 16 | 145 / 129 / 16 |
| 独立重建协议事件 / 幂等请求 | 11 / 5 | 11 / 5 |
| 历史 cursor / 精确事件读取 | 6 / 2 | 6 / 2 |
| 暂停后、首次 Mine 前的 head / 部署收据 | 0 / null | 0 / null |

虚拟轨迹、driver 原始 RPC 轨迹、原始收据包及独立检查器输出均逐字节一致。RPC 检查发现命令 152 的第四次挖矿已返回真实成功结果，driver 观察到丢答后返回 WAIT。另一条 Mine（153）被阻止。同 ID 152 重试只读取 head、block 和 receipts，完成前缀核对。命令 5 缓存重放与冲突重放均没有 RPC。

整个锁内验收命令本机耗时 12.649 秒，含两次墙钟停顿、构建、两次链运行和故障 replay。虚拟 16,000,000 µs 由脚本 tick 决定。网络时延未测。本次正式 Tester 无异常失败或超时。129 笔 status 0 均为预期拒绝/回滚，逐个由命名场景断言并用原始收据复核。故意的丢答和材料故障已通过 WAIT、REJECT 或重试用例。

## 矩阵

| ID | 结果 | 证据 |
| --- | --- | --- |
| A01 | PASS | 编译 ABI 匹配。Golden 检查通过 4 条签名消息、57 项字段或域突变、40 项非规范签名及 34 项 Cast 交叉检查。实际合约 digest 和独立签名恢复一致。 |
| A02 | PASS | Force 和 Resume 的 chain、合约、config、type、caller、rid 和字段突变实链收据为 status 0 且无日志。 |
| A03 | PASS | seq 0、跳号 seq 2、uint64 最大序号接纳，旧/重复/下降序号被拒。 |
| A04 | PASS | 5 次语义重试收据成功且无新日志，换有效低 s 签名仍返回原结果。冲突、错误签名及非规范 trailing/alias 布局收据为 status 0。 |
| A05 | PASS | 固定 mask/order 生效。非法 mask/order 拒绝。l1Floor 在字段突变检查中受签名保护，90000 的签名正例成功。 |
| A06 | PASS | 真实接纳 2-height 和 1-height 批次。submitter、完整 prev 字段、source、range 和 origin 用例按预期拒绝。 |
| A07 | PASS | 实际 Certifier / DA 签名校验覆盖错误 signer、域、commitment、替换 DA certificate 和非规范签名。 |
| A08 | PASS | 等于或低于 cursor 的恢复目标、未来 origin 和外部直接 Override 均为 status 0 且无日志。 |
| A09 | PASS | Set/Resume 在同一交易原子产生，RESUME(seq) 与签名 origin 一致。旧批次见证被拒，恢复后 1-height 批次成功。 |
| A10 | PASS | Authority 内部 Override 失败和独立 test-only fixture 内层写入后回滚均为实际 status 0、零日志。序号、请求标记和 cursor 状态复核，之后同请求成功。 |
| A11 | PASS | Block 4 收录 Force 和两笔 Resume，三个事件的 act 均为 1700000014。 |
| A12 | PASS | Block 4 两对 tx-local 坐标为 `(4,1,0)→(4,1,1)` 与 `(4,2,0)→(4,2,1)`，原始 RPC 全局索引分别为 1→2 与 3→4。 |
| A13 | PASS | 6 次 historical cursor、2 次精确事件读取和独立全链重放核对初始化、空块读、历史 written_at 及 A/B 一致性。 |
| A14 | PASS | 两个墙钟停顿结束后，首次 Mine 前 head=0、部署收据为 null。每个显式 slot 的实际 header 时间准确。 |
| A15 | PASS | Included、finalized、delivery 由不同命令推进。B 收到后续范围时有 gap，未来读取 WAIT，补齐后与 A 对同一前缀读结果相同。 |
| A16 | PASS | 使用真实收据包的 Rust 故障 replay 1/1 通过。缺 block、tx 明细、receipt 或 log 得 WAIT。完整材料缺 Resume、错误 emitter、全局索引、tx 或 parent 得 REJECT。前缀未误推进，补齐后同命令完成且无额外 Mine。 |
| A17 | PASS | 同 ID 缓存与冲突、真实 Mine 已执行后丢答、只读核对、两种墙钟停顿及三份原始/规范轨迹逐字节比较均通过。 |
| A18 | INCONCLUSIVE | 等待 02 真实 Clock/transport、grant/ack 与 03 公共对象/reader 映射，在准确集成 revision 做前缀及历史查询回归。 |
| A19 | INCONCLUSIVE | 等待 11 真实 DA/Certifier/G 来源，12 真实历史 ResumeCut 与 13 端到端恢复联调。 |

## 原始证据与界限

数据盘目录 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/09/artifacts/anvil-l1-simulator-87daee8-run1` 内含 34 份由 manifest 哈希的文件。每个暂停目录含 `driver-trace.json`、`raw-rpc-bundle.json`、`virtual-trace.json`、`result.json`、`diagnostic-rpc.json`、Anvil 启动命令和配置。可复制机器摘要、工具和完整文件哈希在 `/tmp/anvil-l1-simulator-87daee8-manifest.json`（SHA-256 `10ce07d7922f0c53f579a9f820acf80b28d036666bcc14ae79b9c60ab186bdb9`）。

| 文件 | SHA-256 |
| --- | --- |
| 两次 `virtual-trace.json` | `114bb15dc6c444da432ab5841dde21792ab8aa97032d0a7c42eb48a85b20b025` |
| 两次 `driver-trace.json` | `4729922497f770808b8088f596546c13b4ac5ca15ebefd77e8c589586e818561` |
| 两次 `raw-rpc-bundle.json` | `c4eee3e00c96b524e31802d71ab883764aa6733beb0686f2ec8b86194eb5426c` |
| adapter 可执行文件 | `7bf99a50f6f9035e609f5e0df6ea2bb5e2bca8f7447f5119da796fca79126c9e` |

结果限于固定脚本、私有新鲜 Anvil、可信本地 RPC 和内存命令状态。虚拟 finality 由脚本驱动。跨进程恢复、Ethereum 证明验证和公开网络未测。当前 gpos 为不透明字节，覆盖签名和来源绑定。实际分支重放、DA 持久性和派生切面属于 A18/A19。
