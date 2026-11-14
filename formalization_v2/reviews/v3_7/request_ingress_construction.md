# Pending-request ownership and finite-cut hydration

Status: constructive design analysis only. No manuscript amendment, FIFO decision, new publication guarantee, or complete request-liveness theorem is adopted here. This note accompanies `request_report_scope_analysis.md`. Its result is a concrete fixed-pair handoff construction and an exact statement of what remains missing when that construction is composed across arbitrary future rosters.

All source references below are to `snapshot/`: `main` means `sync_hotstuff_rr_part2_rotation_v3_7.tex`; `retirement` means `sections/rotation_v3_4_retirement.tex`; `bundle` means `sections/rotation_v3_5_ready_bundle.tex`; Part I means `baseline/sync_hotstuff_rr_part1_core.tex`.

## 1. What can and cannot be concluded

A correct node can be required to keep a pending-request owner after its BFT worker exits. That is a protocol lifecycle rule, rather than necessarily a new environmental assumption. The owner must retain full request bytes, immediately relay them when the applicable recipient roster is known, and release its last copy only after global inclusion or an actual completed transfer. A callback, URI, unacknowledged send, or transfer to an unresolved retrieval task is insufficient.

The regional network assumption is **not expressly restricted to Active workers or the latest AUTH**: it says correct-to-correct messages within region `i` take at most `Delta_i` (main:392–404). The existing retirement proof explicitly applies that bound to a historical snapshot's manager/archive even when there is no worker or there is an exit tombstone (retirement:183–193, 196–210). Therefore worker termination alone is not an argument that its node-level owner is unreachable. Main:2141–2151 also retains pending requests and explicitly separates termination from stake withdrawal.

Nevertheless, the frozen text does not identify a complete communication and discovery interface between an old owner roster and an arbitrary disjoint future roster. In particular, it does not specify which regional bound applies to a mixed-snapshot exchange if the authority changes endpoints or `Delta_i`, how a fresh member discovers the current pending owners, or how late client arrivals find an owner after physical-node retirement. These omissions cannot be filled merely by naming an `AcquirePending` operation.

The construction below closes the *finite queue-cut/completeness* problem for a known old/new pair whose participants are covered by one existing regional communication interface. It shows why Byzantine withholding and the absence of honest quorum intersection do not make that local problem inherently impossible. It does not yet derive that communication interface, globally discoverable ownership transitions, or the integrated size/timing profile from the current manuscript.

## 2. Separate the owner from a production attempt

Give each correct node a durable pending-request service, outside individual `(beta,r,g,t)` programs. Its records include the canonical request bytes and identity, first local receipt position, whether inclusion in `G` has been verified, and any outgoing transfer obligation. DATA in a discarded suffix never discharges an obligation. Changing beta does not reset the owner or its completed transfer work.

On first receipt of an admissible request `x`, the owner persists `x` and dispatches its full bytes to every member of each currently installed regional recipient roster for which it has a delivery obligation. “Every member” is operationally enumerable from the authenticated snapshot; the code does not try to identify correct members. Correct recipients persist and relay the request. These are client-data actions and do not create a BFT view, signature, timer, admission permission, or publication result.

If one such correct owner and every correct recipient are within the applicable regional interface, the owner's complete send at `s` makes every correct recipient aware by `s+Delta`. A certificate of weight `>F` formed from reports **after that dissemination barrier** therefore contains an aware correct signer. A certificate formed before it need not.

This argument uses direct dissemination to all correct recipients. It does not use honest intersection of two certificates. For example, `{H0,B}` may acknowledge storage while `{H1,B}` later reports; the intersection may be only Byzantine `B`. A storage acknowledgement of weight `>F` is not by itself the dissemination barrier.

## 3. A concrete finite cut for one old/new roster pair

Let `O` be a known old owner roster and `N` a known target roster. Their authenticated membership and public keys are available before the procedure starts. Let `U=O union N`. The following construction is conditional on all correct participants in `U` being reachable through the applicable original regional interface throughout this finite procedure. Write its applicable delay as `Delta_*`; this notation does not grant a new cross-region or mixed-snapshot bound.

The procedure has a shared identifier, target authority, and scheduled start time `T`. All correct participants obtain that complete start context before `T`. Establishing this common context without a trusted coordinator is part of the ownership-transition control plane discussed below; it must not be assumed from independently received, possibly equivocating `START` messages.

At `T`, each correct old owner atomically cuts its pending queue into a finite immutable snapshot `S_p` and a later-arrival stream. The snapshot contains every locally retained, not globally included request at that instant. Locally finite execution gives a finite snapshot at a finite time; it does not give a uniform size bound. Subsequent arrivals never change `S_p` or its length.

### 3.1 Common finite descriptors without waiting for Byzantine nodes

Run one synchronous authenticated-broadcast instance per old identity for its signed descriptor `(cut_id, target, count, snapshot_digest)`. Use an implementation that delivers the same descriptor or the same failure value at every correct participant. A correct sender's descriptor is delivered intact. Byzantine omission or equivocation produces a bounded-time failure outcome, rather than a wait for the sender to cooperate.

There is a direct signed-round implementation: in a population of `m=|U|`, use `m` rounds, round numbers bound to the cut, and chains of distinct signatures accepted only at the matching round. Correct recipients relay newly admitted values with their signature. A value first admitted in the final round has a chain of `m` distinct identities, hence includes a correct signer that already relayed it; a value admitted earlier is relayed by a correct recipient in time. Consequently a purported value cannot remain known to only one correct recipient at completion. All correct participants use the same singleton-or-failure output rule. A concrete implementation must bound equivocation/flood handling, signature-chain bytes, and per-round load; the existence of this logical construction does not establish that implementation's old-profile resource bounds.

After the descriptor instances finish, every correct participant has the same finite descriptor vector. Invalid encodings and missing descriptors are failure entries. A Byzantine sender may choose a very large finite count; this can make preparation very long, but cannot turn a fixed finite descriptor into an endlessly expanding snapshot. A count cap would be an additional capacity choice and is not silently assumed here.

### 3.2 Bounded per-request transmissions

The descriptor vector determines a finite public sequence of slots `(old_identity, index)`. In each slot, run authenticated broadcast for at most one full request and its snapshot-membership material. A valid request matching the descriptor is added to the common cut pool; failure is skipped. Every correct old owner's snapshot requests appear in their specified slots. Byzantine withheld bytes expire in their slots and do not block the next slot.

The pool transfer is therefore a finite succession of per-request transmissions. It is not an entire-pool query charged to `2Delta`, nor an assumption that an arbitrarily large queue fits one message. Request bytes and authentication material must obey the actual existing message/processing profile; if they require fragmentation, every fragment round must be explicit in the preparation schedule. No fixed post-entry deadline may absorb these rounds.

At the deterministic end of this finite schedule, all correct members of `N` have the same delivered pool, including the full snapshot of every correct member of `O`. Completeness here is a protocol theorem about correct senders and synchronous rounds. It is not an assertion that a recipient can identify which old members are correct. Byzantine snapshots may be incomplete or absent; this does not delete any request retained by a correct old owner at `T`.

### 3.3 The certificate must attest the completed procedure

A correct new member signs `HYDRATED(cut_id, O, N, descriptor_digest, pool_digest, completion_round)` only after completing the entire prescribed procedure. Correct new members alone have weight `>F_N`, so a certificate can form without Byzantine cooperation.

This certificate differs materially from “a quorum stores `x`.” A correct signature attests completion of a synchronous protocol whose completion theorem gives the bytes to **all correct new members**. Thus a certificate with a correct signer establishes that stronger fact under the stated communication contract. It needs no honest intersection with a later report certificate.

The certificate does not independently prove a network delay assumption. If some new member was not covered by the joint exchange or was allowed to join only afterwards, the completion theorem does not apply to that member. A signature cannot repair that missing premise.

## 4. Old/new actor lifecycle and arrivals after the cut

The snapshot cut is not an ingress shutdown. If old owner `p` receives `x` after `T`, it must place `x` in its later-arrival stream, retain it, and send the full request to the designated new service. It cannot discard `x` because the snapshot was already signed.

A safe per-request discharge rule is:

1. A correct new recipient persists the full request and immediately relays it to all members of `N` before acknowledging it.
2. The old owner obtains context-bound storage acknowledgements of weight `>F_N` at its own local time `a`, and retains the request through `a+Delta_*`. At least one correct acknowledger dispatched its full relay before sending the acknowledgement, hence before `a`; the extra interval covers that relay. Alternatively the owner obtains a certificate from a completed dissemination round with the same all-correct conclusion.
3. The new service accepts the durable future ownership obligation before the old owner releases its last required copy.

The correct signer guaranteed by the threshold supplies an actual relay; the subsequent regional interval, not quorum intersection, makes all correct new members aware. The new recipient must remain an owner across its own worker/beta changes.

This rule can finish a fixed transfer while new arrivals continue, but **physical retirement still needs an ingress-routing cut**. If requests may continue to arrive at the old address forever, “wait until the tail is empty” need not finish. A protocol must specify when the old endpoint stops admitting new deliveries, what successor record it returns during draining, how already delivered/in-flight requests are preserved, and how a correct client with a stale address discovers or retries at the successor. Treating a fully received request as “not admitted” does not authorize forgetting it under the existing retention rule.

Possible mechanisms are a stable regional ingress route whose ownership is atomically transferred, or a client protocol that resolves authenticated successor records and retries until a correct owner acknowledges the full request. Either is a substantive interface amendment. Neither is stated by the current phrase “requests reach the regions that continue serving.” A new trusted mempool or bounded Espresso publication of bare requests is unnecessary, but a concrete routing mechanism is necessary.

Keeping an old owner until its finite acknowledged duties finish is a correct-behavior obligation. Requiring an arbitrary external host to remain online despite an allowed physical departure, or requiring communication across endpoints outside the existing regional domain, would instead strengthen availability. These must not be conflated. This note requires no old BFT engine or old quorum to vote forever; it also does not claim that physical departure has already been implemented merely because program termination exists.

## 5. Discoverability and composing ownership epochs remain separate work

The fixed-pair procedure assumes that `O`, `N`, and a common cut start are known. To compose it across arbitrary future membership, the system needs an authenticated ownership history independent of beta and a nonforking rule for replacing the current owner set. New members must be able to locate the current owners from already public authority information, or from a durably discoverable chain of successor records. Correct old owners must not acknowledge incompatible successor transitions.

The threshold `>F` does not itself provide that nonforking rule: two transition certificates can have disjoint correct signers. One must use an actual synchronous agreement/dissemination procedure for ownership metadata, or derive a unique transition from existing authenticated history. “Take whichever owner certificate arrives first” is not sufficient.

Likewise, requiring every intermediate AUTH roster to take ownership can block progress if an intermediate roster never becomes an available service. Skipping it needs an explicit authenticated transition rule. Searching all historical rosters is finite for a fixed public prefix, but it is not a constant-size post-entry query, and chasing a moving successor forever is not a proof of eventual discovery. The old owner must transfer outstanding discovery/ingress obligations along with bytes, and the new owner must actually service them before further retirement.

These are precise outstanding algorithmic obligations. The fixed-pair construction does not hide them in an `AcquirePending` call or assume that source-history hydration automatically discovers private pending queues.

## 6. Ready skew and report deadlines

All queue-cut rounds belong to Waiting/preparation. Their completed contents and certificates are retained independently of beta. No queue-cut round may be placed between Ready and the original atomic Enter. An expanding queue must not restart a completed finite cut.

For the fixed-pair construction, every correct target member completes the same schedule and has the common cut before any correct member is permitted to enter on its hydration certificate. This gives a route to preserve the original `e_v <= e+Delta` entry skew: the first ReadyBundle reaches recipients that have already completed the additional pending-data preparation, so no new queue transfer remains in their handlers.

That implication requires a real all-recipient completion proof, including recipients that have not entered a BFT program. It cannot rely on an unknown cache. The frozen bundle interface requires complete dependencies and expressly forbids unresolved fetches after first Ready. An integrated amendment must specify whether hydration is an already completed node-level precondition or a new bundle dependency. If any recipient instead needs an arbitrary pool fetch after receiving the first bundle, the old skew and bootstrap deadlines are lost. Adding only a root or certificate while leaving that fetch unresolved does not fix them.

Once all correct members of a fixed session know `x`, no whole-pool acquisition is required for that request's awareness in its later INPUT_REPORTs. Ongoing full-request pushes within that fixed roster take the original regional interval. The report rule must still respect its exact parent, admission and no-ancestry checks. DRAIN supplies no new request-bearing opportunity.

The first-DATA minimum/primary rule remains conditional on the pending user scope choice. This note does not authorize replacing FIFO or claim that awareness alone repairs FIFO's accepted-proposal gap.

## 7. The eventual-awareness proof that composition would need

Suppose the ownership/ingress construction is completed with these proved invariants:

- Every received, globally unaccepted `x` has a correct durable owner until another correct service accepts its full obligations.
- Each usable ownership transition has a finite immutable cut containing all earlier correct-owner requests; its later-arrival route delivers every later received request to the successor before that request can be lost to retirement.
- Every newly serving roster completes all-correct hydration before its first Ready, or receives `x` by a proved finite dissemination event while it remains serving.
- Subsequent disjoint rosters cannot bypass the applicable ownership lineage or its outstanding tail obligations.

For a fixed `x`, follow its durable ownership obligation. After its receipt is included in a completed cut, every correct member of that target roster has `x`. Each later completed transition preserves that statement by induction, even if successive memberships are disjoint. If `x` arrives in a tail, the tail obligation first establishes the same statement at its successor, after which the induction applies. Beta changes and discarded speculative suffixes do not discharge it.

Consequently, apart from finitely many locally finite pre-dissemination reports, every accepted term-first DATA from those hydrated serving rosters has an `INPUT_REPORT` certificate containing an honest signer aware of `x`. If `x` remains admissible after the accepted parent, fits alone, and is absent from the parent's ancestry, that signer reports an eligible candidate with key at most `key(x)`. The finite-predecessor primary argument in the companion note then gives inclusion, provided the primary rule is authorized and integrated progress remains valid.

This induction applies once the four invariants have implementations. It is not a derivation of those invariants from bare retention. The fixed-pair procedure proves the snapshot part; discoverability, tail routing across arbitrary epochs, and the old-profile integration still remain open here.

## 8. Scoped counterexample and final boundary

The existing manuscript, without the new owner/handoff rules, permits the following awareness failure. One continuing physical region uses disjoint fresh rosters. Each roster contributes an accepted term-first DATA before the client's request is delivered to its members. Deliver `x` at a finite later time to every roster's correct nodes, after that roster's report-producing activity has ended. Every receiving node retains `x`; each local queue has only finitely many predecessors; `x` reaches the continuing region and even eventually reaches every named correct node. Existing correct-to-correct messages obey their regional bounds. Since the current protocol has no mandatory full-request push/queue-cut handoff to future members, every accepted report certificate may still be drawn from currently unaware correct members. Retention does not rule out this execution.

This is a counterexample to deriving awareness from the existing text. It is **not** a counterexample to the proposed immediate-push rule when its sender and recipients are already covered by the same regional bound. Nor does it establish an impossibility theorem for all protocol amendments. If one additionally allows the last owner to leave before a discoverable successor accepts its duties, failure is immediate; that is exactly the lifecycle link the proposed rule must prohibit or the environment must guarantee.

The constructive conclusion is therefore narrow: finite signed queue-cut rounds can avoid Byzantine withholding, whole-pool timing assumptions, and false quorum-intersection arguments. A node-level owner is compatible with the existing worker-retirement architecture. A full request-awareness solution for arbitrary future rosters still needs the explicit cross-snapshot regional interface, nonforking and discoverable ownership transitions, finite ingress-route transfer, and a complete all-recipient Ready/timing/size audit. No raw request or isolated report acquires `T_pub`, `T_E`, or a new trusted sequencer in this analysis.

Validation: source-level analysis against the frozen v3.7 sources and the companion scope note; cross-checked the historical-manager network interpretation with the parallel model-boundary audit. No manuscript edits, compilation, model checking, benchmark, or whole-protocol certification was performed.
