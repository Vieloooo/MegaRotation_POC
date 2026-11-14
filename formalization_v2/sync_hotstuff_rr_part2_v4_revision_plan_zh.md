# Part II v3.7 → v4 修订计划

日期：2026-10-19

基线：[Part II v3.7 英文正文](sync_hotstuff_rr_part2_rotation_v3_7.tex)、[中文说明](sync_hotstuff_rr_part2_rotation_v3_7_zh.md)。

当前方案：**只保留 Hard Force。缺块检测和是否触发 Force，全部移入 multisig 策略。**

性质：修订方案，尚未生成 v4 正文，也不表示下述新规则已完成形式化验证。本版替换此前计划中的 AUTO、deadline 和 Force gap 方案。

## 1. 这轮要得到什么

**v4 的 CL 负责执行明确的交接规则。multisig 负责决定何时需要强切、哪些地区应当 ON 或 OFF。**

根据本轮讨论，采用以下范围：

1. 删除 Owed set，不保证每个地区都轮到服务。
2. 允许一个地区被关闭后一直保持 OFF。协议不负责自动恢复资格。
3. 交易重投和未确认交易的保留由 EL / miniblock builder 处理。CL 不承诺请求最终被包含。
4. **删除全部跨地区 AUTO Force 逻辑。** 缺块、交付速度和健康检测属于 multisig 策略。
5. **只有有效的 L1 MSIG 指令才能触发跨地区 Force。** 某个缺块条件成立，本身不会改变 CL 状态。
6. **Hard Force 可以连续执行。** 不要求经过 Force gap，也不要求新地区先 Ready、产出一个完整块或推进 \(G\)。
7. 保留旧 \(\beta\) 匹配检查。MSIG / AUTO 双检查及两者之间的优先级一并删除。
8. 保留 Ready 上界作为准备服务假设，但它不再决定 Force 何时生效。
9. regular rotation 仍走原有 terminal receipt、DATA / DRAIN 和 Ready 路径，不增加全局确认等待。

这里删除的是**跨地区强切计时与公平机制**。区域内部 BFT 的 leader schedule、同步条件、故障界、提交计时器、BLAME、Quit 和恢复规则保留。

## 2. 为什么可以直接从 v3.7 改

v3.7 已经有本轮需要的接口：

- 每个 \((\beta,r)\) 独立运行，完整 view 为 \((\beta,r,g,t)\)。
- Session Manager 观察 L1、Espresso 和区域 BFT 结果。
- regular rotation 使用 terminal receipt 和原有 Ready 检查。
- Force 关闭整个旧 \(\beta\)，保留触发位置之前已经接受的 \(G\)。
- Publish Layer 发布完整区域结果；observer / certifier 重放同一套接受规则。

因此，这轮不继承 v3.8–v3.11 为请求活性增加的 owner、输入交接、HYDRATED、OC、INPUT_REPORT 等机制。以 v3.7 的区域引擎和最终确认边界为基线，集中简化跨地区选择、控制和活性声明。

### 2.1 删除、移出和保留

| 当前机制 | v4 处理 | 原因 |
|---|---|---|
| Owed set、服务轮次、已偿还服务记录 | 删除 | 不再保证各地区公平获得服务 |
| PrepareOwed、NextOwed、DATA 对 debt 的扣减 | 删除 | 接班直接使用 NextLive |
| 必须预先安排 ON 的 OFF lease | 删除 | OFF 可以无限期持续 |
| 按 \(G\) 或地区产块量发放的 MSIG / lease / CLEAR credits | 删除 | 本轮不限制治理干预频率，也不保证资格恢复 |
| AUTO detector、自动 Force 事件及其证明字段 | 从 CL 删除 | 监测和触发决定移入 multisig 策略 |
| \(M=1+\beta\)、连续 miss、失败窗口、批量结算 | 删除 | CL 不再因缺块自行关闭分支 |
| 上一版计划的 \(H,W\)、deadline 和 Force gap | 删除 | 不再需要 CL 的自动强切时钟 |
| AUTO OFF、cooldown、CLEAR | 删除 | 地区资格只由显式 L1 ON / OFF 改变 |
| 缺块和交付速率阈值 | 移入后续 multisig 设计 | 可作为提交 MSIG 的依据，但不是 CL 的接受条件 |
| CL 的 pending-request 保留、FIFO、请求最终包含要求 | 删除或移交 EL | 不是本轮 CL 的责任 |
| 区域 DATA / DRAIN / FINAL / Quit / Completion | 保留 | 属于区域共识及交接正确性 |
| regular rotation 的 terminal receipt 和直接传输路径 | 保留 | 不给常规接班增加全局确认等待 |
| 精确 anchor、authority、执行状态检查 | 保留 | Ready 必须知道接着哪个状态执行 |
| 已接受 DATA 的可取回性及必要证明保留 | 保留 | 与交易 mempool 保留不是同一件事 |
| MSIG 的旧 \(\beta\) 检查、未来公告、确定性顺序 | 保留 | 指令必须有明确作用对象和截止位置 |
| 外部解释器的逐 DATA 接受、持久化 cursor、不可回滚 \(G\) | 保留 | 最终确认安全性不随治理策略变化 |

相应的状态、处理分支、符号、图和证明一起删除，不能只改介绍段落。

## 3. 简化后的模块关系

原来的 AUTO detector 合并进 multisig 的策略职责，不再是 CL 中的独立模块。跨地区设计保留三类职责：

1. **Multisig controller**：观察、判断、提交 L1 控制。
2. **Session Manager**：根据已验证的控制和区域结果管理各个 session。
3. **Ready / Enter**：确认准确起点和执行状态，然后进入区域 BFT。

共同的来源验证和重放仍由节点、observer、certifier 执行，不增加新的可信控制者。

~~~mermaid
flowchart LR
    HEALTH["区域健康与运维观察"] --> GOV
    GOV["Multisig controller<br/>缺块检测、健康策略、签名提交"] --> L1["L1<br/>AUTH / ON / OFF / MSIG"]
    PUB["Publish Layer<br/>发布完整区域结果"] --> ESP["Espresso<br/>有序日志、最终确认、数据读取"]
    L1 --> REPLAY["共同验证与重放<br/>控制、当前 session、G"]
    ESP --> REPLAY
    REPLAY -->|"已验证的进展与责任地区"| GOV
    REPLAY --> HANDLER["Session Manager<br/>管理多个 beta,r 程序"]
    HANDLER --> GATE["Ready / Enter<br/>准确 anchor、状态、authority"]
    GATE --> BFT["区域 BFT<br/>DATA / DRAIN / FINAL"]
    BFT --> PUB
    BFT -->|"直接 terminal receipt"| HANDLER
    EL["EL / miniblock builder<br/>交易重投、状态恢复接口"] --> GATE
    EL --> BFT
~~~

multisig 的控制输出只通过 L1 进入协议。健康报告、检测器输出或私下通知都不能直接关闭某个 \(\beta\)。

Espresso 继续提供有序历史、最终确认和数据读取。**不要求 Espresso 执行本协议的智能合约。**

| 模块 | 输入 | 输出 | 权限边界 |
|---|---|---|---|
| Multisig controller | 已验证的 \(G\)、责任地区、健康观察、运维要求 | 未来生效的 ON、OFF、MSIG | 决定是否干预；不能选更旧的 anchor 或撤回 \(G\) |
| 共同验证与重放 | 完整 L1 / Espresso 历史及证据 | 资格状态、有效 Force、固定边界、更新后的 \(G\) | 验证控制有效性，不重做 multisig 的健康诊断 |
| Session Manager | 已验证的公开更新、区域 receipt、本地任务完成事件 | 准备、进入、关闭和清理 session | 不因本地检测结果直接改变 \(\beta\) |
| Ready / Enter | 固定 handoff、准确 anchor、执行状态、authority | Waiting，或一次原子的 Ready → Active | 不等待第一块产出，也不为 Force 提供否决权 |

Waiting、Ready、Active、Frozen、Completed、Terminated 六种状态保留。Force 关闭外部分支与区域程序安全退场仍分开处理；不能直接改写区域锁、提交计时器或伪造 Quit。

## 4. 资格管理：只执行明确的 ON / OFF

### 4.1 资格与实际服务能力

- **有资格**：公开规则允许选择这个地区。
- **实际能服务**：这个地区有足够正确成员在线，且满足网络、状态恢复和区域 BFT 的服务条件。

ON 表示前者，不自动证明后者。一个完全离线的地区可能仍为 ON；现在由 multisig 发现并提交相应控制，CL 不自动关闭它。

每个地区只维护一个资格标志 \(on_i\)：

| 事件 | 更新 |
|---|---|
| L1 ON 在指定 Espresso timestamp 生效 | \(on_i\leftarrow1\) |
| L1 OFF 在指定 Espresso timestamp 生效 | \(on_i\leftarrow0\) |
| 本地缺块检测、健康报告、下载失败 | 不改变 \(on_i\) |
| 仅执行 MSIG Force，没有相应 OFF | 不改变 \(on_i\) |

\(on_i\) 由完整 L1 控制历史及其 Espresso 生效位置确定。不再有第二套来自 AUTO 的关闭状态、cooldown 或 CLEAR。

OFF 不安排自动 ON。以后是否恢复由 L1 / multisig 决定；永不恢复也是合法状态。ON 不恢复旧 session，也不重开已经关闭的 \(\beta\)。

### 4.2 接班选择

保留固定地区循环顺序和 NextLive：

1. 从原地区之后开始，寻找下一个 \(on_i=1\)、且在目标 authority 下有正权重的地区。
2. 找到后固定它，不因之后的普通 ON / OFF 重新挑选。
3. 没有其他候选时，原地区仍有资格才允许选回原地区。
4. 没有候选，就保留 anchor 和待分配边界，等待后续有效资格变化。

没有 Owed 过滤，也不检查某个地区是否已经获得过服务。缺少 authority 证据必须等待，不能把它解释成候选集为空。

**要排除已故障地区，multisig 应显式提交 OFF。** 仅提交 Force 不自动意味着把原地区关掉。否则当它是唯一有资格的地区时，NextLive 仍可能选回同一个物理地区，只是 session 的 \(\beta\) 已经变化。

### 4.3 OFF 与 Force 的分工

| 需要的效果 | 使用的控制 |
|---|---|
| 禁止某地区在以后的选择中入选 | OFF |
| 关闭当前 \(\beta\)，从最新 \(G\) 重新接班 | MSIG Force |
| 关闭当前分支并排除故障地区 | 显式安排 OFF 与 MSIG Force |
| 恢复未来入选资格 | ON |

单独 OFF 不撤销已固定 handoff，也不改变当前 voting snapshot。需要停止当前分支时，必须有 Force 指令。

建议新控制继续使用独立 STATUS 和 MSIG。若要同时排除 A 并强切，可以安排 OFF(A) 与 MSIG 在同一合法阈值生效；共同重放先应用独立 STATUS，再处理 Force。

独立 OFF 有自己的效力：即使旁边的 MSIG 因旧 \(\beta\) 不匹配而 stale，OFF 仍按其规则生效。multisig 后续设计需要考虑这两个指令的关系，CL 不临时猜测它们是否应当一起取消。

## 5. 缺块判断全部放到 multisig 策略

### 5.1 分清“观察到问题”和“执行 Force”

建议正文只保留这一条路径：

~~~text
观察区域交付或健康情况
        ↓
multisig 按自己的策略判断需要干预
        ↓
实际向 L1 提交有效的 MSIG；需要排除地区时另提交 OFF
        ↓
控制完成 L1 最终确认与公告
        ↓
Espresso 到达指定生效位置
        ↓
各节点 / certifier 执行同一个 Hard Force
~~~

“满足原 AUTO 条件”以后只是 multisig 的一个触发理由。它不会成为另一种 Force，也不会让指令绕过 L1 公告过程。

### 5.2 后续 multisig 设计负责的内容

本轮只列接口，不提前固定具体策略：

- 监控负责 \(G\) 后下一个 DATA 的 session，还是同时监控其他地区。
- 怎样使用 accepted DATA 的速度判断交付异常。
- 怎样区分区域离线、Publish Layer 故障、Espresso 服务问题和观察者自身落后。
- 怎样选择检测阈值、观察周期、干预间隔和重复告警处理方式。
- 什么情况下只 Force，什么情况下同时 OFF，什么情况下恢复 ON。
- 谁收集报告、谁签名，以及控制应多快实际提交。

原来的窗口或缺块指标可以成为这层的候选方案，但不会继续出现在 CL 的运行状态、AUTH 的 Force-window 参数或 certifier 的接受谓词中。

各地区仍可自行监控并提交报告；**报告只有建议作用，必须经过有效 L1 控制才能影响跨地区分支。**

### 5.3 Certifier 验证什么

Certifier 验证：

1. 指令来自合法的、最终确认的 L1 控制历史。
2. 指令具有正确 chain、授权、唯一控制身份和可确定的生效位置。
3. 到期时的目标 \(\beta\) 是否匹配。
4. 截止、anchor、接班选择和后续 DATA 接受是否遵循共同规则。

Certifier **不要求 MSIG 再附一份缺块证明，也不验证 multisig 的健康判断是否达到某个内部阈值**。这些属于治理服务的策略与审计，不是新的 CL 分支。

### 5.4 新的恢复活性依赖

当前地区离线而 multisig 没有提交有效控制时，CL 不会自行切换 \(\beta\)。区域内部恢复和正常 handoff 规则仍可运行，但完全离线地区的强切恢复依赖 multisig。

原有 \(T_{\rm L1}\) 只从控制**实际提交**后计时，不能覆盖此前的检测、决策、集签和等待提交时间。

因此，若后续要证明“故障后一定恢复”或给出恢复延迟，需要 multisig 策略提供相应的响应条件。**诚实提交合法指令，不自动等于一定及时提交指令。** 本轮不把新的响应上界悄悄加进旧假设。

## 6. Ready 上界：与 Force 执行条件分开

### 6.1 保留准备服务假设

沿用本轮已允许增加的 Ready 上界，但现在可以直接用真实时间 \(T_{\rm ready}\) 表述：

> 对持续正常服务、被有效选中的地区，从公开任务的开启位置成为最终确认的真实时间 \(t_0\) 起，若该 handoff 在准备期间没有被关闭或取代，正确成员在 \(t_0+T_{\rm ready}\) 前完成准确 anchor、完整 DATA 和证明、匹配执行状态、authority 及所需公开上下文的准备，并可通过 Ready / Enter 检查。

起点不能等到下载完成或 authority 到达后才算。预算应覆盖观察开启位置、获取、验证和安装所需状态。

开启位置继续使用原有公共边界：立即分配的 Force 使用触发位置；regular 使用共同重放中开启 successor 任务的位置；确实没有候选时，使用后来首次有效选择的位置。已经提前 Ready 的 regular successor 自然满足准备要求。

服务前提应写清：

- 正确成员在线，区域通信和处理符合规定范围。
- 所需 Espresso 历史、DATA 和证明能取回。
- L1 authority 与控制上下文及时提供。
- EL 提供从准确 anchor 继续执行的状态接口。
- 输入规模、节点跟进程度和状态恢复方式在声明的服务范围内。

这不承诺一个离线多年、没有 checkpoint 的节点能够在固定时间内重放所有历史。

### 6.2 不再由 Ready 推导 Force 间隔

删除上一版计划的 \(H>R\)、启动宽限和 AUTO deadline。

Ready 上界不阻止 multisig 在 \(t_0+T_{\rm ready}\) 之前强切。如果发生这种干预，旧候选程序关闭，新程序按自己的固定 handoff 准备。

也不要求先产出 DATA 才能执行下一次 Force。Ready 是进入条件；Force 是独立的控制事件。

### 6.3 时间假设可以恢复为简单分工

现在分别使用：

| 时间 | 用途 |
|---|---|
| Espresso timestamp | ON / OFF / MSIG 的生效阈值，以及来源与边界引用 |
| 真实时间 \(T_{\rm ready}\) | 准备服务的耗时上界 |
| 原有区域时间参数 | 区域 BFT 的提交、恢复与 leader 轮转 |

不再为了证明 Force gap 而增加 Espresso 双边时钟误差假设。保留 v3.7 的单边 clock-lead 条件，用于原有提前公告要求：

\[
\theta_a>s+T_{\rm L1}+\epsilon_E+\eta,\qquad \eta>0,
\]

其中 \(s\) 是控制实际提交的真实时间。按原有服务假设，这保证指令在相应 Espresso 阈值首次最终确认之前已经对正确在线参与者可见。

这不把 L1 timestamp 当作 Force 截止时间，也不要求等待“Espresso 结果再回 L1 验证”才能交接。

v4 的假设结构保留原有 L1、发布、Espresso 和区域故障模型，并明确标出新增 Ready 服务条件。multisig 的检测和响应策略另行设计。

## 7. 唯一的 Force：有效 MSIG 指令

### 7.1 指令与生效

一个 MSIG 指令至少绑定：

\[
(\text{chain},\ \beta_{\rm target},\ \theta_a,\ \text{control ID}).
\]

指令需要满足原有授权、完整 L1 控制历史、最终确认和提前公告要求。L1 最终确认使它成为已知的待执行指令，**不是立即切换分支**。

在 Espresso 首次达到 \(\theta_a\) 的最终确认位置：

1. \(\beta_{\rm target}\) 等于进入该位置时的当前 \(\beta\)：执行 Force。
2. 不匹配：消费为 stale，不把它改成针对当前 \(\beta\)。

regular rotation 不改变 \(\beta\)，所以不会仅因发生 regular handoff 就取消已公告的 Force。

缺少适用控制历史时，验证器等待证据，不能假定没有指令并继续确认可能被截止的旧输出。

### 7.2 Force 的执行不检查产出或等待时长

不检查当前 session 是否 Ready、是否产出过 DATA、是否已经推进 \(G\)，也不检查距离上次 Force 是否超过某个最短间隔。

连续 Force 仍需要各自有效、目标匹配的指令。重复投递一条旧指令不能连续关闭多个版本。

每个 Espresso prelude 至多执行一次 Force。多条指令同位置到期时，沿用固定顺序和旧 \(\beta\) 匹配规则，不能在一个 prelude 内连锁处理新 \(\beta\)。

### 7.3 不改变 finality 权限

multisig 可以决定什么时候强切，但不能指定一个更早的 retained block。

Force 始终保留触发区块 payload 之前已经接受的全部 \(G\)。未确认后缀可以被丢弃；已经接受的数据不能被撤销。无论连续执行多少次 Force，这个边界都不变。

## 8. Protocol 5：处理控制并执行 Hard Force

建议写成下面的程序结构：

~~~text
处理一个已认证的 Espresso 区块 J：

  1. 取得完整的适用 L1 控制前缀和所需历史。
     缺证据则在当前重放位置等待。

  2. 按固定顺序执行到期的独立 ON / OFF。

  3. 按原有顺序检查到期 MSIG：
       若没有有效且目标 beta 匹配的指令：
           不发生跨地区 Force。
       若存在：
           对进入本位置时的 beta 执行一次 Force；
           按既定规则消费其他到期指令，不连锁执行新 beta。

  4. 若发生 Force：
       保存本区块 payload 之前的 G 作为固定 anchor；
       关闭整个旧 beta，beta 加一；
       确定目标 r，并用 NextLive 选择地区；
       若选中，固定 handoff 并开始准备；
       若没有候选，保留待分配边界；
       通知 Session Manager 关闭旧程序、准备新程序。

  5. 再处理 payload：
       按原有规则逐 DATA 推进 G；
       terminal DATA 完成当前 session 后，再处理 regular successor；
       不更新任何跨地区 Force deadline 或失败计数。
~~~

所需 authority 暂时缺失与已经证明候选集为空，必须分开。前者等待原位置的证据；后者保留未分配边界并继续观察后续状态。迟到的证据不改变原来的 Force 位置和 anchor。

没有候选时，不自动增加 \(\beta\)，也不启动新的计时器。后来的 ON 可以使边界获得候选；另一条有效 MSIG 也可以按原有规则取代这条待分配边界。

保持 v3.7 的限制：触发 Force 的同一区块不接受新 \(\beta\) 的 DATA；旧 \(\beta\) 的未确认后缀在截止后也不能继续推进 \(G\)。

不存在“缺块但没有 MSIG，于是走另一个自动分支”的后备规则。

## 9. View、公开责任和并发程序

### 9.1 保留四字段表示

\[
(\beta,r,g,t).
\]

| 字段 | 含义 |
|---|---|
| \(\beta\) | Force 版本；每次有效 Force 加一 |
| \(r\) | session 的 authority / 配额索引，不是物理地区编号 |
| \(g\) | 区域恢复 generation |
| \(t\) | 区域内部 leader turn |

建议本轮保留 v3.7 的目标索引规则：

\[
r'=
\begin{cases}
r,&\text{被关闭的公开责任 session 尚无 DATA 进入 }G,\\
r+1,&\text{该 session 已有 DATA 进入 }G.
\end{cases}
\]

Force 后的入口都是 \((\beta+1,r',0,0)\)。没有已分配 session 时，保留待分配边界已确定的目标索引。

这两行不是公平机制，也不是 Force 的许可条件。即使一块也没产出、也没接受，仍然可以强切。复用 \(r\) 允许继续使用不可变 authority；物理地区由 NextLive 单独选择。

### 9.2 删除计时，不删除必要的责任归属

仍需知道谁负责 \(G\) 后下一个 DATA，沿用 export-session 的含义。它用于：

- 验证连续 DATA 的来源和配额；
- 确定被关闭的 source context 与目标 \(r'\)；
- 解析 regular boundary 和 Force handoff；
- 向 multisig 提供已经验证的进展信息。

它不再带 Force window 或 deadline。accepted count 仍可用于配额和 \(r'\) 判定，不再用于公平 credit 或自动强切。

公开责任 session 不一定是跑得最快的本地程序。例如 \(G\) 仍在 A，B 已经根据 terminal receipt 启动 regular successor。保留这个区别，才能正确计算 Force anchor 和关闭范围。

### 9.3 并发准备保持原设计

Session Manager 继续同时处理：

1. 当前 Active 的区域程序；
2. 为 regular successor 或未来 Force 准备的 Waiting 程序；
3. 已完成或已关闭程序的数据与证明保留工作。

未来 MSIG 公告允许提前获取数据和状态，但最终 retained anchor 仍要等实际生效位置才能固定。准备好的旧 anchor 不能直接冒充更新后的 Ready 状态。

Force 截止由共同重放决定。旧程序再按原有 retirement 规则退场；新程序在准确 handoff 上 Ready 后即可进入，不必等待旧程序的一切后台清理完成。

## 10. 五个具体例子

以下时间数字只表示合法生效阈值的示意，不是 Espresso 性能测量。所有 MSIG 示例均以满足 L1 提前公告等条件为前提。

### 例 1：A 完全离线，由 multisig 明确关闭 A 并强切

初始：

- 当前版本为 7，公开责任 session 为 A 的 \((7,20)\)。
- A 本次 session 没有 DATA 进入 \(G\)，当前 anchor 为 DATA 108。
- B 有资格、正常服务，目标 authority 已满足选择条件。

过程：

1. multisig 观察到 A 交付停止，按自己的策略决定干预。
2. 它提交独立 OFF(A) 和目标为 \(\beta=7\) 的 MSIG，安排在同一合法 Espresso 阈值 130 生效。
3. 在指令生效前，CL 不因缺块自行切换；截止前仍按原有规则接受合法 DATA。
4. 首次达到 130 的区块先应用 OFF(A)，再执行匹配的 MSIG。
5. 若这期间 \(G\) 仍为 108，就保留 108；\(\beta:7\to8\)，目标复用 \(r=20\)。
6. NextLive 选择 B。B 从固定 handoff 进入 \((8,20,0,0)\)，在 Ready 后从 108 后面继续执行。

**r 仍为 20，但物理地区已从 A 换成 B。** A 可以无限期 OFF。

若截止前确实有新 DATA 被接受，则必须保留更新后的 \(G\)，并按实际 accepted count 重新计算 \(r'\)，不能沿用 multisig 最初观察时的 108。

### 例 2：缺块条件成立，但 multisig 没有提交

- A 已经很久没有交付，任何监控者都可能认为应该强切。
- L1 中没有相应的有效 MSIG。

结果：**CL 不执行跨地区 Force，\(\beta\) 不变。**

这就是检测与执行分离后的行为。后续治理设计要解决“应当干预时，委员会是否会及时提交”，不能再依靠已经删除的 AUTO 补救。

### 例 3：连续 Hard Force，允许 B 尚未 Ready

- 第一条有效 MSIG 关闭版本 9，选择 B，进入版本 10。
- 在 B 尚未完成准备时，另一条针对版本 10 的有效 MSIG 生效。
- 立即关闭版本 10，进入版本 11；不要求 B 先 Ready 或先产出 DATA。

没有 Force gap 检查。每条指令仍需独立满足公告和目标匹配要求。

一条针对版本 9 的旧指令不能自动改成针对版本 10；重复投递它也不能完成第二次强切。

### 例 4：regular 与 Force 竞争

- 当前版本为 \(\beta=7\)，A 正在 \((7,20)\)。
- L1 已公告目标为 7、在 Espresso timestamp 300 生效的 MSIG。
- 在 295，B 根据 A 的合法 terminal receipt 和完整 entry 条件，正常进入 \((7,21)\)。无需等 terminal DATA 全局确认。
- 截止前，旧版本的合法 DATA 仍可以按顺序进入 \(G\)。
- 首次达到 300 的区块 prelude 执行 MSIG，关闭整个 \(\beta=7\)，包括 B 的 regular successor。
- 新版本从此位置前最新的 \(G\) 接班。它可能已经包含 B 的部分 DATA，也可能仍停在 A。

~~~mermaid
sequenceDiagram
    participant M as Multisig
    participant L as L1
    participant A as A：旧 session
    participant B as B：regular successor
    participant E as Espresso 与共同重放
    participant N as Force 后的新 session
    M->>L: 提交目标 beta 7、未来阈值 300 的 MSIG
    L-->>E: 最终确认且已公告的指令
    A->>B: terminal receipt + 数据
    B->>B: 完成 Ready / Enter (7,21)
    B->>E: Publish Layer 提交完整区域结果
    E->>E: 截止前逐 DATA 推进 G
    E->>E: 首次达到 300，先执行匹配 MSIG
    E-->>A: 关闭 beta 7
    E-->>B: 关闭 beta 7
    E-->>N: beta 8，固定 anchor = 此位置之前的 G
    N->>N: 准备准确 anchor；Ready 后启动
~~~

若在 300 之前另一条 Force 已把版本改成 8，针对 7 的指令 stale。仅发生 regular rotation 不会使它 stale。

若旧版本 DATA 恰好位于首次达到 300 的区块 payload 中，不能倒过来取消已经执行的 Force。

### 例 5：只 OFF、不 Force，以及全部 OFF

- OFF(A) 生效，但没有 MSIG：A 不再有未来选择资格，当前已固定 handoff 和 snapshot 不被撤销，也不自动改变 \(\beta\)。
- 随后 MSIG 生效，且 B 有资格：按 NextLive 接班。
- 若此时所有地区都 OFF：保留 \(G\) 和待分配边界，等待明确的 ON。
- 后来 ON(B) 生效、证据完整：在首次有效选择位置固定 B，准备并进入新 session。

整个过程没有自动恢复资格或循环空转的 Force。若委员会始终不恢复任何候选，协议允许保持等待。

## 11. 保留哪些保证，哪些留给治理设计

### 11.1 保留并重新检查的安全与接口性质

1. **全局前缀安全。** MSIG 不能指定更早的 anchor，也不能撤销 \(G\)。
2. **截止一致。** 相同完整 L1 / Espresso 历史导出相同的资格、Force、选择和接受结果。
3. **无指令不强切。** 缺块或健康报告不会独自改变 \(\beta\)。
4. **旧版本隔离。** 旧 \(\beta\) 的锁、票、Quit 和程序不能成为新 session 的有效状态。
5. **一次入场。** 本地 Ready / Enter 保持原子性与持久化，不能重复启动。
6. **准确 Ready。** 状态必须对应最终固定的 anchor；提前预取不授予提前进入权。
7. **条件式准备界。** handoff 持续有效、地区满足服务条件时，在 \(T_{\rm ready}\) 内具备进入条件。
8. **有效指令后的交接。** 有完整证据和 eligible 候选时，执行唯一确定的 Force handoff；没有候选时保留待分配边界。

删除旧 AUTO 正确性、超时间隔、失败窗口、Owed 服务和自动资格恢复等命题。

### 11.2 不再声称

- 当前地区故障后，CL 无需 multisig 就能自行恢复。
- 每个 ON 地区最终都得到服务，或 OFF 地区一定恢复。
- 每次 Force 后一定先有新 DATA，再发生下一次 Force。
- 只凭 Ready 上界就能证明 \(G\) 永远增长。
- 无限次、任意时机的 Hard Force 下仍能 Ready 或产块。
- 交易 x 在未确认后缀被丢弃后，CL 必须负责重新提交。

### 11.3 活性证明需要怎样写

可以保留条件式结论：存在 eligible 且满足区域服务条件的地区，控制与数据服务正常，并且它获得足够完成共识与交付的连续执行机会时，系统可以继续推进 \(G\)。

若要覆盖“当前地区已经离线”的恢复，还需后续 multisig 策略保证：

- 会识别需要干预的情况并实际提交有效控制；
- 必要时排除故障候选或恢复可服务候选；
- 不永久阻断所有地区的服务机会。

这些属于**控制服务活性前提**，不是 CL 的 Force 验证条件。本轮不把它们做成 credit、lease、自动恢复或强制 gap。

multisig 现在承担更多恢复活性责任，但仍没有回滚全局最终确认的权力。Ready 完成、区域 FINAL 形成、Espresso 收录、DATA 被 \(G\) 接受仍须分别说明。

整个地区离线应写成失去服务能力的情形；不能一边声称它满足及时区域服务条件，一边用完全离线作为该条件下的反例。旧证书的恶意签名权重突破 \(F_i\) 是另一类问题，不由本次控制简化解决。

## 12. 按原文结构修改

保持 v3.7 的十个主要章节和六个 Protocol 编号。删除 AUTO 模块不要求重排区域协议编号。

| 章节 | v4 修改 |
|---|---|
| 1 Overview | 只保留显式 MSIG Force；说明治理判断与 CL 执行的分工；删除公平与请求包含目标 |
| 2 Model and assumptions | 保留原有假设结构；标明 Ready 服务界；删除窗口参数；说明 multisig 响应条件留待后续 |
| 3 Regional engine: definitions | 保留 view、DATA / DRAIN、固定 authority / 配额；删除跨地区公平和 AUTO 状态 |
| 4 Regional protocols | 保持 Protocol 1–2 的共识与恢复逻辑，保留区域 BFT 计时器 |
| 5 Regional and global finality | 保留 FINAL、完整结果验证、连续 \(G\) 接受和来源信息 |
| 6 Cross-region rotation | 将 AUTO detector 移入 multisig 策略；更新模块图；保留并发程序、Ready 和 regular 直接路径 |
| 7 The Force handler | 重写为独立 STATUS、唯一 MSIG 路径、NextLive 和保留 \(G\) |
| 8 The external observer | 删除 Owed / deadline / miss 更新；保留逐 DATA 接受；交易重投移交 EL |
| 9 Examples and guarantees | 使用本计划五例；删除 AUTO 与公平证明；保留安全和有条件活性 |
| 10 Implementing the Publish Layer | 保留 leader first、permissionless backup、费用代付和数据证明保留 |

### 12.1 六个 Protocol 的改动边界

| Protocol | 处理 |
|---|---|
| 1：区域 steady state | 保留 |
| 2：区域 recovery | 保留 |
| 3：Session Manager | 删除 AUTO 输入和公平依赖；Force 输入只来自已验证的 MSIG closure；保留 regional receipt 与并发任务事件 |
| 4：Ready → Active / bootstrap | 保留原子 Enter；“Choose the anchor”改为“Declare the fixed anchor” |
| 5：Force prelude | 删除自动分支、deadline 和 cooldown；只处理有效 L1 控制 |
| 6：外部接受规则 | 保留逐 DATA commit；删除公平和超时副作用；terminal 已接受后再解析 successor |

### 12.2 文件修改边界

后续创建：

- 英文正式文稿：sync_hotstuff_rr_part2_rotation_v4.tex。
- 中文同步说明：sync_hotstuff_rr_part2_rotation_v4_zh.md。
- 编译结果：sync_hotstuff_rr_part2_rotation_v4.pdf。
- 必要的 v4 control / governance-interface / liveness 片段。
- 更新后的模块图、生命周期图和 Force / regular 时间线。

| v3.7 使用的文件 | 后续处理 |
|---|---|
| [rotation_v3_6_control.tex](sections/rotation_v3_6_control.tex) | 新建 v4 control，只保留显式控制处理；删除 window / AUTO / Owed / cooldown |
| [rotation_v3_5_health.tex](sections/rotation_v3_5_health.tex) | 替换为短的 multisig 输入输出接口；不设计具体缺块阈值或治理策略 |
| [rotation_v3_7_liveness.tex](sections/rotation_v3_7_liveness.tex) | 重写结论范围；不沿用依赖 AUTO、\(M=1+\beta\) 和 Owed 的证明 |
| [rotation_v3_7_source_step.tex](sections/rotation_v3_7_source_step.tex) 及相关接受片段 | 保留来源处理与逐步 commit，清理公平、自动检测与计时副作用 |
| [rotation_v3_5_ready_bundle.tex](sections/rotation_v3_5_ready_bundle.tex) | 保留准确状态交接，补充与 Ready 服务界的关系 |
| [rotation_v3_4_retirement.tex](sections/rotation_v3_4_retirement.tex) | 保留区域引擎安全退场规则 |

不原地修改被旧版本引用的共享片段。仓库已有的 arch_v4.tex 是此前的架构文稿，与此次 **Part II v4** 编号不是同一轮产物，不覆盖它。

## 13. 后续实施顺序与验收

### 第一步：先固定最小状态

只保留：

- 当前 \(\beta\)、\(G\)、各地区资格；
- 公开责任 session、配额及必要 accepted count，或未分配边界；
- 已公告 L1 控制、适用来源证据与持久化重放位置；
- 各 session 的本地生命周期和持久化入场标志。

删除 Owed、credit、lease、AUTO cause、deadline、misses、Force gap、cooldown，以及“已产一块才允许 Force”的许可位。

### 第二步：先改模型和 Protocol 5

把 Force 明确定义为有效 MSIG 的执行结果。ON / OFF 是资格更新的唯一来源。PrepareOwed / NextOwed 替换为适用的 NextLive。

删除 AUTO 原因证明、窗口参数及处理分支，同时改掉对应的模型文字和定理。不能在某个“故障兜底”段落保留隐式自动强切。

### 第三步：同步 Protocol 3–4、6 和图

画清：

1. multisig 观察与决策，输出 L1 控制；
2. 共同重放执行控制并导出固定边界；
3. Session Manager 同时管理 Active 和 Waiting 程序；
4. Ready 在准确 anchor 上进入区域引擎。

数据接受不再重置任何 Force 时钟。公共责任 session 仍用于归属和连续性，不因删除监控计时而删除。

### 第四步：逐条检查执行轨迹

| 轨迹 | 应得到的结果 |
|---|---|
| A 长期缺块，没有 MSIG | 不自动 Force，不自动 OFF |
| A 零输出离线；OFF(A) 与有效 MSIG 生效；B eligible | 选择 B；复用 \(r\) 也能完成物理地区切换 |
| 只提交 Force，未 OFF 原地区 | 原地区仍有资格；不能假装它已被排除 |
| 只 OFF 当前地区 | 不自动关闭当前 \(\beta\)，不重写固定 snapshot |
| 全部 OFF | 保留 \(G\) 和待分配边界，不自动恢复 |
| 后来 ON 使候选出现 | 按首次有效选择位置固定接班人，无额外宽限计时 |
| 连续 MSIG，目标分别匹配 | 即使新地区未 Ready 或未产块，也能再次 Force |
| 多条 MSIG 同位置到期 | 固定顺序，至多一次有效 Force |
| regular 先发生，MSIG 后到期 | regular 不改 \(\beta\)，指令仍可关闭整个旧版本 |
| 另一 Force 已改变 \(\beta\) | 旧 MSIG stale，不能自动修改目标 |
| 独立 OFF 旁边的 MSIG stale | OFF 仍按自身规则生效 |
| 旧 DATA 位于触发区块 payload | 先执行 Force，旧后缀不能补进 \(G\) |
| 缺少控制历史、节点重启或证据迟到 | 等待或恢复重放，不猜测没有 Force |
| 健康报告只发给部分节点 | 不产生 CL 状态变化 |
| 未确认后缀中的交易 x 被丢弃 | 不产生 CL 重投义务，已接受 \(G\) 保留 |
| multisig 不提交，或持续打断准备 | 不宣称恢复与进展已经由 CL 保证 |

这些是后续验证项目，本计划尚未运行模型检查或证明新算法。

### 第五步：编译、核对与交付

生成 v4 后检查 LaTeX、图、交叉引用和中英文一致性，并核对：

- AUTO 只在旧版本差异或治理策略来源的说明中出现，不再是 CL 事件、状态、证据类型或执行分支。
- 没有残留上一版计划的 Force 参数 \(H,W\)、deadline、cooldown、CLEAR 或隐式自动 OFF；区域原有的同名高度或权重符号不受影响。
- 没有隐藏的 Force gap、Ready 或产块前置条件。
- 旧 \(\beta\) 匹配、控制在 payload 前执行、固定 anchor 和 \(G\) 不回滚规则仍完整。
- regular DATA / DRAIN、区域 BFT 计时器和并发 session 结构未被连带删除。
- Ready 上界与治理响应条件分别说明，不据此夸大恢复或产块保证。
- 不承诺每地区公平、OFF 必恢复或交易最终包含。
- v3.7 及其原有共享源文件未被改写。
