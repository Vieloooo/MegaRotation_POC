# 07 independent source-first expectations v1

Prepared 2026-11-16 by the independent `bft_verifier` (GPT-6 Astra / xhigh), before any 07 builder implementation or explanation was available or inspected. Ownership is limited to `poc/verification/viewer-gfr/`. This document defines expectations, not observed PASS results.

## Authority and boundaries

Baseline after normal dependency merges: `335553359cd39a3a7494fbd9760436cdb2c41a33`. Exact consumable dependencies: 03 `e9c15e04bc83b335668f3ecd837367aa87344a89`, 06 `4b034dfbb6c3e1c7143208347b546daab8dfe298`. Only their public APIs were inspected during preparation. Regional BFT gitlink remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`; its design pins Commonware `2026.10.0` (`92a2f68`). The local submodule is not populated; the pinned design was read using `git show` in the existing `/mnt/reth/jy/regional_BFT_sim` repository, without modifying it.

| Source | SHA-256 |
| --- | --- |
| v10 09 GFR | `6922b77e1de0c8eaddde9f08d08d6734ae63844d406f0e6e32438e33ba0dabcc` |
| v10 00 architecture | `2de0929fc2f75fda51802c7c354146849a285a247283ca1d8fc7338b2a8e8871` |
| v10 02 Session Program | `a8cafc846acffda6b2d08915b9ccc251825c33cf84e776dae94d67611db47eb2` |
| v10 03 Manager | `dbcc467b6f3f52d7832112217c3caf27de1e09bdda11b9745393aea326fd5954` |
| v10 11 claims | `d34b5e270f7be5019a608e198b1e3d078670b3a86e1bfdb8173b0a53c35d5fde` |
| v2 PoC design | `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab` |

Relevant authority: v10 09 §§3–4; 00 §§3.2–3.4; 02 §§3.10,4.1,4.4,4.9; 03 §3.9; 11 §§1.1–1.4; v2 §§3,9–12; handoff 07; commander independent-wave dispatch and dependency-plan-v2. P0 has 3 regions × 5 replicas, quorum 4, TB=10 and fixed configuration-bound execution fixtures. This review does not establish BFT agreement, native signature correctness, 15-worker integration, live Espresso timing, full Force/RESUME, or bounded liveness.

## Rules derived before implementation

1. Event order is `(public key, class, class index)`, where Force and Resume share control class 0, then Expire=1, SessionEnd=2, Espresso=3. Controls mix by exact `(L1 block,tx,log)` position. Never order by enum declaration, delivery tick, worker clock or wall time. Source: 09:111–133, 319–324.
2. To commit any event at x, Espresso must expose a complete finalized prefix whose timestamp reaches x, and L1 must expose the entire finalized block that first reaches x−M_a, including all its controls. These are different from later L1/origin/DEP/Plan validation inputs. Source: 09:135–148,236,248.
3. An event is atomic across chain, publication index, selected/closed sets, Tail, Rec, pc and lifecycle outputs. WAIT may retain earlier completed events but must expose no effects of the blocked event. Retrying identical delivered inputs is idempotent. Source: 09:219,392,416–418.
4. Publicly absent proof/body means result not yet eligible; already published bytes absent from this reader mean WAIT before the affected event. Local cache cannot establish publication, and a reader cannot inspect producer-side future objects. Source: 09:98–109,146; 06 PE-READER-1.
5. For a selected descriptor and target hash, q_avail is the earliest public prefix containing any valid target proof and every actual physical body from the selected anchor to that target. Multiple publications/proofs may combine. Optional anchor evidence does not participate. A descendant proof covers its physical ancestors only with the entire linked target chain available. Padding has no body publication requirement. Source: 09:98–109,223,346–350.
6. Preserve q_avail computed from original public object positions when a cached candidate later becomes selected. Check the publication deadline with theta(q_avail), strictly less than session end. Check expiry with the current acceptance key x, strictly less than expiry at EVERY new virtual height, including padding. Source: 09:225–244.
7. Acceptance requires exact selected cid, open branch/session, exact anchor identity, hash-linked continuity with the already accepted physical prefix, fixed-window physical heights, valid finalized inputs and canonical execution/Merge. Metadata/root failures must not be repaired by overwriting proposed values. Source: 09:225–242; 00:554–558.
8. Sort eligible extensions by canonical physical-block hash and remove overlap against the updated prefix after each append. Proof-arrival order cannot alter a result's q_avail. Full-window acceptance closes the old session COMPLETE, computes its successor, and continues processing cached matching results within the SAME Espresso event. Source: 09:240–244.
9. Full-window handoff anchor is the accepted h_max physical block. SessionEnd anchor is the last accepted physical block or initial anchor if none. The successor reference remains the predecessor INITIAL anchor; successor window remains `[old_hmax+1,old_hmax+TB]` even with gaps. No terminal padding is invented. Source: 09:248–263; 00:558.
10. Expire applies only to the next missing global height in NEXT/PENDING. It preserves G and emits branch closure EXPIRED, then HALT. No SessionEnd applies in PENDING/HALT and no further Expire is pending in HALT. Source: 09:88,188–207,250,288–292.
11. Checkpoints require replay verification of complete supported state against exact public cuts. Matching only a head hash is insufficient; missing verification material is WAIT, inconsistent state/cuts are REJECT. Chain and Responsibility are pure reads at the same pc. Source: 09:381–422.
12. Distinct same-position lifecycle events must remain distinct. Identity is full `(position,kind,body)`; preserve event order and exact typed body. No dedupe by pc alone or by `(pc,kind)` alone. Source: 09:372–379.
13. Current-stage Force/Resume input must produce an explicit unsupported result at that position, with no partial event effects and no event skip. This is the dispatch scope restriction, not implementation of 09 §§4.4/4.6.
14. The real 04 FinalVerifier remains open. Pure-rule fixtures must be named non-cryptographic and bind exact descriptor, target and proof bytes; a boolean test verdict cannot be presented as certificate acceptance. Missing verifier prerequisites must remain WAIT; unavailable/unknown verifier capability cannot silently approve.
15. Missing prefix coverage may prevent proving WHICH event is next. An explicit unknown-frontier wait is honest; do not label a speculative timer as the proven original next position when an unseen earlier event can still exist. Once ordering coverage establishes the blocked event, WAIT must retain that exact position. In both cases last-completed pc and completed effects remain unchanged. This refines the API representation of 09 §3.4; it does not permit skipping or reordering an unknown event.

## Same-position comparison projection

Compare ordered G blocks/native identities and execution state; per-height GlobalSource result keys; branch and policy; selected descriptors; closed branch/session sets; Tail with boundary/reference/anchor/window; pc; ordered completed records/effects; ordered lifecycle bodies with their positions; and selected-result q_avail with original public positions. Include enough publication evidence to reproduce first completeness. Compare only data determined through pc, not an ahead reader's downloaded future prefixes, pending network fragments, receive ticks, wall metrics or opportunistic cache.

Represent proof provenance semantically by result key plus the public proof evidence needed for rechecking. If implementation retains all proofs, compare a sorted public proof set. If it compresses to one representative, the accepted contract must fix a deterministic representative OR exclude witness bytes from the semantic comparison and independently verify the retained witness. Never exclude result key, q_avail or source coverage merely because proofs differ. 09:348 explicitly leaves witness compression open.

## Lifecycle decision and remaining integration gate

There is a real source tension: 09:307–308 lists SessionAssigned in RESUME step 4 and Resumed in step 5, while 03:226 expects Resumed to install StateMaterial before a subsequent SessionAssigned prepares the new candidate. Escalated to lead before implementation. Lead decision: freeze one atomic per-event lifecycle batch; retain v10 record order; future consumer must stage the position-bound ResumeCut, anchor state and ALL Plan prerequisites before allocation/Enter. This is compatible with event atomicity and preserves both observable record order and material-before-allocation; actual consumer dispatch in 13 remains an OPEN integration gate. Current 07 must not implement a speculative partial RESUME.

## Finite acceptance cases

Every case uses complete, valid configuration and public inputs except the explicitly varied boundary. `WAIT unchanged` means unchanged state for the blocked event, retaining any preceding completed prefix. Native proof cases below are fixture verdict tests until 04 integration.

| ID | Trigger / perturbation | Independent expectation | Source |
| --- | --- | --- | --- |
| VG01 | Genesis with all required Plan material | NEXT(0,0), fixed 10-height window, genesis-only G | 09 §4.8 |
| VG02 | Genesis Plan material absent; repeat after delivery | WAIT, then same state as VG01 | 09 §4.8 |
| VG03 | Deliberately no eligible Plan | PENDING with exact boundary; later Expire can HALT | 09 §§4.1,4.5 |
| VG04 | Mixed Force/Resume, expiry, end, two Espresso blocks all same key | control L1 position order, expiry, end, q order | 09 §3.3 |
| VG05 | Watermark one microsecond before timer; local/wall time beyond timer | WAIT at timer, no closure; equality watermark enables it | 09 §§3.4,4.3 |
| VG06 | L1 header reaches x−M_a but complete controls missing | WAIT before event; complete controls enable identical retry | 09 §3.4 |
| VG07 | Controls covered, candidate still needs later finalized origin/DEP | WAIT same Espresso position; no exclusion or partial append | 09 §4.2 |
| VG08 | Public proof only at q0, body only at q1 | q0 completes without result; q_avail=1 and append at q1 | 09 §3.2 |
| VG09 | Body public but reader missing its message/header | incomplete reader prefix; WAIT rather than public-incomplete | 09 §3.4 |
| VG10 | Referenced body exists only in local cache | no q_avail and no append | 09 §3.2 |
| VG11 | Target proof plus target body but actual physical ancestor absent | no q_avail; no replacing ancestor with padding | 09 §§3.2,4.2 |
| VG12 | Complete descendant chain, only descendant has proof | accept covered ancestors and target; padding uses descendant result key | 09 §§3.2,4.2 |
| VG13 | Additional valid proof/optional anchor evidence arrives later | original q_avail unchanged; no duplicate G | 09 §§3.2,4.7 |
| VG14 | Candidate published before selected, selected at later Espresso event | original q_avail, current acceptance key, exact anchor match | 09 §§3.2,4.2 |
| VG15 | Result publication at end−1us and exactly end | first eligible; equality excluded and SessionEnd precedes it | 09 §§3.3,4.2 |
| VG16 | Any new virtual height expires at acceptance key, target itself later | reject whole segment/HALT first; no partial prefix or padding bypass | 09 §§4.2,4.5 |
| VG17 | Every new height expiry strictly after acceptance key | whole segment accepted once, including padding | 09 §4.2 |
| VG18 | Wrong cid/anchor identity, equal anchor height only | no append; wrong candidate never selected | 09 §§4.2,4.3 |
| VG19 | Wrong hash, parent, root, metadata, origin, window independently | invalid extension excluded; no field repair | 09 §4.2; 00 §3.3 |
| VG20 | Invalid proof before valid proof; duplicate/wrong subject verdict | first invalid cannot establish q_avail; valid later can | 09 §3.2; 02 §3.10 |
| VG21 | h10 accepted; successor h11 cached in same q | G through h11; old session COMPLETE; window11–20; same pc events retained | 09 §§4.2–4.3 |
| VG22 | h9 accepted then SessionEnd | anchor h9, reference initial anchor, window11–20; G stays h9 | 09 §4.3 |
| VG23 | SessionEnd with no new physical block | original anchor, fixed next window, no invented padding | 09 §4.3;00 §3.3 |
| VG24 | Old SessionEnd after immediate full-window handoff | no second closure/assignment for the old session | 09 §4.3 |
| VG25 | Two qualifying extensions in one q arrive in reverse input order | canonical hash order, continuous append, overlap deduped | 09 §4.2 |
| VG26 | Later handoff Plan waits within Espresso block after an earlier tentative append | entire Espresso event rolls back; no lifecycle/Rec/pc leak | 09 §§4.1,4.8 |
| VG27 | Expire next missing h, then later Espresso and SessionEnd | preserve G, HALT, no new append or reassignment | 09 §4.5 |
| VG28 | Unsupported Force/Resume before same-key Espresso | explicit unsupported exact control position; no skip/no partial effects | dispatch scope |
| VG29 | Input shorter/non-extending/conflicting finalized prefix | reject inconsistent continuation; preserve committed state | 09 §4.8 |
| VG30 | Valid checkpoint restored with complete exact cuts | replay-equivalent state; future events emitted exactly once | 09 §§4.7–4.8 |
| VG31 | Checkpoint wrong source/Tail/closed set/q_avail/record while head unchanged | REJECT, not trusted by head hash | 09 §§4.7–4.8 |
| VG32 | Checkpoint reads missing original proof/body/L1 material | WAIT; no unverified restored state | 09 §4.8 |
| VG33 | Two regular handoffs at same pc (fixture only) | all distinct lifecycle bodies survive dedupe | 09 §4.7 |
| VG34 | A sequential; B q2 then q0, q1 partial, last missing message; separate states | B waits for its own complete prefix; same-position semantic outputs match literals | 09 §3.4;11 G-S1 |
| VG35 | Same input delivery with different wall pauses and repeated calls | identical public state/records, no duplicate lifecycle | v2 §§10–12 |
| VG36 | Frozen theta with increasing q | no higher timer/unsupported control progress until watermark reaches key | 09 §3.4 |
| VG37 | Timestamp regression/source mismatch/gap | invalid source or WAIT according to 06; cannot advance through gap | 06 PE-READER-1 |
| VG38 | Chain/Responsibility called repeatedly between deliveries | same pc, no mutation, no independent local timer transition | 09 §4.8 |
| VG39 | Future RESUME lacks cut/state/Plan input | no allocation/close/reset event released; explicit deferred gate in 07 | 09 §4.6;03 §3.9 |
| VG40 | Native verifier unavailable | explicit capability boundary; no cryptographic PASS claim | dispatch;v2 §3.3 |

These are finite safety/conformance checks. No conclusion about eventual progress follows merely from a trace with missing public inputs or violated E-L/E-TS. Full-feature 07 stays OPEN until the real 04 certificate gate and required integration are checked.
