#!/usr/bin/env python3
"""Run the committed M1 smoke entry with the M2 build and Cargo cache."""

import argparse
from pathlib import Path
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worktree", type=Path, required=True)
    parser.add_argument("--build-root", type=Path, required=True)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    worktree = args.worktree.resolve(strict=True)
    if Path.cwd().resolve() != worktree:
        parser.error("run from the specified candidate worktree")
    build_root = args.build_root.resolve(strict=True)
    for name in ("cargo", "target"):
        if not (build_root / name).is_dir():
            parser.error(f"missing isolated build directory: {build_root / name}")

    sys.dont_write_bytecode = True
    sys.path.insert(0, str(worktree / "poc/scenarios/m1-healthy-regular"))
    import run as m1

    m1.SMOKE_CACHE = build_root
    return m1.run_smoke(args)


if __name__ == "__main__":
    raise SystemExit(main())
