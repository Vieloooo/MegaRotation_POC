# v3.5 独立审查结论

状态：**不能判为完整通过。** 本文汇总三份新审查，保留各自结论和准确快照；不覆盖原报告。

审查对象为 63 页英文稿，主源 SHA256：`2f47bca40d7ebdf80bb68a49a5fb8c479c4a50484a753d0134defc9381831d6b`。三名审查者均重新读取来源、核对完整 v3.1 与 Part I 依赖，并验证快照哈希。英文 Model 和中文第 2 节均未改动。

## 1. 哪些部分得到条件性支持

[安全审查](fresh_safety.md)未发现符合冻结假设的分叉、G 回滚、缓存导致公共解释不同或新的退役安全 Critical Error。固定交接、旧 beta 检查、MSIG 优先、全 phase 关闭传播、锁和计时器隔离的依赖已核对。

[活性审查](fresh_liveness.md)核对了固定 slot 后的有限 authority 准备、区域响应常数、健康签名额度和租期。若公共响应界成立，后续的时钟求和和 owed 集论证可继续成立。这里的“若”不能省略。

Ready 和 SOURCESTEP 的时间论证仍依赖已有完整消息处理模型与已准备的直接可执行状态；它们不是对任意数据库导入或具体 native adapter 的性能认证。

## 2. 发布接口缺口

**原始区域结果已被 Espresso 收录，不等于外层 EXPORT 被 G 接受。**

例如，结果的 DATA、FINAL 和依赖都真实完整，但抢先发布者填写了错误 base。原发布保证允许“已经 recorded”满足交付义务；纠正 base 后的接受重试没有统一时限。因此 v3.5 不能直接用 `T_pub + T_E` 推出接受响应界。

[假设审查](fresh_assumptions.md)给出了超过所称有限响应界、随后才重试成功的执行，将其列为 Critical Error；活性审查将缺失的桥接论证列为 Justification Gap。两者都不接受当前公共响应证明。依赖该步的持续全球进展与地区服务结论仍未验证，不能用其中一个审查的较弱分类表示“已通过”。

## 3. terminal 与下一 authority 的依赖

当 quota 为 1 时，首个正结果也是 terminal。v3.5 把它的 EXPORT 更新与后继 `PrepareOwed` 放在同一原子检查中；后者会查询 `AUTH[r+1,0]`，而该 authority 不在已证明准备完成的集合里。

需要明确：当前 terminal 的 G 提交和公开，是否先于后继 authority 等待。仅写“无合格后继也能接受”不够，因为缺 authority 与已证明没有合格后继是两件事。

## 4. 请求 inclusion 仍失败

审查者验证了稿中反例：Byzantine leader 先交付一个合法空 DATA；正确 recovery leader 在很晚到达的 authority 下提出最老请求 x；Force 在 receipt 形成前关闭该版本。正输出使下一次切到新 slot，下一 authority 可以再次很晚才到。

这条执行可以让 G 和地区 credit 持续增长，而 x 从不进入 G。它没有使用失联的正确 quorum、超出同步界的区域消息或不公平的实际正确 leader 进入。请求始终留在 pending queue 也不能单独修复它。

## 5. 版本推进

v3.5 快照与 PDF 保留为本轮候选及审查证据。v3.6 正在按[修补设计](v36_publication_repair_design.md)改变接受接口：

1. 从已收录的签名区域结果与完整证明推导下一条兼容 DATA；外层提示和无关坏条目不能否决它。
2. 逐 DATA 持久提交；terminal 的提交先于后继 authority 和选区。
3. 明确可恢复的记录内处理位置，保留 Force prelude、原有 cutoff 和 DATA/DRAIN。
4. 修正旧 observer 图里统一写成 `r_c+1` 的残留公式。

下一版必须证明这确实覆盖原 already-recorded 情况，而不是偷加重试时限、重新定义完整结果或假设无限解码能力。请求服务单独研究；本轮没有将它删除，也没有把完整目标标记为完成。
