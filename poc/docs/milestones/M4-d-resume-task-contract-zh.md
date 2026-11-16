# M4 D 派生与预设 RESUME 合同 v1

合同标识 `M4-D-RESUME-1`。基线为 `b404818e18be62663e84a7fa580b151c686ad951`，目标工作树 `/home/jy/.codex/worktrees/poc-m4-resume/megaroation_poc`，分支 `codex/poc-m4-resume`。Lead 提交本合同和独立预期后开始实现。正常合并完整组件交付 `aab8a5180801ed7f97f7e437d147c97f972b4dd8` 与 `1f6d1b3b4076ff16908dc59f272a06686b1d1ea5`，最终矩阵绑定集成后的准确候选 SHA。

## 来源与实现边界

| 来源 | 本合同要求 |
| --- | --- |
| v10 Authority §3.10、§3.12 | RESUME 授权、目标单调、同交易 Override、恢复预算和历史 Plan |
| v10 GFR §3.3–3.4、§4.6、§5.4–5.5 | 公共全序与覆盖、原子等待、G 对齐、独立 D、精确 ResumeCut |
| v10 Manager §3.10、Session Program §3.4–3.12 | 恢复材料先于准备、永久终止、错时进入、真实 Final 与签名守卫 |
| PoC v2 §9.3、§10、§12，M1–M5 计划 M4 | 真实批次/DA、原生窗口回退、共同高度分歧和恢复首块终点 |
| 功能 11 `task-contract-batch-services-v1.md` 与实际交付 README | 唯一 BS-FRAME-v1 编解码、DA commitment、签名及 AcceptedRecord 接口 |
| 功能 12 `task-contract-derivation-v1.md` §§3–10 | 已交付 Stage A 的 D 父状态、原生 origin、窗口闭合、CursorSet 和 Stage B 批次接入义务 |

固定 Commonware 2026.10.0、Rust 1.95.0、Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。区域共识采用三区各五个等权身份，Final 至少四个不同合法 BLS signer，窗口长度十。治理沿用 M3 的独立 EIP-712 成员与默认 2/3 门槛。恢复请求在本阶段使用预设合法目标，签署前核验其实际公共前提和预算。staking 与禁任仍按固定 registry 的 mock。

## 所有权和公共接口

一个 Builder 负责 M4 所需的 `poc/crates/derivation/`、`poc/crates/batch-services/`、`poc/crates/l1-adapter/`、`poc/crates/viewer-gfr/`、`poc/crates/session-manager/`、`poc/crates/publication-bridge/`、`poc/prototypes/i1b-runtime/src/` 及 `poc/scenarios/m4-resume/`。允许按实际需要扩展 `poc/contracts/src/` 与测试，用于已有 RESUME 和 CursorSet 的调用/回滚验证。Lead 拥有共享类型、Cargo 依赖和编码冻结，Builder 修改这些接口前列明精确文件及兼容行为。新实现属于 `poc/`。Verifier 独占 `poc/verification/m4-resume/` 及经 Lead 明确批准的既有独立 checker 扩展，Tester 独占其 `tester/` 和正式证据目录。回放 UI 由另一个 chat 负责。

功能 12 增加对功能 11 的实际依赖并复用其 frame、commitment、签名和 accepted input 能力。`Feature11AcceptedPayloadRequired` 的替代路径必须由真实成功的 finalized CursorAdvanced 和验证后的实际 DA bytes 满足。D 的外部输入仅来自已验证 L1、DA、固定配置及 DEP，不能接收 G 的执行结果作为 D 输出。

GFR 新增受验证的 ResumeReader 输入，按 RESUME 的 L1Position 查询切面。切面必须携带实际目标、完整历史派生前缀、目标 ExecutionAnchor、匹配 CursorSet 的位置和配置/输入依据。生命周期批次固定为 `BranchClosed → Resumed → SessionAssigned`。低目标终止分支使用 `BranchClosed → Resumed → SessionClosed(EXPIRED)`。同一批次对 Manager 原子核验，材料缺失时保持事件前状态，成功后按上述顺序提交并释放动作。

RESUME 控制编码、Resumed 生命周期投影、DER 来源、ResumeCut 持久引用、批次源 gpos 与历史 Plan 域隔离的精确字节须由 Lead 在实现公共接口前记录。既有 M1–M3 的版本化编码和命令继续可重放。新域至少绑定控制类型、精确 L1 position/sequence、目标高度、真实恢复 anchor 和切面引用，避免与 Force 互换。Plan 的历史依据使用恢复 anchor 的实际 origin。

## 从 G 到批次及独立 D

Batcher 和 Certifier 从真实 GFR 已提交的重放位置读取规范块、普通交易、父状态和原生哈希。gpos 必须能回到该真实位置与连续区间，服务端核对提交之前的 cursor。允许通过既有 BatchSource 的新实现提供该视图，保存足以独立重放该视图的原始日志。服务角色保有各自身份、输入、持久目录和结果，所有输出经宿主受控事件及提交边界。

DA 使用功能 11 的唯一帧格式、真实 commitment 和 DA receipt。批次使用真实 Certifier attestation，经 Anvil SubmitBatch 收录。D 由实际配置的 finalized Inbox reader 读取原成功 CursorAdvanced，核对完整 receipt、事件位置、签名、批次范围和 commitment，再经 `AcceptedRecord::load_payload` 或等价的已验证生产接口取回 typed records。不得将任意 JSON 或调用方提供的 finalized 标记提升为认证输入。

同一条接受输入的数据缺失或 commitment 不匹配时，D 停在该精确位置。错误对象不能产生派生块，补齐原 commitment 的正确 bytes 后继续。后续 CursorSet、回退和其他批次均不能越过这个未解决输入。当地读者延迟不改变实际 inclusion height。

普通批次接入沿用功能 12 Stage B 的已冻结原生规则。按实际 D 父状态顺序检查时间和 origin。未来时间记录保留等待，旧记录舍弃。晚到判据严格为 `inclusion > origin + sequence_window_blocks`，等于边界仍及时。格式错误不得被转为成功的同高度替换。原生指定的 origin/执行替代规则在输出当前高度前应用，再用新父状态检查下条记录。已输出 D 前缀永久保留。

无合格批次时，只能在完整扫描其原生窗口后执行 deposit-only 回退。D 的 origin 由功能 12 的实际父状态和原生规则确定，不能调用区域生产的 origin 选择代替。普通批次、回退、替换和 CursorSet 的 provenance 都必须进入 DerivedItem 证据。比较 D/G 使用同一高度实际 NativeHash、metadata、普通交易与执行状态。

## RESUME、历史切面与 G 对齐

SubmitResume 验证治理签名、序号和 policy，目标严格高于 Inbox cursor，目标 origin 不超过实际收录 L1 height。Authority 在同一 EVM 交易内执行 Inbox Override 并写 ResumeRecorded。内部调用失败时，cursor、请求缓存、序号和事件全部回滚。控制 act 为实际收录 timestamp 加 M_a。

Derivation 按 `(block,tx,transaction-local log)` 顺序先处理此前接受输入。CursorSet 重置 channel，并从实际 D 父块按已冻结填充规则补至目标。每步 origin 最多加一且不超过包括该记录的 L1 block。请求携带 origin 作为核验输入，不能直接覆盖派生规则选出的 origin。

切面键为 ResumeRecorded 的精确位置，值是处理其配对 CursorSet 之后的 D。配对记录必须同块、同交易、相同 tx hash，保留 transaction-local log 与 RPC block-global log 的对应。D 已超过目标时保持完整末端，切面可含目标之上部分，但 target anchor/state 必须取目标高度。后续 D 增长不改变旧切面。DeriveAtResume 查询不会回退 live D。

GFR 处理 RESUME 的前提为 Espresso watermark ≥ act、完整 finalized L1 控制前缀覆盖 act−M_a，以及精确切面、DA、Plan 所需历史全部就绪。缺失时原位 WAIT，不能提前关闭旧分支、替换 G 或安装 policy。完成前所有变化在私有 staged state 内计算。

就绪后关闭旧分支全部 cid，取切面 D 的 genesis 至目标前缀替换 G。共同原生前缀保留，旧 G 分歧后缀被撤销，采用的派生部分记录 DER 来源。D 保持自身末端。新 branch 为 β+1，session 为 (β+1,0)，window=[h_res+1,h_res+10]。anchor 的连接标识使用 `derived_anchor_id(height,native_hash)`，状态为 D[target] 执行后状态。新 floor=A.origin，Ref=BlockRef(A)，sourceRegion=None，Plan 历史读取高度等于 A.origin，从新 policy 排列首项开始选区。

若新下一高度的 Expire 或新窗口 SessionEnd 严格早于 act，G 仍完成对齐，Tail 置 HALT，输出 Resumed 和新 session 的 EXPIRED 关闭，跳过 Plan/Assign。等于 act 时先处理同键全部 L1 控制，再处理仍适用的 Expire/SessionEnd。成功上链 RESUME 不因旧 branch、旧 Tail 或低目标被 GFR 忽略。

Manager 从 Resumed 的精确位置重新取得并核验同一切面，核对 anchor/state/root 和派生前缀，然后才 Prepare/Ready/Enter。新区域真实 native engine 的首块 parent 连接该派生 anchor，并在实际派生状态上执行。旧 branch 的迟到结果不能扩展新 G。重复生命周期及迟到 Prepare/Enter 不复活旧 cid，terminal 先持久再 Close。

## ResumeCut 接口与引用冻结

复用功能12现有 `ResumeCut` 数据类型及字段顺序。以 `serde_json::to_vec(&cut)` 的紧凑序列化得到 cut_bytes，使用已有 `poc_types::hash_parts("PoC/Derivation/ResumeCutCheckpoint/v1", &[cut_bytes])` 得到32字节 cut_digest，继续与功能12检查点采用同一域和长度前缀规则。含普通批次的新 DerivedItem cause 使用明确的新variant并由同一确定性serializer承载。校验先重放实际输入再比对摘要，缓存摘要本身不授予真实性。

受验证reader逻辑接口为 `read_resume_cut(resume_position) -> Ready(ResumeCut, cut_digest) | Wait(next_position, missing) | Reject(reason)`。消费者核验 position、同交易CursorSet关联、target、configRef、实际包括block及交易hash、目标原生hash/state和整个前缀一致性。对同一键的相同值幂等，冲突值拒绝，读者推进不得更新旧键值。具体Rust trait及adapter位置由Lead冻结，接口不得允许未经验证的任意ResumeCut导入GFR。

GFR在自己的replay/checkpoint保留该次引用和采用的目标前缀，Resumed沿用共享类型内现有control_seq、branch、anchor、derived_prefix字段，并在对应replay input/basis保存cut_digest。Manager按事件的L1位置重新查询同一个reader，核对该引用和完整StateMaterial后提交事件批次。worker安装材料包含该cut的规范bytes或可验证的持久引用、目标状态与描述符，沿用controlled Install校验到相同anchor。

首轮精确路径允许沿用以下模块，并由单Builder拥有新增相邻专用模块。批次源接线为 `poc/crates/batch-services/src/{source,accepted_view,batcher,certifier,lib}.rs`；D接线为 `poc/crates/derivation/src/{derive,model,source,lib}.rs`；GFR为 `poc/crates/viewer-gfr/src/{api,state,replay,frontier,codec,publications}.rs`；Manager为 `poc/crates/session-manager/src/{api,state,lib,replay,actions}.rs`；发布桥为 `poc/crates/publication-bridge/src/{record,result,stop,verifier}.rs`。运行接线为 `poc/prototypes/i1b-runtime/src/m1_smoke/`、必要的 `native_application.rs/native_config.rs` 与新 `poc/scenarios/m4-resume/`。L1读取和控制投影只在已有 `poc/crates/l1-adapter/src/` 修改必需接口。上述对应 tests 可增补合同案例。新增依赖及公共类型的具体变更由Lead记录，组件原协议格式复用功能11/12的既有编码。

## 主场景 profile 与可行性

主运行 seed42，15 个健康独立 OS worker 持续到观察截点。保持 ±1ms 读钟误差、RPC 单向250µs、区域链路4ms、工作5ms、Enter 错时3ms。L2 genesis timestamp10s、slot1s，L1 slot2s、finality1.5s，M_a10s、grace3s、W_exp30s。M4 sequencing window 选4个 L1 blocks，作为场景显式配置。T_prep12s、T_ready≤0.5s、T_drift15s、ε1s。Espresso 独立 heartbeat250ms继续提供公共覆盖。所有配置在签名和启动前冻结并写入 profile。

M4 统一 PublicConfig.chain_id 与真实 Anvil chain31337，重新计算配置摘要、原生上下文和治理域。实际 L1 身份绑定 genesis hash、Authority、Inbox、submitter、configRef 与参数，避免沿用 M3 原生 chain7/Anvil31337 的跨组件不匹配。有限 DEP/L1 材料至少覆盖主恢复至h21所需高度及origin。

Lead 已确认以下主场景时间表。

1. R0 正常产生并向 G 接纳 h1。功能11从该真实 G位置生成含普通交易的 h1批次，及时在 L1 h6/timestamp12s 收录，finalized后由 D 独立执行。h1 的 origin3，窗口末端为 L1 h7。
2. h2 也真实进入 G。对其对应批次施加显式 Batcher 发送延迟，使 inclusion 为 L1 h8/timestamp16s，严格晚于 origin3 的窗口末端 h7。D 在完整 h7 于15.5s可读后按规则回退，形成可核对的共高 h2 hash 分歧。h2的迟到批次仍保存成功/失败收据及真实处理结果，不移除失败轨迹。
3. 在17s检查 G、D、Inbox和恢复输入，按预设 h_res20、请求 origin9 发出合法2/3 RESUME，于 L1 h9/timestamp18s 收录。`18+12=30=ts*(20)` 且 `30<=18+15−1`，目标高于发送时的 heads/cursor。新policy排列为[2,0,1]且开启三区，从无来源开始选R2。
4. D 于已验证该控制后建立精确切面，CursorSet从其实际父状态填至h20。GFR 在28s公共键应用RESUME，关闭R0/R1旧分支并采用D[0..20]，新R2窗口[21,30]。R0→R1正常接棒在此之前发生，R2尚未安装旧cid，从而本主场景可沿用每进程一个native实例的已验profile。
5. R2 至少四个固定成员在30.999s之前Ready/Enter，首个真实h21在31s之后完成Final/发布/G接纳。下一高度Expire为ts*(21)+30=61s，首接纳严格早于该键和本session公开截止。观察截止为新h21进入G并完成受控收尾。

以上高度必须由实际收据和日志独立复算，运行时不得硬写D或G结果。若实际批次/控制错过冻结槽，保留证据并判对应正例失败。正常路径不等待所有节点统一就绪。D 在控制激活前可以继续推进到目标以上，GFR仍查询该条历史切面。

## 验收矩阵

| ID | 层级与触发 | 必须观测 |
| --- | --- | --- |
| DR01 | 主15-worker运行 | 真实G来源及时普通交易批次实际上链并进入独立D，第二批迟到产生共高分歧，预设RESUME采用正确切面，新R2首个h21进入G |
| DR02 | 同SHA/seed +650ms墙钟暂停 | 完整规范化虚拟轨迹、原生frames、真实L1/DA内容和切面输出一致，原始诊断保留 |
| DR03 | 实际接受数据缺失/错误commitment及恢复 | D在原接受位置WAIT，后续Set与fallback不越过，错误bytes不执行，正确bytes补齐后结果与连续读取相同 |
| DR04 | 缺L1/水位/切面/Plan历史/DEP | GFR或Manager在受影响事件前原位WAIT，无部分G替换或生命周期，补齐在同位置恢复 |
| DR05 | 真实EVM成功与回滚 | ResumeRecorded与CursorSet同tx绑定，目标不高于cursor/内部Override失败整体回滚，正确重试有唯一成功记录 |
| DR06 | 同L1块两次真实RESUME | 每个控制有自己的Set与位置切面，允许中间实际批次输入，后续D增长不改旧切面，两个控制按精确位置依次处理 |
| DR07 | D已经高于目标 | D末端不回退，切面保留整个历史前缀，target anchor/state来自指定h，G只采用至目标 |
| DR08 | 低目标严格过期及同键边界 | 完成对齐后HALT，无Plan/Assign；等键先处理全部控制再动态Expire，G保留正确已采用前缀 |
| DR09 | 生命周期、恢复材料和晚到工作 | 正确批次顺序，Running/Preparing/Paused旧cid关闭，重复及迟到Prepare/Enter不会复活，新状态使用精确ResumeCut |
| DR10 | 原生批次规则向量 | inclusion在window_end前/等于/之后、延迟读者、actual-parent origin、替换后下一记录重验、未来/旧/畸形和DEP缺失均符合功能12 StageB规则 |
| DR11 | 双reader和历史恢复 | 同公共位置的G/D/cut一致，检查点从较长L1前缀恢复仍重现同切面，错误位置/hash/commitment/config拒绝 |
| DR12 | 同候选M1–M3回归 | M1健康满窗、M2健康单签Force、M3自动2/3Force的正常路径及触及的协议向量通过 |

DR01/02采用真实15进程、原生证书、Anvil与服务产物。DR03/05/06须包含实际接受/控制收据，故障和重放可以在已封存真实证据上运行。DR04/07/08/09/10/11可用生产入口的确定性独立向量，并报告具体覆盖层级。DR06的实际批次必须由验证后的功能11接口进入D。新的接口通过后，对用户指明的功能11真实样本进行离线接入检查，确认原先Feature11AcceptedPayloadRequired缺口被可核验资料完成。

## 证据、限界和交付

每次使用新目录，保存完整候选SHA、组件merge祖先、干净状态、合同/预期hash、锁文件、工具与二进制hash、seed/profile、fault调度、15个PID和退出结果、grant/Ack/journal、真实GFR位置、普通交易、batch帧、DA bytes/receipt/commitment、batcher/certifier持久阶段、Anvil raw tx/receipts、控制精确日志位置、D逐height输出/原因、历史ResumeCut、G前后共同前缀/替换后缀、Manager事件批次与新Final。

单重型锁为 `/tmp/megaroation-poc-heavy.lock`，Cargo jobs2、锁定离线依赖。主运行虚拟上界60s、最多1,000,000个事件、每命令wall watchdog1800s。到上界未恢复就保留失败/未完成证据。独立Verifier先有预期，再审查候选；Tester只在同一已复核SHA执行。源码改变后命名新候选并重跑受影响范围。

交付给主chat准确SHA、manifest、独立checker与结论、全部原始证据、一条矩阵复现命令及一条离线检查命令。回放可据相同原始位置投影控制、D、G和恢复新块。M5将复用本合同验证过的RESUME路径扩展自动观察与目标求解。

## 具体接线

M4 使用一个 09 Driver，由 11 InboxGateway 处理真实 finalized 读取、当前 Inbox cursor 与已认证 AcceptedPayload。12 可在 driver 所在进程内接收 InboxGateway 的闭合 FinalizedBlock 与 AcceptedPayload，经现有 VerifiedInputs 推进。旧 DriverSource/FixtureSource 路径保留。正式 D 运行只用实际 driver，组件间不得新开独立链写入者。运行器可复用 M3 的 Anvil/线协议外壳，在新的 M4 端口模式中运行这一个 gateway、Batcher、Certifier、DA 与 D。批次的 GFR 快照由 Hub 的实际已提交 G 投影，绑定 gpos、区间、父状态和 native hash，且经 11 的执行重验。

GFR 的只读 L1 reader 增设默认等待的精确位置切面方法，使既有 ReplayReads 和其他 reader 兼容。M4 的 reader 交付由 12 重放得到的不可变切面。GFR 输入可采用 12 ResumeCut 的验证后窄投影，字段至少为位置、配对位置、目标高度、配置引用、实际收录块与交易 hash、切面摘要、genesis、目标 ExecuteAnchor 和包含目标的连续执行前缀。进入 GFR 前重新校验完整切面，状态机仍检查位置、相邻父链、摘要/根及目标对应关系。派生来源以 `GlobalSource::Derived { resume_position, height }` 或等价的有类型表示；GFR 保存单独有效锚点，公共共同前缀在 target 本身相等时也使用派生 anchor 标识接续。切面支持前缀和历史记录随 checkpoint 验证。Manager 消费具备同一切面引用和恢复材料的 RecordDelivery，缺材料时 WAIT。

`ControlRef::source_bytes` 的 M4 RESUME tag 为 ASCII `PoC/PolicyResume/v1`，后接 target_height:u64、signed_origin:u64、mask/order 四个 u8、signed_floor:u64、CursorSet 交易内 log:u64、实际 transaction_hash 32 字节。整数大端。外层 L1Position 为 ResumeRecorded，control_seq 取真实事件；CursorSet block/tx 相同，其本地 log 严格更早。历史 Plan 复用 `PoC/HistoricalPlan/v1` 及已有 RuleSpec id2，原始授权 preimage 含 BoundaryCause::Resume，历史控制位置域取 ResumeRecorded，floor 取 D 目标实际 origin，保留原签名 floor 的绑定。发布桥为 Reset/Resumed/DER/Resume AnchorEvidence 使用未占用 tag，老 tag 字节保持一致。具体字段顺序应描述于与实现同交付的编码说明中供 independent checker 固定。

Lead 允许依赖调整限于 12 增加 11、native runtime 增加 11/12/09，及由此实际影响的 Cargo manifests/locks。锁定现有依赖，不整体升级。共享 domain 文件如确需调整，需先给出具体路径和兼容说明。Verifier 可在 M2 独立 Rust public oracle 追加可选 M4 输入模式及在 M3/M2 verifier 辅助脚本增加向后兼容分支，保留原 CLI 和既有证据含义。其余验证修改限于 M4 独立目录。
