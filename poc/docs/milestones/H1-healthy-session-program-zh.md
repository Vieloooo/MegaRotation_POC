# H1：15健康worker与真实Session Program

复用原02 chat/worktree/分支，主模型Astra/ultra，按[总计划](README.md)角色流程。现在可冻结来源、接口和独立预期；实现只在总协调交付的**已验证M0完整SHA**上正常合并后开始，不能用浮动main代替。

## 本次范围

把已有02运输内核、03身份/Plan/执行与受控Commonware普通运行接通，启动15个真实OS worker。先在一个地区运行5个真实Simplex engine，其余10个进程保持独立Clock/reader和未选中状态；没有脚本签名或Hub代签。实现slot-aware Build/Accept、真实公开FinalVerifier和最小Create/Ready/Enter/Close接口，交付H2/H3使用。

采用无BFT进程重启profile。正常journal/sync/签名守卫/原始首收和完整grant-Ack不能省略；异常进程退出使场景失败/中断，不自动空状态重启。01跨PID恢复、全部fault cuts继续留在原任务。


单R0的REG-A是首个运行检查，不是H1的全部可消费接口。H1须提供真实Create/Prepare/Enter/Pause/Close能力，按cid隔离engine、签名历史、timer和storage，支持H2管理候选并在同一进程选回原实例。H1独占native接线；H2只拥有Manager纯逻辑，避免H2完成后再大改runtime。

## 路径与接口

继承[normal-running草案](../contracts/normal-running-reg-a-preparation-v1.md)，由本chat Docer修订成冻结H1合同、Verifier先给独立预期。单Builder拥有原型`src/normal_running/`、`src/bin/normal-{hub,worker}.rs`、`tests/normal_running_*.rs`、`poc/proto/normal_running.proto`及正常场景；本lead串行集成原型manifest/lock/build/lib声明。

01已安全封存后，总协调把**普通profile的最小共享扩展**移交H1：`native_scheme.rs`显式view/profile、`witness.rs`的PoP/namespace/section profile、`native_step_journal.rs`的profile-neutral绑定；必要driver/context扩展逐项冻结，旧构造器/Fresh语义不变。不得合入01未验收WIP、改native恢复状态机或另拷一套Commonware。真实签名域与03 PoP域须显式匹配，不能关闭验证来通过。

早发布进程身份/descriptor/Clock、Program Create/Prepare/Enter/Pause/Close能力、RegionalResult/完整proof/body/anchor验证接口和准入记录格式。H2拥有Manager，H3拥有publication/GFR adapter；你只提供其所需的明确runtime输入与输出接口。共享域对象变更交总协调单一owner处理。

## 本阶段验收

冻结NR-T/NR-A及对应边界：15真实PID、每区5身份/四票；10-height健康生产；真实3/4 signer、错域/subject/body；错时Enter 0/20/100/500/1500ms；Accept5a–5d等号/NotBefore/缺body或L1 WAIT；Clock误差与RPC latency分开；Close/stop/lease后无新签名；全部真实消息/输出受控。

先在准确组合SHA做Fresh兼容检查；修改共享scheme/witness/journal后复验受影响旧路径。Verifier从原始证书/正文/执行重算，Tester同一SHA跑固定seed、启动次序和墙钟暂停对照。独立验证放`poc/verification/healthy-session-program/`，artifact落任务数据盘。交付明确“健康profile”，不关闭完整01/04重启范围。
