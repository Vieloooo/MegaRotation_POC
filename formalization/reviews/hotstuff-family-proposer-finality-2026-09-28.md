# HotStuff family：以外部 proposer P 的不可回滚确认延迟为目标

日期：2026-09-28。用户明确选择的目标：**外部 proposer P 收到可验证、不可回滚的确认**。本文是文献比较及设计推导，不修改任何协议。目标源码仍为 `main_v1.tex`；本文不把已有 `main_PBFT.tex` 当成其替代入口。

## 1. 必须先固定确认的含义

当前 [main_v1.tex](../main_v1.tex) 230–248 行区分：

- `Preview`：包含 regional decision certificate；同一 global view 内不因 local view change 回滚。
- `Deliver`：额外包含覆盖该范围的 globally finalized checkpoint；跨 geographic changes 仍是 append-only。
- `Preview` 后缀可以在 global view change，包括同区域重新激活时，被替换。

因此，“区域 BFT finality”与“整个系统对 P 的永久 finality”不同。下面首先比较区域最终证明的产生和传输。要把它提升为跨区域也不可回滚的确认，必须让 global handoff/activation 保留所有获此保证的区域结果，或继续等待当前的 global finality。只换 regional BFT 不会自动改变这个接口。

## 2. 延迟计量口径

- `δ_R`：正常路径内一次区域消息传输及所需处理的延迟。
- `Δ_R`：配置的区域同步延迟上界；通常不能当成实际 `δ_R`。
- `δ_P`：成员/collector 向 P 传输证据并验证完成的延迟。
- 起点：完整 proposal 已经准备好，leader 开始发送。
- 已在正确 view、前缀与配置已知，相关 leader 正常工作；不计交易提交、batch accumulation、排队、超时、执行和全球 finality。
- 若 block/body 与 vote 大小差异显著，必须分别测量其延迟。下面相同 `δ_R` 的公式是消息路径模型，不是生产环境的实测承诺。

区分三个指标：区域节点的 local commit、P 的 verified finality、连续 proposal 的 block period。P 若直接收取最终投票，可能比普通成员更早持有完整决定证明。

## 3. 主要文献与改进

| 协议 | 核心改变 | 正常路径的含义 | 对本项目的意义 |
|---|---|---|---|
| HotStuff / Chained HotStuff | 三阶段，QC＋lock，线性 leader handover；链式流水线 | 常见 next-leader collector 时序下，区域节点约 `7δ_R` commit | 清晰的通信基线，非最少延迟方案 |
| Fast-HotStuff | 两阶段／two-chain；恢复时携带聚合的新 view 信息 AggQC | 区域节点约 `5δ_R` | 降低阶段数；AggQC 中的多个 QC 描述不能因聚合签名而当成常数数据 |
| Jolteon / DiemBFT v4 | two-chain，加 TC 驱动的 pacemaker 和恢复证据 | 区域节点约 `5δ_R`；正常线性、恢复有二次通信 | 有公开工程实现的候选 |
| Ditto | 在 Jolteon 基础上增加随机化 asynchronous fallback | 改善恶劣网络下的活性 | 重点是故障路径，不继续缩短正常确认路径 |
| HotStuff-2 | 两阶段，同时设计线性 handover | 区域节点约 `5δ_R`；失败后可能等待 `Δ_R` 级时长 | 很适合作为线性通信的区域基线 |
| HotStuff-1 | 客户端直接收取足够的 commit votes／speculative responses；slotting | 客户端取得充分证据时有真正的安全确认；单个 speculative response 不足够 | 与外部 P 目标最直接相关；需保持 prefix speculation 条件 |
| MonadBFT | two-chain、tail-fork-resistant recovery、backup QC | `3δ_R` 的一轮 QC 路径是可撤销 speculative commit；与 full finality 分开 | 故障恢复值得借鉴；不能把 speculative QC 用作 P 的永久证明 |
| Sync HotStuff | 利用同步时间窗口，以计时器控制 commit | 基本模式投票后等待 `2Δ_R`；从 proposal 发送起约 `δ_R+2Δ_R` | 不能与本稿 `2δ_R` 的快路径混淆 |

上表的短值线性 word bounds 依赖论文的紧凑认证原语与计费方式。普通 aggregate signature 加 signer bitmap、消息体和执行数据都应单列。经典论文通常按 faulty identity 数计故障；本稿按权重计，需要核对 quorum、采样、threshold setup 和恢复规则的适配。

### HotStuff → Fast-HotStuff / Jolteon

[HotStuff](https://arxiv.org/abs/1803.05069) 利用三阶段保证：若某个节点已经锁定，足够多诚实节点持有支撑该锁的 prepareQC，新 leader 收到 quorum 状态即可恢复安全进度。

[Fast-HotStuff](https://arxiv.org/abs/2010.11454) 与 [Jolteon / Ditto](https://arxiv.org/abs/2106.10362) 都把正常流程降为两阶段，但不能只删除一阶段。它们增加/调整 leader 交接证据，使恢复时的提案选择可以被验证。Jolteon 的 TC 携带 quorum 的 QC 状态；Fast-HotStuff 的 AggQC 也包含 QC 描述集合。签名聚合不会自动消除这些描述的传输成本。

Jolteon 的工程参考：[Flow consensus/hotstuff README](https://github.com/onflow/flow-go/blob/master/consensus/hotstuff/README.md)。这里引用的是实际实现来源，不以实验室论文结果冒充部署测量。

### HotStuff-2：保留两阶段，改善 handover

[HotStuff-2](https://eprint.iacr.org/2023/397) 的要点：若新 leader 获得紧邻前一 view 的 QC，已经知道不可能有更高的旧锁，可以立即推进；若没有这种证据，则在有相应 view synchronization 保证时等待网络上界以获得必要状态。它保留正常路径的实际网络速度和线性 handover，但不能把失败恢复中的等待也写成完全不依赖 `Δ_R`。

作者解释：[HotStuff-2 与前代的差异](https://decentralizedthoughts.github.io/2023-04-01-hotstuff-2/)、[Leader Handover Rule](https://malkhi.com/posts/2025/01/handover/)。两阶段不是两个单程延迟；proposal、vote、QC fan-out、下一轮 vote、最终证书 fan-out 仍是五跳。

### HotStuff-1：改变谁先知道已经决定

[HotStuff-1，SIGMOD 2025](https://arxiv.org/pdf/2408.04728)，§3 与 Figure 1：成员获得 prepare certificate 后，可在发送 commit vote 的同时向客户端发送 speculative execution response。客户端必须收集 `n-f` 份匹配回复，而不是普通执行后回复常用的 `f+1` 份。满足协议规则的完整 quorum 可使客户端确定最终性，即便部分副本还没有获知 commit certificate。

这与“先相信一张普通 QC，以后若作恶就回滚”不同。论文专门处理 prefix speculation dilemma：不能因为后续块可以投票，就无条件提前确认所有尚未决定的祖先。详见 [作者教程](https://decentralizedthoughts.github.io/2024-08-24-hotstuff1/)。

对本项目，P 关心 ordering/contents 的确认，不必要求 sequencer 执行 EVM。可以借鉴 external learner 的证据收集方式，而不照搬 speculative execution 业务。具体到有父结果依赖的 stream，仍需明确 vote 的 instance/parent/phase 绑定和前缀确认条件。

### MonadBFT：恢复与 speculative confirmation

[MonadBFT 作者稿 v4](https://arxiv.org/html/2502.20692v4) 使用 local tips、TC、reproposal 和必要时的 NEC，防止后继 leader 任意丢掉前任已经获得支持的尾块；backup votes / QC 还用于减少一个 leader 故障引起的连续 timeout。

其 `3δ` speculative commit 是 proposal→vote collector→QC dissemination。原 leader equivocation 时仍允许撤销。因此论文将 speculative 与 full finality 区分。这个 speculative latency 不满足本任务的不可回滚目标；罚没也不能把条件保证变成完整安全保证。它的公开实现采用 stake weights，但仍需单独核对任何移植中的权重与证据规则。

## 4. 用 P 的观察点重新计算

以下是**依据消息路径得到的设计推导**，不是声称所有论文/现有实现已经向 P 导出这些消息。需 digital signatures / threshold shares 可由 P 验证；已有完整 proposal body、配置与前缀；不把普通 QC 当作 final proof。

| 确认方式 | 最终确认票/证书何时产生 | P 获得区域不可回滚证明的目标延迟 |
|---|---|---|
| 三阶段 Basic HotStuff，collector 聚合最终 QC 并直接发 P | 最后一轮票在 `6δ_R` 到 collector | `6δ_R+δ_P` |
| 两阶段 HotStuff，collector 聚合最终 QC 并直接发 P | 最后一轮票在 `4δ_R` 到 collector | `4δ_R+δ_P` |
| 两阶段 HotStuff 的 external learner / HotStuff-1 思路 | 成员在 `3δ_R` 获得 prepareQC 并发 commit votes | `3δ_R+δ_P` |
| signed PBFT，成员将 COMMIT 同步发送给 P | 成员在 `2δ_R` 获得 prepared quorum 并发 COMMIT | `2δ_R+δ_P` |
| 当前同步 `2δ_R` 快路径，将 DECIDED 直接发送给 P | 正常成员在 `2δ_R` decide 并签 DECIDED | `2δ_R+δ_P` |

两阶段 HotStuff 的直接学习者路径：

1. `0`：leader 发 proposal。
2. `δ_R`：成员收到 proposal，回第一轮票。
3. `2δ_R`：collector 形成 prepareQC 并发送。
4. `3δ_R`：成员收到 prepareQC，按协议产生 commit vote；同时将对应证明发给 P。
5. `3δ_R+δ_P`：P 获得 quorum，验证精确的 block/instance/view/phase/parent 与必要证据。

P 只是 learner，不参与 quorum、leader election 或 view change；P 离线不应影响区域共识推进。

PBFT 的优势来自 PREPARE all-to-all：每个成员在第二跳就能获得 prepared quorum，无需先经 collector 聚合再送回。代价是区域 `O(n²)` 控制通信。上述 P 接口是对签名版 PBFT 的导出设计，不能直接把原始 MAC 优化版的内部消息当成公开可转移证明。[PBFT 算法](https://static.usenix.org/publications/library/proceedings/osdi99/full_papers/castro/castro_html/node4.html) 及其 [tentative reply 优化](https://www.usenix.org/legacy/events/osdi99/full_papers/castro/castro_html/node5.html) 提供相关依据。

当前 main_v1 的 R5 仅在区域内 multicast DECIDED，R6 再导出证书。稿件给出保守 `4δ_R+δ_P` preview 证据路径。直接把 R5 的签名也发给 P 是一个可单独研究的小改动：P 用原有 `>2W/3` DECIDED 验证规则聚合。它降低 proof export latency，却不消除 mandatory completion 或下一实例的 `(n_i+5)Δ_R` 等待。本笔记未实施该改动。

## 5. 若目标是最少延迟，应加入非 HotStuff 基线

[Moonshot，DSN 2024 作者版本](https://arxiv.org/pdf/2401.01791) 通过 vote multicast 与 optimistic proposals，把部分同步、旋转 leader 的最小区域 commit latency 降至 `3δ_R`，proposal period 可达 `δ_R`，代价是二次通信。它说明 collector 的线性通信与最少延迟之间存在重要工程权衡。Moonshot 有多个变体，应按具体 commit 证明设计 P 的导出路径，不能从 `3δ_R` 自动推导一个外部 finality 数字。

[Good-case Latency of Byzantine Broadcast](https://arxiv.org/pdf/2102.07240v3)，Table 1：同步 authenticated BB、`0<f<n/3` 的内部 good-case bound 为 `2δ`；partial synchrony 的 Psync-BB 在 `3f+1≤n<5f−1` 范围需要三轮，在 `n≥5f−1` 可以两轮。这是该论文任务和故障模型下的界，不是对每种外部 learner 或任意加权配置的统一定理。

本稿的同步 `2δ_R` 内部决定已经达到对应广播模型的界。换成 HotStuff family 的意义应是获得不同的安全/恢复模型、连续流水线和通信量；不能期待常规 two-chain 自动把 `2δ_R` 再降下来。

[Sync HotStuff](https://dahliamalkhi.wordpress.com/wp-content/uploads/2019/11/synchotstuff-sp2020.pdf) 的同步分支在 vote 后等待 `2Δ_R`，从 proposal 起计为 `δ_R+2Δ_R`，并非本稿的 `2δ_R`。其其他响应模式还有额外前提，不能混用常数。

## 6. 当前建议

1. **如果保留当前同步区域模型：** 先评估 P 直接收取 DECIDED quorum。比较基线应是 `2δ_R+δ_P` 的目标证据路径，并独立优化 completion 和持续出块间隔。
2. **如果要求临时超出区域延迟上界仍然保证安全：** 比较 signed PBFT 直接 COMMIT-to-P 与 HotStuff-2 / HotStuff-1 式 external learner。前者以更多区域通信换少一次串行传播；后者适合更重视线性控制通信的委员会。
3. **如果特别重视每块轮换 leader 与故障后的尾块保留：** 再比较 Jolteon、Moonshot、MonadBFT 的完整恢复协议，不将 speculative finality 计入不可回滚指标。
4. **全系统不可回滚：** 固定跨区域 certificate inheritance 规则或继续计算 global finality 路径。区域 BFT 优化只是总目标的一部分。

最终应分别测量：从完整 proposal 到 P 持有证明的时间；P 获得完整 body 的时间；批处理等待；持续实例间隔；失败 leader 后的恢复时间。正文所述只是一组有明确端点的理论比较，不是生产性能排名或完整新协议证明。
