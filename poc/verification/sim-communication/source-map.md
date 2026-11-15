# 独立来源映射与读取边界

以下来源（FRESH 补读说明除外）在阅读功能 02 新合同及 PoC native 实现之前用于推导预期。基线 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，Regional BFT gitlink 与只读源 checkout 实际 HEAD 均为 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。只读源路径是 `/mnt/reth/jy/megaroation_poc/regional_BFT_sim`，未初始化本 worktree 的子模块，也未改子模块历史。

| Source ID | 来源及具体范围 | 推导义务 |
| --- | --- | --- |
| V2-ID | [v2 §§2–3](../../../formalization_v10/poc_design_zh_v2.md) | 3×5、public pid=5×region+member；路由身份与 signer/public-key 顺序显式映射；每委员会 quorum=4，不是全局 15 人委员会。 |
| V2-CLOCK | [v2 §§4.1–4.4](../../../formalization_v10/poc_design_zh_v2.md) | τ、C_m、RPC、公开键、墙钟分离；rate=1、ε∈[-1000,1000]；grant 内固定钟；有效 RPC reply 才续租；ClockUnsafe/Terminated。 |
| V2-RUNTIME | [v2 §§5.1–5.2](../../../formalization_v10/poc_design_zh_v2.md) | 单一调度、完整 grant/Ack、受控 work、所有 observable output、无 worker 自行快进；脚本 crash 与基础设施故障分开。 |
| V2-NET | [v2 §5.3](../../../formalization_v10/poc_design_zh_v2.md) | 默认区内 link 有序，channel 分流；跨区 final/proof/body/state 可靠 outbox/ACK；raw vote 保留依赖原生修复；Close 保留历史输出。 |
| V2-START | [v2 §6](../../../formalization_v10/poc_design_zh_v2.md) | 独立 Ready/Enter；不重置绝对窗口；不以全员 Ready 屏障掩盖错时进入。 |
| V2-READERS | [v2 §§4.5、7、8.1、10、11.2](../../../formalization_v10/poc_design_zh_v2.md) | public timestamp 与取时分离；各 reader 自己的交付前缀；治理/协议不能读取全知 fault script；Web 无协议背压。 |
| V2-EVIDENCE | [v2 §§12.2–13.3](../../../formalization_v10/poc_design_zh_v2.md) | C1/C2/I2、固定 seed 重放、完整证据和有限边界；Proof/Delivery/G/Inbox/D 不混为一谈。 |
| SP-LIFE | [v10 Session Program §§3.1–3.8、3.12、4.4、4.9](../../../formalization_v10/02_session_program_zh_v0_1.md) | first_received、签名与不可逆终止跨重启；不同 cid 不转用签名；历史证明可以在 stop 后出现。 |
| I1-SCHED | [I1 §§4–6](../../docs/contracts/task-contract-i1-v1.md) | 1 MiB frame、64 KiB payload、64/link/channel、全局一个 outstanding grant；整份 Ack 先验证后提交，重复幂等，tie 不依赖 OS。 |
| I1-CLOCK | [I1 §§7–9](../../docs/contracts/task-contract-i1-v1.md) | 半开 lease、原 timer、有效样本全 guard、拒绝不污染水位、旧 lease 到期恢复保守终止。 |
| I1-ANNEX | [I1 interface annex v1](../../docs/contracts/i1-interface-annex-v1.md) | 已冻 I1a 字节/hash、取时分段、default 100ms lease/20ms refresh、原绝对 timer、seed 7/42/99 与墙钟 0/2ms 对照。 |
| RECOVERY | [native recovery §§6–11，amendments 2/4/6](../../docs/contracts/task-contract-native-recovery-v1.md) | 旧 signed/root Ack 字节保留；current incarnation 包装原逻辑效果；old timer/work/ClockRequest 不直接重排程；原 deadline、startup fence 与接收保管。 |
| FRESH | [Fresh 的 Fixed registry / Numeric profile / Transport 条款](../../docs/contracts/task-contract-native-engine-fresh-v1.md)；[recovery amendment 3](../../docs/contracts/task-contract-native-recovery-v1.md) | Fresh 字节、hash、JSON omission 与独立回归保留；现有 2 worker/scripted peers 不能冒称 15 active engines。 |
| CW-CLOCK | Commonware 2026.10.0 runtime `src/lib.rs:525` Clock | current 同步；sleep/sleep_until 是 future。同步调用不能等待网络。 |
| CW-P2P | 同版本 p2p `src/lib.rs:58–135`，`src/simulated/mod.rs:1–11` | accepted 不等于收到；offline 可丢；单 link 保序；与 deterministic runtime 结合才有其确定性承诺。 |
| RBFT | 固定 Regional BFT `DESIGN.md` §§1、2.1、3.1、5 | 证据与交付分开；丢失后的进展需要明确修复；消息 context/signing-domain 不由 channel 名替代。 |
| H02 | [功能 02 交接](../../docs/handoffs/02-sim-communication-zh.md) | 全部 endpoint、原子拒绝、outbox 三 crash、C1/C2、真实 native channel、回归与下游接口的任务义务。 |

FRESH 的字节兼容/保留回归预期先从已读 RECOVERY amendment 3 与 H02 得出；Fresh 完整合同在来源框架形成后补读，用于核对现有 profile 和回归入口，不冒称该全文也在第一次实现定位之前已读。

Commonware 缓存只读根：`/mnt/reth/jy/regional_BFT_sim/.tooling/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`。已读 `commonware-runtime-2026.10.0/.cargo_vcs_info.json`，revision 为 `92a2f68a0e131c8b69de41e84aec37da6280f212`。固定 DESIGN 指向 Simplex ePrint 2023/463；本阶段依据锁定 Commonware 接口和既有适配合同保留共识行为，没有重新证明或修改论文规则，也未采用 Kudzu/Minimmit 的 15-member 参数。

| 文件 | SHA-256 |
| --- | --- |
| v2 design | `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab` |
| v10 Session Program | `a8cafc846acffda6b2d08915b9ccc251825c33cf84e776dae94d67611db47eb2` |
| I1 contract | `04658debb7888b1fed9835b0a91c05b3ae34b71cbaf9696c0693c55b5a2c0504` |
| I1 interface annex | `f7451d889d897ce988fee6756a62fd87bf722c90f04faf867d353f5a6d79d12d` |
| native recovery contract | `3dd752806c22fce5069667e76a435c55c15bf9aa346cfb611648f14b0f587b99` |
| handoff 02 | `385ef4198fe1c400a30603d5b929beadcd380689ccc33b7eb25a66b23f735851` |

## 预期形成后才查看的基线定位

这些是 **ada48a5 的只读适配定位**，不是新候选审查，不是未来接口已经存在的证据。01 接受后必须重新核对实际行号和行为。

| 当前路径/位置 | 只读观察与下一阶段边界 |
| --- | --- |
| `poc/prototypes/i1b-runtime/src/native_config.rs:53,99` | epsilon 长度为 2，Resolved 单 cid；新 registry 不能只把 member 当 global pid。 |
| `poc/prototypes/i1b-runtime/src/native_hub.rs:266,303,1111` | 注册/运行使用两个 worker，keys 在 2 处分割为 scripted peers；真实 PID 与 active engine 必须独立报告。 |
| `poc/prototypes/i1b-runtime/src/native_network.rs:118` | member<5，provenance 依据 member<2/≥2 区分本地与脚本；全局 endpoint 映射不能破坏此旧 profile 的身份含义。 |
| `poc/prototypes/i1b-runtime/src/native_hub.rs:666,761,1468` | AckPlan 先克隆/计划，再提交 worker/timers/routes/attempts/events/counter/logs；新 outbox、dedup、prefix、trusted floor 也必须包含在事务验证观测集中。 |
| `poc/prototypes/i1b-runtime/src/native_hub.rs:795` | stage_packet 按 member/channel 队列、固定 delay、attempt 连续；无通用 service endpoint 或可靠 outbox 的完成声明。 |
| `poc/prototypes/i1b-runtime/src/native_wire_support.rs:23`；`poc/proto/native.proto:54` | Fresh message hash 与 VOTE/CERTIFICATE/RESOLVER/BODY 已固定；新外层 route 不能改签名字节或悄改 Fresh hash。 |
| `poc/prototypes/i1b-runtime/src/driver.rs:378,841,1371`；`src/clock.rs:145,207` | 受控 driver、clock checkpoint、timer 恢复位置；需与 01 接受后的 startup/recovery API 对齐。 |
| `poc/crates/sim-hub/src/scenario.rs:18,30,58` | I1a clock overrides、link fault 配置可复用语义；当前只有两节点。实际主调度文件为 `hub.rs`，不是 `engine.rs`。 |
| `poc/verification/{i1,native-engine-fresh,native-recovery}/` | 既有独立入口；本阶段未运行，旧 artifacts 不代表功能 02 通过。 |
