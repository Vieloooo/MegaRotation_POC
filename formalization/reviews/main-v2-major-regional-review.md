# Regional protocol verification: major revision of main_v2

Date: 28 September 2026. Reviewer mode: verifier only, following `/Users/jin/.codex/skills/proof-verifier/SKILL.md`. No manuscript or figure changes were made by this reviewer.

## Summary

**Final Verdict:** the stated fixed-session regional safety, conditional progress, receipt-soundness and complexity arguments are valid under the explicitly stated assumptions; no unresolved Critical Error or Justification Gap remains in the reviewed scope.

Frozen final target: `formalization/main_v2.tex`, SHA-256 `c0340188273735d89f094e39599129b00f95d1d6c34ce4346bb62667477a1fe4`. Final source checked after the author's fixes; any later purely typographic edits require recording a new hash, not reinterpreting this as a machine-checked proof.

Source: Abraham et al., *Sync HotStuff*, [ePrint revision 20200504:200638](https://eprint.iacr.org/archive/2019/270/20200504:200638), Section II, Figures 1–2, Lemmas 1–2 and 4, Theorem 3. Source PDF SHA-256: `51d2ba5a1fe109543ff5f3db5e68b6ea7c96f77a162abfe2e3a8259a147769a7`. The older November 2019 author PDF was not used as an interchangeable algorithm specification.

List of findings: **none unresolved**. The following two intermediate findings were checked and closed on the frozen final target:

1. **Intermediate location:** lines 322–328 and 337–338, earlier-view certificate versus initial checkpoint. **Classification:** Justification Gap. **Resolution:** final lines 325–326 explicitly designate the common initial checkpoint as the initial certificate of rank `(-1,0)`. First entry and later strict rank descent are now compatible.
2. **Intermediate location:** lines 529–531, unqualified `O(Delta_R)` local recovery. **Classification:** Justification Gap. **Resolution:** final lines 530–532 restrict this to handover to an honest successor and explicitly separate detection, additional failed successors and state acquisition.

Earlier issues identified on initial hash `62480c2027294fe3a1d4fc34c842ec61e2523be52097b8bf45bdb97ca2f9f251` were already corrected in this snapshot: NEWVIEW now explicitly authenticates the current leader and requires an older certificate; batches must contain a nonempty mini-block range; ORDER evidence uses session, producing view and mini-block height, avoiding false accusations for legal later-view replacements.

## Detailed Verification Log

### 1. Model and weighted certificate premises

The model requires less than one third faulty stake, continuously participating correct members, a complete-message/verification delay bound, no clock-rate drift, and authenticated fixed-session membership. Consequently correct weight exceeds two thirds. Every accepted strict-two-thirds certificate contains a correct signer, and all correct participants can assemble a certificate without faulty help. These are precisely the threshold substitutions needed by the source's temporal arguments.

The draft does not assume an unjustified node-count bound. It also does not claim that standard synchrony automatically survives mobile outages, that a whole-region failure is automatically fail-silent, or that a dynamic validator change is proved here. Those are explicitly outside the established scope.

### 2. State machine and application

The standard definition states safety and request liveness separately. Commands are mini-block contents before consensus positions/views are assigned, so a failed view-specific proposal is not silently identified with a permanently rejected abstract request. The input-service assumption separately supplies honest service/fair inclusion for persistently available eligible commands. The consensus theorem itself distinguishes chain growth from request inclusion.

The revision does not derive censorship resistance from the native progress watchdog: a Byzantine leader that keeps the valid chain growing need not be removed. Request liveness is therefore conditional as expressly stated. That condition is an assumption, not a result of this verification.

The nonempty-batch validity rule means growth of the consensus log implies growth of the mini-block stream. Empty transaction lists remain permitted inside a real mini-block.

### 3. Steady-state algorithm

“Current-view parent certificate” is essential and correctly specified. An older-view certificate cannot directly authorize a fresh child; it first passes through NEWVIEW recertification. Re-voting an inherited block does not change original mini-block author metadata.

Each accepted fresh proposal is echoed, voted, and given a nonblocking two-Delta timer. A new parent QC allows the next child before the old timer expires. Thus throughput and commitment latency are distinct, as the manuscript states.

Equivocation and quit proofs cancel outstanding timers. Equal-time evidence is processed before timer expiry. This ordering supplies the closed-boundary case in the temporal proof. The no-confirmation-without-certificate statement is consistent with the proof that correct participants assemble the certificate by a valid timer expiry.

No claim that all same-view certificates are comparable solely because members vote once per height is used. The proof establishes the needed stronger property only after a correct timed commitment, using dissemination of incomparable leader messages.

### 4. Native view change and watchdog

The sequence agrees with the May 2020 source: quorum BLAME or authenticated equivocation; relay and quit; wait Delta; choose an entry lock and send STATUS; next leader waits two Delta; NEWVIEW re-certifies the chosen inherited block; first fresh child starts normal commit timers. Late old certificates do not change the already chosen entry lock or redirect an announced anchor.

The old-certificate rank guard is at least the local entry lock. It correctly does not require extending an uncommitted locked block, which could prevent recovery. The explicit strictly-earlier-view condition makes the later minimal-counterexample safety proof well founded. The initial certificate convention handles the base case without a predecessor quit certificate.

The watchdog is cumulative from each participant's own view entry, for every positive integer p, and counts fresh proposals that triggered votes. Recertification and duplicates do not count. Honest entry skew is at most Delta because the first correct quitter broadcasts a valid quit proof and all correct participants perform the same Delta delay after their local quit.

For the earliest entrant, the honest leader can enter one Delta later, wait two Delta, and deliver NEWVIEW one Delta later. Anchor votes return within another Delta, and the first fresh child arrives within another Delta: six Delta in total. Each additional fresh proposal costs at most two Delta. Thus `(2p+4) Delta` is the correct bound, including the equality case handled by message-before-deadline processing.

### 5. Certificate and handover preservation

For a correct timed commitment at time t, its proposal was echoed at `t-2 Delta`. All correct participants have entered the view and receive it by `t-Delta`. A conflicting earlier vote or a quit by that time would return authenticated evidence to the timer owner by t and prevent the commitment. Correct members therefore vote consistently, and their aggregate weight gives the certificate everywhere by t.

The forwarding rule also excludes future correct votes for incomparable leader messages: a receiver seeing the proposed committed block and an incomparable message detects equivocation and exits rather than voting. Because every certificate needs a correct vote, no conflicting same-view certificate exists. The mandatory wait after exit carries the certificate into every next-view entry lock.

This argument is source-faithful and valid under the complete-data delay assumption. It would not apply to an arbitrary partially synchronous or temporarily sluggish population; the manuscript does not assert that extension.

### 6. Unique extension and SMR safety

Consider a least-ranked later certificate inconsistent with a direct commitment. An honest vote that re-certifies NEWVIEW requires an older certificate meeting its monotone lock; that older certificate would already be a lower-ranked counterexample. A fresh-proposal vote requires a same-view parent certificate, also a lower-ranked counterexample. This contradiction establishes unique extensibility.

Any two directly committed blocks are therefore comparable. Indirect commitment of ancestors yields compatible committed logs; deterministic command and mini-block projections preserve the order.

The statement concerns semantic blocks, not random signature encodings. The manuscript explicitly treats alternate valid witnesses as evidence for the same content.

### 7. Receipts, delivery and accountability

A confirmed-stream receipt requires matching post-commit endorsements above the regional threshold. It therefore includes an honest endorsement of an actual commit. Ordinary VOTE QCs are expressly not accepted as confirmation receipts. Authenticated configuration, producer signatures, commitments and ancestry make the accepted prefix content binding; hash-linked contiguous delivery prevents conflicting local histories.

A commit seen initially by only a subset remains preserved across handovers and is eventually included in a later descendant committed by all correct members. Under the retention and live-consumer access assumptions, matching endorsements and content become available for delivery. Poll/fetch does not alter consensus decisions.

The explicit ORDER comparison domain now includes original producing view. Re-voting the same block in a later certificate view is legitimate. Positive signed equivocation is distinguished from silence or legitimate recovery. This verifier did not review a concrete L1 adjudicator implementation; only the stated semantic evidence predicates are in scope.

### 8. Progress and cost claims

An honest successor obtains every correct entry lock after its two-Delta collection and establishes a current-view anchor certificate. Fresh proposals then elicit votes within the watchdog bound and complete timers. If Byzantine views repeatedly fail, the finite cyclic list reaches an honest successor; if replacement stops, continuing nonconflicting proposal progress supplies commits. Input-specific fairness remains a separate condition.

Steady-state good-case commit latency is `delta_R+2 Delta_R`, and direct endorsement delivery to P adds `delta_P` if needed contents are already available or included in that complete-object delay. The pipeline interval is about `2 delta_R`, not the commit latency.

Quadratic forwarding/vote transmissions and a conservative cubic unaggregated-certificate control-bit bound are correctly separated. Body traffic can be quadratic under literal echoing. Post-commit endorsements add quadratic regional signature bits and linear bits per subscribed P. Retrieval, backlog/state acquisition and Byzantine flooding are not silently treated as free. The handover latency row now correctly assumes an honest successor and excludes additional failed successors.

## Verification limits

This is a paper-level local proof verification against the pinned primary source, not model checking, implementation verification, or proof-assistant certification. The fixed regional stake/synchrony, service fairness/capacity, retention and consumer reachability assumptions are taken from the statement. Cross-region CFT, stake reconfiguration, EVM execution correctness and external global finality are unproved and are explicitly left outside the manuscript's completed protocol.
