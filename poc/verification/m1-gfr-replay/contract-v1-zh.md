# M1 Regular GFR 离线回放投影与验收 v1

冻结基线 `82028b212eb942e222f6f74a88ab2881322212ab`。真实输入为封存的 `healthy-smoke-91547857-seed42-run01` 及相邻 `independent-verification-91547857-seed42-run01/public-replay.json`，运行提交 `9154785732d803c1f2f69aee3b2af67220a12f58`，seed 42。依据 v10 GFR §3.2–3.4、§4.2–4.3、§4.7 和 v2 §9.1、§10.1。写入范围为 `poc/web/` 与 `poc/verification/m1-gfr-replay/`。

## 输入与投影

- 在生成端解码 POC3 L1 fixture、候选 config/profile、PELG Espresso block 及其中 GFRP 正文与证明、regional result wire。有来源歧义、截断、输入缺失、记录与独立证据不一致时停止生成。可复用原生 codec，或按固定 wire 版本实现有边界检查的只读解码器。核对证据包 manifest 中所读文件的大小与 SHA-256，绑定独立报告的 public-replay 文件摘要。页面记录源修订、seed、输入标识及关键文件摘要。
- L1 轨道显示全部 fixture header 的高度、hash、父 hash、timestamp、fixture finalized 虚拟时刻和控制列表。以 fixture finalized 时刻进入共享时间轴；逐 header reader 收取时刻显示为未记录。接纳详情单列按配置计算的最早 L1 control covering header，并区别 origin 等验证数据。
- Espresso 轨道从 PELG 取得 finalized q、公开时间键 θ、proposal tick 和 finalized tick，显示正文和证明的公开 `(q,message_index)`，逐行对应 `PublisherRecorded`。区域轨道将解码的结果按 `(cid,physical_id)` 关联 `RegionalDelivered`，保留一次区域结果的全部 replica 交付引用，时间轴显示首条观测。提供 height、region、当前 descriptor、父块、正文/证明出处。区域 BFT 的投票、消息与共识过程省略。
- G 轨道以 `GAccepted`、`GfrReplayRecord`、`Handoff` 生成连续前缀、追加范围、ReplayPosition、选定职责变化、连接 anchor、Plan reference 和 successor 固定窗口。后继职责在同一公共 Espresso 转移产生，保留迹线行顺序以供审计。G 的公共位置仅随已完成 GFR 记录前进。
- 共同横轴使用整数虚拟 μs。实际事件同 tick 按 trace seq，fixture finalized 与 trace 同 tick 如存在并列须标明显示顺序为投影约定。每个时点仅显示截至该时点的事件与状态，支持回退重算。各事件详情分开标示记录所见的 tick、协议 key 和 `q_avail`；播放速度只影响呈现。浏览器无需网络、依赖或服务。提供单文件 HTML 及可重复生成命令。
- 每条重点记录有可复制的 artifact 相对文件路径、JSONL 一基行号及 seq、PELG 消息索引（从零起）、独立报告索引。二进制和 trace 给核查者定位，页面嵌入所需证据字段。固定输入无额外生成运行。独立对照 21 条完整有序记录的高度、公用位置及 key、q_avail、cid、区域/序号、physical/parent/anchor、native/state root 和精确公开证明字节与位置。证明有效性使用封存独立报告结论。

## 规则解释

- 可点击所有接纳与两次接棒。显示处理键 `x=θ_q`，Espresso 前缀 `J_0…J_q` 已覆盖 x，L1 control 前缀覆盖到首个 `ts_L1≥x−M_a` 的 finalized header，和另需的 origin/metadata/强制前缀/Plan 输入。L1 覆盖位置是推导值。
- 显示当前选定 cid、anchor 与父链延续，`q_avail` 是证明有效且完整锚定 physical 路径正文首次公开的位置。分别展示公开截止 `θ(q_avail)<SessionEnd`、追加段逐高度过期条件 `x<ts*(h)+W_exp`、固定窗口与原生验证条件。公共缺正文与 reader 尚未读齐仅作为规则解释。轨迹中 `q_avail=q`、每次追加一高度。
- 同键全序为 L1 控制、Expire、SessionEnd、Espresso，类内按索引。满窗 regular 在接纳上界的同一 Espresso 位置关闭旧 session（COMPLETE），分支不变，r 加一。连接 anchor 是刚接纳的末端，Plan reference 是前驱的初始 anchor。后继窗口 `[old_max+1,old_max+TB]`。SessionEnd 早停作为规则说明，使用公开覆盖及旧职责仍为 NEXT 的前提，以实际已接纳末端为 anchor。

## 通过条件

| 选择 | 封存记录与预期 |
| --- | --- |
| h10／q9 | θ 20,044,000 μs，finalized 20,045,000 μs，trace 7425–7428。L1 控制覆盖 ℓ6，origin ℓ7。R0 → R1，connection anchor h10，Plan reference h0，后继 [11,20]。 |
| h20／q19 | θ 30,090,000 μs，finalized 30,091,000 μs，trace 15173–15176。L1 控制覆盖 ℓ11，origin ℓ12。R1 → R2，connection anchor h20，Plan reference h10，后继 [21,30]。 |
| h21／q20 | θ 31,084,000 μs，finalized 31,085,000 μs，trace 16037–16039。L1 控制覆盖 ℓ11，origin ℓ13。h21 首个 R2 结果接 h20 进入 G，职责仍 R2。 |

验证播放、暂停、前后单步、拖动及两次接棒跳转。各轨道同 cursor 联动并支持筛选，筛选保留已导出的协议状态。检查从 h21 回退后状态与前缀正确，接纳和接棒详情能返回证据。项目验证命令检查生成内容与原始输入，复现时输出相同字节。记录同一候选的独立验收、浏览器操作和固定场景范围。
