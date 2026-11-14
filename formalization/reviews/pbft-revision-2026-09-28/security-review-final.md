# Final independent security verification: PBFT revision

## Summary

**Final Verdict:** The revised arguments are valid under the manuscript's explicit fault, persistence, synchrony, admission, capacity, and tenure assumptions; no residual **Critical Error** or **Justification Gap** was identified in this review.

**List of Findings:** None remaining. Both findings from `security-review-initial.md` are resolved in the frozen version below. This is a mathematical protocol review, not implementation verification or a claim of service under arbitrary partitions or unlimited physical network flooding.

### Frozen source

| Artifact | SHA-256 |
|---|---|
| `formalization/main_PBFT.tex` | `42ae5d3fc3b94b84f5a12d71a910909ec045e07363129ce8b4aa43e399f41eeb` |
| `formalization/figures/service-architecture-pbft.tex` | `dcd8e440a75caf1dcbefef99f1ac197c50fbfebdc2d68591d742f69701c5aa13` |
| `formalization/figures/stream-pipeline-pbft.tex` | `cd9e32402772307b8e05391892b7e1cf9792f1a5bbe63db896673579db26a244` |

The manuscript snapshot is `main_PBFT-final-security-snapshot.tex` beside this report. Line references below refer to that exact source. This reviewer edited neither the manuscript nor `main_v1.tex`. A final diff against the substantively reviewed hash `f15fc125239c815a14fa06aa1d8733dbfa7a30c684220ca5d3f7c738b2902142` contains only two `Needspace` layout changes at lines 574 and 775; semantic text and line counts are unchanged. The verdict therefore applies unchanged to the final hash above.

## Detailed Verification Log

### 1. F1 resolved: canonical identities exclude certificate encodings

**Text checked:** “An activation has a canonical semantic record $A$ and a detached finality witness $\pi_A$” (279–291), followed by `$R_0=((c,\gv,0),\Hash(A),\Tag{INIT},h,q)$` (293).

The activation record now contains the configuration, rotation/checkpoint identifiers, finalizing child identifier, successor view and region, retained tip, and fixed activation time. It contains no quorum subset or signature bytes. Finality witnesses are detached. Thus two honest parties with different valid quorum subsets compute the same activation record and initial regional parent.

**Text checked:** “Record hashes include semantic fields only” and the regional report chain's semantic value is the ordered record list (395–399); NEWVIEW hashes sender-sorted semantic summaries with signatures detached (361–364).

Global report vectors have a fixed origin order. Report signatures, regional certificates, and activation witnesses do not affect report or checkpoint identity. The pending READY package now stores the deterministic origin-identity list, with signed reports detached; its reason also has deterministic precedence. The transition copies unspecified state fields before explicit updates. These definitions close the earlier proof-valued-state ambiguity and make the global equality induction concrete.

The previous dependency limitation on installing a common `R_0`, global agreement/authority, consumer safety, and service liveness is therefore discharged.

### 2. F2 resolved: admitted verification work and hostile traffic are separated

**Text checked:** Regional admission precedes authentication, bounds packet encodings per authenticated sender, type, and active domain, and excludes remote future domains (462–478). Global FFG/export/announcement admission has separate per-sender/type/epoch allowances (672–677). Broadcast admission has a per-sender/invocation allowance (1452–1460).

Previously, arbitrary authentic conflicting packets could cause arbitrarily many signature checks before duplicate suppression. Now each bounded authenticated channel has a finite allowance before expensive application-signature verification; exact duplicates do not consume additional allowances. A Byzantine sender cannot spend an honest sender's allowance. Quotas are separated by message type/domain, so conflict traffic cannot consume an honest sender's prescribed PREPARE, COMMIT, VIEWCHANGE, or finality packet slot.

The specified honest regional behavior fits these allowances: one proposal/NEWVIEW, one phase vote, one VIEWCHANGE snapshot, and one relay per relevant domain. The predecessor or certificate relay needed for progress remains admissible from honest senders. Global votes and certificate exports likewise fit their separate allowances. The statement of fair per-peer service and bounded complete-object delivery remains an assumption; the protocol does not derive protection against physical bandwidth exhaustion from these quotas.

**Text checked:** “These are admitted protocol-work bounds” and “Add the original-author and inherited-proof checking work of each admitted witness” (1326–1333).

The signature count now explicitly counts new PBFT phase signatures, while full batch and inherited-proof work is added separately. Regional metadata `lambda_R` includes original producer and inherited-evidence signatures; global metadata `lambda_G` includes regional/finality evidence and excludes only the new signed-chain signatures (1250–1256). This prevents hidden omission of full witness verification from a committee-size-only operation bound.

Honest-generated regional messages remain quadratic per normal batch. Relaying unaggregated commit certificates still has cubic signature-bit cost. The admitted-work accounting does not claim that Byzantine nodes obey the message count or that raw flood processing is bounded by it.

### 3. The new broadcast quota preserves the existing theorem

**Text checked:** “Each honest sender forwards at most two values, so its prescribed packets fit” (1455), together with A3's at-most-once forwarding of each retained value (1461–1467).

The quota is per authenticated **protocol sender** and invocation, not per alleged origin value or a shared queue that a faulty peer can exhaust for an honest peer. An honest origin sends one input packet. An honest non-origin forwards at most two distinct retained values, once each. Thus every prescribed honest packet is admitted at every responsive honest recipient under the stated channel and fair-service assumptions.

If an honest member retains `z` through a Byzantine sender whose other traffic was admitted differently elsewhere, the proof still works: either the accepted chain already contains a responsive honest signer, whose earlier multicast was admitted everywhere, or the retaining honest member appends its signature and multicasts `z`. That honest multicast fits the sender's cap-two allowance everywhere. If it already forwarded `z`, its previous multicast suffices. A terminal length-`n` chain necessarily contains a responsive honest signer. Consequently the original singleton-versus-multiple agreement argument remains valid despite receiver-dependent admission of Byzantine packets.

Network relays must preserve the authenticated protocol sender used by this abstract communication interface; a physical relay's shared ingress queue is not a substitute for the specified per-protocol-sender allowances. Fair delivery of the prescribed honest messages remains part of the model, not an unstated theorem.

### 4. Regional safety still covers hidden commits, lower-view retirement, and restarts

**Text checked:** R3 persists the prepared certificate before COMMIT and sends only while the retry is active; R4 explicitly collects matching COMMIT signatures and assembles the quorum certificate (496–512). V1 first persists retirement of lower retries before signing its immutable VIEWCHANGE (521–528).

The explicit assembly step completes the operational path from COMMIT votes to a transferable certificate without changing its safety predicate. Retired retries cannot receive new honest PREPARE/COMMIT signatures, but an already valid old certificate remains acceptable. Accepting such a certificate does not reopen lower-view voting.

For a possible COMMIT quorum in retry `v`, honest COMMIT signers have weight above one-third and persisted a prepared certificate first. Every later NEWVIEW quorum intersects them. An intersecting honest member could not have signed the later VIEWCHANGE before its old COMMIT, since retirement would then forbid that COMMIT. Its report therefore carries a certificate at least as recent as `v`. The earliest later conflicting-prepared-certificate argument forces the selected value to remain the committed value. This depends on individual signatures, so it covers a quorum assembled or revealed only after the view change.

Within-retry PREPARE uniqueness and this carryover invariant imply commit uniqueness. Induction over sequence number gives one regional chain, now rooted at the explicitly canonical activation record. The original author remains bound by `(configuration, global view, sequence, author retry, height)`; carrying a batch changes its PBFT wrapper, not its ORDER/BATCH signatures. Restart restores signing history, and a node with unknown safe timer origin abstains until a later sequence or activation.

### 5. Regional timing and consumer repair remain correctly scoped

The fixed responsive regional quorum enters a sequence within one regional complete-object delay after the first responsive holder's relay. Equal-rate timers preserve this skew. An honest retry primary waits long enough to receive the quorum's VIEWCHANGE reports; NEWVIEW, PREPARE, COMMIT, and relay finish within seven regional bounds from the earliest retry start. `T_V >= 8Delta_R` therefore suffices. The cyclic identity schedule eventually reaches a responsive honest primary; the proof does not confuse stake fraction with identity count.

The view-zero `3delta_R` proposal-to-commit bound uses the complete-message delay and the stated immediate-input/batch-construction qualification. It is not a throughput lower bound or a partial-synchrony guarantee.

The model now specifies `theta_R >= 2Delta_R`, `theta_G >= 2Delta_G`, and `theta_P >= 2Delta_P`, including complete round-trip service and verification. Consumers use their own `theta_P` regardless of supplier region. The repair table includes that case (1196), and local-recovery commentary now adds `A_P+F_P`, not the internal-region repair bound (1214). This closes the consumer timing mismatch encountered during recheck.

### 6. Global composition, retention, and end-to-end theorem dependencies

With canonical state identities, synchronized complete report agreement gives one report vector; deterministic transition and regional chain uniqueness give one target checkpoint. A responsive honest global supermajority signs the common adjacent FFG link after retaining and verifying data. This justifies the target and finalizes its parent. The protocol does not rely on FFG to make inconsistent reports converge.

ROTATE retains the parent's prefix for which READY was collected; CONFIRM preserves the pending package. Finalization fixes the activation boundary and the new global-view domain. Old regional decisions cannot extend the adopted new global history. Consumers apply all intervening finalized activations in order and retain delivered data, including during catch-up. The data-retention and sequential-repair arguments continue to apply to complete bounded objects.

Service liveness additionally requires a continuously healthy eligible successor, sufficient global responsive weight, full outcome fit in a report, bounded backlog, retained reachable data, and minimum tenure. The proof separately budgets old-tail arrivals, readiness/state transfer, global confirmation/activation, and successor local progress. None of these requirements is derived merely from PBFT or nominal maximum report sizes.

All dependent correctness branches from the initial report were reconsidered after the canonical-state repair. No remaining dependency on F1 or F2 invalidates the stated conclusions. The final editorial changes were also checked: sender order is lexicographic on configured identity pairs; genesis supplies its complete retained state at time zero with activation epoch one and is explicitly trusted; ancestor finalization is stated consistently with F3; the global agreement/progress envelope is named explicitly; and the generic control-cost parameter has a defined bit-size scope. None changes or weakens the preceding arguments.

### 7. Figures and retained limitations

The architecture figure still separates raw data, regional PBFT certification/Preview, backbone control, and the external proposer. The pipeline now labels retry as “no commit certificate by deadline” and global state as justified control state, avoiding confusion between prepared and committed or adopted and finalized state. No LMD-GHOST dependency is present in the reviewed manuscript or its two PBFT figures.

The following are explicit limitations rather than residual findings:

- Regional certificate safety uses fixed fault bounds and persistent honest signing restrictions but does not require regional timing.
- Regional progress needs a fixed responsive quorum through entry and retries; recovery with a dynamically changing quorum is not proved.
- Global agreement and progress use continuous synchronized complete-object rounds. Restored networking alone does not repair an unspecified stalled global epoch.
- Geographic recovery still requires a global responsive supermajority; a sufficiently large regional outage can block replacement.
- No bounded retrieval follows if every relevant holder is unreachable. No throughput guarantee follows without capacity/backlog premises.
- Execution correctness, transaction inclusion/fairness, financial penalties, and arbitrary-partition service are outside the verified claims.

## Assurance scope and evidence

This review checked the supplied algorithms and proof steps directly, rechecked every security-relevant repair, and followed the theorem dependencies. The initial full log remains in `security-review-initial.md`. It did not run an implementation, claim exhaustive model checking, or certify deployment readiness.

The original PBFT primary source was consulted for the prepared/committed distinction and highest-prepared view-change structure: [Castro and Liskov, OSDI 1999, Sections 4.2, 4.4 and 4.5](https://www.usenix.org/legacy/events/osdi99/full_papers/castro/castro_html/node4.html). The weighted, single-outstanding-sequence, fixed-timer variant was verified from this manuscript rather than accepted by citation.
