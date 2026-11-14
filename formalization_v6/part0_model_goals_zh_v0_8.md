# MegaETH Sequencer Rotation 技术报告 · Part 0

## 模型、假设与设计目标（中文工作稿 v0.8）

日期：2026-10-26。
状态：工作稿。本稿写四件事：(1) 系统模型；(2) 假设与外部组件；(3) 去中心化区域轮换 sequencer（DRS）的定义、性质与设计目标；(4) 以 DRS 为子模块的 L2。
各模块的接口与保证见 [Part 1 v0.3](part1_module_interfaces_zh_v0_3.md)。

---

## 0. 写法与约定

### 0.1 形式化方式

本稿采用共识文献中常见的 property-based 写法 [GKL15, PS17]。

1. **执行。**
   一次执行由环境 $\mathcal Z$、敌手 $\mathcal A$ 与协议 $\Pi$ 组成，记作 $\mathrm{EXEC}_{\Pi,\mathcal A,\mathcal Z}(\kappa)$。$\mathcal Z$ 提供全部外部输入；$\mathcal A$ 控制被腐化的参与方，并在假设允许的范围内决定消息延迟。
2. **外部系统写成假设。**
   L1、Espresso、DA 不写成理想功能。对每个外部系统，我们只给出必要的使用接口，以及协议所依赖的性质。协议写入这些系统的数据直接继承这些性质。
3. **内部计算写成规则。**
   原生执行、全局重算、L1 派生、状态结算等，写成所有人可以独立计算的确定性规则。它们既不是参与方，也不是功能。
4. **设计目标是执行上的性质。**
   称性质 $P$ 成立，若对所有满足相应假设的 PPT $(\mathcal A,\mathcal Z)$，有 $\Pr[\mathrm{EXEC}\text{ 违反 }P]\le\mathrm{negl}(\kappa)$。
5. **不追求密码学意义上的可组合性。**
   本稿不要求 UC 或 GUC 意义下的 realization。模块之间的组合在后续 Part 中用 assume–guarantee 的方式论证。

### 0.2 记号

| 记号 | 含义 |
| --- | --- |
| $\kappa$；$\mathrm{negl}$ | 安全参数；可忽略函数。 |
| $\mathcal Z,\mathcal A,\Pi$ | 环境、敌手、协议。 |
| $\tau$ | 真实时间。$t$ 保留给 leader term，不表示时间。 |
| $\ell$，$\mathit{ts}_\ell$ | L1 高度及其时间戳。 |
| $q$，$J_q$，$\theta_q$ | Espresso 高度、块及其时间戳。 |
| $h$；$t(h)$ | L2 高度；高度 $h$ 的 L2 时间戳（N1）。 |
| $o(B)$ | 块 $B$ 的原生 L1 origin。 |
| $\mathcal R$；$o,pk_o,i_o$ | 区域集合；质押 position、其公钥与所属区域。 |
| $s$；$V_s,w_o,W_s,F_s$ | session；其委员会、成员权重、总权重、故障门槛（§1.3）。 |
| $\mathcal W_s=[\tau_s^{\rm open},\tau_s^{\rm close}]$ | session $s$ 的职责窗口（§1.3）。 |
| $\mathsf M$；$\mathsf C$ | 治理方（multisig）；可信 certifier。 |
| $h_{\rm res}$；$L$ | RESUME 的目标高度；RESUME 所在的 L1 块。 |
| $\mathrm{LOG}^X_P(\tau)$ | 诚实参与方 $P$ 在 $\tau$ 时看到的账本 $X$ 的已最终化日志（§2.4）。 |
| $X\preceq Y$ | 序列 $X$ 是 $Y$ 的前缀。 |
| $X\sim Y$ | $X\preceq Y$ 或 $Y\preceq X$（可比较）。 |
| $\mathrm{Rec}^u(\tau)$，$G^u(\tau)$ | 诚实观察者 $u$ 在 $\tau$ 时的重算记录及其给出的全局链（§3.1）。 |
| $G'$ | Batch Inbox 已接纳的高度前缀，即 cursor 以下的高度（§4.1）。 |
| $D^u(\tau)$ | 诚实派生节点 $u$ 在 $\tau$ 时由 L1 与 DA 派生出的原生链（§4.1）。 |
| $D_L$ | 按 L1 顺序处理到 RESUME 的 `CursorSet` 为止的派生链（§4.1）。 |

### 0.3 接口返回值约定

本稿和后续 Part 的所有接口统一使用以下返回值：

- **值，或经验证的 $\bot$。** 查询在调用者已经掌握的数据范围内可以判定时，返回结果；$\bot$ 表示“已证明不存在”。
- **`WAIT`。** 所需数据尚未取得。`WAIT` 不等于 $\bot$。
- **`REJECT`。** 请求格式错误，或可判定为无效。

“提交”指请求已到达外部系统，且格式正确、费用充足；本地尝试发送不算提交。

---

## 1. 系统模型

### 1.1 参与方与角色

- **区域。** $\mathcal R$ 是 genesis 固定的有限区域集合。
- **运营者与 position。** 质押注册产生 position $o$，绑定签名公钥 $pk_o$ 与区域 $i_o\in\mathcal R$。下文的“成员”指 position 的运营者。
- **治理方 $\mathsf M$。** multisig 委员会，作为**一个**诚实的逻辑参与方建模（A-GOV）。它在 L1 Authority 中发布两类控制：
  - **Force（强制轮换）：** 跳过故障区域或更新区域 policy；
  - **RESUME（紧急恢复）：** 在紧急状态下（§4.1）指定一个未来的 L2 高度 $h_{\rm res}$。合约把 Batch Inbox 的 cursor override 到 $h_{\rm res}$，L1 派生把空缺填成 deposit-only；全局链随即改为 L1 派生链，并从派生链在 $h_{\rm res}$ 处的块重新出块（§3.1）。
- **可信 certifier $\mathsf C$。** 原先由 TEE 承担的 certifier，作为**一个**可信的诚实参与方建模（A-CERT）。它只在 L2 层（§4）核对批次，不参与恢复决策，也不进入 DRS。
- **角色。** 用户、publisher、batcher、观察者、证据提交者都是角色而非身份。任何参与方，包括被腐化的参与方，都可以承担这些角色；一个运营者可以兼任多个角色。
- **L1 合约不是参与方。** L1 Authority、Deposit Portal（强制包含入口）、Batch Inbox（批次收件箱）等是部署在 L1 上的确定性程序，属于 $\Pi$ 的一部分。

**术语。** “Force”（强制轮换）是治理动作；“强制包含”（forced inclusion）是用户经 Deposit Portal 提交交易、绕过 sequencer 审查的机制。两者无关，下文严格区分。

环境 $\mathcal Z$ 提供全部外部输入：用户交易、强制包含交易、质押请求，以及交给 $\mathsf M$ 的治理指令。
$\mathcal Z$ 与 $\mathcal A$ 可以任意通信。

### 1.2 时间

全局时钟给出真实时间 $\tau\in\mathbb R_{\ge 0}$，所有参与方读到同一 $\tau$。
有界时钟偏差可以按标准做法并入下文的 $\Delta$。

本稿区分四种坐标，它们彼此不能互换：

1. 真实时间 $\tau$；
2. L1 高度 $\ell$ 及时间戳 $\mathit{ts}_\ell$；
3. Espresso 高度 $q$ 及时间戳 $\theta_q$；
4. L2 高度 $h$ 及时间戳 $t(h)$。

**链时间戳。** 对 L1 或 Espresso 的任一块 $b$，记 $\tau_{\rm prop}(b)$ 为 $b$ 被提出（产生）的真实时间。

**A-TS（时间戳准确）。** 存在常数 $\Delta_{\rm ts}$，使得：

- 对每个 L1 块 $b$：$\lvert\mathit{ts}(b)-\tau_{\rm prop}(b)\rvert\le\Delta_{\rm ts}$；
- 对每个 Espresso 块 $J_q$：$\lvert\theta_q-\tau_{\rm prop}(J_q)\rvert\le\Delta_{\rm ts}$；
- 一个块不会早于它被提出的时刻 finalized。

**推论（跨链时间戳偏差）。** 对任意 L1 块 $b$ 与 Espresso 块 $J$，

$$
\bigl\lvert\,(\mathit{ts}(b)-\theta(J))-(\tau_{\rm prop}(b)-\tau_{\rm prop}(J))\,\bigr\rvert\ \le\ 2\Delta_{\rm ts}.
$$

特别地，两条链在同一真实时刻提出的块，时间戳之差不超过 $2\Delta_{\rm ts}$。

“提出”与“最终化”是两件事。块被提出之后，要经过一段时间才 finalized 并对所有诚实方可见；这段延迟由 $T_{\rm L1}$、$T_E$ 约束（§2.4）。
一个块的内容在它被提出时就已固定。即使它很晚才被最终化，其中也不会出现提出之后才产生的数据。

**原生时间规则（OP Stack 规则，不是假设）。** 以下规则由原生派生强制执行（见 `megaeth_specs/specs/block-certification/block-records.md` 与 `derivation-pipeline.md`）：

- **N1（固定出块时间）。** $t(h)=t_{\rm gen}+(h-h_{\rm gen})\cdot b_t$，其中 $b_t$ 是固定的 L2 出块间隔。高度与时间戳一一对应。
- **N2（sequencer drift）。** 带用户交易的 L2 块，其 L1 origin $o(B)$ 满足 $\mathit{ts}(o(B))\le t(h(B))\le\mathit{ts}(o(B))+D_{\rm drift}$。
- **N3（sequencing window）。** 高度 $h$ 的批次若未能在该高度的截止 L1 块 $\mathrm{ddl}(h)$ 之前被接纳，派生就以 deposit-only 块填充 $h$。按时间计，$\mathit{ts}(\mathrm{ddl}(h))\le t(h)+W_{\rm seq}$，其中 $W_{\rm seq}$ 是 `seq_window_size` 对应的时长。

### 1.3 session 与职责窗口

为了陈述假设与目标，这里只引入最少的 session 概念。session 怎样选出、怎样交接，属于后续模块。

**定义 1.1（session）。**
DRS 把普通出块划分为 session。每个 session $s$ 确定：

- 区域 $i_s\in\mathcal R$ 与固定委员会 $V_s\subseteq\{o: i_o=i_s\}$；
- 每个成员的正整数权重 $w_o$，取自某个冻结的质押快照；总权重 $W_s=\sum_{o\in V_s}w_o$；
- 故障门槛 $F_s$，满足 $0\le F_s<W_s/2$；
- 锚点 $A_s$ 与固定的高度区间 $(h(A_s),H_s]$，其中 $H_s=h(A_s)+\mathrm{TB}$。由 N1，$s$ 能生产的块对应的时间戳区间也就固定了。

$s$ 的**证书**是来自 $V_s$ 中不同成员、总权重严格大于 $F_s$ 的一组匹配签名。
这些字段一经确定便不再改变；之后的 slash 或新快照都不改写它们。

**定义 1.2（职责窗口）。** session $s$ 的职责窗口 $\mathcal W_s=[\tau_s^{\rm open},\tau_s^{\rm close}]$ 由两个时刻界定。

- **开始 $\tau_s^{\rm open}$：** 首次有参与方能够算出 $V_s$ 的时刻。
- **结束 $\tau_s^{\rm close}$：** 最早的时刻 $\tau$，使得任何在 $\tau$ 之后**提出**的 Espresso 块，都不能再把 $s$ 的块加入全局链。

用“提出时刻”而不是“最终化时刻”来定义结束，原因是：块的内容在提出时就固定了，晚于 $\tau_s^{\rm close}$ 才被腐化的密钥，无法再把新签名放进任何有效的块里。
RESUME 时全局链采纳的派生链块（出处 $\mathsf{DER}$，§3.1）只含此前已被 Batch Inbox 接纳的内容，不带任何 session 的新签名，所以不影响这个定义。

职责窗口在本稿中有三个用途：

- **拜占庭计数（A-BYZ）。** 只计算在 $\tau_s^{\rm close}$ 之前被腐化的成员。
- **诚实的含义（§1.4）。** $s$ 的诚实成员须在整个窗口内在线并保持同步。
- **质押释放（S6）。** 只有在 $\tau_s^{\rm close}$ 再加上证据窗口之后，$s$ 的成员才能释放质押。

**例子。** 设区域为 A、B、C，每个 session 的 DATA 配额为 10，当前分支版本为 7。

| 场景 | 发生了什么 | 窗口的开始与结束 |
| --- | --- | --- |
| 开始（正常） | $s_1=(7,20)$ 在区域 A，从锚点 100（前驱的末块）出发，生产 101–110。$s_2=(7,21)$ 在区域 B，是 $s_1$ 的常规后继。 | $s_2$ 的委员会由 $s_1$ 的锚点 100 与来源区域 A 决定。所以块 100 一得到区域 FINAL，任何人都能算出 $V_{s_2}$：$\tau^{\rm open}_{s_2}$ 就是这一时刻。 |
| 正常结束 | 块 110 在 Espresso 块 $J_q$ 的重算中进入全局链。 | 此后提出的 Espresso 块都排在 $J_q$ 之后，不能再加入 $s_1$ 的块，所以 $\tau^{\rm close}_{s_1}=\tau_{\rm prop}(J_q)$。 |
| Force | 全局链停在 108 时，Force 在 Espresso 块 $J_{q'}$ 生效，关闭分支 7。 | 分支 7 上所有 session 的窗口都在 $\tau_{\rm prop}(J_{q'})$ 结束。新分支 8 的第一个 session 从 108 出发。 |
| Espresso 停机 | $s_2$ 出块期间 Espresso 停机，全局链停在 115。 | 按 R-EXP，$s_2$ 的块只能经由 $\theta\le t(H_{s_2})+W_{\rm exp}$ 的 Espresso 块进入全局链，而这样的块在 $t(H_{s_2})+W_{\rm exp}+\Delta_{\rm ts}$ 之前就已提出。所以即使停机很久，$\tau^{\rm close}_{s_2}$ 也有上界。 |

### 1.4 腐化、诚实与离线

**腐化模型。**
敌手可以**自适应地**在任意时刻腐化任意参与方，立即取得其全部状态，包括私钥与历史状态（不假设安全擦除），此后完全控制它。腐化不可撤销。
$\mathsf C$ 不可被腐化（A-CERT）。$\mathsf M$ 按 A-GOV 建模为诚实；安全性质 S1–S6 不依赖这一点（§5.1）。

**诚实参与方。**
从未被腐化、并按 $\Pi$ 运行的参与方，称为诚实参与方。诚实参与方可能崩溃或离线，之后重新上线；其持久化存储不丢失已提交的写入，包括签名记录。
称参与方在区间 $I$ 上**活跃**，若它在整个 $I$ 上诚实且在线。

session 成员的分类要更细一些，因为同步 BFT 要求诚实成员始终在线。

**定义 1.3（对 session 的分类）。** 对 session $s$ 与成员 $o\in V_s$：

- **拜占庭：** $o$ 在 $\tau_s^{\rm close}$ 或之前被腐化。
- **诚实：** 在整个 $\mathcal W_s$ 内，$o$ 未被腐化、持续在线，与 $V_s$ 的其他诚实成员保持区域内同步连接（§1.5），并按 $\Pi$ 运行。若 $o$ 在窗口开始前离线过，它须在 $\tau_s^{\rm open}$ 之前同步完所需的公开历史。
- **离线：** 既不是拜占庭的，也不是诚实的。这类成员没有被腐化，但在窗口内某段时间离线、掉队或崩溃。

下文说“$s$ 的诚实成员”时，总是指定义 1.3 意义下的诚实。

**定义 1.4（良态 session）。**
$s$ 是良态的，当且仅当

$$
\sum_{o\in V_s,\ o\text{ 不是对 }s\text{ 诚实的}} w_o\ \le\ F_s ,
$$

即拜占庭成员与离线成员的总权重不超过 $F_s$。
良态是关于一次执行的**谓词**，不是全局假设。区域整体离线时，相应 session 不良态，但这仍在模型之内。

### 1.5 通信

参与方之间使用认证点对点信道：敌手不能伪造或篡改诚实方的消息，但可以重排、重复消息，并在下述时限内决定延迟。信道不提供机密性。
消息大小及其随附的验证依赖不超过声明的上界 $m_{\max}$；$\Delta$ 包含接收方对这些依赖的验证处理。

- **区域内同步 $\Delta_i$。**
  对 $s$ 的任意两个诚实成员，于 $\tau$ 发出的消息在 $\tau+\Delta_{i_s}$ 之前送达并处理完毕。
  这一时限既用于区域活性，**也是快速确认安全（S4）的前提**，因为同步 BFT 的安全性依赖它 [AMNRY20]。
- **跨区域 $\Delta_{\rm cross}$。**
  跨区域网络可以暂时分区。在**连通期间**，任意两个在线诚实方之间于 $\tau$ 发出的消息在 $\tau+\Delta_{\rm cross}$ 之前送达。
  $\Delta_{\rm cross}$ 只用于活性。

大对象（例如可执行状态）不受 $\Delta$ 约束，见 §1.6 的 $T_{\rm ready}$。

### 1.6 本地计算与资源

- **本地计算视为瞬时，有一个例外。** 为给定锚点准备可直接执行的状态（下载、重建、验证、安装）需要时间。在所需输入可得、大小与负载在声明范围内时，这一步不超过 $T_{\rm ready}$。
- **费用。** 诚实方持有足够的资金，用以支付其职责所需的 L1、Espresso 与 DA 费用。

---

## 2. 假设与外部组件

### 2.1 密码学

**A-CRYPTO。**
签名方案满足 EUF-CMA 安全；哈希函数抗碰撞；承诺具有绑定性。敌手是 PPT 的。

### 2.2 PKI

**A-PKI。**
成员公钥经 L1 Authority 注册，并附 proof of possession；诚实方诚实地生成密钥。
$\mathsf M$ 与 $\mathsf C$ 的公钥在 genesis 固定，所有参与方与 L1 合约都知道。

### 2.3 区域拜占庭界

**A-BYZ。** 对 DRS 产生的每个 session $s$：

$$
\sum_{o\in V_s,\ o\text{ 对 }s\text{ 拜占庭}} w_o\ \le\ F_s .
$$

腐化按累计计算：在 $\tau_s^{\rm close}$ 之前任何时刻被攻陷的密钥都要计入。
A-BYZ 只限制拜占庭成员，不要求成员在线；在线与同步的要求放在“良态”（定义 1.4）里。

### 2.4 公共账本：L1 与 Espresso

**使用方式。** 对账本 $X\in\{\mathrm{L1},E\}$，参与方可以：

- **提交**一笔交易（对 Espresso 而言是一份 publication）；
- **读取**已最终化的日志。

诚实参与方 $P$ 在 $\tau$ 时看到的已最终化日志记为 $\mathrm{LOG}^X_P(\tau)$，它是块的序列。
协议只读取已最终化的块。

**通用账本假设。** 对 $X\in\{\mathrm{L1},E\}$，各性质如下：

- **X-S1（一致性）。** 对任意诚实方 $P,Q$ 与时刻 $\tau,\tau'$：$\mathrm{LOG}^X_P(\tau)\sim\mathrm{LOG}^X_Q(\tau')$。
- **X-S2（最终性）。** 若 $\tau\le\tau'$，则 $\mathrm{LOG}^X_P(\tau)\preceq\mathrm{LOG}^X_P(\tau')$。已最终化的块永不回退。
- **X-S3（完整读取）。** 对 $\mathrm{LOG}^X_P(\tau)$ 中的任意区间，$P$ 能读到该区间内相关数据的完整内容；数据为空时也能确认为空。因此，在自己的日志范围内，“不存在”是可以判定的。
- **X-L1（有界活性）。** 若诚实方在 $\tau$ 时提交一笔格式正确、费用充足的交易，且 $X$ 在 $[\tau,\tau+T_X]$ 上可用，则对每个在该区间上活跃的诚实方 $P$，这笔交易都在 $\mathrm{LOG}^X_P(\tau+T_X)$ 中。
- **X-L2（视图同步）。** 若某块在 $\tau$ 时出现在某个诚实方的日志中，且 $X$ 在 $[\tau,\tau+T_X]$ 上可读，则到 $\tau+T_X$ 时，它出现在每个在该区间上活跃的诚实方的日志中。

账本只负责排序与最终化，不理解本协议的语义，也不提供排序公平。在活性保证之外，日志的内容与顺序由敌手决定，可以包含任意垃圾输入。

**A-L1（Ethereum L1）。**
L1 满足上述全部性质，参数为 $T_X=T_{\rm L1}$，并且**始终可用**。此外：

- **L1-S4（合约忠实执行）。** 任意高度 $\ell$ 的合约状态、按顺序排列的完整事件列表，以及每笔调用的结果（包括失败），都由已最终化日志唯一确定。合约调用按日志顺序原子执行；已部署的合约代码在模型中固定不变。合约可以读取所在块的时间戳。
- **L1-S5（时间戳单调）。** 时间戳沿链严格递增。
- **L1-L3（出块与最终化节奏）。** 存在 $\delta_{\rm slot}$，使任意长为 $\delta_{\rm slot}$ 的区间内，都有一个最终会被 finalized 的 L1 块被提出。任何最终被 finalized 的块 $b$，在 $\tau_{\rm prop}(b)+T_{\rm L1}$ 之前出现在所有在该区间上活跃的诚实方的日志中。

**A-ESP（Espresso）。**
Espresso 满足 X-S1 至 X-S3（记为 E-S1 至 E-S3）。其活性性质只在可用期间成立，参数为 $T_X=T_E$。此外：

- **Namespace。** publication 带 namespace 标签。读取时，每块按顺序给出本链 namespace 的完整内容 $M_q$，可以为空。Espresso 只负责排序，不验证本协议的任何内容；本链 namespace 中的数据可以由任何人提交，包括敌手。
- **两种可用性。**
  - **排序可用：** 新 publication 能被收录并最终化。X-L1 只在排序可用期间成立。
  - **读取可用：** 已最终化的块能被读取。X-L2 只在读取可用期间成立。

  **停机**定义为排序不可用。
- **E-S4（时间戳单调）。** $\theta_q$ 沿日志非减。
- **E-L3（时间推进）。** 若 Espresso 在 $[\tau,\tau+T_E]$ 上排序可用，则到 $\tau+T_E$ 时，存在 $\theta\ge\tau-\Delta_{\rm ts}$ 的已最终化块。

E-S1 至 E-S4 在停机期间照样成立；停机只影响活性。
注意：一个在停机前提出的块，可能在停机结束后才被最终化，其时间戳仍是提出时的时间。

**记号。** 记 L1-S 为 X-S1 至 X-S3、L1-S4、L1-S5；E-S 为 E-S1 至 E-S4；L1-L、E-L 为相应的活性性质。L1-S 与 E-S **不含** A-TS：时间戳与真实时间的关系只由 A-TS 给出，用到它的目标单独列出。

### 2.5 数据可用性层（DA）

DA 由 L2 层（§4）使用；此外，$\mathsf{Replay}$ 只在处理 RESUME 时，经由 L1 派生规则读取已接纳批次的数据（§3.1）。先给出接口，再用假设陈述其性质。

**状态。**
存储表 $\mathsf{Store}: d\mapsto(\mathit{payload},\tau_{\rm cert})$，其中 $d$ 是承诺，$\tau_{\rm cert}$ 是认证时刻。表项只增不改。

**接口。**

- $\textsf{Disperse}(\mathit{payload})\to d$：提交 payload，认证完成后返回其承诺 $d$；认证完成前为 `WAIT`。
- $\textsf{IsCert}(d)\to\tau_{\rm cert}$ 或 $\bot$：查询 $d$ 是否已认证，以及认证时刻。
- $\textsf{Retrieve}(d)\to\mathit{payload}$ 或 `WAIT`：取回 $d$ 对应的完整 payload。

**A-DA。** DA 满足以下性质：

- **DA-S1（绑定）。** 每个 $d$ 至多对应一个 payload；$\textsf{Retrieve}(d)$ 只会返回这个 payload，绝不返回其他字节。
- **DA-S2（认证一致）。** 一旦 $d$ 被认证，此后所有参与方调用 $\textsf{IsCert}(d)$ 都得到同一个 $\tau_{\rm cert}$。
- **DA-L1（认证时限）。** 若 DA 在 $[\tau,\tau+T_{\rm DA}]$ 上可用，则于 $\tau$ 提交的 dispersal 在 $\tau+T_{\rm DA}$ 之前完成认证。
- **DA-L2（保留期内可取回）。** 若 $d$ 已认证、$\tau\le\tau_{\rm cert}+T_{\rm ret}$，且 DA 在 $[\tau,\tau+\Delta_{\rm DA}]$ 上可用，则于 $\tau$ 发出的 $\textsf{Retrieve}(d)$ 在 $\tau+\Delta_{\rm DA}$ 之前返回 payload。

DA-S1 与 DA-S2 始终成立；DA 不可用只影响活性。
L1 合约不能直接调用 $\textsf{IsCert}$，DA 绑定要经由 $\mathsf C$ 的签名消息带到 L1 上。

**A-RET（已接纳数据可取回，仅用于活性）。**
每份已被 Batch Inbox 接纳的批次 payload，在派生需要它时都可以取回：保留期 $T_{\rm ret}$ 之内由 DA-L2 保证，之后由 $\Pi$ 的保管义务保证，即至少一个诚实方保存它，直到它不再被任何派生或恢复需要。
违反 A-RET 时，派生停在该输入处等待，不会把未知内容当成空；依赖派生的强制包含（L2-L2）与 RESUME 也随之等待。安全不受影响。

### 2.6 治理方

**A-GOV（诚实治理方）。**

- **$\mathsf M$ 诚实。** 它按 $\mathcal Z$ 的指令及时签署并提交 Force。它只在紧急状态下（定义 4.3）发出 RESUME，并按以下规程选取 $h_{\rm res}$：
  - Espresso 已恢复；若故障在批次层，batcher 与 $\mathsf C$ 也已恢复；
  - $h_{\rm res}$ 高于 Batch Inbox 的当前 cursor，也高于全局链的末端；
  - $\mathit{ts}(L)+T_{\rm prep}\le t(h_{\rm res})\le\mathit{ts}(L)+D_{\rm drift}-\epsilon$，其中 $\epsilon>0$ 是余量。下界保证 $h_{\rm res}$ 高于 $L$ 处派生链的末端，并留出准备时间；上界保证新 session 的第一块满足 drift。
- **用到 A-GOV 的目标。**
  - 紧急恢复的活性：DG7 中超过 $W_{\rm exp}$ 的情形、DG8 的长停机、DG9 中超过 sequencing window 的情形；
  - “全局最终的块在非紧急情况下进入批次结算”（§3.2，§4.2 的 L2-L1）。在非紧急状态下发出的 RESUME 会撤销本可结算的全局链内容。
- **紧急恢复只有治理路径。** Espresso 长时间停机、批次层故障超过 sequencing window，或区域故障超过 $W_{\rm exp}$ 之后，只有 RESUME 能重新开出 session。

### 2.7 可信 certifier

**A-CERT。**
$\mathsf C$ 是一个可信的诚实参与方：从不被腐化，按 $\Pi$ 规定的 certifier 规则运行。由 A-CRYPTO 与 A-PKI，任何带有 $\mathsf C$ 签名的消息，都是 $\mathsf C$ 按规则实际产生的。
本稿不讨论 trusted setup 与远程认证。

$\mathsf C$ 可能离线；离线只影响活性。在线期间，$\mathsf C$ 对请求在 $T_{\rm cert}$ 内作出响应。
$\mathsf C$ 只在 L2 层使用：它核对批次与 $G$ 的对应关系及 DA 绑定。DRS 的安全目标 S1–S6 都不依赖 $\mathsf C$。
RESUME 之后全局链采纳派生链，所以 $\mathsf{DER}$ 块的内容来源继承 L2-S1 的性质，后者依赖 A-CERT（§4.3）。

### 2.8 中继与交付

**A-RELAY（仅用于活性）。**
对每项中继职责——向 Espresso 发布完整区域结果、提交批次、向 L1 提交证据——在相关时间窗内至少有一个在线诚实方承担。session 的诚实成员可以承担这些职责。

**A-DELIVER（交付预算，仅用于活性）。**
存在 $T_{\rm batch}$，使得：若在 $[\tau,\tau+T_{\rm batch}]$ 上 $\mathsf C$ 在线、DA 可用，且有诚实的注册 batcher 承担提交，则在 $\tau$ 进入全局链的块，其批次在 $\tau+T_{\rm batch}$ 之前被 Batch Inbox 接纳。
这一条不能从 A-RELAY 推出：恶意的 primary batcher 可以反复开 channel 再放弃，拖住接手者。$T_{\rm batch}$ 必须覆盖排队、接手、完成 channel、DA 认证与 L1 纳入。

### 2.9 诚实 leader 的频率

**A-LEAD（仅用于活性）。**
存在常数 $T_{\rm lead}$，使得：对每个被选中的良态 session $s$，以及 $\mathcal W_s$ 中任意长为 $T_{\rm lead}$ 的区间，区间内至少有一个 term 由 $s$ 的诚实 leader 主持，并且这个 term 完整落在区间内。

为简单起见，这条写成假设。它也可以在 A-BYZ 与按权重随机选 leader 的规则下，以概率形式推出。

### 2.10 参数与约束

| 参数 | 含义 | 计时起点 | 何时成立 | 是否用于 safety |
| --- | --- | --- | --- | --- |
| $\Delta_i$ | 区域内送达并处理 | 发送 | $s$ 的诚实成员之间，始终 | **是**（S4） |
| $\Delta_{\rm cross}$ | 跨区域送达并处理 | 发送 | 连通期间 | 否 |
| $\Delta_{\rm ts}$ | 时间戳与提出时刻的偏差 | — | 始终 | **是**（S6） |
| $\delta_{\rm slot}$ | L1 出块间隔上界 | — | 始终 | 否 |
| $T_{\rm ready}$ | 准备精确锚点的可执行状态 | 输入可得 | 大小与负载在声明范围内 | 否 |
| $T_{\rm L1}$ | L1 最终化并可见 | 提交或提出 | 始终 | **是**（S4：区域投票只接受足够旧的 origin） |
| $T_E$ | Espresso 最终化并可见 | 提交 | 可用期间 | **是**（S4：区域投票只接受在本地出现至少 $T_E$ 的 Espresso 引用） |
| $T_{\rm DA}$, $\Delta_{\rm DA}$, $T_{\rm ret}$ | DA 认证、取回、保留 | 提交、请求、认证 | 可用期间 | 否 |
| $T_{\rm cert}$ | $\mathsf C$ 的响应 | 请求 | $\mathsf C$ 在线期间 | 否 |
| $T_{\rm batch}$ | 批次交付预算 | 块进入全局链 | A-DELIVER 的条件下 | 否 |
| $T_{\rm lead}$ | 诚实 leader 出现的间隔 | — | 良态 session 内 | 否 |

**原生常数**（N1–N3）：出块间隔 $b_t$、sequencer drift $D_{\rm drift}$、sequencing window $W_{\rm seq}$。

**协议常数**：公告提前量 $M_a$、过期时长 $W_{\rm exp}$、证据窗口 $T_{\rm ev}$。它们须满足：

| 约束 | 作用 |
| --- | --- |
| $M_a\ge T_{\rm L1}+2\Delta_{\rm ts}+\delta_{\rm slot}$ | 正常情况下，$\mathsf{Replay}$ 处理事件时不必等待 L1 |
| $W_{\rm exp}+M_a+T_{\rm batch}+D_{\rm drift}\le W_{\rm seq}$ | 进入全局链的块在交付预算内能赶在截止前被接纳（§4.2 的 L2-L1）；$W_{\rm exp}$ 在此约束下取较大的值，使更长的停机可以无损继续 |
| $T_{\rm slash}\le T_{\rm ev}$ | 证据在质押释放前处理完毕 |
| $M_a+T_{\rm ready}\le T_{\rm prep}\le D_{\rm drift}-\epsilon$ | $\mathsf M$ 选取 $h_{\rm res}$ 的操作参数（A-GOV），合约不检查 |
| $D_{\rm drift}>T_{\rm L1}+\Delta_{\rm ts}$（留余量） | 按节奏出块时，存在既足够旧（区域投票的要求）、又满足 R-FRESH 的 origin |

由协议推导的时限在后续 Part 中给出：$T_{\rm live}$、$T_{\rm fi}$、$T_{\rm force}$、$T_{\rm pub}$、$T_{\rm pos}$、$T_{\rm sess}$、$T_{\rm slash}$、$T_{\rm release}$。

### 2.11 模型之外

以下情形不在本稿范围内，相关目标对此不作任何承诺：

- L1 的 safety 或 finality 失效（违反 L1-S）；
- Espresso 违反 E-S，例如出现冲突的 finalized 历史；
- $\mathsf C$ 被攻破；
- DA 违反 DA-S1。

A-TS 失效、A-BYZ 被违反、A-RET 被违反、$\mathsf M$ 被腐化，都在模型之内；§5.1 说明哪些目标在这些情况下仍然成立。

---

## 3. 去中心化区域轮换 sequencer（DRS）

### 3.1 定义

**定义 3.1（DRS）。** 去中心化区域轮换 sequencer 不是一个模块，而是以下模块的组合。各模块的接口与保证见 [Part 1 v0.3](part1_module_interfaces_zh_v0_3.md)。

1. **L1 Authority（质押层）：**
   - 负责成员注册、冻结快照、委员会与 leader 的确定、证据处理（slash）与质押释放；
   - 记录 $\mathsf M$ 的 Force 与 RESUME，并检查这些记录的格式与授权；
   - 作为 Batch Inbox 的 admin，只在 RESUME 时执行 cursor override，目标为 RESUME 给出的 $h_{\rm res}$。
2. **Session Program（共识层）：** 每个 session 在每个成员节点上的区域共识实例。诚实成员只为在父块上一定会被 L1 派生原样采用的块投票（派生一致投票）。
3. **Session Manager：** 管理 Program 的生命周期与跨区域交接，在节点上运行 Global Finality Rule，并在本地关闭已不可能进入全局链与派生链的 session（Kill）。
4. **Block Builder：** 交易入口与候选块构造；只在本区域负责时接受交易，不在区域之间转发交易。
5. **Publisher（发布层）：** 把完整的区域结果发布到 Espresso。
6. **Global Finality Rule（全局最终化规则，记作 $\mathsf{Replay}$）：** 任何观察者读取 L1 与 Espresso，独立计算全局链 $G$；只在处理 RESUME 时，还要按 L1 派生规则读取已接纳批次的 DA 数据。

DRS 以 Deposit Portal（原生的强制包含入口）为输入；其外部依赖是 L1、Espresso、通信与时钟，以及 RESUME 时的 DA。
Batcher、Batch Inbox、Certifier（$\mathsf C$）、DA、L1 派生与状态结算**不属于** DRS，它们属于 §4 的 L2 层。
$\mathsf{Replay}$ 在 RESUME 处调用 L2 层的派生规则 $\mathsf{Derive}$（§4.1），这是 DRS 对 L2 层唯一的读依赖。

**DRS 的接口。**

| 方向 | 接口 | 发起方 → 接收方 |
| --- | --- | --- |
| 输入 | $\textsf{SubmitTx}(tx)$：提交用户交易 | 客户端 → 负责区域的成员（由客户端或 RPC 网关路由） |
| 输入 | 强制包含交易 | 用户 → Deposit Portal |
| 输入 | 质押操作：加入、退出、提交证据等 | 任何人 → L1 Authority |
| 输入 | Force、RESUME | $\mathsf M$ → L1 Authority |
| 输出 | 全局链 $G^u(\tau)$ | 任意观察者运行 $\mathsf{Replay}$ |
| 输出 | 快速确认 $\mathsf{Conf}_v(s,B)$ | session $s$ 的成员 $v$ |

**确定性规则。** 下面两条规则的精确内容在后续 Part 中给出。

| 规则 | 输入 | 输出 |
| --- | --- | --- |
| $\mathsf{Exec}$（原生执行） | 前一状态、一个原生块的输入 | 下一状态，或“无效” |
| $\mathsf{Replay}$（全局最终化） | L1 日志、Espresso 日志；RESUME 处另需派生链 $D_L$ | 重算记录 $\mathrm{Rec}$，以及全局链 $G$ |

$\mathsf{Replay}$ 按**合并顺序**处理 Espresso 块、L1 事件（Force、RESUME）与时间事件（高度过期），细节见 Part 1 v0.3 §5。
Part 0 的目标只依赖以下事实：

- **确定性：** 合并顺序与各条子规则都是 L1 日志、Espresso 日志，以及 RESUME 处所读 DA 数据的确定函数。
- **接纳时限（R-EXP）：** 高度 $h$ 的普通块只能在 $\theta\le t(h)+W_{\rm exp}$ 的 Espresso 块中被接纳。若全局链的下一个高度过期时仍没有块，当前分支关闭，全局链停在它前面，直到下一次 RESUME。
- **授权引用（R-FRESH）：** 普通块的授权 L1 引用就是它的原生 origin $o(B)$，并满足 $\mathit{ts}(o(B))\ge t(h(B))-D_{\rm drift}$。
- **紧急恢复（R-RES）：** 处理 RESUME 记录时，$G$ 被替换为派生链 $D_L$ 直到 $h_{\rm res}$，新分支以 $D_L$ 在 $h_{\rm res}$ 处的块为锚点开出。$\mathsf{Replay}$ 不在其他任何地方生成或预测 deposit-only 块。

**定义 3.2（全局块与出处）。**
全局链 $G$ 是 genesis 之后的块序列。每个块 $B$ 带有出处

$$
\sigma(B)\in\{\mathsf{ORD}(s),\ \mathsf{DER}\}.
$$

- $\mathsf{ORD}(s)$：由 session $s$ 生产、经 $\mathsf{Replay}$ 接纳的普通块；
- $\mathsf{DER}$：RESUME 时从派生链 $D_L$ 采纳、且不在原全局链中的块。它是 deposit-only 块，或者是已接纳批次在实际父块上重新执行得到的块。

$\mathrm{native}(B)$ 表示 $B$ 对应的原生 L2 块。$G$ 上的前缀关系同时比较块与出处。
RESUME 时，$D_L$ 中与原全局链同一高度、同一原生块的块保留原来的出处。

**定义 3.3（观察者输出）。**
诚实观察者 $u$ 读取 L1 与 Espresso，并运行 $\mathsf{Replay}$；处理 RESUME 时，它按 L1 派生规则读取已接纳批次的 DA 数据。它在 $\tau$ 时的重算记录 $\mathrm{Rec}^u(\tau)$ 是已处理事件及其效果的序列；全局链 $G^u(\tau)$ 由 $\mathrm{Rec}^u(\tau)$ 确定。
这是一条结构要求（公开可重算）：输出不依赖任何私有输入、消息到达顺序，或有自由裁量权的服务。DA 数据是公开数据，取不到时观察者等待，不改变结果。

**定义 3.4（快速确认）。**
session $s$ 的诚实成员 $v$ 在 $\tau$ 时输出 $\mathsf{Conf}_v(s,B)$，表示 $v$ 在区域内提交（commit）了块 $B$。

**定义 3.5（锚点与关闭）。**
新分支的锚点只有两种来源：

- **Force：** 关闭目标分支，新分支以生效时的 $\mathrm{last}(G)$ 为锚点。此时 $G$ 可以领先 $D$。
- **RESUME：** 关闭当前分支，$G$ 被替换为 $D_L$ 直到 $h_{\rm res}$，新分支以 $D_L$ 在 $h_{\rm res}$ 处的块为锚点。

session 在以下情形被关闭：

- **Force 或 RESUME 生效：** 被关闭分支上的全部 session，位置由 $\mathsf{Replay}$ 确定。
- **过期：** 全局链的下一个高度过期而仍没有块时，当前分支上的全部 session，包括提前进入的常规后继；此后全局链停住，只有 RESUME 能重开。
- **Kill：** Session Manager 在本地关闭已不可能进入全局链与派生链的 session $s$，判据与上面的锚点规则一致：
  - **$G$ 领先 $D$ 时，$s$ 的链必须从 $\mathrm{last}(G)$ 派生。** 从 $\mathrm{last}(D)$ 派生、但不从 $\mathrm{last}(G)$ 派生的 session，不可能再进入 $G$；
  - **$D$ 已经定下与 $s$ 的链不相容的块时，$s$ 不可能再进入 $D$。** 例如某个高度的 sequencing window 结束，$D$ 在那里补了 deposit-only，而 $s$ 的链在那里需要普通块。此后只有 RESUME 能以 $D$ 在 $h_{\rm res}$ 处的块开出新 session。

  Kill 只停止本节点的区域工作，不改变 $G$。Espresso 停机时 $\mathsf{Replay}$ 暂停，第二种判据只依赖 L1 与 DA，照样起作用。精确判据见 Part 1 v0.3 §4.3.8。

### 3.2 DRS 的性质

安全性质刻画重算记录与全局链，活性性质刻画交易的包含；另外三条附加性质刻画快速确认与质押层。

**安全（S1、S2）。**

- **S1（一致性）。** 对任意诚实观察者 $u,v$ 与时刻 $\tau,\tau'$：$\mathrm{Rec}^u(\tau)\sim\mathrm{Rec}^v(\tau')$。
  因此，只要两份记录之间没有 RESUME 事件，就有 $G^u(\tau)\sim G^v(\tau')$。
- **S2（最终性）。** 若 $\tau\le\tau'$，则 $\mathrm{Rec}^u(\tau)\preceq\mathrm{Rec}^u(\tau')$。$G^u$ 只在末尾追加，唯一的例外是 RESUME：$G$ 被替换为 $D_L$ 直到 $h_{\rm res}$，原全局链在两者第一个不同的高度及之后的块被撤销。这一事件发生在所有诚实观察者记录中的同一位置，替换的结果也相同。

**“全局最终”的含义。** 称 $B$ 在 $\tau$ 时对观察者 $u$ 全局最终，若 $B\in G^u(\tau)$。它的含义如下：

1. **sequencer 层的常规动作不会撤销它。** 所有诚实观察者对 $B$ 所在位置的判定一致（S1）；轮换、Force、过期、区域故障都不会撤销 $B$。
2. **只有 RESUME 能撤销它。** 由 A-GOV，RESUME 只在紧急状态下发生；那时派生链已越过或偏离全局链，被撤销的只会是没能进入派生链的块。
3. **非紧急情况下，$B$ 以同一个块进入批次结算（§4.2 的 L2-L1）。** 进入批次结算之后，$B$ 不可撤销。

**真正不可撤销的最终是批次结算，即 $B$ 进入 $D$。**“全局最终”指 sequencer 层已经确定、在非紧急情况下必然结算。
我们**不**把“$G$ 严格不可撤销”作为目标，因为那会把 sequencer 层与批次层的假设耦合在一起。

**有效性（S3）。** 这一条对应 [CKPS01] 的 external validity。对任意诚实观察者 $u$ 与时刻 $\tau$：

- (a) **原生有效。** $\mathrm{native}(G^u(\tau))$ 是一条在 $\mathsf{Exec}$ 下从 genesis 状态出发的有效原生链，并满足 N2 等依赖 L1 的原生检查。
- (b) **授权。** 对每个 $\sigma(B)=\mathsf{ORD}(s)$ 的块：$s$ 是 $\mathsf{Replay}$ 在该位置指定的负责 session，并且 $B$ 带有 $s$ 的区域最终证书（直接证书，或经同 session 后代覆盖的证书）。
- (c) **派生一致。** 对每个 $\sigma(B)=\mathsf{DER}$ 的块：$\mathrm{native}(B)$ 等于采纳它的那次 RESUME 所用派生链 $D_L$ 在该高度的块。

**活性（L）。** 称 DRS 在条件 $C$ 下满足 $T$-活性，若 $C$ 在 $[\tau,\tau+T]$ 上成立时：

- **L-tx（用户交易，$T=T_{\rm live}$）。** 诚实客户端在 $\tau$ 时把交易 $tx$ 提交给当时负责 session 的一个在线诚实成员，并在负责 session 每次更换时，向新的负责 session 重新提交。若 $tx$ 从 $\tau$ 起持续有效，且负载不超过声明的容量，则对每个在该区间上活跃的诚实观察者 $u$，$tx$ 都出现在 $G^u(\tau+T_{\rm live})$ 中。负责的 session 由 $\mathsf{Replay}$ 公开可算。
- **L-fi（强制包含，$T=T_{\rm fi}$）。** 一笔经 Deposit Portal 提交的交易在 $\tau$ 时于 L1 finalized，则它按原生规则规定的位置，在 $\tau+T_{\rm fi}$ 之前出现在 $G^u$ 中。

“持续有效”指 $tx$ 相对从提交到包含之间的每个状态都有效，例如不存在同 nonce 的冲突交易。
L-fi 是抗审查性质：即使负责的 leader 是拜占庭的，N2 也迫使 L1 origin 随时间推进，因而迫使强制包含交易进入链中。

**附加性质。**

- **S4（快速确认一致）。** 设 $s$ 是良态 session。
  - (a) $s$ 的两个诚实成员不会在同一高度快速确认不同的块。
  - (b) 若 $s$ 的诚实成员输出 $\mathsf{Conf}_v(s,B)$，则对任意诚实观察者 $u$ 与时刻 $\tau$：只要 $G^u(\tau)$ 中有出处为 $\mathsf{ORD}(s)$、高度为 $h(B)$ 的块，那个块就是 $B$。

  快速确认可以因 Force、过期、Kill 或 RESUME 而失效；S4 不排除这些情况。
- **S5（问责）。**
  - (a) **无错误惩罚。** 若某个 position 的密钥直到释放之前从未被腐化，它不会被 slash。
  - (b) **不重复处罚。** 同一次违规实例至多被处罚一次。
- **S6（质押安全）。** 若 position $o$ 在 $\tau$ 时被释放，则对每个满足 $o\in V_s$ 的 session $s$，都有

  $$
  \tau_s^{\rm close}+T_{\rm ev}\le\tau .
  $$

  解锁时间由 L1 Authority 按时间上界直接计算，依据是 R-EXP 与 R-FRESH 给出的 $\tau_s^{\rm close}$ 上界；L1 不需要读取 Espresso。

### 3.3 DRS 的设计目标：不同假设下的安全与活性

设计目标写成“在假设集 $\mathcal H$ 下，DRS 满足性质 $P$”。下表最后一列列出**不需要**的假设，它们和所需假设同样重要。

| 目标 | 性质 | 所需假设 | 不需要 |
| --- | --- | --- | --- |
| DG1 | S1 一致性、S2 最终性 | L1-S, E-S, DA-S1；协议规则与派生规则的确定性 | A-BYZ；A-GOV；$\mathsf C$；网络同步；A-TS；Espresso 可用 |
| DG2 | S3 有效性 | DG1 的假设 + A-CRYPTO + A-BYZ | 成员在线；网络同步；A-GOV |
| DG3 | S4 快速确认一致 | DG2 的假设 + $s$ 良态（含 $\Delta_i$）+ A-TS、L1-L3 与 Espresso 的 X-L2（区域投票只接受足够旧的引用） | A-GOV |
| DG4 | S5 问责 | A-CRYPTO, A-PKI, L1-S，持久化存储 | A-GOV；网络同步 |
| DG5 | S6 质押安全 | A-CRYPTO, L1-S, E-S, A-TS；规则 R-EXP 与 R-FRESH | $\mathsf C$；A-GOV；A-BYZ |
| DG6 | 正常运行的活性（L-tx、L-fi） | 见下文 | A-GOV（只要 $\mathsf M$ 不发控制） |
| DG7 | 区域故障后恢复活性 | $W_{\rm exp}$ 内：A-GOV（Force）；超过：同 DG8 的长停机 | $\mathsf C$ |
| DG8 | Espresso 停机后恢复活性 | Espresso 恢复；长停机另需 A-GOV 与 A-RET | $\mathsf C$ 参与恢复决策 |
| DG9 | 批次层故障后恢复活性 | 批次层恢复；超过 sequencing window 时另需 A-GOV 与 A-RET | — |

**DG6（正常运行的活性）。** DRS 满足 $T_{\rm live}$-活性与 $T_{\rm fi}$-活性，条件为以下各项同时成立：

1. 没有控制生效，全局链也没有因过期而停住；当前 policy 中至少有一个 ON 区域的权重非零；
2. Espresso 排序可用且读取可用（E-L）；L1-L；A-TS；
3. 区间内被选中的 session 都是良态的，且它们之间交接所需的跨区域连接是连通的；
4. A-LEAD；
5. A-RELAY 成立；
6. DG2 的假设成立，因为活性要求被包含的交易有效。

**DG7（区域故障）。** 只有治理路径。

- **在 $W_{\rm exp}$ 内 Force。** 设 $\mathcal Z$ 在 $\tau_0$ 时指示诚实的 $\mathsf M$ 发出 Force，且 Force 在负责高度过期之前生效。新分支以 $\mathrm{last}(G)$ 为锚点，新 session 追赶缺口，全局链不丢失任何高度。若 DG6 的条件对按新 policy 选出的 session 成立，则到 $\tau_0+T_{\rm force}$ 时 Force 已经生效，此后 DRS 恢复活性。
- **超过 $W_{\rm exp}$。** 负责高度过期，当前分支关闭，全局链停住；之后按 DG8 的长停机，由 RESUME 恢复。

两种情形下，强制包含都经 L1 派生链照常有效（L2-L2）。

**DG8（Espresso 停机）。**
停机期间 $\mathsf{Replay}$ 暂停；batcher 继续提交全局链中已有的块，L1 派生的原生回退继续推进，强制包含照常有效（L2-L2）。按停机时长分两种情形：

1. **短停机（停机时长短于 $W_{\rm exp}$ 减去余量）。** 原 session 恢复后继续出块，追上停机期间的高度，不丢失内容，不需要 $\mathsf M$。
2. **长停机（不短于 $W_{\rm exp}$）。**
   - Espresso 恢复后，$\mathsf{Replay}$ 处理到第一个过期高度，关闭当前分支，全局链停住；
   - 该高度的 sequencing window 结束后，L1 派生链用 deposit-only 越过全局链，系统进入紧急状态；各 Session Manager 由此 Kill 全部旧 session，这一步在停机期间就会发生；
   - 诚实的 $\mathsf M$ 在 Espresso 恢复后发出 RESUME($h_{\rm res}$)：L1 派生把空缺填成 deposit-only 直到 $h_{\rm res}$，$G$ 被替换为 $D_L$，新分支从 $D_L$ 在 $h_{\rm res}$ 处的块出块；
   - 最早可恢复的时间约为停机开始加上 $W_{\rm seq}$。

**DG9（批次层故障）。** batcher、DA 或 $\mathsf C$ 不能及时交付，而 Espresso 与区域正常：

1. **在 sequencing window 内恢复。** 从 cursor 接着补交；每个高度都在截止前被接纳，全局链的块原样进入派生链，无损。
2. **超过 sequencing window。**
   - 第一个未交付高度的窗口结束后，L1 派生在该高度补 deposit-only，与全局链分叉，系统进入紧急状态；
   - 各 Session Manager 由此 Kill 全部旧 session，全局链停止增长；
   - 批次层恢复后，诚实的 $\mathsf M$ 发出 RESUME($h_{\rm res}$)：$G$ 被替换为 $D_L$，分叉点之后的全局链内容被撤销，新分支从 $D_L$ 在 $h_{\rm res}$ 处的块出块。

**从这张表可以读出五条结构性结论：**

1. **一致性与最终性（DG1）只依赖两本公共账本与 DA 绑定的 safety。** DA 绑定只在 RESUME 处用到。它不依赖任何区域的诚实、$\mathsf M$、$\mathsf C$、网络同步、A-TS 或 Espresso 可用。
2. **“全局最终”只可能被 RESUME 撤销，而 RESUME 只在紧急状态下发生。** 被撤销的只会是没能进入派生链的块。真正不可撤销的最终是批次结算。
3. **有效性（DG2）只多依赖拜占庭界 A-BYZ，不依赖成员在线。**
4. **快速确认（DG3）是唯一依赖区域同步 $\Delta_i$ 的 safety 目标；它与质押安全（DG5）都依赖 A-TS：DG3 用它判断引用是否足够旧，DG5 用它把时间戳换成真实时间。**
5. **$\mathsf M$ 与 $\mathsf C$ 都不在 S1–S6 的依赖中。** $\mathsf M$ 用于紧急恢复，以及“全局最终的块在非紧急情况下进入批次结算”的保证。

---

## 4. 以 DRS 为子模块的 L2

### 4.1 组成与输出

L2（rollup）由 DRS 加上结算路径组成：

- **DRS**（§3），输出全局链 $G$；
- **batcher：** 只把 $G$ 中的普通块编码成批次，经 DA 分发，并提交到 Batch Inbox；不为 deposit-only 高度提交任何批次；
- **可信 certifier $\mathsf C$：** 核对批次与 $G$ 的对应关系及 DA 绑定，签名后由 Batch Inbox 接纳；
- **Batch Inbox：** 按 cursor 连续接纳带 $\mathsf C$ 签名的批次的 L1 合约；admin 为 L1 Authority，cursor override 只在 RESUME 中执行；
- **DA**（§2.5）；
- **L1 派生规则 $\mathsf{Derive}$：** 现有的 mega-espresso 派生，保留 sequencing window 回退与 cursor override 的填充规则（方案 A）；$\mathsf{Replay}$ 在 RESUME 处调用同一个 $\mathsf{Derive}$；
- **状态结算规则：** 现有的 Kailua / fault-proof 路径，使用同一个 $\mathsf{Derive}$。

| 规则 | 输入 | 输出 |
| --- | --- | --- |
| $\mathsf{Derive}$（L1 派生） | L1 日志、已取回的 DA 数据 | 原生块序列 |

**定义 4.1（L1 派生链）。**
诚实派生节点 $u$ 在 $\tau$ 时的输出为

$$
D^u(\tau):=\mathsf{Derive}\bigl(\mathrm{LOG}^{\rm L1}_u(\tau),\ u\text{ 已取回的 DA 数据}\bigr).
$$

$D$ 已派生的前缀不会再改变：派生只读已最终化的 L1，规则确定，DA 绑定字节。

**定义 4.2（已接纳前缀 $G'$ 与 RESUME 处的派生链 $D_L$）。**

- $G'$ 是 Batch Inbox 的 cursor 以下的高度前缀。$\mathsf C$ 只为与 $G$ 一致的批次签名，所以在两次 RESUME 之间 $G'\preceq G$；RESUME 之前签发的证书在 override 之后不再被接纳（Part 1 v0.3 的 O-5）。
- 设 RESUME 记录在 L1 块 $L$ 中。$D_L$ 是按 L1 顺序处理到这笔交易的 `CursorSet` 为止的派生链：先处理此前已接纳的全部输入，再把空缺立即填成 deposit-only，直到 $h_{\rm res}$，填充块的 origin 不超过 $L$。override 不改写此前已派生的前缀。
- $D_L$ 是已最终化 L1 前缀与 DA 数据的确定函数：所有诚实节点得到同一个 $D_L$，取不到数据时等待（A-RET）。

**定义 4.3（紧急状态）。**
称 L1 高度 $\ell$ 处于紧急状态，若 Batch Inbox 的 cursor 已在一个完整的 sequencing window 内没有前进：

```text
ℓ > cursor.epoch_num + seq_window_size + 1
```

其中 `cursor.epoch_num` 是 cursor 末块的 L1 origin。
此时 cursor 之后的第一个高度在派生中一定已被补成 deposit-only：派生链已越过全局链（全局链不再增长时），或已与它分叉（批次层故障时）；cursor 之上的全局链内容都不可能原样进入 $D$。
这个条件只读 L1，任何人都能检查。精确边界由派生规则决定（Part 1 v0.3 §3.5）。

### 4.2 三层确认与 L2 的性质

一个块依次经过三层：

1. **区域最终：** session 的诚实成员给出快速确认；
2. **全局最终：** 进入 $G$，即 sequencer 层的最终（§3.2）；在紧急状态下可被 RESUME 撤销；
3. **批次结算：** 批次在截止前被 L1 接纳，块进入 $D$；这是真正不可撤销的最终。

L2 的性质如下：

- **L2-S1（内容或 deposit-only）。** 对任意高度 $h$，L1 派生链在 $h$ 处的块只有两种可能：
  - 它的普通交易列表等于 $G$ 在 $h$ 处普通块的交易列表；
  - 它是一个 deposit-only 块。

  不会出现第三种普通内容。这是 mega-espresso Certification Claim 在轮换版本中的对应。
- **L2-S2（派生一致）。** 对任意诚实派生节点 $u,v$：$D^u(\tau)\sim D^v(\tau')$，且 $D^u$ 随时间只增长。
- **L2-S3（与 $G$ 对应）。** 在 L2-L1 的条件下、且没有 RESUME 的执行中，$\mathrm{native}(G^u(\tau))\sim D^v(\tau')$。一般情况下，分歧在下一次 RESUME 处消除：$G$ 被替换为 $D_L$。
- **L2-L1（全局链进入派生链）。** 设 $B\in G^u(\tau_0)$ 为普通块，高度为 $h$。若
  1. 生产 $B$ 及其祖先的 session 满足 A-BYZ，诚实成员执行派生一致投票；
  2. 协议常数满足 $W_{\rm exp}+M_a+T_{\rm batch}+D_{\rm drift}\le W_{\rm seq}$；
  3. 在 $[\tau_0,\tau_0+T_{\rm batch}]$ 上 A-DELIVER 的条件成立，DA 可用；
  4. A-GOV 成立，期间没有 RESUME 越过 $h$；

  则对每个活跃的诚实派生节点 $v$，$D^v(\tau_0+T_{\rm batch})$ 在高度 $h$ 处的块就是 $\mathrm{native}(B)$。这里要求的是同一个块，即同一 hash、同一执行状态，而不只是同一交易列表。
  证明对高度归纳：每个 session 的锚点都在 $D$ 中（Force 的锚点按归纳在 $D$ 中，RESUME 的锚点就是 $D$ 的块）；若 $D$ 与 $G$ 在 $h-1$ 处是同一个块，由条件 2、3，$B$ 的批次在截止前被接纳，派生在同一父块上处理它；由条件 1，各项检查都通过，派生构造的 deposit 前缀与区域块相同，得到同一个块（Part 1 v0.3 §8.4）。
- **L2-L2（L1 上的强制包含）。** 一笔经 Deposit Portal 提交的交易在 $\tau$ 时于 L1 finalized，则它在约 $\tau+W_{\rm seq}+T_{\rm L1}$ 之前出现在 $D^v$ 中。这一条只依赖 L1-L 与 A-RET，不依赖 sequencer、Espresso、$\mathsf M$ 或 $\mathsf C$，是方案 A 保留下来的经典 rollup 逃生口。

L2 继承 DRS 的全部性质。

### 4.3 L2 的设计目标

| 性质 | 所需假设 | 不需要 |
| --- | --- | --- |
| L2-S1 | A-CERT, DA-S1, L1-S, DG2 的假设 | 任何时限；A-GOV |
| L2-S2 | L1-S, DA-S1，$\mathsf{Derive}$ 的确定性 | 任何时限；DRS |
| L2-S3（无 RESUME 的形式） | DG1 的假设 + L2-L1 的条件 | — |
| L2-L1 | 派生一致投票、A-BYZ、§2.10 的常数约束；A-DELIVER, DA-L, L1-L；A-GOV | Espresso 在 $B$ 进入 $G$ 之后可用 |
| L2-L2 | L1-L, A-RET | DRS、Espresso、A-GOV、$\mathsf C$ |

### 4.4 三级确认

| 级别 | 含义 | 由谁给出 | 延迟量级 | 可被什么推翻 |
| --- | --- | --- | --- | --- |
| 区域最终 | $\mathsf{Conf}_v(s,B)$ | $s$ 的诚实成员 | 若干 $\Delta_i$ | Force 在 $B$ 进入 $G$ 之前生效；$B$ 过期；Kill；RESUME；$s$ 不良态 |
| 全局最终 | $B\in G^u(\tau)$ | 任意诚实观察者 | 发布 + Espresso | 仅 RESUME（紧急状态） |
| 批次结算 | $\mathrm{native}(B)\in D^u(\tau)$ | 任意诚实派生节点 | 批次 + DA + L1 | 不可推翻 |

---

## 5. 故障场景、非目标与设计决定

### 5.1 各故障场景下的含义

| 场景 | DRS 安全 | DRS 活性 | L2 |
| --- | --- | --- | --- |
| 某 session 有少数拜占庭成员（良态） | 全部 | 继续 | 正常 |
| 整区离线（A-BYZ 成立，不良态） | S1–S3, S5, S6；该 session 的 S4 不保证 | $W_{\rm exp}$ 内 Force；超过则全局链停住，进入紧急状态后 RESUME | Force 时正常；RESUME 时空缺高度为 deposit-only，被全局链采纳 |
| 跨区域网络分区 | 全部 | 恢复连通后继续交接 | 正常 |
| Espresso 短停机（短于 $W_{\rm exp}$） | 全部 | 恢复后原 session 追赶，无内容丢失 | 正常 |
| Espresso 长停机 | 全部；全局链停住，RESUME 时被替换为派生链 | Kill 旧 session；RESUME 后恢复 | 原生回退继续推进，强制包含照常 |
| batcher、DA 或 $\mathsf C$ 故障，未超过 sequencing window | 全部 | 不受影响 | 恢复后补交，无损 |
| batcher、DA 或 $\mathsf C$ 故障，超过 sequencing window | 全部；分叉点之后的全局链内容在 RESUME 时被撤销 | Kill 旧 session；RESUME 后恢复 | 受影响的高度被 deposit-only 填充 |
| $\mathsf M$ 被腐化（违反 A-GOV） | S1–S6 仍成立 | 可被反复 Force；可以拒绝恢复 | 可在非紧急状态下 RESUME，撤销尚未结算的全局链内容；派生链已派生的前缀不受影响 |
| 违反 A-RET | 全部 | 派生与 RESUME 停在取不回的输入处等待 | 同左；强制包含也随之等待 |
| 违反 A-TS | S1–S5；S6 不保证 | $\mathsf{Replay}$ 可能暂停，直到 L1 日志追上 | — |
| 违反 A-BYZ | S1、S2 仍成立；S3、S4 不保证 | — | L2-S1 不保证 |
| L1 或 Espresso 的 safety 失效、$\mathsf C$ 被攻破、违反 DA-S1 | 不作承诺（模型之外） | — | — |

### 5.2 非目标

- 超出 SMR 活性与强制包含的抗审查，例如对拜占庭 leader 的即时包含保证或时间公平。
- 区域之间、leader 之间的公平，以及排序公平或 MEV 相关性质。
- 自动检测 Espresso 停机或区域故障。
- 无治理的紧急恢复。
- $G$ 的严格不可撤销（§3.2）。
- 紧急状态下已被撤销的全局链内容的恢复：它们的交易需要用户重新提交。
- Accountable safety，即安全被破坏时必能指认一定权重的违规者。
- 消息机密性与交易隐私。
- 密码学意义上的可组合性（见 §0.1）。

### 5.3 设计决定

| 编号 | 决定 |
| --- | --- |
| D1 | 保留快速的全局最终化：Force 的公告提前量由合约检查，观察者处理事件前等待相应的 L1 与 Espresso 数据。 |
| D2 | $\mathsf M$ 建模为诚实（A-GOV）。S1–S6 不依赖 $\mathsf M$；紧急恢复与“全局最终 → 批次结算”依赖 $\mathsf M$。 |
| D3 | 方案 A：不改派生规则，确认分三层。“全局最终”在紧急状态下可被 RESUME 撤销，批次结算是真正不可撤销的最终。 |
| D4 | Force 以生效时的 $\mathrm{last}(G)$ 为锚点。 |
| D5 | 以 D 为准的恢复：$\mathsf{Replay}$ 不预测 deposit-only 块；紧急状态下由 RESUME($h_{\rm res}$) 把 $G$ 替换为 $D_L$，新分支以 $D_L$ 在 $h_{\rm res}$ 处的块为锚点。 |
| D6 | $\mathsf{Replay}$ 在 RESUME 处经派生规则读取 DA；DG1 依赖 DA-S1。 |
| D7 | 全局链的下一个高度过期后，当前分支关闭、全局链停住，直到 RESUME；$W_{\rm exp}$ 在 §2.10 的约束下取较大的值。 |
| D8 | 派生一致投票：诚实成员只为在父块上一定被派生原样采用的块投票。 |
| D9 | Session Manager 在本地 Kill 已不可能进入全局链与派生链的 session，判据与锚点规则一致（定义 3.5）。 |
| D10 | 授权 L1 引用就是块的原生 origin；R-FRESH 的新鲜度界取 $D_{\rm drift}$。 |
| D11 | 用时间上界解锁质押（R-EXP、R-FRESH），L1 不读取 Espresso。 |
| D12 | Batch Inbox 的 admin 为 L1 Authority，cursor override 只经由 RESUME 执行。 |
| D13 | Espresso 的可用性分为排序与读取。 |
| D14 | 诚实 leader 的频率写成假设 A-LEAD。 |
| D15 | 交易转发不放进协议：Builder 只在本区域负责时接受交易，客户端在负责 session 更换时重新提交。 |
| D16 | 命名：Deposit Portal（强制包含入口）与 Batch Inbox（批次收件箱）。 |

### 5.4 待定问题

1. **派生一致投票的条件清单。** 须与派生规则逐条核对：origin 连续、drift（空块也不例外）、deposit 前缀、执行有效性，并对 L1 视图的扩展稳健（Part 6）。
2. **紧急状态判断的精确边界。** 定义 4.3 中的“+1”须对照原生 batch queue 核对（Part 6）。
3. **Certifier 的签名时机。** $\mathsf C$ 签名所用的全局链位置不早于 prev cursor 的写入位置，以保证 RESUME 之后 $G'\preceq G$（Part 1 v0.3 §4.7.1，待确认）。
4. **手动 override 的场景。** mega-espresso 中依赖 admin override 的场景（publisher 锚点超前、无效记录、override 位置不可达），在只有 RESUME 一个 override 入口时如何处理（Part 6）。
5. **推导时限。** $T_{\rm live}$、$T_{\rm fi}$、$T_{\rm force}$、$T_{\rm pub}$、$T_{\rm pos}$、$T_{\rm sess}$、$T_{\rm slash}$、$T_{\rm release}$ 的具体表达式（各模块的 Part）。

---

## 参考文献

- [GKL15] J. Garay, A. Kiayias, N. Leonardos. *The Bitcoin Backbone Protocol: Analysis and Applications.* EUROCRYPT 2015.
- [PS17] R. Pass, E. Shi. *The Sleepy Model of Consensus.* ASIACRYPT 2017.
- [AMNRY20] I. Abraham, D. Malkhi, K. Nayak, L. Ren, M. Yin. *Sync HotStuff: Simple and Practical Synchronous State Machine Replication.* IEEE S&P 2020.
- [CKPS01] C. Cachin, K. Kursawe, F. Petzold, V. Shoup. *Secure and Efficient Asynchronous Broadcast Protocols.* CRYPTO 2001（external validity）。
- mega-espresso 规格：`megaeth_specs/specs/block-certification/`（overview、block-records、batch-inbox、derivation-pipeline、full-node、recovery、activation）与 `megaeth_specs/specs/rollup/derivation.md`、`megaeth_specs/specs/rollup/data-availability.md`。
