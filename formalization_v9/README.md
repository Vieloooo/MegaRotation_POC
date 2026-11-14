# formalization_v9

v8 的 [arch v0.2](../formalization_v8/arch_zh_v0_2.md) 与 [Regional BFT v0.13](../formalization_v8/regional_bft_abstraction_zh_v0_13.md) 拆分重组后的版本。

## 阅读顺序

1. [00 系统架构与模块总览](00_arch_zh_v0_1.md)：外部 building blocks、参与者与 session、链数据与映射；模块文件表、接口连接表（W1–W28）与四条链 $R,G,G',D$。
2. 模块文件，每个都包含“定位与直觉、最小假设、接口”，五个模块另有“性质”：
   - [01 L1 Authority](01_l1_authority_zh_v0_1.md)
   - [02 Session Program（Regional BFT）与 Block Builder](02_session_program_zh_v0_1.md) — 性质 BFT-1…7，组合目标 G1–G3
   - [03 Session Manager](03_session_manager_zh_v0_1.md) — 另含生命周期规则与 Kill 判据
   - [04 Publisher](04_publisher_zh_v0_1.md) — 性质 PUB-1…5
   - [05 Batcher](05_batcher_zh_v0_1.md) — 性质 BAT-1…4
   - [06 Certifier](06_certifier_zh_v0_1.md) — 性质 CERT-1…5
   - [07 Batch Inbox](07_batch_inbox_zh_v0_1.md)
   - [08 DA](08_da_zh_v0_1.md) — 性质 DA-S1、DA-S2、DA-L1、DA-L2、DA-RET
3. [09 最终性规则 v0.2](09_finality_rules_zh_v0_2.md)：Global Finality Rule（事件与排序、状态机、记录与接口）、Derivation Rule、原生结算、各层结果的保证。本版起 Force 与 RESUME 统一以 L1 收录时间加 $M_a$ 为激活时刻。上一版 [v0.1](09_finality_rules_zh_v0_1.md) 保留作对照。
4. [10 端到端流程 v0.2](10_pipelines_zh_v0_2.md)：按显式示例参数（1 s L2 block、假设的 finalized 预算、12 h 窗口换算等）重算的 Regular handoff、Force handoff、Espresso 长停机后的 RESUME、批次层超窗后的 RESUME、短故障与死区，以及校准结论。上一版 [v0.1](10_pipelines_zh_v0_1.md) 使用压缩的高度，保留作对照。

[review_zh_v0_1.md](review_zh_v0_1.md) 记录先前的表述检查。

[security_review_zh_v0_1.md](security_review_zh_v0_1.md) 是独立的协议安全审查，列出反例、模型边界及待你决定的修复方向。数值复算脚本为 [checks/calibrate_261027.py](checks/calibrate_261027.py)；审查基线摘要为 [checks/security_review_baseline.json](checks/security_review_baseline.json)。

[security_review_zh_v0_2.md](security_review_zh_v0_2.md) 在 v0.1 的 S1–S9 之后加入第二轮四个子代理的结果（S10–S27）、两条跨条目修复建议、对流程 v0.2 的两处勘误和完整的人工检查顺序；v0.1 的条目原样保留。基线摘要为 [checks/security_review_v0_2_baseline.json](checks/security_review_v0_2_baseline.json)。

[clock_skew_zh_v0_1.md](clock_skew_zh_v0_1.md) 分析把 [00 §2.3](00_arch_zh_v0_1.md) 的完美全局时钟换成有界偏差 $\Delta_G$（另加对真实时间的偏移界 $\delta_G$）时，各模块需要的修改与 10 的数值重算；只做分析，未改动规格原文。对应的安全审查为 [clock_skew_review_zh_v0_1.md](clock_skew_review_zh_v0_1.md)（K1–K10）。

[clock_skew_zh_v0_2.md](clock_skew_zh_v0_2.md) 把模型扩展为副本、L1、Espresso 三方各自漂移：L1 timestamp 加偏移界并改按 header 读取，Espresso 的 E-TS 拆成超前与滞后两个方向并与 $\mathsf{Live}_E$ 脱钩；列出 GFR 中所有跨时间域的比较与各方向的容忍量。对应的安全审查为 [clock_skew_review_zh_v0_2.md](clock_skew_review_zh_v0_2.md)（K11–K19，含 Espresso 超前、跳变、滞后、冻结与 L1 finality 停滞）。

[clock_decisions_zh_v0_1.md](clock_decisions_zh_v0_1.md) 记录时钟模型修改中已经做出的设计决定和尚待决定的事项，供之后修改规格时使用。

按这些决定加入时钟偏差模型后的规格见 [formalization_v10](../formalization_v10/README.md)。

## 约定

- 每个具名记录只在一个文件中定义，其他文件链接到它；定义位置见 [00 §4.6](00_arch_zh_v0_1.md)。
- 跨模块的数据交换都列在 [00 §4.4](00_arch_zh_v0_1.md) 的接口连接表中；各模块“最小假设”一节的上下游表引用相同编号。
- 文中 “[0x §a.b]” 指对应文件中的小节。
