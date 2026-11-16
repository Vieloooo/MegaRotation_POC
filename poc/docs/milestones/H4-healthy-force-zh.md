# H4：15健康节点的合法主动Force

复用原09 chat/worktree和已交付本地Anvil/合约组件，Astra/ultra。现在可写合同/独立向量；实现需要总协调提供的已验收M1完整SHA，M0已包含09交付`c443ae925ed549b046731c2d8ef3dd9ce0311d24`，由远端merge chat准备统一基线，H4不自行合并功能分支。不等01，不做自动停滞检测或RESUME。

## 范围与移交

15个BFT worker保持健康、持续运行。用固定授权M在预定逻辑时刻主动提交有效Force，经过真实Anvil transaction、手动出块/finalize、独立reader公共前缀和GFR的act顺序切换branch。提交原因记为测试计划的主动控制，不冒称观测到生产故障，也不把M改成committee投票。

H4独占`poc/crates/healthy-force/`、`poc/scenarios/m2-healthy-force/`及专属验证。本chat原有09合约/driver可作必要小适配。M1交付后，H3/H2停止编辑其交付范围，总协调将Force扩展所需viewer-gfr/session-manager具体模块串行移交H4；不能并发改同一文件，也不能复制GFR/Manager。合同列清全部精确路径。

最小控制平面仅使用ControlAuthority和L1 driver/reader。若现有Authority构造要求部署BatchInbox，允许部署现有合约依赖，但不运行Batcher/Certifier/DA、D或RESUME流水线。统一runtime调度，EVM收录、finalize、reader交付和控制激活分开。

## 规则与验收

- 合法签名/sequence/static policy、准确目标branch、Tail非HALT与公共覆盖；Force使用实际激活前G head为anchor，保留G，关闭旧branch全部Preparing/Running/Paused候选。
- 根据激活时职责是否贡献计算新r和固定window，再由03确定实际地区。不能在签名时猜anchor/r，不能让GFR只因cid已变而忽略同branch合法Force。
- 新分支独立状态准备与错时Enter，首个真实Final结果经Publisher→SimEspresso→GFR进入G。旧节点存活但不能继续为已Close实例生成新输出；旧分支迟到结果不能错误进入新G。
- 正例：健康持续生产中主动Force成功。边界：Force在途发生regular、与SessionEnd/旧publication同key、stale branch/no-op和重复控制幂等。至少保留一次明确无效控制对照，不扩为完整故障矩阵。

先冻结输入格式/签名/act与lifecycle顺序，再Builder/独立Verifier/同SHA Tester。回归M1满窗与SessionEnd，输出阶段时间和原因，交付候选给M2 merge chat；不自行合入main，不以ForceRecorded替代恢复生产成功。
