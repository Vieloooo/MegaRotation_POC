#!/usr/bin/env python3
"""Check the GFR-07-v1 fixture traces without using the viewer or its reducer."""

import argparse
from collections import defaultdict
import hashlib
import itertools
import json
from pathlib import Path
import struct
import subprocess
import sys

CANDIDATE = "8907ad64d882673d6ef2142defdf6bff2e208a71"
PROFILE = "gfr-07-noncryptographic-rule-fixture-v1"
SEEDS = (7, 42, 99)
PAUSES = (0, 2)
KEYS = {
    "G1": [30_000_000, 31_000_000, 53_999_999, 54_000_000, 61_000_000, 64_000_000],
    "G2": [12_000_000, 13_000_000, 14_000_000, 15_000_000, 16_000_000, 23_999_999, 24_000_000, 52_000_000],
}
EXPECTED_SOURCE = {
    "G1": [("F20.1", "B20", "B11"), ("B10", "F10.1", "B1"), (), ("ignored",), (), ()],
    "G2": [
        ("F1.3", "F1.1"), ("B1",), ("F4.1", "B4"), ("B3", "F4.2", "F3.1"),
        ("F4.2", "F4.1", "B9"), ("F9.1", "B11", "F11.1", "B11", "F11.1"),
        ("B10", "F10.1"), (),
    ],
}


def eq(actual, expected, description):
    if actual != expected:
        raise ValueError(f"{description}: got {str(actual)[:280]}, expected {str(expected)[:280]}")


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def domain_hash(domain, *parts):
    return list(hashlib.sha256(b"".join(
        struct.pack(">Q", len(part)) + part for part in (domain.encode(), *parts)
    )).digest())


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def pack_blobs(blobs):
    return struct.pack(">I", len(blobs)) + b"".join(
        struct.pack(">I", len(blob)) + bytes(blob) for blob in blobs
    )


class Wire:
    def __init__(self, raw):
        self.raw = raw
        self.offset = 0

    def take(self, size):
        part = self.raw[self.offset:self.offset + size]
        eq(len(part), size, "wire length")
        self.offset += size
        return part

    def int(self, size, endian):
        return int.from_bytes(self.take(size), endian)

    def frame(self, endian):
        return self.take(self.int(4, endian))

    def finish(self):
        eq(self.offset, len(self.raw), "consumed wire length")


def source_objects(source, name):
    eq((source["kind"], source["format"], source["scenario"], source["finality"]),
       ("source", "gfr-07-fixture-trace-v1", name, PROFILE), "source profile")
    eq(source["l1_authentication"], "finite fixture; synthetic controls when present",
       "L1 authentication label")
    eq(bytes.fromhex(source["espresso_source_hex"]), b"gfr-07/noncrypto-espresso", "Espresso ID")
    eq(bytes.fromhex(source["l1_source_hex"]), f"gfr-07/noncrypto-l1-{name}".encode(), "L1 ID")
    l1 = [row["header"] for row in source["l1"]["rows"]]
    eq([(row["height"], row["timestamp_s"]) for row in l1],
       [(h, 2 * h) for h in range(31)], "finite L1 fixture")
    eq(source["dep"], {"first_height": 1, "rows": [[{"to": 2, "amount": h}] for h in range(1, 41)]},
       "finite DEP fixture")
    eq(source["parameters"],
       {"announcement_us": 12_000_000, "grace_us": 13_000_000 if name == "G1" else 3_000_000,
        "expiry_us": 30_000_000}, "fixture timer parameters")
    eq(source["controls"], [
        {"kind": kind, "position": {"block": 26, "tx": 1, "log": i}, "control_seq": i + 1,
         "source_hex": f"explicit-non-ABI-control-fixture-{i}".encode().hex()}
        for i, kind in enumerate(("Resume", "Force")) if name == "G1"
    ], "finite synthetic control inputs")

    eq(len(source["espresso"]), len(KEYS[name]), "finite Espresso fixture")
    bodies = {}
    proofs = {}
    labels = []
    for q, (row, key) in enumerate(zip(source["espresso"], KEYS[name], strict=True)):
        wire = Wire(bytes.fromhex(row["pelg_hex"]))
        eq((wire.take(4), wire.int(2, "little"), wire.frame("little")),
           (b"PELG", 1, b"gfr-07/noncrypto-espresso"), "PELG frame/source")
        eq([wire.int(8, "little") for _ in range(3)], [q, key, key + q * 1000],
           "q, public key, and virtual proposal tick")
        eq((wire.int(1, "little"), wire.int(8, "little")), (1, key + q * 1000 + 100),
           "scripted live flag and virtual finalization tick")
        eq(row["timestamp_assessment_1s"], "WithinBound", "fixture timestamp assessment label")
        labels.append([])
        for i in range(wire.int(4, "little")):
            codec = wire.int(4, "little")
            payload = Wire(wire.frame("little"))
            loc = (q, i)
            if codec != 0x47465201:
                eq((q, i, codec, payload.raw), (3, 0, 42, bytes([42])), "unknown codec fixture")
                eq(name, "G1", "unknown codec scenario")
                labels[-1].append(("ignored",))
                continue
            eq((payload.take(4), payload.int(2, "big")), (b"GFRP", 1), "GFR-PUBLIC-1 envelope")
            tag = payload.int(1, "big")
            if tag == 0:
                body = Wire(payload.frame("big"))
                eq(body.int(2, "big"), 1, "physical body version")
                parent = list(body.take(32))
                h, timestamp, origin = [body.int(8, "big") for _ in range(3)]
                root = list(body.take(32))
                txs = [list(body.frame("big")) for _ in range(body.int(4, "big"))]
                body.finish()
                bodies[loc] = {
                    "digest": domain_hash("PoC/PhysicalBlock/v1", body.raw),
                    "parent": parent, "height": h, "timestamp_us": timestamp, "origin": origin,
                    "root": root, "transactions": txs,
                }
                labels[-1].append(("body", loc))
            else:
                eq(tag, 1, "GFR-PUBLIC-1 proof tag")
                descriptor = payload.frame("big")
                target = payload.take(32)
                proof = payload.frame("big")
                eq(descriptor[:2], b"\x00\x01", "proof descriptor version")
                eq(proof[1:], PROFILE.encode() + bytes(domain_hash(PROFILE, descriptor, target, proof[:1])),
                   "proof fixture subject bytes")
                proofs[loc] = {
                    "key": {"cid": list(descriptor[-32:]), "block_hash": list(target)},
                    "payload": payload.raw.hex(), "variant": proof[0], "anchor_id": list(descriptor[-72:-40]),
                }
                labels[-1].append(("proof", loc))
            payload.finish()
        wire.finish()
    heights = {tuple(body["digest"]): body["height"] for body in bodies.values()}
    observed = [
        tuple("ignored" if kind[0] == "ignored" else (
            f"B{bodies[kind[1]]['height']}" if kind[0] == "body"
            else f"F{heights[tuple(proofs[kind[1]]['key']['block_hash'])]}.{proofs[kind[1]]['variant']}"
        ) for kind in block) for block in labels
    ]
    eq(observed, EXPECTED_SOURCE[name], "ordered raw proof/body sequence")
    return bodies, proofs


def pos(key, kind, index):
    index_name = {"Espresso": "Espresso", "Resume": "L1", "Expire": "Height", "SessionEnd": "Session"}[kind]
    return {"key_us": key, "kind": kind, "index": {index_name: index}}


def espresso(name, q):
    return pos(KEYS[name][q], "Espresso", q)


def session_end(key, r):
    return pos(key, "SessionEnd", {"beta": 0, "r": r})


def position_bytes(position):
    kind = position["kind"]
    index = next(iter(position["index"].values()))
    tag = {"Expire": 2, "SessionEnd": 3, "Espresso": 4}[kind]
    values = tuple(index[k] for k in ("beta", "r")) if kind == "SessionEnd" else (index,)
    return struct.pack(">HQB" + "Q" * len(values), 1, position["key_us"], tag, *values)


def compare_reader_state(steps):
    private_fields = {"reader", "label", "status", "records", "kind"}
    projections = defaultdict(dict)
    for reader in ("A", "B"):
        committed = []
        ids = set()
        previous_entries = []
        for s in (step for step in steps if step["reader"] == reader):
            key = canonical(s["pc"])
            eq(s["entries"][:len(previous_entries)], previous_entries, f"{reader} unchanged G prefix")
            previous_entries = s["entries"]
            for record in s["records"]:
                position = record["position"]
                position_wire = position_bytes(position)
                eq(record["position_hex"], position_wire.hex(), "record position bytes")
                for event in record["events"]:
                    eq(event["event"]["position"], position, "lifecycle position")
                    projection = bytes.fromhex(event["projection"])
                    eq(projection[:6 + len(position_wire)],
                       struct.pack(">HI", 1, len(position_wire)) + position_wire, "lifecycle position frame")
                    eq(event["id"], domain_hash("PoC/GfrLifecycle/v1", projection), "lifecycle ID")
                    identity = bytes(event["id"])
                    eq(identity in ids, False, "duplicate lifecycle ID")
                    ids.add(identity)
            committed.extend(s["records"])
            public = {
                **{k: v for k, v in s.items() if k not in private_fields},
                "records": committed.copy(),
            }
            prior = projections[key].setdefault(reader, canonical(public))
            eq(canonical(public), prior, f"{reader} unchanged public projection at {s['pc']}")
            if reader == "B":
                eq(canonical(public), projections[key]["A"], f"A/B public projection at {s['pc']}")
    for readers in projections.values():
        eq(set(readers), {"A", "B"}, "common completed positions")
    per_pc = {key.decode(): digest(value["A"]) for key, value in projections.items()}
    return len(projections), digest(canonical(per_pc))


def check_indexing(steps, bodies, proofs, name):
    for s in steps:
        if s["pc"]["kind"] == "Espresso":
            through_q = s["pc"]["index"]["Espresso"]
        else:
            through_q = max(q for q, key in enumerate(KEYS[name]) if key < s["pc"]["key_us"])
        expected = {
            loc: (bytes(body["digest"]),) for loc, body in bodies.items() if loc[0] <= through_q
        }
        expected.update({
            loc: (bytes(proof["key"]["cid"]), bytes(proof["key"]["block_hash"]), proof["payload"])
            for loc, proof in proofs.items() if loc[0] <= through_q
        })
        observed = [(tuple(loc), (bytes(body["digest"]),)) for body in s["bodies"] for loc in body["locations"]]
        observed += [
            (tuple(loc), (bytes(result["key"]["cid"]), bytes(result["key"]["block_hash"]), proof["payload_hex"]))
            for result in s["results"] for proof in result["proofs"] for loc in proof["locations"]
        ]
        eq(len(observed), len(expected), f"{s['label']} indexed object count")
        eq(dict(observed), expected, f"{s['label']} public object provenance")
        eq(s["ignored"], [{"error": "UnknownCodec(42)", "location": [3, 0]}]
           if name == "G1" and through_q >= 3 else [], f"{s['label']} ignored source location")


def metadata_wire(metadata):
    return struct.pack(">QQQ", *(metadata[k] for k in ("height", "timestamp_us", "origin")))


def state_root(metadata, accounts):
    encoded = b"\x00\x01" + metadata_wire(metadata) + struct.pack(">I", len(accounts))
    encoded += b"".join(
        struct.pack(">BQQ", int(k), accounts[k]["nonce"], accounts[k]["balance"])
        for k in sorted(accounts, key=int)
    )
    return domain_hash("PoC/ExecutionState/v1", encoded)


def check_execution(source, final, bodies, physical_in_order):
    genesis = source["genesis"]
    parent_hash = genesis["native_hash"]
    physical_parent = genesis["anchor"]["block"]["block_id"]
    parent_root = state_root(genesis["state"]["metadata"], genesis["state"]["accounts"])
    eq(parent_root, source["config"]["genesis_state_root"], "fixture genesis execution root")
    nonce = 0
    body_by_digest = {tuple(body["digest"]): body for body in bodies.values()}
    eq([s["h"] for s in final if s["physical"] is not None], physical_in_order, "physical/padding heights")
    for h, entry in enumerate(final, start=1):
        eq(entry["h"], h, "contiguous heights")
        timestamp = (h + 10) * 1_000_000
        metadata = {"height": h, "timestamp_us": timestamp, "origin": (h + 5) // 2}
        dep = [list(bytes([3]) + metadata_wire(metadata)), list(struct.pack(">BBQ", 2, 2, h))]
        kept = []
        if h in physical_in_order:
            kept = [list(struct.pack(">BBBQQ", 1, 1, 2, nonce, 3))]
            body = body_by_digest[tuple(entry["physical"])]
            eq(body, {"digest": entry["physical"], "parent": physical_parent, **metadata,
                      "root": parent_root, "transactions": kept * 2}, f"raw body and parent h{h}")
            physical_parent = entry["physical"]
            nonce += 1
        accounts = {
            "1": {"nonce": nonce, "balance": 100 - 3 * nonce},
            "2": {"nonce": 0, "balance": h * (h + 1) // 2 + 3 * nonce},
        }
        root = state_root(metadata, accounts)
        eq(entry["virtual"], {"metadata": metadata, "dep": dep, "transactions": kept * 2,
                             "pre_state_root": parent_root}, f"virtual raw transactions h{h}")
        eq(entry["post_state"], {"metadata": metadata, "accounts": accounts}, f"state transition h{h}")
        eq(entry["native"], {"metadata": metadata, "dep": dep, "kept": kept,
                            "parent": parent_hash, "post_state_root": root}, f"native Kept h{h}")
        native = (b"\x00\x01" + bytes(parent_hash) + metadata_wire(metadata) +
                  pack_blobs(dep) + pack_blobs(kept) + bytes(root))
        parent_hash = domain_hash("PoC/NativeBlock/v1", native)
        eq(entry["native_hash"], parent_hash, f"native hash h{h}")
        parent_root = root


def result_for(step, proof):
    key = proof["key"]
    matches = [r for r in step["results"] if r["key"] == key]
    eq(len(matches), 1, f"{step['label']} target {bytes(key['block_hash']).hex()}")
    return matches[0]


def wait(step, pc, next_position, diagnostic_prefix, reader="B"):
    eq((step["reader"], step["pc"], step["records"]), (reader, pc, []), f"{step['label']} rolls back")
    status = step["status"]
    eq((status["kind"], status["next_position"]), ("WAIT", next_position), step["label"])
    eq(any(m.startswith(diagnostic_prefix) for m in status["missing"]), True, f"{step['label']} diagnostic")


def ready(step, pc, head, records):
    eq(step["status"], {"kind": "READY"}, f"{step['label']} ready")
    eq((step["pc"], len(step["entries"])), (pc, head), f"{step['label']} pc and G height")
    eq([r["position"] for r in step["records"]], records, f"{step['label']} committed event order")
    eq(step["branch"], 0, f"{step['label']} branch")


def effects(record):
    return [(e["effect"], e.get("first"), e.get("last")) for e in record["effects"]]


def assignment(record, before, anchor, reference, window):
    events = [event["event"]["body"] for event in record["events"]]
    eq(events[0], {"SessionClosed": {"reason": "Complete", "session": {"beta": 0, "r": before}}},
       "predecessor closure")
    descriptor = events[1]["SessionAssigned"]["descriptor"]
    eq(descriptor["anchor"]["block"], anchor, "successor production anchor")
    eq(descriptor["plan"]["reference_block"], reference, "successor reference block")
    eq(descriptor["plan"]["session"], {"beta": 0, "r": before + 1}, "successor session")
    eq(descriptor["plan"]["window"], {"h_min": window[0], "h_max": window[1]}, "fixed successor window")
    eq(descriptor["plan"]["region"], (before + 1) % 3, "successor region")
    return descriptor


def accepted_ref(step, height):
    entry = step["entries"][height - 1]
    return {"height": height, "origin": entry["native"]["metadata"]["origin"], "block_id": entry["physical"]}


def segment(step, first, last, key, q_avail, accept):
    result = result_for(step, {"key": key})
    eq(result["q_avail"], q_avail, f"h{first}..{last} first public completeness")
    eq(result["merge_verified"], True, f"h{first}..{last} Merge verdict")
    eq([(e["source"], e["acceptance"]) for e in step["entries"][first - 1:last]],
       [(key, accept)] * (last - first + 1), f"h{first}..{last} sources and acceptance positions")


def halted(step, height):
    eq(step["tail"]["kind"], "HALT", "Expire tail")
    eq(step["tail"]["anchor"]["block"], accepted_ref(step, height), "Expire preserves production anchor")
    eq(step["closed_branches"], [0], "Expire closes branch")
    record = step["records"][-1]
    eq(effects(record), [("DUTY", None, None)], "Expire duty effect")
    eq([e["event"]["body"] for e in record["events"]],
       [{"BranchClosed": {"branch": 0, "reason": "Expired"}}], "Expire lifecycle")


def g1(a, b, source, bodies, proofs, seed):
    q0, q1, q2, q3, q4, _ = [espresso("G1", q) for q in range(6)]
    end = session_end(54_000_000, 2)
    expiry = pos(61_000_000, "Expire", 21)
    genesis = source["genesis"]["anchor"]["block"]
    checkpoints = [
        ("cache-before-selection", q0, 0, [q0]),
        ("two-handoffs-in-one-espresso-event", q1, 20, [q1]),
        ("old-session-timers-skipped", q2, 20, [q2]),
        ("session-end-without-new-physical", end, 20, [end]),
        ("process-watermarker-only-at-its-event", q3, 20, [q3]),
        ("expire-preserves-g", expiry, 20, [expiry]),
        ("halt-processes-espresso", q4, 20, [q4]),
    ]
    stop_label = "full-control-list-resume-precedes-force-and-espresso"
    eq(list(a), [row[0] for row in checkpoints] + [stop_label], "G1 A checkpoint sequence")
    for label, pc, h, records in checkpoints:
        ready(a[label], pc, h, records)
        ready(b[label], pc, h, records)
    cache, admitted, _, timer, _, expired, final = [a[row[0]] for row in checkpoints]
    eq(effects(cache["records"][0]), [], "cached unselected publication")
    eq(result_for(cache, proofs[(0, 0)])["q_avail"], None, "cached before selected merge")
    expected_waits = [
        ("partial-public-message-list", q0, None, f"EspressoMessages {{ q: 1, indices: [{seed % 3}] }}"),
        ("covering-l1-control-list-undelivered", q0, None, "L1ControlCoverage { through_height: 10 }"),
        ("later-origin-not-in-control-cut", q0, q1, "L1Header { height: 11 }"),
        ("second-handoff-plan-wait", q0, q1, "Input(Reference(BlockRef { height: 10,"),
        ("watermark-one-us-before-session-end", q2, None, "EspressoWatermark { key_us: 54000000 }"),
        ("halt-control-header-without-complete-list", q4, None, "L1ControlCoverage { through_height: 26 }"),
    ]
    for label, pc, next_position, reason in expected_waits:
        wait(b[label], pc, next_position, reason)
    eq(set(b), set(a) | {row[0] for row in expected_waits}, "G1 B checkpoint set")

    record = admitted["records"][0]
    eq(effects(record), [("APPEND", 1, 10), ("DUTY", None, None),
                         ("APPEND", 11, 20), ("DUTY", None, None)], "two same-q handoffs")
    eq(len(record["events"]), 4, "four lifecycle events in one record")
    h10, h20 = [accepted_ref(admitted, h) for h in (10, 20)]
    first = assignment(record, 0, h10, genesis, (11, 20))
    second = assignment({"events": record["events"][2:]}, 1, h20, h10, (21, 30))
    eq(admitted["tail"], {"kind": "NEXT", "descriptor": second}, "same-q successor selection")
    eq(first["cid"], proofs[(0, 0)]["key"]["cid"], "cached successor CID")
    segment(admitted, 1, 10, proofs[(1, 1)]["key"], 1, q1)
    segment(admitted, 11, 20, proofs[(0, 0)]["key"], 0, q1)
    eq(effects(timer["records"][0]), [("DUTY", None, None)], "SessionEnd without append")
    next_descriptor = assignment(timer["records"][0], 2, h20, h20, (31, 40))
    eq(timer["tail"], {"kind": "NEXT", "descriptor": next_descriptor}, "empty session fallback")
    eq(expired["entries"], admitted["entries"], "Expire preserves q1 G")
    halted(expired, 20)

    for reader in (a, b):
        stopped = reader[stop_label]
        eq((stopped["pc"], stopped["records"], stopped["tail"], stopped["entries"]),
           (q4, [], final["tail"], final["entries"]), "unsupported control preserves committed prefix")
        eq(stopped["status"], {"kind": "UNSUPPORTED_CONTROL", "control": "Resume",
                              "position": pos(64_000_000, "Resume", {"block": 26, "tx": 1, "log": 0})},
           "stop on Resume before later Force and Espresso q5")
    check_execution(source, final["entries"], bodies, [1, 10, 11, 20])


def g2(a, b, source, bodies, proofs):
    q0, q1, q2, q3, q4, q5, q6, q7 = [espresso("G2", q) for q in range(8)]
    end = session_end(24_000_000, 0)
    expiry = pos(52_000_000, "Expire", 12)
    checkpoints = [
        ("proof-only-public-history", q0, 0, [q0]),
        ("first-completeness-at-body-publication", q1, 1, [q1]),
        ("target-with-missing-public-physical-ancestor", q2, 1, [q2]),
        ("descendant-and-canonical-target-order", q3, 4, [q3]),
        ("multiple-proof-variants-retain-first-q", q4, 4, [q4]),
        ("strict-before-cutoff-and-cached-successor", q5, 9, [q5]),
        ("session-end-after-h9", end, 9, [end]),
        ("cutoff-equality-and-successor-at-current-key", q6, 11, [q6]),
        ("expire-before-equal-key-espresso", expiry, 11,
         [session_end(34_000_000, 1), session_end(44_000_000, 2), expiry]),
        ("halt-retains-accepted-prefix", q7, 11, [q7]),
    ]
    eq(list(a), [row[0] for row in checkpoints], "G2 A checkpoint sequence")
    for label, pc, h, records in checkpoints:
        ready(a[label], pc, h, records)
        ready(b[label], pc, h, records)
    proof_only, first, ancestor, descendant, variants, before, timer, successor, expired, final = [
        a[row[0]] for row in checkpoints
    ]
    eq(set(b), set(a) | {"public-body-locally-undelivered", "exact-retry-idempotent"}, "G2 B checkpoint set")
    wait(b["public-body-locally-undelivered"], q0, None, "EspressoMessages { q: 1, indices: [0] }")
    ready(b["exact-retry-idempotent"], q4, 4, [])
    eq(proof_only["bodies"], [], "proof-only q0 has no public body")
    eq(result_for(proof_only, proofs[(0, 1)])["q_avail"], None, "q0 body-gated proof")
    segment(first, 1, 1, proofs[(0, 1)]["key"], 1, q1)
    proof_states = result_for(first, proofs[(0, 1)])["proofs"]
    eq({p["payload_hex"]: p["verdict"] for p in proof_states},
       {proofs[(0, 1)]["payload"]: "Valid", proofs[(0, 0)]["payload"]: "Invalid"},
       "initial invalid proof excluded")
    eq(result_for(ancestor, proofs[(2, 0)])["q_avail"], None, "physical ancestor absent until q3")
    eq(effects(ancestor["records"][0]), [], "missing ancestor does not become padding")
    eq(effects(descendant["records"][0]), [("APPEND", 2, 3), ("APPEND", 4, 4)], "canonical target order")
    segment(descendant, 2, 3, proofs[(3, 2)]["key"], 3, q3)
    segment(descendant, 4, 4, proofs[(2, 0)]["key"], 3, q3)
    eq(variants["entries"], descendant["entries"], "proof variants do not change admitted prefix")
    eq(effects(variants["records"][0]), [], "proof variants do not append")
    evidence = result_for(variants, proofs[(2, 0)])
    eq(evidence["q_avail"], 3, "later valid proof publication preserves q_avail")
    eq({proof["payload_hex"]: (proof["verdict"], proof["locations"]) for proof in evidence["proofs"]},
       {proofs[(2, 0)]["payload"]: ("Valid", [[2, 0], [4, 1]]),
        proofs[(3, 1)]["payload"]: ("Valid", [[3, 1], [4, 0]])},
       "both valid proof payloads and all public locations")

    segment(before, 5, 9, proofs[(5, 0)]["key"], 5, q5)
    eq(result_for(before, proofs[(5, 2)])["q_avail"], None, "cached successor before selection")
    eq(effects(timer["records"][0]), [("DUTY", None, None)], "SessionEnd ahead of same-key Espresso")
    h9 = accepted_ref(timer, 9)
    nxt = assignment(timer["records"][0], 0, h9, source["genesis"]["anchor"]["block"], (11, 20))
    eq(timer["tail"], {"kind": "NEXT", "descriptor": nxt}, "SessionEnd fixed responsibility")
    eq(proofs[(5, 2)]["anchor_id"], h9["block_id"], "cached successor anchors to h9")
    eq(proofs[(5, 4)]["anchor_id"], [0x33] * 32, "wrong successor uses a distinct same-height anchor")
    eq(effects(successor["records"][0]), [("APPEND", 10, 11)], "only successor extends G at cutoff")
    segment(successor, 10, 11, proofs[(5, 2)]["key"], 5, q6)
    eq(successor["entries"][9]["physical"], None, "h10 is successor-covered padding")
    eq(result_for(successor, proofs[(6, 1)])["q_avail"], None, "old session cutoff-equal candidate excluded")
    eq(result_for(successor, proofs[(5, 4)])["q_avail"], None, "wrong h9 anchor candidate excluded")
    eq(expired["entries"], successor["entries"], "Expire preserves G through h11")
    halted(expired, 11)
    eq(final["tail"], expired["tail"], "q7 keeps HALT")
    eq(effects(final["records"][0]), [], "q7 has no acceptance after Expire")
    check_execution(source, final["entries"], bodies, [1, 3, 4, 9, 11])


def semantic_trace(raw, scenario, seed):
    lines = raw.splitlines()
    source, *steps = [json.loads(line) for line in lines]
    bodies, proofs = source_objects(source, scenario)
    eq({(s["kind"], s["reader"]) for s in steps}, {("step", "A"), ("step", "B")}, "trace readers")
    check_indexing(steps, bodies, proofs, scenario)
    comparisons, projection_hash = compare_reader_state(steps)
    by_reader = [{s["label"]: s for s in steps if s["reader"] == reader} for reader in ("A", "B")]
    eq(sum(map(len, by_reader)), len(steps), "duplicate trace step labels")
    if scenario == "G1":
        g1(*by_reader, source, bodies, proofs, seed)
    else:
        g2(*by_reader, source, bodies, proofs)
    return {
        "source_sha256": digest(lines[0] + b"\n"), "common_pc_comparisons": comparisons,
        "common_pc_projection_sha256": projection_hash,
        "reader_steps": {reader: len(steps) for reader, steps in zip(("A", "B"), by_reader, strict=True)},
        "final_pc": steps[-1]["pc"], "final_height": len(steps[-1]["entries"]),
        "final_status": steps[-1]["status"], "virtual_espresso_keys_us": KEYS[scenario],
    }


def check_capture(directory, summary):
    capture = json.loads((directory / "capture.json").read_text())
    eq(capture["candidate"], CANDIDATE, "captured revision")
    eq(capture["source_sha256_before"], capture["source_sha256_after"], "tracked source hashes")
    eq(capture["real_network_measurements"], None, "real network evidence")
    summary["capture_sha256"] = digest((directory / "capture.json").read_bytes())
    summary["binary"] = capture["binary"]
    summary["tools"] = capture["tools"]
    summary["environment"] = {
        **capture["environment"],
        "PATH": capture["environment"]["PATH"].split(":")[0] + ":$PATH",
    }
    summary["initial_revision"] = capture["initial_revision"]
    summary["final_revision"] = capture["final_revision"]
    eq(capture["tools"]["rustc"].splitlines()[0].startswith("rustc 1.95.0 "), True, "Rust version")
    eq(capture["tools"]["cargo"].startswith("cargo 1.95.0 "), True, "Cargo version")
    eq(capture["environment"]["CARGO_BUILD_JOBS"], "2", "build jobs")
    for revision in (capture["initial_revision"], capture["final_revision"]):
        check_revision(revision)

    summary["scenario_sources"] = {}
    for scenario in KEYS:
        name = f"{scenario}-7-0"
        source_line = (directory / f"{name}.jsonl").read_bytes().splitlines()[0]
        for seed, pause in itertools.product(SEEDS, PAUSES):
            name = f"{scenario}-{seed}-{pause}"
            summary["current"] = name
            run = json.loads((directory / f"{name}.run.json").read_text())
            for suffix in ("before", "after"):
                check_revision(run[f"revision_{suffix}"])
            eq((run["case"], run["scenario"], run["seed"], run["pause_ms"]),
               (name, scenario, seed, pause), "recorded run selection")
            eq(run["command"], [
                "flock", "--exclusive", "--timeout", "60", "/tmp/megaroation-poc-heavy.lock",
                "cargo", "run", "--locked", "--offline", "--manifest-path", "poc/crates/viewer-gfr/Cargo.toml",
                "--example", "replay", "--", scenario, str(seed), str(pause),
            ], "actual cargo command")
            eq(run["timed_out"], False, "no watchdog timeout")
            eq(run["exit_code"], 0, "case exit code")
            raw = (directory / f"{name}.jsonl").read_bytes()
            stderr = (directory / f"{name}.stderr").read_bytes()
            eq(digest(raw), run["stdout_sha256"], "raw stdout hash")
            eq(digest(stderr), run["stderr_sha256"], "raw stderr hash")
            tip = f"Some(({len(KEYS[scenario]) - 1}, PublicTimeUs({KEYS[scenario][-1]})))"
            eq(stderr.decode().splitlines()[-1],
               f"{scenario} seed={seed} wall_pause_ms={pause} A_tip={tip} B_tip={tip}", "runner stderr identity")
            eq(raw.splitlines()[0], source_line, "byte-invariant source line across seeds and pauses")
            checked = semantic_trace(raw, scenario, seed)
            scenario_source = {k: checked[k] for k in ("source_sha256", "common_pc_projection_sha256")}
            first_source = summary["scenario_sources"].setdefault(scenario, scenario_source)
            eq(scenario_source, first_source, "source and public projections invariant across seeds and pauses")
            summary["cases"].append({
                **checked, **{k: run[k] for k in (
                    "case", "scenario", "seed", "pause_ms", "exit_code", "timed_out", "local_wall_elapsed_ns",
                    "stdout_sha256", "stderr_sha256", "binary_sha256",
                )},
                "verdict": "PASS", "replay_fixture": str(directory / f"{name}.jsonl"),
                "stderr": str(directory / f"{name}.stderr"),
            })
            if pause:
                baseline = (directory / f"{scenario}-{seed}-0.jsonl").read_bytes()
                eq(raw == baseline, True, f"{scenario}/{seed} byte-identical pause traces")
                summary["pause_pairs"].append({
                    "scenario": scenario, "seed": seed, "verdict": "PASS", "stdout_sha256": digest(raw),
                })
    names = [case["case"] for case in summary["cases"]]
    eq(capture["complete_cases"], names, "recorded complete cases")
    eq(capture["requested_cases"], names, "recorded requested cases")
    binary_hashes = {case["binary_sha256"] for case in summary["cases"]}
    eq(len(binary_hashes), 1, "replay binary hash invariant")
    eq(None in binary_hashes, False, "binary hash present")
    summary["binary_sha256"] = next(iter(binary_hashes))
    summary["input_hashes_checked_after_capture"] = check_revision_inputs(capture)


def check_revision(revision):
    eq(revision["head"], CANDIDATE, "exact HEAD")
    eq(revision["branch"], "codex/poc-viewer-gfr", "exact branch")
    eq([line for line in revision["status"] if not line.startswith("?? ")], [], "tracked worktree changes")


def check_revision_inputs(capture):
    worktree = Path(capture["worktree"])
    inputs = capture["source_sha256_after"]
    supplemental = [
        "poc/Cargo.toml",
        "poc/proto/i1.proto",
        "poc/verification/mock-plan-execution/implementation-v1/public-registry.json",
    ]
    for path in [*inputs, *supplemental]:
        committed = subprocess.check_output(["git", "show", f"{CANDIDATE}:{path}"], cwd=worktree)
        sha = digest(committed)
        eq(digest((worktree / path).read_bytes()), sha, f"{path} matches verified Git object")
        if path in inputs:
            eq(inputs[path], sha, f"{path} captured input hash")
    return {path: digest((worktree / path).read_bytes()) for path in [*supplemental, MANIFEST_LOCK]}


MANIFEST_LOCK = "poc/crates/viewer-gfr/Cargo.lock"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifact_directory", type=Path)
    parser.add_argument("--summary", required=True, type=Path)
    args = parser.parse_args()
    directory = args.artifact_directory.resolve()
    summary = {
        "candidate": CANDIDATE, "contract": "GFR-07-v1", "artifact_directory": str(directory),
        "cases": [], "pause_pairs": [], "verification_profile": PROFILE, "real_network_measurements": None,
        "fixture_bounds": {"L1": [0, 30], "DEP": [1, 40], "G1_Espresso_q": [0, 5], "G2_Espresso_q": [0, 7]},
        "tester_script_sha256": {
            name: digest(Path(__file__).with_name(name).read_bytes())
            for name in ("run-matrix.py", "check-traces.py")
        },
        "open_gates": [
            "V22 native Regional BFT certificates and exact 04 FinalVerifier",
            "09 authenticated nonempty L1 controls",
            "VG39 and 12/13 historical ResumeCut and material staging",
            "08 controlled real-producer regular integration and downstream consumption",
        ],
    }
    try:
        check_capture(directory, summary)
        summary["verdict"] = "PASS"
        summary.pop("current")
        summary["VG35_wall_pause"] = "PASS"
    except (ValueError, OSError, KeyError, IndexError, subprocess.CalledProcessError) as error:
        summary["verdict"] = "INCONCLUSIVE" if isinstance(error, OSError) else "FAIL"
        summary["error"] = str(error)
    capture_path = directory / "capture.json"
    if capture_path.is_file():
        capture = json.loads(capture_path.read_text())
        summary["incomplete_cases"] = sorted(set(capture["requested_cases"]) - set(capture["complete_cases"]))
        if capture.get("failed_or_incomplete_case") in summary["incomplete_cases"]:
            name = capture["failed_or_incomplete_case"]
            run = json.loads((directory / f"{name}.run.json").read_text())
            summary["failed_or_incomplete_execution"] = {
                k: run[k] for k in ("case", "timed_out", "exit_code", "local_wall_elapsed_ns")
            }
            summary["verdict"] = "INCONCLUSIVE" if run["timed_out"] else "FAIL"
    args.summary.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(
        f"{summary['verdict']}: {len(summary['cases'])}/12 semantic cases, "
        f"{len(summary['pause_pairs'])}/6 byte-identical pause pairs",
        flush=True,
    )
    if summary["verdict"] != "PASS":
        print(f"{summary.get('current', 'capture')}: {summary.get('error')}", file=sys.stderr)
    return 0 if summary["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
