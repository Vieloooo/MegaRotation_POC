#!/usr/bin/env bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
candidate=8ba78f1ace3b383caa1c4360401c645d25907326
data=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/06
exact_head=0
if [[ ${1:-} == --exact-head ]]; then
  exact_head=1
  shift
fi
run=${1:-$data/artifacts/verification/$candidate/independent-run1}
cd "$repo"
if [[ $exact_head == 1 ]]; then
  test "$(git rev-parse HEAD)" = "$candidate"
fi
git diff --exit-code "$candidate" -- poc/crates/publisher poc/crates/sim-espresso
test -z "$(git ls-files --others --exclude-standard -- poc/crates/publisher poc/crates/sim-espresso)"
mkdir -p "$(dirname "$run")" "$data/target/verification"
mkdir "$run"
mkdir "$run/external-fixture" "$run/recovery-faults"
export PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH
export CARGO_HOME=$data/cargo
export CARGO_TARGET_DIR=$data/target/verification
export CARGO_BUILD_JOBS=2
export PE_VERIFY_DIR=$run/external-fixture
export PE_TEST_EVIDENCE_DIR=$run/recovery-faults
{
  printf 'candidate=%s\nactual_head=' "$candidate"
  git rev-parse HEAD
  git status --short
  rustc --version
  cargo --version
} > "$run/environment.txt"
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/verification/publisher-espresso/independent-harness/Cargo.toml --locked --offline -- --nocapture --test-threads=1 2>&1 | tee "$run/independent-harness.log"
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/crates/publisher/Cargo.toml --locked --offline --lib recovery_tests:: -- --nocapture --test-threads=1 2>&1 | tee "$run/reviewed-recovery-tests.log"
git diff --exit-code "$candidate" -- poc/crates/publisher poc/crates/sim-espresso
python3 - "$run" <<'PY'
import hashlib, pathlib, sys
root = pathlib.Path(sys.argv[1])
items = []
for file in sorted(root.rglob('*')):
    if file.is_file():
        items.append(f'{hashlib.sha256(file.read_bytes()).hexdigest()}  {file.relative_to(root)}')
(root/'SHA256SUMS').write_text('\n'.join(items)+'\n')
print(f'Raw evidence: {root}')
print(f'Hashed evidence files: {len(items)}')
PY
