"""Bounded semantic model of cap-two signed-chain broadcast.

This does not model signatures or wire parsing. Byzantine-only chains may
introduce arbitrary values through round b. A chain containing a responsive
honest signer cannot introduce a value which that signer did not multicast.
The protocol's complete-object delivery premise delivers each honest multicast
by the following deadline. These are assumptions of the model, not tested facts.
"""
from itertools import product
import json
from pathlib import Path


def check(k=4, b=2, h=2):
    n = b + h
    subsets = [
        {v for v in range(k) if mask >> v & 1} for mask in range(1 << k)
    ]
    cases = 0
    for injections in product(subsets, repeat=b * h):
        retained = [set() for _ in range(h)]
        forwarded = [set() for _ in range(h)]
        pending = [set() for _ in range(h)]
        for rnd in range(1, n + 1):
            incoming = [
                pending[p]
                | (injections[(rnd - 1) * h + p] if rnd <= b else set())
                for p in range(h)
            ]
            pending = [set() for _ in range(h)]
            for p in range(h):
                for value in sorted(incoming[p]):
                    if value not in retained[p] and len(retained[p]) < 2:
                        retained[p].add(value)
                    if (
                        value in retained[p]
                        and value not in forwarded[p]
                        and rnd < n
                    ):
                        forwarded[p].add(value)
                        for q in range(h):
                            pending[q].add(value)
        outputs = [next(iter(s)) if len(s) == 1 else None for s in retained]
        assert len(set(outputs)) == 1, (injections, retained, outputs)
        cases += 1
    print(
        f"PASS: {cases} exhaustive abstract cap-two schedules; n={n}, "
        f"Byzantine-only chain injection rounds={b}, responsive honest={h}, "
        f"semantic values={k}."
    )
    print("Bounded sanity check only; not a proof or implementation test.")
    return {
        "n": n,
        "byzantine_identities": b,
        "responsive_honest_identities": h,
        "semantic_values": k,
        "exhaustive_schedules": cases,
        "disagreements": 0,
    }


if __name__ == "__main__":
    results = [
        check(k=3, b=1, h=3),
        check(k=3, b=2, h=2),
        check(k=4, b=2, h=2),
        check(k=3, b=3, h=2),
    ]
    artifact = {
        "scope": (
            "Exhaustive bounded semantic forwarding/retention abstraction; "
            "complete verified honest messages arrive by the next deadline. "
            "Signatures, parsers, resource admission and clocks are assumed. "
            "No proof of general correctness or implementation behavior."
        ),
        "results": results,
    }
    Path(__file__).with_name("safety_ab_results.json").write_text(
        json.dumps(artifact, indent=2) + "\n"
    )
