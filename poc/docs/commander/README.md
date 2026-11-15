# PoC v2 总协调与批次集成

2026-11-15。用户授权本会话管理功能 chat：各 chat 使用 Astra / ultra 和独立 managed worktree，最多并发三个 job；每完成三个新功能，创建一个新的 merge-to-main chat。功能范围沿用[16 项交接索引](../handoffs/README.md)。本文是调度合同，不改变协议或功能完成标准。

## 执行约定

- 总协调在 `/mnt/reth/jy/megaroation_poc` 管理调度和交付登记；功能 chat 在自己的 managed worktree 工作。新工作在 `poc/`，Regional BFT 子模块及其 pin 保持不变。
- 任意时刻最多三个由总协调派发的活跃 job。为避免第四个活跃 job，批次合并也占一个槽位。主协调本身不占功能槽。功能 chat 内的角色使用实际可用的并发槽，不另建用户 chat 扩大工作数量。
- 各功能主 chat 显式请求 `gpt-6-astra` / `ultra`。Docer、Verifier、Builder、Tester 使用[共用约定](../handoffs/workflow-zh.md)和源 checkout `.codex/agents/` 的角色配置。不能启用指定模型或 effort 时报告实际状态，不静默替换。
- 顺序为 Docer 合同 → Verifier 独立期望 → Builder → Verifier 审查/复查 → Tester。每个源码路径只有一个 writer；独立期望先于候选实现审查。功能主 chat 负责自己任务的接口冻结和专属集成，跨任务接口变更交总协调决定。
- 依赖未验收时只能做获分配的合同和独立期望。准备完成不计为功能完成。总协调核验上游 SHA 和证据后，向下游发出实现基线；不要求上游提前单独合入 main，因此三功能一批的规则不会造成依赖死锁。
- 所有 Rust build、重型测试和仿真在同一宿主通过 `flock /tmp/megaroation-poc-heavy.lock <command...>` 串行。默认 `CARGO_BUILD_JOBS=2`。若需多条命令，锁覆盖整个脚本；不嵌套加同一把锁。各 worktree 使用自己的 target、run 目录和实际 PID，不清理其他任务的进程。
- 首次派发的实际宿主为已连接的 `xuelab_jump`，项目 `ccb826e2-d47a-4623-9b20-2632ac96d8e3`；同路径的 `xuelab_direct` 当时不可连接。三个功能 chat 的实际 turn_context 已确认 `gpt-6-astra` / `ultra`。
- 磁盘核对时 `/home` 与 `/tmp` 所在根分区剩余 11 GB，源 checkout 的 target 已有 13 GB，数据盘 `/mnt/reth` 剩余约 1.5 TB。任务将 worktree 的 `.build` 指向 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/<编号>`，保留并安全迁移已有内容。每项的 cargo、target 和大型原始 artifacts 各自独立，避免在根分区编译。只复用公开依赖缓存，不复制凭证。

## 状态与完成定义

总协调独占本目录的登记文件。功能 chat 在自己的文档/verification 路径记录合同、审查、命令和结果，在最终回复交付 SHA 与绝对路径；总协调使用 chat 状态工具读取，不依赖子 chat 向其他 chat 发消息。

功能状态依次为 queued、preparing、implementing、verifying、testing、accepted、merged。waiting-dependency 和 needs-attention 单列。只有完整交接范围的独立审查与约定矩阵通过、候选提交可复现、限制已记录且经总协调接受，才计为一个 accepted 功能。仅通过子项时保持任务开放；不得为凑批次缩小 R1–R18 或其他既有 gate。

每份交付必须列出：实际模型/effort 可见情况，worktree 和基线 SHA，合同/独立期望提交，候选 SHA，变更路径，验证与测试 SHA，命令/seeds，原始 artifacts 和摘要，PASS/FAIL/INCONCLUSIVE、未覆盖范围及下游接口。发生新的接口/实现修改后，重新验证受影响行为。

## main 与合并 chat

启动时仓库没有 main 或远端；总协调将现有文档保存为提交并从该基线建立本地 main。源工作分支保留。main 只接收通过复核的本地批次结果；本授权不涉及对外推送、发布或部署。

每累计三个 accepted 功能，创建一个新的 Astra / ultra 合并 chat 和 managed worktree，给定旧 main SHA、三个候选 SHA、依赖顺序和验收材料。合并 chat 独占集成提交，保存各功能历史，处理冲突并在最终集成 SHA 上复跑受影响验证。成功后以比较旧值的方式更新本地 main，报告合并提交和验证结果。失败时 main 保持原值，返回具体问题。总协调复核实际 main 后才登记 merged。

同批存在依赖时，总协调可以将已接受的候选或包含其依赖的临时集成 SHA 交给后继；候选基线必须明确包含所需接口。共享 manifest、lockfile、types 或 proto 由总协调指定单一修改者，并由批次合并 chat 串行整合。全部功能完成后若余数不足三个，再建立最终收尾合并 chat。

## 初始排程

首批为 01 原生恢复/I1、02 时钟通信、03 Mock Plan/轻量执行。01 可以实现；02/03 先准备合同与独立向量，待 01 被接受后转入实现。

后续按实际依赖动态派发。可行的批次顺序是 `01/02/03` → `04/06/09` → `05/07/16` → `08/10/11` → `12/13/14` → `15`。同批内部仍遵守实现基线依赖，14 可在 08 之后提前准备。合同预备、组件子项或 failed/inconclusive 场景不占用“已完成三个功能”的计数。

当前派发、候选与批次状态见 [state.json](state.json)。

合并时使用[批次合并启动模板](merge-job-template-zh.md)，填入准确旧 main、候选提交和检查范围。
