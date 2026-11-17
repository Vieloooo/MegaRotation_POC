#!/usr/bin/env python3
"""Independently verify M5's real stalled/recovered public logs and native data."""

import argparse
import json
from pathlib import Path

from evidence import (
    EXPIRY, US, need, one, public_replay, read, replay_d, save, seal, setup, sha, source_inputs,
)
from governance import check_authorization
from timeline import check_g, check_resume, check_rpc, check_stall, events, hx

POSITIVE = {"stopped", "frozen", "slow-rpc", "divergence"}
CASES = POSITIVE | {"late", "short", "inputs-wait", "no-target"}


def read_mode(manifest, requested):
    actual = manifest.get("scenario", manifest.get("case"))
    need((requested is None or actual == requested) and actual in CASES,
         f"sealed run scenario differs: {actual!r}, expected {requested!r}")
    return actual


def check_main(args, repository, run, manifest, manifest_hash, m4, legacy, wire):
    from wire import frames

    mode = read_mode(manifest, args.mode)
    has_resume = mode in POSITIVE or mode == "late"
    (raw, bundle, config, credits, genesis, committee, abi, headers, times, batches, resumes,
     driver, da_index) = source_inputs(run, manifest, wire, legacy, expect_resume=has_resume)
    need(config["genesis"]["timestamp_us"] == 10 * US and config["l1_block_us"] == 2 * US
         and config["l2_slot_us"] == US and config["t_drift_us"] == 15 * US,
         "frozen M5 clock/slot/drift configuration")
    need(len(credits) >= 50 and len(batches) == 1, "expected one real ordinary h1 batch and complete DEP")
    [batch] = batches
    need(batch.position[0] == 6 and batch.records[0]["origin"] == 3 and
         batch.records[0]["height"] == 1 and batch.records[0]["transactions"],
         "ordinary h1 accepted batch does not fit frozen Batcher profile")
    trace = list(legacy.trace_rows(raw))
    need(trace and not any(r["kind"] == "RunFailed" for r in trace)
         and [row["seq"] for row in trace] == list(range(1, len(trace) + 1))
         and [row["tick_us"] for row in trace] == sorted(row["tick_us"] for row in trace),
         "observed virtual trace ordering, completeness or success")

    checker = args.public_checker.resolve(strict=True)
    driver_config = read(raw / "l1/driver-config.json")
    refs = replay_d(raw, config, genesis, credits, times, batches, resumes, driver)
    args.output.mkdir()
    public, cuts, cut_paths = public_replay(
        args.output, raw, checker, config, genesis, bundle, driver_config, refs, resumes, headers, times,
    )
    old_g = {item["height"]: item for item in public["regional_entries"] if item["beta"] == 0}
    need(1 in old_g and refs["A"].blocks and old_g[1]["native"] == refs["A"].blocks[0]["native_hash"]
         and refs["A"].blocks[0]["kept"], "independent D h1 is not the actual ordinary G h1")
    m4.check_batch_sources(raw, batches, old_g, genesis, config, credits, bundle, driver)
    g_h1 = one((row for row in events(trace, "GAccepted") if row["detail"]["height"] == 1),
               "only old h1 execution entering G")
    need(g_h1["tick_us"] < driver["submit_us"][batch.tx_hash], "batch broadcast before its actual G source")
    pid_grants = {pid for direction, pid, _ in frames(raw / "frames.bin") if direction == 0}
    need(set(range(5)) <= pid_grants, "no actual native path through original R0")
    lifecycle = legacy.check_frames(raw, public, repository / "poc/proto/native.proto", trace,
                                    grant_pids=pid_grants, local_stop_ack_reuse=True)
    rpc = check_rpc(raw, legacy, {mode})
    g_rows, timers = check_g(trace, public)
    stall = check_stall(trace, public, driver, refs, lifecycle, mode)
    gov = check_authorization(raw, trace, public, refs, driver, genesis, bundle, batches,
                              resumes, config, committee, legacy, mode)
    report = {
        "status": "PASS", "mode": mode, "candidate": args.candidate,
        "manifest_sha256": manifest_hash, "public_checker_sha256": sha(checker),
        "ordering": stall, "clock": rpc, "governance": gov,
        "lifecycle": lifecycle, "g_append_count": len(g_rows), "timer_count": len(timers),
        "receipt": abi,
        "d_tips": {reader: len(model.blocks) for reader, model in refs.items()},
        "independently_verified_cuts": cuts,
    }

    if has_resume:
        [resume], [cut] = resumes, cuts
        # Every application decision must use its own reader's persisted compact
        # cut bytes. A new later D head can never substitute for this witness.
        name = f"cut-A-{'-'.join(map(str, resume.position))}.json"
        canonical = read(raw / "derivation" / name)
        resumed = check_resume(raw, trace, public, resume, cut, canonical, driver, lifecycle, config, mode)
        if mode != "divergence":
            need(resume.target == 30 and resume.signed_origin == 13 and
                 resumed["common_height"] == resumed["old_head"] == 1
                 and resumed["live_d_at_act"] >= 30,
                 "principal cases did not recover the compatible frozen h1 from target h30")
        if mode != "late":
            need(resumed["first_global_accept_us"] <= 60 * US, "runtime exceeded virtual cutoff")
        report["protocol"] = resumed
    else:
        need(not list(events(trace, "GReset")) and not list(events(trace, "ResumeActivated")),
             "negative run altered G through an unauthorised RESUME")
    return report, cut_paths, da_index


def check_repeat(run, repeat, manifest, second, cuts, da_index, legacy):
    need(read_mode(manifest, None) == read_mode(second, None) == "stopped", "wrong pause scenario")
    need(manifest["wall_pause_ms"] == 0 and second["wall_pause_ms"] == 650
         and manifest["binaries"] == second["binaries"], "repeat pause or binaries")
    raw, other = run / "raw", repeat / "raw"
    trace = legacy.normalized_trace(raw)
    need(trace == legacy.normalized_trace(other)
         == (raw / "trace.virtual.jsonl").read_bytes() == (other / "trace.virtual.jsonl").read_bytes(),
         "same-candidate wall pause changed canonical virtual trace")
    raw_der = raw / "derivation"
    paths = {
        raw / "frames.bin", raw / "l1/raw-rpc-bundle.json", raw / "l1/driver-trace.json",
        raw / "derivation/trace.jsonl", raw / "derivation/cursor-reads.jsonl",
        *cuts, *raw_der.glob("cut-*.json"), *raw_der.glob("batch-h*.json"),
        *raw_der.glob("accepted-*.json"), *raw_der.glob("da/0x*.receipt"),
        *(raw_der / file for file in da_index.values()),
    }
    matches = {}
    for path in sorted(paths):
        if not path.exists():
            need(path.name == "cursor-reads.jsonl", f"missing expected raw repeat artifact {path}")
            continue
        name = path.relative_to(run).as_posix()
        digest = sha(path)
        need(digest == sha(repeat / name), f"wall pause changed {name}")
        matches[name] = digest
    for pattern in ("cut-*.json", "batch-h*.json", "accepted-*.json", "da/0x*"):
        names = lambda base: {p.relative_to(base) for p in (base / "derivation").glob(pattern)}
        need(names(raw) == names(other), f"repeat produced a different artifact set for {pattern}")
    import hashlib
    return {"status": "PASS", "wall_pause_ms": 650,
            "normalized_trace_sha256": hashlib.sha256(trace).hexdigest(), "equal": matches}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--repository", type=Path, required=True)
    p.add_argument("--candidate", required=True)
    p.add_argument("--run", type=Path, required=True)
    p.add_argument("--repeat", type=Path)
    p.add_argument("--mode", choices=sorted(CASES))
    p.add_argument("--public-checker", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    args = p.parse_args()
    repository, run = [path.resolve(strict=True) for path in (args.repository, args.run)]
    m4, legacy, wire = setup(repository, args.candidate)
    manifest, digest = seal(run, repository, args.candidate)
    report, cuts, da_index = check_main(args, repository, run, manifest, digest, m4, legacy, wire)
    if args.repeat:
        repeated_run = args.repeat.resolve(strict=True)
        repeated, repeat_digest = seal(repeated_run, repository, args.candidate)
        report["repeat"] = check_repeat(run, repeated_run, manifest, repeated, cuts, da_index, legacy)
        report["repeat"]["manifest_sha256"] = repeat_digest
    save(args.output / "report.json", report)


if __name__ == "__main__":
    main()
