# Final source and PDF presentation review

Reviewed: 25 September 2026.

- Source: `formalization/main.tex`, SHA-256 `eeefda017320e101cb5be660ce2b8132844daa829b541ba3c36ae8a25188328d`.
- PDF: `formalization/build/main.pdf`, SHA-256 `17803ca8cd176a8811fd02e84c366a4cd2f3850fc9260f36667c12d32c125d75`.
- Final size: 17 A4 pages.
- Scope: independent notation, presentation, theorem-scope, and rendered-layout review. No source edits were made by this reviewer.

## Verdict

**Pass.** All five findings in `broadcast-presentation-v1.md` are resolved. No outstanding correctness-scope, notation, or material layout finding remains from this review. The draft's results remain conditional interface-composition claims; this review does not turn the explicitly open BA implementation, global convergence, dynamic-membership, or incentive questions into established results.

The final text preserves all requested distinctions: local LMD-GHOST selects speculative order; Appendix C.1/Figure 10 synchronous broadcast decides a checkpoint with up to `B_C` order packets; all members complete BA; external acceptance needs DECIDED export; global FFG supplies A3 justification and A4 finality; a rotation remains pending until its finality proof authorizes activation. S execution is optional and private, while P must execute its assigned publication input. There is no PBFT stage.

## Resolved findings

1. The global checkpoint's action, evidence, optional activation/rotation package, effective generation, and `q(C)`/`k(C)` accessors are defined.
2. DECIDED endorsements are immediately multicast to regional members and P, supplying the explicit dissemination step used by export progress and spacing.
3. The schedule family explicitly includes geographic, local, global-control, and proposer schedules.
4. The FFG source epoch is `e_s`; it no longer shadows the regional-instance index.
5. Every counted HOLD requires a health-report quorum and justification proof, and consecutive HOLDs require explicit adjacent links. The last pair finalizes the penultimate HOLD.

The strengthened FFG progress paragraph expressly requires the next direct-child target to use the newly justified checkpoint as its source; repeated links from an older source are correctly excluded as a finalization argument.

## Final visual verification

All 17 pages were rendered and inspected. After two diagram-label corrections, pages 5 and 7 were re-rendered and re-inspected against the final PDF hash above. The remaining 15 page renderings were verified unchanged.

| Pages | Result |
|---|---|
| 1–4 | Title, abstract, architecture table, assumptions, notation table, execution formulas, and acceptance/goal lists are readable and within margins. |
| 5 | Figure 1 is legible at document scale; the checkpoint box now says **up to `B_C` packets**. A3 remains provisional; A4 gates irreversible jobs. |
| 6–7 | BB timers and equations fit. Figure 2 explicitly labels `2δ` as **GOOD CASE**, distinguishes DECIDED export latency, and shows mandatory BA for early deciders. |
| 8–10 | Global fields, FFG formulas, signing constraints, liveness premise, health guard, and activation text fit without clipping. |
| 11 | Figure 3 is legible; HOLD links, pending ROTATE, CONFIRM finalization, and later proof consumption are visually distinct. Connectors do not cross text. |
| 12–13 | Publication job formulas, P duties, incentive table, and evidence domains render cleanly. |
| 14–15 | Proof statements, proof endings, latency/throughput equations, and section transitions render cleanly. |
| 16–17 | Worked traces, adversarial-case table, remaining choices, and references are readable; no unresolved reference markers or broken glyphs appear. |

No clipped text, overlapping elements, overflowing formulas/tables, unreadable figure labels, or connector/label collisions were found. The final three figure PDFs match their protocol roles and the two corrected labels were also updated in the editable `.drawio` sources by the diagram author.
