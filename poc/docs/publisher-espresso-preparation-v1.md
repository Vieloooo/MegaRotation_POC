# 06 Publisher / SimEspresso 准备记录 v1

日期：2026-11-15。本记录是接口与实施范围交接，不是组件PASS或完整06 accepted。合同为[task-contract-publisher-espresso-v1](contracts/task-contract-publisher-espresso-v1.md)。

后续进展：2026-11-16已完成本次授权的组件实现、独立复核和同SHA矩阵，见[组件交付](publisher-espresso-component-handoff-v1.md)。本文保留冻结前准备记录；完整06仍待接线验收。

- 实际worktree：`/home/jy/.codex/worktrees/1fe1/megaroation_poc`；分支`codex/poc-publisher-espresso`。初始clean detached `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，正常fast-forward到政策提交`0a951c482a9712678ab9de34765b781b9c3c9f42`；没有reset源checkout。
- `.build`最初不存在，lead建立指向`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/06`的symlink，无旧数据迁移。cargo/target/artifacts均本任务独占。lead实际preflight记录root剩13G、data约1.5T。
- Rust `1.95.0 (59807616e)`、cargo `1.95.0 (f2d3ce0bd)`。全部Rust build/重型test/simulation使用`flock /tmp/megaroation-poc-heavy.lock`、`CARGO_BUILD_JOBS=2`，不嵌套锁。
- v2设计SHA-256：`95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`。本checkout gitlink与只读源submodule HEAD均为`691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`；Commonware固定2026.10.0。未初始化/修改本worktree子模块。
- 主chat实际模型由总协调runtime turn_context核验为`gpt-6-astra/ultra`，对应turn_id `01a1dfd1-e7a6-7191-aafb-31527a5cce90`；这是lead收到的运行时证据，不仅是请求配置。命名Docer `bft_spec`和Verifier `bft_verifier`已实际启动（Astra/xhigh）。后续Builder/Tester使用实际命名`bft_builder`/`bft_simulator`（Sol/high），由lead记录spawn及候选/测试SHA。

已只读查阅03准备提交`183430e71e5d61533f54a76b2d7d22c5021f1704`、02准备提交`125d8cf4e30b5affe3cd731fbb5d4a73d0e40a10`的合同。两者都是草案；其codec/wire/state不存在已验收实现承诺。分层依赖v2允许本次纯组件先行，真实grant/Ack、公共类型、历史验证和原生证书仍等待准确依赖SHA。

本次接口发布供07/16使用：PE-LOG-1日志记录、PE-LOG-WIRE-1 opaque payload envelope、PE-READER-1独立完整前缀/水位查询和PE-PUB-1持久任务。proof/body可以分布于不同公共消息，06完整日志与语义材料完整分开；q_avail和G接纳归07。TimestampPolicy的E-TS相对proposal-time且只在proposal时live，等θ/冻结不破坏E-S，倒退拒绝；本地wall不推进协议。Publisher支持结构化验证trait、file-backed耐久snapshot、stable attempt、显式dispatch确认、公共stop和历史recorded确认。

所有权：单Builder仅`poc/crates/sim-espresso/`和`poc/crates/publisher/`及各自manifest/lockfile，Verifier仅`poc/verification/publisher-espresso/`；本合同和准备说明由Docer写，冻结/提交由lead执行。根workspace/types、其他任务源码、commander、source checkout和Regional BFT只读。

冻结顺序：lead核对合同→Verifier保存独立source/expected向量→提交接口基线→单Builder实现→准确源码SHA→Verifier复核→同SHA Tester矩阵。组件结果与真实02/03/04/07/16接线gate分别报告；未启动的真实联调不得写PASS。
