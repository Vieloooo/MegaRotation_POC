# Lightweight security patch: comparison and implementation

日期：2026-09-25。对应 [attack audit](security-attack-audit-2026-09-25.md) 的 F1–F3；用户要求比较可行方案，并采用整体负担较小的补丁。

## 方案比较

| F1 方案 | 能否恢复缺少 RC 的区域 | 正常路径负担 | 故障路径与证明负担 | 选择 |
| --- | --- | --- | --- | --- |
| 仅收窄 Progress 的保证 | 否；仍依赖另一可接管区域 | 无 | 仅修改声明 | 不采用为功能修复 |
| 同区域开启 fresh generation | 能；保留全局 prefix，从新 R0 开始 | 无新增消息类型、quorum、持久状态或轮次 | 复用一次全局 ROTATE/CONFIRM 与 activation；补充选择分支的证明 | **采用** |
| 在原区域增加 retry/abort/view-change 恢复协议 | 可以另行设计 | 依实现而定 | 需要新的协调、锁定与安全状态转移规则及证明 | 对当前需求过重 |
| 清空原 generation 的签名记录后重投 | 不安全 | 表面很小 | 破坏 DECIDED sign-once 与 quorum-intersection 依据 | 排除 |

F2 采用**顺序应用全部中间 handoff**。这些 ancestry 原本就必须验证，所以只需明确本地状态处理顺序；相比额外的 digest reconciliation 流程或清空全部 preview，它对现有行为的改动更小。

F3 采用**显式绑定已有 timing/capacity/tenure 前提**。保留原来的计时机制，不新增“至少一次 adoption 才能轮换”的控制状态。后者还需要处理真正无产出区域的退出条件，整体更重。

## 已实现的规则

1. READY 明确表示可以从 global parent 的已验证 prefix 启动下一 generation，因此旧 instance 没有 RC 不妨碍发送 READY。
2. 仍优先选择 cyclic order 中第一个其他 READY 区域。
3. **仅在** `z^+ >= K_G`、没有其他 READY 区域、当前区域自己具备 READY quorum 时，选择 `j=i`。
4. 仍然生成 `a+1`，保持 parent `(h,q)`，通过原 ROTATE → CONFIRM → finality，在固定边界从新 `R0` 激活。旧 signing history 不清除；旧 generation 的投票不复用。
5. consumer 在追赶时，按 generation 顺序执行每次已 finalized activation 和对应 preview truncation，不能直接跳到最后一个 retained height。
6. Setup、Progress、global safety、stream safety、liveness 和 recovery-cost 文字已同步。Region list 明确每个区域恰好出现一次，以支持 cyclic-selection 的进展论证。

## 为什么额外 READY 区域不会阻止恢复

设健康区域 r 始终能够为当前 parent 发出 READY。当 active i 不等于 r 时，选取第一个 distinct READY 区域不可能越过 r；若未直接选到 r，沿环到 r 的距离严格缩小。当 i=r 时，若没有其他 READY 就自重启；否则先离开，再按上述规则返回。因此即使其他区域 READY 后没有产出，也不会永远跳过持续健康的 r。采用 self-first 则没有这个优点，所以保留 distinct-region priority。

## 成本与边界

- 正常 regional broadcast、global report agreement、FFG 和消息格式不变；没有新增 quorum、计时参数或持久状态字段。
- 自重启仍需等待 no-progress guard、全局 ROTATE/CONFIRM 和固定 activation boundary；它不是快速本地修复。原 recovery bound 适用，状态缺失时仍支付 retrieval 成本。
- 新 generation 与旧 generation 的签名域不同；全局保留的全部数据继续保留，未被全局保留的 preview 仍可能撤销。
- consumer 需要处理所有遗漏 handoff 的本地状态更新。这些证据本来就需要验证，没有新增网络 round。
- `Q`、retrieval 和 workload bound 仍是执行前提；本补丁没有把它们误写成静态配置检查能够保证的事实。
- 若既没有健康 global quorum，也没有具备数据和局部通信条件的健康区域，本补丁不提供恢复保证。

## 验证记录

- 检查 1–8 个区域的 13,053 个 READY-set/active/healthy 组合：distinct-region selection 不会跳过持续 READY 的健康区域；无其他 READY 时允许 self-reset。
- 重放 audit 中保留高度依次为 100、105 的 catch-up 场景：第一次 truncation 已删除旧分支的 101–105，第二次不会让其重新出现。
- `latexmk -pdf -interaction=nonstopmode -halt-on-error main.tex` 成功，21 页；最终日志无 undefined reference、LaTeX warning、overfull 或 underfull box。
- 已渲染全部 21 页，检查总览及 D6、T3、liveness、recovery-cost 所在页面；没有发现新增裁切、重叠或公式越界。
- [独立证明复核](lightweight-security-patch-verification.md) 已通过：在所列前提下，三个补丁及修改过的证明依赖未发现 Critical Error 或 Justification Gap。有限状态检查和人工证明复核不是完整机器证明。
- 复核源码 SHA-256：`bb2e1d9c73fe89f524dfb803d01a3ace9f129fc7a408010e90584032ed1b69a6`；正文从 1350 行变为 1384 行，协议框仍为 10 个。`main.pdf` 已与 `build/main.pdf` 同步。

补丁前源码保存在忽略目录 `build/lightweight-security-patch/main.before.tex`；原 attack audit 保留为冻结版本的历史记录。
