# H1-API-1 独立冻结复核

结论：**READY_FOR_SINGLE_BUILDER。纯API切片符合来源；lead已采纳下述两项狭窄shape澄清，作为冻结补充。** 不等待Q-ADMIT，不等待已交付M0或整项01。没有Rust实现候选，编译、API tests、密码学、fsync及runtime全NOT_RUN。

审阅合同：`poc/docs/healthy-session-program/api-slice-v1.md`，SHA256 `c8a3bbad1a27bacc1a672aab1df48ee007a66cc415f9d4da4ac2115242d21cb3`。先保存42条独立预期与来源hash（independent-capture.json），之后才读本合同。独立原始包不为适应合同而改写。

## 来源一致性

公共03/07类型和codec保持唯一；本地ID明确不冒充cid/lifecycle id/public编码；单worker持久owner、ManagerDelta/intents、RootCommitted与ActionsApplied分层、knownClose不等Ready执行状态、终止失败单调、原实例Pause、exact ReadyToken、subject-only proof和完整Delivery分开，均与d55及H2/H3来源一致。

`LocalId`是共享不透明控制ID类型，不具有每个IdKind的Rust类型隔离。可重算的Action/ReplayBatch ID由shape检查；Program/Ready/其他原身份真实性和同id冲突由后续owner ledger核对。此明确限制不应被组件测试描述为不可伪造capability。

DTO可手工构造；shape只证明非零/长度/类型/绑定和结构自洽。即使ReadyState root相等、receipt字段完整、profile SHA格式正确，也没有证明执行正确、真实fsync、提交存在或BLS有效。API13明确保留此限制；没有默认true verifier或隐含副作用。

## 精确黄金向量

goldens.json由本目录记录的Python命令直接实现合同§3 framing推导，不调用被测Rust或候选helper。3个Docer向量及209B preimage一致。另有：

- 13个合法LocalId输入，包括8个kind、ordinal10、pid0/14及不增加公共约束的零opaque config/subject。
- 6个编码/域/绑定变异：裸pid、整数LE、错误domain、其他worker/config/run。
- 3个严格从delta.id派生的action ordinal向量。
- 2个ReplayBatch sequence/previous链向量。

这些只是SHA-256控制ID黄金数据，无伪BLS签名。API实际一致性要在Builder命名候选上执行后才能报PASS。07公开codec只调用既有实现对照消费，不在新crate复制算法。

## 两项已由lead采纳的狭窄shape澄清

1. `api-slice-v1.md:374` 的FailClosed目前只在CommittedPending句子明确要求scope/delta与commit相等。lead已决定**任何committed=Some**均要求两者匹配；无commit时也显式限制actions_pending≤128且unique。最小负例为RuntimeStopped/GuardsRetained装入其他worker/delta的RootCommitted。此为自洽性shape边界，不要求API证明durability。
2. `api-slice-v1.md:371` 同时要求ActionIntent ID按ordinal重算以及重复报DuplicateId。第二项重复第一项ID同时也是派生值错误，lead已固定**先检查重复，再逐项对比派生ID**，避免Builder与独立测试选择不同首错。

上述不改变协议/路径/依赖；lead已明确采纳并将同步合同FROZEN状态，不需要用户重授权。其他独立shape守卫沿合同原顺序，不扩大错误优先级变更。其余错误类型按合同已有规则；不因opaque元数据不能证明外部真实性而阻塞DTO切片。

## 后续候选核对范围

先在单crate检查typed公共值可直接消费、本地ID黄金、全部shape边界、H2/H3两个编译例；检查无新public codec、无runtime/Clock/IO/crypto/选中逻辑。之后才能把该命名API候选交H2/H3。完整H1/M1与旧77个runtime recipes保持原层级NOT_RUN/OPEN。

API不冻结Q-ADMIT候选A；其安全/条件活性与native hook分析独立进行，不能用本报告释放native voting路径。
