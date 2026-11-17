# M5 Espresso 停滞与自动 RESUME 合同 v1

合同标识 `M5-AUTO-RESUME-1`，基线 `e44b6e8e77aeb1aa3895c14109a57ff3894d7252`，工作树 `/home/jy/.codex/worktrees/poc-m5-auto-resume/megaroation_poc`，分支 `codex/poc-m5-auto-resume`。本合同与独立预期在实现前提交冻结。最终验收绑定同一干净候选 SHA，交付主 chat 集成。

## 来源和固定范围

| 来源 | 本合同采用的规则 |
| --- | --- |
| M1–M5 计划 §6、v2 §8.5–8.6 | 已验证分歧或公开 HALT 的恢复理由、服务就绪、收录区间、H_guard、最小合法目标和完整时限 |
| v10 Authority §3.10、§3.12 | 授权、目标单调、同交易 CursorSet、实际收录决定 act、历史 Plan |
| v10 GFR §3.2–3.5、§4.5–4.6、§5.4–5.5 | 水位与完整控制覆盖、全序、Expire/HALT、位置切面、原子对齐、独立 D |
| v10 Manager §3.10、Session Program §3.4–3.12 | 本地 stop、永久 terminal、持久关闭、错时 Ready/Enter 和原生签名守卫 |
| M3/M4 已验合同与原始证据 | 真实 2/3 EIP-712、09/11/12 组件、实际 DA/收据、历史切面和恢复首块接纳 |

固定三区各五个等权 replica，区域 Final 至少四个不同合法 BLS signer。Commonware 2026.10.0、Rust 1.95.0、Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。治理三名 signer、默认门槛两名，继续使用 M3 身份和 M4 RESUME 编码。M5 是公开观察与目标求解扩展，D 继续只消费实际 finalized L1、已验证 DA、配置和 DEP。

一个 Builder 负责本合同必需的运行及治理模块 `poc/prototypes/i1b-runtime/src/m1_smoke/`、新 `poc/scenarios/m5-auto-resume/`，以及必要的 `poc/crates/{viewer-gfr,session-manager,publication-bridge,derivation,batch-services,l1-adapter}/src/` 和对应 tests。复用既有 M3/M4 Python 端口时，只可添加保持原默认行为的参数或模式入口。Lead 负责公共类型、Cargo manifests/locks、治理求解输入格式和证据 schema。Verifier 拥有 `poc/verification/m5-auto-resume/` 及 Lead 明确批准的旧独立 checker 兼容扩展，Tester 拥有其 tester 子目录与专属证据。UI 投影由其他 chat 负责。

## 服务故障和公共重放

两类 Espresso 故障分别建模。

1. 停止 finalized。在故障区间内 finalized q 和水位均保持原前缀。待发布结果可在受控队列中等待，恢复后依日志规则发布。L1 出块、finality、DA、D 和 Clock RPC 按各自事件继续推进。
2. 冻结时间键。故障区间可以继续 finalize 空 heartbeat，q 严格增加，但 θ 固定于故障开始时的最后合法键。普通结果的公开完整交付延迟到恢复后，避免在陈旧冻结键中提前接纳随后生产的结果。保留实际排队结果与故障策略。故障区间标记 E-TS 前提失效，仅按公开顺序和覆盖判断；恢复后的正常时间戳重新满足声明预算。

故障脚本只在调度器可见。signer 看到的输入是各自被实际交付并校验的已公布 finalized head、完整日志及各自 GFR/D 位置，不能读取恢复脚本或未来日志。故障结束事件只改变服务行为，不直接触发治理授权或替换 G。

旧 worker 在自己的 Clock RPC/lease 和协议时钟下继续运行到本地 stop。必须保存停止前签名和已形成结果，并在本地 terminal 持久化后禁止新签名。公开 GFR 只有获得覆盖当前事件键的 Espresso watermark 和完整 L1 控制前缀后才能处理 SessionEnd/Expire，节点本地 stop 不推进公共 pc。恢复时按全序处理积压的事件，迟到旧结果保留其原实际公开位置，并按截止与当前职责排除。

## 每名 signer 的恢复判断

每名 signer 有独立的 L1 reader、Espresso reader、GFR replay、D 输入/派生视图和时钟样本。共享无状态验签或原始对象缓存可以复用，状态位置与取得材料的时间分别记录。D、G、Inbox head 取该 signer 已验证的同一公开观察边界，不能直接采用调度器的全局 head。

每次轮询和发送前复查以下条件。

- 有效时钟样本、lease 和误差预算，已公布前缀完整，必需 DA/切面/历史来源齐备，GFR/D 已处理到本次可用边界。
- 至少两次连续完整观察显示 finalized q 和 θ 都前进，当前 watermark 新鲜度满足正常 E-TS/reader 预算。仅 q 前进不满足恢复 live 条件。没有已知等待中的必要服务输入。
- 恢复理由为 `HASH_DIVERGENCE` 或 `HALTED_D_ADVANCED`。前者必须提供某个共同高度不相等的实际 native hash。后者必须有已公开处理的 HALT 和独立 D 已越过被冻结 G 前缀的证据。主场景的 G/D 共同前缀相等，以后者作为理由。
- 仅有 D 落后、D 高于仍处于 NEXT 的兼容 G、普通下载落后或者服务刚恢复，均不构成恢复理由。
- Batcher、Certifier、DA、L1 以及预计获选地区的状态准备和公开读取具备本次预算。服务就绪通过正常可观察能力/探针建立，读者无法证明时记录 `RECOVERY_INPUTS_NOT_READY`。
- 无未决控制，已有请求和签名不被重复轮询叠加。至少 k 名 signer 对相同 payload 的恢复理由、heads、policy、区间及预算独立验证后才可广播。

每次观察保存 signer id、sample/lease、已公布与已验证 tips、GFR pc、Tail/branch/cid、G/D/Inbox heads、共同高度/hash、服务状态、前后 q/θ、拒签原因和本次求解输入。reason 及算出的目标都能从这些资料独立复算。

## H_guard 和最小目标求解

候选目标只使用发送时可验证资料和预先冻结的推进预算。输入含本地观察时刻、三个 heads、已处理 finalized L1 tip、当前职责、已知排队的批次/控制、预计收录 timestamp 区间 `[t_L_min,t_L_max]`、T_prep、T_drift、ε 与 L2 时间映射。区间覆盖签名/广播延迟和所声明的 L1 排队。低于实际安全裕量的点预测不得冒充区间。

分别求到 t_L_max 的 `G_max`、`D_max`、`Inbox_max`，然后 `H_guard=max(G_max,D_max,Inbox_max)`。每个分量记录来源和最大增量。公开 HALT 的 G 保持原 head。NEXT 的 G 按当前窗口和允许接棒次数/区间的上界估算。D 必须已排空本次已验证前缀中的可推进工作，缺接受数据或待执行历史不能只按 wall 时间加一个速率。未知在途批次或控制导致上界无法确定时，先 WAIT。

本主 profile 在候选形成前只有 h1 批次且没有其他在途批次/控制。D 已处理完最新公布的 finalized L1 前缀，来源只有实际原生回退。采用以下固定保守包络。令 b_hi 为 t_L_max 对应的最晚包含 L1 高度，W 为原生窗口，`E_hi=max(0,b_hi−W+1)`，`h_time=ceil_slot(ts_L1(E_hi))`，`extra_origin=max(0,E_hi−D_head.origin)+1`。`H_D=max(h_D,H_Inbox,h_time)+extra_origin`，其中最后一项覆盖尚可前移的 origin 和首次输出保留。`H_Inbox` 为实际 cursor 加上明确已知待收录批次在区间内的最大推进，主场景无此类额外输入时为 cursor。含未知在途控制、未认证队列或未排空历史时返回 WAIT。公开 HALT 时 `H_G=h_G`，仍处于 NEXT 时 `H_G=max(h_G,floor_slot(t_L_max+δ_G))` 并核对无超越此时间映射的合格生产。`H_guard=max(H_G,H_D,H_Inbox)`。所有 slot 操作使用配置的 genesis height/time 及周期，h_time 的 ceiling 向上取整且不小于 genesis。来源规则与边界向量必须证明上述包络覆盖实际可能推进，运行记录实际 heads 在收录时不超过声明上界。

令 `ts*(h)=t0+(h−h0)*b_L2`。使用 checked 整数运算计算所有满足下列条件的最小整数 h。

`h>H_guard`，且 `t_L_max+T_prep <= ts*(h) <= t_L_min+T_drift−ε`。

下界 ceiling、上界 floor、h>H_guard 的严格性分别测试。上界小于下界、算术溢出、区间反向或参数无效均不得产生签名。检查 `T_prep>=M_a` 与 `M_a+T_ready+δ_G<=T_prep+b_L2`。请求 signed_origin 固定采用预计收录区间下端对应的 L1 block 高度，主场景为 b13，并独立核对实际收录时的合约限制。该请求字段不代替派生规则产生的实际恢复 origin。目标和 origin 不能由最终实际收录反推后改签。

`t_eligible` 定义为首次已有 k 名 signer 的实际输入和时钟满足恢复、就绪、同 payload 和存在解的时刻。记录材料到齐、轮询、签名开始/完成及广播各时间，要求 `t_submit−t_eligible <= T_poll+T_sign+T_broadcast`。t_eligible 可由原始逐 signer 输入重放重算，不能以最终广播时刻代替。签署后若原交易结果未知，沿用 M3 已验证的持久请求/原交易对账流程。

## 正例时序与限界

默认 seed42，15 个 OS worker，时钟偏差分布 ±1000µs，Clock RPC 单向250µs，区域链路4ms，显式工作5ms，Enter 错时3ms。M5 使用 M4 的 chain31337、L2 genesis timestamp10s/slot1s、L1 slot2s/finality1.5s、原生窗口4个L1 blocks、真实单一驱动与 DA。M_a10s、T_grace3s，M5 特定 W_exp12s。T_prep12s、T_drift15s、ε1s、T_ready≤0.5s。Espresso 正常 heartbeat250ms/finalize1ms，signer轮询100ms、签名10ms、广播10ms，reader交付0/10/20ms。

主故障开始于11.250s，h1已从真实原生Final进入G。h1对应普通交易批次仍在L1 h6/t12收录、13.5s finalized后由D独立执行并与G h1相等。后续区域结果等待公共交付，D按原生窗口推进。R0窗口的本地stop为21s，最晚真实停止界21.001s；公共SessionEnd为24s，下一缺失h2的Expire也为24s。冻结或停止的水位无法覆盖它们，GFR原位等待。恢复后的同键排序使Expire先于SessionEnd，旧分支HALT，旧普通候选不会获得新的公开职责。

服务在24.500s恢复正常公开时间键。必须从恢复后的实际连续q/θ观测建立live，再进行每名signer判断。预期签署发生在26s之前，计划L1收录区间为[26s,28s]。该区间与T_prep12s、T_drift15s、ε1s给出唯一时间键40s，即目标h30；只有实际H_guard<30时才能选它。运行时必须从记录中的输入求出该数。默认实际收录26s，act36s，恢复新session=(β+1,0)，policy order[2,0,1]选择尚未安装旧cid的R2，窗口[31,40]。R2共同quorum须于40.999s之前Ready/Enter，首个新物理h31在41s之后通过Final、Publisher和GFR进入G，早于其Expire53s及session公开截止54s。

若正常用例实际落在区间的第二槽28s，act改为38s而目标仍h30，仍按同样截止验收。必须保存实际收录点。用例至少在生产入口向量中覆盖区间两端，禁止因实际落点不同改写签名。恢复完成点是第一份合格新 physical 实际进入G。

M5配置的有限DEP至少覆盖h50，实际L1输入至少覆盖至虚拟60s。这样覆盖晚收录和新窗口完整上下文。主正例截点为恢复首块接纳并完成受控收尾，负例按明确观测上界结束。所有运行最大虚拟时间60s、最多1,000,000事件、每命令wall watchdog1800s。暂停/终止仅控制本次启动的进程。

## 对照与验收矩阵

| ID | 层级与刺激 | 通过条件 |
| --- | --- | --- |
| AR01 | 主15-worker、Espresso停止finalized后恢复 | G h1、实际h1批次和D相等；本地stop与缺覆盖WAIT；恢复后Expire→公开HALT、D越过且共同前缀相等；2/3自动最小目标、真实RESUME、R2首块及时进入G |
| AR02 | 同SHA、同seed主运行 +650ms墙钟暂停 | 规范化完整虚拟trace、Grant/Ack frames、实际L1收据、DA和历史切面逐字节一致，原始诊断保留 |
| AR03 | 15-worker、q前进但θ冻结后恢复 | 故障期间q增长，θ保持不变，无恢复签名；本地stop和公开WAIT保持可追溯；恢复后同一自动路径成功，按实际收录与目标核对全时限 |
| AR04 | 短停、普通D落后、D较高但兼容NEXT、无恢复理由 | 相同生产治理入口给出准确拒签/等待。短停可用有界公共流或15-worker短运行，恢复后足够观察内无RESUME。普通高度差不被记录为hash分歧 |
| AR05 | 实际共同高度hash分歧的观察向量 | 使用M4已验证或新构造的真实G/D执行证据，signer核验不相等native hash后给出HASH_DIVERGENCE；可选实跑在12.25s停滞、22.5s恢复，以[24,26]区间求h28，具体仍需H_guard允许。篡改、不完整或仅高度差不给出该理由 |
| AR06 | 输入/服务未就绪、signer读者落后、时钟不可信 | 原位WAIT/拒签，已接受缺DA不被补空；补齐后重算当前heads与目标，至少两个实际就绪者才形成门槛 |
| AR07 | 求解生产入口与H_guard边界 | 下界ceil/上界floor、严格大于heads、最大推进、区间两端、无整数解、无进展界及溢出正确；最小目标有独立数值预期 |
| AR08 | 实际L1迟收录 | 签署内容固定，收录超预计区间被记录。用真实actual act和位置切面处理，若安排已过期则完成对齐并HALT；不将此运行计为正常时限成功 |
| AR09 | 时钟和RPC对照 | ±1ms偏差与独立RPC延迟有不同配置/原始记录；至少一组提高RPC延迟仍在lease/准备预算内实际恢复，过期样本或超过stop的向量拒绝新签名 |
| AR10 | 生命周期、去重及历史切面 | 持久terminal、同事件批次、重复请求/读者输入、迟到旧结果/工作不复活；实际D已越过目标时仍按精确ResumeCut恢复 |
| AR11 | 同候选M1–M4回归 | M1健康满窗、M2健康单签Force、M3自动2/3Force和M4预设RESUME主路径及受影响专项向量通过 |
| AR12 | 可回放证据与独立检查 | 每条触发、求解、签名、收录、GFR应用和首块能回到原始输入。两个reader同位置一致，独立oracle重算G/D、求解、签名门槛及时间界 |

AR01/02/03为真实15进程正例。AR08至少包含一笔真实迟收录RESUME交易及其原始receipt，GFR低目标后HALT可由相同记录的生产重放补验。迟收录 fixture 在实际 Anvil Submit 之前显式保留已签 h30 的同一原始交易，直至44s前释放并在 b22/t44 收录。分别记录请求准备、预驱动保留、实际发送、收录和finalized。此时act54s，下一高度h31的Expire53s严格早于act，确定完成对齐后HALT。请求 signed_origin 仍为原b13。该故障可能使本次新窗口无法启动，报告其真实结果。AR09的提高RPC延迟正例采用单向1000µs，其余参数保持默认，需核对实际时钟年龄和新成员准备。AR04/05/06/07/10可使用生产入口的独立向量，明确具体执行层级及所用原始资料。

## 证据与交付

保存完整候选SHA、父基线、clean状态、合同与独立预期hash、工具/二进制/lockfile hashes、seed、profile和场景故障参数。原始证据包括15个PID/退出、受控grant/Ack/journal、Clock样本与lease、本地stop、公共pc与WAIT、Espresso每个q/θ和停滞恢复观测、每signer独立L1/GFR/D/Inbox位置、理由/服务探针、H_guard各分量和增量依据、最小整数解与区间、实际签名与原交易、收据/DA、位置ResumeCut、G后缀与来源、新quorum准备以及首个Final/公开完整/G接纳。

单重型锁 `/tmp/megaroation-poc-heavy.lock`、Cargo jobs2、固定Rust1.95和锁定离线依赖。旧M4封存证据和受测候选保留。Verifier从来源与合同独立预期后审查候选，Tester在同一复核SHA运行。所有实际观测分别记录虚拟、协议本地、公开时间键和wall诊断。正常时限只在其声明前提成立时给PASS，缺前提/迟收录明确保留结果。交付准确SHA、manifest、独立报告、全部原始trace、一条运行命令及一条离线检查命令。

## 实际接线与编码兼容

M5 复用一个 09 Driver 和 11 Gateway。除 A/B 外，三位 signer 分别从具名 finalized reader 和对应独立 D feed 取得完整 D native hash 前缀、当前 D 的 origin/末端、已排空边界以及同位置已验证 Inbox cursor。GFR 和 Clock 样本在 signer 内逐份验证。记录已公布 vs 已交付位置、数据缺口和完成边界。正常的未来窗口等待可与缺失已经公开的材料区分。独立 checker 需要从实际 L1/DA 重新计算 D 及证明这些观察输入。真实 h1 批次、DA 取回、当前服务可用状态和无未决队列是本场景的服务/增长依据。配置所选的新区域及其准备预算在签署前冻结，求解不得读取故障脚本。

原生 LocalStop 使用现有 driver 的 lease、本地协议时钟及持久化 Ack 证据。若 Manager 另需收尾，绑定实际终止证据，不能以无关或重复的 Close 捏造终止。旧 G 的显式 Expire 位置用于实际 GFR 重放 WAIT 观察。Lead 允许必要的 `poc/prototypes/i1b-runtime/src/native_config.rs` 仅用于 M5 时钟配置，以及 M4/M3 Python 入口的最小复用接口。M1–M4 的已有公开编码、脚本默认行为和测试 profile 保持兼容。Signer 用相同的 M4 EIP-712 RESUME message，单独把理由和数值依据写入可检查的观察/签名记录；不更改合约签名域。新增字段以场景自己的版本化证据格式记录并在 Builder 交付中列出。其他共享类型和依赖改动须由 Lead 具体核准。
