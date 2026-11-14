# MegaETH Sequencer Rotation 技术报告 · Part 0

## 模型、假设与设计目标（中文工作稿 v0.2）

日期：2026-10-25。
状态：迭代草稿。本稿写三件事：(1) 系统模型；(2) 假设，以及外部组件的接口；(3) 设计目标。
模块架构与各模块的 functionality 在这三部分定稿后再写。
上一版保留在 [v0.1](part0_model_goals_zh.md)。

**相对 v0.1 的变化**

1. 不再采用 GUC 框架，也不写功能框和敌手接口。L1 与 Espresso 改为账本假设：一致性、最终性、有界活性。协议日后写入这两本账本的数据，直接继承这些性质。
2. §1.2 增加时间戳假设 A-TS：两条链的时间戳与真实时间的偏差都不超过 $\Delta_{\rm ts}$，因而两条链之间的时间戳偏差不超过 $2\Delta_{\rm ts}$。它取代 v0.1 中的 $\varepsilon_{\rm L1}$ 与 $\varepsilon_E$。
3. 职责窗口改用更直白的定义，并给出正常交接、Force、STOP 三个例子（§1.3）。
4. “正确”统一改为“诚实”。对一个 session 而言，“诚实成员”包含在线与同步的要求（§1.4）。
5. DA 改为“接口 + 假设”的写法；可信 certifier 改为一个可信的诚实参与方 $\mathsf C$。
6. 删去原生执行与状态结算的抽象。它们以及全局重算、L1 派生，日后都写成确定性**规则**。
7. 假设改用助记名（A-L1、A-ESP 等），依赖表随之更新。

---

## 0. 写法与约定

### 0.1 形式化方式

本稿采用共识文献中常见的 property-based 写法 [GKL15, PS17]。

1. **执行。**
   一次执行由环境 $\mathcal Z$、敌手 $\mathcal A$ 与协议 $\Pi$ 组成，记作 $\mathrm{EXEC}_{\Pi,\mathcal A,\mathcal Z}(\kappa)$。$\mathcal Z$ 提供全部外部输入；$\mathcal A$ 控制被腐化的参与方，并在假设允许的范围内决定消息延迟。
2. **外部系统写成假设。**
   L1、Espresso、DA 不写成理想功能。对每个外部系统，我们只给出必要的使用接口，以及协议所依赖的性质。
3. **内部计算写成规则。**
   原生执行、全局重算、L1 派生、状态结算等，日后写成所有人可以独立计算的确定性规则。它们既不是参与方，也不是功能。
4. **设计目标是执行上的性质。**
   称性质 $P$ 成立，若对所有满足 §2 假设的 PPT $(\mathcal A,\mathcal Z)$，有 $\Pr[\mathrm{EXEC}\text{ 违反 }P]\le\mathrm{negl}(\kappa)$。
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
| $\mathsf M$；$\mathsf C$ | 治理方（multisig）；可信 certifier。两者都是诚实参与方。 |
| $\mathrm{LOG}^X_P(\tau)$ | 诚实参与方 $P$ 在 $\tau$ 时看到的账本 $X$ 的已最终化日志（§2.4）。 |
| $X\preceq Y$ | 序列 $X$ 是 $Y$ 的前缀。 |
| $X\sim Y$ | $X\preceq Y$ 或 $Y\preceq X$（可比较）。 |
| $G^u(\tau)$ | 诚实观察者 $u$ 在 $\tau$ 时输出的全局链（§3.1）。 |
| $D^u(\tau)$ | 诚实派生节点 $u$ 在 $\tau$ 时由 L1 与 DA 派生出的原生链（§3.1）。 |

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
- **治理方 $\mathsf M$。** multisig 委员会，作为**一个**逻辑参与方建模（A-GOV）。
- **可信 certifier $\mathsf C$。** 原先由 TEE 承担的 certifier，作为**一个**可信的诚实参与方建模（A-CERT）。
- **角色。** 用户、publisher、batcher、观察者、证据提交者都是角色而非身份。任何参与方，包括被腐化的参与方，都可以承担这些角色；一个运营者可以兼任多个角色。
- **L1 合约不是参与方。** Authority、Inbox 等是部署在 L1 上的确定性程序，属于 $\Pi$ 的一部分。

环境 $\mathcal Z$ 提供全部外部输入：用户交易、质押请求，以及交给 $\mathsf M$ 的**治理指令**（A-GOV）。
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
因此，比较 L1 时间戳与 Espresso 时间戳是有意义的：误差在 $2\Delta_{\rm ts}$ 以内。

“提出”与“最终化”是两件事。块被提出之后，要经过一段时间才 finalized 并对所有诚实方可见；这段延迟由账本的活性时限 $T_{\rm L1}$、$T_E$ 约束（§2.4）。
对 Ethereum 而言，提出到 finalized 的时间可达十余分钟，所以 $\mathit{ts}(b)$ 接近提出时刻，而不接近 finalized 时刻。

### 1.3 session 与职责窗口

为了陈述假设与目标，这里只引入最少的 session 概念。session 怎样选出、怎样交接，属于后续模块。

**定义 1.1（session）。**
$\Pi$ 把普通出块划分为 session。每个 session $s$ 确定：

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

  精确判定由日后的全局重算规则给出。

职责窗口在本稿中有三个用途：

- **拜占庭计数（A-BYZ）。** 只计算在 $\tau_s^{\rm close}$ 之前被腐化的成员。窗口结束后才泄露的密钥，已经不能再改变全局链。
- **诚实的含义（§1.4）。** $s$ 的诚实成员须在整个窗口内在线并保持同步。
- **质押释放（S7）。** 只有在 $\tau_s^{\rm close}$ 再加上证据窗口之后，$s$ 的成员才能释放质押。

**例子。** 设区域为 A、B、C，每个 session 的 DATA 配额为 10，当前分支版本为 7。

| 场景 | 发生了什么 | 窗口的开始与结束 |
| --- | --- | --- |
| 开始（正常） | $s_1=(7,20)$ 在区域 A，从 anchor 100（前驱的末块）出发，生产 101–110。$s_2=(7,21)$ 在区域 B，是 $s_1$ 的常规后继。 | $s_2$ 的委员会由 $s_1$ 的 anchor 100 与来源区域 A 决定。所以块 100 一得到区域 FINAL，任何人都能算出 $V_{s_2}$：$\tau^{\rm open}_{s_2}$ 就是这一时刻，比 $s_2$ 开始出块早了大约一个 session 的时长。同理，$\tau^{\rm open}_{s_1}$ 是 $s_1$ 的前驱的 anchor 被确定的时刻。 |
| 正常结束 | 块 110 在 Espresso 块 $J_q$ 的重算中进入全局链。 | $\tau^{\rm close}_{s_1}=\tau_E(q)$，即 $J_q$ 最终化的时刻。此后即使 $s_1$ 成员的密钥泄露，也不能再改变全局链。 |
| Force | 全局链停在 108 时，Force 在 Espresso 块 $J_{q'}$ 生效，关闭分支 7。 | 分支 7 上所有 session（$s_1$、$s_2$，以及更远的推测性后继）的窗口都在 $\tau_E(q')$ 结束。$s_1$ 的 109、110 即使已有区域 FINAL，也不再进入全局链。新分支 8 的第一个 session 从 108 出发，其 $\tau^{\rm open}=\tau_E(q')$。 |
| STOP | $s_2$ 出块期间 Espresso 停机，全局链停在 115。 | Espresso 不再产生新块，无法由它确定 $s_2$ 的结束。STOP 在 L1 上的生效记录 finalized 的时刻，就是 $\tau^{\rm close}_{s_2}$。若一直没有 STOP，则 $\tau^{\rm close}_{s_2}=\infty$（见 D5）。 |

### 1.4 腐化、诚实与离线

**腐化模型。**
敌手可以**自适应地**在任意时刻腐化任意参与方，立即取得其全部状态，包括私钥与历史状态（不假设安全擦除），此后完全控制它。腐化不可撤销。
$\mathsf M$ 与 $\mathsf C$ 不可被腐化（A-GOV、A-CERT）。

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
快速确认与 NORMAL 活性只对良态 session 声称（§3）。

### 1.5 通信

参与方之间使用认证点对点信道：敌手不能伪造或篡改诚实方的消息，但可以重排、重复消息，并在下述时限内决定延迟。信道不提供机密性。
消息大小及其随附的验证依赖不超过声明的上界 $m_{\max}$；$\Delta$ 包含接收方对这些依赖的验证处理。

- **区域内同步 $\Delta_i$。**
  对 $s$ 的任意两个诚实成员，于 $\tau$ 发出的消息在 $\tau+\Delta_{i_s}$ 之前送达并处理完毕。
  这一时限既用于区域活性，**也是快速确认安全（S5）的前提**，因为同步 BFT 的安全性依赖它 [AMNRY20]。
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
成员公钥经 L1 上的 Authority 注册，并附 proof of possession；诚实方诚实地生成密钥。
$\mathsf M$ 与 $\mathsf C$ 的公钥在 genesis 固定，所有参与方与 L1 合约都知道。

### 2.3 区域拜占庭界

**A-BYZ。** 对 $\Pi$ 产生的每个 session $s$：

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

- **L1-S4（合约忠实执行）。** 任意高度 $\ell$ 的合约状态、按顺序排列的完整事件列表，以及每笔调用的结果（包括失败），都由已最终化日志唯一确定。合约调用按日志顺序原子执行；已部署的合约代码在模型中固定不变。
- **L1-S5（时间戳）。** 时间戳严格递增，并满足 A-TS。

$T_{\rm L1}$ 覆盖从提交到 finalized、再到对活跃诚实方可见的全过程。

**A-ESP（Espresso）。**
Espresso 满足 X-S1 至 X-S3（记为 E-S1 至 E-S3）。其活性性质只在可用期间成立，参数为 $T_X=T_E$。此外：

- **Namespace。** publication 带 namespace 标签。读取时，每块按顺序给出本链 namespace 的完整内容 $M_q$，可以为空。Espresso 只负责排序，不验证本协议的任何内容；本链 namespace 中的数据可以由任何人提交，包括敌手。
- **两种可用性。**
  - **排序可用：** 新 publication 能被收录并最终化。X-L1 只在排序可用期间成立。
  - **读取可用：** 已最终化的块能被读取。X-L2 只在读取可用期间成立。

  **停机**定义为排序不可用。停机期间读取可能仍然可用：STOP 路径需要读旧块，但不需要新块。
- **E-S4（时间戳）。** $\theta_q$ 非减，并满足 A-TS。
- **E-L3（时间推进）。** 若 Espresso 在 $[\tau,\tau+T_E]$ 上排序可用，则到 $\tau+T_E$ 时，存在 $\theta\ge\tau-\Delta_{\rm ts}$ 的已最终化块。

E-S1 至 E-S4 在停机期间照样成立；停机只影响活性。

### 2.5 数据可用性层（DA）

先给出接口，再用假设陈述其性质。

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

**A-GOV。**
$\mathsf M$ 从不被腐化。它只针对 $\mathcal Z$ 发来的治理指令签署控制记录，并遵守 $\Pi$ 为 $\mathsf M$ 规定的控制纪律，例如公告提前量；具体规则在 Authority Part 中给出。

**治理决定的内容与时间由 $\mathcal Z$ 选择。** 因此：

- safety 目标必须对所有治理行为成立；
- 活性目标只在明确写出的治理条件下声称。

$\mathsf M$ 也可以离线或沉默；这只影响依赖治理行动的活性目标。

### 2.7 可信 certifier

**A-CERT。**
$\mathsf C$ 是一个可信的诚实参与方：从不被腐化，按 $\Pi$ 规定的 certifier 规则运行。由 A-CRYPTO 与 A-PKI，任何带有 $\mathsf C$ 签名的消息，都是 $\mathsf C$ 按规则实际产生的。
本稿不讨论 trusted setup 与远程认证。

$\mathsf C$ 可能离线；离线只影响活性。在线期间，$\mathsf C$ 对请求在 $T_{\rm cert}$ 内作出响应。

**注。**

1. $\mathsf C$ 像其他诚实方一样读取 L1、Espresso 与 DA，因此它的日志视图可能落后。$\Pi$ 必须让 $\mathsf C$ 的结论显式绑定所用的来源切面，并且对视图前进单调：同一请求在更长的视图下不得给出相矛盾的结论。
2. 任何人都可以向 $\mathsf C$ 发请求，包括敌手。因此 safety 论证不能依赖“由谁来请求”。
3. “$\mathsf C$ 可信”只保证它按规则计算，不保证规则本身正确。例如，共同模式边界的正确性仍然是 $\Pi$ 的证明义务。

### 2.8 中继

**A-RELAY（仅用于活性）。**
对每项中继职责——向 Espresso 发布完整区域结果、提交批次、向 L1 提交证据——在相关时间窗内至少有一个在线诚实方承担。session 的诚实成员可以承担这些职责。

### 2.9 参数表

| 参数 | 含义 | 计时起点 | 何时成立 | 是否用于 safety |
| --- | --- | --- | --- | --- |
| $\Delta_i$ | 区域内送达并处理 | 发送 | $s$ 的诚实成员之间，始终 | **是**（S5） |
| $\Delta_{\rm cross}$ | 跨区域送达并处理 | 发送 | 连通期间 | 否 |
| $\Delta_{\rm ts}$ | 时间戳与提出时刻的偏差 | — | 始终 | 现方案经控制纪律间接使用（D1） |
| $T_{\rm ready}$ | 准备精确 anchor 的可执行状态 | 输入可得 | 大小与负载在声明范围内 | 否 |
| $T_{\rm L1}$ | L1 最终化并可见 | 提交 | 始终 | 现方案经控制纪律间接使用（D1） |
| $T_E$ | Espresso 最终化并可见 | 提交 | 可用期间 | 否 |
| $T_{\rm DA}$ | DA 认证 | 提交 | 可用期间 | 方案 A 中用于 S4（D3） |
| $\Delta_{\rm DA}$ | DA 取回 | 请求 | 可用期间、保留窗口内 | 方案 A 中用于 S4 |
| $T_{\rm ret}$ | DA 保留窗口 | 认证 | 始终 | 方案 A 中用于 S4 |
| $T_{\rm cert}$ | $\mathsf C$ 的响应 | 请求 | $\mathsf C$ 在线期间 | 方案 A 中用于 S4 |

另有一组由协议推导的时限，在后续 Part 中给出：$T_{\rm grow}$、$T_{\rm incl}$、$T_{\rm force}$、$T_{\rm stop}$、$T_{\rm dep}$、$T_{\rm resume}$、$T_{\rm batch}$、$T_{\rm slash}$、$T_{\rm release}$，以及证据窗口 $T_{\rm ev}$。
它们必须是上表参数的有限函数。

### 2.10 模型之外

以下情形不在本稿范围内，相关目标对此不作任何承诺：

- L1 违反 A-L1，包括 safety 或 finality 失效，以及 $T_{\rm L1}$ 不成立；
- Espresso 违反 E-S1 至 E-S4，例如出现冲突的 finalized 历史（模型内只有 Espresso 的活性失效）；
- 时间戳违反 A-TS；
- $\mathsf C$ 或 $\mathsf M$ 被攻破；
- 某个 session 违反 A-BYZ；
- 保留窗口之外、且没有任何诚实方保存的数据丢失。

---

## 3. 设计目标

### 3.1 后续规则、系统输出与三级确认

**后续定义的规则。** 下面三条确定性规则在后续 Part 中给出。Part 0 只引用它们的名字与输入输出类型。

| 规则 | 输入 | 输出 |
| --- | --- | --- |
| $\mathsf{Exec}$（原生执行） | 前一状态、一个原生块的输入 | 下一状态，或“无效” |
| $\mathsf{Replay}$（全局重算） | L1 日志、Espresso 日志、已取回的 DA 数据 | 全局链 $G$ |
| $\mathsf{Derive}$（L1 派生） | L1 日志、已取回的 DA 数据 | 原生块序列 |

状态根在 L1 上的结算，日后也写成使用同一 $\mathsf{Derive}$ 的规则。

**定义 3.1（全局块与出处）。**
全局链 $G$ 是 genesis 之后的块序列。每个块 $B$ 带有不可变的出处

$$
\sigma(B)\in\{\mathsf{ORD}(s),\ \mathsf{DEP}(p)\}.
$$

- $\mathsf{ORD}(s)$：由 session $s$ 生产的普通块；
- $\mathsf{DEP}(p)$：Espresso 停机后，由 L1 位置 $p$ 处的输入确定性派生出的 deposit-only 块。

$\mathrm{native}(B)$ 表示 $B$ 对应的原生 L2 块。$G$ 上的前缀关系同时比较块与出处。

**定义 3.2（观察者输出）。**
诚实观察者 $u$ 只读取 L1、Espresso 与 DA，并运行 $\mathsf{Replay}$。它在 $\tau$ 时的输出记为 $G^u(\tau)$。

**定义 3.3（快速确认）。**
session $s$ 的诚实成员 $v$ 在 $\tau$ 时输出 $\mathsf{Conf}_v(s,B)$，表示 $v$ 在区域内提交（commit）了块 $B$。

**定义 3.4（L1 派生链）。**
诚实派生节点 $u$ 在 $\tau$ 时的输出为

$$
D^u(\tau):=\mathsf{Derive}\bigl(\mathrm{LOG}^{\rm L1}_u(\tau),\ u\text{ 已取回的 DA 数据}\bigr).
$$

**定义 3.5（治理关闭）。**
治理控制（Force 或 STOP）生效时，会关闭一组 session。其生效位置由 $\mathsf{Replay}$ 确定，落在 $G$ 的两个相邻位置之间；所有诚实观察者对这一位置的判定一致。

对用户而言，系统提供三级确认：

| 级别 | 含义 | 由谁给出 | 延迟量级 | 可被什么推翻 |
| --- | --- | --- | --- | --- |
| 快速确认 | $\mathsf{Conf}_v(s,B)$ | $s$ 的诚实成员 | 若干 $\Delta_i$ | 治理关闭在 $B$ 进入 $G$ 之前生效；或 $s$ 不良态 |
| 全局最终 | $B\in G^u(\tau)$ | 任意诚实观察者 | 发布 + Espresso | 不可推翻 |
| L1 确定 | $\mathrm{native}(B)\in D^u(\tau)$ | 任意诚实派生节点 | 批次 + DA + L1 | 不可推翻 |

### 3.2 安全目标

除非另行注明，以下目标在 §2 的全部假设下**无条件**成立：对任意治理行为，以及 Espresso、DA、$\mathsf C$ 的任意可用性模式，都成立。

**S0（公开可重算）。**
对每个诚实观察者 $u$ 与每个时刻 $\tau$，

$$
G^u(\tau)=\mathsf{Replay}\bigl(\mathrm{LOG}^{\rm L1}_u(\tau),\ \mathrm{LOG}^{E}_u(\tau),\ u\text{ 已取回的 DA 数据}\bigr).
$$

换言之，$G$ 不依赖任何私有输入、消息到达顺序，或有自由裁量权的服务。

**S1（全局一致）。**
对任意诚实观察者 $u,v$ 与任意时刻 $\tau,\tau'$：$G^u(\tau)\sim G^v(\tau')$。

**S2（持久）。**
若 $\tau\le\tau'$，则 $G^u(\tau)\preceq G^u(\tau')$。特别地，已进入 $G$ 的块及其出处永不改变。

**S3（有效）。** 这一条对应 [CKPS01] 的 external validity。对任意诚实观察者 $u$、时刻 $\tau$：

- (a) **原生有效。** $\mathrm{native}(G^u(\tau))$ 是一条在 $\mathsf{Exec}$ 下从 genesis 状态出发的有效原生链。
- (b) **授权。** 对每个 $\sigma(B)=\mathsf{ORD}(s)$ 的块：$s$ 是 $\mathsf{Replay}$ 在该位置指定的负责 session，并且 $B$ 带有 $s$ 的区域最终证书（直接证书，或经同 session 后代覆盖的证书）。
- (c) **派生正确。** 对每个 $\sigma(B)=\mathsf{DEP}(p)$ 的块：$\mathrm{native}(B)$ 等于原生规则由截至 $p$ 的 L1 输入确定性派生出的块。

**S4（L1 对应）。**
对任意诚实观察者 $u$、诚实派生节点 $v$ 与时刻 $\tau,\tau'$：

$$
\mathrm{native}(G^u(\tau))\ \sim\ D^v(\tau').
$$

只要日后的状态根结算规则使用同一个 $\mathsf{Derive}$，S4 就保证结算的状态与 $G$ 一致。
这条性质的强度取决于 D3 的选择：在方案 A 下它依赖时间界，在方案 B 下不依赖。

**S5（快速确认的兑现）。** 设 $s$ 是良态 session。

- (a) $s$ 的两个诚实成员不会在同一高度快速确认不同的块。
- (b) 若 $s$ 的诚实成员输出 $\mathsf{Conf}_v(s,B)$，则对任意诚实观察者 $u$ 与时刻 $\tau$：只要 $G^u(\tau)$ 中有出处为 $\mathsf{ORD}(s)$、高度为 $h(B)$ 的块，那个块就是 $B$。

**S6（问责）。**

- (a) **无错误惩罚。** 若某个 position 的密钥直到释放之前从未被腐化，它不会被 slash。
- (b) **不重复处罚。** 同一次违规实例至多被处罚一次。

**S7（质押安全）。**
若 position $o$ 在 $\tau$ 时被释放，则对每个满足 $o\in V_s$ 的 session $s$，都有

$$
\tau_s^{\rm close}+T_{\rm ev}\le\tau .
$$

也就是说，释放之后，$o$ 的签名不再能授权任何全局输出，并且针对 $o$ 的证据窗口已经过去。

**注（S1/S2 的结构来源）。**
把三份输入（L1 日志、Espresso 日志、DA 数据）按分量取前缀序 $\sqsubseteq$。若 $\mathsf{Replay}$ 单调，即 $V\sqsubseteq V'\Rightarrow\mathsf{Replay}(V)\preceq\mathsf{Replay}(V')$，那么 S1 与 S2 直接由 X-S1、X-S2 与 DA-S1 推出：任意两个诚实视图都 $\sqsubseteq$ 它们按分量取的较长者，从而两份输出都是同一序列的前缀。
困难在于跨两本账本的单调性。观察者在处理 $J_q$ 时，可能尚未看到会影响 $J_q$ 的 L1 控制。
现方案用控制公告提前量 $\theta_a>\tau_{\rm submit}+T_{\rm L1}+\Delta_{\rm ts}$ 保证：凡是能在诚实方处同时出现的视图组合，都已包含相关控制。因此 $\mathsf{Replay}$ 只在**可达视图**上单调，S1 也就间接依赖 $T_{\rm L1}$ 与 $\Delta_{\rm ts}$（见 D1）。
STOP/RESUME 的共同边界规则要满足的，也正是这条单调性。

### 3.3 活性目标

统一形式为：“若条件 $C$ 在区间 $I=[\tau_0,\tau_0+T]$ 上成立，则在 $\tau_0+T$ 时有结论。”
其中 $T$ 是由协议推导的时限，后续 Part 会把它写成 §2.9 参数的有限函数。
下文的“观察者”均指在 $I$ 上活跃的诚实观察者。

**L1（NORMAL 增长）。** 若在 $I=[\tau_0,\tau_0+T_{\rm grow}]$ 上同时满足：

1. 没有治理控制生效，且系统处于 NORMAL 模式；
2. Espresso 排序可用且读取可用；
3. $\mathsf{Replay}$ 能选出 session，即当前 policy 中至少有一个 ON 区域的权重非零；
4. 在 $I$ 内被选中的 session 都是良态的，并且它们交接所需的跨区域连接是连通的；
5. A-RELAY 成立，

则每个观察者都有 $|G^u(\tau_0+T_{\rm grow})|>|G^u(\tau_0)|$。

**L2（快速确认的收录）。**
设 $s$ 的诚实成员在 $\tau_0$ 时输出 $\mathsf{Conf}_v(s,B)$，且 $s$ 良态。若在 $I=[\tau_0,\tau_0+T_{\rm incl}]$ 上：

- 没有覆盖 $s$ 的关闭在 $B$ 之前的位置生效；
- Espresso 排序可用且读取可用；
- A-RELAY 成立，

则 $B\in G^u(\tau_0+T_{\rm incl})$。

**L3（治理替换故障区域）。**
设 $\mathcal Z$ 在 $\tau_0$ 时指示 $\mathsf M$ 发出采用 policy $P$ 的 Force。若在 $I=[\tau_0,\tau_0+T_{\rm force}]$ 上：

- Espresso 排序可用且读取可用；
- 按 $P$ 选出的新 session 都是良态的，且所需的跨区域连接连通；
- A-RELAY 成立，

则在 $\tau_0+T_{\rm force}$ 之前 Force 已经生效，并且 $G$ 至少增长了一块。
注意：$\Pi$ 不自动检测区域故障，是否以及何时发出 Force 由治理（即 $\mathcal Z$）决定。

**L4（停机后经由 L1 推进）。**
设 $\mathcal Z$ 在 $\tau_0$ 时指示 $\mathsf M$ 发出 STOP。若在 $I=[\tau_0,\tau_0+T_{\rm stop}]$ 上同时满足：

- Espresso 读取可用（排序可以不可用）；
- $\mathsf C$ 在线；
- 受保护前缀所需的数据在 DA 中可取回；
- A-RELAY 成立，

则：

- (a) 在 $\tau_0+T_{\rm stop}$ 之前，每个观察者的 $G$ 都已进入 DEPOSIT_ONLY 模式；
- (b) 此后，若某个 L1 位置 $p$ 在 $\tau_p\ge\tau_0+T_{\rm stop}$ 时 finalized，并且原生规则使 $p$ 产生 deposit-only 块，那么只要 A-RELAY 在 $[\tau_p,\tau_p+T_{\rm dep}]$ 上成立，这些块就在 $\tau_p+T_{\rm dep}$ 之前进入 $G^u$。

这一目标不要求 Espresso 产生任何新块。

**L5（恢复）。**
设 $\mathcal Z$ 在 $\tau_0$ 时指示 $\mathsf M$ 发出采用 policy $P$ 的 RESUME。若在 $I=[\tau_0,\tau_0+T_{\rm resume}]$ 上：

- Espresso 排序可用且读取可用，$\mathsf C$ 在线；
- 按 $P$ 选出的 session 是良态的，且跨区域连接连通；
- A-RELAY 成立，

则在 $\tau_0+T_{\rm resume}$ 之前，系统回到 NORMAL 模式，并且 $G$ 至少增加一个出处为 $\mathsf{ORD}$ 的块。

**L6（L1 确定的推进）。**
设 $B\in G^u(\tau_0)$。若在 $[\tau_0,\tau_0+T_{\rm batch}]$ 上 $\mathsf C$ 在线、DA 可用，且 A-RELAY 成立，则对每个在该区间上活跃的诚实派生节点 $v$，有 $\mathrm{native}(B)\in D^v(\tau_0+T_{\rm batch})$。

**L7（证据处理）。**
设 $s$ 的拜占庭成员犯下可 slash 的违规，且 $s$ 的某个诚实成员在 $\tau_0$ 时看到了证据。则该违规在 $\tau_0+T_{\rm slash}$ 之前在 L1 上处理完毕（slash，或返回“已处理”）。
协议须保证 $T_{\rm slash}\le T_{\rm ev}$，以便与 S7 衔接。

**L8（释放推进）。**
设一个从未被腐化的 position 在 $\tau_0$ 时请求退出。只要覆盖其最后一个职责窗口的关闭发生，它就在有限时间内被释放。
确切条件待定，见 D5。

### 3.4 依赖表

记 L1-S 为 L1 的全部 safety 性质（X-S1 至 X-S3、L1-S4、L1-S5），E-S 为 E-S1 至 E-S4；L1-L、E-L 为相应的活性性质。

| 目标 | safety 类依赖 | 时间或可用性类依赖 |
| --- | --- | --- |
| S0–S2 | A-CRYPTO, A-GOV, A-CERT, L1-S, E-S, DA-S1 | $T_{\rm L1}$, $\Delta_{\rm ts}$：仅经由控制公告纪律（D1） |
| S3 | A-CRYPTO, A-BYZ, L1-S, E-S | — |
| S4 | S3 的全部依赖, DA-S1, A-CERT | 方案 B：无。方案 A：$T_{\rm L1}$, $T_{\rm DA}$, $T_{\rm cert}$, $T_{\rm ret}$, A-RELAY（D3） |
| S5 | A-CRYPTO，$s$ 良态，以及 S0–S2 的依赖 | $\Delta_i$，作为 safety 前提 |
| S6 | A-CRYPTO, A-PKI, L1-S，持久化存储 | — |
| S7 | A-CRYPTO, A-CERT, L1-S | — |
| L1 | S 类全部依赖，被选中的 session 良态 | E-L, L1-L, $\Delta_i$, $\Delta_{\rm cross}$, $T_{\rm ready}$, A-RELAY |
| L2 | S5 的依赖 | E-L, $\Delta_i$, A-RELAY |
| L3 | 新 session 良态 | $\mathsf M$ 发出 Force, E-L, L1-L, $\Delta_{\rm ts}$, $\Delta_{\rm cross}$, $T_{\rm ready}$, A-RELAY |
| L4 | — | $\mathsf M$ 发出 STOP, L1-L, E 读取可用, $\mathsf C$ 在线, DA-L, $T_{\rm ret}$, A-RELAY |
| L5 | 新 session 良态 | $\mathsf M$ 发出 RESUME, L1-L, E-L, $\mathsf C$ 在线, $\Delta_{\rm cross}$, $T_{\rm ready}$, A-RELAY |
| L6 | — | $\mathsf C$ 在线, DA-L, L1-L, A-RELAY |
| L7 | — | $\Delta_i$, L1-L, A-RELAY |
| L8 | — | L1-L, $\mathsf C$ 在线，关闭事件发生（D5） |

从这张表可以读出三条结构性结论：

1. **全局一致（S0–S2）不依赖任何区域的诚实，也不依赖网络同步。** 它只依赖公共账本的 safety、$\mathsf C$ 与 $\mathsf M$ 的可信，以及（现方案下）控制可见性的时间界。
2. **有效性（S3）与 L1 对应（S4）只依赖拜占庭界 A-BYZ，不依赖成员在线。** 诚实签名者只为原生有效的块签名；离线成员不签任何东西，所以不影响这两条。
3. **快速确认（S5）是唯一依赖区域同步 $\Delta_i$ 的 safety 目标。** 因此它只对良态 session 声称，并且可被治理关闭撤销。

### 3.5 各故障场景下目标的含义

| 场景 | 仍然成立的 safety 目标 | 活性如何恢复 |
| --- | --- | --- |
| 某 session 有少数拜占庭成员（良态） | 全部 | NORMAL 继续，由区域内恢复处理 |
| 整区离线或区域网络中断（A-BYZ 成立，不良态） | S0–S4, S6, S7；该 session 的 S5 不保证 | 治理 Force（L3） |
| 跨区域网络分区 | 全部 | 恢复连通后继续交接 |
| Espresso 停机（排序不可用） | 全部 | STOP 后进入 L1 路径（L4），之后 RESUME（L5） |
| DA 不可用 | 方案 B 下全部成立；方案 A 下 S4 可能失效 | 等待 DA 恢复；L6 暂停 |
| $\mathsf C$ 离线 | 全部 | 批次提交与模式切换暂停，直到 $\mathsf C$ 恢复 |
| 治理沉默 | 全部 | 无：L3–L5 需要治理行动 |
| 违反 A-BYZ、A-L1、A-TS 或 E-S，$\mathsf C$ 或 $\mathsf M$ 被攻破 | 不作承诺（模型之外） | — |

### 3.6 非目标

- 普通交易的包含保证与抗审查。L1 deposit 与 forced inclusion 按原生规则继承。
- 区域之间、leader 之间的公平，以及排序公平或 MEV 相关性质。
- 自动检测 Espresso 停机或区域故障。这些由治理判断。
- Accountable safety，即安全被破坏时必能指认一定权重的违规者。在同步、$F_s<W_s/2$ 的区域协议中，两张证书不必相交。
- 超出保留窗口的数据丢失之后的无损恢复。
- 消息机密性与交易隐私。
- 密码学意义上的可组合性（见 §0.1）。

### 3.7 待讨论的设计选择

**D1｜控制可见性：S1 是否应依赖时间界？**
现方案用公告提前量 $\theta_a>\tau_{\rm submit}+T_{\rm L1}+\Delta_{\rm ts}$，保证观察者在处理 $J_q$ 时已经看到相关控制。代价是 S1 依赖 $T_{\rm L1}$ 与 $\Delta_{\rm ts}$。
另一种做法有两部分：

- $\mathsf{Replay}$ 在处理 $J_q$ 之前，先等自己的 L1 已最终化日志覆盖到时间戳不小于 $\theta_q$；
- 合约拒绝 $\theta_a\le$ 纳入块时间戳的控制。

于是，凡是在 $J_q$ 生效的控制，都在时间戳早于 $\theta_q$ 的 L1 块中，必然已在观察者日志内。这样 S1 只依赖 safety 类假设；代价是 $G$ 的延迟增加，量级约为 L1 finality 的延迟。

**D2｜$\mathsf M$ 的信任范围。**
是否把“即使 $\mathsf M$ 被攻破，S0–S3 仍然成立，只损失活性”列为目标？若是，所有控制纪律都必须由合约或 $\mathsf{Replay}$ 强制执行，而不是依赖 $\mathsf M$ 的诚实。
A-TS 让合约可以用 L1 时间戳检查 $\theta_a$，因此这与 D1 相关。

**D3｜受保护前缀：方案 A 还是方案 B。**

- 方案 A：依靠交付时限，在 sequencing window 内完成批次提交。
- 方案 B：修改 inbox 与 derivation，使其识别并保护已进入 $G$ 的前缀。

这一选择决定 S4 的依赖列，也决定 DA 不可用时 S4 是否仍然成立。

**D4｜快速确认的承诺强度。**
在现方案下，Force 只能从 $\mathrm{last}(G)$ 开始，所以尚未进入 $G$ 的快速确认会被撤销。
需要决定：对用户的承诺是否只到“若无治理关闭则收录”（L2）？还是要求 Force 尽量保留已经快速确认的块？后者会改变 Force 的 anchor 规则。

**D5｜停机期间的职责窗口与释放。**
Espresso 停机且没有 STOP 时，$\tau_s^{\rm close}$ 无界，S7 会让相应 position 的释放无限延迟。
需要决定：L8 的条件是否接受“释放必须等待下一次关闭”这一代价？

**D6｜Espresso 的可用性建模。**
“排序可用”与“读取可用”两种状态是否足够？是否需要把 $T_E$ 再拆成收录时限与可见时限？

---

## 参考文献

- [GKL15] J. Garay, A. Kiayias, N. Leonardos. *The Bitcoin Backbone Protocol: Analysis and Applications.* EUROCRYPT 2015.
- [PS17] R. Pass, E. Shi. *The Sleepy Model of Consensus.* ASIACRYPT 2017.
- [AMNRY20] I. Abraham, D. Malkhi, K. Nayak, L. Ren, M. Yin. *Sync HotStuff: Simple and Practical Synchronous State Machine Replication.* IEEE S&P 2020.
- [CKPS01] C. Cachin, K. Kursawe, F. Petzold, V. Shoup. *Secure and Efficient Asynchronous Broadcast Protocols.* CRYPTO 2001（external validity）。
