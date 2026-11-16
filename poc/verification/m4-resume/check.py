#!/usr/bin/env python3
"""Independent raw evidence checker for the frozen M4 primary and pause repeat."""

import argparse
import hashlib
import importlib.util
import json
import subprocess
import sys
from dataclasses import asdict
from pathlib import Path

CONTRACT = "poc/docs/milestones/M4-d-resume-task-contract-zh.md"
CONTRACT_SHA = "ce79e5457980b039d7d75c812816482fdaf02446add713922836a1ebd18f791f"
EXPECTATIONS = "poc/verification/m4-resume/source-expectations.md"
EXPECTATIONS_SHA = "2d8e8218a65dbc5ca31e3d9efbda01de4449729be218a474546020200f028965"
DELIVERIES = (
    "b404818e18be62663e84a7fa580b151c686ad951",
    "aab8a5180801ed7f97f7e437d147c97f972b4dd8",
    "1f6d1b3b4076ff16908dc59f272a06686b1d1ea5",
)
BFT = "691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6"
US = 1_000_000


def need(ok, detail):
    if not ok:
        raise ValueError(detail)


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read(path):
    def strict_object(pairs):
        data = dict(pairs)
        need(len(data) == len(pairs), f"duplicate JSON key in {path}")
        return data
    return json.loads(path.read_bytes(), object_pairs_hook=strict_object)


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write("\n")


def one(rows, detail):
    rows = list(rows)
    need(len(rows) == 1, f"{detail}: {len(rows)} occurrences")
    return rows[0]


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def setup(repository):
    directory = repository / "poc/verification"
    sys.path[:0] = [
        str(directory / subdir) for subdir in (
            "m2-healthy-force", "anvil-l1", "m3-auto-force", "batch-services", "derivation",
        )
    ]
    return (
        load_module("m4_legacy_frames", directory / "m2-healthy-force/check.py"),
        load_module("m4_public_wire", directory / "m1-gfr-replay/wire.py"),
    )


def sealed(run, repository, candidate):
    path = run / "manifest.json"
    digest = sha(path)
    need(path.with_suffix(".sha256").read_text().split() == [digest, path.name],
         "missing or invalid manifest checksum")
    manifest = read(path)
    need(manifest["candidate"] == candidate and manifest["seed"] == 42
         and manifest["status"] == "COMPLETED", "sealed run revision, seed or status")
    before, after = manifest["before"], manifest["after"]
    need(before == after and before["head"] == candidate and not before["status"], "candidate was dirty")
    need(before["bft_gitlink"] == BFT and manifest["commonware"] == "2026.10.0", "pinned versions differ")
    frozen = {CONTRACT: CONTRACT_SHA, EXPECTATIONS: EXPECTATIONS_SHA}
    inputs = manifest["inputs"]
    for name, digest_expected in frozen.items():
        need(inputs.get(name) == digest_expected, f"missing frozen input {name}")
    for name, digest_expected in inputs.items():
        item = (repository / name).resolve(strict=True)
        need(item.is_relative_to(repository) and sha(item) == digest_expected, f"source mismatch for {name}")
    for name, expected in manifest["files"].items():
        item = run / name
        need(not item.is_symlink() and item.resolve().is_relative_to(run)
             and item.is_file() and item.stat().st_size == expected["bytes"]
             and sha(item) == expected["sha256"], f"unsealed or corrupt artifact: {name}")
    raw_files = {item.relative_to(run).as_posix() for item in (run / "raw").rglob("*") if item.is_file()}
    need(raw_files == {p for p in manifest["files"] if p.startswith("raw/")}, "raw inventory differs")
    return manifest, digest


def raw_file(run, supplied, pattern):
    if supplied:
        candidate = supplied.resolve(strict=True)
        need(candidate.is_relative_to(run), "explicit evidence outside sealed run")
        return candidate
    return one((run / "raw").rglob(pattern), f"expected one sealed {pattern}")


def check_batch_sources(raw, batches, old_g, genesis, config, credits, bundle, stages):
    from check_cut import hx
    from derive_oracle import l1_pos, to_transfer
    from independent_reference import be, digest, execute, state_bytes
    import independent_codec as crypto
    import receipt_checker as receipt

    for batch in batches:
        [record] = batch.records
        height = record["height"]
        old = old_g[height]
        tx = bundle["blocks"][batch.position[0]]["transactions"][batch.position[1]]
        _, (_, submission) = receipt.decode_call(tx)
        da_signature = submission[3]
        need(old["metadata"]["origin"] == record["origin"]
             and old["kept_bytes"] == [tx.hex() for tx in record["transactions"]],
             "accepted frame differs from independently verified G source")
        expected_pos = {"kind": "Espresso", "key_us": old["theta_us"],
                        "index": {"Espresso": old["q"]}}
        # GFR ReplayPosition v1, canonical Espresso kind tag 4.
        gpos = be(1, 2) + be(old["theta_us"], 8) + b"\x04" + be(old["q"], 8)
        need(batch.gpos == gpos, "attested batch gpos is not the independent G append position")
        job = read(raw / f"derivation/batch-h{height}.json")
        prepared = crypto.as_bytes(job["raw"])
        need(crypto.hx(crypto.keccak256(prepared)) == batch.tx_hash
             and job["hash"].lower() == batch.tx_hash
             and job["nonce"] == receipt.q(tx["nonce"]), "prepared Batcher transaction vs actual L1 write")
        need(hx(job["commitment"]) == batch.commitment[2:], "prepared job commitment vs actual L1")
        snapshot = job["snapshot"]
        need(snapshot["position"] == expected_pos and bytes(snapshot["gpos"]) == gpos,
             "snapshot replay position differs from attested G")
        [block] = snapshot["blocks"]
        parent = snapshot["parent"]
        parent_meta = [parent["metadata"][key] for key in ("height", "timestamp_us", "origin")]
        meta = [block["metadata"][key] for key in ("height", "timestamp_us", "origin")]
        expected_meta = [old["metadata"][key] for key in ("height", "timestamp_us", "origin")]
        need(meta == expected_meta, "G-backed batch metadata differs")
        parent_hash = hx(snapshot["native_parent"])
        expected_parent_hash = old_g[height - 1]["native"] if height > 1 else hx(genesis["native_hash"])
        expected_parent_root = old_g[height - 1]["state_root"] if height > 1 else config["genesis_state_root"]
        need(parent_hash == expected_parent_hash
             and digest("PoC/ExecutionState/v1", state_bytes(parent_meta, parent["accounts"])) == expected_parent_root,
             "snapshot parent is not authentic prior G state")
        out = execute(
            parent_meta, parent["accounts"], parent_hash, meta,
            [(row["to"], row["amount"]) for row in credits[height]],
            [to_transfer(bytes(raw_tx)) for raw_tx in block["transactions"]],
        )
        need(out["native_hash"] == old["native"] and out["post_state_root"] == old["state_root"]
             and out["kept"] == old["kept_bytes"] and out["dep"] == [hx(raw_dep) for raw_dep in block["dep"]]
             and out["pre_state_root"] == hx(block["pre_state_root"]),
             "snapshot independently reexecuted block is not the accepted G block")
        path = raw / f"derivation/da/{batch.commitment}.receipt"
        need(path.read_bytes() == da_signature, "persisted DA receipt differs from signed Inbox calldata")
        block_number, index, log = batch.position
        expected_receipt = one((row for row in bundle["receipts"]
                                if row["transactionHash"].lower() == batch.tx_hash),
                               "accepted actual receipt")
        for reader in ("A", "B"):
            accepted = read(raw / f"derivation/accepted-{reader}-{block_number}-{index}-{log}.json")
            evidence = accepted["evidence"]
            identity = accepted["identity"]
            need(accepted["position"] == l1_pos(batch.position)
                 and accepted["rpc_log_index"] == batch.log_index
                 and accepted["submission"]["commitment"].lower() == batch.commitment
                 and accepted["submission"]["gpos"].lower() == ("0x" + gpos.hex())
                 and hx(identity["block_hash"]) == batch.block_hash[2:]
                 and hx(identity["transaction_hash"]) == batch.tx_hash[2:],
                 "accepted input not bound to authenticated G and real Inbox position")
            need(evidence["reader"] == reader
                 and evidence["header"] == bundle["blocks"][block_number]
                 and evidence["transaction"] == tx and evidence["receipt"] == expected_receipt
                 and stages["finalized_us"][block_number] <= evidence["reference_us"]
                 and accepted["reader_visible_through"] >= block_number,
                 "accepted input evidence escaped finalized configured-reader receipt")
        need(stages["submit_us"][batch.tx_hash] < stages["mine_us"][block_number],
             "batch submission missing before inclusion")


def check_d_trace(raw, trace, config, genesis, credits, headers, batches, resumes, stages, prefix, old_g):
    from check_cut import hx, verify_item
    from derive_oracle import Derive, l1_pos

    per_reader = {"A": [], "B": []}
    file = raw / "derivation/trace.jsonl"
    for line in file.read_bytes().splitlines():
        row = json.loads(line)
        per_reader[row["reader"]].append(row)
    stats = {}
    for reader, rows in per_reader.items():
        canonical = Derive(config, genesis, credits, headers, 4).advance(
            stages["reader_tips"][reader], batches, resumes,
        )
        actual_items = [item for row in rows for item in row.get("new_items", [])]
        need(len(actual_items) == len(canonical.blocks), "live D head at complete reader tip differs")
        for actual, expected in zip(actual_items, canonical.blocks, strict=True):
            verify_item(actual, expected)
        for row in rows:
            height = row["l1_height"]
            need(height == 0 or row["tick_us"] >= stages["deliver_us"][reader][height],
                 "D consumed a block before actual finalized reader delivery")
            for item in row.get("new_items", ()):
                need(item["state"]["metadata"]["origin"] <= height, "D read origin beyond local L1 prefix")
        early = [row for row in rows if any(b["state"]["metadata"]["height"] == 2
                                             for b in row.get("new_items", ()))]
        h2 = one(early, "actual D h2 fallback")
        need(h2["l1_height"] == 7 and h2["tick_us"] >= stages["finalized_us"][7],
             "D fallback happened before the exact complete native window")
        late = one((row for row in rows if row.get("step", {}).get("kind") == "accepted_record"
                    and row["step"]["position"] == l1_pos(batches[-1].position)),
                   "real late batch consumed at its exact position")
        need(late["step"]["height"] == 2 and late["step"]["outcome"] in {"late", "old"}
             and not late["new_items"] and late["tick_us"] > h2["tick_us"],
             "accepted but late batch was adopted after fallback")
        cut = one((row for row in rows if row.get("step", {}).get("kind") == "cursor_set"),
                  "the actual D cursor set")
        need(cut["step"]["position"] == l1_pos(resumes[0].set_position)
             and cut["step"]["resume_position"] == l1_pos(resumes[0].position)
             and cut["step"]["fill_count"] == 17 and len(cut["new_items"]) == 17,
             "D CursorSet fill/reset bound to wrong historical position")
        stats[reader] = {"head": len(actual_items), "h2_fallback_us": h2["tick_us"],
                         "late_batch_disposition": late["step"]["outcome"],
                         "cursor_set_us": cut["tick_us"], "steps": len(rows)}
    conflict = one((row for row in trace if row["kind"] == "DerivationConflict"), "live G/D mismatch")
    proposal = one((row for row in trace if row["kind"] == "ResumePreset"), "preset governance observation")
    need(conflict["detail"]["height"] == 2
         and hx(conflict["detail"]["d_hash"]) == prefix[1]["native_hash"]
         and hx(conflict["detail"]["g_hash"]) == old_g[2]["native"]
         and hx(conflict["detail"]["common_h1"]) == prefix[0]["native_hash"],
         "governance did not observe independently derived shared-height divergence")
    need(stats["A"]["h2_fallback_us"] <= conflict["tick_us"] < proposal["tick_us"]
         and max(proposal["detail"][name] for name in ("g_head", "d_head", "inbox_height")) < resumes[0].target,
         "preset control observation or monotonicity")
    return stats


def timeline(trace, public, resume, cut, canonical_cut, stages, lifecycle):
    from check_cut import hx
    from derive_oracle import l1_pos

    projected = one(public["resumes"], "independent Resume transition")
    batches = [row for row in trace if row["kind"] == "GfrReplayRecord"
               and row["detail"]["position"]["kind"] == "Resume"]
    row = one(batches, "exactly one applied Resume replay record")
    applied = row["detail"]
    position = {"key_us": resume.act_us, "kind": "Resume",
                "index": {"L1": l1_pos(resume.position)}}
    need(applied["position"] == position, "replay Resume position differs")
    events = [event["body"] for event in applied["events"]]
    need([next(iter(event)) for event in events] == ["BranchClosed", "Resumed", "SessionAssigned"]
         and all(event["position"] == position for event in applied["events"]),
         "expected ordered BranchClosed/Resumed/SessionAssigned")
    closed, resumed, assigned = events
    need(closed["BranchClosed"] == {"branch": projected["closed_beta"], "reason": "Resume"},
         "wrong old branch closed")
    resumed = resumed["Resumed"]
    descriptor = assigned["SessionAssigned"]["descriptor"]
    cid = hx(descriptor["cid"])
    need(resumed["control_seq"] == resume.seq and resumed["branch"] == projected["beta"]
         and resumed["anchor"] == cut["anchor"]["anchor"]
         and resumed["derived_prefix"] == [item["virtual_block"]
             for item in canonical_cut["derived_prefix"][:resume.target]],
         "Resumed event differs from exact authenticated cut")
    reset = one((entry for entry in trace if entry["kind"] == "GReset"), "trace G Reset")["detail"]
    need(reset["resume_position"] == l1_pos(resume.position)
         and hx(reset["cut_digest"]) == cut["cut_digest"]
         and reset["target_height"] == resume.target
         and reset["old_height"] == projected["old_head"]
         and reset["common_height"] == projected["common_height"]
         and [hx(item) for item in reset["old_hashes"]] == [item["native"] for item in projected["old_suffix"]],
         "G Reset did not retain the independently computed common prefix")
    derived = [entry["detail"] for entry in trace if entry["kind"] == "GDerived"]
    adopted = [entry for entry in public["expected"] if entry.get("source") == "DER"]
    need(len(derived) == len(adopted), "G DER adoption trace count")
    for actual, expected in zip(derived, adopted, strict=True):
        need(actual["height"] == expected["height"] and actual["position"] == position
             and actual["resume_position"] == l1_pos(resume.position)
             and hx(actual["native_hash"]) == expected["native"]
             and hx(actual["state_root"]) == expected["state_root"], "G DER adoption/native state")
    activation = one((entry for entry in trace if entry["kind"] == "ResumeActivated"),
                     "trace Resume activation")["detail"]
    need(activation["position"] == position and activation["set_position"] == l1_pos(resume.set_position)
         and activation["resume_position"] == l1_pos(resume.position)
         and hx(activation["cut_digest"]) == cut["cut_digest"]
         and hx(activation["anchor_native_hash"]) == hx(cut["anchor"]["native_hash"])
         and activation["plan_basis_height"] == activation["l1_floor"] == projected["floor"]
         and activation["descriptor"] == descriptor and activation["record_id"] == applied["id"],
         "trace activation/cut/Plan basis")
    need(descriptor["anchor"] == cut["anchor"]["anchor"], "worker assignment anchor")
    plan = descriptor["plan"]
    need(plan["cause"] == "Resume" and plan["session"] == {"beta": projected["beta"], "r": 0}
         and plan["source_region"] is None and plan["region"] == projected["region"]
         and plan["window"] == {"h_min": projected["h_min"], "h_max": projected["h_max"]},
         "wrong Plan on assigned descriptor")
    accepted = [entry for entry in trace if entry["kind"] in ("GAccepted", "GPadding")]
    need(len(accepted) == len(public["regional_entries"]), "G regional append trace length")
    for logged, expected in zip(accepted, public["regional_entries"], strict=True):
        detail = logged["detail"]
        need((detail["height"], hx(detail["cid"]), hx(detail["native_hash"]), hx(detail["state_root"]))
             == (expected["height"], expected["cid"], expected["native"], expected["state_root"]),
             "independently recomputed G append does not match trace")
        need(detail["position"] == {
            "key_us": expected["theta_us"], "kind": "Espresso",
            "index": {"Espresso": expected["q"]},
        }, "G append position differs")
        need(hx(detail["physical_id"]) == (expected["physical"] or expected["target"]),
             "G append physical identity")
        if expected["physical"]:
            need(logged["kind"] == "GAccepted" and hx(detail["parent"]) == expected["parent"],
                 "G append physical parent")
    final, accepted_final = public["regional_entries"][-1], accepted[-1]
    need(final["cid"] == cid and final["height"] == resume.target + 1
         and final["parent"] == hx(cut["anchor"]["anchor"]["block"]["block_id"])
         and final["kept"] >= 1, "the first restored native block is not retained in G from the derived anchor")
    need(final["metadata"]["origin"] == 10 and final["theta_us"] < 61 * US,
         "restore origin continuity or expiry")
    need(projected["common_height"] == 1 and projected["old_head"] >= 2
         and len(projected["old_suffix"]) == projected["old_head"] - 1
         and all(item.get("source") == "DER" for item in public["expected"][1:20]),
         "wrong shared prefix or DER suffix recovery")
    ready = [entry for entry in trace if entry["kind"] == "Ready" and hx(entry["detail"]["cid"]) == cid]
    enter = [entry for entry in trace if entry["kind"] == "Enter" and hx(entry["detail"]["cid"]) == cid]
    pids = set(range(10, 15))
    need({entry["pid"] for entry in ready} == {entry["pid"] for entry in enter} == pids
         and len(ready) == len(enter) == 5, "five staggered R2 workers must complete startup")
    readiness = {entry["pid"]: entry for entry in ready}
    need(all(row["seq"] < readiness[entry["pid"]]["seq"] <= entry["seq"] < accepted_final["seq"]
             and entry["tick_us"] < projected["latest_ready_us"] for entry in enter),
         "premature or late worker readiness/entry")
    need(len({entry["tick_us"] for entry in enter}) == len(enter), "resumed worker start was not staggered")
    need(all(entry["detail"]["state_root"] == projected["anchor_root"] for entry in ready),
         "worker Ready state differs from native target")
    old_closed = {entry["pid"] for entry in trace if entry["kind"] == "Close"
                  and entry["seq"] < accepted_final["seq"]}
    need(set(range(10)) <= old_closed, "missing terminal Close on the old branch")
    resumed_closes = [entry for entry in trace if entry["kind"] == "Close"
                      and entry["detail"]["cause"] == "Public(Resume)"]
    need({entry["pid"] for entry in resumed_closes} == set(range(5, 10))
         and all(row["seq"] < entry["seq"] < min(ready_row["seq"] for ready_row in ready)
                 for entry in resumed_closes), "R1 old duty was not durably closed before R2 startup")
    need(all(seq > accepted_final["seq"] for seq in lifecycle["exits"].values()),
         "worker exited before first restored block entered G")
    need(stages["finalized_us"][resume.position[0]] <= row["tick_us"]
         and resume.act_us <= row["tick_us"] < min(entry["tick_us"] for entry in ready),
         "incorrect Resume apply/readiness ordering")
    live_d_at_activate = max(entry["detail"]["head"]["state"]["metadata"]["height"]
        for entry in trace if entry["kind"] == "DerivationAvailable"
        and entry["detail"]["reader"] == "A" and entry["tick_us"] <= row["tick_us"]
        and entry["detail"]["head"] is not None)
    need(live_d_at_activate > resume.target, "main profile did not observe D ahead at activation")
    return {
        "apply_us": row["tick_us"], "act_us": resume.act_us,
        "live_d_at_activate": live_d_at_activate,
        "resume_position": resume.position, "cut_digest": cut["cut_digest"],
        "common_height": projected["common_height"], "old_head": projected["old_head"],
        "region": projected["region"], "restored_height": final["height"], "restored_native": final["native"],
        "ready_us": [entry["tick_us"] for entry in ready], "entered_us": [entry["tick_us"] for entry in enter],
        "first_global_accept_us": accepted_final["tick_us"],
    }


def check_primary(args, repository, run, manifest, manifest_sha, legacy, wire):
    from verified_inputs import project
    from check_l1 import audit
    from derive_oracle import Derive, genesis_from_bytes
    from check_cut import check_positioned_cut

    raw = run / "raw"
    bundle = read(raw_file(run, args.bundle, "raw-rpc-bundle.json"))
    if args.da_index:
        index_path = raw_file(run, args.da_index, "da-index.json")
        da_root = index_path.parent
        da_index = read(index_path)["payloads"]
    else:
        da_root = raw / "derivation"
        da_index = {path.stem.lower(): path.relative_to(da_root).as_posix()
                    for path in (da_root / "da").glob("0x*.bin")}
    artifact = one((run / name for name in manifest["files"]
                    if name.endswith("/CommitteeControlAuthority.sol/CommitteeControlAuthority.json")),
                   "sealed compiled committee artifact")
    committee, abi, headers, batches, resumes = project(bundle, artifact, da_index, da_root)
    need((committee.threshold, len(committee.members)) == (2, 3), "primary M4 committee is not 2/3")
    need(not [row for row in abi["events"] if row["kind"] == "ForceRecorded"], "unexpected Force")
    resume = one(resumes, "exactly one primary preset Resume")
    need(resume.target == 20 and resume.signed_origin == 9 and resume.position[0] == 9
         and resume.act_us == 28 * US and resume.policy == {"enabled_mask": 7, "order": [2, 0, 1]},
         "frozen positive Resume profile differs")
    need(len(batches) == 2, "expected one timely and one late batch")
    timely, late = batches
    need([(row.position[0], row.previous_height, len(row.records), row.records[0]["origin"])
          for row in batches] == [(6, 0, 1, 3), (8, 1, 1, 3)]
         and all(row.records[0]["transactions"] for row in batches), "frozen primary batch receipts differ")
    stages = audit(bundle, legacy.load_rows(raw_file(run, args.driver_trace, "driver-trace.json")))
    need("A" in stages["reader_tips"] and stages["reader_tips"]["A"] >= 9, "Resume L1 prefix unavailable")
    need(stages["mine_us"][6] == 12 * US and stages["mine_us"][8] == 16 * US
         and stages["mine_us"][9] == 18 * US, "frozen L1 mine schedule")
    submit = stages["submit_us"][resume.tx_hash]
    need(stages["deliver_us"]["A"][7] <= submit < stages["mine_us"][9] and submit >= 17 * US,
         "Resume submitted without completed native fallback window")
    config_bytes, dep_bytes = (raw / "config.bin").read_bytes(), (raw / "dep.bin").read_bytes()
    config = wire.decode_config(config_bytes, "config.bin")
    credits = wire.decode_dep(dep_bytes, "dep.bin")
    genesis = genesis_from_bytes((raw / "genesis.bin").read_bytes(), wire)
    need(config["ref"] == bundle["config"]["configRef"].lower()[2:] and config["chain_id"] == 31337,
         "L1 and BFT profile configuration differs")
    need(wire.domain_hash("PoC/DepFixture/v1", dep_bytes) == config["dep_fixture_hash"],
         "deposits do not match authenticated configuration")
    times = {height: int(row["timestamp"], 16) * US for height, row in headers.items()}
    reference = Derive(config, genesis, credits, times, 4).advance(
        stages["reader_tips"]["A"], batches, resumes,
    )
    prefix = reference.cuts[resume.position]
    need(prefix[0]["cause"] == ("batch", timely.position, timely.commitment, 0) and prefix[0]["kept"]
         and prefix[1]["cause"] == ("fallback", 7, 7) and not prefix[1]["kept"],
         "h1 timely acceptance and h2 native-window fallback do not match the verified receipt order")
    need(reference.discarded == [{
        "position": late.position, "height": 2, "late": True, "already_derived": True,
    }], "late h2 altered already-derived history")
    need(prefix[19]["metadata"] == [20, 30 * US, 9] and len(prefix) == 20,
         "independent target cut differs from frozen schedule")
    if args.cut:
        cut_paths = [path.resolve(strict=True) for path in args.cut]
    else:
        cut_paths = [path for path in raw.rglob("*.json") if "cut" in path.name.lower()
                     and isinstance(read(path), dict) and "resume_position" in read(path)]
    need(cut_paths, "no canonical ResumeCut evidence. Supply --cut for compact cut files")
    cut_data = set()
    for path in cut_paths:
        need(path.is_relative_to(raw), "cut evidence outside sealed raw directory")
        cut_data.add(path.read_bytes())
    [cut_bytes] = cut_data
    canonical_cut = json.loads(cut_bytes)
    cut = check_positioned_cut(
        cut_bytes, canonical_cut, resume, prefix, genesis, config, bundle,
        read(raw_file(run, args.driver_config, "driver-config.json")), 4,
    )

    args.output.mkdir()
    controls, projected_cuts, actual_headers = [
        args.output / name for name in ("controls.json", "verified-cuts.json", "verified-l1-headers.json")
    ]
    save(controls, [])
    save(projected_cuts, [cut])
    save(actual_headers, [
        {"height": height, "timestamp_s": times[height] // US,
         "hash": list(bytes.fromhex(block["hash"][2:]))}
        for height, block in headers.items()
    ])
    public_checker = args.public_checker.resolve(strict=True)
    public = json.loads(subprocess.check_output([
        str(public_checker), str(raw), str(controls), str(10 * US), str(3 * US),
        str(30 * US), str(3 * US), str(actual_headers), str(projected_cuts),
    ]))
    old_g = {row["height"]: row for row in public["regional_entries"] if row["beta"] == 0}
    check_batch_sources(raw, batches, old_g, genesis, config, credits, bundle, stages)
    need(old_g[1]["native"] == prefix[0]["native_hash"]
         and old_g[2]["native"] != prefix[1]["native_hash"], "wrong shared-height G/D hash comparison")
    trace = list(legacy.trace_rows(raw))
    need([row["tick_us"] for row in trace] == sorted(row["tick_us"] for row in trace),
         "virtual time regressed")
    need(not any(row["kind"] == "RunFailed" for row in trace), "completed run contains failure")
    lifecycle = legacy.check_frames(raw, public, repository / "poc/proto/native.proto", trace)
    d_evidence = check_d_trace(raw, trace, config, genesis, credits, times, batches, resumes,
                               stages, prefix, old_g)
    protocol = timeline(trace, public, resume, cut, canonical_cut, stages, lifecycle)
    d_on_send = max(height for height, at in stages["deliver_us"]["A"].items() if at <= submit)
    old_g_on_send = max(row["detail"]["height"] for row in trace
                        if row["kind"] == "GAccepted" and row["tick_us"] <= submit)
    ready_latency = max(protocol["entered_us"]) - protocol["act_us"]
    target_ts = cut["anchor"]["anchor"]["timestamp_us"]
    include_ts = times[resume.position[0]]
    need(resume.target > max(reference.at_end[d_on_send], old_g_on_send, late.records[-1]["height"])
         and include_ts + 12 * US <= target_ts <= include_ts + 14 * US
         and 0 <= ready_latency <= US // 2
         and 10 * US + ready_latency + 1000 <= 12 * US + config["l2_slot_us"],
         "positive Resume target guard or runtime preparation budget")
    # A submitted payload only counts as a G source if its accepted block had appeared before send.
    for batch in batches:
        last = batch.records[-1]["height"]
        g_tick = one((row["tick_us"] for row in trace if row["kind"] == "GAccepted"
                      and row["detail"]["height"] == last and row["tick_us"] < submit), "source G append")
        need(g_tick < stages["submit_us"][batch.tx_hash], "batch submitted before its G source")
    report = {
        "status": "PASS", "candidate": args.candidate, "seed": manifest["seed"],
        "manifest_sha256": manifest_sha, "public_checker_sha256": sha(public_checker),
        "lifecycle": lifecycle, "protocol": protocol, "l1": stages, "receipts": abi,
        "d_evidence": d_evidence,
        "batches": [
            {**asdict(row), "gpos": row.gpos.hex(),
             "records": [{**b, "transactions": [tx.hex() for tx in b["transactions"]]} for b in row.records]}
            for row in batches
        ],
        "resumes": [asdict(row) for row in resumes],
        "shared_hashes": [
            {"height": index + 1, "g": old_g[index + 1]["native"], "d": prefix[index]["native_hash"]}
            for index in range(2)
        ],
    }
    save(args.output / "public-replay.json", public)
    save(args.output / "independent-D.json", {"blocks": reference.blocks, "discarded": reference.discarded})
    return report, cut_paths, da_root, da_index


def check_repeat(run, repeat, manifest, repeated, cut_paths, da_root, da_index, legacy):
    need(manifest["wall_pause_ms"] == 0 and repeated["wall_pause_ms"] == 650, "wrong pause profiles")
    need(manifest["binaries"] == repeated["binaries"], "changed binaries in the pause repeat")
    raw, replay = run / "raw", repeat / "raw"
    normalized = legacy.normalized_trace(raw)
    need(normalized == legacy.normalized_trace(replay)
         == (raw / "trace.virtual.jsonl").read_bytes()
         == (replay / "trace.virtual.jsonl").read_bytes(),
         "pause repeat changed normalized virtual trace")
    paths = {
        raw / "frames.bin",
        raw_file(run, None, "raw-rpc-bundle.json"),
        raw_file(run, None, "driver-trace.json"),
        raw / "derivation/trace.jsonl",
        raw / "derivation/cursor-reads.jsonl",
        *((raw / "derivation").glob("batch-h*.json")),
        *((raw / "derivation").glob("accepted-*.json")),
        *cut_paths,
        *((da_root / file).resolve(strict=True) for file in da_index.values()),
        *((da_root / "da").glob("0x*.receipt")),
    }
    hashes = {}
    for path in sorted(paths):
        key = path.relative_to(run).as_posix()
        need(sha(path) == sha(repeat / key), f"pause repeat changed {key}")
        hashes[key] = sha(path)
    return {
        "status": "PASS", "wall_pause_ms": 650,
        "normalized_trace_sha256": hashlib.sha256(normalized).hexdigest(), "equal": hashes,
    }


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--repository", type=Path, required=True)
    p.add_argument("--candidate", required=True)
    p.add_argument("--run", type=Path, required=True)
    p.add_argument("--repeat", type=Path)
    p.add_argument("--public-checker", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--bundle", type=Path)
    p.add_argument("--da-index", type=Path)
    p.add_argument("--driver-config", type=Path)
    p.add_argument("--driver-trace", type=Path)
    p.add_argument("--cut", type=Path, action="append")
    args = p.parse_args()
    repository, run = [path.resolve(strict=True) for path in (args.repository, args.run)]
    git = ["git", "-C", str(repository)]
    need(subprocess.check_output([*git, "rev-parse", "HEAD"], text=True).strip() == args.candidate
         and not subprocess.check_output([*git, "status", "--porcelain=v1", "--untracked-files=all"]),
         "verifier checkout not clean at exact candidate")
    for sha_arg in DELIVERIES:
        subprocess.run(
            [*git, "merge-base", "--is-ancestor", sha_arg, args.candidate], check=True,
        )
    legacy, wire = setup(repository)
    manifest, manifest_sha = sealed(run, repository, args.candidate)
    report, cut_paths, da_root, da_index = check_primary(
        args, repository, run, manifest, manifest_sha, legacy, wire,
    )
    if args.repeat:
        repeat = args.repeat.resolve(strict=True)
        repeated, digest = sealed(repeat, repository, args.candidate)
        report["repeat"] = check_repeat(
            run, repeat, manifest, repeated, cut_paths, da_index_path, da_index, legacy,
        )
        report["repeat"]["manifest_sha256"] = digest
    save(args.output / "report.json", report)


if __name__ == "__main__":
    main()
