# 区域 BFT 的形式化：假设与要求（工作稿 v0.1）

日期：2026-10-30。
状态：草稿。本文不修改协议正文。

**本文做什么。**
- [Part 1 v0.3](part1_module_interfaces_zh_v0_3.md) §4.4 给出了 Session Program 的接口和保证 G-PROG-1 至 8，它们是按当前的同步实现（[Part 3 v0.1](../formalization_v5/part3_session_program_zh_v0_1.md)，Sync HotStuff）写的。
- 本文把这一层抽象出来，回答两个问题：
  - 区域 BFT **依赖哪些假设**（取自 [Part 0 v0.8](part0_model_goals_zh_v0_8.md)，加上少量新增）；
  - 区域 BFT **必须满足哪些要求**，与具体用哪种协议无关。
- 任何实现，包括现在的 Part 3，或者换成 Simplex 一类的部分同步协议，都可以逐条对照本文核对。

**本文吸收的新结论**（来自本目录的讨论稿）：

- N1 是对输出的**有效性条件**，不是对出块速度的要求（[b_t 与轮换空档](block_time_and_rotation_gaps_zh.md)）。
- 出块变慢用**空块补位**处理，链上时间要**跟随全局时钟**（同上，以及后续讨论）。
- 区域内换 leader **不等 L1**（同上 §4 之后的讨论）。
- 块级 L1 地板与原生 origin 规则冲突（[恢复时的时间假设](recovery_timing_assumptions_zh.md) 问题 1）。

---

## 0. 定位

**区域 BFT 是什么。** 区域 BFT 就是一个 session 内的状态机复制（SMR）实例：

- 委员会、权重与门槛固定；
- 从固定锚点 $A_s$ 出发，生产至多 TB 个高度；
- 输出三样东西：已提交的块（快速确认）、FINAL 证书（区域结果），以及 quit 与 offense 证据。

**它不负责什么：**

| 事情 | 由谁负责 |
| --- | --- |
| 选委员会与 leader 名单 | L1 Authority 的 Plan、Leader |
| 下一个高度由谁负责、全局最终 | GFR |
| 何时开始、何时关闭、跨 session 交接 | Session Manager |
| 发布、交付、派生 | Publisher、Batcher、Derivation |

**本文的结构：**

- §1 给出模型：一个 session 实例的输入、输出和有效性谓词；
- §2 列出假设；
- §3 列出要求；
- §4 拿当前的 Part 3 逐条核对；
- §5 汇总需要的改动。

---

## 1. 模型：一个 session 实例

### 1.1 固定参数

由 handoff 给定，实例存续期间不变（Part 1 §2.3、§2.7）：

| 参数 | 含义 |
| --- | --- |
| $C_s=(\Pi_s,A_s)$，$\mathrm{cid}_s$ | context 及其标识；所有区域签名都绑定 $\mathrm{cid}_s$ |
| $V_s$，$w_o$，$W_s$，$F_s$ | 委员会、权重、总权重、故障门槛 |
| $A_s$，$Q^0_s$ | 锚点与它的承诺证据 |
| $H_s=h(A_s)+\mathrm{TB}$ | 本 session 能生产的最高高度 |
| $q^{\rm open}$ | Espresso 引用的下界 |
| $\Delta_i$ | 区域内的消息时限 |

**全局常数**（Part 0、Part 1 §2.2）：$b_t$、$t_{\rm gen}$、$h_{\rm gen}$、$D_{\rm drift}$、$W_{\rm exp}$、$T_{\rm L1}$、$T_E$、$\Delta_{\rm ts}$，以及本文新增的时钟误差 $\epsilon$（§2）。

### 1.2 输入

| 输入 | 来源 |
| --- | --- |
| 候选块 | Block Builder（Part 1 §4.1） |
| 已最终化的 L1 与 Espresso 引用 | Session Manager 转来 |
| leader 名单 | L1 Authority 的 Leader 函数 |
| 区域内的签名消息 | 同 session 的其他成员 |
| 本地时钟 $\mathit{now}$ | 全局时钟（§2 的 H-CLK） |

### 1.3 输出

- **已提交链：** 每个诚实成员 $v$ 的已提交链 $\mathrm{Commit}_v(\tau)$，是从 $A_s$ 出发、哈希相连的块序列。每提交一块，输出一次 $\mathsf{Conf}_v(s,B)$。
- **区域结果：** $\mathit{Res}=(C,B,D,\Sigma,\pi)$，其中 $\Sigma$ 是权重大于 $F_s$ 的 FINAL 签名（Part 1 §2.6）。
- **末块结果：** $\mathsf{Complete}(T_s,\mathit{Res})$，$T_s$ 是高度 $H_s$ 的块。
- **证据：** quit 证据与 offense 证据。

### 1.4 块与有效性谓词

一个块对应一个原生 L2 块，即一个 EVM 块：
- 高度 $h$、时间戳 $t(h)$；
- L1 origin $o$、前缀（L1 attributes、deposit、系统交易）、普通交易；
- 引用 $\rho=o$ 与 $\xi$。

**补位块**是普通交易为空的块，前缀照常包含。它不是一种新的块类型，只是"普通交易为空"的特例；Builder 已经允许普通交易为空（Part 1 §4.1.2）。

**公开有效性 $\mathrm{Valid}_s(P,B)$：** 任何人只凭已最终化的 L1 就能检查，结论确定，且不随时间改变。

| 条件 | 内容 |
| --- | --- |
| V0 结构 | $h(B)=h(P)+1\le H_s$；父链接正确；签名绑定 $\mathrm{cid}_s$ |
| V1 时间戳表（N1） | $t(B)=t_{\rm gen}+(h(B)-h_{\rm gen})\,b_t$ |
| V2 派生一致（DerivOK） | 在 $P$ 上满足 Part 1 §3.2 的四条：origin 连续，drift（含 R-FRESH），前缀等于 deposit 规则构造的前缀，执行有效 |
| V3 引用下界 | $\xi(B)$ 指向已最终化的 Espresso 块，沿祖先不减，第一块不低于 $q^{\rm open}$ |

**本地可接受性 $\mathrm{Acc}_v(B,\mathit{now})$：** 成员投票时按自己的时钟与视图检查。

| 条件 | 内容 | 需要它的协议 |
| --- | --- | --- |
| W1 不超前 | $t(B)\le\mathit{now}+\epsilon$ | 所有实现 |
| W2 引用足够旧（V-AGE） | $\mathit{ts}(o(B))\le\mathit{now}-(T_{\rm L1}+\Delta_{\rm ts})$；$\xi$ 已在本地出现至少 $T_E$ | 安全依赖"我能验证的，别人在 $\Delta_i$ 内也能验证"的同步协议 |

**单调性要求：** $\mathrm{Acc}$ 里的每一条，都只能随时间从不满足变为满足，不能反过来。

- W1、W2 都满足这一点。
- 不满足这一点的条件，只能约束诚实 leader，不能放进投票。例如"不能太旧"、"未过期"。理由见 Part 3 §7.1：放进投票，会让"我投过的块，别人稍晚也会投"不再成立。

---

## 2. 假设

| 编号 | 假设 | 出处 | 用于 |
| --- | --- | --- | --- |
| H-CRYPTO | 签名不可伪造，哈希抗碰撞 | Part 0 A-CRYPTO、A-PKI | 全部安全要求 |
| H-BYZ | 拜占庭成员的总权重不超过 $F_s$。同步实现要求 $F_s<W_s/2$；部分同步实现要求 $F_s<W_s/3$ | Part 0 A-BYZ、定义 1.1 | RS3–RS6、RS9 |
| H-GOOD | session 良态：拜占庭与离线成员的总权重不超过 $F_s$ | Part 0 定义 1.4 | RS1、RS2、RT5、全部活性要求 |
| H-NET | 良态 session 的诚实成员之间，消息在 $\Delta_i$ 内送达并处理 | Part 0 §1.5 | 同步实现：RS1 与活性；部分同步实现：只用于活性（GST 之后） |
| H-CLK | 所有参与方读同一个全局时钟，误差不超过 $\epsilon$ | Part 0 §1.2（误差原本并入 $\Delta$，本文单列为 $\epsilon$） | RS6、RT3、RT5 |
| H-VIS | 已最终化的 L1 块在 $T_{\rm L1}$ 内、Espresso 块在 $T_E$ 内，出现在所有诚实成员的视图中 | Part 0 L1-L3、Espresso 的 X-L2 | 同步实现的 RS1（经 W2）；所有实现的活性 |
| H-TS | 链时间戳与提出时刻相差不超过 $\Delta_{\rm ts}$ | Part 0 A-TS | 只用于 W2 |
| H-STORE | 诚实成员先持久化槽位记录，再签名；崩溃后不丢失 | Part 0 §1.4 | RS8 |
| H-INPUT | 候选块、引用与锚点状态可以得到 | Part 1 G-BLD-1、2、3，G-MGR-6，$T_{\rm ready}$ | 活性 |
| H-LEAD | 诚实 leader 出现得足够频繁 | Part 0 A-LEAD | 活性；leader 按权重随机轮换时可以按概率推出 |

**明确不需要的假设：**

| 不需要 | 说明 |
| --- | --- |
| L1 活性 | session 内的任何步骤都不等 L1。证据异步提交 L1，只用于罚没。当前 Part 3 不满足，见 §4 |
| Espresso 可用 | 只用于取引用。Espresso 停机时，区域可以用已有的引用继续，直到引用下界卡住 |
| 跨区域网络 | 只影响交接（§3.5），不影响 session 内的安全与活性 |
| $\mathsf M$、$\mathsf C$、A-GOV | 与区域 BFT 无关 |
| 固定的出块速度 | 出块快慢不受约束；N1 只约束时间戳（V1） |

---

## 3. 要求

**编号约定：** RS 为安全；RT 为时间；RL 为活性；RH 为交接；RA 为问责。每条在方括号里注明所需假设。

### 3.1 安全

- **RS1 区域一致。** $s$ 良态时，两个诚实成员不会在同一高度提交不同的块。〔H-GOOD、H-CRYPTO；同步实现另需 H-NET 与 H-VIS（经 W2）〕
  - 对应 Part 0 S4(a)。
- **RS2 只追加。** 每个诚实成员的已提交链从 $A_s$ 出发，只在末尾追加，不回滚。〔H-STORE〕
- **RS3 FINAL 可靠。** 任何权重大于 $F_s$ 的 FINAL 证书所签的块，都曾被某个诚实成员在完整验证之后提交。〔H-BYZ、H-CRYPTO〕
  - 证书里至少有一个诚实签名，而诚实成员只为自己提交过的块签 FINAL。
  - 对应 G-PROG-2，是 GFR 只做签名检查（CheckFINAL）就可以接纳块的依据。
- **RS4 FINAL 唯一。** $s$ 良态时，同一高度至多一个块有 FINAL 证书。〔由 RS1、RS3 推出〕
- **RS5 外部有效。** 任何带证书（QC 或 FINAL）的块 $B$ 及其同 session 祖先，都满足 $\mathrm{Valid}_s$，即 V0–V3。〔H-BYZ、H-CRYPTO〕
  - 特别地，B 满足 N1 与 deposit 规则：时间戳由高度决定，前缀包含应有的 deposit。
  - 对应 Part 0 S3(a)、S3(b)，以及 L2-L1 的前提"派生一致投票"。
- **RS6 不超前。** 任何带证书的块 $B$ 满足 $t(B)\le\tau_{\rm cert}(B)+\epsilon$，其中 $\tau_{\rm cert}$ 是证书形成的时刻。〔H-BYZ、H-CLK〕
  - 证书里至少有一个诚实投票者，它投票时检查过 W1。
  - 这一条保证链上时间不会被拜占庭 leader 拨快。
- **RS7 关闭与隔离。** 不同 session 的实例不共享核心状态；Close 之后不再产生任何签名；每个实例至多一次 Enter。
  - S6 的"session 关闭后不再影响 $G$"依赖它。
- **RS8 诚实不留证据。** 诚实成员不会在同一槽位签两份冲突的消息，所以不存在针对诚实密钥的有效 offense 证据。〔H-STORE〕
  - 对应 Part 0 S5(a)。
- **RS9 签名绑定。** 所有签名都绑定 $\mathrm{cid}_s$ 与槽位，不能在别的 session 或别的槽位重放。〔H-CRYPTO〕

### 3.2 时间

- **RT1 时间戳由高度决定。** 这一条就是 V1，已包含在 RS5 中。
  - 它约束的是输出，不是速度：协议不需要也不承诺固定的出块间隔。
- **RT2 不超前。** 即 RS6。
- **RT3 时间跟随（活性）。**
  - **滞后的定义：** $\lambda_v(\tau)=\max\{0,\ \tau-t(\text{$v$ 已提交链末端的下一高度})\}$。
  - **要求：** $s$ 良态、H-INPUT 成立、实例未关闭时：
    1. **稳态：** 诚实 leader 在 $t(h)$ 提议 $h$，$\lambda_v\le b_t+T_{\rm commit}$。
    2. **中断之后：** 中断可以是换 leader、Enter 晚于锚点时间，或其他停顿。从中断结束起，$T_{\rm catch}$ 内回到稳态。$T_{\rm catch}$ **不随空档长度增长**。
  - **怎样做到：** 用补位。诚实 leader 的下一次提议先补齐空出来的高度，再接一个落在当前 slot 的普通块，一轮共识就能完成。
- **RT4 补位不带用户交易（诚实 leader 规则）。** 诚实 leader 只在当前 slot 放普通交易；补的过去高度都是补位块。
  - 这一条只能约束诚实 leader，不能放进投票：一个块"是否属于过去"会随时间从否变为是，不满足单调性。
  - 目的：避免时间戳在过去的块里装用户交易，从而给 leader 事后选择的机会（MEV）。
- **RT5 节奏与计时器相容。** $s$ 良态时，诚实 leader 主持的 term 不会产生 quit 证据，包括它为等待 $t(h)$ 而暂缓提议的时候。〔H-GOOD、H-NET、H-CLK〕

### 3.3 活性

**前提：** 以下各条都在 H-GOOD、H-NET、H-INPUT、H-LEAD 成立、实例未关闭的条件下陈述。

- **RL1 同步进入。** 第一个诚实成员 Enter 之后 $\Delta_i$ 内，所有诚实成员都已进入。对应 G-PROG-6。
- **RL2 单块时限。** 每个高度在可以生产之后 $T_{\rm pos}$ 内被提交。"可以生产"指 $\max\{\tau_{\rm entry},t(h)\}$ 已到。
- **RL3 末块结果。** $T_s$ 的区域结果在 $T_{\rm sess}$ 内形成。
- **RL4 换 leader 不依赖 L1。**
  - **时限：** 沉默或双签的 leader，在 $T_{\rm vc}$ 内被替换。$T_{\rm vc}$ 只由若干 $\Delta_i$（或超时参数）构成，不含 $T_{\rm L1}$。
  - **排除双签者：** 证明双签过的成员，最迟从下一个 session 起不再当 leader。这是延迟排除，与以太坊的做法相同。
  - **证据上链：** 证据仍要提交 L1，但不等它最终化。
- **RL5 交易包含。** 诚实 leader 在容量之内包含待处理集合中所有持续有效的交易。与 G-BLD-3 一起，构成 Part 0 L-tx 的区域部分。
- **RL6 证据输出。** 诚实成员观察到的 quit 或 offense 证据，在 $\Delta_i$ 内交给 Manager。对应 G-PROG-8。

### 3.4 交接

| 编号 | 要求 | 用途 |
| --- | --- | --- |
| RH1 锚点 | 第一块的父块是 $A_s$，入口锁为 $Q^0_s$；不继承前驱的锁、投票与计时器 | 交接不引入前驱的状态 |
| RH2 配额 | 不提交高于 $H_s$ 的块 | 后继从 $H_s$ 接棒；S6 的引理 C |
| RH3 末块及时 | $T_s$ 提议之后 $T_{\rm fin}$ 内形成 $\mathsf{Complete}(T_s,\mathit{Res})$；同步实现中 $T_{\rm fin}\approx4\Delta_i$ | 常规交接的空档 $g\approx T_{\rm fin}+\Delta_{\rm cross}$ |
| RH4 末块唯一 | $s$ 良态时 $T_s$ 唯一（由 RS4 推出） | 后继的锚点确定 |
| RH5 末块可以补位结束 | 剩余配额全部用补位块填满时，末块也照常形成结果 | 空档长于剩余配额时，session 正常结束、后继接棒 |

### 3.5 问责

- **RA1 证据可在 L1 上验证。** quit 与 offense 证据只需相对于所声称的 context 就能验证，不依赖提交者是否诚实（Part 2 §5.5 的相对验证）。
- **RA2 证据的含义。** 有效的 offense 证据，意味着对应密钥在同一槽位签了两份冲突的消息。有效的 quit 证据，意味着某个 term 的 leader 失职：要么是权重大于 $F_s$ 的 BLAME，要么是双签。
- **RA3 准入证明不依赖 L1 记录。**
  - 第 $g$ 代证据的准入证明，取上一代的 quit 证据或本代的 ANCHOR QC，不取 L1 上的 REC。
  - 这是 RL4 的配套要求：换代不等 L1，证据就不能依赖 L1 上的换代记录。

### 3.6 与 Part 0 目标的对应

| Part 0 目标 | 用到的区域要求 |
| --- | --- |
| S3 有效性 | RS3、RS5 |
| S4(a) 快速确认一致 | RS1 |
| S4(b) 快速确认与 $G$ 一致 | RS1、RS3，加上 GFR 的 CheckFINAL |
| S5 问责 | RS8、RS9、RA1、RA2 |
| S6 质押安全 | RS7、RH2，加上 GFR 的 R-EXP 与 R-FRESH |
| L2-L1 进入 $G$ 的块进入 $D$ | RS5（DerivOK） |
| 时间跟随（Part 0 待补，见 §5） | RS6、RT3、RT4 |
| DG6 正常运行的活性 | RL1–RL5、RT3、RH3 |
| DG7 区域故障 | RL4（区域内）；超出区域能力时由 Force 处理 |

---

## 4. 对当前实例的核对：Part 3 v0.1（Sync HotStuff）

| 要求 | 是否满足 | 说明 |
| --- | --- | --- |
| RS1 | ✓ | G-PROG-1，依赖 H-NET 与 W2 |
| RS2、RS7、RS8、RS9 | ✓ | G-PROG-4、5；签名绑定 $\mathrm{cid}_s$ |
| RS3 | ✓ | G-PROG-2 |
| RS5 | ✓，有一处问题 | V0–V3 都检查。但 V-REF 还多了一条块级 L1 地板：第一块 $\lambda\ge\lambda^{\rm open}$。区域故障后发的 Force 因此开不出第一块（恢复稿问题 1） |
| **RS6 不超前** | **✗** | 节奏规则只约束诚实 leader，投票不检查 W1。拜占庭 leader 可以提前提议未来的高度：V-AGE 加 drift 只把它限制在约 $D_{\rm drift}-T_{\rm L1}$（约 15 分钟）之内 |
| **RT3 时间跟随** | **部分** | 允许快于 $b_t$ 追赶，但每个高度要一个位置，追赶时间与空档长度成正比；没有补位提议 |
| RT4 | ✗ 未规定 | Builder 允许空块，但没有"追赶期只出空块"的规则 |
| **RT5 计时器相容** | **✗** | P4 从 term 进入时刻起，在 $(2j+4)\Delta_i$ 检查第 $j$ 个位置。$b_t>2\Delta_i$ 时，按节奏等待的诚实 leader 会被 BLAME。$b_t=1$ 秒、$\Delta_i=50$ ms 时几乎每个 term 都会这样 |
| RL1–RL3、RL5、RL6 | ✓ | G-PROG-6、7、8；$T_{\rm pos}$、$T_{\rm sess}$ 见 Part 3 §7.2 |
| **RL4 换 leader 不依赖 L1** | **✗** | 进入新一代要等 $\mathsf{REC}(\mathrm{cid}_s,g+1)$ 在 L1 最终化（Part 3 P4 第 3 步），约 $T_{\rm L1}$，即 13–19 分钟 |
| RH1–RH4 | ✓ | Part 1 §4.4、§4.3.3 |
| RH5 | 未写明 | 配额用补位块填满时的结束方式没有单独说明 |
| RA1、RA2 | ✓ | Part 2 §5.5 |
| **RA3** | **✗** | 第 $g>0$ 代的准入证明就是合约里的 REC（Part 2 §5.5 的 GenAdmitted） |

---

## 5. 需要的改动

| 编号 | 改动 | 满足 | 改哪里 |
| --- | --- | --- | --- |
| C1 | 投票条件增加 W1：$t(B)\le\mathit{now}+\epsilon$ | RS6 | Part 3 V 系列检查；Part 1 §4.4 生产约束 |
| C2 | 进度检查改从 $\max\{e_v(g,t),\ t(h_j)\}$ 起算，并改为按提议计数（不按高度） | RT5 | Part 3 P4 第 1 步 |
| C3 | 换代不等 REC：$\mathsf{Ban}_{s,g}:=\mathsf{Ban}(\ell(\Pi_s))$；只等 $\Delta_i$ 让锁传播；证据异步提交 L1 | RL4 | Part 3 P4 第 3 步；Part 2 §7.3 的 Auth |
| C4 | 合约接受上一代 quit 证据或 ANCHOR QC 作为准入证明；REC 只作记录 | RA3 | Part 2 §5.5 |
| C5 | 补位：一个提议可携带 $k\ge0$ 个补位块加至多一个普通块，一次投票全部认证；诚实 leader 补到当前 slot；补位块占配额 | RT3、RH5 | Part 3 块格式、P2；Part 1 §2.4 |
| C6 | 诚实 leader 在过去的高度只出补位块 | RT4 | Part 3 P2；Part 1 §4.1 Builder |
| C7 | 删除块级 L1 地板，地板只留给 Plan 的读取高度 | RS5（恢复 Force 的可用性） | Part 3 V-REF；Part 1 §2.7、§4.4 |
| C8 | Part 0 增加"时间跟随"性质（RS6、RT3）与时钟误差 $\epsilon$ | — | Part 0 §1.2、§3.2 |

**各改动的安全影响：**

- C1 是新增的单调条件，只会让诚实成员少投票，不影响 RS1 的证明。
- C2、C3 只改变活性条件。Part I 的安全证明只用到同步网络和锁传播（等 $\Delta_i$），没有用到 L1。
- C5 的补位块是合法的原生块，满足 V0–V3，所以 RS5 不变。
- C7 不影响 S6：Part 2 §8.7 只用到 Plan 的读取高度。

---

## 6. 待定问题

1. **部分同步实现。** 换成 Simplex 一类协议时：
   - 本文的要求不变；
   - 假设要变：H-BYZ 的门槛变为 $F_s<W_s/3$；H-NET 只用于活性；W2 不再是 RS1 的前提；RL4 天然满足（超时之后在区域内用 skip 证书换 view）。
2. **按 mini-block 确认。** 若区域一致性按 mini-block 提供（Part 3 §10 第 4 条），本文的"块"要拆成两层：
   - RS1、RS3 作用于 mini-block；
   - V1、V2 与补位作用于 EVM 块；
   - 还要补上 EVM 块的封口规则。
3. **$\epsilon$ 的取值。** 由节点时钟同步的实际精度决定；它同时限制链上时间最多超前多少。
4. **$T_{\rm commit}$、$T_{\rm catch}$、$T_{\rm vc}$、$T_{\rm fin}$ 的表达式。** 由具体实现推导，与 Part 0 DG6 的 $T_{\rm live}$ 一起核对。
5. **按会话固定 leader 名单的代价。** 采用 C3 后，同一 session 内被证明双签的成员可能再次被抽为 leader。需要估算一个 session 内最多损失多少，并确认 H-LEAD 的概率形式仍然成立。
