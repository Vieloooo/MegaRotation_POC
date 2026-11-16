# M1 健康 15 节点 smoke 交接

> **执行结果。** 该 smoke 已在 `9154785732d803c1f2f69aee3b2af67220a12f58` 通过。用户授权以该结果合入本地 `main`。下文保留启动时的交接信息。[验收证据](results/M1-healthy-smoke-result-zh.md)

在 **poc jump**（`xuelab_jump`）打开一个 Codex chat，选择 **GPT-6.1 Sol / xhigh**。把本文件作为启动任务。

## 目标

尽快实现并实际跑通一次健康满窗口 smoke。启动 15 个真实 OS worker，组成三个各 5 人的委员会。用一个固定 seed、错时 Ready/Enter、现有 L1 fixture 和少量会改变执行状态的交易。R0 的 h10 和 R1 的 h20 完整公开并进入 G 后，由 GFR 的真实结果驱动接棒。到 R2 的第一块 h21 经 Publisher 和 SimEspresso 进入 G 时成功。

保留实际四个不同签名者的 Simplex Final 校验、完整正文和 ancestry 校验、真实 anchor/state 准备、固定窗口 `[1,10]` → `[11,20]` → `[21,30]`、受控时钟及完整 grant/Ack 后输出。已关闭 `cid` 停止新签名并保留历史结果。Manager 消费实际公共重放，场景脚本只提供输入与调度。各节点使用自己的签名身份。发生意外进程退出即记录运行失败。

本次先交付这条健康路径。SessionEnd 的 `h9→h11`、同一 session 多候选选回、完整多 seed/墙钟矩阵、官方 Espresso 同字节发布和崩溃恢复留作后续验收。固定配置可限制每个逻辑 session 只保留一个合格候选。后继 `cid` 仍须从真实 descriptor 和 anchor 得出并受控安装，保留已有队列与交付义务。原完整 M1 合同继续单独追踪。

## 直接使用现有工作区

- 附加现有 managed worktree `/home/jy/.codex/worktrees/5d7a/megaroation_poc`，在工具返回的实际路径工作。分支 `codex/poc-m1-regular`，已知起点 `5c6235a6d78d28aef434c3fd353c9fab29ed2502`。若无法直接附加，使用受支持的工作区权限申请。源 checkout `/mnt/reth/jy/megaroation_poc` 供查阅与共享 Git 对象。
- 原 worktree 可能有用户更新留下的 `.codex/config.toml`、`.codex/agents/bft_spec.toml` 和 `.codex/agents/bft_verifier.toml` 三项修改。开工先核对并保留。需要干净候选时，与配置提交 `b5babc233c1e93e4c71bee0039aa02f4a44bd053` 对比后单独保存一致的配置变更。保留所有其他改动与原始证据。
- 复用 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m1-regular` 的 Cargo 缓存和 target。每次运行使用新证据目录。Rust `1.95.0`，Commonware `2026.10.0`，Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。重构建与仿真使用 `/tmp/megaroation-poc-heavy.lock` 及 `CARGO_BUILD_JOBS=2`，已有内部锁的 runner 直接调用。
- 先读源 checkout 的 `AGENTS.md`、`DESIGN.md` 和其中指定的 v2 文档。只在该 M1 worktree 做集成。旧 feature chats 已停，沿用其提交和报告。

## 复用输入

| 组件 | 精确版本与用法 |
| --- | --- |
| M0 | `35d382ac445cc8bc779f0d18fd23cb57e54680da` 是已验证的基础，在 M1 分支祖先中。包含现有受控 runtime、02 transport/Clock、03 registry/PoP/Plan/执行/L1 fixture、06 Publisher/SimEspresso、07 GFR。 |
| H2 | `b639076fb50f5fcbc06e309f67b1fb6125697242` 的纯 Manager 已合入 M1，接现有 reducer 到真实 `Prepare/Enter/Close` 及提交回执。 |
| H1 | `e35b9504e306c26a26b195b3ccce8df919924384` 交付 `healthy-session-api`，实际受测 `ed3915290f0890ffcb878f3f705df3ff790cb008`。用它接真实 worker、FinalVerifier 和受控生命周期。普通 profile 草案可通过 `git show 37d17fd2242a825cf6147e431ed7ae0a6e19c08f:poc/docs/healthy-session-program/native-profile-v1.md` 查阅。 |
| H3 | `983054682b04ae6ec75e7a6f334b2026abcc68e6` 已交付经过验证的 A/B/C。M1 起点已有 A。整合剩余增量，复用 publisher bridge、公共完整性、stop/custody、受控 transport 与独立 reader。官方节点 payload 输入已具备，可留到首轮 smoke 之后。 |
| M1 runner | 复用现有 runner 和证据封装。阶段报告 `poc/docs/milestones/results/M1-integration-stage-20261116.md` 列明已通过的组件检查与还缺真实绑定的 ports。 |

## 执行与交付

先冻结简短的健康 smoke 合同和来源先行的独立预期。沿用项目角色，Docer Astra/high、Verifier Sol/xhigh、Builder 和 Tester Sol/high。你是本次唯一集成 chat，协调一个源码 writer 加独立复核，按精确提交整合 H1/H3，再补真实运行代码。复用已有 runtime、scheme、witness、journal、网络和 codec。Commonware 的本地准入与全局验证谓词仍按已锁定源码区分。只有受控提交成功才释放输出。

从五个真实引擎的第一份 Final 和正文跑起，先在 SimEspresso 收录并由 GFR 接纳，再扩到 h10、h20、h21。先聚焦这一次真实运行。独立检查至少从原始证据核实 15 个真实 PID、四个有效签名、区块身份、h10/h20/h21 的接纳、handoff anchor/window 和完整提交顺序。fixture 证据继续按组件范围报告。

跑通后提交可复现候选。回报准确 SHA、分支、一条运行命令、原始 trace 与证据目录、观察到的 PID 和中间高度，以及剩余验收。结果标记为 **健康满窗口 M1 smoke**。正式 `main` 提升由完整 M1 验收处理。遇到失败，保留原始证据，修复可恢复问题并继续跑到真实结果。
