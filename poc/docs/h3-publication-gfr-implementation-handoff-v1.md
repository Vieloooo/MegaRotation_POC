# H3 implementation handoff — A/B/C

2026-11-16，Asia/Shanghai。**本次授权的 H3 独立组件 A/B/C 均已实现并通过独立复核、同 SHA 测试。健康 15-worker M1 仍 OPEN / NOT_RUN。** 收尾只保存已完成代码、测试和证据，不扩大工作范围。

## 基线与准确提交

Worktree：`/home/jy/.codex/worktrees/1e2f/megaroation_poc`；实施分支：`codex/h3-publication-gfr`。旧 `codex/poc-viewer-gfr` 和准备提交 `c4a944a754e6f9cfa50f57a2399b707a2555ce94` 保留。由已验收 M0 `35d382ac445cc8bc779f0d18fd23cb57e54680da` 创建实施分支，只重放自身准备文件，得到 `a1bee707b92b6b550a1b53c914e667994db15c84`。授权为 `d55d39f73593676325bdc0fbb7ba3845ca93025f`。

| 切片 | 可消费源码 / Verifier / Tester 的共同 SHA | 结果 |
| --- | --- | --- |
| A：完整性与 Publisher adapter | `e59d026e8c08b64ff744b57c63cc7756f0b34019` | 独立 14/14；旧 codec 167/167、规则 117/117；Tester 14/14 和 G1/G2 seed42、pause0/2ms 四重放，两对及原 07 完整 trace 逐字节相同 |
| C：官方 runner artifact 输入 | `bca02b095bac1e5701eb78e352d605c0ed887e10` | 独立 52/52；Tester 52/52、8 次 Rust 离线 replay、5 组旧向量；旧官方 first/requery audit/projection 保持一致 |
| B：record、stop、custody、transport、credit | `d289066082e0b89964e16653360a5c72356142f8` | 独立 19/19；Tester B19/19、A14/14 和独立字面向量检查通过，无超时或未完成用例 |

A 证据交付 `f0ad79e33933f23eb527af62c6bc5096a77844fb`；C 证据交付 `21358cfb63fb3a7547fda662672efdeb87afef56`。最终收尾提交只补报告、独立 harness/证据与测试 lockfile，生产代码保持 B 源码候选不变。未合入 main、未 push、未自行合并其他 feature 实现。

## 已实现

- 独立 workspace `poc/crates/publication-bridge/`：Native-only 验证注入、受限 `BridgeResult`/私有存储 codec、已有 06 Publisher 的 Offer/Recorded adapter。生产路径无 fixture fallback；缺真实能力保持 WAIT。
- 从已有 viewer-gfr 抽取一处公共完整性计算，供 GFR 与 Publisher 共用。新查询只接 sealed `VerifiedPrefix` 和显式 cut，不信任调用方可改 index；内部 GFR 仅复用同绑定的已验证历史。未选中候选可先 RECORDED，不能因此进入 G。
- 完整 `RecordDelivery` 保留来源、全部 input/effects/lifecycle、POS/LIFE 身份及独立 GFR 参数。算法完成与实际 root commit、H1 LocalId、ReadyToken 分开。
- `StopLedger` 的计划/确认/恢复依赖匹配 owner 回执，保存闭合范围以处理晚到 Offer；scope 查询重算既有 03 CID/config，watermark 使用真实 Native Publisher task 的 verified deadline 并核对本 ledger 参数。已安装停止事实不因后续失败回滚。
- `AttemptCustody` 保留单个不可变 attempt 及消息 ordinal，支持 proof/body 在不同公共 q 发布；custody、proposal/finalization、reader 完整性与 RECORDED 分开。
- 83-byte H3TF 运输封装、有界重组与 credit plans，复用实际 02 类型/算法，输出 `ServiceEffect::Send` 意图而非直接网络操作。实际 owner/delivery/submission/credit 验证由注入端口承担。
- 已移交的四个 16 文件增加可选 `--payload-manifest`，严格验证并保存输入副本，保持原字节、重复 ordinal、实际 POST/namespace/q/index 映射；旧 marker 模式和原 launcher/隔离保留。官方 Unknown/Unsupported 与 QC/VID 信任边界未变。

具体接口、例子和限制见 [A handoff](h3-publication-gfr-slice-a-handoff-v1.md)、[C handoff](h3-publication-gfr-slice-c-handoff-v1.md)、[B contract](contracts/h3-publication-gfr-slice-b-v1.md)、[crate README](../crates/publication-bridge/README.md)及[H1 接口消费映射](h3-healthy-api-consumption-map-v1.md)。H1 候选 `f9364a2a8e59913f7f9373fab2774be4ed528e57` 仅只读检查，未合并/复制/编译。实际获准的集成基线与薄适配由 M1 merge owner 接续。

## 测试与证据

命名 Docer/Verifier、单 Builder、Tester 按冻结→实现→独立复核→同 SHA 测试完成；角色/模型执行记录在各切片报告。Rust/Cargo 1.95.0，重任务使用同宿主 flock、2 jobs 和 task07 独立 cargo/target。轻量 Python 检查无协议模拟副作用。

| 数据盘下的目录（前缀为 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/`） | 内容 |
| --- | --- |
| `verification/h3-a-e59d026-independent-v1` | A 原始复核日志和来源摘要 |
| `simulation/h3-a-e59d026-acceptance-v1` | A 同 SHA 测试、G1/G2、原 trace 比较 |
| `verification/h3-c-bca02b-independent-v1` | C 52 用例与保存的官方录制兼容性；679 个封存条目 |
| `simulation/h3-c-bca02b-acceptance-v1` | C 52 用例、Rust 离线重放及独立 raw oracle |
| `verification/h3-b-d289066-independent-v1` | B 19 用例、命令、固定 fixture、审阅说明 |
| `simulation/h3-b-d289066-acceptance-v1` | B19、A14、独立向量、锁文件更新前后及精确命令 |

[B 独立审阅](../verification/publication-gfr-healthy/slice-b/d289066082e0b89964e16653360a5c72356142f8/review.md)与[Tester 报告/manifest](../verification/publication-gfr-healthy/slice-b/tester/report.md)记录测试边界。其实际 02 route fixture 使用 seed7、20µs 模型延迟；其他固定向量 seed 不适用。B 包含 q0–q2 的 split publication、原 proof position 与 first-complete q 分离，及合成 owner 端口确认前后状态。没有随机全网络或真实 15-worker 运行。

B Tester 首次 A 回归因旧测试 lockfile 缺新增 `sim-transport` 依赖而在测试前 exit101；授权离线更新仅增加该本地包和依赖边，原 hash/差异保留，之后 `--locked` A14 通过。生产锁、源码和测试代码未改。A 原始验收证据未重写。

Lead 已核对 A 的 22 个来源文件、13 个原始复核文件、5 个 Tester 命令产物及两对 trace；C 的 679 个封存条目、4 个来源文件、22 个 Tester 命令流及 manifests。所有最终测试的真实 SHA 和有限范围保持原记录，不把报告提交改标为新一次运行。

## 仍需集成的真实 gate

1. H1 的实际 Native FinalVerifier/历史 context、真实证书与完整结果；真实 root/aux、grant/Ack、submission/delivery/shared-credit 端口。合成回执和 exact-subject 测试 callback 不能关闭这些 gate。
2. H1/H2 的正式 API/receipt/ID 薄适配与实际 Manager action 执行。保留 `Committed(actions_pending)`/`ActionsApplied` 和不可回滚 terminal；H3 不宣布 Ready。完整 record 上限取 H3/H1/H2 的适用交集，不能截断原子批次。
3. 独立 M1 merge 的 15 真实健康 worker、R0→R1→R2/R2 首结果进入 G、固定窗口 SessionEnd 两类真实证据。h9→h11 必须提前生产缓存，晚合法 slot＋padding 不冒称该例。
4. C07：将上述真正 M1 输出的同一 payload 提交官方本地节点并读回，另用真实 H1 checker 复核区域证明。此次没有新官方节点或 genuine H1 publication 运行；官方时序不进入确定性 GFR 证明。

01 恢复、Force、D/RESUME/DA/Web 没有扩展。收尾时出现的 `.codex` 用户配置改动保持原样，未纳入 H3 提交。旧工作树/历史/失败和通过证据全部保留。
