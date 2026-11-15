# Contract review — Publisher/Espresso v1

2026-11-15; named role `bft_verifier`; source expectations were derived before contract review and before any candidate code. Reviewed `poc/docs/contracts/task-contract-publisher-espresso-v1.md`, version `publisher-espresso-component-v1`, against the source mapping in [expectations-v1.md](expectations-v1.md).

**PASS for freezing this independent component contract. No implementation has been verified.** No blocking divergence from the cited v10/v2 rules was found.

The contract preserves proposal-time/live-conditioned E-TS, legal equal/frozen keys, invalid regressing sources, byte-prefix completeness separate from semantic result material, independently delivered readers, validated keyed Publisher tasks, and actual file-backed recovery. It expressly leaves 02/03/04 integration and 07/16 ownership open.

The three local choices raised during independent preparation are explicitly resolved:

1. `RecordedAt` separates the publication position from the complete prefix actually used. A proof at q1 with body at q2 cannot masquerade as complete public evidence through q1. Earliest GFR q_avail is not computed by 06.
2. STOP never erases a verified historical receipt. A valid late receipt may move STOPPED to RECORDED while keeping stop_reason and permanently forbidding new attempts. This preserves PUB-2's receipt/phase invariant and PUB-3 persistence.
3. SessionAssigned excludes a competing cid only within the relevant session-selection scope. Production scope proof remains the trusted 07 adapter's responsibility; a generic other-cid event cannot stop every task.

The lead confirmed three freeze clarifications, now included in expectations: duplicate finalize is pure idempotent replay and cannot rewrite finalization time or advance the tick frontier; receive timestamps must be at/after finalization (message-before-header buffering defers that check until its header exists); a VERIFYING or STOPPED task with no complete verified Offer cannot become RECORDED solely from a receipt. The last transition must preserve PUB-1 by retaining or first establishing full Offer validation. Tests must preserve source/clock distinctions.

`PREPARED` before release, exact-ID replay, directory sync, and poison-on-unknown-commit are A engineering requirements; they are not claims that this component already implements 01/02 root custody. Receipt and Offer fixtures exercise the same traits but carry no cryptographic acceptance claim. The strongest current conclusion is **contract ready for lead freeze**, subject to the named candidate and independent execution afterward.
