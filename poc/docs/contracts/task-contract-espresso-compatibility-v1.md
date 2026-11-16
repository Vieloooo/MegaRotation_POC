# ESPRESSO-COMPATIBILITY v1 — 官方本地来源与录制回放合同

日期：2026-11-16。版本 `espresso-compatibility-v1`。状态：**16 lead 冻结语义 v1，独立 Verifier 已先从来源推导预期；独立向量已在89925a6提交，类型/来源接口已在9747268提交，随后进入源码阶段。合同不是运行通过证据。** 06 固定依赖 `4b034dfbb6c3e1c7143208347b546daab8dfe298`；本 worktree 分支 `codex/poc-espresso-compatibility`。

## 1. 权威与所有权

本合同的来源规则（S）来自 [v2 §§4.5、13.1、14](../../../formalization_v10/poc_design_zh_v2.md)、[v1 §§5.1–5.2](../../../formalization_v10/poc_design_zh.md)、[v10 架构 E-S/E-TS](../../../formalization_v10/00_arch_zh_v0_1.md)、[16 handoff](../handoffs/16-espresso-compatibility-zh.md)、[06 冻结合同 §§2–3](task-contract-publisher-espresso-v1.md)和下列固定官方代码。v1 §5.1 明确允许官方 dev-node 的可信本地 API 边界，未验 finality proof 时不得声称独立验证 Espresso 共识安全。v2 的确定性主测试与官方墙钟兼容模式分开。

适配选择（A）：只支持下列 release/protocol/API profile；以可信本地 API 的完整 block 元数据和逐位置交易查询收集完整列表；保存原始记录，确定性离线投影。此选择不改 06 的公共协议，不替代 HotShot 或 Regional BFT 验签。Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`、Commonware `2026.10.0` 保持原样；本功能不改其源码或模型。

| 路径 | 唯一 writer |
| --- | --- |
| 本合同 | Docer；冻结后由 16 lead 管理 |
| `poc/crates/espresso-compatibility/`、`poc/scenarios/espresso-compatibility/` | 单 Builder；独立 workspace/lockfile |
| `poc/verification/espresso-compatibility/` | Verifier；Tester 只写明确交接的 runner/report 路径 |
| 本功能 preflight/交付报告 | 16 lead |
| 其他功能、根 Cargo/types/core、native、Regional BFT、`poc/docs/commander/` | 只读 |

## 2. 固定官方来源、支持范围与运行边界

- 官方仓库 [EspressoSystems/espresso-network](https://github.com/EspressoSystems/espresso-network/tree/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171)，release tag `20260918`，peeled commit **`fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171`**。不运行 moving `main`。
- 官方镜像 `ghcr.io/espressosystems/espresso-network/espresso-dev-node:20260918`；index digest **`sha256:af73f55630690caa35aa0bc015d60dc03333791f966fdcc07b0798e33d4f768e`**；linux/amd64 manifest digest **`sha256:cfb845e49cb5369f77cd8bbda0b4fe4aad340793cfd130b7f2bb631e59d2db66`**。运行证据保存 manifest/config/layers 的实际 hash 和 image revision，执行文件 hash 单列。
- 显式 **`--version 0.3`**，HTTP **`/v1`**，publication namespace **`160016`**。这三个都是 A，不能从 moving 文档推断。固定源码 `crates/espresso/dev-node/src/main.rs:232` 默认 0.3，`src/lib.rs` 支持 0.3/0.4/0.5/0.6；main 文档的 0.6/0.7 不是本 profile。
- 官方 Dockerfile 直接执行 `/bin/espresso-dev-node`，另装 Anvil，没有必须依赖 Docker daemon 的入口脚本。允许两种执行方式：固定镜像容器；或从同一固定官方镜像核验并提取**未修改** binary、运行库、SRS，使用任务目录内匹配 loader/runtime 运行原 executable。后者明确标为“official-image binary native execution”，不是容器运行。不可替换二进制、自己伪造 official server 或更改系统 loader/daemon/代理。
- 必须显式连接本任务专属本地 Anvil，记录 L1 chain-id、URL、binary hash、配置；不得默默另启不受记录的 L1，不调用公共生产/测试网络。固定 upstream `crates/espresso/node/src/lib.rs:1514–1516` 的 `TestConfigBuilder::default()` 还会在 `main.rs:332–338` 将 l1_url改为显式L1之前启动一个bootstrap Anvil。该原binary内部行为不修改；须通过任务私有PATH wrapper调用同一已验证镜像的Anvil，记录每次原始argv、namespace PID/PPID、端口参数及退出清理。wrapper只记录后exec固定binary/匹配loader，不伪造Anvil输出。显式L1为18775，bootstrap辅助链是upstream初始化资源，二者明确区分；均受同一私有network/PID namespace约束。禁止继承宿主foundry可执行文件来隐式满足依赖。允许 upstream dev-node 自身必要的本地部署操作；不把它们当 PoC staking 实现。
- 端口建议：18770 API、18771 dev-info、18772 builder、18773 prover、18774 tonic、18775 L1。以实际可用端口为准且冻结到 run manifest。upstream 若监听 `0.0.0.0`，可以用本任务独立 user/network namespace 隔离到其 loopback；禁止改全局网络/daemon 配置。所有客户端、节点、Anvil 在同一任务隔离环境内。只能终止本次保存的 PID/容器。
- Rust 编译和重型运行经 `flock /tmp/megaroation-poc-heavy.lock`，默认 `CARGO_BUILD_JOBS=2`，不嵌套锁。`.build` 指向 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16`；每次新 artifact 目录，旧结果只读。

### 2.1 固定源代码证据

所有路径均相对于上述 pinned commit。原始源码可由官方 tarball 重取；本次调查副本 `/tmp/espresso16-source-fd221e8e4b/` 只作工作副本，正式证据需保存来源与 hash。

| 固定路径 / 精确职责 | SHA-256 |
| --- | --- |
| [`doc/espresso-dev-node.md`](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/doc/espresso-dev-node.md) / 支持的本地开发节点 | `cedb782b44dc03420160827ba4385de86d5a82e1a8a05633ded0c0c9e05ad2ee` |
| [`docker/espresso-dev-node.Dockerfile`](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/docker/espresso-dev-node.Dockerfile) / binary、Anvil、env、CMD | `a9141d711a9c15e24451d4d7b4390cc9a1cc2a95e59ead662da57b4d07901981` |
| `Cargo.lock` / 官方依赖锁 | `f78533a0e33f11e6180340bdd9a05865786d59e029e71cb228690bb6474c0ec5` |
| [`crates/espresso/api/src/axum/routes.rs`](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/crates/espresso/api/src/axum/routes.rs) / `/v1` 路径 | `b7ed916d4e2da2b6532d2e9e8f884ee0390fdd2d7b5f5f6caf0190a9c2f16e53` |
| [`crates/espresso/node/src/api/state.rs`](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/crates/espresso/node/src/api/state.rs) / `get_transaction_by_position`，flat index 通过 payload.nth | `b9da55c7a37c47aa3a71787818d5908404afd62e00a77fb8b31e63e30a7d1226` |
| [`hotshot-query-service-types/src/availability.rs`](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/hotshot-query-service-types/src/availability.rs) / LeafQueryData、BlockQueryData、TransactionQueryData | `d4658de4fd16a95ea2f1d754745180a93ed45e04416a7bfc8dfa5636dbd1c6f7` |
| [`crates/espresso/types/src/v0/impls/header.rs`](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/crates/espresso/types/src/v0/impls/header.rs) / versioned JSON、height、seconds timestamp、nondecrease | `ea9018d5b7b3badaeceb678dfacdb8a96bbb4482c32b7f49b4ed8c42ac0f5001` |

公开文档补充：[submit](https://docs.espressosys.com/network/api-reference/sequencer-api/submit-api)、[availability](https://docs.espressosys.com/network/developer/espresso-api/availability-api)、[node](https://docs.espressosys.com/network/api-reference/espresso-api/node-api)。它们用于定位，不覆盖上述固定源码；查询 hash 回应只是提交确认，`/node/block-height` 是 latest q+1 的提示，可能有早期缺数据。

## 3. 官方 API 与完整列表读取

| 操作 | 本 profile 的请求 | 语义及必留记录 |
| --- | --- | --- |
| 提交 | `POST /v1/submit/submit`，JSON `{ "namespace":160016, "payload":"<base64 exact bytes>" }` | 保存原请求、状态码、原响应 transaction hash；ACK 仅 `Submitted`，不能作为 finalized 或 publication 完整记录。 |
| tip 提示 | `GET /v1/node/block-height` | H 是 latest known block height+1；H 不是连续完整前缀或 finalized 证明。不能从 H 跳过低 q。 |
| 完整 block | `GET /v1/availability/block/{q}` | `BlockQueryData {header,payload,hash,size,num_transactions}`；保存完整原始 bytes 和解析的对象。 |
| 对应 leaf | `GET /v1/availability/leaf/{q}` | 保存 `leaf`、`qc`，其 `leaf.block_header` 必须与完整 block 的 `header` 在 JSON 语义上相等；QC 只留证据，不声称已验签。 |
| 每个 flat 位置 | `GET /v1/availability/transaction/{q}/{i}/noproof`，i=0..N−1 | `TransactionQueryData` 的 `index` 是 block 内从零开始的 flat index，包含 `transaction,hash,index,block_hash,block_height,namespace,pos_in_namespace`。不把 ns 内 index 当全局 index。 |
| 诊断 hash 查找 | `GET /v1/availability/transaction/hash/{hash}/noproof` | 可辅助定位 publication，但同 hash 多次出现时返回哪个 occurrence 不作保证，不能用它枚举或去重完整列表。 |
| 独立重复读取 | 重新建立 HTTP 连接，再查相同 q/block/leaf/每个 i | 比较解析后的公共对象，不要求 JSON 空白和字段排列相同；保留两次 raw 响应供审计。 |
| 缺失输入 | 404、超时、断连、尚无某个 q/i | `WAIT`，记录 endpoint/状态及未完位置；不能造空 block、零条交易或人工水位。 |

冻结澄清：lookup HTTP408/429/5xx 表示没有可用内容，按有限墙钟预算内的Unavailable/WAIT记录和重试，保留准确status，不把服务错误称为健康证据；提交4xx/5xx是SubmitRejected而非ACK，重定向拒绝且不跟随。manifest引用文件缺失仍是InvalidRecording，不属于日志WAIT。

S：flat index 的官方实现通过 `block.payload().nth(block.metadata(), index)` 得到 namespace 内部 index，再构造 API 交易。因此按 flat i 查询得到的顺序就是该 block 的公开遍历顺序；不得自行按 namespace、transaction hash、提交时间排序。

A：本 profile **信任本地官方 API 的 `num_transactions=N` 与逐位置响应共同给出完整列表**，并核对所有 q/hash/index/namespace 绑定。它不在 16 内重新实现 Espresso payload/namespace table 的全部恶意编码规则，不重算 VID、QC 或 block commitment。原 raw payload、namespace table、leaf/QC 均保存。来源 total decoder 对特殊 payload 的容错不等于本适配器可以声称它们共识无效；未支持表示返回 `UnsupportedRepresentation`。这个信任边界必须在导出数据和报告中保留，不能由裸 bool 替代。

只收到 namespace subset、hash 查找、header、leaf 或 N−1 个交易不形成完整 block。namespace 筛选只能在完成全 namespace 的 N 个交易后做；原始 `source_transaction_index=i` 保留；筛选后的 publication 列表另用连续 `message_index=0..K−1`，二者有明确映射。不同 namespace 的相同 bytes、同 namespace 重复 transaction，全部保留。

## 4. 最小可消费接口 EC16-NATIVE-1

此接口是 16 的**来源观察包**，不是另一套 03 对象编码或 06 PELG。Rust 名称可 snake_case；Builder 首个接口提交附精确导出签名。依赖既有 `sim-espresso` 的 `EspressoPosition`、`PublicTimeUs`、`LogMessage` 及界限，不复制其定义。

```text
SourceIdentity {
  schema="EC16-SOURCE-1", official_commit, image_amd64_digest,
  protocol_version="0.3", api_version="v1", namespace=160016,
  run_id, genesis_block_hash
}
SourceBinding { identity, source_id, capture_manifest_sha256 }
FinalityBasis = TrustedLocalOfficialApi { binding, leaf_record_ref, block_record_ref }
UnknownTiming { proposed_at:Unknown, finalized_at:Unknown, live_at_proposal:Unknown }
NativePosition { q:u64, source_transaction_index:u32 }
ProjectedMessage { position:EspressoPosition, source_position:NativePosition, message:LogMessage }
NativeTransaction {
  position: NativePosition, namespace:u64, payload:Vec<u8>,
  transaction_hash:String, block_hash:String, pos_in_namespace:u32
}
NativeBlock {
  q:u64, theta:PublicTimeUs, block_hash:String, ordered_transactions,
  complete_count:u32, finality_basis, timing:UnknownTiming, evidence_refs
}
NativeRead = Complete(NativeBlock) | Wait{next_q,missing} | InvalidSource(reason)
           | UnsupportedRepresentation(reason) | Capacity(reason)
OfficialCaptureReader::receive(capture_record) -> [NativeBlock / errors]
OfficialCaptureReader::tip() -> Option<(q,theta)>
OfficialCaptureReader::read(q) -> NativeRead
OfficialCaptureReader::require_prefix_through(q) -> NativePrefix | Wait / error
OfficialCaptureReader::require_watermark(theta) -> NativePrefix | Wait / error
project_publication_messages(q,configured_codec) -> Vec<ProjectedMessage> | error
// 全块已完整后筛选 namespace160016；原生位置不变，message_index 在筛选结果内连续。
export_pe_log() / export_pe_reader() -> UnsupportedTimingEvidence{missing fields}
```

`NativePrefix` 名称/文档必须保留 `TrustedLocalOfficialApi`；不能名为独立 HotShot verified prefix。实现可只提供上述等价行为，不要求与 PE trait 同名。JSON recording/result schema 是本任务 provenance，不作为区域结果的 canonical bytes。

### 4.1 身份及录制包绑定

- 一次本地 dev-node 启动一个 `run_id`（ASCII `[A-Za-z0-9_-]`，1..64 bytes）；重建 genesis 需新 run_id。`genesis_block_hash` 必须来自此 run q0 的完整 block，不从配置猜测。相同 release/端口不能合并不同 run。
- A 的 source-id recipe：依次把 `EC16-SOURCE-1`、official commit、image amd64 digest、protocol version、API version、namespace 的十进制、run_id、genesis block hash 作为 UTF-8 行，每项后接 LF；所有字段不含 CR/LF。`source_id = ASCII("EC16:") || lowercase_hex(SHA256(these bytes))`，69 bytes。这是部署来源隔离，不是官方 chain hash 或 03 object codec。
- Manifest 含 source identity、官方 artifact hash、完整命令/配置、端口、L1、fixture/official 分类、每条 raw response/request 路径/hash、HTTP method/path/status。记录排序 id 单调、每条明确属于哪个 reader/会话与源。路径禁止绝对路径/`..`/越出包目录；缺文件、hash 不符或绑定冲突拒绝。
- Manifest SHA 与其 raw 文件哈希只防意外损坏并提供审计定位；不能抵抗能重写整包/manifest 的攻击者。可信来源是本任务本地 launch/capture provenance，不是 manifest 自签名。
- q0 后同一 source q/header/payload/list 不可改。若录制不从 q0 开始，它只能作为缺前缀的片段，不能宣称完整 source prefix。

### 4.2 时间

`q = official header.fields.height`（protocol0.3）；`theta_us = checked(header.fields.timestamp * 1_000_000)`。保留原 seconds 值，若来源额外含 millis 则保留原字段但不用作本 profile 公共键。θ允许相等，回退拒绝；溢出拒绝，禁止 saturating/clamping。跨 block 总序只按 q。

离线公共摘要只覆盖 source_id、q、theta_us、block_hash、完整有序交易及投影内容；排除 evidence_refs、capture_manifest_sha256、reader身份/会话、HTTP记录序号及所有观测时间。摘要字节按下列 EC16-AUDIT-1 固定，由 Verifier 独立生成 literal expected，不能拿实现输出当 expected。

每条 HTTP request/send/receive 的本地 wall/monotonic 时间是**观测元数据**。它们不参与 source-id、公共对象内容或离线 projection 摘要；不能当提议时刻、共识 finalization 时刻、Clock RPC virtual tick 或 live 依据。

`proposed_at`、`finalized_at`、`live_at_proposal` 固定为显式 **Unknown**。不能将 Unknown 表成 0、false、θ、reader receipt time 或估算时间。因而本 profile 不评估 E-TS、T_E、PUB 时限或 1 ms 时钟保证。wall run 与 deterministic replay 各自报告；同录制两次重放一致不意味着 dev-node 两次执行产生相同区块。

### 4.3 到齐、前缀及错误原子性

块 q 在以下材料全部存在且有效之前不 `Complete`：对应 block+leaf；header version 精确0.3、height=q；block.hash 非空 tagged BLOCK 身份；leaf.block_header 与 block.header JSON 语义相等；qc 为被记录的官方响应结构（未验签）；N≤MAX_MESSAGES；恰好 0..N−1 的逐位置 tx。

每个 tx 校验 `index=i`、`block_height=q`、`block_hash=block.hash`；`transaction.namespace` 与外层 `namespace` 相同，namespace 在 u32 范围内；payload 是可严格解码的 base64 字节。保留 hash 字符串及 `pos_in_namespace`，不推导签名有效。已知多出 index、冲突 tx 或错误绑定是 `InvalidSource`/包不一致；只是缺少需要的响应为 `Wait`。N=0 也必须有真实完整 block+leaf，不能由404产生。

`tip` 只从 q0 连续到齐且 θ非递减时推进。先收到 q2 时缓存；q0 后 tip0；q1 未完时不能借 q2、水位提示或 wall clock 越过。不同 reader 拥有各自独立的已交付记录，禁止从其他 reader 或 recorder 的未来文件补数据。`require_watermark(x)` 只以已完整连续前缀的 θ≥x 判断，不给后续同θ列表做完整性保证。native 水位仍带可信 API 标签，不能被误当 PE verified prefix。

重复完整公共内容幂等；只改变响应 JSON 空白/字段顺序不冲突。同 q 任何被消费公共字段或原始 block/leaf/tx 的解析后对象冲突都拒绝。检验必须先于安装；拒绝保留现有 tip 和数据，不覆盖历史；后续合法缺失项可继续补齐。未发生语义差异的 receipt metadata 不造成 public conflict。重复请求的首次观测记录保留。

### 4.4 容量与 PE 映射

复用 06 已冻上限：source_id≤256 bytes、每条投影消息≤65,536 bytes、单块 N≤1,024、投影 block envelope≤1,048,576 bytes；边界包含等号。有限 reader 最多4,096个不同q（含pending），所有namespace的decoded交易payload累计≤64MiB，重复交付不重复计数；每条原生交易也受65,536-byte上限，不能靠过滤其他namespace逃逸容量检查。投影列表用06的validate_message_list核对envelope上限。原始单 HTTP request/response与manifest均≤16MiB；单 recording bundle的manifest加去重后各原始文件大小总和≤256MiB，读取/解码前检查；超限具名 `Capacity`，不截断、不跳过、不删除历史。真实来源出现超上限不表示 Espresso 共识无效，只说明此 PoC profile 无法消费。

| 06 项 | EC16 映射结果 |
| --- | --- |
| PE-LOG-1 的 `source_id` | 上述与 run/genesis 绑定的69字节 id。 |
| q / index | q 始终 absolute block height；NativePosition 保留原 flat source_transaction_index，筛选后的 EspressoPosition.message_index 从0连续编号并附原生位置。 |
| `PublicTimeUs` | 原 seconds timestamp checked×1e6，允许相等。 |
| `LogMessage` | 仅对已完整列表中 namespace160016 的各项，按原顺序取原 payload bytes 配上调用方明确的非零 codec。其他 namespace 不进入投影，仍保留在 NativeBlock；即使完整原生块 N>0，若无目标 namespace，投影也合法为空。兼容 publication 是明确的 non-cryptographic fixture，不能冒充03/04真实结果。 |
| `ProposedBlock` / `FinalizedBlock` | **UnsupportedTimingEvidence**：缺少真实 proposal tick、live、finalization tick，禁止伪造构造。 |
| PE-LOG-WIRE-1 `encode_finalized` | **UnsupportedTimingEvidence**，不产出 PELG。记录包不是 PELG。 |
| PE-READER-1 `ReaderState` / `VerifiedPrefix` | **UnsupportedTimingEvidence**，不得注入伪造 `FinalizedBlock`。native reader 是已说明可信边界的局部接口，禁止暗中实现PE trait后填假时间。 |
| TimestampAssessment | Unknown/Unavailable；不能调用06 assess后声称NotAssumedNonLive就是未知。 |

这份矩阵是兼容测试的明确结果：可以交付来源身份、public position、完整列表、opaque payload和确定性 replay，但当前06接口不能完整表达官方缺失时间证据。完整 PE-READER 接线保留局部 gate；需要06 owner将时间证据能力分离后另冻精确变更，16不擅改共享对象。这个缺口不阻塞官方真实 publication/读取/重放实验，也不伪称整个06已完成。

### 4.5 EC16-AUDIT-1 摘要字节（仅审计，不是03 canonical codec）

`u32/u64` 均无符号 LE；`LP(x)=u32(len(x)) || x`，文本为UTF-8。fixture publication codec固定`0xEC160001`，不冒称03编码。接口允许显式其他非零codec，摘要覆盖实际值。

```text
ASCII("EC16-PUBLIC-1") || 0x00
LP(source_id)
u32(complete_prefix_block_count)
for each complete block by ascending q:
  u64(q) || u64(theta_us) || LP(block_hash)
  u32(native_transaction_count)
  for each native transaction by source_transaction_index:
    u32(source_transaction_index) || u64(namespace) || u32(pos_in_namespace)
    LP(transaction_hash) || LP(payload_bytes)
  u32(projected_message_count)
  for each projected message by compact message_index:
    u32(message_index) || u32(source_transaction_index) || u32(codec)
    LP(payload_bytes)
```

对以上精确bytes求SHA-256，输出lowercase hex。block_count为当前连续完整前缀长度；pending不进入摘要。必须提供上述bytes或可独立复核的等价projection输出。source_id绑定namespace/run/genesis；重复项保留。所有长度转换checked；不含wall、reader identity、manifest或evidence引用。界限/编码属于A，正式Node的opaque payload绝不重写。

## 5. 验收矩阵（独立预期先于 Builder）

Verifier 已从 pinned source 和06合同推导下列 V01–V14；literal vectors、测试程序与报告在其独占路径，源码实现不能生成 expected。fixture 与官方运行分别标注。

| ID | 触发 / 边界 | 必须结果 |
| --- | --- | --- |
| V01 | pin、run、genesis、namespace/source binding；错误 image/commit/run | 正确绑定稳定；混源拒绝，端口相同不等于同源。 |
| V02 | 提交ACK，未完整block；再补齐含publication的完整block | ACK仍Submitted；只在完整来源列表里确认q/i和exact payload。 |
| V03 | q0..H−1；H提前/低q缺失/只从中间读取 | q不重编号，H仅提示，缺前缀WAIT。 |
| V04 | N=0/1/多条；相同bytes、跨namespace、同namespace重复 | 全flat位置保持原序/重复；缺项WAIT、多项/错index拒绝。 |
| V05 | gap/404/超时/leaf-only/header-only/缺一个tx | 无新prefix/watermark；补齐后连续推进；A不泄漏到B。 |
| V06 | θ相等、回退、秒→微秒溢出、wall不同 | 相等合法；回退/溢出拒绝不消耗q；wall不改公共对象。 |
| V07 | 重复查询、JSON重排、同q公共内容冲突 | 语义重复幂等；冲突原子拒绝，已有tip不变。 |
| V08 | 完整全namespace后筛选；失败查询/空subset | 筛选保留原i并另编连续message_index；不把subset当全列表；原生空列表须实际N=0，目标namespace投影可在完整N>0块上合法为空。 |
| V09 | 同录制离线重放两次，wall pause不同 | 公共投影和规范摘要相同，原始网络时间单列。seed=N/A。 |
| V10 | truncated/malformed JSON/base64，错q/hash/namespace，缺file/错hash、越界path、未知schema | 明确拒绝或按缺日志WAIT；无半前缀，不能默认空库。 |
| V11 | Unknown timing +请求PELOG/PEReader | 明确Unsupported且缺项准确；不产出假PE blocks/ticks/false live。 |
| V12 | 本地端点、显式L1及upstream bootstrap Anvil、独立端口/PID/隔离环境与cleanup | 所有Anvil均为已验证镜像binary；bootstrap wrapper记录argv/PID/PPID/端口并检查清理；无公共network请求、无未知进程终止。 |
| V13 | 官方binary本机publication、原submit/blocks/leaf/tx、第二连接重复查询、缺数据probe | 原始真实transcript与source pin可核验，按实际API行为报告。 |
| V14 | fixture矩阵和V13真实证据/环境失败 | 两者分开；fixture PASS不能充官方PASS；无实际运行则真实gate INCONCLUSIVE。 |

再测每个容量的等号/超1边界、错误输入后继续接受合法记录。V13 至少两个可辨识 publication（不同固定bytes），并重发一个 exact publication；不能要求真实节点一定把重复提交纳入多次，记录其实际行为，离线fixture仍覆盖公共日志中的重复条目。真实采集只需有限连续前缀，包含q0、两次publication的所在块和其间空/非空块；准确高度/窗口留证，不偷换为长期性能结论。

## 6. 交付、状态与未决 gate

首个接口提交包含本合同、独立预期/向量、最小可编译导出和锁文件；不必等真实节点运行完才给下游可审查语义。后续源码候选先提交，Verifier与Tester检查同一SHA；修复后复验受影响项。报告 baseline/interface/candidate/test SHA、实际模型可见状态、worktree/branch、命令、seed/profile、绝对证据路径及hash、PASS/FAIL/INCONCLUSIVE。

- **G-API**：官方 pinned binary 真实本地提交/读取/重复查询通过，完整原始证据可重放；未运行不可代替。
- **G-REPLAY**：独立源规则向量、同录制两次投影与摘要一致，缺数据/冲突/容量/timing Unknown均按合同处理。
- **G-PE-TIMING**：当前明确不支持完整PE-LOG/PE-READER，等待06 owner认可的精确接口修订；16保持缺项，不伪造06时序保证。
- **G-CRYPTO**：此兼容profile采用可信本地API，不声称已验HotShot共识证明/VID或真实Regional FINAL。这是声明的范围限制，不是布尔测试桩通过。
- **G-ENV**：若缺Docker权限，先调查官方native方式并记录实际失败；不得更改系统daemon/代理。若仍无法运行，已授权离线实现/验证继续，真实G-API保持INCONCLUSIVE。

本阶段无待用户决定的协议选择。lead需冻结本合同版本、路径和准确导出；对来源表示/运行环境的新发现只修订受影响条款，不自行升级pin或更改其他功能。全部原16工作及独立验收完成后才申请功能accepted；准确报告G-PE-TIMING兼容缺口，是否可接受此受限profile由总协调决定。

2026-11-16 运行条件澄清：候选8750535的正式run01在genesis前以SpawnError(NotFound)退出，原始证据保留。此修订只冻结upstream bootstrap辅助进程的可审计启动，不改变EC16公共对象/摘要/读取规则。修复须新候选SHA并重新复核实际运行路径；旧离线PASS不能填补真实运行失败。
