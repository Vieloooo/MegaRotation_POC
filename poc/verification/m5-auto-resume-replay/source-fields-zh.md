# M5 回放的证据字段

M5 主运行与对照分别按各自的 sealed manifest 和独立 checker 绑定。这里列出投影依赖的原始观察及其关联。既有 M3 的 signer、收据和 Espresso 证据与 M4 的 `ResumeCut`、DA、G/G′/D 证据提供格式参考。[投影合同](contract-v1-zh.md) · [M3 回放证据](../m3-auto-force-replay/README.md) · [M4 回放证据](../m4-resume-replay/source-fields-zh.md)

| 阶段 | 所需字段和关联 |
| --- | --- |
| Espresso 停止与冻结 | 故障事件、模式、起止虚拟 tick，实际 finalized PELG 的 `q`、公开 `θ`、proposal/finalized tick 和读取者可用位置。θ 不动期间保留每个已处理的等键位置。 |
| 公共 WAIT 与 L1/D | 各观察的最后完成 pc、下一公共事件位置/键、所缺 coverage 或对象，读者已验证 L1 与 Espresso 前缀。保留 L1 header timestamp、收录/finalized/可读 tick、实际 DA/D 原因与独立头部。 |
| 本地与全局停止 | replica/member/cid、真实时钟样本、`C_m`、阈值、签名门禁或 terminal 的实际 tick。GFR 中另用 SESSION_END/Expire 的公用位置和记录解释旧职责关闭。 |
| signer 各自验证 | signer 身份、已公布和已验证 L1/Espresso tips、前次 q/θ、pc/Tail、G/D/Inbox heads、是否存在已验证同高不相容或 D 越过的公共 HALT、clock/readiness、缺少输入或在途请求、可签结论和绑定具体提议。 |
| t_eligible 与求解器 | 同一请求的每名 signer 的首次满足 guard 观测及 quorum 的 t_eligible。各 heads、原生推进预算、H_guard、预计收录区间、slot/grid、T_prep、T_drift、ε、M_a、T_ready、δ_G、最小解或无解的边界。 |
| 签名到控制落链 | EIP-712 digest、request/target/seq/policy、成员签名与实际入包、持久准备、实际广播、收据、完整控制和同交易 CursorSet 位置、实际收录 timestamp、finalized 和 reader tick，按真实收录算 act。 |
| 应用及恢复 | act 的实际覆盖键、公用处理 tick、精确历史 ResumeCut 字节/摘要、目标块 native hash、执行后 root 和派生 anchor 连接 ID、旧分支与新 Plan、各 worker Ready/Enter、Regional Final、公开完整位置和首个新 G 接纳。 |

最小映射要保留同一请求的可签观察及实际进入签名包的身份。多个对象来自一次观测时，显示共享的原始 seq/tick 与派生子步骤。若日志只给出被覆盖键和后续已完成事件，等待展示用规则重建并标记出处。G′ 与历史切面相关的重建按 M4 规则标识。

## 已冻结的 M5 计划值

后端合同为 `712c018`，来源预期为 `51330b6`。AR01 与 AR03 计划从虚拟 11.250 s 停止或冻结，到 24.500 s 恢复服务。R0 的本地 stop 阈值为 21 s，公共 `Expire(h2)` 和旧 SessionEnd 均为 24 s。恢复水位后按事件全序先处理有效 Expire。按实际记录核对每个事件是否完成。

正例的预估收录区间为 [26,28] s。以 `b_hi=ℓ14`、`W=4` 求 `E_hi=max(0,b_hi−W+1)=ℓ11`，然后按高度网格求 `h_time=ceil_slot(ts_L1(E_hi))`。输入保留 D head 的实际 origin，`extra_origin=max(0,E_hi−D_head.origin)+1`。取 `H_D=max(h_D,H_Inbox,h_time)+extra_origin`。H_Inbox 包含已知在途推进，公开 HALT 时 `H_G=h_G`，NEXT 时另按合同记录时间推进上界。取三路上界最大作为 H_guard。

T_prep=12 s、T_drift=15 s、ε=1 s 使可行 timestamp 区间为 [40,40] s。若实际 H_guard 小于 30，最小合法目标是 h30，signed origin 取区间下端的 ℓ13。实际 L1 收录决定 act，起点则由对应 Cut 的 D(h30) 决定。计划后继为 R2 [31,40]。以上是运行前计划，正式回放以封存的实际输入和独立报告为准。

## 正式运行的原始字段

| 原始证据 | 映射 |
| --- | --- |
| OrderingFault、OrderingRecovered、OrderingWatermark、finalized PELG | 实际故障信号、q、θ、finalized tick、冻结期间空记录和服务恢复后的新水位。 |
| GfrCoverageWait，GfrReplayRecord | WAIT 的观察包含 `pc`、本次检查的 `through` 和原始 `status/missing`。公共 Expire 由独立完成记录建立。 |
| LocalStopped 与 AckCommitted | pid、cid、逻辑步骤、`local_time_us`、`local_stop_us`、Ack，保留与虚拟 tick 的对应。 |
| GovernanceL1Available、GovernanceEspressoAvailable、GovernancePoll、ResumeInputEvaluation | 各 signer 自己的完成位置和前缀、其 D/Inbox 视图。本次原始观察可能重复当前 tip，判断 q/θ 严格前进时另外引用较早的完整观察。 |
| ResumeEligibility、GovernanceSignature、GovernanceState、GovernanceSubmission | 首个 quorum 的 tick、求解量、观察时合格的成员、实际入包及独立广播/收录记录。 |
| Anvil、CursorSet、ResumeActivated 和恢复结果 | 精确 L1 坐标和真实 act、精确 Cut、原子重新对齐、R2 新职责和首个 G 接纳。G′ 保留认证来源与历史切面重建标志。 |

以上字段已与候选 `d6a36431a42cb6e155e9d192bea37f154f742eaa` 的封存 AR01 与 AR03 和相应独立 PASS 核对。[证据位置与复现命令](README.md)
