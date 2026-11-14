# v3 跨区轮换提案

> 后续写作入口已更新为 [status-entry 新大纲](../v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md)：status map 用来过滤预定义列表；非活动 region 恢复不强制轮换。本目录保留前一轮设计记录。

日期：2026-09-28。
当前版本采用用户最新决定：**旧区 finalized 后新区才开始排序；委员会只授权事故截止 batch 高度及区域状态**。

## 当前阅读入口

1. [中文设计文档](design-zh.md)：先确认后交接、全局 batch 高度、委员会高度声明与区域内容 witness 分离、L1 发布验证及事故完整性条件。
2. [v3 大纲与修订计划](v3-outline-and-revision-zh.md)：同步删去条件会话，增加截止高度唯一解析的定义/证明。
3. 正常轮换图：[PDF](figures/normal-handoff.pdf)、[PNG](figures/normal-handoff.png)、[原生 TikZ](figures/normal-handoff.tex)。
4. 区域 blame 图：[PDF](figures/blame-pipeline.pdf)、[PNG](figures/blame-pipeline.png)、[TikZ](figures/blame-view-change-pipeline.tex)、[编译入口](figures/blame-view-change-preview.tex)。

正常图已按本轮设计重新生成；blame 图本轮未修改，仍展示区域本地恢复流程。
固定 quota 下的末批 Drain 适配仍需完成，不因删除条件会话而自动解决。

## 本轮变更

- 删除条件会话、候选 root 晋升/丢弃，以及独立的 StreamFinal 阶段。
- session 启动前验证已 finalized 的父锚点；正常交接不需要新的委员会签名或 L1 事件。
- 区分 $2\Delta_G+\delta_X$ 的理想边界预算、未隐藏 bootstrap 与新区首批的最终确认时间。
- 委员会事故事件签全局 batch 高度 b*，不签 tip hash 或新区首 header；独立区域 witness 决定该位置的内容。
- 不采用委员会定期 stream checkpoint 的候选方案；L1 继续需要区域证明或等价聚合证明。
- 保留完整 cutoff、fencing、数据恢复及历史补发布约束，避免用较早高度回滚 finalized 后缀。
- 委员会恢复资格的生效边界必须明确；恢复为 eligible 不等于立即 active。

## 历史独立分析

以下文件是第一轮的三路子代理分析，保留以便追溯，**不代表本轮采用的协议**：

- [正常 conditional pipeline 的旧方案](working/normal-rotation.md)
- [委员会和 L1 的早期分析](working/committee-l1.md)
- [Slashing、blame 和初步大纲](working/slashing-blame-outline.md)

此前独立交叉检查发现了 Drain、nonce、历史补发布等问题；相关必要约束继续保留。
其中条件会话和按分支 tip 授权等选择已由本轮决定取代。
本轮为本地一致性修订，没有新增子代理评审，也不引用旧评审作为新协议的证明认证。

## 验证记录

- 起始读取的 [main_v2.tex](../../main_v2.tex) 为 694 行，SHA-256 `d201da53bceb0df51f8be46784f9f7b576f8ce1f8bb512526c15e23d30f08e35`。
- 验证时检测到正文同期发生变化，读到 hash `24fcf93848b06cce8df91a321200b4291b3b6b9818938b723254f10d283f1eb6`；不能报告前后 hash 相同。本轮未执行对正文的写入，也未恢复或覆盖这次变化。
- 先前 629 行版本的 hash 是 `e61c6e5083979464cef09f86d101a62940d040bd8735673fcad03a705deeb919`，仅是历史基线。
- 本轮修改本目录的设计、大纲、README 和正常轮换图；未修改 manuscript、其图输入或历史 working 分析。
- 正常轮换 TikZ 重新编译为单页 PDF，最终 PNG 已目视检查，无裁切或重叠。
- Markdown 本地链接已检查存在；旧阶段名称仅保留在“已删除/历史方案”的说明中。
- 未运行协议实现测试或 model checking，未声称已有端到端证明。
- 未完成的工作包括 terminal adapter、事故 closed-cut 的具体机制、普通恢复资格激活，以及 DA/execution 接口实现。

在 `figures/` 内重建：

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build normal-handoff.tex
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build blame-view-change-preview.tex
```

编译文件位于 `figures/build/`；稳定阅读副本为 `normal-handoff.pdf` 和 `blame-pipeline.pdf`。
