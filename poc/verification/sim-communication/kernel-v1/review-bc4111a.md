# 独立组件审查：bc4111a

2026-11-16。结论：**FAIL，需修复两项P2局部snapshot不变量**。生产候选为 `bc4111aab6240b364a1a9c632d1bf02474a57272`；来源/API/独立期望基线为 `444613345e045e861f94298ac68042050bb8e4db`。本轮只新增外部harness和报告，生产crate未改；harness尚待lead提交，精确内容hash与命令见 [结果记录](results-bc4111a.json)。

最终harness共42 tests：40 PASS、2 FAIL。覆盖46条独立向量及追加的Hold/origin/active ACK/局部snapshot边界；fmt检查通过。完整覆盖映射、原始日志和fixture hash均在结果记录中。原receive record的`last_action_tick_us=0`及`c49322…`黄金hash保持不变，四份完整custody记录逐字节匹配。

## F-01 / P2：恢复接受不可能的raw_due

位置：[route.rs](../../../crates/sim-transport/src/route.rs) `RouteKernel::validate_snapshot`，候选第721–742行仅核对raw_due下界。

最小case为`route_cases::snapshot_raw_due_respects_fixed_profile_and_last_committed_tick`：固定latency20、jitter0、无fault，唯一Send在tick1000；合法snapshot的`raw_due_us=1020`。保持JSON字段顺序，只改该值为`18446744073709551615`，`from_snapshot`返回成功，`next_wakeup_us=Some(u64::MAX)`。该状态违背从最后提交tick和固定profile直接可推导的上界，会错误推迟既有消息。

依据：kernel合同§4的checked raw_due计算、§6的snapshot局部一致性，API§4的计数/绑定/结构核验；独立KV24/K03及K07负例要求。应只增加能从当前snapshot和固定profile推出的必要上下界，不要求无prior state时证明唯一历史。上界中间计算应使用u128，避免`last_tick+最大delay`溢出而误拒其他合法近u64边界状态。

原始fixture：`mutants/route-valid-snapshot.json`、`route-impossible-due-snapshot.json`、`route-mutant-config.json`。

## F-02 / P2：tombstone跳过原retry期限校验

位置：[reliable.rs](../../../crates/sim-transport/src/reliable.rs) `MailboxSnapshot::validate`，候选第758–763行对acknowledged条目直接continue；第785–791行的retry时间关系仅用于active条目。

最小case为`custody_clock_cases::snapshot_tombstone_preserves_possible_original_retry_deadline`：使用独立golden delete record，origin tick1000000、last_attempt1、retry_interval20000，原`retry_due_us=1020000`。仅改candidate中的retry_due为0，并为该恶意明文record构造匹配的mock receipt hash，restore仍成功并返回Acknowledged release。已经确认并清除payload不能使保留的原retry时间关系失效。

依据：kernel合同§5保留原retry义务与§6局部snapshot检查，API§4明确tombstone保留历史retry_due，KV46/K04/K07要求结构负例被拒绝。可用保留header、last_action_tick、retry_interval与last_attempt检查必要时间关系。**这里没有声称伪造了真实storage receipt、绕过密码学认证或trusted floor**；mock receipt用于让测试进入明确由组件负责的局部结构校验。

原始fixture：`mutants/tombstone-valid-record.json`、`tombstone-impossible-retry-record.json`、`tombstone-mutant-mock-receipt.json`。

## 证据与复现

原始证据绝对目录：

```text
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/verification/sim-communication/kernel-v1/bc4111a/
```

最终日志为`harness-freeze-review.log`和`harness-freeze-fmt.log`；此前各轮日志独立保留。42 tests包括默认1024 queued copies、4MiB payload预算、8192 tracked IDs，以及4096 commands、8192 events和32MiB plan编码上限。全部Rust调用使用同一外层flock、固定1.95工具链、CARGO_BUILD_JOBS=2和02独占target/cache；没有并行build。

精确过滤测试名称如上；可在结果记录中的相同命令后增加完整test名称及`-- --exact --nocapture --test-threads=1`。复跑必须换新的artifact目录，保留此处原失败证据。完整命令、工具版本、harness文件hash和原始artifact hash由结果JSON保存，不靠报告摘要代替原始输入。

初次38-test运行曾将“低于最近destination incarnation”的错误类误写为InvalidReceipt；依据冻结API§7修正harness断言为BindingMismatch，行为预期仍为拒绝且原子不变。未修改独立golden或生产实现来迎合结果。之后两项FAIL均是上述生产snapshot缺口。

## 范围与后续

命名角色为bft_verifier/Astra/xhigh；另一个同配置只读Verifier先读来源后独立定位snapshot问题，无写入/构建。主Verifier通过外部Rust API独立复现。纯组件12组seed/config-declaration/wall-pause对照及原始trace只属于本失败候选的诊断，**不是最终Tester矩阵通过**。active native engines=0；无真实RPC、磁盘fsync/BLS、native多进程或业务reader前缀验收。

修复由Builder在生产scope进行。lead先保存本harness和失败报告，再对修复后的命名SHA复核；只有复核通过后，bft_simulator才能在同SHA重跑正式组件矩阵。完整SC-01–18与01/Fresh/I1a/native接线仍按 [coverage](coverage.md) 保持开放。
