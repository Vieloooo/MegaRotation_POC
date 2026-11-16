#!/usr/bin/env python3
"""Independent EC16 source/capture oracle; uses no candidate library or output to derive expected bytes."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import struct

COMMIT = "fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171"
IMAGE = "sha256:cfb845e49cb5369f77cd8bbda0b4fe4aad340793cfd130b7f2bb631e59d2db66"
INDEX = "sha256:af73f55630690caa35aa0bc015d60dc03333791f966fdcc07b0798e33d4f768e"
CONFIG = "sha256:b376be602febaa35fd38c1f4bc1144fb79017bfed274a4a26ed90bf052bfd06f"
CODEC = 0xEC160001
GFR_PUBLIC = 1195790849
NS = 160016
MAX_FILE = 16 * 1024 * 1024
MAX_BUNDLE = 256 * 1024 * 1024
INPUT_LIMIT = 65536
INPUT_MODES = ("synthetic-adapter", "native-regional")


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


def exact_fields(value, *keys):
    require(type(value) is dict and set(value) == set(keys), "unexpected publication input fields")


def matches(value, pattern):
    return isinstance(value, str) and re.fullmatch(pattern, value) is not None


def input_path(name):
    require(isinstance(name, str) and "\\" not in name and "\0" not in name, "invalid input path")
    components = name.split("/")
    require(all(component not in ("", ".", "..") for component in components), "input path escapes manifest directory")
    return components


def sha(data):
    return hashlib.sha256(data).hexdigest()


def parse_payload_manifest(raw):
    require(len(raw) <= INPUT_LIMIT, "payload manifest exceeds bound")
    value = parse(raw.decode("utf-8"))
    exact_fields(value, "schema", "evidence_kind", "producer_candidate_sha", "run_id",
                 "result_key_hex", "native_evidence", "messages")
    require(value["schema"] == "H3-PUBLICATION-ARTIFACT-1" and value["evidence_kind"] in INPUT_MODES,
            "publication input schema/kind mismatch")
    require(matches(value["producer_candidate_sha"], r"[a-f0-9]{40}") and matches(value["run_id"], r"[A-Za-z0-9_-]{1,64}"),
            "invalid producer identity")
    key, native = value["result_key_hex"], value["native_evidence"]
    require(key is None or matches(key, r"0001[a-f0-9]{128}"), "invalid canonical ResultKey")
    if value["evidence_kind"] == "native-regional":
        require(key is not None and native is not None, "missing native input provenance")
    if native is not None:
        exact_fields(native, "h1_revision", "report_path", "report_sha256")
        input_path(native["report_path"])
        require(matches(native["h1_revision"], r"[a-f0-9]{40}") and matches(native["report_sha256"], r"[a-f0-9]{64}"),
                "invalid native evidence identifiers")
    messages = value["messages"]
    require(type(messages) is list and 1 <= len(messages) <= 11, "input message count bound")
    declared = {}
    total = 0
    for ordinal, item in enumerate(messages):
        exact_fields(item, "ordinal", "codec", "path", "length", "sha256")
        require(integer(item["ordinal"]) == ordinal and integer(item["codec"]) == GFR_PUBLIC, "wrong ordinal or GFR codec")
        input_path(item["path"])
        length = integer(item["length"], INPUT_LIMIT)
        require(length > 0 and matches(item["sha256"], r"[a-f0-9]{64}"), "invalid declared length or hash")
        total += length
        require(total <= INPUT_LIMIT, "aggregate manifest payload capacity")
        attributes = (length, item["sha256"])
        require(declared.setdefault(item["path"], attributes) == attributes, "conflicting payload path declarations")
    if native is not None:
        require(native["report_path"] not in declared, "native report path conflicts with payload")
    return value


def bounded_input(root_fd, name, limit):
    components = input_path(name)
    parent = os.dup(root_fd)
    try:
        for part in components[:-1]:
            child_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parent)
            os.close(parent)
            parent = child_fd
        fd = os.open(components[-1], os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW, dir_fd=parent)
        with os.fdopen(fd, "rb") as file:
            info = os.fstat(fd)
            require(stat.S_ISREG(info.st_mode) and info.st_size <= limit, "nonregular or oversized input file")
            data = file.read(limit + 1)
            require(len(data) == info.st_size and len(data) <= limit, "input size changed during read")
            return data
    finally:
        os.close(parent)


def load_payload_manifest(path):
    """Independently read declared input files and return exact immutable bytes."""
    path = Path(path)
    root_fd = os.open(path.parent.resolve(strict=True), os.O_RDONLY | os.O_DIRECTORY)
    try:
        raw = bounded_input(root_fd, path.name, INPUT_LIMIT)
        value = parse_payload_manifest(raw)
        unique = {}
        for item in value["messages"]:
            name = item["path"]
            require(name != path.name, "payload collides with input manifest")
            if name not in unique:
                data = bounded_input(root_fd, name, item["length"])
                require(len(data) == item["length"] and sha(data) == item["sha256"], "wrong payload length or SHA-256")
                unique[name] = data
        native = value["native_evidence"]
        report = None
        if native is not None:
            require(native["report_path"] != path.name, "report collides with input manifest")
            report = bounded_input(root_fd, native["report_path"], INPUT_LIMIT)
            require(sha(report) == native["report_sha256"], "native report SHA-256 mismatch")
        return value, raw, tuple(unique[item["path"]] for item in value["messages"]), report
    finally:
        os.close(root_fd)


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
    with manifest_path.open("rb") as file:
        raw = file.read(MAX_FILE + 1)
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
        file_path = root / name
        input_copy = (manifest["configuration"].get("input_mode") in INPUT_MODES
                      and name.startswith("publication-input-"))
        if input_copy:
            require(not file_path.is_symlink(), "publication input copy is a symlink")
        resolved = file_path.resolve(strict=True)
        require(resolved.is_relative_to(root) and resolved.is_file() and resolved != manifest_path, "recorded path escapes bundle")
        if resolved not in files:
            length = resolved.stat().st_size
            maximum = INPUT_LIMIT if input_copy else MAX_FILE
            require(length <= maximum and total + length <= MAX_BUNDLE, "raw file/bundle capacity")
            with resolved.open("rb") as file:
                contents = file.read(length + 1)
            require(len(contents) == length, "raw file changed while read")
            files[resolved] = contents
            total += length
        require(hashlib.sha256(files[resolved]).hexdigest() == ref["sha256"], "raw file SHA256 mismatch")
        by_path[name] = files[resolved]
    return manifest, by_path, total, hashlib.sha256(raw).hexdigest()


def publication_positions(blocks, tx_hash, payload):
    return [
        {"q": block["q"], "source_transaction_index": row["index"],
         "pos_in_namespace": row["pos_in_namespace"], "message_index": projected_index}
        for block in blocks
        for projected_index, row in enumerate(tx for tx in block["transactions"] if tx["namespace"] == NS)
        if row["hash"] == tx_hash and row["bytes"] == payload
    ]


def check_payload_identity(manifest, files, blocks, payload_manifest):
    config = manifest["configuration"]
    mode = config.get("input_mode", "marker-default")
    require(mode in ("marker-default", *INPUT_MODES), "unknown capture input mode")
    if mode == "marker-default":
        require(payload_manifest is None and "input_identity" not in config, "capture has no declared publication manifest")
        return {"input_mode": mode}

    identity = config["input_identity"]
    exact_fields(identity, "manifest_path", "manifest_sha256", "producer_candidate_sha", "producer_run_id",
                 "runner_candidate_sha", "result_key_hex", "native_evidence", "native_verification", "messages", "repeat")
    artifact_names = {item["path"] for item in manifest["artifacts"]}

    def copied_file(name):
        require(name in artifact_names and name in files, "missing publication input copy")
        data = files[name]
        require(len(data) <= INPUT_LIMIT, "oversized publication input copy")
        return data

    require(identity["manifest_path"] == "publication-input-manifest.json", "incorrect manifest artifact path")
    raw = copied_file(identity["manifest_path"])
    source = parse_payload_manifest(raw)
    require(identity["manifest_sha256"] == sha(raw) and source["evidence_kind"] == mode, "input manifest hash or mode mismatch")
    require(identity["producer_candidate_sha"] == source["producer_candidate_sha"]
            and identity["producer_run_id"] == source["run_id"]
            and identity["result_key_hex"] == source["result_key_hex"], "producer identity mismatch")
    require(matches(identity["runner_candidate_sha"], r"[a-f0-9]{40}")
            and identity["runner_candidate_sha"] == config["candidate_sha"], "runner revision identity mismatch")
    require(identity["native_verification"] == "NOT_PERFORMED", "invalid native verification disposition")
    native = source["native_evidence"]
    report = None
    if native is None:
        require(identity["native_evidence"] is None, "unexpected native evidence metadata")
    else:
        report_name = "publication-input-native-report.bin"
        require(identity["native_evidence"] == dict(native, copied_path=report_name), "native evidence mapping mismatch")
        report = copied_file(report_name)
        require(sha(report) == native["report_sha256"], "copied native report SHA-256 mismatch")

    message_records = identity["messages"]
    messages = source["messages"]
    require(type(message_records) is list and len(message_records) == len(messages), "input copy mapping incomplete")
    submits = [record for record in manifest["records"] if record["path"] == "/v1/submit/submit"]
    require(len(submits) == len(messages) + 1, "missing or extra submit records")

    def require_post(record, payload):
        require(record["method"] == "POST" and record["reader_id"] == "first", "unexpected manifest submit method or reader")
        request = parse(files[record["request"]["path"]])
        exact_fields(request, "namespace", "payload")
        require(integer(request["namespace"]) == NS and decoded(request["payload"]) == payload,
                "input bytes or namespace differ from exact POST")
        if record["status"] == 200:
            ack = parse(files[record["response"]["path"]])
            require(matches(ack, r"TX~[A-Za-z0-9_-]+"), "invalid actual manifest submit ACK")
            return ack
        require(record["status"] is None or 400 <= integer(record["status"]) <= 599, "invalid submit HTTP status")
        return None

    payloads = []
    verified_rows = []
    for item, saved, submit_record in zip(messages, message_records, submits):
        ordinal = item["ordinal"]
        name = f"publication-input-{ordinal:03d}.bin"
        payload = copied_file(name)
        require(len(payload) == item["length"] and sha(payload) == item["sha256"], "wrong copied payload bytes")
        ack_hash = require_post(submit_record, payload)
        require(ack_hash is not None, "manifest input lacks an acknowledged submit")
        occurrences = publication_positions(blocks, ack_hash, payload)
        require(occurrences, "manifest publication absent from complete independently reconstructed prefix")
        expected = dict(item, copied_path=name, submit_record_id=submit_record["id"], ack_hash=ack_hash,
                        occurrences=occurrences, byte_equal=True)
        require(json.dumps(saved, sort_keys=True) == json.dumps(expected, sort_keys=True), "ordinal/submit/readback mapping mismatch")
        payloads.append(payload)
        verified_rows.append(expected)

    repeat = submits[-1]
    repeat_hash = require_post(repeat, payloads[0])
    require(repeat_hash is None or repeat_hash == verified_rows[0]["ack_hash"], "repeat ACK changed transaction identity")
    expected_repeat = {
        "ordinal": 0, "submit_record_id": repeat["id"], "status": repeat["status"], "ack_hash": repeat_hash,
        "occurrences": verified_rows[0]["occurrences"] if repeat_hash is not None else [], "byte_equal": repeat_hash is not None,
    }
    require(json.dumps(identity["repeat"], sort_keys=True) == json.dumps(expected_repeat, sort_keys=True), "repeat submit/readback mapping mismatch")
    missing_path = f"/v1/availability/transaction/0/{len(blocks[0]['transactions'])}/noproof"
    require(any(row["reader_id"] == "first" and row["method"] == "GET" and row["path"] == missing_path
                and row["status"] == 404 for row in manifest["records"]), "missing actual genesis out-of-bounds 404")
    if payload_manifest is not None:
        require(load_payload_manifest(payload_manifest) == (source, raw, tuple(payloads), report),
                "original input manifest or files differ from captured copies")

    return {
        "input_mode": mode, "publication_scope": "byte-publication-only",
        "input_identity": dict(identity, messages=verified_rows, repeat=expected_repeat),
        "byte_equality": True,
        "official_timing": {"proposed_at": "Unknown", "finalized_at": "Unknown", "live_at_proposal": "Unknown"},
        "pe_exports": "Unsupported",
    }


def check(root, reader_id, expected_candidate=None, payload_manifest=None):
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
    report.update(check_payload_identity(m, files, blocks, payload_manifest))
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
    p.add_argument("--payload-manifest", type=Path)
    a = p.parse_args()
    if a.vectors:
        require(a.payload_manifest is None, "publication input manifest requires a capture bundle")
        report = check_vectors(a.vectors)
    else:
        require(a.bundle is not None, "bundle required")
        report, encoded, projected = check(a.bundle, a.reader, a.candidate, a.payload_manifest)
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
