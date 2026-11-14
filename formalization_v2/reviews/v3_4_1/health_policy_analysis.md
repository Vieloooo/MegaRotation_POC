# Multisig Health Controller：可审查的有限干预政策与证明边界

日期：2026-10-18。本文是 v3.4.1 后续设计分析，不修改协议源文件，不选择尚待用户决定的 delayed-Force / old-output acceptance 方案，也不声称完整安全性或活性目标已经证明。

核对对象：`formalization_v2/sync_hotstuff_rr_part2_rotation_v3_4_1.tex`（下称 V），及 `formalization_v2/revisions/v3_1_frozen/model_and_assumptions.tex`（下称 M）。行号均指这两个当前文件。

## 1. 结论与需要明确接受的政策限制

**可以把既有 honest, coordinated committee 细化成一个有限签发预算的算法，而不改变任何已经签发、提交的 MSIG 的执行语义。** 这个算法能证明：如果 accepted prefix `G` 从某时刻起不再增长，之后由 committee 新制造的 MSIG 抢占只有有限次；配合有限 OFF 租期，还能消除永久的健康误判排除。它不能单独排除无限 AUTO、解决 deadline 前不能形成完整结果的问题，或证明请求被接受。

这是明确限制 committee **以后签什么、提交什么**，不是要求解释器忽略一个已经有效提交的干预。预算耗尽后，新干预请求进入未签发的待办项，不具有控制效力。如果产品要求在同一个 `G` 下无限次 emergency MSIG 都必须获签，本政策不可用；不能一面允许该旁路，一面证明干预有限。可预留有限 emergency 额度，但每次仍须消耗总额度。

健康控制和失效预言机也必须区分：超时、同步落后、交付失败、人工报告可以成为**发起有限干预的依据**，不能成为“该区域永久失效”的正确性定理。V:2176–2182 已明确说明 delivery miss 不识别原因；V:1614–1615 也明确说明 ON 不认证未来 synchrony。

## 2. 冻结接口与没有获得的新能力

以下约束完整保留：

| 接口 | 不可改动的含义 |
|---|---|
| 未来时间 | 每次实际提交仍满足 `theta_a > s + T_L1 + epsilon_E + eta`；不能拿签名时刻代替实际提交时刻。M:272–283。 |
| 不可变性 | 已 final 的记录不改 target、activation、配置或控制序号；修订只能是新的未来记录。M:46–59。 |
| first crossing | 在 `J_q(theta_a)` 的 prelude 执行；不等待 quota 完成、健康检查通过或新区域 Ready。M:61–71。 |
| target 与优先级 | 先独立 STATUS/CLEAR，再第一条匹配 entry-beta 的 MSIG，再否则 AUTO。普通 regional handoff 不使 MSIG 过期；版本不匹配以及同一 prelude 中未被选中的其他 due Force 仍按原规则消费为 stale。V:1607–1610、2214–2237。 |
| stale | stale MSIG 不实施 attached configuration；不能退还预算后自动重签新 beta。 |
| retained anchor | 仍保留触发块 payload 前的全部 `G`；controller 不指定旧高度、不保留私有保护前缀。V:2239–2243，M:199–207。 |
| OFF / ON | OFF 只影响后续 selection，不撤销已固定 REGULAR witness；ON 不 clear cooldown，不重开旧 beta。M:80–84。 |
| 固定配置 | 不改变既有 AUTH、snapshot、quota、window 参数；不把健康判断转换为新的 regional fault assumption。 |

本文只新增 honest controller 的持久化本地政策状态。它不是新的 canonical replay state，也不是要求所有 verifier 重新评估 controller 的健康判断。若未来希望由 L1 合约强制这些签发预算，需另写合约接口；该实现并非本文证明的前提。

## 3. 输入与持久化状态

controller 仍使用现有的完整 finalized L1 / Espresso history。健康报告是运营输入，可以不存在、迟到或误判；下面的次数上界不依赖报告正确，也不引入新的有界健康观测通道。

持久化以下状态：

1. `seen_G`：已重放的 accepted tip，包含 chain、DATA height、hash 与 provenance。只向后继 accepted prefix 前进，绝不因重启、换 beta 或换 controller 实例倒退。
2. `spent_F[g]`、`spent_L[g]`、`spent_C[g]`：在 accepted-tip key `g` 下已消耗的 MSIG、OFF-lease、CLEAR 额度。固定有限上限分别为 `B_F >= 1`、`B_L >= 0`、`B_C >= 0`；最小政策可取 `B_F=1, B_C=0`。
3. `signed[id]`：每个控制 intent 的精确 payload、key、预算预留、签名释放、实际提交、finalization 与 canonical processing 状态。unknown outcome 仍占额度；重试只能复用同一个 exact record。
4. `lease[i]`：该区域当前唯一的一对独立 `OFF_i / ON_i`；两个记录均 canonical processed 前，不允许下一份健康 OFF 租期覆盖它。
5. 可选但推荐的 `service_i`：重放得到的、**由 i 生产且已接受进 G 的 DATA 数量**；及每个 `service_i` 值下有限的 region lease 额度 `spent_L_i[service_i]`。DRAIN、QC、Ready、raw inclusion、其他区域的 DATA 均不补充这个额度。

所有会让有效签名离开 coordinated service 的路径都经过同一 journal。**先持久化扣款，再释放签名或调用提交接口。** 仅在 L1 finalization 时扣款太晚：多个 pending 签名以及延迟广播可以同时绕过那个计数器。服务重启时从 journal 恢复，不开新预算。

`seen_G` 只能由真实 accepted DATA 推进。以下事件一律不重置预算：AUTO / MSIG、beta/r/g/t 改变、失败窗口、冷却开始或结束、STATUS、CLEAR、旧结果 raw publication、stale target、重新同步到同一个 tip、controller 更换实例。

## 4. MSIG 签发算法

```text
On an authorized request to interrupt the current production arrangement:
    advance the controller's authenticated replay monotonically
    g := seen_G
    if spent_F[g] == B_F:
        retain an unsigned operational request; return
    reserve one F token durably before releasing any signature
    read beta from that same verified replay context
    choose an activation satisfying advance announcement at actual submission
    sign and submit one MSIG(chain, beta, theta, sequence, attached_changes = empty)
    record the exact operation; reconcile unknown outcomes without minting another id
```

上述 target 可能在观察后已经变旧。到点时仍按既有解释器处理：匹配就生效，旧 target 就消费为 stale；**两者都不退款**。原来针对 A 的 beta 也可能在生效前已通过 regular handoff 轮到 B；由于 beta 未变，它仍可能关闭 B。这个副作用由现有语义决定，不能靠健康 controller 的初衷消除。

新 MSIG 不附带状态变化，仅为下面的可证明 OFF/ON 租期避免一个特定排序陷阱；冻结接口仍允许附带状态的旧记录，解释器照常执行。需要隔离某区域时，单独调用独立 STATUS 租期流程。预算不足不会降低已提交 MSIG 的优先级，也不会把到期指令改成 soft request。

## 5. OFF / ON：用既有不可变记录实现有限健康租期

### 5.1 为什么不能只写“恢复后 ON”

如果健康报告可以误判，或恢复报告可以一直不出现，“OFF 后等待恢复证据再 ON”允许一个完全可服务的区域永久被排除。冻结假设没有 perfect failure detector，也没有健康报告最终正确的保证。因此，把所有健康误判最终修复写进 theorem premise 就是在新增环境前提。

候选政策把健康 OFF 定义成有限租期。租期结束的 ON 仅表示**撤销临时排除**，不是认证该区域已经恢复。若业务要求没有恢复证明就永不 ON，则只能证明“持续 eligible 区域”的条件性性质，不能保证所有模型内可服务区域最终获服务。这是必须明说的选择。

### 5.2 具体签发顺序

健康管理范围内的 baseline flag 为 ON；永久行政移除另行明确记录为目标范围变更，不伪装成健康误判修复。

```text
RequestHealthLease(i):
    refuse if lease[i] is still outstanding
    require a remaining global L token under seen_G
    if regional-service fairness is desired:
        also require an L_i token under current service_i
    durably reserve both applicable tokens

    first submit an independent STATUS ON(i), with a finite future theta_on
    wait until that exact ON is finalized and its record is available

    at the actual prospective OFF submission time s_off:
        choose theta_off with
            s_off + T_L1 + epsilon_E + eta < theta_off < theta_on
        if no such threshold remains:
            submit no OFF; the token remains spent; retain the harmless pending ON
        otherwise:
            submit independent STATUS OFF(i) at theta_off

    do not issue another health OFF for i until both records are canonically processed
```

`theta_on` 可以在首次提交时选得较远，以留出正常操作余量；若 controller 自身处理拖延，则按上面的检查放弃 OFF，而不是假定一个未给出的固定处理上界。ON 的实际提交仍独立满足 advance announcement。

ON **先提交**不意味着先激活。记录激活顺序由 `(theta_a,c)` 决定，M:50–51 明确允许 activation 与 sequence 的先后不同。若 Espresso 一次跳过两个 threshold，两条都是独立 STATUS，先 OFF 后 ON，同一 prelude 的最终 flag 为 ON。这种情况下租期没有产生有效隔离，政策承认这一点，不谎称租期至少持续某个实时长度。

先确保恢复 ON 已经提交并 final，再允许 OFF，可避免“先 OFF，之后 controller 停止工作而忘记提交 ON”。不需要为未提交的未来动作另加活性保证。未知 OFF 提交结果必须继续作为 pending 跟踪；不能另发同一租期的重复 OFF。

### 5.3 附带 OFF 的时间跳跃陷阱

设一条旧 MSIG 在 `theta=10` 激活并附带 `OFF(A)`，独立 `ON(A)` 的 threshold 为 20。下一块从 9 跳到 25，两者首次 crossing 相同。Protocol 5 **先处理所有独立 STATUS**，故先 ON；随后匹配 MSIG 的附带 OFF 才实施。最终 A 是 OFF，虽然 ON 的 threshold 更晚。该 ON 已消费，不会再恢复 A。

因此，不可仅用 `theta_on > theta_MSIG` 为 attached OFF 证明有限租期。上述新政策只使用 independent OFF+ON。对旧 attached OFF 的修复必须在观察到其实际 canonical processing 后，再提交新的 future independent ON，见第 7 节。

### 5.4 全局进度不能给别人的误判无限续期

仅按 global `G` 补充租期额度仍有缺口：A 不断提交 accepted empty DATA，每次都给 controller 新额度，C 却可被反复 OFF，永远没有 accepted output。

为排除此种健康政策自身的饥饿，租期同时消耗 `service_i` 下的有限额度；只有 i 自己的 accepted DATA 才更新它。若 C 只有有限 accepted output，则 C 只拥有有限个 service key，故只能遭受有限个新的健康 OFF。最后一个租期结束后，C 的 health flag 保持 ON。这个结论独立于 A/B 是否持续推进 G，但不排除 C 继续受 AUTO、MSIG 或 selector 影响。

该限制是真实政策：一个反复报告异常、但没有新 accepted DATA 的区域最终会耗尽健康 OFF 信用。若希望永久隔离它，必须明确作出行政移除，随之放弃覆盖该区域的 all-regions 服务目标；不能偷偷走无限“紧急续期”。

## 6. CLEAR、恢复报告与 control ingestion

最小政策不签 CLEAR，等待现有 `C_auto` 自然结束。若保留提前清除能力，则一次 CLEAR 也须先扣 `B_C` token，绑定所观察的 cooldown 证据并由既有授权规则提交。过时证据导致无用的 CLEAR 也不退款。健康报告不能直接改 cooldown；ON 仍不清除它。

恢复报告可以解释运营决定，但不能自动重置 F/L/C 预算。最小租期算法无须提前 ON；若增加提前 ON，它只能是该租期内有界数量的额外独立记录，并且原 ON canonical processed 前仍不能开始下一租期，避免旧 ON 意外覆盖新 OFF。

controller 本地没有完整 control prefix 时停止新的签发决策，不把未知状态当作健康或故障。已经提交的记录仍按其原 target / time 等待 canonical processing。其他 verifier 也照原规则缺证据则暂停 replay；controller 缓存不变成新的共识来源。M:46–59 与 V:2269–2283 的完整 ingestion 规则全部保留。

## 7. 已在队列中的控制与政策上线

不能上线时把旧 queue 清空、标成 stale 或重新定时。区分两种情形：

1. **从 genesis 起执行政策。** 每个已释放签名都有预算项；每个健康 OFF 之前已有其恢复 ON。以下计数证明直接适用。
2. **在已有系统中启用政策。** 先冻结旧签发路径，盘点有限集合 `P0`：已 final 尚未激活、已实际提交但未 final、已经释放且可能被提交的授权签名。新算法的预算计数与这批旧记录分别记账。旧记录全部按原规则执行；不能声称它们本来就在新预算之内。

有限 backlog 可以造成任意大的有限延迟：其时间阈值可能非常远。因此只有“最终处理完”的结论，没有统一的 backlog 排空实时上界。`P0` 未知时不能给出可审计的数值上界；应该先恢复 coordinated service 的签发 journal，而不是让各 verifier 自行丢掉不认识的旧签名。

旧 independent OFF 及旧 attached OFF 可能没有恢复 ON。仅在所有已知旧控制都 canonical processed 后，由 controller 根据 baseline 再提交一批 future independent ON，才不会被某个更晚处理的旧 attached OFF 覆盖。该步骤需要 controller 实际继续读取公共历史并执行修复；若这一点没有从既有 correct-service 执行语义获得，就必须把**迁移完成**单独列为实现前提，不能从被冻结的运输界自动推出。对于已永久 OFF 的遗留状态，同样不能凭预算算法声称它已经得到修复。

本节的 finite-backlog 前提指一个已明确盘点的有限迁移状态，不是“未来错误干预总会自行停止”的环境假设。全新协议的 journal 从开始就排除这个未知集合。

## 8. 固定 G 下的精确有限性命题

**命题 H1（有限 committee 抢占）。** 设某个真实 accepted prefix `G*` 从时刻 `t0` 起不再增长，其高度为 `h*`。controller 在 `t0` 已观察的 accepted height 是 `h0 <= h*`；它采用上述 durable、monotone、无退款政策。设 `N0_F` 是 `t0` 前已释放、此后仍可能生效的 MSIG 数量。则 `t0` 之后新释放 MSIG 不超过

`B_F (h* - h0 + 1)`，

此后有效 MSIG 次数不超过 `N0_F + B_F (h* - h0 + 1)`。

**证明。** 在真实 G 不增长的区间，controller 能新观察的 accepted-tip key 只来自 `G*` 的有限祖先链。单调游标最多访问 `h* - h0 + 1` 个 key，每个 key 至多释放 `B_F` 个不同 MSIG。每条记录最多形成一次有效 Force；stale、重复或同一 prelude 中被消费的其他记录不会增加次数。已经释放的 `N0_F` 逐条原样保留。证毕。

这个公式特意处理 control/replay ingestion delay。只有 controller 在 `t0` 已经跟上 `G*` 时，才可把新提交上界简写为 `B_F`；不能把迟到观察到的多个旧 tip 当成多次真实新进度。

**命题 H2（健康 OFF 最终结束）。** 在 H1 的固定 G 条件下，假设所有健康 OFF 均按上述 independent lease 算法创建、没有永久行政 OFF，且没有未修复的旧 attached/status OFF。则健康 OFF 租期数量有限。在 growing Espresso timestamps 的既有 progress 条件下，每个已提交恢复 ON 最终 canonical processed；最后一份租期之后，各健康管理区域的治理 flag 为 ON。

**证明。** 全局 lease key 数有限，每个最多 `B_L` 个租期。每个 OFF 之前，已有一个 final 的未来 independent ON；其 threshold 有限。时间最终跨过它，且该区域不允许重叠租期；同块 threshold jump 也按独立 STATUS 顺序结束为 ON。新 MSIG 没有 attached configuration，不会在该 prelude 中把它再改回 OFF。有限个租期有最后一个。证毕。

**命题 H3（其他地区的增长不能无限补充 i 的 OFF）。** 若额外启用 per-region `service_i` 预算，且 i 只有有限 accepted DATA，则不论全局 G 是否增长，i 的健康 OFF 租期都有限；在 H2 的其他条件下，最终 health flag 持续为 ON。证明是对有限 service keys 作相同计数。

H2/H3 的结论是 **governance flag 为 ON**，不是永久 eligible。仍可能存在 AUTO cooldown、指定 authority 无正权重、已固定历史 selection 或新旧边界等待。尤其 AUTO 可以在固定 G 下无限次触发并重启 cooldown；这些命题没有把它偷偷纳入有限 committee 次数。

这些命题给出一个可接入后续 liveness proof 的接口：在假设“G 最终停止”后，先推出最后一次 committee MSIG 和最后一段健康 OFF 存在，再单独分析 AUTO / preparation / AUTH / regional result / acceptance。不能直接从它们推出矛盾；v3.4.1 的 slow-session trace 完全不用 committee 操作，仍然需要别处修复。

## 9. 哪些是协议限制，哪些会偷偷强化环境

| 可以明确规定为算法 | 不能无证明地冒充算法结论 |
|---|---|
| 所有签名路径先扣持久化 token。 | “委员会诚实，所以不会无限打断健康区域。” |
| stale / failed submission 不退款；G 之外的状态变化不刷新额度。 | “真正健康的区域迟早会被 controller 识别。” |
| OFF 前先提交恢复 ON；不重叠租期。 | “恢复消息一定在一个有界时间到达。” |
| 新 MSIG 无 attached configuration；旧记录原样执行。 | “先安排一个更晚 ON 就能覆盖所有旧 attached OFF。” |
| 某区域的 lease credit 只由它自己的 accepted DATA 补充。 | “别的区域一直增长，所以该区域也会获得 useful service。” |
| 上线时明确盘点、处理并修复旧 controls。 | “完整 L1 prefix 自动包含所有未广播旧签名。” |

M:309–311 的“committee leaves working regions enough uninterrupted service time”仍是冻结文字。本文提供其中一个可检查的固定-G实例化方向，而非把它解释成已经保证任意未来 duty 的成功。M:304–306 的“some sustained service interval”也没有被提升成 recurring successful intervals 或一个无限的成功尾段。

## 10. “all regions”与“only one healthy”的精确范围

冻结模型对**每个区域**都规定 Byzantine weight 至多 `F_i < W_i/2`、correct-to-correct delay 不超过该区域 `Delta_i`，并在 progress 条件中要求所需 state/authority 最终可用。它并不是“任意多整个区域可永久 crash、只剩一个 correct region”的故障模型。把其它区域的 correct 多数永久停机，或把永久 state 不可恢复说成普通低健康状态，不能自动成为模型内攻击。

因此有三个不同目标：

1. **只有一个 eligible 区域，而所有区域仍满足冻结 regional 模型。** 可以检查 selector 的同区域 renewal；本政策确保健康租期不会把这唯一候选永久 OFF。但既有“waiting boundary eventually has another eligible region”原文并未因此消失，且 AUTO/Ready/receipt cutoff 仍可能阻止增长。不能宣称完整 one-eligible theorem 已完成。
2. **每个长期可服务、受健康管理的区域都得到 accepted output。** H3 去掉了无限误判 OFF 这一种障碍。还需要跨区域 service fairness，例如经独立审查的 service-debt selector，以及防止它的 correct output 永远输给截止时间。只看 global G 增长不够。
3. **任意其它区域可永久失效，只要一个区域仍能服务就持续进度。** 这改变了冻结 regional 模型，不能由当前 theorem 覆盖。一个给所有 ON 区域维护不可免除 service debt 的 selector 在该更宽模型中可能被 dead-but-ON 区域阻塞；一个完全自由的 health OFF oracle 又可能误排除唯一健康区域。没有准确失败检测或额外服务条件，不能同时从现有信息推出这两个保证。

同理，“all regions”若字面包含被合法永久行政 OFF 的区域，与 OFF 的既有选择含义直接冲突。目标必须明确是否限于未永久退出、模型内可服务的区域；不能在证明途中偷换这个量词。

## 11. 接入下一版本前的具体工作

1. 确定有限 `B_F/B_L/B_C`、紧急预留是否包含在总额度内，以及是否接受健康 OFF 到期无条件撤销。
2. 将 durable signing ledger、未决提交的同 id 重试、per-region lease 非重叠和 own-service credit 写成 controller 算法；保持 Protocol 5 不变。
3. 新 MSIG 使用空 attached configuration，健康隔离全部走 independent OFF+ON；为既有附带配置保留原执行路径并定义迁移修复流程。
4. 在 theorem 中只使用 H1–H3 的明确结论；继续单独解决 AUTO、state/AUTH preparation、useful regional fairness 和 request acceptance。
5. 检查以下最小 traces：stale target 不退款；beta 无限变化而 G 固定；迟到依次观察旧 tips；ON 已提交但 OFF 放弃；OFF/ON 同 block crossing；旧 attached OFF 覆盖早处理的 ON；其他区域 empty DATA 不刷新 i 的 lease credit；旧 pending MSIG 的 first crossing 不因上线政策而改变。

验证状态：完成了对冻结模型、当前控制顺序和候选算法的逐项人工推演，以及上述 H1–H3 的局部计数证明；没有修改 LaTeX、没有运行模型检查或可执行协议测试，没有给出整套协议的最终安全性/活性认证。
