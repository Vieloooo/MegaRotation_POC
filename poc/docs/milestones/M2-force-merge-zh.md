# M2：远端Healthy Force集成验收

新建独立Astra/ultra merge chat及managed worktree，分支`codex/poc-m2-force`。输入为准确M1基线、H4候选/独立报告和旧main SHA，由总协调在派发时填写。按[总计划](README.md)角色流程，保留功能历史，不由主协调本地直接merge。

单一集成writer处理H4与M1接线/冲突；Verifier先从H4合同/v10/v2导出预期。命名最终候选SHA后Tester执行：15节点全程存活的主动Force成功、同branch在途regular、同key次序、明确无效控制及幂等对照，并回归M1两条regular路径。

成功要求真实授权控制已由GFR在正确位置激活，旧G保留、旧branch候选正确关闭、新window/region/anchor正确，且新branch首个合格结果实际进入G。记录submit/include/finalize/activate、每worker Ready/Enter/Close、regional final/publication/G accepted及A/B同位置结果。保留真实L1 receipts、SimEspresso日志、proof/body和离线checker结果。

不重启worker、不依赖01、D/RESUME、批次服务或Web。正常输出仍受完整grant/Ack和持久边界约束。官方Espresso发布沿M1已固定的独立测试方式保留，不能把wallclock引入确定性主trace。

独立复核和同SHA矩阵通过后才声明M2 PASS，并按旧值比较更新本地main；失败保留候选和诊断，不force覆盖。提交只涉及M2集成/验收和明确修复，输出新的可复现基线与一条运行命令/一条离线检查命令。没有远端push或公网部署。
