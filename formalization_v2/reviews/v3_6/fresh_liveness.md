# Summary

**Final Verdict:** The liveness argument is viable but incomplete due to one **Justification Gap** in deriving the complete public-source handling deadline from the frozen transport assumptions. The regional-response, finite-health-interference, adaptive-window arithmetic and owed-service arguments otherwise check out at the stated abstract level. I did not establish that the eventual global-growth theorem is false. The request-liveness objective remains unresolved, and the supplied request-starvation example is consistent with the regional timing and retirement rules.

**List of Findings:**

- **Location:** `sections/rotation_v3_6_source_step.tex:143–160`, especially “Its complete source evidence is therefore available by $a+T_E$” and “The immediate regional sends occur by that common source deadline”; `sections/rotation_v3_6_liveness.tex:167–173,237–242`.
  **Issue — Justification Gap:** These passages move from bounded availability of Espresso records and their Espresso evidence to bounded formation and processing of a dependency-complete `SOURCESTEP`. The frozen assumptions do not explicitly provide that bound for fetching old public dependency objects on a cache miss. The new packet requires their full bytes (`source_step:75–79`); common completeness deliberately permits references into older raw records and says that a cache miss retrieves and resumes (`source_step:14–35`). Retaining a source cursor, checkpoint and dependency index (`source_step:4–5`) does not explicitly require retaining all of those old raw bytes. The regional communication bound applies after a complete correct-to-correct message exists. It cannot itself bound first assembling that message. The exact bound $L=2T_E+2\Delta_i+R_i+T_{\rm pub}$ therefore has an unproved prerequisite. The subsequent global-progress and repeated-service proofs depend on that prerequisite.

This finding is narrower than saying that result normalization fails. A self-contained, independently framed complete result with a malicious outer base, beta, interval or unrelated bad item is handled correctly by the new extraction rule. The gap concerns whether every source operation preceding its acceptance is already dependency-complete by the deadline used in the proof.

## Review identity and coverage

- Reviewed only the frozen `snapshot/`, the review scope and manifests. No other review report or prior verdict was read or used.
- Main SHA-256 verified: `ef6d858b8ead057dd77f1a43e4198d47ceceb016e5c7b826eb941878b59d96d5`.
- All **21** entries of the parent manifest matched their bytes, including the baseline manifest, both included figure PDFs, all figure sources, all seven active fragments, main source and translations. The four entries in the baseline manifest also matched independently.
- Read the full v3.6 main, all seven active fragments, all five figure sources, the complete Part I baseline and the frozen model file. Read the complete original v3.1 by checking exact equal-block correspondence against the already-read main and reading every nonmatching original line, including the original exporter, handoff, force, observer and progress sections.
- Programmatically extracted the complete Model section from v3.6 and v3.1: the strings are identical, and both equal `baseline/model_and_assumptions.tex` exactly. Literal equality does not resolve the semantic timing issue above.
- This is a manual source/proof review. No executable protocol test, model checking, native-adapter timing test or concrete Espresso certification was supplied or performed.

# Detailed Verification Log

All main-source references below mean `snapshot/sync_hotstuff_rr_part2_rotation_v3_6.tex`; fragment references mean files in `snapshot/sections/`.

## 1. Frozen premises and dependency map

> “The delay bound includes delivery and processing of complete validation dependencies.” — main:395–397; baseline model:177–179.

The regional network bound includes complete message validation. It is a regional correct-to-correct bound, not an independently supplied deadline for arbitrary archive retrieval, native restoration or cross-region transport.

> “A result already recorded in Espresso satisfies this delivery guarantee; retries needed for acceptance into $G$ are a separate task.” — main:407–408; baseline model:189–190.

The publication premise cannot be invoked a second time simply because a recorded complete result had a bad outer wrapper. The candidate correctly attempts to accept the originally recorded closure instead. This review does not assume an acceptance retry, exact-base submission or correction deadline.

> “Required authorities and recoverable native state become available.” — main:525–527.

This is eventual availability. The finite preparation argument may take a finite maximum for a fixed stalled slot. It may not assign an a priori constant to every future authority or state restoration. The proof explicitly avoids doing that.

> “Retried EXPORTs for the next required DATA are accepted in time during some sustained service interval.” — main:522–524.

This inherited condition is present literally, but the new proof expressly declines to use successful acceptance to prove acceptance (`liveness:5–12`). I likewise did not use that condition to fill the source-response gap.

The proof dependency chain is:

1. Stable zero-output retries + persistent preparation + Part I fairness/exclusion imply a finite prepared regional response $R_i$.
2. Bounded complete opening-source handling + $R_i$ + result delivery + bounded complete result-source handling imply a public response $L$.
3. Health credits and leases leave finitely many committee changes at a fixed accepted frontier.
4. Conditional cooldown and owed selection give immediate AUTO reassignment in the prepared tail.
5. Public response gives $a_{n+1}-a_n\le L$ under no growth; adaptive misses give a quadratic lower bound on Espresso timestamps; clock upper lead gives a contradiction.
6. Global growth plus finite owed rounds and cyclic regular selection give repeated accepted regional DATA service.
7. None of these steps implies request inclusion; the manuscript supplies a separate starvation execution.

Step 2 has the finding above. Steps 5–6 are therefore conditional downstream conclusions, not independently certified theorems.

## 2. Stable slot and finite preparation

> “After it, the zero-output rule keeps a fixed slot $r^*$.” — `rotation_v3_6_liveness.tex:41–49`.

Correct. Once $G$ is fixed, a current duty can have positive accepted count only because it contributed before that fixed tail. At most one later Force closes that duty. Subsequent assigned duties have zero accepted DATA and reuse the numeric slot. An unassigned boundary preserves its stored target instead of repeatedly incrementing it (main:1767–1799; `control:240–249`). Regular completion cannot occur infinitely often while $G$ remains fixed, because public terminal completion itself commits DATA.

> “The background authority task continues through successive indices of $r^*$, rather than restarting at zero on Force.” — `liveness:47–49`.

Supported by the protocol rule at main:1733–1742. Cached authorities and the fetch cursor survive beta changes; new engines nevertheless start at $g=t=0$ with their own admission proofs. No old-beta Quit is imported. This avoids silently treating a cached high-generation authority as permission to enter it.

> “Taking a maximum of these finite completion times does not impose a uniform bound on authority delivery or state restoration.” — `liveness:89–96`.

Correct for the finite, fixed snapshot and anchor under the stated persistent-preparation interface. It does not establish a bound for future slots. The direct executable-state attachment contract is explicit in `source_step:168–176` and `ready_bundle:99–146`; arbitrary database imports remain outside the supported claim. The native adapter is an unverified implementation obligation, not a certified performance result.

## 3. Finite regional response $R_i$

> “Let $E_i$ be the set of its identities whose admissible equivocation is ever exposed in this fixed authority namespace.” — `liveness:70–87`.

The finite maximum over exclusion indices is legitimate. Part I:185–206 and 852–866 use finite identities and a permanent generation-index exclusion tail. Beta does not reseed immutable slot authorities (main:1793–1799). Hence, after $\bar g_i$, an exposed identity cannot occupy a leader position that would produce another admissible equivocation in that namespace. Old/lower-generation evidence cannot freeze the new current generation.

Initializer fairness supplies finite $g_i^*\ge\bar g_i$ (Part I:208–223). No numerical exclusion bound is needed: $g_i^*$ is a witness depending on the execution and fixed slot.

> “$D_i=(Q+1)(2K+12)\Delta_i$, $R_i=(g_i^*+1)D_i+2g_i^*\Delta_i+9\Delta_i$.” — `liveness:98–143`.

The bound is conservative and consistent with the core rules, assuming the stated complete Ready handler and no external closure during the relevant interval:

- Initial and cached-authority recovery entry skew is at most $\Delta_i$; ordinary term entry skew is at most $2\Delta_i$. The latter follows from Part I:630–657, including its correct-signature and full-forwarding argument.
- The last progress check is at most $u+(2K+6)\Delta_i$ from the first correct entry to a term. If the opening is unknown, the first check still runs (main:1191–1208), so silence cannot suppress BLAME.
- If a correct terminal vote exists, full forwarding and correct voting produce its QC within two further regional delays unless a genuine Quit intervenes. At an endpoint, surviving timers plus FINAL collection fit within the four-delay allowance. If no terminal vote exists, all correct last checks yield matching BLAME, and propagation reaches a freeze within the stated allowance.
- At most $Q$ DATA terms fit before the fixed endpoint, and a drain generation has one term. There is no infinite sequence of ordinary terms inside a fixed finite session quota.
- After a first correct Quit, proof propagation and the original recovery wait cost at most $2\Delta_i$ when the next authority is cached. Buffered proofs may cause immediate subsequent freezes, which are still covered by this finite prefix.
- At $g_i^*$, the correct initializer's first fresh proposal is protected against premature BLAME exactly as in Part I:870–894. The timeline gives proposal by $e+5\Delta_i$, commitments by $e+8\Delta_i$, and FINAL collection by $e+9\Delta_i$.
- If that fresh block is DRAIN, the resulting FINAL is for the terminal DATA of this same session. Since $Q\ge1$, it certifies positive DATA beyond the retry anchor. A different session's receipt is never substituted.

I found no independent error in this regional-response lemma. It is conditional on the baseline regional soundness and complete-handler interfaces, not a re-proof of every regional safety lemma.

## 4. ReadyBundle and opening-source timing

> “Each correct selected member constructs its complete bundle before its first Ready/Enter event.” — `rotation_v3_5_ready_bundle.tex:61–70`.

Once such a bundle exists and is sent, the $e+\Delta_i$ entry-skew conclusion is properly conditional on validation, executable object adoption and Enter belonging to the modeled complete-message operation (`ready_bundle:107–144`). The proof does not count unbounded reconstruction after first Enter as if it were covered by the regional network bound. The current opening header suffices; no new empty/header-only Espresso block is needed before first DATA.

> “Every predecessor … became final no later than $a$. Its complete source evidence is therefore available by $a+T_E$.” — `rotation_v3_6_source_step.tex:143–146`.

This establishes availability deadlines for each predecessor's source record/namespace and Espresso evidence, under the continuing-online premise. It does not by itself establish the subsequent claim that the full `SOURCESTEP` specified at lines 56–79 can be sent by that same deadline.

The packet's dependency field requires **full bytes** for required public references not resolved by its checkpoint. In contrast, the completeness rule allows exact references into earlier rejected raw records, and expressly distinguishes cache-hit validation from cache-miss retrieval (`source_step:14–35`). The specified persistent objects are cursors, checkpoints and an index; the source does not expressly require that every old raw object remain locally resident in a form that can immediately populate future packets. Long-term Espresso history is retrievable, but the frozen history paragraph gives no uniform bound for a new old-history read (main:436–452). The $T_E$ deadline at main:454–472 covers the submitted record, namespace and Espresso evidence; regional/application validation remains separate.

### In-model timing trace exposing the missing premise

1. An earlier authenticated raw record contains a framed object $X$. Correct participants receive and authenticate its namespace within the original $T_E$ deadline. $X$ is not accepted DATA and need not be part of the current anchor's executable state or validated dependency cache.
2. Later, participants retain the required source cursor, replay checkpoint and public object index, but $X$'s bytes are available only from the archival source. This does not discard committed regional DATA or the fixed anchor, and the history remains retrievable.
3. A new record contains an exact public reference to $X$. Its public completeness is determined by the same earlier source position for all observers. The rule at `source_step:33–35` requires retrieval and resumption on a cache miss; it does not classify this reference as an absent/future dependency that can simply be skipped.
4. The new record's bytes and Espresso evidence arrive within $T_E$, but the fresh historical read finishes after an arbitrary finite delay $D$. The original availability deadline for $X$ was met when its containing block finalized; the frozen archival retrieval clause does not give a fresh $T_E$ deadline for this later access.
5. Until $X$ is retrieved, a sender cannot supply the required full dependency bytes in this `SOURCESTEP`. The regional $\Delta_i$ guarantee for processing a complete packet has not yet started. An already-online participant can therefore face this wait without being an arbitrarily long offline catch-up case.

This is a trace of the timing implication that is missing, not a demonstrated fork or a demonstrated infinite-stall execution. It can arise before a subsequent result record reaches its canonical scan; an invalid extra proof choice must not be assumed to be instantly rejected before all of its publicly identified dependencies are read unless that processing rule is established. A self-contained complete result's own successful regional relay can supply its own dependencies; that fact alone does not prove that every preceding source operation or unrelated proof choice is dependency-complete by the claimed source deadline.

Calling the eventual complete receive event atomic is sufficient to eliminate an extra scheduling tick after its dependencies are present. It does not prove that first dependency acquisition finishes by that event's proposed start time. The proof's explicit warning that an unbounded operation cannot be renamed “normalization” (`liveness:237–242`) recognizes the distinction but does not establish the missing acquisition bound.

The same scope distinction applies to nonregional observers: the frozen visibility premise covers their Espresso evidence, while the described `SOURCESTEP` multicast and $\Delta_i$ bound are regional. A uniform response deadline for every such observer needs its actual complete processing path covered; deterministic replay alone establishes agreement, not a wall-clock deadline.

## 5. Complete-result extraction and already-recorded bad wrappers

> “A false hint neither authorizes invalid DATA nor vetoes a complete valid result carried in the same record.” — `rotation_v3_6_exports.tex:7–12`.

The rule closes the outer-hint failure mode for the declared framing. The decoder inventories typed DATA and evidence recursively, including ancestry objects (`exports:14–39`). Candidate completeness is closure-local, and a malformed known frame does not discard independently framed neighbors. The signed chain/domain remains authoritative even when the outer routing chain or beta hint is false.

For a record with wrong outer expected base, beta or interval, but a valid complete closure for the current next DATA $B_1$, the scan still considers $B_1$ at $h(G)+1$. It checks the actual $G$ and signed path, incoming boundary, producing session, current beta, quota, historical authorities, native validity and strictly preceding Espresso references (`exports:63–102`). An unrelated bad object neither removes this candidate nor creates a duplicate marker. Duplicate signers/encodings cannot yield additional credits.

> “If it was already finalized by $s$ … available by $s+T_E$, or earlier.” — `liveness:195–203`.

For the **record itself**, this inequality is correct: if its first finalization is $f\le s$, its source availability is at most $f+T_E\le s+T_E$. The already-recorded branch does not need $T_{\rm pub}$ or a replacement wrapper. For a new record, the bound is $s+T_{\rm pub}+T_E$. The unproved part is the complete interpreter/source handling prerequisite discussed above, not these delivery inequalities.

The supported transport interpretation is consistent with a complete result containing its required DATA, ancestry and evidence. Opaque bytes, an unrelated undiscoverable namespace or omitted required ancestry do not yield the claimed decoding result. The document explicitly withholds progress under those broader readings (`exports:198–224`). This review does not treat that disclaimer as proof of binary compatibility with an unspecified implementation encoding.

## 6. First-result causal order and no skipped first DATA

> “The carrying record cannot precede this retry's authenticated opening.” — `liveness:205–214`.

Correct under the cryptographic/source premises. A valid FINAL receipt has a correct signer. That signer committed only after validating the signed block and its finalized Espresso reference. The first DATA reference is at least the opening height, and subsequent DATA references are monotone (main:635–650). Therefore the complete signed result exists only after a finalized reference at or after the opening. An immutable containing block carrying that newly formed result must be strictly later than its reference. A publisher's false outer metadata does not change this causal dependency. This also rules out same-block acceptance of the new Force version.

> “Its receipt root directly or indirectly covers $B_1$ in this same session.” — `liveness:187–193`.

Correct. The regional response supplies positive DATA after the fixed anchor in the retry's own session. Its same-session coverage reaches the first DATA; later-session certificates cannot substitute. Under no prior growth and no intervening closure, that first DATA has the authorized next height and ancestry. If another valid candidate was committed first, growth already holds.

## 7. Terminal DATA, $Q=1$, and cursor waits

> “Durably expose the updated $G$ and accounting before attempting a later DATA or successor operation.” — `exports:104–120`.

The per-DATA commit order is explicit and sufficient. For $Q=1$, $B_1$ is terminal. `CommitNextDATA` appends it, increments counts, discharges the producing region's debt, cancels the current window and stores the pending REGULAR boundary. None of those operations requires `AUTH[r+1,0]` (`exports:122–159`).

Only `ResolveBoundary` subsequently obtains that target authority and runs `PrepareOwed`. The public cursor and global-finality definition expose the earlier commit even if the outer record, block or successor resolution remains incomplete (`exports:161–196`; main:1408–1435,2134–2184). Thus an unbounded successor-authority wait does **not** defeat the first-growth alternative.

For a continuing prepared duty, a malformed or genuinely absent candidate-local proof is skipped rather than made into a canonical boundary wait. The potentially blocking public operation is a genuine pending boundary/prelude operation; its fixed-slot initial authority belongs to the finite prepared prefix in the stalled-tail argument. The distinct old-public-dependency retrieval issue in Finding 1 remains; it is not the repaired terminal-successor wait.

## 8. Accounting for $L$

> “$L=\max_i\{2T_E+2\Delta_i+R_i+T_{\rm pub}\}$.” — `liveness:151–161`.

Conditional on both complete source handlers starting by their claimed source deadlines, the constants are sufficient:

| Stage | Deadline/allowance |
| --- | --- |
| Opening source availability | $a+T_E$ |
| Complete regional source relay and first Enter | $a+T_E+\Delta_i$ |
| Regional complete positive result offer | $a+T_E+\Delta_i+R_i$ |
| New result record availability | add $T_{\rm pub}+T_E$ |
| Complete result-source regional relay and commit | add $\Delta_i$ |

The ReadyBundle's entry skew is already part of the regional argument defining $R_i$; it must not be charged again and then silently omit the final source relay. The displayed $L$ can cover the two source-processing regional allowances. Already-recorded publication is faster. $Q=1$ contributes no successor-authority term. There is no numerical bound for initial preparation, and none is required by this finite-tail argument.

However, an unbounded pre-send source-dependency delay $D$ is absent from this table. Without establishing that it is already completed, the claimed fixed $L$ is not proved from the frozen assumptions.

## 9. Health budgets and restoration

> “For each $\kappa$, three initially unused bits … at most one MSIG, one health lease, and one optional CLEAR.” — `rotation_v3_5_health.tex:18–45`.

The monotone accepted-tip cursor and nonrefundable reservations imply the finite-control bound at a fixed accepted height. Uncertain submissions, stale targets, restarts and beta changes do not refill credits. Finite preexisting reservations are counted through $N_0$. Merely observing historical accepted tips late cannot create more than the finite ancestor-chain number of keys.

> “First submit an independent STATUS ON … wait until that exact record is finalized and available.” — `health:60–76`.

The OFF lease always has a finite later ON already finalized. Ordering by activation time handles a timestamp jump that crosses both: OFF precedes ON among independent STATUS records. Empty attached configuration on new MSIG controls is essential and is explicitly required; the proof does not reinterpret a legacy attached OFF after independent ON processing. Legacy controls are scoped separately (`health:93–118`).

The per-region credit $u_i[d_i]$ also prevents accepted DATA from other regions from funding infinitely many exclusions of an unserved region. Under the policy from genesis, eventual restoration follows without health accuracy or health-report delivery assumptions. Unlimited bypass signing would violate the specified policy, not refute its lemma.

## 10. Adaptive windows and clock contradiction

> “There is no AUTO-created unassigned interval that could replenish arbitrary real-time clock lag.” — `liveness:267–278`.

Given the fixed prepared slot, stable governance and a nonempty eligible owed candidate, the conditional cooldown rule preserves at least one candidate. If the source is the only eligible owed member, its cooldown is unchanged; if it is outside the owed set, an owed candidate already exists. Existing cooldowns expire at finite Espresso positions. This reasoning does not CLEAR an old cooldown or override OFF.

> “$\theta_{n+1}-\theta_n\ge M_nW_n$” and “$a_{n+1}-a_n\le L$.” — `liveness:280–309`.

The first inequality follows from $M_n$ consecutive failed startup windows at zero accepted output, including timestamp jumps. A jump creates one Force, and the successor starts at the realized timestamp. $M_n=1+\beta_0+n$ and the finite fixed parameter set gives $W_{\min}>0$. The empty-window arithmetic shortcut in `control:74–90` preserves the same transition.

The second inequality depends precisely on the public-response lemma and immediate reassignment. Assuming that lemma temporarily, summing gives a quadratic lower bound on $\theta_N$ and a linear upper bound on $a_N+\epsilon_E$. The frozen upper-lead bound suffices; no lower-lag bound, empty-block rate or recurring successful-interval assumption is secretly used in this algebra. Any finite initial lag is absorbed into a constant. This branch is correct **conditionally**, but remains unverified downstream of Finding 1.

## 11. Owed-set repeated accepted service

> “Only Force selection is filtered by $\mathcal O$. REGULAR selection retains its historical NextLive rule.” — `liveness:332–343`.

The proof uses the actual mixed selection rules. A round is finite; accepted DATA or effective OFF/authority pruning removes a debt once; new ON does not insert into an unfinished round. An empty set is refilled only at a public boundary/selection, not per DATA of a continuing finite duty (`control:101–164`).

If infinitely many Forces occur while persistently eligible $C$ remains owed, the first accepted DATA of every productive new version must come from its originally selected owed session. Contiguity cannot skip it, and strict cutoff excludes prior-beta output. Each such productive version consumes a finite debt removal or a pruning event. Infinitely many productive versions cannot avoid $C$; finitely many productive versions followed by infinitely many Forces would contradict global growth.

If only finitely many Forces occur, unbounded accepted DATA requires infinitely many finite-quota regular sessions. The new floor $q_0^{\rm regular}\ge q^{\rm open}+1$ eventually moves historical selections beyond $C$'s permanent eligibility and earlier cooldown. The fixed cyclic scan cannot perpetually skip eligible $C$, and contiguous acceptance must include its selected session before passing it. These arguments also discharge a previous round before a newly permanently ON region joins a later round.

I found no separate debt-ledger liveness error. This proof remains conditional on global progress, which inherits Finding 1. Its conclusion is repeated **accepted regional DATA**, including empty DATA; it is not transaction inclusion or a positive throughput rate.

## 12. Request-starvation example and objective scope

> “The two necessary correct DATA votes at $d+4$ and $d+5$, and their commit timers at $d+6$ and $d+7$.” — `liveness:400–430`.

The example can be instantiated with three unit-weight voters, $F=1$, $\Delta=1$, quota at least two and a correct recovery initializer. With first recovery entry at $d$, a $2\Delta$ leader wait, NEWVIEW/ANCHOR exchanges, and legal delays can give the initializer's DATA self-vote at $d+4$ and the other correct vote at $d+5$. Their timers expire at $d+6,d+7$. A Force crossing at $d+5.5$ occurs before any correct FINAL for that DATA. Both votes already exist before closure, so retirement can preserve and later drain their timers and publish the resulting complete late receipt; it does not make the result globally eligible again.

An earlier valid empty DATA in that duty is accepted before the crossing. That positive accepted count legitimately advances the slot after Force, producing a fresh eventual-but-unbounded recovery-authority wait in the next slot. Every finite tolerance can be met by placing the correct recovery entry sufficiently late. No equivocation is required, high-generation schedules can satisfy fairness, correct initializers actually enter repeatedly, and late complete results can satisfy both publication bounds. The strict cutoff rejects their request-containing suffixes. Pending-queue rules retain $x$ for reproposal but do not ensure one of those proposals reaches $G$.

Therefore this execution is a valid limitation of the requested end-to-end request goal, not a counterexample to the narrower empty-DATA progress or regional-credit statements. The manuscript explicitly acknowledges the limit. That acknowledgement is accurate, but it does not complete the broader objective.

## 13. Final propagation and validation boundary

- **Checked independently:** stable retry-slot preservation; finite authority-prefix witness; regional bound $R_i$ after preparation; bad-wrapper extraction for the supported complete syntax; causal first-result ordering; $Q=1$ durable commit before successor authority; finite health budgets and lease restoration; adaptive-window arithmetic; owed-ledger counting; request-starvation timing.
- **Not established from the frozen premises:** the uniform deadline for obtaining a dependency-complete public source packet/operation in every cache-miss path preceding next-DATA acceptance. The specified handler removes post-read scheduling waits but does not itself prove the missing pre-send acquisition bound.
- **Consequences:** the numerical public response $L$, continuing global DATA progress via its clock contradiction, and repeated accepted service via global progress remain conditionally justified rather than fully verified.
- **Still outside the delivered result:** full request liveness, arbitrary native-adapter performance, and executable/model-checking certification.

No protocol source was modified, no assumption was strengthened, and no repair was supplied in this review.
