# v3.7 public FINAL-kernel analysis

This is a narrow design analysis of the root's proposed external receipt validator, following `v37_carrier_refinement_design.md`. It applies the proof-verifier workflow to the supplied honest-FINAL argument and supplies the explicitly requested candidate lemma and algorithm boundary. It is not a new whole-protocol review. Paths below are relative to `snapshot/` unless specified otherwise. No manuscript or snapshot was edited.

## Summary

**Final Verdict:** The proposed honest-FINAL implication is valid under the frozen fault, cryptographic and sound-validation assumptions, provided correct regional replicas keep the existing full Vote/Ready/admission checks and only timer-origin commits emit new FINAL signatures. It supports a sound **new external verification rule for the hash-bound input history**. It does not establish validity of arbitrary publisher-supplied proof sidecars or native state/output claims, and it must be described as an explicit algorithm change rather than as the old receipt definition already meaning this rule.

**List of findings and boundaries:**

- **Correct step, no circularity:** A raw matching FINAL set of distinct weight greater than `F` contains a genuine correct signer without invoking `def:receipt(iii)`. The frozen lifetime key-compromise bound and signature unforgeability justify this step.
- **Correct step:** Every new correct FINAL traces to a timer started only after full proposal/ancestry validation. A commit triggered by a receipt sends no FINAL. Therefore the certificate implies that a complete valid historical witness existed for the exact hash-bound DATA, regardless of the publisher's chosen alternative proposal/admission witness.
- **Required explicit protocol change:** The frozen external rule requires the observer to check condition (iii) from evidence. A kernel validator discharges the semantic validity claim through the derived honest-signer lemma instead. This changes the verification algorithm, but requires no stronger environment or adversarial assumption. Leaving the old external instruction unchanged while silently skipping it would be a specification contradiction.
- **Critical Error if overextended:** “The raw FINAL set validates every attached native execution/state claim” is false unless those claims are bound by the signed DATA commitment and within the honest signer's sound validation predicate. The input-prefix, execution-state, DA-correspondence and Ready obligations remain separate.
- **Critical Error if overextended:** Threshold `>F` alone is not a quorum-intersection proof of agreement. Different such sets can have different correct signers. The existing regional safety proof is still needed for compatibility of two genuinely committed results.
- **Not established by this lemma:** Binary carrier coverage, arbitrary namespace discovery, missing DATA availability, complete source-handler running time and native preparation time. It removes a particular dependence on publisher-selected `AUTH_(r,g)` evidence; it is not a proof of the whole public-response budget.

For the narrowly specified rule below, I found no in-model invalid-DATA counterexample. The counterexamples in Section 6 delimit stronger claims that the proposal must avoid.

## Detailed verification log

### 1. Raw threshold signatures, before any receipt-validity definition

The supplied argument starts with “threshold FINAL >F under fixed AUTH[r,0].” Formalize this as a new syntactic predicate, not as “a valid receipt.” Let `C` be the immutable session context obtained from the already authenticated incoming boundary. It fixes:

\[
 C=(\mathsf{chain},\mathsf{domain},\beta,r,i,\mathit{cfg}_r,
       V_i,(\omega_v)_{v\in V_i},F_i,A,H_{\beta,r}).
\]

Let

\[
 m_D=(\mathsf{FINAL},\mathsf{chain},\mathsf{domain},\beta,r,i,
       \mathit{cfg}_r,h(D),\mathsf{Hash}(D)).
\]

The displayed FINAL notation in the manuscript suppresses some common signed-context fields; they must all be checked here. Define `RawFinal(C,D,Sigma)` by:

1. The bytes of `D` hash to the commitment in `m_D`; its named DATA fields agree with the statement.
2. Every counted signature verifies on this exact message, under a distinct member key in `C`.
3. The total weight of counted distinct signers is strictly greater than `F_i`.

Do **not** include historical admission validity, native validity or `def:receipt(iii)` in this definition. Do not let the sender choose `C`; derive it from the boundary and authenticated immutable initial authority first.

The frozen Model states that the adversarial weight in that snapshot is at most `F_i`, that membership/weights are fixed through all recoveries, that the compromise bound counts every snapshot key compromised while certificates remain accepted, and that signatures are unforgeable (`baseline/model_and_assumptions.tex:135–153`; unchanged v3.6 main `353–371`). Consequently a signer set of greater weight contains at least one key belonging to a correct, uncompromised participant. Its signature is a real correct-protocol action on `m_D`.

This inference uses neither an observer's current receipt-validity test nor a majority-intersection claim. It is unaffected by the absence of `g,t` in FINAL: those fields are deliberately omitted, while session/domain/configuration and exact DATA hash remain bound (baseline main `749–768,791–812,827–828`; v3.6 main `784–803,826–851,865–868`).

### 2. Trace a genuine correct FINAL to an actual timer

The supplied argument says the signer sends “only after timer or ancestor commit with full valid regional admission/native checks.” The operative rules support precisely that statement:

- Vote stops for a missing dependency or invalid proposal and starts a timer only after complete validation (baseline main `1043–1064`; v3.6 main `1103–1124`).
- Complete validation checks signatures, authority, admission, block type, quota, positions, L1/Espresso references, parent rules/QCs, hash links and application validity throughout the required ancestry. Relevant ancestor data and signed proposals are present (baseline main `881–896`; v3.6 main `920–935`).
- On timer expiry, Commit commits the proposal block and uncommitted ancestors and sends FINAL for same-session DATA thereby committed. A DRAIN timer can generate FINAL for terminal DATA, never FINAL for DRAIN (baseline main `1067–1073`; v3.6 main `1127–1133`).
- Receipt-triggered commitment, including Completion, sends no new FINAL (baseline main `834–839`; v3.6 main `873–878`).
- Recovery/ANCHOR certification does not itself start a commit timer (baseline main `744–747`; v3.6 main `779–782`). The retirement drain preserves surviving timer-origin FINAL generation rather than inventing a new signature source (`sections/rotation_v3_4_retirement.tex:92–104`).

Thus the correct signature has an earlier Vote event on some proposal `P_X`, an associated surviving timer, and a timer-origin commit of `X`. The named DATA `D` is `X` when `X` is DATA, or a DATA ancestor of `X` in that same session. The signer's earlier validation covered this exact `D` and its required ancestry. A later timer can also encounter an already committed ancestor; any allowed re-transmission of an existing FINAL still traces to its original valid signature. No receipt-forwarding step can create the first correct FINAL for an otherwise unsupported statement.

This is an operational provenance proof. Starting with “a valid receipt has a correct signer” would be insufficient here because “valid” already includes the conclusion being replaced. The argument instead begins with the raw cryptographic threshold and follows the signer's actual rules.

### 3. What the validated timer implies

The supplied argument proposes “proving honest-FINAL soundness ... instead of replaying publisher-selected PROPOSE/AUTH[r,g].” The precise consequence is existential:

\[
 \mathsf{RawFinal}(C,D,\Sigma)
 \Longrightarrow
 \exists v,P_X,W,t_v,t_c:\quad
 \begin{cases}
  v\text{ is correct and signed }m_D,\\
  v\text{ fully validated }(P_X,W)\text{ at }t_v<t_c,\\
  v\text{'s timer committed }X\text{ at }t_c,\\
  D\text{ is the named same-session DATA in that committed ancestry.}
 \end{cases}
\]

`W` is the genuine complete witness used by the correct replica. It includes the necessary admission/recovery authorities and native input checks. It is not necessarily the witness `W'` carried beside the certificate by a later publisher.

The frozen Model assumes the regional/application validity interfaces are sound (`baseline/model_and_assumptions.tex:285–291`; v3.6 main `503–509`). Applying that assumption to the **actual full check executed by the correct Vote event**, rather than to the proposed new kernel, gives the historical validity of the hash-bound DATA and its validated ancestry. There is no assumption here that the new kernel is already sound.

Hashes are binding. Therefore a public reader that has the DATA bytes matching `m_D` and an exact hash-linked ancestry path is looking at the same DATA objects the honest signer committed. It cannot be shown a different payload or predecessor under the same signed hash. The reader need not establish that an arbitrary accompanying `PROPOSE_(g',t')` was the genuine proposal that caused this commitment.

The external conclusion is **existence of a valid historical admission path for these exact DATA objects**, not “every proposed admission path for this hash is valid.” FINAL's deliberate omission of `g,t` makes that distinction necessary. The genuine path can use `g`; a publisher can attach a bogus path using `g'`. Ignoring the bogus sidecar does not change the authenticated DATA, its actual provenance or the existence proof.

### 4. Noncircularity across receipts, boundaries and anchors

Two forms of induction suffice to make the previous step formal.

First, trace a correct FINAL back to its earlier timer and Vote. Receipt-triggered commitments emit no FINAL, so there is no chain in which a certificate justifies its own first honest signature. This is exactly the noncircular provenance structure already used for receipt-based commitment safety at `sections/rotation_v3_4_retirement.tex:353–360`, though that passage starts from a fully valid receipt and must not be substituted for the raw-threshold proof above.

Second, use a joint invariant over finite execution prefixes for the broader system:

1. The genesis/authorized starting anchor has the validity assumed by the existing checkpoint contract.
2. Correct regional Vote events continue to require the original full validation and sound native input interface. They do not use the proposed kernel as a shortcut for missing local execution state, incomplete admission bundles or unverified QCs.
3. A new public-kernel acceptance is justified by a genuine correct signature generated at an earlier timer event, which in turn used earlier complete validation. Any prior accepted anchor it depends on is covered by the induction hypothesis.
4. Public acceptance also checks that the exact hash-linked path extends the current authenticated anchor and current `G`; it cannot substitute a different starting state merely because a signature threshold exists.

All dependencies in this proof point to an earlier event or an ancestor in a finite validated witness. No induction step assumes that the very kernel acceptance under consideration was already valid. If regional full validation is also changed to trust an unexecuted/unvalidated kernel shortcut, this proof must be revisited; that is outside the proposal assessed here.

### 5. Candidate lemma and external algorithm

**Lemma (honest-FINAL kernel soundness).** Assume the frozen fixed-snapshot fault bound including accepted-lifetime key compromise, signature unforgeability and hash binding, sound regional/native input-validation interfaces, and unchanged correct regional full-validation and FINAL-generation rules. Let `C` be the historically authentic incoming context of session `(beta,r)`. If `RawFinal(C,D,Sigma)` holds, then a correct replica actually committed this exact DATA `D` by a direct timer or as an ancestor at a timer event, after completely validating a genuine regional proposal/history witness containing `D`. In particular a valid historical regional witness exists for `D` under `C`, and every exact same-session DATA ancestor covered by that witnessed path inherits its commitment and input-validity conclusions.

**Proof.** The threshold and fault bound provide a correct signer; unforgeability gives its actual FINAL action. The unchanged generation rules give an earlier timer-origin commitment and Vote-created timer. That Vote passed full proposal/ancestry validation, so soundness of those original checks gives the claimed genuine witness and input-validity judgment. Hash binding identifies the public DATA/path with the objects in that witness. Receipt-driven commitment cannot create the first correct FINAL, which prevents circular bootstrap. Same-session coverage follows from the timer's ancestor commit rule and exact parent links. ∎

This is a validity/provenance lemma, not a proof that all correct replicas have committed, that they will all finish committing by a deadline, or that two correct timers can never conflict. The separate regional safety theorem supplies that last compatibility fact.

**Candidate `VerifyPublicResult`.** Its proof-bearing input is the following kernel, extracted from the existing complete result; it is not a new network message or certificate:

- The authenticated incoming boundary and fixed initial authority `AUTH_(r,0)`, or their already verified interpreter identities.
- The complete DATA body `D` named by a raw FINAL set, that set, and the exact bodies needed to link a covered next DATA `B` to `D` and back to the required accepted/incoming anchor. Keep supporting DRAIN bodies if the hash path traverses them. Do not pretend DATA-only heights remove physical parent links.
- Required public selection/source information for the **global** acceptance gates and boundary, obtained from canonical source replay.

The algorithm:

1. Derive the region, snapshot, incoming anchor and quota from canonical history; verify domain and the exact raw FINAL statement under that snapshot. A sender-supplied authority or region cannot choose its own voter set.
2. Check exact DATA/hash/parent linkage and same-session coverage, original block provenance, `h(B)=h(G)+1`, quota and the authenticated incoming anchor. A later session's receipt does not replace a receipt for an earlier session; preserve baseline main `842–856`.
3. Check current branch/duty permission, historical boundary/selection, strict Espresso selection-reference precedence, and the post-Force containing-block rule. Preserve the per-DATA durable commit and terminal-before-successor ordering.
4. Use the lemma for historical regional admission and native **input** validity already certified by the honest signer's actual full validation. Do not replay a publisher-selected alternative `PROPOSE`, recovery-QC chain or `AUTH_(r,g)` solely to re-establish those existential facts.
5. Apply any independent application/publication checks whose outputs are not part of that certified input-validity judgment. In particular, do not accept a supplied execution-state sidecar, rollup state claim, DA correspondence claim, or Ready state merely because this kernel passed.

The old full evidence can remain in the complete published result and retained archives. The new public rule just does not let an unneeded alternate witness make the mandatory acceptance operation fetch an arbitrary recovery authority. Signed-material inspection and full admission checks in the regional protocol remain unchanged. Optional archival audit of a publisher's bogus witness must not become a public acceptance dependency.

## 6. Counterexample checks and precise scope

### A. Publisher chooses a bogus alternate recovery witness

Let a genuine `D` have a threshold FINAL set after correct validation in generation `g`. A publisher attaches a syntactically signed alternative proposal using generation `g'`, plus an unavailable or nonexistent `AUTH_(r,g')` name. The original `D` omits `g,t` from its block tuple, and FINAL omits them as intended. This is the specific obstacle identified by the retention audit.

The proposed rule is sound if it authenticates the fixed snapshot and exact DATA and does not claim the alternative witness valid. The raw threshold proves that some genuine witness existed; the alternate path is irrelevant to that conclusion. There is no need to fetch its generation authority for public DATA acceptance. A regional participant that actually needs to execute or audit that alternate regional message must still reject/wait according to the unchanged full-validation rules.

If `D` itself is invalid under every authentic historical witness, this attack fails: a correct FINAL signer could not have produced the required signature. Thus there is no in-model invalid-DATA counterexample here.

### B. Forged native sidecar beside genuine DATA — real overextension counterexample

Let valid hash-bound transaction inputs `D` and their genuine FINAL set be published with an auxiliary claimed post-state `z'` that is not bound by `Hash(D)`. The correct signer's actual validation used the correct state or native context `z`, not `z'`.

The kernel remains valid for `D`, but says nothing about `z'`. An algorithm accepting `z'` solely from the certificate would be unsound. The same example applies to an unbound output receipt, withdrawal claim or arbitrary execution-state snapshot. This is why the frozen distinction between input-prefix finality and native state/output finality remains necessary (baseline main `1422–1427`; v3.6 main `1444–1449,2209–2217`; frozen Model `285–291`).

If the actual native protocol hashes such an output claim into `D` and the correct full-validation predicate soundly checks it, the lemma covers that **specific bound claim**. That cannot be presumed for unspecified native context.

### C. Two threshold sets — invalid replacement for regional safety

With three unit-weight members, `F=1`, and one Byzantine member `b`, `{b,h1}` and `{b,h2}` both exceed `F` and need not share a correct member. Raw certificate arithmetic alone therefore does not rule out two different correct signers on different blocks. Their inability to commit incompatible blocks comes from the regional timer/recovery safety argument, not from this threshold lemma. Keep that proof dependency explicit.

This is not an in-model attack on the unchanged regional protocol; it is a counterexample to a threshold-only agreement argument.

### D. Closed beta or wrong global prefix

A perfectly genuine raw FINAL for an old closed version, or for a valid speculative branch not extending current `G`, remains historically meaningful. Accepting it without the existing current-duty/boundary/path gates would break the public rule. The kernel does not grant append permission and does not replace Force's strict prelude cutoff.

### E. Missing payload or wrong covering path

A threshold FINAL authenticates a hash, not availability. Without the named DATA and path bodies, the public reader cannot determine the next input or check extension of `G`. Supply of hash-linked paths is essential. A covering root's signature cannot authenticate an arbitrary same-height sibling, and the current rule must not use a later-session root to substitute for an earlier session's receipt.

### F. Public L1 controls versus redundant regional L1 witnesses

The argument can remove public fetching of a publisher's alternate recovery authority. It does not remove authentication of the fixed initial snapshot, incoming boundary, selection history or complete applicable L1 controls. FINAL does not certify that an observer's L1 control view is complete. The frozen Model expressly distinguishes that requirement from a signature or TEE (`model:285–291`).

Historical L1-reference/admission correctness that was actually checked by the correct signer can be certified by the lemma as a property of the exact DATA. A new external control action, or source data needed to derive the observer's own boundary, is a different object and still requires public source authentication. An implementation must mark this dependency split explicitly.

## 7. Does this preserve the original Model?

Yes, as a change of the **external algorithm** with the foregoing scope:

- No new honest party, quorum threshold, certificate round, signed field, archival access promise or timing bound is assumed.
- The existing regional fault model already guarantees a correct signer. The existing full-validation rule already constrains what that signer can sign. The proof derives the kernel's soundness from those facts.
- Sound native input validation is used exactly where the old correct Vote already used it; it is not strengthened to certify unbound execution claims.
- The original complete-result publication contract may remain unchanged. A full old complete result contains the kernel as a subset. Using that subset for public acceptance does not authorize the Publish Layer to stop providing payloads/ancestry or change the meaning of “already recorded.”

But the external **specification** must change. Baseline main `823–825` explicitly requires an outside verifier to check condition (iii) from evidence; baseline main `881–896` and v3.6 main `920–935` specify historical admission witnesses. v3.6 `exports:92–94` also currently requires all admission/parent/native checks. These are algorithmic requirements, not merely informal commentary.

Do not call the new rule “the same validator with a faster parser.” State that public input acceptance uses a derived FINAL certificate rule for historical regional validity, while regional voting, recovery, Ready, complete-result formation and audits retain full witness validation. If public receipt validation is named separately from regional/full receipt validation, their roles remain clear and the old definition need not be weakened globally.

There is a loss of one operational capability if full evidence is never independently audited: a public observer no longer detects a publisher's malformed auxiliary regional witness as a reason to reject otherwise valid DATA. That is the intended behavior here, and does not weaken in-model DATA validity. It is not a promise of independent cryptographic validity outside the regional fault/validation assumptions. The old protocol was already conditional on those assumptions for its consensus guarantees, but the distinction should be candid.

## 8. Recommended exact changes and proof dependencies

1. Add `RawFinal` as a syntactic threshold predicate, explicitly independent of `def:receipt(iii)`.
2. Add the honest-FINAL lemma after the operational FINAL-generation rules and before its first public use. Its proof must cite Vote, timer Commit, no-FINAL-on-receipt, fixed snapshot lifetime faults and hash/signature assumptions.
3. Define a separate `VerifyPublicResult` or public receipt kernel. Preserve the full regional receipt definition and correct regional validators. Update the outside-verifier sentence to name the new public rule rather than leave two contradictory requirements.
4. Specify the minimal kernel dependency graph. A publisher's proposal/admission/QC sidecar is optional for this public check and cannot introduce a generation-authority wait. Keep native outputs not certified by the lemma in their separate verification path.
5. In the public-response proof, extract this kernel from the actual original complete result. Do not import a later private genuine witness into an old record. The certificate itself supplies the existence argument; it does not change what was in the old public input.
6. Preserve the global safety dependency on regional commitment compatibility. The honest-FINAL lemma supplies validity/provenance, while the regional safety theorem supplies uniqueness/compatibility; neither proves the other.
7. Recompute the bounded-handler argument over this smaller required public dependency graph. It removes the arbitrary alternate `AUTH_(r,g)` branch, but a finite signature/path scan still needs the declared input/load and handler-processing contract. No new timing theorem follows automatically from this design note.

## Validation status

This note checks the raw-signature-to-timer-to-full-validation implication directly against the frozen v3.1 source and current v3.6 operational rules. It is a manual semantic analysis, not an executable/model-checking result. No counterexample was found to the narrow public input-validity rule. Full v3.7 prefix safety, carrier compatibility and bounded liveness still require their own integrated proofs. Only this note was written for this extension.
