# `main_v2.tex` interface review, pass 1

Reviewed 28 September 2026. Target: `formalization/main_v2.tex`, 783-line snapshot, before root-agent integration of this review. This pass concerns SMR/application semantics, stake configuration, evidence soundness, virtual-region abstraction, and source scope. It is not an independent certification of the full Sync HotStuff adaptation.

## Blocking findings

### R1 — Deterministic state transition does not determine the stated state [P1]

**Location:** lines 216–223, with the definition at 191–200 and theorem proof at 619–620.

The state includes the committed mini-block sequence and tip `(h,q_h)`, but the command is `Append(r)`, which contains only the client request. Mini-block grouping, empty mini-blocks, producer/local-view metadata, and the attempt identifier are not determined by the sequence of these commands. For example, the same requests `r1,r2` may form one mini-block or two, giving different `h` and `q_h`; an idle empty mini-block changes the stated state without appending any `Append(r)` command. Consequently the displayed deterministic function `T:S×C→S×O` has not actually been instantiated for the state claimed.

**Fix:** define the internal replicated command as a validated full batch semantic object (`AppendBatch(B)` with its committed mini-block headers/contents), and define a deterministic transition that appends those objects and executes/deduplicates the contained requests. Describe the client `Append(r)` interface as a linearizable projection of this batch-command machine. An alternative is to limit `T`'s state to the request log/deduplication table and move all batch/header metadata into a separately specified consensus wrapper. Specify the returned transaction position, e.g. `(h,offset)` or a global request-log index. Keep alternative signature/witness encodings outside deterministic semantic state.

### R2 — Fork evidence omits the global generation [P1]

**Location:** evidence predicate (d), lines 541–542; session domain at 139 and new-session limitation at 143–144.

The predicate says same region and same local view for incomparable branches but does not require the same global generation `v_G`. Legitimate new sessions reset the local view to zero and begin from their new anchor. An identity can therefore sign valid, incomparable session-0 and session-1 branches with the same local-view number. The literal predicate accepts that pair as slashable. Verifying each statement's domain in F1 does not establish equality of the pair's generation.

**Fix:** explicitly require equal `(c,i,v_G,v_L)` and the same statement type before testing incomparability. The other predicates already bind a globally unique attempt `u` or an explicit full STATUS domain.

### R3 — Claimed proposal cap does not bound Byzantine-view induced honest work [P2]

**Location:** 383–384, 403–407, 408–419, and complexity claim 750–752.

`N_R` constrains only an honest leader. Neither `ValidBatch` nor the honest voting rule rejects further valid same-view certified descendants after the limit. A Byzantine leader can cause more than `N_R` honest votes/certificate multicasts by proposing such a chain. The network upper bound gives no positive lower bound on delay, so a fixed view duration is not itself a uniform count bound. Thus the assertion that `N_R` bounds per-view honest work is unsupported and the stated complete-object capacity premise is underspecified for this traffic.

**Fix:** explicitly cap each member's first votes/accepted pipeline positions in a local view, without blocking the required certificate/evidence drain; account for non-voting relay work separately. Or narrow the bound to honest views and state a separate enforced admission rule for Byzantine views. Do not reject safety-critical old certificates solely because the proposal quota was reached.

## Scope / compatibility issue to decide explicitly

### R4 — The mini-block tuple is preserved, but transaction bytes change meaning [P2]

**Location:** lines 206–210 versus `main_v1.tex` 334–343; preservation claim at 102–104 and 258–259.

v1 defines each `tx` as transaction bytes. v2 makes it the canonical encoding of a signed service request `(a,s,z,sigma_a)`. This keeps the outer tuple/header constructor but changes the bytes consumed by the proposer. Because the user specifically requested preserving mini-block structure and notation, this semantic change deserves a deliberate choice.

**Recommended fix:** retain `tx=z` in the existing transaction list and carry the request-ID/authentication association as explicitly bound detached metadata in the batch command/witness, or define a byte-level extraction map and state clearly that service envelopes are a new application-level encoding. This is not a consensus attack; it is interface compatibility and self-containment.

## Reviewed points without a blocking finding

- The finite-predecessor-queue fairness assumption and retention across view changes are substantive enough to support a request-completion argument once the replicated-command semantics are fixed. This is stronger and more appropriate than v1's empty-block growth claim.
- Linearizability is restricted to the regional append service. Its response follows commitment and includes evidence. The real-time proof should refer to a fresh, correctly invoked operation/request; normal correct-client semantics suffice.
- The L1 snapshot binds membership, keys, integer weights, a disjoint regional partition and leader lists. The text explicitly separates stake-weighted voting from non-stake-proportional cyclic producer frequency.
- Evidence does not alter the active quorum denominator. Stake admission, exit, key replacement, historical-key protection and cross-configuration state transfer are explicitly out of scope or assumed.
- The injective `u` pairing resolves the otherwise serious v1 slashing-domain collision when a batch position is retried under another local view. Different signature sets on the same semantic statement are excluded from conflict evidence.
- `DECIDED` certifies local commitment rather than mere vote collection. The manuscript does not claim that the listed evidence predicates prove a complete accountable-safety threshold.
- The virtual participant contract correctly requires replication of outer terms, votes, log state, timeout inputs, outbox and deduplication. It does not conflate sequencing with implementing a Raft replica.
- The candidate regional process abstraction explicitly depends on the inner timing/fault envelope. Whole-region loss and arbitrary synchrony violations are not silently equated with harmless crash failure.
- Outer election, replication, fencing, global commit, recovery and authority installation are plainly marked unfinished. The manuscript consistently separates `LocalDeliver` from future global delivery, and gives no completed CFT theorem.
- The compiler citation is correctly limited to partially synchronous base-protocol properties. It is presented as related architecture, not a theorem certifying this fixed-snapshot synchronous construction.
- The Sync HotStuff citation points to the final S&P2020 author PDF and distinguishes the scheduled handover as this manuscript's adaptation. The bibliography does not silently equate it to early ePrint versions.

## Source verification used in this pass

- Schneider's author-hosted SMR survey: <https://www.cs.cornell.edu/fbs/publications/SMSurvey.pdf>.
- Final Sync HotStuff paper, including its §I.A standard SMR definition: <https://dahliamalkhi.wordpress.com/wp-content/uploads/2019/11/synchotstuff-sp2020.pdf>.
- Generic compiler publication: <https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2025.18>.
- Raft extended paper: <https://raft.github.io/raft.pdf>.

The root agent owns manuscript edits. These findings were sent immediately for integration; this review file records the original line locations.
