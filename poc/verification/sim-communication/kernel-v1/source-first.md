# Transport kernel v1：来源先行预期

阶段：独立组件实现前准备；本文件先于新 kernel 合同/API 阅读落盘。未读 Builder 源码，未构建或运行组件。Verifier 为 `bft_verifier` / `gpt-6-astra` / `xhigh`。

准备基线：`c64709871827128aae974343bc28bdafb9f89486`，已验证包含依赖安排 `0a951c4` 与原准备提交 `125d8cf4e30b5affe3cd731fbb5d4a73d0e40a10`。Regional BFT pin 保持 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。当前独立组件只允许 `poc/crates/sim-transport/`；本 Verifier 只拥有本目录。native/I1a/types/根manifest均不属于本阶段。

已读 [dependency-plan-v2](../../../docs/commander/dependency-plan-v2.md)，SHA-256 `4dcaca52eda5c680151e9d2c06c69d2ad7af9d96dba8ab3b14033605625f6762`。既有 [64向量](../vectors-v1.json) SHA-256为 `dfd018e2ee9d18d902c00c943c8cbfd28db9cc3388cc8e0671ffb0f7e3939920`。规则依据沿用 [source-map](../source-map.md) 中 V2-ID/CLOCK/RUNTIME/NET/READERS、I1-SCHED/CLOCK、RECOVERY、CW-P2P/CLOCK、H02；没有另定协议签名域或公开日志canonical。

在读取新 API 前已向 lead 交付以下独立预期：

- Replica(0)、Service(0)、Reader(0) 是不同命名域；public pid0/5/10的局部member均为0不能造成路由别名。context是外部验证的opaque引用；kernel不生成Descriptor或验证BLS。
- 逻辑message/origin/payload身份跨attempt与当前incarnation保留；同ID不同payload/context/origin拒绝。65,536-byte payload为合法边界，65,537拒绝；边界通过不等于native语义有效。
- prepare route plan只读；commit整批验证且绑定Kernel/config/revision/content。非法尾effect、stale plan、跨instance、caller篡改都不能修改queues/counters/outbox/dedup/可见事件。clone plan在首次commit后不能再生效。
- 容量2的三条合法消息可产生2条入队和1条可见drop；第三条若是格式非法则整批零变化。normal overflow与malformed rejection分开。
- 同link send=[1000,1001]，delay+jitter=[20,1]，raw due=[1020,1002]仍m1先；跨link允许m2先。hold区间[1010,1030)内raw due1020不能早于1030投递；若最终选择drain interval5，后继不早于1035。精确jitter/tie字面值在算法冻结后独立计算。
- 三个durable边界独立：sender enqueue确认后首次发送；receiver custody确认后逻辑交付/receive ACK；sender删除确认后退休。错误/缺失确认不能释放对应效果。mock receipt只测试状态机，不证明fsync。
- adapter须独占串行prepare→persist→confirm。若已persist意图遇stale，不可将拒绝解释为durable义务撤销；必须失败关闭并交01恢复协调。真实不丢义务仍需01磁盘/进程证据。
- Clock/scheduler输入来自明确能力边界，不以墙钟推进，不把route ACK当校准。reader只获得自己endpoint交付；公开前缀、ReplayPosition和canonical排序归06/03。

原SC分层：SC05为纯运输算法；SC01/03/04/06/07/08/09/13/14/15/16/18只能证明组件子断言。SC02、SC10–12、SC17及真实PID/native签名、磁盘恢复、RPC/lease、完整reader公共前缀必须保留集成gate。若新组件提供Clock纯状态模型，只能增加模型级断言，仍不等于真实Clock RPC/current/signing出口通过。

下一步才读取新合同/API，从其明确的适配算法推导独立字面值，并在本目录建立外部Rust harness；任何实质歧义先报lead，不能根据Builder实现倒推期望。
