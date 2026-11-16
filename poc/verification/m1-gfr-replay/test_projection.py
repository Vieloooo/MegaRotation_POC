"""Exercise the sealed projection and reject byte-level or oracle corruption."""

import argparse
import copy
import tempfile
from pathlib import Path

from generate import VERIFY_NAME, Bundle, build_projection, read_independent
from wire import EvidenceError, decode_espresso, decode_l1, decode_result, require


def rejected(operation, label):
    try:
        operation()
    except EvidenceError:
        return
    raise AssertionError(f"expected rejection: {label}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--independent", type=Path)
    args = parser.parse_args()
    bundle = Bundle(args.bundle)
    independent = read_independent(args.independent or args.bundle.parent / VERIFY_NAME)
    projected = build_projection(bundle, independent)

    actual = []
    for height in (10, 20, 21):
        entry = projected["entries"][height - 1]
        handoff = entry["handoff"]
        coverage = entry["coverage"]
        actual.append((entry["body"]["height"], entry["q"], entry["q_avail"],
                       entry["position"]["key_us"], entry["accept"]["tick_us"],
                       coverage["l1_control_height"], coverage["l1_origin_height"],
                       handoff["window"] if handoff else None))
    require(actual == [(10, 9, 9, 20_044_000, 20_045_000, 6, 7, [11, 20]),
                       (20, 19, 19, 30_090_000, 30_091_000, 11, 12, [21, 30]),
                       (21, 20, 20, 31_084_000, 31_085_000, 11, 13, None)], "landmark projection")

    for path, decoder in [
        ("raw/l1.bin", decode_l1), ("raw/espresso/0009.bin", decode_espresso),
        ("raw/results/10.bin", decode_result),
    ]:
        source = bundle.read(path)
        for mutated in (source[:-1], source + b"\0"):
            rejected(lambda: decoder(mutated, path), path)
    with tempfile.TemporaryDirectory() as root:
        directory = Path(root)
        (directory / "manifest.json").write_bytes((args.bundle / "manifest.json").read_bytes())
        (directory / "raw").mkdir()
        broken = directory / "raw/l1.bin"
        broken.write_bytes(b"tampered")
        rejected(lambda: Bundle(directory).read("raw/l1.bin"), "manifest file mismatch")
    corrupted = copy.deepcopy(independent)
    corrupted["public_finals"][9]["proof"] = "00"
    rejected(lambda: build_projection(bundle, corrupted), "independent proof mismatch")
    print("PASS: three landmarks, truncated/trailing wire bytes, artifact digest, oracle mismatch")


if __name__ == "__main__":
    main()
