# MegaETH Sequencer Rotation 技术报告 · Part 4

## Session Manager 协议（中文工作稿 v0.1）

日期：2026-10-26。
状态：草稿。

本稿实现 [Part 1 v0.3](part1_module_interfaces_zh_v0_3.md) §4.3 的 Session Manager（MGR）：每个成员节点上的协调者。先复述接口与保证，再给出状态、事件循环、六组处理过程（跟踪、区域结果、交接、准备与进入、程序事件、生命周期）、Kill 的精确判据，以及各条保证的证明。

前置文档：

- [Part 0 v0.8](part0_model_goals_zh_v0_8.md)、[Part 1 v0.3](part1_module_interfaces_zh_v0_3.md)；
- [Part 2 v0.1](part2_l1_authority_zh_v0_1.md)（L1 Authority）、[Part 3 v0.1](part3_session_program_zh_v0_1.md)（Session Program）、[Part 5 v0.1](part5_global_finality_rule_zh_v0_1.md)（Global Finality Rule）；
- Part II v4.9 §8（Session Manager、handoff、ReadyBundle）与 §11（节点存储）。

**本稿对 Part 1 v0.3 的补充**（实现时发现，待确认）

1. **派生视图在每个 `CursorSet` 处保存检查点** $\mathrm{Ckpt}[c]=(D_L,\mathit{st}(D_L[h_{\rm res}]))$，供 GFR 的 `ResumeChain` 与 RESUME 锚点的状态准备共用（§4.2）。
2. **派生视图须给每个块标注来源**：批次、窗口回退填充，或 override 填充。Kill 的“越过”判据要用它（§10.2）；这是对 Derivation Rule 输出的要求，列入 Part 6。
3. **Kill 的“过期”情形交给 GFR。** Part 1 §4.3.8 的 G 判死含“下一个高度在 $G$ 中已过期”。本地无法在 Espresso 停机时可靠地判断它（停机前提出、时间戳更早的 Espresso 块可能晚些才最终化），而 Espresso 可用时 GFR 的 `Expire` 已经关闭整个分支。所以 Kill 只实现“$G$ 接纳了别的块”这一种 G 判死（§10.3）。
4. **锚点状态的固定副本。** 每个 `SessionAssigned` 的锚点在基线状态继续推进之前固定一份状态副本，保证准备的是精确锚点（§7.2）。

---

## 0. 定位

**负责什么。**

- 运行 GFR 与派生规则，跟踪全局链 $G$、派生链 $D$ 与生产分支 $R$；
- 按 handoff 创建、进入、关闭本节点的 Session Program；
- 准备精确锚点的可执行状态，发出 ReadyBundle；
- 把 Program 的输出变成持久的交付任务：发布、证据提交、交接、快速确认；
- 本地关闭已不可能进入 $G$ 与 $D$ 的 session（Kill）。

**不负责什么。** 它不做自由选择：区域、锚点、$G$ 与关闭位置全部来自 GFR、派生规则与 L1 Authority。它不签任何共识消息：签名都在 Program 中。

---

## 1. 复述：接口与保证

### 1.1 接口

| 接口 | 调用者 | 作用 |
| --- | --- | --- |
| `ObservePublic(reads)` | 节点的数据源 | 验证并保存来源；推进派生视图与 GFR；应用生命周期事件；运行 Kill |
| `ObserveRegional(Res, handoffEvidence)` | 本区域或其他区域的成员 | 验证区域结果，扩展 $R$，交给 Publisher；末块时产生常规后继的 handoff |
| `InstallHandoff(handoff)` | 本模块 | 验证 handoff，检查能否进入，创建 Program |
| `PrepareAndEnter(s, stateMaterial)` | 本模块 | 准备精确锚点的状态，原子地复查后进入 |
| `HandleProgramEvent(s, event)` | 本节点的 Program | 把 Conf、Result、Complete、Quit、Offense 变成持久任务 |
| `ApplyLifecycle(event)` | 本模块 | 按 GFR 的生命周期事件更新 Program 表、$R$ 与负责性视图 |
| `Responsibility()` | Block Builder、RPC | 当前与下一个负责的区域 |
| `Kill(s)` | 本模块 | 本地关闭已不可能进入 $G$ 与 $D$ 的 session |

### 1.2 保证

| 保证 | 内容（简述） | 依赖 |
| --- | --- | --- |
| G-MGR-1 handoff 真实 | 只为通过 VerifyHandoffHistory 的 handoff 创建 Program；同一 pair 的固定字段不被替换 | G-PROG-2、G-GFR-1、6 |
| G-MGR-2 单次进入，不复活 | 每个 pair 至多一次初次 Enter；已终止或已关闭的 pair 不再进入 | 持久化存储 |
| G-MGR-3 及时关闭 | 处理关闭事件后、任何后续 BFT 动作前，关闭标记已持久化、Program 已关闭 | — |
| G-MGR-4 无自由裁量 | 区域、锚点与 $G$ 来自 GFR、派生规则与 L1 Authority；Kill 只依据 $D$ 与 $\mathcal X$ | G-GFR-1、G-DER-1 |
| G-MGR-5 持久与交付 | 结果、证据与任务在 Program 终止或节点崩溃后保留；每份证据都会提交 | 持久化存储、A-RELAY |
| G-MGR-6 准备活性 | 分配之后 $T_{\rm ready}$ 内达到 Ready | $T_{\rm ready}$ |
| G-MGR-7 Kill 不误杀、及时 | Killed 的 session 不可能再进入 $D$；$D$ 越过或偏离之后至多一个 L1 块内关闭 | L1-S、DA-S1；G-DER-1 |
| G-MGR-8 负责性视图 | 与 GFR 的 Tail 及常规后继的 plan 一致 | G-GFR-1 |

---

## 2. 状态

| 状态 | 内容 | 持久化 |
| --- | --- | --- |
| PublicStore | 已验证的 L1 块头、日志与 receipt；Espresso 块头与本链 namespace 的对象；已取回的 DA payload；检查点 | 是 |
| $\mathcal X$ | GFR 的重算状态（Part 5 §3），连同处理位置 | 是，每处理一个事件 |
| 派生视图 $\mathcal D$ | 派生状态；已派生的链 $D$，每个块带来源标签；`CursorSet` 检查点 $\mathrm{Ckpt}[c]$ | 是 |
| 生产分支 $R$ | 当前分支上已验证的区域结果，按 session 分组，经 handoff 相连；可以领先于 $G$ | 是 |
| `Programs[s]` | 固定的 handoff；阶段（Assigned、Running、Terminated）；初次进入记录；终止原因 | 是 |
| `Closed` | 已关闭的分支版本与 session，以及 Kill 写入的标记；只增不减 | 是 |
| 已准备的状态 | 基线 $\mathit{st}(\mathrm{last}(G))$；常规后继的后继状态；锚点副本 `AnchorState[s]`；RESUME 锚点的状态 | 可重建 |
| `Outbox` | 待发送任务：发布、证据提交、REC 等待、交接消息、ReadyBundle | 是 |
| `RespView` | 负责性视图（§11） | 可重建 |

**记号。** 对 session $s$，$\mathrm{chain}(s)$ 是 $A_s$ 及其全部祖先，加上本节点所知 $s$ 已提交的块（Part 1 §4.3.8）。$\mathrm{top}(s)$ 是 $\mathrm{chain}(s)$ 中的最高高度。

---

## 3. 事件循环与持久化

Manager 一次处理一个事件。事件包括：公共输入到达、区域结果或 ReadyBundle 到达、Program 输出事件、本地计时器（任务重试）。

每个事件的处理分两段：

1. **决定段：** 只读状态，算出新状态与要执行的副作用，把它们一起写入预写日志并持久化。
2. **执行段：** 执行副作用：调用 Program（Create、Enter、Close、Provide）、发送网络消息、提交 L1 交易、交给 Publisher。

**崩溃恢复。** 重启时重放预写日志，恢复状态；对执行段未确认完成的副作用，按幂等方式重做：Program 接口与 L1 Authority 接口都去重（Part 1 §0.4），网络消息允许重复。

**与 Program 的交错。** Program 的计时器与消息处理也作为本节点事件循环中的事件。所以 Manager 在一个事件中“持久化关闭标记，再调用 `PROG.Close`”之后，同一 Program 不可能在中间插入任何 BFT 动作〔G-MGR-3〕。

---

## 4. 跟踪公共输入：`ObservePublic(reads)`

### 4.1 算法

```text
ObservePublic(reads):
  # 1. 来源
  验证 reads：L1 块头与最终性、日志与 receipt 的包含证明；Espresso 块头与最终性、
  namespace 完整性证明；DA payload 与其承诺 d 的绑定。写入 PublicStore。

  # 2. 派生视图（只读已最终化的 L1 与已取回的 DA）
  loop:
    r := DER.Derive(𝒟, PublicStore.L1_final, PublicStore.DA)
    if r == WAIT(missing, pos): 安排取回 missing；break
    𝒟 := r.state；把新派生的块连同来源标签（BATCH、WINDOW_FILL、OVERRIDE_FILL）追加到 D
    对 r 中处理过的每个 CursorSet（属于 RESUME 记录 c，位于 L1 块 L）：
        Ckpt[c] := (D_L := 处理完该 CursorSet 时的 D,  st(D_L[h_res]))     # §4.2

  # 3. GFR
  loop:
    r := GFR.AdvanceGlobal(𝒳, PublicStore.L1_final, PublicStore.Esp_final, ResumeIface)
    if r == WAIT(next key, missing): break
    持久化 (𝒳 := r.state, r.events)
    for ev in r.events: ApplyLifecycle(ev)                                   # §9

  # 4. Kill
  KillScan()                                                                 # §10
```

**接口 `ResumeIface`。** GFR 处理 RESUME 记录 $c$ 时调用 `ResumeChain`（Part 5 §5.4），它需要 $D_L$。`ResumeIface.Get(c)` 返回 $\mathrm{Ckpt}[c]$ 中的 $D_L$；派生视图尚未处理到该 `CursorSet` 时返回 `WAIT`，GFR 随之停在这条记录上。

### 4.2 为什么在 `CursorSet` 处保存检查点

$D_L$ 是“按 L1 顺序处理到 RESUME 的 `CursorSet` 为止”的派生链（Part 0 定义 4.2）。派生视图继续推进之后，$D$ 会越过这一点，所以必须在处理 `CursorSet` 的那一刻留下快照。
派生是确定的（G-DER-1），检查点只是缓存：丢失时可以从更早的派生检查点重算。
同一个检查点同时提供 RESUME 锚点的状态 $\mathit{st}(D_L[h_{\rm res}])$（§7.2）。

### 4.3 处理顺序的理由

- **先派生，后 GFR。** GFR 在键 $\mathit{ts}(L)+M_a$ 处理 RESUME，需要派生视图已经处理到 $L$ 中的 `CursorSet`。派生只读已最终化的 L1，而 R-WAIT 保证此时 L1 已推进到 $L$ 之后，所以只要 DA 数据可取回（A-RET），派生视图一定先到达这一点。
- **GFR 之后才 Kill。** 生命周期事件可能已经关闭一些 session；Kill 只需检查剩下的。
- **不按到达顺序做决定。** 每一步都是其输入的确定函数；消息到达的先后只影响何时得到结果，不影响结果〔G-MGR-4〕。

---

## 5. 区域结果与生产分支

### 5.1 `ObserveRegional(Res, handoffEvidence)`

```text
ObserveRegional(Res = (C', B, D, Σ, π), handoffEvidence):
  s := sid(B)
  # 1. 自己推出 context，不用发送方声称的 C'
  if Programs[s] 存在: C := Programs[s].handoff 的 context
  elif s 的 handoff 由 GFR 的 SessionAssigned 给出: C := 对应 descriptor 的 context
  else:
      r := VerifyHandoffHistory(handoffEvidence)                    # §6.2
      if r == WAIT: return WAIT;  if r == REJECT: return REJECT
      C := r.context
  # 2. 区域最终检查
  switch CheckFINAL(C, B, D, Σ, π):
      incomplete → return WAIT；false → return REJECT
  # 3. 已关闭版本只存档
  if Closed[β(s)] 或 Closed[s]: 存档 Res；return Stored(s, h(B), H(B))
  # 4. 扩展生产分支并发布
  把 π 中从 B 到 D 的块并入 R[s]（按高度，与 R 中已有块核对哈希）
  Outbox.add(Offer(Res))                                             # 交给 Publisher
  # 5. 末块：常规后继
  if h(B) == H_s 或 h(D) == H_s:
      RegularSuccessor(s, T_s, Res)                                  # §5.2
  return Stored(s, h(B), H(B))
```

$R$ 中的块都带有直接或覆盖的 FINAL 证据。同一高度出现两个不同的块，只可能来自不同 session（不同分支版本），按分支版本分开保存。

### 5.2 常规后继

```text
RegularSuccessor(s, T_s, Res):
  ν_s := Programs[s].handoff.ν（或 GFR 给出的 descriptor）
  Π' := Plan(P_β, β, r+1, A_s, i_s; 本节点的 L1 视图)               # Part 2 §7.2
  if Π' == WAIT: Outbox.add(Retry(RegularSuccessor, s)); return
  q' := max{ height(ξ(T_s)), q^open_s + 1 }
  if region(Π') == ⊥: return                                          # GFR 将输出 PENDING
  ν' := (β, r+1, region(Π'), T_s, h(T_s) + TB, q', REGULAR, Π')
  handoff' := (ν', P_β, E = A_s, i_src = i_s,
               λ^open = max{λ(T_s), λ(Π')}, Q^0 = T_s 的 FINAL 证据（取自 Res）,
               evidence = (handoff_s, Res))
  if region(Π') == 本区域: InstallHandoff(handoff')                    # 不必等 T_s 进入 G
  Outbox.add(SendResult(region(Π'), Res, handoff_s))                  # 无论是否选中本区域
```

后继的 plan 以前驱的初始锚点 $A_s$ 为选区参考块，所以在前驱开出时就能算出；这里只是在末块结果到达时把它和 $q'$ 组成 handoff。
Plan 只读已最终化的 L1 数据（快照是 cutoff 之前历史的确定函数，不需要任何交易），所以 Manager 与 GFR 读到的是同一份数据，算出同一个 plan（Part 2 §7.2）。提前安装的 handoff 因此与 GFR 之后输出的 `SessionAssigned` 一致（§9）。

---

## 6. 交接

### 6.1 handoff 的构造

handoff 的格式见 Part 1 §2.7：$(\nu,\mathcal P_\beta,E,i_{\rm src},\lambda^{\rm open},Q^0,\mathit{evidence})$，其中 $\lambda^{\rm open}=\max\{\lambda(A),\lambda(\Pi)\}$。

| cause | 来源 | $E$ | $Q^0$ | evidence |
| --- | --- | --- | --- | --- |
| GENESIS | genesis 检查点 | $A_0$ | genesis 检查点 | 检查点 |
| REGULAR | §5.2 | 前驱的初始锚点 $A_p$ | 前驱对 $T_p$ 的 FINAL 证据 | 前驱的 handoff 与 $T_p$ 的完整区域结果 |
| FORCE | GFR 在位置 $x$ 输出的 `SessionAssigned(ν)` | $A=\mathrm{last}(G)$ | $A$ 原有的承诺证据：普通块用 $G$ 中记录的 kernel，DER 块用采纳它的 RESUME 的派生证据 | 位置 $x$（Espresso 高度与 L1 高度） |
| RESUME | GFR 在位置 $x$ 输出的 `SessionAssigned(ν)` | $A=D_L[h_{\rm res}]$ | 派生证据：RESUME 序号 $c$、L1 块 $L$ | 位置 $x$、$c$、$L$ |

### 6.2 `VerifyHandoffHistory(handoff)`

```text
VerifyHandoffHistory(handoff = (ν, P_β, E, i_src, λ^open, Q^0, evidence)):
  require P_β == 𝒳 中版本 β 的 policy（版本 β 尚未出现时 WAIT）
  require Π(ν) == Plan(P_β, β, r, E, i_src; 本节点 L1 视图)；WAIT 则 WAIT
  require λ^open == max{λ(A), λ(Π)}，H(ν) == h(A) + TB
  switch cause(ν):
    REGULAR:
      (handoff_p, Res_p) := evidence
      先验证 handoff_p：已安装或已缓存则直接用，否则递归调用本过程
      require β(p) == β，r(p) + 1 == r，E == A_p，i_src == i_p
      require Res_p 的目标块是 T_p，h(T_p) == H_p，A == T_p
      require CheckFINAL(C_p, T_p, D, Σ, π) == true                  # C_p 由 handoff_p 给出
      require q^open(ν) == max{height(ξ(T_p)), q^open_p + 1}
    FORCE, GENESIS:
      require 𝒳 已处理到 evidence 中的位置 x，且在 x 处输出了 SessionAssigned(ν)，字段完全相同
    RESUME:
      同 FORCE；另外 require A == Ckpt[c].D_L[h_res]                 # 用自己的派生视图核对
  return OK(context = (Π(ν), A))
```

递归在“已安装或已缓存的 handoff”处停止。常规 handoff 链的根总是一个 GENESIS、FORCE 或 RESUME 的 handoff，后者由本节点自己的 GFR 核对，所以递归有限。
继承来的末块结果只用 CheckFINAL 检查（公开的 kernel）；目标 session 自己的授权、状态与准入另做完整检查（§7）。

### 6.3 `CanEnter(s)`

$$
\mathsf{CanEnter}(s)\iff \neg\mathsf{Closed}[\beta(s)]\ \wedge\ \neg\mathsf{Closed}[s]\ \wedge\ i(\nu_s)=\text{本区域}\ \wedge\ \text{$s$ 既未进入也未终止}.
$$

它在每条创建与进入路径上都检查，包括本地尚无 Program 时收到 ReadyBundle 的路径。

### 6.4 `InstallHandoff(handoff)`

```text
InstallHandoff(handoff):
  s := sid(handoff.ν)
  if Programs[s] 存在:
      if Programs[s].handoff == handoff: return Assigned(s, C)       # 幂等
      else: return REJECT(CONFLICT)                                   # 同一 pair 的固定字段不被替换
  r := VerifyHandoffHistory(handoff)；WAIT 或 REJECT 则原样返回
  require CanEnter(s)，否则 REJECT(CLOSED)
  if 本节点 ∉ V_s: 记下 handoff 供转发与验证；return NotAssigned
  持久化 Programs[s] := (handoff, Assigned)
  PROG.Create(s, handoff)
  安排准备工作（§7）
  return Assigned(s, r.context)
```

---

## 7. 准备与进入

### 7.1 Ready

已分配的 Program 在满足以下全部条件时为 Ready（同 Part II v4.9 定义 8.1）：

1. handoff 通过 VerifyHandoffHistory 与 CanEnter；
2. 精确锚点、其原始生产身份、完整块体、承诺证据与所需祖先已经验证；
3. 本节点持有精确锚点之后的、已验证的可执行状态，不再需要任何重建或安装；
4. plan（委员会与配额）、分支 policy、引用下界及其验证依据都已可得。

$q^{\rm open}$ 所指的 Espresso 头不必已经存在；第一块会等它（Part 3 §6.3）。为别的锚点准备的状态永远不满足 Ready。

### 7.2 锚点状态的来源

| cause | 状态来源 |
| --- | --- |
| GENESIS | genesis 状态 |
| REGULAR | 后继状态：本区域被选为常规后继时，按前驱已验证的区域结果，逐块执行到 $T_p$（前驱运行期间就持续推进） |
| FORCE | 锚点副本：GFR 输出 `SessionAssigned` 时，若 $A=\mathrm{last}(G)$ 正是基线状态所在的块，立即复制一份 `AnchorState[s]`；否则从最近的状态检查点执行到 $A$ |
| RESUME | $\mathrm{Ckpt}[c]$ 中的 $\mathit{st}(D_L[h_{\rm res}])$；它由派生视图执行得到，与派生规则同版本（O-1、O-10） |

**为什么要副本。** 基线状态随 $G$ 推进。Force 固定锚点之后，新 session 的块进入 $G$ 时基线会越过锚点；没有副本，迟到的准备工作就得重新执行。副本在 `SessionAssigned` 的同一事件中固定，此后不受基线影响。

### 7.3 `PrepareAndEnter(s, stateMaterial)`

```text
PrepareAndEnter(s, stateMaterial):
  require Programs[s].phase == Assigned
  用 stateMaterial 完成 §7.1 的第 2、3 项；未完成则 WAIT
  # 原子段：
  require CanEnter(s)，否则 REJECT(CLOSED)
  rb := ReadyBundle(handoff, 锚点的完整字节与证据, st(A_s) 及其原生验证依据, 所需的来源读取结果)
  持久化 Programs[s].entered := true
  r := PROG.Enter(s, rb)                 # Program 在进入事件中把 rb 发给本区域全体成员
  return r
```

ReadyBundle 只携带完整字节：状态根、URL 或“稍后再取”的承诺都不够。它不携带任何锁、投票或计时器。

### 7.4 收到别人的 ReadyBundle

```text
OnReadyBundle(rb):
  s := sid(rb.handoff.ν)
  if Programs[s] 已进入或已终止，或 Closed 标记已置：忽略
  if Programs[s] 不存在: InstallHandoff(rb.handoff)；不是 Assigned 则停止
  验证 rb：锚点块与承诺证据；状态与锚点的状态根一致（逐项哈希核对）
  PrepareAndEnter(s, rb 中的状态)
```

这是 G-PROG-6（同步进入）的快速路径：第一个诚实成员进入时发出的 ReadyBundle，使其他诚实成员在 $\Delta_i$ 内进入，而不必各自完成准备。

### 7.5 准备时限

由 Part 0 的 $T_{\rm ready}$：从分配事件（GFR 的 `SessionAssigned`，或常规后继的末块进入 $G$ 的位置）起，选中区域的诚实成员在 $T_{\rm ready}$ 内达到 Ready 或已经进入〔G-MGR-6〕。PENDING、OPEN 与 HALT 不选中任何区域，不产生准备义务。

---

## 8. Program 事件与交付任务

### 8.1 `HandleProgramEvent(s, event)`

| 事件 | 持久化的任务 |
| --- | --- |
| `Conf(B)` | 记录；经 RPC 向客户端提供快速确认 |
| `Result(Res)` | 推进 $R$（同 §5.1 第 4 步）；`Offer(Res)`，除非 $s$ 已被 Kill 或关闭且 $B$ 不在 $G$ 中 |
| `Complete(T_s, Res)` | 记录正常终止 `Programs[s] := Terminated(COMPLETE)`；`Offer(Res)`；`RegularSuccessor(s, T_s, Res)` |
| `Quit(C, g, Q)` | `SubmitQuit(C, g, Q)`；`WatchREC(cid, g+1)` |
| `Offense(C, o, ev)` | `SubmitOffense(C, o, ev)` |

重复的事件返回原结果，不重复建任务。

### 8.2 任务的执行

```text
Outbox 中的每个任务带请求标识 rid := H(任务内容)，失败后按退避重试，直到确认完成：
  Offer(Res)              → PUB.Offer(Res, handoffRef)；确认 = Recorded（或 Res 的块已进入 G）
  SendResult(i', Res, h)  → 把完整结果与 handoff 发给区域 i' 的全体成员；确认 = 对方回执或 T_s 进入 G
  SubmitQuit(C, g, Q)     → AUTH.SubmitQuit；确认 = RecoveryRecorded 已最终化
  SubmitOffense(C, o, ev) → AUTH.SubmitOffense；确认 = Slashed 或 AlreadyProcessed 已最终化
  WatchREC(cid, g+1)      → 等 REC(cid, g+1) 在本节点 L1 视图中最终化，然后
                            AUTH.Auth(C, g+1) → PROG.Provide(s, Authority(g+1, AUTH_{s,g+1}))
  Retry(f, args)          → 重新执行 f
```

Program 终止不删除任务：已关闭 session 的证据照样提交（证据窗口 $T_{\rm ev}$ 内），这是 S6 的释放公式所假设的〔G-MGR-5〕。

### 8.3 向 Program 提供输入

- **References：** 每当 PublicStore 中出现新的已最终化 L1 头或 Espresso 头，Manager 把它们交给 Running 的 Program（`Provide(References)`），供 leader 选取 $\rho$、$\xi$。这是读取，不需要任何交易（Part 3 §5.3）。
- **Authority：** 见 `WatchREC`。
- **Candidate：** Program 在担任 leader 时直接调用 `BLD.Build`；Manager 不转手候选。

---

## 9. 生命周期：`ApplyLifecycle(event)`

| 事件 | 处理 |
| --- | --- |
| `Appended(B, σ)` | 推进基线状态到 $B$；若 $B$ 在 $R$ 中，标记为已全局接纳；若 $\sigma=\mathsf{ORD}(\nu)$ 且 $B$ 是 $H(\nu)$，前驱的交接任务可以结束 |
| `SessionAssigned(ν)` | 若 $i(\nu)$ 为本区域：cause 不是 REGULAR 时，按 §6.1 构造 handoff，固定 `AnchorState`，调用 InstallHandoff；cause 为 REGULAR 且已有提前安装的 handoff 时，核对二者一致（不一致即报警：违反 G-AUTH-1 或 G-MGR-1）。更新 `RespView` |
| `SessionClosed(s, reason)` | 原子地：持久化 `Closed[s]`；若 `Programs[s]` 存在且未终止，调用 `PROG.Close(s, reason)`（Assigned 的实例同样关闭）；停止为 $s$ 中尚未进入 $G$ 的结果发布 |
| `BranchClosed(β, reason)` | 原子地：持久化 `Closed[β]`；对每个 $\beta(s)=\beta$ 的 Program 调用 `PROG.Close(s, reason)`；尚未创建的 pair 也因标记而不能再进入 |
| `Resumed(c, β′, h_f, A)` | 把基线状态重置为 $\mathrm{Ckpt}[c]$ 中的 $\mathit{st}(A)$；把 $R$ 重置为新的 $G$（旧分支的结果存档）；丢弃旧的后继状态。旧分支的 Program 已由同一位置的 `BranchClosed(·, RESUME)` 关闭 |
| `Halted(ω)`、`Pending(ω)` | 更新 `RespView`：当前负责者为 $\bot$；不创建 Program |

处理完一批事件后，调用 `KillScan`（§10）。

**关闭先于一切。** 关闭标记先持久化、再调用 `PROG.Close`，两者在同一事件内完成；Program 的每个 BFT 动作都检查自己的关闭标记（Part 3 §6.1）〔G-MGR-3〕。

---

## 10. Kill

### 10.1 目的

Espresso 停机时，GFR 受 R-WAIT 约束而暂停，不再输出任何关闭事件；批次层故障时，GFR 照常增长，但它不知道哪些块进不了 $D$。Kill 让每个节点只凭已最终化的 L1 与 DA，在本地停掉已不可能有用的 session：它们的块既不可能进入 $D$，也不可能再进入 $G$。
Kill 只停止本节点的区域工作：不改变 $G$，不产生 L1 交易，不签任何消息。

### 10.2 D 判死（$\mathsf{DEAD\_D}$）

设本节点的派生视图已派生到高度 $\mathrm{tip}(D)$，每个块带来源标签。

| 检查 | 条件 | 含义 |
| --- | --- | --- |
| K-FORK（分叉） | 存在 $k\le\min\{\mathrm{tip}(D),\mathrm{top}(s)\}$，$\mathrm{native}(D[k])\ne\mathrm{native}(\mathrm{chain}(s)[k])$ | $s$ 的链已与 $D$ 分开 |
| K-OVER（越过） | 存在 $k$，$\mathrm{top}(s)<k\le\min\{\mathrm{tip}(D),H_s\}$，$D[k]$ 的来源为 WINDOW_FILL 或 OVERRIDE_FILL | $D$ 在 $s$ 尚未出块的高度上补了 deposit-only |
| K-EMERG（只读 L1） | 在本节点最新的已最终化 L1 高度 $\ell$ 上处于紧急状态（Part 0 定义 4.3），且 $H_s>\mathit{cursor}(\ell).\mathrm{l2Height}$ | 不必等 DA：cursor 之后的第一个高度已被补成 deposit-only |

**K-EMERG 为什么成立。** 设 $c=\mathit{cursor}(\ell).\mathrm{l2Height}$。紧急状态说明高度 $c+1$ 的截止已过而没有批次被接纳，派生在 $c+1$ 处补了 deposit-only。若 $h(A_s)\le c$，$s$ 还要在 $c+1$ 或之后出块，而它在 $c+1$ 的块不可能来自批次；若 $h(A_s)>c$，$s$ 的锚点经过 $G$ 在 $c+1$ 的普通块，而 $D[c+1]$ 是填充块。两种情形下 $s$ 的后续块都不可能在 $D$ 中。紧急状态的精确边界由派生规则决定，列为 Part 6 的核对项。

**落后的成员不会被误判。** 本地的 $\mathrm{chain}(s)$ 可能落后于 $s$ 的真实进度。若 $s$ 在高度 $k$ 已出好块、且其批次已被接纳，$D[k]$ 的来源是 BATCH，K-OVER 不会触发；K-FORK 只比较本地已知的高度。

**保守的边角情形。** 若 $s$ 在 K-OVER 所指的高度 $k$ 上恰好会出一个与填充块原生相同的空块（同样的 origin、没有普通交易），$s$ 理论上仍可能接着进入 $D$。K-OVER 也会把它关掉。这只损失活性：紧急状态下诚实的 $\mathsf M$ 会发出 RESUME。

### 10.3 G 判死（$\mathsf{DEAD\_G}$）

| 检查 | 条件 |
| --- | --- |
| K-G | 存在 $k\le\min\{h(\mathrm{last}(G)),\mathrm{top}(s)\}$，$\mathrm{native}(G[k])\ne\mathrm{native}(\mathrm{chain}(s)[k])$ |

即 $G$ 在 $s$ 需要的某个高度上已经接纳了别的块；特别地，$G$ 已越过 $h(A_s)$ 而 $G[h(A_s)]\ne A_s$。$G$ 只在末尾追加（RESUME 除外，而 RESUME 同时关闭整个分支），所以 $s$ 永远不能再向 $G$ 追加。

**“过期”不在这里判。** Part 1 §4.3.8 把“$s$ 的下一个高度在 $G$ 中已过期”也列为 G 判死。本稿把它交给 GFR：Espresso 可用时，GFR 处理 `Expire` 并输出 `BranchClosed(β, EXPIRED)`，§9 直接关闭；Espresso 停机时，本地无法可靠判断，因为停机前提出、时间戳不晚于截止的 Espresso 块可能晚些才最终化，从而仍能把 $s$ 的块加入 $G$。停机时起作用的是只依赖 L1 与 DA 的 D 判死。

**它对应原方案的判据 (b)。** $G$ 领先 $D$ 时，session 必须从 $\mathrm{last}(G)$ 派生（Part 0 定义 3.5）；从 $\mathrm{last}(D)$ 派生、却不在 $G$ 上的 session 满足 K-G。在本稿的锚点规则下（Force 取 $\mathrm{last}(G)$，RESUME 取 $D_L[h_{\rm res}]$ 并把 $G$ 换成 $D_L$），GFR 不会开出这样的 session；K-G 是一道兜底检查。

### 10.4 算法

```text
KillScan():
  for s in Programs，phase ∈ {Assigned, Running} 且 ¬Closed[s]:
      r := Kill(s)

Kill(s):
  reason := ⊥
  if K-FORK(s) 或 K-OVER(s) 或 K-EMERG(s): reason := DEAD_D
  elif K-G(s): reason := DEAD_G
  if reason == ⊥: return Alive
  原子地：持久化 Closed[s] := KILL(reason)；PROG.Close(s, KILL)
  RespView 中去掉 s；Outbox 中取消 s 尚未进入 G 的结果的 Offer
  保留区域结果、签名记录与证据；未完成的证据提交照常进行
  return Killed(s, reason)
```

`KillScan` 在每次 `ObservePublic` 之后运行，也就是每当派生视图或 L1 视图前进一个块之后。
检查代价：K-FORK 与 K-G 只需比较 $\mathrm{chain}(s)$ 与 $D$、$G$ 的最高共同高度一带，因为三者都是链，较低高度一致时只要比较新增部分；K-OVER 只看 $(\mathrm{top}(s),\min\{\mathrm{tip}(D),H_s\}]$ 中新派生的块；K-EMERG 只读 cursor。

### 10.5 与 GFR 关闭的关系

| 情形 | 谁先关闭 | 之后 |
| --- | --- | --- |
| Espresso 长停机 | Kill（K-EMERG 或 K-OVER），在停机期间 | Espresso 恢复后 GFR 处理 `Expire`，输出 `BranchClosed(β, EXPIRED)`，对已关闭的 Program 是无操作 |
| 批次层故障超过窗口 | Kill（K-EMERG，或派生到分叉点后的 K-FORK、K-OVER） | GFR 照常处理；最终由 RESUME 的 `BranchClosed(β, RESUME)` 关闭整个分支 |
| 区域停摆超过 $W_{\rm exp}$（Espresso 正常） | GFR 的 `Expire` | Kill 随后也会因 K-EMERG 成立，无操作 |
| Force、RESUME | GFR | — |

两者只在时间上不同，结论一致：都关闭不可能再进入 $G$ 与 $D$ 的 session。

---

## 11. `Responsibility()`

```text
Responsibility():
  cur := ⊥; nxt := ⊥
  if 𝒳.Tail == NEXT(ν) 且 ¬Closed[sid(ν)]: cur := (sid(ν), i(ν))
  if 𝒳.Tail == NEXT(ν):
      Π' := Plan(P_β, β, r(ν)+1, A(ν), i(ν); 本节点 L1 视图)        # 常规后继
      if Π' != WAIT 且 region(Π') != ⊥: nxt := region(Π')
  return (cur, nxt)
```

Tail 为 PENDING、OPEN 或 HALT 时，当前负责者为 $\bot$。被 Kill 的 session 不再算作负责者。
提前进入的常规后继就是 `nxt`：它在前驱的末块进入 $G$ 之前就可以接受交易与出块（Part 1 §4.1.1）。

---

## 12. 保证的证明

### 12.1 G-MGR-1（handoff 真实）

Program 只在 InstallHandoff 中创建，而后者先调用 VerifyHandoffHistory。

- **REGULAR：** 前驱的末块结果在前驱自己的 context 下通过 CheckFINAL，所以由 G-PROG-2，$T_p$ 曾被前驱的某个诚实成员完整验证后提交；目标 pair、plan 与 $q^{\rm open}$ 由确定的规则重算，与发送方无关。前驱的 context 本身递归地由同样的检查或由本节点的 GFR 给出。
- **FORCE、RESUME、GENESIS：** handoff 与本节点 GFR 在同一位置输出的 `SessionAssigned` 完全相同；由 G-GFR-1，所有诚实节点在该位置得到同一个 descriptor。RESUME 的锚点另用本节点的派生视图核对（G-DER-1）。
- **固定字段不被替换：** `Programs[s]` 一经持久化，InstallHandoff 对不同的 handoff 返回 `REJECT`。

### 12.2 G-MGR-2（单次进入，不复活）

PrepareAndEnter 在原子段中先检查 CanEnter，再持久化 `entered`，然后调用 `PROG.Enter`；Program 自己也有 SR-INIT（Part 3 §2.3）。`Closed` 只增不减，CanEnter 拒绝已关闭或已终止的 pair，包括崩溃重启之后。

### 12.3 G-MGR-3（及时关闭）

§3 的事件循环保证 Manager 的一个事件内不插入 Program 的动作；§9 与 §10.4 在同一事件内先持久化关闭标记、再调用 `PROG.Close`。之后 Program 的守卫拒绝一切 BFT 动作（G-PROG-4）。

### 12.4 G-MGR-4（无自由裁量）

Manager 的每个决定都是以下输入的确定函数：GFR 的输出（G-GFR-1）、派生视图（G-DER-1）、L1 Authority 的纯函数（G-AUTH-1）、经 CheckFINAL 验证的区域结果。Kill 的四个检查只读 $D$（只从已最终化 L1 与 DA 派生）、已最终化 L1 上的 cursor，以及 $\mathcal X$。两个诚实 Manager 在相同的输入前缀上作出相同的决定；它们只可能在不同时刻到达这个前缀。

### 12.5 G-MGR-5（持久与交付）

结果、证据与任务在决定段持久化（§3），任务一直重试到确认完成（§8.2），不随 Program 终止而删除。由 A-RELAY，每项任务都有在线诚实方承担，所以每份 quit 与 offense 证据都会提交给 L1 Authority。〔持久化存储、A-RELAY〕

### 12.6 G-MGR-6（准备活性）

后继状态在前驱运行期间持续推进；FORCE 的锚点副本在分配事件中固定；RESUME 的状态在派生视图处理 `CursorSet` 时就已算出。剩余的工作（验证、组装 ReadyBundle）在 $T_{\rm ready}$ 的前提之内（Part 0 §2.10）。〔$T_{\rm ready}$〕

### 12.7 G-MGR-7（Kill 不误杀、及时）

**不误杀。** 设 `Kill(s)` 输出 Killed。

- K-FORK：$D$ 在高度 $k$ 的块已经固定（G-DER-1：已派生的前缀不再改变），与 $\mathrm{chain}(s)[k]$ 不同。$s$ 的任何块都以 $\mathrm{chain}(s)[k]$ 为祖先或就在 $k$ 之上延伸它，所以都不可能出现在 $D$ 中。
- K-OVER：$D[k]$ 是填充块，已经固定。$s$ 在 $k$ 及之上的块要进入 $D$，$s$ 在 $k$ 的块就必须等于 $D[k]$，而 $s$ 在 $k$ 处的块要么不同于填充块，要么是 §10.2 的边角情形。
- K-EMERG：见 §10.2 的理由，归结为 K-OVER 或 K-FORK 在高度 $c+1$ 处成立。
- K-G：$G$ 在高度 $k$ 已接纳了别的块；$G$ 只在末尾追加，唯一的例外 RESUME 同时关闭整个分支（G-GFR-2），所以 $s$ 永远不能再向 $G$ 追加。

在这四种情形下，$s$ 已不可能对 $G$ 或 $D$ 有贡献；Kill 之后本节点不再为 $s$ 签任何消息（G-PROG-4）。

**及时。** `KillScan` 在派生视图或 L1 视图每前进一个块之后运行。所以 $D$ 越过或偏离 $\mathrm{chain}(s)$ 之后，只要本节点已派生到该高度，至多一个 L1 块内关闭 $s$；用 K-EMERG 时不需要派生，只需 L1。〔L1-S、DA-S1；依赖 G-DER-1〕

### 12.8 G-MGR-8（负责性视图）

`Responsibility()` 只读 $\mathcal X$ 的 Tail 与常规后继的 plan；由 G-GFR-1 与 G-AUTH-1，它与 GFR 的 Tail 以及公开可算的常规后继一致。被 Kill 的 session 被去掉，这只会让 Builder 更早拒绝交易，不影响任何安全性质。

---

## 13. 例子：Manager 在两类紧急情形下的动作

### 13.1 Espresso 长停机

设 $W_{\rm exp}=10$ h，$W_{\rm seq}=12$ h。区域 B 的 session $s_B$ 负责 501–600，Espresso 在 $G$ 到 520 时停机。

| 时刻 | 公共事件 | 区域 B 的 Manager |
| --- | --- | --- |
| 停机后 | GFR 在 R-WAIT 处等待 | 不变；$s_B$ 继续区域出块，结果留在 $R$ 与 Outbox |
| $t(521)+W_{\rm seq}$ 前后 | 派生在 521 处窗口回退，补 deposit-only；cursor 进入紧急状态 | KillScan：K-EMERG（或派生到 521 后的 K-OVER）成立，`Kill(s_B)`；Outbox 中 $s_B$ 的 Offer 取消 |
| Espresso 恢复 | GFR 处理 `Expire(521)`：`BranchClosed(β, EXPIRED)`、`Halted` | 对已关闭的 $s_B$ 无操作；`RespView` 为 $\bot$ |
| RESUME 记录在 $L$ | 派生处理 `CursorSet`，补到 $h_{\rm res}$ | 保存 $\mathrm{Ckpt}[c]$ |
| 键 $\mathit{ts}(L)+M_a$ | GFR：`Resumed`；下一个 Espresso 块 `SessionAssigned(ν)` | 重置基线为 $\mathit{st}(D_L[h_{\rm res}])$；若被选中，InstallHandoff → PrepareAndEnter（状态取自检查点）→ 新 Program 在 $t(h_{\rm res}+1)$ 出块 |

### 13.2 批次层故障超过窗口

$G$ 照常增长到 700，Batch Inbox 的 cursor 停在 640。

| 时刻 | 公共事件 | Manager |
| --- | --- | --- |
| $t(641)+W_{\rm seq}$ 前后 | 派生在 641 处补 deposit-only，$D$ 与 $G$ 分叉；cursor 进入紧急状态 | KillScan：所有 $H_s>640$ 的 session（当前负责者与提前进入的常规后继）被 K-EMERG 关闭；$G$ 不再增长 |
| 批次层恢复，RESUME | GFR：`BranchClosed(β, RESUME)`；$G$ 被替换为 $D_L$，641 起的旧内容被撤销，$h_f=641$ | 与 13.1 相同 |

---

## 14. 与 Part II v4.9 的差异

| 项目 | v4.9 | 本稿 |
| --- | --- | --- |
| 跟踪对象 | $G$、$R$、更新日志 | 另加派生视图 $D$ 与 `CursorSet` 检查点 |
| handoff 的 cause | REGULAR、FORCE、GENESIS | 另加 RESUME（锚点 $D_L[h_{\rm res}]$，用派生视图核对） |
| 关闭来源 | 验证到的 Force | GFR 的 `SessionClosed`、`BranchClosed`（FORCE、EXPIRED、RESUME、COMPLETE），以及本地 Kill |
| 生命周期事件 | 无（Force handler） | `ApplyLifecycle` 处理 Part 1 §2.7 的全部事件 |
| 锚点状态 | 基线与后继状态 | 另加锚点副本与 RESUME 状态 |
| 程序事件 | 完成 | Conf、Result、Complete、Quit、Offense，经 Outbox 持久交付 |
| 负责性视图 | 无 | `Responsibility()`，供 Builder 判断是否接受交易 |

---

## 15. 待定问题

1. **派生来源标签（Part 6）：** Derivation Rule 是否能为每个块输出 BATCH、WINDOW_FILL、OVERRIDE_FILL 的标签；mega-espresso 的实现中这一信息是否现成。
2. **紧急状态的精确边界（Part 6）：** `ℓ > cursor.epoch_num + seq_window_size + 1` 与原生 batch queue 的窗口回退时刻是否逐块一致；K-EMERG 的正确性依赖它。
3. **状态检查点的保留策略：** FORCE 的锚点可能是 $G$ 中较旧的块（Force 生效时 $G$ 停在那里）；节点需要保留足够的状态检查点，或保留锚点副本，直到对应 session 进入或被关闭。
4. **快速确认的 RPC 接口** 属于实现范围。
