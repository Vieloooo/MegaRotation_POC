# 01 本地进程恢复：新会话接手任务

日期：2026-11-16。用户明确要求新建 session 完成现有 recovery job，并明确全部工作属于本地模拟。主 chat 使用 **GPT-6 Astra / ultra**，内部使用既定 Docer → Verifier → Builder → Verifier复核 → Tester 多 agent 流程。

## 可复制的启动任务

继续完成 Sequencer Rotation PoC 的功能01及 I1 恢复验收。读取本文件及源 checkout `/mnt/reth/jy/megaroation_poc` 的 `AGENTS.md`、`DESIGN.md`、`formalization_v10` 中指定的交接/搭建/v2设计，以及 `poc/docs/commander/dependency-plan-v2.md`。

这是用户自己的 Rust/Commonware 确定性模拟工程：只运行本机 worker、loopback RPC、临时 journal 和固定测试密钥；通过调度本次启动的 child 进程 crash/restart、延迟/丢弃模拟消息及本地 fixture 输入，核对状态持久化、原 deadline、签名缓存、恢复屏障和 Ack 对账。损坏快照、错误 cid、过期 lease 等负例只作用于本次测试的私有文件副本。没有外部目标、漏洞利用、扫描或未授权系统访问。保留原合同的全部正常/边界验收，不因任务说明澄清而删减测试。

源仓库为 `/mnt/reth/jy/megaroation_poc`，已连接宿主为 `xuelab_jump`（项目 `ccb826e2-d47a-4623-9b20-2632ac96d8e3`）。使用 `create_worktree` 从以下准确快照创建自己的 managed worktree；如果已经获得适用 managed worktree，先核对基线和状态，复用它。

```text
ref: 8a81391c7a0b08be3ce1d2adf3b44e8bf4bbc509
existing branch: codex/poc-native-recovery-resume
suggested managed worktree name: poc-native-recovery-resume
```

该分支尚无 worktree 检出，快照为 **WIP，不是已通过候选**。核对工具返回路径及实际 HEAD，明确记录所有命令的 workdir。旧 worktree `/home/jy/.codex/worktrees/5efa/megaroation_poc` 及其未提交内容保持只读，不在两处同时实施。Regional BFT 子模块 pin 保持 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，可以只读使用源 checkout 已初始化的副本，不修改子模块历史或工作树。

实际主模型/effort若无法核实则如实报告。子角色定义可只读使用源 checkout `.codex/agents/`；Docer/Verifier 为 Astra/xhigh，Builder/Tester 为 GPT-6.1 Sol/high。遵守实际并发槽位和全局最多五个活跃 job；不另建用户 chat。采用源 checkout 的 `regional-bft-conformance` skill。单个 Builder 独占获分配实现文件，Verifier 先从来源/合同给出期望，再读候选。

接收后持续完成以下工作，不以只做环境核对或重复 Builder smoke 结束：

1. Docer 核对已冻结的 `poc/docs/contracts/task-contract-native-recovery-v1.md` 及批准修订、`native-recovery-01-execution.md`、`native-recovery-01-contract-review.md`。整理 R1–R18 的逐项覆盖及缺口，明确哪些记录是源码审查、synthetic oracle、Builder 过程运行或独立验收。
2. 将当前快照作为明确、可复现的待审候选。Verifier 先核对原有 e668450 两项实证问题的修复，再检查 root事务、RecoveredAck、Archive/body durability、签名缓存、actor/helper/clock/timer恢复和 resolver路径。恢复前后的证据、证明观察和最终交付分别检查。
3. 单 Builder 补剩余实现/场景；沿原合同允许路径修改原型、native proto/wire及必要 opt-in vendored hook/patch。当前这些 native 恢复路径移交给本新chat；02仍只拥有独立transport组件。根公共 types/根workspace由03负责，不能顺带重定义。任何协议或跨任务接口变化先记录明确决议。
4. 独立复核通过后，Tester 在**同一准确 SHA**运行完整约定矩阵。涉及顺序的用例使用 seeds7/42/99、wall pause0/2ms、启动顺序对照，真实恢复用新 PID/incarnation。保留失败轨迹和范围限制；不得把“进程退出0”或实现产生的valid标记当独立证明。
5. 完整通过后提交代码/合同/独立报告与复现说明，给总协调回传候选与测试SHA、明确文件移交、命令/seeds、证据绝对路径及摘要、限制。未覆盖条目保持OPEN；不自行合入main或push。

## 快照与已有证据

旧主chat `01a1df9d-73d2-7a12-a9b8-e03a0db6d9dd` 被平台检查中断。总协调核对其 Docer、Builder、两名 Verifier 均已停止运行；Builder在主chat中断后留下进一步开发交付。新快照保存旧 HEAD `b9fce86f8c286fb110bafbae5d902581ac0c19f3` 上的 **33 个变更/新增文件**；使用独立 Git index，旧 worktree、旧 index 和原 branch tip 原样保留。`poc/artifacts` 的本地绝对符号链接未加入快照，原始文件继续保存在数据盘。

快照清单及每文件 SHA-256：

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/01-resume-handoff/manifest.json`

现有原始证据（只读复用，新的运行使用新目录）：

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/01/artifacts/`

- `poc/docs/native-recovery-01-builder.md`：四轮实现和已知缺口；Builder最终报告26个真实进程用例，14个命名崩溃切点至少观察一次。
- `feature01-increment3-matrix/`：固定 base+source.patch+二进制的24次R16运行，总协调已重算同seed的frames/virtual-trace哈希，均一致。该快照早于后续修改，不能代替新候选验收。
- `feature01-final-native-tests.log`：132 passed/0 failed，总协调读取核对；`feature01-final-i1a-tests.log` 的20项结果由Builder报告。两者仍须核对快照与最终候选关系。
- `poc/verification/native-recovery/feature01/`：第一轮有界R1/Fresh独立检查、旧问题复现和后续检查脚本。
- `poc/verification/native-recovery/feature01-transactions/phase-v2/`：45项synthetic oracle期望，零真实进程/BLS验收，不计实现通过。

当前已开发到真实 resolver 请求/丢响应/新PID重启/新generation请求/证书持久化、三处 root/Ack恢复切点、helper原due/ready/durable/offered、Scheduled Waiting/Closed、prepared/barrier Nullify、两次启动中再崩溃及快照拒绝等路径。R16保留缺body，交付高度为零；证书修复不能表述为最终区块交付成功。

仍开放的主要验收：coalesced/suppressed ingress来源、没有权威own-vote Artifact的已消费结果、view退出后的取消/迟到回调、完整deadline/latch和逐参与者readiness阻塞矩阵、terminal后的archive、R17非匹配parent、跨多代重复RecoveredAck，以及全部R1–R18同候选独立复核。以合同逐条核对，不能将此摘要当作缩减后的完成标准。

## 构建资源与持续交付

新任务 `.build` 使用专属数据盘 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/01-resume`，旧01缓存/证据只读。不得覆盖已有结果或共享正在使用的target；公开Cargo下载缓存可以复制/复用，凭据不复制。Rust工具链可显式选择：

```text
REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin
```

所有Rust build/重型测试/仿真使用 `flock /tmp/megaroation-poc-heavy.lock <command...>`，`CARGO_BUILD_JOBS=2`，不嵌套同一锁。只管理本次run的PID。虚拟参考时间、节点协议时钟、公共日志时间键和墙钟分开；输出不能绕过受控grant/Ack。先发布经过独立审查的精确接口切片及未覆盖范围，再继续完整I1，避免02无必要等待。

## 会话创建状态

本文件与快照已由总协调准备。**当前工具列表缺少 create_thread 和 send_message_to_thread，因此截至本次交付尚未创建新chat，也未启动新的功能01子agent。** 在应用中新建 Astra/ultra chat后，将本文件作为启动任务即可接续。此前五chat规则继续有效。
