#!/usr/bin/env python3
"""Independent EC16 source/capture oracle; uses no candidate library or output to derive expected bytes."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import re
import struct

COMMIT = "fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171"
IMAGE = "sha256:cfb845e49cb5369f77cd8bbda0b4fe4aad340793cfd130b7f2bb631e59d2db66"
INDEX = "sha256:af73f55630690caa35aa0bc015d60dc03333791f966fdcc07b0798e33d4f768e"
CONFIG = "sha256:b376be602febaa35fd38c1f4bc1144fb79017bfed274a4a26ed90bf052bfd06f"
CODEC = 0xEC160001
NS = 160016
MAX_FILE = 16 * 1024 * 1024
MAX_BUNDLE = 256 * 1024 * 1024


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f"duplicate JSON key {key}")
        result[key] = value
    return result


def parse(raw):
    return json.loads(raw, object_pairs_hook=unique_object,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))


def integer(value, maximum=(1 << 64) - 1):
    require(type(value) is int and 0 <= value <= maximum, f"invalid integer {value!r}")
    return value


def decoded(value):
    require(type(value) is str, "base64 is not a string")
    result = base64.b64decode(value, validate=True)
    require(base64.b64encode(result).decode() == value, "noncanonical base64")
    return result


def source_id(identity):
    expected = {"schema": "EC16-SOURCE-1", "official_commit": COMMIT,
                "image_amd64_digest": IMAGE, "protocol_version": "0.3",
                "api_version": "v1", "namespace": NS}
    require(all(identity.get(k) == v for k, v in expected.items()), "source pin mismatch")
    require(re.fullmatch(r"[A-Za-z0-9_-]{1,64}", identity["run_id"]) is not None, "invalid run ID")
    require(re.fullmatch(r"BLOCK~[A-Za-z0-9_-]+", identity["genesis_block_hash"]) is not None, "invalid genesis tag")
    fields = [identity[k] for k in ("schema", "official_commit", "image_amd64_digest",
                                   "protocol_version", "api_version", "namespace", "run_id", "genesis_block_hash")]
    return "EC16:" + hashlib.sha256("".join(str(v) + "\n" for v in fields).encode()).hexdigest()


def u32(value):
    return struct.pack("<I", integer(value, (1 << 32) - 1))


def u64(value):
    return struct.pack("<Q", integer(value))


def lp(value):
    value = value.encode() if isinstance(value, str) else value
    return u32(len(value)) + value


def audit(sid, blocks, codec=CODEC):
    result = bytearray(b"EC16-PUBLIC-1\0" + lp(sid) + u32(len(blocks)))
    for block in blocks:
        txs = block["transactions"]
        result += u64(block["q"]) + u64(block["theta_us"]) + lp(block["hash"]) + u32(len(txs))
        for tx in txs:
            result += (u32(tx["index"]) + u64(tx["namespace"]) + u32(tx["pos_in_namespace"])
                       + lp(tx["hash"]) + lp(tx["bytes"]))
        selected = [tx for tx in txs if tx["namespace"] == NS]
        result += u32(len(selected))
        for index, tx in enumerate(selected):
            result += u32(index) + u32(tx["index"]) + u32(codec) + lp(tx["bytes"])
    return bytes(result)


def source_payload(block):
    """Pinned source: NsTable validation + total namespace transaction-table interpretation.

    This validates list consistency, never QC/VID/commitment authenticity. Namespace
    offsets have the upstream deserialization invariants. Transaction table counts
    and ranges use upstream truncation, including valid zero-length transactions.
    """
    payload = decoded(block["payload"]["raw_payload"])
    require(integer(block["size"]) == len(payload), "raw payload size mismatch")
    table = decoded(block["payload"]["ns_table"]["bytes"])
    require(block["payload"]["ns_table"] == block["header"]["fields"]["ns_table"], "namespace metadata mismatch")
    require(len(table) >= 4 and (len(table) - 4) % 8 == 0, "invalid namespace table length")
    n = int.from_bytes(table[:4], "little")
    require(n == (len(table) - 4) // 8, "namespace count mismatch")
    previous = 0
    namespaces = set()
    transactions = []
    for i in range(n):
        namespace, end = struct.unpack_from("<II", table, 4 + 8 * i)
        require(namespace not in namespaces and end > previous and end <= len(payload), "namespace offset or identity invalid")
        namespaces.add(namespace)
        ns = payload[previous:end]
        declared = int.from_bytes(ns[:4].ljust(4, b"\0"), "little")
        count = min(declared, max(0, len(ns) - 4) // 4)
        body_start = 4 + 4 * declared
        prior_tx_end = 0
        for position in range(count):
            tx_end = int.from_bytes(ns[4 + 4 * position:8 + 4 * position], "little")
            bounded_end = min(body_start + tx_end, len(ns))
            bounded_start = min(body_start + prior_tx_end, bounded_end)
            transactions.append((namespace, position, ns[bounded_start:bounded_end]))
            prior_tx_end = tx_end
        previous = end
    require(previous == len(payload), "namespace table does not cover full raw payload")
    require(len(transactions) == integer(block["num_transactions"], 1024), "source iterator count mismatch")
    return transactions


def assemble(identity, observations, check_payload=True):
    blocks = []
    require(len({key[1] for key in observations}) <= 4096, "pending/complete height capacity")
    q = 0
    while ("block", q) in observations and ("leaf", q) in observations:
        b, leaf = observations[("block", q)], observations[("leaf", q)]
        header = b["header"]
        require(header["version"] == {"Version": {"major": 0, "minor": 3}}, "unsupported header profile")
        require(integer(header["fields"]["height"]) == q, "header height mismatch")
        theta = integer(integer(header["fields"]["timestamp"]) * 1_000_000)
        require(not blocks or theta >= blocks[-1]["theta_us"], "timestamp regression")
        require(leaf["leaf"]["block_header"] == header and isinstance(leaf["qc"], dict), "leaf/header mismatch")
        require(re.fullmatch(r"BLOCK~[A-Za-z0-9_-]+", b["hash"]) is not None, "invalid block tag")
        if q == 0:
            require(b["hash"] == identity["genesis_block_hash"], "genesis source mismatch")
        n = integer(b["num_transactions"], 1024)
        require(not any(k[0] == "tx" and k[1] == q and k[2] >= n for k in observations), "extra transaction position")
        if any(("tx", q, i) not in observations for i in range(n)):
            break
        raw_txs = source_payload(b) if check_payload else None
        txs = []
        for i in range(n):
            t = observations[("tx", q, i)]
            require(integer(t["index"]) == i and integer(t["block_height"]) == q and t["block_hash"] == b["hash"], "transaction q/index/hash mismatch")
            namespace = integer(t["namespace"], (1 << 32) - 1)
            require(namespace == integer(t["transaction"]["namespace"], (1 << 32) - 1), "transaction namespace mismatch")
            position = integer(t["pos_in_namespace"], (1 << 32) - 1)
            data = decoded(t["transaction"]["payload"])
            require(len(data) <= 65536, "transaction payload capacity")
            require(re.fullmatch(r"TX~[A-Za-z0-9_-]+", t["hash"]) is not None, "invalid transaction tag")
            if check_payload:
                require(raw_txs[i] == (namespace, position, data), f"raw payload vs transaction response mismatch q={q} i={i}")
            txs.append({"index": i, "namespace": namespace, "pos_in_namespace": position, "hash": t["hash"], "bytes": data})
        blocks.append({"q": q, "theta_us": theta, "hash": b["hash"], "transactions": txs})
        q += 1
    require(sum(len(t["bytes"]) for b in blocks for t in b["transactions"]) <= 64 * 1024 * 1024,
            "decoded native payload capacity")
    for b in blocks:
        selected = [t for t in b["transactions"] if t["namespace"] == NS]
        require(116 + sum(8 + len(t["bytes"]) for t in selected) <= 1048576,
                "selected projection envelope capacity")
    return blocks


def projection(sid, blocks, digest):
    result = {"schema": "EC16-AUDIT-1", "source_id": sid, "public_sha256": digest, "blocks": []}
    for block in blocks:
        native = [{"source_position": {"q": block["q"], "source_transaction_index": tx["index"]},
                   "namespace": tx["namespace"], "pos_in_namespace": tx["pos_in_namespace"],
                   "transaction_hash": tx["hash"], "payload_base64": base64.b64encode(tx["bytes"]).decode()}
                  for tx in block["transactions"]]
        selected = [{"message_index": j,
                     "source_position": {"q": block["q"], "source_transaction_index": tx["index"]},
                     "codec": CODEC, "payload_base64": base64.b64encode(tx["bytes"]).decode()}
                    for j, tx in enumerate(t for t in block["transactions"] if t["namespace"] == NS)]
        result["blocks"].append({"q": block["q"], "theta_us": block["theta_us"], "block_hash": block["hash"],
                                  "native_transactions": native, "publication_messages": selected,
                                  "timing": {"proposed_at": "Unknown", "finalized_at": "Unknown", "live_at_proposal": "Unknown"}})
    return result


def load_bundle(root):
    root = root.resolve(strict=True)
    manifest_path = (root / "manifest.json").resolve(strict=True)
    require(manifest_path.is_relative_to(root) and manifest_path.is_file(), "manifest escapes bundle")
    require(manifest_path.stat().st_size <= MAX_FILE, "manifest capacity")
    raw = manifest_path.read_bytes()
    require(len(raw) <= MAX_FILE, "manifest capacity")
    manifest = parse(raw)
    require(manifest["schema"] == "EC16-CAPTURE-1", "unknown recording schema")
    sid = source_id(manifest["identity"])
    require(manifest["source_id"] == sid, "manifest source ID mismatch")
    refs = list(manifest["artifacts"])
    last = -1
    for record in manifest["records"]:
        require(integer(record["id"]) > last, "nonincreasing record IDs")
        last = record["id"]
        require(record["source_id"] == sid, "record source ID mismatch")
        require(re.fullmatch(r"[A-Za-z0-9_-]{1,64}", record["reader_id"]) is not None, "invalid reader ID")
        for key in ("request", "response"):
            if record[key] is not None:
                refs.append(record[key])
        require((record["status"] is None) == (record["response"] is None), "HTTP status/body mismatch")
    files = {}
    by_path = {}
    total = len(raw)
    for ref in refs:
        name = ref["path"]
        require(not name.startswith("/") and "\\" not in name and all(p not in ("", ".", "..") for p in name.split("/")), "unsafe recorded path")
        resolved = (root / name).resolve(strict=True)
        require(resolved.is_relative_to(root) and resolved.is_file() and resolved != manifest_path, "recorded path escapes bundle")
        if resolved not in files:
            length = resolved.stat().st_size
            require(length <= MAX_FILE and total + length <= MAX_BUNDLE, "raw file/bundle capacity")
            contents = resolved.read_bytes()
            require(len(contents) == length, "raw file changed while read")
            files[resolved] = contents
            total += length
        require(hashlib.sha256(files[resolved]).hexdigest() == ref["sha256"], "raw file SHA256 mismatch")
        by_path[name] = files[resolved]
    return manifest, by_path, total, hashlib.sha256(raw).hexdigest()


def check(root, reader_id, expected_candidate=None):
    m, files, total, manifest_sha = load_bundle(root)
    sid = m["source_id"]
    if m["capture_kind"] == "official":
        c = m["configuration"]
        if expected_candidate is not None:
            require(c["candidate_sha"] == expected_candidate, "wrong candidate revision in official run")
        for name, expected in (("official-index.json", INDEX), ("official-manifest-amd64.json", IMAGE), ("official-config.json", CONFIG)):
            require("sha256:" + hashlib.sha256(files[name]).hexdigest() == expected, "official OCI pin mismatch")
        require(parse(files["official-config.json"])["config"]["Labels"]["org.opencontainers.image.revision"] == COMMIT, "official image revision mismatch")
    observations = {}
    unavailable = []
    acknowledgements = []
    repeats = 0
    selected_records = [r for r in m["records"] if r["reader_id"] == reader_id]
    require(selected_records, "reader absent from capture")
    for r in selected_records:
        status = r["status"]
        if m["capture_kind"] == "official":
            service = "anvil" if r["path"] == "/" else "dev" if r["path"] == "/api/dev-info" else "api"
            require(r["observation"]["origin"] == f'http://127.0.0.1:{m["configuration"]["ports"][service]}', "nonlocal/wrong recorded origin")
        if status is None or status in (404, 408, 429) or (type(status) is int and 500 <= status <= 599):
            unavailable.append(r["id"])
            continue
        if r["method"] == "POST" and r["path"] == "/v1/submit/submit":
            if status != 200:
                require(400 <= status <= 599, "invalid submit status")
                continue
            request = parse(files[r["request"]["path"]])
            require(request["namespace"] == NS, "submit namespace mismatch")
            acknowledgements.append((r["id"], parse(files[r["response"]["path"]]), decoded(request["payload"])))
            continue
        require(status == 200, "unexpected response/redirect")
        value = parse(files[r["response"]["path"]])
        match = re.fullmatch(r"/v1/availability/(block|leaf)/(0|[1-9][0-9]*)", r["path"])
        tx = re.fullmatch(r"/v1/availability/transaction/(0|[1-9][0-9]*)/(0|[1-9][0-9]*)/noproof", r["path"])
        key = (match[1], int(match[2])) if match else ("tx", int(tx[1]), int(tx[2])) if tx else None
        if key is not None:
            if key in observations:
                require(observations[key] == value, "conflicting repeated raw public object")
                repeats += 1
            observations[key] = value
    blocks = assemble(m["identity"], observations)
    encoded = audit(sid, blocks)
    digest = hashlib.sha256(encoded).hexdigest()
    pubs = []
    for rid, txhash, data in acknowledgements:
        positions = [{"q": b["q"], "source_transaction_index": t["index"]} for b in blocks for t in b["transactions"]
                     if t["namespace"] == NS and t["hash"] == txhash and t["bytes"] == data]
        pubs.append({"record_id": rid, "transaction_hash": txhash, "payload_hex": data.hex(), "positions": positions})
    report = {"status": "PASS", "basis": "independent raw payload/list and audit consistency; trusted local API; no QC/VID verification",
              "capture_kind": m["capture_kind"], "reader_id": reader_id, "source_id": sid,
              "manifest_sha256": manifest_sha, "bundle_unique_bytes": total,
              "complete_blocks": len(blocks), "tip": blocks[-1]["q"] if blocks else None,
              "raw_payload_cross_checks": len(blocks), "native_transactions": sum(len(b["transactions"]) for b in blocks),
              "public_sha256": digest, "audit_bytes": len(encoded), "unavailable_records": unavailable,
              "repeated_public_responses": repeats, "publications": pubs}
    return report, encoded, projection(sid, blocks, digest)


def check_vectors(path):
    v = parse(path.read_bytes())
    sid = source_id(v["source_identity"])
    require(sid == v["expected_source_id"], "frozen source ID differs")
    observations = {}
    for n in range(5):
        if n:
            b = v["blocks"][n - 1]
            observations[("block", b["q"])] = b["block_response"]
            observations[("leaf", b["q"])] = b["leaf_response"]
            for tx in b["transaction_responses"]:
                observations[("tx", b["q"], tx["index"])] = tx
        encoded = audit(sid, assemble(v["source_identity"], observations))
        expected = v["expected_audit_prefixes"][n]
        require(encoded.hex() == expected["bytes_hex"] and hashlib.sha256(encoded).hexdigest() == expected["sha256"], "frozen audit literal differs")
    return {"status": "PASS", "frozen_prefix_vectors": 5, "source_id": sid, "final_public_sha256": expected["sha256"]}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("bundle", type=Path, nargs="?")
    p.add_argument("--reader", default="first")
    p.add_argument("--candidate")
    p.add_argument("--vectors", type=Path)
    p.add_argument("--audit-file", type=Path)
    p.add_argument("--projection-file", type=Path)
    p.add_argument("--output", type=Path)
    a = p.parse_args()
    if a.vectors:
        report = check_vectors(a.vectors)
    else:
        require(a.bundle is not None, "bundle required")
        report, encoded, projected = check(a.bundle, a.reader, a.candidate)
        if a.audit_file:
            require(a.audit_file.read_bytes() == encoded, "candidate audit bytes differ from independent source oracle")
        if a.projection_file:
            require(parse(a.projection_file.read_bytes()) == projected, "candidate projection differs from independent source oracle")
    result = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if a.output:
        with a.output.open("x") as f:
            f.write(result)
    print(result, end="")


if __name__ == "__main__":
    main()
