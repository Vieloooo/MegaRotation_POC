#!/usr/bin/env python3
"""Capture H3-C-1 offline acceptance with the reviewed source and unchanged EC16 replay."""

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time
from unittest import mock


REPO = Path(__file__).resolve().parents[5]
SCRIPT = Path(__file__).resolve().relative_to(REPO)
SLICE = SCRIPT.parent.parent
HARNESS = SLICE / "independent-harness"
FIXTURES = SLICE / "fixtures"
CRATE = Path("poc/crates/espresso-compatibility")
SCENARIO = Path("poc/scenarios/espresso-compatibility")
CHECKER = Path("poc/verification/espresso-compatibility/check_capture.py")
CANDIDATE = "bca02b095bac1e5701eb78e352d605c0ed887e10"
OFFICIAL_CANDIDATE = "6248072b3fd0eee1ae468f0961a4d3589291cba6"
BASELINE = Path("/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02")
BASELINE_MANIFEST_SHA = "08bdc1aec6b5735df90ab3ceeab7a6fe13868fd0dc8212c78158ddbf2fe61dfb"
TOOLCHAIN = "/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin"
LOCK = "/tmp/megaroation-poc-heavy.lock"
TIMING = {"proposed_at": "Unknown", "finalized_at": "Unknown", "live_at_proposal": "Unknown"}
PROFILES = {"single": 1, "duplicates-eleven": 11}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def read_json(path):
    return json.loads(path.read_bytes())


def git(*args):
    return subprocess.check_output(["git", *args], cwd=REPO)


def revision():
    info = {"head": git("rev-parse", "HEAD").decode().strip(),
            "branch": git("branch", "--show-current").decode().strip(),
            "status": git("status", "--porcelain=v1", "--untracked-files=all").decode().splitlines()}
    require(info["head"] == CANDIDATE and info["branch"] == "codex/h3-publication-gfr", "Unexpected checkout")
    return info


def tracked_inputs():
    paths = git("ls-files", "-z", str(CRATE), "poc/crates/sim-espresso", str(SCENARIO), str(CHECKER),
                str(CHECKER.parent / "vectors-v1.json"), str(SLICE),
                "poc/docs/contracts/h3-publication-gfr-slice-c-v1.md").decode().split("\0")
    paths = [path for path in paths if path]
    require(not git("diff", "--no-ext-diff", "--name-only", CANDIDATE, "--", *paths), "Tracked Slice C inputs changed")
    return {path: sha(REPO / path) for path in sorted(set(paths) | {str(SCRIPT)})}


def old_hashes():
    paths = [path for dirname in ("bundle", "offline-first", "offline-requery")
             for path in (BASELINE / dirname).rglob("*") if path.is_file()]
    return {str(path.relative_to(BASELINE)): sha(path) for path in sorted(paths)}


def record_fixture(name, root):
    """Use the candidate recorder with the verifier's independent, no-network HTTP model."""
    sys.path.insert(0, str(REPO / HARNESS))
    import fixture_model as model
    import tests as independent

    capture = independent.load_module(REPO / SCENARIO / "capture.py", "h3_c_tester_capture")
    inputs = root / "inputs"
    bundle = root / "bundle"
    inputs.mkdir(parents=True)
    bundle.mkdir()
    path = inputs / "input.json"
    shutil.copyfile(REPO / FIXTURES / f"{name}.json", path)
    shutil.copyfile(REPO / FIXTURES / "payload-000.bin", inputs / "payload-000.bin")
    payload = (inputs / "payload-000.bin").read_bytes()
    payloads = (payload,) * PROFILES[name]
    manifest, raw, loaded, report = capture.load_payload_manifest(path)
    require((manifest["evidence_kind"], raw, loaded, report) == ("synthetic-adapter", path.read_bytes(), payloads, None),
            "Frozen synthetic input drift")
    record, source = independent.mock_recorder(capture, bundle, payloads)
    identity = capture.copy_payload_inputs(record, (manifest, raw, loaded, report), CANDIDATE)
    for part in ("block", "leaf"):
        record.http("first", "GET", f"/v1/availability/{part}/0")
    with mock.patch.object(capture.time, "sleep", lambda _: None), \
         mock.patch.object(capture.http.client, "HTTPConnection", side_effect=AssertionError("real HTTP forbidden")):
        observations = capture.observe_publications(record, independent.QuietProcesses(), source[0][0], source[0][2],
                                                    payloads=payloads, input_identity=identity)
    source_identity = model.source_identity(f"h3-c-tester-{name}")
    sid = model.source_id(source_identity)
    for item in record.records:
        item["source_id"] = sid
    config = {"candidate_sha": CANDIDATE, "input_mode": "synthetic-adapter", "input_identity": identity,
              "classification": "SYNTHETIC_OFFLINE_NO_NODE_NO_QC_NO_NATIVE_PROOF"}
    write_json(bundle / "manifest.json", {"schema": "EC16-CAPTURE-1", "capture_kind": "fixture",
                                         "identity": source_identity, "source_id": sid, "configuration": config,
                                         "records": record.records, "artifacts": record.artifacts})
    (root / "expected-audit.bin").write_bytes(model.audit(source_identity, source))
    receipt = {"classification": config["classification"], "input_identity": identity,
               "capture_manifest_sha256": sha(bundle / "manifest.json"), "observer": observations,
               "source_timing": TIMING, "source_blocks": [0, 1], "source_transactions": 2}
    write_json(root / "driver-receipt.json", receipt)
    print(json.dumps(receipt, sort_keys=True))


class Acceptance:
    def __init__(self, output, launch):
        self.output = output
        self.output.mkdir(exist_ok=False)
        self.python = [sys.executable, "-B"]
        relevant = {"PATH": TOOLCHAIN + ":" + os.environ["PATH"], "CARGO_HOME": str(REPO / ".build/cargo"),
                    "CARGO_TARGET_DIR": str(REPO / ".build/target"), "CARGO_BUILD_JOBS": "2",
                    "PYTHONDONTWRITEBYTECODE": "1"}
        self.env = dict(os.environ, **relevant)
        self.binary = Path(relevant["CARGO_TARGET_DIR"]) / "debug/espresso-compatibility"
        self.meta = {"candidate": CANDIDATE, "contract": "H3-C-1", "worktree": str(REPO), "artifact_directory": str(output),
                     "revision_before": revision(), "sources_before": tracked_inputs(), "environment": relevant,
                     "outer_lock_command": launch, "commands": [], "profiles": {}, "seed": None,
                     "real_network_measurements": None, "verdict": "INCOMPLETE"}
        self.meta["tools"] = {cmd[0]: subprocess.check_output(cmd, cwd=REPO, env=self.env, text=True).strip()
                              for cmd in [["cargo", "-V"], ["rustc", "-Vv"], ["git", "--version"], ["flock", "--version"]]}
        self.meta["tools"]["python"] = sys.version
        require(self.meta["tools"]["rustc"].startswith("rustc 1.95.0 "), "Unexpected Rust toolchain")
        self.meta["saved_official_before"] = old_hashes()
        require(self.meta["saved_official_before"].get("bundle/manifest.json") == BASELINE_MANIFEST_SHA,
                "Saved official marker manifest hash differs from accepted artifact")
        require(not git("diff", "--no-ext-diff", "--name-only", OFFICIAL_CANDIDATE, CANDIDATE, "--", str(CRATE)),
                "EC16 crate differs from saved official run candidate")
        self.persist()

    def persist(self):
        write_json(self.output / "manifest.json", self.meta)

    def run_command(self, name, argv, *, timeout=30, expect=0, uses_rust=False):
        stdout = self.output / f"{name}.stdout"
        stderr = self.output / f"{name}.stderr"
        row = {"name": name, "argv": list(map(str, argv)), "stdout": stdout.name, "stderr": stderr.name,
               "watchdog_s": timeout, "timed_out": False, "expected_exit": expect}
        self.meta["commands"].append(row)
        revision()
        start = time.perf_counter_ns()
        with stdout.open("wb") as out, stderr.open("wb") as err, subprocess.Popen(
            row["argv"], cwd=REPO, env=self.env, stdout=out, stderr=err, start_new_session=True
        ) as process:
            try:
                process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                row["timed_out"] = True
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
        row.update(exit_code=process.returncode, local_wall_elapsed_ns=time.perf_counter_ns() - start,
                   stdout_sha256=sha(stdout), stderr_sha256=sha(stderr))
        if uses_rust:
            row["binary_sha256"] = sha(self.binary) if self.binary.is_file() else None
        self.persist()
        require(not row["timed_out"] and process.returncode == expect, f"{name} exit {process.returncode}; see {stderr}")
        revision()
        return stdout

    def replay(self, name, root, baseline=False):
        bundle = root / "bundle"
        candidate = OFFICIAL_CANDIDATE if baseline else CANDIDATE
        manifest = read_json(bundle / "manifest.json")
        outputs = self.output / f"{name}-replays"
        outputs.mkdir()
        variants = [("first", 0), ("requery", 0)] if baseline else [("first", 0), ("first", 37), ("requery", 0)]
        receipts = []
        audit = projection = None
        for reader, pause in variants:
            label = f"{name}-{reader}-{pause}"
            replay = outputs / f"{reader}-{pause}"
            self.run_command(label, [self.binary, "replay", bundle, reader, replay, "--pause-ms", pause], uses_rust=True)
            cli = self.python + [CHECKER, bundle, "--reader", reader, "--candidate", candidate,
                                 "--audit-file", replay / "public-audit.bin", "--projection-file", replay / "public-projection.json"]
            if not baseline:
                cli += ["--payload-manifest", root / "inputs/input.json"]
            audit_bytes = (replay / "public-audit.bin").read_bytes()
            projection_bytes = (replay / "public-projection.json").read_bytes()
            if audit is None:
                audit, projection = audit_bytes, projection_bytes
            require((audit_bytes, projection_bytes) == (audit, projection), f"{label} full outputs differ")
            if baseline:
                saved = BASELINE / f"offline-{reader}"
                require((audit, projection) == ((saved / "public-audit.bin").read_bytes(),
                                               (saved / "public-projection.json").read_bytes()), f"{label} differs from saved official replay")
            else:
                require(audit == (root / "expected-audit.bin").read_bytes(), f"{label} differs from model audit")
            checked = read_json(self.run_command(f"{label}-oracle", cli))
            receipt = read_json(replay / "receipts.json")
            blocks = json.loads(projection)["blocks"]
            require(all(block["timing"] == TIMING for block in blocks), "Replay projected known source timing")
            require(all(set(receipt[key]) == set(TIMING) for key in ("pe_log_unavailable", "pe_reader_unavailable")),
                    "Replay PE export timing missing fields changed")
            receipts.append({"reader": reader, "pause_ms": pause, "path": str(replay.relative_to(self.output)),
                             "sha256": sha(replay / "receipts.json"), "audit_sha256": sha(replay / "public-audit.bin"),
                             "projection_sha256": sha(replay / "public-projection.json")})
            require(checked["complete_blocks"] == (6 if baseline else 2), f"{label} prefix length differs")
            if not baseline:
                self.verify_ordinals(name, manifest, checked, receipt, blocks, root, reader)
        profile = {"classification": "recorded-official-marker-read-only" if baseline else "synthetic-adapter",
                   "capture_manifest_sha256": sha(bundle / "manifest.json"), "capture_runner_revision": candidate,
                   "receipts": receipts, "audit_sha256": receipts[0]["audit_sha256"],
                   "projection_sha256": receipts[0]["projection_sha256"], "complete_blocks": 6 if baseline else 2,
                   "source_timing": TIMING, "pe_exports": "Unsupported"}
        if not baseline:
            profile.update(input_ordinals=PROFILES[name], submit_records=PROFILES[name] + 1, source_occurrences=1,
                           input_manifest_sha256=sha(root / "inputs/input.json"),
                           payload_sha256=sha(root / "inputs/payload-000.bin"), distinct_runner_and_producer=True)
        self.meta["profiles"][name] = profile
        self.persist()

    def verify_ordinals(self, name, manifest, checked, receipt, blocks, root, reader):
        identity = checked["input_identity"]
        require(checked["input_mode"] == "synthetic-adapter" and checked["byte_equality"] is True
                and checked["publication_scope"] == "byte-publication-only" and checked["official_timing"] == TIMING,
                f"{name} checker classification or byte equality changed")
        require(identity == manifest["configuration"]["input_identity"], f"{name} independent input identity did not match recording")
        require(identity["native_verification"] == "NOT_PERFORMED" and identity["native_evidence"] is None
                and identity["producer_candidate_sha"] == "a" * 40 and identity["runner_candidate_sha"] == CANDIDATE,
                f"{name} provenance drift")
        rows = identity["messages"]
        flat = {"q": 1, "source_transaction_index": 1}
        occurrence = dict(flat, pos_in_namespace=0, message_index=0)
        require([row["ordinal"] for row in rows] == list(range(PROFILES[name])), f"{name} ordinal sequence changed")
        require(all(row["codec"] == 0x47465201 for row in rows), f"{name} GFR input codec changed")
        require(identity["repeat"]["ordinal"] == 0 and all(row["occurrences"] == [occurrence]
                and row["byte_equal"] is True for row in [*rows, identity["repeat"]]), f"{name} positional binding changed")
        payload = (root / "inputs/payload-000.bin").read_bytes()
        require(len(blocks[1]["native_transactions"]) == 2, f"{name} source namespace filtering not exercised")
        projected = blocks[1]["publication_messages"]
        require(projected == [{"codec": 0xEC160001, "message_index": 0, "source_position": flat,
                               "payload_base64": base64.b64encode(payload).decode()}], f"{name} opaque EC16 projection changed")
        if reader == "first":
            expected = [{"submit_record_id": row["submit_record_id"], "transaction_hash": row["ack_hash"],
                         "payload": list(payload), "positions": [flat]} for row in [*rows, identity["repeat"]]]
            require(receipt["publications"] == expected, f"{name} Rust submit receipts changed order, bytes, or positions")
        else:
            require(receipt["publications"] == [], f"{name} requery claimed it made submissions")

    def wrong_namespace(self):
        original = self.output / "single/bundle"
        changed = self.output / "wrong-namespace-bundle"
        shutil.copytree(original, changed)
        manifest = read_json(changed / "manifest.json")
        submit = next(row for row in manifest["records"] if row["method"] == "POST")
        request = changed / submit["request"]["path"]
        value = read_json(request)
        value["namespace"] = 9
        write_json(request, value)
        submit["request"]["sha256"] = sha(request)
        write_json(changed / "manifest.json", manifest)
        self.run_command("wrong-namespace-oracle", self.python + [CHECKER, changed], expect=1)
        require(b"submit namespace mismatch" in (self.output / "wrong-namespace-oracle.stderr").read_bytes(),
                "Wrong namespace fixture failed before reaching the namespace guard")

    def run(self):
        matrix = self.output / "matrix"
        self.run_command("matrix", self.python + [HARNESS / "tests.py", "--repo", REPO, "--candidate", CANDIDATE,
                                                 "--output", matrix], timeout=120)
        summary = read_json(matrix / "summary.json")
        require(summary["passed"] == 52 and summary["failed"] == 0 and summary["official_node"] == "NOT_RUN", "Expected 52 offline passes")
        self.meta["independent_cases"] = {"passed": 52, "failed": 0, "summary_sha256": sha(matrix / "summary.json")}
        self.run_command("build-ec16", ["cargo", "build", "--locked", "--offline", "--manifest-path", CRATE / "Cargo.toml"],
                         timeout=180, uses_rust=True)
        for profile in PROFILES:
            root = self.output / profile
            self.run_command(f"record-{profile}", self.python + [SCRIPT, "record", profile, root])
            self.replay(profile, root)
        self.wrong_namespace()
        vectors = read_json(self.run_command("legacy-vectors", self.python + [CHECKER, "--vectors", CHECKER.parent / "vectors-v1.json"]))
        require(vectors["frozen_prefix_vectors"] == 5 and vectors["status"] == "PASS", "Legacy marker vectors changed")
        self.meta["marker_vectors"] = vectors
        self.replay("saved-official-markers", BASELINE, baseline=True)
        require(len({row["binary_sha256"] for row in self.meta["commands"] if "binary_sha256" in row}) == 1,
                "EC16 binary hash changed during tests")

    def finish(self):
        self.meta.update(revision_after=revision(), sources_after=tracked_inputs(), saved_official_after=old_hashes())
        require(self.meta["sources_before"] == self.meta["sources_after"], "Captured source hashes changed")
        require(self.meta["saved_official_before"] == self.meta["saved_official_after"], "Saved official input bytes changed")
        self.meta["timeouts"] = [row["name"] for row in self.meta["commands"] if row["timed_out"]]
        self.meta["incomplete_profiles"] = sorted({*PROFILES, "saved-official-markers"} - self.meta["profiles"].keys())
        self.persist()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="operation", required=True)
    batch = sub.add_parser("run")
    batch.add_argument("output_name")
    record = sub.add_parser("record")
    record.add_argument("profile", choices=PROFILES)
    record.add_argument("output", type=Path)
    args = parser.parse_args()
    if args.operation == "record":
        revision()
        record_fixture(args.profile, args.output)
        return
    require(args.output_name.startswith("h3-c-bca02b-") and Path(args.output_name).name == args.output_name,
            "Use a new h3-c-bca02b-* artifact basename")
    output = (REPO / ".build/artifacts/simulation" / args.output_name).resolve()
    launch = ["flock", "--exclusive", "--timeout", "60", "--conflict-exit-code", "75", LOCK,
              "python3", "-B", str(SCRIPT), "run", args.output_name]
    acceptance = Acceptance(output, launch)
    try:
        acceptance.run()
        acceptance.finish()
        acceptance.meta["verdict"] = "PASS"
    except Exception as error:
        acceptance.meta["verdict"] = "FAIL"
        acceptance.meta["error"] = str(error)
        acceptance.finish()
        raise
    finally:
        acceptance.persist()
    print(f"PASS {output}", flush=True)


if __name__ == "__main__":
    main()
