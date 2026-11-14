# MegaETH Sequencer Rotation 技术报告 · Part 0

## 模型、假设与设计目标（中文工作稿 v0.3）

日期：2026-10-25。
状态：迭代草稿。本稿写四件事：(1) 系统模型；(2) 假设与外部组件；(3) 去中心化区域轮换 sequencer 的定义、性质与设计目标；(4) 以它为子模块的 L2。
各模块的内部设计在 Part 0 定稿后再写。
上一版：[v0.2](part0_model_goals_zh_v0_2.md)。

**相对 v0.2 的变化**

1. **治理方只影响活性（D2）。** $\mathsf M$ 可以被腐化，所有 safety 目标对任意 $\mathsf M$ 行为成立。控制纪律改由 L1 合约与重算规则强制。
2. **D1 的落实方式。** 保留基于时间戳间隔的快速最终化，同时增加一条观察者等待规则，使全局链的一致性不再依赖时间界；正常情况下延迟不变（§5.3，请确认）。L1 增加“出块与最终化节奏”性质 L1-L3。
3. **§3 重写。** 先定义去中心化区域轮换 sequencer（DRS）：质押层、L1 控制收件箱、共识层、发布层、全局最终化规则。它的性质是标准 SMR 的安全与活性；然后按不同假设给出设计目标。
4. **新增 §4。** 把 batcher、可信 certifier、DA、L1 派生放进以 DRS 为子模块的 L2 层；L1 对应（原 S4）等性质移到这里。
5. **记录已定的设计决定 D1、D2、D4、D5、D6；** 新增待讨论 D7–D9；D3 给出 A/B 的详细比较。
6. **活性改为标准 SMR 活性。** 包括用户交易与 L1 deposit 的有界包含；“交易包含”不再列为非目标。

---

## 0. 写法与约定

### 0.1 形式化方式

本稿采用共识文献中常见的 property-based 写法 [GKL15, PS17]。

1. **执行。**
   一次执行由环境 $\mathcal Z$、敌手 $\mathcal A$ 与协议 $\Pi$ 组成，记作 $\mathrm{EXEC}_{\Pi,\mathcal A,\mathcal Z}(\kappa)$。$\mathcal Z$ 提供全部外部输入；$\mathcal A$ 控制被腐化的参与方，并在假设允许的范围内决定消息延迟。
2. **外部系统写成假设。**
   L1、Espresso、DA 不写成理想功能。对每个外部系统，我们只给出必要的使用接口，以及协议所依赖的性质。协议写入这些系统的数据直接继承这些性质。
3. **内部计算写成规则。**
   原生执行、全局重算、L1 派生、状态结算等，日后写成所有人可以独立计算的确定性规则。它们既不是参与方，也不是功能。
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
| $h$ | L2 高度。 |
| $\mathcal R$；$o,pk_o,i_o$ | 区域集合；质押 position、其公钥与所属区域。 |
| $s$；$V_s,w_o,W_s,F_s$ | session；其委员会、成员权重、总权重、故障门槛（§1.3）。 |
| $\mathcal W_s=[\tau_s^{\rm open},\tau_s^{\rm close}]$ | session $s$ 的职责窗口（§1.3）。 |
| $\mathsf M$；$\mathsf C$ | 治理方（multisig）；可信 certifier。 |
| $\mathrm{LOG}^X_P(\tau)$ | 诚实参与方 $P$ 在 $\tau$ 时看到的账本 $X$ 的已最终化日志（§2.4）。 |
| $X\preceq Y$ | 序列 $X$ 是 $Y$ 的前缀。 |
| $X\sim Y$ | $X\preceq Y$ 或 $Y\preceq X$（可比较）。 |
| $G^u(\tau)$ | 诚实观察者 $u$ 在 $\tau$ 时输出的全局链（§3.1）。 |
| $D^u(\tau)$ | 诚实派生节点 $u$ 在 $\tau$ 时由 L1 与 DA 派生出的原生链（§4.1）。 |

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
- **治理方 $\mathsf M$。** multisig 委员会，作为**一个**逻辑参与方建模。它可以被腐化，但只能影响活性（A-GOV）。
- **可信 certifier $\mathsf C$。** 原先由 TEE 承担的 certifier，作为**一个**可信的诚实参与方建模（A-CERT）。它属于 L2 层（§4），不属于 DRS（§3）。
- **角色。** 用户、publisher、batcher、观察者、证据提交者都是角色而非身份。任何参与方，包括被腐化的参与方，都可以承担这些角色；一个运营者可以兼任多个角色。
- **L1 合约不是参与方。** 质押层、控制收件箱、batch inbox 等是部署在 L1 上的确定性程序，属于 $\Pi$ 的一部分。

环境 $\mathcal Z$ 提供全部外部输入：用户交易、质押请求，以及交给 $\mathsf M$ 的**治理指令**。
$\mathcal Z$ 与 $\mathcal A$ 可以任意通信。

### 1.2 时间

全局时钟给出真实时间 $\tau\in\mathbb R_{\ge 0}$，所有参与方读到同一 $\tau$。
有界时钟偏差可以按标准做法并入下文的 $\Delta$。

本稿区分四种坐标，它们彼此不能互换：

1. 真实时间 $\tau$；
2. L1 高度 $\ell$ 及时间戳 $\mathit{ts}_\ell$；
3. Espresso 高度 $q$ 及时间戳 $\theta_q$；
4. L2 高度 $h$。

两条链的时间戳都近似真实时间。对 L1 或 Espresso 的任一块 $b$，记 $\tau_{\rm prop}(b)$ 为 $b$ 被提出（产生）的真实时间。

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
对 Ethereum 而言，提出到 finalized 的时间可达十余分钟，所以 $\mathit{ts}(b)$ 接近提出时刻，而不接近 finalized 时刻。
若采用 §5.3 中 D1 的落实方式，A-TS 只用于活性。

### 1.3 session 与职责窗口

为了陈述假设与目标，这里只引入最少的 session 概念。session 怎样选出、怎样交接，属于后续模块。

**定义 1.1（session）。**
DRS 把普通出块划分为 session。每个 session $s$ 确定：

- 区域 $i_s\in\mathcal R$ 与固定委员会 $V_s\subseteq\{o: i_o=i_s\}$；
- 每个成员的正整数权重 $w_o$，取自某个冻结的质押快照；总权重 $W_s=\sum_{o\in V_s}w_o$；
- 故障门槛 $F_s$，满足 $0\le F_s<W_s/2$。

$s$ 的**证书**是来自 $V_s$ 中不同成员、总权重严格大于 $F_s$ 的一组匹配签名。
这些字段一经确定便不再改变；之后的 slash 或新快照都不改写它们。

**定义 1.2（职责窗口）。** session $s$ 的职责窗口 $\mathcal W_s=[\tau_s^{\rm open},\tau_s^{\rm close}]$ 由两个时刻界定。

- **开始 $\tau_s^{\rm open}$：** 首次有参与方能够算出 $V_s$ 的时刻。从这时起，$V_s$ 的成员就可能被要求为 $s$ 做准备或签名。这个时刻可能早于 $s$ 开始出块，也可能早于相关数据进入全局链。
- **结束 $\tau_s^{\rm close}$：** L1 或 Espresso 上首次有已最终化的数据确定“$s$ 再也不能向全局链追加任何块”的时刻。有两种情况：
  1. $s$ 的最后一块已经进入全局链；
  2. 覆盖 $s$ 的治理关闭（Force 或 STOP）已经生效。

  精确判定由全局最终化规则 $\mathsf{Replay}$ 给出。

职责窗口在本稿中有三个用途：

- **拜占庭计数（A-BYZ）。** 只计算在 $\tau_s^{\rm close}$ 之前被腐化的成员。窗口结束后才泄露的密钥，已经不能再改变全局链。
- **诚实的含义（§1.4）。** $s$ 的诚实成员须在整个窗口内在线并保持同步。
- **质押释放（S6）。** 只有在 $\tau_s^{\rm close}$ 再加上证据窗口之后，$s$ 的成员才能释放质押。

**例子。** 设区域为 A、B、C，每个 session 的 DATA 配额为 10，当前分支版本为 7。

| 场景 | 发生了什么 | 窗口的开始与结束 |
| --- | --- | --- |
| 开始（正常） | $s_1=(7,20)$ 在区域 A，从 anchor 100（前驱的末块）出发，生产 101–110。$s_2=(7,21)$ 在区域 B，是 $s_1$ 的常规后继。 | $s_2$ 的委员会由 $s_1$ 的 anchor 100 与来源区域 A 决定。所以块 100 一得到区域 FINAL，任何人都能算出 $V_{s_2}$：$\tau^{\rm open}_{s_2}$ 就是这一时刻，比 $s_2$ 开始出块早了大约一个 session 的时长。同理，$\tau^{\rm open}_{s_1}$ 是 $s_1$ 的前驱的 anchor 被确定的时刻。 |
| 正常结束 | 块 110 在 Espresso 块 $J_q$ 的重算中进入全局链。 | $\tau^{\rm close}_{s_1}=\tau_E(q)$，即 $J_q$ 最终化的时刻。此后即使 $s_1$ 成员的密钥泄露，也不能再改变全局链。 |
| Force | 全局链停在 108 时，Force 在 Espresso 块 $J_{q'}$ 生效，关闭分支 7。 | 分支 7 上所有 session（$s_1$、$s_2$，以及更远的推测性后继）的窗口都在 $\tau_E(q')$ 结束。$s_1$ 的 109、110 即使已有区域 FINAL，也不再进入全局链。新分支 8 的第一个 session 从 108 出发，其 $\tau^{\rm open}=\tau_E(q')$。 |
| STOP | $s_2$ 出块期间 Espresso 停机，全局链停在 115。 | Espresso 不再产生新块，无法由它确定 $s_2$ 的结束。STOP 在 L1 上生效的时刻，就是 $\tau^{\rm close}_{s_2}$（截断点的来源见 D7）。若一直没有 STOP，则 $\tau^{\rm close}_{s_2}=\infty$。 |

### 1.4 腐化、诚实与离线

**腐化模型。**
敌手可以**自适应地**在任意时刻腐化任意参与方，立即取得其全部状态，包括私钥与历史状态（不假设安全擦除），此后完全控制它。腐化不可撤销。
$\mathsf C$ 不可被腐化（A-CERT）。$\mathsf M$ 可以被腐化；safety 目标不依赖 $\mathsf M$（A-GOV）。

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

良态是关于一次执行的**谓词**，不是全局假设。
区域整体离线或区域网络中断时，相应 session 不良态，但这仍在模型之内（A-BYZ 依然可以成立）。

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

- **本地计算视为瞬时，有一个例外。** 为给定 anchor 准备可直接执行的状态（下载、重建、验证、安装）需要时间。在所需输入可得、大小与负载在声明范围内时，这一步不超过 $T_{\rm ready}$。
- **费用。** 诚实方持有足够的资金，用以支付其职责所需的 L1、Espresso 与 DA 费用。

---

## 2. 假设与外部组件

### 2.1 密码学

**A-CRYPTO。**
签名方案满足 EUF-CMA 安全；哈希函数抗碰撞；承诺具有绑定性。敌手是 PPT 的。

### 2.2 PKI

**A-PKI。**
成员公钥经 L1 上的质押层注册，并附 proof of possession；诚实方诚实地生成密钥。
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
- **L1-S5（时间戳）。** 时间戳沿链严格递增，并满足 A-TS。
- **L1-L3（出块与最终化节奏）。** 存在 $\delta_{\rm slot}$，使任意长为 $\delta_{\rm slot}$ 的区间内，都有一个最终会被 finalized 的 L1 块被提出。任何最终被 finalized 的块 $b$，在 $\tau_{\rm prop}(b)+T_{\rm L1}$ 之前出现在所有在该区间上活跃的诚实方的日志中。

$T_{\rm L1}$ 覆盖从提交（或提出）到 finalized、再到对活跃诚实方可见的全过程。

**A-ESP（Espresso）。**
Espresso 满足 X-S1 至 X-S3（记为 E-S1 至 E-S3）。其活性性质只在可用期间成立，参数为 $T_X=T_E$。此外：

- **Namespace。** publication 带 namespace 标签。读取时，每块按顺序给出本链 namespace 的完整内容 $M_q$，可以为空。Espresso 只负责排序，不验证本协议的任何内容；本链 namespace 中的数据可以由任何人提交，包括敌手。
- **两种可用性。**
  - **排序可用：** 新 publication 能被收录并最终化。X-L1 只在排序可用期间成立。
  - **读取可用：** 已最终化的块能被读取。X-L2 只在读取可用期间成立。

  **停机**定义为排序不可用。停机期间读取可能仍然可用。
- **E-S4（时间戳）。** $\theta_q$ 非减，并满足 A-TS。
- **E-L3（时间推进）。** 若 Espresso 在 $[\tau,\tau+T_E]$ 上排序可用，则到 $\tau+T_E$ 时，存在 $\theta\ge\tau-\Delta_{\rm ts}$ 的已最终化块。

E-S1 至 E-S4 在停机期间照样成立；停机只影响活性。
注意：一个在停机前提出的块，可能在停机结束后才被最终化，其时间戳仍是提出时的时间（D7 与此有关）。

### 2.5 数据可用性层（DA）

DA 只被 L2 层（§4）使用。先给出接口，再用假设陈述其性质。

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

**注。**
DA-S1 与 DA-S2 始终成立；DA 不可用只影响活性。
L1 合约不能直接调用 $\textsf{IsCert}$，DA 绑定要经由 $\mathsf C$ 的签名消息带到 L1 上。
超出保留窗口 $T_{\rm ret}$ 之后，数据能否取回由 $\Pi$ 自己的保管义务负责。

### 2.6 治理方

$\mathsf M$ 在协议中只有两个角色：

1. 在 L1 控制收件箱中发布 Force，强制进行区域轮换；
2. 发布 STOP 宣告 Espresso 停机，之后发布 RESUME 宣告恢复。

**A-GOV（只用于活性）。**

- **safety 不依赖 $\mathsf M$。** $\mathsf M$ 可以被腐化。被腐化的 $\mathsf M$ 可以在任意时刻签署任意控制记录；但只有通过 L1 合约检查的记录才会被记入控制收件箱，而这些检查保证控制不能破坏 safety（§5.3 的 D1、D2）。
- **需要治理行动的活性目标，假设 $\mathsf M$ 诚实。** 诚实的 $\mathsf M$ 按 $\mathcal Z$ 的指令及时签署并提交控制记录。治理决定的内容与时间由 $\mathcal Z$ 选择。
- **停机由治理负责宣告（D5）。** Espresso 停机时，依靠诚实的 $\mathsf M$ 发出 STOP。

### 2.7 可信 certifier

**A-CERT。**
$\mathsf C$ 是一个可信的诚实参与方：从不被腐化，按 $\Pi$ 规定的 certifier 规则运行。由 A-CRYPTO 与 A-PKI，任何带有 $\mathsf C$ 签名的消息，都是 $\mathsf C$ 按规则实际产生的。
本稿不讨论 trusted setup 与远程认证。

$\mathsf C$ 可能离线；离线只影响活性。在线期间，$\mathsf C$ 对请求在 $T_{\rm cert}$ 内作出响应。

**注。**

1. $\mathsf C$ 像其他诚实方一样读取 L1、Espresso 与 DA，因此它的日志视图可能落后。$\Pi$ 必须让 $\mathsf C$ 的结论显式绑定所用的来源切面，并且对视图前进单调：同一请求在更长的视图下不得给出相矛盾的结论。
2. 任何人都可以向 $\mathsf C$ 发请求，包括敌手。因此 safety 论证不能依赖“由谁来请求”。
3. “$\mathsf C$ 可信”只保证它按规则计算，不保证规则本身正确。
4. $\mathsf C$ 属于 L2 层。DRS 目前仍有两处可能需要它：STOP 的截断点（D7）与质押释放的关闭确认（D8）。

### 2.8 中继

**A-RELAY（仅用于活性）。**
对每项中继职责——向 Espresso 发布完整区域结果、转发用户交易、提交批次、向 L1 提交证据——在相关时间窗内至少有一个在线诚实方承担。session 的诚实成员可以承担这些职责。

### 2.9 参数表

| 参数 | 含义 | 计时起点 | 何时成立 | 是否用于 safety |
| --- | --- | --- | --- | --- |
| $\Delta_i$ | 区域内送达并处理 | 发送 | $s$ 的诚实成员之间，始终 | **是**（S4） |
| $\Delta_{\rm cross}$ | 跨区域送达并处理 | 发送 | 连通期间 | 否 |
| $\Delta_{\rm ts}$ | 时间戳与提出时刻的偏差 | — | 始终 | 否（采用 §5.3 的 D1 方式时） |
| $\delta_{\rm slot}$ | L1 出块间隔上界 | — | 始终 | 否 |
| $T_{\rm ready}$ | 准备精确 anchor 的可执行状态 | 输入可得 | 大小与负载在声明范围内 | 否 |
| $T_{\rm L1}$ | L1 最终化并可见 | 提交或提出 | 始终 | 否（采用 §5.3 的 D1 方式时） |
| $T_E$ | Espresso 最终化并可见 | 提交 | 可用期间 | 否 |
| $T_{\rm DA}$ | DA 认证 | 提交 | 可用期间 | L2 方案 A 中用于 L2-S1（D3） |
| $\Delta_{\rm DA}$ | DA 取回 | 请求 | 可用期间、保留窗口内 | L2 方案 A 中用于 L2-S1 |
| $T_{\rm ret}$ | DA 保留窗口 | 认证 | 始终 | L2 方案 A 中用于 L2-S1 |
| $T_{\rm cert}$ | $\mathsf C$ 的响应 | 请求 | $\mathsf C$ 在线期间 | L2 方案 A 中用于 L2-S1 |

协议常数：公告提前量 $M_a\ge T_{\rm L1}+2\Delta_{\rm ts}+\delta_{\rm slot}$（§5.3 的 D1）。
由协议推导的时限，在后续 Part 中给出：$T_{\rm live}$、$T_{\rm force}$、$T_{\rm stop}$、$T_{\rm dep}$、$T_{\rm resume}$、$T_{\rm batch}$、$T_{\rm slash}$、$T_{\rm release}$，以及证据窗口 $T_{\rm ev}$。
它们必须是上表参数的有限函数。

### 2.10 模型之外

以下情形不在本稿范围内，相关目标对此不作任何承诺：

- L1 的 safety 或 finality 失效（违反 X-S1 至 L1-S5）；
- Espresso 违反 E-S1 至 E-S4，例如出现冲突的 finalized 历史（模型内只有 Espresso 的活性失效）；
- $\mathsf C$ 被攻破；
- 保留窗口之外、且没有任何诚实方保存的数据丢失。

时间界（A-TS、$T_{\rm L1}$ 等）失效、A-BYZ 被违反、$\mathsf M$ 被腐化，都在模型之内：§3.3 与 §4.3 分别说明哪些目标在这些情况下仍然成立。

---

## 3. 去中心化区域轮换 sequencer（DRS）

### 3.1 定义

**定义 3.1（DRS）。** 去中心化区域轮换 sequencer 由以下五部分组成：

1. **质押层：** L1 上的合约。负责成员注册、冻结快照、委员会与 leader 的确定、证据处理（slash）与质押释放。
2. **L1 控制收件箱（force inbox）：** L1 上的合约。接收 $\mathsf M$ 的 Force、STOP、RESUME 记录，并强制检查控制纪律（§5.3）。
3. **共识层（CL）：** 每个 session 一个区域共识实例，以及管理这些实例生命周期与跨区域交接的逻辑。
4. **发布层：** 把完整的区域结果发布到 Espresso。
5. **全局最终化规则 $\mathsf{Replay}$：** 任何观察者都可以只读 L1 与 Espresso，独立计算全局链 $G$。

DRS 的外部依赖是 L1、Espresso、通信与时钟。
batcher、$\mathsf C$、DA、L1 派生与状态结算**不属于** DRS，它们属于 §4 的 L2 层。

**DRS 的接口。** 作为一个状态机复制（SMR）系统，DRS 的输入与输出如下：

| 方向 | 接口 | 发起方 → 接收方 |
| --- | --- | --- |
| 输入 | $\textsf{SubmitTx}(tx)$：提交用户交易 | 客户端 → DRS 的任一成员 |
| 输入 | 质押操作：加入、退出、提交证据等 | 任何人 → L1 质押层 |
| 输入 | 控制记录：Force、STOP、RESUME | $\mathsf M$ → L1 控制收件箱 |
| 输入 | L1 deposit 与系统输入 | L1 → DRS（由 $\mathsf{Replay}$ 与区域成员读取） |
| 输出 | 全局链 $G^u(\tau)$ | 任意观察者运行 $\mathsf{Replay}$ |
| 输出 | 快速确认 $\mathsf{Conf}_v(s,B)$ | session $s$ 的成员 $v$ |

**后续定义的规则。** 下面两条确定性规则在后续 Part 中给出。Part 0 只引用它们的名字与输入输出类型。

| 规则 | 输入 | 输出 |
| --- | --- | --- |
| $\mathsf{Exec}$（原生执行） | 前一状态、一个原生块的输入 | 下一状态，或“无效” |
| $\mathsf{Replay}$（全局最终化） | L1 日志、Espresso 日志 | 全局链 $G$ |

**模式。** DRS 有两种模式：

- **NORMAL：** $G$ 由各 session 生产的块延长；
- **DEPOSIT_ONLY：** Espresso 停机且 STOP 生效后，$G$ 由 $\mathsf{Replay}$ 依原生规则从 L1 输入派生的 deposit-only 块延长。这一步只读 L1，不需要 batcher、$\mathsf C$ 或 DA；STOP 截断点的来源见 D7。

**定义 3.2（全局块与出处）。**
全局链 $G$ 是 genesis 之后的块序列。每个块 $B$ 带有不可变的出处

$$
\sigma(B)\in\{\mathsf{ORD}(s),\ \mathsf{DEP}(p)\}.
$$

- $\mathsf{ORD}(s)$：由 session $s$ 生产的普通块；
- $\mathsf{DEP}(p)$：DEPOSIT_ONLY 模式下，由 L1 位置 $p$ 处的输入确定性派生出的 deposit-only 块。

$\mathrm{native}(B)$ 表示 $B$ 对应的原生 L2 块。$G$ 上的前缀关系同时比较块与出处。

**定义 3.3（观察者输出）。**
诚实观察者 $u$ 只读取 L1 与 Espresso，并运行 $\mathsf{Replay}$。它在 $\tau$ 时的输出为

$$
G^u(\tau)=\mathsf{Replay}\bigl(\mathrm{LOG}^{\rm L1}_u(\tau),\ \mathrm{LOG}^{E}_u(\tau)\bigr).
$$

这是一条结构要求（公开可重算）：$G$ 不依赖任何私有输入、消息到达顺序，或有自由裁量权的服务。

**定义 3.4（快速确认）。**
session $s$ 的诚实成员 $v$ 在 $\tau$ 时输出 $\mathsf{Conf}_v(s,B)$，表示 $v$ 在区域内提交（commit）了块 $B$。

**定义 3.5（治理关闭）。**
Force 或 STOP 生效时，会关闭一组 session。其生效位置由 $\mathsf{Replay}$ 确定，落在 $G$ 的两个相邻位置之间；所有诚实观察者对这一位置的判定一致。
Force 从 $\mathrm{last}(G)$ 开始新分支（D4）。

### 3.2 DRS 的性质

DRS 的核心性质是标准 SMR 的安全与活性。另外三条附加性质刻画快速确认与质押层。

**安全（S1、S2）：$G$ 中的任何日志都是安全的。**

- **S1（一致性）。** 对任意诚实观察者 $u,v$ 与任意时刻 $\tau,\tau'$：$G^u(\tau)\sim G^v(\tau')$。
- **S2（最终性）。** 若 $\tau\le\tau'$，则 $G^u(\tau)\preceq G^u(\tau')$。已输出的块及其出处永不改变。

**有效性（S3）。** 这一条对应 [CKPS01] 的 external validity。对任意诚实观察者 $u$ 与时刻 $\tau$：

- (a) **原生有效。** $\mathrm{native}(G^u(\tau))$ 是一条在 $\mathsf{Exec}$ 下从 genesis 状态出发的有效原生链，并满足原生规则中依赖 L1 的检查（L1 origin、时间戳、deposit 包含）。
- (b) **授权。** 对每个 $\sigma(B)=\mathsf{ORD}(s)$ 的块：$s$ 是 $\mathsf{Replay}$ 在该位置指定的负责 session，并且 $B$ 带有 $s$ 的区域最终证书（直接证书，或经同 session 后代覆盖的证书）。
- (c) **派生正确。** 对每个 $\sigma(B)=\mathsf{DEP}(p)$ 的块：$\mathrm{native}(B)$ 等于原生规则由截至 $p$ 的 L1 输入确定性派生出的块。

**活性（L）：标准 SMR 活性。** 称 DRS 在条件 $C$ 下满足 $T_{\rm live}$-活性，若 $C$ 在 $[\tau,\tau+T_{\rm live}]$ 上成立时：

- **L-tx（用户交易）。** 诚实客户端在 $\tau$ 时把交易 $tx$ 提交给 DRS 的任一在线诚实成员。若 $tx$ 从 $\tau$ 起持续有效，且负载不超过声明的容量，则对每个在该区间上活跃的诚实观察者 $u$，$tx$ 都出现在 $G^u(\tau+T_{\rm live})$ 中。
- **L-dep（L1 输入）。** 一笔 L1 deposit 在 $\tau$ 时 finalized，则它按原生规则规定的顺序，在 $\tau+T_{\rm live}$ 之前出现在 $G^u$ 中。

“持续有效”指 $tx$ 相对从提交到包含之间的每个状态都有效，例如不存在同 nonce 的冲突交易。

**附加性质。**

- **S4（快速确认一致）。** 设 $s$ 是良态 session。
  - (a) $s$ 的两个诚实成员不会在同一高度快速确认不同的块。
  - (b) 若 $s$ 的诚实成员输出 $\mathsf{Conf}_v(s,B)$，则对任意诚实观察者 $u$ 与时刻 $\tau$：只要 $G^u(\tau)$ 中有出处为 $\mathsf{ORD}(s)$、高度为 $h(B)$ 的块，那个块就是 $B$。

  按 D4，Force 从 $\mathrm{last}(G)$ 开始，所以尚未进入 $G$ 的快速确认可以被 Force 撤销；S4 不排除这一点。
- **S5（问责）。**
  - (a) **无错误惩罚。** 若某个 position 的密钥直到释放之前从未被腐化，它不会被 slash。
  - (b) **不重复处罚。** 同一次违规实例至多被处罚一次。
- **S6（质押安全）。** 若 position $o$ 在 $\tau$ 时被释放，则对每个满足 $o\in V_s$ 的 session $s$，都有

  $$
  \tau_s^{\rm close}+T_{\rm ev}\le\tau .
  $$

  也就是说，释放之后，$o$ 的签名不再能授权任何全局输出，并且针对 $o$ 的证据窗口已经过去。

### 3.3 DRS 的设计目标：不同假设下的安全与活性

设计目标写成“在假设集 $\mathcal H$ 下，DRS 满足性质 $P$”。下表最后一列列出**不需要**的假设，它们和所需假设同样重要。

| 目标 | 性质 | 所需假设 | 不需要 |
| --- | --- | --- | --- |
| DG1 | S1 一致性、S2 最终性 | A-CRYPTO, L1-S, E-S；以及模式边界规则的前缀稳定（协议义务，见 D7） | A-BYZ；$\mathsf M$ 诚实；网络同步；时间界（按 §5.3 的 D1 方式）；Espresso 可用 |
| DG2 | S3 有效性 | DG1 的假设 + A-BYZ | 成员在线；网络同步；$\mathsf M$ 诚实 |
| DG3 | S4 快速确认一致 | DG1 的假设 + $s$ 良态（含 $\Delta_i$） | $\mathsf M$ 诚实 |
| DG4 | S5 问责、S6 质押安全 | A-CRYPTO, A-PKI, L1-S，持久化存储；S6 另需关闭确认的来源（D8） | $\mathsf M$ 诚实；网络同步 |
| DG5 | NORMAL 活性（L-tx、L-dep） | 见下文 | $\mathsf M$ 诚实（只要它不发控制） |
| DG6 | 区域故障后恢复活性 | DG5 的条件（对新 session）+ $\mathsf M$ 诚实并发出 Force | — |
| DG7 | Espresso 停机期间的 L-dep | $\mathsf M$ 诚实并发出 STOP；L1-L；Espresso 读取可用；（若 D7 用 $\mathsf C$）$\mathsf C$ 在线 | Espresso 排序可用 |
| DG8 | Espresso 恢复后恢复活性 | $\mathsf M$ 诚实并发出 RESUME；DG5 的条件 | — |

记 L1-S 为 L1 的全部 safety 性质（X-S1 至 X-S3、L1-S4、L1-S5），E-S 为 E-S1 至 E-S4；L1-L、E-L 为相应的活性性质。

**DG5（NORMAL 活性）。** DRS 满足 $T_{\rm live}$-活性，条件 $C$ 为以下各项同时成立：

1. 没有控制记录生效，系统处于 NORMAL 模式；当前 policy 中至少有一个 ON 区域的权重非零；
2. Espresso 排序可用且读取可用（E-L）；L1-L；A-TS；
3. 区间内被选中的 session 都是良态的，且它们之间交接所需的跨区域连接是连通的；
4. 每个被选中的 session 中，诚实 leader 出现得足够频繁（具体条件见 D9）；
5. A-RELAY 成立。

DG2 的假设也须成立，因为活性要求被包含的交易有效。

**DG6（区域故障后恢复活性）。**
设 $\mathcal Z$ 在 $\tau_0$ 时指示诚实的 $\mathsf M$ 发出采用 policy $P$ 的 Force。若在 $[\tau_0,\tau_0+T_{\rm force}]$ 上，DG5 的条件对按 $P$ 选出的 session 成立（条件 1 中“无控制生效”除外），则：

- 到 $\tau_0+T_{\rm force}$ 时，Force 已经生效；
- 此后 DRS 恢复 $T_{\rm live}$-活性。

DRS 不自动检测区域故障；是否以及何时发出 Force 由治理决定。

**DG7（Espresso 停机期间的 deposit 活性）。**
设 $\mathcal Z$ 在 $\tau_0$ 时指示诚实的 $\mathsf M$ 发出 STOP。若 L1-L 成立、Espresso 读取可用（排序可以不可用），且（若 D7 采用 $\mathsf C$）$\mathsf C$ 在线，则：

- 到 $\tau_0+T_{\rm stop}$ 时，每个活跃诚实观察者的 $G$ 都已进入 DEPOSIT_ONLY 模式；
- 此后每笔在 $\tau\ge\tau_0+T_{\rm stop}$ 时 finalized 的 L1 deposit，在 $\tau+T_{\rm dep}$ 之前出现在 $G^u$ 中。

用户交易在此期间不被包含。

**DG8（恢复）。**
设 $\mathcal Z$ 在 $\tau_0$ 时指示诚实的 $\mathsf M$ 发出采用 policy $P$ 的 RESUME。若 DG5 的条件对按 $P$ 选出的 session 成立，则到 $\tau_0+T_{\rm resume}$ 时系统回到 NORMAL 模式，此后 DRS 恢复 $T_{\rm live}$-活性。

**从这张表可以读出四条结构性结论：**

1. **安全（DG1）的假设最少。** 它不依赖任何区域的诚实，不依赖 $\mathsf M$，不依赖网络同步与时间界，也不依赖 Espresso 可用。唯一的协议义务是模式边界规则的前缀稳定（D7）。
2. **有效性（DG2）只多依赖拜占庭界 A-BYZ，不依赖成员在线。** 诚实签名者只为有效的块签名；离线成员不签任何东西。
3. **快速确认（DG3）是唯一依赖区域同步 $\Delta_i$ 的 safety 目标。** 因此它只对良态 session 成立，并可被 Force 撤销。
4. **诚实的 $\mathsf M$ 只在故障恢复中需要（DG6–DG8）。** 平时的活性（DG5）只要求 $\mathsf M$ 不发控制。

---

## 4. 以 DRS 为子模块的 L2

### 4.1 组成与输出

L2（rollup）由 DRS 加上结算路径组成：

- **DRS**（§3），输出全局链 $G$；
- **batcher：** 把 $G$ 中的块编码成批次，经 DA 分发，并提交到 L1 batch inbox；
- **可信 certifier $\mathsf C$：** 核对批次与 $G$ 的对应关系及 DA 绑定，签名后由 inbox 接纳；
- **DA**（§2.5）；
- **L1 派生规则 $\mathsf{Derive}$：** 从 L1 日志与 DA 数据派生原生链；
- **状态结算规则：** 现有的 Kailua / fault-proof 路径，使用同一个 $\mathsf{Derive}$。

| 规则 | 输入 | 输出 |
| --- | --- | --- |
| $\mathsf{Derive}$（L1 派生） | L1 日志、已取回的 DA 数据 | 原生块序列 |

**定义 4.1（L1 派生链）。**
诚实派生节点 $u$ 在 $\tau$ 时的输出为

$$
D^u(\tau):=\mathsf{Derive}\bigl(\mathrm{LOG}^{\rm L1}_u(\tau),\ u\text{ 已取回的 DA 数据}\bigr).
$$

### 4.2 L2 的性质

- **L2-S1（L1 对应）。** 对任意诚实观察者 $u$、诚实派生节点 $v$ 与时刻 $\tau,\tau'$：

  $$
  \mathrm{native}(G^u(\tau))\ \sim\ D^v(\tau').
  $$

  这就是 $G$ 的“强最终”：一旦 $B\in G$，L1 不会在 $h(B)$ 处得出别的块。只要状态结算规则使用同一个 $\mathsf{Derive}$，结算的状态也与 $G$ 一致。
- **L2-S2（派生一致）。** 对任意诚实派生节点 $u,v$：$D^u(\tau)\sim D^v(\tau')$，且 $D^u$ 随时间只增长。
- **L2-L1（L1 确定的推进）。** 设 $B\in G^u(\tau_0)$。若在 $[\tau_0,\tau_0+T_{\rm batch}]$ 上 $\mathsf C$ 在线、DA 可用，且 A-RELAY 成立，则对每个在该区间上活跃的诚实派生节点 $v$，有 $\mathrm{native}(B)\in D^v(\tau_0+T_{\rm batch})$。

L2 继承 DRS 的全部性质。

### 4.3 L2 的设计目标

L2-S2 只依赖 L1-S、DA-S1 与 $\mathsf{Derive}$ 的确定性。L2-S1 的依赖取决于 D3 的选择：

|  | 方案 A：在时限内交付 | 方案 B：用规则保护 |
| --- | --- | --- |
| L2-S1 所需假设 | DG2 的假设、A-CERT、DA-S1、L1-S；**加上**时限：$T_{\rm DA}$、$T_{\rm cert}$、$T_{\rm L1}$、$T_{\rm ret}$、batcher 按时交付；管理员的 cursor override 不得跳过 $G$ 中的高度 | DG2 的假设、A-CERT、DA-S1、L1-S |
| DA 或 $\mathsf C$ 长时间不可用 | $G$ 中的块可能在 L1 上被 deposit-only 替换，L2-S1 失效 | L1 派生暂停，不替换 |
| sequencer 永久失效时，L1 deposit 的包含 | 由 sequencing window 自动完成，无需治理 | 需要诚实的 $\mathsf M$ 发出 STOP |
| 对现有规则的改动 | 小 | 修改 inbox、derivation 与 fault-proof 程序中的派生 |

L2-L1 在两种方案下的条件相同：$\mathsf C$ 在线、DA 可用、L1-L、A-RELAY。

### 4.4 三级确认

| 级别 | 含义 | 由谁给出 | 延迟量级 | 可被什么推翻 |
| --- | --- | --- | --- | --- |
| 快速确认 | $\mathsf{Conf}_v(s,B)$ | $s$ 的诚实成员 | 若干 $\Delta_i$ | Force 或 STOP 在 $B$ 进入 $G$ 之前生效；或 $s$ 不良态 |
| 全局最终 | $B\in G^u(\tau)$ | 任意诚实观察者 | 发布 + Espresso | 不可推翻 |
| L1 确定 | $\mathrm{native}(B)\in D^u(\tau)$ | 任意诚实派生节点 | 批次 + DA + L1 | 不可推翻 |

---

## 5. 故障场景、非目标与设计决定

### 5.1 各故障场景下的含义

| 场景 | DRS 安全 | DRS 活性 | L2 |
| --- | --- | --- | --- |
| 某 session 有少数拜占庭成员（良态） | 全部 | 继续 | 正常 |
| 整区离线或区域网络中断（A-BYZ 成立，不良态） | S1–S3, S5, S6；该 session 的 S4 不保证 | 需要 Force（DG6） | 正常 |
| 跨区域网络分区 | 全部 | 恢复连通后继续交接 | 正常 |
| Espresso 停机 | 全部 | STOP 后只有 deposit 活性（DG7），RESUME 后恢复（DG8） | 方案 A：停机前的块须在窗口内交付；方案 B：等待交付 |
| DA 不可用 | 全部 | 不受影响 | 方案 A：L2-S1 可能失效；方案 B：派生暂停 |
| $\mathsf C$ 离线 | 全部 | 若 D7 采用 $\mathsf C$，STOP 受阻 | 派生暂停 |
| $\mathsf M$ 被腐化 | 全部 | 可被反复关闭或停止；失去故障恢复 | 方案 B：不受影响；方案 A：取决于 override 是否受限 |
| $\mathsf M$ 沉默 | 全部 | 失去故障恢复（DG6–DG8）；质押释放可能停滞 | 方案 B 下，sequencer 失效时 deposit 无法经 L1 自动包含 |
| 时间界失效（$T_{\rm L1}$、A-TS） | 全部（采用 §5.3 的 D1 方式时） | $G$ 暂停，直到 L1 日志追上 | — |
| 违反 A-BYZ | S1、S2 仍成立；S3、S4 不保证 | — | L2-S1 不保证 |
| L1 或 Espresso 的 safety 失效、$\mathsf C$ 被攻破 | 不作承诺（模型之外） | — | — |

### 5.2 非目标

- 超出 SMR 活性的抗审查，例如对拜占庭 leader 的即时包含保证或时间公平。
- 区域之间、leader 之间的公平，以及排序公平或 MEV 相关性质。
- 自动检测 Espresso 停机或区域故障。这些由治理判断。
- Accountable safety，即安全被破坏时必能指认一定权重的违规者。在同步、$F_s<W_s/2$ 的区域协议中，两张证书不必相交。
- 超出保留窗口的数据丢失之后的无损恢复。
- 消息机密性与交易隐私。
- 密码学意义上的可组合性（见 §0.1）。

### 5.3 已定的设计决定

**D1｜保留快速的全局最终化（已定）。**
$G$ 不等待 L1 finality；Force 依靠公告提前量，在 Espresso 时间阈值 $\theta_a$ 处生效。
**落实方式（请确认）：**

- **合约检查。** L1 控制收件箱只接纳满足 $\theta_a\ge\mathit{ts}(b)+M_a$ 的 Force，其中 $b$ 是纳入该记录的 L1 块，$M_a\ge T_{\rm L1}+2\Delta_{\rm ts}+\delta_{\rm slot}$ 是 genesis 固定的常数。
- **观察者等待规则。** $\mathsf{Replay}$ 处理 $J_q$ 之前，要求自己的 L1 日志中已有时间戳 $\ge\theta_q-M_a$ 的块。

效果如下：

- **安全不依赖时间界。** 若某个 Force 在 $J_q$ 或更早生效，则 $\theta_a\le\theta_q$，故 $\mathit{ts}(b)\le\theta_q-M_a$。由于 L1 时间戳严格递增、日志是前缀，$b$ 必然已在观察者的日志中。这只用到 L1-S，不用 A-TS 或 $T_{\rm L1}$。
- **正常情况下延迟不变。** 由 A-TS 与 L1-L3，活跃观察者在 Espresso 块到达时，其 L1 日志已经覆盖到 $\theta_q-M_a$，无需等待。
- **时间界失效只导致暂停。** 若 L1 finality 变慢，观察者等待，$G$ 暂停，但不分叉。

$\mathsf M$ 的提前量要求与旧方案相同（约一个 $T_{\rm L1}$），但现在由合约强制。

**D2｜$\mathsf M$ 只影响活性（已定）。** 推论如下：

1. 所有控制纪律由 L1 合约或 $\mathsf{Replay}$ 强制，不依赖 $\mathsf M$ 诚实。包括：公告提前量（D1）；policy 格式（每个区域恰有一个 ON/OFF 标记，并在顺序中恰出现一次）；序号递增。
2. 多个控制在同一个 Espresso 块生效时，$\mathsf{Replay}$ 按序号确定性地依次处理，不再假设“每块至多一个”。
3. STOP 的截断点由规则确定，不由 $\mathsf M$ 指定（D7）。
4. 在 Espresso 正常时，恶意的 STOP 只会让系统进入 DEPOSIT_ONLY；在 Espresso 停机时，恶意的 RESUME 只会让系统无法出块。两者都只损失活性。
5. 质押释放的安全（S6）不依赖 $\mathsf M$。
6. L2 层若保留管理员的 cursor override，它也不能跳过 $G$ 中的高度（与 D3 相关）。

**D4｜Force 从 $\mathrm{last}(G)$ 开始（已定）。**
尚未进入 $G$ 的快速确认可以被 Force 撤销。S4 只保证：$G$ 中出处为 $s$、同一高度的块就是被快速确认的块。

**D5｜依赖负责任的 $\mathsf M$ 宣告停机（已定）。**
Espresso 停机时，由诚实的 $\mathsf M$ 发出 STOP。若 $\mathsf M$ 沉默，受影响 session 的 $\tau^{\rm close}$ 无界，相应质押的释放停滞；这只是活性损失，与 D2 一致。

**D6｜Espresso 的可用性建模（已定）。** 保留“排序可用”与“读取可用”两种状态。

### 5.4 待讨论

**D3｜受保护前缀：方案 A 还是方案 B。**
背景：现有 mega-espresso 保证的是“二选一”——每个高度要么按选中记录的内容结算，要么被 deposit-only 块替换。替换有三个来源：

1. sequencing window 过期时仍没有被接纳的批次；
2. 依赖 L1 的检查（origin、时间戳、sequencer drift）失败，或执行被拒（Δ5）；
3. 管理员的 emergency cursor override 跳过高度。

L2-S1 要求这三种替换都不能落到 $G$ 中的块上。第 2 种由区域验证排除（DG2），两个方案都需要；区别在第 1、3 种：

- **方案 A** 保持派生规则不变，用时限保证每个块在剩余窗口内交付，并约束 override。
- **方案 B** 修改派生规则：NORMAL 模式下批次未到就等待、不回退；deposit-only 只在 L1 上提交了 STOP 边界、并且受保护前缀全部交付之后才开始；override 不得跳过受保护前缀。

对比见 §4.3。

**D7｜STOP 的截断点从哪里来。**
Force 在 Espresso 时间阈值处生效，只需要 Espresso 继续出块。STOP 恰恰在 Espresso 停机时使用，不能等 Espresso 出块。
麻烦在于：停机前提出的块可能在恢复后才最终化，且时间戳早于阈值。因此单凭 L1 与 Espresso 的公开数据，观察者在停机期间无法就 $G$ 的截断点达成一致。
旧稿用 $\mathsf C$ 的 MODE 证书在 L1 上固定截断点。在 DRS 不含 $\mathsf C$ 的前提下，有两个选择：

- (i) 让截断证书成为 DRS 的一部分：DG1 将依赖 A-CERT，DG7 依赖 $\mathsf C$ 在线；
- (ii) DRS 的 STOP 只在 Espresso 恢复后按阈值生效，停机期间的 deposit-only 推进完全交给 L2 层。但这样，L2 层停机期间使用的截断点，与 DRS 之后确定的截断点可能不一致，需要额外的规则把两者对齐。

**D8｜质押释放怎样得知关闭。**
S6 要求释放晚于 $\tau_s^{\rm close}+T_{\rm ev}$，而 $\tau_s^{\rm close}$ 往往由 Espresso 数据决定，L1 合约读不到。旧稿用 $\mathsf C$ 的 CLOSURE 证书告知 L1。
若 DRS 不含 $\mathsf C$，要么让质押层把 $\mathsf C$ 当作外部证明来源（DG4 依赖 A-CERT），要么找一种只用 L1 数据的保守判定。

**D9｜活性中的 leader 条件。**
同步 BFT 的活性需要诚实 leader 主持完整的 term。leader 按权重随机选出，因此“诚实 leader 出现得足够频繁”有两种处理方式：作为假设写入 DG5；或者在 A-BYZ 下给出概率性的推导。

---

## 参考文献

- [GKL15] J. Garay, A. Kiayias, N. Leonardos. *The Bitcoin Backbone Protocol: Analysis and Applications.* EUROCRYPT 2015.
- [PS17] R. Pass, E. Shi. *The Sleepy Model of Consensus.* ASIACRYPT 2017.
- [AMNRY20] I. Abraham, D. Malkhi, K. Nayak, L. Ren, M. Yin. *Sync HotStuff: Simple and Practical Synchronous State Machine Replication.* IEEE S&P 2020.
- [CKPS01] C. Cachin, K. Kursawe, F. Petzold, V. Shoup. *Secure and Efficient Asynchronous Broadcast Protocols.* CRYPTO 2001（external validity）。
