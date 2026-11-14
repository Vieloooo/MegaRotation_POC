# Independent self-containedness review — initial PBFT revision

## Summary

**Final verdict:** The regional proof and the synchronous global-chain argument have a coherent dependency structure, but the manuscript is not yet fully self-contained as written. Several deterministic-encoding, transition-completeness, timing-premise, and cost-accounting details need to be made explicit. These are **Justification Gaps** in the written specification; this review does not establish a counterexample to the intended protocol after those details are fixed.

Scope: all 1,410 lines of `formalization/main_PBFT.tex` and both included PBFT TikZ sources were read. This review did not edit the manuscript, the figures, or `main_v1.tex`, and did not use the old protocol as an unstated premise. The proof-verifier workflow was applied to theorem dependencies. Compilation and security review are separate root-agent tasks.

Frozen copies are in `self-contained-frozen/`. All line numbers below refer to those copies:

| Source | SHA-256 |
| --- | --- |
| `main_PBFT.tex` | `69da5fb098f108cca877181f5e4cf44cca4a42ce878203ff324bf16dc99d0e23` |
| `service-architecture-pbft.tex` | `dcd8e440a75caf1dcbefef99f1ac197c50fbfebdc2d68591d742f69701c5aa13` |
| `stream-pipeline-pbft.tex` | `7c2218dae0575d1e87cc2237611dd384ad4ded90b0015eb06f4d6fc35194bc18` |

### List of findings

1. **SC-1 / high / Justification Gap:** The activation proof is embedded in semantic hashes without an explicit projection to a canonical activation record.
2. **SC-2 / high / Justification Gap:** The global state transition omits default assignments and a deterministic definition of the rotation reason.
3. **SC-3 / medium / Justification Gap:** Pull and consumer-progress bounds need a complete request-response timing premise.
4. **SC-4 / medium / Justification Gap:** Global control costs omit the regional certificate signatures excluded by the definition of `L_G`.
5. **SC-5 / medium / Operational gap:** The regional protocol needs an explicit COMMIT-quorum-to-certificate assembly event.
6. **SC-6 / medium / First-use gap:** Acronyms, global justification/finality, and the activation-proof symbol need earlier definitions.
7. **SC-7 / low / Notation gap:** Latency variables and control-size variables collide, and the appendix reuses the global-checkpoint symbol for signature chains.
8. **SC-8 / low / Figure/text mismatch:** The architecture caption describes three drawn sequencers although the new figure has role/group boxes; the retry arrow and global-anchor annotation could be more precise.
9. **SC-9 / low / Standalone exposition:** A few descriptions still refer to an unstated previous protocol; no live LMD-GHOST rule was found.

## Detailed verification log

### SC-1. Give activation and ready evidence explicit semantic encodings

**Locations and exact text:**

- Main lines 149–150: “Semantic identifiers exclude witness encodings and signer subsets.”
- Main lines 270–275: “an activation proof $A$ fixes a tip” and `$R_0=((c,\gv,0),\Hash(A),\Tag{INIT},h,q)$`.
- Main lines 348–350: `$\mathcal S=(\gv,i,A,h,q,R,z,e_{\gv},p)$`, where “$A$ is the global view's activation proof”.
- Main lines 690 and 733: `p=(\gv+1,j,h,q,R,e,\mathsf{readyProof},\mathsf{reason})` and `$A=\pi_r$`.

**Issue:** The general instruction about semantic identifiers is good, but the concrete equations still put proof objects directly in the hashed `R_0`, checkpoint state, and pending rotation. Two valid finality certificates or READY quorum witnesses can have different signer subsets. A reader cannot reconstruct the exact semantic encoding to which `Hash(A)` or the checkpoint hash applies. The global proof at lines 935–939 relies on deterministic equality of these outputs, not merely logical equivalence of certificates.

**Minimal recommendation:** Define a canonical activation record using the finalized ROTATE identity, its confirming-child identity, named successor, retained tip, and fixed activation boundary; keep the finality proof detached. Define the pending rotation's READY field as the fixed sender-indexed semantic reports selected from the common report vector, with signatures/proofs detached. Use these canonical records in all hashes and `S`, and explicitly state what the genesis activation identifier is. Do not introduce a signature-subset-dependent identity indirectly through the readiness field.

### SC-2. Specify all global-state fields and the rotation reason

**Locations and exact text:**

- Main lines 348–353 define nine components of `S`.
- Main lines 674–676: “output `CONFIRM` with exactly the same stream tip, region and pending package.”
- Main lines 688–691: “output `ROTATE`, retain the parent's tip and cursor, and store” the pending package ending in `readyProof, reason`.
- Main lines 697–699: “Otherwise adopt $(R^+,h^+,q^+)$, set the failure count to $z^+$”.

**Issue:** The transition claims to be deterministic, but the branches leave other state fields implicit. In particular T1 does not say what happens to `z`, `A`, `e_v`, or the cursor; T3 does not say whether `z` stays unchanged or becomes `z+`. The field `reason` has neither a value set nor precedence when both planned tenure and no-progress guards hold. These differences change a checkpoint hash even when the retained data is identical.

**Minimal recommendation:** Start the transition with an explicit default-copy rule for the effective parent state and override only listed fields. Specify T3's chosen `z` policy and define a deterministic reason enum and precedence, such as recovery whenever the no-progress guard is true, planned otherwise. Initialize all genesis-state components explicitly or state that `G_0` contains the fully specified valid `C_0`, `g_0`, activation record, and all initial fields.

### SC-3. State when a request deadline is a successful-response bound

**Locations and exact text:**

- Main lines 134–135: “complete request deadlines $\theta_R,\theta_G$.”
- Main lines 156–160 bound transmission and verification of one admitted complete object.
- Main lines 540–545 advance to the next supplier after `theta_R`/`theta_G`.
- Main lines 986–988 claim retrieval within `$(a+1)\theta$`.
- Main lines 995–996: “an honest holder returns the complete original object within that deadline.”
- Main lines 1007–1008 promise unbounded growth for “live consumers”; lines 1059–1062 introduce `Delta_P` only as announcement propagation and `F_P` as the corresponding consumer repair bound.

**Issue:** A configured timeout need not be long enough for a request and full reply. The proof currently imports that adequacy premise. The global and regional delay assumptions also concern sequencer nodes, while proposer/consumer delivery needs its own live-path condition.

**Minimal recommendation:** Require request deadlines to cover the complete request-response path, e.g. `theta_R >= 2 Delta_R`, `theta_G >= 2 Delta_G` under the stated per-object timing model, and define the analogous consumer bound. Define a live consumer as one with the required reachable honest holders, processing capacity, and bounded complete request-response path. Then reference these conditions in the repair lemma and liveness theorem. If `theta` is intended as a directly assumed end-to-end bound instead, state that explicitly before Protocol D.

### SC-4. Include new regional evidence signatures in global control costs

**Locations and exact text:**

- Main lines 370–371 admit up to `b_G` complete regional outcomes in a report.
- Main lines 1184–1186: `$L_G$` bounds one report's metadata/evidence “excluding its newly appended chain signatures and bulk bodies.”
- Main lines 1216–1217: `$C_{\rm backbone}(k,L)=O(Ek^2(L+k\kappa))$`.
- Main line 1231: global report agreement costs `$O(EN^2(L_G+N\kappa))$` control bits.

**Issue:** A report can carry `b_G` newly appended regional commit certificates, each with up to `n_i` signatures. If “newly appended chain signatures” excludes these signatures from `L_G`, the report-control row has no `b_G n_i kappa` term. The `N kappa` term accounts for the enclosing authenticated-broadcast signature chain, not all appended regional certificates. Later complete-object bounds involving `B_G` do not repair a separately stated control-bit total that excludes these witnesses.

**Minimal recommendation:** Define `L_G` to include every report validity-witness signature, including regional certificates and missing inherited evidence, excluding only the new AB forwarding-chain signatures and bulk bodies. Alternatively introduce a distinct report-witness-size term and carry it through the formulas/table. Explicitly distinguish this from the appendix chain's at-most-`N` signatures.

### SC-5. Make local certificate assembly operational

**Locations and exact text:**

- Main lines 447–451: on PREPARE quorum, multicast COMMIT.
- Main lines 452–453: “A complete valid $\RC_u$ for any retry ... commits $R_u$.”
- Main lines 904–905: “COMMIT votes by three.”

**Issue:** Protocol R has no explicit event saying that a member collecting matching COMMIT votes constructs `RC_u`. Receiving a previously assembled certificate is specified, but the first certificate's production is left to inference. The phase proofs and latency argument use that inference.

**Minimal recommendation:** Begin R4 with: on collecting a quorum of matching COMMIT statements and the complete valid batch, assemble `RC_u`; on assembling or receiving such a certificate, perform the existing commit/export actions. State that old-retry COMMIT statements may be accumulated for proof assembly without reopening old-retry voting.

### SC-6. Complete first-use definitions

**Locations and exact text:**

- Title/abstract lines 62–75 use L1, PBFT, and FFG.
- Main line 254 uses “EVM execution”.
- Main line 131 uses “finalized genesis checkpoint”; line 413 uses “justified/finalized checkpoints”; G's initial state at lines 625–626 requires the parent's justification proof.
- The actual inductive justification/finality rules first appear at lines 717–725.
- Main line 733 assigns `$A=\pi_r$` without an earlier formal definition of `pi_r`.

**Issue:** The regional variables, original-author versus current-primary distinction, quorum threshold, and sequence/retry distinction are generally well introduced. The remaining issues are primarily first-use exposition: acronym expansions occur late or not at all, and the global state uses justification/finality before explaining their inductive meaning. `pi_r` is understandable in context but undeclared.

**Minimal recommendation:** Expand layer one (L1), Practical Byzantine Fault Tolerance (PBFT), and Friendly Finality Gadget (FFG) in the abstract/introduction, and Ethereum Virtual Machine (EVM) at its first use. Before global-object/state definitions, introduce a short definition: genesis is justified/finalized; a valid quorum link from a justified source justifies its target; an adjacent child link finalizes its justified parent and ancestors. Then Protocol F can specify verification and signing. Define `pi_r` as the complete proof finalizing ROTATE `C_r`, or use the new detached activation-proof notation from SC-1 consistently. In the setup, list `K_G,N_G` explicitly as members of `Theta` and state positivity/feasibility of all batch/request limits used by the progress claims.

### SC-7. Avoid same-symbol collisions in adjacent sections

**Locations and exact text:**

- Main line 1143: `$L_R(q)\leq qT_V+7\Delta_R+\delta_P$`.
- Main line 1163: `$L_G\leq ...$` is geographic-recovery latency.
- Main lines 1183–1185 immediately redefine `$L_R$` and `$L_G$` as control-metadata sizes.
- Appendix lines 1351 and 1354 define signature chain `$C_r=...$` and `$C_0=()$`, while main lines 354–358 and 625–626 already use `C_e` for global checkpoints and `C_0` for finalized genesis.

**Issue:** Local contexts technically distinguish most of these uses, but the latency and size definitions are adjacent and their units differ. The appendix's `C_0=()` directly resembles an assignment to the genesis checkpoint, needlessly complicating the supposedly self-contained global-chain proof.

**Minimal recommendation:** Use `ell_R, ell_G` (or another distinct size notation) for control-bit bounds. Use `Sigma_r` or `mathcal C_r` for signed forwarding chains, including their empty base case. The committee `Q`, report-epoch bound `Q`, and local appendix node `p` are locally scoped, but renaming the report-epoch bound to `Q_adopt` would further reduce avoidable reuse.

### SC-8. Make the diagrams match the operational rules precisely

**Locations and exact text:**

- Main lines 96–99: “The three drawn sequencers illustrate communication roles ... A three-member equal-weight committee ...”.
- Architecture figure lines 24–29 instead show “Current primary” and “Regional peers”; lines 56–59 show two “Other-region sequencers” group boxes. There are no three individual sequencers.
- Pipeline figure lines 37–39 draw a retry arrow from the box containing “COMMIT / commit certificate”, labeled “timeout: rotate the primary”.
- Pipeline figure lines 70–71 say “Install the adopted anchor / at the next epoch boundary”.

**Issue:** The stale architecture caption is a direct mismatch. The retry arrow can be read as permitting retry after commitment, which R5 forbids. The lower annotation can be read as installation merely after report selection, without the epoch's justification certificate.

**Minimal recommendation:** Describe the diagram's role/group boxes without naming a drawn committee size; keep the one-third example only if it has a clear purpose. Relabel the retry arrow “no commit certificate by deadline”. Relabel the global annotation “carry the justified control state into the next epoch”, and retain the separate fixed-boundary activation note. The two figures otherwise correctly separate Preview from Deliver, preserve original producers during carryover, and keep execution/publication outside the service.

### SC-9. Remove unexplained references to a previous manuscript from the protocol

**Locations and exact text:**

- Main line 426: “replace the previous broadcast/completion path.”
- Main line 1211: “The unchanged global agreement ...”.
- Main lines 1245–1248: “removal of compulsory cubic authenticated-broadcast completion ... The old completion's ... term is absent. The global signed-broadcast term is unchanged”.

**Issue:** No live LMD-GHOST rule or obsolete regional completion algorithm remains in the specified protocol. However, these comparisons refer to an unnamed earlier design. A reader given only this paper does not know what “previous”, “old”, or “unchanged” means.

**Minimal recommendation:** State the actual PBFT and global-broadcast behavior directly in their protocol sections. If comparison with the previous `n_i`-round regional broadcast is desired, add one concise, explicitly defined baseline in the comparison/cost discussion and state its counted objects. The comparison must not become a hidden dependency of the PBFT proofs.

## Checks that passed at the frozen source

- **Indices and roles:** `i`, global view, regional sequence, retry, author view, epoch, and mini-block height are defined separately; `alpha` remains immutable when a later primary carries the same batch. Original ORDER/BATCH signatures and current PBFT primary signatures have distinct roles (lines 188–220, 269–343, 601–607).
- **Regional safety dependency:** Strict weighted quorum intersection gives same-retry prepared uniqueness. For carryover, the intersecting honest COMMIT signer must have committed before signing the immutable higher VIEWCHANGE, because retirement prevents the reverse ordering. The earliest-conflicting-prepared-view argument then propagates the committed value (lines 837–876). No synchrony premise is used in this branch.
- **Regional liveness dependency:** The shared predecessor certificate bounds responsive entry skew. An honest retry primary waits for that skew plus one delivery interval, obtains a full honest quorum of reports, and completes voting before the next timeout when `T_V >= 8 Delta_R` (lines 889–924). This depends on the specified complete-object workload premise and immediately eligible batch input; arbitrary construction delay is not included.
- **Global-chain proof structure:** The induction goes from common genesis/parent, to a common validity predicate, to one signed-broadcast output vector, to a deterministic state transition, to honest-weight FFG link formation, then the next epoch's common state (lines 926–955). SC-1 and SC-2 are required to make its “deterministic” step literal. The paper does not rely on FFG alone to make reports converge.
- **Broadcast proof structure:** Appendix A's retained sets may differ when an origin equivocates to more than two values, but the proof only needs singleton-versus-multiple agreement. Honest forwarding of each retained value gives this property; it does not falsely claim all cap-two sets contain the same pair (lines 806–834 and 1362–1392).
- **Global theorem scope:** Continuous global synchrony is explicitly assumed and the loss of that guarantee is disclosed (lines 1306–1325). Local PBFT safety is separately delay-independent. These are not presented as safety under arbitrary global partitions.
- **Consumer semantics:** Verification uses full regional chains and finality evidence, geographic handoff processes every intervening activation, and only an unretained preview tail may be replaced (lines 551–572, 957–982).
- **Liveness scope:** The theorem excludes transaction inclusion/fairness, requires eligible global weight, bounded backlog, per-outcome admissibility and sufficient tenure, and retains a late-tail term in geographic recovery. The bounds do not infer a throughput ceiling from a latency upper bound.
- **Complexity scope:** Normal regional phase messages are quadratic, the unaggregated full-certificate relay remains cubic in signature bits, and global broadcast may dominate. These distinctions are correctly stated apart from SC-4's missing report-evidence accounting.

## Review boundaries

This is a source-level independent review, not a rendered-layout certificate or implementation test. No claim was imported from PBFT, Casper, or Dolev–Strong merely by citation; the supplied arguments were checked on their own. Prior workspace memory was used only to reinforce source freezing and the separation of regional safety from the global synchrony envelope; the current manuscript remains authoritative.
