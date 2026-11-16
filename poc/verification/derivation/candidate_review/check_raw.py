#!/usr/bin/env python3
"""Verifier: derive Stage-A outputs from fresh raw 09 receipts and frozen 03 oracle.

No task-12 production module or builder checker is imported. The inherited 09
independent receipt checker authenticates ABI/signatures/positions; its scope
remains trusted local Anvil, not Ethereum consensus or MPT proof verification.
"""
import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "verification/anvil-l1"))
import receipt_checker

spec = importlib.util.spec_from_file_location("frozen_derivation_oracle", HERE.parent / "independent_reference.py")
oracle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(oracle)


def load(path):
    return json.loads(path.read_text())


def pos(value):
    return [value[key] for key in ("block", "tx", "log")]


def hx(value):
    return bytes(value).hex()


def check_case(run):
    bundle = load(run / "raw-rpc-bundle.json")
    receipts = receipt_checker.Checker(bundle).run()
    profile = load(run / "derivation-profile.json")
    config = profile["config"]
    public = config["public"]
    genesis = profile["genesis"]
    genesis_metadata = genesis["state"]["metadata"]
    metadata = [genesis_metadata[k] for k in ("height", "timestamp_us", "origin")]
    accounts = copy.deepcopy(genesis["state"]["accounts"])
    native_parent = hx(genesis["native_hash"])
    physical_parent = hx(genesis["anchor"]["block"]["block_id"])
    headers = {int(h["number"], 16): h for h in bundle["blocks"]}
    groups = {}
    for event in receipts["events"]:
        groups.setdefault(event["position"][0], []).append(event)
    assert bundle["config"]["configRef"] == profile["config_ref"]
    assert genesis_metadata == {"height": 0, "timestamp_us": 1_000_000_000, "origin": 0}
    assert public["l2_slot_us"] == 1_000_000 and config["sequence_window_blocks"] == 40
    dep = profile["deposits"]
    blocks, cuts = [], []
    epoch_has_output = False
    frontier = None
    blocked = None

    def emit(origin, cause):
        nonlocal metadata, accounts, native_parent, physical_parent, epoch_has_output
        next_metadata = [metadata[0] + 1, metadata[1] + public["l2_slot_us"], origin]
        assert next_metadata[1] >= int(headers[origin]["timestamp"], 16) * 1_000_000
        credits = dep["rows"][next_metadata[0] - dep["first_height"]]
        credits = [[row["to"], row["amount"]] for row in credits]
        out = oracle.execute(metadata, accounts, native_parent, next_metadata, credits, [])
        physical = (oracle.be(1, 2) + bytes.fromhex(physical_parent)
                    + oracle.metadata_bytes(next_metadata) + bytes.fromhex(out["pre_state_root"])
                    + oracle.byte_vector([]))
        out["physical_parent"] = physical_parent
        out["physical_hash"] = oracle.digest("PoC/PhysicalBlock/v1", physical)
        out["cause"] = cause
        blocks.append(out)
        metadata, accounts = next_metadata, out["accounts"]
        native_parent, physical_parent = out["native_hash"], out["physical_hash"]
        epoch_has_output = True

    for number in sorted(headers):
        for event in groups.get(number, []):
            if event["kind"] == "CursorAdvanced":
                blocked = event["position"]
                break
            if event["kind"] == "CursorSet":
                target = event["cursor"][0]
                while metadata[0] < target:
                    e, t = metadata[2], metadata[1] + public["l2_slot_us"]
                    origin = e + 1 if (e + 1 <= number and int(headers[e + 1]["timestamp"], 16) * 1_000_000 <= t) else e
                    emit(origin, {"kind": "cursor_set", "position": dict(zip(("block", "tx", "log"), event["position"]))})
                anchor = blocks[target - 1]
                cuts.append({"set": event["position"], "resume": event["resumePosition"], "target": target,
                             "prefix_hashes": [item["native_hash"] for item in blocks], "anchor": anchor})
        if blocked:
            break
        frontier = number
        while number >= metadata[2] + config["sequence_window_blocks"]:
            e, t = metadata[2], metadata[1] + public["l2_slot_us"]
            origin = e if not epoch_has_output or t < int(headers[e + 1]["timestamp"], 16) * 1_000_000 else e + 1
            emit(origin, {"kind": "fallback", "window_end": e + config["sequence_window_blocks"],
                          "frontier": {"height": number, "hash": list(bytes.fromhex(headers[number]["hash"][2:]))}})

    def compare(actual):
        assert len(actual["derived"]) == len(blocks) == len(actual["native"])
        for item, wire, expected in zip(actual["derived"], actual["native"], blocks):
            meta = dict(zip(("height", "timestamp_us", "origin"), expected["metadata"]))
            assert wire["metadata"] == meta and item["state"]["metadata"] == meta
            for key, refkey in (("native_bytes", "native_bytes"), ("native_hash", "native_hash"),
                                ("state_root", "post_state_root"), ("derived_anchor_id", "derived_anchor_id")):
                assert wire[key] == "0x" + expected[refkey], (key, meta)
            assert item["state"]["accounts"] == expected["accounts"]
            assert hx(item["native_hash"]) == expected["native_hash"]
            assert hx(item["native_block"]["parent"]) == expected["native_parent"]
            assert hx(item["native_block"]["post_state_root"]) == expected["post_state_root"]
            assert [hx(raw) for raw in item["native_block"]["dep"]] == expected["dep"]
            assert item["native_block"]["kept"] == []
            assert hx(item["physical_block"]["parent"]) == expected["physical_parent"]
            assert hx(item["physical_block"]["pre_state_root"]) == expected["pre_state_root"]
            assert item["physical_block"]["metadata"] == meta and item["physical_block"]["transactions"] == []
            assert item["virtual_block"]["metadata"] == meta and item["virtual_block"]["transactions"] == []
            assert hx(item["virtual_block"]["pre_state_root"]) == expected["pre_state_root"]
            assert [hx(raw) for raw in item["virtual_block"]["dep"]] == expected["dep"]
            assert item["cause"] == expected["cause"]
        assert len(actual["cuts"]) == len(cuts)
        for cut, expected in zip(actual["cuts"], cuts):
            assert pos(cut["resume_position"]) == expected["resume"]
            assert pos(cut["cursor_set_position"]) == expected["set"]
            assert cut["target_height"] == expected["target"]
            assert [hx(item["native_hash"]) for item in cut["derived_prefix"]] == expected["prefix_hashes"]
            anchor = cut["target_anchor"]
            assert hx(anchor["native_hash"]) == expected["anchor"]["native_hash"]
            assert hx(anchor["anchor"]["block"]["block_id"]) == expected["anchor"]["derived_anchor_id"]
            assert anchor["state"]["accounts"] == expected["anchor"]["accounts"]
            assert [anchor["state"]["metadata"][k] for k in ("height", "timestamp_us", "origin")] == expected["anchor"]["metadata"]
            n, tx, local = expected["resume"]
            receipt = next(r for r in bundle["receipts"] if int(r["blockNumber"],16) == n and int(r["transactionIndex"],16) == tx)
            assert hx(cut["evidence"]["transaction_hash"]) == receipt["transactionHash"][2:]
            assert cut["evidence"]["resume_rpc_log_index"] == int(receipt["logs"][local]["logIndex"],16)
            assert cut["evidence"]["cursor_set_rpc_log_index"] == int(receipt["logs"][expected["set"][2]]["logIndex"],16)
            assert cut["evidence"]["including_block"]["height"] == n
            assert hx(cut["evidence"]["including_block"]["hash"]) == headers[n]["hash"][2:]
        assert actual["canonical_checkpoint"]["covered"][-1]["reference"]["height"] == frontier
        if blocked:
            assert not blocks and not cuts
            assert pos(actual["canonical_checkpoint"]["cursor"]["next_log"]) == blocked
        return True

    a, b = load(run / "d-a.json"), load(run / "d-b.json")
    compare(a); compare(b)
    for field in ("native", "derived", "cuts", "head", "canonical_checkpoint", "action_trace"):
        assert a[field] == b[field]
    negatives = []
    if blocks:
        bad = copy.deepcopy(a)
        bad["native"][0]["native_hash"] = "0x" + "00" * 32
        try:
            compare(bad)
        except AssertionError:
            negatives.append("changed_native_hash_rejected")
        else:
            raise AssertionError("checker accepted changed native hash")
    if cuts:
        bad = copy.deepcopy(a)
        bad["cuts"][0]["target_height"] += 1
        try:
            compare(bad)
        except AssertionError:
            negatives.append("changed_cut_target_rejected")
        else:
            raise AssertionError("checker accepted changed cut target")
    missing = copy.deepcopy(bundle)
    missing["receipts"].pop()
    try:
        receipt_checker.Checker(missing).run()
    except ValueError:
        negatives.append("missing_receipt_rejected")
    else:
        raise AssertionError("receipt checker accepted missing receipt")
    return {"run": str(run), "status": "PASS", "headers": len(headers), "receipts": len(bundle["receipts"]),
            "derived": len(blocks), "cuts": len(cuts), "frontier": frontier, "blocked_at": blocked,
            "checker_negative_controls": negatives,
            "source_sha256": hashlib.sha256((run / "raw-rpc-bundle.json").read_bytes()).hexdigest(),
            "head_native_hash": blocks[-1]["native_hash"] if blocks else native_parent}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("artifact_root", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    results = [check_case(args.artifact_root / f"{case}-{pause}")
               for case in ("window", "ahead", "cap", "accepted_wait") for pause in (300, 650)]
    report = {"status": "PASS", "candidate": "d0afeee9363ef6703f51eecfd742915f83aa27f2",
              "scope": "raw receipt/signature checks plus independent Stage-A derivation and 03 bytes",
              "seed": "none (explicit scripts)", "runs": results,
              "limits": ["trusted private Anvil RPC; no Ethereum proof verification", "accepted opaque payload tests Stage-A gate only", "no feature-11 decoding or DA persistence", "no BFT process recovery"]}
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": "PASS", "runs": len(results), "native_blocks_compared_per_reader": sum(r["derived"] for r in results), "cuts_compared_per_reader": sum(r["cuts"] for r in results)}))


if __name__ == "__main__":
    main()
