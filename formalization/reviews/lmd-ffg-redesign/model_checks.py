"""Finite adversarial checks for the abstract rules, not a consensus proof.

Run from any directory with Python 3. The manuscript is authoritative. These
checks enumerate conflicting report trees and exercise state-transition edges
that previously depended on per-batch agreement.
"""

from itertools import permutations, product


def head(paths, weights):
    """Batch IDs are canonical tie-break IDs; () is the common anchor."""
    cursor = ()
    while True:
        children = {p[: len(cursor) + 1] for p in paths
                    if len(p) > len(cursor) and p[: len(cursor)] == cursor}
        if not children:
            return cursor
        cursor = max(children, key=lambda c: (
            sum(w for p, w in zip(paths, weights) if p[: len(c)] == c), c[-1]
        ))


def snapshot_checks():
    # Competing branches, shared prefixes, truncation at internal vertices,
    # an empty origin, and unequal voting weights.
    options = [(), (1,), (1, 2), (1, 3), (4,), (4, 5)]
    weights = (1, 2, 3, 1)
    cases = 0
    for paths in product(options, repeat=4):
        selected = head(paths, weights)
        assert (selected != ()) == any(paths)
        assert selected in paths  # selected leaf was an admitted report tip
        for order in permutations(range(4)):
            assert head([paths[i] for i in order],
                        [weights[i] for i in order]) == selected
        for budget in (1, 2):
            truncated = [p[:budget] for p in paths]
            assert head(truncated, weights) in truncated
        cases += 1
    return cases


def latest_message_checks():
    latest = -1
    tips = set()

    def receive(slot, tip, now):
        nonlocal latest, tips
        if slot > now or slot < latest:
            return
        if slot > latest:
            latest, tips = slot, {tip}
        else:
            tips.add(tip)

    receive(7, "x", 7)
    receive(7, "y", 7)
    assert len(tips) == 2  # equivocation: zero weight
    receive(6, "old", 7)
    assert latest == 7 and tips == {"x", "y"}
    receive(100, "future", 7)
    assert latest == 7
    receive(8, "unavailable", 8)
    assert latest == 8 and tips == {"unavailable"}
    # Missing data makes the current vote unusable; it does not restore slot 6.
    receive(6, "old", 8)
    assert tips == {"unavailable"}
    return 6


def schedule_and_cutoff_checks():
    leaders = [((u - 1) // 3) % 4 for u in range(1, 25)]
    assert leaders[:12] == [0] * 3 + [1] * 3 + [2] * 3 + [3] * 3
    # Silence does not alter this schedule or require a GAP record.
    assert leaders[3] == 1 and leaders[12] == 0
    # At an epoch boundary, a newly released current-slot candidate must not
    # enter the just-ended snapshot, even if it wins the unrestricted head.
    paths = [(1,), (2,)]
    last_slot = {1: 9, 2: 10}
    assert head(paths, (0, 1)) == (2,)
    ended_slot = 9
    filtered = [p for p in paths if all(last_slot[b] <= ended_slot for b in p)]
    assert head(filtered, (0,)) == (1,)
    # Installing one globally adopted child removes the incompatible branch.
    anchor = (1,)
    old_paths = [(1, 2), (1, 3), (4, 5)]
    rooted = [p[len(anchor):] for p in old_paths if p[:len(anchor)] == anchor]
    assert rooted == [(2,), (3,)]
    return 5


def equivocating_body_check():
    # One origin offers a disjoint pair to each honest receiver. Each pair is
    # within that receiver's cap, but the honestly forwarded union is larger
    # than the final report vector. Report bodies are each one unit here.
    n = 4
    retained = [{f"report-{i}-a", f"report-{i}-b"} for i in range(3)]
    forwarded_union = set().union(*retained)
    assert len(forwarded_union) == 6 > n
    assert all(len(pair) <= 2 for pair in retained)
    return 1


if __name__ == "__main__":
    count = snapshot_checks()
    edges = (latest_message_checks() + schedule_and_cutoff_checks()
             + equivocating_body_check())
    print(f"PASS: {count} conflicting report vectors, each checked in 24 origin orders")
    print(f"PASS: {edges} latest-message, clock, cutoff, re-anchor and equivocation checks")
    print("Scope: finite abstract checks; not a machine-checked proof of the protocol.")
