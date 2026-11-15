# 02 全局虚拟时钟与仿真通信

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以 **01 验收并合入的 SHA** 建 managed worktree `poc-sim-communication`，分支 `codex/poc-sim-communication`。完成可供多 session 和服务使用的时钟与通信层，接在真实 native worker 的路径上。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。按[共用工作约定](workflow-zh.md)交付。可在 01 实现期间先准备合同和独立输入。

## 起点与本次范围

I1a 已有租约、计时器和有界链路队列。当前 native hub 仍使用双 worker、单 cid 和固定本地链路的 fixture。复用两条路径中的既有受控语义，在 native 正常入口接入可配置 endpoint、路由和故障；保留 I1a 与 Fresh 作为回归。此任务验收到时钟与真实网络事件，04 在同一通信层跑 15 个真实复制节点的 slot-aware Session Program。 [v2 §4–5](../../../formalization_v10/poc_design_zh_v2.md#4-时间设计rpc-服务是时间权威) · [I1a 链路配置](../../crates/sim-hub/src/scenario.rs) · [native hub](../../prototypes/i1b-runtime/src/native_hub.rs) · [原生恢复合同](../contracts/task-contract-native-recovery-v1.md)

## Docer 冻结的接口

| 边界 | 合同写清 |
| --- | --- |
| Endpoint 与 envelope | 公共 pid、service/reader ID、region/member、incarnation、可选 cid、run/config、channel、稳定逻辑 message ID、原发送位置、attempt、payload 字节/摘要。外层路由和证书身份映射分开 |
| LinkProfile | 有向链路及 channel 的匹配规则、同链路默认 FIFO、跨链路交错、同 tick 排序、seed、延迟/jitter、online intervals、queue bound、overflow、丢失、重复、重试和恢复 drain |
| 可靠通道 | 跨区 final/proof/body/state 的 durable enqueue、receive ACK、重发与删除边界。确定哪些 channel 用原生 vote/certificate/resolver 修复，哪些通过可靠 outbox |
| Clock 与授权 | SimHub 只推进虚拟时间，grant 内固定 ClockView。RPC 分开请求、采样和响应。有效 sample 唯一续租，置入受控 timer 的绝对 deadline 不随重试/恢复改变 |
| 服务接入 | Publisher/Espresso、L1、DA 与独立 reader 使用同一调度接口。不同 reader 只收到各自实际交付的前缀。admin 故障动作、wall watchdog 和协议 crash 有可区分记录 |

共享接口须注明 root-step Ack 的原子性、跨 cid/context 的拒绝条件以及现有 Fresh profile 的兼容要求。默认 rate=1、逐节点 offset 相对虚拟参考时间 ±1 ms。模拟 RPC 延迟独立改变样本年龄与就绪时刻。 [时钟与调度合同](../contracts/task-contract-i1-v1.md) · [接口附件](../contracts/i1-interface-annex-v1.md)

## Builder 所有权

由 01 交出 native `native_hub.rs`、`native_config.rs`、`native_network.rs` 和 `driver.rs` 的 route/transport 部分。需要复用或抽出队列时，在合同列出具体新路径，沿用 `poc/crates/sim-hub/`、`poc/crates/sim-runtime/` 中前置逻辑。主会话集成本任务的 `poc/proto/` 与 native wire，`poc/crates/types/` 的公共协议域对象交给 03。总协调处理重叠的 workspace/lockfile。独立案例和轨迹检查放 `poc/verification/sim-communication/`。04/05 在本任务集成后接手原生接线。

## Verifier 与 Tester 的完成条件

- 验证 15 replica 和附加服务/reader endpoint 可配置、注册和按 cid 交付，包含多个 public pid 映射到各区相同局部 member 的对照。真实 native vote/certificate/resolver/body 通过 route 验收；原样保存经过 Ack 提交的逻辑消息和签名。报告实际开过的 PID、active engine 和覆盖的 channel。
- 注入不同 cid、run、config、destination、旧 incarnation、payload 超界和未授权 Ack。验收拒绝原子性，绝不生成半份提交或投递。
- 逐项验证 FIFO、跨 link 重排、overflow、分区恢复和 drain。测试可靠 outbox 在持久 enqueue 后、发送后 ACK 前和 receiver 已接收但 ACK 丢失时的重启。观察重发 attempt 与单次逻辑交付，覆盖已关闭 cid 的历史结果继续转发。
- 运行 C1/C2 的 offset 两端、RPC 高延迟/丢回复/乱序、旧样本与 lease 边界。核对真实每次读钟、原 timer、lease 后的签名门禁和 Terminated 持久状态。
- 同 seed 与逻辑输入下，在启动顺序和 0/2 ms 墙钟扰动下比较完整虚拟 trace。回归 I1a loopback、Fresh 和 01 的受影响恢复用例。记录输出来向各模块的接口示例、真实终端覆盖和未覆盖项；向 04/06/09 提交可接入的路由和 Clock API。
