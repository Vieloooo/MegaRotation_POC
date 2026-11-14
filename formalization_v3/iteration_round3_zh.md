# 第三轮迭代：Part II v4.8 → v4.9，Part III v5 → v6

日期：2026-10-24。前两轮见 [iteration_round1_zh.md](iteration_round1_zh.md)、[iteration_round2_zh.md](iteration_round2_zh.md)。本轮按第二轮校对的建议 S1、S2 和局部问题修订，流程不变：修订 → 校对 → 补丁思路。

## 0 产物

| | 正文 | PDF | 本轮 diff | 相对起点的累计 diff |
|---|---|---|---|---|
| Part II v4.9 | [tex](sync_hotstuff_rr_part2_rotation_v4_9.tex) | [pdf](sync_hotstuff_rr_part2_rotation_v4_9.pdf)（49 页）| [v4.8 → v4.9](../output/pdf/sync_hotstuff_rr_part2_rotation_v4_9_diff_v4_8.pdf) | [v4.6 → v4.9](../output/pdf/sync_hotstuff_rr_part2_rotation_v4_9_diff_v4_6.pdf) |
| Part III v6 | [tex](sync_hotstuff_rr_part3_staking_v6.tex) | [pdf](sync_hotstuff_rr_part3_staking_v6.pdf)（7 页）| [v5 → v6](../output/pdf/sync_hotstuff_rr_part3_staking_v6_diff_v5.pdf) | [v3 → v6](../output/pdf/sync_hotstuff_rr_part3_staking_v6_diff_v3.pdf) |

- 两份文件都编译通过，没有 overfull/underfull 和未定义引用。
- Part II 的图换用 `figures/cross_region_rotation_v4_9/`：Force 图的注释从"complete policy $\mathcal P_{\rm new}$"改为"complete map and order"。
- v5 → v6 的 diff 需要两处手工修补才能编译：一是 latexdiff 删除表格行时留下半行，二是 v6 已不再定义 `\emin`。两处都只改了 diff 文件，正文未动。
- 规模：本轮 Part II +86/−86 行，Part III +48/−49 行。
- 相对起点累计：Part II 为 +445/−357，比 v4.8 时的 +464/−376 更小，因为附录 B 和例子回到了 v4.6 的写法；Part III 为 +425/−353。

## 1 本轮改了什么

一句话：**去掉"最小快照编号"，改为"L1 地板"。** 每个分支记住创建它的 update 所在的 L1 高度 $\lambda_\beta$，Part III 永远不用比它更旧的 L1 状态为该分支排班。

### Part II v4.8 → v4.9

| 位置 | 改动 | 为什么 |
|---|---|---|
| Def 2.1 | $U=(\ldots,\beta_{\rm target},S_{\rm new},\mathcal L_{\rm new})$：update 只携带委员会选择的两样东西（表与顺序）；记 $\lambda_U$ 为包含 $U$ 的 L1 高度 | $e^{\min}$ 不再是可选字段（S1）|
| Def 2.2 | 删去 (V6)，恢复为 V1–V5；(V4) 仍检查 $S$ 和 $\mathcal L$ 完整 | 不再需要 Part III 的 minimum check |
| Def 2.3、Protocol 3 第 1 步 | 执行时安装 $\mathcal P_{\beta'}:=(S_{\rm new},\mathcal L_{\rm new},\lambda_{U})$ | 地板由 L1 决定，委员会不能选 |
| §2.1 Genesis 段 | 定义 **L1 floor** $\lambda_\beta$：创建该版本的 update 所在的 L1 高度；Part III 不会用更旧的 L1 状态为该版本排班；Force 就是这样让旧快照退役的 | 用一个不变量取代"最小快照编号"规则 |
| 表 3、记号表 | Policy 行改为"表与顺序，加上 $\lambda_U$"；$\mathcal P_\beta=(S_\beta,\mathcal L_\beta,\lambda_\beta)$；$U$ 的元组移到说明列 | 同上；顺带消除 underfull |
| 摘要、§1.1、§1.2、表 1、§7.2/7.4/7.5/7.7/7.8 | "complete policy"改回"complete map (and order)" | update 本身只携带表和顺序；文字回到 v4.6 |
| 附录 B | 回到 $S^\circ$、$S_{\rm new}$ 的写法；删去 (R7) Raise the minimum；"Keep the order"成为 (R7)，并说明每次执行的 update 都会抬高地板；WITHDRAW 改为"$S^\circ$ unchanged"；Retire 条件改为"当前版本的地板尚未覆盖" | 每次 Force 自动推进清退，策略里不再需要专门抬高最小值 |
| 表 3 AUTH 行 | 自引用"section 2.1"改为"see the authority interface" | 第二轮遗留的局部问题 |

### Part III v5 → v6

| 位置 | 改动 | 为什么 |
|---|---|---|
| §3.1 | $\mathcal P_\beta=(S_\beta,\mathcal L_\beta,\lambda_\beta)$；删去 minimum check | S1 |
| §3.2 第 1 步 | 读取高度 $\ell=\max\{\lambda(E),\lambda_\beta\}$；去掉"等价于 $\max\{e^{\min},\ldots\}$"的说明 | 读取高度直接取"参考块引用的 L1"与"分支地板"中较大者，不再绕道 $a_{e^{\min}}$ |
| §3.2 第 4 步 | 源高度 $\lambda(\Pi)$ = max{$\ell$, 快照发布高度}；策略的 L1 高度已包含在 $\ell$ 中 | 简化 |
| §2.1 状态表、§3.3、§4.1 | REC 只记录创建时的 L1 高度 $h$，该代禁任集为 $\mathsf{Ban}(h)$ | S2：所有禁任集合都写成"某个 L1 高度上的登记" |
| §4.2 ProveForce | 清退编号为 $e_U=e(\lambda_U)$；论证改为"floors only grow" | 不再依赖委员会填写的数字 |
| §5 G7、接受的代价 | G7 改为 $e_x\le e(\lambda_\beta)$；"任何一次有效 Force"都能清退它之前退役的 position | 同上 |
| 第 1 页 | 接口表删去 "Minimum check" 行，只剩 2 个调用、2 类回报；核心思路第 5 条改写 | 接口变小 |
| §1 | Force 的 $i_{\rm src}$ 补上"或 PENDING 中保存的 source" | 第二轮遗留的局部问题 |
| 例子 | U41 在 L1 高度 3100 被包含，地板 3100，读取高度 $\max\{2600,3100\}=3100$，所以用 $\mathcal C_8$ | 例子跟着规则走 |

**对设计的影响**：原先委员会可以给新分支指定任意最小快照（只要不减且已激活），现在固定为"update 被包含时已激活的最新快照"。这正是第二轮附录 B (R7) 已经建议的默认做法。退出的安全论证不变。

## 2 校对：从整体看

### 2.1 抽象是否清晰、干净

**清晰。** 三轮下来，接口收敛成下面这样：

| 方向 | 内容 |
|---|---|
| 装 | update 携带表 $S$ 和顺序 $\mathcal L$；执行时安装 $\mathcal P_\beta=(S_\beta,\mathcal L_\beta,\lambda_\beta)$，其中地板 $\lambda_\beta$ 由 L1 决定 |
| 查 | $\Pi=\mathsf{Plan}(\mathcal P_\beta,\beta,r,E,i_{\rm src})$；$\mathsf{Leader}(\mathsf{AUTH}_{\beta,r,g},t)$ |
| 报 | `SubmitQuit`、`SubmitOffense`、`ProveForce` |

**更干净了**：

- Part II 的有效性定义回到 V1–V5，不再引用 Part III 的检查；
- Part III 不再有"最小值"这个需要额外单调性检查的数字；
- 所有 L1 读取（快照、$g=0$ 的禁任集、REC 的禁任集）都统一成"在某个 L1 高度读"。

Part III 第 1 页的接口表只剩 4 行。

### 2.2 抽象是否合理

**合理，而且比上一轮更自然。** "一个分支不能用早于其创建时刻的 L1 状态来排班"是一句不需要解释动机的不变量。退出边界直接从它推出：

> 任何一次有效 Force $U$ 都关闭旧分支；此后所有 plan 读取 L1 的高度都不低于 $\lambda_U$，所以凡是 $e_x\le e(\lambda_U)$ 的 position 都不会再进入任何委员会。

一个副作用需要知道：Force 后新分支至少使用 update 被包含时已激活的快照，其中可能有刚激活的新成员。这些成员从快照激活起就处于 Active 状态、一直跟随 $G$，不需要额外准备，所以我认为没有问题。

### 2.3 能否更简单：留给下一轮的思路

1. **在 L1 提交时拒绝 stale update。** `Eff(U)` 在提交时就已确定，直接拒绝 stale update 后：
   - 控制日志里只剩会执行的 update；
   - Part III 的 `ProveForce` 不再需要 `Eff`；
   - Part II/IV 中"stale 整体 no-op"的语义可以收为"无效"。

   代价是要改 Part II 的 Def 2.3、Protocol 3 第 1 步、§7.3 的例子、附录 B (R1)(R2)，以及 Part IV §1.1。读者仍需理解"目标版本必须匹配"，只是检查提前了，所以概念负担并不减少。收益中等，改动面较大，**建议你决定后再做**。
2. **descriptor 去冗余**：$\nu$ 改为 $(\Pi,A,q^{\rm open},\mathit{cause})$，其余字段由它们导出。收益小，改动多处示例，不建议现在做。
3. **$\Pi$ 与 $\mathsf{AUTH}_{\beta,r,0}$ 同名两义**：为了与 Part I 的 $\mathsf{AUTH}_g$ 对齐而保留，不建议改。

### 2.4 局部问题（措辞，不影响结论）

- Part III 接受的代价写着 "Release needs an effective Force, any one, ..."，读起来别扭。建议改为 "Release needs some effective Force after the retirement, ..."。
- Part II (R7) 说 "Every executed update also raises the L1 floor"。严格说是"不降低，通常抬高"：同一个 L1 块里的两条 update 地板相同。
- Part II 附录 B 的 WITHDRAW 效果写作 "the new L1 floor clears retired stake"。更准确的说法是：让 Part III 在 `ProveForce` 和持有期之后清退。
- Part III 核心思路第 2 条可以补上地板："the later of its reference's L1 citation and its branch's floor"。

## 3 需要你确认的设计决定（累计）

| 来源 | 决定 | 可否单独回退 |
|---|---|---|
| 第一轮 | $g=0$ 的禁任集取 $\mathsf{Ban}(\ell)$ | 可以 |
| 第一轮 | 附带双签的 quit 先禁任，再冻结 REC | 可以 |
| 第一轮 | 有效 offense 使该 position 强制退役 | 可以 |
| 第二轮 | 每个分支一条循环顺序，不再按轮次换排列 | 可以 |
| **第三轮** | 最小快照编号改为 L1 地板，委员会不能再选择 | 可以：在 update 中重新加回 $e^{\min}$ 字段即可 |

## 4 检查范围与未处理

- **编译与渲染**：v4.9、v6 及四份 diff 都编译通过。
- **全文检索**：确认没有遗留 $\mathcal P_{\rm new}$、$e^{\min}$、$\mathcal P^\circ$、minimum check、`\emin`。
- **逐段通读**：Part III v6 全文按编译后的文本读过；Part II 读了 Def 2.1–2.3、地板段和附录 B。
- **Part IV v0.1 仍是旧接口**，现在差得更多：它没有 $\mathcal L$，仍使用 `NextLive` 和 $\mathit{TB}_r$。上一轮提的同步任务卡针对的是 v4.8，需要换成 v4.9 的版本。
- **快速导读任务卡**同理，应改为针对 v6。
- `README.md` 和 `build.sh` 仍指向 v4.6 和 v3。
