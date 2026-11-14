# 集成后的目标进展与待定模型域审计

日期：2026-10-19。范围：在 v3.1 Model 冻结、原目标不缩减的条件下，判断当前还有没有已识别的、无需先选择原模型解释即可消除现存目标缺口的算法或证明任务。本审计不修改协议，不调整 goal 状态，也不判断阻塞回合数。

## 结论

**当前存在明确的研究工作停滞点，但不是协议不可能性结论。** 公开导入、ownership/hydration 与 Ready adapter、历史任务设计、受限目录 adapter、runtime/control 集成，以及这些工作的局部独立审查，已经完成各自可独立推进的具体任务。最新 [集成审查](historical_integration_fresh_review.md) 已实际完成，未发现声明范围内的新 Critical Error 或额外 Justification Gap，也没有给出一个待修复的局部错误。因此不能用“审查还没结束”或“再修复其发现”继续描述下一步。

目前未识别出另一项能在**不选定原模型服务域**的情况下，消除完整请求进展缺口的具体算法或证明任务。剩余路径依赖尚未由原文唯一确定的端点与参与时间域。继续把这些前提重写成条件性定理、把候选抄入主稿，或者对未指定实现做性能测试，都不能解除这个依赖。这是对当前已构造路径及已知缺口的工作判断，不是对所有未来协议设计的穷尽证明。

## 按目标检查现状

| 目标要求 | 已有证据 | 仍缺什么 |
|---|---|---|
| 保留 v3.1 假设 | 本次直接抽取 v3.9 的整个 Model，与冻结文件逐字节相同 | 不能替用户选择原文未唯一确定的通信与参与域，再把该选择称作既有定理 |
| 全局 DATA 与 live-region service | v3.9 原有条件性论证保留；三份 v3.9 审查的范围见 `verdict_zh.md:5–25` | 历史任务 pin 版本尚不能直接继承较宽的 live-subset 进展范围：每个可能阻塞队列的历史 incumbent 都需要其服务前提 |
| retained request 最终进入 G | `rotation_v3_9_requests.tex:247–305` 给出 accepted-report awareness ⇒ inclusion；公开导入定理补齐真实载体后的本地工作 | 尚未从冻结模型推出足够多的 **accepted aware** 事件；存在请求字节、一次选择、Ready、完整但未接受的结果均不等于此事件 |
| 可审查的历史任务算法 | runtime 已给出确定事件顺序、固定切面、pin、延迟队列物化、原子 DATA/discharge 与 Handler 接口；control 已给出仲裁、持久 credit 和 legacy 处理 | 最新独立审查只确认这些局部条件性结论；未证明全目录覆盖、历史任务服务域或完整候选安全/活性 |

最新审查还明确使用原有 Espresso timestamps 持续增长条件；不能仅从 accepted DATA 增长推出时间戳增长。本审计没有把这项原条件替换成新假设。

## 完成的替代工作为何仍未接通请求证明

1. **公开导入**：`rotation_v3_9_public_requests.tex:134–175` 的起点是实际包含 x 的原完整公开结果。它不形成第一份 x-bearing 结果，也不保证导入赶在某个新 roster 的 REPORT 之前。故不能用其局部完成定理填补 awareness 前提。
2. **ownership/hydration 与 Ready adapter**：已经把有限 cut、实际传输、HYDRATED、精确 parent 选择和 Ready/REPORT 顺序做成可审查构造；局部审查没有发现额外错误。但其历史 owner 到同区未来 roster 的跨 AUTH 服务边，以及 owner 持续履责的时间域，仍是显式条件。证书本身不能证明该环境条件。
3. **历史 exact-AUTH 任务调度**：runtime 已消除稳定 pin 重试时不断扫描目录尾部或重启固定工作的算法依赖；control 已消除 optional H1 抢占全部本地调度 credit、其他 roster DATA 无限续借目标任务 OFF credit 的问题。这些是真实进展。不过，pin 一个不在服务域内的 incumbent 不能据此推出 DATA；跳过它让其他任务持续移动 G 又不继承固定任务、固定 anchor 的请求证明。这里没有把历史节点离线认定为必然合法的冻结模型执行。
4. **受限目录**：`historical_inventory_adapter.md:39–57` 已推导 canonical public-boundary catalogue，包含公开零输出 session；它尚未覆盖仅由 speculative handoff 安装且未形成自身完整结果的 AUTH。固定 L1 cut、有效单点 AUTH 证明和完整 control prefix，均没有自动给出完整 AUTH 枚举的完成条件。
5. **延长旧程序 drain 或允许已 Enter 的旧程序继续完成**：前者不能制造缺失 FINAL；后者虽能修复一个 first-carrier 执行前缀，仍留下 quota 已耗尽、未 Enter 即关闭、反复只含较小 key 的未接受结果以及导入晚于 REPORT 等边界（`historical_service_design.md:168–189`）。这些已识别边界不能靠再陈述“最终完成一些结果”消除。

## 最小必要输入与两条替代路线

**首先需要的是原模型的权威解释或明确的原来源定义**：`within region i` 是否覆盖不同 AUTH 中的正确端点；这些端点在 worker Completion/Terminated 后，只要仍有未完成的 node-level request duty，是否继续处于原正确通信域；若 AUTH 的 Delta 不同，跨快照消息适用哪一个已有参数。原文的不可变 AUTH、生命周期密钥故障预算和 “Keep serving data” 不能单独决定全部这些量词；它们也不授权把旧 owner 直接视为消失。证据见 `frozen_region_domain_audit.md:5–14,78–122,138–172`，以及冻结 Model `31–44,137–153,171–179,310–312`。

这不是要求同时批准下面所有路线的条件，也不是要求改写冻结 Model。若用户确认的是原模型本来的含义，应据此选择并完成相应构造；确认本身不等于端到端证明通过。

| 原定义/来源支持哪条路线 | 随后具体要完成的协议与证明工作 | 无须为此路线额外索取的另一条路线条件 |
|---|---|---|
| **A：持续 owner 与同区未来 roster 的混合 AUTH 服务已属于原域** | 将 ownership candidate v3 与 Ready adapter 合并到完整协议：持久 owner、有限已知 AUTH 前缀、paced hydration、OC/advertisement、精确 parent 和 first-own-session DATA 验证；重做准备、恢复、receipt/kernel、retirement 与全局 accepted-session 潜势的组合证明，再做 fresh whole-protocol checks 和中英文差分 | 不必采用历史 exact-AUTH pin 调度；因此不必先取得它的 all-history AUTH 枚举或 mandatory scheduling-MSIG 证明 |
| **B：原域覆盖重新服务的历史 fixed-snapshot voting task，采用历史调度路线** | 明确每个可能 blocking incumbent 的覆盖；从现有 L1/source 接口补出 cofinal canonical 注册，或证明另一个已存在公共发生点覆盖所有相关 speculative/zero-output AUTH；将已给出的 runtime/control 状态机纳入完整 handoff、安全、prepared response、任务公平和请求证明并独立审查 | 不必假定旧 owner 到 disjoint 新 roster 的跨 AUTH 传输；任务在其自身 AUTH 中获得 accepted aware service |

只有选择 B 时，完整目录来源才是该路线的额外必要输入/推导义务。可接受证据是现有接口的完整 authority-emission traversal、有限认证 key bound 加 terminating membership/nonmembership，或每个已安装 AUTH 的必达公共发生点及其 canonical 处理规则；当前 adapter 没有推导出它们。不能把这项目录要求变成选择 A 前也必须回答的问题。

若权威解释不支持 A，也不覆盖 B 所需历史任务，当前两个构造就不能宣称在原目标范围内完成。此时需要原来源中另一项足以支撑请求连续服务的明确接口，或由用户明确决定是否改变冻结边界/目标范围；本审计没有擅自增加这类许可。

## 原抽象接口与实现限制

runtime 的 complete-handler conformance 与 control 的 actual Submit 是当前候选明确保留的接口绑定点（runtime `154–182`；control `224–265,385–420`）。采用相应路线时，形式论证必须说明使用的是原 complete-message/提交动作，不能把任意有限 catch-up 叫作一个 Delta 操作，也不能把一次本地尝试当成实际提交。若原抽象 Submit 就是正确 committee 可执行的 enabled action，应在该抽象层完成证明；无需凭空添加外部 API 失败环境。

**这些不是所有路线共同的新用户门槛，也不要求在形式协议目标之外完成具体 codec、CPU、磁盘或 RPC benchmark。** 实现测试只能验证某个给定实现是否满足既定接口，不能替代原模型域的解释。本次停滞判断不以“没有做性能测试”为依据。

## 来源身份与验证

本次实际阅读最新集成审查后才形成上述结论。以下 SHA-256 为本次读取的实际文件；路径相对 `formalization_v2/`。v3.9 抽取 Model 的 hash 与冻结 Model 相同。

| 文件 | SHA-256 |
|---|---|
| `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |
| `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` |
| `sync_hotstuff_rr_part2_rotation_v3_9.tex` | `63dd6770de6aaed0f442c53c6fcd3685fb5a37b478140d67970ee2c4719058cb` |
| `sections/rotation_v3_9_requests.tex` | `593814bfdf9988439de249533da32c79543426870ac25d7087aebf5987226021` |
| `sections/rotation_v3_9_public_requests.tex` | `1c25bbbedcd64fecd660c74943e4c0d583caa8df95eb7cc47e5ca9d0e60e7ee9` |
| `reviews/v3_9/verdict_zh.md` | `9942c823858b2751efe7e0e3df250599cb98107418f75255128d5cae388a3750` |
| `reviews/v3_9/frozen_region_domain_audit.md` | `dc7bad3b1577df4bdd247428e8d195c6f4f7879053ff77f1274d449540fc26bd` |
| `reviews/v3_9/historical_service_design.md` | `bbfdcc3f29b395196e0871b115f98e9859d86bb291881c3da49ca37c882a972d` |
| `reviews/v3_9/historical_service_fresh_check.md` | `99fc1e945b682e5151dca7ffe24166a61e97ebd2f7c139d81fc0e9ca8b6624bf` |
| `reviews/v3_9/historical_inventory_adapter.md` | `563d8adc2fea2be65eef11f718a7835deca7814aa880e9051ef2fdec7033908b` |
| `reviews/v3_9/historical_runtime_integration.md` | `e383e49f572892cdeb2b7ec5d6a6dd9773f985502bbff2225337c1c6b1551cb3` |
| `reviews/v3_9/historical_control_integration.md` | `3b4528a9e720e20d3e786aff321dcb5ad74f1499e8bbe91b960f848f500537db` |
| `reviews/v3_9/historical_integration_fresh_review.md` | `abfbc2d659a81c2d8437ee068c94d3122a18423a7ba8bb7b39457a05f23d71da` |
| `reviews/v3_8/v39_ownership_candidate_v3.md` | `a71b64b8bb845463987de8d79b92fed6d8ef61db7a60eec0719dfe1b77a67bea` |
| `reviews/v3_8/v39_hydration_ready_adapter.md` | `efbd9c31860f891c7aa1c298b4428644dd7710354b4e2df17b884afc7eb0d3c7` |
| `reviews/v3_8/v39_hydration_ready_adapter_focused_review.md` | `2e94352daad879677fab930374ad2d0710e8e9c792a1b341eacdce3d9538b8f1` |

验证：源文件和报告逐项核对、直接 Model 字节比较、SHA-256 核验。本审计是有既有报告作为输入的 requirement/dependency audit，不冒充新的隔离 proof-verifier 审查。未执行模型检查、协议测试或 TeX 编译；只新增本文件。
