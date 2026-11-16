#!/usr/bin/env python3
"""Prepare a formal M1 run or execute the fixed healthy full-window smoke."""

import argparse
import fcntl
import hashlib
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

sys.dont_write_bytecode = True

import evidence as ev

ROOT = Path(__file__).resolve().parents[3]
RUST_BIN = Path("/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/"
                "1.95.0-x86_64-unknown-linux-gnu/bin")


def binary_info(binary):
    binary = binary.resolve(strict=True)
    with ev.regular(binary) as (stream, _):
        return {"path": str(binary), "sha256": hashlib.file_digest(stream, "sha256").hexdigest()}


def not_ready(request, extra=()):
    return {
        "code": "NotReady",
        "missing_ports": ev.missing_ports(request["scenario"]),
        "missing_inputs": ev.missing_inputs(request) + list(extra),
    }


class RunEvidence:
    def __init__(self, output, manifest):
        self.output = output
        self.manifest = manifest
        self.files = ev.Inventory(output)
        self.env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        self.env["GIT_OPTIONAL_LOCKS"] = "0"
        self.env["GIT_NO_REPLACE_OBJECTS"] = "1"
        self.env["LC_ALL"] = "C"
        output.mkdir(mode=0o700)
        self.save()
        (self.output / "invocation.json").write_bytes(ev.json_bytes({"argv": sys.orig_argv}))
        ref, _ = self.files.inspect("invocation.json")
        self.manifest["artifacts"].append({
            "role": "invocation", "producer": "launcher", "schema": "raw", **ref,
        })
        self.save()

    def save(self):
        temporary = self.output / "manifest.tmp"
        temporary.write_bytes(ev.json_bytes(self.manifest))
        temporary.replace(self.output / "manifest.json")

    def seal(self):
        self.save()
        digest = hashlib.sha256((self.output / "manifest.json").read_bytes()).hexdigest()
        with (self.output / "manifest.sha256").open("x", encoding="ascii") as sidecar:
            sidecar.write(f"{digest}  manifest.json\n")

    def capture(self, binary, *args, timeout_s=30, lock_owner=None, data=True):
        argv = [binary["path"], *args]
        name = f"commands/{len(self.manifest['commands']):03d}"
        stdout, stderr = (self.output / f"{name}.{suffix}" for suffix in ("stdout", "stderr"))
        stdout.parent.mkdir(exist_ok=True)
        code, timeout = None, False
        started = time.time_ns()
        before = time.monotonic_ns()
        with stdout.open("xb") as out, stderr.open("xb") as err:
            try:
                with subprocess.Popen(argv, cwd=ROOT, env=self.env, stdin=subprocess.DEVNULL,
                                      stdout=out, stderr=err, start_new_session=True) as child:
                    try:
                        code = child.wait(timeout=timeout_s)
                    except subprocess.TimeoutExpired:
                        timeout = True
                        try:
                            os.killpg(child.pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                        child.wait()
            except OSError as error:
                err.write(f"{error}\n".encode())
        record = {
            "argv": argv, "cwd": str(ROOT), "binary_sha256": binary["sha256"],
            "started_wall_ns": started, "elapsed_wall_ns": time.monotonic_ns() - before,
            "exit_code": code, "timeout": timeout, "lock_owner": lock_owner,
        }
        for stream, path in (("stdout", stdout), ("stderr", stderr)):
            ref, _ = self.files.inspect(path.relative_to(self.output).as_posix())
            record[stream] = ref
            self.manifest["artifacts"].append({
                "role": f"command_{stream}", "producer": "launcher", "schema": "raw", **ref,
            })
        self.manifest["commands"].append(record)
        self.save()
        ev.require(code == 0, f"command failed: {argv!r}; see {stderr}")
        return stdout.read_bytes() if data else None

    def git(self, *args):
        return self.capture(self.manifest["tool_versions"]["git"], *ev.GIT_OPTIONS, *args)

    def snapshot(self, store, prefix, name, expected=None, *, read=False):
        path = self.output / prefix / name
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("xb") as destination:
            _, data = store.inspect(name, expected, read=read, copy=destination)
        result, _ = self.files.inspect(path.relative_to(self.output).as_posix())
        self.manifest["source_files"].append(result)
        return result, data

    def revision(self):
        lines = self.git(*ev.GIT_REVISION).decode().splitlines()
        ev.require(len(lines) == 2, "invalid Git worktree or HEAD")
        top, head = lines
        ev.require(Path(top) == ROOT, f"wrong Git worktree: {top}")
        branch = self.git(*ev.GIT_BRANCH).decode().strip()
        status = self.git(*ev.GIT_STATUS).decode().splitlines()
        return {"head": head, "branch": branch, "status": status}

    def guard(self, revision):
        ev.require(revision["head"] == self.manifest["candidate"],
                   f"stale candidate: HEAD is {revision['head']}")
        ev.require(not revision["status"], "candidate worktree is dirty")

    def snapshot_candidate(self, *, additional=()):
        candidate = self.manifest["candidate"]
        deliveries = {item["delivery_sha"] for item in self.manifest["components"].values()
                      if item is not None}
        for ancestor in sorted({ev.M0} | deliveries):
            ev.require(self.git("cat-file", "-t", ancestor) == b"commit\n",
                       f"{ancestor}: expected a commit")
            self.git("merge-base", "--is-ancestor", ancestor, candidate)

        original = self.git("show", f"{ev.DISPATCH_SHA}:{ev.DISPATCH}")
        ev.require(hashlib.sha256(original).hexdigest() == ev.DISPATCH_CONTENT_SHA,
                   "wrong original dispatch")
        ev.check_git_index(self.git("ls-files", "-v", "-z"))
        commit = self.git("cat-file", "commit", candidate)
        root_tree = self.git(*ev.GIT_ROOT_TREE, candidate)
        poc_tree = self.git(*ev.GIT_POC_TREE, candidate, "--", "poc")
        repo = ev.Inventory(ROOT)
        sources = ev.candidate_sources(candidate, commit, root_tree, poc_tree, additional=additional)
        for path, object_id in sources.items():
            ref, data = self.snapshot(repo, "source", path, read=True)
            ev.check_source(path, data, ref["sha256"], object_id)
        self.manifest["tool_versions"]["bft_gitlink"] = ev.BFT
        self.manifest["tool_versions"]["commonware"] = ev.COMMONWARE


class Preparation(RunEvidence):
    def __init__(self, output, operation, input_ref, request):
        self.request = request
        manifest = {
            "schema": "M1-RUN-1",
            "operation": operation,
            "contract_sha256": ev.CONTRACT_SHA,
            "input": {**input_ref, "path": f"input/{input_ref['path']}"},
            "candidate": request["candidate"],
            "revision_before": None,
            "revision_after": None,
            "worktree": str(ROOT),
            "components": request["components"],
            "source_files": [],
            "tool_versions": {
                "git": None, "python": None, "rustc": None, "cargo": None,
                "commonware": None, "bft_gitlink": None,
            },
            "commands": [],
            "ports": [{**port, "status": "NotReady"} for port in request["ports"]],
            "scenario": request["scenario"],
            "seed": request["seed"],
            "start_order": request["start_order"],
            "wall_pause_ms": request["wall_pause_ms"],
            "limits": request["limits"],
            "status": "NOT_RUN",
            "readiness": "NotReady" if operation == "preflight" else None,
            "reason": {**not_ready(request), "stage": "Preparing"},
            "gates": [],
            "artifacts": [],
            "cutoff": {
                "virtual_tick_us": None, "event_id": None,
                "reader_pc": {"a": None, "b": None}, "reason": "NotStarted",
            },
            "pending": None,
        }
        super().__init__(output, manifest)

    def preflight(self, input_store):
        extra = []
        try:
            for name, ref in sorted(input_store.refs.items()):
                self.snapshot(input_store, "input", name, ref)
            tools = self.manifest["tool_versions"]
            tools["python"] = {**binary_info(Path(sys.executable)), "version": sys.version.split()[0]}
            git = shutil.which("git")
            if git is None:
                self.finish_not_ready(["tool_versions.git"])
                return
            tools["git"] = binary_info(Path(git))
            tools["git"]["version"] = self.capture(tools["git"], "--version").decode().strip()
            self.manifest["revision_before"] = self.revision()
            self.guard(self.manifest["revision_before"])
            self.snapshot_candidate()
            rust_dir = Path(os.environ.get("REGIONAL_BFT_RUST_BIN", RUST_BIN))
            for name in ("rustc", "cargo"):
                path = rust_dir / name
                if not path.is_file() or not os.access(path, os.X_OK):
                    extra.append(f"tool_versions.{name}")
                    continue
                tool = binary_info(path)
                tool["version"] = self.capture(tool, "--version").decode().strip()
                ev.require(tool["version"].split()[:2] == [name, ev.RUST],
                           f"wrong {name} version: {tool['version']}")
                tools[name] = tool
            self.manifest["revision_after"] = self.revision()
            self.guard(self.manifest["revision_after"])
            ev.require(self.manifest["revision_before"] == self.manifest["revision_after"],
                       "Git revision changed during preflight")
        except (ev.Invalid, OSError, UnicodeError) as error:
            self.manifest["status"] = "FAIL"
            if self.manifest["operation"] == "preflight":
                self.manifest["readiness"] = "Reject"
            self.manifest["reason"] = {"code": "InvalidInput", "detail": str(error)}
            self.seal()
            raise
        self.finish_not_ready(extra)

    def finish_not_ready(self, extra):
        self.manifest["reason"] = not_ready(self.request, extra)
        if self.manifest["operation"] == "run":
            self.manifest["gates"] = [
                {"id": gate, "status": "NOT_RUN", "reason": self.manifest["reason"],
                 "evidence_refs": []}
                for gate in ev.required_gates(self.request["scenario"])
            ]
        self.seal()


SMOKE_CONTRACT = "poc/docs/contracts/m1-healthy-smoke-v1.md"
SMOKE_CONTRACT_SHA = "e571b74e4f1aace1a8d670a33a909f24e6e66d675fcef71868392ae8cfd8425c"
SMOKE_DELIVERIES = {
    "h1": {"delivery_sha": "e35b9504e306c26a26b195b3ccce8df919924384"},
    "h2": {"delivery_sha": "b639076fb50f5fcbc06e309f67b1fb6125697242"},
    "h3": {"delivery_sha": "983054682b04ae6ec75e7a6f334b2026abcc68e6"},
}
SMOKE_RUNTIME = "poc/prototypes/i1b-runtime"
SMOKE_CACHE = Path("/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m1-regular")
HEAVY_LOCK = Path("/tmp/megaroation-poc-heavy.lock")
SMOKE_SEED = 42
BUILD_TIMEOUT_S = 1800
SMOKE_TIMEOUT_S = 1800


class Smoke(RunEvidence):
    def __init__(self, output, candidate):
        manifest = {
            "schema": "M1-SMOKE-RUN-1", "candidate": candidate, "worktree": str(ROOT),
            "scenario": "healthy-full-window", "seed": SMOKE_SEED, "start_order": "forward",
            "contract_sha256": SMOKE_CONTRACT_SHA, "components": SMOKE_DELIVERIES,
            "revision_before": None, "revision_after": None, "source_files": [],
            "tool_versions": {}, "commands": [], "artifacts": [], "status": "RUNNING",
            "limits": {"build_timeout_s": BUILD_TIMEOUT_S, "run_timeout_s": SMOKE_TIMEOUT_S,
                       "cargo_build_jobs": 2, "max_raw_bytes": ev.MAX_BYTES},
        }
        super().__init__(output, manifest)

    def prepare(self):
        tools = self.manifest["tool_versions"]
        tools["python"] = {**binary_info(Path(sys.executable)), "version": sys.version.split()[0]}
        git = shutil.which("git")
        ev.require(git is not None, "Git is required")
        tools["git"] = binary_info(Path(git))
        tools["git"]["version"] = self.capture(tools["git"], "--version").decode().strip()
        self.manifest["revision_before"] = self.revision()
        self.guard(self.manifest["revision_before"])
        source_paths = [SMOKE_CONTRACT, *self.git("ls-files", "--", f"{SMOKE_RUNTIME}/src")
                        .decode().splitlines()]
        self.snapshot_candidate(additional=source_paths)
        contract = next(ref for ref in self.manifest["source_files"]
                        if ref["path"] == f"source/{SMOKE_CONTRACT}")
        ev.require(contract["sha256"] == SMOKE_CONTRACT_SHA, "smoke contract differs from freeze")

        rust_dir = Path(self.env.get("REGIONAL_BFT_RUST_BIN", RUST_BIN))
        tool_paths = {name: binary_info(rust_dir / name) for name in ("rustc", "rustdoc", "cargo")}
        for name, info in tool_paths.items():
            info["version"] = self.capture(info, "--version").decode().strip()
            ev.require(info["version"].split()[:2] == [name, ev.RUST],
                       f"incorrect {name} version: {info['version']}")
            tools[name] = info
        build_env = {
            "PATH": str(rust_dir) + os.pathsep + os.defpath,
            "CARGO_HOME": str(SMOKE_CACHE / "cargo"),
            "CARGO_TARGET_DIR": str(SMOKE_CACHE / "target"),
            "CARGO_BUILD_JOBS": "2",
            "RUSTC": tools["rustc"]["path"],
            "RUSTDOC": tools["rustdoc"]["path"],
            "CARGO_NET_OFFLINE": "true",
            "CARGO_TERM_COLOR": "never",
        }
        self.env.update(build_env)
        self.manifest["build_environment"] = build_env
        self.save()

    def execute(self):
        self.prepare()
        lock_owner = f"pid={os.getpid()} path={HEAVY_LOCK}"
        try:
            with HEAVY_LOCK.open("a") as lease:
                deadline = time.monotonic() + BUILD_TIMEOUT_S
                while True:
                    try:
                        fcntl.flock(lease, fcntl.LOCK_EX | fcntl.LOCK_NB)
                        break
                    except BlockingIOError:
                        ev.require(time.monotonic() < deadline, f"timed out waiting for {HEAVY_LOCK}")
                        time.sleep(1)
                self.capture(self.manifest["tool_versions"]["cargo"], "build", "--locked", "--offline",
                             "--manifest-path", f"{SMOKE_RUNTIME}/Cargo.toml", "--bin", "m1-smoke",
                             timeout_s=BUILD_TIMEOUT_S, lock_owner=lock_owner, data=False)
                binary = binary_info(SMOKE_CACHE / "target/debug/m1-smoke")
                self.manifest["smoke_binary"] = binary
                run_env = {"M1_CANDIDATE": self.manifest["candidate"]}
                self.env.update(run_env)
                self.manifest["run_environment"] = run_env
                self.capture(binary, "--seed", str(SMOKE_SEED), "--output", str(self.output / "raw"),
                             timeout_s=SMOKE_TIMEOUT_S, lock_owner=lock_owner, data=False)
        finally:
            self.manifest["revision_after"] = self.revision()
            self.guard(self.manifest["revision_after"])
            ev.require(self.manifest["revision_before"] == self.manifest["revision_after"],
                       "Git revision changed during smoke")
        for name in ("raw/trace.jsonl", "raw/frames.bin"):
            ref, _ = self.files.inspect(name)
            ev.require(ref["bytes"] > 0, f"empty smoke evidence: {name}")
        self.manifest["status"] = "COMPLETED"

    def index_raw(self):
        raw = self.output / "raw"
        if not raw.exists():
            return
        for parent, dirs, files in os.walk(raw, followlinks=False):
            dirs.sort()
            for directory in dirs:
                with ev.directory(Path(parent) / directory):
                    pass
            for name in sorted(files):
                path = (Path(parent) / name).relative_to(self.output).as_posix()
                ref, _ = self.files.inspect(path)
                self.manifest["artifacts"].append({
                    "role": "raw_smoke", "producer": "m1-smoke", "schema": "raw", **ref,
                })


def run_smoke(args):
    try:
        output = ev.fresh_destination(args.output)
        ev.hex_id(args.candidate, 40, "--candidate")
        attempt = Smoke(output, args.candidate)
    except (ev.Invalid, OSError) as error:
        print(f"m1 smoke: {error}", file=sys.stderr)
        return 2

    try:
        try:
            attempt.execute()
        finally:
            attempt.index_raw()
    except (ev.Invalid, OSError, UnicodeError) as error:
        attempt.manifest["status"] = "FAIL"
        attempt.manifest["failure"] = str(error)
        print(f"m1 smoke: {error}", file=sys.stderr)
        return 2
    finally:
        attempt.seal()
    print(output / "manifest.json")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    operations = parser.add_subparsers(dest="operation", required=True)
    for operation in ("preflight", "run"):
        command = operations.add_parser(operation)
        command.add_argument("--candidate", required=True)
        command.add_argument("--input", type=Path, required=True)
        command.add_argument("--output", type=Path, required=True)
    smoke = operations.add_parser("smoke")
    smoke.add_argument("--candidate", required=True)
    smoke.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.operation == "smoke":
        return run_smoke(args)
    output, attempt = None, None
    try:
        output = ev.fresh_destination(args.output)
        ev.hex_id(args.candidate, 40, "--candidate")
        store = ev.Inventory(args.input.parent)
        input_ref, request = ev.input_document(store, args.input.name)
        ev.require(args.candidate == request["candidate"], "--candidate differs from input candidate")
        attempt = Preparation(output, args.operation, input_ref, request)
        attempt.preflight(store)
    except (ev.Invalid, OSError, UnicodeError) as error:
        if output is not None and attempt is None:
            try:
                output.mkdir(mode=0o700)
                (output / "rejection.json").write_bytes(ev.json_bytes({
                    "operation": args.operation, "argv": sys.orig_argv, "status": "FAIL",
                    "reason": {"code": "InvalidInput", "detail": str(error)},
                }))
            except OSError as report_error:
                print(f"m1 {args.operation}: could not save rejection: {report_error}",
                      file=sys.stderr)
        print(f"m1 {args.operation}: {error}", file=sys.stderr)
        return 2
    print(ev.json_bytes(attempt.manifest["reason"]).decode(), end="")
    return 3


if __name__ == "__main__":
    sys.exit(main())
