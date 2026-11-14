# Sequencer Rotation 协议修订记录（v5 迭代）

日期：2026-10-25 初稿；2026-10-26 更新三次。

**用途。** 记录 Part 0 迭代（[v0.1](part0_model_goals_zh.md) 至 [v0.8](part0_model_goals_zh_v0_8.md)）中对协议本身做出的修订，供后续各模块 Part 写作时直接引用。各模块接口的完整写法见 [Part 1 v0.3](part1_module_interfaces_zh_v0_3.md)（上一版 [v0.2](part1_module_interfaces_zh_v0_2.md)）。

**修订历史。**

- 2026-10-25：初稿，R1–R18。
- 2026-10-26：确认 R8、R10、R12、R16、R17；新增 R19（交易转发不放进协议）与 R20（命名）；全文改用 Deposit Portal 与 Batch Inbox 两个名字；新增 §2.8 Block Builder。
- 2026-10-26（第二次）：恢复逻辑改为“以 D 为准”，新增 R21–R28（§6），对应 Part 0 v0.7 与 Part 1 v0.2。R8–R11、R16 与 R12 的 STOP 部分被取代；R2、R3、R6、R12、R15 被修改。被取代的 §2、§3 原文保留作历史，并在开头注明。推导与审阅见[以 D 为准的恢复](d_anchored_recovery_zh.md)。
- 2026-10-26（第三次）：新增 R29（RESUME 锚点改为 $D_L[h_{\rm res}]$）与 R30（授权引用即原生 origin、R-FRESH 取 $D_{\rm drift}$）（§7），对应 Part 0 v0.8 与 Part 1 v0.3。R22 的锚点写法与 §2.1 的释放公式随之更新；R7、R13、R25 被修改；D12 已定。

**基线。** `formalization_v3` 的 Part II v4.9、Part III v6，组合式抽象 v2，以及 `megaeth_specs` 中的 mega-espresso 规格。凡本文未提到的部分，沿用基线。

**状态标记：**

- **已定：** 已在讨论中确认。
- **请确认：** 为落实已定决定而提出的具体规则，尚待确认。本版已没有这类条目。

---

## 1. 修订总表

| 编号 | 修订 | 取代的旧设计 | 落到哪个模块 | 状态 |
| --- | --- | --- | --- | --- |
| R1 | 分两层：DRS（L1 Authority、Session Program、Session Manager、Block Builder、Publisher、Global Finality Rule，以 Deposit Portal 为输入）与 L2 层（Batcher、Batch Inbox、Certifier、DA、Derivation Rule、状态结算）。DRS 与 L2 都是组合名，不是模块 | certifier 参与 authority 与模式切换 | 全局 | 已定 |
| R2 | 方案 A：不改派生规则；确认分三层；“全局最终”只可能被批次失败回退撤销 | 组合式抽象 §6.4 的 A/B 二选一；受保护前缀 | Replay、L2 | 已定；“全局最终”的部分被 R28 修改 |
| R3 | $\mathsf M$ 只影响活性；控制纪律由合约与 Replay 强制 | multisig 作为诚实协调服务 | Authority、Replay | 被 R28 修改（$\mathsf M$ 诚实） |
| R4 | Force 的公告提前量由合约检查，观察者遵守 R-WAIT | $\mathsf M$ 自觉满足 $\theta_a>\tau+T_{\rm L1}+\epsilon_E$ | Authority、Replay | 已定 |
| R5 | 合并顺序：Espresso 块与 L1 事件按 Espresso 时间键统一排序 | 待定的边界策略 $\mathfrak B$ | Replay | 已定 |
| R6 | R-EXP：普通块的过期规则 | 无 | Replay、共识层 | 已定；过期效果被 R24 修改 |
| R7 | R-FRESH：授权 L1 引用的新鲜度 | 无 | Replay、共识层 | 已定；授权引用与新鲜度界被 R30 修改 |
| R8 | R-DEP：过期高度按规范 deposit-only 规则补位 | DRS 自己的 DEPOSIT_ONLY 模式 | Replay | 被 R21 取代 |
| R9 | R-OVR：区域被越过后自动轮换 | 无（D11） | Replay、Manager | 被 R21 取代 |
| R10 | R-ANCHOR：Force、R-OVR、RESUME 共用的新锚点规则 | Force 从 $\mathrm{last}(G)$；RESUME 从 C 证书给出的末端 | Replay、Manager | 被 R21、R22 取代（Force 仍从 $\mathrm{last}(G)$） |
| R11 | R-FB：L1 决定优先；批次失败回退 | 无 | Replay | 被 R21、R22 取代 |
| R12 | STOP($h_{\rm cut}$) 与 RESUME，后者执行 Batch Inbox 的 cursor override | ScheduleMode、MODE 证书、ModeApplied、ApplySettlementBoundary | Authority、Batch Inbox | STOP 被 R23 删除；RESUME 被 R22 修改 |
| R13 | 质押按时间上界解锁（P2） | ProveClosure、Clear 表、CLOSURE 证书 | Authority | 已定；公式中的 $\delta_{\rm ref}$ 按 R30 换成 $D_{\rm drift}$ |
| R14 | Deposit Portal（强制包含入口，原称 force inbox）沿用原生合约；新增强制包含活性 L-fi | 无 | 原生规则 | 已定 |
| R15 | $\mathsf C$ 只核对批次与 $G$ 的对应及 DA 绑定（含 DEP 块的空批次） | BATCH、MODE、CLOSURE 三种证书 | Certifier、Batch Inbox | 已定；被 R27 修改（不再认证 DEP 空批次） |
| R16 | batcher 为 DEP 块提交空批次，保持 Batch Inbox 的 cursor 连续 | 无 | Batcher、Certifier | 被 R27 取代 |
| R17 | Batch Inbox 的 admin 改为 L1 Authority；override 只能经由 RESUME 执行 | 独立的 admin | Batch Inbox | 已定（D14） |
| R18 | 诚实 leader 的频率写成假设 A-LEAD | Part I 的 schedule fairness 假设 | 共识层 | 已定 |
| R19 | 交易转发不放进协议：Block Builder 只在本区域是当前或下一个负责区域时接受交易，否则拒绝并告知负责区域；L-tx 改为向负责 session 提交，负责 session 更换时由客户端重新提交 | Part 0 早期“提交给任一成员”的 L-tx | Block Builder、Session Manager | 已定 |
| R20 | 命名：Deposit Portal（强制包含入口）与 Batch Inbox（批次收件箱）；不再使用“force inbox”与单独的“inbox” | force inbox、L1 inbox | 全局 | 已定 |
| R21 | 以 D 为准的恢复：Replay 不再生成或预测 deposit-only 块；紧急情况由 RESUME 以派生链替换 $G$（§6.1） | R8–R11 | Replay、Manager | 已定 |
| R22 | RESUME($h_{\rm res}$)：$h_{\rm res}$ 由 $\mathsf M$ 给出；合约只检查授权与格式并 override；Replay 在 RESUME 处以 $D_L$ 替换 $G$（直到 $h_{\rm res}$），锚点为 $D_L[h_{\rm res}]$；Replay 在此处经派生规则读取 DA | R12 的 RESUME 部分、R10 的 RESUME 情形、R11 | Authority、Replay、Manager | 已定；锚点按 R29 更新 |
| R23 | 删除 STOP | R12 的 STOP 部分、D15 | Authority、Replay、Manager、共识层 | 已定 |
| R24 | R-EXP 的过期效果：全局链的下一个高度过期时关闭当前分支，Tail 进入 HALT，直到 RESUME；$W_{\rm exp}$ 约 10 h | R6 的过期效果、D13 | Replay | 已定 |
| R25 | 派生一致投票（DerivOK）：诚实成员只为在父块上一定被派生原样采用的块投票 | R19 中 Builder 的构块检查（扩展） | Block Builder、共识层 | 已定；drift 条件被 R30 修改；条件清单在 Part 6 逐条核对 |
| R26 | Session Manager 的本地 Kill：关闭已不可能进入 $G$ 与 $D$ 的 session | 无 | Manager | 已定；判据 (b) 的含义请确认 |
| R27 | batcher 只提交 $G$ 的普通块；$\mathsf C$ 只认证普通块，签名的 GFR 位置不早于 prev cursor 的写入位置 | R16；R15 的 DEP 部分 | Batcher、Certifier | 已定；$\mathsf C$ 的签名时机请确认 |
| R28 | 治理与确认语义：$\mathsf M$ 诚实；“全局最终”在紧急情况下可被 RESUME 撤销，批次结算是真正的最终 | R3；R2 的“全局最终”部分 | 全局 | 已定 |
| R29 | RESUME 的锚点为派生链在目标高度处的块 $D_L[h_{\rm res}]$；$G$ 被替换为 $D_L$ 直到 $h_{\rm res}$（§7） | R22 的锚点 $\mathrm{last}(D_L)$ | Replay、Manager | 已定 |
| R30 | 授权 L1 引用即块的原生 origin；R-FRESH 为 $\mathit{ts}(o(B))\ge t(h(B))-D_{\rm drift}$，删除常数 $\delta_{\rm ref}$；派生一致投票的 drift 条件对空块也不例外（§7） | R7 的授权引用与 $\delta_{\rm ref}$；D12 | Replay、共识层、Authority | 已定 |

---

## 2. 分模块修订

> **2026-10-26（第二次）注：** 本节中以下内容已被 §6 取代，原文保留作历史：
> - §2.1 的 STOP 记录与 RESUME 记录（RESUME 的新写法见 §6.1 的 R22）；
> - §2.3 中与 STOP 有关的生产约束；
> - §2.4 中的 DEP 锚点；
> - §2.5 中与 STOP 有关的发布限制；
> - §2.6 中的 R-DEP、R-OVR、R-ANCHOR、R-FB、STOP 与 `CursorSet` 的处理；
> - §2.7 中的 DEP 空批次与 DepFill 核对项。
>
> Force 记录、质押释放、R-WAIT、R-EXP 的接纳条件与 R-FRESH 仍然有效。
>
> **2026-10-26（第三次）注：** R-FRESH 的授权引用与新鲜度界按 R30 修改（§7）：§2.6 中的 $\mathit{ts}(\lambda(B))\ge t(h(B))-\delta_{\rm ref}$ 现为 $\mathit{ts}(o(B))\ge t(h(B))-D_{\rm drift}$。§2.1 的释放公式已直接改写。

### 2.1 L1 Authority（质押层）

**Force 记录** $U=(c,\theta_a,\beta_{\rm target},\mathcal P_{\rm new})$。合约检查：

1. $\theta_a\ge\mathit{ts}(b)+M_a$，其中 $b$ 是纳入该记录的 L1 块（R4）；
2. $\mathcal P_{\rm new}$ 给每个区域恰好一个 ON/OFF 标记，且每个区域在循环顺序中恰好出现一次；
3. 序号 $c$ 严格递增；
4. 当前没有处于生效状态的 STOP。

**STOP 记录** $(c,h_{\rm cut})$。合约检查：

1. 当前没有处于生效状态的 STOP；
2. $t(h_{\rm cut})\ge\mathit{ts}(b_{\rm stop})+M_a+2\Delta_{\rm ts}+D_{\rm drift}$，即截断高度足够靠后（理由见 §3.6）。

记录后，Authority 进入 STOPPED 状态。

**RESUME 记录** $(c,\mathcal P_{\rm new})$。合约检查：

1. Authority 处于 STOPPED 状态；
2. 满足下列二者之一：Batch Inbox 的 cursor 已不低于 $h_{\rm cut}$；或者 $\mathit{ts}(b_R)\ge t(h_{\rm cut})+W_{\rm seq}+\delta_{\rm slot}$（所有不高于 $h_{\rm cut}$ 的高度都已过了截止）；
3. $\mathcal P_{\rm new}$ 格式正确。

合约随后计算 $N:=\max\{h: t(h)\le\mathit{ts}(b_R)-\delta_c\}$。若 $N$ 高于当前 cursor，就以 admin 身份对 Batch Inbox 执行 cursor override 到 $N$（R17），最后清除 STOPPED 状态。

**接口名。** Part 1 中，Force、STOP、RESUME 三类记录对应的接口分别为 `SubmitForce`（原 SubmitUpdate）、`SubmitStop`、`SubmitResume`（二者取代 ScheduleMode）。

**质押释放（R13）。** position $o$ 在 epoch $e_x$ 被移出快照，该快照于 L1 高度 $a_{e_x}$ 激活。只有在时间戳满足下式的 L1 块中，才允许释放 $o$：

$$
\mathit{ts}\ \ge\ \mathit{ts}(a_{e_x})+D_{\rm drift}+2\mathrm{TB}\cdot b_t+W_{\rm exp}+2\Delta_{\rm ts}+T_{\rm ev}.
$$

推导见 Part 0 v0.4 的 P2。删除 ProveClosure、Closures 与 Clear 表。
公式中的第一项原为 $\delta_{\rm ref}$；按 R30，授权引用即原生 origin，新鲜度界取 $D_{\rm drift}$。

**不变的部分。** Join、Cancel、RequestExit、CutSnapshot、Plan、Auth、Leader、SubmitOffense、SubmitQuit 沿用 Part III v6。唯一的变化是 Plan 所用的选区参考块须满足 R-FRESH。

### 2.2 Deposit Portal（强制包含入口）

沿用原生的 deposit 与强制包含机制，不作修改（R14）。
DRS 的强制包含活性 L-fi 来自原生规则 N2：L1 origin 必须随 L2 时间推进，所以强制包含交易必然进入链中。L2 层的 L2-L2 来自 sequencing window 回退。

### 2.3 共识层（Session Program）

- **不生产注定无效的块。** 不提议以下三类块：STOP 生效后高度超过 $h_{\rm cut}$ 的块；按 R-EXP 已经来不及进入 $G$ 的块；授权引用不满足 R-FRESH 的块。
- **追赶模式。** 新锚点（R-ANCHOR）可能在过去：时间戳约为当前时间减去 $M_a+\delta_c$。新 session 须以快于 $b_t$ 的速度出块，直到追上实时。
- **快速确认的语义。** 快速确认可以因 Force、STOP、过期或批次失败回退而失效（Part 0 S4）。

### 2.4 Session Manager

- **新的关闭原因。** 除 Force 外，STOP、RESUME 与 R-OVR 也会关闭 session。
- **新的 anchor 来源。** anchor 可以是 DEP 块。它的“承诺证据”不是区域 FINAL，而是：规范 deposit-only 规则，加上所用的 L1 数据。
- **DEP 锚点的状态准备。** 在前一个块的状态上，依次执行 DEP 块，就得到锚点状态。
- **负责性视图（R19）。** 新增 `Responsibility()`，向 Block Builder 与客户端提供当前与下一个负责区域。

### 2.5 发布层

不变。STOP 生效后，不再发布高度超过 $h_{\rm cut}$ 的普通块。

### 2.6 全局最终化规则 $\mathsf{Replay}$

**事件与键。** $\mathsf{Replay}$ 按键从小到大处理三类事件。键相同时：L1 事件在前、时间事件其次、Espresso 块最后；同类之间按 L1 位置或高度排序。

| 事件 | 键 |
| --- | --- |
| Espresso 块 $J_q$ | $\theta_q$ |
| L1 事件 $e$（Force、STOP、RESUME 记录；高度 $h$ 的截止 $\mathrm{ddl}(h)$；Batch Inbox 的 `CursorSet`） | $\mathit{ts}(b_e)+M_a$；Force 的生效键为 $\theta_a$ |
| 时间事件：高度 $h$ 过期 | $t(h)+W_{\rm exp}$ |

**R-WAIT（处理条件）。** 处理键为 $x$ 的事件之前，须满足：

- 观察者的 L1 日志已有时间戳 $\ge x-M_a$ 的块；
- 对 L1 事件与时间事件，Espresso 日志还须已有 $\theta\ge x$ 的块。

第二条保证：所有 $\theta<x$ 的 Espresso 块都已确定，因为 $\theta$ 沿 Espresso 日志非减。
所以停机期间，L1 事件与时间事件都会等待，$G$ 暂停。

**处理 Espresso 块 $J_q$。**

1. 先执行此前已到期的控制。
2. 再按 v4.9 的 Eligible 谓词接纳普通块，并另加三条：
   - **R-EXP：** $\theta_q\le t(h)+W_{\rm exp}$；
   - **R-FRESH：** $\mathit{ts}(\lambda(B))\ge t(h(B))-\delta_{\rm ref}$；
   - **R-STOP：** 若 STOP 已生效，则 $h\le h_{\rm cut}$。

**处理时间事件“高度 $h$ 过期”（R-DEP）。** 若 $h=h(\mathrm{last}(G))+1$，且 $G$ 尚无高度 $h$，则追加 DEP 块：

$$
\mathsf{DepFill}(\mathrm{last}(G),t(h)).
$$

这条**规范 deposit-only 规则**定义为：

- 没有普通交易；
- 设父块的 origin 为 $e$。若 L1 块 $e+1$ 的时间戳 $\le t(h)$，则 origin 取 $e+1$，否则保留 $e$；
- deposit、L1 attributes 与系统交易按原生规则确定。

R-WAIT 保证此时 L1 日志已覆盖 $t(h)+W_{\rm exp}-M_a\ge t(h)+\delta_c$，所以 origin 可以确定。

**R-OVR。** 若 R-DEP 追加的高度落在当前负责 session $s$ 的区间内，则：

1. 关闭 $s$，Tail 记为 OVERRUN，来源区域为 $i_s$；
2. 在下一个 Espresso 块 $J$（第一个 $\theta\ge$ 当前键的块）处，按 R-ANCHOR 开出新 session；
3. 新 session 的区域按 policy 的循环顺序取 $i_s$ 之后第一个合格区域。

STOP 生效期间不执行 R-OVR。

**R-ANCHOR（统一的新锚点规则）。** 在 Espresso 块 $J$ 处需要开出新 session 时：

$$
h_A:=\max\bigl(h(\mathrm{last}(G)),\ h_{\rm det}(\theta_J)\bigr),\qquad
h_{\rm det}(\theta):=\max\{h: t(h)\le\theta-M_a-\delta_c\}.
$$

1. 先对 $(h(\mathrm{last}(G)),h_A]$ 中的每个高度，按 $\mathsf{DepFill}$ 依次追加 DEP 块；
2. 新 session 以高度 $h_A$ 的块为 anchor，并且 $E=A$；
3. 第一个普通块的高度为 $h_A+1$。

三种使用情形：

- **Force：** 在生效块 $J_q$ 处，关闭旧分支，以 $\theta_J=\theta_q$ 开出新分支。正常情形下 $G$ 是最新的，$h_A=h(\mathrm{last}(G))$，与 D4 相同。
- **R-OVR：** 见上文。
- **RESUME：** 在第一个 $\theta\ge\kappa_R$ 的 Espresso 块处，以 RESUME 的 policy 开出新分支 $\beta+1$。

**处理 L1 事件。**

- **STOP：** 键 $\mathit{ts}(b_{\rm stop})+M_a$。记录 $h_{\rm cut}$，此后 R-STOP 生效；超过 $h_{\rm cut}$ 的高度只能由 R-DEP 补位。
- **RESUME：** 键 $\kappa_R=\mathit{ts}(b_R)+M_a$。解除 STOP，待下一个 Espresso 块时按 R-ANCHOR 开出新分支。
- **`CursorSet`（与 RESUME 在同一交易）：** 该事件让 L1 派生把 (safe head, $N$] 填成 deposit-only。由 §3.6，这些高度在 $G$ 中都是或都将是 DEP 块，内容相同，无需额外操作。
- **截止 $\mathrm{ddl}(h)$（R-FB）：** 若 $G$ 在 $h$ 处是普通块，而 L1 在截止前没有接纳其批次，就发生**批次失败回退**：$G$ 从 $h$ 起改为 L1 派生链的对应块。其中仍以原内容结算的块保留出处 $\mathsf{ORD}(s)$，被 deposit-only 替换的块出处改为 DEP。否则无操作。

**多个控制。** 按键排序，键相同按 L1 位置排序，依次处理。

**确定性。** 合并顺序、各条子规则与 $\mathsf{DepFill}$ 都是 L1 日志与 Espresso 日志的确定函数，重算记录对日志前缀单调。这正是 Part 0 S1 与 S2 的来源。

### 2.7 L2 层

- **batcher：**
  - 正常提交普通块的批次；
  - 对 DEP 块提交空批次（R16），以保持 Batch Inbox 的 cursor 连续；被 RESUME 的 override 跳过的高度除外；
  - 空批次的 origin 必须与 $\mathsf{DepFill}$ 的选择一致。
- **$\mathsf C$：** 只核对两件事：批次与 $G$ 的对应（普通块的内容，或 DEP 块的空内容），以及 DA 绑定。删除 MODE 与 CLOSURE 证书（R15）。
- **Batch Inbox：**
  - admin 为 L1 Authority，override 只能经由 RESUME 执行（R17）；
  - 其余沿用 mega-espresso 的 batch inbox。
- **派生：** 不改（方案 A）。须在 Derivation Part 中逐条核对一点：对 $t(h)$ 不晚于 override 所在 L1 块时间戳的高度，$\mathsf{DepFill}$ 与 OP Stack 的窗口过期空批次规则、mega-espresso 的 override 填充规则给出同一个块。

### 2.8 Block Builder

- **只在负责时接受交易（R19）。** `SubmitTx` 只在本区域是当前负责区域、或已知的下一个负责区域时接受交易；否则返回 `REJECT(NOT_RESPONSIBLE, i_resp)`，告知当前负责的区域。谁负责由 Replay 的 Tail 与常规后继的 plan 公开可算，经 Session Manager 的 `Responsibility()` 提供。
- **不转发。** 协议不在区域之间转发交易。负责区域更换时，尚未被包含的交易由客户端或 RPC 网关重新提交。抗审查由 Deposit Portal 的强制包含保证（L-fi），与转发无关。
- **构块。** 候选块必须包含原生规则要求的 deposit；`Build` 与 `ValidateCandidate` 沿用组合式抽象。
- **相应的 Part 0 修改。** L-tx 改为向负责 session 提交；A-RELAY 去掉“转发用户交易”。

---

## 3. 恢复逻辑详解

> **2026-10-26（第二次）注：** 本节描述的是 v0.6 的恢复逻辑（R-DEP、R-OVR、R-ANCHOR、STOP 与 RESUME 的 N），已被 §6 的“以 D 为准的恢复”取代，原文保留作历史。§3.7 的参数约束中仍然有效的条目，已重新列在 §6.4。

### 3.1 总览：两类故障，三条恢复路径

| 故障 | 恢复路径 | 是否需要 $\mathsf M$ |
| --- | --- | --- |
| 区域停摆（Espresso 正常） | 高度过期后由 R-DEP 补位，R-OVR 轮换到下一区域；或 $\mathsf M$ 发出 Force | 不需要；Force 只用于加速或更改 policy |
| Espresso 停机 | 恢复后由 R-DEP、R-OVR、R-ANCHOR 自动追上 | 不需要 |
| Espresso 长时间停机 | STOP 提交截断高度，RESUME 经 override 让 L1 立即填到新锚点 | 用于减少 L1 侧的空批次开销 |

**核心思想。** 一个高度如果已经不可能再有普通块——过期了，或者被关闭的 session 负责——它的内容就**完全由 L1 决定**：按规范 deposit-only 规则，由 L1 数据与父块确定。
因此 $G$ 可以在 L1 真正填充之前就把它算出来，而 L1 之后无论通过窗口过期、空批次还是 override 填充，得到的都是同一个块。
这就是新锚点“计入 deposit-only 高度”的方式：锚点之前的空档全部是 DEP 块，与 L1 的 deposit-only 块逐块相同，锚点的状态就是 L1 派生链在该高度的状态。

### 3.2 Espresso 停机：自动流水线

**阶段 0：停机开始（$\tau_d$）。**
Espresso 不再最终化新块。没有人能立即区分“停机”与“延迟”，协议也不需要区分。

**阶段 1：停机期间。**

- **DRS：** 按 R-WAIT，L1 事件与时间事件都要等待 Espresso 前进，所以 $G$ 暂停。区域 session 即使继续出块也无法发布。
- **L2：** batcher 继续提交 $G$ 中已有、但尚未批次化的普通块（mega-espresso 的 S9 行为）。对 $G$ 中没有内容的高度，窗口过期后由 L1 自动填 deposit-only。强制包含照常有效（L2-L2）。

**阶段 2：Espresso 恢复。** 第一个新块 $J_r$ 出现后，观察者按合并顺序处理积压的事件：

1. **停机前提出、恢复后才最终化的旧块。** 它们的 $\theta$ 早于恢复时刻，所以排在最前面，其中的普通块照常接纳（受 R-EXP 约束）。若某个这样的块的截止已过而批次未被接纳，随后的截止事件会触发批次失败回退。这与 mega-espresso S9 的结论一致：迟到的记录不能恢复已过截止的高度。
2. **过期事件。** 时间戳满足 $t(h)+W_{\rm exp}<\theta_r$ 的高度依次过期。第一个落在负责 session 区间内的过期高度触发 R-OVR。
3. **新锚点。** R-OVR 在下一个 Espresso 块（即 $J_r$）处，按 R-ANCHOR 计算 $h_A=h_{\rm det}(\theta_r)$，也就是时间戳约为 $\theta_r-M_a-\delta_c$ 的高度。$h_A$ 之前的空档全部以 DEP 块补齐，新 session 从 $h_A+1$ 开始追赶。

**阶段 3：L1 侧跟上。**

- 窗口已过期的高度，L1 早已填好 deposit-only，与 $G$ 的 DEP 块相同。
- 尚未过期的 DEP 高度，由 batcher 提交空批次（R16），使 cursor 前进。
- 之后新 session 的普通块正常批次化。

**停机不足 $W_{\rm exp}$ 的情形。** 没有高度过期。原 session 恢复后直接追赶停机期间的高度，不丢失任何内容，也不产生 DEP 块。

### 3.3 Espresso 恢复后的新锚点

$$
h_A=\max\bigl(h(\mathrm{last}(G)),\ h_{\rm det}(\theta_J)\bigr),\qquad h_{\rm det}(\theta)=\max\{h: t(h)\le\theta-M_a-\delta_c\}.
$$

其中 $J$ 是开出新 session 的那个 Espresso 块：自动路径下是恢复后的第一个块，治理路径下是 RESUME 生效的块。

**为什么锚点选在这里。**

- **上界：** $t(h_A)\le\theta_J-M_a-\delta_c$。R-WAIT 保证处理 $J$ 时，L1 日志已覆盖到 $\theta_J-M_a$，所以到 $h_A$ 为止每个 DEP 块的 origin 与 deposit 都已确定。
- **下界：** 锚点不能更早。更早的话，新 session 的第一个高度会立刻因 R-EXP 过期：它需要大约 $T_{\rm switch}$ 来规划、准备、出块与发布，而 $W_{\rm exp}\ge M_a+\delta_c+T_{\rm switch}$ 正好留出这段时间。
- **不会回退：** 取 $\max$ 保证锚点不低于 $\mathrm{last}(G)$，绝不改写已有的 $G$。

**为什么它计入了 deposit-only 填充的高度。**

- 空档 $(h(\mathrm{last}(G)),h_A]$ 中的每个高度都没有、也不可能再有普通块：旧 session 已关闭，新 session 从 $h_A+1$ 开始。
- 由 A-CERT，$\mathsf C$ 只核对与 $G$ 一致的批次，所以这些高度在 L1 上只会是 deposit-only。
- 无论 L1 通过哪种途径填充（窗口过期、空批次、override），所用规则都与 $\mathsf{DepFill}$ 相同，因此逐块一致。
- 锚点状态等于 L1 派生链在 $h_A$ 处的状态。新 session 在它之上出块，不会与 L1 冲突。

### 3.4 治理加速：STOP 与 RESUME

自动路径在长停机后有一个代价：cursor 必须逐个高度地越过空档。两天的停机，按 $b_t=1\,\mathrm s$ 计约 17 万个高度，需要大量空批次。
$\mathsf M$ 的 STOP 与 RESUME 用一次 override 解决这个问题。

1. **STOP($h_{\rm cut}$)。** $\mathsf M$ 在发现停机后提交截断高度。合约要求 $t(h_{\rm cut})\ge\mathit{ts}(b_{\rm stop})+M_a+2\Delta_{\rm ts}+D_{\rm drift}$。生效后，$G$ 不再接纳高于 $h_{\rm cut}$ 的普通块，超过 $h_{\rm cut}$ 的高度只能是 DEP。这为 L1 提供了一个它能读到的上界：$G$ 的普通内容不会超过 $h_{\rm cut}$。
2. **等待。** 截断以下的高度须先由 L1 定下来：要么 cursor 已越过 $h_{\rm cut}$，要么这些高度都已过截止。
3. **RESUME。** Espresso 恢复后，$\mathsf M$ 提交 RESUME 与新 policy。合约计算 $N$ 并执行 override，L1 派生立即把 (safe head, $N$] 填成 deposit-only。
4. **Replay 生效。** 在 $\kappa_R=\mathit{ts}(b_R)+M_a$ 之后的第一个 Espresso 块 $J$ 处，$G$ 按 R-ANCHOR 补齐 DEP 块。此时 $h_A=h_{\rm det}(\theta_J)$，通常就是 $N$。随后以新 policy 开出分支 $\beta+1$，新 session 从 $h_A+1$ 出块，它的批次紧接 override 之后被接纳。

若 Espresso 在 RESUME 之后才恢复，$J$ 更晚，$h_A$ 会高于 $N$。多出的 DEP 高度用空批次补上即可。

### 3.5 数字例子

参数：$b_t=1\,\mathrm s$，$W_{\rm seq}=12\,\mathrm h$，$W_{\rm exp}=2\,\mathrm h$，$M_a=15\,\mathrm{min}$，$\delta_c=1\,\mathrm{min}$，$D_{\rm drift}=30\,\mathrm{min}$，每个 session 10 分钟（600 个高度）。
下面用时间戳称呼块，例如“块 @10:00”。

**例 1：短停机（30 分钟，无治理）。**

- Espresso 在 day0 10:00 停机，10:30 恢复。$G$ 停在块 @09:59:58。负责 session $s$（区域 A）覆盖 @09:55–@10:05。
- 恢复后，$s$ 继续发布 @09:59:59 至 @10:05 的块。这些块满足 $\theta\approx10{:}30\le t+2\,\mathrm h$，照常接纳。
- 后继 session（区域 B）覆盖 @10:05–@10:15，同样追赶。
- 没有 DEP 块，没有内容丢失；批次的截止在 $t+12\,\mathrm h$，绰绰有余。

**例 2：长停机（两天，无治理）。**

- Espresso 在 day0 10:00 停机，day2 10:00 恢复。
- **停机期间，L1 侧：** 从 day0 22:00 起，@10:00 之后的高度因窗口过期依次被填成 deposit-only；到 day2 10:00，已填到 @day1 22:00。
- **恢复时，第 1 步：** 设有一个 day0 09:59:59 提出的旧块，含 $s$ 的 @10:00，恢复后才最终化。它先被接纳；但 @10:00 的截止已过、批次未被接纳，随后的截止事件触发批次失败回退，@10:00 改为 DEP。
- **恢复时，第 2 步：** @10:00:01 的过期键（day0 12:00:01）触发 R-OVR，关闭 $s$。
- **恢复时，第 3 步：** 在第一个新块 $J_r$（$\theta_r=$ day2 10:00）处执行 R-ANCHOR：$h_A=$ @day2 09:44。$G$ 补齐 day0 @10:00:01 至 day2 @09:44 的 DEP 块。
- **新 session：** 区域 B 的新 session 从 @09:44:01 开始追赶，落后约 16 分钟加上切换时间。
- **L1 侧：** @day1 22:00 之前的高度早已是 deposit-only，与 $G$ 相同。cursor 停在 @09:59:58，要越过两天的高度，需要 batcher 提交大量空批次。

**例 3：长停机（两天，有治理）。**

- **STOP：** $\mathsf M$ 在 day0 10:20 发现停机，提交 STOP，取 $h_{\rm cut}=$ @11:10。它满足下界：10:20 + 15 分钟 + $2\Delta_{\rm ts}$ + 30 分钟 ≈ 11:06。
- **RESUME：** Espresso 在 day2 10:00 恢复。$\mathsf M$ 在 L1 时间 day2 10:01 提交 RESUME。合约检查 $10{:}01\ge t(h_{\rm cut})+12\,\mathrm h=$ day0 23:10，通过；计算 $N=$ @day2 10:00。
- **L1 侧：** override 把 cursor 移到 $N$，派生立即把 (safe head ≈ @day1 22:01, @day2 10:00] 填成 deposit-only。
- **Replay：** 在 $\kappa_R=$ 10:16 之后的第一个 Espresso 块处，$G$ 按 R-ANCHOR 补齐到 $h_A=$ @10:00，与 $N$ 相同。
- **新分支：** 新分支采用 RESUME 的 policy，第一个 session 从 @10:00:01 追赶；它的批次紧接 override 被接纳，不需要任何空批次。

### 3.6 为什么 $\mathsf M$ 只影响活性

- **Force。** 合约保证 $\theta_a\ge\mathit{ts}(b)+M_a$，R-WAIT 保证任何观察者在处理 $\theta\ge\theta_a$ 的块之前都已看到它，所以 Force 不会追溯生效。Force 只关闭 session，并按 R-ANCHOR 开出新分支，从不截断 $G$。
- **STOP。** 它只限制键 $\ge\mathit{ts}(b_{\rm stop})+M_a$ 之后接纳的普通块；同理不会追溯生效。
- **RESUME 的 override。** override 只影响高于 $h_{\rm cut}$、且不高于 $N$ 的高度：不高于 $h_{\rm cut}$ 的高度，合约已要求 L1 先把它们定下来。高于 $h_{\rm cut}$ 的高度在 $G$ 中不可能有普通块，原因有两条：
  - STOP 生效之后，R-STOP 直接禁止；
  - STOP 生效之前接纳的普通块 $B$，其所在 Espresso 块满足 $\theta<\mathit{ts}(b_{\rm stop})+M_a$。由 N2 与 A-TS，$t(h(B))<\mathit{ts}(b_{\rm stop})+M_a+2\Delta_{\rm ts}+D_{\rm drift}\le t(h_{\rm cut})$。

  所以 override 填的都是 DEP 高度，与 $G$ 逐块相同。

**这一论证依赖的假设。**

- 第二条用到 A-TS。若 A-TS 失效，被腐化的 $\mathsf M$ 有可能让 override 跳过一个尚未批次化的普通块。后果是批次失败回退：S1、S2 仍成立，只是撤销了该块。
- R-ANCHOR 的预测依赖 A-CERT：$\mathsf C$ 只核对与 $G$ 一致的批次。

**恶意 $\mathsf M$ 能做的事。** 反复 Force；在 Espresso 正常时 STOP 并且不再 RESUME。后者会让 $G$ 只能以 DEP 块增长（落后约 $W_{\rm exp}$），用户交易无法进入 $G$。这些都只是活性损失。

### 3.7 参数约束

| 约束 | 作用 |
| --- | --- |
| $M_a\ge T_{\rm L1}+2\Delta_{\rm ts}+\delta_{\rm slot}$ | R-WAIT 在正常情况下无需等待 |
| $\delta_c\ge\delta_{\rm slot}$ | DEP 块的 origin 可以确定 |
| $W_{\rm exp}\ge M_a+\delta_c+T_{\rm switch}$ | R-ANCHOR 开出的新 session 来得及出第一块 |
| $W_{\rm exp}+M_a+T_{\rm batch}\le W_{\rm seq}-D_{\rm drift}$ | 进入 $G$ 的块（含 DEP 空批次）在交付预算内能赶在截止前被接纳 |
| $t(h_{\rm cut})\ge\mathit{ts}(b_{\rm stop})+M_a+2\Delta_{\rm ts}+D_{\rm drift}$ | override 不会跳过普通块（§3.6） |
| RESUME：cursor $\ge h_{\rm cut}$，或 $\mathit{ts}(b_R)\ge t(h_{\rm cut})+W_{\rm seq}+\delta_{\rm slot}$ | 截断以下的高度先由 L1 定下来 |
| $T_{\rm slash}\le T_{\rm ev}$ | 证据在释放前处理完毕 |

**$W_{\rm exp}$ 的取舍（D13）。**

- $W_{\rm exp}$ 越大，越长的停机能由原 session 无损追赶；
- 但质押解锁也越慢（R13 的公式含 $W_{\rm exp}$）；
- 而且一旦需要越过故障区域，$G$ 的 DEP 补位会落后得更多。

### 3.8 边界情况

1. **迟到的旧块。** 停机前提出、恢复后才最终化的块，按 $\theta$ 排在最前处理；截止已过的，会触发批次失败回退。
2. **在 Espresso 正常时恶意 STOP。** 高于 $h_{\rm cut}$ 的高度只能由 R-DEP 补位，$G$ 落后约 $W_{\rm exp}$ 增长，用户交易暂停，直到 RESUME。是否给 STOP 设一个自动失效期限，见 D15。
3. **Espresso 仍停机时就 RESUME。** L1 立即填到 $N$。$G$ 等 Espresso 恢复后才处理 RESUME；那时 $h_A$ 可能高于 $N$，多出的 DEP 高度用空批次补上。
4. **DA 不可用。** 派生会在已接纳但取不回的批次处等待；override 也会先等前面已接纳的输入处理完。$G$ 不受影响，除非等待超过截止，触发批次失败回退。
5. **$\mathsf C$ 离线。** 批次与空批次都提交不了，高度在截止后被 deposit-only 填充：DEP 高度内容相同，无影响；普通块高度触发批次失败回退。
6. **所有区域都停摆。** R-OVR 依次轮换各区域，每轮都失败；$G$ 仍以 DEP 块增长（落后约 $W_{\rm exp}$），强制包含照常有效。
7. **STOP 期间的 Force。** 合约拒绝。policy 的更改随 RESUME 一起提交。

---

## 4. 删除或替换的旧设计

| 旧设计 | 出处 | 替代 |
| --- | --- | --- |
| DRS 自己的 DEPOSIT_ONLY 模式与“转移 D” | 组合式抽象 v2 §5.10 | R-DEP、R-ANCHOR |
| 共同边界策略 $\mathfrak B$、$\mathsf{Frames}_{\mathfrak B}$、$K_S$/$K_R$、ModeApplied、MODE 证书 | v2 §5.4、§5.9、§5.11 | 合并顺序、STOP、RESUME |
| ScheduleMode | v1 §4.2.12 | STOP、RESUME |
| ProveClosure、Closures、Clear 表、CLOSURE 证书 | v1 §4.2.13、§4.7.2 | R13 的时间上界 |
| ApplySettlementBoundary | v1 §4.6.3 | RESUME 的 override |
| 方案 B（受保护前缀） | v1 §6.4 | 方案 A 与 R-FB |
| “每个 Espresso 块至多一个更新激活”的假设 | Part II v4.9 | Replay 按序处理多个控制 |
| multisig 作为诚实协调服务 | Part II v4.9 | A-GOV（只用于活性） |
| schedule fairness 假设 | Part I | A-LEAD |
| 独立的 Batch Inbox admin 与任意 override | mega-espresso batch-inbox | R17 |
| “提交给任一成员”的 L-tx 与跨区域交易转发 | Part 0 v0.3–v0.5 | R19 |

---

## 5. 待定问题

> **2026-10-26（第二次）注：** 本节的状态已由 §6.5 更新，原文保留作历史。

- **D12：** 授权 L1 引用 $\lambda(B)$ 是否与原生 origin 统一。若统一，R-FRESH 可取 $\delta_{\rm ref}=D_{\rm drift}$，且对 DEP 块自动成立。
- **D13：** $W_{\rm exp}$ 的具体取值（§3.7）。
- **手动 override 的场景（Part 6）：** R17 去掉了 mega-espresso 中 admin 的手动 override。需要确认它原先处理的场景（publisher anchor 超前、S5、S6、S8）在轮换设计下是否仍会出现，以及没有手动 override 时如何处理。
- **Certifier 与 cursor（Part 6）：** 轮换版本的 Certifier 如何认证没有 Espresso 记录的 DEP 高度；cursor 中 GFR 位置字段的编码。
- **批次失败回退之后的续接（Part 5）：** R-FB 与 R-DEP 的优先级，以及被回退影响的 session 如何结束。
- **D15：** STOP 是否设自动失效期限，以免沉默的 $\mathsf M$ 长期冻结用户交易。
- **核对项：** 在 Derivation Part 中逐条核对 $\mathsf{DepFill}$ 与 OP Stack 空批次生成、mega-espresso override 填充规则的一致性（§2.7）。
- **已确认（2026-10-26）：** R8、R10、R12、R16、R17。

---

## 6. 以 D 为准的恢复（R21–R28，2026-10-26 第二次更新）

**一句话。** Replay 不再预测 deposit-only 块。L1 派生链 D 按原生规则自己补位；已不可能进入 D 的 session 由各 Session Manager 在本地关闭；紧急情况下，诚实的 $\mathsf M$ 发出 RESUME($h_{\rm res}$)，把 D 一次补到未来高度 $h_{\rm res}$，Replay 把 $G$ 替换为 $D_L$ 直到 $h_{\rm res}$，并以 $D_L[h_{\rm res}]$ 为锚点开出新分支（锚点按 R29）。

对应文档：最初为 [Part 0 v0.7](part0_model_goals_zh_v0_7.md)、[Part 1 v0.2](part1_module_interfaces_zh_v0_2.md)；含 R29、R30 的现行版本为 [Part 0 v0.8](part0_model_goals_zh_v0_8.md)、[Part 1 v0.3](part1_module_interfaces_zh_v0_3.md)。推导、例子与审阅见[以 D 为准的恢复](d_anchored_recovery_zh.md)。该文 §9 中的守卫、R-DIV、R-LATE、恢复纪元与无许可 RESUME 是当时提出的备选，本次没有采用：$\mathsf M$ 诚实，Kill 留在本地，$\mathsf C$ 的签名时机规则代替恢复纪元。

### 6.1 修订条目

- **R21（总则）。** 删除 R-DEP、R-OVR、R-ANCHOR、R-FB。$G$ 中的 deposit-only 块只来自 RESUME 时采纳的派生链，出处记为 $\mathsf{DER}$。
- **R22（RESUME）。**
  - **L1 Authority：** `SubmitResume(c, P_new, h, Σ_M)` 检查签名、序号、policy 格式，以及 $h$ 高于当前 cursor；同一交易内 override 到 $h$。合约不计算 $h$，也不判断是否处于紧急状态。
  - **派生：** 先处理此前已接纳的输入，再把 (派生末端, $h$] 立即填成 deposit-only，origin 不超过 RESUME 所在的 L1 块 $L$。这沿用 mega-espresso 的 override 派生规则。
  - **Replay：** 在键 $\mathit{ts}(L)+M_a$ 处计算 $D_L$，这里要读 DA；关闭当前分支；令 $G$ 替换为 $D_L$ 直到 $h_{\rm res}$，与原 $G$ 同一原生块的块保留原出处，其余为 $\mathsf{DER}$；令 $\beta:=\beta+1$；以 $A=D_L[h_{\rm res}]$ 为锚点（R29），在下一个 Espresso 块处开出新 session。数据不足时等待，等待不改变结果。
  - **$\mathsf M$ 的操作规程**（A-GOV，合约不检查）：
    - 派生链已越过或偏离全局链之后才发出，即 cursor 已超窗停滞；
    - Espresso 已恢复，批次层故障时 batcher 与 $\mathsf C$ 也已恢复；
    - $t(h)\ge\mathit{ts}(L)+T_{\rm prep}$，且 $t(h)-\mathit{ts}(L)\le D_{\rm drift}$ 减去余量；
    - $h$ 高于旧全局链的末端。
- **R23（删除 STOP）。** 删除 STOP 记录、`Stopped` 状态、R-STOP、限高与 `StopEffective`；Force 的合约检查中去掉“当前没有生效的 STOP”一条。
- **R24（过期停住）。** R-EXP 的接纳条件不变。处理 $\mathsf{Expire}(h)$ 时，若 $h$ 是全局链的下一个高度且仍没有块：关闭当前分支上的全部 session，包括提前进入的常规后继；Tail 进入 HALT，直到 RESUME。$W_{\rm exp}$ 取约 10 h。
- **R25（派生一致投票）。** 诚实成员只为满足 DerivOK 的块投票，每一条都对 L1 视图的扩展稳健：
  - origin 为父块的 origin 或其下一块，已最终化，且时间戳不晚于本块；
  - 不超过 drift，空块也不例外（按 R30；原写法允许空块在推进 origin、或已看到下一个 origin 晚于本块时超过）；
  - deposit 前缀与派生构造的完全相同；
  - 执行有效；
  - 满足 R-FRESH。
- **R26（Kill）。** Session Manager 在本地关闭 session $s$，当且仅当：派生链已在某个高度上定下一个与 $s$ 的链不相容的块（分叉或越过）；或全局链已不可能再接 $s$ 的块。实现上可以用“cursor 超窗停滞”只读 L1 来判断。Kill 不改变 $G$。
- **R27（L2 层）。**
  - batcher 只提交 $G$ 的普通块，不再提交 DEP 空批次。
  - $\mathsf C$ 只认证普通块，签名所用的 GFR 位置不早于 prev cursor 的写入位置。因此，RESUME 之前签发的证书在 override 之后失效，RESUME 之后签发的只覆盖新的 $G$。
- **R28（语义）。**
  - A-GOV 改为 $\mathsf M$ 诚实：S1–S6 不依赖 $\mathsf M$；紧急恢复，以及“全局最终的块在非紧急情况下进入批次结算”，依赖 $\mathsf M$。
  - “全局最终”的名称保留，但它在紧急情况下可被 RESUME 撤销；批次结算才是真正不可撤销的最终。
  - DG1 增加 DA-S1。

### 6.2 恢复流程

| 故障 | 处理 | 需要 $\mathsf M$ |
| --- | --- | --- |
| 区域停摆（Espresso 正常） | $W_{\rm exp}$ 内 Force，从 $\mathrm{last}(G)$ 接棒、追赶缺口；超过则过期停住，等 D 越过全局链后 RESUME | 是 |
| Espresso 短停机（短于 $W_{\rm exp}$） | 原 session 恢复后接着出块，无损 | 否 |
| Espresso 长停机 | 过期停住；窗口结束后各 Manager Kill 旧 session，停机期间即可完成；Espresso 恢复后 RESUME | 是 |
| 批次层故障，窗口内恢复 | 从 cursor 补交，无损 | 否 |
| 批次层故障，超过窗口 | D 分叉；各 Manager Kill 旧 session，全局链停止增长；批次层恢复后 RESUME，分叉点之后的全局链内容被撤销 | 是 |

模块级的步骤见 Part 1 v0.2 §6，带时间的例子见[以 D 为准的恢复](d_anchored_recovery_zh.md) §8。该例中的“分叉事件”这一版没有采用，由各 Manager 的 Kill 承担。

### 6.3 正确性要点

1. **目标 T：** 非紧急情况下，$G$ 中的块以同一个块进入 D。前提是：R25、A-BYZ、§6.4 的接纳时限约束、批次层在交付预算内工作，以及 $\mathsf M$ 不在非紧急情况下 RESUME。证明对高度归纳（Part 1 v0.2 §8.4）。
2. **RESUME 结果唯一：** $D_L$ 是 $L$ 及之前的已最终化 L1 前缀与 DA 数据的确定函数；锚点在 RESUME 的处理位置固定，等待只推迟结果，不改变结果。
3. **不撤销已结算内容：** override 保留派生链已派生的前缀，所以 RESUME 只会撤销没能进入 D 的全局链内容。
4. **Kill 不误杀：** 派生链已派生的前缀不再改变（L1 最终性、派生确定性、DA 绑定），所以被 Kill 的 session 不可能再进入 D。

### 6.4 参数约束

| 约束 | 作用 |
| --- | --- |
| $M_a\ge T_{\rm L1}+2\Delta_{\rm ts}+\delta_{\rm slot}$ | R-WAIT 在正常情况下无需等待 |
| $W_{\rm exp}+M_a+T_{\rm batch}+D_{\rm drift}\le W_{\rm seq}$ | 进入 $G$ 的块在交付预算内能赶在截止前被接纳（目标 T）；取 $W_{\rm exp}\approx 10$ h |
| $T_{\rm slash}\le T_{\rm ev}$ | 证据在释放前处理完毕 |
| $M_a+T_{\rm ready}\le T_{\rm prep}\le D_{\rm drift}-\text{余量}$ | $\mathsf M$ 选取 $h_{\rm res}$ 的操作参数，合约不检查 |

质押释放公式（§2.1，R13）的第一项按 R30 换成 $D_{\rm drift}$：

$$
\mathit{ts}\ \ge\ \mathit{ts}(a_{e_x})+D_{\rm drift}+2\mathrm{TB}\cdot b_t+W_{\rm exp}+2\Delta_{\rm ts}+T_{\rm ev}.
$$

其中的 $W_{\rm exp}$ 取新值，解锁相应延后约 8 小时。
已删除的约束：$\delta_c$；STOP 的截断下界；RESUME 的两条合约条件（cursor 不低于 $h_{\rm cut}$，或截断以下已过截止）。

### 6.5 待定问题（更新 §5）

- **D12：** 已定，见 §7 的 R30。
- **D13：** 已定，约 10 h。
- **D15：** 不再适用（STOP 已删除）。
- **批次失败回退之后的续接：** 由 RESUME 统一处理，已解决。
- **Certifier 与 cursor（Part 6）：** DEP 高度的认证问题已不存在；仍需定义 GFR 位置字段的编码，以及 RESUME 写入的完整 cursor。
- **核对项（Part 6）：** 不再需要 DepFill 的一致性核对；改为核对 DerivOK 与派生检查逐条一致（G-DER-6），以及 cursor 超窗停滞判断的精确边界。
- **手动 override 的场景（Part 6）：** 仍待讨论。
- **请确认：** Kill 判据 (b) 的含义；$\mathsf C$ 的签名时机规则。
- **硬边界：** RESUME 处已接纳批次的数据取不回时，派生与 Replay 都等待。

---

## 7. RESUME 锚点与授权引用（R29–R30，2026-10-26 第三次更新）

对应文档：[Part 0 v0.8](part0_model_goals_zh_v0_8.md)、[Part 1 v0.3](part1_module_interfaces_zh_v0_3.md)。

### 7.1 修订条目

- **R29（RESUME 锚点）。** 修改 R22。
  - RESUME 的锚点从 $\mathrm{last}(D_L)$ 改为 $D_L[h_{\rm res}]$，即派生链在 RESUME 目标高度处的块；Replay 把 $G$ 替换为 $D_L$ 直到 $h_{\rm res}$。
  - $\mathsf M$ 按操作规程选取 $h_{\rm res}$ 时，两种写法给出同一个块：$h_{\rm res}$ 高于 $L$ 处派生链的末端，override 正好把派生链补到 $h_{\rm res}$，Batch Inbox 的 cursor 也停在 $h_{\rm res}$。
  - 若违反规程、$h_{\rm res}$ 不高于派生末端，派生链已经越过 $D_L[h_{\rm res}]$，新 session 会被 Kill 判死，需要再发一次 RESUME；安全不受影响。
  - 新分支的锚点规则因此只有两种来源：Force 以生效时的 $\mathrm{last}(G)$ 为锚点，RESUME 以 $D_L[h_{\rm res}]$ 为锚点。
- **R30（授权引用即 origin）。** 修改 R7、R13、R25；D12 已定。
  - 授权 L1 引用 $\lambda(B)$ 就是块的原生 L1 origin $o(B)$ 的高度；区域块的字段 $\rho$ 必须等于它，DER 块同样取 origin。Plan 用 $\lambda(E)$ 选快照时，读的就是锚点的 origin。
  - R-FRESH 改为 $\mathit{ts}(o(B))\ge t(h(B))-D_{\rm drift}$：新鲜度界取 $D_{\rm drift}$，常数 $\delta_{\rm ref}$ 删除。
  - 派生一致投票的 drift 条件对空块也不例外，与 R-FRESH 合为同一个不等式。原生规则允许空块超过 drift 的两种例外不再使用：GFR 的 R-FRESH 本来就会拒绝这样的块，而其中一种例外还取决于投票者是否已看到下一个 origin。
  - 质押释放公式（§2.1）的第一项从 $\delta_{\rm ref}$ 换成 $D_{\rm drift}$。

### 7.2 待定问题（更新 §6.5）

- **D12：** 已定（R30）。
- **Kill 判据 (b)：** 已定。$G$ 领先 $D$ 时，session 必须从 $\mathrm{last}(G)$ 派生；这与 R29 之后的锚点规则一致（Part 0 v0.8 定义 3.5）。
- **$\mathsf C$ 的签名时机规则：** 仍待确认。
- 其余条目同 §6.5。
