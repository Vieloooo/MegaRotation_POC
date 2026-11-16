"""Read the sealed M1 smoke's v1 wire objects for offline projection.

The layouts are from poc-types/domain/objects.rs, sim-espresso/wire.rs,
viewer-gfr/codec.rs, and publication-bridge/result.rs at the smoke revision.
Final proof bytes are opaque here and compared to the independent verifier.
"""

import hashlib


class EvidenceError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise EvidenceError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def domain_hash(domain, *parts):
    digest = hashlib.sha256()
    for part in (domain.encode("ascii"), *parts):
        digest.update(len(part).to_bytes(8, "big"))
        digest.update(part)
    return digest.hexdigest()


class Reader:
    def __init__(self, data, label, byteorder="big"):
        self.data = data
        self.label = label
        self.byteorder = byteorder
        self.offset = 0

    def check(self, condition, message):
        require(condition, f"{self.label} @ byte {self.offset}: {message}")

    def take(self, size):
        end = self.offset + size
        self.check(0 <= size and end <= len(self.data), "truncated input")
        result = self.data[self.offset:end]
        self.offset = end
        return result

    def uint(self, size):
        return int.from_bytes(self.take(size), self.byteorder)

    def digest(self):
        return self.take(32).hex()

    def count(self, maximum):
        count = self.uint(4)
        self.check(count <= maximum, f"count/length exceeds {maximum}")
        return count

    def frame(self, maximum):
        return self.take(self.count(maximum))

    def expect(self, prefix):
        self.check(self.take(len(prefix)) == prefix, "bad magic or wire version")

    def finish(self, value):
        self.check(self.offset == len(self.data), "trailing input bytes")
        return value


def canonical(data, label):
    reader = Reader(data, label)
    reader.expect(b"\x00\x01")
    return reader


def block_ref(reader):
    return {
        "height": reader.uint(8), "origin": reader.uint(8), "physical": reader.digest()
    }


def anchor(reader):
    return {**block_ref(reader), "timestamp_us": reader.uint(8)}


def decode_config(data, label):
    r = canonical(data, label)
    config = {"chain_id": r.uint(8), "rule_version": r.uint(2), "verifier_version": r.uint(2)}
    r.check((config["rule_version"], config["verifier_version"]) == (1, 1), "config versions")
    for field in ("registry_hash", "dep_fixture_hash", "l1_fixture_hash"):
        config[field] = r.digest()
    config["genesis"] = anchor(r)
    config["genesis_state_root"] = r.digest()
    config["genesis_native_hash"] = r.digest()
    for field in (
        "l2_slot_us", "l1_genesis_timestamp_us", "l1_block_us", "t_dep_us", "t_drift_us",
        "metadata_window_us"
    ):
        config[field] = r.uint(8)
    config["byte_limit"] = r.uint(4)
    config["ref"] = domain_hash("PoC/PublicConfig/v1", data)
    r.check(0 < config["l2_slot_us"] <= config["l1_block_us"], "slot parameters")
    return r.finish(config)


def decode_l1(data, label):
    r = canonical(data, label)
    rows = []
    for i in range(r.count(4096)):
        row = {
            "height": r.uint(8), "hash": r.digest(), "parent": r.digest(),
            "timestamp_s": r.uint(8), "finalized_us": r.uint(8), "control_commitment": r.digest(),
            "controls": [],
        }
        row["timestamp_us"] = row["timestamp_s"] * 1_000_000
        row["reader_received_us"] = None
        r.check(row["height"] == i, "noncontiguous L1 height")
        r.check(row["finalized_us"] >= row["timestamp_us"], "premature L1 finality")
        r.check(
            row["control_commitment"] == domain_hash("PoC/EmptyL1Control/v1", b"\0\1\0\0\0\0"),
            "expected empty control commitment",
        )
        parent = rows[-1]["hash"] if rows else "00" * 32
        r.check(row["parent"] == parent, "L1 parent mismatch")
        if rows:
            r.check(row["timestamp_us"] > rows[-1]["timestamp_us"], "L1 timestamp order")
            r.check(row["finalized_us"] >= rows[-1]["finalized_us"], "L1 finality order")
        rows.append(row)
    r.check(bool(rows), "empty L1 fixture")
    return r.finish(rows)


def decode_dep(data, label):
    r = canonical(data, label)
    first_height = r.uint(8)
    rows = {}
    for i in range(r.count(4096)):
        credits = []
        for _ in range(r.count(1024)):
            r.expect(b"\x02")
            credits.append({"to": r.uint(1), "amount": r.uint(8)})
        rows[first_height + i] = credits
    return r.finish(rows)


def decode_descriptor(data, label):
    r = canonical(data, label)
    d = {"beta": r.uint(8), "session": r.uint(8), "cause": r.uint(1)}
    source_tag = r.uint(1)
    r.check(source_tag in (0, 1) and d["cause"] in (0, 1, 2), "healthy boundary tag")
    d["source_region"] = r.uint(1) if source_tag else None
    d["region"] = r.uint(1)
    committee = []
    for _ in range(r.count(5)):
        committee.append(r.uint(1))
        r.take(32 + 96)
        r.check(r.uint(8) == 1, "committee weight")
    r.check((r.uint(8), r.uint(8)) == (5, 1), "total weight/fault bound")
    eligible = [r.uint(1) for _ in range(r.count(5))]
    r.check(d["region"] in range(3) and committee == eligible ==
            list(range(d["region"] * 5, d["region"] * 5 + 5)), "committee membership")
    rule, version, size = r.uint(2), r.uint(2), r.uint(4)
    r.check(version == 1 and (rule, size) in ((1, 0), (2, 32)), "unsupported leader rule")
    if rule == 2:
        r.check(d["cause"] in (1, 2), "historical leader rule without a predecessor")
        d["historical_rule_ref"] = r.digest()
    h_min, h_max = r.uint(8), r.uint(8)
    r.check(h_max - h_min == 9, "fixed window")
    d["window"] = [h_min, h_max]
    d["reference"] = block_ref(r)
    d["config_ref"] = r.digest()
    d["authorization_ref"] = r.digest()
    r.check(d.get("historical_rule_ref", d["authorization_ref"]) == d["authorization_ref"],
            "historical rule does not bind authorization ref")
    d["anchor"] = anchor(r)
    d["cid"] = r.digest()
    r.check(d["anchor"]["height"] < h_min, "invalid anchor height")
    return r.finish(d)


def decode_physical(data, label):
    r = canonical(data, label)
    block = {
        "parent": r.digest(), "height": r.uint(8), "timestamp_us": r.uint(8),
        "origin": r.uint(8), "pre_state_root": r.digest(),
        "physical": domain_hash("PoC/PhysicalBlock/v1", data),
    }
    txs = []
    for _ in range(r.count(4)):
        tx = Reader(r.frame(19), label)
        tx.expect(b"\x01")
        txs.append(tx.finish(
            {"from": tx.uint(1), "to": tx.uint(1), "nonce": tx.uint(8), "amount": tx.uint(8)}
        ))
    block["transactions"] = txs
    return r.finish(block)


def decode_espresso(data, label):
    require(len(data) <= 1_048_576, f"{label}: oversized PELG block")
    r = Reader(data, label, "little")
    r.expect(b"PELG\x01\x00")
    source = r.frame(256).decode("ascii")
    r.check(bool(source), "empty source")
    q, key_us, proposal_us = r.uint(8), r.uint(8), r.uint(8)
    r.check(r.uint(1) == 1, "expected live proposal")
    finalized_us = r.uint(8)
    r.check(finalized_us >= proposal_us, "premature Espresso finality")
    messages = []
    for index in range(r.count(1024)):
        r.check(r.uint(4) == 0x47465201, "unrecognized publication codec")
        label_at_message = f"{label} message[{index}]"
        obj = Reader(r.frame(65_536), label_at_message)
        obj.expect(b"GFRP\x00\x01")
        tag = obj.uint(1)
        if tag == 0:
            body = decode_physical(obj.frame(65_536), label_at_message)
            message = {"kind": "body", "body": body}
        else:
            obj.check(tag == 1, "unknown GFRP variant")
            descriptor = decode_descriptor(obj.frame(16_384), label_at_message)
            message = {
                "kind": "proof", "descriptor": descriptor,
                "target": obj.digest(), "proof": obj.frame(32_768).hex(),
            }
        messages.append(obj.finish({"index": index, **message}))
    return r.finish({
        "source": source, "q": q, "key_us": key_us, "proposal_us": proposal_us,
        "finalized_us": finalized_us, "file": label, "messages": messages,
    })


def decode_result(data, label):
    require(len(data) <= 128 * 1024, f"{label}: oversized regional result")
    r = Reader(data, label)
    r.expect(b"H3RS\x00\x01")
    descriptor = decode_descriptor(r.frame(16_384), label)
    body = decode_physical(r.frame(186), label)
    proof = r.frame(32_768).hex()
    chain = [decode_physical(r.frame(186), label) for _ in range(r.count(10))]
    r.check(chain and chain[-1] == body, "regional chain has no matching target")
    source_tag = r.uint(1)
    r.check(source_tag in (0, 1), "regional anchor source kind")
    source = canonical(r.frame(66 if source_tag else 34), label)
    reference = {"cid": source.digest(), "physical": source.digest()} if source_tag else {
        "config_ref": source.digest()
    }
    source.finish(None)
    objects = [sha256(r.frame(4096)) for _ in range(r.count(16))]
    return r.finish({
        "file": label, "descriptor": descriptor, "body": body, "chain": chain, "proof": proof,
        "anchor_evidence": {"source": reference, "object_hashes": objects},
    })
