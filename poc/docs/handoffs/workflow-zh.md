# 新会话共用工作约定

给新会话的输入包括任务 Markdown、本文、源 checkout `/mnt/reth/jy/megaroation_poc` 和实现基线 SHA。若前置提交未就绪，先完成 source mapping、合同草案和独立期望。基线齐备后再进入依赖它的实现。分任务交接说明记录范围与期望，合同经主会话冻结后作为 Builder 和 Verifier 的执行依据。01 直接继承已经冻结的恢复合同。

## Worktree 与基线

1. 从任务给出的 ref 创建或领取本会话的 **Codex managed worktree**。新会话若已经以 worktree 模式创建，核对现有 worktree 的基线和状态，直接在该路径工作。需创建时使用 `create_worktree`，填写源 checkout、任务名与集成 SHA。使用工具返回的绝对路径作为每次命令的工作目录。分支使用任务给出的 `codex/` 名称，托管 worktree 起初若为 detached HEAD，按宿主方式保存候选提交并报告其 SHA。
2. 开始时运行 `git rev-parse --show-toplevel`、`git rev-parse HEAD`、`git status --short --branch` 和 `git ls-tree HEAD regional_BFT_sim`，记实际路径、基线 SHA、未提交改动和子模块 pin。01 的起点为 `36eff1c76d5c4df5d0790133c0e14719c0c73ebd`。其余任务使用[任务索引](README.md)列出的前置工作已经集成并验证的 **一个 SHA**；不能用同目录其他分支或历史 smoke 代替前置结果。
3. 保留源 checkout、现有未提交修改及 Regional BFT 子模块。当前子模块 pin 为 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。该提交目前有本地来源，远端初始化可能取不到。需要查阅 BFT 设计时可只读使用源 checkout 上与 gitlink 一致的已存在子模块，先核对 SHA。PoC 本身在 `poc/`，固定版本的 Commonware 代码和补丁也在其中。
4. 当前交接文档若尚未纳入 Git 提交，新 worktree 无这些未提交文件。需在新会话提交时附上任务文档与本文，或从已提交交接文档的适用基线启动。文档链接以本仓库目录为基准。

## Docer → Verifier → Builder → Tester

| 顺序 | 角色与配置 | 交付物 |
| --- | --- | --- |
| 协调 | 主会话 GPT-6 Astra / ultra | 核对实际模型、权限和可用并发；登记路径所有权；决定共用接口、shared types、protobuf、Cargo workspace/lockfile 和集成 |
| 1 | Docer `bft_spec`，GPT-6 Astra / xhigh | 读取 v10、v2、已冻结合同和 pinned source，写触发事件、前置状态、guard、下一状态和输出，并注明规则来源、P0 mock、待定项及验收。主会话核对并冻结版本和路径 |
| 2 | Verifier `bft_verifier`，GPT-6 Astra / xhigh | 在看候选实现或 Builder 说明前，从规格与合同独立给出结果、输入向量、边界和失败预期。将期望和复现命令记录到独立路径 |
| 3 | Builder `bft_builder`，GPT-6.1 Sol / high | 按冻结合同实现。每个路径一个 writer，指定的源文件和 Builder 回归测试由该 writer 负责。返回候选 SHA、依赖及实际检查结果 |
| 4 前的复核 | Verifier `bft_verifier` | 用先前期望独立检查同一候选，复现问题并列出规则、代码位置、影响与最小输入。修复由 Builder 完成，Verifier 对更新后的候选复查 |
| 4 | Tester `bft_simulator`，GPT-6.1 Sol / high | 只在独立复核通过的集成 SHA 上跑约定矩阵，保存原始证据，报告结果、时间、前提、缺项和已知限制 |

角色定义在仓库 `.codex/agents/`。采用实际可用的命名角色和任务并发。配置允许主会话外六个子线程，实际宿主可能更少，按可用槽排队。发现指定模型或 effort 无法启用时，明确记录实际状态并向总协调报告。BFT 相关 Docer 和 Verifier 先读根 `DESIGN.md`、固定的 Regional BFT `DESIGN.md` 与[验收技能](../../../.agents/skills/regional-bft-conformance/SKILL.md)。

先让 Docer 和 Verifier 完成各自交接，独立期望形成后启动一个 Builder。Verifier 可同时补充独立用例。总协调安排跨 worktree 的共享路径与合入顺序。若需更改已冻接口，先由总协调定合同修订并登记受影响的工作，再统一修改。初始一次只运行一个 Rust build 或重型仿真。

## 工具与复现环境

- 先读根 `AGENTS.md`、`DESIGN.md`、v2 [交接](../../../formalization_v10/poc_v2_xuelab_handoff_zh.md)、[搭建](../../../formalization_v10/poc_v2_xuelab_setup_zh.md)和[设计](../../../formalization_v10/poc_design_zh_v2.md)。固定 Rust 1.95.0、Commonware 2026.10.0 和两个 workspace 的 lockfiles。现有 xuelab 工具路径见[根 README](../../../README.md)。
- `./poc/scripts/bootstrap.sh` 使用**当前 worktree** 的 `.build/cargo` 和 `.build/target`。新 worktree 需要准备依赖缓存，再跑 `--locked --offline`。可在授权范围复用源工作区的已验证 Cargo 下载缓存，编译产物和实验输出留在任务 worktree。遇到空缓存或工具不可用时记录缺项和实际命令。Anvil 使用[现有 smoke 的版本与哈希](../setup/foundry-2026-11-15.md)。
- 从 worktree 根运行默认 `./poc/scripts/bootstrap.sh`、原生格式检查 `./poc/scripts/bootstrap.sh fmt --manifest-path prototypes/i1b-runtime/Cargo.toml --all --check` 和原生测试 `./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline`。按任务适用范围选择。真实进程测试需要允许绑定 loopback，使用新的输出目录和本次进程的 PID。
- 保持虚拟参考时间、节点协议时钟、公开日志键、墙钟四种量独立。节点误差界为相对虚拟时间 ±1 ms，RPC 延迟独立建模。控制输出、消息和服务故障都经过受控调度和提交边界。P0 使用三个五成员等权委员会、四个不同 signer 的 quorum、10-height 窗口及错时 Enter。实现范围为 v2 mocks 和三类交接。

## 交付与合入

沿用已冻合同，新增合同按 `poc/docs/contracts/task-contract-<任务>-v1.md` 命名。每份任务指定的 `poc/verification/<任务>/` 保存独立期望、fixture、审查记录与可版本化结果。原始文件保存在 `poc/artifacts/{verification,simulation}/<任务>/<SHA>/` 或任务明确记录的位置。对外报告仅包含公开测试信息，私有 fixture key 和本机凭证保存在本地。

结束时回传基线和候选 SHA、工作路径、合同版本、拥有与变更的文件、已验收接口、独立报告、准确命令、seed/配置、原始轨迹位置和摘要哈希。每个场景注明 PASS、FAIL 或 INCONCLUSIVE、实际观测窗口与时序前提，分别记 regional proof、已验证区域交付、G 接纳、Inbox 接纳与 D 派生。需要真实进程或数据的用例在相同候选上留出闭环证据。总协调复核并集成，下游使用该集成 SHA。
