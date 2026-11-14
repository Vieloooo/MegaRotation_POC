# Espresso 时间驱动的区域轮换：正确性评估与修订计划 v3

**日期：2026-10-17。**
本文先评估本轮提出的设计，再固定编写新架构和协议提纲所需的默认规则。
配套文件为 [Architecture v3 提纲][ARCH3] 和 [Part II v2 提纲][PART2V2]。
本轮不直接改写原 [arch_v2.tex][ARCH2] 或 [Part II TeX][PART2]。

## 1. 评估结论

**这个方向可以形成条件性安全的协议，而且能保持正常轮换不等待全局确认。**
你的四项设定之间没有必然冲突：

1. L1 最终化服务时间覆盖“实际提交 → 所有满足模型条件的正确参与者能够看到并验证 finalized 记录”，包括 certifier。
2. 多签控制、活性更新和轮换都以 Espresso timestamp 定位生效边界。
3. Block-missing Force 由确定性重放规则自动导出，多签 Force 由提前公告的 L1 指令触发。
4. Publisher、Espresso 和 Certifier 分别承担输出传送、外部最终排序、跨链验证职责。

但这仍不是一个已经完成证明的协议。
需要补齐的主要内容是**确定性事件顺序、可验证的进展指标、正常交接的历史选择依据，以及完整 L1 控制视图**。
这几项决定两个正确观察者是否会得到同一个 G 和 successor。

本轮采用下表中的规则把这些缺口具体化。

| 必须明确的问题 | 本轮默认规则 | 不明确时的后果 |
| --- | --- | --- |
| 没有按时产生足够块，具体指什么？ | 按 export session 的固定 Espresso 时间窗口，计算真正进入 G 的 DATA 数量 | 不同观察者可能用本地收包、原始 inclusion 或各自窗口作判断 |
| 正常生产已经领先，自动检测谁？ | 检测“负责提供下一个全局 DATA 的 session”，称为 export session | 将 G 的旧 owner、最新生产 leader 和发布故障混为一谈 |
| 截止点上的 DATA 是否及时？ | 服务窗口为半开区间；timestamp 等于 deadline 的 DATA 不属于旧窗口 | 同 timestamp 的多个 Espresso 区块可能产生回溯救活 |
| 多签更新、两类 Force 与 DATA 同时出现怎么办？ | 先状态更新，再有效多签 Force，再自动 Force，最后处理 DATA | 一部分节点继续旧分支，另一部分节点已经切换 |
| Espresso 时间大幅跳跃怎么办？ | 新强制 session 从实际触发区块的 timestamp 获得新的启动预算 | 一个区块内可能连锁耗尽所有 successor 的过期计时器 |
| 活性 off 是否立即使旧凭证失效？ | 普通状态更新影响选择资格；要停止已打开分支，使用多签 Force | 对历史授权或已提前启动的 session 产生追溯改写 |
| Certifier 能否使用任意有效 L1 snapshot？ | 不可以；需要完整且足以覆盖当前 Espresso 时间的控制视图 | Host 可提供遗漏 Force 的旧视图，让错误分支通过 |

## 2. 接受并固定你的时间模型

### 2.1 一个完整的 L1 服务时间界

记 `T_L1` 为：控制操作在真实时间 s 实际提交后，到 s + T_L1 时，所有符合模型在线与同步条件的正确参与者都已能取得并验证其 L1 finality 及完整控制记录。
这里已经包括 inclusion、finality、传播、读取和验证，不再额外加一段 announcement 时间。
原 Part II 的 Force finalization 假设已经采用了相近的服务定义。[现有模型][PART2]

这是本轮接受的模型假设，不是把 Ethereum 的典型最终性时间当作无条件上界。
离线、新加入或失去必要控制视图的节点先同步，不能直接宣布新的 global finality。

### 2.2 Espresso timestamp 的性质

记 Espresso 区块 e 的认证 timestamp 为 `tau(e)`，固定单位和整数比较规则。
基础模型要求：

- 区块高度形成确定顺序，timestamp 不倒退；timestamp 相同不等于区块位置相同。
- Finalized header、namespace 完整内容或真实缺席都可验证。
- 为安排提前控制，timestamp 不能相对实际最终化时间任意超前。
- 为推进计时器，正常服务下 timestamp 最终继续增长，且所需历史可以获取。

若采用上界 `tau(e) <= t_E_final(e) + epsilon_E`，多签在时间 s 提交控制时可保守地选择：

```text
activate_at > s + T_L1 + epsilon_E + margin
```

这使控制记录在 Espresso 越过该时间之前已经公开可验证。
也可以直接把“每条控制都在截止前完成该服务”作为抽象假设。
具体 Espresso 版本是否满足所选时钟界，需要在实现阶段固定和核对；存在 timestamp 字段本身不是这个界的证明。

### 2.3 生效时间与实现该事件的位置

对一个 Espresso 时间阈值 d，定义：

```text
cross(d) = 第一个满足 tau(e) >= d 的 finalized Espresso 区块
```

阈值 d 是名义生效时间；真正的公共状态转换发生在 `cross(d)` 开头、处理该区块 DATA 之前。
因此无需 Espresso 运行智能合约，也无需它主动生成一条系统交易。
节点和 certifier 从同一条日志导出同一个事件。

## 3. 全局最终性规则

一个区域 DATA 块在以下条件同时成立时获得 global finality：

1. 有完整数据和有效 regional FINAL，或同 session 的合法覆盖凭证。
2. 它的 session、固定 authority、正常交接依据和 branch epoch 在该位置有效。
3. 它与必要祖先构成当前 G 的完整连续扩展。
4. 承载它的 Espresso 记录已 finalized，且记录之前所有应生效的控制和自动事件已经按规则处理。

不再等待 `T_mon + T_msig + T_L1`，也不等待 Espresso 状态回传到 L1。
多签 Force 和自动 Force 都只关闭未来追加资格；保留此前完整 G。

这里的保证首先针对 DATA 的内容、顺序和来源身份。
执行结果的正确性、原生签名接续、相关 L1 执行输入的最终性及最后的发布/结算，仍分别依赖其接口和假设。

## 4. 两类 Force 的定义

### 4.1 Block-missing Force

它是 replay 的确定性输出，不是多签请求，也不需要额外提交到 L1 或 Espresso 才能生效。
其检查对象是当前 **export session**：按已验证的分支与 quota，负责 G 下一高度的 session。
正常生产可能已经推进到后面的区域；那些更后的记录不能跨过当前缺口直接计作进展。

初版采用固定窗口，而不只检查“多久完全没有块”：

- 启动窗口长度 `W_start`，至少接受 `m_start` 个 DATA。
- 后续窗口长度 `W_run`，每窗至少接受 `q` 个 DATA。
- 目标数不超过该窗口开始时的剩余 quota。
- Timestamp 等于截止点的记录不计入旧窗口。
- Session 提前完成 quota 时，取消它剩余的检查并转到 successor。

这样能够区分“完全停止”和“持续但不足以满足服务要求的低速交付”。
进展按进入 G 的 DATA 计，不按 publisher 请求次数、重复凭证、heartbeat、DRAIN 或原始 Espresso inclusion 计。
有效空 DATA 的计数政策必须固定；沿用当前设计时它可计入 DATA quota，但不代表抗审查。

窗口失败时，保留 G，关闭旧 branch epoch，暂时暂停相应 export 服务区域，并选择 successor。
这是交付失败的恢复政策，不自动证明该区域、某个 proposer 或 publisher 作恶。

### 4.2 多签 Force

多签仅向 L1 提交带有 `activate_at`、目标 branch epoch 和控制 nonce 的指令。
它不要求 block-missing 条件成立；例如，即使区域仍输出，也可以因风险判断而隔离。

在 `cross(activate_at)` 开头生效，保留此前 G，关闭旧 branch epoch，按新控制状态选择 successor。
不额外等待一次 Espresso→L1 证明提交，更不允许多签提供任意更旧的 retained height。

两种 Force 共用同一个关闭与 bootstrap 函数；不同的是触发证据与管理状态变更。

## 5. 活性状态必须区分来源和作用范围

建议拆成：

```text
GovernanceAllowed_i(tau)   // 来自 L1、按 activate_at 生效
CooldownUntil_i           // 来自自动 Force 的确定性派生
AuthorityEligible_i       // 来自固定 staking / authority
ReadyHint_i               // 可选的就绪信息，不是最终性证据
```

用于选区的有效资格由前三项共同决定。
自动状态由 Espresso 历史与规则推导；L1 可以通过后续 certifier checkpoint 登记它，但该登记不处于自动 Force 的关键路径。
因此“控制权在 L1”不等于“每次自动状态变化都要等一笔 L1 写入”。

多签 `OFF` 的默认含义是改变未来选择资格，不追溯撤销已经绑定历史选择依据的 session。
如果要停止当前分支或取消其提前打开的 successor，使用多签 Force。
`ON` 恢复选择资格，不复活旧 branch epoch，也不自动清除独立的 cooldown；需要清除时应有明确的管理字段。

## 6. 正常轮换的选择依据

普通 handoff 继续使用 terminal FINAL、完整状态、历史 authority 和确定的 successor 选择，不等待该 terminal block 的 Espresso finality。
为了让所有人得到同一 successor，terminal block / witness 必须绑定一个已经 finalized 的 Espresso 选择参考点 `selection_ref`。
根据其 timestamp 读取历史生效状态，而不在 witness 到达或发布时重新读取“最新 live map”。

参考点需要被区域签名内容绑定、可认证、沿正常历史不倒退，并满足 session 的参考点下界。
第一版采用这个保守的历史授权语义，不声称它证明真实交接时刻的最新健康状态。

**这是一项需要明确接受的代价：**在物理时间上较晚启动的 session，仍可能使用较早的合法选择参考点。
如果要求一个区域从某个 Espresso 时间起绝对不得继续当前生产分支的服务，必须使用 Force，而不是依靠一个被追溯解释的 OFF bit。
以后可以增加更强的 freshness / reservation 规则，但不能把“最新”留作不可验证的口头条件。

## 7. 明确的边界处理顺序

每个 finalized Espresso 区块开头，先找出所有在此区块首次跨越阈值的管理事件和当前 deadline。
本轮采用：

1. 按名义生效时间与 L1 控制序号，应用到期的资格更新。
2. 应用有效且目标匹配的多签 Force。
3. 对仍然有效的 export session 检查到期窗口；失败则导出自动 Force。
4. 最后解释本区块中的 EXPORT，按 namespace 顺序处理。

多签 Force 成功后，旧 session 的自动计时器被关闭，不再重复切换。
新强制 session 的启动窗口从实际触发区块的 timestamp 开始，不能从已经过去很久的 deadline 开始。
若没有 successor，就进入等待状态，不在同一个区块内不断尝试并超时。

这个顺序也覆盖 Espresso timestamp 跳跃的情形：多个阈值可能映射到同一实际边界，必须按统一优先级解释，不能由本地到达顺序决定。

## 8. 修订范围与工作顺序

| 阶段 | 修订产物 |
| --- | --- |
| 1 | 固定时间、global finality、窗口、活性状态作用域和事件优先级 |
| 2 | 写 Architecture v3：Publisher、Espresso、Certifier 三个独立模块及其共享对象 |
| 3 | 写 Part II v2：正常交接、export session、两类 Force 和公共 replay 状态机 |
| 4 | 接入完整 L1 控制历史、Espresso 证明和 certifier attestation 的版本绑定 |
| 5 | 核对 native mini-block / EVM continuation、精确发布区间和 DA / 执行关系 |
| 6 | 给出条件性安全论证、活性条件与边界案例，再决定参数和实现 profile |

优先修订原 Part II 的 global-finality、Force 与 observer 章节；区域内部 steady state / recovery 尽量维持现有语义。
Publisher 的职责包含完整数据与凭证扩散、首发/补发和确认跟踪，但奖励与罚没政策单列，不以“超时”自动证明恶意。

## 9. 本轮尚不声称完成的部分

- 具体 Espresso 版本的 timestamp 约束、时钟偏差和历史证明 profile。
- 所有必要 L1 控制的完整性与 certifier 输入新鲜度的最终实现。
- 正常选择参考点的更强 freshness 政策。
- `W_start`、`W_run`、`q`、cooldown 等数值；需要根据区域恢复、发布备份和 Espresso 服务条件确定。
- 旧 engine 的提前停止、数据保留和 staking retirement 的完整组合。
- 原生执行状态接续和强全局前缀语义与 rollup 派生规则的适配。

这些是应写入后续正式协议的具体接口或证明义务，不应以“Espresso 保证安全”略去。

[ARCH3]: arch_v3_outline_zh.md
[PART2V2]: sync_hotstuff_rr_part2_v2_outline_zh.md
[ARCH2]: arch_v2.tex
[PART2]: sync_hotstuff_rr_part2_rotation.tex
