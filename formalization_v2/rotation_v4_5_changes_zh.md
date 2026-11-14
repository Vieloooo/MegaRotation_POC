# v4.5 修订与全文通读记录

## 版本与范围

- 基线：sync_hotstuff_rr_part2_rotation_v4_4.tex。
- 新稿：sync_hotstuff_rr_part2_rotation_v4_5.tex。
- v4.4 源文件保持原样；本次没有修改 Part I。
- Section 6 重写为三个小节；前五节保留原结构，只做新启动、退出语义必需的局部修改及少量措辞修正。
- 主稿仍为英文，使用直接、简短的表述。本文件用中文说明修改和检查结果。

## Section 6 现在说什么

### 6.1 What the Session Manager keeps

Manager 跟踪三类进度：G、当前生产分支的区域结果 R、已验证的 updates。
同时保存程序的固定启动信息、Enter/终止记录，并准备精确 anchor 对应的执行状态。
程序只有 Assigned、Running、Terminated 三种生命周期状态。

### 6.2 Handoffs and session management

- Regular：完整 terminal receipt 确定下一 session；anchor 是 terminal DATA，不等待它进入 G。
- Force：使用 Protocol 3 确定的新 session 和保留的 G tip；target index、固定 map、reference floor 的规则不变。
- Protocol 4：Track、Create、Start、Force-close、Complete、Serve and persist。
- Force-close 只在首次验证有效激活后运行，终止整个旧 version 的本地 programs，取消 BFT timers，并禁止迟到消息或重启重新创建它们。
- 正常退出继续使用 Protocol 1 的 Completion。Force-close 不制造 terminal receipt，也不假称正常完成。
- 保留数据和已有结果的服务责任，但删除独立的 close-bundle、closure duty table、timer-drain、c_v 和 4 Delta 等待。

### 6.3 ReadyBundle and direct startup

保留精确 handoff/state/authority 验证和完整 ReadyBundle。
Protocol 5 只有 Enter and relay、Propose directly 两步。
每个新 session 的首次启动都取消 STATUS、NEWVIEW、ANCHOR vote、ANCHOR QC 和 bootstrap 等待。

首块的规则是：

\[
B_{\rm parent}=A_{\beta,r,0,0},\qquad Q_{\rm parent}=Q^0_{\beta,r}.
\]

之后的 DATA、普通 term rotation 和 recovery 使用原有规则。
特别是 recovery 的 STATUS、等待和 ANCHOR QC，以及用于完成 terminal DATA 的 DRAIN block，都保留。

## 前五节的局部联动

1. 明确完整 terminal receipt 才触发 regular handoff。
2. 在 parent-selection 表中加入 generation-zero 首块使用 Q^0 的行。
3. initial admission 不再要求 NEWVIEW；所有 recovery generations 仍要求 ANCHOR QC。
4. Program guard 覆盖 timer callbacks；verified Force-close 可以取消剩余 timers。
5. 区域一致性陈述限定为 Force 前的适用执行，删除对废弃后缀退出期间的无条件保证。
6. 删除已经不用的三种 view 别名。

核对结果：Section 2 的模型与假设、Protocol 3 算法主体、Protocol 2 recovery 算法主体均与 v4.4 一致。

### 一个需要单独指出的签名绑定

在 regional BFT 签名上下文中明确加入初始 anchor 的 hash；RawFINAL 检查和 kernel soundness 证明同步检查这一项。

原因是：v4.5 不再保证 Force 后废弃后缀的区域一致性，不能继续无条件假设同一 (beta,r) 在所有迟到节点处只有一个 incoming anchor。
显式绑定 anchor 可以防止把某个启动上下文中的 FINAL 套用到另一个上下文。
这是签名上下文的明确要求，不只是语言润色；公开显示的简写仍省略公共上下文字段。

## 全文同步处理

- Section 7 明确其 Force handler 是 Protocol 3 prelude 的展开，不重复执行 update。
- 验证 matching update 和激活上下文后即可通知 Manager 停止旧程序；不等待 successor authority 到齐。
- 全局前缀证明依赖共同输入、确定的接收规则和 version cutoff，不再依赖 Force 期间旧区域日志的一致性。
- 本地程序隔离命题只承诺本节点固定 handoff、一次 Enter 和跨 session 隔离。
- 删除原 synchronous-stop 子节的三个命题及整份附录证明。
- storage appendix 删除关闭转发和 drain 要求，继续保留所需数据与已有发送任务。
- 两张 handoff 图改成 ReadyBundle 后直接提首块。
- 可选控制策略使用控制器实际记录的观察时间，并补全其处理时限前提；其示例证据不再把废弃后缀的分歧直接当作诚实节点违规。

## 通读时逐项核对的执行

| 场景 | 核对后的规则 |
|---|---|
| Terminal receipt 早于 G 到达 | 正常 successor 可启动，仍使用固定 S_beta 和 reference floor |
| 第一个 DATA 早于某成员 Enter 到达 | 缓存；完整 ReadyBundle 使其满足 Ready 后才投票 |
| 引用下界对应的 header 尚未出现 | 可 Enter；首块等待合法 finalized reference |
| 初始 leader 不工作或产生有效 equivocation | 原有 progress checks、Quit 和 recovery 继续适用 |
| Force 在准备期间被验证 | Assigned 被取消，关闭标记禁止之后 Enter |
| Force 在 timer 尚未到期时被验证 | 取消 timer，不生成该 timer 的新 FINAL |
| 普通 term rotation | 原有 commit timers 继续运行 |
| 已有多个 speculative regular successors | 关闭整个旧 version，覆盖所有这些 programs |
| 重复 Force 证据、重复 bundle、迟到回调 | 不重复截断 R，不重复 Enter，不重启旧程序 |
| 更新尚未激活或 target 已 stale | 不停止当前 program |
| 新 version 复用 r | authority index 沿用，但 program、locks、votes 和 timers 独立 |
| 所有区域均不合格 | 保持 PENDING，不创建 program |
| Force 后补到旧 receipt | 可保存历史证据，不能增加旧 version 的 G |
| 不同 incoming anchors 的同 pair 证据 | FINAL 的初始 anchor hash 检查阻止混用 |

## 验证结果与边界

- 已通读新稿的定义、协议、示例、保证、发布规则及附录，并检查首次启动和 recovery 的区别。
- 旧 retirement 标签和引用已清除；没有缺失引用或重复标签。
- LaTeX 成功编译；最终日志无 undefined-reference、Overfull/Underfull 或其他 warning。
- 初稿 PDF 为 49 页，已检查全篇页面缩略图，并放大检查 Section 6、两个协议框和两张 handoff 图；图中文字重叠已修正。新增下述概览图后为 50 页，已重新编译并检查新图及相邻页面。
- 已检查新稿和新增图文件的 diff whitespace。
- 这是人工论证与文档一致性检查，没有运行协议模型检查器或机器证明。
- 同步启动结论仍依赖完整 ReadyBundle 的传输、验证和状态接入属于 Delta 处理界；任意离线恢复不在该界内。
- v4.5 明确保护 G，不承诺 Force 废弃后缀在退出期间仍保持区域一致性或必然完成。

## Section 6 开头新增概览图

新增 concurrent-handoffs.tex，并在 Section 6 开头作为 Figure 1 展示：

- A 的 terminal receipt 先使 B 从 110 启动 regular session (7,21)；此时 targeting version 7 的 Force 尚未激活。
- Force 在 J_501 激活时，G 仍只到 108，export session 仍是 A 的 (7,20)。
- 旧 version 7 的 109--115 不能再进入 G；B 验证 Force 后终止旧 program 和 timers。
- 新 B session (8,21) 从 108 及其精确状态启动，经过 ReadyBundle、Enter 后直接提出新 DATA。
- 若先验证 Force，再收到旧 terminal receipt，则不能再启动旧 session。

Section 6 标题、例子说明和用户概括的四点核心规则排在 PDF 第 27 页，概览图接在第 28 页。四点分别是：跟踪 G/R/updates、regular handoff、Force handoff、Force-close；只整理英文表达并明确 receipt 和已验证激活等既有触发条件。新增内容之外，前五节和 Section 6.1 起的源码保持原样；协议语义未变。编译无警告，引用无缺失或重复。
