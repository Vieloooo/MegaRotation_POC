# Final regional proof verification — main_v2

Review date: 28 September 2026. The proof-verifier skill was applied in verifier-only mode; this review makes no manuscript changes.

Reviewed source: `formalization/main_v2.tex`, SHA-256 `e3d5bb95852c3f74615068f5f64e49afa79fbe77340d969e2c9abd2bf09bf54f`. References below are to that snapshot, also retained temporarily at `/tmp/main-v2-regional-review-final-source.tex`.

## Summary

**Final Verdict:** The regional safety, deterministic batch execution, admitted-request completion, regional-response linearizability, and regional evidence-soundness arguments are valid under the explicitly stated fixed-session synchronous, stake, persistence, admission, and fair-service assumptions. No remaining Critical Error or Justification Gap was identified in these regional arguments.

**List of Findings:** None. All four substantive findings in `main-v2-regional-review-v1.md` are resolved in the reviewed source.

This verdict is a manual verification of the written arguments, not a mechanized proof or implementation test. The fixed-window weighted protocol is a stated adaptation of the standard Sync HotStuff core, not a claim that the original paper proved this exact variant. Cross-region CFT, whole-region recovery outside the regional envelope, stake reconfiguration, and global finality remain deliberately unverified because they are not implemented.

## Detailed Verification Log

### Resolution of the preceding review

1. **Slash comparison domain:** lines 558–560 now require the same statement type and equal `(c,i,v_G,v_L)` before comparing branches. Lawful signatures from distinct regional sessions no longer satisfy the fork offense. ORDER/PROPOSE/VOTE/DECIDED identifiers remain safe under the injective attempt encoding.
2. **Deterministic application:** lines 218–231 define the replicated command as `AppendBatch(B,M_B,reqs_B)`, including the exact framing and positions; `Append(r)` is the client interface. Lines 301–306 bind request identifiers and byte hashes through `reqRoot` inside the signed semantic proposal. Lines 315–323 verify correspondence. This resolves the earlier ambiguity about how identical request order determines mini-block boundaries. Request signatures remain validity witnesses rather than state-transition outputs, and raw transaction bytes remain unchanged in mini-blocks.
3. **Honest issuance timing:** S1 at 417–420 now issues the first proposal at `A_v`. S2 at 421–424 mandates the next proposal immediately after the parent QC, subject to the count and cutoff. This supplies the exact timing premise used by the honest-view lemma.
4. **Proposal cap:** lines 315–323 apply the same-view branch-length bound to every producer. Together with comparable voting, an honest member signs at most `N_R` current-view blocks (400–401). Together with quorum intersection, certified current-view blocks also lie on one chain and are bounded in number. This supports the prescribed consensus-work cap; unrestricted adversarial input validation is still subject to the explicit admission/flooding limitation at 768–770.

### Quorums and fixed-session scope

The regional model at 146–163 supplies fixed Byzantine weight below one third, persistent honest state, shared logical timing, and complete-object delivery/verification within `Delta_R`. The weighted quorum lemma at 179–187 is correct: strict greater-than-two-thirds quorums intersect in weight strictly greater than one third, so their intersection includes an honest signer; honest weight alone forms a quorum.

The model explicitly excludes arbitrary mobile sluggishness and loss of the synchrony envelope. It does not assert that violation of this envelope becomes merely a crash at the future outer interface. Historical key compromise and dynamic configuration changes are outside the theorem.

### Object binding and vote discipline

The pairing at 278–287 injectively identifies `(region,local view,batch position)` inside the retained attempt `u`. Therefore votes signing `(VOTE,I,Hash(x))` also fix the otherwise redundant envelope fields. The batch binds its certified parent, resulting mini-block tip, and ordered request metadata.

Validation at 315–326 checks complete ancestry, the unit increment in batch position, nondecreasing local views along parent edges, agreement of mini-block/session/attempt fields with the enclosing batch, the transaction/request correspondence, authentication and uniqueness, and count/size limits. The decreasing batch position in a backwards ancestry traversal ensures termination at the root.

S3 at 425–432 forbids a signer from voting for incomparable blocks anywhere in one view, not merely at one height. It also preserves the committed prefix and frozen entry rank. The current-view restriction at 454–455 prevents a late old message from generating a backfilled vote.

### Boundary and recovery argument

The times at 404–411 provide one full delay from voting/timer freeze to the snapshot, followed by one full delay to the next leader. Direct timer decisions occur strictly before the freeze. Equal-time evidence and freeze events have precedence over a timer decision.

Commitment requires a complete QC (442–447). The honest holder has multicast that QC under S4. Hence a commit at `t<F_v` implies receipt by every honest member by `t+Delta_R<S_v`. Their frozen statuses cannot fall below that certificate. H2–H4 at 469–481 preserve this watermark, deliver every honest status to an honest next leader, and keep late older-view certificates from changing current guards after collection.

A Byzantine leader cannot bypass preservation by omitting one honest status: all honest snapshots already dominate every prior honest commitment's QC. Conversely, an honest leader learns every honest frozen rank and can choose a parent that passes all their guards.

### Regional safety

The comparable-certificates lemma (593–600) is valid by honest quorum intersection and S3. It applies across different batch positions; equal rank implies the same semantic block.

The inheritance lemma (602–622) is valid. The current-view case follows from certificate comparability. For later views, an honest signer supplies the rank guard; following current-view certified parents backwards reaches the earlier-view parent and invokes induction. The explicit parent-view monotonicity now makes the case split immediate. Retained certificate state ensures future watermarks cannot lose the protected rank.

The regional-safety theorem (624–639) follows: direct commitments are comparable within a view and preserve earlier commitments across views; ancestor commitment preserves the same order. The newly explicit framed `AppendBatch` command determines mini-block state, duplicate-ID updates, and returned positions. Detached witness encodings do not alter semantic results.

### Progress, requests and real-time order

For the honest-view lemma (641–658), the first proposal occurs at `A_v`; each parent certificate takes at most `2Delta_R` to reach the leader; proposal j therefore occurs by `A_v+2(j−1)Delta_R`. Its honest recipients finish their timers by `A_v+(2j+1)Delta_R`. Their full QCs are available by then. This is strictly before `F_v=A_v+(2N_R+4)Delta_R` for every `j<=N_R`. The extra issuance cutoff does not obstruct this sequence, since the final required issuance time is earlier than `F_v−4Delta_R`.

For request completion (660–682), a correct-client request reaches all honest queues through retries and forwarding. It either already appears on a selected ancestry or faces only finitely many predecessors in the finite union of honest queues. Fair honest views commit at least one still-uncommitted predecessor or the request itself; discarded tentative ancestry does not erase queued requests. Honest leaders recur in the finite cyclic schedule, so the request eventually reaches an honest view's committed chain. Honest DECIDED signatures then suffice for a transferable response.

The fresh-request rule at 208–210 states that correct clients create and first release the request at invocation. Together with response-after-commit and irreversible prefix extension, this proves real-time precedence: a later invocation cannot be inserted before an operation whose response already completed. Duplicate retries use the recorded position rather than executing again. This is regional-session linearizability, not a global linearizability claim across the missing outer protocol.

### Transferable evidence

The evidence-soundness proposition (684–693) is valid: every DECIDED certificate includes an honest signer, and the export rule permits that signature only after a local commitment. The regional prefix-safety theorem therefore excludes conflicting certified regional logs. An ordinary VOTE QC is correctly distinguished from a publicly verifiable post-commit attestation.

The slashing predicates are positive signed conflicts. They neither slash silence/timeouts nor silently alter the active stake denominator, and they do not imply a separate economic or accountable-safety theorem.

### Communication and remaining scope

The latency expressions at 745–757 correctly distinguish `delta_R+2Delta_R` regional confirmation, consumer delivery of evidence, and an approximately `2delta_R` pipelined proposal interval. Timer expiry is not replaced by parent certification.

For each proposed batch, raw votes use quadratic transmissions. Unaggregated certificates disseminated by all holders can use cubic signature bits. The draft states this overhead rather than attributing the linear communication of other HotStuff variants to this protocol. Required data and ancestry remain charged to the complete-object envelope. The `N_R` check bounds admitted same-view vote/certificate chains; rejection/verification of unrestricted floods is expressly outside the bounded admitted-workload model. No finite-history pruning theorem is claimed.

The outer interface at 695–742 identifies the durable outer state, deterministic outgoing transitions, and remaining crash-only refinement obligation. Since election, log replication, region activation, fencing and global finality are absent, the manuscript appropriately claims no end-to-end CFT theorem or cross-region irreversible-delivery guarantee.
