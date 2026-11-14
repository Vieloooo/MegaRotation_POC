# Formalization v4：单篇 Sequencer Rotation 稿件详细提纲

日期：2026-10-25。本文是写作蓝图，不是已经完成的协议正文或证明。
目标产物为本目录未来的单一 `main.tex`：正文、附录、协议块、图代码和参考文献均在同一文件内。
本轮先完成中文提纲；节标题附英文候选，便于之后沿用 v3 的写作风格。

## 0. 组织原则、来源与阅读方式

新稿以 [v3 Part II v4.9](../formalization_v3/sync_hotstuff_rr_part2_rotation_v4_9.tex) 为骨架，吸收 [Part I](../formalization_v3/sync_hotstuff_rr_part1_core.tex) 的核心协议与必要证明、[Part III v6](../formalization_v3/sync_hotstuff_rr_part3_staking_v6.tex) 的 Authority 状态机，并整合 [组合式抽象 v2](../formalization_v3/composability_abstraction_v2_zh.md) 的完整系统边界与全局回放规则。
[Part IV v0.1](../formalization_v3/sync_hotstuff_rr_part4_force_v0_1.tex) 只提供 Publisher 的转发、付费与保留思路；其旧 Force/authority 接口不再单独成为正文的一层。
mega-espresso 指本仓库 `megaeth_specs` 的现有规格，特别是 block-certification、batch inbox、derivation 与 recovery；本文不把其中未激活的草案说成线上行为。

### 0.1 总—分架构

先回答“整套协议如何让 L2 形成一条不可回退的链”，再回答“各参与者怎样完成其中一步”。
建议正文顺序如下：

| 部分 | 章节 | 读者应得到的理解 |
| --- | --- | --- |
| 总览 | §1 | 正常出块、常规轮换、Force、停机恢复怎样组成同一条数据流。 |
| 建模与公共对象 | §2–3 | 外部服务假设是什么；块、session、context、证据和状态属于谁。 |
| 正常协议 | §4–7 | Authority 如何授权，区域引擎如何形成结果，公开规则如何接纳，Manager 如何交接。 |
| 恢复与结算 | §8–9 | 如何停用 Espresso 路径、接续 deposit-only、恢复出块，以及如何进入 DA/L1。 |
| 保证与范围 | §10–11 | 哪些性质得到证明、依赖什么前提，哪些保证没有声称。 |
| 同一文件中的附录 | A–D | 完整接口字段、核心证明、组合证明、参考控制策略与边界例子。 |

正文不直接照搬上一稿 46 个接口小节；行为由协议块组织，内部接口的精确字段集中在附录 A。
每个内部接口仍采用用户要求的格式：第一份 itemize 列输入，第二份 itemize 列输出，随后一段说明功能。
L1/Espresso 不再各有 ideal-functionality 或 Submit/Read/Verify 协议块；正文只给出它们被本协议使用的假设。
TEE 只作为可信第三方 $\mathcal T$，不描述 measurement、密钥注册、远程认证或硬件 API。

### 0.2 单文件与“完整”的含义

`main.tex` 必须能独立阅读：不能用“详见 Part I/III/IV”替代关键规则、前提或证明。
可以引用 Sync HotStuff 等外部工作说明来源，但本协议的加权门槛、有限 session、DRAIN、Completion、Force/STOP 与恢复适配必须在本稿内交代。
TikZ 图直接写在 `figure` 环境内；不依赖原目录的 `\input{figures/...}`，参考文献使用同文件的 `thebibliography`。
标准 TeX 宏包属于编译环境，不计作另一个协议源文件。

“单篇完整协议”与“完成所有外部系统的实现证明”不同：原生执行、DA 和状态证明系统可有清楚的外部正确性假设。
但共同切换边界 $\mathfrak B$ 与受保护前缀的交付策略属于本协议自己的设计，不能改名为“L1 假设”后省略。
定稿前必须完成它们的具体化；若保留参数，则总定理和标题范围要明确为条件化规范。

### 0.3 标识与状态

- `P-章节.编号`：目标稿中的一个计划自然段；下面逐段给出该段要完成的任务。
- `A-01…A-16`：协议块，完整写作卡见“协议块逐项设计”。
- `F-01…F-13`：图，布局、箭头、caption 要点见“图逐项设计”。
- `T-01…T-12`：表，字段与用途见“表与正式陈述清单”。
- `I-01…I-38`：内部接口，附录 A 的每个 I 项对应两份列表及一个功能段。
- `AS-01…AS-08`：外部假设；`D-01…D-12`：公共定义；`R-01…R-10`：目标结果。
- `待闭合 D1…D4`：作者工作标记，不直接写进最终论文；工作内容见末尾。

### 0.4 贯穿例子与术语纪律

统一采用 A/B/C 三个区域、$\mathrm{TB}=10,K=4$，前驱 A 的 session $(7,20)$ 从 DATA 100 开始，terminal 为 110。
普通生产分支 $R$ 可已到 B 的 111–115，而全局 $G$ 仍只到 108。
后续展示两个**互斥的示例分支**：普通 Force 从 108 开新 session；或者 STOP 保留 108、deposit-only 推进到 120、RESUME 从 120 生产 121。
不能把两个示例的 $\beta,r$ 没有更新地拼成同一次执行。

沿用 $(\beta,r,g,t),j,\Pi,\mathcal P_\beta,C=(\Pi,A),\mathrm{cid},\nu,\sigma_G,\mathsf{Tail},G,R$。
$E_s$ 专指 selection reference，进入凭证写 `EntryEvidence`；真实时间用 $\tau$，term 用 $t$。
staking 的 RequestExit、共识代的 Quit、session 的 Completion、普通 Force 和模式 STOP 必须分别命名。
DRAIN 不是 deposit-only；$\lambda(B)$ 不是原生 `epoch_num`；global envelope hash 不是 native L2 hash。

---

## 1. 摘要与正文逐段提纲

以下段落号对应未来稿件，不要求把说明文字逐字搬入论文。
标有“插入”的正式块放在指定段落之后；其定义/输入输出在后续清单中展开。

### 摘要 / Abstract

**P-0.01｜唯一摘要段。** 用四个连续内容单元写出：多区域 sequencer 轮换的问题；固定 session 的区域共识与公开全局接纳的职责划分；L1 Authority、显式模式切换和 mega-espresso 结算路径；在列明前提下的前缀安全、生命周期隔离和条件性进展。说明 TEE 是可信第三方，不宣称已经证明 UC realization，也不把任意 Espresso 安全性失效纳入本稿。

### §1 总览 / System Overview

#### §1.1 问题与系统全貌

**P-1.01｜协议要提供什么。** 从外部观察者出发，说明需要唯一、连续、不可被轮换或恢复推翻的 L2 链 $G$。区分“谁当前生产”“区域结果形成”“全球接纳”“L1 状态结算”，避免一开始用模块名代替系统目标。

**P-1.02｜完整数据流。** 顺序说明用户交易进入 builder，session program 形成区域回执，publisher 送 Espresso，观察者执行 global rule，batcher/certifier 将受约束输入送 DA/L1。并行说明 Authority 提供权重和控制；插入 F-01 与 T-01。

#### §1.2 正常运行与显式控制

**P-1.03｜正常运行的快路径。** 说明 region 内使用同步共识，regular successor 可提前取得 terminal receipt 和状态，普通轮换不要求每个 session 发一笔新的 L1 授权交易。此处只讲因果关系，不提前展开投票细则。

**P-1.04｜两种接棒。** 常规交接继承前驱 terminal DATA，Force 继承公开 $G$ 的精确末端。指出 $R$ 可领先于 $G$，Force 关闭的是旧版本的全部程序和未接纳后缀，不允许选择更旧的 retained anchor。

**P-1.05｜停机期间的另一条推进路径。** 说明 Espresso liveness failure 后由 multisig 显式公告并提交模式边界，deposit-only 使用 L1 派生，恢复后从真实末端开新 session。明确强 $G$ 中已有普通块仍需保留，不能为恢复速度而替换。

#### §1.3 贯穿例子、贡献与路线

**P-1.06｜数字例子。** 给出 100、108、110、115 的关系，展示 ordinary Force 与 STOP/RESUME 两个互斥后续。插入 F-02；例子中每个块的 producing session 保持不变。

**P-1.07｜本稿给出的对象。** 说明本文统一给出 Authority 状态机、session 子协议、Manager 生命周期、公开最终化、故障模式和结算连接，并在同一文件给出必要证明。描述贡献时只声称已经写出的规则和实际证明范围。

**P-1.08｜阅读路线与范围。** 指引 §2–3 的前提/对象、§4–9 的协议、§10 的保证和附录的完整细节。说明这是一份组合式协议规范；外部服务用假设表达，内部切换逻辑不能被当成外部 oracle。

### §2 模型、假设与目标 / Model, Assumptions, and Goals

#### §2.1 参与者、时间与故障范围

**P-2.01｜参与者与执行环境。** 定义用户、stakers、regional replicas、multisig、publisher、batcher、可信 certifier 与外部观察者；一个运营者可兼任角色。定义 chain/rule version、全局时钟、L1 高度/位置、Espresso 高度/时间戳。

**P-2.02｜故障范围。** 保留网络同步和区域固定权重故障界；仅让 Espresso 的新记录最终化活性在 outage 中失效。已 finalized 历史安全性、外部数据保留和其他服务是否继续有效分别说明，不讨论 partial safety。

#### §2.2 八项外部假设

**P-2.03｜AS-01：L1。** 一项命名假设涵盖 finalized L1 状态/事件的一致性、持久性、可认证完整读取，以及正常条件下已准入操作的处理、最终化与可见性界 $T_{\rm L1}$。处理成功/失败都可有最终结果；该界不包含委员会决定或收集签名前的时间。

**P-2.04｜AS-02：Espresso。** 一项命名假设涵盖 finalized blocks 的公共顺序、完整 namespace/空 namespace、持久性和可认证性；健康期间另有 $T_E$ 与 header/time progress。给出 $\theta_q\le\tau_E(q)+\epsilon_E$；将安全条款 AS-02-S 与健康期活性条款 AS-02-L 分开，以便 outage 定理只保留前者。

**P-2.05｜AS-03：认证同步通信与时钟。** 规定正确在线节点间区域内 $\Delta_i$、跨区 mixed p2p 骨干 $\Delta_{\rm cross}$，包含限定大小消息的完整依赖验证处理。说明跨区骨干保证的对象和正确路径条件；公共时钟不等于同时收到来源数据。

**P-2.06｜AS-04：密码学与区域权重故障界。** 定义不可伪造签名、binding hash、$F_s<W_s/2$ 及被接受证书生命周期中的累计 compromised weight。点出 $>F_s$ 保证至少一个正确 signer，以及正确总权重 $W_s-F_s>F_s$；不能套用经典 $2/3$ quorum intersection 论证。

**P-2.07｜AS-05：DA、保留与可恢复状态。** 分开声明 commitment 绑定、已接受字节的取回条件和长期保留责任。只在规定大小/负载与 prepared-node 条件下承诺状态准备；任意离线 catch-up 没有统一时限。

**P-2.08｜AS-06：multisig 控制服务。** 假设其授权、协调和未来公告纪律正确；普通 Force 保留 $\theta_a>\tau_{\rm submit}+T_{\rm L1}+\epsilon_E$。诚实不自动意味着及时响应或永不打断正常服务，恢复期限的额外响应条件留至进展定理。

**P-2.09｜AS-07：可信 TEE。** 将 $\mathcal T$ 当作可信第三方，直接信任其有类型的 BATCH、MODE、CLOSURE 消息；来源真实性由模型给定。说明 L1 合约因此不需要验证 Espresso header，Espresso 也不验证 L1；TEE 不自动扩张为 state-root 正确性的保证。

**P-2.10｜AS-08：原生执行与状态证明。** 给出确定性原生执行及 Kailua/fault-proof 的外部 soundness 边界。所有节点使用同一 rule version、origin、deposit 与系统输入规则；本稿不重写 EVM 或凭空补齐子规格仍为空的证明系统实现。

插入 T-03：每项假设中的安全条款、活性条款、适用模式和消费模块。
这里不出现三个外部 API 的独立规范，也不出现 $\mathcal F_{\rm L1}/\mathcal F_E$ 的 ideal-functionality 算法。

#### §2.3 要证明的性质

**P-2.11｜安全目标。** 用序列前缀关系表述诚实观察者的 $G$ 一致性和不回退，用精确块/来源元数据表述不可替换性，并声明 session 隔离及无错误处罚/安全释放目标。原生结算对应性作为单独目标，不能由 append-only 代码自动推出。

**P-2.12｜进展目标。** 分别提出健康 NORMAL 的高度进展、STOP 后 L1 派生进展、RESUME 后普通服务恢复，列出各自需要的服务和控制条件。有限 session 的目标是完成 quota/形成 terminal receipt，不是在单个 session 内获得无界 DATA 高度。

**P-2.13｜非目标与必要限定。** 不承诺区域间公平、每笔普通交易的无条件包含、任意治理行为下的进展或任意数据丢失后的无损恢复。由 L1 派生的 forced-inclusion 路径按原生规则继承，不重新加入区域共识的公平性机制。

### §3 公共对象、状态归属与接口约定 / Shared Objects and Contracts

#### §3.1 标识、上下文与固定配置

**P-3.01｜四层坐标。** 定义 $s=(\beta,r)$ 与 view $(\beta,r,g,t)$、fresh position $j$，解释哪些事件改变哪一维。给出 D-01 与 T-02 的坐标部分，并引用 F-03(a)；整张 F-03 放在 P-3.06 后。

**P-3.02｜policy 与 plan。** 定义 $\mathcal P_\beta=(S_\beta,\mathcal L_\beta,\lambda_\beta)$、$\Pi_s$ 的字段和 immutable 性；说明 $E_s$ 用于选择，$A_s$ 用于执行。这里只定义数据类型，具体 Plan 算法放 §4。

**P-3.03｜完整 context。** 定义 $C=(\Pi,A)$ 与 $\mathrm{cid}=H(\mathrm{id}(\Pi),H(A))$，列出被绑定的 chain、session、region、固定权重和初始 anchor。给出 D-03；提前说明“由真实 handoff 导出的 context”和“账本相对验证的 claimed context”用途不同。

#### §3.2 区块、引用与 quota

**P-3.04｜DATA、DRAIN 与原生投影。** 定义 normal envelope、native block 和两种 hash 类型；一个 DATA 对应一个完整 L2 块，可含多个 mini-blocks。DRAIN 无新普通输入、不增 DATA 高度；deposit-only 是另一种全局块来源，不能复用 DRAIN 名称。给出 D-02。

**P-3.05｜L1/Espresso references。** 精确规定 $\rho,\lambda(B),\xi(B)$ 的绑定、沿 ancestry 单调和 opening floors。解释引用并不表示本块的 commit time，也不改变 historical committee；首次 DATA 的 Espresso header 可以晚于 Ready 才可得。

**P-3.06｜quota、位置与 endpoint。** 写出 $H_s=h(A_s)+\mathrm{TB}$、$m_{s,g,t}=\min\{K,H_s-h(A_{s,g,t})\}$ 及 DATA 的 $h,k$ 公式，单独定义 recovery at endpoint 的 drain generation。插入 F-03(b)，用 $K=4$ 的 101–104、105–108、109–110 展示最后短 term。

#### §3.3 消息、证书与 admission

**P-3.07｜签名字段与证书。** 给出 D-04 和 T-04：PROPOSE、fresh vote、STATUS、NEWVIEW、ANCHOR、BLAME、FINAL 各绑定什么、quorum 权重如何去重。明确 FINAL 无 $g,t$，DATA/DRAIN 共用 fresh-vote 槽，proposal QC 不是 FINAL。

**P-3.08｜opening、rank 与 parent justification。** 定义 $Q^0_s,Q_v,Q^*$、QC rank 与 recovery opening identity。插入 T-05，分别覆盖 gen-zero 第一块、恢复第一块、term 后继、普通后继和 DRAIN parent；不得继承另一个 session 的锁。

**P-3.09｜admission 和完整 quit proof。** 给出 D-05：gen-zero boundary、recovery entry chain、先前 terminal QCs 构成的 admission witness；列出 BLAME、proposal 双签、NEWVIEW 双开口三种 quit proof。区分相对结构/签名检查与完整 regional/native validation，以供 §4 的 L1 相对验证引用。

#### §3.4 handoff、Ready 与完整结果

**P-3.10｜handoff descriptor。** 定义 $\nu,\omega$、Handoff 字段、固定 $\lambda^{\rm open},q^{\rm open}$ 与原 producer 的 anchor evidence。D-06 同时允许 genesis、regional FINAL 和经 L1 派生确认的 deposit-only anchor。

**P-3.11｜Ready 与初始进入条件。** 将精确执行状态、固定 plan、source proofs 和 lifecycle guard 合成 Ready；完整 ReadyBundle 含可直接使用的状态而非 URL/root。Ready 不要求尚未出现的 opening-floor Espresso header 已存在，提议第一块另检查它。

**P-3.12｜完整区域结果。** 定义 DATA、direct/covering FINAL、完整路径和 producing context 的组合，给出 D-07。说明 covering 不跨 session，继承 anchor 不改生产身份，也不由新 session 重签 FINAL。

#### §3.5 状态归属与通用调用纪律

**P-3.13｜三个主要状态拥有者。** Authority 拥有资金与授权历史，Program 拥有单 session 的 votes/locks/timers，Manager 拥有生命周期、全局回放与 prepared states。说明 Publisher/Batcher 的交付进度和可信 certifier 的校验状态，不创建“global consensus module”；对照 T-01。

**P-3.14｜缺失、无效和幂等。** 统一 VALUE、WAIT、REJECT、verified $\bot$ 与 INCONSISTENT 的含义，规定持久化先于外部发送、稳定 request/object ID 和重复调用行为。附录 A 保存完整接口字段；本文协议块不得以 `deps` 或省略号隐藏关键输入。

### §4 合并的 L1 Authority / Staking and Control Authority

#### §4.1 状态与 genesis

**P-4.01｜账本 state。** 用 T-06 完整列出 Pos、KeyPosition、两条 FIFO、ReqResults、每 epoch 快照、Ban、REC、SeenOffenses、控制日志、Closures 与 Clear。每项注明 key、value 和单调/不可变条件。

**P-4.02｜初始配置。** 列出固定资产、最低 bond、成熟期与队列预算、$a_0,M,N,\rho_{\rm slash},T_{\rm hold},\mathrm{TB},K,\Delta_i,\varphi_i$，以及初始 positions、$\mathcal C_0,\mathcal P_0,A_0$。后来的 checkpoint 恢复完整状态，不能重新清空 liability 表。

**P-4.03｜权限与时间。** owner 管资金请求，任意参与者可报有效证据，multisig 管控制，TEE 报有类型业务结论。L1 顺序原子处理请求，finalized 后才供节点使用；请求完成和分配 duty 的时刻分别说明。

#### §4.2 成员生命周期与快照

**P-4.04｜Join、Cancel、RequestExit。** 讲明 region、公钥、提款 owner、实际资产数量、PoP/ownerAuth 的含义，取消只限未被任何冻结快照使用的 position。退出请求不解除当前 duty；引用 I-04–I-06。

**P-4.05｜epoch 权重。** 定义 $a_e=a_0+eM,c_e=a_e-N$，快照取严格早于 $c_e$ 的完整历史。分别处理强制退役、入队和自愿退出预算，冻结 $w_{e,o}$，发布高度独立记录；插入 A-01 与 F-04。

**P-4.06｜状态转移的细节。** 说明低 escrow/offense 强制移除，cutoff 后新请求保留，未处理 exit 的旧成员保持 Exiting；迟 cut 不重算内容。用一笔 slash 后 escrow 下降但旧权重不变的小例子结束该段。

#### §4.3 Plan、Auth 与 Leader

**P-4.07｜Plan 的读取切面。** 写出 $\ell=\max\{\lambda(E),\lambda_\beta\}$ 和 $e(\ell)$，选区使用 fixed cyclic list、ON 状态与非零 eligible proposer weight。missing snapshot 返回 WAIT，不能选旧快照或将缺失视为零。

**P-4.08｜固定委员会与代 authority。** region 确定后包含该快照全部正权重成员，ban 只缩小 proposer 集，不改投票权重；REC 以 $(\mathrm{cid},g)$ 唯一。定义 source height 与 plan ID，regular successor 提前一 session 计划。

**P-4.09｜选主规则与代价。** 给出 v6 的 WeightedRand seed 坐标及 position-ID 排序规则，说明 public/predictable schedule、空 eligible 集和晚生效 ban。schedule fairness 是进展前提，不从哈希函数或 staking 自动推出。

#### §4.4 Quit、offense 与相对验证

**P-4.10｜claimed context 的验证范围。** L1 检查真实 snapshot、固定参数、同 cid 的签名和 slot，但不裁定 speculative session 是否 canonical，也不读 Espresso 来重放 handoff。解释伪造 context 不能使用诚实 key 在另一 cid 下的签名。

**P-4.11｜slash 的精确定义。** 用 T-07 给出五类 offense 槽与冲突内容；金额为 $\min\{\rho_{\rm slash}w_o,B_o\}$，去重键为 position/type/slot。timeout、BLAME、OFF、普通转发不受罚，旧 epoch 权重不改。

**P-4.12｜恢复记录。** 所有合法 quit proof 都创建或复用下一代 REC；包含双签时先处罚/ban，再固定记录高度对应的 ban 集。为 A-02 的证据处理分支给出依据，强调裸 counter 不能抢占恢复槽。

#### §4.5 控制、清退与释放

**P-4.13｜两类控制记录。** 普通 $U$ 含完整 $S,\mathcal L$ 和 Espresso 阈值；mode 记录含 STOP/RESUME、L1 截止/最早激活条件与 boundary。控制日志接收进度、pending 和实际 Done 结果分开；普通 stale update 整体 no-op。

**P-4.14｜可信关闭确认。** 说明 ProveClosure 消费 TEE 的 CLOSURE，普通 Force 与 STOP 的实际生效依据不同；不再保留 L1 Espresso light-client 路径。future floor 不能降低，清退 epoch 从已确认 floor 得出，不能仅凭公告释放。

**P-4.15｜安全释放与等待。** 把 retired、cleared、hold、released 分开，说明先处理已接受处罚再一次性支付余额。停机持有期不能沿用含健康期 $T_E$ 的旧公式；插入 A-02 的 release 分支，并将新的预算推导列为 D4 工作。

### §5 区域 Session Program / Regional Session Protocol

#### §5.1 实例状态、guard 与启动

**P-5.01｜本地状态。** 列出固定 $(s,C,H_s,\mathrm{TB},K)$，以及 phase、view、locks、votes、不可覆盖的已签名槽记录、QCs、timers、committed prefix 和 execution state。Assigned/Running/Terminated 与 Active/Frozen 分层表达，不把 Quit 当作另开 session。

**P-5.02｜统一动作 guard。** 每个 BFT action、timer callback 和迟到 authority callback 均检查本机 entry/termination/Closed 标记。关闭后只保管既有材料；尚未获知公共关闭的节点可能短暂继续，但其输出不能越过全局 cutoff。

**P-5.03｜初始 Enter。** 按 ReadyBundle、精确 anchor 与固定 context 原子进入 $(g,t)=(0,0)$，立即发完整 bundle 并启动 progress checks。插入 A-03；没有新的 READY quorum、STATUS 或 ANCHOR bootstrap round。

#### §5.2 稳态、term 与 Completion

**P-5.04｜提议与投票。** 按 T-05 使用正确 parent justification，完整验证后首次投票，转发并启动唯一 $2\Delta_i$ commit timer。保留对同代 earlier-term proposal 的处理，重复包不重启 timer；插入 A-04。

**P-5.05｜term handoff 与时间。** terminal QC 允许下一 term 先于 parent commit 启动，普通 term 切换不取消已有 commit timers。所有进度时钟从真实本地 Enter 开始，而不是等第一份 proposal；插入 F-05。

**P-5.06｜形成 FINAL 与正常完成。** 只有直接 timer commit 或其 ancestor commit 发新 FINAL；receipt-induced commit/Completion 不再发新 FINAL。terminal QC 或本地 commit 不等于完成，完整 terminal receipt 才触发 Completion、取消 timers 并持久转发。

#### §5.3 Quit 与代恢复

**P-5.07｜检测与冻结。** 写清 $(2p+4)\Delta_i$ 的 progress checks、证据优先顺序、message/timer 同时发生时先消息。完整 quit proof 冻结当前代并取消未到期 timers，保持 commitments 并继续收集可抬高 entry lock 的 QCs。

**P-5.08｜L1 REC 与进入下一代。** 每类 proof 都提交 Authority；进入条件为本地 freeze 后的规定等待与 finalized REC 均满足。source relay 如何使不同节点的 authority 可用时间落在区域界内，放到 §7 与附录 B.1，不能假定所有节点同时看到 L1。

**P-5.09｜STATUS、NEWVIEW 与 ANCHOR。** entry 时冻结 $Q_v$，正确 leader 等待规定时间后选择 $Q^*$，ANCHOR voting 检查 opening/rank 且不启动 commit timer。插入 A-05，完整说明 old/future generation 证据的保管与缓冲。

**P-5.10｜endpoint recovery 与 DRAIN。** 如果 recovery opening 已在 $H_s$，只能产生一个无新 DATA 的 DRAIN，以其 timer 完成 terminal DATA。后续被中断的恢复可能还有 DRAIN，但它们不进入 $G$、不增加 session quota；不能拿 physical height 的进展当 DATA 高度进展。

### §6 发布与正常全局最终化 / Publication and Normal Global Finality

#### §6.1 从区域结果到公共对象

**P-6.01｜Publisher 的职责。** 完整结果可以由 leader、其他 replica 或第三方发布，不以原 publisher key 独占内容授权。完整结果交付界从“持有完整结果”起算，形成 FINAL 的时间不算作发布服务已完成。

**P-6.02｜持久交付。** 描述去重、备份付费路径、重试和退出后保管；publisher 不做新的共识，publication acknowledgement 不阻塞区域 vote/timer。插入 A-06。

**P-6.03｜公开集合。** 定义 $M_q,\mathcal O_q$，只包含真实 finalized namespace 的 bytes/signatures；跨记录补齐、空 namespace、无效对象和 optional sidecar 各怎样处理。插入 F-06 的来源集合部分。

#### §6.2 公共 FINAL kernel

**P-6.04｜验证输入。** 从 canonical boundary 导出 $C_\nu$，验证目标 $B$、FINAL root $D$、签名集合和精确 path；不用发送者自报委员会。插入 A-07，给出 D-07 的 public-check 形式。

**P-6.05｜检查结果的含义。** VALID 只证明历史区域输入正确且 committed，不独立授予 global append 权限。介绍“原始签名阈值→正确 signer 的 timer→完整验证”的证明路线，避免把待证明的 validity 放进 RawFINAL 的前提。

#### §6.3 全局状态、初始化与驱动

**P-6.06｜全局对象。** 定义 $G$、不可变 $\sigma_G$、$\gamma(G)$、Tail 和 source/store 进度，给出 D-08、T-08。global envelope 与 native block 的对应必须显式；TEE、Manager 与外部观察者解释相同状态。

**P-6.07｜genesis 与 checkpoint。** 给出完整初始参数、source origin、初始 OPEN 和 ResolveOpen。后续 checkpoint 包含状态与仍可引用对象，不允许只拿一个末块 hash 代替历史依赖。

**P-6.08｜统一驱动。** 以输入序列的固定位置与内部 pc 推进回放，摄入不等于接纳，Done 不等于单一连续执行计数。插入 A-08；此处声明模式分支调用 §8 的具体子程序，不把其语义复制一遍。

#### §6.4 Eligible 与 NORMAL 转移

**P-6.09｜接纳谓词。** 明写 $\mathrm{Fin}_q$ 的公开对象存在条件和 $\mathrm{Eligible}_q$ 的 mode、session、parent、height、quota、receipt 与 $q^{\rm open}\le q'<q$。给出 D-09，说明 least-hash 是固定确定性 tie-break，不是额外竞争规则。

**P-6.10｜prelude 与普通 Force。** Espresso block 开始先处理普通 Force，核对 target，保留精确 $G$，用 NextTurn 处理零产出/已有产出的 $r$，安装不下降的 L1 floor。插入 A-09 的 normal prelude；没有另一个独立 Force-handler 算法副本。

**P-6.11｜追加、OPEN 与 regular successor。** 追加 terminal DATA 后先持久化，再计算 successor，缺来源只留下 OPEN；完整 Plan 返回 $\bot$ 才是 PENDING。展示 q-floor 的递增、selection reference 的固定性和跨 publication 补证后的重试，完成 F-06。

#### §6.5 全局最终性的意义

**P-6.12｜观察者何时确认。** 按已完成内部转移 $m$ 定义 $B\in G_m^u$；正常完整 block 末端可继续记 $G_q$。一个 receipt、Espresso inclusion、MODE 公告或 BATCH certificate 都不能单独替代该追加规则，给出 D-12 的 normal 分支。

**P-6.13｜输入安全与结算对应。** 说明 append-only 的回放代码只证明不删自己的输出；要保证原生结算同高度不替换，需要 §8 的保护策略和 §9 的一致性连接。该段引出后文，不提前把未实现的保护当作已经证明。

### §7 Session Manager 与跨区域轮换 / Session Lifecycle and Handoffs

#### §7.1 全局基线与提前准备

**P-7.01｜Manager 的三个进度。** 跟随 $G$、当前版本的 $R$ 和完整控制历史，保存 plans、handoffs 和程序生命周期。强调自己执行公共规则，不能根据某个远端“已完成”消息自行挑选下一 region。

**P-7.02｜两份准备状态。** baseline 是 $\mathit{st}(\operatorname{last}(G))$，regular successor state 跟随前驱完整区域结果。运行例子中分别是 108 与 110，不可混用；一旦 handoff 固定 anchor，后续 baseline 变化不修改该实例。

#### §7.2 regular 与 Force handoff

**P-7.03｜regular 交接。** 给出 target $(\beta,r+1)$、$E_{s'}=A_s,A_{s'}=T_s$ 和 reference floors，依据 terminal FINAL 而非 terminal QC。可以在 predecessor 进入 $G$ 前创建后继，插入 F-07。

**P-7.04｜普通 Force 交接。** 从公共 prelude 的 retained $G^\star$ 和 export Tail 确定 target/source，不能取最远 speculative program。迟到旧回执只是历史证据，插入 F-08。

**P-7.05｜验证和权限分离。** `VerifyHandoffHistory` 确认固定事实，`CanEnter` 检查当前版本、成员、未进入/未终止。合法历史证明不能复活关闭实例；相同 pair 的不同固定字段必须拒绝。

#### §7.3 Ready 与有界准备

**P-7.06｜准备和状态验证。** 节点验证精确 native state 及来源后调用 A-03，不把 root/URL 当完整 state。首个正确 entrant 向固定成员发完整 ReadyBundle，接收者做自己的验证。

**P-7.07｜同步启动。** 陈述在完整处理预算内，首个正确 Enter 后 $\Delta_i$ 内其他正确成员进入的条件。把初始进入与 recovery entry 的论证区分，不增加 READY voting round。

**P-7.08｜T_ready 的起点。** regular 取 terminal DATA 被公共接纳的来源时点，Force 取 activation block，RESUME 取已提交模式边界；direct delivery 可更早开始。源读取、native reconstruction 和 installation 都计入规定预算，不从各节点晚下载的时刻重新计时。

#### §7.4 并发、关闭与持久化

**P-7.09｜并发实例。** old session 完成收尾、successor 已运行、更远 successor 已准备可以并存，G 仍只有一个 next-block owner。插入 F-09 与 T-09，并在 A-10 中枚举 create/start/complete/close 事件。

**P-7.10｜关闭优先。** 先持久化版本关闭标记，再阻断所有旧 callbacks 与动作，包含未创建但以后才收到 ReadyBundle 的 pair。Force/STOP 终止旧程序不删除保留数据，也不伪造 Completion。

**P-7.11｜source relay 与恢复重启。** 完整来源材料先入 PublicStore 再立即区域转发，entry/termination/发送任务持久化。初始 hydration 与长期 retention 单独描述；这同时支持恢复代 authority 到达界及重启幂等。

### §8 Espresso 停机与模式恢复 / Explicit Outage Recovery

#### §8.1 恢复目标

**P-8.01｜正常接纳为什么会停。** Espresso 不再最终化时，新普通记录不能完成当前路径；旧已 finalized 记录在数据/证明可用时仍可交付。L1、DA 或任意其他服务同时失效不被本节自动修复。

**P-8.02｜同一条 G 的两种来源。** 明确 $\mu\in\{NORMAL,DEPOSIT\_ONLY\}$；deposit-only 空的是普通交易列表，不是所有交易。新增 envelope、来源标记和 L1 派生 anchor evidence，使恢复不创建一条可覆盖旧 $G$ 的新历史。

#### §8.2 公共边界与候选证书

**P-8.03｜两个来源不能按到达顺序混排。** 给出规范输入片段 $E\ldots STOP\ldots D\ldots RESUME\ldots E$，定义固定 cuts 与前缀稳定要求 D-10。解释该要求是本协议要实现的性质，不是 Ethereum/Espresso 的外部假设。

**P-8.04｜具体边界规则的必填位置。** 放置 A-11：规定公告、接纳截止、cut 的封口、候选证书和 L1 提交的具体顺序。作者必须在此完成 D1；不能在最终稿仅写 `GetSafeBoundary()` 或假设 $\mathfrak B$ 已有目标性质。

**P-8.05｜MODE 与 CLOSURE 的差别。** MODE 可先认证一个由公共规则确定的候选边界，ModeApplied 再给实际 L1 提交位置；CLOSURE 只证明已经执行的关闭。说明如何避免“证书依赖自己的 ModeApplied、ModeApplied 又等证书”的循环。

**P-8.06｜生效前后的权限。** 分清公告中的截止/最早激活条件、等待完整材料的区间、实际模式提交。旧输入不能因证书晚到而追溯失效，恢复也不能选在其他观察者已经确认的 deposit-only 前缀内部；插入 F-10。

#### §8.3 受保护前缀的交付

**P-8.07｜保护从何时开始。** 对每个普通块从进入 $G$ 的那一步起要求原生结算同高度不可替换，STOP 只是固定已有义务。以 G=108、外部节点只重建到 105 说明 106–108 必须补齐，不能填空。

**P-8.08｜A/B 方案的定稿选择。** 在 D-11 中选择并完整写出“剩余窗口内有界交付”的环境/准入条件，或“inbox/derivation 强制保护”的具体修改，完成 D2。两种选择有不同的活性/结算语义；提纲阶段保留比较，最终主协议必须固定一种 profile 或完整给出两种可选配置的分立定理。

#### §8.4 STOP 转移

**P-8.09｜输入与精确匹配。** 检查 ModeApplied、控制 target、source cut、末块 hash/height 和 $\gamma(G)$；缺来源 WAIT，不匹配 INCONSISTENT，恶意未认证输入只拒绝而不停止系统。插入 A-12。

**P-8.10｜关闭与保存续接信息。** 保存 NextTurn、source region 和最小 floor，关闭旧版本，切换 Tail=DEPOSIT 而不增 $\beta$。只按固定控制 cut 取消该范围内的 pending 更新，后来消息不能改变已经持久化的解释。

#### §8.5 deposit-only 转移

**P-8.11｜原生前缀准备。** 定义 ReadyDep：模式已提交、保护交付成立、精确状态可从已接受输入重建。外部派生若已有更长的已确认 deposit-only 前缀，要保留并按原轨迹接入，不重写原生结果。

**P-8.12｜确定性展开。** 给出 DepNext 的 BLOCK/DONE/WAIT 结果，按固定 L1 cut 产生数据、计算唯一 $j_D$、验证 native parent/height，并追加来源标记。插入 A-13；不要求新 Espresso block、区域 FC、Plan 或 TB 消耗。

#### §8.6 RESUME 转移

**P-8.13｜共同恢复点。** 先完成到 $p_*$ 的原生派生，再核对 $d_*,A_R,\gamma_R$；模式消息不允许回到更早的 deposit-only anchor。认证到 Espresso $q_R$ 的历史，区间可取回但不重演旧普通产出。

**P-8.14｜新版本与过期控制。** 新 $\beta=\beta_s+1$，保留保存的 $r_{\rm next}$，设置不下降的 floor，以实际末端同时作为 $E$ 和 $A$。阈值落在跳过区间的未执行普通控制按固定范围取消，不在恢复后补执行；插入 A-14。

**P-8.15｜Ready 与第一次普通追加。** 新 Plan 可能 OPEN/WAIT 或 PENDING，不能复活旧程序；第一块仍满足 $q^{\rm open}=q_R$ 与严格早于 export block 的引用规则。示例从 120 的真实状态产生 121，旧 session 的 109 不再有权限。

#### §8.7 边界一致性与恢复范围

**P-8.16｜重放与本地实现。** 说明所有来源 cut、pc、Done 与输出原子持久化，重复输入不重新增版本/输出 deposit 块。cut 被违反时保留已确认结果并停止在明确冲突点，不能以恢复名义重算过去的 G。

**P-8.17｜恢复活性的条件。** 对照当前 mega-espresso 的 backlog、替代 batcher、deposit-only 与恢复后跟随真实状态的原则；明确显式模式增加委员会响应/提交依赖。插入 T-11 的 outage 行，不能将 AS-02-L 的健康期 $T_E$ 用来证明停机期间恢复期限。

### §9 原生构块、批次认证与派生 / Native Blocks, Batching, and Derivation

#### §9.1 Builder 与原生输入

**P-9.01｜builder 的范围。** 接收普通交易、按固定 parent/context 输出 ordered list 和 mini-block 划分；Appendix A 详细列 I-01–I-03。只保留足以将区域 DATA 对应到原生块的字段，不展开 mempool 政策。

**P-9.02｜输入绑定。** 原生 record 的交易列表、origin 和 height-derived timestamp 与被 FINAL 绑定的完整块一致；L1 deposits 与系统输入另由确定规则构造。普通交易执行 revert 不自动使整块无效，也不由 builder 的接收回执产生最终性。

#### §9.2 Batcher、可信 certifier 与 inbox

**P-9.03｜编码和 cursor。** 将已确认普通块编码为同一 channel 的连续 frames，说明 accepted-input cursor 与 state-root settlement 进度不同。中间 frames 不消耗完整 L2 块，模式区间的跳转须有已提交边界。

**P-9.04｜可信核对。** TEE 自行取得来源，按本稿 global rule 选择 $G$ 并核对 payload、DA commitment 和完整块字段，产生有类型 BATCH。L1 消费可信结论但仍原子检查权限、prev cursor 和 mode；插入 A-15 与 F-11。

**P-9.05｜失效与接手。** 说明 alternative batcher、channel timeout/repack、精确 bytes 取回、stale certificate 和部分 channel 的跨模式处置。不得把旧 arbitrary cursor override 直接继承成跳过已在 G 的普通块的权限。

#### §9.3 外部 derivation 与状态证明

**P-9.06｜观察者的输入。** 从 finalized L1 接受事件、模式事件、deposits 与 DA 对应字节按序派生，说明不需要节点在该路径重新验证 Espresso/TEE 硬件。插入 A-16，并引用 D-12 建立 native 对应关系，不再定义另一份全局最终性。

**P-9.07｜模式与缺失数据。** 按公开 mode boundary 保留前缀、处理 channel 并恢复实际 parent/state；已接受但不可取回的数据要求 WAIT。历史 native fallback 和新模式语义必须明确衔接，不能默默换掉已确认块。

**P-9.08｜state-root claim。** 独立说明输入对应性与状态根正确性：batcher/proposer 提交 range/inputRef/stateRoot，原有 fault-proof 模块处理争议。未来实现的 proof program 必须采用相同 derivation/mode version。

#### §9.4 两条观察路径的连接

**P-9.09｜早确认与晚结算。** 直接读 L1/Espresso 的节点得到 $G$，读 L1/DA 的节点得到 native derived chain；给出保护前提下两个序列的对应关系。较晚的结算和取回不改变早已授予的强最终性内容。

**P-9.10｜保留的实现自由。** 不同节点可采用不同 batch packing、缓存、转发和状态恢复实现，但必须得到同一已确认块和状态。说明本文不规定不可观察的内部流水线，也不把性能预算等同于硬件实现认证。

### §10 安全、隔离与进展 / Guarantees and Progress

#### §10.1 论证组织

**P-10.01｜结果与依赖。** 插入 T-10 和 F-12，列出每个 R 依赖哪些外部假设、已定义协议和内部引理。把区域安全、全局前缀安全、结算对应和进展四条论证链分开。

**P-10.02｜正文与附录分工。** 正文给明确陈述和关键证明思路，附录 B/C 在同一 TeX 内补齐完整论证。所有新增恢复性质应标为本稿证明目标，不能把旧 Part II 的结论视作自动覆盖。

#### §10.2 授权与区域结果

**P-10.03｜R-01：Authority 的确定性与不变性。** 从固定 L1 cut、snapshot、policy 和 C 推出所有正确验证者得到同一 plan/leader，historical weights 不变。强调 escrow 的变化不追溯影响证书。

**P-10.04｜R-02/R-04：区域安全与进入。** 陈述在关闭前、固定 context 与同步启动前提下的区域安全/输入有效性，以及 ReadyBundle 形成的初始 entry-skew 界。用 timer、传播和 recovery-lock 路线说明，而不是用错误的加权 quorum 交集捷径。

**P-10.05｜R-03：FINAL kernel soundness。** 从 raw signatures 中的正确 signer 追到实际 timer commit，解释 receipt-induced commit 不发新 FINAL 因而不能循环自证。覆盖路径只推广到同 session 的精确 ancestors。

#### §10.3 全局前缀与原生对应

**P-10.06｜R-05：全局一致性与持久性。** 对共同规范输入与内部转移归纳，N/D 只追加，Force/STOP/RESUME 只改变后续责任与状态。明确 proof 使用已实现边界策略的前缀稳定引理；此依赖不能被遗漏。

**P-10.07｜边界策略的证明接入。** 用 A-11 的具体规则证明未来控制不会插入已暴露输入前面，双重观察进度只造成 WAIT。D1 未闭合前，本段只能是条件性结果，不能写成全系统已经获得无条件 agreement。

**P-10.08｜R-09：结算对应性。** 连接源数据、trusted BATCH、inbox、DA bytes、原生 derivation 与状态证明。明确需要 D2 选定的保护策略，从每个块进入 G 的时刻起成立，而非只检查模式切换瞬间。

#### §10.4 生命周期与资金责任

**P-10.09｜R-06：实例隔离。** 对 entry/termination/Closed 和 pc 持久化归纳，证明每个 pair 初始 Enter 至多一次，终止后不能复活，旧 callbacks 不修改新实例。滞后节点仍可能短暂发消息，公开接纳边界排除其旧输出。

**P-10.10｜R-07/R-08：处罚与释放。** 同 cid 的完整签名证据防止跨 context 错罚，固定 offence key 防止重复扣款；实际 closure、future floor 与持有期共同隔离旧职责。把证据可能永远隐藏的情况和退出等待成本明确排除在有限持有期保证之外。

#### §10.5 条件性进展

**P-10.11｜R-10(a)：NORMAL。** 由合格 live region、source/ref 可用、有效 leader 机会、区域结果形成、publisher 路径与 Espresso 服务组成进展条件。有限 session 完成与跨 session 的全局高度进展分别论证，不从 Ready 自动推得 Global progress。

**P-10.12｜R-10(b)：STOP 与 deposit-only。** 从实际控制响应、边界提交、受保护前缀交付和 L1/native 数据服务推出相应阶段可继续；没有这些前提时仅保证安全等待。解释继承的自动 native fallback 与显式 G 模式的确切关系，不能用一个模式标签证明交易已经进入结算。

**P-10.13｜R-10(c)：RESUME。** 需要共同截止点、可用 Espresso、符合计划的 region 和正确状态安装；首次 DATA 的引用下界仍有真实等待。若目标无合格区域则 PENDING，下一次治理动作是新的进展前提。

**P-10.14｜不声称的时间界。** 区分 $T_{\rm pub}+T_E$ 的完整结果交付界、$T_{\rm ready}$、委员会响应和结算预算，避免相加时重复计入 source visibility。不存在由“诚实 multisig”推出的无条件恢复时间，也没有区域公平性保证。

### §11 实现边界与讨论 / Scope and Integration

**P-11.01｜对 mega-espresso 的最小必要改动。** 摘要列出 publisher 授权、选取规则从 single publisher 到 G、带模式的 cursor/boundary，以及保护已确认输入的规则。能继承编码与 DA/native execution 的部分不重复规范。

**P-11.02｜可接受的代价。** 公开 leader schedule、固定 session 权重、Authority 的 L1 等待、归档/状态保留、显式治理恢复与可能的 stake release 延迟。不得把这些成本改写成未经证明的抗攻击或公平性保证。

**P-11.03｜故障范围。** 用 T-11 回顾 region 离线、Espresso 停机、DA 缺失、source 缺失、stale control 等的不同结果。保持 partial safety 不在本稿范围，不临时扩展为任意外部链安全故障恢复。

**P-11.04｜单篇规范与组合实现。** 说明各内部模块可以独立实现/替换的前提是保留这些状态与接口契约；明确没有构造 UC simulator 的结论。只有当 D1–D4 和相关证明完成，才可声称当前配置的完整协议闭合。

**P-11.05｜结束段。** 用简短一段重述本稿实际获得的性质及其明确前提，不引入新机制或新性能数字；将部署、编码审计和实现验证留在单独的后续工作范围。


## 2. 协议块逐项设计

以下 A 编号是提纲定位符；最终 TeX 用统一 `protocol` 环境自动编号，并保留稳定 label。
每块必须显示 owner、输入、输出、持久状态、具体步骤和 WAIT/REJECT/重复处理语义。
子过程可在同一块中定义，不能再复制一份略有不同的 Force 或 replay 规则。

### A-01｜成员请求与快照冻结

**位置。** §4.2，P-4.05 后；label `alg:membership-snapshot`。

**执行者/触发。** L1 Authority；由已准入的成员/快照调用触发，状态按 L1 顺序原子更新。

**输入**

- Join/Cancel/RequestExit/CutSnapshot 的有类型请求、已认证 caller/rid、固定参数。
- 本账本 positions、keys、FIFO、历史余额/处罚、前一快照和当前 L1 位置。

**输出**

- 注册/取消/退出记录，或唯一的 $\mathcal C_e$、退役记录与队列更新。
- 明确的 rejected/重复已处理结果，且没有重复转账或重分配 ID。

**核心功能段。** 解释资金状态与可用投票权重在不同时间改变，本块不改变任何固定 session 的委员会。

**步骤骨架。** (1) requestHash/rid 去重；(2) Join 核对 owner、PoP、region、资金与 key 唯一性；(3) Cancel/Exit 检查生命周期并入相应队列；(4) Cut 固定严格早于 $c_e$ 的读取前缀；(5) 先强制移除，再按成熟 FIFO 与分别的 amount/count 预算处理；(6) 冻结 rows/root，更新退役记录；(7) 原子提交状态/事件。
只消费本次 cut 处理的 queue entries，保留 cutoff 后的新请求；输出时区分 Active 与仍有退出请求的 Exiting。

### A-02｜证据、恢复记录与安全释放

**位置。** §4.5，P-4.15 后；label `alg:accountability-release`；在前面证据段落先引用其分支。

**执行者/触发。** L1 Authority；由证据、可信关闭确认或 owner 提款请求触发。

**输入**

- `SubmitOffense(C,o,ev)`、`SubmitQuit(C,g,Q)`、`ProveClosure(χ_close)` 或 `Release(o,ownerAuth)`。
- snapshot/ban/REC、SeenOffenses、escrow、Closures、Clear 与持有期参数。

**输出**

- 去重的 slash/ban、唯一 REC、clearance 水位或一次性支付结果。
- 无效证据不改变状态，纯 BLAME 不处罚。

**核心功能段。** 将“报告可验证不当行为”“授权下一恢复代”“解除旧职责”和“退款”分成四个确定分支。

**步骤骨架。** (1) 解析 $C$ 并从真实快照取固定权重；(2) 依 T-07 检查同 key、同 cid、同 slot 的证据；(3) 按 offence key 去重再扣 $\min\{\rho_{\rm slash}w_o,B_o\}$；(4) Quit 原子先处理附带 offense 再创建 REC；(5) Closure 直接信任 TEE 的实际关闭/future-floor 结论并核对本链控制；(6) Release 检查 $e_x$、Clear 和持有期，结算已接受处罚后支付。
在 D4 预算没有闭合的配置中，Release 保持等待/拒绝，不能先假定一个适用于停机的有限时限。

### A-03｜初始 Enter 与 ReadyBundle 传播

**位置。** §5.1，P-5.03 后；label `alg:enter`。

**执行者/触发。** 被选中区域的本机 Program；Manager 在完整 Ready 条件满足后调用。

**输入**

- 唯一 Assigned Handoff、固定 plan、精确 $A_s$ 与原始 commitment evidence、可执行 $\mathit{st}(A_s)$。
- 本机 membership、entry/termination 记录与 Closed 标记。

**输出**

- 单次 `Entered(s,τ)`、本区域广播的完整 ReadyBundle、Running 的初始状态。

**核心功能段。** 从一个已固定的 anchor 启动独立子协议实例，不能拷贝前驱 locks，也不生成额外启动证书。

**步骤骨架。** (1) 验证 Ready；(2) 原子复查 CanEnter；(3) 保存 entry、$Q_s^0$、quota/floors/state；(4) 同事件发送 ReadyBundle；(5) 启动 progress checks；(6) 按证据优先顺序处理缓存；(7) 若自己为 leader 且所需 references 可用，执行 A-04 的首次 Propose。
重复 bundle 不重置 entry time；初始进入的同步界只覆盖 prepared、仍被授权且完整消息可处理的正确成员。

### A-04｜Propose、Vote、Commit、term rotation 与 Completion

**位置。** §5.2，P-5.04 后；label `alg:regional-steady`。

**执行者/触发。** 每个 regional replica 的对应 Program；由提议输入、消息、QC、receipt 或 timer 触发。

**输入**

- 当前固定 context、view、parent rule、候选 payload、认证 proposal/QC/FINAL 或 timer event。
- votes、locks、QC 集、entry times、timers、committed prefix 与可执行状态。

**输出**

- 协议消息、DATA commitments、FINAL signatures、完整区域结果、term entry 或 Complete 事件。

**核心功能段。** 用一套 vote/commit timer 规则服务普通 DATA 与 DRAIN，同时保留跨 term 的流水线执行。

**步骤骨架。** (1) guard 与 evidence-first；(2) leader 按 T-05/剩余 quota 顺序提议，不等 parent commit；(3) 对本代完整有效的未投 fresh proposal 投票并转发，启动 $2\Delta_i$ timer；(4) timer commit 自身及 ancestors，并只对本 session DATA 发 FINAL；(5) terminal QC 处理历史 proposals 后进入下一 term，只有 progress checks 更新、commit timers 保留；(6) 完整 terminal FC 触发 Completion，转发并保管结果，取消剩余 BFT 工作。
明确不同世代、earlier-term、重复包、终止后回调及同时时间事件的分支；receipt-induced commit 不产生新的 FINAL。

### A-05｜BLAME、Quit 与 generation recovery

**位置。** §5.3，P-5.09 后；label `alg:regional-recovery`。

**执行者/触发。** 每个 regional replica 的对应 Program；由进度 deadline、quit proof 和 REC 到达触发。

**输入**

- 当前 generation 的进度事件、完整 quit proof、finalized REC、STATUS/NEWVIEW/ANCHOR 材料。
- 固定 session context、entry times、最高 QCs 与锁。

**输出**

- BLAME/证据传播、提交 L1 的 proof、冻结/恢复进入事件、NEWVIEW 与 ANCHOR QC。

**核心功能段。** 保留已有 commitments，在固定 session 内通过公开 REC 改变 proposer eligibility 与恢复 opening。

**步骤骨架。** (1) 按 $(2p+4)\Delta_i$ 判断 BLAME；(2) 首次完整 proof 冻结代、停 timer、广播并提交 L1；(3) 等规定 freeze 间隔与 REC 都满足；(4) entry 时固定 $Q_v$，发送 STATUS；(5) 正确 leader 在规定 $2\Delta_i$ 收集期后选 $Q^*$ 并发 NEWVIEW；(6) 验证并投 ANCHOR，不启动 commit timer；(7) 依 opening 的 DATA height 进入 DATA 或 DRAIN generation。
late/future proof 的处理不得重置无关 recovery；恢复不能改变 $H_s$。

### A-06｜发布完整区域结果

**位置。** §6.1，P-6.02 后；label `alg:publish`。

**执行者/触发。** 完整结果持有者及其 Publisher；由新完整结果、提交失败或已最终化回执触发。

**输入**

- RegionalResult 的 $C,B,D,\Sigma,\pi_{\rm path}$、完整 handoff/source 材料。
- 待发送队列、稳定内容 ID 和可用的提交/付费路径。

**输出**

- 精确 EXPORT objects、提交请求和 finalized-publication 记录。

**核心功能段。** 把完整结果可靠送到公共来源，不替 global rule 决定哪块应被接纳。

**步骤骨架。** (1) 核对 producing context 和完整 kernel；(2) 先保管再入队；(3) 去重/打包并提交；(4) 失败切换可用发送路径；(5) 记录精确公开位置；(6) 直到交付及保管责任移交完成才释放本地任务。
可批量/分片，签名和 body 不可被修改；Espresso 停机时只保管重试，不继续声称 $T_E$ 已保证新记录可见。

### A-07｜公开 FINAL 检查

**位置。** §6.2，P-6.04 后；label `alg:check-final`。

**执行者/触发。** 任何需核验历史区域结果的节点；这是无副作用的公开检查。

**输入**

- 检查者自行导出的 authentic $C_\nu$、目标 DATA $B$、root $D$、FINAL 集合与完整 path。

**输出**

- VALID、INVALID 或 INCOMPLETE。

**核心功能段。** 从固定委员会 raw signatures 与实际 ancestry 判断区域承诺，完整应用有效性由正确 signer 的真实验证/commit 事件支撑。

**步骤骨架。** (1) 检查 chain/domain/cid 和 typed hashes；(2) 按固定成员去重并求权重 $>F_s$；(3) 核对每段真实 body/hash/parent/position；(4) 保证 root 与 target 同 session、目标在 quota 内；(5) 返回三值结果。
optional sidecar 不是必需输入；不能由另一 session 的后继回执覆盖前驱 anchor。

### A-08｜统一 Global Replay Driver

**位置。** §6.3，P-6.08 后；label `alg:global-driver`。

**执行者/触发。** Manager、可信 certifier 或外部观察者；由新来源或已等待依赖到达触发。

**输入**

- 完整持久化 $\mathcal X$、PublicStore、新认证来源与固定的边界规则。
- 当前 source cut、规范输入位置 $z$ 和内部 $pc$。

**输出**

- 新 $(G,\mathsf{Tail})$、生命周期事件、以及带明确暂停位置的状态。

**核心功能段。** 用一套驱动运行 A-09/A-12/A-13/A-14；来源取得、语义处理与对外发送分别进行。

**步骤骨架。** (1) 验证并持久化来源；(2) 扩展完整控制前缀但不提前执行；(3) 恢复进行中的输入/OPEN；(4) 根据固定公共顺序取得下一输入；(5) 调用唯一分支；(6) 原子持久化状态、Done、pc 和待发事件后发送；(7) 仅在输入完成时前进 $z$。
无效网络包被拒绝，不成为 INCONSISTENT；只有可信规范来源矛盾才安全停在该处。

### A-09｜NORMAL、Force prelude 与 ResolveOpen

**位置。** §6.4，P-6.10 后；label `alg:global-normal`。

**执行者/触发。** A-08 的本地回放器；仅由确定的下一 NORMAL Espresso 输入调用。

**输入**

- 下一完整 $J_q$、$M_q$、适用普通控制与当前 NORMAL replay state。

**输出**

- 应用或忽略控制的结果、追加 DATA、更新后的 NEXT/OPEN/PENDING 和 Handoff。

**核心功能段。** 先处理控制，再按唯一接纳谓词追加连续普通输入；所有处理仍由同一 driver 管理。

**步骤骨架。** (1) 摄入 $M_q$；(2) 找出该 block 首次跨越阈值的普通控制；(3) stale target 记 no-op，否则保存 NextTurn、关闭版本、安装新 policy/floor；(4) ResolveOpen 固定 Plan，来源不足 WAIT、无 region 才 PENDING；(5) 求 Eligible 集并取 least-hash；(6) 原子追加及写 $\sigma_G$；(7) terminal 后先保存追加和 successor OPEN，再回第4步。
本块明确 NextTurn 的零产出分支、ordinary Force 在 J_q prelude 生效、$q'<q$ 与 floor 不减；不复制进 Part IV 风格的第二份 Force 算法。

### A-10｜Manager 的生命周期与 source relay

**位置。** §7.4，P-7.09 后；label `alg:session-manager`。

**执行者/触发。** 每个节点的 Manager；对公共来源、程序事件和关闭/恢复事件逐个处理。

**输入**

- 验证过的公共来源、RegionalResult、Handoff、Program events、Mode/Force events 与可验证状态材料。
- Program 表、Closed、baseline/successor states 与 PublicStore。

**输出**

- Create/Prepare/Enter/Close 调用，发布、证据提交、state/source relay 和持久化记录。

**核心功能段。** Manager 管实例权限与状态准备，不替 Program 产生共识结果，也不自行修改公共链选择。

**步骤骨架。** (1) Track 公共回放与 $R$；(2) 首次完整 source 到达时先存后向区域转发；(3) 验证并固定唯一 Handoff；(4) 准备精确 state 后调用 A-03；(5) 处理 Result/Evidence/Complete 并持久化任务；(6) 关闭时先记版本禁权，再终止全部相关实例；(7) 重启按记录恢复而不重入。
允许多个 session 并发，数据保管/发送任务的生命周期不依附于已终止 Program。

### A-11｜模式边界的确定、认证与提交【待闭合 D1】

**位置。** §8.2，P-8.04 后；label `alg:mode-boundary`。

**执行者/触发。** multisig、公共规则执行者、可信 certifier 与 L1 inbox；正文必须逐步标出各自负责的动作。

**输入**

- finalized 模式公告、当前公开控制状态、可认证的 L1/Espresso cuts 和既有确认进度。
- 最终选定的公共截止/封口规则，以及 D2 选定的 protected-prefix 条件。

**输出**

- 唯一、不可追溯更改的 STOP 或 RESUME 候选边界，可信 MODE 消息，以及 L1 ModeApplied；材料不足则 WAIT。

**核心功能段。** 本块必须实现规范输入前缀稳定，不能把“返回一个不回退的边界”本身当成实现。

**必须具体化的步骤。** (1) 公告一个可由共同来源判断的截止条件；(2) 规定截止前哪些普通/derived 输入还能授予最终性；(3) 固定完整 source cut 并封口，处理迟到数据和相同 timestamp；(4) 检验候选包含所有此前允许确认的输入；(5) TEE 认证候选但不等待自己的未来 ModeApplied；(6) 满足激活与保护条件后在 L1 原子提交；(7) 后续再确认 actual closure。
第2–4步目前是必须完成的设计工作，不在提纲里假装已有代码。若实现需要新增可信确认或延迟，必须同步更新 F-01、假设和延迟讨论，不能藏在一个 helper 里。

### A-12｜STOP 转移

**位置。** §8.4，P-8.09 后；label `alg:stop`。

**执行者/触发。** A-08 的本地回放器；消费已认证的 STOP 输入，再由 Manager 落实本机关闭。

**输入**

- finalized ModeApplied、可信 MODE、完整 $K_S$ 与到 cut 的稳定 NORMAL state。

**输出**

- 保留的 $G^\star$、Closed 标记、DEPOSIT Tail、保存的 $(r_{next},i_{src},\lambda_{min})$ 和模式事件。

**核心功能段。** 关闭旧普通片段并保存精确续接信息，不选择另一条旧前缀，也不凭关闭立即退款。

**步骤骨架。** (1) 检查授权/target/类型/激活条件；(2) 核对 source cut 与完成状态；(3) 精确匹配 G 的末块与承诺；(4) 保存 NextTurn 与不降的 future floor；(5) 原子关闭旧版本、记录 Done 并进入 DEPOSIT；(6) 按该公共 cut 取消规定范围内的旧 pending 控制。
缺资料等待，较旧 boundary 不授权截断；STOP 不增加 $\beta$。

### A-13｜Protected prefix 准备与 deposit-only 推进

**位置。** §8.5，P-8.12 后；label `alg:deposit-mode`。

**执行者/触发。** 带有确定性原生派生状态的回放节点；消费固定的 L1 派生 cut。

**输入**

- 已提交的 STOP、protected prefix、接受的 L1/DA 输入、原生派生状态及下一固定 L1 cut $p$。

**输出**

- 确定的 $B^{dep}$ 与来源元数据、派生后状态，或带缺失对象的 WAIT。

**核心功能段。** 把原生规则确定的 deposit-only 块接入同一 G，无新的区域共识或 Espresso 最终化要求。

**步骤骨架。** (1) 核对 ReadyDep 与精确 native prefix；(2) 初始化或恢复派生 pc，保留已知原生结果；(3) DepNext 返回 BLOCK/DONE/WAIT；(4) BLOCK 检查高度、native parent、空普通列表和执行结果；(5) 从已持久化同 $(c_s,p)$ 输出计数确定 $j_D$；(6) 构造唯一 envelope，原子追加/记录来源与状态；(7) 继续至当前 cut 完成。
accepted payload 不可取回时不能补空块，proof/state 不匹配不能以重启修复为由回滚。

### A-14｜RESUME 与新版本开设

**位置。** §8.6，P-8.14 后；label `alg:resume`。

**执行者/触发。** A-08 的本地回放器；消费已认证的 RESUME 输入，后续由 Manager 准备新实例。

**输入**

- finalized ModeApplied、可信 MODE、$K_R$、已完成到 $p_*$ 的 deposit-only state、source prefix 到 $q_R$。

**输出**

- 保留全部 G 的新 NORMAL state、新 $\beta$、OPEN/NEXT/PENDING、以及下一 Program 的 Handoff。

**核心功能段。** 用真实派生末端重新开始普通生产，停机期间的高度不消耗区域 turn 编号。

**步骤骨架。** (1) 验证 mode/target/cuts 与精确末端；(2) 认证停机区间 Espresso 数据但不重演其普通追加；(3) 取消阈值落入跳过区间的指定未决控制；(4) 安装 $\beta_s+1$、保存的 $r_{next}$ 与不降 floor；(5) 取 $E=A=last(G)$，opening q-floor 为 $q_R$；(6) 原子记录 Done 与 `R.OPEN`；(7) ResolveOpen 后再由 Manager 进行 Ready/Enter。
重试 `R.OPEN` 不重复增加版本，旧程序不重新生效。

### A-15｜Batcher—可信 Certifier—Inbox 交互

**位置。** §9.2，P-9.04 后；label `alg:batch-certification`。

**执行者/触发。** batcher、DA、可信 certifier 和 L1 inbox；各步使用明确角色，不假设它们由同一运营者承担。

**输入**

- 待交付 G 区间、finalized inbox cursor、channel state、精确 payload、DA commitment 和当前模式。

**输出**

- 可信 BATCH、L1 CursorAdvanced 或模式边界提交结果；无效请求不推进 cursor。

**核心功能段。** 将可自由选择的打包方式与不能自由选择的全局块内容分离，TEE 业务结论被直接信任。

**步骤骨架。** (1) 从 G 构造连续 frames；(2) 取得 DA 绑定；(3) TEE 读取来源并重放同一 global rule；(4) 检查 frame/完整块对应，生成绑定 prev/next/d/range/globalRef 的 BATCH；(5) inbox 检查本链权限、prev cursor 和 mode；(6) finalized acceptance 供 derivation 使用。
未闭合 channel 不提前消耗块；MODE 边界不能绕过 protected-prefix 检查；state-root claim 走独立接口。

### A-16｜外部节点的确定性 Derivation

**位置。** §9.3，P-9.06 后；label `alg:derive`。

**执行者/触发。** 独立观察者/derivation 节点；由新的完整 L1/DA 输入驱动，不由 sequencer 指定结果。

**输入**

- 前次原生派生状态、完整 finalized L1 prefix、已接受 DA 字节和固定 rule version。

**输出**

- 原生块、执行状态/roots、input/cursor 进度与 deposit-only 接续材料，或 WAIT。

**核心功能段。** 从 L1/DA 复现实际 L2 链，与提前确认路径在受保护输入上得到相同结果。

**步骤骨架。** (1) 按 L1 事件顺序读取 deposits/configuration/acceptance/mode；(2) 只解析真实接受的批次；(3) 核对 DA bytes 与 commitment；(4) 处理 channel 和 mode boundary；(5) 依固定原生规则构造/执行；(6) 持久化实际 parent/state 与进度；(7) 输出需要的接续/证明输入。
不读取 caller 自报的“正确状态”，不以 ModeApplied calldata 替代 batch 内容，不在缺失 accepted bytes 时继续推进。


## 3. 图逐项设计

F 编号是完整图清单，不是已生成的图片。
最终统一使用 inline TikZ：蓝色表示区域结果/生产，绿色表示已进入 G，橙色表示控制，灰色表示未被接纳或已关闭的后缀；颜色之外还必须有文字/线型区别。
所有 timeline 都标明横轴是实时时间、Espresso 高度还是 L1 位置，不能用同一横轴混合三种坐标。

### F-01｜整套协议的架构图

- **位置与目标：** §1.1 P-1.02 后；让读者一眼看到数据路径、控制路径和观察者的两种读取方式。
- **布局：** 左到右为用户/Builder、区域 Program、Publisher/Espresso、Global Rule、Batcher/可信 Certifier/DA/L1、Derivation；Manager 与 Authority 放上方控制带。
- **必须画出的箭头：** Authority→Manager/Program 的上下文；Program→Publisher 的完整结果；Espresso+L1→Global Rule；G→Batcher；TEE→inbox 的可信业务结论；L1+DA→Derivation；deposit-only derivation→G 的模式分支。
- **图例与 caption：** 实线数据、细线控制、虚线可信结论；L1/Espresso 用“假设的外部来源”表示，不画 ideal-functionality API 或链间 light client。Global Rule/Derivation 用规则框，避免误画成新共识网络。
- **来源/修改：** 从组合式抽象 v2 的总图重排，减少重复角色节点；若 D1 最终引入新的正常路径依赖，必须在此明确加入。

### F-02｜一个例子看懂三条路径

- **位置与目标：** §1.3 P-1.06 后；在形式化之前解释 G、R、接棒 anchor 与恢复。
- **布局：** 三个并列小图，共用起点 A:100→110、G:108、B speculative:111–115。
- **Panel (a)：** 正常交接，从 terminal 110 接 B；G 可以稍后追上。
- **Panel (b)：** 独立的 Force 场景，从 G 的 108 开新版本，灰化旧 109–115。
- **Panel (c)：** 独立的 STOP 场景，保留 108，L1 派生109–120，再从120恢复普通121；数字是共同例子，不表示任意模式都能跳过旧输入。
- **Caption：** 三个 panel 是互斥的可能执行，不共享同一个后续版本计数；已接受前缀在三种路径都保持不变。

### F-03｜四层坐标、quota 与区块类型

- **位置与目标：** §3.1/§3.2，最终置于 P-3.06 后；解释索引变化和 DRAIN。
- **布局：** Panel (a) 用嵌套带表示 branch/session/generation/term；Panel (b) 用双行 h/k 展示 TB=10、K=4 的 DATA 101–110。
- **关键内容：** term0=101–104，term1=105–108，term2=109–110；恢复 opening=110 时，DRAIN 只使 k 增一，h 保持110。
- **必须避免：** 不画 DATA generation 自发在 terminal 后加 DRAIN；不把 L1 deposit-only 放在区域 physical-position 带中。
- **Caption：** normal term、Quit、regular handoff、Force/RESUME 分别改变不同坐标；quota 固定于初始 anchor，恢复不重置。
- **来源：** Part II 的 §3.2 quota 例子，图为新绘。

### F-04｜Stake 生命周期与快照时间

- **位置与目标：** §4.2 P-4.05 后；将资金、未来权重和已分配责任拆开。
- **Panel (a)：** Pending→Active→Exiting→Retired→Cleared→Released；Pending 的 Cancel 分支；offense/低余额导致下一 cut 的强制 Retired。
- **Panel (b)：** L1 轴标 $c_e,a_e$、publication height、Plan 读取 $\ell$；cutoff 后 slash 降 escrow，但已冻结的 $w_{e,o}$ 仍不变。
- **Caption：** retirement、closure 与 holding period 是三个条件；禁任改变 proposer eligibility，不能删除旧证书的投票权重。
- **来源：** Part III v6 生命周期图可重画，但 closure 标签改为可信实际关闭确认，并标出停机预算另行规定。

### F-05｜区域快路径与 recovery 概览

- **位置与目标：** §5.2 P-5.05 后；配合 A-04/A-05 解释计时器与消息。
- **Panel (a)：** proposal→完整验证/首次 vote→$2\Delta_i$ timer→commit/FINAL；终端 QC 可先触发新 term，旧 timer 继续。
- **Panel (b)：** freeze/提交 quit→REC 与等待条件→STATUS→NEWVIEW→ANCHOR→DATA/DRAIN。
- **关键标注：** QC 和 ANCHOR vote 不启动 commit timer；publication/settlement 不在区域 vote 热路径；message/timer 同时发生先 message。
- **Caption：** 具体时界以完整依赖和 prepared-node 前提为条件，图中箭头是因果步骤而非额外必须等待。
- **来源：** [steady-proposal](../formalization_v3/figures/sync_hotstuff_rr_core/steady-proposal.tex)、[term-rotation](../formalization_v3/figures/sync_hotstuff_rr_core/term-rotation.tex)、[quit-recovery](../formalization_v3/figures/sync_hotstuff_rr_core/quit-recovery.tex)；合并重画，完整 entry/progress 推导见 F-13。

### F-06｜对象累计与公开接纳

- **位置与目标：** §6.4 P-6.11 后；说明 publication、Fin_q 和 G 的不同。
- **Panel (a)：** 以 Espresso q 为横轴；J10 含 B101 body 与部分签名，J11 补齐权重，只有 J11 时 Fin 成立并追加。B101 的 xi 指向较早 J9。
- **Panel (b)：** terminal110 追加→Tail=OPEN→固定来源到达后 NEXT 或经完整 Plan 得出的 PENDING；G 不后退。
- **细节：** 在 O_q 轨道展示跨 publication 合并；optional sidecar 放在不参与主线的灰框；outer EXPORT hint 不决定是否追加。
- **Caption：** 所有候选材料来自同一完整 namespace 前缀，合格候选依确定规则处理；缓存缺字节与从未发布分别标识。
- **来源：** 新绘，依据 v4.9 与抽象 v2 的公开对象/等待规则。

### F-07｜regular 跨区域 handoff

- **位置与目标：** §7.2 P-7.03 后；展示 early handoff 与两份 prepared state。
- **布局：** 前驱区域、继任区域、Espresso 三条泳道；terminal FC 可直接跨区先到，Espresso 记录稍后。
- **必须标出：** $E_{s'}=A_s$、$A_{s'}=T_s$、$q^{open}_{s'}$；baseline st(108) 与 successor st(110)；跨区 $\Delta_{cross}$ 和区域内 ReadyBundle 的 $\Delta_i$ 分别标注。
- **禁止误导：** 没有新增 READY quorum，不把 terminal QC 画成 FC；Ready 可以先于 opening-floor header，但第一 DATA 不能提前引用不存在的 header。
- **Caption：** 接棒依据固定的 terminal result，不依赖谁先在网络上看见它；状态准备和最终性是两件事。
- **来源：** [v4.9 regular-handoff](../formalization_v3/figures/cross_region_rotation_v4_9/regular-handoff.tex)，按新统一记号与状态角色更新。

### F-08｜普通 Force 的精确 cutoff

- **位置与目标：** §7.2 P-7.04 后；解释普通 Force 的 Espresso prelude。
- **布局：** 上行 L1 公告，下行 Espresso J_qf，底部 G/R 分支树；J_qf 开始先 Force 再处理 objects。
- **关键内容：** G 保留108；旧版本109–115灰化；新版本从108开；新 session 的 q-floor 为 q_f，首次全球接纳不能在同一 q_f 自引用。
- **Caption：** 控制不指定一个任意旧 anchor；旧 receipt 仍是历史证据，但不能追加关闭版本后缀；本图不表示 STOP 也等待 Espresso。
- **来源：** [v4.9 forced-handoff](../formalization_v3/figures/cross_region_rotation_v4_9/forced-handoff.tex)，移除旧 Part IV 的 NextLive/TB_r 表述。

### F-09｜并发 session 与一次关闭

- **位置与目标：** §7.4 P-7.09 后；说明 Manager 为什么管理多个实例。
- **布局：** 区域/实例泳道，横向画 Assigned、Running、Completed；上方单独画 G 的 export owner，避免将它等同于最远生产实例。
- **关键内容：** A 完成收尾、B 已运行、C 已准备；同一 Force/STOP 的 Closed 标记终止该版本全部实例；新 beta 可复用 r 而不复用 cid/state。
- **Caption：** 程序终止不终止数据/证据保管和已形成结果的发送任务；跨实例共享的是验证过的 immutable 事实，不是 locks/votes/timers。
- **来源：** [v4.9 concurrent-handoffs](../formalization_v3/figures/cross_region_rotation_v4_9/concurrent-handoffs.tex)，补充 Manager 的实例表。

### F-10｜STOP—deposit-only—RESUME 的来源边界

- **位置与目标：** §8.2 P-8.06 后；全稿恢复路径的核心图。
- **布局：** L1 控制/ModeApplied、Espresso source cut、原生交付/派生、全局 G 四条泳道，使用明确的跨轴投影而非伪造共同 block clock。
- **关键节点：** ScheduleMode、截止条件、候选 MODE、finalized ModeApplied、actual CLOSURE；q_* 后 Espresso 无新块；G108受保护，先补齐106–108，再接109–120，RESUME后普通121。
- **恢复部分：** 固定 p_*、d_*、A_R 与 q_R；后台读取停机 E 区间不重演旧普通块；新 beta/source floor 与第一次合法 reference 标在入口。
- **Caption：** 以不回退的共同 cut 接合两条输入来源；图展示目标因果约束，A-11 必须给出实现这些约束的规则，图本身不是对边界存在性的证明。
- **来源：** 新绘；不纳入 partial safety 或“后来撤销已确认块”的箭头。

### F-11｜输入认证与状态结算

- **位置与目标：** §9.2 P-9.04 后；区分两条证明责任。
- **布局：** 上路 G→batcher→DA/payload→可信 Certifier→inbox；下路 accepted L1+DA→Derivation→native state→状态 claim/原有 fault-proof。
- **关键标注：** BATCH 只绑定 exact input/DA/cursors，state-root correctness 由另一路保证；partial frames 不消费完整块；ModeApplied 控制跨模式接续。
- **Caption：** L1/Espresso 不互相验证，可信 TEE 直接提供业务结论；外部节点可从 L1/DA 派生，提前确认节点另行运行 global rule。
- **来源：** mega-espresso 总览可作参考，但单 publisher 的 selected-record 路径改为 G，TEE 硬件细节删除。

### F-12｜证明依赖图

- **位置与目标：** §10.1 P-10.01 后；防止循环证明与把设计义务藏进假设。
- **布局：** 左列外部假设，中列区域/Authority/生命周期引理，右列 global prefix、settlement correspondence、conditional progress。
- **必须画出：** timer/validation→FINAL kernel→正常 G；A-11 的边界前缀稳定引理→扩展 G agreement；保护策略+native/DA/TEE→原生结算对应；closure+floor+hold→exit safety。
- **区分：** 进展条件用虚线单独进入进展结果，不画成 safety 的前提；边界策略和 protected-prefix 是内部实现/论证节点，不放入“外部 L1 假设”列。
- **Caption：** 条件性结果的依赖必须真实存在；D1/D2 未闭合时，作者工作图可标空框，定稿必须补齐或明确降低陈述范围。

### F-13｜附录中的 entry 与 commit-window 时间图

- **位置与目标：** 附录 B.2 末、P-B.04 后；B.1 先引用 panel (a)。支撑恢复 entry-skew 与正确 leader 进展。
- **Panel (a)：** 各正确节点 freeze 时间 a_v、首次取得完整 REC 的 d_v，以及 entry=max(规定等待,authority availability)；标出完整 source relay 导致的区域界。
- **Panel (b)：** 首个正确 terminal-QC holder 的真实时间 $\tau_Q$、后继提议与全体处理上界；正确 recovery initializer 的 STATUS/NEWVIEW/ANCHOR/首个 fresh vote 窗口。
- **Caption：** q 保留给 Espresso 高度，QC 接收时间用 $\tau_Q$；界从正确节点验证完整 QC 起算，不从 Byzantine 私下凑齐 QC 起算。所有 native/source 依赖必须满足该条引理的处理前提。
- **来源：** Part I 的 timing/recovery 图与 Part II 的 source relay 规则，重写成加权、固定 session 的版本。

## 4. 表、定义、假设和结果清单

### 4.1 表

| ID | 放置位置 | 必须包含的列/内容 |
| --- | --- | --- |
| T-01 | §1.1/§3.5 | 角色、持久状态、接收内容、产生内容、决定权限；所有权只定义一次。 |
| T-02 | §3.1 | v3 记号、作用域、是否 immutable；补 $\mu,p,\mathfrak B,pc$，保留 q/t/epoch 的类型区别。 |
| T-03 | §2.2 | AS 编号、安全/活性条款、适用模式、服务计时起点、使用它的结果。 |
| T-04 | §3.3 | message type、signed context、slot、payload、quorum 条件、是否触发 timer。 |
| T-05 | §3.3 | gen-zero/recovery/term 后继/后续 fresh/DRAIN 各自的 parent 与 justification。 |
| T-06 | §4.1 | Authority 的全部状态字段、键、值、初始化、更新入口和不可变条件。 |
| T-07 | §4.4 | offence type、同一 cid 下的冲突槽、需要的 evidence、非 offence 情形。 |
| T-08 | §6.3 | replay state、source cache、Tail 四种类型、Done/pc；何时可修改及谁负责持久化。 |
| T-09 | §7.4 | term/Quit/regular/Force/STOP/RESUME 的触发、改变的索引、anchor、policy、旧实例处理。 |
| T-10 | §10.1 | R-01…R-10、前提、精确结论、正文证明思路位置、完整证明位置。 |
| T-11 | §8.7/§11.3 | region/Espresso/DA/source/control 各类故障、允许继续的路径、需要等待的对象、恢复所需条件。 |
| T-12 | 附录 A 开头 | I-01…I-38、owner、调用方、主文协议块与完整字段定义位置。 |

T-09 是补充导航表，不独立定义第二套状态转移。
T-03 不是以“假设”承诺内部 global rule 自己应证明的结论。

### 4.2 公共定义与假设

| ID | 名称/位置 | 正式内容 |
| --- | --- | --- |
| D-01 | session/view，§3.1 | 四层索引、成员与一次 session 的固定字段。 |
| D-02 | 区块与位置，§3.2 | DATA/DRAIN/native projection；全局 deposit-only envelope 的扩展引用 §8。 |
| D-03 | policy/plan/context，§3.1 | policy、plan ID、C、cid 与固定 anchor 绑定。 |
| D-04 | 签名与证书，§3.3 | signed domains、quorum、slots、rank/justification 类型。 |
| D-05 | admission/quit proof，§3.3 | 完整准入证据和三类 quit proof，避免在 §4 只有未定义的 context/evidence。 |
| D-06 | Handoff/Ready，§3.4 | 固定 descriptor、原 commitment evidence、可执行 state 与 entry guard。 |
| D-07 | regional finality/kernel，§3.4/§6.2 | direct/covering FC 与 authentic producing context，kernel 是检查过程而非 validity 公理。 |
| D-08 | G 与 replay state，§6.3 | 链连续性、来源元数据、Tail、genesis/checkpoint 和 pc。 |
| D-09 | Fin/Eligible，§6.4 | 真实公开对象集合、parent/height/session/reference 条件和确定性选择。 |
| D-10 | mode boundary，§8.2 | cuts、规范输入、prefix stability 与 actual ModeApplied，具体产生规则由 A-11 完成。 |
| D-11 | protection profile，§8.3 | A/B 所选配置的准入/交付/不可替换契约及适用范围。 |
| D-12 | global finality，§6.5 | observer 何时确认 G；§9 引用本定义建立 native derived history 的对应关系，state-claim settlement 保持独立。 |

AS-01…AS-08 的完整内容分别由 P-2.03…P-2.10 写成命名 assumption 环境。
服务中的大小、负载、在线、持有完整证据和触发时间必须写进量词范围；不使用“足够快”“可靠服务”等无边界措辞代替。

### 4.3 目标结果与证明位置

以下是稿件要建立的陈述清单，**不是本提纲宣告已经验证的结果**。

| 结果 | 精确范围 | 主要证明位置 |
| --- | --- | --- |
| R-01 Authority determinism/immutability | 固定输入产生同一 plan/leader；历史 snapshot 与 vote weights 不改。 | 主文10.03，附录C.1。 |
| R-02 Regional safety and validity | 固定 authentic context、同步/故障/启动前提成立且尚未关闭时，正确区域 commitments 相容有效。 | 主文10.04，附录B.3–B.6。 |
| R-03 FINAL-kernel soundness | raw threshold 有正确 signer，其真实 timer/完整验证支持精确 DATA 及同 session ancestry。 | 主文10.05，附录B.7。 |
| R-04 Synchronous initial entry | complete ReadyBundle 与 prepared-node 前提下，首个正确 Enter 后区域界内其他正确成员进入。 | 主文10.04，附录B.1。 |
| R-05 Global agreement/persistence | 已实现的共同 boundary 策略与一致来源下，正确观察者的 G 前缀相容，已有块/元数据不变。 | 主文10.06–10.07，附录C.2–C.4。 |
| R-06 Lifecycle isolation | 单次初始 Enter、终止后不复活、独立核心状态、关闭后的输出不能跨全局 cutoff。 | 主文10.09，附录C.6。 |
| R-07 No false or duplicate slashing | 在明确的正确签名纪律与 key-compromise 范围内，claimed context 不能转用诚实 key 证据，重复同 offence 不重复扣款。 | 主文10.10，附录C.7。 |
| R-08 Safe release | 实际 closure、单调 future floor、证据/保留服务与适用 hold 条件下，释放 position 不再承担可用旧职责。 | 主文10.10，附录C.7。 |
| R-09 Settlement correspondence | 选定 protection profile、原生有效性、DA/TEE/inbox/状态证明条件下，G 的精确原生块不被后续同高度替换。 | 主文10.08，附录C.5。 |
| R-10 Conditional progress | NORMAL 完成/增长、STOP后的派生、RESUME后三个分立条件结论；不含无条件区域公平或每交易包含。 | 主文10.11–10.14，附录B.8–B.10、C.8。 |


## 5. 同一 TeX 中的附录逐段提纲

### 附录 A｜完整内部接口目录

T-12 放在本附录开头，并将每个接口回链到 owner 与主文 A 协议块。
下面每个 I 项固定两份列表和一个功能段；`P-A.xx` 就是该接口在正式稿中唯一的功能说明段，不在主文重复全文。
caller/rid/chain/rule version 的共用字段集中定义，字段内容和来源不得被一句“其他 deps”省略。
来源读取、网络与 DA 的基础操作由 §2 假设和原生规格承担，不另列八个外部 ideal-functionality 接口。

#### I-01｜Builder.SubmitTx

**输入**

- 完整原生签名交易 tx，含 chain、nonce、target、value、data、gas/fee 字段。

**输出**

- Received(txid) 或 REJECT(reason)。

**P-A.01｜功能段。** 说明本地接收/去重，不授予包含或最终性。

#### I-02｜Builder.Build

**输入**

- session s、精确 parent 与 parentState、nativeContext（height/time/origin/deposits/system rules）、块与 mini-block 限制。

**输出**

- 含有序普通交易与 mini-block 划分的候选 x，或 WAIT/REJECT。

**P-A.02｜功能段。** 固定 parent/context 上构块，生产权限由 Program 决定。

#### I-03｜Builder.ValidateCandidate

**输入**

- parent、parentState、候选 x 与固定 nativeContext。

**输出**

- VALID(B_native,st(B),root)、INVALID 或 WAIT。

**P-A.03｜功能段。** 核对 mini-block/完整块的输入与执行对应，不代替区域 admission。

#### I-04｜Authority.Join

**输入**

- region i、operator key pk、withdrawal owner、实际 escrow 金额 b、key-possession proof、ownerAuth。

**输出**

- 唯一 position ID、Joined 事件、Pending/queue 更新，或 REJECT。

**P-A.04｜功能段。** 原子收款和注册 whole position，保持 key/ID 约束，不即时改历史权重。

#### I-05｜Authority.Cancel

**输入**

- 未被冻结快照使用的 position o 与 ownerAuth。

**输出**

- Cancelled(o,refund)、queue/key/escrow 更新，或 REJECT。

**P-A.05｜功能段。** 仅取消尚未承担冻结快照责任的加入请求，退款一次。

#### I-06｜Authority.RequestExit

**输入**

- 已进入冻结快照且没有待处理 exit 的 position o 与 ownerAuth。

**输出**

- ExitRequested(o,requestPosition)、Exiting/outbound FIFO 更新，或 REJECT。

**P-A.06｜功能段。** 不可撤销地申请未来退出，不解除当前 duty 或直接释放余额。

#### I-07｜Authority.CutSnapshot

**输入**

- epoch e，以及账本内部的前驱快照、严格早于 cutoff 的状态和队列；调用者不能传替代权重表。

**输出**

- 唯一 C_e、publication height、队列/phase/exit-index 更新，或 WAIT/REJECT。

**P-A.07｜功能段。** 按确定截止与预算冻结每 epoch 权重，迟发布仍返回同一内容。

#### I-08｜Authority.ReadAuthority

**输入**

- 确定 finalized L1 高度与 Position/Snapshot/Ban/REC/Controls/Clearance 的有类型 query。

**输出**

- 完整指定范围的状态/记录及来源依据、verified absence 或 WAIT。

**P-A.08｜功能段。** 给本账本状态一个查询约定，不重新定义 L1 ideal functionality。

#### I-09｜Authority.Plan

**输入**

- 固定 policy、目标 beta/r、selection reference E 与 source region，以及相应固定高度的 snapshot/ban 来源。

**输出**

- Pi 的 region、固定成员/权重/W/F、TB/K/Delta、ban、source height、ID；无 region 或 WAIT 分开。

**P-A.09｜功能段。** 一次固定切面的确定性选择，引用块与执行 anchor 的职责分开。

#### I-10｜Authority.Auth

**输入**

- 完整 C、generation g，以及 g>0 时对应 REC 与其高度上的 ban 集。

**输出**

- AUTH_sg，或缺少 finalized REC 的 WAIT。

**P-A.10｜功能段。** 冻结一代的 proposer eligibility，保留该 session 的原投票权重。

#### I-11｜Authority.Leader

**输入**

- 固定 AUTH_sg 与 term t。

**输出**

- 指定 leader 成员/公钥，或 verified 空集合对应的无 leader。

**P-A.11｜功能段。** 按固定 seed/权重规则选择，不能将可预测 schedule 当成已证明公平。

#### I-12｜Authority.SubmitOffense

**输入**

- C、被指控 position o、同 key 的两条完整签名消息、kind/slot 与其需要的 admission/quota 材料。

**输出**

- Slashed、ban 与 SeenOffenses 更新，AlreadyProcessed 或 REJECT。

**P-A.12｜功能段。** 相对真实快照与同 cid 证据扣减当前 escrow，保持历史权重并防重复处罚。

#### I-13｜Authority.SubmitQuit

**输入**

- C、generation g、完整 BLAME/proposal-equivocation/NEWVIEW-equivocation proof 和准入材料。

**输出**

- 唯一下一代 REC，必要的先行 slash/ban，或 REJECT。

**P-A.13｜功能段。** 所有 quit 类型统一报告，纯 BLAME 不扣款，裸 counter 不占用恢复槽。

#### I-14｜Authority.SubmitUpdate

**输入**

- chain、序号 c、Espresso 阈值、目标 beta、完整 S_new/L_new、multisig 认证。

**输出**

- 带 L1 位置的不可变控制记录，或 REJECT。

**P-A.14｜功能段。** 记录普通 Force 意图；实际匹配、prelude 和 stale 语义由公共回放执行。

#### I-15｜Authority.ScheduleMode

**输入**

- chain/c/target、STOP 或 RESUME、L1 截止/最早激活条件、boundary 材料/标识、必要 policy 与 multisig 认证。

**输出**

- ModeScheduled 记录或 REJECT，区分已公告与待完成边界。

**P-A.15｜功能段。** 公告由 L1 驱动的模式变化，不凭公告直接改变已确认 G 或释放 stake。

#### I-16｜Authority.ProveClosure

**输入**

- 可信 CLOSURE 消息，绑定实际控制 c、关闭版本、实际生效来源位置和 future floor。

**输出**

- ClosureRecorded、递增的 clearance 水位与持有期起点，或重复/不适用结果。

**P-A.16｜功能段。** 使用可信业务结论清退旧职责，不在 L1 内验证 Espresso header。

#### I-17｜Authority.Release

**输入**

- position o、ownerAuth，以及账本内部 e_x、首个覆盖 clearance、hold 和当前余额/处罚状态。

**输出**

- 一次性 Released(o,amount)、phase/key 更新，或条件不满足结果。

**P-A.17｜功能段。** 原子支付剩余 escrow，时间与证据前提不足时不提前解除责任。

#### I-18｜Manager.ObservePublic

**输入**

- 新的完整 finalized L1/Espresso 来源材料，带确定 cut、bytes 与认证依据。

**输出**

- 新 replay state、G/Tail/mode、确定的 lifecycle events 或 WAIT。

**P-A.18｜功能段。** 存储后运行唯一 global rule，不按到达顺序混排控制与数据。

#### I-19｜Manager.ObserveRegional

**输入**

- 完整 RegionalResult 与建立其 authentic producing context 的 Handoff/source 链。

**输出**

- 持久化结果、当前 R 的更新或 successor Handoff，或 WAIT/REJECT。

**P-A.19｜功能段。** 用于直接 handoff/准备；关闭版本只作历史保管，不直接追加 G。

#### I-20｜Manager.InstallHandoff

**输入**

- 固定完整 Handoff 与本机成员身份。

**输出**

- Assigned(s,C)、现有实例、NotAssigned、WAIT 或 REJECT。

**P-A.20｜功能段。** 区分历史验证与创建权限，拒绝同 pair 的不同固定 opening。

#### I-21｜Manager.PrepareAndEnter

**输入**

- Assigned session 和精确 anchor 的完整 state/provenance 或可重建输入。

**输出**

- ReadyBundle、单次 Enter 结果，或缺依赖的 WAIT。

**P-A.21｜功能段。** 准备完真实 state 后复查权限，不能将安装工作藏在通信时限之外。

#### I-22｜Manager.HandleProgramEvent

**输入**

- session 与 Result/Evidence/Offense/Complete 的完整有类型事件。

**输出**

- 发布、报 L1、relay 的持久化任务和相应程序 phase 更新。

**P-A.22｜功能段。** Program 输出转化为不会随实例退出丢失的交付责任。

#### I-23｜Manager.ApplyControl

**输入**

- 公共规则产生或可重放验证的 ControlEvent 与对应 source/boundary 材料。

**输出**

- Closed 标记、被终止实例、确定的新 Tail/Handoff 或 WAIT/no-op。

**P-A.23｜功能段。** 落实公开生命周期结果，不自行选择 retained prefix；OPEN 时不提前创建新运行实例。

#### I-24｜Program.Create

**输入**

- session s 与已验证固定 Handoff。

**输出**

- 绑定独立状态的 Assigned 实例、原实例或 REJECT。

**P-A.24｜功能段。** 创建不等于进入，也不复制前驱投票/锁/计时器。

#### I-25｜Program.Enter

**输入**

- session、完整 ReadyBundle、本机 Closed/entry/termination 记录。

**输出**

- Entered、完整 bundle 广播与 initialized Running state，或 REJECT。

**P-A.25｜功能段。** 原子检查后只进入一次，保存 entry 再发送，并按证据优先顺序启动。

#### I-26｜Program.Provide

**输入**

- 指定 session 的 Candidate(parent,x,nativeContext)、来源 references 或 Auth(g,AUTH_sg)。

**输出**

- Installed(kind,id)、WAIT 或 REJECT。

**P-A.26｜功能段。** 验证后提供输入，不因收到对象就改变固定 context 或获得新投票权限。

#### I-27｜Program.Step

**输入**

- session 与 Receive(完整签名消息) 或真实本机 Timer(timerId,tau)。

**输出**

- peer messages、Result/Evidence/Complete 事件、更新状态或等待任务。

**P-A.27｜功能段。** 执行 A-04/A-05；消息输入与 timer 输入共享 guard 和证据优先纪律。

#### I-28｜Program.Close

**输入**

- session 与已验证覆盖其版本的 ControlEvent。

**输出**

- Terminated 与回执/证据/待发任务的移交记录。

**P-A.28｜功能段。** 禁止后续 BFT 动作但保管既有事实，不把终止当作 Completion 或退款。

#### I-29｜Publisher.Offer

**输入**

- C、B、FINAL root D、Sigma、完整 path，以及 authentic handoff/source 材料。

**输出**

- Queued、Submitted、Recorded 的稳定标识与公开位置，或 WAIT/REJECT。

**P-A.29｜功能段。** 交付完整结果；published 不等于 globally accepted。

#### I-30｜Batcher.BuildPayload

**输入**

- G 中的连续普通区间、finalized inbox cursor、已验证 mode/replay state 与本地 channel state。

**输出**

- 同一 channel 的连续新 frames、后继 channel state 与待认证范围。

**P-A.30｜功能段。** 允许重打包但不允许自选 R 中的 speculative 内容或擅自跨模式跳过高度。

#### I-31｜Inbox.SubmitBatch

**输入**

- 可信 BATCH 消息和当前满足提交权限的 caller。

**输出**

- CursorAdvanced(prev,next,d) 及 cursor 更新，或 REJECT。

**P-A.31｜功能段。** 信任 TEE 对内容/DA 的业务结论，同时原子核对本链 prev cursor、权限与 mode。

#### I-32｜Inbox.ApplySettlementBoundary

**输入**

- 可信 MODE、对应 finalized 控制、本链 cursor 和 protected-prefix 条件。

**输出**

- 不可变 ModeApplied、相应 cursor/channel/mode 更新，或 WAIT/REJECT。

**P-A.32｜功能段。** 提交模式边界而不丢弃受保护输入；实际 L1 提交位置由执行事件给出。

#### I-33｜StateClaim.Submit

**输入**

- 高度区间、parent claim、确定 L1/DA/mode 输入引用、claimed state root。

**输出**

- ClaimRegistered，及原有证明系统后来产生的 Resolved/Rejected/Pending 状态。

**P-A.33｜功能段。** 连接外部原生状态证明，不由 BATCH certificate 代替执行结算。

#### I-34｜Certifier.CertifyBatch

**输入**

- prev cursor、精确 payload、DA 绑定；必要已接受 frames 从公共来源取得。

**输出**

- 可信 BATCH(chain,version,prev,next,d,range,globalRef)，或 WAIT/REJECT。

**P-A.34｜功能段。** 自行重放 G 并核对完整输入，未闭合 channel 不提前消费块，也不承诺未来交付活性。

#### I-35｜Certifier.ConfirmControl

**输入**

- 控制序号 c 与 MODE/CLOSURE purpose；业务规则要求的来源与边界从公共历史取得。

**输出**

- 候选边界 MODE 或实际关闭 CLOSURE，包含对应的 cuts/anchor/floor 字段，或 WAIT/REJECT。

**P-A.35｜功能段。** MODE 可先于自己的接受事件，CLOSURE 必须对应已执行关闭；可信性不替代 A-11 的边界算法。

#### I-36｜Public.CheckFINAL

**输入**

- authentic C、目标 B、root D、固定权重签名集合与精确 path。

**输出**

- VALID、INVALID 或 INCOMPLETE。

**P-A.36｜功能段。** 唯一公共 kernel 的输入输出约定，行为由 A-07 定义。

#### I-37｜Public.AdvanceGlobal

**输入**

- 持久化 X、PublicStore、新来源材料和固定的边界/protection 配置。

**输出**

- X'、G/Tail、追加/生命周期事件及 OK/WAIT/INCONSISTENT。

**P-A.37｜功能段。** 唯一回放调用，不能让每个 caller 另选边界策略或重新排序输入。

#### I-38｜Observer.Derive

**输入**

- 已验证 native derivation state、完整 L1 输入、accepted DA objects 与固定 rule version。

**输出**

- native blocks、状态/roots、精确进度及接续材料，或 WAIT。

**P-A.38｜功能段。** 为晚结算观察者重建真实链，并提供与 G 接合所需的确定性原生结果。

### 附录 B｜区域核心的完整论证

每个 B 小节含两段计划证明；需要的正式 lemma/theorem 放在这两段之前。
它们必须在同一 `main.tex` 中展开，不用 Part I 的文件链接代替证明。

#### B.1 Entry、freeze 与 source availability

**P-B.01｜传播与进入。** 以首次正确 freeze/source holder 为起点，利用 A-10 的完整即时 relay 证明正确 prepared 节点取得 proof/REC 的区域传播界；不要由全局时钟推出同时可用。

**P-B.02｜取最大值后的 entry-skew。** 对每个节点的 freeze 等待与 authority availability 写明确的 max 表达式，证明适用条件下其差不超过区域界；初始 ReadyBundle 另作基例。引用 F-13(a)，覆盖未来 proof 已缓冲、进入后立即冻结的情形。

#### B.2 terminal QC 与正确 leader 的时间

**P-B.03｜可用 QC 的起点。** 从首个正确节点验证完整 terminal QC 的 $\tau_Q$ 起算，证明转交、后继进入/提议和正确节点收到 proposal 的界；不能用私下 QC formation time。

**P-B.04｜进度时间线。** 按初始/恢复两种 entry 展开 leader 消息和 vote，解释为何 progress checks 不应误触发。显式列出 source/native inputs 已在预算内可用的前提，在此插入包含两个 panel 的完整 F-13。

#### B.3 同代链或冲突证据

**P-B.05｜第一处分歧。** 从完整签名上下文和同一 fresh slot 出发，分析同代两条不相容 proposal 历史；对应到可验证的 leader equivocation，而不是只依赖两个 quorum 的集合交集。

**P-B.06｜权重适配。** 将原人数证明中的“有正确 signer/正确成员可组成证书”逐一替换为 $>F$ 和 $W-F>F$，保留传播与时间步骤；列出不能仅做符号替换的地方。

#### B.4 Commit timer 的保护

**P-B.07｜存活的 timer。** 从首次 vote/forward 到 $2\Delta_i$ 到期，证明完整 proposal/votes/QC 的必要到达关系。分析消息在到期时同时到达时优先处理的作用。

**P-B.08｜Completion 与取消。** 若某正确节点提前 Completion，其完整 terminal receipt 会在规定界内传播并取消相关 timer；因此不能把已停止投票的成员悄悄计入一个仍存活的 commit window。Force/STOP 之后被丢弃后缀不继承这条区域保证。

#### B.5 Recovery locks 与扩展性

**P-B.09｜固定 entry lock。** 在 entry 时固定 Q_v，证明合法 NEWVIEW 的 rank/ANCHOR 条件如何保留先前受保护的 commitment；晚到 QC 不重写 entry lock。

**P-B.10｜跨代推导。** 对 generation/rank 归纳得到唯一可扩展性，处理 inherited Q^0 与只属于本 session 的 QCs。明确 OTHER-session QC 不能代替本 context 的 justification。

#### B.6 有限 quota、DRAIN 与区域安全

**P-B.11｜有限 endpoint 的适配。** 把 Part I 的安全论证接到固定 A/H、部分最后 term 和 drain generation；DRAIN 不消耗 quota，但能给 terminal ancestor 提供 timer。

**P-B.12｜精确陈述。** 陈述 R-02 的有效期、准备/验证/故障前提，以及 direct/ancestor commitments 的处理。不能由 Part I 的 physical-height 进展直接推出无限 DATA 或强制关闭后 discarded suffix 的 agreement。

#### B.7 FINAL kernel 的非循环证明

**P-B.13｜从 raw signatures 到真实事件。** 权重阈值和不可伪造性给出正确 signer，追到其直接 timer 或 ancestor timer commit，继而得到真实完整验证。不要在 RawFINAL 定义里预先假定要证明的应用 validity。

**P-B.14｜继承与 covering 的归纳。** receipt-induced commit 不发新 FINAL，按真实事件先后对 inherited anchor/早先 timer 归纳；hash binding 保证验证路径就是正确 signer 的精确输入。仅推广到同 session ancestors。

#### B.8 排班与排除的进展前提

**P-B.15｜有限已暴露身份。** 在固定有限委员会及最终生效 ban 条件下，论证可被未来 admissible proof 指认的已暴露 Byzantine leader 范围；不把中断次数粗称为至多 f。

**P-B.16｜正确机会。** 明确正确 initializer/term leader 机会与输入可用性的 fairness 前提；公开 WeightedRand 只是排班构造，不自动证明这些前提。

#### B.9 Session completion

**P-B.17｜有效 commit window。** 在合适正确 leader、无新的有效 equivocation 干扰及来源可用条件下获得一个存活 fresh timer；区分普通 DATA 与 endpoint DRAIN 的作用。

**P-B.18｜到达 terminal receipt。** 将有限 quota 的进展接到 terminal DATA 的 commitment 和正确权重形成 FC，再通过完整传播得到 Completion。跨 session 的无界 G 高度还需 Manager、publication 和 control 的额外前提。

#### B.10 区域保证与全局进展的边界

**P-B.19｜有效期结束。** 说明合法关闭可能取消未完成工作，区域 proof 只用于关闭前适用执行；对 discarded suffix 不额外承诺 completion/agreement。

**P-B.20｜交易级活性。** 若以后要加普通用户交易包含定理，需要单独的 builder/input fairness 与持久待处理规则；本稿当前只使用完成/高度进展，不重新设计区域 inclusion fairness。

### 附录 C｜组合性质的完整论证

#### C.1 Authority 不变性

**P-C.01｜确定来源。** 用 snapshot cutoff、固定 policy、固定 E 和 immutable REC 证明 R-01；missing evidence 的 WAIT 不改变选区或采用旧快照。

**P-C.02｜历史权重与角色。** 区分 escrow、snapshot weight、proposer eligibility 和 duty，说明后续 slash/control 不重新解释历史证书；源高度与 future floor 的 max 规则必须覆盖 RESUME。

#### C.2 NORMAL 的共同解释

**P-C.03｜相同公开集合。** 完整 namespace 与固定对象解析给出相同 O_q、Fin 和 Eligible 集；外层 hints/optional sidecars 不成为额外输入。

**P-C.04｜同一选择与元数据。** 对 prelude、least-hash、regular boundary 和 OPEN 解析逐步归纳，得到相同追加与 sigma；后台读取进度可不同，只能导致 WAIT。

#### C.3 模式边界的前缀稳定

**P-C.05｜A-11 的实际构造证明。** 使用最终选定的公告/截止/封口机制，证明未来来源扩展不插入或删改已暴露规范输入。此段须由 D1 的实际步骤完成，不能以结论本身作为外部假设。

**P-C.06｜两种切换的共同性。** 分别证明 STOP 不等待新 Espresso block、RESUME 不截掉已确认 deposit-only 前缀；阈值相等、迟到证书、先取回更高来源和重复公告都给出一致处理。

#### C.4 扩展全局前缀

**P-C.07｜按内部转移归纳。** 对 N/S/D/R 的每种状态变化逐项检查：只有 N/D 增加块，其他步骤不删历史、不改 sigma，floor 不减，source pc 原子持久化。

**P-C.08｜跨观察者与重启。** 利用 C.3 的共同输入前缀，将不同等待点、partial processing 和重启恢复映射到同一确定性执行前缀，得到 R-05。INCONSISTENT 不以回滚“修复”。

#### C.5 原生结算对应性

**P-C.09｜正常输入链。** 连接 authentic FC、G、可信 BATCH、inbox 接纳、DA binding 与确定性执行；解释 native/global hash 类型与输入投影。

**P-C.10｜保护与模式接续。** 使用 D2 选定的 profile 覆盖每个普通块从进入 G 起的责任，处理 partial channel、已存在 native fallback、STOP backlog 与恢复 anchor。明确证明的是同高度精确内容/执行对应，不是 claim 一经提交就已结算。

#### C.6 实例隔离

**P-C.11｜本地许可不变量。** 对 Create/Enter/Complete/Close 与重复调用归纳，证明单次初始进入和不可重建终止实例，核心状态不跨 session 复制。

**P-C.12｜公共 cutoff。** 本地尚未收到关闭通知的行为不反向改变 global interpretation；同一个关闭标记覆盖 speculative successor 和未创建 pair，得到 R-06。

#### C.7 Slash 与 release

**P-C.13｜无错罚与去重。** 将 R-07 建立在明确的 individual signing discipline、固定 cid 与真实 snapshot 上，逐项核对 offence slot；特别检查 forced termination 后的个人签名记录，不能仅从“Force 前区域 agreement”推出全部处罚安全。若实现缺少必要的不可覆盖签名记录，须先补入 Program state/guard。

**P-C.14｜退出责任。** 从实际 closure、关闭全部旧实例、后续 floor 排除退役 position 和证据预算推导 R-08；hold 从真实接受的 clearance 时点起算。deposit-only 期间新退役者的释放若需下一次闭合，就明确保留该等待成本，不能自动解锁。

#### C.8 分阶段进展

**P-C.15｜正常阶段。** 将区域完成、直接/公开交付、global acceptance 与 successor preparation 的前提串联；引用 B.9 而非假定 Ready 就能产生 FC。

**P-C.16｜恢复阶段。** 按控制、保护交付、L1 派生、共同恢复点、Espresso 服务和 Ready 分段给出 R-10(b)(c)。每个时间预算说明起点与覆盖路径，未给预算的阶段只陈述 eventual/conditional 结果。

### 附录 D｜参考控制策略与边界案例

本附录是解释/可选控制策略，不能产生绕过正文验证规则的权限。
每个段落对应一个检查场景，作为后续实现与证明核对的输入，不把它们当作实现已通过的测试。

**P-D.01｜参考控制服务。** 给出控制者能观察的公开状态、BOOT/STALL/FAULT/MAINT 等现有触发条件及适用范围；它们只有提交有效控制后才产生动作，timer 本身不强制切换。

**P-D.02｜阈值与稳定服务窗口。** 在声明了 first/next complete-result 界的条件下，写出 $\tau_{\rm boot}\ge T_{\rm ready}+T_{\rm first}+T_{\rm pub}+T_E$ 与 $\tau_{\rm stall}\ge T_{\rm next}+T_{\rm pub}+T_E$；这些区域服务界不是从公平排班自动推得。注明 control honesty 不保证响应，连续 Force 可阻止进展；若保留“不误打断服务区域”的命题，完整列其条件，不称作完整恢复定理。

**P-D.03｜相同时间戳与多条控制。** 明确 ordinary activation 唯一性是额外发行/模型约束，不由 T_update 自动推出；模式切换对旧 pending 控制按固定 source cut 处理。

**P-D.04｜零产出、重复 Force 与 PENDING。** 检查 a=0 时 r 不增、a>0 时 r 增，PENDING 没有 program；全 OFF 或无 eligible proposer 不靠时间自动变更。

**P-D.05｜terminal QC、晚 DRAIN 与早交接。** 展示 QC≠FC、DRAIN 的作用、terminal receipt 到达先后与 regular successor 的固定性。

**P-D.06｜补齐对象与丢失缓存。** 分别讨论 body/signature 尚未公开、已经公开但本地没取到、恶意 optional sidecar 和重复 publications，确保 WAIT 与无效候选的处理不混淆。

**P-D.07｜关闭和回调竞争。** 对 receipt 先到、closure 先到、ReadyBundle 迟到、source callback 重入和进程重启逐项说明结果；都必须落到同一 Closed/entry/pc 规则。

**P-D.08｜模式切换前的积压。** 用 G108/native105 检查受保护块不能填空，已接受 DA 不可取回时必须等待，替代 batcher 的资格不自动保证剩余窗口够用。

**P-D.09｜恢复点与迟到旧结果。** 用 deposit-only 到120、普通从121接续，检查旧109、旧 future-target control 和较早恢复证书都不能截断已确认输出。

**P-D.10｜资金退出与证据尾部。** 检查 epoch cutoff 前后请求、slash 与 REC 同序处理、实际 closure 前/后的 ProveClosure、hold 边界和 custody transfer。明确无期限的证据隐藏不被有限 hold 处理。


## 6. 定稿前必须闭合的设计工作

本节只服务于作者工作流，不进入最终论文的公开编号体系。
它说明哪些位置已有来源可写，哪些位置必须完成设计才能把“条件性抽象”写成“当前配置的完整协议”。

### D1｜实现共同切换边界，而不是假设它存在

**当前已有。** 抽象 v2 给出了 cuts、规范输入类型和前缀稳定契约，STOP/RESUME 的状态转移已能在给定正确边界时描述。
**仍需完成。** 明确公告之后、模式提交之前如何限制新确认；如何唯一封口 source cut；如何处理相同/滞后的时间戳、迟到旧材料和不同读取进度。
时间阈值若用于 cut，须解释它如何约束未来迟到 publication，不能只凭当前最新 header 推断“不再有此前时间戳的数据”。

**定稿交付物。** 可执行的 A-11、F-10 的确切事件顺序，以及 C.3 的前缀稳定/停机可完成证明。
如果具体实现引入额外可信许可或正常路径等待，应在 AS-07、F-01、进展结果和成本讨论中明示，而不是隐藏在 $\mathfrak B$ 名称里。

### D2｜选择并落实已确认前缀的保护 profile

**A 路径。** 给出从每块进入 G 起到正确交付的完整预算与剩余 sequencing-window 条件，包含 batch accumulation、channel、DA、certifier 业务处理、替代提交资格、L1 和模式衔接的必要时间。
不要把“允许另一个 batcher 提交”写成“它已经来得及提交”。

**B 路径。** 写出 inbox、derivation 和原生状态证明实际识别/保护已确认前缀的规则，以及补交、部分 channel、数据等待和普通 mode 的处理。
不能只在观察者标签中称某块 finalized，而让同高度在底层被 fallback 替换。

**定稿交付物。** D-11 的确定配置、A-12/A-13/A-15/A-16 的兼容条件、C.5 的结算对应证明；若保留两个配置，必须分别说明适用假设和性质，不用一个未选的 A-or-B 作为完成声明。

### D3｜固定块类型、引用和模式/协议接口

**必须写清。** normal envelope 与 native block 的一一对应、两个 parent/hash 类型、deposit-only envelope 的来源字段、恢复 anchor 的 L1 证据、$\lambda(B^{dep})$ 与 native origin 的区别。
固定 OPEN/PENDING/DEPOSIT 类型、mode activation 与 actual ModeApplied 的区别、Done 的公共来源范围和 pc 的原子重入行为。

**定稿交付物。** D-02/D-06/D-08/D-10、T-04/T-08、附录 A 与 A-08…A-16 使用同一字段集合。
MODE 候选和 CLOSURE 事实不得互换；trusted TEE 不引入链间 light-client 验证接口。

### D4｜闭合恢复期间的责任、签名纪律与释放

**必须写清。** 个体签名槽的持久化纪律，尤其是 FINAL 与强制终止交界；actual closure 如何覆盖全部旧实例和预计划职责；future floor 如何排除退役 stake。
分别推导健康 ordinary Force 与 L1 STOP 下的证据/保留/处罚窗口，不把停机时不存在的有限 $T_E$ 放进 release 保证。

**定稿交付物。** A-02/A-04/A-10 的必要状态与 guard、P-4.15 的明确持有期范围、C.7 的 R-07/R-08 证明。
若某类退出只能等待下一次有效 closure，应作为设计成本明确保留，不能在没有证明时宣称无条件安全快速释放。

## 7. 从现有文档迁移到单篇稿件

| 现有来源 | 新稿位置 | 迁移方式 |
| --- | --- | --- |
| Part II v4.9 Overview/例子 | §1、F-02、附录D | 用总数据流和固定例子重写，避免重复多轮修订历史。 |
| Part II Model/assumptions | §2 | 保留真正外部条件，拆清健康/停机活性；不保留理想链 API 章节。 |
| Part II Regional engine definitions | §3 | 统一符号、类型、消息、quota、admission 和父关系。 |
| Part II Regional protocols | §5、A-03…A-05 | 正文完整重述，不写“在 Part I 上加以下差异”来省略规则。 |
| Part II Regional/global finality | §6、A-06…A-09 | 保留公开 kernel 与对象集语义，扩展统一 state/pc；Force 只定义一次。 |
| Part II Cross-region rotation | §7、A-10 | 保留固定 handoff、two-state preparation、ReadyBundle 和关闭语义。 |
| Part II security/lifecycle appendix | §10、附录B/C | 将 conditional scope 显式迁入陈述，新增 mode 的部分重新证明。 |
| Part II reference controller appendix | 附录D.1–D.4 | 作为参考策略，不能暗中成为所有部署必有的自动 Force。 |
| Part II storage/source relay appendix | §3.5、§7.4、附录B.1 | 正确节点持久化职责是协议的一部分；时间论证使用真实完整 relay。 |
| Part I steady/recovery 与 proofs | §5、附录B | 吸收必要规则与证明，处理加权门槛、有限 quota、Completion、DRAIN 和 closure。 |
| Part III v6 ledger/authority/accountability | §4、附录A/C | 合并 staking 与 multisig，固定 C/state/epoch 表，closure 改用 trusted message。 |
| Part IV v0.1 Publisher | §6.1、§7.4 | 只取备份转发、费用、重试与 custody；不复制旧 NextLive/Force handler。 |
| 抽象 v2 的 §3 external interfaces | §2 的 AS-01…AS-08 | 改成假设文字，源验证/完整性/时限不因删 API 而消失。 |
| 抽象 v2 的内部接口 | 附录A | 38 个内部接口按同一模板给出；主文协议块引用同一约定。 |
| 抽象 v2 的 §5 formal global rule | §6、§8、§10、附录C | 拆到总—分结构；填实 boundary/protection 的内部义务，不把未实现参数当外部能力。 |
| mega-espresso certification/batching/derivation | §9、附录A | 继承输入与帧格式的业务语义，明确替换 publisher authorization 和 record selection。 |
| mega-espresso recovery | §8、§9、附录D | 继承可用 backlog、替代提交、实际 derived-state 接棒等原则，调整与强 G 冲突的丢弃行为。 |
| fault-proof stub 与用户给定 Kailua 职责 | AS-08、§9.3 | 保留外部模块正确性边界，实际引用应在写正文时选定可支持该声明的正式来源。 |

这些来源在稿件内用于归因与相关工作；不能作为“读者要另开四份 Part 才能知道规则”的依赖。
旧文档版本标签、revision color、latexdiff 宏、跨文件 `xr` 依赖和失效章节引用都不迁入。

## 8. 写作顺序与完成检查

### 8.1 建议的落笔顺序

1. 先固定 §1 的总图/运行例子与 §2 的假设边界，确保新稿讲同一个系统。
2. 写 §3 的公共类型和符号，再写 §4 Authority 的全部状态与确定转移。
3. 写 §5 的区域协议和 §7 的生命周期，完成附录 B 的加权/有限-session 适配检查。
4. 写 §6 的正常 global rule，验证正常运行不需要另一份 Force handler。
5. 闭合 D1/D2，再完成 §8；如果暂未完成，稿件状态明确为条件性草案。
6. 按同一模式/输入语义完成 §9，随后写 §10 陈述与附录 C 证明。
7. 最后写讨论、摘要和所有 captions，核对 Appendix A 与 protocol blocks 的字段完全一致。

这只是作者顺序；最终论文仍按 §1–11 的总—分顺序呈现。

### 8.2 正文定稿的核对清单

- 每个协议动作有 owner、输入、输出、持久状态、触发点和失败/等待行为。
- 每个 claim 分清区域 commitment、global finality、native derivation 与 state-claim settlement。
- 每个 source cut、context、snapshot、epoch、cursor 和引用都有固定来源；无 sender-selected authority 或未定义的 `context`。
- 网络/来源假设只提供外部能力；prefix stability、正确 boundary 和 protected-prefix 是内部构造/契约及其证明。
- weighted threshold 的证明未偷换成 $2/3$ intersection；finite quota 的进展未偷换成无限 DATA。
- 初始进入、正常 term 与 recovery 的 entry/progress 时间分别定义，来源 availability 不被假定同时。
- ModeApplied 与 candidate MODE 不循环依赖；关闭/恢复不重写已确认块或 native state。
- 待计划来源的 OPEN、无合格区域的 PENDING、等待数据和 mode 变化保持不同含义。
- 所有 partial frames、迟到对象、旧版本控制、重复处理和重启都由同一 pc/Closed/Done 规则处理。
- 正式稿的每条 theorem 都有完整前提与证明位置；结果清单不把本提纲当成已完成验证。
- 所有图的时间轴、颜色/线型和版本/anchor 示例一致；F-02 的互斥分支不会在后文被错误拼接。
- 单个 `main.tex` 含全部正文、附录、TikZ、宏与 references，独立编译且无对旧 Part 辅助文件的依赖。

### 8.3 本轮实际产物与后续产物

本轮实际创建 `formalization_v4/README.md` 与本提纲 `manuscript_outline_zh.md`。
`main.tex` 是下一阶段的目标文件，尚未创建；本提纲不声称协议已实现、证明已验证或稿件已编译。
后续成稿应在生成正文、图和证明后，分别进行源码一致性、编译/引用检查、渲染检查与数学/协议审阅。

