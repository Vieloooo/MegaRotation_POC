# v9 协议安全审查（v0.2）：核心反例与待决问题

日期：2026-11-03。上一版：[v0.1](security_review_zh_v0_1.md)。对象：本目录 00–09 的 v0.1 规格；[校准流程 v0.2](10_pipelines_zh_v0_2.md) 是说明性实例，不是新增的协议规则。

## 0. 本版变化

- **v0.1 的条目原样保留。** §1–§11（阅读方式、校准边界、S1–S9）照录 v0.1。受第二轮结果影响的 v0.1 条目，只在 §13 中加“v0.2 注”，不改原文。
- **第二轮发现。** 第二轮四个子代理（共识与交接、GFR、认证与派生、恢复与治理）在读取 v0.1 之后继续检查。主代理逐条对照所引条文核对，结果列为 S10–S27（§12）。与 v0.1 重叠的部分只记录新增内容，并注明关系。
- **新增章节。** §14 汇总两条跨条目的修复建议；§15 列出流程 v0.2 中与新发现冲突的表述（勘误，未改动流程原文）；§16 给出覆盖全部条目的人工检查顺序；附录 A 是条目来源与合并对照。
- **基线。** v0.1 记录哈希之后，[00 §1.2](00_arch_zh_v0_1.md) 删除了两处：“因此合约状态和事件列表由这个前缀唯一确定”一句，以及解释“互为前缀”的一段。01–09 未变。本版的核对基于当前文件，哈希见 [v0.2 基线](checks/security_review_v0_2_baseline.json)。这处删除不改变本版任何结论；不过 S1 与 G 的确定性论证都用到“同一 finalized L1 前缀给出同一合约状态”，请确认删除不是为了放弃这一性质。

本报告不把“候选区域结果最终没被选中”“RESUME 替换尚未原样派生的 G 后缀”本身判为 BFT 安全漏洞；这两者在当前设计中允许。真正需要优先检查的是：同一公开历史是否有唯一解释，来源证明是否足以支持下一模块的结论，以及各条活性前提能否同时实现。（本段照录 v0.1。）

## 1. 阅读方式与基线

每条发现给出触发能力、相关规则、最小轨迹、影响范围、修复选项和人工检查项。**高优先级**表示可能破坏公开解释的一致性、跨模块保证或造成长期停顿；**中优先级**表示局部活性、形式化缺口或需要显式限定的承诺。这是修订优先级，不是已部署代码漏洞的 CVSS 评分。

状态采用：**确认**（文本可直接推出反例或缺口）、**条件性**（明确列出的条件下成立，尚缺实现证据）、**不构成模型内漏洞**（属于已允许语义或违反明确假设）。缺少转移定义时，只能确认“当前规格不能唯一决定结果”，不能据此声称已有实现真的分叉。

00–09 与旧流程 v0.1 的文件 SHA-256 保存在 [审查基线](checks/security_review_baseline.json)。本轮没有将修复选项写回这些协议规则。原生对照仅使用仓库 `megaeth_specs` 中实际读取的文件，不据此推定部署版本。

## 2. 校准中已确认的边界

[流程 v0.2 §2–§9](10_pipelines_zh_v0_2.md) 已修正以下会影响安全判断的例子：

1. **账本位置与真实时间分开。** Espresso 的 timestamp 可以早于最终可读的真实时刻；RESUME 的排序键不等于 Ready 或首块产出时刻。19.2 min 是本例的 L1 finalized＋读取假设，不能由 Ethereum 的 slot/epoch 长度推出无条件上界。
2. **三条时限同时检查。** 区域结果先受 session 公开截止约束，再受逐高度 Expire 约束；原生期限还受具体 origin、channel 完成位置和实际派生父 epoch 约束。不能把 10 h 当成每份区域结果都可等待的时长。
3. **原生窗口边界。** `epoch_num + sequence_window_size < inclusion_block_number` 才触发超窗；等号本身不超窗。缺失高度 fallback 使用实际派生状态，不使用一份未被采用的区域 block 所声称的 origin。
4. **接纳不等于执行。** 未 finalized 的接纳收据尚不进入本文的 finalized 接纳视图。已接纳的迟到批次也可能被派生丢弃；cursor height 与派生内容一致性是两件事。
5. **父链分叉不表示后续批次全部丢弃。** 本地原生 certified derivation 忽略批次 parent_hash，并在实际派生父状态上继续判断后续内容；按旧父状态产生的结果不一定原样执行，但不能仅因 parent_hash 不同就断言全部丢弃。
6. **恢复工作量是切面相关的。** 填充数为 $\max\{0,h_{\rm res}-h_D^-\}$，其中 $h_D^-$ 是处理相应 override 前的派生末端。44 700 仅对应稿中选定的切面；48 s 内完成需要约 931.25 blocks/s，5 min 内完成需要 149 blocks/s，均未实测。

数值复算脚本只验证这些算术关系；它不是协议实现或形式验证器。下面涉及设计的修改均列为建议，等你决定。

## 3. S1：预读 publication 与重放位置之间缺少显式边界

**高优先级；规格缺口确认，分歧轨迹取决于一种待排除的实现解释；置信度中。** 由 GFR 子代理提出，主代理核对接纳条件后保留。

位置：[09 §3.2](09_finality_rules_zh_v0_1.md) 第 102 行将 publication 索引定义在“已完整读取的公开历史”上；[09 §4.4](09_finality_rules_zh_v0_1.md) 第 263–274 行列出接纳条件，却没有显式要求 `first_complete_q` 不晚于**当前已处理**的 Espresso height。读取缓存可以比 pc 更远；这两个前缀必须区分。

取小型构造参数：$\mathit{ts}^*(h)=h$、$W_{\rm exp}=10$，当前 session 窗口为 [91,100]，$T_{\rm grace}=10$，所以公开截止为 111；$G$ 到 94。

| 相同公开日志 | 读者甲：只使用已处理前缀 | 读者乙：预先索引已读长前缀 |
| --- | --- | --- |
| $J_q$ 的键 100，消息为空 | 无扩展 | 已下载 $J_{q+1}$ 中的完整 Res(95) |
| 处理 $J_q$ | $G$ 留在 94 | 以当前键 100 检查 `100 < 95+10`；以首次公开键 106 检查 `106 < 111`，两项分别通过，提前追加 95 |
| 下一个 $J_{q+1}$ 的键 106，Res(95) 首次完整公开 | 先在键 105 执行 Expire(95)，HALT 于 94 | 先在键 106 执行 Expire(96)，HALT 于 95 |

无需伪造区域证明；问题在于乙提前使用未来事件里的真对象。如果“按事件顺序处理”已被作者定义为禁止这种索引方式，乙就不合法；当前接口仍应把这个前提写出，避免消费者自由补全。

**建议。** 保留预读缓存，但另设已处理的 Espresso frontier；在处理 $J_q$ 时只允许使用截至 $q$ 已完整公开的结果。非 Espresso 事件也应读取该 frontier，而非下载缓存末端。用 height/message position 限制，不能只用 timestamp，因为 Espresso 时间戳可以相等。

**人工确认：** [ ] 明确索引只随事件更新；或 [ ] 明确缓存与可接纳 frontier 分离。

## 4. S2：Channel 超时后，认证与原生接手入口的 prev 不一致

**高优先级；跨规格接口冲突确认；置信度高。** 由认证与派生子代理提出。适用前提是 v9 确实继承当前仓库的原生 channel timeout 规则；未核验部署适配层。

位置：[06 CertifyBatch](06_certifier_zh_v0_1.md) 第 97 行要求 prev 等于读取的当前 cursor；[07 SubmitBatch](07_batch_inbox_zh_v0_1.md) 第 141 行也写精确相等。原生 [batch-inbox Acceptance](../megaeth_specs/specs/block-certification/batch-inbox.md#acceptance) 第 239–251 行在 channel 超时后**临时计算** `expected.frames_digest = 0`；第 430 行要求接手者用清零的 prev 发起新 channel，未要求先把存储 cursor 清零。

最小轨迹：L1 height $O$ 接纳未闭合 channel，存储 cursor 为 $c=(h,o,f,source)$，$f\ne0$；之后主 Batcher 停止。在 $O+51$（本例 timeout 为 50 L1 blocks）且备选提交权限已开放时，诚实接手者尝试开始 frame 0：

- 传入 $c_0=(h,o,0,source)$，原生 Inbox 会接受其连续性，但 v9 Certifier 的 `prev == stored cursor` 失败。
- 传入 $c$，Certifier 可通过第一项，但原生 Inbox 的 `prev == expected` 失败。

所有服务此时都可在线，正常接手路径仍没有一份双方接受的 prev。RESUME 可以另走治理恢复，但不能代替声称继承的自动 timeout 接手机制。这里没有反驳以 A-DELIVER 为前提的 BAT-4；恰恰需要解释所用实现如何满足 A-DELIVER。

**建议。** 区分 stored cursor 与按已验证 L1 height 计算的 effective cursor，将 channel_open_block、timeout 和清零规则纳入共同接口；认证与合约应核对同一份有效 cursor，同时处理认证后、交易收录前发生 timeout 的竞态。另一选择是定义显式链上 reset 转移。两者均是设计选择。

**人工确认：** [ ] v9 继承原生 timeout 并补适配；或 [ ] v9 使用不同 reset 语义并单独证明接手活性。

## 5. S3：CERT-4 在 CursorSet 后缺少归纳起点

**中优先级；性质按原量词不成立；置信度高。** 由认证与派生子代理提出，主代理复核。无须腐化任何角色。

位置：[06 CERT-4](06_certifier_zh_v0_1.md) 第 130–136 行；[07 接纳视图](07_batch_inbox_zh_v0_1.md) 第 171 行。

1. 旧 $G$ 到 $h$；合法 RESUME 的目标为 $N>h$。
2. L1 上 CursorSet 已 finalized，派生已取得相应切面，故读取者按规则令 $G'=D_L[0:N]$。
3. 该 CursorSet 之后尚无批次证书；GFR 尚缺 RESUME 所需 watermark，停在处理该 RESUME 之前的位置 $x$。
4. CERT-4 要求 $x$ 不早于“该区间每个证书的 gpos”，且这些 gpos 与 $x$ 之间没有新 RESUME。证书集合为空，这些条件没有给出有效下界；但 $G'$ 比 $G_x$ 长，$G'\npreceq G_x$。

这是两种重放切面没有对齐，**没有证明错误批次被 Inbox 接纳**。CERT-3 的旧 prev 拒绝机制不受此反例影响。

**建议。** 以区间起始 CursorSet 对应的 RESUME 重放位置 $r_0$ 为归纳基；要求 $x\succeq r_0$，并晚于本区间所有证书的 gpos，且 $x$ 尚未处理下一次 RESUME。Genesis 段使用 genesis 基点。对齐后再表述前缀关系。

**人工确认：** [ ] 接受加入 $r_0$ 下界；[ ] 所有跨层前缀陈述都标明切面。

## 6. S4：治理“仅影响活性”的声明不足以覆盖所有 RESUME 输入

**中优先级；信任范围与状态转换的缺口确认；置信度高。** 认证与派生、GFR 子代理分别给出以下两个反例。它们使用合约接纳、但违反链外治理规程的输入；没有证明遵守完整规程的治理会产生它们。

[01 §2](01_l1_authority_zh_v0_1.md) 第 22 行称治理诚实只用于恢复活性；[01 SubmitResume](01_l1_authority_zh_v0_1.md) 第 258 行只检查目标高于 cursor，较强的目标与时间条件放在第 260–271 行的链外规程。需要决定安全结论是否也依赖这份规程。

### 6.1 低目标使 G 截短到已派生共同前缀之内

设 cursor.height=0，而 $G$ 与 $D$ 的前缀都已到 2。这个状态可在没有批次接纳、区域普通交易为空、原生 fallback 与区域 origin 对齐的历史中出现。治理签署目标 $N=1$，满足合约的 `1 > 0`。原生派生保留已建立的前缀到 2；但 [09 §4.6](09_finality_rules_zh_v0_1.md) 第 323 行执行 $G\leftarrow D_L[0:1]$，从 $G$ 删除已在 $D$ 中的 block 2。

这与第 326 行“共同原生前缀保留”“旧 G 是 D_L 的前缀时不撤销任何块”不一致，也超出 [09 §7](09_finality_rules_zh_v0_1.md) 将替换描述为未派生后缀的范围。**D 没有回滚**；问题是对 G 的承诺过强。

### 6.2 新下一高度的 Expire 被生成在 pc 之前

取 $\mathit{ts}_{L1}(\ell)=\ell$、$\mathit{ts}^*(h)=h$、$M_a=2$、$W_{\rm exp}=10$、$TB=10$、$T_{\rm grace}=10$。旧 $G$ 与 cursor 到 89，在键 100 处理 Expire(90) 后 HALT。L1 height 100 收录目标 90 的 RESUME，在键 102 执行后，新的下一高度 91 的 Expire 键为 101，已落在 pc 之前。新窗口 [91,100] 的 SessionEnd 键却是 111，所以第 332 行只针对 SessionEnd 的“不能启动”规则没有定义如何处理这个倒序 Expire。

可以立即 HALT、确定性拒绝该安排，或定义另一种转移；现稿没有选定。不能把新事件插回已经完成的历史，也不能无说明地忽略过期。

**建议。** 先写清治理安全边界。若安全结论依赖规程，需明确纳入前提；若希望对任意授权输入都有确定语义，需为低目标、过远未来目标、实际派生已越过目标，以及新事件键早于 pc 等情况给出一致的处理。不能仅修改 GFR 的目标而不同时协调 Inbox cursor 和后继窗口。

**人工确认：** [ ] 治理诚实也是 G 保留范围的安全前提；或 [ ] 所有授权控制都必须由状态机守卫限定其影响。

## 7. S5：D 不相容按前驱 cid 传播，会误杀正确后继

**高优先级；模型内生命周期反例确认；置信度高。** 由区域共识子代理提出。没有违反 BFT-1，问题出在 Manager 的关闭范围。

位置：[03 ApplyLifecycle](03_session_manager_zh_v0_1.md) 第 226 行关闭不相容 cid 及以它的决定链为 anchor 来源的后继；[03 D 判死](03_session_manager_zh_v0_1.md) 第 275 行把范围写成该决定链中的“某个 block”，没有要求后继继承实际冲突部分。

设前驱窗口 [1,100]，后继窗口 [101,200]（高度可整体平移到合法时间范围）：

1. 前驱合法决定链含 $B_{98}\prec B_{100}$，其中 $B_{100}$ 有实际执行的普通交易。$B_{98}$ 及时进入 G，$B_{100}$ 的完整证明直到前驱公开截止之后才被当前 Manager 取得；这种晚取得没有违反 BFT-7。
2. GFR 正确选定 $B_{98}$ 为 regular anchor。后继 $q$ 从它决定 $B_{101}$，将 99、100 展开成 padding；这些内容及时交付并成为 D 的前缀。后继仍在自己的窗口内运行。
3. Manager 后来验证迟到的 $B_{100}$，按 [03 ObserveRegional](03_session_manager_zh_v0_1.md) 第 178 行更新前驱区域视图。下一次比较 D 时，发现前驱在 height 100 的 hash 与 D[100] 不同。
4. 规则产生针对前驱 cid 的 IncompatibilityFact，并因 $q$ 的 anchor $B_{98}$ 来自该 cid 的决定链而关闭 $q$。

但 $q$ **没有继承 $B_{100}$**，而且自己的前缀已经进入 D。正确的规则应关闭从冲突 block 或其后代接续的候选，而不是前驱 cid 的所有后继。这条轨迹可以由同一区域连续承担职责的合法配置实现，使足够权重的 Manager 都保有前驱与后继状态；无需超出 Byzantine 权重限制。后继已 Close 不可恢复，可能失去整个剩余生产窗口。

这也说明 [流程 v0.2 §3.4](10_pipelines_zh_v0_2.md) 的迟到证书不能仅作为“丢一点预确认”的例子处理；还要防止旧实例的迟到结果触发过宽的 Kill。

**建议。** 对每个后继重新构造属于它自身的 IncompatibilityFact。若采用传播优化，必须证明后继的已验证 ancestry 包含冲突高度的那个候选 block；在已知同链关系时，anchor.height 不小于冲突高度只是必要检查的一部分，不能以高度替代 hash/ancestry。

**人工确认：** [ ] Kill 沿实际冲突前缀传播，不沿 cid 来源关系整体传播。

## 8. S6：固定恢复目标依赖尚未确定的 L1 收录时间

**中高优先级；治理规程可实现性缺口确认；置信度高。** 由治理与恢复子代理提出。

位置：[01 SubmitResume](01_l1_authority_zh_v0_1.md) 第 254–271 行。治理签署时固定 $h_{\rm res}$，但规程用未来实际收录 block $L$ 的 timestamp 检查：

$$
\mathit{ts}_{L1}(L)+T_{\rm prep}
\le \mathit{ts}^*(h_{\rm res})
\le \mathit{ts}_{L1}(L)+T_{\rm drift}-\epsilon.
$$

如果可发生的收录 timestamp 位于 $[a,b]$，同一个预签目标要对所有可能收录位置都合法，必须存在

$$
b+T_{\rm prep}\le\mathit{ts}^*(h_{\rm res})\le a+T_{\rm drift}-\epsilon,
$$

也就是 $b-a\le T_{\rm drift}-\epsilon-T_{\rm prep}$。示例右侧只有 3 min。

治理于 00:00 提交目标 timestamp 00:25:12，预计收录在 00:00:12；模型却允许它到 00:10 才收录，并在 00:19:12 前 finalized，仍满足示例的提交到 finalized 预算。此时目标只比实际收录时间晚 15 min 12 s，低于 25 min 的准备要求；RESUME 的键变为 00:30，比目标首块 slot 更晚。合约只检查 cursor 单调，仍可接纳。

这是“发送者无法在当前前提下保证自己的规程”的缺口，不是满足全部规程后的安全攻击。晚启动也不必然永远无法恢复，可能跳到当前 slot 并 padding；但原先的首块恢复时间保证不能成立。

**建议。** 治理授权一个相对时间偏移或有限目标选择规则，由合约根据实际收录时间确定并记录目标；或签署有效收录时间区间，合约超界拒绝，再重新提交。若仍保留固定目标且无守卫，需为目标失败定义完整的恢复路径。

**人工确认：** [ ] 目标在执行时计算；或 [ ] 签署区间＋链上时间检查；或 [ ] 接受失败风险并收窄时限结论。

## 9. S7：每条 RESUME 都满足列出的条件，仍可能永远没有普通块

**中高优先级；条件性恢复活性缺口；置信度中高。** 由治理与恢复子代理提出；是否被“适当控制”一词隐含排除，需作者明确。

位置：[01 治理规程](01_l1_authority_zh_v0_1.md) 第 260–271 行，以及 [09 RESUME](09_finality_rules_zh_v0_1.md) 第 319–327 行。

初始已有紧急状态，此后所有服务正常。选一条允许的执行，使每次控制交易在 1 min 内 finalized。治理每隔 4 min 提交新 RESUME，第 $i$ 条收录时间为 $t_i$，目标 timestamp 为 $t_i+25$ min：

- 目标严格增加，超过旧 cursor、G 与当时 D；上一条 CursorSet 已 finalized，而 GFR 尚未到公告键，使 D 越过当前生产分支，因此“紧急”表面条件仍可成立。
- 第 $i$ 条在键 $t_i+20$ min 开出安排，首个可生产 slot 在约 $t_i+25$ min。
- 第 $i+1$ 条在键 $t_i+24$ min 关闭它。每个 program 都在开始前被下一次 RESUME 关闭，永远没有新普通块。

这不是对任意恶意治理也要求活性的主张；它说明逐请求条件不足以形式化“诚实治理恢复系统”。BFT-5 要求持续运行同一 cid 的区间，反复控制恰好破坏该前提。

**建议。** 恢复活性显式要求：最终存在一段足够长的、没有相冲突 Force/RESUME 的运行区间；同时给治理规程定义未完成恢复的串行化或替代条件，不把自身尚未重放的 CursorSet 超前误当成新故障。

**人工确认：** [ ] 补控制静默区间；[ ] 明确何时允许替换一条尚未完成的 RESUME。

## 10. S8：准备历史 anchor 不能从最新 D 状态倒着 Apply

**中优先级；操作规程缺口确认；置信度中。** 由治理与恢复子代理提出。

[03 PrepareCandidate](03_session_manager_zh_v0_1.md) 第 194 行指定从 D 的已验证末端 checkpoint 逐高度 Apply 到 $A.h$；但 ObservePublic 先推进 D，再重放 GFR。一个追赶节点处理历史 Force 时，完全可能已有 $h_D>A.h$。两条链即使相容，也不能把后代的执行状态通过向前 Apply 还原成祖先状态。

ReadyState 与 StateMaterial 的一般定义已经允许历史 checkpoint，因此不能据此断言实现必定死锁；具体准备规则应选择处于 anchor 已验证祖先链上的 checkpoint $C$，要求 $C.h\le A.h$，或直接取得该 anchor 的精确状态与验证依据。历史输入的取得与重放应计入准备预算。RESUME 继续使用精确 ResumeCut.anchor_state。

**人工确认：** [ ] 将“最新末端”改为“已验证祖先 checkpoint 或精确 anchor 状态”。

## 11. S9：跨 session 的密钥泄露计入哪个 Byzantine 集合

**待明确的模型契约；未确认攻击；不能据此宣布 BFT-1/BFT-4 失效。** 由区域共识子代理提出。

[00 §2.1](00_arch_zh_v0_1.md) 第 223 行固定 position 公钥；§2.3 第 286–292 行允许 Byzantine 方暴露私钥，并允许不同 session 的 Byzantine 集合不同。如果同一四成员委员会的三个 session 分别只把 a、b、c 算作 Byzantine，且敌手保留已泄露的长期私钥，名义上的“每次至多一个坏成员”不能阻止它累计掌握三把钥匙。Cid 只区分签名用途，不能阻止持钥者生成新签名。

但如果 Byz_s 已包含所有仍能冒用该 session 身份的泄露密钥，上述执行就超出故障上界；认证信道假设也可能意在排除这种复用。当前没有指定具体 BFT verifier，三把钥匙不能直接等同于有效 FINAL。审查结论只到：需明确跨 session 的腐化、恢复和密钥有效期语义。

**建议。** 将“敌手可为其产生有效签名”的身份计入该 session 的 Byzantine 权重；或另定义重密钥、密钥擦除及历史证明保护机制。不要仅用“每个 session 内静态”替代跨 session 约束。

**人工确认：** [ ] 密钥泄露持续计入故障集合；或 [ ] 提供恢复和重密钥契约。

## 12. 第二轮发现（v0.2 新增）

新条目沿用 v0.1 的格式与状态记号：每条给出触发能力、相关规则、最小轨迹、影响、修复选项和人工检查项，并注明由哪个子代理提出、与 v0.1 条目的关系。数值使用流程 v0.2 的示例参数。

### 12.1 S10：前驱末块证书是每次交接的“免费期权”

**高优先级；设计缺陷，模型内轨迹确认；置信度：规则轨迹高，具体 BFT 的证书组装方式中。** 由共识与交接子代理提出，恢复与治理子代理给出同类结论。主代理核对了 [02 BFT-7](02_session_program_zh_v0_1.md) 第 388–390 行、[09 §4.5](09_finality_rules_zh_v0_1.md) 第 278 行与 [03 ApplyLifecycle](03_session_manager_zh_v0_1.md) 第 226 行。本条把流程 v0.2 的变体 1b 从“条件性轨迹”升级为敌手可主动构造的轨迹，并给出按需触发 S5 的方法。

**触发能力。** 下面两种之一：

- 在 leader 收集投票的 BFT 中，一个负责前驱最后一个已决定 view 的 Byzantine leader；
- 在全员广播投票的 BFT 中，该 Byzantine leader 加上对末块起决定作用、总权重不超过 $F_s$ 的投票。

与 Espresso proposer 合谋还可多得至多 $\Delta_{\rm ts}$。

**最小轨迹**（流程 v0.2 的数值）：

1. c 的 Byzantine leader 在 09:00:10.2 提议 32 410，内含自己的交易 T。它只让不足以单独组成证书的诚实权重在 $\tau^{\rm stop}=$ 09:00:11 之前收到并投票；之后收到的诚实成员按 Accept 的时间条件拒绝。BFT-7 只约束诚实方，CheckFINAL 不检查时间，所以 Byzantine 一方可以在任何时刻补齐证书。
2. a 以 cid(32 409) 进入，并区域决定 32 411 起的 block。
3. 09:02:05 之前，Byzantine 一方二选一：
   - **公开证书**（$\theta\approx$ 09:02:06，早于 09:02:11）：GFR 接纳 32 410（$h_{\max}$），以 COMPLETE 关闭 (7,6) 并选定 cid(32 410)。a 已区域决定的约 115 个 block 被排除，成为 padding；T 按 09:00:10 的价格执行。
   - **扣留证书**：32 410 从未存在。之后再把 Res(32 410) 交给 a 的 Manager。[03 ObserveRegional](03_session_manager_zh_v0_1.md) 不检查公开截止，照常更新前驱视图；一旦 $D[32\,410]$ 是 a 的 padding，针对 (7,6) 的 cid 产生 IncompatibilityFact，按 S5 的传播规则关闭 a **已被选中**的 cid(32 409)。a 失去本小时剩余的生产。
4. 变体：只把证书泄露给 a 的一半诚实成员，使他们在约 $T_{\rm pub}+T_E$ 内分裂在两个 cid 上。

**影响。**

- 每次交接开头约 $T_{\rm grace}$ 加发布延迟之内的区域快速确认都可被撤销，相信区域确认的对手方可被双花。末块是否包含成为 2 分钟的免费期权（MEV），也是低成本的活性攻击。
- 跨 cid 的冲突签名不可罚没：CheckEvidence 按 descriptor 逐个检查（BFT-4 第 4 项），而诚实成员也会合法地在多个候选上签名（[03 §4.2](03_session_manager_zh_v0_1.md) 第 255 行）。
- 每当敌手主持前驱最后一个已决定的 view 时都可以重复，频率见 S17。
- v6 Part 0 的 S4(b)（快速确认与 G 一致）在 v9 中没有对应陈述。

**建议**（设计选择，待你决定）：

- (a) 明确“快速确认”只指已 Selected cid 中的区域最终，并写入产品语义。
- (b) 由后继的第一份结果固定前驱的终止 anchor：Tail 为 $\mathsf{NEXT}(\nu_s)$ 且 $\theta_q\ge\tau_s^{\rm stop}$ 时，第一份完整公开的、预先计算的后继 cid(X) 的结果关闭 $s$ 并以 X 为 anchor；X 须延续 $G$ 中 $s$ 的已接纳前缀，其链作为 anchor 依据随结果公开；此后 $s$ 的结果不再合格。诚实后继在 $\tau_s^{\rm stop}+\Delta_{i}+\Delta_{\rm cross}$ 之后才提议，届时诚实方已形成的证书都已由 W26 送达。这把期权压缩到一次发布延迟，也消除变体 1a 中等到 $\theta_s^{\rm end}$ 的停顿。
- (c) 采纳 S5 的修法：Kill 只沿实际 ancestry 传播。

**人工确认：** [ ] 快速确认的语义；[ ] 是否由后继结果固定终止 anchor；[ ] 是否把“末块证书在 stop 前传播给足够诚实权重”写成 BFT 必须满足的性质。

### 12.2 S11：Certifier 依赖 finalized L1，使多次提交的 channel 必然超时，并限制结算吞吐

**高优先级；设计缺陷确认；置信度：条文高，吞吐数值中。** 由认证与派生、恢复与治理两个子代理分别提出。与 S2 的关系：S2 需要主 Batcher 弃置 channel；本条在没有任何故障时就会到达 S2 的超时状态。

**位置。** [06 CertifyBatch](06_certifier_zh_v0_1.md) 第 97–98 行要求 prev 等于“自身 L1 视图中的当前 cursor”，并用 prev 的写入位置 $p_{\rm prev}$ 检查恢复对齐；[00 §1.2](00_arch_zh_v0_1.md) 第 68 行把 L1 定义为 finalized block 序列。因此 Certifier 只能在上一次接纳 finalized 之后，才认证下一次 cursor 转换。

**最小轨迹。**

1. 一个需要两次提交的 channel：frame 0 在 L1 block $O$ 被接纳。
2. 认证 frame 1 需要 $O$ 已 finalized（至少 64 个 slot），所以 frame 1 最早在 $O+65$ 落地，已超过 CHANNEL_TIMEOUT = 50。
3. 两种读法都失败：
   - 若 [07 SubmitBatch](07_batch_inbox_zh_v0_1.md) 只按字面检查 prev，闭合 frame 被接纳，cursor 越过派生已经丢弃的 channel。这些高度在原生截止后成为 deposit-only，随后 Kill，RESUME 撤销约 $W_{\rm seq}$ 的 $G$（流程 v0.2 §6 的规模）。
   - 若 07 继承原生 timeout，Inbox 期望 frames_digest 清零的 prev，而 Certifier 从不签这种 prev（S2），RESUME 之前无人能再提交。
4. 吞吐：每个 finality 周期至多一次提交，每次一个闭合 channel。子代理按 EigenDA 单 blob 约 16 MB 估算，约每秒十几 KB 的压缩数据（未在本仓库规格中核对）。负载高于此值时积压无界增长，每个高度最终错过原生截止；L2 上的廉价垃圾交易即可推高负载。

**影响。** A-DELIVER 的前提（诚实 Batcher、C 在线）成立，交付却失败，BAT-4 无从使用；按字面读法时，[00 §4.5](00_arch_zh_v0_1.md) 的 $D\preceq G'$ 也不成立。

**建议。** 让 CertifyBatch 不依赖 L1 视图，与原生 certifier 核心一致：

- 由请求携带已接纳前缀的证据，对照 prev.frames_digest；
- 用 prev.source 检查恢复对齐：$\mathrm{RESUME}(c)$ 要求 Certifier 的 GFR 已处理 $c$；$\mathrm{GLOBAL}(g')$ 要求本次 gpos 不早于 $g'$；
- 新鲜度与 timeout 交给 Inbox 检查，并把原生 effective cursor 的 timeout 写入 07；
- A-DELIVER 增加“每个 channel 在 CHANNEL_TIMEOUT 内完成”和数据率上界。

主代理核对了用 source 检查的论证：RESUME 总在同一交易内改写 cursor，因此 prev.source 为 GLOBAL 时，自 prev 写入以来没有 RESUME 生效；若 RESUME 在此之后才落地，Inbox 的 prev 相等检查会拒绝旧转换，CERT-3 仍然成立。请确认 source 字段的语义足以承担这一职责。

**人工确认：** [ ] Certifier 是否依赖 finalized L1；[ ] 结算吞吐目标与 DA payload 上限。

### 12.3 S12：RESUME 的目标没有链上时间界

**高优先级；设计缺陷，相对 v6 的回退，确认；置信度高。** 由认证与派生、恢复与治理两个子代理提出。它扩展了 S4（低目标）与 S6（收录时间不确定）。v6 依赖合约检查 C-RES-5（$\mathit{ts}(L)<t(h_{\rm res})\le\mathit{ts}(L)+D_{\rm drift}$）；v9 的 [01 SubmitResume](01_l1_authority_zh_v0_1.md) 第 258 行只检查目标高于 cursor。

**三条轨迹。**

1. **远目标不可逆冻结。** 治理把 88 212 误签为 882 120，合约接受（高于 cursor）。按 [原生 override 规则](../megaeth_specs/specs/rollup/derivation.md#emergency-cursor-overrides)，每个派生节点必须先执行 838 608 个填充 block，才能处理之后的 L1 输入；按 149 blocks/s 计约 94 分钟。
   - 填充的 origin 停在 $L$，锚点 drift 约 9.2 天，Enter 不会成功；Force 沿用同一锚点。
   - 高于 882 120 的高度要到约 10 天后才到 slot 时间，普通 L2 在此之前无法恢复；L2 的 block 时间戳领先真实时间约 9 天。
   - 目标接近 $2^{64}$ 时，填充永远完成不了，派生与 L1 deposits 永久停止；按 [原生 recovery S8](../megaeth_specs/specs/block-certification/recovery.md#s8-a-previous-override-left-the-cursor-at-an-unreachable-position)，cursor 到达 uint64 最大值后，override 接口没有修复路径。
   - 结果：强制包含（v6 的 L2-L2）也依赖治理。
2. **正常运行时的 RESUME 同样会撤销 $G$。** cursor 正常落后 $G$ 约 20 分钟。在 $L$ 收录目标为 $\mathit{ts}(L)+25$ min 的 RESUME；到其键 $\mathit{ts}(L)+20$ min 时 $G\leftarrow D_L$，约 35–40 分钟（2 100–2 400 个 height）的全局最终内容被撤销，而系统并未处于紧急状态。
3. **签名的控制记录没有期限。** 若 SubmitResume 的 caller 不受限，同一控制序号的旧签名可以被第三方抢先提交；序号被占用后，更正后的签名失败（条件性：取决于 caller 规则）。

**影响。** [01 §2](01_l1_authority_zh_v0_1.md) 第 22 行“Authority 的检查本身不依赖 $\mathsf M$ 诚实”不成立；$G$ 的保留范围与强制包含都依赖治理诚实和链外规程。

**建议。**

- 合约按收录时间计算目标：$h_{\rm res}:=\mathrm{slot}(\text{block.timestamp}+T_{\rm prep})$；或检查 $\text{block.timestamp}+T_{\rm prep}\le\mathit{ts}^*(h_{\rm res})\le\text{block.timestamp}+T_{\rm drift}-\epsilon$。这同时解决 S6 与远目标。有了下界，“高于 cursor”也就蕴含“高于 $D$ 的末端”：$D$ 由批次派生的末端不超过 cursor，回退的末端不超过 $\mathit{ts}_{\rm L1}-W_{\rm seq}+T_{\rm drift}$。
- 链上紧急守卫：要求 $\text{block.timestamp}\ge\mathit{ts}^*(\text{cursor.height}+1)+W_{\rm exp}$。它在正常运行时为假，在 GFR 可能 HALT 时为真。迟到接纳的批次会刷新 cursor（流程 v0.2 §6.4），因此宜与 §14 的 R1 一起采用。
- 控制记录增加有效期，或绑定一个近期 L1 block hash。
- $pk_{\mathsf M}$ 增加带时间锁的轮换。
- 若仍保留链外规程，把治理诚实写成 $G$ 保留范围与强制包含的安全前提（与 S4 一起决定）。

**人工确认：** [ ] 合约计算目标或检查时间界；[ ] 链上紧急守卫；[ ] 控制记录有效期；[ ] 治理诚实是否列为安全前提。

### 12.4 S13：主 Batcher 与 Certifier 密钥不可替换

**高优先级；信任边界缺口确认；置信度：接口缺失高，竞争的经济性中。** 由认证与派生、恢复与治理两个子代理提出。它补充 S2，以及 v0.1 §12 中“主 Batcher 的反复占用与弃置”一项。

**位置。** [07 §3.1–§3.2](07_batch_inbox_zh_v0_1.md) 第 44、87、89 行固定 admin、提交权限与 certifier_pk；[01 §3.3](01_l1_authority_zh_v0_1.md) 的接口中，没有修改 Batcher 权限或轮换 $pk_{\mathsf C}$ 的调用。

**轨迹。**

1. 主 Batcher 提交一个已认证、未闭合的 frame 0 后停止。按 v9 的 Certifier，超时后无人能提交（S2）。
2. 即使修好 S2，主 Batcher 在 cursor.origin + EXCLUSIVE_WINDOW 之前独占，并可在每次 timeout 边界重开 channel。每赢一次占用 10 分钟；赢约 8 次，就能让一个在 $\mathit{ts}^*(h)+W_{\rm exp}$ 才进入 $G$ 的高度错过原生截止。原生规则靠 admin 移除地址来限制这种行为，v9 的 Authority 没有这个调用；而每次闭合提交都会重新开启独占窗口。
3. 每次停顿都以错过原生截止、Kill、撤销约 12 h $G$ 的 RESUME 告终，可以无限重复。RESUME 规程的前提“Batcher 与 Certifier 已恢复”永远不成立。
4. $pk_{\mathsf C}$ 固定。密钥永久丢失（模型允许 $\mathsf C$ 离线）即永久结束普通 L2。密钥泄露时可签 next.height $=2^{64}-1$：07 不检查 next 的任何字段，此后“目标高于 cursor”永远不能成立，RESUME 也不可用。后一情形超出 A-CERT，但损害范围没有在任何地方说明。

**影响。** [05 §1](05_batcher_zh_v0_1.md)“Batcher 的行为只影响交付何时完成”不成立；$\mathsf C$ 被攻破时的损害范围没有界定。

**建议。** Authority 增加由 $\mathsf M$ 签署、带时间锁的调用，用于替换主 Batcher 与备选、轮换 $pk_{\mathsf C}$；允许 RESUME 指定新的主 Batcher，并让 override 写入的 origin 使独占窗口已经过期；Inbox 检查 $\mathit{ts}^*(\text{next.height})\le\text{block.timestamp}+\varepsilon$。

**人工确认：** [ ] 增加权限与密钥管理接口；[ ] 写明 $\mathsf C$ 被攻破的损害范围。

### 12.5 S14：non-live 期间提出的 Espresso block 可带旧 timestamp，把整段停机期间的结果带入 $G$

**高优先级；隐含假设，模型内轨迹确认；置信度中（未核对 HotShot 的时间戳校验）。** 由恢复与治理子代理提出。它是 v6 问题 3 的延续，比变体 A′ 更强：不需要在途 block。

**位置。** [00 §1.4](00_arch_zh_v0_1.md) 第 191 行明确：non-live 期间提出的 block 不获得 E-TS 保证；[09 §4.2、§4.4](09_finality_rules_zh_v0_1.md) 只用 $\theta$ 判断公开截止与过期。

**最小轨迹**（流程 v0.2 §5，停机从 10:00:01 起）：

1. 停机后期，一个 proposer 提出 $\theta=$ 10:00:02 的 block J，内含 a 的 36 000–36 010 和 b 在停机期间决定的 36 011–39 610。J 在恢复后 finalized。E-S 只要求 timestamp 不递减，允许这个值。
2. 重放时，J 的键早于 SessionEnd(7,7) 的 10:02:11。GFR 接纳 a 的结果直到 $h_{\max}=36\,010$，以 COMPLETE 关闭 (7,7) 并选定 cid(36 010)。同一 Espresso 处理循环中，(7,8) 的结果满足 $\theta<$ 11:02:11 与 $\theta<\mathit{ts}^*(h)+W_{\rm exp}$，也被接纳（常规后继不受“下一 block 才接纳”的限制，流程 v0.2 §3.3 第 8 步采用同一读法）。
3. 3 611 个 height 在 D2 00:00 成为全局最终。$D$ 自 21:40 起已是 deposit-only，RESUME 把它们全部撤销。在 stop 之后长期扣留的证书，也可以这样进入。

**影响。** 流程 v0.2 §5.7 与 §7 的“没有在途 block 时 $G$ 不撤销”在模型内不成立（见 §15 勘误 E1）；09 §4.2 的公开截止失去真实时间含义；S18 的退押上界在 non-live 期间无界。

**建议。** 采纳 v6 的修法 F1：Espresso block 的排序键取“证明它已 finalized 的、live 期间提出的后继 block”的 timestamp。或者把“HotShot 按本地时钟校验 $\theta$”写成 Espresso 的显式假设，并核对 HotShot 的实现。

**人工确认：** [ ] 采纳 F1；或 [ ] 增加 HotShot 时间戳假设。

### 12.6 S15：L1 最终化没有余量，origin schedule 也没有回退

**中高优先级；隐含假设确认；置信度：机制高。** 由恢复与治理子代理提出。流程 v0.2 §2.3 记录了 48 s 的余量，但没有写超出后的后果。

**位置。** [00 §3.4](00_arch_zh_v0_1.md) 的 schedule 与 $\omega_A$；[02 Build](02_session_program_zh_v0_1.md) 第 198 行执行 $\mathrm{Dep}^*_A(h)$，Accept 要求 $B.\mathit{os}=\mathit{os}^*_A(B.h)$；[09 §4.3](09_finality_rules_zh_v0_1.md) 的 R-WAIT。

**轨迹。** 示例的 $T_{\rm L1}=19.2$ min 恰好是以太坊正常情况下的最坏值（epoch 第 1 个 slot 的 block 经 95 个 slot finalized）。只要一个 epoch 未能按时 justify，最终化就再晚 6.4 分钟：

- schedule 选中的 origin 尚未 finalized，Build 与 Accept 等待约 5.6 分钟，跳过的 slot 变成 padding；
- $M_a$ 只有 6 s 余量，R-WAIT 也使 GFR 停顿。

**影响。** 所有区域的生产同时停顿，而这正发生在 BFT-5 已经假设掉的条件下。

**建议。**

- 把 $T_{\rm dep}$ 提高到约 27–28 min。$T_{\rm dep}+b_{\rm L1}\le T_{\rm drift}$ 仍然成立，Force 与 RESUME 之后首块的 origin 仍在 drift 内。
- 代价：稳态 origin 滞后变大，Expire 与原生截止之间的余量从 1 h 40 min 降到约 1 h 32 min，仍覆盖 $M_a+T_{\rm batch}=1$ h 20 min。需要与 $W_{\rm exp}$、$T_{\rm batch}$ 一起重算。
- 或者允许 schedule 的 origin 尚未 finalized 时，停在已 finalized 的前一个 L1 block。
- 把 $T_{\rm L1}$ 改称“典型值”，不再称为上界。

**人工确认：** [ ] $T_{\rm dep}$ 的取值；[ ] 是否允许 origin 回退。

### 12.7 S16：RESUME 开出的安排“不能启动”之后 Tail 未定义

**中优先级；规格缺口确认，实现可能分歧；置信度高。** 由恢复与治理、认证与派生两个子代理提出，与 S4 §6.2 同类。

**位置。** [09 §4.6](09_finality_rules_zh_v0_1.md) 的 RESUME 第 4 步把新窗口固定为 $[h_{\rm res}+1,h_{\rm res}+\mathrm{TB}]$；第 332 行的统一规则只输出 SessionClosed（EXPIRED），既不规定 Tail，也不分配后继。

**轨迹。** 合规的 RESUME 在 $L=$ D2 00:05:12 收录，$h_{\rm res}=88\,212$，键为 00:25:12，(8,0) 的 SessionEnd 键为 01:32:13。

1. 若 Espresso 在 (00:25:12, 01:32:13] 内没有 block（约 67 分钟的再次停机，或 RESUME 晚约 67 分钟收录，见 S6），开出时该安排已不能启动。
2. GFR 只输出 SessionClosed（EXPIRED），没有后继，直到 Expire(88 213) 的 10:30:13 才 HALT。
3. 按 01 的规程，再次 RESUME 要等 $D$ 的回退越过 88 212，约在 D2 12:05:12，损失约 12 小时。
4. 一种实现若把此时的 Tail 设为 HALT，另一种保留 NEXT，两者对之后的 Force 是否有效判断不同，$G$ 可能分叉。

**建议。** 像 Force 一样开出 RESUME 的窗口：$h_{\min}=\max\{h_{\rm res}+1,\mathrm{slot}(\theta_{\rm open}+T_{\rm init})+1\}$。或者规定“不能启动”即 HALT，并采用 §14 的 R2，使 HALT 且 $G\preceq D$ 时可以 RESUME。

**人工确认：** [ ] RESUME 窗口的开出方式；[ ] “不能启动”之后的 Tail。

### 12.8 S17：已证明为 Byzantine 的权重仍可投票；leader 规则未受约束

**中优先级；规格缺口确认（S9 的具体实例）；置信度：条文高，影响中（leader 规则未指定）。** 由共识与交接子代理提出。

**位置。** [01 §3.2](01_l1_authority_zh_v0_1.md) 第 132–136 行（被禁任者保留冻结投票权重）、§3.12 第 292 行；[09 §4.5](09_finality_rules_zh_v0_1.md) 第 287 行；G3.3。

**轨迹。**

- **权重。** 取 $W=100$、$F=33$。K1（权重 20）在 $s_1$ 中双签被禁任，K2（权重 20）在 $s_2$ 中双签被禁任。同一快照上的后续 session 仍给 K1∪K2 共 40 > F 的冻结权重：Plan 明知地安排了已证明 Byzantine 权重超过 $F_s$ 的委员会。被罚没到零的密钥仍以零成本投票。
- **时效。** 禁任只在 $\ell_s$ 不早于禁任所在高度时生效，而 $\ell_{s+1}=A_s.\mathit{os}$。例如 (7,7) 运行到 10:00:10，使用的是 07:40:00 的 L1 状态。
- **Leader。** leader_rule 的参数可以由选择输入导出。若以 Ref.block_id（前驱的末块）为种子，前驱的最后一个 leader 可以研磨交易顺序，影响后继的 leader 序列。未加权的轮换还让不超过 $F_s$ 的权重拆成许多 position 后主持大多数 view，包括决定 S10 频率的末尾 view。

**影响。** $\mathrm{Byz}_s\le F_s$ 可以被明知地违反；G3.3 的进展；S10 的发生频率。

**建议。** Plan 对 $\mathsf{Ban}(\ell_s)$ 中的公钥置零权重，并重算 $W_s,F_s,Q_s$（剩余 Byzantine 权重不超过 $F-b\le F'$，因此安全）；leader 按质押加权，种子取 $\ell_s$ 处的 L1 随机性，不取 block_id；规定每个 position 的最低质押。

**人工确认：** [ ] 禁任即置零权重；[ ] 约束 leader 规则的种子与权重。

### 12.9 S18：退押与证据窗口缺失

**中优先级；规格缺口确认；置信度：缺口高，利用需要长时间停机。** 由共识与交接、恢复与治理两个子代理提出。v6 的 S6 与问题 3 在 v9 中仍未处理。v0.1 §12 只指出合约缺失，本条给出具体路径。

**位置。** [01 Release](01_l1_authority_zh_v0_1.md) 第 236 行把条件全部交给 $\mathcal P_{\rm stake}$；[00 §1.4](00_arch_zh_v0_1.md) 第 191 行；[09 §4.2、§4.5–§4.6](09_finality_rules_zh_v0_1.md)；§4.4 第 274 行按 hash 排序合格扩展。

**要点。**

- v9 没有证据窗口 $T_{\rm ev}$、退出隔离，也没有把退押与 session 关联。
- **Espresso live 时期限有界。** 一个仍包含 pid 的 session 接纳结果的期限是有界的：空 session 沿用初始 anchor，Expire 限制缺口，Force 经 $\mathrm{last}(G)$ 刷新 Ref。因此 $\theta^{\rm end}<\mathit{ts}_{\rm L1}(a_{\rm ex})+T_{\rm drift}+W_{\rm exp}+T_{\rm init}+2\,\mathrm{TB}+T_{\rm grace}\approx 12$ h 34 min，其中 $a_{\rm ex}$ 是第一份不含该 pid 的快照的激活高度；再过 $\Delta_{\rm ts}+T_E$，它在真实时间中失效。v6 的公式可以沿用。
- **Espresso non-live 时无界。** 10:01 停机；第 7.5 天，一个 non-live 期间提出、$\theta=$ 10:01:30 的 block，携带退押之后才产生的 (7,7) 签名（S10 的迟到证书，或超过 $F_s$ 的权重伪造的冲突链）；它在第 8 天 finalized，于 7 天解绑期之后进入 $G$。冲突的合格扩展按 hash 排序，攻击者可以研磨优先级。L1 观察不到这些。
- G 停住期间，每个新 session 的 Ref 停在 $\mathrm{last}(G)$，委员会来自约 $W_{\rm exp}+T_{\rm dep}$ 之前的快照。

**建议。** 在 E-L/E-TS 下重述 S6：退押不早于上式加 $\Delta_{\rm ts}+T_E+T_{\rm ev}$，$T_{\rm ev}$ 从承载结果的 Espresso block finalized 起算。无条件的 S6 需要在 L1 上可验证的 Espresso watermark，证明最后一个可能使用含该 pid 快照的 session 已有 $\theta\ge\theta^{\rm end}$；或者与 S14 的 F1 一起采用。$\mathcal P_{\rm stake}$ 的解绑期须覆盖这个上界，并在 GFR watermark 落后 L1 时暂停计时。

**人工确认：** [ ] 恢复 S6 及其前提；[ ] 解绑期与 watermark 的关系。

### 12.10 S19：单区域超过 $F_s$ 时的后果未界定

**中优先级；隐含假设确认；置信度：(a)(b) 高，(c) 中。** 由共识与交接子代理提出。它不破坏 $G$ 的确定性，也不影响其他区域。

**位置。** [02 BFT-4](02_session_program_zh_v0_1.md) 第 321–330 行与 Accept；[09 §4.4](09_finality_rules_zh_v0_1.md) 第 263–271 行；[01 Join](01_l1_authority_zh_v0_1.md)。

- **(a) 没有可追责安全。** $F_s<\mathrm{Byz}<Q_s$ 时，被选中的 cid 中可以有两条已决定链，$G$ 取先到 Espresso 的一条。BFT-4 只禁止错误证据（“不要求每次违规都必然被发现”），能否罚没取决于所选 BFT 的取证能力。
- **(b) 未来时间戳进入 $G$。** $\mathrm{Byz}\ge Q_s$ 时，没有诚实方接受过的 block 也能最终化。GFR 检查截止、过期与 origin，但从不比较 $\mathit{ts}^*(h)$ 与 $\theta$。于是 timestamp 领先真实时间约 $T_{\rm dep}$ 减去实际 L1 最终化时间的 block（典型 5–7 分钟；按 19.2 min 计为 48 s）可以进入 $G$ 与 $D$；原生检查只相对 height 与 origin。
- **(c) 攻击成本由最小区域决定。** Join 允许任意选择区域：超过 $F$ 需要多于 $W_i/2$ 的新增权重，达到 $Q$ 需要 $2W_i$。

**建议。** 增加 BFT-8 可追责安全：同一 cid 的两条冲突已决定链，可以由两份证书构造出针对超过 $F_s$ 权重的证据。GFR 增加 $\mathit{ts}^*(B.h)\le\theta_{q_{\rm avail}}+\Delta_{\rm ts}$；诚实结果在 E-TS 下满足它。在 $\mathcal P_{\rm stake}$ 中增加每区域最低质押或上限。

**人工确认：** [ ] BFT-8；[ ] GFR 的时间上界检查；[ ] 区域质押下限。

### 12.11 S20：BFT 配置接口默认排除两类 BFT

**中优先级；隐含假设，实现可能分歧；置信度中。** 由共识与交接子代理提出。

**位置。** [02 §3.2](02_session_program_zh_v0_1.md) 第 117 行：first_received 以“proposal 标识”为键，而这个标识没有定义；§3.7–§3.9；BFT-5。

- **(a) 重新提议。** 若按 view 区分 proposal，Tendermint 式对已锁定 block 的重新提议会得到新的 $\tau_{\rm recv}\ge\mathit{ts}^*(h+1)+T_{\rm win}$：未锁定的诚实方拒绝它，已锁定的拒绝其他值，该 cid 在 $\tau^{\rm stop}$ 之前死锁。若按 block hash 区分，Byzantine leader 可以提前算出 height $h$ 的 block，在 $\mathit{ts}^*(h)$ 之前发给部分成员，使他们永久拒绝；其他成员锁定后，同样死锁。
- **(b) 链式提交。** HotStuff、Jolteon 这类链式提交的 BFT 需要后续高度才能提交，窗口的最后 1–2 个高度永远无法决定。“$h_{\max}$ 已被接纳”永远不会发生，每次交接都是变体 1a，S10 的窗口每小时都打开。

**建议。** first_received 以 $(\mathrm{cid},\mathsf H(B))$ 为键，对重新提议用 QC/POL 验证，而不重新做时间检查；或者把抽象限定为每个 view 提议新 block、且对末块有显式 finalize 阶段的 BFT（如 Simplex）。

**人工确认：** [ ] proposal 标识的定义；[ ] 适用的 BFT 族。

### 12.12 S21：CertifyBatch 未要求解码结果与派生一致

**中优先级；规格缺口；置信度中（照搬原生 certifier 的实现是安全的）。** 由认证与派生子代理提出。

**位置。** [06 CertifyBatch](06_certifier_zh_v0_1.md) 第 101 行（“本次可完整解码的 block records”）与 CERT-1(4)；[05 BlockRecord](05_batcher_zh_v0_1.md)（原生批次携带 timestamp，不携带 block_number）；原生 certifier 与 [Closed-Channel Decoding](../megaeth_specs/specs/rollup/derivation.md#closed-channel-decoding)。

**轨迹。**

1. Batcher 闭合一个 channel，其中带损坏的 zlib 结尾、超过 100 MB 的原始大小，或平移 $k$ 个 slot 的 timestamp。
2. 按 v9 文本运行的 Certifier 认证可解码的前缀，因为这些记录与 $G$ 一致。
3. 派生不释放该 channel 的任何内容，或把这些批次当作未来批次丢弃。
4. cursor 越过已丢失的内容，后果与 S11 相同。

**建议。** CERT-1(4) 要求 Certifier 的解码与派生完全一致：timestamp $=\mathit{ts}^*(h)$、原始大小上限、channel_id 相等，并做完整流校验与闭合。

**人工确认：** [ ] Certifier 的解码与派生逐项一致。

### 12.13 S22：A-CUSTODY 的释放条件对新加入节点无法满足

**低优先级；隐含假设。** 由认证与派生子代理提出。

**位置。** [08 A-CUSTODY](08_da_zh_v0_1.md) 第 26 行；[09 §3.4](09_finality_rules_zh_v0_1.md) 的 checkpoint 只能对照公共前缀的重放结果验证；ResumeCut.derived_prefix 从 $h^0$ 开始。

**问题。** 保管者在 checkpoint 覆盖之后可以丢弃字节，但 checkpoint 只有经过重放才算验证。数据丢弃之后加入的节点无法验证任何 checkpoint，也无法处理 RESUME。

**建议。** 保管永久化，或把 checkpoint 锚定到已在 L1 结算的 StateClaim。

**人工确认：** [ ] 保管期限与 checkpoint 的信任来源。

### 12.14 S23：接纳条件 5 依赖读取者本地的 L1 前缀，诚实读取者可以算出不同的 $G$

**高优先级（按字面读法是全文最严重的一条：破坏 $G$ 的一致性）；规格缺口加缺失的参数约束，确认；置信度：条文高，实现会按字面读法的可能性中。** 由 GFR 子代理提出。主代理核对了 [09 §4.4](09_finality_rules_zh_v0_1.md) 第 269 行（“且所需 L1 输入已 finalized”是接纳条件）、R-WAIT（第 245–251 行）与第 251 行“未完整公开的单份 publication 暂不合格，不阻塞其他事件”。它也使 v6 问题 3 更严重。

**根因。** R-WAIT 只保证读取者已有 timestamp 不低于 $x-M_a$ 的 L1 block。新高度 $h$ 的 origin 的 timestamp 不超过 $\mathit{ts}^*(h)-T_{\rm dep}$。流程 v0.2 取 $T_{\rm dep}=M_a=20$ min；E-TS 允许 $\theta_q$ 比提出时刻低 $\Delta_{\rm ts}=15$ s。只要 $\theta_q<\mathit{ts}^*(h)$，origin 就可能落在 R-WAIT 保证的前缀之外。条件 5 于是取决于各读取者自己同步到的 L1 前缀，而缺少数据被当作“不合格”跳过，不是 WAIT。

**触发能力。** 一个比真实时间慢、但仍在 E-TS 之内的 Espresso 时间戳。第二个读取者是正在追赶的诚实节点，其 L1 同步落后于 Espresso 同步，例如重启后的 Manager、Batcher 或 Certifier。

**最小轨迹。**

1. $h=37\,800$（10:30:00）的 origin 是 L1 block 3 050（10:10:00）。它在 10:30:00.6 被决定，收录于 10:30:01 提出、$\theta_q=$ 10:29:47 的 $J_q$。
2. $J_q$ 的 R-WAIT 只要求 L1 覆盖到 timestamp 不低于 10:09:47，block 3 049 即可。
3. 读取者 P 在线，已有 3 050，接纳 37 800。读取者 Q 的 L1 前缀止于 3 049，条件 5 不成立，跳过这份结果，而且不阻塞。
4. 一条 $\theta_a=$ 10:29:48 的 Force（收录于不晚于 3 049 的 block）在 $\theta=$ 10:29:48 的 $J_{q+1}$ 中执行，R-WAIT 仍由 3 049 满足。
5. Force 的 anchor $\mathrm{last}(G)$ 在 P 处为 37 800，在 Q 处为 37 799：$\mathrm{id}(A)$ 不同，新 cid 不同，两者的 $G$ 从此永久分叉。

**更宽的变体**（v6 问题 3 的情形）：$\theta=$ 10:00:01 的旧时间戳 block 携带 (7,8) 直到 39 000 的结果，其 origin 到达 block 3 150，而 R-WAIT 的切面约在 block 2 901。Q 跳过这些结果，SessionEnd(7,8) 于 11:02:11 在 Q 处固定 regular anchor 约 36 020 或更低，P 处为 39 000。这一变体不需要 Force。

**影响。** [09 §1](09_finality_rules_zh_v0_1.md)“任何读取者……都得到同一条 $G$”不成立；Tail 与生命周期事件也不一致。

**建议。**

- 条件 5、6 只对按键确定的切面 $C(x)=\{\ell:\mathit{ts}_{\rm L1}(\ell)\le x-M_a\}$ 求值，不对本地前缀求值；本地缺少该切面内的数据时返回 WAIT。
- 增加参数约束 $T_{\rm dep}\ge M_a+\Delta_{\rm ts}$（例如 20 min 15 s），使诚实、live 期间的结果不会因此被推迟。
- 附带效果：键为 $\theta$ 的 block 只能接纳按 schedule 生产、$\mathit{ts}^*(h)\lesssim\theta+27$ s 的高度。这条因果约束挡住了旧时间戳 block 一次带入整段停机期间结果（S14 的主要部分），但挡不住内容早于 $\theta^*$ 的在途 block（A′）。

**人工确认：** [ ] 条件 5、6 按键确定的切面求值；[ ] 缺数据时 WAIT；[ ] $T_{\rm dep}\ge M_a+\Delta_{\rm ts}$。

### 12.15 S24：first_complete_q 不是 (result_key, 日志) 的函数

**高优先级；规格缺口确认；置信度中高。** 由 GFR 子代理提出。主代理核对了 [09 §3.2](09_finality_rules_zh_v0_1.md) 第 102 行与 [09 §4.2](09_finality_rules_zh_v0_1.md)（“证明、必要 bodies 和 context 来源均已在公开对象集中”）、[04 §3.2](04_publisher_zh_v0_1.md)（按引用省略已公开对象）、[02 AnchorEvidence](02_session_program_zh_v0_1.md)。

**背景。** result_key $=(\mathrm{cid},\mathsf H(B))$ 不含 $\pi_{\rm fin}$ 与 $\pi_A$，不同的 publication 会合并到同一条目；规格没有定义哪些组成部分是“必需的”。

**轨迹。**

- **(a) 截止处分裂。** (7,7) 的一个 Byzantine 成员在 $\theta=$ 10:01:30 公开 Res(36 010)：$\pi_{\rm fin}$ 与 $C$ 有效，但 $\pi_A$ 引用 anchor 32 410 的一个从未公开的证明版本。它同时经 W26 转发这份 Res；后继的 Publisher 按 key 去重（PUB-2），保留并重新发布这个 $\pi_A$。a 自己的诚实副本因手续费上涨在 $\theta=$ 10:02:20 才落地。取对象并集的读取者（anchor 已在 $G$ 中）认为结果在 10:01:30 已完整，接纳 36 010；要求每条消息自带 $\pi_A$ 来源的读取者到 10:02:20 才认为完整，已过截止。两者的 regular anchor 分别为 36 010 与 36 009，$G$ 分叉。
- **(b) 严格读取者停顿。** FORCE 与 RESUME 的 anchor 来源在 L1 或重放中，从不在 Espresso 上。要求这些来源“在公开对象集中”的读取者，永远无法认定 RESUME 之后 (8,0) 的任何结果完整；它的 $G$ 停住，之后遇到 Expire，而其他读取者继续前进。
- **(c) checkpoint 不一致。** 同一 block 可以有两份有效的 $\pi_{\rm fin}$。$\mathrm{src}_G$ 记录哪一份没有规定，一个实现生成的 GlobalCheckpoint 会被另一实现的 Init 拒绝（第 174 行）。

**建议。** 定义 $q_{\rm avail}(\mathrm{cid},B)$ 为最小的 $q$，使 $J_0\ldots J_q$ 中已验证的对象包含 $B$ 或其同 cid 后代的一份有效证明，以及从 $\nu.\mathit{anchor}$ 到 $B$ 的全部 bodies。anchor 由 $\nu$ 本身确定，$\pi_A$ 不参与 GFR 的判断。$\mathrm{src}_G$ 记录位置最早的有效证明，同位置时取 hash 最小者。

**人工确认：** [ ] $q_{\rm avail}$ 的定义；[ ] $\pi_A$ 是否参与 GFR；[ ] $\mathrm{src}_G$ 的证明选择规则。

### 12.16 S25：L1 最终化停顿使按时进入 $G$ 的内容错过原生截止

**中高优先级；隐含假设确认；置信度中（未核对 HotShot header 的 L1 字段语义）。** 由 GFR 子代理提出，是对 v0.1 §12 中在途 block 一项与 S14 的补充：F1 只解决 Espresso 时间戳的失真，不解决 L1 最终化停顿造成的处理延迟。

**轨迹。** R-WAIT 让每个事件都等待 L1 finalized。原生截止约在块 slot 之后 11 h 40 min，而接纳最晚可在块 slot 之后 10 h（以 padding 补缺口时），Batcher 需要 1 h，只剩约 40 分钟（扣除 $M_a$ 后约 20 分钟）吸收处理延迟。一份以 padding 补缺口、键为 $\mathit{ts}^*(X)+9$ h 59 min 的结果，若以太坊一次失去最终化使其处理推迟约 1 小时（2023 年 5 月曾出现约 25 分钟与约 1 小时的事件），就会错过原生截止：$D[X]$ 成为 deposit-only，下一次 RESUME 把它从 $G$ 中撤销。

**建议。** 在接纳时增加守卫：$\mathrm{ddl}(h)-\text{l1\_head}(J_{c(q)})\ge\lceil T_{\rm batch}/b_{\rm L1}\rceil$，其中 l1_head 取 Espresso header 记录的 L1 head。这把“接纳后被撤销”变成“不予接纳”。

**人工确认：** [ ] 是否在接纳时检查原生截止余量。

### 12.17 S26：Force 与同时段 SessionEnd／Expire 的先后由单个 Espresso proposer 决定

**中优先级；设计缺陷确认；置信度高。** 由 GFR 子代理提出。主代理核对了 [09 §4.3](09_finality_rules_zh_v0_1.md) 的 block 内部顺序（Force 在第一个 $\theta_q\ge\theta_a$ 的 block 开头执行）与 §4.6 的 Force 窗口。结果是确定的，但可被单个 proposer 操纵。

**触发能力。** 第一个 $\theta\ge\theta_a$ 的 Espresso block 的 proposer，在 E-TS 之内选择 $\theta$。

**轨迹。**

1. SessionEnd(7,7) 的键为 10:02:11。一条 $\theta_a=$ 10:02:00 的 Force 安装新的循环排列 (a, b, c)，全部区域启用。下一个 block 在 10:02:01 提出。
2. 诚实 proposer（$\theta=$ 10:02:01）：Force 先执行。旧 NEXT 是 (7,7)，区域 a，已贡献 block，来源区域为 a，Plan 选 b。
3. proposer 取 $\theta=$ 10:02:12（不超过 $\tau+15$ s，允许）：SessionEnd 先执行，把 (7,8) 分配给 b；Force 看到的旧 NEXT 是 (7,8)，来源区域为 b，Plan 选 c；窗口也平移 11 s。
4. 同一手段用于 $\theta_a$ 之后紧接的 Expire 键，可以使 Force 失效（HALT，进入死区，只能 RESUME）。

**影响。** 治理意图可被操纵。

**建议。** 把 FORCE 设为独立事件，键为 $\theta_a$，类别序为 RESUME ≺ FORCE ≺ EXPIRE ≺ SESSION_END ≺ ESPRESSO；窗口按 $\theta_a+T_{\rm init}$ 计算；R-WAIT 要求 L1 覆盖到 $\theta_a-M_a$、Espresso watermark 不低于 $\theta_a$。治理规程增加 $\theta_a\le\mathrm{key}(\mathsf{Expire}(h_G+1))-T_{\rm init}-$ 余量。

**人工确认：** [ ] Force 是否成为独立的键控事件。

### 12.18 S27：交接时几秒的发布延迟导致约 2 分钟的工作作废

**中优先级（骚扰）；设计缺陷确认；置信度高。** 由 GFR 子代理提出。与 S10 不同，这里不需要扣留任何东西。

**位置。** [03 §4.3](03_session_manager_zh_v0_1.md) 第 259–263 行（调度已就绪的最高候选）与 W26；[09 §4.5](09_finality_rules_zh_v0_1.md) 第 278 行；PUB-4。

**轨迹。**

1. (7,6) 在 09:00:10.5 决定 32 410；a 经 W26 得知，以最高候选 cid(32 410) 进入。
2. 32 410 的 publication 推迟到 $\theta=$ 09:02:15，比截止晚 4 s；32 409 已按时公开。
3. 09:02:11 时 GFR 选定 cid(32 409)，a 在 cid(32 410) 上的约 120 个 block 被排除。
4. a 约在 09:02:12 重新进入，32 411 到约 32 531 成为 padding。
5. 交接时刻固定且公开，手续费上涨或 namespace 拥堵可以在每个 session 边界重复制造这种情况。

**影响。** 流程 v0.2 §3.5“交接没有停顿”的结论需要附加条件。

**建议。** Manager 只在更高候选的 publication 已以 $\theta<\theta_s^{\rm end}$ 收录后才切换到它；代价是至多 $T_{\rm pub}+T_E$ 的确认延迟，而不是 2 分钟的作废工作。Publisher 在 $\tau^{\rm stop}$ 之前的末尾 block 提高手续费。

**人工确认：** [ ] 候选切换的条件。

### 12.19 GFR 子代理对已覆盖条目的补充

- **S14 的修法 F1 在 v9 中需要五处同步替换。** 令 $\hat\theta_q=\theta_{c(q)}$，$c(q)$ 是在 finalized 链上以规范方式确定的、其 QC 链提交 $J_q$ 的最小 block。$\hat\theta$ 要替换：Espresso 事件的键；公开截止判断与 $q_s^{\rm end}$；watermark；Force 的阈值；Force 的窗口。$\hat\theta$ 不递减，watermark 的完整性与同键规则仍成立；E-L 的 $T_E$ 已包含最终化，PUB-4 仍给出 $\hat\theta\le\tau+T_{\rm pub}+T_E+\Delta_{\rm ts}$，$T_{\rm grace}$ 与 $M_a$ 的预算不变。在 A′ 中 $J^*$ 的键移到约 D2 00:00，SessionEnd(7,7) 先关闭 (7,7)，36 000 不进入 $G$。代价：Espresso block 要暴露提交结构，块 $c(q)$ 仍依赖 E-TS。较便宜的近似是 $\hat\theta_q=\theta_{q+k}$，$k$ 不小于提交深度，代价是 $k$ 个 block 时间。
- **S16 的“不能启动”。** 除 Tail 未定义外，这条规则还比常规的 SessionEnd 重放差：常规重放会继续推进。改为继续推进，或像 Force 一样重算窗口，对 origin 是安全的：$\omega_A$ 从 $A.\mathit{os}=L$ 追上 $\ell^{\rm sch}$，跨越 padding 后 O1–O3 仍成立。
- **S14 与 S23 的组合。** 经由 S23，旧时间戳 block 不只会使 $G$ 被撤销，也会使诚实读取者对 $G$ 意见不一。

GFR 子代理确认成立的事项：同键事件的顺序严格且全；Force 的 $\theta_a\ge\mathit{ts}_{\rm L1}(\ell_U)+M_a$ 保证 $\ell_U$ 在 R-WAIT 的切面内；同一 block 中的多条 Force 会重新核对目标；与 RESUME 同键的 Force、抢在 RESUME 的 OPEN 步骤之前的 Force 都有确定结果；重放中计算的 Plan 对三种边界都满足 $\ell_s\le x-M_a$；resumeReader 只依赖 $p_{\rm res}$ 之前的 L1 与 DA，同一 block 中的多条 RESUME 可以正确复合；按 hash 去重加 BFT-1 已足够；$q_{\rm avail}$ 只会提前，垃圾消息或拆分对象不能把结果推过截止。

## 13. 已检查但不提升为新漏洞的事项

### 13.1 v0.1 的列表（照录，附 v0.2 注）

- **不同候选 cid 决定不同内容。** BFT agreement 明确限于同一 cid；跨候选快速确认需要额外产品承诺。当前不能因两份候选 FINAL 不同就声称双花已进入 D。
  *v0.2 注：S10 表明敌手可以主动制造这种分歧，并据此双花区域快速确认；是否给出产品承诺见 S10 建议 (a)。*
- **停机前的 Espresso block 恢复后才 finalized。** 这是模型允许的轨迹；E-TS 约束提出时间，不约束后来可读的时间。A′ 会使 G 在 RESUME 时撤销，但 v9 §7 已允许该语义。应纠正“停机绝不撤销 G”的解释，不能伪装成全新的共识漏洞。
  *v0.2 注：S14 表明不需要在途 block：non-live 期间提出、带旧 timestamp 的 block 就能把整段停机期间的结果带入 $G$，再被撤销。*
- **已接纳 DA 数据不可取回。** v9 明确 WAIT，不能把它当作不存在批次而填空。它阻塞派生和恢复；违反 DA-RET 时不适用强制包含活性，未找到绕过 WAIT 的合法入口。
  *v0.2 注：S22 补充 A-CUSTODY 的期限问题。*
- **等 timestamp 的 watermark。** 当前 `≥x` 足以排除未来 `<x` 的 Espresso block；非 Espresso 类别排在同键 Espresso 前，配合严格 `<` 截止，没有确认必须改成 `>x` 的漏洞。
- **治理可选择过远未来目标。** 合约只有 cursor 单调守卫，确实需要说明这种授权输入的资源和活性影响，但它违反链外时间规程。未把任意恶意治理造成的停机当作普通 Byzantine 区域的能力。
  *v0.2 注：S12 将此项升为高优先级：远目标迫使派生先执行全部填充，可以永久停止强制包含；正常运行时的 RESUME 也会撤销约 40 分钟的 $G$。*
- **主 Batcher 的反复占用与弃置。** 原生权限/timeout 可能使诚实备选无法满足交付预算；v9 已明确 A-DELIVER 不是由“存在诚实 Batcher”推出。S2 列出的是更具体的接口冲突；一般性的 1 h 交付保证仍是待落实的服务前提。
  *v0.2 注：S11 表明没有故障时 channel 也会超时；S13 表明 v9 没有移除主 Batcher 或轮换 $pk_{\mathsf C}$ 的接口。*
- **罚没、退押。** Release 把条件交给抽象的 P_stake，未给出退出隔离期或历史证据期限。不能凭这个省略证明攻击者一定能逃罚；也不能声称 v9 已证明“释放前覆盖所有历史责任”。若需要该性质，须补出抽象契约及其实现依据。
  *v0.2 注：S17（禁任者保留投票权重）与 S18（退押期限的具体路径）。*
- **O1–O3、证据无误伤。** 本轮没有确认固定 origin 构造在其声明前提下失败；具体 BFT 证据关系依赖指定 verifier，未进行实现证明。

### 13.2 第二轮核对后成立的事项

- **$G$ 的确定性。** 在 Byzantine 区域、以及受 E-TS 约束的 Espresso proposer 下，$G$ 仍是确定的。至多 $\Delta_{\rm ts}$ 的时间偏移不能把按 PUB-4 及时交付的诚实结果推过 $\theta_s^{\rm end}$。
- **Context 隔离与 cid 内的罚没。** 同一 cid 内的双签即使发生在未选中的候选中也可罚没，因为 SubmitOffense 接受任何已验证的 $\nu$。诚实成员不会因跨 cid 签名而自证违规。
- **区域实例的时间与生命周期。** 在 $\le F_s$ 时，Accept、Build、NotBefore 给出 G1.2、G1.3。Enter 幂等，Close 不可逆，暂停保留锁，恢复前复查 stop。
- **Plan 的刷新。** Ref 在前驱选定时固定，$\ell_s$ 经 Force（$\mathrm{last}(G)$ 与 floor）和 RESUME 刷新；跳过没有可提议者的区域，并检查 $3F_s<W_s$。一级预备（只预备已选定前驱的后继）不能被单个区域利用：$s+2$ 的 Plan 约在 58 分钟前即可算出；只有 R-WAIT 造成的 GFR 停顿有影响，流程 v0.2 §5.7 已记录。
- **Force。** Force 从不撤销 $G$；针对已关闭分支的过时 Force 没有效果。远落后的区域 anchor 之后，O1–O3 仍成立。旧分支的结果不能在同一 $J_q$ 中赶在 Force 之前进入 $G$。v6 问题 1 在 v9 中已修复：新窗口从 $\theta_q+T_{\rm init}$ 之后开始，$\omega_A$ 从 $\mathrm{last}(G)$ 出发满足 O1–O3。
- **RESUME 与认证的边界。** RESUME 关闭旧的接纳窗口，旧分支结果之后不能进入。同一 L1 block 中 SubmitBatch 与 CursorSet 的先后顺序是安全的：override 之后的批次过不了 prev 检查，之前的批次属于绑定精确 RESUME 位置的 $D_L$；RESUME 的键要求 $L$ 已 finalized，所以没有证书能跨越一次 RESUME。过时的 gpos 只剩 S3 的归纳起点问题。CERT-2 与 CERT-3 成立。
- **Deposits。** $\omega_A$（O1、G2.4）、填充 origin 规则（不超过 $L$，timestamp 不超过块时间）与同高度替换都不会跳过或重复任何 L1 block 的 deposits。合规 RESUME 的 anchor origin 为 $L$，第一个新 origin $L+1$ 在 drift 之内。
- **强制包含延迟。** 在 Espresso、区域或批次层故障下约为 $W_{\rm seq}$；例外是违反 DA-RET（08 已写明 D 与 RESUME 等待，没有不安全的绕过）和 S12 的远目标。
- **Leader 的自由度。** Leader 不能自由选择 timestamp 或 origin（VP1；Accept 检查 $\mathit{os}^*$）。少数诚实副本的时钟漂移只影响活性，但慢时钟可能产生 S10 那样的迟到证书。
- **Kill。** 除已知的约 19 分钟 D 滞后外，没有发现 Kill 的漏判；除 S5 外，没有发现误判。

### 13.3 已知且仍然存在

- **Espresso 永久停机时没有出口**（v6 问题 6）。处理 RESUME 需要 Espresso watermark，所以 Espresso 永不恢复时没有普通恢复路径。这一前提仍没有在 00 或 01 中写明。

## 14. 跨条目的修复建议

### 14.1 R1：Certifier 与 Inbox 执行原生截止

由认证与派生子代理提出，主代理认为方向成立，细节待核对。原生 Inbox 接纳已过窗口的批次，理由是拒绝它们“只会让 cursor 停住”而没有补救手段（[batch-inbox Rationale](../megaeth_specs/specs/block-certification/batch-inbox.md#rationale)）。v9 有 RESUME 作为补救，这个理由不再成立。

建议：CertifyBatch 证明、SubmitBatch 检查——闭合提交所在的 L1 block 不晚于相应高度实际父 epoch 的窗口截止（子代理写作 prev.origin + seq_window_size；对 36 000 为 6 499，对 35 401 为 6 450）。效果：

- 变体 A′ 的 36 000 与流程 v0.2 §6.4 的批次不能再推进 cursor。
- cursor 停住即意味着分叉，01 规程中依据 cursor 的判断与“目标高于 cursor”才有意义；$D\preceq G'$ 成立；CERT-4 不受影响。

前提：截至当前 cursor 不变式已成立。需要核对它与原生 Batch Queue 逐高度检查的等价性，包括多高度 channel 的截止取最早高度。

### 14.2 R2：允许“已 HALT 且 $G\preceq D$”时 RESUME

由恢复与治理子代理复核 v6 的修法 4，结论是在 v9 中成立。

- Expire 一旦处理（watermark 已不低于其键），Espresso 的 timestamp 单调性排除了之后出现更小 $\theta$ 的 block，此后只有 RESUME 能改变 $G$；$D$ 只增长。所以 $\mathsf M$ 签署时成立的 $G\preceq D$ 在 $D_L$ 处仍然成立，不撤销任何块。
- 比较必须在 finalized $D$ 上按原生 hash 进行，不能按 cursor 高度（流程 v0.2 §6.4）。

收益：消除发布故障导致 HALT 之后约 2 小时的死区。局限：

- $G$ 中仍有未进入 $D$ 的迟到或旧 timestamp 结果时（S14）不适用；
- 发布故障的主要代价仍是 $W_{\rm exp}$（10 h）。

HALT 也蕴含 S12 的链上紧急守卫，可作为该前提的链上支撑。

## 15. 对流程 v0.2 的勘误

下列表述与本版发现冲突。按你的工作约定，本版没有改动流程 v0.2 的原文，只列出建议的替换。

| # | 位置 | 现有表述 | 冲突 | 建议 |
| --- | --- | --- | --- | --- |
| E1 | [流程 v0.2](10_pipelines_zh_v0_2.md) §5.7 第一条；§7“Espresso 停机超过约 10 h”一行 | “没有在途 block 时，原 $G$ 全部保留”；“没有在途 block 时 $G$ 不撤销” | S14：non-live 期间提出、带旧 timestamp 的 block 也能使 $G$ 被撤销，00 §1.4 第 191 行允许这种 block | 改为“没有在途 block，且没有 non-live 期间提出的 block 带入停机期间的结果时” |
| E2 | 同上 §3.4 变体 1b 末段 | “这里是符合 BFT-7 的条件性轨迹，不是对每种实现都成立的扣留攻击” | S10：Byzantine leader 可以只让不足以单独组成证书的诚实权重在 stop 前投票，主动构造这条轨迹 | 改为指向 S10，并注明“敌手可构造，频率取决于 leader 规则” |

## 16. 建议的人工检查顺序与验证状态

同一修法方向的条目放在一起检查。

| 顺序 | 条目 | 首先决定什么 |
| --- | --- | --- |
| 1 | S23、S24、S1 | $G$ 的确定性：接纳条件按键确定的 L1 切面求值；$q_{\rm avail}$ 的定义；下载进度与已处理 frontier 分离 |
| 2 | S10、S5、S27 | 交接：末块证书的期权、Kill 只沿实际 ancestry 传播、候选切换的条件 |
| 3 | S11、S2、S21、R1 | Certifier 与 Inbox：是否依赖 finalized L1、timeout 的 effective cursor、解码一致、原生截止 |
| 4 | S12、S4、S6、S16、R2 | RESUME：目标由合约计算或检查时间界、链上紧急守卫、“不能启动”之后的状态、HALT 后的 RESUME |
| 5 | S13 | 权限与密钥：替换主 Batcher、轮换 $pk_{\mathsf C}$、$pk_{\mathsf M}$ |
| 6 | S14、S25 | Espresso 时间戳：F1 或 HotShot 假设；接纳时的原生截止余量 |
| 7 | S26 | Force 是否成为独立的键控事件 |
| 8 | S15、S17、S18、S19、S20 | 参数与模型契约：$T_{\rm dep}$、禁任即置零、退押期限、BFT-8、适用的 BFT 族 |
| 9 | S3、S7、S8、S9、S22 | v0.1 的其余条目与保管期限 |

**验证状态。**

- 第二轮四个子代理均已返回报告。GFR 子代理因速率限制中断一次，之后从保存的进度继续完成。
- 主代理对每条新发现核对了所引条文，核对过的行号写在各条的“位置”中。
- 以下数值未在本仓库规格中核对，按子代理的估计记录：
  - S11 的吞吐数值（依赖 EigenDA 单 blob 上限）；
  - S20 对 Tendermint、HotStuff 的行为判断；
  - S25 依赖的 Espresso header 字段语义。
- 没有运行真实的 BFT、Ethereum、Espresso 或 DA 网络测试，也没有把新的反例写入 [checks/security_counterexamples.py](checks/security_counterexamples.py)。本报告不是完整的安全证明或实现审计。
- 00–09 的修复都尚未写回规格，全部等你决定。

## 附录 A：条目来源与合并

| 条目 | 来源（第二轮子代理的报告编号） | 与 v0.1 的关系 |
| --- | --- | --- |
| S10 | 共识 #1；治理 #8 | 升级流程 v0.2 变体 1b；可按需触发 S5 |
| S11 | 认证 #1；治理 #3 | 无故障时到达 S2 的状态 |
| S12 | 认证 #3、#5（控制记录重放部分）；治理 #1 | 扩展 S4、S6；升级 v0.1 §12 的“过远未来目标” |
| S13 | 认证 #2；治理 #2 | 补充 S2 与 v0.1 §12 的“主 Batcher” |
| S14 | 治理 #4 | 强于 v0.1 §12 的在途 block；v6 问题 3 |
| S15 | 治理 #5 | 新 |
| S16 | 治理 #6；认证 #5 | 与 S4 §6.2 同类 |
| S17 | 共识 #2 | S9 的具体实例 |
| S18 | 共识 #3；治理 #9 | 升级 v0.1 §12 的“罚没、退押”；v6 S6 |
| S19 | 共识 #4 | 新 |
| S20 | 共识 #5 | 新 |
| S21 | 认证 #4 | 新 |
| S22 | 认证 #7 | 新 |
| S23 | GFR #1 | 新；与 S1 同属 $G$ 的确定性 |
| S24 | GFR #2 | 新；与 S1 同属 $G$ 的确定性 |
| S25 | GFR #3（L1 停顿部分） | 补充 S14 |
| S26 | GFR #4 | 新 |
| S27 | GFR #5 | 新 |
| §12.19 | GFR #3（F1 部分）及附注 | 补充 S14、S16 |
| R1 | 认证 #6 | 修复流程 v0.2 §6.4 与变体 A′ 第 3 步 |
| R2 | 治理 #7 | 复核 v6 修法 4，处理死区 |
| §13.2 | 四个子代理“确认成立”的部分 | 新 |
