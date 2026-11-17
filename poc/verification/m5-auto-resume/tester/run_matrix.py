#!/usr/bin/env python3
"""Record the exact candidate and seal the serial M5 acceptance matrix."""

import argparse
import importlib.util
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

WORKTREE = Path.cwd().resolve()
POC = WORKTREE / "poc"
HOST = Path("/mnt/reth/jy/megaroation_poc")
JOB = HOST / ".build/commander-jobs/m5-auto-resume"
VERIFY = POC / "verification/m5-auto-resume"
M5_RUN = POC / "scenarios/m5-auto-resume/run.py"
RUNTIME = POC / "prototypes/i1b-runtime"
PUBLIC = POC / "verification/m2-healthy-force/public"
M4_REPLAY = POC / "verification/m4-resume/replay-vectors"
BASE = "e44b6e8e77aeb1aa3895c14109a57ff3894d7252"
BFT = "691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6"
LIVE = (
    ("primary", "stopped", 0),
    ("primary-pause650", "stopped", 650),
    ("watermark-frozen", "frozen", 0),
    ("late-inclusion", "late", 0),
    ("slow-rpc", "slow-rpc", 0),
    ("short-stall", "short", 0),
    ("divergence", "divergence", 0),
    ("no-target", "no-target", 0),
    ("inputs-wait", "inputs-wait", 0),
)
M5_FILTER = "m1_smoke::m5::tests"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def compare_repeat(helpers, out):
    a = out / "primary/raw"
    b = out / "primary-pause650/raw"
    files = {
        "trace.virtual.jsonl", "frames.bin", "l1/raw-rpc-bundle.json", "l1/driver-trace.json",
        "derivation/trace.jsonl", "derivation/cursor-reads.jsonl", "governance.jsonl",
        "l1/hub-l1.jsonl", "l1/governance-transaction.json",
    }
    groups = ("derivation/da/**/*", "derivation/batch-*.json",
              "derivation/accepted-*.json", "derivation/cut-*.json")
    for pattern in groups:
        base = {str(path.relative_to(a)) for path in a.glob(pattern) if path.is_file()}
        repeat = {str(path.relative_to(b)) for path in b.glob(pattern) if path.is_file()}
        if not base or not repeat:
            raise ValueError(f"required repeat evidence group is empty: {pattern}")
        files.update(base | repeat)
    result = {}
    for name in sorted(files):
        base = helpers.sha(a / name) if (a / name).is_file() else None
        repeat = helpers.sha(b / name) if (b / name).is_file() else None
        result[name] = {
            "primary_sha256": base, "repeat_sha256": repeat,
            "equal": base is not None and base == repeat,
        }
    helpers.write(out / "repeat-compare.json", result)
    return all(row["equal"] for row in result.values())


def execute(helpers, m4, args, out, report):
    rust, foundry, python = helpers.RUST, helpers.FOUNDRY, helpers.PYTHON
    settings = {
        "PATH": f"{rust}{os.pathsep}{os.defpath}",
        "RUSTC": str(rust / "rustc"), "RUSTDOC": str(rust / "rustdoc"),
        "REGIONAL_BFT_RUST_BIN": str(rust),
        "CARGO_HOME": str(JOB / "cargo"), "CARGO_TARGET_DIR": str(JOB / "target"),
        "CARGO_BUILD_JOBS": "2", "CARGO_NET_OFFLINE": "true", "CARGO_TERM_COLOR": "never",
        "PYTHONDONTWRITEBYTECODE": "1", "LC_ALL": "C",
    }
    for version in ("M2", "M3", "M4", "M5"):
        settings[f"{version}_FOUNDRY_BIN"] = str(foundry)
        settings[f"{version}_SCENARIO_PYTHON"] = str(python)
    env = {**os.environ, **settings}
    report["environment"] = settings
    flock = shutil.which("flock", path=settings["PATH"])
    timeout = shutil.which("timeout", path=settings["PATH"])
    if flock is None or timeout is None:
        raise RuntimeError("host flock and timeout required")
    report["tools"] = {
        str(tool): {
            "version": subprocess.check_output([tool, "--version"], env=env, text=True).strip(),
            "sha256": helpers.sha(tool),
        }
        for tool in (
            python, rust / "cargo", rust / "rustc", rust / "rustdoc", foundry / "anvil",
            foundry / "forge", foundry / "solc", Path(flock), Path(timeout),
        )
    }
    inputs = (
        POC / "docs/milestones/M5-auto-resume-task-contract-zh.md",
        VERIFY / "source-expectations.md", M5_RUN, M5_RUN.with_name("l1.py"),
        RUNTIME / "Cargo.lock", PUBLIC / "Cargo.lock", M4_REPLAY / "Cargo.lock",
        POC / "crates/l1-adapter/Cargo.lock", POC / "crates/derivation/Cargo.lock",
        POC / "crates/batch-services/Cargo.lock",
        POC / "verification/m3-auto-force/tester/run_matrix.py",
        POC / "verification/m4-resume/tester/run_matrix.py",
        VERIFY / "tester/regression.py",
    )
    report["inputs"] = {str(path.relative_to(WORKTREE)): helpers.sha(path) for path in inputs}

    def capture(label, argv):
        return helpers.capture(out, report, env, label, argv) == 0

    def locked(label, argv):
        return capture(label, helpers.focused(argv, flock, timeout))

    def live(label, argv, scenario=None, pause=0):
        if not capture(label, argv):
            return False
        try:
            report["bundles"][label] = helpers.check_run(out, args.candidate, label, scenario, pause)
        except (OSError, ValueError, KeyError) as error:
            report.setdefault("evidence_failures", {})[label] = str(error)
            return False
        return True

    okay = True
    candidate = ("--candidate", args.candidate)
    def run_m5(label, scenario, pause):
        return live(label, [
            python, "-B", M5_RUN, *candidate, "--seed", "42", "--scenario", scenario,
            "--wall-pause-ms", str(pause), "--output", out / label,
        ], scenario=scenario, pause=pause)

    for case in LIVE[:2]:
        okay = run_m5(*case) and okay
    if {"primary", "primary-pause650"} <= report["bundles"].keys():
        try:
            report["repeat_equal"] = compare_repeat(helpers, out)
        except (OSError, ValueError) as error:
            report["repeat_equal"] = False
            report.setdefault("evidence_failures", {})["repeat"] = str(error)
        okay = report["repeat_equal"] and okay

    cargo = rust / "cargo"
    built = locked("independent-public-build", [
        cargo, "build", "--locked", "--offline", "--manifest-path", PUBLIC / "Cargo.toml",
    ])
    if built:
        binary = JOB / "target/debug/m2-force-independent"
        report["public_checker"] = {"path": str(binary), "sha256": helpers.sha(binary)}
        print(f"public checker {binary} {report['public_checker']['sha256']}", flush=True)
    okay = built and okay
    for case in LIVE[2:]:
        okay = run_m5(*case) and okay

    label = "m1-regression"
    ran = capture(label, [
        python, "-B", POC / "verification/m2-healthy-force/tester/m1_regression.py",
        "--worktree", WORKTREE, "--build-root", JOB, *candidate, "--output", out / label,
    ])
    if ran:
        try:
            report["bundles"][label] = m4.check_m1(helpers, out / label, args.candidate)
        except (OSError, ValueError, KeyError) as error:
            report.setdefault("evidence_failures", {})[label] = str(error)
            ran = False
    okay = ran and okay
    for milestone, scenario in (("m2", None), ("m3", "outage"), ("m4", "preset-resume")):
        label = f"{milestone}-regression"
        ran = live(label, [
            python, "-B", VERIFY / "tester/regression.py", milestone, *candidate, "--output", out / label,
        ], scenario=scenario)
        okay = ran and okay

    for label, manifest, filter_, needed in (
        ("m5-vectors", RUNTIME / "Cargo.toml", ("--lib", M5_FILTER), 3),
        ("m5-resume-clock-vector", RUNTIME / "Cargo.toml",
         ("--lib", "resume_clock_uses_a_half_open_lease_and_checks_transport_order"), 1),
        ("m5-native-stop-vector", RUNTIME / "Cargo.toml",
         ("--lib", "native_cache_obeys_the_half_open_lease_and_permanent_close"), 1),
        ("m4-replay-vectors", M4_REPLAY / "Cargo.toml", (), 10),
    ):
        passed = locked(label, [
            cargo, "test", "--locked", "--offline", "--manifest-path", manifest,
            *filter_, "--", "--nocapture", "--test-threads=1",
        ])
        counts = re.findall(
            r"test result: ok\. (\d+) passed", (out / f"{label}.stdout").read_text()
        )
        count = sum(map(int, counts))
        report[label] = {"passed": count, "expected": needed}
        okay = passed and count == needed and okay
    report["after"] = helpers.revision(args.candidate)
    return okay


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True, help="reviewed full Git SHA")
    parser.add_argument("--output", type=Path, required=True, help="fresh M5 artifact directory")
    args = parser.parse_args()
    output = args.output.resolve()
    artifacts = JOB / "artifacts"
    if output == artifacts or not output.is_relative_to(artifacts):
        parser.error("output must be a fresh directory within the M5 artifact root")
    sys.dont_write_bytecode = True
    m4 = load("m4_tester_helpers", POC / "verification/m4-resume/tester/run_matrix.py")
    helpers = m4.load_matrix_helpers()
    before = helpers.revision(args.candidate)
    if before["bft_gitlink"] != BFT:
        parser.error("candidate differs from pinned Regional BFT gitlink")
    subprocess.run(["git", "merge-base", "--is-ancestor", BASE, args.candidate], check=True)
    output.mkdir(mode=0o700)
    shutil.copyfile(Path(__file__), output / "run_matrix.py")
    shutil.copyfile(VERIFY / "tester/regression.py", output / "regression.py")
    report = {
        "candidate": args.candidate, "base": BASE, "seed": 42, "status": "FAIL",
        "argv": sys.orig_argv, "before": before,
        "heavy_lock": helpers.LOCK, "commands": [], "bundles": {},
        "runner_sha256": helpers.sha(output / "run_matrix.py"),
        "regression_runner_sha256": helpers.sha(output / "regression.py"),
    }
    try:
        okay = execute(helpers, m4, args, output, report)
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
