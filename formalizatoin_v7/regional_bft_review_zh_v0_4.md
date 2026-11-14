# 区域 BFT 抽象 v0.4：漏洞与不清楚之处

日期：2026-11-01。
状态：审查稿。本文不修改协议正文。
对象：[提纲](BFT_outline.md) 与 [v0.4](regional_bft_abstraction_zh_v0_4.md)。

---

## 0. 结论先行

**站得住的部分：**

- **安全很干净。** 物理链的一致性来自 BFT；虚拟链是物理链与 L1 的纯函数，所以也一致。时钟、节奏、Espresso 都不进入安全证明。
- **leader 碰不到 deposit。** deposit 由高度决定，物理块里根本没有它。
- **v0.3 的三个问题消失了：**
  - $K$ 的取舍（v0.3 §6.2）不存在了；
  - 时间戳下界（v0.3 §8 第 2 条）可以放进投票，因为链式 Simplex 不重提旧块；
  - 孤立的沉默 leader 不丢高度，v0.3 表中 $K=1$ 时"损失 29%"的是时间，不是高度。

**问题集中在两处：** 活性的边界（末块、执行失败、等节奏）；与派生的接口（补位批次、锚点 origin）。

| 编号 | 问题 | 不修的后果 | 处理 |
| --- | --- | --- | --- |
| H1 | 高度 $H_s$ 之后没有合法的块 | 一个拜占庭 leader 可以让 session 永远收不了尾（已模拟） | v0.4 §4.5 收尾块 |
| H2 | 交易只做基本检查，而 Exec 可能拒绝整块 | 一个拜占庭 leader 让后面所有 leader 都算不出状态根，session 停住 | v0.4 §3.3 $\mathsf{Exec}^{*}$ |
| H3 | 补位高度没有批次 | cursor 过不去，$D$ 与 $G$ 分叉，任何一次补位都走到 RESUME | **要改 Part 1**（§4） |
| H4 | $\mathit{os}(h)$ 不是全局函数 | RESUME 之后 origin 倒退，派生换掉这个高度的块 | v0.4 §2.3 |
| H5 | 等节奏与 Simplex 计时器冲突 | 每个 slot 烧掉约 5 个视图，约四分之一的 slot 当场决定不了 | v0.4 §4.6 门控 |
| H6 | 时效检查不单调 | 换成要重提旧值的协议，会永久死锁 | v0.4 BFT-11 |
| H7 | "$\le1/3$"含等号 | 两份冲突的决定证书可以同时存在 | v0.4 改为 $3F_s<W_s$ |

**还没解决、需要决定的：** D1（mini-block 与 sequencer 密钥规则）、D2（共识单位与用户延迟）、D3（节奏的实现）。见 §3。

---

## 1. 会破坏目标的问题

### H1 末块收不了尾

**问题。** 提纲要求 $h>h(P)$，而 session 在 $H_s$ 结束。Chain Growth 的前提是"有合法的下一块"，到 $H_s$ 之后这个前提不成立。

普通高度不怕这一点：下一个 leader 扩展它，新块的决定证书决定整条前缀。末块后面没有块，只能靠自己所在的视图决定。而一个视图可以既没有决定证书，也没有跳过证书。按 [CGB 的规则](https://decentralizedthoughts.github.io/2026-04-23-deconstructing-simplex/)：计时器到期前拿到 VC 的成员发 Final，到期时没发 Final 的成员发 $\bot$，两者互斥。

**例子。** $n=4$，$f=1$，法定 3 票，$\Delta=50$ ms，计时器上限 150 ms。$d$ 是拜占庭成员，也是视图 $w$ 的 leader，提议高度 $H_s$。这段时间在 GST 之后，所有消息延迟都不超过 40 ms。

| 时刻（ms） | 事件 |
| --- | --- |
| 0 | $b$、$c$ 进入视图 $w$ |
| 20 | $a$ 进入视图 $w$（入口差 20 ms，不超过 $\Delta$） |
| 140 | $d$ 发出高度 $H_s$ 的块 |
| 145 | $a$、$b$、$c$ 收到并投票。发给 $b$、$c$ 的票被延迟 40 ms |
| 146 | $d$ 只把自己的票发给 $a$ |
| 150 | $a$ 收齐 3 票（自己、$d$、$b$），计时 130 ms，发 Final，并转发 VC |
| 150 | $b$、$c$ 计时到期，发 $\bot$ |
| 155 | $b$、$c$ 收到 VC，但已发过 $\bot$，不能再发 Final |
| 之后 | Final 只有 1 票，$\bot$ 只有 2 票，$d$ 都不发。**既没有决定证书，也没有跳过证书** |

所有诚实成员都拿着 $\mathrm{VC}_w$ 进入视图 $w+1$。诚实 leader 只有两条路：

- 扩展 $\mathrm{VC}_w$：$H_s$ 之后没有合法高度；
- 从更早的证书重新提议 $H_s$：需要 $\mathrm{SC}_w$，而 $d$ 不投 $\bot$ 就凑不齐。

此后每个视图都超时。按提纲的规则模拟：$H_s$ 永远决定不了，后继 session 拿不到锚点，只能等 Force（约 20 分钟）。

**攻击条件：** $d$ 当上 $H_s$ 的 leader 并控制好时机。每个 session 的机会约等于拜占庭权重占比。

**之前的文档。**
- Part 3 的 DRAIN 正是为此设的，[时间依赖稿](../formalization_v6/timing_dependencies_and_simplex_zh.md) §3.1 也说要保留。但那里说的是"末块所在 view 被 skip"。被 skip 反而不要紧，有 $\mathrm{SC}_w$ 就能从更早的块重新提议。真正的问题是没有 $\mathrm{SC}_w$。
- v0.3 的 G3.4 与[学习笔记](../learning/zh-CN/16_simplex_and_regional_bft.md) §6 说末块不必等后继视图，这只在好情形下成立。

**修法。** 收尾块（v0.4 §4.5）：高度不变、没有交易、不产生虚拟块。同一调度下加上这条规则，模拟结果是 165 ms 时所有诚实成员决定。

### H2 基本检查与执行之间有缝

**问题。** 提纲的投票不执行本块，只查"每笔交易有效（可以 revert，但有效）"。而下一块的状态根要执行它：$\mathsf{Exec}(B_{\rm prev}.s,\mathrm{Dep},B_{\rm prev}.\mathit{txs})$。"有效"有两种读法：

| 读法 | 查什么 | 问题 |
| --- | --- | --- |
| A 无状态 | 解码、签名 | 查不出 nonce、余额、块内 gas。而[规范](../megaeth_specs/specs/protocol/sequencing.md#transaction-admission-and-ordering)要求节点验证 nonce 与余额，原生执行会拒绝整块 |
| B 带状态 | 再加 nonce、余额、累计 gas | 这些要依次模拟全部交易，等于执行了本块。状态根晚一块的好处没有了 |

**读法 A 的后果。** 块 $B$ 带一笔 nonce 错误的交易，得到证书。下一个 leader 折叠时 Exec 返回 `INVALID`，提纲的折叠没有定义这种情况，算不出状态根。所有诚实 leader 都卡住。若 $B$ 已决定，再也绕不过去。**一个拜占庭 leader 停住整个 session。**

**修法。** 定义全函数 $\mathsf{Exec}^{*}$：执行被拒时，同一高度、同一 origin，只留 deposit。这不是新规则，是[派生本来的规则](../megaeth_specs/specs/rollup/derivation.md#channel-retention-after-execution-rejection)，所以 $G$ 与 $D$ 仍然一致。

V4 取什么，规范也给了判据（[block-records Rationale](../megaeth_specs/specs/block-certification/block-records.md#rationale)）：派生的拒绝分两类。

| 类型 | 例子 | 代价 | 放在哪 |
| --- | --- | --- | --- |
| 批次被丢弃 | 零长度条目、deposit 类型、Interop 激活块带交易 | 整个窗口停住 | V4 必须查 |
| 执行被拒绝 | nonce、余额、块内 gas | 只损失这个高度的内容 | 留给 $\mathsf{Exec}^{*}$ |

**连带结论：**

1. **`Final` 不等于交易生效。** 拜占庭 leader 的块可能已决定，交易却全部作废。用户要看 $\mathsf{Exec}^{*}$ 之后的结果，本地执行就能算出。Part 0 的 $\mathsf{Conf}$ 要改。
2. **诚实 leader 必须先模拟**（§4.3 第 4 步），否则一笔坏交易毁掉整块。
3. **没有新的攻击面：** 拜占庭 leader 最多浪费自己的 slot，这和提议空块一样。

### H3 补位高度没有批次

**问题。** 提纲把补位块叫 deposit-only，而 Part 1 §2.10 规定 deposit-only 高度不提交批次。三条规则合起来就出问题：

1. Batch Inbox 只按 cursor 连续接纳（G-BIN-1）。补位高度没有批次，后面的普通高度也上不去。
2. 派生要等 sequencing window 结束（示例约 12 小时），才给缺批次的高度补块。
3. 补的块用[另一条 origin 规则](../megaeth_specs/specs/rollup/derivation.md#same-height-deposit-only-substitution)：能前进就前进，相当于 $T_{\rm dep}=0$。而 $\mathit{os}(h)$ 滞后 $T_{\rm dep}$，所以第一个补位高度的 origin 就可能不同，L1 attributes 与状态根也就不同。

**后果。** 任何一次补位都走到：$D\ne G$，紧急状态，RESUME。而补位在这个模型里是常态，$b_t=1$ 秒、$T_{\rm vc}=0.25$ 秒时，连续 4 个沉默 leader 就会出现。

**修法。** 补位块与派生的回退块要分开，虽然内容都没有普通交易：

- Batcher 为补位高度显式提交空记录：`BlockRecord{block_number=i, epoch_num=os(i), transactions=[]}`；
- Certifier 认证这些高度；
- 派生会接受这样的记录：origin 连续（c1），drift 满足（c3）。

连续的空记录用 span batch 压缩，开销很小。

v0.3 §6.3 的"一处更正"已经指出两者 origin 可能不同，当时的结论是不影响安全。现在补位块要结算，所以有影响。

### H4 $\mathit{os}(h)$ 不是全局函数

**问题。** 提纲说任何高度都有固定的 origin。这只在锚点的 origin 正好在公式上时成立。RESUME 的锚点来自派生的填充块，填充的 origin 能前进就前进。

**例子。** $T_{\rm prep}=20$ 分钟，$T_{\rm dep}=25$ 分钟。锚点的 origin 约是 $\mathit{ts}(h_{\rm res})-20$ 分钟，全局公式给下一高度的 origin 约是 $\mathit{ts}-25$ 分钟，**倒退了约 25 个 L1 块**。派生见到 `batch.epoch_num < e`，当场把这个高度换成 deposit-only（Δ5），$D\ne G$。

**修法。** $\mathit{os}(h)$ 从锚点算起（v0.4 §2.3）：先不动，等公式追上来。锚点落后时每个高度前进 1。这与 v0.3 的 $\mathrm{Next}$ 相同，只是写成了闭式。

例子中 origin 冻结约 5 分钟，期间块里没有新 deposit。这不会推迟 deposit：它们本来也要等 $T_{\rm dep}$。

### H5 等节奏与 Simplex 计时器冲突

**问题。** [Chained Simplex](https://decentralizedthoughts.github.io/2026-05-20-from-single-shot-to-chained/) 说内层不用改。但 CGB 的规则 1 是进入视图就启动计时器，同时 leader 发提议；好情形的证明依赖这一点。而这里 leader 要等到下一个 slot。

**后果。** $b_t=1$ 秒，$\Delta=50$ ms，$\delta=20$ ms 时：

- 一个空视图约 170 ms，每个 slot 烧掉约 5 个视图，每个都要全员广播 $\bot$；
- 能提议的是 slot 翻转时碰巧在任的 leader，不再是"每块一个 leader"；
- 下一个提议要带上这些跳过证书；
- slot 翻转落在视图最后 $2\delta$ 内的概率约 24%，这时只有 VC，没有决定。

**修法。** 契约 BFT-9：等节奏不算失职。默认做法是超时从 $\mathrm{Gate}_v$ 起算。计时器不影响安全，所以这一改只需要重新论证活性：诚实成员的门控相差不超过 $\Delta$，因为 VC 会被转发（梗概）。其他做法见 D3。

### H6 时效检查不单调

**问题。** V2 会从"满足"变成"不满足"。标准的外部有效性要求：对一个诚实方有效，就对所有诚实方一直有效。这个模型放宽了这一点，只对"每次提议都是新块"的协议没有问题。

**例子（Tendermint 类，$n=4$，$f=1$）。** 诚实成员 $a$、$b$ 在第 $r$ 轮锁定了 $X$。

1. 下一轮 leader 按协议重提 $X$，但已过时效，诚实成员 $c$ 投 nil。$d$ 不投。$X$ 只有 2 票。
2. 再下一轮 leader 提新值，$a$、$b$ 因为锁而投 nil。新值也凑不齐。
3. GST 之后也一直如此。

**修法。** 契约 BFT-11：要么只扩展不重提；要么重提时跳过时效。后者有先例：[CometBFT 的 PBTS](https://github.com/cometbft/cometbft/blob/main/spec/consensus/proposer-based-timestamp/README.md) 只对首次提议检查 `timely`，因为上一轮已经查过。

**窗口定错了不致命。** PBTS 曾因为 `MSGDELAY` 低估而可能永久停住，后来改成随轮次增大。这里每次提议都是新块，而且 slot 开始时提议的块有 $b_t+\Delta_{\rm m}$ 的余量，不需要自适应。

### H7 "$\le1/3$"必须是严格不等式

$W=3$，$F=1$，$Q=2$。法定集合 $\{a,d\}$ 与 $\{b,d\}$ 只交于拜占庭成员 $d$，两份冲突的决定证书可以同时存在。所以 $3F_s<W_s$ 不能放宽。

---

## 2. 提纲里不清楚、v0.4 定下来的地方

| 编号 | 问题 | 决定 |
| --- | --- | --- |
| U1 | 验证窗口里的 $\tau$ 是什么时刻 | 首次收到提议的时刻 $\tau_{\rm recv}$。验证本身可以慢，否则执行时间会让诚实块过期 |
| U2 | leader 从求 Meta 到发出的时间算不算 | 算。$\Delta_{\rm m}=\Delta+T_{\rm build}$ |
| U3 | $\mathrm{slot}(\tau)\le h(P)$ 时 Meta 没有定义 | 返回 `WAIT`，等到 $\mathrm{NotBefore}(P)$ |
| U4 | $\mathrm{slot}(\tau)>H_s$ 时怎么办 | 封顶在 $H_s$。这个块不能带用户交易（V5），否则带着很旧的时间戳选交易 |
| U5 | $\mathrm{Dep}(h)$ 只有 deposit 吗 | 还有 L1 attributes 与系统交易，从不为空。块头输入（如 gas limit、prevRandao）也要是 $\mathit{os}(h)$ 的函数 |
| U6 | "状态根"是哪个时刻的 | 执行本块**之前**。与原生块头的状态根差一个高度。所以决定证书不证明本块执行后的状态；末块的执行后状态没有本 session 的签名（有收尾块时除外），`Enter` 要重放验证。另外 $B_{\rm prev}.s$ 是根不是状态，成员要保留完整状态，还要按分叉分别保留 |
| U7 | 没有交易时 leader 还提议吗 | 要。deposit-only 块要等后面的物理块才最终，不提议虚拟链就不前进 |
| U8 | dummy 块算不算物理块 | 不算。视图、物理块、虚拟高度是三个坐标（v0.4 §3.1） |
| U9 | Meta 与状态映射带 $v$、$\tau$ 参数 | 除 Meta 依赖 $\tau$ 外，结果都与它们无关。由此可知 leader 身份不能进入执行：手续费接收方要是固定地址 |
| U10 | "静态"相对什么而言 | 写成名单种子揭晓之前固定。$r_{\max}$ 取最坏情况，所以敌手看到种子再选，$\Lambda$ 也成立。安全不用静态 |
| U11 | L1 可读在 GST 之前也成立吗 | 不需要。读不到就 `WAIT`，只影响活性 |
| U12 | 固定节奏是否必要 | 不必要。以太坊会漏块。$\ell^*$ 的 $\max$ 形式不依赖它，只需相邻 L1 块间隔下界不小于 $b_t$，上界 $\delta_{\rm slot}$ 替换 c3 中的 $b_{\rm L1}$。闭式公式才需要固定节奏 |
| U13 | 时效查不查祖先，外部验证者查不查 | 都不查。一个块只要有证书，就有诚实成员查过 |
| U14 | "可替换"到什么程度 | 规范层可替换，不是运行时切换。CheckFINAL 与证据检查要进入 GFR 与 L1 合约，换协议是一次规则版本升级 |
| U15 | 空档大于配额 | 提议封顶在 $H_s$，后继 session 接着补。每过一个 session 就要一次跨区域交接 |
| U16 | 旧证书丢了怎么办 | A1 是有界部分同步，旧消息都在 GST 之后 $\Delta$ 内到。真实网络会丢。BFT-10 要求证书持久化并可取回（[Partial Synchrony variants](https://decentralizedthoughts.github.io/2025-12-19-cc-under-harsh-ps/)） |
| U17 | 提纲说"相对 v0.2" | A1–A4 在 v0.2 与 v0.3 中相同，以 v0.3 为基线 |

---

## 3. 需要决定的问题

### D1 mini-block 与 sequencer 密钥规则

**冲突。** MegaETH 规范要求：

- 每个 EVM 块至少有一个 mini-block，它是状态同步的最小单位（[blocks](../megaeth_specs/specs/protocol/blocks.md)）；
- 每个 mini-block 的签名必须恢复到 `sequencerAt(block_number)`，否则节点拒绝（[sequencing](../megaeth_specs/specs/protocol/sequencing.md#sequencer-authority)）；
- 节点不重新执行，直接应用 sequencer 签名发布的状态差量（[state-sync](../megaeth_specs/specs/protocol/state-sync.md)）；
- 换密钥有最小延迟（[SequencerRegistry](../megaeth_specs/specs/system-contracts/sequencer-registry.md)）。

而补位块没有 leader，没有人能签它的 mini-block 与差量；每块换 leader，也不可能每块改一次 registry。v0.3 §8 第 1 条只提到其中一半。

**建议。** 在 DRS 下，EVM 块的权威改为"决定证书加纯函数派生"。mini-block 签名只用于 leader 的实时流（软确认），不再是块有效性条件。补位块由节点本地算出，状态同步要支持这类块。这要改 Blocks、Sequencing、State Sync 三页。

system address 是另一个角色：oracle 的维护交易可以作为普通 $\mathit{Txs}$ 经任何 leader 进入，拜占庭 leader 最多拖一个 slot。

### D2 共识单位与用户延迟

**现状。** $h>h(P)$ 使物理块不能快于 $b_t$。用户确认约等于"等 slot + $3\delta$"：$b_t=1$ 秒、$\delta=20$ ms 时平均约 0.56 秒。BFT 的 $3\delta$ 优势用不上，MegaETH 的 10 ms mini-block 也不在模型里。

**选项：**

| 选项 | 内容 | 问题 |
| --- | --- | --- |
| a 保持（**建议**） | 共识按 EVM 块；leader 流式发 mini-block 作为软确认 | 软确认可撤销，语义依赖 D1 |
| b 同高度多个物理块 | 放宽到 $h\ge h(P)$，虚拟块拼接同高度的 $\mathit{Txs}$ | **不是免费的扩展。** $\mathsf{Exec}^{*}$ 是整块作废，后面一个拜占庭 leader 的坏交易，会作废同高度已决定的交易。要么投票前执行，要么改派生为跳过坏交易 |

### D3 节奏的实现（Part 3 的选择）

以下四种都满足 BFT-9。

| 做法 | 沉默 leader 的代价 | 对协议的改动 |
| --- | --- | --- |
| 门控（v0.4 默认，**建议先写这个**） | 约 $T_{\rm vc}$，孤立时不丢高度 | 改内层计时器 |
| 长超时 $T_{\rm to}\ge b_t+3\Delta+T_{\rm build}$ | 超过一个 $b_t$，丢一个高度 | 只改参数 |
| FVS，一个 slot 一个视图 | 正好一个 $b_t$，丢一个高度 | 换协议；session 开始时要能跳过没有人进入过的视图 |
| 空转块：等节奏时不断提议空物理块 | 约 $T_{\rm vc}$ | 不改协议；每秒约 $b_t/2\delta$ 个视图的签名开销 |

**FVS 值得评估。** 它的主要缺点是视图长度固定，而这里的出块节奏本来就固定，缺点不明显。拜占庭 leader 无论如何都能提议空块，所以 slot 利用率的最坏情况在四种做法下一样，都是 $1-\rho$。差别只在崩溃的 leader 带来多少延迟。

---

## 4. 连带影响

| 文档 | 位置 | 改动 | 原因 |
| --- | --- | --- | --- |
| Part 0 | §2.4 | L1 固定节奏与可读性是新假设 | §2.1 |
| Part 0 | 定义 3.2 | $G$ 的块多一种：session 隐含的补位块 | §3.4 |
| Part 0 | S3(a)、定义 3.4 | 原生有效按 $\mathsf{Exec}^{*}$ 理解；$\mathsf{Conf}$ 确认的是顺序 | H2 |
| Part 1 | §2.4 | 区域块改为物理块；DATA、DRAIN 对应普通块、收尾块 | §3.2 |
| Part 1 | §2.5 | 槽位与冲突关系由所选协议定义，按视图 | BFT-5 |
| Part 1 | §2.10、§4.6.1、§4.7.1、R27 | **补位高度要提交批次，Certifier 要认证** | H3 |
| Part 1 | §3.1、§3.2 | Exec 改 $\mathsf{Exec}^{*}$；DerivOK 的 V1–V3 由构造保证，V4 不再是投票条件 | H2 |
| Part 1 | §3.3 | CheckFINAL 由所选协议提供；证书可以落在收尾块上 | BFT-4 |
| Part 1 | §4.1 | Builder 只出普通交易，并先模拟 | §4.3 |
| Part 1 | §4.3.8 | Kill 的"分叉"按 $\mathsf{Exec}^{*}$ 之后的原生块比较，否则 $\mathsf{Exec}^{*}$ 作废的块会被误判 | H2 |
| Part 1、Part 5 | GFR 总述，G-GFR-4 | "GFR 不生成 deposit-only"要改：追加普通块时，同时把空档记成补位描述符 $(i,\mathit{os}(i),\varnothing)$。GFR 仍不执行 | §3.4 |
| Part 1、Part 5 | Eligible (b) | 允许跳高；收尾块不进入 $G$ | §3.2 |
| Part 1、Part 5 | Eligible (d)，$q^{\rm open}$ 的公式 | 物理块没有 $\xi$。(d) 改为只要求 $q>q^{\rm open}(\nu)$，保留"最早在开出块的下一块被接纳" | 不依赖 Espresso |
| Part 1、Part 5 | Eligible (e) | 应比较 $h(P)+1$，这是最早过期的高度。其实已由 $\mathsf{Expire}$ 事件蕴含 | 跳高 |
| Part 1、Part 5 | `ResumeChain` 的 $h_f$ | $G$ 里没有原生块哈希。要在 RESUME 处执行 $G$ 的后缀，按 $\mathsf{Exec}^{*}$ 之后比较 | U6 |
| Part 2 | Plan、Leader、SubmitOffense | 去掉 $K$；Leader 按视图；证据验证按协议版本 | §3.1、BFT-5 |
| Part 3 | 全部 | 换成满足 §6 契约的协议 | §6 |
| 学习笔记 §6，v0.3 G3.4 | 末块 | "不必等后继视图"只在好情形成立 | H1 |
| MegaETH 规范 | Blocks、Sequencing、State Sync | 见 D1 | D1 |

---

## 5. 代价（不是漏洞，但要心里有数）

1. **deposit 至少晚 $T_{\rm dep}$。** 只用已最终化的 L1，所以约 25 分钟。leader 不再能审查，但用户等得更久。
2. **L1 finality 停滞会停住区域。** "L1 总是活的"遮住了这一点。$T_{\rm L1}=19$ 分钟、$T_{\rm dep}=25$ 分钟时，只能多容忍 6 分钟（v0.3 §6.3、§8 第 5 条）。
3. **所有投票成员都要在一个 slot 内重放整块。** $T_{\rm exec}\lesssim b_t-2\Delta$ 是吞吐上限。MegaETH 现在的节点[不重新执行](../megaeth_specs/specs/protocol/state-sync.md)，只应用 sequencer 发布的状态差量。成员要检查状态根，就不能只靠 leader 给的差量。这一条可能是最硬的约束，要实测。
4. **状态根晚一块。** 轻客户端要多等一个块，才能拿到带证书的执行结果。
5. **时序博弈。** leader 可以拖到接近超时才提议，以收集更多 MEV，这是它的[最优反应](https://decentralizedthoughts.github.io/2026-03-07-simultaneous-agreement/)。想要决定证书时上限约 $T_{\rm to}$，只要 VC 时上限是 $b_t+\Delta_{\rm m}$（G1.3）。
6. **仍然需要区域内转发交易。** 不转发时，交易要等接收它的成员当 leader：20 个等权成员、$b_t=1$ 秒时平均约 20 秒。比 $K=10$ 时的 200 秒好，但不够（v0.3 §8 第 3 条）。
7. **长停机后提议很大。** 提议要带所有中间视图的跳过证书。用聚合签名，或者评估 Complex（学习笔记 §11.2）。
8. **名单公开，每块换 leader。** 定向 DoS 下一个 leader 的成本很低。A1 把这种情况算作 GST 之前，模型里看不出来。

---

## 6. 核对范围

**核对过的：**

| 内容 | 怎样核对 | 结果 |
| --- | --- | --- |
| $\mathit{os}(h)$ 的闭式等于 v0.3 的递推 | 脚本，20000 个高度，锚点在公式上、超前、落后三种 | 相等；origin 连续；每个 L1 块的 deposit 恰好一次。锚点落后过多（超过 $D_{\rm drift}$）时才违反 drift，与锚点条件一致 |
| $b_t>b_{\rm L1}$ 不可行 | 脚本，$b_t=24$ 秒 | 闭式每步跳 2；递推则越落越远，违反 drift |
| V2 的 $\exists\tau''$ 形式等于闭式 | 脚本，整数毫秒网格枚举 | 没有差异 |
| H1 | 按 CGB 规则 1–6 与链式提议规则模拟，$n=4$，实际延迟最大 40 ms | 没有收尾块：没有任何决定。有收尾块：165 ms 决定 |
| $\Lambda$ 的上界 | 在抽象模型上随机测试，$r_{\max}=0$–$5$（$b_t=1$ 秒，$T_{\rm fin}=0.15$ 秒，$T_{\rm vc}=0.3$ 秒） | 成立，且偏松：$r_{\max}=2$ 时观测最坏 3.25 秒，上界 4.75 秒 |
| H3、H4、H2 的派生行为 | 读 MegaETH 规范原文 | 结论来自规范文字，没有运行 mega-espresso |
| H6 的 PBTS 先例 | 读 CometBFT 规范原文 | 规范明确写了重提的块不查 `timely` |

**没有核对的：**

- **组合命题没有完整证明。** v0.4 §8 是梗概，没有机器检查。
- **门控对 Simplex 活性证明的改动**只有梗概（H5），任何论文都没有这个变体。
- **§6.4 的实例表**只有 Chained Simplex 一行对照原文证明读过。其余是初步判断，特别是 FVS 的 session 开始问题、Kuplex 的证书唯一性。
- **所有数字都是示例。** $T_{\rm build}$、$T_{\rm exec}$、$\delta$ 没有实测。H5 的 5 个视图与 24% 是算术估计，不是模拟。
- **H1 的模拟**只覆盖这一个调度，它用来说明问题存在，不证明收尾块在所有调度下都够。后者靠 §8 的推导。
