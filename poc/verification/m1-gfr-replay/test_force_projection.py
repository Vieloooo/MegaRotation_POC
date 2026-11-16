"""Inspect the sealed Force projection and exercise targeted evidence mismatches."""

import argparse
import copy
from pathlib import Path

from force_projection import ForceBundle, VERIFY_NAME, build_force_projection, read_force_independent
from generate import equal
from test_projection import rejected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--independent", type=Path)
    args = parser.parse_args()
    bundle = ForceBundle(args.bundle)
    oracle = read_force_independent(args.independent or args.bundle.parent / VERIFY_NAME)
    replay = build_force_projection(bundle, oracle)
    entries = replay["entries"]
    force = replay["force"]
    equal((entries[9]["handoff"]["window"], force["anchor"]["height"], force["window"], force["q_star"]),
          ([11, 20], 10, [16, 25], 10), "Regular in flight and Force successor")
    equal((force["position"]["key_us"], force["replay"]["tick_us"], entries[10]["append"],
           entries[10]["body"]["height"], entries[10]["position"]["key_us"], entries[10]["excluded"]["seq"]),
          (22_000_000, 22_001_000, [], 11, 22_000_000, 8767), "same-key exclusion")
    equal((entries[11]["body"]["height"], entries[11]["replay"]["tick_us"], entries[11]["append"],
           [(p["height"], p["physical"], p["proof_target"]) for p in entries[11]["padding"]]),
          (16, 26_041_000, [11, 16], [(h, None, entries[11]["body"]["physical"]) for h in range(11, 16)]),
          "padding and physical h16")
    corrupt = copy.deepcopy(oracle)
    corrupt["public-replay"]["public_finals"][10]["proof"] = "00"
    rejected(lambda: build_force_projection(bundle, corrupt), "excluded old result still verifies exact proof bytes")
    corrupt = copy.deepcopy(oracle)
    corrupt["public-replay"]["expected"][10]["physical"] = entries[10]["body"]["physical"]
    rejected(lambda: build_force_projection(bundle, corrupt), "old physical h11 cannot become padding h11")
    corrupt = copy.deepcopy(oracle)
    corrupt["report"]["stages"]["reader_us"]["A"] += 1
    rejected(lambda: build_force_projection(bundle, corrupt), "reader A time must tie to actual delivery")
    print("PASS: L1/Regular/Force/exclusion/h16 landmarks and old result, padding, reader evidence mismatches")


if __name__ == "__main__":
    main()
