# v3.8 candidate: historical regional owners and finite-cut hydration

Status: constructive candidate, not a completed protocol-verification report. This file changes no manuscript or frozen assumption. It gives an actual discovery, transfer, and admission procedure. Its essential choice is that a node that has received a still-pending request keeps its node-level request service until global inclusion. A production worker may terminate while that service continues. An independent physical-departure deadline is not promised.

Sources checked: `revisions/v3_1_frozen/model_and_assumptions.tex`; current `sync_hotstuff_rr_part2_rotation_v3_7.tex`, especially Requests and retained data and Graceful close and termination; `sections/rotation_v3_4_retirement.tex`; `sections/rotation_v3_7_liveness.tex`; `request_ingress_construction.md`; and `request_report_scope_analysis.md`. The latter's permission question about changing FIFO is obsolete for this design task: the parent task expressly permits protocol evolution, while preserving the v3.1 environment.

## 1. Concrete outcome and its exact boundary

Use an immutable, AUTH-indexed directory of **historical regional request owners**, and copy their finite retained cuts into each new roster before its first Ready. Do not replace the directory by the latest roster, and do not transfer the only pending copy merely because a new roster acknowledged it.

There is no new consensus protocol. AUTH already fixes the order and the membership of the directory. Different correct hydrators may acquire different supersets. Their certificates need agree only on the target authority and completed procedure, not on a unique mempool root.

The useful completion rule is:

> A correct target member signs HYDRATED only after its own finite query has obtained every correct historical owner's cut, and after it has actually disseminated that acquired pool to every target member and allowed the existing regional delivery interval to elapse.

A HYDRATED certificate of weight greater than the target's F contains one correct signer. That signer's actual completed multicast, rather than intersection with any later certificate, makes every correct target member aware of the required requests. This removes the need for the synchronous descriptor-agreement construction in the earlier fixed-pair note.

The cut cannot mean “every request delivered anywhere just before the first DATA is accepted.” A request received by an old owner an arbitrarily small time before acceptance need not have traversed even one permitted regional message delay. The precise guarantee below covers requests pending before that roster's preparation starts, plus independently completed subsequent pushes. This is sufficient for eventual inclusion: only finitely many different roster preparations can have started before one fixed request's receipt time in a locally finite execution.

## 2. What the frozen communication statement is used to mean

The frozen condition says “Within region i, the actual delay of a correct-to-correct message is delta <= Delta_i.” It does not restrict the endpoints to one Active engine, one beta, or one simultaneous AUTH roster. Retirement already applies the regional bound to managers/archives after worker termination. This candidate takes that **literal regional-domain reading**.

A historical correct owner and a future correct member of the same physical/logical region are therefore endpoints of that regional service while they have a protocol duty. No cross-region delivery bound is used. Giving the historical owner a persistent handler is an algorithm rule, just as keeping the existing closure handler after Terminated is an algorithm rule.

For target index r, let `d_i(r)` be the maximum of the finitely many regional delay parameters appearing in the authenticated snapshots through r. It is a conservative allowance for exchanges among the finitely many listed same-region owners and the target. This is not a uniform maximum over all future configurations. The claim uses the original same-region message bound; taking a maximum would **not** repair an interpretation under which old and new same-region endpoints have no communication guarantee at all.

This distinction must be retained in a manuscript integration. If “within region” was intended to mean “only between members of the very same voting snapshot,” this candidate does not derive the missing transport from the frozen model. Under that narrower interpretation it fails at a specific premise, and the model must not silently be strengthened. The present literal text and the existing manager/archive interpretation support the broader reading, but this is the main model-reading point to have independently checked.

The old size/load envelope also remains in force. Transfers below are a finite sequence of individually admissible complete messages. They do not charge an arbitrarily large pool, a history scan, or an unresolved storage import to one Delta. Concrete encoding and aggregate-load compatibility remain implementation obligations, identified in Section 10.

## 3. Durable state and a directory that does not chase a moving owner

For region i and authority index r, define

`D_i(r) = union of the regional identities in AUTH[k,0], 0 <= k <= r`.

The union is deduplicated by persistent node identity, and retains every historical authority needed to authenticate a response. For a fixed r it is finite. It is obtained from the complete authenticated authority history through r; no endpoint supplies an unauthenticated current-owner list. Recovery authorities change schedules only and add no ownership epoch. Beta is absent from all ownership keys.

Each required authority prefix is prepared as a finite task before the timed query stage. Its acquisition has no new uniform deadline. AUTH records for unselected regional rosters do not require those rosters to run BFT or to take custody of another node's queue. The directory merely includes identities that **might** hold requests. A listed identity with no pending request may give an empty answer or have completed physical retirement; waiting for every listed identity is unnecessary.

Each correct regional node runs the following service, separate from `(beta,r,g,t)`:

* `Pending`: canonical request bytes and identities, removed only on verified inclusion in G.
* `ReceiptJournal`: a persistent append-only sequence of first-received request records. Requests need not be removed from this journal when G includes them. The journal supports a retained prefix length/root and indexed reads without scanning the entire queue when a query arrives.
* `Cuts[(i,r,v)]`: the immutable finite journal prefix promised to target member v for target r, together with the authenticated target roster and outgoing transfer work. Repeated queries for the same key reuse the cut; beta never restarts it.
* `Targets`: every authenticated target roster for which a query has been processed. New requests are pushed to these rosters.
* outstanding per-request sends, query responses, and complete request-reception duties.

An accepted request is immediately persisted before acknowledging its receipt. The node then sends its full bytes to its own installed roster and every registered target roster. Receivers retain new complete requests under the same rule. Duplicate receipt preserves identity and existing queue position; it does not reset the journal entry or trigger unbounded relay loops. Deduplicate each `(request_id, target authority)` first multicast, while retaining unfinished sends.

The original owner retains the pending bytes even after one or all target rosters receive them. Consequently a future query never needs to discover a sequence of private transfers or chase a moving “current owner.” Public immutable AUTH order is enough. It is permissible to optimize copies later, but the proof here does not use such an optimization.

Requests first delivered to a member of a future AUTH are covered once that identity enters the relevant authenticated prefix. The request theorem uses a region that continues serving and hence reaches unbounded slot indices. It does not claim that an earlier roster discovers private requests at identities that are not yet members of any applicable authenticated roster.

## 4. Actual finite-cut acquisition

Each correct target member v performs one preparation for `(i,r,v)` outside every production attempt. Let N be the target roster and d = d_i(r).

First obtain the complete finite directory and the authentic target context. Prepare and transfer any large authority/context material as an explicit finite sequence of ordinary bounded messages. A responder must not be asked to perform an unbounded source fetch inside a timed query reply. Only after the complete context is available through the corresponding regional receive handler does the timed step begin.

At its own time s_v, v sends `CUT_QUERY(i,r,v)` to every identity in D_i(r). The query carries or has already delivered the complete authenticated target context. At first processing, each correct owner p:

1. registers N in Targets;
2. atomically retains its current finite ReceiptJournal prefix as `Cuts[(i,r,v)]`;
3. returns its signed descriptor `(i,r,v,p,length,root)` immediately from the maintained prefix metadata;
4. serves indexed full records from that immutable prefix on subsequent requests.

New arrivals after the cut append to the live journal and use the continuing push rule. They do not extend the descriptor and cannot keep the fixed transfer unfinished. A cut pins its finite index range, not bytes that cease to be pending after verified G inclusion. A correct indexed reply is either the still-pending full request or a signed tombstone for a request already in G. The hydrator treats a tombstone as an empty slot; it does not use that statement as proof of any new global acceptance. This is safe for coverage because a correct owner never tombstones a globally unaccepted request, while a Byzantine owner was already allowed to withhold its own slot. There is no need to hold all accepted bytes forever because a Byzantine requester never finished its snapshot.

At s_v + 2d, after processing all messages due at that time, v freezes its descriptor set: at most one complete descriptor per directory identity. Every correct owner that has a pending request and the required persistent handler has responded by this point. A silent identity contributes no descriptor. An equivocating Byzantine identity contributes at most the first complete locally accepted descriptor; different target members need not make the same choice.

The resulting vector has finite length and contains finite counts. Process its indexed slots in a fixed local order. For each slot, send an indexed record request and allow a full 2d query/reply interval, with message-first processing at equality. A valid complete response contributes its canonical request bytes. A missing, tombstoned, or invalid response consumes its finite slot and is skipped. Every correct owner's cut record that remains globally unaccepted is obtained. If one request plus authentication material needs multiple allowed messages, use explicit, individually bounded fragment slots and their complete processing; do not silently fit them into one exchange.

A Byzantine descriptor can lengthen this finite preparation considerably. It cannot make the chosen integer count increase while the transfer is running. This is an eventual-completion construction, not a practical uniform resource bound or a denial-of-service benchmark. A finite count is not a promise that it fits the old load profile; see Section 10 before adopting a concrete encoding.

At completion, `Pool_v` contains every still-pending request held by every correct directory owner at that owner's query-processing cut. In particular it contains each such request that was pending before s_v and remains globally unaccepted. No identification of correct owners and no wait for Byzantine cooperation was needed.

## 5. A completion certificate that gives all-correct awareness

The important additional step is after v's local acquisition, not an assumption that every other member acquired the same pool.

For every request in `Pool_v`, v sends the full request to **every** target identity. Use a finite per-request schedule within the existing message envelope. Every correct target manager persists and processes these packets even if it has not created an engine, has not finished its own acquisition, or has already archived another beta's worker. A received request becomes ordinary pending service data and is not a new BFT vote or authority.

Let b_v be the last actual complete send of this finite schedule. Only at or after b_v + d, with all simultaneous message processing first, may v sign

`HYDRATED(chain, i, r, hash(AUTH[r,0]), directory_prefix_commitment)`.

The signature asserts execution of the entire query, finite-transfer, and all-target dissemination procedure. It does not assert equality of private pools. v also retains its full pending pool and the target receive route. A target member collects distinct matching signatures of weight greater than F under the target snapshot. Correct target members alone can supply that weight after finitely many independent preparations.

**Hydration lemma.** Fix a certificate and one of its correct signers v. Every correct target member knows every globally unaccepted request held by a correct directory owner before s_v. The owner returned its actual immutable cut during v's deadline. The finite slot transfer includes its bytes in Pool_v. v's final dissemination delivered and processed all Pool_v bytes by b_v + d, before v signed. Any certificate containing that signature exists only afterward. A different correct target's own query may still be unfinished; its actual received bytes are already present.

There is no appeal to honest intersection. A certificate `{H0,B}` can have no correct intersection with a later report certificate `{H1,B}`. H0's compulsory completed multicast is precisely why H1 nevertheless has the requests.

Every correct target may adopt the valid HYDRATED certificate and stop redundant local acquisition once all its own already-promised serving duties are preserved. It need not identify the certificate's correct signer or fetch that signer's pool afterwards. The bytes are already at the recipient by the lemma. A mere pool root, callback, “stored by >F” acknowledgement, or certificate signed before the final dissemination wait would not establish this fact.

## 6. Ready, beta independence, and core timing

The target manager persists the completed hydration and full received pending requests under `(i,r)`. Ready requires this certificate in addition to the existing exact-anchor, authority, and native-state prerequisites. The first ReadyBundle supplies the bounded certificate and exact immutable target context. It contains no unresolved request-pool fetch. The existing atomic Ready/Enter behavior is otherwise preserved.

By the hydration lemma, every correct target already holds the necessary requests before the first correct Ready is possible. A recipient whose own acquisition was slower can use the certificate immediately. Thus this stage does not insert an arbitrary wait between receiving the first complete ReadyBundle and Enter, and does not alter the existing Delta entry-skew proof. Its variable preparation cost is paid before first Ready and retained across every beta retry.

In the stalled-G proof, r is fixed. Its directory is finite; each correct query fixes finitely many finite cuts; all per-request slots and disseminations finish; the correct-only HYDRATED threshold forms. Add this finite preparation to the existing fixed-anchor, AUTH-prefix, and public-history preparation maximum. No new term in the post-preparation regional response R_i is needed for pool hydration. An implementation may need to change constants for an additional bounded certificate check, but no whole-pool transfer belongs there.

Fresh arrivals use the separate push path. They do not invalidate HYDRATED or restart a completed Ready task. If a fixed roster keeps serving after hydration, a request received by a persistent owner whose query registered that roster is actually pushed to it within the applicable regional interval. If the owner was not yet registered, its response to a later roster query includes the request.

The candidate does not change source acceptance, Force cutoff, FINAL validity, or frozen publication scope. HYDRATED is regional preparation evidence. It does not receive a T_pub or T_E deadline as a bare record.

## 7. Arbitrary future rosters: finite-exception awareness proof

Fix a request x received at real time t_x by a persistently correct owner p of region i. Assume x remains globally unaccepted, stays admissible at the accepted parents under consideration, and fits the inherited DATA request envelope. Let k be a finite authority index placing p in the historical directory.

Consider infinitely many accepted serving sessions of region i. Their finite quotas imply unbounded slot indices. Discard the finitely many targets r < k. At every later target preparation whose correct certificate signer's query begins after t_x, p appears in D_i(r), remains a live pending owner, and places x in its cut. That signer's completed multicast makes all correct target members know x before HYDRATED can authorize Ready.

There are only finitely many target preparations whose first correct query began before t_x in a locally finite execution. A certificate for a target with an earlier-started correct signer may legitimately attest an older cut; charge that target to this finite exceptional set. At most finitely many accepted DATA can come from those targets because each installed slot has a finite quota. Beta retries do not increase that quota or create a new target hydration namespace.

Therefore all but finitely many of the accepted target-roster contributions have universal correct-member awareness of x. A roster need not intersect any earlier roster. No finite union of every future roster and no uniform future Delta is assumed. The original owner is the fixed discovery point, and the finite prefix directory is reconstructed from public immutable AUTH each time.

For a target that continues to serve within one installed roster, the push rule gives the analogous eventual awareness directly. The finite-exception argument is not a Zeno-execution result and does not assert a numerical request deadline.

## 8. Finishing the request argument with a changed selection rule

Hydration alone does not fix FIFO's accepted-Byzantine-proposer counterexample. Integrate it with the companion note's first-DATA report/primary rule, which the present task authorizes as a protocol change:

* each correct report is bound to the exact first-DATA parent and the actual entered term;
* it reports the least eligible pending request under the fixed finite-predecessor order, skipping accepted ancestry and checking admission at that parent;
* a valid first DATA carries reports of weight greater than F and executes the minimum nonempty reported primary first;
* other batch capacity can retain FIFO behavior; DRAIN gives no new request opportunity.

A report order such as `(canonical_encoding_length, canonical_encoding_bytes)` has finitely many predecessors for every fixed canonical request. It avoids relying on globally comparable first-receipt timestamps or on a finite union of different nodes' FIFO predecessor sets.

By Section 7, all but finitely many accepted term-first opportunities at the continuing region have correct report signers aware of x. Some correct signer in every report certificate then supplies an eligible candidate y <= x. The minimum primary z is no greater than y. Correct application and ancestry checks make z an actual fresh request in the accepted block. No logical primary can be fresh in accepted ancestry twice. Infinitely many distinct accepted z <= x are impossible because the order has a finite lower set. Hence x is included in G.

The accepted proposer need not be correct. Correctness enters through mandatory report validity, all-correct hydration, and the original correct signer in a FINAL receipt. The old progress proof still needs its explicit integration audit for report timing, application validity, and size/load; this note does not certify that audit by referring to it.

## 9. Late arrivals and physical exit

Worker Terminated never stops this owner service. That is already the structure used for closure, data-service, and publication owners. Requests arriving during a finite cut are retained in the live journal and pushed; they do not reopen the old cut. Requests arriving after a worker exits are handled by the same manager route. A whole node cannot release its only pending-request service while x remains globally unaccepted merely because a future roster's certificate exists.

The minimal candidate allows physical exit only when all of the node's pending requests have verified G inclusion, its existing archival/control/publication duties have an actual owner, and the dispatcher has drained every complete request already delivered to it. Atomic ingress shutdown precedes release. A complete admissible request received during shutdown is persisted and makes the pending-exit guard false again; it is not relabelled “unadmitted” and forgotten. After transport shutdown, a failed delivery to the departed endpoint is not evidence that the request reached a correct regional owner.

Correct clients retain their request until G inclusion and retry against authenticated current regional membership. This is a client algorithm, not a bounded ingress service. It does not strengthen the inherited premise that requests reach regions which continue serving. A client that forever sends only to a departed endpoint does not satisfy that premise. No cross-region bound or bare-request Espresso guarantee is inferred.

There is no guarantee of a physical exit while a node continues receiving new requests forever, while its region is permanently denied service, or while required old archival duties remain. None is part of the v3.7 program-termination theorem. For a node that has stopped ingress with a finite pending set and whose requests meet the continuing-service premises, Section 8 makes that set globally included after finite individual times; taking their finite maximum eventually discharges the request-specific exit guard.

This is the principal cost of avoiding a private owner-transfer oracle: historical owners remain discoverable while their pending bytes matter. Replacing this with finite-time physical withdrawal independent of inclusion would require a separate specified stable-route transfer protocol and its own proof. The present candidate does not assume such an external service.

## 10. Remaining obligations, rather than hidden assumptions

The abstract ownership/discovery proof is complete under the literal regional-domain reading and actual complete-message profile above. The following integration obligations remain explicit:

1. **Mixed-snapshot scope.** Independently confirm that the frozen universal within-region communication statement applies to persistent historical owners and future roster members in the same region. If it is narrower, this construction is conditional at precisely that point; do not silently amend the environment.
2. **Physical lifecycle.** Define correct ownership as continuing the described pending handler until global inclusion or an actually implemented alternative. Existing Terminated already does not authorize stake withdrawal. A deployment that permits physical disappearance despite outstanding pending duties is outside this candidate's correct behavior, rather than repaired by its certificate.
3. **Finite preparation mechanics.** Implement retained prefix descriptors and indexed request reads, supply complete authority context before timed replies, and ensure finite per-request slot schedules. No scan, unresolved fetch, or unbounded database import is charged to d.
4. **Original resource envelope.** Give an encoding and dispatch discipline that fit the old network, DATA, and complete-result size/load conditions. Finite transfer proves eventual completion in the abstract synchronous model; it does not establish practical overload resistance, bounded storage, report-witness headroom, or a concrete codec. Concurrent preparations must not manufacture aggregate load exceeding the unchanged declared envelope.
5. **First-DATA integration.** Recheck the report gate, immediate original ANCHOR vote, request preimages, exact-parent admission, first-DATA deadline, FINAL intrinsic validity, and public-kernel reliance. The companion timing analysis is evidence for this work, not an independent whole-protocol verdict.

If the desired target additionally requires old owners to vanish before request inclusion, a uniformly bounded history directory, or a uniform wall-clock request latency across all future AUTH configurations, this candidate does not supply it. Those requirements are separate from the requested safety and eventual-liveness goals under the frozen environment.

## 11. Short Chinese explanation

这个方案把“出块 worker 退出”和“仍持有 pending request 的节点服务退出”分开。只要 x 还没进 G，最初收到 x 的正确 regional owner 就保留完整字节和查询服务。新 roster 从不可变 AUTH 历史枚举本区域历史节点，逐项获取有限快照，不需要猜当前 mempool owner，也不需要旧新 roster 有诚实交集。

一个正确新节点先取完自己的有限快照，再把实际字节发给新 roster 的所有成员，等一个已有区域传播界限，才签 HYDRATED。`>F` 证书只保证有一个正确签名者，但这个签名者已经完成了向所有正确成员的传播，因此后面的 REPORT quorum 无法避开“已知 x 的正确成员”。这里靠的是实际传播，不是错误的 quorum-intersection 推理。

新请求不不断扩大旧快照，而走独立 push。固定 x 到达之前启动过的 roster 准备任务只有有限个；之后的新 roster 查询都会找到仍保留 x 的原始 owner。因此允许成员任意换新，也只会有有限个尚未获知 x 的旧准备结果。再把首个 DATA 改成强制执行 REPORT 里最小的有限前驱序请求，就能把“区域反复获得 DATA 服务”推进为“x 最终进入 G”。

代价是：仍有 pending request 时，原始 owner 的请求服务不能擅自物理消失。worker 可以退出；物理退出要等请求进入 G，或另行实现并证明真正的接管协议。该方案没有给 bare request 增加 Espresso 发布保证，也没有改 Force 的严格 cutoff。

Validation: source-level construction and local proof analysis only. No main/fragments/frozen files edited; no compile, model check, concrete transport benchmark, or whole-protocol verification claimed.
