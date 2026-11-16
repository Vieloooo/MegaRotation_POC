# H2：Regular所需最小Session Manager

新建Astra/ultra managed-worktree chat，按[总计划](README.md)角色流程。M0准确SHA+H1冻结生命周期/准备接口是实现输入；可先从v10 §02/03、03对象和07 GFR-LIFE-1独立写期望，不等H1全部通过。

## 范围与所有权

单Builder独占`poc/crates/session-manager/`（独立workspace）、`poc/verification/session-manager-healthy/`及专属合同/报告。复用03身份/descriptor与07生命周期codec；不修改H1原型文件或另造公共编码。真实worker接线通过H1接口在M1 merge时串行安装。

实现每worker的候选表、精确anchor/state准备、Ready/一次Enter、Selected/Excluded、Pause/选回原实例、SessionClosed/BranchClosed及持久Terminated的正常运行路径。A→B→A必须保留原engine/view/签名历史、timer和待完成工作，不能销毁后造新实例。先保留供M2消费的branch-close/assignment入口，当前不实现Force控制策略或RESUME。

regular必须覆盖提前准备后继候选和公开选中后的切换。末h10未及时公开时，以h9为anchor的候选可能需在h11准入窗口内提前生产并缓存；不能等SessionEnd之后才开始准备，再用移动窗口掩盖错过slot。后继窗口始终11–20。协议数据/公共生命周期来自真实来源，Hub故障脚本不能直接决定选中cid。

## 合同与验收

先冻结worker输入/输出、生命周期原子消费、状态准备证据、相同事件幂等、候选选择与调用顺序。独立向量覆盖重复Create/Enter、候选未Ready、A→B→A、关闭Preparing/Paused、迟到材料/结果、ClockUnsafe/stop和原签名状态保留。无OS重启profile允许进程内回切，不声称磁盘恢复验收。

先用同接口明确fixture完成状态机测试，H1可用后做至少一条真实engine候选停车/选回与停止测试；真实路径未接时不报该gate通过。与H3核对GFR-LIFE-1事件，使旧结果继续发布与停止新生产分开。交付精确接口/候选/验证SHA，供M1集成。
