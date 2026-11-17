#!/usr/bin/env python3
"""Run the M5 virtual-time ordering and automatic RESUME profiles and seal their evidence."""

import argparse
import fcntl
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
spec = importlib.util.spec_from_file_location("m2_launcher", HERE.parent / "m2-healthy-force/run.py")
baseline = importlib.util.module_from_spec(spec)
spec.loader.exec_module(baseline)
JOB = baseline.HOST_ROOT / ".build/commander-jobs/m4-resume"
CONTRACT = ROOT / "poc/docs/milestones/M5-auto-resume-task-contract-zh.md"
EXPECTATIONS = ROOT / "poc/verification/m5-auto-resume/source-expectations.md"


def normalize(raw):
    with (raw / "trace.jsonl").open(encoding="utf-8") as stream:
        with (raw / "trace.virtual.jsonl").open("x", encoding="utf-8") as out:
            for line in stream:
                row = json.loads(line)
                if row["kind"] in {"WorkerSpawn", "WorkerExit", "WorkerOffline"}:
                    for key in ("os_pid", "wall_time_ns", "proc_stat"):
                        row["detail"].pop(key, None)
                out.write(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n")


def execute(args, output, manifest):
    manifest["before"] = baseline.revision(args.candidate)
    manifest["inputs"] = {
        str(path.relative_to(ROOT)): baseline.sha(path)
        for path in (CONTRACT, EXPECTATIONS, baseline.RUNTIME / "Cargo.lock", baseline.L1 / "Cargo.lock", ROOT/"poc/crates/derivation/Cargo.lock", ROOT/"poc/crates/batch-services/Cargo.lock")
    }
    rust = Path(os.environ.get("REGIONAL_BFT_RUST_BIN", baseline.RUST)).resolve(strict=True)
    foundry = Path(os.environ.get("M5_FOUNDRY_BIN", baseline.FOUNDRY)).resolve(strict=True)
    python = Path(os.environ.get("M5_SCENARIO_PYTHON", baseline.PYTHON)).resolve(strict=True)
    extra = dict(
        PATH=str(rust) + os.pathsep + os.defpath, LC_ALL="C",
        CARGO_HOME=str(JOB / "cargo"), CARGO_TARGET_DIR=str(JOB / "target"),
        CARGO_BUILD_JOBS="2", CARGO_NET_OFFLINE="true", CARGO_TERM_COLOR="never",
        RUSTC=str(rust / "rustc"), RUSTDOC=str(rust / "rustdoc"), M1_CANDIDATE=args.candidate,
        M2_PYTHON=str(python), M2_L1_PORT=str(HERE / "l1.py"),
        M2_ANVIL=str(foundry / "anvil"), M2_DRIVER=str(JOB / "target/debug/m4-driver"),
        M2_CONTRACTS=str(output / "forge-out"), PYTHONDONTWRITEBYTECODE="1",
        M5_SCENARIO=args.scenario,
    )
    env = {**os.environ, **extra}
    manifest["environment"] = extra
    tools = [rust / "rustc", rust / "cargo", foundry / "forge", foundry / "solc",
             foundry / "anvil", python]
    manifest["tools"] = {
        str(tool): {
            "sha256": baseline.sha(tool),
            "version": subprocess.check_output([tool, "--version"], env=env, text=True).strip(),
        }
        for tool in tools
    }
    if not manifest["tools"][str(rust / "rustc")]["version"].startswith("rustc 1.95.0 "):
        raise ValueError("M5 requires Rust 1.95.0")
    manifest["coincurve_version"] = subprocess.check_output(
        [python, "-B", "-c", "from importlib.metadata import version; print(version('coincurve'))"],
        env=env, text=True,
    ).strip()
    with baseline.HEAVY_LOCK.open("a") as lease:
        deadline = time.monotonic() + baseline.LIMIT_S
        while True:
            try:
                fcntl.flock(lease, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.monotonic() >= deadline:
                    raise TimeoutError(f"host build lock busy after {baseline.LIMIT_S}s")
                time.sleep(1)

        def command(*argv):
            baseline.command(output, manifest, ROOT, env, *argv)

        command(rust / "cargo", "build", "--locked", "--offline", "--manifest-path",
                baseline.RUNTIME / "Cargo.toml", "--bin", "m1-smoke", "--bin", "m4-driver")
        command(foundry / "forge", "build", "--root", ROOT / "poc/contracts", "--use",
                foundry / "solc", "--offline", "--threads", 2, "--out", output / "forge-out",
                "--cache-path", output / "forge-cache")
        command(python, "-B", baseline.L1 / "tests/check_abi.py", output / "forge-out")
        smoke = JOB / "target/debug/m1-smoke"
        manifest["binaries"] = {str(path): baseline.sha(path) for path in (smoke, Path(env["M2_DRIVER"]))}
        command(smoke, "--auto-resume", args.scenario, "--seed", args.seed, "--output", output / "raw",
                "--wall-pause-ms", args.wall_pause_ms)
    normalize(output / "raw")
    manifest["after"] = baseline.revision(args.candidate)
    if manifest["before"] != manifest["after"]:
        raise ValueError("candidate changed during run")
    manifest["status"] = "COMPLETED"
    manifest["summary"] = json.loads((output / "raw/summary.json").read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--seed", type=int, choices=[42], default=42)
    parser.add_argument("--scenario", choices=("stopped", "frozen", "slow-rpc", "late",
                                               "short", "no-target", "inputs-wait", "divergence"),
                        default="stopped")
    parser.add_argument("--wall-pause-ms", type=int, default=0)
    args = parser.parse_args()
    if not 0 <= args.wall_pause_ms <= 60_000:
        parser.error("wall pause must be between zero and 60000 ms")
    output = args.output.resolve()
    output.mkdir(mode=0o700)
    manifest = {
        "schema": "M5-AUTO-RESUME-RUN-1", "candidate": args.candidate, "seed": args.seed,
        "scenario": args.scenario, "wall_pause_ms": args.wall_pause_ms,
        "status": "FAILED", "commands": [], "argv": sys.orig_argv,
        "heavy_lock": str(baseline.HEAVY_LOCK), "limits": {"command_wall_s": baseline.LIMIT_S},
        "worktree": str(ROOT), "commonware": "2026.10.0",
    }
    try:
        execute(args, output, manifest)
    except BaseException as error:
        manifest["failure"] = str(error)
        raise
    finally:
        manifest["files"] = {
            str(path.relative_to(output)): {"sha256": baseline.sha(path), "bytes": path.stat().st_size}
            for path in sorted(output.rglob("*")) if path.is_file()
        }
        baseline.write(output / "manifest.json", manifest)
        (output / "manifest.sha256").write_text(f"{baseline.sha(output / 'manifest.json')}  manifest.json\n")
    print(output / "manifest.json")


if __name__ == "__main__":
    main()
