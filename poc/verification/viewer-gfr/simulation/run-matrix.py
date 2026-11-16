#!/usr/bin/env python3
"""Capture the exact verified G1/G2 runs, with a shared host lock per case."""

import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import time

CANDIDATE = "8907ad64d882673d6ef2142defdf6bff2e208a71"
WORKTREE = Path("/home/jy/.codex/worktrees/1e2f/megaroation_poc")
TOOLCHAIN = "/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin"
DEFAULT_OUTPUT = WORKTREE / ".build/artifacts/simulation/gfr-8907ad6-g1-g2-v1"
LOCK = "/tmp/megaroation-poc-heavy.lock"
CASES = list(itertools.product(("G1", "G2"), (7, 42, 99), (0, 2)))
MANIFEST = "poc/crates/viewer-gfr/Cargo.toml"


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, payload):
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def environment():
    env = os.environ.copy()
    relevant = {
        "PATH": TOOLCHAIN + ":" + env["PATH"],
        "CARGO_HOME": str(WORKTREE / ".build/cargo"),
        "CARGO_TARGET_DIR": str(WORKTREE / ".build/target"),
        "CARGO_BUILD_JOBS": "2",
    }
    env.update(relevant)
    return env, relevant


def git(*args):
    return subprocess.check_output(["git", *args], cwd=WORKTREE, text=True)


def capture_revision():
    head = git("rev-parse", "HEAD").strip()
    branch = git("branch", "--show-current").strip()
    status = git("status", "--porcelain=v1", "--untracked-files=all").splitlines()
    if head != CANDIDATE or branch != "codex/poc-viewer-gfr":
        raise RuntimeError(f"unexpected candidate {branch} {head}")
    if any(not line.startswith("?? ") for line in status):
        raise RuntimeError(f"tracked changes at {head}: {status}")
    return {"head": head, "branch": branch, "status": status}


def source_hashes():
    paths = git(
        "ls-files", "poc/crates/viewer-gfr", "poc/crates/authorization",
        "poc/crates/core", "poc/crates/types", "poc/crates/sim-espresso",
        "poc/docs/contracts/task-contract-viewer-gfr-v1.md",
        "poc/verification/viewer-gfr/source-first-expectations-v1.md",
    ).splitlines()
    return {path: sha(WORKTREE / path) for path in paths}


def run_case(output, env, binary, scenario, seed, pause):
    case = f"{scenario}-{seed}-{pause}"
    command = [
        "flock", "--exclusive", "--timeout", "60", LOCK, "cargo", "run",
        "--locked", "--offline", "--manifest-path", MANIFEST,
        "--example", "replay", "--", scenario, str(seed), str(pause),
    ]
    meta = {
        "case": case, "scenario": scenario, "seed": seed,
        "pause_ms": pause, "command": command, "revision_before": capture_revision(),
        "watchdog_s": 180, "lock_timeout_s": 60, "timed_out": False,
    }
    (output / f"{case}.command.sh").write_text(shlex.join(command) + "\n")
    stdout_path = output / f"{case}.jsonl"
    stderr_path = output / f"{case}.stderr"
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        start = time.perf_counter_ns()
        with subprocess.Popen(
            command, cwd=WORKTREE, env=env, stdout=stdout, stderr=stderr,
            start_new_session=True,
        ) as proc:
            try:
                proc.wait(timeout=meta["watchdog_s"])
            except subprocess.TimeoutExpired:
                meta["timed_out"] = True
                os.killpg(proc.pid, signal.SIGTERM)
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(proc.pid, signal.SIGKILL)
                    proc.wait()
        meta["local_wall_elapsed_ns"] = time.perf_counter_ns() - start
    meta.update(
        exit_code=proc.returncode,
        stdout_sha256=sha(stdout_path),
        stderr_sha256=sha(stderr_path),
        binary_sha256=sha(binary) if binary.is_file() else None,
    )
    write_json(output / f"{case}.run.json", meta)
    meta["revision_after"] = capture_revision()
    write_json(output / f"{case}.run.json", meta)
    print(
        f"{case}: exit {meta['exit_code']}, watchdog {meta['timed_out']}, "
        f"local wall {meta['local_wall_elapsed_ns'] / 1e9:.3f} s",
        flush=True,
    )
    return meta


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", default=DEFAULT_OUTPUT, type=Path)
    args = parser.parse_args()
    env, relevant_env = environment()
    initial_revision = capture_revision()
    binary = WORKTREE / ".build/target/debug/examples/replay"
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    metadata = {
        "candidate": CANDIDATE,
        "worktree": str(WORKTREE),
        "artifact_directory": str(output),
        "environment": relevant_env,
        "initial_revision": initial_revision,
        "local_wall_clock": "time.perf_counter_ns, includes Cargo and lock acquisition",
        "real_network_measurements": None,
        "binary": str(binary),
        "tools": {
            tool: subprocess.check_output(command, cwd=WORKTREE, env=env, text=True).strip()
            for tool, command in {
                "rustc": ["rustc", "-Vv"],
                "cargo": ["cargo", "-V"],
                "python": ["python3", "-V"],
                "git": ["git", "--version"],
                "flock": ["flock", "--version"],
            }.items()
        },
        "source_sha256_before": source_hashes(),
        "requested_cases": [f"{scenario}-{seed}-{pause}" for scenario, seed, pause in CASES],
        "complete_cases": [],
    }
    write_json(output / "capture.json", metadata)
    try:
        for case in CASES:
            result = run_case(output, env, binary, *case)
            if result["timed_out"] or result["exit_code"] != 0:
                metadata["failed_or_incomplete_case"] = result["case"]
                return 1
            metadata["complete_cases"].append(result["case"])
    finally:
        metadata["final_revision"] = capture_revision()
        metadata["source_sha256_after"] = source_hashes()
        write_json(output / "capture.json", metadata)
    if metadata["source_sha256_before"] != metadata["source_sha256_after"]:
        raise RuntimeError("tracked inputs changed while running the matrix")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
