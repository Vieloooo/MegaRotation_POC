"""Check M5 application replay, lifecycle and clock evidence against verified records."""

from collections import defaultdict

from evidence import (
    ANNOUNCEMENT, CLOCK_SKEW, EXPIRY, GRACE, READY, US, need, one, timestamp,
)


def hx(value):
    if isinstance(value, str):
        return value.removeprefix("0x").lower()
    return bytes(value).hex()


def pos(kind, key, index):
    index_kind = {"Expire": "Height", "Resume": "L1"}.get(kind, kind)
    return {"kind": kind, "key_us": key, "index": {index_kind: index}}


def events(trace, kind):
    return (row for row in trace if row["kind"] == kind)


def l1_pos(position):
    return dict(zip(("block", "tx", "log"), position, strict=True))


def check_g(trace, public):
    rows = [row for row in trace if row["kind"] in {"GAccepted", "GPadding"}]
    need(len(rows) == len(public["regional_entries"]), "regional G append count")
    for row, expected in zip(rows, public["regional_entries"], strict=True):
        actual = row["detail"]
        need((actual["height"], hx(actual["cid"]), hx(actual["native_hash"]), hx(actual["state_root"]))
             == (expected["height"], expected["cid"], expected["native"], expected["state_root"]),
             "regional G append hash, context or state")
        need(actual["position"] == pos("Espresso", expected["theta_us"], expected["q"]),
             "regional G append replay position")
        need(hx(actual["physical_id"]) == (expected["physical"] or expected["target"]),
             "regional G append physical identity")
        if expected["physical"]:
            need(row["kind"] == "GAccepted" and hx(actual["parent"]) == expected["parent"],
                 "regional G append physical parent")
    timer_rows = [row for row in events(trace, "GfrReplayRecord")
                  if row["detail"]["position"]["kind"] in {"Expire", "SessionEnd"}]
    need(len(timer_rows) == len(public["timers"]), "actual and independent timer count differ")
    for row, timer in zip(timer_rows, public["timers"], strict=True):
        detail = row["detail"]
        actual = detail["position"]
        need((actual["kind"], actual["key_us"]) == (timer["kind"], timer["key_us"]),
             "actual timer ran at the wrong public position")
        covering = public["espresso"][timer["q_star"]]
        need(row["tick_us"] >= covering["finalized_us"] and covering["theta_us"] >= timer["key_us"],
             "timer ran before its public Espresso covering input")
        if timer["kind"] == "Expire":
            need(actual["index"] == {"Height": timer["height"]},
                 "Expire ran on a height other than the next missing G height")
            need([next(iter(item["body"])) for item in detail["events"]] == ["BranchClosed"],
                 "Expire did not produce exactly its branch closure")
            need(detail["events"][0]["body"]["BranchClosed"] == {
                "branch": timer["branch"], "reason": "Expired",
            }, "Expire BranchClosed branch/reason")
    return rows, timer_rows


def check_stall(trace, public, driver, references, lifecycle, mode):
    fault = one(events(trace, "OrderingFault"), "OrderingFault")
    recovered = one(events(trace, "OrderingRecovered"), "OrderingRecovered")
    start, end = fault["tick_us"], recovered["tick_us"]
    cut = [e for e in public["espresso"] if e["finalized_us"] < start]
    held = [e for e in public["espresso"] if start <= e["finalized_us"] < end]
    after = [e for e in public["espresso"] if e["finalized_us"] >= end]
    need(cut and after and start < end, "ordering stall lacks public before/after inputs")
    before = cut[-1]
    need((fault["detail"]["q"], fault["detail"]["theta_us"]) ==
         (before["q"], before["theta_us"]), "fault marker disagrees with last genuine public row")
    if mode == "frozen":
        need(held and all(e["theta_us"] == before["theta_us"] and e["messages"] == 0 and not e["live_at_proposal"] for e in held)
             and held[-1]["q"] > before["q"], "frozen watermark profile did not continue empty finalized q")
    else:
        need(not held, "stopped ordering profile finalized during its stall")

    need(after[0]["theta_us"] > before["theta_us"] and after[0]["q"] > before["q"]
         and after[0]["live_at_proposal"],
         "public ordering did not advance after recovery")
    need(not [r for r in trace if start <= r["tick_us"] < end
              and r["kind"] in {"GovernanceSignature", "ResumeEligibility", "ResumeSubmitted"}],
         "RESUME action during a public ordering stall")
    finalized = sorted(height for height, at in driver["finalized_us"].items() if start <= at < end)
    if mode != "short":
        need(len(finalized) >= 2, "actual L1 finality halted during the ordering fault")
        before_l1 = max(h for h, t in driver["read_us"]["A"].items() if t < start)
        after_l1 = max(h for h, t in driver["read_us"]["A"].items() if t < end)
        model = references["A"]
        need(model.at_end[after_l1] > model.at_end[before_l1],
             "independent D did not grow while Espresso was stalled")
    if mode in {"stopped", "frozen", "slow-rpc", "late", "no-target", "inputs-wait"}:
        need(start == 11_250_000 and end >= 24_500_000, "frozen principal fault boundaries")
        stop_key = 21 * US
        stop = [r for r in events(trace, "LocalStopped") if r["pid"] in range(5)]
        need({r["pid"] for r in stop} == set(range(5)) and len(stop) == 5,
             "no explicit native local stop for all five old replicas")
        for row in stop:
            pid = row["pid"]
            tick = row["tick_us"]
            actual = row["detail"]
            terminal = one((entry for entry in lifecycle["terminal"]
                            if entry["pid"] == pid and entry["cid"] == hx(actual["cid"])),
                           "native terminal effect from durable Ack")
            need(start <= tick < end and stop_key - CLOCK_SKEW <= tick <= stop_key + CLOCK_SKEW,
                 "old native stop outside configured clock bound or stall")
            need(terminal["tick_us"] == tick and terminal["event"] == row["event_id"]
                 and terminal["step"] == actual["logical_step_id"]
                 and actual["local_stop_us"] == stop_key and actual["local_time_us"] >= stop_key
                 and abs(actual["local_time_us"] - tick) <= CLOCK_SKEW,
                 "LocalStopped trace does not identify the actual bounded native terminal Ack")
            ack = one((r for r in events(trace, "AckCommitted")
                       if r["pid"] == pid and r["detail"]["logical_step_id"] == terminal["step"]),
                      "native stop Ack committed")
            need(ack["seq"] < row["seq"] and ack["detail"]["root_hash"] == actual["ack"],
                 "observed local stop escaped its own durable Ack")
        waits = [r for r in events(trace, "GfrCoverageWait") if start <= r["tick_us"] < end]
        need(waits, "missing actual GFR missing-watermark result")
        for row in waits:
            detail = row["detail"]
            if row["tick_us"] >= 24 * US:
                previous = [r for r in events(trace, "GfrReplayRecord") if r["seq"] < row["seq"]]
                need(previous and detail["pc"] == previous[-1]["detail"]["position"]
                     and detail["pc"]["key_us"] < 24 * US
                     and "EspressoWatermark" in detail["status"] and detail["status"].startswith("Wait")
                     and detail["through"] == pos("Expire", 24 * US, 2),
                     "stalled GFR did not wait at the original actual public cursor")
        need(any(row["tick_us"] >= 24 * US for row in waits), "no observation of WAIT at/after expiry")
        expiry = one((t for t in public["timers"] if t["kind"] == "Expire" and t["branch"] == 0),
                     "actual recovered public expiry")
        need(expiry["key_us"] == 24 * US and expiry["height"] == 2
             and public["espresso"][expiry["q_star"]]["finalized_us"] >= end
             and not [t for t in public["timers"] if t["branch"] == 0 and t["kind"] == "SessionEnd"],
             "public recovery did not apply Expire ahead of its same-key SessionEnd")
    return {"start_us": start, "recovered_signal_us": end, "pre_fault_q": before["q"],
            "frozen_rows": len(held), "first_restored_q": after[0]["q"],
            "first_restored_public_key_us": after[0]["theta_us"], "stall_l1_finalized": finalized}


def check_resume(raw, trace, public, resume, cut, canonical_cut, stages, lifecycle, config, mode):
    oracle = one(public["resumes"], "independent exact RESUME")
    actual = one((r for r in events(trace, "GfrReplayRecord")
                  if r["detail"]["position"]["kind"] == "Resume"), "public RESUME record")
    coordinate = pos("Resume", resume.act_us, l1_pos(resume.position))
    detail = actual["detail"]
    need(detail["position"] == coordinate, "actual RESUME uses wrong public position")
    need(oracle["act_us"] == resume.act_us and oracle["target"] == resume.target
         and oracle["position"] == list(resume.position), "oracle and authenticated original control")
    bodies = [r["body"] for r in detail["events"]]
    expected_roles = ["BranchClosed", "Resumed", "SessionClosed" if oracle["stopped"] else "SessionAssigned"]
    need([next(iter(body)) for body in bodies] == expected_roles
         and all(r["position"] == coordinate for r in detail["events"]),
         "public RESUME is not one ordered lifecycle batch")
    need(bodies[0]["BranchClosed"] == {"branch": oracle["closed_beta"], "reason": "Resume"},
         "wrong old branch closed on RESUME")
    resumed = bodies[1]["Resumed"]
    need(resumed["control_seq"] == resume.seq and resumed["branch"] == oracle["beta"]
         and resumed["anchor"] == cut["anchor"]["anchor"]
         and resumed["derived_prefix"] == [item["virtual_block"]
             for item in canonical_cut["derived_prefix"][:resume.target]],
         "Resumed body did not use immutable cut bytes")
    reset = one(events(trace, "GReset"), "actual G Reset")
    r = reset["detail"]
    need(reset["tick_us"] == actual["tick_us"] and r["resume_position"] == l1_pos(resume.position)
         and hx(r["cut_digest"]) == cut["cut_digest"]
         and r["target_height"] == resume.target and r["old_height"] == oracle["old_head"]
         and r["common_height"] == oracle["common_height"]
         and [hx(v) for v in r["old_hashes"]] == [entry["native"] for entry in oracle["old_suffix"]],
         "actual G reset did not keep/replace independently derived prefix")
    adopted = [entry for entry in public["expected"] if entry.get("source") == "DER"]
    derived = list(events(trace, "GDerived"))
    need(len(derived) == len(adopted), "DER adoption count")
    for row, expected in zip(derived, adopted, strict=True):
        entry = row["detail"]
        need(entry["height"] == expected["height"] and entry["position"] == coordinate
             and entry["resume_position"] == l1_pos(resume.position)
             and hx(entry["native_hash"]) == expected["native"]
             and hx(entry["state_root"]) == expected["state_root"], "DER provenance/native state")
    cutoff_q = oracle["q_star"]
    need(stages["finalized_us"][resume.position[0]] <= actual["tick_us"]
         and public["espresso"][cutoff_q]["finalized_us"] <= actual["tick_us"]
         and resume.act_us <= actual["tick_us"], "RESUME applied before verified covering inputs")
    live_d = max(
        r["detail"]["head"]["state"]["metadata"]["height"]
        for r in events(trace, "DerivationAvailable")
        if r["detail"]["reader"] == "A" and r["tick_us"] <= actual["tick_us"] and r["detail"]["head"] is not None
    )
    need(live_d >= resume.target, "historical cut exceeds verified live D")
    report = {"actual_act_us": resume.act_us, "apply_us": actual["tick_us"], "position": resume.position,
              "target": resume.target, "live_d_at_act": live_d, "cut_digest": cut["cut_digest"],
              "common_height": oracle["common_height"], "old_head": oracle["old_head"],
              "halt_after_alignment": oracle["stopped"]}
    active = one(events(trace, "ResumeActivated"), "RESUME activation")["detail"]
    need(active["position"] == coordinate and active["set_position"] == l1_pos(resume.set_position)
         and active["resume_position"] == l1_pos(resume.position)
         and hx(active["cut_digest"]) == cut["cut_digest"]
         and active["l1_floor"] == oracle["floor"] and active["record_id"] == detail["id"],
         "activation did not bind the independently verified cut and position")
    if mode == "late":
        need(oracle["stopped"] and active["halted"] and active["descriptor"] is None
             and active["plan_basis_height"] is None
             and bodies[2]["SessionClosed"]["reason"] == "Expired"
             and all(entry["beta"] == oracle["closed_beta"] for entry in public["regional_entries"]),
             "late expired actual RESUME did not align and HALT")
        return report

    need(not oracle["stopped"] and oracle["region"] == 2, "normal recovery did not assign R2")
    descriptor = bodies[2]["SessionAssigned"]["descriptor"]
    cid = hx(descriptor["cid"])
    plan = descriptor["plan"]
    need(descriptor["anchor"] == cut["anchor"]["anchor"] and plan["cause"] == "Resume"
         and plan["session"] == {"beta": oracle["beta"], "r": 0} and plan["region"] == oracle["region"]
         and plan["source_region"] is None
         and plan["window"] == {"h_min": resume.target + 1, "h_max": resume.target + 10},
         "actual post-Resume assignment/anchor")
    need(not active["halted"] and active["plan_basis_height"] == oracle["floor"]
         and active["descriptor"] == descriptor,
         "activation not bound to independently verified historic Plan position")
    new = [r for r in public["regional_entries"] if r["beta"] == oracle["beta"]]
    need(new and new[0]["height"] == resume.target + 1 and new[0]["physical"]
         and new[0]["parent"] == hx(cut["anchor"]["anchor"]["block"]["block_id"])
         and new[0]["cid"] == cid and new[0]["metadata"]["origin"] >= oracle["floor"],
         "new physical first block did not execute from exact derived anchor")
    ready, enter = [
        {r["pid"]: r for r in events(trace, kind) if hx(r["detail"]["cid"]) == cid}
        for kind in ("Ready", "Enter")
    ]
    need(len(enter) >= 4 and ready.keys() == enter.keys()
         and all(pid // 5 == oracle["region"] for pid in enter)
         and len({r["tick_us"] for r in enter.values()}) >= 4,
         "expected at least four distinct staggered new workers, one CID")
    deadline = timestamp(config, resume.target + 1) - CLOCK_SKEW
    eligible = [r["tick_us"] for pid, r in enter.items() if actual["seq"] < ready[pid]["seq"] <= r["seq"]
                and r["tick_us"] < deadline
                and ready[pid]["detail"]["state_root"] == oracle["anchor_root"]]
    need(len(eligible) >= 4, "new Resume quorum missed its verified state/start deadline")
    ready_quorum = sorted(eligible)[3]
    need(0 <= ready_quorum - resume.act_us <= READY, "quorum missed frozen recovery/readiness budget")
    first = one((r for r in events(trace, "GAccepted") if hx(r["detail"]["cid"]) == cid
                 and r["detail"]["height"] == resume.target + 1), "actual first new physical G accept")
    proof = one((r for r in public["public_finals"] if r["cid"] == cid
                 and r["target"] == new[0]["physical"]), "native proof present at recovered G accept")
    observed = proof["observed"][0]
    proposal_us = native_proposal(raw, trace, cid, new[0]["physical"], observed,
                                  timestamp(config, resume.target + 1))
    publication = public["espresso"][new[0]["q_avail"]]
    need(new[0]["q_avail"] >= oracle["q_star"] and resume.act_us <= ready_quorum <= observed
         and observed <= publication["proposed_us"] <= publication["finalized_us"] <= first["tick_us"],
         "Ready → Native Final → publication → G accept order")
    need(new[0]["theta_us"] < min(timestamp(config, resume.target + 1) + EXPIRY,
                                 timestamp(config, resume.target + 11) + GRACE),
         "restored block misses public Expire or SessionEnd")
    need(all(seq > first["seq"] for seq in lifecycle["exits"].values()),
         "one of the 15 workers exited before restoration completed")
    report.update({"ready_quorum_us": ready_quorum, "not_before_us": timestamp(config, resume.target + 1),
                   "first_proposal_us": proposal_us, "first_final_us": observed,
                   "first_complete_q": new[0]["q_avail"],
                   "first_global_accept_us": first["tick_us"], "new_height": new[0]["height"],
                   "new_native_hash": new[0]["native"], "region": oracle["region"], "cid": cid})
    return report


def native_proposal(raw, trace, cid, physical, before_final, not_before):
    from wire import fields, frames, single, zigzag

    proposals = [r for r in events(trace, "NativeProposal")
                 if hx(r["detail"]["cid"]) == cid and hx(r["detail"]["physical_id"]) == physical]
    need(proposals, "no actual native proposal witness for first new physical")
    first = proposals[0]
    detail = first["detail"]
    step = detail["logical_step_id"]
    pid = first["pid"]
    committed = one((r for r in events(trace, "AckCommitted")
                     if r["pid"] == pid and r["detail"]["logical_step_id"] == step),
                    "native proposal persisted Ack")
    need(committed["seq"] < first["seq"] and committed["event_id"] == first["event_id"]
         and committed["detail"]["root_hash"] == detail["ack"],
         "native proposal has no durable Ack identity")
    for direction, actor, encoded in frames(raw / "frames.bin"):
        if direction != 1 or actor != pid:
            continue
        ack = fields(encoded, 6)
        if ack[2] != step:
            continue
        signs = []
        for effect in ack[6]:
            tag, body = single(effect)
            if tag != 6:
                continue
            sign = fields(body)
            if (hx(sign[1]) == cid and sign[3] == detail["view"] and sign[4] == 1):
                signs.append(sign)
        # Only a new leader notarization over this proposal denotes first issuance.
        matched = [sign for sign in signs if sign[5] == 1 and hx(sign[6][-32:]) == physical]
        [actual] = matched
        need((detail["view"] - 1) % 5 == pid % 5 and zigzag(actual[12]) >= not_before
             and actual.get(11, 0) == first["tick_us"] <= before_final,
             "native proposer identity, local NotBefore or initial Final order")
        return first["tick_us"]
    raise ValueError("missing raw Ack containing new proposal")


def check_rpc(raw, legacy, modes):
    # Sample fields are taken from actual committed raw frames. The lifecycle
    # checker already binds each frame and lease to the durable Ack/journal.
    from wire import fields, frames, single, zigzag
    delays, offsets = defaultdict(set), defaultdict(set)
    for direction, pid, frame in frames(raw / "frames.bin"):
        if direction:
            continue
        grant = fields(frame)
        kind, data = single(grant[7])
        if kind != 2:
            continue
        sample = fields(data)
        as_of = sample.get(8, 0)
        offsets[pid].add(zigzag(sample.get(9, 0)) - as_of)
        delays[pid].add(grant.get(4, 0) - as_of)
    expected_delay = 1000 if "slow-rpc" in modes else 250
    need(delays and all(values == {expected_delay} for values in delays.values()),
         "Clock RPC latency is not the actual independent transport profile")
    need(offsets and all(len(values) == 1 for values in offsets.values())
         and {-1000, 1000} <= set().union(*offsets.values())
         and all(abs(value) <= CLOCK_SKEW for value in set().union(*offsets.values())),
         "clock offsets must include both extremes and stay within +/-1ms")
    return {"delay_us": expected_delay, "offsets_us": {pid: next(iter(v)) for pid, v in offsets.items()}}
