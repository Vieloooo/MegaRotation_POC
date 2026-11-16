# M3 自动 Force 合同 v1

合同标识 `M3-AUTO-FORCE-1`，指定 worktree `/home/jy/.codex/worktrees/poc-m3-auto-force/megaroation_poc`，分支 `codex/poc-m3-auto-force`，基线 `ea4d853bd8b62ee04c4e8e470550648242073950`。Lead 将本文件及独立预期提交并记录 hash 后，Builder 才开始实现。M2 的提交及证据继续作为回归依据。候选交给主 chat 集成。

## 来源与 PoC 配置

| 来源 | 固定规则 |
| --- | --- |
| v10 Authority §3.1–3.2、3.9、3.11–3.14 | 控制序号及请求去重，固定分支 policy，真实收录位置与 act，历史 Plan 的读取高度和循环选区 |
| v10 GFR §3.2–3.5、4.2–4.5 | 公共全序与覆盖，原子 WAIT，实际激活前 anchor，Force 序号/窗口与关闭 |
| v10 Session Program §3.4–3.12、Session Manager §3.7–3.10 | 独立准备/Enter、签名门禁、永久 terminal、保留已形成结果 |
| PoC v2 §8.1–8.4、§12 | 只用自身公共视图的停滞检测、在途状态、健康时钟、恢复时间预算 |
| M1–M5 计划 §4、当前 M3 派发 | 三名治理 signer 默认 2/3，通过 L1 policy 跳过禁用区，五个当前区域 worker 离线，M2 回归 |

三地区各五个等权 replica，区域 Simplex Final 要求四个不同合法 BLS signer。Commonware `2026.10.0`、Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`、Rust 1.95.0，治理 EIP-712/secp256k1 身份与区域 BLS、Certifier、DA 身份分别配置。治理 threshold、成员表与规则版本绑定本次 config。M3 的多签和历史区域 policy 是本阶段明确批准的 PoC 扩展。委员会成员及等权快照继续使用公开 registry，staking/ban 集合采用固定空禁任的显式 mock。每个地区的五名身份均保持注册资格，治理通过 policy 的 enabledMask 改变地区选择资格。

## 所有权与接口冻结

单 Builder 负责 `poc/contracts/src/` 和对应 contract tests、`poc/crates/authorization/`、`poc/crates/l1-adapter/`、`poc/crates/viewer-gfr/`、`poc/crates/session-manager/`、`poc/crates/publication-bridge/`、`poc/prototypes/i1b-runtime/src/`、新 `poc/crates/governance/`（若采用 Rust 包）及 `poc/scenarios/m3-auto-force/`，仅修改本合同必需的文件。明确新增路径含 `poc/contracts/src/CommitteeControlAuthority.sol`，治理状态机及独立 reader 可放 `poc/prototypes/i1b-runtime/src/m1_smoke/governance.rs` 和相邻 M3 模块，runner 放 `poc/scenarios/m3-auto-force/`。Lead 负责公共类型、Cargo 依赖与编码版本变更，并在动这些文件前批准精确接口。Verifier 独占 `poc/verification/m3-auto-force/`，Tester 独占其中 `tester/` 和正式运行目录。回放界面由其他 chat 负责。

M2 的入口继续选原单签 Authority、静态 schedule 与原 profile。M3 增加派生合约 `CommitteeControlAuthority`，复用原 `submitForce` 的 ABI 和事件，通过受限内部授权/policy hooks 实现多签，部署成员表与 threshold 作为固定配置。新增 policy Plan 模式和运行 profile。签名包规范冻结为多个 65-byte `r || s || v` 串接，整体长度为 `65*m`，`m = k`，按实际恢复的 signer 地址严格升序。地址必须是配置成员，无零地址、重复或降序。每个签名满足既有低 s、合法 r、v=27/28 规则。M3 默认 n=3/k=2，部署接受 `1<=k<=n` 的合法配置；成员表须非空、唯一、规范排序，签名规模按实际实现上界显式拒绝。

沿用既有 Force 规范 payload 的 caller、requestId、controlSeq、targetBranch、policyHash，以及 EIP-712 域的 chainId、verifyingContract、configRef。控制类型由 typeHash 绑定。合约对全部签名验证完成后查请求表。相同请求内容可以由另一组有效门槛签名重试，返回首次接纳结果而不新增控制日志。坏签名在缓存命中时仍拒绝，同键内容变化冲突，过期序号不消耗新请求。新合法请求序号严格超过已接纳最大值。

本合同冻结逻辑字段及签名包格式。Lead 已指定 additive Plan provenance 接口，保留 PlanRecord/ProgramConfig 编码及选中委员会内部的 round robin。M3 authorization_ref 使用新的 domain hash，绑定规范已激活 policy 的 mask/order、Force L1 position/sequence、effective floor，历史 ℓ_s header hash，原有 selection inputs/config/registry 及选中 region。active policy 持久于 replay/checkpoint，provenance 字节保存到原始 trace/witness，Plan 的实际 L1 支持前缀同时记录。下游只接纳 GFR 公开选中的已验证 descriptor。新增 M3 provider 或明确的结构验证入口可核对 registry/committee/CID，但新 policy 的授权必须由历史证据和 GFR 选择建立。具体类型/字节编码补充由 Lead 与 Builder 在公共类型修改前完成，沿用本节固定语义及域隔离。

## 治理 signer 的可观察输入

三名 signer 分别拥有自己的 finalized L1 reader、Espresso reader、GFR 状态和已处理位置。允许在同一宿主进程中作为三个独立状态机运行，每个 signer 只访问其被交付的公共对象、已验证状态和自己的时钟样本。每个签名决策保存输入前缀、GFR pc、Tail/cid/region、G head、进展键、观测键、时钟状态、policy 摘要和拒绝或签署理由。

场景故障脚本只交给调度器。治理停滞决策、mask 的选择与交易时间均由公共进度计算。主场景的 policy 提议为已生效 mask 清除当前被观测为停滞的负责 region，保留已生效循环顺序。每个 signer 独立核对该提议与自己的视图相符后，签署同一 payload。

每次轮询前，signer 从读取服务取得当前已公布 finalized tip 的引用，并分别验证前缀及 material 完整性。已知存在但未下载的历史、缺正文/证明或 GFR 仍 WAIT 时，signer 返回 `INPUTS_NOT_READY`。引用只能来自服务已公布的公共事实。reader 的陈旧前缀须经配置的新鲜度检查，不能以内部调度器的未来日志或故障状态证明最新性。测试必须能独立显示“已发布 tip”和“该 signer 已验证 tip”的差异。

对当前 `NEXT`，定义 `x_begin=max(ts*(window.h_min),assignment_key)`。已有该 session 对 G 的贡献时，`x_progress` 为最近一次合格追加的接纳键，否则为 x_begin。`x_obs` 是该 signer 已完整处理的 Espresso 键。无效签名、未选中 cid、重复旧结果和尚未完整公开的结果均不更新 x_progress。已公开而本地缺材料时停在原位。

只有下列条件同时成立才可签署自动 Force。

- 当前 Tail 为 NEXT，处于公开职责生产期，branch 尚未关闭。
- signer 时钟可信，公共前缀、验证材料与自己的 replay 就绪。
- 当前轮询观察相对于上次观测有 finalized q 与水位推进，并且水位的新鲜度在声明的 E-TS/reader 预算内。只有 q 增长或全部停滞均不给出地区停滞判定。
- `x_obs-x_progress >= T_stall`，并有足够时间完成本次控制及新块接纳。
- 本地请求状态无未决控制，公共视图中没有应优先处理的 HALT/恢复状态。
- 发送前重新核对 branch、当前 Tail、最新合格进展和预算。

轮询差值严格小于门槛时保持观察，恰等于门槛时可满足该 guard。不同 signer 可以在不同 tick 达到条件，收集器只组合完全相同 digest 的签名。若他们观察到了不同 branch、policy 或进展事实，保留 WAIT/拒签原因，重新同步再形成提议。

## 请求持久性与受控输出

治理请求状态至少区分 Observing、Proposed、Signed、SubmittedUnknown、Included、Finalized、Applied/Noop/Failed 和 RecoveryObserved。写入请求内容、签名集合、caller nonce、原始交易及 hash 后，才允许向 L1 driver 释放提交。重复轮询及重载保持同一个 request/tx，广播结果未知时先查询或重发同一原始交易。一次只有一条在途控制。

明确失败或公共 GFR 已处理为生效/无效时，结束相应在途控制。已生效后恢复观察使用独立记录，首个新结果进入 G 后标记成功。读者重放同一公共输入、重复收到相同签名、重复广播均不可生成第二条控制。治理服务状态重载用专门向量核验，完整 BFT 进程恢复按既有 I1 合同处理。

所有 worker 输出继续经过 grant/Ack 和持久提交。治理轮询、签名完成、广播、L1 Mine/Finalize/Deliver、Espresso heartbeat、治理 reader 交付均由统一虚拟事件队列驱动并记录。墙钟仅用于进程 watchdog 和复跑暂停。工作、取时 RPC 延迟与时钟误差分别计量。

## Policy 与历史 Plan

M3 Policy 的 enabledMask 是三位 bitset，0..7 合法。region order 必须恰含 0、1、2 一次。零 mask 是合法无可选地区输入，Plan 返回 NONE，由 GFR 保留 PENDING。L1 收录仅记录控制，分支 policy 在有效 Force 被公共重放激活时才替换。stale target 或 HALT 的 Force 不修改生效 policy。

Force 取处理前实际 G head 的 BlockRef 和执行后状态为 anchor，beta 加一，当前 NEXT 有贡献则 r+1，无贡献沿用 r，PENDING 使用已保存边界。新 floor 按 v10 为 max(旧分支 floor,实际 inclusion height)，并保存签署 policy 中的 floor 字段供原始请求核对。新窗口按 `max(A.h+1,slot(act+T_init)+1)..+9` 得出。

Plan 使用实际分支已生效 policy，令 `l1_reference_height=max(Ref.origin,branch_floor)`，从该高度已验证历史取得 registry/资格 mock 的固定版本。来源区域存在时，从其在循环排列中的下一项开始逐项寻找第一个启用且具备合法 proposer 的地区，无来源时从首项开始。委员会、权重、eligible proposers、leader、window 和历史授权依据在 Plan 中冻结。

每个 Plan 的授权引用必须绑定 selection input、policy/生效控制位置、实际历史读取高度、registry/config 以及解析出的委员会。可由版本化 witness 加现有字段的可验证引用承载，但验证器必须从冻结历史重新计算全部字段。新的当前 policy、后来地区上线、最新 L1 tip 或 worker 在线数不得改变既有 Plan、cid 或 native certificate 的历史解释。缺 Ref 或 L1 历史返回 WAIT，无合格地区返回 NONE，格式/内容矛盾返回 REJECT。

## 主场景与时间预算

固定 seed42，15 个 OS worker，原 M2 的身份、时钟偏移 ±1ms、Clock RPC 单向250µs、链路4ms、显式工作5ms、Enter 间隔3ms。L2 genesis timestamp10s，slot1s，L1 slot2s，M_a10s、T_grace3s、T_init3s。M3 W_exp30s、T_stall2s，poll100ms，签名完成预算每 signer10ms，聚合/广播预算10ms。Espresso 独立 heartbeat 周期250ms，finalize 延迟1ms，读取者交付延迟依次0/10/20ms。正文发布使用已有有界链路，配置内容随原始 profile 封存。

主场景在虚拟11.250s、已确认 h1 合格接纳后，让当前 R0 的五个 worker 停止运行并保持离线至截点。这是预定故障输入，故障注入包含准确 pid、tick 和预期退出，存活的十个 worker 继续运行。期望 h1 为最终旧 G head。治理在各自验证视图中观察到停滞后，自动提出 mask6/order012/target beta0 的 Force，至少两名 signer 独立形成签名，经真实 Anvil 收录。实际 inclusion 与 act 由收据确定，不能在治理签署前写死交易时刻。

预期故障之后首次足够观测约在13.25–13.4s，正常控制可在 L1 timestamp14s 收录，act24s，h_min18、R1 (beta1,r1)。主场景必须赶上 14 s 槽。act=24 s 与旧 R0 的 SessionEnd 同键，Force 优先。若错过该槽，仍用真实 inclusion 和实际 Tail 重放，但按主场景时限失败报告。主验收从真实 receipt 和公共规则独立计算预期，要求实际 anchor=h1、正确 r=1、source R0、region R1，首个新 Final 经公开完整 publication 进入 G。

正常预算采用 `T_L1<=2s, Delta_ts<=1s, T_E<=1s, T_replay<=0.1s, T_state<=0.5s, T_produce<=0.5s,T_pack<=0.05s,T_pub<=0.2s`。收集器在首次 k 名 signer 独立可签的 t_eligible 后，应在 poll+T_sign+T_broadcast 内提交。按最近 slot 和实际发送进度求保守 inclusion 上界，再计算 a_max。

准备要求 `T_init >= Delta_ts+T_E+T_replay+T_state+delta_G`。首个新 session 至少四名固定成员在 `ts*(h_min)-delta_G` 前 Ready/Enter。提交时的当前缺失高度为 h2，Expire(h2)=42s。必须核对 `a_max+3+1+0.5+0.05+0.2+1+1 < 42`，并核对实际第一个新结果对全部追加 padding heights 的接纳键均严格早于其 Expire。结果只形成证明、只收录控制、只 Assign 均不构成主场景完成。

## 验收矩阵

| ID | 层级与刺激 | 独立预期 |
| --- | --- | --- |
| AF01 | 主进程场景，R0 五人故障且空 heartbeat 正常 | 由各 signer 自己视图达到门槛，真实2/3 Force，历史policy选R1，新原生 Final 及时进入G，故障PID有完整退出证据 |
| AF02 | 同SHA/seed，增加650ms墙钟暂停 | 规范化全虚拟轨迹、原生frames、L1交易收据逐字节一致，原始诊断保留 |
| AF03 | 健康自动治理场景 | 相同治理/heartbeat入口，正常R0至少持续到h10接棒；零Force签名/交易，普通G持续增长 |
| AF04 | signer读者落后或已公开material缺失 | 不足门槛的独立就绪签名人导致WAIT/不提交；补齐后按当前视图重新判断。单个落后signer不阻断另两名确实就绪signer的合法threshold |
| AF05 | finalized停止及q增长但水位冻结 | 有限观测区间内零地区Force，记录ordering/input原因；恢复后按新公共视图判断 |
| AF06 | 生产治理入口的阈值/进度向量 | 阈值−1µs不签、等阈值可签；无效、重复、未选中/不完整结果不刷新；本地缺已公开对象WAIT；不安全时钟/HALT/PENDING和预算不足均不签 |
| AF07 | 真实合约签名/配置向量 | k−1拒绝，k成功，重复/非成员/坏顺序/错误域或类型/篡改policy/畸形包拒绝；另一个合法阈值配置实际验签 |
| AF08 | 真实L1去重与治理持久重载 | 同原请求重发只收录一次，替换合法签名集合不改变content；未知广播保持同tx；冲突rid/过期seq拒绝；重复轮询/replay无新控制 |
| AF09 | Plan生产入口历史与资格向量 | mask6/sourceR0→R1，mask4→R2，后续授权重新启用区可按循环选回；不同排列、无source、零mask及缺历史分别得到确定Plan/NONE/WAIT |
| AF10 | policy收录/激活/历史重放 | act前旧policy，act后有效控制的新policy；旧Plan/cid及证明不随后来policy变化；同L1块多控制按位置应用；stale/HALT不得安装policy |
| AF11 | 独立读者和worker验证 | A/B不同交付顺序在同位置有相同policy/Plan/G，worker从历史证据验证实际Plan；篡改history height/policy/Ref/committee拒绝 |
| AF12 | 同候选M2健康Force回归 | 原单签/静态profile主运行与独立checker通过，h10 anchor、R2[16,25]和h16终点保持既有规则 |

AF01/02/03 是真实多进程场景。AF04/05 可采用相同治理服务入口的有界公共流场景，报告是否包含BFT worker。AF06/09/10/11 为独立向量，AF07/08 必须保留实际 Anvil 收据或治理持久记录。每项报告层级、触发、前状态、guard、输入证据、后状态及局限。Verifier 先导出预期，再阅读候选；Tester 对同一已复核 SHA 执行。

## 证据及交付

新的 M3 专属目录封存代码/合同/预期 hash、完整SHA、clean状态、工具、二进制、锁文件、seed/profile、15进程及故障退出、逐signer已验证前缀与replay、阈值决定、每个签名的digest和恢复身份、请求持久状态、真实交易/收据/完整L1位置、policy生效记录、历史Plan依据、worker Ready/Enter/Close、区域Final/body、Espresso和G轨迹。

只允许一个 build/重型simulation，复用 `/tmp/megaroation-poc-heavy.lock`，两Cargo jobs及锁定离线依赖。最终提供一条运行命令、一条离线检查命令、候选SHA、各用例结果和证据路径。每次重跑使用新目录。代码改动后重新命名候选并复验受影响部分。主运行虚拟上界60s、事件上界1,000,000、单命令墙钟watchdog1800s。

本合同冻结以下公共编码。任何改变本合同 guard、输出或验收的决议都需修订合同。

## 历史 Plan 与控制投影字节

GFR 正常 genesis 和首次 Force 前沿用原静态 Plan。M3 控制的 ControlRef.source_bytes 为 ASCII `PoC/PolicyForce/v1`、target_beta:u64、四个 u8（mask、order0、order1、order2）、signed_l1_floor:u64。整数均大端。真实 Force 仍用 L1Position、kind 与 control_seq。M2 control tag 与 projection 保持原字节。M3 持久化 replay record 使用独立 tag，绑定完整 M3 source_bytes；checkpoint 保留 Plan 读取的真实 L1 header 支持前缀。

历史 Plan 的 authorization_ref 使用 `poc_types::hash_parts("PoC/HistoricalPlan/v1", parts)`。parts 顺序为旧静态 authorization_ref(config, registry, input) 的 32 字节、Force L1 block:u64、tx:u32、log:u32、control_seq:u64、四字节 mask/order、signed_l1_floor:u64、effective_floor:u64、historical_height:u64、historical_block_hash:32 字节、selected_region:u8。hash_parts 的域名及每个输入都带 u64 大端长度。历史 Plan 的 RuleSpec 为 rule_id=2、version=1、32 字节 params 等于该 authorization_ref，使用现有 RuleSpec 长度字段编码。因此新 rule 的历史依据进入 ProgramConfig 与 cid，同时选中委员会内部的轮转保持原算法。M1/M2 的 RuleSpec 和摘要字节保持原规则。

GFR 以已激活 policy 和经验证的真实 L1 cut 计算历史 Plan。worker 的 Install 连同已选中 descriptor 提供完整 Plan basis，使用相同规范重算，不能从最新 reader 状态或静态 timestamp 投影替代真实历史 header。Publisher 对历史规则核对该 descriptor 在公开状态中的完整选择记录。空 mask 在所需历史完整时返回 NONE，缺历史返回 WAIT，非法 mask/order 拒绝。

Lead 批准 Builder 在 `poc/crates/types/src/domain/objects.rs` 进行此 RuleSpec/PlanRecord 编码与校验变更，并在授权、GFR、发布桥和 native 准备路径贯通上述历史证明。其他 Cargo、公共类型或 wire 变更仍需协调。Verifier 可为现有 M2 独立 Rust 投影增加可选 M3 路径，既有 CLI 数据含义不变。
