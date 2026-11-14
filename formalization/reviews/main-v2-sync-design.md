# Sync HotStuff regional design review for main_v2

Date: 28 September 2026. Scope: standard synchronous regional SMR only; cross-region CFT remains unspecified. This report recommends a conservative weighted adaptation; it is not a claim that the original Sync HotStuff paper specified the new wrapper.

## Primary-source baseline

[Abraham, Malkhi, Nayak, Ren, and Yin, Sync HotStuff, IEEE S&P 2020, final author PDF](https://dahliamalkhi.wordpress.com/wp-content/uploads/2019/11/synchotstuff-sp2020.pdf): §I.A defines SMR, §II.A and Fig. 1 give the standard synchronous pipeline, §II.B and Fig. 2 give recovery, §II.C Lemmas 1–2/Theorems 3–4 give safety/liveness, §II.D gives latency. The original uses `n=2f+1`, `f+1` certificates, and all-to-all votes. A vote relays the signed leader proposal. Commit timers start with the vote and run for `2Δ`; equivocation or quitting cancels them. The next proposal needs a certified parent, not a committed parent. Highest certificates are ordered by `(view,height)`. Native recovery collects `f+1` blames, relays them, stops voting/timers, waits `Δ`, and reports highest certificates to the next leader. Its progress budget is `p` proposals per `(2p+1)Δ`. Good-case proposal spacing is about `2δ`, regional decision latency `δ+2Δ`.

Do not equate this with partially synchronous HotStuff, its three-chain commit rule, or linear collector communication. Source §III is a separate mobile-sluggish variant and should not be mixed into this construction.

[Decouchant, Ozkan, Zhou, PRDC 2023](https://arxiv.org/pdf/2310.09006), §III.C, explicitly tests the preliminary Sync HotStuff eprint `20190312:115828` and discusses its force-locking attack. It does not establish that the final S&P2020 version is broken. Pin the final source and give the adaptation's own proof.

## Recommended regional construction

Keep the v1 mini-block tuple literally:

- `H_h=(c,v_G,v_L,u,h,q_{h-1},H(TXLIST,X_h))`;
- `q_h=H(MINIBLOCK,H_h)`;
- `M_h=(H_h,X_h,Sign_leader(ORDER,H_h))`.

Introduce SMR batch height `k`, distinct from mini-block height `h` and proposal attempt `u`. Set `u=pair(v_L,k)` using a specified injective integer pairing. A batch consensus block points to its preceding consensus block, not to `R_{u-1}`. Each block carries a nonempty mini-block range; its certificate can already support the next proposal before its timer expires. Keep the `>2W_i/3` threshold for votes, DECIDED attestations and other exported regional certificates. This is stronger than the source threshold, useful for the simple proof below, and requires responsive honest weight `>2W_i/3` throughout an active regional execution.

Require each honest sequencer's votes in one local view to form one comparable chain. This strengthens the source's first-vote-per-height rule: a sequencer must not sign two incomparable consensus blocks even when their heights differ. A leader proposes one comparable chain per view. Full ancestry witnesses make incomparability decidable. A valid certificate binds configuration, authorized regional incarnation, local view, batch height, fresh attempt, and exact semantic block digest.

### Fixed view schedule

Use an explicit common logical clock, already available in v1. Choose active duration `L >= 4Δ_R`; a larger configured duration amortizes handoff costs. Define:

- active interval of local view `v`: `[A_v,F_v)`, with `F_v=A_v+L`;
- drain interval: `[F_v,F_v+Δ_R]`;
- recovery snapshot instant: `S_v=F_v+Δ_R`;
- next active start: `A_{v+1}=F_v+2Δ_R`.

The local cyclic L1 list selects the next leader. At genesis give all replicas a common certified genesis/activation anchor and the same `A_0`. At boundary ties, process messages and equivocation evidence before checking a timer; all direct commitments must occur strictly before `F_v`.

This wrapper intentionally replaces native blame-triggered immediate recovery. Evidence can cancel timers early, but the next leader starts at the public boundary. Faulty leaders can consume a whole active window; fixed finite windows and cyclic honest leaders still ensure progress. An omission watchdog may mark a view failed but is not an alternate unsynchronized activation mechanism.

### Active pipeline

1. The first proposal extends the selected old-view parent described below. Thereafter the leader extends its most recently proposed block once it has that block's vote certificate. Each block is at parent batch height plus one and has fresh attempt `u=pair(v,k)`.
2. Validate the complete signed proposal, parent certificate, complete ancestry to a trusted anchor, body data, ordering commitments, L1-scheduled author, and size/admission bounds. A proposal learned through an authenticated vote remains the original leader's proposal, not a new proposal by the relay.
3. An honest sequencer votes once for the first admissible block at each height, only along its current-view comparable chain; relay the proposal and multicast its vote. On its first vote start a `2Δ_R` timer. Late old-view certificates do not modify the frozen recovery guard.
4. On a new QC, retain it and multicast it once under bounded deduplication. A later same-view valid proposal can also carry it. Advancing the proposal pipeline does not wait for commitment.
5. At expiry, commit the block and its ancestors only if still in the active interval, no equivocation/abort occurred, complete data and that block's full QC are present, and it extends the persistent committed prefix. The full-QC condition is a conservative extra guard. Multicast the commitment's QC and produce the post-decision attestation immediately.
6. On authenticated same-view leader equivocation, relay the pair and cancel all unexpired timers; stop signing that view. A failed view does not append a fictional mini-block/GAP command. At `F_v`, every honest replica stops voting and cancels every remaining timer.

The proof below does not require that a QC alone be called a commitment. The timer remains the recognizable Sync HotStuff rule and protects the intended synchronous decision interface; the boundary protocol also ensures preservation independently of hidden old certificates.

### Snapshot and leader recovery

At `S_v`, each honest replica freezes `H_r`, its highest verified certificate from views at most `v`, ordered by `(certificate.view,batch_height)`. It sends signed `STATUS(v+1,H_r)` and the necessary complete proof/data to the scheduled next leader. Its own durable commitment must be represented in its known certified chain. The leader receives every honest status by `A_{v+1}` under the complete-object envelope, selects a maximum-rank valid certificate from the received statuses and its own frozen state, and includes it as the first parent QC.

Each honest replica accepts that first parent only when:

- its certificate is from an earlier view and is fully valid;
- its rank is at least the replica's frozen `H_r` rank;
- its chain extends the replica's own persistent committed prefix.

Thereafter, current-view votes remain on the chosen comparable chain and require certified parents. A newly learned old-view QC after `S_v` is retained as historical evidence/data but does not update the current-view guard. This cutoff is explicit protocol state, not an assumption that Byzantine parties reveal every certificate promptly.

**No full STATUS-vector agreement or certificate is necessary here.** Honest leaders learn all honest frozen states by the bounded collection time; Byzantine leaders must still pass every honest voter's local rank guard. The crucial guard is frozen certificate rank, not merely the voter's personally committed tip.

If an implementation allows early unscheduled view changes, clock skew, or asynchronous STATUS processing, these exact arguments no longer apply without a replacement proof.

## Safety proof kernel for the adaptation

Let `b_i<W_i/3`. Two vote signer sets of weight greater than `2W_i/3` intersect in honest weight. Since an honest sequencer votes only comparable blocks within a view, all certified blocks from that view are mutually comparable, including certificates at different batch heights. Equal `(view,height)` certificates therefore identify the same semantic block.

Suppose an honest sequencer commits `B` in view `v` at time `τ<F_v`. The direct-commit guard requires its full QC, which it multicasts immediately. Every honest sequencer receives that QC by `τ+Δ_R<S_v`. Thus every frozen snapshot has rank at least `QC(B)`; at the same view, comparability implies that any such highest certificate extends `B`.

Induct on views. Assume certificates issued after a prior honest commitment extend that commitment. The previous paragraph makes every replica's frozen high certificate cover every honest commitment from the just-finished view. Any honest voter in the next view accepts a first parent whose rank is at least its frozen rank. A same-view predecessor certificate of equal/higher rank extends the committed block by comparability; a certificate from an intervening higher view does so by the induction hypothesis. Consequently every first vote, every descendant vote, and hence every new QC extends every previous honest commitment. A Byzantine leader cannot hide a prior honest commitment merely by omitting its STATUS: every honest frozen state already dominates that commitment's QC. Later exposure of an old uncommitted QC cannot change the current guard.

This gives prefix agreement across local rotations. Persistent signing history and committed state are required after recovery. A recovered process must catch up before voting; treating arbitrary delayed honest behavior as standard synchrony is invalid.

## Progress and throughput

At `A_{v+1}`, an honest leader has all honest frozen certificates and can select a parent meeting all rank guards; the same-view uniqueness property removes equal-rank ambiguity. Its proposal is received by all honest replicas within `Δ_R`, and all honest votes return within another `Δ_R`, giving a QC. Timer expiry occurs by `3Δ_R` after the first proposal, strictly within an active interval of length at least `4Δ_R`. Thus each honest view has room to commit at least one batch. In steady state, proposals can be spaced by roughly `2δ_R` while timer decisions occur roughly `δ_R+2Δ_R` after proposal.

A finite cyclic leader list contains an honest leader. Repeated honest views therefore ensure log growth; per-client request liveness additionally requires an explicit admission and fair inclusion rule. An ordering service cannot claim the standard SMR request-liveness property merely because its leader can emit empty batches forever.

Regional plain-signature message count is quadratic per batch because votes multicast. A QC has linear signature material without an explicit aggregation setup, so count bit complexity separately from message complexity. Evidence and complete-data traffic must be bounded; signatures alone are not the entire transfer object.

## Externally verifiable decision and abstraction limits

A normal vote QC is not a proof that honest timers have expired. After local commitment, honest replicas sign `DECIDED` over an immutable outcome/block reference and never sign conflicting decisions for the same key. A `>2W_i/3` DECIDED certificate gives an external proposer transferable positive evidence of a genuine honest commit; its good-case arrival is about `δ_R+2Δ_R+δ_P`, excluding request ingress, queueing, and missing-prefix transfer. Quorum gathering may occur later after a skipped local commit; subsequent honest views commit its preserved descendant chain.

The standard SMR specification should define a deterministic transition function, a totally ordered prefix-consistent committed log, validity, exactly-once processing by request identifiers, responses after commit, and eventual inclusion for admitted client requests under explicit fairness. For the ordering application the transition appends ordered mini-block ranges; it does not execute EVM transactions or certify execution roots.

The regional-to-CFT abstraction is conditional. All outward region-level messages must be deterministic outputs of the replicated regional state, with durable outer term/vote/log state and quorum attestations. Standard Sync HotStuff loses its claimed safety if its synchrony envelope is violated; a whole-region outage or partial partition must therefore be constrained to fail-silent behavior or handled by an additional proved wrapper. The new draft can state this as a remaining outer-layer assumption/obligation; it cannot claim completed cross-region safety/finality.

## Slashing and notation compatibility

The existing v1 ORDER key is `(c,v_G,u,h)`; PROPOSE/VOTE and DECIDED use `I=(c,v_G,u)`. Fresh `u=pair(v_L,k)` prevents normal re-proposal at a reused log height in a later view from becoming an honest double-sign offense. Forwarding the same origin proposal and another replica's vote is not a new own vote. Both proposer and voter statements should carry the full block digest while the comparison keys remain unchanged.

Keeping the literal v1 `R_{u-1}` adjacency would conflict with this fresh-attempt numbering. Explicitly replace it with parent-consensus-block pointers. `k`, `h`, and `u` must remain distinct. Preserve ORDER/VOTE/DECIDED conflict predicates and add the stated same-view incomparable-vote condition if desired; late conflicting evidence must be verified before ordinary-message rejection. Silence, timeout, honest view changes, and absent DATA responses are not publicly proven equivocation. Historical REPORT/FFG conditions apply only to actual statements of those domains; the new unfinished CFT layer does not emit them automatically.
