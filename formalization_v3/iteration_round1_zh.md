# 第一轮迭代：Part II v4.6 → v4.7，Part III v3 → v4

日期：2026-10-24。依据：[Authority 接口分析](part2_part3_authority_interface_zh.md)。流程：修订 → 校对 → 补丁思路（第二轮见 [iteration_round2_zh.md](iteration_round2_zh.md)）。

## 0 产物

| | 正文 | PDF | 逐字 diff（红删蓝增）|
|---|---|---|---|
| Part II v4.7 | [tex](sync_hotstuff_rr_part2_rotation_v4_7.tex) | [pdf](sync_hotstuff_rr_part2_rotation_v4_7.pdf)（49 页）| [v4.6 → v4.7](../output/pdf/sync_hotstuff_rr_part2_rotation_v4_7_diff_v4_6.pdf) |
| Part III v4 | [tex](sync_hotstuff_rr_part3_staking_v4.tex) | [pdf](sync_hotstuff_rr_part3_staking_v4.pdf)（7 页）| [v3 → v4](../output/pdf/sync_hotstuff_rr_part3_staking_v4_diff_v3.pdf) |

- Part II 的图换用 `figures/cross_region_rotation_v4_7/`，只改了图注文字。
- 两份文件都编译通过，没有 overfull/underfull 和未定义引用。v4.6 里原有的未定义引用 `sec:late-receipt` 已改为指向 §7.1。
- 规模：Part II 改动 +443/−374 行（约 2.8 万词）；Part III 改动 +415/−353 行，3319 词。
- 旧版本文件保持不变。

## 1 Part II v4.6 → v4.7：改了什么

一句话：**Part II 只通过一个接口取用 authority。** 分支只携带一个 policy 对象。选区、委员会和 leader 都来自 Part III 的 plan。

| 位置 | 改动 | 为什么 |
|---|---|---|
| 摘要、§1.1 第 2–3 点、§1.2、表 1 | "region map"改为"branch policy"；后继由 plan 决定；Part III 通过一个小接口提供 plan、委员会和 leader | 读者第一页就知道两份文档怎样相接 |
| §1.3 末尾 | 修掉以 ", BLAME and Quit…" 开头的残句 | v4.6 遗留的编辑错误 |
| Def 2.1–2.3、表 3、Ingesting controls | $U=(\ldots,\beta_{\rm target},\mathcal P_{\rm new})$，$\mathcal P=(S,\mathsf{Ord},e^{\min})$；新增 (V6)，即 Part III 的 policy check；Force 执行时安装 $\mathcal P_{\beta'}$ | branch update 原先没有候选规则和最低快照字段（D3）|
| §2.1 "Authority interface"（替换 "Authority contexts and leaders"）| 定义 $\Pi_{\beta,r}=\mathsf{Plan}(\ldots)=\mathsf{AUTH}_{\beta,r,0}$：包含 region、委员会、TB/K/Δ、$\mathrm{id}(\Pi)$、$\lambda(\Pi)$，不包含 anchor；定义 $\mathrm{cid}$；$\mathsf{AUTH}_{\beta,r,g}$ 由 $\mathsf{REC}(\mathrm{cid},g)$ 给出；$L=\mathsf{Leader}(\mathsf{AUTH},t)$；WAIT 语义；"Part II reads no other staking state" | 统一 AUTH 的含义（D1），接口只在一处定义 |
| §2.1 "Choosing the next region"、Genesis | 删去 `NextLive`；region 取 $\mathsf{region}(\Pi)$；原循环规则保留为"default cyclic rule"，§7 的例子使用它 | 两套选区函数并为一套（D2、D7）|
| §2.3 Stake and faults | 成员和权重改为 plan 的委员会 $V_{\beta,r},w_o,W_{\beta,r},F_{\beta,r}$；补一句：委员会"仍被接受"的期间至迟到关闭其分支的 Force | 去掉 $\omega_{i,v}$ 与 PENDING($\omega$) 的记号冲突；故障假设的期限与退出规则对齐 |
| §3.2 等处 | quota 统一为常数 TB | 原先的 $\mathit{TB}_r$ 按轮次索引，与 Part III 不一致（D4）|
| §3.3 签名上下文、FINAL、Def 3.1、CheckFINAL | 签名绑定 (domain, cid, type)；FINAL 消息中去掉 cfg | REC 和 offense 按 cid 归属（F1）|
| Protocol 2 第 2–3 步、Changes 第 7 条、Ass 2.4、Ass 2.9(ii) | **所有类型**的 quit proof 都经 `SubmitQuit` 提交；等待 $\mathsf{REC}(\mathrm{cid},g+1)$；L1 服务界覆盖所有 L1 操作 | BLAME 型 quit 原先没人负责送上 L1（F2）|
| §5.2–5.4（session context、descriptor、Protocol 3）| $\nu$ 增加字段 $\Pi$；$C=(\nu,\Pi)$；Protocol 3 的初始化、Force 和 terminal 三处都调用 Plan；replay state 改为 $(\mathrm{Out},\mathcal P_\beta,\kappa)$ | CheckFINAL 所需的委员会可以从 $\nu$ 直接取得 |
| §6 规则、§6.1 新段 "Plans one session ahead"、§6.2、Def 6.1 | 后继的 plan 在前驱 boundary 固定时就能算出；handoff 与 Ready 都改用 plan | 把 Part III 的提前排班写进 Session Manager |
| Force-close | Force 本身不放款；Part III 的 release 用它作为关门点 | 两部分的退出语义对齐 |
| §7 | 例子注明使用默认循环规则；"map" 改为 "policy" | 例子本身无需重写 |
| 附录 A（Prop A.1/A.3/A.4/A.5）| fixed-committee；"fixed-policy plans are deterministic"；"Part III never changes β" | 证明用到的契约写明 |
| 附录 B 参考策略 | $S^\circ$ 改为 $\mathcal P^\circ$；新增 `Retire` 条件和 WITHDRAW 行；新增 (R7)：每次 update 默认把 $e^{\min}$ 抬到最新已激活快照；R5 在新快照下检查 | 退出依赖 Force，但多数 Force 本来就会发生 |
| 附录 C | L1 对象包括 staking 记录；position 被 Released 后不再负有存储义务 | 退出成员的数据义务（Q6）|

**未改：** 四个 view 编号、Force 截止与保留 $G$、Protocol 1/2/5 的 quit/lock/timer 规则、全局规则的结构、Ready/Enter、所有命题的陈述。

## 2 Part III v3 → v4：改了什么

一句话：**先给接口，再分三块：L1 账本、纯函数、L1 入口。** 同时修正了三处接口缺陷。

**结构**

| v3 | v4 |
|---|---|
| §1 Data and selection interfaces（数据对象与选择函数混在一起）| 第 1 页：核心思路框（5 条）+ 接口表（Part II 调什么、回报什么）+ $\mathrm{cid}$ |
| — | §1 Sessions and anchors：$E_s=A_p$、$A_s=T_p$、首个 session $E=A$；贯穿例子 |
| §2 Stake lifecycle and liability | §2 L1 staking ledger：状态表、**生命周期图**、迁移表、snapshot cut 规则 |
| §1.2 + §3.1–3.3 的选择与 authority | §3 Deriving authority：policy → plan（**单一读取高度 $\ell$**）→ 各代 authority 与 leader → 例子表 |
| §2.2–2.3 slashing 与提款 | §4 Accountability and release：**相对验证**、SubmitOffense、SubmitQuit、`Eff`、ProveForce、清退水位、Release |
| §3.4 Part II interfaces（放在最后）| §5 Guarantees and costs：G1–G7、Part II 的义务、对 Part I 假设的说明、接受的代价 |
| §3.1 Prepare/Bind/Enter（重述 Part II 流程）| 删除；时机只在 §1 和 G3 中说明 |

**规则变化**

| 变化 | 内容 | 性质 |
|---|---|---|
| REC 按 $(\mathrm{cid},g)$ 唯一，L1 只做相对验证 | 原先按 $(\beta,r,g)$ 唯一，可被伪造上下文抢注 | 修复缺陷（F1）|
| 释放改为 `Eff` + Espresso header + 清退水位 | 原先要求重放 Part II（$G$、Tail 等）；现在 L1 只看控制日志和一个 header | 简化（F4）|
| 候选规则 $\mathsf{Ord}(\mathit{cause},r,i_{\rm src})$，默认循环 | 原来的 randList/boot list 成为非默认实例 | 修复 Force 语义（F3）|
| 读取高度 $\ell=\max\{\lambda(E),a_{e^{\min}}\}$ | 快照编号与原式 (1) 等价 | 表述 |
| $g=0$ 的禁任集取 $\mathsf{Ban}(\ell)$ | 原先取快照截止时的 $J_e$；现在排除只滞后"L1 最终化 + 约两个 session" | **设计变化**（接口文档 Q1）|
| SubmitQuit 若附带双签，先禁任再冻结 REC | 引起 quit 的双签 leader 在下一代就被排除 | **设计变化**（Q2）|
| 有效 offense 使该 position 在下一个快照起强制退役 | 原先只禁止它当 proposer | **设计变化**（Q4）|
| region 资格要求至少一个未被禁任的正权重成员 | 避免 $g=0$ 时没有 leader | 边角修补（P6）|
| 记号 | $J\to\mathsf{Ban}$，$m_\beta\to e^{\min}_\beta$，截止提前量 $P\to N$ | 避开与 Part II 的符号冲突 |
| 接受的代价 | 补充：L1 引用的选择在 epoch 边界附近会影响快照选择；"adoption lags about one session" | 补全 |

三处设计变化（Q1、Q2、Q4）都可以单独回退。

## 3 校对：从整体看

### 3.1 抽象是否清晰、干净

**是，接口的形状对了。**

- 依赖单向：Part II 调用 `Plan` 和 `Leader`，读 plan 的几个字段，回报 quit proof 和冲突签名。
- Part III 拥有全部 staking 状态。
- $\mathcal P$ 在 Part II 里是只存、只传的不透明对象。

两份文档读下来，各自都能单独理解：Part II 用默认规则讲完全部例子，Part III 只引用 Part II 的 anchor、cid 和 Force-close。

**不够干净的地方：**

1. **候选规则的接口太虚。** $\mathsf{Ord}$ 被写成"genesis 固定的一族确定性规则"，输入是 $(\mathit{cause},r,i_{\rm src})$，而这一族究竟包含什么，没有定义。policy check 里"its rule belongs to the family"因此无法检查。`cause` 这个输入也只是为它而存在。这是本轮最大的不干净之处。
2. **同一个对象有两个名字**：$\Pi$ 和 $\mathsf{AUTH}_{\beta,r,0}$。可以接受，因为 Part II 一直用 AUTH 记号，但两份文档都应只在一处说明二者等同。
3. **descriptor 有冗余字段。** $\nu=(\beta,r,i,A,H,q^{\rm open},\mathit{cause},\Pi)$ 中，$i$ 和 $H$ 已由 $\Pi$ 和 $A$ 决定。保留是为了照顾可读性和示例，应在正文注明。

### 3.2 抽象是否合理

**合理。** 三个关键判断都成立：

- 提前一棒排班（$E_s=A_p$）不需要任何 L1 交易；
- L1 只做相对验证：L1 本来就无法知道跑在 $G$ 之前的 session，不验证真实性反而是唯一可行的选择；
- 退出关门只依赖 L1 可算的 `Eff`：控制日志的目标序列决定了每条 update 是否执行。

**缺两条显式假设：**

1. **控制日志在 L1 合约中，由合约检查有效性（V1–V6）并计算 `Eff`。** Part II 只说"finalized L1 history 中的有效 update"，没有说 L1 上谁来判定；Part III 的 `ProveForce` 依赖这一点。
2. **L1 上有 Espresso light client，用来验证最终化的 header。** 会议记录确认这条路径存在，但它较慢；它的延迟只推迟 $t_c$，不影响 $T_{\rm hold}$ 的论证。正文应写明。

### 3.3 能否更简单

可以，主要有一处：**把候选规则换成每个分支一条循环顺序** $\mathcal L_\beta$。

- 候选列表：先是 $\mathcal L_\beta$ 中 $i_{\rm src}$ 之后的 region，最后是 $i_{\rm src}$ 本身；regular 与 Force 用同一条规则。
- 由此去掉的东西：`cause` 参数、"规则族"、Ord 与 Boot 的区分。
- policy check 变成纯结构检查：$\mathcal L_\beta$ 是所有 region 的一个排列；$e^{\min}$ 不减且已激活。
- 会议记录的两个例子都能表达：$\mathcal L_7=(A,C,B,D)$ 时，A 之后依次是 C、B；$\mathcal L_8=(A,B,C,D)$ 时，A 之后是 B。
- **代价**：无法再做"每个轮次换一个排列"（v3 的 randList 随 $r$ 变化）。但 ON/OFF 表和逐分支的顺序已经覆盖了已知用途，leader 本身仍按权重随机选出。以后确有需要，可以作为 policy 的扩展加回。

其余可简化的地方很小：Part II §2.1 的接口段过长，拆成几个短段会好读；Part III 的接口表与 §5 的保证表有少量重复，但两者分别回答"调什么"和"保证什么"，保留。

### 3.4 局部问题

- Part II §2.1 第一段引用 `\cref{sec:authority-context}`，排出来是"section 2.1"，即引用自己。
- Part II (R7) 写的是"so every Force clears retired stake"，不准确：每次 Force 只推进清退水位，放款还要经过 `ProveForce` 和 $T_{\rm hold}$。同一条里还混进了 BOOT/STALL 的说明，应拆开。
- Part III 没有解释 "export session"，需要加一句注释。
- Part III 接口表把 $\mathsf{AUTH}_{\beta,r,g}$ 称为"the proposer set of generation $g$"，不准确，应为"plan plus the generation's ban set"。

## 4 补丁思路 → 第二轮（Part II v4.8，Part III v5）

| # | 补丁 | 涉及 |
|---|---|---|
| R2-1 | **候选规则改为逐分支循环顺序** $\mathcal L_\beta$：$\mathcal P=(S,\mathcal L,e^{\min})$；`Plan` 去掉 `cause`；(V4) 检查 $S$ 和 $\mathcal L$ 的完整性，(V6) 只检查 $e^{\min}$ | II §2.1、Def 2.1/2.2、表 3、Protocol 3、§6.2、§7、附录 B；III §3 |
| R2-2 | 写明两条假设：L1 控制合约检查有效性并计算 `Eff`；Espresso L1 light client | II §2.1 Ingesting controls；III §4.2、§5 |
| R2-3 | Part II 接口段拆成短段并加小表"Part II 从 plan 读什么"；修正自引用 | II §2.1 |
| R2-4 | (R7) 改写为"推进清退水位"，BOOT/STALL 的说明移到表后 | II 附录 B |
| R2-5 | Part III：注释 export session；AUTH 措辞；例子改用 $\mathcal L_7$、$\mathcal L_8$ | III 第 1 页、§1、§3.4 |
| R2-6 | 注明 descriptor 中的 $i$、$H$ 由 $\Pi$、$A$ 决定 | II §5.3 |

## 5 本轮未处理

- **Part IV v0.1 仍是旧接口**（$S_{\rm new}$、`NextLive`）。它的 Protocol 1 复述了 Part II 的 Force 前奏，需要做一次机械同步：$\mathcal P_{\rm new}$、Plan、TB。按本次要求只迭代 II 和 III，所以没有改。
- `build.sh` 和 README 仍指向 v4.6 和 v3，等你确认后再更新。
