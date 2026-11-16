# f9364a2 API 候选独立验证

结论：**PASS — 有限 typed API / shape / consumer 切片；无发现的候选合同偏差。** 候选 `f9364a2a8e59913f7f9373fab2774be4ed528e57`，合同冻结提交 `2ea612a16fb4620e5faf7191b7b9c4f32dd7b07b` 加候选中已独立同意的 ReplayBatch 专门条款澄清。最终合同SHA256：`d713561ecc002b0ac2993539fde07a6b4b11146d8e70e9df46b29a137473677e`。

独立harness只写本verification路径，生产crate及合同只读。测试预期来自事前42 recipes、独立goldens和明确范围澄清；未读取Builder测试来生成oracle。Builder报告的15 tests不计作这里执行的测试。

## 实际结果与重放

最终26个独立Rust tests通过，0失败、0忽略；harness fmt检查通过。两个真实consumer源文件直接按path包含，实际编译并调用有关函数，没有复制其实现。第一次review-01为24 tests通过；补足basis Ready与event数量上界两个独立case后，review-02为最终26 tests。两个run原始文件均保留。

原始证据目录：

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/verification/healthy-session-program/api-v1/f9364a2/review-02/`

完整外层flock命令、锁定工具链、候选/source-tree/contract/golden/harness/log哈希及test名称见candidate-f9364a2-result.json。单个外层`flock /tmp/megaroation-poc-heavy.lock`、jobs2、Rust1.95.0；复用指定data02 Cargo cache/target。没有重跑M0矩阵、网络simulation或伪造seed；这些局部输入不使用随机数。

执行代码绑定：

- `harness/tests/independent.rs`：`568b7fb612af3ca62a83cee62d85ac637938178cb747c7083b5fee2237edf8f6`
- `harness/Cargo.lock`：`0dfde3997e27bb9284862358fa17f4e37ed7a0f21a2ed2baf09bffb9be4587b9`
- `harness/run-checks.sh`：`97df34673ef4a4ec3b6b62bfd7ddd740dec1d24ee929fd1c0a172d027c08999a`

首次默认沙箱不能写已分配data02路径；经该路径的授权提升执行成功，未发生自动批准审查拒绝。fixture初稿一处括号在首次编译前修正；没有生产修改或生产失败被隐藏。运行过程的dirty状态仅新增verification harness；原始git-status和源diff检查均记录。

## 检查范围与重要阴性

| 来源/规则 | 候选位置 | 实际检查与结果 |
| --- | --- | --- |
| API§3本地ID；03 framing | shape.rs derive_local_id | 13 literal输入、3 action和2 batch黄金、重试身份稳定，编码/域变异对照；全部通过 |
| API§4 scope/Ready/command | shape.rs WorkerScope、ReadyState、ProgramCommand | worker/cid/instance/job/anchor/root/native hash变异拒绝；root-only/空材料只可表达准备，不计真实Ready |
| d55§2/3；lead FailClosed澄清 | shape.rs ManagerDelta、RootCommitted、ActionsApplied、FailClosed | duplicate优先；所有disposition的commit scope/delta匹配；pending≤128/unique；有序子集；精确receipt覆盖/root/worker；全部通过 |
| ReplayBatch专门条款；07 LIFE-ID | shape.rs ReplayBatch | 同pc不同事件不合并；空events合法；源/previous/LIFE-ID错配拒绝；128 events允许、129拒绝 |
| 明示不深检input/effects | 同上；h3_consumer replay_batch | 已知无效Duty/Tail与65,537B L1 source_bytes完整保留且outer shape通过；这正说明实际consumer仍须验证合法性与总容量 |
| d55§5/6/7；API13 | Ready/ProofObserved/RegionalDelivery/validate_native_binding | 自洽合成Ready、单字节假proof可通过shape；没有据此声明执行或BLS成立；crypto/basis spy确认helper不调用它们 |
| H3 basis≠finality | validate_native_binding及h3_consumer | basis Ready(())与明确false verifier并存；shape/获取binding不产生验签成功、execution Ready或G接纳 |
| bounded DTOs | relevant shape guards | opaque65,536/65,537，proof32,768/+1，refs/prep/actions/guards128/+1，physical chain10/11等有限边界 |
| H2/H3消费ports | 两个examples | WAIT/Reject保持，H1原job token匹配，knownClose+缺后继材料可表达CommittedPending；均为显式测试spy，不模拟真实runtime |

所有production实现文件完整只读审阅，直接依赖仅03/07；没有新增public codec、默认真FinalVerifier或library网络/时钟/文件/签名副作用。未发现需交Builder修复的production问题。

## 限制与后续

26 tests验证静态DTO、局部纯函数和有界形状/绑定，不验证token发行真实性、真实ReadyState执行、BLS/PoP、fsync/root提交原子性、实际幂等ledger、Pause原engine/deadline、Clock/lease/stop出口、15 PID、q4生产、Publisher/GFR/官方节点或Q-ADMIT。ReplayBatch的pass不能授权root提交或动作释放。source42中这些运行时预期保持后续gate，见coverage-f9364a2.json逐项分类。

通过只供lead提交harness/报告后命名最终API候选；仍需对最终SHA做只读source等价核对，Tester在该同SHA执行API局部tests/consumer检查。整体H1/M1及旧75+2 runtime配方未关闭。旧source-first文件、goldens及其封存hash保持不变。
