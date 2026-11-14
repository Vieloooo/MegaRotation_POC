# Part III 迭代记录

正文语言：英文；修改记录：中文。版本上限为 v3。每份正文 PDF 均以 6 页为上限，日志和审阅记录单独保存。

## v1：从中文大纲写成可顺序阅读的正文

状态：已完成可读性校对、协议问题检查和逐页版面检查；作为修订前的原稿保留。

### 保留的设计

- 长 L1 staking epoch，周期快照在指定高度激活。
- `(0,2)` 用 `(0,1)` 的初始 anchor，即 `(0,0)` 的 terminal DATA，选权重与 seed；执行仍从 `(0,1)` 的 terminal DATA 继续。
- Region 由 multisig 的 `randList(beta,r)` 加 status map 选择；区内 leader 按权重和公开高度哈希派生。
- Session 内投票成员、权重与 quota 不变；普通接棒不增加 L1 往返。
- 入场／退出借鉴以太坊的队列和分阶段流程，冻结期按 Part II 与 Espresso 的实际事件安排。

### 大纲中的问题、正文补法与代价

| 问题 | 补法 | 正文中的改变与代价 |
|---|---|---|
| `VerifyRetirement` 只有目标，没有具体的“以后不能再分配”规则 | 复用有效 Force 关闭全部旧分支职责，同时抬高永不回退的最低快照编号 | 配置选择显式取 `max(branch minimum, epoch(anchor L1 reference))`。普通情形保持原规则；退出批次之后旧 anchor 不能复活已退出资金。最终提款依赖一次批量 Force，可能丢弃 G 以外的 speculative 后缀 |
| 只看 G 无法枚举已经跑到前面的 session | 用 Part II 的整分支关闭作为退休边界 | 不要求 L1 追踪每次普通交接；提款时验证一次真实生效的 Force 及全局重放 |
| Force 复用 r 与按新 session 派生配置相冲突 | Authority 绑定 `(beta,r)` 和配置 ID | 在接入说明中明确替换 Part II 的 r-only authority 规则，既有 Part II 文件保持不改 |
| Slash 只有扣款，缺少未来 leader 排除 | 用完整 quit proof 创建唯一 L1 recovery record，固定当时处罚集合 | Recovery 保留 Part II 已有的 L1 authority 等待；投票权重不变，不允许空口预建无限多代授权 |
| 用固定天数或平均 session 时长决定提款不可靠 | 在 L1 接受有效退休边界后，再留完整证据窗口 | 时间预算区分关闭交付、结果发布、调查提交和 L1 slash；证据服务与时钟换算作为明确假设 |

### 本轮流程与结果

- 写作完成：`versions/part3_v1.tex`。
- 可读性校对：见 `reviews/v1_readability_zh.md`。主线可读，但缺少记号定义、初始 authority 的完整叙述，模块分布需要调整。
- 问题检查：见 `reviews/v1_protocol_zh.md` 和 `reviews/v1_authority_zh.md`。共确认四类必须补齐的接口／说明缺口，没有发现需要推翻主设计的 Critical Error；这些结果不等于整篇共识安全证明。
- 编译与版面：6 页，约 2794 个提取词，零 overfull 和未定义引用；六页已逐页检查。详见 `reviews/v1_layout_zh.md`。

## v2：补齐接口，按三个模块重排

状态：已完成再次顺读、问题复核和逐页版面检查；两个新暴露的队列准入条件及三处局部解释进入 v3 修订。

### 必须修补的问题与实际修改

| 问题 | 判断 | v2 的修补与位置 |
|---|---|---|
| Genesis 没有新 authority 的基例 | 必须补齐：首个 session 没有前驱，不能套 regular 公式 | §3.1 明确 genesis anchor、初始配置／策略／status／minimum、L1 引用条件和首个 boot 选择 |
| 强制退役没有明确写 exit record 和 e_x | 必须补齐：有剩余余额的强制退役者缺少提款入口 | §2.1 对所有自愿或强制移除的 incumbent 记录首次排除 epoch，并禁止后来快照重新纳入 |
| TB/F/Delta/K 的公共来源不明确 | 必须补齐：不同参数会导致不同 terminal height 或证书判断 | §1.1 用 genesis 固定 TB、K、各区 Delta 和故障比例；每个 session 从固定权重导出 F 并绑定这些参数 |
| (q,G,beta,m) 不足以继续重放 | 必须补齐：缺少 NEXT/PENDING tail、status、控制游标等 | §2.3 要求 checkpoint 绑定完整重放状态、源承诺、控制状态和确切 Espresso 位置 |

### 可读性与小范围完整性修订

- 六页按三模块排列：§1 数据和选择接口；§2 stake 生命周期、罚没和提款；§3 regular / recovery / Force 的 authority 使用及保证。
- 在首次使用处定义 h、lambda、成员集、冻结权重、fresh position j；并排解释 E 与 A。
- 统一给出实际 leader 的公式，明确 g=0 使用 snapshot cutoff 禁任集，g>0 使用已最终化 REC；投票权重始终不变。
- 明确尚未纳入任何快照的 join 才能由所有者取消，exit 请求不可撤销。
- 规定自愿出队额度按上一快照权重计，加入按锁定余额计。
- 提款 clearance 首次记录后保持不变，重复证明不能回拨或重置等待时间；提款只支付一次。
- 补全 100/120 的例子：120 即使引用新配置，也不重算已经为 (0,2) 固定的计划。
- 解释时间预算中的两个 T_E 分别负责关闭通知和结果交付；修正表格左列的词间空隙。

### 保留为可接受取舍

- 公开高度哈希不是抗操纵的随机信标，不补造公平性定理。
- 提款需批量 Force，multisig 不行动时不保证固定退出时间；Force 可能丢弃 G 以外的生产后缀。
- 证据及时发现／提交、外部验证和时钟换算仍是显式条件，不宣称有限窗口覆盖永远隐藏的证据。
- 保留 L1 recovery record；不为去掉这次既有等待而改写 BFT 恢复协议。

### 本轮检查记录

- 正文：`versions/part3_v2.tex`。
- 重新编译成功：6 页，11pt 正文字号。
- 再次顺读校对：`reviews/v2_readability_zh.md`。原六项主要问题已解决，剩余 G 首次定义、少量术语和跨分支状态措辞三项局部澄清。
- 问题复核：`reviews/v2_protocol_zh.md` 与 `reviews/v2_authority_zh.md`。旧接口缺口已闭合；须补 Join 的最低 bond 和 Exit 的合法申请状态。
- 实际版面：6 页，约 2787 个提取词，11pt；六页逐页检查通过，零 overfull 与未定义引用。

## v3：最后一轮局部修补，停止迭代

状态：最终版。已完成修订后复读、必要问题关闭复核和六页视觉检查；不再派生 v4。

### 问题、补法与实际修改

| 问题 | 是否必须处理 | v3 修改 |
|---|---|---|
| Pending join 或已退休 position 可能进入 exit FIFO，导致上一快照无可计费权重 | 必须补齐 | Exit 仅允许已纳入冻结快照、且没有 exit record 或 pending exit 的 position；需要 owner 授权 |
| 新 join 未显式检查 minimum bond | 必须补齐 | Join 明确要求至少最低 bond 且不超过准入额度 |
| Mandatory retirement 的旧请求可能占用 voluntary 预算 | 必须澄清步骤顺序 | 先清掉被强制移除者的队列项，再选择剩余成熟 FIFO 前缀 |
| G 首次出现时未定义 | 自包含性必须修补 | 在开头说明 Espresso/L1 重放得到 G、区域可跑在 G 前面、Force 保留 G |
| Opening、NEXT/PENDING 和 control cursor 对顺读读者不直观 | 自包含性局部修补 | 用“恢复起始块”“下一 DATA 的 owner 或未分配边界”“控制日志处理位置”解释 |
| “No BFT state crosses” 可能误伤继承的执行状态 | 必须收窄范围 | 只禁止旧 plan、votes、locks、timers 沿用，保留 anchor 的已验证执行状态照常使用 |

同时在首次介绍 terminal DATA 时写出 `h(A_s)+TB`，避免读者靠后文推断 session 结束高度。没有另加机制，也没有扩大保证范围。

### 最终检查结果

- 可读性：`reviews/v3_readability_zh.md`，通过；没有剩余必要表达修订。
- 问题关闭：`reviews/v3_protocol_zh.md`，本次复核范围内通过。此前必要缺口已闭合，未发现新的 Critical Error 或未闭合 Justification Gap；这不是对整个 Part II、合约实现或部署参数的全面认证。
- 版面：正文与参考文献合计 6 页，11pt，约 2906 个提取词。六页逐页检查通过；零 overfull、未定义引用、裁切或重叠。
- 版本：v1、v2、v3 的源码和 PDF 均保留，当前 `main.tex` 指向 v3。
- 仍明确保留：公开排班的可预测性；提款对批量 Force 的依赖；有界证据服务和时钟换算前提；固定历史权重下的 BFT 故障假设。没有把它们包装成已经消失的问题。

后续只有发现必要修改才产生 v3，到 v3 为止。排版调试不额外增加内容版本；每轮保留原正文、审阅结论与中文修改记录。
