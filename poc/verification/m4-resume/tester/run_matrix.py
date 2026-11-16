#!/usr/bin/env python3
"""Run the reviewed M4 acceptance matrix and seal invocation provenance."""

import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

WORKTREE = Path.cwd().resolve()
POC = WORKTREE / "poc"
HOST = Path("/mnt/reth/jy/megaroation_poc")
JOB = HOST / ".build/commander-jobs/m4-resume"
VERIFY = POC / "verification/m4-resume"
M4_RUN = POC / "scenarios/m4-resume/run.py"
PUBLIC = POC / "verification/m2-healthy-force/public"
REPLAY = VERIFY / "replay-vectors"
INPUTS = VERIFY / "input-vectors"
INPUT_CASES = {"legacy_rollback", "same_block", "before", "boundary", "after", "substitute", "future", "malformed"}
ARCHIVED_11 = HOST / ".build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run"
COMPONENTS = (
    "b404818e18be62663e84a7fa580b151c686ad951",
    "aab8a5180801ed7f97f7e437d147c97f972b4dd8",
    "1f6d1b3b4076ff16908dc59f272a06686b1d1ea5",
)


def load_matrix_helpers():
    path = POC / "verification/m3-auto-force/tester/run_matrix.py"
    spec = importlib.util.spec_from_file_location("m3_tester_helpers", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_m1(helpers, root, candidate):
    path = root / "manifest.json"
    digest = helpers.sha(path)
    if path.with_suffix(".sha256").read_text().split() != [digest, path.name]:
        raise ValueError("M1 manifest seal differs")
    report = json.loads(path.read_bytes())
    before = report["revision_before"]
    if (report["status"] != "COMPLETED" or report["candidate"] != candidate or report["seed"] != 42
            or before["head"] != candidate or before["status"] or before != report["revision_after"]):
        raise ValueError("M1 status/candidate/seed mismatch")
    for row in report["artifacts"]:
        item = (root / row["path"]).resolve()
        if not item.is_relative_to(root) or item.stat().st_size != row["bytes"]:
            raise ValueError(f"invalid M1 evidence path or size: {row['path']}")
        if helpers.sha(item) != row["sha256"]:
            raise ValueError(f"M1 evidence digest differs: {row['path']}")
    return {"path": "m1-regression/manifest.json", "sha256": digest}


def compare_pair(helpers, output):
    primary = output / "primary/raw"
    pause = output / "primary-pause650/raw"
    required = {
        "trace.virtual.jsonl", "frames.bin", "l1/raw-rpc-bundle.json", "l1/driver-trace.json",
        "derivation/batch-h1.json", "derivation/batch-h2.json",
        "derivation/cut-A-9-0-1.json", "derivation/cut-B-9-0-1.json",
        "derivation/trace.jsonl", "derivation/cursor-reads.jsonl",
    }

    def inventory(raw):
        objects = {
            str(path.relative_to(raw))
            for path in (raw / "derivation").glob("da/**/*") if path.is_file()
        }
        accepted = {
            str(path.relative_to(raw))
            for path in (raw / "derivation").glob("accepted-*.json") if path.is_file()
        }
        return objects, accepted

    a_da, a_accepted = inventory(primary)
    b_da, b_accepted = inventory(pause)
    result = {}
    for name in sorted(required | a_da | a_accepted | b_da | b_accepted):
        a = helpers.sha(primary / name) if (primary / name).is_file() else None
        b = helpers.sha(pause / name) if (pause / name).is_file() else None
        result[name] = {"primary_sha256": a, "repeat_sha256": b, "equal": a is not None and a == b}
    helpers.write(output / "repeat-compare.json", result)
    return bool(a_da and a_accepted) and all(row["equal"] for row in result.values())


def check_input_vectors(helpers, root, candidate):
    path = root / "manifest.json"
    result = json.loads(path.read_bytes())
    if (result["status"] != "PASS" or result["candidate"] != candidate
            or set(result["checks"]) != INPUT_CASES):
        raise ValueError("input vector status, candidate or required cases mismatch")
    expected = result["artifact_sha256"]
    actual = {
        str(p.relative_to(root)) for p in root.rglob("*") if p.is_file() and p != path
    }
    if set(expected) != actual:
        raise ValueError("input vector artifact inventory differs")
    for name, digest in expected.items():
        item = (root / name).resolve()
        if not item.is_relative_to(root) or helpers.sha(item) != digest:
            raise ValueError(f"input vector artifact digest differs: {name}")
    return {
        "path": "input-vectors/manifest.json", "sha256": helpers.sha(path),
        "checks": sorted(result["checks"]),
    }


def execute(helpers, args, output, report):
    rust, foundry, python = helpers.RUST, helpers.FOUNDRY, helpers.PYTHON
    settings = {
        "PATH": f"{rust}{os.pathsep}{os.defpath}",
        "RUSTC": str(rust / "rustc"), "RUSTDOC": str(rust / "rustdoc"),
        "REGIONAL_BFT_RUST_BIN": str(rust),
        "M2_FOUNDRY_BIN": str(foundry), "M3_FOUNDRY_BIN": str(foundry),
        "M4_FOUNDRY_BIN": str(foundry),
        "M2_SCENARIO_PYTHON": str(python), "M3_SCENARIO_PYTHON": str(python),
        "M4_SCENARIO_PYTHON": str(python),
        "CARGO_HOME": str(JOB / "cargo"), "CARGO_TARGET_DIR": str(JOB / "target"),
        "CARGO_BUILD_JOBS": "2", "CARGO_NET_OFFLINE": "true",
        "CARGO_TERM_COLOR": "never", "PYTHONDONTWRITEBYTECODE": "1", "LC_ALL": "C",
    }
    env = {**os.environ, **settings}
    report["environment"] = settings
    report["before"] = helpers.revision(args.candidate)
    for commit in COMPONENTS:
        subprocess.run(["git", "merge-base", "--is-ancestor", commit, args.candidate], check=True)
    report["ancestors"] = list(COMPONENTS)
    flock = shutil.which("flock", path=settings["PATH"])
    timeout = shutil.which("timeout", path=settings["PATH"])
    if flock is None or timeout is None:
        raise RuntimeError("host flock and timeout required")
    tools = [
        python, rust / "cargo", rust / "rustc", rust / "rustdoc", foundry / "anvil",
        foundry / "forge", foundry / "solc", Path(flock), Path(timeout),
    ]
    report["tools"] = {
        str(tool): {
            "version": subprocess.check_output([tool, "--version"], env=env, text=True).strip(),
            "sha256": helpers.sha(tool),
        }
        for tool in tools
    }
    lockfiles = (
        "prototypes/i1b-runtime/Cargo.lock", "crates/l1-adapter/Cargo.lock",
        "crates/batch-services/Cargo.lock", "crates/derivation/Cargo.lock",
        "verification/m2-healthy-force/public/Cargo.lock",
    )
    inputs = (
        POC / "docs/milestones/M4-d-resume-task-contract-zh.md",
        VERIFY / "source-expectations.md", M4_RUN, *[POC / name for name in lockfiles],
        REPLAY / "Cargo.lock", INPUTS / "Cargo.lock", INPUTS / "run.py",
        VERIFY / "tester/regression.py", POC / "verification/m3-auto-force/tester/run_matrix.py",
    )
    report["inputs"] = {
        str(path.relative_to(WORKTREE)): helpers.sha(path) for path in inputs
    }

    def capture(label, argv):
        return helpers.capture(output, report, env, label, argv) == 0

    def locked(label, argv):
        return capture(label, helpers.focused(argv, flock, timeout))

    def live(label, argv, pause=0, scenario=None):
        if not capture(label, argv):
            return False
        try:
            report["bundles"][label] = helpers.check_run(
                output, args.candidate, label, scenario, pause
            )
        except (OSError, ValueError, KeyError) as error:
            report.setdefault("evidence_failures", {})[label] = str(error)
            return False
        return True

    okay = True
    candidate = ["--candidate", args.candidate]
    for label, pause in (("primary", 0), ("primary-pause650", 650)):
        ran = live(label, [
            python, "-B", M4_RUN, *candidate, "--seed", "42",
            "--wall-pause-ms", str(pause), "--output", output / label,
        ], pause=pause, scenario="preset-resume")
        okay = ran and okay
    if {"primary", "primary-pause650"} <= report["bundles"].keys():
        report["repeat_equal"] = compare_pair(helpers, output)
        okay = report["repeat_equal"] and okay

    built = locked("independent-public-build", [
        rust / "cargo", "build", "--locked", "--offline", "--manifest-path", PUBLIC / "Cargo.toml",
    ])
    if built:
        binary = JOB / "target/debug/m2-force-independent"
        report["public_checker"] = {"path": str(binary), "sha256": helpers.sha(binary)}
        print(f"public checker {binary} {report['public_checker']['sha256']}", flush=True)
    okay = built and okay

    label = "m1-regression"
    ran = capture(label, [
        python, "-B", POC / "verification/m2-healthy-force/tester/m1_regression.py",
        "--worktree", WORKTREE, "--build-root", JOB, *candidate, "--output", output / label,
    ])
    if ran:
        try:
            report["bundles"][label] = check_m1(helpers, output / label, args.candidate)
        except (OSError, ValueError, KeyError) as error:
            report.setdefault("evidence_failures", {})[label] = str(error)
            ran = False
    okay = ran and okay
    for milestone, scenario in (("m2", None), ("m3", "outage")):
        label = f"{milestone}-regression"
        okay = live(label, [
            python, "-B", VERIFY / "tester/regression.py", milestone,
            *candidate, "--output", output / label,
        ], scenario=scenario) and okay

    cargo = rust / "cargo"
    passed = locked("replay-vectors", [
        cargo, "test", "--locked", "--offline", "--manifest-path", REPLAY / "Cargo.toml",
        "--", "--nocapture", "--test-threads=1",
    ])
    counts = re.findall(
        r"test result: ok\. (\d+) passed", (output / "replay-vectors.stdout").read_text()
    )
    count = sum(map(int, counts))
    report["replay_vectors"] = {"expected": 10, "passed": count}
    okay = passed and count == 10 and okay

    driver = JOB / "target/debug/l1-adapter"
    driver_built = locked("input-l1-adapter-build", [
        cargo, "build", "--locked", "--offline",
        "--manifest-path", POC / "crates/l1-adapter/Cargo.toml",
    ])
    vector_port = JOB / "target/debug/m4-input-vectors"
    vector_built = locked("input-vectors-build", [
        cargo, "build", "--locked", "--offline", "--manifest-path", INPUTS / "Cargo.toml",
    ])
    contracts = next(
        (output / name / "forge-out" for name in (
            "primary", "primary-pause650", "m2-regression", "m3-regression"
        ) if name in report["bundles"]), None
    )
    if not driver_built or not vector_built or contracts is None:
        report["input_vectors"] = {"status": "NOT_RUN", "reason": "candidate build prerequisites missing"}
        okay = False
    else:
        report["input_vector_binaries"] = {
            str(binary): helpers.sha(binary) for binary in (driver, vector_port)
        }
        ran = locked("input-vectors", [
            python, "-B", INPUTS / "run.py", *candidate, "--anvil", foundry / "anvil",
            "--driver", driver, "--vector-port", vector_port, "--contracts", contracts,
            "--archive", ARCHIVED_11, "--out", output / "input-vectors",
        ])
        if ran:
            try:
                report["input_vectors"] = check_input_vectors(
                    helpers, output / "input-vectors", args.candidate
                )
            except (OSError, ValueError, KeyError) as error:
                report.setdefault("evidence_failures", {})["input-vectors"] = str(error)
                ran = False
        okay = ran and okay
    report["after"] = helpers.revision(args.candidate)
    return okay


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True, help="reviewed full Git SHA")
    parser.add_argument("--output", type=Path, required=True, help="fresh M4 artifact directory")
    args = parser.parse_args()
    output = args.output.resolve()
    artifacts = JOB / "artifacts"
    if output == artifacts or not output.is_relative_to(artifacts):
        parser.error("output must be a fresh directory within the M4 artifact root")
    helpers = load_matrix_helpers()
    helpers.revision(args.candidate)
    output.mkdir(mode=0o700)
    shutil.copyfile(Path(__file__), output / "run_matrix.py")
    shutil.copyfile(VERIFY / "tester/regression.py", output / "regression.py")
    report = {
        "candidate": args.candidate, "seed": 42, "status": "FAIL", "argv": sys.orig_argv,
        "heavy_lock": helpers.LOCK, "commands": [], "bundles": {},
        "runner_sha256": helpers.sha(output / "run_matrix.py"),
        "regression_runner_sha256": helpers.sha(output / "regression.py"),
    }
    try:
        okay = execute(helpers, args, output, report)
        report["status"] = "PASS" if okay else "FAIL"
    except BaseException as error:
        report["failure"] = str(error)
        raise
    finally:
        manifest = output / "manifest.json"
        helpers.write(manifest, report)
        manifest.with_suffix(".sha256").write_text(
            f"{helpers.sha(manifest)}  manifest.json\n", encoding="utf-8",
        )
    return 0 if okay else 1


if __name__ == "__main__":
    raise SystemExit(main())
