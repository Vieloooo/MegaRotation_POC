#!/usr/bin/env bash
set -euo pipefail
if [[ "$#" -ne 4 ]]; then
    printf '%s\n' 'usage: replay.sh CANDIDATE SEALED_MATRIX PUBLIC_CHECKER NEW_REPORT_DIRECTORY' >&2
    exit 2
fi

candidate=$1
matrix=$(realpath -e -- "$2")
checker=$(realpath -e -- "$3")
output=$4
repo=$(cd -- "$(dirname -- "$0")/../../.." && pwd)
verify="$repo/poc/verification"
python=/home/jy/miniconda3/bin/python3

test "$(git -C "$repo" rev-parse HEAD)" = "$candidate"
test -z "$(git -C "$repo" status --porcelain=v1 --untracked-files=all)"
mkdir -- "$output"

"$python" -B "$verify/m4-resume/check.py" \
    --repository "$repo" --candidate "$candidate" \
    --run "$matrix/primary" --repeat "$matrix/primary-pause650" \
    --public-checker "$checker" --output "$output/primary"

"$python" -B "$verify/m2-healthy-force/check.py" \
    --repository "$repo" --candidate "$candidate" --mode regular \
    --raw "$matrix/m1-regression/raw" \
    --public-checker "$checker" --output "$output/m1-regression"

"$python" -B "$verify/m2-healthy-force/check.py" \
    --repository "$repo" --candidate "$candidate" \
    --raw "$matrix/m2-regression/raw" \
    --receipts "$matrix/m2-regression/raw/l1/raw-rpc-bundle.json" \
    --driver-rpc "$matrix/m2-regression/raw/l1/driver-trace.json" \
    --hub-l1 "$matrix/m2-regression/raw/l1/hub-l1.jsonl" \
    --public-checker "$checker" --output "$output/m2-regression"

"$python" -B "$verify/m3-auto-force/check.py" \
    --repository "$repo" --candidate "$candidate" --mode outage \
    --run "$matrix/m3-regression" \
    --public-checker "$checker" --output "$output/m3-regression"

sha256sum "$checker" "$matrix/manifest.json" "$output"/*/report.json \
    "$output"/*/public-replay.json > "$output/SHA256SUMS"
