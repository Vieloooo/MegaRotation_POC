# 固定 frontier 后的时序义务：首个 Ready、完整响应与累计窗口

日期：2026-10-18。独立设计分析；不修改协议，不作整体正确性结论。

记 `V` 为当前 `sync_hotstuff_rr_part2_rotation_v3_4_1.tex`，`M` 为冻结的 `revisions/v3_1_frozen/model_and_assumptions.tex`，`I` 为 `sync_hotstuff_rr_part1_core.tex`，`R` 为 `sections/rotation_v3_4_ready_bundle.tex`。下述 S1–S3 是 `strict_cutoff_next_design.md` 中的候选修改，尚非当前协议事实。

## 1. 本分析得到的结论

固定 `G`、保留零输出 Force 的 `r`、缓存有限 authority 前缀、保留真正可执行的 native `Prep[G,...]`，能够把这些有限外部依赖的无界恢复等待全部移到一个有限 warm-up 之前。**在通常的原子本地状态机语义下，明确的立即区域输入 relay 和有序 P6 handler 可以补上 source-visible → replay → Manager → 首个 Ready 的接口；无需另加一个任意 native restoration 或 catch-up 的常数界。** 当前 ReadyBundle 自己只从首个实际 Enter 才给 `Δ` 结论，故新接口应写出来，不能只引用 ReadyBundle。

这个构造的关键不是给任意后台工作重新命名。输入必须是完整新 Espresso 输入及其 chain-validation dependencies；已在线跟随的 replica 持久保留此前 replay state；handler 在收到材料的同一逻辑事件中推进可用连续前缀、确定 handoff、附接已经可执行的 anchor state 并 Enter。外部恢复、数据库导入和离线历史 catch-up 均不在该步骤内。

一个需要明确的现存文字边界是：完整 namespace 中缺自己必需 evidence 的 EXPORT 是否按不完整 record 拒绝，还是允许其 dangling reference 暂停整个 public replay。V1403–1407、1457–1462 支持前者；V1452–1455 的“missing history/authority pauses relevant verification”必须只用于真正缺失的必要 source/boundary context，不能成为任意 malformed candidate 的阻塞权。本文给出这一完整性规则下的构造性接口；如果拒绝该规则，则留下的是具体的外部依赖阻塞问题，不是任意 correct-scheduler-delay 反例。

在该接口与另行检查的 finite regional-response 引理成立后，有明确有限 `L`。删除 AUTO 自造的 unassigned cooldown 空档后，累计窗口证明成立；它只用 Espresso 的 upper lead，不需要 lower lag、出块速率界或空块生成界。

## 2. 哪些有限等待可以被合法移到一个有限时间之前

考虑用于反证的停滞尾部：从某时刻起 `G=G*`，S1 使零输出重试的 `r=r*` 固定，且不再有匹配的 hard Force。`AUTH[r*,0]` 已固定全部地区的配置、`Δ_i`、quota、窗口与 cooldown；不能重绑 authority（M30–44）。

这里需要的有限集合是**这个已安装 authority 所列的有限区域快照**。不需要、也不能假设所有未来 `r` 的成员并集有限。设该集合为 `I*`。对于每个可能实际被选中的 `i∈I*`，若：

1. 持久任务以 `(anchor producing pair, anchor hash, cfg/state-format, i)` 为键，对 `G*` 的 native 状态恢复最终完成，完成物不会因 `β` 改变而删除；
2. 独立的区域进展论证给出一个有限可用 generation 前缀 `0,...,g_i*`，其所有 immutable authorities 最终到齐并被缓存；
3. anchor 的历史验真结果和已验证 public replay checkpoint 被保留；

则这些**有限多个实际完成时间**的最大值 `T*` 有限。`T*` 可以依执行而变，也不必能由参数预先计算。这不是给 native restoration 或 future authority 新增统一延迟界。

此处不能把“fairness 给未来某个 honest 名字”代替第 2 项。I185–221 的永久排除、有限固定 Byzantine 身份及 generation/term 公平性，与 M297–300 的实际进入措辞，必须由区域分析给出准确连接。本文只在该连接已建立时用其有限界 `B_i`，不自行宣布已完成。

此外，保留 `Prep` 不能只意味着保留序列化 snapshot。一个失败 session 若会改变其 application state，新 session 必须还能直接使用同一 anchor 的已准备执行对象，例如模型中的不可变 anchor state 和独立 continuation handle。若每次新 `β` 仍需从 snapshot 导入数据库、复制全部状态、启动 native runtime，相关残余时间 `u_n` 不因“anchor 没变”自动有统一上界；R99–146 已明确保留这个限制。允许抽象程序直接操作已经验证的对象，与声称任意数据库导入很快，是两个不同结论。

## 3. 删除 post-opening header 的必要性与准确范围

当前 V1973–1977 与 R38–43 要求 Ready 已获得一个严格晚于 `q_open` 的 Espresso header。冻结模型没有下一个空 Espresso block 的生成时间界。即使 anchor state、authority、所有现存历史已经在本地，该条件仍可使首个 Ready 任意晚，下一块也可能先执行 AUTO。

S2 应同步删除这两个位置的 post-opening-header 条件，并保留：

- 首个 DATA 引用至少 opening header，ancestry 上引用不减，且引用严格早于包含该 DATA 的 EXPORT block；
- regular successor 的搜索下界为 `max(height(xi(T)), q_open+1)`；
- Force 的搜索下界仍为其 triggering height，严格 pre-payload cutoff 不变。

这样完整 terminal result 可以先形成并交付 Publish Layer。它的 Espresso publication 自己提供后续 block，无需假设外部生成空块的固定间隔。注意该论证只覆盖**先形成完整结果之后**的 regular-selector 等待，不会自动填补首次 source replay 或 native entry 延迟。

## 4. 当前 source-visible → 首个 Ready 的实际缺口

冻结保证逐层为：

| 步骤 | 当前可引用的界/限制 |
|---|---|
| 新 Espresso block finalized → 完整相关 namespace 与 Espresso evidence 可用可验 | 对持续在线且跟上 history 的参与者，至多 `T_E`；M236–250 |
| control 证据 | 合法实际提交的 operation 满足 `T_L1`，advance announcement 使适用控制在 crossing finalization 前可用；M208–215、272–283 |
| source authentication → chain acceptance replay | M251–265 明确没有把 regional/application validation 等同于 Espresso evidence validation；V2307–2335 指定过程，但未给新输入进入该过程的时限 |
| replay → fixed handoff → 首个 Ready | V1674–1718 指定串行事件逻辑；没有给源事件排队或整个工作队列一个处理时限 |
| 首个 Enter → 其余正确成员 Enter | 仅在完整 ReadyBundle handler 符合 `Δ_i` 的解释下为 `Δ_i`；R107–146、V2022–2031 |

此表揭示的是要写清的接口，而不是可随意添加的本地 scheduling delay。按本协议通常的语义，enabled 的纯逻辑 state-machine 步骤原子执行；在明确的立即输入 handler 之后，再让正确 replica 任意推迟 P6/Manager，不是一个合规反例。真正可能停顿的是未获得的外部依赖、尚未完成的 native 操作或显式异步队列。第 5 节通过把消息、发送起点、必要依赖和 handler 终点写清来排除最后一类。

`online and keeps up with the history` 字面给的是 source history 的接收验证资格，不宜直接改写成一个新 chain-replay SLA。构造应是：已在线跟随者原有 cursor 被保存，而新 source event 必须立即推进 P6；由 handler 归纳维持 replay cursor，不另行假设每次 opening 都已 replay 完。

M303–309 还含成功 sustained service interval 的条件。本分析不以这句话代替 `L` 的推导，也不宣称一个违反它的永久 stall 是满足全文所有 progress 条件的反例。

## 5. 可审查的 `SOURCESTEP` 契约，以及它没有解决什么

下面是可以考虑的协议接口，不是当前文档已有的界。

### 5.1 消息与发送起点

`SOURCESTEP(chain, region-snapshot-id, q, D_q)` 是经认证的区域消息。对指定的一个已验证 predecessor replay checkpoint `C_{q-1}`，`D_q` 包含：

1. 完整 `J_q`、与已知 Espresso trust anchor/前缀相连的 finality evidence、完整相关 namespace 或 authenticated absence；
2. 完整适用 L1 control prefix 的增量及其 completeness/finality evidence，所有本步实际依赖的历史 authority 与 regional/application evidence；已认证缓存对象用无歧义 commitment 索引；
3. 本步 predecessor cursor 的标识；chain replay state 包含 `β`、`G` provenance、duty/window/miss state、pending control state、eligibility 与 boundary 状态，并由接收者此前的 ordered replay 自行维护。包中若提供 checkpoint，也只有已授权或已自行验证的 checkpoint 才可作为起点；不能以发送者签名代替 replay correctness；
4. 若本步会打开本地新目标，能定位该固定 `G*` 的已完成 executable native object；缺少可执行对象则不宣称本包能触发 Ready。

每个正确区域成员一旦获得上述**完整 source 输入**，就在该 source-availability 事件中向本快照的成员发送，不先等本地 P6 跑完才转发。发送者不必先知道本步选中了哪个 region；固定 `r*` 的各候选区域成员可向自己的已安装快照转发。无须新增跨区域传输保证、Espresso READY 交易或 READY certificate。

“立即”在此必须是具体发送动作及其时刻，而不是排一个未界定时延的后台任务。发送前若仍缺 authority、应用证明或 control 完整性，则该包尚不完整，`T_E` 不能单独给它的发送时间。

### 5.2 完整单步 handler

handler 首先按 height 缓存完整输入，按顺序在同一逻辑事件内 drain 现在已经具备全部依赖的连续 source prefix；若 `q` 先到，其缺失 predecessor 到达的事件必须立即继续该 drain，不等待另一个未界定的 tick。对每个现在可处理的 `q`：

1. 认证和验证输入；
2. 按 P6/P5 顺序执行该 block 的 prelude、eligibility snapshot 与 EXPORT 处理；
3. 用同一验证结果驱动 Manager，安装新 fixed handoff、记录 Closed gate 与持久 outgoing obligations；
4. 若 S2 下所有 Ready 条件已经满足，直接附接已准备 executable state，原子测试 entry flag 并执行 Enter；有更新 closure 则拒绝该 entry。

这些动作必须真的是完整消息的验证/状态转移操作，并服从冻结 `Δ_i` 所覆盖的处理范围（M167–179）。纯逻辑 cursor 推进、字典查询、Force/selector 决策与 entry flag 转移遵循既有原子状态机语义；密码学/应用/源验证使用完整 message-processing 界。这里没有额外 native import；不允许先返回、再留一个无界 Ready queue 或 installation job。

在固定 `G*` 的零输出路径上，S3 不应执行 `M_n` 次实际后台任务来“经过”零计数窗口。当前 timestamp 跨过的零计数 base-window 数量可由 `floor((theta-window_start)/W)` 直接结算，到 `M_n` 则执行一次 AUTO；新 session 窗口从本块 realized timestamp 开始，不能在同块继续触发另一次 AUTO。这是与逐窗口规则等价的状态机实现，不是假设更快时钟。

关闭历史程序的慢清理、扫描所有 archive 和后台重建，不能是新 entry 的前置依赖。需要的即时状态是可查的 Closed gate 与持久未完成义务。V1698–1717 当前对每个已知 snapshot 执行 closure relay 的循环也应在这一接口审查中检查：不可不加论证地把随历史增长的扫描藏进有界步骤。

### 5.3 predecessor cursor 的维持与 `T_E+Δ_i`

这里只考虑从已验证 checkpoint 持续在线跟随且已完成有限 warm-up 的相关区域成员。令 `a=t_E(q)`。对所有 `q'≤q`，都有 `t_E(q')≤a`，故其完整 source evidence 最迟在 `a+T_E` 对这些成员可用；不需要知道新 block 的生成速率。每个对应区域 relay 已在其 source-availability 事件立即发送，不等待它在本地完成 chain replay。

在完整依赖契约下，这些 correct-to-correct 输入的 delivery/validation processing 均在各自 send 后 `Δ_i` 内完成，因而不晚于共同截止点 `a+T_E+Δ_i`。ordered handler 用保留的前置 replay state drain 已就绪输入，原子的状态机推进不再为每个 height 增加一轮网络等待。于是至该截止点，输入 `q` 的 P6 转移及其 Manager/Ready 动作已经执行，或更晚的已验证 closure 已使该 entry 无效。这里取完成时刻的**最大值**，不是把所有历史 message delays 串联成 `qΔ_i`。

这一论证依赖现有 network premise 确实包括完整消息的 validation processing，不能把该 premise 改读成只有字节传输；也不把持续跟随者换成带任意旧 backlog 的 returning replica。对后者，起始 checkpoint、旧依赖和 native state 尚未恢复，M231–234 仍不提供常数 catch-up 界。对持续跟随者，保持 current replay cursor 是上述 handler 的归纳不变量，不是新增独立的实时服务公理。

### 5.4 固定 frontier 下的实际依赖审计

| 输入/处理 | 为什么不会每次引入新的无界外部等待 |
|---|---|
| Espresso headers、完整 namespace、finality/linkage evidence | 来源为冻结 `T_E` 的完整 source-visibility 保证；所有 predecessor 不晚于当前块 finalized |
| 适用 controls | advance announcement 在 crossing 前提供；这里进一步按任务范围取 health-control 有限尾部，有限历史已缓存 |
| opening/Force 需要的 `AUTH[r*,0]`、native anchor 与 provenance | `r*`、`G*` 固定；有限 warm-up 完成后缓存命中，窗口增长不生成新 native 锚点 |
| 当前 session 所需的有限 recovery authority prefix | 由独立区域证明给出有限前缀并提前缓存；不能把未知更深 recovery 无条件算入 `B_i` |
| 已 closed beta 的 raw EXPORT | 检查 domain/base/version 后即拒绝 append；其可保留的历史证据另行处理，不能阻塞当前 replay |
| 未来高度、错误 base、错误 session 或 Force 同块的新 beta DATA | 由 V1427–1440、1457–1460 的结构/当前状态检查拒绝；不先为无权 append 的候选获取 arbitrary future state |
| 当前 beta、当前 base 的完整候选 EXPORT | V1403–1407 要求包含 validation evidence。依赖完整时，有限 validation predicate 由现有 complete-message processing 处理；若全部通过则 `G` 增长，已结束停滞反设；若失败则不改变状态 |
| 完整 namespace 内只有 dangling reference、却未提供应由 record 携带的必要 evidence | 作为不完整/不合 EXPORT 结构的候选拒绝，不向外部无限 fetch；这不同于 namespace 本身未完整到达 |
| 原子状态、窗口和 manager 更新 | 保留 checkpoint，直接更新本步所需状态；不要求回放从 genesis 开始，也不将 archive 扫描/native import 放入临界路径 |

最后一项 EXPORT 完整性语义要在候选中明确。V1452–1455 对正常 deferred successor selection 的必要 authority/history 允许暂停，是合理的；但若它也要求为任意恶意 candidate 的缺失证据暂停整个 P6，则本构造不能宣称所有本步依赖已经齐全。例如一份当前 base 的 candidate 可以引用根本未携带、也不在 finite cache 中的 future authority，从而 source bytes 已完整而验证仍等外部材料。应将 record-validity 的 completeness gate 与真正的 source-availability gate 区分。明确该 gate 是输入/验证规则的细化，不需要给外部材料新增统一获取时限。

因此在完整 record、立即 ordered handler 和已完成 native Prep 的抽象消息解释下，`source-finalization → first Ready ≤ T_E+Δ_i` 可以推出。若具体 adapter 做不到这些操作属于既有完整验证处理，则不能以本抽象证明声称其性能已验证；但这也不构成冻结抽象模型内的无界 scheduler 反例。

## 6. 一旦接口义务满足，可得到的明确 `L`

本节是条件引理，准确展示剩余证明应提供什么。设 `a_n=t_E(q_n)` 是第 `n` 次立即 assigned retry 的 opening block 首次 finalization 实时，`θ_n=θ(q_n)`。

假设充分晚以后，对每个实际选中 `i`：

- 完整 `SOURCESTEP` 最迟在 `a_n+T_E` 发送，且第 5 节的前置状态/完整性条件成立；
- 其 handler 最迟在 `a_n+T_E+Δ_i` 产生首个 Ready/Enter；
- 完整 ReadyBundle 使核心证明所需的正确成员再经至多 `Δ_i` 同步进入；
- 已缓存的有限 authority 前缀和区域规则给出 `B_i<∞`：从上述成员已进入开始，只要当前 branch 未受有效 Force 关闭，就会在 `B_i` 内形成并提供足以让 `G` 增长的完整 next-DATA result；
- 发出的 EXPORT 在固定 `G*` 下携带正确 base、当前 branch、所需 predecessor 与完整证据。若更早已有同样 DATA 被接受，则已达到增长目标；否则该次 inclusion 不会因其它无关 EXPORT 而变为 stale-base。不要在此隐含一个无界 rejected-retry loop。

则可取一个保守的、依固定快照的有限界

\[
L=\max_{i\in I^*}\{2T_E+2\Delta_i+B_i+T_{\rm pub}\}.
\]

这个界从 opening block 的 finalization 算起，涵盖 opening 的 visibility、区域输入与 ReadyBundle、regional result 形成、提交及其 Espresso inclusion。若将 `B_i` 的起点定义为首个 Enter，可省掉重复计入的一个 `Δ_i`；必须在采用公式前统一起点。

结论为：**到 `a_n+L`，要么 `G` 已增长，要么此前或当时已有 effective Force 抢先关闭该 branch。** 在包含结果的同一 block 先发生 Force 也属于后一分支；因此不改变 strict cutoff。仅仅“result 最迟形成”不足以得到这个结论，必须覆盖 publication 与有效 acceptance。

这里 `B_i` 与 `L` 可以依固定执行的有限 usable-authority 前缀而定，不必是运营上可预知的 failover SLA。`T*` 前的 native restoration 与 authority 等待仍无统一界。当前源码尚未写入第 5 节这个完整 source-input contract。将其作为候选协议的明确规则并满足相应 native representation 和 record completeness 后，本节给出构造性的响应引理；它不应被表述为当前 v3_4_1 已经拥有的性质。

## 7. AUTO cooldown 的准确修复与仍需保留的 eligibility 条件

当前 AUTO 无条件对 source 设 `θ+C_auto`（V2232–2236），只有一个 eligible source 时会制造 unassigned interval（V1812–1817）。哪怕 `C_auto>0` 很小，upper lead 也不给等待该 timestamp crossing 的实时上界。没有 export duty 就没有 DATA window；这种空档不能计入 `ΣM_nW`。

适合本证明的规则是：在该 prelude 已应用 due independent STATUS/CLEAR 后，按**指定 target authority 的固定配置**检查 source 之外是否还有 eligible region。

```text
if 存在 eligible alternative:
    对 source 设置 theta + C_auto；按正常 selector 立即选 alternative
else:
    本次 AUTO 不施加新的 source cooldown；
    若 source 此时仍 eligible，则立即续开 source 的 fresh-beta session
```

`C_auto` 在实际使用时仍为 authority 中固定正值；没有把它改成零，也没有改变 Force 的生效 block。若 source 本来就 OFF、没有正权重或已有其它仍有效的 eligibility 障碍，“不新设 cooldown”不会把它变回 eligible。不能清掉先前 cooldown 或绕过 STATUS。

该修复只保证**AUTO 本身不会移除最后一个本来 eligible 的候选**。本任务进一步限定 health-control 为有限尾部，所以可以越过最后一次 STATUS/OFF/CLEAR 生效，等待早期 cooldown 的有限到期位置，然后在固定治理资格上分析。若控制服务让所有 regions 永久 OFF，则当然不存在可服务地区，并违反所需 eligibility 进展范围；不能仅由“无 matching hard Force”隐含排除这种情况。若允许无限 STATUS/OFF 在每个 cutoff 移除唯一 source，M307–309 的 eventual another eligible region 也只保证每次 wait 最终结束，不给这些等待统一时限；那种更宽范围不能直接用本节无 gap 证明。

不需要 eligibility 集合恒定，也不必禁止一切 STATUS。需要的是不会在关键边界消灭所有可用候选。一个在尾部始终 ON、正权重且在此条件 cooldown 规则下可用的地区足以保证非空；它若尚有更早 cooldown，可等 growing timestamps 使这个**单次有限前缀**限制结束。

## 8. 无 unassigned gap 时，upper lead 足够完成摊销证明

现在假设第 6 节的响应引理已证明，且在 `T*` 之后无匹配 hard Force、无 unassigned gap，并反设 `G=G*` 永久不增长。丢弃任意有限前缀后，以 `n=0,1,...` 编号所有重试。因为 duty 一直 assigned，`n+1` 的 opening 就是关闭 `n` 的 AUTO triggering block；没有额外 waiting interval。

固定 `G*` 意味着所有尝试均为 startup 零 accepted-DATA 路径，不会成功切换 running phase。设第 `n` 次尝试容忍 `M_n` 个 startup miss，所选 region 的固定 startup base window 为 `W_n`。有限 installed snapshots 与正参数给出

\[
W_{\min}=\min_{i\in I^*}W_{\rm start}(i)>0.
\]

若多个 base deadlines 被一次跳跃越过，解释器按实际 opening 起点结算，并在达到 `M_n` 个 miss 后执行至多一次 AUTO；因此

\[
\theta_{n+1}-\theta_n\ \ge\ M_nW_n\ \ge\ M_nW_{\min}.\tag{1}
\]

第 6 节又给出：反设没有 `G` 增长，则每个尝试必在 `L` 内被关闭，故

\[
a_{n+1}-a_n\le L,
\qquad a_N\le a_0+NL.\tag{2}
\]

对 (1) 求和，仅使用冻结 upper-lead 条件得到

\[
\theta_0+W_{\min}\sum_{n<N}M_n
\le\theta_N
\le a_N+\epsilon_E
\le a_0+NL+\epsilon_E.\tag{3}
\]

因此只需

\[
\frac1N\sum_{n<N}M_n\longrightarrow\infty
\]

即可矛盾。候选 `M_n=2^{nu_n}` 满足该条件，因为每次 effective AUTO 增加全局 `nu`，且不重置。更一般地，任何使平均容忍预算无界的确定性序列都足够；无需宣称指数是必要的。

式 (3) 允许起点已有任意大的有限 clock lag `a_0-θ_0`。它只成为常数，不能永远支付增长快于 `NL` 的累计 timestamp 预算。另一方面，若每次尝试前有无界 authority/native/replay 等待，或期间有 unassigned gap `g_n`，只能得到 `a_N≤a_0+NL+Σg_n+Σd_n`；这些项可以不断补充 clock lag，此时不再有矛盾。

若某次以后没有 AUTO crossing，则该尝试获得无限长实际时间。已证明的响应引理会在 `L` 内形成、提交并 finalizes 一个 admissible result，其 inclusion 自己提供 Espresso block 并使 `G` 增长。此处也不需要独立的空 block 生成时间界。

## 9. 在线量词与结论边界

`T_E` 只应用于冻结文字指定的持续在线且跟上 source history 的参与者，不能扩展到任意 returning verifier。恢复很久未见过的历史仍在 M231–234 的无统一界范围。区域 `Δ_i` 只应用于固定快照内、满足其通信/处理前提的正确发送者和接收者。完整 receipt 需要的正确成员以及实际 usable leader 必须是区域进展论证中确实可用的那些成员，不能只选一个能读 Espresso 的观察者就宣布整个 region 会产出。

本分析没有假设所有历史/未来配置都有限。只对停滞后固定的 `AUTH[r*,0]` 所安装的有限 snapshots，以及各自已经论证可用的有限 authority 前缀取最大值。

尚待主设计或后续审查完成的具体义务为：

1. 用 frozen schedule/exclusion 条件导出 finite usable authority prefix 和 `B_i`，覆盖实际 entry、所有可能坏 early generations、receipt formation。
2. 把 `SOURCESTEP` 的完整 evidence、立即发送、ordered drain、record completeness gate 和原子 Manager/Enter 写成候选协议规则；沿第 5 节证明维持 predecessor cursor，不为恶意不完整 candidate 增加外部 fetch 阻塞权。
3. 明确 native `Prep` 保存的是可直接用于新的 branch 的执行对象，而非每个 retry 仍需无界 import 的 snapshot。
4. 采用条件 cooldown 时保留 STATUS/OFF 的真实作用；只在每个相关 crossing 仍有 eligible candidate 的范围内使用无 gap 摊销证明。

已完成的是时序分解、满足既有抽象消息界的构造性接口、可检查的条件 `L` 公式与严格 cutoff 下的累计窗口推导。它为下一版规则提供证明路线；当前全文未写入该接口，native adapter 和独立的区域 `B_i` 证明仍需各自验收。
