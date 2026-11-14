# 以太坊和 Restaking 的质押进出机制及区域轮换设计

学习与设计参考日期：2026 年 10 月 10 日。对应当前 [main_v3.tex](../../formalization/main_v3.tex)，补充其尚未展开的 permissionless admission、stake snapshot 和 configuration transition。本文前半部分解释现有机制，后半部分提出适合本项目的设计框架；后者是待形式化的方案，不是已有安全性定理。

**最值得借鉴的是把资金、成员资格、生产排班和历史责任分开管理。** 用户可以随时申请加入或退出，但投票权在统一快照边界生效，退出资金在旧责任结束后才释放。Epoch 提供批量更新的节奏，churn limit 限制每次变化的规模，随机性决定排班，安全交接决定新配置何时真正取得权限。这四件事需要分别定义。

这里将“流量控制”主要理解为质押进入、退出及权重迁移的速率控制，同时讨论节点数量、状态同步和共识带宽。30 分钟先作为我们的候选 epoch 长度，不作为已经论证过的安全参数。

阅读路线：第 1 至 4 节建立以太坊的完整图景，第 5 至 7 节比较 staking service 和 restaking，第 8 至 12 节给出对 v3 的设计建议与例子。

## 1 先分清参与系统的四种身份

| 身份 | 提供什么 | 加入或退出意味着什么 |
| --- | --- | --- |
| 资金提供者 staker | 抵押资产 | 存入、委托、申请赎回、领取资产 |
| 运行节点的 operator | 机器、网络、签名服务 | 注册密钥、同步状态、接受任务、停止新任务 |
| 某配置中的 validator | 该配置下的共识投票权 | 在明确边界取得或失去投票资格 |
| 当前 leader | 某个生产机会的提议权限 | 按名单或抽签临时取得出块权 |

同一个人可以兼任四种角色，但协议不能因此合并四种状态。例如，operator 已经停止担任 validator，委托给它的资金仍可能承担历史罚没责任；一个 validator 没有轮到 leader，也仍可能需要投票和保留数据。

“Permissionless”也有不同层次。任何人都能存款、任何 operator 都能注册、任何满足公开条件的 operator 都能进入服务集合、任何符合排班规则的 leader 都能实际开始生产，分别是不同承诺。一个开放存款的 vault 可以使用有准入限制的 operator 集合；一个开放注册的系统也可能由多签决定哪些 region 可服务。

对我们的项目，比较准确的目标可以是：**质押申请与节点注册开放，成员生效按公开规则确定，区域可用性由有明确边界的委员会管理。** 这个目标本身允许一定治理信任，但必须把信任具体写出来。

## 2 以太坊如何把存款变成验证权

### 2.1 当前版本和基本时间单位

以下以主网已启用的 Pectra／Electra 与 Fusaka／Fulu 机制为基础。官方主网配置给出 Electra 于 2025 年 5 月 7 日、Fulu 于 2025 年 12 月 3 日激活；该配置中的 Gloas 仍使用未激活占位值。因此不能把开发中 Gloas 的退出参数混入当前规则。[主网配置](https://raw.githubusercontent.com/ethereum/consensus-specs/master/configs/mainnet.yaml)

以太坊一个 slot 为 12 秒，一个 epoch 为 32 个 slots，即 384 秒，约 6.4 分钟。我们的 30 分钟约等于 4.69 个以太坊 epoch，并不是一个以太坊原生单位。Epoch 边界用于更新状态和职责，但边界到来不代表本 epoch 所有请求立即生效。[主网时间配置](https://raw.githubusercontent.com/ethereum/consensus-specs/master/configs/mainnet.yaml)、[主网共识预设](https://raw.githubusercontent.com/ethereum/consensus-specs/master/presets/mainnet/phase0.yaml)

### 2.2 实际余额和有效余额

以太坊区分实际余额与有效余额 effective balance。实际余额记录存款、奖励和扣款；有效余额用于共识权重及相关选择，按规定粒度与滞后阈值更新，避免每次微小余额变化都扰动共识计算。

激活门槛仍是 32 ETH。传统 `0x01` 提款凭证的有效余额上限为 32 ETH；`0x02` compounding validator 的有效余额上限为 2,048 ETH。这不意味着新节点必须交 2,048 ETH，也不意味着“所有 validator 的权重都相同”。合并 validator 还具有单独的处理路径和 churn 预算。[EIP-7251](https://eips.ethereum.org/EIPS/eip-7251)

对我们的记号，原始资金余额应与进入共识的权重分开。某人刚增加质押，并不必然立刻改变当前 session 中的投票权。

### 2.3 一笔加入请求经过哪些阶段

可以把新 validator 的加入理解为以下流程：

```mermaid
flowchart LR
    A[执行层存款] --> B[有序存款请求]
    B --> C[等待最终性和处理预算]
    C --> D[余额满足激活条件]
    D --> E[确定未来激活 epoch]
    E --> F[开始共识职责]
```

首先，用户向存款合约提交资产、公钥和提款凭证等信息。EIP-6110 让执行层从存款日志生成请求，按照日志顺序交给共识层处理。这里的“顺序”是链上已经确定的顺序，不是互联网中谁先广播，也不是钱包显示的点击时间。[EIP-6110](https://eips.ethereum.org/EIPS/eip-6110)

随后，共识状态维护 pending deposits。处理受请求位置的最终性、每 epoch 的处理数量限制和资金 churn 预算约束。新密钥还要经过相应的存款签名检查；收到合约日志本身不等于已经获得验证权。达到余额条件之后，激活资格还要满足最终性及未来激活时间的规则。[Electra 存款和注册更新规则](https://ethereum.github.io/consensus-specs/specs/electra/beacon-chain/#epoch-processing)

这形成两层控制：L1 决定请求的共同顺序，共识规则决定请求何时转化为职责。链上交易已成功，只能说明流程中的某一步完成。

### 2.4 Churn 限制的是变更预算

在 Electra 规则下，把总活跃有效余额记作 $B$，以 ETH 为单位，资金 churn 的基础额度可写成

$$
C_{\mathrm{base}}(B)
=\left\lfloor\max\left\{128,\frac{B}{65536}\right\}\right\rfloor,
\qquad
C_{\mathrm{entry/exit}}(B)=\min\{256,C_{\mathrm{base}}(B)\}.
$$

这里的取整对应 1 ETH 粒度；实现内部使用 Gwei。进入和退出使用相应的独立状态记账，不能理解为两者共享一个 256 ETH 池。合并 validator 使用另行计算的预算。[Electra churn 函数](https://ethereum.github.io/consensus-specs/specs/electra/beacon-chain/#beacon-state-accessors)、[参数定义](https://raw.githubusercontent.com/ethereum/consensus-specs/master/configs/mainnet.yaml)

当该额度达到上限时，256 ETH／epoch 对应持续拥堵情况下每方向约

$$
256\times\frac{86400}{384}=57{,}600\ \text{ETH／天}
$$

的额度生成速度。若全部是 32 ETH 的 validator，才可以换算成平均每 epoch 8 个 validator 的规模。对 2,048 ETH 的 validator，不能继续按“一个身份占一个名额”理解。

**这个值是预算生成速率，不是每个边界实际生效金额的绝对上限。** 大额请求可能需要累积预算；实际激活还与资格、最终性和调度相关。因此不能从“256 ETH／epoch”推导出任何单个 epoch 的活跃余额最多只增加 256 ETH。[Fulu pending deposits 规则](https://ethereum.github.io/consensus-specs/specs/fulu/beacon-chain/#modified-process_pending_deposits)

教学例子：在固定 256 ETH 额度下，前方 25,600 ETH 的同方向积压，大致对应 100 个 epoch，也就是 10 小时 40 分钟的预算等待。这只估算排队部分；最终性、处理细节和后续步骤还会增加时间。

### 2.5 还存在另一种流量控制

Churn 保护的是 stake 集合变更速度；请求数量限制保护的是状态处理资源。两者不能互相替代。

例如 EIP-7002 的执行层提款请求采用队列和动态费用，目标速率为每块 2 个请求，单块最多出队 16 个。这是请求管道的速率，不代表每块允许退出 16 个 validator 的全部资金，也不代表 2 个请求是不可突破的硬上限。最终减少有效质押仍受共识层规则约束。[EIP-7002 请求队列和费用机制](https://eips.ethereum.org/EIPS/eip-7002)

这个区分对我们的合约很有用：同时设计“每 epoch 最多变化多少 stake”和“每块最多处理多少注册／退出对象”，才不会被大量小请求压垮。

## 3 以太坊如何退出并保留追责能力

### 3.1 退出和提款是连续的不同阶段

| 阶段 | 共识职责 | 资金状态 |
| --- | --- | --- |
| 提交退出请求 | 通常仍需履行职责 | 仍锁定 |
| 排队等待 exit epoch | 继续按当前规则参与 | 仍锁定并承担责任 |
| 到达 exit epoch | 不再是活跃 validator | 尚未立即转账 |
| 到达 withdrawable epoch | 已停止活跃职责 | 获得提款资格 |
| 提款处理到该 validator | 无活跃职责 | 资产转入提款地址 |

正常退出通常在 `exit_epoch` 之后再等待 256 个 epoch 才达到 `withdrawable_epoch`，约 27 小时 18 分钟；之后还要等待提款处理。**27.3 小时不是从点击退出到资金到账的总时间。** 请求队列可以很长。[以太坊提款说明](https://ethereum.org/staking/withdrawals/)、[提款延迟配置](https://raw.githubusercontent.com/ethereum/consensus-specs/master/configs/mainnet.yaml)

### 3.2 谁可以发起退出

传统路径由 validator 签名密钥发出 voluntary exit。规则还检查最低活跃时长等条件，其中 `SHARD_COMMITTEE_PERIOD` 为 256 个 epoch。这个“至少先工作多久”和“退出后再等多久”是两个不同限制，即使当前数值相同，也不能合并。[Electra voluntary exit](https://ethereum.github.io/consensus-specs/specs/electra/beacon-chain/#modified-process_voluntary_exit)

EIP-7002 增加提款凭证对应的执行层地址触发退出／提款请求的路径，缓解资金所有者受制于运行节点的服务商的问题。该路径仍需满足共识层资格和队列检查；它不会绕过退出节奏。资金控制与节点运行可以因此更明确地分离。[EIP-7002](https://eips.ethereum.org/EIPS/eip-7002)

还要区分全额退出、奖励余额的自动提款，以及 compounding validator 的部分提款。部分提款可以减少资金而保留 validator 身份；因此我们的设计若允许减仓，也必须把减仓视为权重流出，不能只管“删除节点”的请求。[EIP-7251](https://eips.ethereum.org/EIPS/eip-7251)

### 3.3 普通离线和 Slashing

短时离线一般对应未获得奖励或受到参与惩罚。Slashing 针对协议明确规定的可验证冲突，例如同 slot 双重提议、冲突证明中的 double vote 或 surround vote。以太坊还对大规模相关罚没施加相关性惩罚；大量节点同时违反安全规则，与单个节点偶发失误具有不同经济后果。[奖励和惩罚说明](https://ethereum.org/developers/docs/consensus-mechanisms/pos/rewards-and-penalties/)

被 slash 后，提款时间至少推迟到 slashing epoch 加 8,192 个 epoch，约 36.4 天，并与已经排定的提款时间取较晚者。这是为了完成罚没结算，不应误认为普通退出也统一等待 36 天。[Electra slash_validator](https://ethereum.github.io/consensus-specs/specs/electra/beacon-chain/#modified-slash_validator)、[slashings vector 预设](https://raw.githubusercontent.com/ethereum/consensus-specs/master/presets/mainnet/phase0.yaml)

对我们而言，“某 region 没响应，所以换一个”可以是服务恢复决策；“其中某人做了足以罚没的事”需要另外的责任定义和证据。当前 v3 已经作了这个区分：双签使用具体签名证据，状态委员会的可达性观察不自动等价于作恶证明。

## 4 这些安排如何帮助安全

### 4.1 它们分别解决什么问题

| 机制 | 解决的主要问题 | 仍需什么条件 |
| --- | --- | --- |
| 最终化的请求顺序 | 节点对已受理操作达成一致 | L1 的共同历史和最终性 |
| 激活延迟 | 给状态同步和职责准备留时间 | 节点真正完成准备 |
| Churn limit | 限制短期成员／权重变化 | 新配置仍满足诚实 stake 假设 |
| 退出后锁定 | 防止做完坏事立刻转走保证金 | 证据能及时发现、提交和执行 |
| 历史配置 | 按行为发生时的身份和权重验签 | 历史资料可获得 |
| Slashing | 为定义好的违规行为提供经济后果 | 裁决正确、抵押品可罚且有价值 |

这张表是对机制作用的归纳。它说明为什么“加上质押”本身并没有完成共识安全设计：押金不能替代唯一历史的形成规则，等待时间也不能自动制造诚实多数。

### 4.2 Churn 不会阻止所有诚实节点离开

假设某 region 原来总权重为 100，恶意方有 25。若 30 单位诚实 stake 最终退出，恶意方仍持有 25，但占比变为 $25/70\approx35.7\%$。全过程可以严格遵守退出速率限制，最终仍超过三分之一。

因此，churn 提供的是缓冲时间，不是对无限时间范围内攻击比例的保证。协议也无法仅凭地址和 stake 精确识别哪部分是恶意的。最低总 stake、经营主体分散度、公开监控能管理风险，但安全性仍须明确假设或另行分析。

### 4.3 历史密钥带来长期问题

保证金全部释放之后，旧密钥仍可能签名。一个长期离线或首次加入的节点不能仅凭“这批旧签名看起来有效”识别可信历史。以太坊使用 weak subjectivity checkpoint，使同步节点从可信的近期状态出发验证后续历史。[Weak subjectivity](https://ethereum.org/developers/docs/consensus-mechanisms/pos/weak-subjectivity/)

我们的 L1 registry 可以为配置历史和已接受交接提供锚点，但前提是客户端正确验证 L1，并且锚点包含所需的权限和历史承接信息。**只把名单哈希放上 L1，不会自动证明旧 session 已经没有更高的有效输出。** 这是 v3 的 closed-cut／fencing 条件仍然重要的原因。

### 4.4 以太坊排班不等于每个节点各报一个 VRF

以太坊利用 RANDAO 随机性与共识状态确定 proposer，而不是让所有 validator 各自提交一个 VRF 数值，再直接选最小值。Fulu 的 EIP-7917 把未来 proposer lookahead 预计算并存入状态，减少余额变化导致未来名单改变的空间。[EIP-7917](https://eips.ethereum.org/EIPS/eip-7917)

这对我们有直接启发：共同 stake 快照和共同随机输入都确定后，应当一次性确定排班；不能允许看到抽签结果后再通过加仓、换公钥或移动 region 重算当前名单。

RANDAO 也不是完全不可操纵的理想随机源。出块者有放弃出块等影响手段，EIP-4399 因此建议应用使用事先指定的未来随机性，并留足 lookahead。未来随机性在条件合适时降低预测和偏置风险，单纯对可偏置输入再做一次哈希不会消除偏置。[EIP-4399 安全分析](https://eips.ethereum.org/EIPS/eip-4399#security-considerations)

## 5 Staking service 如何在以太坊之上管理节点

以 Lido 为例，用户存入资产、operator 进入服务集合、某个 validator 在以太坊激活，是三层流程。StakingRouter 把 stake 分配给不同模块；每个模块维护自己的 operator 和 validator 集合。模块本身的注册由治理决定，不能因为平台接受任何人的存款，就推导出所有节点准入也无需许可。[Lido StakingRouter](https://docs.lido.fi/contracts/staking-router/)

当前 Lido 文档区分 curated module 与 permissionless Community Staking Module。CSM 使用 operator bond 等机制约束运营责任；文档所列 `0x01` CSM 采用带优先席位的 FIFO 队列，`0x02` CSM 区分初始入场和后续 top-up 队列，且页面分别标注主网／测试网状态。[Lido Staking Modules](https://docs.lido.fi/staking-modules/)

用户赎回 stETH 又使用另一条 FIFO withdrawal queue：请求产生领取权，协议在满足结算条件后为它预留 ETH，用户再 claim。它与底层 validator exit queue 有联系，但不是同一个队列；缓冲资金或其他可用流动性可以影响用户实际等待时间。[Lido WithdrawalQueueERC721](https://docs.lido.fi/contracts/withdrawal-queue-erc721/)

可借鉴之处是把资金分配政策和共识成员政策拆开。例如，将来允许外部用户委托给我们的 operator，并不意味着每一笔小额委托都要生成一个新的共识节点。资金进入可以细粒度处理，节点数量与密钥集合则由独立规则控制。

## 6 EigenLayer 如何管理加入退出和责任

### 6.1 四层关系

可以沿着下面的关系理解 EigenLayer：

**Restaker 提供资产 → 委托给 operator → operator 加入某个 operator set 并分配 stake → AVS 定义具体工作与罚没条件。**

Operator set 可以按硬件要求、工作类型、活性要求或抵押品组成划分。同一个 AVS 可以有多个 set。这与我们的按 region 划分服务集合相似，但 region 的共识、地理含义和轮换仍需由 AVS 自己定义。[Operator Sets Overview](https://docs.eigencloud.xyz/eigenlayer/concepts/operator-sets/operator-sets-concept)

### 6.2 注册不等于有资金为服务负责

在 Unique Stake 模型下，operator 将委托资金的一部分分配给具体 operator set。只有符合规则的分配份额才构成该 set 的可罚没担保。某 operator 共被委托 1,000 单位资产，若仅有 200 单位对应的份额有效分配给我们的 set，我们就不能按 1,000 单位计算本服务的经济担保。

这些 unique allocations 在 operator set 间具有排他性，避免把同一份分配金额重复算成多个 set 各自独占的抵押。注册了 set 但未完成有效 allocation 的 operator，也不能仅凭注册记录被当成有足额抵押的成员。[Unique Stake](https://docs.eigencloud.xyz/eigenlayer/concepts/slashing/unique-stake)

这里的隔离针对特定 allocation。底层资产价值变化、原生 ETH 自身的 beacon slashing、实际委托量变化等因素仍要按所用资产和合约记账处理；“unique”不代表资产价值永不下降。

### 6.3 三种离开动作不能混用

| 动作 | 改变什么 | 不应推导出的结论 |
| --- | --- | --- |
| Operator deregistration | 不再注册于该 operator set | 所有 allocation 已经自动清零 |
| Deallocation | 减少分配给某 set 的可罚没份额 | 发交易后该份额立即无责任 |
| Staker withdrawal／undelegation | 资金提供者减少委托或取回资产 | Operator 的所有其他服务状态同步结束 |

官方文档明确说明，deregistration 后仍有延迟责任窗口；deregistration 本身不等于排队 deallocation。如果保留 allocation 后重新注册，相关份额可以重新立即进入可罚没状态。对仍可罚的 allocation，deallocation 经过延迟后才生效；对本来已经不可罚的份额存在即时解除的例外。[Allocation and Deallocation](https://docs.eigencloud.xyz/eigenlayer/concepts/operator-sets/allocation-deallocation)

### 6.4 当前文档中的安全延迟

| 参数 | 作用 | 官方参考值或设定方式 |
| --- | --- | --- |
| `ALLOCATION_DELAY` | 新 allocation 到实际生效的等待 | Operator 自己设置，协议未统一规定正的最小值 |
| `ALLOCATION_CONFIGURATION_DELAY` | 修改上述延迟到修改生效 | 126,000 blocks，约 17.5 天 |
| `DEALLOCATION_DELAY` | 解除可罚 allocation 或注销后的责任等待 | 100,800 blocks，约 14 天 |
| `WITHDRAWAL_DELAY` | Staker 的排队提款到可完成 | 100,800 blocks，约 14 天 |
| `SLASH_RESOLUTION_DELAY_BLOCKS` | 已被 slash 的 shares 到可清算销毁／重分配 | 50,400 blocks，约 7 天 |

以上是 2026 年 10 月 10 日读取的官方 reference 表，天数按区块时间近似换算，不是对任意部署的不可变承诺。最后一行是罚没资金的处理延迟，不是给用户额外定义的一条普通退出队列。也不能把不同角色的 14 天延迟无条件相加，得到固定 28 天退出时间。[Safety Delays Reference](https://docs.eigencloud.xyz/eigenlayer/reference/safety-delays-reference)

这些延迟让 AVS 有机会更新它的安全视图，并让旧任务在适当窗口内继续可罚。关键是从服务的任务生命周期反推时间：停止接新任务之后，最晚一项旧任务何时可能产生或暴露违规，证据何时能进入链上，裁决何时完成。[Safety Delays](https://docs.eigencloud.xyz/eigenlayer/concepts/slashing/safety-delays-concept)

### 6.5 对我们的映射

我们可以把一个 region 映射为一个 operator set，或者由一个 set 提供全网资金担保，再由 middleware 分配区域职责。两种方案需要不同的风险计算：前者容易隔离区域责任，后者必须解释各 region 是否共享同一可罚资金池。

无论选哪种，读取平台 TVL 或 operator 总委托量都不够。我们的 snapshot 必须读到**本服务、本 region、本责任窗口内实际可使用且可罚的金额**。平台提供 allocation／slashing 基础设施；安全快照、leader 排班、数据保留、交接与恢复仍是我们的协议职责。

## 7 Symbiotic 的 Epoch Vault 和新架构

### 7.1 先区分两组官方文档

当前官方文档中，`learn/core-concepts` 保留 Network、Vault、Delegator、Slasher 以及 stake capture 的机制说明；新的 `get-started` 入口以 Core V2、ERC-4626 vault、Universal Delegator 和 App Adapter 为中心。下文分开解释这两组接口，不能把传统 vault 的等待公式直接当作所有 V2 adapter 的默认行为。[传统 Vault 机制](https://docs.symbiotic.fi/learn/core-concepts/vault)、[Core V2 入口](https://docs.symbiotic.fi/get-started)

### 7.2 传统机制中的两个 Epoch

**Network epoch** 是应用使用某个 operator set 和 stake snapshot 执行工作的时期。**Vault epoch** 是资金退出和历史可罚窗口的时间尺度。它们不需要同长，vault epoch 通常要为 network epoch 结束后的证据与裁决流程留下余量。[Network](https://docs.symbiotic.fi/learn/core-concepts/network)

传统 vault 的提款分为 request 和 claim。在 vault epoch $k$ 期间提出请求，直到 $k+1$ 结束之后才能领取。若每个 epoch 长 $E$，等待时间约在一个至两个 epoch 之间，精确端点取决于时间边界约定。例如 epoch 为 7 天，周一零点开始，在周三零点申请，则要到再下一个周一零点，约等 12 天。期间资金仍可承担适用的罚没责任。[Epochs and Delays](https://docs.symbiotic.fi/learn/mechanics/epochs-and-delays)

Stake capture 是另外一个重要概念：网络在指定时间取得 stake 快照，后续验证和处罚引用这个时间，而不是任意使用今天的余额。可罚窗口会过期，而且以前的罚没会影响剩余担保。因此计算可用担保时还要考虑 cross-slashing，不能简单把所有 operator 报出的 stake 相加。[Network 的 stake capture 与 slashing](https://docs.symbiotic.fi/learn/core-concepts/network)

传统 Slasher 处理 middleware 发出的罚没请求；VetoSlasher 额外给予配置的 resolver 否决窗口。它们负责执行相应权限和金额／时间约束，但不会凭空判断我们的 mini-block 共识是否遭到攻击。具体违规判定仍来自网络自己的规则。[Vault 的 Slasher 类型](https://docs.symbiotic.fi/learn/core-concepts/vault#slasher-types)

### 7.3 Core V2 的资金路径

V2 vault 接受一种抵押资产，发行代表份额的 ERC-4626 shares。资金可能处于 idle 状态，也可能通过 Universal Delegator 分配给 adapters。提款先使用可用 idle 资金，不足部分再按照 adapter 的退出机制取回。[V2 Vaults](https://docs.symbiotic.fi/get-started/what/components/vaults)

Universal Delegator 可以同时限制每个 adapter 的绝对分配额度和占 vault 的比例，并协调即时、部分或异步提款。因而用户界面的赎回速度与应用侧仍需保留的担保期限可以不同：即时流动性可以来自尚未被占用的资金，而不是把正在承担责任的抵押直接抽走。[Universal Delegator](https://docs.symbiotic.fi/get-started/what/components/universal-delegator)

应用通过 middleware 和 AppAdapter 接入担保。Middleware 定义违规、权限、金额和必要的争议流程，AppAdapter 执行对可用抵押的罚没。当前集成文档还包含 AppAdapter 被加入允许列表的步骤，因此具体部署的 permissionless 程度需要按角色和接口分别判断。[Build an Application](https://docs.symbiotic.fi/integrate/applications/deploy-an-application)

### 7.4 对我们最有价值的时间关系

对一个引用历史 capture 的设计，保守的时间预算应满足

$$
T_{\mathrm{capture\to active}}
+T_{\mathrm{service}}
+T_{\mathrm{detect}}
+T_{\mathrm{dispute}}
+T_{\mathrm{L1\ execute}}
+\varepsilon
<T_{\mathrm{guarantee}}.
$$

这是我们的设计约束表达，不是所有 Symbiotic 合约共同采用的公式。它强调：应从捕获担保开始计时，把服务准备、服务运行、发现违规、争议和链上执行全部算进去。只比较“30 分钟 epoch”和“14 天提款延迟”容易漏掉最后一项任务及边界情况。

如果采用普通 staking 合约，可以由我们定义这些窗口；如果采用 restaking 平台，则我们的流程必须落在平台真实提供的保证期内。只要平台可以先释放资金，我们自己的文档再写一句“退出后仍可罚”也不能延长平台合约的权限。

## 8 当前 v3 已有的接口和待补部分

以下依据本地 [main_v3.tex](../../formalization/main_v3.tex) 的当前内容，而不是此前某个版本。

| v3 的位置 | 已有规则 | 本次需要补充的对象 |
| --- | --- | --- |
| Model and Service Definitions | L1 提供 `cfg`，包含成员、公钥、权重和名单 | 如何从申请队列生成 `cfg` |
| Regional Proof-of-Stake SMR | 同一 session 内成员和权重固定 | 新 snapshot 的安全激活协议 |
| Regional Status Committee | 预先公告未来 L1 高度生效的 `live/off` map | 与 epoch 配置变更的事件顺序 |
| Cross-Region Rotation | 固定循环名单，`NextLive` 跳过 off region | 每个 epoch 的名单和初始游标如何生成 |
| Slashing | 按历史域验证双签，违规 ID 去重 | 可罚资金的保留期与资金释放条件 |
| Recovery assumption | 保留完整已确认前缀，封闭旧权限，再启动新 session | 将该接口落实到跨配置交接 |

v3 还明确区分三种区域证书：VOTE 的权重为 $>2W_r/3$，BLAME 和提交后的 CONFIRM 回执为 $>W_r/3$。**以下框架不把这些阈值改写成另一套 BFT 协议。** 动态配置的职责是明确每份证书使用哪一套历史成员和分母，以及什么时候换套。

另一个关键点是：v3 每个身份在循环 producer list 中出现一次，并没有声称生产机会按 stake 比例分配。若新版本需要 stake-proportional leadership，必须明确改变这一条排班语义。

## 9 建议的成员和资金管理框架

本节起均为本项目的候选设计。建议先采用单一抵押品、固定 region 标识集合、显式配置边界；跨抵押品估值、创建新 region 和任意跨区流动可以后续再扩展。

### 9.1 给 stake 加上 Epoch 下标

沿用你的 $s_{r,j}$，写成

$$
s_{r,j}^{(e)}
=\text{第 }e\text{ 个配置中，region }r\text{ 的成员 }j
\text{实际分配且已生效的质押数量},
\qquad
W_r^{(e)}=\sum_{j\in V_r^{(e)}}s_{r,j}^{(e)}.
$$

这里保留你用 $s$ 表示 stake 的含义。第一版采用单一抵押品、线性权重，令 v3 的 $\omega_{r,j}^{(e)}=s_{r,j}^{(e)}$，所以下文可以直接用 $s$ 计算投票和抽样权重。若以后加入多资产折价或权重上限，则另定义 $\omega=f(s)$，并在所有 quorum 和抽样公式中统一使用 $\omega$。

为免账户余额和已生效 stake 混淆，对稳定 operator ID $o$，另记 $b_o(t)$ 为抵押账户余额，$a_{r,o}(t)$ 为本服务在该 region 的有效担保分配。用 $o=\operatorname{op}(e,r,j)$ 将配置内成员映射到稳定身份。$s_{r,j}^{(e)}$ 由该身份已确认的分配、排队规则和生效条件生成，不直接等于任意时刻的 $b_o(t)$。

最低限度应记录：稳定 operator ID、共识公钥、提款权限、region、资金余额、服务分配、入队序号、预计激活／退出配置、责任到期记录。成员索引 $j$ 仅供该配置内部使用；罚没与历史身份不能依赖“现在列表中的第几个”。

若我们自己的合约把同一资产份额独占分配给不同 region，可以强制

$$
\sum_r a_{r,o}(t)\le b_o(t).
$$

如果一个运营者可以在多个 region 提供服务，应按份额分别记账，而不是把完整余额在每个 region 重复计算。仍在承担旧 region 历史责任的份额应继续占用相应分配预算，不能同时被说成新 region 独占的保证金。Restaking 集成还必须按平台实际的剩余可罚 shares／金额修正这一关系。

### 9.2 节点和资金采用两条状态线

节点资格可以采用：

```text
Registered → Pending → Active → ExitScheduled → Retired
                         ↓
                  违规或故障处置
```

资金状态可以采用：

```text
Deposited → Allocated → Supporting duties → Historical liability → Claimable
                                              ↓
                                            Slashed
```

这里 `Retired` 只表示不再承担新的共识职责。数据保留、协助旧任务恢复和已产生的罚没责任，可以持续到另外的期限。

新节点在 pending 期间同步数据、验证 checkpoint、建立连接。Readiness 检查应有公开条件和可追踪的失败原因，避免由现有成员无期限拒绝新成员。若需要第三方认证“机器确实在某地”或“满足某个时延”，那是额外准入假设，必须单独承认。

### 9.3 L1 队列的顺序和生效规则

建议 L1 接受以下操作：注册、增加质押、申请减少分配、申请退出、变更公钥、申请迁移 region。每项操作取得单调递增的 request ID；以最终化链上的执行顺序为准。未完成最终性的操作不进入下一配置候选集。

正常加入可以用 FIFO 加公开资格检查。尚未完成 readiness 的请求可以按明确规则推迟，防止一个无效申请永久堵住所有后续人；大额有效申请则应拆为有序 tranche，或采用公开的预算累积规则，不能由管理员临时决定谁插队。

节点迁移应被视作“源 region 的减配＋目标 region 的增配”。两侧各自消耗流出／流入预算，并记录历史责任。公钥变更也要经过配置边界；旧公钥的证据验证记录继续保留。

退出申请一经进入生效日程，不能因为抽签结果不满意而取消当前 epoch 的职责。允许取消 pending 申请的范围应在随机性揭晓前确定。

### 9.4 全局和区域两级 Churn

至少设置以下四种预算：

$$
Q_{\mathrm{in}}^{\mathrm{global}}(e),\quad
Q_{\mathrm{out}}^{\mathrm{global}}(e),\quad
Q_{\mathrm{in}}^r(e),\quad
Q_{\mathrm{out}}^r(e).
$$

全局预算约束系统总体迁移；区域预算防止全部变更集中冲击某一个 region。增加和减少必须分别计量，不能只限制净变化。100 单位出去、100 单位进来，净变化是零，但可能已经替换了全部成员。

除了金额预算，还需要每 epoch 新公钥数、最大活跃成员数或抽样委员会规模、同步带宽及合约处理数量的限制。v3 的区域广播成本随成员数量增长；一个人拆出很多小 stake 身份可能没有增加总投票权，却显著增加通信量。

如果限制最大活跃成员数，还必须规定席位如何到期、重新抽取或公平让候补者进入。先到者永久占满席位，会使“注册开放”无法转化为实际参与机会。对于只按 FIFO 等待空位的方案，入场活性应明确以有空位或现有成员退出为条件。

这些限制解决两种不同的压力：

| 控制项 | 主要保护对象 |
| --- | --- |
| stake 流入／流出上限 | 权重结构的变化速度 |
| 新身份数和请求数上限 | 合约、验签和索引处理资源 |
| batch 字节数与节点带宽要求 | v3 的完整消息时延假设 |
| 同步并发和状态大小限制 | 新成员准备时间 |
| 退出后数据保留要求 | 旧输出的恢复与追责 |

不建议直接写死“每 30 分钟允许 1% 退出”就宣布安全。同一个比例在 6.4 分钟、30 分钟和一天的 epoch 下意味着不同的日变动速度；持续多轮诚实退出也会改变攻击占比。

### 9.5 退出保证金的释放

时间锁（timelock）至少有四种用途：加入前的准备期、激活后的最短服务期、退出排队后的生效等待、停止职责后的追责保留期。修改治理参数还可能另有公告延迟。这些时钟保护的对象不同，应分别设置；统一写成“等待两个 epoch”容易让加入准备与历史追责共用一个不合适的参数。

可以从以下时间关系开始定义：

$$
t_{\mathrm{release},o}\ge
t_{\mathrm{lastDuty},o}
+T_{\mathrm{detect}}
+T_{\mathrm{challenge}}
+T_{\mathrm{execute}}
+\varepsilon.
$$

其中 $t_{\mathrm{lastDuty},o}$ 指该身份最后一项被协议承认的义务结束时间，可能包含约定的数据保留期。实际减仓可以按责任份额分别记录到期时间。若在截止前已经受理争议，应按预先公布的规则冻结相应金额到裁决结束。若允许无期限新增争议，则又不能承诺有界退出；两者要一起设计。

罚没真实发生后，不应继续宣称原金额仍完整可罚。可以保持当前 session 的历史投票分母不变，同时记录担保下降，并触发既定暂停／安全重配置路径。若选 restaking，还要接收外部罚没造成的余额变化，不能只处理我们主动发起的罚没。

最低 stake 不足时，也不宜靠永久扣住最后几个诚实节点来维持 region。更合理的方向是：按预定规则让 region 进入停止服务和退出流程；无法满足安全条件时接受暂停。持续运行与有界退出的联合保证需要明确的网络、替补成员和恢复假设。

## 10 Epoch 快照与随机排班

### 10.1 四个边界及其顺序

建议对候选配置 $e$ 定义四个有顺序的事件：

$$
\operatorname{Freeze}_e
\prec\operatorname{Randomness}_e
\prec\operatorname{Announce}_e
\prec\operatorname{Activate}_e.
$$

1. **Freeze**：从已最终化的 L1 前缀生成确定的成员、权重和资格快照，固定其哈希。
2. **Randomness**：取得预先约定的未来随机输入／VRF 输出；候选人此时已经无法修改本次权重和密钥。
3. **Announce**：任何人都能由快照和随机值复算名单，公布其承诺及未来生效边界。
4. **Activate**：完成旧配置安全关闭、数据传递与新成员准备后，新 session 才取得生产权。

30 分钟可以是期望的配置更新节奏，但这四步不必挤在同一 30 分钟内，可以提前多个 epoch 流水化准备。若随机源、L1 最终性或安全交接延迟，应明确推迟激活或暂停，不能由节点自己的墙钟到点就各自换表。

建议用 L1 timestamp／slot 规则定义名义 epoch，再用最终化控制记录定义实际配置激活。L1 block height 仍可作为与 v3 对接的生效标记，但“150 个区块约 30 分钟”仅是近似，不能在证明中当作精确时间等式。

### 10.2 VRF 需要配套的随机源协议

一个 VRF 可以抽象成

$$
(\rho_e,\pi_e)=\operatorname{VRF}_{sk}(x_e),
\qquad
\operatorname{Verify}(pk,x_e,\rho_e,\pi_e)=1.
$$

固定输入和密钥后，输出可以被验证，合适构造具有相应唯一性／伪随机性性质。但“谁持有密钥、何时固定输入、拒绝公布怎么办”仍是协议层问题。[RFC 9381](https://datatracker.ietf.org/doc/html/rfc9381)

建议 $x_e$ 绑定 chain、registry、协议版本、epoch、冻结快照哈希和预定随机源标识。密钥也要提前注册或通过独立的委员会密钥生成流程确定，避免看到输入后再挑选有利密钥。

三种实现方向的区别如下：

| 随机源 | 优点 | 需要补的条件 |
| --- | --- | --- |
| 预先指定的未来 L1 随机性 | 复用现有链，公开可复算 | 偏置分析、最终性、精确取样规则 |
| 独立服务提供 VRF | 输出易验证 | 拒绝发布、服务依赖、备用路径 |
| Threshold VRF／随机信标 | 将发布能力分散给多人 | 密钥生成、门槛、成员更新与终止条件 |

不要让状态多签在多个“都有效”的随机结果中选一个。即使每个结果都有证明，允许挑结果仍会形成选择偏置。超时后重抽或切备用源也可能给持钥人“公布／不公布”的选择权；第一版可以在随机性缺失时暂停新配置激活，再对备用方案单独分析。

### 10.3 Region 顺序如何生成

与 v3 改动最小的方案，是对固定且具备基本配置资格的 region ID 集合 $\mathcal R_e$ 生成均匀随机排列：

$$
\mathcal L_R^{(e)}
=\operatorname{Shuffle}\bigl(
\mathcal R_e,H(\texttt{REGION},e,\rho_e)\bigr).
$$

Shuffle 需要指定确定性编码、无偏整数采样和算法版本。这个排列包含暂时 `off` 但仍属于配置的 region；在实际选择时才用已生效状态 map 跳过 off 项，避免每次状态变化都重新抽签。

配置启动时，从列表预定起点选择第一个 live region；之后按现有 `NextLive` 循环后继规则运行。若配置启动恰好选到原 region，也必须启动新的配置 session；它是配置续期，不计作跨 region 轮换。若只有一个 live region，是否允许 quota 到期后同区续期必须另定规则；当前 v3 的 `NextLive` 找不到“其他”region 会等待。

**随机排列只改变顺序，不会自动带来 stake 比例的服务频率。** 若每圈各 region 只出现一次且每次 quota 相同，它们获得的是每圈相同的生产轮次。每个 epoch 可能跑不完一圈，因此有限窗口还有截断效应。

我建议第一版先采用 region 均匀排列，把各区最低担保和内部诚实比例单独处理。这样不会把“stake 多的 region 应当获得更多生产时间”这个尚未决定的经济目标隐含进协议。如果确实需要按 $W_r^{(e)}$ 分配频率，应改成允许重复 region 的抽样机会序列或按权重分配 quota，再重新定义同区连续当选与故障跳过。

### 10.4 区域内部 Leader 名单的两个选项

**选项 A 保留 v3 的每身份一轮。** 对 $V_r^{(e)}$ 作独立随机排列，设

$$
\mathcal L_r^{(e)}
=\operatorname{Shuffle}\bigl(V_r^{(e)},
H(\texttt{LEADER},e,r,\rho_e)\bigr),
\qquad
\ell_r^{(e)}(v)=\mathcal L_r^{(e)}[v\bmod |V_r^{(e)}|].
$$

这最容易接入当前局部 view-change，但生产频率按身份分配，存在拆分身份获取更多机会的问题。最低质押提高拆分成本，却不会完全消除这种激励。

**选项 B 按 stake 分配生产机会。** 若你的目标包括经济上的 stake proportionality，则对每个机会 $q$ 从 $[0,W_r^{(e)})$ 无偏生成整数 $u_{e,r,q}$，按固定 ID 顺序排列的 stake 前缀区间选择 leader：

$$
\ell_r^{(e)}(q)=j
\quad\Longleftrightarrow\quad
\sum_{k<j}s_{r,k}^{(e)}\le u_{e,r,q}
<\sum_{k\le j}s_{r,k}^{(e)}.
$$

资金以最小整数单位计量。伪随机输入绑定 epoch、region、机会编号、冻结快照哈希和 $\rho_e$；同一 opportunity 重放得到同一结果。可以预计算有限长度的名单，或按需展开已承诺种子对应的无限序列。

在理想均匀抽样模型下，单次分配给 $j$ 的概率为 $s_{r,j}^{(e)}/W_r^{(e)}$。这是被分配机会的概率，**不是故障、跳过或不同任期长度之后实际成功生产比例的无条件保证**。同一控制者拆分固定总 stake，也不会增加这类线性抽样下的总当选概率。

有放回抽样可能连续选中同一人，甚至长时间遇到故障 leader。若需要确定的恢复步数，可以为每次主机会生成包含全部候选人的不重复备用排列，并保留 v3 的 view-change／继承状态流程；这会改变原 leader 选择函数和相应延迟分析，需要单独形式化。

还有一个常见误区：按 $-\log U_j/s_j$ 排序得到加权随机排列，可以让第一名按 stake 抽取；但如果每圈还是每人只服务一次，不能因此宣称长期服务次数按 stake 比例增长。

我的取舍是：**第一步用选项 A 补齐快照、激活和退出责任，明确只随机化顺序；如果生产收益也要与 stake 对齐，再采用选项 B 并同步修改 view-change 分析。** 两步的安全接口可以共享。

### 10.5 随机排班不负责创建安全的 Region

本项目仍要求对每个实际服务 region、每个激活配置满足

$$
\sum_{j\in\mathcal B_r^{(e)}}s_{r,j}^{(e)}
<\frac13W_r^{(e)},
$$

其中 $\mathcal B_r^{(e)}$ 是攻击者控制的成员集合。全网恶意比例低于三分之一，不推出每个 region 都如此；如果攻击者可以自由选择加入哪个 region，它可能集中到最弱的一组。

随机打乱 leader 顺序不会改变上述成员构成。要控制区域风险，可以研究随机分配委员会、限制跨区迁移、要求区域最低担保和运营分散度等方向，但地理时延约束可能与随机分配冲突。按地址设置单人上限也容易被 Sybil 拆分绕过。

因此第一版应坦率保留“每区恶意 stake 小于三分之一及区域同步条件”的假设，并把 permissionless region admission 如何维持该假设列为独立问题，不能用 VRF 代替这一分析。

## 11 多签状态和安全配置交接

### 11.1 多签适合管理可选资格

建议继续采用 v3 的职责：多签签署具有版本和未来生效高度的 region status map；它可以让服务在故障时跳过某区，让恢复区重新获得后续选择资格。

正常名单由快照和随机性公开确定。多签不应因此获得改 stake、选随机种子、改历史 checkpoint 或任意指定下一 leader 的权力。若未来确实要加入紧急 override，应把它作为单独治理权限，规定公告、生效和可验证的边界。

即使只允许 `live/off`，多签仍有实际排班影响力：当名单是 $A,B,C$ 时，把 $B$ 标 off 会使下一次选择到 $C$。所以这里可以主张“原始名单可验证”，但不能在委员会可任意删候选人的模型下主张“实际服务分配完全无偏”。

恢复上线路径建议有观察期和状态同步条件，退出可选资格的路径要满足恢复需求。持续抖动可以用最短状态保持时间和更新频率约束控制。超时本身适合触发故障处理；只有定义了可裁决义务、证据和申诉流程，才适合进一步产生罚没。

普通 $t$-of-$n$ 多签也不自动满足 v3 的委员会安全模型。若要靠签名集合交集排除两个冲突证书，在最多 $f$ 个恶意签名者时至少需相应的交集条件，例如等权情形 $2t-n>f$，并要求诚实方持久记录已签版本。L1 合约的连续版本和前驱检查可以确定唯一安装历史，但不能保证多签提交的“某区离线”判断真实。

### 11.2 不能在 Epoch 到点时原地替换分母

假设旧配置总权重为 100，新配置总权重为 80。一个权重 60 的签名集合，在旧配置下未达到 $>2W/3$，在新配置下可能达到。验证旧消息时改用新分母，会改变同一证书的有效性。

因此所有消息绑定 `(chain, registry, cfg, region, session)` 等域；历史消息始终按历史配置验签。增加 stake、退出、罚没和临时失联，都不让接收者自行替换当前 session 的分母。成员权重的更新必须产生新的已授权配置／session。

仅限制每次替换比例也不够。新旧 quorum 的交集需要与锁定状态、已确认前缀和交接证据共同论证，不能从“每次只换 10%”直接推出跨配置安全。

### 11.3 建议把配置激活写成显式控制事件

候选配置可先组织为

$$
\mathrm{Cfg}_e=
\bigl(e,\mathrm{prevCfg},\mathrm{stakeRoot}_e,
\mathrm{keysRoot}_e,\rho_e,\mathrm{scheduleRoot}_e,
\mathrm{earliestActivation}_e,\mathrm{params}_e\bigr).
$$

它描述“准备启用什么”，但仅有这个对象不足以开始生产。另设交接记录

$$
\mathrm{Activate}_e=
\bigl(\mathrm{Cfg}_e,\mathrm{oldSession},
\beta_e,A_{\beta_e},\mathrm{closeWitness},\mathrm{readyWitness}\bigr).
$$

交接最少要回答五件事：

1. **旧前缀保全**：所有必须保留的旧确认和可形成的有效最终回执都被 $A_{\beta_e}$ 覆盖。
2. **旧权限封闭**：旧 session 不能在此后生成可被接受的另一条延伸；不能仅靠“现在时间晚于生效点”判断旧签名无效。
3. **新成员准备**：新集合已获取所需数据、配置和 checkpoint，能够承担约定职责。
4. **唯一激活**：每个前驱只产生一个被接受的新配置和启动锚点，所有验证者按同一控制顺序解释它。
5. **历史责任延续**：退役成员对应的抵押和证据查询接口继续保留到责任结束。

这里的 witness 是需要定义内容和验证算法的接口，不能只给它一个名字就当作实现完成。尤其旧 region 已失联时，多签一句“它 off 了”无法证明没有隐藏的更高确认前缀。v3 已把这部分列为恢复前提；新增 epoch 机制仍须解决它。

### 11.4 三种边界同一时刻发生

可能同时发生配置激活、status map 生效和当前 term quota 耗尽。建议引入统一控制事件序号，并为同一名义边界规定固定处理顺序。一个候选顺序是：

**应用已到期状态更新 → 在当前授权上登记应处理的配置变更／强制交接 → 完成一次统一的关闭和后继选择 → 若仍有必要，再处理 quota 导致的正常轮换。**

它需要保证同一旧 session 只关闭一次；已经 sealed 的后继不会因新名单再次被改选。新配置最终选中的 region 若被有效状态标 off，就按新名单跳过；没有合格 region 则等待。

如果只完成了候选配置的公告而未完成安全激活，旧消息仍按旧 cfg 验证。是否允许旧配置继续接新工作，必须服从预先设定的退休／关闭协议和保证金责任期限，不能无期限自动延期。

## 12 一个完整例子和下一步形式化对象

### 12.1 加入退出与排班的连续例子

以下仅为教学例子，不是建议的生产参数。

系统有三个固定 region：$A,B,C$。配置 $e$ 中，$A$ 的三个 operator 有权重 $(50,30,20)$，所以 $W_A^{(e)}=100$。当前所有旧消息都使用这一分母。

在服务期间，新节点 $A_4$ 申请加入 10 单位；$A_2$ 申请减少 10 单位。L1 确认请求后，它们进入相应队列；只有在流入／流出额度和 readiness 条件都满足时，下一配置才采用 $(50,20,20,10)$。虽然新旧总权重都为 100，仍必须分别计入 10 单位流入与 10 单位流出，并承认成员集已经改变。

候选权重冻结后取得随机值 $\rho_{e+1}$，区域名单生成 $(C,A,B)$。如果使用最小改动方案，$A$ 的局部名单可能是 $(A_3,A_1,A_4,A_2)$，每个身份一轮；如果使用 stake 抽样方案，四个节点单次获得主机会的概率为 $(0.5,0.2,0.2,0.1)$。两者是不同的经济安排。

假设启动时有效状态是 $C=\mathrm{off}$，$A=B=\mathrm{live}$，那么新配置首个 region 为 $A$。委员会之后把 $C$ 恢复为 live，只影响未来符合条件的选择，不抢占当前健康 term。

若旧配置已确认到 batch 105，而公告时只看到 batch 100，交接锚点必须覆盖 105 及其他必须保留的输出。不能为了赶上 30 分钟边界退回 100。完成安全关闭和新配置激活后，$A_2$ 的新投票权才减到 20，$A_4$ 才取得 10 的投票权。

$A_2$ 减少的 10 单位进入历史责任状态，直到相应期限与争议处理完成才可取回。旧 batch 的签名仍按配置 $e$ 的 30 单位权重解释，不能因为提款或新配置权重改变而重算历史回执。

### 12.2 需要先定下来的政策

| 选择 | 建议的第一版方向 | 以后可以扩展什么 |
| --- | --- | --- |
| 资产模型 | 单一抵押品、直接或排他分配 | 多资产折价、跨服务风险 |
| Epoch | 30 分钟作为候选节奏，提前准备 | 根据最终性与同步实验调整 |
| Region 集合 | 固定 region ID | 新区创建、合并、注销 |
| 节点入场 | 公开注册、排队、准备检查 | 随机委员会或其他容量选择机制 |
| 变更限额 | 全局和每区分别控制流入／流出 | 动态额度与更细风险指标 |
| Region 排班 | 一次随机排列、状态过滤 | 按 stake 分配生产频率 |
| 区内 leader | 先保留每身份一轮并随机排序 | Stake 抽样及备用名单 |
| 状态委员会 | 未来生效的区域资格更新 | 受约束紧急操作与治理升级 |
| 配置切换 | 显式关闭、继承锚点、唯一激活 | 更复杂的无停顿重配置 |
| 提款 | 旧职责结束后保留可罚窗口 | 独立流动性缓冲 |

这里“先保留”服务于逐步补齐 v3；若你一开始就要求收益与 stake 对齐，应直接选择加权方案，并把相应 leader 与 view-change 改动纳入同一形式化任务。

### 12.3 后续应写成哪些定义和命题

下一份 LaTeX 草稿可以依次增加：

1. **Membership request 和 liability record**：定义资金、身份、职责和退出责任的不同状态。
2. **Epoch snapshot construction**：明确 L1 截止点、排队顺序、金额／身份预算和确定性输出。
3. **Randomness and schedule generation**：固定随机输入、名单算法、权重语义以及随机源失效行为。
4. **Configuration activation**：明确旧 session 的关闭条件、新 session 的起点和统一控制顺序。
5. **Historical verification and withdrawal**：保证历史配置可查询，资金释放与责任窗口一致。

应分别分析五类性质：

| 性质 | 应当回答的问题 |
| --- | --- |
| 确定性 | 相同最终化 L1 输入和随机性是否生成同一配置与名单 |
| 权限唯一性 | 新旧配置是否可能同时授权冲突的生产路径 |
| 前缀安全 | 配置变化是否保全已确认与必须保留的旧历史 |
| 可追责性 | 在规定时间内出现的证据是否仍能对相应资金执行处罚 |
| 活性和公平性 | 在哪些假设下能完成入场、退出、故障接替；机会如何分配 |

它们不能由一个“stake＋VRF＋multisig”的总括描述共同推出。尤其 permissionless 并不消除区域诚实比例假设，提款等待期并不证明安全交接，随机排班也不证明多签不会选择性排除候选人。

### 12.4 参数应如何用实验和模型确定

先测量新成员同步 checkpoint 和追赶实时数据所需时间、同区完整消息传播尾部时延、L1 最终性与控制消息送达延迟、退出／加入压力下的处理能力。再估计发现违规、形成证据、争议和链上执行所需窗口。

压力场景至少包括：大量小额注册、大额单请求、同一 epoch 大规模增减仓、某一区集中退出、L1 最终性停滞、随机源不发布、状态反复抖动、交接期间还有旧确认到达，以及外部罚没导致担保骤降。

这些测量有助于选择 epoch、队列额度、准备期和追责期；它们不能取代跨配置安全证明。第一版最有价值的成果，是把每个状态转换的输入、证据、责任和失败行为明确写出来，使实验和证明有共同对象。

## 13 原始资料阅读顺序

| 阅读目的 | 建议顺序 |
| --- | --- |
| 理解以太坊资金怎样进入共识 | [EIP-6110](https://eips.ethereum.org/EIPS/eip-6110) → [EIP-7251](https://eips.ethereum.org/EIPS/eip-7251) → [Electra](https://ethereum.github.io/consensus-specs/specs/electra/beacon-chain/) |
| 理解资金拥有者如何退出 | [EIP-7002](https://eips.ethereum.org/EIPS/eip-7002) → [提款说明](https://ethereum.org/staking/withdrawals/) |
| 理解未来名单与随机性 | [EIP-7917](https://eips.ethereum.org/EIPS/eip-7917) → [EIP-4399](https://eips.ethereum.org/EIPS/eip-4399) → [RFC 9381](https://datatracker.ietf.org/doc/html/rfc9381) |
| 理解平台层节点管理 | [Lido StakingRouter](https://docs.lido.fi/contracts/staking-router/) → [Staking Modules](https://docs.lido.fi/staking-modules/) |
| 理解 AVS 的真实可罚担保 | [Operator Sets](https://docs.eigencloud.xyz/eigenlayer/concepts/operator-sets/operator-sets-concept) → [Unique Stake](https://docs.eigencloud.xyz/eigenlayer/concepts/slashing/unique-stake) → [Allocation](https://docs.eigencloud.xyz/eigenlayer/concepts/operator-sets/allocation-deallocation) → [Safety Delays](https://docs.eigencloud.xyz/eigenlayer/reference/safety-delays-reference) |
| 理解捕获快照和责任窗口 | [Symbiotic Network](https://docs.symbiotic.fi/learn/core-concepts/network) → [Epochs and Delays](https://docs.symbiotic.fi/learn/mechanics/epochs-and-delays) |
| 理解 Symbiotic 新的资金架构 | [Core V2 Vaults](https://docs.symbiotic.fi/get-started/what/components/vaults) → [Universal Delegator](https://docs.symbiotic.fi/get-started/what/components/universal-delegator) → [应用集成](https://docs.symbiotic.fi/integrate/applications/deploy-an-application) |

参数按本文日期读取的官方文档和相应 fork 规格解释。具体接入 EigenLayer／Symbiotic 时，仍需将所选网络、合约地址、版本和真实配置固定下来；本文没有把某个网页参数当成所有部署共同的链上事实。

[返回中文学习指南](README.md) · [Restaking 服务专题](restaking-services/README.md)
