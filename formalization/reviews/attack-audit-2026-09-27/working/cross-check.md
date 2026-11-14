# Independent cross-check of liveness traces and report-body amplification

Reviewed `working/liveness_traces.py` against the frozen protocol source. No manuscript change was made. All four executable checks passed. This verifies the displayed traces and arithmetic, not a general implementation or liveness theorem.

## Trace script

| Trace | Verdict | Necessary interpretation |
|---|---|---|
| `delayed_forward` | Valid abstract permanent-stall trace. | Exactly the H0 multicast copies to H1,H2 miss the length-2 deadline. A discarded late packet is equivalent to the model's omitted delivery because the real algorithm rejects it after that deadline. X/Y differ through the report-vector commitment even if B's x contains no extension. Honest weights 1/4 and 1/2 cannot certify with B withholding. The initial missed deadline violates the continuous global envelope; subsequent synchrony does not release the one-vote-per-epoch locks. |
| `depth_one` | Valid fully healthy backpressure trace, not a refutation of eventual height growth. | With d_R=b_G=b_R=1, a boundary-produced batch misses that boundary's preceding snapshot. It is reported next epoch and installed one epoch after that, so new batches occur at t=0,14,28,42 for T_G=7. All remaining producers find a full one-edge path. N_G=5 causes no other-region rotation in the one-region setup, and growth resets z before K_G=5; no omitted restart disturbs the trace. |
| `identity_schedule` | Valid allowed installed-configuration trace. | 100 Byzantine identities share total weight 1/5 and appear before the single honest identity of weight 4/5. Each term lasts 100×0.03=3 seconds, giving 300 seconds of silence. T_G=10.5, K_G=N_G=40 leave 420 seconds before timer pressure, greater than L_i+3T_G=334.54. Thus an earlier restart does not defeat the construction. This is exploitation of the installed identity/weight mismatch, not permission for an attacker to create unregistered identities. |
| `region_outage` | Correct quorum arithmetic. | Losing the all-honest active 40%-weight region leaves only 60% globally responsive, insufficient for a >2/3 certificate even if the surviving region has a 100% regional READY quorum. This violates the global responsive-quorum premise. |

The depth trace's first emitted mini-block can be delivered only after its adopting checkpoint obtains the next finalizing link; this is later than its installation. The script correctly claims production times rather than delivery times. The weighted trace intentionally demonstrates a long finite wait, not infinite deadlock.

## Report-body amplification: valid with two qualifications

The proposed construction is feasible. Two qualifications avoid overstating it:

1. Write **B≤B_G for an attainable legal report-body size**, or explicitly choose legal parameters permitting B=Θ(B_G). B_G is a configured maximum, not a promise that a valid branch can fill it: the b_G batch-count bound and per-batch B_R bound can bind first. A one-batch construction already suffices with B=Θ(B_R) when it fits the report.
2. The schedule must be admitted by the configured direct-Byzantine-input quotas. The document explicitly gives the two-private-reports-per-honest-receiver example at 1112–1116, but reserving prescribed honest traffic alone does not require admitting two direct Byzantine offers under every possible quota policy. After the seed pairs are admitted and retained, their honest forwarding is prescribed traffic and must be reserved (1192–1202). Describe this as a concrete allowed admission scenario, not a lower bound for every possible admission policy.

### Complete valid branch construction

Let the active region contain f≥1 Byzantine report origins and H responsive honest global receivers. Choose an epoch e whose preceding completed slots include a slot u assigned to some Byzantine producer B*, later than the effective anchor's last slot. The simple initial configuration can put B* first: epoch 1's snapshot is empty, C_1 retains genesis, and B*'s slot 1 is completed before epoch 2's report cutoff.

For every tuple (origin o, honest receiver p, a∈{1,2}), B* constructs and signs a different complete one-batch fork X_(o,p,a), all rooted at the same effective adopted tip and labeled with the same authorized slot u. Use distinct transaction-list bodies throughout, so forks for different receivers **and different report origins** have no reusable body bytes apart from common negligible metadata. All headers have the correct configuration, view, local view, u, within-batch indices, consecutive heights and parent hashes. Every envelope and ORDER signature is produced by B*, its actual configured slot authority.

Each Byzantine origin o signs a report carrying its assigned X_(o,p,a), the correct parent/epoch, exact endpoint, and a legal READY value. The slot is no later than the epoch-start cutoff. Keep the report within b_G and B_G; a one-batch report respects b_G≥1. The witness is available in full. The origin is in the active region, so a nonempty report is permitted.

The resulting branches are equivocations, but remain *valid*: the model expressly permits Byzantine valid-parent choices (291–296), ValidReport can endorse any valid branch (383–386), and global validation is independent of the local two-candidate cap (315–318, 387–388). Detecting the common producer's PROPOSE/ORDER equivocation does not remove its configured authority or retroactively reject certified state (665–677). Thus the local cap cannot invalidate the construction. No honest producer signature must be forged.

If B=Θ(B_G) needs multiple batches, choose sufficiently many completed slots assigned to B* or cooperating Byzantine producers, and build each fork using a strictly increasing sequence of those slots. Honest slots may be skipped because absent slots need no GAP proof. This extended construction requires the relevant completed slots to exist after the effective anchor; the one-batch example avoids that additional precondition.

### Schedule and byte accounting

In round 1, each origin o privately gives receiver p its two assigned values. At its deadline every p retains and forwards those two values. All initially retained pairs are disjoint. Thus each origin has 2H distinct semantic values/body sets honestly forwarded in round 2. Every honest receiver obtains the other 2(H−1) values; A2 requires complete chain/witness verification before A3 ignores values beyond its cap-two set (1192–1202). Local speculative caps do not skip ValidReport.

For each origin:

- Distinct report-body union: **2HB** bytes, excluding common metadata and negligible overhead.
- Initial Byzantine-to-honest copies: **2HB** bytes.
- Copies from honest sources to the other honest receivers: **2H(H−1)B** bytes. Each body has a single initial honest holder, so ordinary per-endpoint digest caching cannot suppress these first acquisitions.
- Total honest-endpoint body acquisitions: **2H²B** bytes. Delivery to Byzantine recipients and physical multi-hop transmission can add cost; neither is needed for this lower-bound schedule.

With f origins using disjoint fork bodies, the endpoint cost is Θ(fH²B), reaching Θ(N³B) when both f and H scale with N under suitable allowed weights. Every honest output for each of these origins is bottom because every set has two values. This traffic contributes no adopted extension from the Byzantine origins.

**Retention claim to make conservatively:** each receiver's initially retained pair remains stored even though the origin's final output is bottom (187–189, 1115–1116), so aggregate persistent storage across honest receivers is at least **2fHB** bytes of distinct discarded fork data. The full 2fH²B acquisition volume should not automatically be called a permanent-storage lower bound: whether every additional fully verified but AB-cap-ignored witness is “accepted” for indefinite retention is less explicit than the retention of the initial pairs. This ambiguity does not affect the transfer/verification amplification.

### Correct security classification

This is bounded, valid, authenticated adversarial work; unlimited wire flooding is unnecessary. It refutes provisioning based only on the final N-report vector or on two report bodies per origin system-wide. The manuscript already recognizes the larger honest-forwarding/admission budgets at 1075–1109. With all those complete-object budgets actually satisfied, the construction does **not** refute the conditional liveness theorem. On a finite deployment provisioned only for the smaller estimate, it can exceed bandwidth/verification capacity, miss a global deadline, and become a precursor to the delayed-forward permanent stall. State that provisioning premise explicitly rather than claiming an unconditional in-model deadlock.
