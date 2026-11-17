"""Check each M5 signer's actual delivered views and authorize the mined request independently."""

from collections import defaultdict
from itertools import groupby

from evidence import (
    ANNOUNCEMENT, CLOCK_SKEW, DRIFT, EPSILON, PREPARATION, US,
    need, one, read, solve, timestamp,
)
from timeline import events, hx

MAX_HEAD_AGE = US
POLL = 100_000
SIGN = 10_000
BROADCAST = 10_000


def cursor_height(batches, resumes, at, before=None):
    changes = [(b.position, b.records[-1]["height"]) for b in batches]
    changes += [(r.set_position, r.target) for r in resumes]
    return next((height for position, height in sorted(changes, reverse=True)
                 if position[0] <= at and (before is None or position < before)), 0)


def complete_tip(delivered):
    last = -1
    while last + 1 in delivered:
        last += 1
    return last


def expected_tip(espresso, q):
    if q < 0:
        return None
    row = espresso[q]
    return {"q": row["q"], "theta_us": row["theta_us"]}


def verify_d(d, reader, tick, refs, driver, genesis, batches, resumes):
    model = refs[reader]
    actual = d["l1_height"]
    reads = driver["read_us"][reader]
    need(actual in model.at_end and all(reads[h] <= tick for h in range(actual + 1)),
         "signer D observation exceeds its own independently authenticated read prefix")
    height = d["d_height"]
    need(0 <= height <= model.at_end[actual] and (not d["drained"] or height == model.at_end[actual]),
         "signer D head or drained claim differs from independent native derivation")
    blocks = model.blocks[:height]
    hashes = [hx(genesis["native_hash"]), *(b["native_hash"] for b in blocks)]
    origin = blocks[-1]["metadata"][2] if blocks else genesis["anchor"]["block"]["origin"]
    need(d["d_origin"] == origin and list(map(hx, d["hashes"])) == hashes,
         "signer D observation is not the actual derived native prefix")
    need(d["inbox_height"] == cursor_height(batches, resumes, actual),
         "signer Inbox head differs from actual verified events")
    if d["services_ready"]:
        need(blocks and blocks[0]["cause"][0] == "batch" and blocks[0]["kept"],
             "service-ready claim does not have real ordinary accepted payload")
    return hashes


def independently_observed(raw, trace, public, refs, driver, genesis, bundle,
                           batches, resumes, config, mode, legacy):
    # Each trace observer's address is mapped to a separate named Driver/DA feed.
    need({f"S{i}" for i in range(3)} <= refs.keys(), "no independent D feed for all three signers")
    espresso = public["espresso"]
    d_sources = (batches, resumes)
    l1_final = 0
    q_final = -1
    e_held = defaultdict(set)
    actual_l1 = {}
    d_delivery = {}
    latest_distinct = {}
    motion = {}
    probes = {
        (row["reader"], row["l1_height"], row["tick_us"]): row["service_probe"]
        for row in legacy.load_rows(raw / "derivation/trace.jsonl") if "service_probe" in row
    }
    observations = []
    first_resume = min((row["seq"] for row in events(trace, "ResumeActivated")), default=2**64)
    delay = 1000 if mode == "slow-rpc" else 250
    for row in trace:
        kind, tick, detail = row["kind"], row["tick_us"], row["detail"]
        if kind == "L1Finalized":
            l1_final = detail["height"]
            need(driver["finalized_us"][l1_final] == tick, "advertised L1 differs from actual finality")
        elif kind in {"EspressoHeartbeat", "PublisherRecorded"}:
            q = detail["q"]
            need(q == q_final + 1 and espresso[q]["finalized_us"] <= tick,
                 "advertised Espresso differs from actual complete finalized log")
            q_final = q
        elif kind == "GovernanceEspressoAvailable":
            s, q = detail["signer"], detail["q"]
            need(s in range(3) and q <= q_final and q not in e_held[s]
                 and detail["theta_us"] == espresso[q]["theta_us"],
                 "signer evidence received noncanonical or premature Espresso")
            e_held[s].add(q)
        elif kind == "GovernanceL1Available":
            s, height = detail["signer"], detail["height"]
            reader = f"S{s}"
            need(detail["reader"] == reader and height == actual_l1.get(s, -1) + 1
                 and driver["read_us"][reader][height] <= tick and height <= l1_final,
                 "signer receipt observation exceeds its own actual verified L1 delivery")
            need(hx(detail["hash"]) == hx(bundle["blocks"][height]["hash"]),
                 "signer receipt observation omitted the actual L1 block identity")
            actual_l1[s] = height
            d = detail["d"]
            d_delivery[s] = d
            if d is not None:
                verify_d(d, reader, tick, refs, driver, genesis, *d_sources)
                probe = probes[reader, height, tick]
                need(d["services_ready"] == all(probe[key] for key in (
                    "da_verified", "batches_reconciled", "batcher_available", "certifier_available",
                )), "signer service readiness does not follow actual local capability probes")
        if kind not in {"GovernancePoll", "ResumeInputEvaluation"}:
            continue
        view = detail
        need(view["signer"] in range(3), "unknown governance participant")
        s = view["signer"]
        reader = f"S{s}"
        observed = complete_tip(e_held[s])
        expected = expected_tip(espresso, observed)
        need(view["espresso_verified"] == expected
             and view["espresso_advertised"] == expected_tip(espresso, q_final),
             "signer snapshot claim differs from actual delivered/advertised Espresso")
        explicit = view["l1_verified"]
        read = driver["read_us"].get(reader, {})
        received = complete_tip({height for height, at in read.items() if at <= tick})
        need(view["l1_advertised"] == l1_final and explicit == actual_l1.get(s)
             and (explicit is None or explicit <= min(received, l1_final)),
             "signer snapshot L1 claim lacks actual named-reader coverage")
        clock = view["clock_sample"]
        good = (clock is not None and
                clock["as_of_us"] <= clock["received_us"] <= tick < clock["lease_end_us"] and
                abs(clock["offset_us"]) <= CLOCK_SKEW)
        need(view["clock_good"] == good and view["clock_sample_us"] == tick,
             "signer clock evidence does not imply its claimed lease")
        if clock is not None:
            need(clock["received_us"] - clock["as_of_us"] == delay
                 and clock["lease_end_us"] - clock["as_of_us"] == 180_000
                 and clock["offset_us"] == (s - 1) * 500,
                 "independent signer clock/RPC sample differs from frozen model")
        d = view["d"]
        need(d == d_delivery.get(s), "poll borrowed D from a different reader/event")
        d_hashes = None if d is None else verify_d(d, reader, tick, refs, driver, genesis, *d_sources)
        if row["seq"] >= first_resume:
            continue
        pc = view["replay_position"]
        replay_q = pc["index"]["Espresso"] if pc and pc["kind"] == "Espresso" else None
        projected = None
        if replay_q is not None:
            need(0 <= replay_q <= observed and pc["key_us"] == espresso[replay_q]["theta_us"],
                 "claimed signer replay cursor does not exist in its actual delivered log")
            g_entries = [entry for entry in public["regional_entries"]
                         if entry["beta"] == 0 and entry["q"] <= replay_q]
            g = [hx(genesis["native_hash"]), *(entry["native"] for entry in g_entries)]
            need([entry["height"] for entry in g_entries] == list(range(1, len(g))),
                 "pre-resume G replay has a noncontiguous witness")
            is_halted = any(timer["kind"] == "Expire" and timer["branch"] == 0
                            and timer["q_star"] <= replay_q for timer in public["timers"])
            if "tail" in view:
                need(view["tail"] == ("HALT" if is_halted else "NEXT")
                     and view.get("branch", 0) == 0,
                     "per-signer GFR Tail disagrees with its own publicly covered timer history")
            if "g_head" in view:
                need(view["g_head"] == len(g) - 1, "per-signer G head differs from actual replay")
            if d_hashes is not None:
                mismatch = next(((h, x, y) for h, (x, y) in enumerate(zip(g, d_hashes)) if x != y), None)
                reason = ("HASH_DIVERGENCE" if mismatch else
                          "HALTED_D_ADVANCED" if is_halted and len(d_hashes) > len(g) else None)
                projected = (g, is_halted, mismatch, reason)

        if "ordering_live" in view:
            prior = latest_distinct.get(s)
            need(view["previous"] == prior, "signer did not retain its own preceding distinct verified observation")
            fresh = (expected is not None and espresso[observed]["live_at_proposal"]
                     and 0 <= tick - expected["theta_us"] <= MAX_HEAD_AGE)
            if not fresh:
                latest_distinct[s], motion[s] = None, False
            elif prior != expected:
                motion[s] = bool(prior and prior["q"] < expected["q"]
                                 and prior["theta_us"] < expected["theta_us"]
                                 and tick - prior["theta_us"] <= MAX_HEAD_AGE)
                latest_distinct[s] = expected
            need(view["ordering_live"] == bool(motion.get(s) and fresh),
                 "ordering-live claim is not two distinct fresh verified q and theta samples")
        if view["eligible"] or view.get("solution") is not None:
            need(view.get("ordering_live") is True and good and projected is not None and d is not None and d["drained"] and d["services_ready"]
                 and d["l1_height"] == explicit == l1_final and observed == q_final == replay_q,
                 "solver or signature without complete independently observed inputs")
            need(expected["theta_us"] <= tick and tick - expected["theta_us"] <= MAX_HEAD_AGE,
                 "solved target from stale/future Espresso watermark")
            need(driver["read_us"][reader][explicit] <= tick and
                 int(bundle["blocks"][explicit]["timestamp"], 16) * US
                 >= max(0, expected["theta_us"] - ANNOUNCEMENT),
                 "solved target without needed L1 control coverage")
            g, halted, mismatch, reason = projected
            need(reason is not None and view["recovery_reason"] == reason,
                 "recovery reason did not follow same-height independent native hash comparison")
            comparison = view["common_comparison"]
            need(comparison["through"] == min(len(g), len(d_hashes)) - 1,
                 "recovery evidence omitted a common-height native comparison")
            claimed = comparison["first_mismatch"]
            need((claimed is None and mismatch is None) or (
                claimed is not None and mismatch is not None
                and (claimed["height"], hx(claimed["g_hash"]), hx(claimed["d_hash"])) == mismatch
            ), "signer claimed a different native mismatch")
            lo = ((tick + SIGN + BROADCAST - config["l1_genesis_timestamp_us"])
                  // config["l1_block_us"] + 1) * config["l1_block_us"] + config["l1_genesis_timestamp_us"]
            hi = lo + config["l1_block_us"] * (2 if mode == "no-target" else 1)
            need(d["pending_batch_through"] == d["inbox_height"],
                 "frozen profile must document no additional queued batch")
            solution = solve(config, len(g) - 1, d["d_height"], d["inbox_height"], lo, hi,
                             d_origin=d["d_origin"], tail=view["tail"])
            actual = view["solution"]
            expected_fields = {
                "t_l_min_us": lo, "t_l_max_us": hi, "b_hi": solution["b_hi"],
                "e_hi": solution["epoch_hi"], "h_time": solution["time_height"],
                "extra_origin": solution["origin_extra"], "g_max": solution["g_max"],
                "d_max": solution["d_max"], "inbox_max": solution["inbox_max"],
                "h_guard": solution["h_guard"], "lower_time_us": solution["lower_us"],
                "upper_time_us": solution["upper_us"],
                "lower_height": max(solution["h_guard"] + 1,
                                    (solution["lower_us"] - config["genesis"]["timestamp_us"]
                                     + config["l2_slot_us"] - 1) // config["l2_slot_us"]),
                "upper_height": (solution["upper_us"] - config["genesis"]["timestamp_us"])
                                // config["l2_slot_us"],
                "target": solution["target"],
            }
            need(actual == expected_fields, f"solver evidence differs: actual={actual!r}, expected={expected_fields!r}")
            need(view["eligible"] == (solution["target"] is not None), "invalid/omitted target eligibility")
        observations.append((row, view, projected))
    need({v["signer"] for _, v, _ in observations} == set(range(3)),
         "missing raw governance observations for one of the three independent signers")
    return observations


def check_authorization(raw, trace, public, refs, driver, genesis, bundle, batches,
                        resumes, config, committee, legacy, mode):
    import independent_codec as crypto
    import receipt_checker as receipt
    from verified_inputs import ResumeReceiptChecker

    observations = independently_observed(
        raw, trace, public, refs, driver, genesis, bundle, batches, resumes, config, mode, legacy,
    )
    signer_polls = [(row, view, projection) for row, view, projection in observations
                    if row["kind"] == "GovernancePoll"]
    votes = list(events(trace, "GovernanceSignature"))
    submissions = list(events(trace, "GovernanceSubmission"))
    ready_rows = [(r, v) for r, v, _ in observations if v["eligible"]]
    outcomes = defaultdict(int)
    for _, view, _ in signer_polls:
        outcomes[view["reason"]] += 1
    report = {"polls": len(signer_polls), "independently_checked_views": len(observations),
              "reason_counts": dict(outcomes)}
    if not resumes:
        need(not votes and not submissions and not list(events(trace, "ResumeEligibility"))
             and not list(events(trace, "ResumeSubmitted")),
             "negative scenario generated a RESUME quorum, request or individual authorization")
        contrast = {"short": {"NO_RECOVERY_REASON"}, "no-target": {"NO_TARGET"},
                    "inputs-wait": {"RECOVERY_INPUTS_NOT_READY", "INPUTS_NOT_READY"}}[mode]
        need(any(outcomes[reason] for reason in contrast), f"negative contrast lacks {contrast}")
        if mode == "inputs-wait":
            lagged = {view["signer"] for row, view, _ in observations
                      if row["tick_us"] >= 24_500_000
                      and view["espresso_verified"] != view["espresso_advertised"]
                      and view["reason"] == "INPUTS_NOT_READY"}
            need(len(lagged) >= 2, "no actual independent peer inputs were held back")
            report["lagged_signers"] = sorted(lagged)
        else:
            need(not ready_rows, "unexpected individual eligibility in negative short/no-target scenario")
        return report
    resume = one(resumes, "successful actual M5 RESUME")
    submitted = one(submissions, "one automated durable submission")
    tx = bundle["blocks"][resume.position[0]]["transactions"][resume.position[1]]
    name, (rid, seq, policy, target, origin, signatures) = receipt.decode_call(tx)
    need(name == "submitResume", "signed upload does not contain real RESUME calldata")
    linker = ResumeReceiptChecker(bundle, committee)
    signed, accepted_signers = linker.resume_signers(
        crypto.as_bytes(tx["from"]), rid, seq, policy, target, origin, signatures,
    )
    real_digest = crypto.typed_digest(linker.domains["authority"], signed)
    expected_packed = [signatures[i:i + 65] for i in range(0, len(signatures), 65)]
    proper_mode = "HASH_DIVERGENCE" if mode == "divergence" else "HALTED_D_ADVANCED"
    observed_votes = {}
    for row in votes:
        vote = row["detail"]
        s = vote["signer"]
        match = [(r, view) for r, view, _ in observations
                 if r["seq"] < row["seq"] and view["eligible"] and view["signer"] == s]
        need(match, "committee vote without eligible actual own view")
        last, view = match[-1]
        need(0 <= row["tick_us"] - last["tick_us"] <= SIGN, "signer missed its frozen deadline")
        need(view["reason"] == proper_mode and view["solution"]["target"] == target,
             "signer used the wrong reason or nonminimal target for mined request")
        need(hx(vote["digest"]) == real_digest.hex(), "recorded vote differs from actual EIP712 Resume digest")
        signature = bytes.fromhex(hx(vote["signature"]))
        address = crypto.recover_address(signature, real_digest)
        need(hx(vote["address"]) == address.hex() and address in committee.members
             and committee.members[s] == address and address not in observed_votes,
             "nonmember, incorrect independent identity or duplicate durable signer")
        request = vote["request"]
        need((request["request_id"], request["control_seq"]) == (int.from_bytes(rid, "big"), seq)
             and request["enabled_mask"] == policy[0] and request["order"] == policy[1:4]
             and request["signed_floor"] == policy[4]
             and request["resume"] == {"target": target, "origin": origin}
             and origin == (view["solution"]["t_l_min_us"] - config["l1_genesis_timestamp_us"])
             // config["l1_block_us"], "recorded vote does not bind original signed lower epoch/target/policy")
        observed_votes[address] = (signature, row, view)
    packed_addresses = [crypto.as_bytes(address) for address in accepted_signers]
    need(len(set(packed_addresses)) == committee.threshold and set(packed_addresses) <= observed_votes.keys()
         and [observed_votes[member][0] for member in packed_addresses] == expected_packed,
         "mined committee bundle differs from independently eligible individual votes")
    need(all(observed_votes[member][1]["seq"] < submitted["seq"] for member in packed_addresses),
         "automatic request prepared before genuine threshold")

    attested = submitted["detail"]
    prepared = read(raw / "l1/governance-transaction.json")
    need(prepared == attested and hx(prepared["digest"]) == real_digest.hex()
         and hx(prepared["hash"]) == hx(resume.tx_hash)
         and hx(prepared["calldata"]) == hx(tx["input"])
         and prepared["nonce"] == receipt.q(tx["nonce"])
         and hx(prepared["signature_pack"]) == signatures.hex(),
         "durable signed preparation differs from mined controls")
    need(crypto.keccak256(bytes.fromhex(hx(prepared["raw"]))).hex() == hx(resume.tx_hash),
         "original signed transaction hash or caller nonce changed")
    journal = legacy.load_rows(raw / "governance.jsonl")
    all_states = list(events(trace, "GovernanceState"))
    need(journal == [{"phase": r["detail"]["phase"], "tick_us": r["tick_us"],
                      "detail": r["detail"]["evidence"]} for r in all_states],
         "durable governance journal is not the actual staged observation")
    phases = {phase: next(r for r in all_states if r["detail"]["phase"] == phase)
              for phase in ("Proposed", "Signed", "Prepared", "Submitted", "Included", "Finalized", "Applied")}
    need([r["seq"] for r in phases.values()] == sorted({r["seq"] for r in phases.values()})
         and phases["Prepared"]["detail"]["evidence"] == prepared
         and phases["Prepared"]["seq"] < submitted["seq"],
         "durable phases or disclosure order changed")

    # A quorum first becomes eligible only when two separate verified views
    # attest the same actual solution. Retain the last such view per signer so a
    # later closed interval/contradictory view cannot be carried forward.
    first_eligible = None
    qualifying_signers = None
    keyed = lambda item: (item[0]["event_id"], item[0]["tick_us"], item[0]["kind"])
    for (_, tick, _), group in groupby(observations, key=keyed):
        rows = list(group)
        if rows[0][0]["seq"] >= phases["Prepared"]["seq"]:
            break
        need({view["signer"] for _, view, _ in rows} == set(range(3)) and len(rows) == 3,
             "partial independent signer evaluation wave")
        qualifying = [view["signer"] for _, view, _ in rows if view["eligible"]
                      and view["solution"]["target"] == target
                      and (view["solution"]["t_l_min_us"] - config["l1_genesis_timestamp_us"])
                      // config["l1_block_us"] == origin]
        if len(qualifying) >= committee.threshold:
            first_eligible, qualifying_signers = tick, qualifying
            break
    need(first_eligible is not None, "actual threshold eligibility was never established")
    summary = one((r for r in events(trace, "ResumeEligibility")
                   if r["detail"]["request"]["resume"] == {"target": target, "origin": origin}),
                  "actual earliest threshold eligibility milestone")
    need(summary["detail"]["t_eligible_us"] == first_eligible == summary["tick_us"]
         and summary["detail"]["signers"] == qualifying_signers,
         "declared t_eligible/signers differ from first full independently qualified wave")
    actual_submit = driver["submit_us"][resume.tx_hash]
    actual_include = int(bundle["blocks"][resume.position[0]]["timestamp"], 16) * US
    intervals = {(observed_votes[member][2]["solution"]["t_l_min_us"],
                  observed_votes[member][2]["solution"]["t_l_max_us"]) for member in packed_addresses}
    lower, upper = one(intervals, "single signed inclusion interval")
    hold = mode == "late"
    need(0 <= (submitted["tick_us"] if hold else actual_submit) - first_eligible <= POLL + SIGN + BROADCAST,
         "observed t_eligible-to-submit missed frozen deadline")
    need((actual_include > upper) if hold else (lower <= actual_include <= upper),
         "actual late/normal inclusion classification and signed interval differ")
    timing = one(events(trace, "ResumeTiming"), "recorded receipt-derived timing")["detail"]
    expected_timing = {"height": resume.position[0], "t_l_min_us": lower, "t_l_max_us": upper,
                       "actual_inclusion_us": actual_include, "act_us": resume.act_us,
                       "inside_interval": not hold, "transaction_hash": resume.tx_hash}
    need(all(timing.get(key) == value for key, value in expected_timing.items()),
         "runtime timing report differs from immutable signed horizon and actual receipt")
    if "solution" in timing:
        need(all(observed_votes[member][2]["solution"] == timing["solution"]
                 for member in packed_addresses), "runtime replaced the previously signed solver envelope")
        mine = driver["mine_us"][resume.position[0]]
        genesis_l1 = max(i for i, tick in driver["read_us"]["A"].items() if tick <= mine)
        g_before = max((r["detail"]["height"] for r in events(trace, "GAccepted")
                        if r["tick_us"] <= mine), default=0)
        need(timing["g_before"] == g_before
             and timing["d_verified_before"] == refs["A"].at_end[genesis_l1]
             and timing["inbox_before"] == cursor_height(
                 batches, resumes, resume.position[0], before=resume.set_position),
             "runtime did not record the actual verified pre-own-CursorSet inputs")
    if hold:
        release = read(raw / "l1/governance-release.json")
        need(release["driver_command"]["raw"] == prepared["raw"]
             and hx(release["hash"]) == hx(resume.tx_hash)
             and actual_submit == 43_500_000 and actual_submit > submitted["tick_us"],
             "late real raw transaction was not durably held before release")
    else:
        need(submitted["tick_us"] == actual_submit, "normal prepared transaction not actually broadcast")

    need(resume.act_us == actual_include + ANNOUNCEMENT,
         "actual RESUME activation came from forecast instead of authenticated inclusion")
    if hold:
        need(actual_include == 44 * US and resume.act_us == 54 * US and target == 30,
             "late profile lacks preserved real delayed inclusion and its unchanged original target")
    else:
        # The upper envelope uses the real pre-own-CursorSet histories. Testing
        # against live D *after* our override would make the signed target itself
        # incorrectly invalidate the original decision.
        from derive_oracle import Derive
        times = {i: int(block["timestamp"], 16) * US for i, block in enumerate(bundle["blocks"])}
        model = refs["A"]
        before_own = Derive(config, genesis, model.credits, times, model.window).advance(
            resume.position[0], [b for b in batches if b.position < resume.set_position], [],
        )
        prior_inbox = cursor_height(batches, resumes, resume.position[0], before=resume.set_position)
        g_before = max((r["detail"]["height"] for r in events(trace, "GAccepted")
                        if r["tick_us"] <= driver["mine_us"][resume.position[0]]), default=0)
        for member in packed_addresses:
            sol = observed_votes[member][2]["solution"]
            need(g_before <= sol["g_max"] and len(before_own.blocks) <= sol["d_max"]
                 and prior_inbox <= sol["inbox_max"] and max(g_before, len(before_own.blocks), prior_inbox) < target,
                 "signed pre-inclusion production envelope did not cover actual heads")
    report.update({"eligible_us": first_eligible, "prepared_us": submitted["tick_us"],
                   "submitted_us": actual_submit, "actual_include_us": actual_include,
                   "inclusion_interval_us": [lower, upper], "target": target, "signed_origin": origin,
                   "reason": proper_mode, "votes": len(votes), "late": hold})
    return report
