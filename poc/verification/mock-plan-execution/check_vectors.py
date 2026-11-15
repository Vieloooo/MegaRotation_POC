#!/usr/bin/env python3
"""Independent small fixture oracle; never imports or invokes production code.

No generation/update mode. P profile bytes are proposals, not a protocol codec.
"""
import copy
import hashlib
import json
from pathlib import Path
import struct

HERE = Path(__file__).resolve().parent
V = json.loads((HERE / "semantic-vectors.json").read_text())
G = {r["name"]: r for r in json.loads((HERE / "golden-vectors.json").read_text())["records"]}
RESULTS = []


def check(name, condition, authority="S"):
    if not condition:
        raise AssertionError(name)
    RESULTS.append({"case": name, "authority": authority, "status": "PASS_REFERENCE_ONLY"})


def sha(data):
    return hashlib.sha256(data).digest()


def meta(values):
    return struct.pack("<QQQ", *values)


def lp(data):
    return struct.pack("<I", len(data)) + data


def seq(values):
    return struct.pack("<I", len(values)) + b"".join(lp(x) for x in values)


def state_bytes(context, accounts):
    return (b"MPXPREP/STATE/v0\0" + meta(context) + struct.pack("<I", len(accounts))
            + b"".join(struct.pack("<BQQ", *row) for row in accounts))


def match(name, data):
    expected = G[name]
    check(name + ":bytes", data.hex() == expected["hex"], "P")
    check(name + ":sha256", sha(data).hex() == expected["sha256"], "P")
    return sha(data)


def decode_tx(data):
    if len(data) != 19 or data[0] != 1:
        raise ValueError("malformed transfer")
    return struct.unpack("<BBQQ", data[1:])


def apply_step(accounts, step):
    accounts = {i: [nonce, balance] for i, nonce, balance in accounts}
    account, amount = step["credit"]
    accounts[account][1] += amount
    kept = []
    for index, raw in enumerate(step["ordinary_hex"]):
        sender, receiver, nonce, amount = decode_tx(bytes.fromhex(raw))
        # All concrete inputs use known distinct accounts and bounded arithmetic.
        # This does not decide the pending unknown/self/zero/overflow policy.
        if accounts[sender][0] != nonce or accounts[sender][1] < amount:
            continue
        accounts[sender][0] += 1
        accounts[sender][1] -= amount
        accounts[receiver][1] += amount
        kept.append(index)
    return [[i, *accounts[i]] for i in sorted(accounts)], kept


def l1_read(query):
    fixture = V["l1"]
    if query["height"] not in fixture["bounded_heights"]:
        return "FIXTURE_EXHAUSTED"
    for height in range(query["height"] + 1):
        if height in query.get("permanent_missing", []):
            return "FIXTURE_ERROR"
        if (height in query.get("missing", [])
                or query["now_us"] < fixture["finalized_at_us"][height]
                or query["now_us"] < fixture["reader_delivered_at_us"][query["reader"]][height]):
            return "WAIT"
        if height in query.get("corrupt", []):
            return "REJECT"
        if query["control"] and height in query.get("controls_missing", []):
            return "WAIT"
    return "READY"


def run():
    check("sha256-known-answer", sha(b"abc").hex() == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", "P")
    for index, case in enumerate(V["schedule"]):
        beta, r, view = case["input"]
        region, member = (beta % 3 + r % 3) % 3, (view - 1) % 5
        check(f"schedule-{index}", [region, member, 5 * region + member] == case["expected"])
    for index, (beta, r, view) in enumerate(V["schedule_invalid"]):
        check(f"schedule-invalid-{index}", not (beta >= 0 and r >= 0 and view >= 1))
    mapping = V["identity_mapping"]
    order = sorted(range(5), key=lambda member: bytes.fromhex(mapping["key_labels_by_member"][member]))
    check("sort-member-permutation", order == mapping["expected_sorted_members"])
    inverse = [order.index(member) for member in range(5)]
    check("member-to-signer-inverse", inverse == mapping["expected_signer_index_by_member"])
    for region in range(3):
        check(f"signer-pids-r{region}", [5 * region + member for member in order] == mapping["expected_signer_pid_by_region"][region])
        for member in range(5):
            check(f"mapping-round-trip-pid{5 * region + member}", order[inverse[member]] == member)
    timing = V["timing"]
    for index, case in enumerate(V["origins"]):
        actual = []
        for height in case["heights"]:
            ts = timing["genesis_ts_us"] + height * timing["slot_us"]
            scheduled = (ts - timing["dep_lag_us"]) // timing["l1_period_us"]
            actual.append(min(case["anchor_origin"] + height - case["anchor_height"], max(case["anchor_origin"], scheduled)))
        check(f"origin-{index}", actual == case["expected"])
    boundary = V["regular_boundary"]
    check("regular-window-retained", [boundary["old_window"][1] + 1, boundary["old_window"][1] + 10] == boundary["expected_window"])
    check("regular-reference-distinct", boundary["expected_reference_height"] == boundary["old_initial_anchor_height"] != boundary["expected_new_anchor_height"])
    check("regular-fallback-padding", list(range(boundary["last_accepted_physical_height"] + 1, boundary["first_physical_height"])) == boundary["expected_padding"])
    for index, case in enumerate(V["window_cases"]):
        first, last = case["window"]
        outcome = "VALID" if case["anchor"] < first <= last and last - first + 1 == 10 else "REJECT"
        check(f"window-{index}", outcome == case["expected"])
    rank = {"FORCE": 0, "RESUME": 0, "EXPIRE": 1, "SESSION_END": 2, "ESPRESSO": 3}
    ordered = sorted(V["replay_order"]["input"], key=lambda p: (p["key"], rank[p["kind"]], p["index"]))
    check("replay-order-controls-mixed", [p["label"] for p in ordered] == V["replay_order"]["expected"])
    lifecycle = V["lifecycle_events"]
    unique_events = []
    for event in lifecycle["input"]:
        identity = (lifecycle["position"], event["kind"], event["body"])
        if identity not in unique_events:
            unique_events.append(identity)
    check("lifecycle-same-position-distinct-events", len(unique_events) == lifecycle["expected_distinct_count"])
    for query in V["l1"]["queries"]:
        check(query["id"], l1_read(query) == query["expected"])

    execution = V["execution"]
    anchor = execution["anchor"]
    accounts = copy.deepcopy(anchor["accounts"])
    pre = match("state-h0", state_bytes(anchor["metadata"], accounts))
    native_parent = bytes.fromhex(anchor["native_hash"])
    physical_parent = bytes.fromhex(anchor["physical_id"])
    for step in execution["steps"]:
        height = step["metadata"][0]
        before = copy.deepcopy(accounts)
        accounts, kept_indices = apply_step(before, step)
        check(f"execution-h{height}-manual-state", accounts == step["expected_accounts"], "P")
        check(f"execution-h{height}-manual-kept", kept_indices == step["expected_kept_indices"], "P")
        check(f"execution-h{height}-repeat-same-parent", (accounts, kept_indices) == apply_step(before, step), "P")
        match(f"metadata-h{height}", meta(step["metadata"]))
        dep = [b"\x03" + meta(step["metadata"]), struct.pack("<BBQ", 2, *step["credit"])]
        for index, value in enumerate(dep):
            match(f"dep-h{height}-{index}", value)
        txs = [bytes.fromhex(value) for value in step["ordinary_hex"]]
        kept = [txs[index] for index in kept_indices]
        post = match(f"state-h{height}", state_bytes(step["metadata"], accounts))
        native_parts = [b"MPXPREP/NATIVE/v0\0", native_parent, meta(step["metadata"]), seq(dep), seq(kept), post]
        native = b"".join(native_parts)
        native_hash = match(f"native-h{height}", native)
        if height in execution["physical_heights"]:
            block = b"MPXPREP/PHYSICAL/v0\0" + physical_parent + meta(step["metadata"]) + pre + seq(txs)
            physical_hash = match(f"physical-h{height}", block)
            alternate_block = b"MPXPREP/PHYSICAL/v0\0" + physical_parent + meta(step["metadata"]) + pre + seq(kept)
            check("filtered-raw-affects-physical", physical_hash != match("physical-h3-only-kept", alternate_block), "P")
            alternative = copy.deepcopy(step)
            alternative["ordinary_hex"] = [value.hex() for value in kept]
            alternative_accounts, alternative_kept = apply_step(before, alternative)
            check("filtered-raw-preserves-native", alternative_accounts == accounts and alternative_kept == list(range(len(kept))), "P")
            check("physical-pre-root-after-padding", pre.hex() == G["state-h2"]["sha256"] and pre != post and pre.hex() != G["state-h0"]["sha256"], "P")
            check("physical-parent-distinct-native-parent", physical_parent != native_parent, "P")
            for index, label in enumerate(["native-parent", "metadata", "DEP", "Kept", "post-root"], start=1):
                mutation = native_parts.copy()
                changed = bytearray(mutation[index]); changed[-1] ^= 1
                mutation[index] = bytes(changed)
                check("native-binding-" + label, sha(b"".join(mutation)) != native_hash, "P")
            for index, value in enumerate(txs):
                match(f"ordinary-{index}", value)
        native_parent, pre = native_hash, post
    expanded_heights = list(range(anchor["metadata"][0] + 1, execution["physical_heights"][-1] + 1))
    check("merge-gap-expansion", expanded_heights == execution["expected_virtual_heights"])
    check("execution-covers-expanded-heights", [step["metadata"][0] for step in execution["steps"]] == expanded_heights)
    check("anchor-only-no-padding", list(range(anchor["metadata"][0] + 1, anchor["metadata"][0] + 1)) == execution["expected_anchor_only_virtual_heights"])
    check("merge-stops-at-physical-tip", execution["steps"][-1]["metadata"][0] == execution["expected_no_speculative_padding_after"])
    for index, value in enumerate(execution["malformed_ordinary_hex"]):
        try:
            decode_tx(bytes.fromhex(value))
        except ValueError:
            check(f"malformed-tx-{index}", True, "P")
        else:
            check(f"malformed-tx-{index}", False, "P")
    cid_inputs = [("cid-anchor-11", "11", 0, b"proposal-config-v0"), ("cid-anchor-22", "22", 0, b"proposal-config-v0"), ("cid-session-changed", "11", 1, b"proposal-config-v0"), ("cid-config-changed", "11", 0, b"proposal-config-v1")]
    cids = [match(name, b"REGIONAL\0" + struct.pack("<QQ", 0, r) + bytes.fromhex(anchor_byte * 32) + lp(config)) for name, anchor_byte, r, config in cid_inputs]
    check("cid-anchor-session-config-separation", len(set(cids)) == 4, "P")
    for name, golden in G.items():
        check(name + ":literal-integrity", sha(bytes.fromhex(golden["hex"])).hex() == golden["sha256"], "P")
    return {"scope": "PREPARATION_ONLY", "status": "PASS_REFERENCE_VECTORS", "implementation_status": "INCONCLUSIVE_NOT_RUN", "profile": V["execution"]["profile"], "seed": None, "fixed_golden_record_count": len(G), "case_count": len(RESULTS), "cases": RESULTS, "not_run": ["production implementation", "actual BLS/PoP/native signed subjects", "public registry/config/Plan codec", "native engine/runtime/recovery", "Anvil/ABI/ResumeCut", "candidate verifier/tester matrix", *execution["pending_arithmetic"]]}


if __name__ == "__main__":
    print(json.dumps(run(), indent=2, sort_keys=True))
