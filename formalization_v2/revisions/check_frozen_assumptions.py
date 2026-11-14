"""Check source identity and the literal model section; not a semantic proof."""
from hashlib import sha256
from pathlib import Path
import json
import re
import sys

root = Path(__file__).resolve().parent.parent
frozen = root / "revisions/v3_1_frozen"
manifest = json.loads((frozen / "manifest.json").read_text())
for name, expected in manifest.items():
    assert sha256((frozen / name).read_bytes()).hexdigest() == expected, name
    if name != "model_and_assumptions.tex":
        assert sha256((root / name).read_bytes()).hexdigest() == expected, name
model = (frozen / "model_and_assumptions.tex").read_text()
targets = [Path(p) for p in sys.argv[1:]]
if not targets:
    targets = []
    for path in root.glob("sync_hotstuff_rr_part2_rotation_v3_*.tex"):
        match = re.search(r"_v3_(\d+)(?:_(\d+))?\.tex$", path.name)
        if match and int(match.group(1)) >= 2:
            targets.append(path)
    targets.sort(key=lambda path: tuple(
        int(part) for part in path.stem.split("_v3_", 1)[1].split("_")
    ))
assert targets, "No revised source supplied or found"
for path in targets:
    source = path.read_text()
    start = source.index(r"\section{Model and assumptions}")
    end = source.index(r"\section{Regional engine: definitions}")
    assert source[start:end] == model, f"Model changed: {path}"
    print(f"UNCHANGED MODEL: {path.name}")
print("Literal checks passed. New protocol rules still need semantic assumption review.")
