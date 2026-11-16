# 04 15-worker Session Program 与 REG-A

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以含 **02 与 03 验收产物**的集成 SHA 建 managed worktree `poc-session-program`，分支 `codex/poc-session-program`。启动 15 个独立 worker 并在一个地区运行五个真实 Simplex 引擎，完成 slot-aware Build/Accept 与真实验签入口。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。遵循[共用工作约定](workflow-zh.md)。

## 规则与当前约束

现有 native fixture 使用 2 PID、最多 3 脚本 signer、短窗口和有限 view。v10 已定义 Create、Enter、Close、Build、Accept、NotBefore 和 CheckFINAL。REG-A 按 3×5 registry 运行，选中委员会的五名 signer 都是真实进程，quorum 为四。REG-A 之后由 05 实现多 cid 停车调度，08 接入完整三区 handoff。 [v10 Session Program §3–4](../../../formalization_v10/02_session_program_zh_v0_1.md) · [regular 准备稿](../contracts/regular-preparation-v1.md) · [v2 §6、§12](../../../formalization_v10/poc_design_zh_v2.md)

## 需要实现

- 15 个真实 PID 的启动、注册和独立 Clock/就绪。先给 R0 的五个 member 安装一个真实 session，其余十个 worker 获得各自输入与 Clock，保持无该 cid 签名权限。把原型内固定 2 endpoint、view 1–8、static body profile 的限制放到各自测试配置。
- Create/Ready/Enter 的合法条件与一次性保证。Build 跟随真实 native parent，在固定 10-height 窗口与 NotBefore 下使用 03 的轻量执行和已交付 L1 输入。Accept 落实 5a–5d；在返回有效 propose/verify/certify 和实际新签名出口前重查规定的持久状态与时间守卫。
- 冻结 receipt key、直接 leader/已验证转发/本地 Build 的权威、header 与完整 body 的原 ingress。合法完整 receipt 取两者较晚的 ingress，保留最早适用值跨回调、重试与重启。缺输入或等待时间时保留活 callback；永久本地时间拒绝按准备稿的 abstention 策略。历史证书 CheckFINAL 使用公共材料，复用 Commonware 实际验签入口供 06/07 使用。
- Docer 固定 Slot、T_win、同步与 timer 的边界，建入站认证的处理预算。健康参数和时限以实际 native runs 核验。每个节点记录 descriptor、anchor state、clock ready、Enter、view、Close 的时刻及共同参与区间。

Builder 拥有 `poc/prototypes/i1b-runtime/src/` 中合同列出的 application、replica、config、elector/scheme、worker/hub 的启动和准入接线。使用 02 已验收的网络、03 已验收的排班和编码。主会话协调公共 API、proto 及依赖，保持 01 的恢复和 02 的路由语义。独立用例放 `poc/verification/session-program/`。

## 完成条件

1. 健康时依真实 slot 在固定窗口产出 10 个 physical block，证明、完整验证交付及应用执行可独立核对。3/4 不同 signer 的边界、重复 signer/错域/错 subject 和 10 个非成员的签名出口都有负例。
2. 覆盖 Accept 5a–5d 等号两侧、body/parent/L1 缺输入、延迟认证与重启后首收不变、NotBefore、无效 leader 与错 parent、Close/lease/stop 与待完成工作相交。
3. 执行 0/20/100/500/1500 ms 错时 Enter、三早两晚、late join 较高 view 和 3/2 cid 分裂诊断。只有同 cid 固定 quorum 的共同参与区间支持进展时限；全候选切换由 05/08 验收。
4. Tester 用固定 seed 和墙钟扰动运行 15 进程，签名和 delivery 全部来自经过认证的真实 worker。记录实际 PID/身份、路由、节点时钟采样、L1 prefix 和证书。将 Session Program API、验签入口与 REG-A 集成 SHA 交给 05/07/08。
