# v3 新大纲：状态过滤调度与活动区失效轮换

本目录对应用户关于委员会二元结构的最新要求，以及“非活动 region 恢复不触发强制轮换”的后续澄清。

## 交付

- [大纲、修订计划与 L1 派生关系](outline-and-revision-plan-zh.md)
- [两类 pipeline，PDF 两页](figures/rotation-pipelines.pdf)
- [正常轮换 PNG](figures/regular-rotation.png) / [原生 TikZ](figures/regular-rotation.tex)
- [强制轮换 PNG](figures/status-change-rotation.png) / [原生 TikZ](figures/status-change-rotation.tex)
- [PDF 编译入口](figures/rotation-pipelines.tex)
- [来源快照记录](source-baseline.json)

本轮仅提供 v3 的新大纲和计划，没有创建 main_v3.tex，也没有写入 main_v2.tex。

## 当前规则

委员会业务状态为 `(batch height, region -> status)`。
正常到期后沿预定义 rotation list 跳过 unavailable region。
非活动 region 的恢复/下线只更新资格，不结束当前任期、不重置 quota/session。
当前 active region 被标记 unavailable 才触发强制交接；已等待 successor 的轮换可因某区恢复而继续。
两类交接都要求先验证 finalized anchor。
L1 / proof system 重放调度和轮换关系，不能以任意 status change 解释一次 forced rotation。

## 与以前提案的关系

本目录的新大纲取代 [此前大纲](../v3-committee-rotation-2026-09-28/v3-outline-and-revision-zh.md) 作为后续 main_v3 写作入口。
以前的条件会话、独立 StreamFinal 阶段、每次 map 变化都强制换区，以及分立控制命令均不是当前采用的结构。
旧分析保留作为历史材料；本轮没有新增子代理评审。

## 检查范围

- 基线 main_v2.tex 为 697 行，SHA-256 `24fcf93848b06cce8df91a321200b4291b3b6b9818938b723254f10d283f1eb6`。
- 已核对当前 VOTE >2W_i/3、BLAME >W_i/3、CONFIRM >W_i/3 的区别。
- 两张 native TikZ 图编译为两页 PDF，并检查最终 PNG，无重叠或裁切。
- 本目录本地链接、代码围栏和公式块配对已检查。
- 没有协议实现测试、model checking 或完整证明认证。
- 后续仍须明确共同 L1/status snapshot 的生效规则、terminal Drain、forced-cut 完整性与数据恢复，以及单区续任等边界策略。

在 figures/ 中重建：

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build rotation-pipelines.tex
```

最终稳定副本为 `figures/rotation-pipelines.pdf`。
