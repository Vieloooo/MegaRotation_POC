#!/usr/bin/env python3
"""Replay a sealed M5 Tester matrix and archive independent reports."""

import argparse
from pathlib import Path
import re
import subprocess
import sys

from evidence import BASE, BFT, need, read, save, sha

LIVE = {
    "primary": "stopped",
    "watermark-frozen": "frozen",
    "late-inclusion": "late",
    "slow-rpc": "slow-rpc",
    "short-stall": "short",
    "divergence": "divergence",
    "no-target": "no-target",
    "inputs-wait": "inputs-wait",
}
REGRESSIONS = ("m1-regression", "m2-regression", "m3-regression", "m4-regression")
VECTORS = {
    "m5-vectors": 3,
    "m5-resume-clock-vector": 1,
    "m5-native-stop-vector": 1,
    "m4-replay-vectors": 10,
}


def verify_matrix(root, repository, candidate, checker):
    manifest_path = root / "manifest.json"
    manifest_hash = sha(manifest_path)
    need(manifest_path.with_suffix(".sha256").read_text().split() == [manifest_hash, "manifest.json"],
         "matrix manifest sidecar")
    data = read(manifest_path)
    need(data["status"] == "PASS" and data["candidate"] == candidate and data["seed"] == 42
         and data["base"] == BASE and data["before"] == data["after"]
         and data["before"]["head"] == candidate and not data["before"]["status"]
         and data["before"]["bft_gitlink"] == BFT
         and data["public_checker"]["sha256"] == sha(checker)
         and data["repeat_equal"] is True, "Tester matrix provenance, result or public checker")
    for relative, digest in data["inputs"].items():
        path = (repository / relative).resolve(strict=True)
        need(path.is_relative_to(repository) and sha(path) == digest, f"matrix input {relative}")
    runs = set(LIVE) | {"primary-pause650", *REGRESSIONS}
    need(set(data["bundles"]) == runs, "missing or unexpected Tester run")
    for name, row in data["bundles"].items():
        path = root / name / "manifest.json"
        expected = row["sha256"]
        need(row["path"] == f"{name}/manifest.json" and sha(path) == expected
             and path.with_suffix(".sha256").read_text().split() == [expected, "manifest.json"],
             f"Tester child manifest {name}")
    commands = {row["name"]: row for row in data["commands"]}
    need(len(commands) == len(data["commands"])
         and set(commands) == runs | VECTORS.keys() | {"independent-public-build"},
         "Tester command records")
    for name, row in commands.items():
        need(row["exit_code"] == 0, f"Tester command failed {name}")
        for stream in ("stdout", "stderr"):
            desc = row[stream]
            path = root / f"{name}.{stream}"
            need(desc["path"] == path.name and sha(path) == desc["sha256"]
                 and path.stat().st_size == desc["bytes"], f"Tester command artifact {path.name}")
    for name, expected in VECTORS.items():
        counts = re.findall(r"test result: ok\. (\d+) passed", (root / f"{name}.stdout").read_text())
        need(sum(map(int, counts)) == expected and data[name] == {"passed": expected, "expected": expected},
             f"focused vectors did not execute: {name}")
    return manifest_hash


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--candidate", required=True)
    p.add_argument("--matrix", type=Path, required=True)
    p.add_argument("--public-checker", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    args = p.parse_args()
    here = Path(__file__).resolve().parent
    repository, verify = here.parents[2], here.parent
    matrix, checker, output = args.matrix.resolve(strict=True), args.public_checker.resolve(strict=True), args.output.resolve()
    git = ["git", "-C", str(repository)]
    need(subprocess.check_output([*git, "rev-parse", "HEAD"], text=True).strip() == args.candidate
         and not subprocess.check_output([*git, "status", "--porcelain=v1", "--untracked-files=all"]),
         "requires the clean exact candidate")
    need(not output.is_relative_to(matrix) and not output.is_relative_to(repository),
         "reports require a fresh directory outside the sealed matrix and candidate")
    manifest_hash = verify_matrix(matrix, repository, args.candidate, checker)
    output.mkdir()
    reports = {}

    def replay(name, script, *argv):
        report = output / name
        command = [
            sys.executable, "-B", str(script), "--repository", str(repository),
            "--candidate", args.candidate, "--public-checker", str(checker),
            "--output", str(report), *map(str, argv),
        ]
        print(f"replay {name}", flush=True)
        subprocess.run(command, cwd=repository, check=True)
        result = read(report / "report.json")
        need(result["status"] == "PASS" and result["candidate"] == args.candidate, f"replay {name}")
        reports[name] = {"command": command, "sha256": sha(report / "report.json")}

    for name, mode in LIVE.items():
        repeat = ("--repeat", matrix / "primary-pause650") if name == "primary" else ()
        replay(name, here / "check.py", "--mode", mode, "--run", matrix / name, *repeat)

    old_m2 = verify / "m2-healthy-force/check.py"
    old_l1 = matrix / "m2-regression/raw/l1"
    replay("m1-regression", old_m2, "--mode", "regular", "--raw", matrix / "m1-regression/raw")
    replay("m2-regression", old_m2, "--raw", matrix / "m2-regression/raw",
           "--receipts", old_l1 / "raw-rpc-bundle.json", "--driver-rpc", old_l1 / "driver-trace.json",
           "--hub-l1", old_l1 / "hub-l1.jsonl")
    replay("m3-regression", verify / "m3-auto-force/check.py", "--mode", "outage",
           "--run", matrix / "m3-regression")
    replay("m4-regression", verify / "m4-resume/check.py", "--run", matrix / "m4-regression")

    save(output / "report.json", {
        "candidate": args.candidate, "status": "PASS", "matrix": str(matrix),
        "manifest_sha256": manifest_hash, "public_checker_sha256": sha(checker),
        "public_checker": str(checker), "focused_vectors": VECTORS, "replays": reports,
    })
    paths = sorted(path for path in output.rglob("*") if path.is_file())
    with (output / "SHA256SUMS").open("x", encoding="utf-8") as stream:
        for path in paths:
            stream.write(f"{sha(path)}  {path.relative_to(output).as_posix()}\n")


if __name__ == "__main__":
    main()
