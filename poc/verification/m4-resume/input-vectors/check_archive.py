#!/usr/bin/env python3
"""Verify sealed feature 11 receipt/DA evidence and its actual D config binding."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory

FEATURE11_CHECKERS = Path(__file__).resolve().parents[2] / "batch-services"
sys.path.insert(0, str(FEATURE11_CHECKERS))

import check_accepted_evidence as accepted_check
import check_execution_evidence as execution_check
import check_m2_driver_and_crash as driver_check

CANDIDATE_11 = "e5f77d27edde91a61f7400d1a61961e4cbe09c9c"
SEAL_11 = "ecda33340a99def14cb17253ada97eac6926a933e0dda6525926a5c9b6e78dce"


def require(condition, description):
    if not condition:
        raise ValueError(description)


def load(path):
    return json.loads(path.read_bytes())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_archive_seal(root):
    tree = root.parent.parent
    source_revision = (tree / "revision.txt").read_text().strip()
    require(source_revision == CANDIDATE_11, "unexpected feature 11 sample revision")
    sums = tree / "evidence-sha256.txt"
    require(sha256(sums) == SEAL_11, "feature 11 archived seal differs from the delivery manifest")
    result = {}
    for line in sums.read_text().splitlines():
        expected, archive_name = line.split(maxsplit=1)
        recorded = Path(archive_name)
        if root.parent.name not in recorded.parts:
            continue
        # The original producer addressed these same artifacts through a
        # checkout symlink. Resolve each entry relative to the provided run.
        prefix = recorded.parts.index(root.parent.name) + 1
        suffix = recorded.parts[prefix:]
        if not suffix or suffix[0] != root.name:
            continue
        relative = Path(*suffix[1:])
        path = root / relative
        require(path.resolve().is_relative_to(root.resolve()), f"escaping seal entry {relative}")
        require(str(relative) not in result, f"duplicate seal entry {relative}")
        require(sha256(path) == expected, f"archive SHA-256 mismatch for {relative}")
        result[str(relative)] = expected
    required = {
        "accepted-records.json", "driver-config.json", "driver-trace.json", "rpc-transcript.json",
        "raw-rpc-bundle.json", "da-index.json", "execution-source.json", "fixture-snapshot.json",
    }
    require(required <= result.keys(), "archive seal missing required inputs")
    return {"candidate": source_revision, "manifest_sha256": sha256(sums), "files": result}


def object_checks(root, bundle, index, opened):
    first, *remaining = opened
    position = first["position"]
    key = first["commitment"]
    entries = index["payloads"]

    missing = {"payloads": {name: path for name, path in entries.items() if name != key}}
    without_first = accepted_check.check(bundle, missing, root)
    require(without_first["status"] == "INCONCLUSIVE", "missing payload must block complete evidence")
    require([item["position"] for item in without_first["missing"]] == [position], "wrong missing log")
    require(without_first["missing"][0]["status"] == "WAIT_MISSING_DA", "wrong missing classification")
    require(without_first["accepted"] == remaining, "independent later log authentication changed")

    with TemporaryDirectory(prefix="m4-feature11-corrupt-") as directory:
        bad = Path(directory) / "corrupt.bin"
        original = (root / entries[key]).read_bytes()
        bad.write_bytes(bytes([original[0] ^ 1]) + original[1:])
        offered = {"payloads": {**entries, key: str(bad)}}
        try:
            accepted_check.check(bundle, offered, root)
        except ValueError as error:
            require("commitment mismatch" in str(error), "corruption rejected for unexpected reason")
        else:
            raise ValueError("corrupted DA bytes satisfied the original commitment")

    require(accepted_check.check(bundle, index, root)["accepted"] == opened, "exact byte reopening changed")
    return {"position": position, "commitment": key, "missing": "WAIT_MISSING_DA",
            "corrupt_object": "COMMITMENT_MISMATCH", "exact": "VERIFIED_PAYLOAD"}


def check_archive(root):
    seal = verify_archive_seal(root)
    bundle = load(root / "raw-rpc-bundle.json")
    index = load(root / "da-index.json")
    fixture = load(root / "fixture-snapshot.json")["config"]
    driver = load(root / "driver-config.json")
    accepted = accepted_check.check(bundle, index, root)
    require(accepted["status"] == "PASS", "archived accepted sources incomplete")
    expected = [([2, 0, 0], 1, 2), ([3, 0, 0], 3, 3)]
    positions = [(row["position"], row["first_height"], row["last_height"])
                 for row in accepted["accepted"]]
    require(positions == expected, "unexpected accepted ranges or positions")
    require(any(record["transactions"] for row in accepted["accepted"] for record in row["records"]),
            "archived accepted records have no ordinary transactions")
    for payload in index["payloads"].values():
        require(payload in seal["files"], "unsealed archive payload")

    execution = execution_check.check(load(root / "execution-source.json"), accepted)
    require(execution["status"] == "PASS", "archived G fixture execution mismatch")
    driver_result = driver_check.check(root)
    require(driver_result["status"] == "PASS", "archive driver/receipt audit failed")

    raw_genesis_timestamp_s = accepted_check.r.q(bundle["blocks"][0]["timestamp"])
    require(raw_genesis_timestamp_s == driver["genesis_timestamp_seconds"], "driver genesis claim differs")
    require(driver["config_ref"] == bundle["config"]["configRef"], "signed configRef differs")
    require(fixture["chain_id"] == driver["chain_id"], "driver chain id differs")
    require(fixture["l1_block_us"] == driver["slot_seconds"] * 1_000_000, "driver L1 slot differs")
    require(fixture["l1_genesis_timestamp_us"] == 0 and raw_genesis_timestamp_s == 1_700_000_000,
            "unexpected archived timestamp fixture")
    return {
        "status": "PASS",
        "scope": "sealed real accepted L1/DA and independent fixture Kept audit; D identity precondition",
        "seed": "none (archived explicit script)",
        "archive": seal,
        "accepted_receipts_and_payloads": accepted,
        "execution": execution,
        "driver": driver_result,
        "independent_object_checks": object_checks(root, bundle, index, accepted["accepted"]),
        "derivation_identity": {
            "validated_chain_id": driver["chain_id"],
            "signed_config_ref": driver["config_ref"],
            "fixture_l1_genesis_timestamp_us": fixture["l1_genesis_timestamp_us"],
            "actual_l1_genesis_timestamp_us": raw_genesis_timestamp_s * 1_000_000,
            "expected_strict_source_binding": "REJECT_TIMESTAMP_MISMATCH",
        },
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    report = check_archive(args.run)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(report, stream, indent=2)
        stream.write("\n")
    print(json.dumps({"status": report["status"], "payloads": report["accepted_receipts_and_payloads"]["accepted_count"],
                      "identity_binding": report["derivation_identity"]["expected_strict_source_binding"]}))


if __name__ == "__main__":
    main()
