# v3.9 ownership candidate v2: historical owners and explicit finite hydration

Status: revised constructive candidate for a new independent review, not a completed protocol-verification report. The broad mixed-snapshot communication service remains an explicit conditional premise, not a guarantee established from frozen v3.1. This file changes no manuscript or frozen assumption. It gives an actual discovery, transfer, and admission procedure. Its essential choice is that a node that has received a still-pending request keeps its node-level request service until global inclusion. A production worker may terminate while that service continues. An independent physical-departure deadline is not promised.

Revision basis: the unchanged `reviews/v3_7/v38_ownership_candidate.md` (SHA-256 `047741b4c05cd6cacf9d638027270db2f9935d7f1f9f5fab26b831ef91844680`) and the fresh conditional audit `reviews/v3_8/v39_fresh_ownership_review.md` (SHA-256 `8d2d4fb88ec455cd37014a37f0cd8f97c07d87ece4106f1436b7fbecd6ac8379`). That audit found two procedure-level justification gaps: a missing fixed fragment budget and missing HYDRATED-share dispatch/collection. Sections 4 and 5 instantiate those procedures. This standalone successor preserves the original's ownership, finite-exception, lifecycle, and conditional request argument. No original candidate, manuscript, or frozen source is modified. The original source references remain the v3.1 frozen Model, v3.7 request/retirement/liveness text, and its companion ingress/report notes; the fresh audit additionally checks the corresponding current v3.8 request, retirement, liveness, and ReadyBundle sources.

## 1. Concrete outcome and its exact boundary

Use an immutable, AUTH-indexed directory of **historical regional request owners**, and copy their finite retained cuts into each new roster before its first Ready. Do not replace the directory by the latest roster, and do not transfer the only pending copy merely because a new roster acknowledged it.

There is no new consensus protocol. AUTH already fixes the order and the membership of the directory. Different correct hydrators may acquire different supersets. Their certificates need agree only on the target authority and completed procedure, not on a unique mempool root.

The useful completion rule is:

> A correct target member signs HYDRATED only after its own finite query has obtained every correct historical owner's cut, and after it has actually disseminated that acquired pool to every target member and allowed the existing regional delivery interval to elapse.

A HYDRATED certificate of weight greater than the target's F contains one correct signer. That signer's actual completed multicast, rather than intersection with any later certificate, makes every correct target member aware of the required requests. This removes the need for the synchronous descriptor-agreement construction in the earlier fixed-pair note.

The cut cannot mean “every request delivered anywhere just before the first DATA is accepted.” A request received by an old owner an arbitrarily small time before acceptance need not have traversed even one permitted regional message delay. The precise guarantee below covers requests pending before that roster's preparation starts, plus independently completed subsequent pushes. This is sufficient for eventual inclusion: only finitely many different roster preparations can have started before one fixed request's receipt time in a locally finite execution.

## 2. Conditional communication domain and unchanged resource boundary

For this candidate only, grant the following **broad mixed-snapshot service premise**: a persistently correct historical owner and a correct future member of the same region have bounded complete-message delivery and processing while their duties continue, with the finite-prefix allowance `d_i(r)` used below. This premise is **not derived here from frozen v3.1**. Its within-region wording does not explicitly determine disjoint-snapshot endpoints; the existing archive/manager rules establish phase independence within their fixed snapshot, not the missing mixed-snapshot edge. No model clarification has arrived, and this note makes no change to the frozen environment.

Conditioned on that service domain, let `d_i(r)>0` be the maximum of the finitely many applicable regional delay allowances in the authenticated authority prefix through r. The conditional service must cover the actual historical-owner/target exchanges under this allowance. A finite maximum cannot create a communication edge absent from the premise. This is neither a cross-region bound nor a maximum over all future configurations.

Continued original-owner storage and dispatch are protocol duties of a **persistently correct** owner. The argument starts after that owner's actual complete receipt of x; it does not establish that some arbitrary old machine stays available, that a correct client reaches such an owner, or that a failed client retry succeeds. A correct owner's physical exit guard is specified in Section 9. The existence of the required serving cohort and the broad service above remain conditional premises; a new signature cannot prove them.

Fix a configured chunk payload `b>0` for the abstract transfer procedure. Its complete authenticated envelope, including the metadata fields below, must be admissible under the **unchanged** complete-message size/processing profile. Small control records (CUT descriptor, RECORD_META, fragment query, fragment reply, HYDRATED share) likewise use that existing abstract interface. This parameterization supplies a finite slot schedule; it does not assert that any particular binary encoding has spare room for those fields. If a deployment cannot realize such envelopes under its old profile, this candidate is not implemented there. Increasing a size limit, assuming headroom, or upgrading the publication guarantee is not an allowed repair.

Per-message processing under the granted interface includes the prescribed authentication, indexed storage, and incremental fragment bookkeeping. Any separate whole-object validation or local preparation is performed as actual terminating work before the hydrator marks its pool complete and begins the final dissemination stage. It is not assigned an unproved Delta deadline. Hydration establishes availability of complete retained request bytes at recipients; exact-parent application admission, request-order computation, augmented ReadyBundle validation, and the report gate retain their separately stated integration obligations. If a concrete receive implementation performs a whole-record scan at the last chunk, that cost must be realized within its old complete-dependency processing interface or moved into actual preparation; merely waiting `2d` does not bound that scan.

The old aggregate load envelope also remains in force. Transfers are explicit finite sequences of individually admissible messages, not an arbitrary pool or history scan charged to one delay interval. Section 10 keeps codec, storage, aggregate dispatch, and report/Ready integration limitations open.

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

## 4. Actual finite-cut acquisition and immutable fragment budgets

Each correct target member v performs one preparation for `(i,r,v)` outside every production attempt. Let N be the target roster and d = d_i(r). Only an authenticated query by the named target member v can create its cut; a different sender cannot pre-create v's cut by copying its identity into a packet.

First obtain the complete finite directory and authentic target context. Any large context is transferred with a fixed finite byte length and chunk count by the object procedure below, followed by actual terminating local validation. A correct query must supply all authentication dependencies needed by a historical owner; an unresolved source fetch is not inside a timed reply. Begin the CUT_QUERY deadline only after the complete context has reached the relevant correct handlers under the conditional service. This finite preliminary transfer is retained, not restarted by beta.

At its own time s_v, v sends `CUT_QUERY(i,r,v)` to every identity in D_i(r). At first processing, each correct owner p:

1. registers N in Targets;
2. atomically retains its current finite ReceiptJournal prefix as `Cuts[(i,r,v)]`;
3. immediately returns its signed descriptor `(i,r,v,p,m,root)` from the maintained prefix metadata, where m is the finite record count;
4. serves the indexed records of that immutable prefix using the following metadata and fragment handlers.

New arrivals append to the live journal and use the continuing push rule. They never increase this cut's m. At `s_v+2d`, after processing all messages due at that time, v freezes at most one complete valid descriptor per directory identity. A silent or malformed source contributes none; for an equivocating source use the first locally accepted complete descriptor. Later descriptors, larger counts, and alternative roots never replace it. Correct owners with still-pending requests respond under the granted service; Byzantine cooperation is unnecessary. Different hydrators need not choose the same Byzantine descriptor.

Process the frozen finite descriptors and their record indices in deterministic local order. Here `cut=(chain,i,r,v,p,m,root)` includes the frozen owner descriptor and exact target context. For each `(cut,p,j)` with `0<=j<m`, execute exactly this bounded-budget procedure:

1. **Metadata slot.** Actually send `RECORD_META_QUERY(cut,p,j)`, and open one `2d` request/reply slot. A correct p replies with either a signed tombstone for an already globally included journal entry, or a signed `RECORD_META(cut,p,j,L,H)` containing the exact finite encoded byte length `L>=0` and binding digest H of the immutable canonical record. Its representation includes the request and any fixed record-authentication material required for local reconstruction. The metadata envelope is complete under the existing admitted interface. The requester freezes the first valid same-context metadata received by the slot deadline. A missing, malformed, or tombstoned response ends this record slot without a request. A metadata change never restarts it.
2. **Fixed fragment count.** From that frozen L, compute once `n=ceil(L/b)`. This is a finite integer; for `L=0`, n is zero and proceed directly to final checking. No sender can append fragments, enlarge L or n, replace H, or request a retry under this record key.
3. **Fragment slots.** For each `h=0,...,n-1`, actually send `FRAGMENT_QUERY(cut,p,j,L,H,h)`. Open one `2d` request/reply slot. The signed reply binds exactly `(cut,p,j,L,H,h)` and its bytes. Require payload length b except for the last payload, whose length is `L-b(n-1)`; accept at most one matching valid payload for this h. Process due messages before expiry. Unauthenticated, wrong-sender, wrong-context, and out-of-slot packets are ignored and cannot terminate an honest owner's slot. A malformed or conflicting reply signed by the expected p for this exact active context marks the record failed; absence of a valid matching reply at expiry also fails it. The hydrator may stop the remaining scheduled slots on failure or consume them and ignore their results, but it may not add slots or restart. Late fragments are ignored for this completed attempt.
4. **Final local checking.** After the n fixed slots, successful reconstruction must have exactly L bytes and digest H, with the required canonical request encoding and authentication. Perform the actual terminating local checks now; no new external fetch or sender response is awaited and no assumed Delta bound is assigned to this work. An invalid or incomplete record is skipped. A valid record enters the finite acquired pool. Application admission against a future exact parent is not purportedly decided at this old-owner cut.

Thus a record consumes at most one metadata slot and n fragment slots, each with its fixed `2d` timeout, plus its actual finite local checking. A Byzantine may choose a very large but finite L in a complete admitted metadata envelope, just as it may choose a large finite m. That can cause large finite work, not a moving or infinite attempt count. Neither m nor any frozen L can expand. This is an eventual-completion claim, not a uniform complexity, storage, or practical denial-of-service bound.

A correct owner supplies the actual fixed metadata and every requested fragment of a still-pending request. For such a request every complete reply is processed within its slot, so the exact bytes reconstruct and pass the actual local checks. If G includes the request during transfer, the correct owner may return a signed tombstone instead; that terminates the record and is harmless because the coverage theorem concerns requests that remain globally unaccepted. A tombstone is never used as evidence of a new global acceptance. Byzantine tombstones are no stronger than the omission already permitted from a Byzantine source. Cuts pin a finite index range, not globally accepted request bytes forever.

All sends and continued slot progression are algorithm actions, stored as durable outgoing duties. On finishing or timing out a slot, a correct participant schedules the next fixed slot or its finite local check; it does not await an unsolicited retry, Byzantine permission, or a new beta. New arrivals do not alter this finite job. This specifies continued local execution of the protocol; it supplies no new environmental fair-ingress service and does not override the old admitted aggregate-load condition.

At the end, v freezes `Pool_v`. It contains every still-globally-unaccepted request held by each correct directory owner at that owner's query-processing cut, in particular every such request pending before s_v. The pool is not extended by concurrent live arrivals. Any required preparation or validation of its own records completes before the final dissemination below. A separately received live request still uses the continuing push path.

The same object encoding applies to a finite authority/context object or an outgoing pool record: fix its exact finite L and H before dispatch; derive n once; use exactly those metadata/chunk positions and no dynamically growing fragment list. Context transfers finish, and their actual local validation completes, before the timed cut query that depends on them. Final pool dissemination has no request-dependent missing-data fetch after HYDRATED.

## 5. Completed dissemination, explicit share collection, and strict ordering

The following is a finite operation on the already frozen `Pool_v`, not a wait for every other member to finish its private acquisition.

For every record in that pool, v sends its full bytes to **every** target identity using the fixed metadata and n-chunk encoding just specified. For multicast, the already-held sender needs no query: it actually sends the complete signed metadata and the n complete signed fragment envelopes to every target. Recipients deduplicate by sender, cut/target context, record identity, and fragment index; they retain fragments that arrive out of order and reconstruct the fixed-length object. The pool and each record's counts are frozen. Correct-recipient per-message authentication, storage, and fragment bookkeeping are covered only by the granted old complete-message profile. No uncompleted local validation at v or merely queued transfer job counts as an actual completed send.

Every correct target manager receives and durably retains these packets even without an engine, with unfinished local acquisition, or after a different beta's worker has exited. The resulting complete raw request is ordinary pending-service data. This transfers bytes and their validation material; it does not create a BFT vote, grant authority, or certify application admission at a future parent. Receiver-side whole-record processing must obey the scope in Section 2; no unspecified final scan is assumed to fit by waiting longer.

Let `b_v` be the last **actual** complete envelope send in this finite multicast, including metadata and every fragment to every target. For an empty pool, let b_v be the time v completes its acquisition and final local preparation. Only at or after `b_v+2d` may v create its signed share of

`HYDRATED(chain, i, r, hash(AUTH[r,0]), directory_prefix_commitment)`.

Since `d>0`, every required complete envelope is delivered and processed by `b_v+d < b_v+2d`. The all-target byte-transfer completion therefore strictly precedes the signature event. This does not depend on the order of two handlers at different nodes at an equal clock timestamp. The extra d is a conservative pre-Ready protocol delay, not an enlarged network or final-validation assumption.

**Share dispatch and certificate collection are mandatory actions:**

1. At share creation, v durably stores its signed share, inserts it into its own collector, and immediately executes an authenticated multicast of the full signed share to every target member. Each per-target send is an actual algorithm action; the multicast is not completed by merely storing a future job. Any unfinished sends remain persistent duties across worker exit, beta changes, and storage handoff until dispatched. Creating a signature without sending it does not complete this action.
2. Each correct target manager validates a received share's signature, signer membership, exact common context, and target voting snapshot. It caches at most one valid matching share per distinct signer. Duplicates do not add weight. Shares for another authority or directory prefix remain separate.
3. As soon as the cached same-context weight exceeds F, the holder materializes and persists the full certificate containing those signed shares. At its first possession of a complete valid certificate for this context, it multicasts that full certificate to every target member. Every recipient independently verifies the signatures, distinct members, and weight; persists the complete certificate; and performs the same once-per-context relay. A root or certificate pointer is insufficient. If the full certificate needs transport fragmentation, use its frozen exact length and the same finite object procedure; the materialized logical certificate is still the actual signed-share set.
4. A certificate holder makes the complete certificate available to its local Ready manager and preserves its receive/relay duties. No share or certificate receives a bare-record T_pub or T_E guarantee.

**Formation lemma.** Each correct target's finite acquisition, local checking, finite multicast, and `2d` guard finish under the granted continued-handler and message premises. At that time it actually sends a valid share to every correct target; the complete-message service delivers it. There are finitely many correct target members, so take a finite maximum of their send/delivery times. By then every correct collector has all correct shares. Their weight is `W-B >= W-F > F`, hence each can form a certificate without any Byzantine share. The first full-certificate relay can only make adoption earlier. No independent fair-ingress or external collection service is invoked.

**Hydration lemma.** Fix a valid certificate and one of its correct signers v. Each request that was pending at a correct directory owner before s_v and remains globally unaccepted occurs in v's finite acquired pool. That owner supplied a fixed metadata/fragment schedule with the real bytes. v completed its actual local checking and all-target multicast before signing. Consequently every correct target has the full required bytes by `b_v+d`, strictly before v's share and therefore before any certificate containing it. Another correct member may still be acquiring a different local superset; the actual required bytes are already retained there.

No honest quorum intersection is used. A HYDRATED certificate `{H0,B}` and a later report certificate `{H1,B}` may intersect only at B. H0's completed multicast is why H1 nevertheless holds the required requests.

A correct target may adopt a valid complete certificate and stop redundant local acquisition while preserving every already-promised serving, outgoing-share, certificate-relay, and storage duty. It need not identify the correct signer or fetch that signer's pool after adoption. If no certificate exists, this cancellation rule cannot stop all correct preparations: each continues its finite job and mandatory share broadcast. A mere pool root, local signature, stored-by-`>F` acknowledgement, or premature share does not establish the lemma.

## 6. Ready, beta independence, and core timing

The target manager persists the completed hydration and full received pending requests under `(i,r)`. Ready requires this certificate in addition to the existing exact-anchor, authority, and native-state prerequisites. The first ReadyBundle supplies the complete certificate and exact immutable target context; its augmented size and validation still have to fit the existing ReadyBundle complete-message interface. It contains no unresolved request-pool fetch. The existing atomic Ready/Enter behavior is otherwise preserved.

By the hydration lemma, every correct target already holds the necessary requests before the first correct Ready is possible. A recipient whose own acquisition was slower can use the certificate immediately. Thus this stage adds no unresolved whole-pool transfer after first Ready. Conditional on the augmented ReadyBundle meeting its existing complete-processing interface and the separate report/validation obligations, the original Delta entry-skew proof retains its shape; the larger historical d is spent before Ready and does not replace the target snapshot's post-entry Delta. Its variable preparation cost is paid before first Ready and retained across every beta retry.

In the stalled-G proof, r is fixed. Its directory is finite; each correct query fixes finitely many finite cuts; each record fixes its finite metadata/fragment budget, actual local checks and multicasts finish, and the mandatory correct-share broadcasts make the HYDRATED threshold form. Add this finite preparation to the existing fixed-anchor, AUTH-prefix, and public-history preparation maximum. No new term in the post-preparation regional response R_i is needed for pool hydration. An implementation may need to change constants for an additional bounded certificate check, but no whole-pool transfer belongs there.

Fresh arrivals use the separate push path. They do not invalidate HYDRATED or restart a completed Ready task. If a fixed roster keeps serving after hydration, a request received by a persistent owner whose query registered that roster is actually pushed by the same fixed finite metadata/chunk procedure. This gives eventual complete delivery under the conditional message and continued-dispatch premises; a multi-envelope object is not assigned a single-d total transfer bound. If the owner was not yet registered, its response to a later roster query includes the request.

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

The revised procedures discharge the two specified missing-action/budget obligations at the abstract level and are submitted for fresh independent verification. The ownership/discovery conclusion remains conditional on the broad mixed-snapshot service and actual complete-message profile; neither is established by editing this candidate. The following integration obligations remain explicit:

1. **Mixed-snapshot scope remains unresolved.** The broader service in Section 2 is granted only to evaluate this candidate. Neither the frozen within-region sentence nor phase-independent fixed-snapshot retirement by itself proves that domain. A finite maximum of delay parameters does not supply it. No frozen-model amendment or clarification is claimed.
2. **Physical lifecycle.** Define correct ownership as continuing the described pending handler until global inclusion or an actually implemented alternative. Existing Terminated already does not authorize stake withdrawal. A deployment that permits physical disappearance despite outstanding pending duties is outside this candidate's correct behavior, rather than repaired by its certificate.
3. **Finite preparation implementation.** Realize the now-specified immutable descriptors, metadata lengths, exact fragment budgets, indexed reads, terminating local checking, mandatory outgoing-share duties, and collectors. Supply complete authority context before timed replies. No scan, unresolved fetch, or unbounded database import is charged to d; no representation may silently expand the frozen budget.
4. **Original resource envelope.** Give an encoding and dispatch discipline that fit the old network, DATA, and complete-result size/load conditions. Finite transfer proves eventual completion in the abstract synchronous model; it does not establish practical overload resistance, bounded storage, report-witness headroom, or a concrete codec. Concurrent preparations must not manufacture aggregate load exceeding the unchanged declared envelope.
5. **First-DATA integration.** Recheck the report gate, immediate original ANCHOR vote, request preimages, exact-parent admission, first-DATA deadline, FINAL intrinsic validity, and public-kernel reliance. The companion timing analysis is evidence for this work, not an independent whole-protocol verdict.

If the desired target additionally requires old owners to vanish before request inclusion, a uniformly bounded history directory, or a uniform wall-clock request latency across all future AUTH configurations, this candidate does not supply it. Those requirements are separate from the requested safety and eventual-liveness goals under the frozen environment.

## 11. Short Chinese explanation

这个方案把“出块 worker 退出”和“仍持有 pending request 的节点服务退出”分开。只要 x 还没进 G，最初收到 x 的正确 regional owner 就保留完整字节和查询服务。新 roster 从不可变 AUTH 历史枚举本区域历史节点，逐项获取有限快照，不需要猜当前 mempool owner，也不需要旧新 roster 有诚实交集。

一个正确新节点先取完自己的有限快照，再把实际字节发给新 roster 的所有成员，在最后一次实际发送后等 2d，才签 HYDRATED；已有传播界限 d 保证完整字节在签名之前严格到达。签名份额必须广播给全体目标成员，成员缓存同一上下文的 >F 份额后形成并继续广播完整证书。`>F` 证书只保证有一个正确签名者，但这个签名者已经完成了向所有正确成员的传播，因此后面的 REPORT quorum 无法避开“已知 x 的正确成员”。这里靠的是实际传播，不是错误的 quorum-intersection 推理。

每条记录先固定签名元数据里的字节长度 L 和摘要，再固定 n=ceil(L/b) 个片段槽；每槽都有 2d 截止时间，拜占庭发送者不能不断加长或重启任务。重组后的实际本地检查在准备阶段完成，不假装有新的 Delta 上界。新请求不会扩大旧快照，而走独立 push。固定 x 到达之前启动过的 roster 准备任务只有有限个；之后的新 roster 查询都会找到仍保留 x 的原始 owner。因此允许成员任意换新，也只会有有限个尚未获知 x 的旧准备结果。再把首个 DATA 改成强制执行 REPORT 里最小的有限前驱序请求，就能把“区域反复获得 DATA 服务”推进为“x 最终进入 G”。

代价是：仍有 pending request 时，原始 owner 的请求服务不能擅自物理消失。worker 可以退出；物理退出要等请求进入 G，或另行实现并证明真正的接管协议。该方案没有给 bare request 增加 Espresso 发布保证，也没有改 Force 的严格 cutoff。跨不同 roster 的区域通信仍是本候选方案的条件前提，尚未从冻结 v3.1 模型得到证明。

Validation of this revision: read the fresh conditional audit; explicitly repaired its fragment-budget and share-dispatch gaps; manually checked fixed-count termination, tombstones, strict pre-signature byte delivery, correct-only certificate formation, beta persistence, and Ready/report scope. An independent review of this v2 remains pending. No main/fragments/frozen files edited; no compile, model check, concrete transport benchmark, or whole-protocol verification claimed.
