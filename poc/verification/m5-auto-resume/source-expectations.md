# M5 独立来源预期

来源基线为 `e44b6e8e77aeb1aa3895c14109a57ff3894d7252`。以下预期在读取 M5 实现前推导。依据为 M1–M5 计划 §6，PoC v2 §8.5–8.6，v10 Architecture §2.3、§3.2、§3.4，Authority §3.10，GFR §3.2–3.5、§4.2–4.6、§5.4–5.5 和 Manager §3.10、§4.2、§4.4。Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` 映射 Chan–Pass Simplex 至 Commonware `2026.10.0`／release `92a2f68`。P0 采用 3×5 等权副本，区域 Final 需要同一 subject/context 四个有效签名者。治理门槛来自 M3/M4 的实际部署，为 2/3。

## 从排序停滞到恢复资格

- 停止 finalized 时，排序头的 q 与 θ 均停在最后验证位置。水位冻结时，完整排序日志可继续产生新的 q，θ 保持相同。两者都可独立于 L1 finalized、已验证 DA 上的 D 派生以及 Clock RPC 交付。用实际各服务记录验证四条时间线。
- 每个旧 cid 的本地 stop 为 `ts*(h_max+1)`。以该 worker 的协议时钟检查终止与签名门槛，保留此前形成的签名和 Final，核对终止持久化及之后的 Grant/Ack。声称时钟前提成立时，正常边界为虚拟 `stop±δ_G`，并分别校验采样时钟偏差和 RPC 交付时延。迟来的 Control、Timer 或样本都不能让已 Terminated 的 cid 再签名。用实际 raw Ack/输出辨别新签名和旧结果转发。
- 全局 `Expire(h_G+1)` 和 `SessionEnd(s)` 的公开键从当前 G/Tail 计算。Espresso θ 未覆盖或 L1 控制前缀及材料未覆盖时，GFR 返回 `WAIT{next_position, missing}`，保留已应用位置与 G。仅增加 q 和推进虚拟、墙钟或本地 stop 均不足以清除 WAIT。冻结 θ 分支若旧水位等于某个事件键，也仍可处理已覆盖事件，验收看实际键比较。恢复后按公开全序重放，控制先于同键 Expire，Expire 先于 SessionEnd，后者先于 Espresso。验证当时的 G 下一高度仍然缺失，并据此判断是否公开 HALT。
- 对给定视图的所有共同高度，比较从实际执行结果得到的 native hash。任一经过验证的不等给出 `HASH_DIVERGENCE`。高度不同但共同前缀一致，单独不能给出该理由。又一恢复理由 `HALTED_D_ADVANCED` 需要该视图已经重放公开 HALT，且 D 按原生规则严格越过冻结 G。普通落后、D 较高而 G 仍开放且共同前缀一致、已 HALT 但 D 尚未越过、未读齐或未验证共同前缀，均不给出恢复资格。
- 每个 signer 独立消费实际交付的 verified L1、Espresso、D/DA 与 GFR 状态。一次恢复需有该 signer 已验证的前后两份有序视图，看到 q 与 θ 均严格推进且水位满足合同的最近性和覆盖条件。可信检测依赖交付、验证和分别保存的历史。恢复服务与目标区域资料齐全、没有未决请求、路径与时限可行也要在自身视图确认。签名者须对同一请求域和具体 target、policy、控制序号等内容达门槛。短停后恢复且无上述事由仍为零提案、零签名、零请求。材料不足和读者滞后在实际依据可用前明确 WAIT。
- `t_eligible` 为足够独立 signer 首次同时可以验证同一可提交请求的虚拟时刻，须从实际交付和验证迹线复算。实测 `t_submit-t_eligible ≤ T_poll+T_sign+T_broadcast`，同时核对请求准备、签名、原始交易、持久化、广播和待决状态各自的因果位置。重复观察和重复驱动只允许已有请求的同一内容或交易对账，不能越过真实 M3 2/3 授权。

## 高度与收录预算

公开时钟仿射映射取配置中的 `ts*(h) = ts_0 + (h-h_0)b_L2`。采用完整整数、受检运算。按各 signer 或其经验证的公共请求依据，分别保存当前 G、D、Inbox 头以及直到最迟收录前允许增长的上界。一个有效的 H_guard 至少为三路未来收录上界的最大值。增长界需要覆盖已有在途批次和允许交付的积压、恢复后的 G 追赶和 D 在完整窗口闭合后的原生增长。速率界和收录区间需在签名时可证，与实际 L1 inclusion 的事后观测分开。公开 HALT 且无其他待决控制时该旧 G 可取零增长，D/Inbox 的界仍须各自验证。

给定 `t_L,min..t_L,max`，设 `lo=t_L,max+T_prep`、`hi=t_L,min+T_drift−ε`。校验 `T_prep>=M_a`、`M_a+T_ready+δ_G<=T_prep+b_L2`，再取定义域内的 `h*=max(H_guard+1, h_0+ceil((lo-ts_0)/b_L2))` 并验证 `ts*(h*)<=hi`。预算下界使用最迟收录，上界使用最早收录。等号可行，两个相邻 slot 跨界时拒绝。H_guard 是严格界，等于 H_guard 的候选必须排除。无解时无需签名或播出，以新验证输入重算。验证候选最小性可检查 h*−1 跌出有效高度域、≤H_guard 或早于 lo。对正例收录前的真实各头和积压复核事前增长界，并检查实际收录落区间内。将跨区间端点纳入用例。晚收录保留原始真实收据，事后时间不能反填进先前签名。Signed cursor origin 也须由求解前已验证约束或确定的安全公式选取，且不超过所有允许的实际收录块。实际派生目标 origin 独立从 D 取得。

## 从 Anvil 到接纳新块

- 同一成功治理交易授权 2/3，写入相互关联的 CursorSet 与 ResumeRecorded，核对原始已签字节、domain、nonce/seq、policy 和 h*，保留实际区块时间和精确 L1 三元组。CursorSet 的 Inbox monotonic guard 与 GFR act 不同。若实际收录迟到，事前治理预算失败，不能计为正常时限成功。成功链上记录仍按实际已 finalized 时间键 `act=ts_L1(L)+M_a` 进入 GFR。
- 用 09/11/12 的真实收据及验证 DA 独立重放 D，恢复读取对应正好这一笔 CursorSet 的历史切面。允许 D 后来超过目标且不回退。缺输入、切面和历史 Plan 时整步等待，补齐后按同一位置原子保留共同 G 原生前缀、替换其余，用该切面精确到 h*。新 `(β+1,0)`，从 policy 首个合格区域取 Plan，anchor 是 D(h*) 的实际派生标识、原点和执行状态，窗口 `[h*+1,h*+10]`。同一事件批次按 `BranchClosed → Resumed → SessionAssigned` 消费。若新 Expire 或新 SessionEnd 公开键严格小于 act，则完成对齐、公开 HALT 且不分配可启动角色。相同键处理全部 L1 控制后再适用定时器。
- 以真实虚拟 tick 记录对应 cut、G 对齐、每个新 worker 的 Ready/Enter、首次提议、BLS Final、首次完整公开和 GAccepted。正例至少四个同 cid、持续可调度、有所需数据的活跃副本在 `ts*(h*+1)-δ_G` 前就绪，且 `t_quorum-ready-act<=T_ready`。提议遵守本地 NotBefore。恢复终点是新物理 h*+1 实际执行并经 GFR 接纳。按实际公开键核对 Expire 和 SessionEnd，按虚拟 tick 核对冻结的链路、执行、发布和 GFR 时限预算。
- 主正例、q 继续但 θ 冻结、短停、普通 lag、真共同高度分歧、输入缺失、无解、迟收录、时钟 ±1 ms 和独立 RPC 延迟应以冻结合同规定的实际运行或明确声明的反例验证。每个范围报告触发、输入、验证归属、作用和截止。验收依赖相同候选 SHA 的 raw 15-worker 证据、计划的同 SHA M1–M4 回归和一把宿主构建锁。

可行性核算示例仅供合同冻结。若既有 L2 genesis=10 s、slot=1 s、h1 已采用，旧 h10 的 local stop=21 s、SessionEnd=24 s。选 W_exp=12 s 可使 h2 的 Expire 在 24 s 与 SessionEnd 同键，公开恢复时先 HALT。若预计收录为 [28,30] s、T_prep=12 s、T_drift=15 s、ε=1 s，预算窗口退化至单个 `ts*(h)=42s`，对应 h32。H_guard=31 时可行，H_guard=32 时无解。具体场景数值由冻结合同与独立实际证据校准。
