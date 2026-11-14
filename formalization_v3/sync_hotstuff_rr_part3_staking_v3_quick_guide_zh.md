# Part III v3 中文快速导读

[英文正文](sync_hotstuff_rr_part3_staking_v3.tex) · [六页 PDF](sync_hotstuff_rr_part3_staking_v3.pdf)

这篇文章只做三件事：**从 L1 取一份固定的 stake 名单，提前排好下一 session 的 region 和 leader，再规定资金怎样进入、退出和被罚。**

先记住最重要的区别：**选主可以用较早的块；真正接棒，仍从前一个 session 的最后一块继续。**

## 1 先读懂四个编号

一个 session 写成 $s=(\beta,r)$，其中的一个 view 写成 $(\beta,r,g,t)$。

| 记号 | 人话 | 什么情况下变化 |
|---|---|---|
| $\beta$ | 当前是第几个生产分支版本 | 有效 Force 关闭旧分支，进入新版本 |
| $r$ | 第几轮 session | Regular rotation 时加一；**不是 region 编号** |
| $g$ | 这个 session 内第几次恢复 | Recovery 时增加 |
| $t$ | 这个 generation 内第几任 leader | 正常区内换 leader 时增加 |
| $i$ | 具体哪个 region | 由候选顺序和资格检查选出 |
| $p,s$ | 前一个 session、我们要准备的新 session | Regular 情形下 $p=(\beta,r)$，$s=(\beta,r+1)$ |

例如 `(0,2,1,3)`：分支 0，session 2，恢复代 1，term 3。初始 view 是 `(0,2,0,0)`。

小写 $g$ 是恢复编号；大写 $G$ 是**已经全局接受的整段 DATA 链前缀**。二者没有编号上的对应关系。Region 可以生产到 $G$ 前面，但这些后续块尚未获得全局最终性。

## 2 两个 anchor 是理解全文的关键

| 记号 | 它是什么 | 拿来做什么 |
|---|---|---|
| $A_s$ | Session $s$ 的初始执行 anchor | 从它的执行状态继续出块 |
| $T_s$ | Session $s$ 的最后一个 DATA | 完成后交给下一 session |
| $E_s$ | 给 session $s$ 选主的参考块 | 从它取权重快照的参考高度和 seed |
| $h(B)$ | 块 $B$ 的 BFT DATA 高度 | 用来表示本链的数据位置 |
| $\lambda(B)$ | 块 $B$ 引用的 finalized L1 高度 | 用来查 L1 配置 |

对 regular successor，正文规定：

$$
\boxed{E_s=A_p}\qquad\boxed{A_s=T_p}.
$$

读成人话：**用前驱的起点预排下一棒；用前驱的终点正式交棒。**

沿用你的例子：

$$
(0,0)\longrightarrow(0,1)\longrightarrow(0,2).
$$

假设 `(0,0)` 结束在 DATA 100，`(0,1)` 结束在 DATA 120：

| 为 `(0,2)` 做什么 | 用哪一块 | 什么时候知道 |
|---|---|---|
| 确定权重与 seed | $E_{(0,2)}=A_{(0,1)}=T_{(0,0)}$，即 100 | `(0,1)` 开始时 |
| 确定接续的执行状态 | $A_{(0,2)}=T_{(0,1)}$，即 120 | `(0,1)` 完成交接时 |

所以 `(0,2)` 可以提前同步、计算 leader，但它实际从 **120 后面**开始生产，绝不会退回 100。

这组等式只用于 regular rotation。Force 后的首个 session，则把 $E_s$ 和 $A_s$ 都设为保留的 $G$ 末块。Session 内 recovery 选出的新 opening，也不会改写已经固定的选主参考块 $E_s$。

## 3 第一条公式决定用哪份权重

$$
e_s=\max\{m_\beta,\ e(\lambda(E_s))\}.
$$

从里往外读：

1. $\lambda(E_s)$：选主参考块引用的是哪个 L1 高度。
2. $e(\ell)$：在 L1 高度 $\ell$，最新已经激活的快照编号是多少。
3. $m_\beta$：这个分支至少要用第几份快照。
4. $e_s$：最终给 session $s$ 选定的快照编号；实际名单和权重是 $\mathcal C_{e_s}$。

这里 **$e$ 是配置编号，$a_e$ 才是这份配置的 L1 激活高度**：

$$
e(\ell)=\max\{e:a_e\le\ell\}.
$$

例如，$\mathcal C_7$ 在 L1 高度 2000 激活，$\mathcal C_8$ 在 3000 激活。参考块引用 2500，则 $e(2500)=7$。

- 若 $m_\beta\le7$，用 $\mathcal C_7$。
- 若有效的退出 Force 已把 $m_\beta$ 提到 8，则用 $\mathcal C_8$，不能再用 7。

**$m_\beta$ 是 v3 为安全退出增加的最低快照编号。** 它防止 Force 从旧块重新启动时，又把已经退出、可以提款的资金选回来。它随 branch 固定，不是各节点实时查询“最新 stake”。

快照一旦为 session 固定，后来的 L1 更新不重算它。上面的 DATA 120 即使引用了新快照，也只会影响随后为 `(0,3)` 做的选择。

## 4 先选 region 再读 leader 公式

当前是 $p=(\beta,r)$，准备的是 $s=(\beta,r+1)$。

**选 region：** 读 `randList(beta,r)`，这是 multisig 规定的候选顺序。从前往后，选第一个同时满足以下条件的 region：

- $S_\beta(i)=\mathrm{ON}$：该分支允许 region $i$ 服务。
- $W_i>0$：目标快照里，这个 region 有正的总权重。

例如顺序为 A、B、C、D，A 为 OFF、B 为 ON 且有权重，就选 B。`randList` 的名字不表示现场再随机抽一次，也不表示 region 按 stake 比例获得服务机会。

**选区内 leader：**

$$
L_s(g,t)=\operatorname{WeightedRand}\!\left(
\mathcal W_s^g,
H(\texttt{LEADER},\mathit{chainId},h_s^{\rm sel},\beta,r_s,g,t)
\right).
$$

| 记号 | 在这条公式里做什么 |
|---|---|
| $L_s(g,t)$ | Session $s$ 在 generation $g$、term $t$ 的 leader |
| $h_s^{\rm sel}=h(E_s)$ | 参考块的 BFT 高度，也就是基础 seed |
| $r_s$ | **目标** session 的轮次；上面例子是 $r+1$ |
| $\mathcal W_s^g$ | 已选 region 中，本 generation 允许担任 leader 的成员及原权重 |
| $H$ | 哈希函数；把共同输入变成共同的抽样输入 |
| `LEADER`、`chainId` | 说明用途和所属链，避免不同场景混用输入 |
| `WeightedRand` | 给定权重和输入后，确定性地选出一个成员 |

对 `(0,1) → (0,2)`：**region 列表用 `randList(0,1)`；leader 哈希用目标 `(0,2,g,t)`。** 基础 seed 是 100，不是 120，也不是 L1 高度 2500。

例如候选权重为 `(60,30,10)`，对应 `[0,60)`、`[60,90)`、`[90,100)`。若哈希派生出的抽样整数是 73，就选第二个成员。这个“73”只是示例，并非实际计算出的哈希结果。

## 5 权重 禁任和投票不要混淆

| 记号 | 含义 |
|---|---|
| $o$ | 一笔有唯一 ID 的 stake position，绑定资金所有者、运营密钥和 region |
| $B_o$ | 这笔 position 现在还剩多少实际抵押品 |
| $w_o$ | 它在某份固定快照中的权重 |
| $V_i(\mathcal C_e)$ | 快照 $\mathcal C_e$ 中 region $i$ 的成员集合 |
| $W_i$ | 这个 region 的完整快照权重之和，用于区域资格等判断 |
| $J_e$ | 快照截止时记录的禁任 leader 密钥集合 |
| $J_{s,g}$ | Session $s$ 的 generation $g$ 实际使用的禁任集合 |
| $\operatorname{REC}(\beta,r,g)$ | L1 上固定该 recovery generation 授权与禁任集合的记录 |

$g=0$ 使用快照中的 $J_{e_s}$。$g>0$ 使用相应 REC 固定的集合；REC 根据前一 generation 的完整 quit proof 创建，等待 L1 最终化后使用。

**禁任只影响以后谁能做 leader，不重算当前 session 的投票委员会。** 例如原投票权重 `(60,30,10)` 中第二人被该 generation 禁任，选主候选变成 `(60,10)`；投票权重和历史证书的验证方式仍然不变。加权抽样中的总权重 $W$ 此时是 70，不能与完整 region 权重 $W_i=100$ 混为一谈。

同样，$B_o$ 被 slash 从 100 扣到 70，不会把历史 $w_o=100$ 改成 70。

## 6 退出相关记号只要按这条顺序读

$$
\text{申请退出}
\to\text{记录 }e_x
\to\text{有效 Force 且 }m\ge e_x
\to\text{L1 验证清退结论}
\to\text{等待追责窗口}
\to\text{释放剩余资金}.
$$

- $e_x$：从第几份快照开始，这笔 position 永久不再出现。**它是快照编号，不是提款时刻。**
- $m\ge e_x$：新分支及其后继不能再使用包含该 position 的旧快照；同一次有效 Force 关闭旧分支的 BFT 职责，包括跑在 $G$ 前面的 session 和预排计划。
- $t_c$：L1 首次接受已验证清退结论的真实时间。等待期从这里算，不从提交退出申请时算。
- $T_{\rm hold}$：清退之后还要保留资金多久，以便证据交付、核验和执行罚没。

普通 regular rotation 不等这次清退；**v3 的最终提款要等一次符合条件的批量 Force，再完成追责等待。**

时间公式是：

$$
T_{\rm hold}\ge 2T_E+T_{\rm pub}+T_{\rm ev}
+T_{\rm L1}^{\rm stake}+\varepsilon_{\rm clock}.
$$

| 记号 | 留时间给哪件事 |
|---|---|
| 第一个 $T_E$ | 节点获知旧分支已经关闭 |
| $T_{\rm pub}+T_E$ | 已可用的完整区域结果提交到 Espresso，并完成最终化及交付 |
| $T_{\rm ev}$ | 额外调查、收集和提交相关证据 |
| $T_{\rm L1}^{\rm stake}$ | 有效 slash 交易实际提交后，L1 处理至最终化 |
| $\varepsilon_{\rm clock}$ | 时钟换算的安全余量 |

这些是文中明确的服务和时钟条件，不能把公式理解成已经选定了一个固定天数，也不覆盖永远隐藏的证据。

## 7 剩余参数用到时再查

| 记号 | 人话 |
|---|---|
| $M$ | 一个长 staking epoch 包含多少 L1 区块 |
| $P$ | 快照截止点比激活高度提前多少个 L1 区块 |
| $a_e=a_0+eM$ | 第 $e$ 份快照的激活高度 |
| $c_e=a_e-P$ | 快照截止高度；读取该高度之前的完整 L1 状态 |
| $\mathrm{TB}$ | 一个 session 可以增加多少 DATA；终点为 $h(A_s)+\mathrm{TB}$ |
| $K$ | 一个 term 最多增加多少 DATA |
| $j$ | 一个 term 内的 fresh block 位置，用来识别同一位置的双签 |
| $\Delta_i$ | Region $i$ 的同步通信延迟界限 |
| $\varphi_i$ | 配置给 region $i$ 的故障权重比例上界，小于 $1/2$ |
| $F_i=\varphi_iW_i$ | 允许的恶意权重上界；正文的证书要求签名权重 **大于 $F_i$** |
| $\bot$ | 没有可选成员，不代表可以自行另选一个未授权者 |
| QC | 提案的投票证书；单凭 terminal QC 还不能完成 regular handoff |
| FINAL receipt | 区域提交回执；terminal DATA 的完整回执用于正常交接，但它本身不等于已经进入 $G$ |

尤其注意：$c_e$ 是 **L1 截止高度**，$t_c$ 是 **清退结论被接受的时间**；$T_s$ 是 **terminal DATA 块**，$T_{\rm hold}$ 是 **等待时长**。

## 8 对照英文正文的位置

| PDF 页码 | 先看什么 |
|---|---|
| 1 | 编号、两个 anchor、资金与权重的区别 |
| 2 | 两条核心公式，以及 region 和 leader 怎样选 |
| 3 | 周期快照、进入退出队列、双签类型 |
| 4 | 最低快照编号、批量清退、等待时间 |
| 5 | Genesis、regular、recovery、Force 的实际流程 |
| 6 | 与 Part II 的接口调整，以及明确保留的条件和代价 |

Plan 是提前排班；authority 是绑定配置、实际 anchor 和合法交接的授权上下文；Ready / Enter 才让程序开始 BFT 行为。提前知道 leader，不等于提前取得出块权。公开高度哈希也只提供确定性排班，不自动提供不可预测的公平抽签。
