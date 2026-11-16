# M2 Healthy Force 合同 v1

合同标识为 `M2-HEALTHY-FORCE-1`。基线为 `82028b212eb942e222f6f74a88ab2881322212ab`。指定 checkout 为 `/home/jy/.codex/worktrees/poc-m2-healthy-force/megaroation_poc`，工作分支为 `codex/poc-m2-healthy-force`。本版由 Docer 从规则编写，Lead 已指定运行 profile 和路径所有权，以本文件 hash 确认冻结后 Builder 开始源码修改。合同提交、独立预期提交、受测候选提交分别记录。

## 1. 来源与固定边界

| 来源 | 本合同采用的规则 |
| --- | --- |
| `formalization_v10/01_l1_authority_zh_v0_1.md` §§3.1、3.9 | 单签治理授权、控制序号、policy、L1 位置以及 `act=ts_L1(inclusion)+M_a` |
| `formalization_v10/09_finality_rules_zh_v0_1.md` §§3.2–3.5、4.2–4.5、4.7–4.8 | 公开完整位置、全序、公共覆盖、原位 WAIT、Force anchor/序号/窗口、Expire 与公共记录 |
| `formalization_v10/03_session_manager_zh_v0_1.md` §§3.7–3.10、4.1–4.3 | 状态准备、错时 Enter、暂停、关闭全部相关 cid、terminal 先持久化、幂等 |
| `formalization_v10/02_session_program_zh_v0_1.md` §§3.4–3.12、4 | 原生 Final、签名出口守卫、NotBefore、本地 stop、活性共同参与区间 |
| `formalization_v10/poc_design_zh_v2.md` §§3–7、8.4、9.2、12 | 三个五人等权委员会、静态 schedule、真实 Anvil、受控 grant/Ack、四类时间、准备和成功时限 |
| `poc/docs/milestones/H4-healthy-force-zh.md`、`M1-M5-implementation-plan-zh.md` §3、`M2-and-regular-gfr-replay-handoff-zh.md` A 节 | 健康主动 Force、15 个存活 worker、真实 L1 到新结果进入 G、全部指定边界 |
| `poc/docs/contracts/m1-healthy-smoke-v1.md` | 已验收健康满窗起点及其回归 |

PoC 继续采用 `TB=10`、三区各五个等权身份、每个 Final 至少四个不同合法 BLS signer。Commonware 固定 `2026.10.0`，Regional BFT gitlink 固定 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。本任务使用固定授权 M 的主动测试控制，控制原因写为 `scheduled_healthy_force`。新 session 的实际地区由 `(beta+r) mod 3` 决定，leader 按既有静态规则决定。

本阶段运行持续存活 worker。意外退出或需重启才能完成的主运行判 FAIL，并保存原始诊断。I1 完整恢复仍按原合同追踪，既有 terminal、签名历史和持久提交边界继续生效。M2 的交付终点为独立复核后的候选，由主 chat 集成。

## 2. 写入所有权与接口

Lead 独占合同冻结、依赖、公共类型和最终集成。一个 Builder 独占以下实现路径，按实际需要修改并在候选中列出精确文件清单。

- `poc/prototypes/i1b-runtime/src/m1_smoke/**`，以及有需要时的 `poc/prototypes/i1b-runtime/src/bin/m1-smoke.rs`。
- `poc/crates/viewer-gfr/src/{api,frontier,replay,state}.rs`。
- `poc/crates/session-manager/src/{lib,replay}.rs`。
- `poc/crates/publication-bridge/src/{record,result,stop}.rs`。
- 有 padding 执行前状态修复需要时的 `poc/crates/core/src/merge/mod.rs`。
- `poc/scenarios/m2-healthy-force/**`。
- `poc/crates/viewer-gfr/tests/`、`poc/crates/session-manager/tests/`、`poc/crates/publication-bridge/tests/` 中针对本次修改的测试。

若复用 runtime 需要上述范围以外的文件，由 Lead 在修改前追加明确路径与接口决议。Cargo manifests、lockfile、`poc/crates/types/`、协议消息布局和 schema 迁移的变化先与 Lead 协商并冻结。沿用现有外部 L1 CLI 接口，保持 M1 的 CLI/ABI 默认行为。Verifier 独占 `poc/verification/m2-healthy-force/` 的独立预期和检查器。Tester 独占其 `tester/` 子目录及指定运行证据目录。离线回放 B 在另一个 worktree 工作。

复用 M1 的真实五人 Simplex、Publisher、SimEspresso、GFR 和 Manager。Force 接线输入是已验证 finalized L1 ControlEntry 与公开 reader 前缀，输出为同一次 GFR 转移的 `BranchClosed(FORCE)`、`SessionAssigned` 及后续原生结果。状态准备根据实际 Force anchor 的完整公共前缀执行到 anchor，核对 native hash、执行状态和来源。生命周期由 GFR 产生，由 Manager 在受控提交边界消费。

## 3. 控制、覆盖与原子激活

M2 沿用已锁定 ControlAuthority ABI、EIP-712 domain 和规范编码。签名绑定 chain id、合约地址、配置、caller、request id、Force 类型、控制序号、目标 branch 和 policy。policy 使用 `static-schedule-v1` 对应的三区启用与固定顺序。序号严格递增。相同 caller/request id 与相同规范内容重试返回原接纳结果，控制日志只产生一次。相同 request id 携带不同内容必须拒绝。

Driver 部署真实 ControlAuthority，按其构造依赖部署 BatchInbox，发送真实 transaction，由唯一 L1 driver 手动出块。签名 salt 绑定预先冻结的实验 schedule/config 标识。实际 Anvil header hash 在挖出后核验并封存。若 metadata 使用相同 height/timestamp 的执行 fixture 投影，必须命名该投影、逐 header 核对对应关系，并由真实 finalized driver 前缀控制可读范围。封存 signed calldata、tx hash、receipt、收录 block/header、控制 log 和 decoded ControlEntry。EVM 收录、模拟 finalized、各 reader 交付、GFR 激活分别有事件及虚拟时点。

事件键 `x=act` 可完成的条件为已验证 Espresso finalized 前缀覆盖 `x`，完整 finalized L1 控制前缀覆盖 `x-M_a`，且该转移所需 Plan/anchor 等材料齐备。缺任一输入时保留原 `pc`、G、branch、Tail 和 lifecycle 输出，返回带缺失原因的 WAIT。补齐后在同一位置原子处理。Anvil `latest` 和本地 NodeClock 到期不构成公共覆盖。

同键全序为 L1 控制按 `(block,tx,log)` 排序，随后 Expire、SessionEnd、Espresso 按各类 index 排序。Force 处理前的 G 只含排在该 Force 之前的已接纳结果。旧分支同键 publication 在 Force 之后处理。新 session 的结果须在 Force 的公开位置之后，通过 Espresso 事件接纳。

## 4. Force 状态转移

设事件前状态为 `(G,beta,Tail,policy)`。

1. `target_beta` 已关闭、不同于当前 beta，或 Tail 为 HALT 时，只写控制无效记录。G、branch、policy、Tail 和 cid 生命周期均保持该事件前的值。
2. 有效时输出 `BranchClosed{old_beta,FORCE}`。旧分支全部 cid，包括 Assigned、Preparing、Ready、Running、Paused 和预备常规后继，均进入永久关闭流程。G 的全部已接纳内容保留。
3. `A=last(G)` 取事件前实际 G head 及其执行后状态，`beta_new=beta+1`，`Ref=BlockRef(A)`，新 policy 的 L1 floor 为旧 floor 与实际收录 height 的最大值。
4. 旧 Tail 为 NEXT 时，以当前负责 session 已贡献给 G 的 physical block 判定 `r_new`。无贡献时沿用 r，有贡献时用 r+1。source region 为旧负责区域。旧 Tail 为 PENDING 时，沿用其 Boundary 已保存的 r 与 source region。
5. `h_min=max(A.h+1,slot(act+T_init)+1)`，`h_max=h_min+9`。窗口由实际收录记录、公共参数和实际 anchor 算出。
6. 使用实际 `(beta_new,r_new)`、Ref、source region、window 和新 policy 调用 Plan，绑定唯一 descriptor/cid。Plan 成功时输出 SessionAssigned，Tail 为 NEXT。无可选 Plan 时 Tail 为 PENDING。缺材料时整个转移原位 WAIT。
7. Manager 先持久化旧 cid 的 terminal，再调用 Close。迟到状态准备、Enter、工作完成、timer、proposal/vote/sign 回调逐次检查 terminal。重复 lifecycle 事件无法复活旧实例。关闭后保留已形成结果、证明与发布任务，但旧分支迟到结果不能扩展新 G。

在途 Force 激活前若已发生同分支 regular，继续按激活时当前 Tail 和 G 计算。请求签署时的 cid、anchor、r 与激活时不同，均由上述规则处理。

## 5. 时间 profile 与期限

所有协议时间使用整数微秒。主 Force profile 固定 seed 42、正向启动、相互独立的 15 个 worker、每节点时钟偏移在 ±1000 μs 内、Clock RPC 单向延迟 250 μs、区域链路延迟 4000 μs、显式工作耗时 5000 μs、启动错时步长 3000 μs。沿用 M1 的 L2 genesis timestamp 10 s、L2 slot 1 s、L1 slot 2 s、`M_a=10 s`、`T_grace=3 s`。M2 采用 `T_init=3 s`、`W_exp=8 s`。原 M1 回归沿用其原 profile。

冻结的健康预算为 `T_L1≤2 s`、`delta_L1=0`、`Delta_ts≤1 s`、`T_E≤1 s`、`T_replay≤0.1 s`、`T_state≤0.5 s`、`T_produce≤0.5 s`、`T_pack≤0.05 s`、`T_pub≤0.2 s`。run profile 写明各预算对应的模拟事件、服务边界和实测耗时。主场景在 R0 已健康生产时签署 Force，真实 L1 收录 timestamp 为 12 s，故 `act=22 s`。提交后让 h10 正常进入 G，R0→R1 regular 确实发生。R1 原生 h11 结果的公开交付延迟至 act 同键或之后，使激活前实际 G head 为 h10，当前 NEXT 为 `(beta=0,r=1)` 且尚无贡献。Force 因而选 `(beta=1,r=1)` 的 R2，窗口为 `[16,25]`。R2 的五个 worker 已持续存活，首次进入其获选 cid。延迟的真实 R1 h11 仍保留完整证明与正文并用于迟到结果拒绝检查。此一主运行同时覆盖主动 Force、在途 regular、变化后的 anchor、无贡献序号和真实迟到结果。

至少检查以下不等式和实际事件。

- `M_a >= T_L1+b_L1+Delta_ts+delta_L1`。
- `T_grace > T_pub+T_E+Delta_ts+delta_G`。
- `T_init >= Delta_ts+T_E+T_replay+T_state+delta_G`。
- 新 cid 声明参与进展的至少四个固定成员，在 `ts*(h_min)-delta_G` 前完成必要状态与时钟准备，并分别 Enter。记录全部五名目标成员的 Ready/Enter，实际共同参与起点为其中声明 quorum 全部 Enter 且条件齐备的时刻。
- 各节点 proposal 遵守本地 `NotBefore`，签名出口再次检查 lease、terminal 和本地 stop。旧 cid 本地 terminal 提交后新签名数为零。
- 本健康场景在控制在途期间持续接纳旧分支结果，以激活前实际下一缺失高度 h11 核对恢复期限。`Expire(h11)=29 s`，`ts*(h_min)=26 s`，准备截止为 `25.999 s`。上述预算给 `22+3+1+0.5+0.05+0.2+1+1=28.75 s < 29 s`。逐阶段实测还须满足各自声明值。v2 §8.4 的提交时静态保守预测只用于其适用前提，主场景保留在途期间填入高度的真实证据。
- 实际新结果的接纳键须严格小于本次新增全部 virtual heights 的各自 Expire 键，尤其包括 `A.h+1` 至首个 physical height 的 padding。同时满足被接纳 session 的公开截止。

预算是本次可复现健康实验配置。每项记录声明值、观测值和适用前提。配置或界限必须改变时先修订合同/profile，重新命名候选并复核受影响案例。墙钟耗时只作为基础设施诊断。

## 6. 验收矩阵

| ID | 层级与触发 | 必须观测的结果 |
| --- | --- | --- |
| HF01 | 15-worker 主运行，健康持续生产后合法单签 Force | 真 Anvil 收录、finalized/reader 前缀、公共覆盖、一次激活。激活前 G 比签署时更新，actual G anchor 正确，旧 G 保留，新 branch 首个真实 Final 经 Publisher/SimEspresso 进入 G |
| HF02 | 同 SHA 同 seed 主运行复跑，注入墙钟暂停 | 规范化后的完整虚拟协议 trace 一致，原始文件全部保留，PID/路径/墙钟等诊断字段按预先规则规范化 |
| HF03 | HF01 同一次 15-worker 运行中的在途 regular | Force 提交后、激活前同 branch 发生 regular。Force 仍按当前 Tail 生效，cid、anchor、r、region 和窗口正确，首个新结果进入 G |
| HF04 | 独立 GFR 向量，NEXT 有贡献、无贡献、PENDING | 分别得到 `r+1`、`r`、Boundary 保存的 r/source region。floor、Ref、窗口和唯一 cid 可独立重算 |
| HF05 | 独立 GFR 向量，Force 与旧 publication/SessionEnd 同键 | 控制先处理，旧结果不能进入新 G，旧 timer 按当前状态重新检查适用性 |
| HF06 | 独立 GFR 向量，Force 与下一缺失高度 Expire 同键 | Force 按有效性先处理，随后仍适用的 Expire 使新 branch HALT，Force 保留的 G 未增补 |
| HF07 | 同 L1 block 多控制，位置和重复投递 | 按 tx/log 顺序逐条处理，后一条读取前一条的状态，重复前缀与事件不会重复激活或建实例 |
| HF08 | 生命周期与异步边界向量，关闭时存在 Running/Preparing/Paused 条目 | 全部永久 terminal，terminal 持久提交先于 Close，迟到准备/Enter/签名工作无法复活或新签名，已形成结果仍可读取 |
| HF09 | 主运行保留旧结果，并以同键或更晚公开位置交付 | proof 可以有效，GFR 仍因旧 branch/cid 排除接纳，G 无错误追加 |
| HF10 | 独立 reader 覆盖缺失，依次补齐 L1/Espresso/必需材料 | 原位 WAIT，无部分转移。A/B 延迟不同，在同 replay position 的 G、Tail、branch、descriptor 与 lifecycle 相同 |
| HF11 | stale target 与 HALT 控制向量 | 有真实合法控制输入，GFR 记录无效控制，G/policy/branch/lifecycle 无副作用，HALT 保持 |
| HF12 | 真 Anvil 无效签名、过期序号、冲突 request、合法重复 request | 前三项明确拒绝且无新增控制日志，完全相同请求幂等返回原接纳事实且无第二条 ForceRecorded |
| HF13 | 同候选 M1 健康满窗回归 | 15 PID、h10/h20 两次 regular 与 h21 入 G，原 profile 下通过既有独立 checker |
| HF14 | SessionEnd regular 回归向量或已有可运行入口 | h10 未及时公开时以 h9 为 anchor，后继仍为 [11,20]，合格 h11 才确定 padding10。报告明确记录运行层级 |

主运行层级使用持续运行的 15 个真实 worker 和真实 native Final。规则向量使用生产 GFR/Manager/签名守卫入口及独立预期，逐项报告其层级。HF08 的三种生命周期覆盖和 HF04 的 PENDING 可以通过独立向量提供，结果不能推定为主运行已出现对应状态。HF14 若当前只有规则向量，交付准确保留这一范围，完整 M1 端到端 SessionEnd 路径继续按既有合同跟踪。

## 7. 证据与交付判定

每次使用新的 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m2-healthy-force/artifacts/<run-id>/`。保留 manifest、候选完整 SHA、干净状态、合同/预期 hash、BFT gitlink、Cargo locks、工具/二进制 hashes、seed、argv、输入和 profile、控制 calldata/签名/receipt/header/log、15 PID 与退出状态、原始 grant/Ack/frames/journal、proof/body、L1/SimEspresso 前缀、trace 和独立检查报告。记录 submit/include/finalize/reader-available/activate、全部 worker Ready/Enter/terminal/Close、regional-final/public-complete/G-accepted 的事件索引与四类时间。

Verifier 先从来源与合同生成预期，再读取候选。独立检查重新计算控制签名、公共顺序、最新 anchor、Plan/window/cid、执行状态、四签 native Final、G 接纳与生命周期守卫。候选自报 PASS 不作为独立结论。Tester 在 Verifier 复核的同一命名 SHA 上运行全部约定案例。源码变更后重新复核并运行受影响案例。

所有构建和重型仿真复用 `/tmp/megaroation-poc-heavy.lock`，同一时间一个重任务，`CARGO_BUILD_JOBS=2`，Rust 1.95.0，锁定依赖。runner 已持锁时外层不重复加锁。单次主运行墙钟 watchdog 1800 s、最大虚拟 tick 120 s、最大事件数 1,000,000。达到上界前未达成功终点记录 FAIL/INCOMPLETE，并保留全部已产生证据。

交付包含已独立验收的候选完整 SHA、实际运行命令、离线检查命令、证据路径及逐项结果。HF01 的完成点是新分支首个合格结果实际进入 G。合同冻结本身没有运行结论。
