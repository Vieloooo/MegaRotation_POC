# v3.11 审查结论：原模型范围内的安全与完整活性链通过检查

三名新的独立审查者只读同一个[冻结快照与 manifest](manifest.json)、审查范围和冻结基线，未读作者笔记或其他审查者报告。各自核对的 26 个文件在审查前后完全一致。主稿 SHA-256 为 `de2e36905fef1e2de7bcc53f0cd8f4a95eca92e4561d347b33bc260e7dc58bef`。

| 报告 | 检查范围 | 结果 |
|---|---|---|
| [安全](fresh_safety.md) | 原区域 timer/lock 与退役、OC 和恢复/DRAIN、真实 FINAL 来源、完整 R0、Ready、Force cutoff、公共追加权限 | 未发现 Critical Error 或 Justification Gap |
| [完整活性](fresh_progress.md) | 有限接力与每份证书的覆盖、OC 首次持有转发、准备后响应、全球 DATA、单区续任、地区服务、原 ingress 到请求纳入 G | 未发现 Critical Error 或 Justification Gap |
| [模型与接口](fresh_interfaces.md) | 原假设的语义使用、AUTH 前缀、持久 owner、真实收件/传输、native/消息处理范围、公开与区域材料的分界 | 未发现 Critical Error 或 Justification Gap |

## 现在已经接通的证明

原输入条件保证请求实际到达持续服务的地区。取一个收到完整请求的正确成员，它在请求进入 G 前持续保管和服务这些字节。以后同地区的新成员通过已知 AUTH 前缀找到它，取得固定有限 cut，并在签 HYDRATED 前把数据真正发给全部目标成员。

正确成员取得精确 anchor 的状态后，完成一次有限选择并签 offer。每份合法 OC 都含至少一名正确签名者，所以它选择的请求不会排在该成员已覆盖的请求之后。首次取得 OC 的正确成员必须持续转发；关闭 session 不取消这项职责。新 session 的第一份 own-session DATA 必须执行非空 OC 的选择。

准备启动的 pacing 使一个固定请求只会错过有限多个早期 slot。每个 slot 能贡献的 accepted DATA 又有限。因此持续服务带来无限多个覆盖该请求的首块机会。每次都接受该请求，或接受一个此前尚未入 G、排序不大于它的请求；这样的较早请求总数有限，所以不能永久跳过它。

## 结论的准确范围

在 v3.1 原模型、原进展条件和正文明确的完整消息/可执行状态接口下，现有论证覆盖：

- G 的公共前缀安全；Force 保留已有 G，关闭旧 beta 的追加权限。
- 合格服务集合非空时持续的全球 DATA 进展；选择器允许只有一个合格地区时同区续任。
- 每个持续满足服务及资格条件的地区反复贡献真正进入 G 的 DATA。
- 正确客户端请求满足原 ingress、可装入 DATA，并在相关 opening state 持续 admissible 时，最终进入 G。

“只有一个合格地区”不把其他已分配 quorum 任意永久失联的执行重新算作满足原区域同步/故障前提；“地区得到服务”也不表示让离线地区出块。请求结论不只依赖空块增长或被选中。

Model 原文保持不变。用户明确的两项原含义是：同地区新旧正确 AUTH 成员受区域通信保证覆盖；发送到 region 意味着地区每个成员最终收到。未增加跨地区界、客户端 deadline、所有未来 roster 的初始知情或任意 native 工作的统一时限。

这是人工形式论证与对抗审查，不是可执行 model checker 或部署性能认证。具体实现仍须实现文中声明的接口；原始数据库导入、任意 archive catch-up、容量和资源成本未因此获得常数时间保证。

## 版本与交付

v3.10 的 CE-OC、JG-I1 与排版错误保留在其[审查结论](../v3_10/verdict_zh.md)。v3.11 的[修补说明](change_summary_zh.md)、[完整中文稿](../../sync_hotstuff_rr_part2_rotation_v3_11_zh.md)、[英文主稿](../../sync_hotstuff_rr_part2_rotation_v3_11.tex)和[英文 PDF](../../sync_hotstuff_rr_part2_rotation_v3_11.pdf)对应本轮结果。最终构建与文件身份记录见 [build verification](build_verification.json)。完整目标另按逐项交付审计核对，不以报告数量代替验收。
