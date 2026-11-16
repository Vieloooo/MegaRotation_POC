#!/usr/bin/env python3
"""Independent frozen-recipe check; never imports or executes candidate code."""
import hashlib
import json
from pathlib import Path
import struct

root = Path(__file__).resolve().parent
data = json.loads((root / "wire-vectors-v1.json").read_text())
vectors = {v["id"]: v for v in data["vectors"]}
inputs = data["inputs"]
descriptor = bytes.fromhex(inputs["descriptor_hex"])
block = bytes.fromhex(inputs["target_block_hex"])
proof = b"H3-SLICE-A-PROOF-1"
source = bytes.fromhex(inputs["genesis_source_ref_hex"])
frame = lambda b: struct.pack(">I", len(b)) + b
checks = 0


def equals(actual, expected):
    global checks
    assert actual == expected, (actual, expected)
    checks += 1


def store(proof_bytes, chain, objects):
    return (b"H3RS\x00\x01" + frame(descriptor) + frame(block) + frame(proof_bytes)
            + struct.pack(">I", len(chain)) + b"".join(frame(b) for b in chain)
            + b"\x00" + frame(source) + struct.pack(">I", len(objects))
            + b"".join(frame(b) for b in objects))


h = hashlib.sha256()
for piece in (b"PoC/PhysicalBlock/v1", block):
    h.update(struct.pack(">Q", len(piece)))
    h.update(piece)
equals(h.hexdigest(), inputs["target_hash_hex"])
equals(len(descriptor), 959)
equals(len(block), 94)
equals((b"\x00\x01" + descriptor[-32:] + h.digest()).hex(), vectors["SW01-key"]["expected_hex"])
partial = store(b"", [], [])
equals(partial.hex(), vectors["SW02-partial-store"]["expected_hex"])
equals(len(partial), 1118)
equals(store(proof, [block], [b"\x00\xff"]).hex(), vectors["SW03-structural-chain-store"]["expected_hex"])
equals((b"GFRP\x00\x01\x00" + frame(block)).hex(), vectors["SW04-public-body"]["expected_hex"])
equals((b"GFRP\x00\x01\x01" + frame(descriptor) + h.digest() + frame(proof)).hex(),
       vectors["SW05-public-proof"]["expected_hex"])
equals(bytes.fromhex(vectors["SW06-unknown-private-version"]["hex"])[4:6], b"\x00\x02")
equals(bytes.fromhex(vectors["SW07-truncated-private"]["hex"]), partial[:-1])
equals(bytes.fromhex(vectors["SW08-trailing-private"]["hex"]), partial + b"\x00")
equals(bytes.fromhex(vectors["SW09-oversize-descriptor-declaration"]["hex"]),
       b"H3RS\x00\x01" + struct.pack(">I", 16_385))
equals(bytes.fromhex(vectors["SW10-oversize-chain-count"]["hex"])[1071:1075], struct.pack(">I", 11))
expected_bounds = {
    "physical_body_N4": 94 + 4 * 23,
    "gfr_body_N4": 7 + 4 + 94 + 4 * 23,
    "gfr_final_proof_max": 7 + 4 + 16_384 + 32 + 4 + 32_768,
    "attempt_max": 47 + 256 + 10 * (8 + 197) + 8 + 49_199,
    "private_container_max": 6 + 4 + 16_384 + 4 + 186 + 4 + 32_768
    + 4 + 10 * (4 + 186) + 1 + 4 + 66 + 4 + 16 * (4 + 4096),
}
equals(data["independent_bounds"], expected_bounds)
print(json.dumps({"status": "PASS_RECIPE_SELF_CHECK_ONLY", "checks": checks,
                  "wire_vectors": len(vectors), "candidate_executions": 0,
                  "native_crypto": "NOT_RUN"}, sort_keys=True))
