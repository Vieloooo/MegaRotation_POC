#!/usr/bin/env python3
import json
from pathlib import Path
import sys

here = Path(__file__).resolve()
root = here.parents[4]
out = Path(sys.argv[1])
for name in ("ControlAuthority", "BatchInbox"):
    expected = json.loads((root / f"poc/contracts/abi/{name}.json").read_text())
    actual = json.loads((out / f"{name}.sol/{name}.json").read_text())["abi"]
    assert actual == expected, f"ABI drift in {name}"
print("Both compiled ABIs match the published JSON.")
