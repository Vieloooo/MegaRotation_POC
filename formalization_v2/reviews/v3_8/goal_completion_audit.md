# v3.8 整体目标完成度审计

日期：2026-10-19。审计对象是当前 v3.8 正式稿及其后续请求接力研究。只新增本报告，不修改协议、证明、冻结快照或旧审查报告。

**结论：完整完成不受现有证据支持。** v3.2 成稿、多轮编号修改、中文说明和新的独立审查已经交付；v3.8 的条件性安全、全局 DATA 增长、持续合格地区的接受服务及条件性请求定理得到新的审查支持。但一般请求 inclusion 仍未证明。“至少一个 region alive”也只能按正文明确的区域服务与资格范围报告，不能扩写成容许其他已分配 quorum 任意永久消失的定理。

这不是“所有后续工作都必须等待用户”的结论。已有公开完整结果中的请求导入，是无需先选择跨 snapshot 网络含义便可具体推进的一项工作，见第 5 节。它只能补一段证明链，不能被用来宣布完整请求活性已经解决。

## 1. 权威来源和范围

本审计按本任务提供的原目标核对：按修订提纲写 v3.2；用新审查者对抗检查安全性与活性；以新版本保留攻击、修补和证明；持续推进到安全的跨区轮转；证明全局进展和活地区得到服务等性质；提供易读中文差异；不改变 v3.1 假设。**请求 inclusion 是此前已单独跟踪的验收义务，不因目标摘要只举全局进展和地区服务两个例子而消失。** 其已有证据见[版本记录的验收表](../../revisions/rotation_evolution_zh.md)和 v3.2 新活性审查所记录的 T2 请求饥饿问题。

提纲分为[原提纲存档](../../sync_hotstuff_rr_part2_revision_outline_before_2026_10_19_zh.md)与[10 月 19 日更新后的当前提纲](../../sync_hotstuff_rr_part2_v3_2_revision_outline_zh.md)。后者已引用 v3.7 接口，不能倒过来当作 v3.2 当时已经完成一切的证明。本次检查两者的四模块、并发 Handler、固定交接、Ready、六状态、唯一公共更新与收尾责任要求，并核对 v3.2/v3.8 正文。

假设权威是[冻结 Model](../../revisions/v3_1_frozen/model_and_assumptions.tex)、冻结 Part I 与 v3.1 完整源文件。最新正式英文审查对象是[23 文件 manifest](manifest.json)，不是未并入正文的 v3.9 研究候选。采用 proof-verifier 的核对方式：检查声明、依赖和量词；不在本审计中补写一个新协议来代替缺失证明。

## 2. 逐项完成矩阵

“已完成（条件范围）”指该明确范围已有源码、证明及审查证据，不表示无条件部署保证。“未完成”表示仍有实际义务；“不确定”表示原目标的较宽读法不能仅由当前材料裁定。

| 要求 | 来源／范围是否明确 | 状态 | 现有证据与准确边界 |
|---|---|---|---|
| 按提纲交付完整 v3.2 | 明确要求 | 已完成 | v3.2 英中稿、44 页 PDF、快照及三份新审查保存；本审计重新核对原 5 文件 manifest、对应工作文件和 PDF 哈希。成稿不等于该版已通过安全活性验收。 |
| 四模块、三路持续输入、独立程序记录、固定 anchor、Ready/Enter、六 phase、并发与 race | 明确提纲要求 | 已完成（形式规范范围） | v3.8 主稿第 6 节，尤其 1510–1669、1695–1744、1939–2054、2104–2170 行；Protocol 3/4、独立 retirement fragment。当前 P4 已写 `Declare the fixed anchor`；当前提纲末尾仍要求这项修改的文字已经滞后，不能据此认定正文尚未修改。 |
| v3.1 假设保持不变 | 明确且不可放宽 | 已完成（字面）；语义范围须保留 | 冻结检查重新通过 v3.2、3.3、3.4、3.4.1、3.5、3.6、3.7、3.8；v3.8 中文第 2 节 9,080 bytes 一致。`fresh_model_carriage.md`另查实际依赖。字面相等不证明任意后续解释或新候选仍未增加假设。 |
| 新审查者独立对抗检查安全性、活性 | 明确要求 | 已完成（当前正式版本的一轮） | v3.8 三份 fresh 报告针对同一快照，自述仅读审查范围和快照，未读作者笔记或彼此结论；本审计确认报告与记录哈希一致。独立性流程有报告记录，不能由文件哈希单独证明审查者的认知独立。 |
| 发现问题后编号修补、保留攻击与证明 | 明确要求 | 已完成（过程）；继续条件尚未满足 | 版本记录保存 v3.2 至 v3.8 的修改、攻击及审查；后续 ownership 候选经历 v1/v2/v3 修补。后者是研究稿，没有正式 v3.9 全协议交付。 |
| 全球 prefix safety、strict Force cutoff、生命周期及退役安全 | 明确安全目标的具体展开 | 已完成（已有抽象接口与正确性前提下） | 主稿 2473–2578 行、receipt kernel 与 retirement fragment；`fresh_safety.md`未发现其范围内 Critical Error 或 Justification Gap。原 sound validation、密码学、完整消息和 Ready 接口仍是前提；不能外推到任意 native adapter。 |
| 全局 DATA 无限增长 | 明确例举性质 | 已完成（正文定理范围） | `rotation_v3_8_liveness.tex:295–364`；固定 stalled frontier、有限准备、增长的 miss 容忍、有限健康控制与时间戳上界证明；`fresh_progress.md`复查。没有统一初始恢复时限，也不证明每个外部观察者的应用处理 deadline。 |
| 只剩一个合格可服务地区也不被 selector 永久排除 | 明确 singleton 目标的精确定义 | 已完成（限定范围） | 同区续任与条件 cooldown；同上定理明确不需要两个 eligible regions。它不把 frozen progress 中 “another eligible region” 改成新的环境保证，而是新增算法并证明不再需要第二候选来选区。 |
| “至少一个地区 alive”涵盖任意其他区域／历史 quorum 永久失联时的全部活性 | 原话可能有此宽读法；当前模型没有清楚授权该扩展 | 不确定／未证明 | liveness 358–364 行明确排除 permanently missing correct quorums；ON/positive weight 不是服务可用性证明。不能把 physically alive、governance-ON、AUTH 合格、满足 correct-weight/synchrony 的服务实体混为一谈。 |
| 每个持续健康合格地区反复得到真正的接受服务 | 明确地区公平性要求 | 已完成（条件范围） | liveness 366–444 行的 owed round 与 regular cycle 证明，要求该地区最终持续 ON、正权重并继续满足区域 serving premises。贡献是 DATA 进入 G；空 DATA 可以偿还 owed debt。 |
| 任意允许成员更换下，正确请求最终进入 G | 既有单独验收义务；不得删去 | **未完成** | requests 243–301 行只证明 accepted-report-awareness ⇒ inclusion。liveness 446–495 行明确未推出 awareness。空块进展、地区 credit、实际正确 leader 反复进入，都不能替代这个前提。 |
| 中文完整说明、可读差异与未完成边界 | 明确写作要求 | 已完成交付；可读性非形式认证 | v3.8 中文稿、版本演化记录、后续请求研究记录均在，且具体说明每版规则和代价。中文快照哈希已核对。可读性是人工判断；77 页英文及大量历史片段的存在本身不证明“简单清楚”。本审计没有另做逐句文风认证。 |
| 可编译且交付物与审查版本一致 | 必要交付检查，非安全定理 | 已完成 | 本审计重新隔离编译 v3.8：77 页，最终日志无 warning/undefined/overfull/underfull；主稿、各依赖及已交付 PDF 哈希吻合记录。没有重新逐页视觉检查，原记录的渲染验收仍属于此前检查。 |
| 可部署实现、具体 codec、吞吐／CPU benchmark、model checking | 原目标中没有明确要求这些实现交付 | 不列为新增完成门槛 | 现有报告没有这些验证，应如实说明，但不应为阻止形式研究任务完成而临时增加部署工程范围。 |
| “持续迭代直到完整目标完成” | 明确最终条件 | **未完成** | 请求 inclusion 和较宽 alive/service 解释仍未解决。三份条件性通过报告不能作为整体停止条件。 |

## 3. 请求缺口的最短依赖链

当前已经有：

`正确成员收到 x → 在同一已认证 snapshot 内传播并持久保留 x`。

条件性请求定理需要的是：

`无限多个实际进入 G 的 term-first DATA，其真实 REPORT witness 含一个在报告时能选择 x 的正确成员`。

然后有限 predecessor potential 才推出 `x ∈ G`。两者之间还缺一个能实现的传输或服务算法及其证明。此处的量词不能换成“x 最终到过每个 cohort”“有人始终存着 x”“正确人确实反复轮到”或“地区始终贡献 DATA”。

[recurrent-identity 攻击](v39_recurrent_identity_trace.md)进一步排除了仅靠逐身份回归就自动补齐缺口的推理：知情 cohort 每次晚到，leader 提出 x 并 self-vote，Force 在其他人首次 vote 前关闭旧版本，最后只有一份 FINAL；新 cohort 先以合法 EMPTY 报告贡献 DATA，再收到 x。每个正确身份都能反复回来，但没有含 x 的完整 R0。其[新审查](v39_fresh_recurrent_trace_review.md)仅在 fixed-snapshot 通信及 pointwise eventual preparation 的明确范围内认可轨迹。若原服务承诺本来就是每个知情 cohort 必获完整成功窗口，该轨迹不满足那一更强读法。故它是带作用域的 v3.8 缺口证据，**不是所有新协议的不可实现证明**。

后续两条修补路线都没有完成：

- [ownership v3](v39_ownership_candidate_v3.md)的[独立审查](v39_fresh_ownership_v3_review.md)认可有限切片、实际发送、context barrier 和有限例外论证的条件范围；混合 snapshot 通信、source-authentication handler、Ready/REPORT 接入、资源与生命周期接口仍被明确留作条件。
- [历史 AUTH 作用域审计](v39_historical_authority_scope.md)指出 fresh `(beta_new,r_old)` 可与冻结 authority 语义相容，但当前 selector 不授权任意旧 r。还需新调度、stalled-G 证明和旧投票身份的服务覆盖。既不能说旧 AUTH 自动带来可用 quorum，也不能未经论证说所有旧 quorum 都已不在原同步模型内。

## 4. 精确剩余工作与需要澄清的含义

完整目标至少还需要：选定并真正规定一条请求接力或含请求完整结果形成路径；证明它覆盖原请求义务；把算法接入正式 numbered version；重新证明它不会阻塞唯一可服务地区、不会重置公共窗口或破坏 cutoff/retirement；对新完整快照做新的独立安全与活性审查，更新英中稿和差异记录。未整合的候选条件证明不能代替这一步。

以下是原文需要确定的含义，不是本审计建议新增的假设：

1. 冻结 Model 172–179 行的 `within region i` 是否覆盖相同 physical region 的不同 AUTH snapshot 端点之间的正确消息，还是只覆盖同 snapshot？这是 ownership 路线的确切分岔。
2. 同步／正确成员服务的时间范围是否持续覆盖持有请求的历史 voting identities，包括 fresh historical session，还是只覆盖当前服务的 snapshot？这是 historical-AUTH 路线的确切分岔。
3. Model 297–312 行的实际 entered turns、`enough uninterrupted service time` 和“请求到达继续服务的地区”，究竟按 region、snapshot 还是具体知情 identity 量化？尤其是否保证该知情实体某次 Ready 后有足够窗口形成完整结果。
4. 用户的 alive 是当前模型中的 eligible serving region，还是还要求在其他已分配／历史 quorum 任意永久缺失时继续覆盖全部待处理请求？应保留两种结论的边界。

这些并非四个必须同时选择的新协议参数；它们是现有候选分别依赖的范围问题。若能从既有权威定义推出其中一项，应先给出该推导；如果原作者意图仍有多种相容读法，再请用户确认。不能替用户默认一种更强读法，也不能仅因歧义便宣布所有可行算法都不存在。

## 5. 无需上述选择即可推进的具体下一步

**可以正式规定并独立核对“从已经保留的公开原 DATA 对象导入请求”的非阻塞本地动作。** [existing-carrier 研究第 2 节](v39_existing_carrier_design.md)已明确给出实际起点：现有 PublicStore 保留完整 namespace 和 rejected records；公开 R0 的普通 DATA batch 已含原认证请求字节。v3.8 当前并未把每个此类出现位置都变成明确的请求导入事件。

这一工作的具体交付应是一个单独 candidate fragment 和精确引理：完整原 R0 已经公开且含 x、读者取得相应原字节时，本地导入任务鉴别实际请求、按 logical identity 去重并持久保留；旧 DATA 被 cutoff 拒绝不阻止导入；导入不执行旧状态，不授予旧 DATA 进入 G 的权限，REPORT 时仍按精确新 parent 检查 eligibility。需要给出有限历史 cut 的游标、处理顺序和继续派发规则，并证明有限 cut 的导入完成，不能把“后台会处理”当作证明。

该任务可以先留在非阻塞保留／导入路径，不让 Ready 等待它，也不宣称新 CPU、archive 或跨 roster 时限。独立检查其对原 SOURCESTEP 时限和签名规则的影响；如果实现需要额外有界工作，必须将这项接口问题显式留下，不能收费到旧 Δ 中。然后才决定能否并入正式下一版。

这项工作是真实可审查的算法与局部证明，不是泛泛寻找新想法。它消除 `已有公开请求字节 → 明确本地保留` 的遗漏，可能让任一后续路线复用。它**不能**保证一个 pending-only x 首次进入完整 R0，也不能自动证明无限 accepted awareness，因此不消除第 4 节的全局剩余工作。另一个更小的安全文档动作，是将当前提纲末尾已实现的 `Declare the fixed anchor` 要求改为完成状态；它同样不能被包装成活性进展。

## 6. 本审计实际执行的检查

- 对 v3.8 manifest 的 23 个 snapshot 文件及 23 个对应工作文件重算 SHA-256：全部吻合；包括嵌套 baseline 和全部当前 source dependencies。
- 重算三份 fresh 报告、后续研究记录列出的十个文件：全部与各自 verification JSON 吻合。
- 重算正式英文主稿、中文稿、已交付 PDF 与中文 companion：全部吻合。主稿 SHA-256 为 `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574`。
- 重新运行 `revisions/check_frozen_assumptions.py`：八个修订版本的 English Model 原样不变，冻结原文件身份也通过；重新截取中文第 2 节并与 v3.1 比较，9,080 bytes 及 hash 均吻合。
- v3.2 manifest 的五个 snapshot 文件及对应工作文件、原 PDF hash 重新核对通过；该版编译／渲染的其他历史说法仍以旧记录为据。
- 重新在 `/private/tmp/rotation_v38_completion_audit/` 隔离运行 `latexmk -pdf -interaction=nonstopmode -halt-on-error`：退出成功，77 页；最终 `.log` 中上述警告类别为零。初轮缺引用随后已由 latexmk 重跑消除。原 `.aux` 也确认 Protocol 1–6 标签均存在。
- 源码和证明依赖检查是人工审计；没有协议执行器、model checker、native adapter 或传输容量测量。此次没有重复原 77 页的视觉渲染检查。

因此，记录应继续写“已交付并审查的 v3.8 条件性结果；完整目标未完成”，不能写“secure cross-region rotation 已全面完成”，也不能写“除用户回答以外已没有任何有意义的下一步”。
