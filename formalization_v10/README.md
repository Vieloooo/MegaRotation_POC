# formalization_v10

[formalization_v9](../formalization_v9/README.md) 加入时钟偏差模型后的版本。v9 假设所有参与者读同一个完美全局时钟；本版改为每个 replica 读本地时钟，对真实时间的偏差至多 $\delta_G$，并给 L1 timestamp 加误差界 $\delta_{\rm L1}$。Espresso timestamp 的假设 E-S、E-TS 不变。协议规则只因副本时钟偏差而修改；L1 与 Espresso 的时间只进入模型假设和预算。

依据的设计决定见 [v9 时钟决定记录](../formalization_v9/clock_decisions_zh_v0_1.md)（决定 1–13）；分析与审查见 v9 的 [时钟偏差分析 v0.1](../formalization_v9/clock_skew_zh_v0_1.md)、[多时间域模型 v0.2](../formalization_v9/clock_skew_zh_v0_2.md)、[审查 v0.1](../formalization_v9/clock_skew_review_zh_v0_1.md)（K1–K10）与 [审查 v0.2](../formalization_v9/clock_skew_review_zh_v0_2.md)（K11–K19）。

## 阅读顺序

1. [00 系统架构与模块总览](00_arch_zh_v0_1.md)：§2.3 的本地时钟模型（C2、C3、C5，推论 C1、C4，到达时刻 $\tau_m$ 与 L-CLK）和计入时钟损坏的容错阈值；§1.2 的 L-TS；§1.4 中 E-TS 与 replica 时钟的关系。
2. 模块文件：
   - [01 L1 Authority](01_l1_authority_zh_v0_1.md) — §3.10 RESUME 目标的下界计入 $\delta_G$
   - [02 Session Program（Regional BFT）与 Block Builder](02_session_program_zh_v0_1.md) — Accept 5a–5d 与 WAIT(time)；$T_{\rm win}$ 下界；BFT-5、BFT-7 与 G1.2、G1.3、G3.2 的偏移项
   - [03 Session Manager](03_session_manager_zh_v0_1.md) — 时间判断读本地时钟；交接余量
   - [04 Publisher](04_publisher_zh_v0_1.md) — PUB-4 推论的前提与发布余量
   - [05 Batcher](05_batcher_zh_v0_1.md)、[06 Certifier](06_certifier_zh_v0_1.md)、[07 Batch Inbox](07_batch_inbox_zh_v0_1.md)、[08 DA](08_da_zh_v0_1.md) — 不读时钟，内容同 v9
3. [09 最终性规则](09_finality_rules_zh_v0_1.md)：SessionEnd 的说明；公告、发布余量与原生窗口三项预算；Espresso 假设不成立时的说明。
4. [10 端到端流程](10_pipelines_zh_v0_1.md)：参数表加入 $\delta_G$、$\delta_{\rm L1}$ 与时钟的部署前提；§2.3 的约束与余量；交接与窗口末高度条件。各流程的高度与时刻同 v9。

数值复算：v9 的 [checks/calibrate_261027.py](../formalization_v9/checks/calibrate_261027.py) 检查流程中的时间与高度；本版的 [checks/clock_margins.py](checks/clock_margins.py) 检查 10 §2.3 中计入时钟偏差的余量。

## 本版采用的默认取值

下列事项尚未由你确认，本版先按以下取值写入，文中标〔待确认〕。

| 事项 | 本版取值 | 位置 |
| --- | --- | --- |
| H1：L1 fixed-pace 公式 | 保留 fixed-pace 公式，$\ell$ 按 slot 编号理解 | [00 §1.2](00_arch_zh_v0_1.md)、[09 §3.5](09_finality_rules_zh_v0_1.md) 公告 |
| H2：假设不成立的讨论 | 09 只加一段说明，详细分析留在审查文件 K11–K16 | [09 §3.5](09_finality_rules_zh_v0_1.md)、[00 §1.4](00_arch_zh_v0_1.md) |
| G2：$T_{\rm ready}$ 是否含 watermark 等待 | 含 | [01 §3.10](01_l1_authority_zh_v0_1.md)、[10 §2.3](10_pipelines_zh_v0_1.md) |
| $\delta_{\rm L1}$ | 4 s | [10 §2.2](10_pipelines_zh_v0_1.md) |
| $T_{\rm prep}$ | 保留 25 min；精确下界的余量 999 ms，G2 为“不含”时须取 26 min | [10 §2.3](10_pipelines_zh_v0_1.md) |
| 决定 9：未发现的时钟损坏 | 计入 $\mathrm{Clk}^{\rm bad}_s$ | [00 §2.3](00_arch_zh_v0_1.md) |
| 决定 9：发现时钟损坏的判据 | 不给判据，下线作为行为假设；L1 守卫列为可选 | [00 §2.3](00_arch_zh_v0_1.md) |
| 决定 9：下线的含义 | 关闭当前实例、不再进入新实例；时钟恢复后可在之后配置的 session 中重新参与 | [00 §2.3](00_arch_zh_v0_1.md) |
| $T_{\rm build}$、$T_{\rm fin}$ 的示例值 | 不给；窗口末高度与 $T_{\rm win}$ 只核算偏差项 | [10 §2.3](10_pipelines_zh_v0_1.md) |

审查 K17（L1 finality 停滞超出 $T_{\rm L1}$ 时区域停产）与时钟模型无关，本版未处理。

## 约定

- 同 v9：每个具名记录只在一个文件中定义；跨模块的数据交换列在 [00 §4.4](00_arch_zh_v0_1.md) 的接口连接表中；文中 “[0x §a.b]” 指本目录中对应文件的小节。
- 各文件开头的“本版相对 v9 的改动”一段列出该文件的修改。v9 的文件保留作对照。
