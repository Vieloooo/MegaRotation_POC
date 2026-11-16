#!/usr/bin/env python3
"""Independent bundle/CLI boundary matrix. Invoke under the host heavy-work flock."""
import argparse
import base64
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

import check_capture as oracle


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def write_manifest(root, manifest, padding=None):
    data = encoded(manifest)
    if padding is not None:
        assert len(data) <= padding
        data += b" " * (padding - len(data))
    (root / "manifest.json").write_bytes(data)


def put(root, name, data):
    (root / name).write_bytes(data)
    return {"path": name, "sha256": hashlib.sha256(data).hexdigest()}


def base_manifest(v):
    return {"schema": "EC16-CAPTURE-1", "capture_kind": "fixture", "identity": v["source_identity"],
            "source_id": v["expected_source_id"], "configuration": {}, "artifacts": [], "records": []}


def record(m, reader, method, path, response, request=None, status=200):
    m["records"].append({"id": len(m["records"]), "reader_id": reader, "source_id": m["source_id"],
                         "method": method, "path": path, "status": status, "request": request,
                         "response": response, "observation": {"wall_ns": 100 + len(m["records"])}})


def build_fixture(root, v):
    root.mkdir()
    m = base_manifest(v)
    ack = put(root, "submit-response.json", encoded(v["blocks"][1]["transaction_responses"][1]["hash"]))
    request = put(root, "submit-request.json", encoded({"namespace": oracle.NS, "payload": "QQ=="}))
    record(m, "first", "POST", "/v1/submit/submit", ack, request)
    records = {}
    for b in v["blocks"]:
        q = b["q"]
        rows = [(f"/v1/availability/block/{q}", b["block_response"]),
                (f"/v1/availability/leaf/{q}", b["leaf_response"])]
        rows += [(f'/v1/availability/transaction/{q}/{t["index"]}/noproof', t) for t in b["transaction_responses"]]
        records[q] = [(path, put(root, f"q{q}-{i}.json", encoded(value))) for i, (path, value) in enumerate(rows)]
    for reader, order in (("first", [2, 0, 1, 3]), ("requery", [0, 1, 2, 3])):
        for q in order:
            for path, ref in reversed(records[q]):
                record(m, reader, "GET", path, ref)
    record(m, "first", "GET", "/v1/availability/transaction/0/0/noproof", put(root, "404.json", b'"not found"'), status=404)
    write_manifest(root, m)
    return m


def replace_file(root, manifest, name, data):
    (root / name).write_bytes(data)
    digest = hashlib.sha256(data).hexdigest()
    for row in manifest["records"]:
        for field in ("request", "response"):
            if row[field] is not None and row[field]["path"] == name:
                row[field]["sha256"] = digest
    for ref in manifest["artifacts"]:
        if ref["path"] == name:
            ref["sha256"] = digest


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--binary", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--candidate", required=True)
    a = p.parse_args()
    repo = Path(__file__).resolve().parents[3]
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip() == a.candidate
    a.output.mkdir(parents=True, exist_ok=False)
    out = a.output.resolve()
    binary = a.binary.resolve()
    v = oracle.parse((Path(__file__).parent / "vectors-v1.json").read_bytes())
    rows = []

    def run(name, bundle, ok=True, reader="first", pause=False, expect_digest=None):
        result_dir = out / (name + "-replay")
        argv = [str(binary), "replay", str(bundle), reader, str(result_dir)]
        if pause:
            argv += ["--pause-ms", "17"]
        process = subprocess.run(argv, capture_output=True, timeout=90)
        (out / (name + ".stdout")).write_bytes(process.stdout)
        (out / (name + ".stderr")).write_bytes(process.stderr)
        (out / (name + ".command.json")).write_bytes(encoded(argv))
        assert (process.returncode == 0) == ok, (name, process.returncode, process.stderr.decode())
        if ok and expect_digest is not None:
            assert process.stdout.decode().strip() == expect_digest
        if not ok:
            assert not result_dir.exists(), f"{name}: rejected bundle emitted partial output"
        rows.append({"case": name, "expected_accept": ok, "returncode": process.returncode, "status": "PASS"})
        return result_dir

    base = out / "fixture-bundle"
    manifest = build_fixture(base, v)
    digest = v["expected_audit_prefixes"][4]["sha256"]
    first = run("fixture-first", base, expect_digest=digest)
    second = run("fixture-paused", base, pause=True, expect_digest=digest)
    requery = run("fixture-requery", base, reader="requery", expect_digest=digest)
    for path, reader in ((first, "first"), (second, "first"), (requery, "requery")):
        report, audit, projection = oracle.check(base, reader)
        assert audit == bytes.fromhex(v["expected_audit_prefixes"][4]["bytes_hex"])
        assert (path / "public-audit.bin").read_bytes() == audit
        assert oracle.parse((path / "public-projection.json").read_bytes()) == projection
        (path / "independent-check.json").write_bytes(encoded(report))
    compare = subprocess.run([str(binary), "compare", str(base), "first", "requery"], capture_output=True, timeout=30)
    assert compare.returncode == 0, compare.stderr
    (out / "compare.stdout").write_bytes(compare.stdout)
    rows.append({"case": "independent-source-payload-and-CLI-compare", "status": "PASS", "prefix_blocks": 4})

    def mutation(name, change, ok=False, expected=None):
        root = out / (name + "-bundle")
        shutil.copytree(base, root)
        m = copy.deepcopy(manifest)
        change(root, m)
        write_manifest(root, m)
        return run(name, root, ok=ok, expect_digest=expected)

    mutation("wall-only", lambda root,m: [r["observation"].update({"wall_ns": 999999999, "arbitrary": "metadata"}) for r in m["records"]], True, digest)
    mutation("unknown-schema", lambda root,m: m.update(schema="EC16-UNKNOWN"))
    mutation("source-mix", lambda root,m: m["identity"].update(run_id="another-run"))
    mutation("fixture-cannot-claim-official", lambda root,m: m.update(capture_kind="official"))
    mutation("nonmonotone-records", lambda root,m: m["records"][1].update(id=0))
    mutation("missing-ref", lambda root,m: (root / "q1-0.json").unlink())
    mutation("bad-file-hash", lambda root,m: (root / "q1-0.json").write_bytes(b"altered"))
    mutation("absolute-ref", lambda root,m: m["records"][0]["response"].update(path=str(base / "submit-response.json")))
    mutation("parent-ref", lambda root,m: m["records"][0]["response"].update(path="../fixture-bundle/submit-response.json"))
    def escape(root,m):
        (root / "submit-response.json").unlink()
        (root / "submit-response.json").symlink_to(base / "submit-response.json")
    mutation("symlink-escape", escape)
    mutation("redirect-response", lambda root,m: m["records"][1].update(status=302))
    mutation("truncated-success", lambda root,m: replace_file(root,m,"q1-0.json",b"{"))
    def tx_change(field, value):
        def change(root,m):
            t=oracle.parse((root / "q1-3.json").read_bytes())
            if field == "payload": t["transaction"][field]=value
            else: t[field]=value
            replace_file(root,m,"q1-3.json",encoded(t))
        return change
    for field,value in (("index",0),("block_height",9),("block_hash","BLOCK~wrong"),("namespace",9),("payload","%%%")):
        mutation("wrong-"+field,tx_change(field,value))
    def logical_gap(root,m):
        m["records"]=[r for r in m["records"] if not (r["reader_id"]=="first" and r["path"]=="/v1/availability/transaction/1/1/noproof")]
    mutation("logical-gap-waits",logical_gap,True,v["expected_audit_prefixes"][1]["sha256"])

    def minimal(root):
        root.mkdir();m=base_manifest(v)
        record(m,"first","GET","/v1/node/block-height",put(root,"hint",b"0"))
        return m
    empty_digest=v["expected_audit_prefixes"][0]["sha256"]
    for which in ("manifest","request","response"):
        root=out/("limit-"+which+"-bundle")
        m=minimal(root)
        if which == "request":
            body=encoded({"namespace":oracle.NS,"payload":"QQ=="})
            ref=put(root,"large",body+b" "*(oracle.MAX_FILE-len(body)))
            response=put(root,"ack",b'"TX~ack"')
            record(m,"first","POST","/v1/submit/submit",response,ref)
        elif which == "response":
            ref=put(root,"large",b"0"+b" "*(oracle.MAX_FILE-1))
            m["records"][0]["response"]=ref
        write_manifest(root,m,oracle.MAX_FILE if which=="manifest" else None)
        run("limit-"+which+"-equal",root,expect_digest=empty_digest)
        target=root/("manifest.json" if which=="manifest" else "large")
        with target.open("ab") as f:f.write(b" ")
        if which != "manifest":
            replace_file(root,m,"large",target.read_bytes());write_manifest(root,m)
        run("limit-"+which+"-over",root,ok=False)

    root=out/"limit-total-bundle";m=minimal(root)
    zeros=b"\0"*oracle.MAX_FILE
    zero_digest=hashlib.sha256(zeros).hexdigest()
    for i in range(15):
        name=f"large-{i:02}"
        with (root/name).open("wb") as f:f.truncate(oracle.MAX_FILE)
        m["artifacts"].append({"path":name,"sha256":zero_digest})
    m["artifacts"].append(copy.deepcopy(m["artifacts"][0]))
    m["artifacts"].append({"path":"last","sha256":"0"*64})
    n=oracle.MAX_BUNDLE-len(encoded(m))-1-15*oracle.MAX_FILE
    assert 0 < n <= oracle.MAX_FILE
    with (root/"last").open("wb") as f:f.truncate(n)
    m["artifacts"][-1]["sha256"]=hashlib.sha256(zeros[:n]).hexdigest()
    write_manifest(root,m)
    run("limit-total-equal-duplicate-ref-counted-once",root,expect_digest=empty_digest)
    # Retain a separate manifest recording the exact passing state before the one-byte mutation.
    (out/"limit-total-equal-manifest.json").write_bytes((root/"manifest.json").read_bytes())
    with (root/"last").open("ab") as f:f.write(b"\0")
    m["artifacts"][-1]["sha256"]=hashlib.sha256(zeros[:n+1]).hexdigest();write_manifest(root,m)
    run("limit-total-over",root,ok=False)
    summary={"status":"PASS","candidate_sha":a.candidate,"cases":rows,"seed":None,
             "binary_sha256":hashlib.sha256(binary.read_bytes()).hexdigest(),
             "note":"Fixtures only. Independent raw payload oracle and all bundle size/path boundaries. No HotShot cryptographic verification."}
    (out/"summary.json").write_bytes(encoded(summary))
    print(json.dumps({"status":"PASS","cases":len(rows),"output":str(out)},indent=2))


if __name__=="__main__":
    main()
