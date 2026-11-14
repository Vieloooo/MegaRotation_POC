# Restaking 服务学习资料：从 EigenDA 到 Sequencer Rotation

研究日期：2026-09-22。对应 [09-22 架构需求](../../../meetup_record/meeting_0922_requirements.md)。本组资料以 EigenDA 为主线，回答：一个有链上质押支持的链下服务如何工作，节点究竟对什么签名，哪里需要 consensus，以及哪些机制能帮助我们设计同地小集群。

后续专题：[以太坊和 Restaking 的质押进出机制及区域轮换设计（2026-10-10）](../13_staking_membership_and_randomized_rotation.md)，详细讨论加入、退出、时间锁、churn、可罚担保，以及与 `main_v3.tex` 的衔接。

## 先读哪个文件

| 顺序 | 学习资料 | 读完能回答什么 |
| --- | --- | --- |
| 1 | [AVS 的组织方式与工作流程](01_avs_architecture.md) | Restaker、operator、operator set、任务聚合器和链上合约如何配合？质押如何变成服务约束？ |
| 2 | [EigenDA：从提交数据到取回数据](02_eigenda_workflow.md) | 编码、relay、存储节点、证书和 L1 分别做什么？为什么 EigenDA 有 BFT 保证，却不运行自己的排序链？ |
| 3 | [共识、轮换与我们的架构](03_consensus_and_rotation.md) | 任务型 AVS、MACH、HotStuff、Espresso 各解决什么？本周应该借鉴哪些部分？ |
| 查资料 | [资料索引与阅读路线](sources.md) | 按问题找到官方文档、版本固定的代码和论文，以及各来源的适用范围 |

如果只想先抓住设计方向，读第 2 篇的“它的 consensus 到底是什么”，再读第 3 篇的“映射回我们的需求”。

## 这次 review 得到的主要认识

**Restaking、数据可用性和排序共识是三个可以组合的层。** EigenLayer 提供成员、质押、证书验证、奖励与罚没的基础设施；具体服务仍需自己定义工作和确认规则。EigenDA 为数据的存储与取回提供服务，而一条 mini-block stream 还需要决定“下一段历史是哪一段”。[AVS 官方说明](https://docs.eigencloud.xyz/eigenlayer/concepts/understanding-avs-on-eigenlayer)、[EigenDA Spec](https://layr-labs.github.io/eigenda/)

| 对象 | 主要确定的事情 | 对我们最有用的部分 |
| --- | --- | --- |
| EigenLayer | 谁承担哪项服务的经济责任 | Operator set、stake 分配、退出延迟、奖惩接口 |
| EigenDA | 某份数据在约定条件下可取回 | 数据与控制消息分离、持有数据后签名、消费者验证证书 |
| 任务型 AVS 示例 | 某个指定任务得到了什么结果 | 任务定义、聚合结果、截止时间与挑战 |
| MACH 的公开 M2 实现 | 对 rollup 异常告警提供确认 | 将观察、签名、聚合与链上处理分开 |
| HotStuff / Espresso | 多个参与者对连续历史达成一致 | 前缀、投票、锁定、leader 接替与消费接口 |

上述分类的具体证据和版本见各篇。Espresso 和 HotStuff 在这里是排序共识的对照，不因列在同一份资料中就被视为 EigenLayer AVS。

## 使用这些资料时的版本约定

- **文档与白皮书**说明其描述的机制；路线图与模型不直接视为当前部署事实。
- **代码快照**说明该提交中存在的接口与实现，不证明某条主网已部署相同版本。
- **教学例子与设计推断**在正文标出，不是项目的默认参数或已经证明的集成方案。

本次特别检查了 EigenDA V1/V2 的 L1 路径、较新 API 的完成状态、EigenLayer 的 slashable stake，以及 MACH 与任务型 AVS 的具体签名对象。没有进行节点部署、性能实验或链上配置审计。

[返回中文学习指南](../README.md)
