# MegaETH Sequencer Rotation 技术报告 · Part 3

## Session Program 协议（中文工作稿 v0.1）

日期：2026-10-26。
状态：草稿。

本稿实现 [Part 1 v0.3](part1_module_interfaces_zh_v0_3.md) §4.4 的 Session Program（PROG）：一个固定 session 在一个成员节点上的区域共识实例。先复述接口与保证，再给出实例状态、块与消息的格式、准入与完整验证、五个协议框，以及各条保证的证明。

前置文档：

- [Part 0 v0.8](part0_model_goals_zh_v0_8.md)、[Part 1 v0.3](part1_module_interfaces_zh_v0_3.md)；
- [Part 2 v0.1](part2_l1_authority_zh_v0_1.md)：Plan、Auth、Leader 与证据的链上处理；
- Part I（`formalization_v3/sync_hotstuff_rr_part1_core.tex`）：同步共识的核心与安全证明；
- Part II v4.9（`formalization_v3/sync_hotstuff_rr_part2_rotation_v4_9.tex`）§3–§4：区域引擎（session、配额、DRAIN、FINAL、ReadyBundle）。

本稿的共识核心与 Part II v4.9 相同。下文用 **【改】** 标出相对 v4.9 的修改，其余照搬。

**本稿对 Part 1 v0.3 的补充**（实现时发现，待确认）

1. **引用必须足够旧（V-AGE）。** 投票者只接受这样的引用：L1 origin 满足 $\mathit{ts}(o)\le\mathit{now}-(T_{\rm L1}+\Delta_{\rm ts})$；$\xi$ 所指的 Espresso 块已在本地日志中出现至少 $T_E$。到那时所有诚实成员都已读到这些数据，对同一个 proposal 的结论相同；不需要附任何证明，也不需要任何交易（§5.3）。
2. **FINAL 也有槽位记录。** 诚实成员在签 FINAL 之前持久化 $(h\mapsto\mathsf H(B))$；与自己已提交日志冲突的块，不提交、不签 FINAL，并报警停止签名（§6.3）。这使 S5(a) 在 A-BYZ 失效时仍然成立。
3. **过期时限只约束提议，不放进投票条件。** leader 不早于 $t(h)$ 提议高度 $h$（追赶时 $t(h)$ 已过，立即提议），也不为来不及在 $t(h)+W_{\rm exp}$ 之前进入 Espresso 的高度提议。Part 1 §4.4 原把时效也列为投票条件。放进投票条件其实不会造成冲突的提交：过了时限，谁都不能再为这个高度投票。但这会让 Part I 的证明不能直接套用，又没有收益，因为过期的块由 GFR 的 R-EXP 拒绝（§7.1）。
4. **RESUME 锚点的基础证明。** RESUME 开出的 session，其锚点是 DER 块，基础证明 $Q^0$ 是 handoff 中的派生证据，而不是区域证书（§4.4）。

---

## 0. 定位

**负责什么。** 在固定的 session context 下运行区域共识：提议、投票、提交、term 轮换、恢复代切换；产生快速确认、区域最终证书与区域结果；检测并输出 quit 与 offense 证据。

**不负责什么。**

- 不选择自己是否存在、何时开始、何时被关闭：这由 Session Manager 决定（Part 4）。
- 不构造候选块、不做原生执行：由 Block Builder 负责，Program 只调用它的 `Build` 与 `ValidateCandidate`。
- 不把块写进 $G$：区域最终只是 GFR 接纳的前提（Part 5）。
- 不直接与 L1 或 Espresso 交互：证据提交、发布与 REC 的读取都经由 Manager。

---

## 1. 复述：接口、事件与保证

本节复述 Part 1 v0.3 §4.4。

### 1.1 接口

| 接口 | 输入 | 输出 | 要点 |
| --- | --- | --- | --- |
| `Create(s, handoff)` | Manager 已验证的 handoff | `Assigned(s, C)` 或 `REJECT` | 建立独立状态；不复制前驱的锁，不启动计时器，不授签名权 |
| `Enter(s, readyBundle)` | 对应固定锚点的完整 ReadyBundle | `Entered(s, τ_entry)` 或 `REJECT` | 只进入一次；初始化入口锁 $Q^0_s$、执行状态与 $(g,t)=(0,0)$；向本区域发送 ReadyBundle |
| `Provide(s, input)` | `Candidate`、`References` 或 `Authority(g, AUTH_{s,g})` | `Installed`、`WAIT` 或 `REJECT` | 不改委员会；不自动触发投票 |
| `Step(s, event)` | `Receive(m)` 或 `Timer(id, τ)` | 对等消息、程序事件、新状态 | 先处理证据，再处理投票与计时器；签名前先持久化 |
| `Close(s, reason)` | $\mathit{reason}\in\{\mathsf{COMPLETE},\mathsf{FORCE},\mathsf{EXPIRED},\mathsf{RESUME},\mathsf{KILL}\}$ | `Terminated(s, reason)` | 立即取消一切后续 BFT 权限 |

### 1.2 输出事件

| 事件 | 何时产生 |
| --- | --- |
| `Conf(s, B)` | 本成员在区域内提交了 DATA 块 $B$ |
| `Result(Res)` | 某个 DATA 块的完整区域结果已经形成 |
| `Complete(T_s, Res)` | 末块 $T_s$ 已有完整区域结果；实例正常结束 |
| `Quit(C, g, Q)` | 第 $g$ 代的完整 quit 证据已经形成 |
| `Offense(C, o, ev)` | 观察到成员 $o$ 的冲突签名 |

### 1.3 生产约束

诚实成员只为满足以下全部条件的块提议、投票与签 FINAL：

- **高度：** $h\le H_s$；
- **引用：** $\lambda(B)\ge\lambda^{\rm open}$，$\mathrm{height}(\xi(B))\ge q^{\rm open}$；R-FRESH：$\mathit{ts}(o(B))\ge t(h(B))-D_{\rm drift}$；
- **派生一致：** 候选在父块上满足 DerivOK（经 `BLD.ValidateCandidate`）；
- **时效：** 不为来不及在 $t(h)+W_{\rm exp}$ 之前进入 Espresso 的高度出块；新锚点落在过去时，允许以快于 $b_t$ 的速度追赶。

本稿把“时效”落实为只约束提议（§7.1）；其余三条同时约束提议、投票与 FINAL。另外，投票者只接受足够旧的引用（V-AGE，§5.3）。

### 1.4 保证

| 保证 | 内容（简述） | 依赖 |
| --- | --- | --- |
| G-PROG-1 区域一致 | $s$ 良态时，两个诚实成员不会在同一高度提交不同的块 | 良态、$\Delta_i$、A-CRYPTO |
| G-PROG-2 FINAL 可靠 | A-BYZ 下，权重 $>F_s$ 的 FINAL 证书所签的块曾被诚实成员完整验证后提交 | A-BYZ、A-CRYPTO；G-MGR-1、G-BLD-1、G-BLD-2 |
| G-PROG-3 生产约束 | 诚实成员只为满足 §1.3 的块提议、投票与签 FINAL | — |
| G-PROG-4 隔离与关闭 | 实例互不共享核心状态；关闭后不再签名；至多一次初次 Enter | — |
| G-PROG-5 诚实不留证据 | 诚实成员不在同一槽位签冲突消息；输出的 quit 证据都有效 | 持久化存储 |
| G-PROG-6 同步进入 | 第一个诚实成员 Enter 之后 $\Delta_i$ 内，所有诚实成员都已进入 | 良态、$\Delta_i$ |
| G-PROG-7 区域活性 | 良态、A-LEAD、输入可得且未被关闭时，每个高度在 $T_{\rm pos}$ 内提交，末块结果在 $T_{\rm sess}$ 内形成 | 良态、A-LEAD |
| G-PROG-8 证据输出 | 观察到的 quit 或 offense 证据在 $\Delta_i$ 内输出给 Manager | — |

---

## 2. 实例状态

### 2.1 固定字段（由 handoff 给出，此后不变）

| 字段 | 含义 |
| --- | --- |
| $s=(\beta,r)$；cause | session 与开出它的原因 |
| $C_s=(\Pi_s,A_s)$；$\mathrm{cid}_s=\mathsf H(\mathrm{id}(\Pi_s),\mathsf H(A_s))$ | context 与其标识；所有签名绑定 $\mathrm{cid}_s$ |
| $i$、$V_s$、$\{w_o\}$、$W_s$、$F_s$ | 区域、委员会、权重、门槛（来自 $\Pi_s$） |
| $\mathrm{TB}$、$K$、$\Delta:=\Delta_i$ | 配额、term 位置上限、区域时限 |
| $A_s$、$H_s=h(A_s)+\mathrm{TB}$ | 锚点与末块高度 |
| $Q^0_s$ | 锚点的基础证明（§4.4） |
| $\lambda^{\rm open}_s=\max\{\lambda(A_s),\lambda(\Pi_s)\}$、$q^{\rm open}_s$ | L1 与 Espresso 引用的下界 |
| $E_s$ | 选区参考块，用于 leader 种子 |

### 2.2 可变字段

| 字段 | 含义 |
| --- | --- |
| $\mathit{phase}\in\{\mathsf{Assigned},\mathsf{Running},\mathsf{Terminated}\}$ | 宿主阶段（Manager 也记录） |
| $\mathit{engine}\in\{\mathsf{Active},\mathsf{Frozen},\mathsf{Completed}\}$ | Running 时的引擎状态 |
| $(g,t)$；$e_v(g,t)$ | 当前视图；各视图的本地进入时刻 |
| $\mathsf{AUTH}_{s,g}$ | 当前代的授权（$g>0$ 时由 Manager 经 `Provide` 交来） |
| `Seen` | 收到的 proposal、投票、QC、STATUS、NEWVIEW、BLAME、FINAL，按发送者与槽位索引 |
| $Q_v^{(g)}$ | 进入第 $g$ 代时固定的入口锁 |
| `Timers` | 提交计时器（每个已投 proposal 一个）；进度检查（每个 term 至多 $m$ 个） |
| `Log` | 已提交的块序列，从 $A_s$ 开始；以及每个已提交 DATA 之后的执行状态 |
| `FinalSigs` | 收集到的 FINAL 签名，按 $(h,\mathsf H(B))$ 索引 |
| `Buffer` | 未来代的消息与证据 |
| `closed` | 关闭标记，由 Close 设置 |

### 2.3 持久化记录

以下记录在**发出签名之前**写入持久存储；崩溃重启后从中恢复，绝不重签与之冲突的消息〔G-PROG-5〕。

| 记录 | 键 | 值 |
| --- | --- | --- |
| SR-PROPOSE | $(g,t,j)$ | 已签 proposal 的块哈希 |
| SR-FRESH | $(g,t,j)$ | 已投 DATA-VOTE 或 DRAIN-VOTE 的块哈希 |
| SR-NEWVIEW | $g$ | 已签 NEWVIEW 的开口身份 |
| SR-ANCHOR | $g$ | 已投 ANCHOR-VOTE 的开口身份 |
| SR-FINAL | $h$ | 已签 FINAL 的块哈希【改】 |
| SR-BLAME | $(g,t)$ | 已发送 |
| SR-ENTRY | $g$ | 入口锁 $Q^{(g)}_v$ 与进入时刻 |
| SR-INIT | — | 初次 Enter 已发生 |
| SR-CLOSED | — | 关闭原因 |

已提交的日志也持久保存。计时器不持久化：重启后，未过期的提交计时器视为已取消，等价于一次本地的崩溃—恢复（与 Part I 的模型一致：崩溃的成员在该 session 中按离线计）。

---

## 3. 块、位置与配额

### 3.1 块的格式

$$
B=(\beta,r,k,h,\mathit{type},x,\mathsf H(B_{\rm parent}),\rho,\xi),\qquad\mathit{type}\in\{\mathsf{DATA},\mathsf{DRAIN}\}.
$$

- **DATA：** $x$ 是一个完整的原生 L2 块输入：有序普通交易、高度 $h$、时间戳 $t(h)$、L1 origin、前缀（L1 attributes、deposit、系统交易）与 mini-block 划分。$h=h(B_{\rm parent})+1$。
- **DRAIN：** 不带 $x$；$h=h(B_{\rm parent})$；$\rho$、$\xi$ 复制末块 $T_s$ 的值。
- $k=k_s(B_{\rm parent})+1$，$k_s(A_s)=0$。

**【改】L1 引用即 origin。** DATA 块的 $\rho$ 必须等于 $x$ 的 L1 origin $o(x)$（高度与哈希），$\lambda(B):=\mathrm{height}(o(x))$（R30）。$\rho$ 不再是独立选择的字段。

**Espresso 引用** $\xi(B)=(q,\mathsf H(J_q))$ 指向一个已最终化的 Espresso 块；沿 DATA 祖先不减。它只给 GFR 的 (d) 与常规后继的 $q^{\rm open}$ 用，不参与选区。

### 3.2 term、位置与配额（同 v4.9）

- 视图 $(g,t)$ 的锚点 $A_{g,t}$：$A_{0,0}=A_s$；$g>0$ 时 $A_{g,0}$ 由 NEWVIEW 选定；$t>0$ 时 $A_{g,t}$ 是 term $(g,t-1)$ 的终端块。
- 若 $h(A_{g,0})<H_s$，第 $g$ 代是 **DATA 代**：term $(g,t)$ 有

$$
m_{g,t}=\min\{K,\ H_s-h(A_{g,t})\}
$$

个 fresh 位置 $j=0,\dots,m_{g,t}-1$；位置 $j$ 的块高度为 $h(A_{g,t})+j+1$，物理位置为 $k_s(A_{g,t})+j+1$。终端位置的 QC 若低于 $H_s$，就准入 term $(g,t+1)$。
- 若 $g>0$ 且 $h(A_{g,0})=H_s$，第 $g$ 代是 **DRAIN 代**：只有 $(t,j)=(0,0)$ 一个位置，块为 DRAIN。
- 配额只数 $A_s$ 之后的 DATA；DRAIN、重发与 ANCHOR 投票都不占配额。

### 3.3 父块规则（同 v4.9）

| 条件 | $B_{\rm parent}$ | $Q_{\rm parent}$ |
| --- | --- | --- |
| $g=t=j=0$ | 固定锚点 $A_s$ | 基础证明 $Q^0_s$ |
| $g>0,\ t=j=0$ | 恢复开口 $A_{g,0}$ | 它在 $(g,0)$ 中的 ANCHOR QC |
| $j=0,\ t>0$ | $A_{g,t}$，即 term $(g,t-1)$ 的终端块 | 它在 $(g,t-1)$ 中的 proposal QC |
| $1\le j<m_{g,t}$ | term $(g,t)$ 的位置 $j-1$ | 它在 $(g,t)$ 中的 proposal QC |

---

## 4. 签名消息与证书

### 4.1 签名内容

每条消息签名覆盖公共头 $\mathit{hdr}=(\mathit{domain},\mathit{chain},\mathit{version},\mathit{type},\mathrm{cid}_s)$ 与下表的字段。

| 类型 | 槽位 | 其余签名字段 |
| --- | --- | --- |
| PROPOSE | $(g,t,j)$ | 开口身份、块类型、$k$、$h$、$\mathsf H(B)$ |
| DATA-VOTE、DRAIN-VOTE | $(g,t,j)$ | 同上 |
| STATUS | $(g,0)$ | 入口锁、入口证明 |
| NEWVIEW | $(g,0)$ | 选择的开口身份、其证明 $Q^*$、入口证明 |
| ANCHOR-VOTE | $(g,0)$ | 开口身份 |
| BLAME | $(g,t)$ | 无 |
| FINAL | $h$ | $\mathsf H(B)$；不含 $g,t$ |

第 $g$ 代的**开口身份**为 $(g,k_s(A_{g,0}),h(A_{g,0}),\mathsf H(A_{g,0}))$。

### 4.2 证书

证书是来自 $V_s$ 中不同成员、对同一内容的签名，总权重严格大于 $F_s$。

| 证书 | 签名类型 | 用途 |
| --- | --- | --- |
| proposal QC $\mathrm{QC}_{g,t}(B)$ | DATA-VOTE 或 DRAIN-VOTE | 作为子块的父证明；终端 QC 低于 $H_s$ 时准入下一个 term |
| ANCHOR QC | ANCHOR-VOTE | 第 $g>0$ 代在 fresh 提议之前重新认证开口 |
| BLAME 证书 | BLAME | quit 证据 (a) |
| FINAL 证书 $\mathrm{FC}_s(B)$ | FINAL | 区域最终；CheckFINAL 的签名部分 |

QC 不启动任何计时器；ANCHOR 投票也不启动。

### 4.3 rank

session 内，term $(g,t)$ 中形成的 QC 的 rank 为 $(g,t,k_s(B))$，按字典序比较。第 $g>0$ 代的 ANCHOR QC 的 rank 为 $(g,0,k_s(A_{g,0}))$。相同 rank 用对内容的固定全序打破平局；签名者集合与编码不影响 rank。

### 4.4 基础证明 $Q^0_s$

$Q^0_s$ 证明锚点 $A_s$ 本身，rank 低于本 session 的一切 QC，直接作为第一块的父证明，也作为恢复时“没有更高 QC”时的入口锁。按 cause：

| cause | $A_s$ | $Q^0_s$ |
| --- | --- | --- |
| GENESIS | $A_0$ | genesis 检查点 |
| REGULAR | 前驱的末块 $T_p$ | 前驱对 $T_p$ 的直接或覆盖 FINAL 证据 |
| FORCE | 生效时的 $\mathrm{last}(G)$ | 该块原有的承诺证据：普通块用其生产 session 的 FINAL 证据，DER 块用采纳它的 RESUME 的派生证据 |
| RESUME【改】 | $D_L[h_{\rm res}]$（DER 块） | 派生证据：RESUME 记录的位置与 $L$；任何人可以按派生规则重算 $D_L$ 核对 |

Program 不自己核对 $Q^0_s$ 的来源：handoff 由 Manager 的 VerifyHandoffHistory 核对（G-MGR-1），Program 只把它当作固定字段。

### 4.5 区域结果与覆盖

- **FINAL 证书覆盖祖先。** $\mathrm{FC}_s(D)$ 加上从 $D$ 回到 $B$ 的完整同 session 路径，证明 $B$ 已被提交（至少一个诚实签名者提交了 $D$，因而提交了它的祖先）。覆盖不跨 session。
- **区域结果** $\mathit{Res}=(C_s,B,D,\Sigma,\pi)$：$\Sigma$ 是 $D$ 上的 FINAL 签名集合，$\pi$ 是从 $D$ 回到 $B$ 的块体。$(D,\Sigma,\pi)$ 是 $B$ 的 kernel，供 CheckFINAL 使用（Part 1 §3.3）。

---

## 5. 准入与完整验证

### 5.1 准入（同 v4.9）

- $(0,0)$：在本节点初次 Enter 时准入；入口证明是 handoff，固定 $A_s$ 与 $Q^0_s$。第 0 代没有 NEWVIEW 与 ANCHOR 投票。
- $(g,0)$，$g>0$：持有 $\mathsf{AUTH}_{s,g}$ 与第 $g-1$ 代的 quit 证据（入口证明）时准入。
- $(g,t)$，$t>0$：另持有 term $(g,0),\dots,(g,t-1)$ 的终端 QC，且这些终端块都低于 $H_s$。

视图消息**可准入**，当且仅当其视图已准入、签名上下文与 $\mathrm{cid}_s$ 和 $g$ 一致、签名有效。leader 签名的消息按 $L_s(g,t)=\mathsf{Leader}(\mathsf{AUTH}_{s,g},t)$ 核对。验证远端历史时，用准入见证代替本地准入事件：$(0,0)$ 用 handoff，$(g,0)$ 用入口证明链，$t>0$ 用此前的终端 QC。

### 5.2 完整验证

对一个 proposal $P=\langle\mathsf{PROPOSE},g,t,j,B,Q_{\rm parent}\rangle$，完整验证依次检查：

| 编号 | 检查 |
| --- | --- |
| V-SIG | 签名有效，$\mathit{hdr}$ 中的 cid 为 $\mathrm{cid}_s$ |
| V-ADM | 视图 $(g,t)$ 已准入，或附有准入见证 |
| V-LDR | 签名者是 $L_s(g,t)$ |
| V-POS | 块类型、配额（$h\le H_s$）、$j<m_{g,t}$、$k$ 与 $h$ 符合 §3.2；DRAIN 只出现在 DRAIN 代 |
| V-PAR | 父块与 $Q_{\rm parent}$ 符合 §3.3；$Q_{\rm parent}$ 是有效证书（或 $Q^0_s$、ANCHOR QC） |
| V-HASH | 从 $B$ 回到 $A_s$（或回到已验证的祖先）的哈希链接完整；每个祖先都有签名的 proposal 与块体 |
| V-REF | DATA：$\rho=o(x)$；本 session 第一个 DATA 有 $\lambda\ge\lambda^{\rm open}_s$；$\xi$ 指向已最终化的 Espresso 块，沿祖先不减，第一个 DATA 有 $\mathrm{height}(\xi)\ge q^{\rm open}_s$。DRAIN：$\rho,\xi$ 等于 $T_s$ 的值 |
| V-DER【改】 | DATA：`BLD.ValidateCandidate(native(parent), st(parent), x, nctx)` 返回 `VALID`，即 DerivOK 的 V1–V4 全部成立（含 R-FRESH）；对所需祖先中尚未验证的 DATA 同样检查 |
| V-AGE【改】 | DATA：$\mathit{ts}(o(x))\le\mathit{now}-(T_{\rm L1}+\Delta_{\rm ts})$；$\xi$ 所指的 Espresso 块已在本地日志中出现至少 $T_E$（§5.3）。DRAIN 的引用复制自 $T_s$，不另检查 |

检查结果为三种之一：有效；无效（丢弃）；缺依赖（保留并请求，依赖到达后重做）。V-AGE 尚不满足时，按缺依赖处理：保留，到满足时重做。

**为什么 V-DER 是投票条件。** 批次中没有 deposit；派生从 origin 自己构造前缀。区域块的前缀若与派生构造的不同，或任何一条派生检查不过，$D$ 中的块就与 $G$ 中的块不同，L2-L1 失效。$\mathsf C$ 只核对普通交易列表、时间戳与 origin，查不出这类差异（Part 1 §3.2）。

### 5.3 【改】引用必须足够旧（V-AGE）

**问题。** V-DER 要求 origin 已最终化，V-REF 要求 $\xi$ 指向已最终化的 Espresso 块。每个成员都按自己的日志判断“已最终化”，而不同成员看到同一个块最终化的时刻不同。如果引用的是刚刚最终化的块，A 已经看到、B 还没看到，B 就暂时验证不了。Part I 的锁论证要求“我能投的块，别的诚实成员在 $\Delta_i$ 内也能投”（§8.1），这种差异会破坏它。

**规则。** 投票者只接受足够旧的引用：

- **L1 origin：** $\mathit{ts}(o)\le\mathit{now}-(T_{\rm L1}+\Delta_{\rm ts})$。由 A-TS，$o$ 在 $\mathit{ts}(o)+\Delta_{\rm ts}$ 之前提出；由 L1-L3，它若最终化，就在提出后 $T_{\rm L1}$ 内出现在所有诚实成员的日志中。所以到这个时刻，要么所有诚实成员都已读到 $o$ 已最终化，要么谁也读不到。
- **Espresso 引用：** $\xi$ 所指的块已在本地日志中出现至少 $T_E$。由 Espresso 的 X-L2，此时它已出现在所有诚实成员的日志中。这里不用时间戳判断新旧，因为 Espresso 的块可能在停机之后才最终化，时间戳却是提出时的。

**为什么这样就够。** 两个条件都只会随时间从“不满足”变成“满足”。A 投票时满足，B 稍晚收到时也满足，并且 B 已经读到同样的数据，所以 B 能在 $\Delta_i$ 内完成同样的验证。不需要在提议中附任何证明，也不需要任何交易。

**代价。** origin 至少落后 $T_{\rm L1}+\Delta_{\rm ts}$。要求 $D_{\rm drift}$ 大于它并留余量，leader 才总能找到既足够旧、又满足 R-FRESH 的 origin（§7.1）。区域安全因此也依赖 $T_{\rm L1}$、$T_E$ 与 A-TS（Part 0 §2.10）。

验证包仍按 Part II v4.9 的规则携带块与所需祖先；父块的执行状态由接收方自己按祖先执行得到，或来自已验证的缓存。

### 5.4 证据优先（同 v4.9）

验证一条消息之前、丢弃重复之前，先检查其中的全部签名材料是否构成 quit 证据或 offense 证据（§6.6、§6.7）。证据只需自身的签名与准入见证有效；即使承载它的消息无效或不完整，证据照样计入。比较对象是本实例保存的全部材料，不只是消息内部的对象。

---

## 6. 协议

### 6.1 程序守卫与处理顺序

**守卫。** 每个 BFT 动作——提议、投票、STATUS、NEWVIEW、BLAME、FINAL、提交计时器回调、term 变化、恢复——都要求：

$$
\mathit{phase}=\mathsf{Running}\ \wedge\ \neg\mathit{closed}.
$$

缓冲消息与授权到达的回调同样受守卫约束。Close 一旦执行，守卫在下一个动作之前就已关闭。

**处理顺序。** 实例按以下顺序处理每条收到的消息：

1. 检查全部签名材料中的 quit 与 offense 证据；
2. 检查完整性与有效性（§5.2）；
3. 按父先子后，把附带的 DATA 与 DRAIN proposal 交给 Vote；
4. 按 term 递增，把附带的终端 QC 交给 P2 第 4 步；
5. 处理消息本身。

消息与计时器同时到达时，先处理消息。

### 6.2 P1：进入与第一块

```text
P1 Enter(s, rb):                                   # 由 Manager 调用，至多一次
  require phase == Assigned 且 ¬closed 且 SR-INIT 不存在
  require rb.anchor == A_s 且 rb.state 是 st(A_s)（Manager 已核对，见 Part 4）
  持久化 SR-INIT 与 SR-ENTRY(0) := (Q^0_s, now)
  安装锚点、执行状态 st(A_s)、授权 AUTH_{s,0} = Π_s、配额、引用下界
  (g, t) := (0, 0); e_v(0,0) := now; engine := Active; phase := Running
  向本区域全体成员发送 rb                           # 进入事件的一部分，不延后
  启动 P4 第 1 步对 (0,0) 的进度检查
  处理 Buffer 中的证据，然后处理其余缓冲消息
  if v == L_s(0,0): 按 P2 第 1 步提议 j = 0，父块 A_s，父证明 Q^0_s
  return Entered(s, e_v(0,0))
```

第一块的高度为 $h(A_s)+1$，物理位置为 1。没有 STATUS、NEWVIEW、ANCHOR 投票或启动等待期。成员只在 Enter 之后投票，使用正常的完整验证与提交计时器。
同一 session 内的恢复（$g>0$）仍然走 P4 的 STATUS、NEWVIEW、等待与 ANCHOR QC，因为它必须考虑本 session 已有的锁。

### 6.3 P2：提议、投票、提交与常规轮换

```text
P2 成员 v 在 session s 中运行；当前视图为 (g, t)。

1) Propose.  仅当 v == L_s(g,t)、engine == Active、g 未冻结时运行。
   DATA 代：对 j = 0 .. m_{g,t} - 1 依次：
     a. 等待持有 j 的 B_parent 与 Q_parent（§3.3）。
     b. 【改】h := h(B_parent) + 1；等待 now >= t(h)（追赶时已满足）。
        若 now >= t(h) + W_exp - T_late：停止本 term 的提议。         # §7.1
     c. 【改】取引用：ρ := 候选的 origin；ξ := 一个已在本地日志中出现至少 T_E 的
        Espresso 头，高度不低于父块的 ξ，第一个 DATA 不低于 q^open_s。
     d. 【改】x := BLD.Build(s, B_parent, st(B_parent), nctx, limits)，nctx 的 origin
        满足 DerivOK 的 V1、V2（含 R-FRESH）与 V-AGE；第一个 DATA 的 origin 高度 >= λ^open_s。
        Build 返回 WAIT 时等待；候选为空交易列表也可以。
     e. B := (β, r, k, h, DATA, x, H(B_parent), ρ, ξ)。
     f. 持久化 SR-PROPOSE(g,t,j) := H(B)；广播
        P = ⟨PROPOSE, g, t, j, B, Q_parent⟩_v 及其验证包。
     g. 把 P 交给第 2 步。
   DRAIN 代：只在 (t, j) = (0, 0) 提议一次：父块 A_{g,0}，父证明为 ANCHOR QC，
     DRAIN 块复制 T_s 的 ρ、ξ，高度 H_s。
   不等待父块提交。同一位置绝不签两个不同的块；重发即重发同一个 P。

2) Vote.  对每个 DATA 或 DRAIN proposal P（term (g_P, t_P)，位置 j_P）运行；
   t_P 可以早于 t；附带历史中的 proposal 也走这一步。
     a. 若 P 或其附带材料含第 g 代的 quit 证据：运行 P4 第 2 步，停止。
     b. 若 g_P < g：保留 P，停止。若 g_P > g：缓冲 P，直到进入第 g_P 代。
     c. 若 g 已冻结，或 engine == Completed：保留 P，停止。
     d. 若缺依赖：保留 P，请求依赖，停止；依赖到达后重做。
     e. 若 P 无效（§5.2）：丢弃，停止。
     f. 若 SR-FRESH(g_P, t_P, j_P) 已存在：停止。重复绝不重启计时器。
     g. 否则：τ_v(P) := now；持久化 SR-FRESH(g_P,t_P,j_P) := H(B)；
        把 P 及其验证包转发给本区域全体成员；
        广播 ⟨DATA-VOTE, g, t_P, j_P, H(B)⟩_v（B 为 DRAIN 时为 DRAIN-VOTE）；
        启动 P 的提交计时器，于 τ_v(P) + 2Δ 到期。

3) Commit.  P 的计时器到期时：
     a. 【改】若 B 与本成员已提交的日志冲突（同一高度已提交另一个块）：
        报警，停止本实例的一切签名（A-BYZ 已被违反），停止。
     b. 提交 B 及其未提交的祖先，按高度从低到高。
     c. 对每个新提交的本 session DATA 块 D：
        - 【改】持久化 SR-FINAL(h(D)) := H(D)；
        - 向本区域全体成员与发布层的收集接口发送 ⟨FINAL, h(D), H(D)⟩_v；
        - 输出 Conf(s, D)。
        提交的 DRAIN 为 T_s 产生 FINAL，从不为自己产生 FINAL。
   只有 Quit、Completion 与 Close 取消计时器；进入新 term 不取消。QC 从不启动计时器。

4) 终端 QC 与轮换.  首次得到 term (g, t') 的有效终端 QC 时运行
   （自己从投票拼出，或在任何消息中看到，包括作为别的 proposal 的 Q_parent）：
     a. 按父先子后，把该 QC 历史中未见过的 proposal 交给第 2 步。
     b. 若 g 已冻结，停止。若该 QC 认证的是高度 H_s 的 DATA 或一个 DRAIN，停止：没有下一个 term。
     c. 把该 QC 及其验证包发给 L_s(g, t'+1)。
     d. 若 t <= t'：进入 (g, t'+1)，e_v(g, t'+1) := now；取消所有更早 term 的进度检查。
   QC 从不把成员移回更早的 term，也不改已记录的进入时刻。

5) Completion.  首次得到满足 §4.5 的 FC_s(T_s) 时运行：
     a. 把完整的终端结果及其依赖发给本区域全体成员一次，并持久化这一发送任务。
     b. 提交 T_s 及其未提交的祖先。
     c. engine := Completed；停止提议、投票、进度检查、term 变化与恢复；取消未到期的计时器。
     d. 输出 Complete(T_s, Res)。Manager 负责发布与把结果发往后继区域（Part 4）。
   Close 从不运行这一步。
```

**说明。**

- **接受时刻** $\tau_v(P)$ 是完整验证与首次投票发生的时刻，不是不完整数据包先到达的时刻。
- **进入时刻** $e_v(0,0)$ 在 P1 设置；$g>0$ 时 $e_v(g,0)$ 在 P4 第 3 步设置；$t>0$ 时 $e_v(g,t)$ 在本协议第 4 步设置。所以进度检查从视图在本地被准入时开始，沉默的新 leader 无法推迟它。
- **高度 $H_s$ 的 proposal 或 QC 不结束 session。** 没有 Quit 或 Close 时，$T_s$ 的计时器提交它；不需要后续块。只有 $T_s$ 的 FINAL 证书使引擎正常结束。

### 6.4 P3：FINAL 的收集与区域结果

```text
P3 收到 ⟨FINAL, h, H(D)⟩_o（o ∈ V_s，签名在 cid_s 下有效）：
  1. 若与 Seen 中 o 在高度 h 上的 FINAL 冲突：交给 §6.7（offense），保留两者。
  2. 记入 FinalSigs[(h, H(D))]。
  3. 若 FinalSigs[(h, H(D))] 的总权重首次 > F_s，且本成员持有 D 的块体并已完整验证 D：
       Σ := FinalSigs[(h, H(D))]
       输出 Result((C_s, D, D, Σ, [D]))                 # D 的直接结果，覆盖其同 session 祖先
       若 D == T_s：运行 P2 第 5 步（Completion）。
```

任何成员都可以拼出 FINAL 证书；Manager 把结果交给 Publisher，并用它推进生产分支 $R$（Part 4）。
FINAL 不绑定 $g,t$，所以同一块的 FINAL 可以跨恢复代合并。
一个仍在 Running 的成员拿到本 session 某个块 $D$ 的有效 FINAL 证书时，也可以提交 $D$ 及其未提交的祖先；这样的提交不再发送 FINAL，因为证书已经存在。

### 6.5 P4：blame、quit 与恢复

```text
P4 成员 v 在 session s 中运行；当前视图为 (g, t)。

1) Blame.  对 p = 1 .. m_{g,t}（DRAIN 代只有 p = 1），在
       e_v(g, t) + (2p + 4)Δ
   运行进度检查；p = 1 的检查在进入时就安排，即使开口尚未确定；
   其余的检查在 m_{g,t} 确定后安排；时刻已过的检查立即运行。
   检查时，若以下全部成立，持久化 SR-BLAME(g,t) 并广播 ⟨BLAME, g, t⟩_v：
     - 当前视图仍是 (g, t)，g 未冻结，engine != Completed；
     - v 在 term (g, t) 中投出的 DATA 或 DRAIN 票少于 p 个位置（ANCHOR 票不计）；
     - v 尚未为 (g, t) 发过 BLAME。

2) Quit.  首次持有当前代 g 的 quit 证据时运行：
     - 冻结 g：停止 g 的一切进度检查、提议、投票与 term 变化；取消 g 中未到期的提交计时器。
     - a_v := now；广播该证据。
     - 保留全部已提交内容；继续收集 g 的投票与 QC，它们可以抬高第 3 步选的入口锁；
       继续处理 FINAL（P3）。
     - 输出 Quit(C_s, g, Q)：Manager 把它提交给 L1 Authority（SubmitQuit），不论类型；
       类型 (b)、(c) 同时是 offense 证据。不等待 L1 处理。

3) Status 与进入.  若 engine == Completed，跳过。
   等到 now >= a_v + Δ，且 Manager 已经 Provide(Authority(g+1, AUTH_{s,g+1}))
   （即 REC(cid_s, g+1) 已最终化，Part 4 §8）。然后：
     - Q_v := v 已知的、来自第 0..g 代的 rank 最高的 QC（DATA、DRAIN 或 ANCHOR）；
       一个都不知道时 Q_v := Q^0_s。持久化 SR-ENTRY(g+1) := (Q_v, now)。
       Q_v 此后不再改变，之后到达的 QC 不更新它。
     - 以未冻结状态进入 (g+1, 0)，e_v(g+1, 0) := now，启动其进度检查。
       A_s、H_s 与委员会保持不变。
     - 向 L_s(g+1, 0) 发送 ⟨STATUS, g+1, 0, Q_v, E⟩_v 及其验证包，E 是第 g 代的 quit 证据。
     - 处理缓冲中第 g+1 代的证据。

4) New-view.  只在 L_s(g+1, 0) 运行。进入后等待 2Δ；若 g+1 仍是当前代且未冻结：
     - Q* := leader 已知的、来自第 0..g 代的 rank 最高的 QC，包括 STATUS 中的 QC；
       一个都不知道时 Q* := Q^0_s。
     - A_{g+1,0} := Q* 所认证的块。h(A_{g+1,0}) < H_s 则第 g+1 代是 DATA 代，否则是 DRAIN 代。
     - 持久化 SR-NEWVIEW(g+1)；广播 ⟨NEWVIEW, g+1, 0, A_{g+1,0}, Q*, E⟩ 及其验证包。
       第 g+1 代只选一个开口。

5) Anchor vote.  收到第 g+1 代的 NEWVIEW 时：
     a. 若其中含第 g+1 代的 quit 证据：运行第 2 步，停止。
     b. 验证 leader、入口证明 E、开口块 A_{g+1,0}、Q* 与依赖；缺数据则请求并等待，不投票；
        无效则丢弃。
     c. 只有以下全部成立才投票：g+1 是当前代且未冻结；v 在第 g+1 代尚未投 ANCHOR 票；
        rank(Q*) >= rank(Q_v)。
     d. 投票：持久化 SR-ANCHOR(g+1)；把 NEWVIEW 转发给全体成员；
        广播 ⟨ANCHOR-VOTE, g+1, 0, H(A_{g+1,0})⟩_v。不启动计时器，不占 fresh 位置。
   权重 > F_s 的匹配 ANCHOR 票构成 ANCHOR QC。L_s(g+1, 0) 得到它后，从 j = 0 开始运行
   P2 第 1 步（DRAIN 代则提议 DRAIN）。每个成员，包括没有投 ANCHOR 票的，
   都把带有效 ANCHOR QC 的 proposal 交给 P2 第 2 步。
```

入口锁检查只约束 ANCHOR 投票。已在本地提交 $T_s$ 的成员仍参与恢复，直到持有 $\mathrm{FC}_s(T_s)$，因为其他成员可能需要它的票。

### 6.6 完整的 quit 证据（同 v4.9）

对已准入的第 $g$ 代，quit 证据是以下三者之一：

- **(a)** 某个已准入 term $(g,t)$ 上权重 $>F_s$ 的 BLAME 证书，附该 term 的准入见证；
- **(b)** $L_s(g,t)$ 在同一 $(g,t,j)$ 上签的两份不同的可准入 DATA 或 DRAIN proposal，附准入与配额见证；
- **(c)** $g>0$ 时，$L_s(g,0)$ 签的两份选择不同开口的有效 NEWVIEW。

超出配额的 DATA、或出现在 DRAIN 代以外的 DRAIN，本身无效，不计入 (b)。
不同开口即使彼此可比也算冲突，因为它们固定了不同的起始高度，同一位置会映射到不同高度。同一开口的不同 QC 不是冲突。
任何已准入 term 的证据都能冻结它所在的代，包括本地尚未到达的 term。旧代的证据作为存档，可以提交给 L1。别的 session 的证据从不冻结本实例。未来代的证据缓冲到进入该代时处理。

### 6.7 offense 的检测

```text
OnSigned(m, o, slot, kind):          # 处理顺序第 1 步对每条签名材料调用
  m' := Seen[(o, slot)]
  if m' 不存在: Seen[(o, slot)] := m; return
  if Conflict(kind, m', m)（Part 2 §5.2 的冲突条件）且 (o, kind, slot) 尚未输出:
      输出 Offense(C_s, o, (kind, m', m, ⊥))
```

PROPOSE 双签与 NEWVIEW 双签同时是 quit 证据 (b)、(c)，P4 第 2 步会经由 SubmitQuit 一并处罚；Manager 对同一违规只提交一次即可（L1 Authority 去重）。
双投、ANCHOR 双投与 FINAL 双签只是 offense，不冻结任何代。
`Seen` 对每个 $(o,\mathit{slot})$ 只保留第一条消息与第一条冲突，存储量与 session 的消息量同阶。

### 6.8 Close

```text
Close(s, reason):
  if SR-CLOSED 存在: return Terminated(s, 已记录的原因)
  持久化 SR-CLOSED := reason; closed := true; phase := Terminated
  取消全部计时器与进度检查；丢弃缓冲中尚未处理的 BFT 动作
  return Terminated(s, reason)
```

关闭之后：

- 不再产生任何签名，包括 FINAL；守卫拒绝一切 BFT 动作，包括已排队的回调。
- 已有的块、证书、证据与已提交日志保留，由 Manager 继续提供数据服务与证据提交。
- 关闭不撤销 $G$，不产生新的 FINAL，也不释放质押。

| reason | 来源 |
| --- | --- |
| COMPLETE | 本实例已运行 Completion，Manager 记录正常终止 |
| FORCE、RESUME、EXPIRED | GFR 的 `SessionClosed` 或 `BranchClosed`（Part 5） |
| KILL | Manager 的本地判死（Part 4 §10） |

### 6.9 Provide

| 输入 | 检查 | 效果 |
| --- | --- | --- |
| `Candidate(parent, x, nctx)` | parent 是本实例当前要扩展的块；来自 Builder | 交给 P2 第 1 步；不会自动触发投票 |
| `References(R_L1, R_E)` | 已最终化的 L1 头与 Espresso 头 | 供 P2 第 1 步选取 $\rho$、$\xi$ |
| `Authority(g, AUTH_{s,g})` | 委员会、权重与 $\Pi_s$ 相同；$\mathsf{REC}(\mathrm{cid}_s,g)$ 已最终化 | 解除 P4 第 3 步的等待 |

与 context 不符、试图修改固定字段、或实例已关闭时返回 `REJECT`。

---

## 7. 时效、追赶与推导时限

### 7.1 【改】时效与节奏

- **节奏。** 诚实 leader 不早于真实时刻 $t(h)$ 提议高度 $h$。新锚点落在过去时（Force 或 RESUME 之后的追赶，或常规后继的锚点已经较旧），$t(h)$ 已过，leader 立即提议，速度只受 $\Delta_i$ 与 Builder 限制。
- **时效。** 诚实 leader 不为 $\mathit{now}\ge t(h)+W_{\rm exp}-T_{\rm late}$ 的高度提议，其中 $T_{\rm late}:=T_{\rm pub}+T_E+\Delta_{\rm ts}$：这样的块来不及在过期前进入 Espresso。
- **投票只用由“不满足”变为“满足”的时间条件。** V-AGE 就是这样的条件（§5.3）：A 投票时满足，B 稍晚收到时也满足。过期时限 $\mathit{now}<t(h)+W_{\rm exp}$ 则相反，会由“满足”变为“不满足”。把它放进投票条件其实不会造成冲突的提交：过了时限，谁都不能再为这个高度投票，另一个块也拿不到证书。但它让 Part I 的证明不能直接套用，又没有收益，因为过期的块由 GFR 的 R-EXP 拒绝。所以过期时限只约束 leader 的提议。
- **R-FRESH 与 V-AGE。** origin 要满足 $t(h)-D_{\rm drift}\le\mathit{ts}(o)\le\mathit{now}-(T_{\rm L1}+\Delta_{\rm ts})$。按节奏出块时 $\mathit{now}\approx t(h)$，所以要求 $D_{\rm drift}>T_{\rm L1}+\Delta_{\rm ts}$ 并留余量；追赶时 $t(h)$ 已过，更容易满足。RESUME 之后的第一块可以取 $L$ 为 origin：$\mathsf M$ 的规程给出 $t(h_{\rm res})\le\mathit{ts}(L)+D_{\rm drift}-\epsilon$，而第一块在 $t(h_{\rm res}+1)\ge\mathit{ts}(L)+T_{\rm prep}\ge\mathit{ts}(L)+M_a$ 之后才提议，$M_a\ge T_{\rm L1}+2\Delta_{\rm ts}$。

节奏与时效只约束诚实 leader，不是有效性条件：违反它们的块仍可能被拜占庭 leader 提出并通过投票，但 GFR 的 R-EXP 会拒绝过期的块（Part 5 §5.5）。R-FRESH 是有效性条件（DerivOK 的 V2），只依赖块自身的时间戳与 origin。

### 7.2 推导时限

以下上界在 $s$ 良态、A-LEAD 成立、候选与引用可得、未被关闭的条件下成立；$\Delta:=\Delta_i$。

- **单个高度。** 设高度 $h$ 在时刻 $\tau_h:=\max\{\tau_{\rm entry},\,t(h)\}$ 变得可以生产，且此时本 session 已提交到 $h-1$（不落后）。由 A-LEAD，$[\tau_h,\tau_h+T_{\rm lead}]$ 内有一个完整的、由诚实 leader 主持的 term；它会在节奏允许时提议 $h$，所有诚实成员在 $\Delta$ 内投票，再过 $2\Delta$ 提交。于是

$$
T_{\rm pos}:=T_{\rm lead}+3\Delta .
$$

- **追赶。** 落后 $n$ 个高度时（锚点在过去），每个诚实 term 至少提交 $\min\{K,\text{剩余配额}\}$ 个高度，所以追上需要至多 $\lceil n/K\rceil\cdot T_{\rm lead}+3\Delta$。
- **整个 session。**

$$
T_{\rm sess}:=\max\{0,\ t(H_s)-\tau_{\rm entry}\}+\lceil\mathrm{TB}/K\rceil\cdot T_{\rm lead}+4\Delta ,
$$

最后一个 $\Delta$ 用于收集 $T_s$ 的 FINAL 证书。这是保守的上界：节奏只在 $t(h)$ 未到时起作用，追赶只在 $t(h)$ 已过时起作用，两者的时间不重叠。

$T_{\rm lead}$ 已包含恢复代切换所需的时间（等待 REC 最终化约 $T_{\rm L1}$），因为 A-LEAD 是按“完整落在区间内的诚实 term”陈述的。这两个式子供 Part 0 的 DG6 使用，精确常数待 Part 0 的活性推导核对。

---

## 8. 保证的证明

### 8.1 G-PROG-1（区域一致）

**结论。** 若 $s$ 良态，两个诚实成员不会在同一高度提交不同的块。

**证明梗概。** 本稿的投票、锁、提交计时器与恢复规则与 Part II v4.9 相同，后者是 Part I 加上权重门槛、配额、DRAIN 与 ReadyBundle 的适配，其区域一致性在 Part II v4.9 §9.3 证明。本稿的修改只有三类，逐一说明它们不破坏那个证明：

1. **更严的有效性（V-DER、V-REF、V-AGE）。** Part I 的关键引理是：若诚实成员 $r$ 在 $\tau$ 为 $B$ 投票并在 $\tau+2\Delta$ 提交，则其他诚实成员在 $\tau+\Delta$ 之前收到 $B$ 并为它投票，所以到 $\tau+2\Delta$ 每个诚实成员都持有 $B$ 的证书，恢复时的入口锁不低于它。这需要“$r$ 在 $\tau$ 认为有效的块，其他诚实成员在 $\tau+\Delta$ 之前也认为有效”。V-DER 与 V-REF 读的是已最终化的数据；V-AGE 保证 $r$ 投票时，这些数据已在所有诚实成员的日志中（§5.3）；V-AGE 本身随时间只会由不满足变为满足。所以引理照旧成立。
   反过来，若投票条件依赖接收方自己的进度，例如只看“我的 L1 节点已显示 origin 最终化”而不检查新旧，引理就会失效：某个诚实成员可能暂时验证不了而不投票，$B$ 得不到证书，恢复后同一高度可以换成另一个块。
2. **基础证明的来源（RESUME 的 $Q^0$）。** $Q^0_s$ 只作为 rank 最低的入口锁与第一块的父证明，与 v4.9 的用法相同；它从哪里来不影响锁的比较。
3. **FINAL 槽位与冲突报警。** 只减少签名。

〔良态（含 $\Delta_i$）、A-CRYPTO〕

### 8.2 G-PROG-2（FINAL 可靠）

**结论。** 若 A-BYZ 对 $s$ 成立，则任何在 $\mathrm{cid}_s$ 下、签名都在 $\tau_s^{\rm close}$ 之前产生、总权重 $>F_s$ 的 FINAL 证书所签的 DATA 块 $D$，都曾被某个诚实成员在完整验证之后提交。

**证明。** 由 A-BYZ，在 $\tau_s^{\rm close}$ 之前被腐化的成员总权重不超过 $F_s$；证书权重 $>F_s$，所以至少一个签名者在签名时诚实。诚实成员只在 P2 第 3 步为自己提交的块签 FINAL，而提交之前它为该块或其后代投过票，投票之前完成了 §5.2 的完整验证，包括对所需祖先的 V-DER。所以 $D$ 及其同 session 祖先在各自父块上满足 DerivOK（原生有效、deposit 前缀、drift 与 R-FRESH）、配额与引用下界。
签名时间的限定来自 Part 0 定义 1.2：$\tau_s^{\rm close}$ 之后被腐化的密钥可以补签 FINAL，但这样的签名不能出现在任何能把 $s$ 的块加入 $G$ 的 Espresso 块中。Manager 用直接结果提前开出常规后继时，若用到这样的伪证书，后继的块仍需前驱的末块真正进入 $G$ 才能被接纳（Part 5 §5.5），所以不影响 $G$。
〔A-BYZ、A-CRYPTO；依赖 G-MGR-1（context 真实）、G-BLD-1、G-BLD-2〕

### 8.3 G-PROG-3（生产约束）

P2 第 1 步在提议前检查高度（配额）、节奏、时效、引用下界，并通过 Builder 保证 DerivOK；第 2 步的 V-POS、V-REF、V-DER 拒绝不满足高度、引用与派生一致约束的块；第 3 步只为自己提交、因而投过票的块签 FINAL。所以诚实成员只为满足高度、引用与派生一致约束的块提议、投票与签 FINAL，并且只在时效之内提议。投票与 FINAL 不检查时效，理由见 §7.1 与 §8.1。

### 8.4 G-PROG-4（隔离与关闭）

每个 session 有独立的实例状态，消息按其签名中的 cid 分派，别的 session 的证据从不冻结本实例。
Close 先持久化 SR-CLOSED，再置位守卫；每个 BFT 动作都检查守卫，所以关闭之后没有新签名，包括崩溃重启之后。
SR-INIT 保证至多一次初次 Enter。

### 8.5 G-PROG-5（诚实不留证据）

诚实成员在发出 PROPOSE、DATA/DRAIN-VOTE、NEWVIEW、ANCHOR-VOTE、FINAL 之前，先查并持久化对应的槽位记录；已有记录时只重发同一内容。所以它在同一 cid、同一槽位不会签两份冲突消息，崩溃重启也不会。
FINAL 的冲突（同一高度两个块）还需要排除“诚实成员合法地提交了两个不同的块”：P2 第 3 步 a 在提交前检查与已提交日志的冲突，冲突时停止签名。
它输出的 quit 证据都是它自己完整验证过的（§6.6），所以在 L1 Authority 的检查下有效（Part 2 §5.5）。〔持久化存储〕

### 8.6 G-PROG-6（同步进入）

设第一个诚实成员在 $e$ 时 Enter。P1 在进入事件中把完整的 ReadyBundle 发给全体成员；由 $\Delta_i$，每个诚实成员在 $e+\Delta_i$ 之前收到并完成处理（ReadyBundle 携带可直接使用的完整状态，处理时间计入 $\Delta_i$，Part II v4.9 §7.4）。Manager 核对 handoff 与关闭标记（Part 4 §7.4）之后调用 Enter。
前提：收到者的 Manager 没有关闭 $s$。若某个诚实成员的 Manager 已经关闭 $s$（例如先看到了 Force），它不进入，这正是关闭的语义。〔良态、$\Delta_i$〕

### 8.7 G-PROG-7（区域活性）

在 §7.2 的条件下，Part I 的活性论证（诚实 leader 的 term 中每个位置在 $(2p+4)\Delta$ 内得到投票，失职的 leader 引发 BLAME 证书与恢复）照搬到加权门槛与有限配额；A-LEAD 保证诚实 term 出现的频率；P3 保证 FINAL 证书在提交后 $\Delta$ 内形成。DerivOK 不影响活性：诚实 leader 只提议通过 `ValidateCandidate` 的候选，而候选在 Builder 处总能构造出来（G-BLD-1；空交易列表的候选总是满足 V3、V4；V1、V2 与 V-AGE 只要求有合适的 origin，由 $D_{\rm drift}$ 的余量保证存在，§7.1）。
leader 失职时，BLAME 证书在 $(2m+4)\Delta$ 内形成，即 quit 证据。

### 8.8 G-PROG-8（证据输出）

证据检查是处理顺序的第 1 步，在任何验证之前进行；检测到冲突或 quit 证据时，同一处理事件内输出 Offense 或 Quit。诚实成员转发收到的 proposal 与 NEWVIEW，所以任何诚实成员看到的证据，其他诚实成员在 $\Delta_i$ 内也能看到。〔$\Delta_i$〕

---

## 9. 与 Part II v4.9 的差异

| 项目 | v4.9 | 本稿 |
| --- | --- | --- |
| L1 引用 $\rho$ | 独立选择的已最终化 L1 块，沿祖先不减 | 等于原生 origin（R30） |
| 投票的有效性 | 完整验证，含应用有效性 | 另加 DerivOK（V-DER，R25），含 R-FRESH |
| 引用的新旧 | 无要求 | 投票者只接受足够旧的 L1 origin 与 Espresso 引用（V-AGE，§5.3），所有诚实成员结论相同 |
| 时效与节奏 | 无 | leader 不早于 $t(h)$ 提议，也不为来不及在过期前进入 Espresso 的高度提议；投票不检查过期时限 |
| FINAL | 无槽位记录 | SR-FINAL；与已提交日志冲突时报警停止 |
| 关闭原因 | Completion、Force-close | COMPLETE、FORCE、EXPIRED、RESUME、KILL |
| 锚点的基础证明 | 前驱的 FINAL 证据或 genesis | 另有 RESUME 的派生证据（DER 锚点） |
| 输出事件 | 完成事件 | Conf、Result、Complete、Quit、Offense |

---

## 10. 待定问题

1. **Espresso 读取不可用时。** V-AGE 对 Espresso 引用依赖 X-L2，而 Part 0 中 Espresso 的 X-L2 只在读取可用期间成立。部分诚实成员读不到 Espresso 时，它们只是暂不投票（影响活性）；若某个成员已经投票而其他成员长期读不到，V-AGE 的论证不成立。目前按“读取可用”处理，是否需要更强的假设待定。
2. **$T_{\rm pos}$、$T_{\rm sess}$ 的常数。** §7.2 是保守上界，与 Part 0 DG6 的 $T_{\rm live}$ 推导一起核对。
3. **$D_{\rm drift}$ 的余量。** 按节奏出块时 origin 至少落后 $T_{\rm L1}+\Delta_{\rm ts}$，所以要求 $D_{\rm drift}$ 明显大于它；需要用 MegaETH 的实际参数确认。
4. **mini-block 与 DATA 的对应。** 本稿把一个 DATA 视为一个原生 L2 块，mini-block 只是其中的划分；快速确认是否按 mini-block 粒度提供，属于实现范围。
