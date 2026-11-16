#!/usr/bin/env python3
"""Run and independently inspect the M4 real L1/DA/derivation input vectors."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

from check_archive import accepted_check, check_archive, load, require

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
LEGACY = REPO / "poc/crates/l1-adapter/tests/run_anvil.py"
REFERENCE = REPO / "poc/verification/derivation/independent_reference.py"

spec = importlib.util.spec_from_file_location("derivation_reference", REFERENCE)
oracle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(oracle)

receipts = accepted_check.r
codec = accepted_check.c

CASES = ("same_block", "before", "boundary", "after", "substitute", "future", "malformed")
EXPECTED = {
    "same_block": (3, 1, 0, [(0, []), (1, [[1, 2, 0, 3]]), (1, [])]),
    "before": (3, 0, 0, [(0, [[1, 2, 0, 3]])]),
    "boundary": (4, 0, 0, [(0, [[1, 2, 0, 3]]), (1, [])]),
    "after": (5, 0, 0, [(0, []), (1, []), (1, []), (2, [])]),
    "substitute": (3, 0, 0, [(0, []), (1, [[1, 2, 0, 3]])]),
    "future": (5, 0, 0, [(0, []), (1, []), (1, []), (2, [[1, 2, 0, 3]])]),
    "malformed": (3, 0, 0, []),
}


def dump(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def position(at):
    return [at["block"], at["tx"], at["log"]]


def check_legacy(run):
    bundle = load(run / "raw-rpc-bundle.json")
    verified = receipts.Checker(bundle).run()
    scenarios = load(run / "result.json")["cases"]
    failed = scenarios["rollback-after-inner-write"]["hash"]
    succeeded = scenarios["rollback-fixture-same-request-success"]["hash"]
    by_hash = {receipt["transactionHash"]: receipt for receipt in bundle["receipts"]}
    before, after = by_hash[failed], by_hash[succeeded]
    require(failed in verified["failedReceipts"], "rollback receipt missing from checker")
    require(before["status"] == "0x0" and before["logs"] == [], "reverted transaction committed events")
    require(after["status"] == "0x1" and len(after["logs"]) == 2, "identical request failed to retry")
    transactions = {
        tx["hash"]: tx for block in bundle["blocks"] for tx in block["transactions"]
    }
    call_failure, call_success = transactions[failed], transactions[succeeded]
    failure_bytes = codec.as_bytes(call_failure["input"])
    success_bytes = codec.as_bytes(call_success["input"])
    selector = codec.keccak256(("resume(bytes32,uint64," + receipts.g.CURSOR_SIG + ",bool)").encode())[:4]
    require(failure_bytes[:4] == success_bytes[:4] == selector, "wrong rollback fixture method")
    # The bool shares its canonical 32-byte ABI word with the values u8(0)/u8(1).
    call_types = ["bytes32", "uint64", receipts.g.CURSOR, "uint8"]
    failure_args = receipts.abi_decode(call_types, failure_bytes[4:])
    success_args = receipts.abi_decode(call_types, success_bytes[4:])
    require(failure_args[:3] == success_args[:3] and (failure_args[3], success_args[3]) == (1, 0),
            "retry changed more than injected rollback flag")
    emitter = call_success["to"].lower()
    require(call_failure["to"].lower() == emitter, "rollback must use same fixture")
    require(after["logs"][1]["address"].lower() == emitter, "wrong success emitter")
    require(after["logs"][1]["topics"][0] == codec.hx(codec.keccak256(b"Recorded(uint64)")),
            "wrong success fixture marker")
    name, values = receipts.decode_event(after["logs"][0])
    zero = bytes(32)
    genesis_cursor = [0, 0, zero, 0, b""]
    target_cursor = [8, 0, zero, 2, codec.word(1)]
    require(name == "CursorSet" and values == [genesis_cursor, target_cursor],
            "cursor was not restored to old exact state")
    inbox = after["logs"][0]["address"].lower()
    expected_calls = {
        (emitter, codec.hx(codec.keccak256(b"seq()")[:4])): codec.hx(zero),
        (emitter, codec.hx(codec.keccak256(b"seen(bytes32)")[:4] + codec.word(1))): codec.hx(zero),
        (inbox, codec.hx(codec.keccak256(b"currentCursor()")[:4])): codec.hx(
            codec.abi_encode([receipts.g.CURSOR], [genesis_cursor])
        ),
    }
    observed = {}
    for row in load(run / "diagnostic-rpc.json"):
        if row["request"]["method"] != "eth_call":
            continue
        params = row["request"]["params"]
        # EVM reads specifically from the block containing the reverted call.
        if len(params) == 2 and params[1] == before["blockNumber"]:
            key = (params[0]["to"].lower(), params[0]["data"])
            if key in expected_calls:
                observed[key] = row["response"]["result"]
    require(observed == expected_calls, "reverted-block state reads do not prove atomic rollback")
    return {
        "status": "PASS", "scope": "actual test-only authorized inner write followed by outer revert",
        "failed_transaction": failed, "successful_same_request": succeeded,
        "old_cursor": receipts.jsonable(genesis_cursor), "post_cursor": receipts.jsonable(target_cursor),
        "old_seq": 0, "old_seen": False,
    }


def confirm_finite_receipts(run, bundle):
    headers = {int(block["number"], 16): block for block in bundle["blocks"]}
    receipts_by_hash = {row["transactionHash"]: row for row in bundle["receipts"]}
    readiness = {"A": set(), "B": set()}
    waits = 0
    for row in load(run / "driver-trace.json"):
        if "command" not in row:
            continue
        command = row["command"]
        reply = row["reply"]
        require(reply["id"] == command["id"], "driver command/reply identity mismatch")
        if command["op"] != "read" or command["query"]["kind"] != "block":
            continue
        target = command["query"]["height"]
        reader = command["reader"]
        if reply["status"] == "wait":
            require(reply["visible_heights"][reader] < target, "spurious WAIT above visible frontier")
            waits += 1
            continue
        require(reply["status"] == "ready", "unexpected 09 read outcome")
        block = reply["result"]["block"]
        raw = headers[target]
        require(reply["finalized_height"] >= target and reply["visible_heights"][reader] >= target,
                "read used undisclosed input")
        require(reply["result"]["control_coverage_through"] == target, "missing whole-block coverage")
        require(block["header_and_transactions"] == raw, "read used noncanonical block")
        require(block["receipts"] == [receipts_by_hash[tx["hash"]] for tx in raw["transactions"]],
                "read did not retain original receipts")
        readiness[reader].add(target)
    require(readiness["A"] == readiness["B"] == set(headers), "partial A/B input replay")
    require(waits >= 1, "expected reordered reader WAIT absent")
    return {"complete_heights": sorted(headers), "reader_waits": waits}


def check_rows(profile, derived, expected):
    require(len(derived) == len(expected), "unexpected D prefix length")
    genesis = profile["genesis"]
    dep = profile["deposits"]
    slot_us = profile["config"]["public"]["l2_slot_us"]
    metadata = [genesis["anchor"]["block"]["height"], genesis["anchor"]["timestamp_us"],
                genesis["anchor"]["block"]["origin"]]
    accounts = genesis["state"]["accounts"]
    parent = bytes(genesis["native_hash"]).hex()
    for output, (origin, transfers) in zip(derived, expected, strict=True):
        new = [metadata[0] + 1, metadata[1] + slot_us, origin]
        credits = [[row["to"], row["amount"]] for row in dep["rows"][new[0] - dep["first_height"]]]
        reference = oracle.execute(metadata, accounts, parent, new, credits, transfers)
        require(output["state"]["accounts"] == reference["accounts"], "D account state mismatch")
        require(output["state"]["metadata"] == dict(zip(("height", "timestamp_us", "origin"), new)),
                "D metadata mismatch")
        require(bytes(output["native_hash"]).hex() == reference["native_hash"], "D native hash mismatch")
        require(bytes(output["native_block"]["parent"]).hex() == parent, "D parent mismatch")
        require(bytes(output["native_block"]["post_state_root"]).hex() == reference["post_state_root"],
                "D post-state root mismatch")
        require([bytes(x).hex() for x in output["native_block"]["kept"]] == reference["kept"],
                "D ordinary tx set mismatch")
        metadata, accounts, parent = new, reference["accounts"], reference["native_hash"]
    return parent


def check_live(run, mode):
    bundle = load(run / "raw-rpc-bundle.json")
    coverage = confirm_finite_receipts(run, bundle)
    chain = receipts.Checker(bundle).run()
    report = load(run / f"{mode}.json")
    profile = load(run / "derivation-profile.json")
    require(profile["config_ref"] == bundle["config"]["configRef"], "D config identity mismatch")
    block, tx, log, expected_rows = EXPECTED[mode]
    admitted = [row for row in chain["events"] if row["kind"] == "CursorAdvanced"]
    require(len(admitted) == 1 and admitted[0]["position"] == [block, tx, log], "expected acceptance missing")
    accepted = [item for item in bundle["receipts"] if (
        int(item["blockNumber"], 16), int(item["transactionIndex"], 16)
    ) == (block, tx)]
    require(len(accepted) == 1 and accepted[0]["status"] == "0x1", "accepted vector not successful")

    if mode != "malformed":
        da = {"payloads": {path.stem: path.relative_to(run).as_posix()
                           for path in (run / "da").glob("*.bin")}}
        opened = accepted_check.check(bundle, da, run)
        require(opened["status"] == "PASS" and opened["accepted"][0]["position"] == [block, tx, log],
                "independent payload authentication failed")
    else:
        require(report["payload"] == "REJECT_FRAME", "authenticated malformed object was not rejected")
        require(report["head"]["anchor"]["block"]["height"] == 0, "malformed object advanced D")

    if mode == "same_block":
        controls = [row for row in chain["events"] if row["kind"] == "ResumeRecorded"]
        expected_positions = [[3, 0, 1], [3, 2, 1]]
        require([row["position"] for row in controls] == expected_positions, "control positions differ")
        for key in ("missing", "after_corrupt"):
            require(position(report[key]["position"]) == [3, 1, 0], f"{key} skipped earlier acceptance")
        first, second = report["cuts"]
        rows = second["derived_prefix"]
        for cut, control, target in zip([first, second], controls, (1, 3), strict=True):
            require(position(cut["resume_position"]) == control["position"], "cut Resume position mismatch")
            require(position(cut["cursor_set_position"]) == control["setPosition"], "cut Set position mismatch")
            require(cut["target_height"] == target and len(cut["derived_prefix"]) == target, "cut range mismatch")
            require(cut["target_anchor"]["state"] == cut["derived_prefix"][target - 1]["state"], "cut state mismatch")
            require(cut["target_anchor"]["native_hash"] == cut["derived_prefix"][target - 1]["native_hash"],
                    "cut target native hash mismatch")
        require(first["derived_prefix"] == rows[:1], "later control rewrote first historical cut")
        late_check = load(run / "same_block_tail.json")
        require(late_check["derived_head"]["anchor"]["block"]["height"] > 3, "unchallenged cut stability")
        require(late_check["cut_immutability"] and late_check["checkpoint_restore"], "historical cut lost")
    else:
        rows = report.get("derived", [])
    last = check_rows(profile, rows, expected_rows)
    return {"status": "PASS", "case": mode, "acceptance": [block, tx, log], "reader_evidence": coverage,
            "derived_heights_compared": len(rows), "native_head_compared": "0x" + last}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--anvil", type=Path, required=True)
    p.add_argument("--driver", type=Path, required=True)
    p.add_argument("--vector-port", type=Path, required=True)
    p.add_argument("--contracts", type=Path, required=True)
    p.add_argument("--archive", type=Path, required=True)
    p.add_argument("--out", type=Path, required=True)
    p.add_argument("--cases", nargs="+", choices=CASES, default=CASES)
    p.add_argument("--candidate", help="exact clean checkout SHA when used in a formal run")
    args = p.parse_args()
    args.out.mkdir()
    commands = []

    def verify_candidate():
        if args.candidate:
            head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()
            dirty = subprocess.check_output(["git", "status", "--porcelain"], cwd=REPO, text=True)
            require(head == args.candidate and not dirty, "input vectors require exact clean checkout")

    def run(label, argv):
        argv = [str(value) for value in argv]
        commands.append({"label": label, "argv": argv})
        with (args.out / f"{label}.stdout").open("x") as out, (args.out / f"{label}.stderr").open("x") as err:
            result = subprocess.run(argv, cwd=REPO, stdout=out, stderr=err, timeout=180)
        commands[-1]["exit"] = result.returncode
        require(result.returncode == 0, f"subprocess {label} failed, inspect stage output")

    try:
        verify_candidate()
        dump(args.out / "archived11.json", check_archive(args.archive))
        run("archive-identity", [args.vector_port, "archive-identity", args.archive])
        common = ["--anvil", args.anvil, "--driver", args.driver, "--contracts", args.contracts]
        run("legacy-rollback", [sys.executable, "-B", LEGACY, *common, "--out", args.out / "legacy-rollback",
                                "--wall-pause-ms", 250])
        results = {"legacy_rollback": check_legacy(args.out / "legacy-rollback")}
        for case in args.cases:
            child = args.out / case
            run(case, [sys.executable, "-B", HERE / "live_fixture.py", "--case", case,
                       "--vector-port", args.vector_port, *common, "--out", child, "--wall-pause-ms", 250])
            results[case] = check_live(child, case)
        verify_candidate()
        manifest = {"status": "PASS", "candidate": args.candidate, "seed": "none (explicit fixture slots)", "checks": results,
                    "commands": commands, "artifact_sha256": {
                        str(path.relative_to(args.out)): hashlib.sha256(path.read_bytes()).hexdigest()
                        for path in sorted(args.out.rglob("*")) if path.is_file()
                    }}
        dump(args.out / "manifest.json", manifest)
    except BaseException:
        dump(args.out / "failed-commands.json", commands)
        raise
    print(json.dumps({"status": "PASS", "checks": list(results), "out": str(args.out)}))


if __name__ == "__main__":
    main()
