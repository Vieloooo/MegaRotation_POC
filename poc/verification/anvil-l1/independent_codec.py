#!/usr/bin/env python3
"""Specification-only ABI/EIP-712 vector aid; not production cryptography.

No candidate code imported. Signing uses installed libsecp256k1 via coincurve.
The small Keccak reference is checked against published fixed digests and Cast.
No private key is printed or embedded in serialized results.
"""
from __future__ import annotations
import argparse
import importlib.metadata
import json
import re
import secrets
import subprocess
from typing import Any
from coincurve import PrivateKey, PublicKey

MASK = (1 << 64) - 1
RC = (0x0000000000000001, 0x0000000000008082, 0x800000000000808A,
      0x8000000080008000, 0x000000000000808B, 0x0000000080000001,
      0x8000000080008081, 0x8000000000008009, 0x000000000000008A,
      0x0000000000000088, 0x0000000080008009, 0x000000008000000A,
      0x000000008000808B, 0x800000000000008B, 0x8000000000008089,
      0x8000000000008003, 0x8000000000008002, 0x8000000000000080,
      0x000000000000800A, 0x800000008000000A, 0x8000000080008081,
      0x8000000000008080, 0x0000000080000001, 0x8000000080008008)
ROT = ((0, 36, 3, 41, 18), (1, 44, 10, 45, 2), (62, 6, 43, 15, 61),
       (28, 55, 25, 21, 56), (27, 20, 39, 8, 14))
SECP_N = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141


def rol64(x: int, n: int) -> int:
    return ((x << n) | (x >> ((64 - n) % 64))) & MASK


def keccak_f(a: list[int]) -> None:
    for rc in RC:
        c = [a[x] ^ a[x + 5] ^ a[x + 10] ^ a[x + 15] ^ a[x + 20] for x in range(5)]
        d = [c[(x - 1) % 5] ^ rol64(c[(x + 1) % 5], 1) for x in range(5)]
        for x in range(5):
            for y in range(5):
                a[x + 5*y] ^= d[x]
        b = [0] * 25
        for x in range(5):
            for y in range(5):
                b[y + 5*((2*x + 3*y) % 5)] = rol64(a[x + 5*y], ROT[x][y])
        for x in range(5):
            for y in range(5):
                a[x + 5*y] = b[x + 5*y] ^ ((~b[(x+1)%5 + 5*y]) & b[(x+2)%5 + 5*y])
        a[0] ^= rc


def keccak256(data: bytes) -> bytes:
    rate = 136
    padded = bytearray(data)
    pad = rate - len(padded) % rate
    padded.extend(b'\x01' + b'\0' * (pad - 1))
    padded[-1] |= 0x80
    state = [0] * 25
    for off in range(0, len(padded), rate):
        block = padded[off:off+rate]
        for i in range(rate // 8):
            state[i] ^= int.from_bytes(block[8*i:8*i+8], 'little')
        keccak_f(state)
    return b''.join(x.to_bytes(8, 'little') for x in state)[:32]


def hx(data: bytes) -> str:
    return '0x' + data.hex()


def as_bytes(value: bytes | str) -> bytes:
    return bytes.fromhex(value.removeprefix('0x')) if isinstance(value, str) else value


def word(value: int, bits: int = 256) -> bytes:
    if isinstance(value, bool) or not isinstance(value, int) or bits < 8 or bits > 256 or bits % 8 or not 0 <= value < 1 << bits:
        raise ValueError(f'out-of-range uint{bits}')
    return value.to_bytes(32, 'big')


def dynamic(ty: Any) -> bool:
    return any(dynamic(t) for t in ty) if isinstance(ty, (tuple, list)) else ty in ('bytes', 'string')


def static_size(ty: Any) -> int:
    if dynamic(ty):
        raise ValueError('dynamic type has no static size')
    return sum(static_size(t) for t in ty) if isinstance(ty, (tuple, list)) else 32


def encode_value(ty: Any, value: Any) -> bytes:
    if isinstance(ty, (tuple, list)):
        return abi_encode(ty, value)
    if ty.startswith('uint'):
        return word(value, int(ty[4:] or 256))
    if ty == 'address':
        raw = as_bytes(value)
        if len(raw) != 20:
            raise ValueError('address must be exactly 20 bytes')
        return b'\0' * 12 + raw
    if ty == 'bool':
        if not isinstance(value, bool):
            raise ValueError('bool must be boolean')
        return word(int(value))
    if re.fullmatch(r'bytes(?:[1-9]|[12][0-9]|3[0-2])', ty):
        size = int(ty[5:])
        raw = as_bytes(value)
        if len(raw) != size:
            raise ValueError('fixed bytes wrong length')
        return raw + b'\0' * (32-size)
    if ty in ('bytes', 'string'):
        raw = value.encode('utf8') if ty == 'string' else as_bytes(value)
        return word(len(raw)) + raw + b'\0' * ((-len(raw)) % 32)
    raise ValueError(f'unsupported ABI type {ty!r}')


def abi_encode(types: Any, values: Any) -> bytes:
    if len(types) != len(values):
        raise ValueError('ABI arity mismatch')
    head_size = sum(32 if dynamic(t) else static_size(t) for t in types)
    head, tail = [], bytearray()
    for ty, value in zip(types, values):
        encoded = encode_value(ty, value)
        if dynamic(ty):
            head.append(word(head_size + len(tail)))
            tail.extend(encoded)
        else:
            head.append(encoded)
    return b''.join(head) + tail


def dependencies(types: dict, primary: str) -> set[str]:
    seen = set()
    def visit(name: str) -> None:
        for field_type, _ in types[name]:
            if field_type in types and field_type not in seen and field_type != primary:
                seen.add(field_type)
                visit(field_type)
    visit(primary)
    return seen


def encode_type(types: dict, primary: str) -> str:
    def definition(name: str) -> str:
        return name + '(' + ','.join(t+' '+n for t,n in types[name]) + ')'
    return definition(primary) + ''.join(definition(t) for t in sorted(dependencies(types, primary)))


def hash_struct(types: dict, primary: str, values: dict) -> bytes:
    if set(values) != {n for _, n in types[primary]}:
        raise ValueError('missing or extraneous struct fields')
    words = [keccak256(encode_type(types, primary).encode('ascii'))]
    for ty, name in types[primary]:
        value = values[name]
        if ty in types:
            words.append(hash_struct(types, ty, value))
        elif ty == 'bytes':
            words.append(keccak256(as_bytes(value)))
        elif ty == 'string':
            words.append(keccak256(value.encode('utf8')))
        else:
            encoded = encode_value(ty, value)
            if len(encoded) != 32:
                raise ValueError('EIP712 static field must be one word')
            words.append(encoded)
    return keccak256(b''.join(words))


DOMAIN_FIELDS = [('string','name'),('string','version'),('uint256','chainId'),
                 ('address','verifyingContract'),('bytes32','salt')]


def domain_separator(name: str, version: str, chain_id: int, address: bytes | str, salt: bytes | str) -> bytes:
    return hash_struct({'EIP712Domain': DOMAIN_FIELDS}, 'EIP712Domain', {
        'name':name, 'version':version, 'chainId':chain_id, 'verifyingContract':address, 'salt':salt})


def typed_digest(domain: bytes, body_hash: bytes) -> bytes:
    if len(domain) != 32 or len(body_hash) != 32:
        raise ValueError('typed digest requires two 32-byte hashes')
    return keccak256(b'\x19\x01' + domain + body_hash)


def public_address(key: PublicKey) -> bytes:
    return keccak256(key.format(compressed=False)[1:])[-20:]


def sign_digest(private_key: bytes, digest: bytes) -> bytes:
    if len(digest) != 32:
        raise ValueError('digest must be 32 bytes')
    raw = PrivateKey(private_key).sign_recoverable(digest, hasher=None)
    if raw[64] > 1:
        raise ValueError('rare recovery overflow not supported by EVM profile')
    if int.from_bytes(raw[32:64], 'big') > SECP_N // 2:
        raise ValueError('signer returned high-s')
    return raw[:64] + bytes([27 + raw[64]])


def recover_address(signature: bytes, digest: bytes) -> bytes:
    if len(signature) != 65 or len(digest) != 32 or signature[64] not in (27,28):
        raise ValueError('invalid signature format')
    r, s = int.from_bytes(signature[:32], 'big'), int.from_bytes(signature[32:64], 'big')
    if not 0 < r < SECP_N or not 0 < s <= SECP_N // 2:
        raise ValueError('noncanonical r/s')
    recoverable = signature[:64] + bytes([signature[64]-27])
    return public_address(PublicKey.from_signature_and_message(recoverable, digest, hasher=None))


def selftest(cast: str | None = None) -> dict:
    fixed = {b'':'c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470',
             b'abc':'4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45'}
    for data, expected in fixed.items():
        assert keccak256(data).hex() == expected
    lengths = (0, 1, 31, 32, 64, 135, 136, 137, 271, 272, 273, 1024)
    checked = []
    if cast:
        for n in lengths:
            data = bytes(i % 256 for i in range(n))
            actual = subprocess.check_output([cast, 'keccak', hx(data)], text=True).strip()
            assert actual == hx(keccak256(data)), n
            checked.append(n)
        cases = [
            ('f(uint64,bytes)', ['uint64','bytes'], [7,b'abc'], ['7','0x616263']),
            ('f((uint64,uint64,bytes32,uint8,bytes),bytes)', [('uint64','uint64','bytes32','uint8','bytes'),'bytes'], [(5,3,b'\x11'*32,1,b'gpos'),b'abc'], ['(5,3,0x'+'11'*32+',1,0x67706f73)','0x616263']),
            ('f((uint64,uint64),(bytes,string))', [('uint64','uint64'),('bytes','string')], [(5,3),(b'xyz','hello')], ['(5,3)','(0x78797a,hello)']),
        ]
        for signature, types, values, args in cases:
            actual = subprocess.check_output([cast, 'abi-encode', signature, *args], text=True).strip()
            assert actual == hx(abi_encode(types,values)), signature
    else:
        cases = []
    private = secrets.token_bytes(32)
    key = PrivateKey(private)
    digest = keccak256(b'independent-vector-tool-selftest')
    signature = sign_digest(private,digest)
    address = public_address(key.public_key)
    assert recover_address(signature,digest) == address
    assert recover_address(signature,keccak256(b'changed')) != address
    assert sign_digest(private,digest) == signature
    high_s = signature[:32] + (SECP_N-int.from_bytes(signature[32:64],'big')).to_bytes(32,'big') + bytes([55-signature[64]])
    try:
        recover_address(high_s,digest)
        raise AssertionError('high-s unexpectedly accepted')
    except ValueError:
        pass
    return {'tool':'independent-spec-codec-v1','coincurve':importlib.metadata.version('coincurve'),
            'keccak_fixed_vectors':len(fixed),'cast_keccak_lengths':checked,
            'cast_abi_shapes':[x[0] for x in cases],
            'secp256k1':['recover-original','reject-changed-digest-identity','deterministic-sign','reject-high-s'],
            'private_key_output':False,'status':'PASS'}


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--self-test', action='store_true')
    p.add_argument('--cast')
    args = p.parse_args()
    if args.self_test:
        print(json.dumps(selftest(args.cast), indent=2))
    else:
        p.error('specify --self-test; import for ABI-frozen vectors')
