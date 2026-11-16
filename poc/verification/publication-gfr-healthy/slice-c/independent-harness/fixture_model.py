"""Independent synthetic EC16 source bytes; no candidate imports or QC claim."""
import base64
import hashlib
import json
import struct

NS = 160016
GFR_CODEC = 0x47465201
LEGACY_CODEC = 0xEC160001
COMMIT = "fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171"
IMAGE = "sha256:cfb845e49cb5369f77cd8bbda0b4fe4aad340793cfd130b7f2bb631e59d2db66"


def encoded(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def b64(data):
    return base64.b64encode(data).decode()


def txhash(namespace, payload):
    # A synthetic stable lookup tag. This is NOT an Espresso commitment algorithm.
    return "TX~" + hashlib.sha256(struct.pack("<Q", namespace) + payload).hexdigest()


def source_identity(run="h3-c-independent-fixture"):
    return {"schema": "EC16-SOURCE-1", "official_commit": COMMIT,
            "image_amd64_digest": IMAGE, "protocol_version": "0.3", "api_version": "v1",
            "namespace": NS, "run_id": run, "genesis_block_hash": "BLOCK~h3-fixture-genesis"}


def source_id(identity):
    names = ("schema", "official_commit", "image_amd64_digest", "protocol_version",
             "api_version", "namespace", "run_id", "genesis_block_hash")
    return "EC16:" + hashlib.sha256("".join(str(identity[n]) + "\n" for n in names).encode()).hexdigest()


def block(q, groups):
    """Encode source namespace/transaction tables independently from supplied bytes."""
    raw = bytearray()
    table = bytearray(struct.pack("<I", len(groups)))
    native = []
    for namespace, payloads in groups:
        offsets = []
        body = bytearray()
        for payload in payloads:
            body.extend(payload)
            offsets.append(len(body))
        raw.extend(struct.pack("<I", len(payloads)))
        raw.extend(b"".join(struct.pack("<I", n) for n in offsets))
        raw.extend(body)
        table.extend(struct.pack("<II", namespace, len(raw)))
        for position, payload in enumerate(payloads):
            native.append((namespace, position, payload))
    block_hash = "BLOCK~h3-fixture-genesis" if q == 0 else f"BLOCK~h3-fixture-{q}"
    ns_table = {"bytes": b64(bytes(table))}
    header = {"version": {"Version": {"major": 0, "minor": 3}},
              "fields": {"height": q, "timestamp": q, "ns_table": ns_table,
                         "fixture_annotation": "SYNTHETIC OFFLINE; no HotShot/QC/VID verification"}}
    response = {"header": header, "payload": {"raw_payload": b64(bytes(raw)), "ns_table": ns_table},
                "hash": block_hash, "size": len(raw), "num_transactions": len(native)}
    leaf = {"leaf": {"block_header": header, "block_payload": None, "view_number": q},
            "qc": {"data": {"block_number": q, "leaf_commit": f"COMMIT~synthetic-{q}"},
                   "signatures": None, "view_number": q}}
    transactions = [{"index": index, "block_height": q, "block_hash": block_hash,
                     "namespace": namespace, "pos_in_namespace": position,
                     "hash": txhash(namespace, payload),
                     "transaction": {"namespace": namespace, "payload": b64(payload)}}
                    for index, (namespace, position, payload) in enumerate(native)]
    return response, leaf, transactions


def source_blocks(payloads):
    unique = list(dict.fromkeys(payloads))
    # A different namespace precedes GFR inputs so filtered indices differ from flat indices.
    return [block(0, []), block(1, [(9, [b"unrelated-namespace"]), (NS, unique)])]


def audit(identity, source):
    """Frozen EC16-AUDIT-1: legacy codec remains distinct from HTTP payload protocol."""
    u32 = lambda n: struct.pack("<I", n)
    u64 = lambda n: struct.pack("<Q", n)
    lp = lambda b: u32(len(b)) + b
    out = bytearray(b"EC16-PUBLIC-1\0" + lp(source_id(identity).encode()) + u32(len(source)))
    for q, (b, _, transactions) in enumerate(source):
        out.extend(u64(q) + u64(q * 1_000_000) + lp(b["hash"].encode()) + u32(len(transactions)))
        for tx in transactions:
            payload = base64.b64decode(tx["transaction"]["payload"], validate=True)
            out.extend(u32(tx["index"]) + u64(tx["namespace"]) + u32(tx["pos_in_namespace"])
                       + lp(tx["hash"].encode()) + lp(payload))
        selected = [tx for tx in transactions if tx["namespace"] == NS]
        out.extend(u32(len(selected)))
        for index, tx in enumerate(selected):
            payload = base64.b64decode(tx["transaction"]["payload"], validate=True)
            out.extend(u32(index) + u32(tx["index"]) + u32(LEGACY_CODEC) + lp(payload))
    return bytes(out)


def input_manifest(root, payloads, *, native=False):
    messages = []
    for i, payload in enumerate(payloads):
        path = f"payload-{i:03d}.bin"
        (root / path).write_bytes(payload)
        messages.append({"ordinal": i, "codec": GFR_CODEC, "path": path,
                         "length": len(payload), "sha256": hashlib.sha256(payload).hexdigest()})
    manifest = {"schema": "H3-PUBLICATION-ARTIFACT-1",
                "evidence_kind": "native-regional" if native else "synthetic-adapter",
                "producer_candidate_sha": "a" * 40, "run_id": "h3-input-fixture",
                "result_key_hex": None, "native_evidence": None, "messages": messages}
    if native:
        report = b"SYNTHETIC NATIVE-METADATA TEST; not a valid H1 verifier report\n"
        (root / "report.txt").write_bytes(report)
        manifest["result_key_hex"] = "0001" + "33" * 32 + "44" * 32
        manifest["native_evidence"] = {"h1_revision": "b" * 40, "report_path": "report.txt",
                                       "report_sha256": hashlib.sha256(report).hexdigest()}
    path = root / "input.json"
    path.write_bytes(encoded(manifest))
    return path, manifest
