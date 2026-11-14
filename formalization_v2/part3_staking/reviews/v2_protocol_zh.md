# Part III v2：生命周期修订复查

范围：当前 `versions/part3_v2.tex`，只复核 v1 的两项缺口及 v2 新的生命周期规则；不重审未改动的 Part II 证明，不修改稿件。

## Summary

**Final Verdict：v1 的两项 Justification Gap 已闭合。v2 尚有两项很窄的队列准入 Justification Gap，需要最后版本明确；本次未发现已证实的 Critical Error。除此之外，已声明的条件退出与固定权重边界可在原假设下保留。**

### 必须补齐

1. **Exit 的有效请求者范围。** 未入选任何快照的 pending join、以及已退休 position，均不应进入按上一快照权重记账的自愿 exit FIFO。
2. **Join 的最低 bond 准入条件。** v2 只自动移除低于最低 bond 的 incumbents，不能再依赖这条规则排除不达标的新 join。

这两项均只需要明确状态前置条件，不需要新机制、额外活性定理或扩展篇幅。

## Detailed Verification Log

### 1. v1 的两项缺口已闭合

> 行 172–177：“Every omitted incumbent, voluntary or mandatory, receives its first exclusion epoch $e_x=e$ and an exit record ... A positive residual balance remains withdrawable under the same boundary rule below.”

**通过。** 强制退役与自愿退出现在都产生 §2.3 所需的 `e_x` 和 exit record；永久排除、消费旧请求、残余正余额采用同一提款边界均已明确。v1 的正余额强制退役者缺少提款入口问题已解决。

> 行 75–79：“Genesis fixes quota $\mathrm{TB}>0$, maximum DATA per term $K$, each region's delay bound $\Delta_i$, and fault fraction $0\le\varphi_i<1/2$. A session binds these constants and $F_i=\varphi_i\sum_{o\in V_i}w_o$.”

**通过。** 参数有了公共唯一来源，`F_i` 也由固定 fraction 与所选历史权重确定；行 284–285 将 genesis-derived 参数绑定进 authority。上一版“参数固定但值从何而来”的确定性缺口已解决。

### 2. Exit 缺少生命周期准入 guard：Justification Gap

> 行 164–166：“The owner enqueues one irrevocable exit. The position remains eligible until processed.”
>
> 行 169–170：“Charge joins their locked balances, and voluntary exits their weights in $\cfg_{e-1}$.”

文字没有限定这是已经被冻结快照纳入、且尚未退休的 position。对尚在 inbound FIFO 的新 position，`cfg_(e-1)` 中没有其权重，因此下一步如何计出队金额未定义。

最短轨迹：position `o` 尚未首次入选，owner 又提交 exit；两个请求在同一 cutoff 成熟。Join 按 `B_o` 计费，exit 却须查一个不存在的 `w_o(cfg_(e-1))`。如果 owner 先取消 join，行 162–163 也未明确处理另一个 exit queue entry。已退休者再次请求 exit 会产生同类不存在权重的问题。

**是否必须补：是。** 正文应明确 exit 请求只来自已有快照成员且尚未退休的位置；pending join 使用现有 cancellation 路径。这里“已纳入”以冻结快照为准即可，不必等待实际 session 分配或 snapshot activation。另需保持已有 invariant：mandatory retirement 的 exit 请求被消费且不占 voluntary budget；实现 FIFO prefix 时应按该 invariant 排除这些请求，而不能让其占预算或堵住队头。

### 3. 新 join 的最低 bond 条件缺失：Justification Gap

> 行 148–150：“Genesis fixes the minimum bond ... Each admitted position must fit a fresh budget.”
>
> 行 160–161：“Deposit, register region and keys ... enter the regional inbound FIFO.”
>
> 行 167–171：“First remove incumbents below the minimum remaining bond ... Include surviving positions with their remaining balances as integer weights.”

“fit a fresh budget”规定的是单项不能超过预算，并不要求达到最低 bond。v2 的自动移除明确只针对 incumbents，新 join 不属于这一集合。故按现有字面规则，最低 bond 为 10 时，一个存入 1 且符合 delay/budget 的新 position 可以进入下个快照，到再下一轮才作为低于最低 bond 的 incumbent 被移除。

这没有直接推出 escrow 损失，但使声明的 minimum-bond 准入规则不完整；v1 原先对 included positions 的统一最低余额过滤不能自动沿用到已经改写的 v2 顺序。

**是否必须补：是。** 明确 Join 入队/采用时满足最低 bond 即可；无须新增退出服务或引入部分余额变更。

### 4. 其余修订与可接受限制

- 行 245–246 只记录首次 clearance、重复 proof 不重启或回拨时钟；行 261 只释放一次。对旧窗口规则是明确化，没有发现新增提前释放路径。
- 行 229–233 保持“有效 Force 关闭旧分支 + minimum 排除所有未来 authority”的两个必要条件，并保留历史处罚和数据服务义务。对有明确 `e_x` 的位置，原条件论证未被本轮改写破坏。
- 行 240–243 现在要求完整 replay state 与 source commitments，列出 NEXT/PENDING tail、control cursor 和 pending updates；不再把单一 `G` tip 当作可继续 replay 的完整检查点。
- 行 270–275 明确 genesis base，行 284–285 绑定 genesis 参数。将 `G` 的解释前移、区分 initial execution anchor 与 recovery opening，均属有用的可读性修订。
- 行 323 的 “No plan or BFT state crosses” 过宽，可能被读作连已保留 DATA/执行状态及历史证据也不能继承；应按拟定修订限定为旧分支的 votes/locks/timers 等活动状态。行 264 已要求数据保留或可靠转交，不能被该句抵消。这属于必要的范围澄清，不需要重写 Force。
- 证据服务有界、multisig 实际行动、冻结权重下的 Byzantine bound、公共 hash schedule 不保证公平等限制仍明确。没有理由将本稿加强成无条件提款、无条件活性或抗身份选择保证。

**本轮收束：** 除上述两个准入 guard 和已计划的范围/可读性澄清外，没有发现需要扩大 v3 修订范围的问题。本结论仅覆盖本报告所述的局部复查范围。
