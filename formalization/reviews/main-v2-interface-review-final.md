# `main_v2.tex` interface review, final integration pass

Reviewed 28 September 2026. Final target SHA-256: `90a15545b45170931b4ceef066baf614f71fcd4f050102334eeaf4d5d0c3619a`. Scope: standard SMR/application semantics, client request liveness and real-time ordering, preserved mini-block bytes, stake/PoS configuration, evidence predicates, and the boundary to the unfinished outer CFT protocol. The regional consensus algorithm has a separate protocol-design review; this document is not a mechanized or standalone proof certification.

## Resolved findings

1. **The replicated command now includes batch framing.** At lines 218–231 of the reviewed snapshot, `AppendBatch(B,M_B,reqs_B)` determines the headers, batching, empty mini-blocks, request order and returned `(h,offset)` positions. `Append(r)` is the client-facing projection. This resolves the earlier mismatch between a request-only command and mini-block-dependent state.
2. **Raw transaction bytes are preserved.** Lines 206–212 retain `tx=z`; request authentication/identifiers are detached metadata. Lines 292–306 define `reqRoot` over the ordered `(a,s,H(z))` descriptors as a batch-level field. It does not alter `H_h` or the v1 mini-block tuple.
3. **The detached metadata is authenticated by consensus.** `reqRoot` belongs to `x`; PROPOSE and VOTE both sign `H(x)`. `ValidBatch` checks the ordered request-root correspondence to mini-block transaction bytes, preventing metadata substitution after certification.
4. **The fork evidence has the complete session scope.** Lines 558–560 require equal `(c,i,v_G,v_L)` and same-type statements before testing incomparable branches. Honest new-session reuse of a local-view number is no longer an offense.
5. **The proposal bound applies to Byzantine producers too.** Lines 315–323 validate at most `N_R` same-view descendants on any admissible ancestry, and require monotone parent view. Combined with the comparable-vote rule, this bounds a member's new votes per view. It does not bound arbitrary rejected-input flooding, which remains outside the admitted workload model.
6. **Progress is mandatory in an honest view.** The first proposal is at the common entry boundary; certified descendants are proposed immediately until the quota/cutoff. A fair admitted queue therefore has an actual service opportunity rather than an optional proposal action.
7. **The real-time argument uses fresh invocations.** Correct clients create and first release a fresh request when invoking an operation. A request invoked after a completed response cannot already appear before that committed prefix.

## Final clarifications requested during this pass

At the time of the first reread, two small points were sent to the root agent for immediate integration:

- State explicitly that **all** signatures and certificates, including ORDER signatures, are evidence encodings rather than semantic SMR state. Otherwise two valid producer signatures over the same header could make the stored full `M_h` tuples byte-different despite identical semantic commands. The existing text excluded request signatures only.
- Define the retained verification context `Gamma`, and the configuration fields `Theta` and `G_0`, locally. The first reread used `Gamma` only as a `ValidBatch` argument without defining it. A concise definition as verified configuration, session anchor and cached ancestry suffices.

These points do not require changing the algorithm or the mini-block format. Their integration status is recorded below.

## Claim scope

- The source attribution remains correct: final S&P2020 Sync HotStuff supplies the synchronous pipeline/core, while the scheduled handover is explicitly the draft's adaptation.
- The AFT2025 permissioned-to-PoS compiler is not misapplied as a proof of synchrony-dependent safety. The fixed externally finalized stake snapshot is the construction actually specified.
- Stake selection, frozen weights and deterministic cyclic leader frequency are distinct and correctly described. Slashing evidence does not mutate a live quorum denominator.
- The manuscript promises request completion only in its stated indefinitely live fixed session and admission/fairness envelope. It does not promise unconditional transaction inclusion under arbitrary overload.
- Local certification remains separate from outer commitment. The virtual participant definition requires committed outer state transitions, authenticated outbox contents, persistence and deduplication, but explicitly leaves that transition function unimplemented.
- Cross-region election, handoff, crash recovery, reconfiguration and global proposer confirmation are still marked unfinished. No end-to-end CFT or region-loss durability theorem is claimed.

## Integration status

Both requested clarification edits were reread and verified in the source identified above. `AppendBatch` stores unsigned `(H_h,X_h)` pairs and `(a,s,H(z))` descriptors; every signature and certificate is explicitly in the evidence store rather than semantic state or command log. `Gamma`, `Theta`, and `G_0` now have local definitions. The admission assumption additionally states a reliable request/reply path for the live correct client. The complexity paragraph separates current-view voting/certification from historical synchronization, evidence and subscription costs.

**Verdict: all findings from this interface review are closed; no remaining blocking finding within the stated review scope.** This verdict concerns the fixed regional construction and its explicitly unfinished outer interface, not an end-to-end CFT protocol or unstated failure model.
