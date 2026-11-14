# Academic specification structure: primary-reference review

Reviewed 2026-09-25 for the requested rewrite of `formalization/main.tex`.

## Primary texts actually examined

- Shrestha et al., *Optimistic, Signature-Free Reliable Broadcast and Its Applications*, CCS 2025, DOI [10.1145/3719027.3765220](https://doi.org/10.1145/3719027.3765220), corrected author manuscript [arXiv:2505.02761v4](https://arxiv.org/html/2505.02761v4). Examined §2, §3/Figure 1, §7.1/Figures 5–7, and Appendix B; the initial v2 reading was superseded by v4.
- Abraham et al., *Good-case Latency of Byzantine Broadcast: A Complete Categorization*, [arXiv:2102.07240v3](https://arxiv.org/pdf/2102.07240v3). Examined §5.1, Appendix C.1, Figure 10, and Theorem 16.

The first reference is a presentation reference, not authorization to replace the signed synchronous regional primitive with a signature-free asynchronous RBC. The author's [publication page](https://www.losa.fr/research/) says the published RBC contains an error corrected from v4 onward. The v4 front matter identifies a termination repair; its RBC uses an additional acknowledgement stage and separates optimistic commitment from termination. Use the corrected version as the citation target.

## Observations from the references

The CCS paper introduces parties, faults, channels, timing, and cryptographic notation before specifying task invocation/output events and properties. Its short RBC appears as a complete transition box. Its larger application separates typed data structures and utilities (Figure 5), construction handlers (Figure 6), and commitment logic (Figure 7). Initialized variables, precise guards, thresholds, assignments, and emitted messages appear in the boxes; prose explains why those rules work. Correctness is decomposed into local claims before proving the advertised properties. The model changes between primitives are stated explicitly. [Source: §§2–3, §7.1, Appendix B](https://arxiv.org/html/2505.02761v4).

Figure 10 of the good-case paper includes the proposal rule, the first-proposal vote rule, quorum forwarding and locking, the fast-decision cutoff, and mandatory timed BA. Its correctness proof distinguishes two fast decisions, a fast decision followed by BA, and BA-only decisions. The boxed cutoffs are `2Δ+σ` and `3Δ+2σ`; nearby prose contains inconsistent uses of `δ`, so the box and proof determine the transcription. The two-delay bound measures internal good-case decision, not protocol completion. [Source: Appendix C.1, Figure 10, Theorem 16](https://arxiv.org/pdf/2102.07240v3#page=26).

## Proposed organization for this design

1. **Introduction and construction overview.** Define the exact object the protocol orders, state the S/P separation, summarize guarantees, and show the requested three-sequencer architecture. Keep provenance and differences from deployed MegaETH short.
2. **System model.** Give participant sets, membership and fixed-weight configuration, honest/Byzantine behavior, authenticated channels, PKI and hash assumptions, network timing and clocks, local/global communication domains, initial state, and external execution/DA/L1 contracts. State when each fault/delay premise is required.
3. **Sequencer-rotation problem.** Define API input/output events and independently quantify validity, integrity, agreement/prefix preservation, availability, local progress, global progress, authority exclusivity, recovery, and accountable equivocation. Distinguish regional confirmation, global finality, and L1 settlement.
4. **Objects and state.** Define typed records, hashes, signing domains, message grammars, certificate verification, branch validity, and persistent local stores. One notation table should support the algorithms rather than introduce undefined objects.
5. **Regional protocol.** Separate transaction/packet ingress and speculative LMD, timed checkpoint BB, decision export, and local authority transition. Provide the full confirmation and evidence rules in algorithm boxes. Explain the pipeline in one accompanying figure.
6. **Cross-region protocol.** Give availability acquisition, checkpoint creation/validation, FFG vote production and receipt, proof formation, finalization, geographic readiness, and activation. Explicitly define the branch-local transition function.
7. **Block construction and publication.** Give P's deterministic input-consumption procedure, block boundary rules, native signing constraints, DA publication, L1 submission, and authorized recovery. Show which inputs are speculative, regionally confirmed, or globally finalized.
8. **Analysis.** State exact theorem assumptions. Prove quorum uniqueness, preservation through completion, local prefix induction, FFG accountable safety, geographic fencing, availability, and conditional liveness. Account separately for internal latency, exported-certificate latency, and completion costs.
9. **Incentive interface.** Define configuration/selection maps, admissibility constraints, timeout and epoch parameters, evidence-to-penalty map, and reward/publication interfaces. Do not mix subjective health policy with mechanically verifiable fault evidence.

This ordering is a proposed organization for our construction, not a reproduction of either paper's text.

## Algorithm-box contract

Each algorithm should declare:

- **Scope and actor:** which participant, configuration, generation, and instance runs it.
- **Inputs and outputs:** typed records and explicit events.
- **Persistent state:** defaults, monotonic counters, vote maps, accepted certificates, locks, decisions, and publication cursors.
- **Invocation and receipt guards:** authorization, membership, unique identity, parent validity, instance equality, deadline, and required data.
- **Transitions:** assignments and exact messages, their recipients, and the event they emit.
- **Repeat policy:** whether a handler runs once, per sender, per instance, or whenever its predicate becomes newly true.
- **Timer semantics:** fixed start or receipt-relative timer; event ordering at a cutoff; late-message behavior.
- **Missing dependencies:** buffer and request data, then retry the same validity rule. A digest alone must not satisfy a full-data predicate.
- **Exceptional inputs:** malformed messages, duplicate signatures, conflicting signed statements, invalid certificates, unavailable data, and obsolete generations.
- **Termination:** distinguish an early confirmation event from permission to stop sending recovery messages.

Avoid executable-looking calls such as `validate`, `recover`, `switch`, or `finalize` without a nearby defined predicate/procedure or an explicit external primitive contract. A full protocol may be modular; its module interfaces cannot hide additional consensus decisions.

## Authentication and evidence checklist

- Define canonical serialization; hash exact fields rather than prose descriptions.
- Bind signatures to protocol/version, chain/configuration, type, full instance key, signer identity, and payload commitment.
- Count each configured signer once; explain whether the broadcaster contributes an ordinary vote.
- Define certificate verifier as a predicate over distinct authorized signers and one exact message.
- Specify sign-once keys independently for order production, head updates, BB votes, exported decisions, readiness, and FFG votes.
- Run evidence extraction on correctly authenticated messages before dropping an ordinary duplicate or conflicting input.
- Give `VerifyEvidence` an explicit input and Boolean output; define the two statements and the comparison relation for every offence.
- Keep FFG double-target and surround-vote checks distinct from stream-header equivocation and BB double votes.
- Define which state survives restart; otherwise a sign-once instruction does not establish an honest non-equivocation invariant.

## Presentation acceptance criteria

- Read the document once in dependency order: no essential record type, variable, or predicate should first become defined after use in an algorithm.
- A reader must be able to reconstruct every normal, timeout, equivocation, and handoff transition from the boxes alone.
- Put protocol semantics in the boxes and correctness arguments in the surrounding text; avoid maintaining divergent prose and algorithm versions.
- Use theorem statements to express claims, with local proof obligations cited by name.
- Put detailed schemas and secondary verifiers in an appendix if needed, but retain cross-references from their use sites.
- Both requested figures should be native TikZ: one architecture figure with three regional sequencers and one timeline/block-construction pipeline. Use small monochrome shapes, mathematical labels, meaningful arrows, and informative captions.
- A figure with three displayed sequencers is architectural only: under a strict `<1/3` equal-weight fault bound, three nodes do not tolerate one faulty node. Do not imply otherwise in its caption or example.

## Completeness boundary

A call to a formally specified BA primitive is a complete composition relative to that primitive, but is not an implementation of BA. Given the user's request for a full protocol, either instantiate the fallback with a concrete verified algorithm in a box/appendix or label the remaining module contract honestly. Similarly, “FFG ensures progress” is insufficient: candidate convergence, source selection, and compatible voting/locking rules need a stated mechanism or explicit conditional theorem. More polished typography cannot close these proof obligations.
