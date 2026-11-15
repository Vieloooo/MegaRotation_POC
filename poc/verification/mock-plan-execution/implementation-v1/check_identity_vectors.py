#!/usr/bin/env python3
"""Independent finite-field check of strict Ed25519 public identity vectors."""
import json
from pathlib import Path

P = 2**255 - 19
D = (-121665 * pow(121666, -1, P)) % P
ORDER = 2**252 + 27742317777372353535851937790883648493
IDENTITY = (0, 1)


def decode(raw):
    n = int.from_bytes(raw, "little")
    sign, y = n >> 255, (n & ((1 << 255) - 1)) % P
    squared = (y * y - 1) * pow(D * y * y + 1, -1, P) % P
    x = pow(squared, (P + 3) // 8, P)
    if x * x % P != squared:
        x = x * pow(2, (P - 1) // 4, P) % P
    if x * x % P != squared:
        return None
    return ((-x) % P if x % 2 != sign else x, y)


def encode(point):
    x, y = point
    return (y | ((x & 1) << 255)).to_bytes(32, "little")


def add(left, right):
    x, y = left
    a, b = right
    t = D * x * a * y * b % P
    return ((x * b + y * a) * pow(1 + t, -1, P) % P,
            (y * b + x * a) * pow(1 - t, -1, P) % P)


def multiply(point, scalar):
    result = IDENTITY
    while scalar:
        if scalar & 1:
            result = add(result, point)
        point = add(point, point)
        scalar >>= 1
    return result


def run():
    vectors = json.loads((Path(__file__).parent / "registry-negative-vectors.json").read_text())
    checked = []
    for case in vectors["identity_cases"]:
        raw = bytes.fromhex(case["identity_key_hex"])
        point = decode(raw)
        assert (point is not None) == case["decompresses"], case["id"]
        if point is None:
            accepted = False
        else:
            canonical = encode(point) == raw
            small = multiply(point, 8) == IDENTITY
            torsion_free = multiply(point, ORDER) == IDENTITY
            assert [canonical, small, torsion_free] == [case["canonical"], case["small_order"], case["torsion_free"]], case["id"]
            accepted = canonical and not small and torsion_free
        assert accepted == case["expected_accept"], case["id"]
        checked.append(case["id"])
    return {"status": "PASS_REFERENCE_MATH", "cases": checked, "seed": None,
            "limits": "Finite public point vectors only; not a PoP/BLS verifier or timing-safe cryptography implementation"}


if __name__ == "__main__":
    print(json.dumps(run(), indent=2))
