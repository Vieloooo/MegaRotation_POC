# Final bounded complexity review

Target: `formalization/main_v3.tex`, 1,518 lines; observed SHA-256 `3b6d7add459896616f381de1a8bf98effa1d83424bd79af9efdcf685894bbed4`. Date: 2026-09-28. Read-only review of §8 and its regional startup, terminal-drain and handoff dependencies.

## Summary

**Final verdict:** the timing and communication derivations reviewed here are valid under their stated good-case and completion assumptions. The required issue in `complexity-review-v1.md` is resolved. No remaining critical error or justification gap was identified within this bounded complexity review.

The new four-case communication table is consistent with its preceding definitions and with the protocol's explicit signature-list, direct-transfer and all-to-all bootstrap choices. This verdict does not certify the complete protocol, safe-cut implementation, committee observation policy or a concrete proof system.

## Resolution of prior findings

1. **Successor waiting — resolved.** `W_ij` is now explicit in normal boundary interval `H_ij`, accident latency `T_ij^acc`, and the final-receipt-to-first-new-receipt endpoint. Its zero case is defined by an already eligible successor. The homogeneous ideal throughput example explicitly assumes `W_ij=0`; waiting branches no longer inherit an unsupported fixed delay.

2. **Terminal-drain traffic — resolved.** The term-level message and bit formulas count `d_ctrl` physical DRAIN blocks. They are expressly restricted to fault-free terms without discarded or repeated APP proposals; failed attempts, failed views and retrieval are charged separately. No constant bound on `d_ctrl` is asserted. Q4 closes the physical engine after the terminal receipt is verified, retains its prefix and fetch service, and does not generate additional DRAIN blocks merely while successor eligibility is pending. The complexity text matches this rule.

3. **Uncached registry evidence — resolved.** Terminal registry-proof acquisition is explicitly outside the cached steady-state path, charged separately or to terminal readiness/drain delay when on that measured path. No per-normal-handoff fresh committee certificate or L1 finality wait is introduced by this accounting.

4. **Discarded APP attempts and notation — resolved.** The newly added term formula no longer treats the final K application blocks as all application attempts under arbitrary failures. All control-block counts use `d_ctrl` rather than the application's quota variable `d`.

## Check of the new four-case cost table

- **Ideal application block:** `O(n_i^2)` messages and `O(n_i^2 ell+n_i^3 kappa)` bits correctly reflect all-to-all forwarding/votes with explicit parent certificates. External subscribers and missing data retain their stated separate charges.
- **One local replacement:** `O(n_i^2)` additional control messages and `O(n_i^3 kappa)` control bits correctly include quit proof propagation, statuses, NEWVIEW and anchor re-certification. State retrieval, failed replacement attempts and new APP production are separate.
- **Regular cross-region handoff:** `M_H=O(n_j^2)` includes the `O(n_j)` direct outer transfer plus successor's quadratic bootstrap. `B_H=O(n_j(a_ij+n_i kappa+s_L1)+n_j^3 kappa)` includes the assembled old receipt, missing anchor/history and registry evidence, plus unaggregated successor-bootstrap control traffic. This assumes the stated reachable holder and counts retries separately.
- **Accident handoff:** `M_H+M_U+M_rec` and `B_H+B_U+B_rec` correctly add one status update and extra cut/fencing/recovery work to the already counted transfer/bootstrap. `M_U=M_C+O(N)+1` and `B_U=B_C+O(s_E+N(s_E+s_L1))` distinguish committee-internal work, external dissemination and registry transaction submission. The `1` is expressly not Ethereum consensus traffic. Additional installed status entries continue to incur the separately stated per-update registry cost; no such update is made free by the table.
- **Units and exclusions:** the table caption distinguishes the first per-APP row from additional per-transition rows and separately charges terminal DRAIN blocks, failed attempts and retrieval. This avoids comparing a steady batch's cost with an entire recovered term's cost without stating the measurement unit.

## Rechecked latency boundaries

- `C_i=delta_R+2Delta_R` is regional commitment from the ready proposal transmission; `F_i=C_i+delta_R` is conservative collector receipt availability; consumer delivery remains an additional path.
- `I_i=2delta_R` measures ideal proposal spacing and does not replace finality time.
- `T_i^view=4Delta_R+2delta_R` begins at the first correct quit, assumes an honest next leader and available state, and excludes failure detection and additional failed leaders.
- `A_j<=2Delta_R+2delta_R` includes the retained initial NEWVIEW path after common validated-anchor readiness, with an honest initial leader. There is no silent zero-cost startup assumption.
- Normal proposal gap `H_ij` excludes the new batch's own `F_j`; first-new receipt adds it once. `((K-1)I+H)/K` therefore has the correct repeated-term endpoints.
- Accident timing remains a conditional phase accounting, not a guaranteed numerical Ethereum recovery duration. Safe-cut, data, committee and L1 progress remain explicit prerequisites.

No manuscript files were changed by this review. This final check concerns derivations and accounting; the root editor performs full-document compilation and presentation inspection.
