# v3.11 整体目标完成度审计

日期：2026-10-19。对象：[v3.11 正式源文件](../../sync_hotstuff_rr_part2_rotation_v3_11.tex)、其 26 文件[冻结清单](manifest.json)、三份已完成的新审查，以及原目标要求的历史与中文交付。

**最终结论：原目标在冻结 v3.1 模型、用户已确认的原含义和正文明确的抽象接口范围内完成。** 安全、全球 DATA 进展、singleton 合格地区、重复地区接受服务及 retained request inclusion 都有完整规则、证明和本轮独立审查支持；最后的构建、中文状态与版本交付也已实际核对。本审计未发现该授权范围内仍需完成的具体协议、证明或交付事项。这不是脱离所列前提的部署或任意失效模型认证。

本报告是要求与证据的综合审计，不是第四份相互隔离的协议审查。我读了三份 fresh 报告、必要历史和当前源文件；不能据此声称自己的认知与前三位审查者隔离。三份 fresh 报告分别记录了只读 scope、manifest 和 snapshot 的隔离过程，并核对同一文件集前后不变。本审计没有修改协议、证明、快照或旧审查报告，也没有执行 goal 工具。

## 1. 验收范围和权威

任务给出的原目标是：按修订提纲写 v3.2；以新的审查者对抗检查安全性和活性；发现问题则保留编号补丁与证明，持续迭代到安全的跨区轮转；在原假设下证明至少一个 alive region 能推动进展，活地区得到服务；保留可读中文差异；不改变 v3.1 假设。**请求最终纳入 G 是一直保留的活性验收项，不能以空 DATA 增长、地区 credit 或一个局部引理替代。** 原要求的记录见[版本验收表](../../revisions/rotation_evolution_zh.md)及 v3.2 的 T2 请求饥饿发现。

提纲权威包括[原修订提纲存档](../../sync_hotstuff_rr_part2_revision_outline_before_2026_10_19_zh.md)，尤其 §13–15；[后续更新提纲](../../sync_hotstuff_rr_part2_v3_2_revision_outline_zh.md)仅用于确认当前四模块接口，不能倒过来证明早期版本已通过验收。冻结假设以 [v3.1 保存副本](../../revisions/v3_1_frozen/)为准。

本次适用的两个原模型含义来自 [review_scope.md](review_scope.md)，原话分别是：

> 覆盖同一地区的新旧正确成员

> sent to a region means that every party in the region will receive it.

第一句覆盖同一物理地区内不同 AUTH 的正确端点；第二句使继续服务地区的实际完整请求交付包含正确成员。两句均不提供跨区通信界、客户端时限、所有未来 roster 的初始知情、失效机器恢复或 native 工作时限。它们已经得到用户回答，不再列为等待确认的阻塞项。

“alive”依原目标的“在我们的假设下”解读为满足原正确权重、通信及继续服务前提、且有资格被选择的地区。源文件明确证明单个 eligible serving region 的情形；不把仍在 assigned snapshot 中却永久缺失的正确 quorum 当成满足原区域服务模型。后者没有被证明，但也不是本审计临时加入的验收要求。

## 2. 逐项完成矩阵

“已完成（条件范围）”表示有实际算法、证明及本轮 fresh 审查支持；不表示脱离冻结假设的部署保证。“待核验”表示最终交付仍需实际检查。“未证明／范围外”保留真实边界，不自动变成新增任务。

| 原要求或必要验收项 | 状态 | 精确证据与范围 |
|---|---|---|
| 按提纲交付 v3.2 英文、中文、PDF | 已完成 | v3.2 三项文件仍在；本审计重算主稿与 44 页 PDF 的 hash，与其 [build_verification.json](../v3_2/build_verification.json)吻合。该版早期反例与未通过状态保留。 |
| 编号迭代、保留攻击、修补及证明 | 已完成 | v3.2、3.3、3.4、3.4.1、3.5、3.6、3.7、3.8、3.9、3.10、3.11 的英中稿和 PDF 均存在。v3.10 CE-OC、JG-I1、T-I1 保留；v3.11 给出明确修补及新完整审查。最终版本演化和请求研究记录已更新到 v3.11，并将早期未解决状态明确标为历史。 |
| 四模块及原 Part II 结构 | 已完成（形式规范） | main §6.1–6.8，1530–2236：Soft Detector、HealthController、Handler、Ready/Activation Gate；三路持续输入、并发准备、固定 handoff、启动、race 与 graceful close。十节结构和 Protocol 1–6 保留。 |
| 同一套完整 view、六 phase、程序隔离 | 已完成（形式规范与安全论证） | main 91–105、1745–1847、2605–2645：完整 view=(beta,r,g,t)，session=(beta,r)；Waiting/Ready/Active/Frozen/Completed/Terminated；close_requested 不是第七 phase；每 pair 的 votes/locks/timers 与 entry flag 持久隔离。 |
| regular、DATA/DRAIN、MSIG 优先、旧 beta double-check、strict cutoff | 已完成（条件范围） | main 718–775、1850–1917、2240–2302；control fragment；fresh_safety §§2、5、11。Regular 仍继承 terminal DATA，Force 用 pre-payload G；同触发块先 Force 后 payload，旧版本不能回填。 |
| 原提纲 13 项 lifecycle/manager 检查 | 已完成（形式规范与条件安全） | 见本报告 §3 的逐项映射；唯一固定起点、预热无权、一次 Enter、隔离、公共一致、保留 G、分离时钟、退出不伪造完成、职责移交等均有具体规则。 |
| v3.1 假设不变 | 已完成（字面与本轮语义审查范围） | 11 个编号稿的完整 English Model 均通过冻结检查；v3.11 中文 Model 与 v3.1 同为 9080 bytes。fresh_interfaces 区分字面相等与实际接口含义，采用两项已确认解释。 |
| 新的独立对抗 safety/liveness 审查 | 已完成 | [fresh_safety](fresh_safety.md)、[fresh_progress](fresh_progress.md)、[fresh_interfaces](fresh_interfaces.md)均实际完成；同一 26 文件快照，前后 hash 一致；各自指定范围未发现 CE/JG。没有用旧局部审查冒充本轮完整源链审查。 |
| prefix safety、honest FINAL、退休安全 | 已完成（条件范围） | main 2541–2645，receipt_kernel 与 retirement；fresh_safety 完整检验正当 FINAL 来源、原 timer/lock 依赖、phase-independent closure、early/late close、当前 append authority 与严格 cutoff。 |
| 至少一个 eligible serving region 下的全局 DATA 无界增长 | 已完成（原模型范围） | liveness 70–419：有限固定准备→区域正结果→完整原 R0 公开→prepared replay 的增长/关闭替代→递增容忍与 timestamp upper-lead 矛盾。NextLive self-renewal、条件 cooldown 包含 singleton；不循环假设目标 DATA 已接受。 |
| 每个持续健康合格地区反复得到接受服务 | 已完成（条件范围） | liveness 421–499：最终持续 ON、正 authority weight 并满足 serving premises 的地区，借 owed rounds 与严格递增 regular selection references 反复贡献 DATA 到 G。并非仅被选择或区域投机提交。 |
| retained requests 在允许成员变化下最终进入 G | 已完成（条件范围） | liveness 501–677 与 ownership、input_selection、requests；本报告 §4 逐箭头映射。原 ingress→实际正确 owner→有限例外→无限 covered accepted first DATA→有限势函数 inclusion；不再留下 future-roster-awareness 假设。持续 admissibility 与单 DATA fit 仍是显式原请求范围。 |
| 无新增外部 availability/deadline；不偷换公开输入 | 已完成（抽象接口范围） | 完整原 R0 仍为 Publish Layer 输入；仅新增区域 OC/advertisement/REPORT sidecars 可 erasure。历史 AUTH/bytes、native state、pool selection 均实际准备且无统一时限。fresh_interfaces §§1–11 检查 endpoint、bytes、handler 与 publication 限制。 |
| 中文完整阅读版、最新差异和状态一致 | 已完成 | 当前中文前置状态与 §9.11 已反映三份 fresh 审查；完整 58 个编号标题保留，Model 与 v3.1 相同。companion/verification.json 记录最终中文 hash；中文、verdict、版本演化和请求研究记录的本地链接重新核验无缺失。初检历史 hash 由最终记录明确接续。 |
| 当前稿编译、PDF/render 与快照一致 | 已完成 | 最终 latexmk/stdout、92 页 PDF 与交付 PDF hash 实际吻合；最终 .log 无 LaTeX/package warning、undefined references/citations、overfull/underfull。aux 有 Protocol 1–6。92 页 render 与 6 contact sheets 存在；主任务记录全部 contact sheets 及 45/70/89 页细看。本审计复核机械证据，没有冒称再次逐页目视。 |
| 任意其他 assigned quorum 永久失联、末位正确 owner 实体消失时仍保证一切活性 | 未证明／范围外 | 原正确通信与 serving premises 不覆盖；没有为完成 formal scope 而把此限制隐藏，也不把它增加为新研究目标。 |
| 具体 codec/native adapter 实现、CPU/吞吐 benchmark、executable model checking | 未交付／范围外 | 本次任务是形式协议及人工对抗审查。没有这些证据，不作相应声明；也不因此否定已具备证据的形式结论。 |
| 整体目标完成 | **已完成（原形式模型范围）** | 本矩阵的协议、证明、fresh review、冻结、历史与最终交付项均已有实际证据。当前没有具体未完成项；范围外的永久缺 quorum、具体 adapter/benchmark/model checker 不被伪装成已证明，也不扩为新的完成门槛。 |

## 3. 原提纲的 13 项性质没有在请求修补中丢失

| 原检查项 | 当前规则／证明位置 |
|---|---|
| 1 唯一启动起点；12 不改写旧 anchor 身份 | main 1800–1811、1868–1917、1960–2030；固定 target、authority、原 producing pair/hash/height；VerifyHandoffHistory 拒绝覆盖固定字段。 |
| 2 预热无签名权限；3 初始 Enter 最多一次 | main 1759–1770、2032–2083、2099–2111；STATE_CANDIDATE 也不授 entry/vote/timer；initial flag 在 dispatch 前原子持久化。 |
| 4 正确路由／隔离 | main 1621–1638、1800–1811、2605–2645；MaterialAvailable 绑定原 target/anchor，不能换目标。 |
| 5 同历史同正式 handoff；6 G 不回滚 | main 2547–2602；公共 canonical subposition 归纳，local phases 不增添 G mutation。 |
| 7 两种时钟不混用 | main 2099–2145；Enter 启动区域 timers，Waiting/preparation 不推迟公共窗口或 reset deadline。 |
| 8 退出不伪造 Completion/Quit；13 明确收尾条件 | main 2200–2236，retirement 24–242、248–406；Closed gates 新动作，既有 timers 正常结算；c_v+4Delta 只是必要 drain guard，不单独保证可释放。 |
| 9 退出不丢数据或责任 | retirement 122–179；完整证据、R0 offers/retries、owner journals/cuts、OC relay bytes/cursor、receive routes 都先实际移交，再释放 worker。 |
| 10 detector 可重算 | main 1588–1609 与 control；完整 namespace/controls 缺失会 wait，不记零；只计实际进入 G 的 DATA。 |
| 11 health control 不改写 finality | health fragment、control 222–325；有限 health policy 管签发与未来生效，不能回滚 G 或选择旧 retained prefix；MSIG/AUTO 的唯一公共转换保留。 |

以上证据对应原提纲所要求的形式规则及安全论证；图示辅助阅读，不代替这些证明。当前共享旧版本片段是有意保留的依赖，不代表漏建对应的新文件。

## 4. 不能缩减的端到端请求证明链

| 箭头 | 实际规则、结论与 fresh 检查 |
|---|---|
| 原 ingress → 一个正确成员完整收到 x | 已确认的 region-delivery 含义，加原 fault bound；ownership 91–108 在 acknowledgement 前持久写 bytes/journal；liveness 598–619。不是 send attempt、raw wrapper 或 Byzantine-only receipt。 |
| 正确 receipt → 成员变化后仍可找 owner | ownership 14–56、419–444：已知 0..r AUTH lineage、有限 directory、pending 原 owner 的持续服务与实际 logical-owner handoff；不枚举未来 AUTH，不靠 arbitrary custodian promise。 |
| 找到 owner → 实际取得并向所有 target 传完 bytes | ownership 130–299：context actual send/barrier，immutable count/length/hash/fragment budgets，Byzantine timeout，actual all-target send，share 与 certificate collection/relay。 |
| 每一张 HYDRATED → 每个正确 target 的 selection cut 含 x | ownership 324–417 与 input_selection 88–124、219–240：选任意 valid certificate 的正确 signer，其发送先于 share；正确 offer 在 hydration 后 freeze raw/canonical cut；不推 honest intersection。 |
| 已准备 state → 所有正确 target 最终得到 OC | input_selection 62–86、152–217：STATE_CANDIDATE 先于 Ready；若某正确 holder 已有 OC 则先持久 multicast 再取消；否则所有正确 job 继续并形成第一张；Closed/Terminated 仍传，且不等待 nonselected bodies。这直接修复 v3.10 CE-OC。 |
| OC → 每个 accepted session-first DATA 执行选中请求 | requests 44–165；NEWVIEW/REPORT 签明 exact initializer choice 或已履行的 ancestry；回到 opening 重施规则；DRAIN/ANCHOR 不能充当 first DATA；真实 correct FINAL 证明过去 full validation，无公开 alternate-witness 偷换。 |
| 无限全局进展 → 无限 covered accepted first opportunities | liveness 513–640：有限 region cycle 得 recurring region，正 pacing 使早于 receipt 的 slots 有限；再去掉 r<k。每 slot 有限 quota，positive output 才推进且 strict cutoff 阻止再用；late shares、equal query time、zero-output retries 均覆盖。 |
| 无限 covered opportunities → x 进入 G | liveness 641–666：原持续 admission 与 singleton fit 使 x eligible；任意 valid OC 选真实 y 且 key(y)<=key(x)。长度优先 key 的 bounded-length canonical identities 有限；每个接受首块花费一个从未入 G 的身份，Phi_x 严格下降，故不能永久绕过 x。 |

[fresh_progress.md](fresh_progress.md) §§1–11 实际检查了这条完整链；[fresh_interfaces.md](fresh_interfaces.md)核对 recipient、service、timing 与量词边界；[fresh_safety.md](fresh_safety.md)核对使首块义务可传到公共接受端的历史验证、FINAL 与退休链。这里没有把“空块无限增长”当作请求证明。

## 5. 同一受审版本与机械证据

本审计独立运行 `revisions/check_frozen_assumptions.py`：v3.2 至 v3.11 共 11 个编号版本（包含 v3.4.1）通过；脚本还核对冻结原文件身份。重新计算 snapshot 26/26 及 live counterparts 26/26：零不匹配。递归 TeX input closure 共 17 个文件，全部包含于本轮 manifest。冻结 baseline 在 live 比较中映射到 `revisions/v3_1_frozen/`。

| 对象 | SHA-256 |
|---|---|
| main v3.11 | `de2e36905fef1e2de7bcc53f0cd8f4a95eca92e4561d347b33bc260e7dc58bef` |
| 整个 English Model | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |
| Chinese Model，9080 bytes | `db9f1d7de0a6c9896b436f476fef2bc37148ec4684118ed793523f8930f6b4c0` |
| manifest.json | `518531648c4ca8421cd315466183360290e68459ba503f20ae3a06e03545d103` |
| review_scope.md | `e8762588e9887b50b9949aadf614e681377b15975550792be95fe846de739284` |
| fresh_safety.md | `9bd308094d2152343c5e677411396b5301bd824ccc6d6c3692c047ed4992ad58` |
| fresh_progress.md | `ebe7538ce127120172598778ed391ba6b8c5af622b3f8d3c36e2728528763b32` |
| fresh_interfaces.md | `61c0855380dc25fada65506147e9a9b2939aa9edc10d2409e4482c12c3862f58` |
| v3.2 历史 main | `956b733d9c30bb6aaed3a2b5a7e67f3628e620392b958ed7471455243108dc1d` |
| v3.2 历史 PDF | `e7f7fb835e0a5083ec6128883a3a1b165f49dc759bdf0f994cf04f10f9c147e8` |

报告中的安全与活性证据是人工逻辑审查；以上 byte/hash/closure 检查仅固定对象身份。编译不证明安全，hash 也不能单独证明审查者认知隔离。

## 6. 最终交付核验与结论

收到主任务的最终交付通知后，本审计重新读取实际 [build_verification.json](build_verification.json)、[verdict_zh.md](verdict_zh.md)、[companion/verification.json](companion/verification.json)、当前中文状态及两个版本记录，并独立完成以下核验：

1. v3.11 main、中文稿、PDF 与构建记录的 SHA-256 均吻合；三份 fresh 报告仍为 §5 中的精确 hash。再次核对 26/26 snapshot 与 26/26 live counterparts，零不匹配。
2. `/private/tmp/rotation_v311_build.stdout.log` 记载 latexmk 成功结束且目标 up-to-date；最终 `.log` 记载 92 页、1,035,953 bytes，实际编译 PDF 与正式交付 PDF 相同。最终日志没有 LaTeX/package warning、undefined references/citations、Overfull 或 Underfull；`rerunfilecheck` 的普通 package/info 行不是未解决的 rerun warning。最终 `.aux` 的六个 protocol 标签对应 1–6。
3. 构建目录实际存在 92 个 page renders 和 6 个 contact sheets。主任务记录已看全部 contact sheets，并细看第 45、70、89 页；本审计核对这些证据和产物，未另称执行了一次新的全页视觉检查。
4. 中文前置状态和 §9.11 明确三份审查完成；请求收件与 BFT FINAL receipt 的措辞已区分。中文第 2 节再次与 v3.1 比较，9080 bytes 完全相同；最终 companion 指纹与实际全文一致。当前中文、verdict、rotation_evolution_zh 和 request_handoff_progress_zh 的本地链接均存在。
5. 最新版本记录已写 v3.9→v3.10→v3.11 的实际演变、保留 CE-OC/JG-I1/T-I1，并把 v3.10 记作失败候选。原历史 AUTH 调度研究明确未采用，当前采用实际 owner 接力；没有通过丢掉原请求义务来完成任务。当前提纲亦链接 v3.11，并把 Ready 的 OC 接入写明。

| 最终交付／复核对象 | SHA-256 |
|---|---|
| v3.11 中文全文 | `ddd544d797b31d626d816f399425800523e9b140f30aa96e9972d77ab67cec63` |
| v3.11 正式 PDF，92 页 | `9daf309f2c3ddb474566941dd916d47566e8ba3a179d89168ecdbfc2047659c6` |
| 最终编译 .log | `9b5b260fed93a270cb3c32b5f3796fe805a43fe22e24634063821f2262b2a3e3` |
| companion/verification.json | `17f72c7aea4f9f09bda3d7b517241d736fa9d2cee328bd7d6f292cda1509c624` |
| verdict_zh.md | `b4e47283bb79d23449b28edf8fb157620101d224f72e2fa58274d15d9f188dbe` |
| rotation_evolution_zh.md | `28453907e10dd06c61c3381869b2c9bf3be7b66edd0e6ef97a79f0a29aca9d60` |
| request_handoff_progress_zh.md | `9c4b0d43032e238acefc20f2248cb35a90ef846d3ae5923007e8151c895f5aee` |

构建记录在本审计开始时把 `goal_status` 写成“等待最终审计”；它只是有意保留的流程状态，不是协议或构建失败。本报告完成后可据此同步该状态。这里不把更新状态本身当作新的证明证据。

**最终完成判定：可以在原形式任务范围内标记 complete。** 依据是完整要求矩阵已逐项有证据、三份 fresh 审查覆盖同一完整源链、retained request inclusion 未被省略，以及最终交付与受审版本吻合；不是因为审查数量够多或没有再找到一个局部错误。应对用户报告“冻结模型及已确认含义下的形式安全与完整活性论证完成”，并保留正确服务／admission／抽象完整消息接口的实质前提。没有剩余的已识别必做工作；未完成的部署实现、性能测量及模型外任意故障容忍不属于此次授权目标。
