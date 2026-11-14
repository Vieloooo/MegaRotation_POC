# 模块 11：全局链 $G$ 的安全性与活性（v10 工作稿 v0.1）

本文把 [02 Session Program](02_session_program_zh_v0_1.md)、[03 Session Manager](03_session_manager_zh_v0_1.md) 和 [09 Global Finality Rule](09_finality_rules_zh_v0_1.md) 的性质接起来。安全性讨论读取者如何从公共历史得到唯一的 $G$，以及交接如何固定下一段的起点。活性讨论何时有新的区域结果进入 $G$，以及停顿后何时能够恢复生产。全局确认指 GFR 将内容接入 $G$。原生派生链 $D$ 的结算性质见 [09 §5–§7](09_finality_rules_zh_v0_1.md)。

## 1. 安全性

### 1.1 公共输入与重放位置

读取者使用同一 genesis、规则版本和配置。L1 日志提供 Force 和 RESUME，Espresso 日志提供每个 finalized Espresso block $J_q=(q,\theta_q,\mathcal M_q)$。当前重放状态另外生成两个候选事件，即对 $G$ 下一缺失高度的 Expire，以及对当前持有 NEXT 的 session 的 SessionEnd。重放位置按时间键、事件类别和索引比较。同键时按「L1 控制 → Expire → SessionEnd → Espresso」处理。见 [09 §3.2–§3.3](09_finality_rules_zh_v0_1.md)。

### 1.2 公共重放一致性

**命题 G-S1（同一重放位置的全局链一致）。** 固定同一公共执行、genesis、规则版本和配置。在下列前提下，任意两个遵守 GFR 的读取者，其已完成的事件序列前缀可比。若两者处理到同一公共重放位置，则得到相同的 $G$、当前分支、职责和选定的 descriptor。

| 所用前提 | 在 G-S1 中的用途 |
| --- | --- |
| L1 的 finalized 前缀一致且不回退，控制记录可按 finalized L1 位置核对，L1 时间戳严格递增 | 完整覆盖 $x-M_a$ 的输入排除遗漏同键或更早控制记录。这里只使用公开时间戳的顺序，L1 的 fixed-pace、L-TS 和 finalized 延迟界用于第 2 节的时间预算。 |
| E-S 的 finalized 前缀一致且不回退、已读前缀的消息列表完整、$\theta_q\le\theta_{q+1}$ | 前缀一致固定日志内容，消息完整性固定每个位置有哪些公开对象，覆盖 $x$ 的 watermark 排除以后出现更小键的 Espresso block。 |
| 每步只在 [09 §3.4](09_finality_rules_zh_v0_1.md) 所需的公共输入齐备后完成，并使用相同的确定性转移规则 | 日志覆盖用于确定下一事件。验证材料和固定规则用于确定接纳、Plan、执行与派生结果。 |

**证明。** 对读取者 $r\in\{1,2\}$，记 $e_i^{(r)}$ 为其完成的第 $i$ 个事件，$S_k^{(r)}$ 为完成前 $k$ 个事件后的公共状态，包含重放位置及后续转移使用的字段。日志下载进度和不影响转移的本地缓存不属于 $S_k^{(r)}$。按 $k$ 归纳，只要两个读取者都完成了至少 $k$ 个事件，就有

$$
e_i^{(1)}=e_i^{(2)}\quad(1\le i\le k),
\qquad
S_k^{(1)}=S_k^{(2)}.
$$

$k=0$ 时，两者使用同一 genesis 状态。从 checkpoint 继续时，[09 §4.8](09_finality_rules_zh_v0_1.md) 要求核对它与对应公共历史的重放结果一致，因此可视为从 genesis 重放到该位置后继续。

假设结论对 $k$ 成立，且两个读取者均完成了第 $k+1$ 个事件。先证明它们处理的是同一事件。由 $S_k^{(1)}=S_k^{(2)}$，两者生成同样的待处理定时器。设一个读取者完成的第 $k+1$ 个事件为 $e$，其时间键为 $x$。完成 $e$ 所用的 L1 前缀覆盖 $x-M_a$，因此已包含所有键不大于 $x$ 的控制记录。Espresso 前缀覆盖 $x$，因此已包含所有键严格小于 $x$ 的 Espresso blocks。若 $e$ 是 $\mathsf{Espresso}(q)$，还已读齐截至 $J_q$ 的前缀；以后读到的同键 blocks 按 Espresso height 排在 $e$ 之后。若 $e$ 是控制记录或定时器，同键的 Espresso blocks 按类别排在 $e$ 之后。于是，在 [09 §3.3](09_finality_rules_zh_v0_1.md) 的全序下，$e$ 是共同状态之后的最早事件。对另一个读取者作同样的论证，即得 $e_{k+1}^{(1)}=e_{k+1}^{(2)}$。

再证明该事件产生同一后继状态：

- **Espresso。** 对 $\mathsf{Espresso}(q)$，两者从截至 $J_q$ 的同一公开前缀计算 $q_{\rm avail}$。该前缀中的有效最终证明和 physical 正文决定哪些候选已完整公开。两者用相同的当前职责、接纳键及验证材料检查这些候选，并按固定的候选处理顺序和确定性的 $\mathrm{Virt}$ 追加相同的区间。若触发常规交接，也得到同一 anchor 和后继 Plan。见 [09 §3.2、§4.2–§4.3](09_finality_rules_zh_v0_1.md)。
- **Force、SessionEnd 或 Expire。** 两者面对相同的控制记录或定时器及相同的当前状态，因此关闭同一职责，或选定同一交接 anchor。需要计算 Plan 时，§1.3 的规则给出同一 L1 查询高度，相同的 L1 历史确定相同的 Plan。见 [09 §4.3–§4.5](09_finality_rules_zh_v0_1.md)。
- **RESUME。** 两者按同一个 L1 记录位置 $p_{\rm res}$ 调用 $\mathsf{DeriveAtResume}$。该位置固定对应的 CursorSet。相同的 finalized L1 历史、按 commitment 核验的 DA 数据和确定的原生派生给出同一个 ResumeCut。因此，两者把 $G$ 置为相同的 $D_L[h^0:h_{\rm res}]$，并依据相同的激活键、截止键和恢复 anchor 决定新职责。见 [09 §4.6、§5.5](09_finality_rules_zh_v0_1.md)。

因此 $S_{k+1}^{(1)}=S_{k+1}^{(2)}$，归纳成立。缺少输入时，读取者保留最后完成的位置，未完成的事件不计入其序列。取两者已完成事件数的较小值，即得两条序列前缀可比。相同重放位置对应这个共同序列中的同一步，因此 $G$、分支、职责和选定的 descriptor 相同。

这个归纳依赖公共日志覆盖和转移的确定性。E-L、E-TS、$M_a$ 的时序预算、区域 GST、跨区域传输上界和 Manager 的本地调度影响读取者何时能够推进。

归纳只要求公开证明检查的结果和接纳转移确定。现有 [09 §4.2](09_finality_rules_zh_v0_1.md) 用 BFT-1 经由 BFT-4(2) 得到更具体的输入保证，即同一 cid 中通过公开检查的决定前缀可比较。这个保证用于排除同一 cid 的冲突证明。它与给定公共结果后的重放一致性是两层性质。GFR 若另有对冲突输入的确定处理，重放归纳仍可使用该转移，但不能由此推出区域决定可比较。

### 1.3 交接起点与全局链连续性

| 交接 | 新 session 的生产 anchor | Plan 的 L1 查询高度 | 对旧 $G$ 的效果 |
| --- | --- | --- | --- |
| Regular handoff | 前驱 $h_{\max,s}$ 的 physical block 一经 GFR 接纳，就取它为 anchor。否则在公共输入覆盖截止键、前序事件已处理且前驱仍持有 NEXT 时执行 SessionEnd，取严格早于截止键的合格历史中最后已接纳的 physical block，无新块时沿用前驱初始 anchor。同键结果排在 SessionEnd 之后。后继窗口仍从 $h_{\max,s}+1$ 开始。 | 取**前驱 session 的初始 anchor** 的 origin 和当前分支的 L1 floor 中较大者。前驱一经选定就可以计算后继 Plan。 | 保留旧 $G$，在同一分支开出下一个 session。 |
| Force | 取有效 Force 在公共事件顺序中处理前的 $\mathrm{last}(G)$ 及其执行后状态。它包括排在 Force 之前已经接纳的结果，同键 Espresso block 排在 Force 之后。 | 以新 anchor 为选区参考。查询高度取该 anchor 的 origin、旧分支的 L1 floor 和收录 Force 的 L1 高度三者中的最大值。 | 保留旧 $G$，关闭旧分支，以该 anchor 和新 policy 开出新分支。 |
| L1 RESUME | 取特定 L1 记录 $p_{\rm res}$ 对应的 ResumeCut 中 $A=D_L[h_{\rm res}]$ 及其执行后状态。新窗口从 $h_{\rm res}+1$ 开始。 | 以恢复 anchor $A$ 为选区参考，新分支的 L1 floor 也取 $A.\mathit{os}$。首个 session 的查询高度就是 $A.\mathit{os}$。 | 关闭旧分支，并令 $G\leftarrow D_L[h^0:h_{\rm res}]$。旧 $G$ 超出共同前缀的部分可被撤销。 |

给定表中的查询高度 $\ell_s$，Plan 用 $\mathsf{SnapshotAt}(\ell_s)$ 确定委员会快照，并查询 $\mathsf{Ban}(\ell_s)$。因此，常规交接可以在后继生产 anchor 确定前选定委员会；RESUME 则由同一个 ResumeCut 同时确定生产 anchor 和首个 session 的授权查询高度。见 [01 §3.12](01_l1_authority_zh_v0_1.md) 和 [09 §4.3、§4.4、§4.6](09_finality_rules_zh_v0_1.md)。

交接后有合格 Plan 时，NEXT 指向一个选定 descriptor。**连接正确性**由 GFR 对 cid 和 hash-linked ancestry 的检查给出。首个被接纳的 physical block 连接到选定 anchor，后续接纳延续已接纳的 physical 前缀。若有高度缺口，确定性 Merge 从经验证的 anchor 执行后状态补齐中间 heights。**区域证明的归属**另用 CheckFINAL 对 descriptor 和目标 block 的绑定，以及 BFT-4(4) 中决定签名不可跨 cid 转用的条款。见 [09 §4.2](09_finality_rules_zh_v0_1.md) 和 [02 §3.10、§4.5](02_session_program_zh_v0_1.md)。

几个更强的结论各有独立的前提。要保证公开最终证明覆盖的块已通过应用层 Accept，需要 BFT-2 和 BFT-4(2)。要保证所有区域最终证明的目标都在窗口内，需要 BFT-7 的高度约束；仅保证 **进入 $G$ 的块** 在窗口内，使用 GFR 自己的窗口检查即可。BFT-3 的每 view 块数限制、BFT-4 的违规证据性质和 BFT-7 的停止签名性质都不用于上述 anchor 连接或重放一致性的论证。见 [02 §4.2–§4.9](02_session_program_zh_v0_1.md)。

**Manager 的本地保证。** 初次 Enter 时，核对选定 descriptor、anchor 身份和执行后状态。已经运行过该 cid 时，恢复原共识状态。处理 SessionAssigned 后只调度匹配实例。处理决定之前，Regular handoff 允许不同 Manager 运行不同候选。GFR 的选择和接纳基于公共输入，Manager 的就绪状态和各 replica 的 Enter 时刻只影响哪些参与者能及时运行该 cid。见 [03 §3.7–§3.10、§4.2–§4.3](03_session_manager_zh_v0_1.md)。

### 1.4 无 RESUME 与有 RESUME 的安全性质

**命题 G-S2（无 RESUME 的区间只追加）。** 对同一公共重放中两个位置 $p\le p'$，若 $(p,p']$ 内没有执行 RESUME，则 $G_p\preceq G_{p'}$。这个单次重放结论只使用 GFR 的更新规则。Espresso 接纳只追加连续片段，Regular handoff、Force 和 Expire 都保留旧 $G$。要比较两个读取者各自输出的 $G$，再使用 G-S1 确认它们处理的是同一公共事件序列。若整个执行没有 RESUME，前缀性质覆盖该执行的全部位置。

**命题 G-S3（无 RESUME 执行中的整条前缀及时性）。** 对从 genesis 开始且没有 RESUME 的公共重放，设 physical block $B$ 在位置 $p_B$ 首次经 GFR 接纳，包括由同链后代的完整结果覆盖而接纳。对每个非 genesis 前缀高度 $h^0<h\le B.h$，记 $p_h$ 为该高度首次进入 $G$ 的位置，$x_h$ 为该位置的时间键，$s_h$ 为当次追加所属的 session，$\mathit{Res}_h$ 为使该高度进入 $G$ 的完整区域结果。则

$$
p_h\le p_B,\qquad
\theta_{q_{\rm avail}(\mathit{Res}_h)}\le x_h<\theta_{s_h}^{\rm end},\qquad
x_h<\mathit{ts}^*(h)+W_{\rm exp}.
$$

因此，$B$ 之前的前缀可以更早已在 $G$，也可以与 $B$ 在同一 Espresso 事件中按高度一起追加；不要求每个祖先各自发证明或在更早事件单独接纳。公开截止适用于当次覆盖该高度的完整结果及其所需 physical 前缀。Padding 使用覆盖它的后继结果所属 session 的公开截止，同时仍检查该 virtual height 自己的 Expire，无需另发正文。这里的“按时”按公共时间键判断，不是读取者实际完成重放的真实时间上界。

**论证。** 按公共事件及一个 Espresso 事件内的追加顺序归纳。Genesis 没有需要核对的非 genesis 前缀。每次追加前，旧前缀已满足上述性质；[09 §4.2](09_finality_rules_zh_v0_1.md) 只在完整 physical 结果满足公开截止、整个新增 virtual 区间逐高度未过期后，才连续按高度追加。结果已在当前 Espresso 前缀公开，故其公开键不大于接纳键；接纳时该 cid 仍负责，同键或更早的 SessionEnd 已先处理，故接纳键也严格早于所属 session 的截止。新增高度因而满足各项不等式，旧前缀保持不变。Regular handoff、Force 和 Expire 都保留已接纳前缀，不引入未经上述检查的新高度。归纳即得，且允许 $p_h=p_B$。RESUME 直接采用 $D$ 的指定前缀，该段内容可能没有经 Espresso 及时公开，故不在本命题范围内。

**跨 RESUME 的一致切换。** 这一步用公共重放顺序固定同一条 RESUME，并用绑定该 L1 位置的确定 ResumeCut 固定替换内容。记 $G^-$ 和 $G^+$ 分别为处理一条 RESUME 之前和之后的链。根据 [09 §4.6](09_finality_rules_zh_v0_1.md)，

$$
G^+ = D_L[\,h^0:h_{\rm res}\,].
$$

按原生 block 身份比较，$G^-$ 与 $G^+$ 的最长共同前缀继续保留。旧链在该共同前缀之后的后缀，即使此前已进入 $G$，也可能在这次恢复中被撤销。若 $G^-\preceq G^+$，这次 RESUME 只追加派生内容。若两条链在某个共同 height 的原生 block 不同，第一个不同的 height 就是这次 hard fork 的分叉点。一个读取者已处理 RESUME、另一个尚未处理时，两者的 $G$ 可以不再前缀可比。处理同一条 RESUME 后，它们依据 G-S1 采用同一个派生前缀和同一个新 anchor。

例如，$G^-[36\,000]$ 已凭 Espresso 日志成为全局确认块。随后 RESUME 使用的 $D_L[36\,000]$ 是另一块，两条链共同前缀只到 35 999，原块在全局确认约 25 分钟后被替换。固定 cid 的区域 BFT 决定事实仍成立，发生变化的是该块在 $G$ 中的地位。见 [10 §5.5](10_pipelines_zh_v0_1.md)。

因此，同一公共重放位置的一致性适用于两类执行；前缀持久性适用于无 RESUME 的区间，而 G-S3 的整条前缀及时性只对从 genesis 开始且没有 RESUME 的执行主张。跨 RESUME 的不可撤销结算属于 $D$ 的安全性质。当前规则没有给 $G$ 一个等待若干时间后便免于 RESUME 替换的保证。见 [09 §7](09_finality_rules_zh_v0_1.md)。

## 2. 活性：Espresso 持续 live

将 [00 §1.4](00_arch_zh_v0_1.md) 的 E-L 分成两条使用。**收录保证**为合格消息给出 finalized 且可读的时间界，用于正常块的发布。**Watermark 保证**为日志末端的时间键给出进展界，用于等待 SessionEnd、Expire 或 L1 控制事件。两条各自在所需区间使用 $\mathsf{Live}_E=1$。E-TS 约束单个 Espresso block 的时间键。以下分别注明何时使用这三项，以及区域和 L1 需要提供什么。

### 2.1 已选定 session 中的普通块

设 $B$ 是当前 NEXT 对应 cid 中一次满足 BFT-5 前提的成功 view 所决定的 physical block，height 为 $h$。记 $\tau_f$ 为区域决定的时刻，$\tau_P$ 为完整结果交给诚实在线 Publisher 的时刻，$\tau_G(B)$ 为一个持续在线且跟上公共输入的读取者把 $B$ 追加进 $G$ 的时刻。分析以下三段。

1. **从 slot 到区域决定。** 给定一次满足 [02 §4.6–§4.7](02_session_program_zh_v0_1.md) 参与、同步和剩余时间条件的成功 view，这一步使用 BFT-5(1)。若 leader 在真实时刻 $\tau_0$ 开始 Build，$\hat\tau(h)$ 是 active replicas 中最后一个本地时钟到达 $\mathit{ts}^*(h)$ 的真实时刻，则

   $$
   \tau_f \le \max\{\tau_0+T_{\rm build},\hat\tau(h)\}+T_{\rm fin},
   \qquad
   \tau_f-\mathit{ts}^*(h)<b_{\rm L2}+\delta_G+T_{\rm build}+T_{\rm fin}.
   $$

   从第一式得到第二式只需 Build 的 slot 选择规则和 C2 的时钟偏差界。该界从一个已满足前提的成功 view 起算。若从某次交接或某个失败 view 起算，还要给出该 view 何时就绪，见 §2.2。

2. **从区域结果到 Espresso 最终收录。** 给定完整结果已经交给诚实在线 Publisher，发布预算 $T_{\rm pub}$ 加上 E-L 的收录保证给出 $\tau_P+T_{\rm pub}+T_E$ 的真实时间上界。若承载结果的 Espresso block 满足 E-TS，再用其**超前方向**的界得到 $x=\theta_{q_{\rm avail}}\le\tau_P+T_{\rm pub}+T_E+\Delta_{\rm ts}$。若另有 $\tau_P\le\tau_s^{\rm stop}+\delta_G$，则 $T_{\rm grace}>T_{\rm pub}+T_E+\Delta_{\rm ts}+\delta_G$ 足以使 $x<\theta_s^{\rm end}$。BFT-7 的停止规则只限制新签名，不能推出这项 Publisher 交付前提。新增区间还要满足 Expire 的接纳界。若已知实际公开键满足各项截止，接纳这一步无需用 E-TS 推导该键。见 [04 PUB-4](04_publisher_zh_v0_1.md)。

3. **从可读到进入 $G$。** 读取者处理到完整结果对应的 Espresso 事件后，按 [09 §4.2](09_finality_rules_zh_v0_1.md) 追加合格扩展。承载该结果的 finalized block 及此前完整前缀本身就提供覆盖键 $x$ 的 watermark。L1 控制日志须覆盖 $x-M_a$，并取得此次验证所需输入。若控制日志覆盖已经就绪，直接使用该事实即可。若要用公告参数推出这项覆盖不拖慢重放，则使用 [09 §3.5](09_finality_rules_zh_v0_1.md) 中的条件，

   $$
   M_a\ge T_{\rm L1}+b_{\rm L1}+\Delta_{\rm ts}+\delta_{\rm L1},
   $$

   其推导使用 L1 的产出和 finalized 可读界，以及承载 block 的 E-TS 超前界。这个不等式只保证 L1 控制日志覆盖及时就绪；验证若需要更晚的 finalized L1 输入，仍须按 §1.5 的输入契约提供，不能由该预算推出其就绪时间。稳态的块无需等待 $M_a$ 或 $T_{\rm grace}$。排在前面的输入缺口、日志积压和本地处理另计。

为表示 v10 中没有单独量化的两段延迟，记 $T_{\rm pack}$ 为区域决定到完整结果交给 Publisher 的假定上界，$T_{\rm replay}$ 为所需公共输入就绪后处理到本次接纳的假定上界。正常 Final 输出能提供可验证证明，使用 BFT-4(1)。$T_{\rm pack}$ 还须覆盖该输出和组装所需 bodies、来源依据的实际交付。在上述条件成立、接纳所需验证输入在结果公开可读时已就绪、且接纳时该 cid 仍被选定的情况下，

$$
\tau_G(B)-\mathit{ts}^*(h)
<
b_{\rm L2}+\delta_G+T_{\rm build}+T_{\rm fin}
+T_{\rm pack}+T_{\rm pub}+T_E+T_{\rm replay}.
$$

[10 §2.2](10_pipelines_zh_v0_1.md) 取 $T_{\rm pub}=T_E=30\,\mathrm{s}$、$b_{\rm L2}=1\,\mathrm{s}$、$\delta_G=1\,\mathrm{ms}$，所以式右边为 $61.001\,\mathrm{s}+T_{\rm build}+T_{\rm fin}+T_{\rm pack}+T_{\rm replay}$。这是条件性确认预算，文档尚未给出全部部署值。持续增长还要求后续不断有满足这些条件的结果，而一次成功 view 只保证对应结果的进展。

> In the real world, we can take $T_{pub}$ $T_{E}$ as every small number 
> a tradeoff here: a bigger M_a over the threshold creates the fast finality, but make the regional livesness lockdown last longer. 

### 2.2 三种交接如何恢复共同参与

对新 session $s$，要从交接推出区域进展，需要一个总权重至少为 $Q_s=W_s-F_s$ 的 honest 集合 $U$。这些 replicas 须在线，并由各自的 Manager 准备同一 cid 的 anchor 和执行状态，持续调度该实例。BFT 的有界进展还要求该区间处在新 region 自己的 $\mathrm{GST}_{i_s}$ 之后，所需输入可在预算内取得，并在 $\tau_s^{\rm stop}-\delta_G$ 前留出足够时间。区域 GST 用于这段 BFT 运行，跨区结果到达时间另按 [00 §2.4](00_arch_zh_v0_1.md) 的信道条件计算。

若 $U$ 在最快本地时钟到达窗口起点之前都已首次 Enter，并满足 [02 §4.6](02_session_program_zh_v0_1.md) 的其余同步开始条件，则由 L-CLK 和 NotBefore，它们的 view 1 起算时刻落在

$$
[\tau_s^{\rm start}-\delta_G,\ \tau_s^{\rm start}+\delta_G]
$$

内，相差最多 $\Delta_G=2\delta_G$。若它们稍后进入或从暂停恢复，使用一般的 $T_{\rm sync}$ 预算和 BFT-6 的 view 同步条款。若还要覆盖首个成功 view 之前的失败 view，另用 BFT-6 的退出时限和对 active leader 出现频率的约束。对已成功的 view，决定该块用 BFT-5(1)，进入后继 view 的时限用 BFT-5(2)。见 [02 §4.6–§4.8、G3.3](02_session_program_zh_v0_1.md)。

| 交接 | 何时可让足够权重运行同一 cid | 对 $G$ 进展的影响 |
| --- | --- | --- |
| Regular handoff | 后继可提前算 Plan 并追前驱状态。依靠直接转发来提前就绪时，使用持续 live 区间的 $\Delta_{\rm cross}$、结果交付和状态准备预算，以及时钟余量损失至多 $\Delta_G$ 的界。已经从公开日志取得相同结果的 replica 可以直接准备。 | 前驱的 $h_{\max,s}$ 一经接纳就确定交接。若最后一个高度未被接纳，等待 SessionEnd 使用 E-L 的 watermark 保证。候选可先生产，只有匹配最终 anchor 的结果才具备接纳资格。迟到的前驱证明仍可能让后继切换候选。 |
| Force | 取得激活键的 Espresso watermark 和所需 L1 前缀后，GFR 固定 $G$ 的快照。新窗口通过 $T_{\rm init}$ 留出准备时间。足够权重在最快启动前须完成公共重放及 anchor 状态准备。 | 从激活键 $x$ 到取得 watermark 的 $\Delta_{\rm ts}+T_E$ 预算直接用 E-L 的 watermark 保证。本例按 §2.1 的服务参数，$T_{\rm init}=120\,\mathrm{s}$ 时，扣除这段等待和 $\delta_G$ 后，保守可用准备余量约为 $75\,\mathrm{s}$，重放和状态准备须在其中完成。旧分支已 HALT 时 Force 无效。 |
| L1 RESUME | 除了 L1 和激活键的 watermark，还要及时取得 DA 数据、完成派生并取得 ResumeCut 中的 anchor 状态。选定 $h_{\rm res}$ 时使用 $T_{\rm prep}\ge M_a$ 和 $M_a+T_{\rm ready}+\delta_G\le T_{\rm prep}+b_{\rm L2}$，其中 $T_{\rm ready}$ 覆盖激活后尚需完成的工作。见 [01 §3.10](01_l1_authority_zh_v0_1.md)。 | 只有需要界定 watermark 等待时才用 E-L 的 watermark 保证。对齐 $G$、分配新 session、首个新块进入 $G$ 是依次完成的步骤。最后一步还需区域 BFT、E-L 的收录保证及 GFR 接纳。 |

以常规交接为例，若前驱最后一个 view 已就绪、leader 诚实，并在本地时钟到达最后一个 slot 时开始 Build，则 $\Delta_G+T_{\rm build}+T_{\rm fin}<b_{\rm L2}$ 是前驱末块在最快 replica 停止签名前决定的充分条件。要让后继在窗口起点就使用这个 anchor，还要在前驱决定到后继最快启动之间完成结果转发和状态准备。若走 SessionEnd 路径，持续 live 时可在 $\theta_s^{\rm end}+\Delta_{\rm ts}+T_E$ 前取得相应 watermark，随后仍需完成 L1 核对和重放。候选结果已经完整公开时，也要等 GFR 选中对应 cid 才能进入 $G$。若启动有延迟但仍赶得上窗口，后继可以从较晚的 height 出块，中间高度由 Merge 补齐。见 [10 §3.3–§3.5](10_pipelines_zh_v0_1.md) 和 [09 §3.5](09_finality_rules_zh_v0_1.md)。

## 3. 活性：Espresso non-live 或时间戳异常

### 3.1 non-live，但仍满足 E-S，且已产出 block 的时间键正常

$\mathsf{Live}_E=0$ 表示模型不承诺 $T_E$ 内完成收录或提供新 watermark。它允许 Espresso 彻底停顿，也允许 Espresso 偶尔推进。时间键仍接近真实时间是本小节的情形设定，non-live 时 v10 本身不提供 E-TS 界。

- **仍有 finalized Espresso blocks 可读。** 只要结果完整公开、时间键符合截止、当前 cid 匹配，而且前面的事件已处理，$G$ 就可以继续追加。该期间没有统一的真实时间延迟上界。若以后每次需要的日志前缀、watermark 和接纳材料都能取得，满足接纳条件的扩展仍能重放。
- **finalized 前缀不再推进。** 读取者处理完已有的公共前缀后，新的区域结果无法通过 Espresso 进入 $G$，键高于最后 watermark 的 SessionEnd、Expire、Force 和 RESUME 也无法处理。当前区域实例仍可能在本地停止前出块。后继可在已准备的候选上运行，但跨区转发在 non-live 期间只有最终送达保证，且 Manager 只预备已选定前驱的一个后继。因此区域生产的继续不保证 $G$ 继续增长。L1 正常推进时，$D$ 仍可按其原生规则前进。

Espresso 恢复后，GFR 按积压事件的原有时间键重放。决定结果能否留下的是完整公开位置和该位置上的职责与截止。停机前提出、恢复后才 finalized 的在途 Espresso block 仍用原键，可能排在 SessionEnd 或 Expire 之前。恢复后才首次完整公开的结果，也可能因公开键太晚而被排除。若最新公开记录覆盖了原缺口，或者仍能在 Expire 前及时接纳后继扩展，$G$ 可以继续增长。若重放先到达未覆盖高度的有效 Expire，分支进入 HALT，只有可处理的 RESUME 能恢复。停机的真实时长本身不能决定哪种情形发生。见 [10 §5.5–§5.6、§7](10_pipelines_zh_v0_1.md)。

### 3.2 Espresso 仍 finalize，但时间戳异常

下面保持 E-S，因而 $\theta_q$ 仍不递减，只考察时间键与真实时间脱节。在 v10 的模型内，持续 live 的正常执行满足 E-TS。若 Espresso 在有收录进展时违反相应时间保证，以下是沿用现有 GFR 规则分析的故障轨迹。即使收录本身及时，仍要分别检查 watermark 和接纳截止。

| 时间异常 | GFR 的行为 | 对 $G$ 活性的后果 |
| --- | --- | --- |
| **超前或向前跳变** | watermark 可以很早覆盖 SessionEnd、Expire 或控制记录的键，但处理仍受 L1 前缀约束。大键的 Espresso 结果需要更靠后的 L1 历史。 | 即使区域出块和 Espresso 收录都及时，过大的公开键仍可能使结果排在 SessionEnd 或 Expire 之后。一个跳变可让多段 session 结果无法接纳。如果下一缺失高度的 Expire 先被处理，分支进入 HALT。 |
| **滞后但仍前进** | 已收录结果可以按较小的键处理。SessionEnd、Expire、Force 和 RESUME 要等 watermark 到达各自的键。 | $G$ 可能继续追加，但交接或恢复在真实时间上延迟。新 session 被公开选定时，其原定窗口可能已部分或全部过去。过晚进入 $G$ 的结果还可能赶不上 $D$ 的原生窗口。 |
| **时间键冻结在 $\bar\theta$** | 键为 $\bar\theta$ 的新 Espresso blocks 仍可能被处理。键大于 $\bar\theta$ 的定时器和 L1 控制记录一直等不到 watermark。 | 若前驱能让 $h_{\max,s}$ 被接纳，仍可触发常规交接。若当前 session 无法接纳末高度、且 SessionEnd 的键在 $\bar\theta$ 之后，$G$ 可永久停在当前职责上。键同样在 $\bar\theta$ 之后的 Force 和 L1 RESUME 都无法解除这次停顿。 |

时间戳故障的关键区别是，**Espresso 高度增长并不推出时间 watermark 前进**。恢复需要日志重新提供覆盖相关事件键的 watermark，且之前的事件和恢复切面可重放。E-S 允许同键但禁止倒退，所以向前跳变也无法靠下一块采用较小的键立即消除。若故障进一步破坏时间戳单调性，已超出 §1 的安全前提，见 [v9 时钟审查 K11–K15](../formalization_v9/clock_skew_review_zh_v0_2.md)。

### 3.3 HALT 后恢复的条件

GFR 在 HALT 状态下不能靠新区域结果或 Force 前进。按 [01 §3.10](01_l1_authority_zh_v0_1.md) 的治理规程，须等 $D$ 越过或偏离生产分支，并在服务恢复后提交适当的 RESUME。因此，Expire 已生效到规程允许 RESUME 之间可能有一段等待 $D$ 的时间。L1 记录 finalized 后，GFR 仍须取得覆盖 RESUME 激活键的 Espresso watermark 和该条记录的 ResumeCut。处理之后，还要有合格 Plan、足够权重完成新 anchor 的状态准备，并在窗口结束前完成成功 view 和发布。任一阶段缺少进度保证，都不能从 Espresso 恢复 live 的时刻单独推导出 $G$ 恢复出块的统一上界。见 [10 §5–§7](10_pipelines_zh_v0_1.md)。
