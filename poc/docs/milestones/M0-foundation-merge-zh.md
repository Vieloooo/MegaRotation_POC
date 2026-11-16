# M0：远端合并M1可复用基础

你是独立merge chat，Astra/ultra；使用自己的managed worktree。先读[总计划](README.md)、根AGENTS/DESIGN与各组件交付报告。用户授权立即合并有用成果，本任务是组件集成，不宣布15-worker或整个PoC通过。

## 准确输入

旧main为`ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，启动消息另给出本计划的准确文档提交。保留历史，按依赖正常merge：

1. 03 `e9c15e04bc83b335668f3ecd837367aa87344a89`。
2. 02 `e1a435712ec492cae2bbff120a51ffd507eebf45`。
3. 06 `4b034dfbb6c3e1c7143208347b546daab8dfe298`。
4. 07 `788291735ce6db3c3eb78b4594a28d730cce71cd`（祖先已含03/06，不重复cherry-pick）。
5. 16 `5c34b400ba5245d7775a281566e21bf59708b147`。
6. 09 `c443ae925ed549b046731c2d8ef3dd9ce0311d24`，仅保留M2将用的已验证控制/L1组件。
7. 本次总协调计划提交，保持最新commander状态/计划；旧分支携带的旧管理文档不得覆盖它。

不合入01恢复WIP、11或12分支。09只作为M2备用源码，M1不运行批次、DA或D。检查各候选组件源码与其已测试SHA相同性，核对证据/未覆盖gate。基线普通原型不是fec7ade历史Fresh矩阵同源码，不能继承旧PASS。

## 所有权与过程

分支`codex/poc-m1-foundation`，数据目录`.build/commander-jobs/m0-foundation`。Docer先列合并范围/接口兼容，Verifier独立形成集成检查清单，再由一个Builder解决冲突。公共Cargo/lock/module声明只在必要时调整，不趁合并重写功能或替换已批准编码。需要实质性功能修复时回报具体差异/owner，不隐藏在冲突解决中。

保留各组件独立workspace。H1将接通runtime，M0不复制Commonware、不导入恢复WIP、不把fixture证明替换成默认true的实际验证器。源checkout、其他worktree和子模块保持。

## 验证与交付

在命名集成SHA上做独立复核和所选组件的锁定检查：根workspace/03、02 kernel独立harness、06两个crate/reader、07独立规则harness和G1/G2代表性重放、16离线适配/录制回放、09编译与接口检查。必要时运行普通native Fresh兼容基线检查，记录准确命令/profile；不为了合并强行跑完整恢复矩阵。使用宿主重任务flock及独立数据目录。

保存组件source-tree与测试报告绑定，报告哪些历史完整矩阵因源码不变被复用、哪些因实际集成改动而重跑。16的官方运行证据保持原限界，不能伪称完整PE时序/QC验证。验证失败则保留集成分支，main不动。

通过后核对main仍为启动旧值且没有被其他worktree检出，以比较旧值的方式更新本地main到已验证集成提交。回传旧/新main、集成SHA、冲突处理、命令/结果/证据及下游可消费路径。报告写`poc/docs/milestones/results/M0-...`及专属verification；不要改commander动态登记，不push。
