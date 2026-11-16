# M4 回放输入字段

M4 后端合同已在 `a5ed3a8` 冻结，独立预期为 `e190a26`。集成主场景的事件名和封存文件以 M4 后端候选为准。这里记录现有组件可读取的字段和 M4 回放需要从真实日志联接的最小输入。组件 11 的批次证据与组件 12 的派生证据属于各自运行。[冻结回放合同](contract-v1-zh.md)

## 已有组件的实际字段

| 输入 | 现有可检查位置与字段 |
| --- | --- |
| finalized L1 与收据 | 组件 11 和 12 的 `raw-rpc-bundle.json` 有 `blocks`、`receipts`、`config`、`historicalCursorReads`。使用包含块的 hash、parentHash、timestamp、区块号、交易索引和交易哈希，保留 receipt status、event topic、地址及 RPC 块内 `logIndex`。同交易的规范位置另外保留交易内日志序号。 |
| Inbox batch | 组件 11 的 `accepted-records.json` 有 `config_ref`、`position`、`rpc_log_index`、`request_id`、`submission`、`inclusion_timestamp_seconds`、`finalized_through`、`reader_visible_through`、`evidence`。对照实际成功的 CursorAdvanced、prev/next cursor 和来源 gpos。 |
| DA | 组件 11 的 `da-index.json.payloads` 从实际 commitment 索引不可变 frame 文件。已验组件 profile 为 P0B1/BS-FRAME-v1，完整 frame 用 Keccak commitment。核对页上每个 DA 引用与实际 Inbox 接纳 commitment 对应，展示验证、可读和 WAIT 时点。 |
| G′ | 组件 11 的 `accepted-view-after-override.json` 有 `cursor_read`、`genesis_height`、`virtual_prefix`、`historical_prefix`、`missing_sources`。来源按认证 gpos 的 G 或精确 Cut 绑定。合约 cursor 和已物化 head 各有值。 |
| D | 组件 12 的 `d-a.json`、`d-b.json` 有 `action_trace`、`canonical_checkpoint`、`checkpoints`、`cuts`、`derived`、`head`、`native`、`snapshots`。`derived[]` 中分别有 `cause`、`native_block`、`native_hash`、`physical_block`、`state`、`virtual_block`。M4 需要把实际 DA 采用或原生窗口回退与这些输出联接。 |
| 历史 Cut | 组件 12 的 `cuts[]` 保存 `resume_position`、`cursor_set_position`、`target_height`、`genesis`、`derived_prefix`、`target_anchor`、`evidence`。同块两个交易的控制位置分别为 `(45,0,1)` 和 `(45,1,1)`，第二条控制的 RPC `logIndex` 为 3。Cut 记录是在对应 CursorSet 处理切面取得，目标读取来自 Cut 内的指定高度。 |

组件证据目录：

- 功能 11 批次、收据及 DA，`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run/`。[批次/DA 交接](../../docs/handoffs/11-batch-services-zh.md)
- 功能 12 历史 Cut 和 D，`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/12/artifacts/derivation-tester-d0afeee-run1/ahead-300/`。[D 派生交接](../../docs/handoffs/12-derivation-zh.md)

## 集成主场景的最小联接

| 关联 | 需要保留的实际值 |
| --- | --- |
| 时间和保管 | 使用唯一可排序的 trace seq、虚拟 `tick_us`、原始文件及记录索引。L1 header timestamp、实际收录/模拟 finalized/各 reader 可读、Espresso `θ` 与 `q`、GFR ReplayPosition 分列。每条输入记录在 manifest 有文件大小与摘要，独立 PASS 绑定同一 manifest、候选和所用投影。 |
| batch 到 D/G′ | 交易和成功日志位置、认证 gpos、closed frame 及 commitment、DA 持久/读取/校验、D 的实际 native parent 和 origin、窗口上界及下一 origin 的来源。D 输出用 adopted batch、window fallback 或 cursor fill 表示。G′ 的物化记录保留其独立 cursor、已物化 head 和来源。 |
| 同高分歧 | 可比较的 G 与 D 块库、各自处理位置和同一高度的 native hash，首个冲突及共同前缀终点。每块保留 native parent/hash、连接身份、实际 origin 和执行前/后 root。 |
| RESUME 到 Cut | 同交易的成功 ResumeRecorded/CursorSet，各自规范位置与 RPC 索引、交易哈希、旧/新 cursor、请求序号、目标 `h_res`、精确 Cut 键及摘要、D 的历史前缀与采用范围、target anchor 的连接 ID 与执行后 root。Cut 摘要使用 `PoC/Derivation/ResumeCutCheckpoint/v1` 与 Rust `serde_json::to_vec(&cut)` 的精确字节。保留此原始对象或可按字段顺序重建字节的证据引用。 |
| GFR 活跃和 Manager | WAIT 所缺输入及之前的 pc，覆盖激活所用 finalized L1 和已完整 Espresso 水位，同位置替换前后 G、撤销/采用区间与 DER 来源，关闭 branch 及所受影响的 cid，后继 Plan/Ref/window、实际 Ready/Enter 的节点和时点。后端合同给出 `BranchClosed → Resumed → SessionAssigned`，对齐后 HALT 时最后一项为 `SessionClosed(EXPIRED)`。按候选真实记录核对。 |
| 恢复首块 | 区域交付与其 body/proof、Espresso 公开完整位置 `q_avail`、GFR 接纳位置和 tick。首块 physical parent 为历史派生 anchor 的连接 ID，执行前 root 与 `D(h_res)` 的执行后 root 核对。 |

最小待锁定项是主运行 manifest/checker 的文件与摘要约定，以及上述关联对应的 M4 trace 事件名。G′ 的实际物化、切面的内容身份和原子后缀替换记录需与 D/GFR 分别输出。反馈原始事件路径与字段即可完成后续映射。

## 后端开发运行已见字段

以下字段已经用开发运行核对过，用于正式映射的预检。正式页要求同一候选的封存输入和独立 PASS。

| 原始位置 | 映射到回放 |
| --- | --- |
| `raw/trace.jsonl` 的 BatchSubmitted、L1Included、L1Finalized、L1ReaderAvailable | 批次准备与提交证据、真实 Anvil 收录和 A/B reader 时间。用 `raw/l1/raw-rpc-bundle.json` 核对交易、receipt 和 CursorAdvanced。 |
| `raw/derivation/batch-h*.json`、`accepted-A-*.json`、`da/0x*.bin` 与 `*.receipt` | 批次来源 snapshot 的 `position` 和编码 `gpos`，signed/DA 证据、实际 commitment、accepted payload。 |
| `raw/derivation/trace.jsonl` 的 `reader,tick_us,l1_height,step,new_items`，Hub 的 DerivationAvailable | D 的 batch 采用、晚到处理、BlockEnd 处窗口回退及 CursorSet 填充。通过 reader、L1 height、tick 联接到 Hub seq，保留原始 D 行号。 |
| ResumeSubmitted、真实双日志、`raw/derivation/cut-A-*.json` 和 `cut-B-*.json` | 确切交易、两个 L1 位置、精确 serde 字节以及 A/B 对同一历史切面的核对。 |
| GReset、GDerived、GfrReplayRecord 的 `resume`、ResumeActivated | 用控制位置、Cut 摘要及提交记录联接为一个原子帧。该帧保留共同 native 前缀、旧 G revocation、新 DER 采用和生命周期批次。 |
| RegionalDelivered、PublisherRecorded、GAccepted、`raw/results/*.bin`、`raw/espresso/*.bin` | 实际区域交付、规范物理父连接、正文及最终证明消息索引、`q_avail` 和最后接纳。 |

G′ 当前从认证批次与精确切面重建。开发日志可给出相关 reader 观察的 tick/seq，单独的实际物化时刻需要 G′ 事件。DA 校验采用实际持久对象及接纳证明，单独的获取完成时刻需要 DA 事件。GFR WAIT 的实测路径需要候选输出显式 WAIT 原因、阻塞公共位置与持续时点。
