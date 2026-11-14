# 请求活性：v3.8 之后的研究记录

更新：2026-10-19。最新完整稿为 [v3.11](../sync_hotstuff_rr_part2_rotation_v3_11_zh.md)，[三份新的独立审查](../reviews/v3_11/verdict_zh.md)已检查完整请求链，在原模型、作者澄清和明确的抽象接口范围内未发现错误。下面第 1–8 节保留此前探索及当时的停止点；它们不是当前状态。最新修改见第 9 节。

## 1. 更强的请求饥饿轨迹

新轨迹不再依赖“旧成员以后永远轮不到”。它让每个正确成员都无限次回来，真正进入并提出含请求 x 的 DATA；两个地区也持续有 DATA 进入 G。但 x 仍可一直无法形成完整区域结果。

关键是：原稿只保证第一次精确 native state 准备最终完成，没有统一时限。知道 x 的 cohort 可以每次都在 AUTO 截止前刚刚 Ready；它提出了 x，却来不及让其他成员启动 commit timer。

设三个成员全部正确，配置的上限为 F=1，每份 receipt 仍需要两份 FINAL，区域 Δ=1。一个知道 x 的 session 有如下合法局部时序：

| 时刻 | 发生什么 |
|---|---|
| e | leader 准备完精确状态，Enter，并发送 ReadyBundle |
| e+1 | 另外两名成员 Enter |
| e+2 | leader 发 NEWVIEW |
| e+3 | 其他成员发 ANCHOR vote 和报告 x |
| e+4 | leader 提出 DATA(x)，自己投票，启动到 e+6 的 timer |
| e+4.5 | AUTO 生效，旧 beta 关闭；所有成员在收到 DATA 前得知关闭 |
| e+5 | 其他成员收到 DATA，保存材料，但不再产生旧 session 的新 vote 或 timer |
| e+6 | leader 已经启动的 timer 正常到期，只产生一份 FINAL |

**一份 FINAL 不够组成 receipt，Publish Layer 的完整结果保证还没有开始。** 这不是 publisher 不肯提交，也不是请求被删除。请求和单份签名都保留着，但没有第二份合法 FINAL。

随后另一个 region 的新 cohort 及时出一个空块。它在签发 EMPTY 报告后才收到 x，所以已有空块仍然合法。常规轮转再给原 region 的新 cohort 一次及时服务。两边都拿到 global DATA credit，owed round 正常结束；下轮再安排一个已知 x 的旧 cohort 晚到。

通过逐步轮流安排所有已出生的 cohort，每个成员都能无限次真实进入并作为正确 leader 提出 x。每个尝试的准备时间都有限，但没有共同上界；不断变化的 G 和新精确 anchor 也没有触发“固定 G 后准备最终稳定”的证明。

完整论证见[攻击轨迹](../reviews/v3_8/v39_recurrent_identity_trace.md)和[新的独立审查](../reviews/v3_8/v39_fresh_recurrent_trace_review.md)。审查确认了时序、zero-output 同 slot、cooldown、owed round、原发布保证及逐成员回归安排。

**范围要说清楚。** 这条轨迹使用固定 snapshot 通信和原文写出的 eventual preparation 接口。如果“足够 uninterrupted service”原本是保证每个知情 cohort 在 Ready 后最终得到完整 receipt 窗口，那么这条轨迹不满足那个更强解释。它证明 v3.8 的请求缺口，并不证明所有协议都不可能修好；也不反驳 v3.8 带明确知情条件的定理。

## 2. 已有 Espresso 传输能解决哪一半

只要 x 已经出现在一份合法的原完整区域结果 R0 中，原 Publish Layer 和 Espresso 就能把它的字节传给后续成员。即使这份结果因为 Force 不能加入 G，数据仍可公开读取；新 session 可以重新检查 x 在自己 parent 后是否有效。

但以下东西没有自动获得这项发布保证：仅在 mempool 里的 x、未形成 receipt 的 proposal、一份 FINAL，或临时附加的请求 sidecar。不能把这些称为“完整结果”来跳过原服务入口。

所以仍需证明的关键一步是：**一个正确旧成员保留 x，怎样保证至少形成一次真正含 x 的完整区域结果，或者怎样把 x 可靠传给会形成结果的新成员。** [完整载体分析](../reviews/v3_8/v39_existing_carrier_design.md)区分了这些情况。

v3.9 已补上“取得已有公开字节之后”的操作：扫描固定有限的真实 DATA 对象，先保留未验证候选；有精确且可持续读取的原生状态时，再执行原请求检查，成功后进入 pending。旧 DATA 被拒绝不妨碍这个流程；各队列和 context 跨 worker 退出持久保留。[本版审查结论](../reviews/v3_9/verdict_zh.md)明确指出，这没有制造第一份含 x 的完整结果，也没有给导入增加 deadline。因此上面的只有一份 FINAL 的轨迹仍未被排除。

## 3. 请求接力候选的逐轮修补

这个候选让原 owner 保留 x，新成员从不可变 AUTH 历史找到旧 owner，复制固定有限队列片段，再向目标 snapshot 全体成员传播。HYDRATED 证明只用现有区域成员签名，不增加另一个委员会。

| 阶段 | 独立审查发现 | 修补方向 |
|---|---|---|
| 初稿 | 分片数没有固定有限上限；只写签 HYDRATED，没写把签名发给谁 | 固定 RECORD_META 长度和分片预算；明确 share multicast、收集和 certificate relay |
| v2 | 上面两项补齐；但何时可以开始 CUT_QUERY 的计时仍依赖“远端已准备好”的未实现判断 | 完整上下文实际传完后，执行明确的验证消息和等待步骤，不等待 Byzantine ACK |
| v2 的证明范围 | 有限例外论证使用了没有在冻结 Model 中写明的全局时间局部有限性 | 用本地首轮查询的显式启动节奏限制早期 target 数量，不把环境的处理速率当作假设 |
| v3 | 上述流程已经补齐；新的独立审查未发现条件范围内的 Critical Error 或 Justification Gap | 仍需确认混合 snapshot 通信域，并完成接入主协议后的整体验证 |

初稿和 v2 的报告分别见[首次审查](../reviews/v3_8/v39_fresh_ownership_review.md)、[v2 审查](../reviews/v3_8/v39_fresh_ownership_v2_review.md)。修补保留旧稿和报告，另写新候选。

第三稿见[接力候选 v3](../reviews/v3_8/v39_ownership_candidate_v3.md)和[新审查报告](../reviews/v3_8/v39_fresh_ownership_v3_review.md)。其具体补丁是：

1. 先实际传完所有 L1/AUTH 验证材料，再发送只验证这些完整材料的 CONTEXT_VALIDATE；从实际发送时刻推导 CUT_QUERY 的最早开始时间，不等无法识别的“所有正确人 ACK”。它不把 native restore 或缺失历史读取包装成有界工作。
2. 编号 r 的第一次准备查询不得早于 `tau_0 + r * tau_p`，其中 `tau_p > 0` 是协议参数。这样在一个固定请求到达时刻之前，能开始查询的 r 有显式有限上限，无需额外假定所有节点的全局事件速率有限。每个固定 r 的等待都有限，但这确实可能限制过快或过早的准备；它不延后公开窗口，也不重置 Enter 后的 timers。
3. 每个 owner 的有限本地队列、实际发送和各自查询 deadline 分开记账；不把一次大 multicast 的所有发送假装发生在同一瞬间。

这条路仍有一个独立前提：旧 owner 与新成员之间是否属于原“within region”的有界通信范围。候选只能在该范围成立时证明接力，不能从“worker 退出后仍有存储责任”直接推出跨 snapshot 的网络保证。相关澄清问题仍未得到用户回复。

之后已写出具体的 [hydration → Ready/REPORT adapter](../reviews/v3_8/v39_hydration_ready_adapter.md)：精确 anchor 状态和有限请求 cut 的选择工作在 Ready 前实际完成；区域成员签发绑定该 anchor 的选择材料，首个本 session DATA 执行其中确定的请求。后续同上下文的另一份有效材料不能因本地先前缓存不同而被拒绝。它保留原 NEWVIEW/PROPOSE 的签名字节，新增材料属于区域 sidecar；原公共 R0 不变。

[新独立局部审查](../reviews/v3_8/v39_hydration_ready_adapter_focused_review.md)没有发现该条件性接口论证的 Critical Error 或 Justification Gap。准确审查 hash 与追加审查记录后的文件 hash 分开保存。跨 snapshot 服务、实际 native adoption 与既有输入大小/负载范围仍须成立，所以它尚未启用，不能据此宣布全协议通过。

## 4. 复用历史 AUTH 是另一条待审路径

原 authority 规则允许新 beta 复用同一个 r。理论上可以建立新的 `(beta_new,r_old)`，继承当前 G，以原 TB_r 从这个新 anchor 后继续普通 DATA。它不会重开旧 pair，也不改已经接受的 G。

这说明历史 AUTH 复用在身份和数据格式上不是死路。真正还要证明的是：旧 quorum 是否仍能工作，如何公平安排这些历史请求机会，以及历史任务不工作时如何保持全局 DATA 进展。旧证书仍有效，不等于旧成员今天仍能参与 BFT。不能为了使用这条路而暗中假定所有旧 quorum 永远在线。

这条路径尚未加入正式协议。当前工作继续区分“可以定义的新 session”与“已经证明可获得服务的新 session”。

[独立作用域审计](../reviews/v3_8/v39_historical_authority_scope.md)给出了准确边界：若原同步性持续覆盖历史 snapshot 的正确投票身份，新增 fresh-session handler 可以只是代码修改；若服务只覆盖当前活跃身份，就还需要历史 quorum 可用性。原文没有足够明确地选定这个时间范围，因此不能仅凭 AUTH 或 ON 擅自作结论。

这与请求接力是两条候选路线，不能直接拼起来沿用全部证明。允许历史 r 无限重访后，“每个有限 r 只贡献有限 accepted DATA”的旧计数需要重新检查，历史选择与 stalled-G 论证也要重做。

本轮已把这个想法写成一份[具体条件性调度方案](../reviews/v3_9/historical_service_design.md)，并交给一名新的[独立对抗审查者](../reviews/v3_9/historical_service_fresh_check.md)。直白地说：旧任务按“地区 + 原 AUTH”排队，只有这个任务自己的 DATA 进入 G 才算得到服务；一旦轮到它，零输出的 Force 继续用新的 beta 重试同一任务，避免其他 roster 的空块不断改变它的 anchor。

该审查没有发现所述局部条件论证的 Critical Error，但列出两个尚未完成的 Justification Gap：如何从公开来源完整而一致地发现任务（尤其是零输出的投机 session），以及如何把队列、健康签名额度和公共处理流程真正接好。更根本的旧身份服务范围仍是明示条件，未由 AUTH 自动推出。若被固定的旧 quorum 已不在原服务范围内，单靠这个排队办法不能同时保证它得到服务和其他活地区继续推进。

补充的[目录接口分析](../reviews/v3_9/historical_inventory_adapter.md)已经写出一个有限子目录：在公共交接实际验证目标 AUTH 时登记它，包含这类零输出公共 duty；其登记位置按公共历史确定，不按某个节点下载完成的时间确定。但是只在私下进入、尚未产生完整结果的投机 session 仍可能不在这个目录里。原 control prefix 的完整性不能直接当成全部 AUTH 的枚举证明。这份补充是来源接口分析，未并入正文，也未被上述独立报告覆盖；它没有消除完整历史任务发现的缺口。

另一个较小的办法是 Force 后允许已经进入的旧程序继续形成完整结果，仍拒绝其 DATA 追加 G。它可以补“只来得及一份 FINAL”的具体缺口，但不覆盖在 terminal proposal 已确定后才到达的请求；有限 quota 已经用完，DRAIN 也不能装新的 DATA。若每次只发布尚未被 G 接受的较小请求 y，原请求势函数也没有下降。因此这条办法尚不能作为完整请求活性的修补版。

## 5. 当前状态

v3.9 的三份新审查支持其声明范围内的安全、DATA 进展、地区服务、REPORT 和公开字节导入证明。请求接力第三稿及 Ready 接入方案都有条件性独立审查，但尚不是冻结假设下的完整解决方案。当前继续核对原通信域，并研究历史 AUTH 的服务安排；下一版必须补上实际请求接力或实际结果形成保证，再重新审查整条链。不能只把“正确成员总会再轮到”写进证明。

## 6. 再查原文：哪些保证已经有，哪些还不能直接推出来

一名未读后续设计或旧审查的审查者，重新检查了冻结 Model、Part I 和原 v3.1；[完整记录](../reviews/v3_9/frozen_region_domain_audit.md)给出了逐条引用。

结论有四点：

1. 原文的 “within region i” 没有明确排除跨 AUTH 消息，但成员和 Δ 又随固定 snapshot 定义。不能擅自说跨 AUTH 一定不受保证，也不能不解释就把它当作永久物理地区内所有身份之间的保证。
2. 原 Completion 明确要求 `Keep serving data`。结束 BFT engine 不等于节点消失；给仍在原通信域里的正确节点增加明确数据服务动作，可以是协议修改。让已经退出该服务范围或离线的旧节点重新可用，则需要先确定原参与时段的含义。
3. 原 lifetime F 限制在旧证书被接受期间累计被攻破的旧 key 权重。不能靠不断攻破退役 key 绕过它；但未被攻破也不自动等于当前在线。
4. “请求最终到达”不等于“本次报告前已经到达”。即使每个新 cohort 都在报告后很短时间收到 x，也没有违背逐成员最终到达。因此原 ingress 不能单独消除当前请求饥饿问题。

这次核对没有改假设，也没有给出普遍不可能性结论。需要确定的仍是原通信域与参与期限，而不是添加一个新的“公平请求 oracle”。

## 7. 历史任务候选：把运行与控制接口写成步骤

本轮补写了[队列、P6 与 Handler 接入](../reviews/v3_9/historical_runtime_integration.md)和[控制程序接入](../reviews/v3_9/historical_control_integration.md)。它们是历史任务方案的后续设计稿，尚未并入正式 v3.9，也不是已通过全协议验证的 v3.10。

运行部分有四个具体变化：

1. 每个地区保存一条 FIFO，任务仍只用 `(地区 i, AUTH slot r)` 表示。要选新任务时比较各地区队首，不在每次重试时扫描全部历史 AUTH。
2. 任务已经固定且仍合格时，新目录项只能排到后面，不影响这次重试。处理首次选择时，保存固定的目录 cut 和游标；后来 Force 的原始数据可以先接收，但不能跳过尚未完成的早期选择，反复换一份更大的扫描输入。
3. 一个任务的 DATA 真正进入 G 时，立即记录它得到服务、移除其旧队列项并排到队尾。即使它是 regular 路径产出的、或当时不在队首，也按同一规则处理。只有匹配当前固定任务的 DATA 才解除该固定任务。
4. G 的提交、服务记账和解除固定先持久完成。后续目录整理、下一次选区或 successor AUTH 尚未齐全，不能撤销或隐藏这份已接受 DATA。

控制部分明确了：需要调度时，必要的 MSIG 请求先于可选健康干预取得同一个签名额度；失败、过期、stale 和重启均不返还额度。OFF 租期还要消耗被影响历史任务自己的额度，不能靠同地区的新 roster 产块来无限补充。旧 beta 匹配和实际提交时刻的提前公告条件不变。

这些步骤消除了部分“交给后台处理”的空白，但没有把未知接口变成保证。运行稿把新增检查是否属于原完整消息处理界单独列出；控制稿把本地工作不会饿死与实际有效 L1 提交分开。完整目录覆盖和历史成员服务范围仍需由原模型支持。

[新的独立联合审查](../reviews/v3_9/historical_integration_fresh_review.md)已完成：声明范围内的局部论证未发现 Critical Error 或新增 Justification Gap。它核对了不可替换的固定 cut、非队首任务得到服务时的删除/重排、先提交 DATA 再整理队列、固定任务重试、额度与 legacy lease，以及实际提交时刻。三个候选文件的 hash 前后相同。该结论不等于全协议通过；原稿已经明示的条件仍未由审查自动满足。

后续必须按原模型的实际含义选路线。如果同地区仍承担请求保留义务的新旧正确成员受同一个通信保证覆盖，可以继续接入已有的 ownership/hydration 方案；这条路线不需要同时解决历史任务目录和强制调度 MSIG。若只保证同一 AUTH 内通信，则历史 AUTH 路线还必须有旧成员服务和公开任务发现的依据。这些是替代方案各自的依赖，不是要求用户同时批准一组新假设。

## 8. 当前停止点：需要确定原定义，再继续完整协议整合

[集成后的目标审计](../reviews/v3_9/goal_impasse_after_integration.md)已核对最新文件和完成的联合审查，没有发现当前还可不依赖模型域解释就消除请求进展缺口的具体局部修复。此前识别的公开导入、hydration/Ready 接入、历史任务调度、目录边界和 runtime/control 接入均已做成可审查材料；不能仅通过把它们再写成一份条件性正文宣布完成。

需要确认的是 v3.1 原句 `Within region i ... correct-to-correct ... <= Delta_i` 的本来含义：**同一地区、仍有请求保留职责的新旧 AUTH 正确成员，是否也属于这个通信与服务保证的范围？** 原文没有明确排除它们，也没有唯一规定跨 AUTH 与退出 worker 后的参与时间域。该问题已在连续多轮提出，尚无用户回答；自动继续任务的消息不算回答。

因此本轮在完成新的局部工作与审计后，申请将目标标记为 blocked，完整目标仍未完成。原定义得到澄清后，按它支持的路线继续写下一版、补端到端证明并做新的独立审查。这里不宣称所有协议都不可能，也不要求先完成未请求的部署性能测试。


## 9. 原含义确认后：v3.10 接入，v3.11 修补并重审

用户已确认：原区域通信界覆盖同一地区的新旧正确成员；发送请求到 region 意味着地区每个成员最终收到。这只明确原模型含义，Model 本身逐字保留。此前 blocked 所指的通信域问题已解除。

v3.10 把 ownership/hydration 与精确 anchor 的 opening certificate 接入完整协议。首次新审查发现 OC 传播遗漏：H1 收到有效 OC 后可停止重复计算，但没有明确要求转发；若 session 已关闭，H0 也等不到 Ready 转发。另一份报告要求说清 ingress 的正确收件人，并指出 indicator 排版错误。见[v3.10 结论](../reviews/v3_10/verdict_zh.md)。

v3.11 规定：首次验证得到某 K 的合法 OC，先保留完整副本、登记面向全部 target 的发送任务，再取消冗余 preparation 或通知 Ready。实际发送在 Closed/Terminated 后仍继续；每 K 一次首次 multicast，重复输入只续传。验证/转发只要 selected body，不等待非最小 keys 的 bodies。形成证明因此分为“已有正确持有者则传播”和“否则正确成员继续计算并组证”两支。详见[修补说明](../reviews/v3_11/change_summary_zh.md)。

新的完整请求证明从一个持续服务地区的正确实际收件事件开始。固定 cut 的接力在任意有效 HYDRATED 之前把请求送到所有正确目标；正确 offer 因而覆盖它。Pacing 只留下有限早期 slot，每个 slot 的 accepted quota 有限；剩余无穷多个 accepted 首块执行它或一个此前未接受的较小请求。有限势函数不能无限下降，故请求最终进入 G。

三份新的独立审查已核对整条链及原安全、全球 DATA、地区服务和接口边界。它们不是部署时延或可执行模型检查的保证；[本版结论](../reviews/v3_11/verdict_zh.md)列明范围。历史 AUTH 排队和额外调度 MSIG 路线没有采用。
