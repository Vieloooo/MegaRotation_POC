#!/usr/bin/env python3
"""Run the candidate's M2 scenario using the M3 Cargo and evidence directories."""

import importlib.util
from pathlib import Path
import sys

WORKTREE = Path("/home/jy/.codex/worktrees/poc-m3-auto-force/megaroation_poc")
M3_JOB = Path("/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m3-auto-force")


def main():
    if Path.cwd().resolve() != WORKTREE:
        raise SystemExit(f"run from the designated M3 worktree: {WORKTREE}")
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location(
        "m3_m2_regression", WORKTREE / "poc/scenarios/m2-healthy-force/run.py"
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load the candidate's M2 runner")
    launch = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(launch)
    launch.JOB = M3_JOB
    launch.main()


if __name__ == "__main__":
    main()
