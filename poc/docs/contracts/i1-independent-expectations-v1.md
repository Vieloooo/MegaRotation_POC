# Independent I1 expectations v1

Prepared by bft_verifier from v2/v10/pinned Commonware before inspecting any implementation. Lead transcribed these expectations on 2026-11-15. No acceptance checks were executed at preparation.

1. Historical sample at 1,000,000, delivery at 1,010,000 and offsets ±1000 yields 1,011,000 / 1,009,000; latency affects readiness only.
2. Reject stale/duplicate sequences, wrong run/domain/incarnation, unknown/consumed request, future as-of and expired response without changing calibration. Invalid seq100 cannot poison subsequent valid seq3.
3. Lease end 1,100,000: signing is permitted only strictly before it; repeated grants cannot extend lease and signing cannot wait for queued ClockUnsafe to enforce expiry.
4. Local timer deadline 1,050,000 fires at reference 1,049,000 / 1,051,000. Renewal/restart retain deadline; canceled generations never fire.
5. Work completing after Close/expiry/stop cannot sign. Exercise both equal-time orderings.
6. Restore signing history, terminal state, first-receive time and timer generations; repeated Enter preserves state. Fresh calibration never revives terminal cid.
7. Crash after sync before Ack replays exact cached logical output once via authorized recovery. Altered/unsolicited duplicates fail.
8. Send admission differs from delivery. Test FIFO/cross-channel order, offline/overflow and stable identities.
9. Same seed/config gives identical complete canonical virtual traces across reversed process starts and injected wall pauses. Exclude only PID/port/wall diagnostics.
10. Independent mutation checks reject fabricated lease renewal, missing sync-before-send, altered duplicate and output outside a grant.

Upstream evidence: runtime lib.rs:525–539 synchronous Clock/governor; deterministic.rs:9–18,367–415 external future nondeterminism and self-advance; runtime lib.rs:905–912,993–1000 actual completed sync; Simplex voter actor.rs:365–368,974–986 persistence before publication; p2p lib.rs:125–179 submission/offline semantics. Commonware source revision 92a2f68a0e131c8b69de41e84aec37da6280f212, release 2026.10.0.

This finite fixture gate does not verify BFT quorum/finality, cryptography, complete Commonware runtime conformance or real-network clock synchronization. See full I1 contract for I1b obligation.
