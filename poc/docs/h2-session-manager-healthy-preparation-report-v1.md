# H2 Session Manager：健康 Regular 准备交付 v1

2026-11-16（Asia/Shanghai）。**仅文档准备；合同 DRAFT / NOT FROZEN；H2 尚未实现，未执行行为验收。** 本轮授权到 Docer 与独立 Verifier 的来源、接口草案和验收预期。M0 的准确集成 SHA、H1 的已冻结接口及后续实现授权尚未作为本轮输入交付。

## 当前 checkout、来源与角色

- 实际工作目录：`/home/jy/.codex/worktrees/a6cf/megaroation_poc`，使用本 chat 已有 managed worktree。
- 起点：`4a9a6b0a92a11e5778833988bb6524d929f4608e`；准备分支：`codex/h2-session-manager-preparation`。
- 源 checkout `/mnt/reth/jy/megaroation_poc` 的 AGENTS/DESIGN 仅只读访问。没有自行合入 03、07 或其他功能分支，也没有更新 main 或 push。
- 当前 submodule 未初始化，gitlink 为 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。精确来源通过 `git -C /mnt/reth/jy/regional_BFT_sim show <pin>:<path>` 只读获取，未改变该仓库的工作区、index 或历史。
- v2 SHA-256：`95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`。Commonware 固定 `2026.10.0` / upstream `92a2f68a0e131c8b69de41e84aec37da6280f212`；无依赖升级。
- 主 chat 请求及仓库配置为 GPT-6 Astra/ultra；当前工具未提供可独立核对的主模型运行元数据。实际启动命名角色 `bft_spec` 与 `bft_verifier`，工具定义均为 GPT-6 Astra/xhigh；没有替代模型，未启动 Builder/Simulator。
- 使用 [regional-bft-conformance](../../.agents/skills/regional-bft-conformance/SKILL.md) 从来源推导预期。Docer 是本轮唯一仓库 writer；Verifier 在 `/tmp/h2-session-manager-healthy-verifier/` 独立写预期，冻结 hash 后再读草案；之后由 lead 串行保存并收口。

## 交付材料

- [H2 合同草案](contracts/h2-session-manager-healthy-preparation-v1.md)：公共接口核对、状态/原子消费建议、待决项和 28 项计划验收。
- [独立预期](../verification/session-manager-healthy/independent-expectations-v1.md)：30 项来源先导出的验收；[11 条手写向量](../verification/session-manager-healthy/hand-vectors-v1.yaml)。均为 NOT RUN。
- [独立来源清单](../verification/session-manager-healthy/source-manifest-v1.json) 与 [lead intake 清单](../verification/session-manager-healthy/intake-source-manifest-v1.json)：精确版本与内容 hash。

独立预期在草案审阅前封存，SHA-256 为 `11f745e81faa46c03d85d2ee1ed722a842f8631ef62e96dd556e8610cece7ceb`；向量为 `9bda2836ba7a2029e53f79c070a49cb83a217d60f21e8e7e893c367bf7337fbf`。原封存文件按字节复制到上述路径，未用草案或实现改写 oracle。

两份材料的数值例使用不同时间原点：独立向量 `ts*(h)=h 秒` 对应 stop/end=11/14 秒；合同沿 NR 草案 `ts*(h)=10+h 秒` 对应 21/24 秒。它们是同一规则的平移示例，不是同一个已冻结运行配置。

## 范围与路径

M1 需要十五个持续运行的真实健康 worker，R0/R1/R2 各五成员、四个不同合法 signer 的 quorum，固定十高度窗口，经真实区域证明、Publisher、SimEspresso 和独立 GFR 完成 R0→R1→R2，直到 R2 首个合格结果进入 G；另有 SessionEnd 路径和同一实际结果字节的官方本地 Espresso 提交/读回。官方测试独立报告 wall time/Unknown 时间证据，不增加确定性主验收的结论。

未来 H2 实现仅拥有 `poc/crates/session-manager/`（独立 workspace）及 `poc/verification/session-manager-healthy/` 和 H2 专属文档。H1 拥有原生实例、Create/Prepare/Enter/Pause/Close、cid 隔离的 engine/sign/timer/storage；H3 拥有 publication/GFR bridge。本轮没有创建 H2 生产 crate，也没有改 H1/H3、根 types/manifest、commander 登记或协议来源。

M2 健康主动 Force 仅预留 branch-close/assignment 入口。OS 重启、D/RESUME、批次/DA、Web 不进入本轮；普通持久化、原签名历史、关闭守卫、真实验证及完整 grant/Ack 仍是必需条件。

## 现有接口与待冻结边界

| 所有者 | 本轮核对的准确来源/路径 | H2 接入要求 |
| --- | --- | --- |
| 03 | `e9c15e04bc83b335668f3ecd837367aa87344a89`，`poc/crates/types/src/domain/objects.rs` 及其公共接口合同 | 复用唯一身份、Plan/Descriptor/Cid、状态/执行、结果与生命周期 typed 对象及 canonical codec；不复制编码。 |
| 07 / H3 | `788291735ce6db3c3eb78b4594a28d730cce71cd`，`poc/crates/viewer-gfr/src/api.rs`、`codec.rs`、`replay.rs` | 复用 `ReplayRecord{position,input_ref,effects,events}`、GFR-POS-1 与 GFR-LIFE-1 的 `lifecycle_projection` / `lifecycle_id`。该版本没有对外 `decode_lifecycle`，不可把 projection 函数误报为完整双向 wire codec。 |
| H1 | 尚未收到准确冻结接口 SHA；当前 NR 文档是准备稿 | 精确 anchor/state 准备、一次 Enter、原实例停车/恢复、绝对 deadline、异步完成守卫、durable action receipt/root-Ack，均待 H1 接口冻结。 |
| M0 | 本轮未收到准确集成交付 SHA | 当前基线仅计划提交；只读可见 03/07 对象不表示 M0 已完成集成。 |

同一 replay position 可有多项 lifecycle，甚至连续多次 regular。不能只按 position 去重。建议以完整已提交 ReplayRecord 为 H2 原子消费单位：生命周期 identity 与 cursor、Selected/Excluded、session/branch 关闭记录、终止标记及待执行动作一起提交；下游 Close 等动作可幂等重试。此批次接口是 H2 adapter 提案，尚待 H1/H3 确认。选择某候选但 ReadyState 未齐可留下 Selected+Preparing；不能因为候选下载尚未完成而把公开关闭无限期搁置。

SessionEnd 特例必须提前准备 h9 anchor 的后继，后继窗口固定 11–20；在原 h11 准入窗口内真实生产或缓存并保留首次接收时间。该 h11 验收使用公开 SessionEnd 之后选回的原 cid/原实例及既有结果，不能改 window、重置 Enter/view/deadline 或伪造 firstSeen；选回后其他合法新高度的生产仍按原窗口继续。reference block 使用前驱初始 anchor，production anchor 使用公开固定的末端，两者不可混淆。

## 待 H1/H3 收口的冻结项

1. H1 提供准确接口 revision、导出路径及命令/完成事件类型，标明 Create、状态准备完成、Enter、Pause、原实例恢复和 Close 的幂等键及返回语义。
2. 给出精确 ReadyState 的生产者、验证依据与接受边界：完整 anchor 执行后状态和有序执行证据；不能只传 state root/地址/外部 ready 布尔值。
3. 明确 Paused 原实例的 engine/view、签名历史、绝对 timer deadline、已投递输入的首次接收时刻和异步 callback 保留方式，并提供可审计实例标识。
4. 明确 ClockUnsafe 的关闭原因及作用范围。公共 CloseReason 没有 ClockUnsafe；H2 不能自行把它编码为 Time 或添加第二套公共 enum。所有实际签名出口必须查时钟/终止守卫。
5. 冻结 Manager 持久化、H1 停签 guard、durable action/outcome 和完整 grant/Ack 的提交顺序；关闭与后继启动之间不得出现可签名的中间态。OS restart 验收不在范围，不减弱正常持久边界。
6. H3 确认完整 ReplayRecord 的来源、批次完整性、顺序、重复/冲突检测与消费确认；不能仅凭 Hub 场景指令选 cid。提供 G 链推进导致较低候选排除的查询边界。
7. H1/H3 明确历史 Final/PublicationTask 的身份、对象保留与投递确认。终止新生产不得删除关闭前已形成的证明/body 或待发布任务。
8. 冻结真实 SessionEnd 场景参数及轨迹接口，展示 h11 首收/生产早于原准入上界、选择来自公开重放；确定模型用例、真实候选停车/选回用例与 M1 集成用例的各自通过条件。
9. H1 提供已接受的本地 firstSeen/clock 准入到 Commonware 确定性 verify/certify 的映射和对应 parent/decision 验证证据。保留 callback 或签名时检查不自动满足这项义务；H2 不另造第二套认证谓词。

## 独立复核与文档检查

[独立合同审阅](../verification/session-manager-healthy/contract-review-v1.md) 确认最终 draft2 适合仅准备交付；实现合同冻结仍 OPEN。最终合同 SHA-256 为 `837ed8f317e890c3b7a8e393f768cc69c62f7bcf4cec54790816ccc311340754`，审阅文件 SHA-256 为 `0bf4e7277c2feba7a739835a1f7d06f4500d214aec1f1835bec0bf4efc46bb6a`。已澄清 `lifecycle_id(&LifecycleEvent)` 的真实 API、GFR 选择 A 时无条件关闭 B、后续实现授权，并把 H1 准入/certify 映射作为 O9 显式保留 OPEN。独立 oracle 未因草案修订而变化。

文档检查核对两份清单共 50 项来源内容 hash、三份 oracle 封存 hash、11 条 YAML 向量的语法与唯一 ID，以及新 Markdown 的本地链接与空白格式。检查结果见 [document-checks-v1.json](../verification/session-manager-healthy/document-checks-v1.json)。这些是文档完整性检查，未执行任何状态机或协议验收。

## 后续验收与执行约定

验收应分层报告：独立状态机向量；至少一条真实 H1 engine 的候选停车/选回与停止路径；十五真实 worker 的 M1 全链路；同一结果的官方本地 Espresso 兼容项。模型 fixture 不能关闭原生或集成 gate。

验收重点包括唯一 descriptor、重复 Create/Enter、材料缺失/错误、未 Ready 与已 Selected、A→B→A、所有 phase 的 stop/关闭、关闭后迟到材料、批次重复/冲突/中途等待、旧发布任务、SessionEnd h9→h11 的固定时窗，以及四个不同 signer/正确 cid 域。验收预期详见本交付的独立材料，所有预期均不是测试通过记录。

本轮仅运行只读 Git/文本/哈希检查和文档一致性检查，没有 Rust build、可执行 fixture、仿真或官方网络提交。没有新增运行 seed/trace；文档中的时间向量是手写输入。后续任何重任务使用：

```text
数据目录 /mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager
全宿主锁 /tmp/megaroation-poc-heavy.lock
CARGO_BUILD_JOBS=2
```

确切 cargo/runner 命令必须等实现与接口冻结后记录，本轮不编造可执行入口。下一步由总协调接收本准备提交/diff，结合 M0 与 H1 准确交付收口合同；本提交不构成实现授权或 H2 已完成声明。
