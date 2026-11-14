# 交接文档：把 v10 的 R-WAIT 收成 GFR 的重放输入条件

日期：2026-11-10。工作目录为 `/Users/jin/projects/megaeth_sequenser_rotation`，目标版本为 `formalization_v10`。任务是按用户已确认的最小方案修改中文规格并核对边界例子。

## 1. 背景和目标

用户希望核心 GFR 只说明给定公共 finalized 日志如何重放得到 $G$。日志同步、重试和本地等待是在线调用方的问题。现稿把它们写在 [09 §3.4](09_finality_rules_zh_v0_1.md) 的 R-WAIT 下，容易让人把公开截止键理解为本地时钟上的定时器，或把某个节点还没同步完理解为 GFR 会做出不同的判定。

把这一节改成「可重放的输入」或「事件何时可处理」。给定完整、已验证的公共输入，每次转移确定。节点可以处理到输入支持的最后一个位置。节点何时再次调用 GFR 由实现决定。使用现有术语、字段和事件，不为这个改动加新定时器、新时钟或新接口。

## 2. 需要写清楚的语义

- **公共顺序。** 维持现有时间键和同键次序。L1 控制（Force 和 RESUME，按 L1 位置）先于 Expire，再到 SessionEnd，再到 Espresso（按 Espresso height）。定时器仍由当前重放状态生成。
- **处理键 $x$ 的输入。** Espresso finalized 前缀必须完整覆盖到时间戳至少为 $x$ 的块，以确定不会再出现键小于 $x$ 的 Espresso block。处理 Espresso block 时，该块和此前完整日志已提供这项覆盖。L1 finalized 前缀必须完整覆盖到时间戳至少为 $x-M_a$ 的块，以确定所有同键或更早的控制记录都已读到。若当前转移还需要更晚的 finalized L1 输入来验证 origin、metadata 或强制前缀，也要给齐这部分输入。若事件为 RESUME，则要有绑定该条 L1 记录位置的 ResumeCut。
- **区分输入没读齐和结果没公开全。** 前者表示该转移尚不能完成，读齐后仍按原重放位置处理。后者指已经读全相关 Espresso 前缀，但一份 publication 在公共历史中还缺组成对象，此结果当前不具备接纳资格。沿用现有的补齐和 $q_{\rm avail}$ 规则。读取者缺少必需验证材料不能让它把一个结果永久跳过。
- **明确常规交接的两条路径。** 仍可在接纳前驱的 $h_{\max,s}$ physical block 时立即交接。否则，公开输入覆盖截止键且前序事件都处理后，若前驱仍持有 NEXT，就执行 SessionEnd。选严格早于截止键的合格历史中最后已接纳的 physical block，无新块则沿用前驱 anchor。同键结果在 SessionEnd 之后。
- **保留接口边界。** `AdvanceGlobal` 仍接收 finalized L1 和 Espresso 前缀及 resumeReader，返回已完成的重放进度。保留 `WAIT{next_position, missing}` 作为输入尚未齐备的返回状态。它不推进重放位置、不撤销已经完成的前缀，也不描述拉取日志或重试的流程。不要引入对「节点当前已经知道全网最新 finalized 位置」的要求。
- **时序预算独立。** $M_a\ge T_{\rm L1}+b_{\rm L1}+\Delta_{\rm ts}+\delta_{\rm L1}$ 仍是正常条件下让 L1 控制日志覆盖及时就绪的充分预算。它不保证该位置已经包含验证任一候选所需的 origin，也不参与公共重放确定性的证明。L1 或 Espresso 进度不足时，可重放的位置可能暂时不前进。

## 3. 文件落点

| 文件 | 只做这一层的调整 |
| --- | --- |
| [09_finality_rules_zh_v0_1.md](09_finality_rules_zh_v0_1.md) | §3.4 改为输入条件，§3.5 把相同的时序推导连到该条件。§4.2 明确必需的 finalized 验证输入尚未读齐时当前事件还不能完成。§4.3 明确 SessionEnd 的触发。§4.6 和 §4.8 更新交叉引用和接口说明，保留 ResumeCut 和原子替换。同步更新页首有关 R-WAIT 的版本说明。 |
| [11_global_chain_safety_liveness_zh_v0_1.md](11_global_chain_safety_liveness_zh_v0_1.md) | §1.2 的归纳直接使用已覆盖该重放位置的完整输入。§1.5 第一项收成输入契约，把节点只同步了一半的时序反例移出核心 safety 论证。保留 §1.5 中 publication 完整性的规范问题和低目标 RESUME 问题。§2.1 中把旧术语换为 L1 控制日志覆盖，保留 $M_a$ 预算和普通块确认式。 |
| [01_l1_authority_zh_v0_1.md](01_l1_authority_zh_v0_1.md) | §3.9 说明控制记录何时属于 GFR 的输入由公共覆盖条件决定，$M_a$ 提供公告时间预算。保留 Force／RESUME 的激活公式和 Authority 检查。 |
| [08_da_zh_v0_1.md](08_da_zh_v0_1.md) | §4 把 R-WAIT 的引用改成缺 DA 数据时 ResumeCut 尚不可用，GFR 可重放的前缀停在该事件之前。 |
| [10_pipelines_zh_v0_1.md](10_pipelines_zh_v0_1.md) | 更新参数表、Force 和 RESUME 的时序图及步骤中的 R-WAIT 术语，分别写明何时已有 Espresso 覆盖、L1 覆盖或 ResumeCut。数值和场景不变。 |

00 已定义 watermark 和通用 `WAIT`，03 与 04 已按公共生命周期和 watermark 行动。它们无需因此重写。保留 BFT-7 的本地停止签名时间和 GFR 公开截止键的区分。三种交接的 anchor 来源、Force 开新分支、RESUME 可以替换旧 $G$ 后缀、窗口、参数值和事件顺序沿用现稿。

## 4. 验收例子

1. **截止边界。** 前驱仍为 NEXT，公开截止键为 100。Espresso 只覆盖 95 时，不触发 SessionEnd。之后一块时间键为 99 的 Espresso block 才 finalized，携带满足其余条件的前驱结果。处理键 99 的结果先于 SessionEnd。等 Espresso 和 L1 输入都覆盖截止后再处理键 100 的 SessionEnd。同键 100 的 Espresso 结果在它之后，不能作为前驱截止前的结果。
2. **提前接满窗口。** 若键 99 接纳的是 $h_{\max,s}$，应立即按现有规则交接。旧 session 的键 100 SessionEnd 不能再开第二次交接。
3. **控制日志够了，验证输入还差一块。** 现稿旧例中，Espresso 键 10:29:47，$M_a=20$ 分钟。L1 读到 10:09:48 可以封住控制日志的顺序，但某个通过其余可先行检查的结果还需 10:10:00 的 finalized origin。较慢的读取者只停在前一个位置，拿到数据后与较快的读取者在同一位置得到同一结果。可以用它做检查，无需保留在 11 的正文。
4. **RESUME。** 在 ResumeCut 缺失时不能做一半替换。取得该条 L1 记录对应的切面后，同一重放位置的所有读取者用同一派生前缀替换 $G$ 并取得同一新 anchor。此前的 $G$ 可能与新前缀不相容，这是现有 RESUME 语义。

完成后检查 v10 内旧术语和交叉引用，核对边界例与 §3.3–§4.6 的顺序。检查 Markdown 链接、公式分隔和差异中是否混入无关编辑。

## 5. 工作区注意事项

交接时在本地分支 `clock-skew`，HEAD 为 `2f8dfddceeb2cd562a3ee8b81494903bca6cd6eb`。以开始工作时的实际文件为准。`00_arch_zh_v0_1.md` 有交接前已经存在的本地改动，保留它。模块 11 是本地已有的未跟踪文件，直接修订当前内容，不能只从 Git HEAD 查找。修改计划已获用户确认，按本任务范围完成规格修改即可。
