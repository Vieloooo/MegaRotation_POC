#!/usr/bin/env python3
"""Use the candidate's M2 or M3 runner with the M4 Cargo and output directories."""

import argparse
import importlib.util
from pathlib import Path
import subprocess
import sys

JOB = Path("/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m4-resume")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("milestone", choices=("m2", "m3"))
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    root = Path(subprocess.check_output(
        ["git", "rev-parse", "--show-toplevel"], text=True
    ).strip()).resolve(strict=True)
    if Path.cwd().resolve() != root:
        parser.error("run from the root of the candidate worktree")
    output = args.output.resolve()
    artifacts = JOB / "artifacts"
    if output == artifacts or not output.is_relative_to(artifacts):
        parser.error("output must be a fresh directory under the M4 artifact root")
    if not (JOB / "cargo").is_dir() or not (JOB / "target").is_dir():
        parser.error("M4 Cargo cache and target directories must exist")

    sys.dont_write_bytecode = True
    directory = "m2-healthy-force" if args.milestone == "m2" else "m3-auto-force"
    runner = root / "poc/scenarios" / directory / "run.py"
    spec = importlib.util.spec_from_file_location(f"m4_{args.milestone}_regression", runner)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {runner}")
    launch = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(launch)
    launch.JOB = JOB
    sys.argv[1:] = [
        "--candidate", args.candidate, "--seed", "42", "--wall-pause-ms", "0",
        "--output", str(output),
    ]
    if args.milestone == "m3":
        sys.argv[1:] += ["--scenario", "outage"]
    launch.main()


if __name__ == "__main__":
    main()
