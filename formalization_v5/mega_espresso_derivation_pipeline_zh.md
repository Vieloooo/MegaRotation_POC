# mega-espresso 派生与 cursor 恢复

这份说明只讲完整状态变化：记录怎样被选中，cursor 怎样前进，实际链怎样执行，override 怎样跳过缺口，最后怎样恢复普通生产。
依据当前本地 mega-espresso 草案；这些规则仍标为不稳定，不是已上线行为的验证报告。

## 1. 先分清 cursor 与实际派生链 D

**cursor 管“记录和提交处理到哪里”；D 管“实际执行出了哪些块”。两者可以不在同一高度，也不一定保留相同的普通内容。**

下文把 cursor 的 L2 编号简称为 C。
C=108 表示正常提交已经消费记录到 108，下一期待记录是 109；不表示普通 108 一定执行成功。

cursor 还记录两件重要的事：

- **Espresso 扫描位置 p**：从公开日志的哪里接续找记录。
- **前置 origin e**：记录使用的 L1 区块编号，后续被选记录的 origin 不能低于这个锚。它不一定等于实际 D 的 origin。

完整 cursor 也带验证依据和未完成提交状态，但这里不展开格式。

| 状态 | 含义 |
| --- | --- |
| C=108，D=105 | 批次已被 L1 接纳到 108，节点尚未执行到那里 |
| C=108，D=120 | 后面没有可用普通批次，D 已按规则补到 120；提交扫描仍停在 108 |
| C=125，D=125 | 编号相同，仍可能有一些高度在 D 中是 deposit-only |

**记录如何选择？**从 cursor 规定的位置，按 Espresso 最终化顺序，为下一个期待高度取第一份完整、合法的记录，再找下一高度。
没有合格 109，正常扫描不能直接从 121 开始。
同一扫描里，一个高度选中后，不能随意换成后来另一份记录。

例如起点 C=D=108。
Espresso 有旧记录 109–125。
D 后来变了，也不会让这些旧记录自动消失；它们仍可能需要被提交、消费，或由合法 override 公开跳过。

出处：[Cursor](../megaeth_specs/specs/block-certification/certifier.md#cursor)、[记录选择](../megaeth_specs/specs/block-certification/block-records.md#canonicality)。

![图1：从记录、接纳到实际执行。cursor 的推进不等于 D 已完成执行。](/Users/jin/projects/megaeth_sequenser_rotation/formalization_v5/figures/mega_espresso_derivation_pipeline/figure_1_pipeline.png)

*图1：从记录、接纳到实际执行。cursor 的推进不等于 D 已完成执行。*

<!-- pagebreak -->

## 2. 普通派生和缺批次时的补位

仍从 C=D=108 开始。
batcher 交付选中记录 109、110。

| 步骤 | 动作 | cursor 的 L2 编号 | 实际 D |
| --- | --- | --- | --- |
| 1 | 认证方核对交付内容与选中记录；batcher 提交 L1 | 108 | 108 |
| 2 | Inbox 接纳，发出 CursorAdvanced | 110 | 可以仍为 108 |
| 3 | 派生节点取得完整数据，逐块检查和执行 | 110 | 条件满足时到普通 110 |

**派生 109 时，父块必须是实际 D[108]。**
节点从规范 L1 历史确定本块的 origin、应包含的 deposits 和系统输入，把这些放在批次提供的有序普通交易之前，再执行。
它不照搬旧生产轨迹的状态根。

例如应包含 deposits d1、d2，普通列表是 u1、u2：

`[L1 attributes, d1, d2, 适用系统交易] 接 [u1, u2]`

实际结果有三种：

- **条件合格、完整块可执行**：得到普通 109。
- **命中规定的替换条件，或完整块执行无效**：得到同高度 deposit-only，再检查 110。后续内容不是一律丢弃。
- **必要数据未知或取不回来**：等待，不能猜出一个空块。

deposit-only 仍要执行 deposits 和系统输入，所以也有真实状态变化。
普通交易正常 revert 不等于整块无效。

**如果根本没有可用 109 批次呢？**
相关 sequencing window 尚未处理完时，D 等待。
窗口完整处理后仍无可接受批次、且原生条件满足，才补 deposit-only 109；后续高度按同样规则推进。
窗口按 L1 origin 和 L1 区块位置判断，不是从本地发现故障起开一个统一倒计时。

因此可能出现：

`C 仍为 108；D 已是普通 108 + deposit-only 109–120。`

**D 补位不会自动推进 cursor。**这就是恢复时需要单独处理“记录扫描接续”的原因。

出处：[派生规则](../megaeth_specs/specs/rollup/derivation.md)、[交易前缀构造](https://specs.optimism.io/protocol/derivation.html#deriving-the-transaction-list)。

![图2：同一起点的两种后续。D 自行补位不会推进 cursor。](/Users/jin/projects/megaeth_sequenser_rotation/formalization_v5/figures/mega_espresso_derivation_pipeline/figure_2_progress.png)

*图2：同一起点的两种后续。D 自行补位不会推进 cursor。*

<!-- pagebreak -->

## 3. override 到底改什么 怎样执行

起点是 **C=108、D=120**。
admin 决定放弃未完成的旧范围，执行 emergency override，目标 **N=125**。

**admin 提供的不只是 125，还要提供合法的 Espresso 扫描接续位置、前置 origin 和验证锚。**
目标编号必须大于旧 C；但这些字段是否与真实后续记录相容，不能仅靠这个大小检查保证。
原始 mega-espresso 是 admin 提供目标；你们 Part 1 中“RESUME 让合约计算 N”是另外的扩展。

执行顺序如下：

1. **L1 记录控制。**Inbox 保存新 cursor，发出 CursorSet。后续认证从新位置接续，期待记录 126。
2. **先处理控制之前已经接纳的输入。**派生节点取回并处理其必要数据与完整批次；不能用 override 跳过未知的已接纳内容。
3. **放弃遗留的未完成提交任务。**不再等后来的片段来完成已放弃的旧任务；这不撤销此前已经完整处理的块。
4. **看实际 D 到了哪里，再补缺口。**若仍到 120，就执行 deposit-only 121–125，不额外等窗口结束。
5. **从实际末端继续。**后续批次在真实块和状态上检查，不把数字 125 当作可信 parent 或 state root。

| 处理此前输入后，实际 D 到哪里 | override 目标 | 对 D 的效果 |
| --- | --- | --- |
| 120 | 125 | 保留至 120，只补 121–125 |
| 125 | 125 | 保留已有前缀，不重做 |
| 128 | 125 | 保留至 128，绝不拉回 125 |

补位使用实际 parent、规定的 L2 时间戳和控制位置允许的规范 L1 输入；节点自己执行得到状态。
**cursor 里的 origin 不能替代真实 origin 选择。**

同一 L1 历史中，已经确定的普通块或 deposit-only 前缀都必须保留。
CursorSet 不删除 Espresso 历史，不提交普通交易，不赋予跳过记录有效地位，也不单独赋予填充块最终性。

它使两条路径接续：**认证扫描换到新的公开起点；实际派生补齐尚缺的部分。**
二者不是“把 D 的链头直接设为 N”。

出处：[override 接口](../megaeth_specs/specs/block-certification/batch-inbox.md#emergency-cursor-override)、[override 的派生顺序](../megaeth_specs/specs/rollup/derivation.md#emergency-cursor-overrides)。

![图3：同一目标 125 下，实际 D 末端不同，填充范围也不同。](/Users/jin/projects/megaeth_sequenser_rotation/formalization_v5/figures/mega_espresso_derivation_pipeline/figure_3_override.png)

*图3：同一目标 125 下，实际 D 末端不同，填充范围也不同。*

<!-- pagebreak -->

## 4. 用长停机的同一个例子走完恢复

**共同起点：**Espresso 恢复后，旧记录 109–125 已完整、合法且可读；C=108，实际 D 已用 deposit-only 填到 120。
以下两条是不同选择，不能串成一次执行。

### 路径 A 不 override 继续处理旧记录

| 步骤 | 做什么 | C | D |
| --- | --- | --- | --- |
| 1 | 提交旧记录 109–120，让扫描通过已填过的范围 | 120 | 保留实际 120 |
| 2 | 继续交付 121–125，逐块按实际状态处理 | 125 | 处理到 125，可混合普通与 deposit-only |
| 3 | Sequencer 跟随实际 D[125] 生产新 126，Publisher 发布对应记录 | 125 | 125 |
| 4 | 新 126 能被选中、及时交付并成功执行 | 126 | 普通推进到 126 |

第 1 步的旧批次不能把已填过的高度变回普通块，但可以推进 cursor。
第 2 步里，旧内容仍可执行就保留；依赖丢失交易等原因使整块无效时，再替换。
**这就是原规格的 drain：继续消费旧范围，而不是把所有旧结果一律扔掉。**

这里假定各段数据、接纳和派生条件满足；已填过的旧范围与待执行的新范围分开交付。
若后续高度也已过期，就继续按实际结果接续，不能保证 126 一定成功恢复普通内容。

### 路径 B override 跳过旧范围

admin 提供目标 N=125 和相容的新扫描位置。
派生先处理控制之前的输入，再只补实际缺少的 121–125。
结果为 **C=125、D=125**，其中本例新填的 121–125 都是 deposit-only。

Sequencer 从实际 125 的状态生产新 126，Publisher 发布，认证和交付成功后，D 才接上普通 126。
与路径 A 相比，这条路径主动放弃了 121–125 原本可能保留的普通内容。

**扫描位置也必须接对。**
若扫描仍会先选中旧轨迹的 126，就不能假定随后另发一个新 126 会自动替换它。
admin 必须提供符合记录一致性要求的接续位置；不能把改位置当成任意挑选另一份普通内容的许可。

**D 是否必须停下来等？**
D 会在尚未满足补位条件时等待，但 CursorSet 不额外冻结它。
若新普通 126 交付前，D 已按规则确定了 deposit-only 126，新普通块不能覆盖它。
所以恢复还须满足首个新块的剩余窗口和执行条件；“Espresso 活了”或“已经选定 125”都不保证完成恢复。

出处：[S4 恢复分工](../megaeth_specs/specs/block-certification/recovery.md#s4-a-height-settles-as-a-deposit-only-block)、[S9 Espresso 故障](../megaeth_specs/specs/block-certification/recovery.md#s9-espresso-liveness-failure)。

![图4：继续旧记录与 override 的完整接续，及新 126 的交付条件。](/Users/jin/projects/megaeth_sequenser_rotation/formalization_v5/figures/mega_espresso_derivation_pipeline/figure_4_recovery.png)

*图4：继续旧记录与 override 的完整接续，及新 126 的交付条件。*

<!-- pagebreak -->

## 5. 其他 override 情况与无法跳过的边界

### D 已到 128 却把 cursor override 到 125

结果是 C=125、D=128：认证下一期待记录为 126，派生下一高度为 129。
不能直接让一笔从 129 开始的普通提交跳过认证所缺的记录。

若存在可选且可接纳的 126–128 记录，可先提交这些记录让 C 追到 128，再另行提交 129；已确定的 D[126–128] 不改回去。
或者在存在合法完整 cursor 时，再 override 到 128 或更高目标。
**两条路径都不能回退 D，也不能只根据当前最新 D 自动改 cursor。**

### override 的 origin 或扫描位置错了

例如 cursor 的前置 origin 锚是 L1 编号 1100，实际 D 的 origin 却只有 1000。
新记录忠实使用实际链允许的 1000 或 1001，可能因低于扫描锚而根本无法被选中。
即使 Sequencer 已跟随 D，认证仍会卡住。

需要存在一个更高目标编号、且扫描位置与 origin 等字段全部相容的恢复 cursor。
原接口不能在相同 L2 编号只修正字段；再次 override 必须向前，可能再放弃普通内容。
没有合法更高位置时，不保证恢复；编号已到类型最大值时，现有接口无法再修。

### 三种容易与 emergency override 混淆的变化

| 情况 | 实际规则 |
| --- | --- |
| 首次启用认证，首个高度 H=121 | 成功 bootstrap 的 L1 位置固定切换。先处理此前输入；若 D 只到 118，就补 119、120，再处理认证 121。初始 cursor 不证明块 120 的状态 |
| 未交齐的批次任务超时后重开 | 在允许接手和重开的条件下清理未完成任务，从原期待高度重新交付；不因此跳过 L2 编号，也不是一次 admin override |
| L1 重组移除提交或 CursorSet | 对可重组后缀撤销相应效果，按替代规范历史重放；不是 admin 获得回退已最终化历史的权限 |

**已接纳数据不可取回，是另一条硬边界。**
例如 C 已到 125、D 仍到 120，原因是必要 payload 未知。
后面即使出现目标 130 的 CursorSet，也不能跳过这笔输入直接填到 130；须取得与承诺匹配的数据，才能依顺序继续。
现有规则没有给这类等待定义一个“超时后当空”的恢复出口。

原始 mega-espresso 的 cursor override 解决记录扫描和派生接续。
它不负责你们轮换层的新 session 身份、选区或 Ready / Enter；这些不能从 N=125 自动推出。

出处：[激活](../megaeth_specs/specs/block-certification/activation.md#derivation-transition)、[错误 cursor 修复](../megaeth_specs/specs/block-certification/recovery.md#s8-a-previous-override-left-the-cursor-at-an-unreachable-position)、[数据等待](../megaeth_specs/specs/rollup/data-availability.md#batch-commitment-resolution)。
