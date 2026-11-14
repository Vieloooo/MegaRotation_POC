# v4.6 修订记录：只改表述，便于顺序阅读

## 版本与范围

- 基线：sync_hotstuff_rr_part2_rotation_v4_5.tex（含 Section 6 开头的四点核心规则）。
- 新稿：sync_hotstuff_rr_part2_rotation_v4_6.tex；v4.5 源文件保持原样。
- 图：复制到 figures/cross_region_rotation_v4_6/，内容与 v4.5 相同。
- PDF：output/pdf/sync_hotstuff_rr_part2_rotation_v4_6.pdf；逐字差异见 output/pdf/sync_hotstuff_rr_part2_rotation_v4_6_diff_v4_5.pdf。
- 原则：协议、定义、记号、标签、协议框、命题陈述和证明均不变。只改说明文字：先讲思路，先定义再使用，句子更短。

## Overview 的主要改动

1. **摘要**改成三段：两条链（production branch 与 G）、两种 handoff（regular 与 Force）、节点上的 Session Manager。内容与 v4.5 摘要一致。
2. **开头**先说明要同时满足的两个目标：handoff 要快，G 不能回滚。然后指出解决方法是把两条链分开。
3. **新增 1.1 The core idea**（`sec:core-idea`），用一个框列出五点：
   1. 两条链：R 快，G 安全且只增长；
   2. Regular handoff：terminal DATA 有 receipt 就立刻开始，successor 由不变的 map 预先确定；
   3. Force：只能来自 L1 上预先公布的 atomic update，到点后关闭 branch、安装新 map、从当时的 G tip 重启，不能选择保留哪段，也不删除 G；
   4. 一条公开规则计算 G：只接受延伸 G、来自 export session、带有效 regional receipt 的 DATA；
   5. 每个 session 一个独立程序，从固定 anchor 直接启动；验证 Force 后停止旧 version 的全部程序。

   框后用一句话总结整体思路。
4. **Running example** 标出每一步对应第几点核心思路，并加了一张 Force 前后的文字示意图，同时指向 Section 6 的 Figure 1。
5. **1.2** 增加一句说明“本小节补充第 1–3 点”。stale 的含义改为“target 不是当前 version”（v4.5 原意）。rollup DA 明确为本文之外的第四个阶段。
6. **1.3 视图表**的 trigger 一栏改写为直白的句子，并标出各自定义所在的小节。原来未定义的 `REGULAR witness` 改为“terminal receipt 确定 successor，成员 Ready 后进入”。补充定义了此前未定义的 $r_c$（export session 的 turn index）。
7. **1.4** 首次定义 MSIG：委员会签名的 L1 指令，即 atomic branch update。
8. **新增 1.5 How to read this document**（`sec:roadmap`）：说明各节内容，以及每节对应哪几点核心思路。

## 正文的局部改动

| 位置 | 改动 |
|---|---|
| Section 2 开头、2.1 开头 | 加一段说明 L1 的两个作用：保存 authority，保存 control log。branch update 是改变 region map、触发 Force 的唯一途径 |
| Ingesting controls | 拆成两段，说明 replay 只需要三样状态；不再用“Semantically”这类说法 |
| Prelude | 首次使用 EXPORT 时给出简短说明；把长句拆短 |
| NextLive | 改写为无歧义的三句：先在 i 之后找其他合格区域；没有其他合格区域时，若 i 仍合格则返回 i，否则返回 ⊥ |
| 2.4 Assumptions | 开头用一段话概括全部假设 |
| Section 3 开头 | 点出与 rotation 最相关的三个对象：anchor、terminal DATA、regional receipt |
| Anchor 定义 | 首次出现 fresh position 时给出简短解释 |
| Espresso reference | 首次提到 CheckFINAL 时加前向引用；“precede”改为“严格早于含 EXPORT 的区块” |
| Signed context | 用直白的话解释为什么签名要绑定初始 anchor hash |
| Base justification | 首次使用 boundary witness 时说明它就是开启该 session 的 handoff 记录 |
| Admission | 开头加一句“replica 只在 admit 后才在该 view 中行动” |
| Section 4 开头 | 用一段话说明 session 的生命周期：启动、运行与恢复、两种结束方式 |
| Changes from Part I 第 9 条 | 删去旧词 “bootstrap”，改为“同样经由 Protocol 5 开启” |
| 5.3 Session descriptor | 先说明为什么需要 descriptor，再给出定义 |
| 5.4 Global finality rule | 在正式规则前加两步直白概括（先处理 Force，再逐个追加 DATA）。概括包含 PENDING 和 Espresso reference 的上下界，并说明正式规则以 Protocol 3 为准 |
| Replay state / Output | 前向说明 Out_q 的定义位置；accepted count 补一句英文解释（已进入 G_q 的本 session DATA 数）；PENDING 改为直接写成 NextLive(...)=⊥ |
| Waiting and partial progress | 将 “Step 2 without a matching DATA is not a wait either” 改为更清楚的说法 |
| Section 6 开头 | 四点核心规则移到例子之前，并说明它们对应 1.1 的第 2、3、5 点。随后说明两种 handoff 的共同点，以及唯一的区别：anchor 不同。最后列出 6.1–6.3 的内容。例子段落补一句“B 验证 Force 后旧 (7,21) 程序停止” |
| Figure 1 | 由 `[H]` 改为 `[tbp]`，避免 Section 6 开头所在页下半页空白 |
| ReadyBundle | 开头说明它的作用：代替启动投票轮，使所有正确成员在一个区域延迟界内进入 |
| Section 8 开头 | 加一段说明：前六小节是例子，后面是保证和条件 |
| 8.1 | “after Ready and bootstrap”改为“after Ready and Enter” |
| 8.6 | “outside the v4.5 assumptions”改为“outside the model of this document” |
| Appendix C | 开头加一句总结；未定义的 “timed cohort”“prepared timing claim” 改为直白说法，并引用 Proposition C.1 |

## 未改动的内容

- 所有 Definition、Assumption、Proposition 的陈述，以及所有证明。
- 六个协议框中的步骤。
- 记号表、证书表、parent selection 表、L1 表和 control policy 表。
- Section 6 的四点核心规则原文（只调整位置和引导语）。

## 验证

- LaTeX 编译成功，53 页（v4.5 为 50 页）；日志中无 warning、Overfull/Underfull、未定义引用。
- 已用 `git diff --word-diff` 逐处核对：所有改动都是说明文字，没有改动协议框、定义或命题。
- 已检查全文缩略图，并放大检查第 1–5 页（摘要、核心思路框、例子、读法指南）和第 30–31 页（Section 6 开头和 Figure 1）。核心思路框完整放在第 2 页，不跨页；因此第 1 页下方留有部分空白。
- 这是表述层面的人工检查，没有运行模型检查器或机器证明。
