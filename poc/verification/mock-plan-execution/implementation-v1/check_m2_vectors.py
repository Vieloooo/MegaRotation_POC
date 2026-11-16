#!/usr/bin/env python3
"""Independent frozen BE codec/execution oracle; does not import production."""
import copy
import hashlib
import json
from pathlib import Path
import struct

HERE = Path(__file__).resolve().parent
I = json.loads((HERE / "m2-inputs.json").read_text())
G = {r["id"]: r for r in json.loads((HERE / "m2-golden-vectors.json").read_text())["records"]}
E = json.loads((HERE / "execution-boundaries.json").read_text())
CHECKS, SEEN = [], set()
MAX = (1 << 64) - 1
U8 = lambda x: struct.pack("B", x)
U16 = lambda x: struct.pack(">H", x)
U32 = lambda x: struct.pack(">I", x)
U64 = lambda x: struct.pack(">Q", x)
RAW = lambda x: bytes.fromhex(x)
META = lambda m: struct.pack(">QQQ", *m)
TRANSFER = lambda t: b"\x01" + struct.pack(">BBQQ", *t)
CREDIT = lambda c: b"\x02" + struct.pack(">BQ", *c)
LIST = lambda xs: U32(len(xs)) + b"".join(U32(len(x)) + x for x in xs)


def check(name, ok):
    if not ok:
        raise AssertionError(name)
    CHECKS.append(name)


def framed_hash(domain, data):
    domain = domain.encode("ascii")
    return hashlib.sha256(U64(len(domain)) + domain + U64(len(data)) + data).digest()


def record(name, raw, version=True):
    data = (U16(1) if version else b"") + raw
    expected = G[name]
    check(name + ":bytes", data.hex() == expected["hex"])
    check(name + ":encoding-sha256", hashlib.sha256(data).hexdigest() == expected["encoding_sha256"])
    SEEN.add(name)
    if "domain" in expected:
        result = framed_hash(expected["domain"], data)
        check(name + ":domain-hash", result.hex() == expected["expected_hash"])
        return result
    return data


def state_raw(metadata, accounts):
    return META(metadata) + U32(len(accounts)) + b"".join(struct.pack(">BQQ", *row) for row in accounts)


def apply(accounts, credits, raw_txs, byte_limit=65536):
    # Reject malformed ordinary input before returning any mutation.
    if len(raw_txs) > 1024 or len(credits) > 1024 or sum(map(len, raw_txs)) > min(byte_limit, 65536):
        raise ValueError("bounds")
    txs = []
    for raw in raw_txs:
        if len(raw) != 19 or raw[0] != 1:
            raise ValueError("malformed ordinary")
        txs.append(struct.unpack(">BBQQ", raw[1:]))
    state = {a: [n, b] for a, n, b in accounts}
    for account, amount in credits:
        nonce, balance = state.get(account, [0, 0])
        if balance > MAX - amount:
            raise ValueError("credit overflow")
        if nonce or balance + amount:
            state[account] = [nonce, balance + amount]
    kept = []
    for index, (sender, receiver, nonce, amount) in enumerate(txs):
        sender_nonce, sender_balance = state.get(sender, [0, 0])
        receiver_nonce, receiver_balance = state.get(receiver, [0, 0])
        if nonce != sender_nonce or sender_nonce == MAX or sender_balance < amount:
            continue
        if sender != receiver and receiver_balance > MAX - amount:
            continue
        if sender == receiver:
            state[sender] = [sender_nonce + 1, sender_balance]
        else:
            state[sender] = [sender_nonce + 1, sender_balance - amount]
            if receiver_nonce or receiver_balance + amount:
                state[receiver] = [receiver_nonce, receiver_balance + amount]
        kept.append(index)
    return [[a, *state[a]] for a in sorted(state)], kept


def run():
    anchor = I["anchor"]
    genesis = RAW(anchor["physical_id"])
    native_parent = RAW(anchor["native_hash"])
    pre = record("state-h0", state_raw(anchor["metadata"], anchor["accounts"]))
    accounts = copy.deepcopy(anchor["accounts"])
    for step in I["steps"]:
        height = step["metadata"][0]
        txs = [TRANSFER(t) for t in step["transactions"]]
        dep = [b"\x03" + META(step["metadata"])] + [CREDIT(c) for c in step["credits"]]
        after, kept_indices = apply(accounts, step["credits"], txs)
        check(f"h{height}:manual-state", after == step["expected_accounts"])
        check(f"h{height}:manual-kept", kept_indices == step["expected_kept_indices"])
        check(f"h{height}:repeat-parent", (after, kept_indices) == apply(accounts, step["credits"], txs))
        record(f"metadata-h{height}", META(step["metadata"]))
        for n, raw in enumerate(dep):
            record(f"dep-h{height}-{n}", raw, False)
        for n, raw in enumerate(txs):
            record(f"transfer-{n}", raw, False)
        post = record(f"state-h{height}", state_raw(step["metadata"], after))
        kept = [txs[n] for n in kept_indices]
        native_parent = record(f"native-h{height}", native_parent + META(step["metadata"]) + LIST(dep) + LIST(kept) + post)
        if height == 3:
            physical = record("physical-h3", genesis + META(step["metadata"]) + pre + LIST(txs))
            filtered = record("physical-h3-only-kept", genesis + META(step["metadata"]) + pre + LIST(kept))
            check("raw-filtering-changes-physical", physical != filtered)
            check("raw-filtering-preserves-execution", apply(accounts, step["credits"], kept)[0] == after)
            check("pre-root-is-h2-not-h3", pre.hex() == G["state-h2"]["expected_hash"] and pre != post)
        accounts, pre = after, post
    record("derived-anchor-h3", U64(3) + native_parent)
    rows = I["dep_fixture"]["credits"]
    dep_hash = record("dep-fixture", U64(I["dep_fixture"]["first_height"]) + U32(len(rows)) + b"".join(U32(len(row)) + b"".join(CREDIT(c) for c in row) for row in rows))
    control = record("empty-control", U32(0))
    l1 = U32(len(I["l1_rows"]))
    for row in I["l1_rows"]:
        check(f"L1-{row['height']}:control", control.hex() == row["empty_control_commitment"])
        l1 += U64(row["height"]) + RAW(row["hash"]) + RAW(row["parent_hash"]) + U64(row["timestamp_s"]) + U64(row["finalized_at_us"]) + control
    l1_hash = record("l1-fixture", l1)
    config = I["public_config"]
    check("config-dep-binding", dep_hash.hex() == config["dep_fixture_hash"])
    check("config-L1-binding", l1_hash.hex() == config["l1_fixture_hash"])
    genesis_anchor = U64(0) + U64(3) + genesis + U64(10000000)
    config_raw = U64(config["chain_id"]) + U16(config["rule_version"]) + U16(config["verifier_version"]) + RAW(config["registry_hash"]) + dep_hash + l1_hash + genesis_anchor + RAW(config["genesis_state_root"]) + RAW(config["genesis_native_hash"])
    config_raw += b"".join(U64(config[k]) for k in ["l2_slot_us", "l1_genesis_timestamp_us", "l1_block_us", "t_dep_us", "t_drift_us", "metadata_window_us"]) + U32(config["byte_limit"])
    config_ref = record("public-config", config_raw)
    plan = I["plan"]
    committee = U32(len(plan["committee"])) + b"".join(U8(c["pid"]) + RAW(c["identity_key"]) + RAW(c["vote_key"]) + U64(c["weight"]) for c in plan["committee"])
    eligible = U32(len(plan["eligible_proposers"])) + bytes(plan["eligible_proposers"])
    rule = U16(1) + U16(1) + U32(0)
    session, window = struct.pack(">QQ", *plan["input"]["session"]), struct.pack(">QQ", *plan["input"]["window"])
    ref = U64(0) + U64(3) + genesis
    plan_input = U8(plan["input"]["cause"]) + session + b"\0" + window + ref
    record("plan-input", plan_input)
    record("boundary-with-anchor", plan_input + b"\1" + genesis_anchor)
    record("boundary-no-anchor", plan_input + b"\0")
    auth = record("authorization-preimage", config_ref + RAW(config["registry_hash"]) + plan_input)
    plan_raw = session + b"\0\0\0" + committee + U64(5) + U64(1) + eligible + rule + window + ref + config_ref + auth
    record("plan-record", plan_raw)
    program = U64(42) + U16(1) + U16(1) + config_ref + committee + U64(1) + eligible + rule + window + U32(65536) + U64(200000)
    record("program-config", program)
    cid = record("cid-anchor-11", session + genesis + program)
    other_cid = record("cid-anchor-22", session + b"\x22" * 32 + program)
    check("equal-height-different-anchor-cid", cid != other_cid)
    record("descriptor", plan_raw + genesis_anchor + cid)
    record("execution-anchor", genesis_anchor + state_raw(anchor["metadata"], anchor["accounts"]) + RAW(anchor["native_hash"]))
    for case in E["cases"]:
        before = copy.deepcopy(case["accounts"])
        try:
            actual, kept = apply(case["accounts"], case["credits"], [TRANSFER(t) for t in case["txs"]] + [RAW(x) for x in case.get("extra_raw_hex", [])], case.get("byte_limit", 65536))
        except ValueError:
            check(case["id"] + ":reject", case["expected_status"] == "REJECT")
        else:
            check(case["id"] + ":ready", case["expected_status"] == "READY")
            check(case["id"] + ":accounts", actual == case["expected_accounts"])
            check(case["id"] + ":kept", kept == case["kept"])
        check(case["id"] + ":parent-unchanged", before == case["accounts"])
    check("all-goldens-exercised", SEEN == set(G))
    return {"scope": "M2_REFERENCE_BEFORE_IMPLEMENTATION", "status": "PASS", "production_status": "NOT_RUN", "seed": None, "golden_records": len(G), "execution_boundary_cases": len(E["cases"]), "assertion_count": len(CHECKS), "checks": CHECKS, "not_yet_executable": ["real15-key registry/PoP", "DEP shape negative API cases", "full codec malformed/bounds API cases", "L1 explicit Finalize/Deliver state machine", "actual production API"]}


if __name__ == "__main__":
    print(json.dumps(run(), indent=2, sort_keys=True))
