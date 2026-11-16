# H1 Q-ADMIT-v1：普通profile准入桥接

2026-11-16。**来源映射FROZEN BY LEAD；真实native实现与矩阵NOT_RUN。** 本决定采用路线A，经命名bft_spec与独立bft_verifier分别从固定来源核对。它关闭适配选择问题，不替代实现验证。正式实施授权为d55d39f73593676325bdc0fbb7ba3845ca93025f，M0为35d382ac445cc8bc779f0d18fd23cb57e54680da；不引入01恢复WIP，不修改vendor/quorum/lock/vote规则。

## 1. 适用范围与源规则

固定Commonware2026.10.0/upstream92a2f68a0e131c8b69de41e84aec37da6280f212，普通Engine::new、Dynamic elector、rotating single-view term，无optimistic/pipelined/early-skip。若启用其他profile，本映射不能自动继承。

SP§3.8的本地Accept使用持久原首收；SP§4.3 BFT-2要求进入可用parent/决定前存在honest Accept，并非每个最终投票者都曾本地Accept。SP§4.9仍限制所有新共识签名的stop/Close。Commonware `consensus/src/lib.rs:159–201,317–349`要求verify终值诚实一致、certify确定且一致，propose返回即承诺verify/certify。false/关闭是单次请求终值。

本地永久时间拒绝在本适配中明确解释为**该节点初始proposal/notarize准入的永久弃权**。取得有效notarization后，同节点仍可做确定性certification，并在原生规则和实际生命周期守卫许可时finalize。不得伪造该节点的Accept/notarize记录。历史CheckFINAL不读当前clock。

## 2. 两个谓词与回调

`V(context,payload)`只检查确定性内容：canonical/cid/namespace/window/native-parent与physical-parent绑定、metadata、完整祖先执行、pre-root、BasicTx。缺真实输入保持pending；已知永久语义错误false。V终值不取决于本节点首收、当前Clock、Selected/Pause或Close。

`A(node,proposal)`按SP§3.8检查原首收及5a–5d，持久保存ACCEPT/WAIT_TIME/永久LOCAL_REJECT。适用proposal身份绑定cid/epoch/view/native-parent/physical digest。认证及完整header/body首收规则另按H1 ingress合同，不可用认证完成时间改写首收。

| 原生入口/事件 | 固定映射 |
| --- | --- |
| propose | 使用context指定parent；NotBefore、真实执行、V=true、自身A=ACCEPT、正文持久完成，返回前复查Running/Clock/stop/terminal后才返digest。不得期待后续verify补首次Accept。 |
| verify，V未知 | 及时返回oneshot receiver，保留producer及有界受控工作；缺输入继续pending。 |
| verify，V=false | 返回false，仅表示诚实节点一致的永久语义错误。 |
| verify，V=true/A=WAIT_TIME | 保留同请求和原首收；原时间条件满足后重试，不重置native deadline。 |
| verify，V=true/A=LOCAL_REJECT | 保存HeldLocalReject，不返回false、不主动丢producer、不再变为ACCEPT；等待原生真实取消该verify。native timer/nullify正常运行。 |
| verify，V=true/A=ACCEPT | 返回true；原生parent/lock/nullify/issuance规则仍决定是否notarize。 |
| certify | 从真实已验证notarization取完整(round,payload,parent-view/digest)绑定，执行V并持久保存完整body/ancestry后true；缺输入pending，语义错误false。不得从不存在的本地A缓存猜parent，也不因A拒绝或当前时间返回false。 |
| Finalization | 公共历史验签后记录proof observation；正文/祖先/执行完整才连续delivery；公开完整与G接纳仍在H3。 |

所有方法自身须及时返回receiver，耗时工作在receiver后执行；不得在propose/verify/certify返回receiver的方法future里等待材料，因为voter直接await方法会阻塞其timer选择。propose成功后，其对象已具备V/A和持久数据；之后时间变晚不能把certify改false。实际新签名仍由Running/Clock/lease/stop/terminal守卫控制。

## 3. 原生状态、取消与有限进展

锁定源码核对：

- `voter/state.rs:1067–1083`及`round.rs:453–461`：本地propose直接标Verified。
- `round.rs:728–749`：只有Verified可notarize；`slot.rs:175–203`及`round.rs:494–502`：certificate/certify不授予Verified。
- `round.rs:241–260,755–784`：notarization可直接启动certify，finalize明确不要求本地verification。
- `voter/actor.rs:1013–1019,1243–1269`：Reporter先可保存完整notarization，随后dispatch certify；不因此新增vendor观察hook。
- `actor.rs:503–535`：view退出取消propose/verify，不取消certify。covering Finalization（`state.rs:660–683`）/prune才取消相应certify；跟随真实receiver关闭处理。Pause保留producer、receiver、generation和deadline，不等于Close。
- `state.rs:743–751,796–800`及`round.rs:783`：调用Scheme.sign之前已写notarize/finalize issuance或Decision。返回None不会回滚，不能实现可恢复WAIT/Pause。M0 vendor对应state.rs:849–857/902–906保持该顺序。

回调审计区分Requested/Waiting/Held/Offered/Cancelled；发送oneshot仅证明Offered。不得没有实际观察hook就声称native已Consumed或投票。当前映射不要求新增精确native callback consumption receipt；若未来确需该能力，须单列最小观察hook和授权。

q4/5且f≤1的notarization至少包含三个honest notarizers，每个honest notarize来自已通过A的propose/verify。保持原生parent/nullification/lock规则，由此归纳得到可认证parent与Final所需的honest Accept存在性。V负责真实应用与祖先执行有效性。此为来源映射理由，不是新形式化证明。

有限活性仅适用于同cid固定active quorum、完整输入和声明处理预算、`T_win≥T_build+Δ_region+2δ_G`、native timer能吸收NotBefore及WAIT等待、截止严格在stop−δ_G之前的区间。不能无条件继承全部Commonware活性。尤其不能把certify也按本地时间拒绝永久held：既有notarization后一个late honest永久不认证、另一节点停止合作时，其余三个不足新quorum。

## 4. 必须实现后独立执行的配方

| 场景 | 预期（全部PLANNED/NOT_RUN） |
| --- | --- |
| h1 recv10,998,000，now<11,000,000，随后到时 | 同receiver/原首收WAIT后通过；recv10,997,999为永久HeldLocalReject。 |
| h1 recv12,199,999与12,200,000 | 前者可准入、等号永久本地拒绝；补body/认证/重试不改写首收。 |
| 一个honest本地拒绝，其余支持真实q4 notarization | 被拒绝者可V+durable后certify/finalize，但无本地Accept/notarize伪记录；不是所有票本地A的另一profile。 |
| 所有honest未Accept | 一个Byzantine无法形成q4，不能产生新可用parent/Final。当前健康M1不把此配方冒充已运行恶意场景。 |
| propose返回后截止/ClockUnsafe/Close | 不把V/certify改false；所有新签名停止，历史证明可验证。 |
| Pause→原deadline已到→SelectBack | 原future/receiver/generation/deadline不变，先检查安全guard，再处理原到期事件。 |
| view-exit与covering Finalization | 前者只取消propose/verify，后者按真实native规则取消覆盖certify；迟到完成不复活旧请求。 |
| 暂时在Scheme返回None | 作为禁用路线反例核对已写issuance无法恢复；生产Pause必须停车，不用None。 |
| proof先到、完整body/ancestry后到 | proof observation与delivery分开；历史CheckFINAL与观察者当前Clock无关。 |

所有具体callback/ingress/body存储实现仍由后续单Builder切片完成并在同SHA验证。若上述固定profile或前提不能保持，停止相关native路径并回到精确问题；不把这一局部失败扩成停止已冻结API/route/guard工作。
