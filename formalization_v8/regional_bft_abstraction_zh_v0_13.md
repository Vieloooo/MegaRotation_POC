# Regional BFT：接口、Required Properties 与安全目标（工作稿 v0.13）

日期：2026-11-02。  
上一版：[v0.12](regional_bft_abstraction_zh_v0_12.md)。  
共享定义：[arch v0.2](arch_zh_v0_2.md)。

## 0. 范围与共享定义

Regional BFT 在一个固定 context 中决定 physical chain，并输出可公开验证的最终证明和违规证据。本文规定它的接口、实现需要满足的性质，以及这些性质与应用规则组合后要达到的安全目标。

以下定义直接使用 arch，不再分别定义一套：

- **Arch §1–§2：** 签名、PKI、L1 与 DEP 输入、replica、逻辑 session、委员会与权重、故障模型、协议时钟及区域内通信。
- **Arch §3：** Anchor、physical／virtual chain、Apply、VP1–VP3、Merge／Strip，以及区域生产的固定 origin 规则。
- **Arch §4.3：** Program descriptor、Config 和接口参数。本文使用相同的接口签名。
- **Arch §5–§6：** 全局接纳、regular anchor、Force／RESUME 和候选 program 的生命周期；这些由系统层处理。

下文固定 arch 中的一个 descriptor $\nu$，其逻辑 session 为 $s=(\beta,r)$、anchor 为 $A$。保留 arch 的 session 下标：委员会为 $V_s$，权重为 $w_s(m)$，总权重为 $W_s$，故障上界为 $F_s$，参与门槛为 $Q_s$，leader 为 $\mathrm{Leader}_s(v)$。区域内同步参数使用 $\mathrm{GST}_{i_s}$ 与 $\Delta_{i_s}$；窗口和停止时刻使用 $h_{\min,s}$、$h_{\max,s}$、$\tau_s^{\rm start}$、$\tau_s^{\rm stop}$。

只写 cid 时指 $\nu.\mathrm{cid}$。Anchor 的连接标识使用 $\mathrm{id}(A)$，不会将整个执行状态任意编码进 anchor hash。本文不将 session 标识 $s$ 用作时间变量。

本实例的 anchor、配置和窗口在创建后固定。不同候选 anchor 对应不同 cid，各自满足本文的性质；哪个候选最终进入全局链，由 GFR 选择。区域 BFT 不读取 Espresso 来决定其内部投票。

本文列出的是对 BFT 实现的要求。后面的安全目标以这些要求成立为前提，不表示已经证明某个具体实现满足它们。涉及签名和 hash 的结论采用 arch 的密码学假设，并允许相应可忽略失败概率。

## 1. Regional BFT 接口

### 1.1 状态

每个 replica 为每个 cid 保存独立实例状态。

固定部分是 arch §4.3 的 descriptor $\nu$ 和配置 $\mathcal K_s$。可变部分包括：

- entered 与 terminated 标志、进入时刻和关闭原因；
- 当前 view，以及该 BFT 实现的投票、锁、计时器等内部共识状态；
- 已生成的签名记录，以及 proposal 的首次接收时刻；
- 已知 physical blocks、决定前缀和验证所需的 ancestor bodies；
- 执行状态、L1 输入缓存，以及待交付的 Final／Evidence 事件。

具体内部消息和锁的格式由 BFT 实现规定。实例执行规则要求在发送签名前保存对应状态和签名记录；暂停或崩溃后恢复继续使用原状态，不能重新初始化来绕过旧投票。更换 BFT 或 verifier 是协议版本选择，不在同一 cid 内进行。

### 1.2 接口概览

Session Manager 通过 Create、Enter、Close 管理实例。BFT 通过 Config 的 Build、Accept、NotBefore 构造和检查 physical payload。输出事件为 Final 和 Evidence；外部读取者通过 CheckFINAL 和 CheckEvidence 验证它们。

WAIT 表示输入未取得或提议时间未到；REJECT 表示已判定无效。控制接口的重复请求按 arch 的幂等规则处理。下面列出 BFT 边界上的输入、输出和状态效果，完整应用规则使用 arch §4.3。

### 1.3 控制接口

#### Create(Π_s, A)

**输入。** 已验证 Plan 和固定 anchor 的身份、metadata。此时可以尚未取得 $A.\sigma$。

**输出。** 新实例的 Assigned(cid) 与 descriptor；cid 已存在时返回原实例及当前状态；配置或 anchor 不符时返回 REJECT。

**处理。** 固定签名域并创建独立状态。已存在实例不被重置，已终止实例不复活；创建本身不启动提议。

#### Enter(A, π_A, σ_A)

**输入。** 对固定实例提供完整 anchor、来源依据 $\pi_A$ 和精确状态 $\sigma_A=A.\sigma$；读取本 replica 的委员会身份和当前协议时刻。

**输出。** Entered(cid, $\tau_{\rm entry}$)、WAIT 或 REJECT；$\tau_{\rm entry}$ 为首次进入的协议时刻。

**处理。** 核对身份、来源和状态，满足 arch 的进入守卫后，只进行一次初次初始化。可早于 $\tau_s^{\rm start}$ 进入，但 proposal 仍等待 NotBefore；达到 $\tau_s^{\rm stop}$ 或已终止时拒绝进入。尚未终止实例的重复调用返回原进入结果。

#### Close(reason)

**输入。** Manager 给出的关闭原因，或实例到达固定 stop 的本地触发。

**输出。** Terminated(cid, reason)；重复关闭返回原终止记录。

**处理。** 永久停止新共识签名，保留已有 blocks、签名和证据。自动停止不等待 Manager 发出 Close。暂停恢复前也检查关闭标记和 stop；暂停只改变是否调度，不执行第二次 Enter。

### 1.4 应用配置接口

Config 使用 arch 的链数据与执行规则，不读取 BFT 的内部锁来改写应用有效性。BFT 负责选择安全的 parent，Config 检查给定 parent 上的 payload。

#### Build(P, τ)

**输入。** 同一 context 的可扩展 parent $P$、当前协议时刻 $\tau$，以及配置、ancestor bodies、状态、L1 数据和 Builder 的 Payload 接口。

**输出。** Physical block $B$，或 WAIT／REJECT。

**处理。** 使用 arch §4.3.4 的构造规则，目标 height 为 $\mathrm{slot}(\tau)$。窗口、parent 和停止检查通过后，计算中间 padding、执行前 root、固定 origin 和普通交易列表。发送前再次检查停止守卫；时间到期后不封顶到 $h_{\max,s}$ 补提议。

#### Accept(P, B, τ_recv)

**输入。** Parent、完整 payload 和本 replica 首次收到该 proposal 的时刻 $\tau_{\rm recv}$，以及检查所需的配置和来源数据。

**输出。** ACCEPT、WAIT(missing) 或 REJECT(reason)。

**处理。** 执行 arch §4.3.5 的 metadata、时间、root 和 BasicTx 检查。等待数据后的重试使用原首次接收时刻；检查通过后，实际投票还须满足 BFT 内部规则和签名时的停止守卫。

验证一个已经存在的最终证明，不重新用观察者当前时间调用 Accept。历史决定验证与新 proposal 的投票检查是两个入口。

#### NotBefore(P)

**输入。** 本实例的合法 parent，以及固定窗口。

**输出。** Arch §4.3.6 定义的最早提议时刻；parent 已达到高度上界时为 $+\infty$。

**处理。** 只计算等待边界，不推进 view，也不产生 payload。没有可提议的后继时，实例仍可在 stop 前处理已有 proposal 的合法共识消息。

### 1.5 输出与公开验证

#### 输出事件 Final(B, π_fin)

**输入／触发。** 当前 context 对 physical block $B$ 形成决定及最终证明。

**输出。** Final(B, $\pi_{\rm fin}$)，并提供消费者取得相应 physical ancestry 所需的已存 blocks。

**处理。** Manager 使用这些材料和 anchor 依据组装 arch 定义的完整 Res。消费者对决定链执行 Virt，得到对应 virtual 片段；原 BFT 接口不逐个产生 padding 的最终证书。

#### 输出事件 Evidence(ev)

**输入／触发。** 实例取得该 BFT 版本可验证的违规证据。

**输出。** Evidence(ev)，连同 cid 交给 Manager。

**处理。** 保存并交付证据，不等待 L1 处罚。Program 关闭不会删除已有证据。

#### CheckFINAL(ν, B, π_fin)

**输入。** Descriptor、目标 block、最终证明，以及用于重建固定配置的公开协议配置。

**输出。** true 或 false。

**处理。** 核对 cid、历史公钥、权重、验证版本，以及证明与 block 的绑定。验证算法是所选 BFT 实现提供的公开算法。

#### CheckEvidence(ν, ev)

**输入。** Descriptor、公开配置，以及包含被指控身份和签名材料的完整证据。

**输出。** true 或 false。

**处理。** 按固定 verifier 版本的 conflict relation 判断违规。每个 BFT 实现必须明确给出这个关系和证据编码；只给出一个未定义的“有效证据”谓词不满足该接口。

## 2. Required Properties

### 2.1 性质的范围与观察事件

以下性质固定同一个有效 descriptor，并采用 arch 的静态故障集合及 $3F_s<W_s$ 条件。其安全部分不要求网络已经同步；涉及真实时间的结论使用 arch 的协议时钟假设。

**Decided chain。** 若 CheckFINAL 对 $B$ 返回 true，称 $B$ 及其同链 physical ancestors 已决定，也称 regional finalized。从固定 anchor 到 $B$ 的链称为 decided chain。证明可以稍后才被其他读取者取得。

**进入 BFT chain。** 为覆盖“已经被 honest replica 用作 parent、但还没有最终证明”的块，对每个非 anchor physical block $B$ 定义：

- $\tau_{\rm par}(B)$：首次有 honest replica 选择 $B$ 或其 physical 后代作为 parent 的时刻，包括据此调用 Build 或 Accept；
- $\tau_{\rm fin}(B)$：首次有任一参与方持有对 $B$ 或其 physical 后代有效最终证明的时刻；
- 未发生的事件时间为 $+\infty$，并令 $\tau_{\rm in}(B):=\min\{\tau_{\rm par}(B),\tau_{\rm fin}(B)\}$。

$\tau_{\rm in}(B)<+\infty$ 表示该 block 已进入 BFT chain；以这样的 block 为末端的 physical chain 称为一条 BFT chain。Anchor 是已验证的外部输入，不作为本实例新产生的 block 处理。

Proposal 所属 view 由其签名共识消息标识，不在 physical payload 中另加一个 view 字段。下文“view 贡献一个 block”指该 view 的 proposal 成为所讨论链上的 physical block。

### 2.2 BFT-1 — Agreement

同一 cid 的任意两条 decided chain 都前缀可比。即对任意两条这样的链 $C,C'$，

$$
C\preceq C'\quad\text{或}\quad C'\preceq C.
$$

这项要求在异步期间、replicas 进入时间不同、部分 replicas 暂时离线或暂停时仍适用。它不比较不同候选 cid 的决定链。

### 2.3 BFT-2 — Chain validity

对每个进入 BFT chain 的 physical block $B$，必须存在一个 honest replica，在实际首次收到该 proposal 时记录接收时刻，并对它与其 parent 调用 Accept；该调用在 $\tau_{\rm in}(B)$ 之前或当时返回 ACCEPT。

若该 replica 先因缺数据得到 WAIT，完成检查时仍使用原首次接收时刻。仅有格式正确的签名，或尚未完成的检查，不能替代这次 ACCEPT。

因此，本性质覆盖最终决定之前和 honest replica 开始依赖该 block 作为 parent 之前的应用检查。

### 2.4 BFT-3 — One block per view

同一 context 的任意一条 BFT chain 上，同一 view 至多贡献一个 physical block。

这是对链上结果的限制，不禁止 Byzantine leader 发送多个相互冲突的 proposals。相邻 blocks 对应不同 view；是否由不同 replica 提议，取决于固定的 $\mathrm{Leader}_s$。

### 2.5 BFT-4 — External verifiability

实现必须提供公开、确定的 CheckFINAL 与 CheckEvidence，并满足：

1. 每个 Final 输出的证明都通过 CheckFINAL，且绑定该 descriptor 与目标 block。
2. 通过 CheckFINAL 的结果适用 BFT-1、BFT-2 等决定性质；敌手不能通过另造证明绕过这些要求。
3. 按协议运行的 honest replica 不会产生针对其身份、通过 CheckEvidence 的违规证据。
4. 另一个 cid 下的签名不能被转用为本 cid 的有效决定或违规证据。

验证只使用声明的公开输入，不要求外部观察者具有同步时钟，也不要求它在签名形成时在线。这里不要求每次违规都必然被发现；证据完备性与证据交付不是“无错误指控”的同义词。

### 2.6 有界进展的适用条件

有界进展使用一个区间 $I=[\tau_{\rm lc},\tau_{\rm end})$，以及固定的 honest replica 集合 $U\subseteq V_s\setminus\mathrm{Byz}_s$，满足

$$
\tau_{\rm lc}\ge\mathrm{GST}_{i_s},\qquad
\tau_{\rm end}\le\tau_s^{\rm stop},\qquad
\sum_{m\in U}w_s(m)\ge Q_s.
$$

$U$ 中的 replicas 称为这个区间的 active replicas。它们不晚于 $\tau_{\rm lc}$ 完成 Enter 或从原状态恢复，整个区间持续在线并调度同一个 cid，未 Close；所需 L1 输入、ancestor bodies、执行状态与 Payload 在声明的处理预算内可取得。委员会和门槛不按当前在线人数重算。

$\tau_{\rm lc}$ 从足够 replicas 已就绪时选取，不从第一个 replica Enter 时起算。切换候选导致某些 replicas 不再调度这个 cid 时，需要重新检查是否仍存在满足条件的区间。

**时间参数。** 在声明的消息大小、状态准备和处理负载范围内，$T_{\rm build}$ 是从开始 Build 到发出 proposal 的上界。所选 BFT 实现给出 $T_{\rm sync}$、$T_{\rm fin}$、$T_{\rm next}$、$T_{\rm vc}$：分别为恢复 view 同步、决定、推进／同步 view、离开不成功 view 的预算。$T_{\rm fin}$ 与 $T_{\rm next}$ 包括相应的验证及数据取得工作。

Config 的 metadata window 使用

$$
T_{\rm win}\ge T_{\rm build}+\Delta_{i_s}.
$$

这些常数的适用输入范围由实现明确声明；存在完整对象但取回或验证时间无界，不满足上述预算前提。

**View 起算时刻。** 对 replica $m$ 和 view $v$，令 $\tau_{\rm view}(m,v)$ 为进入该 view 的时刻，$P_{m,v}$ 为它进入时按 BFT 规则选定的 parent，定义

$$
\tau_{\rm st}(m,v)
:=\max\{\tau_{\rm view}(m,v),\mathsf{NotBefore}(P_{m,v})\}.
$$

它表示 replica 已进入 view，且应用层已经允许从该 parent 构造后继的时刻。Parent 已到高度上界时，这个起算时刻可以是 $+\infty$。

**同步起点。** 通常取

$$
\tau_{\rm sync}:=
\max\{\tau_{\rm lc}+T_{\rm sync},\tau_s^{\rm start}\}.
$$

只有当 $\tau_{\rm lc}=\tau_s^{\rm start}$、全部 active replicas 已在该时刻前初次 Enter、此前无人到达任何 view 的起算时刻，且所需同步条件已经成立时，才称为同步开始，并取 $\tau_{\rm sync}=\tau_s^{\rm start}$。候选切换或暂停恢复不自动满足这个例外。

View $v$ 在 $\tau_{\rm sync}$ 后开始，指 $\tau_{\rm sync}$ 之前没有 active replica 到达该 view 的起算时刻，也没有 active replica 进入更高 view。

### 2.7 BFT-5 — Progress

设 view $v$ 在 $\tau_{\rm sync}$ 后开始，其 leader $m=\mathrm{Leader}_s(v)$ 属于 $U$，且进入 view 时选定的 parent 满足 $P_{m,v}.h<h_{\max,s}$。令 $\tau_0:=\tau_{\rm st}(m,v)$。

Leader 在 $\tau_0$ 开始 Build 并提议。以下每一项都以相应的完成时限严格早于 $\tau_{\rm end}$ 为前提：

1. 该 view 的 block 不晚于 $\tau_0+T_{\rm build}+T_{\rm fin}$ 被决定。
2. Active replicas 不晚于 $\tau_0+T_{\rm build}+T_{\rm next}$ 进入下一 view 或更高 view。进入紧接的 view $v+1$ 时，以这个 block 为 parent；已经推进到更高 view 时，所选 parent 延续这个 block 所在的 physical 前缀。

这里要求决定的是这个成功 view 提议的 block，而不只是任意某个同 height 的 block。必要输入、参与条件或剩余运行时间不足时，不使用本条的完成时限。

### 2.8 BFT-6 — View synchronization and bounded unsuccessful view

在 §2.6 的区间内，要求：

**View 同步。** 若一个 active replica 在 $\tau\ge\tau_{\rm sync}$ 已进入 view $v$，且 $\tau+T_{\rm next}<\tau_{\rm end}$，则所有 active replicas 在该时限内到达 $v$ 或更高 view。

**离开不成功 view。** 对不能按 BFT-5 推进的 view，例如 leader 不属于 active 集合，active replica $m$ 从 $\max\{\tau_{\rm st}(m,v),\tau_{\rm sync}\}$ 起，在 $T_{\rm vc}$ 内离开该 view；这一要求仅在起算时刻有限、且完整时限严格早于 $\tau_{\rm end}$ 时使用。

等待 NotBefore 的时间不算作已经开始的不成功 view 时间；更早发生 Close 或 scheduled shutdown 时，实例执行停止规则。

### 2.9 BFT-7 — Bounded height and scheduled shutdown

**高度有界。** 对任何通过 CheckFINAL 的 block，

$$
h_{\min,s}\le B.h\le h_{\max,s}.
$$

**按时停止。** 所有 honest replicas 在

$$
\tau\ge\tau_s^{\rm stop}=\mathit{ts}^*(h_{\max,s}+1)
$$

时不再生成该 cid 的任何新共识签名；已经较早 Close 的实例也不恢复签名。离线或暂停 replica 恢复时，先检查这一停止条件。

已有 blocks、签名和证据仍可读取。停止前的签名可能在停止后才被组合或公开为有效证书，因此本条不将“停止新签名”等同于“停止后不再出现新的有效证明”。

本条不要求 $h_{\max,s}$ 一定被提议或决定。窗口结束时，最后决定的 physical height 可以更低，也可以没有任何新决定；对外固定最后接纳 block 的规则由 arch 的 GFR 给出。

## 3. 安全目标与有条件进展

本节参照 [v0.11 §5](regional_bft_abstraction_zh_v0_11.md) 的 G1／G2／G3 分类，保留适用于当前固定窗口语义的目标。以下目标以 arch 的输入与模型、Config 规则和 BFT-1 至 BFT-7 成立为前提；“依赖”说明相应条件，不替代对具体 BFT 实现的证明。

### 3.1 G1 — Height 与时间

**G1.1 Fixed timestamp。** 决定链 $C$ 的 Virt 展开在每个 height 使用 $\mathit{ts}^*(h)$。依据 arch 的 VP1 与 Merge 定义；不要求每个 slot 都产生 physical block。

**G1.2 Not ahead。** 每个进入 BFT chain 的新 physical block 满足

$$
\mathit{ts}^*(B.h)\le\tau_{\rm in}(B).
$$

依赖 BFT-2、Config 的首次接收时间检查和协议时钟。

**G1.3 Timely admission。** 对这样的 block，存在完成检查的 honest replica，其真实首次接收时刻满足

$$
\mathit{ts}^*(B.h)\le\tau_{\rm recv}
<\min\{\mathit{ts}^*(B.h+1)+T_{\rm win},\tau_s^{\rm stop}\}.
$$

依赖 BFT-2 与 Accept。这约束首次接收及应用准入，不约束证书后来向外部公开的时间。

### 3.2 G2 — Safety

**G2.1 Agreement。** 同一 cid 的决定 physical 前缀可比较；使用相同的 finalized L1 输入执行 Virt 后，所得 virtual 前缀也可比较。依赖 BFT-1 和 arch 的确定性 Merge。

**G2.2 Prefix closure。** 后代的最终证明确定其同链 physical ancestors；这些 blocks 之间的 virtual heights 由 Merge 唯一确定。证书不需要逐个为 padding 提供单独投票。依赖最终证明的 ancestry 语义与 Merge。

**G2.3 Application validity。** 进入 BFT chain 的 block 满足 Config 的稳定 metadata、root 和 BasicTx 检查。其 root 是执行该 height 之前的状态 root；普通交易仍按 Apply 过滤。依赖 BFT-2 和 arch 的 VP3。

**G2.4 Deposit completeness in the covered prefix。** 设决定链末端为 $h_C$。对

$$
A.\mathit{os}<\ell\le\mathit{os}_A^*(h_C)
$$

的每个 L1 height $\ell$，其 deposit 列表在该 virtual 片段中首次推进到 origin $\ell$ 的 height 执行一次。Leader 不能省略、重排或重复这份强制前缀。依赖连续的 origin 轨迹、VP2 和 Merge；有限前缀不声称已经包含尚未到达的未来 origins。

**G2.5 One block per view。** 每条 BFT chain 上，一个 view 至多贡献一个 physical block。依赖 BFT-3；不同 view 的 leader 可以是同一个 replica。

**G2.6 Close and context isolation。** Honest replica 在 Close 或 stop 后不生成该实例的新共识签名；不同 cid 的签名与证据不能相互转用。依赖控制接口、BFT-4 和 BFT-7。历史证书的迟到公开不构成违反停止规则。

**G2.7 No false evidence。** Honest replica 按本 context 的协议运行，不会被该版本的 verifier 判定为产生有效违规证据。依赖 BFT-4 与签名认证假设。

**G2.8 Execution compatibility。** 在 arch §3.4 的参数与 anchor 前提下，区域 origin 满足 O1–O3；决定链按 Apply 得到与强制前缀和普通交易一致的执行结果。要使同一内容在 L1 原样派生，还需要相同的原生父状态、全局选择及原生期限内的批次交付条件，见 arch §5.7。区域证书本身不完成这些系统步骤。

**G2.9 Unique virtual expansion。** 给定同一个有效决定 physical chain、anchor 状态与 finalized L1 输入，Virt 给出唯一的 virtual 片段，满足 VP1–VP3。扩展只到已给定 physical tip；没有后继 block 时不会因时钟推进而增加尾部。依赖 arch 的 Merge 定义及 BFT-2 对输入链的有效性约束。

G2 不依赖区域网络已经达到 GST。时间相关的停止行为仍使用协议时钟；L1 数据尚未取得时可以等待，而不改变已确定的内容。

### 3.3 G3 — 停止之前的条件性进展

**G3.1 Successful-view production。** 在 §2.6 的区间中，满足 BFT-5 前提且有足够完成时间的 view，其 block 在规定时限内决定，height 为 $\mathrm{slot}(\tau_0)$。依赖 Build 与 BFT-5。

**G3.2 Catch-up by padding。** 若该成功 view 的 parent 较低，决定后由 Merge 补齐到新 physical height 的中间 heights。这个结果依赖实际决定的后继，不因一段时间没有消息就自行产生。若决定时刻为 $\tau_f$，则该成功 block 满足

$$
\tau_f-\mathit{ts}^*(B.h)
<b_{\rm L2}+T_{\rm build}+T_{\rm fin}.
$$

这是单个成功 view 的 freshness 界，不是对整个 session 任意时刻的连续 lag 承诺。

**G3.3 Unsuccessful-view rotation。** 对起算时刻有限、完整时限落在活性区间内的不成功 view，active replicas 在 BFT-6 的时限内离开。要进一步给出等待下一个 active leader 的统一上界，还需对 $\mathrm{Leader}_s$ 的选择序列作相应约束；不能仅由足够在线权重推出。

**G3.4 Selected-transaction execution。** 若某笔普通交易已被 Payload 放入 BFT-5 所决定的 block，且在 Apply 轮到它时通过 Valid，则它保留在该 block 的 Kept 列表中。任何“待处理交易最终一定被选入”的目标还需要 Builder 的交易选择条件，本 BFT 抽象不额外假设公平选取。

旧版“末高度一定决定”的目标不适用于 BFT-7 的当前语义。Regular handoff 的 anchor 由 GFR 固定，后继的低延迟进入还取决于跨区域通信、状态准备和最终选定 cid 的参与情况；这些条件采用 arch §6，不作为单个区域实例的无条件保证。

