# M1：远端Regular集成与验收

新建独立Astra/ultra merge chat，managed worktree和`codex/poc-m1-regular`分支。输入由总协调填写：M0 SHA、H1/H2/H3各候选/独立报告SHA、旧main SHA。没有准确提交就先准备矩阵，不猜浮动HEAD。按[总计划](README.md)角色流程，合并也是一项远端任务。

## 集成与所有权

按H1→H2→H3接口依赖正常合并，保留历史。一个集成writer负责launcher、实际worker/服务接线、必要公共声明与冲突处理，独占`poc/scenarios/m1-healthy-regular/`和专属验证/报告。组件缺陷回对应owner或经明确移交后修复，改动后重验受影响范围。01恢复/批次/D分支不进入本里程碑。

## 必须执行的主场景

- 15个真实健康worker，三个五成员委员会，全部签名来自所属worker；地区R0→R1→R2，窗口1–10→11–20→21–30。满窗路径在R2首个正确结果进入G后成功。
- 所有worker持续存活，错时Ready/Enter；后继必须准备真实anchor/state，不使用全员barrier掩盖时差。至少覆盖一次进程内候选切换/选回，不使用OS重启。
- 另跑末结果未在公共deadline内完整公开的SessionEnd路径：可通过合法的publication延迟构造，worker仍健康；后继窗口不改，h9 anchor后的padding10及正确后继结果由真实规则处理。明确该场景改变了发布时序前提，不把它称健康发布活性证明。
- 区域证明、完整body验证、publication、SimEspresso finalized/reader received、G接纳、lifecycle与各node Enter/Close分别记录。A/B同replay position状态一致；独立checker重算实际证明、quorum、hash/执行/窗口。
- 保存来自实际worker的最终结果包，用16的官方本地节点真实提交/读回，核对同一字节/hash及来源；官方wallclock/profile另记，不能进入SimEspresso的虚拟时序证明。

固定seeds7/42/99、wall pause0/2ms与约定启动次序对照，界限和观察区间在冻结矩阵中明确。不调用同cid恢复；意外进程退出导致该run失败/中断。CLI启动/运行/离线检查足够，不建设Web、DA或D。

同一集成SHA独立复核与Tester通过后，才报M1 PASS并按旧值比较更新本地main；不把component smoke当端到端成功。交付manifest、原始trace、实际15 PID、证书/body、官方读回记录、准确命令与限制，给H4一个固定M1基线。
