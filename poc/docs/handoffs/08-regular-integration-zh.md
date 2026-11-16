# 08 三区 regular 交接集成

> **2026-11-16普通运行路线修订：** 可先完成15-worker持续运行的regular两路径及多地区轮转，不等待完整01。必须真实后继首结果进入G；意外进程退出记失败/中断，不自动空journal重启。候选Pause/选回、state preparation与Close规则仍真实执行。 详见[不重启worker的交接路线](../commander/restart-free-handoff-track-20261116.md)。

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，从含 **04、05、06、07** 验收产物的一个集成 SHA 建 managed worktree `poc-regular-integration`，分支 `codex/poc-regular-integration`。交付 REG-B，15 个真实 worker 经过三个地区完成 regular，两个独立 viewer 可重放。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 集成边界

从 04 的 15-worker 准入、05 的每节点 Manager、06 的服务与日志、07 的 GFR 组成单机系统。前三个 session 为 `(0,0)→(0,1)→(0,2)`，地区 R0→R1→R2，窗口 `[1,10]→[11,20]→[21,30]`。应用继续使用 03 的 L1 fixture 和有状态工作量。后继从合法来源得到状态和 descriptor，每 worker 自己 Ready/Enter。所有消息、fault 和链读者投递均经 02 调度。 [REG-B 准备](../contracts/regular-preparation-v1.md) · [v10 regular 流程](../../../formalization_v10/10_pipelines_zh_v0_1.md#3-流程一regular-handoff) · [v2 验收](../../../formalization_v10/poc_design_zh_v2.md#122-第一阶段验收矩阵)

Builder 拥有 `poc/scenarios/regular/`、启动和场景接线，具体 launcher 和 native 入口在合同列出。Verifier 拥有 `poc/verification/regular-integration/` 的独立期望、检查器和审查记录。若集成暴露公共接口或组件缺陷，由主会话登记到对应所有者并在修复后更新单一集成 SHA。

## 验收

| 场景 | 完成条件 |
| --- | --- |
| 满窗口 regular（v2 R1） | h10 和 h20 的合格结果触发交接，R2 首个结果进入 G；增加回到 R0 的下一轮场景 |
| SessionEnd regular（v2 R2） | 前驱只接到 h9，公共覆盖到 SessionEnd 后才切换，后继固定为 11–20。以 h9 为 anchor 的候选须提前准备并在 h11 的 Accept 界内生产，可提前公开并缓存。获选后合法 h11 覆盖 padding10 后进入 G。另测未提前产出 h11 时，后继仍按原窗口和当前合法 slot 生产 |
| 错时启动与候选切换 | 记录各 worker descriptor/anchor/clock-ready/Enter/view/Close；覆盖早到、晚到、3/2 候选分裂、选回已有候选 |
| 双读者与独立 checker | A/B 有不同交付顺序；同位置核对 G、Tail、生命周期、q_avail，独立 checker 从公开日志、证书和执行输入重算 |
| 回归与可重放 | seeds 与墙钟扰动按合同，保留 04/05 的相关验收；输出原始 trace、已验证输入、fixture、命令及运行位置 |

单条 regular 的成功位置是**正确后继 cid 的首个新合格结果进入 G**。Tester 标注前提、各阶段实际观测时间与完整 `PASS/FAIL/INCONCLUSIVE`，说明经验时限采用的共同参与区间。向 10/11 交付可启动的 15-worker 基线，向 14 交付 trace 版本、状态查询和独立 reader 的接入资料。
