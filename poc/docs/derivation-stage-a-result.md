# 功能12：独立 D 与精确 ResumeCut 阶段 A 交付

2026-11-16（Asia/Shanghai）。**冻结阶段 A 组件 PASS；完整功能12保持 INCONCLUSIVE。** 无frame依赖的实现、独立复核和正式测试均已完成。11准确接口SHA及真实payload/DA样本尚未收到，实际批次矩阵留待续派；不增加完整accepted功能计数。

## 提交、路径与角色

| 项目 | 精确值 |
| --- | --- |
| 初始干净基线 | `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b` |
| 消费03 | `e9c15e04bc83b335668f3ecd837367aa87344a89`，总协调已接受 |
| 消费09 | `c443ae925ed549b046731c2d8ef3dd9ce0311d24`，A01–A17组件范围 |
| 顺序正常合并后的基线 | `37b4f2b8a6d67b303ba90b1d4d1981c93ed8dd12` |
| 合同及来源独立预期 | `4a86f8d5c6f2d39e52994bb7c862fcc7313ccf8c` |
| 小接口提交 | `3b1b34fce6f3a3bac5099d53bdf5f7df56079f80` |
| 源码候选／Verifier／Tester | `d0afeee9363ef6703f51eecfd742915f83aa27f2` |
| Worktree | `/home/jy/.codex/worktrees/29df/megaroation_poc` |
| Branch | `codex/poc-derivation` |

最终归档提交只增加本文和独立验证材料、更新接口记录的当前状态，不改变上述已复核和测试的源码候选。源码完整比较及两份manifest的源码/工具hash均已再次核对。

主协调的自身turn metadata确认 `gpt-6-astra` / `ultra`；实际使用 `bft_spec`、`bft_verifier`（Astra/xhigh）、单 `bft_builder`、`bft_simulator`（GPT-6.1 Sol/high）。先冻结来源与独立期望，再实现；源码先提交，Verifier和Tester检查同一SHA。只读explorer仅核对09 runner复用。角色与启动核对详见[接口记录](derivation-interface-v1.md)。

任务使用已有managed worktree，没有第二个worktree或新增用户chat。所有子agent已停止计算。源checkout只读；未reset、改写历史、合入main或push。Regional BFT gitlink保持 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，Commonware pin保持2026.10.0。相对集成基线的实现、合同和初始向量20个文件均在12允许路径；后续归档文件也仅在12文档与verification目录。

## 已实现的组件

- 独立workspace `poc/crates/derivation/` 及自己的lockfile。复用03的Metadata、DEP、Apply、native/physical/virtual对象和编码，没有共享types/core/根Cargo修改或第二种frame编码。
- `DriverSource`拥有09 driver，只接受指定reader成功读取的完整finalized、delivered区块。header可用、收据完整和派生处理完毕分别记录；有限03 fixture是独立且有界的来源路径。
- 从实际D parent执行完整窗口后的deposit-only fallback。窗口按实际L1 inclusion及完整输入覆盖判断；不读G、Espresso、墙钟或全知调度器。
- CursorSet整条marker暂存，输入齐备才原子提交fill/reset/cut。目标低于实际D时保持D前缀，target anchor/state仍来自目标高度。
- 精确`(block,tx,transaction-local log)`索引的不可变ResumeCut，另存RPC全局log索引作为证据。同区块不同RESUME分别冻结，后续增长不改变已有切面。
- checkpoint恢复从genesis重放到准确动作位置并比较状态、输入及切面依据，支持更长可用输入前缀和不同reader；不把本地checksum当成语义验证。
- 任意真实CursorAdvanced均在原位置返回`Feature11AcceptedPayloadRequired`。不以未知payload、外层cursor或缺数据为由越过接纳输入补空块。

`G0`（genesis信任锚不计本派生器首块）、`L0`（03配置与真实09来源分开绑定）及whole-marker暂存是明确的PoC决议。固定MegaETH快照仍标注未激活/不完整，本交付不声称完整原生兼容。规则、来源hash和限定见[冻结合同](contracts/task-contract-derivation-v1.md)与[独立预期](../verification/derivation/source-expectations.md)。

## 独立复核与正式测试

[Verifier报告](../verification/derivation/candidate_review/review-d0afeee.md)给出DA01–DA18逐项范围和来源映射，无源码finding。[Tester报告](../verification/derivation/simulation/tester-d0afeee.md)对同一源码SHA另开新目录完成正式矩阵。

| 真实本地09场景，每项分别300/650ms墙钟暂停 | 每次L1 blocks／receipts | 最终D／cuts | 结果 |
| --- | --- | --- | --- |
| window | 52／6 | 24／1 | PASS；origin10/W40，frontier49保持D20，50产生21/e10、22/e11 |
| ahead | 47／8 | 14／3 | PASS；`(45,0,1)`冻结prefix10/anchor8，`(45,1,1)`冻结prefix11/anchor11 |
| cap | 5／6 | 30／1 | PASS；L1 block3填至D30，origin上限3，允许规定的空块drift例外 |
| accepted_wait | 42／7 | 0／0 | PASS于阶段A门禁；接纳`(40,0,0)`后frontier39，后续Set不越过它 |

正式Tester共8次新私有Anvil运行，四项Rust边界测试通过。A/B reader采用不同交付和读取顺序，规范输出一致；每场景两种墙钟暂停的虚拟轨迹和D报告逐字节一致。

独立raw checker从292个原始L1 blocks及54份receipts重建事件和D，每reader比较136个派生块的native bytes/hash、实际状态、physical/virtual链接及10条切面。它不调用12实现或Builder checker；改native hash、改cut目标、漏receipt三个负控制均被拒绝。独立API harness成功恢复111个动作位置checkpoint，拒绝21类checkpoint篡改，另覆盖genesis/config/DEP、有限fixture、错误/未来位置和跨reader恢复。

真实block12收据的缺失是在新鲜RPC记录上进行离线注入：D等待于前沿11；恢复同一真实收据、对原Mine command 30重试后才到D20/e10。另一个独立进程从block45两条marker之间的checkpoint恢复，结果与连续执行相同。这些不冒充真实DA恢复、Batcher crash或BFT同cid进程恢复。opaque batch虽有真实EVM接纳及实际签名核对，仍没有11有效frame/DA持久性证据。

## 命令、证据及摘要

正式运行cwd为上述worktree，Rust/Cargo1.95.0、Python3.12.8、coincurve20.0.0、Foundry1.0.0、solc0.8.20。seed=`none (explicit script)`。所有heavy均使用全宿主锁、两个Cargo jobs；没有嵌套锁。

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:/home/jy/miniconda3/bin:$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/derivation"
export CARGO_BUILD_JOBS=2
export L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0
out=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/12/artifacts/derivation-tester-d0afeee-run1
flock /tmp/megaroation-poc-heavy.lock bash poc/crates/derivation/tests/validate.sh "$out"
python3 poc/verification/derivation/candidate_review/check_raw.py "$out" "$out/raw-check-d0afeee.json"
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline \
  --manifest-path poc/verification/derivation/candidate_review/api_harness/Cargo.toml -- \
  "$out" > "$out/api-check-d0afeee.json"
```

这是实际执行的路径；再次运行须选择新输出目录，原证据只读保留。`.build`指向任务12的数据盘目录。Verifier原始证据为：

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/12/artifacts/derivation-verifier-d0afeee-run1`

Tester原始证据为上述`derivation-tester-d0afeee-run1`。工具、二进制、原始文件及精确命令均在[Verifier manifest](../verification/derivation/candidate_review/evidence-manifest-d0afeee.json)和[Tester manifest](../verification/derivation/simulation/evidence-manifest-d0afeee.json)。

| 摘要 | SHA-256 |
| --- | --- |
| Verifier manifest | `359d07d78e2b6efd0adb9df806cda3fb511db910986848550198f2bb9142e4a2` |
| Tester manifest | `ec08d4380ee37a405ea627b65c12599667541ba5ed2be4364e92c4665793d2bf` |
| Tester独立raw结果 | `c0b08d03d9ad0b7cda07923d8f60c69142b6204fad9f53a89dd905de159803d9` |
| Tester独立API结果 | `364ed3fbd62971e76412d9663d295f6309faced70e69364f5d18d999a16c82df` |

Lead再次逐项验证Verifier的124份原始文件、7份验证支持、12份候选源码和3个binary，以及Tester的126份原始文件和17项工具/checker输入，大小和SHA-256全部匹配。没有拿旧smoke替代正式结果。

Tester validation的21.406秒是宿主墙钟耗时（含锁/构建/暂停），不是协议延迟。四场景最终虚拟参考时间分别104/94/10/84秒；L1 genesis timestamp=1000秒、slot=2秒、模拟finality delay=2秒。私有loopback Anvil均正常退出并清理，本阶段不测公网延迟。

## 剩余依赖与续派

11仍是唯一frame/DA编码owner。消费者需要的精确字段和不变量已列在[接口记录](derivation-interface-v1.md#11-接入所需字段与不变量)。收到准确接口SHA及真实样本后再冻结阶段B适配，正常合并依赖并复验实际批次：49/50/51 inclusion、缺失/不符DA后恢复、普通交易实际parent执行、fallback后接纳、同高度替代及第二条record重验、同区块控制之间的真实batch。

当前仅这些依赖11的部分等待。13的GFR/治理恢复终点及受控普通运行接线另列集成范围，不倒置为已完成阶段A的前置；完整01 R1–R18不是所有handoff测试的统一前置。没有实际Batcher或BFT进程crash验收结论。

无frame阶段已全部完成，所有子agent停止；按派发规则释放活跃执行槽，保留本chat/worktree/分支及证据等待11接口续派。完整功能12尚未accepted，main未合入，未push。
