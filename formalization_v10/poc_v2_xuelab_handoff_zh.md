# Sequencer Rotation PoC v2：xuelab 交接

日期：2026-11-14。交接目的：在 `xuelab_jump` 启动新的 Ultra chat，按既有 Regional BFT 多 agent 流程继续 PoC 搭建。

## 1. 首先读什么

1. 本文：任务范围、当前成果、授权与下一步。
2. [v2 设计](poc_design_zh_v2.md)：当前实施依据；v1 只供历史对照。
3. [远端搭建说明](poc_v2_xuelab_setup_zh.md)：目录、工具、协作分工与启动检查。
4. v10 的 [02 Session Program](02_session_program_zh_v0_1.md)、[03 Manager](03_session_manager_zh_v0_1.md)、[09 GFR](09_finality_rules_zh_v0_1.md)、[11 Claims](11_global_chain_safety_liveness_zh_v0_1.md)。
5. 接收目录根部的 `AGENTS.md`、`DESIGN.md`、`.codex/agents/`；后端参考仓库 `/mnt/reth/jy/regional_BFT_sim` 的 `README.md`、`AGENTS.md`、`DESIGN.md`。

## 2. 用户已经确定的范围

- 核心是 regular、Force、RESUME 三类 handoff；不是再做 BFT 性能排名。
- 单服务器 3 region、每区 5 个等权 replica；最终需要 15 worker、两条链的本地环境、治理服务和 Web Observer。
- 每 session 固定 10-height window；不强制故障情况下也产生 10 个 physical blocks。
- 第一阶段不实现 staking/slashing。公开确定的 `(β,r,v)→(region,leader)`；保留未来 staking provider 的接口。
- 复用 Commonware Simplex 与普通 BLS aggregate/multisignature；5 节点 profile 用 4 个不同 signer 的 quorum。
- 统一 RPC 虚拟时钟；replica 相对虚拟参考时间误差不超过 1 ms。RPC 延迟单独建模；不是“1 ms 内返回就证明时钟同步”。
- 每个 replica 的 Create/Ready/Enter/view 开始可以不同步；不能等全员就绪后统一启动来掩盖问题。
- L1 使用 Foundry/Anvil 手动出块；第一阶段仅 ControlAuthority/Inbox 合约，无 token/staking。
- Espresso 主测试使用确定性日志模型；官方 dev-node 单独兼容测试，不能声称其已支持外部虚拟时钟。
- `dep[h]` 固定输入、轻量执行；Publisher、Batcher、Certifier、DA 支持延迟/crash/restart。
- Batcher/Certifier 是 crash-only；不研究恶意批次。D 必须从 L1/DA 独立派生，不直接复制 G。
- 治理组织用预置公钥的单一可信 Agent M；Force 依据当前职责的公开生产停滞；RESUME 依据已验证分歧或符合规程的 HALT/D 推进，并检查恢复条件。
- TLA、形式化证明、生产部署、真实资金和云性能实验不在本次范围。
- 用户已请求使用多 agent；按 Doc → Implement → Verify → Simulation，先冻结接口，再并行独立验收准备与一个 writer。

## 3. 已有成果与尚未完成

已经完成：v1/v2 中文设计、官方工具与本地 Commonware 接口查阅、文档链接/格式、示例参数不等式和 schedule 小检查。

**尚未完成：** PoC Rust 工程、Clock RPC、受控多进程 runtime、5-node Session Program adapter、合约、治理策略代码、D、Web Observer以及端到端实验。设计稿中的接口与参数不是已运行结果。

现有 Regional BFT 工程有真实 Simplex engine、BLS multisig、模拟链路、journal 和实验基础。但旧 benchmark/5-of-15 编码 profile 不代表 3×5 个独立委员会已实现。部分路径仍固定 `COMMITTEE_SIZE=15`；旧文档也明确不代表完成 v10 Session Program 集成。

## 4. 源码快照与当前修改

本次将“checkout current modification”解释为**保留并检出当前工作区快照**。没有对本机执行 discard/reset，也没有将远端已有修改混入本地规范。

源仓库：`/Users/jin/projects/megaeth_sequenser_rotation`；源分支 `clock-skew`；读取时 HEAD：

```text
476ee764ebd38f820f7aab7d20100436d45c140c
```

快照包含以下已有工作区修改的实际内容：

| 文件 | 状态/主题 |
| --- | --- |
| `01_l1_authority_zh_v0_1.md` | 现有修改，授权/恢复历史语义 |
| `04_publisher_zh_v0_1.md` | 现有修改，完整结果及发布前提 |
| `08_da_zh_v0_1.md` | 现有修改，数据可取回/保管前提 |
| `09_finality_rules_zh_v0_1.md` | 现有修改，q_avail、公共输入等待、整段接纳等 |
| `10_pipelines_zh_v0_1.md` | 现有修改，端到端时序与条件 |
| `11_global_chain_safety_liveness_zh_v0_1.md` | 现有修改，G-S1/2/3 与条件活性；保留用户批注 |
| `poc_design_zh.md`、`poc_design_zh_v2.md` | 本 chat 的两版设计 |
| `meetup_record/261110_chain_g_safety_liveness_zh.md` | 已有未跟踪会议记录，一并保留 |
| 本交接文档与搭建说明 | 本次新增 |

精确状态、binary diff 和文件 SHA-256 以交付包的 `handoff_provenance/` 为准。主题摘要不是对现有数学论证的重新验证。`megaeth_specs` 来源提交为 `a738f9b14c7880953d8cff8788db14e8e3e79124`，本次携带其文档内容；不传递其 `.git`、任何 SSH key、凭证或代理订阅。

远端参考 BFT 仓库读取时 HEAD 为 `2b508adb9dec368f8a22021129361d22b746ffce`，branch 为 main，存在：

```text
 M README.md
?? examples/consensus_replay.rs
?? examples/consensus_replay/
?? visualization/
```

这些属于远端已有工作，**不得覆盖、清理、重置或随 PoC 提交**。本机 BFT HEAD 与远端不同，不把本机整个 BFT 目录同步覆盖远端。

## 5. 接收目录与隔离方式

接收端独立 checkout：

```text
/mnt/reth/jy/regional_BFT_sim/poc/sequencer_rotation_v2
```

放在已有远端 Codex 项目的目录内，便于新 chat 获得该项目的访问范围；内部为独立 Git 仓库和独立 PoC baseline。它不会变更父仓库已有 tracked 文件。父仓库可能显示新增 `poc/`；不要为隐藏它而改父仓库 `.gitignore` 或提交父仓库。

接收 checkout 的 snapshot commit 由实际交付时生成，记录在接收记录中；它是工作区内容快照，不是源仓库 HEAD，也不是已验证实现 revision。

## 6. 第一项任务：只先打通时间与多进程接口

新 chat 应先完成以下动作，然后自主向 v2 的后续阶段推进：

1. 核对接收清单/hash、实际 cwd、Git 状态与已交付配置；保存 intake/preflight 报告。
2. 确认 Ultra reasoning 在实际 chat 生效；若模型/effort 不支持，明确报告，不静默降级。
3. 由 `bft_spec` 写 I1 task contract：Clock RPC、样本/租约、timer、controlled grant/ack、网络发送、journal 的边界与验收。
4. `bft_verifier` 先从 v2 与 Commonware 锁定源码推导验收，不先看 builder 的实现解释。
5. `bft_builder` 只在冻结接口及分配路径内创建最小 Rust workspace、两个 worker 小样；暂不同时上 15 节点、Web 和完整链。
6. verifier 检查候选；simulator 对同一集成 revision 做固定 seed 重放和不同墙钟停顿对照，输出虚拟 trace。
7. I1 通过后依次推进 15 nodes/regular、组织/Force、D/RESUME、Web 与综合矩阵。Web 可从 regular 阶段开始建设。

I1 必须证明的是实现行为：一个时间权威、没有 worker 自行快进、RPC 延迟不会被墙钟偷偷代替、时钟 lease 不被 grant 免费刷新、迟到/重放回复处理明确、所有协议输出被收敛到可审查的事件顺序。

若发现 Commonware 外部 runtime 适配不满足可控执行的要求，先写出具体接口缺口和最小替代方案；不能在没有说明的情况下把多进程验收改成只有 15 个共享状态的函数调用。

## 7. 特别容易回归的协议点

- `Clock::current()` 是同步接口，不能在其中阻塞等待网络；RPC 样本与本地受控 ClockView 桥接。
- 误差界为 `δ_G=1 ms`，两两差可以达到 2 ms；Enter 时差可以远大于它。
- 活性从同 cid 的共同参与区间起算，不能从第一个 Enter 起算；3/2 候选分裂不是一个 quorum。
- GFR 按公共日志排序；本地 clock 到期不能绕过 watermark 直接 SessionEnd/Expire。
- Force 在途期间 regular 后若 branch 未变仍可能有效；不要自行加入 cid 变化则无效的规则。
- HALT 不能被 Force 唤醒。
- RESUME 采用具体 L1 position 的历史 ResumeCut；D 不回退，G 可以换后缀。
- G/D 的 hash 分歧与 D 只是落后/更高必须区分。
- 已接纳 DA 不可读必须 WAIT，不得按未提交补空块。
- 单一治理方的策略是 PoC 选择；不把 timeout 当作恶意证明，不引入 slash。
- 最终证明、公开完整、G 接纳、Inbox 接纳、D 派生是不同事件。

## 8. 交付记录的读取方法

接收目录有 `handoff_provenance/manifest.json`、`source-working.patch`、`source-status.txt` 和 `SHA256SUMS`。实际上传包 hash、远端 snapshot revision、chat id、host 与开始状态另记录到 delivery receipt；这些记录用来核对本次交接，不是 PoC 通过证据。

请新 chat 在第一轮明确回复：实际工作目录、收到的 v2 hash、模型/effort可见状态、多 agent 分工、第一项 gate、当前可用工具/缺失依赖。此后按授权继续工作，不必再询问是否开始创建基础工程。
