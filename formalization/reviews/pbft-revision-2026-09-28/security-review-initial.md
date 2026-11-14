# Initial independent security verification: PBFT revision

## Summary

**Final Verdict:** The regional PBFT safety argument is sound under its stated fault and persistence assumptions, including hidden commit certificates. The complete manuscript is not yet verified: proof-valued state fields create a **Justification Gap** in canonical activation and global agreement, and its unrestricted verification-work upper bounds contain a **Critical Error** when read as bounds on all work induced by Byzantine traffic. No conflicting regional commit construction was found.

This is a verifier-only review. It does not repair the manuscript, certify an implementation, or import missing protocol rules. The review request uses Markdown, so the proof-verifier skill's default LaTeX output format is not used. All line numbers below refer to the frozen initial source, not to later edits.

### Frozen artifacts

| Artifact | SHA-256 |
|---|---|
| `formalization/main_PBFT.tex` | `69da5fb098f108cca877181f5e4cf44cca4a42ce878203ff324bf16dc99d0e23` |
| `formalization/figures/service-architecture-pbft.tex` | `dcd8e440a75caf1dcbefef99f1ac197c50fbfebdc2d68591d742f69701c5aa13` |
| `formalization/figures/stream-pipeline-pbft.tex` | `7c2218dae0575d1e87cc2237611dd384ad4ded90b0015eb06f4d6fc35194bc18` |

The manuscript is preserved beside this report as `main_PBFT-initial-snapshot.tex` (same hash). The review leaves `main_v1.tex` and the manuscript untouched.

### List of findings

1. **F1 — Justification Gap: semantic activation and readiness state are not concretely separated from proof encodings.**
   - **Location:** Lines 270–276: “an activation proof $A$” and `$R_0=((c,\gv,0),\Hash(A),\Tag{INIT},h,q)$`; lines 348–350: the global state contains `$A$`, described as “the global view's activation proof”; lines 688–690: the pending package contains `readyProof`; lines 731–736 assign `$A=\pi_r$` before the next target is hashed.
   - **Issue:** Lines 149–150 say that semantic identifiers exclude witness encodings and signer subsets, but the actual typed fields contain proofs. A finality link can have several valid quorum subsets; no unique proof-byte string is guaranteed. Different honest nodes can therefore possess equally valid proofs without possessing identical `A` values. The blanket semantic convention indicates an intended normalization, but the protocol never defines that projection for these proof-valued fields. Under literal canonical encoding of the displayed tuples, different proof subsets produce different `R_0`, regional parents, and subsequent checkpoint hashes. Under the intended witness-excluding interpretation, an explicit activation identity and readiness representation are missing. Consequently the global proof's assertion that a common finalized checkpoint gives a common next activation/state is not established by the concrete objects.
   - **Consequence:** The local PBFT theorem remains valid **for a fixed common authorized `R_0`**, but its installation at geographic activation, global target equality, authority, and downstream service safety/liveness are unverified until the semantic objects are defined. This finding is a specification gap rather than an unconditional theorem refutation because the manuscript explicitly declares the intended witness-independent convention.

2. **F2 — Critical Error in verification-work bounds; no regional agreement attack follows.**
   - **Location:** Lines 427–428: “Every member first authenticates a signed message and extracts any conflict evidence before applying the phase-specific duplicate rules.” Lines 1205–1209 bound accepted proposals and evidence pairs; lines 1220–1223 claim uncached endpoint checking is “at most $O(k^3(k+v_F))$ operations”; lines 1250–1251 state that a normal receiver checks `$O(n_i)$` signatures and a full NEWVIEW or all uncached certificate variants cost `$O(n_i^2)$`. Appendix A2, lines 1370–1373, verifies chains and witnesses before A3's cap-two retention.
   - **Issue:** A cap on votes, retained values, or forwarded evidence is not a cap on received signature verification. A Byzantine primary can send arbitrarily many correctly signed conflicting PREPREPARE messages for one retry. Under the required authenticate-before-duplicate rule, each triggers authentication even though only the first proposal can receive a PREPARE and only one conflict pair is retained. This is not merely invalid unauthenticated traffic. Equivalent valid witness encodings and signed-chain variants present the same issue globally. Thus no bound depending only on committee size covers all incoming-induced checks without an additional admission budget or explicit restriction of the accounting to prescribed traffic. Caching body validity by semantic digest does not, by itself, cap authentication of distinct signatures/wrappers.
   - **Consequence:** The stated polynomial verification-operation upper bounds are false as unrestricted adversarial-work bounds. The honest-generated regional message bounds remain justified because honest nodes vote, relay, and emit evidence only a bounded number of times per key. The delay envelope already assumes admitted honest traffic completes and therefore excludes a successful capacity-overload execution; this finding does **not** refute conditional safety or liveness within that envelope. What is missing is a sound resource-accounting scope and a justified admitted-verification budget, not another quorum argument.

## Detailed Verification Log

### 1. Model, fault sets, and authorization

**Text checked:** “A quorum ... weight $>2\mathrm{wt}(Q)/3$” (120–121), fixed Byzantine set and persistent restrictions (162–173), fixed responsive global/regional sets (175–186).

The per-region strict Byzantine bound implies a strict global Byzantine bound after summing over the partition. A certificate quorum and a responsive honest quorum intersect in positive honest weight. The model correctly separates permanent cryptographic safety from interval-dependent availability. Whole-region outages are allowed only while the global responsive weight remains above the threshold. It is not sound to read the manuscript as tolerating arbitrary regional outages or arbitrary global partitions; it does not make that claim.

The body-delivery assumption includes retrieval, queueing, verification, and full view-change bundles (153–160). Therefore a small digest is not being substituted for an availability proof. The regional timing theorem is a bounded-delay theorem with known retry periods, not the original PBFT adaptive partial-synchrony theorem.

### 2. Immutable batch identity and certificates

**Text checked:** Mini-block and batch tuples (279–297), prepared certificates (302–313), and `VerifyRC` (315–325).

The signing domains include configuration, global view, sequence, author retry, height, and parent as appropriate. Resetting the retry number at each sequence does not create signing-domain collisions because `u` remains part of the domain. Carrying an old batch into a later PBFT retry preserves original ORDER/BATCH signatures; the later primary signs the wrapper. The commit verifier checks author retry no greater than commit retry. Binding the immutable value to its predecessor and exact consecutive range prevents substitution or splicing.

Prepared and commit certificates need not recursively carry every older NEWVIEW. With more than two-thirds signatures, an honest signer has checked that local justification; induction over retry number is sufficient. A prepared certificate alone is correctly distinguished from a committed outcome.

Semantic `R_u` excludes the retry of commitment and signer subset, so identical batches committed in different retries remain the same outcome. **F1** concerns the separate activation root and proof-valued global state, not this correctly normalized regional outcome definition.

### 3. Within-retry uniqueness and hidden-certificate carryover

**Text checked:** “Never prepare a different digest in that retry” (441–450); retirement and immutable VIEWCHANGE snapshot (469–474); prepared uniqueness/carryover proof (837–857).

The quorum-intersection calculation at 796–804 is correct. Two conflicting PREPARE certificates in one retry would share an honest signer, forbidden by R2.

For a COMMIT quorum for `x` in retry `v`, its honest COMMIT signers have weight strictly greater than `W_i/3`; all persisted a prepared certificate first. Any later NEWVIEW quorum intersects that set. At an intersecting honest signer, the later VIEWCHANGE cannot have preceded its old COMMIT, because sending VIEWCHANGE retires all lower retries. Hence the signed snapshot contains a prepared certificate at least as recent as `v`. If `v'` is the first later retry with a conflicting prepared certificate, every reported certificate with retry in `[v,v')` names `x`; the highest reported certificate must therefore select `x`. Any honest PREPARE signer for a conflicting `v'` value gives a contradiction.

This argument does not require that the COMMIT quorum was assembled or revealed before the new view. It reasons about the individual signatures and retirement ordering, so hidden certificates are covered. A higher-prepared value may replace a merely locally prepared value when no older possible commit protects that value; the absence of a separate permanent prepare veto is correct.

R4 accepts late complete commit certificates, and R5 gates the next sequence on a verified predecessor. Induction over sequence number gives a unique chain from a **fixed** `R_0`. Restart persistence prevents duplicate voting. No regional agreement defect was found in these rules.

### 4. Regional liveness, skew, and restart behavior

**Text checked:** Sequence entry and predecessor relay (430–460), timer/construction times (465–492), recovered-node abstention (496–501), and progress proof (889–924).

For a sequence after genesis, the first responsive honest holder relays its predecessor certificate; all members of the fixed responsive regional quorum enter within one `Delta_R`. The preceding holder need not be Byzantine or available outside that set. Equal-rate local timers preserve that entry skew; peer claims cannot accelerate them.

At retry `v`, all responsive VIEWCHANGE reports reach an honest responsive primary by `a+vT_V+2Delta_R`. Its scheduled construction occurs between that bound and `+3Delta_R`; NEWVIEW, PREPARE, COMMIT, and relay fit through `+7Delta_R`. The next earliest retirement is `+T_V`, so `T_V >= 8Delta_R` leaves positive slack. An honest member that already committed need not report because its relayed certificate terminates the remaining members' unresolved sequence. The cyclic identity schedule eventually reaches a responsive honest primary; it cannot use a one-third **identity** fault bound inferred from weights.

The normal honest view-zero path has three actual complete-message delays after its proposal send, provided proposal construction meets the eligibility/timing qualification at 1106–1108. Dependent sequences are not pipelined. The stated bound does not establish a throughput ceiling.

A node with an unknown timer origin abstains until a new sequence or global activation; it does not reset into a retired retry. This preserves safety, but it does not prove liveness with a changing responsive set. The theorem explicitly assumes a fixed responsive quorum through entry and retries, so that limitation is in scope, not a counterexample.

### 5. Authenticated global report broadcast

**Text checked:** Lemma 806–834 and Appendix algorithm 1341–1392.

The cap-two proof is valid for agreement on a singleton versus bottom. For a retained value whose chain contains a responsive honest signer, that signer's earlier prefix was multicast. Otherwise the chain length is at most `n-1`, so its first responsive honest recipient can append and forward. If the recipient has already forwarded that value, its earlier send suffices. A length-`n` chain must include a responsive honest signer. Hence any honestly retained value reaches each other responsive honest member or encounters an already size-two set. If one has two values, all finish with two and return bottom; otherwise the single retained value propagates. The sets need not contain the same pair.

This reasoning requires exactly the complete-object round and admission assumptions stated. It does not survive selectively missed verification deadlines. The manuscript explicitly acknowledges that restored connectivity alone does not recover an already stalled global epoch (1309–1314). **F2** concerns the claimed incoming verification-work bounds and enforceability/scope of admission, not this conditional broadcast argument.

### 6. Global transitions, FFG, activation, and retention

**Text checked:** Global epoch and transition protocols (624–701), finality/activation (708–752), and global proof (926–955).

Conditioning temporarily on a canonical common activation identity/state (the unresolved **F1** obligation), the global argument is coherent. Common parent plus synchronous report agreement produces the same report vector. Regional chain uniqueness makes longest compatible extension selection deterministic. FFG votes occur only after target derivation and complete data verification; a responsive honest global supermajority suffices for the link. Adjacent links do not create surround votes, and one vote per target epoch excludes conflicting same-epoch certificates.

ROTATE retains the **parent's** adopted tip, matching its READY proof, rather than adopting a simultaneous extension for which readiness was never collected. Pending CONFIRM preserves it. The adjacent link finalizes the rotation; activation occurs at the fixed beginning of epoch `r+2`. A new global view creates a fresh regional signing domain, even if the selected region equals the old region. Old regional certification cannot extend the accepted new global history. READY guarantees possession at the signed tip, not future responsiveness.

The proof does not establish arbitrary-partition global consensus from FFG alone. Its synchronous report layer already establishes the unique target chain. That scope is accurately stated.

### 7. Consumer safety, catch-up, and repair

**Text checked:** D4–D6 (551–572), soundness proof (957–982), and retention lemma (984–999).

Conditioning on the global canonical-state obligation, consumers verify complete bytes, regional chains, finality ancestry, and authority. They append only previously undelivered positions and retain delivered data across handoff. Applying every intermediate activation during catch-up avoids preserving an old preview through a later reuse of the same heights. Historical statements remain admissible as evidence without reactivating old production authority.

Global endorsers retain the entire adopted prefix, and the responsive global set holds it before voting. The sequential retry formula `(a+1)theta` is correct for bounded requests when a responsive holder responds within the complete deadline; multiple chunks add their bounds. No proof of recovery exists when every holder is unreachable, and the manuscript explicitly states that limit. Execution validity, transaction inclusion/fairness, and censorship resistance are not proved or implied.

### 8. Service liveness, geographic selection, and capacity

**Text checked:** Service theorem/proof (1001–1035), backlog definitions and tenure condition (1064–1092), and recovery bounds (1139–1178).

After temporarily assuming F1's common-state conclusion, a continuously healthy eligible region originates READY for the retained prefix, independent of whether the failed regional sequence completed. Distinct selections proceed cyclically and cannot repeatedly jump over a continuously ready region; same-region recovery creates a fresh global view. The global responsive-quorum condition remains necessary.

The report limits must fit each complete outcome. The workload bound `Q` is not derived from nominal maximum packing; it explicitly bounds backlog up to the target. Later outcomes cannot overtake a fixed target. Minimum tenure exceeds startup plus synchronization/adoption slack, avoiding repeated replacement of a healthy successor before its first adoption. These hypotheses are substantive and are not consequences of PBFT.

The geographic latency bound includes old-tail arrivals that can reset no-progress, successor state transfer/readiness, ROTATE/CONFIRM, activation, and successor progress. It does not silently promise finality for a preview discarded by a handoff. No additional arithmetic defect was found in these conservative bounds.

### 9. Honest traffic, Byzantine resource behavior, and payload accounting

**Text checked:** Complexity sections (1181–1303).

For prescribed honest transmissions, PREPREPARE plus all-to-all PREPARE/COMMIT is quadratic. Every honest member relays one full unaggregated commit certificate, so relay **messages** stay quadratic while relay signature **bits** are cubic. A NEWVIEW carries at most one prepared certificate per sender; its multicast is `O(n^2 L_R+n^3 kappa)` control bits and, without caching, `O(n^2 B_R)` payload bytes. The single-outstanding design removes compulsory regional authenticated-broadcast completion after each successful batch. The global signed-chain broadcast remains cubic in direct transmissions and can dominate total costs.

Those counts must be described as prescribed/honest-generated traffic. They do not bound a Byzantine sender's own output. **F2** additionally prevents treating the signature-check counts as total adversarial processing bounds. The current delay premise can assume those floods do not delay admitted honest traffic, but that premise is not a concrete resource-defense protocol.

The payload inequality carefully distinguishes total primary pushes `M_0` from distinct missing bytes `M`. Its final `(a+2)M` comparison is explicitly conditional on `M_0,M_r <= M`; repeated identical content at different positions need not satisfy that condition. The extra copy correctly accounts for an in-flight push racing a successful pull. Retained history and missing proof ancestry are not treated as constant-sized roots.

### 10. Conflict evidence and figures

**Text checked:** Evidence algorithm/proposition (769–789, 1037–1049) and both PBFT figures.

Comparison keys distinguish author retry, sequence, vote phase, origin, and target epoch. Permitted cross-retry votes, alternative witnesses for the same semantic report, forwarding, and silence are not automatically equivocation. Reading each primary proposal/NEWVIEW action as the specified one-shot action and persisting the full signing history, the listed evidence predicates cannot be produced by an honest participant. Attribution does not establish a penalty schedule.

The architecture figure correctly shows PBFT phases, full validation, unaggregated weight thresholds, Preview versus backbone/finality, and consumer pull. The pipeline figure shows the highest-prepared carry rule and the two-link global finality dependency. Its arrows are not equal-duration rounds. No LMD-GHOST string or dependency remains in these three draft artifacts. No figure-specific security contradiction was found.

## Dependency disposition

| Result | Initial verification status |
|---|---|
| Quorum intersection | Verified |
| Authenticated origin broadcast | Verified under explicit continuous round/admission envelope |
| Prepared uniqueness/carryover | Verified, including hidden commit certificates |
| Regional PBFT safety / unique chain | Verified conditional on one common authorized `R_0` |
| Regional PBFT progress | Verified under fixed responsive quorum, entry delivery, timer, and complete-capacity premises |
| Global agreement/authority | Unverified due to F1; subsequent algebra/rules checked conditionally |
| Verification soundness / consumer safety | Depends on unresolved F1 |
| Data retention/repair | Correct conditional on common globally endorsed state and responsive-holder premises |
| Service liveness | Depends on F1; otherwise conditional reasoning checked with capacity and tenure premises |
| Equivocation evidence | Verified under the stated one-shot protocol actions and persistent signing history |
| Honest-generated message/bit costs | Supported |
| Unrestricted incoming verification-work costs | Invalid as written, F2 |

## Reference and scope note

The original PBFT primary source was checked for the distinction between prepared and committed-local states and for its highest-prepared view-change rule: [Castro and Liskov, OSDI 1999, Sections 4.2, 4.4 and 4.5](https://www.usenix.org/legacy/events/osdi99/full_papers/castro/castro_html/node4.html). The manuscript's weighted quorums, per-sequence retry reset, immutable author metadata, and bounded-skew fixed timers were checked directly rather than accepted by citation. Prior project memory was used only to preserve the distinction between failures outside the continuous global envelope and theorem refutations; the findings above were derived from the frozen current source.
