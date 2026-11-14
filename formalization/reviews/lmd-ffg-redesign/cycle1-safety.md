# Cycle 1: independent safety, validity, authority, and finality verification

## Summary

**Final Verdict:** The certificate-safety, branch-validity, authenticated-broadcast, and deterministic-checkpoint arguments are viable under the stated strong synchronous model. I found no justified counterexample producing conflicting finalized prefixes. The complete proof package is nevertheless incomplete: report snapshot timing and consumer catch-up need more precise rules, and the stated repair-time bound lacks a necessary timeout assumption.

**Artifact reviewed:** `formalization/main.tex`, SHA-256 `97d709fcfcfe9e72e93d05dfca9da679d79c82a282b27e41dbcce2412ef7f5e9`. All line numbers below refer to this frozen version. Only that manuscript and the proof-verifier skill were consulted; historical reviews were not used. No manuscript changes were made.

**Scope:** Independent verification of the safety-related definitions and proofs, including the concrete cap-two authenticated broadcast, complete-batch snapshot construction, adjacent FFG certificates, authority changes, replay handling, consumer output, and retained-data claims. This is not a proof of an unmodeled deployment, partial synchrony, execution correctness, transaction inclusion, or LMD convergence.

### List of Findings

1. **S1 — Justification Gap, medium:** At an epoch boundary the report cutoff excludes the newly starting slot, but the locally selected tree is not explicitly filtered by that cutoff before the mandatory nonempty-report rule is applied. A pre-sent Byzantine boundary-slot batch exposes inconsistent reporting requirements. Locations: lines 351–363, 450–453, 534–546.
2. **S2 — Critical Error in the stated numerical bound, medium:** The repair lemma assumes a responsive honest reply arrives within the chosen attempt timeout, without a parameter requirement that makes this true. The bound can be false while the stated network bound holds. Locations: lines 488–492, 823–835, 937–945. This is not a conflicting-finality attack.
3. **S3 — Justification Gap / catch-up specification gap, low to medium:** The consumer proof uses an installed-anchor-versus-delivered-prefix invariant more strongly than the late catch-up rules establish. The prose implies synchronization before current output, but the concrete Deliver step only explicitly replays activations; it does not explicitly install already-due anchors before allowing a subsequent preview. Locations: lines 498–509, 513–521, 814–819. This does not establish a delivered-log rollback.

## Exact assumptions used

The following are assumptions of the manuscript, not conclusions of this review.

- A fixed configured participant set and weights, with Byzantine weight strictly below one third in each region (lines 116–145, 174–179). Summing the regional inequalities also gives global Byzantine weight below one third.
- Unforgeable historical honest signatures, binding hashes, canonical semantic encodings, distinct certificate signers, and persistence of signing history. Signature encodings and signer-set witnesses do not change semantic object identity (lines 147–155, 174–179, 392–396).
- A common logical clock and complete-object delay bounds, including required ancestry, verification, queueing, and prescribed concurrent traffic (lines 157–171). These are substantially stronger than a bound on delivery of a digest.
- For the global progress induction, a fixed responsive honest set `H_G` of weight greater than `2W/3`, complete-object global delivery throughout the execution, and the full `T_G >= (N+3) Delta_G` epoch budget (lines 180–185, 547–560).
- AB uses one common instance and deterministic complete-object predicate at all responsive honest receivers; honest forwarding traffic is admitted even after a receiver has retained two values (lines 1105–1154).
- Report validation is independent of local speculative admission, and accepted report bodies and adopted prefixes remain stored and served for the configuration (lines 187–193, 372–382).
- Honest FFG signers actually follow the adjacent-link construction and persist one statement per target epoch (lines 552–555, 606–609). A generic verifier rule allowing `s<t` does not itself prove every hypothetical certificate has adjacent epochs; the signing algorithm supplies that restriction.
- Consumers start from a trusted control anchor and check the complete control ancestry, producer authority, and bytes; HEAD messages and provider suggestions are not finality proofs (lines 493–509, 610–614).

## Dependency map

1. Quorum intersection + persistent adjacent-epoch FFG signing -> uniqueness of a certified checkpoint at every epoch -> uniqueness of a certified control history.
2. Common effective parent + AB agreement -> common report vector; common vector + deterministic snapshot transition -> common checkpoint target.
3. Complete-batch validity + snapshot selection -> an extending adopted prefix; rotation and confirmation preserve that prefix.
4. Global synchronous envelope + valid common epoch processing + responsive honest quorum -> certificate production and adjacent-parent finalization each epoch.
5. Unique extending control history + complete consumer verification -> compatible append-only delivered logs.
6. Retention + an actually successful bounded pull -> data recovery. The numerical timeout part additionally needs the missing assumption in S2.

S1 affects the precise executable input construction for step 4 and the nonempty-report progress argument. It does not invalidate quorum intersection or demonstrate that two conflicting certificates can exist. S3 concerns the preview/catch-up invariant in step 5; delivered-prefix compatibility also has an independent argument from the unique certified stream.

## Detailed Verification Log

### 1. Quorums — verified

**Location, lines 671–678:** “Two quorums ... share weight `> W_Q/3`.”

Weighted inclusion–exclusion gives the stated strict inequality. Byzantine weight is strictly below the intersection threshold, so the intersection includes an honest signer. There is no integer-count substitution for a weighted quorum. The regional assumption implies the global assumption needed by FFG.

### 2. Clock authority and complete branch validity — verified in its stated scope

**Location, lines 283–296 and 680–696:** “For a fixed activation, exactly one configured producer is authorized in each slot.”

Given the activation time, region, configured list, and positive slot parameters, the local-view formula chooses one list entry. A missing prior proposal cannot stop the arithmetic progression of slots. ValidBatch requires the derived producer, exact parent, consecutive heights, batch fields, original signatures, and all bytes. Induction over the complete batch sequence gives the authenticated ordered branch.

This proves authorization of the claimed slot, not proof of the wall-clock instant at which a Byzantine signer actually signed. That is sufficient for the stated data-validity property because report cutoffs and effective-view checks govern use of that data. It correctly allows a Byzantine authorized producer to create two valid branches; the manuscript does not mistake authenticity for agreement.

**Partial-batch check:** Batches are explicitly indivisible for both fork choice and checkpoint packing (lines 278–281). A report contains whole batches, and an honest report truncates only between complete batches (lines 351–363). A Byzantine producer may sign two different envelopes with a shared mini-block prefix; those are separately authenticated complete batches and possibly equivocation evidence. Adoption of the shorter envelope does not authorize stripping the prefix off the longer envelope and treating its remainder as a fresh batch. Relative branch validation still requires an exact complete-batch parent at the installed anchor. I found no partial-batch safety counterexample under these rules.

### 3. Regional Head — verified without an agreement conclusion

**Location, lines 427–437 and 698–713:** “Continue even when all child weights are zero.”

For one finite eligible tree, the maximizing weight/digest pair deterministically chooses a child. Binding/canonical identifiers distinguish distinct semantic child batches. Every iteration strictly descends, so termination follows. If the root has a child, the result is nonempty even when all usable HEAD weights are zero.

The conclusion is correctly local to the inputs. Different latest-message maps, selectively available bodies, or equivocations may yield incompatible previews. Neither the proof nor the review assumes LMD convergence from honest voting weight.

### 4. Concrete cap-two authenticated broadcast — verified

**Location, lines 715–741 and 1125–1154:** “Continue prescribed forwarding after retaining two values.”

For each value retained by a responsive honest receiver, the needed propagation invariant is: every responsive honest receiver eventually retains that value or has already retained two values.

- If an accepted signature chain contains a responsive honest signer at position `k`, that signer previously sent the corresponding complete prefix in round `k`. Complete-object synchrony delivers it to every responsive honest receiver by that round's deadline. Each receiver retains the value unless its cap is already full.
- If the chain contains no responsive honest signer, its length is less than `n`. The receiver can append itself for the next round. If it had already forwarded that value, the earlier honest multicast gives the same propagation argument.
- A length-`n` chain necessarily contains a responsive honest signer, because the identities are distinct and at least one committee member is responsive and honest.
- A receiver with two retained values invokes the invariant for both. Another receiver with fewer than two slots cannot reject either distinct value for the cap reason, so it also reaches two values and returns bottom. Receivers need not retain the same pair when there are more than two origin values.
- If no receiver has two values, all nonempty honest sets contain the same sole value; its propagation also rules out another honest set remaining empty. An honest origin supplies only one possible origin-signed value and its first-round multicast reaches all responsive honest receivers.

Thus cap-two storage and once-per-value forwarding do not themselves cause disagreement. Early/late chain handling, length/round correspondence, detached witnesses, and admission of honest forwarding are essential premises. No standard broadcast theorem has been silently substituted for the concrete algorithm.

### 5. Report snapshot and global determinism — verified conditional on a well-defined valid snapshot

**Location, lines 571–599 and 743–761:** “A selected leaf is the endpoint of a reported path.”

Every valid relative report begins at the common effective parent and consists of complete linked batch edges. Semantic deduplication makes the union a rooted batch tree. Each origin contributes once to its own reported endpoint. The deterministic Head rule chooses a unique root-to-leaf path. A leaf occurs in some original report path, and that report includes its entire path from the root, so the selected path respects one report's bounds despite the larger union-verification cost.

There is no hidden assumption that the regional reports are compatible. HOLD/CONFIRM retain the parent; ROTATE deliberately retains the parent rather than the unadopted snapshot candidate. Readiness and cyclic selection are deterministic at the semantic level. The convention that certificate witnesses do not enter semantic object identity is needed when the same statement has different sufficient signer sets.

**S1 — report cutoff versus selected tree.**

**Quoted requirements:** “All its producer slots end no later than the epoch's start” (lines 353–354); “If the selected branch is nonempty, the admitted prefix must be nonempty” (lines 360–361); “Buffer future-slot objects without selecting or voting for them” (lines 450–452).

A permitted boundary trace is:

1. The effective adopted tip is `alpha`; the locally eligible completed-slot tree has no child.
2. A Byzantine producer authorized for slot `u`, which starts exactly at the next epoch boundary `t`, signs a valid batch extending `alpha` and sends it before `t`.
3. Honest receivers buffer it while it is future-slot data. At `t`, slot `u` is current. The text gives no explicit rule that snapshotting must precede release of buffered boundary-slot objects, or that the snapshot tree must be filtered to slots whose **end** is at most `t`.
4. If that batch becomes eligible, Head selects a nonempty branch. Its only batch ends at `t + tau_R`, so no nonempty prefix satisfies ValidReport. The mandatory nonempty-prefix sentence and the report cutoff cannot both be followed for this selected input.

Ordering honest new production after G1 does not by itself exclude already buffered Byzantine data. A similar definition issue applies to which HEAD records are usable for a cutoff-restricted snapshot. This is a missing boundary-state rule, not an attack against certificate uniqueness. The current proof does not establish the exact reporting step for this case; downstream per-epoch/progress claims are conditional on resolving it.

### 6. FFG certificate safety and activation — verified

**Location, lines 603–625 and 763–792:** “two different certificates for one target epoch would intersect in an honest signer.”

Every constructed FFG certificate has a target epoch and source at the immediately previous epoch. Two different certified statements at a target epoch require an honest double vote by quorum intersection. Persistent signing history rules that out. Starting from the trusted anchor, induction over adjacent target epochs rules out divergent certified control histories. This certificate-safety argument does not need AB to make honest nodes choose the same target; AB is needed to ensure that a quorum actually forms on one target in each synchronous epoch.

The globally responsive honest set supplies enough weight to form the common link once the common target and complete data exist. The timing budget covers AB, voting, certificate export, and the next-boundary margin. The target is justified, and its already-justified direct parent is finalized.

**Activation and circularity check:** A ROTATE at `C_r` becomes finalized by the link to `C_(r+1)`; its successor starts at epoch `r+2`. The activation proof therefore refers to already-existing strict ancestors of the next target. It need not certify its own target. Pending CONFIRM retains the old adopted tip, and activation clears the pending package, changes the view domain, and retains the tip. The same reasoning covers a same-region restart. There is no justified self-referential activation certificate defect here.

### 7. Old-view replay and retained authority — verified for synchronized/current output

**Location, lines 291–296, 372–382, 498–521, 615–624.**

New relative report batches must use the effective global view and extend its adopted tip. An old-view batch or delayed REPORT cannot be counted as a new current-view report merely because its height matches: configuration, view, epoch, parent checkpoint, slot authority, and digest are checked. Replayed ancestor proofs cannot reduce an installed anchor. A producer cannot keep the old global view active in the accepted new control history by continuing to sign old slots.

Historical bytes and signatures remain valid evidence and may be delivered as part of an already certified prefix. Retaining that evidence is compatible with excluding it from current fork choice. Stale clients do not prove current authority simply by possessing a formerly valid activation proof. The missing-checkpoint pause rule is necessary here.

### 8. Consumer delivered-prefix safety — verified core argument; catch-up invariant incomplete

**Location, lines 799–821:** “Its delivered log is therefore a prefix of this single finalized stream.”

Given the unique certified control history and monotone adopted prefixes, complete verification before appending positions makes any two delivered logs prefix-compatible. Persistent delivered state and the append-only D5 rule prevent a previously delivered position being replaced. SetPreview is explicitly replaceable and is not itself evidence of finality. No EVM correctness conclusion follows.

**S3 — exact late catch-up obligation.**

**Quoted proof:** “only above the installed anchor, which extends every earlier delivered prefix” (lines 814–815); “applying its intervening activations and anchor transitions before current output” (lines 815–817).

For an on-time consumer, `C_e` is installed at the start of epoch `e+1`, before its finalization later in that epoch, so the quoted invariant is justified. A stale consumer can instead learn a finalized `C_e` after missing that installation boundary. D5 permits verification and delivery of its complete prefix and explicitly says to replay intervening **activations**, while D4 schedules anchor installation “at its specified epoch boundary.” The executable late-arrival case and the resume condition are not explicit.

The surrounding phrases “until synchronization” and restoring the current anchor strongly suggest the intended behavior. They do not establish an attack on the append-only delivered log. They do leave the proof relying on an unstated sequencing invariant: before resuming SetPreview/current-authority output, the consumer must install the latest required already-due anchor, and that anchor must extend the delivered tip. Delivery may be allowed while current-authority output stays paused; in that case the proof's invariant needs the corresponding scope. This is a specification/proof gap rather than demonstrated conflicting delivery.

### 9. Retention and repair — retention verified; timing bound not established

**Location, lines 823–838:** “the next admitted honest response verifies within that deadline.”

All responsive honest global voters retain the complete selected prefix before voting and retain it for the configuration. Thus a globally justified checkpoint under the global envelope has responsive honest holders. This conclusion uses actual retention and network availability, not a digest or a HEAD signature alone.

**S2 — absent timeout inequality.**

Neither D2 nor the timing section requires `theta_R`, `theta_G`, or the consumer timeout to cover a complete successful request/response/verification attempt. A network delivery bound `Delta` does not imply delivery within an arbitrarily smaller timeout `theta`.

For a concrete numerical counterexample to the stated lemma, take the first attempted supplier to be responsive and honest (`a = 0`), with `theta = 1` time unit. Let its complete valid response take 2 time units on a path whose permitted delay bound is at least 2. The cryptographic, retention, and network assumptions hold, but repair cannot finish within the claimed `(a+1) theta = 1`. If the timed-out reply is discarded, the attempt instead fails despite the supplier being responsive; that also defeats the proof's inference that the next responsive supplier succeeds within the deadline.

This is a false numerical inference unless a successful-attempt deadline premise is added. It does not imply that the bytes are unavailable forever or that any invalid bytes can pass verification. Claims using `F_R`, `F_G`, or corresponding consumer repair terms inherit this missing timing premise.

### 10. Conflict evidence — verified

**Location, lines 645–663 and 912–923.**

The verifier compares semantic statements in exact persistent signing domains and checks both signatures. PROPOSE and ORDER conflict pairs violate the single-batch-per-slot rule; HEAD and REPORT pairs violate their one-statement domains; FFG pairs violate the persisted target-epoch or surround rule. Different signature encodings on the same semantic object do not create evidence. Forwarding two conflicting AB-origin values is deliberately not classified as a forwarding signer's equivocation. Silence, ordinary head changes, and reorgs do not by themselves yield the claimed cryptographic blame.

## Consequences and limits of the verdict

- I found no justified conflicting-finality, old-view authority-reuse, partial-batch adoption, or activation-circularity attack under the frozen manuscript's assumptions.
- The cap-two AB construction withstands the independent propagation check; retaining different pairs at different receivers is harmless because their common result is bottom.
- The synchronous global certificate-production and progress package should not be called fully verified until the report boundary input is specified consistently and the repair timing premise is explicit.
- Consumer append-only compatibility follows from the certified-prefix argument. The stronger preview-floor/catch-up statement needs an explicit resume/anchor sequencing rule or a scoped invariant.
- No conclusion here extends to a stalled global envelope, unspecified global-epoch recovery, availability from a mere HEAD signature, eventual inclusion of a chosen preview, or execution correctness.
