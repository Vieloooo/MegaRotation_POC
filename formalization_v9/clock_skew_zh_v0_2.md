# 多时间域模型：副本、L1 与 Espresso 各自漂移（v0.2）

日期：2026-11-07。上一版：[v0.1](clock_skew_zh_v0_1.md)。对象：本目录当前规格（00–08 v0.1，09、10 v0.2）。本文只做分析，不改动规格原文；安全问题见 [时钟偏差安全审查 v0.2](clock_skew_review_zh_v0_2.md)。

本版改动：

- 出发点改为：副本、L1、Espresso 三方各自相对真实时间漂移，漂移量互不相同。v0.1 只让副本时钟漂移。
- 新增 L1 timestamp 的偏移界 $\delta_{\rm L1}$，并把 00 §1.2 的 fixed-pace 改为按 header timestamp 读取（§1.3）。
- Espresso 的 E-TS 拆成超前、滞后两个方向的界 $\delta_E^+,\delta_E^-$，并与 $\mathsf{Live}_E$ 脱钩，用来讨论“live 但时间错”的情形（§1.4）。
- 列出所有跨时间域的比较（§3），给出每项预算对各方向漂移的容忍量（§4）。
- 副本时钟域的内容（v0.1 §1.2–§1.3 的 C1–C5、L-CLK，§3.2–§3.3 对 02、03 的修改）不变，本文直接引用。

新增条件一律标 〔补充〕，待你确认。

## 0. 结论

1. **Safety 不依赖任何一方的时间值，只依赖 Espresso timestamp 的单调性。** 全部时间键都是公开值，GFR 的确定性只用 E-S 与 L1 safety。唯一的例外是 watermark：它要求 $\theta_q\le\theta_{q+1}$。若 HotShot 不在共识规则中强制这一条，GFR 可以改用前缀最大值作键，代价为零（§5.1，审查 K11）。
2. **GFR 把三种时钟放在“同一坐标”上。** 时间键来自 Espresso timestamp $\theta$、L1 timestamp 加 $M_a$、L2 时间表 $\mathit{ts}^*$（由副本时钟实现）。每一处不同来源的键相比较，都要为两方的相对漂移留预算（§2、§3）。
3. **Espresso 时间是 GFR 的活性单点，包括治理。** Force、RESUME、SessionEnd、Expire 都要等到 Espresso watermark 越过各自的键才能处理。Espresso 的时间一旦停止前进（live 但 $\theta$ 不动），L1 治理在 GFR 中完全失效（审查 K15）。
4. **两个方向的后果不对称。**
   - Espresso **超前**：结果的公开时刻被高估，会被截止规则拒绝。代价是区域已决定、用户已看到的 block 被排除出 $G$。持续超前约 1 h，或一次跳变约 $W_{\rm exp}$，分支就会 HALT，只能 RESUME。
   - Espresso **滞后**：截止与治理在真实时间上推迟。代价是 S10 的期权窗口变长、Force 生效变慢、新窗口的准备时间被吃掉，以及超过 10 min 后 $G$ 可能赶不上原生截止。
5. **按 10 的参数，Espresso 超前的容忍量很小。** 超前 36 s 起，每个 GFR 事件被 L1 拖慢（$M_a$ 余量）；超前约 60 s 起，每个 session 末尾的结果被拒绝（$T_{\rm grace}$ 余量）。滞后的主要阈值是 $T_{\rm init}$ 的约 90 s 与 $W_{\rm seq}$ 的 10 min（§4）。
6. **L1 的 timestamp 本身很准，L1 的“慢”体现在 finalized 延迟。** 以太坊的 timestamp 由 slot 决定；真正的风险是 finality 停滞：区域生产在 $T_{\rm dep}-T_{\rm L1}$ 的余量用完后停止，$G$ 也随 R-WAIT 停止（审查 K17）。

## 1. 时间域

### 1.1 真实时间

$\tau\in\mathbb R_{\ge0}$ 只用于分析，没有参与方读它。服务预算（$T_{\rm pub}$、$T_E$、$T_{\rm L1}$、$T_{\rm batch}$ 等）都是真实时长。下面三个时间域各用一个**带符号误差**描述它与真实时间的关系：误差为正表示超前（快），为负表示滞后（慢）。

### 1.2 副本时钟域 $\mathsf{Clk}$

沿用 v0.1 §1.2：honest $m\in\mathsf{Clk}$ 的本地时钟 $C_m(\tau)$ 满足 C1（两两偏差 $\le\Delta_G$）、C2（$|C_m(\tau)-\tau|\le\delta_G$）、C3–C5；换算引理 L-CLK 不变。读这个域的只有 replica 与 Session Manager。

### 1.3 L1 时间域〔补充〕

**Timestamp 的来源。** 00 §1.2 的 fixed-pace 模型 $\mathit{ts}_{\rm L1}(\ell)=\tau^0_{\rm L1}+\ell b_{\rm L1}$ 把 height 与时间绑死。以太坊有 missed slots，height 落后于 slot。空 slot 只增不减：每 1 000 个空 slot，按公式算出的时间就比 header timestamp 早 3 h 20 min。改为：

- **L-PACE。** $\mathit{ts}_{\rm L1}(\ell)$ 取自 block $\ell$ 的 header，严格递增，且 $\mathit{ts}_{\rm L1}(\ell+1)-\mathit{ts}_{\rm L1}(\ell)\in\{b_{\rm L1},2b_{\rm L1},\ldots\}$。

规格中用到 $\mathit{ts}_{\rm L1}$ 的地方（$\ell^{\rm sch}$、O2、O3、$\mathrm{act}$、RESUME 的目标区间）都只需要严格递增与读取 header，不需要线性公式。“相邻 L2 slot 的 scheduled L1 height 至多前进 1”在有空 slot 时仍成立。实现不得用 height 推算 timestamp。

**偏移。** 令 $\tau^{\rm prod}(\ell)$ 为 block $\ell$ 的真实产生时刻，定义误差 $e_{\rm L1}(\ell):=\mathit{ts}_{\rm L1}(\ell)-\tau^{\rm prod}(\ell)$。

- **L-TS。** $|e_{\rm L1}(\ell)|\le\delta_{\rm L1}$。

以太坊上，honest attester 拒绝 timestamp 在自己时钟未来的 block，proposer 在 slot 开始后出块，所以 $e_{\rm L1}$ 通常在 $[-4\,{\rm s},+0.5\,{\rm s}]$ 量级（待按部署核实）。L1 时间远比 Espresso 时间可靠：要让它整体偏移，需要以太坊多数验证者的时钟一起出错。

**Finalized 延迟。** 00 §1.2 的“block $\ell$ 在 $\mathit{ts}_{\rm L1}(\ell)+T_{\rm L1}$ 之前 finalized”把 L1 时间与真实时间混用。改为以真实时刻表述：block $\ell$ 在真实时刻 $\tau^{\rm prod}(\ell)+T_{\rm L1}$ 之前 finalized 且可读。换回 L1 时间时多出 $\delta_{\rm L1}$。L1 的“慢”主要是 finalized 延迟超出 $T_{\rm L1}$，属于 L1 活性，不是 timestamp 误差（审查 K17）。

### 1.4 Espresso 时间域〔补充〕

**误差。** 定义 $e_E(q):=\theta_q-\tau_{\rm prop}(J_q)$，其中 $\tau_{\rm prop}$ 是 $J_q$ 的真实提出时刻。

- **E-TS′。** 在时间正常的区间内，$-\delta_E^-\le e_E(q)\le\delta_E^+$。

现行 E-TS 是对称的特例：$\delta_E^+=\delta_E^-=\Delta_{\rm ts}$。拆成两个方向，是因为两个方向破坏的预算不同（§4）。

**与 $\mathsf{Live}_E$ 脱钩。** 现行规格只在 $\mathsf{Live}_E=1$ 时使用 E-TS，non-live 时不假设 timestamp 贴近真实时间。这把“出块”与“时间准”合在一个状态里，无法表达本文要讨论的情形：Espresso 正常 finalize，但 timestamp 持续超前、滞后、跳变或冻结。本文把两者分开：

- $\mathsf{Live}_E$ 只表示 E-L 成立；
- E-TS′ 是另一项假设，它不成立的区间称为**时间故障**。

时间故障不是公共可知的状态。参与方只能拿 $\theta$ 与自己的时钟、L1 timestamp 比较来发现它（审查 K16）。协议规则不读 $\mathsf{Live}_E$（它只出现在活性陈述中），所以这一拆分不改变任何规则。

**单调性。** E-S 中的 $\theta_q\le\theta_{q+1}$ 是 safety 条件，对所有执行都要求成立，与 E-TS′ 无关。它是否由 HotShot 的共识规则强制，待核实（§5.1）。

### 1.5 记号小结

| 时间域 | 读数 | 相对真实时间 | 谁读它 |
| --- | --- | --- | --- |
| 副本时钟 | $C_m(\tau)$ | $\pm\delta_G$，两两 $\Delta_G$ | replica、Session Manager |
| L1 时间 | $\mathit{ts}_{\rm L1}(\ell)$ | $\pm\delta_{\rm L1}$ | 公开值；GFR、Authority、schedule |
| Espresso 时间 | $\theta_q$ | $[-\delta_E^-,+\delta_E^+]$ | 公开值；GFR、Publisher 的停止判断 |
| L2 时间表 | $\mathit{ts}^*(h)$ | 公开值，由副本时钟实现 | 全部 |

## 2. GFR 的单一坐标

[09 §3.1](09_finality_rules_zh_v0_2.md) 规定“所有键使用同一坐标”。五类事件的键实际来自三个时间域：

| 事件 | 键 | 键的来源 | 键在真实时间上的位置 |
| --- | --- | --- | --- |
| Espresso($q$) | $\theta_q$ | Espresso | 真实提出时刻 $+e_E(q)$ |
| Force、RESUME | $\mathit{ts}_{\rm L1}(\ell)+M_a$ | L1 | 收录 block 的产生时刻 $+e_{\rm L1}(\ell)+M_a$ |
| SessionEnd($s$) | $\tau_s^{\rm stop}+T_{\rm grace}$ | L2 时间表 | 副本在 $[\tau_s^{\rm stop}-\delta_G,\tau_s^{\rm stop}+\delta_G]$ 内停止，再加 $T_{\rm grace}$ |
| Expire($h$) | $\mathit{ts}^*(h)+W_{\rm exp}$ | L2 时间表 | 副本在 $\mathit{ts}^*(h)\pm\delta_G$ 生产 $h$，再加 $W_{\rm exp}$ |

GFR 只比较键，与真实时间无关，所以重放结果总是确定的。但每条规则的**用意**都写成真实时间的话：“停止前交出的结果赶得上截止”“Force 在激活后生效”“过期的高度已经来不及进入 $D$”。这些话成立，要求被比较的两个键所在的时间域之间偏差有界。一条规则比较的是哪两个域，决定了它对哪种漂移敏感。

R-WAIT（[09 §3.4](09_finality_rules_zh_v0_2.md)）还把三个域耦合到处理进度上：处理键 $x$ 之前，L1 finalized 前缀要覆盖 $x-M_a$，Espresso watermark 要覆盖 $x$。前者使 Espresso 超前时 GFR 被 L1 拖住；后者使 Espresso 滞后或冻结时，所有非 Espresso 事件都推迟。

## 3. 跨域比较清单

“域”一列写出被比较的两方；E 为 Espresso，L 为 L1，C 为副本时钟。

| # | 位置 | 比较 | 域 | 现行预算 | 新预算〔补充〕 |
| --- | --- | --- | --- | --- | --- |
| X1 | 02 Build、Accept | 本地时钟对 $\mathit{ts}^*$ | C | 完美时钟 | v0.1 §3.2 |
| X2 | 00 §3.4 scheduled origin | 到达 slot 时 $\ell^{\rm sch}(h)$ 已 finalized | C–L | $T_{\rm L1}\le T_{\rm dep}$ | $T_{\rm L1}+\delta_G+\delta_{\rm L1}\le T_{\rm dep}$ |
| X3 | 09 SessionEnd，04 PUB-4 推论 | 停止前的结果 $\theta<\theta_s^{\rm end}$ | C–E | $T_{\rm grace}>T_{\rm pub}+T_E+\Delta_{\rm ts}$ | $T_{\rm grace}>T_{\rm pub}+T_E+\delta_E^++\delta_G$ |
| X4 | 09 Expire | 按时生产的 block 在 $\mathit{ts}^*(h)+W_{\rm exp}$ 前公开 | C–E | 未单列（$W_{\rm exp}=10$ h 远大于时延） | $W_{\rm exp}$ 大于区域决定与发布的时延，再加 $\delta_E^++\delta_G$ |
| X5 | 09 R-WAIT 第一条 | Espresso 事件 $\theta$ 的处理不被 L1 拖慢 | E–L | $M_a\ge T_{\rm L1}+2\Delta_{\rm ts}+b_{\rm L1}$ | $M_a\ge T_{\rm L1}+b_{\rm L1}+\delta_E^++\delta_{\rm L1}$（见下） |
| X6 | 09 §3.3 Force 切点 $q^*(\mathrm{act})$ | 哪些 Espresso 结果排在控制记录之前 | E–L | 无（只依赖 E-S） | 切点在真实时间上的位移为 $e_E-e_{\rm L1}$ |
| X7 | 09 §4.4 第 5 步，Force 新窗口 | 从得知 Force 到新窗口开始的准备时间 | L–E–C | “E-TS 成立时 $T_{\rm init}$ 是准备预算” | 准备时间 $\ge T_{\rm init}-T_E-\delta_E^--\delta_G-\delta_{\rm L1}$ |
| X8 | 01 §3.10 RESUME 目标 | 从得知 RESUME 到 $h_{\rm res}+1$ 的准备时间 | L–E–C | $M_a+T_{\rm ready}\le T_{\rm prep}$ | $M_a+T_E+\delta_E^-+T_{\rm ready}+\delta_G+\delta_{\rm L1}\le T_{\rm prep}$（见下） |
| X9 | 09 §3.5 原生窗口，05 BAT-4 | 进入 $G$ 的 block 赶上 L1 截止 | E–L | $W_{\rm exp}+M_a+T_{\rm batch}+T_{\rm drift}\le W_{\rm seq}$ | 左边加 $\delta_E^-+\delta_{\rm L1}$ |
| X10 | 04 PUB-3 STOPPED | watermark 越过 $\theta_s^{\rm end}$ 后停止重试 | E | — | 不变 |
| X11 | 03 Kill（依据 $D$） | L1 位置 | L | — | 不变 |
| X12 | 审查 v0.1 K8(b) 公开时间守卫 | 本地时钟对 $\theta$、L1 head | C–E–L | — | 只参照 L1（审查 K19） |

**X5 的推导。** 设 Espresso 事件的键为 $x=\theta_q$。$J_q$ 最早在真实时刻 $\tau_{\rm prop}=x-e_E(q)$ 可读。R-WAIT 需要 timestamp 不小于 $x-M_a$ 的 L1 block 已 finalized；第一个这样的 block 的 timestamp 小于 $x-M_a+b_{\rm L1}$，在真实时刻 $x-M_a+b_{\rm L1}-e_{\rm L1}+T_{\rm L1}$ 之前 finalized。L1 不拖慢处理，当且仅当

$$
e_E(q)-e_{\rm L1}\le M_a-T_{\rm L1}-b_{\rm L1}.
$$

所以起作用的是 **Espresso 相对 L1 的超前量**，取最坏情况即得表中的式子。按此推导只需要一个 $\Delta_{\rm ts}$。现行式中的 $2\Delta_{\rm ts}$ 多出一项，规格里没有找到它的来源；若它针对 Espresso 滞后方向的某类事件，应改写为 $\delta_E^++\delta_E^-$。待你确认。

**X7、X8 的推导。** Force、RESUME 都不是 Espresso 事件，处理前要等 timestamp 不小于 $\mathrm{act}$ 的 Espresso watermark。滞后 $\delta_E^-$ 时，这份 watermark 最晚在真实时刻 $\mathrm{act}+\delta_E^-+T_E$ 可读（E-L）。Manager 由此才得到 SessionAssigned。新窗口的快时钟副本在 $\mathit{ts}^*(h_{\min})-\delta_G$ 开始，而 $\mathrm{act}$ 作为真实时刻又有 $\delta_{\rm L1}$ 的误差。现行 X8 的预算没有计入 watermark 等待 $T_E+\Delta_{\rm ts}$（按 10 的参数为 45 s）；若 $T_{\rm ready}$ 已包含它，只需加 $\delta_G+\delta_{\rm L1}$。待你确认。

**X9 的推导。** Expire 的键在 Espresso 坐标上。Espresso 滞后 $y$ 时，GFR 处理到键 $\mathit{ts}^*(h)+W_{\rm exp}$ 的真实时刻推迟 $y$，block 进入 $G$ 的最晚真实时刻也推迟 $y$。原生截止在 L1 坐标上，从 origin 起算。

## 4. 各方向漂移的容忍量

按 10 的参数计算：$M_a=20$ min、$T_{\rm L1}=19.2$ min、$b_{\rm L1}=12$ s、$T_{\rm grace}=2$ min、$T_{\rm pub}=T_E=30$ s、$T_{\rm init}=2$ min、$W_{\rm exp}=10$ h、$W_{\rm seq}$ 余量 10 min、$\mathrm{TB}=1$ h。$\delta_G,\delta_{\rm L1}$ 记为小量，表中不展开。

### 4.1 Espresso 超前（$e_E=+x$）

| 超前量 $x$ | 首先失效的预算 | 后果 |
| --- | --- | --- |
| $\le15$ s | 无（模型内，$\Delta_{\rm ts}$） | — |
| $>36$ s | X5：$M_a-T_{\rm L1}-b_{\rm L1}=36$ s | 每个 Espresso 事件的处理被 L1 拖慢 $x-36$ s；$G$ 的确认延迟同样增加 |
| $>60\,{\rm s}-\delta_G$ | X3：$T_{\rm grace}-T_{\rm pub}-T_E=60$ s | 每个 session 末尾约 $x-60$ s 的区域结果错过截止，被排除出 $G$；后继在错误候选上的工作也作废（审查 K12） |
| $\gtrsim\mathrm{TB}+T_{\rm grace}$（约 1 h） | X3 覆盖整个 session | 每个 session 的结果都错过自己的截止，$G$ 不再增长；约 $W_{\rm exp}$ 之后下一缺失高度过期，分支 HALT，只能 RESUME |
| 一次跳变 $X\gtrsim W_{\rm exp}$ | X4 | 不必持续：跳变之后的第一个结果已晚于过期键，直接 HALT（审查 K13） |

X6 方向：切点在真实时间上提前约 $x$。Force 激活前真实时间最后约 $x$ 秒内公开的旧 session 结果被排除。

### 4.2 Espresso 滞后（$e_E=-y$）

| 滞后量 $y$ | 首先失效的预算 | 后果 |
| --- | --- | --- |
| $\le15$ s | 无 | — |
| 任意 $y$ | R-WAIT 第二条 | Force、RESUME、SessionEnd、Expire 的处理推迟约 $y$；Espresso 事件本身不推迟 |
| 任意 $y$ | X3 的反面 | 截止在真实时间上推迟 $y$，S10 的期权窗口从 $T_{\rm grace}$ 变为 $T_{\rm grace}+y$ |
| $>90\,{\rm s}-\delta_G$ | X7：$T_{\rm init}-T_E=90$ s | Force 开出的窗口在 Manager 得知前就已开始，开头的 heights 变为 padding |
| $>0$ | X8：下界余量本来为 0 | RESUME 的新 session 来不及在 $h_{\rm res}+1$ 出块 |
| $>10$ min | X9：$W_{\rm seq}$ 余量 | 在过期边缘才进入 $G$ 的 block 可能赶不上原生截止，$D$ 补入 deposit-only，之后 RESUME 撤销 $G$ 的后缀 |
| 无界（冻结） | 全部非 Espresso 事件 | 治理锁死（审查 K15） |

X6 方向：切点在真实时间上推迟约 $y$，旧 session 在 Force 激活后仍能扩展 $G$ 约 $y$ 秒。

### 4.3 L1

$\delta_{\rm L1}$ 只在 X2、X5、X7、X8、X9 中以小量出现。按 10 的余量（$T_{\rm dep}$ 48 s、$M_a$ 36 s、$W_{\rm seq}$ 10 min），以太坊秒级的 timestamp 误差不构成约束。L1 的实质风险是 finalized 延迟超出 $T_{\rm L1}$（审查 K17）。

### 4.4 副本

与 v0.1 §3.8 相同。新增的交叉项只有 X3 的 $\delta_G$ 与 $\delta_E^+$ 相加：两者合计不得超过 60 s。

## 5. 设计方向〔补充〕

### 5.1 用前缀最大值作 Espresso 键

定义

$$
\bar\theta_q:=\max_{q'\le q}\theta_{q'}.
$$

GFR 的键、watermark 与 $q^*$ 都改用 $\bar\theta$。$\bar\theta$ 由公开前缀确定，按构造不递减，所以 watermark 的正确性不再依赖 HotShot 是否强制单调。若 HotShot 已强制单调，$\bar\theta=\theta$，规则不变。这一项没有代价，建议直接采用。

### 5.2 用 L1 夹住 Espresso 键

**前提（待核实）。** 每个 Espresso block 携带一个由 HotShot 验证的 L1 引用 $\lambda_q$（例如 header 中的 L1 finalized 引用），且 $\lambda_q$ 不递减、不超过验证者看到的 L1。

**规则。** 给定公开常数 $K^+,K^-$，令有效键为

$$
\hat\theta_q:=\max_{q'\le q}\ \min\bigl\{\max\{\theta_{q'},\ \mathit{ts}_{\rm L1}(\lambda_{q'})+K^-\},\ \mathit{ts}_{\rm L1}(\lambda_{q'})+K^+\bigr\}.
$$

$\hat\theta$ 仍是公开前缀的函数，且不递减。效果如下：

- **超前被上界截住。** 有效键不超过 L1 时间加 $K^+$，单个 timestamp 跳变的影响不超过 $K^+$。$K^+$ 要覆盖 L1 finalized 延迟，所以只能截住 20 min 以上的超前，主要保护 $W_{\rm exp}$ 量级的 HALT（审查 K13）。60 s 量级的超前仍要靠 HotShot 自身的时间戳校验。
- **冻结被下界顶住。** 只要 $\lambda$ 还在前进，有效键就随 L1 前进。$\theta$ 停止时，Force、RESUME 与各定时器仍能被处理（审查 K15）。
- **剩余风险。** 若 Byzantine 的 Espresso proposer 能提交陈旧的 $\lambda$，下界失效，此时的防护取决于 HotShot 是否限制 $\lambda$ 的陈旧程度。

**代价。** 有效键在正常时要等于 $\theta$，需要 $K^-\le\theta_q-\mathit{ts}_{\rm L1}(\lambda_q)\le K^+$ 在正常运行中成立。这要求标定 L1 引用的正常滞后。

### 5.3 让治理不依赖 Espresso 时间

即使不采用 §5.2，也可以只为控制记录加一个 L1 侧的超时：若 L1 finalized 前缀越过 $\mathrm{act}+K_{\rm gov}$ 时，仍没有 timestamp 不小于 $\mathrm{act}$ 的 Espresso watermark，就按某个由 L1 决定的切点处理 Force 或 RESUME。困难在于切点必须是公开前缀的确定函数。没有 $\lambda_q$ 时，读取者无法一致地确定“哪些 Espresso blocks 排在 L1 时刻之前”。因此 §5.3 也依赖 §5.2 的前提，只是用途更窄。

### 5.4 只参数化

不改规则，只采用 §3 的新预算，并在部署中监控 $\theta$ 与 L1 timestamp、本地时钟的偏差（审查 K16）。这样只覆盖模型之内的漂移。

## 6. 落地方式〔待确认〕

需要改动的规格位置：

- 00 §1.2（L-PACE、L-TS、finalized 延迟改用真实时刻）、§1.4（E-TS′ 与 $\mathsf{Live}_E$ 脱钩，单调性）、§3.4（X2）；
- 01 §3.10（X8）；
- 04 PUB-4 推论（X3）；
- 09 §3.1（若采用 §5.1 或 §5.2，改键）、§3.5（X4、X5、X9）、§4.4 第 5 步的说明（X7）；
- 10 §2.3 的数值。

v0.1 列出的副本域修改不变。与 v0.1 一样，可以出新版本文件，也可以就地修改；并与 Accept 修法一起定。
