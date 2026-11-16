#!/usr/bin/env python3
"""Independent reference checks for frozen M1 data; no production imports."""
import hashlib
import json
from pathlib import Path
import struct

HERE = Path(__file__).resolve().parent
V = json.loads((HERE / "m1-vectors.json").read_text())
H = json.loads((HERE / "i1-hash-compatibility.json").read_text())
LIMIT = {"Pid": 15, "RegionId": 3, "MemberId": 5, "SignerIndex": 5}
LENGTH = {**{kind: 3 for kind in LIMIT}, "Hash32": 34, "Cid": 34,
          "ConfigRef": 34, "L1Position": 26, "L1BlockRef": 42, "ResultKey": 66}
CHECKS = []


def require(label, value):
    if not value:
        raise AssertionError(label)
    CHECKS.append(label)


def encode(kind, value):
    if kind in LIMIT:
        raw = struct.pack("B", value)
    elif kind in ("Hash32", "Cid", "ConfigRef"):
        raw = bytes.fromhex(value)
    elif kind == "L1Position":
        raw = struct.pack(">QQQ", value["block"], value["tx"], value["log"])
    elif kind == "L1BlockRef":
        raw = struct.pack(">Q", value["height"]) + bytes.fromhex(value["hash"])
    else:
        raw = bytes.fromhex(value["cid"] + value["block_hash"])
    return struct.pack(">H", 1) + raw


def decode(kind, raw):
    if len(raw) != LENGTH[kind]:
        raise ValueError("InvalidLength")
    if struct.unpack(">H", raw[:2])[0] != 1:
        raise ValueError("UnsupportedVersion")
    body = raw[2:]
    if kind in LIMIT:
        if body[0] >= LIMIT[kind]:
            raise ValueError("Invalid" + kind.replace("Id", ""))
        return body[0]
    if kind in ("Hash32", "Cid", "ConfigRef"):
        return body.hex()
    if kind == "L1Position":
        return dict(zip(("block", "tx", "log"), struct.unpack(">QQQ", body)))
    if kind == "L1BlockRef":
        return {"height": struct.unpack(">Q", body[:8])[0], "hash": body[8:].hex()}
    return {"cid": body[:32].hex(), "block_hash": body[32:].hex()}


def run():
    for case in V["objects"]:
        expected = bytes.fromhex(case["expected_hex"])
        require(case["id"] + ":encode", encode(case["type"], case["value"]) == expected)
        require(case["id"] + ":decode", decode(case["type"], expected) == case["value"])
        require(case["id"] + ":fingerprint", hashlib.sha256(expected).hexdigest() == case["encoding_sha256"])
    for case in V["invalid_encodings"]:
        try:
            decode(case["type"], bytes.fromhex(case["encoded_hex"]))
        except ValueError as error:
            require(case["id"], str(error) == case["expected_error_class"])
        else:
            require(case["id"], False)
    for item in V["identity_mapping"]:
        require(f"pid-map-{item['pid']}", (item["region"], item["member"]) == divmod(item["pid"], 5)
                and 5 * item["region"] + item["member"] == item["pid"])
    positions = V["l1_position_input"]
    actual = sorted(range(len(positions)), key=lambda i: tuple(positions[i][key] for key in ("block", "tx", "log")))
    require("l1-position-lexicographic", actual == V["l1_position_expected_order_indices"])
    for case in H["cases"]:
        segments = [case["domain"].encode("utf-8"), *[bytes.fromhex(x) for x in case["parts_hex"]]]
        data = b"".join(struct.pack(">Q", len(segment)) + segment for segment in segments)
        require(case["id"] + ":framing", data.hex() == case["framed_hex"])
        require(case["id"] + ":hash", hashlib.sha256(data).hexdigest() == case["expected_hash"])
    hashes = {case["id"]: case["expected_hash"] for case in H["cases"]}
    require("segmentation-binding", hashes["segments-a"] != hashes["segments-b"])
    require("empty-component-binding", hashes["empty"] != hashes["empty-component"])
    return {"scope": "M1_REFERENCE_BEFORE_IMPLEMENTATION", "status": "PASS",
            "implementation_status": "NOT_RUN", "seed": None,
            "objects": len(V["objects"]), "negative_encodings": len(V["invalid_encodings"]),
            "i1_compatibility_hashes": len(H["cases"]), "assertion_count": len(CHECKS), "checks": CHECKS}


if __name__ == "__main__":
    print(json.dumps(run(), indent=2, sort_keys=True))
