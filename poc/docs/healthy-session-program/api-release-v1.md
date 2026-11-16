# H1-API-1 可消费接口交付

2026-11-16。**有限API切片PASS；H1/NR-T/NR-A/M1仍OPEN。** H2/H3可消费准确候选 `ed3915290f0890ffcb878f3f705df3ff790cb008`，其生产crate tree为 `1db217a616c61827d6b5c132316ddcdd007b1b71`。之后的本交付记录只增加文档，不改变已测试代码。

## 接入

独立crate：[poc-healthy-session-api](../../crates/healthy-session-api/README.md)，Rust导入名`poc_healthy_session_api`，版本0.1.0。依赖03/07既有public对象与codec，未登记或修改根workspace。接口合同见[H1-API-1](api-slice-v1.md)。

- H2生成`ManagerDelta`，经`ManagerCommitPort`提交；`RootCommitted(actions_pending)`与`ActionsApplied`分开。`ProgramPort`提供Create/Prepare/Supply/Enter/Pause/SelectBack/Close及既有观察/Ready读取。
- H1 runtime拥有真实Ready发行、root/aux持久化与安全guard。`FailClosed`保留已持久事实；API没有恢复Running或默认成功提交实现。
- H3消费`ProofObserved/RegionalDelivery/NativeVerifierPort`，提供`RegionalResultPort/ReplayFeedPort`。同pc所有完整ReplayRecord与07 LIFE-ID保留；H3不发行ReadyToken。
- [H2消费例](../../crates/healthy-session-api/examples/h2_consumer.rs)和[H3消费例](../../crates/healthy-session-api/examples/h3_consumer.rs)均为可编译DTO/注入port示例，不模拟生产runtime或默认true verifier。

`validate_shape()`只核对版本化的字段、界和绑定。完整`input_ref/effects`原样保留，API不递归验证其中所有域对象或L1字节；它不认证合法ReplayRecord、实际执行、fsync、BLS或Ready发行，真实producer/consumer仍须验证。原独立source-first/goldens与形状通过的限制均保留。

## 准确版本与验证

| 项目 | SHA/结果 |
| --- | --- |
| M0验收基线 | `35d382ac445cc8bc779f0d18fd23cb57e54680da` |
| H1保存准备/正常合并M0 | `f857675` / `4741489ba7a07d2a3f3f194dbd59ae3fd8d17bb9` |
| API合同与独立预期冻结 | `2ea612a16fb4620e5faf7191b7b9c4f32dd7b07b` |
| 初始API实现 | `f9364a2a8e59913f7f9373fab2774be4ed528e57` |
| 独立harness与报告 | `2ab84f5` |
| **最终复核及Tester同一候选** | **`ed3915290f0890ffcb878f3f705df3ff790cb008`** |
| Builder/正式Tester API测试 | **15/15 PASS** |
| 独立harness/正式Tester | **26/26 PASS**，42条来源预期分层映射 |
| 编译/格式 | 两consumer all-targets检查、两consumer实际CLI退出0、API与harness fmt PASS |

独立审查无production finding，见[报告](../../verification/healthy-session-program/api-v1/candidate-f9364a2-report.md)。最终审计通过source/dependency/harness/contract/goldens的相等绑定复用26项执行证据；Tester在最终SHA实际重跑15+26项并运行两个例子。所执行的本地ID及shape输入无随机数，`seeds_executed=[]`；actual native engines/workers=0，未运行15PID、BLS、fsync或网络仿真。

实际角色保持lead Astra/ultra、Docer/Verifier Astra/xhigh、Builder/Tester GPT-6.1 Sol/high。Rust/Cargo1.95.0，offline locked，jobs2，全重任务单外层`flock /tmp/megaroation-poc-heavy.lock`。正式测试本机墙钟约2.295s；不作为网络/协议性能结论。M0的43条矩阵未因API新增重跑。

## 原始证据与复现

```text
最终审计：
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/verification/healthy-session-program/api-v1/ed39152/final-audit.json
SHA256 00c4e7816c1fd2c7cf9b276ff7f151389ab77f0cbb90bb2a5094e9b5b3d2de75

正式Tester：
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/simulation/healthy-session-program/api-v1/ed39152/manifest.json
SHA256 66d2d52edcfee6252c3b7fea5317dd3129ff27d742d2aab13be2b0beef7a389a
```

Tester目录保留`replay.py`、完整命令/工具链/env覆盖、stdout/stderr/exit、源树/依赖/源码/运行binary hash和29项`SHA256SUMS`。H2/H3运行binary SHA分别为`1ac8a81e868362d449b3cc4b357d447021cb5c47745b7c8a69b70600a5d41ee3`和`29068453c73d34807d339f7cdc3b841197847fcb20e3c9909c24d0da3c9d0bb0`。候选与工作区运行前后不变且clean。

运行入口（在实际worktree，设置固定1.95 PATH、任务专属CARGO_HOME/TARGET、jobs2；新输出目录，不覆盖原证据）：

```sh
flock /tmp/megaroation-poc-heavy.lock cargo check --locked --offline --all-targets --manifest-path poc/crates/healthy-session-api/Cargo.toml
flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline --all-targets --manifest-path poc/crates/healthy-session-api/Cargo.toml
flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline --manifest-path poc/verification/healthy-session-program/api-v1/harness/Cargo.toml -- --test-threads=1
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline --manifest-path poc/crates/healthy-session-api/Cargo.toml --example h2_consumer
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline --manifest-path poc/crates/healthy-session-api/Cargo.toml --example h3_consumer
```

完整实际argv、时间与输出以manifest为准。harness旧开发runner绑定初始f936候选，最终SHA使用上列独立命令，不把旧runner成功冒充新候选。

## 后续切片

动态context A的kernel/mailbox原子准入、normal profile/真实公开FinalVerifier、cid-scoped driver/parking/多witness、完整normal wire/root、15worker/五真实engine、slot-aware准入与M1端到端仍需实施和独立验收。Q-ADMIT来源映射见[路线A](admission-bridge-v1.md)，它未替代真实native重放。当前继续按小切片推进，不再等待已交付M0或完整01恢复；不改共享API以虚构这些事实。
