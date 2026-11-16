#!/usr/bin/env python3
"""Recheck public registry framing/order and the frozen 4096-height boundary."""
import hashlib
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parent
U32 = lambda n: struct.pack(">I", n)
U64 = lambda n: struct.pack(">Q", n)


def hash_parts(domain, data):
    domain = domain.encode("ascii")
    return hashlib.sha256(U64(len(domain)) + domain + U64(len(data)) + data).digest()


def run():
    public = json.loads((ROOT / "public-registry.json").read_text())
    golden = json.loads((ROOT / "registry-golden.json").read_text())
    assert hashlib.sha256((ROOT / "public-registry.json").read_bytes()).hexdigest() == golden["public_fixture_sha256"]
    raw = b"\x00\x01" + U32(15)
    assert [entry["pid"] for entry in public["entries"]] == list(range(15))
    for entry in public["entries"]:
        raw += bytes([entry["pid"]])
        for field, length in [("identity_key", 32), ("vote_key", 96), ("pop", 48)]:
            value = bytes.fromhex(entry[field])
            assert len(value) == length
            raw += value
    registry_hash = hash_parts("PoC/Registry/v1", raw).hex()
    assert raw.hex() == golden["registry_canonical_hex"]
    assert registry_hash == golden["registry_hash"]
    by_region, by_pid = [], [None] * 15
    for region in range(3):
        entries = sorted(public["entries"][region*5:region*5+5], key=lambda entry: bytes.fromhex(entry["identity_key"]))
        by_region.append([entry["pid"] for entry in entries])
        for index, entry in enumerate(entries):
            by_pid[entry["pid"]] = index
    assert by_region == golden["sorted_signer_pids_by_region"]
    assert by_pid == golden["signer_index_by_pid"]

    boundary = json.loads((ROOT / "merge-boundary-golden.json").read_text())
    def metadata(height):
        return struct.pack(">QQQ", height, 10000000 + height*1000000, max(3, (5+height)//2))
    def state(height):
        return b"\0\1" + metadata(height) + U32(2) + struct.pack(">BQQBQQ", 0, 0, 100, 2, 0, 7)
    parent = bytes([0xaa]) * 32
    for height in range(1, 4097):
        post = hash_parts("PoC/ExecutionState/v1", state(height))
        attr = b"\3" + metadata(height)
        native = b"\0\1" + parent + metadata(height) + U32(1) + U32(len(attr)) + attr + U32(0) + post
        last_parent = parent
        parent = hash_parts("PoC/NativeBlock/v1", native)
    pre = hash_parts("PoC/ExecutionState/v1", state(4095))
    physical = b"\0\1" + b"\x11"*32 + metadata(4096) + pre + U32(0)
    assert pre.hex() == boundary["pre_state_root"]
    assert post.hex() == boundary["post_state_root"]
    assert last_parent.hex() == boundary["native_parent"]
    assert parent.hex() == boundary["native_hash"]
    assert hash_parts("PoC/PhysicalBlock/v1", physical).hex() == boundary["physical_hash"]
    return {"status": "PASS_REFERENCE_ONLY", "registry_hash": registry_hash,
            "sorted_signer_pids": by_region, "expanded_heights": 4096,
            "native_hash": parent.hex(), "seed": None,
            "limits": "No production imports; cryptographic PoP validity is checked by the separate actual Rust primitive/registry tests"}


if __name__ == "__main__":
    print(json.dumps(run(), indent=2))
