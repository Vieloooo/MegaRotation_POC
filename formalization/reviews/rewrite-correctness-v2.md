# Summary

**Final Verdict:** The repaired frozen artifact's supplied correctness arguments are valid under its explicitly stated fixed-configuration, synchronous-delivery, retention, healthy-successor, and workload assumptions; all three v1 findings are closed, and this re-verification identified no remaining Critical Error or Justification Gap in the changed proof-relevant sections or their checked dependencies.

**List of Findings:** None remaining in the reviewed v2 artifact. This is a mathematical verification of the supplied reference model and algorithms, not a claim of deployed compatibility, asynchronous safety of signed-chain completion, implementation correctness, or an unconditional throughput guarantee.

Artifact: `formalization/build/rewrite/review-v2.tex`.

SHA-256: `8b1a77c22119932f8a24bd69e4937abd6f42da6fd51029f658754080ad9ad551`.

Mode: independent verifier only, following `/Users/jin/.codex/skills/proof-verifier/SKILL.md`. I compared the entire frozen v1-to-v2 diff, read the changed proof-relevant sections in their current context, and rechecked their dependencies on the authenticated-broadcast, regional-completion, global-transition, safety, and liveness arguments. No manuscript source was edited.

The following conditions are assumptions supplied by the artifact, not conclusions proved by the protocol: established L1 authorization and authenticated identities; unforgeability and binding commitments; a common logical clock; complete-object delivery within the configured bounds; an always-responsive global honest cohort above two thirds of weight; local Byzantine weight below one third; persistent signing history and uncompromised historical keys; no pruning of the required history; bounded request service; sufficient complete-object sizes and an appropriate workload/backlog bound; and an eventually reached healthy eligible successor.

# Detailed Verification Log

## 1. Closure of v1 F1: authentication of the exact broadcast value

> Lines 326–332: the proposal witness includes `Sign_ell(PROPOSE,I,H(x))`, and `ValidProposal` checks “the leader's PROPOSE signature on this exact batch.”

The original strict-prefix substitution is no longer admissible. Changing the batch endpoint or another proposal field changes H(x), and a relay with only the mini-block ORDER signatures cannot produce the required proposal signature. Because an honest leader originates one proposal for the instance, every accepted proposal in that honest-broadcaster instance has the same signed value. This restores the inference at lines 902–905 that all responsive honest members vote for the leader's eligible proposal and decide it after two complete-object delays.

> Lines 451–452: all received signed statements pass authentication and conflict checking before ordinary conflict/duplicate discard.

This ordering is consistent with the evidence algorithm. It does not authorize a conflicting authenticated statement for ordinary protocol processing, and it does not replace the new exact PROPOSE-signature requirement. The proposal witness also remains available in completion and external RC verification, so the repair is not confined to the initial direct-send path.

**Disposition:** v1 F1 is closed.

## 2. Closure of v1 F2: prescribed honest READY behavior

> Lines 667–672: an honest member sets READY true exactly when it holds the complete verified parent prefix/evidence and is ready to participate locally; “A healthy eligible member with these data must set true.”

The previous all-false READY execution is no longer an honest execution when a synchronized healthy eligible standby exists. The predicate references the exact retained parent tip, and its next-generation binding is interpreted after any pre-transition activation. A READY statement cannot thereby be replayed for another parent or generation. Honest members lacking the data continue pulling instead of making an unsupported readiness claim.

The proof step at lines 1011–1014 now invokes behavior actually required by the algorithm. READY still certifies readiness at its signing point; it does not promise that a region can never fail later, as explicitly stated at lines 773–775.

**Disposition:** v1 F2 is closed.

## 3. Closure of v1 F3: global and regional responsive quorums

> Lines 202–205: a healthy eligible successor j satisfies `wt(H_G intersect V_j)>2W_j/3` and the regional communication bound on that set.

This supplies the missing connection. That cohort participates in the global report protocol, has the adopted parent state under the global induction, must issue true READY when eligible, and has sufficient regional weight for the new regional protocol. The global epoch budget delivers its activation proof before the fixed start. The same set can therefore witness the regional live-cohort assumption after activation.

The regional Byzantine bound remains separate and continues to support exported-certificate uniqueness during a regional outage. The new condition does not attempt to derive local Byzantine resilience from the global weight bound alone.

**Disposition:** v1 F3 is closed.

## 4. Authenticated broadcast and weighted completion

> Lines 842–882 and the unchanged complete authenticated-broadcast appendix: timed signed chains, cap-two semantic sets, common output vector, and a strict-supermajority input selector.

The proof-relevant algorithm and cap-two argument are unchanged from the independently checked v1 construction. The new PROPOSE-signature check strengthens the deterministic witness predicate used by completion without changing the dissemination argument. Both the semantic value and the predicate remain common within the fixed instance context.

For each honestly retained value, a prior responsive honest chain signer or the current responsive receiver supplies timely dissemination; a length-n chain necessarily contains a responsive honest identity. Every other responsive receiver either retains that value or already retains two values. Applying the property to two distinct values yields unanimous bottom output; otherwise all nonempty sets share the same singleton. Honest-origin validity follows from the unique origin signature. Witness variation does not create a new semantic value.

Weighted completion then has agreement from the common origin vector, unanimity from the responsive honest weight above two thirds, and external validity from the full checked witness. No bound on the number of Byzantine identities is inferred from the stake bound; the algorithm continues to use n identity rounds.

## 5. Regional state, scheduling, and exported confirmation

> Lines 310–318: R_0 is an INIT record with the generation, activation digest, inherited tip, initial view and leader; INIT requires activation evidence rather than a fabricated DECIDED quorum.
>
> Lines 449–450: `t_{a,u}=t_a+(u-1) tau_R(i)`.

The initial record and all subsequent scheduled starts are common functions of certified activation and fixed configuration. The explicit schedule does not permit a new instance to bypass the existing requirements for prior completion and its predecessor RC. During a live interval, the established spacing `(n_i+5) Delta_R` is sufficient for those prerequisites.

> Lines 630–639: the deterministic transition stores H(x) for a non-bottom decision and applies a coincident planned/failure change only once.

This resolves the value-versus-digest representation explicitly. The outcome hash commits to the actual predecessor, tip and next authority; persistent one-DECIDED-per-instance signing still gives a unique exported chain. Its uniqueness does not require an unavailable region to complete every internal broadcast.

> Lines 645–647: a recovering member restores its own persistent signing history and pulls missing public state.

The revised wording correctly separates private anti-double-signing state from transferable public certificates. It does not infer an honest member's complete signing history from peer reports.

## 6. Speculative HEAD protocol and evidence

> Lines 433–463: HEAD slots equal regional instance numbers; a member signs once on its first valid proposal; conflicting same-slot statements exclude that member's weight; HEAD affects only the speculative display.

The HEAD protocol has no dependency that can prevent regional voting, completion or global adoption. Its new explicit originating rule is compatible with the signed-message evidence predicate. A later unambiguous slot can legitimately replace an earlier one. Speculative display differences therefore do not enter the certified-prefix proofs.

> Lines 811–824: bounded conflict-evidence propagation, fixed quorum weights, and an explicit exemption for relaying multiple AB-origin values.

The changes limit communication without changing evidentiary validity or silently removing stake from an active quorum. Required authenticated-broadcast relay behavior is still not misclassified as consensus double voting.

## 7. Global transition, activation, and external verification

> Lines 399–408: `ValidCheckpoint` checks the parent, report commitment, complete non-bottom reports, and equality to the deterministic transition; a global vote certificate separately authenticates endorsement after report agreement.

This is a sound separation. A report vector alone need not prove private arrival-time facts. A valid global quorum contains honest signers who ran the protocol and checked the target. External receivers verify the deterministic state transition and transferable vote/finality certificates, obtaining missing ancestry back to their trusted anchor.

> Lines 663–690: earliest contiguous bounded reports; common report agreement; target derivation; adjacent FFG voting; certificate dissemination; fixed next boundary.

The revised reporter selection is deterministic for each node's known data and does not require their knowledge to be identical. Authenticated broadcast produces the common report-output vector. The chosen longest valid extension is then common because all reported extensions lie on the unique certified regional chain.

> Lines 705–726: pending rotation only CONFIRMs; ROTATE preserves the parent's tip/cursor; otherwise the selected extension is adopted.

The transition retains the safe readiness boundary. It does not adopt an extension that was absent from the readiness statements. Gap-only cursor advancement does not delete stream data. Exact proof/signature encodings remain witnesses outside semantic identifiers, so different local certificate representations do not create different checkpoint values.

> Lines 751–766: C_r is finalized by its child link; the new region starts at epoch r+2; activation resets the local initial state and failure counter while preserving the retained stream tip.

The timing and state dependencies are non-circular. Finality exists before the fixed activation boundary; the proof is consumed in a later descendant's pre-transition. Neither C_r nor C_{r+1} is rewritten. The healthy-successor definition now guarantees that the globally informed regional cohort is sufficient for a common regional start. These changes preserve the global induction at lines 925–948.

## 8. Consumer authority, permanent prefix safety, and repair

> Lines 428–430 and 574–592: persist the finalized control anchor and activated generation; require the current generation and reached activation time for Preview; queue a valid future activation; reject authority regression from an ancestor/replayed proof.

These guards prevent a historical valid rotation from rolling the consumer's authority backward or a future activation from becoming effective early. Complete new control evidence is processed relative to a verified anchor, and generation advancement is monotone. Historical records remain usable as evidence and to verify retained data; they are not fresh authority assignments.

The Deliver path still checks the full contiguous certified range and only appends previously undelivered positions. The global chain retains every adopted prefix across a rotation. Thus the permanent-prefix argument at lines 954–977 continues to apply: only an unretained Preview suffix can be removed, not a delivered entry.

> Lines 570–573: cross-wrapper body caching, one outstanding attempt per missing body, cancellation on successful push or pull, and accounting for an in-flight response.

The transport changes do not change the confirmation predicate. Data is accepted only against the original certified context and body commitment. Caching is by body content rather than merely by one report-wrapper identity, so N reports of the same range do not require N downloads at one receiver. The repair lemma remains a sum of bounded attempts and explicitly retains its honest-holder premise.

## 9. Liveness, workload, and timing repairs

> Lines 1055–1066: Q is a workload bound for a target and its unadopted predecessors; later outcomes cannot overtake that target; every individual outcome and its relative evidence must fit.

The revised argument no longer treats maximum packing capacities as a guaranteed service rate. Once the target's required prefix is retrievable and a continuing active generation can report it, longest-fitting contiguous reporting adopts at least one pending outcome per live reporting epoch. With J_0 predecessors/outcomes up to the target, Q at most J_0 is a conservative target bound. A uniform bound over time additionally requires the stated workload envelope. A Preview cancelled at handoff retains no unconditional Deliver promise.

> Lines 1067–1078 and 1130–1135: startup uses `(n_max N_R+1) tau_max`; intermittent faulty leaders are bounded by planned boundaries, not by a failure counter they can reset.

Correct. Every tenure reaches a planned boundary within N_R instances, even when a Byzantine leader occasionally produces a valid batch and resets the consecutive-failure count. Continuously unproductive tenures can still use the smaller K_R bound. The minimum-tenure condition explicitly includes startup, synchronization and extra global epochs; the revised larger startup class is conservative.

> Lines 1048–1053 and 1094–1101: announcement emission period is separated from propagation via A_R, A_G and A_P; the delivery bound includes discovery on the consumer path.

The revised quantities consistently account for both waiting for an announcement and delivering it. Multi-chunk transfer remains a sum of attempt bounds. The geographic recovery formula at lines 1143–1159 explicitly ends at the first successor Preview, and the extra finalized-delivery interval is added separately.

With mandatory READY and the healthy-successor cohort condition now supplied, the liveness proof at lines 996–1025 can use its stated premises: finite unproductive local tenures are bypassed; a healthy standby can supply a ready quorum; ROTATE/CONFIRM yields a fixed common start; sufficient tenure admits a reportable batch; global adoption/finalization and bounded repair yield delivery. Repetition under continuing health and the workload assumptions gives unbounded growth. It does not imply selection of every submitted transaction.

## 10. Communication and payload bounds

> Lines 1196–1204: explicit HEAD, announcement, pull, and evidence costs are additional interval costs.

The additions are consistent with their generation rules and do not alter the cap-two consensus-round cost. The admitted-traffic qualification remains necessary; unrestricted Byzantine wire flooding is not assigned a finite protocol cost.

> Lines 1212–1224: `B_body <= M_0+(a+1)M_r <= (a+2)M`, with the last inequality expressly conditioned on `M_0,M_r<=M`.

The first term counts the primary copies; the second counts bounded sequential failed/successful repair responses. A late primary push can race a repair response, so the extra copy is legitimate. The simplified `(a+2)M` expression is conditional as stated, not a claim for arbitrary repeated primary pushes or repeated repair campaigns. Pure pull retains the `(a+1)M` special case. Control bytes and route-length multiplication remain separate.

> Lines 1237–1243: retained storage is `O(8X_history+A_history)` bits, with explicit units and no pruning.

The conversion and storage scope are consistent. The protocol assumes retained history; it does not infer recovery from a root alone.

## Verification boundary

This verdict applies to the frozen v2 hash above. It relies on the reference model's strong synchronous and capacity assumptions and its fixed membership. It neither weakens those assumptions to eventual synchrony nor extrapolates the result to dynamic membership, implementation details, downstream execution, or publication. No new lemma, algorithm rule, or unstated assumption was inserted during verification.
