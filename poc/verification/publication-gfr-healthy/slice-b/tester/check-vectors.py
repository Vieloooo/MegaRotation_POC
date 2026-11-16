#!/usr/bin/env python3
"""Recompute frozen H3-B compositions and model boundary expectations with stdlib only."""

import hashlib
import json
from pathlib import Path
import struct


SLICE = Path(__file__).resolve().parents[1]
GOLDENS = SLICE.parent / "slice-a/independent-harness/tests/fixtures/m2-golden-vectors.json"


def read(path):
    return json.loads(path.read_bytes())


def eq(actual, expected, label):
    if actual != expected:
        raise ValueError(f"{label}: got {actual!r}, expected {expected!r}")


def pack(fmt, *values):
    return struct.pack(">" + fmt, *values)


def frame(data):
    return pack("I", len(data)) + data


def digest(domain, *parts):
    return hashlib.sha256(b"".join(pack("Q", len(part)) + part for part in (domain.encode(), *parts))).digest()


def check_identity(vectors):
    v1 = pack("H", 1)
    key = v1 + b"\x33" * 32 + b"\x44" * 32
    pc = pack("HQBQ", 1, 100, 4, 5)
    common = [v1 + b"\x11" * 32, b"E", b"L", v1, v1, b"2" * 40, pc]
    head = v1 + frame(pack("BQ", 0, 5))
    append = pack("BQQB", 0, 1, 3, 0) + frame(key)
    lifecycle = v1 + frame(pc) + pack("BQQB", 1, 0, 0, 1)
    empty = head + pack("II", 0, 0)
    full = head + pack("I", 1) + frame(append) + pack("I", 1) + frame(lifecycle)
    nonempty = vectors["record_nonempty"]
    eq(append.hex(), nonempty["append_effect_hex"], "append")
    eq(lifecycle.hex(), nonempty["lifecycle_projection_hex"], "lifecycle")
    eq(len(full), nonempty["projection_bytes"], "full projection length")
    ids = {}
    for name, projection in (("record_empty", empty), ("record_nonempty", full)):
        variant = vectors[name]
        parts = [*common, projection]
        eq([part.hex() for part in parts], variant["parts_hex"], f"{name} hash parts")
        eq(projection.hex(), variant["projection_hex"], f"{name} projection")
        ids[name] = digest("H3/Record/v1", *parts).hex()
        eq(ids[name], variant["id"], f"{name} ID")

    payload = b"abc"
    run, config, source = b"\x01" * 32, b"\x02" * 32, b"E"
    ordinal, codec = pack("I", 0), pack("I", 0x47465201)
    attempt_payload = digest("H3/AttemptPayload/v1", source, pack("I", 1), ordinal, codec, payload)
    binding = digest("H3/SubmissionBinding/v1", run, config, source, b"P", pack("Q", 1), pack("Q", 2),
                     key, attempt_payload, ordinal, codec)
    bindings = {"attempt_payload_id": attempt_payload.hex(), "submission_binding": binding.hex(),
                "submission_digest": digest("H3/TransportObject/v1", b"\0", binding, payload).hex(),
                "finalized_q5_binding": digest("H3/FinalizedBlockBinding/v1", run, config, source, pack("Q", 5)).hex()}
    eq(bindings, vectors["bindings"], "binding commitments")
    eq(vectors["zero_length_rule"], "REJECT total_len0,chunk_count0,emptychunk", "final empty-object rule")
    return {**ids, **bindings}


def lengths(size, limit):
    maximum = min(61_440, limit - 83)
    if not (0 < size <= 1_048_576 and maximum > 0):
        return None
    complete, last = divmod(size, maximum)
    if complete + bool(last) > 1_024:
        return None
    return [maximum] * complete + ([last] if last else [])


def check_wire(vectors):
    row = vectors["frame_vector"]
    kind = pack("B", row["kind"])
    binding = bytes.fromhex(row["binding_id_hex"])
    payload = bytes.fromhex(row["object_hex"])
    object_hash = digest("H3/TransportObject/v1", kind, binding, payload)
    header = b"H3TF" + pack("H", 1) + kind + binding + object_hash + pack("IHHI", len(payload), 0, 1, len(payload))
    eq(object_hash.hex(), row["object_digest_hex"], "object digest")
    eq(len(header), row["header_bytes"], "frame header")
    eq((header + payload).hex(), row["frame_hex"], "exact frame bytes")
    eq(len(header + payload), row["frame_bytes"], "frame length")
    eq(lengths(len(payload), row["payload_limit"]), [len(payload)], "single chunk frame")
    eq(lengths(0, 65_536), None, "final empty-object guard")
    for case in vectors["segmentation"]:
        result = lengths(case["object_bytes"], case["payload_limit"])
        if "expected" in case:
            eq("CAPACITY" if result is None else "OK", case["expected"], "segmentation capacity")
        elif "chunks" in case:
            eq(result, case["chunks"], "segmentation lengths")
        else:
            eq(len(result), case["chunk_count"], "segmentation count")
    full = lengths(1_048_576, 65_536)
    framed = sum(83 + size for size in full)
    four_max = 4 * (83 + 61_440)
    fanout = vectors["fanout"]
    eq(framed, fanout["one_mib_framed_bytes"], "framed 1 MiB")
    eq(4 * framed, fanout["four_copies_bytes"], "four full copies")
    eq(4 * framed > fanout["default_global_queue_bytes"], True, "shared queue cannot fit four copies")
    eq(four_max <= 262_144, True, "four maximum frames fit sender byte cap")
    return {"object_digest": object_hash.hex(), "frame_hex": (header + payload).hex(),
            "maximum_object_chunks": len(full), "last_chunk_bytes": full[-1], "maximum_object_framed_bytes": framed,
            "four_maximum_frame_bytes": four_max, "four_full_copies_bytes": 4 * framed}


def check_parameters(vectors, record_vector):
    fields = ("announcement_us", "grace_us", "expiry_us")
    record = [bytes.fromhex(part) for part in record_vector["parts_hex"]]
    record_id = digest("H3/Record/v1", *record).hex()
    checks = {}
    for field, case in zip(fields, vectors["cases"], strict=True):
        base = case["ledger_parameters"]
        alternate = case["delivery_or_snapshot_parameters"]
        eq(set(base), set(fields), "ledger parameter fields")
        eq(set(alternate), set(fields), "delivery parameter fields")
        eq([key for key in fields if base[key] != alternate[key]], [field], "single perturbation")
        eq(0 < alternate[field] < base[field], True, "positive shorter configured bound")
        eq(case["config_source_profile_equal"], True, "same immutable config/source/profile")
        eq(record_id == record_vector["id"], case["record_id_equal"], "parameters outside record identity")
        decision = "REJECT_BEFORE_DEDUPE_OR_SCOPE_MUTATION" if base != alternate else "CHECK_RECORD"
        eq(decision, case["expected"], "exact binding required before equal-ID dedupe")
        eq(case["synthetic_matching_snapshot_receipt_does_not_override_parameter_mismatch"], True, "restore requires both bindings")
        checks[field] = {"ledger_us": base[field], "delivery_us": alternate[field], "shared_record_id": record_id,
                         "decision": decision, "restore_with_owner_receipt": "REJECT_PARAMETER_MISMATCH"}
    return checks


def check_scope(vectors):
    records = {row["id"]: row for row in read(GOLDENS)["records"]}
    program = bytes.fromhex(records["program-config"]["hex"])
    window_start = len(program) - 28  # Window precedes byte_limit and metadata_window_us.
    config_start = 14  # version, chain_id, rule and verifier versions.
    config = program[config_start:config_start + 32]
    anchor = b"\x11" * 32

    def cid(session, anchor_id, canonical_program):
        return digest("PoC/Regional/v1", pack("HQQ", 1, *session) + anchor_id + canonical_program[2:])

    def window(minimum, maximum):
        return program[:window_start] + pack("QQ", minimum, maximum) + program[window_start + 16:]

    closed = tuple(vectors["setup"]["ledger_closed_session"])
    opened = tuple(vectors["setup"]["legitimate_open_session"])
    eq(vectors["setup"]["same_key_fields"], ["descriptor.cid", "target canonical body digest"], "ResultKey members")
    closed_cid = cid(closed, anchor, program)
    eq(closed_cid.hex(), records["cid-anchor-11"]["expected_hash"], "03 closed-session CID golden")
    eq(cid(closed, b"\x22" * 32, program).hex(), records["cid-anchor-22"]["expected_hash"], "03 changed anchor golden")
    open_program = window(11, 20)
    open_cid = cid(opened, anchor, open_program)
    physical = records["physical-h3"]
    body = digest(physical["domain"], bytes.fromhex(physical["hex"]))
    eq(body.hex(), physical["expected_hash"], "03 target canonical body digest golden")
    supplied_key = pack("H", 1) + open_cid + body
    foreign_program = open_program[:config_start] + b"\x66" * 32 + open_program[config_start + 32:]
    changes = {
        "replace plan.session with closed(0,0), keep cid/body": (closed, anchor, open_program),
        "alter plan.window, keep cid/body": (opened, anchor, window(21, 30)),
        "alter anchor.block.block_id, keep cid/body": (opened, b"\x55" * 32, open_program),
        "replace plan.config_ref with foreignconfig, keep cid/body": (opened, anchor, foreign_program),
    }
    checks = []
    for row in vectors["cases"]:
        session, subject, submitted_program = changes[row["mutation"]]
        equal_config = config == submitted_program[config_start:config_start + 32]
        new_cid = cid(session, subject, submitted_program)
        eq(equal_config and open_cid == new_cid, False, "forged descriptor binding")
        eq(row["expected"], "REJECT_BEFORE_SCOPE_LOOKUP_OR_STOPDECISION", "scope rejection ordering")
        eq(row["legitimate_task_unchanged"], True, "unchanged legitimate key expected")
        checks.append({"mutation": row["mutation"], "submitted_key": supplied_key.hex(),
                       "recomputed_cid": new_cid.hex(), "config_matches": equal_config, "decision": row["expected"]})
    eq({row["mutation"] for row in checks}, set(changes), "all scope mutations exercised")
    eq(closed != opened and closed_cid != open_cid, True, "different closed/open session subjects")
    return {"classification": "03 canonical composition; scheduled plan and actual ledger tested in Rust",
            "open_control": {"cid": open_cid.hex(), "scope": "OPEN"},
            "closed_control": {"cid": closed_cid.hex(), "scope": "SESSION_CLOSED"}, "mutations": checks}


if __name__ == "__main__":
    identity = read(SLICE / "identity-vectors-v1.json")
    report = {"status": "PASS", "classification": "independent composition and boundary oracle; no runtime receipt",
              "identities": check_identity(identity), "wire": check_wire(read(SLICE / "wire-vectors-v1.json")),
              "parameters": check_parameters(read(SLICE / "parameter-vectors-v1.json"), identity["record_empty"]),
              "descriptor_scope": check_scope(read(SLICE / "descriptor-scope-vectors-v1.json"))}
    print(json.dumps(report, indent=2, sort_keys=True))
