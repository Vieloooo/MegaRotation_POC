# 16 官方 Espresso 本地兼容交付与验收申请

2026-11-16，Asia/Shanghai。**固定官方节点的可信本地 API profile 完成，G-API / G-REPLAY / G-ENV PASS，提交总协调验收；本 chat 未自行标记整个功能 accepted、增加批次计数、合入 main 或 push。** 实际官方 publication、完整前缀、重复查询、离线回放及独立原始数据检查已完成。完整 PE 时间接口保持明确 Unsupported；这项兼容缺口作为验收结果保留，不伪造来源事实。

## 提交与工作范围

| 项目 | 精确记录 |
| --- | --- |
| Worktree / branch | `/home/jy/.codex/worktrees/d2be/megaroation_poc` / `codex/poc-espresso-compatibility` |
| 干净起点 | `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b` |
| 唯一消费依赖 | 06 交付 `4b034dfbb6c3e1c7143208347b546daab8dfe298`，其已测试组件源码为 `8ba78f1ace3b383caa1c4360401c645d25907326` |
| 正常 no-ff 合入基线 | `bcd30b0879e6c17afc177bfea76f7fee170cc069`；未合入无关 03/09 |
| 来源合同与独立向量 | `89925a6ed47cf8a57d6d052d8db192007019661c` |
| 首个可编译接口 | `9747268e52410c1e7ce6b79d96df2ca858598a78`，source/native 类型及 Unknown/Unsupported 边界 |
| 原完整组件候选 | `8750535d8f02e920353c1ca827c78046e72d578a`，独立 Rust 7/7、离线矩阵 30/30 PASS；其首次官方启动未完成 |
| bootstrap 运行合同澄清 | `2ed1540b507af076b904816ff0cadc47d2e286ef` |
| **最终源码候选 / 定向复核 / 正式测试 HEAD** | **`6248072b3fd0eee1ae468f0961a4d3589291cba6`** |
| Regional BFT / Commonware | gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` / `2026.10.0`，保持不变 |
| 数据盘 | `.build` → `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16` |

本交付的后续报告/验证工具提交不改变上述候选的 crate 或场景源码。所有正式结果保留真实执行 SHA，不将旧测试改标为新 HEAD。相对 `bcd30b0`，新增或修改仅在本功能 crate、scenarios、verification 以及专属文档。06 源码、根 workspace/lockfile/types/core、native 恢复、commander 和子模块未由本任务编辑。

已只读核对源 checkout 的最新 `restart-free-handoff-track-20261116.md` 以及 handoff 提交 `32a3eb6b31ff8c1dfe2d05dbc2ed26c0c031bb77`。16 继续独立于 01/04/三类 handoff；没有为读取通知覆盖 WIP 或另建用户 chat/worktree。

## 角色与流程

实际启动并复用命名角色：Docer `bft_spec` → Verifier `bft_verifier` 先从来源推导预期 → 单 `bft_builder` → 同 Verifier 复核 → `bft_simulator`。每阶段串行移交 tracked writer；源码候选先提交，正式运行使用被复核的同一 SHA。主协调承担接口、依赖、集成和最终证据核对。

| 角色 | 实际可见模型信息 |
| --- | --- |
| 主协调 | 项目 `.codex/config.toml` 为 `gpt-6-astra` / `ultra`；本 chat 未获得独立 runtime attestation，不将配置冒称运行时证明 |
| Docer / Verifier | 实际调用命名 `bft_spec` / `bft_verifier`，角色固定 `gpt-6-astra` / `xhigh`；各报告保留配置与 runtime 可见性的区别 |
| Builder / Tester | 两者运行时报告 `gpt-6.1-sol`，命名角色固定 high；未替换模型 |

全部重构建、测试及官方运行经 `/tmp/megaroation-poc-heavy.lock`，Rust 1.95.0、默认两个 build jobs；runner 内部持锁，不套同锁。未重写历史、修改共享 daemon/SSH/代理或操作未知进程。

## 实现及来源边界

[冻结合同](contracts/task-contract-espresso-compatibility-v1.md)和[crate API](../crates/espresso-compatibility/README.md)定义 `EC16-NATIVE-1` 来源观察及 `EC16-AUDIT-1` 审计格式。复用 06 的 `PublicTimeUs`、`EspressoPosition`、`LogMessage` 和容量规则；不重定义 03 canonical 对象或签名字节。

- 固定官方 [release 20260918](https://github.com/EspressoSystems/espresso-network/tree/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171)，源码 `fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171`，协议 `0.3`、API `/v1`、namespace `160016`。
- 官方镜像 index `sha256:af73f55630690caa35aa0bc015d60dc03333791f966fdcc07b0798e33d4f768e`，amd64 manifest `sha256:cfb845e49cb5369f77cd8bbda0b4fe4aad340793cfd130b7f2bb631e59d2db66`。dev-node 二进制 SHA-256 `62f6aec1a217eeda71f1c284dde8501afadf5a359a40c82cd93c68d033de1e94`。
- 本机 Docker socket EACCES；使用同一固定镜像的未修改 executable、loader、运行库与 SRS。在独立 user/network/PID namespace 内运行，只有 loopback；显式 Anvil L1 为 18775，API/dev/builder/prover/tonic 为 18770–18774。
- 官方源码会先启动 bootstrap Anvil 再设置显式 L1。修复后的私有 PATH wrapper 原样 exec 同一镜像 Anvil，持久记录 argv、PID/PPID、start ticks 和端口参数，随后检查清理。实际 helper 请求 `-p 0`；其分配端口/退出码未观测，不填造。显式 L1 与该辅助进程分开记录。
- 信任任务本地官方 API 的 finality 和列表语义；保留 block、leaf/QC、原 raw payload 与全部 flat 位置响应。q 是绝对高度，θ 为 checked seconds×1e6，重复和顺序保留。筛选目标 namespace 后另生成连续 message index，并保留原生 index。缺项 WAIT、冲突原子拒绝，两个 reader 不共享进度。
- 严格检查录制文件哈希、路径及容量；独立 Python oracle 进一步按固定源规则解码 raw namespace/交易表并核对列表、审计字节。该检查不验证 QC、VID 或 Regional FINAL。

## 已执行结果

| 检查 | 结果与范围 |
| --- | --- |
| 最终候选 crate tests | 8/8 PASS；包括七类上限等号/超 1、重复/乱序/缺口、源及 reader 绑定、错误原子性、Unknown/Unsupported |
| 独立组件核验 | `8750535` 上外部 Rust 7/7 + 最终离线矩阵 30/30 PASS；Rust 和冻结输入与 `6248072` 完全相同，明确作为未变源码证据继承 |
| 修复路径独立检查 | `6248072` 上真实镜像 Anvil 的 wrapper/隔离/RPC/身份/退出检查 PASS；没有以该小样替代 Espresso publication |
| 官方正式 run02 | **PASS**，72 条原始 HTTP 观测；q0–q5 共 6 个完整块，q5 两条真实 publication，位置分别为原生 i0/i1 |
| 重复及缺数据 | exact repeat 的 ACK 与首次相同；在有限前缀内只观察 q5/i0 对应 occurrence；q0/i0 越界请求为实际 HTTP 404 |
| 独立重查/离线回放 | first、first+37ms pause、requery 的 889 字节 audit 和公共 projection 完全一致；独立 oracle 各核对 6 份 raw payload、2 条交易，全部 PASS |
| 独立实际证据复核 | Verifier 只读重核原始 ACK、q5 block/交易、404、source/run/genesis、OCI pin、wrapper及清理，G-API/G-REPLAY 在声明 profile 内 PASS，无新 finding |
| lead 封存核对 | **7 份 SHA256SUMS、690 个文件条目全部匹配**；含首次失败、组件矩阵、wrapper 和正式成功数据 |

实际公共 θ 为 `0,1794793206000000,1794793208000000,1794793209000000,1794793210000000,1794793210000000`µs；q4/q5 同键。source ID 为 `EC16:13c2cc6d232444486b12122cca75024fd4d52d6a1793e95f197ac3d506fe3540`。公共 audit SHA-256：

```text
43f106c905bb0877c2131bd7926eb099701d4e3936c0845e96ec7fb36a4e7cf5
```

Seed=N/A。正式 runner 墙钟 17.46s（含锁、构建及回放）；HTTP 观察跨度 13.187s。它们不是 proposal/finalization virtual tick，也不是节点时钟误差界。初次 `8750535` 官方运行在 genesis 前因 bootstrap PATH 缺项失败；其 21+15 份证据保留。早期 setup 探针的 helper 来源限制已在 [preflight](espresso-compatibility-preflight-v1.md)更正，未作为正式接受证据。

## 证据、复现和剩余接口 gate

- [独立预期/向量](../verification/espresso-compatibility/expectations-v1.md)、[组件复核](../verification/espresso-compatibility/candidate-review.md)、[修复复核](../verification/espresso-compatibility/runner-review-6248072.md)。
- [正式测试报告](../verification/espresso-compatibility/simulation-report.md)、[机器结果](../verification/espresso-compatibility/simulation-results.json)：全部命令、版本、记录编号、hash 和有限观察范围。
- 正式原始数据：`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02/`；116 文件清单 SHA-256 `a983c8a2d8c70de3a0064b9614a08892f382feddca5795ec3062ae210153a88e`。
- Tester 独立结果：同级 `ec16-official-6248072-run02-tester/`；27 文件清单 SHA-256 `d6702d85e80d2bdc159c5f84fbe6a645258801072318436f41384ce4c5b0d599`。
- Lead 校验：`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/lead-seal-6248072/checksums-audit.json`，保存全部七份清单摘要和 690 条核对结果；该 audit JSON 的 SHA-256 为 `2bdc1e1f3bcb96cb17895beb45edece562f85e52bbb6163664ab51b09bda4a01`。
- 公共官方源码 archive：`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/provenance/official-source-fd221e8e4b.tar.gz`，SHA-256 `a973cc8954eb6b7252c60a2f8810f2a0434ffdf8b025aa35f7940b7cd2dced3f`。

实际正式命令在测试报告中固定为 `run-local.sh ec16-official-6248072-run02 6248072b3fd0eee1ae468f0961a4d3589291cba6`。从报告交付 HEAD 复跑时，先核对其 crate/scenarios 与 `6248072` 无差异，再传当前准确 HEAD 和全新 run ID；脚本记录真实 HEAD，不能把新运行冒称旧 SHA：

```bash
cd /home/jy/.codex/worktrees/d2be/megaroation_poc
git diff --exit-code 6248072b3fd0eee1ae468f0961a4d3589291cba6 HEAD -- poc/crates/espresso-compatibility poc/scenarios/espresso-compatibility
poc/scenarios/espresso-compatibility/run-local.sh ec16-reproduction-new "$(git rev-parse HEAD)"
```

runner 自带同一个重任务锁，不另包 flock。选新的名称，已有证据目录不覆盖。停机后的原始包可直接用 CLI replay，再用版本化 `check_capture.py` 独立核对；准确 argv 见 Tester 的 `commands.txt` 和原始 `offline-commands.json`。

| Gate / 能力 | 当前结论 |
| --- | --- |
| G-API、G-REPLAY、G-ENV | **PASS**，限固定 release/protocol、本机隔离环境及该有限记录 |
| G-PE-TIMING / 完整 PE-LOG-WIRE-1、PE-READER-1 | **Unsupported / OPEN**：官方数据没有可核验 proposed_at、finalized_at virtual tick 或 live_at_proposal；显式 Unknown，不输出伪造 PELG/VerifiedPrefix |
| E-TS、T_E、PUB 时限、外部虚拟时钟/1ms保证 | 未验证，不由墙钟观测或离线可重复推出 |
| QC/VID/Regional FINAL 密码学验证 | 不在本可信本地 API profile 内，fixture 和哈希一致性不计为密码学 PASS |
| Stream / reconnect | 固定源码支持已核对，未做现场 stream/reconnect 验收 |
| 完整 06 接线、15-worker及三类handoff | 保持其独立任务 gate，不因16结果关闭 |
| 整个16 accepted / main合入 | 等待总协调对本验收申请及上述兼容缺口作决定；本 chat 不计数、不合并 |

若后续需要让完整 PE reader 消费官方来源，应由 06 owner 冻结能把公共 q/θ/消息列表与可选时间证据分开的接口修订；当前 16 不修改 06 或用默认值绕过该缺口。
