# H3 发布与 GFR 接线：准备交付 v1

2026-11-16，Asia/Shanghai。**准备阶段 PASS；H3 实现和健康 15-worker M1 均未执行。** 已完成来源核对、最小接口提案、独立预期和准备语义冻结。没有修改生产代码、创建 Builder/Tester、运行重型矩阵或自行合并其他功能。

## 基线、范围与角色

- Worktree：`/home/jy/.codex/worktrees/1e2f/megaroation_poc`；分支：`codex/poc-viewer-gfr`；干净起点及保留的旧 07 交付：`788291735ce6db3c3eb78b4594a28d730cce71cd`。
- 本次准确计划：`4a9a6b0a92a11e5778833988bb6524d929f4608e` 的 `poc/docs/milestones/README.md`、`H3-publication-gfr-zh.md`。计划及 H1/H2 交接仅用 `git show`/只读源查阅；尚未获交付准确 M0 SHA。
- 已有 03/06/07 保持；02 `e1a435712ec492cae2bbff120a51ffd507eebf45` 和 16 `5c34b400ba5245d7775a281566e21bf59708b147` 仅按准确提交读取，未合并。Regional BFT pin 仍为 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，Commonware 固定 2026.10.0。
- 实际启动 `bft_spec` 和 `bft_verifier`，两者均为命名角色 GPT-6 Astra/xhigh。主配置 `.codex/config.toml` 为 Astra/ultra；不将配置冒称独立运行时证明。未创建新用户 chat 或第二 worktree。
- `.build` 仍指向任务独立数据盘 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07`；旧证据保持原样。本次仅轻量文档/JSON/来源摘要检查；后续重任务仍须宿主 flock。

新健康 M1 是 15 个真实健康 worker、3×5/q4、R0→R1→R2、真实结果经过 Publisher/SimEspresso/GFR 并驱动 Manager，直到 R2 首个结果进入 G；另测 SessionEnd 且窗口不移动。旧 07 文档中名为 M1/M2 的接口/纯规则阶段不是这个健康里程碑。后续主动 Force 属于健康 M2；本次不扩 D/RESUME、恢复矩阵或完整系统。

## 可消费文档

1. [最小接口提案](h3-publication-gfr-interface-proposal-v1.md)：供 M0/H1/H2 协调接口与路径归属。
2. [准备合同 H3-PUB-GFR-HEALTHY-PREP-1](contracts/h3-publication-gfr-healthy-preparation-v1.md)：H3 语义要求、原子边界、容量和后续验收。仅准备语义冻结，不批准未定 wire、上游 API 或跨 owner 改动。
3. [独立来源预期](../verification/publication-gfr-healthy/source-first-expectations-v1.md)及[冻结记录](../verification/publication-gfr-healthy/expectations-freeze-v1.json)：28 个初始用例；提案讨论后的 4 项补充保存在[独立增补](../verification/publication-gfr-healthy/additive-expectations-v1.md)，保留先后顺序。
4. [15 组字面向量](../verification/publication-gfr-healthy/literal-vectors-v1.json)、[未来重放配方](../verification/publication-gfr-healthy/replay-recipes-v1.md)及[固定来源](../verification/publication-gfr-healthy/source-pins-v1.json)。结构对象和 signer 集合配方明确不是实际 native 证书。

[独立准备复核](../verification/publication-gfr-healthy/interface-review-v1.md)和[封存结果清单](../verification/publication-gfr-healthy/preparation-results-v1.json)在同一验证目录内交付。共 **52 项轻量来源/格式/字面自检 PASS，15 组向量，0 次候选执行**。所有 **32 项接线/集成用例 NOT_RUN**；六类实施就绪 gate 为 INCONCLUSIVE。Lead 另核对全部 36 份固定来源摘要、初始预期冻结摘要及所选规范字节/容量算术。没有编译、运行 native 验签或启动任何服务。

可复查的轻量命令为 `python3 poc/verification/publication-gfr-healthy/check-preparation.py`，seed 不适用；它只核对 JSON、规范字节/算术及来源摘要，不导入候选实现。冻结合同 SHA-256 为 `381f408f63e805040d0f3dd01a72150ca1ee2ef3fb023d6ab4f2cd781c4c52c4`，最小提案为 `892376107f8f08c0534f5a77cdbcbc2129d98627fe96bf6052f0b554c64b0ce4`，独立结果清单为 `69374813702bf7245f1c49f833a87453d8f9550f29998a6fa51032672e8362f3`。无 H3 实现候选 SHA 或执行 trace 可交付，不能用旧 07 证据改标替代。

## 已明确的接线决议

- **唯一编码与真实验签。** 03 对象及 GFR-PUBLIC-1/POS-1/LIFE-1 原样复用。H1 提供准确真实 FinalVerifier、历史输入 readiness、完整 proof/body/ancestry/anchor 及受控来源；Publisher 与 GFR 共用它。主场景必须绑定 Native profile，不回退到 bool fixture，也不能改标签继承旧 fixture checkpoint/cache。历史 CheckFINAL 不依赖仍存活的 engine、当前 lease、Selected 或本地 Close。
- **公开完整与选中分开。** 提议在已有 viewer-gfr 中抽取并导出共用的纯公共完整性能力，供 Publisher 与 GFR 复用。未选中的有效候选可以已经完整公开并 RECORDED；这不意味着 G 接纳。不能只查询现有 NEXT 上的 q_avail，也不在 bridge 复制另一套完整性算法。
- **完整 Offer 与分 q 发布分开。** 先验证完整结果并保留单个不可变 attempt；受控服务可将其既有 proof/body 消息发布到不同 q。06 合同已允许这种情况。单 q 的片段晚到仅是本地 WAIT，不能替代公开 proof-first/body-later 测试。保留 receipt 原始 publication position、实际完整前缀与 q_avail 的区别。
- **公开停止范围持久保留。** 06 的 `StopDecision` 只作用于现有 key。H3 需保存已提交的完整公开生命周期身份和范围，在晚到 Offer、prepare 和未提交 dispatch 前检查；本地 TIME/Close 不停止历史合法结果发布。不能撤回已 root-committed 的在途提交。
- **分片只在运输层。** 06 单消息 65,536 bytes、含 framing 的块 1,048,576 bytes；02 payload 是可配置且至多 65,536 bytes。完整 attempt 仍必须符合 06 单块容量检查。不得用分片扩大公共对象上限或新增公共 fragment 编码。02 默认全局队列 4 MiB，15-reader 扇出需要受控 credits/背压，不能只检查单帧大小。
- **完整 lifecycle 批次与本地安全状态分开。** 保留整个 ReplayRecord 的顺序和每个完整 LIFE-1 身份，不按 pc 单独去重。GFR committed cursor、Manager completed-batch cursor 与 H1 terminal guard 不混用；无全员 Ready 屏障。后继材料 WAIT 不能削弱已适用的永久 Close/ClockUnsafe；具体 fence/ack 机制由 H2/H1 冻结，H3 未替它们决定实现。
- **官方同字节发布另列。** 保存主场景实际发出的规范 payload 及来源/摘要，经固定 16 local dev-node 提交并完整读回，核对 namespace/native q/flat index/projected index 和字节。官方坐标不等于 SimEspresso 坐标，Unknown/Unsupported 时序和未验证 QC/VID 保持原样，不进入确定性 GFR 时序证明。

## 总协调需收口的具体输入

| Owner / gate | 实施前需要的交付 |
| --- | --- |
| M0 | 准确、已验证的集成 SHA。H3 不自行整合功能分支或使用浮动 main。 |
| H1 native | 真实 proof/subject/namespace/verifier/basis/anchor 能力及准确接口提交；与 03 版本/域兼容。必要共享修改交总协调分配 owner。 |
| H1/H3 transport | 实际 endpoint/channel/codec/grant/Ack/service custody 接口，具体 payload/global/link/custody 限额，wire 与扇出 credits。提案中的 chunk 大小尚不是已交付 wire。 |
| H2/H1 lifecycle | 全记录批次、材料验证、每 worker applied cursor/ack、幂等和永久关闭机制；覆盖“old close + new material WAIT”负例。 |
| H1/H3 size profile | 测得并冻结完整 proof/body/ancestry、public message/list、私有结果/anchor 材料的最大编码尺寸，符合现有 03/06/07 上限。 |
| 16 runner owner | 当前 `capture.py` 固定测试 MARKERS，不能直接发布真实 M1 artifact。优先 H3 自有 payload driver 复用 16 解析/录制及原启动隔离设施；若需要 wrapper 参数，先移交最小 `capture.py`/`run-local.sh` 及受影响检查路径，不擅改 16。 |

未来 H3 的单 writer 路径为独立 `poc/crates/publication-bridge/`、必要的 `poc/crates/viewer-gfr/` 接线，以及专属验证/文档。06/16 仅消费；没有授予其源码编辑权。本次准备提交交由总协调和独立 merge 工作纳入，不计 H3 实现完成，更不计健康 M1 PASS。
