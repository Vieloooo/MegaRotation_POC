#!/usr/bin/env python3
"""Bounded source oracle, never a production decoder or verified L1 source.

The hand-stated outcomes in vectors.json precede candidate implementation.
03 byte encoding below is independently transcribed from its frozen contract.
No candidate code, runtime, network, or third-party library is used.
"""

import argparse
import copy
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
MAX_U64 = (1 << 64) - 1


def be(value, width):
    if not 0 <= value < (1 << (8 * width)):
        raise ValueError("REJECT_OVERFLOW")
    return value.to_bytes(width, "big")


def add(left, right):
    value = left + right
    if value > MAX_U64:
        raise ValueError("REJECT_OVERFLOW")
    return value


def metadata_bytes(metadata):
    return b"".join(be(value, 8) for value in metadata)


def byte_vector(values):
    return be(len(values), 4) + b"".join(be(len(raw), 4) + raw for raw in values)


def digest(domain, raw):
    name = domain.encode("ascii")
    return hashlib.sha256(be(len(name), 8) + name + be(len(raw), 8) + raw).hexdigest()


def state_bytes(metadata, accounts):
    rows = sorted((int(key), row) for key, row in accounts.items()
                  if row["nonce"] or row["balance"])
    return (be(1, 2) + metadata_bytes(metadata) + be(len(rows), 4)
            + b"".join(be(key, 1) + be(row["nonce"], 8) + be(row["balance"], 8)
                       for key, row in rows))


def account(accounts, key):
    return copy.deepcopy(accounts.get(str(key), {"nonce": 0, "balance": 0}))


def put(accounts, key, row):
    if row["nonce"] or row["balance"]:
        accounts[str(key)] = row
    else:
        accounts.pop(str(key), None)


def execute(parent_metadata, parent_accounts, parent_hash, metadata, credits, transfers):
    """The 03 nonce/balance semantics, with all expected inputs explicit."""
    pre_bytes = state_bytes(parent_metadata, parent_accounts)
    accounts = copy.deepcopy(parent_accounts)
    dep = [b"\x03" + metadata_bytes(metadata)]
    for recipient, amount in credits:
        row = account(accounts, recipient)
        row["balance"] = add(row["balance"], amount)
        put(accounts, recipient, row)
        dep.append(b"\x02" + be(recipient, 1) + be(amount, 8))
    kept = []
    kept_indices = []
    for index, (sender, recipient, nonce, amount) in enumerate(transfers):
        raw = b"\x01" + be(sender, 1) + be(recipient, 1) + be(nonce, 8) + be(amount, 8)
        src, dst = account(accounts, sender), account(accounts, recipient)
        if src["nonce"] != nonce or src["balance"] < amount or nonce == MAX_U64:
            continue
        if sender != recipient and dst["balance"] + amount > MAX_U64:
            continue
        src["nonce"] += 1
        if sender != recipient:
            src["balance"] -= amount
            dst["balance"] += amount
            put(accounts, recipient, dst)
        put(accounts, sender, src)
        kept.append(raw)
        kept_indices.append(index)
    post_bytes = state_bytes(metadata, accounts)
    post_root = digest("PoC/ExecutionState/v1", post_bytes)
    native = (be(1, 2) + bytes.fromhex(parent_hash) + metadata_bytes(metadata)
              + byte_vector(dep) + byte_vector(kept) + bytes.fromhex(post_root))
    native_hash = digest("PoC/NativeBlock/v1", native)
    anchor = be(1, 2) + be(metadata[0], 8) + bytes.fromhex(native_hash)
    return {
        "metadata": list(metadata), "accounts": accounts,
        "pre_state_bytes": pre_bytes.hex(),
        "pre_state_root": digest("PoC/ExecutionState/v1", pre_bytes),
        "dep": [raw.hex() for raw in dep], "kept": [raw.hex() for raw in kept],
        "kept_indices": kept_indices, "post_state_bytes": post_bytes.hex(),
        "post_state_root": post_root, "native_parent": parent_hash,
        "native_bytes": native.hex(), "native_hash": native_hash,
        "derived_anchor_id": digest("PoC/DerivedAnchor/v1", anchor),
    }


class FixtureMachine:
    def __init__(self, vectors, case):
        self.vectors, self.case = vectors, case
        self.profile = copy.deepcopy(vectors["profiles"][case["profile"]])
        self.headers = {int(k): v for k, v in self.profile.get("headers", {}).items()}
        schedule = self.profile.get("header_schedule")
        if schedule:
            # Explicit synthetic rule fixture only; never emitted as verified L1.
            self.headers.update({height: schedule["genesis_us"] + height * schedule["slot_us"]
                                 for height in range(schedule["first"], schedule["last"] + 1)})
        self.metadata = list(self.profile["genesis"])
        self.accounts = copy.deepcopy(vectors["accounts"])
        self.native_hash = vectors["genesis_native_hash"]
        self.epoch_has_output = False
        self.blocks, self.cuts = [], []
        self.cursor = self.metadata[0]
        self.complete_through = None
        self.blocked_at = None
        self.reset_count = 0
        self.status = "READY"
        self.action_summaries = []

    def emit(self, origin, reason):
        height = add(self.metadata[0], 1)
        timestamp = add(self.metadata[1], self.profile["slot_us"])
        if origin not in self.headers:
            raise ValueError("WAIT_SELECTED_ORIGIN")
        if timestamp < self.headers[origin]:
            raise ValueError("REJECT_TIME_BEFORE_ORIGIN")
        if height == self.case.get("pending_dep"):
            raise ValueError("WAIT_DEP")
        if height > self.case.get("dep_last", 4096):
            raise ValueError("REJECT_DEP_EXHAUSTED")
        block = execute(self.metadata, self.accounts, self.native_hash,
                        [height, timestamp, origin],
                        self.vectors["credits"].get(str(height), []), [])
        block["reason"] = reason
        self.blocks.append(block)
        self.metadata, self.accounts = block["metadata"], block["accounts"]
        self.native_hash = block["native_hash"]
        self.epoch_has_output = True

    def fallback(self):
        for _ in range(4096):
            epoch = self.metadata[2]
            boundary = add(epoch, self.profile["window"])
            if self.complete_through is None or self.complete_through < boundary:
                return "WAIT_WINDOW"
            next_origin = add(epoch, 1)
            if next_origin not in self.headers:
                return "WAIT_NEXT_ORIGIN"
            timestamp = add(self.metadata[1], self.profile["slot_us"])
            origin = epoch if (timestamp < self.headers[next_origin]
                               or not self.epoch_has_output) else next_origin
            self.emit(origin, "fallback")
        raise ValueError("REFERENCE_BOUND_EXCEEDED")

    def override(self, action):
        previous_cut = next((cut for cut in self.cuts if cut["resume"] == action["resume"]), None)
        if previous_cut:
            if previous_cut["set"] != action["set"] or previous_cut["target"] != action["target"]:
                raise ValueError("REJECT_CONFLICTING_MARKER")
            return "READY"
        if action["target"] <= self.cursor:
            raise ValueError("REJECT_NONADVANCING_CURSOR")
        if action["set"][:2] != action["resume"][:2] or action["set"][2] >= action["resume"][2]:
            raise ValueError("REJECT_POSITION_PAIR")
        staged = copy.deepcopy(self)
        while staged.metadata[0] < action["target"]:
            epoch = staged.metadata[2]
            timestamp = add(staged.metadata[1], staged.profile["slot_us"])
            origin = epoch
            if epoch < MAX_U64 and epoch + 1 <= action["set"][0]:
                if epoch + 1 not in staged.headers:
                    raise ValueError("WAIT_NEXT_ORIGIN")
                if staged.headers[epoch + 1] <= timestamp:
                    origin += 1
            staged.emit(origin, "override")
        target_block = next(block for block in staged.blocks if block["metadata"][0] == action["target"])
        staged.cuts.append({
            "set": action["set"], "resume": action["resume"], "target": action["target"],
            "prefix_head": staged.metadata[0], "prefix_native_hashes": [b["native_hash"] for b in staged.blocks],
            "anchor_metadata": target_block["metadata"],
            "anchor_state_bytes": target_block["post_state_bytes"],
            "anchor_state_root": target_block["post_state_root"],
            "anchor_native_hash": target_block["native_hash"],
            "anchor_id": target_block["derived_anchor_id"],
        })
        staged.cursor = action["target"]
        staged.reset_count += 1
        self.__dict__.update(staged.__dict__)
        return "READY"

    def summary(self):
        return {
            "head": self.metadata[0], "metadata": [b["metadata"] for b in self.blocks],
            "origins": [b["metadata"][2] for b in self.blocks],
            "epoch_has_output": self.epoch_has_output, "status": self.status,
            "blocked_at": self.blocked_at,
            "cuts": [{k: c[k] for k in ("resume", "target", "prefix_head", "anchor_metadata")} for c in self.cuts],
        }

    def run(self):
        for action in self.case["actions"]:
            if self.blocked_at is not None:
                break
            try:
                if action["kind"] == "accepted":
                    self.blocked_at = action["position"]
                    self.status = "WAIT_FEATURE11"
                elif action["kind"] == "end_block":
                    self.complete_through = action["complete_through"]
                    self.status = self.fallback()
                elif action["kind"] == "override":
                    self.status = self.override(action)
                else:
                    raise AssertionError("unknown explicit fixture action")
            except ValueError as exc:
                self.status = str(exc)
                self.action_summaries.append(self.summary())
                break
            self.action_summaries.append(self.summary())
        actual = self.summary()
        for key, expected in self.case["expected"].items():
            assert actual[key] == expected, (self.case["id"], key, actual[key], expected)
        return {"id": self.case["id"], "reference_result": "PASS_HAND_STATED_EXPECTATIONS",
                "candidate_status": "NOT_RUN", "summary": actual, "actions": self.action_summaries,
                "blocks": self.blocks, "cuts": self.cuts, "reset_count": self.reset_count}


def calculate(vectors):
    cases = [FixtureMachine(vectors, case).run() for case in vectors["cases"]]
    stage_b = vectors["stage_b_rule_vectors"]
    for case in stage_b["lateness"]:
        assert (add(case["origin"], case["window"]) < case["inclusion"]) == case["late"]
    for case in stage_b["origin_choices"]:
        e, t, next_time = case["parent_origin"], case["next_timestamp_us"], case["next_origin_timestamp_us"]
        assert (e if t < next_time or not case["epoch_has_output"] else e + 1) == case["fallback"]
        assert (e if t < next_time else e + 1) == case["substitution"]
        assert (e + 1 if e + 1 <= case["including_block"] and next_time <= t else e) == case["override"]
    transfer = stage_b["actual_parent_transfer_filter"]
    executed = execute(vectors["profiles"]["poc"]["genesis"], vectors["accounts"],
                       vectors["genesis_native_hash"], transfer["metadata"],
                       vectors["credits"]["1"], transfer["transactions"])
    assert executed["kept_indices"] == transfer["expected_kept_indices"]
    assert executed["accounts"] == transfer["expected_accounts"]
    return {
        "schema": "derivation-independent-results-v1", "seed": "none (explicit fixtures)",
        "scope": "bounded source/contract oracle, no candidate execution or real receipt validation",
        "vector_file_sha256": hashlib.sha256((HERE / "vectors.json").read_bytes()).hexdigest(),
        "cases": cases,
        "stage_b_reference_only": {"integration": "PENDING_FEATURE11", "lateness_predicates": "PASS",
                                   "three_origin_rule_predicates": "PASS", "actual_parent_transfer_filter": executed},
    }


def main():
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    vectors = json.loads((HERE / "vectors.json").read_text())
    result = calculate(vectors)
    encoded = json.dumps(result, sort_keys=True, indent=2) + "\n"
    destination = HERE / "reference-results.json"
    if args.write:
        destination.write_text(encoded)
    else:
        assert destination.read_text() == encoded, "reference output changed; review before regeneration"
    print(json.dumps({"status": "PASS", "semantic_cases": len(result["cases"]),
                      "candidate_test_status": "NOT_RUN", "stage_b_integration": "PENDING_FEATURE11",
                      "result_sha256": hashlib.sha256(encoded.encode()).hexdigest()}, sort_keys=True))


if __name__ == "__main__":
    main()
