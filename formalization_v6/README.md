# 当前形式化文档（formalization_v6）

2026-10-30 从 `formalization_v5` 迁入 Part 0 与 Part 1 的现行版本，作为新的工作目录；各文档保留自己的版本号。
原路径保留兼容链接，指向这里的同一份文件，所以 v5 的 Part 2–5、meetup 记录与学习笔记中的链接照常可用。
迁移前后的文件哈希见 [`relocation_2026_10_30.json`](relocation_2026_10_30.json)：Part 0 未改动；Part 1 只把指向仍留在 v5 的文档的相对链接改为 `../formalization_v5/…`，正文未改。

| Part | 内容 | 版本 | 文件 |
| --- | --- | --- | --- |
| 0 | 模型、假设与设计目标 | v0.8 | [part0_model_goals_zh_v0_8.md](part0_model_goals_zh_v0_8.md) |
| 1 | 模块与接口 | v0.3 | [part1_module_interfaces_zh_v0_3.md](part1_module_interfaces_zh_v0_3.md) |

## 分析稿

- [区域 BFT 的抽象 v0.3](regional_bft_abstraction_zh_v0_3.md)（2026-10-31）：假设为部分同步、认证通信、全局时钟，加小于 1/3 的敌手界。控制输入 Create、Enter、Close；只读输入 Now、Dep、EspHead、Payload；输出 Final、Evidence。删除预确认；加入每个视图至多 K 个片段的区域内轮换；固定 origin 规则，使 deposit 成为显式输入 Dep(h)。
- [区域 BFT 的抽象 v0.2](regional_bft_abstraction_zh_v0_2.md)（2026-10-30，已由 v0.3 取代）：含预确认与条件模式的版本。
- [区域 BFT 的形式化 v0.1](regional_bft_abstraction_zh_v0_1.md)（2026-10-30，已由 v0.2 取代）：同步假设下的版本，含对当前 Part 3 的逐条核对与改动清单 C1–C8。
- [L2 出块节奏 b_t 与轮换空档](block_time_and_rotation_gaps_zh.md)（2026-10-30）：N1 只定时间戳、不定出块时刻；用"滞后与追赶"模型比较常规轮换（约 0.35 秒）与 Force（约 20 分钟）的空档，说明 b_t 为什么要明显大于区域单块用时，并给出 b_t 的配置约束（建议 1 秒，速度放在子块层）。
- [恢复时的时间假设：Espresso 停机、RESUME 与 Force 追赶](recovery_timing_assumptions_zh.md)（2026-10-30）：先列出八条时间假设与各安全目标对它们的依赖，再用同一组例子逐一检查短、中、长停机、发布失败与区域故障后 Force 五种情形，指出哪里成立、哪里被破坏，以及怎样修。
- [时间依赖：Espresso、L1、L2 的同步关系，以及区域共识换成部分同步之后](timing_dependencies_and_simplex_zh.md)（2026-10-30）：梳理三条链之间的十五条时间耦合，并评估区域 Sync HotStuff 换成 Simplex 类部分同步 BFT 对假设的影响。其中 §4–§6（停机与恢复）已由上一篇取代；§3（换成部分同步）仍以本文为准。

## 仍在 v5 的文档

- [Part 2：L1 Authority v0.1](../formalization_v5/part2_l1_authority_zh_v0_1.md)
- [Part 3：Session Program v0.1](../formalization_v5/part3_session_program_zh_v0_1.md)
- [Part 4：Session Manager v0.1](../formalization_v5/part4_session_manager_zh_v0_1.md)
- [Part 5：Global Finality Rule v0.1](../formalization_v5/part5_global_finality_rule_zh_v0_1.md)
- [协议修订记录](../formalization_v5/protocol_revisions_zh.md)、[以 D 为准的恢复](../formalization_v5/d_anchored_recovery_zh.md)
- Part 0、Part 1 的历史版本（Part 0 v0.1–v0.7、Part 1 v0.1–v0.2）

相关讨论：[2026-10-27 meetup：Liveness Recovery](../meetup_record/261027.md)、[Simplex 学习笔记与区域 BFT 选型](../learning/zh-CN/16_simplex_and_regional_bft.md)。
