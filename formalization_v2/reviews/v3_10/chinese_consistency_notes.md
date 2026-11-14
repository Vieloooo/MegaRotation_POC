# v3.10 中文 companion 一致性记录

本记录是中文写作与机械检查记录，不是独立 safety／liveness verdict。英文审查快照在编写期间冻结；未修改英文文件或旧中文版。中文状态保持“正在进行新一轮独立审查的候选”，最终 proofread、审查状态、英文校正与最终 diff 由主任务整合。

中文文件：[sync_hotstuff_rr_part2_rotation_v3_10_zh.md](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md)。写作依据为 v3.10 英文正文、全部 v3.10 included fragments 与 [作者原模型澄清](../../revisions/v3_10_scope_zh.md)。原版未变材料直接保留；变化集中在新增请求接力及其接口和证明。未加入历史任务 pin scheduling 或额外公共请求服务。

## 冻结 §2 与结构

- 取第 `## 2. Model and assumptions` 标题起、到第 `## 3. Regional engine` 标题之前的全部 UTF-8 bytes（包含原空行）。
- 与 v3.9 完全相等：**True**；长度 **9080 bytes**。
- §2 SHA-256：`db9f1d7de0a6c9896b436f476fef2bc37148ec4684118ed793523f8930f6b4c0`。
- 保留全部 58 个原编号二／三级章节的相同编号次序；相关标题随新语义更新。新增细分解释放在原 §6.5、§8.1 内。
- 当前旧 v3.9 中文 SHA-256：`d899054ecdf8d3a0c8a654033c4ebd19503033c142e68ff3286b1cd1fcb628b3`。本任务仅读取该文件。

## 修改映射

| 位置 | 中文入口 | 具体调整 |
|---|---|---|
| 导言与状态 | [第 1 行](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md) | 以 v3.10 文件和审查范围为链接目标；说明作者澄清、完整候选论证与待新审查状态。 |
| §3–4 区域验证与 BFT 动作 | [第 301 行](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md) | 加入真实 OC／advertisement／REPORT provenance；区分 first-own-session DATA 与 term-first REPORT；保持 ANCHOR 和 evidence-first 独立。 |
| §6.3–6.4 Handler／owner／cache | [第 732 行](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md) | InputPrepared、request-handoff retained objects、beta-independent 精确 key、source callbacks 与 prepared tail。 |
| §6.5–6.6 Ready／bootstrap | [第 924 行](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md) | Ready 接受任意完整匹配 OC，不额外等 HYDRATED、directory 或私有 job；STATE_CANDIDATE 无循环；实际 packet／handler footprint 与旧计时范围。 |
| §6.8 退役 | [第 1138 行](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md) | 新 fresh-advertisement gate；owner cuts、transfer jobs、state-candidate delivery 和 OC 工作持久接管。 |
| §8.1 请求接力与选择 | [第 1371 行](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md) | 完整原公开导入保留；增加 known-key directory、context bytes／barriers、immutable descriptors、fixed fragment slots、raw completion hook、all-target HYDRATED、state candidate、finite cut、OC、finite exceptions、四例与物理退出边界。 |
| §8.2–8.6 运行期证据与请求计数 | [第 1631 行](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md) | OPENING／DISCHARGED REPORT 与 signed initializer advertisement；原 R0 全量发布；4Delta／6Delta；session-first finite-potential inclusion；late cut 和 retry 解释。 |
| §9.6–9.11 完整候选进展链 | [第 1951 行](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md) | 固定 slot 准备 maxima 包含 ownership／OC；原 DATA response 保持；原 ingress 到 recurring region、finite exceptions、真实 FINAL witness 和有限势函数逐箭头说明；删去已过时的未澄清成员范围文字。 |
| §10 发布与保留 | [第 2147 行](../../sync_hotstuff_rr_part2_rotation_v3_10_zh.md) | 明确只擦除新 regional sidecars，保留整个原 R0；扩展持久 owner 职责并链接新的条件性 inclusion。 |

## 已完成的覆盖核对

- 作者“覆盖同一地区的新旧正确成员”的答复准确保留；不改 §2、不引入跨地区界、不将失败或任意离线机器算作正确 owner。
- 历史目录来自实际 admitted slot 的有限 AUTH predecessor lineage；只取已知存在 keys，不枚举未来 authority。
- Context capsule 完整字节先发送、source-validation trigger 后发送、first query 再发送，两个严格 barrier 与 release pacing 均列出。
- Descriptor／metadata／fragment lengths 和 budgets 冻结，正确消息不被 wrong-sender 杂包判失败；Byzantine silence 不无限扩大任务。
- `RawJournal` 的最后碎片增量 receipt hook 在 selection cut 前保留完整 raw handles；不把尚未扫描 source root 当成请求 handle。
- HYDRATED 只在实际全体发送和严格传播等待后签 share；share／certificate 的持久收集和实际 relay 不是可选。每一合法证书都有正确 signer，其因果传输覆盖全部正确 targets；不使用不同证书之间的 honest intersection。
- STATE_CANDIDATE 在 Ready／OC 前独立传送原完整可执行状态；selection 在完整 HYDRATED 后 freeze，等待真实 exact-anchor state 后完成有限原检查。
- OC 最小值只针对 supplied signed keys；Ready 不传第二套 HYDRATED 或 directory，不等私有 job，也无 local-OC veto。
- Initializer advertisement 绑定真实选中 OC；报告只确认 OPENING 或 DISCHARGED。原 ANCHOR、forwarding、Quit inspection、rank、locks 和 timers 不由坏 sidecar 压制或重新定义。
- Session 首块义务覆盖 recovery 回 opening、保留后继、DRAIN、receipt／Completion；所选 body 先执行、精确 ancestry exclusion 不只查当前 G。
- 原 Publish Layer 全量 R0 保留；只擦除新增 regional sidecars。真实 correct FINAL signer 的 timer-origin Vote 提供 witness provenance，不把 publisher 任意 annotation 变成证明。
- 准备与 beta retry 复用、有限 exceptional slots（含 equality 与 late assembly）、原 ingress 到 recurring serving region、无限 accepted session 首块和 finite potential 的链条均展开。
- 4Delta／6Delta 与全局 L 界仍以原 admitted complete-message／native interface 为范围；明确没有新 archive、CPU、pool、native reconstruction 或未来 roster 统一时限。
- 四例覆盖 old/new disjoint membership 转交 x、同 r/A 的 Force retry、cut 后到达 x、accepted candidate 与本地 OC 不同。

## 机械检查与待整合项

- 本地 Markdown links：最终检查 19 个，全部 targets 均存在（v3.10 PDF 已由主任务生成）。
- `git diff --no-index --check`：无 whitespace diagnostics；返回 1 仅表示两个版本有差异。
- Markdown fenced blocks 配对、display-math delimiter 配对通过；没有生成中文 PDF，也没有以这些检查充当模型检查或证明验证。
- 英文 ReadyBundle OC-size 公式的 `+mathbf 1` 少一个反斜杠已报告给主任务；中文使用预期的 `\mathbf 1` indicator。主任务确认在冻结审查之后处理英文校正。
- 当前中文为 2187 行、278756 bytes；SHA-256：`a16fa6f90f791e9ade6bdf7b0eec9a37ab33fb2e826cb152f0d75cdf4a0271c1`。后续主任务修改状态或 proofread 后需重算。

## 本次核对的英文输入指纹

这些是编写完毕时的 live source hashes；正式审查以主任务保存的 snapshot／manifest 为准。最终英文 typo 修复或审查驱动修改需要重新对照。

| 输入 | SHA-256 |
|---|---|
| `sync_hotstuff_rr_part2_rotation_v3_10.tex` | `1555702839d5d117a95428953d91efcd28154c96a719119a2d178086d7e45bab` |
| `sections/rotation_v3_10_exports.tex` | `5b5cdd9745d255cc30961bd551217f8a828960edc2cdf871e2c7a6ae7db4098e` |
| `sections/rotation_v3_10_input_selection.tex` | `484249a49dbc293140d61bbf518f3df6594cbff37d3ec7cd7b7bc7cb1d1ddce8` |
| `sections/rotation_v3_10_liveness.tex` | `3c0c1a5943df36908c09ec34dac34dcbcae845fd39c3074ed3ac7cc830615ea1` |
| `sections/rotation_v3_10_ownership.tex` | `5952df7ab66e12fdbbdbb48f8b6c3c4dd3beeca03ac6d11db9e79ecd97319f1c` |
| `sections/rotation_v3_10_public_requests.tex` | `1be92deba4065fe976b82238f5d0ea827eb0abe66bd0c2703bfd983f96a711da` |
| `sections/rotation_v3_10_ready_bundle.tex` | `a2d57b26d16c920b47c6911a43185cc4a7bf20a74df64a032758e1dbd878869a` |
| `sections/rotation_v3_10_receipt_kernel.tex` | `5891df938ef061b57ccae04bef91a3df907b4e71ebd7542468883d03ec5e38ae` |
| `sections/rotation_v3_10_requests.tex` | `3e406d464e0d7468c296299a5c7cfa76ce919a6dd96d6b3c3d93a2083bea4ffa` |
| `sections/rotation_v3_10_retirement.tex` | `e047357f63aaf81e6877b9c06b6ffd0e1ab5592cc77df810fe0bc18e85f8d785` |
| `sections/rotation_v3_10_source_step.tex` | `179822ad6248789ab14255456ab9480bdd03529562d8907ed8b15964bde68f57` |
| `revisions/v3_10_scope_zh.md` | `dbeed1fa2439f379722deb0875cda3189ea44550224e3a716eafb391b669af8c` |
