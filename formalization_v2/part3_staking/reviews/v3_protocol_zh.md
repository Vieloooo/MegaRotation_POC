# Part III v3：最终局部闭合检查

对象：`versions/part3_v3.tex`。检查了当前全文与 v2→v3 差异，范围限于此前必要 findings 的闭合及相关规则有无回退；不重新认证 Part II 的全部证明，不检查实现代码。本次未修改稿件。

## Summary

**Final Verdict：PASS（在本次限定范围与正文所列条件下）。v1/v2 的必要缺口均已闭合；未发现仍未解决的 Critical Error 或 Justification Gap，也未发现本轮修订使固定快照、minimum+Force 退出边界或 REC 规则回退。无需为本次审查再增加版本或功能。**

**List of Findings：无新增必要 findings。**

## Detailed Verification Log

| 检查项 | 当前直接依据 | 判定 |
|---|---|---|
| Join 最低 bond | 行 167–169：“Deposit at least the minimum bond within the admission cap” | **通过。** 新 join 不能利用仅检查 incumbents 的移除步骤绕过最低 bond。 |
| Exit 生命周期准入 | 行 172–175：“already included in a frozen snapshot, with no exit record or pending exit” | **通过。** 排除 pending join、已退休位置和重复排队；按上一快照权重记自愿 exit 的规则不再面对这些不存在的成员。未首次入选者仍走行 170–171 的取消路径。 |
| Mandatory retirement 先清队列 | 行 176–180：“First remove incumbents ... and clear their queue entries. Then select each remaining FIFO's longest mature prefix” | **通过。** 强制退役条目在 voluntary prefix 选择前消失，不占自愿预算、不堵队头；与行 185–186 一致。 |
| 所有退休位置的 exit record | 行 182–187：“Every omitted incumbent, voluntary or mandatory, receives its first exclusion epoch $e_x=e$ and an exit record.” | **通过。** 强制退役的正余额也有同一提款入口，且以后永久缺席。v1 原缺口保持闭合。 |
| 参数公共来源 | 行 82–86：“Genesis fixes quota ... $K$ ... $\Delta_i$ ... $\varphi_i$”，并定义 $F_i=\varphi_i\sum w_o$；行 295–296 将其绑定到 authority | **通过。** quota 与 BFT 参数的唯一来源没有回退。 |
| Genesis base | 行 281–286：“For $(0,0)$ set $E=A=A_0$ ... complete initial L1/Espresso replay state” | **通过。** 有初始 anchor、足够的 L1 reference、初始 policy/minimum 和完整初始状态；没有依赖不存在的 regular predecessor。 |
| 完整 replay checkpoint | 行 250–254：“complete replay state and source commitments”，包括 NEXT/PENDING tail、policy/minimum、control-log position 与 pending updates、准确 Espresso position | **通过。** 此次只是将标签解释得更清楚，没有退回仅凭 `G` tip 的检查。 |

### 固定快照、退出边界与 REC 没有回退

> 行 79–86：“old certificates still use 100”；行 131–132：“All voting members and certificate weights stay fixed, including banned proposers.”

余额和历史权重仍分离；行 191–192 保留不改已固定 session/plan，行 304–306 保留同一 session 在恢复过程中不重选 snapshot、region、selection reference 或 quota。没有引入 slash 后重算旧证书分母。

> 行 232–243：“minima never decrease”及“effective Force installs $m\ge e_x$”；行 246–250 要求完整历史和匹配当前 branch 的真实 activation。

旧 branch 的 speculative programs/plans 被关闭，同时未来 authority 的 minimum 不下降，两个必要条件都保留。stale update 不清退。行 256–275 保留首次 clearance 单次记时、足够持有期、deadline 前有效证据先处理、残余余额只释放一次，以及历史数据保留/转交义务。

> 行 312–321：“at most one immutable” REC；ban 集合取交易执行前的 L1 状态，必须有完整前代 quit proof，进入 generation 前等待 finality。

REC 仍只改变 proposer eligibility，不改 voting quorum。行 323–327 保留 initial generation 的 cutoff ban 集合，以及后续 exclusion 对 proof delivery/authority update 的依赖。没有将 ban 追溯写入已经冻结的 schedule。

> 行 334–336：“Old plans, votes, locks and timers do not carry over ... The new session uses the verified execution state at its retained anchor.”

原先过宽的“No BFT state crosses”已收窄，旧活动状态与可继承执行状态得到区分；没有误删历史 DATA、证据或保留 anchor 的状态。

### 本次 PASS 保留的条件

- 本文是条件化授权与退出机制，不保证 multisig/外部服务永远及时行动；无 eligible region/proposer 时仍按正文等待，Force closure 不构成无条件立即启动保证。
- 证据窗口依赖 `T_ev`、有效 slash 提交后的 L1 服务界与部署时钟 margin；不覆盖无限隐藏或无界离线调查，不承诺赔付所有历史 offense。
- 固定历史权重下的 Byzantine bound、Part II 的同步/持续服务、认证的 L1/Espresso 历史、sound replay 和历史数据可用性仍是前提。
- 公共可预测的 weighted schedule 不等于抗 grinding、抗身份选择或公平 beacon；本次未要求或认证这些额外性质。

以上条件均为正文已声明的范围，不是本次审查临时添加的假设。
