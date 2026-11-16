#!/usr/bin/env python3
"""Run the reviewed M3 Tester matrix and seal invocation provenance."""

import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import time

WORKTREE = Path.cwd().resolve()
HOST = Path("/mnt/reth/jy/megaroation_poc")
JOB = HOST / ".build/commander-jobs/m3-auto-force"
RUST = Path("/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/"
            "1.95.0-x86_64-unknown-linux-gnu/bin")
FOUNDRY = HOST / ".build/tools/foundry-1.0.0"
PYTHON = Path("/home/jy/miniconda3/bin/python3")
LOCK = "/tmp/megaroation-poc-heavy.lock"
FOCUSED_LOCK_WAIT_S = "900"
FOCUSED_WALL_LIMIT_S = "1800"
M3_RUN = WORKTREE / "poc/scenarios/m3-auto-force/run.py"
VERIFY = WORKTREE / "poc/verification/m3-auto-force"
RUNTIME = WORKTREE / "poc/prototypes/i1b-runtime"
PUBLIC = WORKTREE / "poc/verification/m2-healthy-force/public"
FOCUSED_L1 = WORKTREE / "poc/crates/l1-adapter"
LIVE = (
    ("outage", "outage", 0),
    ("outage-pause650", "outage", 650),
    ("healthy", "healthy", 0),
    ("reader-lag", "reader-lag", 0),
    ("ordering-stopped", "ordering-stopped", 0),
    ("watermark-frozen", "watermark-frozen", 0),
)


def sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, sort_keys=True, indent=2)
        stream.write("\n")


def git(*args):
    return subprocess.check_output(
        ["git", "-C", WORKTREE, *args], text=True
    ).strip()


def revision(candidate):
    root = git("rev-parse", "--show-toplevel")
    head = git("rev-parse", "HEAD")
    status = git("status", "--porcelain=v1", "--untracked-files=all",
                 "--ignore-submodules=none")
    if Path(root) != WORKTREE or head != candidate or status:
        raise ValueError(f"requires exact clean candidate {candidate}: {(root, head, status)!r}")
    return {
        "worktree": root, "head": head, "status": status,
        "branch": git("branch", "--show-current"),
        "bft_gitlink": git("rev-parse", f"{head}:regional_BFT_sim"),
    }


def capture(out, report, env, label, argv):
    revision(report["candidate"])
    row = {"name": label, "argv": list(map(str, argv)), "cwd": str(WORKTREE)}
    report["commands"].append(row)
    print(f"start {label}", flush=True)
    stdout = out / f"{label}.stdout"
    stderr = out / f"{label}.stderr"
    before = time.monotonic()
    try:
        with stdout.open("xb") as std_out, stderr.open("xb") as std_err:
            result = subprocess.run(
                row["argv"], cwd=WORKTREE, env=env, stdin=subprocess.DEVNULL,
                stdout=std_out, stderr=std_err, check=False,
            )
            row["exit_code"] = result.returncode
    finally:
        row["wall_elapsed_s"] = time.monotonic() - before
        for name, path in (("stdout", stdout), ("stderr", stderr)):
            if path.exists():
                row[name] = {"path": path.name, "sha256": sha(path), "bytes": path.stat().st_size}
    revision(report["candidate"])
    print(f"finish {label} exit={row['exit_code']} wall={row['wall_elapsed_s']:.3f}s", flush=True)
    return row["exit_code"]


def check_run(root, candidate, name, scenario=None, pause=0):
    path = root / name / "manifest.json"
    digest = sha(path)
    sidecar = root / name / "manifest.sha256"
    if sidecar.read_text(encoding="utf-8").split() != [digest, "manifest.json"]:
        raise ValueError(f"manifest sidecar differs for {name}")
    child = json.loads(path.read_bytes())
    if (child["candidate"] != candidate or child["status"] != "COMPLETED"
            or child["seed"] != 42 or child["wall_pause_ms"] != pause
            or child["before"] != child["after"] or child["before"]["status"]):
        raise ValueError(f"unexpected completed manifest for {name}")
    if scenario is not None and child["scenario"] != scenario:
        raise ValueError(f"unexpected scenario manifest for {name}")
    for relative, descriptor in child["files"].items():
        artifact = root / name / relative
        if (not artifact.resolve().is_relative_to(root / name)
                or artifact.stat().st_size != descriptor["bytes"]
                or sha(artifact) != descriptor["sha256"]):
            raise ValueError(f"sealed file changed in {name}: {relative}")
    return {"path": str(path.relative_to(root)), "sha256": digest, "summary": child["summary"]}


def compare_repeat(out):
    values = {}
    for relative in (
        "trace.virtual.jsonl", "frames.bin", "l1/raw-rpc-bundle.json", "l1/driver-trace.json"
    ):
        base = sha(out / "outage/raw" / relative)
        repeated = sha(out / "outage-pause650/raw" / relative)
        values[relative] = {
            "primary_sha256": base, "repeat_sha256": repeated, "equal": base == repeated,
        }
    write(out / "repeat-compare.json", values)
    required = ("trace.virtual.jsonl", "frames.bin", "l1/raw-rpc-bundle.json")
    return all(values[name]["equal"] for name in required)


def focused(command, flock, timeout):
    return [
        flock, "-w", FOCUSED_LOCK_WAIT_S, "-E", "75", LOCK,
        timeout, "--kill-after=10s", FOCUSED_WALL_LIMIT_S, *command,
    ]


def run(args, out, report):
    env = {**os.environ, **{
        "PATH": f"{RUST}{os.pathsep}{os.defpath}",
        "RUSTC": str(RUST / "rustc"), "RUSTDOC": str(RUST / "rustdoc"),
        "REGIONAL_BFT_RUST_BIN": str(RUST),
        "M2_FOUNDRY_BIN": str(FOUNDRY), "M3_FOUNDRY_BIN": str(FOUNDRY),
        "M2_SCENARIO_PYTHON": str(PYTHON), "M3_SCENARIO_PYTHON": str(PYTHON),
        "CARGO_HOME": str(JOB / "cargo"), "CARGO_TARGET_DIR": str(JOB / "target"),
        "CARGO_BUILD_JOBS": "2", "CARGO_NET_OFFLINE": "true",
        "CARGO_TERM_COLOR": "never", "PYTHONDONTWRITEBYTECODE": "1", "LC_ALL": "C",
    }}
    environment_keys = (
        "PATH", "RUSTC", "RUSTDOC", "REGIONAL_BFT_RUST_BIN", "M2_FOUNDRY_BIN",
        "M3_FOUNDRY_BIN", "M2_SCENARIO_PYTHON", "M3_SCENARIO_PYTHON", "CARGO_HOME",
        "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "CARGO_NET_OFFLINE",
        "CARGO_TERM_COLOR", "PYTHONDONTWRITEBYTECODE", "LC_ALL",
    )
    report["environment"] = {key: env[key] for key in environment_keys}
    report["before"] = revision(args.candidate)
    flock = shutil.which("flock", path=env["PATH"])
    timeout = shutil.which("timeout", path=env["PATH"])
    if flock is None or timeout is None:
        raise RuntimeError("host locking and focused watchdog tools are required")
    tools = [PYTHON, RUST / "cargo", RUST / "rustc", RUST / "rustdoc", FOUNDRY / "anvil",
             FOUNDRY / "forge", FOUNDRY / "solc", Path(flock), Path(timeout)]
    report["tools"] = {
        str(tool): {
            "sha256": sha(tool),
            "version": subprocess.check_output([tool, "--version"], env=env, text=True).strip(),
        }
        for tool in tools
    }
    report["inputs"] = {
        str(path.relative_to(WORKTREE)): sha(path)
        for path in (
            WORKTREE / "poc/docs/milestones/M3-auto-force-task-contract-zh.md",
            VERIFY / "source-expectations.md", M3_RUN, RUNTIME / "Cargo.lock",
            PUBLIC / "Cargo.lock", FOCUSED_L1 / "Cargo.lock",
            VERIFY / "plan-vectors/Cargo.lock", VERIFY / "l1-vectors/run.py",
        )
    }
    base = ["--candidate", args.candidate, "--seed", "42"]
    ok = True
    for name, scenario, pause in LIVE:
        code = capture(out, report, env, name, [
            PYTHON, "-B", M3_RUN, *base, "--scenario", scenario,
            "--wall-pause-ms", str(pause), "--output", out / name,
        ])
        if code == 0:
            report["bundles"][name] = check_run(out, args.candidate, name, scenario, pause)
        else:
            ok = False

    if {"outage", "outage-pause650"} <= report["bundles"].keys():
        report["repeat_equal"] = compare_repeat(out)
        ok = report["repeat_equal"] and ok

    code = capture(out, report, env, "m2-regression", [
        PYTHON, "-B", VERIFY / "tester/m2_regression.py", *base, "--wall-pause-ms", "0",
        "--output", out / "m2-regression",
    ])
    if code == 0:
        report["bundles"]["m2-regression"] = check_run(out, args.candidate, "m2-regression")
    else:
        ok = False

    cargo = RUST / "cargo"
    for name, command in (
        ("independent-public-build", [
            cargo, "build", "--locked", "--offline", "--manifest-path", PUBLIC / "Cargo.toml"
        ]),
        ("governance-vectors", [
            cargo, "test", "--locked", "--offline", "--manifest-path", RUNTIME / "Cargo.toml",
            "--lib", "m1_smoke::governance::tests", "--", "--nocapture", "--test-threads=1",
        ]),
        ("plan-vectors", [
            cargo, "test", "--locked", "--offline",
            "--manifest-path", VERIFY / "plan-vectors/Cargo.toml",
            "--", "--nocapture", "--test-threads=1",
        ]),
    ):
        code = capture(out, report, env, name, focused(command, flock, timeout))
        if name == "independent-public-build" and code == 0:
            binary = JOB / "target/debug/m2-force-independent"
            report["public_checker"] = {"path": str(binary), "sha256": sha(binary)}
            print(f"public checker {binary} {report['public_checker']['sha256']}", flush=True)
        elif code == 0:
            expected = 3 if name == "governance-vectors" else 12
            matches = re.findall(r"test result: ok\. (\d+) passed", (out / f"{name}.stdout").read_text())
            actual = sum(map(int, matches))
            report.setdefault("rust_tests", {})[name] = {"passed": actual, "expected": expected}
            ok = (actual == expected) and ok
        ok = (code == 0) and ok

    m3_build = next((name for name, _, _ in LIVE if name in report["bundles"]), None)
    if m3_build is None:
        report["l1_vectors"] = {"status": "NOT_RUN", "reason": "successful M3 Forge build unavailable"}
        ok = False
    else:
        contracts = out / m3_build / "forge-out"
        code = capture(out, report, env, "l1-vectors", focused([
            PYTHON, "-B", VERIFY / "l1-vectors/run.py", "--candidate", args.candidate,
            "--anvil", FOUNDRY / "anvil", "--driver", JOB / "target/debug/l1-adapter",
            "--contracts", contracts, "--out", out / "l1-vectors",
        ], flock, timeout))
        if code == 0:
            path = out / "l1-vectors/manifest.json"
            l1 = json.loads(path.read_bytes())
            digest = sha(path)
            sidecar = path.with_name("manifest.sha256").read_text().split()
            require = (
                l1["status"] == "PASS" and l1["candidate"] == args.candidate
                and l1["before"] == l1["after"] and sidecar == [digest, "manifest.json"]
            )
            receipts = sum(p["receipts"] for p in l1["profiles"])
            report["l1_vectors"] = {
                "path": str(path.relative_to(out)), "sha256": digest,
                "receipts": receipts, "expected": 37,
            }
            ok = (require and receipts == 37) and ok
        ok = (code == 0) and ok
    report["after"] = revision(args.candidate)
    return ok


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True, help="reviewed full Git SHA")
    parser.add_argument("--output", type=Path, required=True, help="new M3 evidence directory")
    args = parser.parse_args()
    out = args.output.resolve()
    if out == JOB / "artifacts" or not out.is_relative_to(JOB / "artifacts"):
        parser.error("output must be a new directory within the M3 evidence root")
    revision(args.candidate)
    out.mkdir(mode=0o700)
    shutil.copyfile(Path(__file__), out / "run_matrix.py")
    report = {
        "candidate": args.candidate, "status": "FAIL", "argv": sys.orig_argv,
        "runner_sha256": sha(out / "run_matrix.py"), "heavy_lock": LOCK, "seed": 42,
        "commands": [], "bundles": {},
    }
    try:
        ok = run(args, out, report)
        report["status"] = "PASS" if ok else "FAIL"
    except BaseException as error:
        report["failure"] = str(error)
        raise
    finally:
        manifest = out / "manifest.json"
        write(manifest, report)
        (out / "manifest.sha256").write_text(
            f"{sha(manifest)}  manifest.json\n", encoding="utf-8"
        )
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
