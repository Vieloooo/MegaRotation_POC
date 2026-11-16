#!/usr/bin/env python3
"""Tiny preparation artifact checks only. No candidate imports or protocol execution."""
from pathlib import Path
import hashlib
import json
import struct
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
checks = []


def check(name, predicate):
    if not predicate:
        raise AssertionError(name)
    checks.append(name)


freeze = json.loads((HERE / "expectations-freeze-v1.json").read_text())
check("source_expectations_freeze", hashlib.sha256(
    (ROOT / freeze["path"]).read_bytes()).hexdigest() == freeze["sha256"])
check("source_expectations_28_cases", sum(
    line.startswith("| H3-") for line in (ROOT / freeze["path"]).read_text().splitlines()) == 28)
data = json.loads((HERE / "literal-vectors-v1.json").read_text())
vs = {v["id"]: v for v in data["vectors"]}
check("15_unique_preparation_vectors", len(vs) == len(data["vectors"]) == 15)
check("no_candidate_execution_claim", data["candidate_execution"] == "NOT_RUN")
v = vs["L02-public-body-literal"]
body = bytes.fromhex("0001") + bytes.fromhex("11" * 32)
body += struct.pack(">QQQ", 11, 11_000_000, 7) + bytes.fromhex("22" * 32) + bytes(4)
public = b"GFRP" + bytes.fromhex("0001000000005e") + body
check("03_structural_literal_94", len(body) == 94 and body.hex() == v["physical_hex"])
check("07_public_literal_105", len(public) == 105 and public.hex() == v["public_hex"])
check("frozen_public_codec", v["public_codec"] == 0x47465201)
check("literal_public_hash", hashlib.sha256(public).hexdigest() == v["public_sha256"])
life = vs["L03-lifecycle-identity"]
position = bytes.fromhex("00010000000000000064040000000000000007")
check("position_literal_19", len(position) == 19 and position.hex() == life["position_hex"])
prefix = bytes.fromhex("000100000013") + position
check("close_projection_43", (prefix + b"\x01" + bytes(16) + b"\x01").hex()
      == life["events"][0]["projection_hex"])
check("branch_projection_35", (prefix + b"\x02" + bytes(8) + b"\x04").hex()
      == life["events"][1]["projection_hex"])
for i, case in enumerate(vs["L12-whole-attempt-size"]["cases"]):
    check(f"pelg_boundary_{i}", 48 + sum(8 + n for n in case["message_lengths"])
          == case["encoded_block_bytes"])
fan = vs["L15-transport-fanout"]
check("illustrative_fanout", fan["fragment_count"] == (1_048_576 + 65_280 - 1) // 65_280
      and fan["framed_copy_bytes"] == 1_048_576 + 17 * 256
      and fan["four_copy_bytes"] == fan["framed_copy_bytes"] * 4
      and fan["fifteen_copy_bytes"] == fan["framed_copy_bytes"] * 15
      and fan["four_copy_bytes"] > fan["default_global_queue_payload_bytes"])
check("proposed_60k_profile_arithmetic", 1_048_576 == 17 * 61_440 + 4096
      and 61_440 + 4096 == 65_536)
pins = json.loads((HERE / "source-pins-v1.json").read_text())
for entry in pins["source_entries"]:
    if entry.get("access", "").startswith("git show"):
        content = subprocess.check_output(
            ["git", "show", entry["git_revision"] + ":" + entry["path"]], cwd=ROOT)
    else:
        content = (ROOT / entry["path"]).read_bytes()
    check("source_sha256:" + entry["path"], hashlib.sha256(content).hexdigest() == entry["sha256"])
print(json.dumps({"status": "PASS_PREPARATION_SELF_CHECK_ONLY", "checks": len(checks),
                  "vectors": len(vs), "candidate_executions": 0,
                  "native_integration_official": "NOT_RUN", "seed": None}, sort_keys=True))
