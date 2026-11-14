# Sync HotStuff RR Part II v4.3：原子 branch update 与 Protocol 3 修订说明

本文说明 `sync_hotstuff_rr_part2_rotation_v4_3.tex` 相对 v4.2 的设计变化。v4.2 原稿保留不动；v4.3 将本轮确认的三项修改统一落实到定义、Protocol 3、Session Manager、Force handler、例子、证明和控制策略附录。

## 1. 本版冻结的三项决定

### 1.1 每条 update 都是完整的原子 branch update

新的控制记录是

\[
U=(\mathit{chain},c,\theta_a,\beta_{\rm target},S_{\rm new}).
\]

- `chain`：链标识；
- `c`：单调递增的 update 序号，也是这次 update 的身份；
- `theta_a`：未来的 Espresso 激活阈值；
- `beta_target`：这条 update 想关闭的 branch version；
- `S_new`：新 branch 使用的完整 ON/OFF map。

`S_new` 不能缺省，也不能只写本次变化的几个地区。它必须给 `L_R` 中每个地区一个 ON/OFF 值；允许所有地区都为 OFF，但不允许“没有 map”。

本版不再有两类控制，也不再有独立的 `force tag`：

- 没有 status-only update；
- 没有 `f=bot`；
- 改 map 和 Force 是同一个原子动作。

说人话：**每次想改地区资格表，都必须同时结束当前 branch，并用一张完整的新表创建下一个 branch。**

### 1.2 stale update 整条无效

当 update 在激活块到来时：

- 若 `beta_target` 等于该 Espresso block prelude 开始时的当前 `beta`，update 成功执行；
- 若不相等，update 已 stale，整条 update 是 no-op。

stale 的结果不是“Force 失效但 map 仍生效”，而是：

\[
\text{不关 branch， 不增加 }\beta，\text{ 不安装 }S_{\rm new}，\text{ 不改变 Tail。}
\]

如果治理方仍想实施这张 map，必须读取新的当前状态，重新生成一条 update，绑定新的 `beta_target`。

### 1.3 每个 branch version 只有一张固定 map

Genesis 固定 `S_0`。每次成功 update 创建新 version `beta+1`，同时固定

\[
S_{\beta+1}:=S_{\rm new}.
\]

在该 version 关闭前，`S_beta` 永远不变。普通 regular handoff 只把 `r` 变成 `r+1`，不改变 `beta`，所以整个 regular-handoff 链始终使用同一张 `S_beta`。

这删除了旧稿中的历史 map 搜索：

- 不再有“从某个 Espresso 高度开始，寻找第一个 `S_q` 能选出地区的位置”；
- 不再用 receipt 到达时节点看到的最新 map；
- 不再需要 `Resolve_q`；
- 两个节点即使在不同时间收到 terminal receipt，也会用同一个 `S_beta` 选出同一个 successor。

## 2. Protocol 3 现在读什么、记什么、输出什么

### 2.1 两条外部输入

对每个新 finalized Espresso block `J_q`，Protocol 3 只消费两类逻辑输入：

1. `O_q`：截至 `J_q` 已在 Espresso 中公开的累计 regional object 集合；
2. `Lambda_q`：激活点第一次落在 `J_q` 的有效 atomic update，按控制日志顺序排列。

`Lambda_q` 写成 batch，而不是强行写成一条 update，是因为 Espresso timestamp 可能一次跳过多个不同的激活阈值。即使相邻阈值至少相隔 `T_update`，同一个 `J_q` 仍可能同时首次跨过多个阈值。

Protocol 3 不把完整历史 update log 当作每个高度重新读取的输入。L1 ingestion 层用 pending queue 和 control cursor 增量地产生 `Lambda_q`。

### 2.2 维持的最小 replay state

公开输出仍然只有两个东西，但状态机在相邻高度之间必须记住当前 map。因此文中显式写出内部 replay state：

\[
\mathcal X_q=(\mathrm{Out}_q,S_{\beta_q},\kappa_q).
\]

这里：

- `Out_q` 是公开输出；
- `S_{beta_q}` 是当前 branch 的固定完整 map；
- `kappa_q` 是已处理 update 的 cursor，用于防止遗漏或重复执行。

`S_beta` 和 `kappa` 是维护状态，不是第三、第四条外部输入，也不是 Protocol 3 的新增公开输出。选择 regular successor 时只用当前 `S_beta`，不回查历史 `S_q` 序列。

### 2.3 公开输出仍然是 `(G, Tail)`

\[
\mathrm{Out}_q=(G_q,\mathsf{Tail}_q).
\]

- `G_q`：已经 global-finalized 的连续 DATA chain；
- `Tail_q`：说明 `G_q` 后面的下一块由谁负责。

Tail 有两种形式。

#### NEXT

\[
\mathsf{Tail}=\tagg{NEXT}(\nu),
\qquad
\nu=(\beta,r,i,A,H,q^{\rm open},\mathit{cause}).
\]

它表示下一 session 已经完整确定：session id、地区、anchor、terminal height、Espresso reference floor 和原因都已确定。

#### PENDING

\[
\mathsf{Tail}=\tagg{PENDING}(\omega),
\qquad
\omega=(\mathit{cause},\beta,r',i_{\rm src},A,q^{\rm open}).
\]

它表示 branch、目标 turn index、source、anchor 和 reference floor 都已确定，但固定 map `S_beta` 在目标 authority 下选不出任何 eligible region。

因为 `S_beta` 在该 version 内不会再变，所以 PENDING 不会靠后面的普通 Espresso block 自动解决。只有下一条成功 atomic update 创建新 version 和新 map，才会重新尝试选择地区。

## 3. Protocol 3 的两步

### Step 1：处理 atomic update

设 `beta_0` 是 `J_q` prelude 开始时的当前 version。

1. 在 `Lambda_q` 中找第一条 `beta_target=beta_0` 的 update。
2. 若不存在：所有 due update 都 stale；`G`、`Tail` 和 `S_beta0` 不变，只推进 cursor。
3. 若存在第一条匹配 update `U*`：
   - 保留当前全部 `G`；
   - 关闭 `beta_0`；
   - 设置 `beta'=beta_0+1`；
   - 安装 `S_beta'=S_new`；
   - 根据旧 Tail 的 accepted count 决定目标 `r'`；
   - 用 `NextLive(i_src,S_beta';r')` 直接选择新地区；
   - 能选出地区则写 `NEXT(nu')`，否则写 `PENDING(omega')`。

Force cause 绑定 update 身份和激活块：

\[
\tagg{FORCE}(c^*,q).
\]

这比只写 `FORCE(q)` 更明确：同一个激活块内即使有多条 due update，也能知道究竟是哪一条造成了 branch change。

同一 `Lambda_q` 中其余 update 全部是 whole-update no-op。本 prelude 创建的新 `beta'` 不会在同一个 prelude 中再次被另一条 update 命中。

### Step 2：从 `O_q` 追加匹配 DATA

只要当前 Tail 是 `NEXT(nu)`，Protocol 3 就从累计 `O_q` 中寻找满足以下条件的下一块 DATA：

- 它由 `sid(nu)=(beta,r)` 产生；
- 它准确接在 `last(G)` 后面，高度连续且不超过 `H(nu)`；
- regional FINAL kernel 验证通过；
- 它引用一个已经 finalized、且严格早于当前 carrying block 的 Espresso header；
- 引用高度至少为该 session 的 `q_open`。

每接受一块 `B`，就把完整 session descriptor 绑定为该块的 session record：

\[
\sigma_G(B):=\nu.
\]

因此 session record 不是悬空的全局对象，而是 **对每个被接受的非 genesis DATA block 单独绑定的记录**。

若 `B` 不是 terminal DATA，Tail 不变，继续找下一块。若 `B` 是 terminal DATA，则进入 regular handoff。

## 4. Regular handoff：地区选择与 reference floor 分开

对 terminal DATA `T_{beta,r}`：

\[
i'=\tagg{NextLive}(i_{\beta,r},S_\beta;r+1).
\]

地区直接由固定 `S_beta` 和目标 authority 决定。`xi(T)` 不再用于选择地区。

下一 session 的 Espresso reference floor 仍为

\[
q^{\rm open}_{\beta,r+1}
=\max\{\operatorname{height}(\xi(T_{\beta,r})),
q^{\rm open}_{\beta,r}+1\}.
\]

这两个概念必须分开：

- `S_beta` 决定 **谁接班**；
- `q_open` 决定 successor 第一块 DATA 最低可以引用哪个 Espresso 高度。

`q_open` 不是 selection time，也不要求该高度的 header 在 handoff 时已经存在。因此：

- successor 可以只凭 direct terminal receipt 提前被 Assign、完成 Ready 并 Enter；
- 不需要等待 terminal DATA 先进入 `G`；
- 如果 `q_open` 指向尚未出现的未来高度，successor 只是在提出第一块有效 DATA 前等待一个不低于该 floor 的 finalized header。

这保留了 v4.2 的 speculative regular-handoff 能力，同时消除了“不同 receipt 到达时间看到不同 map”的分叉风险。

## 5. Force 与 PENDING 的例子

假设 Force 前：

\[
G=100\cdot101\cdots108,
\qquad
\mathsf{Tail}=\tagg{NEXT}(\nu_A),
\qquad
\operatorname{sid}(\nu_A)=(7,20),
\qquad
a(G,\nu_A)=8.
\]

Update 41 在 `J_501` 激活：

\[
U_{41}=(\mathit{chain},41,\theta_a,7,S_8),
\]

其中 `S_8` 把 A 设为 OFF、B 设为 ON。因为 target 7 与当前 `beta=7` 匹配：

- `G` 保持到 108；
- 新 version 是 8；
- A 的 session 已向 `G` 提交过数据，所以目标 index 从 20 变为 21；
- `NextLive(A,S_8;21)=B`；
- 新 Tail 为

\[
\tagg{NEXT}
\bigl(8,21,\mathrm B,108,108+\TB_{21},501,
\tagg{FORCE}(41,501)\bigr).
\]

若 `S_8` 中没有任何 eligible region，则 Tail 变成

\[
\tagg{PENDING}
\bigl(\tagg{FORCE}(41,501),8,21,\mathrm A,108,501\bigr).
\]

后续空 Espresso block 不会把它变成 NEXT。治理方需要再提交一条以 version 8 为 target、带完整新 map 的 update。若新 update 成功，则创建 version 9，并继续使用存下来的 target index 21；没有真正开始的 session 不会把 index 再加一次。

若 Update 41 激活前 version 7 已被另一条 update 关闭，Update 41 stale。它不会关闭当前 version，也不会把 A 改成 OFF。治理方必须针对新的当前 version 重提。

## 6. 文档结构调整

旧 §6.2 的控制接口不再等到 cross-region runtime 才定义。本版把以下内容提前到 §2.1、放在 global finality 之前：

- atomic branch update 的结构；
- update validity；
- control cursor；
- Espresso first-crossing / prelude 规则；
- stale whole-update no-op；
- `S_beta` 的 branch 内不变性；
- `NextLive` 的固定-map 语义。

原 §6.2 改成简短的 runtime 使用说明，只解释 Session Manager 如何消费已定义的 update 结果，不再重复定义控制协议。

## 7. 同步修改的范围

本版不仅改了 Protocol 3，还同步改了：

- abstract、overview 和角色表；
- L1 control 定义、符号表和 committee assumption；
- Espresso reference 的作用说明；
- session descriptor、block-bound session record 和 Tail；
- Protocol 4 / Session Manager 的 Assign、baseline 和 Ready；
- handoff packet 与 `VerifyHandoffHistory`；
- Protocol 6 / Force handler；
- Protocol 7 / observer replay；
- regular / forced / stale / zero-output / repeated-Force 例子；
- determinism、global-prefix、program-isolation 和 explicit-control 证明；
- storage / replay-state 说明；
- Appendix B 的 reference control policy；
- regular-handoff TikZ 图。

旧的 `public interpreter` 和 Table 7 没有重新引入。Protocol 3 继续直接产生公开 `(G,Tail)`。

## 8. 新设计的代价与操作要求

这项简化消除了历史 map 搜索和 map/Force 半成功状态，但引入了明确代价：

1. **任何 map change 都会关闭当前 branch。** 即使只是恢复一个空闲地区，或安排 maintenance，也会 Force；治理方应批量修改，不能把链上 map 当高频健康 telemetry。
2. **stale 会丢掉全部意图。** 旧 target 的 map 不会部分保留，必须重提。
3. **推荐 single-flight。** 同一控制方通常只保留一条未激活 update。第一条成功后 `beta` 会变化，后续仍绑定旧 `beta` 的 update 都会 stale。
4. **紧急操作可能被 in-flight update 拖慢。** 因此不要设置过远的激活时间；计划中的远期变化应先留在链下。
5. **PENDING 不会自动恢复。** 必须再来一条成功 atomic update；后续普通 Espresso block 不改变固定 map。
6. **完整 map 增加控制记录大小。** 换来的是每条 update 自包含，不依赖旧 map 才能解释。
7. **early regular start 仍保留，但首块 DATA 可能等 reference header。** Enter 与能够提出第一块有效 DATA 是两件事。

## 9. 构建与检查

- v4.3 TeX 已独立生成，v4.2 未覆盖；
- PDF 已成功编译为 66 页；
- 没有 unresolved reference、LaTeX warning 或 overfull box；
- 已检查 Protocol 3、handoff、Force handler、regular/forced-handoff TikZ 和 Appendix B policy table 的渲染。
