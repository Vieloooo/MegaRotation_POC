"""Authenticate sealed M5 inputs, then independently replay D and G."""

import hashlib
import importlib.util
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

BASE = "e44b6e8e77aeb1aa3895c14109a57ff3894d7252"
BFT = "691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6"
CONTRACT = "poc/docs/milestones/M5-auto-resume-task-contract-zh.md"
EXPECTATIONS = "poc/verification/m5-auto-resume/source-expectations.md"
FROZEN = {
    CONTRACT: "72e3a946f0ba9f26624249cba9c73875541051e41c06186e0e9ceec37353a617",
    EXPECTATIONS: "ba025f53543fda045febf30afa254f1b65d3c2916fb1498c92ee836c0fc6eb4f",
}
US = 1_000_000
ANNOUNCEMENT = 10 * US
GRACE = 3 * US
EXPIRY = 12 * US
PREPARATION = 12 * US
DRIFT = 15 * US
EPSILON = US
READY = US // 2
CLOCK_SKEW = 1_000
WINDOW = 4


def need(condition, detail):
    if not condition:
        raise ValueError(detail)


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read(path):
    def unique(pairs):
        data = dict(pairs)
        need(len(data) == len(pairs), f"duplicate JSON key in {path}")
        return data
    return json.loads(path.read_bytes(), object_pairs_hook=unique)


def save(path, data):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(data, stream, sort_keys=True, indent=2)
        stream.write("\n")


def one(rows, label):
    rows = list(rows)
    need(len(rows) == 1, f"{label}: {len(rows)} occurrences")
    return rows[0]


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def setup(repository, candidate):
    git = ["git", "-C", str(repository)]
    need(subprocess.check_output([*git, "rev-parse", "HEAD"], text=True).strip() == candidate
         and not subprocess.check_output([*git, "status", "--porcelain=v1", "--untracked-files=all"]),
         "independent checker requires the clean exact candidate checkout")
    subprocess.run([*git, "merge-base", "--is-ancestor", BASE, candidate], check=True)
    m4path = repository / "poc/verification/m4-resume"
    sys.path.insert(0, str(m4path))
    m4 = load_module("m5_m4_checks", m4path / "check.py")
    legacy, wire = m4.setup(repository)
    return m4, legacy, wire


def seal(run, repository, candidate):
    manifest_path = run / "manifest.json"
    digest = sha(manifest_path)
    need(manifest_path.with_suffix(".sha256").read_text().split() == [digest, "manifest.json"],
         "manifest seal")
    manifest = read(manifest_path)
    need(manifest["candidate"] == candidate and manifest["seed"] == 42
         and manifest["status"] == "COMPLETED", "sealed run revision, seed or status")
    before, after = manifest["before"], manifest["after"]
    need(before == after and before["head"] == candidate and not before["status"]
         and before["bft_gitlink"] == BFT and manifest["commonware"] == "2026.10.0",
         "unverified or dirty candidate")
    inputs = manifest["inputs"]
    need(all(inputs.get(k) == v for k, v in FROZEN.items()), "exact frozen inputs not sealed")
    for name, expected in inputs.items():
        path = (repository / name).resolve(strict=True)
        need(path.is_relative_to(repository) and sha(path) == expected, f"source hash differs: {name}")
    inventory = manifest["files"]
    for name, expected in inventory.items():
        path = run / name
        need(not path.is_symlink() and path.resolve().is_relative_to(run) and path.is_file()
             and path.stat().st_size == expected["bytes"] and sha(path) == expected["sha256"],
             f"unsealed artifact: {name}")
    actual = {p.relative_to(run).as_posix() for p in (run / "raw").rglob("*") if p.is_file()}
    need(actual == {p for p in inventory if p.startswith("raw/")}, "raw artifact inventory differs")
    return manifest, digest


def slot(config, timestamp, upward=False):
    genesis = config["genesis"]
    delta = timestamp - genesis["timestamp_us"]
    base, span = genesis["height"], config["l2_slot_us"]
    rounded = -((-delta) // span) if upward else delta // span
    return max(base, base + rounded)


def timestamp(config, height):
    genesis = config["genesis"]
    return genesis["timestamp_us"] + (height - genesis["height"]) * config["l2_slot_us"]


def solve(config, g, d, inbox, horizon_min, horizon_max, *, d_origin, tail,
          inbox_additions=0, ready_budget=READY, prep=PREPARATION,
          announcement=ANNOUNCEMENT, drift=DRIFT, epsilon=EPSILON, skew=CLOCK_SKEW):
    """Independently compute the frozen AR07 envelope in unbounded integers."""
    bounded = (g, d, inbox, horizon_min, horizon_max, d_origin, inbox_additions,
               ready_budget, prep, announcement, drift, epsilon, skew)
    need(all(type(x) is int and 0 <= x <= 2**64 - 1 for x in bounded),
         "negative, noninteger or overflowing solve input")
    epoch0, l1_span = config["l1_genesis_timestamp_us"], config["l1_block_us"]
    need(epoch0 <= horizon_min <= horizon_max and l1_span and config["l2_slot_us"] > 0
         and (horizon_min - epoch0) % l1_span == (horizon_max - epoch0) % l1_span == 0
         and tail in {"HALT", "NEXT"}, "invalid solve epoch, inclusion interval or duty")
    b_lo, b_hi = (horizon_min - epoch0) // l1_span, (horizon_max - epoch0) // l1_span
    epoch_hi = max(0, b_hi - WINDOW + 1)
    time_height = slot(config, epoch0 + epoch_hi * l1_span, upward=True)
    origin_extra = max(0, epoch_hi - d_origin) + 1
    inbox_max = inbox + inbox_additions
    d_max = max(d, inbox_max, time_height) + origin_extra
    g_max = g if tail == "HALT" else max(g, slot(config, horizon_max + skew))
    guard = max(g_max, d_max, inbox_max)
    lower, upper = horizon_max + prep, horizon_min + drift - epsilon
    earliest = max(guard + 1, slot(config, lower, upward=True))
    selected_ts = timestamp(config, earliest)
    params = (lower, upper, d_max, inbox_max, g_max, guard, earliest, selected_ts, horizon_max + skew)
    bounded_output = all(0 <= x <= 2**64 - 1 for x in params)
    feasible = (bounded_output and prep >= announcement
                and announcement + ready_budget + skew <= prep + config["l2_slot_us"]
                and lower <= selected_ts <= upper)
    if feasible and earliest > config["genesis"]["height"]:
        prior = earliest - 1
        need(prior <= guard or timestamp(config, prior) < lower, "solution is not minimal")
    return {
        "b_lo": b_lo, "b_hi": b_hi, "epoch_hi": epoch_hi, "time_height": time_height,
        "origin_extra": origin_extra, "g_max": g_max, "d_max": d_max, "inbox_max": inbox_max,
        "h_guard": guard, "lower_us": lower, "upper_us": upper,
        "target": earliest if feasible else None, "target_us": selected_ts if feasible else None,
    }


def source_inputs(run, manifest, wire, legacy, *, expect_resume):
    from check_l1 import audit
    from derive_oracle import genesis_from_bytes
    from verified_inputs import project
    raw = run / "raw"
    bundle = read(raw / "l1/raw-rpc-bundle.json")
    config = wire.decode_config((raw / "config.bin").read_bytes(), "config.bin")
    dep_bytes = (raw / "dep.bin").read_bytes()
    need(bundle["config"]["chainId"] == config["chain_id"] == 31337
         and bundle["config"]["configRef"].lower()[2:] == config["ref"]
         and wire.domain_hash("PoC/DepFixture/v1", dep_bytes) == config["dep_fixture_hash"],
         "authenticated chain, configuration or deposits")
    credits = wire.decode_dep(dep_bytes, "dep.bin")
    genesis = genesis_from_bytes((raw / "genesis.bin").read_bytes(), wire)
    authority = one((run / path for path in manifest["files"]
                     if path.endswith("/CommitteeControlAuthority.sol/CommitteeControlAuthority.json")),
                    "sealed deployed Authority artifact")
    da_root = raw / "derivation"
    da_index = {path.stem.lower(): path.relative_to(da_root).as_posix()
                for path in (da_root / "da").glob("0x*.bin")}
    committee, abi, headers, batches, resumes = project(bundle, authority, da_index, da_root)
    need((committee.threshold, len(committee.members)) == (2, 3), "actual M5 authority is not 2-of-3")
    need(not [row for row in abi["events"] if row["kind"] == "ForceRecorded"], "unexpected Force")
    need(len(resumes) == int(expect_resume), "unexpected number of successful RESUMEs")
    stages = audit(bundle, legacy.load_rows(raw / "l1/driver-trace.json"))
    times = {height: int(row["timestamp"], 16) * US for height, row in headers.items()}
    return (raw, bundle, config, credits, genesis, committee, abi, headers, times,
            batches, resumes, stages, da_index)


def replay_d(raw, config, genesis, credits, times, batches, resumes, stages):
    from check_cut import verify_item
    from derive_oracle import Derive, l1_pos
    from wire import load
    rows = defaultdict(list)
    for line in (raw / "derivation/trace.jsonl").read_bytes().splitlines():
        row = load(line)
        rows[row["reader"]].append(row)
    need({"A", "B"} <= rows.keys(), "need both independent application-reader D traces")
    refs = {}
    for reader, stream in rows.items():
        seen = stages["read_us"].get(reader, {})
        tip = -1
        while tip + 1 in seen:
            tip += 1
        need(tip >= genesis["anchor"]["block"]["origin"], f"{reader}: incomplete genesis coverage")
        model = Derive(config, genesis, credits, times, WINDOW).advance(tip, batches, resumes)
        items = []
        for row in stream:
            height = row["l1_height"]
            need(height in seen and height <= tip and seen[height] <= row["tick_us"],
                 f"{reader}: D consumed an L1 block before its own verified delivery")
            for item in row.get("new_items", ()):
                need(item["state"]["metadata"]["origin"] <= height, "D origin beyond reader delivery")
                items.append(item)
        need(len(items) == len(model.blocks), f"{reader}: executed D prefix length differs")
        for actual, expected in zip(items, model.blocks, strict=True):
            verify_item(actual, expected)
        for resume in resumes:
            if resume.position[0] > tip:
                continue
            row = one((row for row in stream if row.get("step", {}).get("kind") == "cursor_set"
                       and row["step"]["position"] == l1_pos(resume.set_position)), "actual D CursorSet")
            need(row["step"]["resume_position"] == l1_pos(resume.position),
                 "D CursorSet has wrong historical RESUME coordinate")
        refs[reader] = model
    shared_l1 = min(max(refs[reader].at_end) for reader in ("A", "B"))
    shared_height = refs["A"].at_end[shared_l1]
    need(shared_height == refs["B"].at_end[shared_l1]
         and refs["A"].blocks[:shared_height] == refs["B"].blocks[:shared_height],
         "independent D inputs differ at the same complete L1 position")
    return refs


def public_replay(output, raw, checker, config, genesis, bundle, driver, refs, resumes, headers, times):
    from check_cut import check_positioned_cut
    cuts, paths = [], []
    for resume in resumes:
        pos = "-".join(map(str, resume.position))
        pair = [raw / "derivation" / f"cut-{reader}-{pos}.json" for reader in ("A", "B")]
        data = pair[0].read_bytes()
        need(pair[1].read_bytes() == data, "A/B canonical cut bytes differ")
        prefix = refs["A"].cuts[resume.position]
        need(prefix == refs["B"].cuts[resume.position], "A/B derive different positioned cuts")
        cuts.append(check_positioned_cut(
            data, json.loads(data), resume, prefix, genesis, config, bundle, driver, WINDOW,
        ))
        paths.extend(pair)
    controls_file = output / "controls.json"
    cuts_file = output / "verified-cuts.json"
    headers_file = output / "verified-l1-headers.json"
    save(controls_file, [])
    save(cuts_file, cuts)
    save(headers_file, [
        {"height": height, "timestamp_s": times[height] // US,
         "hash": list(bytes.fromhex(block["hash"][2:]))}
        for height, block in sorted(headers.items())
    ])
    public = json.loads(subprocess.check_output([
        str(checker), str(raw), str(controls_file), str(ANNOUNCEMENT), str(GRACE), str(EXPIRY),
        str(3 * US), str(headers_file), str(cuts_file), "--timer-history",
    ]))
    need(len(public["resumes"]) == len(resumes), "public replay RESUME count")
    save(output / "public-replay.json", public)
    save(output / "independent-D.json", {
        reader: {"blocks": model.blocks, "discarded": model.discarded, "at_end": model.at_end}
        for reader, model in refs.items()
    })
    return public, cuts, paths
