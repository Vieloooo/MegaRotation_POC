# Fresh native-engine verification

Independent candidate acceptance: `6686a009e0eeea261026dac807742efed76b17c6`, within the frozen Fresh profile only. The implementation runs two actual native worker processes; the other three participants are explicitly scripted signers. Quorum is four, never a two-worker quorum.

The independent suite passed111 Rust tests (106 prior plus5 new), checked21 actual native runs, and confirmed six complete-evidence seed/wall-pause pairs. It checked14,655 trace rows,3,755 Acks,112 underlying signing creations and93 cache hits. Eight materialized semantic mutants with recomputed hashes/receipts were rejected. Eight actual transport controls and three real-worker completion-boundary probes passed. The earlier duplicate-Ack test failure was an oracle event-label error, resolved against the raw duplicate frames and effect counts; the failed report remains preserved.

See `poc/verification/native-engine-fresh/REVIEW-6686a00.md` and `candidate-6686a00-report.json` for exact commands, code/binary hashes, artifacts and coverage. Original run inputs and public evidence copies are under `poc/artifacts/verification/native-engine-fresh/`. Generated private fixture launch files remain task-local and are excluded from public copies.

Formal66-run replay follows on the integrated revision. These runs produced VOTE/CERTIFICATE/BODY traffic, but no RESOLVER packets; resolver behavior is source-reviewed, not an executed repair/liveness result. Same-cid recovery with auxiliary callbacks/timing, v10 validity/windows, 15 nodes and handoffs remain pending. Full I1b/I1 has not passed.
