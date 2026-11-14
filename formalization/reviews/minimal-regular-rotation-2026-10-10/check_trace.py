"""Replay the decisive timer/entry trace, not an exhaustive protocol model.

Times use exact fractions. The checks cover correct voting eligibility,
normal versus recovery entry, timer cancellation, quorum sizes, and the
displayed synchronous delivery bounds. Byzantine packets are scheduled
explicitly. The recovery continuation uses the recorded highest QC.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from fractions import Fraction as F
import heapq
import json
from pathlib import Path


def time(s):
    return F(str(s))


@dataclass
class Replica:
    name: str
    view: int = 0
    aborted: set = field(default_factory=set)
    equivocation: set = field(default_factory=set)
    pending: int | None = None
    known_qcs: set = field(default_factory=lambda: {(0, 0, "G")})
    voted: set = field(default_factory=set)
    timers: dict = field(default_factory=dict)
    committed: dict = field(default_factory=dict)
    entries: dict = field(default_factory=dict)


def replay(profile, qc_threshold, extra_fast=(), require_qc=False):
    fast = ("P",) + tuple(extra_fast)
    honest = fast + ("Caro", "D")
    faulty = {"Alice", "Bob"}
    nodes = {n: Replica(n) for n in honest}
    # Valid initial-entry skew: exactly one Delta from earliest to latest.
    initial = {n: time(-5) for n in fast}
    initial.update(Caro=time(-6), D=time(-5.2))
    assert max(initial.values()) - min(initial.values()) <= 1
    for n, t in initial.items():
        nodes[n].entries[0] = t
    queue, serial, votes, log, cancelled = [], 0, {}, [], []
    heights = {"X": 1, "Y": 2, "Z": 2, "W": 3}

    def schedule(t, kind, who, v=None, block=None):
        nonlocal serial
        serial += 1
        heapq.heappush(queue, (time(t), serial, kind, who, v, block))

    def quit_view(r, v, now):
        if v not in r.aborted:
            r.aborted.add(v)
            log.append([float(now), r.name, "quit", v])
        if r.view == v:
            schedule(now + 1, "enter_recovery", r.name, v + 1)

    def enter(r, v, now):
        if v <= r.view:
            return
        assert v == r.view + 1
        r.view = v
        r.entries[v] = now
        r.pending = None
        log.append([float(now), r.name, "enter", v])
        if v in r.equivocation:
            quit_view(r, v, now)

    def vote(r, v, b, now):
        if r.view != v or v in r.aborted or v in r.equivocation:
            return
        h = heights[b]
        if h in r.committed and r.committed[h] != b:
            return  # Even a local committed-prefix check does not stop the fork.
        assert (v, h) not in r.voted
        r.voted.add((v, h))
        votes.setdefault(b, set()).add(r.name)
        r.timers[(v, b)] = now + 2
        schedule(now + 2, "timer", r.name, v, b)
        log.append([float(now), r.name, "vote", b])

    # A late Alice terminal proposal overlaps a legitimate old-view timeout.
    schedule(0, "blame", "Caro", 0)
    schedule(.8, "blame", "D", 0)
    schedule(.2, "quit", "Caro", 0)
    schedule(1.0, "quit", "D", 0)
    for n in fast:
        schedule(.05, "proposal", n, 0, "X")
        schedule(.06, "terminal_qc", n, 0, "X")
        schedule(1.2, "quit", n, 0)
    for n in ("Caro", "D"):
        schedule(1.05, "proposal", n, 0, "X")
        schedule(1.06, "terminal_qc", n, 0, "X")

    # Bob gives P a fresh proposal; slow entrants receive it while closing Alice.
    schedule(.10, "proposal", "P", 1, "Y")
    for n in honest:
        if n != "P":
            schedule(1.10, "proposal", n, 1, "Y")
    schedule(1.15, "equivocation", "Caro", 1)
    schedule(1.20, "equivocation", "D", 1)
    for n in fast:
        schedule(2.15, "equivocation", n, 1)

    # Caro gathers STATUS, re-certifies X, and honestly extends it by Z != Y.
    schedule(4.20, "select_anchor", "Caro", 2, "X")
    for n in honest:
        schedule(4.24, "anchor_qc", n, 2, "X")
        schedule(4.24 if n == "Caro" else 4.30, "proposal", n, 2, "Z")
        schedule(4.40, "terminal_qc", n, 2, "Z")
    # Complete the next honest leader's immediate work on the Z branch too.
    for n in honest:
        schedule(4.40 if n == "D" else 4.45, "proposal", n, 3, "W")
        schedule(4.60, "terminal_qc", n, 3, "W")

    while queue:
        now, _, kind, who, v, b = heapq.heappop(queue)
        if now > time(6.30):
            break
        r = nodes[who]
        if kind == "blame":
            assert r.view == v and not any(wv == v for wv, _ in r.voted)
            assert now == r.entries[v] + 6
            assert r.pending is None
            r.pending = v
        elif kind == "quit":
            quit_view(r, v, now)
        elif kind == "enter_recovery":
            enter(r, v, now)
        elif kind == "equivocation":
            r.equivocation.add(v)
            # Evidence is relayed immediately; a future view is aborted on entry.
            if r.view >= v:
                quit_view(r, v, now)
        elif kind == "proposal":
            vote(r, v, b, now)
        elif kind == "terminal_qc":
            signers = votes.get(b, set()) | faulty
            assert len(signers) >= qc_threshold, (profile, b, signers)
            assert b != "Y"
            h = heights[b]
            r.known_qcs.add((v, h, b))
            if h in r.committed and r.committed[h] != b:
                continue
            # This is the checked composition choice: finish a pending raw exit wait.
            if r.view == v and v not in r.aborted:
                enter(r, v + 1, now)
        elif kind == "timer":
            if v in r.aborted or v in r.equivocation:
                cancelled.append([who, b, float(now)])
                continue
            if require_qc and (v, heights[b], b) not in r.known_qcs:
                continue
            assert heights[b] not in r.committed or r.committed[heights[b]] == b
            r.committed[heights[b]] = b
            log.append([float(now), who, "commit", b])
        elif kind == "select_anchor":
            assert who == "Caro" and r.entries[2] == time(2.2)
            assert now == r.entries[2] + 2
            # Every correct STATUS can arrive by 4.15, before this selection.
            assert max(n.entries[2] + 1 for n in nodes.values()) <= now
            highest = max(q for n in nodes.values() for q in n.known_qcs if q[0] <= 1)
            assert highest == (0, 1, "X")
            assert len(votes["Y"]) < qc_threshold
        elif kind == "anchor_qc":
            r.known_qcs.add((2, 1, "X"))
        else:
            raise AssertionError(kind)

    # The public blame proof comprises Alice, Bob and Caro, for Alice's view.
    assert len(faulty | {"Caro"}) == 3
    assert len(faulty | set(fast)) == qc_threshold
    assert votes["Y"] == set(fast)
    assert len(votes["Y"]) < qc_threshold
    assert nodes["P"].entries[1] == time(.06)
    assert nodes["Caro"].entries[1] == time(1.2)
    assert nodes["D"].entries[1] == time(2)
    outcomes = {n: r.committed.get(2) for n, r in nodes.items()}
    violation = len({x for x in outcomes.values() if x is not None}) > 1
    assert violation is (not require_qc)
    if not require_qc:
        assert [2.1, "P", "commit", "Y"] in log
        assert [6.24, "Caro", "commit", "Z"] in log
    return {"profile": profile, "qc_threshold": qc_threshold,
            "qc_gate": require_qc, "Y_honest_votes": sorted(votes["Y"]),
            "committed_at_height_2": outcomes, "safety_violation": violation,
            "events": log}


# Delays for the decisive correct-origin multicasts, including every slow edge.
deliveries = [(0, 1.0), (.2, 1.0), (.2, 1.2), (.05, 1.05), (.06, 1.06),
              (.10, 1.10), (1.15, 1.20), (1.15, 2.15), (3.0, 4.0),
              (3.15, 4.15), (4.2, 4.22), (4.22, 4.24), (4.24, 4.30)]
assert all(0 <= time(end) - time(start) <= 1 for start, end in deliveries)
results = [replay("raw n=5 f=2", 3),
           replay("strong quorum n=7 f=2", 5, ("E", "F")),
           replay("same strong-quorum trace with an extra QC commit gate", 5,
                  ("E", "F"), require_qc=True)]
out = Path(__file__).with_name("trace-results.json")
out.write_text(json.dumps({"method": "bounded trace replay; not exhaustive model checking",
                           "all_listed_correct_message_delays_at_most_delta": True,
                           "results": results}, indent=2) + "\n")
for r in results:
    print(r["profile"], "violation=" + str(r["safety_violation"]),
          r["committed_at_height_2"])
