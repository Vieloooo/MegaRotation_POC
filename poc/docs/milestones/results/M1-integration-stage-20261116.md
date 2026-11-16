# M1 增量集成阶段记录 — 2026-11-16

**本阶段：机械 harness、H2 纯 reducer、H3 Slice A 的集成检查通过。M1：NOT_RUN / OPEN。** 未启动真实 worker 或官方节点，尚无 R0→R1→R2 / R2 首结果进入 G 的系统证据；24 个正式系统用例均未运行。`main` 保持 M0，未尝试移动 main、push 或部署。

工作目录 `/home/jy/.codex/worktrees/5d7a/megaroation_poc`；分支 `codex/poc-m1-regular`。复用本 chat 原 managed worktree，未创建第二个。初始 checkout 干净、detached 于准确 M0；既有源 checkout、各 feature worktree 和历史均保留。最后实际受测集成提交为 **`012bb0b7d9efe5bd22ec544321eed8c12219b9d8`**；本记录的后续提交仅保存报告，不冒称另一次运行。

## 精确提交与消费边界

| 提交 | 本阶段用途 |
| --- | --- |
| `35d382ac445cc8bc779f0d18fd23cb57e54680da` | 已验收 M0、原 main 值；本阶段结束仍不移动。 |
| `d55d39f73593676325bdc0fbb7ba3845ca93025f` → `daebb514d4bf1ea596b828e320257bf54b68ceaa` | 仅 cherry-pick 实施决议文档；预检核对原始文档字节，不错误要求原 dispatch commit 是祖先。 |
| `3f4b671224c0b68bbd2ede40934822c33eafcbe8` | 冻结 M1-HARNESS-1 和来源先行的独立预期。 |
| `5c978db105d52e8df3b7f5104175e0552a37263c` | 首个机械 runner/checker 候选；独立审查发现两项问题，未放行。 |
| `1ade576f2312a75c3b9652eb8147d47d0ad6cb09` | 修复源码 Git 对象绑定和 H09 嵌套报告校验；机械范围独立验收与 Tester 通过。 |
| `b639076fb50f5fcbc06e309f67b1fb6125697242` | 明确发布的 H2 纯组件交付；实际受测 `2340f9c1c8cbc3bc4be57a5c18f884cc74390563`。代码/测试树一致，交付提交仅增加报告。 |
| `f9220e7811b42fe137a96f93143b38246d201565` | 保留 H2 历史的正常 merge；无冲突、无公共 workspace 改动；35 个新增路径均属 H2。 |
| `f0ad79e33933f23eb527af62c6bc5096a77844fb` | 明确发布的 H3 Slice A 及报告；可消费源码/受测 SHA 为 `e59d026e8c08b64ff744b57c63cc7756f0b34019`。B/C 只带准备合同与向量，不代表实现。 |
| `012bb0b7d9efe5bd22ec544321eed8c12219b9d8` | 保留 H3 历史的正常 merge；无冲突。66 项变化限 H3 授权路径，H2 与 M1 harness 未变。 |

未导入 01 恢复 WIP、批次或 D 分支。H2 生产树 `78a9250a93da0c5db78578f19951bad5c04f6b80`；H3 bridge 树 `ed28e3518d238dbb9f6193a4eb0a7324fa4a64d0`、GFR 树 `a75b17bbc41c675550389a71d5e0a7768694c333`，均与已交付、已审源码一致。H3 独立 harness 的 Cargo.lock 在原源码候选中未 tracked，但实际执行 hash 与交付/集成的文件一致，已独立核对，没有重新生成 lock。

## 新增 harness 与首次问题

[runner 用法](../../../scenarios/m1-healthy-regular/README.md)和[checker](../../../verification/m1-healthy-regular/check.py)实现自包含输入、精确候选/版本/来源守卫、raw 索引与封存、离线机械检查和缺能力时的 `NOT_RUN`。未知 binding 只校验文件完整性，不执行其 argv。有效准备态退出 3；坏输入/坏证据/已有输出退出 2。普通文件、路径、符号链接、重复 JSON key、类型和 hash 均受检查。

首轮独立检查 78 个变体，75 符合、3 个反例归为两项：重封源码和自报 Git tree 后仍获完整性 PASS；H09 嵌套报告接受非法类型/路径。原失败包保留。修复后从封存 commit/root/subtree/blob 对象重算身份，并复用嵌套结构校验；101 个独立命令全部符合预期，包括新格式包的源码、全目录树、commit 重封攻击。没有用 manifest 自报 PASS 或两个相同摘要代替验证。

`BundleIntegrity=PASS` 只代表机械范围。真实进程/签名/custody、独立执行/公共重放及官方同字节检查的 oracle 尚未接入；总结果必须保持 NOT_RUN。准备输入可以缺失 fixture/profile，不能用伪造 worker、proof 或 state 填空。

## 同版本检查与证据

数据根：`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m1-regular/artifacts/`。下表均为该根内目录；每包保存准确 argv/cwd/env、source/lock/tool/binary hash、exit、stdout/stderr、时间与限制。轻量机械检查不构建 Rust；所有 Cargo 重任务用单层 `/tmp/megaroation-poc-heavy.lock`、jobs=2、`--locked --offline`，显式固定 cargo/rustc/rustdoc/PATH，使用 M1 自有 cargo/target。

| 实际受测 SHA / 目录 | 观察结果 | 清单 SHA-256 |
| --- | --- | --- |
| `5c978db…` / `verification-5c978db-first` | 首轮 78 命令，两个阻塞问题；失败证据完整保留。 | `cef347b1bea3e9de356e0a8b6e04467199e0e61ac3e79a8403cf68abca0c3476`（artifact-seal.json） |
| `1ade576…` / `verification-1ade576-final` | 独立 101/101；两项问题闭合。 | `6b2e7002377f81d7b29cc4e64c13a9a86d0dda6efcb32308cc7becfd4dbe4033`（artifact-seal.json） |
| `1ade576…` / `mechanical-1ade576-run01` | Tester 12/12；preflight/run/check/compare 均按预期 exit3；重用输出 exit2，81 个原文件未变。 | `3ac3a3b2f21f9636a018f601763b3c374ecff582fc6a47460edfb9c50a877870`（artifact-manifest.json） |
| `f9220e…` / `integration-f9220e-h2-run01` | H2 8/8 + 27/27，含 doc 阶段；新候选/真实 H2 release 祖先检查通过；91 源文件未变。 | `cea4fd7fae02c1d94491d372d724219cd05bda61ae23c0fd8f888ba7d0f32a1b`（artifact-manifest.json） |
| `012bb0…` / `integration-012bb0-h2-h3a-run01` | H3 14/14；旧 07 codec/capacity 167/167、规则 117/117；H2 8/8 + 27/27；G1/G2 四重放；154 源/锁文件未变。 | `14361527ec6913f10a157bf31d5acbfee3610488e5211756576e021013b301ad`（artifact-manifest.json） |

`012bb0…` 的 9 条 Cargo 命令全部 exit0，无 timeout/incomplete。G1/G2 使用 seed42、wall pause0/2ms，通过原 07 `semantic_trace`（分别 7/10 个共同 replay positions）；原始 source row 与完整 stdout 与原 07/H3 已验包逐字节相同，两对 pause 输出相同。G1 SHA-256 `53eabf5b9d4b98ab59cefd3dac8be07ad62ede15d96b04a0d175af2fe979bc5f`，G2 `6c4ba42753752599226376eb956ccaebf8e6dbaacbd464d09909ef55f5afd371`。这些是既有组件 fixture 回归，旧 07 的 M1/M2 用例名称不是本里程碑完成证据。H2/H3 纯测试为固定输入，seed=N/A。

完整集成命令和复现脚本见最后一包 `summary.json` / `replay.py`；summary SHA-256 `ddedd940c6e094eb40836113937a2a0139a86dfd543e877bba900af20db16c7e`。新包实际绑定 H2/H3 release、合同和独立报告，祖先探针均 exit0；preflight/run/check 仍 exit3、协议 gate NOT_RUN。H3 Native 标签测试使用明确的 synthetic exact-tuple proof/context，不是 H1 真证书。

源码组合审查另存 `integration-review-f9220e/` 与 `integration-review-012bb0/`；报告及源码身份摘要同时归档在本仓库专属 verification 路径。本 chat 的命名候选测试前后 HEAD 固定且工作区干净；记录文件提交发生在测试之后。旧失败记录未覆盖，其他 owner 的 target、服务与 PID 未动。

## 尚未解除的系统 gate

1. H1：准确已审 shared API/runtime 发布 SHA；15 真实 worker/五引擎、真实 FinalVerifier/namespace/PoP、slot/Accept 映射、精确 ReadyToken、原实例 Pause/SelectBack、terminal/signed/root/aux/grant-Ack 原始证据。H2 已有纯 reducer 不替代这些事实。
2. H2 薄接线：消费 H1 接口与完整 H3 ReplayRecord，真实 durable commit/receipt/action 相关性；`h2.manager` 仍 NotReady。无第二数据库或仅靠 Hash32/CommitRef 自报成功。
3. H3 后续：受控 transport/custody、公开 Stop、完整本地 reader/GFR→Manager 通路；真实 H1 验签绑定；16 的已审 artifact 输入接口。Slice A 本身不启动这些服务。
4. 运行输入：完整真实 fixtures/workload、精确微秒 schedule、处理/队列/时钟预算和有限 L1 覆盖；独立 BLS/执行/公共 replay oracle。h9→h11 要真实提前生产缓存；晚 slot/padding 不能替代该例。
5. 正式 H01/H02：seeds7/42/99 × forward/reverse × wall0/2ms 共24 runs、12 pause pairs，全部 NOT_RUN。H03 为可选补充，H09 官方同字节发布也 NOT_RUN。官方 wallclock/Unknown 时间、QC/VID 未验证边界继续单列。

只有真实 R2 首结果进入 G、所需系统矩阵/独立 checker 与官方同字节门禁全部通过，才可按原 main 值 CAS 提升本地 main。本阶段未执行该动作。Force、OS BFT 重启、DA/D/RESUME、Web、原完整 I1/恢复验收均未被本阶段关闭。

Rust/Cargo 保持1.95.0，Commonware2026.10.0，BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。本 checkout 的 submodule 未初始化，始终只读固定副本，未改其 worktree/history。角色实际使用命名 bft_spec/bft_verifier（Astra/xhigh）与 bft_builder/bft_simulator（Sol/high）；主配置为 Astra/ultra，工具未提供独立运行模型证明。墙钟只作本地诊断，没有把它当虚拟时间、公共时间或真实网络测量。
