#!/usr/bin/env python3
"""Run the fixed M2 Healthy Force profile and seal its raw evidence."""

import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
HOST_ROOT = Path("/mnt/reth/jy/megaroation_poc")
JOB = HOST_ROOT / ".build/commander-jobs/m2-healthy-force"
RUST = Path("/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/"
            "1.95.0-x86_64-unknown-linux-gnu/bin")
FOUNDRY = HOST_ROOT / ".build/tools/foundry-1.0.0"
PYTHON = Path("/home/jy/miniconda3/bin/python3")
HEAVY_LOCK = Path("/tmp/megaroation-poc-heavy.lock")
CONTRACT = ROOT / "poc/docs/milestones/M2-healthy-force-task-contract-zh.md"
EXPECTATIONS = ROOT / "poc/verification/m2-healthy-force/source-expectations-v1.md"
RUNTIME = ROOT / "poc/prototypes/i1b-runtime"
L1 = ROOT / "poc/crates/l1-adapter"
LIMIT_S = 1800


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write(path, data):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(data, stream, sort_keys=True, indent=2)
        stream.write("\n")


def git(*args):
    return subprocess.check_output(["git", "-C", ROOT, *args], text=True).strip()


def revision(candidate):
    head = git("rev-parse", "HEAD")
    status = git("status", "--porcelain", "--untracked-files=all")
    if head != candidate or status:
        raise ValueError(f"requires clean candidate {candidate}, found {head} {status!r}")
    return {
        "head": head, "branch": git("branch", "--show-current"), "status": status,
        "bft_gitlink": git("rev-parse", f"{head}:regional_BFT_sim"),
    }


def command(run, manifest, cwd, env, *argv):
    argv = list(map(str, argv))
    number = len(manifest["commands"])
    out = run / f"command-{number:02}.stdout"
    err = run / f"command-{number:02}.stderr"
    row = {"argv": argv, "cwd": str(cwd), "stdout": out.name, "stderr": err.name}
    manifest["commands"].append(row)
    start = time.monotonic()
    try:
        with out.open("xb") as stdout, err.open("xb") as stderr:
            with subprocess.Popen(argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                  stdout=stdout, stderr=stderr, start_new_session=True) as child:
                try:
                    row["exit"] = child.wait(timeout=LIMIT_S)
                except BaseException:
                    if child.poll() is None:
                        os.killpg(child.pid, signal.SIGKILL)
                        child.wait()
                    row["exit"] = child.returncode
                    raise
    finally:
        row["wall_elapsed_s"] = time.monotonic() - start
    if row["exit"] != 0:
        raise RuntimeError(f"command failed: {argv!r}; see {err}")


def normalize(raw):
    with (raw / "trace.jsonl").open(encoding="utf-8") as source:
        with (raw / "trace.virtual.jsonl").open("x", encoding="utf-8") as target:
            for line in source:
                row = json.loads(line)
                if row["kind"] in {"WorkerSpawn", "WorkerExit"}:
                    for key in ("os_pid", "wall_time_ns", "proc_stat"):
                        row["detail"].pop(key, None)
                target.write(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n")


def execute(args, out, manifest):
    manifest["before"] = revision(args.candidate)
    manifest["inputs"] = {
        str(path.relative_to(ROOT)): sha(path)
        for path in (CONTRACT, EXPECTATIONS, RUNTIME / "Cargo.lock", L1 / "Cargo.lock")
    }
    rust = Path(os.environ.get("REGIONAL_BFT_RUST_BIN", RUST)).resolve(strict=True)
    foundry = Path(os.environ.get("M2_FOUNDRY_BIN", FOUNDRY)).resolve(strict=True)
    python = Path(os.environ.get("M2_SCENARIO_PYTHON", PYTHON)).resolve(strict=True)
    env = dict(os.environ)
    env.update(
        PATH=str(rust) + os.pathsep + os.defpath, LC_ALL="C",
        CARGO_HOME=str(JOB / "cargo"), CARGO_TARGET_DIR=str(JOB / "target"),
        CARGO_BUILD_JOBS="2", CARGO_NET_OFFLINE="true", CARGO_TERM_COLOR="never",
        RUSTC=str(rust / "rustc"), RUSTDOC=str(rust / "rustdoc"),
        M1_CANDIDATE=args.candidate,
        M2_PYTHON=str(python), M2_L1_PORT=str(HERE / "l1.py"),
        M2_ANVIL=str(foundry / "anvil"), M2_DRIVER=str(JOB / "target/debug/l1-adapter"),
        M2_CONTRACTS=str(out / "forge-out"), PYTHONDONTWRITEBYTECODE="1",
    )
    names = ("CARGO_HOME", "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "CARGO_NET_OFFLINE",
             "RUSTC", "RUSTDOC", "M1_CANDIDATE", "M2_PYTHON", "M2_L1_PORT",
             "M2_ANVIL", "M2_DRIVER", "M2_CONTRACTS")
    manifest["environment"] = {key: env[key] for key in names}
    tools = [rust / "rustc", rust / "cargo", foundry / "forge", foundry / "solc",
             foundry / "anvil", python]
    manifest["tools"] = {}
    for tool in tools:
        result = subprocess.check_output([tool, "--version"], env=env, text=True).strip()
        manifest["tools"][str(tool)] = {"sha256": sha(tool), "version": result}
    if not manifest["tools"][str(rust / "rustc")]["version"].startswith("rustc 1.95.0 "):
        raise ValueError("M2 requires Rust 1.95.0")
    manifest["coincurve_version"] = subprocess.check_output(
        [python, "-B", "-c", "from importlib.metadata import version; print(version('coincurve'))"],
        env=env, text=True,
    ).strip()
    with HEAVY_LOCK.open("a") as lease:
        deadline = time.monotonic() + LIMIT_S
        while True:
            try:
                fcntl.flock(lease, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.monotonic() >= deadline:
                    raise TimeoutError(f"host build lock busy after {LIMIT_S}s")
                time.sleep(1)
        command(out, manifest, ROOT, env, rust / "cargo", "build", "--locked", "--offline",
                "--manifest-path", RUNTIME / "Cargo.toml", "--bin", "m1-smoke")
        command(out, manifest, ROOT, env, rust / "cargo", "build", "--locked", "--offline",
                "--manifest-path", L1 / "Cargo.toml")
        command(out, manifest, ROOT, env, foundry / "forge", "build", "--root", ROOT / "poc/contracts",
                "--use", foundry / "solc", "--offline", "--threads", 2, "--out", out / "forge-out",
                "--cache-path", out / "forge-cache")
        command(out, manifest, ROOT, env, python, "-B", L1 / "tests/check_abi.py", out / "forge-out")
        smoke = JOB / "target/debug/m1-smoke"
        manifest["binaries"] = {str(path): sha(path) for path in (smoke, Path(env["M2_DRIVER"]))}
        command(out, manifest, ROOT, env, smoke, "--healthy-force", "--seed", args.seed,
                "--output", out / "raw", "--wall-pause-ms", args.wall_pause_ms)
    normalize(out / "raw")
    manifest["after"] = revision(args.candidate)
    if manifest["after"] != manifest["before"]:
        raise ValueError("candidate changed during the run")
    manifest["status"] = "COMPLETED"
    manifest["summary"] = json.loads((out / "raw/summary.json").read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--seed", default=42, type=int, choices=[42])
    parser.add_argument("--wall-pause-ms", default=0, type=int)
    args = parser.parse_args()
    if not 0 <= args.wall_pause_ms <= 60000:
        parser.error("wall pause must be between zero and 60000 ms")
    out = args.output.resolve()
    out.mkdir(mode=0o700)
    manifest = {
        "schema": "M2-HEALTHY-FORCE-RUN-1", "candidate": args.candidate, "seed": args.seed,
        "wall_pause_ms": args.wall_pause_ms, "status": "FAILED", "commands": [],
        "argv": sys.orig_argv, "heavy_lock": str(HEAVY_LOCK), "limits": {"command_wall_s": LIMIT_S},
        "commonware": "2026.10.0", "worktree": str(ROOT),
    }
    try:
        execute(args, out, manifest)
    except BaseException as error:
        manifest["failure"] = str(error)
        raise
    finally:
        manifest["files"] = {
            str(path.relative_to(out)): {"sha256": sha(path), "bytes": path.stat().st_size}
            for path in sorted(out.rglob("*")) if path.is_file()
        }
        write(out / "manifest.json", manifest)
        (out / "manifest.sha256").write_text(f"{sha(out / 'manifest.json')}  manifest.json\n")
    print(out / "manifest.json")


if __name__ == "__main__":
    main()
