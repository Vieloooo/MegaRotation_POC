# H1 独立来源预期（实现前）

状态：SOURCE_DERIVED / DOCS_STATIC_ONLY。基线 e1a435712ec492cae2bbff120a51ffd507eebf45；新 H1 合同在本文件首次落盘时尚未读取。没有 H1 实现候选、运行时测试或密码学黄金签名。精确来源及 hash 见 source-map.json；输入配方见 vectors.json。本文在 lead writer 交接后写入，不改 Docer 合同或生产文件。

## 1. 来源顺序与范围

v10 Session Program、Manager、v2 mock 与固定 Regional BFT/Commonware 映射给出语义；已接受的 03 canonical/registry/执行合同和 02 transport 合同给出复用边界；4a9a6b0 的 H1/H2/H3 milestone 覆盖旧 NR 草案的单 cid、R0-only、停车全交后续、自行合并等范围。NR 数值提案只用于条件化静态边界，不能冒充真实五引擎校准结果。

M1 最终是 15 健康真实 OS worker 的 R0→R1→R2，完整结果经 Publisher/SimEspresso 后 R2 首个合格结果进入 G；SessionEnd 另测。同一实际结果字节的官方本地发布/读回属于独立证据，wall/Unknown 不混入主虚拟 trace。H1 提供能力，H2 管理候选，H3 发布/重放。M2 健康主动 Force 与旧恢复/D/RESUME/批次/DA/Web不在本次运行结果内；全部 runtime NOT_RUN。

## 2. 逐边界预期

| 边界 | 触发与前态 | 独立预期 | 来源 |
| --- | --- | --- | --- |
| 身份 | 15 worker 注册，初始选中 R0 cid | 15 独立 PID；R0 五个真实 engine；10 非成员仍有 Clock/reader/root，但无 R0 签名、脚本或伪 witness。实际轮转/候选的 engine 数逐 cid 记录，不把“初始五 engine”当全程总上限。 | V2 §§2–3；H1 |
| 唯一域 | 同 session 不同 anchor | 使用 03 Descriptor/cid；engine、view、签名缓存、首收、timer、storage、work id 按 cid 隔离；pid/member/SignerIndex 显式映射。 | SP §§3.1–3.2；MPX §§2–4 |
| Create/Enter | 缺状态；重复命令；非成员；到 stop | Create不启动；重复返回原实例；精确状态未齐 WAIT；首次 Enter 一次；重复不重置；非成员/terminal/stop拒绝；早 Enter 仍等 NotBefore。 | SP §§3.4–3.5；SM §§3.7–3.8 |
| State preparation | root-only、错anchor、缺链/输入、迟到完成 | root/URL不足 Ready；需完整状态与已验证 checkpoint 到 exact anchor 的执行依据。取消/terminal后的 stale completion不能 Ready/Enter；正常未取消候选的完成可以归还原 cid，不能误给当前候选。 | SM §§3.1–3.2、3.7 |
| Pause/Resume | A→B→A，同 PID | 保留原 engine/view/lock/签名历史/首收/pending callback/绝对deadline，不再 Enter、不空journal重建。恢复先查terminal/stop；原超时到期不改为 now+timeout。 | SM §§3.9、4.2–4.3；V2 §4.3 |
| Close | Preparing/Paused/Running；work未完成 | 先持久 Terminated，再 Close 原program；全部受影响cid关闭；真实签名出口复查；历史结果及发布义务保留。TIME不自动等于Excluded。 | SP §3.6、§4.9；SM §3.10 |
| Build/NotBefore | 任意 view 与当前本地slot | h=slot(C_m)，P.h<h，固定窗口，执行padding；不拿view当height；NotBefore=max(start,ts(P.h+1))，parent末高度为∞；不凭钟补尾。 | SP §§3.7、3.9；MPX §5.3 |
| Accept | 完整proposal与原首次接收 | 5a≥ts(h)-2000µs；5b当前钟≥ts(h)；5c原首收<ts(h+1)+T_win；5d原首收<stop。5a/c/d失败永久本地拒绝；仅5b未到 WAIT(time)。WAIT保留原首收与活回调。 | SP §3.8 |
| 收包认证 | header/body乱序；认证晚完成；重复 | 最早合格header/body配对的原ingress较晚值，认证完成不重写；无权header不占首收；认证队列/预算有界，在永久判定前解决可能更早的已admitted输入。 | NR §5 |
| 时间 | ±1ms offset与高RPC delay；lease等号 | Manager/Build/engine同NodeClock；pairwise可2ms；RPC delay独立；grant不续租；只有实际有效sample续租；lease等号无新签名；Clock恢复不复活terminal。 | V2 §4；SIMT §6 |
| 证明 | 3/4票、重复、错域/kind/cid/view/subject/body | 四个不同登记合法signer的真实Finalization才可通过；Notarization不是Final；保留实际原始证书；CheckFINAL使用历史配置，不读当前钟。physical digest与native hash不可互换。 | SP §3.10、BFT-4；V2 §3.3；MPX §3 |
| 交付 | proof先到，body/ancestor后到 | 分开ProofObserved、完整验证连续交付、publication、G接纳；缺材料WAIT；证明不代替完整physical chain/执行依据；同block多proof不成多次决定。 | SP §3.12；RBFT §2.1；GFR §3.2 |
| Ack | 有效effect之后一个非法effect；未持久receipt；重复/stale | 全批拒绝，无部分可见路由/reader消费；native已授权IO不能假称回滚。真实root+aux/witness持久后release；相同grant返回原完整Ack；stale已持久义务不能丢。 | V2 §5.1；SIMT §§4–5；NR §3.2 |
| 兼容 | 准确M0及最终共享shim revision | 旧Fresh构造器/domain/编码/恢复解析语义保留；同SHA重验实际受影响矩阵；历史fec结果不继承。 | H1；NR §8 |

## 3. 精确时序向量与活性前提

条件参数：genesis height0/10,000,000µs；slot1,000,000µs；窗口1–10；start11,000,000；stop21,000,000；ΔG2,000；T_win200,000。此 T_win 来自 NR 提案，尚非真实引擎预算证明。

h1首收10,997,999拒绝5a；10,998,000等号允许，若当前10,999,999则WAIT(time)，到11,000,000为ACCEPT。12,199,999允许5c，12,200,000等号拒绝。h10首收20,999,999允许5d，21,000,000等号拒绝。首收早于stop但检查完成时当前钟已到stop，历史应用条件和实际新签名守卫必须分开：不能据Accept通过继续新签名。

错时Enter 0/20/100/500/1500ms不设全员屏障。仅就Enter而言，前四名到500ms才可形成候选的q4集合；所需状态、网络GST、持续调度同cid及预算同时成立后才可选τ_lc。3/2不同合法候选分裂没有任何cid的q4。BFT-5/6时限必须严格早于声明τ_end≤stop−δ_G；不把固定timeouts当已经证明的进展上界。

## 4. SessionEnd 两种不同验收前提

若 R0 stop21s、grace3s，公开SessionEnd键是24s；R1窗口固定11–20，h11 slot从21s开始，首收及时界为22.2s。

若场景**额外要求h11成为physical block**，必须在其合法slot/准入期间准备已知合法候选、生产并缓存；不能24s后才产h11。此额外目标不授权提前知道未来anchor/cid。

一般SessionEnd规则不要求h11 physical。例如R0最后接纳h9，后继在材料/共同参与与剩余预算满足时于local24.4s Build，可在固定窗口中产h14，并由Merge覆盖h10–13的virtual padding；所有新增高度仍检查Expire，公开完整/接纳仍依H3规则。迟到造成少于10 physical不是自动liveness反例。不得通过移到“现在起10块”掩盖错过slot。此区分来自SP Build、V2 §6.3及lead对M1b范围的澄清。

## 5. 三组接口的独立最小要求

**H1↔H2 lifecycle。** 03 descriptor/cid及稳定pid；不可变program handle；幂等命令身份与受控step/root；Create/Prepare/Enter/Pause/Resume/Close及状态结果；保留首次Enter/Close原因；全部phase关闭；ClockUnsafe/stop实际出口守卫；历史结果获取。H2决定调度/选择，H1落实真实engine停放；不能把selection booleans当公开来源。

**H1↔H2 exact-anchor准备。** anchor id/height/完整ExecutionState/执行依据；具体缺失项WAIT；request/work generation/cid绑定；取消和迟到完成语义；只在验证完成后Ready，且实际Enter再核对同一个descriptor。03是唯一域对象/编码来源。

**H1→H3证明与结果。** 实际Finalization bytes、canonical physical block/digest、完整physical ancestry、anchor evidence、历史FinalVerifier/registry/namespace profile；ProofObserved与完整交付不同事件；关闭后历史结果仍可交付。06 1MiB与02单payload64KiB须冻结审查过的分片/完整前缀方案，不截断或用本地缓存假装公开。

## 6. 必须由lead冻结的工程决议

1. **BLOCKING：本地Accept拒绝与Commonware一致性契约。** CW Automaton::verify要求honest对同context/payload一致；certify亦要求一致以保证活性；false/close均结束该oneshot。v10本地首收时间拒绝可因节点不同而不同。需显式abstention、取消及liveness处理，WAIT不能false/drop；propose返回即承诺可verify/certify，不能先返回再无论如何拒绝。没有该桥接决议不进入实现。
2. **BLOCKING：future-cid transport准入。** 02 permits静态，Rebind只改incarnation。需由合法来源Create/lifecycle触发的版本化权限接入，保留既有queue/counter/dedup/reliable义务；不得预装未知cid或重建空kernel。具体接口/owner待冻结。
3. normal namespace/profile、03 PoP域映射、epoch/view/section上界与兼容入口；黄金编码先固定，真实BLS黄金后执行生成。
4. normal完整Grant/Ack及root版本、无native artifact的合法Ack表示、multi-cid witness/terminal/admission持久绑定及实际输出顺序。
5. 输入认证预算/容量/永久结论顺序、Pause隔离的timer/work/inbox语义、数据分片上限及重新组装身份。
6. M0准确已验证SHA、串行shared-path交接、后续运行命令/profile与校准后的有界活性前提。

上述是准备阶段接口风险，非尚不存在的新H1候选代码缺陷。纸本PDF读取403，未声称复核其全部证明；以pin DESIGN和Commonware源的明确映射作为实现期望。全部运行时、重启、15-worker与M1结论均NOT_RUN，整体I1和恢复gate继续OPEN。
