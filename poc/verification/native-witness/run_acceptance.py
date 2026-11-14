#!/usr/bin/env python3
"""Run the full frozen-candidate regression matrix under an external wall cap."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(tempfile.mkdtemp(prefix="native-witness-independent-"))
COMMAND = [
    "timeout", "--kill-after=5s", "60s", "./poc/scripts/bootstrap.sh", "test",
    "--manifest-path", "prototypes/i1b-runtime/Cargo.toml", "--locked", "--offline",
    "--no-fail-fast", "--", "--test-threads=1",
]


def provenance():
    paths = [
        *ROOT.glob("poc/prototypes/i1b-runtime/src/*.rs"),
        *ROOT.glob("poc/prototypes/i1b-runtime/*.toml"),
        *ROOT.glob("poc/prototypes/i1b-runtime/*.lock"),
        ROOT / "poc/vendor/commonware-runtime/src/tokio/runtime.rs",
        ROOT / "poc/docs/contracts/task-contract-native-witness-v1.md",
        Path(__file__), Path(__file__).with_name("acceptance.rs"),
    ]
    return {
        str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(paths) if path.is_file()
    }


before = provenance()
environment = os.environ.copy()
environment.update({
    "NATIVE_WITNESS_VERIFY_ARTIFACTS": str(OUT / "witness"),
    "NATIVE_STORAGE_VERIFY_ARTIFACTS": str(OUT / "storage"),
    "I1B_VERIFY_ARTIFACT_DIR": str(OUT / "runtime"),
})
result = subprocess.run(COMMAND, cwd=ROOT, env=environment, capture_output=True, timeout=67)
stdout = result.stdout.decode(errors="replace")
stderr = result.stderr.decode(errors="replace")
(OUT / "stdout.log").write_text(stdout)
(OUT / "stderr.log").write_text(stderr)
rows = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", stdout)
report = {
    "candidate": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "command": COMMAND, "cwd": str(ROOT), "output": str(OUT), "exit_code": result.returncode,
    "hashes_before": before, "hashes_after": provenance(),
    "test_result_rows": rows,
    "passed": sum(int(row[1]) for row in rows),
    "failed": sum(int(row[2]) for row in rows),
    "failed_tests": re.findall(r"^test (\S+) \.\.\. FAILED$", stdout, re.MULTILINE),
    "scope": "NATIVE-WITNESS fixture plus prior runtime/storage regressions; no native live engine, guarded signing, transport or full I1b claim",
}
(OUT / "run.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({key: report[key] for key in ("candidate", "output", "exit_code", "passed", "failed", "failed_tests")}, indent=2))
raise SystemExit(result.returncode)
