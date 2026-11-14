# MegaETH Sequencer Rotation 技术报告 · Part 1

## 模块与接口（中文工作稿 v0.2）

日期：2026-10-26。
状态：草稿。
上一版：[v0.1](part1_module_interfaces_zh.md)。

本稿给出系统的模块划分、每个模块的接口与保证，以及模块之间如何耦合。
它不给协议框：每个模块的协议与证明在后续各 Part 中给出。那些 Part 先复述本稿的对应小节，再实现它。

前置文档：

- [Part 0 v0.7](part0_model_goals_zh_v0_7.md)：模型、假设与设计目标；
- [协议修订记录](protocol_revisions_zh.md)：对协议本身的修订 R1–R28；
- [以 D 为准的恢复](d_anchored_recovery_zh.md)：本版恢复方案的推导与审阅。

**相对 v0.1 的变化**（对应修订记录 R21–R28）

1. **GFR 不再生成或预测 deposit-only 块。** 删除 DepFill、R-DEP、R-OVR、R-ANCHOR（`AnchorHeight`）、R-FB（截止事件与 `Replaced`）。
2. **RESUME 改为“以 D 为准”（§4.2.9、§5.2、§5.5）。** `SubmitResume(c, P_new, h, Σ_M)` 由 $\mathsf M$ 给出目标高度；合约只检查授权与格式，然后 override。GFR 处理 RESUME 时，把 $G$ 替换为派生链 $D_L$，以 $\mathrm{last}(D_L)$ 为锚点开出新分支。这是 GFR 唯一读取 DA 的地方。
3. **删除 STOP。** 删除 `SubmitStop`、`Stopped`、`StopEffective`、R-STOP、限高 `Limit`；L1 Authority 的接口重新编号。
4. **过期的效果改变（§5.2）。** G 的下一个高度过期而仍没有块时，关闭当前分支，Tail 进入 HALT，直到 RESUME。
5. **新增共享规则 DerivOK（§3.2）**：派生一致投票的检查，取代 DepFill 的位置。
6. **新增 `MGR.Kill`（§4.3.8）**：Session Manager 在本地关闭已不可能进入 $G$ 与 $D$ 的 session。
7. **出处改为 $\{\mathsf{ORD}(\nu),\mathsf{DER}\}$（§2.4）。** batcher 不再提交 DEP 空批次；$\mathsf C$ 只认证普通块，且签名的 GFR 位置不早于 prev cursor 的写入位置（§4.7.1）。
8. **§6 的恢复流程按新规则重写；§8 的义务与对应表相应更新。**

---

## 0. 定位、来源与写法

### 0.1 来源与取舍顺序

1. **Part 0 v0.7：** 记号、假设（A-*）、设计目标（S0–S6、L-tx、L-fi、DG1–DG9、L2 的性质）。
2. **协议修订记录 R1–R28：** 对协议本身的改动。
3. **组合式抽象 v1/v2**（[v1](../formalization_v3/composability_abstraction_zh.md)、[v2](../formalization_v3/composability_abstraction_v2_zh.md)）：模块划分与接口骨架。
4. **Part II v4.9 与 Part III v6：** 字段与语义的细节。

几份材料冲突时，按以上顺序取舍。

### 0.2 与组合式抽象的对应

本稿沿用组合式抽象的模块划分，只按修订记录修改接口内容。章节对应如下：

| 本稿 | 组合式抽象 | 说明 |
| --- | --- | --- |
| §1 抽象的组织方式 | §1 | 模块表、架构图、典型流程 |
| §2 记号与共享类型 | §2 | 类型按修订更新 |
| §3 共享规则 | §3.5、§4.2、§5.1 中的确定性函数 | 汇总为共享规则，并新增 DerivOK |
| §4 各模块的接口 | §4 | Builder、Authority、Manager、Program、Publisher、Batcher 与 Batch Inbox、Certifier，以及原生组件 |
| §5 Global Finality Rule | §5 | v2 的模式状态机被替换 |
| §6 恢复中各模块的分工 | §6 | 停机模式扩展被恢复流程取代 |
| §7 Derivation Rule | §7 | 规则不改（方案 A） |
| §8 通信、职责与组合检查 | §8 | 跨模块义务、与 Part 0 目标的对应、新旧接口对照 |

组合式抽象的 §3（外部共享功能）已由 Part 0 §2 承担，本稿不再重复。

### 0.3 每个模块的写法

每个模块按同一个模板写：

1. **职责：** 一段话。
2. **状态：** 它拥有哪些状态，谁能修改。
3. **接口：** 每个接口一小节，先列**输入**，再列**输出**，最后一段写**功能**。功能段写清调用者、前置条件、幂等性，以及何时返回 `WAIT`、`REJECT` 或 $\bot$。
4. **保证：** 编号为 G-<简称>-k，先列 safety 类，再列 liveness 类。每条注明依赖 Part 0 的哪些假设。
5. **依赖：** 用到其他模块的哪几条保证。

### 0.4 通用调用语义

以下约定适用于本稿的每个接口。

**返回值**（同 Part 0 §0.3）：

- 值，或经验证的 $\bot$：在调用者已掌握的数据范围内可以判定时返回；$\bot$ 表示“已证明不存在”；
- `WAIT`：所需数据尚未取得，并指明缺什么；`WAIT` 不等于 $\bot$；
- `REJECT(reason)`：请求格式错误，或可判定为无效。

**三类接口的调用方式：**

- **L1 合约接口**（L1 Authority、Batch Inbox、Deposit Portal、状态结算）：
  - 经由 L1 交易调用；输出指已最终化的执行结果与事件；
  - `REJECT` 即交易回滚，合约状态不变；
  - 改变状态的调用带请求标识 $(\mathit{caller},\mathit{rid})$：相同标识、相同内容的重复调用返回原结果；相同标识、不同内容则 `REJECT`。
- **节点内接口**（Block Builder、Session Manager、Session Program、Publisher 之间）：
  - 本地调用；先持久化，后对外发送；
  - 重复调用返回原结果，不重复执行逻辑动作。
- **确定性规则**（Global Finality Rule、Derivation Rule 与 §3 的共享规则）：
  - 纯函数，同输入同输出；
  - 输入不足时返回 `WAIT`，并给出缺失的对象与停止位置；已完成的输出不撤销。

**网络消息。** 使用 Part 0 §1.5 的认证信道。区域内的签名一律绑定 $(\mathit{domain},\mathit{chain},\mathit{version},\mathit{type},\mathrm{cid})$。

**完整对象。** 输入要么包含完整字节，要么包含接收方能在处理预算内解析的稳定标识。状态根、URL 或“稍后再取”的承诺都不能代替完整对象。

**记号提醒。** $\mathsf C$（无衬线）指 Certifier；$C$（斜体）指 session context（§2.3）；$\mathcal C_e$ 指 epoch $e$ 的质押快照。

---

## 1. 抽象的组织方式

### 1.1 四类对象

| 类别 | 对象 | 回答的问题 |
| --- | --- | --- |
| 外部组件（Part 0 §2） | Ethereum L1、Espresso、DA、通信与时钟 | 能读到什么、能验证什么、哪些服务在什么条件下按时完成 |
| 协议模块 | Block Builder、L1 Authority、Session Manager、Session Program、Publisher、Batcher、Batch Inbox、Certifier | 谁保管状态、接受请求、发送消息、履行义务 |
| 确定性规则 | Global Finality Rule、Derivation Rule；共享规则 Exec、DerivOK、CheckFINAL、Plan/Auth/Leader | 给定相同的已认证输入，任何人都得到什么相同结果 |
| 原生组件 | Deposit Portal、状态结算（Kailua） | 沿用现有合约，本稿只列接口 |

### 1.2 模块表

| 模块 | 简称 | 运行位置 | 归属 | 拥有的状态 | 实现所在（建议） |
| --- | --- | --- | --- | --- | --- |
| Block Builder | BLD | 每个成员节点 | DRS | 本地待处理交易 | Part 4 |
| L1 Authority | AUTH | L1 合约 | DRS | 质押、快照、禁任、REC、控制日志 | Part 2 |
| Session Manager | MGR | 每个成员节点 | DRS | 重算状态、派生视图、生产分支、程序表、关闭标记、已准备的状态、待发送任务 | Part 4 |
| Session Program | PROG | 每个成员节点，每个 session 一个实例 | DRS | 视图、签名记录、锁、投票、QC、计时器、执行状态 | Part 3 |
| Publisher | PUB | 任何持有完整结果的参与方 | DRS | 待发布队列 | Part 4 |
| Global Finality Rule | GFR | 确定性规则；观察者、MGR、Certifier、Batcher 都运行它 | DRS | 无：输出由两本账本的日志决定，RESUME 处另由派生链决定 | Part 5 |
| Batcher | BAT | 角色（注册的 batcher） | L2 | channel 构造状态 | Part 6 |
| Batch Inbox | BIN | L1 合约 | L2 | cursor、batcher 注册表、Certifier 公钥、admin | Part 6 |
| Certifier | CERT | 可信方 | L2 | 自己的来源视图 | Part 6 |
| Derivation Rule | DER | 确定性规则 | L2 | 无 | Part 6 |
| Deposit Portal | DP | L1 合约（原生） | 原生 | deposit 记录 | 沿用 |
| 状态结算 | SETTLE | L1 合约（原生，Kailua） | L2 | 状态 claim | 沿用 |

**DRS 与 L2 只是组合名，不是模块：**

- **DRS** = Block Builder + L1 Authority + Session Manager + Session Program + Publisher + Global Finality Rule，并以 Deposit Portal 为输入；
- **L2** = DRS + Batcher + Batch Inbox + Certifier + Derivation Rule + 状态结算。

### 1.3 架构图

```mermaid
flowchart TB
    USR[用户] -->|SubmitTx，路由到负责区域| BLD[Block Builder]
    USR -->|Deposit：强制包含| DP[Deposit Portal（原生）]
    STK[质押者与证据提交者] -->|Join、RequestExit、Release| AUTH[L1 Authority]
    GOV[治理方 M] -->|SubmitForce、SubmitResume| AUTH
    MGR[Session Manager] -->|Create、Enter、Provide、Close| PROG[Session Program]
    BLD -->|候选块| PROG
    PROG -->|Conf、Result、Complete、Quit、Offense| MGR
    PROG <-->|同 session 共识消息| PEER[同 session 的其他成员]
    MGR <-->|区域结果、handoff、ReadyBundle| XMGR[其他区域的 Manager]
    MGR -->|SubmitQuit、SubmitOffense| AUTH
    MGR -->|Offer| PUB[Publisher]
    PUB -->|publication| ESP[(Espresso)]
    AUTH --> L1[(Ethereum L1)]
    DP --> L1
    L1 --> GFR[Global Finality Rule]
    ESP --> GFR
    GFR -->|G、Tail、生命周期事件| MGR
    MGR -->|负责区域| BLD
    GFR -->|G| BAT[Batcher]
    GFR -->|G| CERT[Certifier]
    BAT -->|Disperse| DA[(DA)]
    BAT -->|CertifyBatch| CERT
    CERT -->|签名 χ| BAT
    BAT -->|SubmitBatch| BIN[Batch Inbox]
    AUTH -->|Override（仅 RESUME）| BIN
    BIN --> L1
    L1 --> DER[Derivation Rule]
    DA --> DER
    DER -->|L1 派生链 D| SETTLE[状态结算（原生）]
    DER -.->|D_L（仅 RESUME）| GFR
    DER -.->|D（Kill 与锚点状态）| MGR
```

图中所有跨信任域的交互，都经由 L1、Espresso、DA 或 Certifier 的签名；节点之间的直接消息只用来加快进度，不决定任何结果。

### 1.4 四条典型流程

**流程 1：正常出块与常规轮换。**

1. 用户把交易发给负责区域的成员：`BLD.SubmitTx`。
2. leader 的 Program 取候选：`BLD.Build`；成员验证：`BLD.ValidateCandidate`；经区域共识后提交，输出快速确认 `Conf` 与区域结果 `Result`。
3. Manager 把结果交给 Publisher：`PUB.Offer`，结果进入 Espresso。
4. 各方运行 `GFR.AdvanceGlobal`，把块追加到 $G$。
5. 区域 A 的末块得到区域 FINAL 后，区域 B 的 Manager 经 `MGR.ObserveRegional` 收到它，算出常规后继的 handoff，并依次执行 `MGR.InstallHandoff` 与 `MGR.PrepareAndEnter`；B 的 Program 不必等末块进入 $G$ 就开始出块。
6. L2 侧：`BAT.BuildPayload` → `DA.Disperse` → `CERT.CertifyBatch` → `BIN.SubmitBatch`，随后由 `DER.Derive` 派生出 L1 上的链。

**流程 2：治理 Force。**

1. 治理方调用 `AUTH.SubmitForce`；合约检查公告提前量等条件。
2. GFR 在 $\theta\ge\theta_a$ 的第一个 Espresso 块处生效：关闭旧分支，以 $\mathrm{last}(G)$ 为锚点开出新分支的 session。
3. Manager 在 `MGR.ApplyLifecycle` 中关闭旧 Program（`PROG.Close`），并为新 session 安装 handoff。

**流程 3：区域停摆（Espresso 正常）。**

1. 负责区域不再出块，$G$ 停在 $\mathrm{last}(G)$。
2. 治理方在负责高度过期（$W_{\rm exp}$）之前调用 `AUTH.SubmitForce`：新 session 以 $\mathrm{last}(G)$ 为锚点，追赶缺口，不需要 deposit-only。
3. 若超过 $W_{\rm exp}$：GFR 关闭当前分支，Tail 进入 HALT；之后按流程 4 由 RESUME 恢复。

**流程 4：紧急恢复（Espresso 长停机或批次层故障）。**

1. $G$ 不再增长（Espresso 停机），或 Batch Inbox 跟不上 $G$（批次层故障）。
2. 相应高度的 sequencing window 结束后，L1 派生以 deposit-only 补位，$D$ 越过或偏离 $G$。各 Manager 关闭已不可能进入 $D$ 的 session（`MGR.Kill`）。
3. 故障恢复后，治理方调用 `AUTH.SubmitResume(c, P_new, h)`：同一交易内 `BIN.Override` 把 cursor 移到 h，产生 `CursorSet`；`DER.Derive` 把空缺填成 deposit-only，直到 h。
4. GFR 处理 RESUME：$G := D_L$，关闭旧分支，开出新分支；新 session 以 $\mathrm{last}(D_L)$ 为锚点。

§6 按模块接口逐步写出流程 3、4。

---

## 2. 记号与共享类型

本节定义所有跨模块传递的对象。§4–§7 的接口只引用这里的类型，每个对象只有一个定义处。

### 2.1 记号（补充 Part 0 §0.2）

| 记号 | 含义 |
| --- | --- |
| $s=(\beta,r)$ | session：分支版本 $\beta$，轮次 $r$ |
| $(\beta,r,g,t)$；$j$ | 区域视图：恢复代 $g$，leader term $t$；term 内的 fresh 位置 $j$ |
| $k_s(B)$ | 块在 session $s$ 内的物理位置，DATA 与 DRAIN 都计数 |
| $h(B)$；$t(h)$ | L2 高度；高度 $h$ 的 L2 时间戳（Part 0 N1） |
| $\lambda(B)$；$\xi(B)$ | 块的授权 L1 引用的高度；块引用的 Espresso 块 $(q,\mathsf H(J_q))$ |
| $\mathsf H$ | 抗碰撞哈希 |
| $\mathrm{last}(G)$ | 全局链的末块 |
| $\mathit{st}(B)$ | 精确执行到块 $B$ 之后的状态 |
| $G'$ | Batch Inbox 已接纳的高度前缀，即 cursor 以下的高度 |
| $L$；$h_{\rm res}$ | RESUME 记录所在的 L1 块；RESUME 的目标高度 |
| $D_L$ | 按 L1 顺序处理到 $L$ 中 RESUME 的 `CursorSet` 为止的派生链（§7.1） |
| $\mathrm{chain}(s)$ | session $s$ 的锚点及其全部祖先，加上 $s$ 已提交的块（§4.3.8） |

### 2.2 全局常数

以下常数在 genesis 固定。“使用方”一列同时是一张一致性清单：同一常数在所有使用方处必须取同一个值（§8.2 的义务 O-2）。

| 常数 | 含义 | 使用方 |
| --- | --- | --- |
| $\mathit{chain}$、ruleVersion | 链标识与规则版本 | 全部 |
| $\mathcal R$、$\mathcal L_R$ | 区域集合与 genesis 的循环顺序 | AUTH、GFR |
| $t_{\rm gen},h_{\rm gen},b_t$ | 高度与 L2 时间戳的对应（N1） | BLD、PROG、AUTH、GFR、DER |
| $D_{\rm drift}$、`seq_window_size`（时长 $W_{\rm seq}$） | 原生 sequencer drift 与 sequencing window | BLD、PROG（DerivOK）、MGR（Kill）、DER |
| $\mathrm{TB},K$ | session 的 DATA 配额；每个 term 的位置上限 | AUTH（plan 与释放公式）、PROG、GFR |
| $\Delta_i,\varphi_i$ | 区域 $i$ 的消息时限与故障比例 | AUTH（plan）、PROG |
| $M_a$ | 公告提前量；L1 事件在合并顺序中的键偏移 | AUTH、GFR |
| $W_{\rm exp}$ | 普通块的过期时长（R-EXP），约 10 h | GFR、PROG、AUTH（释放公式） |
| $\delta_{\rm ref}$ | 授权 L1 引用的新鲜度（R-FRESH） | GFR、PROG、AUTH（释放公式） |
| $\Delta_{\rm ts}$ | 时间戳偏差上界（A-TS） | AUTH（释放公式） |
| $T_{\rm ev}$ | 证据窗口 | AUTH |
| 质押参数 | $B_{\min}$、入队与出队预算、$M,N,a_0$、$\rho_{\rm slash}$ | AUTH |
| $\mathit{pk}_{\mathsf M}$、$\mathit{pk}_{\mathsf C}$ | 治理方与 Certifier 的公钥 | AUTH、BIN |
| $A_0$、$q_{\rm init}$、初始 policy 与控制前缀 | genesis 检查点 | AUTH、GFR |

### 2.3 Policy、plan 与 context

- **Policy** $\mathcal P_\beta=(S_\beta,\mathcal L_\beta,\lambda_\beta)$：每个区域的 ON/OFF 标记、循环顺序、L1 floor。它在分支版本 $\beta$ 内不变，由 genesis、Force 或 RESUME 安装；floor 在分支之间不减。
- **Plan** $\Pi=\mathsf{Plan}(\mathcal P_\beta,\beta,r,E,i_{\rm src})$（§4.2）。字段为：
  - 选择输入：$\mathit{chain},\beta,r,h(E),\mathsf H(E),\lambda(E),i_{\rm src}$；
  - 所用快照：epoch $e$ 与快照根 $\mathit{root}_e$；所用 policy；初始禁任集 $\mathsf{Ban}_0$；
  - 结果：区域 $i$ 或 $\bot$；委员会 $\{(o,\mathit{pk}_o,w_o)\}$；$W$、$F=\varphi_iW$、$\mathrm{TB}$、$K$、$\Delta_i$；
  - 来源高度 $\lambda(\Pi)$ 与标识 $\mathrm{id}(\Pi)$。
- **恢复代的授权** $\mathsf{AUTH}_{s,0}=\Pi_s$；当 $g>0$ 时，$\mathsf{AUTH}_{s,g}=(\Pi_s,\mathsf{REC}(\mathrm{cid}_s,g),\mathsf{Ban}_{s,g})$。
- **Context** $C_s=(\Pi_s,A_s)$，$\mathrm{cid}_s=\mathsf H(\mathrm{id}(\Pi_s),\mathsf H(A_s))$，其中 $A_s$ 是 session 的初始锚点。所有区域签名都绑定 $\mathrm{cid}_s$。
- **授权 L1 引用** $\lambda(E)$：对区域块，是其字段 $\rho$ 所指 L1 块的高度；对 DER 块，暂取其原生 L1 origin（D12 待定）。

### 2.4 块与出处

- **区域块** $B=(\beta,r,k,h,\mathit{type},x,\mathsf H(B_{\rm parent}),\rho,\xi)$，$\mathit{type}\in\{\mathsf{DATA},\mathsf{DRAIN}\}$，与 Part II v4.9 相同：
  - DATA 通过 $x$ 绑定一个完整的原生 L2 块：有序普通交易、L1 origin、deposit、系统交易；其高度为父块加一。
  - DRAIN 不带新的原生块，高度不变，只推进 $k$。
  - $\rho$ 是授权 L1 引用，$\xi$ 是 Espresso 引用。
- **DER 块**：RESUME 时从派生链 $D_L$ 采纳、且不在原全局链中的块（§5.5）。它是 deposit-only 块，或者是已接纳批次在实际父块上重新执行得到的块。外层信封为 $(h,\mathsf{DER},\mathit{native},\mathsf H(\mathit{parent}))$。
- **原生块与状态**：$\mathrm{native}(B)$ 是块 $B$ 对应的原生 L2 块，$\mathit{st}(B)$ 是执行它之后的状态。信封哈希与原生块哈希是不同的类型，不能混用。
- **出处** $\sigma(B)\in\{\mathsf{ORD}(\nu),\ \mathsf{DER}\}$：普通块记下它被接纳时所用的 descriptor $\nu$（§2.7），即 Part 0 的 $\mathsf{ORD}(s)$，其中 $s=\mathrm{sid}(\nu)$；DER 块只记类型。RESUME 时，$D_L$ 中与原全局链同一高度、同一原生块的块保留原来的出处与信封。

### 2.5 区域签名消息与证书

每条区域签名消息都签名覆盖 $(\mathit{domain},\mathit{chain},\mathit{version},\mathit{type},\mathrm{cid}_s,\mathit{slot},\mathit{payload})$。本稿只列出跨模块用得到的部分；完整定义在 Session Program 的 Part 中给出。

| 类型 | 槽位 | 跨模块用途 |
| --- | --- | --- |
| PROPOSE | $(g,t,j)$ | 双签构成 offense 与 quit 证据 |
| DATA-VOTE、DRAIN-VOTE | $(g,t,j)$，两者共用一个槽位 | 双投构成 offense |
| STATUS | $(g,0)$ | 仅区域内使用 |
| NEWVIEW | $(g,0)$ | 恢复 leader 双开口构成 offense 与 quit 证据 |
| ANCHOR-VOTE | $(g,0)$ | 双投构成 offense |
| BLAME | $(g,t)$ | 权重 $>F_s$ 的 BLAME 证书构成 quit 证据 |
| FINAL | $(s,h(B))$，绑定 $\mathsf H(B)$，不含 $g,t$ | 区域最终证书，被 CheckFINAL 使用；同高度双签构成 offense |

**证书**：来自 $V_s$ 中不同成员、总权重严格大于 $F_s$ 的一组匹配签名。

### 2.6 区域结果与证据

- **区域结果** $\mathit{Res}=(C,B,D,\Sigma,\pi)$：
  - $B$ 是目标 DATA 块；
  - $D$ 是承载 FINAL 证书的同 session DATA 块，可以就是 $B$；
  - $\Sigma$ 是 $D$ 上的 FINAL 签名集合；$\pi$ 是从 $D$ 回到 $B$ 的完整、哈希相连的块体。

  $(D,\Sigma,\pi)$ 称为 $B$ 的 kernel。
- **快速确认** $\mathsf{Conf}_v(s,B)$：成员 $v$ 在区域内提交了块 $B$。
- **Quit 证据** $Q=(\mathit{kind},\mathit{coords},\mathit{msgs},\mathit{admission})$，$\mathit{kind}$ 为以下三者之一：
  - 某个已准入 term 上权重 $>F_s$ 的 BLAME 证书；
  - 该 term leader 在同一 $(g,t,j)$ 上的两份有效 proposal；
  - $g>0$ 时，恢复 leader 的两份不同 NEWVIEW 开口。
- **Offense 证据** $\mathit{ev}=(\mathit{kind},m_1,m_2,\mathit{admission})$，$\mathit{kind}\in\{\mathsf{PROPOSE},\mathsf{FRESH\_VOTE},\mathsf{NEWVIEW},\mathsf{ANCHOR\_VOTE},\mathsf{FINAL}\}$：同一把密钥在同一槽位签了两份冲突的消息。
- **恢复记录** $\mathsf{REC}(\mathrm{cid},g)$：L1 Authority 为 context $\mathrm{cid}$ 的第 $g$ 代记下的 L1 高度 $\ell_{\rm rec}$。每个 $(\mathrm{cid},g)$ 至多一条。

### 2.7 交接与生命周期

- **Descriptor**
  $$\nu=(\beta,r,i,A,H,q^{\rm open},\mathit{cause},\Pi),\qquad H=h(A)+\mathrm{TB},$$
  $$\mathit{cause}\in\{\mathsf{GENESIS},\ \mathsf{REGULAR},\ \mathsf{FORCE}(c,q),\ \mathsf{RESUME}(c,q)\}.$$
  其中 $c$ 是控制记录的序号，$q$ 是开出该 session 的 Espresso 块高度。
- **未定边界** $\omega=(\mathit{cause},\beta,r,i_{\rm src},A,q^{\rm open})$：目标 session 的标识已定，但区域尚未确定，或在等开出它的 Espresso 块。$A$ 在边界产生时就已固定：FORCE 取生效时的 $\mathrm{last}(G)$，RESUME 取 $\mathrm{last}(D_L)$；HALT 中 $A$ 为 $\bot$。
- **Tail**：GFR 关于“下一块由谁负责”的输出，取以下四种之一：
  - $\mathsf{NEXT}(\nu)$：session $\mathrm{sid}(\nu)$ 负责下一个高度；
  - $\mathsf{PENDING}(\omega)$：plan 已完整算出，但没有合格区域；只有新的 Force 或 RESUME 能替换它；
  - $\mathsf{OPEN}(\omega)$：边界与锚点已定，在等 plan 的来源，或等下一个 Espresso 块来开出 session；
  - $\mathsf{HALT}(\omega)$：$G$ 的下一个高度已过期（R-EXP）而仍没有块，当前分支已关闭；只有 RESUME 能替换它。
- **Handoff** $=(\nu,\mathcal P_\beta,E,i_{\rm src},\lambda^{\rm open},Q^0,\mathit{evidence})$，其中 $\lambda^{\rm open}=\max\{\lambda(A),\lambda(\Pi)\}$，$Q^0$ 是锚点原有的承诺证据。按 cause 不同，evidence 为：
  - GENESIS：genesis 检查点；
  - REGULAR：前驱的 handoff，加上前驱末块的完整区域结果；
  - FORCE：GFR 固定该边界的重算位置（Espresso 高度与 L1 高度）；
  - RESUME：GFR 处理 RESUME 的重算位置，以及 RESUME 所在的 L1 块 $L$。任何人都可以按派生规则重算 $D_L$，核对锚点 $A=\mathrm{last}(D_L)$。
- **ReadyBundle**：handoff；锚点的完整字节与证据；$\mathit{st}(A)$ 及其原生验证依据；所需的来源读取结果。它不携带任何锁、投票或计时器。
- **生命周期事件**（GFR 的输出，§5）：

| 事件 | 含义 |
| --- | --- |
| $\mathsf{Appended}(B,\sigma(B))$ | 块 $B$ 被追加到 $G$ |
| $\mathsf{SessionAssigned}(\nu)$ | Tail 变为 $\mathsf{NEXT}(\nu)$，且 $\nu$ 是新的 descriptor |
| $\mathsf{SessionClosed}(s,\mathit{reason})$ | session $s$ 不能再向 $G$ 追加块；$\mathit{reason}\in\{\mathsf{COMPLETE},\mathsf{FORCE},\mathsf{EXPIRED},\mathsf{RESUME}\}$ |
| $\mathsf{BranchClosed}(\beta,\mathit{reason})$ | 分支版本 $\beta$ 上的所有 session 都被关闭；$\mathit{reason}\in\{\mathsf{FORCE},\mathsf{EXPIRED},\mathsf{RESUME}\}$ |
| $\mathsf{Halted}(\omega)$ | Tail 进入 $\mathsf{HALT}$：$G$ 的下一个高度已过期 |
| $\mathsf{Resumed}(c,\beta',h_f,A)$ | RESUME 生效：$G$ 被替换为 $D_L$，原 $G$ 在高度 $h_f$ 及之后的块被撤销（没有撤销时 $h_f=\bot$）；新分支版本为 $\beta'$，锚点 $A=\mathrm{last}(D_L)$ |
| $\mathsf{Pending}(\omega)$ | Tail 进入 $\mathsf{PENDING}$ |

### 2.8 控制记录

治理方 $\mathsf M$ 签署、经 L1 Authority 检查后记入控制日志 $\Lambda$。每条记录连同它的 L1 位置 $p=(\ell,\mathit{txIndex},\mathit{logIndex})$ 与所在块的时间戳 $\mathit{ts}(b)$ 一起保存。

- **Force** $U=(c,\theta_a,\beta_{\rm target},\mathcal P_{\rm new})$：在 Espresso 时间 $\theta_a$ 关闭分支 $\beta_{\rm target}$，并以 $\mathcal P_{\rm new}=(S_{\rm new},\mathcal L_{\rm new})$ 开出新分支。新分支的 floor 由 GFR 取为 $\max\{\lambda_\beta,\ \lambda_U\}$，$\lambda_U$ 是记录所在的 L1 高度。
- **RESUME** $(c,\mathcal P_{\rm new},h_{\rm res})$：$h_{\rm res}$ 由 $\mathsf M$ 给出；同一交易内产生 Batch Inbox 的 `CursorSet`（§4.2.9）。

### 2.9 重算状态、事件与键

**重算状态** $\mathcal X=(G,\sigma,\mathsf{Tail},\beta,\mathcal P_\beta,\mathsf{Closed},\kappa,\mathit{pc})$：

- $G$ 与出处 $\sigma$；
- Tail；当前分支版本及其 policy；
- 已关闭的分支与 session $\mathsf{Closed}$；
- 已处理的控制位置 $\kappa$；
- 合并顺序中的处理位置 $\mathit{pc}$。

**事件与键。** GFR 按键从小到大处理三类事件：

| 事件 | 键 |
| --- | --- |
| Espresso 块 $J_q$ | $\theta_q$ |
| L1 事件 $e$：控制记录（Force、RESUME） | $\mathit{ts}(b_e)+M_a$；Force 的生效键为 $\theta_a$ |
| 时间事件 $\mathsf{Expire}(h)$ | $t(h)+W_{\rm exp}$ |

键相同时：L1 事件在前、时间事件其次、Espresso 块最后；同类之间按 L1 位置、高度或 Espresso 高度排序。

**R-WAIT（处理条件）。** 处理键为 $x$ 的事件之前，须满足：

- 观察者的 L1 日志已有时间戳 $\ge x-M_a$ 的块；
- 对 L1 事件与时间事件，Espresso 日志还须已有 $\theta\ge x$ 的块。

**重算记录** $\mathrm{Rec}^u(\tau)$：观察者 $u$ 到 $\tau$ 为止已处理的事件序列及其效果。$G^u(\tau)$ 由它确定。

### 2.10 L2 对象

- **Cursor**：沿用 mega-espresso batch inbox 的 cursor（已接纳的 L2 高度、L1 origin 编号、未完成 channel 的 frame 摘要等）。原来记录 Espresso 记录位置的字段，改为记录 Certifier 核对时所用的 GFR 重算位置；精确编码在 Part 6 中定义。
- **批次 payload**：一个 channel 的连续 frame，只覆盖 $G$ 的普通块：每个高度带完整的普通交易列表与元数据。deposit-only 高度不提交任何批次：它们由派生的窗口回退或 RESUME 的 override 填充。
- **DA 承诺** $d$ 与 DA 证书 $\pi_{\rm DA}$（Part 0 §2.5）。
- **Certifier 证书** $\chi$：$\mathsf C$ 对 $(\mathit{chain},\text{Batch Inbox 地址},\mathit{prev},d,\mathsf H(\pi_{\rm DA}),\mathit{next},[a,b],\mathit{gpos})$ 的签名。其中 $[a,b]$ 是本批次覆盖的高度区间，$\mathit{gpos}$ 是 Certifier 核对时所用的 GFR 位置。

---

## 3. 共享规则

共享规则是多个模块都要调用的确定性函数。所有使用方必须使用同一个规则版本（§8.2 的义务 O-1）。

### 3.1 Exec：原生执行

**输入**

- 前一状态 $\mathit{st}$；
- 一个原生块的输入 $x$：有序普通交易、高度、时间戳、L1 origin、应包含的 deposit、L1 attributes 与系统交易、生效配置。

**输出**

- $(\mathit{st}',N,\mathit{root})$：执行后的状态、原生块与状态根；
- `INVALID(reason)`：块级条件不成立，例如时间戳与 origin 不符合 N2、漏掉应包含的 deposit；
- `WAIT`：所需输入（例如 L1 header 或 receipt）尚未取得。

**功能。** 固定 rule version 下的 MegaETH 原生执行。块内交易 revert 不会使整块无效。Exec 不选择分支，也不赋予最终性。
**使用方：** BLD、PROG、MGR、GFR（RESUME 时经 DER 计算 $D_L$）、DER、状态结算。

### 3.2 DerivOK：派生一致性检查

**输入**

- 父块 $P$ 及其状态 $\mathit{st}(P)$；记 $e=o(P)$ 为 $P$ 的 L1 origin；
- 候选原生块的输入 $x$：高度 $h=h(P)+1$、时间戳 $t(h)$、origin $o(x)$、前缀（L1 attributes、deposit、系统交易）与有序普通交易；
- 调用者的已最终化 L1 视图 $\ell$。

**输出**

- `OK`：派生在父块 $P$ 上处理这个块的批次时，一定原样采用它；
- `FAIL(reason)`；
- `WAIT`：所需的 L1 header 或 receipt 尚未在 $\ell$ 中最终化。

**功能。** 以下五条全部成立才返回 `OK`：

1. **V1 origin：** $o(x)\in\{e,e+1\}$；$o(x)$ 已在 $\ell$ 中最终化；$\mathit{ts}(o(x))\le t(h)$。
2. **V2 drift：** 非空块满足 $t(h)\le\mathit{ts}(o(x))+D_{\rm drift}$。空块只在两种情形下可以超过：推进了 origin（$o(x)=e+1$）；或者保留 origin，且 L1 块 $e+1$ 已在 $\ell$ 中最终化、其时间戳晚于 $t(h)$。
3. **V3 前缀：** 前缀恰好等于派生在 $(P,o(x))$ 上构造的前缀：L1 attributes；若 $o(x)\ne e$，该 L1 块中 Deposit Portal 记录的全部 deposit；适用的系统交易与升级交易。顺序也一致。普通交易中没有 deposit 类型的交易。
4. **V4 执行：** $\mathsf{Exec}(\mathit{st}(P),x)$ 不是 `INVALID`。
5. **V5 新鲜度（R-FRESH）：** 块的授权 L1 引用满足 $\mathit{ts}(\lambda(x))\ge t(h)-\delta_{\rm ref}$，与 GFR 的 Eligible (f) 相同。$\lambda$ 是否与 origin 统一见 D12。

**稳健性。** 每一条都只依据 $\ell$ 中已经最终化的事实，所以 $\ell$ 以后变长，结论不变。V2 的第二种例外因此要求“已经看到 $e+1$”，不能依据“还没看到 $e+1$”。

**为什么需要它。** 批次中没有 deposit：派生从 origin 自己构造前缀，再执行批次中的普通交易。区域块的前缀与派生构造的不同，或任何一条派生检查不过（mega-espresso 的 Δ5、Δ6 会把它换成 deposit-only），D 中的块就与 G 中的块不同。$\mathsf C$ 只核对普通交易列表、时间戳与 origin，查不出这类差异。DerivOK 是 Part 0 的 L2-L1（目标 T）的前提。sequencing window 的时间条件不在这里：它由 R-EXP 与交付预算保证。

**核对项（Part 6）。** 逐条对照 mega-espresso 的 Δ5 转换条件、Holocene 的执行替换条件与上游的 drift 例外，记为 G-DER-6。
**使用方：** BLD（`Build`、`ValidateCandidate`）；PROG 在投票前经 BLD 调用。

### 3.3 CheckFINAL：公开的区域最终检查

**输入**

- $C=(\Pi,A)$：检查者自己从公开历史推出的 producing context，不取发布者声称的 context；
- $B$：待检查的 DATA 块；
- $D$：承载 FINAL 证书的同 session DATA 块，可以就是 $B$；
- $\Sigma$：$D$ 上的 FINAL 签名；$\pi$：从 $D$ 回到 $B$ 的完整块体。

**输出**

- `true`：两部分检查都通过；
- `false`：任一部分不通过；
- `incomplete`：缺必需的对象。

**功能。** 与 Part II v4.9 相同，分两部分：

1. **签名：** $\Sigma$ 中来自委员会不同成员的有效 FINAL 签名，在 $\mathrm{cid}(C)$ 下绑定 $D$ 的哈希与高度，总权重 $>F$。
2. **路径：** $\pi$ 的每个块体哈希正确、父链接正确、位置正确，终点是 $B$；$D$ 与 $B$ 属于同一 session；$h(B)\le h(A)+\mathrm{TB}$。

它只是密码学与结构检查，不重新做原生执行。区域有效性由 Session Program 的 G-PROG-2 保证。
**使用方：** GFR、MGR（常规交接与区域结果）、PUB（发布前的可选自查）。

### 3.4 Plan、Auth、Leader

这三个纯函数定义在 L1 Authority 的状态之上（§4.2 的接口 11–13），任何人都可以在一份已认证的 L1 前缀上计算。
**使用方：** GFR（选择后继 session）、MGR（核对 handoff）、PROG（leader 与恢复代的授权）、AUTH（相对验证 offense 与 quit 证据）。

### 3.5 时间与键函数

- **高度与时间戳：** $t(h)=t_{\rm gen}+(h-h_{\rm gen})\cdot b_t$。
- **cursor 超窗停滞：** 在 L1 高度 $\ell$ 上，`ℓ > cursor.epoch_num + seq_window_size + 1`。成立时，cursor 之后的第一个高度在派生中一定已由窗口回退填成 deposit-only，派生链已越过或偏离全局链。精确边界在 Part 6 对照原生 batch queue 核对。它供 `MGR.Kill`（§4.3.8）与 RESUME 的操作规程（§4.2.9）使用。
- **事件键：** 见 §2.9。

### 3.6 版本一致性

| 规则 | 使用方 | 若使用方版本不一致 |
| --- | --- | --- |
| Exec | BLD、PROG、MGR、GFR、DER、SETTLE | 区域验证与 L1 派生分歧，L2-S1 可能失效 |
| DerivOK | BLD、PROG（须与 DER 逐条对齐） | 区域投票接受的块被派生替换，目标 T 失效 |
| Derive | DER、GFR（RESUME）、MGR（Kill 与锚点状态）、SETTLE | 观察者在 RESUME 处的替换结果不同，S1 失效 |
| CheckFINAL | GFR、MGR、PUB | 观察者之间对同一结果判断不同，S1 失效 |
| Plan、Auth、Leader | GFR、MGR、PROG、AUTH | 委员会或 leader 判断不同 |
| GFR 本身 | 观察者、MGR、CERT、BAT | Certifier 认证的内容与观察者的 $G$ 不同 |

---

## 4. 各模块的接口

### 4.1 Block Builder（BLD）

**职责。**
每个成员节点的交易入口与候选块构造器。它只为本区域当前或下一个负责的 session 接受交易，并为 Session Program 构造满足原生规则的候选块。它不选择 parent，不选择授权，也不赋予候选任何共识许可。

**状态。**

- 本地待处理交易集合，按交易标识去重；
- 本区域的负责性视图，由 Session Manager 提供（`MGR.Responsibility`）：当前负责的 session 与区域，以及已知的下一个负责区域。

#### 4.1.1 `SubmitTx(tx)`

**输入**

- $tx$：用户签名的完整原生交易字节。

**输出**

- `Received(txid)`：交易已进入本地待处理集合；
- `REJECT(NOT_RESPONSIBLE, i_resp)`：本区域既不是当前负责区域，也不是下一个负责区域；$i_{\rm resp}$ 是当前负责的区域；
- `REJECT(reason)`：格式、签名或准入条件不满足。

**功能。**
调用者是客户端，或替客户端路由的 RPC 网关。只有本区域是当前负责区域或已知的下一个负责区域时，才接受交易（R19）；谁负责由 GFR 的 Tail 与常规后继的 plan 公开可算。
Builder **不向其他区域转发交易**。负责区域更换时，本地尚未被包含的交易不保证被新区域看到，由客户端重新提交。
按交易标识去重。接受交易不代表包含、排序优先级或最终性。

#### 4.1.2 `Build(s, parent, parentState, nctx, limits)`

**输入**

- $s$：请求候选的 session；调用者是本节点上该 session 的 Program，在其担任 leader 时调用。
- $\mathit{parent}$、$\mathit{parentState}$：精确的父块及其执行后状态。
- $\mathit{nctx}$：原生上下文，包括高度 $h$、时间戳 $t(h)$、L1 origin header、该 origin 对应的 deposit（来自 Deposit Portal）、L1 attributes 与系统交易、生效配置。
- $\mathit{limits}$：gas、字节数、交易数与 mini-block 格式的限制。

**输出**

- $x$：候选 payload，包括有序普通交易列表、原生上下文与 mini-block 划分；普通交易列表可以为空；
- `WAIT`：必要的来源或状态尚未就绪；
- `REJECT(reason)`：上下文或限制无效。

**功能。**
在给定 parent 上选择并排序待处理交易，组织成满足 DerivOK（§3.2）的候选。候选必须包含原生规则要求的 deposit，也就是强制包含交易。mini-block 的拼接顺序须与完整交易列表一致。
Builder 不选择 parent 或授权；高度上限等生产约束由 Program 在调用前检查。

#### 4.1.3 `ValidateCandidate(parent, parentState, x, nctx)`

**输入**

- $\mathit{parent}$、$\mathit{parentState}$、$x$、$\mathit{nctx}$：含义同上。

**输出**

- `VALID(N, st', root)`：格式、交易顺序、mini-block 对应与原生执行都通过；
- `INVALID(reason)` 或 `WAIT`。

**功能。**
检查两件事：mini-block 与完整交易列表一致；候选在 parent 上满足 DerivOK（§3.2），即 origin、drift、deposit 前缀、执行有效性与 R-FRESH 都与派生逐条一致，并对 L1 视图的扩展稳健。结果是确定性的。
调用者是 Program：每个成员在投票前都调用它。通过验证不等于区域最终或全局最终。

**保证。**

- **G-BLD-1（候选有效）：** Build 只输出 ValidateCandidate 接受的候选；ValidateCandidate 是确定性的。
- **G-BLD-2（派生一致）：** 任何通过 ValidateCandidate 的候选都满足 DerivOK；特别地，它包含原生规则要求的 deposit。
- **G-BLD-3（交易包含，活性）：** 诚实成员担任 leader 时，Build 在容量之内包含其待处理集合中所有持续有效的交易。

**依赖。** Exec（§3.1）与 DerivOK（§3.2）；Deposit Portal 与 L1-S（deposit 的来源）；Session Manager 的负责性视图（G-MGR-8）。

---

### 4.2 L1 Authority（AUTH）

**职责。**
L1 上的应用状态机。它管理质押与冻结快照，提供委员会与 leader 的确定性计算，处理 offense 与 quit 证据，记录治理方的控制，充当 Batch Inbox 的 admin，并按时间上界释放质押。
它不读 Espresso，也不判断某个 session 是否真的进入了 $G$；这些由 GFR 决定。

**状态。** 以下是已最终化 L1 高度 $\ell$ 上的逻辑状态；历史查询读取确定高度上的版本。

| 状态 | 内容 |
| --- | --- |
| $\mathsf{Pos}[o]$ | $(i_o,\mathit{pk}_o,\mathit{owner}_o,B_o,\mathit{phase}_o,p^{\rm join}_o,p^{\rm exit}_o,e_x(o))$：区域、公钥、提款 owner、剩余 escrow、生命周期、入队与出队请求位置、退役 epoch |
| `ReqResults` | 请求去重表 |
| $\mathsf{KeyPosition}[\mathit{pk}]$、`nextId` | 尚未解除责任的 key 到 position 的映射；不复用的 position 编号 |
| $Q^{\rm in}_i$、$Q^{\rm out}_i$ | 每个区域的入队与出队 FIFO |
| $\mathcal C_e$、$\ell_{\rm pub}(e)$ | epoch $e$ 的冻结快照及其发布高度 |
| $\mathsf{Ban}(\ell)$ | 已登记禁任的 key 集合，只增不减 |
| $\mathsf{REC}[(\mathrm{cid},g)]$ | 恢复记录 |
| `SeenOffenses` | 已处理的 $(o,\mathit{kind},\mathit{slot})$ |
| $\Lambda$、`lastSeq` | 控制日志与已用的最大序号 |

$\mathit{phase}_o\in\{\mathsf{Pending},\mathsf{Active},\mathsf{Exiting},\mathsf{Retired},\mathsf{Released},\mathsf{Cancelled}\}$。
快照的时间与权重沿用 Part III v6：$a_e=a_0+eM$，$c_e=a_e-N$，$w_{e,o}=B_o(c_e^-)$。
v1 中的 `Closures` 与 `Clear` 表已删除，由释放的时间上界取代（R13）。

**接口总览。** 接口 1–9 是改变状态的 L1 调用，10–13 是只读或纯函数。

| 组 | 接口 | 相对 Part III v6 / 组合式抽象的变化 |
| --- | --- | --- |
| 质押 | 1 Join、2 Cancel、3 RequestExit、4 CutSnapshot、5 Release | Release 改用时间上界（R13），其余不变 |
| 追责 | 6 SubmitOffense、7 SubmitQuit | 不变 |
| 治理控制 | 8 SubmitForce、9 SubmitResume | SubmitUpdate 改名为 SubmitForce 并加合约检查（R4）；ScheduleMode 改为 SubmitResume（R22），v0.1 的 SubmitStop 已删除（R23）；删除 ProveClosure |
| 只读与纯函数 | 10 ReadAuthority、11 Plan、12 Auth、13 Leader | Plan 的选区参考块可以是 DER 块 |

#### 4.2.1 `Join(i, pk, owner, b, pop, ownerAuth)`

**输入**

- $i\in\mathcal R$：加入的区域；$\mathit{pk}$：operator 的签名公钥；
- $\mathit{owner}$：提款 owner；$b\ge B_{\min}$：本次转入 escrow 的数量；
- $\mathit{pop}$：$\mathit{pk}$ 对 $(\mathit{chain},\text{Authority},\mathit{rid},i,\mathit{owner},b)$ 的 possession 证明；
- $\mathit{ownerAuth}$：owner 对该请求的授权。

**输出**

- `Joined(o, i, pk, owner, b)`；状态效果：创建 $\mathsf{Pos}[o]$，置 $B_o=b$、阶段为 Pending，登记 key 映射，加入 $Q^{\rm in}_i$；
- `REJECT(reason)`：区域、资金、授权、key 唯一性或准入限制不满足。

**功能。** 原子地接收资金并登记一个整 position 的入队请求。同一 key 不能同时对应多个未解除责任的 position；position 编号不复用。Join 不改变任何已冻结的快照或委员会。

#### 4.2.2 `Cancel(o, ownerAuth)`

**输入**

- $o$：尚未被任何冻结快照纳入的 Pending position；$\mathit{ownerAuth}$。

**输出**

- `Cancelled(o, refund)`；状态效果：移出入队队列，置 $B_o=0$ 与 Cancelled，解除 key 映射；
- `REJECT(reason)`。

**功能。** 让尚未承担任何快照责任的 position 取消加入并一次性退款。一旦进入过冻结快照，就只能走 RequestExit 与 Release。

#### 4.2.3 `RequestExit(o, ownerAuth)`

**输入**

- $o$：已被冻结快照纳入、尚无退役记录、也没有未完成退出请求的 position；$\mathit{ownerAuth}$。

**输出**

- `ExitRequested(o, p_exit)`；状态效果：登记退出请求位置，阶段改为 Exiting，加入 $Q^{\rm out}_{i_o}$；
- `REJECT(reason)`。

**功能。** 排队申请从未来的快照中移除整个 position。它不改变当前或历史的权重，不退款，也不解除已分配的 session 职责。

#### 4.2.4 `CutSnapshot(e)`

**输入**

- $e>0$：要冻结的 epoch；调用时 L1 高度至少为 $c_e$。
- 合约内部来源：$\mathcal C_{e-1}$，以及严格早于 $c_e$ 的 position、队列、余额与 offense 历史；调用者不能另传一份状态。

**输出**

- $\mathcal C_e$ 与 $\ell_{\rm pub}(e)$；若已存在，返回同一快照；
- 状态效果：更新队列；新纳入的 position 变为 Active；被移除的旧成员获得 $e_x(o)=e$ 并变为 Retired；
- `WAIT` 或 `REJECT(reason)`。

**功能。** 沿用 Part III v6：先移除 cutoff 之前已有 offense 或 escrow 低于最低额的成员，再按成熟条件与预算从入队、出队 FIFO 中各取最长的合格前缀，并以 cutoff 时的 escrow 冻结权重。迟发布时，消费者等待，快照内容不会因此改变。

#### 4.2.5 `Release(o, ownerAuth)`

**输入**

- $o$：已退役的 position；$\mathit{ownerAuth}$。
- 合约内部来源：$e_x(o)$ 与快照 $\mathcal C_{e_x}$ 的激活高度 $a_{e_x}$；当前 L1 块的时间戳 $\mathit{ts}$。

**输出**

- `Released(o, amount)`，$\mathit{amount}$ 为扣除已生效处罚后的剩余 escrow；状态效果：支付 owner，置 $B_o=0$ 与 Released，解除 key 映射；
- `REJECT(reason)`：尚未退役，或时间上界未到。

**功能。** 只有满足下式时才释放（R13）：

$$
\mathit{ts}\ \ge\ \mathit{ts}(a_{e_x})+\delta_{\rm ref}+2\mathrm{TB}\cdot b_t+W_{\rm exp}+2\Delta_{\rm ts}+T_{\rm ev}.
$$

合约不读 Espresso。这个公式之所以安全，依赖 GFR 对每个普通块执行的 R-EXP 与 R-FRESH（G-GFR-5），以及 A-TS。推导见 Part 0 v0.4 的 P2。

#### 4.2.6 `SubmitOffense(C, o, ev)`

**输入**

- $C=(\Pi,A)$：被声称的 context；
- $o$：在 $\Pi$ 的快照中对应被指控公钥的 position；
- $\mathit{ev}=(\mathit{kind},m_1,m_2,\mathit{admission})$：同一 key 的两条冲突签名消息，都绑定 $\mathrm{cid}(C)$，并附上该类型所需的准入材料。

**输出**

- `Slashed(o, kind, slot, amount, B_after)`，其中 $\mathit{amount}=\min\{\rho_{\rm slash}w_o,B_o\}$，$w_o$ 取自 $C$ 的快照；状态效果：扣减 escrow，永久禁任，记入 SeenOffenses；
- `AlreadyProcessed`：同一 $(o,\mathit{kind},\mathit{slot})$ 已处理，不再扣款；
- `REJECT(reason)`：证据无效或不完整，状态不变。

**功能。** 相对 $C$ 做验证：核对签名者、历史权重、admission 与冲突，但不判断 $C$ 对应的 session 是否真的开过。伪造的 context 不能转用诚实成员在另一个 $\mathrm{cid}$ 下的签名。timeout、BLAME、区域 OFF 与单纯转发都不构成 offense。

#### 4.2.7 `SubmitQuit(C, g, Q)`

**输入**

- $C$、$g$：要退出的 context 与恢复代；
- $Q$：完整的 quit 证据（§2.6），连同相对 $C$ 的 admission 材料。

**输出**

- `RecoveryRecorded(cid, g+1, ℓ_rec)`：唯一的新 REC，或已存在的同一条 REC；
- 若 $Q$ 中含有 offense，先应用相应的 slash 与禁任；
- `REJECT(reason)`：证据无效，不创建 REC，也不预留恢复槽。

**功能。** 在 L1 上原子地处理所有类型的 quit 证据。REC 的键是 $(\mathrm{cid},g+1)$，不是裸的 $(\beta,r,g+1)$，所以任意 context 都无法抢占真实 session 的恢复槽。新一代的禁任集为 $\mathsf{Ban}(\ell_{\rm rec})$。

#### 4.2.8 `SubmitForce(U, Σ_M)`（原 SubmitUpdate）

**输入**

- $U=(c,\theta_a,\beta_{\rm target},\mathcal P_{\rm new})$（§2.8）；
- $\Sigma_{\mathsf M}$：治理方的签名。

**输出**

- `ForceRecorded(c, p, ts(b))`：记入控制日志的位置与所在块的时间戳；
- `REJECT(reason)`。

**功能。** 合约依次检查：

1. $\Sigma_{\mathsf M}$ 有效；
2. $c>$ `lastSeq`；
3. $\theta_a\ge\mathit{ts}(b)+M_a$，其中 $b$ 是收录这笔交易的 L1 块（R4）；
4. $\mathcal P_{\rm new}$ 给每个区域恰好一个 ON/OFF 标记，且每个区域在循环顺序中恰好出现一次。

合约只记录意图。实际生效位置与效果由 GFR 决定：在 $\theta\ge\theta_a$ 的第一个 Espresso 块处生效；若届时目标分支已关闭或已不是当前分支，整条记录无效。

#### 4.2.9 `SubmitResume(c, P_new, h, Σ_M)`

**输入**

- $c$：控制序号；
- $\mathcal P_{\rm new}$：恢复后使用的 policy；
- $h$：RESUME 的目标高度 $h_{\rm res}$；
- $\Sigma_{\mathsf M}$：治理方的签名。

**输出**

- `ResumeRecorded(c, P_new, h, p, ts(b))`；
- 在同一交易内调用 `BIN.Override`，产生 `CursorSet`；
- `REJECT(reason)`。

**功能。** 合约依次检查：

1. $\Sigma_{\mathsf M}$ 有效；$c>$ `lastSeq`；
2. $\mathcal P_{\rm new}$ 格式正确（同 §4.2.8 第 4 条）；
3. $h$ 高于 Batch Inbox 当前 cursor 的 L2 高度（override 只能前进，`BIN.Override` 本身也会检查）。

通过后，合约以 admin 身份调用 `BIN.Override`，写入的 cursor 为：L2 高度 $h$；frame 摘要 0；epoch_num 不超过当前 L1 高度；GFR 位置字段指向本条 RESUME。字段的精确编码在 Part 6 定义。
合约**不**判断是否处于紧急状态，也**不**计算 $h$：这两件事由诚实的 $\mathsf M$ 按下面的操作规程决定（Part 0 的 A-GOV）。
新分支的锚点由 GFR 在处理这条 RESUME 时确定，为 $\mathrm{last}(D_L)$（§5.5）；按规程选取 $h$ 时，它就是派生链在 $h$ 处的块。

**$\mathsf M$ 的操作规程**（合约不检查）：

1. **时机：** 派生链已越过或偏离全局链之后才发出，即 Batch Inbox 的 cursor 已超窗停滞（§3.5）。过早发出不影响安全，但会撤销本可结算的全局链内容。
2. **前提：** Espresso 已恢复；若故障在批次层，batcher 与 $\mathsf C$ 也已恢复。
3. **$h$ 的选取：**
   - $h$ 高于 cursor 与旧全局链的末端；
   - $t(h)\ge\mathit{ts}(b)+T_{\rm prep}$，其中 $T_{\rm prep}\ge M_a+T_{\rm ready}$（例如 20 min），保证 $h$ 高于派生链在 $b$ 处的末端，并留出准备时间，使新 session 能在 $t(h+1)$ 准时出块；
   - $t(h)-\mathit{ts}(b)\le D_{\rm drift}$ 减去余量，使新 session 的第一块满足 drift；
   - 远离 cursor 编码的上限。
4. **policy：** $\mathcal P_{\rm new}$ 排除仍有故障的区域。

#### 4.2.10 `ReadAuthority(ℓ, query)`

**输入**

- $\ell$：确定的已最终化 L1 高度；
- $\mathit{query}\in\{$`Position(o)`、`Snapshot(e)`、`Ban`、`Recovery(cid, g)`、`Controls(c_from, c_to)`$\}$。

**输出**

- 对应的状态值或有序记录（`Snapshot(e)` 返回完整的权重表、根与发布、激活信息），连同与该高度绑定的 L1 读取结果；
- 已证明不存在则返回 $\bot$；尚未取得该高度则返回 `WAIT`。

**功能。** 把 Part 0 §2.4 的 L1 读取专用于本合约的状态。调用者不能把“没读到快照”解释为零权重。

#### 4.2.11 `Plan(P_β, β, r, E, i_src)`（纯函数）

**输入**

- $\mathcal P_\beta$、$\beta$、$r$：固定 policy 与目标 session；
- $E$：选区参考块，提供 $h(E)$、$\mathsf H(E)$、$\lambda(E)$；
- $i_{\rm src}$：来源区域，genesis 时为 $\bot$。

**输出**

- Plan $\Pi$（§2.3），其中区域可能为 $\bot$；
- `WAIT`：所需的快照或禁任记录尚未取得。

**功能。**

1. 取 $\ell=\max\{\lambda(E),\lambda_\beta\}$，$e=\max\{e':a_{e'}\le\ell\}$，读取 $\mathcal C_e$ 与 $\mathsf{Ban}(\ell)$。
2. 按 $\mathcal L_\beta$ 从 $i_{\rm src}$ 之后循环，取第一个 ON、且未禁任成员的总权重为正的区域；没有则为 $\bot$。
3. 该区域在 $\mathcal C_e$ 中所有正权重成员组成委员会（被禁任的成员仍投票）；$F=\varphi_iW$。
4. $\lambda(\Pi)=\max\{\ell,\ell_{\rm pub}(e)\}$；$\mathrm{id}(\Pi)$ 哈希全部输入与结果。

**选区参考块的取法。** REGULAR 取前驱的初始锚点，所以后继的 plan 提前一个 session 就已确定；FORCE、RESUME、GENESIS 取新锚点本身。RESUME 的锚点是 $\mathrm{last}(D_L)$，所以它的委员会要等派生链算到那里才能确定。
Plan 是纯函数：它不写 L1，也不单独授权任何 session 开始。

#### 4.2.12 `Auth(C, g)`（纯函数）

**输入**

- $C=(\Pi,A)$ 与恢复代 $g\ge0$；当 $g>0$ 时，还需本合约中的 $\mathsf{REC}(\mathrm{cid}(C),g)$。

**输出**

- $\mathsf{AUTH}_{s,0}=\Pi$；或 $\mathsf{AUTH}_{s,g}=(\Pi,\mathsf{REC},\mathsf{Ban}(\ell_{\rm rec}))$；
- `WAIT`：REC 缺失或尚未最终化。

**功能。** 固定该代可以提议的成员集合。委员会、投票权重、配额与门槛在所有代中不变。

#### 4.2.13 `Leader(AUTH_{s,g}, t)`（纯函数）

**输入**

- $\mathsf{AUTH}_{s,g}$；term 编号 $t$。

**输出**

- leader $L_s(g,t)$；未禁任成员为空时返回 $\bot$。

**功能。** 在未禁任成员上按原始权重做 WeightedRand，种子为 $\mathsf H(\texttt{LEADER},\mathit{chain},\beta,r,h(E_s),g,t)$。它是确定性的，不提供不可预测性，也不改变投票集合。

**保证。**

- **G-AUTH-1（确定性）：** Plan、Auth、Leader 是已最终化 L1 前缀的确定函数；所需记录缺失时返回 `WAIT`，绝不返回别的结果。〔L1-S〕
- **G-AUTH-2（不可变）：** 每个 $\mathcal C_e$ 的内容只确定一次；同一组输入的 plan 不变；每个 $(\mathrm{cid},g)$ 至多一条 REC，创建后不变。〔L1-S〕
- **G-AUTH-3（委员会与门槛）：** 一个 session 的委员会与权重对所有 $g,t$ 相同；$w_o$ 为正整数；$F_s=\varphi_iW_s<W_s/2$；slash 只改 escrow，不改 $w_o$。
- **G-AUTH-4（禁任单调）：** Ban 只增不减；$\mathsf{Leader}(\mathsf{AUTH}_{s,g},t)\notin\mathsf{Ban}_{s,g}$；$\mathsf{Ban}_{s,g}\subseteq\mathsf{Ban}_{s,g+1}$。
- **G-AUTH-5（控制纪律）：** 被记录的 Force、RESUME 都满足 §4.2.8–§4.2.9 的检查，无论 $\mathsf M$ 签署了什么。RESUME 的时机与 $h$ 不在合约检查之内，由 A-GOV 保证。〔L1-S4〕
- **G-AUTH-6（问责）：** 一个从未在同一槽位签过冲突消息的 key，不会被 slash；同一 $(o,\mathit{kind},\mathit{slot})$ 至多处罚一次；证据只在它所绑定的 $\mathrm{cid}$ 下有效。〔A-CRYPTO、A-PKI；依赖 G-PROG-5〕
- **G-AUTH-7（质押安全）：** 被释放的 position 满足 Part 0 的 S6。〔A-TS、E-S；依赖 G-GFR-5〕
- **G-AUTH-8（退出隔离）：** 在 epoch $e_x$ 被移出的 position，不在任何 $e\ge e_x$ 的快照中，因而不在读取高度 $\ell\ge a_{e_x}$ 的 plan 的委员会中。
- **G-AUTH-9（唯一 override 入口）：** Batch Inbox 的 cursor override 只能经由 SubmitResume 执行，目标为该 RESUME 记录中的 $h$，且与记录在同一交易内产生。〔与 G-BIN-2 一起成立〕
- **G-AUTH-10（活性）：** 每个准入的调用在 $T_{\rm L1}$ 内最终化，并对活跃诚实方可见；有效的 quit 证据在 $T_{\rm L1}$ 内产生 REC。〔L1-L〕

**依赖。** A-L1、A-CRYPTO、A-PKI、A-TS；G-PROG-5（用于 G-AUTH-6）；G-GFR-5（用于 G-AUTH-7）；Batch Inbox 的 Override 接口（G-BIN-2）。

---

### 4.3 Session Manager（MGR）

**职责。**
每个成员节点上的协调者。它运行 GFR 与派生规则，跟踪全局链 $G$、派生链 $D$ 与生产分支 $R$，创建、进入和关闭 Session Program，准备精确锚点的状态，并把 Program 的输出持久地交付出去：发布、追责、交接与快速确认。它还在本地关闭已不可能进入 $G$ 与 $D$ 的 session（Kill，§4.3.8）。
它不做自由选择：区域、锚点与 $G$ 都来自 GFR、派生规则与 L1 Authority。

**状态。**

- 重算状态 $\mathcal X$ 与 PublicStore（来源字节、区域证据、检查点）；
- 派生视图：从已最终化 L1 与 DA 派生的 $D$，供 Kill 与 RESUME 的锚点状态使用；
- 生产分支 $R$：按 session 分组、已验证的区域结果，可以领先于 $G$；
- `Programs[s]`：固定的 handoff、阶段（Assigned、Running、Terminated）、初次进入与终止记录；
- `Closed`：已关闭的分支与 session 的标记，只增不减，包括 Kill 写入的标记；
- 已准备的状态：基线状态 $\mathit{st}(\mathrm{last}(G))$、常规后继的状态、RESUME 锚点的状态 $\mathit{st}(\mathrm{last}(D_L))$；
- 待发送任务：发布、证据提交与交接消息，在 Program 结束后仍保留。

#### 4.3.1 `ObservePublic(reads)`

**输入**

- $\mathit{reads}$：新增的已最终化 L1、Espresso 读取结果，以及已取回的 DA 数据。

**输出**

- 更新后的 $\mathcal X$ 与派生视图 $D$，以及本次产生的生命周期事件（随即交给 ApplyLifecycle）；
- `WAIT(next key, missing)`。

**功能。** 先验证并持久化来源，再推进派生视图（`DER.Derive`），然后调用 `GFR.AdvanceGlobal`（§5.2）；GFR 处理 RESUME 时所需的 $D_L$ 由派生视图提供。来源不足时，停在 R-WAIT 或派生规则指定的位置；已经产生的事件不撤销。每次推进之后调用 `Kill`（§4.3.8）检查本节点的 Program。Manager 不按消息到达顺序决定任何结果。

#### 4.3.2 `ObserveRegional(Res, handoffEvidence)`

**输入**

- $\mathit{Res}=(C,B,D,\Sigma,\pi)$：来自本区域或其他区域的区域结果；
- $\mathit{handoffEvidence}$：建立 $C$ 的 handoff 链及其来源，或已验证的缓存标识。

**输出**

- `Stored(s, h(B), H(B))`：已验证并保存，生产分支 $R$ 随之更新；
- 若 $B$ 是负责 session 的末块，另输出常规后继的 handoff；
- `REJECT` 或 `WAIT`。

**功能。**
用 CheckFINAL 在真实的 $C$ 下检查结果，扩展 $R$，并把结果交给 Publisher（`PUB.Offer`）。
若 $B$ 是末块 $T_s$：

1. 按 $\mathsf{Plan}(\mathcal P_\beta,\beta,r+1,A_s,i_s)$ 算出常规后继，并取 $q^{\rm open}=\max\{\mathrm{height}(\xi(T_s)),\ q^{\rm open}_s+1\}$；
2. 若后继选中本区域，调用 InstallHandoff，不必等末块进入 $G$；
3. 无论是否选中本区域，都把完整的末块结果持久地发往后继区域。

已关闭版本的结果只作为历史保存，不再推进 $R$。

#### 4.3.3 `InstallHandoff(handoff)`

**输入**

- $\mathit{handoff}$（§2.7）；本节点的身份 $v$。

**输出**

- `Assigned(s, C)`：保存唯一的 handoff，并调用 `PROG.Create`；
- `NotAssigned`：handoff 有效，但本节点不在委员会中；
- `REJECT`：与已安装的 handoff 冲突，或目标已关闭；`WAIT`：来源不全。

**功能。** 两步检查：

1. **VerifyHandoffHistory：**
   - REGULAR：核对前驱的边界、末块高度与完整末块结果，并重算目标与 $q^{\rm open}$；
   - FORCE、RESUME、GENESIS：核对它与 GFR 在相应重算位置输出的 `SessionAssigned` 完全一致；RESUME 时用派生视图核对锚点 $A=\mathrm{last}(D_L)$。
2. **CanEnter：** 目标版本未关闭；handoff 选中本区域；该 pair 既未进入也未终止。

同一 pair 的 handoff 一经安装，就不再被不同字段的 handoff 替换。

#### 4.3.4 `PrepareAndEnter(s, stateMaterial)`

**输入**

- $s$：已 Assigned 的 session；
- $\mathit{stateMaterial}$：精确锚点 $A_s$ 的可执行状态及其验证依据；或者从已验证检查点到 $A_s$ 的完整执行输入（普通块的祖先；RESUME 时为派生出 $D_L$ 所需的 L1 与 DA 输入）。

**输出**

- ReadyBundle，以及 `PROG.Enter` 的结果 `Entered(s, τ_entry)`；
- `WAIT`：仍需下载、重建或验证；`REJECT`：目标已关闭。

**功能。** 完成 Part II v4.9 定义的 Ready 之后，原子地再次检查 CanEnter，然后调用 `PROG.Enter`。只有可直接使用的完整状态才满足 Ready；状态根或下载地址都不够。

#### 4.3.5 `HandleProgramEvent(s, event)`

**输入**

- $\mathit{event}\in\{$`Conf(B)`、`Result(Res)`、`Complete(T_s, Res)`、`Quit(C, g, Q)`、`Offense(C, o, ev)`$\}$。

**输出**（持久化的任务；已处理过的事件返回原结果）

- `Conf` → 向客户端提供快速确认；
- `Result` → `PUB.Offer`，并更新 $R$；
- `Complete` → 记录正常终止，把末块结果发往后继区域；
- `Quit` → `AUTH.SubmitQuit`；REC 最终化之后，把 `AUTH.Auth(C, g+1)` 经 `PROG.Provide` 交给 Program；
- `Offense` → `AUTH.SubmitOffense`。

**功能。** 把 Program 的逻辑输出变成不会随 Program 退出而丢失的交付任务。终止不删除结果、证据或未完成的发送。

#### 4.3.6 `ApplyLifecycle(event)`

**输入**

- GFR 的一个生命周期事件（§2.7）。

**输出**

- 对本节点 Program 表、生产分支与负责性视图的更新。

**功能。** 按事件类型处理：

- `SessionAssigned(ν)`：若选中本区域，构造对应的 handoff 并调用 InstallHandoff。若同一 pair 已有提前安装的常规 handoff，核对二者一致。
- `SessionClosed`、`BranchClosed`：先原子地持久化关闭标记，再对受影响的 Program 调用 `PROG.Close`，取消其计时器与一切后续 BFT 动作；尚未进入的 Assigned 实例同样关闭。
- `Resumed(c, β′, h_f, A)`：把基线状态与生产分支 $R$ 重置为 $D_L$，准备 $\mathit{st}(A)$；旧分支的 Program 已由同一位置的 `BranchClosed` 关闭。
- `Halted`、`Pending`：更新负责性视图；它们不创建 Program。
- `Appended`：推进基线状态。

每处理完一批事件，调用 `Kill`（§4.3.8）检查剩下的 Program。

#### 4.3.7 `Responsibility()`

**输入**

- 无；读取 $\mathcal X$ 与已知的常规后继 plan。

**输出**

- (当前负责的 session 与区域，已知的下一个负责区域)；Tail 为 PENDING、OPEN 或 HALT 时，当前负责者为 $\bot$。

**功能。** 供 `BLD.SubmitTx` 判断是否接受交易，也可经 RPC 提供给客户端用于路由。被 Kill 的 session 不再算作负责者。

#### 4.3.8 `Kill(s)`

**输入**

- $s$：本节点上处于 Assigned 或 Running 的 session；
- 本节点的派生视图 $D$（只从已最终化的 L1 派生）与重算状态 $\mathcal X$；
- $\mathrm{chain}(s)$：$s$ 的锚点 $A_s$ 及其全部祖先，加上 $s$ 已提交的块。

**输出**

- `Killed(s, reason)`，$\mathit{reason}\in\{\mathsf{DEAD\_D},\mathsf{DEAD\_G}\}$：关闭标记已持久化，并已调用 `PROG.Close(s, KILL)`；
- `Alive`：两种判死条件都不成立。

**功能。** 判断 $s$ 是否已不可能再进入 $G$ 与 $D$。

- **D 判死（$\mathsf{DEAD\_D}$）。** $D$ 已在某个高度上定下一个块，而 $\mathrm{chain}(s)$，连同 $s$ 将来要出的块，在该高度上不可能是它：
  - **分叉：** $D$ 与 $\mathrm{chain}(s)$ 在某个共同高度上的原生块不同；
  - **越过：** $D$ 在 $s$ 尚未出块的某个高度 $k\le H_s$ 上，已有由窗口回退或 override 填出的 deposit-only 块。

  实现上可以只读 L1：cursor 超窗停滞（§3.5）时，任何还要越过 cursor+1 出块的 session 都已满足 $\mathsf{DEAD\_D}$，不必等 DA。
- **G 判死（$\mathsf{DEAD\_G}$）。** $G$ 在 $s$ 需要的某个高度上已经接纳了别的块；或者 $s$ 的下一个高度在 $G$ 中已过期而仍没有块。GFR 关闭 $s$ 或其分支的情形，由 ApplyLifecycle 直接处理（§4.3.6）。

本地的 $\mathrm{chain}(s)$ 可能落后于 $s$ 的真实进度。$s$ 已经出好、并已进入 $D$ 的块来自批次，不是 deposit-only，不会触发“越过”，所以落后的成员不会被误判。

Kill 只停止本节点的区域工作：它不改变 $G$，也不产生任何 L1 交易。Espresso 停机时 GFR 受 R-WAIT 约束而暂停，Kill 是那时唯一能起作用的关闭依据。它与 GFR 的关闭只在时间上不同，结论一致。

具体动作：

- 先持久化关闭标记，再关闭 Program；
- 保留区域结果、签名记录与证据，未完成的证据提交照常进行；
- Publisher 不再发布该 session 尚未进入 $G$ 的结果；
- 负责性视图把它从负责者中去掉。

**判据的来源。** 原方案的判据是：(a) $s$ 不是从 $\mathrm{last}(D)$ 派生的；(b) $s$ 从 $\mathrm{last}(D)$ 派生，但不是从 $\mathrm{last}(G)$ 派生的。$\mathsf{DEAD\_D}$ 是 (a) 的精确写法；$\mathsf{DEAD\_G}$ 的第一种情形对应 (b)，按“$G$ 领先 $D$ 时，$s$ 还必须从 $\mathrm{last}(G)$ 派生”理解，待确认。

**保证。**

- **G-MGR-1（handoff 真实）：** 只为通过 VerifyHandoffHistory 的 handoff 创建 Program。REGULAR handoff 来自前驱真实的末块区域结果；其他 cause 与 GFR 的 `SessionAssigned` 完全一致。同一 pair 的固定字段不被替换。
- **G-MGR-2（单次进入，不复活）：** 每个 pair 至多一次初次 Enter；已终止或已关闭的 pair 不再进入。
- **G-MGR-3（及时关闭）：** 处理一个关闭事件之后、本节点对受影响 session 做任何后续 BFT 动作之前，关闭标记已经持久化，Program 已经关闭。
- **G-MGR-4（无自由裁量）：** 区域、锚点与 $G$ 全部来自 GFR、派生规则与 L1 Authority；Kill 只依据从已最终化 L1 派生的 $D$ 与 $\mathcal X$。同一公开输入下，所有诚实 Manager 做出相同的生命周期决定。
- **G-MGR-5（持久与交付）：** 区域结果、证据与待发送任务在 Program 终止或节点崩溃后保留；每份 quit 与 offense 证据都会提交给 L1 Authority。〔持久化存储、A-RELAY〕
- **G-MGR-6（准备活性）：** 在分配事件之后 $T_{\rm ready}$ 内达到 Ready，并在 Enter 时发出 ReadyBundle。〔$T_{\rm ready}$〕
- **G-MGR-7（Kill 不误杀、及时）：**
  - 若 `Kill(s)` 输出 `Killed`，则 $s$ 的任何后续块都不可能出现在 $D$ 中；本节点此后不再为 $s$ 签任何消息。
  - 反过来，$D$ 越过或偏离 $\mathrm{chain}(s)$ 之后，至多再过一个 L1 块，本节点就会关闭 $s$；用派生视图判断时，前提是本节点已派生到该高度。

  〔L1-S、DA-S1；依赖 G-DER-1〕
- **G-MGR-8（负责性视图）：** `Responsibility()` 与 GFR 的 Tail 及常规后继的 plan 一致。

**依赖。** GFR（G-GFR-1、3、6、7）；L1 Authority（G-AUTH-1、2、10）；Derivation Rule（G-DER-1、3，用于派生视图、Kill 与 RESUME 的锚点状态）；Session Program；Publisher；Exec、CheckFINAL；跨区域网络 $\Delta_{\rm cross}$（只用于交接消息，仅影响活性）；$T_{\rm ready}$。

---

### 4.4 Session Program（PROG）

**职责。**
一个固定 session 在一个成员节点上的区域共识实例：Part I 的同步共识，加上 Part II 的适配（加权门槛、有限配额、DRAIN、恢复代、ReadyBundle 直接启动）。它产生区域最终块与快速确认，并输出 quit 与 offense 证据。
不同 session 的实例互不共享核心状态。

**状态。**

- 固定：$s$、$C_s$、$H_s$、$\mathrm{TB}$、$K$、锚点 $A_s$ 与入口证据 $Q^0_s$；
- 可变：阶段（Assigned、Running、Terminated）；视图 $(g,t)$；逐槽位的签名记录；锁、投票、QC、计时器；已提交前缀与执行状态。

#### 4.4.1 `Create(s, handoff)`

**输入**

- $s$；Manager 已验证的 handoff。

**输出**

- `Assigned(s, C)`；相同实例已存在时返回它；
- `REJECT`：handoff 不同，或 pair 已关闭、已终止。

**功能。** 建立独立的 session 状态与持久化身份。不复制前驱的锁；不启动计时器，也不授予签名权。

#### 4.4.2 `Enter(s, readyBundle)`

**输入**

- $s$；完整的 ReadyBundle，其中的状态必须对应固定的锚点 $A_s$。

**输出**

- `Entered(s, τ_entry)`；同时向本区域全体成员发送 ReadyBundle；
- `REJECT`：权限已失效。

**功能。** 原子地复查权限之后，只进入一次：初始化入口锁 $Q^0_s$、执行状态与 $(g,t)=(0,0)$，置为 Running，并启动进度检查。leader 随即可以直接提议第一块（父块为 $A_s$），没有额外的启动投票轮。

#### 4.4.3 `Provide(s, input)`

**输入**

- $\mathit{input}$ 为以下有类型对象之一：
  - `Candidate(parent, x, nctx)`：来自 Builder；
  - `References(R_L1, R_E)`：已最终化的 L1 与 Espresso 引用；
  - `Authority(g, AUTH_{s,g})`：来自 L1 Authority 的恢复代授权。

**输出**

- `Installed(kind, id)`；
- `WAIT`，或 `REJECT`（context 不符、试图修改固定字段，或目标已关闭）。

**功能。** 每种输入都须与当前 context 和协议进度相符。提供新的授权不改变委员会与权重；提供候选不会自动触发投票。

#### 4.4.4 `Step(s, event)`

**输入**

- $\mathit{event}$ 为 `Receive(m)`（完整的签名消息，带该类型所需的 admission 与父块证据），或 `Timer(id, τ)`。

**输出**

- $\mathit{peerMessages}$：按协议发出的 PROPOSE、投票、STATUS、NEWVIEW、BLAME、FINAL 与 ReadyBundle；
- $\mathit{programEvents}$：新产生的 Conf、Result、Complete、Quit、Offense，可以为空；
- 更新后的实例状态。

**功能。** 执行区域状态转移：先处理有效的冲突与 quit 证据，再处理投票与计时器。每个 BFT 动作之前都检查实例仍获授权（未被关闭）。签名之前先持久化槽位记录。

#### 4.4.5 `Close(s, reason)`

**输入**

- $s$；$\mathit{reason}\in\{\mathsf{COMPLETE},\mathsf{FORCE},\mathsf{EXPIRED},\mathsf{RESUME},\mathsf{KILL}\}$，由 Manager 给出。

**输出**

- `Terminated(s, reason)`；重复关闭返回同一结果。

**功能。** 立即取消该实例的一切后续 BFT 权限，包括延迟的回调、term 变化与恢复。关闭不撤销 $G$，不产生新的 FINAL，也不释放质押。

**输出事件**（交给 `MGR.HandleProgramEvent`）

| 事件 | 何时产生 |
| --- | --- |
| `Conf(s, B)` | 本成员在区域内提交了 DATA 块 $B$（快速确认） |
| `Result(Res)` | 某个 DATA 块的完整区域结果已经形成（含 FINAL 证书与路径） |
| `Complete(T_s, Res)` | 本 session 的末块 $T_s$ 已有完整区域结果；实例正常结束 |
| `Quit(C, g, Q)` | 第 $g$ 代的完整 quit 证据已经形成 |
| `Offense(C, o, ev)` | 观察到成员 $o$ 的冲突签名 |

**生产约束。** 诚实成员只为满足以下全部条件的块提议、投票与签 FINAL；GFR 对其中可以公开检查的部分再检查一次（§5.4）。

- **高度：** $h\le H_s$；
- **引用：** $\lambda(B)\ge\lambda^{\rm open}$、$\mathrm{height}(\xi(B))\ge q^{\rm open}$；R-FRESH：$\mathit{ts}(\lambda(B))\ge t(h(B))-\delta_{\rm ref}$；
- **派生一致：** 候选在父块上满足 DerivOK（§3.2，经 `BLD.ValidateCandidate`）；
- **时效：** 不为来不及在 $t(h)+W_{\rm exp}$ 之前进入 Espresso 的高度出块；新锚点落在过去时，允许以快于 $b_t$ 的速度追赶。

**保证。**

- **G-PROG-1（区域一致，S4(a)）：** $s$ 良态时，两个诚实成员不会在同一高度提交不同的块。〔良态、$\Delta_i$、A-CRYPTO〕
- **G-PROG-2（FINAL 可靠）：** 若 A-BYZ 对 $s$ 成立，则任何在 $\mathrm{cid}_s$ 下权重 $>F_s$ 的 FINAL 证书所签的 DATA 块 $D$，都曾被某个诚实成员在完整验证之后提交。因此 $D$ 及其同 session 祖先在各自父块上满足 DerivOK（原生有效、deposit 前缀、drift 与 R-FRESH），并满足配额与引用下界。〔A-BYZ、A-CRYPTO；依赖 G-MGR-1、G-BLD-1、G-BLD-2〕
- **G-PROG-3（生产约束）：** 诚实成员只为满足上述生产约束的块提议、投票与签 FINAL。
- **G-PROG-4（隔离与关闭）：** 不同 session 的实例不共享核心状态；关闭之后不再产生任何签名；每个实例至多一次初次 Enter。
- **G-PROG-5（诚实不留证据）：** 诚实成员不会在同一槽位签两份冲突的消息，因为签名前先持久化槽位记录；因此不存在针对诚实 key 的有效 offense 证据。它输出的 quit 证据都有效。〔持久化存储〕
- **G-PROG-6（同步进入）：** 第一个诚实成员 Enter 之后 $\Delta_i$ 内，所有诚实成员都已进入。〔良态、$\Delta_i$〕
- **G-PROG-7（区域活性）：** 若 $s$ 良态、A-LEAD 成立、候选与引用可得，且未被关闭，则每个 fresh 高度在 $T_{\rm pos}$ 内被提交（输出 Conf），末块结果在 $T_{\rm sess}$ 内形成（输出 Complete）；leader 失职时会形成 quit 证据。
- **G-PROG-8（证据输出）：** 诚实成员观察到的 quit 或 offense 证据，在 $\Delta_i$ 内输出给 Manager。

$T_{\rm pos}$ 与 $T_{\rm sess}$ 是由协议推导的时限，在 Session Program 的 Part 中给出。

**依赖。** Session Manager（G-MGR-1、2、3、7）；L1 Authority（G-AUTH-1 至 4，以及 REC）；Block Builder（G-BLD-1、3）；Exec；A-BYZ、A-LEAD、$\Delta_i$、A-CRYPTO。

---

### 4.5 Publisher（PUB）

**职责。**
把完整的区域结果发布到 Espresso 的本链 namespace。任何持有完整结果的参与方都可以发布，不要求原 leader 在线。发布本身不授予任何追加权限。

**状态。** 按 $(\mathrm{cid},\mathsf H(B))$ 去重的待发布队列；Espresso 提交状态。

#### 4.5.1 `Offer(Res, handoffRef)`

**输入**

- $\mathit{Res}=(C,B,D,\Sigma,\pi)$：完整的区域结果；
- $\mathit{handoffRef}$：建立 $C$ 的 handoff 与来源，或可在处理预算内解析的标识。

**输出**

- `Queued(id)`，随后 `Submitted(id, rid_E)`，再随后 `Recorded(id, q)`：发布位置与可验证的读取结果；
- `REJECT`：证据无效；`WAIT`：证据暂缺。

**功能。** 可选地先用 CheckFINAL 自查，再把 kernel 所需的全部对象（块体、FINAL 签名、路径）打包，经 Part 0 的 Espresso 提交接口发布。重复的或分片的结果可以合并；停机期间队列持久保存。`Recorded` 只表示已经发布，是否进入 $G$ 由 GFR 决定。
已被 Kill 或已关闭的 session，尚未进入 $G$ 的结果不再交给 Publisher；队列中已有的可以丢弃，它们不影响任何结果。

**保证。**

- **G-PUB-1（不授权）：** 发布不授予追加权限；$G$ 只由 GFR 决定。
- **G-PUB-2（完整与去重）：** 发布的内容包含 CheckFINAL 所需的全部对象；同一结果的重试不会重复发布。
- **G-PUB-3（活性）：** 在 $\tau$ 交给诚实 Publisher 的完整结果，在 $\tau+T_{\rm pub}+T_E$ 之前在 Espresso 中最终化并可读。〔E-L、费用、A-RELAY〕

**依赖。** A-ESP；CheckFINAL。

---

### 4.6 Batcher 与 Batch Inbox

这一节按组合式抽象放在一起，但分成两组接口：Batcher 是角色，Batch Inbox 是 L1 合约。

#### Batcher（BAT）

**职责。**
把 $G$ 中的普通块编码成批次，分发到 DA，请 Certifier 认证，再提交到 Batch Inbox。它不为 deposit-only 高度提交任何批次（R27）。Batcher 可以有多个，它不决定内容：内容由 $G$ 决定。

**状态。** channel 构造状态；它自己运行 GFR 得到的 $G$；从 L1 读到的 Batch Inbox cursor。

##### 4.6.1 `BuildPayload(range, cur, channelState)`

**输入**

- $\mathit{range}=[a,b]$：$G$ 中尚未被接纳的连续高度；
- $\mathit{cur}$：从 L1 读到的 Batch Inbox cursor；
- $\mathit{channelState}$：本 batcher 已构造的 channel 前缀；新 channel 时为空。

**输出**

- $\mathit{payload}$：同一 channel 的一个或多个连续的新 frame，以及更新后的 $\mathit{channelState}$；
- `REJECT`：区间不连续或超出 $G$；`WAIT`：$G$ 或来源尚未就绪。

**功能。** 只使用 $G$ 中的普通块，不使用生产分支 $R$ 中尚未进入 $G$ 的内容。每个高度写入完整的普通交易列表与元数据。RESUME 之后，从 override 写入的 cursor 接着提交新分支的块；被 override 越过的高度不再提交。

##### 4.6.2 `Deliver(payload)`

**输入**

- $\mathit{payload}$ 及其 channel 信息。

**输出**

- `Accepted(prev, next, d)`：批次已被 Batch Inbox 接纳；
- `WAIT`：DA、Certifier 或 L1 暂不可用；`REJECT`：例如 cursor 已被其他 batcher 推进。

**功能。** 依次调用 `DA.Disperse(payload)` 得到 $(d,\pi_{\rm DA})$，调用 `CERT.CertifyBatch(cur, payload, d, π_DA)` 得到 $\chi$，再调用 `BIN.SubmitBatch`。cursor 被其他 batcher 推进时，从新的 cursor 重新构造。

**保证。**

- **G-BAT-1（只提交 $G$ 的内容）：** Batcher 只为 $G$ 中的块构造批次。Certifier 也会强制这一点（G-CERT-1）。
- **G-BAT-2（RESUME 之后接续）：** RESUME 之后，Batcher 只为新的 $G$ 中高于 $h_{\rm res}$ 的块构造批次，并以 override 写入的 cursor 为 prev。
- **G-BAT-3（活性）：** 至少有一个诚实的注册 batcher 时，$G$ 中每个块在进入 $G$ 之后 $T_{\rm batch}$ 内被 Batch Inbox 接纳。〔Certifier 在线、DA-L、L1-L、A-RELAY、交付预算〕

**依赖。** GFR（G-GFR-1）；Certifier（G-CERT-3）；Batch Inbox（G-BIN-4）；A-DA。

#### Batch Inbox（BIN）

**职责。**
L1 合约，即 mega-espresso 的 batch inbox，按 R17、R22 修改。它按 cursor 连续接纳带 Certifier 签名的批次，执行唯一的 cursor override 入口，并按 L1 顺序发出事件供 Derivation Rule 使用。

**状态。** cursor；batcher 注册表（primary、alternatives、独占窗口）；Certifier 公钥注册；admin 固定为 L1 Authority。

##### 4.6.3 `SubmitBatch(sub)`

**输入**

- $\mathit{sub}=(\mathit{prev},\mathit{next},d,\pi_{\rm DA},\chi)$；调用者是注册的 batcher（EOA）。

**输出**

- `CursorAdvanced(prev, next, d)`；
- `REJECT(reason)`。

**功能。** 沿用 mega-espresso 的接纳规则：$\mathit{prev}$ 必须等于当前 cursor；调用者须有提交权限（primary，或独占窗口过后的 alternative）；DA 证书须通过检查；Certifier 签名须覆盖 $(\mathit{prev},d,\pi_{\rm DA},\mathit{next},\dots)$。Batch Inbox 只接纳 Certifier 认证过的内容。

##### 4.6.4 `Override(next)`

**输入**

- $\mathit{next}$：目标 cursor，其中 L2 高度为 RESUME 给出的 $h_{\rm res}$，frame 摘要为 0，origin 编号不超过当前 L1 高度；
- 调用者：admin，即 L1 Authority。

**输出**

- `CursorSet(old, next)`；
- `REJECT(reason)`。

**功能。** 只能由 L1 Authority 在 SubmitResume 中调用（R17）。只能前进，并放弃任何未完成的 channel。Derivation Rule 根据 `CursorSet` 把 (派生末端, $h_{\rm res}$] 填成 deposit-only（G-DER-3）；GFR 在处理同一条 RESUME 时读取这一结果（§5.5）。

##### 4.6.5 `ReadCursor(ℓ)`

**输入**

- $\ell$：确定的已最终化 L1 高度。

**输出**

- 该高度上的 cursor 及其 L1 读取结果；或 `WAIT`。

**功能。** 供 Batcher、Certifier 与 L1 Authority（RESUME 的条件）读取 cursor。

batcher 注册与 Certifier 公钥注册沿用 mega-espresso，这里不再列出。

**保证。**

- **G-BIN-1（只接纳认证且连续的提交）。** 〔L1-S4、A-CERT〕
- **G-BIN-2（唯一 override 入口）：** cursor 只能被已接纳的提交或 L1 Authority 的 Override 改变。
- **G-BIN-3（事件有序）：** `CursorAdvanced` 与 `CursorSet` 按 L1 顺序产生。〔L1-S〕
- **G-BIN-4（活性）：** 准入的提交在 $T_{\rm L1}$ 内最终化。〔L1-L〕

**依赖。** Certifier（G-CERT-1）；L1 Authority（G-AUTH-9）；A-L1。

---

### 4.7 Certifier（CERT）

**职责。**
可信方（Part 0 的 A-CERT）。它核对一个批次的内容与 $G$ 一致，并核对 DA 绑定，然后签名。它不出具任何关闭证书或模式证书（R15）。

**状态。** 自己的 L1、Espresso 与 DA 视图；与观察者同版本的 GFR 状态。

#### 4.7.1 `CertifyBatch(prev, payload, d, π_DA)`

**输入**

- $\mathit{prev}$：本批次开始时的 cursor；
- $\mathit{payload}$：完整的提交字节；
- $d$、$\pi_{\rm DA}$：这份 payload 已完成 DA 认证的结果；
- 延续 channel 所需的已接纳 frame：经 DA 读取。

**输出**

- $\chi$（§2.10）；
- `REJECT`：内容与 $G$ 不符；`WAIT`：来源或 frame 尚未取得。

**功能。**

1. 从 L1 读取并核对 $\mathit{prev}$；把 GFR 至少运行到 $\mathit{prev}$ 被写入的那个 L1 位置之后，取这个位置为 $\mathit{gpos}$。这样，写入 $\mathit{prev}$ 的 RESUME 一定已经处理过，$\mathsf C$ 不会以新的 cursor 去认证旧全局链的内容。
2. 解码 payload，逐个高度核对：每个高度都是 $G$ 在 $\mathit{gpos}$ 处的普通块，普通交易列表、时间戳与 origin 都与它一致。出处为 $\mathsf{DER}$ 的高度不认证，它们在派生中已经确定。
3. 核对 channel 与 frame 的连续性，以及 $d$ 与 payload 的绑定。
4. 计算 $\mathit{next}$ cursor，然后签名。未闭合的 channel 不消耗高度。

**保证。**

- **G-CERT-1（对应）：** 带 $\chi$ 的批次所解码出的每个高度，都与 $G$ 在 $\mathit{gpos}$ 处给出的普通块一致；$d$ 绑定这份 payload。〔A-CERT；依赖 G-GFR-1〕
- **G-CERT-2（切面绑定）：** $\chi$ 绑定 $\mathit{gpos}$，且 $\mathit{gpos}$ 不早于 $\mathit{prev}$ 被写入的 L1 位置。因此：
  - 任何诚实观察者在更长的日志下得到的 $G$，都以 $\mathit{gpos}$ 处的 $G$ 为前缀，RESUME 的情形除外；
  - RESUME 之后签发、以 override 写入的 cursor 为 prev 的证书，只覆盖新 $G$ 的内容；
  - RESUME 之前签发的证书，其 prev 是旧 cursor，override 之后不再被 Batch Inbox 接纳。
- **G-CERT-3（活性）：** 在线期间，$T_{\rm cert}$ 内作出响应。

**依赖。** GFR（同版本）；A-DA、A-L1、A-ESP、A-CERT。

---

### 4.8 原生组件

以下组件沿用现有合约，不重新设计；这里只列出其他模块用到的接口。

#### Deposit Portal（DP）

OP Stack 的 deposit 合约（OptimismPortal），对应 MegaETH specs 的《Deposits and Force Inclusion》（`rollup/deposits.md`）。

##### 4.8.1 `Deposit(tx)`

**输入**

- 一笔从 L1 发起的 L2 交易；调用者是任何 L1 用户。

**输出**

- `DepositEvent(ℓ, logIndex, depositTx)`。

**功能。** 这是强制包含（抗审查）的入口。按原生规则，以 L1 块 $e$ 为 origin 的第一个 L2 块包含 L1 块 $e$ 中的全部 deposit。

##### 4.8.2 `ReadDeposits(ℓ)`

**输入**

- $\ell$：L1 高度。

**输出**

- L1 块 $\ell$ 中按顺序排列的全部 deposit；或 `WAIT`。

**功能。** 供 Builder（DerivOK）、Exec 与 Derivation Rule 读取。

**沿用的保证。** deposit 按 L1 顺序记录且不可变〔L1-S〕；准入的 deposit 在 $T_{\rm L1}$ 内最终化〔L1-L〕；它们按原生规则进入 $G$（L-fi）与 L1 派生链（L2-L2）。

#### 状态结算（SETTLE）

现有的 Kailua / fault-proof 路径，不修改。

##### 4.8.3 `SubmitStateClaim(a, b, parentClaim, inputRef, stateRoot)`

**输入**

- $[a,b]$：claim 覆盖的连续 L2 高度；$\mathit{parentClaim}$：前置 claim；
- $\mathit{inputRef}$：确定该段派生输入的 L1 位置与 DA 承诺；$\mathit{stateRoot}$：声称的状态根。

**输出**

- `ClaimRegistered(claimId)`，随后 `Resolved(claimId, accepted/rejected)`；未决时为 `PENDING`。

**功能。** 把状态根交给现有的争议流程。该流程使用与 Derivation Rule 相同的派生，保证被接受的状态根与同一派生输入的正确执行一致。

---

## 5. Global Finality Rule（GFR）

**职责。**
确定性规则：从 L1 日志与 Espresso 日志计算重算记录、全局链 $G$ 与 Tail，并输出生命周期事件；处理 RESUME 时，另调用派生规则取得 $D_L$（§5.5）。它由外部观察者、Session Manager、Certifier 与 Batcher 分别运行，不构成额外的共识层，也没有任何自由裁量。Part 0 中把它记作 $\mathsf{Replay}$。
GFR 不在任何地方生成或预测 deposit-only 块；$G$ 中的 deposit-only 块只来自 RESUME 时采纳的派生链。

**状态。** 重算状态 $\mathcal X$（§2.9）。来源字节保存在使用方的 PublicStore 中。

### 5.1 `Init(checkpoint)`

**输入**

- genesis 检查点 $(A_0,\mathcal P_0,q_{\rm init}$，初始控制前缀$)$；或授权检查点，此时给出完整的 $\mathcal X$。

**输出**

- 初始状态 $\mathcal X_0$。genesis 时 $G=A_0$，$\mathsf{Tail}=\mathsf{OPEN}((\mathsf{GENESIS},0,0,\bot,A_0,q_{\rm init}))$。

**功能。** 后来的检查点必须给出完整的 $\mathcal X$，不能只给 $\mathrm{last}(G)$。

### 5.2 `AdvanceGlobal(X, LOG_L1, LOG_E)`

**输入**

- $\mathcal X$；
- 调用者的已最终化 L1 日志与 Espresso 日志；
- 派生规则的读取接口：只在处理 RESUME 时使用，用来取得 $D_L$（§5.5）。

**输出**

- $(\mathcal X',\Delta\mathrm{Rec},\mathit{events})$：新的状态、新增的重算记录，以及本次产生的生命周期事件（§2.7）；
- `WAIT(next key, missing)`：下一个事件还不满足 R-WAIT，或 RESUME 处所需的 L1 与 DA 数据尚未取得；已处理的部分保留。

**功能。** 按 §2.9 的合并顺序逐个处理事件，只要 R-WAIT 允许就继续。三类事件的处理如下。

**(1) Espresso 块 $J_q$（键 $\theta_q$）。**

1. **执行 Force。** 对生效键不超过 $\theta_q$、尚未处理的 Force 记录，按序号依次处理。若目标分支是当前分支且未关闭：输出 `BranchClosed(β, FORCE)`；令 $\beta:=\beta+1$；安装 $\mathcal P_\beta:=(S_{\rm new},\mathcal L_{\rm new},\max\{\lambda_{\beta-1},\lambda_U\})$；令 Tail 为 $\mathsf{OPEN}(\omega)$，cause 为 $\mathsf{FORCE}$，锚点 $A:=\mathrm{last}(G)$；然后在 $J_q$ 处开出新 session（见下文“开出新 session”）。若目标分支已关闭或已不是当前分支，整条记录无效。
2. **完成待开的 session。** 若 Tail 为 $\mathsf{OPEN}(\omega)$，在此开出新 session。
3. **追加普通块。** 当 $\mathsf{Tail}=\mathsf{NEXT}(\nu)$，且已发布的对象集合中存在满足 `Eligible`（§5.4）的普通块时，追加其中哈希最小者，输出 `Appended`。若该块是 $\nu$ 的末块（$h=H(\nu)$）：
   - 输出 `SessionClosed(sid(ν), COMPLETE)`；
   - 计算常规后继 $\mathsf{Plan}(\mathcal P_\beta,\beta,r+1,A(\nu),i(\nu))$，取 $q^{\rm open}=\max\{\mathrm{height}(\xi(B)),\ q^{\rm open}(\nu)+1\}$；
   - 选出区域则 $\mathsf{Tail}:=\mathsf{NEXT}(\nu')$ 并输出 `SessionAssigned(ν')`；没有合格区域则 $\mathsf{Tail}:=\mathsf{PENDING}(\omega)$ 并输出 `Pending`。

   然后继续寻找下一个合格的块。

**(2) L1 事件（键 $\mathit{ts}(b_e)+M_a$）。** 合并顺序中只有两类 L1 事件：Force（键为 $\theta_a$，在 (1) 中执行）与 RESUME。v0.1 的 STOP、`CursorSet` 与截止事件 $\mathrm{ddl}(h)$ 都已删除。

- **RESUME 记录** $(c,\mathcal P_{\rm new},h_{\rm res})$，位于 L1 块 $L$（R-RES）：
  1. 取得 $(D_L,h_f,A):=\mathsf{ResumeChain}(\mathcal X,L)$（§5.5）。数据不足时返回 `WAIT`，停在这条记录上；等待不改变结果。
  2. 若当前分支尚未关闭，输出 `BranchClosed(β, RESUME)`，关闭其上仍未关闭的 session。
  3. 把 $G$ 替换为 $D_L$：$h_f$ 之前的块保留原来的出处与信封；其余块的出处为 $\mathsf{DER}$。
  4. 令 $\beta:=\beta+1$，安装 $\mathcal P_\beta:=(S_{\rm new},\mathcal L_{\rm new},\max\{\lambda_{\beta-1},\lambda_{\rm RESUME}\})$。
  5. 令 Tail 为 $\mathsf{OPEN}(\omega)$，cause 为 $\mathsf{RESUME}$，锚点为 $A$；输出 `Resumed(c, β, h_f, A)`；在下一个 Espresso 块处开出新 session。

  同一交易产生的 `CursorSet` 不单独处理：它的效果已经包含在 $D_L$ 中。

**(3) 时间事件 $\mathsf{Expire}(h)$（键 $t(h)+W_{\rm exp}$）。**

- **过期停住：** 若 $h=h(\mathrm{last}(G))+1$、$G$ 尚无高度 $h$，且 Tail 不是 $\mathsf{HALT}$：输出 `BranchClosed(β, EXPIRED)`，关闭当前分支上的全部 session，包括提前进入的常规后继；令 Tail 为 $\mathsf{HALT}(\omega)$，输出 `Halted`。此后 $G$ 停在 $h-1$，只有 RESUME 能替换 Tail。
- 其他情形无操作。v0.1 的 R-DEP 与 R-OVR 已删除：GFR 不补 deposit-only 块，也不自动轮换。

**开出新 session。** FORCE、RESUME 共用以下步骤，在 Espresso 块 $J$ 处执行：

1. 锚点 $A$ 取 $\omega$ 中已固定的值：FORCE 为生效时的 $\mathrm{last}(G)$，RESUME 为 $\mathrm{last}(D_L)$，此时 $\mathrm{last}(G)=A$。按 v4.9 的规则确定目标轮次 $r'$ 与来源区域 $i_{\rm src}$（RESUME 时以被关闭分支上最后负责的 session 为准），计算 $\Pi:=\mathsf{Plan}(\mathcal P_\beta,\beta,r',A,i_{\rm src})$，以 $A$ 为选区参考块。
2. 按 Plan 的结果：
   - 选出区域 $i$：令 $\nu:=(\beta,r',i,A,h(A)+\mathrm{TB},q,\mathit{cause},\Pi)$，$\mathsf{Tail}:=\mathsf{NEXT}(\nu)$，输出 `SessionAssigned(ν)`；
   - 区域为 $\bot$：$\mathsf{Tail}:=\mathsf{PENDING}(\omega)$，输出 `Pending`；
   - Plan 返回 `WAIT`：Tail 保持 $\mathsf{OPEN}$，到下一个 Espresso 块重试。锚点已经固定，重试不改变它。

新 session 的 $q^{\rm open}=q$，所以它的普通块最早在 $J_{q+1}$ 中被接纳。

**原子性。** 每处理完一个事件，就把状态变化、事件输出与处理位置一起持久化，然后才对外输出。重新执行时从持久化的位置继续，不会重复追加。

### 5.3 `CheckFINAL`

见 §3.3。

### 5.4 `Eligible_q(X, B)`

**输入**

- $\mathcal X$；
- 候选 DATA 块 $B$，它属于截至 $J_q$ 已发布的对象集合 $\mathcal O_q$。

**输出**

- `true` 或 `false`。

**功能。** 当 $\mathsf{Tail}=\mathsf{NEXT}(\nu)$，且以下各条同时成立时为 `true`：

- (a) $B$ 在本链 domain 下由 session $\mathrm{sid}(\nu)$ 签名，且 $\beta(\nu)$ 未关闭；
- (b) $\mathrm{parent}(B)=\mathsf H(\mathrm{last}(G))$，且 $h(B)=h(\mathrm{last}(G))+1\le H(\nu)$；
- (c) $\mathrm{Fin}_q(\nu,B)$：存在 $D$、$\Sigma$、$\pi$，其块体与签名都在 $\mathcal O_q$ 中，使 $\mathsf{CheckFINAL}(C_\nu,B,D,\Sigma,\pi)=$ `true`；
- (d) $\xi(B)$ 指向已最终化的 $J_{q'}$，且 $q^{\rm open}(\nu)\le q'<q$；
- (e) **R-EXP：** $\theta_q\le t(h(B))+W_{\rm exp}$；
- (f) **R-FRESH：** $\mathit{ts}(\lambda(B))\ge t(h(B))-\delta_{\rm ref}$。

(a)–(d) 与 Part II v4.9 相同；(e)、(f) 是修订新增的条件。v0.1 的 (g) R-STOP 随 STOP 一起删除。

### 5.5 `ResumeChain(X, L)`：RESUME 处的派生链

**输入**

- $\mathcal X$；
- RESUME 记录及其 L1 位置（所在块 $L$）；
- 派生规则的读取接口（§7.1）：已最终化的 L1 前缀与已取回的 DA 数据。

**输出**

- $(D_L,h_f,A)$；
- `WAIT(missing)`：L1 或 DA 数据尚未取得。

**功能。**

1. **派生到 $L$。** 用派生规则按 L1 顺序处理到 $L$ 中这条 RESUME 的 `CursorSet` 为止：先处理此前已接纳的全部输入，再按 override 的规则把 (派生末端, $h_{\rm res}$] 立即填成 deposit-only，填充块的 origin 不超过 $L$（G-DER-3）。结果记为 $D_L$。
2. **分叉点。** $h_f$ 为原 $G$ 中第一个不在 $D_L$ 中的块的高度：同一高度、同一原生块哈希才算在内。原 $G$ 的每个块都在 $D_L$ 中时，$h_f=\bot$。
3. **锚点。** $A:=\mathrm{last}(D_L)$。

**确定性。** $D_L$ 只取决于 $L$ 及之前的已最终化 L1 前缀与 DA 数据（DA-S1）。数据未取回时返回 `WAIT`，不返回别的结果；所以所有诚实观察者得到同一个 $(D_L,h_f,A)$，与下载的先后无关。

**$h_{\rm res}$ 的作用。** $\mathsf M$ 按操作规程选取 $h_{\rm res}$ 时（§4.2.9），$h_{\rm res}$ 高于 $L$ 处派生链的末端，所以 $A$ 恰好是派生链在 $h_{\rm res}$ 处的块，Batch Inbox 的 cursor 也恰好停在 $h_{\rm res}$。
若 $h_{\rm res}$ 不高于派生末端，$A$ 仍是派生链的真实末端，新 session 可以从那里接着出块；但 cursor 落后于 $A$，而 batcher 不再为 deposit-only 高度提交批次，新块无法接续，只能再发一次 RESUME。

### 5.6 查询

- `Chain(X)` → $(G,\sigma)$；
- `Tail(X)`；
- `Responsible(X)` → 当前负责的 session 与区域；Tail 不是 $\mathsf{NEXT}$ 时为 $\bot$。

### 5.7 保证与依赖

**保证。**

- **G-GFR-1（确定性与单调，S0、S1）：** $\mathrm{Rec}$ 是 $(\mathrm{LOG}^{\rm L1},\mathrm{LOG}^E)$ 以及 RESUME 处所读 DA 数据的确定函数，并且对日志前缀单调。〔L1-S、E-S、DA-S1；依赖 G-DER-1〕
- **G-GFR-2（只追加，S2）：** $G$ 只在末尾追加，唯一的例外是 `Resumed`：$G$ 被替换为 $D_L$，原 $G$ 在 $h_f$ 及之后的块被撤销。它发生在所有诚实观察者记录中的同一位置，替换结果也相同。
- **G-GFR-3（授权，S3(b)）：** 每个被追加的普通块都满足 `Eligible`，其中 CheckFINAL 使用负责 descriptor 的 context。
- **G-GFR-4（派生一致，S3(c)）：** 每个出处为 $\mathsf{DER}$ 的块，都等于采纳它的那次 RESUME 所用 $D_L$ 在该高度的块。GFR 不在其他地方生成 deposit-only 块。
- **G-GFR-5（公开约束，供 S6 使用）：** 每个被追加的普通块都满足 R-EXP 与 R-FRESH。
- **G-GFR-6（锚点）：** FORCE 开出的 session 以生效时的 $\mathrm{last}(G)$ 为锚点；RESUME 开出的 session 以 $\mathrm{last}(D_L)$ 为锚点。锚点在边界产生时就已固定，Plan 的 WAIT 不改变它。
- **G-GFR-7（控制定位）：** 每条控制记录在合并顺序中只有一个生效位置，且不追溯生效。〔依赖 G-AUTH-5〕
- **G-GFR-8（RESUME 与 D 对齐）：** 处理 RESUME 之后 $G=D_L$。因此 $G$ 与派生链之间的任何分歧，都在下一次 RESUME 处消除。〔依赖 G-DER-1、G-DER-3〕
- **G-GFR-9（活性）：** L1-L 成立且 Espresso 可用时，键不超过最新 Espresso 块时间戳的事件都会在 $T_E+T_{\rm L1}$ 内被处理；RESUME 处另需此前已接纳批次的数据可以取回。在 Part 0 DG6 的条件下，$G$ 持续增长。

**依赖。** L1 Authority（G-AUTH-1、2、5、9）；Session Program（G-PROG-2，经 CheckFINAL 用于 S3）；Derivation Rule（G-DER-1、3，用于 RESUME）；A-L1、A-ESP、A-DA。

G-GFR-1 与 G-GFR-2 只依赖 L1 Authority 状态的确定性（L1-S4）与派生规则的确定性（G-DER-1，DA-S1）。这正是 Part 0 中 DG1 的依赖。

### 5.8 与 Part II v4.9、组合式抽象 v2 及本稿 v0.1 的差异

- **保留：** v4.9 的两步（Force 前奏，然后追加合格的普通块），成为“处理 Espresso 块”的第 1、3 步；Eligible 的 (a)–(d)；CheckFINAL；常规后继的选择。
- **新增：** 合并顺序与 R-WAIT；Force 与 RESUME 两类 L1 事件；时间事件 Expire 的“过期停住”；Eligible 的 (e)、(f)；RESUME 以 $D_L$ 替换 $G$（`ResumeChain`）；Tail 的 $\mathsf{OPEN}$ 与 $\mathsf{HALT}$。
- **相对 v0.1 删除：** DepFill、R-DEP、R-OVR、R-ANCHOR（`AnchorHeight`、$h_{\rm det}$）、R-FB（截止事件与 `Replaced`）、STOP（`StopEffective`、R-STOP、`StopState`）、`CursorSet` 事件。
- **相对 v2 删除：** 边界策略 $\mathfrak B$、$\mathsf{Frames}_{\mathfrak B}$、$K_S$ 与 $K_R$、ModeApplied、转移 S/D/R、DEPOSIT_ONLY 模式与 Tail 的 $\mathsf{DEPOSIT}$；v4.9 的“每个 Espresso 块至多一个更新激活”假设。

---

## 6. 恢复中各模块的分工

本节把修订记录 §6 的恢复流程，写成“谁在什么时候调用哪个接口”。例子与时间线见[以 D 为准的恢复](d_anchored_recovery_zh.md) §8。

### 6.1 区域停摆（Espresso 正常）

| 步骤 | 触发 | 模块与接口 | 结果 |
| --- | --- | --- | --- |
| 1 | 区域 A 不再出块 | — | 负责 session $s_A$ 的高度没有普通块，$G$ 停在 $\mathrm{last}(G)$ |
| 2 | 治理方在负责高度过期之前决定 Force | `AUTH.SubmitForce(U)` | `ForceRecorded`；新 policy 排除区域 A |
| 3 | 第一个满足 $\theta_q\ge\theta_a$ 的 Espresso 块 | `GFR.AdvanceGlobal` | `BranchClosed(β, FORCE)`；锚点为 $\mathrm{last}(G)$；`SessionAssigned(ν_B)`，cause 为 FORCE |
| 4 | 收到生命周期事件 | 各节点的 `MGR.ApplyLifecycle`；B 的 `InstallHandoff` → `PrepareAndEnter` | 关闭 A 的 Program；B 的 Program 进入，从 $\mathrm{last}(G)+1$ 追赶缺口，不需要 deposit-only |

若在 $W_{\rm exp}$ 内没有 Force：负责高度过期时，`GFR.AdvanceGlobal` 输出 `BranchClosed(β, EXPIRED)` 与 `Halted`，$G$ 停住；之后按 §6.4 由 RESUME 恢复。

### 6.2 Espresso 停机

| 步骤 | 触发 | 模块与接口 | 结果 |
| --- | --- | --- | --- |
| 1 | Espresso 停机 | `GFR.AdvanceGlobal` 返回 `WAIT` | $G$ 暂停 |
| 2 | 停机期间 | `BAT.Deliver`（$G$ 中已有的块）；`DER.Derive`（窗口过期回退） | L1 派生链继续推进，强制包含照常有效 |
| 3a | 短停机：Espresso 在负责高度过期前恢复 | `PUB.Offer` 重试；`GFR.AdvanceGlobal` | 原 session 的积压块照常进入 $G$，原 session 追上停机期间的高度，无损 |
| 3b | 长停机：某高度的 sequencing window 结束 | 各节点的 `MGR.Kill`（派生视图，或 cursor 超窗停滞） | 停机期间就关闭全部旧 session；GFR 仍在等待 Espresso |
| 4b | Espresso 恢复 | `GFR.AdvanceGlobal` | 处理到第一个过期高度：`BranchClosed(β, EXPIRED)`，`Halted` |
| 5b | 诚实的 $\mathsf M$ 按操作规程发起恢复 | 按 §6.4 | $G$ 被替换为 $D_L$，新分支出块 |

### 6.3 批次层故障（batcher、DA 或 Certifier 超出交付预算）

| 步骤 | 触发 | 模块与接口 | 结果 |
| --- | --- | --- | --- |
| 1 | 批次停止被接纳 | — | $G$ 照常增长；Batch Inbox 的 cursor 停在 $G'$ 的末端 |
| 2a | 在 sequencing window 内恢复 | `BAT.Deliver` 从 cursor 接着补交 | 每个高度都在截止前被接纳，$G$ 的块原样进入 $D$，无损 |
| 2b | 第一个未交付高度的窗口结束 | `DER.Derive` | 派生在该高度补 deposit-only，$D$ 与 $G$ 分叉 |
| 3b | 派生到分叉点，或看到 cursor 超窗停滞 | 各节点的 `MGR.Kill` | 关闭全部旧 session；$G$ 停止增长，分叉点之后的 $G$ 内容将在 RESUME 时被撤销 |
| 4b | 批次层恢复后，诚实的 $\mathsf M$ 发起恢复 | 按 §6.4 | $G$ 被替换为 $D_L$，新分支出块，batcher 从 $h_{\rm res}$ 接着提交 |

### 6.4 RESUME（6.1–6.3 共用的恢复步骤）

| 步骤 | 触发 | 模块与接口 | 结果 |
| --- | --- | --- | --- |
| 1 | 派生链已越过或偏离全局链，故障已恢复 | `AUTH.SubmitResume(c, P_new, h)` | `ResumeRecorded`；同一交易内 `BIN.Override` 把 cursor 移到 $h$，产生 `CursorSet` |
| 2 | 交易所在的 L1 块 $L$ 最终化 | `DER.Derive` | 先处理此前已接纳的输入，再把 (派生末端, $h$] 立即填成 deposit-only（origin 不超过 $L$）；得到 $D_L$ |
| 3 | 键 $\mathit{ts}(L)+M_a$ 满足 R-WAIT | `GFR.AdvanceGlobal`（`ResumeChain`） | 必要时 `BranchClosed(β, RESUME)`；$G:=D_L$；`Resumed(c, β+1, h_f, A)`，$A=\mathrm{last}(D_L)$ |
| 4 | 下一个 Espresso 块 | `GFR.AdvanceGlobal` | Plan 选出区域；`SessionAssigned(ν)`，cause 为 RESUME |
| 5 | 收到生命周期事件 | 被选区域的 `MGR.ApplyLifecycle` → `InstallHandoff` → `PrepareAndEnter` | 用派生视图准备 $\mathit{st}(A)$；新 Program 进入，在 $t(h+1)$ 提出第一块 |
| 6 | 新的普通块进入 $G$ | `BAT.BuildPayload` → `BAT.Deliver` → `CERT.CertifyBatch` → `BIN.SubmitBatch` | 以 override 写入的 cursor 为 prev 接着提交；$\mathsf C$ 的 gpos 不早于这次 override |

所需的 DA 数据取不回时，第 2、3 步都停下等待，直到数据可得。这是上游规格本身的硬边界（§9）。

### 6.5 治理 Force（正常情形）

| 步骤 | 触发 | 模块与接口 | 结果 |
| --- | --- | --- | --- |
| 1 | 治理方决定轮换或更改 policy | `AUTH.SubmitForce(U)` | `ForceRecorded`；合约已检查提前量与格式 |
| 2 | 第一个满足 $\theta_q\ge\theta_a$ 的 Espresso 块 | `GFR.AdvanceGlobal` | `BranchClosed(β, FORCE)`；锚点为 $\mathrm{last}(G)$；`SessionAssigned(ν)`，cause 为 FORCE |
| 3 | 收到生命周期事件 | 各节点的 `MGR.ApplyLifecycle` | 关闭旧 Program；被选区域安装新 handoff 并进入 |

---

## 7. Derivation Rule（DER）

**职责。**
从 L1 日志与 DA 数据派生原生链 $D$，即 L1 派生链。它沿用 mega-espresso 的派生（方案 A），包括：Batch Inbox 接纳的批次、sequencing window 过期回退、Δ5 同高度替换、Δ6 执行失败替换、`CursorSet` 的填充，以及 DA 缺失时等待。状态结算使用同一个派生；GFR 在 RESUME 处、Session Manager 在派生视图中也使用它。

### 7.1 `Derive(D, LOG_L1, DAobjects)`

**输入**

- $\mathcal D$：前一次的派生状态，或一个检查点；
- L1 日志：`CursorAdvanced`、`CursorSet`、Deposit Portal 的 deposit、系统配置；
- 从已接纳的承诺取回的 DA 数据 $(d,\mathit{payload},\pi_{\rm DA})$，可由派生节点自己按需读取。

**输出**

- 新派生的原生块、执行后的状态与新的派生状态 $\mathcal D'$；
- `WAIT(missing, position)`：缺 L1、DA 或前置状态时停在明确的位置；已完成的部分不撤销。

**功能。**

- 按 L1 顺序处理输入：对普通批次，核对 DA 字节后解码，在实际父块上检查并执行；没有可用批次的高度，在窗口过期后按原生规则填充 deposit-only；遇到 `CursorSet` 时，先处理此前的输入，再把 (派生末端, $h_{\rm res}$] 立即填成 deposit-only。
- 派生不重新验证 Espresso，也不重新验证 Certifier 的结论；它通过 Batch Inbox 的接纳来承接这些结论。
- 已接纳但取不回的 payload 使派生在该处等待，绝不以 deposit-only 代替。

**保证。**

- **G-DER-1（确定性与单调，L2-S2）。** 〔L1-S、DA-S1〕
- **G-DER-2（内容或 deposit-only，L2-S1）：** 每个高度要么是 $G$ 中普通块的普通交易列表，要么是一个 deposit-only 块。〔依赖 G-CERT-1、G-BIN-1〕
- **G-DER-3（override 填充）：** 处理 `CursorSet` 时，此前已派生的前缀保持不变；若派生末端低于 $h_{\rm res}$，(派生末端, $h_{\rm res}$] 被立即填成 deposit-only，填充块的 origin 不超过 `CursorSet` 所在的 L1 块；若不低于 $h_{\rm res}$，只重置未完成的 channel。〔沿用 mega-espresso 的 override 派生规则〕
- **G-DER-4（强制包含，L2-L2）：** deposit 在约 $W_{\rm seq}+T_{\rm L1}$ 内进入 $D$。〔L1-L；此前已接纳批次的数据可取回〕
- **G-DER-5（DA 缺失时等待）：** 不会因取不回数据而替换已接纳的内容。
- **G-DER-6（与 DerivOK 一致，核对项）：** 若块 $x$ 在父块 $P$ 上满足 DerivOK（§3.2），$P$ 是派生的实际父块，且 $x$ 的批次在截止前被接纳、数据可取回，则派生在高度 $h(P)+1$ 得到的块就是 $x$ 对应的原生块。这是目标 T 的归纳步（§8.4），在 Part 6 逐条核对。

**依赖。** Batch Inbox（G-BIN-1、3）；Certifier（G-CERT-1）；Deposit Portal；A-DA、A-L1。

---

## 8. 通信、职责与组合检查

### 8.1 通信归属

| 从 → 到 | 内容 | 途径 | 时限与作用 |
| --- | --- | --- | --- |
| 用户 → Block Builder | 普通交易 | 直接发送，由客户端或 RPC 网关路由到负责区域 | 不保证包含；L-tx 的输入 |
| 用户 → Deposit Portal | 强制包含交易 | L1 | $T_{\rm L1}$；L-fi 与 L2-L2 的输入 |
| 质押者、证据提交者 → L1 Authority | 质押操作、quit 与 offense 证据 | L1 | $T_{\rm L1}$ |
| 治理方 → L1 Authority | Force、RESUME | L1 | $T_{\rm L1}$；合约检查授权与格式，RESUME 的时机与 $h$ 由诚实的 $\mathsf M$ 决定（A-GOV） |
| Manager ↔ 本机 Program、Builder、Publisher | 生命周期、候选、引用、关闭、程序事件 | 节点内调用 | 先持久化，后发送 |
| Program ↔ 同 session 的其他成员 | 共识消息、ReadyBundle | 区域网络 | $\Delta_i$；区域安全的前提 |
| Manager ↔ 其他区域的 Manager | 区域结果、末块结果、ReadyBundle | 跨区域网络 | $\Delta_{\rm cross}$；只影响活性 |
| Publisher → Espresso | 完整区域结果 | Espresso | $T_{\rm pub}+T_E$ |
| L1、Espresso → GFR | 两本日志 | 每个使用方自己读取 | 所有决定的来源 |
| DA → GFR（经派生规则） | RESUME 处计算 $D_L$ 所需的已接纳批次数据 | 每个使用方自己读取 | 只在 RESUME 时；取不到时等待 |
| Batcher ↔ DA、Certifier；Batcher → Batch Inbox | payload、证书 $\chi$、批次提交 | DA、直接请求、L1 | $T_{\rm batch}$ |
| L1 Authority → Batch Inbox | cursor override | 同一笔 L1 交易内的合约调用 | 只在 RESUME 中 |
| L1、DA → Derivation Rule | 接纳事件、deposit、payload | 每个派生节点自己读取 | 得到 L1 派生链 $D$ |

节点之间的直接消息（区域网络、跨区域网络）只用来加快进度：$G$ 的内容只由两本账本的日志决定，RESUME 处另由公开的 DA 数据决定。

### 8.2 跨模块义务

下列义务各涉及至少两个模块。每一条都必须在相关模块的 Part 中分别兑现，组合论证才能成立。

- **O-1（规则同版本）：** §3.6 表中的每条规则，所有使用方使用同一个版本。
- **O-2（常数一致）：** §2.2 表中的每个常数，所有使用方取同一个值。尤其是：
  - $M_a$：L1 Authority 的 Force 检查，与 GFR 的事件键和 R-WAIT；
  - $W_{\rm exp}$、$\delta_{\rm ref}$：L1 Authority 的释放公式、GFR 的 R-EXP 与 R-FRESH、Program 的生产约束与 DerivOK；
  - $t_{\rm gen}$、$h_{\rm gen}$、$b_t$：所有用到 $t(h)$ 的地方；
  - $W_{\rm seq}$、$D_{\rm drift}$：DerivOK（BLD、PROG）、Kill 的 cursor 超窗判断（MGR），与 Derivation Rule 的窗口和 drift 规则；
  - $\mathrm{TB}$：L1 Authority 的 plan 与释放公式、Program 的配额、GFR 的 $H(\nu)$。
- **O-3（cid 绑定）：** Program 的每个签名都绑定 $\mathrm{cid}_s$；L1 Authority 的相对验证与 GFR 的 CheckFINAL 都在同一个 $\mathrm{cid}$ 下检查；Manager 从公开历史推出 $C$，不接受发送方声称的 context。
- **O-4（RESUME 一致）：** L1 Authority 在 RESUME 的同一交易内执行 override；Derivation Rule 按 `CursorSet` 填充（G-DER-3）；GFR、Session Manager 与 Certifier 使用同一个 $D_L$，即同一个 $L$、同一个派生规则版本。
- **O-5（认证切面）：** Certifier 的 gpos 不早于 prev 被写入的 L1 位置（§4.7.1）。因此 $G'\preceq G$ 只在 RESUME 前后各自成立：override 之前签发的证书不再被接纳，之后签发的证书只覆盖新 $G$。
- **O-6（关闭传播）：** GFR 的关闭事件或本地的 Kill → Manager 持久化关闭标记 → `PROG.Close`；Program 在每个 BFT 动作之前都检查关闭标记。
- **O-7（派生一致投票）：** BLD 与 PROG 使用的 DerivOK，与 Derivation Rule 的批次检查、drift 规则与 deposit 规则逐条一致，并对 L1 视图的扩展稳健（G-DER-6）。
- **O-8（证据链）：** Program 输出 Quit 或 Offense → Manager 持久化并提交 → L1 Authority 产生 REC 或执行 slash → `AUTH.Auth` 给出新一代授权 → Manager 经 `PROG.Provide` 交给 Program。
- **O-9（负责性视图）：** GFR 的 Tail 与常规后继的 plan → `MGR.Responsibility` → `BLD.SubmitTx` 的接受条件。
- **O-10（原生规则同版本）：** Exec、DerivOK、Derivation Rule 与状态结算使用同一个原生 rule version，包括 deposit 的包含规则与 sequencing window 规则。

### 8.3 Part 0 目标与模块保证的对应

下表把 Part 0 的每个设计目标，追溯到具体的模块保证与 Part 0 假设。

| Part 0 目标 | 模块保证 | Part 0 假设 |
| --- | --- | --- |
| S0、S1、S2（DG1） | G-GFR-1、2、7；G-AUTH-5；G-PUB-1；G-DER-1 | L1-S、E-S、DA-S1 |
| S3 有效性（DG2） | G-GFR-3、4；G-PROG-2、3；G-BLD-1、2；G-MGR-1；G-AUTH-1 至 3；G-DER-1 | A-CRYPTO、A-BYZ |
| S4 快速确认一致（DG3） | G-PROG-1、2、4；G-MGR-2、3；G-GFR-3；G-AUTH-3、4 | $s$ 良态（含 $\Delta_i$）、A-CRYPTO |
| S5 问责（DG4） | G-AUTH-6；G-PROG-5；G-MGR-5 | A-CRYPTO、A-PKI、持久化存储 |
| 证据在释放前处理完毕（修订记录 §6.4 的 $T_{\rm slash}\le T_{\rm ev}$） | G-PROG-8；G-MGR-5；G-AUTH-10 | L1-L、A-RELAY |
| S6 质押安全（DG5） | G-AUTH-7、8；G-GFR-5 | A-TS、L1-S、E-S |
| L-tx（DG6） | G-BLD-3；G-MGR-4、6、8；G-PROG-6、7；G-PUB-2、3；G-GFR-9 | DG6 的条件、A-LEAD、A-RELAY |
| L-fi（DG6） | G-BLD-2；G-PROG-7；G-GFR-9；Deposit Portal | DG6 的条件 |
| DG7 区域故障 | $W_{\rm exp}$ 内：G-AUTH-5、10，G-GFR-6、7，G-MGR-1、6；超过：同 DG8 的长停机 | DG6 的条件（对新 session）；A-GOV |
| DG8 Espresso 停机 | 短停机：G-GFR-9，G-PROG-7，G-PUB-3；长停机：G-MGR-7，G-AUTH-9，G-BIN-2，G-DER-3，G-GFR-6、8，G-MGR-6 | Espresso 恢复；E-S；A-GOV（长停机） |
| DG9 批次层故障 | 窗口内：G-BAT-3；超过：G-MGR-7，G-AUTH-9，G-BIN-2，G-DER-3，G-GFR-6、8，G-BAT-2，G-CERT-2 | 批次层恢复；A-GOV（超过窗口） |
| L2-S1 | G-BAT-1；G-CERT-1、2；G-BIN-1；G-DER-2、5 | A-CERT、DA-S1、L1-S、A-BYZ |
| L2-S2 | G-DER-1；G-BIN-3 | L1-S、DA-S1 |
| L2-S3 | G-GFR-8；G-BAT-3；G-CERT-2 | 交付预算 |
| L2-L1（目标 T） | G-BLD-2；G-PROG-2；G-GFR-5、6；G-BAT-3；G-CERT-1、3；G-BIN-4；G-DER-6 | A-BYZ、Certifier 在线、DA-L、L1-L、A-RELAY、交付预算、A-GOV |
| L2-L2 | G-DER-4、5；Deposit Portal | L1-L；此前已接纳批次的数据可取回 |

**完整性检查。** Part 0 的每个目标都至少由一条模块保证支撑；本稿的每条模块保证都至少被一个目标用到。没有无人负责的目标，也没有无人使用的保证。

### 8.4 组合论证的形态

每个模块的保证，在它自己的 Part 中证明。证明可以使用两类前提：本模块“依赖”一栏所列的 Part 0 假设，以及其他模块的保证。这是 assume–guarantee 的组合方式，不需要构造 UC simulator。

**依赖关系中有一个环：**

- Session Program 的 G-PROG-2（FINAL 可靠）依赖 Manager 的 G-MGR-1（handoff 真实）；
- G-MGR-1 对 REGULAR handoff，依赖前驱 session 的 G-PROG-2；对其他 cause，依赖 GFR 的输出；
- GFR 的 G-GFR-3（授权）又依赖 G-PROG-2。

**打破环的方法：** 沿 GFR 的处理位置与 session 的先后做归纳。

- **基础：** genesis 检查点。G-GFR-1、2 不依赖任何其他模块，在任何位置都成立。
- **归纳步：** 假设处理位置 $m$ 之前开出的所有 session 都满足 G-PROG-2，并且 GFR 到位置 $m$ 为止的输出都满足 G-GFR-3、4。那么在位置 $m$ 开出的 session，其 handoff 是真实的（G-MGR-1），于是它也满足 G-PROG-2；从而位置 $m+1$ 处的追加满足 G-GFR-3。
- **良基性：** 每个 session 的 handoff 只引用更早的 session（常规后继引用其前驱）或更早的处理位置（FORCE、RESUME 引用开出它的位置），所以归纳是良基的。RESUME 的锚点 $\mathrm{last}(D_L)$ 由派生规则确定（G-DER-1），不依赖任何 session 的 G-PROG-2，相当于一个新的基础情形。

**目标 T（Part 0 的 L2-L1）也按同样的方式归纳，对象是高度：**

- **基础：** 每个 session 的锚点都在 $D$ 中。genesis 如此；RESUME 的锚点就是 $\mathrm{last}(D_L)$；FORCE 与常规交接的锚点是 $G$ 中更早的块，按归纳已在 $D$ 中。
- **归纳步：** 设 $G$ 与 $D$ 在高度 $h-1$ 处是同一个块。$G[h]$ 满足 DerivOK（G-PROG-2、G-BLD-2）；由 R-EXP（G-GFR-5）与批次层活性（G-BAT-3、G-CERT-1），它的批次在截止前被接纳、数据可取回；由 G-DER-6，派生在同一父块上得到同一个块。
- **前提：** 期间没有 RESUME 越过 $h$，这由 A-GOV 保证。

### 8.5 与组合式抽象接口的对照

| 组合式抽象（v1/v2） | Part 1 | 变化 |
| --- | --- | --- |
| §3.5 Execute | §3.1 Exec | 改为共享规则 |
| — | §3.2 DerivOK | 新增（v0.1 的 DepFill 已删除） |
| §4.1 SubmitTx | §4.1.1 SubmitTx | 只在本区域负责时接受；不转发（R19） |
| §4.1 Build、ValidateCandidate | §4.1.2–§4.1.3 | 改用 DerivOK 检查（R25） |
| §4.2 Join、Cancel、RequestExit、CutSnapshot、ReadAuthority、Plan、Auth、Leader、SubmitOffense、SubmitQuit | §4.2 | 不变；Plan 的参考块可以是 DER 块 |
| §4.2.11 SubmitUpdate | §4.2.8 SubmitForce | 改名；合约检查提前量与格式（R4） |
| §4.2.12 ScheduleMode | §4.2.9 SubmitResume | RESUME 由 $\mathsf M$ 给出 $h$ 并执行 override（R17、R22）；v0.1 的 SubmitStop 已删除（R23） |
| §4.2.13 ProveClosure | — | 删除（R13） |
| §4.2.14 Release | §4.2.5 Release | 改用时间上界（R13） |
| §4.3 ObservePublic、ObserveRegional、InstallHandoff、PrepareAndEnter、HandleProgramEvent | §4.3.1–§4.3.5 | 新的 handoff cause；派生视图与 RESUME 锚点；Conf 事件 |
| §4.3.6 ApplyControl | §4.3.6 ApplyLifecycle | 改名；处理全部生命周期事件 |
| — | §4.3.7 Responsibility | 新增，供 Builder 使用 |
| — | §4.3.8 Kill | 新增：本地关闭已不可能进入 $G$ 与 $D$ 的 session（R26） |
| §4.4 Create、Enter、Provide、Step、Close | §4.4.1–§4.4.5 | 输出事件增加 Conf，Offense 单列；关闭原因增加 KILL |
| §4.5 Offer | §4.5.1 Offer | 不变 |
| §4.6.1 BuildPayload | §4.6.1 BuildPayload | 只提交普通块（R27） |
| §4.6.2 SubmitBatch | §4.6.3 SubmitBatch | 不变 |
| — | §4.6.2 Deliver、§4.6.4 Override、§4.6.5 ReadCursor | 新列出；Override 只能经由 RESUME |
| §4.6.3 ApplySettlementBoundary | — | 删除，由 RESUME 的 override 取代 |
| §4.6.4 SubmitStateClaim | §4.8.3 SubmitStateClaim | 移到原生组件，不变 |
| §4.7.1 CertifyBatch | §4.7.1 CertifyBatch | 以 $G$ 为准，只认证普通块；gpos 不早于 prev 的写入位置（R15、R27） |
| §4.7.2 ConfirmControl | — | 删除（R15） |
| §5.1 CheckFINAL | §3.3 CheckFINAL | 不变 |
| §5.2 AdvanceGlobal；v2 §5.3–§5.14 | §5.2–§5.6 | 合并顺序版本；RESUME 以 $D_L$ 替换 $G$（`ResumeChain`）；v2 的模式状态机删除 |
| §6 模式扩展 | §6 恢复中各模块的分工 | 由恢复流程取代 |
| §7.1 Derive | §7.1 Derive | 不变；写明 override 填充与 DerivOK 的一致性 |
| — | §4.8.1–§4.8.2 Deposit Portal | 新列出（原生组件） |

---

## 9. 待定问题

1. **D12：** 授权 L1 引用 $\lambda(B)$ 是否与原生 origin 统一。目前暂定 DER 块的 $\lambda$ 取其 origin（影响 §2.3 与 Plan）。
2. **Kill 判据 (b) 的含义（请确认）：** 本稿按“$G$ 领先 $D$ 时，$s$ 还必须从 $\mathrm{last}(G)$ 派生”理解（§4.3.8）。
3. **核对项（Part 6）：** DerivOK 与 mega-espresso 的 Δ5 转换条件、Holocene 的执行替换条件、上游 drift 例外逐条对齐（G-DER-6）；cursor 超窗停滞判断的精确边界（§3.5）。
4. **Certifier 与 cursor（Part 6）：** cursor 中 GFR 位置字段的编码；RESUME 写入的完整 cursor（位置、epoch_num、quorum 摘要）怎样由合约构造；$\mathsf C$ 的 gpos 规则怎样落到签名格式。
5. **RESUME 处数据取不回（硬边界）：** 已接纳批次的数据取不回时，派生与 GFR 都停在 RESUME 处等待。这是上游规格本身的限制，本稿不另设出口。
6. **手动 override 的场景（Part 6）：** mega-espresso 中依赖 admin override 的场景（publisher anchor 超前、S5、S6、S8），在轮换设计下是否仍会出现，以及只有 RESUME 一个入口时如何处理。
7. **推导时限（各模块的 Part）：** $T_{\rm pos}$、$T_{\rm sess}$、$T_{\rm pub}$、$T_{\rm batch}$、$T_{\rm live}$、$T_{\rm fi}$ 等的具体表达式。
8. **快速确认的交付：** 如何把 `Conf` 交给客户端（RPC 接口）属于实现范围，不影响协议保证。

v0.1 中的 D13（$W_{\rm exp}$ 的取值）已定为约 10 h；D15（STOP 自动失效）随 STOP 删除；批次失败回退之后的续接由 RESUME 统一处理，不再单列。
