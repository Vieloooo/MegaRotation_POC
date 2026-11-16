# Snapshot修复独立复核：653a065

2026-11-16。**组件复核PASS：46/46 tests，fmt PASS。** 生产候选为 `653a0654ba1765cb600e603250efa0ee40d85aeb`；本轮新增Verifier测试尚待lead提交，精确harness hash及命令见 [结果JSON](results-653a065.json)。这不是最终命名候选的复核，也不是正式Tester矩阵；lead提交后还需只读核对/执行最终SHA。

相对失败候选 `bc4111a`，生产修复仅涉及route/reliable及Builder测试；外部harness只写自己的验证路径，未修改生产代码、API或原golden。旧失败报告及原始artifact完整保留，结果JSON逐文件复查了旧artifact hash。

## 两项finding关闭

- **F-01：关闭。** Route restore以u128核对必要raw_due上下界。来源时间和最后提交tick夹住实际Send时刻，固定latency与该消息的jitter保持明确；只使用已发生ordinal的fault范围，分别限制primary与duplicate，未发生的未来fault不扩大窗口。原`raw_due→u64::MAX`负例已拒绝。
- **同类pacing字段：通过。** `next_release_us`上界在queue-empty分支之前检查。独立测试对有队列和空队列分别将其改为“最大可能值+1”及u64::MAX，均得CorruptSnapshot；合法近u64时刻保留的pacing不会因理论上界加法超过u64而误拒。
- **F-02：关闭。** Active和tombstone统一检查 `origin.sent + last_attempt×interval ≤ retry_due ≤ last_action + interval`（零次attempt右端不加interval），中间值用u128。原retry_due→0负例拒绝。新case还验证两次Attempt不能留下仅一次interval的deadline，较新near-u64 watermark不抹去合法旧deadline。

同配置只读bft_verifier子角色独立复核了F-02的必要上下界及10组整数边界，无写入/构建；本Verifier通过外部public API执行全部Rust用例。检查针对可由现有snapshot和固定profile推导的必要局部条件，未要求没有prior history时证明唯一完整历史。

## 新增独立控制

新增 [snapshot_recheck_cases.rs](snapshot_recheck_cases.rs) 的4项测试：

1. `next_release_mutants_reject_with_nonempty_and_empty_queues`
2. `valid_near_u64_pacing_and_raw_due_roundtrip_without_bound_overflow`
3. `queued_fault_bounds_use_past_primary_duplicate_and_unlisted_ordinals`
4. `tombstone_attempt_count_bounds_keep_old_due_under_near_u64_watermark`

负变异使用原始canonical JSON局部替换，保留字段顺序；检查结构拒绝而非仅碰到NonCanonical。正控制包括primary/duplicate到期分别为MAX−17/MAX、延后No-op让理论上界超过u64、空payload最小pacing、旧fault+unlisted ordinal共存以及未来巨大fault未生效。实际在MAX时再次Drain产生pacing加法溢出仍须原子失败，合法snapshot本身可恢复。

mock receipt仍只是可信adapter声明；重算恶意record的mock hash用于进入组件局部检查，不是实际存储认证攻击。Receive仍不推进last_action_tick，原`c49322…`golden记录逐字节通过。

## 原始证据与剩余范围

```text
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/verification/sim-communication/kernel-v1/653a065/
```

`harness-recheck.log`记录46 PASS，`harness-fmt.log`记录格式检查；`case-inputs/`包含旧负例在新候选上的输入、新pacing/near-u64/fault/tombstone控制及12个纯组件诊断trace。保存函数现使用create_new；若同名文件已存在，只接受完全相同bytes，不覆盖原证据。

46条原静态向量的执行映射见 [coverage-executed-653a065.json](coverage-executed-653a065.json)。全部K-01–07仅为有限组件范围通过，active native engines=0。真实Clock RPC、root fsync/进程恢复、BLS/native四channel、实际reader业务与完整SC-01–18仍未验收。正式12组Tester矩阵须在最终复核通过的同一SHA上重新运行。
