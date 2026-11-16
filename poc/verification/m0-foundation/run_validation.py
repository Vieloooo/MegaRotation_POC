#!/usr/bin/env python3
"""Run the frozen M0 compatibility checks on a clean, named integration revision."""

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parents[3]
DATA = Path("/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m0-foundation")
TOOLS = DATA.parents[1] / "tools/foundry-1.0.0"
RUST_BIN = Path("/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin")
PYTHON = Path("/home/jy/miniconda3/bin/python3")
LOCK = "/tmp/megaroation-poc-heavy.lock"
VERIFY = Path("poc/verification")
CRATES = Path("poc/crates")
RECORDING_SHA = "6248072b3fd0eee1ae468f0961a4d3589291cba6"
RECORDING = DATA.parent / "16/artifacts/ec16-official-6248072-run02/bundle"
AUDIT_SHA = "43f106c905bb0877c2131bd7926eb099701d4e3936c0845e96ec7fb36a4e7cf5"


def sha256(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def inventory(directory):
    return {
        str(path.relative_to(directory)): sha256(path)
        for path in sorted(directory.rglob("*"))
        if path.is_file() and not path.is_symlink() and path.name != "launch-private.json"
    }


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def revision():
    return {
        "head": git("rev-parse", "HEAD").decode().strip(),
        "branch": git("branch", "--show-current").decode().strip(),
        "status": git("status", "--porcelain=v1", "--untracked-files=all").decode().splitlines(),
    }


def source_hashes():
    return {
        os.fsdecode(path): sha256(ROOT / os.fsdecode(path))
        for path in git("ls-files", "-z", "--", "poc").split(b"\0") if path
    }


def import_checker(path):
    spec = importlib.util.spec_from_file_location(path.stem, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Run:
    def __init__(self, candidate, output):
        self.candidate = candidate
        self.output = output
        self.target = DATA / "target"
        self.start = revision()
        self.guard(self.start)
        require((ROOT / ".build").resolve() == DATA, "expected task-local .build link")
        require(output.is_relative_to(DATA / "artifacts"), "output must use the M0 artifact directory")
        output.mkdir(mode=0o700)
        rust = Path(os.environ.get("REGIONAL_BFT_RUST_BIN", RUST_BIN)).resolve()
        self.cargo = rust / "cargo"
        self.debug = self.target / "debug"
        settings = {
            "REGIONAL_BFT_RUST_BIN": str(rust),
            "CARGO_BUILD_JOBS": "2",
            "CARGO_HOME": str(DATA / "cargo"),
            "CARGO_TARGET_DIR": str(self.target),
            "RUST_TEST_THREADS": "1",
            "PYTHONDONTWRITEBYTECODE": "1",
            "SIMT_VERIFY_ARTIFACTS": str(output / "kernel-case-inputs"),
            "PE_VERIFY_DIR": str(output / "pe-independent-files"),
            "PE_TEST_EVIDENCE_DIR": str(output / "pe-recovery-files"),
            "I1B_VERIFY_ARTIFACT_DIR": str(output / "native-runtime-files"),
            "I1B_VERIFY_WALL_PAUSE_MS": "0",
            "TMPDIR": str(output / "tmp"),
        }
        self.env = {
            **os.environ, **settings,
            "PATH": f"{rust}:{PYTHON.parent}:{os.environ['PATH']}",
        }
        for key in ("PE_VERIFY_DIR", "PE_TEST_EVIDENCE_DIR", "TMPDIR"):
            Path(settings[key]).mkdir()
        self.manifest = {
            "schema": "m0-foundation-validation-v1",
            "candidate": candidate,
            "worktree": str(ROOT),
            "output": str(output),
            "run_environment": settings,
            "revision_before": self.start,
            "submodule_index": git("ls-files", "-s", "regional_BFT_sim").decode().strip(),
            "commands": [],
            "checks": {},
            "groups": {},
            "wall_time_basis": "local monotonic elapsed seconds, including any lock wait",
        }

    def guard(self, actual):
        require(actual["head"] == self.candidate and not actual["status"],
                f"expected a clean candidate {self.candidate}, found {actual}")

    def save(self, name, value):
        (self.output / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")

    def run(self, name, args, *, heavy=False, fixture=None, suffix="stdout", binaries=()):
        self.guard(revision())
        args = [str(arg) for arg in args]
        stdout = self.output / f"{name}.{suffix}"
        stderr = self.output / f"{name}.stderr"
        command = ["timeout", "--kill-after=10s", "1800s"]
        if heavy:
            command += ["flock", "--timeout", "600", "--conflict-exit-code", "75", LOCK]
        command += args
        print(f"Running {name}", flush=True)
        before = time.monotonic()
        with stdout.open("wb") as out, stderr.open("wb") as err:
            result = subprocess.run(command, cwd=ROOT, env=self.env, stdout=out, stderr=err)
        record = {
            "name": name,
            "argv": command,
            "exit_code": result.returncode,
            "watchdog_or_kill": result.returncode in (124, 137),
            "lock_timeout": heavy and result.returncode == 75,
            "local_wall_elapsed_s": round(time.monotonic() - before, 3),
            "fixture": fixture,
            "stdout": stdout.name,
            "stdout_sha256": sha256(stdout),
            "stderr": stderr.name,
            "stderr_sha256": sha256(stderr),
            "binary_sha256": {str(path): sha256(path) for path in binaries if path.is_file()},
        }
        self.manifest["commands"].append(record)
        self.save("manifest.json", self.manifest)
        require(result.returncode == 0, f"{name} exited {result.returncode}, see {stderr}")
        require(all(path.is_file() for path in binaries), f"{name}: expected binary missing")
        return stdout

    def cargo_run(self, name, operation, manifest, *args, **kwargs):
        return self.run(
            name,
            [self.cargo, operation, "--manifest-path", manifest, "--locked", "--offline", *args],
            heavy=True, **kwargs,
        )

    def test(self, name, manifest, *options):
        self.cargo_run(name, "test", manifest, *options, "--", "--nocapture", "--test-threads=1")

    def compare(self, name, paths, expected=None):
        files = {str(path.relative_to(self.output)): sha256(path) for path in paths}
        self.manifest["checks"][name] = files
        first = paths[0].read_bytes()
        require(all(path.read_bytes() == first for path in paths[1:]), f"{name}: bytes differ")
        require(expected is None or sha256(paths[0]) == expected, f"{name}: unexpected digest")

    def root_and_03(self):
        self.test("03-workspace", "poc/Cargo.toml", "--workspace")
        base = VERIFY / "mock-plan-execution/implementation-v1"
        for name in ("m1", "m2", "identity", "extended"):
            self.run(f"03-{name}-reference", [PYTHON, base / f"check_{name}_vectors.py"])

    def kernel_02(self):
        self.test("02-kernel", VERIFY / "sim-communication/kernel-v1/Cargo.toml")
        evidence = self.output / "kernel-case-inputs"
        seeds = [7, 42, 99]
        pauses = [0, 2]
        files = [
            evidence / f"pure-replay-seed{seed}-reverse{reverse}-pause{pause}.json"
            for seed in seeds for reverse in ("false", "true") for pause in pauses
        ]
        require(all(path.is_file() for path in files), "02 kernel embedded replay evidence incomplete")
        self.manifest["checks"]["02-kernel-fixture"] = {
            "evidence_directory": str(evidence),
            "replays": len(files),
            "seeds": seeds,
            "declaration_orders": ["forward", "reverse"],
            "wall_pause_ms": pauses,
            "native_engines": 0,
        }

    def publisher_06(self):
        self.test("06-sim-espresso", CRATES / "sim-espresso/Cargo.toml")
        self.test("06-publisher", CRATES / "publisher/Cargo.toml")
        self.test("06-independent", VERIFY / "publisher-espresso/independent-harness/Cargo.toml")

    def viewer_07(self):
        harness = VERIFY / "viewer-gfr/independent-harness/Cargo.toml"
        self.cargo_run("07-m1-with-capacity", "run", harness, "--", "--capacity",
                       binaries=(self.debug / "viewer-gfr-independent-harness",))
        self.cargo_run("07-m2", "run", harness, "--bin", "m2", binaries=(self.debug / "m2",))
        checker = VERIFY / "viewer-gfr/simulation/check-traces.py"
        semantic_trace = import_checker(checker).semantic_trace
        checks = self.manifest["checks"].setdefault("07-semantic-traces", {})
        for scenario in ("G1", "G2"):
            traces = []
            for pause in (0, 2):
                name = f"{scenario}-7-{pause}"
                path = self.cargo_run(
                    name, "run", CRATES / "viewer-gfr/Cargo.toml",
                    "--example", "replay", "--", scenario, "7", str(pause),
                    suffix="jsonl", fixture={"scenario": scenario, "seed": 7, "wall_pause_ms": pause},
                    binaries=(self.debug / "examples/replay",),
                )
                traces.append(path)
                checks[name] = semantic_trace(path.read_bytes(), scenario, 7)
            self.compare(f"07-{scenario}-pause-pair", traces)
        self.save("07-semantic-traces.json", {"oracle": str(checker), "cases": checks})

    def espresso_16(self):
        manifest = CRATES / "espresso-compatibility/Cargo.toml"
        verification = VERIFY / "espresso-compatibility"
        binary = self.debug / "espresso-compatibility"
        self.test("16-compatibility", manifest)
        self.test("16-independent", verification / "independent-harness/Cargo.toml")
        self.cargo_run("16-build", "build", manifest, binaries=(binary,))
        self.run("16-offline", [
            PYTHON, verification / "run_offline_checks.py",
            "--binary", binary, "--output", self.output / "ec16-offline", "--candidate", self.candidate,
        ], heavy=True, binaries=(binary,))
        self.manifest["recording"] = {
            "candidate": RECORDING_SHA, "directory": str(RECORDING),
            "input_sha256": inventory(RECORDING),
        }
        directories = []
        for label, reader, pause in (("first", "first", 0), ("paused", "first", 37), ("requery", "requery", 0)):
            destination = self.output / f"ec16-official-{label}"
            directories.append(destination)
            fixture = {"recording_candidate": RECORDING_SHA, "reader": reader, "wall_pause_ms": pause}
            command = [binary, "replay", RECORDING, reader, destination]
            if pause:
                command += ["--pause-ms", str(pause)]
            self.run(f"16-recording-{label}", command, heavy=True, fixture=fixture, binaries=(binary,))
            self.run(f"16-recording-oracle-{label}", [
                PYTHON, verification / "check_capture.py", RECORDING,
                "--reader", reader, "--candidate", RECORDING_SHA,
                "--audit-file", destination / "public-audit.bin",
                "--projection-file", destination / "public-projection.json",
                "--output", self.output / f"16-recording-oracle-{label}.json",
            ], fixture=fixture)
        self.compare("16-audits", [path / "public-audit.bin" for path in directories], AUDIT_SHA)
        self.compare("16-public-projections", [path / "public-projection.json" for path in directories])
        require(self.manifest["recording"]["input_sha256"] == inventory(RECORDING), "recording inputs changed")

    def l1_09(self):
        manifest = CRATES / "l1-adapter/Cargo.toml"
        self.cargo_run("09-build", "build", manifest, binaries=(self.debug / "l1-adapter",))
        self.test("09-tests", manifest)
        forge_output = self.output / "forge-out"
        self.run("09-forge", [
            TOOLS / "forge", "build", "--root", "poc/contracts", "--use", TOOLS / "solc",
            "--offline", "--threads", "2", "--out", forge_output,
            "--cache-path", self.output / "forge-cache",
        ], heavy=True)
        self.run("09-abi", [PYTHON, CRATES / "l1-adapter/tests/check_abi.py", forge_output])
        self.run("09-golden", [PYTHON, VERIFY / "anvil-l1/golden_vectors.py", "check", "--cast", TOOLS / "cast"])

    def native(self):
        manifest = "poc/prototypes/i1b-runtime/Cargo.toml"
        verification = VERIFY / "native-engine-fresh"
        hub, worker, oracle = [self.debug / name for name in ("native-hub", "native-worker", "native-engine-verifier")]
        self.cargo_run("native-build", "build", manifest, "--bin", "native-hub", "--bin", "native-worker",
                       "--bin", "native-engine-verifier", binaries=(hub, worker, oracle))
        self.test("native-controlled-runtime", manifest, "--test", "i1b_runtime_independent")
        profiles = []
        for pause in (0, 2):
            destination = self.output / f"native-healthy-7-{pause}"
            profiles.append(destination)
            self.run(f"native-healthy-7-{pause}", [
                hub, "--scenario", "healthy", "--seed", "7", "--output", destination,
                "--wall-pause-ms", str(pause),
            ], heavy=True, fixture={"scenario": "healthy", "seed": 7, "wall_pause_ms": pause,
                                    "real_workers": 2, "scripted_signers": True}, binaries=(hub, worker))
            require((destination / "scenario.json").is_file(), f"{destination}: missing scenario")
        self.run("native-oracle", [
            PYTHON, verification / "check.py", *profiles,
            "--output", self.output / "native-oracle.json",
        ], heavy=True, binaries=(oracle,))
        reports = json.loads((self.output / "native-oracle.json").read_bytes())
        require(len(reports) == 2 and all(
            row["status"] == "PASS" and row["delivered"] == [2, 2] and row["seed"] == 7 for row in reports
        ), "expected two checked healthy seed 7 traces with both workers delivering two heights")
        for name in ("scenario.json", "frames.bin", "virtual-trace.jsonl"):
            self.compare(f"native-pause-pair/{name}", [path / name for path in profiles])
        evidence = import_checker(verification / "verify_pairs.py").evidence
        pairs = [evidence(path) for path in profiles]
        self.manifest["checks"]["native-complete-pair"] = pairs
        require(pairs[0] == pairs[1], "native journals/storage differ across pause pair")

    def execute(self):
        before = source_hashes()
        self.save("source-sha256.json", before)
        try:
            tools = {
                "rustc": [self.cargo.with_name("rustc"), "--version"],
                "cargo": [self.cargo, "--version"],
                "python": [PYTHON, "--version"],
                "git": ["git", "--version"],
                "flock": ["flock", "--version"],
                **{tool: [TOOLS / tool, "--version"] for tool in ("forge", "cast", "solc")},
            }
            self.manifest["tools"] = {}
            for name, argv in tools.items():
                version = self.run(f"version-{name}", argv).read_text().strip()
                self.manifest["tools"][name] = version
            for tool in ("rustc", "cargo"):
                require(self.manifest["tools"][tool].startswith(f"{tool} 1.95.0 "), f"unexpected {tool} version")
            for name, group in (
                ("M0-T03", self.root_and_03), ("M0-T02", self.kernel_02), ("M0-T06", self.publisher_06),
                ("M0-T07", self.viewer_07), ("M0-T16", self.espresso_16), ("M0-T09", self.l1_09),
                ("M0-TN", self.native),
            ):
                try:
                    group()
                    self.manifest["groups"][name] = "PASS"
                except Exception:
                    self.manifest["groups"][name] = "FAIL"
                    failure = traceback.format_exc()
                    (self.output / f"{name}.failure").write_text(failure)
                    print(failure, file=sys.stderr, flush=True)
                self.save("manifest.json", self.manifest)
        finally:
            after = revision()
            self.manifest["revision_after"] = after
            self.manifest["source_sha256_unchanged"] = source_hashes() == before
            self.manifest["passed"] = (
                after == self.start and self.manifest["source_sha256_unchanged"]
                and len(self.manifest["groups"]) == 7
                and all(value == "PASS" for value in self.manifest["groups"].values())
            )
            files = inventory(self.output)
            files.pop("manifest.json", None)
            self.manifest["artifact_sha256"] = files
            self.save("manifest.json", self.manifest)
        return 0 if self.manifest["passed"] else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True, help="full commit SHA, equal to clean HEAD")
    parser.add_argument("--output", type=Path, required=True, help="new directory under M0 artifacts")
    args = parser.parse_args()
    os.umask(0o077)
    # In-process import of the two legacy checkers must keep the candidate clean.
    sys.dont_write_bytecode = True
    return Run(args.candidate, args.output.resolve()).execute()


if __name__ == "__main__":
    raise SystemExit(main())
