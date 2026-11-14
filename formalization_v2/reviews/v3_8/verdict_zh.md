# v3.8 审查结论：首块请求规则成立，跨成员请求接力仍待完成

本轮完成中英文 v3.8 候选稿、77 页英文 PDF，以及三份新的独立审查。三名审查者没有参与作者工作，没有读取彼此报告或旧版结论；各自核对了同一份 23 文件快照。

正式英文主稿 SHA256：`c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574`。审查前后哈希一致。英文 Model 与 v3.1 逐字一致；中文第 2 节也逐字节一致，共 9,080 bytes。

## 已完成的修补

每个 term 的第一块 DATA 要有权重严格大于 F 的 INPUT_REPORT。报告绑定精确 view 和 parent；有非空报告时，leader 必须先执行报告中最小键的请求，再按 FIFO 填充。正确副本验证同一份报告材料，不能拿自己的队列作额外否决。

收到请求的正确节点会在自己所属的已认证 snapshot 内转发。Pending 请求及转发责任不随 worker 退出删除。新 REPORT 只进入区域完整验证包；公开发布仍包含原接口要求的整个完整结果，不删掉原证据，也不为裸请求增加发布服务保证。

## 三项审查支持什么

| 审查 | 结果与准确范围 |
|---|---|
| [安全性](fresh_safety.md) | 在原抽象完整消息、Ready、密码学与 sound validation 前提下，REPORT → 实际 timer FINAL → public kernel → G 与 retirement 的安全链未发现 Critical Error 或 Justification Gap |
| [进展与请求证明](fresh_progress.md) | 普通 term 首块 4Δ、term zero 首块 6Δ 的条件性界，以及区域响应、全局 DATA 增长、地区反复获得接受服务、条件性请求 inclusion 证明，未发现上述两类错误 |
| [模型与发布接口](fresh_model_carriage.md) | 原完整公开结果 R0 与增加 REPORT 的区域包 R+ 的区分、原公开证据保留及 FINAL 存在性论证成立；没有把新请求查询或跨 roster 服务藏进假设 |

这是人工源码与证明审查，辅以实际哈希、逐字比较、编译和渲染检查；不是执行协议模拟或 model checking。具体 native adapter、CPU/带宽界、codec 和部署性能未被验证。

## 还不能宣布完成的地方

请求证明需要：当 x 还没被接受时，足够多的 accepted 首块证书含有一个在报告时知道且能选择 x 的正确成员。若全部正确成员已经知道 x，则任何合法报告集合都含这样的成员。

但“旧成员保留 x”不能直接推出“所有未来新成员都知道 x”。互不重叠的新 roster 可能先报告 EMPTY 并出块，然后才收到 x；同 snapshot 转发不跨过这个边界。原文的 `within region i` 没有明确指定不同 AUTH 快照之间的通信域，因此本稿没有自行增加跨快照 Δ，也没有让 Ready 等待一个尚未证明会完成的请求转移。

已单独写出[请求 owner 接力候选](../v3_7/v38_ownership_candidate.md)和[通信域源码审查](../v3_7/v38_network_domain_audit.md)。前者尚未纳入正式协议；它依赖的混合快照通信范围仍待解决。问题已向用户提出，未把默认选项当作答复。

**整个目标保持 active。** 本轮已经完成一次具体的“漏洞 → v3.8 算法修改 → 条件证明 → 新审查”循环；完整任意成员更换请求活性仍未完成，不能用 DATA 增长替代这个结论。

后续研究补出了每个身份都反复回归的更强请求饥饿轨迹，并修订、独立检查了请求接力候选。它们不修改本次冻结快照或上述条件性审查结论，见[后续研究记录](../../revisions/request_handoff_progress_zh.md)。

## 交付检查

- 六个原 Protocol、DATA/DRAIN、严格 Force cutoff、MSIG 优先级、旧 beta 匹配和六种生命周期状态保留。
- 英文编译成功，77 页全部渲染；五张 contact sheets 和新增请求规则的第 58–61 页已检查，无布局或引用警告。
- 中文保留完整结构，加入三成员具体例子；本地链接有效，公式与代码围栏检查通过。
- [版本差异](../../revisions/rotation_evolution_zh.md)已记录 v3.7 → v3.8 的实际修改、代价和未完成边界。
