# L1 与 Espresso 时间漂移的安全审查（v0.2）

日期：2026-11-07。上一版：[v0.1](clock_skew_review_zh_v0_1.md)（K1–K10，只涉及副本时钟）。对象：本目录当前规格在 [多时间域模型 v0.2](clock_skew_zh_v0_2.md)（下称“模型稿”）之下的行为。误差 $e_E$、$e_{\rm L1}$，界 $\delta_E^\pm$、$\delta_{\rm L1}$，跨域比较 X1–X12 与容忍量表，都在模型稿 §1、§3、§4 中定义，本文直接引用。

本版新增 K11–K19。K1–K10 原样有效，其中 K8(b)、K9 由 K19 修订。优先级与状态的用法同 [安全审查 v0.2](security_review_zh_v0_2.md) §1；所有修复都是建议，等你决定。

## 1. Safety

**不受影响的部分。** v0.1 §1 的表在 L1、Espresso 漂移下仍然成立：BFT-1/3/4、CheckFINAL、GFR 重放的确定性、$D$、PUB-1…3 的证明都只读公开值，不读任何一方的时间是否准确。$\theta$ 与 $\mathit{ts}_{\rm L1}$ 取任何值，不同读取者重放同一份日志都得到同一条 $G$。

**唯一的时间相关前提：$\theta$ 的单调性（K11）。** watermark 用“末端 timestamp 至少为 $x$”证明“不会再出现 timestamp 小于 $x$ 的 block”。这一推理需要 $\theta_q\le\theta_{q+1}$，而且是对所有执行都要求的 safety 条件，不是活性预算。L1 的 timestamp 由共识规则保证严格递增，没有对应问题。

**会被破坏的承诺。** 漂移超出预算时，下列承诺失效，但都不造成读取者之间的分歧：

- “区域已决定且按时发布的 block 会进入 $G$”（K12、K13）；
- “按时进入 $G$ 的 block 会被原样派生”，它依赖 X9（K14）；
- “L1 控制记录在激活后生效”（K14、K15）；
- “停止后的证书不能再扩展 $G$”在真实时间上的读法（K14、K15）。

## 2. 条目总览

| 编号 | 标题 | 优先级 | 状态 | 关系 |
| --- | --- | --- | --- | --- |
| K11 | $\theta$ 不单调时，读取者之间的 $G$ 分叉 | 高 | 条件性（取决于 HotShot） | safety |
| K12 | Espresso 持续超前：区域确认被成批撤销 | 高 | 确认 | S26 的模型外版本 |
| K13 | Espresso timestamp 跳变：长时间停顿，可直至 HALT | 高 | 条件性（取决于 HotShot） | — |
| K14 | Espresso 滞后：截止与治理推迟 | 中 | 确认 | 加重 S10 |
| K15 | Espresso 时间冻结：L1 治理在 GFR 中失效 | 高 | 确认 | 09 §4.4 的 non-live 说明不完整 |
| K16 | 时间故障的检测 | 低 | 建议 | — |
| K17 | L1 finality 停滞：区域生产与 $G$ 一起停止 | 高 | 确认 | — |
| K18 | 按 height 推算 L1 timestamp 的实现陷阱 | 中 | 条件性 | — |
| K19 | 修订 K8(b)、K9：守卫不能参照 $\theta$ | 中 | 确认 | v0.1 K8、K9 |

## 3. 条目

### K11：$\theta$ 不单调时，读取者之间的 $G$ 分叉

**高优先级；条件性**，HotShot 不在共识规则中强制 $\theta_q\le\theta_{q+1}$ 时成立。

**位置。** 00 §1.4 的 E-S；[09 §3.3–§3.4](09_finality_rules_zh_v0_2.md) 的全序、$q^*$ 与 R-WAIT。

**最小轨迹。** session $s$ 的 $\theta_s^{\rm end}=100$。

1. $J_5$ 的 $\theta_5=100$。读取者 A 读到 $J_0,\dots,J_5$，watermark 已越过 100，于是处理 SessionEnd($s$)，关闭 $s$。
2. 之后 finalized 的 $J_6$ 带 $\theta_6=99$，其中是 $s$ 的一份完整结果。A 的处理位置已在键 100，无论丢弃 $J_6$ 还是在当前位置处理它，$s$ 都已关闭，不接纳；读取者 B 一次读到 $J_0,\dots,J_6$，按键排序后 $J_6$（99）排在 SessionEnd（100）之前，接纳。
3. A、B 的 $G$ 不同。

**影响。** $G$ 在读取者之间分叉，Batcher、Certifier、各区域 Manager 的依据不一致。

**建议。** 采用模型稿 §5.1：GFR 的键改用前缀最大值 $\bar\theta_q$。它按构造单调，若 HotShot 已强制单调则规则不变。另需核实 HotShot 是否在 header 校验中强制 $\theta$ 不递减。

**人工确认：** [ ] 采用 $\bar\theta$；[ ] 核实 HotShot 的单调性规则。

### K12：Espresso 持续超前

**高优先级；确认。**

**触发能力。** 不需要敌手：Espresso 验证者共用的时间源整体偏快即可。在 $\Delta_{\rm ts}$ 以内由 Byzantine proposer 造成的版本是 S26；本条讨论超出模型的部分。

**位置。** X3（SessionEnd 截止）、X5（R-WAIT 的 L1 条件）；[03 §4.3](03_session_manager_zh_v0_1.md) 的候选调度。

**轨迹。** 取 10 的参数，Espresso 持续超前 $x=90$ s。Session (7,7) 的 $\tau^{\rm stop}=$ 10:00:11，$\theta^{\rm end}=$ 10:02:11。

1. 真实时刻 $t$ 形成的结果，公开时刻约为 $t+T_{\rm pub}+T_E+x=t+150$ s，赶上截止要求 $t<$ 10:00:11 − 30 s。最后约 30 s（约 30 个 block）的区域结果被排除出 $G$。它们已经被区域决定，用户已经看到确认。
2. 后继 a 在 10:00:11 以最高候选进入，从被排除的那个末端接着出块。GFR 处理 SessionEnd 要等 L1 覆盖 $\theta^{\rm end}-M_a$，在真实时刻约 10:01:35 才选定较低的 anchor；a 在此之前约 84 s 的工作在错误候选上，作废，对应 heights 由后继的 padding 填补。
3. 每个 Espresso 事件的处理还被 L1 拖慢 $x-36=54$ s，$G$ 的确认延迟同样增加。
4. 每次交接都重复。每小时约 2 min 的区域确认被撤销。

$x$ 达到约 1 h（$\mathrm{TB}+T_{\rm grace}$）时，每个 session 的全部结果都错过截止，$G$ 不再增长，约 $W_{\rm exp}$ 之后分支 HALT。

**影响。** 区域确认的可信度直接取决于 Espresso 的时钟，而容忍量只有约 60 s（$T_{\rm grace}-T_{\rm pub}-T_E$）减去 $\delta_G$。

**建议。**

- 核实 HotShot 对 timestamp 的校验：是否以验证者本地时钟为上界，容差多大。这决定 $\delta_E^+$ 的实际取值。
- 按模型稿 X3 把 $\delta_E^+$ 显式写进 $T_{\rm grace}$ 的预算。加大 $T_{\rm grace}$ 能扩大容忍量，代价是 1a 交接的停顿和 S10 的期权窗口都变长。
- 监控 $\theta$ 相对 L1 的偏差（K16），在接近 36 s、60 s 时报警。

**人工确认：** [ ] $\delta_E^+$ 的取值；[ ] 是否调整 $T_{\rm grace}$。

### K13：Espresso timestamp 跳变

**高优先级；条件性**，HotShot 允许 timestamp 一次前进远超真实时间时成立。

**触发能力。** 一个 Espresso proposer，若 HotShot 不以验证者时钟限制 timestamp；否则需要足以形成 quorum 的验证者时钟一起出错。

**轨迹。** 真实时刻 10:30，$J_q$ 的 $\theta$ 跳到 13:30，即 $X=3$ h。此后 honest proposer 只能给出不小于 13:30 的 timestamp，直到真实时间追上。

1. **Espresso 事件停顿。** 键不小于 13:30 的事件要等 L1 覆盖 13:10，真实时刻约 13:29 才能处理。定时器不受影响：watermark 已越过它们的键，只受 L1 约束，各在键前约 36 s 处理。
2. **结果被排除。** 10:30 之后公开的结果，键都不小于 13:30。$\theta^{\rm end}$ 早于 13:30 的 session，即 11:00、12:00、13:00 结束的三个，失去 10:30 之后的全部结果。它们的 SessionEnd 按时处理，anchor 一直停在 10:30 附近的 block。
3. **恢复。** 13:00 开始的 session 在约 13:01:24 从 SessionEnd 得到这个 anchor，切换到匹配的候选。它的结果键为 13:30，早于自己的截止 14:02:11，从约 13:29 起被接纳。
4. **合计。** 约 $X$（10:30 到约 13:01）的区域确认被撤销，对应 heights 成为 padding。期间各区域照常出块，产出全部作废。
5. **$X\gtrsim W_{\rm exp}$（10 h）时。** 跳变后的第一个结果已晚于下一缺失高度的过期键，分支直接 HALT，只能 RESUME。

**影响。** 单个 block 的 timestamp 可以造成数小时的全局停顿与成批撤销。由于 $\theta$ 单调，错误不能被后续 honest block 纠正，只能等真实时间追上。

**建议。**

- 核实 HotShot 的上界校验（同 K12）。
- 采用模型稿 §5.2 的上界夹逼，把影响限制在 $K^+$ 以内。这需要 Espresso block 携带经验证的 L1 引用。
- 定义运维响应。Force 不能修复已经偏大的键；HALT 之后只有 RESUME。

**人工确认：** [ ] 是否采用 §5.2 的上界。

### K14：Espresso 滞后

**中优先级；确认。**

**位置。** R-WAIT 第二条；X3、X6、X7、X8、X9。

**轨迹。** Espresso 持续滞后 $y=5$ min。沿用 10 §4 的 Force 流程：$U$ 收录于 10:10:00，$\mathrm{act}=$ 10:30:00，新窗口从 10:32:01 开始。

1. timestamp 不小于 10:30:00 的 watermark 在真实时刻约 10:35:30 才可读，GFR 这时才处理 Force。
2. 旧 session 在激活后的真实 5 min 内仍能扩展 $G$。Force 若是为了移除一个作恶的区域，这 5 min 仍由它控制。
3. 新窗口的 Manager 在 10:35:30 才得到 SessionAssigned。10:32:01 起的约 3.5 min 没有生产，由后续 padding 填补。

其他后果：

- **截止推迟。** SessionEnd 在真实时间上推迟 $y$，S10 中隐藏证书的期权窗口变为 $T_{\rm grace}+y$。
- **RESUME 来不及。** X8 的下界余量本来为 0。
- **$y>10$ min 时 $G$ 与 $D$ 不一致。** 在过期边缘才进入 $G$ 的 block 可能赶不上原生截止：$D$ 补入 deposit-only，之后 RESUME 撤销 $G$ 的后缀。

**建议。** 按模型稿 X7、X8、X9 加入 $\delta_E^-$。确认 $T_{\rm ready}$ 是否已含 watermark 等待。在 09 §4.4 第 5 步的说明中写明，准备预算只在 Espresso 时间正常时成立。

### K15：Espresso 时间冻结

**高优先级；确认。** 这是 K14 中 $y$ 无界增长的极限，但后果在性质上不同。

**触发能力。** Espresso 正常 finalize，但 $\theta$ 不再前进或明显慢于真实时间。可能的原因：验证者时间源整体故障；或 HotShot 只要求 $\theta$ 不小于 parent，而 proposer 连续沿用 parent 的 timestamp。

**轨迹。** $\theta$ 从真实 10:00 起停在 10:00:00。

1. 键为 10:00:00 的 Espresso 事件照常处理。区域结果仍被接纳，接纳到 $h_{\max}$ 时的常规交接（[09 §4.2](09_finality_rules_zh_v0_2.md)）也照常进行。
2. 键更大的事件全都等不到 watermark：Force、RESUME、SessionEnd、Expire 都不会被处理。
3. 截止条件 5 永远成立，隐藏多久的证书都能被接纳。
4. 一个决定不了 $h_{\max}$ 的 session 使 $G$ 永久停在 NEXT；没有 SessionEnd 交接，也没有 Expire。
5. 这时 Force 与 RESUME 同样等不到 watermark，GFR 内部没有任何恢复路径。

$D$ 仍按 L1 推进：Batcher 无可提交时，$W_{\rm seq}$ 之后 $D$ 补入 deposit-only blocks，L1 deposits 仍被强制包含。

**影响。** “live 但时间错”的 Espresso 使 L1 治理完全失效，而且模型目前无法表达这种情形：$\mathsf{Live}_E=1$ 时规格假设 E-TS 成立。[09 §4.4](09_finality_rules_zh_v0_2.md) 的结论“Force 只在 Espresso live 时有实际作用”应改为“live 且时间正常时”。一个作恶区域如果能决定 $h_{\max}$，就可以在冻结期间一直控制生产，治理无法移除它。

**建议。**

- 模型：采用模型稿 §1.4，把 E-TS′ 与 $\mathsf{Live}_E$ 分开，并在 09 中如实写明治理依赖 Espresso 时间。
- 规则：采用模型稿 §5.2 的下界夹逼，或 §5.3 的 L1 超时。两者都需要 Espresso block 携带经验证的 L1 引用。
- 若两者都不可行，在 10 中写明这一失效模式与它的出口：$D$ 的原生回退。

**人工确认：** [ ] 是否接受“Espresso 时间冻结时治理失效”作为已知限制。

### K16：时间故障的检测

**低优先级；建议。**

L1 的 timestamp 可靠（模型稿 §1.3），可以作为参照。任何在线参与方都能比较新 finalized 的 Espresso block 的 $\theta$ 与当时 L1 head 的 timestamp：

- 差值持续偏正，说明 Espresso 超前（K12、K13）；
- 偏负或不再增长，说明 Espresso 滞后或冻结（K14、K15）；
- 缓慢变化，说明 Espresso 时间源有速率漂移，最终会落到上面某一种。

这与 v0.1 K8 的情形不同：那里以 Espresso 为参照检测副本时钟，只有一个方向可检测；这里以 L1 为参照，两个方向都可检测。但检测结果依赖观察时刻，不能直接写成 GFR 的确定性规则；要写成规则，需要模型稿 §5.2 的 L1 引用。

**建议。** 监控阈值对应模型稿 §4 的容忍量：超前 36 s、60 s，滞后 90 s、10 min。为每档写明运维响应。

### K17：L1 finality 停滞

**高优先级；确认。** 属于 L1 活性，不是 timestamp 误差。

**位置。** [00 §1.3](00_arch_zh_v0_1.md)（所需 L1 前缀不可读时计算等待）、§3.4（$T_{\rm L1}\le T_{\rm dep}$）；09 R-WAIT 第一条。

**轨迹。** 以太坊 finality 在真实时刻 $t_0$ 停止，finalized head 停在 timestamp 约为 $t_0-\lambda$ 的 block，$\lambda$ 是停止前的 finalized 滞后（正常约 13 min，预算上界 $T_{\rm L1}=19.2$ min）。

1. **区域生产停止。** $\ell^{\rm sch}(h)$ 要求 timestamp 不超过 $\mathit{ts}^*(h)-T_{\rm dep}$ 的 block 已 finalized。区域生产在 $\mathit{ts}^*(h)$ 越过 $t_0-\lambda+T_{\rm dep}$ 后等待，即 $t_0$ 之后约 7 min（正常滞后）或 48 s（最坏滞后）。预确认服务随之停止。
2. **$G$ 停止。** R-WAIT 要求 L1 finalized 覆盖 $x-M_a$，GFR 在相近的位置停止。
3. **$D$ 停止。** $D$ 只读 finalized L1。
4. **恢复。** finality 恢复后，区域按时间表从当前 slot 继续，中间的 heights 由 padding 补齐（G3.2）。

**影响。** 整个系统的可用性以 L1 finality 为下限。2023 年 5 月以太坊主网曾两次失去 finality，分别约 25 min 与 1 h（待核对）。

**建议〔补充〕。** 先确认这是不是有意的设计。若希望区域在 L1 finality 停滞时继续出块，一种方向是让 origin 在 $T_{\rm drift}$ 以内停止前进：由 leader 提议 origin，BFT 检查它已 finalized 且满足 O1–O3，而不是由 $\ell^{\rm sch}$ 决定。代价是 origin 不再由公开时间表唯一决定，00 §3.4 与 G2 的有关证明要重写。

**人工确认：** [ ] L1 finality 停滞时区域停产是否可以接受。

### K18：按 height 推算 L1 timestamp 的实现陷阱

**中优先级；条件性**，实现按 00 §1.2 的公式由 height 计算 timestamp 时成立。

**内容。** 原生派生用 header timestamp 检查 origin 的 not-ahead 与 drift。若 schedule、$\mathrm{act}$ 或 Builder 按 $\tau^0_{\rm L1}+\ell b_{\rm L1}$ 计算，每个空 slot 使公式值比 header 早 12 s，误差只增不减：

- Builder 给出的 origin 可能违反原生 O2、O3，批次不被派生采用，$D$ 补入 deposit-only，$G$ 与 $D$ 不一致；
- 不同实现若一个读 header、一个用公式，$\ell^{\rm sch}$、$\mathrm{act}$ 不同，GFR 读取者之间也会分歧。

**建议。** 采用模型稿 §1.3 的 L-PACE。10 中“12 h = 3 600 个 L1 block”的换算已注明不含空 slot；按 block 计数的窗口在有空 slot 时只会更长，是保守的。

### K19：修订 K8(b)、K9

**中优先级；确认。**

**K8(b) 的问题。** v0.1 K8(b) 的公开时间守卫是：$C_m<\max\{\theta_q-\Delta_{\rm ts},\mathit{ts}_{\rm L1}(\text{head})\}-\delta_G$ 时停止签名。Espresso 一旦超前超出 $\Delta_{\rm ts}$，每个 honest replica 都会把自己判为“时钟过慢”而停止签名，一个 Espresso 时间故障就会变成整个区域的停产。

**修订。** 守卫只参照 L1：$C_m<\mathit{ts}_{\rm L1}(\text{head})-\delta_{\rm L1}-\delta_G$ 时停止签名。L1 head 的 timestamp 不会超前于以太坊多数验证者的时钟，而 replica 读到 head 只会更晚，所以这个守卫不会误伤时钟正常的 replica。

**K9 的修订。** 共同故障模式改用模型稿的记号表述：副本误差、$e_E$、$e_{\rm L1}$ 来自同一时间源时，它们会同向偏移，X3、X5 的余量同时被消耗。建议副本、Espresso 验证者与以太坊节点的时间源分开部署。

## 4. 修复与模型稿的对应

| 条目 | 模型稿位置 |
| --- | --- |
| K11 | §5.1 |
| K12 | §3 X3、X5；§4.1 |
| K13 | §5.2 上界 |
| K14 | §3 X7、X8、X9；§4.2 |
| K15 | §1.4；§5.2 下界；§5.3 |
| K16 | §4 的阈值 |
| K17 | §1.3 finalized 延迟 |
| K18 | §1.3 L-PACE |
| K19 | §3 X12 |

## 5. 人工确认清单

- [ ] 核实 HotShot 的 timestamp 校验：单调性、相对验证者时钟的上界与容差、是否携带并验证 L1 引用（K11、K12、K13、K15）。这一项决定后面几项的取舍，建议最先做。
- [ ] Espresso 键改用前缀最大值 $\bar\theta$（K11）。
- [ ] 采用不对称的 $\delta_E^+,\delta_E^-$，并取值（K12、K14）。
- [ ] E-TS′ 与 $\mathsf{Live}_E$ 分开（K15）。
- [ ] 是否采用 L1 夹逼的有效键（模型稿 §5.2），或只为治理加 L1 超时（§5.3），或把治理失效作为已知限制（K13、K15）。
- [ ] $M_a$ 预算中第二个 $\Delta_{\rm ts}$ 的来源（模型稿 §3 X5）。
- [ ] $T_{\rm ready}$ 是否已含 watermark 等待（X8）。
- [ ] L1 finality 停滞时区域停产是否可接受（K17）。
- [ ] L-PACE 替换 fixed-pace（K18）。
- [ ] 公开时间守卫只参照 L1（K19）。
