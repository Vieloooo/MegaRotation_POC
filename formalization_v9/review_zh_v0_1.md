# v9 v0.1 表述检查记录

日期：2026-11-02。

v9 初稿完成后，由三个独立的检查代理分别通读全部文件，各自只看一个方面：

- **A 记号**：记号是否清楚、一致、自足；每个记录是否只定义一次；交叉引用是否正确。
- **B 可读性**：是否同时给出直觉与形式定义；阅读顺序是否顺畅；例子是否与规则一致。
- **C 可组合性**：每个接口输入是否有产生者、每个输出是否有使用者；两侧类型是否一致；端到端流程是否能按规格逐步对上。

下面按主题汇总检查发现和已做的修改。文件编号指 v9 中的文件。

## 1. 可组合性（C 为主）

| 问题 | 修改 |
| --- | --- |
| Force 与 RESUME 的 Plan 输入（Ref、来源区域、session 序号、窗口）未定义，各方无法算出同一份 $\Pi_s$ | 09 §4.6 为 Force 与 RESUME 各给出完整 Boundary；RESUME 新 session 为 $(\beta+1,0)$、来源区域 $\bot$、窗口 $[h_{\rm res}+1,h_{\rm res}+\mathrm{TB}]$；Force 增加第 6 步（Plan、SessionAssigned 或 PENDING、从 $J_{q+1}$ 起接纳）。09 §4.5 写出常规交接的 Boundary。01 §3.2 说明三种情形下 Ref 的取值 |
| Manager 侧没有接口计算后继 Plan；W14 无调用点 | 03 §3.10 的 SessionAssigned 处理中计算后继 Plan；03 §3.4 ObservePublic 增加调用 `Responsibility` 与 `Chain` 的步骤 |
| 跨区域转发区域结果没有接线 | 新增 W26；03 §3.6 HandleProgramEvent 转发给后继区域，并说明可从 GFR publication 索引组装同一结果作为后备 |
| ProgramEntry 不保存 $\pi_A$，Enter 与组装 Res 无处读取 | ProgramEntry 增加 anchor_evidence |
| RESUME 的 anchor 状态没有到达 Manager 的路径 | 03 §3.10 的 Resumed 处理调用 `DeriveAtResume`，把 anchor_state 存为 StateMaterial，等 SessionAssigned 后再 PrepareCandidate；10 流程三第 14 步同步 |
| REGIONAL／FORCE anchor 的状态材料没有来源 | 03 §3.7 写明状态材料来源（$D$ checkpoint + $G$ + 候选链）；Manager 加入 W15 使用者 |
| 证据路径类型不符：SubmitOffense 需要 pid，EvidenceTask 没有；无人触发提交 | EvidenceTask 增加 accused_pid；04 新增 `QueueEvidence`（§3.6），SubmitEvidence 移到 §3.7 并写明调用 `SubmitOffense(ν, accused_pid, ev)`；新增 W28 |
| Batcher、Certifier 的本地 GFR 处理 RESUME 需要 resumeReader | W23 使用者扩展为各处本地 GFR；05、06 上下游表加入 W23、W19 |
| G 判死“更高前驱块已进入 $G$”没有事件承载 | 03 §4.4 写明由 GFR.`Chain` 读出 |
| 接口连接表与各模块表不一致 | W16 改为 Batcher→DA；新增 W27（CertificationRequest）；W20 去掉 Authority（合约内直接读 InboxState.cursor）；W21 加入 Certifier；W13 加入 Publisher；W25 的产生者改为新增的 `InputRef`；各模块上下游表按新编号对齐 |
| CertificationRequest 含 prefix_frames，但 CertifyBatch 只有四个参数 | 删除该字段，写明 Certifier 从 L1 接纳事件与 DA 自行取得 frame 前缀 |
| W25 没有产生接口 | 09 新增 §5.6 `InputRef` |
| “安排不能启动”不产生事件 | 09 统一规则输出 SessionClosed（EXPIRED） |
| $G'$ 物化需要历史 $G_{\mathit{gpos}}$，但 Chain 只返回当前 $G$ | 07 §3.7 说明在两次 RESUME 之间可用当前 $G$ 读取 |
| 流程三按重放顺序漏掉先于 Expire 的 SessionEnd | 10 流程三第 8 步补上 SessionEnd、regular anchor 108 与后续 session 的处理 |

## 2. 记号（A 为主）

| 问题 | 修改 |
| --- | --- |
| 10 中不带文件前缀的“§4.4／§4.5”指向本文件的其他小节 | 全部改为 `[09 §4.4]`、`[09 §4.5]` |
| SessionEnd 关闭的原因不一致（TIME／COMPLETE） | 统一为 COMPLETE；03 的 CloseReason 说明区分 TIME（本地时钟到 $\tau_s^{\rm stop}$）与 COMPLETE（GFR 公开完成） |
| 区域名 A／B／C 与 anchor 记号 $A$ 冲突；`cid(A)` 违反 cid(·) 的定义 | 10 中区域改为 a／b／c，`cid(A)` 改写 |
| 00 §4.6 漏列 CloseReason | 已补 |
| `Deliver(y)` 与其输入说明不一致 | 改为 `Deliver(y, prev, channelInfo)` |
| BuildPayload 的区间 $I$ 与证书完成范围 $I$ 同名 | BuildPayload 的区间改为 $J$；返回标签 `Payload` 改为 `Built`，避免与 Builder.Payload 同名 |
| $\tau_{\rm del}$、$D_L$ 中的 $L$、$\mathcal X$、$\mathcal D$ 未定义或未链接 | 00 §2.4 定义 $\tau_{\rm del}$；09 §4.6 定义 $L$ 并说明切面绑定 $p_{\rm res}$；03、05 链接到 09 |
| anchor 被直接赋值为 virtual block | 09 §4.6 写明取对应 Anchor，连接标识按 00 §3.2 计算 |
| WAIT 有三种写法 | 流式接口统一为 `WAIT{next_position, missing}`，00 §4.2 说明 |
| “position”同时指质押登记和 L1 位置 | Authority 输出、ControlEntry、Inbox 事件与 Batcher 交付结果中的 L1 位置字段改名为 L1_position |
| Boundary.cause 的 REGULAR 与 AnchorEvidence.kind 的 REGIONAL 易混 | cause 改为 HANDOFF |
| `DeriveAtResume(position)` 的位置有歧义 | 改为 `DeriveAtResume(p_res)`，$p_{\rm res}$ 为 RESUME 控制记录的位置 |
| 链名被当作 height 使用（$R=115$ 等） | 改为 $h_R=115$ 等 |
| 00 §4.2 规则版本记号 $\mathit{ver}$ 与 rule_version 不一 | 统一为 rule_version |

## 3. 可读性（B 为主）

| 问题 | 修改 |
| --- | --- |
| 流程三的时间例子与压缩的高度数值矛盾 | 说明时刻只用于计时规则，与高度数值分开；$h_{\rm res}$ 改述为“固定时间戳为 22:21” |
| 流程二的起点 $G=100$ 与流程一的结束状态不衔接 | 流程二改为“设 a 已决定 100” |
| 例 A 复用 (7,20)、(7,21)，窗口与流程一冲突 | 改用 (7,30)、(7,31) 并给出窗口 |
| sequencing window 与“窗口回退”没有直观定义 | 00 §3.4 末段重写，先讲直觉再给 $\mathrm{ddl}$、$W_{\rm seq}$ |
| 09 §3、§5 缺少开头的动机 | 各加一段直觉说明；Tail 各值加指向规则的链接 |
| 预算不等式各项含义不清 | 09 §4.3 逐项解释 |
| 09“统一规则”难以解析 | 重写 |
| 02 同步起点的条件挤在一句 | 先给直觉，再把四个条件列成列表 |
| DA-RET 混合了 DA 保证与保管假设 | 08 新增 A-CUSTODY 假设，DA-RET 改为两部分的组合保证 |
| PUB-5 缺少持续在线前提 | 已补 |
| 10 中 $T_{\rm prep}$ 与 01 中的含义不同 | 改为 $T_{\rm lead}$ 并就地定义 |
| 00 中残留英文与指向 v8 源文的措辞；DA 直觉段重复 | 改为中文；删除“沿用区域 BFT 的…”“沿用 Regional BFT abstraction 的顺序”；删去 00 §1.5 与 08 重复的直觉段 |
| 03 Kill 判据的“或者”分句是第一条的特例 | 重写为“必须确认原生 hash 不同；典型情形是…” |
| 06 恢复对齐措辞与 CERT-4 表述绕 | 重写 |
| TB 未解释 | 00 §2.2 补“每个 session 可使用的 height 数量” |

## 4. 未采纳或留待后续

- **派生状态中的 position 字段。** DerivationState.position 表示派生处理位置，含义在 09 §5.2 已定义，未改名。
- **v0.1 来源说明。** 00 开头说明本版如何由 v8 拆分而来，保留在原处，供读者对照。
- **G3.3 的 leader 序列约束、Builder 的公平选取。** 这两项是性质本身的已知空缺，本次只检查表述，没有补充新假设。
- **20 分钟恢复预算与 10h／12h 参数。** 二者沿用 261027 的示例值，仍需按实际参数校准。
