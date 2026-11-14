# Skeptical final review of the synthesized liveness report

Reviewed `attack-review.md` against the frozen `source/main.tex`. No protocol or main report edits were made by this reviewer.

## Verdict

The principal conclusions are correctly scoped. No substantive counterexample-invalidating defect was found in L1–L5. One timeline description in L3 needs a precision correction before delivery. The report does not overclaim a permanent stall under all stated assumptions.

## Must-fix precision issue

In the L3 table, the row for time **8–13** says “`C_2` is justified, but installation is fixed at the next boundary.” Epoch 2 begins at time 7 and its four-round broadcast ends at time 11. Thus `C_2` cannot yet be justified at times 8–10. It is constructed and justified **during** the stated interval; installation still waits until time 14. Suggested replacement: “`C_2` is constructed and justified during this interval; even after justification, installation waits until time 14.” This correction does not affect the claimed production pause or batch times.

## Claims checked

- **L1, minimal deadline miss:** Valid. `H_1` retains and forwards the Byzantine origin's value in round 1. The two late length-two copies are ignored at `H_2,H_3`; no honest node can extend those ignored chains, and `H_1` has already used its one forwarding opportunity. AB therefore returns one value versus bottom. The vector commitment makes the checkpoints different even when their adopted states agree. With weights 1 versus 2 and Byzantine abstention, neither target can reach the three-signature threshold. Persistent target-epoch votes prevent honest retargeting. No adjacent justified-parent continuation exists. The word “absorbing” is correctly qualified by continued Byzantine abstention: if the Byzantine signer voluntarily signs `Y`, that target could receive its third signature. The all-honest two-region partition variant independently establishes permanent split votes without this ongoing withholding choice.
- **L2, outage arithmetic:** Correct. `U` and `B\U` are disjoint, so their weights add without double counting. If unavailable plus withholding weight is at least `W/3`, the remaining possible voting weight is at most `2W/3`, which fails the strict threshold. A ready regional quorum does not supply global voting weight.
- **L3, healthy depth-one pause:** Correct after the table correction above. The first report precedes the batch, the next epoch reports it, and the next boundary installs it. The resulting batch times are `0,14,28,…`. Parameters satisfy regional timing, global timing, integer alignment, and tenure. With one region, the planned timer has no different successor; `z` alternates between 1 and 0, so the example is stable indefinitely.
- **L4, weighted identities:** Correct. `100×0.002W=0.2W`; the honest producer's `0.8W` satisfies both the regional honest premise and global certificate threshold by itself. The first 100 identity terms occupy `100×100×0.03=300` seconds. `L_*=(101×100+1)×0.03+0.01=303.04` seconds. `T_G/τ_R=350`, `T_G≥104Δ_G`, and `40T_G=420>303.04+3T_G` all hold. The first honest producer appears before the no-progress restart trigger. This is a permitted delay, not permanent liveness failure.
- **L5, short-tenure cyclic starvation:** Correct. A region activated in epoch `a` emits HOLD, ROTATE at `a+1`, CONFIRM at `a+2`, and activates its successor at `a+3`. Its six-second tenure never reaches the ten-second first Byzantine term's end. Honest weights `3+3=6>16/3` keep global certificates progressing; each honest regional weight is `3>8/3`, supplying READY. All named timing constraints hold except the explicitly violated tenure inequality. No accidental extra activation epoch is missing from the trace.
- **Summary and proof assessment:** Appropriate. The report distinguishes allowed healthy pauses, failures of availability/capacity premises, and the absence of an established all-assumptions permanent-stall counterexample. It also separates new production from adoption and finalized delivery. Describing L1 as an existential failure after a single timing event is justified; the text does not claim every missed deadline causes deadlock.

## Presentation note

Several inline formulas are currently literal parenthesized LaTeX, including commands such as `\operatorname` and escaped set braces. Standard inline math delimiters would make these easier to read in a Markdown renderer. This is not a correctness issue.
