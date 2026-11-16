#!/usr/bin/env python3
"""Verify sealed M3 run evidence using source-derived public replay and ABI custody."""

import argparse
import hashlib
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

VERIFY = Path(__file__).resolve().parents[1]
sys.path[:0] = [str(VERIFY / "m2-healthy-force"), str(VERIFY / "anvil-l1")]

import authority
import independent_codec as crypto
from wire import load

m2_spec = importlib.util.spec_from_file_location(
    "m2_evidence_checker", VERIFY / "m2-healthy-force/check.py"
)
m2 = importlib.util.module_from_spec(m2_spec)
m2_spec.loader.exec_module(m2)

BFT = "691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6"
CONTRACT = "poc/docs/milestones/M3-auto-force-task-contract-zh.md"
CONTRACT_SHA256 = "c828ca8a50ca94f6aeba06463623283273c18cb9b24531a968584153260a30e3"
EXPECTATIONS = "poc/verification/m3-auto-force/source-expectations.md"
EXPECTATIONS_SHA256 = "5c25d5178e60d728c1d535098203784ef6e57b7d8c8b4e3b7537299e81812fe8"
FAULT_TICK = 11_250_000
RECOVERY_MODES = {"outage", "reader-lag"}
ANNOUNCEMENT_US = 10_000_000
GRACE_US = 3_000_000
EXPIRY_US = 30_000_000
INIT_US = 3_000_000


def need(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, data):
    with path.open("x") as output:
        json.dump(data, output, indent=2, sort_keys=True)
        output.write("\n")


def one(rows, message):
    rows = list(rows)
    need(len(rows) == 1, f"{message}, got {len(rows)}")
    return rows[0]


def sealed(run, repository, candidate):
    path = run / "manifest.json"
    manifest_sha = digest(path)
    need((run / "manifest.sha256").read_text().split() == [manifest_sha, path.name],
         "manifest checksum missing or invalid")
    manifest = load(path.read_bytes())
    need(manifest["candidate"] == candidate and manifest["seed"] == 42 and
         manifest["status"] == "COMPLETED", "sealed status, candidate or seed differs")
    before, after = manifest["before"], manifest["after"]
    need(before == after and before["head"] == candidate and not before["status"],
         "worktree was dirty or changed during the run")
    need(before["bft_gitlink"] == BFT and manifest["commonware"] == "2026.10.0",
         "PoC Commonware or Regional BFT revision differs")
    files = manifest["files"]
    for name, expected in files.items():
        item = run / name
        need(not item.is_symlink() and item.resolve().is_relative_to(run) and item.is_file()
             and item.stat().st_size == expected["bytes"] and digest(item) == expected["sha256"],
             f"unsealed or mismatched evidence file: {name}")
    actual = {item.relative_to(run).as_posix() for item in (run / "raw").rglob("*") if item.is_file()}
    need(actual == {name for name in files if name.startswith("raw/")},
         "sealed raw inventory and actual files differ")
    for name, expected in ((CONTRACT, CONTRACT_SHA256), (EXPECTATIONS, EXPECTATIONS_SHA256)):
        need(manifest["inputs"][name] == expected and digest(repository / name) == expected,
             f"missing frozen input binding for {name}")
    for name, sha in manifest["inputs"].items():
        need(digest(repository / name) == sha, f"candidate input hash mismatch for {name}")
    return manifest, manifest_sha


def sealed_committee_artifact(run, manifest):
    names = (path for path in manifest["files"]
             if path.endswith("/CommitteeControlAuthority.sol/CommitteeControlAuthority.json"))
    return run / one(names, "expected one sealed committee contract artifact")


def bind_l1_driver(path, transcript):
    """Match scheduled Hub operations to persisted original driver requests and replies."""
    groups = []
    for row in transcript:
        if row["command"]["op"] == "tick":
            groups.append([row])
        else:
            need(groups, "L1 driver operation precedes its Hub tick")
            groups[-1].append(row)
    remaining = iter(groups)
    signed = {}
    submitted = None
    commands = {}
    for row in m2.load_rows(path):
        hub, result = row["command"], row["result"]
        tick, op = hub["tick_us"], hub["op"]
        if op == "governance_sign":
            need(result["request"] == hub["request"] and result["signer"] == hub["signer"],
                 "secp signing response changed request or member")
            signed[hub["signer"]] = result
            continue
        if op == "governance_prepare":
            need(submitted is None and result["request"] == hub["request"]
                 and result["signers"] == hub["signers"]
                 and all(signed[vote["signer"]] == vote for vote in hub["signers"]),
                 "prepare did not bind each retained committee vote")
            submitted = result
            continue
        batch = next(remaining)
        require_ready = op not in {"governance_broadcast", "governance_reconcile"}
        need(batch[0]["command"]["reference_us"] == tick and
             all(step["reply"]["reference_us"] == tick and step["reply"]["id"] == step["command"]["id"]
                 for step in batch), "Hub and L1 driver disagree on command identity or time")
        calls = batch[1:]
        kinds = [step["command"]["op"] for step in calls]
        replies = [step["reply"].get("result") for step in calls]
        if op in {"governance_broadcast", "governance_reconcile"}:
            need(submitted is not None and kinds == ["submit"] and
                 calls[0]["command"] == submitted["driver_command"],
                 "governance released different command or signed bytes")
            driver = calls[0]["reply"]
            need(result["hash"] == submitted["hash"] and result["driver_id"] == driver["id"]
                 and result["nonce"] == submitted["nonce"] and result["status"] == driver["status"],
                 "governance broadcast/reload conflicts with original durable transaction")
            if driver["status"] == "ready":
                need(replies[0]["submitted_tx_hash"] == submitted["hash"],
                     "actual L1 broadcast acknowledgment conflicts with prepared hash")
        elif op == "mine":
            need(kinds == ["mine"] and calls[0]["command"]["slot"] == hub["height"]
                 and replies[0]["included_block"] == result, "scheduled manual inclusion mismatch")
        elif op == "finalize":
            need(kinds == ["finalize"] and replies == [result]
                 and result["finalized_through"] == hub["height"], "scheduled finalization mismatch")
        elif op in {"genesis", "deliver"}:
            need(kinds == (["read"] if op == "genesis" else ["deliver", "read"])
                 and calls[-1]["command"]["reader"] == hub["reader"]
                 and calls[-1]["command"]["query"] == {"kind": "block", "height": hub["height"]}
                 and replies[-1]["control_coverage_through"] == hub["height"]
                 and replies[-1]["block"] == result, "scheduled finalized L1 reader delivery mismatch")
            if op == "deliver":
                need(replies[0]["reader"] == hub["reader"] and
                     replies[0]["delivered_through"] == hub["height"], "driver did not deliver the finalized prefix")
        elif op == "deploy":
            need(kinds == ["submit"] * len(result)
                 and [r["submitted_tx_hash"] for r in replies] == [r["hash"] for r in result],
                 "scheduled deployment does not match actual driver submission")
        else:
            raise ValueError(f"unknown M3 Hub L1 operation {op}")
        if require_ready:
            need(all(step["reply"]["status"] == "ready" for step in batch),
                 "scheduled ready Hub operation has a failed driver step")
        for step in batch:
            command = step["command"]
            previous = commands.setdefault(command["id"], command)
            if previous is not command:
                need(op == "governance_reconcile" and previous == command == submitted["driver_command"],
                     "duplicate driver id did not describe a restart of the same pending submission")
    need(next(remaining, None) is None, "raw driver calls have no scheduled Hub operation")
    return len(groups)


def check_submission_journal(raw, trace, control, broadcast):
    data = load((raw / "l1/governance-transaction.json").read_bytes())
    need(data == broadcast["detail"], "durable prepared transaction differs from trace")
    need(crypto.hx(crypto.keccak256(crypto.as_bytes(data["raw"]))) == control["tx_hash"],
         "signed transaction bytes do not determine the actual chain hash")
    need(data["driver_command"]["raw"] == data["raw"] and data["driver_command"]["op"] == "submit"
         and data["digest"] == control["request_digest"] and data["nonce"] == control["nonce"],
         "prepared L1 command changed raw transaction/nonce or signed payload")
    need(data["signature_pack"] == "0x" + "".join(signature[2:] for signature in control["signatures"]),
         "durable packed signatures differ from actual threshold payload")
    req = data["request"]
    need(req["request_id"] == int(control["request_id"], 16)
         and req["control_seq"] == control["seq"] and req["target_beta"] == control["target"]
         and req["enabled_mask"] == control["policy"]["enabled_mask"]
         and req["order"] == control["policy"]["order"] and req["signed_floor"] == control["l1_floor"],
         "durable request does not match actual Authority payload")

    states = [row for row in trace if row["kind"] == "GovernanceState"]
    durable = m2.load_rows(raw / "governance.jsonl")
    need([{"phase": row["detail"]["phase"], "tick_us": row["tick_us"],
           "detail": row["detail"]["evidence"]} for row in states] == durable,
         "raw durable governance phases disagree with the controlled trace")
    indexes = {}
    for row in states:
        phase = row["detail"]["phase"]
        indexes.setdefault(phase, row["seq"])
        if phase in {"Submitted", "SubmittedUnknown"}:
            value = row["detail"]["evidence"]
            need(value["hash"] == control["tx_hash"] and value["nonce"] == control["nonce"]
                 and value["driver_id"] == data["driver_command"]["id"],
                 "in-flight request reconciled a different raw transaction")
    need(indexes["Proposed"] < indexes["Signed"] < indexes["Prepared"] < broadcast["seq"]
         < indexes["Submitted"] < indexes["Included"] < indexes["Finalized"]
         < indexes["Applied"] < indexes["RecoveryObserved"], "governance journal phase order")
    prepared = one((row for row in durable if row["phase"] == "Prepared"), "one durable prepare")
    need(prepared["detail"] == data and prepared["tick_us"] == broadcast["tick_us"],
         "L1 raw transaction was not committed before release")
    recovered = [row for row in durable if row["phase"] == "Submitted" and row["detail"]["reloaded"]]
    need(indexes["SubmittedUnknown"] < indexes["Submitted"] and bool(recovered),
         "frozen lost-response stimulus did not visibly reconcile its durable command identity")
    return {"journal_phases": [row["phase"] for row in durable], "durable_reconciliations": len(recovered)}


def normalized_trace(raw):
    rows = []
    for row in m2.trace_rows(raw):
        if row["kind"] in {"WorkerSpawn", "WorkerExit", "WorkerOffline"}:
            for field in ("os_pid", "wall_time_ns", "proc_stat"):
                row["detail"].pop(field, None)
        rows.append(json.dumps(row, sort_keys=True, separators=(",", ":")))
    return ("\n".join(rows) + "\n").encode()


def compare_repeat(run, second, manifest, repeated):
    base, pause = run / "raw", second / "raw"
    need(manifest["wall_pause_ms"] == 0 and repeated["wall_pause_ms"] == 650,
         "wrong base/repeat wall pause")
    need(manifest["binaries"] == repeated["binaries"], "repeat used a different binary")
    base_trace = normalized_trace(base)
    need(base_trace == normalized_trace(pause)
         and base_trace == (base / "trace.virtual.jsonl").read_bytes()
         and base_trace == (pause / "trace.virtual.jsonl").read_bytes(),
         "full normalized protocol traces or saved projections differ")
    paths = ("frames.bin", "l1/raw-rpc-bundle.json")
    equal = {}
    for name in paths:
        equal[name] = digest(base / name)
        need(equal[name] == digest(pause / name), f"repeat raw {name} differs")
    return {
        "status": "PASS",
        "wall_pause_ms": 650,
        "normalized_trace_sha256": hashlib.sha256(base_trace).hexdigest(),
        "equal_raw_sha256": equal,
    }


def check_fault(trace, last):
    group = set(range(5))
    actors = [row for row in trace if row["kind"] == "WorkerSpawn"]
    need(len(actors) == 15 and {row["pid"] for row in actors} == set(range(15)),
         "expected 15 separately launched replicas")
    pids = {row["pid"]: row["detail"]["os_pid"] for row in actors}
    offline = {}
    for row in trace:
        kind, pid, detail, now = (row[key] for key in ("kind", "pid", "detail", "tick_us"))
        if kind == "WorkerOffline":
            need(pid in group and now == FAULT_TICK and pid not in offline,
                 "offline process outside frozen R0 fault")
            need(detail["os_pid"] == pids[pid] and detail["signal"] == 9 and detail["region"] == 0,
                 "offline process identity differs from startup")
            offline[pid] = row["seq"]
        if kind == "Grant" and pid in group:
            need(now < FAULT_TICK, "offline R0 received a grant")
        if kind == "WorkerExit" and pid not in group:
            need(last["seq"] < row["seq"], "online process exited before first new result")
    need(set(offline) == group, "all five old workers must become offline")
    return offline


def check_primary(trace, public, control, stages):
    need(control["position"][0] == 7 and control["act_us"] == 24_000_000 and control["target"] == 0,
         "main Force did not meet frozen inclusion/act boundary")
    need(control["policy"] == {"enabled_mask": 6, "order": [0, 1, 2]},
         "signed Authority policy does not disable R0 while preserving cycle")
    force = one(public["force"], "expected one independently replayed control")
    need(force["valid"] and force["anchor"]["block"]["height"] == 1
         and (force["closed_beta"], force["beta"], force["r"], force["region"],
              force["source_region"], force["floor"]) == (0, 1, 1, 1, 0, 7),
         "Force does not match actual pre-activation chain, region or floor")
    need((force["h_min"], force["h_max"], force["latest_ready_us"], force["expiry_key_us"])
         == (18, 27, 27_999_000, 42_000_000), "computed Force deadline or window differs")
    historical = force["historical_plan"]
    need(historical["historical_height"] == max(force["anchor"]["block"]["origin"], force["floor"])
         and historical["historical_height"] == 7, "wrong historical Plan reference header")
    need(not public["regular"], "old same-key SessionEnd incorrectly processed before Force")
    accepted = m2.acceptance_rows(trace, public)
    need(accepted[0]["tick_us"] < FAULT_TICK < stages["submit_us"] < stages["include_us"],
         "no demonstrated old G progress before the cut or inconsistent submit/inclusion")
    first = public["expected"][-1]
    need([r["height"] for r in public["expected"]] == list(range(1, 19))
         and [r["height"] for r in public["expected"] if r["physical"] is None] == list(range(2, 18)),
         "recovery did not preserve h1 and fill the exact Force gap")
    need(first["height"] == 18 and first["physical"] and (first["beta"], first["r"], first["region"])
         == (1, 1, 1) and first["theta_us"] < force["expiry_key_us"]
         and first["parent"] == m2.address(force["anchor"]["block"]["block_id"]),
         "first G physical result does not append the selected descriptor before expiry")
    need(first["q"] >= force["q_star"], "new result entered before the Force covering block")

    applied = one((row for row in trace if row["kind"] == "GfrReplayRecord" and
                   row["detail"]["position"]["kind"] == "Force"), "expected one actual GFR Force event")
    shown = one((row for row in trace if row["kind"] == "ForceActivated"),
                "expected one activation announcement")
    activated, desc = shown["detail"], shown["detail"]["descriptor"]
    pos = applied["detail"]["position"]
    block, tx, log = control["position"]
    need(pos == {"key_us": control["act_us"], "kind": "Force",
                 "index": {"L1": {"block": block, "tx": tx, "log": log}}},
         "Force applied at wrong position")
    need(activated["position"] == pos and shown["tick_us"] == applied["tick_us"]
         and activated["head_height"] == 1 and activated["old_branch"] == 0
         and activated["new_branch"] == 1 and activated["l1_floor"] == force["floor"]
         and activated["anchor_native_hash"] == force["native_hash"]
         and desc["anchor"] == force["anchor"] and m2.address(desc["cid"]) == first["cid"],
         "activation does not bind old G and the actual new descriptor")
    policy = activated["policy"]
    need(policy == {
        "enabled_mask": historical["enabled_mask"],
        "order": historical["order"],
        "signed_floor": historical["signed_l1_floor"],
        "floor": force["floor"],
        "position": pos["index"]["L1"],
        "control_seq": control["seq"],
    }, "activation policy does not bind actual signed and effective policy data")
    plan_basis = activated["plan_basis"]
    need(plan_basis["height"] == historical["historical_height"] and
         plan_basis["block_hash"] == historical["historical_hash"],
         "activation Plan basis differs from actual historical L1 header")
    need(accepted[0]["seq"] < applied["seq"] < accepted[-1]["seq"]
         and not applied["detail"]["append"], "Force rewrote G or changed visible ordering")
    effects = applied["detail"]["events"]
    need(len(effects) == 2 and effects[0]["body"] == {"BranchClosed": {"branch": 0, "reason": "Force"}}
         and effects[1]["body"] == {"SessionAssigned": {"descriptor": desc}}
         and all(e["position"] == pos for e in effects), "Force close/assignment ordering")
    now = applied["tick_us"]
    q_star = public["espresso"][force["q_star"]]
    need(public["espresso"][force["q_star"] - 1]["theta_us"] < control["act_us"] <= q_star["theta_us"],
         "incorrect earliest Espresso covering position")
    need(0 <= now - max(q_star["finalized_us"], stages["reader_us"]["A"]) <= 100_000,
         "Force preceded public coverage or exceeded replay bound")
    need(q_star["finalized_us"] - q_star["proposed_us"] <= 1_000_000
         and abs(q_star["theta_us"] - q_star["proposed_us"]) <= 1_000_000,
         "Espresso covering watermark violates declared budget")

    prep = {"Ready": {}, "Enter": {}}
    for row in trace:
        if row["kind"] not in prep or m2.address(row["detail"]["cid"]) != first["cid"]:
            continue
        by_pid = prep[row["kind"]]
        need(row["pid"] not in by_pid, "repeated Ready/Enter for selected cid")
        by_pid[row["pid"]] = row["tick_us"]
        if row["kind"] == "Ready":
            need(row["detail"]["anchor"] == force["anchor"] and
                 row["detail"]["state_root"] == force["anchor_root"], "Ready used wrong actual G state")
    ready, enter = prep["Ready"], prep["Enter"]
    need(set(ready) == set(enter) == set(range(5, 10)), "five independent new R1 replicas must prepare")
    need(len(set(enter.values())) > 1 and sum(t < force["latest_ready_us"] for t in enter.values()) >= 4,
         "stable new quorum missed Ready/Enter bound or was started in lockstep")
    need(all(now <= ready[pid] <= enter[pid] and ready[pid] - now <= 500_000 for pid in enter),
         "Ready precedes applied Force, Enter precedes Ready, or state preparation exceeded budget")

    native = min(p["observed"][0] for p in public["public_finals"] if p["target"] == first["target"])
    arrived = min(p["delivered"][0] for p in public["public_finals"] if p["target"] == first["target"])
    first_row = public["espresso"][first["q"]]
    need(force["latest_ready_us"] <= native <= 28_500_000 and 0 <= arrived - native <= 50_000
         and 0 <= first_row["proposed_us"] - arrived <= 200_000, "regional production/packing/publication budget")
    need(first_row["finalized_us"] - first_row["proposed_us"] <= 1_000_000
         and abs(first_row["theta_us"] - first_row["proposed_us"]) <= 1_000_000
         and 0 <= accepted[-1]["tick_us"] - first_row["finalized_us"] <= 100_000,
         "Espresso finalization / GFR publication replay budget")
    offline = check_fault(trace, accepted[-1])
    return {
        "state": force,
        "first_new": first,
        "activation_tick_us": now,
        "ready_us": ready,
        "enter_us": enter,
        "offline_worker_trace_seq": offline,
        "first_native_tick_us": native,
        "first_global_tick_us": accepted[-1]["tick_us"],
    }


def check_metadata_projection(trace, public, bundle):
    readers = {}
    projection_count = 0
    for row in trace:
        kind, detail = row["kind"], row["detail"]
        if kind == "L1ReaderAvailable":
            reader, height = detail["reader"], detail["height"]
            need(readers.get(reader, -1) + 1 == height, "noncontiguous finalized L1 reader prefix")
            need(m2.address(detail["hash"]) == bundle["blocks"][height]["hash"][2:].lower(),
                 "reader finalized hash differs from raw actual L1 header")
            readers[reader] = height
        if kind == "Grant":
            horizon = max((header["height"] for header in public["l1"]
                           if header["finalized_us"] + 1 <= row["tick_us"]), default=-1)
            need(detail["worker_l1_through"] == horizon and readers.get("A", -1) >= horizon,
                 "worker executed on a projected L1 header outside real finalized view A")
            projection_count += 1
    return projection_count


def evm_hex(value):
    if isinstance(value, str):
        return crypto.hx(crypto.as_bytes(value)).lower()
    return crypto.hx(bytes(value)).lower()


def check_views(trace, public):
    """Independently tie claimed poll observations to finalized published positions and G."""
    polls = []
    previous = {}
    finalized_l1 = 0
    last_finalized_q = -1
    e_rows = {row["q"]: row for row in public["espresso"]}
    g_rows = public["expected"]
    checked = 0

    for row in trace:
        kind, now, detail = row["kind"], row["tick_us"], row["detail"]
        if kind == "L1Finalized":
            finalized_l1 = detail["height"]
        elif kind in {"EspressoHeartbeat", "PublisherRecorded"}:
            q = detail["q"]
            need(e_rows[q]["finalized_us"] <= now, "log exposes Espresso before finalized")
            need(q == last_finalized_q + 1, "trace did not advance finalized Espresso in order")
            last_finalized_q = q
            if kind == "EspressoHeartbeat":
                need(detail["theta_us"] == e_rows[q]["theta_us"] and detail["message_count"] == 0,
                     "heartbeat differs from finalized empty public input")
        if kind != "GovernancePoll":
            continue
        polls.append(row)
        signer = detail["signer"]
        advertised, verified = detail["espresso_advertised"], detail["espresso_verified"]
        if advertised is not None:
            need(advertised["q"] == last_finalized_q
                 and advertised["theta_us"] == e_rows[advertised["q"]]["theta_us"]
                 and e_rows[advertised["q"]]["finalized_us"] <= now,
                 "governance advertised Espresso is not the public finalized position")
        else:
            need(last_finalized_q < 0, "governance dropped advertised finalized position")
        if verified is not None:
            need(advertised is not None and verified["q"] <= advertised["q"]
                 and verified["theta_us"] == e_rows[verified["q"]]["theta_us"],
                 "governance verified Espresso claim cannot be tied to finalized prefix")
        need(detail["l1_advertised"] == finalized_l1 and
             (detail["l1_verified"] is None or detail["l1_verified"] <= finalized_l1),
             "governance advertised/verified L1 claim differs from finalized service")

        pos = detail["replay_position"]
        replay_q = pos["index"]["Espresso"] if pos and pos["kind"] == "Espresso" else None
        if replay_q is not None:
            need(verified is not None and replay_q <= verified["q"]
                 and pos["key_us"] == e_rows[replay_q]["theta_us"],
                 "signer replay position is not a verified Espresso block")
            local_g = [g for g in g_rows if g["q"] <= replay_q]
            head = local_g[-1]["height"] if local_g else 0
            need(detail["head_height"] == head, "signer head differs from public replay at its pc")
            checked += 1

        if detail["eligible"]:
            need(detail["clock_good"] and verified is not None and advertised == verified
                 and replay_q == verified["q"] and detail["tail"].upper() == "NEXT"
                 and detail["observation_us"] == verified["theta_us"]
                 and 0 <= detail["clock_sample_us"] - verified["theta_us"] <= 1_000_000,
                 "signer voted from unsafe clock, lagged view, or non-NEXT duty")
            prior = detail["previous"]
            need(prior is not None and prior["q"] < verified["q"]
                 and prior["theta_us"] < verified["theta_us"]
                 and prior["theta_us"] == e_rows[prior["q"]]["theta_us"]
                 and e_rows[prior["q"]]["finalized_us"] <= now,
                 "signer voted without public q and watermark progress")
            if signer in previous and previous[signer] is not None:
                need(prior == previous[signer], "claimed liveness compare is not the signer's preceding poll")
            need(detail["l1_verified"] == detail["l1_advertised"] and
                 public["l1"][detail["l1_verified"]]["timestamp_s"] * 1_000_000
                 >= max(0, verified["theta_us"] - ANNOUNCEMENT_US),
                 "signer voted with incomplete L1 control coverage")
            selected = evm_hex(detail["cid"])[2:]
            produced = [g for g in local_g if g["cid"] == selected]
            # The primary only signs during genesis duty, after its h1 contribution.
            need((detail["branch"], detail["region"], detail["head_height"]) == (0, 0, 1)
                 and produced and produced[-1]["height"] == 1,
                 "eligible signer has unexpected primary responsibility or contribution")
            progress = produced[-1]["theta_us"]
            need(detail["policy"]["enabled_mask"] == 6 and detail["policy"]["order"] == [0, 1, 2]
                 and detail["policy"]["signed_floor"] == 0,
                 "primary signer proposed policy not derived by disabling its own stalled responsibility")
            need(detail["progress_us"] == progress and verified["theta_us"] - progress >= 2_000_000,
                 "signer voted before the last selected-cid progress plus the stall threshold")
            need(detail["expiry_us"] == 42_000_000
                 and now <= detail["predicted_inclusion_us"]
                 and detail["predicted_inclusion_us"] + ANNOUNCEMENT_US + 6_750_000 < detail["expiry_us"],
                 "signer voted with insufficient time before next missing height expires")
        if (verified is not None and advertised == verified and replay_q == verified["q"]
                and detail["l1_verified"] == detail["l1_advertised"]):
            previous[signer] = verified
    need(len(set(row["detail"]["signer"] for row in polls)) == 3,
         "governance trace does not show three independent signer views")
    return polls, checked


def check_governance(mode, raw, trace, public, committee, controls, stages):
    polls, checked_views = check_views(trace, public)
    votes = [row for row in trace if row["kind"] == "GovernanceSignature"]
    broadcasts = [row for row in trace if row["kind"] == "GovernanceSubmission"]
    report = {"polls": len(polls), "espresso_replay_views": checked_views}
    if mode not in RECOVERY_MODES:
        need(not votes and not broadcasts and not any(r["detail"]["eligible"] for r in polls),
             "healthy/ordering control produced governance authorization")
        if mode != "healthy":
            need(any(r["tick_us"] >= FAULT_TICK + 2_000_000 and
                     r["detail"]["reason"] == "ORDERING_NOT_ADVANCING" for r in polls),
                 "missing ordering-specific suppression after the two-second observation interval")
            after_cut = [r for r in public["espresso"] if r["proposed_us"] >= FAULT_TICK]
            if mode == "ordering-stopped":
                need(not after_cut, "ordering-stopped fixture published new Espresso blocks")
            else:
                need(len(after_cut) >= 2 and len({r["theta_us"] for r in after_cut}) == 1,
                     "frozen watermark contrast must advance q at one theta")
            report["ordering_rows_after_fault"] = len(after_cut)
        return report
    control = controls[0]
    collect = {}
    for vote in votes:
        detail = vote["detail"]
        signer = detail["signer"]
        observed = [row for row in polls if row["detail"]["signer"] == signer and
                    row["detail"]["eligible"] and row["seq"] < vote["seq"]]
        need(observed, "governance vote lacks its signer's independently eligible poll")
        observed = observed[-1]
        need(vote["tick_us"] - observed["tick_us"] <= 10_000,
             "signer exceeded the 10ms completion budget")
        need(observed["detail"]["predicted_inclusion_us"] >= stages["include_us"],
             "actual inclusion exceeds the public decision bound")
        digest_bytes = crypto.as_bytes(evm_hex(detail["digest"]))
        signature = crypto.as_bytes(evm_hex(detail["signature"]))
        recovered = crypto.hx(crypto.recover_address(signature, digest_bytes))
        need(recovered == evm_hex(detail["address"])
             and crypto.as_bytes(recovered) in committee.members
             and evm_hex(detail["digest"]) == control["request_digest"]
             and recovered not in collect and signature not in (item[1] for item in collect.values()),
             "individual governance signature not independently bound to mined request")
        collect[recovered] = (vote, signature, observed)
    need(set(control["signers"]) <= set(collect),
         "mined signer did not supply an individual durable vote")
    need([crypto.hx(collect[member][1]) for member in control["signers"]] == control["signatures"],
         "signed-bundle bytes do not match individual retained committee votes")
    broadcast = one(broadcasts, "expected one automatic governance submission")
    need(evm_hex(broadcast["detail"]["hash"]) == control["tx_hash"]
         and evm_hex(broadcast["detail"]["digest"]) == control["request_digest"]
         and broadcast["tick_us"] == stages["submit_us"]
         and broadcast["detail"]["nonce"] == control["nonce"]
         and all(collect[signer][0]["seq"] < broadcast["seq"] for signer in control["signers"]),
         "automatic submit not caused by gathered signers / actual raw broadcast")

    eligible = set()
    eligible_at = None
    for row in polls:
        detail = row["detail"]
        if detail["eligible"]:
            eligible.add(detail["signer"])
            if len(eligible) >= committee.threshold:
                eligible_at = row["tick_us"]
                break
    need(eligible_at is not None and 0 <= stages["submit_us"] - eligible_at <= 120_000,
         "auto submission missed first quorum eligibility plus poll/sign/broadcast bound")
    if mode == "reader-lag":
        lagged = {r["detail"]["signer"] for r in polls
                  if r["tick_us"] >= 13_250_000 and r["seq"] < broadcast["seq"]
                  and r["detail"]["reason"] == "INPUTS_NOT_READY"
                  and r["detail"]["espresso_advertised"] != r["detail"]["espresso_verified"]}
        need(len(lagged) >= 2 and 13_650_000 <= eligible_at,
             "lagging independent views were not held pending until real input catch-up")
        report["lagged_signers"] = sorted(lagged)
    report["eligible_tick_us"] = eligible_at
    report["vote_ticks_us"] = {member: item[0]["tick_us"] for member, item in collect.items()}
    report["submit_tick_us"] = broadcast["tick_us"]
    report.update(check_submission_journal(raw, trace, control, broadcast))
    return report


def check_runs(args):
    repository, run, output = (p.resolve() for p in (args.repository, args.run, args.output))
    head = subprocess.check_output(["git", "-C", str(repository), "rev-parse", "HEAD"], text=True).strip()
    need(head == args.candidate, "checker checkout differs from candidate")
    status = subprocess.check_output(
        ["git", "-C", str(repository), "status", "--porcelain=v1", "--untracked-files=all"], text=True
    )
    need(not status, "checker checkout is dirty")
    manifest, manifest_sha = sealed(run, repository, head)
    need(manifest["schema"] == "M3-AUTO-FORCE-RUN-1" and manifest["scenario"] == args.mode,
         "wrong scenario manifest schema or mode")
    report = {"candidate": head, "mode": args.mode, "manifest_sha256": manifest_sha}
    if args.repeat:
        repeat = args.repeat.resolve()
        repeated, sha = sealed(repeat, repository, head)
        need(repeated["scenario"] == manifest["scenario"], "pause repeat changed the scenario")
        report["repeat"] = compare_repeat(run, repeat, manifest, repeated)
        report["repeat"]["manifest_sha256"] = sha
    raw = run / "raw"
    evm = load((raw / "l1/raw-rpc-bundle.json").read_bytes())
    committee, receipt_checks, controls = authority.verified_force(evm, sealed_committee_artifact(run, manifest))
    need((committee.threshold, len(committee.members)) == (2, 3), "expected default 2-of-3 deployment")
    report["committee"] = {"threshold": committee.threshold,
                           "members": [crypto.hx(member) for member in committee.members]}
    report["receipts"] = receipt_checks

    output.mkdir()
    controls_json = output / "verified-controls.json"
    headers_json = output / "verified-l1-headers.json"
    write_json(controls_json, controls)
    write_json(headers_json, authority.historical_headers(evm))
    public_binary = args.public_checker.resolve()
    report["public_checker_sha256"] = digest(public_binary)
    public = load(subprocess.check_output([
        str(public_binary), str(raw), str(controls_json), str(ANNOUNCEMENT_US), str(GRACE_US),
        str(EXPIRY_US), str(INIT_US), str(headers_json),
    ]))
    trace = list(m2.trace_rows(raw))
    need([row["tick_us"] for row in trace] == sorted(row["tick_us"] for row in trace),
         "virtual scheduler tick regressed")
    need(not any(row["kind"] == "RunFailed" for row in trace), "unexpected scenario failure")
    faulted = set(range(5)) if args.mode != "healthy" else set()
    grant_pids = {row["pid"] for row in trace if row["kind"] == "Grant"}
    need(set(range(10 if args.mode in RECOVERY_MODES else 5)) <= grant_pids,
         "responsible replicas missing controlled Grants")
    lifecycle = m2.check_frames(
        raw, public, repository / "poc/proto/native.proto", trace,
        grant_pids=grant_pids, faulted_pids=faulted,
    )
    report["lifecycle"] = lifecycle
    report["projection_grants"] = check_metadata_projection(trace, public, evm)
    need(report["projection_grants"] == lifecycle["counts"]["durable_acks"],
         "metadata projection check omitted worker grants")
    stages = None
    if args.mode in RECOVERY_MODES:
        control = one(controls, "exactly one canonical committee Force")
        stages = m2.driver_times(raw / "l1/hub-l1.jsonl", raw / "l1/driver-trace.json", evm,
                                control, bind_calls=bind_l1_driver, replay_commands=True)
        report["l1_stages"] = stages
        report["force"] = check_primary(trace, public, control, stages)
    else:
        need(not controls, "healthy/ordering comparison emitted a Force control")
        accepted = m2.acceptance_rows(trace, public)
        if args.mode == "healthy":
            need(public["expected"][-1]["height"] >= 10 and bool(public["regular"]),
                 "healthy comparison lacks h10 regular handoff")
            need(all(lifecycle["exits"][pid] > accepted[-1]["seq"] for pid in range(15)),
                 "healthy comparison had early process exit")
            report["healthy"] = {"last_g_tick_us": accepted[-1]["tick_us"], "handoffs": public["regular"]}
        else:
            need(len(accepted) == 1 and accepted[0]["detail"]["height"] == 1,
                 "ordering contrast progressed beyond frozen h1")
            last_poll = [row for row in trace if row["kind"] == "GovernancePoll"][-1]
            check_fault(trace, last_poll)
    report["governance"] = check_governance(args.mode, raw, trace, public, committee, controls, stages)
    write_json(output / "public-replay.json", public)
    report["status"] = "PASS"
    write_json(output / "report.json", report)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--repeat", type=Path)
    parser.add_argument("--public-checker", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--mode", choices=("outage", "healthy", "reader-lag", "ordering-stopped", "watermark-frozen"), required=True)
    args = parser.parse_args()
    need(__debug__, "the oracle requires Python assertions")
    check_runs(args)


if __name__ == "__main__":
    main()
