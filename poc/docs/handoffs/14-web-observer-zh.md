# 14 Web Observer、独立视图与回放

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以 **08 的集成 SHA** 建 managed worktree `poc-web-observer`，分支 `codex/poc-web-observer`。先完成 regular 的只读 Observer，13 合入后用同接口补验四层链、Force 和 RESUME。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 交付

Docer 冻结版本化快照、增量事件、单调 trace ID、历史查询、断线续读和缺输入的表现。Observer A/B 使用不同下载调度，各自持有 GFR 与 D 状态。同 replay position 显示可核对的差异，并给出原始证据链接。服务和浏览器消费只读数据，隔离展示慢请求、断线与仿真调度。

页面覆盖 15 节点 phase/选择/cid/region/leader/view/Enter/Close、虚拟 tick、时钟偏差与租约、Clock RPC 和网络/backlog、两条链的 finalized 与 reader 位置、R/G/G′/D 四层 head、治理原因、handoff anchor/window 与验收状态。分别展示 Regional Final、G Accepted、Inbox Accepted 和 D Derived。显示来源的本地观测时刻与公共位置。未实现阶段的视图呈现缺失状态。 [v2 §11、W1](../../../formalization_v10/poc_design_zh_v2.md#11-web-observer观察协议位置而不只看区块高度)

Builder 拥有 `poc/crates/observer/`、`poc/web/`、专属测试与依赖。沿用 v2 的 Rust/HTTP/SSE 和 TypeScript/React/Vite 方案，冻结可复现的工具与依赖版本。只消费 08、11、12、13 提供的查询与 trace，协议运行、admin CLI、公共 rule 与 join 顺序仍在各自模块。主会话集成共享声明和启动命令。Verifier 写 `poc/verification/web-observer/`。

## 完成条件

- regular 初版能从完整 15-worker 场景读出两次交接，逐事件查询，并保持两个 reader 的独立 cursor。将快照和场景入口交给 15 做展示。
- 以 13 合入后的场景验证 Force 和两类 RESUME 原因、L1/DA WAIT、精确恢复位置、G 后缀变化与 D head，以及实际新分支首个 G 接纳。R、G、G′、D 与 reader 前缀均有真实来源。
- W1 对照运行慢浏览器、断线、重连以及长时间未读取 SSE。协议虚拟 trace 验证不受影响，快照及续读能回到同一轨迹位置。浏览器壁钟仅用于呈现诊断。
- Tester 交付本机启动命令与可加载记录、查询/SSE 样本、断线恢复证据，区分 UI 测试、协议重放和浏览器无背压对照。
