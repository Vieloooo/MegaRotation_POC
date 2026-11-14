# Independent interface and scope review

Reviewed file: `formalization/main_v2.tex`.

Reviewed SHA-256: `62480c2027294fe3a1d4fc34c842ec61e2523be52097b8bf45bdb97ca2f9f251`.

Scope: approved six-section outline, concise presentation, implementation-independent stream contract, standard SMR definition, command/proposal mapping, PoS scope, conditional input liveness, confirmation and evidence interfaces. This is not an independent verification of the whole Sync HotStuff temporal proof. Proof-verifier discipline was applied to the implications within this scope; no manuscript edits were made.

## Summary

**Final verdict:** the revised architecture and interface are coherent and substantially match the approved outline; three local definition/justification gaps should be resolved before calling the interface completely self-contained. No counterexample to the underlying regional consensus safety was identified in this scoped review.

The earlier request-ID/FIFO/reqRoot/Cantor/fixed-window machinery has been removed. There are exactly six substantive sections. Local notation appears where it supports an operation. The stream proof is opaque at the interface, and cross-region finality remains explicitly outside the implemented scope.

### Findings

1. **ORDER evidence comparison domain — Justification Gap.** At lines 359–360, the phrase “same authorized position” does not state whether two signed headers must share the producing view. A producer may lawfully return in a later view and propose a different candidate at an uncommitted mini-block height. Comparing only session and height would falsely slash that recovery. State the comparison domain as the same session, producing view, and mini-block height. This is a clarification of already-present authorization context, not another field.

2. **Submitted versus persistent request quantifiers — Justification Gap.** Lines 180–182 define liveness for every submitted well-formed correct-client request. Lines 203–205 and the final sentence of the progress proof cover persistent eligible submissions. State that a correct submission keeps the pending command available until confirmation, or explicitly bind the definition to that input-service discipline. Otherwise a one-shot request sent to a faulty recipient satisfies the literal definition but is not covered by the proof. No request ID, FIFO queue, or exactly-once mechanism is needed.

3. **Mini-block semantic equality — Justification Gap.** The public object at lines 133–137 includes the producer signature, while the block identity at lines 232–233 excludes signature encodings. Prefix comparability at lines 162–163 should therefore use semantic mini-block identity, not equality of the evidence bytes. State once that headers and contents identify mini-block values and signatures/receipts are evidence. This prevents two valid randomized signature encodings of the same header from becoming distinct delivered values in the formal definition.

## Detailed verification log

### Architecture and outline

Lines 68–95 identify inner replication, external consumer, and unfinished cross-region authority transfer. The inner synchrony caveat is appropriately one sentence rather than a sprawling auxiliary model. The six sections follow architecture, model/interface, regional protocol, retrieval, proof, and complexity. The remaining abstract implementation references support the protocol rather than forming an object catalogue.

### Stream contract

Lines 146–155 expose `Deliver` and `VerifyStream` without requiring the consumer to interpret a vote stage or run a commitment timer. The proof encoding can change with the BFT while the semantic contract remains constant. Early body buffering is separated from acceptance. Lines 157–171 limit immutable-prefix claims to the authorized regional session, so local evidence does not silently assert cross-region finality.

Authenticity follows from the concrete receipt adapter only after the inner safety proof is assumed: every receipt has at least one honest post-commit endorsement; the producer/header commitments identify its contents. Prefix safety then follows from agreement of committed block logs and contiguous consumer append. Semantic equality should be made explicit as Finding 3.

### SMR and batch mapping

Lines 174–189 use the standard agreement-at-position and eventual-client-request-completion formulation found in Sync HotStuff §I.A. Deterministic execution is stated without inventing a client API or a deduplication implementation.

Lines 191–197 distinguish an immutable batch-content command from its view-specific placement on a parent. The command-log and stream projections are different projections of the same committed block chain, so the text no longer requires a payload-only transition function to recreate view-dependent headers. Re-certification retains old contents/signatures, consistently with that mapping.

The input-service assumption correctly separates stream growth from transaction censorship. It explicitly excludes the unsupported inference that a productive Byzantine leader must eventually include every mempool transaction. Request liveness is conditional, not an unconditional property established by no-progress blame. The persistent-submission convention should be aligned with the definition as Finding 2.

### PoS scope

Lines 216–228 honestly distinguish L1-backed permissionless admission from the analyzed fixed-snapshot regional execution. No mid-view stake-denominator change is introduced, and reconfiguration is not purportedly solved by observing a new L1 state.

The compiler paragraph matches the AFT 2025 publisher abstract: its preservation guarantee requires the base properties under partial synchrony. The new draft does not claim that this theorem proves synchronous Sync HotStuff safe under a weaker network model.

### Confirmation, evidence and retrieval

Lines 349–354 give a concrete quorum-of-post-commit-endorsements realization of the opaque receipt. A vote QC is explicitly insufficient. The consumer can use a receipt for a committed descendant to authenticate the included prefix, provided the missing ancestry is supplied, matching the interface's prefix semantics.

The VOTE comparison domain contains session/view/position, so lawful cross-view re-certification is not mistaken for double voting. Producer/ORDER evidence needs Finding 1's explicit domain. DECIDED evidence is correctly tied to conflicting prefixes within a session, not mere timeouts or missing messages.

The poll-fetch protocol uses hints only to locate evidence, applies one acceptance rule across suppliers, and ties eventual recovery to available honest holders and fair retries. No extra region-election authority is implied by a data timeout. The retention assumptions are explicit; archive/pruning and total loss are not falsely claimed solved.

## Primary-source pointers

- [Sync HotStuff author PDF](https://dahliamalkhi.wordpress.com/wp-content/uploads/2019/11/synchotstuff-sp2020.pdf), §I.A for the SMR safety/liveness formulation. This older author PDF was used for the interface definition only; the core revised protocol's specified 4 May 2020 version is outside this review's source comparison.
- [From Permissioned to Proof-of-Stake Consensus, AFT 2025 publisher page](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2025.18), abstract for the compiler's exact partial-synchrony premise.
