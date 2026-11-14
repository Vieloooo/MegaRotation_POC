#### 完整源输入与有序回放

每个正确区域成员在 session 更换后保留 authenticated source cursors、已验证 interpreter checkpoint 和 public dependency index。新的源输入一旦可用，就触发下面的 handler，而不是仅加入一个另行调度的 replay 队列。该 handler 使用既有 complete-message processing 界和原子本地状态机步骤，不改变 P5、P6 的 canonical order。

#### 共同的 record-completeness 规则

对于位置 \(p\) 的 EXPORT，其 **public dependency basis** 是严格早于 \(p\) 的 authenticated source prefix，以及该 canonical source context 指定的 protocol-authorized checkpoint。verifier 为本地回放另选较新的起始 checkpoint，不能改变这个 basis。指定 checkpoint 明确其依赖对象的 commitments 和 source identities。EXPORT 必须提供每个传递验证依赖的完整 bytes，或者以精确 commitment 和该 public basis 中的位置引用相同 bytes。被引用对象必须确实存在于这个 basis；裸 hash、URL、future authority 名称或发送者声明都不是 public dependency。

source authentication 本身不证明 regional/application validity；无论对象由 record 携带还是被引用，都仍须通过相应有效性检查。DATA reference 的既有下界以及“严格早于 containing Espresso block”的规则不变。

这个规则由 record 和 canonical public basis 决定，不能由 verifier 的私有 cache 决定。cache 可以省去对同一 authenticated object 的重复获取或验证，但私有 cache hit 不能使本来不完整的 EXPORT 变成有效。反过来，本地 cache miss 也不能使 publicly complete 的 EXPORT 变成无效：节点应获取其指定公共对象，再从原位置继续。不同 cache 状态的 verifiers 因而使用相同的 completeness predicate。

以下情况暂停 source cursor：namespace 尚未完整下载、适用 control prefix 不完整，或者解析当前 canonical boundary 真正必需的 authority 尚不可用。不能把它们解释成空 history 或 region 不 eligible。

但在 **complete namespace** 内，如果某候选未提供必需的 nonpublic proof bytes，或者仅依赖 public basis 之外的 reference，则它没有通过 record completeness。拒绝该 record，不改变 \(G\) 或 counts，并继续按 namespace 顺序处理；不能为了它去无限获取 arbitrary future material。深入检查依赖前，先检查 domain、current base、open version 和 structural append permission。closed-beta record 可以保留有用的历史证据，但可选的 archival validation 不能阻塞当前 public replay。

#### 区域 SOURCESTEP 消息

\(\mathsf{SOURCESTEP}\) 是发送给固定区域 snapshot 的 authenticated message，字段如下。

| 字段 | 必需内容 |
|---|---|
| Domain and routing | chain/protocol domain、encoding version、sender identity、接收 snapshot 的 immutable authority identity。routing 本身不选择 session。 |
| Source position | Espresso height \(q\)、完整 header \(J_q\)、finality/linkage evidence、完整相关 namespace bytes 或 authenticated absence，以及 canonical record order。 |
| Control context | 完整适用 finalized L1 control context，包括 pending records 与 completeness evidence；当前 boundary 所需且已经可用的 authority objects。 |
| Predecessor basis | 前一个 source position，以及 authorized 或 already verified replay checkpoint 的 identifier。接收者自行推导后续 interpreter states，不信任发送者提供的新 checkpoint。 |
| Dependencies | 对该 checkpoint 尚未解析的必要公共引用，提供完整 bytes 和 source evidence；normalized index 将对象连接到其 commitment 和 canonical source position。候选自身的 proof bytes 仍属于 namespace record，并受共同 completeness 规则约束。 |

每个正确成员一旦取得并认证新的 complete source input，就在该 availability event 内向自己已安装的区域 snapshot 发送 source packet，**不先等待本地 P6 replay 的结果**。它无需预先知道 replay 将选中哪个 region。packet 携带已经可用的 authenticated boundary dependencies；真正缺失的 boundary authority 仍是显式 wait，不能把它伪装成 complete step。native preparation 独立进行，不延迟 source relay。相同 packet 按 source position 和 commitment 去重，但不能因此省掉第一次 correct relay。

#### 接收、推进 ready prefix 与激活

接收者认证 packet 及完整依赖，按 source position 保存输入，并从持续维护的 checkpoint 按序处理连续 ready prefix。乱序 packet 暂存；缺失 predecessor 到达时，立即在同一个 ordered handler 中继续推进，不再等新的 polling tick 或 replay queue。

对于每个 ready block，handler 顺序执行 P6，包括 P5 prelude、共同 record-completeness gate 和 EXPORT validation。它先持久记录新的 cursor、eligibility、duty、window、boundary 和 branch state，再对外暴露结果。本地接收时间不能替换 canonical opening timestamp。

同一事件把已验证 transition 交给 Session Manager。Manager 固定由此得到的 handoff，检查当前 activation predicate；若精确 executable anchor state 和其余 Ready dependencies 已准备完成，就按 Protocol 4 原子执行 Ready 与 initial Enter。每个 pair 的 durable entry flag 只测试并设置一次。Manager 不重跑 Force，也不自行增加 beta。重复输入不能重置 window、复用旧 locks/votes、重置 entry time 或二次进入同一 pair。新的 forced version 仍不能在 triggering block 内 export。

发生 closure 时，原子持久保存 authenticated version-wide \(\mathsf{Closed}\) gate 及其 durable control-routing obligations。每个 initial entry、fresh core action 和 delayed callback 都按自己的 beta 查询该 gate，与 program phase 无关。关闭旧版的入口与新 pair 的合法 entry，不需要等待扫描 old workers、archive copy 或 retirement 完成。

按 version 索引的 control records 保存所有已知 snapshots 的义务；缺少 snapshot context 的义务保持 pending。这种组织方式**不能推迟 retirement contract 已要求的 complete-close handler 或 multicast**。任何完整 old-session context 到达时，包括 Waiting、Completed 或 Terminated record，其 receive handler 都必须先验证 context、完成并 relay closure，再作普通响应，并遵守既有区域 complete-message 界。原来的 per-session \(c_v\)、\(4\Delta\) drain 规则与 evidence duties 不变。仅记录 Closed gate 不启动 drain clock，也不授权提前退出。

#### 准备完成尾部的时间界

考虑 \(G=G^*\)、installed slot \(r=r^*\) 固定的尾部，且所需有限 authority prefix、anchor provenance 和 native preparation 都已完成并持久保留。只使用冻结模型覆盖的持续在线、跟随历史的区域参与者，并保留其此前已验证的 replay checkpoint。

设 \(J_q\) 打开 region \(i\) 的一个 retry，首次 finalized 的实时为 \(a\)。对每个 predecessor \(J_{q'}\)、\(q'\le q\)，其 finalization 不晚于 \(a\)，故完整 source evidence 最迟在 \(a+T_E\) 可用。适用 controls 已在 crossing 前可用，prepared boundary dependencies 已缓存。malformed candidate 不能借共同 completeness 规则制造新的 external wait。

立即区域发送最迟发生在这个共同 source deadline。它们的 delivery 与 complete validation processing 最迟在 \(a+T_E+\Delta_i\) 完成。ordered handler 在已维护 checkpoint 上以原子逻辑步骤处理这些就绪输入，不为每个 predecessor height 再加一轮网络等待。接着 Manager 附接已经 executable 的 anchor object，并执行 initial Enter。因此，只要目标仍可激活，第一次正确 initial entry 满足

\[
e(\beta,r,0,0)\le a+T_E+\Delta_i.
\]

若此时更晚的已验证 closure 已经生效，则进入 closure 分支。Ready 只需要 opening header，不需要一个在 opening 之后才生成的 header。该推导不添加 block-generation rate 或 lower-lag 界；growing Espresso time 与 upper-lead 条件继续在进展证明中承担各自的作用。

这一时间结论只覆盖 complete message validation 和 logical state attachment。持久 executable anchor object 可以直接附接给 fresh continuation，且不带入旧 consensus state。若保存的只是 serialized snapshot，仍需数据库 import、复制或 native-runtime initialization，则尚未满足 prepared contract；这些 external operations 必须在使用该时间界前完成。

本结论也不给 returning replica 的任意旧 backlog 一个常数 catch-up 界。具体 native adapter 必须证明自己实现了这个接口；仅把操作命名为 handler 并未证明其性能。SOURCESTEP 不增加新的 service assumption、READY certificate 或 Espresso publication obligation。
