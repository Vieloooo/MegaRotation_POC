# Sequencer Rotation 的组合式抽象：外部功能、模块接口与恢复

日期：2026-10-25。状态：接口设计稿；本次修订统一输入/输出格式，简化链与 TEE 抽象。

本文以 `formalization_v3` 中 Part II v4.9、Part III v6 为轮换与 authority 的当前工作基线，以子仓库 `megaeth_specs` 的本地版本 `a738f9b14c7880953d8cff8788db14e8e3e79124` 为 **mega-espresso** 基线。
Part IV v0.1 只用于理解 publisher 的实现职责；其中旧的 authority 接口不覆盖 v4.9/v6。
mega-espresso 的 block-certification 文档仍标为未激活的草案，本文的“继承”指继承这些规格的语义，不代表线上部署现状。

本稿保留本次讨论的三个设计决定：

1. **保持强全局最终性。** 一旦 $B\in G$，后续轮换、停机和恢复均不能删除或替换 $B$；对应执行状态必须由同一历史确定。
2. **Session program 是 Session Manager 管理的子协议实例。** 约束重点是生命周期与实例状态隔离，不限制它只能向两个接口发消息。
3. **合并 staking 与 multisig。** 它们共同构成部署在 L1 上的 Authority 模块，提供权重、追责、区域配置和模式切换。

本稿只讨论 Espresso 的 **liveness failure**；partial safety 不在本轮范围内。
停机后允许 multisig 显式切换至 L1 deposit-only 模式，再从实际派生链恢复普通出块。
“怎样保护已经进入 $G$、但尚未结算的积压块”保留为明确的待选实现契约，见 §6.4。
本次修订将两条来源链各收敛为 Submit/Read/Verify，将 TEE 视为可信第三方，并逐接口列出输入、输出和功能。

## 1. 抽象的组织方式

把系统分成三类对象，避免每个动作都成为一个新“层”。

| 类别 | 对象 | 回答的问题 |
| --- | --- | --- |
| 共享外部功能 | Ethereum L1、Espresso、通信与时钟、DA、可信第三方与状态证明设施 | 我们可以读取和验证什么？哪些服务在什么条件下按时完成？ |
| 协议模块 | Builder、L1 Authority、Session Manager、Session Program、Publisher、Batcher、Certifier | 谁保管状态、接受请求、发送消息并履行服务义务？ |
| 确定性规则 | Global Finality Rule、Derivation Rule，以及 `Plan`、`Leader` 等查询 | 给定相同的已认证输入，任何观察者应得到什么相同结果？ |

Global Finality Rule 不运行额外共识；Manager、certifier 和外部观察者各自执行同一规则。
Derivation Rule 同样不是一个中心化服务，由外部节点读取 L1 与 DA 独立执行。
L1 Authority 和 L1 inbox 都是 L1 上的应用状态机，不能仅凭“运行在 L1”就假定应用逻辑正确。

本文借鉴 UC 的“先声明功能，再证明实现满足功能”的组织方式。
L1、Espresso 等是所有 session、其他链和外部观察者共享的功能，不能为每个 session 分配一份彼此独立的账本。
共享设置的组合问题可参见 [UC](https://eprint.iacr.org/2000/067) 与 [GUC](https://eprint.iacr.org/2006/432)；本文给出接口与组合义务，不声称已经构造模拟器或证明 UC/GUC realization。

```mermaid
flowchart TB
    USR[用户] --> BB[Block Builder]
    STK[Stakers 与 multisig] --> AUTH["L1 Authority<br/>staking、追责、控制日志"]
    AUTH --> SM[Session Manager]
    CRT -->|可信关闭确认| AUTH
    BB --> SP["Session Program<br/>每个 replica、每个 session 一个实例"]
    SM -->|创建、启动、终止、提供上下文| SP
    SP -->|结果、证据、完成事件| SM
    SP <-->|同 session 共识消息| PEER[其他 replicas 的 Session Programs]
    SP -->|完整区域结果| PUB[Publisher]
    SM <-->|handoff、数据与状态准备| MSM[其他区域的 Managers]
    PUB --> ESP[Espresso]
    ESP --> GF["Global Finality Rule<br/>NORMAL 模式的公开输入"]
    AUTH --> GF
    GF -->|G 与生命周期事件| SM
    GF --> BAT[Batcher]
    BAT -->|frames| CRT["可信 Certifier（TEE）"]
    ESP --> CRT
    AUTH --> CRT
    CRT <--> DA[DA]
    BAT -->|分散 payload| DA
    CRT -->|attested commitment| BAT
    BAT --> INB[L1 Batch Inbox]
    INB --> DR[Derivation Rule]
    DA --> DR
    AUTH -->|模式切换| DR
    L1["finalized L1<br/>deposits、系统输入、配置"] --> DR
    DR -->|DEPOSIT_ONLY 模式的确定性块| GF
    DR --> OBS[外部观察者的 L2 链与状态]
    BAT -.->|state claim，逻辑上独立| FP[原有状态证明与 L1 结算接口]
```

图中 `Derivation → Global Finality` 只表示 deposit-only 分支。
它根据已经固定的模式边界和 finalized L1 派生新区块，不读取“尚未生成的 $G$”；NORMAL 分支也不等待 batcher 才形成区域回执。
通信骨干、时钟和验证证明的传递省略在图外，统一由 §3 的接口承载。

## 2. 记号与共同约定

### 2.1 尽量沿用 v3 的记号

| 记号 | 本文含义 |
| --- | --- |
| $(\beta,r,g,t)$；$j$ | 分支版本、session 轮次、恢复代、leader term；term 内 fresh position。$t$ 不作为墙钟时间。 |
| $s=(\beta,r)$ | 一个 session；其每个 replica 运行独立的本地实例。 |
| $i_{\beta,r}$，$V_{\beta,r}$ | session 的区域与固定委员会。 |
| $w_o,W_{\beta,r},F_{\beta,r}$ | 固定投票权重、总权重、拜占庭权重上界；$F<W/2$，证书签名权重须 $>F$。 |
| $\mathcal C_e$；$a_e,c_e$ | staking epoch 快照及其激活、截止 L1 高度。 |
| $\mathcal P_\beta=(S_\beta,\mathcal L_\beta,\lambda_\beta)$ | 分支内不变的 ON/OFF 表、循环区域顺序、L1 floor。 |
| $\Pi_s=\mathsf{AUTH}_{s,0}$；$\mathsf{AUTH}_{s,g}$ | session plan；恢复代 authority。 |
| $E_s$；$A_s=A_{\beta,r,0,0}$ | 选区/选主参考块；实际执行起点。regular successor 分别使用前驱的初始 anchor 与 terminal DATA。 |
| $\mathrm{cid}_s$ | $\mathsf H(\mathrm{id}(\Pi_s),\mathsf H(A_s))$；用于签名和恢复记录的上下文绑定。 |
| $\mathrm{TB},K,H_s,T_s$ | 每 session 的 DATA 配额、每 term 的上限、session terminal 高度与 terminal DATA。 |
| $h(B)$；$\mathit{st}(B)$ | 块高度；精确执行至该块后的状态。 |
| $\rho,\lambda(B)$；$\xi(B)$ | 区块绑定的 finalized L1 引用及高度；finalized Espresso header 引用。 |
| $\mathsf{FC}_s(B)$ | 区域 FINAL 回执，或同 session 内的 covering evidence；不是 proposal QC。 |
| $J_q,\theta_q$ | Espresso finalized block 及其认证时间戳。 |
| $U,c,\Lambda,\kappa$ | 控制更新、序号、完整控制日志、处理进度及其承诺。 |
| $G$；$R$ | 不可回退的全局链；可能领先于 $G$ 的区域生产分支。 |
| $\nu,\sigma_G(B),\mathsf{Tail}$ | session descriptor、块绑定的来源记录、下一块的生产责任。 |
| $\Delta_i,T_{\rm pub},T_E,T_{\rm L1},T_{\rm ready}$ | 区域通信、发布、Espresso、L1 与准备服务的时间界。 |
| $\epsilon_E,T_{\rm update}$ | Espresso 时间戳相对真实时间的最大提前量；普通更新的最小激活间隔。 |

新增记号只用于当前接口确实缺少的概念：

| 新记号 | 用途 |
| --- | --- |
| $\tau$ | 真实时间，避免与 term $t$ 混淆。 |
| $\Delta_{\rm cross}$ | mixed p2p 骨干的跨区域完整消息交付界。 |
| $\mu\in\{\mathsf{NORMAL},\mathsf{DEPOSIT\_ONLY}\}$ | 全局生产模式。 |
| $G^\star,A^\star$ | 模式切换保护的完整前缀，以及 $A^\star=\operatorname{last}(G^\star)$。 |
| $\ell_a$ | 模式切换的 L1 激活高度；与普通 Force 的 Espresso 阈值 $\theta_a$ 分开。 |
| $B_n^{\rm dep}$ | L1 规则派生的 deposit-only 块，绝不是 v3 的 DRAIN。 |

Part II 中 $E$ 还曾表示 entry proof；本文只用 $E_s$ 表示 Part III 的 selection reference，进入凭证写作 `EntryEvidence`。
原生记录里的 `epoch_num` 是 **L1 origin 高度**，不是 staking epoch $e$，也不自动等于 authority 引用 $\lambda(B)$。

### 2.2 通用调用语义

每个接口明确调用者、输入、输出和失败行为。
查询返回值或 `WAIT`；可判定的坏输入返回 `REJECT`；只有已取得完整证据且确认不存在合格对象，才返回 $\bot$ 或空集合。
`WAIT` 不等于 OFF、零权重、空 namespace、证据无效或已经完成。

提交接口先返回请求标识，最终返回带证明的执行结果。
“已提交”指完整请求已被服务接收并满足准入条件，不是本地发起了一次网络发送。
状态相关的竞争交易仍可执行失败；L1 的按时处理承诺不保证每笔调用都成功改变应用状态。
下文每个接口均采用两个列表：输入列表与输出列表，随后一段功能说明。
L1 应用接口默认通过 `Submit_L1` 调用；`caller`、请求标识 `rid`、chain 与 rule version 是公共调用字段，不在每个接口重复列出。
其输出表示 finalized 执行结果；`REJECT` 不改变应用状态，`WAIT` 表示依赖暂缺而不提交新的状态转移。
所有实际执行仍由 L1 原子排序；重复请求返回已记录结果，不能重复扣款或新建同一恢复记录。
L1 应用记录 `ReqResults[(caller,rid)]=(requestHash,result)`；相同标识配不同请求内容时拒绝，纯查询不使用此去重状态。

所有认证对象绑定 chain/deployment、协议版本、消息类型和相关 session context；结构化对象使用固定编码，快照 rows 按 position ID 排序。
请求具有稳定标识，重试不能重复扣款、重新 Enter 或追加同一块。
角色可以由同一机器承担，但两个角色之间的验证责任不因此消失。
本模型不提供消息保密性；攻击者可观察公开数据、发送垃圾输入并控制拜占庭参与者。
诚实节点间的交付顺序可以由网络调度决定，但在声明的服务范围内受时间界约束。
区域故障界约束固定委员会在其证书仍可被接受期间累计被攻陷的权重，不能由“已质押”推出诚实多数。
沿用 v3 对 multisig 的诚实协调服务假设；其响应速度另行声明。

## 3. 外部共享功能

### 3.1 Ethereum L1 与 Espresso：只保留 Submit、Read、Verify

分别记为 $\mathcal F_{\rm L1}$ 与 $\mathcal F_E$，以下用 $X\in\{\mathrm{L1},E\}$ 表示来源。
**本稿不要求 L1 与 Espresso 验证彼此的数据，也不建模 light client。**
`Verify` 是提供给数据消费者的抽象能力：节点、外部观察者和可信 certifier 可以读取并核验两条链；L1 合约需要跨链结论时接收可信 certifier 的消息。
Espresso 只排序其收到的数据，不验证本协议的 L1 authority 或区域回执。

$\mathcal F_X$ 维护唯一的 finalized 历史 $\mathcal H_X$；L1 历史包含交易执行结果、事件和应用状态，Espresso 历史包含有序 blocks 及完整 namespace 内容。
未 finalized 的数据不通过这些接口输出。
证明 $\pi_X$ 是不展开内部结构的验证材料；不规定其密码学格式、quorum 更新或跨链验证实现。

正常运行时，finalized histories 在正确消费者之间前缀相容；它们可以因传播而处在不同进度。
Espresso 的 $\theta_q$ 非递减，且首次最终化时间 $\tau_E(q)$ 满足 $\theta_q\le\tau_E(q)+\epsilon_E$。
普通 Force 的进展另需 header/time progress；Espresso 停机撤销新记录的最终化时限，旧历史的持久性仍成立，历史字节的可取得性另按保留条件承担。

#### 3.1.1 `Submit_X(req)`

**输入**

- $X$：目标为 L1 或 Espresso。
- $req=(rid,u,target,data,value)$：请求标识、提交者、目标、精确数据字节和随请求支付的费用/资金。
- 对 L1，$target$ 是合约地址，$data=(method,args)$；对 Espresso，$target$ 是本链 namespace，$data$ 是 publication 字节。

**输出**

- `Accepted(rid)`：完整请求已满足准入条件并进入处理；不满足时返回 `REJECT(reason)`。
- 后续 `Finalized(rid,pos,result)`：确定的来源位置与执行结果；L1 的 `result` 包括成功或失败及相应事件，Espresso 的结果标识已记录的数据。

**功能。** 将请求纳入目标来源的处理流程，在正常服务条件下于 $T_{\rm L1}$ 或 $T_E$ 内完成处理、最终化并使正确在线消费者可读取验证。L1 的状态竞争可能使调用执行失败；该时限保证处理结果，不保证每个请求都成功改变应用状态。

#### 3.1.2 `Read_X(query)`

**输入**

- $query=([a,b],scope)$：要读取的来源区间和范围；$b=\mathsf{latest}$ 表示读取服务当前提供的 finalized 截止点。
- L1 的 `scope` 指定合约日志、交易结果或在截止高度的状态键；Espresso 的 `scope` 指定本链 namespace。

**输出**

- $R_X=(X,query^*,D,\pi_X)$：解析为确定截止点的查询、该范围内的完整有序数据及验证材料。
- 已认证的空序列也是合法 $D$；来源尚未最终化或材料暂缺时返回 `WAIT`。

**功能。** 返回指定范围内全部相关的 finalized 数据，单条读取是 $a=b$ 的特例，历史前缀读取使用同一接口。`latest` 只确定本次返回的截止点，不证明未来没有新数据；本地未取到数据与已验证的空范围不同。

#### 3.1.3 `Verify_X(R_X)`

**输入**

- $R_X=(X,query^*,D,\pi_X)$：一个含确定来源、区间、范围和精确字节的读取结果。

**输出**

- `VALID`：$D$ 确为所声明范围内完整、有序、不可回退的 finalized 数据。
- `INVALID`：声明或材料不成立，包括缺少必要验证材料的输入。

**功能。** 使任意消费者对同一个 finalized 事实得到相同判断，并将完整性一并纳入验证。证明只覆盖 `query*` 声明的范围；一份有效的单记录读取结果不能冒充完整控制前缀。该接口是抽象保证，不表示 L1 或 Espresso 必须执行对另一条链的验证。

### 3.2 通信与时钟

采用 v3 的公共时钟，以 $\tau$ 表示真实时间。
网络只保证认证交付，不负责判断消息内容是否符合协议。

#### 3.2.1 `Send(u,v,domain,m)`

**输入**

- $u,v$：发送者与接收者身份及其所属区域。
- $domain=(chain,version,s,\mathrm{cid})$：消息所属协议与 session；非 session 消息使用相应模块的 domain。
- $m$：完整消息字节及本次处理所需的验证依赖。

**输出**

- 接收者事件 `Deliver(u,v,domain,m)`，保留发送者身份和精确内容。
- 对正确、在线端点，区域内交付与约定验证处理不晚于发送后 $\Delta_i$，跨区域不晚于 $\Delta_{\rm cross}$。

**功能。** 将同一套认证通信语义用于区域内快速网络和跨区域 mixed p2p 骨干。时间界以规定大小、负载和正确传递路径为前提；允许乱序和重复，拜占庭发送者可以沉默。任意规模的状态重建不由网络时限兜底，仍使用 $T_{\rm ready}$ 的独立预算。

### 3.3 DA

$\mathcal F_{\rm DA}$ 维护 commitment 到已分散 payload 的绑定，并在声明的保留条件下提供取回。
本稿省略 DA 内部 operator、registry 和证书算法；这些由 DA 功能承担。

#### 3.3.1 `Disperse(payload)`

**输入**

- `payload`：一个批次提交的精确字节，包括 derivation version 与同一 channel 的连续 frames。
- `rid` 与服务费用：用于识别这次请求并满足 DA 准入条件。

**输出**

- $(d,\pi_{\rm DA})$：payload 的数据承诺及 DA 的确认材料。
- `WAIT` 或 `REJECT(reason)`：尚未完成或不满足准入条件。

**功能。** 将完整 payload 交给 DA 保管并固定其内容绑定；只有完成分散才产生成功结果。DA 不决定这些输入的顺序是否合法，也不使它们自动进入 $G$。

#### 3.3.2 `ReadDA(d)`

**输入**

- $d$：已经存在的 DA commitment。

**输出**

- $(payload,\pi_{
m DA})$：与 $d$ 对应的完整字节及确认材料；暂不可取得时返回 `WAIT`。

**功能。** 为 certifier、derivation 与恢复节点取得批次数据。取回保证依赖声明的保留服务；已被 L1 接受的数据取不回来时，derivation 在该处等待，不以 deposit-only 代替。

#### 3.3.3 `VerifyDA(d,payload,π_DA)`

**输入**

- $d$、`payload`、$\pi_{\rm DA}$：数据承诺、拟使用的精确字节及 DA 确认材料。

**输出**

- `VALID` 或 `INVALID`：确认材料是否成立，以及字节是否与该承诺一致。

**功能。** 保证实际读取和执行的是 DA 所确认的那份数据。长期可用性仍是保留条件，不由一次成功验证推得永久可取回；当前方案不增加其他 DA 网络的自动 fallback。

### 3.4 TEE 只作为可信第三方

记 $\mathcal T$ 为可信第三方，并假设它正确执行本稿规定的 certifier 业务规则。
**凡是由 $\mathcal T$ 发出的消息，消费者直接信任；消息来源归属和不可被冒充由模型隐式提供。**
不定义 TEE 的 `Run`、远程认证、measurement、密钥注册、签名或验签接口，也不把 host/core 划分带入协议抽象。
TEE 消息记作 $\chi=\langle\mathcal T,kind,body\rangle$；其业务字段在 §4.7 的输出中定义。

§4.7 描述的是“批次核对、控制生效确认”这两个业务接口，不是 TEE 硬件接口。
L1 Authority/inbox 接收相应 $\chi$ 时直接使用其结论，因此不需要在 L1 内验证 Espresso 历史。
可信性不增加业务范围：批次对应性消息只声明输入对应关系，不声明一个尚未经状态证明系统处理的 state root 已完成结算。

### 3.5 原生执行与状态结算边界

原生执行是确定性规则；Kailua/fault-proof 负责提交的 state root 与正确派生执行的一致性。
本稿沿用其职责，不为子仓库仍是 stub 的 fault-proof 页面虚构具体争议算法；提交接口在 §4.6.4 单独列出。

#### 3.5.1 `Execute(parentState,ordinaryTxs,nativeContext)`

**输入**

- `parentState`：精确 parent 块之后、已验证的状态 $\mathit{st}(parent)$。
- `ordinaryTxs`：按确定顺序排列的普通交易完整字节列表。
- `nativeContext`：chain/rule version、下一块高度与 timestamp、finalized L1 origin header、应纳入的 deposits、系统交易及生效配置。

**输出**

- $(B_{\rm native},\mathit{st}(B),stateRoot,receipts)$：确定的原生块、执行后状态、state root 和交易结果。
- `INVALID(reason)`：块级执行条件不成立；必要输入缺失时返回 `WAIT`。

**功能。** 按既定 MegaETH 执行/派生规则计算唯一结果；普通交易执行 revert 仍属于合法的执行结果，不自动使整块无效。执行不会选择 global branch，也不会给结果授予最终性；普通模式和 deposit-only 模式都使用同一原生规则与参数。

### 3.6 持久化前提

沿用 v3 的 `PublicStore`，作为节点状态，不新增一个外部接口层。
它保存读取结果的精确字节、区域证据、未完成发送、checkpoint 与精确 anchor 状态。
session 终止不删除这些材料；stake release 前完成保管交接，离线节点同步完成后再确认新输出。

## 4. Sequencer Rotation 各模块的接口

### 4.1 Block Builder：产生有序候选输入

Builder 维护本地待处理交易集合及其稳定交易标识。
一个区域 DATA 在本稿对应一个完整 L2/EVM block，该块可含多个 mini-blocks；DRAIN 不产生新的 L2 块。
原生投影为 $\mathsf{Record}(B)=(block\_number,epoch\_num,transactions)$，其中交易列表只包含普通交易。
这个投影不解除完整块对 parent、origin 和 timestamp 的绑定；deposits 与系统输入使用 §3.5 的 `nativeContext`。

#### 4.1.1 `SubmitTx(tx)`

**输入**

- `tx`：一笔用户签名的完整原生交易字节，含 chain、nonce、目标、value、调用数据、gas 与费用字段。

**输出**

- `Received(txid)`：记录交易的稳定标识；格式或准入不满足时返回 `REJECT(reason)`。

**功能。** 将交易纳入本地候选集合并按交易标识去重，不授予包含、排序优先级或最终性。若要保证某笔用户交易最终包含，需要单独的输入公平性规则。

#### 4.1.2 `Build(s,parent,parentState,nativeContext,limits)`

**输入**

- $s=(\beta,r)$：请求候选块的 session，调用者为其本机 proposer/program。
- `parent` 与 `parentState`：精确 parent 块及 $\mathit{st}(parent)$。
- `nativeContext`：§3.5 的高度、timestamp、finalized L1 origin、deposits 与系统配置。
- `limits`：本块 gas、字节数、交易数及 mini-block 格式的限制。

**输出**

- $x$：候选 payload，包含有序普通交易列表、原生上下文及 mini-block 划分；也可产生普通交易列表为空的合法候选。
- `WAIT`：必要来源或状态未准备好；`REJECT(reason)`：请求的上下文或限制无效。

**功能。** 在当前 parent 和既定限制上选取、排序并组织候选交易，mini-block 的拼接顺序须与完整交易列表一致。Builder 不选择新的 authority 或 parent branch，也不使候选获得共识许可。

#### 4.1.3 `ValidateCandidate(parent,parentState,x,nativeContext)`

**输入**

- `parent`、`parentState`、$x$ 与 `nativeContext`：精确前置块、前置状态、候选 payload 和执行上下文。

**输出**

- `VALID(B_native,st(B),stateRoot)`：格式、交易顺序、mini-block 对应与原生执行检查通过。
- `INVALID(reason)` 或 `WAIT`：候选不合法或依赖暂缺。

**功能。** 通过 §3.5 的执行规则验证候选，并核对 mini-block 与完整块输入绑定；区域协议还须检查 session、view、quota 和签名。此接口输出不等于区域或全局最终性。

### 4.2 L1 Authority：明确状态、context 与状态转移

Authority 是 L1 上的应用状态机，成员管理、追责和 multisig 控制共用该状态。
它不重放 Espresso，也不判断每个 speculative session 是否真的存在；canonical session 的确定由 Manager/global rule 负责。

**固定参数。** Genesis 固定 chain/rule version、区域集合 $\mathcal L_R$、multisig authority $\mathcal K$、质押资产、最低质押 $B_{\min}$、准入上限、每区域入队/出队的 amount/count 预算及成熟等待、$M,N,a_0$、惩罚比例 $\rho_{\rm slash}$、$T_{\rm hold}$，以及 $\mathrm{TB},K,\Delta_i,\varphi_i$。
$\rho_{\rm slash}$ 是 Part III 的 penalty fraction $\rho$，这里加下标以免与区块的 L1 reference $\rho$ 混淆。

**初始化。** Genesis checkpoint 给定初始 positions/escrow、$\mathcal C_0$、$\mathcal P_0$、$A_0$、初始 ban 集与控制前缀。
KeyPosition 从初始 positions 建立，`nextId` 大于所有已分配 ID，`lastSeq` 与初始控制前缀一致；queues、REC、SeenOffenses、ReqResults、Closures 与 Clear 在新系统 genesis 为空。
从后来的授权 checkpoint 启动则恢复其完整状态，不重新清空这些表。

**维护状态。** 下表是 finalized L1 高度 $\ell$ 上的逻辑状态；历史查询读取确定高度的版本。

| 状态 | 精确内容 |
| --- | --- |
| $\mathsf{Pos}[o]$ | $(i_o,pk_o,owner_o,B_o,phase_o,p^{\rm join}_o,p^{\rm exit}_o,e_x(o))$：区域、operator 公钥、提款 owner、剩余 escrow、生命周期、join/exit 请求位置、退役 epoch。未定义的 exit 字段为 $\bot$。 |
| `ReqResults` | §2.2 的调用去重表，保存已执行请求的内容承诺与结果，避免重复 Join/转账。 |
| $\mathsf{KeyPosition}[pk]$、`nextId` | 尚未解除责任的 key 到 position 的映射；永久不复用的 position 编号。 |
| $Q_i^{\rm in},Q_i^{\rm out}$ | 按 L1 `(height,txIndex,logIndex)` 排序的 FIFO position 列表；每项带请求位置和成熟条件。 |
| $\mathcal C_e$、$\ell_{\rm pub}(e)$ | epoch 的冻结权重表、root、cutoff/activation，以及发布高度。每个 $e$ 的内容只确定一次。 |
| $\mathsf{Ban}(\ell)$ | 在该 finalized L1 高度已登记禁任的 operator key 集合，只增不减。 |
| $\mathsf{REC}[(\mathrm{cid},g)]$ | 恢复代创建的 L1 高度 $\ell_{\rm rec}$；该代的禁任集为 $\mathsf{Ban}(\ell_{\rm rec})$，在该高度最终化后固定。 |
| $\mathsf{SeenOffenses}$ | 已处理的 $(o,kind,slot)$；`slot` 含 $\mathrm{cid}$ 和该冲突类型的坐标。 |
| $\Lambda,\mathsf{lastSeq}$ | 完整的有类型控制日志及最大序号；每项保存控制内容、multisig 认证、L1 发布位置。 |
| $\mathsf{Closures}$ | 由可信 certifier 确认已经生效的关闭记录，按控制序号去重；不是仅已公告的控制。 |
| $\mathsf{Clear}$ | 按严格增加的 $e^\star$ 保存 $(e^\star,\tau_c)$；表示相应退役责任已清退，持有期从 $\tau_c$ 起算。 |

$phase_o\in\{\mathsf{Pending},\mathsf{Active},\mathsf{Exiting},\mathsf{Retired},\mathsf{Cleared},\mathsf{Released},\mathsf{Cancelled}\}$。
`Cleared` 可由 $e_x(o)$ 与 Clear 表推导，不要求逐 position 写入；进入冻结快照也不等于立即获得某个 session 的签名权。

epoch 的固定时间与权重表为：

$$
a_e=a_0+eM,\qquad c_e=a_e-N,\qquad
\mathcal C_e=(e,c_e,a_e,\mathsf{Rows}_e,root_e),
$$

$$
\mathsf{Rows}_e=\{(o,i_o,pk_o,w_{e,o}):o\text{ 被纳入 epoch }e\},
\qquad
w_{e,o}=B_o(c_e^-),\quad
W_{e,i}=\sum_{o:i_o=i}w_{e,o}.
$$

未纳入的 position 在该表的权重为零；$c_e^-$ 指 L1 高度严格小于 $c_e$ 的完整历史。
之后的 slash 只减当前 $B_o$，不改任何已经冻结的 $w_{e,o}$。
$root_e$ 绑定快照内容；实际发布高度单独记录，因此迟发布不改变该 epoch 应有的权重表。

**Context 的统一定义。** 全文使用

$$
C=(\Pi,A),\qquad
\mathrm{cid}(C)=\mathsf H(\mathrm{id}(\Pi),\mathsf H(A)).
$$

$\Pi$ 是下文 `Plan` 的完整输出，绑定 chain、session、区域、快照权重和固定参数；$A$ 是该 session 声称的精确初始 anchor，而不是 recovery anchor。
完整 anchor 字节固定其 hash 与高度，因此 $H_s=h(A)+\mathrm{TB}$ 也确定。
所有 BFT 签名额外绑定协议 domain、消息类型和 $\mathrm{cid}(C)$。
**Authority 对 $C$ 作相对验证**：核对 plan 与其 L1 快照/ban 来源一致，按 $C$ 的固定成员、权重、leader 和 slot 检查签名与证据；不由此认定该 anchor/session 已被全局规则授权。
伪造一个从未运行的 context 不能转用诚实成员在另一 $\mathrm{cid}$ 下的签名，也不能占用真实 context 的 REC。

下文 `admission` 表示相对该 $C$ 的 generation entry/quit 证据、所需 REC、之前各 term 的 terminal QCs，以及具体 proposal 的 opening、parent 和 quota 材料。
它不要求 L1 验证 Espresso 历史或重新执行全局 handoff；完整的 canonical handoff 与原生区块验证仍由共识节点承担。

签名消息在接口中写成 $m=(domain,\mathrm{cid},kind,slot,body,pk,signature)$，签名覆盖除签名本身之外的固定编码。
PROPOSE/fresh vote 的 body 绑定 generation opening、block type、physical position、DATA height 与 block hash；NEWVIEW 绑定 opening、justification 与 entry proof；ANCHOR_VOTE 绑定 opening；FINAL 绑定 DATA height/hash。
BLAME 的 slot 为 $(\mathrm{cid},g,t)$，所有该 view 的投诉签署同一声明，不附带本地进度计数。
`admission` 的材料由 `(quitChain,RECrefs,terminalQCs,opening,openingQC,blocks)` 组成：此前 generation 的 quit 证明链、对应 REC、此前各 term 的 terminal QCs、精确 opening、恢复 opening 的 ANCHOR QC 及检查 position/quota 所需的完整块。
Generation zero 相对 $C$ 的固定 $A$ 起步，相关恢复材料为空；仅取当前消息类型需要的字段，FINAL 不需要 generation/term admission。

offense 的类型、冲突槽与判定内容固定如下；`slot` 总是包含 context ID。

| kind | 同一个 slot | 必须冲突的内容 |
| --- | --- | --- |
| PROPOSE | $(\mathrm{cid},g,t,j)$ | designated leader 签署两份不同且满足 admission/quota 的 DATA 或 DRAIN proposal。 |
| FRESH_VOTE | $(\mathrm{cid},g,t,j)$ | 同一成员选择不同 block；DATA/DRAIN 共用一次 fresh-vote 权限。 |
| NEWVIEW | $(\mathrm{cid},g)$ | recovery leader 签署不同 opening，即使两个 opening 可比较。 |
| ANCHOR_VOTE | $(\mathrm{cid},g)$ | 同一成员对不同 recovery opening 投票。 |
| FINAL | $(\mathrm{cid},h)$ | 同一成员对同一 DATA 高度签署不同 block hash；不使用 $g,t$。 |

#### 4.2.1 `Join(i,pk,owner,b,pop,ownerAuth)`

**输入**

- $i\in\mathcal L_R$：加入的区域；$pk$：operator 的 BFT 签名公钥。
- `owner`：最终提款的 owner 地址；$b\ge B_{\min}$：本次实际转入 escrow 的资产数量。
- `pop`：$pk$ 对绑定 `(chain,Authority,rid,i,owner,b)` 的 possession 证明。
- `ownerAuth`：owner 对同一请求的授权；若 owner 即已认证的 L1 caller，可由其调用授权承担。

**输出**

- `Joined(o,i,pk,owner,b)`：新 position ID 与注册内容。
- 状态效果：创建 $\mathsf{Pos}[o]$，令 $B_o=b,phase_o=\mathsf{Pending}$，登记 key 映射并加入 $Q_i^{\rm in}$。
- `REJECT(reason)`：区域、资金、授权、key 唯一性或准入限制不满足。

**功能。** 原子接收资金并登记一个 whole-position 入队请求；同一 key 不能同时对应多个未解除责任的 position，position ID 永不复用。Join 不立即改变正在使用的任何 epoch 权重表或 session 委员会。

#### 4.2.2 `Cancel(o,ownerAuth)`

**输入**

- $o$：尚未被任何冻结快照纳入的 Pending position。
- `ownerAuth`：$owner_o$ 对取消该 position 的授权。

**输出**

- `Cancelled(o,refund)`，其中 `refund` 为剩余 escrow。
- 状态效果：移出 inbound queue、置 $B_o=0$ 和 `Cancelled`、解除该 key 的未结责任映射；不满足条件时 `REJECT(reason)`。

**功能。** 允许尚未承担冻结快照责任的 position 取消加入并一次性退款。一旦已进入任意冻结快照，就必须使用退出与清退流程，不能改用 Cancel。

#### 4.2.3 `RequestExit(o,ownerAuth)`

**输入**

- $o$：已被冻结快照纳入、尚无退役记录且没有未完成 exit request 的 position。
- `ownerAuth`：$owner_o$ 对不可撤销退出请求的授权。

**输出**

- `ExitRequested(o,p_exit)`：L1 中的唯一请求位置。
- 状态效果：登记 $p_o^{\rm exit}$，令 phase 为 `Exiting` 并加入 $Q_{i_o}^{\rm out}$；不满足条件时 `REJECT(reason)`。

**功能。** 排队申请从未来快照中移除整个 position，不即时改变当前/历史权重、不退款，也不授权停止已经分配的 session 职责。

#### 4.2.4 `CutSnapshot(e)`

**输入**

- $e>0$：要冻结的 epoch；调用时 L1 高度至少为 $c_e$。
- 本账本内部来源：$\mathcal C_{e-1}$ 与严格早于 $c_e$ 的 positions、queues、余额和 offense 历史，调用者不能另传一份替代状态。

**输出**

- $\mathcal C_e$ 与发布高度 $\ell_{\rm pub}(e)$；已存在时返回同一快照。
- 状态效果：更新队列；新纳入的 Pending position 成为 Active；尚有未处理退出请求的旧成员保持 Exiting；被移除的旧成员获得 $e_x(o)=e$ 并成为 Retired。
- `WAIT` 或 `REJECT(reason)`：前驱/历史未齐或请求尚不满足冻结条件。

**功能。** 先移除 cutoff 前已发生 offense 或 escrow 低于最低额的旧成员，再分别按成熟条件和 amount/count 预算取 inbound/outbound FIFO 的最长合格前缀，join 按锁定金额、voluntary exit 按前一快照权重计入各自预算，保留其余旧成员，并以 cutoff 时 escrow 冻结权重；自愿退出预算不限制强制退役，也不以区域最低 stake 否决退出。只消费本次 cut 处理的 queue entries，保留 cutoff 之后的新请求；快照按 $a_e$ 激活，迟发布时消费者等待而不重算其内容。

#### 4.2.5 `ReadAuthority(ℓ,query)`

**输入**

- $\ell$：确定的 finalized L1 高度。
- `query`：`Position(o)`、`Snapshot(e)`、`Ban`、`Recovery(cid,g)`、`Controls(c_from,c_to)` 或 `Clearance`。

**输出**

- 对应状态值或有序记录；`Snapshot(e)` 返回完整 $\mathsf{Rows}_e$、root 与发布/激活信息，`Controls` 返回指定区间的完整日志。
- 与该高度绑定的 $R_{\rm L1}$；已认证不存在则返回 $\bot$，未取得该高度数据则返回 `WAIT`。

**功能。** 将 §3.1 的 `Read_L1/Verify_L1` 专用于本 Authority 状态；它不引入另一套链访问机制。调用者可以由同一 epoch 表计算每个区域的总权重，不能将未取得快照解释为零权重。

#### 4.2.6 `Plan(Pβ,β,r,E,i_src)`

**输入**

- $\mathcal P_\beta=(S_\beta,\mathcal L_\beta,\lambda_\beta)$、$\beta,r$：固定 policy 与目标 session。
- $E$：精确 selection-reference 块，包含 $h(E),\mathsf H(E),\lambda(E)$；$i_{\rm src}$ 为 source region，genesis 可为 $\bot$。
- 读取依赖：$\ell=\max\{\lambda(E),\lambda_\beta\}$ 上的 $\mathsf{Ban}(\ell)$ 和 $\mathcal C_{e(\ell)}$，其中 $e(\ell)=\max\{e':a_{e'}\le\ell\}$。

**输出**

- $\Pi$：保存以上固定输入与来源；输出选中的区域 $i$、$V_s$、每个成员的 $w_o$、$W_s,F_s$、$\mathrm{TB},K,\Delta_i$、初始 ban 集、$\lambda(\Pi)$ 与 $\mathrm{id}(\Pi)$。
- 若完整来源表明无合格区域，则 $i=\bot$；来源未齐返回 `WAIT`。

**功能。** 按 $\mathcal L_\beta$ 从 $i_{\rm src}$ 之后循环选择第一个 ON 且未禁任 proposer 正权重总和非零的区域；$i_{\rm src}=\bot$ 时从顺序起点选择。该区域快照内所有正权重成员组成固定委员会，包括被禁任的投票成员，$F_s=\varphi_iW_s$；$\lambda(\Pi)=\max\{\ell,\ell_{\rm pub}(e)\}$，plan ID 绑定全部选择输入及结果。该纯查询不写 L1，也不单独授权一个 session 启动。

#### 4.2.7 `Auth(C,g)`

**输入**

- $C=(\Pi,A)$ 与恢复代编号 $g\ge0$。
- 对 $g>0$，本账本中的 $\mathsf{REC}[(\mathrm{cid}(C),g)]$ 及其记录高度的 ban 集。

**输出**

- $\mathsf{AUTH}_{s,0}=\Pi$；或 $\mathsf{AUTH}_{s,g}=(\Pi,REC,\mathsf{Ban}(\ell_{\rm rec}))$。
- 对缺失或尚未 finalized 的 REC 返回 `WAIT`。

**功能。** 固定该代可提议的成员集合；同一 session 的委员会、投票权重、quota 与阈值不变。未来的新快照或 slash 不重写已取得的 authority。

#### 4.2.8 `Leader(AUTH_sg,t)`

**输入**

- $\mathsf{AUTH}_{s,g}$：已取得的代 authority；$t$：term 编号。

**输出**

- $L(\beta,r,g,t)$：按原始快照权重从该代未禁任成员中确定的 leader；集合为空时为 $\bot$。

**功能。** 沿用 v6 的 `WeightedRand`，seed 为 $\mathsf H(\texttt{LEADER},chain,\beta,r,h(E_s),g,t)$，而非可由 producer 改变的 $\mathsf H(E_s)$。它是确定性查询，不提供不可预测性或 schedule fairness 保证，也不改变投票集合。

#### 4.2.9 `SubmitOffense(C,o,ev)`

**输入**

- $C=(\Pi,A)$：上述完整 context；$o$：在 $\Pi$ 的真实快照中对应被指控公钥 $pk_o$ 的 position。
- $ev=(kind,m_1,m_2,admission)$：同一 key 的两条完整签名消息，均绑定该 $\mathrm{cid}(C)$；`admission` 含该类型所需的代/term 准入、opening 与 quota 材料。
- `kind` 为 `PROPOSE`、`FRESH_VOTE`、`NEWVIEW`、`ANCHOR_VOTE` 或 `FINAL`；冲突槽定义见本节的类型表。

**输出**

- `Slashed(o,kind,slot,amount,B_after)`，其中 $amount=\min\{\rho_{\rm slash}w_o,B_o\}$，$w_o$ 取自 $C$ 固定的快照。
- 状态效果：减少当前 escrow、永久登记 proposer ban、写入 SeenOffenses；下一次适用 cut 将该 position 退役。
- 已处理同一 offense 时返回 `AlreadyProcessed` 且不再扣款；无效/不完整证据返回 `REJECT` 且不改状态。

**功能。** 相对 $C$ 验证签名者、历史权重、admission 与冲突，不以最新权重替换历史权重。按 `(o,kind,slot)` 去重而非按证据字节去重，总扣款不超过 escrow；timeout、BLAME、区域 OFF 和单纯转发都不构成 slash。

#### 4.2.10 `SubmitQuit(C,g,Q)`

**输入**

- $C=(\Pi,A)$、$g$：要退出的 context 与恢复代。
- $Q$：以下三种完整证明之一：同一 admitted term 上权重 $>F_s$ 的 BLAME 证书；该 term leader 在同一 $(g,t,j)$ 的有效 proposal 双签；或 $g>0$ 时 recovery leader 的两个不同有效 NEWVIEW openings。
- `admission`：相对 $C$ 的 entry/quit 链、$g>0$ 时已有 REC、$t>0$ 时前序 terminal QCs，以及双签 proposal 所需的 quota 材料；作为 $Q$ 的组成部分提交。

**输出**

- `RecoveryRecorded(cid,g+1,ℓ_rec)`：唯一的新 REC，或已存在的同一 REC。
- 若 $Q$ 包含 offense，先返回并应用相应 slash/ban 结果，再记录恢复代来源高度；纯 BLAME 不扣款。
- 无效或缺必要证据时 `REJECT`，不创建 REC，也不预留该槽。

**功能。** 在 L1 原子处理所有类型的 quit proof；新代 ban 集由其记录高度的 finalized $\mathsf{Ban}$ 决定，之后保持固定。REC 的 key 为 $(\mathrm{cid},g+1)$，不是裸 $(\beta,r,g+1)$；调用者不能用任意 context 抢占真实 session 的恢复槽。

#### 4.2.11 `SubmitUpdate(U,Σ_U)`

**输入**

- $U=(chain,c,\theta_a,\beta_{\rm target},S_{\rm new},\mathcal L_{\rm new})$：目标链、序号、Espresso 激活阈值、目标分支、完整区域状态表与完整循环顺序。
- $\Sigma_U$：满足固定 multisig authority $\mathcal K$ 的认证。

**输出**

- `UpdateRecorded(c,λ_U,U)`：在 $\Lambda$ 的确定位置与 L1 发布高度。
- 状态效果：追加日志、更新 `lastSeq`；认证、序号、字段完整性或阈值规则不满足则 `REJECT`。

**功能。** 记录一个不可变的普通 Force 意图；$c>lastSeq$，$\theta_a$ 相对前一有效普通更新至少增加 $T_{\rm update}$，每个区域必须恰有一个 ON/OFF 标记并在顺序中恰出现一次。实际生效仍由 global rule 在对应 Espresso prelude 判定，stale target 整体 no-op；提前公告、$T_{\rm update}$ 与每 block 至多一个普通激活的条件沿用 §5，而不由 L1 自行验证 Espresso。

#### 4.2.12 `ScheduleMode(U_mode,Σ_U)`

**输入**

- $U^{\rm mode}=(chain,c,\beta_{\rm target},mode,\ell_a,boundary,policy)$：`mode` 为 STOP 或 RESUME，$\ell_a$ 为未来 L1 激活高度。
- `boundary`：§6 定义的共同截断/接续材料或待完成材料的标识；RESUME 的 `policy` 提供完整 $S_{\rm new},\mathcal L_{\rm new}$，STOP 可为空。
- $\Sigma_U$：multisig authority $\mathcal K$ 的认证。

**输出**

- `ModeScheduled(c,λ_U,U_mode)`：进入有类型控制日志的不可变记录。
- 状态效果：追加 $\Lambda$ 并更新 `lastSeq`；认证、序号、激活条件或字段格式不成立则 `REJECT`。

**功能。** 要求 $c>lastSeq$ 且 $\ell_a$ 晚于处理该请求的 L1 高度，公告一个可由 L1 驱动的模式切换；缺少最终 boundary 时只记录待完成状态，不把它解释为已安全完成接续。边界生效由 §6 的公共规则确定，可信 certifier 可将该结果报告给 L1；公告本身不授权回退 $G$ 或释放 stake。

#### 4.2.13 `ProveClosure(χ_close)`

**输入**

- $\chi_{\rm close}=\langle\mathcal T,\mathsf{CLOSURE},(chain,c,\beta_{\rm closed},activation,\lambda_{\rm floor})\rangle$：§4.7 输出的可信关闭确认。
- 本账本中的对应控制记录 $\Lambda[c]$；`activation` 指认其实际生效位置，$\lambda_{\rm floor}$ 是所有后续 plan 必须遵守的最低 L1 来源高度。

**输出**

- `ClosureRecorded(c,β_closed)`；若 $e^\star=e(\lambda_{\rm floor})$ 高于现有 clearance，则追加 `ClearedThrough(e*,τ_c)`。
- 状态效果：登记 Closures 与递增的 Clear 表；重复确认不重复清退，消息类型/链/控制对应关系不符则 `REJECT`。

**功能。** 直接信任 TEE 对“旧版本已按协议关闭且后续 floor 隔离旧职责”的结论，L1 不验证 Espresso header。该接口统一承接普通 Force 与 STOP 的清退，明确替换 v6 的 `ProveForce(U,J)` light-client 路径；$e_x(o)\le e^\star$ 的退役 position 可进入持有期，但关闭不等于马上退款。

#### 4.2.14 `Release(o,ownerAuth)`

**输入**

- $o$ 与 $owner_o$ 的提款授权。
- 本账本中的 $e_x(o)$、首个覆盖它的 $(e^\star,\tau_c)$、剩余 escrow 和已处理 offense 状态。

**输出**

- `Released(o,amount)`，其中 `amount` 为结算已有有效处罚后的剩余 $B_o$。
- 状态效果：一次性支付 owner、置 $B_o=0$ 和 `Released`、解除 key 的未结责任映射；条件不满足则 `REJECT`。

**功能。** 仅在已清退且真实持有期严格超过 $\tau_c+T_{\rm hold}$ 后释放资金，操作员在此前仍承担证据与数据保管责任。停机路径的持有期须覆盖 L1 关闭传播、证据收集与处罚提交，不能继续套用包含无界停机 $T_E$ 的旧预算；预算未定时保持锁定。

### 4.3 Session Manager：管理子协议生命周期

先固定模块间共享的数据类型，后续接口直接引用其精确结构。

| 类型 | 字段与含义 |
| --- | --- |
| $\nu$ | $(\beta,r,i,A,H,q^{\rm open},cause,\Pi)$，沿用 v3 descriptor；$H=h(A)+\mathrm{TB}$。 |
| `Handoff` | $(\nu,\mathcal P_\beta,\lambda^{\rm open},E,i_{\rm src},Q^0,evidence)$；$\lambda^{\rm open}=\max\{\lambda(A),\lambda(\Pi)\}$，$Q^0$ 为精确 anchor 的继承 commitment evidence。 |
| `Handoff.evidence` | REGULAR：前驱 handoff、精确 terminal DATA 与其 FINAL kernel；FORCE：控制记录与对应公开回放边界；RESUME：模式边界与 deposit-only anchor 的 L1 派生材料；GENESIS：初始 checkpoint。 |
| `RegionalResult` | $(C,B,D,\Sigma,\pi_{\rm path})$：目标 DATA、同 session 的 FINAL root、该 root 的签名集合、从 root 到目标的完整 hash-linked bodies。直接回执取 $D=B$。 |
| `ReadyBundle` | `(Handoff,st(A),stateEvidence,sourceReads)`；包含可直接使用的完整状态、其原生验证依据及 plan/reference 依赖的 `Read` 结果。 |
| `ControlEvent` | `(kind,c,closedVersion,newTail,boundary)`：由公共规则产生的关闭/恢复事件，固定实际 source cut 与 anchor。 |

完整 bytes 已在 `PublicStore` 中时，接口可传稳定标识；接收者必须能在该接口的处理预算内解析为上述完整对象。
Manager 维护 $(\mathcal X,R,PublicStore,Programs,ClosedVersions)$，以及按精确 anchor 索引的 baseline/successor states。
`Programs[s]` 保存固定 handoff、Assigned/Running/Terminated、单次 entry 与 termination 记录。

#### 4.3.1 `ObservePublic(reads)`

**输入**

- `reads`：一组 $R_{\rm L1}$ 或 $R_E$，即 §3.1 的 finalized 读取结果；可只含其中一种来源的新数据。

**输出**

- 更新后的公开回放状态 $\mathcal X$、$(G,\mathsf{Tail})$ 和模式 $\mu$。
- 由回放生成的 `ControlEvent` 与需准备的 handoff；来源尚不足时返回当前已完成进度及 `WAIT`。

**功能。** 先调用相应 `Verify` 并持久化来源，再执行 `AdvanceGlobal`；控制与数据处理由公共规则决定，不按网络到达先后选择分支。发现关闭事件后调用本模块的 `ApplyControl`，已有追加不因后来缺依赖而撤销。

#### 4.3.2 `ObserveRegional(result,handoffEvidence)`

**输入**

- `result`：上述 `RegionalResult`，其中 $C=(\Pi,A)$ 使用原 producing session。
- `handoffEvidence`：建立该 $C$ 的完整 Handoff 链与相应来源读取结果，或其已验证缓存标识。

**输出**

- `Stored(s,h(B),H(B))`：已验证并保管的区域结果，及相应更新的生产分支 $R$。
- 若是前驱 terminal DATA，则输出确定的 regular successor Handoff；验证失败返回 `REJECT`，来源缺失返回 `WAIT`。

**功能。** 为跨区交接接收公开可验证的区域结果，准备 successor state，并将完整结果交 Publisher。REGULAR 使用 $E_{s'}=A_s,A_{s'}=T_s$ 及 $q^{\rm open}_{s'}=\max\{\operatorname{height}(\xi(T_s)),q^{\rm open}_s+1\}$；有效当前版本的结果可以推进 $R$ 和准备后继，但不直接推进 $G$，关闭版本的结果只作历史保管。

#### 4.3.3 `InstallHandoff(handoff)`

**输入**

- `handoff`：完整 Handoff，包含固定 descriptor、policy、reference floors、anchor evidence 和 plan。
- 本机节点身份 $v$：用于检查是否属于被选中的固定委员会。

**输出**

- `Assigned(s,C)`：保存唯一 handoff 并创建本机 Program；重复相同输入返回现有实例。
- `NotAssigned`：验证通过但本机无该 session 的成员职责；冲突/关闭上下文返回 `REJECT`，来源未齐返回 `WAIT`。

**功能。** 验证 handoff 历史与 plan；已有相同实例时复用其当前状态，首次创建时检查目标版本当前有效且未关闭、该 pair 未进入或终止，然后调用 Program 的 `Create`。不会因为一个合法历史 handoff 仍可验证，就重新创建已关闭实例。

#### 4.3.4 `PrepareAndEnter(s,stateMaterial)`

**输入**

- $s$：已经 Assigned 的本机 session。
- `stateMaterial`：精确 $A_s$ 的完整状态及原生验证依据，或从已验证 checkpoint 到 $A_s$ 的完整执行输入；来源读取结果须覆盖该 handoff 的依赖。

**输出**

- `ReadyBundle` 以及 Program 的 `Entered(s,τ_entry)`；需要下载、重建或验证时返回 `WAIT`。
- 状态效果：保管 $\mathit{st}(A_s)$ 与 provenance，记录单次 entry；目标已关闭则 `REJECT`。

**功能。** 完成精确 anchor 的状态准备，在最终调用 Enter 前再次检查生命周期权限。只有可直接使用的完整状态才满足 Ready；state root 或下载地址不够，跨区通信界不代替重建时间。

#### 4.3.5 `HandleProgramEvent(s,event)`

**输入**

- $s$：事件所属 Program。
- `event`：`Result(RegionalResult)`、`Evidence(C,g,Q)`、`Offense(C,o,ev)` 或 `Complete(T_s,RegionalResult)`，均带上述接口要求的完整对象。

**输出**

- 持久化任务：`Publisher.Offer`、`Authority.SubmitQuit` 或 `Authority.SubmitOffense` 的精确调用参数。
- 对 Complete，更新 `Programs[s]` 为 Terminated，并输出 terminal relay/regular handoff 的待办；已处理事件返回原结果。

**功能。** 将程序的逻辑输出转换为不会随程序退出而丢失的交付任务，所有类型的 quit proof 都提交 L1。终止不删除回执、处罚证据或尚未完成的发送，Manager 不能把 Complete 事件当作一份尚未形成的 FINAL 回执。

#### 4.3.6 `ApplyControl(event,sourceEvidence)`

**输入**

- `event`：本机公共回放产生的 ControlEvent，或可由同一规则重新验证的外部事件。
- `sourceEvidence`：对应 finalized 控制、共同 boundary 和相关读取结果；RESUME 包含实际派生末端的 anchor 材料。

**输出**

- `Closed(β,sessionIds)`：原子持久化关闭标记，终止该版本全部 Assigned/Running 实例。
- 对有效开设/恢复，输出新的 Tail、plan 与 Handoff；旧/stale 事件输出 no-op，材料不足则在相应步骤 `WAIT`。

**功能。** 将公开控制结果落实到本机生命周期：关闭旧 callbacks、votes、timers、term/recovery 动作，按固定边界更新生产分支并保存已有 $G$。RESUME 从实际 deposit-only 状态准备新实例，不能从本机最远的旧 speculative program 接棒。

### 4.4 Session Program：固定 session 的区域共识子协议

Program 固定保存 $(s,C,H_s,\mathrm{TB},K)$，可变状态为 `(phase,g,t,locks,votes,QCs,timers,committedPrefix,executionState)`。
`phase` 沿 `Assigned → Running → Terminated` 变化，quit/recovery 只改变 Running 内部状态。
另一个 session 的实例不继承其 locks、votes 或 timers。
`Result/Evidence/Complete` 是程序输出事件，不再混作外部可以任意调用的完成指令。

#### 4.4.1 `Create(s,handoff)`

**输入**

- $s=(\beta,r)$ 与 Manager 已验证的完整 Handoff。

**输出**

- `Assigned(s,C)`：一个绑定固定 context、委员会和初始 anchor 的实例。
- 相同实例已存在则返回原实例；不同 handoff、已关闭或已终止的 pair 返回 `REJECT`。

**功能。** 建立独立的 session 状态和持久化身份，不复制前驱锁或授权提前投票。创建本身不启动 BFT timers，也不授予绕过 Ready 的签名权。

#### 4.4.2 `Enter(s,readyBundle)`

**输入**

- $s$ 与完整 ReadyBundle，精确状态必须对应该实例固定的 $A_s$。
- 当前 Manager 的版本关闭标记与单次 entry 记录，作为本机运行时状态读取。

**输出**

- `Entered(s,τ_entry)` 与本区域所有成员的 ReadyBundle 发送消息。
- 状态效果：初始化 entry lock $Q_s^0$、执行状态与 $(g,t)=(0,0)$，记录初始 entry 并设为 Running；权限失效则 `REJECT`。

**功能。** 在原子复查权限后只进入一次，并于该事件发送完整 ReadyBundle；随后按区域协议启动进度检查。genesis/regular/Force 初始进入不增加 STATUS/NEWVIEW/ANCHOR 轮，首个 DATA 仍须等待合法的 L1/Espresso references。

#### 4.4.3 `Provide(s,input)`

**输入**

- $s$：当前实例。
- `input`：`Candidate(parent,x,nativeContext)`、`References(R_L1,R_E)` 或 `Authority(g,AUTH_sg)` 三种有类型对象之一。

**输出**

- `Installed(kind,id)`：输入经验证后进入该实例的候选/来源缓存。
- `WAIT` 或 `REJECT(reason)`：缺少依赖、上下文不符、试图修改固定字段或目标已关闭。

**功能。** 将 builder 候选、已 finalized 来源和 REC 派生 authority 交给程序；每种输入均须匹配当前 context 与协议进度。提供新 authority 不改变 session 的固定成员权重，提供候选也不自动触发投票。

#### 4.4.4 `Step(s,event)`

**输入**

- $s$：目标实例；`event` 为 `Receive(m)` 或本机公共时钟产生的 `Timer(timerId,τ)`。
- $m$：完整签名消息，包含 domain、$\mathrm{cid}$、消息类型、对应 view/slot、payload 及该类型的 admission/parent 证据；FINAL 不绑定 $g,t$。

**输出**

- `peerMessages`：按协议发送的 PROPOSE、vote、STATUS、NEWVIEW、BLAME、FINAL 或 ReadyBundle 消息。
- `programEvents`：新形成的完整 Result、quit/offense Evidence、或带 terminal receipt 的 Complete；可以为空。
- 更新后的实例状态，或因必要依赖未齐而保留的等待任务。

**功能。** 执行 Part I/II 的区域状态转移，先处理有效冲突/quit 证据，再执行普通投票和 timer 逻辑；每次 BFT 动作检查该实例仍获授权。收到 QC 不等于已形成 FINAL，别的 session 的材料仅按明确的历史/handoff 用途读取，不能改变本实例的核心锁。

#### 4.4.5 `Close(s,controlEvent)`

**输入**

- $s$ 与 Manager 已验证的、覆盖该实例版本的关闭 ControlEvent。

**输出**

- `Terminated(s,c)`：记录控制序号和终止状态，重复关闭返回同一结果。
- 原有回执、证据与未完成输出的持久化移交记录。

**功能。** 立即取消该实例所有后续 BFT 权限，包括延迟 callback、term 变化和恢复，不等待物理资源清理。关闭不撤销已有 $G$，不生成新的 FINAL，也不直接释放 stake。

### 4.5 Publisher：交付完整区域结果

Publisher 维护按 `(cid,DATA hash)` 去重的待发布队列和 Espresso 提交状态。
任何完整结果持有者都可以调用，不要求原 leader 继续在线；付费路径属于 $T_{\rm pub}$ 的服务前提。

沿用 mega-espresso 的 record framing 思路，但区域 FINAL 取代单 publisher 的独占内容授权。
不能让一个仍然必须持有旧 publisher key 的提交条件破坏备份发布路径。

#### 4.5.1 `Offer(result,handoffEvidence)`

**输入**

- `result=(C,B,D,Σ,π_path)`：完整 RegionalResult；$\Sigma$ 是 $D$ 的 FINAL 签名集合，路径包含精确 DATA/DRAIN bodies。
- `handoffEvidence`：建立 producing context 的 Handoff 与所需已验证来源，或其可在服务界内解析的缓存标识。

**输出**

- `Queued(id)` 与随后 `Submitted(id,rid_E)`：稳定结果标识及 Espresso 请求标识。
- 后续 `Recorded(id,R_E)`：发布位置与可验证的 finalized 读取结果；证据无效时 `REJECT`，暂缺时 `WAIT`。

**功能。** 检查 producing context、`CheckFINAL` kernel 与 payload 绑定后，打包完整对象并调用 Espresso 的 Submit；正常条件下从完整结果可用起 $T_{\rm pub}$ 内提交，继而在 $T_E$ 内可见。`Recorded` 仅表示材料已发布，是否进入 $G$ 仍由全局规则决定；重复/分片结果可共同补齐，停机时队列持久保管。

### 4.6 Batcher 与 L1 Inbox：批次及状态 claim 的提交

Batcher 维护待提交 $G$ 区间和 channel 编码状态；inbox 维护已接受输入进度。
在本抽象中，cursor 写为

$$
cur=(n_{\rm acc},b_{\rm acc},ch,j_{\rm next},digest_{\rm frames},c_{\rm mode}),
$$

分别表示已接纳完整块的高度与标识、当前 channel、下一 frame 序号、已接纳 frame 前缀的承诺、模式边界序号。
$n_{\rm acc}$ 表示输入接纳进度，不表示 state root 已完成争议结算；中间 frames 不提前推进它。
该结构规定抽象字段含义，不要求原有实现的 cursor 使用相同 wire encoding。

#### 4.6.1 `BuildPayload(a,b,cur,X,channelState)`

**输入**

- $[a,b]$：拟提交的连续普通块区间，来自已验证 $G$ 且未被 inbox 消耗。
- $cur$：finalized L1 上读取的接受进度；$\mathcal X$：已验证的 global/mode 状态。
- `channelState`：本 batcher 已构造的 channel 前缀、压缩状态与 frame 进度；新 channel 则为空。

**输出**

- `payload`：按原有 codec 生成的同一 channel 的一个或多个连续新 frames。
- `channelState'` 与 `(prevCursor=cur,range=[a,b])`：本地编码进度及待认证目标。
- 不连续、跨越未认证模式边界或来源未齐时 `REJECT / WAIT`。

**功能。** 从精确全局输入构造批次，不使用仅在 $R$ 的 speculative 内容。不同 batcher 可采用不同压缩/打包方式；中断 channel 可按继承的接手规则重新打包，但必须满足 §6.4 对已有 $G$ 的保护。

#### 4.6.2 `SubmitBatch(χ_batch)`

**输入**

- $\chi_{\rm batch}$：§4.7.1 的可信 BATCH 消息，含 chain、前后 cursor、DA commitment 与对应输入范围。
- L1 caller：满足 inbox 当前提交权限的 batcher。

**输出**

- `CursorAdvanced(prev,next,d)`：finalized L1 接受事件。
- 状态效果：只有 `prev` 与 inbox 当前 cursor 匹配且普通提交仍被对应模式允许时更新 cursor；不满足则 `REJECT`。

**功能。** 直接使用 TEE 对输入对应性和 DA 绑定的可信结论，L1 不重验 Espresso 或 TEE attestation。合约仍原子检查自身的提交权限、cursor 连续性和模式接纳条件；一次旧证书不能覆盖已经改变的接受进度。

#### 4.6.3 `ApplySettlementBoundary(χ_mode)`

**输入**

- $\chi_{\rm mode}$：§4.7.2 的可信 MODE 消息，绑定控制序号、实际激活位置、protected prefix、接续 anchor 与派生位置。
- inbox 当前 cursor 与对应 finalized L1 模式控制记录。

**输出**

- `ModeApplied(c,prev,next,boundary)`：可供 derivation 按 L1 顺序处理的模式事件。
- 更新后的模式/cursor 与适当的 channel reset；protected prefix 尚未按选定契约完成交付则 `WAIT`。

**功能。** 让 settlement 的接纳边界与全局模式一致，并授权跳过已确定的 deposit-only 区间，而不是允许 batcher 自行跳过高度。channel reset 只能在受保护内容已妥善处理后执行；这里是对既有 inbox/override 的适配要求，具体时机随 §6.4 的 A/B 选择细化。

#### 4.6.4 `SubmitStateClaim(a,b,parentClaim,inputRef,stateRoot)`

**输入**

- $[a,b]$：claim 覆盖的连续 L2 高度；`parentClaim`：前置状态/claim 的固定标识。
- `inputRef`：确定该段派生输入的 L1 接受/模式位置及 DA commitments，绑定 chain 与 rule version。
- `stateRoot`：声称执行至 $b$ 后得到的状态根。

**输出**

- `ClaimRegistered(claimId)`：进入原有状态证明/争议流程的 claim 标识。
- 后续 `Resolved(claimId,accepted/rejected)`：原有机制给出的结算结果；未决时为 `PENDING`。

**功能。** 把状态根提交给已有 Kailua/fault-proof 路径，由该模块保证与同一派生输入和正确执行一致。该逻辑角色可由 batcher 的运营者承担，但 BATCH 消息不能替代状态证明；具体挑战、证明与时限沿用原系统，本文不另设 TEE 的 state-root 保证。

### 4.7 Certifier：可信第三方承担的业务功能

Certifier 由 §3.4 的 $\mathcal T$ 实现，维护其已验证的公开来源、global replay 状态和已认证业务结果。
它自行调用来源的 Read/Verify 与 DA 接口，不需要由 host 提供 quorum、远程认证或密钥证明。
以下返回的 $\chi$ 被消费者直接信任，无需额外的 `VerifyTEE` 接口。

#### 4.7.1 `CertifyBatch(prevCursor,payload,d,π_DA)`

**输入**

- `prevCursor`：本次请求以之开始的 inbox 接受进度；TEE 从 L1 Read 核对。
- `payload`：完整提交字节；$d,\pi_{\rm DA}$：这份 payload 已完成 DA dispersal 的结果。
- 延续 channel 所需的已接受 frames：通过其 DA commitments 读取，或使用 TEE 已保存的相同字节。

**输出**

- $\chi_{\rm batch}=\langle\mathcal T,\mathsf{BATCH},(chain,version,prev,next,d,range,globalRef)\rangle$。
- `range` 为本次完整消费的块区间，可为空；`globalRef` 绑定本次核对的 $G$ 前缀与模式控制，`next` 给出新的 frame/完整块进度。
- 内容不匹配返回 `REJECT`；必要来源或 frame 未取得则 `WAIT`，两者都不输出成功消息。

**功能。** 从已验证的 L1/Espresso 历史运行同一 global rule，检查 channel/frame 连续性和 DA 绑定，对完整解码块逐项核对高度、timestamp、L1 origin 与有序普通交易。未闭合 channel 不消耗 L2 块；可信消息只确认这些业务事实，既不保证未来交付时限，也不证明 state root。相比 mega-espresso，选择依据明确改为轮换后的 $G$，不再是单 publisher 的 first-valid record。

#### 4.7.2 `ConfirmControl(c,purpose)`

**输入**

- $c$：finalized L1 控制日志中的更新序号。
- `purpose`：`CLOSURE` 或 `MODE`；TEE 通过 Read/Verify 自行取得该控制、适用的完整来源与边界材料。

**输出**

- 对 `CLOSURE`，返回 $\chi_{\rm close}=\langle\mathcal T,\mathsf{CLOSURE},(chain,c,\beta_{\rm closed},activation,\lambda_{\rm floor})\rangle$。
- 对 `MODE`，返回 $\chi_{\rm mode}=\langle\mathcal T,\mathsf{MODE},(chain,c,mode,activation,sourceCut,protectedPrefix,anchor,derivePos)\rangle$，精确标识保留前缀、接续 anchor 与派生进度。
- 控制尚未生效或边界无法按 §6 规则确定时返回 `WAIT`；无效或 stale 控制返回 `REJECT`，不输出关闭/模式成功消息。

**功能。** 确认控制实际产生的公共效果，将跨链判断交给可信第三方而非 L1/Espresso 互相验证。CLOSURE 还确认未来 plan 的 floor 足以隔离旧职责；MODE 不替委员会任意选择更旧前缀。可信 TEE 执行已规定的规则，不能代替尚待细化的共同截断算法；边界不确定时仍不产生证书。

## 5. Global Finality Rule：公开验证与统一推进接口

该规则由每个 Manager、certifier 或外部观察者独立执行，不形成一个额外共识层。
正常模式沿用 v4.9 的 $\mathcal O_q=M_1\cup\cdots\cup M_q$ 与在 $J_q$ 生效的 $\Lambda_q$。
$M_q$ 只取自完整验证过的 namespace；直接收到的区域结果不加入该集合。

回放状态以原有 $\mathcal X=(\mathrm{Out},\mathcal P_\beta,\kappa)$ 为基础，增加模式 $\mu$、待生效控制、固定模式边界与 deposit-only 派生进度。
$\mathrm{Out}=(G,\mathsf{Tail})$，来源的精确字节保存在 `PublicStore`，而不是只保存哈希。

### 5.1 公开回执检查

#### 5.1.1 `CheckFINAL(C,B,D,Σ,π_path)`

**输入**

- $C=(\Pi,A)$：检查者已经从 canonical handoff 历史导出的 producing context，不是仅凭发布者声明的 context。
- $B$：待检查 DATA；$D$：直接承载 FINAL 的同 session DATA，允许 $D=B$。
- $\Sigma$：绑定 $\mathrm{cid}(C)$、$h(D)$ 与 $\mathsf H(D)$ 的不同成员 FINAL 签名。
- $\pi_{\rm path}$：从 $D$ 回到 $B$ 的精确 DATA/DRAIN bodies；直接回执时只有 $B$。

**输出**

- `VALID`：同 context 的签名权重 $>F_s$，且路径/hash/position 与 session quota 检查通过。
- `INVALID`：检查失败；`INCOMPLETE`：必要对象尚缺。

**功能。** 检查 v4.9 的公开 FINAL kernel，覆盖仅限同一 producing session 的 ancestry；其 soundness 依赖固定权重故障界和正确签名者的完整区域验证。它不把一个旧/关闭 session 的历史回执变成当前 global append 权限，也不要求 publisher 提供每个 recovery 的 optional sidecar。

### 5.2 统一推进

#### 5.2.1 `AdvanceGlobal(X,reads)`

**输入**

- $\mathcal X$：从 genesis、授权 checkpoint 或本机已验证前次转移恢复的回放状态，及对应 `PublicStore`。
- `reads`：新增的完整 L1/Espresso 读取结果，覆盖当前处理步骤的 namespace、控制前缀、plan 来源；deposit-only 步骤可以只增加 L1 输入。

**输出**

- $\mathcal X'$ 与 $(G',\mathsf{Tail}')$：新的已完成回放状态。
- `appendedBlocks` 与 `lifecycleEvents`：本次确定追加的块，以及确定的 open/close/resume ControlEvents。
- `status=OK` 或 `WAIT(missingObjects,subposition)`：缺依赖时保存精确暂停位置，已经完成的追加仍被保留。

**功能。** 验证并存储来源后，NORMAL 按 §5.3 推进，模式转换与 deposit-only 按 §6 推进；不以消息到达顺序创建第三个排序来源。只有公共规则确定的责任才能追加块和产生生命周期事件，Manager 不能改写输出；候选缺材料与处理本身缺必要来源按不同等待规则处理。

### 5.3 NORMAL 模式的核心转移

对每个 finalized $J_q$，先处理有效控制，再反复追加与当前 Tail 匹配的 DATA。
普通控制在首个 $\theta_q\ge\theta_a$ 的 Espresso block 开始处生效，早于该块中的普通对象接纳。
沿用 $\theta_a>\tau_{\rm submit}+T_{\rm L1}+\epsilon_E$ 的提前公告条件，以及每个 Espresso block 至多一个普通更新激活的独立假设；单靠 $T_{\rm update}$ 不能推出后者。

| 检查 | 要求 |
| --- | --- |
| 来源与责任 | $\mu=\mathsf{NORMAL}$；块属于当前 descriptor 的 chain、session 和 context。 |
| 连续性 | parent 为当前 $G$ 的精确末端，高度加一，且不超过该 session 的 $H_s$。 |
| 区域最终性 | $\mathcal O_q$ 中的 bodies 与签名使 `CheckFINAL` 返回 VALID。 |
| Espresso reference | $\xi(B)$ 指向 finalized $J_{q'}$，满足 $q^{\rm open}_s\le q'<q$。 |
| 控制边界 | 已生效 Force/STOP 没有撤销该 session 对下一块的责任；已在 $G$ 的块不被追溯撤销。 |

追加时绑定 $\sigma_G(B)=\nu$；到 terminal DATA 后，使用固定 policy 和前驱初始 anchor 作为 selection reference，确定 regular successor。
若多个候选满足条件，沿用 least-hash tie-break 使函数完全确定；正常安全论证仍须排除冲突的有效结果。
对象集合累积，缺失材料后来补齐后可重新使候选合格，不因 outer EXPORT hint 过期而要求重新发布。

单个候选缺 body/signature 只使它尚不合格，optional sidecar 不得卡住其他候选。
缺完整 namespace、适用控制前缀或真实目标 plan 的来源，则停在该操作等待。
terminal DATA 先持久化追加，再计算 successor，后者来源暂缺不撤销已接纳 terminal DATA。

普通 Force 保留此前 $G$ 并安装新 policy，从精确 retained tip 创建新分支，关闭旧版本全部实例。
旧回执保留历史意义，但旧后缀不能借迟到发布重获 append 权限。

## 6. Espresso 停机时的模式扩展

### 6.1 两种模式，共同的一条全局链

本稿采用如下最小扩展：$G$ 包含普通 DATA 对应的 L2 块，也包含由 L1 派生并确认的 deposit-only L2 块。
两类块共享连续的 L2 高度和明确的 parent/state 关系，但使用不同的最终性证据。
这个扩展改变了旧稿中“$G$ 只含区域 DATA”的类型，不能直接套用旧定义而不修改。

| 模式 | 谁确定新区块 | 追加到 $G$ 所需证据 |
| --- | --- | --- |
| $\mathsf{NORMAL}$ | 获准的区域 session | 区域 FINAL、Espresso finalized 对象、Authority 与 global-rule 检查。 |
| $\mathsf{DEPOSIT\_ONLY}$ | 确定性 L1 derivation | 已认证的模式边界、完整 finalized L1 输入、正确的派生与执行。无需区域 FINAL 或新 Espresso block。 |

“Deposit-only”沿用原生含义：普通交易列表为空，仍含按 L1 规则要求的 deposits、forced-inclusion 和系统输入。
局部 timeout 只能触发监控报告；multisig 的 finalized L1 控制才改变模式。
这个恢复范围不包含任意 L1、DA 或执行证明系统同时失效的情形。

```mermaid
stateDiagram-v2
    NORMAL --> DEPOSIT_ONLY: L1 STOP 生效，保护共同前缀，关闭旧实例
    DEPOSIT_ONLY --> DEPOSIT_ONLY: 根据 finalized L1 追加确定性块
    DEPOSIT_ONLY --> NORMAL: L1 RESUME 生效，从实际派生末端开新 session
```

### 6.2 模式控制的接口契约

新增控制记录在概念上包含：

$$
U^{\rm mode}=
(\mathit{chain},c,\beta_{\rm target},\mathit{mode},
\ell_a,\mathit{boundary},\mathit{policy}),
$$

并由既定 multisig authority $\mathcal K$ 认证。
`policy` 在 RESUME 时提供完整 $S_{\rm new},\mathcal L_{\rm new}$；STOP 不需要虚构一个正在出块的新 session。
本稿用未来的 finalized L1 高度 $\ell_a$ 表示激活点；若改用时间戳，也必须是具有明确 finalized L1 判定规则的时间条件。
L1 激活点、Espresso 的 $\theta_q$、L2 block timestamp 和观察者的到达时间不是同一坐标。

`boundary` 不是让委员会任意指定一个历史块，而是携带或引用**所有观察者可验证的共同截断/接续材料**：

| 控制 | boundary 必须确定的对象 | 生效效果 |
| --- | --- | --- |
| STOP | 可重放的 Espresso source cut、完整控制上下文、对应 $G^\star$ 及其原有证据 | 停止普通接纳，关闭旧版本所有实例，保留 $G^\star$，固定 deposit-only 的起点。 |
| RESUME | 按 L1 控制顺序确定的 deposit-only 结束位置、精确 anchor 与状态来源，以及可验证的 Espresso 恢复上下文 | 从该 anchor 开新版本，设置新 policy/floor 和 reference bounds，再准备 session。 |

对 STOP 的核心不变量是：

$$
\text{任何正确观察者此前已确认的 }G\text{ 都是 }G^\star\text{ 的前缀}.
$$

这个条件不能靠“收到指令时取本机的 last(G)”实现，因为节点可能有不同的同步进度。
也不能仅靠“拒绝 block timestamp 晚于某值”实现：旧时间戳的迟到结果与已派生的 deposit-only 高度仍须有唯一归属。

**待细化的边界实现。** 当前 v3 和 mega-espresso 都没有上述 STOP boundary 协议。
后续需要把预告、普通接纳的截止、source cut 的固定及 L1 确认之间的顺序补成可执行规则，保证既不回退已确认前缀，也不漏掉仍被允许最终化的结果。
公告和边界材料可以分阶段取得，但在材料完整前不得自行认定 deposit-only 的起点。
停用生效后，边界材料缺失只能使接续等待，不能作为继续确认新普通块的许可；这一停止条件也必须按共同来源规则判定，不能取决于节点收到消息的先后。
这里声明的是模式接口必须满足的组合义务，不把“存在一个完美截断 oracle”新增为外部假设，也不声称单个时间过滤已经实现它。

### 6.3 STOP、deposit-only 与 RESUME 的转移

**STOP。** 在可验证的 L1 生效条件与完整 boundary 均满足后，所有节点按同一控制顺序固定 $G^\star$，关闭旧版本，并把普通生产责任替换为 deposit-only 派生责任。
关闭覆盖所有旧 session，包括未 Enter 的 Assigned 实例和领先于 $G$ 的后继实例。
已进入 $G^\star$ 的块及其证据继续可用于批处理；超出该前缀的旧结果不再拥有新增全局块的权限。

**Deposit-only。** 先取得 $A^\star$ 的精确状态，履行 §6.4 的受保护前缀交付规则，再按 L1 派生规则顺序追加 $B^{\rm dep}_{h(A^\star)+1},\ldots$。
这些块的内容和推进条件不能由本地墙钟或 multisig 自行提供的 state root 决定。
如果前置状态或此前必须处理的已接受 DA 数据缺失，等待；不能将“未知”解释为空交易列表。

**RESUME。** 本稿建议仍以显式 finalized L1 控制恢复，避免各节点按自己的 Espresso 网络观测自行切换。
恢复边界先按 L1 事件顺序完成 deposit-only 前缀，再把其末块作为 $A_s=E_s$，安装新 $\beta$ 和 $\lambda_\beta$，计算 plan 并执行 Ready/Enter。
regional turn index $r$ 沿用关闭边界记下的下一轮编号；deposit-only 块不虚构区域 session，也不消耗某个旧 session 的 quota。
新 session 第一块继续满足已定义的 L1/Espresso reference floors；如果来源或状态没有准备好，就等待。
被关闭 session 的迟到结果不能替换任何已进入 $G$ 的 deposit-only 块。

例如，STOP 保护至 108，deposit-only 生成 109–120，则恢复后的普通 session 从 120 的真实状态产生 121。
旧 session 即使后来交出针对原生产分支的 109 或 121，也不会因此取得当前生产权限。

为了保留记号而不伪造区域证明，作以下类型扩展：

- $\sigma_G(B)$ 对普通块仍是原来的 $\nu$；对 deposit-only 块记录 `L1DEPOSIT(mode-control, derivation-position)` 及其可验证来源。
- $\mathsf{Tail}$ 增加 `DEPOSIT(...)` 分支，表示下一块由 L1 规则决定；`NEXT(ν)` 与 `PENDING(ω)` 保留原义。
- handoff 的 anchor evidence 从“原 producing session 的 FINAL 或 genesis 证据”扩展为“也可使用已确认 deposit-only 块的 L1 派生证据”。
- 正常片段仍用 $G_q$；整个系统的进度还取决于 finalized L1 位置和模式状态，不能只用 Espresso 高度 $q$ 索引。

回放仍是两份公开来源历史上的确定性规则，不为 L1 和 Espresso 假设一个天然共同的交易排序。
NORMAL 内的顺序沿用 Espresso prelude；模式转换按已认证的 L1 boundary 接合，观察者消息到达顺序不得成为第三个排序输入。

所有模式统一使用 §5.2 的 `AdvanceGlobal`；模式、边界与派生进度是回放状态的一部分。
一次已完成追加不会因后续边界材料缺失而撤销。

### 6.4 已进入 $G$ 的积压块：必须显式选择的契约

Espresso 仅停止新块最终化时，已经存在的完整记录与证明仍可用于结算。
因此，保护 $G^\star$ 本身不要求 Espresso 恢复出块。
但强最终性还要求这些块不因后续 batch/DA/L1 交付迟延而被替换。

| 实现取向 | 需要提供的保证 | 与 mega-espresso 的关系 |
| --- | --- | --- |
| A：在服务假设内保护 | 进入 $G$ 的块通过原生有效性检查；已有证据与数据保留；正常/替代 batcher、DA、L1 在该块**剩余** sequencing window 内完成所需交付，覆盖 channel、batcher 准入、接手等等待 | 可以尽量保留现有批处理/派生规则，但强最终性明确以这些服务前提为条件。不能只写一个未证明满足窗口的总时限。 |
| B：规则强制保护 | L1 inbox、derivation 与状态证明系统识别已认证的 protected prefix；允许所需的补交/接续，并禁止把其中块替换成 deposit-only | 需要修改现有窗口/恢复语义；数据不可取得时保住安全性但可能停止进展。不能只改观察者的 $G$ 标签。 |

**本稿不在 A/B 之间替用户作最终选择。**
共同上层契约是 `PreserveGlobalPrefix(G*)`：后续结算必须原样保留这一前缀，并从其精确状态开始 deposit-only。
这不是一个新模块，也不是已证明可调用的 ideal oracle，而是 Batcher–Inbox–Derivation–状态证明四个边界必须共同实现的性质。

例如，$G^\star$ 到 108 而 L1 尚只结算至 105，则 106–108 必须完成交付，或在不可得时等待；不能用 deposit-only 替换它们以声称“恢复成功”。
如果既不提供 A 的及时交付前提，也不实现 B 的保护规则，就不能声称“$B\in G$ 保证该块最终被保留”。

### 6.5 与 mega-espresso 恢复能力的对照

| 情形 | 本稿的处理 | 继承或差异 |
| --- | --- | --- |
| Espresso 不再最终化新记录 | 旧 $G$ 的可用记录继续交付；经 L1 STOP 后进入 deposit-only | 保留其基本恢复路线，新增显式模式控制。 |
| 原 batcher 停止或 channel 未完成 | 替代 batcher、超时后重新打包 | 可继承，但不能因接手太晚丢弃已有 $G$；须满足 §6.4。 |
| 停机期间仍有区域生产 | 结果可保管，但关闭边界之外不进入 $G$ | 区域回执不能绕过 STOP。 |
| Espresso 恢复 | 从实际 deposit-only 链末端恢复普通 session | 继承“跟随实际派生链”的原则；补上轮换版的 RESUME/handoff。 |
| 已接受的 DA 数据不可得 | 停在缺失处等待 | 继承现有取回规则，不把取回失败当作空块。 |
| 多签没有行动 | 显式 STOP 不会自行发生 | 比自动 sequencing-window fallback 多一个控制服务进展前提。 |

本稿可以实现与 mega-espresso 同类的“普通服务暂停—L1 继续—普通服务恢复”。
它不继承“任何未结算普通内容都可为恢复进度而丢弃”，因为用户已选择更强的 $G$ 语义。
有界恢复还需要委员会响应界、L1 生效界、边界材料交付界及状态准备界；“委员会诚实”本身不提供这些时间界。

## 7. Derivation Rule：外部观察者如何得到同一条链

每个观察者维护自己的派生状态

$$
\mathcal D=(head,\mathit{st}(head),l1Cursor,channelBuffer,\mu,boundary,acceptedInputIds).
$$

`l1Cursor` 精确到交易/事件位置，`channelBuffer` 保存已接纳且尚未完整解码的 frames；模式与边界按 L1 公共事件顺序确定。
初始状态来自独立验证的历史或明确授权的 checkpoint。

### 7.1 派生接口

#### 7.1.1 `Derive(D,reads_L1,DAobjects)`

**输入**

- $\mathcal D$：已验证的前次派生状态或初始 checkpoint。
- `reads_L1`：完整有序的 finalized L1 读取结果，包含 deposits、系统配置、inbox 的 `CursorAdvanced/ModeApplied` 和适用的控制记录。
- `DAobjects`：从已接受 commitments 解析出的 $(d,payload,\pi_{\rm DA})$；可由本节点按需 ReadDA 取得。

**输出**

- `derivedBlocks`、相应执行状态/roots 与新的 $\mathcal D'$。
- 已确认 deposit-only 区间的派生位置与精确末端，供 global rule 或 RESUME boundary 使用。
- `WAIT(missingObjects,subposition)`：必要 L1/DA/前置状态缺失时停在明确位置；已完成部分不撤销。

**功能。** 按 finalized L1 顺序读取被 inbox 接受的普通输入与模式事件，核对 DA 字节后解码并执行；DEPOSIT_ONLY 根据相同 L1 历史构造普通交易列表为空的块。节点不直接重验 Espresso，也不重验 TEE，而通过 L1 已接受的可信业务结果承接输入对应性；模式控制 calldata 不是 batch 数据，任何跳过高度都必须由同一已认证边界解释。

### 7.2 与强最终性的连接

提前确认 $G$ 的观察者读取 L1、Espresso 并运行 global rule；最终派生的观察者主要读取 L1、DA。
两条路径必须得到相同的受保护块及执行状态，这是需要建立的组合性质。
普通块的原生合法性、L1 origin、timestamp 与执行条件必须在授予强最终性前由区域完整验证保证；输入对应性证书本身不替代这些检查。

跨模式的 channel 必须保留和完成 protected prefix，按 §6.4 的实现契约舍弃不再获准的未完成后缀。
已接受但取不回的 DA payload 会使节点等待，不能转成空普通交易列表。
已最终化的 deposit-only 高度不能被补发的旧普通记录覆盖；恢复使用实际派生 parent/state。

## 8. 通信、职责与组合检查

### 8.1 各类通信的归属

| 从 → 到 | 内容 | 边界 |
| --- | --- | --- |
| 用户 → Builder | 普通交易 | 用户入口；无隐含包含保证。 |
| Staker/任意证据提交者/multisig → L1 Authority | 质押、quit/offense、配置与模式控制 | L1 认证与执行；权限按入口区分。 |
| Manager ↔ 本机 Program/Builder | 生命周期、候选输入、来源、状态、回报 | 本机接口；不可绕过验证与持久化。 |
| Program ↔ 同 session replicas | BFT、FINAL、recovery、ReadyBundle | $\Delta_i$；域与 $\mathrm{cid}$ 隔离。 |
| Manager ↔ 跨区域 Manager | 完整结果、handoff、可验证状态 | $\Delta_{\rm cross}$；不传递可继承的投票权限或锁。 |
| Program/Manager → Publisher → Espresso | 完整区域结果与 EXPORT 对象 | $T_{\rm pub}+T_E$ 的正常服务契约。 |
| Batcher ↔ 可信 Certifier；两者 ↔ DA | payload、DA commitment、可信 BATCH 消息 | Certifier 自行读取核对来源；消费者直接信任其业务结论。 |
| Batcher → L1 inbox；观察者 ← L1/DA | 批次接受与派生 | 接受事件、commitment 绑定和模式边界。 |

### 8.2 功能覆盖检查

| 所需功能 | 负责的接口/规则 | 不能遗漏的条件 |
| --- | --- | --- |
| 不同 agent 验证 finalized 数据 | §3.1 | Submit/Read/Verify 同时覆盖最终性、精确内容和查询范围完整性。 |
| 区域内快速同步、跨区域同步 | §3.2 | 正确路径、负载与完整依赖；状态重建另计。 |
| 每 epoch 权重表、join/quit/slash | §4.2 | 快照冻结、责任保留、历史权重不重写。 |
| 配置区域顺序与 Force | §4.2、§5 | 完整 policy、公共激活点、保留 $G$。 |
| 多 session 并发与生命周期 | §4.3–4.4 | 独立核心状态、单次 Enter、版本关闭与持久化。 |
| leader 不发布时仍能送达 | §4.5 | 任何完整结果持有者可转发，有可用付费路径。 |
| 唯一全局链 | §5–6 | 来源完整、回放确定、模式边界共同且不回退。 |
| DA 内容与全局输入一致 | §4.6–4.7 | 以 $G$ 为选择规则；可信消息绑定精确 DA 输入与模式。 |
| state root 对应正确输入执行 | §3.5、§4.6.4、§7 | 原有证明系统覆盖相同 derivation 与模式版本。 |
| Espresso 停机后仍能推进 L1 路径 | §6 | L1 驱动的 STOP、protected prefix、可派生状态。 |
| Espresso 恢复后重新出块 | §6.3 | 实际派生 anchor、新版本、Ready、重新验证来源。 |
| 停机后的 staking 安全释放 | §4.2、§4.7 | 可信关闭确认、L1 清退记录与不依赖停机 $T_E$ 的持有期。 |
| 旧数据取回与节点重新上线 | §3.6 | 数据保留或授权 checkpoint，不能只保留哈希。 |

这个接口集合覆盖了本次要求的职责，但不意味着每一项已由现有规格实现。
特别是模式切换的共同截断、强最终化前缀保护、停机后的 stake 清退和轮换版 certifier，都是本次需要补齐的集成工作。

### 8.3 保持抽象简单的约束

1. **只保留两个持久来源链。** 不额外增加跨区域共识日志；跨区消息用于提前准备，公开接纳仍由相同规则决定。
2. **只保留一个 L1 Authority 逻辑模块。** 快照、ban、REC、multisig 共享认证和日志来源；`Plan/Leader` 是其中的纯查询。
3. **共识只拆成 Manager 与实例。** 部署在同一节点的持久化、取回和转发不各自成为新层。
4. **全局最终性与 derivation 都是规则。** 不引入一个有自由裁量权的“global finalizer”。
5. **恢复只新增两种模式之间的明确边界。** “准备中/等待证据”是运行状态，不额外命名一串协议模式。
6. **把跨模块保证写清楚。** 可用性不由 finality certificate 代替，执行正确性不由 TEE 输入对应性代替，强最终性不由 timeout 后静默丢弃替代。

### 8.4 下一轮形式化的四个具体任务

| 任务 | 可检查的完成条件 |
| --- | --- |
| 共同 STOP/RESUME 边界 | 从 finalized L1 与 Espresso 历史确定唯一衔接；迟到消息不改结果；不等待新的 Espresso block 才能停止；旧 $G$ 始终被保留。 |
| protected-prefix 实现选择 | 明确选择 §6.4 的 A 或 B，写出服务预算或派生/接纳修改，覆盖已部分提交的 channel。 |
| 两个来源的接纳与执行对应 | 规定普通块与 deposit-only 的统一高度、parent、anchor evidence、metadata 编码；global rule、certifier、derivation 和状态证明使用同一语义。 |
| 生命周期与退出闭合 | STOP 关闭所有旧实例，RESUME 使用新的 floor；由可信 Certifier 确认关闭并在 L1 清退，给出适用的持有期，不能遗留一个永久等待 Espresso 的释放条件。 |

随后才证明模块替换所需的性质：输入与状态一致性、全局前缀不可回退、session 隔离，以及各服务前提下的进展。
现有局部证明可以作为子结论使用，但新增模式下的完整组合结论需要重新论证。

## 9. 本稿所依据的源文件

| 来源 | 使用范围 |
| --- | --- |
| [Part I](sync_hotstuff_rr_part1_core.tex) | 区域同步共识与恢复引擎。 |
| [Part II v4.9](sync_hotstuff_rr_part2_rotation_v4_9.tex) | §2 authority/外部服务、§3 记号、§5 FINAL 与 global rule、§6 Manager/Ready、附录存储职责。 |
| [Part III v6](sync_hotstuff_rr_part3_staking_v6.tex) | 快照、L1 floor、Plan/Leader、REC、slash 与 release。 |
| [Part IV v0.1](sync_hotstuff_rr_part4_force_v0_1.tex) | Publisher 的备份发送、费用与保留；不取其旧 authority 接口覆盖新稿。 |
| [第三轮迭代说明](iteration_round3_zh.md) | 最新工作版本与尚未同步的 README/Part IV 状态。 |
| [mega-espresso 总览](../megaeth_specs/specs/block-certification/overview.md) | 输入认证、DA、TEE 与结算的职责划分。 |
| [Block records](../megaeth_specs/specs/block-certification/block-records.md) | 普通交易记录、publisher 授权、batch/fragment 与 namespace。 |
| [Certifier](../megaeth_specs/specs/block-certification/certifier.md) | 批次与来源输入对应性的业务语义；其 TEE 实现机制在本稿用可信第三方替代。 |
| [Batch inbox](../megaeth_specs/specs/block-certification/batch-inbox.md) | 接纳、替代 batcher、channel recovery 与 override。 |
| [Recovery](../megaeth_specs/specs/block-certification/recovery.md) | 无损恢复条件，S1/S2/S4/S9/S10。 |
| [Derivation](../megaeth_specs/specs/rollup/derivation.md) | 接受事件、原生派生、deposit-only 与 override 顺序。 |
| [DA](../megaeth_specs/specs/rollup/data-availability.md)、[batch submission](../megaeth_specs/specs/rollup/batch-submission.md) | 精确 payload 取回与 provider 接口。 |
| [Fault proofs stub](../megaeth_specs/specs/rollup/fault-proofs.md) | 标识原有状态证明接口尚未在该子仓库完整规范化。 |
