#!/usr/bin/env python3
"""Run and check the frozen H3-A-1 acceptance cases under the host heavy-task lock."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import shlex
import signal
import subprocess
import time


WORKTREE = Path(__file__).resolve().parents[5]
CANDIDATE = "e59d026e8c08b64ff744b57c63cc7756f0b34019"
BRANCH = "codex/h3-publication-gfr"
ACCEPTED_07 = "8907ad64d882673d6ef2142defdf6bff2e208a71"
LOCK = "/tmp/megaroation-poc-heavy.lock"
TOOLCHAIN = "/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin"
SIMULATION = WORKTREE / ".build/artifacts/simulation"
BASELINE = SIMULATION / "gfr-8907ad6-g1-g2-v1"
OLD_SUITE = Path("poc/verification/viewer-gfr/simulation")
HARNESS = Path("poc/verification/publication-gfr-healthy/slice-a/independent-harness")
HARNESS_CASE = "slice-a-independent"
VIEWER = Path("poc/crates/viewer-gfr")
REPLAYS = [(scenario, 42, pause) for scenario in ("G1", "G2") for pause in (0, 2)]
WATCHDOG_S = 180


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def git(*args):
    return subprocess.check_output(["git", *args], cwd=WORKTREE)


def revision():
    return {"head": git("rev-parse", "HEAD").decode().strip(),
            "branch": git("branch", "--show-current").decode().strip(),
            "status": git("status", "--porcelain=v1", "--untracked-files=all").decode().splitlines()}


def check_revision(value):
    if (value["head"], value["branch"]) != (CANDIDATE, BRANCH):
        raise ValueError(f"Unexpected checkout {value['branch']} {value['head']}")
    if any(not line.startswith("?? ") for line in value["status"]):
        raise ValueError(f"Tracked worktree changes at {value['head']}")


def input_hashes():
    tracked = git("ls-files", "-z", "poc/crates", "poc/Cargo.toml", "poc/Cargo.lock", "poc/proto",
                  "poc/docs/contracts/h3-publication-gfr-implementation-v1.md",
                  "poc/docs/contracts/h3-publication-gfr-slice-a-v1.md", str(HARNESS), str(OLD_SUITE),
                  str(HARNESS.parent / "wire-vectors-v1.json"),
                  str(HARNESS.parent / "source-expectations-v1.md")).decode().split("\0")
    paths = {path for path in tracked if path}
    paths.update({str(HARNESS / "Cargo.lock"), str(Path(__file__).resolve().relative_to(WORKTREE))})
    return {path: sha(WORKTREE / path) for path in sorted(paths)}


def test_command(profile, path):
    return ["cargo", profile, "--locked", "--offline", "--manifest-path", str(path / "Cargo.toml")]


def check_harness(stdout, stderr, env):
    outcomes = re.findall(rb"^test (\S+) \.\.\. ok$", stdout, re.MULTILINE)
    if len(set(outcomes)) != 14 or not re.search(
        rb"^test result: ok\. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;", stdout, re.MULTILINE
    ):
        raise ValueError("The independent harness did not report 14 distinct passing tests")
    match = re.search(rb"Running tests/slice_a\.rs \(([^)]+)\)", stderr)
    if match is None:
        raise ValueError("Cargo did not report the independent harness executable")
    observed = (WORKTREE / os.fsdecode(match[1])).resolve()
    if observed.parent != (Path(env["CARGO_TARGET_DIR"]) / "debug/deps").resolve():
        raise ValueError(f"Unexpected independent harness executable {observed}")
    return {"passed": 14, "tests": [name.decode() for name in outcomes],
            "binary": str(observed), "binary_sha256": sha(observed)}


def check_replay(stdout, stderr, replay, semantic_trace, accepted, env):
    scenario, seed, pause = replay
    case = f"{scenario}-{seed}-{pause}"
    old = (BASELINE / f"{case}.jsonl").read_bytes()
    if digest(old) != accepted[case]["stdout_sha256"]:
        raise ValueError(f"Accepted 07 raw trace hash mismatch for {case}")
    if stdout.splitlines()[0] != old.splitlines()[0]:
        raise ValueError(f"Source line differs from accepted 07 for {case}")
    checked = semantic_trace(stdout, scenario, seed)
    for key in checked:
        if checked[key] != accepted[case][key]:
            raise ValueError(f"{case} {key} differs from accepted 07")
    if stdout != old:
        raise ValueError(f"Full replay stdout differs from accepted 07 for {case}")
    last_key = checked["virtual_espresso_keys_us"][-1]
    tip = f"Some(({len(checked['virtual_espresso_keys_us']) - 1}, PublicTimeUs({last_key})))"
    if stderr.decode().splitlines()[-1] != f"{scenario} seed={seed} wall_pause_ms={pause} A_tip={tip} B_tip={tip}":
        raise ValueError(f"Replay stderr case identity mismatch for {case}")
    binary = Path(env["CARGO_TARGET_DIR"]) / "debug/examples/replay"
    return {**checked, "accepted_raw_trace": str(BASELINE / f"{case}.jsonl"),
            "accepted_raw_trace_sha256": digest(old), "byte_identical_to_accepted_07": True,
            "binary": str(binary), "binary_sha256": sha(binary)}


def capture_case(output, env, record, verify):
    case = record["case"]
    command = record["command"]
    stdout = output / record["stdout"]
    stderr = output / record["stderr"]
    record["revision_before"] = revision()
    check_revision(record["revision_before"])
    (output / f"{case}.command.sh").write_text(shlex.join(command) + "\n")
    record.update(watchdog_s=WATCHDOG_S, timed_out=False, complete=False)
    started = time.perf_counter_ns()
    try:
        with stdout.open("wb") as out, stderr.open("wb") as err, subprocess.Popen(
            command, cwd=WORKTREE, env=env, stdout=out, stderr=err, start_new_session=True
        ) as process:
            try:
                process.wait(timeout=WATCHDOG_S)
            except subprocess.TimeoutExpired:
                record["timed_out"] = True
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
        record["exit_code"] = process.returncode
        record["local_wall_elapsed_ns"] = time.perf_counter_ns() - started
        record["stdout_sha256"] = sha(stdout)
        record["stderr_sha256"] = sha(stderr)
        if record["timed_out"] or process.returncode:
            raise RuntimeError(f"{case} exit {process.returncode}, timeout {record['timed_out']}")
        record.update(verify(stdout.read_bytes(), stderr.read_bytes()))
        record["complete"] = True
        print(f"{case} PASS ({record['local_wall_elapsed_ns'] / 1e9:.3f} s local wall)", flush=True)
    finally:
        record["revision_after"] = revision()
        write_json(output / f"{case}.run.json", record)
        check_revision(record["revision_after"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output_name", help="New task07 artifact directory basename")
    args = parser.parse_args()
    if not re.fullmatch(r"h3-a-e59d026-[a-zA-Z0-9-]+", args.output_name):
        parser.error("Use a new h3-a-e59d026-* directory basename")
    output = (SIMULATION / args.output_name).resolve()
    script = str(Path(__file__).resolve().relative_to(WORKTREE))
    lock_command = ["flock", "--exclusive", "--timeout", "60", "--conflict-exit-code", "75",
                    LOCK, "python3", "-B", script, args.output_name]
    env = os.environ.copy()
    relevant = {"PATH": TOOLCHAIN + ":" + env["PATH"], "CARGO_HOME": str(WORKTREE / ".build/cargo"),
                "CARGO_TARGET_DIR": str(WORKTREE / ".build/target"), "CARGO_BUILD_JOBS": "2"}
    env.update(relevant)
    initial = revision()
    check_revision(initial)
    accepted_path = OLD_SUITE / "summary.json"
    accepted_bytes = (WORKTREE / accepted_path).read_bytes()
    if accepted_bytes != git("show", f"{CANDIDATE}:{accepted_path}"):
        raise ValueError("Accepted 07 summary differs from the candidate Git object")
    accepted_summary = json.loads(accepted_bytes)
    if accepted_summary["candidate"] != ACCEPTED_07 or accepted_summary["verdict"] != "PASS":
        raise ValueError("Unexpected accepted 07 summary provenance")
    accepted = {row["case"]: row for row in accepted_summary["cases"] if row["verdict"] == "PASS"}
    semantic_trace = runpy.run_path(str(WORKTREE / OLD_SUITE / "check-traces.py"))["semantic_trace"]
    cases = [{"case": HARNESS_CASE, "profile": "SYNTHETIC_NATIVE_BOUNDARY_NOT_CRYPTO", "seed": None,
              "command": test_command("test", HARNESS) + ["--", "--test-threads=1"],
              "stdout": f"{HARNESS_CASE}.stdout", "stderr": f"{HARNESS_CASE}.stderr"}]
    for scenario, seed, pause in REPLAYS:
        case = f"{scenario}-{seed}-{pause}"
        cases.append({"case": case, "profile": accepted_summary["verification_profile"],
                      "scenario": scenario, "seed": seed, "pause_ms": pause,
                      "command": test_command("run", VIEWER) + ["--example", "replay", "--",
                                                               scenario, str(seed), str(pause)],
                      "stdout": f"{case}.jsonl", "stderr": f"{case}.stderr"})
    data = {"candidate": CANDIDATE, "contract": "H3-A-1", "worktree": str(WORKTREE),
            "artifact_directory": str(output), "revision_before": initial, "environment": relevant,
            "outer_lock_command": lock_command, "source_sha256_before": input_hashes(),
            "accepted_07_revision": ACCEPTED_07, "accepted_07_summary_sha256": digest(accepted_bytes),
            "local_wall_clock": "perf_counter_ns around each Cargo command, within the acquired host lock",
            "real_network_measurements": None, "cases": cases, "pause_pairs": [], "verdict": "INCOMPLETE"}
    data["tools"] = {cmd[0]: subprocess.check_output(cmd, cwd=WORKTREE, env=env, text=True).strip()
                     for cmd in [["cargo", "-V"], ["rustc", "-Vv"], ["python3", "-V"],
                                 ["git", "--version"], ["flock", "--version"]]}
    output.mkdir(exist_ok=False)
    manifest = output / "manifest.json"
    write_json(manifest, data)
    try:
        capture_case(output, env, cases[0], lambda out, err: check_harness(out, err, env))
        for case, replay in zip(cases[1:], REPLAYS, strict=True):
            capture_case(output, env, case, lambda out, err: check_replay(
                out, err, replay, semantic_trace, accepted, env
            ))
        if len({row["binary_sha256"] for row in cases[1:]}) != 1:
            raise ValueError("Replay binary hash changed across cases")
        for first, second in ((cases[1], cases[2]), (cases[3], cases[4])):
            if (output / first["stdout"]).read_bytes() != (output / second["stdout"]).read_bytes():
                raise ValueError(f"{first['scenario']} pause traces differ")
            data["pause_pairs"].append({"scenario": first["scenario"], "seed": first["seed"],
                                        "byte_identical": True, "stdout_sha256": first["stdout_sha256"]})
        data["revision_after"] = revision()
        check_revision(data["revision_after"])
        data["source_sha256_after"] = input_hashes()
        if data["source_sha256_before"] != data["source_sha256_after"]:
            raise ValueError("Input hashes changed during acceptance")
        data["verdict"] = "PASS"
    except Exception as error:
        data["verdict"] = "FAIL" if isinstance(error, (ValueError, AssertionError)) else "INCOMPLETE"
        data["error"] = str(error)
        raise
    finally:
        data["revision_after"] = revision()
        data["source_sha256_after"] = input_hashes()
        data["incomplete_cases"] = [case["case"] for case in cases if not case.get("complete")]
        data["timeouts"] = [case["case"] for case in cases if case.get("timed_out")]
        write_json(manifest, data)


if __name__ == "__main__":
    main()
