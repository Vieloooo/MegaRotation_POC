# H2 pure Manager reducer：组件交付 v1

2026-11-16（Asia/Shanghai）。本次交付可消费的纯 Manager reducer，独立复核及同 SHA Tester 均为 **PASS（有限可信输入纯组件范围）**，属于 M1 的 H2 组件切片。真实 H1/H3 接线和 15-worker M1 终点仍为 OPEN；本报告不将可信输入 fixture 结果记为 native 验证。

## 来源、提交与范围

- 真实工作目录：`/home/jy/.codex/worktrees/a6cf/megaroation_poc`；实施分支 `codex/h2-session-manager-implementation`。
- 准确 M0 基线：`35d382ac445cc8bc779f0d18fd23cb57e54680da`。实施授权只读来源：`d55d39f73593676325bdc0fbb7ba3845ca93025f` 的 `poc/docs/milestones/M1-implementation-dispatch-20261116.md`，SHA-256 `4a211cd38a5db108dc3b8548a87c19e599cb3870e534e513b41e487b4c5cc8b6`。
- 原准备提交 `027ba8be9f8f64e35313f990f4f03c5d3c238a0c` 和分支保留；仅自身准备内容重放为 `dd3110f77724a273a4e7982c05d6c4b36dea4b9d`。冻结合同提交 `f2a63c412de6ad244d93c71fb684a244c2f0097b`。
- 初版代码与独立 harness 候选：`129c6674356a98a908289a94df07802808603ff1`。后续候选/实际受测 SHA 见下表；文档收口提交不替代真实受测身份。
- 新生产代码只在 `poc/crates/session-manager/`；独立验收位于 `poc/verification/session-manager-healthy/reducer-v1/`。未修改 H1/H3 源码、根 types/manifest、commander、Regional BFT submodule 或其 Git 历史；没有合入其他 feature 实现、更新 main 或 push。

使用 [regional-bft-conformance](../../.agents/skills/regional-bft-conformance/SKILL.md) 先封存独立预期，再冻结、单 Builder、独立验证及同 SHA Tester。Docer/Verifier 使用命名 Astra/xhigh 角色，Builder/Tester 使用命名 Sol/high 角色；主 chat 请求与配置为 Astra/ultra，运行模型元数据未由工具独立暴露。

## 可消费接口

crate 为独立 workspace：`poc-session-manager` / `poc_session_manager`，直接引用已有 03 类型和 07 ReplayRecord/codec，没有第二套公共编码。入口及可执行消费示例见 [crate README](../crates/session-manager/README.md) 和 [组件测试](../crates/session-manager/tests/reducer.rs)。

| 边界 | 行为 |
| --- | --- |
| `Manager::new` | 建立一个 worker 的逻辑状态；本地区 genesis 有唯一 revision-zero Create。运行时必须先完成 bootstrap root 安装，才能释放该意图。 |
| `plan(input, local_time_us)` | 返回不可变 delta、WAIT 或未变化状态；不修改已接受状态，不做 IO。绑定完整 `(input,local_time_us)` 与原 Manager 状态快照。 |
| `accept_commit(delta, CommitRef)` | 仅在可信 H1 已实际提交 root/aux/guards 后安装内存状态。构造 CommitRef 本身不证明 fsync/验证；重复原 attestation 不重复生成义务。 |
| `latch_commit_failure` | 保存已知单调 terminal/scope facts，保留原未成功的 cursor/assignment，进入不可恢复 Faulted。H1 已安装 guard 仍须独立 sticky。 |
| `view` / `delta.view` | 只读候选、Selected/phase、cursor、完整 records、action ledger 和 guards。Entered 与 Entering、Committed 与 ActionsApplied 分开。 |
| `dispatchable(now)` | 返回当前可释放的意图；依赖原动作 receipt、最新 schedule、clock/stop/terminal 和 Pause/Close barrier。必须由 H1 在受控 grant/Ack 路径执行。 |

支持完整 source record/seq/位置/事件投影核对、重复与冲突、同 pc 多轮 regular、scope tombstone、已接纳 physical tip 排除、唯一 descriptor、准确准备完成引用、最高 Ready 调度、未确认 Enter 停车、原实例 SelectBack、私有 ClockUnsafe 和旧结果 Publish 义务保留。ActionId 是私有 typed `(manager_id,worker,origin_revision,ordinal)`，不改变公共 cid。

回执在已 Applied/Cancelled 后收到旧 Retryable 时保持不变；矛盾成功回执拒绝。初始 Create 的 bootstrap 安装、复用 Enter 意图的动态 parking barrier、`accept_commit` 的单决策进度与 `view().status()` 全待办进度，均由 README 明确。

## 独立预期与证据

[冻结合同](contracts/h2-session-manager-reducer-v1.md) SHA-256 `fe2f962d8480a0cb6b5d0bbb26ff6748d3e7149dfa6e6b05a9732c3c91c70a96`；[freeze review](../verification/session-manager-healthy/reducer-v1/contract-review-v1.md) 接受该纯组件边界。

来源预期在实现前封存：增量 `9ad71186b41e85a5e30100a7fd63d9a009435e12e4459046e3056ad8cd7965e5`，矩阵 `4d78b0c12f65f7be5f0c5a2da0048b7815f3ef9e027290e942960ac39cd87395`。旧 30 项 oracle、11 条向量与旧来源清单保持原 hash。独立 harness 的初始22 tests 在读取 reducer 算法前依据这些来源写成，仅按公开 API/getter 对齐调用。

构建统一使用 Rust/Cargo 1.95.0，H2 数据根 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager`，私有 cargo/target/artifacts，复用 M0 的公共 registry cache。每次重任务单层获取 `/tmp/megaroation-poc-heavy.lock`，`CARGO_BUILD_JOBS=2`，使用 `--locked --offline`。固定输入无需随机 seed；seed 记为 N/A。纯测试输出不是 native 协议 trace，wall 耗时仅为执行诊断。

Builder 原始包 `artifacts/builder.Xpp6vO/` 保存初次编译可见性错误、修正后检查与8项组件测试、命令及候选补丁。初版独立包 `artifacts/independent-129c667-v1/` 保存 clean 候选上22tests、精确 argv、工具/input hashes、stdout/stderr、exit0。后续增补只针对冻结合同 R17 已明确的容量覆盖缺口，未迁就实现改变既有 oracle。

最终受测候选为 **`2340f9c1c8cbc3bc4be57a5c18f884cc74390563`**，生产 tree 为 `78a9250a93da0c5db78578f19951bad5c04f6b80`，与 `129c667…` 完全相同。C2 只增加五项独立容量/溢出测试与 coverage 说明。文档收口提交不改变该受测生产或 harness 输入。

| 检查 | 实际候选与结果 | 可审查证据 |
| --- | --- | --- |
| 独立 source review + 全 harness | C2，27/27、exit0，前后 clean；无生产 conformance finding | [审查报告](../verification/session-manager-healthy/reducer-v1/candidate-review-2340f9c-v1.md)、[运行 manifest](../verification/session-manager-healthy/reducer-v1/independent-result-2340f9c-v1.json) |
| Tester 组件 suite | 同 C2，8/8 integration tests，通过；适用 unit/doc 阶段完成 | [Tester 报告](../verification/session-manager-healthy/reducer-v1/tester-report-2340f9c-v1.md) |
| Tester 独立 suite 与精确复跑 | 同 C2，两次各27/27，exit0；22项输入 hash 前后相同，工作区 clean | [机器摘要](../verification/session-manager-healthy/reducer-v1/tester-summary-2340f9c-v1.json) |

独立复核包为 `artifacts/independent-2340f9c-v1/`，manifest SHA-256 `c351fdfbbea1f318d99fbe113e3e0bf49fe7bead8e7a3904c33398aeaaf2b566`。Tester 完整包为 `artifacts/tester-2340f9c-v1-3/`，summary SHA-256 `a67ffdedb930c07dba84bb68037662fc72bc3f2da5c029f21ebaef82e291735a`。

两次独立 stdout 仅剔除 `test result` 行末的墙钟耗时后逐字节相同，canonical SHA-256 为 `44086f0bf26c72a9d9e052cc14603a5b0aa216c8e75aba914694a2963e0355a0`；保留每次28条手写 PASS 行。该比较是固定测试输出重放，不是 native 虚拟事件轨迹。8+27是35个不同 Rust 测试，重复27项不另算独立覆盖。

Tester 的先前环境 attempts 原样保留：`tester-2340f9c-v1` 在8项组件测试后，doc-test 误经默认 rustup shim 尝试下载而中断；`tester-2340f9c-v1-2` 在共享锁队列中取消，未运行 Cargo。完整成功矩阵显式设 `RUSTDOC=<Rust1.95.0 bin>/rustdoc` 与 `PATH=<Rust1.95.0 bin>:/usr/bin:/bin` 后在新目录重新执行；没有安装工具链或改源码/预期。

实际独立触达 Programs64、session128、branch16、records2048、results256、events64、effects128 的界内/界外及 timestamp/height 溢出。branch/effects 用例是结构容量 fixture，不声称实际 M1 激活16个 branch 或 GFR 产生冗余 Duty。较大 descriptor/lifecycle byte、dependencies64、actions4096 防御上限有源码 guard 审查及当前固定 Plan/profile 下的条件可达性说明，没有任意私有状态注入或全局穷举声明。`Manager.commits` 保留精确历史，合同没有给它单独的长期上界；本次结论限于有限 M1 输入轨迹，不是永久服务内存界。


## 具体剩余接线

H1 依赖盘点（2026-11-16 11:48:23，Asia/Shanghai）时的准确 HEAD 为 `4741489ba7a07d2a3f3f194dbd59ae3fd8d17bb9`：`poc/crates/healthy-session-api/` 尚不存在，`api-slice-v1.md` 仍是未跟踪的 FREEZE CANDIDATE；尚无可消费的命令/Ready/commit/receipt crate 发布 SHA。这是本轮具体依赖观察，不是“等待整个 H1/01”。获得独立 M1 merge chat 提供的准确 API 集成基线后，再按该接口做薄映射，不自行合入别的 feature 源码。

后续需真实 H1 ReadyToken/精确执行依据、同实例 Pause/SelectBack 与 callback/deadline、实际 root/aux/terminal guards/完整 Ack，以及 H3 已提交 record 的完整来源/连续 source seq 和发布任务接收。H2 不持有第二数据库，不替 H1 实现最终证明或本地准入/certify 映射。可信输入构造器、hash 和 CommitRef 不能防可信调用方伪造前置事实。

早 h11 与晚合法 h14 两条组件输入例均保留 fixed window11–20；它们未生成真实 block/proof，也未执行 GFR 接纳，不能关闭 M1 SessionEnd 验收。M1 仍要求15真实健康 worker、R0→R1→R2、R2首结果进入G、原窗口 SessionEnd和同一实际结果的官方本地 Espresso读回。Force、OS重启、D/RESUME、批次/DA、Web没有在本切片扩展。
