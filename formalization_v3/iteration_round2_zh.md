# 第二轮迭代：Part II v4.7 → v4.8，Part III v4 → v5（终版）

日期：2026-10-24。上一轮见 [iteration_round1_zh.md](iteration_round1_zh.md)。按约定最多两轮，本轮校对后的补丁思路只作为建议列出，不再派生新版本。

## 0 产物

| | 正文 | PDF | 本轮 diff | 两轮累计 diff |
|---|---|---|---|---|
| Part II v4.8 | [tex](sync_hotstuff_rr_part2_rotation_v4_8.tex) | [pdf](sync_hotstuff_rr_part2_rotation_v4_8.pdf)（49 页）| [v4.7 → v4.8](../output/pdf/sync_hotstuff_rr_part2_rotation_v4_8_diff_v4_7.pdf) | [v4.6 → v4.8](../output/pdf/sync_hotstuff_rr_part2_rotation_v4_8_diff_v4_6.pdf) |
| Part III v5 | [tex](sync_hotstuff_rr_part3_staking_v5.tex) | [pdf](sync_hotstuff_rr_part3_staking_v5.pdf)（7 页）| [v4 → v5](../output/pdf/sync_hotstuff_rr_part3_staking_v5_diff_v4.pdf) | [v3 → v5](../output/pdf/sync_hotstuff_rr_part3_staking_v5_diff_v3.pdf) |

- 两份文件都编译通过，没有 overfull/underfull 和未定义引用。
- 规模：本轮 Part II +90/−71 行，Part III +48/−37 行。两轮累计：Part II +464/−376 行；Part III +426/−353 行，3418 词。
- Part II v4.8 继续使用第一轮的 `figures/cross_region_rotation_v4_7/`，图注无需再改。

## 1 本轮改了什么

本轮按第一轮校对的 R2-1 至 R2-6 实施。核心只有一条：**候选规则换成每个分支一条循环顺序。**

### Part II v4.7 → v4.8

| 位置 | 改动 | 对应 |
|---|---|---|
| Def 2.1、表 3、记号表 | $\mathcal P=(S,\mathcal L,e^{\min})$：$\mathcal L$ 是全部 region 的一条循环顺序，取代"候选规则 Ord" | R2-1 |
| Def 2.2 | (V4) 由 Part II 自己检查 $S$ 和 $\mathcal L$ 是否完整；(V6) 只剩 Part III 的 minimum check：不减，且该快照已激活 | R2-1 |
| §2.1 Authority interface | 拆成短段，加一张小表"Part II 从 $\Pi$ 读什么"（region、委员会、TB/K/Δ、$\lambda(\Pi)$、$\mathrm{id}(\Pi)$）；`Plan` 去掉 `cause` 参数；修正自引用 | R2-1、R2-3 |
| §2.1 Choosing the next region | 一句话定义：$\mathcal L_\beta$ 中 $i_{\rm src}$ 之后（最后才是 $i_{\rm src}$ 本身）第一个 ON、且有未被禁任的正权重成员的 region。genesis 取 $\mathcal L_0=\mathcal L_R$。"default cyclic rule"的说法删去 | R2-1 |
| §2.1 Ingesting controls | 新增一段：有效 update 按日志顺序激活，所以"会不会执行"在入日志时就已确定；保管日志的 L1 合约可以在提交时记下这一结果，供 Part III 的 release check 读取 | R2-2 |
| Protocol 3、§6.2、Genesis | 三处 `Plan` 调用都去掉 cause | R2-1 |
| §5.3 descriptor | 注明 $i$、$H$ 由 $\Pi$、$A$ 决定，保留它们只为可读性 | R2-6 |
| §3.1 例子、§7 引言 | 改为"$\mathcal P_7$ 的循环顺序为 A, B, C"；"每个分支都沿用 genesis 顺序 $\mathcal L_R$" | R2-1 |
| 附录 B | (R7) 改为"每次执行的 update 推进清退水位"；BOOT/STALL 的说明独立成 (R8) "Keep the order"；WITHDRAW 行的效果改为"advances the clearance list" | R2-4 |

### Part III v4 → v5

| 位置 | 改动 | 对应 |
|---|---|---|
| §3.1 Branch policy | $\mathcal P_\beta=(S_\beta,\mathcal L_\beta,e^{\min}_\beta)$。**候选列表**：$\mathcal L_\beta$ 中 $i_{\rm src}$ 之后的 region，再接 $i_{\rm src}$。regular 与首个 session 用同一条规则。"规则族"删去，"policy check"收窄为"minimum check" | R2-1 |
| §3.2 Plans | `Plan` 去掉 cause；第 2 步改用候选列表；$\mathrm{id}(\Pi)$ 不再包含 cause | R2-1 |
| 第 1 页接口表 | `Plan` 签名更新；AUTH 改写为"the plan with the ban set of generation $g$"；"Minimum check" 行 | R2-1、R2-5 |
| §1 | 解释 export session："the session that owned the next DATA after $G$" | R2-5 |
| §3.4 例子 | $\mathcal L_7=(A,C,B,D)$，A 之后是 C、B；$\mathcal L_8=(A,B,C,D)$，A 之后是 B。补一句：用 genesis 顺序也会选 B | R2-5 |
| §4.2 | `Eff` 由"保管控制日志的 L1 合约"在提交时计算（对应 Part II §2.1） | R2-2 |
| §5 | 新增段落 "Assumptions added by this module"：ledger、REC、控制日志都在 L1 合约中，受 $T_{\rm L1}$ 约束；Espresso 的 L1 light client 只推迟 $t_c$；evidence service 在 $T_{\rm ev}$ 内提交证据；时钟余量成立 | R2-2 |
| 接受的代价 | 补充：每个分支一条顺序；要改顺序就需要一次 branch update | R2-1 |

### 两轮合计

**Part II（v4.6 → v4.8）**
- 只通过一个接口取用 authority：分支携带 $\mathcal P=(S,\mathcal L,e^{\min})$；plan 给出 region、委员会和参数；签名与 REC 都绑定 $\mathrm{cid}$；所有类型的 quit proof 都送上 L1；quota 为常数 TB。
- 例子和全部命题陈述都没有改。

**Part III（v3 → v5）**
- 结构：接口在前，正文分成 L1 账本、纯函数、L1 入口三块。
- REC 按 cid 唯一，L1 只做相对验证。
- 释放不再重放 $G$，改为 `Eff` + header + 清退水位。
- 只在一个 L1 高度读取。
- 选区规则是一条逐分支的循环顺序。

## 2 终版校对：从整体看

### 2.1 抽象是否清晰、干净

**清晰。** 现在可以用三句话讲完两份文档的接口：

1. **装**：每个分支一份 $\mathcal P=(S,\mathcal L,e^{\min})$，只由 Force 原子安装。
2. **查**：$\Pi=\mathsf{Plan}(\mathcal P,\beta,r,E,i_{\rm src})$ 给出 region、委员会、参数、源高度和 id；$\mathsf{Leader}(\mathsf{AUTH}_{\beta,r,g},t)$ 给出 leader。缺证据时返回 WAIT。
3. **报**：`SubmitQuit`、`SubmitOffense`、`ProveForce`。L1 只做相对验证。

**干净。** 第一轮的主要问题——候选规则只说是"一族"，却没有定义——已经消失。现在：

- `Plan` 有 5 个参数，每个参数都有用途；
- policy 有 3 个字段：两个由 Part II 检查完整性，一个由 Part III 检查单调性；
- Part II 不出现快照编号、读取高度、禁任集合等 staking 内部记号；
- Part III 不重述 Part II 的流程。

**仍不完美，但可以接受：**

- $\Pi$ 和 $\mathsf{AUTH}_{\beta,r,0}$ 两个名字指同一对象。原因是 Part I/II 一直用 AUTH 记号，两份文档各只在一处说明二者等同。
- descriptor 中有冗余字段 $i$、$H$，正文已注明。

### 2.2 抽象是否合理

合理。关键判断都有依据：

| 判断 | 依据 |
|---|---|
| 提前一棒排班 | 后继的 region 和 leader 要依赖快照，而快照依赖 $E$ 的 L1 引用。只有 $E_s=A_p$ 才能在前驱开始时就算出来。两个 anchor 带来的概念负担，换来的是无需 L1 交易和充分的准备时间 |
| L1 相对验证 | L1 无法知道跑在 $G$ 之前的 session 是否真实。按 cid 记账后，伪造只会产生无人使用的记录 |
| 关门只看 `Eff` | 激活顺序等于日志顺序，每个执行的 update 使 $\beta$ 加一，所以是否执行在提交时就已确定 |
| 每个分支一条循环顺序 | Part II 原来的 NextLive 就是 $\mathcal L_\beta=\mathcal L_R$ 的特例；Force 后"跳过 source"的推进性自动保留 |

**这次改变了你原设计的四处，请确认：**

1. $g=0$ 的禁任集从快照截止集改为 $\mathsf{Ban}(\ell)$，排除更快。
2. SubmitQuit 若附带双签，先禁任再冻结 REC。
3. 有效 offense 使该 position 在下一个快照起强制退役。
4. 不再支持"每个轮次换一个排列"（randList 随 $r$ 变化），每个分支只有一条顺序。

每一处都可以单独回退，回退时只改 Part III 的对应段落。

### 2.3 还能更简单吗：留给下一轮的建议

**S1：去掉 $e^{\min}$ 字段，改为由规则推出。** 附录 B 的 (R7) 已经建议每次 update 都把最小值设成"已激活的最新快照"。那就不如直接规定：

> 新分支的最小快照编号，等于包含该 update 的 L1 高度上已激活的最新快照编号。

由此：

- policy 只剩 $(S,\mathcal L)$，(V6) 和 minimum check 一起删掉；
- 单调性自动成立，因为包含高度只增不减；
- 每次 Force 都会推进清退水位，WITHDRAW 只是"很久没有 Force 时补发一次"。

代价是委员会不能刻意选一份更旧的快照。我想不出需要这样做的场景：最新快照里的成员本来就已经处于 Active 状态。这是我最推荐的下一步简化。

**S2：REC 只记录一个 L1 高度。** 让 $\mathsf{Ban}_{s,g}=\mathsf{Ban}(h_{\rm rec})$，与 $g=0$ 的 $\mathsf{Ban}(\ell)$ 写法统一，所有禁任集合都是"某个 L1 高度上的登记"。这只是表述上的统一，不改变语义。

**S3（必须）：Part IV 同步到新接口。** Part IV v0.1 的 Protocol 1 仍复述旧的 Force 前奏：$S_{\rm new}$、`NextLive`、$\mathit{TB}_{r'}$。Part II 说 Part IV "restates its prelude"，两者现在不一致。这是一次机械修改：

- $\mathcal P_{\rm new}$；
- $\Pi'=\mathsf{Plan}(\mathcal P_{\beta'},\beta',r',A,i_{\rm src})$；
- TB；
- 若采纳 S1，则去掉 $e^{\min}$。

按本次约定只迭代 II 和 III，所以没有改。

### 2.4 局部问题（不影响结论）

- Part III §1 说 Force 的 $i_{\rm src}$ 是"export session 的 region"。如果此前是 PENDING，实际是 $\omega$ 中保存的 source region，应补半句。
- Part II 表 3 的 $\mathsf{AUTH}_{\beta,r,0}$ 行仍引用"section 2.1"，即所在小节自身，影响很小。
- [Part III 中文快速导读](sync_hotstuff_rr_part3_staking_v3_quick_guide_zh.md) 仍对应 v3，需要按 v5 更新：两个 anchor 的部分不变；选区一节改用循环顺序；退出一节改用清退水位。

## 3 检查范围

- **编译与渲染**：v4.8、v5 及四份 diff 都编译通过。
- **全文检索**：确认没有遗留 `NextLive`、Ord、cause 参数、$\mathit{TB}_r$、cfg 以及旧的 $J$、$m_\beta$。
- **逐段通读**：Part III v5 和 v4 全文按编译后的文本读过；Part II 读了 §1–§2.4、§3.3、§6.1–6.2 和附录 B。
- **版面抽查**：抽查了 Part III 第 1–4 页，以及 Part II 接口页和一页 diff。
- **未做**：没有逐页检查 Part II 全部 49 页的版面；没有运行模型检查或机器证明。正确性论证与第一轮接口文档相同，仍是人工推理。

## 4 尚未处理

- `README.md` 和 `build.sh` 仍指向 v4.6 和 v3。确认采用新版本后，可以更新这两个文件并重新构建。
- 旧版本（v4.6、v3）、第一轮版本（v4.7、v4）和接口分析文档都保留在原位置。
