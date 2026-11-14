# MegaETH Sequencer Rotation 技术报告 · Part 0

## 模型与假设、外部全局功能、设计目标（中文工作稿 v0.1）

日期：2026-10-25。
状态：迭代草稿。本稿只写三件事：(1) 模型与假设；(2) 外部全局功能；(3) 设计目标。
模块架构与各模块的 functionality 等这三部分定稿后再写。

**相对 v4 §2 与组合式抽象的主要变化**

1. 外部组件写成理想功能（状态、诚实接口、敌手接口、守卫），再推导出编号性质。Submit/Read 接口恢复，以便后续协议框直接调用。
2. 故障分两层：A2 只约束**拜占庭**权重；“良态 session”另外要求成员在线且区域内同步。整区离线因此仍在模型之内，只是该 session 不良态。
3. 跨区域网络允许分区，$\Delta_{\rm cross}$ 只用于活性；$\Delta_i$ 明确标为快速确认安全（S5）的前提。
4. 治理建模为环境输入：$\mathsf M$ 诚实，但“做什么、何时做”由环境 $\mathcal Z$ 决定。safety 目标必须对所有治理行为成立。
5. Espresso 的健康拆成“排序”与“读取”两个标志：STOP 路径要读旧块，但不需要新块。
6. 设计目标分为 S0–S7 与 L1–L8，并给出逐目标的依赖表和三级确认。

---

## 0. 写法与记号

### 0.1 形式化框架

本稿沿用共识与应用密码学文献的标准写法。

1. **执行。**
   一次执行由环境 $\mathcal Z$、敌手 $\mathcal A$ 与协议 $\Pi$ 组成，记作 $\mathrm{EXEC}_{\Pi,\mathcal A,\mathcal Z}(\kappa)$。
   $\Pi$ 在若干**全局功能**的 hybrid 模型中运行。这些功能由所有 session、所有模块、外部观察者乃至其他功能共享，不是每个子协议各有一份；这正是 GUC 意义下的 shared setup [CDPW07]。
2. **外部组件写成理想功能。**
   每个外部组件给出一个功能框：状态、诚实接口、敌手接口，以及功能强制执行的**守卫**。
   守卫规定敌手无法让功能违反的约束：若敌手没有按时推进，功能自己推进。
   框后列出协议可直接引用的编号性质：`-S` 为 safety 类，始终成立；`-L` 为 liveness 类，只在声明的健康条件下成立。
   这一写法来自 [BMTZ17] 的账本功能，这里做了大幅简化。
3. **设计目标写成执行上的性质。**
   目标是关于执行 trace 的谓词（一致、持久、有效、活性），与 [GKL15, PS17] 相同。
   称性质 $P$ 成立，若对所有满足 §1 假设的 PPT $(\mathcal A,\mathcal Z)$，有 $\Pr[\mathrm{EXEC}\text{ 违反 }P]\le\mathrm{negl}(\kappa)$。
4. **不声称 UC realization。**
   功能框只用来精确界定外部组件的能力。本协议各模块之间的组合采用 assume–guarantee 论证（后续 Part 给出），不构造 simulator。

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
| $\mathcal W_s$ | session $s$ 的职责窗口（定义 1.2）。 |
| $\mathsf M$ | 治理方（multisig），作为一个逻辑参与方。 |
| $X\preceq Y$ | 序列 $X$ 是 $Y$ 的前缀。 |
| $X\sim Y$ | $X\preceq Y$ 或 $Y\preceq X$（可比较）。 |
| $G^u(\tau)$ | 诚实观察者 $u$ 在 $\tau$ 时输出的全局链（§3.1）。 |
| $D^u(\tau)$ | 诚实派生节点 $u$ 在 $\tau$ 时由 L1 与 DA 派生出的原生链（§3.1）。 |

---

## 1. 系统模型与假设

### 1.1 参与方与角色

- **区域。** $\mathcal R$ 是 genesis 固定的有限区域集合。
- **运营者与 position。** 质押注册产生 position $o$，绑定签名公钥 $pk_o$ 与区域 $i_o\in\mathcal R$。下文的“成员”指 position 的运营者。
- **治理方 $\mathsf M$。** multisig 委员会，作为**一个**逻辑参与方建模（A4）。
- **角色。** 用户、publisher、batcher、settlement 声明者、观察者都是角色而非身份。任何参与方，包括被腐化的参与方，都可以承担这些角色；一个运营者可以兼任多个角色。
- **L1 合约不是参与方。** Authority、Inbox 等是部署在 $\mathcal G_{\rm L1}$ 上的确定性程序，属于 $\Pi$ 的一部分。
- **可信 certifier 不是参与方。** 它是功能 $\mathcal F_{\rm cert}$（§2.5）。

环境 $\mathcal Z$ 提供全部外部输入：用户交易、质押请求，以及交给 $\mathsf M$ 的**治理指令**（A4）。
$\mathcal Z$ 与 $\mathcal A$ 可以任意通信。

### 1.2 时间

全局时钟 $\mathcal G_{\rm clk}$ 给出真实时间 $\tau\in\mathbb R_{\ge 0}$，所有参与方读到同一 $\tau$。
有界时钟偏差可以按标准做法并入下文的 $\Delta$。

本稿区分四种坐标，它们彼此不能互换：

1. 真实时间 $\tau$；
2. L1 高度 $\ell$ 及时间戳 $\mathit{ts}_\ell$；
3. Espresso 高度 $q$ 及时间戳 $\theta_q$；
4. L2 高度 $h$。

### 1.3 生产权限的抽象：session 与委员会

为了陈述假设与目标，这里只引入最少的 session 概念。session 怎样选出、怎样交接，属于后续模块。

**定义 1.1（session）。**
$\Pi$ 把普通出块划分为 session。每个 session $s$ 确定：

- 区域 $i_s\in\mathcal R$ 与固定委员会 $V_s\subseteq\{o: i_o=i_s\}$；
- 每个成员的正整数权重 $w_o$，取自某个冻结的质押快照；总权重 $W_s=\sum_{o\in V_s}w_o$；
- 故障门槛 $F_s$，满足 $0\le F_s<W_s/2$。

$s$ 的**证书**是来自 $V_s$ 中不同成员、总权重严格大于 $F_s$ 的一组匹配签名。
这些字段一经确定便不再改变；之后的 slash 或新快照都不改写它们。

**定义 1.2（职责窗口）。**
$\mathcal W_s=[\tau_s^{\rm open},\tau_s^{\rm close}]$，其中：

- $\tau_s^{\rm open}$ 是任何参与方首次能够确定 $V_s$ 的时刻；
- $\tau_s^{\rm close}$ 是已最终化的公开数据（$\mathcal G_{\rm L1}$ 与 $\mathcal G_{\rm E}$ 上的记录）首次确定“$s$ 不能再向全局链追加任何块”的时刻。这有两种情况：$s$ 的最后一块已进入全局链且下一责任已经确定；或者覆盖 $s$ 的治理关闭已经生效。精确判定由 §3 的公开重算规则给出。

Espresso 停机且没有 STOP 时，$\tau_s^{\rm close}$ 可能无界（见 D5）。

### 1.4 腐化与“正确”

**腐化模型。**
敌手可以**自适应地**在任意时刻腐化任意参与方，立即取得其全部状态，包括私钥与历史状态（不假设安全擦除），此后完全控制它。腐化不可撤销。
$\mathsf M$ 与外部功能不可被腐化（A4–A6）。

**崩溃与持久化。**
诚实参与方可能崩溃或离线，之后重新上线。其持久化存储不丢失已提交的写入，包括签名记录。

**定义 1.3（拜占庭与正确）。** 对 session $s$ 与成员 $o\in V_s$：

- $o$ **对 $s$ 拜占庭**：$o$ 在 $\tau_s^{\rm close}$ 或之前被腐化。
- $o$ **对 $s$ 正确**：在整个 $\mathcal W_s$ 内，$o$ 未被腐化、持续在线，与 $V_s$ 的其他正确成员保持区域内同步连接（§1.5），并按 $\Pi$ 运行。若 $o$ 在 $\mathcal W_s$ 开始前离线过，则它在 $\tau_s^{\rm open}$ 之前已同步完所需的公开历史。
- 其余成员是诚实的，但离线、掉队或崩溃过；它们既不是拜占庭的，也不是正确的。

**定义 1.4（良态 session）。**
$s$ 是良态的，当且仅当

$$
\sum_{o\in V_s,\ o\text{ 对 }s\text{ 不正确}} w_o\ \le\ F_s .
$$

良态是关于一次执行的**谓词**，不是全局假设。
区域整体离线或区域网络中断时，相应 session 不良态，但这仍在模型之内（A2 依然可以成立）。
快速确认与 NORMAL 活性只对良态 session 声称（§3）。

### 1.5 通信

参与方之间使用认证点对点信道 $\mathcal F_{\rm net}$：敌手不能伪造或篡改诚实方的消息，但可以重排、重复消息，并在下述时限内决定延迟。信道不提供机密性。
消息大小及其随附的验证依赖不超过声明的上界 $m_{\max}$；$\Delta$ 包含接收方对这些依赖的验证处理。

- **区域内同步 $\Delta_i$。**
  对 $s$ 的任意两个正确成员，于 $\tau$ 发出的消息在 $\tau+\Delta_{i_s}$ 之前送达并处理完毕。
  这一时限既用于区域活性，**也是快速确认安全（S5）的前提**，因为同步 BFT 的安全性依赖它 [AMNRY20]。
- **跨区域 $\Delta_{\rm cross}$。**
  跨区域网络可以暂时分区。在**连通期间**，任意两个在线诚实方之间于 $\tau$ 发出的消息在 $\tau+\Delta_{\rm cross}$ 之前送达。
  $\Delta_{\rm cross}$ 只用于活性。

大对象（例如可执行状态）不受 $\Delta$ 约束，见 §1.6 的 $T_{\rm ready}$。

### 1.6 本地计算与资源

- **本地计算视为瞬时，有一个例外。** 为给定 anchor 准备可直接执行的状态（下载、重建、验证、安装）需要时间。在所需输入可得、大小与负载在声明范围内时，这一步不超过 $T_{\rm ready}$。
- **费用。** 诚实方持有足够的资金，用以支付其职责所需的 L1、Espresso 与 DA 费用。

### 1.7 假设汇总

**A1（密码学）。**
签名方案满足 EUF-CMA 安全；哈希函数抗碰撞；承诺具有绑定性。敌手是 PPT 的。

**A2（区域拜占庭界）。**
对 $\Pi$ 产生的每个 session $s$：

$$
\sum_{o\in V_s,\ o\text{ 对 }s\text{ 拜占庭}} w_o\ \le\ F_s .
$$

腐化按累计计算：在 $s$ 的证书仍可能影响全局链期间被攻陷的密钥，都要计入。

**A3（PKI）。**
公钥经 $\mathcal G_{\rm L1}$ 上的 Authority 注册，并附 proof of possession。诚实方诚实地生成密钥。

**A4（治理方）。**
$\mathsf M$ 从不被腐化。它只针对 $\mathcal Z$ 发来的治理指令签署控制记录，并遵守 $\Pi$ 为 $\mathsf M$ 规定的控制纪律（例如公告提前量；具体规则在 Authority Part 中给出）。
**治理决定的内容与时间由 $\mathcal Z$ 选择。** 因此 safety 目标必须对所有治理行为成立；活性目标只在明确写出的治理条件下声称。

**A5（可信 certifier）。**
$\mathcal F_{\rm cert}$ 按 §2.5 运行，不可被腐化。

**A6（外部功能）。**
$\mathcal G_{\rm L1}$、$\mathcal G_{\rm E}$、$\mathcal G_{\rm DA}$、$\mathcal G_{\rm set}$ 按 §2 运行；敌手只能经由它们的敌手接口影响它们。

**A7（中继，仅用于活性）。**
对每项中继职责——向 Espresso 发布完整区域结果、提交批次、维护 settlement 声明、向 L1 提交证据——在相关时间窗内至少有一个在线诚实方承担。正确的区域成员可以承担这些职责。

### 1.8 模型之外

以下情形不在本稿范围内，相关目标对此不作任何承诺：

- L1 的 safety 或 finality 失效，或 L1 的活性时限 $T_{\rm L1}$ 不成立；
- Espresso 的 safety 失效，即出现冲突的 finalized 历史（模型内只有 Espresso 的活性失效，见 §2.3）；
- $\mathcal F_{\rm cert}$ 或 $\mathsf M$ 被攻破；
- A2 被违反；
- 保留窗口之外、且没有任何诚实方保存的数据丢失。

---

## 2. 外部全局功能

### 2.0 共同约定

- **共享。**
  下列功能对所有参与方、所有 session 及其他功能都是同一个实例。
  $\mathcal F_{\rm cert}$ 可以读取各账本与 DA。$\mathcal G_{\rm L1}$ 上的合约可以调用 $\mathcal F_{\rm cert}.\textsf{Verify}$；在实现中，这就是验证已注册 TEE 密钥的签名。
- **健康标志。**
  允许停机的功能带有由 $\mathcal A$ 控制的健康标志。称功能在区间 $I$ 上健康，若相应标志在 $I$ 上恒为 1。
  守卫只在健康期间强制执行。**safety 性质与健康状态无关。**
- **视图与返回值。**
  每个参与方对每个账本有一个只增不减的视图指针，读取只返回视图内的前缀。由此统一返回值约定：
  - 视图内可判定的查询，返回值或经验证的 $\bot$（“已证明不存在”）；
  - 超出视图的查询，返回 `WAIT`；
  - 格式错误的请求，返回 `REJECT`。
  `WAIT` 不等于 $\bot$。
- **准入。**
  “提交”指请求已到达功能，且格式正确、费用充足。本地尝试发送不算提交。
- **在线。**
  称参与方 $P$ 在区间 $I$ 上活跃，若它在整个 $I$ 上诚实且在线。

### 2.1 账本模板 $\mathcal G_{\rm Ledger}[\mathsf V,T,\varepsilon]$

L1 与 Espresso 都是这个模板的实例。

> **功能 $\mathcal G_{\rm Ledger}[\mathsf V,T,\varepsilon]$**
>
> **参数.** 有效性谓词 $\mathsf V$；服务时限 $T$；时间戳超前量 $\varepsilon$。
>
> **状态.**
> - 最终化日志 $\mathbf L=\langle b_1,b_2,\dots\rangle$，其中 $b_k=(k,\mathit{ts}_k,\mathit{txs}_k)$；同时记录每块的最终化时刻 $\tau_{\rm fin}(k)$。
> - 每个参与方 $P$ 的视图指针 $p_P$，初值为 $0$。
> - 待处理集合 $\mathsf{Pend}$，元素为 $(tx,\tau_{\rm sub},P)$。
> - 若实例允许停机：健康标志 $\mathsf{up}_{\rm ord},\mathsf{up}_{\rm read}\in\{0,1\}$；否则两者恒为 $1$。
>
> **诚实接口.**
> - $\textsf{Submit}(tx)$：将 $(tx,\tau,P)$ 加入 $\mathsf{Pend}$，并把 $tx$ 泄露给 $\mathcal A$。
> - $\textsf{Read}()$：返回 $\mathbf L[1..p_P]$。
> - $\textsf{Prove}(\phi)$：$\phi$ 是关于 $\mathbf L[1..p_P]$ 的陈述，例如“第 $a$ 至 $b$ 块在范围 $X$ 内的完整内容为 $D$”。若 $\phi$ 为真，返回可转发的凭证 $\pi$。
> - $\textsf{Verify}(\phi,\pi)$：当且仅当 $\pi$ 由 $\textsf{Prove}(\phi)$ 生成时返回 $1$。
>
> **敌手接口.**
> - $\textsf{Extend}(\mathit{txs},\mathit{ts})$：若 $\mathit{txs}$ 中每一笔相对 $\mathbf L$ 满足 $\mathsf V$，且 $\mathit{ts}_{|\mathbf L|}\le\mathit{ts}\le\tau+\varepsilon$，则追加块 $(|\mathbf L|+1,\mathit{ts},\mathit{txs})$，并记 $\tau_{\rm fin}:=\tau$。
> - $\textsf{Advance}(P,p)$：若 $p_P\le p\le|\mathbf L|$，则置 $p_P:=p$。
> - $\textsf{SetHealth}(\cdot)$：仅在实例允许停机时可用。
>
> **守卫.** 功能在每个时刻检查以下条件；若敌手没有做到，功能自行追加或推进视图。
> - **收录。** 设诚实方于 $\tau'$ 提交了准入请求 $tx$，且两个健康标志在 $[\tau',\tau'+T]$ 上恒为 $1$。则到 $\tau'+T$ 时，$tx$ 已在 $\mathbf L$ 中，并对在该区间上活跃的诚实方可见。
> - **可见。** 设第 $k$ 块在 $\tau_0$ 前已最终化，且 $\mathsf{up}_{\rm read}$ 在 $[\tau_0,\tau_0+T]$ 上恒为 $1$。则到 $\tau_0+T$ 时，每个在该区间上活跃的诚实方都有 $p_P\ge k$。
> - **推进。** 若 $\mathsf{up}_{\rm ord}$ 在 $[\tau,\tau+T]$ 上恒为 $1$，则到 $\tau+T$ 时，$\mathbf L$ 中存在时间戳 $\mathit{ts}\ge\tau$ 的块。

**派生性质。** 以下性质对任意实例成立。S 类始终成立；L 类只在相应健康条件下成立。

- **LED-S1（一致）。** 对任意诚实方 $P,Q$ 与时刻 $\tau,\tau'$，有 $\mathrm{view}_P(\tau)\sim\mathrm{view}_Q(\tau')$。
- **LED-S2（持久）。** 若 $\tau\le\tau'$，则 $\mathrm{view}_P(\tau)\preceq\mathrm{view}_P(\tau')$。
- **LED-S3（完整读取）。** 视图内任意区间、任意范围的内容都可以完整取得，因此视图内的“不存在”是可判定的。
- **LED-S4（时间戳）。** 时间戳非减，且 $\mathit{ts}_k\le\tau_{\rm fin}(k)+\varepsilon$。
- **LED-S5（凭证可靠）。** $\textsf{Verify}$ 只接受对 $\mathbf L$ 为真的陈述。
- **LED-L1 / L2 / L3。** 分别是收录、可见、推进三条守卫。

**注。**
账本只按 $\mathsf V$ 检查，不理解本协议的任何语义，也不提供 chain quality 或排序公平。在守卫之外，日志的内容与顺序完全由敌手决定，可以包含任意垃圾输入。
$\textsf{Prove}/\textsf{Verify}$ 只为活性服务：落后的参与方可以凭转发来的凭证立即使用来源记录，而不必等待自己的视图追上。safety 论证从不需要它们。

### 2.2 Ethereum L1：$\mathcal G_{\rm L1}$

$$
\mathcal G_{\rm L1}:=\mathcal G_{\rm Ledger}[\mathsf V_{\rm EVM},\,T_{\rm L1},\,\varepsilon_{\rm L1}],
$$

不带健康标志，即始终健康。在模板之上附加三点：

- **应用状态。** $\mathsf{St}_{\rm L1}(\ell)=\mathsf{Apply}_{\rm EVM}(\mathbf L[1..\ell])$ 是确定性函数。合约调用按日志顺序原子执行，可以执行失败，失败结果同样被收录。已部署的合约代码在模型中固定不变。
- **时间戳严格递增。**
- **只使用 finalized 块。** 视图即已 finalized 的前缀；$T_{\rm L1}$ 覆盖从提交到 finalized、再到对活跃诚实方可见的全过程。

编号性质：

- **L1-S1（一致与持久）。** 即 LED-S1 与 LED-S2。
- **L1-S2（完整读取）。** 对视图内任意高度 $\ell$，可以读取合约状态，以及按顺序排列的完整事件列表（可为空）。
- **L1-S3（合约忠实执行）。** 调用结果（包括失败）由 $\mathsf{Apply}_{\rm EVM}$ 唯一确定。
- **L1-S4（时间戳）。** 时间戳严格递增，且 $\mathit{ts}_\ell\le\tau_{\rm fin}(\ell)+\varepsilon_{\rm L1}$。
- **L1-L1（收录）。** 准入请求在 $T_{\rm L1}$ 内 finalized，并对活跃诚实方可见。
- **L1-L2（推进）。** L1 始终在出块，其时间戳追上真实时间。

### 2.3 Espresso：$\mathcal G_{\rm E}$

$$
\mathcal G_{\rm E}:=\mathcal G_{\rm Ledger}[\mathsf V_{\rm any},\,T_E,\,\varepsilon_E],
$$

带两个健康标志。在模板之上附加：

- **Namespace。** 交易形如 $(ns,\mathit{bytes})$。$\textsf{Read}_{ns}$ 按块顺序给出 namespace $ns$ 的完整内容 $M^{ns}_q$，可以为空。
- **记号。** $J_q:=b_q$，$\theta_q:=\mathit{ts}_q$，$\tau_E(q):=\tau_{\rm fin}(q)$。
- **停机。** 定义为 $\mathsf{up}_{\rm ord}=0$：此时不再保证新的收录与推进。$\mathsf{up}_{\rm read}$ 单独描述已 finalized 的块能否被读取。

编号性质：

- **E-S1（一致与持久）。** 始终成立，包括停机期间。
- **E-S2（完整 namespace）。** 每块中本链 namespace 的内容（包括“为空”）都可以完整读取。
- **E-S3（时间戳）。** $\theta_q$ 非减，且 $\theta_q\le\tau_E(q)+\varepsilon_E$。
- **E-L1（收录）。** 若两个健康标志在 $[\tau,\tau+T_E]$ 上恒为 $1$，则于 $\tau$ 提交的 publication 在 $\tau+T_E$ 之前被收录，并对活跃诚实方可见。
- **E-L2（可见）。** 若 $\mathsf{up}_{\rm read}$ 在相应区间上为 $1$，已 finalized 的块在 $T_E$ 内对活跃诚实方可见。停机期间这一条仍可成立。
- **E-L3（推进）。** 若 $\mathsf{up}_{\rm ord}$ 在 $[\tau,\tau+T_E]$ 上恒为 $1$，则 $\tau+T_E$ 之前存在 $\theta\ge\tau$ 的已 finalized 块。

**注。** Espresso 只负责排序，不验证本协议的任何内容。本链 namespace 中的数据可以由任何人提交，包括敌手。

### 2.4 数据可用性：$\mathcal G_{\rm DA}$

> **功能 $\mathcal G_{\rm DA}[T_{\rm DA},\Delta_{\rm DA},T_{\rm ret}]$**
>
> **状态.** 存储表 $\mathsf{Store}:d\mapsto(\mathit{payload},\tau_{\rm cert})$；健康标志 $\mathsf{up}_{\rm DA}$。
>
> **诚实接口.**
> - $\textsf{Disperse}(\mathit{payload})$：计算承诺 $d:=\mathsf{Com}(\mathit{payload})$。在敌手选定的时刻 $\tau$ 认证，即置 $\mathsf{Store}[d]:=(\mathit{payload},\tau)$，并向提交者返回 $d$。
> - $\textsf{IsCert}(d)$：返回 $\tau_{\rm cert}$；若 $d$ 未被认证，返回 $\bot$。
> - $\textsf{Retrieve}(d)$：在敌手选定的时刻返回 $\mathsf{Store}[d].\mathit{payload}$。在返回之前为 `WAIT`。
>
> **敌手接口.** 选择上述延迟；$\textsf{SetHealth}$。敌手**不能**修改 $\mathsf{Store}$。
>
> **守卫.**
> - **认证。** 若 $\mathsf{up}_{\rm DA}$ 在 $[\tau,\tau+T_{\rm DA}]$ 上恒为 $1$，则于 $\tau$ 提交的 dispersal 在 $\tau+T_{\rm DA}$ 之前被认证。
> - **取回。** 若 $d$ 已认证、$\tau\le\tau_{\rm cert}+T_{\rm ret}$，且 $\mathsf{up}_{\rm DA}$ 在 $[\tau,\tau+\Delta_{\rm DA}]$ 上恒为 $1$，则于 $\tau$ 发出的 $\textsf{Retrieve}(d)$ 在 $\tau+\Delta_{\rm DA}$ 之前返回。

编号性质：

- **DA-S1（绑定）。** 每个 $d$ 至多对应一个 payload；$\textsf{Retrieve}(d)$ 只会返回这个 payload，绝不返回其他字节。
- **DA-S2（认证一致）。** 一旦 $d$ 被认证，所有参与方此后调用 $\textsf{IsCert}(d)$ 都得到同一个 $\tau_{\rm cert}$。
- **DA-L1（认证时限）** 与 **DA-L2（保留窗口内的取回）**：即上述两条守卫。

**注。**
L1 合约不能直接调用 $\textsf{IsCert}$；DA 绑定通过 $\mathcal F_{\rm cert}$ 的证书带到 L1 上。
超出保留窗口之后，数据能否取回由 $\Pi$ 自己的保管义务负责。

### 2.5 可信 certifier：$\mathcal F_{\rm cert}[\mathsf{prog}]$

> **功能 $\mathcal F_{\rm cert}[\mathsf{prog},T_{\rm cert}]$**
>
> **参数.** 确定性程序 $\mathsf{prog}$，由 $\Pi$ 规定。它包括批次对应、模式边界、关闭确认等业务规则，在后续 Part 中给出。
>
> **状态.**
> - 对 $\mathcal G_{\rm L1}$ 与 $\mathcal G_{\rm E}$ 的视图指针，受相同可见性守卫约束；可以访问 $\mathcal G_{\rm DA}$。
> - 已签发集合 $\mathsf{Issued}$；健康标志 $\mathsf{up}_{\rm cert}$。
>
> **诚实接口**（任何参与方都可以调用，包括被腐化的参与方）.
> - $\textsf{Certify}(x)$：令 $V$ 为当前视图，计算 $y\leftarrow\mathsf{prog}(x,V)$。若 $y\in\{\texttt{WAIT},\texttt{REJECT}\}$，原样返回。否则令 $\chi:=(\texttt{cert},y)$，将 $\chi$ 加入 $\mathsf{Issued}$ 并返回 $\chi$。返回时刻由敌手选择。
> - $\textsf{Verify}(\chi)$：当且仅当 $\chi\in\mathsf{Issued}$ 时返回 $1$。参与方与 $\mathcal G_{\rm L1}$ 上的合约都可以调用。
>
> **敌手接口.** 推进视图指针；选择返回延迟；$\textsf{SetHealth}$。敌手不能让功能输出 $\mathsf{prog}$ 之外的结论。
>
> **守卫.**
> - **响应。** 若 $\mathsf{up}_{\rm cert}$ 在 $[\tau,\tau+T_{\rm cert}]$ 上恒为 $1$，则于 $\tau$ 发出的调用在 $\tau+T_{\rm cert}$ 之前返回。

编号性质：

- **CERT-S1（正确）。** 若 $\textsf{Verify}(\chi)=1$，则 $\chi=(\texttt{cert},y)$，且存在输入 $x$ 与视图 $V$，使 $y=\mathsf{prog}(x,V)$。其中 $V$ 是真实账本历史的前缀，DA 内容取自真实的 $\mathsf{Store}$。
- **CERT-S2（不可伪造）。** $\mathsf{Issued}$ 之外的任何值都不被 $\textsf{Verify}$ 接受。
- **CERT-L1（响应时限）。** 即响应守卫。

**注。**

1. 信任 TEE，意味着这里不定义 attestation、measurement、密钥注册等硬件接口。若以后要降低这一信任，可以改用 [PST17] 的 attested execution 抽象。
2. $\mathcal F_{\rm cert}$ 只保证“按 $\mathsf{prog}$ 正确计算”，不保证 $\mathsf{prog}$ 本身正确。例如，共同模式边界的正确性仍然是 $\Pi$ 的证明义务。
3. $\mathcal F_{\rm cert}$ 的视图可能落后于其他参与方。因此 $\Pi$ 必须让 $\mathsf{prog}$ 的输出显式绑定所用的来源切面，并且对视图前进单调：同一个 $x$ 在更长的视图下不得给出相矛盾的结论。
4. 任何人都可以调用 $\textsf{Certify}$，所以 safety 论证不能依赖“由谁来调用”。

### 2.6 原生执行：确定性函数，而非功能

- **执行函数。** $\mathsf{Exec}(\mathit{st},x)\to(\mathit{st}',N)$ 或 $\bot$。其中 $x$ 包括有序的普通交易列表，以及原生上下文：parent、高度、timestamp、L1 origin、deposits、系统交易。$\mathsf{Exec}$ 是固定 rule version 下的确定性函数。块内交易 revert 不会使整块无效。
- **状态承诺。** genesis 状态为 $\mathit{st}_0$；$\mathsf{Root}(\mathit{st})$ 是绑定的状态承诺。
- **有效原生链。** 一个原生块序列是有效的，若其中每一块都能相对前一状态被 $\mathsf{Exec}$ 接受。
- **派生函数。** $\mathsf{Derive}(\text{L1 前缀},\mathsf{Store}_{\rm DA})\to$ 原生块序列。$\mathsf{Derive}$ 由 $\Pi$ 规定，包括 inbox 的接纳规则与模式事件的解释，在 Settlement Part 中给出。Part 0 对它只提两项要求：
  - 对 L1 前缀单调；
  - 所需的 DA 数据缺失时，停在缺失处，不越过。

这里没有任何假设，只要求所有诚实方使用同一个固定的 rule version。

### 2.7 状态结算：$\mathcal G_{\rm set}[\mathsf{Derive}]$

> **功能 $\mathcal G_{\rm set}[\mathsf{Derive},T_{\rm set}]$**（部署在 L1 上；结果可经由 $\mathcal G_{\rm L1}$ 读取）
>
> **诚实接口.** $\textsf{Claim}(k,r,\ell)$：声称“由 L1 前缀 $\mathbf L_{\rm L1}[1..\ell]$ 与 DA 内容派生出的前 $k$ 个原生块，执行后的状态根为 $r$”。
>
> **正确性判定.** 令 $N=\mathsf{Derive}(\mathbf L_{\rm L1}[1..\ell],\mathsf{Store}_{\rm DA})$。若 $|N|\ge k$ 且 $r=\mathsf{Root}(\mathit{st}(N[1..k]))$，称该声明正确。
>
> **守卫.**
> - **可靠性。** 只有正确的声明可能被接受。
> - **结算时限。** 若有活跃诚实方维护某个正确声明，它在 $T_{\rm set}$ 内被接受。

编号性质：**SET-S1（可靠）**、**SET-L1（结算时限）**。

**注。**
$\mathcal G_{\rm set}$ 是对现有 Kailua / fault-proof 机制的抽象，本协议不修改其争议逻辑。
集成要求是：其派生程序必须实现与 $\Pi$ 相同的 $\mathsf{Derive}$。

### 2.8 参数表

| 参数 | 含义 | 计时起点 | 何时成立 | 是否用于 safety |
| --- | --- | --- | --- | --- |
| $\Delta_i$ | 区域内送达并处理 | 发送 | $s$ 的正确成员之间，始终 | **是**（S5） |
| $\Delta_{\rm cross}$ | 跨区域送达并处理 | 发送 | 连通期间 | 否 |
| $T_{\rm ready}$ | 准备精确 anchor 的可执行状态 | 输入可得 | 大小与负载在声明范围内 | 否 |
| $T_{\rm L1}$ | L1 收录、finalized 并可见 | 提交 | 始终 | 现方案经控制纪律间接使用（D1） |
| $\varepsilon_{\rm L1}$ | L1 时间戳超前量 | — | 始终 | 视 D1 的选择而定 |
| $T_E$ | Espresso 收录并可见 | 提交 | 健康期间 | 否 |
| $\varepsilon_E$ | Espresso 时间戳超前量 | — | 始终 | 现方案经控制纪律间接使用（D1） |
| $T_{\rm DA}$ | DA 认证 | 提交 | 健康期间 | 方案 A 中用于 S4（D3） |
| $\Delta_{\rm DA}$ | DA 取回 | 请求 | 健康期间、保留窗口内 | 方案 A 中用于 S4 |
| $T_{\rm ret}$ | DA 保留窗口 | 认证 | 始终 | 方案 A 中用于 S4 |
| $T_{\rm cert}$ | certifier 响应 | 调用 | 健康期间 | 方案 A 中用于 S4 |
| $T_{\rm set}$ | 结算判定 | 声明 | 有诚实维护者 | 否 |

另有一组由协议推导的时限，在后续 Part 中给出：$T_{\rm grow}$、$T_{\rm incl}$、$T_{\rm force}$、$T_{\rm stop}$、$T_{\rm dep}$、$T_{\rm resume}$、$T_{\rm batch}$、$T_{\rm slash}$、$T_{\rm release}$，以及证据窗口 $T_{\rm ev}$。
它们必须是上表参数的有限函数。

---

## 3. 设计目标

### 3.1 系统输出与三级确认

**定义 3.1（全局块与出处）。**
全局链 $G$ 是 genesis 之后的块序列。每个块 $B$ 带有不可变的出处

$$
\sigma(B)\in\{\mathsf{ORD}(s),\ \mathsf{DEP}(p)\}.
$$

- $\mathsf{ORD}(s)$：由 session $s$ 生产的普通块；
- $\mathsf{DEP}(p)$：Espresso 停机后，由 L1 位置 $p$ 处的输入确定性派生出的 deposit-only 块。

$\mathrm{native}(B)$ 表示 $B$ 对应的原生 L2 块。$G$ 上的前缀关系同时比较块与出处。

**定义 3.2（观察者输出）。**
诚实观察者 $u$ 运行 $\Pi$ 的公开观察算法，只读取 $\mathcal G_{\rm L1}$、$\mathcal G_{\rm E}$、$\mathcal G_{\rm DA}$。它在 $\tau$ 时的输出记为 $G^u(\tau)$。

**定义 3.3（快速确认）。**
session $s$ 的正确成员 $v$ 在 $\tau$ 时输出 $\mathsf{Conf}_v(s,B)$，表示 $v$ 在区域内提交（commit）了块 $B$。

**定义 3.4（L1 派生链）。**
诚实派生节点 $u$ 在 $\tau$ 时的输出为

$$
D^u(\tau):=\mathsf{Derive}\bigl(\mathrm{view}^u_{\rm L1}(\tau),\ u\text{ 已取回的 DA 数据}\bigr).
$$

**定义 3.5（治理关闭）。**
治理控制（Force 或 STOP）生效时，会关闭一组 session。其生效位置由公开重算规则确定，落在 $G$ 的两个相邻位置之间；所有诚实观察者对这一位置的判定一致。

对用户而言，系统提供三级确认：

| 级别 | 输出 | 由谁给出 | 延迟量级 | 可被什么推翻 |
| --- | --- | --- | --- | --- |
| 快速确认 | $\mathsf{Conf}_v(s,B)$ | $s$ 的正确成员 | 若干 $\Delta_i$ | 治理关闭在 $B$ 进入 $G$ 之前生效；或 $s$ 不良态 |
| 全局最终 | $B\in G^u(\tau)$ | 任意诚实观察者 | 发布 + Espresso | 不可推翻 |
| L1 结算 | 状态根被 $\mathcal G_{\rm set}$ 接受 | L1 | 批次 + DA + 结算 | 不可推翻 |

### 3.2 安全目标

除非另行注明，以下目标在 §1 的全部假设下**无条件**成立：对任意治理行为，以及 Espresso、DA、certifier 的任意健康模式，都成立。

**S0（公开可重算）。**
存在确定性函数 $\mathsf{Replay}$，使对每个诚实观察者 $u$ 与每个时刻 $\tau$，

$$
G^u(\tau)=\mathsf{Replay}\bigl(\mathrm{view}^u_{\rm L1}(\tau),\ \mathrm{view}^u_{E}(\tau),\ \mathrm{DA}^u(\tau)\bigr).
$$

换言之，$G$ 不依赖任何私有输入、消息到达顺序，或有自由裁量权的服务。

**S1（全局一致）。**
对任意诚实观察者 $u,v$ 与任意时刻 $\tau,\tau'$：$G^u(\tau)\sim G^v(\tau')$。

**S2（持久）。**
若 $\tau\le\tau'$，则 $G^u(\tau)\preceq G^u(\tau')$。特别地，已进入 $G$ 的块及其出处永不改变。

**S3（有效）。** 这一条对应 [CKPS01] 的 external validity。对任意诚实观察者 $u$、时刻 $\tau$：

- (a) **原生有效。** $\mathrm{native}(G^u(\tau))$ 是一条从 $\mathit{st}_0$ 出发的有效原生链。
- (b) **授权。** 对每个 $\sigma(B)=\mathsf{ORD}(s)$ 的块：$s$ 是公开规则在该位置指定的负责 session，并且 $B$ 带有 $s$ 的区域最终证书（直接证书，或经同 session 后代覆盖的证书）。
- (c) **派生正确。** 对每个 $\sigma(B)=\mathsf{DEP}(p)$ 的块：$\mathrm{native}(B)$ 等于原生规则由截至 $p$ 的 L1 输入确定性派生出的块。

**S4（结算对应）。**
对任意诚实观察者 $u$、诚实派生节点 $v$ 与时刻 $\tau,\tau'$：

$$
\mathrm{native}(G^u(\tau))\ \sim\ D^v(\tau').
$$

结合 SET-S1 可以得到：任何被接受的高度 $k$ 状态根，都等于 $\mathrm{native}(G)$ 前 $k$ 块执行后的状态根。
这条性质的强度取决于 D3 的选择：在方案 A 下它依赖时间界，在方案 B 下不依赖。

**S5（快速确认的兑现）。** 设 $s$ 是良态 session。

- (a) $s$ 的两个正确成员不会在同一高度快速确认不同的块。
- (b) 若 $s$ 的正确成员输出 $\mathsf{Conf}_v(s,B)$，则对任意诚实观察者 $u$ 与时刻 $\tau$：只要 $G^u(\tau)$ 中有出处为 $\mathsf{ORD}(s)$、高度为 $h(B)$ 的块，那个块就是 $B$。

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
设视图三元组按分量取前缀序 $\sqsubseteq$。若 $\mathsf{Replay}$ 单调，即 $V\sqsubseteq V'\Rightarrow\mathsf{Replay}(V)\preceq\mathsf{Replay}(V')$，那么 S1 与 S2 直接由 LED-S1、LED-S2 与 DA-S1 推出：任意两个诚实视图都 $\sqsubseteq$ 它们按分量取的较长者，从而两份输出都是同一序列的前缀。
困难在于跨两本账本的单调性。观察者在处理 $J_q$ 时，可能尚未看到会影响 $J_q$ 的 L1 控制。
现方案用控制公告提前量保证：凡是能在诚实方处同时出现的视图组合，都已包含相关控制。因此 $\mathsf{Replay}$ 只在**可达视图**上单调，S1 也就间接依赖 $T_{\rm L1}$ 与 $\varepsilon_E$（见 D1）。
STOP/RESUME 的共同边界规则要满足的，也正是这条单调性。

### 3.3 活性目标

统一形式为：“若条件 $C$ 在区间 $I=[\tau_0,\tau_0+T]$ 上成立，则在 $\tau_0+T$ 时有结论。”
其中 $T$ 是由协议推导的时限，后续 Part 会把它写成 §2.8 参数的有限函数。
下文的“观察者”均指在 $I$ 上活跃的诚实观察者。

**L1（NORMAL 增长）。** 若在 $I=[\tau_0,\tau_0+T_{\rm grow}]$ 上同时满足：

1. 没有治理控制生效，且系统处于 NORMAL 模式；
2. $\mathcal G_{\rm E}$ 健康；
3. 公开规则能选出 session，即当前 policy 中至少有一个 ON 区域的权重非零；
4. 在 $I$ 内被选中的 session 都是良态的，并且它们交接所需的跨区域连接是连通的；
5. A7 成立，

则每个观察者都有 $|G^u(\tau_0+T_{\rm grow})|>|G^u(\tau_0)|$。

**L2（快速确认的收录）。**
设 $s$ 的正确成员在 $\tau_0$ 时输出 $\mathsf{Conf}_v(s,B)$，且 $s$ 良态。若在 $I=[\tau_0,\tau_0+T_{\rm incl}]$ 上：

- 没有覆盖 $s$ 的关闭在 $B$ 之前的位置生效；
- $\mathcal G_{\rm E}$ 健康；
- A7 成立，

则 $B\in G^u(\tau_0+T_{\rm incl})$。

**L3（治理替换故障区域）。**
设 $\mathcal Z$ 在 $\tau_0$ 时指示 $\mathsf M$ 发出采用 policy $P$ 的 Force。若在 $I=[\tau_0,\tau_0+T_{\rm force}]$ 上：

- $\mathcal G_{\rm E}$ 健康；
- 按 $P$ 选出的新 session 都是良态的，且所需的跨区域连接连通；
- A7 成立，

则在 $\tau_0+T_{\rm force}$ 之前 Force 已经生效，并且 $G$ 至少增长了一块。
注意：$\Pi$ 不自动检测区域故障，是否以及何时发出 Force 由治理（即 $\mathcal Z$）决定。

**L4（停机后经由 L1 推进）。**
设 $\mathcal Z$ 在 $\tau_0$ 时指示 $\mathsf M$ 发出 STOP。若在 $I=[\tau_0,\tau_0+T_{\rm stop}]$ 上同时满足：

- $\mathcal G_{\rm E}$ 的 $\mathsf{up}_{\rm read}=1$（$\mathsf{up}_{\rm ord}$ 可以为 $0$）；
- $\mathcal F_{\rm cert}$ 健康；
- 受保护前缀所需的数据在 $\mathcal G_{\rm DA}$ 中可取回；
- A7 成立，

则：

- (a) 在 $\tau_0+T_{\rm stop}$ 之前，每个观察者的 $G$ 都已进入 DEPOSIT_ONLY 模式；
- (b) 此后，若某个 L1 位置 $p$ 在 $\tau_p\ge\tau_0+T_{\rm stop}$ 时 finalized，并且原生规则使 $p$ 产生 deposit-only 块，那么只要 A7 在 $[\tau_p,\tau_p+T_{\rm dep}]$ 上成立，这些块就在 $\tau_p+T_{\rm dep}$ 之前进入 $G^u$。

这一目标不要求 Espresso 产生任何新块。

**L5（恢复）。**
设 $\mathcal Z$ 在 $\tau_0$ 时指示 $\mathsf M$ 发出采用 policy $P$ 的 RESUME。若在 $I=[\tau_0,\tau_0+T_{\rm resume}]$ 上：

- $\mathcal G_{\rm E}$ 与 $\mathcal F_{\rm cert}$ 健康；
- 按 $P$ 选出的 session 是良态的，且跨区域连接连通；
- A7 成立，

则在 $\tau_0+T_{\rm resume}$ 之前，系统回到 NORMAL 模式，并且 $G$ 至少增加一个出处为 $\mathsf{ORD}$ 的块。

**L6（结算推进）。**
设 $B\in G^u(\tau_0)$。若在 $[\tau_0,\tau_0+T_{\rm batch}+T_{\rm set}]$ 上 $\mathcal F_{\rm cert}$ 与 $\mathcal G_{\rm DA}$ 健康，且 A7 成立，则：

- 到 $\tau_0+T_{\rm batch}$ 时，对每个活跃的诚实派生节点 $v$，有 $\mathrm{native}(B)\in D^v(\tau_0+T_{\rm batch})$；
- 到 $\tau_0+T_{\rm batch}+T_{\rm set}$ 时，覆盖 $h(B)$ 的正确声明已被接受。

**L7（证据处理）。**
设 $s$ 的拜占庭成员犯下可 slash 的违规，且 $s$ 的某个正确成员在 $\tau_0$ 时看到了证据。则该违规在 $\tau_0+T_{\rm slash}$ 之前在 L1 上处理完毕（slash，或返回“已处理”）。
协议须保证 $T_{\rm slash}\le T_{\rm ev}$，以便与 S7 衔接。

**L8（释放推进）。**
设一个从未被腐化的 position 在 $\tau_0$ 时请求退出。只要覆盖其最后一个职责窗口的关闭发生，它就在有限时间内被释放。
确切条件待定，见 D5。

### 3.4 依赖表

| 目标 | safety 类依赖 | 时间或可用性类依赖 |
| --- | --- | --- |
| S0–S2 | A1, A4, L1-S, E-S, DA-S1, CERT-S | $T_{\rm L1}$, $\varepsilon_E$：仅经由控制公告纪律（D1） |
| S3 | A1, A2, L1-S, E-S | — |
| S4 | S3 的全部依赖, DA-S1, CERT-S | 方案 B：无。方案 A：$T_{\rm L1}$, $T_{\rm DA}$, $T_{\rm cert}$, $T_{\rm ret}$, A7（D3） |
| S5 | A1，$s$ 良态，以及 S0–S2 的依赖 | $\Delta_i$，作为 safety 前提 |
| S6 | A1, A3, L1-S，持久化存储 | — |
| S7 | A1, L1-S, CERT-S | — |
| L1 | S 类全部依赖，被选中的 session 良态 | E-L, L1-L, $\Delta_i$, $\Delta_{\rm cross}$, $T_{\rm ready}$, A7 |
| L2 | S5 的依赖 | E-L, $\Delta_i$, A7 |
| L3 | 新 session 良态 | A4 发出 Force, E-L, L1-L, $\varepsilon_E$, $\Delta_{\rm cross}$, $T_{\rm ready}$, A7 |
| L4 | — | A4 发出 STOP, L1-L, E 可读, CERT-L, DA-L, $T_{\rm ret}$, A7 |
| L5 | 新 session 良态 | A4 发出 RESUME, L1-L, E-L, CERT-L, $\Delta_{\rm cross}$, $T_{\rm ready}$, A7 |
| L6 | — | CERT-L, DA-L, L1-L, SET-L1, A7 |
| L7 | — | $\Delta_i$, L1-L, A7 |
| L8 | — | L1-L, CERT-L，关闭事件发生（D5） |

从这张表可以读出三条结构性结论：

1. **全局一致（S0–S2）不依赖任何区域的诚实，也不依赖网络同步。** 它只依赖公开账本的 safety、certifier 的可信、$\mathsf M$ 的控制纪律，以及（现方案下）控制可见性的时间界。
2. **有效性（S3）与结算对应（S4）只依赖拜占庭界 A2，不依赖成员在线。** 正确签名者只为原生有效的块签名；离线的诚实成员不签任何东西，所以不影响这两条。
3. **快速确认（S5）是唯一依赖区域同步 $\Delta_i$ 的 safety 目标。** 因此它只对良态 session 声称，并且可被治理关闭撤销。

### 3.5 各故障场景下目标的含义

| 场景 | 仍然成立的 safety 目标 | 活性如何恢复 |
| --- | --- | --- |
| 某 session 有少数拜占庭成员（良态） | 全部 | NORMAL 继续，由区域内恢复处理 |
| 整区离线或区域网络中断（A2 成立，不良态） | S0–S4, S6, S7；该 session 的 S5 不保证 | 治理 Force（L3） |
| 跨区域网络分区 | 全部 | 恢复连通后继续交接 |
| Espresso 停机（$\mathsf{up}_{\rm ord}=0$） | 全部 | STOP 后进入 L1 路径（L4），之后 RESUME（L5） |
| DA 不可用 | 方案 B 下全部成立；方案 A 下 S4 可能失效 | 等待 DA 恢复；结算推进（L6）暂停 |
| $\mathcal F_{\rm cert}$ 不可用 | 全部 | 结算与模式切换暂停，直到其恢复 |
| 治理沉默 | 全部 | 无：L3–L5 需要治理行动 |
| A2 被违反、L1 失效、Espresso safety 失效、$\mathcal F_{\rm cert}$ 或 $\mathsf M$ 被攻破 | 不作承诺（模型之外） | — |

### 3.6 非目标

- 普通交易的包含保证与抗审查。L1 deposit 与 forced inclusion 按原生规则继承。
- 区域之间、leader 之间的公平，以及排序公平或 MEV 相关性质。
- 自动检测 Espresso 停机或区域故障。这些由治理判断。
- Accountable safety，即安全被破坏时必能指认一定权重的违规者。在同步、$F_s<W_s/2$ 的区域协议中，两张证书不必相交。
- 超出保留窗口的数据丢失之后的无损恢复。
- 消息机密性与交易隐私。
- UC realization（见 §0.1）。

### 3.7 待讨论的设计选择

**D1｜控制可见性：S1 是否应依赖时间界？**
现方案用公告提前量 $\theta_a>\tau_{\rm submit}+T_{\rm L1}+\varepsilon_E$，保证观察者在处理 $J_q$ 时已经看到相关控制。代价是 S1 依赖 $T_{\rm L1}$ 与 $\varepsilon_E$。
另一种做法：让 $\mathsf{Replay}$ 在处理 $J_q$ 之前，先等 L1 的 finalized 视图覆盖到时间戳不小于 $\theta_q$；同时合约拒绝 $\theta_a\le$ 纳入块时间戳的控制。这样 S1 只依赖 safety 类假设，但 $G$ 的延迟会增加，量级约为 L1 finality 的延迟。

**D2｜$\mathsf M$ 的信任范围。**
是否把“即使 $\mathsf M$ 被攻破，S0–S3 仍然成立，只损失活性”列为目标？若是，所有控制纪律都必须由合约或 $\mathsf{Replay}$ 强制执行，而不是依赖 $\mathsf M$ 的诚实。这与 D1 相关。

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

**D6｜$\mathcal G_{\rm E}$ 的健康建模。**
排序与读取两个标志是否足够？是否需要把 $T_E$ 再拆成收录时限与可见时限？

---

## 参考文献

- [CDPW07] R. Canetti, Y. Dodis, R. Pass, S. Walfish. *Universally Composable Security with Global Setup.* TCC 2007. 另见 [UC, ePrint 2000/067](https://eprint.iacr.org/2000/067) 与 [GUC, ePrint 2006/432](https://eprint.iacr.org/2006/432)。
- [BMTZ17] C. Badertscher, U. Maurer, D. Tschudi, V. Zikas. *Bitcoin as a Transaction Ledger: A Composable Treatment.* CRYPTO 2017.
- [GKL15] J. Garay, A. Kiayias, N. Leonardos. *The Bitcoin Backbone Protocol: Analysis and Applications.* EUROCRYPT 2015.
- [PS17] R. Pass, E. Shi. *The Sleepy Model of Consensus.* ASIACRYPT 2017.
- [AMNRY20] I. Abraham, D. Malkhi, K. Nayak, L. Ren, M. Yin. *Sync HotStuff: Simple and Practical Synchronous State Machine Replication.* IEEE S&P 2020.
- [PST17] R. Pass, E. Shi, F. Tramèr. *Formal Abstractions for Attested Execution Secure Processors.* EUROCRYPT 2017.
- [CKPS01] C. Cachin, K. Kursawe, F. Petzold, V. Shoup. *Secure and Efficient Asynchronous Broadcast Protocols.* CRYPTO 2001（external validity）。
