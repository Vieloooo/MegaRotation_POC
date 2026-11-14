# 时钟模型修改：设计决定记录（v0.1）

日期：2026-11-08；2026-11-09 加入决定 12、13。分支：`clock-skew`。依据：[时钟偏差分析 v0.1](clock_skew_zh_v0_1.md)、[多时间域模型 v0.2](clock_skew_zh_v0_2.md) 及两份审查（[v0.1](clock_skew_review_zh_v0_1.md)、[v0.2](clock_skew_review_zh_v0_2.md)）。

本文只记录已经做出的决定和尚待决定的事项，供之后修改规格时使用。2026-11-09 已按本文写成 [formalization_v10](../formalization_v10/README.md)，落地情况与本文的出入见 §7。§2 中“我的理解”一栏是对决定的展开，与原话有出入时以原话为准，请核对。

## 1. 范围与落地

**决定 1：落地方式。** 出新版本文件：00、01、02、03、04 各出 v0.2，09、10 各出 v0.3。旧版本保留作对照，README 指向新版本。（落地时改为新目录 formalization_v10，见 §7。）

**决定 2：协议改动的范围。** 规则只因副本时钟偏差而修改，即 v0.1 分析 §3 的内容，按本文的决定取舍。L1 与 Espresso 的时间只进入模型假设和预算，不为它们改规则。

## 2. 已定决定

| 编号 | 主题 | 决定 | 我的理解与对规格的影响 |
| --- | --- | --- | --- |
| 决定 3 | Accept 的时间条件 | 采用等待修法 | 02 §3.8 第 5 项拆为 5a–5d：过早超过 $\Delta_G$ 才 REJECT，否则返回 WAIT(time)。G1.2 为 $\mathit{ts}^*(B.h)\le\tau_{\rm in}(B)+\delta_G$ |
| 决定 4 | Espresso timestamp 单调性 | 保留为假设：$\theta_q\le\theta_{q+1}$ | 仍是 00 §1.4 E-S 的一部分。GFR 不改用前缀最大值。审查 K11 只作为“假设被违反时会怎样”的讨论 |
| 决定 5 | Espresso timestamp 精度 | 正常情况下有固定界 $\Delta_{\rm ts}$；超出时只讨论，不改协议 | E-TS 保持现行形式（对称 $\Delta_{\rm ts}$，live 时使用），不拆分，也不与 $\mathsf{Live}_E$ 脱钩。不采用 L1 夹逼或治理超时。E-TS 不成立时，接受 $D$ 的活性损失与 $G$、$D$ 之间的不一致，由 RESUME 恢复 |
| 决定 6 | L1 timestamp | 始终在真实时间的固定界内，不讨论超出的情形 | 00 §1.2 新增假设 $\lvert\mathit{ts}_{\rm L1}(\ell)-\tau^{\rm prod}(\ell)\rvert\le\delta_{\rm L1}$，对所有执行成立。$\delta_{\rm L1}$ 作为小量进入 $T_{\rm dep}$、$M_a$、$T_{\rm init}$、$T_{\rm prep}$、$W_{\rm seq}$ 的预算（落地时只进入 $T_{\rm dep}$、$M_a$、$W_{\rm seq}$，见 §7） |
| 决定 7（原 A） | 副本时钟的参数 | 只用 $\delta_G$，由它推出两两偏差 | 两两偏差界取 $\Delta_G:=2\delta_G$，接受这个保守的界。00 §2.3 只给出 C2（$\lvert C_m(\tau)-\tau\rvert\le\delta_G$）及 C3–C5；C1 作为推论。v0.1 分析中凡用 $\Delta_G$ 的公式，一律代入 $2\delta_G$ |
| 决定 8（原 B） | 本地停止时刻 | 停止键保持在本地 $\tau_s^{\rm stop}$，不提前 | BFT-7 改为本地形式，并加推论：真实 $\tau_s^{\rm stop}+\delta_G$ 之后没有 honest 签名。下游预算各加 $\delta_G$（$T_{\rm grace}$、PUB-4 推论、09 SessionEnd 的说明） |
| 决定 9（原 D） | 时钟损坏的节点 | 诚实节点发现自己的时钟损坏时下线；这类节点与 Byzantine 节点合计不超过区域的容错阈值 | 见 §3 的展开 |
| 决定 10（原 E） | WAIT(time) | 接受 | 02 §4 写成对所选 BFT 实现的明确要求：收到 WAIT(time) 时不投票、不判无效；本地时钟到达 $\mathit{ts}^*(h)$ 时以原首次接收时刻重试；$T_{\rm fin}$、$T_{\rm next}$ 能吸收至多 $\delta_G$ 的等待 |
| 决定 11（原 F） | 示例取值 | $\delta_G=1$ ms | 于是 $\Delta_G=2$ ms。见 §4 |
| 决定 12（原 C） | 交接重叠（审查 K5） | 不改协议，加预算条件 | 后继不推迟首次提议。10 的交接一节加条件：$\Delta_G$ 小于交接余量，即前驱最后一个高度的结果到达时刻与后继本地 $\tau^{\rm start}$ 之差。示例余量 0.5 s，$\Delta_G=2$ ms 时满足。若以后采纳 S10(b)，推迟规则届时一并引入，推迟时长含 $\Delta_G$ |
| 决定 13（原 G1） | $M_a$ 的公告预算 | 改写为精确式 $M_a\ge T_{\rm L1}+b_{\rm L1}+\Delta_{\rm ts}+\delta_{\rm L1}$ | 替换 09 §3.5“公告”与 10 校准表中的 $T_{\rm L1}+2\Delta_{\rm ts}+b_{\rm L1}$；$M_a=20$ min 不变。推导：键为 $x$ 的 Espresso 事件最早在真实 $x-\Delta_{\rm ts}$ 可读；覆盖 $x-M_a$ 的 L1 block 的 timestamp 小于 $x-M_a+b_{\rm L1}$，在真实 $x-M_a+b_{\rm L1}+\delta_{\rm L1}+T_{\rm L1}$ 之前 finalized。控制记录只需 $T_{\rm L1}+\Delta_{\rm ts}+\delta_{\rm L1}$，已被蕴含。余量 $21\,{\rm s}-\delta_{\rm L1}$，取 4 s 时约 17 s。$b_{\rm L1}$ 一项假设该 block 在一个 slot 内出现；若 H1 改读 header timestamp，连续空 slot 会加大这一项，只影响“L1 不拖慢 R-WAIT”的结论 |

## 3. 决定 9 的展开

**原意。** 诚实节点一旦发现自己的时钟损坏，就下线。时钟损坏仍受区域的 $3f$ 或 $5f$ 阈值约束：损坏节点加上恶意节点不超过阈值。

**我的理解。** 记 $\mathrm{Clk}^{\rm bad}_s\subseteq V_s$ 为 session $s$ 中时钟不满足 C2、C3 的节点。00 §2.3 的容错假设由

$$
\sum_{m\in\mathrm{Byz}_s}w_s(m)\le F_s
$$

改为

$$
\sum_{m\in\mathrm{Byz}_s\cup\mathrm{Clk}^{\rm bad}_s}w_s(m)\le F_s .
$$

时钟条件 C1–C5 只约束其余节点。时钟损坏的节点在区域 BFT 中按故障节点计入阈值，安全性质与 G1.2 等时间性质都按时钟正常的 honest 节点陈述。若下面第一项的答案为“计入”，v0.1 审查 K8 中超出 $\delta_G$ 的时钟故障就全部进入了模型，后果与等量的 Byzantine 权重相同。

**待确认。**

- [ ] 时钟损坏但**没有发现**的节点，是否也计入 $\mathrm{Clk}^{\rm bad}_s$？我建议计入：这类节点的行为按 Byzantine 计算即可覆盖，也最贴近“损坏节点加恶意节点不超过阈值”的说法。
- [ ] “发现时钟损坏”要不要在 02 中给出具体判据？可选判据是 v0.2 审查 K19 的 L1 守卫：$C_m<\mathit{ts}_{\rm L1}(\text{head})-\delta_{\rm L1}-\delta_G$ 时下线。由决定 6，时钟正常的节点不会触发它，但它只能发现慢时钟。不给判据时，下线只作为节点行为的假设。
- [ ] “下线”是否等同于停止签名（Close 当前实例、不再 Enter）？之后时钟恢复，能否在后续 session 重新参与？

## 4. 决定 11 的数值后果

取 $\delta_G=1$ ms、$\Delta_G=2$ ms，并沿用 10 的其余参数：

| 预算 | 新形式 | 结果 |
| --- | --- | --- |
| $T_{\rm dep}$ | $T_{\rm L1}+\delta_G+\delta_{\rm L1}\le T_{\rm dep}$ | 余量约 $48\,{\rm s}-\delta_{\rm L1}$ |
| $T_{\rm grace}$ | $>T_{\rm pub}+T_E+\Delta_{\rm ts}+\delta_G$ | 余量约 45 s |
| $T_{\rm prep}$ 下界 | $M_a+T_{\rm ready}+\delta_G+\delta_{\rm L1}\le T_{\rm prep}$；若 G2 的答案为否，左边再加 $T_E+\Delta_{\rm ts}$ | 现行余量为 0，**必须调大 $T_{\rm prep}$**，取值待定（§5）。落地时改为精确式，25 min 保留，见 §7 |
| 窗口末尾 | $\Delta_G+T_{\rm build}+T_{\rm fin}<b_{\rm L2}$ | 偏差项只占 2 ms |
| 正常交接（决定 12） | $\Delta_G<$ 交接余量（示例 0.5 s） | 满足，余量约 498 ms |
| $T_{\rm win}$ | $\ge T_{\rm build}+\Delta_{i_s}+2\delta_G$ | 偏差项 2 ms |

部署前提：跨区域、相对真实时间 1 ms 的界，通常需要 PTP 或 GPS 校准的时钟；广域网上的 NTP 一般只能做到毫秒到数十毫秒。10 中应把这一点写成部署前提。

## 5. 尚待决定

| 原编号 | 事项 | 当前状态 |
| --- | --- | --- |
| G2 | $T_{\rm ready}$ 是否已含 Espresso watermark 等待（约 $\Delta_{\rm ts}+T_E=45$ s）。10 第 75 行已把 watermark 列入 $T_{\rm ready}$ 须覆盖的工作，按原文为“包含”。$T_{\rm init}$ 从 $\mathrm{act}$ 起算，Manager 最晚在 $\mathrm{act}+45$ s 才得知新窗口，实际准备时间约 75 s；建议定义不变，只在 10 中写明 | 待确认 |
| H1 | L1 fixed-pace 公式：把 $\ell$ 解释为 slot，或改为读 header timestamp。按决定 6，需要二选一 | 未答复 |
| H2 | “假设不成立”的讨论放在哪里：09 v0.3 只加一段说明，详细分析保留在审查文件中 | 未答复 |
| — | $\delta_{\rm L1}$ 的示例取值（建议 4 s） | 未定 |
| — | $T_{\rm prep}$ 的新取值（建议 26 min） | 未定 |
| — | 10 是否给出 $T_{\rm build}$、$T_{\rm fin}$ 的示例值，以核算窗口末尾条件 | 未定 |
| — | 决定 9 的三项待确认（§3） | 未定 |
| — | 审查 K17：L1 finality 停滞时区域停产是否可以接受 | 未答复 |

## 6. 对既有分析文件的影响

- **v0.2 分析中不再采用的内容。** §1.4 把 E-TS 拆为 $\delta_E^\pm$ 并与 $\mathsf{Live}_E$ 脱钩、§5.1 前缀最大值、§5.2 L1 夹逼、§5.3 治理超时，都与决定 4、5 不符。修改规格时，另出 v0.3 分析，把这些内容改写为模型外讨论。
- **审查文件。**
  - K11–K15 属于 Espresso 假设不成立时的讨论，保留，但不再对应协议修改。
  - K16 以 L1 为参照检测 Espresso 时间故障，与决定 6 一致，保留为监控建议。
  - K17 是 finality 延迟超出 $T_{\rm L1}$ 的活性问题，与 timestamp 误差无关，不受决定 6 影响，仍待确认。
  - K18 对应 §5 的 H1。
  - v0.1 的 K5 由决定 12 处理：不加推迟规则，只加预算条件。
  - v0.1 的 K8、v0.2 的 K19 由决定 9 吸收。
- **v0.1 分析中的公式。** 按决定 7，以 $2\delta_G$ 代入 $\Delta_G$；附录 A 中滞后修法的比较不再需要。

## 7. 落地记录（2026-11-09）

**落地位置。** 不在 v9 中出 v0.2、v0.3，而是新建 [formalization_v10](../formalization_v10/README.md)，十一个文件都从 v0.1 编号；v9 保留作对照。05–08 不读时钟，只改了文件头与链接。

**与上文的出入。**

- 决定 6：$\delta_{\rm L1}$ 只进入 $T_{\rm dep}$、$M_a$、$W_{\rm seq}$。$T_{\rm prep}$ 与 $T_{\rm init}$ 从激活时刻起算，所需 L1 block 已由公告预算保证在激活前 finalized，不再计入 $\delta_{\rm L1}$。
- 决定 7：00 §2.3 只给出 C2、C3、C5；C1（两两偏差 $\Delta_G$）与 C4（本地时长与真实时长相差至多 $\Delta_G$）都作为推论。另加到达时刻 $\tau_m(x)$ 与 L-CLK，供各模块把本地阈值换成真实时刻。
- §4 的 $T_{\rm prep}$ 下界：改为精确式 $T_{\rm prep}\ge M_a$ 与 $M_a+T_{\rm ready}+\delta_G\le T_{\rm prep}+b_{\rm L2}$（v10 [01 §3.10](../formalization_v10/01_l1_authority_zh_v0_1.md)）。$b_{\rm L2}$ 来自新 session 首个 height 为 $h_{\rm res}+1$。$T_{\rm prep}=25$ min 保留，余量 999 ms；G2 为“不含”时须取 26 min。
- 同步开始：02 §4.6 的同步开始取 $\tau_{\rm sync}=\tau_s^{\rm start}-\delta_G$。分析稿中的 $\tau_s^{\rm start}+\delta_G$ 晚于部分 replicas 的 view 1 起算时刻，会使 view 1 不满足“在 $\tau_{\rm sync}$ 后开始”的定义。

**§3、§5 待决事项采用的默认取值。** 文中标〔待确认〕，汇总于 [v10 README](../formalization_v10/README.md)：

| 事项 | 默认取值 |
| --- | --- |
| H1 | 保留 fixed-pace 公式，$\ell$ 按 slot 编号理解 |
| H2 | v10 09 §3.5 只加一段说明，详细分析留在审查 K11–K16 |
| G2 | $T_{\rm ready}$ 含 watermark 等待；10 §4.4 写明 Force 后最坏实际准备时间约 75 s |
| $\delta_{\rm L1}$ | 4 s |
| $T_{\rm prep}$ | 25 min |
| $T_{\rm build}$、$T_{\rm fin}$ | 不给示例值 |
| 决定 9 第一项 | 未发现的时钟损坏也计入 $\mathrm{Clk}^{\rm bad}_s$ |
| 决定 9 第二项 | 不给判据，下线作为行为假设；L1 守卫列为可选判据 |
| 决定 9 第三项 | 下线即关闭当前实例、不再进入新实例；时钟恢复后可在之后配置的 session 中重新参与 |
| K17 | 未处理 |

