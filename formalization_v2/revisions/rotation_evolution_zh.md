# Cross-region rotation：版本修改与验证记录

本记录跟踪同一个目标：完成 v3.2 中英文稿，经新的独立审查者找安全性和活性问题，按版本修补并证明可成立的性质。不能通过增加 v3.1 的假设来让证明成立，也不能把“全球链增长”偷换成“所有健康地区最终轮到”。

当前最新稿是 [v3.11 中文](../sync_hotstuff_rr_part2_rotation_v3_11_zh.md) / [英文](../sync_hotstuff_rr_part2_rotation_v3_11.tex) / [PDF](../sync_hotstuff_rr_part2_rotation_v3_11.pdf)。在原模型及作者明确的两项含义下，三份新的独立审查已检查安全、全球 DATA、地区服务和完整请求纳入链，均未发现 Critical Error 或 Justification Gap；详见[本版审查结论](../reviews/v3_11/verdict_zh.md)。原 Model 逐字不变。完整目标另按逐项交付审计核对。

## 最新变化：v3.9 → v3.10 → v3.11

| 版本 | 实际变化 | 审查与后续 |
|---|---|---|
| v3.9 | 从已公开 DATA 中导入请求；第一份含请求的结果怎样形成仍开放 | 保留为此前的局部进展 |
| v3.10 | 旧 owner 保留请求；新成员获取固定 cut；Ready 前形成精确 anchor 的 OC；首份 own-session DATA 执行选中请求 | 新审查发现 CE-OC：接收已有 OC 后可取消计算，但缺少首次持有转发；另发现 ingress 定义缺口和公式排版错误 |
| v3.11 | 收到或形成合法 OC 都先登记持久转发，再取消冗余工作；Closed/Terminated 后仍继续；补齐形成证明并修正公式 | 三名新审查者重新检查准确的完整快照；安全与完整活性链在声明范围内通过 |

作者已明确原文的两项含义：同地区新旧正确 AUTH 成员受原通信界覆盖；发送给 region 表示每个成员最终收到。记录见[原模型解释](v3_10_scope_zh.md)。没有因此添加跨地区传输、客户端到达时限或新成员在准备前全部知情的保证。

现在的正常路径是：旧正确成员保管请求 → 新成员完成固定范围的接力 → 针对确定的 anchor 形成 OC → Ready/Enter → 第一份 DATA 执行 OC 的选择 → 区域 FINAL → 原 Publish Layer → Espresso 上的连续接受。若这次选择的是排在 x 前面的请求，也必须是尚未进入 G 的有效请求；这类较早请求有限，所以不能永久跳过 x。

工作代价也保留在定义中：owner 不能因 worker 结束就删除 pending 请求；新程序的 Ready 需要实际完成有限准备；收到 OC 后的发送职责不能因 Force 被取消。准备没有新增统一时限，既有公开窗口和进入后的 timers 不重置。首次持有的去重键是准备 context K，不是另造一套 view 编号。

[v3.10 的失败记录](../reviews/v3_10/verdict_zh.md)和[v3.11 的具体修补](../reviews/v3_11/change_summary_zh.md)均保留。以下是早期迭代的历史记录，其中“未解决／blocked”描述的是当时状态；通信域和 ingress 澄清现已完成。v3.2 的[中文](../sync_hotstuff_rr_part2_rotation_v3_2_zh.md)和[英文](../sync_hotstuff_rr_part2_rotation_v3_2.tex)亦保留不改。

## 历史变化：v3.8 → v3.9

这次补齐的是“公开结果里已经有请求字节，读者接下来具体做什么”，不是“请求一定能首次发布”。

| 问题 | v3.9 的动作 | 保留的限制 |
|---|---|---|
| 含 x 的旧 DATA 已公开，但因为 Force 不能加入 G | PublicStore 保留真实原字节，后台扫描 typed DATA batch | 不接纳旧 DATA，也不重开旧 session |
| 任意字节不能直接作为有效请求 | 先存入未验证候选 U；使用已备齐的固定 parent 状态检查 | 没有状态或检查不通过时，不进入 pending |
| 恶意编码冒用真实 client ID | 验证前仅按精确字节去重；认证成功后才按真实身份合并 | 失败请求不能占用真实请求的身份 |
| G 和候选不断变化，后台工作可能反复重来 | 固定有限输入、固定只读 context、持久 FIFO 游标，继续执行到完成 | 不承诺新的处理时限、吞吐或无限资源 |
| worker 结束后任务丢失 | 节点持久 owner 接管候选、context 和未完成工作 | 不因此增加跨 AUTH 通信保证 |

导入不阻塞 SOURCESTEP、P6、Ready 或 REPORT。当前 main 和 24 个依赖文件的准确快照分别经[安全审查](../reviews/v3_9/fresh_safety.md)、[进展审查](../reviews/v3_9/fresh_progress.md)及[模型与导入审查](../reviews/v3_9/fresh_model_import.md)。三者在声明范围内均未发现 Critical Error 或 Justification Gap，见[中文结论](../reviews/v3_9/verdict_zh.md)。79 页英文稿已编译、渲染并检查；模型与假设仍逐字保持 v3.1。

独立研究另补出了 [hydration 到 Ready/REPORT 的接入方案](../reviews/v3_8/v39_hydration_ready_adapter.md)，并完成一次[条件性接口审查](../reviews/v3_8/v39_hydration_ready_adapter_focused_review.md)。它还依赖原模型是否覆盖新旧成员通信，尚未并入正式稿。完整请求活性仍需要真正的接力或结果形成保证，不能用这次局部补丁代替。

历史 AUTH 替代路线又补出了[运行接口](../reviews/v3_9/historical_runtime_integration.md)和[控制接口](../reviews/v3_9/historical_control_integration.md)，并完成[新的独立联合审查](../reviews/v3_9/historical_integration_fresh_review.md)。局部条件论证未发现新错误；完整任务发现和历史成员服务范围仍未解决。这些是候选研究，未被包装成 v3.10 或已完成的全目标。直白说明见[请求研究记录第 7 节](request_handoff_progress_zh.md#7-历史任务候选把运行与控制接口写成步骤)。

[当时的停止点审计](../reviews/v3_9/goal_impasse_after_integration.md)确认，已识别的独立修补任务完成后，进一步完整整合需要先确定原通信与参与域。当时该澄清连续多轮未决，目标被置为 blocked；这不撤销 v3.9 的交付，也不把局部通过写成全目标完成。需要回答的原定义见[研究记录第 8 节](request_handoff_progress_zh.md#8-当前停止点需要确定原定义再继续完整协议整合)。

## 固定基线

- 基线：`v3_1_frozen/` 保存当前英文、中文 v3.1 和 Part I 完整源文件及 SHA-256。
- 冻结范围：英文第 2 节的模型、全部假设、紧邻说明及进展条件原样保留。
- `check_frozen_assumptions.py` 检查文字和源文件身份；它不能排除其他章节暗中引入额外前提，独立审查还需检查语义。
- v3.2 之前已完成四模块提纲、当时的两张图，并获得退役与活性范围预检。预检不是对完整 v3.2 的独立验收。

## v3.2：把并发程序和接口写完整

状态：中英文正文与 44 页英文 PDF 已完成；三名新的独立审查者已审查固定快照。v3.2 未通过完整安全活性验收，保留为可追踪版本。

本轮保留原 DATA/DRAIN、常规交接、MSIG/AUTO 优先级、旧 β 匹配、历史选区 reference、`NextLive` 排除前驱以及原窗口规则。

实质修改：

1. 每个正确区域节点运行 Handler；每个 `(β,r)` 有独立程序记录、BFT 状态和异步任务。
2. 生命周期写成 Waiting、Ready、Active、Frozen、Completed、Terminated；Protocol 4 的 Enter 完成 Ready → Active。
3. 明确定义交接包、精确 anchor 和 Ready；候选预热不产生投票权，旧回调不能启动新目标。
4. P5/P6 是公共状态的唯一更新路径；P3 只使用结果管理本地程序，避免 Force 执行两次。
5. 明确单个 FINAL 的聚合与返回接口。修正 v3.1 正文与协议框中 FINAL 接收方不一致：签名送区域成员及发布收集方；完整 receipt 返回源 engine 和 Handler，再交给 Publish Layer。
6. 增加明确的 RequestClose/FinishClose。停止新的旧版本工作，继续证据检查与已有计时器；程序退出后保留归档、迟到消息路由和发布责任。该变化需要新证明，不能声称原保护 QC 引理原样适用。

已知待审问题：

- 原进展前提只明确给出某段成功服务区间，不能直接推出无限全球进展。
- 历史 ξ 可能永久跳过后来恢复的健康地区。
- `NextLive` 排除源地区，与“只剩一个健康地区也持续服务”的目标存在差距；该场景不能冒充满足“最终出现另一个 eligible region”的原进展前提。
- Force 后停止新投票可能使旧计时器的提交凑不齐完整 receipt。必须保留安全性，但不能保证退役实例继续活跃。

## 验收需要分别证明什么

| 目标 | 需要的证据 | 当前状态 |
|---|---|---|
| 中英文 v3.2 完整、清楚且一致 | 完整文档、编译和逐节核对 | 已写完；44 页 PDF 编译与页面检查通过 |
| v3.1 假设不变 | 源码比较及独立语义检查 | v3.2 至 v3.11 的 Model 均逐字一致；各轮还检查了实际证明依赖 |
| 全局 prefix safety | 对公共重放各转换的证明 | v3.11 的全局前缀和输入有效性证明经新审查；包括 public FINAL kernel、逐 DATA 提交及 Force cutoff |
| 生命周期和退役安全 | 唯一进入、隔离、归档、剩余计时器证明 | v3.11 复核全 phase 关闭传播、唯一进入、接力/OC 服务和退役责任移交 |
| 至少一个 eligible 地区下的持续全球 DATA 进展 | 非循环响应界、重试及公共时钟证明 | v3.11 在原区域模型和明确的 prepared-message 接口下经独立检查；不扩展到其他被选 quorum 永久缺失 |
| 每个持续健康且合格地区的服务活性 | 公平选区及接受服务证明，不能仅凭 G 增长 | v3.11 保留并检查 owed 集与 regular 循环的证明；地区服务仍以 DATA 真正进入 G 为准 |
| 用户请求最终纳入 G | 不能以空块进展代替的请求性质 | v3.11 的 ingress → 持久正确 owner → 有限例外 slots → OC → accepted 首块 → 有限势函数链已独立检查；须满足原输入与持续 admissibility 条件 |
| 新独立对抗审查及修补循环 | 每个准确版本快照的独立报告 | v3.2 至 v3.11 的报告、快照和修补均保留；v3.10 的新发现经 v3.11 修补后，又完成三份新报告 |

如果某个目标在冻结模型下不成立，记录完整反例和限制来源；不得把目标删掉或写成已完成。

## v3.2 独立审查结果

审查材料固定在 [`reviews/v3_2/snapshot/`](../reviews/v3_2/snapshot/)。三名审查者均重新核验文件哈希；未参与原文写作，未互相读取报告。

| 来源 | 发现 | 处理 |
|---|---|---|
| [安全审查 S1](../reviews/v3_2/fresh_safety.md) | early timer 的 protecting-QC 论证漏了已有成员 Completed、因而不投票的情况；给出了无 QC 的合法 DRAIN 轨迹 | v3.3 增加完整 receipt 区域 multicast，并补 Completion 取消分支 |
| [安全审查 S2](../reviews/v3_2/fresh_safety.md) | 用“版本现在仍开放”递归验证历史 anchor，可能误拒绝保留的旧证据 | v3.3 分开历史 handoff 验证和当前 CanEnter |
| [证明审查 P1](../reviews/v3_2/fresh_proofs.md) | 首个 Ready 后全员在 Δ 内 Enter 的界没有明确覆盖执行状态安装 | v3.3 定义完整 ReadyBundle；仅对已有完整消息处理界确实覆盖的采用动作给出该界，不能假装原生数据库导入有新上界 |
| [活性审查 T1](../reviews/v3_2/fresh_liveness.md) | W_start=6、Δ=1 时，诚实 leader 真正出块并提交，但完整结果在 AUTO 之后形成；可导致 G 停止 | 当前窗口与 cutoff 规则仍需实质修补 |
| [活性审查 T2](../reviews/v3_2/fresh_liveness.md) | 穿插快速合法空 DATA，可让 G、各区域输出都增长，而正确请求一直留在被关闭的后缀 | 不能用吞吐量或 leader 公平代替请求活性 |

安全审查没有发现已确认的 global fork 或 G 回滚，但这不等于完整证明已通过。两名审查者对 retirement 的条件论证宽严不同；本轮采用给出具体 Completion 缺口的意见补齐，而不是按多数票判定证明正确。

活性反例明确区分原文量词：T1 在“存在一段有限成功服务区间”的读法下成立；若原文意指每个未来 duty 都有足够成功服务时间，需要把该含义从既有文字中证明清楚，不能悄悄加上。历史 ξ 饥饿是否满足全局 leader 公平前提也有解释边界；singleton 不满足原来的 another-eligible 前提。这些限定已经写入 v3.3。

## v3.3：修补已发现的接口与选区问题

状态：英文及中文候选稿已写，49 页英文 PDF 编译、渲染与页面检查通过。准确快照已固定，两名新的独立审查者已完成审查，发现新的关闭传播缺口，正在 v3.4 修补。

| 改动 | 新规则 | 不应误解为 |
|---|---|---|
| 完整 receipt 回传 | 正确 holder 首次拿到 receipt 后向固定区域成员广播完整证明；Completed 和归档 owner 也保留这项责任 | 不是新共识，也不是单个 FINAL 已构成 receipt |
| 历史与启动分开 | VerifyHandoffHistory 验证当时的固定事实；CanEnter 检查当前版本、目标、退出记录和唯一进入 | 旧证据有效不代表旧程序还能启动 |
| ReadyBundle | 传递完整可执行状态及验证依赖，明确 validate/adopt/Enter 的处理范围 | 不给任意磁盘导入或恢复工作增加时间保证 |
| terminal ξ 向前 | terminal 的 Espresso height 严格大于本 session 的 opening reference；Ready 前准备可用 header | 不需要等待这个 terminal DATA 本身全局确认 |
| 同区续任 | 优先选其他 eligible region；没有时才选仍 eligible 的源地区；AUTO cooldown 和 OFF 仍有效 | 不复用旧锁、旧 timer 或旧 session 身份 |

新增选区命题只证明：若常规交接无限继续且资格集合稳定，reference 不会永远停在旧高度，循环会反复访问每个 eligible region；以及单一区域不再被选择器永久排除。它不证明这些无限成功交接一定发生。完整的持续活性和请求活性目标保持未完成。

## v3.3 独立审查与 v3.4 在做的修补

[v3.3 安全报告](../reviews/v3_3/fresh_safety.md)发现一条此前没有覆盖的路径：H0 已经进入，H1 还在 Waiting；H1 先看见 Force，直接 Terminated，却没有明确向 H0 转发完整关闭证据。H0 在自己的 Espresso 可见延迟内继续运行，H1 不再投票。按这个字面流程，报告构造了 H0 无 QC 提交后又恢复到旧 anchor、提交冲突块的区域执行。全局 G 没有因此回滚，但区域退役证明不能接受。

这里有一个规范歧义：原句“Notify regional nodes”可能想涵盖所有成员，但没有明确完整证据、接收范围和未进入程序的义务。不能用这句宽泛描述排除具体执行，必须把动作写进协议。

**v3.4 的修补方向：** 关闭证据属于 Handler 的持久职责，不属于某个 Active worker。每个正确节点无论程序处于什么 phase，都保存 `Closed[β]`。已知固定成员快照时立即传播完整关闭；尚未知晓时保留待传播任务。以后收到该旧 session 的完整 proposal、ReadyBundle 或 authority context，就先补齐关闭传播，再做普通处理。Terminated 后的归档路由也承担此职责。

v3.4 中英文主稿已完成，并增加“拒绝旧投票不能静默”的引理。其 51 页 PDF 已编译和检查；没有改动 v3.2/v3.3 的已审查快照。

[v3.3 活性报告](../reviews/v3_3/fresh_liveness.md)认可完整消息处理解释下的 Ready 时间推导，以及选区局部命题；仍未证明完整活性目标。[严格 cutoff 方案分析](../reviews/v3_3/strict_cutoff_options.md)进一步指出，不能笼统断言严格截止无解：固定 G 时保留准备与调度进度、避免每次依赖全新 authority、配合自适应预算仍是一条需要证明的路线。当前也向用户询问是否允许改变 Force 截止语义；未收到答复前，不实施延迟 Force 或接受关闭版本输出的方案。

## v3.4 与 v3.4.1 的检查结果

[v3.4 安全报告](../reviews/v3_4/fresh_safety.md)与[接口报告](../reviews/v3_4/fresh_interfaces.md)没有发现全 phase 关闭传播、完整 context 回复、4Δ drain 或 early/late 退役论证的新反例。结论仍以既有区域/应用 soundness 和明确的完整消息处理合同为条件，不是对具体 native adapter 或所有实现的保证。

两者独立发现一个表述错误：完整 interpreter state 中包含未来控制队列和来源读取游标，所以同一 Espresso 前缀不能保证这些完整状态相同。**这不是 G 分叉，但原命题的该子句确实过强。**

v3.4.1 只修正定义和证明：比较已生效的公共 replay result，明确排除尚未到期的 L1 控制和本地读取进度。六个协议框、运行片段及全部模型文字均与 v3.4 相同。新审查者的[勘误检查](../reviews/v3_4_1/fresh_erratum.md)确认该范围内无 Critical Error / Justification Gap。51 页英文 PDF 与中文稿同步完成。

下一步继续研究严格 cutoff 下的活性修补，重点是防止零进展重试反复丢弃准备/调度工作，以及解决强制切换前后的正确请求饥饿。已经验证的局部安全性不能替代这一剩余目标。

## 2026-10-19：按补充更新架构提纲与中文图

本轮按用户对系统模块的补充，更新[架构修订提纲](../sync_hotstuff_rr_part2_v3_2_revision_outline_zh.md)，并生成[三页中文架构图](../figures/cross_region_rotation_arch_revision/architecture_zh.pdf)：四模块、并发准备时间线、六状态生命周期。原提纲保存在[历史副本](../sync_hotstuff_rr_part2_revision_outline_before_2026_10_19_zh.md)。

四模块仍为 Soft Detector、Multisig HealthController、Handler、Per-session Ready Gate。新增说明明确：前两者并列；Protocol 5/6 只执行一次公共转换；候选可更新而正式 anchor 固定；History 与 CanEnter 分开；Ready/Enter 在同一事件完成；关闭证据服务不随 worker 删除。

示例与当前 v3.5 草稿对齐：零输出 Force 保留 slot，已有接受结果时推进 slot。v3.5 尚在整合与后续独立审查阶段，本轮图文校对不构成其安全性或活性通过声明。v3.1 冻结模型逐字检查通过；本轮没有改动协议主稿或假设。

## v3.5：持久重试、窗口容忍与地区服务

英文、中文及 63 页英文 PDF 已完成，六个 Protocol 编号保留。编译无未解析引用或排版溢出；图和页面已检查。审查输入固定在 [v3.5 快照](../reviews/v3_5/snapshot/)，主稿 SHA256 为 `2f47bca40d7ebdf80bb68a49a5fb8c479c4a50484a753d0134defc9381831d6b`。英文 Model 与中文第 2 节均保持 v3.1 原文。

| v3.5 改动 | 要解决的问题 | 保留的边界 |
|---|---|---|
| 零输出 Force 保留 slot；准备与 authority 缓存跨 beta 保留 | 避免每次重试都等待全新的 authority、重做同一 anchor 状态 | 新 beta 的锁、votes、timers、g/t 全部重新初始化 |
| 固定基础窗口，容忍 `1+beta` 次连续 miss | 避免固定短窗口一直打断正常区域结果形成 | 不推迟已经匹配的 MSIG；Force 仍按公开 crossing 先于 payload |
| REGULAR 在完成后推进 selection floor | 避免初始 Ready 先等待更晚 header 而被 AUTO 反复关闭 | terminal DATA 自身仍无需先全球确认；DATA/DRAIN 不变 |
| SOURCESTEP 与完整 ReadyBundle | 把源输入、公共重放、Handler 与 Enter 的事件连接写完整 | 不给任意 native import、离线 catch-up 添加时间界 |
| 有限健康签名额度与独立 OFF/ON 租期 | 避免固定 G 下控制服务无限重启或永久临时排除 | 已签控制的目标、优先级和激活语义不变 |
| Owed 集只限制 Force 选择，最后一个候选不新增 AUTO cooldown | 尝试保证地区实际交付，并消除自动制造的无人接班间隙 | REGULAR 不受 owed 过滤；不覆盖 OFF 或已有 cooldown |

### 新独立审查：尚未通过完整活性目标

[安全审查](../reviews/v3_5/fresh_safety.md)未发现符合冻结模型的新分叉、回滚或退役安全 Critical Error，但发现旧 observer 图仍把所有 Force 的目标写成 `r_c+1`。这是图文规范不一致，下一版需要复制并修正该图，不能修改已经审过的旧版本。

[假设与目标审查](../reviews/v3_5/fresh_assumptions.md)指出一个实质缺口：**结果已被 Espresso 收录，可以满足原发布保证；外层 EXPORT 错误却仍可能使它无法进入 G。** 原假设明确没有为接受重试提供统一时限，因此证明不能直接把 `T_pub+T_E` 用在“纠正后的 exact-base EXPORT”上。审查给出超过所称响应界、随后才成功重试的合法执行。依赖该界的持续全球进展与地区服务证明暂不成立。

[独立活性审查](../reviews/v3_5/fresh_liveness.md)也定位到同一发布缺口，并增加第二项：quota 为 1 时，第一个完整结果就是 terminal DATA；原子 EXPORT 接受会调用下一 slot 的 `PrepareOwed`，而 `AUTH[r+1,0]` 没有被纳入已证明的准备集合。正文没有清楚保证当前 terminal 的 G 更新先于这个无统一上界的查询。两份报告分别将发布问题归为 Critical Error 和 Justification Gap；本轮不靠多数票决定，统一撤回其依赖证明的通过状态，并在下一版修补接口。

请求服务仍有独立问题：一个 Byzantine 空 DATA 先进入 G，随后正确 leader 含请求 x 的结果被 Force 截断；正输出又使下一次转到新 slot，新的 authority 可再次延迟。这可以反复发生，同时满足区域公平与发布条件。**G 增长和地区得到空块 credit，不能代替请求 inclusion。**

下一版优先修复两个接受接口：让真实区域结果的验证不被发送者随意填写的外层 base 阻断；让当前 terminal DATA 的提交不依赖下一轮 authority。修补必须覆盖原“already recorded”保证，不得偷偷增加重试时限。请求活性另外研究，不把它从目标中删除。

## v3.6：按真实区域结果接受，terminal 先提交

状态：中英文候选稿与 67 页英文 PDF 已完成；新一轮三名独立审查者正在核对准确快照。主稿 SHA256 为 `ef6d858b8ead057dd77f1a43e4198d47ceceb016e5c7b826eb941878b59d96d5`。英文 Model、中文第 2 节仍与 v3.1 逐字一致；未改动 v3.5 审查快照。

| 修补 | v3.5 | v3.6 候选规则 |
|---|---|---|
| 外层 base 等提示 | 错误提示可拒绝整个 EXPORT | 从签名 DATA、原 receipt 和完整依赖判断；外层错误提示不能否决独立有效结果 |
| 坏批次夹带好结果 | 外层批次全部成功或全部失败 | 按每条兼容 DATA 提交；无关坏条目不能回滚已接受前缀 |
| 结果位于 ancestry 字段 | 声明区间可能漏掉实际下一块 | 解码实际 DATA／证据对象；用完整 receipt 覆盖推导下一块，不相信发送者索引 |
| terminal 后继 authority | 可能把当前接受拖入无界等待 | 当前 DATA、计数、pending boundary 先持久提交并公开，再查询下一 authority |
| 处理中断 | 主要以完整记录或区块描述 | 保存确定的记录内位置，按原 timestamp 继续；不重复 prelude、计数或选区 |
| 发布时限证明 | 依赖尚未覆盖的 corrected retry | 分析第一次携带完整结果的记录，包括攻击者先发的错误提示记录 |

逐条提交仍先执行原 Force prelude，旧版本在 cutoff 后不能追加，新版本不能在触发块内追加。过去已经处理完的记录不会在未来被重新打开接受。新逻辑明确要求可解码的完整区域结果；它不能把 opaque bytes 或不可发现 namespace 说成已有有界可见保证。该解释是否完整覆盖冻结的传输合同，是本轮独立审查重点。

图中的 `r_c+1` 残留已在 v3.6 自己的 observer 图修正。四模块与最新中文架构图继续保留。请求 inclusion 反例仍公开保留，未在这轮偷偷改变 FIFO、增加请求发布假设或声称完整目标已完成。

### v3.6 新独立审查结果

三份报告已完成：[安全](../reviews/v3_6/fresh_safety.md)、[活性](../reviews/v3_6/fresh_liveness.md)、[传输与假设](../reviews/v3_6/fresh_transport.md)。[中文汇总](../reviews/v3_6/verdict_zh.md)保留各报告的条件与分类。

未发现支持格式内的分叉／回滚或新的错误包装反例；逐 DATA 提交和 terminal 先于后继 authority 的修补得到认可。仍未通过完整活性：旧公共引用如果发生缓存缺失，重新读取完整字节没有既有统一时限；仅保存 cursor/index 不能证明完整 SOURCESTEP 在 `a+T_E` 已经可发送。传输审查还要求明确从原完整逻辑记录到新对象解码的覆盖关系，以及 handler/native 动作属于原处理界的具体范围。

下一轮将以协议规定持久保留和有限初始化处理这个缺口，而不是增加档案读取、authority 或 native 恢复时限。v3.6 的准确快照和候选证明保留不改。请求 inclusion 和更宽的失效地区模型仍是公开的未完成事项。

## v3.7：完整公共字节保留与 FINAL 验证

状态：中英文稿、72 页英文 PDF 和三份新独立报告已完成。准确主源 SHA256：`440d639a3f9a7a0a236836153880171db40c904a1862eef2444b8313c9d2b476`。完整 Model 和中文第 2 节保持原文。主稿仍保留六个 Protocol、六种 phase、原 DATA/DRAIN 和严格 Force cutoff。

| 修改 | 作用 |
|---|---|
| 节点级 PublicStore | 保留完整原始 namespace、对象字节和证据；不随 session 退出、beta 改变或 G 前进删除 |
| 固定有限历史补齐 | 先完成固定 cut 的读取，后续输入另行持续保留；不靠不断移动的目标掩盖无界等待 |
| RawFINAL 与 CheckFINAL 明确定义 | 从原固定快照的真实正确签名推导输入有效性，不循环引用“有效 receipt”的定义 |
| 公共与区域验证分开 | 区域内部继续完整检查；公共解释器不跟随发布者的可选 AUTHg／PROPOSE／native 证明路径 |
| 逻辑 carrier 保留引理 | 保留旧逻辑 EXPORT 的 DATA 与证据子图、原 source denotation；不假装证明了未知二进制编码兼容 |
| 完整依赖分类与时间证明 | 逐项说明首个 DATA 前要用什么；旧 E 字节已保存、控制提前可见、当前 authority 已准备，可选证明不阻塞 |

[安全](../reviews/v3_7/fresh_safety.md)、[活性](../reviews/v3_7/fresh_liveness.md)、[模型与传输](../reviews/v3_7/fresh_model.md)审查均未发现所检查抽象证明中的 Critical Error 或 Justification Gap。详见[中文结论](../reviews/v3_7/verdict_zh.md)。这次支持了持续全球 DATA 进展和持续 eligible 地区的反复接受服务，不只支持局部窗口引理。

仍未完成的是请求 inclusion：原空 DATA／迟到 recovery／Force 截断反例保留。已请求澄清后续是否允许修改 FIFO 选请求算法；在答复前，不自动把“地区 credit”作为请求完成，也不修改冻结假设。具体 codec、native adapter、存储实现和原模型外的永久缺 quorum 也没有被冒充为已验证部署性质。

## v3.8：让第一块 DATA 的请求选择可以验证

状态：中英文候选稿、77 页英文 PDF 和三份新的独立审查已完成。英文主稿 SHA256 为 `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574`。原 Model 和中文第 2 节保持逐字一致。所审条件性结论未发现 Critical Error 或 Justification Gap；完整请求目标仍未完成。

后续源码审查区分了环境假设与算法：v3.1 的 first-receipt FIFO 选块行为写在协议部分，Model 不要求所有节点采用同一请求顺序。本候选版在“继续演进协议”的授权内显式修改首块选取；未把此前选项问题的默认项当作用户回复，也未加强请求输入条件。

| 问题 | v3.7 | v3.8 候选规则 |
|---|---|---|
| 恶意 leader 持续用空块拿全局进展 | 区域证书只证明原块有效 | 每个 term 的首个 DATA 带权重 >F 的 INPUT_REPORT；报告有请求，就先执行其中最小键请求 |
| 本地队列顺序不同 | 本地 FIFO 决定正确 leader 自己的选择 | 验证者检查同一份报告证明，不用自己的队列否决合法提案 |
| 最小键是什么 | 没有这个选择顺序 | 规范请求字节长度，再比较 hash；每个固定请求只有有限个更小键 |
| FIFO 的位置 | 整个批次都按本地 FIFO | 首个 DATA 的 primary 优先；剩余空间及后续 DATA 按原 FIFO 填充 |
| 一个正确成员收到请求 | 主要依赖原输入条件 | 转发给其所属的已认证 snapshot；worker 退出不删除 pending 和 relay 责任 |
| REPORT 如何影响原 BFT | 没有 REPORT | 不是 vote、QC、Quit 或 timer；ANCHOR vote 不等 REPORT；进度检查不重置 |
| 额外证据如何发布 | 原完整 regional result | REPORT 是区域验证 sidecar；Espresso 收到完整原结果 R0，不能只发 kernel，也不新增公共 REPORT 字节 |
| 请求证明 | 空块与 Force 截断反例保留 | 无限多个 accepted 首块中若有知道 x 的正确报告者，有限前驱论证迫使 x 进入 G |
| 尚缺的结论 | 全请求活性 | 新旧 AUTH 成员完全不重叠时，pending requests 如何接力，仍需解决 |

直白例子：三个成员 H0、H1、B，前两个正确，阈值为两份签名。若 H0、H1 都知道唯一待处理请求 x，任何两份 REPORT 都包含一个正确报告，其值是 x。因此恶意 B 不能再给空 DATA 配出合法的全 EMPTY 证明。它可以停机，但那会走已有恢复或 Force 路径；不能同时拿空块进展并一直绕过 x。若新 roster 还不知道 x，这段推理就不适用。

候选稿未增加跨 roster 通信界、可信 mempool、裸请求的 Espresso 发布保证或新的 Ready 等待。独立模型审查指出：历史 snapshot 内的 manager 服务有原 Δ 支持；不相交新旧 roster 之间的时限不能只靠取两个 Δ 的最大值推出。已向用户询问原“within region”通信域的含义；未收到回复前不采用更强解释。

新的完整审查记录见[中文结论](../reviews/v3_8/verdict_zh.md)，以及[安全](../reviews/v3_8/fresh_safety.md)、[进展](../reviews/v3_8/fresh_progress.md)、[模型与发布](../reviews/v3_8/fresh_model_carriage.md)。条件定理不等于完整请求活性，整个目标保持 active。

下一版研究已得到经独立复核的逐成员回归饥饿轨迹，并完成请求接力候选的三轮修订与审查。接力第三稿在明确的条件范围内通过检查，混合 snapshot 通信与主协议整合仍未解决。另完成历史 AUTH 复用的独立作用域审计。正式稿尚未改成 v3.9。具体攻击时序、候选修补和剩余模型边界见[请求接力进展](request_handoff_progress_zh.md)。
