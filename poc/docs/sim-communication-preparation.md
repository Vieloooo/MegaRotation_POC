# 功能02：时钟与通信准备记录

日期：2026-11-15。当前状态：**KERNEL CONTRACT REVIEW；native-integration-waiting-dependency=01**。原准备材料保存于 `125d8cf4e30b5affe3cd731fbb5d4a73d0e40a10`；下方“环境、基线与角色”至旧准备结论记录首轮事实，不冒称当前仍受whole-feature门禁。[dependency-plan-v2](commander/dependency-plan-v2.md) 已于 `0a951c4` 冻结并正常合并为本worktree `c647098`，授权独立生产kernel先实现，完整02/I1验收范围不变。

当前新增交付为 [kernel合同](contracts/task-contract-sim-transport-kernel-v1.md) 与 [API v1](sim-transport-api-v1.md)。Docer本轮只修四份文档；Verifier同时只读独立推导；lead冻结并提交接口版本后，交棒Verifier保存预期，再由一个Builder实现新 `poc/crates/sim-transport/` 独立workspace/lock。native/I1a/根manifest/types/proto仍禁改。组件通过不计完整功能accepted，不启动额外用户chat。

资源状态更新（lead交接）：`.build` 已安全链接至02专属数据盘目录，776MB公开Cargo registry已复制，Rust1.95版本已核对；当前root/home约13GB、数据盘约1.4TB。`.build` 是生成资源链接，由现有common `.git/info/exclude` 忽略，不纳入Git（该exclude非本chat修改）。所有重型命令仍全宿主flock串行，CARGO_BUILD_JOBS=2。Docer未构建、未执行源码或提交。

当前决议：Q3/Q4纯运输算法与组件持久确认API由本kernel合同选定，等待lead冻结；真实fsync/root floors/恢复签名gate仍属01，公共registry/cid/ReplayPosition属03，reader prefix/日志顺序属06，L1业务属09。上述边界不阻塞独立kernel。当前无需要总协调先行裁决才能写kernel的新增实质阻塞项。

## 环境、基线与角色

| 项目 | 核对结果 |
| --- | --- |
| 实际 worktree | `/home/jy/.codex/worktrees/0b8e/megaroation_poc`；所有命令显式使用该路径。 |
| 宿主 | 主 chat 通过 app `read_thread` 核对 `remote-ssh-discovered:xuelab_jump`；未使用 direct 别名。 |
| 分支 | `codex/poc-sim-communication`。原 worktree 为 detached HEAD，主 chat 正常新建请求的分支。 |
| 起始 HEAD | `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`；与交接准确一致，不需要合并。 |
| 代码基线 | `36eff1c76d5c4df5d0790133c0e14719c0c73ebd`；上层提交为交接文档。 |
| 起始状态 | `git status --short` 无输出；源 checkout 文件未改动。创建分支首尝试被 sandbox 的共享 `.git` 只读限制挡住，自动审批 escalation 后成功。 |
| BFT gitlink | `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`；本 worktree 未初始化子模块。 |
| BFT 只读来源 | `/mnt/reth/jy/megaroation_poc/regional_BFT_sim` HEAD 与 gitlink 一致；未改其 worktree/history。 |
| v2 SHA-256 | `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`。 |
| 固定版本 | Commonware `2026.10.0` / upstream `92a2f68a0e131c8b69de41e84aec37da6280f212`；Rust要求 `1.95.0`。本阶段未编译，不以缓存存在推断可构建。 |
| 构建资源安排 | 总协调报告根分区/home/tmp约剩11GB，数据盘/mnt/reth约1.5TB，已建立02专属 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02`。这是上游资源交接，非本阶段重新测量。 |

主 chat 已只读核对本会话日志 `turn_context` 的实际模型字段，未复制完整 session 日志或凭据：

| 角色 | 实际命名角色/配置 | 运行时核对 turn |
| --- | --- | --- |
| Lead | 主 chat `gpt-6-astra` / `ultra` | `01a1df9d-eef9-7652-be99-3b80befc9cbb` |
| Docer | `bft_spec`，`gpt-6-astra` / `xhigh` | `01a1df9f-01a8-7593-b69a-f4513c8afdf2` |
| Verifier | `bft_verifier`，`gpt-6-astra` / `xhigh` | `01a1df9f-387c-7ab3-b76b-40a5dc9e23a7` |
| Builder / Tester | 未启动 | 不适用 |

使用 [regional-bft-conformance](../../.agents/skills/regional-bft-conformance/SKILL.md)；`protocol-spec-writer` 未在可用技能目录出现，不冒称已使用。Docer 先读权威来源再只读定位实现；Verifier 同时只读独立推导，尚不从 Docer/实现计算期望。Docer 完成后释放写入权，由 Verifier 保存自己的文件。后续审查/修订保持一个 writer。

## 路径登记

| 路径 | 所有者与本阶段状态 |
| --- | --- |
| `poc/docs/contracts/task-contract-sim-communication-v1.md` | Docer；新文档，来源硬规则 + 待决适配提案。 |
| `poc/docs/sim-communication-preparation.md` | Docer；本记录。 |
| `poc/verification/sim-communication/` | Verifier；独立来源推导、预期与静态输入。完成后的具体清单由 lead 交付记录列明。 |
| runtime/proto/types/manifests/locks | 本阶段无写入授权。 |
| `poc/docs/commander/`、源checkout、Regional BFT | 不修改。 |

未来路径提案和移交边界见合同 §2；它们不是当前所有权。特别是 native `driver.rs` 同时含01恢复/I/O与02 transport责任，只有01接受并交出精确基线后才能串行接手。

## 来源读取与规则推导

已按顺序读取根 `AGENTS.md`/`DESIGN.md`、v10指定交接/搭建/v2设计、`poc/docs/handoffs/workflow-zh.md`、`poc/docs/commander/README.md`、02/03交接与 conformance 技能，再读取 I1/I1 annex/native recovery 合同、v10 Session Program 的生命周期/first_received/历史证明规则、固定子模块 DESIGN/README 和 Commonware release 源码。

主协议来源使用仓库文件；Commonware trait/语义通过源 checkout 中只读缓存检查：

```text
/mnt/reth/jy/megaroation_poc/.build/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/commonware-runtime-2026.10.0/src/lib.rs
/mnt/reth/jy/megaroation_poc/.build/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/commonware-p2p-2026.10.0/src/lib.rs
/mnt/reth/jy/megaroation_poc/.build/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/commonware-p2p-2026.10.0/src/simulated/mod.rs
```

可直接从来源冻结的预期：唯一虚拟时钟、grant内ClockView固定；样本投影与半开lease；有效回复唯一续租；原timer/first_received/terminal跨恢复保留；全部输出完整Ack后释放；默认FIFO与显式跨link交错；可靠结果durable enqueue/ACK/retry；历史结果保留和live-cid隔离；每个reader只消费自己的实际前缀。新ID编码、channel enum、FIFO重试算法、drain计费与root新增记录布局仍为提案。

本次来源工作使用本地固定源码，未查询浮动 `main/latest`，未修改或更新依赖。原始阅读是只读命令，不作为运行行为PASS证据。

## 现有 native 边界（只读定位）

以下行号对应准备基线 `ada48a5`，不是01最终实现；01交出后必须重新核对。

| 文件/锚点 | 观察与接口影响 |
| --- | --- |
| `poc/prototypes/i1b-runtime/src/native_hub.rs:227` `Hub` | `workers: [Worker;2]`；routes/attempts/timers/accepted_acks在hub内；新endpoint表需去除双worker寻址假设。 |
| 同文件 `:666` `commit_ack`、`:765` `AckPlan` | 先检查binding/hash/receipt，再在副本计划effects后整体安装；必须保留并扩展到outbox/inbox和新计数，不能逐effect直接提交。 |
| 同文件 `:795` `stage_packet` | 现有route是局部 `(src_member,dst_member,channel)`；容量/attempt顺序显式，固定link delay，body hold为fixture。多个region的同member必须由外层endpoint隔离。 |
| 同文件 `:1447` `event_key` | tick、causal wave、seeded tie和event_id；Fresh重放不应因新profile改变。 |
| `native_config.rs:24–92` 常量/`BaseProfile` | 单cid、五成员registry、两个offset、固定20ms link/250us Clock transit/100ms lease；既有Fresh不能被新配置静默重解释。 |
| `native_network.rs:82` `PacketFactory::packet`、`:118` `validate_envelope` | `member<5`、单cid与epoch、payload/hash/attempt、local/scripted provenance；外层public pid映射与原生证书成员映射须分开。 |
| `driver.rs:243` `Authority::stage_route`、`:636` `stage_witnessed_send` | 输出受controlled authority和witness约束；历史转发须专门权限/入口，不能绕过旧cid终止gate。 |
| `driver.rs:1972` `arm`、`:2059` `clock_snapshot` | runtime timer与clock audit已存在；01将交出恢复checkpoint语义，02保留原绝对期限。 |
| `poc/proto/native.proto` | `Binding`/`Envelope`/四原生channel/`RecoveredAck`已有；新服务外层transport采用版本化提案，不直接把worker_id等同public pid。 |
| `poc/crates/sim-hub/src/scenario.rs` | I1a已有clock request/response overrides、FIFO/queue/offline/duplicate/retry配置；可复用语义，但不是native已接入证据。 |

这些观察只解释适配位置，不决定独立预期。准备阶段不评判01未完成实现为FAIL，也不把历史Fresh/有限native结果当作02通过。

## 跨任务接口提案

1. **01 / root与恢复。** 需要接受SHA、路径移交、root/aux/witness receipt与外部floor、RecoveredAck授权、原timer/work/RNG/quota字段、terminal历史发送入口和R1–R18命令。建议durable outbox/inbox由端点root custody承载，hub只保管调度；Hub crash恢复是否纳入由总协调明确。
2. **03 / 公共对象。** public pid、region/member、排序signer、registry/config_ref、Descriptor/cid/context、ReplayPosition均使用03唯一canonical编码。02只定义transport外层；public config与transport config分开命名并避免循环hash。服务/reader ID命名空间与运行身份需统一。
3. **02 lead / 确定性配置。** 需冻结endpoint匹配优先级、jitter与tie算法、同link显式重排、online区间端点、drain字节/消息计费、outbox/inbox容量/GC、retry、ACK receipt认证以及新wire字段号/hash向量。默认FIFO只针对已入队transport attempts；可靠层at-least-once + durable dedup不自动提供跨逻辑消息FIFO，reader连续前缀另设gate。
4. **04 / 原生接入。** 提供15endpoint+多cid路由、Clock API、真实vote/certificate/resolver/body闭环；列PID和active engine，不替04宣称15个slot-aware引擎通过。
5. **06/09及服务消费方。** 统一受控service/reader接口，保留source/finalized/delivered/consumed位置；transport receive ACK只确认可恢复收件，不代表Espresso发布、L1收录/终结、Inbox接纳或G/D进展。

准确决议项、推荐及阻塞关系见合同 Q1–Q6。没有尚需用户确认的协议自由选择；本阶段由总协调接收提案，等待上游接受SHA后冻结。

## 准备检查与后续命令

已执行的只读基线检查（工作目录均为上述实际 worktree）：

```sh
git status --short
git rev-parse HEAD
git branch --show-current
git submodule status
git -C /mnt/reth/jy/megaroation_poc/regional_BFT_sim rev-parse HEAD
sha256sum formalization_v10/poc_design_zh_v2.md
```

本阶段文档检查为准备完整性检查，不是协议验收：lead完成所有文档/独立输入后执行 `git diff --check`，核对仅授权路径变化及链接，保存准备提交与静态向量摘要。具体执行结果和准确准备SHA由lead最终交付列明，本文不预填PASS。

以下全部 **PLANNED / NOT RUN**，且须在01接受、实现附件冻结并获授权后使用：

```sh
# PLANNED: 在实际 worktree 根执行；Rust 1.95.0 工具路径先核对。
REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin CARGO_BUILD_JOBS=2 flock /tmp/megaroation-poc-heavy.lock ./poc/scripts/bootstrap.sh fmt --manifest-path prototypes/i1b-runtime/Cargo.toml --all --check
REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin CARGO_BUILD_JOBS=2 flock /tmp/megaroation-poc-heavy.lock ./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline
# PLANNED: 02新矩阵/离线checker入口尚未实现，不能当作当前可执行命令。
# 未来命令须以候选SHA、独立fixture、seed 7/42/99、start-order正/逆、wall-pause 0/2ms和新output目录为显式输入。
# PLANNED: 01接受时交付的精确R1–R18/受影响回归命令，在02同一候选SHA重跑。
```

实现阶段构建前先安全迁移worktree现有 `.build` 并链接到 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02`，保持内容与所有权，target和大型原始artifacts放此02独占数据盘目录；Cargo只复用/复制公开下载cache，不复用别的任务可变target。大型证据实际根目录为 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/`，manifest同时记录仓库逻辑路径和实际绝对路径。全宿主heavy锁仍用 `/tmp/megaroation-poc-heavy.lock`。当前准备不构建，因此无须现在迁移或创建链接。

不在当前阶段下载缓存、初始化子模块、启动进程或试跑缺失runner。将来独立checker与矩阵的CLI由实现附件冻结并记录原始 `commands.txt`；未实现命令不报告成功。

准备交付的结论只能是“来源/合同/独立输入准备已完成”或具体准备缺项。运行验收、Verifier候选审查、Tester矩阵全部 `INCONCLUSIVE / NOT RUN`；`waiting-dependency=01` 保持，不能计入 accepted 功能数。
