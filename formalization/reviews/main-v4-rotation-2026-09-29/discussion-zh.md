# v4 阅读说明：正常区域内轮换与 HotStuff 实际实现

对应 [main_v4.tex](../../main_v4.tex) 新增的 Section 9。v3 文件及其图没有改动；v4 保留原文中的 quota-BLAME 实验，明确指出原活性证明没有覆盖它。

## 实际链提供了什么启发？

查到的共同结构是：**成功时凭进展证书推进 round，失败时凭 timeout/recovery evidence 恢复；推进 round 与最终确认是不同事件。** 这不意味着任意收到的 QC 都能作为我们当前同步协议的 final receipt。

| 系统 | 本次核对的实现证据 | 对当前稿件的意义 |
|---|---|---|
| Flow / Jolteon | pacemaker 分别处理 QC 和 TC，安全规则区分上一 view 的 QC 与跳过 view 所需的 TC | 正常换 leader 不需要制造故障；但正常路径和恢复路径的安全规则必须一起采用 |
| Aptos | `process_certificates` 明确区分 `QCReady` 与 `Timeout`；普通投票路径可发给下一 proposer；乐观 proposal 仍检查 parent QC | 可以预先组织下一 leader 的工作，但不能省掉所需的认证条件 |
| MonadBFT | 下一 leader 收集 votes；另有 backup QC、timeout 携带 votes 及 reproposal/NEC 机制 | 减少投票 fanout 后，必须处理 successor 隐藏 votes/QC 的风险 |

所有源码链接、commit、行号及官方文章见 [证据索引](README.md#claim-to-source-map)。Flow 和 Aptos 是此次读取的公开仓库快照，不是已经确认的主网部署二进制。Aptos 的 Baby Raptr 有官方生产升级公告；不能只依赖早期 DiemBFT 描述。Monad 的 tail-forking 保护也不能被写成“普通 HotStuff 会回滚已经 finalized 的区块”。

## v4 保留三个层次

**A：先完成固定边界，再初始化下一段。**

引入本地 segment `e`，每段至多 `K_loc` 个 APP，故障恢复 view `v` 不重置段配额。段末固定在同一 application height；达到 proposal 上限后保留 terminal timer，必要时使用 DRAIN。获得 terminal final receipt 后，下一段以该 application anchor 为 checkpoint。旧 leader 不需要单独签署 release。

这先解决“正常轮换不走 BLAME”，但不保证正常边界没有流水线收尾成本。DRAIN 属于旧内部 SMR instance，不能把它从同一条物理日志中悄悄删掉；v4 明确把 application continuity 与 physical-log continuity 分开。

**A+：用 READY/OPEN 优化下一段的初始化。**

在 terminal finality 和数据就绪之后，成员持久化共同起点并发送 READY，权重超过 `2W_i/3` 的 READY 形成 opening certificate。该证书需要新的首个 proposal 验证规则。它与权重超过 `W_i/3` 的 post-commit receipt 含义不同。

`O(delta_R)` 是 receipt 之后 opening delay 的设计目标，不是当前已经证明的界；不能据此删去前面的 `2Delta_R` terminal 等待。READY 与旧 recovery 并发、下一 leader 宕机、进入时间偏差及数据缺失仍是证明义务。

**B：采用本来就支持跨 leader 流水线的引擎。**

如果希望 terminal 尚未 commit 就让下一 leader 接着提出 child，则需要采用一致的 locking、commit、timeout 和 QC advancement 规则。当前 S4 要求 timer 到期时仍在原 view，S1 又要求 current-view parent QC，因此不能只改变 leader 函数。v4 同时给出生产 HotStuff 家族路线与专门的同步 rotating-leader 文献路线。

## 一个不能忽略的 weighted-liveness 问题

当前假设是 faulty stake `< W_i/3`，leader 列表却按身份循环。一个 honest identity 持有 80% stake、三个 Byzantine identities 共持有 20% stake，仍满足这个故障假设，但身份循环里没有连续两个 honest leaders。

因此，如果所选 chained engine 的活性证明需要连续若干 honest leaders，必须另行说明 schedule/leader tenure 如何提供该条件。不能把基于节点数量的原论文结论直接套到此处。这是迁移时需要证明的条件，不是对 Flow、Aptos 或 Monad 已部署配置的攻击结论。

## 阅读成本公式时看清端点

Candidate A 的边界是：terminal proposal → terminal receipt → 同区域证据/数据就绪 → 下一段初始化 → first fresh proposal。

Candidate A+ 只优化最后的 opening 过程。单段有 `q` 批时，均摊 proposal 间隔仍按 `((q-1)I + H_local)/q` 计算。区域内边界与跨区任期边界重合时只进行一次真正需要的 successor opening，不应重复相加。

当前已完成的是设计讨论、源码对照和文档验证；没有把这些候选方案升级为已证明的 normative protocol。
