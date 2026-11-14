# Liveness attack audit — 27 September 2026

Read the [liveness and live-stream stall review](/Users/jin/projects/megaeth_sequenser_rotation/formalization/reviews/attack-audit-2026-09-27/attack-review.md).

The audit is analysis only. It does not modify the proposed protocol.

The main report formalizes seven findings and distinguishes:

- permanent deadlocks after a stated availability/timing assumption fails;
- long production pauses allowed by valid configurations;
- infinite starvation when the required tenure inequality is violated;
- resource attacks whose effect depends on real provisioning;
- attempted attacks blocked by the specified protocol.

The frozen source and two included figure files are retained in `source/`.
Their hashes and the final verification record are in `validation.md`.
Independent reviews, executable abstractions, and their results are in `working/`.

The executable checks use only Python's standard library:

```sh
python3 formalization/reviews/attack-audit-2026-09-27/working/liveness_traces.py
python3 formalization/reviews/attack-audit-2026-09-27/working/safety_ab_model.py
```

These are bounded rule-level checks, not a tested implementation of the full protocol.
