# v3.7 审查结论与剩余目标

**本轮三份新独立审查没有发现所列抽象安全、持续 DATA 进展和地区服务证明中的 Critical Error 或 Justification Gap。请求 inclusion 仍有模型内反例，因此没有把完整目标标记为完成。**

审查对象是固定的 v3.7，主源 SHA256：`440d639a3f9a7a0a236836153880171db40c904a1862eef2444b8313c9d2b476`。三名审查者分别读取来源、完整 v3.1 和 Part I 依赖，核对全部 23 个快照文件，没有读取其他审查结论。英文 Model 与中文第 2 节仍与 v3.1 逐字一致。

## 1. 这次解决了什么

v3.6 已解决错误 EXPORT 提示和 terminal 等待下一 authority 的问题，但公共解释器仍可能为了一个旧引用重新读档案，或者被发布者附带的假 admission 路径拖去读取未知 `AUTH[r,g]`。

v3.7 分两步处理：

1. **保留完整公共字节。** 节点先补齐一个固定的有限历史区间，之后持续保存新 Espresso 原始输入，包括被拒绝的记录。程序关闭、beta 改变和 G 前进都不自动删除它们。索引不能代替字节。
2. **用原 FINAL 证明区域输入有效。** 固定投票快照中超过 F 的签名必含正确签名。正确成员只能在完整区域验证后的 timer/ancestor Commit 产生 FINAL；收到 receipt 后的提交不再产生 FINAL。因此该推导没有用“票据有效”循环证明“票据有效”。

公共验证由此无需重放发布者随意选择的 PROPOSE、recovery authority 或 native sidecar。它仍独立检查真实 incoming boundary、当前 beta、G 的精确后继、quota、Espresso reference 和 Force cutoff。原区域内部完整验证规则保留。

这是明确的公共验证算法修订，不是新增可信委员会，也不是只删掉原有效性检查。证明覆盖的是被 hash 绑定、正确成员实际检查过的输入，不是任意附带执行状态。

## 2. 哪些性质已得到本轮检查支持

| 性质 | 本轮结果 |
|---|---|
| 全球 DATA prefix 不分叉、不回滚，旧 beta 在 cutoff 后不能追加 | [安全审查](fresh_safety.md)核对公共 kernel、独立验收条件、逐 DATA 提交及 Force 顺序，未发现证明缺陷 |
| 每份 session 唯一启动、状态隔离及退役安全 | 同一报告重新核对历史票据、REGULAR、全 phase 关闭传播和已有 timer 收尾 |
| 有可服务的 eligible region 时，全球 DATA 不会永远停在同一高度 | [活性审查](fresh_liveness.md)核对有限准备、区域响应、SOURCESTEP、原发布保证和时钟反证 |
| 每个持续满足资格和区域服务条件的地区反复贡献被 G 接受的 DATA | 同一报告分别检查无限 Force 和有限 Force 两种情况的 owed／regular 证明 |
| 新 public kernel 与逻辑 EXPORT 规范化没有暗加故障或读取时限假设 | [模型与传输审查](fresh_model.md)核对原语义、完整结果保留、类型引用及时间界适用范围 |

响应证明继续使用

\[
L=\max_i\{2T_E+2\Delta_i+R_i+T_{\rm pub}\}.
\]

这里的准备、源码保留和完整区域消息接口必须已经满足规定；没有给首次历史补齐、未知 authority 或任意数据库导入一个新的统一时限。`T_pub` 仍只用于原来的完整区域结果，不扩展到裸请求、QC、单个 FINAL 或一次接受重试。

## 3. “只有一个 live region”要准确理解

证明不再要求选择器必须找到另一个地区。同一地区可在符合条件时续任，所以非空 eligible 集可以只有一个地区。

但原区域故障、正确投票权重和通信前提仍适用于被选择的 snapshot。该结论不能改说成：其他仍可能被选中的地区永久失去正确 quorum，也不影响证明。永久 OFF 的地区也不属于持续 eligible 地区服务结论。

这不是扩大失败模型后的容灾认证，而是在不改 v3.1 假设下得到的准确性质。

## 4. 请求仍可能一直进不了 G

合法反例仍是：每轮先有一个 Byzantine 空 DATA 进入 G；正确 recovery leader 很晚取得该轮所需 authority，提出最老请求 x；Force 在完整 FINAL 形成前关闭旧 beta。因为已有正输出，下一轮转到新 slot，又可出现新的 authority 等待。

这样可以同时发生：

- G 不断增长；
- 每个地区反复获得接受 credit；
- 正确 leader 反复实际进入并提出 x；
- x 始终没有进入 G。

若 Force 更早发生，x 甚至可能只有单个 FINAL，未形成受发布保证保护的完整结果。因此不能直接把“收集迟到的完整结果”当成完整请求活性修复。

后续需要区分：是否保持当前 FIFO 选请求算法，还是允许修改它，同时保留网络、故障、L1/Espresso 可见性等原假设。该范围已向用户澄清；尚未自动采用新的选请求协议，也没有把请求问题从跟踪目标中删除。

两份后续设计已单独保存，未改动本轮文稿：

- [保留 FIFO 的 rotation／authority 路线](request_fifo_rotation_design.md)：检查持久 initial-leader cursor 和有限 membership round；指出 AUTH producer 的实际控制权及队列／身份连续性仍需解决。
- [输入报告与 primary request 路线](request_report_scope_analysis.md)：给出候选接口、时序和条件性包含证明；明确它会改变选请求行为，跨 roster 的请求可见性和原容量／负载限制仍是待证义务。

这些是研究方案，不是已经通过本轮三份审查的 v3.7 规则。

## 5. 验证边界

本轮是人工数学／源码审查，加上哈希、模型原文、LaTeX 编译、链接和页面检查。72 页 PDF 已生成并检查。没有运行完整实现的 model checker，也没有测量 native adapter、档案 I/O、具体 Espresso 部署或旧二进制 codec。

逻辑 EXPORT 的保留引理不等于所有旧字节格式都已兼容；完整区域 handler 的抽象时间界也不等于任意外部 observer 的 CPU 性能。文稿和三份报告均保留这些实现义务。
