#!/usr/bin/env python3
"""Builder smoke assertions against fresh receipts and the sealed verification vectors."""
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]


def load(path):
    return json.loads(path.read_text())


def coordinates(value):
    return [value[key] for key in ("block", "tx", "log")]


def hx(byte_array):
    return bytes(byte_array).hex()


def check_reference(case, result):
    named = {"window": "real_plan_window50", "ahead": "real_plan_ahead_two_markers"}
    cases = load(ROOT / "poc/verification/derivation/reference-results.json")["cases"]
    expected = next(row for row in cases if row["id"] == named[case])
    assert len(result["native"]) >= len(expected["blocks"])
    for actual, ref in zip(result["native"], expected["blocks"]):
        metadata = actual["metadata"]
        assert [metadata[k] for k in ("height", "timestamp_us", "origin")] == ref["metadata"]
        for key, expected_key in (("native_hash", "native_hash"), ("native_bytes", "native_bytes"),
                                  ("state_root", "post_state_root"), ("derived_anchor_id", "derived_anchor_id")):
            assert actual[key][2:] == ref[expected_key], (case, metadata, key)
    assert len(result["cuts"]) == len(expected["cuts"])
    for actual, ref in zip(result["cuts"], expected["cuts"]):
        assert coordinates(actual["resume_position"]) == ref["resume"]
        assert coordinates(actual["cursor_set_position"]) == ref["set"]
        assert actual["target_height"] == ref["target"]
        assert len(actual["derived_prefix"]) == ref["prefix_head"]
        hashes = [hx(item["native_hash"]) for item in actual["derived_prefix"]]
        assert hashes == ref["prefix_native_hashes"]
        anchor = actual["target_anchor"]
        assert hx(anchor["anchor"]["block"]["block_id"]) == ref["anchor_id"]
        assert hx(anchor["native_hash"]) == ref["anchor_native_hash"]
        assert anchor["state"] == actual["derived_prefix"][ref["target"] - 1]["state"]


def main():
    case, run, replay = sys.argv[1], Path(sys.argv[2]), sys.argv[3]
    a, b = (load(run / f"d-{reader}.json") for reader in ("a", "b"))
    for key in ("head", "native", "derived", "cuts", "canonical_checkpoint", "action_trace", "checkpoints"):
        assert a[key] == b[key], (case, key)
    assert any(s["block_read"]["status"] == "wait" for s in b["snapshots"])
    epochs = {s["requested"]: s for s in a["snapshots"]}
    assert epochs[2]["head"] == 0 and epochs[2]["epoch_has_output"] is False
    receipts = load(run / "raw-rpc-bundle.json")["receipts"]
    assert any(r["status"] == "0x0" and not r["logs"] for r in receipts)
    assert any(r["status"] == "0x1" and not r["logs"] for r in receipts)
    if case in ("ahead", "window"):
        check_reference(case, a)
    if case == "window":
        assert epochs[49]["frontier"]["height"] == 49 and epochs[49]["head"] == 20
        assert epochs[50]["frontier"]["height"] == 50 and epochs[50]["head"] == 22
        assert epochs[49]["waiting"]["reason"] == {"kind": "complete_window", "through": 50}
        assert epochs[50]["waiting"]["reason"] == {"kind": "complete_window", "through": 51}
    if case == "ahead":
        assert epochs[44]["head"] == 10 and epochs[45]["head"] == 12
        assert [c["target_height"] for c in a["cuts"][-2:]] == [8, 11]
        checkpoint = a["checkpoints"][-2]
        assert checkpoint["cuts"][-1]["resume_position"] == {"block": 45, "tx": 0, "log": 1}
        saved = run / "checkpoint-first-45.json"
        saved.write_text(json.dumps(checkpoint, sort_keys=True) + "\n")
        resumed = run / "d-a-resumed.json"
        subprocess.run([replay, "replay", str(run), "A", str(resumed), str(saved)], check=True)
        result = load(resumed)
        assert result.pop("independent_process_checkpoint_replayed") is True
        normal = dict(a)
        assert normal.pop("independent_process_checkpoint_replayed") is False
        assert result == normal
    if case == "cap":
        assert a["cuts"][0]["target_height"] == 30 and epochs[3]["head"] == 30
        assert a["head"]["anchor"]["block"]["origin"] == 3
        assert a["head"]["anchor"]["timestamp_us"] == 1_030_000_000
        assert all(row["metadata"]["origin"] <= 3 for row in a["native"])
    if case == "accepted_wait":
        assert a["native"] == [] and a["cuts"] == []
        assert epochs[39]["frontier"]["height"] == 39
        for i in (40, 41):
            summary = epochs[i]
            assert summary["frontier"]["height"] == 39 and summary["head"] == 0
            assert summary["waiting"]["position"] == {"kind": "log", "at": {"block": 40, "tx": 0, "log": 0}}
            assert summary["waiting"]["reason"]["kind"] == "feature_11_accepted_payload_required"
        block = epochs[40]["block_read"]["result"]["block"]
        assert [event["event"] for event in block["events"]] == ["cursor_advanced", "cursor_set", "resume"]
        assert bytes(epochs[40]["waiting"]["reason"]["commitment"]).hex() == block["events"][0]["batch"]["commitment"][2:]
    print(json.dumps({"case": case, "head": a["head"]["anchor"]["block"]["height"], "cuts": len(a["cuts"]), "status": "PASS"}))


if __name__ == "__main__":
    main()
