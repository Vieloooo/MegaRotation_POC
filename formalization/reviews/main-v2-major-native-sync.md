# Native Sync HotStuff source and adaptation audit

28 September 2026. This is a bounded source/proof review for the major revision of `main_v2.tex`; it does not edit the manuscript or certify the finished manuscript.

## Authoritative source and a material correction

Use Abraham, Malkhi, Nayak, Ren, and Yin, *Sync HotStuff: Simple and Practical Synchronous State Machine Replication*, **ePrint 2019/270, revision 20200504:200638**, Section II, Figures 1–2, Lemmas 1–2 and 4, Theorem 3.

- [Pinned revision](https://eprint.iacr.org/archive/2019/270/20200504:200638).
- [Metadata and license](https://eprint.iacr.org/2019/270). The metadata identifies 4 May 2020 as the latest revision and lists CC BY.
- [Version history](https://eprint.iacr.org/archive/versions/2019/270).
- Retrieved PDF SHA-256: `51d2ba5a1fe109543ff5f3db5e68b6ea7c96f77a162abfe2e3a8259a147769a7`.
- Temporary local inspection copies: `/tmp/synchotstuff-current.pdf` and `/tmp/synchotstuff-current.txt`.

The previously used [author-hosted November 2019 PDF](https://dahliamalkhi.wordpress.com/wp-content/uploads/2019/11/synchotstuff-sp2020.pdf) is **not interchangeable with the May 2020 algorithm**. Its Figure 2 returns directly from STATUS to the steady state, and its liveness discussion allows only Delta after leader entry to collect STATUS. The revised Figure 2 expressly introduces a view-entry lock, a 2-Delta collection delay at the new leader, a NEW-VIEW message, and recertification of the selected old block in the new view. Its progress watchdog changes from `(2p+1) Delta` to `(2p+4) Delta`.

This resolves the main ambiguity that previously motivated the invented fixed-window handover. The revision should use the native May 2020 view change, rather than patch the older text or retain scheduled active/freeze/drain windows.

## Native objects, stated where needed

A consensus block is a batch plus its parent hash. Its batch height is distinct from the mini-block heights inside it. A certificate `C_v(B)` consists of votes for exactly `B` in certificate view `v`. Certificates are ranked lexicographically by `(v, height(B))`; blocks form an ancestry relation, and a block extends itself.

**Certificate view is not necessarily block-production view.** During recovery, an existing old block is recertified in a new view. Its mini-block headers, producer signature, contents, and block identity remain unchanged. Avoid deriving the block's content identity from its latest certificate view or rewriting mini-block fields during recertification.

The source maintains highest known certificates but changes its **lock only at entry to the next view**. Steady-state voting does not compare each incoming parent against a continuously increasing old-view lock. This distinction is necessary to avoid delayed old certificates disrupting an already established new-view chain.

## Minimal explicit native algorithm

All messages are domain-separated by the fixed L1 configuration, region, regional session, and local view. The source assumes no clock drift; synchronized wall-clock offsets and globally scheduled view intervals are unnecessary. At equal event times, process arrived proof/exit messages before timer expiry.

### Beginning a view

For the initial view, use the common genesis/authorized anchor. It can be treated as carrying an initial certificate, or passed through the same NEW-VIEW recertification routine.

For any subsequent view, use the recovery procedure below. A valid first NEW-VIEW message chooses an old certified anchor. A member checks that the old certificate's rank is at least its stored entry lock, forwards the signed NEW-VIEW, and votes for that **same anchor block** in the new view. This yields the first current-view certificate. Figure 2 does not start a commit timer on this recertification vote. The first fresh child will eventually commit the anchor as an ancestor.

### Chained steady state

1. An active leader with `C_v(B)` proposes a fresh valid child `B'`, carrying `C_v(B)`. The parent certificate is in the **current** view, not merely any earlier view.
2. An active member receives the signed proposal directly or via another member. It checks complete batch/ancestry validity and current-view parent certification. If it has not detected leader equivocation, it forwards the original proposal and multicasts its own vote. It starts a separate `2 Delta` timer for this proposal. Duplicate delivery never restarts that timer or creates a second semantic vote.
3. A member's same-view voted blocks remain comparable. Receipt of two leader-signed PROPOSE/NEW-VIEW statements identifying incomparable blocks is publicly checkable leader equivocation: multicast both, quit the view, and abort its pending timers.
4. When a timer expires while the member is still active in that view, commit its block and all ancestors. All complete block data and valid ancestry must be available before voting; the model's delay bound must cover the relevant bounded protocol objects.
5. The leader may propose the next child as soon as the parent's current-view certificate is available. It does not wait for the parent's commit timer.

The source timer rule implies a certificate is formed by expiry under its assumptions. The manuscript may explicitly require a locally assembled QC at commit as a harmless defensive guard, provided its proof and latency analysis explain why the guard is satisfied. It must not replace the timer with immediate one-QC finality.

### Native local view change

1. Lack of progress causes a signed BLAME. The source's cumulative watchdog is: for every positive integer `p`, at `(2p+4) Delta` since local view entry, fewer than `p` proposals that triggered fresh votes causes BLAME. Received-but-invalid proposals do not count. Empty valid heartbeat batches can prevent accidental stalls when there are no pending commands, if the application permits them.
2. On a quorum of BLAME statements, forward the certificate and quit. On valid leader-equivocation evidence, forward it and quit immediately, without waiting for a BLAME quorum. Quitting stops all old-view voting and cancels old-view timers.
3. Wait `Delta`. At the end of this wait, choose a highest valid known certificate, retain it as the entry lock, send it with needed evidence to the next L1-list leader, and enter the next view. Do not subsequently raise that view's entry lock because a delayed old certificate arrives; store it for the next recovery instead.
4. The next leader waits **`2 Delta` after its own entry**, collecting STATUS certificates. It then selects a highest known old certificate and broadcasts NEW-VIEW carrying it.
5. Members re-vote the selected block in the new view if its old certificate rank meets their entry lock. Continue with the chained steady state after the resulting current-view certificate forms.

A future-view message does not by itself authorize skipping the quit/wait transitions. Missing exit evidence and STATUS/data must be processed or fetched so that view advancement remains justified. Honest exit evidence propagates in one Delta; hence honest entries to a given view differ by at most Delta, which is why two Delta after leader entry is sufficient to receive every honest STATUS.

There is **no maximum number of blocks per view, no fixed tenure, and no forced idle interval**. A good leader can run indefinitely. The L1 rotation list specifies successors when native view change occurs. If periodic leader rotation is additionally required, that is a separate modification; it is not already provided by the native failure-driven pacemaker.

## Weighted adaptation

Let regional total weight be `W`, adversarial/unavailable weight at most `b < W/3`, and continuously participating honest weight greater than `2W/3`. Use `wt(Q) > 2W/3` for block and BLAME certificates. The source itself uses cardinality `n=2f+1` and certificate size `f+1`; the weighted supermajority variant is a deliberately stronger deployment assumption.

The source proof needs two threshold properties:

1. Every accepted certificate contains an honest participant: `q > b`.
2. All correct responsive participants can form a certificate without Byzantine help: `q <= W-b`, with the corresponding strict-threshold interpretation.

Both hold for `q > 2W/3` when honest weight is greater than `2W/3`. In this adaptation, quorum intersection also gives honest intersection between any two regional supermajorities; with comparable same-view honest votes, certified blocks from one view form one chain. Do not argue that weighted correctness follows solely from changing `f+1` to a weight symbol: identify these properties and repeat the temporal view-change invariant below.

Stake weights are fixed during this proof. A finalized L1 update does not retroactively change the denominator, a certificate's signer weights, or an active local view. Dynamic-set handover requires its own protocol/assumption and is outside the native Sync HotStuff proof.

## Proof skeleton that matches this algorithm

### Timely witness dissemination

Suppose correct member `r` directly commits `B` at time `t` in view `v`. It voted and forwarded the leader proposal at `t-2 Delta`. By `t-Delta`, every correct member has seen the proposal. If a correct member had already quit or observed/forwarded an incomparable leader message by then, its exit/equivocation witness would reach `r` by `t`, contradicting `r`'s uninterrupted timer. Entry skew also implies correct members have reached view `v` by that point. Thus they can vote for `B`; their votes give every correct member `C_v(B)` by `t`. The mandatory post-quit wait means every correct member entering `v+1` locks on a certificate of rank at least `C_v(B)`.

For this argument the receive handler must process necessary certified ancestry and detect equivocation before voting. With only exact-height first-vote rules, make the equivalent same-view comparable-chain rule explicit. Handling already received descendants as dependencies must not create an apparent refusal to vote for a prerequisite that breaks the dissemination argument.

### Unique extensibility

After a correct direct commit of `B`, no incomparable block can receive an honest vote in that view without exposing timely equivocation. A later conflicting certificate of minimal rank cannot arise either:

- if caused by NEW-VIEW recertification, it requires an earlier certificate whose rank meets an honest entry lock that already covers `B`; this contradicts minimality;
- if caused by a normal proposal, its same-view parent is an earlier conflicting certificate, again contradicting minimality.

This is the May 2020 source's proof structure. It relies on current-view parent certificates and explicit NEW-VIEW recertification, not on the older source's less precise dynamic-highest voting condition.

### Prefix safety

Every commit is a direct timed commit or an ancestor of one. Unique extensibility makes any two directly committed blocks comparable, so all correct committed batch logs are prefix-compatible. Deterministic flattening and valid mini-block linkage preserve this property for the mini-block stream.

### Progress

An honest leader's two-Delta collection obtains all honest entry locks. Its NEW-VIEW anchor meets every honest lock; recertification completes with honest weight alone. The first fresh proposal follows, and subsequently one proposal/certificate pair takes at most `2 Delta`. Honest entry skew and NEW-VIEW startup fit the source `(2p+4) Delta` watchdog, so an honest leader is not removed for lack of progress. If a Byzantine leader does not keep eliciting votes, honest BLAME weight removes it; finite cyclic successors eventually reach an honest leader if removals continue. Otherwise certified proposal progress itself continues and, without equivocation/exit, timers yield commits.

This proves **ledger growth**, as the source's Lemma 4 states. It does not by itself prove every mempool transaction or every competing batch proposal is included.

## Request-liveness caveat

The paper's introductory SMR definition says every client request eventually commits, but its liveness proof is a log-growth proof. A stable Byzantine leader can keep producing valid blocks and censor one request forever. A rewriting that claims request-level liveness must identify an input-fairness condition or mechanism; it must not manufacture FIFO/request-ID machinery merely to fill the gap.

For this manuscript, distinguish (a) the standard abstract SMR contract; (b) consensus log growth; (c) producer selection of transactions and admission of batch commands. State an explicit fair-input assumption for any eventual-inclusion theorem, and do not claim that native progress-triggered rotation itself guarantees eventual honest leadership if a Byzantine leader keeps extending the chain.

## Complexity and confirmation

Under a stable honest leader and complete-data delay `delta <= Delta`, the steady-state proposal spacing is about `2 delta`, while local commit occurs at most `delta + 2 Delta` after proposal emission. NEW-VIEW recertification adds startup cost once per recovery, not per batch.

The native protocol echoes proposals and multicasts votes: quadratic honest-party sends per batch. If a forwarded proposal contains a linear-size ordinary-signature parent QC, bit cost can include cubic signature material without deduplication or compression. Do not quote original HotStuff collector linearity for Sync HotStuff.

For the external proposer, a normal vote QC is not a transferable proof that an honest member's timer completed. After local commit, members can sign a decision statement bound to the immutable batch/session identity. A supermajority of those attestations proves at least one honest commit and hence the prefix safety above. With direct member-to-P delivery, no missing data, and all-honest good-case commit, confirmation latency is `delta + 2 Delta + delta_P`. This export is our interface adapter, distinct from source's internal timer rule.

## Remaining limits

This review covers standard synchrony only, not the paper's mobile-sluggish or optimistic-responsive sections. A violation of the synchronous envelope is not automatically a regional crash; an outer CFT abstraction must separately constrain failures or prove a wrapper. The L1 stake binding and regional evidence authenticate regional outputs, but do not prove cross-region finality or dynamic reconfiguration.
