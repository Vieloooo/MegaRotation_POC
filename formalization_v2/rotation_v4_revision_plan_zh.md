# Sync HotStuff RR Part II v4 写作修订方案

对象：`sync_hotstuff_rr_part2_rotation_v4.tex` 及 `sections/rotation_v4_*.tex`（合计约 3.1 万词）。

目标：让第一次读的人从头读到尾能看懂。每句话简单直白。正文只写"协议是什么、为什么这样做"，不写"和上个版本比改了什么"。

本方案只改写作，不改协议语义。每一处改动都应能回答：协议行为是否变了？答案必须是"没有"。

---

## 1. 现状诊断

### 1.1 版本差异信息混在正文里

读者被迫去想"以前是怎样的"，但手上没有以前的版本。典型位置：

| 位置 | 原文 | 问题 |
|---|---|---|
| 标题、页眉、`pdftitle` | "Version 4: Explicit Multisignature Control"、"Version 4 / Explicit multisignature Force" | 标题在讲版本号 |
| §1 `Scope of version 4`（L103–118） | "no automatic failure detector, delivery deadline, Force gap, cooldown, service-debt ledger…" | 列出旧版本有、新版本删掉的机制。新读者不知道这些是什么 |
| L111–113 | "…are retained from version 3.7" | 直接指向旧版本 |
| L524–526 | "No new two-sided Espresso clock bound is needed" | 回应旧版本或评审意见 |
| L550–551 | "This version specifies no detection or response bound and no enforced interval between Forces" | 同上 |
| L656 | "The additional outer field $\xi(B)$" | "additional" 是相对旧版本说的 |
| L887–890 | "This public algorithm change does not relax the regional full-evidence checks" | 在讲修改，不是在讲协议 |
| L978 | "One proposal-processing rule during normal operation" | 标题暗示以前有多条规则 |
| 多处 | "the original DATA/DRAIN engine"、"original Completion action"、"original drain deadline"、"existing receipt rules" | "original/existing" 相对于什么？ |
| section 片段首行注释 | `% Version 3.5: …`、`% Version 4: …` | 源码注释也带版本号，且 3.5 与 4 混用 |
| `governance.tex` L31 | "This version leaves detection thresholds…" | 同上 |

量化：全文 "retain" 出现 123 次，"original/existing/unchanged/remain" 198 次。很大一部分在表达"沿用旧版本"。

**"retain" 一词多义**，读者最难受：
- "retained from version 3.7"：沿用旧版本
- "Every effective Force retains the entire accepted global prefix"：Force 保留前缀（协议语义）
- "retained anchor"、"retained DATA"：Force 后保留的锚点（专有名词）
- "retain $P$ and stop"：本地缓存
- "PublicStore… retains content"：持久存储

同一个词承担五种意思，读者每次都要猜。

### 1.2 不说人话

1. **否定句过多。** 否定词（does not / is not / never / cannot / need not…）出现 416 次，平均每 75 词一次。很多段落是"X 不做 A。X 也不做 B。X 不意味着 C"，读完仍不知道 X 做什么。
   例（L1469–1474）："No extra waiting period follows this verification… An old-version record after closure can still be finalized as raw Espresso bytes, but it is not an accepted chain result. Espresso inclusion alone, or a receipt alone, does not establish this definition."
2. **名词堆叠。** 如 "the complete applicable finalized L1 control context"、"source-availability cursor"、"node-level full-byte public evidence store"、"publication\_and\_retention\_handles"。
3. **一句话塞太多限定。** 如 "under the stated history, announcement and verification premises"、"Ready retains complete target admission, exact-state and boundary inputs; an external receipt for its inherited anchor may use the public kernel for the producer's intrinsic validity."
4. **概念先用后定义。** Overview 和 Model 里已经大量使用 Ready、Enter、Completion、export session、DRAIN、EXPORT、retained anchor、subposition，它们要到 §3–§6 才定义。`ass:ready-service`（L501）放在 Model 里，但它依赖的 "assigned export session""initial Ready/Enter" 读者此时还没见过。
5. **防御性免责声明打断叙事。** 如 "This qualification is a scope of the claim, not an added assumption about all executions"、"This is local replicated logic, not another trusted regional controller"。这些多是在回应评审意见，放在正文里像在和人吵架。
6. **术语不统一。**
   - 同一个东西多个名字：Session Manager / Cross-region Handler / manager / handler；program / engine / session / pair；branch version / version / $\beta$ / beta（L1733 甚至直接写 "beta changes"）。
   - 同一个名字多个意思：Ready 既是一个 phase，又是 `def:ready` 的校验，又是 "Ready service"。
7. **硬编码引用。** 如 "as specified in Section~8"、"Protocol~4"、"Protocols~5--6"、"P6"。重排后会错，也让读者翻来翻去。

### 1.3 结构问题

- Overview 太长且太密：一次性抛出 $(\beta,r,g,t)$、Force、Ready、DRAIN、export session、$C_{\rm pub}$、两张表和一个 $(7,20)\to(8,21)$ 例子。读者还没建立心智模型就被细节淹没。
- 缺一条贯穿全文的例子。现在的例子分散在各节（A/B/C 区域、高度 100–110、session (7,20)），彼此不接续。
- 主线（正常轮换 → 全局确认 → 强制切换）被实现细节（PublicStore、AuthCache、shared preparation cache、retirement 细节）切碎。

---

## 2. 修订原则（写作规范）

所有改写都按下面的规则来，作为后续每一遍的检查清单。

### 2.1 版本信息
- 正文**零版本信息**：不出现 "version 3.x / version 4 / this version / previous / no longer / additional / new / original / existing / unchanged / retained from"。
- 和 Part I 的差异是合法的（Part II 建立在 Part I 之上），但**只集中在一处**：§Regional protocols 里的 "Changes from Part I" 列表和红字标注。其他地方不再说 "as in Part I""Part I's X does not apply"，除非是必要的引用。
- 所有被删掉的版本差异信息汇总到一个单独文件 `formalization_v2/rotation_v4_changes_zh.md`（changelog），不进 PDF 正文。是否作为附录由你决定（见 §6）。
- tex 源码注释里的 `% Version 3.5:` 等改为说明片段内容和插入位置，不带版本号。

### 2.2 句子
- 一句话一个意思。超过约 25 词或有两个以上从句就拆开。
- **先说正面，再说边界。** 每段第一句说这个东西"是什么 / 做什么"。否定只用于防止常见误解，每段最多一句；并列的多个否定合成一句 "X only does A" 或放进表格的 "Not responsible for" 一列。
- 少用名词堆叠，改成动词短语。如 "the complete applicable finalized L1 control context" → "all finalized L1 control records up to this point"。
- 免责声明（scope / not an assumption / not a controller）：能删就删；必须保留的移到该节末尾的 "Scope" 小段，或移到 §Assumptions。
- 引用一律用 `\cref`，不写 "Section~8""Protocol~4""P6"。

### 2.3 术语（定一张表，全文只用一个名字）

| 概念 | 统一用词 | 不再使用 |
|---|---|---|
| $(\beta,r)$ 对应的一段区域生产 | session | pair（作名词时）、slot（除非专指 index $r$） |
| 节点上管理某 session 的本地状态机 | session program（简称 program） | 与 engine 混用 |
| program 内部跑 Protocol 1–2 的部分 | regional engine | core（统一成 engine） |
| 跨区域协调逻辑 | Session Manager | Cross-region Handler、handler |
| $\beta$ | branch version | version、beta |
| Force 之后保留的全局前缀终点 | retained anchor（专有名词，保留） | — |
| "沿用旧版本" | 删掉，不换词 | retained from、original、existing |
| 本地缓存 / 持久化 | keep / store | retain（只留给 retained anchor） |
| Ready 校验 | Ready check（`def:ready`） | 单独说 "Ready" |
| Ready 阶段 | Ready phase | — |
| Ready 服务假设 | preparation bound（$T_{\rm ready}$） | Ready service |

术语表放在 §Notation 旁边。改写时先做全局替换，再逐句通读确认没有语义变化。

---

## 3. 结构调整

原则：先讲直观故事，再讲形式定义；主线在前，实现细节在后；假设靠近使用处。

### 3.1 新目录（建议）

1. **Introduction**（约 1 页，原 §1 压缩）
   - 一段话：问题是什么（多个区域轮流出块，拼成一条链）。
   - 一段话：三件事——正常轮换靠区域 receipt，全局顺序靠 Espresso，强制切换只能由多签在 L1 上显式发起，且永远不删已确认的全局数据。
   - 一张图：时间线上 A → B → C 正常轮换，然后 Force 到 D。（复用或简化现有 `figures/cross_region_rotation_v4`）
   - 角色表（原 `tab:roles`）。
   - 阅读指引：后面各节分别讲什么。
   - **删除**：`Scope of version 4` 整段、`tab:views` 和 $(7,20)\to(8,21)$ 例子（移到 §5 Force 一节）。
2. **A running example**（新增，约 1 页）
   - 一条例子贯穿全文：区域 A、B、C、D；quota、高度 100–115；一次正常轮换、一次区域内 recovery、一次 Force。
   - 后面每节用"回到例子"来讲新概念。现有的分散例子（L680 A/B/C、L764 高度 100–110、examples.tex 的 108/109–115、$(7,20)$）统一到这一个例子的数字上。
3. **Model and assumptions**（原 §2，变薄）
   - 保留：L1、Espresso、网络、stake、多签委员会、各 bound。
   - `ass:ready-service` 移到 Ready 一节，紧挨使用处。
   - "Verification interfaces"、"Conditions for progress" 移到 Liveness 一节。
4. **Regional engine**（原 §3–§4）
   - 开头一句话：这就是 Part I 的引擎，加上 session、quota、DRAIN 和 receipt。
   - "Changes from Part I" 列表提前到本节开头，作为本节导读。
   - Receipt kernel（`receipt_kernel.tex`）移出，放到全局确认一节（它是给外部验证者用的）。
5. **Normal rotation**（原 §6 的 handoff、Ready、bootstrap 部分）
   - 先讲 selection reference 为什么需要（L675 那段已经写得不错，作为样板）。
   - 再讲 NextLive、REGULAR handoff、Ready check、Enter。
6. **Global finality**（原 §5 + exports）
   - Publish Layer → EXPORT → 解释器 → $G$ 的推进。
   - 区域 finality 与全局 finality 的区别放在本节开头，用例子讲。
7. **Force**（原 governance + control + Force 部分）
   - 多签发 L1 指令 → 激活时间 → 关闭旧 branch version → 从 $G$ 开新 session。
   - `tab:views` 和 $(7,20)\to(8,21)$ 例子放这里（改用 running example 的数字）。
   - 并发候选与竞态（原 "Concurrent candidates and races"）。
8. **Node implementation**（原 Session Manager、lifecycle、retirement、PublicStore、AuthCache）
   - 明确说明：这一节是"一个正确节点怎么组织本地状态"，读者只想理解协议可以跳过。
9. **Guarantees**（原 safety + liveness）
   - 先用一句话列出每个命题保证什么，再给命题和证明。
10. **Implementing the Publish Layer**（原 §9，基本不动）

附录（可选）：完整 notation 表、术语表。

### 3.2 每节固定开头

每个 section 第一段用 2–3 句话回答：
- 这一节解决什么问题？
- 读完你会知道什么？
- 它依赖前面哪些概念？

---

## 4. 逐节动作清单

| 原位置 | 动作 |
|---|---|
| 标题 / 页眉 / `pdftitle` / `pdfsubject` | 去掉 "Version 4" 和 "Explicit Multisignature Control"。版本号只留在 `\date` 或一个脚注里 |
| Abstract（L63–80） | 重写为 4–5 句：问题、方法、关键保证。删掉 "keeps the original DATA/DRAIN engine"、"stated separately from…" 以及末尾一串 "does not enforce" |
| §1 Overview（L82–212） | 按 3.1 第 1 项压缩。`Scope of version 4` 整段删，内容进 changelog；其中对新读者有用的边界（"只有多签能触发 Force"）改成正面陈述放进 Introduction |
| §2.1 L1 control（L216–312） | 拆短句。Genesis 段（L294–312）太密，拆成"起点包含什么"+"第一个 session 怎么打开"两段 |
| §2.2 Publish Layer | 基本可读，只删 "There is no separate cross-region transport service in this model" 这类否定 |
| §2.5 Assumptions | 每个假设后面的解释段只留一段，正面说"这个假设让我们能做什么"。L465–477、L521–526 的否定列表删到最多一句 |
| `ass:ready-service`（L501–526） | 移到 Ready 一节 |
| §2.6 Conditions for progress | 移到 Liveness |
| §3 Notation 表 | 按 2.3 术语表统一用词；补一列"首次定义位置" |
| §3.1 Blocks | "additional outer field" → "The field $\xi(B)$"。L666–673 拆句 |
| §3.3 Certificates / receipts | L884–890 删 "This public algorithm change…"。receipt kernel 移到全局确认一节 |
| §3.4 Admission | "One proposal-processing rule during normal operation" 改名 "Processing proposals"。"Complete regional validation" 第一段（L955–960）重写：先说它检查什么，再说谁用它 |
| §4 Regional protocols | 保留红字和 "Changes from Part I"，这是对 Part I 的合法差异。删除 "retained regional changes" 中的 "retained"。"Retirement gate" 段移到 retirement 一节，这里只留一句指针 |
| §5.1 Regional finality | 删 L1358–1359 的免责声明。Remark 的 5 条保留，每条第一句改成结论句 |
| §5.2 Global finality | L1441–1456 的 public replay result 定义改成列表；删 "Section~8" |
| §5.3 "References do not provide a commit clock" | 标题改成正面："What block references are for" |
| §6.1 architecture | 统一 Session Manager 名称；删 "not another trusted regional controller" |
| §6.3 "Cross-region Handler" | 改名 "Session Manager"，与 §6.1 一致 |
| §6.4 lifecycle | 六个 phase 改成一张状态图 + 一张表（phase、能做什么、怎么离开）。"there is no seventh phase" 删 |
| shared preparation / AuthCache 段 | 移到 Node implementation；"beta changes" → "a branch-version change" |
| `sections/*.tex` 首行注释 | 去掉版本号，改成"本片段内容 + 插入位置" |
| `governance.tex` L31 | "This version leaves…" → "The following are outside this protocol: …" |
| Examples | 统一成 running example 的数字 |
| Safety / Liveness | 每个命题前加一句"这个命题说的是……"；删 "Neither proposition independently re-proves all of Part I"，改为在开头一次性说明证明依赖 Part I 的哪些结论 |

---

## 5. 执行步骤（分五遍，每遍单独提交，便于审阅 diff）

1. **删版本差异。** grep `version|retained from|original|existing|unchanged|no longer|additional|this version|3\.[0-9]` 逐条处理，删除或改写为正面陈述。被删内容汇总到 `rotation_v4_changes_zh.md`。不改结构。
2. **统一术语。** 按 2.3 表做替换，逐句确认语义。更新 Notation 表，新增术语表。
3. **改句子。** 逐节过一遍：拆长句、正面开头、压缩否定、名词堆叠改动词、硬编码引用改 `\cref`。
4. **调结构。** 按 3.1 移动章节、新增 running example 和每节导语。这一步改动最大，放在最后，避免和前三遍的 diff 搅在一起。
5. **核对。** 见第 6 节。

建议另存为 `sync_hotstuff_rr_part2_rotation_v5.tex`（以及 `sections/rotation_v5_*.tex`），v4 原样保留作对照。

---

## 6. 验收标准

- `latexmk` 编译通过，没有 undefined reference，没有 "Section~N" 硬编码。
- grep 检查：正文（不含 changelog）中 `version 3`、`this version`、`no longer`、`retained from`、`original`、`existing` 为 0；"Version" 只出现在日期或脚注。
- 否定词数量较现在（416）下降一半以上；"retain" 只用于 retained anchor / retained DATA。
- 每个术语首次使用时已定义，或带有指向定义的 `\cref`。
- 语义对照：每个 Definition、Assumption、Protocol 步骤、Proposition 逐条与 v4 对比，确认协议行为不变。差异列表为空。
- 通读测试：找一个没参与设计的人（或用一个新 session）从头读到尾，记录卡住的位置，作为最后一轮修改的输入。

---

## 7. 需要你决定的事

1. **changelog 放哪？** 单独的 `rotation_v4_changes_zh.md`（推荐，PDF 完全干净），还是作为 PDF 附录 "Relation to earlier drafts"？
2. **红字（`\RR`）保留吗？** 它标的是相对 Part I 的改动，属于合法差异，但满页红字也干扰阅读。可选：保留；或改为只在 "Changes from Part I" 列表中说明，协议框里不再着色。
3. **结构调整（第 4 遍）做不做？** 前三遍只改措辞、风险低；第 4 遍会移动章节和新增例子，改动大但收益也最大。
4. **新文件名**：另存 v5，还是直接在 v4 上改？
