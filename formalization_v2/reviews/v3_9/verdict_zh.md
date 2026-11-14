# v3.9 审查结论：公开请求导入已补齐，完整请求活性仍未完成

日期：2026-10-19。对象是 [manifest](manifest.json) 固定的 24 个英文源文件及依赖，主稿 SHA-256 为 `63dd6770de6aaed0f442c53c6fcd3685fb5a37b478140d67970ee2c4719058cb`。

三名新的审查者分别核对安全、进展、模型与导入接口。各自仅读同一冻结快照和审查范围，未用作者笔记或其他审查报告作为依据；审查前后均核对 24 个 hash，没有修改协议。三份报告都在各自声明范围内未发现 Critical Error 或 Justification Gap。

| 报告 | 通过检查的内容 | 未宣称的结论 |
|---|---|---|
| [安全](fresh_safety.md) | 未验证字节与 pending 分离、身份去重、固定 context、原 receipt/kernel、cutoff 和持久责任接口 | 任意部署实现已经安全；完整请求活性 |
| [进展](fresh_progress.md) | 固定有限导入任务持续完成；与既有条件性区域响应、DATA 进展和地区服务论证相容 | 第一个含 x 的完整结果必然形成；导入有统一时限 |
| [模型与导入](fresh_model_import.md) | Model 与 v3.1 逐字一致；持久本地算法没有冒充跨成员服务保证；精确 parent 检查与 REPORT 的关系 | 新旧 AUTH 间已获有界通信；任意 native adapter 满足原完整消息处理界 |

## 这版实际证明了什么

设原完整公开结果实际包含请求 x，正确节点取得并保留其真实字节，并继续执行规定的本地队列。则它最终把 x 存入未验证候选集合。如果已有或后来实际准备出的固定 parent read context 使 x 通过原请求检查，检查任务继续执行后，x 最终进入 pending，除非该身份已经在 G 中。

两步有不同条件。下载到字节不等于认证成功；已认证也不等于在任意新 parent 后可执行。导入不修改 G、不复活被 Force 关闭的程序、不让 Ready 等待后台队列。新收到的任务不能反复改变旧任务的输入或抢到其前面。

## 尚未解决的完整目标

如果只有某个旧成员收到 x，而含 x 的 proposal 每次都在足够 FINAL 形成前被 Force 截止，那么原 Publish Layer 的完整结果入口一直没有成立。v3.9 不能导入一个从未成为公开完整结果的请求，也没有证明每个未来 roster 在首份实际 accepted REPORT 前已经获知 x。

因此，“G 持续增长”“健康合格地区反复得到 DATA 服务”和“每个有效请求最终进入 G”仍是三项不同性质。前两项的既有条件性证明不能代替第三项。全目标保持未完成；也没有把故障地区、失联 quorum 或任意实现延迟悄悄包括进通过范围。

独立的 [hydration/Ready adapter](../v3_8/v39_hydration_ready_adapter.md) 已完成[局部条件性审查](../v3_8/v39_hydration_ready_adapter_focused_review.md)，但未并入本版。原“within region”通信域和历史投票身份的服务范围仍需从原定义确定；没有依据前，不把它们写成已获保证。

## 验证方法

英文稿 79 页，隔离编译及渲染检查已完成；所有页面 contact sheet 和新增内容的放大页面均检查过。英文 Model、中文第 2 节与冻结 v3.1 保持一致。源码身份、链接、PDF 页数、六个 Protocol 标签和最终日志另记在 [build verification](build_verification.json)。

这是人工对抗审查和形式论证检查，没有协议执行器、model checker、具体 codec 或 native 性能测试。编译成功证明文档可生成，不证明协议正确。
