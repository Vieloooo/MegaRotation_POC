# Validation record

Review began at **2026-09-26 17:39:44 UTC** (27 September 01:39:44, UTC+08:00).
The final source/link/model-result check completed at **2026-09-26 18:01:10 UTC**,
approximately 22 minutes after the start and within the requested six-hour maximum.

## Source integrity

The manuscript and both included figures match their hashes recorded before the audit.
They were not edited. Frozen copies preserve the manuscript's figure directory layout.

| Active file under formalization/ | SHA-256 |
|---|---|
| main.tex | `325ab6a05c0fd903f0f999a1d474218d122a302a54aa9c65a744a5dc989a4223` |
| figures/service-architecture.tex | `e55c603d5b85abb8baf04fe7776714185e116de4e856a46e10682cf919c7d587` |
| figures/stream-pipeline.tex | `9806798a3f7f7b1db3c54b6c7d6ac60b66b3d7201bbc6da67bc48c992f452e56` |

The Git HEAD at entry was `bb9ff167f0505cf4e2c89cc1edd26ad09b6bb9be`.
The workspace already contained extensive manuscript, figure, index, and other changes.
The audit target is the working-copy hash above, not a claim that HEAD contains that text.
This task added the audit directory and a link in the existing reviews index.
It did not commit, push, or reset any work.

## Final report integrity and checks

Main report: `attack-review.md`.

SHA-256: `70b52678809368fbd967c7aba67bf7cb39c0c250ac5a0d8400097ea0d1764020`.

- All **47** local source/artifact links in the main report exist; referenced line numbers are in range.
- Inline and display mathematical delimiters and fenced blocks are balanced.
- The existing reviews-index edit passes whitespace validation.
- Three independent first-pass reviews covered liveness, handoff/consumers, and broadcast/safety.
- Targeted final reviews checked the synthesized findings, not merely the source.
- The final report corrects the time at which C2 becomes justified in the depth-one example, distinguishes honest/current-authority output from Byzantine wire traffic, and requires the selective deadline pattern before claiming that overload causes the L1 deadlock.
- No manuscript build was run because the protocol and figures were unchanged. No PDF or implementation test is claimed.

## Executable evidence

`working/liveness_traces.py` passed five rule-level checks:

1. One missed AB forwarding deadline produces different origin outputs and locked FFG weights 1/4 versus 2/4, with no quorum if the Byzantine member abstains.
2. Fully healthy depth-one production occurs at times 0, 14, 28, and 42 seconds under parameters satisfying every displayed timing/tenure inequality.
3. The valid weighted-identity example permits 300 seconds of initial silent producer terms.
4. An offline 40%-weight region leaves only 60% global weight, below the strict global certificate threshold despite full successor READY support.
5. Twenty-one epochs of an inadmissibly short-tenure configuration repeat HOLD/ROTATE/CONFIRM without ever scheduling an honest producer; the repeat state establishes the infinite symbolic extension.

`working/safety_ab_model.py` checked **332,288** bounded semantic schedules:

| n | Byzantine identities | Responsive honest identities | Values | Schedules | Disagreements |
|---:|---:|---:|---:|---:|---:|
| 4 | 1 | 3 | 3 | 512 | 0 |
| 4 | 2 | 2 | 3 | 4,096 | 0 |
| 4 | 2 | 2 | 4 | 65,536 | 0 |
| 5 | 3 | 2 | 3 | 262,144 | 0 |

These are transparent finite abstractions. Complete timely honest forwarding is assumed in the successful AB search; the deadlock example deliberately violates that premise. Neither check is a full implementation or machine-checked proof. The recorded JSON outputs are retained alongside the scripts.

## Literature and review scope

The exact local source is authoritative. Official/primary references were consulted for scope: Casper FFG, Combining GHOST and Casper, Ebb-and-Flow Protocols, and the Ethereum consensus fork-choice specification. The main report cites the relevant papers directly; it imports no unstated Ethereum recovery mechanism. The SIAM Dolev–Strong DOI page was inaccessible through the browsing tool, so no unseen text from that source is relied upon. The concrete signed-chain algorithm was checked directly from the manuscript.

The previous LMD/FFG redesign review index and final closure record were consulted by the coordinator only after independent first-pass reviews had been dispatched. Prior positive verdicts were not substituted for this audit's reasoning.
