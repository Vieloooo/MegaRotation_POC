#!/usr/bin/env python3
"""Independent M1 native protobuf and durable journal decoders, reused for M2."""
import hashlib
import json
import re
import struct
import sys
from pathlib import Path

MAX = 2**64 - 1


def h(domain, *parts):
    return hashlib.sha256(b"".join(len(p).to_bytes(8, "big") + p for p in (domain.encode(), *parts))).digest()


def unique(items):
    output = {}
    for key, value in items:
        assert key not in output, f"duplicate JSON key {key}"
        output[key] = value
    return output


def load(raw):
    return json.loads(raw, object_pairs_hook=unique)


def varint(data, start):
    value = 0
    for i in range(10):
        j = start + i
        assert j < len(data), "truncated protobuf varint"
        byte = data[j]
        value |= (byte & 127) << (i * 7)
        if byte < 128:
            assert value <= MAX and (i == 0 or byte != 0), "noncanonical protobuf varint"
            return value, j + 1
    raise AssertionError("oversized protobuf varint")


def wire(data):
    idx = 0
    while idx < len(data):
        start = idx
        tag, idx = varint(data, idx)
        field, representation = divmod(tag, 8)
        assert field > 0, "invalid protobuf tag"
        if representation == 0:
            value, idx = varint(data, idx)
        elif representation == 2:
            size, idx = varint(data, idx)
            assert idx + size <= len(data), "truncated protobuf bytes"
            value = data[idx:idx + size]
            idx += size
        else:
            raise AssertionError(f"unexpected wire type {representation} for native smoke")
        yield field, value, data[start:idx]


def fields(data, repeated=None):
    result = {}
    if repeated:
        result[repeated] = []
    for field, value, _ in wire(data):
        if field == repeated:
            result[field].append(value)
        else:
            assert field not in result, f"unexpected duplicate field {field}"
            result[field] = value
    return result



class NativeProto:
    """Reflect over the local native.proto scalar/message/oneof subset. No generated Rust."""

    field = re.compile(r"(?:(optional|repeated)\s+)?(\w+)\s+(\w+)\s*=\s*(\d+)\s*;")
    oneof = re.compile(r"oneof\s+(\w+)\s*\{([^{}]*)\}")
    definitions = re.compile(r"\b(message|enum)\s+(\w+)\s*\{")

    def __init__(self, path):
        source = re.sub(r"//[^\n]*", "", path.read_text())
        self.types, self.enums = {}, set()
        for match in self.definitions.finditer(source):
            kind, name = match.groups()
            if kind == "enum":
                self.enums.add(name)
                continue
            nesting = 1
            at = match.end()
            while nesting:
                nesting += (source[at] == "{") - (source[at] == "}")
                at += 1
            body = source[match.end():at - 1]
            normal = [(*entry, None) for entry in self.field.findall(self.oneof.sub("", body))]
            variants = [(*entry, group) for group, lines in self.oneof.findall(body) for entry in self.field.findall(lines)]
            self.types[name] = {int(tag): (quantifier, ty, field, group) for quantifier, ty, field, tag, group in normal + variants}

    def default(self, quantifier, ty):
        if quantifier == "repeated" or ty == "bytes":
            return []
        if ty == "string":
            return ""
        if ty == "bool":
            return False
        if ty in self.types or quantifier == "optional":
            return None
        return 0

    def decode(self, name, data):
        fields_by_tag = self.types[name]
        obj = {}
        for quantifier, ty, field, group in fields_by_tag.values():
            obj[group or field] = None if group else self.default(quantifier, ty)
        seen = set()
        for tag, raw_value, _ in wire(data):
            quantifier, ty, field, group = fields_by_tag[tag]
            if ty in self.types:
                value = self.decode(ty, raw_value)
            elif ty == "bytes":
                value = list(raw_value)
            elif ty == "string":
                value = raw_value.decode()
            elif ty == "bool":
                assert raw_value in (0, 1)
                value = bool(raw_value)
            elif ty in ("sint32", "sint64"):
                value = zigzag(raw_value)
            else:
                assert isinstance(raw_value, int)
                value = raw_value
            if group:
                assert group not in seen
                seen.add(group)
                obj[group] = {"".join(part.title() for part in field.split("_")): value}
            elif quantifier == "repeated":
                obj[field].append(value)
            else:
                assert tag not in seen
                seen.add(tag)
                obj[field] = value
        return obj

def single(data):
    [[tag, value]] = fields(data).items()
    return tag, value


def zigzag(value):
    return (value >> 1) ^ -(value & 1)


def only_variants(data, expected):
    return b"".join(raw for tag, _, raw in wire(data) if tag not in expected)


def frames(path):
    with path.open("rb") as infile:
        while header := infile.read(9):
            assert len(header) == 9, "truncated frame header"
            direction, pid, count = struct.unpack(">BII", header)
            assert 0 <= pid < 15 and 0 < count <= 16 * 1024 * 1024
            payload = infile.read(count)
            assert len(payload) == count, "truncated frame"
            yield direction, pid, payload


def journal(path, magic, domain, binding):
    head = path.with_name(path.name + ".head").read_bytes()
    assert len(head) == 88 and head[:8] == magic
    worker = binding.get(4, 0).to_bytes(4, "big")
    assert head[56:] == h(f"I1b/{domain}-head/v1", binding[2], binding[3], worker, head[:56])
    extent, sequence = struct.unpack(">QQ", head[8:24])
    assert extent == path.stat().st_size
    digest = bytes(32)
    with path.open("rb") as infile:
        for seq in range(1, sequence + 1):
            size, complement = struct.unpack(">QQ", infile.read(16))
            assert 0 < size <= 16 * 1024 * 1024 and (size ^ complement) == MAX
            raw = infile.read(size)
            rec_hash = infile.read(32)
            assert rec_hash == h(f"I1b/{domain}-record/v1", raw)
            rec = load(raw)
            assert rec["schema"] == 1 and rec["sequence"] == seq
            assert bytes(rec["previous_record_hash"]) == digest
            digest = rec_hash
            yield seq, digest, rec
        assert not infile.read(1)
    assert digest == head[24:56]


def identity_of_envelope(envelope):
    return tuple(envelope.get(tag, 0) for tag in range(1, 10))


def reference(root, p, stem):
    return root / f"worker-{p}" / stem
