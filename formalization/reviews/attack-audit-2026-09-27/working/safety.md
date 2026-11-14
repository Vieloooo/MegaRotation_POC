# Independent safety and authenticated-broadcast audit

Source: `source/main.tex`, frozen SHA-256 `325ab6a05c0fd903f0f999a1d474218d122a302a54aa9c65a744a5dc989a4223`. Line references below refer to that frozen file. The manuscript was not edited. I read all 1239 lines before using any other review. This report covers authorized branches, cap-two authenticated broadcast, checkpoint determinism, FFG uniqueness, activation composition, and their safety proof dependencies; the coordinator separately investigates consumers and resources.

## Verdict

**No concrete in-model finalized-safety or broadcast-agreement counterexample found.** The proposed attacks below fail under the document's exact fixed-membership, static-corruption, persistent-signing, complete-object synchrony, and admitted-traffic premises. This is a bounded review, not a claim that all attacks are absent. There are specification/implementation hardening points and important excluded failure modes; they should not be promoted to proved attacks on the stated model.

The proof-verifier workflow was used for the supplied correctness claims. Within this review's lane, the main proof chain is viable: quorum intersection → fixed-epoch certificate uniqueness → unique certified control ancestry; independently, complete-object cap-two broadcast agreement → common report vector → common deterministic target → timely honest quorum. Prefix-preserving transitions and verified producer signatures then supply stream safety. No Critical Error in these steps was established. The explicit reliance on complete witnesses and persistent historical honesty is essential.

## Exact target and premises

The relevant design goals are: every accepted branch has complete consecutive bytes signed by its slot-authorized producers (241–243); delivered logs are append-only and mutually prefix-compatible (244–246); finalized handoff preserves all globally adopted entries and current speculative data uses the current view (247–249). There is explicitly no EVM/fairness guarantee (243), no preview agreement (246), and no guaranteed preservation of a particular batch or transaction (251–253).

The Byzantine set is fixed, each region has honest weight strictly above two thirds, and historical honest keys and signing histories remain uncompromised (174–178). Consequently global Byzantine weight is below one third. Complete admitted objects, including missing ancestry and verification, arrive within their bounds; capacity must reserve every prescribed honest message (158–164, 1192–1195). All report bodies and endorsed/adopted data are retained throughout the configuration (187–193, 383–393, 1115–1116). Global progress additionally uses a fixed responsive honest set of weight above two thirds (180–185). A quorum has distinct signers and weight strictly greater than two thirds (122, 152–155).

## Adversarial schedules checked

### S1. Three-or-more-way report equivocation against cap-two retention

**Attempt.** Let origin B sign three valid reports x, y, z. Deliver x,y to honest H1 and z to honest H2 in round 1. Try to exploit H1's two-value capacity to make H1 output bottom while H2 outputs z. Variants give each receiver a disjoint pair, deliver a new value only at a late deadline, or manipulate digest ordering to fill a receiver before its desired value arrives.

**Why it fails.** Both H1's retained values are forwarded even after ambiguity (1198–1202). H2 receives them at the next required deadline and retains enough distinct values to become ambiguous. If H2 is already full, it is already destined to output bottom. If a late chain contains a responsive honest signer, that signer already multicast its signed prefix in the corresponding earlier round. If it contains none, its length is at most n−1 and its first responsive honest receiver forwards at the next boundary. A length-n chain necessarily contains a responsive honest signer. Applying this propagation fact to two distinct retained values makes all responsive honest receivers ambiguous by termination (742–759). Different receivers need not retain the same pair.

**Verdict:** blocked by the stated protocol; no safety counterexample. The cap must remain two distinct *semantic values*, never two signature/witness encodings (1178–1181), and forwarding must continue after ambiguity (1202).

**Bounded validation.** `safety_ab_model.py` exhaustively checked the following abstract schedules; persisted output is in `safety_ab_results.json`:

| Identities n | Byzantine identities f | Responsive honest identities | Semantic values | Schedules | Disagreements |
|---:|---:|---:|---:|---:|---:|
| 4 | 1 | 3 | 3 | 512 | 0 |
| 4 | 2 | 2 | 3 | 4,096 | 0 |
| 4 | 2 | 2 | 4 | 65,536 | 0 |
| 5 | 3 | 2 | 3 | 262,144 | 0 |

Adversarial Byzantine-only-chain injections are arbitrary and independent at each honest endpoint through round f. All outputs agreed. The apparently large faulty identity counts are compatible with weighted honest supermajority: for example, n=4/f=2 can assign honest weights (.4,.4) and Byzantine weights (.1,.1). The model assumes correct signatures and reliable timely honest forwarding; it is not a wire-level test or a replacement proof.

### S2. Last-round selective delivery / signature-chain laundering

**Attempt.** Withhold a valid report until the last round, supply a length-n signature chain to H1 alone, and make H1 accept a sole value while H2 sees no value. Alternatively replay a short chain at the last deadline or reuse a chain from another parent/epoch.

**Why it fails.** A chain has distinct configured identities, starts with the designated origin, and every signature binds J, o, z, its position, and the earlier chain (1171–1181). The instance includes configuration, epoch, and exact parent (566–568). A length-n chain includes every identity, hence an honest responsive signer whose earlier multicast already propagated the value or ambiguity. Short chains are associated only with their exact round and rejected after that deadline (1192–1194). Another parent/epoch changes the signed domain.

**Verdict:** blocked. This conclusion depends on deadline enforcement applying to complete verified witnesses, not just arrival of a digest or incomplete packet.

### S3. Two certified checkpoints at one epoch

**Attempt.** Partition report views and persuade two quorums to certify different targets X and Y at target epoch e. Let Byzantine members double-sign, and deliver certificates selectively to different consumers.

**Why it fails.** Quorum intersection has weight greater than W/3 (690–697); Byzantine weight is below W/3, so the two quorums intersect in an honest signer. That signer would have to violate its persistent at-most-one-FFG-statement-per-target-epoch rule (625–628). If report agreement failed because timing assumptions failed, honest votes could split and prevent progress, but two certificates still cannot exist under these signing rules. Epoch-specific domains include c and both source/target identifiers (572–574).

**Verdict:** blocked without assuming report agreement. Removing historical-key integrity or crash-safe signing persistence would change this conclusion and is expressly outside the model.

### S4. Conflicting certificates at different epochs / adopted-prefix rollback

**Attempt.** Certify an alternative branch at a later epoch, rather than produce two same-epoch certificates, so the attacker can evade the per-target-epoch double-vote rule. Alternatively replay an old valid control chain after handoff.

**Why it fails.** Targets commit the immediate predecessor g_(e−1) (336–340); honest voting requires the complete deterministic target and justified parent (625–628). By induction from trusted genesis (557–558, 633), a divergence must first occur at one epoch, where S3 rules out two certificates. Every valid report extends the effective adopted tip (350–354, 383–388). ADVANCE extends it; HOLD and CONFIRM retain it; ROTATE retains the parent tip (587–608, 771–779). Replays cannot decrease the consumer's installed anchor (511–527).

**Verdict:** blocked under the adjacent-chain construction and trusted-anchor verification. This is stronger than an argument relying on LMD convergence, which the protocol deliberately does not assume.

### S5. READY certified at one tip, activate at a different tip

**Attempt.** Collect READY for parent tip q, choose a longer or incompatible snapshot tip q+, and use the existing READY proof to activate a successor that has not retained the actual inherited data.

**Why it fails.** READY binds the parent and exact inherited h,q (377–381). When a successor is selected, ROTATE explicitly retains the parent h,q and adopts no new range (601–606). CONFIRM freezes that pending tip (587–589). Activation requires the finalized strict-ancestor ROTATE and preserves its tip (634–643). A healthy receiver missing the proof or bytes must not start (636–637).

**Verdict:** blocked. This is an availability/readiness mismatch the present text already avoids.

### S6. Activation proof self-reference / alternate activation time

**Attempt.** Make the checkpoint's validity depend on its own finality certificate, or supply two certificate witnesses for one ROTATE that compute different activation times or successor views.

**Why it fails.** The report carries an already-existing activation proof at epoch start (344–350). ROTATE C_r becomes final through its direct child C_(r+1), and activates at the fixed start of epoch r+2 (634–643); this is a strict-ancestor proof relative to C_(r+2). Certificate signer sets and signature encodings are witnesses excluded from semantic object IDs (154–155). Activation resets view/slot domains once while preserving prior checkpoints and the stream tip.

**Verdict:** no circularity or witness-dependent state transition found. Implementations must enforce the semantic/witness separation stipulated by the text.

### S7. Fork-choice tree ambiguity via variable-size batches

**Attempt.** Produce overlapping batches or different envelopes with the same endpoint, so unioning report paths yields a DAG or ambiguous root-to-leaf path rather than the tree required by deterministic selection.

**Why it fails.** Each miniblock header binds slot u, within-batch index j, height, and preceding digest (263–270). The signed batch binds initial tip, k, and terminal digest (274–281). A terminal digest therefore commits to its full miniblock ancestry and within-batch indices; changing a batch boundary changes a j value or violates strictly increasing inter-batch slots (283–295). Subject to complete structural validation and collision resistance, two branches reaching the same endpoint cannot supply different batch ancestry. Identical signature witness encodings are immaterial.

**Verdict:** no structural counterexample found. An implementation that merely verifies individual signatures but does not enforce all header/envelope equalities would violate ValidBatch, rather than exploit an admitted branch.

### S8. Backdated authorized batches to conceal an outage

**Attempt.** A Byzantine scheduled producer withholds or later signs valid past-slot batches, releases them one at a time, and repeatedly resets the no-growth counter. It hopes to prevent useful failover while honest participants receive only attacker-chosen history.

**Why this is not a break of the stated goals.** Historical signature time is not proven; old valid batches from the effective current view can be admitted if they extend the current anchor and obey the cutoff (283–295, 350–354). However each adoption still strictly grows height, fulfilling the defined weak progress metric. The independent age timer N_G can trigger planned rotation even when z resets (596–600). Censorship resistance, persistence of a specified preview, and transaction inclusion are excluded (251–253, 1138–1148).

**Verdict:** a permitted adversarial behavior and a warning against interpreting height growth as useful transaction progress, not a demonstrated violation of the formal service.

## Specification and deployment boundaries to keep separate from proved attacks

### D1. A single missed AB deadline can permanently stop checkpoints and new streaming

**Classification:** concrete deadlock trace under a *transient violation* of the global delivery envelope; not a counterexample to the conditional theorem, which excludes arbitrary global-epoch-stall recovery (1145–1148). It becomes a design failure if the desired deployment goal includes recovery after a temporary network slowdown.

**Setup.** Four equal-weight validators B,H1,H2,H3 in one region satisfy the one-third fault assumption: only B is Byzantine. Epoch e starts from the common justified parent. All honest-origin reports disseminate correctly. B's report x is itself valid; it can even endorse only the existing anchor, since Hash(vector) is part of the checkpoint identity (338).

| Phase | Adversarial action | Honest result |
|---|---|---|
| AB round 1 | B sends its signed report x only to H1. | H1 retains x and must forward its length-2 chain. |
| AB round 2 | Delay H1's complete forwarded packet to H2,H3 past the round-2 deadline; every other required message can arrive on time. | H2,H3 reject this chain as late (1192–1194). This delay is the single violated global-envelope premise. |
| AB rounds 3–4 | B sends no useful chain. Restore prompt networking. | H1 forwards x only once (1201). H1 returns x for B; H2,H3 return bottom. No later honest propagation is prescribed for this value. |
| FFG epoch e | B withholds all FFG votes. | H1 signs X; H2,H3 sign Y≠X. X has weight 1/4, Y has 1/2. Neither exceeds 2/3. |
| All later time | Network remains timely; B keeps withholding. | Honest signing histories forbid changing the epoch-e target (625–628). No retry, new same-epoch voting round, parent skip, or global recovery procedure is specified. No C_e can be certified from these locked honest signatures. |
| Next epoch boundary | Required C_e is absent. | The next global epoch lacks its required justified parent (557–558); regional members cannot originate batches, HEADs, or reports from their stale anchor (531–537). Local slots keep advancing, but rotating to another local producer cannot release the global dependency. |

**Deadlock dependency:** `missed AB deadline → different report vectors → split persistent FFG votes → no justified parent → no next global epoch / no current-anchor regional origination`. No finalized-safety violation is needed. The malicious origin's valid value plus one delayed honest multicast is sufficient. This trace should be presented prominently if the review prioritizes live-stream stalls, while preserving the out-of-envelope classification.

1. **Explicit verifier adjacency.** The FFG verifier description allows a link with s<t (629–633), whereas the construction and safety induction use only adjacent links (628, 809). Under honest signing and less-than-one-third Byzantine weight this is not exploitable: honest members produce only adjacent links, and Byzantine members cannot forge a nonadjacent quorum. Still, a standalone implementation should explicitly reject t≠s+1 and require the checkpoint's predecessor to equal the certified source. Otherwise its acceptance language is broader than the argument it is intended to implement.
2. **Complete-object admission is a security premise.** An honest signer in a chain is useful only because its complete multicast reaches every responsive honest receiver on time. Digest-only forwarding, failed body retrieval, premature timeout, filtering honest forwarded equivocations after a per-origin byte quota is consumed, or stopping forwarding after cap-two would invalidate the broadcast proof. These are concrete implementation deviations, not schedules inside the exact admission premise.
3. **Consumer current-authority freshness needs a direct implementation invariant.** D4 pauses output when required control proof or bytes are missing; the consumer should enforce this at every known epoch boundary, including when an adversary supplies only a stale but valid activation proof. D3 alone only lists activation/authority/data checks. The intended combined rule blocks stale authority, but a preview-only implementation that omits D4 could violate goal (iii). The coordinator is reviewing this scope independently.
4. **Unbounded retained history and speculative width.** Two candidates per producer slot and finite path depth (309–318) do not provide a total memory bound over infinitely many slots. The text expressly retains admitted fork history throughout the configuration (187–193, 1115–1116, 1132–1136). Finite-storage exhaustion and LMD CPU growth are deployment concerns; they cannot honestly be reported as a counterexample under ideal retention and complete admitted-object delivery. In particular, 158–164 expressly includes verification in the network budget but does not separately state a bound for growing LMD traversal (1088–1091).
5. **Stall recovery, partitions, and compromised historical keys.** A violated global delay/quorum premise can halt checkpoint progress, and no arbitrary global-epoch-stall recovery is specified (1145–1148). Crash rollback or later compromise of old honest signing keys can defeat the certificate proof but contradicts 175–178. These boundaries should be prominent deployment assumptions, not hidden inside an unconditional safety/liveness claim.

## Remaining verification limits

This was a source-level adversarial proof review. There is no implementation to fuzz, no network parser or signature library to validate, no deployed clock/admission policy to test, and no proof that a physical system can maintain the stated complete-object budgets indefinitely. The bounded broadcast model verifies only the semantic forwarding/retention logic. The source's claimed execution correctness, fairness, censorship resistance, economic penalties, dynamic membership, and arbitrary global-stall recovery are outside its service and outside any affirmative result in this report.
