# MegaETH 和 OP Stack 的 L1 强制包含及 Espresso 停机处理

**核对日期：2026 年 10 月 24 日。**
MegaETH 强制包含部分依据现行官方文档；Espresso 部分依据 [specification PR #8][PR] 的 10 月 23 日版本 `a738f9b`。
该版本仍明确标注：**Block Certification 尚未在任何 MegaETH 网络激活**。[集成总览][E-overview]

先记住三个结论：

- **MegaETH：**排序器不处理你的交易时，可以改从 Ethereum L1 投递，让协议把它纳入 L2。
- **OP Stack：**排序器不交有效批次，超过规定窗口后，节点按 L1 输入生成只含 deposit 类交易的区块。这是强制包含能够抵抗排序器停机的关键。
- **Espresso 集成草案：**Espresso 停止确认新记录时，没有继续提交新普通交易的绕行模式；保留的是上述 L1 投递和 deposit-only 兜底。

## 1 MegaETH 的 L1 强制包含怎么走

正常路径是“用户 → MegaETH 排序器 → L2”。
**强制包含换了入口：用户 → Ethereum 上的 Portal 合约 → 节点从 L1 派生交易 → L2。**

具体过程是：

1. 用户在 Ethereum 调用 MegaETH 的 `OptimismPortal.depositTransaction`，写明 L2 目标合约、调用内容和 gas 上限。
2. 这笔 L1 交易成功后，Portal 发出 `TransactionDeposited` 事件。
3. 节点读取事件，把它转换成一笔 L2 的特殊交易，类型为 `0x7E`。
4. 这笔交易属于协议要求纳入的输入，排序器不能永久忽略它。[MegaETH 开发说明][M-dev]

这里的 **deposit 不只表示充值，而是“从 L1 投递、在 L2 执行的交易”**。
例如，你在 MegaETH 已经有某种代币，可以从 L1 投递对该代币合约的 `transfer` 调用，不必再桥入一笔代币。
普通钱包账户使用同一地址作为 L2 调用者；合约钱包还要考虑地址别名规则。[MegaETH 开发说明][M-dev] [OP Deposits][OP-deposits]

MegaETH 官方用户文档给出的时间是：**Ethereum 上提交后，12 小时内包含到 L2；正常情况约 5–20 分钟。**
这是包含时间，不是提现到账时间；合约调用仍可能因 gas 不足、余额或业务条件不满足而失败。[MegaETH 用户说明][M-user]

## 2 OP Stack 为什么在排序器离线时也能包含

关键在 **derivation，中文可理解为“按 L1 资料重建 L2 链的规则”**。
L1 上既有用户投递的 deposits，也有 batcher 提交的普通交易批次；所有节点依照相同规则读取和执行这些输入。[OP 派生规范][OP-derivation]

协议给普通批次一个提交期限，叫 **sequencing window**。
如果窗口结束后仍没有可用批次，节点就生成 **deposit-only block**：保留系统交易和按规则必须包含的 L1 deposits，普通交易列表为空。[OP 派生规范][OP-derivation]

**这些区块由各节点依照相同 L1 历史独立构造和执行，不需要先选一个新排序器，也不需要对交易顺序另做一次共识。**
所以，“排序器离线”不必等于“协议永远无法继续”；只是原来的高速普通交易服务没有因此恢复。[OP 总览][OP-overview]

OP Mainnet 当前配置是 **3600 个 L1 区块**，按 Ethereum 每个 slot 12 秒估算约为 **12 小时**。
它是按 L1 区块计算的窗口，不是所有 OP Stack 链统一的墙钟承诺；其他链要看自己的配置。[OP Mainnet 配置][OP-config]

另外，已经发给普通 L2 RPC、却没被纳入的交易，不会自动变成 deposit。
用户需要走 L1 投递路径，重新表达要执行的调用。

## 3 Escape hatch 怎样帮助用户退出

这里可以把 escape hatch 理解为：**即使排序器不配合，用户仍有一条从 L1 发起 L2 操作的应急通道。**

如果要提现，可以通过这条通道发起相应的 L2 提现调用。
之后仍须走完整的提现流程：

**L1 投递 → L2 执行提现调用 → 形成提现消息 → 在 L1 提交证明 → 满足等待和争议处理条件 → L1 支付。**

因此，常见的“约 12 小时强制包含”和“约 7 天提现等待”回答的是两个问题：前者管交易进入 L2，后者管提现何时可在 L1 完成。
实际提现条件取决于该链的证明系统、参数和暂停状态，不能把 OP 的等待时间直接套给 MegaETH。[OP 提现流程][OP-withdrawals] [当前 OP Portal 规范][OP-portal]

## 4 Espresso 停机时 MegaETH 草案怎么处理

### 4.1 普通交易多了一道提交门槛

集成设计要求先把普通交易列表发布到 Espresso，等待其确认，再由 certifier 检查后续批次与选中的记录一致；批次还需要 EigenDA 数据可用性证据，才能被 L1 inbox 接受。
其中 **publisher 发布记录，batcher 提交批次，certifier 核对内容**，三个职责可以由同一运营者承担。[集成总览][E-overview]

**L1 deposits 和强制包含交易不在 Espresso 的普通交易记录里；它们由每个节点直接从 L1 派生。**
这就是 L1 应急通道能够保留下来的原因。[集成总览][E-overview]

### 4.2 Espresso 不再确认新区块后会发生什么

草案的 **S9 Espresso Liveness Failure** 明确写明：没有适用的恢复通道，可以让需要新 Espresso 确认的普通内容继续提交。[Recovery S9][E-S9]

| 对象 | 停机后的处理 |
| --- | --- |
| 已被 Espresso 确认的完整积压记录 | 数据、证明仍可取得，并满足提交期限和其他检查时，可以继续提交 |
| 仍需要 Espresso 新确认的普通交易 | 认证提交停住，等待 Espresso 恢复 |
| 一直没有可用批次的高度 | 超过 sequencing window 后，按规则生成 deposit-only 区块 |
| 从 L1 投递的交易 | 继续适用 L1 派生规则，不要求先写进 Espresso 记录 |

所以，**Espresso 停机可能让新的普通交易无法完成认证提交，同时链仍以 deposit-only 方式推进。**
排序器即使还在提供实时输出，也不代表那些普通交易最终会保留。[Recovery S9][E-S9]

这里有一个重要条件：节点必须能取得并处理必要的 L1 历史和此前已接受的数据。
**如果 L1 inbox 已接受的 EigenDA payload 取不回来，草案要求节点停在那里等待，不能跳过它再用 deposit-only 填空。**
“批次一直没来”和“已接受批次的数据丢了”是两种不同故障。[数据取回规则][E-DA]

### 4.3 换 batcher 或管理员介入能救什么

- **替代 batcher：**原 batcher 不提交时，登记过的替代者可在独占窗口结束、必要的 channel 超时后接手。它仍需要 Espresso 已确认的记录，所以不能修复 Espresso 停机。
- **管理员 override：**可以向前移动提交进度 cursor，放弃尚未落实的部分，并用 deposit-only 填补相应缺口；它不能制造新普通交易的认证，也没有建立持续绕过 Espresso 的模式。[恢复机制][E-hatches]

### 4.4 Espresso 恢复后能补回原来的交易吗

如果还没错过提交期限，记录和数据仍有效，原内容可能保住。
如果某高度已经最终成为 deposit-only 区块，就不能把原普通交易补回那个高度。
恢复时，排序器要跟随实际派生出的链和状态；后续积压记录逐个检查，有效的保留，需要替代的变成 deposit-only。
丢失的普通交易需要在后续重新提交，重新提交也不等于恢复原来的执行结果。[Recovery S4][E-S4]

## 5 用一个例子串起来

假设 Espresso 停机前已经确认了高度 100 的记录，101 的记录还没确认。

1. 100 的数据和证明仍可取得，且没有超期，batcher 可以继续提交它。
2. 101 的普通交易不能靠更换 batcher 绕过 Espresso；若一直没有可用批次，超时后该高度成为 deposit-only。
3. 你此时从 L1 投递一笔代币转账，仍可按 L1 派生规则进入后续 L2 区块，前提是节点没有被此前不可取得的数据卡住。
4. Espresso 恢复后，系统从实际派生状态继续；不能再把已丢弃的 101 普通交易列表塞回原高度。

## 原始资料

- [MegaETH 强制包含用户说明][M-user]、[开发说明][M-dev]：实际入口、时间和调用方式。
- [OP Deposits][OP-deposits]、[派生规范][OP-derivation]、[Mainnet 配置][OP-config]：为什么强制包含能抵抗排序器停机。
- [MegaETH Espresso 集成总览][E-overview]、[Recovery S9][E-S9]、[Recovery S4][E-S4]：Espresso 停机及恢复后的处理。

[M-user]: https://docs.megaeth.com/user-guide/force-inclusion
[M-dev]: https://docs.megaeth.com/developer-docs/overview-1/force-inclusion
[OP-deposits]: https://specs.optimism.io/protocol/deposits.html
[OP-derivation]: https://specs.optimism.io/protocol/derivation.html#batch-queue
[OP-overview]: https://specs.optimism.io/protocol/overview.html#epochs-and-the-sequencing-window
[OP-config]: https://github.com/ethereum-optimism/superchain-registry/blob/main/superchain/configs/mainnet/op.toml
[OP-withdrawals]: https://specs.optimism.io/protocol/withdrawals.html
[OP-portal]: https://specs.optimism.io/fault-proof/stage-one/optimism-portal.html#finalized-withdrawal
[PR]: https://github.com/megaeth-labs/specification/pull/8
[E-overview]: https://github.com/megaeth-labs/specification/blob/a738f9b14c7880953d8cff8788db14e8e3e79124/specs/block-certification/overview.md
[E-S9]: https://github.com/megaeth-labs/specification/blob/a738f9b14c7880953d8cff8788db14e8e3e79124/specs/block-certification/recovery.md#s9-espresso-liveness-failure
[E-S4]: https://github.com/megaeth-labs/specification/blob/a738f9b14c7880953d8cff8788db14e8e3e79124/specs/block-certification/recovery.md#s4-a-height-settles-as-a-deposit-only-block
[E-hatches]: https://github.com/megaeth-labs/specification/blob/a738f9b14c7880953d8cff8788db14e8e3e79124/specs/block-certification/recovery.md#the-three-escape-hatches
[E-DA]: https://github.com/megaeth-labs/specification/blob/a738f9b14c7880953d8cff8788db14e8e3e79124/specs/rollup/data-availability.md#batch-commitment-resolution
