# Independent verification of the lightweight security patch

## Summary

**Final Verdict:** The three targeted repairs and their changed proof steps are valid under the manuscript's stated global delivery, persistent-signing, availability, capacity, and minimum-tenure premises. No Critical Error or Justification Gap was found in this patch review.

**List of Findings:** None.

Reviewed source: `formalization/main.tex`, SHA-256 `bb2e1d9c73fe89f524dfb803d01a3ace9f129fc7a408010e90584032ed1b69a6`.

Comparison baseline: `formalization/build/lightweight-security-patch/main.before.tex`, SHA-256 `1fc6cbfa619412b506d0d4f6d9d8b911e89609c8c03cdf1f858e9db148ed549e`.

Line references below refer to that reviewed source snapshot. Later line wrapping can change the line numbers without changing this verdict. This was a read-only manuscript review using the proof-verifier workflow; it is a verification of the targeted changes and their dependencies, not an independent certification of every unrelated quantitative claim in the manuscript.

## Detailed Verification Log

1. **Premises and theorem scope — lines 139–143, 188–220, 279–285, 1020–1027, 1084–1112.** “Progress additionally requires [the minimum-tenure inequality] … those bounds are environmental premises.” The revised setup, service specification, and liveness theorem all carry the needed capacity and tenure restrictions. A healthy eligible region contains more than two-thirds regional weight from the always-responsive global set; regional connectivity alone is expressly insufficient. The workload bound `Q`, complete-outcome admission, retrieval bounds, and sustained-capacity premises are assumed, not derived from maximum report size. No strengthened assumption was silently supplied in this review.

2. **No-certificate regional interruption — lines 497–504, 654–663, 681–687, 728–742.** “The region cannot resume that generation merely by restoring connectivity.” This accurately retains the serial predecessor-certificate barrier. READY now concerns starting the next generation from the complete adopted parent prefix; it does not require the failed instance's outcome. When no different region is ready and the no-progress guard fires, a ready current region can be selected. Thus the one-region case and the case with only the current region ready no longer wait forever for a missing DECIDED certificate. Previously produced unadopted outcomes can postpone this guard; the retained tail term at lines 1171–1183 explicitly accounts for that delay.

3. **Selection fairness — lines 128–129, 728–733, 1036–1048.** “Distinct-region selections advance along the cyclic list and cannot skip a continuously ready healthy region.” This follows from the first-ready selection rule and the list containing each region once. If the healthy region differs from the current one, every non-target selection lies strictly before it on that cyclic traversal, so the remaining cyclic distance decreases. If the healthy region is initially current and another region is chosen, this argument applies from that next selection. If no different region is ready, same-region recovery applies. Changing readiness of other regions cannot bypass a continuously ready target. The argument does require the healthy region to remain available, as the revised theorem states.

4. **Fresh activation and safety — lines 318–326, 359–364, 774–784, 928–942, 960–970.** “The new generation separates all regional signing domains.” Same-region selection creates generation `a+1`, a new activation-derived `R_0`, and new instance identifiers. The old sign-once records therefore neither block the new instances nor authorize reuse of an old instance. The finalized ROTATE/CONFIRM path, fixed start boundary, and common retained prefix remain unchanged. Old-generation certificates cannot extend the new effective regional cursor. Quorum intersection and persistent sign-once behavior still exclude conflicting exported regional histories even if the interrupted regional broadcast itself lost synchrony.

5. **Catch-up preview truncation — lines 582–603, 976–1000.** “Process every not-yet-applied finalized activation … in increasing generation order.” The receiver now applies the earliest intervening rollback before accepting data under a later activation. For example, an old preview through position 110 is first cut to 100 by an intermediate activation even if a later activation retains position 120. The later numerical tip cannot preserve the old 101–110 suffix. Missing ancestry blocks authority advancement; queued activations wait for their fixed boundaries; replayed older proofs cannot roll authority backward. Keeping the delivered log while truncating only previews preserves append-only delivery, including when several activations are processed during one catch-up.

6. **Tenure and recovery accounting — lines 1030–1054, 1084–1112, 1151–1196.** “The minimum-tenure condition allows a healthy successor to finish local recovery and report a new batch.” The conservative local-start bound allows a complete traversal of planned leader terms and an additional instance spacing. Synchronization then includes discovery, complete retrieval, and bounded contiguous-backlog adoption. The strict minimum-tenure inequality adds report-alignment/adoption margin before either geographic guard can repeatedly preempt healthy startup. Same-region recovery pays the existing detection, READY, ROTATE/CONFIRM, activation, and successor-output costs. It adds a branch to deterministic selection, not a new protocol message or extra normal-service round.

### Scope limits retained by the patch

- This does not establish recovery after a loss of the global synchronous envelope: global authenticated broadcast and the always-responsive global quorum remain premises.
- This does not promise that an unadopted regional preview survives a generation change.
- No finite recovery bound follows without finite tail/retrieval/backlog bounds and an eventual healthy eligible region. The manuscript already states these limits.

No manuscript edits were made by this verifier. Compilation and visual layout are outside this report.
