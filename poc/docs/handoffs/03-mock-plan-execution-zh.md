# 03 Mock Plan、公共对象与轻量执行

> 调度以[五chat分层依赖v2](../commander/dependency-plan-v2.md)及最新总协调派发为准。下文原基线要求约束真实接线/完整验收；获明确分配的独立组件可先实现。

> **新会话启动任务**
>
> 在 `/mnt/reth/jy/megaroation_poc` 上，以 **01 的集成 SHA** 建 managed worktree `poc-mock-plan-execution`，分支 `codex/poc-mock-plan-execution`。完成 P0 排班、公共对象的规范编码和共用轻量执行。与 02 分开生产路径。主会话用 Astra / ultra。走 Docer `bft_spec` → Verifier `bft_verifier` 先写独立期望 → Builder `bft_builder` → Verifier 复核候选 → Tester `bft_simulator`。按[共用工作约定](workflow-zh.md)交付。

## 当前基础与权威

v2 §3 已给出静态排班、五成员 quorum 和授权接口。原型的 `MemberElector` 独立实现区内轮转。regular 准备稿给出 physical block、原生 hash 与收包语义的方向，具体字节和边界仍待冻结。 [v2 §3 与 §10.2](../../../formalization_v10/poc_design_zh_v2.md) · [regular 准备稿](../contracts/regular-preparation-v1.md) · [Native elector](../../prototypes/i1b-runtime/src/native_elector.rs) · [v10 架构](../../../formalization_v10/00_arch_zh_v0_1.md)

## 本次交付与所有权

1. 冻结 15 个公开等权身份、PoP 校验、registry/config hash，定义 `StaticScheduleProvider`、`PlanRecord` 和 `Leader`。用 `i=(β+r)%3`、`j=(v−1)%5` 与 `pid=5i+j`。明确 pid／region member／Commonware 排序 signer index 映射、合法域和上溢/坏输入处理。`reference_block` 与生产 anchor 各自独立记录。
2. 冻结 `Boundary`、`Plan`、`Descriptor/cid`、`AnchorEvidence`、`PhysicalBlock`、`RegionalResult`、`ReplayPosition`、`LifecycleEvent` 的共用编码或最小骨架。明示未来扩展的来源与版本。定义公开验签接口与证书绑定向量。04 提供同接口的原生证书验证接线。ResumeCut 和 ABI 的完整细节由 09/12/13 补充。
3. 实现配置绑定的 `dep[h]`、实际 metadata/origin、nonce/balance 的轻量执行、Kept 过滤与 Merge。physical root 使用必要 padding 后本块 Begin/DEP 前的状态。供 G/D 比较的 native hash 使用实际 parent、metadata、DEP、Kept 与执行后 root。height、view 和实际 parent view/digest 分开表达。
4. 提供受限的 L1 输入 fixture 与逐读者 finalized gateway，供 REG-A 和规则用例使用。对未来、缺行、缺控制覆盖和损坏来源给出可测试结果。真实 Anvil 读者由 09 实现相容的输入接口。

建议 Builder 拥有 `poc/crates/authorization/`、`poc/crates/core/src/{execution,metadata,merge}/` 以及单独的 fixture 路径。主会话拥有 `poc/crates/types/` 的公共域对象、workspace/lockfile 和 `core` 的公共模块声明。冻结前先核对 04/06/07/09/12 的消费方式，只有一处 canonical 编码。独立向量放 `poc/verification/mock-plan-execution/`。

## 通过标准

- 独立重算跨 branch/r/view 的排班、边界与地址映射。含 `(0,0,1)→(0,0)`、`(0,0,2)→(0,1)`、`(0,1,1)→(1,5)`，以及乱序 key、重复身份、错误 PoP、v=0、无效 region/window。
- 提供版本化黄金字节、摘要与公共证书 subject/domain 用例。anchor 的高度相同但 hash 不同会改变 cid 和连接判定。维护 3/4 signer、重复 signer、错误 cid/view/kind、错误 parent/body 等向量供 04 实际验签。
- 验证会改变状态的转账、合法格式但无效 nonce/balance 的过滤、非法编码拒绝、padding、高度 gap、错误 parent、缺 metadata/DEP 和重复执行的结果。独立 checker 用固定交易字节重算 pre-root、post-root、physical hash 与 native hash。
- Tester 回传固定输入和期望摘要、锁定检查结果，以及下游可以直接调用的最小接口例子。完整 REG-A 的五引擎结果由 04 提供。
