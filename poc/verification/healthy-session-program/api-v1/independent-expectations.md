# healthy-session-api v1 独立来源预期

状态：SOURCE_DERIVED_BEFORE_API_CONTRACT。42 条配方已在读取新 api-slice-v1.md 之前从 d55、固定H2/H3及v10/03/07来源推导并向lead报告；首次保存时仍未读新API合同。基线4741489ba7a07d2a3f3f194dbd59ae3fd8d17bb9，M0=35d382ac445cc8bc779f0d18fd23cb57e54680da，实施授权d55d39f73593676325bdc0fbb7ba3845ca93025f。

## 必须保持的接口语义

1. 全部公共对象直接复用03，公开位置、lifecycle身份和publication编码复用07。新增crate只含typed DTO/ports、绑定、本地ID与纯形状检查；不是第二套Manager、BFT、runtime、公开codec或默认真verifier。
2. 命令、句柄、job、receipt的本地身份不能在run/worker/cid/anchor/generation/operation之间误用。完整操作重试保持稳定身份；在旧身份下替换绑定材料须拒绝或明确冲突。精确字段/tag/framing等待合同，不先伪造golden。
3. 完整已提交ReplayRecord是batch，保留全部有序GFR-LIFE-1事件身份；同pc不同事件不合并，events为空不等于没有完整record。source/context/batch完整性缺失与后继执行状态缺失分开。
4. d55已决定每worker单一root owner；H2返回不可变ManagerDelta/intents。合法关闭/tombstone、Selected+Preparing、cursor与actions共同接受为Committed(actions_pending)，执行材料缺失只阻止Ready/Enter，不能延迟已知合法关闭。ActionsApplied与持久接受不同。
5. 已安装terminal/ClockUnsafe fence不可因后来提交/action失败回滚。提交前拒绝仅针对尚未生效候选，不清除先前安全事实；后续仍pending/失败关闭。GFR pc、Manager接受cursor与action执行状态保持分层。
6. Pause/SelectBack元数据保留原handle、绝对deadline、pending工作与一次Enter身份；API不假称已验证真实engine停车。ClockUnsafe为私有local cause，不新增03公共CloseReason或伪装成公开Time。
7. H1 ReadyToken绑定worker/cid/anchor/实例或job代次/验证版本，以及真实ExecutionAnchor与来源/执行依据。root/URL/bool或H3 VerificationBasis::Ready(())不能升级成执行Ready。shape成功仍须由后续H1真正验证来源、执行与持久记录。
8. subject证书观察、完整block绑定证明、连续本地delivery、公共完整和G接纳分开。保持03 RegionalResult所有真实proof/body/ancestry/anchor；缺basis为WAIT、已知错误Reject。直接复用FinalVerifier/VerifierBinding，历史验签不依赖当前clock/lease/live engine/Selected。
9. 本地Time/Pause/ClockUnsafe不能伪造Publisher公共stop来源。所有纯构造/hash/check无隐含网络、fsync、clock、随机、timer、engine、签名或reader推进。
10. API元数据不是不可伪造权限或真实fsync/crypto证据。即使DTO可构造，也只能声明形状/绑定；生产issuer/consumer必须核验实际事实。测试中的fixture verifier必须显式仅测试，不计密码学通过。

## 已决范围与未覆盖

动态cid已选A，route/mailbox同步revision；不再把M0或整项01当API阻塞。Q-ADMIT真实native回调映射仍独立审查，不阻塞本纯API切片。Force/M2、D/RESUME、OS恢复、15-worker与M1端到端不属于本切片PASS。

此包是原75基础＋2复核runtime配方之外的新独立材料；旧来源封存与哈希不覆盖。当前没有新Builder代码候选，全部production/Rust/crypto/runtime执行NOT_RUN。读取新API合同后才增加精确goldens与合同一致性结论。
