# M5 自动 RESUME 离线回放投影合同 v1

基线 `e44b6e8e77aeb1aa3895c14109a57ff3894d7252`。规范来源为 M1–M5 计划 §6/§7、v2 §8.5–8.6、v10 GFR §3.2–3.4、§4.2–4.6、Session Program BFT-7 与 M4 的精确 Cut 合同。M5 后端候选冻结后按其具体 profile 和真实事件补全字段映射。写入范围是既有回放 `poc/web/m1-gfr-replay/` 与 `poc/verification/m5-auto-resume-replay/` 的投影、校验和文档。

## 输入和范围

- 正式页由已独立验收的 M5 封存候选生成。核对 manifest SHA、完整候选 SHA、seed/profile、逐文件大小/摘要及独立 checker PASS 的 run 绑定，保留原 checker 输出。所采用的每个观察或证据可定位到原始文件、seq/行号、消息或日志索引。显示推导值时单列规则及输入。原始证据不完整、不一致时拒绝替换已有页面。
- 样本与正式页各有独立输出路径。样本显著标识计划参数、合成记录与规则推导。样本包含 finalized 停止、q 增但 θ 不动、恢复后的 quorum、成功恢复，以及缺覆盖或无整数目标等待的演示。正式页按已封存场景展示实际对照，可用独立对照页或同页切换复用时间轴。复用 M1–M4 既有离线交互和语义。

## 时间、排序和恢复

- 区分虚拟 `tick_us`、各 replica 自己的 `C_m` 与 stop 阈值、L1 timestamp 和 tx/log 位置、Espresso 提议/actual finalized/公共 θ，及 GFR 公开 ReplayPosition。finalized 停止以已观测 q/θ 与服务事件展示。冻结 θ 的场景保留 q 及仍可处理的等键 Espresso 记录。后续键缺 watermark 时停在其前，显示被阻塞的事件、所缺覆盖与最后已完成 pc。L1 与派生 D 的独立增长，以及 G/G′/D 各自高度、来源与共高 native hash 分开显示。
- 本地 stop 由本地 `C_m ≥ ts*(h_max+1)` 与 gate/terminal 证据表示，保留已形成签名和其后结果的位置。公开 SessionEnd 键另加 grace，Expire 键是 `ts*(h_G+1)+W_exp`。GFR 的定时器须完整覆盖，顺序为时间键、类别（L1 → Expire → SessionEnd → Espresso）和类内索引。公共 Expire 关闭旧分支并置 HALT。WAIT、local stop、公共 Close/HALT 和恢复后首次 replay 分列。有晚到旧输入时核对其实际处理结果。
- 展示各 signer 自己的已公布与已验证 L1/Espresso tip、上次 q/θ、已处理位置、自己的 G 与 D/Inbox 视图、仍缺输入或等待原因、clock readiness、public HALT 或同高度实际 native hash 分歧，以及可行的候选请求。HALTED_D_ADVANCED 同时需要公共 HALT 与满足规则的 D 越过冻结前缀，HASH_DIVERGENCE 要比较共同高度。旧节点停机或 D 高度变化各自只作为观测。没有恢复为 live 的排序水位、必要路径未就绪或已有在途请求时显示 guard。
- 以同一请求的独立签署门槛定义 `t_eligible`，保留每名 signer 首次可签记录及实际签署/入包。预算展示 `t_submit−t_eligible ≤ T_poll+T_sign+T_broadcast`，明示实际采用的阶段时间。若只能以轮询记录给出首次已观察到 quorum 的时点，标注观测边界。
- 求解时保留 G/D/Inbox heads、收录前推进的依据和 `H_guard`、预计收录 timestamp 区间 `[t_L,min,t_L,max]`、`T_prep,T_drift,ε,M_a,T_ready,δ_G,b_L2` 及实际高度网格。复算最小整数 `h > H_guard`，满足 `t_L,max+T_prep ≤ ts*(h) ≤ t_L,min+T_drift−ε`，检查两条准备约束。无解时显示界限和拒签或等待。核对实际签名 payload 的目标/branch/policy/seq、digest、治理成员与入包次序，区别区域 Final 门槛。
- 分列签名包、持久准备、广播、收录/收据、finalized 与 reader 交付。用实际收录 timestamp 得 `act`，单列与原收录预算的对照。精确 `ResumeRecorded` 与同交易 CursorSet 的规范位置和 RPC logIndex 分开。用 M4 规则核对历史 Cut 的摘要、D(h_res) 的真实 origin、连接 ID/native hash/post-state root，以及 live D 可在 Cut 之上。公共同键操作按序执行，原子对齐后可启动或 HALT 以真实事件为准。恢复后 worker Ready/Enter、Regional Final、发布 `q_avail` 与首个 G 接纳各有位置/时点。

## 验收

- 冻结/恢复、公共 WAIT、各 signer、求解、收录、act、Cut 和首块阶段可点击。共享虚拟时间游标支持播放/暂停、前后单步、拖动和筛选。每个公共完成转移只有一帧可见新状态。同一观测拆分的信息标示共享原始 seq 和推导顺序。
- 独立测试覆盖 q 增 θ 不动、缺覆盖时拒绝提前推进、local stop 与公共 HALT、不同请求不合成 quorum、两个收录端点和等号/无解、迟到收录后仍按实际 act 处理、错误 Cut/anchor、撤换原子性、M1–M4 的原生成和投影。正式页用通过独立验收的主场景及已提供的对照重新生成，核对证据哈希，浏览器检查关键阶段和时间区分。记录 SHA、生成和验收命令、实际数据范围及需要额外日志的字段。
