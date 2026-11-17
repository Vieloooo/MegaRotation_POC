#!/usr/bin/env python3
"""Independent receipt, native evidence and event checks for M2-HEALTHY-FORCE-1."""

import argparse
import hashlib
import importlib
import json
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path

from wire import NativeProto, fields, frames, h, identity_of_envelope, journal, load, only_variants, reference, single, zigzag


def require(ok, problem):
    if not ok:
        raise ValueError(problem)


def address(raw):
    return bytes(raw).hex()


def receipt_projection(repository, path):
    verification = repository / "poc/verification/anvil-l1"
    sys.path.insert(0, str(verification))
    codec = importlib.import_module("receipt_checker")
    evm = load(path.read_bytes())
    checker = codec.Checker(evm)
    base = checker.run()
    config = evm["config"]
    tx_at = {
        (int(block["number"], 16), int(tx["transactionIndex"], 16)): tx
        for block in evm["blocks"]
        for tx in block["transactions"]
    }
    tx_hashes = {tx["hash"].lower(): tx for tx in tx_at.values()}
    controls = []
    seen_request = {}
    for event in base["events"]:
        if event["kind"] == "CursorInitialized":
            continue
        require(event["kind"] == "ForceRecorded", "unexpected L1 batch/recovery activity")
        pos, seq = event["position"], event["seq"]
        tx = tx_at[tuple(pos[:2])]
        method, (rid, actual_seq, target, policy, _signature) = codec.decode_call(tx)
        require(method == "submitForce" and seq == actual_seq, "Force event/calldata projection")
        seen_request[codec.raw(tx["from"]), rid] = (seq, target, policy)
        controls.append({
            "position": pos,
            "seq": seq,
            "act_us": event["act"] * 1_000_000,
            "target": target,
            "l1_floor": policy[-1],
            "tx_hash": tx["hash"].lower(),
        })
    categories = Counter()
    for tx_hash in base["failedReceipts"]:
        tx = tx_hashes[tx_hash]
        call = codec.decode_call(tx)
        if not call or call[0] != "submitForce":
            continue
        sender = codec.raw(tx["from"])
        rid, seq, target, policy, _signature = call[1]
        try:
            request, actual_seq, _digest = checker.control_args("submitForce", call[1], sender)
        except ValueError:
            categories["signature"] += 1
            continue
        require(request == rid and seq == actual_seq, "Force rejection calldata parse")
        old = seen_request.get((sender, rid))
        if old is not None and old != (seq, target, policy):
            categories["conflicting_request"] += 1
        elif old is None and any(
            tuple(control["position"][:2]) < (int(tx["blockNumber"], 16), int(tx["transactionIndex"], 16))
            and control["seq"] >= seq
            for control in controls
        ):
            categories["stale_sequence"] += 1
        else:
            raise ValueError(f"unexplained failed control {tx_hash}")
    categories["exact_retry"] = len(base["idempotentRetries"])
    return evm, controls, {"receipt_report": base, "negative_and_retry_counts": dict(categories),
                           "config_ref": config["configRef"]}


def trace_rows(raw):
    with (raw / "trace.jsonl").open() as stream:
        for index, line in enumerate(stream, 1):
            row = load(line)
            require(index == row["seq"], "non-contiguous trace indexes")
            yield row


def check_frames(raw, public, proto_file, trace, *, grant_pids=None, faulted_pids=(), local_stop_ack_reuse=False):
    grant_pids = set(range(15)) if grant_pids is None else set(grant_pids)
    proto = NativeProto(proto_file)
    counts = Counter()
    pending = None
    steps = {}
    paired = defaultdict(list)
    binds = {}
    starts = {}
    terminal = {}
    signs = {}
    sent = set()
    observed = {}
    delivered = {}
    ack_event = {}
    work_due = {}
    completed_before_terminal = set()
    last_tick = 0
    post_close = []
    samples = {}
    target_heights = {bytes.fromhex(p["target"]): p["height"] for p in public["public_finals"]}
    admit = set()

    for direction, pid, raw_frame in frames(raw / "frames.bin"):
        if direction == 0:
            require(pending is None, "concurrent grants")
            grant = fields(raw_frame)
            require(grant[8] == h("I1b/fresh-grant/v1", only_variants(raw_frame, {8})), "grant hash mismatch")
            binding = fields(grant[1])
            require(binding[1] == binding[6] == 1 and binding.get(4, 0) == pid and binding.get(5, 0) == pid % 5,
                    "native worker binding mismatch")
            require(binds.setdefault(pid, binding) == binding, "worker incarnation/binding changed")
            tick = grant.get(4, 0)
            require(grant[2] not in steps and tick >= last_tick, "grant replay/regression")
            last_tick = tick
            kind, data = single(grant[7])
            p = fields(data)
            if kind == 3:
                key = pid, p[1]
                require(key not in starts and key not in terminal, "old cid started again")
                starts[key] = (tick, zigzag(p[9]))
            elif kind == 2:
                require(p.get(8, 0) <= tick < p[11], "clock sample time/lease bound")
                require(abs(zigzag(p.get(9, 0)) - p.get(8, 0)) <= 1000, "clock sample error")
                samples[pid] = p
            elif kind in (5, 6, 7, 8):
                cid = p[1] if kind == 8 else p[2]
                if kind == 5:
                    require(identity_of_envelope(p) in sent, "delivery outside committed route")
                if (pid, cid) in terminal and kind != 8:
                    probe = {"pid": pid, "cid": cid.hex(), "input_tag": kind, "event": grant[3], "tick_us": tick}
                    if kind == 7:
                        key = (pid, p[1])
                        probe["operation_id"] = p[1].hex()
                        probe["already_completed"] = key in completed_before_terminal
                        probe["scheduled_due_us"] = work_due[key]
                    post_close.append(probe)
                elif kind == 7:
                    completed_before_terminal.add((pid, p[1]))
            pending = pid, grant, raw_frame
            continue

        require(direction == 1 and pending is not None and pending[0] == pid, "unexpected Ack")
        _, grant, raw_grant = pending
        pending = None
        ack = fields(raw_frame, 6)
        seq = ack[10]
        tick = grant.get(4, 0)
        require((ack[1], ack[2], ack[3], ack[4]) == (grant[1], grant[2], grant[3], grant[8]),
                "Ack/grant binding")
        require(ack[5] in (1, 2) and seq == len(paired[pid]) + 1, "Ack status/durable sequence")
        require(ack[12] == h("I1b/fresh-ack/v1", only_variants(raw_frame, {11, 12})), "Ack hash mismatch")
        require(grant[3] not in ack_event, "duplicate granted event")
        steps[grant[2]] = (pid, grant, ack)
        paired[pid].append((grant, ack, raw_grant, raw_frame))
        ack_event[grant[3]] = False
        application = load(ack.get(8, b"{}"))
        for work in application.get("work", []):
            if work["event"] == "work_completed" and work["result_valid"]:
                view = work["binding"]["view"]
                digest = bytes(work["body_digest"])
                kind = work["binding"]["work"]["kind"]
                if kind in (1, 2):
                    admit.add((pid, view, digest))
        if post_close and post_close[-1]["event"] == grant[3] and post_close[-1]["input_tag"] == 7:
            probe = post_close[-1]
            probe["discarded"] = any(work["event"] == "work_discarded"
                and work["reason"] == "unknown_or_retired_work"
                and bytes(work["work"]["operation_id"]).hex() == probe["operation_id"]
                for work in application.get("work", []))
            require(probe["discarded"] and all(single(effect)[0] not in (5, 6) for effect in ack[6]),
                    "closed cid completion reached a native send/sign output")
        for effect in ack[6]:
            tag, data = single(effect)
            p = fields(data)
            if tag == 4:
                key = (pid, p[1])
                require(key not in work_due, "native work issued twice")
                work_due[key] = tick + p[4]
            elif tag == 5:
                require((pid, p[2]) not in terminal, "new network output from terminal cid")
                sent.add(identity_of_envelope(p))
            elif tag == 6:
                cid, epoch, view, kind, action = (p[n] for n in range(1, 6))
                require(epoch == 1 and 1 <= view <= 128, "signature coordinates")
                require(p.get(9, 0) == grant[2] and p.get(11, 0) == tick, "signature outside Grant")
                if action == 3:
                    require(not p.get(7), "denied sign emitted bytes")
                    if (pid, cid) in terminal:
                        counts["terminal_denials"] += 1
                    continue
                require(action in (1, 2) and p[7], "unexpected sign action")
                require((pid, cid) in starts and (pid, cid) not in terminal,
                        "new or cached signing output after terminal or before Enter")
                require(p.get(8, 0) == public["signer_indexes"][pid], "signature signer/process mismatch")
                local = zigzag(p.get(12, 0))
                stop = zigzag(p.get(15, 0))
                sample = samples[pid]
                require(local < stop == starts[pid, cid][1] and abs(local - tick) <= 1000, "local stop or clock bound")
                require(tick < p[14] == sample[11] and p[13] == sample[7], "sample lease")
                require(local == zigzag(sample.get(9, 0)) + tick - sample.get(8, 0), "clock formula")
                target = p[6][-32:]
                if kind == 1 and target in target_heights:
                    require((pid, view, target) in admit, "notarize before successful local admission")
                    require(local >= (10 + target_heights[target]) * 1_000_000, "signing before NotBefore")
                coordinate = pid, cid, view, kind
                record = p[6], p[7], tick
                if action == 1:
                    require(coordinate not in signs, "recreated signature coordinate")
                    signs[coordinate] = record
                else:
                    require(signs[coordinate][:2] == record[:2], "cache hit does not reuse signature bytes")
            elif tag == 7:
                require(p[2] == 1, "proof epoch")
                observed.setdefault((p[1].hex(), p[3], p[4].hex(), p[5].hex()), (tick, pid, grant[3]))
            elif tag == 8:
                require(p[3] == h("PoC/PhysicalBlock/v1", p[5]), "body hash mismatch")
                delivered.setdefault((p[1].hex(), p[3].hex()), (tick, pid, grant[3]))
            elif tag == 9 and p[2] == b"terminated":
                key = pid, p[1]
                require(key not in terminal, "repeated terminal effect")
                terminal[key] = (tick, grant[2], grant[3], p.get(3, b"").decode())
        counts["acks"] += 1
    require(pending is None and set(binds) == grant_pids, "lost Ack or expected worker grant")
    for pid in sorted(grant_pids):
        entries = list(journal(reference(raw, pid, "root-step"), b"I1SHEAD1", "root-step", binds[pid]))
        for (index, digest, disk), (grant, ack, wire_grant, wire_ack) in zip(entries, paired[pid], strict=True):
            require(index == ack[10] and digest == ack[11], "Ack durable record identity")
            expected_grant = proto.decode("Grant", wire_grant)
            expected_ack = proto.decode("Ack", wire_ack)
            expected_ack["durable_step_hash"] = []
            if expected_ack["recovery_receipt"] is None:
                expected_ack.pop("recovery_receipt")
            require(disk["grant"] == expected_grant and disk["ack"] == expected_ack,
                    "durable Grant/Ack differs from raw issued/delivered bytes")
            counts["durable_acks"] += 1

    for p in public["public_finals"]:
        tup = p["cid"], p["view"], p["target"], p["proof"]
        require(tup in observed, "public native Final does not match any committed worker proof")
        require((p["cid"], p["target"]) in delivered, "public result lacks controlled regional body delivery")
        for signer in p["signers"]:
            subject, _, _ = signs[signer, bytes.fromhex(p["cid"]), p["view"], 3]
            require(subject.hex() == p["subject"], "native Finalization aggregate subject differs from signer audit")
        p["observed"] = observed[tup]
        p["delivered"] = delivered[p["cid"], p["target"]]

    released = {}
    active = None
    spawn, exits = {}, {}
    close_intents = {}
    local_stops = {}
    for r in trace:
        kind, pid, seq, detail = [r[k] for k in ("kind", "pid", "seq", "detail")]
        if kind == "WorkerSpawn":
            require(pid not in spawn, "worker respawn in healthy run")
            spawn[pid] = detail["os_pid"], seq
        elif kind == "WorkerExit":
            require(pid not in exits and spawn[pid][0] == detail["os_pid"], "unexpected worker exit identity")
            if pid not in faulted_pids:
                require(detail["exit"] == 0 and detail["expected"], "unplanned worker exit")
            exits[pid] = seq
        elif kind == "ManagerCommitted":
            revision = detail["revision"]
            durable = (raw / f"worker-{pid}/manager/{revision:04}.json").read_bytes()
            expected = {k: v for k, v in detail.items() if k != "commit_ref"}
            expected["event_id"] = r["event_id"]
            require(load(durable) == expected, "Manager commit trace differs from durable decision")
            require(address(detail["commit_ref"]) == h("PoC/M1/manager-commit/v1", durable).hex(),
                    "Manager commit does not bind terminal decision bytes")
            for intent in expected["intents"]:
                if intent["kind"].startswith("Close {"):
                    action = json.dumps(intent["id"], sort_keys=True)
                    require(action not in close_intents, "duplicate Close intent")
                    close_intents[action] = (seq, intent["kind"])
            counts["durable_manager_commits"] += 1
        elif kind == "Grant":
            index = detail["logical_step_id"]
            actual_pid, grant, _ = steps[index]
            require(active is None and pid == actual_pid and detail["grant_hash"] == list(grant[8]),
                    "trace Grant mismatch")
            require(r["event_id"] == grant[3] and r["tick_us"] == grant.get(4, 0), "trace Grant position")
            active = index
        elif kind == "AckCommitted":
            index = detail["logical_step_id"]
            actual_pid, grant, ack = steps[index]
            require(active == index and pid == actual_pid and index not in released, "trace Ack order")
            require(r["event_id"] == grant[3] and r["tick_us"] == grant.get(4, 0), "trace Ack position")
            require(detail["root_hash"] == list(ack[11]) and detail["effects"] == len(ack[6])
                    and detail["durable_step_sequence"] == ack[10], "trace Ack durable boundary")
            released[index] = seq
            active = None
        elif kind == "LocalStopped" and local_stop_ack_reuse:
            cid, index = bytes(detail["cid"]), detail["logical_step_id"]
            actual_pid, grant, ack = steps[index]
            require(actual_pid == pid and grant[3] == r["event_id"]
                    and grant.get(4, 0) == r["tick_us"] and list(ack[11]) == detail["ack"]
                    and released[index] < seq and (pid, cid) not in local_stops
                    and terminal[pid, cid] == (r["tick_us"], index, r["event_id"], "LocalStop"),
                    "local stop marker differs from its unique durable native terminal")
            local_stops[pid, cid] = seq, index
        elif kind in {"NetworkAdmitted", "ProofObserved", "RegionalDelivered", "Ready", "Enter", "Close",
                      "CutoffRetire", "PublishScheduled", "CloseProbe"} and "logical_step_id" in detail:
            index = detail["logical_step_id"]
            actual_pid, grant, ack = steps[index]
            require(released[index] < seq and pid == actual_pid, "release before persisted Ack")
            reused = local_stop_ack_reuse and kind == "Close" and detail["cause"] == "LocalTime"
            if reused:
                stop_seq, stop_index = local_stops[pid, bytes(detail["cid"])]
                require(stop_index == index and stop_seq < seq and r["tick_us"] >= grant.get(4, 0)
                        and list(ack[11]) == detail["ack"], "Close reused a nonterminal/unrelated Ack")
            else:
                require(r["event_id"] == grant[3] and r["tick_us"] == grant.get(4, 0), "output mismatches grant")
            if "ack" in detail:
                require(detail["ack"] == list(ack[11]), "output mismatches durable root")
            if kind == "Close":
                cid = bytes(detail["cid"])
                require(terminal[pid, cid][1:3] == (index, grant[3]), "Close trace differs from worker terminal")
                action = json.dumps(detail["action"], sort_keys=True)
                intent_seq, intent_body = close_intents[action]
                ordered = stop_seq < intent_seq < seq if reused else intent_seq < released[index]
                require(ordered and detail["cause"] in intent_body,
                        "Close released before terminal Manager commit or unrelated local stop")
                counts["durable_close_intents"] += 1
            counts["after_ack_outputs"] += 1
    require(active is None and len(released) == len(steps), "trace omits grants or Ack commits")
    require(set(spawn) == set(exits) == set(range(15)) and len({p[0] for p in spawn.values()}) == 15,
            "must run all 15 worker PIDs")
    return {
        "counts": dict(counts),
        "spawn": spawn,
        "exits": exits,
        "terminal": [{"pid": pid, "cid": cid.hex(), "tick_us": at[0], "step": at[1],
                      "event": at[2], "reason": at[3]} for (pid, cid), at in terminal.items()],
        "post_close_grants": post_close,
        "signed_count": len(signs),
    }


def load_rows(path):
    data = path.read_text()
    if data.lstrip().startswith("["):
        return load(data)
    return [load(line) for line in data.splitlines() if line.strip()]


def bind_hub_calls(path, transcript):
    # A Hub operation always begins with a Tick, including multiple operations at one tick.
    calls = []
    for row in transcript:
        if row["command"]["op"] == "tick":
            calls.append([row])
        else:
            calls[-1].append(row)
    for step, group in zip(load_rows(path), calls, strict=True):
        hub, result = step["command"], step["result"]
        tick, height, reader = hub["tick_us"], hub["height"], hub["reader"]
        require(all(row["reply"]["status"] == "ready" and row["reply"]["reference_us"] == tick for row in group),
                "Hub step does not match ready driver replies at the scheduled virtual time")
        require(group[0]["command"]["reference_us"] == tick, "Hub and driver tick disagree")
        rest = group[1:]
        op = hub["op"]
        operations = [row["command"]["op"] for row in rest]
        replies = [row["reply"]["result"] for row in rest]
        if op == "mine":
            require(operations == ["mine"] and rest[0]["command"]["slot"] == height and
                    replies[0]["included_block"] == result, "Hub inclusion evidence")
        elif op == "finalize":
            require(operations == ["finalize"] and rest[0]["command"]["through_height"] == height and
                    replies == [result] and result["finalized_through"] == height, "Hub finalization evidence")
        elif op in {"genesis", "deliver"}:
            expected = ["read"] if op == "genesis" else ["deliver", "read"]
            require(operations == expected and rest[-1]["command"]["reader"] == reader and
                    rest[-1]["command"]["query"] == {"kind": "block", "height": height} and
                    replies[-1]["control_coverage_through"] == height and replies[-1]["block"] == result,
                    "Hub reader receipt evidence")
            if op == "deliver":
                require(replies[0]["delivered_through"] == height and replies[0]["reader"] == reader,
                        "missing driver finalized reader delivery")
        elif op in {"deploy", "submit", "retry"}:
            require(operations == ["submit"] * len(result) and
                    [r["submitted_tx_hash"] for r in replies] == [r["hash"] for r in result],
                    "Hub submitted transaction result")
        else:
            raise ValueError(f"unexpected Hub L1 operation: {op}")
    return len(calls)


def driver_times(hub_path, rpc_path, evm, control, *, bind_calls=bind_hub_calls, replay_commands=False):
    rpc = load_rows(rpc_path)
    virtual = [row for row in rpc if "command" in row]
    hub_operations = bind_calls(hub_path, virtual)
    transcript = [r["startup_rpc"] for r in rpc if "startup_rpc" in r]
    require(len(transcript) == 1, "missing unique Anvil startup evidence")
    automine = [r for r in transcript[0] if r["request"]["method"] == "anvil_getAutomine"]
    require(len(automine) == 1 and automine[0]["response"]["result"] is False, "manual mining guard")
    chain = [r for r in transcript[0] if r["request"]["method"] == "eth_chainId"]
    require(len(chain) == 1 and int(chain[0]["response"]["result"], 16) == evm["config"]["chainId"],
            "RPC chain ID")
    ids = [entry["reply"]["id"] for entry in rpc if "reply" in entry]
    require(replay_commands or len(set(ids)) == len(ids), "duplicate driver reply")
    imported = importlib.import_module("independent_codec")
    including_height = control["position"][0]
    canonical_block = evm["blocks"][including_height]
    tx = canonical_block["transactions"][control["position"][1]]
    stages = {}
    readers = {}
    last_tick = 0
    last_heights = 0, 0
    reader_prior = {}
    all_mined = set()
    for row in virtual:
        cmd, output = row["command"], row["reply"]
        reply = row
        require(output["id"] == cmd["id"], "virtual driver command and raw RPC transcript disagree")
        now = output["reference_us"]
        included = output["included_height"]
        finalized = output["finalized_height"]
        require(now >= last_tick and 0 <= finalized <= included and
                included >= last_heights[0] and finalized >= last_heights[1], "driver prefix/regression")
        last_tick = now
        last_heights = included, finalized
        for reader, height in output["visible_heights"].items():
            require(reader_prior.get(reader, 0) <= height <= finalized, "reader saw unfinalized or regressing prefix")
            reader_prior[reader] = height
        calls = reply["rpc"]
        methods = [call["request"]["method"] for call in calls]
        if output["status"] != "ready":
            if replay_commands and cmd["op"] == "submit":
                submitted = imported.hx(imported.keccak256(imported.as_bytes(cmd["raw"])))
                if submitted == control["tx_hash"]:
                    [send] = [c for c in calls if c["request"]["method"] == "eth_sendRawTransaction"]
                    require(send["request"]["params"] == [cmd["raw"]],
                            "uncertain Force send did not submit the prepared bytes")
                    stages.setdefault("submit_us", now)
            continue
        if cmd["op"] == "submit" and output["result"].get("submitted_tx_hash", "").lower() == control["tx_hash"]:
            first_raw_send = "eth_sendRawTransaction" in methods
            if first_raw_send and "submit_us" not in stages:
                [send] = [call for call in calls if call["request"]["method"] == "eth_sendRawTransaction"]
                require(send["request"]["params"] == [cmd["raw"]] and
                        send["response"]["result"].lower() == control["tx_hash"], "Force raw tx submission mismatch")
                raw_hash = imported.hx(imported.keccak256(imported.as_bytes(cmd["raw"])))
                require(raw_hash == control["tx_hash"], "mined transaction hash does not bind raw input")
                stages["submit_us"] = now
        if cmd["op"] == "mine":
            require("evm_mine" in methods and "evm_setNextBlockTimestamp" in methods,
                    "driver mine must record real EVM calls")
            slot = cmd["slot"]
            require(slot not in all_mined, "duplicated successful L1 mine")
            all_mined.add(slot)
            if slot == including_height:
                stages["include_us"] = now
                expected_block = False
                expected_receipt = False
                for event in calls:
                    method = event["request"]["method"]
                    obj = (event["response"] or {}).get("result")
                    if not isinstance(obj, dict):
                        continue
                    if method == "eth_getBlockByNumber" and obj.get("hash") == canonical_block["hash"]:
                        require(obj == canonical_block, "mined header/transaction inventory differs")
                        expected_block = True
                    if method == "eth_getTransactionReceipt" and obj.get("transactionHash") == tx["hash"]:
                        matched = [r for r in evm["receipts"] if r["transactionHash"] == tx["hash"]]
                        require(matched == [obj], "actual receipt differs from raw evidence bundle")
                        expected_receipt = True
                require(expected_block and expected_receipt, "including mine lacks raw RPC header or Force receipt")
        if cmd["op"] == "finalize" and finalized >= including_height:
            stages.setdefault("finalize_us", now)
            require("evm_mine" not in methods, "finalize emitted a mine")
        if cmd["op"] == "deliver" and output["visible_heights"][cmd["reader"]] >= including_height:
            readers.setdefault(cmd["reader"], now)
    require(set(stages) == {"submit_us", "include_us", "finalize_us"} and len(readers) >= 1,
            "missing separated Force L1 stages")
    require(stages["submit_us"] < stages["include_us"] < stages["finalize_us"] <= min(readers.values()),
            "misordered Force L1 stages")
    require(min(readers.values()) - stages["include_us"] <= 2_000_000, "L1 finalized reader availability exceeds 2s")
    return {**stages, "reader_us": readers, "checked_hub_operations": hub_operations}


def acceptance_rows(trace, public):
    accepted = [r for r in trace if r["kind"] in ("GAccepted", "GPadding")]
    require(len(accepted) == len(public["expected"]), "production and independent G lengths differ")
    for got, expected in zip(accepted, public["expected"], strict=True):
        row = got["detail"]
        require(row["height"] == expected["height"], "G height differs")
        require(address(row["cid"]) == expected["cid"], "G cid differs")
        require(address(row["native_hash"]) == expected["native"], "G native hash differs")
        require(address(row["state_root"]) == expected["state_root"], "G state root differs")
        if expected["physical"] is not None:
            require(got["kind"] == "GAccepted", "physical/padding result misclassified")
            require(address(row["physical_id"]) == expected["physical"], "G physical head differs")
            require(address(row["parent"]) == expected["parent"], "G physical parent differs")
        else:
            require(got["kind"] == "GPadding" and address(row["physical_id"]) == expected["target"],
                    "padding must cite its admitting native Final, without a synthetic body")
        pos = row["position"]
        require(pos["kind"] == "Espresso" and pos["index"] == {"Espresso": expected["q"]}
                and pos["key_us"] == expected["theta_us"], "G public position differs")
    return accepted


def check_primary(trace, public, controls, lifecycle, stages, evm):
    require(len(controls) == 1, "primary run must have exactly one L1 control log")
    [control] = controls
    require(control["position"][0] == 6 and control["act_us"] == 22_000_000 and control["target"] == 0,
            "primary Force does not use frozen inclusion/act/branch")
    [force] = public["force"]
    require(force["valid"] and force["anchor"]["block"]["height"] == 10 and
            (force["closed_beta"], force["beta"], force["r"], force["region"], force["source_region"]) == (0, 1, 1, 2, 1),
            "Force applied to incorrect actual activation state")
    require((force["h_min"], force["h_max"], force["latest_ready_us"], force["expiry_key_us"]) ==
            (16, 25, 25_999_000, 29_000_000), "Force window or budget differs from freeze")
    [regular] = public["regular"]
    require((regular["beta"], regular["r"], regular["anchor"]["block"]["height"]) == (0, 0, 10),
            "in-flight regular expected at h10")
    accepted = acceptance_rows(trace, public)
    require(accepted[0]["tick_us"] < stages["submit_us"] < accepted[9]["tick_us"] < control["act_us"],
            "Force was not submitted during healthy R0 before in-flight full window")
    [replayed] = [r for r in trace if r["kind"] == "GfrReplayRecord" and r["detail"]["position"]["kind"] == "Force"]
    pos = replayed["detail"]["position"]
    require(pos["key_us"] == force["act_us"], "Force replay position time mismatch")
    require(pos["index"] == {"L1": {"block": control["position"][0], "tx": control["position"][1],
                                    "log": control["position"][2]}}, "Force replay L1 position mismatch")
    lifted = replayed["detail"]["input"]
    require(lifted == {"position": pos["index"]["L1"], "control_seq": control["seq"], "target_beta": control["target"]},
            "GFR input does not match the independently decoded Force record")
    [activated] = [r for r in trace if r["kind"] == "ForceActivated"]
    supplied = activated["detail"]
    desc = supplied["descriptor"]
    plan = desc["plan"]
    require(supplied["position"] == pos and activated["tick_us"] == replayed["tick_us"],
            "Force activation summary differs from public replay record")
    require(supplied["old_branch"] == 0 and supplied["new_branch"] == 1 and supplied["head_height"] == 10
            and desc["anchor"] == force["anchor"] and supplied["anchor_native_hash"] == force["native_hash"],
            "Force actual selected descriptor/retained state mismatch")
    require(plan["session"] == {"beta": 1, "r": 1} and plan["region"] == 2 and plan["source_region"] == 1
            and plan["window"] == {"h_min": 16, "h_max": 25} and plan["reference_block"] == force["anchor"]["block"],
            "Force Plan derived from wrong activation-time inputs")
    require(supplied["l1_floor"] == force["floor"], "Force public L1 floor mismatch")
    q_star = public["espresso"][force["q_star"]]
    now = replayed["tick_us"]
    require(min(stages["reader_us"].values()) <= now and q_star["finalized_us"] <= now,
            "activation occurred before joint finalized reader coverage")
    require(
        public["espresso"][force["q_star"] - 1]["theta_us"] < force["act_us"] <= q_star["theta_us"],
        "independent q* public watermark mismatch",
    )
    require(q_star["finalized_us"] - q_star["proposed_us"] <= 1_000_000 and
            abs(q_star["theta_us"] - q_star["proposed_us"]) <= 1_000_000, "watermark service/clock budget")
    require(now - max(q_star["finalized_us"], min(stages["reader_us"].values())) <= 100_000,
            "activation replay exceeded 100ms budget")
    require(replayed["detail"]["append"] == [], "Force directly changed the G prefix")
    events = replayed["detail"]["events"]
    require(len(events) == 2 and events[0]["body"] == {"BranchClosed": {"branch": 0, "reason": "Force"}}
            and set(events[1]["body"]) == {"SessionAssigned"} and all(e["position"] == pos for e in events),
            "Force replay must close the old branch and assign in one ordered public record")
    require(accepted[9]["seq"] < replayed["seq"] < accepted[10]["seq"], "Force trace/appended old prefix order")
    entries = public["expected"]
    require([row["height"] for row in entries] == list(range(1, 17)), "frozen first-result cutoff")
    require([row["height"] for row in entries if row["physical"] is None] == [11, 12, 13, 14, 15],
            "actual Force gap must be filled by first new result only")
    first = entries[-1]
    require(first["beta"] == 1 and first["r"] == 1 and first["region"] == 2 and
            first["parent"] == address(force["anchor"]["block"]["block_id"]) and
            first["theta_us"] < force["expiry_key_us"], "first new physical result responsibility/expiry")
    require(first["q"] >= force["q_star"], "new result entered before Force")
    for pid in range(15):
        require(lifecycle["spawn"][pid][1] < replayed["seq"] < accepted[-1]["seq"] < lifecycle["exits"][pid],
                "15 workers must remain alive throughout activation and first result")
    ready = {}
    enter = {}
    cid = first["cid"]
    require(address(desc["cid"]) == cid, "G first new physical must follow actual Force assignment")
    for event in trace:
        detail = event["detail"]
        if event["kind"] in {"Ready", "Enter"} and address(detail["cid"]) == cid:
            pid = event["pid"]
            dest = ready if event["kind"] == "Ready" else enter
            require(pid not in dest, "repeated new-cid Ready/Enter")
            dest[pid] = event["tick_us"]
            if dest is ready:
                require(detail["anchor"] == force["anchor"] and detail["state_root"] == force["anchor_root"],
                        "new-cid preparation did not use actual G state")
    require(set(enter) == set(ready) == set(range(10, 15)), "all five new R2 members must Ready/Enter")
    require(len(set(enter.values())) > 1 and len([p for p in enter if enter[p] < force["latest_ready_us"]]) >= 4,
            "ready/entered deadline or staggered start")
    require(all(now <= ready[p] <= enter[p] and ready[p] - now <= 500_000 for p in enter),
            "state preparation budget or Ready-before-Enter")
    terminal = {(row["pid"], row["cid"]): row for row in lifecycle["terminal"]}
    old_r0 = entries[9]["cid"]
    require(all((pid, old_r0) in terminal and terminal[pid, old_r0]["tick_us"] < now for pid in range(5)),
            "old R0 closed before Force by regular")
    late = [r for r in public["late"] if r["height"] == 11]
    require(late, "old valid higher h11 must be publicly complete after Force")
    old_r1 = late[0]["cid"]
    require(all((pid, old_r1) in terminal and now <= terminal[pid, old_r1]["tick_us"] < accepted[-1]["tick_us"]
                for pid in range(5, 10)), "Force terminal missing on old R1")
    for p in late:
        proofs = [r for r in public["public_finals"] if r["cid"] == p["cid"] and r["target"] == p["physical"]]
        require(any(r["observed"][0] < now and r["q_avail_theta_us"] >= force["act_us"] for r in proofs),
                "h11 must use pre-close actual native proof and late public completion")
    require(not any(row["cid"] == old_r1 for row in entries), "old h11 or descendant entered G")
    probes = [row for row in lifecycle["post_close_grants"] if row["cid"] == old_r1]
    require({p["pid"] for p in probes if p.get("discarded")} == set(range(5, 10)),
            "old-committee post-close Work replay not bound to raw discarded callbacks")
    for kind, key in (("L1Submitted", "submit_us"), ("L1Included", "include_us"), ("L1Finalized", "finalize_us")):
        matches = [r for r in trace if r["kind"] == kind and r["tick_us"] == stages[key]]
        if kind == "L1Submitted":
            matches = [r for r in matches if r["detail"]["hash"].lower() == control["tx_hash"]
                       and r["detail"]["reason"] == "scheduled_healthy_force"]
        else:
            matches = [r for r in matches if r["detail"]["height"] == 6]
        require(matches, f"{kind} not bound to verified raw L1 stage")
    projected = set()
    reader_tip = {}
    projection_grants = 0
    for event in trace:
        detail = event["detail"]
        if event["kind"] == "L1ReaderAvailable":
            reader, height = detail["reader"], detail["height"]
            require(reader_tip.get(reader, -1) + 1 == height, "finalized L1 reader is not a continuous prefix")
            reader_tip[reader] = height
            require(address(detail["hash"]) == evm["blocks"][height]["hash"][2:].lower(),
                    "L1 reader hash differs from real canonical RPC data")
            if height == 6 and event["tick_us"] == stages["reader_us"].get(reader):
                projected.add(reader)
        elif event["kind"] == "Grant":
            eligible = [r["height"] for r in public["l1"] if r["finalized_us"] + 1 <= event["tick_us"]]
            horizon = max(eligible, default=-1)
            require(detail["worker_l1_through"] == horizon and reader_tip.get("A", -1) >= horizon,
                    "worker metadata visibility was not constrained by actual Anvil reader A")
            projection_grants += 1
    require(projected == set(stages["reader_us"]), "live reader delivery not bound to finalized L1 driver evidence")
    require(projection_grants == lifecycle["counts"]["acks"], "projection gate omitted worker grants")
    new = [r for r in public["public_finals"] if r["target"] == first["target"] and r["cid"] == cid]
    require(new, "first new accepted physical result lacks verified finality")
    first_native = min(r["observed"][0] for r in new)
    pack_tick = min(r["delivered"][0] for r in new)
    require(0 <= pack_tick - first_native <= 50_000, "finality-to-regional result exceeded pack bound")
    q_first = public["espresso"][first["q"]]
    require(0 <= q_first["proposed_us"] - pack_tick <= 200_000, "regional result-to-publication delay")
    require(0 <= accepted[-1]["tick_us"] - q_first["finalized_us"] <= 100_000, "result replay budget")
    require(25_999_000 <= first_native <= 26_500_000, "first production exceeded declared budget")
    publisher = [r for r in trace if r["kind"] == "PublisherRecorded" and
                 address(r["detail"]["physical_id"]) == first["target"] and r["detail"]["q"] == first["q"]]
    require(len(publisher) == 1 and publisher[0]["seq"] < accepted[-1]["seq"], "missing exact Publisher transition")
    require(q_first["finalized_us"] - q_first["proposed_us"] <= 1_000_000 and
            abs(q_first["theta_us"] - q_first["proposed_us"]) <= 1_000_000, "new Espresso finality/deviation")
    require(accepted[-1]["tick_us"] < 29_000_000, "observed first new G misses virtual recovery deadline")
    return {
        "activation": {"key_us": force["act_us"], "tick_us": now, "trace_seq": replayed["seq"], "q_star": force["q_star"]},
        "first_new": first, "first_new_g_tick_us": accepted[-1]["tick_us"],
        "last_old": entries[9], "late": late, "ready_us": ready, "enter_us": enter,
        "post_close_probes": probes, "first_native_us": first_native, "pack_us": pack_tick,
        "checked_projected_worker_grants": projection_grants,
        "first_espresso": q_first,
    }


def check_regular(trace, public, lifecycle):
    require(not public["force"], "M1 regression has Force")
    require([r["anchor"]["block"]["height"] for r in public["regular"]] == [10, 20],
            "M1 regular anchor progression")
    require([r["q"] for r in public["regular"]] == [9, 19], "M1 handoff public position")
    accepted = acceptance_rows(trace, public)
    require([r["height"] for r in public["expected"]] == list(range(1, 22)), "M1 contiguous full window")
    require(all(lifecycle["exits"][pid] > accepted[-1]["seq"] for pid in range(15)), "early M1 exit")
    return {"last_g_tick_us": accepted[-1]["tick_us"], "last_g": public["expected"][-1]}


def digest(path):
    sha = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1 << 20):
            sha.update(chunk)
    return sha.hexdigest()


def sealed_manifest(path, candidate, repository, raw):
    content = path.read_bytes()
    sha = hashlib.sha256(content).hexdigest()
    require(path.with_suffix(".sha256").read_text().split() == [sha, path.name], "manifest SHA-256 seal mismatch")
    manifest = load(content)
    require(manifest["candidate"] == candidate and manifest["seed"] == 42 and manifest["status"] == "COMPLETED",
            "sealed run status/candidate/seed mismatch")
    if manifest["schema"] == "M2-HEALTHY-FORCE-RUN-1":
        before, after = manifest["before"], manifest["after"]
        require(before == after and before["head"] == candidate and before["status"] == "",
                "M2 checkout was dirty or changed during run")
        require(before["bft_gitlink"] == "691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6"
                and manifest["commonware"] == "2026.10.0", "M2 pinned sources")
        for name, sha_expected in manifest["inputs"].items():
            require(digest(repository / name) == sha_expected, f"candidate frozen input mismatch: {name}")
        inventory = manifest["files"]
    elif manifest["schema"] == "M1-SMOKE-RUN-1":
        before, after = manifest["revision_before"], manifest["revision_after"]
        require(before == after and before["head"] == candidate and before["status"] == [],
                "M1 checkout was dirty or changed during run")
        inventory = {r["path"]: r for r in manifest["artifacts"]}
        require(len(inventory) == len(manifest["artifacts"]), "duplicate M1 inventory entries")
    else:
        raise ValueError("unsupported run manifest schema")
    run = path.parent.resolve()
    for name, expected in inventory.items():
        item = (run / name).resolve()
        require(item.is_relative_to(run) and item.is_file() and item.stat().st_size == expected["bytes"]
                and digest(item) == expected["sha256"], f"sealed evidence mismatch: {name}")
    if raw is not None:
        require(raw.resolve() == run / "raw", "raw evidence directory is outside sealed manifest")
        actual_raw = {str(p.relative_to(run)) for p in raw.resolve().rglob("*") if p.is_file()}
        require(actual_raw == {name for name in inventory if name.startswith("raw/")},
                "sealed raw inventory differs from actual files")
    return manifest, sha



def normalized_trace(raw):
    normalized = []
    for row in trace_rows(raw):
        detail = row["detail"]
        if row["kind"] == "WorkerSpawn":
            for field in ("os_pid", "proc_stat", "wall_time_ns"):
                detail.pop(field, None)
        elif row["kind"] == "WorkerExit":
            for field in ("os_pid", "wall_time_ns"):
                detail.pop(field, None)
        normalized.append(json.dumps(row, sort_keys=True, separators=(",", ":")))
    return ("\n".join(normalized) + "\n").encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--receipts", type=Path)
    parser.add_argument("--hub-l1", type=Path)
    parser.add_argument("--driver-rpc", type=Path)
    parser.add_argument("--repeat-raw", type=Path)
    parser.add_argument("--raw", type=Path)
    parser.add_argument("--public-checker", type=Path)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--mode", choices=("force", "regular", "abi"), default="force")
    args = parser.parse_args()
    candidate = subprocess.check_output(
        ["git", "-C", str(args.repository), "rev-parse", "HEAD"], text=True
    ).strip()
    require(candidate == args.candidate, "candidate revision mismatch")
    args.output.mkdir()
    require(__debug__, "checks require Python assertions enabled")
    manifest = args.manifest or (args.raw.parent / "manifest.json" if args.raw else None)
    require(manifest is not None, "sealed run manifest required")
    manifest_data, manifest_sha = sealed_manifest(manifest, candidate, args.repository, args.raw)
    controls = []
    report = {"candidate": candidate, "mode": args.mode, "run_manifest_sha256": manifest_sha}
    if args.repeat_raw:
        repeat_manifest = args.repeat_raw.parent / "manifest.json"
        repeat, repeat_sha = sealed_manifest(repeat_manifest, candidate, args.repository, args.repeat_raw)
        require(args.mode == "force" and manifest_data["wall_pause_ms"] == 0 and repeat["wall_pause_ms"] > 0,
                "pause comparison requires a plain and a wall-paused Force run")
        require(manifest_data["binaries"] == repeat["binaries"], "primary and repeat binaries differ")
        base_trace = normalized_trace(args.raw)
        repeat_trace = normalized_trace(args.repeat_raw)
        require(base_trace == repeat_trace == (args.raw / "trace.virtual.jsonl").read_bytes()
                == (args.repeat_raw / "trace.virtual.jsonl").read_bytes(), "normalized full logical traces differ")
        replay_digests = {}
        for name in ("frames.bin", "l1/raw-rpc-bundle.json"):
            base_hash = digest(args.raw / name)
            require(base_hash == digest(args.repeat_raw / name), f"raw deterministic artifact differs: {name}")
            replay_digests[name] = base_hash
        report["repeat"] = {"status": "PASS", "wall_pause_ms": repeat["wall_pause_ms"],
                            "normalized_sha256": hashlib.sha256(base_trace).hexdigest(),
                            "run_manifest_sha256": repeat_sha, "equal_raw_sha256": replay_digests}
    if args.mode in ("force", "abi"):
        evm, controls, report["receipts"] = receipt_projection(args.repository, args.receipts)
        expected = {"signature", "conflicting_request", "stale_sequence", "exact_retry"}
        require(len(controls) == 1, "Force/ABI scenario has additional Force writes")
        require(all(report["receipts"]["negative_and_retry_counts"].get(k, 0) >= 1 for k in expected),
                "missing a real ABI rejection or duplicate receipt")
    if args.mode != "abi":
        controls_path = args.output / "controls.json"
        controls_path.write_text(json.dumps(controls, indent=2) + "\n")
        public = load(subprocess.check_output([
            str(args.public_checker.resolve()), str(args.raw), str(controls_path),
            "10000000", "3000000", "8000000", "3000000",
        ]))
        trace = list(trace_rows(args.raw))
        require([r["tick_us"] for r in trace] == sorted(r["tick_us"] for r in trace), "virtual time regressed")
        lifecycle = check_frames(args.raw, public, args.repository / "poc/proto/native.proto", trace)
        if args.mode == "force":
            require(evm["config"]["announcementSeconds"] == 10 and evm["config"]["genesisTimestampSeconds"] == 0
                    and evm["config"]["slotSeconds"] == 2, "real L1 profile differs")
            profile = load((args.raw / "profile.json").read_bytes())
            require(profile["force"]["worker_l1_projection"] == "reg-a-height-timestamp-v1"
                    and profile["force"]["worker_l1_visibility_lag_us"] == 1,
                    "execution metadata projection missing profile provenance")
            require(evm["config"]["configRef"].lower() == "0x" + public["config_ref"], "L1 EIP-712 config binding")
            require(len(evm["blocks"]) >= 11, "L1 evidence omits execution origin horizon")
            for real, projected in zip(evm["blocks"], public["l1"]):
                require(projected["height"] == int(real["number"], 16)
                        and projected["timestamp_s"] == int(real["timestamp"], 16), "real L1/metadata header projection")
            hub_l1 = args.hub_l1 or (args.raw / "l1/hub-l1.jsonl")
            stages = driver_times(hub_l1, args.driver_rpc, evm, controls[0])
            report["stages"] = stages
            report["force"] = check_primary(trace, public, controls, lifecycle, stages, evm)
        else:
            report["regular"] = check_regular(trace, public, lifecycle)
        public_path = args.output / "public-replay.json"
        public_path.write_text(json.dumps(public, indent=2) + "\n")
        report["lifecycle"] = lifecycle
        report["public_sha256"] = digest(public_path)
        report["raw_trace_sha256"] = digest(args.raw / "trace.jsonl")
    report["status"] = "PASS"
    (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
