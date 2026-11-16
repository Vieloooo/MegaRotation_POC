# 功能 16 正式本地兼容测试

2026-11-16（Asia/Shanghai）。**固定官方 dev-node 的可信本地 API 兼容范围 PASS。** 在复核通过的候选 `6248072b3fd0eee1ae468f0961a4d3589291cba6` 上，两条内容不同的 publication 均被真实节点收录。独立检查器从两个读取会话的原始响应重建同一连续前缀，三次离线回放的公共投影和审计字节完全一致。完整 PE 时间能力仍为 `UnsupportedTimingEvidence`。

Worktree 为 `/home/jy/.codex/worktrees/d2be/megaroation_poc`，分支为 `codex/poc-espresso-compatibility`。起点 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，正常合入 06 白名单交付 `4b034dfbb6c3e1c7143208347b546daab8dfe298` 后的基线为 `bcd30b0879e6c17afc177bfea76f7fee170cc069`。冻结合同和向量提交 `89925a6ed47cf8a57d6d052d8db192007019661c`，接口 `9747268e52410c1e7ce6b79d96df2ca858598a78`，引导 Anvil 合同澄清 `2ed1540b507af076b904816ff0cadc47d2e286ef`。正式测试 HEAD 等于受复核候选，crate 与场景路径干净，并行存在的未跟踪文件属于独立复核证据。Tester 使用 `bft_simulator`，运行时模型 `gpt-6.1-sol`，角色配置推理强度 high，运行时未单独公开强度。Seed 为 N/A。

## 正式运行与公开前缀

[冻结合同](../../docs/contracts/task-contract-espresso-compatibility-v1.md)指定官方 release `20260918`，源码 `fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171`，镜像 index `sha256:af73f55630690caa35aa0bc015d60dc03333791f966fdcc07b0798e33d4f768e`，linux/amd64 manifest `sha256:cfb845e49cb5369f77cd8bbda0b4fe4aad340793cfd130b7f2bb631e59d2db66`。本次原样执行核验后的镜像内 dev-node 与 Anvil 二进制，使用镜像内 loader 和运行库，协议 `0.3`、HTTP `/v1`、publication namespace `160016`。环境为 Linux x86_64、Bubblewrap 0.6.1、独立用户/网络/PID namespace，仅有 loopback，服务端口为 18770–18775。

| 用例 | 结果 |
| --- | --- |
| V01、V03、V04、V06、V08、V13 | **PASS。** 同一 source ID 下，两个会话均完整读取 q0–q5 的 block、leaf 和全部原生 flat 位置。q0–q4 的原生交易列表为空，q5 有 2 条。公开 `θ_q`（整数微秒）依次为 `0, 1794793206000000, 1794793208000000, 1794793209000000, 1794793210000000, 1794793210000000`，q4 与 q5 合法同键。 |
| V02、V04、V13 | **PASS。** 提交记录 19、20、21 均返回 HTTP 200。记录 19 与 20 的 payload 分别在 q5 原生索引 0、1 收录，命名空间投影索引也为 0、1。记录 21 的请求原始字节与 19 相同，返回相同交易哈希。q0–q5 中只观察到对应 q5/i0 的一次收录。提交 ACK 与随后从完整列表确认的收录位置分别保存。 |
| V05、V13 | **PASS。** 首个会话查询 `/v1/availability/transaction/0/0/noproof`，记录 57 保留真实 HTTP 404。该查询在回放中归为不可用。 |
| V07、V09、V13 | **PASS。** 首个会话与重新建连的第二会话分别提交 58、14 条 HTTP 记录。录制后运行首会话的 0/37 ms 墙钟停顿两组回放，以及第二会话回放。三份 `public-projection.json` 和 889 字节 `public-audit.bin` 逐字节相同。独立 Python 检查器在每组重建 6 个完整块、独立解码对照 6 份 raw payload、核对 2 条交易，再与候选输出比对，三组均通过。 |
| V12 | **PASS。** 专属 L1 `127.0.0.1:18775` 的记录 1 验证 `eth_chainId=0x7a69`（31337），dev-info 同时确认该显式 L1 URL。引导 Anvil 的原始 argv 为任务 wrapper 加 `-p 0 --slots-in-an-epoch 0 --balance 1000000`，wrapper 用镜像 loader 执行已验证镜像 Anvil。namespace PID 10、PPID 7、start ticks 517771399，实际分配端口和自身退出码未观测。专属 Anvil PID 3、Espresso PID 7 均退出 0，随后以 PID 和 start ticks 检查引导 Anvil 无残留。 |
| V01–V11、V14 fixture | **PASS。** 在最终候选锁定离线运行 8 个 crate 测试，包含乱序及缺口、原子拒绝、来源绑定、七类容量边界与明确时序缺项。独立 Python 核对 5 个冻结前缀向量。独立 Verifier 的 7 项 Rust、30 项离线用例及修复后的受影响路径复核见归档报告。fixture 证据分类独立于真实节点运行。 |

公开连贯前缀的 source ID 为 `EC16:13c2cc6d232444486b12122cca75024fd4d52d6a1793e95f197ac3d506fe3540`，审计 SHA-256 为 `43f106c905bb0877c2131bd7926eb099701d4e3936c0845e96ec7fb36a4e7cf5`。原始记录共 72 条，66 条 HTTP 200、1 条 HTTP 404 和 5 条未取得 HTTP 响应的启动探测。正式 runner 本地墙钟耗时 17.46 秒（含锁等待与构建、采集、回放），记录内最早请求至最后响应为 13.187 秒。Rust 测试命令实际墙钟 24.28 秒，测试 harness 报告 24.14 秒。公开 `θ`、本地 HTTP 观测、离线停顿和协议时序能力各自记录。详见[机器可读结果](simulation-results.json)。

## 首次运行与证据校验

初版候选 `8750535d8f02e920353c1ca827c78046e72d578a` 的 fixture 测试 8/8、冻结向量 5/5 通过。正式运行在 genesis 前以状态 101 退出，runner 返回 1，G-API 为 **INCONCLUSIVE**。固定 upstream 的 `TestConfigBuilder::default()` 会在设置显式 L1 URL 前启动引导 Anvil，初版 runner 的受限 PATH 未提供该命令。修订合同后采用镜像中已验证的 Anvil，由私有 wrapper 记录启动和清理。修复经过[受影响路径独立复核](runner-review-6248072.md)，上述真实结果来自修订后候选。

| 绝对证据目录（各自含 `SHA256SUMS`） | 文件数 | `SHA256SUMS` SHA-256 |
| --- | ---: | --- |
| `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-8750535-run01` | 21 | `427e4f883c0f2f8cbaf718e00b2e15b0c132003cab916f2c0c0ca2b78ebfe814` |
| `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-8750535-run01-tester` | 15 | `28e69eaf0c9cc88384a5a3d0b3ef0782cd986fdbdbe3650541b6277f3f0f2dac` |
| `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02` | 116 | `a983c8a2d8c70de3a0064b9614a08892f382feddca5795ec3062ae210153a88e` |
| `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02-tester` | 27 | `d6702d85e80d2bdc159c5f84fbe6a645258801072318436f41384ce4c5b0d599` |

运行目录的 `bundle/manifest.json` 引用原始 HTTP 请求/响应、镜像元数据和生命周期日志，SHA-256 为 `08bdc1aec6b5735df90ab3ceeab7a6fe13868fd0dc8212c78158ddbf2fe61dfb`。回放二进制 SHA-256 为 `dcffb26c638cf82c75b4ace68b2e888cf5ce38218c66a33c4712c94139d873d1`，独立检查器为 `e42e68df3f40c991dcce199ba6fa53d03fd2c6d75f243d0bf3ace9c25ead157a`。每份证据清单均通过 `sha256sum -c SHA256SUMS`，准确启动参数、离线 argv、工具版本和原始日志分别保存在 run 根及 Tester 目录。

## 复现

选择已复核的 `6248072b3fd0eee1ae468f0961a4d3589291cba6` 工作区。下面命令中的 `RUN_ID` 应为新的合法名称。正式原始命令使用 `ec16-official-6248072-run02`，失败尝试使用 `ec16-official-8750535-run01`，两次目录均已保留。

```bash
export PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH
export CARGO_HOME=/mnt/reth/jy/megaroation_poc/.build/cargo
export CARGO_TARGET_DIR="$(realpath .build)/target"
export CARGO_BUILD_JOBS=2
flock -w 600 /tmp/megaroation-poc-heavy.lock cargo test \
  --manifest-path poc/crates/espresso-compatibility/Cargo.toml --locked --offline -- --test-threads=1
poc/scenarios/espresso-compatibility/run-local.sh "$RUN_ID" 6248072b3fd0eee1ae468f0961a4d3589291cba6
```

`run-local.sh` 内部持有同一宿主锁。正式启动使用镜像缓存，运行中只访问 namespace 的本地端点。查阅 Tester 目录的 `commands.txt` 可逐条重放独立检查。它对首会话、37 ms 停顿的首会话与第二会话分别执行 `check_capture.py`，并一次检查冻结向量。

G-API、G-REPLAY 和本机原生 G-ENV 通过。官方 API 提供的区块位置和公共时间键可映射至 06 对象，`proposed_at`、`finalized_at` 和 `live_at_proposal` 保持 Unknown，完整 PE-LOG-WIRE-1/PE-READER-1 导出维持 Unsupported。共识 QC、VID、Regional FINAL 验证以及 06 完整接线超出本次实证范围。流式接口仅做固定源码核对，未验证现场流式读取或重连。该有界观测只覆盖本地墙钟节点运行和停机后的离线回放。
