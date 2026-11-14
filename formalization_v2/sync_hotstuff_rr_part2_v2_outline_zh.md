# Sync HotStuff RR Part II v2：跨区域轮换修订提纲

**日期：2026-10-17。**
本文配套[正确性评估与修订计划][PLAN]和 [Architecture v3 提纲][ARCH3]。
它明确拟写入正式协议的规则、顺序与证明义务；原 [Part II][OLD] 仍作为未改动的基线。

## 1. 核心模型与性质

本版保留区域 Sync HotStuff 引擎，在其外部组合三类行为：

1. 正常区域交接：基于 terminal FINAL 和 Ready，不等待 global finality。
2. 全局确认：完整合法区域结果在 Espresso finalized 后，按统一解释规则加入 G。
3. 强制交接：自动窗口失败或预先公告的多签 Force，在 Espresso 时间边界上关闭旧分支，从 G 继续。

G 是不可回滚的 DATA 前缀。
R 是可领先于 G 的区域生产分支，尚未进入 G 的后缀可能被 Force 放弃。
管理资格和恢复行为不能把 G 改为更旧的前缀。

这项保证依赖已声明的区域输出、Espresso、L1 公告和验证假设；本文不声称已经完成完整安全与活性证明。

## 2. 时间与控制输入

### 2.1 L1 公告服务

每个控制操作实际提交于 s 后，在 s + T_L1 前，对所有满足模型条件的正确在线参与者可取得、可验证且已经 finalized。
T_L1 已包括 inclusion、finality、传播和读取验证，不再重复加入单独 announcement delay。

多签只在 L1 记录控制。
它为每条控制选择足够未来的 Espresso `activate_at`，满足修订计划中的提前公告条件。
新加入节点先补齐所需历史，不用自己的加入时间判断旧指令是否“迟到”。

### 2.2 Espresso 时间和位置

```text
E_h          Espresso 区块
tau(h)       被该 finalized header 认证的 timestamp
p            (h, namespace transaction index, record index)
cross(d)     第一个 tau(h) >= d 的 finalized Espresso 区块
```

Timestamp 单调不减，区块及区块内位置构成全序。
所有名义阈值 d 都在 `cross(d)` 开头实现。
同 timestamp 的后续区块不能被插回已经完成的截止检查之前。

管理控制采用 L1 授权顺序和未来 Espresso 生效时间；自动事件直接由完整 Espresso 历史推导。
双方都不以本地收包时间或 caller 声称的观察时间决定公共状态。

### 2.3 两类外部输入

```text
L1 ControlRecord {
    chain_id, control_seq, previous_control_commitment,
    kind, activate_at, payload
}

Espresso EXPORT {
    chain_id, branch_epoch, expected_global_base,
    consecutive_DATA, receipts, regular_witnesses,
    historical_authority, native_context
}
```

L1 控制种类至少包含：

- `SET_STATUS(region, allowed, activate_at)`：更新未来选择资格。
- `MSIG_FORCE(target_branch_epoch, activate_at, next_configuration)`：强制关闭当前目标分支。
- 明确的可选管理操作，例如恢复资格、清除 cooldown；不能用字段缺省表达无限权限。

自动 Force 不属于必须由某人提交的输入，它是 Interpreter 的确定性结果。

## 3. 身份与公共状态

### 3.1 身份隔离

| 身份 | 作用 |
| --- | --- |
| `branch_epoch` | 一条在两次强制重开之间延续的生产分支 |
| `session_id` | 一个区域的生产回合，包含 epoch、边界、anchor 和 authority |
| `(g,t)` | 一个 session 内的 recovery generation 和 leader term |
| `DATA height` | 全局应用输入的连续位置 |

正常跨区域交接、区域内部 leader rotation 和 recovery 都不改变 branch epoch。
两类 Force 都产生新 epoch，关闭旧 epoch 的后续全局追加资格。
历史上已经属于 G 的旧 epoch 块仍保持原始身份和凭证。

### 3.2 Export session

`export_session` 是公共解释状态中负责提供 G 下一 DATA 的 session。
它不同于区域网络中最新的生产 session。
例如 A 已正常交给 B、B 又交给 C，而 G 尚未接受 A 的终端 DATA，export session 仍可能是 A。

自动检测的是这个共同可判定的下一段交付职责。
如需监控实际最新生产分配，可以增加认证的 HANDOFF / READY 通知，但不能把未验证的自报 leader 当作公共事实。
这些通知不重置 DATA 交付窗口。

### 3.3 状态草案

```text
State {
    G_height, G_digest,
    branch_epoch,
    export_session, export_region,
    session_base, quota, confirmed_in_session,
    authority_ref, normal_boundary_context,
    phase,                       // Exporting 或 Waiting
    window_phase, window_start, deadline,
    window_target, accepted_in_window,
    governance_status_history, cooldown_until,
    processed_control_seq, control_commitment,
    epoch_open_E_height, epoch_open_timestamp,
    espresso_position, espresso_quorum_anchor
}
```

区域副本单独维护它已打开的 provisional sessions、锁、votes、timers、执行缓存及用户请求队列。
任何全局恢复都不能复用旧 session 的协议状态冒充新 session。

## 4. 正常区域协议：保留快路径

### Protocol R1：区域生产与提交

尽量保留当前 Part II 的 steady state、区域内 recovery、DRAIN、FINAL 和 Completion。
所有相关签名绑定完整 session / branch epoch。
正确副本在区域提交后传播 FINAL、数据和必要依赖；持有完整 receipt 时继续 relay。

区域 FINAL 或同 session 覆盖凭证是输出证据，proposal QC 不是替代物。
固定快照、Byzantine 权重界及历史 key 的接受条件不因本次重构而改变。

### Protocol R2：正常交接

收到 predecessor 的 terminal FINAL 后：

1. 验证 quota、terminal block、完整必要状态及历史 authority。
2. 验证由 terminal 内容绑定的 `selection_ref`。
3. 从该历史选择依据计算 successor，形成唯一 REGULAR witness。
4. Successor 各成员完成验证后达到自己的 Ready，继承 terminal anchor 并 bootstrap。

此流程不等待 terminal DATA 或 REGULAR witness 被 Espresso finalized。
正常 session ID 由已知 witness 字段计算，不能依赖未来的 Espresso inclusion position。

### 正常选区的确定性

`selection_ref` 至少绑定一个已 finalized Espresso header 的高度、hash 和 timestamp，并受 session 的参考点下界约束。
它由区域签名内容固定，不由 publisher 在发布时挑选。
它属于外层 DATA / terminal 上下文，不需要修改原生 mini-block 编码；区域 proposal hash 与 FINAL 的内容绑定必须覆盖它。
在全局验证时，该参考点必须位于已认证的 Espresso 前缀内，不能引入向未来记录的循环依赖。

初版沿用历史选择的思路：从该参考点起，取第一个按有效历史资格有其他 eligible region 的已认证 Espresso 位置，再按固定循环顺序选区。
若该参考点已经有候选，直接使用它；没有则等待第一个合格参考点。
一旦找到，后来的状态变更不重新选择这个 witness。

资格检查使用候选 session 对应的 authority，不随当前墙钟或新的 staking epoch 隐式替换。
缺少所需证据的节点等待，不因自己的缺失改选另一个区域。

该规则允许历史参考点，不声称它一定是物理交接时刻的最新健康视图。
需要立即停止这样的已打开或按旧依据打开的分支时，使用 MSIG_FORCE。
更强的 freshness 规则是独立的后续设计项。

## 5. Publisher 与全局输出接受

### Protocol P1：导出连续结果

Publisher 从 G 开始构造完整连续区间，可以包含多个正常 session。
每个 session 的凭证必须来自自身历史 authority；不能用 C 的 receipt 替代 A 或 B 的证据。
若有缺口，补齐后重投；不通过改变扫描顺序绕过缺口。

### Protocol G1：接受 EXPORT

整个包先完成结构、签名、来源、连续性和适用规则检查，然后原子接受。
至少要求：

- `branch_epoch` 仍有追加资格；
- expected base 对应当前 G；
- DATA 没有省略、插入、重排、重复高度或错误父链接；
- 所属 session 的正常边界、authority 和 quota 正确；
- 区域结果满足约定的输出有效性契约；
- 必要的原生数据与上下文完整可验证。

每个新接受 DATA 推进 G，并计入其 export session 的窗口。
不符合条件的原始 namespace 数据不推进 G，也不重置窗口。

当一个 session 的 quota 全部进入 G，完成其 export 职责并取消其剩余窗口。
Interpreter 根据固定的正常交接依据转到 successor；其生产可能已经提前开始。
一个包内可以依次完成 A、打开 B 的 export 职责并接受 B 的 DATA。

若正常边界暂时没有 eligible successor，保留 terminal anchor 并进入 Waiting。
之后按同一历史选择规则等待第一个合格候选；已完成 quota 的 A 不被误判为还欠新 DATA。
必要 authority 或验证材料不可得时不能猜测其内容。

## 6. Block-missing 的窗口规则

### 6.1 统计什么

统计的是本 session **新进入 G 的 DATA 块数**。
不统计：QC、DRAIN、重复 DATA、同一块的不同 signer sets、heartbeat、仅有提交成功回应，以及尚不能接续 G 的未来区域记录。

参数至少为：

```text
Q          本 session 的 DATA quota，基础 profile 要求 Q > 0
W_start    export 职责打开后的启动窗口长度
m_start    启动窗口要求的 DATA 数量，通常可取 1，具体仍待定
W_run      后续固定窗口长度
q          后续每窗要求的 DATA 数量
C_auto     自动暂停后的 cooldown 长度
```

所有时间参数为同一 Espresso 时间单位的正整数；数量目标也为正整数，并裁剪到窗口开始时的剩余 quota。
本轮不根据现有平均区块速度擅自填入数值。
窗口参数及 quota 在相应 export session 打开时固定；后续参数更新不追溯缩短旧 deadline 或改变已经完成窗口的目标。
管理层需要提前结束旧职责时，使用 MSIG_FORCE。

### 6.2 打开与推进窗口

一个 export session 在某个 Espresso 区块被公共状态确定为下一交付者时，以该处理位置的 timestamp `tau_0` 打开启动窗口：

```text
window = [tau_0, tau_0 + W_start)
target = min(m_start, remaining_quota)
```

达到 deadline 时：

- 已完成 session quota：旧窗口已取消，不再检查。
- 数量达到 target：结束该窗，打开下一固定窗口，计数清零。
- 数量不足：导出 AUTO Force。

后续窗口长度为 W_run，目标为 `min(q, remaining_quota_at_window_open)`。
窗口内超额数量不自动存成未来任意长时间的活性额度。
若要采用允许 burst 的其他政策，应另列 profile，不能由实现各自决定。

### 6.3 截止点与时间跳跃

窗口为半开区间，所以 DATA 的 Espresso timestamp 等于 deadline 时已不属于旧窗。
第一次跨越 deadline 的 Espresso 区块，先检查旧窗，再处理其中的数据。

如果一个区块跨过多个 deadline，依次结清已经完成的窗口，遇到第一个失败窗口就进行一次 AUTO Force。
新强制 session 的启动时间取**当前触发区块的 timestamp**，不取那个已经过去的 deadline。
因此不会在同一个区块内让 A、B、C 因历史时间跳跃而连续全部超时。

Espresso 自身长期停摆后恢复，可能使旧交付窗口失败；这说明端到端服务没有满足窗口，不自动归责区域。
需要区分故障原因时依赖诊断或额外政策，不改变本轮统一的前缀保护规则。

## 7. 两类 Force 与共同的关闭函数

### Protocol F1：自动 Force

失败窗口确定后，生成一个派生身份：

```text
AutoForceID = H(
    chain_id, protocol_version, old_branch_epoch,
    export_session_id, failed_window_id,
    triggering_E_block_hash
)
```

逻辑原因是该窗口未满足数量目标；实际边界是触发 Espresso 区块开头。
无需额外的签名委员会、keeper 请求或 L1 写入。

区域的自动暂停从实际触发 timestamp 起计时，例如：

```text
cooldown_until[export_region] = tau_trigger + C_auto
```

它暂停的是这次欠交付服务的区域，不声称已经定位了真实故障组件。

### Protocol F2：多签 Force

多签指令包含目标 branch epoch、未来 `activate_at`、唯一控制序号及授权配置。
在 `cross(activate_at)` 开头检查目标是否仍匹配，然后执行关闭。
不要求当前区域缺块；风险隔离可以在区域持续输出时进行。

指向已经关闭 epoch 的控制不再次切换，也不影响后来复用某个数字高度的新 session。
同一操作重放不产生新 epoch，不重启任何窗口。
若 Force 自身携带配置变化，它只随有效 Force 原子执行；独立 SET_STATUS 则按自己的授权和生效规则处理，不因相邻 Force 过时而消失。

### 共用函数 CloseAndOpen

```text
CloseAndOpen(cause, triggering_block):
    anchor := current G
    close old branch epoch for all future appends
    cancel old export-session windows
    archive unconfirmed old-branch descendants
    apply cause-specific runtime status changes
    select successor using the defined effective eligibility context
    create a new branch epoch and session identity
    open startup budget from triggering_block.timestamp
```

Anchor 不由多签或 publisher 任选，始终是边界前完整接受的 G。
关闭范围包括旧 epoch 的正常 successor 输出，避免旧分支通过其他区域重新接回。

新 epoch 的身份绑定原因、触发区块、anchor、successor 与 authority。
新区域在验证触发区块已 finalized、L1 控制视图完整且达到 Ready 后启动。
它的新输出不得在触发该 epoch 的同一个 Espresso 区块中获得接受；后续区块才承载该新 epoch 的 DATA。

如果没有 eligible successor，则保持 Waiting，保存固定 anchor 和关闭事实。
之后在资格更新或 cooldown 到期所对应的确定性处理位置重新选择；不撤销旧关闭，也不从本地时钟自行选区。

## 8. 活性状态及生效语义

### 8.1 来源分离

```text
GovernanceAllowed_i(tau)
    由 L1 控制记录及其 activate_at 历史决定

AutoEligible_i(tau)
    由经过认证的自动 Force 与 cooldown 历史决定

SelectionEligible_i(tau, authority)
    = GovernanceAllowed_i(tau)
      and AutoEligible_i(tau)
      and valid_nonempty_authority(i)
```

这是选区资格，不用于重新计算旧 snapshot 的投票权或历史 receipt 有效性。
当前 session 的服务权限来自其已验证边界；要主动终止它所在的生产分支，使用 Force。

### 8.2 OFF、ON 和更高权限

普通 OFF 在 activate_at 起改变未来选择依据；它不修改过去的选择参考点。
普通 ON 恢复管理层许可，不复活旧 epoch，不自动消除尚未结束的自动 cooldown。
若管理层需要额外恢复权，应定义显式 `CLEAR_COOLDOWN` 等操作，且同样提前公告、按 Espresso 时间生效。

多签更高权限表现为可以主动 Force、阻止自动恢复、恢复候选资格，不表现为重写 G。
如果它拥有任意升级验证程序的能力，该能力属于另外的治理信任假设。

### 8.3 待命区域与候选不足

待命区域没有当前 DATA 输出义务，不能因沉默直接触发 block-missing。
READY 可作为候选的可用性提示，但不证明未来同步性或故障权重界。

基础正常轮换保留“选择其他 eligible region”的规则。
如果没有其他候选，等待；若希望仅剩一个健康区域时也可持续服务，需要另行启用同区域续任或冷却后的重试规则。
任何此类重试都建立新 session，不能复用已关闭的锁、投票或身份。

## 9. 统一事件循环

所有节点与 certifier 应执行同一份逻辑。
以下为提纲级伪代码，具体编码和证据格式另定：

```text
ProcessFinalizedEspressoBlock(E):
    authenticate E, its predecessor relation and complete namespace
    require a complete applicable finalized L1 control context
    tau := E.timestamp

    apply due status updates in (activate_at, control_seq) order
    closed := apply applicable due MSIG_FORCE, if any

    if not closed and there is an export session:
        while current window deadline <= tau:
            if window requirement is met:
                advance to the next fixed window
            else:
                derive AUTO_FORCE and CloseAndOpen at E
                break

    if Waiting:
        resolve any newly eligible successor by the fixed selection rule

    for EXPORT in namespace order:
        if complete validation succeeds:
            atomically apply its DATA and regular session transitions
        else:
            do not modify G or count it as progress

    persist G, control context, window state and replay checkpoint
```

新打开的 export session 总是得到正长度启动窗口，所以同一触发区块不会再次对它运行超时。
如果多个管理阈值和 deadline 映射到同一区块，采用本节优先级；它们之间没有任何本链 DATA 被插入。
控制输入排序和具体失败类型必须一致：完整对象格式错误可拒绝，历史下载不完整则整个判断等待。

在已经持久确认的 checkpoint 之后发现一条此前遗漏、且本应在已处理历史中生效的 L1 控制，意味着控制完整性或提前公告前提已经被违反。
不能把它按本地发现时间悄悄改为另一处切换，也不能自动回滚 G。
这是模型或实现错误的处理边界，不是允许各节点各自忽略迟到指令的一条共识规则。

## 10. 观察者、发布验证与旧引擎

### 10.1 观察者

观察者维护完整 L1 控制上下文、Espresso replay cursor、G 和派生状态。
它不维护旧版逐块的首次 relay 时间、委员会保护 deadline 或 `G_msig`。
它也不能只看单条 inclusion proof 就跳过区间中的控制和自动事件。

### 10.2 Proposer 与 L1 Publication Verifier

发布从 C_pub 的下一 DATA 开始，准确对应 G 的完整连续区间。
强制切换不允许丢掉 G 中尚未发布的 backlog。
Certifier 证明其使用的控制版本、Espresso 区间、派生 Force、数据对应与适用执行接口；L1 检查版本和前置 checkpoint 的一致性。

全局确认、L1 登记和提现/执行声明最终化是不同阶段。
如果保留强 G 语义，派生规则不能仅因数据提交较晚就覆盖已经确认的普通内容。

### 10.3 旧区域引擎

Force 首先关闭公共追加资格，不伪造 regional quit，不抹去区域本地提交事实。
可以单独定义 `ExternallyClosed`，允许验证到关闭证据的旧 engine 停止新生产和投票。
它与 Part I 的内部恢复规则如何组合，需要作为明确的修改检查；不能在证明中默认为原协议已有行为。
数据服务、证据保留和提款责任不会因停止生产而自动结束。

## 11. 应证明的性质

| 性质 | 主要依赖 |
| --- | --- |
| 公共状态一致 | Espresso finalized 前缀一致、完整输入、L1 提前公告、确定性事件顺序 |
| 全局前缀不可回退 | EXPORT 只追加，两个 Force 都精确保留当前 G |
| 分支隔离 | 完整身份绑定；旧 epoch 的迟到输出不能重入 |
| 正常交接无需 global wait | REGULAR witness 与 session ID 在 terminal 区域提交后即可确定 |
| 自动 Force 不依赖多签活性 | 窗口失败是 replay 的派生事件，不需要外部触发交易 |
| 发布忠实性 | 准确下一范围、DA 对应和执行验证 |

证明结构应从共同输入和唯一转换出发，不再依赖“委员会在 deadline 前是否提交了删除指令”的旧分情况论证。
MSIG 控制仍依赖诚实、非追溯的提前公告假设；该性质不能被隐藏成无条件的恶意多签容错。

活性另需：至少一个可服务区域、可用 authority、足够数据与状态、区域持续工作窗口、Publisher 主备、Espresso 继续推进，以及适合服务能力的窗口参数。
当前 backbone 的 eventual delivery 本身不足以证明所有诚实服务都能赶上固定窗口。
错误设置窗口可以造成反复切换，但不能作为允许回滚 G 的理由。

## 12. 编写正式 Part II v2 的章节安排

1. Overview：生产、全局确认和发布三个前沿。
2. Model：区域输出契约、L1 公告服务、Espresso 时间与证明。
3. Identities and objects：session、branch epoch、EXPORT、control records。
4. Regional engine deltas：签名上下文、FINAL 导出、保留与关闭职责。
5. Regular handoff：历史选择参考点、terminal witness、Ready / bootstrap。
6. Global interpretation：连续输出接受、export session、窗口状态。
7. Automatic block-missing Force：固定窗口、半开边界、时间跳跃。
8. Multisignature control：时间生效、权限、目标匹配和幂等。
9. Common forced bootstrap：完整 G、候选不足、native continuation。
10. Observer and certifier：完整输入、控制版本和发布接口。
11. Safety and progress：不变量、证明义务和条件性活性。
12. Examples and open parameters：竞争情形及实现 profile。

## 13. 必须覆盖的例子

一个具体的计数例子：假设 Q = 4、W_start = 4、m_start = 1、W_run = 4、q = 2，时间与数量只用于演示。

| Espresso timestamp | 已被接受的结果 | 状态 |
| --- | --- | --- |
| 0 | 打开 A 的 export 职责 | 启动窗为 [0,4)，要求 1 块 |
| 1 | A 的第一块进入 G | 启动要求已满足 |
| 4 | 先结清启动窗 | 打开 [4,8)，要求 2 块 |
| 5 | A 的第二块进入 G | 当前运行窗只有 1 块 |
| 8 | 同一区块还携带 A 的第三块 | 先判定 [4,8) 不足，自动切换；第三块不补计 |

切换保留已经进入 G 的两块；successor 从下一 DATA 高度、以新 epoch 开始。
如果最后一行的 timestamp 改为 30，新启动窗从 30 起算，不从已经过去的 8 起算。

| 例子 | 正确结果 |
| --- | --- |
| B 已区域生产，G 仍缺 A 的输出 | 监控 A 的 export 职责；B 不能跳过缺口计数 |
| A 与 B 的完整连续包在 deadline 前进入 | 按序完成 A 并接受 B，无正常 global wait |
| 每窗偶尔有块，但数量一直低于 q | 固定窗口失败，仍触发 AUTO |
| 旧输出 timestamp 恰等于 deadline | 先处理失败，再拒绝旧 epoch 的追加 |
| 多个 Espresso 区块 timestamp 相同 | 不回溯补计旧窗口，保持唯一边界 |
| 长时间无区块，恢复时跨过多个 deadline | 至多对旧职责自动关闭一次，新职责从实际触发 timestamp 起算 |
| 状态更新和多签 Force 与自动 deadline 同时生效 | 先更新资格并处理管理 Force，不重复执行旧 AUTO |
| 非当前区域 OFF | 改变相应历史时间后的选择资格；不删除旧 receipt |
| AUTO 后 ON | 不复活旧 epoch；cooldown 是否清除按显式规则处理 |
| G 高于 C_pub | 保留 G，全量补齐未发布区间 |
| 不完整 namespace / 控制历史 | 等待验证，不据此判定缺块或无 Force |

[PLAN]: espresso_revision_plan_v3_zh.md
[ARCH3]: arch_v3_outline_zh.md
[OLD]: sync_hotstuff_rr_part2_rotation.tex
