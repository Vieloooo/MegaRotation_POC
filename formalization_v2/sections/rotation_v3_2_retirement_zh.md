#### 关闭与退出规则

固定一个 session $(\beta,r)$，令 $\Delta=\Delta_{i_{\beta,r}}$，使用该 session 不变的 voting snapshot。程序记录新增 `close_requested`、关闭证据和首次本地关闭时间 $c_v$。这些字段不构成第七个 phase。收尾中的 Active/Frozen 程序保持原 phase；只有真实 Quit 或 terminal Completion 才改变相应 core 状态。以下规则明确扩展运行中的 core，不把 Force 当作 quit proof 或 terminal receipt。

**RequestClose。** Session Manager 对每份程序执行一次以下操作：

1. 验证完整证据，确认某个 authenticated finalized Espresso block 的规范 prelude 已关闭本程序的版本 $\beta$。证据包括 triggering header、从已验证 checkpoint 开始所需的完整 control/namespace 历史，以及重放得到的关闭与 retained-prefix 上下文。重新检查既有 MSIG/AUTO 条件。pending MSIG、OFF、未 finalized 的 header 或单独时间戳都不够。关闭不等待后继被选中或达到 Ready。
2. 在首次验证完成时，原子设置 `close_requested`，记录当前时间 $c_v$ 与证据，把完整 validation bundle 转发给固定区域成员。这只是传送已有来源证明，不是区域投票或新增证书。持久记录旧版本已关闭，阻止晚到 witness 再创建或启动旧 $\beta$ 程序。重复证据不重置 $c_v$。
3. 关闭本实例所有新动作入口：不能提出 DATA/DRAIN 或 NEWVIEW，不能投 DATA/DRAIN/ANCHOR vote，不能启动新 commit timer，不能运行会产生新 BLAME 的 progress check，不能进入新 term 或 generation。该限制也覆盖 initial Enter、延迟到达的 authority/recovery callback、terminal-QC rotation 和 buffered-message processing。新的 STATUS 或其他 entry 动作不能绕过限制。已经发出的消息不撤回。新 session 的准备与启动使用自己的 pair，无需等待旧实例退出。
4. 保留已有 commit timers 及其原 deadline。继续完整验证、比较保留的 signed material，并按 evidence-first 顺序处理消息。真实的 current-generation quit proof 仍执行 Quit、广播 proof、取消相应 timers 并保留 commitments，但不能重新打开 entry gate。存活到期的 timer 仍提交其 block 和 ancestors。每个由此提交的 DATA 所产生的 Protocol 1 FINAL，**同时发送给固定区域成员和 publication collector**，保留原 receipt 正文规定的区域传播。单个或不足阈值的 FINAL 不触发 Publish Layer 针对完整结果的保证。
5. 继续收集旧 votes、QCs、FINAL 与 receipts，把每个完整结果交给 Publish Layer。有效 terminal receipt 仍执行真实 Completion，可以转为 Completed 并取消 timers；已经开始的 close 仍保留其 drain deadline。其他有效 receipts 维持原处理规则。receipt 或后续 ON 都不能重新打开旧全局分支。在持久移交前继续承担 data service 与 publication 责任。

消息先于同一时刻到期的 timer 或 retirement event 处理，包括真实 quit evidence。RequestClose 不删除 receipt、lock、vote、已提交 block 或 provenance。

**FinishClose。** 对 RequestClose 当时处于 Active/Frozen 的程序，设置

$$
d_v=c_v+4\Delta.
$$

只有在时间 $t\ge d_v$ 且满足以下全部条件时，才能把程序标记为 Terminated：

1. 所有已有 commit timers 均已执行，或被真实 Quit/Completion 取消；任何排队动作都不能再通过新动作入口。
2. 持久存储已经接管 exit record、closed-version record、committed log、必要签名证据与 validation dependencies、receipts、anchor data 和可恢复 state，不能释放最后一份仍有义务保留的数据。
3. 持久责任方已经接管未完成的完整结果提交、publication retries 和 data service；每个本地完整结果均已 offered 给 Publish Layer。
4. dispatcher 把以后到达的 $(\beta,r)$ 消息交给这些责任方。它们可以收集或转发已有签名、验证迟到 receipt、服务数据并继续发布，但不能重启 engine、执行其 BFT timers 或为其产生新的 BFT 签名。

该移交可以发生在同一个正确副本内，不需要新增 agreement 或 certificate。全部条件满足后，才可释放生产/恢复 worker 与 timer machinery。迟到 terminal receipt 成为归档证据，不把 Terminated 程序改回 Completed 或 Active。

已被确定无效的 Waiting/Ready 程序尚未 Enter，没有区域 timers 需要收尾。请求退出时已经 Completed 的程序，也已通过原 Completion 取消 timers。这些程序在完成对应持久退出与责任移交后，可以不等待 $4\Delta$。尚未固定的候选仅暂时失去资格，不必因此终止。Terminated 不删除 commitment、不从 $G$ 删除 DATA，也不授权 stake withdrawal。

**命题：关闭传播与诚实消息收尾。** 在既有区域同步通信假设下，令

$$
c=\min_{v\text{ correct}}c_v
$$

为本区域首次正确 RequestClose 时间，且它转发上述完整 bundle。所有正确成员在 $c+\Delta$ 前关闭。全部存活的诚实 commit timers 在 $c+3\Delta$ 前到期；这些 timer events 产生的区域 FINAL 在 $c+4\Delta$ 前到达所有正确成员。因此，$d_v=c_v+4\Delta$ 是保守的本地 drain threshold，不是完成全部退出责任的 deadline。

**证明。** 首个正确转发是完整的 correct-to-correct message，其交付与验证受既有区域界约束，因此

$$
c\le c_v\le c+\Delta.
$$

正确成员在 $s=\tau_v(P)$ 投 DATA/DRAIN vote 时启动唯一 timer，其到期时间为 $s+2\Delta$。关闭入口给出 $s\le c_v\le c+\Delta$，也不允许 callback 启动替代 timer。因此到期时间至多为 $c+3\Delta$。Quit/Completion 可以取消 timer，不能推迟它。存活 timer 到期时实际生成的 FINAL，再经过最多 $\Delta$ 区域传播即送达。最后，由 $c_v\ge c$ 得 $d_v\ge c+4\Delta$；message-first 顺序处理等号情形。证毕。

该界从完整验证关闭并实际区域转发开始，不从 control 签名、triggering Espresso timestamp 或任意长历史 catch-up 的开始算起，也没有增加新的来源可见性假设。三个本地 $\Delta$ 足以覆盖最后一个诚实 timer event，却不足以覆盖其 FINAL 送达：某成员可在 $c+\Delta$ 前一刻投票，在 $c+3\Delta$ 前一刻提交，FINAL 直到 $c+4\Delta$ 前一刻才送达。

四个 $\Delta$ **不约束**后续 receipt relay、拜占庭消息释放、storage copy、$T_{\rm pub}$、$T_E$ 或成功 EXPORT acceptance。这些未完成职责转交持久责任方。没有 bounded storage size 或 storage-transfer latency 的承诺，单凭等待结束不能执行 FinishClose。

**命题：退役期间的条件性区域安全。** 在已声明的区域模型及 baseline DATA/DRAIN、receipts、Completion 的 soundness 前提下，上述扩展保持正确区域 commitments 的 prefix-comparability 与 irrevocability，也保持外部 global-prefix 论证。它不保留“每个存活 timer 都得到 protecting QC”的无条件结论，也不保证旧 session 的 progress 或完整 receipt 形成。

**证明。** 以下检查新增关闭入口相对于 baseline soundness 的影响，不重新证明全部 DATA/DRAIN 与 Completion 规则。考虑正确副本对 generation $g$ 中 block $B$ 的 direct timer：它在 $s$ 启动，并存活到 $s+2\Delta$。如果该 generation 的首次正确真实 Quit 发生在 $a\le s+\Delta$，proof 会在 timer 到期前送达 committer 并取消 timer。收尾仍处理此 proof，因此任何这样的 Quit 必须满足

$$
a>s+\Delta.
$$

每个正确副本进入 $g+1$ 的时间必须满足

$$
e_v(\beta,r,g+1,0)\ge a_v+\Delta
\ge a+\Delta>s+2\Delta.
$$

分两种情况：

- 若 $s+\Delta<c$，在 $B$ 的完整 proposal 传播窗口内，retirement gate 尚未压制任何 vote。baseline protecting-QC 和 recovery-lock 论证继续适用于该 timer，并保留原有真实 Completion 的处理。
- 若 $s+\Delta\ge c$，timer 到期时间至少为 $c+\Delta$。此时所有正确成员均已关闭，而任何后续 generation entry 必须更晚才可能发生，因此被 entry gate 禁止。后续 generation 的 QC 又必须包含正确 signer，所以这一晚期窗口内缺少 votes 不能通过再次 recovery 造成冲突。

同一 generation 内，完整 signed histories 或者相互延伸，或者给出原规则中的 admissible equivocation proof。正确 voter 转发其 history。对不相容 history 投票的正确成员，要么先收到前者并检测出 proof，从而不能投冲突票；要么已足够早地转发其冲突 history，使首个 voter 在自身 $2\Delta$ 到期前发现 proof。evidence-first Quit 因而阻止冲突 timer commitments。这依赖保留并继续检查证据，不依赖已关闭成员继续贡献 matching vote。

跨 generation 时，如果存在正确的更高 generation vote，较低 generation 的 direct commitment 就不可能属于上述晚期情形。早期情形的 baseline protecting-QC/lock 论证保证 extension。Ancestor commitments 继承可比性；receipt-based commitment 继续依赖已提交该 DATA 的正确 FINAL signer，包括 baseline terminal-Completion 情形。没有退役步骤撤销 commitment。

公共解释器仍在规范 Force boundary 保留整个 $G$ 并拒绝后续旧版本 DATA。局部退役既不选择 anchor，也不授予 append 权限，因此不改变这些外部状态转换。证毕。

不能直接照搬 Part I 的 universal protecting-QC lemma，因为该证明要求所有正确成员给出 matching vote，而关闭成员现在只保留 proposal、不再投票。例如，三个单位权重成员、$F=1$：正确 $H_0$ 在 $c=0$ 关闭，正确 $H_1$ 在 $\Delta$ 才收到关闭。拜占庭 leader 在 $0<s<\Delta$ 把有效 proposal 交给 $H_1$；后者投票、转发并最终提交，$H_0$ 仅保留。如果拜占庭成员拒绝提供 vote 与 FINAL，则只有 $H_1$ 的 FINAL，权重不大于 $F$，proposal QC 和完整 receipt 都可能永远不形成。退出规则保留这份本地 commitment 和证据，却不能称之为 Completion。Publish Layer 的完整结果前提也不能补出缺少的签名。
