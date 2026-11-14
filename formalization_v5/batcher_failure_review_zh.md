# 批次层失效审阅：D 与 G 分歧之后，当前恢复逻辑缺什么

审阅日期：2026-10-26。
配套阅读：[批次层失效流程说明](batcher_failure_pipeline_zh.md)。

**结论：目前的 R-FB 表达了“让 G 服从实际派生结果”，但还没有实现一条完整的批次故障恢复路径。**
Espresso 和区域共识继续正常运行，会让这个问题比 Espresso 停机更复杂：受影响的旧生产后缀可能不断增长，部分内容已经得到 C 的认证或被 L1 接纳，不能统一当作尚未产生内容的空档。

最值得先处理的两处是：

- **相关高度的 Expire 可能已经消费过，回退后不一定还会自动触发 R-OVR。**
- **实际 D tip 虽然安全，却可能太旧，下一块已经无法满足 R-EXP；把 anchor 推到实时附近又需要解释中间已进入 G 的普通内容。**

下面保持当前方案 A，不用“必须让 G 无条件不可撤销”回避问题，也不直接改写协议正文。
明确错误、尚未定义完整的规则、已经接受的可用性代价分别标出。

## 1. 检查的假设与材料

主要来源：

- [Part 0 v0.6](part0_model_goals_zh_v0_6.md)：确认语义、A-CERT、A-RELAY 和目标；
- [Part 1](part1_module_interfaces_zh.md)：`Replaced`、R-FB、R-DEP、R-OVR、C 的认证和 Batch Inbox 接纳；
- [协议修订记录](protocol_revisions_zh.md)：当前恢复方案；
- 本地 mega-espresso 的 [S1 / S2 / S4](../megaeth_specs/specs/block-certification/recovery.md)、[派生](../megaeth_specs/specs/rollup/derivation.md)、[Batch Inbox](../megaeth_specs/specs/block-certification/batch-inbox.md)和 [DA](../megaeth_specs/specs/rollup/data-availability.md)；
- 官方 OP Stack 的派生规范及 Holocene 差异。

核心执行中，L1、Espresso、区域权重和通信假设继续成立，普通结果正常发布；DA 也可以一直正常。
失败的是批次的成功交付：所有 batcher 停摆、primary 拖延、C 不提供新认证，或这些服务恢复得太迟。
C 仍然诚实；“C 失效”不表示允许它签错误内容。

如果全部 batcher 都失效，就不再满足批次活性所需的 A-RELAY。
因此，不能把“没有普通批次进展”本身当作推翻条件性活性定理的攻击。
本次检查的是这种允许的服务失败怎样传到 D、G、session 和恢复；恢复服务后，已经写出的规则是否足够重新工作。

| 编号 | 发现 | 分类 |
| --- | --- | --- |
| B1 | 回退后等待 Expire 自愈，可能在等待一个已经处理完的事件 | 明确错误的进展推断；具体续接缺口 |
| B2 | `last(D)` 可能安全但过旧，R-ANCHOR 的前跳又会触及已有普通内容 | 证明缺口与设计取舍 |
| B3 | R-FB 没有固定读取 D 的公共切面和替换终点 | 规则缺口，可能引入观察者差异 |
| B4 | “截止前接纳过”不足以排除派生替代；一个 ddl(h) 也需精确定义 | 明确错误的充分条件；触发规则不全 |
| B5 | 父历史替换后，旧 Program、FINAL、prepared state 和已消费事件需要一起处理 | 生命周期缺口 |
| B6 | 改写的 G、历史 drain 与迟到 C 证书之间缺少一致的授权边界 | 潜在并发攻击面 |
| B7 | 生产继续增长，恢复 drain 没有固定的结束边界或追赶条件 | 活性证明缺口 |
| B8 | Espresso 正常时，Force 和 STOP/RESUME 不能不加区分地代替批次恢复 | 治理接口与语义缺口 |
| B9 | 解锁不等于可以删除未完成恢复所需的数据 | 保管义务与恢复依赖 |
| B10 | G/D 分歧、D 的安全、普通内容兑现是三个不同结论 | 需要准确陈述的既定取舍 |

## 2. B1：等待过期，可能等不到第二次事件

**位置。** Part 1 §5.2 的一次性事件处理、R-DEP / R-OVR，以及 §6.4 的第 4 步：

> 负责 session 之后的块父块不符，不再合格；其高度过期后由 R-DEP 与 R-OVR 接管。

这一步漏掉了“过期事件可能早已处理”的情况。

### 一个具体的顺序

用示意值 `W_exp=2 小时`，并取一段 L1 历史，使相关批次窗口约在 12 小时后结束。
小时数只用于展示事件先后，不替代真正的 L1 区块截止条件。

| 时刻 | 事件 | 草稿中的效果 |
| --- | --- | --- |
| 10:00 左右 | 普通 106–115 已进入 G，batcher 停止成功交付 | D 仍只有 105；区域可继续生产更远后缀 |
| 12:00 左右 | `Expire(106)` 等事件依次发生 | 这些高度已在 G，R-DEP 不追加；相应事件处理位置前进 |
| 22:00 左右及控制传播之后 | 106 的原生 fallback 已确定，GFR 执行 R-FB | 输出 `Replaced(106)`，要求改用实际 D |
| 此后 | Manager 把 R 重置，旧 Program 的后续 parent 不再相容 | 草稿没有重新调度旧 Expire，也没有直接建立新的恢复 context |

假设按“改用已派生 D”的自然读法，G 暂时只保留到共同位置下的 D tip，例如 106。
下一缺口是 107，但 `Expire(107)` 已经处理过。
以后新产生的 `Expire(k)` 对 k>107 不满足 `k=h(last(G))+1`，也不会把 107 补上。
之后的截止事件若只检查“当前 G 在该高度是否为普通块”，107 不存在时又是 no-op。

**因此，“再等过期就一定重新轮换”这个推断不成立。**
这是遵循现有一次性事件守卫的一种停滞执行；当前文本没有规定额外转移来排除它。
不能因为 Espresso 还在出块，就假定旧的时间事件会重放。

如果作者打算不截短 G，而保留尚未派生的旧后缀，则需要证明这些后缀的 parent 和状态仍合法；这进入 B5，不能直接保留原块体。

**一个相关的文字冲突。**
修订记录要求 R-OVR 在“R-DEP 确实追加了当前 session 区间内的高度”之后触发。
Part 1 的简写只检查 Tail、`h≤H(ν)` 和无 STOP。
若把后者独立执行，早已在 G 中的旧高度 Expire 时，也可能关闭正在服务的后续 session。
不能靠这种无条件关闭来弥补前面的停滞；两处守卫需要有同一个明确含义。

**分类。** Critical Error 是“该步骤自动保证进展”的推断；完整 R-FB 接续仍是 Justification Gap。
这里没有声称任何可能补完的协议都会永久死锁。

## 3. B2：安全的旧 anchor，不一定能通过当前的接纳时限

**位置。** R-EXP、`AnchorHeight` 与当前恢复说明。

恢复不能违背 D 已确定的前缀，但不能据此直接推导“从 `last(D)+1` 出块就恢复了”。

R-EXP 要求首个普通块满足：

$$
\theta_{\rm accept}\le t(h_A+1)+W_{\rm exp}.
$$

若 D 的末端时间戳大约是 10:00，当前 Espresso 时间已到 22:00，而 `W_exp=2 小时`，从这个 anchor 产生的下一块无论区域共识多快，也不能在 12:00 之前被现在的 Espresso 接纳。
Batcher 和 C 此刻全部恢复，也不会改变这条不等式。

当前 R-ANCHOR 试图把起点提高到 `θ-M_a-δ_c` 附近。
**但在 Espresso 停机的例子里，这一段可能尚无普通结果；在本次故障里，它很可能早已被 G 的普通块占满。**

例如 D 到 120，旧 G 已到 200。
121–200 可能包括：

- 已认证但尚未提交的普通批次；
- 已接纳、仍待处理的 channel 内容；
- 尚未超过各自交付期限的普通块；
- 在新 parent 上还能执行的历史交易。

不能仅因为原 parent 的某一块失败，就把这些高度全部等同于“从未有内容的空档”。
也不能取旧 `max(h(G),h_det)`，然后继续使用旧 200 的状态。

**分类：Justification Gap。**
需要同时解释已确定前缀的保留、旧普通内容的处置，以及首个新普通块如何满足时间限制。
本条不要求选方案 B；它指出当前实际 anchor 与时间前跳之间尚未建立合法接续。

## 4. B3：R-FB 使用 D，但没有固定 D 读到哪里

**位置。** `AdvanceGlobal(X, LOG_L1, LOG_E)` 和“G 从 h 起改用 L1 派生链的块”。

这句话缺少两个输入：固定的派生来源位置，以及这次替换的终点。

**观察者差异的例子。**
两个节点看见相同截止事件，甲已经处理相关输入到 D=120，乙只处理到 D=110。
如果两者各自复制“我当前的 D”，它们会在同一个回放位置产生不同长度、不同 Tail 状态的替换。
稍后所有数据齐了，也不能靠约定“下次再补”解释已经发出的 session 分配事件。

这不意味着 GFR 在理论上一定要直接调用 DA 网络。
如果公开 G 历史及 L1 接纳证明已经足以重建输入，也可以证明那条重建路径；但要规定对应的编码、接受切面、channel 完整性、派生状态及缺失时的等待行为。
现在接口里一句“改用 D”没有完成这些工作。

**分类：Justification Gap，可能影响确定性。**
“两本来源账本安全”只能保证来源不冲突，不能自动保证一个依赖未固定执行进度的恢复动作一致。
这一问题在 Espresso 持续前进时更明显：回放和派生可能一直以不同速度工作。

## 5. B4：接纳、及时可消费、实际执行成功不是同一个判定

**位置。** R-FB 只在“截止前没有接纳批次”时替换，其余 no-op；G-GFR-8 却声称处理截止后 G 与 D 在该高度相同。

当前判定没有覆盖全部派生结果：

| 发生的事 | 接纳记录 | 实际派生结果 |
| --- | --- | --- |
| 合法认证的批次提交得太晚 | 可以存在并推进 cursor | 仍可能失去该高度 |
| 更早高度被替代，后续批次的 origin 已不适用 | 后续批次可能已接纳 | 按 Δ5 替代 |
| 更早交易丢失，后续 payload 执行无效 | 后续批次可能已接纳且完整可读 | 按 Δ6 处理 |
| 接纳了必要 payload，但无法取回 | 接纳记录存在 | 等待，尚无后续派生结论 |

上游 [Batch Inbox](../megaeth_specs/specs/block-certification/batch-inbox.md#security-considerations)已明确说明，推进 cursor 不保证派生接受该块。
本地 [Δ5 / Δ6](../megaeth_specs/specs/rollup/derivation.md#same-height-deposit-only-substitution)也不是“没有提交”的规则。

**反例的关键是：** 107 在截止前被接纳，但因 106 被替代而无法在实际 parent 上执行；R-FB 对 107 若一律 no-op，就不能证明 G-GFR-8。

此外，`ddl(h)` 不能只从固定 L2 timestamp 加一个常数就得到。
原生窗口与 L1 origin、实际派生进度和可读完整批次有关；发生 origin 替代后，需要说明后续 deadline 使用哪个确定上下文。

**分类：Critical Error 是把接纳当作充分条件；Justification Gap 是完整的回退触发和 deadline 定义。**
不能把缺 DA 数据也放进同一个“失败就补空”的分支。

## 6. B5：一个 parent 的变化可能跨过很多 session

**位置。** `MGR.ApplyLifecycle(Replaced(h))` 只把 R 重置；Session Program 的 context、锁和签名槽仍固定。

106 属于 A，但 106 改变会影响旧 110 的状态，进而影响从旧 110 开始的 B、从 B 的 terminal DATA 开始的下一 region，乃至已经提前安装但尚未 Enter 的后继。
只关闭当前负责区域，也不一定覆盖全部依赖者。

有三种做法不能直接使用：

1. **只替换 G 的数组，Program 继续跑。** 它的 parent、执行状态和锁还属于旧轨迹。
2. **在原 `(β,r)` 里重置状态。** 该身份已有签名槽，重跑可能违背单次 Enter 和不可复活规则；诚实节点应拒绝冲突 handoff，而不是清空记录。
3. **普通交易列表没变，就保留旧 FINAL。** FINAL 绑定完整区块及 context，不只是一个交易列表；新 parent 或 state root 改变后不能冒用旧证明。

**分类：Justification Gap，错误的证书复用推断可以直接排除。**
正式恢复需要交代关闭集合、保留的证据、作废的 prepared states、正在执行的回调，以及后续实例身份。
这正是从 OP 的单一执行分支调整到多 region / 多 session 时，不能省略的一层协议。

## 7. B6：C 认证哪个版本的 G，决定历史积压能否安全排空

**位置。** `CertifyBatch` 核对某个 gpos 上的 G；`BuildPayload` 使用 G 的内容；R-FB 允许改写 G。

上游 mega-espresso 的恢复以不变的 selected records 为依据，逐个提交旧内容，让 D 决定保留或替代。
轮换草稿的选取依据却是可能被改写的 G。
如果 G 的旧普通 106 已变成 DEP，恢复中的 batcher 是提交旧 106 的 record，还是提交新 G 的空内容？两条路径的 cursor、origin 和已有 frames 可能不同。
当前“某个 gpos”尚未规定这种选择的授权范围。

### 一个必须排除的并发轨迹

1. 回退前，诚实 C 已对一个旧普通后缀签发证书，提交者暂不发送。
2. GFR 处理恢复，Manager 准备从实际 D 建立新 context。
3. 如果恢复没有使该旧请求的 prev-cursor 或接纳权限失效，旧证书仍可能先于新批次被提交。
4. 对尚未过窗口的高度，旧普通内容仍可能通过原生检查；新 Program 却已经按另一份后续输入建立状态。

这不是 C 作恶，也不是伪造签名；证书签发时完全可能合法。
如果 `prev` 已因真实 cursor 更新而不匹配，Batch Inbox 会拒绝旧请求，这条攻击就被挡住。
问题是当前恢复没有为所有路径建立这个前提，也没有规定同一历史 gpos 的证书何时失去后续接纳权限。

**分类：Justification Gap / 潜在并发攻击面。**
我没有把它当成所有恢复执行必然出现的安全破坏。
需要明确历史 drain、旧认证、当前生产许可和 L1 接纳之间的边界；单纯“C 始终诚实”不足以替代它。

## 8. B7：Espresso 不停，旧生产后缀可能一直延长

**位置。** 当前正常发布路径、R-FB 的延后处理，以及恢复时间的假设。

Espresso 停机时，公开普通后缀会停止增长。
批次层故障时则不同：第一处分歧可能已经出现，Publisher 仍在发布从旧状态产生的更多记录。

上游 Recovery S4 的 drain 需要识别发现分歧时已经存在的记录范围，并承认发现越晚，受影响范围可能越长。
轮换版本还需要让所有观察者共同确定哪些记录属于旧责任、什么时候换到新责任；不能让每个 batcher 按自己的“发现时刻”选末端。

另一个取舍是吞吐：假设积压产生速度为 λ、恢复后的成功交付速度为 μ。
若 μ 不大于 λ，不能仅凭服务重新在线就断言积压会清零。
即使 μ 大于 λ，恢复时长也依赖积压规模和接手条件。
这里是对恢复服务条件的检查，不是在已明确保证有界 backlog 的模型之外构造攻击。

**分类：Justification Gap。**
需要一个共同边界和足够强的服务条件；“C 返回了”“alternative 已注册”都不是完成历史接续的证据。

## 9. B8：Force 只能换生产者，不能代替批次故障恢复

**位置。** Force 从 G 接棒；override 只允许由 SubmitResume 调用。

若 G 的原 parent 已与 D 不同，M 发 Force 再从这个未纠正的 G tip 开新 session，仍然会继承错误的恢复基线。
换 leader、换 region、提高 L1 floor 都不会替代派生与状态重建。

STOP / RESUME 也不能机械复用 Espresso 停机场景。
本次可能存在很长的已全局接受普通后缀；若 M 直接用 override 把尚未到期的部分填空，它改变的是这些内容的保留机会，不只是未来生产速度。
当前截断和接纳守卫可能阻止一些这样的请求，但必须验证其完整条件，不能在解释中跳过守卫再宣称 M 只影响活性。

还有一个接口适配问题：Batch Inbox 的 admin 改为 Authority 后，原有 batcher 注册、替换 primary、修复不可继续的 cursor 等管理动作，需要有实际可调用路径。
“只允许 RESUME 做 override”并不自动实现其他管理入口。

**分类：Justification Gap。**
M 可以协调故障处理，但不能把未知 D 状态变成已验证 anchor，也不能靠 STOP 治好离线的 C。

## 10. B9：P2 解锁后，历史恢复输入仍可能有用

**位置。** 时间解锁、PublicStore 和历史结果交付义务。

长期没有成功批次时，一些旧 session 已过 R-EXP，质押也可能按 P2 到达释放时间。
但恢复 batcher 仍可能需要它们的 record、完整证明、已接纳 frames 和状态重建材料。
如果“已过生产截止”被实现成删除这些数据，后续批次恢复可能只剩下 hash，无法继续。

**分类：保管契约缺口，不是推翻 P2 的时间推导。**
若已有独立且满足保留要求的归档和 DA 路径，这个问题可以被相应假设覆盖；必须把责任交接写清楚。
区域本身没有违规，不能因为外部 batcher 不工作，就在未定义 offense 的情况下 slash 区域成员。

## 11. B10：需要分别评价三个保证

| 问题 | 本次故障下能否同时成立 |
| --- | --- |
| Espresso 上已最终化的区域记录是否一致 | 可以，批次失败不使 Espresso 历史分叉 |
| 诚实派生节点是否得到相同 D | 可以，遵守相同 L1 输入和派生规则即可；缺数据时等待 |
| G 中每个普通块是否按原内容和原状态落到 D | 不保证，这是方案 A 的条件性部分 |

所以，G 与 D 出现分歧，不自动等于 L2 派生安全失效。
但反过来，D 仍安全，也不能证明新的 sequencer anchor、旧签名权限和 G 的恢复转移已经正确。

“普通内容保持”“完整区块保持”“执行结果保持”还要分开。
某个后续批次保住同样的交易列表，也可能因为前面的状态改变而得到不同结果。
普通交易被移到后续高度重新提交，同样不是原高度内容恢复。

OP 的 unsafe / safe 对应提供了一个处理这种分歧的基础，但 G 还承担区域证书和轮换权限，不能只把名字换成 unsafe 就获得完整恢复协议。

**分类：已经接受的取舍，需要准确表述。**
这不是要求把批次可用性重新塞进所有 sequencer safety 假设；它要求把不可避免的恢复反馈接口写出来。

## 12. 必须一起检查的并发情况

| 并发情形 | 需要有唯一答案的问题 | 关联发现 |
| --- | --- | --- |
| cutoff 所在 L1 块恰有最后一个 frame | 按完整 L1 位置和 channel 条件，究竟已可消费还是仍缺输入 | B4 |
| 同时发生两个高度的 fallback | 一次更早替换是否已经覆盖后面的替换；不能反复套旧状态 | B3、B5 |
| A 的历史被替换，B 正好凭旧 terminal receipt Enter | 新 handoff 是否仍获授权，关闭覆盖哪些后继 | B5 |
| Force 与 Replaced 接近同一回放位置 | Force 用替换前还是替换后的 G；旧 policy 的责任何时结束 | B3、B8 |
| C 签发旧批次后发生恢复，提交尚未执行 | prev-cursor、gpos 和恢复权限怎样使接纳结论唯一 | B6 |
| alternative 重打包，primary 的旧 channel 又恢复 | 接手、超时、重复提交如何与真实 L1 顺序一致 | B4、B6、B7 |
| G 回退到已发生过 Expire 的高度 | 如何避免依赖已消费事件重新出现 | B1 |
| 派生节点处理进度不同 | 同一个恢复事件绑定哪个固定 cut，而不是谁先算到哪里 | B3 |
| 解锁到期时历史批次仍未处理完 | escrow 释放与数据保管责任如何区分 | B9 |

## 13. 目前可以保留的方向，以及尚不能下的结论

可以保留的方向：先尝试备用 batcher 和有效认证路径；无法及时交付时沿用原生 fallback；恢复时尊重实际派生前缀；已经决定的 deposit-only 高度不再补回旧普通内容。
这些与方案 A 一致。

尚不能下的结论：

- “batcher 一恢复，D 自然就会追上原 G”；
- “执行 Replaced 后，再等过期就一定能换区”；
- “实际 D tip 就是马上可用的新实时 anchor”；
- “把旧 G 后缀全部填成 DEP 不会影响已有内容承诺”；
- “切换到诚实 C 或改变 inbox cursor 就完成了状态恢复”。

这次最需要写清楚的是一项横跨模块的恢复契约：**固定旧责任的结束位置，处理到一个共同的派生切面，作废依赖旧 parent 的生产权限，并证明新的起点能满足接纳时间和批次接续条件。**
当前 R-FB、Replaced 和 R-ANCHOR 分别表达了其中一部分，但尚未共同完成这件事。

## 14. 来源与验证范围

本次使用人工执行序列、条件分支和时间不等式检查，没有把协议补完后再宣布安全，也没有运行完整状态空间模型。
关键来源位置如下：

| 来源 | 对应内容 |
| --- | --- |
| `part1_module_interfaces_zh.md:325–347` | 全局处理位置与事件键 |
| `part1_module_interfaces_zh.md:924–929` | Replaced 的 Manager 效果 |
| `part1_module_interfaces_zh.md:1216–1243` | C 认证的 gpos、payload 和 cursor |
| `part1_module_interfaces_zh.md:1355–1372` | R-FB、Expire、R-OVR、新 session |
| `part1_module_interfaces_zh.md:1496–1505` | 现有批次故障恢复链条 |
| `megaeth_specs/specs/block-certification/recovery.md` | S1、S2、S4；恢复 drain 与未定问题 |
| `megaeth_specs/specs/rollup/derivation.md` | parent / origin、Δ5、Δ6、CursorSet |
| `megaeth_specs/specs/rollup/data-availability.md` | 已接纳数据不可得时等待 |

官方 OP 参考为 [派生规范](https://specs.optimism.io/protocol/derivation.html)和 [Holocene 派生变化](https://specs.optimism.io/protocol/holocene/derivation.html)。
本文采用的 MegaETH 差异以本地 specs 为准，不把上游全部 channel 丢弃规则原样套入。

审阅基线 SHA-256：

```text
part0_model_goals_zh_v0_6.md
262f08614bbc07b7b71fdec0a738a6e7383af09f22bd20f0407098ebb75e0560

part1_module_interfaces_zh.md
24bb17a6ac073437a6bef38eceb691a46dae8b617453fc40a3c05cfaa11e5521

protocol_revisions_zh.md
be844b3e2cbb885593f5fdaba15f3f73b4127fdc85831921b7c77b4b0ff16af2
```

本文只记录审阅结果，没有直接修改上述协议正文。
