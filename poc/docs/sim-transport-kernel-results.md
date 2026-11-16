# 功能02：独立运输内核交付

2026-11-16，Asia/Shanghai。**有限组件验收PASS；完整功能02未accepted，整体I1仍开放。** 本交付完成独立运输内核的授权范围，未修改现有native、I1a、公共types/proto或根workspace。机器可读索引见[delivery JSON](sim-transport-kernel-delivery-v1.json)。

## 修订与角色

worktree：`/home/jy/.codex/worktrees/0b8e/megaroation_poc`；分支：`codex/poc-sim-communication`；宿主：`xuelab_jump`。未合入main、未推远端。

| 阶段 | 准确提交 |
| --- | --- |
| 交接基线 | `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b` |
| 首轮准备 | `125d8cf4e30b5affe3cd731fbb5d4a73d0e40a10` |
| 正常合并分层调度授权 | `c64709871827128aae974343bc28bdafb9f89486`，包含授权 `0a951c482a9712678ab9de34765b781b9c3c9f42` |
| 冻结接口与独立预期 | `444613345e045e861f94298ac68042050bb8e4db` |
| 首次实现候选 | `bc4111aab6240b364a1a9c632d1bf02474a57272` |
| 独立harness及失败证据 | `c749973e9c1bb9e6625eb4521ec89f50bcb6fbee` |
| Snapshot修复 | `653a0654ba1765cb600e603250efa0ee40d85aeb` |
| **最终验证与正式Tester的同一候选** | **`4fd6f115bd969b0a8f252d7933edd21c27f00a8f`** |

最终生产crate Git tree为 `3d2ff54ca9231d2611e424599b0db73170faa9a2`。之后的交付记录提交只更新文档/结果索引，不改变该生产树或已测试harness。Regional BFT gitlink始终为 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。

实际turn_context已核对：lead为`gpt-6-astra/ultra`；Docer `bft_spec`、Verifier `bft_verifier`及只读复核角色为Astra/xhigh；Builder `bft_builder`、正式Tester `bft_simulator`为`gpt-6.1-sol/high`。单writer串行交接，Rust build和重型测试/仿真使用全宿主同一flock，jobs=2。

## 可消费组件

[sim-transport crate](../crates/sim-transport/README.md)是独立workspace，含自己的manifest/lockfile。直接依赖固定serde1.0.229、serde_json1.0.154、sha2 0.11.1；实际Rust/cargo为1.95.0。

- Typed Endpoint与opaque Message/Envelope：身份域、权限、context、原始字节/摘要、原发送位置、attempt及当前incarnation分开。
- RouteKernel：确定性profile匹配、FIFO/显式Unordered、jitter/tie、Hold/Drop/duplicate、整数payload pacing、容量、原子plan/commit、stale/cross-instance拒绝与snapshot校验。
- ReliableMailbox：显式plan/adapter持久确认/释放、原retry、ACK、接收去重、消费标记、tombstone及有界保留。receipt真实性、fsync和root恢复仍由真实adapter提供。
- ClockTracker/ClockView纯算法及[GrantConsumer接口](sim-transport-api-v1.md)：供06/09与fixture使用同一类型，reader仅收到自己的opaque交付。

完整功能原64条期望保留；新增46条组件向量与来源先行记录在[独立验证目录](../verification/sim-communication/kernel-v1/README.md)。具体SC/K映射见[coverage](../verification/sim-communication/kernel-v1/coverage.md)。

## 独立发现与验证

初次候选42项独立测试为40 PASS、2 FAIL，见[原报告](../verification/sim-communication/kernel-v1/review-bc4111a.md)。恢复接受不可能的raw_due，以及tombstone跳过原retry约束，均作为P2局部不变量问题修复。另补pacing字段上界，并使用u128避免近整数上限的合法状态被误拒；API、snapshot字段和黄金记录未改变。

修复后的Builder测试10/10通过。最终候选上独立harness **46/46 PASS、fmt PASS**，包括原负例、完整黄金记录、资源界、实例隔离、空/非空队列pacing、过去/未来fault范围与近u64控制。旧失败证据逐文件校验保留。

最终Verifier原始报告：

```text
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/verification/sim-communication/kernel-v1/4fd6f11/final-verifier.json
SHA256 697e699a9a0f40a2958016f14c167aeb42adb056e9138eec8484a91650a3572a
```

该JSON保存准确命令、每项测试、harness/binary/artifact hash、开始/结束状态。harness aggregate SHA256为 `0a6918330be068cab59815c92d6a1b8e57ccbbdd2f1c6112bea788e776befd50`。原始bc失败目录及653复核目录均位于同一verification父目录，未覆盖。

## 正式12组矩阵

Tester在同一`4fd6f11`重新构建example binary后执行seed `{7,42,99}` × 配置声明顺序 `{forward,reverse}` × 墙钟暂停 `{0,2}ms`，**12/12 PASS**，全部CLI正常退出，无timeout/incomplete。每seed四组完整logical trace逐byte相等，未删协议事件。

| Seed | 四组共同的完整trace SHA256 |
| --- | --- |
| 7 | `73c3adc7c16489ee914e822d767a1de4f05b0176b5f3d855ec44b1d8217e083e` |
| 42 | `77ab1819569654b7b9a9496eb3ab52191217ca3f66c9ab57b726d48f8046dbed` |
| 99 | `02dac34bc05409f29b23fe2d53a8ffc20436a4c5a978d6488c33c69a92585e8b` |

每组观察到首次attempt在虚拟1,000,000us丢弃，1,020,000us重试，原/重复副本分别在1,030,000/1,030,003us交付；logical inbox仅插入一次、最终consumed=true、Reader A=1/B=0、sender retry=None。声明顺序是配置排序变量，native engines/workers均为0。

本机CLI墙钟：0ms暂停组13.802–14.331ms，2ms暂停组21.712–23.065ms；build0.312s，锁内build+矩阵约0.885s。它们是本机组件执行成本，没有真实网络时延测量。

```text
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/simulation/sim-communication/kernel-v1/4fd6f11/
manifest.json SHA256 64a1ecdb4a496a7820cfaf0317f3a6ed43e74ed491b7d4dbf32bd09ad6b288f4
replay binary SHA256 63e3503c214338997a7e0c93ccf8cf0f898154e5fd2a4b24ea2a9a9e28ec4939
SHA256SUMS SHA256 48e19904a7888386c5835546c03e46a6122ec2c3733171dccba1774a4074e50a
```

目录含准确`command.txt`、`run-matrix.py`、manifest、summary、全部12组argv/PID/exit/stdout/stderr/trace/diagnostics；lead复核71/71文件checksum通过。运行前后HEAD/clean及Verifier报告hash相同。

## 复现入口

cwd固定为本worktree。先使用命名候选和新的输出目录；以下重放示例不覆盖本次证据，完整原命令见上述JSON与command.txt。

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH"
export CARGO_HOME=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/cargo
export CARGO_TARGET_DIR=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/target/sim-transport
export CARGO_BUILD_JOBS=2
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/verification/sim-communication/kernel-v1/Cargo.toml --locked --offline -- --test-threads=1
flock /tmp/megaroation-poc-heavy.lock cargo build --manifest-path poc/crates/sim-transport/Cargo.toml --locked --offline --example replay
# 把NEW_OUTPUT替换成新的绝对目录；parent须存在。逐个seed/order/pause组合运行。
flock /tmp/megaroation-poc-heavy.lock "$CARGO_TARGET_DIR/debug/examples/replay" --seed 7 --declaration-order forward --wall-pause-ms 0 --output NEW_OUTPUT
```

## 剩余集成gate与所有者

1. **01/02真实root与恢复。** 接收01准确接受SHA及文件移交，接入真实receipt、trusted floor、当前peer认证、同owner串行和release ID去重；保留terminal/signed/timer状态，复跑受影响R1–R18、Fresh/I1a。已持久计划stale不能丢弃义务。
2. **03/05/02动态cid授权。** 当前permits固定、Rebind仅改incarnation。未来cid须由03公共对象、05合法准备/生命周期及02受控权限更新接入；版本化迁移要保留旧队列/outbox/dedup/历史权限，不能重建空kernel或预知未来cid。
3. **03/02配置绑定。** 运行清单需冻结opaque Message.config、完整kernel_config_hash、run、codec、registry之间的对应关系；注册与root恢复一并核对。两类config不互相冒充，不构造自引用hash。
4. **06/09服务接线。** GrantConsumer仍需真实target/step/ClockView授权及完整root Ack。业务状态与MarkConsumed需同一持久事务；真实Anvil副作用需命令身份、崩溃后结果核对与幂等边界，不能盲重放已出块调用。
5. **06/09 reader及运行预算。** 公共前缀、ReplayPosition、q/θ、L1 finalized由对应模块解释；02只负责各reader的实际opaque交付。固定permits、8192 tracked IDs和有限inbox/tombstone保留上限须进入实验预算，本轮不证明无限session或无界运行。

接口/组件可供下游按准确SHA消费。上述是分层接线与完整验收依赖，组件结果不增加完整accepted功能计数，不替代真实Clock RPC、BLS/native四channel、多进程或G/Inbox/D结果。
