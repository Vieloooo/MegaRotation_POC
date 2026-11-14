# 有界时钟偏差 $\Delta_G$：模型替换与各模块修改（v0.1）

日期：2026-11-07。对象：本目录当前规格（00–08 v0.1，09、10 v0.2）。本文只做分析，不改动规格原文；安全问题另见 [时钟偏差安全审查 v0.1](clock_skew_review_zh_v0_1.md)。

出发点是“所有参与方之间的时钟偏差不超过 $\Delta_G$”。为使这个模型能与 L1、Espresso 时间戳和真实时间预算衔接，本文另加了几项条件和修法，一律标 〔补充〕，待你确认。

## 0. 结论

1. **安全性不变。** BFT-1、BFT-3、BFT-4、CheckFINAL、CheckEvidence、G2、GFR 重放与 Derivation 都不读本地时钟；$G$ 仍是公开 L1 与 Espresso 日志的函数。论证见审查 §1。
2. **读本地时钟的只有 replica 与它的 Session Manager。** Publisher、Batcher、Certifier、Inbox、DA、GFR 读取者与 Derivation 不读协议时钟，$\Delta_G$ 对它们没有约束作用；它们的时限都是真实时间的活性预算。
3. **需要修改三类内容。**
   - 读时钟的规则改读本地时钟（02、03）。
   - 以真实时间表述的时间性质要放宽：BFT-7 的真实时间读法、G1.2、G1.3、G3.2。
   - 活性预算要加入偏差项：00 §3.4、02 §4.6、04 PUB-4 推论、09 §3.5 与 §4.4、01 §3.10。
4. **Accept 的时间条件必须改。** 现行下界是 $\mathit{ts}^*(B.h)\le\tau_{\rm recv}$，而且重试保留首次接收时刻。两者合在一起，慢时钟的诚实 replica 会永久拒绝快时钟诚实 leader 的 proposal，不需要任何敌手（审查 K1）。
5. **按 10 的参数核算：**
   - $T_{\rm prep}$ 的下界余量为 0，只要 $\delta_G>0$ 就要调高 $T_{\rm prep}$。
   - $T_{\rm grace}$ 要求 $\delta_G<45$ s，$T_{\rm dep}$ 要求 $\delta_G\le48$ s。
   - 正常交接示例要做到无停顿，需要 $\Delta_G<0.5$ s（§3.8）。

## 1. 时钟模型

### 1.1 现行假设

[00 §2.3](00_arch_zh_v0_1.md) 第 292 行：每个 honest replica 及其本地 Session Manager 在真实时刻 $\tau$ 读到的协议时钟就等于 $\tau$，用于 block 时间检查、窗口开始和停止。v7、v8 的 Regional BFT 抽象把漂移交给时钟模块处理。v6 的 A3 则用 $|\mathit{now}-\tau|\le\epsilon$ 表述（[v6 Regional BFT v0.2](../formalization_v6/regional_bft_abstraction_zh_v0_2.md)）。本文沿用 v6 的处理方式，但拆成两个参数，对应关系见附录 A。

### 1.2 替换后的模型（建议写入 00 v0.2 的 §2.3）

**时钟域。** 记 $\mathsf{Clk}$ 为全体 replica 及其本地 Session Manager。$m\in\mathsf{Clk}$ 在真实时刻 $\tau$ 读到的本地协议时钟记为 $C_m(\tau)$。以下条件只约束 $\mathsf{Clk}$ 中的 honest 成员。Byzantine 成员可以声称任何时刻，对它们的时钟加约束没有意义。

- **C1 两两偏差。** 对 $\mathsf{Clk}$ 中任意两个 honest 成员 $m,m'$ 和任意 $\tau$：$|C_m(\tau)-C_{m'}(\tau)|\le\Delta_G$。
- **C2 对真实时间的偏移〔补充〕。** 对 $\mathsf{Clk}$ 中任意 honest 成员 $m$：$|C_m(\tau)-\tau|\le\delta_G$。
- **C3 单调连续〔补充〕。** $C_m$ 不递减且连续。校时只调整速率（slew），不跳变（step）。
- **C4 短区间速率〔补充〕。** 在长度为 $T_{\rm build}+\Delta_i+\Delta_G$ 量级的区间上，本地经过的时长等于真实经过的时长。若要保留速率误差 $\rho$，把本地测得的时长 $D$ 换成 $(1+\rho)D$。本文取 $\rho=0$。
- **C5 同节点同源〔补充〕。** 同一节点上的 replica 与 Manager 读同一个时钟。

**为什么需要 C2。** 只有 C1 时，全部诚实时钟可以一起偏离真实时间任意远。但 L1 时间戳、Espresso 的 $\theta$ 都以真实时间为准（E-TS 的基准是真实提出时刻），$T_{\rm pub}$、$T_E$、$T_{\rm L1}$、$T_{\rm batch}$ 等服务预算也是真实时长。要把本地时钟上的开始、停止同这些量衔接，就需要一个对真实时间的偏移界。

C2 本身推出两两偏差不超过 $2\delta_G$，所以单参数部署可以只给 $\delta_G$，并取 $\Delta_G=2\delta_G$。仍单独保留 $\Delta_G$，是因为有些条件只比较两个诚实时钟（Accept 窗口、交接重叠），用 $\Delta_G$ 可以得到更紧的界。

**为什么需要 C3 与 C5。** first_received 记录首次接收的本地时刻，且不能覆写（[02 §3.2](02_session_program_zh_v0_1.md) 第 117 行）。时钟回跳会使同一 replica 给先后收到的 proposal 记下颠倒的时刻。Close 不可撤销，回跳不会恢复已经停止的签名，但会推迟尚未触发的停止。

C5 保证 Manager 的 TIME 关闭与 Program 自身的停止在同一时刻触发。不采用 C5 时，两者之差由 C1 限制在 $\Delta_G$ 以内，只影响活性。

**外部观察者。** 不变。读取者、Publisher 和 Batcher 通过公开日志验证结果，不以自己的接收时刻改写历史顺序。

### 1.3 换算引理

对公开时刻 $x$（如 $\tau_s^{\rm start}$、$\tau_s^{\rm stop}$、$\mathit{ts}^*(h)$），把 honest 成员 $m$ 的本地时钟到达 $x$ 的真实时刻记为

$$
\tau_m(x):=\min\{\tau: C_m(\tau)\ge x\}.
$$

**L-CLK〔补充〕。** 由 C2、C3，$x-\delta_G\le\tau_m(x)\le x+\delta_G$。由 C1、C3、C4，对任意 honest $m,m'$，$|\tau_m(x)-\tau_{m'}(x)|\le\Delta_G$。

*证明。* 由连续性，$C_m(\tau_m(x))=x$；由 C2 得 $|x-\tau_m(x)|\le\delta_G$。由 C1，在这一时刻 $C_{m'}\ge x-\Delta_G$；由 C4，$m'$ 在此后至多 $\Delta_G$ 的真实时间内也到达 $x$。交换 $m,m'$ 即得另一侧。∎

**用法。** 现行规格中每一处“协议时刻到达 $x$”的判断，改写后都在真实时间区间 $[x-\delta_G,x+\delta_G]$ 内触发，不同 honest 成员的触发时刻至多相差 $\Delta_G$。下文的性质仍以真实时刻 $\tau$ 陈述；接口输入里的“协议时刻”一律指本地时钟 $C_m$ 的读数。

## 2. 读本地时钟的位置

| 位置 | 规则 | 需要的修改 |
| --- | --- | --- |
| [02](02_session_program_zh_v0_1.md) §3.5 第 180 行 | Enter：到达 $\tau_s^{\rm stop}$ 后返回 REJECT | 只改读法 |
| 02 §3.6 第 186、190 行 | Close：到达 $\tau_s^{\rm stop}$ 时本地触发 | 只改读法 |
| 02 §3.7 第 198 行 | Build：$h=\mathrm{slot}(\tau)$；发送前检查当前时刻小于 $\tau_s^{\rm stop}$ | 只改读法 |
| 02 §3.8 第 213–220 行 | Accept 的时间条件 | **改公式**（§3.2(c)） |
| 02 §3.9 | NotBefore 返回公开时刻，比较在本地进行 | 只改读法 |
| 02 §4.6 | $T_{\rm win}$、$\tau_{\rm st}$、$\tau_{\rm sync}$、同步开始、$\tau_{\rm end}$ | **改公式** |
| 02 §4.7、§4.9 | BFT-5 的时限、BFT-7 的按时停止 | **改陈述** |
| 02 §4.10 | G1.2、G1.3、G3.2 | **改界** |
| [03](03_session_manager_zh_v0_1.md) 第 97–98 行 | ClockEvent{protocol_time} | 写明取 $C_m$，并采用 C5 |
| 03 第 225、257 行 | 到达 $\tau_s^{\rm stop}$ 时以 TIME 关闭，任何 phase 都适用 | 只改读法 |
| 03 第 250 行 | Enter 守卫 5：当前协议时刻小于 $\tau_s^{\rm stop}$ | 只改读法 |
| 03 第 168 行 | ObservePublic 第 6 步重查时间守卫 | 只改读法 |

以下部分不读本地时钟，本身不需要改：

- 01 的合约接口；
- 02 的 CheckFINAL、CheckEvidence；
- 03 的 Kill 判据；
- 04 的 PUB-1…3；
- 05–08 全部；
- 09 的 GFR 重放（事件键只取 $\mathit{ts}_{\rm L1}$、$\theta_q$ 和公开参数）与 Derivation。

各文件中依赖“协议时刻等于真实时刻”的预算和说明，在 §3 逐条列出。

## 3. 各模块修改

### 3.1 00 系统架构

1. §2.3：用 §1.2 的内容替换 Global clock 段。
2. §3.4 参数前提（第 575 行）：$T_{\rm L1}\le T_{\rm dep}$ 改为

$$
T_{\rm L1}+\delta_G\le T_{\rm dep}.
$$

   快时钟 leader 在真实时刻 $\mathit{ts}^*(h)-\delta_G$ 就到达 slot $h$。这时离 $\ell^{\rm sch}(h)$ 的 timestamp 只过去了 $T_{\rm dep}-\delta_G$。不满足这条时，Build 与 Accept 只是等待 L1 输入，影响的是活性。$T_{\rm dep}+b_{\rm L1}\le T_{\rm drift}$ 只含公开量，不变。
3. §1.4 E-TS 不变。$\Delta_{\rm ts}$ 约束的是 Espresso proposer，与 $\mathsf{Clk}$ 无关；两者在预算中相加（§3.4、§3.5）。共同故障模式见审查 K9。
4. §4.2 第 660 行“协议时刻由 §2.3 的本地协议时钟提供”保留原文，含义改为 $C_m$。

### 3.2 02 Session Program

**(a) 停止：Enter、Close、BFT-7。** 三处的判断都改为 $C_m(\tau)\ge\tau_s^{\rm stop}$。BFT-7 第 388 行改为本地形式，再加一条真实时间推论：

> **按时停止。** 每个 honest replica $m$ 在 $C_m(\tau)\ge\tau_s^{\rm stop}$ 时不再生成该 cid 的任何新共识签名；已较早 Close 的实例也不恢复签名。离线或暂停的 replica 恢复时先检查这一条件。
>
> *推论。* 由 L-CLK，真实时刻 $\tau_s^{\rm stop}+\delta_G$ 之后没有 honest 签名；最早的 honest 停止可以发生在 $\tau_s^{\rm stop}-\delta_G$。

一种备选做法是把本地停止键前移到 $\tau_s^{\rm stop}-\delta_G$，换回“真实时刻 $\tau_s^{\rm stop}$ 之后没有 honest 签名”。代价是快时钟 replica 在真实 $\tau_s^{\rm stop}-2\delta_G$ 就停止，每个窗口末尾损失 $2\delta_G$ 的生产，K4 的问题也更重。本文建议停止键保持不变，由下游预算吸收 $\delta_G$。

**(b) Build。** 取 $h=\mathrm{slot}(C_m(\tau))$，发送前检查 $C_m<\tau_s^{\rm stop}$。公式不变。

**(c) Accept 的时间条件（第 213–217 行）〔补充：等待修法，把“过早”从拒绝改为等待〕。** 下面的 $\tau_{\rm recv}$ 指本 replica 首次收到该 proposal 时的本地时刻。把第 5 项拆成四条：

- **5a 过早界。** $\tau_{\rm recv}\ge\mathit{ts}^*(B.h)-\Delta_G$，否则 REJECT。
- **5b 不超前。** 当前本地时刻满足 $C_m\ge\mathit{ts}^*(B.h)$，否则返回 WAIT(time)。之后重试时仍用原来的 $\tau_{\rm recv}$。
- **5c 及时。** $\tau_{\rm recv}<\mathit{ts}^*(B.h+1)+T_{\rm win}$。
- **5d 停止。** $\tau_{\rm recv}<\tau_s^{\rm stop}$。

*诚实 proposal 一定满足 5a。* 设 honest leader $m$ 在真实时刻 $r_0$ 开始 Build，令 $c_0=C_m(r_0)$。由于 $\mathrm{slot}(c_0)=h$，有 $c_0\ge\mathit{ts}^*(h)$。设 honest replica $n$ 在 $r_1\ge r_0$ 收到。由 C1、C3：

$$
C_n(r_1)\ge C_m(r_1)-\Delta_G\ge c_0-\Delta_G\ge\mathit{ts}^*(h)-\Delta_G.\ ∎
$$

因此 5a 只拒绝不可能来自 honest leader 的 proposal，同时给 WAIT(time) 的缓存设了上界。

*5b 只推迟，不拒绝。* 现行条件把“过早”判为 REJECT，而首次接收时刻不能覆写（第 117、220 行），所以一次拒绝就是永久拒绝。5b 把它改成等待，最长等到 $\tau_n(\mathit{ts}^*(h))\le\mathit{ts}^*(h)+\delta_G$。

*放宽修法。* 直接把下界放宽为 $\mathit{ts}^*(B.h)-\Delta_G\le\tau_{\rm recv}$，不加 5b。实现更简单，但 G1.2 会放宽到 $\Delta_G+\delta_G$（见 (g)）。本文建议采用等待修法。v6 用的是第三种做法，即滞后修法（leader 主动滞后），比较见附录 A。

WAIT(time) 是新增的 WAIT 原因。BFT 内核须像处理 WAIT(missing) 一样处理它：暂不投票，到时重试。first_received 用什么作键，仍是 S20 的待决项。

**(d) NotBefore 与 view 起算时刻。** NotBefore 的返回值不变。§4.6 的 $\tau_{\rm st}$ 改为用真实时刻定义：

$$
\tau_{\rm st}(m,v):=\max\{\tau_{\rm view}(m,v),\ \tau_m(\mathsf{NotBefore}(P_{m,v}))\}.
$$

由 L-CLK，第二项与公开值 NotBefore 至多相差 $\delta_G$。

**(e) §4.6 有界进展的适用条件。**

1. **区间终点。** 要求 $\tau_{\rm end}\le\min_{m\in U}\tau_m(\tau_s^{\rm stop})$，充分条件是 $\tau_{\rm end}\le\tau_s^{\rm stop}-\delta_G$。快时钟 replica 最早在真实 $\tau_s^{\rm stop}-\delta_G$ 停止，此后不能再计入 active replicas。
2. **Metadata window。**

$$
T_{\rm win}\ge T_{\rm build}+\Delta_{i_s}+\Delta_G.
$$

   *推导。* 沿用 (c) 的记号，有 $c_0<\mathit{ts}^*(h+1)$ 和 $r_1\le r_0+T_{\rm build}+\Delta_{i_s}$。由 C1、C4，

$$
C_n(r_1)\le C_m(r_1)+\Delta_G=c_0+(r_1-r_0)+\Delta_G<\mathit{ts}^*(h+1)+T_{\rm build}+\Delta_{i_s}+\Delta_G.
$$

   若不用 C4、只用 C2，最后一项换成 $2\delta_G$。∎
3. **同步开始。** 四个条件中的“$\tau_{\rm lc}=\tau_s^{\rm start}$”改为 $\tau_{\rm lc}\le\tau_s^{\rm start}-\delta_G$；“全部 active replicas 已在该时刻前初次 Enter”中的“该时刻”指 $\tau_{\rm lc}$；同时取 $\tau_{\rm sync}=\tau_s^{\rm start}+\delta_G$。

   理由：各 active replica 的窗口开始分布在 $[\tau_s^{\rm start}-\delta_G,\tau_s^{\rm start}+\delta_G]$ 内，view 1 的起算时刻至多相差 $\Delta_G$。所选 BFT 的 $T_{\rm sync}$、$T_{\rm next}$ 必须能吸收 $\Delta_G$ 的起算差，这一点应写进 BFT 实现给出的预算前提。

**(f) BFT-5 的时限。** 采用等待修法后，follower 可能因 5b 而等待。令 $h$ 为该 view 提议的 height，定义

$$
\hat\tau(h):=\max_{n\in U}\tau_n(\mathit{ts}^*(h))\le\mathit{ts}^*(h)+\delta_G.
$$

第 1 项改为

$$
\tau_f\le\max\{\tau_0+T_{\rm build},\ \hat\tau(h)\}+T_{\rm fin}.
$$

第 2 项同样用 $\max\{\tau_0+T_{\rm build},\hat\tau(h)\}$ 代替 $\tau_0+T_{\rm build}$。也可以换一种写法：要求 $T_{\rm fin}$、$T_{\rm next}$ 本身包含至多 $\Delta_G$ 的等待。

**(g) G1。**

- **G1.2 Not ahead。** 改为 $\mathit{ts}^*(B.h)\le\tau_{\rm in}(B)+\delta_G$（等待修法）。

  *推导。* 由 BFT-2（第 311 行），某个 honest replica $n$ 在 $\tau_{\rm in}(B)$ 之前或当时返回了 ACCEPT。按 5b，那时 $C_n\ge\mathit{ts}^*(B.h)$；由 C2，那一刻的真实时间不早于 $\mathit{ts}^*(B.h)-\delta_G$。∎

  采用放宽修法时，界为 $\mathit{ts}^*(B.h)\le\tau_{\rm in}(B)+\Delta_G+\delta_G$。
- **G1.3 Timely admission。** 本地形式改为

$$
\mathit{ts}^*(B.h)-\Delta_G\le\tau_{\rm recv}<\min\{\mathit{ts}^*(B.h+1)+T_{\rm win},\ \tau_s^{\rm stop}\}.
$$

  对应的真实时间读法是：首次接收的真实时刻早于 $\mathit{ts}^*(B.h+1)+T_{\rm win}+\delta_G$。
- **G3.2 Catch-up by padding。** 改为

$$
\tau_f-\mathit{ts}^*(B.h)<b_{\rm L2}+T_{\rm build}+T_{\rm fin}+\delta_G.
$$

  *推导。* 由 $r_0\le c_0+\delta_G<\mathit{ts}^*(h)+b_{\rm L2}+\delta_G$ 和 $\hat\tau(h)\le\mathit{ts}^*(h)+\delta_G$，(f) 中 $\max$ 的两项都小于 $\mathit{ts}^*(h)+b_{\rm L2}+\delta_G+T_{\rm build}$。∎
- G1.1 与 G2 全部不变。G2.6 按本地停止判断，也不变。

**(h) first_received。** 保存本地时刻，并跨重启持久化；重启后的时钟仍须满足 C3。键的定义留给 S20。

### 3.3 03 Session Manager

1. **ClockEvent（第 97–98 行）。** `protocol_time` 取 $C_m(\tau)$，与本节点 Program 同源（C5）。
2. **TIME 关闭、Enter 守卫 5、ObservePublic 第 6 步**（第 225、257、250、168 行）：只改读法。在真实时间上，各 Manager 的 TIME 关闭分布在 $[\tau_s^{\rm stop}-\delta_G,\tau_s^{\rm stop}+\delta_G]$ 内。
3. **候选调度（§4.3）** 规则本身不必改，交接处有两个可选的保护〔补充〕：
   - 只为消除时钟造成的重叠（审查 K5），后继的首次提议推迟到本地 $\tau_s^{\rm stop}+\Delta_G$。这样只恢复完美时钟下的交接余量（10 §3.3 的 0.5 s），不处理结果的送达延迟。
   - 若采纳 S10(b) 或 S27 的修法，后继在本地 $\tau_s^{\rm stop}+\Delta_G+\Delta_i+\Delta_{\rm cross}$ 之后才提议，即原推迟时长再加 $\Delta_G$。

   *推导。* 前驱的最后一个 honest 签名，签于该签名者本地时刻早于 $\tau_s^{\rm stop}$ 的时候。由 C1，同一真实时刻后继的本地时钟早于 $\tau_s^{\rm stop}+\Delta_G$。之后证书在 $\Delta_i$ 内组合，再经 $\Delta_{\rm cross}$ 由 W26 送达。∎

### 3.4 04 Publisher

Publisher 不读时钟，PUB-1…5 本身不变。变化在 PUB-4 推论（第 144–151 行）：慢时钟 replica 在自己的本地停止前形成的结果，在真实时间上最晚到 $\tau_s^{\rm stop}+\delta_G$ 才交给 Publisher。推论改为：若 $\tau\le\tau_s^{\rm stop}+\delta_G$，且

$$
T_{\rm grace}>T_{\rm pub}+T_E+\Delta_{\rm ts}+\delta_G,
$$

则 $\theta_q<\theta_s^{\rm end}$。

### 3.5 09 Finality Rules

1. GFR 的事件、排序键、全序、R-WAIT 与状态机都不变，重放仍只读公开日志。
2. §3.2 SessionEnd（第 95 行）原说明为“$\tau_s^{\rm stop}$ 是 honest replicas 停止新共识签名的时刻（BFT-7）”。改为“$\tau_s^{\rm stop}$ 是 honest replicas 按本地时钟停止的键；真实停止时刻不晚于 $\tau_s^{\rm stop}+\delta_G$”。$\theta_s^{\rm end}$ 的定义不变。
3. §3.5 的发布余量（第 141 行）换成 §3.4 的新不等式。
4. §3.5 的公告、Espresso 进度、原生窗口三条不变：$M_a$ 衔接的是 L1 收录与 Espresso 时间键，$W_{\rm exp}$ 从公开的 $\mathit{ts}^*(h)$ 起算，都不经过本地时钟。
5. §4.4 第 5 步（第 243–251 行）的窗口公式不变。但把 $T_{\rm init}$ 解释为真实准备预算时，要扣去 $\delta_G$：新窗口中的快时钟 replica 在 $\mathit{ts}^*(h_{\min,\rm new})-\delta_G$ 就开始，因此从 $\mathrm{act}(U)$ 起算能保证的准备时间是 $T_{\rm init}-\delta_G$，而不是 $T_{\rm init}$。

### 3.6 01 L1 Authority

§3.10 RESUME 治理规程（第 267–273 行）中，上界只含公开量，不变。下界的准备预算改为

$$
M_a+T_{\rm ready}+\delta_G\le T_{\rm prep}.
$$

原因是新 session 中的快时钟 replica 在真实时刻 $\mathit{ts}^*(h_{\rm res}+1)-\delta_G$ 就开始。$T_{\rm ready}$ 按 10 §2.2 的用法，指激活之后的处理与状态准备时间。

### 3.7 05–08 与 Derivation

不改。A-DELIVER、CERT-5、DA-L1/L2 都是真实时间的活性预算；Certifier 与 Inbox 用 L1 位置判断截止，不用时钟。实现中若用本地计时器做重试或超时，只影响活性。

### 3.8 10 端到端流程

**§2.3 约束重算。**

| 约束（新形式） | 代入 10 的参数 | 余量 | 对偏差的要求 |
| --- | --- | --- | --- |
| $T_{\rm L1}+\delta_G\le T_{\rm dep}$ | 19.2 min + $\delta_G$ ≤ 20 min | 48 s | $\delta_G\le48$ s |
| $T_{\rm grace}>T_{\rm pub}+T_E+\Delta_{\rm ts}+\delta_G$ | 120 s > 75 s + $\delta_G$ | 45 s | $\delta_G<45$ s |
| $M_a+T_{\rm ready}+\delta_G\le T_{\rm prep}\le T_{\rm drift}-\epsilon$ | 25 min + $\delta_G$ ≤ 25 min ≤ 28 min | 下界为 0（首块在 $h_{\rm res}+1$，另有 1 s） | 须调高 $T_{\rm prep}$。上界还有 3 min，例如取 26 min 可容 $\delta_G$ 到约 1 min |
| $T_{\rm win}\ge T_{\rm build}+\Delta_i+\Delta_G$ | 10 未给出 $T_{\rm build}$、$\Delta_i$ 的部署值 | — | — |
| $M_a\ge T_{\rm L1}+2\Delta_{\rm ts}+b_{\rm L1}$；$T_{\rm dep}+b_{\rm L1}\le T_{\rm drift}$；$W_{\rm exp}+M_a+T_{\rm batch}+T_{\rm drift}\le W_{\rm seq}$ | 不变 | 6 s；9 min 48 s；10 min | 无 |

**§3.3 正常交接。** 示例中 Res(32 410) 在 09:00:10.5 到达 a，NotBefore 为 09:00:11，余量 0.5 s。有偏差时，记 c 的 leader 本地时钟到达 09:00:10 的真实时刻为 $r$。由 L-CLK，a 的任一 honest replica 不早于 $r+1\,{\rm s}-\Delta_G$ 到达 09:00:11。决定加转发需要 0.5 s，因此无停顿的结论要求 $\Delta_G<0.5$ s。否则 a 的一部分 replicas 先在 cid(32 409) 上出块，随后再切换（03 §4.3），期间权重短暂分裂在两个 cid 上（审查 K5）。

**§3.4 变体 1b。** “32 410 的投票都在 09:00:11 之前签出”改为“都在各签名者本地 09:00:11 之前签出，真实时刻不晚于 09:00:11 + $\delta_G$”。

**窗口末尾。** 完美时钟下，$h_{\max}$ 能被决定的条件是 $T_{\rm build}+T_{\rm fin}<b_{\rm L2}$。有偏差时，$h_{\max}$ 的 leader 可能是慢时钟，而 quorum 又需要快时钟 replicas 的签名，条件就变为

$$
\Delta_G+T_{\rm build}+T_{\rm fin}<b_{\rm L2}.
$$

与完美时钟相比，窗口末尾多出约 $\Delta_G/b_{\rm L2}$ 个没有进展保证的 heights。$h_{\max}$ 决定不了时，交接退化为变体 1a（审查 K4）。

## 4. 落地方式〔待确认〕

这是跨模块的模型替换，不是结构重组。可以选的落地方式有两种：

- 出新版本文件：00 v0.2（§2.3、§3.4）、02 v0.2（§3.8、§4.6–§4.10）、03 v0.2（ClockEvent）、04 v0.2（PUB-4 推论）、09 v0.3（§3.2 的说明、§3.5、§4.4 的说明）、01 v0.2（§3.10）、10 v0.3（数值）。
- 在原文件上就地修改。

选哪一种，以及 Accept 用等待、放宽、滞后哪一种修法，待你决定。

## 附录 A：与 v6 的 $\epsilon$ 处理的对应

v6 的 Regional BFT 抽象 v0.2 的 A3 只有一个参数 $\epsilon$，即 $|\mathit{now}-\tau|\le\epsilon$，对应本文的 C2，取 $\delta_G=\epsilon$。

| v6 | 本文 |
| --- | --- |
| W1 not-ahead：$t(h(B))\le\mathit{now}+\epsilon$ | 放宽修法（下界放宽 $\Delta_G$） |
| Leader 目标 $h^*=\min\{\mathrm{slot}(\mathit{now}-\epsilon),H_s\}$ | 滞后修法（见下） |
| View 超时从 $\max\{\text{enter},t(h(P)+1)+2\epsilon\}$ 起算 | §3.2(d) 的 $\tau_m(\mathsf{NotBefore})$ 与 §3.2(e) 的同步开始 |
| G1.2：$t(h(B))\le\tau_{\rm cert}(B)+2\epsilon$ | §3.2(g) 的 G1.2：等待修法下为 $+\delta_G$，放宽修法下为 $+\Delta_G+\delta_G$ |
| 时钟失效只影响 G1.2、G1.3 | 审查 §1、K8：v9 中同样只影响时间性质与活性 |

**滞后修法。** Leader 取 $h=\mathrm{slot}(C_m-\Delta_G)$。这样 honest proposal 在任何 honest receiver 处都不会过早，Accept 的下界可以保持严格，也不需要 WAIT(time)；G1.2 仍是 $+\delta_G$。代价有三：

- 每个 block 滞后 $\Delta_G$，G3.2 的界再加 $\Delta_G$，$T_{\rm win}$ 的下界也再加 $\Delta_G$；
- 每个窗口开头损失 $\Delta_G$；
- $h_{\max}$ 能被决定的条件变为 $2\Delta_G+T_{\rm build}+T_{\rm fin}<b_{\rm L2}$。

在 $b_{\rm L2}=1$ s 的设定下，窗口末尾的代价最明显，所以本文建议采用等待修法。
