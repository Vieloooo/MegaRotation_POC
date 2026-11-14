# 有界时钟偏差 $\Delta_G$ 下的安全审查（v0.1）

日期：2026-11-07。对象：本目录当前规格（00–08 v0.1，09、10 v0.2）在 [时钟模型替换 v0.1](clock_skew_zh_v0_1.md)（下称“替换稿”）之下的行为。替换稿 §1.2 定义了 C1–C5、$\Delta_G$ 与 $\delta_G$，§1.3 给出换算引理 L-CLK，本文直接引用。

条目编号为 K1–K10，与 [安全审查 v0.2](security_review_zh_v0_2.md) 的 S1–S27 互相独立；与某个 S 条目重叠时注明关系。优先级与状态的用法同 v0.2 §1。所有修复都是建议，等你决定。

## 1. 不受影响的安全性质

在 C1–C5 之内，下列性质的证明都不读本地时钟，因此不变。

| 性质 | 不依赖时钟的原因 |
| --- | --- |
| BFT-1 agreement | 在异步、进入时间不同、部分离线时都要求成立（[02 §4.2](02_session_program_zh_v0_1.md)）；区域 agreement 不依赖延迟界（[00 §2.4](00_arch_zh_v0_1.md)） |
| BFT-3、BFT-4（每个 view 一个 block；context 隔离） | 由签名与 descriptor 决定 |
| CheckFINAL、CheckEvidence | 只用声明的公开输入，观察者不需要同步时钟（02 第 330 行） |
| GFR 重放、$G$ 的确定性 | 事件键只取 $\mathit{ts}_{\rm L1}$、$\theta_q$ 与公开参数；v0.2 §13.2 的确定性结论成立 |
| Derivation $D$、Authority 检查、PUB-1…3 | 只读公开数据 |

**模型之外同样成立。** 时钟误差超出 $\delta_G$ 时，上表仍然成立，因为证明里根本没有时钟。v9 中没有任何安全性质以时钟为前提。时钟误差能破坏的只有：

- 时间性质 G1.2、G1.3、G3.2；
- BFT-7 的真实时间读法；
- 活性预算。

这把 v0.2 §13.2 第 590 行的判断（“少数诚实副本的时钟漂移只影响活性”）推广到了任意时钟误差，但需要补一句：慢时钟会产生由 honest 投票组成的迟到证书（K6、K8）。

**变成有界放宽的承诺。**

| 性质 | 偏差下的形式 |
| --- | --- |
| BFT-7 的真实时间读法 | 真实时刻 $\tau_s^{\rm stop}+\delta_G$ 之后没有 honest 签名 |
| G1.2 | 等待修法下为 $+\delta_G$ |
| G1.3、G3.2 | 各加 $\delta_G$ |
| PUB-4 推论 | 需要 $T_{\rm grace}$ 多留 $\delta_G$ |

推导见替换稿 §3.2–§3.4。

## 2. 条目总览

| 编号 | 标题 | 优先级 | 状态 | 关系 |
| --- | --- | --- | --- | --- |
| K1 | 慢时钟 honest replica 永久拒绝快时钟 honest leader 的 proposal | 高 | 确认 | 加重 S20(a) |
| K2 | $T_{\rm win}$ 没有计入 $\Delta_G$，快时钟 receiver 拒绝末尾的 honest proposal | 中 | 确认 | — |
| K3 | Not ahead 放宽：block timestamp 可以领先真实时间 $\delta_G$ | 中 | 确认 | — |
| K4 | 停止时刻分散，使窗口末尾失去进展保证，变体 1a 成为常态 | 中 | 确认 | 加重 S20(b)、S10 |
| K5 | 交接重叠 $\Delta_G$：后继在过时的候选上出块，诚实交接也会作废工作 | 中 | 确认 | S27 的无敌手版本 |
| K6 | S10 的期权窗口与 S27 的代价按 $\delta_G$、$\Delta_G$ 放大 | 中 | 条件性 | S10、S27 |
| K7 | 偏差加宽 Byzantine leader 分裂投票的时间带 | 中 | 条件性 | S20 |
| K8 | 超出 $\delta_G$ 的时钟攻击 | 高 | 模型外 | v0.2 §13.2 |
| K9 | 与 Espresso $\Delta_{\rm ts}$ 的共同故障模式 | 低 | 条件性 | S14、S26 |
| K10 | Manager 与 Program 不同源时的停止不一致 | 低 | 条件性 | — |

## 3. 条目

### K1：慢时钟 honest replica 永久拒绝快时钟 honest leader 的 proposal

**高优先级；确认；置信度高。**

**触发能力。** 不需要敌手。只要 honest leader 的时钟领先某些 honest receiver 超过 $T_{\rm build}+d$ 就会发生，其中 $d$ 是那一次的实际网络延迟（不是上界 $\Delta_i$）。

**位置。** [02 §3.8](02_session_program_zh_v0_1.md) 第 215 行的下界 $\mathit{ts}^*(B.h)\le\tau_{\rm recv}$；第 117、220 行：重试保留首次接收时刻。

**最小轨迹。** 区域委员会 4 个等权成员 $m_1,\dots,m_4$，全部 honest，$F_s=1$，$Q_s=3$。取 $\Delta_G=200$ ms、$T_{\rm build}=50$ ms、$d=50$ ms。$m_1$ 快 100 ms，$m_3$、$m_4$ 慢 100 ms。

1. $m_1$ 是 height 36 000（$\mathit{ts}^*=$ 10:00:00）的 leader，在本地 10:00:00（真实 09:59:59.900）开始 Build，真实 09:59:59.950 发出。
2. $m_3$、$m_4$ 在真实 10:00:00.000 收到，本地读数为 09:59:59.900，低于 $\mathit{ts}^*$，Accept 返回 REJECT。
3. 首次接收时刻不能覆写，所以这次拒绝是永久的。只有 $m_1$、$m_2$ 投票，2 < 3，这个 view 失败。
4. 每个由快时钟 leader 主持的 view 都会这样失败。若 BFT 在后续 view 重新提议已锁定的同一 block（Tendermint 式），$m_3$、$m_4$ 会一直拒绝它，这就是 S20(a) 的死锁，而且此处不需要敌手。

**影响。** 有系统性的活性损失，与 leader 的时钟相关。规格本身没有给这种情况留任何余量：只要 $\Delta_G>T_{\rm build}+d_{\min}$ 就可能触发。时钟同步良好时（同区域内毫秒级）很少发生，但跨可用区部署或 NTP 退化时会出现。

**建议。** 采用替换稿 §3.2(c) 的等待修法：过早界 $\mathit{ts}^*(B.h)-\Delta_G$ 之内的 proposal 返回 WAIT(time)，超出才 REJECT。或采用放宽或滞后修法，取舍见替换稿附录 A。

**人工确认：** [ ] 等待、放宽、滞后三种修法中选哪一种；[ ] BFT 内核是否允许 WAIT(time)。

### K2：$T_{\rm win}$ 没有计入 $\Delta_G$

**中优先级；确认。**

**位置。** 02 §4.6 第 347 行：$T_{\rm win}\ge T_{\rm build}+\Delta_{i_s}$。

**轨迹。** 慢时钟 leader 在本地 slot 的末尾开始 Build（例如等 L1 输入或 parent 决定后才开始）。快时钟 receiver 收到时，本地读数可达 $\mathit{ts}^*(h+1)+T_{\rm build}+\Delta_i+\Delta_G$，超出现行 $T_{\rm win}$ 的下界，被判 REJECT，而且同样是永久的。

**建议。** 改为 $T_{\rm win}\ge T_{\rm build}+\Delta_{i_s}+\Delta_G$（替换稿 §3.2(e)）。

### K3：Not ahead 放宽

**中优先级；确认。**

**触发能力。** 一个 Byzantine leader，或者只是一个快时钟的 honest leader。

**轨迹。** 采用等待修法时，block 进入 BFT chain 的条件是：某个 honest replica 的本地时钟已经到达 $\mathit{ts}^*(B.h)$。时钟最快的 honest replica 在真实时刻 $\mathit{ts}^*(B.h)-\delta_G$ 就满足这一条。

**影响。**

- $\mathit{ts}^*(h)$ 可以比 block 被区域决定的真实时刻晚至多约 $\delta_G$，即 block 在“未来”的时间戳下被确认。
- 依赖 `block.timestamp` 的合约（拍卖截止、解锁、TWAP 窗口）可能提前至多 $\delta_G$ 执行；预确认用户看到的时间戳也领先真实时间。
- Leader 无法越过这个界，因为它受 honest 接受的约束。采用放宽修法时界变为 $\Delta_G+\delta_G$。

**建议。** 产品语义中写明 $\mathit{ts}^*$ 的精度为 $\pm\delta_G$。部署时取 $\delta_G\ll b_{\rm L2}$。

### K4：停止时刻分散导致窗口末尾失去进展保证

**中优先级；确认。**

**位置。** BFT-7（02 第 388 行）；§4.6 的 $\tau_{\rm end}$；GFR SessionEnd（[09 §4.3](09_finality_rules_zh_v0_2.md)）。

**触发能力。** 不需要敌手；最坏的时钟分布就够了。若 $h_{\max}$ 的 leader 是 Byzantine，它可以选择最不利的提议时刻。

**轨迹。** 取 $b_{\rm L2}=1$ s、$T_{\rm build}+T_{\rm fin}=600$ ms、$\Delta_G=500$ ms。

1. 完美时钟下，$h_{\max}$ 可以决定，因为 600 ms < 1 s。
2. 有偏差时，$h_{\max}$ 的 leader 是慢时钟，而 quorum 需要快时钟 replicas 的签名。快时钟 replicas 在 leader 开始 Build 后 $1\,{\rm s}-\Delta_G=500$ ms 就到达本地 $\tau_s^{\rm stop}$，停止签名。
3. $h_{\max}$ 于是无法决定。SessionEnd 在 $\theta_s^{\rm end}$ 固定 regular anchor，交接走变体 1a：后继开头约 $T_{\rm grace}$（2 min）的 block 的全局确认被推迟。
4. 只要时钟分布不变，每次交接都重复。

**影响。** 与完美时钟相比，窗口末尾多出约 $\Delta_G/b_{\rm L2}$ 个没有进展保证的 heights。$h_{\max}$ 可决定的条件由 $T_{\rm build}+T_{\rm fin}<b_{\rm L2}$ 变为 $\Delta_G+T_{\rm build}+T_{\rm fin}<b_{\rm L2}$。与 S20(b)（链式提交的 BFT 永远决定不了末尾高度）叠加后，1a 每小时都会发生，S10 的窗口每小时都打开。

**建议。**

- (a) 部署时要求 $\Delta_G+T_{\rm build}+T_{\rm fin}<b_{\rm L2}$，并留出余量。
- (b) 采纳 S10(b)，由后继的第一份结果固定前驱的终止 anchor。这样 $h_{\max}$ 是否被决定不再引起 2 分钟的停顿。
- 不建议把本地停止键前移（替换稿 §3.2(a)），那会使末尾损失加倍。

### K5：交接重叠 $\Delta_G$

**中优先级；确认。** 这是 S27 的无敌手版本，也是 1b 的诚实版本。

**位置。** [03 §4.3](03_session_manager_zh_v0_1.md) 第 259–263 行；W26；[10 §3.3](10_pipelines_zh_v0_2.md) 第 3–5 步。

**轨迹。** 沿用 10 §3.3 的数值：

1. c 的慢时钟 replicas 在真实时间上最晚签到 09:00:11 + $\delta_G$。
2. a 的快时钟 replicas 最早在 09:00:11 − $\delta_G$ 开始提议 32 411。
3. Res(32 410) 原本在 09:00:10.5 到达，余量 0.5 s。若 $\Delta_G>0.5$ s，一部分 a 的 replicas 已在 cid(32 409) 上开始出块，另一部分已切到 cid(32 410)，权重分裂在两个 cid 上。
4. 若最终选中 cid(32 410)，cid(32 409) 上的 block 被排除，成为 padding。

**影响。** 交接时作废工作和短暂停顿不再需要发布延迟（S27）或扣留证书（S10），时钟分布本身就能造成。规模约为 $\Delta_G$ 秒的生产。

**建议。** 后继的首次提议推迟到本地 $\tau_s^{\rm start}+\Delta_G$，这只恢复完美时钟下的交接余量。若同时采纳 S10(b)，推迟到 $\tau_s^{\rm start}+\Delta_G+\Delta_i+\Delta_{\rm cross}$（替换稿 §3.3）。代价是每次交接约有 $\Delta_G/b_{\rm L2}$ 个 padding heights。

**人工确认：** [ ] 是否在交接时加入 $\Delta_G$ 保护。

### K6：S10 与 S27 的放大

**中优先级；条件性**，以 S10 的触发能力为前提。

**位置。** S10（v0.2 §12.1）；S27（§12.18）；PUB-4 推论。

**内容。**

- **期权窗口变长。** $T_{\rm grace}$ 必须增加 $\delta_G$（替换稿 §3.4），S10 的期权窗口随之增加 $\delta_G$。
- **迟到证书不必扣留。** S10 第 1 步原来要求“Byzantine 一方在 stop 之后补齐证书”。偏差下，慢时钟 honest replicas 在真实 $\tau_s^{\rm stop}$ 到 $\tau_s^{\rm stop}+\delta_G$ 之间仍会合法投票，迟到证书可以完全由 honest 签名组成。
- **S10(b) 的推迟加长**为 $\Delta_G+\Delta_i+\Delta_{\rm cross}$。
- **S27 每次作废的工作**约增加 $\Delta_G$ 秒。

**建议。** S10 的修法 (b) 仍然有效，只需把推迟时长换成上面的值。v0.2 §13.2 第 590 行的表述按本文 §1 修改。

### K7：偏差加宽分裂投票的时间带

**中优先级；条件性**，取决于 BFT 的 proposal 传播方式。

**位置。** Accept 的三个阈值：5a 的 $\mathit{ts}^*(h)-\Delta_G$、5c 的 $\mathit{ts}^*(h+1)+T_{\rm win}$、5d 的 $\tau_s^{\rm stop}$；first_received（S20）。

**内容。**

- 点对点发送 proposal 时，Byzantine leader 本来就能选择对每个成员的发送时刻，从而跨过阈值分裂 honest 投票（S20 已述）。偏差不增加它的这种能力。
- 若 BFT 用可靠广播或纠删码传播，leader 不能逐个选择发送时刻，各成员的接收时刻相差在 $\Delta_i$ 以内。这时偏差把阈值附近的分裂带从 $\Delta_i$ 加宽到 $\Delta_i+\Delta_G$，Byzantine leader 只需选一个落在带内的发送时刻。
- 采用 K1、K2 的修法后，honest proposal 不会落入分裂带。
- 分裂一旦发生，first_received 使它成为永久的。在加锁的 BFT 中这会引向 S20(a) 的死锁。

**建议。** 与 S20 一并处理：first_received 以 $(\mathrm{cid},\mathsf H(B))$ 为键，对重新提议用 QC/POL 验证，不重新做时间检查。

### K8：超出 $\delta_G$ 的时钟攻击

**高优先级；模型外**（违反 C1–C3）；置信度高。

**触发能力。** 下列任一种，作用于 honest 成员：

- NTP 欺骗或延迟攻击；
- 对时间服务器的 BGP 劫持；
- 恶意的云宿主机；
- 闰秒或配置错误；
- 运维直接修改时间。

**安全性。** 不受影响（§1）：$G$ 不会分叉，不会出现双重选中，签名不能跨 cid 转用。

**快时钟偏移 $x$（权重 $w_f$）。**

- **G1.2 被违反 $x$。** BFT-2 只要求一个 honest replica 接受；只要有一个被攻击的 honest replica，block 就可以带着领先 $x$ 的时间戳进入 BFT chain。区域决定还需要 Byzantine 权重加上被攻击的 honest 权重达到 $Q_s$。
- **提前停止。** 被攻击的 replicas 提前 $x$ 停止。若剩余的 honest 权重低于 $Q_s$，窗口末尾 $x$ 的进展丢失。若 $x\ge\mathrm{TB}\cdot b_{\rm L2}$，它们进入时就已越过 $\tau_s^{\rm stop}$，Enter 返回 REJECT，整个 session 失去这部分权重。

**慢时钟偏移 $x$。**

- **迟到签名。** 被攻击的 replicas 在真实 $\tau_s^{\rm stop}+x$ 之前仍签名，迟到证书由 honest 签名组成（K6）。若 $x>T_{\rm grace}-(T_{\rm pub}+T_E+\Delta_{\rm ts})$，它们的结果错过 $\theta_s^{\rm end}$，这只是活性损失。
- **大规模 K1。** 它们按 K1 拒绝正常的 honest proposal。若这部分权重超过 $W_s-Q_s$，区域停顿。这里 $W_s$ 是委员会总权重。
- **无用的进入。** Manager 可能在真实 $\tau_s^{\rm stop}$ 之后才进入，进入后什么也产不出。

**不对称性〔补充〕。** 慢时钟可以用公开数据检测，快时钟不能：

- Espresso：在 live 期间，finalized 的 $J_q$ 满足 $\theta_q\le\tau_{\rm prop}+\Delta_{\rm ts}$，而它在 $\tau_{\rm prop}$ 之后才被读到。若本地读数 $c<\theta_q-\Delta_{\rm ts}-\delta_G$，说明本地时钟已慢于模型允许的范围。
- L1：头部 block 不早于其 slot 时间产生，同理可用。
- 快时钟的检测需要“最新 block 应当已经出现”这类活性前提，Espresso non-live 时无法与停机区分。

**建议〔补充〕。**

- (a) 时间来源：至少三个独立的认证时间源（NTS、Roughtime），取中位数，只调速不跳变，拒绝超过 $\delta_G$ 的校正。
- (b) 公开时间守卫：replica 发现 $C_m<\max\{\theta_q-\Delta_{\rm ts},\mathit{ts}_{\rm L1}(\text{head})\}-\delta_G$ 时停止签名（fail-stop），把可检测的慢时钟错误转为崩溃。
- (c) 监控委员会的时钟偏移分布，在权重接近 $W_s-Q_s$ 时报警。

**人工确认：** [ ] 是否把 (b) 写入 02 的停止守卫。

### K9：与 Espresso $\Delta_{\rm ts}$ 的共同故障模式

**低优先级；条件性。**

**内容。**

- 在模型之内，$\Delta_{\rm ts}$ 与 $\delta_G$ 在预算中直接相加（$T_{\rm grace}$、$T_{\rm init}$、$T_{\rm prep}$），不需要两者独立。
- 在模型之外，若 Espresso 节点与 replicas 用同一套时间基础设施（同一云厂商的 NTP），一次故障会同时破坏 E-TS 与 C2，而 K8(b) 的守卫正是以 E-TS 为参照，这时守卫也一起失效。
- S14（non-live 期间 Espresso block 可带旧 timestamp）使守卫在 non-live 时本来就不可用。
- S26（proposer 在 $\Delta_{\rm ts}$ 内移动 $\theta$）与本地时钟无关，偏差不加重它。

**建议。** Replicas 的时间源与 Espresso 节点的时间源分开部署。K8(b) 的守卫只在 Espresso live 时启用，且同时参照 L1。

### K10：Manager 与 Program 不同源

**低优先级；条件性**，在不采用 C5 时成立。

**内容。** Program 自己的停止守卫仍然保证本地形式的 BFT-7。但 Manager 的 TIME 关闭、Enter 守卫 5 与 Program 的 REJECT 可能相差 $\Delta_G$：Manager 认为可以进入，Program 却已拒绝。这会造成无效的 WAIT 或 REJECT 循环，只影响活性。

**建议。** 采用 C5。

## 4. 修复与替换稿的对应

| 条目 | 替换稿位置 |
| --- | --- |
| K1 | §3.2(c) 等待修法；附录 A 滞后修法|
| K2 | §3.2(e) 第 2 项 |
| K3 | §3.2(g) 的 G1.2 |
| K4 | §3.2(e) 第 1 项；§3.8 窗口末尾；与 S10(b) 一并处理 |
| K5 | §3.3 第 3 项；§3.8 正常交接 |
| K6 | §3.4；§3.5 第 2–3 项 |
| K7 | 与 S20 一并处理 |
| K8 | 本文建议 (a)–(c)，尚未写入替换稿 |
| K9 | 部署约束 |
| K10 | §1.2 C5 |

## 5. 人工确认清单

- [ ] $\Delta_G$ 只约束时钟域 $\mathsf{Clk}$（replica 与 Manager），还是在模型陈述上覆盖所有参与方（对其他参与方没有约束作用）。
- [ ] 是否接受 C2（$\delta_G$）作为独立参数，或采用单参数 $\Delta_G=2\delta_G$。
- [ ] Accept 采用等待、放宽、滞后哪一种修法（K1）。
- [ ] 本地停止键保持在 $\tau_s^{\rm stop}$（建议），或前移。
- [ ] 交接是否加入 $\Delta_G$ 保护（K5），以及是否同时采纳 S10(b)。
- [ ] $T_{\rm prep}$ 调到多少（10 的下界余量为 0）。
- [ ] 是否把公开时间守卫写入规格（K8(b)）。
- [ ] 以新版本文件落地，还是在原文件上就地修改（替换稿 §4）。
