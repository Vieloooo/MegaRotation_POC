#!/usr/bin/env python3
"""Run the Publisher/SimEspresso component matrix and seal raw evidence."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import time
import zlib

CANDIDATE = "8ba78f1ace3b383caa1c4360401c645d25907326"
ROOT = Path(__file__).resolve().parents[3]
DATA = Path("/mnt/reth/jy/megaroation_poc/.build/commander-jobs/06")
RUST = Path("/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin")
CRATES = ["poc/crates/sim-espresso", "poc/crates/publisher"]
LOCK = ["flock", "-w", "120", "-E", "75", "/tmp/megaroation-poc-heavy.lock", "timeout", "120"]
ORDERS = {7: [2, 0, 1], 42: [2, 1, 0], 99: [0, 2, 1]}
TIPS = {7: [None, 0, 2], 42: [None, None, 2], 99: [0, 0, 2]}
WAIT_TICKS = {7: [141], 42: [], 99: [140, 141]}
THETAS = [104, 104, 104, 105]
PROPOSED = [110, 112, 114, 116]
FINALIZED = [111, 113, 115, 117]
PAYLOADS = [b"P\x09\x02\x01", b"B\x09\x02", b"A\x09\x01\x04", None]
NAMESPACE = b"fixture-publisher"
READER = b"publisher-reader"


def require(condition, label):
    if not condition:
        raise RuntimeError(label)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def pack(fmt, *values):
    return struct.pack("<" + fmt, *values)


def field(data):
    return pack("I", len(data)) + data


def message(payload):
    return pack("I", 7) + field(payload)


def pelg(q):
    count = int(PAYLOADS[q] is not None)
    return (
        b"PELG" + pack("H", 1) + field(b"S")
        + pack("QQQBQI", q, THETAS[q], PROPOSED[q], 1, FINALIZED[q], count)
        + (message(PAYLOADS[q]) if count else b"")
    )


def reader_snapshot(reader, visible, pending, ticks):
    expected = b"PERS" + pack("H", 1) + field(b"S") + field(reader)
    expected += pack("II", len(visible), len(pending))
    for q in visible + pending:
        count = int(PAYLOADS[q] is not None)
        expected += pack("QBQQBQIQI", q, 1, THETAS[q], PROPOSED[q], 1,
                         FINALIZED[q], count, ticks[q], count)
        if count:
            expected += pack("IQ", 0, ticks[q]) + message(PAYLOADS[q])
    return expected


def publisher_snapshot(final):
    # PESP v1 encodes the stable attempt identity by namespace, task number and
    # attempt position. Assert the full payload, dispatch/ACK and terminal data.
    body = field(NAMESPACE) + field(b"fixture-config-v1") + field(b"S") + field(READER)
    body += pack("QI", 9 if final else 3, 1)
    body += field(b"\x09\x02") + field(b"\x09\x02\x09\x01\x02\x04\x01")
    body += pack("QIBQB", 1, 0, 1, 105, 1)
    body += field(b"non-crypto:matched symbolic proof/body/ancestry/anchor")
    body += pack("I", 3) + b"".join(message(payload) for payload in PAYLOADS[:3])
    if final:
        body += pack("IQB", 2, 102, 1) + field(b"fixture-controlled-1")
        body += pack("QB", 106, 2) + field(b"fixture-controlled-2") + field(b"fixture-service-ack")
        body += pack("BQIQ", 1, 0, 0, 2) + field(b"S")
        body += field(b"non-crypto:located first proof and both bodies")
        body += pack("BQQQ", 4, 3, 105, 105)
    else:
        body += pack("IQBBB", 1, 102, 0, 0, 0)
    return b"PESP" + pack("HII", 1, len(body), zlib.crc32(body)) + body


def expected_trace(seed):
    def phase(event, tick, name, generation, attempts):
        return dict(event=event, tick=tick, phase=name, generation=generation, attempts=attempts)

    def reader(event, tick, tip, ticks):
        return dict(event=event, tick=tick, tip=tip, receipt_ticks=ticks)

    log = [
        dict(event="scenario", seed=seed, fixture="non-cryptographic"),
        phase("offer_wait", 100, "VERIFYING", 1, 0),
        phase("offer_queued", 101, "QUEUED", 2, 0),
        phase("attempt_prepared", 102, "QUEUED", 3, 1),
        dict(event="prepared_replay", tick=103, task_number=1, attempt=1,
             namespace_hex=NAMESPACE.hex(), original_tick=102),
        phase("dispatch_committed", 104, "SUBMITTED", 4, 1),
        dict(event="attempt_retry", tick=106, task_number=1, attempt=2,
             namespace_hex=NAMESPACE.hex()),
        phase("service_acknowledged", 108, "SUBMITTED", 7, 2),
    ]
    log.extend(dict(event="finalized", q=q, theta_us=THETAS[q],
                    proposed_at_us=PROPOSED[q], finalized_at_us=FINALIZED[q],
                    wire_hex=pelg(q).hex()) for q in range(4))
    log.append(phase("finalized_unreceived_reopen", 120, "SUBMITTED", 7, 2))
    log.extend(reader("reader_a", 130 + q, q, list(range(130, 131 + q))) for q in range(4))
    ticks = dict(zip(ORDERS[seed], [140, 141, 142]))
    for offset, (q, tip) in enumerate(zip(ORDERS[seed], TIPS[seed])):
        tick = 140 + offset
        log.append(dict(event="reader_b_delivered", tick=tick, q=q))
        receipts = [] if tip is None else [ticks[q] for q in range(tip + 1)]
        log.append(reader("reader_b", tick, tip, receipts))
        if tick in WAIT_TICKS[seed]:
            log.append(phase("public_material_wait", tick, "SUBMITTED", 7, 2))
    log.extend([
        dict(event="recorded_receipt", tick=144, q=0, message_index=0, complete_through_q=2),
        phase("recorded", 144, "RECORDED", 8, 2),
        reader("reader_b", 145, 3, [ticks[q] for q in range(3)] + [145]),
        phase("recorded_and_stopped_reopen", 146, "RECORDED", 9, 2),
    ])
    return log


def check_scenario(seed, trace, outputs):
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    expected = expected_trace(seed)
    require(len(rows) == len(expected), "canonical trace length")
    for index, (actual, wanted) in enumerate(zip(rows, expected), 1):
        require(actual == wanted, f"canonical trace line {index}: expected {wanted}, got {actual}")

    ticks = dict(zip(ORDERS[seed], [140, 141, 142]))
    first_q = ORDERS[seed][0]
    visible, pending = ([0], []) if first_q == 0 else ([], [first_q])
    snapshots = {
        "prepared.snapshot": publisher_snapshot(False),
        "publisher/state.snapshot": publisher_snapshot(True),
        "reader-a.snapshot": reader_snapshot(b"fixture-observer-A", list(range(4)), [],
                                              dict(enumerate(range(130, 134)))),
        "reader-b.snapshot": reader_snapshot(READER, list(range(4)), [], {**ticks, 3: 145}),
        "reader-b-pending.snapshot": reader_snapshot(READER, visible, pending, ticks),
    }
    artifacts = {**snapshots, **{f"q{q}.pelg": pelg(q) for q in range(4)}}
    actual_names = {path.relative_to(outputs).as_posix() for path in outputs.rglob("*") if path.is_file()}
    require(actual_names == set(artifacts) | {"publisher/writer.lock"}, "fixture output inventory")
    hashes = {"stdout.jsonl": sha(trace.read_bytes())}
    for name, wanted in artifacts.items():
        actual = (outputs / name).read_bytes()
        require(actual == wanted, f"literal bytes differ for {name}")
        hashes[name] = sha(actual)
    return hashes


def run(out_dir, command):
    stdout_name = "stdout.jsonl" if "scenario_trace" in command else "stdout.log"
    wall_start = time.monotonic()
    with (out_dir / stdout_name).open("wb") as stdout, (out_dir / "stderr.log").open("wb") as stderr:
        exit_code = subprocess.run(command, cwd=ROOT, stdout=stdout, stderr=stderr).returncode
    return dict(command=command, exit_code=exit_code, local_wall_seconds=round(time.monotonic() - wall_start, 6),
                stdout=str(out_dir / stdout_name), stderr=str(out_dir / "stderr.log"),
                status="PASS" if exit_code == 0 else "INCONCLUSIVE" if exit_code in (75, 124) else "FAIL")


def check_source(exact):
    actual = git("rev-parse", "HEAD")
    if exact:
        require(actual == CANDIDATE, f"expected execution HEAD {CANDIDATE}, got {actual}")
    git("diff", "--exit-code", CANDIDATE, "--", *CRATES)
    require(not git("ls-files", "--others", "--exclude-standard", "--", *CRATES), "untracked component source")
    return actual


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("run_dir", type=Path, help="new evidence directory under the pinned candidate's simulation directory")
    cli.add_argument("--exact-head", action="store_true", help="require HEAD itself to equal the reviewed candidate")
    args = cli.parse_args()
    actual = check_source(args.exact_head)
    run_dir = args.run_dir.resolve()
    base = DATA / "artifacts" / "simulation" / CANDIDATE
    require(run_dir.is_relative_to(base) and run_dir != base, f"run_dir must be under {base}")
    run_dir.mkdir(parents=True)
    fixture_dir = run_dir / "test-fixture-files"
    fixture_dir.mkdir()
    environ = dict(PATH=f"{RUST}:{os.environ.get('PATH', '')}", CARGO_HOME=str(DATA / "cargo"),
                   CARGO_TARGET_DIR=str(DATA / "target" / "simulation"), CARGO_BUILD_JOBS="2",
                   PE_TEST_EVIDENCE_DIR=str(fixture_dir))
    os.environ.update(environ)
    versions = {command: subprocess.check_output([command, "--version"], text=True).strip()
                for command in ["rustc", "cargo", "python3"]}
    require(all(versions[name].split()[1] == "1.95.0" for name in ["rustc", "cargo"]), "Rust pin mismatch")
    status_before = git("status", "--porcelain=v1", "--untracked-files=all")
    checked_files = git("ls-files", "--", *CRATES).splitlines()
    verify = Path(__file__).resolve().parent.relative_to(ROOT)
    verification_paths = [verify / name for name in ["expectations-v1.md", "vectors-v1.json", "candidate-review.md",
        "run-independent.sh", "independent-harness/Cargo.toml", "independent-harness/Cargo.lock",
        "independent-harness/tests/independent.rs", "run-matrix.py"]]
    inputs = checked_files + verification_paths + ["poc/docs/contracts/task-contract-publisher-espresso-v1.md"]
    meta = {"candidate": CANDIDATE, "execution_head": actual, "worktree": str(ROOT),
            "branch": git("branch", "--show-current"), "git_status_before": status_before.splitlines(),
            "source_and_input_sha256": {str(p): sha((ROOT / p).read_bytes()) for p in inputs},
            "tool_versions": versions, "environment": {**environ, "PATH": str(RUST) + ":<inherited>"}}
    (run_dir / "environment.json").write_text(json.dumps(meta, indent=2) + "\n")

    tests = []
    for crate, expected in zip(CRATES, [7, 8]):
        results_dir = run_dir / Path(crate).name
        results_dir.mkdir()
        res = run(results_dir, LOCK + ["cargo", "test", "--locked", "--offline", "--manifest-path",
                  f"{crate}/Cargo.toml", "--", "--test-threads=1"])
        res["passed"] = sum(int(count) for count in re.findall(r"test result: ok\. (\d+) passed", Path(res["stdout"]).read_text()))
        res["expected"] = expected
        if res["status"] == "PASS" and res["passed"] != expected:
            res.update(status="FAIL", oracle_error="unexpected test count")
        tests.append(res)

    runs, pairs = [], []
    for seed in ORDERS:
        pair = []
        for pause in [0, 2]:
            dir_path = run_dir / f"seed-{seed}-pause-{pause}ms"
            dir_path.mkdir()
            outputs = dir_path / "fixture"
            res = run(dir_path, LOCK + ["cargo", "run", "--locked", "--offline", "--manifest-path",
                      f"{CRATES[1]}/Cargo.toml", "--example", "scenario_trace", "--", str(seed), str(pause), str(outputs)])
            res.update(seed=seed, wall_pause_ms=pause, reader_b_delivery_order=ORDERS[seed] + [3])
            if res["status"] == "PASS":
                try:
                    res["canonical_sha256"] = check_scenario(seed, Path(res["stdout"]), outputs)
                    require(f"fixture=non-cryptographic seed={seed} wall_pause_ms={pause}" in Path(res["stderr"]).read_text(),
                            "missing wall-pause diagnostic")
                except (OSError, ValueError, RuntimeError) as err:
                    res.update(status="FAIL", oracle_error=str(err))
            pair.append(res)
            runs.append(res)
        pair_status = "INCONCLUSIVE"
        if all(res["status"] == "PASS" for res in pair):
            no_pause, with_pause = (Path(res["stdout"]).parent for res in pair)
            files = [Path("stdout.jsonl")]
            files.extend(Path("fixture") / name for name in pair[0]["canonical_sha256"] if name != "stdout.jsonl")
            same = all((no_pause / name).read_bytes() == (with_pause / name).read_bytes() for name in files)
            pair_status = "PASS" if same else "FAIL"
        pairs.append(dict(seed=seed, status=pair_status))

    require(check_source(args.exact_head) == actual, "execution HEAD changed")
    (run_dir / "git-status-after.txt").write_text(git("status", "--porcelain=v1", "--untracked-files=all") + "\n")
    statuses = {item["status"] for item in tests + runs + pairs}
    status = "FAIL" if "FAIL" in statuses else "INCONCLUSIVE" if "INCONCLUSIVE" in statuses else "PASS"
    summary = dict(candidate=CANDIDATE, execution_head=actual, status=status,
                   raw_evidence=str(run_dir), test_suites=tests, runs=runs, pairs=pairs,
                   deferred_integration={gate: "INCONCLUSIVE / OPEN" for gate in ["D02", "D03", "D04", "D07", "D16", "D06-integrated"]})
    (run_dir / "matrix.json").write_text(json.dumps(summary, indent=2) + "\n")
    files = sorted(path for path in run_dir.rglob("*") if path.is_file())
    manifest = "".join(f"{sha(path.read_bytes())}  {path.relative_to(run_dir)}\n" for path in files)
    (run_dir / "SHA256SUMS").write_text(manifest)
    print(f"{status} {run_dir} ({len(files)} files, SHA256SUMS {sha(manifest.encode())})")
    print(json.dumps({"tests": [item["status"] for item in tests], "runs": [item["status"] for item in runs], "pairs": pairs}))
    return 0 if status == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
