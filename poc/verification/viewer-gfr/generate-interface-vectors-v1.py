#!/usr/bin/env python3
"""Materialize 07 contract layouts without importing or invoking candidate code.

The only imported fixture is 03's independently reviewed production-BE-v1
golden. That fixture explicitly does not claim valid registry/certificates.
"""
from pathlib import Path
import hashlib
import json

HERE = Path(__file__).resolve().parent
GOLDEN = HERE.parent / "mock-plan-execution/implementation-v1/m2-golden-vectors.json"
gold = {r["id"]: r for r in json.loads(GOLDEN.read_text())["records"]}
u16 = lambda n: n.to_bytes(2, "big")
u32 = lambda n: n.to_bytes(4, "big")
u64 = lambda n: n.to_bytes(8, "big")
frame = lambda b: u32(len(b)) + b
lst = lambda xs: u32(len(xs)) + b"".join(frame(x) for x in xs)
raw = lambda name: bytes.fromhex(gold[name]["hex"])


def position(key, kind, index):
    tag = {"Force": 0, "Resume": 1, "Expire": 2, "SessionEnd": 3, "Espresso": 4}[kind]
    return u16(1) + u64(key) + bytes([tag]) + b"".join(u64(x) for x in index)


def hash_parts(domain, data):
    d = domain.encode()
    return hashlib.sha256(u64(len(d)) + d + u64(len(data)) + data).hexdigest()


positions = []
for name, key, kind, index in [
    ("expire_100_9", 100, "Expire", [9]),
    ("espresso_100_7", 100, "Espresso", [7]),
    ("session_100_2_3", 100, "SessionEnd", [2, 3]),
    ("resume_100_1_2_3", 100, "Resume", [1, 2, 3]),
    ("force_100_1_2_3", 100, "Force", [1, 2, 3]),
    ("zero_espresso", 0, "Espresso", [0]),
    ("max_expire", 2**64 - 1, "Expire", [2**64 - 1]),
    ("max_control", 2**64 - 1, "Force", [2**64 - 1] * 3),
    ("max_session", 2**64 - 1, "SessionEnd", [2**64 - 1] * 2),
]:
    b = position(key, kind, index)
    positions.append(dict(id=name, key_us=key, kind=kind, index=index,
                          expected_hex=b.hex(), expected_length=len(b)))

negative_positions = []
for p in positions:
    b = bytes.fromhex(p["expected_hex"])
    for suffix, malformed in [("truncated", b[:-1]), ("extra", b + b"\0"),
                              ("version2", b"\0\2" + b[2:])]:
        negative_positions.append(dict(id=p["id"] + "_" + suffix,
                                       hex=malformed.hex(), expected="REJECT"))
negative_positions.extend([
    dict(id="empty", hex="", expected="REJECT"),
    dict(id="unknown_kind", hex=(u16(1) + u64(0) + b"\xff" + u64(0)).hex(), expected="REJECT"),
])

comparisons = [
    {"a": [100, "Resume", [1, 0, 0]], "b": [100, "Force", [1, 0, 1]], "expected": "Less"},
    {"a": [100, "Force", [1, 0, 1]], "b": [100, "Resume", [1, 0, 0]], "expected": "Greater"},
    {"a": [100, "Force", [1, 0, 0]], "b": [100, "Resume", [1, 0, 0]], "expected": "REJECT"},
    {"a": [100, "Force", [9, 9, 9]], "b": [100, "Expire", [0]], "expected": "Less"},
    {"a": [100, "Expire", [9]], "b": [100, "SessionEnd", [0, 0]], "expected": "Less"},
    {"a": [100, "SessionEnd", [9, 9]], "b": [100, "Espresso", [0]], "expected": "Less"},
    {"a": [100, "Espresso", [7]], "b": [100, "Espresso", [8]], "expected": "Less"},
    {"a": [99, "Espresso", [999]], "b": [100, "Force", [0, 0, 0]], "expected": "Less"},
    {"a": [100, "Resume", [1, 0, 0]], "b": [100, "Resume", [1, 0, 0]], "expected": "Equal"},
]

pos = position(100, "Espresso", [7])
lifecycle = []


def life(name, body_input, tag, body):
    b = u16(1) + frame(pos) + bytes([tag]) + body
    lifecycle.append(dict(id=name, position=[100, "Espresso", [7]], body=body_input,
                          expected_hex=b.hex(), expected_length=len(b),
                          expected_id=hash_parts("PoC/GfrLifecycle/v1", b)))


reasons = ["Time", "Complete", "Force", "Resume", "Expired", "Excluded", "Incompatible"]
for i, reason in enumerate(reasons):
    life("session_closed_" + reason, dict(kind="SessionClosed", beta=2, r=3, reason=reason),
         1, u64(2) + u64(3) + bytes([i]))
    life("branch_closed_" + reason, dict(kind="BranchClosed", branch=2, reason=reason),
         2, u64(2) + bytes([i]))
life("session_closed_different_body", dict(kind="SessionClosed", beta=2, r=4, reason="Complete"),
     1, u64(2) + u64(4) + b"\1")
life("assigned_03_descriptor", dict(kind="SessionAssigned", descriptor_hex=raw("descriptor").hex()),
     0, frame(raw("descriptor")))

# Existing 03 Anchor canonical layout: version + raw BlockRef + timestamp.
anchor = u16(1) + u64(0) + u64(3) + bytes([0x11]) * 32 + u64(10_000_000)
vblock = dict(metadata_hex=raw("metadata-h1").hex(),
              dep_hex=[raw("dep-h1-0").hex(), raw("dep-h1-1").hex()],
              pre_state_root=gold["state-h0"]["expected_hash"], transactions_hex=[])
vbytes = (frame(raw("metadata-h1")) + lst([raw("dep-h1-0"), raw("dep-h1-1")])
          + bytes.fromhex(vblock["pre_state_root"]) + lst([]))
for ident, aid in [("resumed_03_anchor", anchor),
                   ("resumed_changed_anchor", anchor[:18] + bytes([0x22]) * 32 + anchor[50:])]:
    life(ident, dict(kind="Resumed", control_seq=5, branch=2, anchor_hex=aid.hex(), prefix=[vblock]),
         3, u64(5) + u64(2) + frame(aid) + lst([vbytes]))

publications = []
body = b"GFRP" + u16(1) + b"\0" + frame(raw("physical-h3"))
proof = bytes.fromhex("000102ff")
target = bytes.fromhex(gold["physical-h3"]["expected_hash"])
fin = b"GFRP" + u16(1) + b"\1" + frame(raw("descriptor")) + target + frame(proof)
publications.append(dict(id="body_03_h3", object="Body", inner_hex=raw("physical-h3").hex(), expected_hex=body.hex()))
publications.append(dict(id="proof_03_h3", object="FinalProof", descriptor_hex=raw("descriptor").hex(),
                         target_hex=target.hex(), proof_hex=proof.hex(), expected_hex=fin.hex()))
negative_publications = []
for name, b in [("body", body), ("proof", fin)]:
    for suffix, malformed in [("extra", b + b"\0"), ("truncated", b[:-1]),
                              ("version2", b[:4] + u16(2) + b[6:]),
                              ("wrong_magic", b"FAIL" + b[4:])]:
        negative_publications.append(dict(id=name + "_" + suffix, hex=malformed.hex(), expected="REJECT"))
negative_publications.extend([
    dict(id="unknown_tag", hex=(b"GFRP" + u16(1) + b"\xff").hex(), expected="REJECT"),
    dict(id="oversized_body_claim", hex=(b"GFRP" + u16(1) + b"\0" + u32(2**32 - 1)).hex(), expected="REJECT"),
    dict(id="oversized_descriptor_claim", hex=(b"GFRP" + u16(1) + b"\1" + u32(16385)).hex(), expected="REJECT"),
])

out = dict(version="viewer-gfr-independent-interface-v1", source="GFR-07-v1 sections4–5; independently encoded Python big-endian integers and hashlib; no candidate import",
           predecessor_golden=str(GOLDEN.relative_to(HERE.parents[2])),
           predecessor_golden_sha256=hashlib.sha256(GOLDEN.read_bytes()).hexdigest(),
           predecessor_limits="03 encoding-only Descriptor fixture is not a cryptographically valid registry; no certificate acceptance claim",
           publication_codec=0x47465201, positions=positions, negative_positions=negative_positions,
           comparisons=comparisons, lifecycle=lifecycle, publications=publications,
           negative_publications=negative_publications,
           bounds={"max_message_bytes":65536,"max_proof_bytes":32768,"max_descriptor_bytes":16384,
                   "max_resume_blocks":4096,"max_lifecycle_bytes":67108864})
(HERE / "interface-vectors-v1.json").write_text(json.dumps(out, indent=2) + "\n")
print(json.dumps({k: len(v) for k, v in out.items() if isinstance(v, list)}, sort_keys=True))
