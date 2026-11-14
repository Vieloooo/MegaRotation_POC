# Flow Aptos 和 MonadBFT 轮换资料索引

对应[中文学习笔记 12](../../zh-CN/12_hotstuff_regular_leader_rotation.md)。源码读取日期为 2026 年 10 月 9 日，branch 在读取时解析成固定 commit，再按该 commit 下载。所有原始版权与许可证头保持不变；这些文件是研究引用材料，不是项目的实现修改。

## 固定源码版本

| 项目 | Branch | Commit |
| --- | --- | --- |
| Flow | `master` | `b85954c90eb9f8ab4b62a71641b2a53fb311a170` |
| Aptos | `main` | `bc7baea6b85295edb533c22284f8474613d32624` |
| MonadBFT | `master` | `33e55fcca08ed3690a9a968217a0c54d33000fe4` |

[code-manifest.json](code-manifest.json) 列出每个文件的原始 URL、本地文件名、commit 与 SHA-256。这些是公开仓库快照，不是对三条主网部署二进制或配置的证明。

## 机制与源码对应

| 机制 | 固定文件中的定位 |
| --- | --- |
| Flow 加权 leader schedule | `committees/leader/leader_selection.go`，`ComputeLeaderSelection`、`weightedRandomSelection`、`LeaderForView` |
| Flow 正常与失败推进 | `pacemaker/pacemaker.go`、`pacemaker/view_tracker.go` 的 QC/TC 路径 |
| Flow 进入与投票条件 | `safetyrules/safety_rules.go`，`validateEvidenceForEnteringView`、`HighestAcknowledgedView` |
| Flow two-chain finality | `forks/forks.go`，`checkForAdvancingFinalization`，尤其 parent 与 certified child 的连续 view 条件 |
| Aptos proposer 权重 | `liveness/leader_reputation.rs`，`get_valid_proposer_and_voting_power_participation_ratio` |
| Aptos round events | `liveness/round_state.rs`，`process_certificates` 中的 `QCReady` 与 `Timeout` |
| Aptos vote routing | `round_manager.rs` 1551–1573；包含 broadcast 配置分支 |
| Aptos optimistic proposal | `round_manager.rs` 1576–1635 的提前生成与 965–986 的 parent QC 检查 |
| Aptos two-chain vote 和 OrderVote | `safety_rules_2chain.rs` 53–116、190–213；`round_manager.rs` 1695 起的配置门槛 |
| Aptos 执行后 commit 声明 | `safety_rules.rs` 的 `guarded_sign_commit_vote`；与 ordering/round progression 分开 |
| Monad QC/TC 推进 | `monad-consensus/src/pacemaker.rs`，`process_certificate` |
| Monad current/next 双路收票 | `monad-consensus-state/src/lib.rs`，`send_vote_and_reset_timer` 1208–1264 |
| Monad 单调投票和 no-endorsement 约束 | `monad-consensus/src/validation/safety.rs` |
| Monad commit 候选与 block-tree 连贯性 | `monad-blocktree/src/blocktree.rs`，`get_high_committable_qc` |

学习笔记中的外链均使用上述固定 commit。部分大文件包含测试代码；本文只引用指定函数，不把源码存在某个分支等同于所有部署已启用该分支。

## 官方资料与论文

- [Flow Jolteon 工程说明](https://flow.com/engineering-blogs/jolteon-advancing-flows-consensus-algorithm)：正常流水线、active pacemaker 与 two-chain finalization 的背景。
- [Aptos Baby Raptr 生产升级公告](https://aptosnetwork.com/currents/baby-raptr-lands-on-mainnet)，2025 年 6 月 20 日：Jolteon*、Quorum Store 与投票前数据获取。公告日期不是本笔记日期，公告也不证明所有后续 Raptr 方案已部署。
- [MonadBFT 官方文档](https://docs.monad.xyz/monad-arch/consensus/monad-bft)：normal path、block 状态和 recovery 的分层解释。
- [Monad staking](https://docs.monad.xyz/monad-arch/consensus/staking)：staking weights、epoch leader schedule、blocks 与 rounds 的区别。笔记没有依赖易变的出块间隔、staking 门槛或奖励数值。
- [Category Labs fast-recovery 更新](https://www.category.xyz/blogs/monadbft-update-fast-recovery-leader-fault-isolation)，2025 年 11 月 21 日：backup votes/QC、timeout vote reconstruction，以及仍然保留的复杂恢复路径。
- [MonadBFT 论文 v4](https://arxiv.org/html/2502.20692v4)，2026 年 7 月 21 日：模型、full/speculative finality、tail-forking 与完整恢复规则。

## 验证范围

笔记是机制解释，不是三条链的独立安全审计。四节点、连续轮次和消息延迟计算均是教学例子。两个 Mermaid sequence diagrams 表示角色与因果顺序，不表示已测量的网络时延。正文明确区分标准路径、可配置优化、客户端最终结果和当前稿件的拟议迁移。

检索 GitHub API 时遇到公共限流，随后用公开 Git branch references 取得 commit，并从固定 raw URLs 保存文件。内容来自公开官方仓库，未借用第三方解释替代关键源码检查。
