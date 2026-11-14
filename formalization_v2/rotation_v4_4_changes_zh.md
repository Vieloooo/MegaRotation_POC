# v4.4 变更：单 update 激活与删除原 Section 8

v4.4 从 v4.3 复制，v4.3 保持不变。控制日志采用以下核心简化：

\[
|\Lambda_q|\le 1.
\]

即对每个 Espresso block $J_q$，在该 block 首次跨过激活阈值的有效 atomic update 最多一条。因此 $\Lambda_q$ 只有两种情况：

- $\Lambda_q=\varnothing$；
- $\Lambda_q=\{U_q\}$。

这是一条明确的 model assumption，不从 $T_{\rm update}$ 推导。

## Definition 2.3

Definition 2.3 现在只表达：

1. 没有 activated update：control state 不变；
2. 有唯一的 $U_q$，但其 target 不是当前 $\beta_0$：整条 update stale，什么都不改，只推进 control cursor；
3. 有唯一的 $U_q$，且 target 等于 $\beta_0$：关闭 $\beta_0$、创建 $\beta_0+1$、安装完整 $S_{\rm new}$，再执行 Protocol 3 的 Force transition。

不再有：

- 同一 block 多条 activated update；
- first-matching update；
- 同一 prelude 内 update 排序或连续 Force；
- “其余 due update 怎么处理”的规则。

## 同步修改

- Protocol 3 的第二项输入改成 empty-or-singleton $\Lambda_q$；
- Protocol 3 Step 1 直接检查唯一 update 的 target；
- Protocol 6 直接读取唯一 activated update，不再从完整 log 中选择 first match；
- handoff verification、closure evidence、determinism proof 和 explicit-Force proof 改为 unique activated update；
- forced-handoff TikZ 图同步删除 batch/first-match 逻辑。

## 删除原 Section 8

原 `The external observer` 整节及 Protocol 7 已删除。Protocol 3 本身就是公开 replay / global-finality rule，不再另外写一个只负责调用 Protocols 3 和 6 的 wrapper protocol。

原 Section 8 中的 `Execution-layer requests and stored data` 与 `Faithful publication` 也全部删除。Part II 不定义 execution layer，不讨论 request queue、reproposal 或 application payload 的生成；区域块中的 payload $x$ 只被视为 opaque application payload。

同步清理了：

- 全文对 `alg:external`、`sec:observers` 和 `sec:requests` 的引用；
- runtime architecture 图中的 Execution layer 模块；
- discarded suffix 的 request reproposal 规则；
- execution-layer request persistence / eventual inclusion 的责任描述。

后续章节自动重新编号；原 `Examples and guarantees` 成为新的 Section 8。

Protocol 3 的公开输出仍然只有

\[
\mathrm{Out}_q=(G_q,\mathsf{Tail}_q),
\]

regular handoff 的固定 $S_\beta$ 选择和 $q^{\rm open}$ reference-floor 规则均不变。
