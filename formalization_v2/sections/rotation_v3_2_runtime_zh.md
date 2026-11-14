## 6. Cross-region rotation：四模块与并发程序

### 6.1 Four-module cross-region architecture

每个正确区域副本独立运行一个 **Session Manager**。它观察 finalized L1 和 Espresso 历史，接收区域结果，并管理以 $(\beta,r)$ 命名的独立程序。Manager 是各副本执行的本地逻辑，不是新增的受信任区域控制器。

一个区域可以同时运行旧 session、准备可能的后继，并把更早 session 的数据责任移交给存储和发布服务。完整区域 view 仍是 $(\beta,r,g,t)$，不增加另一套 session 编号。

| 模块与执行者 | 输入 | 输出与权限边界 |
|---|---|---|
| Soft Force Detector；任何验证节点 | 完整已验证 Espresso 历史、export duty、固定窗口与实际接受计数 | 可重算的窗口失败条件；不自行改变 $\beta$ |
| Multisig Health Controller；既有 committee | 区域健康、同步和交付观测 | 未来生效的 L1 MSIG、STATUS、CLEAR；不选择保留高度 |
| Cross-region Handler；每个正确区域副本 | 已验证公共更新、区域 receipt、本地任务事件 | 准备、激活、请求收尾、责任移交；不再次执行 Force |
| Ready and Activation Gate；每份 session 程序 | 已固定交接、精确 anchor、执行状态、authority | 继续 Waiting，或通过 Protocol 4 原子进入；预取不授予投票权 |

Soft 检测和 Multisig 健康管理是两个并列的干预来源。Handler 管理候选和正式交接，每份 session 的启动检查负责验证其精确起点。

健康的非活跃地区仍持续观察两条公共流，可以在被选中前准备数据和 authority。公共重放游标、投机程序和存储任务是不同对象。一个本地程序处于什么 phase，不能代替 EXPORT 能否进入 $G$ 的公共验证。

### 6.2 Soft detection and multisig health control

Soft Force Detector 从一个已验证 replay checkpoint 和后续完整 Espresso 历史开始，重建当前 export duty，并按第 7 节结算固定窗口。它返回以下三种结果之一：

- 尚无失败；
- 失败窗口描述、首次越过 deadline 的 header 和支持历史；
- 缺少证据，需要继续获取。

失败描述绑定 chain、当前 branch version、export-session pair、窗口起点、deadline、target 和 accepted count。缺少 namespace 或 control 证明不能被解释成 count 为零。

只有实际追加到 $G$ 的 DATA 计数。重复 DATA、DRAIN、proposal QC、无效 EXPORT 都不计数。被观察的 export session 可能比最新生产 session 落后多个常规轮次。轮询只用于取得新历史；轮询频率和本地收取时间不决定是否失败。任何接收者都要对照自己的已认证前缀重算外部提供的失败描述。

Protocol 5 仍是唯一的 Force 状态转换入口，顺序不变：

$$
\text{独立 STATUS/CLEAR}
\longrightarrow \text{首个目标版本匹配的 MSIG}
\longrightarrow \text{否则检查 AUTO}
\longrightarrow \text{EXPORT payload}.
$$

已经算出的 Soft 条件不能越过同一 prelude 中匹配的 Hard Force。

Multisig 的健康政策区分四种动作：

| 目的或观测 | 控制动作 | 含义 |
|---|---|---|
| 某区域不适合在后续接班 | 未来 OFF | 从后续选区资格中排除 |
| 需要结束目标版本的生产安排 | 未来 MSIG Force；需要持续隔离时另含或另发 OFF | 到规范激活位置且目标版本匹配时切换 |
| 已检查同步、数据服务和运维恢复 | 未来 ON | 恢复后续选区资格 |
| 需要在自动 cooldown 到期前解除它 | 显式 CLEAR | ON 本身不清除 cooldown |

每条操作记录目标、获授权的变化和未来激活时间。具体健康阈值仍是现有诚实 committee 假设下的运维政策，本节不新增数值阈值。

MSIG 针对 branch version。普通交接只改变 $r$，不会使它失效；先发生的 Force 改变 $\beta$，可能使它过期。只有目标匹配时才应用其附带配置；独立 STATUS 按自己的激活规则处理。健康观测不能直接改写其他副本的公共状态，所有动作都要经过 finalized L1 与预先公告的 Espresso 激活位置。一次 ON 也不证明未来的区域同步和故障假设永久成立。

### 6.3 Cross-region Handler

Manager 串行处理会修改本地控制状态的事件。下载和状态恢复可以并行。

| 输入事件 | 验证与路由 |
|---|---|
| `L1Finalized` | 认证完整 control prefix 和 authority；把未来控制放入队列，不提前激活 |
| `EspressoFinalized` | 认证完整有序历史；使用 Protocol 6/5 产生的状态转换 |
| `RegionalFinalized` | 验证 DATA、完整 FINAL receipt 和必要 ancestry；路由给原生产 pair，并交给 Publish Layer |
| `MaterialAvailable` | 核对目标 pair、anchor 原生产 pair、height、hash、所请求对象和验证结果；只更新相匹配的准备任务或固定交接 |
| `CoreEvent` | 接收指定本地 engine 的事件；记录 view、Quit、恢复、Completion 或退役进度 |

**单个 FINAL 签名不是 `RegionalFinalized` 事件。** 既有参与者收集匹配签名，形成满足 receipt 定义的完整证书。正确持有者把完整结果交给 Manager 和已经进入的源 engine，并提交给 Publish Layer。源 engine 执行原有 receipt 和 Completion 规则。尚未进入的程序只缓存结果，不能通过 receipt 回调创建 engine 或跳过交接验证。聚合不是新增共识轮次；$T_{\rm pub}$ 仍从完整结果可用时开始。

输出动作如下：

| 输出 | 接收方与内容 |
|---|---|
| `Prepare/Fetch` | 数据或状态任务；绑定目标与精确材料 |
| `Activate` | 指定 session 的 Protocol 4；执行一次初始 Enter |
| `RequestClose` | 旧程序；携带已验证关闭原因 |
| `OfferResult` | Publish Layer；保留原 DATA、receipt 字段 |
| `Persist/Release` | 存储和资源管理；持久记录状态并移交责任 |

回调提供的是材料，不是激活命令。一个晚到结果可以对存储有用，却不匹配程序当前的准备目标。

**Protocol 3：处理跨区域事件与管理程序。** 每个正确区域副本独立执行：

1. **验证事件。** 按完整 context 路由公共输入、区域结果和本地回调，保留 pending requests 与源证据。材料或 session 与请求不匹配的完成回调不能满足请求。
2. **使用公共重放结果。** 对新公共历史运行或使用 Protocol 6，包括规范顺序中的 Protocol 5，读取结果中的 $\beta$、$G$、资格 snapshot、duty 和 boundary。本 Handler 不独立增加 $\beta$、选择另一 cutoff 或改写 $G$。
3. **核对正式交接。** 对尚未关闭版本的有效 terminal receipt 导出 REGULAR boundary；接收公共重放确定的 Force boundary；按第 6.5 节验证。为本地区的正式目标准备材料，并向被选成员发送完整交接。未来 Force 不阻止当前普通交接。
4. **核对候选。** 根据当前 export context、$G$、合格地区和已知未来控制重算可能目标。只更新尚未固定的准备计划；目标 pair 改变时使用另一份程序记录。暂停过时的投机工作；只有已验证事实确定目标不可能成立，或固定交接已分配给其他地区时，才终止尚未进入的程序。
5. **处理关闭与区域结果。** 对版本已关闭的已进入程序，按退役合同发出幂等 `RequestClose`，保留其签名路由、历史证据和发布责任。terminal receipt 仍交给已进入的源 engine 执行原 Completion。
6. **检查并进入。** 对本地区已有固定交接的 Waiting 程序检查 Ready。在同一个本地事件内，最终复核公共有效性和持久 entry flag，记录 Ready，并原子调用 Protocol 4 Enter。检查失败则继续 Waiting 或执行有依据的取消，不能留下半初始化 engine。
7. **持久化并派发。** 在对外暴露动作前，持久记录状态、进入和退出记录以及必要输出动作。异步任务携带完整目标和 anchor context。重复事件可以继续未完成动作，但不能创建第二次逻辑进入。

公共解释器只有一条更新路径：Protocol 5 定义控制、窗口和 Force 转换；Protocol 6 认证并执行这些转换以及 EXPORT 接受；Protocol 3 使用结果；Protocol 1/2 只推进指定 session 的区域 core。把它们放在同一个进程中实现，也不改变这项权限划分。

### 6.4 Session lifecycle

Manager 为每份本地程序维护：

```text
Program[(beta, r)] = {
    phase,
    preparing_plan,
    fixed_handoff,
    anchor_block,
    execution_state,
    authority_and_bounds,
    core_state,
    pending_fetches,
    entered,
    close_requested,
    publication_and_retention_handles
}
```

六个 phase 描述本地程序，不描述全局 finality。

| Phase | 含义 |
|---|---|
| **Waiting** | 准备候选或正式交接，可以获取数据、恢复状态和缓存证据；尚未进入区域 engine、签区域 vote，或启动区域 commit/progress timer |
| **Ready** | 已固定交接、精确 anchor 状态和全部依赖通过检查；同一个串行事件紧接着执行 Enter，不额外引入调度等待 |
| **Active** | 已进入尚未 Frozen 的 generation；包括初始 STATUS、$2\Delta$ 等待、NEWVIEW、ANCHOR 认证与后续生产；收到关闭请求时仅受明确退役合同约束 |
| **Frozen** | 当前 generation 有合法 Quit proof；沿用原有 evidence、receipt 和 recovery 规则；Force 本身不会产生此 phase |
| **Completed** | 已进入的 engine 取得合法 terminal receipt 并执行原 Completion，停止生产并按原规则取消 timer；数据交付和保留责任可能仍在 |
| **Terminated** | 逻辑程序退出，不再处理 BFT timer 或发出 BFT 签名；持久存储和发布服务继续承担历史责任，退出记录阻止晚到消息重新启动它 |

正常转换是 Waiting → Ready → Active；合法 Quit 导致 Active → Frozen；原恢复准入使 Frozen → Active，同时推进 $g$、令 $t=0$；terminal Completion 使 Active/Frozen → Completed。

Waiting/Ready 在目标确定失效时可以终止；Completed 在责任移交后可以终止。Active/Frozen 只有满足退役合同才能 Terminated。请求收尾期间保留原 core phase，并设置 `close_requested`，不增加第七个状态。

尚未固定的 `preparing_plan` 可以随着 $G$、源 duty 或候选资格改变。固定交接的目标 pair、地区、初始 anchor、source context、选择位置、authority、reference bounds 都不可替换。后来的 OFF 尤其不能改写历史上已经固定的常规选区。

锁、votes、view entry time 和 timers 只属于一个 pair。进程崩溃后恢复同一持久逻辑记录，不能清除 votes 或重做初始 Enter。存储和任务引用绑定完整 pair 与 anchor 身份，不需要另一套协议级 session ID。

多个地区可以同时为同一未来 pair 保存未授权的 Waiting 准备记录，只有固定交接选中的地区可以进入。暂时失去候选资格可以仅暂停任务；如果以后仍可能被选中，不必立即 Terminated。退出记录在释放运行资源后仍然保存。

旧程序仍 Active 时可能已经无权推进 $G$；Completed/Terminated 程序的既有结果也可能继续被接受，只要版本和区间仍满足公共接受规则。生命周期 phase 不创造或撤销这项权限。

### 6.5 Handoff records、anchor 与 Ready

`NextLive(i, M)` 在固定循环顺序 $\mathcal L_R$ 中，从 $i$ 之后寻找第一个**其他**合格地区，最多检查一整轮。没有则返回 $\bot$。资格由历史 governance flag、cooldown 和指定 authority 的正总权重共同决定。genesis 的合成源为 $\bot$，从列表首地区开始搜索整轮。本版保留必须选择其他地区的规则。

pending boundary 保存 source context $(i_b,\beta_b,r_b)$，下一个数字 index 是 $r_b+1$。常规边界保存已完成前驱；存在 export duty 时，Force 捕获该 duty 的 context；没有已分配 duty 时，Force 复用已经保存的 source context。分配前发生多次 Force，会替换 pending cause 和 version，不会逐次增加尚未打开的 session index。genesis 使用 $(\bot,\bot,-1)$。

REGULAR 从 $(\beta,r)$ 打开 $(\beta,r+1)$，精确继承 $T_{\beta,r}$。Force 在新版本打开 index $r_b+1$，精确继承触发块 payload 前的 $G$。这个 anchor 的生产 session 可能早于被捕获的 export session，它保留原 producing pair、hash 和 commitment evidence。

常规边界令 $q_0$ 为 $\xi(T_{\beta,r})$ 的高度；Force 令 $q_0$ 为触发 Espresso 高度。使用上述源地区与下一 authority：

$$
q_s=\min\{q\ge q_0:\operatorname{NextLive}(i_b,\chi_E(q))\ne\bot\},
\qquad
i_{\rm next}=\operatorname{NextLive}(i_b,\chi_E(q_s)).
$$

只搜索已验证历史。缺少证据就等待，不能跳过可能的更早候选。后来的 snapshot 不改变已经确定的最小值。普通选区不必等待 terminal DATA 发布到 Espresso；它的签名 reference 可以是历史位置，本版不新增按接收时间判断 freshness 的规则。

一份带类型的 `Handoff` 包含：

| 字段 | 类型和意义 |
|---|---|
| Domain、kind | chain/protocol domain；REGULAR、MSIG、AUTO 之一 |
| Source context | 前驱 pair $(\beta_b,r_b)$ 和源地区 $i_b$；未分配 duty 时包括保存的 context |
| Target | 目标 pair $(\beta',r')$ 和被选地区 $i_{\rm next}$ |
| Anchor | 原 producing pair、DATA height、hash、完整 DATA 或引用、原 direct/covering commitment evidence |
| Authority | finalized $\mathsf{AUTH}_{r',0}$、固定配置、quota 与 publication-height proof |
| Selection、bounds | $q_0,q_s$、已认证 selection snapshot、$q^{\rm open}_{\beta',r'}$、$\lambda^{\rm open}_{\beta',r'}$ |
| Cause、evidence | 类型对应的原因；可信或已验证 replay checkpoint、必要完整 L1/Espresso 历史与区域证明 |

引用可以指向已验证缓存，也可以指向尚待获取的对象。只有引用本身不能使程序 Ready。

`VerifyHandoff(C, history, packet)` 接受一个获授权 replay checkpoint、必要源历史和交接包，返回已验证固定字段、invalid 或 needs evidence。它不独立修改公共状态。

1. 检查 domain、字段类型、source 与 target 名字。从 checkpoint 认证并重放 cause、selection 和当前已处理公共前缀所需的兼容历史。检查完整适用 controls，不能只证明某条 control 被包含。可以使用已验证缓存。
2. 对 REGULAR，验证前驱自身的固定边界、初始 authority、精确 quota，以及 $h(T_{\beta_b,r_b})=h(A_{\beta_b,r_b,0,0})+\TB_{r_b}$；验证 terminal FINAL receipt、ancestry 和签名 $\xi$。要求 $\beta'=\beta_b$、$r'=r_b+1$，且该版本在已处理前缀中尚未关闭；anchor 必须就是 terminal DATA。
3. 对 MSIG，验证已授权 finalized L1 record、control sequence 和首次 crossing header $J_{q(\theta_a)}$。重放完整 prelude，确认它是首个 target 与 prelude entry version 匹配的到期 Force。只有匹配时才检查并应用附带配置。过期指令及同一 prelude 消耗掉的其他指令不能形成第二个边界。
4. 对 AUTO，从完整历史重放固定窗口计数与首次 deadline crossing，检查 accepted DATA 不足、同一 prelude 不存在优先匹配的 MSIG，并检查在实际触发 timestamp 导出的 cooldown。报警或本地 timeout 不构成证明。
5. 对两种 Force，从唯一 replay transition 取得 captured source context、旧/新版本、保留前缀和 boundary。要求 $\beta'=\beta_{\rm entry}+1$、$r'=r_b+1$，anchor 与 payload 前 $G$ 完全一致，包括 provenance。未分配 duty 期间，要验证存储 context 和中间 Force，不能虚构一个空 export session。结果版本在已处理前缀中仍须开放；后续有效 Force 会取代此边界。
6. 从 $q_0$ 和必要完整历史 snapshot 重算首个合格 $q_s$，核对地区与不可变下一 authority。设置 $q^{\rm open}_{\beta',r'}=q_s$，并令 $\lambda^{\rm open}_{\beta',r'}$ 为 anchor 的 L1 reference height 与初始 authority publication height 的最大值。验证 anchor 原 direct/covering evidence、完整 block 和必要 ancestry，不能把它重新标记成 target pair。
7. 把导出的固定字段与 packet、同一 target pair 已有记录比较。相同重复包使用同一记录；同一个 pair 的固定字段不同则拒绝。packet 不能替换已经安装的交接。进入前再次对 Manager 当前已处理公共前缀检查有效性。

Force 发生但没有合格后继时，cause、anchor、source context 和新 version 仍已固定，公共解释器保存该 pending boundary。但此时没有完整的已选中交接包，程序不能 Ready；要等首个历史合格位置确定 target region。此间另一次 Force 可以替代 pending boundary，同时沿用保存的前驱 context。

genesis 合成边界按初始 checkpoint 和第 2 节的规则验证，不冒充不存在的 REGULAR、MSIG 或 AUTO 原因。

**Ready 的定义。** 本地 Waiting 程序 $(\beta,r)$ 当且仅当以下条件全部满足时 Ready：

1. 交接验证确定该 pair，且选中的就是本副本所属地区；或认证的 genesis 规则得出同样结果。
2. 精确 anchor 的原 producing pair、height、hash、完整 DATA 与 commitment evidence 已验证。
3. 从这个 anchor 后面继续执行所需的 native state 已恢复，并验证它对应同一个 anchor。
4. 匹配的 finalized authority、固定 quota、两种 reference lower bound 和全部验证依赖齐全。
5. Manager 已处理的公共历史没有关闭或替代这次交接。
6. 持久记录从未执行初始 Enter，也没有 Terminated tombstone。

状态恢复使用应用现有验证接口，不增加新的 execution-trust 假设。缓存 DATA 110 的状态不能使 anchor 固定为 112 的程序 Ready。即使未来 Force 已公告且候选状态全部下载完成，程序仍是 Waiting。每个被选正确成员通过检查后转发完整已验证 bundle。最后的 Ready 检查与 Enter 相对公共重放串行化，并在同一事件内完成。

### 6.6 Activation and bootstrap

**Protocol 4：进入并启动被选 session。** 对固定目标 $(\beta,r)$，副本 $v\in V_{i_{\beta,r}}$ 执行，令 $\Delta=\Delta_{i_{\beta,r}}$。

1. **Enter。** 在 Ready 事件中原子检查并设置持久 initial-entry flag。安装 $A_{\beta,r,0,0}$、$\mathsf{AUTH}_{r,0}$、$\TB_r$ 和两种 opening reference bound。设置 entry lock 为 $Q^0_{\beta,r}$，以 unfrozen 状态进入 $(\beta,r,0,0)$，完成 Ready → Active，只记录一次 $e_v(\beta,r,0,0)$ 并启动原 progress checks。向初始 leader 发送带 lock 和固定交接的 STATUS，处理该 session 的缓冲证明。在发出签名前持久化进入记录。
2. **选择 anchor。** 初始 leader 从自己的 entry 等待 $2\Delta$。如果 view 仍 current、unfrozen，且退役合同允许此动作，则发送 NEWVIEW，精确选择 $A_{\beta,r,0,0}$，附带 $Q^0_{\beta,r}$ 和固定交接。
3. **认证与生产。** 成员执行 Protocol 2 原 ANCHOR-vote 步骤。得到 ANCHOR QC 后，leader 在 DATA height $h(A_{\beta,r,0,0})+1$、physical position 1 启动 Protocol 1。本地失败继续使用原恢复规则。除非 Quit、Completion 或明确退役转换介入，所有这些 bootstrap 动作都在 Active 内发生。

令 $e=\min_v e_v(\beta,r,0,0)$，其中取所有正确成员。如果固定交接持续有效，且没有退役打断 bootstrap，完整 bundle 转发和原区域通信界保证所有正确成员在 $e+\Delta$ 前进入。若 initializer 正确且没有 Quit，NEWVIEW 在 $e+3\Delta$ 前发出，ANCHOR QC 在 $e+5\Delta$ 前到达 leader，第一份 DATA proposal 在 $e+6\Delta$ 前到达所有正确成员。

Ready 和 Enter 是同一事件，所以保留原来的 entry-based 时间界，不增加异步 Ready 队列或调度宽限。进入前的状态传输使用直接传输或已有假设下的认证 Espresso 获取路径；这些 bootstrap 时间界不从状态传输开始计时。

公共 export clock 独立于程序 phase。普通后继已经可选时，window 从包含相应交接的 EXPORT timestamp 开始；Force 立即选中后继时，从触发块 timestamp 开始；此前没有分配 duty 时，从首个合格 selection block 开始。被选程序仍 Waiting 时，公共窗口可能已经运行。下载完成、Ready、进程重启和重复交接都不能重置 deadline。只有尚未分配 duty 时才没有 DATA window。缺少证据会把 replay 暂停在规范位置，不会创造更晚的公共 opening time。

### 6.7 Concurrent candidates and races

设 A 的 session 为 $(7,20)$，terminal DATA 是 110；B 已在 $(7,21)$ 生产，而 $G$ 只到 108。一条未来 MSIG 针对版本 7。B 可以保留旧 Active 程序，同时为候选 $(8,21)$ 预取 108。若 $G$ 推进到 109，准备目标可以改为 109，但旧 Active 程序的初始 anchor 仍是 110。

如果 Force 最终选中 B 并保留 109，之后就只补齐这个固定 anchor 的材料。B 可以在旧 $(7,21)$ 按退役合同收尾期间进入 $(8,21,0,0)$，两个程序不共享锁、votes、entry time 或 timers。

如果 A 的 terminal 110 在 Force 前进入 $G$，export context 可以转到 B，候选则变成 $(8,22)$，从 B 后面选区。设顺序为 A、B、C、D；如果未来 OFF(C) 已在 Force 选区前生效，可以把尚未固定的候选改为 D。若 Force 最终保留 B 的 112，正式目标就是 D 的 $(8,22)$，anchor 的原生产 pair 仍为 $(7,21)$。D 必须验证 112 的状态，才能开始 DATA 113。此前准备过 $(8,21)$ 或 109 的状态不会获得进入权。OFF 不能改写已固定的 REGULAR 交接。

terminal receipt 先到时，节点可以准备或启动旧版本常规后继；随后验证 Force，就取消失效的未进入目标，或请求已进入程序退役。Force 先到时，晚到 receipt 作为区域证据保存，不能复活已关闭目标。节点可以短暂持有不同的投机程序，但相同已验证公共前缀必须给出相同公共状态和 Force boundary。

每份 fetch 回调保留原 target 和 anchor 身份。为 $(7,21)$ 的 110 请求的回包不能使从 108 开始的 $(8,21)$ Ready。旧候选回包可以缓存并在其他用途下重新验证，但不能覆盖 fixed anchor。重复 receipts、交接包和回调不能再次 initial Enter。

AUTO 已提前切换后，过期 MSIG 不会关闭新版本，也不会应用其附带配置。Manager 丢弃的是过期的准备依据，不能因此使另一份已经固定的交接失效。

### 6.8 Graceful close and termination

公共关闭与程序退出是不同事件。Protocol 5 立即撤销旧版本继续全局追加的权限；Protocol 3 再向本地已进入程序提交带已验证原因的 `RequestClose`。请求幂等，只记录 `close_requested`，它不是 Quit，也不是 terminal Completion。

哪些新工作停止、已有 timer 和消息怎样处理、何时允许 `FinishClose`，由正文的退役合同统一规定。这个 Manager 接口不新增关闭时间假设或固定退出 deadline。新 session 无须等待旧程序完全退出。

对尚未进入的程序，目标确定失效后可取消进入与任务并记录 Terminated，它没有本 session 已启动的区域 timers 或 votes 需要收尾。Completed 程序已按原 Completion 停止 core。Active/Frozen 程序必须由退役合同授权 `FinishClose`。本地 timer 列表为空、公共版本已经不同，或本地看到了 terminal block，都不能单独作为授权。

所有资源释放都要求持久退出记录和责任记录。必须保留的 committed DATA、receipts、regional evidence、anchors 和可恢复状态继续遵守既有 retention 规则。未完成的完整结果和发布重试移交给 Publish Layer；被丢弃 suffix 中的 pending requests 继续等待重提。存储和发布服务确认接管责任后，程序才能释放自己的最后一份必需副本。

晚到消息可以更新这些服务，不能重新激活逻辑 engine。Terminated 不撤销区域 commitment，不删除 $G$ 中 DATA，也不授予 stake 提款权。
