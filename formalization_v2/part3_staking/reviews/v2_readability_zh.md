# Part III v2：修订后可读性复核

审阅对象：`versions/part3_v2.tex`，按正文顺序重新通读；以下为 TeX 行号。主任务已确认 PDF 为六页。本轮只检查表达、自包含性和组织，不判断协议是否有效，不修改正文。

## 结论

**主体已明显达到清楚、可顺读的程度；可读性有条件通过，仍有三处必要的局部澄清。** 不需要再次调整整体结构。三个模块现在可见，读者能够顺着“固定输入与选择—资金变化与释放—authority 的生成和使用”理解全文。上一轮最主要的理解障碍已解决；剩余问题集中在少量首次出现的外部术语，以及一句范围过宽的表述。

## 上一轮六项问题复核

| 上轮问题 | v2 结果 |
|---|---|
| 首次使用的记号与借用术语 | **大部解决。** `h`、`lambda` 在 44–45 行，`V_i`、`w_o` 在 72–74 行，`W` 在 107–111 行，`j` 与 DRAIN 在 187–189 行解释；QC、quit proof、Ready/Enter 也有足够的功能描述；原未定义的 `q` 已删除。尚余下述局部术语问题。 |
| 两个 anchor 与 plan/authority 的介绍顺序 | **解决。** 50–58 行先并列写出关系及允许的行为，§3 再给完整步骤，顺序合理。 |
| 实际余额与冻结权重区别 | **解决。** 72–74 行的 100/70 例子短而有效；后文能直接沿用这个区别。 |
| 基础排班与实际 leader 的关系 | **解决。** 115–127 行直接定义实际 leader，先解释 generation 的禁任集，再给过滤后的成员表。§3 只补记录的来源与生效步骤。 |
| 取消申请路径 | **解决。** 160–166 行明确只有未进入任何冻结快照的 pending join 可由 owner 取消，exit 不可撤销。 |
| 三模块组织 | **解决。** §1、§2、§3 分工清楚；recovery 与 Force bootstrap 已集中在 authority 模块，不再夹在资金规则里。 |

## 必要的局部修复

1. **把 G 的定义移回首次使用之前。** `G` 在 218 行已经承担提款动机的核心作用，而“Part II 对 finalized Espresso 与 L1 controls 重放得到 G”的解释直到 361–363 行才出现。建议在 §1 的 session/receipt 说明后加一句：“Part II derives the accepted global DATA prefix G by replaying finalized Espresso inputs and L1 controls; Force retains this prefix.” 末尾 External history 可以保留其信任假设，不必重复整段定义。没有这句，首次阅读者只能从上下文猜测 G 是哪种最终历史。

2. **给两组借用术语加极短的功能释义。** 197–203 行的 recovery opening 是罚没判断对象，首次出现前应简述它在 Part II 中指什么；不能只靠 “one opening extends the other” 反推。242–243 行的 NEXT/PENDING tail、control cursor 也是首次出现的内部状态名称：分别用短括注说明它记录哪一段未完成交接状态、控制日志已处理到哪里。保持 Part II 的实际含义即可，不需要在本文增加新接口或复述算法。这里要求的是读懂已有字段，不是扩大协议内容。

3. **收窄 “No plan or BFT state crosses the closed branch boundary”。** 319–324 行先让新 session 从保留的 G 末块启动，随后说没有任何 BFT state 跨过边界，容易让读者以为保留 anchor 对应的执行状态也不能沿用。请明确这里所排除的是旧 session 的 authority、运行状态或其他具体协议状态，并与新 session 使用保留 anchor 的执行状态相协调。此处只需把原本意图说准确；本轮不裁定哪些具体状态应保留。

## 可选的收紧

- **例子已足够，无需继续添加。** 129–135 行现在同时解释选择参考高度、实际执行高度和新快照不覆盖既定 plan，已经覆盖原例子欠缺的重点。
- **轻微减掉结尾的重复防御性措辞。** 137–140 行已经限定公平性，369–378 行再总结公开排班、提款及服务代价是合理的；350–358 行最后一句 “every ever-hidden offense” 则显得别扭，可简化为“Liability coverage remains conditional on timely evidence.” 不要因此删掉前文关于隐藏证据的条件。
- **少数术语可更直白。** 79 行的 “accepted lifetime” 可改成 authority 仍有效的期间；94 行的 “latest local head” 可明确为读者本地看到的最新 L1 高度；109 行补 “from z” 能让 WeightedRand 的输入与取样步骤直接对应。这些不再构成主流程理解障碍。
- **页数维护。** 本轮必要修复均可用短句或括注完成；优先替换或移动已有句子，修订后再检查六页，不需要降低字号。

## 最终可读性判定

**有条件通过：三模块组织、正常流程和上一轮六项问题已基本修复；补齐 G 与两组术语的首次释义，并澄清跨分支 state 的范围后，可通过本轮可读性检查。** 这些都是局部编辑要求，不能据此推断协议有效性或安全性。
