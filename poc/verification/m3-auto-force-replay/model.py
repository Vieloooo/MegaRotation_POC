"""Facts and timeline for an offline replay of the AF01 scenario."""


def require(condition, message):
    if not condition:
        raise ValueError(message)


def seconds(us):
    return f"{us / 1_000_000:.6f} s" if us is not None else "—"


def position_name(value):
    if value is None:
        return "—"
    return f"{value['kind']} {value['index']} · {seconds(value['key_us'])}"


def event(lane, stage, label, tick, seq, facts=(), refs=(), **state):
    return dict(lane=lane, stage=stage, label=label, tick_us=tick, seq=seq,
                facts=list(facts), refs=list(refs), **state)


def signer_view(detail, stall_us):
    previous, verified = detail["previous"], detail["espresso_verified"]
    advertised = detail["espresso_advertised"]
    q_advances = bool(previous and verified and verified["q"] > previous["q"])
    theta_advances = bool(previous and verified and verified["theta_us"] > previous["theta_us"])
    position = detail["replay_position"]
    ready = bool(verified and advertised == verified and
                 detail["l1_verified"] == detail["l1_advertised"] and position and
                 position["kind"] == "Espresso" and
                 position["index"]["Espresso"] == verified["q"] and
                 position["key_us"] == verified["theta_us"])
    progress, observed = detail.get("progress_us"), detail.get("observation_us")
    gap = observed - progress if observed is not None and progress is not None else None
    if detail["eligible"]:
        require(detail["reason"] == "ELIGIBLE" and ready and detail["clock_good"] and
                detail["tail"] == "NEXT" and q_advances and theta_advances and
                gap is not None and gap >= stall_us,
                f"signer {detail['signer']}: eligibility contradicts public evidence")
    return {**detail, "ready": ready, "gap_us": gap, "stall_us": stall_us,
            "q_advances": q_advances, "theta_advances": theta_advances}


def signature_set(rows, threshold):
    require(len(rows) == threshold, "governance signature threshold was not met exactly")
    signers = {row["signer"] for row in rows}
    addresses = {row["address"].lower() for row in rows}
    require(len(signers) == len(addresses) == threshold, "duplicate governance signer")
    digest = rows[0]["digest"]
    request = rows[0]["request"]
    require(all(row["digest"] == digest and row["request"] == request for row in rows),
            "governance signatures disagree on digest or request")
    return digest, request


def historical_choice(policy, source_region, reference_origin, inclusion_height, old_floor):
    cycle = policy["order"]
    mask = policy["enabled_mask"]
    require(sorted(cycle) == [0, 1, 2] and 0 <= mask <= 7, "invalid policy cycle/mask")
    start = (cycle.index(source_region) + 1) % len(cycle)
    traversal = cycle[start:] + cycle[:start]
    floor = max(old_floor, inclusion_height)
    return next((r for r in traversal if mask & (1 << r)), None), traversal, floor, max(reference_origin, floor)


def assemble(mode, events, profile, provenance):
    points = [event("g", "start", "起点", 0, 0, head=0, duty="R0 · β0/r0")]
    points.extend(sorted(events, key=lambda row: (row["tick_us"], row["seq"])))
    require(all(left["seq"] < right["seq"] for left, right in zip(points, points[1:])),
            "timeline seq order differs from virtual time/seq ordering")
    return dict(schema="m3-auto-force-replay-v1", mode=mode, events=points, profile=profile, provenance=provenance)
