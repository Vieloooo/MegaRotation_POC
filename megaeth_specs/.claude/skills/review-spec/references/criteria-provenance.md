# Criteria Provenance

The eight criteria in [SKILL.md](../SKILL.md) merge the criteria of two independent reviews of `specs/block-certification/` run in September 2026, one by Claude (review A) and one by Codex (review B), each settled with the specification's author before its review started.
Both registers live under `.sisyphus/plans/`, which is gitignored, so this file is the durable record of where each criterion came from.

## Review A

Five ranked criteria, with readability last:

1. Precision and completeness — every MUST rewritten as "actor, on event, MUST do X" with no empty slot; every decoder, predicate and procedure total over extremal and aberrant inputs; a cross-rule interaction matrix of rules against disruptive events; nothing defined by reference to unpinned behavior; one term, one definition site; every MUST admits a test assertion.
2. Attack program — an explicit threat model over every actor (honest, rational, Byzantine); an enforcement test per MUST; invariants listed then attacked; attack trees by goal; the RFC 3552 attack list against every message type; escape hatches first; rational deviation as well as malicious.
3. Reasonableness of Security Considerations and Rationale — RFC 3552 structure; each Rationale paragraph names the alternative and why it lost; every claimed necessity checked; guarantees on one page derivable from rules on others; no invented guarantees.
4. Readability, ranked last with conciseness winning every tie — reported only where ordering or wording produces ambiguity or a misleading heading.
5. Conformance and traceability — spec-versus-implementation discrepancies logged in a separate register, never as spec defects; activation and compatibility; executable counterexamples where a trace exceeds a few steps.

Review A's method contributed the claims, actors, invariants and enforcement map built before attacking, the interaction matrix, executable counterexamples, hash-based revision tracking with per-finding re-evaluation, and the three-category classification the author asked for.

## Review B

Eight agreed criteria:

1. Precise and complete semantics.
2. Consistency and interoperability.
3. Explicit assumptions and guarantees.
4. Safety under adversarial behavior.
5. Liveness and recovery.
6. Sound rationale and security analysis.
7. Verifiability and traceability.
8. Concise and usable presentation, with conciseness preferred over readability.

Review B's method contributed the threat model written out per review, each guarantee decomposed into the proof obligations its argument needs, the paragraph-level relevant-and-supported audit of Rationale and Security Considerations, a complete inventory of every observation with its disposition, pinned source revisions, and the separation of demonstrated defects from open questions, accepted limitations and editorial observations, with severity and confidence recorded independently.

## Mapping

| Source | Criterion                                               | Merged into                                                            |
| ------ | ------------------------------------------------------- | ---------------------------------------------------------------------- |
| A-1    | Precision and completeness                              | 1 (semantics), 2 (interaction matrix)                                  |
| A-2    | Attack program                                          | 3 (threat model), 4 (safety), 5 (timing and contention)                |
| A-3    | Reasonableness of Security Considerations and Rationale | 6 (rationale), 2 (cross-page derivability), 3 (no invented guarantees) |
| A-4    | Readability                                             | 8                                                                      |
| A-5    | Conformance and traceability                            | 7                                                                      |
| B-1    | Precise and complete semantics                          | 1                                                                      |
| B-2    | Consistency and interoperability                        | 2                                                                      |
| B-3    | Explicit assumptions and guarantees                     | 3                                                                      |
| B-4    | Safety under adversarial behavior                       | 4                                                                      |
| B-5    | Liveness and recovery                                   | 5                                                                      |
| B-6    | Sound rationale and security analysis                   | 6                                                                      |
| B-7    | Verifiability and traceability                          | 7                                                                      |
| B-8    | Concise and usable presentation                         | 8                                                                      |

Nothing from either list was dropped.
Criterion 5 is the only one without a counterpart in review A's list, where its content was spread across the attack program.
