"""Small, transparent counterexample traces for the liveness review.

These are abstractions of selected rules, not an implementation of the protocol.
The delayed-forward trace deliberately violates complete-object synchrony once.
The depth-cap and weighted-schedule traces meet the displayed model conditions.
Run with Python 3; no third-party dependencies or network access are required.
"""
from fractions import Fraction as F
import json
from pathlib import Path


def delayed_forward():
    # B is the Byzantine origin. Three responsive honest receivers H0,H1,H2.
    retained = [set() for _ in range(3)]
    forwarded = [set() for _ in range(3)]
    pending = [set() for _ in range(3)]
    events = []
    for rnd in range(1, 5):
        incoming = [set(s) for s in pending]
        pending = [set() for _ in range(3)]
        if rnd == 1:
            incoming[0].add("x")
        for p in range(3):
            for value in sorted(incoming[p]):
                if value not in retained[p] and len(retained[p]) < 2:
                    retained[p].add(value)
                if value in retained[p] and value not in forwarded[p] and rnd < 4:
                    forwarded[p].add(value)
                    for q in range(3):
                        if rnd == 1 and p == 0 and q in (1, 2):
                            events.append({"round": 2, "from": "H0", "to": f"H{q}",
                                           "result": "complete length-2 chain misses 2*Delta deadline; rejected"})
                        else:
                            pending[q].add(value)
    outputs = [next(iter(s)) if len(s) == 1 else "BOTTOM" for s in retained]
    targets = ["X" if v == "x" else "Y" for v in outputs]
    weights = {target: sum(F(1, 4) for t in targets if t == target)
               for target in set(targets)}
    assert outputs == ["x", "BOTTOM", "BOTTOM"]
    assert max(weights.values()) <= F(2, 3)
    # Every honest member has already persisted its one vote at epoch e.
    # B withholds. Repeating signatures cannot add distinct honest weight.
    assert all(sum(F(1, 4) for old in targets if old == target) <= F(2, 3)
               for target in ("X", "Y", "new-common-target"))
    return {"scope": "One transient complete-object deadline violation; outside continuous global envelope",
            "events": events, "byzantine_origin_outputs": outputs,
            "honest_FFG_targets": targets,
            "honest_target_weights": {k: str(v) for k, v in sorted(weights.items())},
            "quorum": "> 2/3", "certificate_possible_with_B_withholding": False,
            "after_healing": "No justified C_e; adjacent e+1 cannot start; same-epoch retargeting forbidden"}


def depth_one():
    # A single region with four honest equal-weight nodes.
    tau, dr, dg, tg, nr, kg, ng = F(1), F(1, 10), F(1), F(7), 1, 5, 5
    li = (4 * nr + 1) * tau + dr
    assert tau > 2 * dr and tg >= (4 + 3) * dg and (tg / tau).denominator == 1
    assert min(kg, ng) * tg > li + 3 * tg
    anchor = justified = 0
    candidate = None
    rows = []
    emitted_times = []
    for epoch in range(1, 9):
        anchor = justified
        if candidate is not None and candidate <= anchor:
            candidate = None
        snapshot = candidate  # Snapshot precedes production in boundary slot.
        emitted = None
        if candidate is None:  # d_R = b_G = b_R = 1.
            candidate = anchor + 1
            emitted = candidate
            emitted_times.append((epoch - 1) * int(tg))
        # All remaining slots of this epoch see a full eligible path.
        justified = snapshot if snapshot is not None else anchor
        rows.append({"epoch": epoch, "start_time": (epoch - 1) * int(tg),
                     "installed_height": anchor, "reported_tip": snapshot,
                     "new_batch_height": emitted, "target_adopted_height": justified})
    assert emitted_times == [0, 14, 28, 42]
    return {"scope": "Fully healthy allowed configuration; intentional backpressure, not theorem refutation",
            "parameters": {"N": 4, "regions": 1, "tau_R": 1, "Delta_R": 0.1,
                           "Delta_G": 1, "T_G": 7, "N_R": 1, "K_G": 5, "N_G": 5,
                           "d_R": 1, "b_G": 1, "b_R": 1},
            "tenure_lhs": str(min(kg, ng) * tg), "tenure_rhs": str(li + 3 * tg),
            "epochs": rows, "new_batch_times": emitted_times}


def identity_schedule():
    byzantine_count, honest_weight = 100, F(4, 5)
    each_byzantine_weight = F(1, 5) / byzantine_count
    n, nr, tau, dr, dg, tg = 101, 100, F(3, 100), F(1, 100), F(1, 10), F(21, 2)
    li, tenure = (n * nr + 1) * tau + dr, 40
    assert honest_weight > F(2, 3)
    assert honest_weight + byzantine_count * each_byzantine_weight == 1
    assert tau > 2 * dr and tg >= (n + 3) * dg and (tg / tau).denominator == 1
    assert tenure * tg > li + 3 * tg
    silence = byzantine_count * nr * tau
    assert silence == 300
    return {"scope": "Allowed weighted configuration, with honest identity last in installed order",
            "Byzantine_identities": byzantine_count, "honest_identities": 1,
            "Byzantine_weight": "1/5", "honest_weight": str(honest_weight),
            "N_R": nr, "tau_R_seconds": str(tau), "T_G_seconds": str(tg),
            "K_G": tenure, "N_G": tenure, "L_i_seconds": str(li),
            "initial_silent_seconds": str(silence),
            "tenure_lhs": str(tenure * tg), "tenure_rhs": str(li + 3 * tg)}


def region_outage():
    regions = [F(2, 5), F(3, 5)]
    responsive = regions[1]
    assert responsive <= F(2, 3)
    return {"scope": "All honest, active 40%-weight region offline; global responsive-quorum assumption fails",
            "remaining_weight": str(responsive), "global_quorum": "> 2/3",
            "successor_region_READY_fraction": "1", "global_ROTATE_can_certify": False}


def premature_rotation():
    # Each region's list is [Byzantine(weight 1), honest(weight 3)].
    n, nr, tau, dr, dg, tg, kg, ng = 4, 10, F(1), F(1, 10), F(1, 10), F(2), 100, 1
    li = (2 * nr + 1) * tau + dr
    assert F(3) > F(2, 3) * 4 and F(6) > F(2, 3) * 8
    assert tau > 2 * dr and tg >= (n + 3) * dg and (tg / tau).denominator == 1
    assert min(kg, ng) * tg <= li + 3 * tg  # Deliberately invalid tenure.
    active, activation_epoch, pending_rotate_epoch = 0, 1, None
    rows = []
    for epoch in range(1, 22):
        if pending_rotate_epoch is not None and epoch == pending_rotate_epoch + 2:
            active = 1 - active
            activation_epoch = epoch
            pending_rotate_epoch = None
        first_slot = int((epoch - activation_epoch) * tg / tau) + 1
        slots = list(range(first_slot, first_slot + int(tg / tau)))
        owners = [((slot - 1) // nr) % 2 for slot in slots]
        assert all(owner == 0 for owner in owners)  # No honest term is reached.
        if pending_rotate_epoch is not None:
            mode = "CONFIRM"
        elif epoch - activation_epoch >= ng:
            mode = "ROTATE"
            pending_rotate_epoch = epoch
        else:
            mode = "HOLD"
        rows.append({"epoch": epoch, "active_region": active + 1, "slots": slots,
                     "mode": mode, "adopted_height": 0})
    assert [r["mode"] for r in rows] == ["HOLD", "ROTATE", "CONFIRM"] * 7
    return {"scope": "All availability and timing premises hold; minimum-tenure inequality is violated",
            "first_silent_term_seconds": int(nr * tau), "residence_seconds": int(3 * tg),
            "tenure_lhs": str(min(kg, ng) * tg), "required_rhs": str(li + 3 * tg),
            "epochs": rows,
            "infinite_extension": "Every three epochs reset to identical state modulo region/view/epoch labels"}


if __name__ == "__main__":
    result = {"model_limit": "Hand-built rule abstractions; no network/parser implementation or general model proof",
              "delayed_forward_deadlock": delayed_forward(),
              "healthy_depth_backpressure": depth_one(),
              "weighted_identity_silence": identity_schedule(),
              "global_quorum_outage": region_outage(),
              "premature_rotation_starvation": premature_rotation()}
    Path(__file__).with_name("liveness_trace_results.json").write_text(json.dumps(result, indent=2) + "\n")
    print("PASS: delayed-forward deadlock, healthy depth-cap stalls, weighted-identity silence, regional-outage quorum arithmetic, and premature-rotation starvation.")
