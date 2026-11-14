# Part II v4 / v4.1：从正文移出的版本差异信息

[v4.1 英文稿](sync_hotstuff_rr_part2_rotation_v4_1.tex) 是 v4 的写作修订版，协议不变。v4 正文里有一些"和以前版本相比"的说明，v4.1 已从正文删去，汇总在这里，供熟悉早期版本的读者对照。

## v4 相对 v3.x 的机制变化（原 v4 §1 "Scope of version 4" 等处）

- 跨地区 Force 只能由已最终确认、经授权的 MSIG 指令触发。共识层不再有自动故障检测器（AUTO）、交付截止时间、Force gap、冷却期、service-debt / Owed 账本，也不强制恢复地区资格。
- 地区资格只通过显式 L1 STATUS 记录改变。MSIG 不再附带隐含 OFF。
- Force 不必等待 Ready，也不要求新 session 先有输出；连续 Hard Force 被允许。
- 目标版本检查（stale MSIG 不执行）和严格的 pre-payload 截止规则从 v3 保留。
- NextLive 不再检查 Owed，只按固定地区顺序和当前资格选人。
- `CommitNextDATA` 只更新 accepted count，不再更新 service-debt、失败计数或超时。

## 从 v3.7 沿用的部分（原 v4 L111–113）

区域 DATA/DRAIN 引擎、完整的 Ready 校验、公共 FINAL 检查、按原始来源保存字节、逐 DATA 接受。v4 新增了有界 Ready 服务假设（`ass:ready-service`），但不从它推出委员会响应界或全局进度。

## 公共检查器的改变（原 receipt kernel、carrier 片段中的说明）

- 公共解释器改用 `CheckFINAL`：只检查 FINAL 签名、DATA 字节和哈希路径，不再验证发布者附带的 proposal / recovery 见证。区域内的 Vote、恢复和 Ready 仍做完整校验。
- EXPORT 的外层字段（chain、version、expected base、interval）降为提示。接受规则从"整条 EXPORT 要么全收要么全拒"改为逐 DATA 原子接受；外层提示错误的记录仍可贡献其中有效的 DATA。v4 原文称之为相对 baseline 解释器的 "explicit public-checker refinement"。
- 发布输入仍是完整区域结果；$T_{\rm pub}$ 不对仅含 kernel 的弱输入作承诺。

## v4.1 的写作改动（协议不变）

- 标题、页眉去掉版本号；正文不再出现 version 3.x / this version / baseline / retained from 等说法。
- 新增贯穿全文的例子（A/B/C/D，101–115，$G=108$），各节回到这个例子讲解。
- 每章开头说明本章解决什么问题；`ass:ready-service` 移到 Ready 一节，"Conditions for progress" 移到 liveness 一节；receipt kernel 移到全局确认一章；各 section 片段并入单个 tex 文件。
- 术语统一：Session Manager（不再叫 Cross-region Handler / manager）、branch version（不再写 beta）、target index（不再叫 slot / retry slot）；"retain" 只保留给 retained anchor 等专有含义，缓存和存储改用 keep / store。
- 三个外部服务时间假设（v4 的 2.3 发布层 $T_{\rm pub}$、2.5 L1 $T_{\rm L1}$、2.7 Espresso $T_E$）合并为一条 Assumption 2.5 "Bounded external services"，分 (i)(ii)(iii) 三项，条件逐项不变；"Espresso history and time"（时钟超前界 $\epsilon_E$、长期可取回、namespace 完整性）仍单列为 2.4。
- 硬编码的 "Section~8" "Protocol~4" "P6" 全部改为交叉引用。
- 红字 `\RR{}`（相对 Part I 的改动）原样保留，共 45 处。
- 所有 Protocol、Definition、Assumption、Proposition 与 v4 逐词对照：只改措辞，未改条件或步骤。

## v4.2（`sync_hotstuff_rr_part2_rotation_v4_2.tex`）：§6.1、Fig 1、§6.2、附录 B

- §6 引言：例子缩短，细节指向 §6.7（Concurrent candidates）。
- §6.1 重写为：组件列表 → 组件表（读什么 / 产出什么 / 不能做什么）→ 五步数据流 → 三条权限规则 (A1)–(A3)。去掉了 "three control roles" 与四行表格不一致的问题。
- Fig 1 改用新文件 `figures/cross_region_rotation_v4_2/modules.tex`。协议编号改为 `\cref`（旧图的 "Protocols 5–6 / Protocol 3 / Protocols 4, 1–2" 都是错的，应为 3/6/7、4、5/1–2）。补上：区域结果 → Session Manager → Publish Layer 的回路、已停止 program 的发布重试、执行层、直接交接材料。
- §6.2 "The multisignature control interface"：多签只有一个接口，往 L1 控制日志追加 **control update**
  $U=(\mathit{chain},c,\theta_a,S,f)$：一个激活时间、覆盖所有区域的完整 status map $S$、一个 force tag $f\in\mathbb N\cup\{\bot\}$。不带 reason。
  - `def:control-update`；`def:control-valid`（V1–V5），其中 (V5)：$\theta_a\ge\theta_a(U')+T_{\rm update}$，$U'$ 是上一条有效 update，所以激活时间沿日志严格递增、间隔至少 $T_{\rm update}$；L1 合约可以直接强制 (V3)(V5)。
  - `def:control-effect`：同一 prelude 到期的 update 按序号全部生效，map 依次覆盖；只有第一条 force tag 等于 $\beta_0$ 的执行 Force。正常情况每块最多一条到期，只有 Espresso 时间戳一次跳过超过 $T_{\rm update}$ 才会多条同块。
  - 结果标签：PENDING / APPLIED / FORCED / STALE，组成 outcome log $\mathcal E_q$；解释器只保存 $S_{\rm eff}$。
  - 推论：日志顺序即生效顺序；已上链的 update 不可撤回（Force 不可逆）；两次 Force 的阈值至少相隔 $T_{\rm update}$；过期 tag 不阻止 map。
  - `def:force-log`：Force entry $\phi=(c,q,\beta_0,A,\mathit{src},r')$ 与 Force log $\Phi_q$；$\tagg{Closed}[\beta]$ 即对应 entry 加证明。
  - 新段落 "Choosing $T_{\rm update}$"：响应速度与稳定性的取舍；间隔按 Espresso 时钟计，只有时间戳跟随真实时间时才约束真实时间。
  - 例子：$T_{\rm update}=20$，三条 update（FORCED、APPLIED、STALE）、一条违反 (V5) 的无效记录、以及时间戳跳跃导致两条同块生效。
- 参考策略移到附录 B（`app:policy`）：条件谓词、条件 → update 表（base map，无 carried tag）、发出规则（最早可行阈值、排期上限 $H$）、阈值与 `prop:policy-noninterference`。
- 语义变化（相对 v4.1）：
  - STATUS 与 MSIG 合并为一条 update；status map 是绝对值（覆盖全部区域），不再是逐条 ON/OFF。
  - 一条 update 的 map 和 force tag 共用一个激活时间。force tag 过期时 map 照常生效（与原 "independent STATUS still takes effect" 一致）。
  - 激活时间沿日志严格递增且间隔至少 $T_{\rm update}$；原文 "Forces need no gap / no minimum interval between Forces" 改为只有 $T_{\rm update}$ 这一个间隔。
  - Genesis 明确控制权限 $\mathcal K$、$T_{\rm update}$ 和初始 status map。
- 跟着改的位置：§1.4、§2.1 L1 表与 Ingesting / prelude / Genesis 段、`ass:committee`、Protocol 3 输入 (I2) 与 Step 1、Protocol 6 Steps 1–2 及其后一段、§7.1、§7.2、§6.6、`prop:explicit-force` 证明、§8.2/§8.4/§8.5/§8.6 的例子、§9 两处关于 Force 间隔的话、符号表、Fig 1 标签。

## v4.2：§6.3 起重写（Session Manager / session program 状态机）

结构（旧 §6.3–6.10 → 新 §6.3–6.7 + 附录 C、D）：

| 新位置 | 内容 | 来自 |
|---|---|---|
| §6.3 The Session Manager | 三个视图 $\mathcal V_R,\mathcal V_G,\mathcal V_L$；本地数据（baseline $S_G$、successor $S_T$、program table、duty table、evidence store）；Protocol 4 改为六条规则 Baseline / Fix / Start / Stop / Serve / Persist；三条不变量；事件表；"Why a baseline instead of candidates" | 旧 Session Manager 与 lifecycle |
| §6.4 The session program | 只做 regional BFT；通信表（同 $(\beta,r)$ 成员、Manager、Publish Layer 单向）；L1/Espresso 作为输入（Enter 时、引用、恢复授权都由 Manager 提供）；结果与 receipt 中继；四个状态 + Fig 2；与 Manager 的接口表；程序依赖的两条同步保证 | 新写 |
| §6.5 Fixing a handoff | NextLive、target index、历史选择、handoff packet、VerifyHandoffHistory；"Baseline state and races" | 旧 Handoff records + Concurrent candidates |
| §6.6 Synchronous start | CanEnter、`def:ready`、准备时间、ReadyBundle、新 `prop:sync-start`、Protocol 5 | 旧 Ready / Activation and bootstrap |
| §6.7 Synchronous stop | 关闭处理、Exit、三个命题（只留陈述） | 旧 Graceful close + retirement |
| 附录 C | 三个 retirement 命题的证明、反例、"Force before the first ReadyBundle" | 从 §6 移出 |
| 附录 D Node storage and source relay | 原 node storage 一节整体移入 | 从 §6 移出 |

语义与命名：

- program 状态改为 Assigned → Running → Closing → Terminated，另有 Assigned → Terminated 的 Cancel。去掉 Waiting / Ready 两个状态：Ready 现在只是 Assigned 程序可以启动的条件（`def:ready`）。Running 对应引擎 Active 或 Frozen；Closing 对应引擎 Completed，或在关闭门后的 Active/Frozen。
- 不再为预测的 Force 候选单独建程序。每个运行节点都把执行状态维持在 $\operatorname{last}(G)$（$S_G$），预期的下一个 regular 接手地区另外维持 $S_T$。Force 锚点总是 $\operatorname{last}(G)$，所以不需要预测候选。旧的 "Concurrent candidates" 一节和 candidate 图删除。
- ReadyBundle 接收步骤 (iii)：本地 baseline 状态对应同一锚点时可直接用；否则用包里的 $S_A$。包仍须带全量状态字节，$e+\Delta$ 界不依赖 baseline。
- 改名：RequestClose / `close_requested` → Close（Manager 的本地原子调用）；FinishClose → Exit；Ready/Enter event → Enter。"workers"、"persistent / control owner" 统一为 Manager 的 duty table。
- 程序不读 L1 / Espresso：$\rho$、$\xi$、$\tagg{AUTH}_{r,g}$、开放下界都由 Manager 提供；缺输入时程序等待，与 §3.4 缺依赖相同。
- 程序可单向推送结果到 Publish Layer；重试由 Manager 的 Serve 规则负责。
- Fig 1 第 3–5 行重画（Manager 视图与 baseline、三个程序状态框、区域引擎框、右侧"one-way push / Manager retries"）。新增 Fig 2 `figures/cross_region_rotation_v4_2/program_states.tex`，每条边引用对应子协议。
- 附录引用改为 "appendix X"（`\crefalias{section}{appendix}`），同时修正了附录 B 原来显示为 "section B" 的问题。

跟着改的位置：§3.4 Admission 与 Engine states、§4 退休门、§5 例外说明两处、§6.1（组件列表、组件表、数据流、(A3)、Fig 1 标题）、§7 引言与 export session 段、§8 Protocol 7 步骤 2、§8.4/§8.5 例子、`prop:program-isolation` 证明、`prop:ready-bound`、§9 发布追踪段、附录 D 两处。

## v4.2：符号清理（全文）

去掉的记号（只是派生量，或已被别的记号取代）：

- outcome log $\mathcal E_q$ 及 PENDING / APPLIED / FORCED / STALE 表：每条 update 是否执行 Force 可以由 $\Lambda$ 加 replay 直接算出，没有任何规则读它。改成两句：到期 update 的 map 总是生效；只有 $\Lambda_q$ 里第一条 tag 等于 $\beta_0$ 的 update 执行 Force，其余 tag 为 stale（`def:control-effect`）。
- Force log $\Phi_q$、Force entry $\phi$：内容已在 $\mathrm{Out}_q$ 的 tail / session record 里（cause、新版本、$r'$、$i_{\rm src}$、$A$），序号在 $\Lambda$ 里。新增 `def:closure`：closure record $\mathsf{Closed}[\beta_0]=(U,\Sigma_U,J_q\text{ 头},\text{控制前缀})$。Protocol 6 第 2、3 步、$\mathcal V_L$、Fig 1、6.1 表格随之修改。
- $\vartheta(B)$（未使用）；$\eta$（不等式已是严格的）；$C_{\rm pub}$（只留文字）；$G_u,R_u$ 的下标 $u$；§7.1 的旧 MSIG 元组 $(\mathrm{chain},\beta_{\rm target},\theta_a,c)$ 与 $\beta_{\rm target}$（改为 $U$ 的 $f$）；$\beta_{\rm entry}$（$=\beta_0$）；$i_{\rm next}$（$=i_{\beta',r'}$）；$\mathsf{Closed}_v$；$t_{\rm open}$（$=t_0$）；$q_b$（$=q$）；$\Pi_A$；$d_v$（直接写 $c_v+4\Delta$）。
- 源上下文 $(i_b,\beta_b,r_b)$、$a_b$：统一用 $\mathrm{Out}_q$ 的 $(\beta_m,r_m,i_m)$、$a$ 和 tail 的 $i_{\rm src}$、$r'$。

合并：

- $S_{\rm eff}$ 与 $\chi_E(q)$ 合并为 $S_q$（$J_q$ prelude 之后的有效 status map，§2.1 定义）。$\mathsf{NextLive}(i,S)$ 在指定 authority 下检查 ON 且权重为正。$\xi(B)$ 绑定的也是 $S_q$。

冲突修正：

- $S$：执行状态改为 $\mathit{st}(B)$（block $B$ 之后的执行状态）。baseline state $=\mathit{st}(\operatorname{last}(G))$，successor state $=\mathit{st}(\hat B)$，ReadyBundle 里是 $\mathit{st}(A)$；Fig 2（manager baselines）同步。$S$ 只表示 status map。$\mathsf{CanEnter}(P)$、$\mathsf{VerifyHandoffHistory}(P)$ 去掉了 $S$、$C$、$\mathcal H$ 参数。
- $K$：kernel 不再叫 $K$，写成 $\mathsf{CheckFINAL}(C,B,D,\Sigma,\pi)$。
- $q$：Protocol 2 第 1 步直接用 $m_{\beta,r,g,t}$（drain generation 已有 $m=1$）。
- $D_q$ → $\Lambda_q$；附录 B 的排期上限 $H$ → $T_{\rm horizon}$；附录 D 命题里的时间 $a$ → $t_E(q)$；附录 C 的 replica $H_0,H_1$ → $v_0,v_1$。

顺带修正的旧内容：§6 引言 "What B does at the same time" 的表格仍写着候选 program $(8,21)$，改为"一个 program + baseline state"；`prop:program-isolation` 的 "Candidate preparation" 改为 "Baseline execution"。Notation 表相应更新（新增 $\Lambda_q$、$\mathsf{Closed}[\beta]$、$\mathit{st}(B)$、三个 view）。

## v4.2：删除 closure record，统一本地关闭标记

- 删除上一轮新增的 `def:closure`（closure record $\mathsf{Closed}[\beta]$）。公共层面，一个版本是否已关闭由 $\Lambda$ 加 Espresso header 的 replay 决定：replay 到 $J_q$ 的 prelude 时，已对它执行过 Force 即为关闭。Force 的其余信息仍在 $\mathrm{Out}_q$ 的 tail 里。§6.2 中原定义改为段落 "What a Force records"，例子里的 $\mathsf{Closed}[7]$ 一句删除；6.1 组件表、数据流、Fig 1 输出、$\mathcal V_L$、Notation 表去掉 closure records。
- 本地状态统一叫 **closed mark**（"$\beta$ is marked closed at the node"）。原文混用的 Closed record / Closed map / Closed flag / Closed gate / $\mathsf{Closed}[\beta]$ 全部改为这个叫法。§6.3 定义它：节点通过自己的 replay，或更早通过验证过的 close bundle 得知关闭，并持久化 closed mark；retirement gate 与 entry 检查测试这个标记，而不是 replay 位置。Local data 增加一项"closed marks 及其 closure evidence"。
- §6.7 Closure handling (ii) 写明 complete close bundle 的两部分：
  - closure evidence：执行 Force 的 update $U$、$\Sigma_U$ 及其 L1 finality proof；激活块 $J_q$ 的 header；从已验证 checkpoint 到 $U$ 的控制历史和激活 header，用来证明 $J_q$ 开始时版本为 $\beta$，且 $U$ 是 $\Lambda_q$ 中第一条 tag 为 $\beta$ 的 update；
  - snapshot context：session 的 authority、成员和历史准入上下文。
  接收方只凭这些字节就能验证，不需要自己的 replay 先到达 $J_q$。
- 跟着改的位置：§4 retirement gate、§6.6 CanEnter 与 Protocol 5 Enter、§6.7 开头与 (i)(iii)–(vi)、Exit、prop 6.8 与其后说明、Protocol 6 第 3 步（改为保留 $U$ 和 $J_q$ header 作为 closure evidence）、附录 C 三处、附录 D 两处。协议步骤不变。

## v4.2：Protocol 4 的 Fix 改为 Assign，并写明两种 handoff 的分配条件

- Protocol 4 第 2 条改为 **Assign**，分两种情况，各自写明 session id、anchor 和 $q^{\rm open}$：
  - (a) Force：$\mathcal V_G$ 新增一条 session record $\sigma=(\beta,r',i,A,H,q_s,\mathsf{FORCE}(q_f))$，且 $i$ 是本 region。session id $(\beta,r')$，$r'$ 按 target index 规则取 $r_m$（$a=0$）、$r_m+1$（$a>0$）或 pending boundary 存的值；anchor 是 $J_{q_f}$ 开始时的 $\operatorname{last}(G)$；$q^{\rm open}=q_s$。
  - (b) Regular：$\mathcal V_R$ 拿到 fixed session $(\beta,r)$ 终端 DATA 的完整 receipt，$\beta$ 未 marked closed，且从 $q_0=\max\{\mathrm{height}(\xi(T)),q^{\rm open}+1\}$ 起第一个让 NextLive 不为 $\bot$ 的已 finalized $q_s$ 选中本 region。session id $(\beta,r+1)$，anchor $T_{\beta,r}$，$q^{\rm open}=q_s$；不等 $G$，只在 $J_{q_0}$ 尚未 final 时等这个 header。
  - 共同部分：authority 为 $\mathsf{AUTH}_{r,0}$，$\lambda^{\rm open}$ 取 anchor 的 L1 引用高度与 authority 发布高度的较大者；用 VerifyHandoffHistory 验证；每个 pair 只分配一次；genesis 按 (a) 处理；收到新 pair 的已验证 ReadyBundle 时按同样方式分配后立即进入。
- Baseline 规则整理措辞（含义不变）；协议开头 "At espresso height q" 改为 "Below, q is the node's current replay position"。
- 新增段落 "Example: assigning sessions"：A 的 $(7,20)$ 以 110 结束、$\xi(110)$ 指向 $J_{500}$，B 按 (b) 分配 $(7,21)$（anchor 110，$q^{\rm open}=500$，此时 $G=108$）；update 41 在 $J_{501}$ 生效，B 按 (a) 分配 $(8,21)$（anchor 108，$q^{\rm open}=501$），并 Stop $(7,21)$；另给出 $a=0$ 时为 $(8,20)$、全部 OFF 时不分配的两种变化。
- 跟着改名：事件表（Assign (a)/(b)，ReadyBundle 行"新 pair 先 Assign"）、§6.4 状态转移、§6.5 races 段、`prop:program-isolation` 证明、6.1 数据流、Fig 1 Manager 框、Fig program states 的入口边（Assign，引用 Protocol 4）。
