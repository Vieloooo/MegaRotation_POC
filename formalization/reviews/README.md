# Review record for the service rewrite

The [28 September main_v3 review and validation record](main-v3-2026-09-28/validation.md)
covers the status-aware cross-region wrapper, four-case complexity analysis,
exact preservation of the regional definition, and final PDF presentation checks.
The historical reports below apply only to their recorded snapshots.

The [27 September liveness attack audit](attack-audit-2026-09-27/README.md)
examines deadlocks, interruptions to live streaming, and recovery boundaries
of the current source without changing the protocol. It distinguishes
concrete conditional failures from pauses permitted by the stated model.

The current September 26 LMD/FFG redesign and its proof--flaw--repair cycles
are recorded in [lmd-ffg-redesign/README.md](lmd-ffg-redesign/README.md).
That revision replaces regional completion with reorgable LMD streaming.
The reports below apply to their historical source snapshots, not this redesign.

The current manuscript specifies a standalone L1-authorized sequencing service. Its boundary ends at verifiable stream delivery.

The preceding security patch added same-region recovery in a fresh generation, ordered consumer handoff replay, and explicit progress prerequisites:

- [Adversarial audit](security-attack-audit-2026-09-25.md): frozen pre-patch attack traces and scope qualifications.
- [Patch comparison and implementation](lightweight-security-patch.md): selected lightweight repair, alternatives, cost and validation.
- [Independent patch verification](lightweight-security-patch-verification.md): review of the modified algorithms and their proof dependencies.

The following reports concern the preceding September 25 service rewrite and apply to their recorded source snapshots:

- [Correctness v1](rewrite-correctness-v1.md): frozen first-draft findings, including missing whole-proposal authentication and explicit READY behavior.
- [Correctness v2](rewrite-correctness-v2.md): strict independent re-verification after repairs.
- [Correctness v3](rewrite-correctness-v3.md): final targeted review of clarified retrieval/backlog assumptions and their theorem dependencies.
- [Complexity v1](rewrite-complexity-v1.md): first independent transport and cost audit.
- [Complexity v2](rewrite-complexity-v2.md): recheck and two remaining accounting qualifications.
- [Complexity v3](rewrite-complexity-v3.md): final check closing those qualifications.
- [Presentation v1](rewrite-presentation-v1.md): figure, interface, algorithm and scope findings, repaired in subsequent source revisions.
- [Final validation](rewrite-validation.md): final artifact hashes, build/render checks and review coverage.

The mathematical reviews apply to the explicit fixed-configuration, synchronous delivery, responsive-weight, retained-data, healthy-successor and workload assumptions. They are independent human-readable proof audits, not machine-checked verification or a deployment security certification.

Other files in this directory concern earlier designs or source research. They are preserved for history and must not be treated as validation of the current manuscript. The previous manuscript and PDF are in `../build/pre-rewrite-20260925/`.
