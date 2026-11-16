#!/usr/bin/env python3
"""Prepared independent H3-C offline tests. No node, network, native proof or launch claim."""
import argparse
import base64
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import traceback
from unittest import mock

import fixture_model as model

sys.dont_write_bytecode = True
MARKERS = (b"EC16:non-cryptographic-publication:A:v1", b"EC16:non-cryptographic-publication:B:v1")
RUNNER_SCHEMA = "H3-C-INDEPENDENT-OFFLINE-1"


def load_module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def require(condition, explanation):
    if not condition:
        raise AssertionError(explanation)


def rehash_ref(bundle, manifest, name, data):
    """Mutate a recording while retaining its outer file-integrity checksum."""
    (bundle / name).write_bytes(data)
    digest = hashlib.sha256(data).hexdigest()
    for ref in manifest["artifacts"]:
        if ref["path"] == name:
            ref["sha256"] = digest
    for record in manifest["records"]:
        for key in ("request", "response"):
            ref = record[key]
            if ref is not None and ref["path"] == name:
                ref["sha256"] = digest


class QuietProcesses:
    def __init__(self):
        self.checks = 0

    def check(self):
        self.checks += 1
        require(self.checks <= 128, "unexpected unbounded polling in synthetic source")


def mock_recorder(capture, root, payloads):
    """Use candidate file recorder, replacing every HTTP operation with independent source bytes."""
    expected = model.source_blocks(payloads)

    class SyntheticRecorder(capture.Recorder):
        def http(self, reader, method, path, *, service="api", body=None):
            require(service == "api", "publication seam attempted non-API service access")
            rid = len(self.records)
            request = None if body is None else self.file(f"raw/{rid:06d}.request", body)
            status = 200
            if method == "POST" and path == "/v1/submit/submit":
                value = json.loads(body)
                require(value["namespace"] == model.NS, "submission namespace changed")
                payload = base64.b64decode(value["payload"], validate=True)
                require(payload in payloads, "submitted payload differs from independently fixed input")
                value = model.txhash(model.NS, payload)
            elif path == "/v1/node/block-height":
                value = 3
            else:
                block = re.fullmatch(r"/v1/availability/(block|leaf)/(\d+)", path)
                tx = re.fullmatch(r"/v1/availability/transaction/(\d+)/(\d+)/noproof", path)
                if block and int(block[2]) < len(expected):
                    value = expected[int(block[2])][0 if block[1] == "block" else 1]
                elif tx and int(tx[1]) < len(expected) and int(tx[2]) < len(expected[int(tx[1])][2]):
                    value = expected[int(tx[1])][2][int(tx[2])]
                else:
                    status, value = 404, "synthetic unavailable position"
            response = self.file(f"raw/{rid:06d}.response", model.encoded(value))
            self.records.append({"id": rid, "reader_id": reader, "method": method, "path": path,
                                 "request": request, "status": status, "response": response,
                                 "observation": {"classification": "SYNTHETIC_NO_NETWORK"}})
            return capture.Response(status, model.encoded(value), rid)

    return SyntheticRecorder(root), expected


class Matrix:
    def __init__(self, args):
        self.args = args
        self.repo = args.repo.resolve()
        self.out = args.output.resolve()
        self.out.mkdir(parents=True, exist_ok=False)
        self.capture_path = self.repo / "poc/scenarios/espresso-compatibility/capture.py"
        self.checker_path = self.repo / "poc/verification/espresso-compatibility/check_capture.py"
        self.capture = load_module(self.capture_path, "h3_c_candidate_capture")
        self.checker = load_module(self.checker_path, "h3_c_candidate_checker")
        self.rows = []
        self.sequence = 0

    def fresh(self, label):
        self.sequence += 1
        path = self.out / f"{self.sequence:03d}-{label}"
        path.mkdir()
        return path

    def case(self, name, fn):
        try:
            detail = fn()
            self.rows.append({"case": name, "status": "PASS", "detail": detail})
        except BaseException as error:
            self.rows.append({"case": name, "status": "FAIL", "exception": type(error).__name__,
                              "detail": str(error), "traceback": traceback.format_exc()})
        print(json.dumps(self.rows[-1], sort_keys=True), flush=True)

    def loaders_accept(self, name, payloads, native=False, pad_manifest=None, report_bytes=None):
        root = self.fresh(name)
        path, manifest = model.input_manifest(root, payloads, native=native)
        if report_bytes is not None:
            (root / "report.txt").write_bytes(report_bytes)
            manifest["native_evidence"]["report_sha256"] = hashlib.sha256(report_bytes).hexdigest()
            path.write_bytes(model.encoded(manifest))
        if pad_manifest is not None:
            raw = path.read_bytes()
            require(len(raw) <= pad_manifest, "test manifest already exceeds requested padding")
            path.write_bytes(raw + b" " * (pad_manifest - len(raw)))
        original = path.read_bytes()
        for module in (self.capture, self.checker):
            loaded = module.load_payload_manifest(path)
            require(isinstance(loaded, tuple) and len(loaded) == 4, "loader seam shape changed")
            value, raw, contents, report = loaded
            require(value == manifest and raw == original, "loader rewrote manifest")
            require(isinstance(contents, tuple) and contents == tuple(payloads), "ordered immutable payloads differ")
            expect_report = (root / "report.txt").read_bytes() if native else None
            require(report == expect_report, "native report copy mismatch")
        return {"messages": len(payloads), "payload_bytes": sum(map(len, payloads)),
                "classification": "metadata/input handling only; no native proof check"}

    def loaders_reject(self, name, mutate, *, native=False, prelaunch=False):
        root = self.fresh(name)
        path, manifest = model.input_manifest(root, [b"synthetic-payload"], native=native)
        mutate(root, path, manifest)
        rejected = []
        for module in (self.capture, self.checker):
            try:
                module.load_payload_manifest(path)
            except Exception as error:
                rejected.append(type(error).__name__)
            else:
                raise AssertionError("invalid manifest accepted by " + module.__name__)
        if prelaunch:
            run = root / "RUN_MUST_REMAIN_EMPTY"
            run.mkdir()
            argv = [sys.executable, str(self.capture_path), "--payload-manifest", str(path),
                    "--image", str(root / "IMAGE_MUST_NOT_EXIST"), "--run", str(run),
                    "--run-id", "h3-invalid-input", "--candidate", self.args.candidate,
                    "--replay-binary", str(root / "BINARY_MUST_NOT_EXIST")]
            process = subprocess.run(argv, capture_output=True, timeout=10)
            (root / "command.json").write_bytes(model.encoded(argv))
            (root / "stdout").write_bytes(process.stdout)
            (root / "stderr").write_bytes(process.stderr)
            require(process.returncode != 0 and not any(run.iterdir()), "invalid manifest touched run/launch path")
            require(b"IMAGE_MUST_NOT_EXIST" not in process.stderr, "image inspected before input rejection")
            require(b"BINARY_MUST_NOT_EXIST" not in process.stderr, "binary inspected before input rejection")
        return {"rejected_by_both": rejected, "prelaunch_sentinel": prelaunch}

    def immutable_copy(self):
        root = self.fresh("immutable-copy")
        inputs = root / "inputs"; inputs.mkdir()
        path, manifest = model.input_manifest(inputs, [b"frozen-original"], native=True)
        loaded = self.capture.load_payload_manifest(path)
        (inputs / manifest["messages"][0]["path"]).write_bytes(b"changed-after-load")
        bundle = root / "bundle"; bundle.mkdir()
        record = self.capture.Recorder(bundle)
        identity = self.capture.copy_payload_inputs(record, loaded, self.args.candidate)
        require(identity["native_verification"] == "NOT_PERFORMED", "metadata promoted to native verification")
        require(identity["producer_candidate_sha"] == "a" * 40, "producer revision overwritten")
        require(identity["runner_candidate_sha"] == self.args.candidate, "runner revision lost")
        require((bundle / identity["manifest_path"]).read_bytes() == loaded[1], "manifest copy changed")
        require((bundle / identity["messages"][0]["copied_path"]).read_bytes() == b"frozen-original",
                "copy reread mutable input after validation")
        native = identity["native_evidence"]
        require((bundle / native["copied_path"]).read_bytes() == loaded[3], "report copy changed")
        return {"immutable_payload": True, "native_verification": "NOT_PERFORMED"}

    def synthetic_capture(self, name, payloads, *, marker=False, native=False):
        root = self.fresh(name)
        inputs = root / "inputs"; inputs.mkdir()
        path, manifest = model.input_manifest(inputs, payloads, native=native)
        bundle = root / "bundle"; bundle.mkdir()
        record, source = mock_recorder(self.capture, bundle, payloads)
        identity = None
        if not marker:
            identity = self.capture.copy_payload_inputs(record, self.capture.load_payload_manifest(path), self.args.candidate)
        # Initial source prefix is independently supplied as before the real observe seam.
        for part in ("block", "leaf"):
            record.http("first", "GET", f"/v1/availability/{part}/0")
        processes = QuietProcesses()
        with mock.patch.object(self.capture.time, "sleep", lambda _: None), \
             mock.patch.object(self.capture.http.client, "HTTPConnection", side_effect=AssertionError("real HTTP forbidden")):
            observations = self.capture.observe_publications(record, processes, source[0][0], source[0][2],
                payloads=tuple(payloads), input_identity=identity)
        posts = [r for r in record.records if r["method"] == "POST"]
        actual = [base64.b64decode(json.loads((bundle / r["request"]["path"]).read_bytes())["payload"], validate=True)
                  for r in posts]
        require(actual == list(payloads) + [payloads[0]], "submit order/duplicates/repeat changed")
        if marker:
            require(tuple(self.capture.MARKERS) == MARKERS and tuple(payloads) == MARKERS, "old markers changed")
            require(len(observations["distinct_submit_ids"]) == 2, "marker submit identity changed")
        else:
            require(identity["native_verification"] == "NOT_PERFORMED", "unperformed native check claimed")
            unique = list(dict.fromkeys(payloads))
            for i, msg in enumerate(identity["messages"]):
                for key, value in manifest["messages"][i].items():
                    require(msg[key] == value, f"input field changed: {key}")
                selected = unique.index(payloads[i])
                expected = [{"q": 1, "source_transaction_index": selected + 1,
                             "pos_in_namespace": selected, "message_index": selected}]
                require(msg["occurrences"] == expected, "native/filtered position or duplicate mapping changed")
                require(msg["byte_equal"] is True and msg["ack_hash"] == model.txhash(model.NS, payloads[i]),
                        "submitted bytes/hash mismatch")
            require(identity["repeat"]["ordinal"] == 0 and identity["repeat"]["byte_equal"] is True,
                    "repeat identity missing")
        source_identity = model.source_identity()
        sid = model.source_id(source_identity)
        for record_item in record.records:
            record_item["source_id"] = sid
        configuration = {"candidate_sha": self.args.candidate,
                         "input_mode": "marker-default" if marker else manifest["evidence_kind"],
                         "classification": "SYNTHETIC_OFFLINE_NO_NODE_NO_QC_NO_NATIVE_PROOF"}
        if identity is not None:
            configuration["input_identity"] = identity
        capture_manifest = {"schema": "EC16-CAPTURE-1", "capture_kind": "fixture",
                            "identity": source_identity, "source_id": sid,
                            "configuration": configuration, "records": record.records,
                            "artifacts": record.artifacts}
        (bundle / "manifest.json").write_bytes(model.encoded(capture_manifest))
        (root / "expected-audit.bin").write_bytes(model.audit(source_identity, source))
        return root, path, capture_manifest

    def checker_cli(self, root, manifest_path, *, positive=True, allow_incomplete=False, reader="first"):
        argv = [sys.executable, str(self.checker_path), str(root / "bundle"), "--reader", reader,
                "--candidate", self.args.candidate]
        if manifest_path is not None:
            argv += ["--payload-manifest", str(manifest_path)]
        if positive:
            argv += ["--audit-file", str(root / "expected-audit.bin")]
        process = subprocess.run(argv, capture_output=True, timeout=15)
        (root / f"checker-{reader}-command.json").write_bytes(model.encoded(argv))
        (root / f"checker-{reader}.stdout").write_bytes(process.stdout)
        (root / f"checker-{reader}.stderr").write_bytes(process.stderr)
        if positive:
            require(process.returncode == 0, process.stderr.decode(errors="replace"))
            report = json.loads(process.stdout)
            require(report["public_sha256"] == hashlib.sha256((root / "expected-audit.bin").read_bytes()).hexdigest(),
                    "checker changed source-audit codec or bytes")
            require(report["complete_blocks"] == 2, "complete synthetic source prefix not read")
            return report
        if process.returncode == 0 and allow_incomplete:
            report = json.loads(process.stdout)
            identity = report.get("input_identity")
            require(isinstance(identity, dict), "incomplete success lacks explicit input identity status")
            require(any(not msg.get("byte_equal") or not msg.get("occurrences")
                        for msg in identity.get("messages", [])), "incomplete input falsely fully confirmed")
        else:
            require(process.returncode != 0, "corrupted input/readback accepted")
        return {"returncode": process.returncode, "no_complete_identity_claim": True}

    def readback_positive(self, name, payloads, **kwargs):
        root, path, _ = self.synthetic_capture(name, payloads, **kwargs)
        manifest = None if kwargs.get("marker") else path
        first = self.checker_cli(root, manifest)
        second = self.checker_cli(root, manifest, reader="requery")
        require(first["public_sha256"] == second["public_sha256"], "independent reader public projections differ")
        return {"public_sha256": first["public_sha256"], "first_complete_blocks": first["complete_blocks"],
                "requery_complete_blocks": second["complete_blocks"], "synthetic_only": True}

    def mutation(self, name, change, *, native=False, external=True, incomplete=False, reader="first"):
        root, path, manifest = self.synthetic_capture(name, [b"source-fixed-A", b"source-fixed-B"], native=native)
        change(root, root / "bundle", manifest)
        (root / "bundle/manifest.json").write_bytes(model.encoded(manifest))
        return self.checker_cli(root, path if external else None, positive=False, allow_incomplete=incomplete, reader=reader)

    def clone_manifest(self):
        root, _, _ = self.synthetic_capture("cloned-input-path", [b"clone-A", b"clone-B"])
        shutil.copytree(root / "inputs", root / "cloned-inputs")
        return self.checker_cli(root, root / "cloned-inputs/input.json")

    def legacy_vectors(self):
        argv = [sys.executable, str(self.checker_path), "--vectors",
                str(self.repo / "poc/verification/espresso-compatibility/vectors-v1.json")]
        p = subprocess.run(argv, capture_output=True, timeout=15)
        require(p.returncode == 0, p.stderr.decode(errors="replace"))
        report = json.loads(p.stdout)
        require(report["status"] == "PASS" and report["frozen_prefix_vectors"] == 5, "old marker/audit vectors changed")
        return report

    def cli_surface(self):
        for path in (self.capture_path, self.checker_path):
            result = subprocess.run([sys.executable, str(path), "--help"], capture_output=True, timeout=10)
            require(result.returncode == 0 and b"--payload-manifest" in result.stdout,
                    "frozen manifest CLI option missing")
        return {"capture_and_checker_manifest_flag": True}

    def wrapper_private_tmp_argv(self):
        """Execute only the wrapper's argv construction; cargo/image/timeout are task-local mocks."""
        root = self.fresh("wrapper-private-tmp")
        repo = root / "mock-repo"; scenario = repo / "poc/scenarios/espresso-compatibility"
        scenario.mkdir(parents=True); (repo / ".build").mkdir()
        source = self.repo / "poc/scenarios/espresso-compatibility/run-local.sh"
        shutil.copy2(source, scenario / "run-local.sh")
        shutil.copy2(self.capture_path, scenario / "capture.py")
        (scenario / "prepare-image.py").write_text("# test stand-in, intercepted by python3 shim\n")
        stubs = root / "mock-bin"; stubs.mkdir()
        def executable(name, text):
            path = stubs / name; path.write_text(text); path.chmod(0o755)
        executable("git", "#!/bin/sh\ncase \"$1\" in rev-parse) printf '%s\\n' \"$H3_EXPECTED_SHA\";; status) exit 0;; *) exit 77;; esac\n")
        executable("rustc", "#!/bin/sh\nprintf '%s\\n' 'rustc 1.95.0 (synthetic wrapper seam)'\n")
        executable("cargo", "#!/bin/sh\nexit 0\n")
        executable("flock", "#!/bin/sh\nexit 0\n")
        executable("timeout", "#!/bin/sh\nexit 0\n")
        executable("python3", "#!/bin/sh\ncase \"$1\" in */prepare-image.py) printf '{}\\n';; *) exec /usr/bin/python3 \"$@\";; esac\n")
        env = dict(os.environ, PATH=str(stubs) + ":/usr/bin:/bin", ESPRESSO16_RUST_BIN=str(stubs),
                   CARGO_HOME=str(root / "mock-cargo"), H3_EXPECTED_SHA=self.args.candidate)
        with tempfile.TemporaryDirectory(prefix="h3-c-wrapper-input-", dir="/tmp") as temporary:
            inputs = Path(temporary)
            path, _ = model.input_manifest(inputs, [b"wrapper-input"])
            unusual = inputs / "manifest with spaces;$literal.json"; path.rename(unusual)
            argv = ["bash", str(scenario / "run-local.sh"), "wrapper-fixture", self.args.candidate,
                    "--payload-manifest", str(unusual)]
            process = subprocess.run(argv, env=env, capture_output=True, timeout=15)
            (root / "command.json").write_bytes(model.encoded(argv))
            (root / "stdout").write_bytes(process.stdout); (root / "stderr").write_bytes(process.stderr)
            require(process.returncode == 0, process.stderr.decode(errors="replace"))
            launch = json.loads((repo / ".build/artifacts/wrapper-fixture/launch-command.json").read_bytes())
            flag = launch.index("--payload-manifest")
            require(launch[flag+1] == "/tmp/ec16-publication-input/" + unusual.name, "manifest argv was expanded or mapped outside guest input")
            bind = [i for i in range(len(launch)-2) if launch[i] == "--ro-bind"
                    and launch[i+1] == str(inputs) and launch[i+2] == "/tmp/ec16-publication-input"]
            require(len(bind) == 1, "input directory not separately read-only bound")
            private_tmp = [i for i in range(len(launch)-1) if launch[i:i+2] == ["--tmpfs", "/tmp"]]
            require(len(private_tmp) == 1 and private_tmp[0] < bind[0], "private /tmp overwritten or bound before tmpfs")
            require(not any(launch[i:i+3] == ["--ro-bind", "/tmp", "/tmp"] for i in range(len(launch)-2)),
                    "host /tmp exposed over private guest /tmp")
            (root / "input-manifest.json").write_bytes(unusual.read_bytes())
        return {"classification": "WRAPPER_ARGV_ONLY_ALL_LAUNCH_TOOLS_MOCKED", "private_tmp_preserved": True}

    def run(self):
        self.case("frozen-cli-surface", self.cli_surface)
        self.case("wrapper-guest-input-and-private-tmp", self.wrapper_private_tmp_argv)
        self.case("legacy-audit-vectors", self.legacy_vectors)
        self.case("load-one", lambda: self.loaders_accept("one", [b"A"]))
        self.case("load-eleven-duplicate", lambda: self.loaders_accept("eleven", [b"A"] * 11))
        self.case("aggregate-equality", lambda: self.loaders_accept("aggregate-equal", [b"A" * 32768, b"B" * 32768]))
        self.case("manifest-equality", lambda: self.loaders_accept("manifest-equal", [b"A"], pad_manifest=65536))
        self.case("native-report-equality-metadata-only", lambda: self.loaders_accept("report-equal", [b"A"], native=True, report_bytes=b"R" * 65536))
        self.case("immutable-input-copy", self.immutable_copy)

        def rewrite(field, value, message=False):
            def mutate(root, path, m):
                target = m["messages"][0] if message else m
                target[field] = value; path.write_bytes(model.encoded(m))
            return mutate
        invalids = [
            ("schema", rewrite("schema", "unknown")), ("unknown-root-field", rewrite("extra", 1)),
            ("bool-length", rewrite("length", True, True)), ("float-length", rewrite("length", 1.0, True)),
            ("bool-codec", rewrite("codec", True, True)), ("bool-ordinal", rewrite("ordinal", False, True)),
            ("wrong-codec", rewrite("codec", model.GFR_CODEC + 1, True)),
            ("ordinal-gap", rewrite("ordinal", 1, True)), ("unknown-message-field", rewrite("extra", 1, True)),
            ("invalid-run-id", rewrite("run_id", "contains space")),
            ("uppercase-sha", rewrite("producer_candidate_sha", "A" * 40)),
            ("wrong-length", rewrite("length", 1, True)), ("wrong-hash", rewrite("sha256", "0" * 64, True)),
            ("absolute-path", rewrite("path", "/not-an-input", True)),
            ("parent-path", rewrite("path", "../not-an-input", True)),
            ("missing-path", rewrite("path", "missing.bin", True)),
            ("empty-list", rewrite("messages", [])),
        ]
        for name, mutate in invalids:
            self.case("reject-" + name, lambda name=name, mutate=mutate:
                      self.loaders_reject(name, mutate, prelaunch=name in ("schema", "bool-length", "wrong-hash")))
        self.case("reject-duplicate-json-key", lambda: self.loaders_reject("duplicate-key",
            lambda r,p,m: p.write_bytes(b'{"schema":"H3-PUBLICATION-ARTIFACT-1",' + model.encoded(m)[1:]), prelaunch=True))
        self.case("reject-nonfinite", lambda: self.loaders_reject("nonfinite",
            lambda r,p,m: p.write_bytes(model.encoded(m).replace(b'"length":17', b'"length":NaN'))))
        def symlink(r, p, m):
            original = r / m["messages"][0]["path"]
            saved = r / "saved.bin"; original.rename(saved); original.symlink_to(saved.name)
        self.case("reject-symlink", lambda: self.loaders_reject("symlink", symlink, prelaunch=True))
        def too_many(r,p,m):
            m["messages"] = [dict(m["messages"][0], ordinal=i) for i in range(12)]; p.write_bytes(model.encoded(m))
        self.case("reject-twelve", lambda: self.loaders_reject("twelve", too_many))
        self.case("reject-manifest-over", lambda: self.loaders_reject("manifest-over",
            lambda r,p,m: p.write_bytes(p.read_bytes() + b" " * (65537-len(p.read_bytes())))))
        def file_size(length):
            def mutate(r,p,m):
                data = b"X" * length; (r / m["messages"][0]["path"]).write_bytes(data)
                m["messages"][0].update(length=length, sha256=hashlib.sha256(data).hexdigest()); p.write_bytes(model.encoded(m))
            return mutate
        self.case("reject-empty-payload", lambda: self.loaders_reject("empty-payload", file_size(0)))
        self.case("reject-payload-over", lambda: self.loaders_reject("payload-over", file_size(65537)))
        self.case("reject-aggregate-over", lambda: self.loaders_reject("aggregate-over",
            lambda r,p,m: model.input_manifest(r,[b"A"*32768,b"B"*32769])))
        def report_over(r,p,m):
            data=b"R"*65537; (r/"report.txt").write_bytes(data)
            m["native_evidence"]["report_sha256"]=hashlib.sha256(data).hexdigest(); p.write_bytes(model.encoded(m))
        self.case("reject-native-report-over", lambda: self.loaders_reject("report-over", report_over, native=True))
        def native_extra(r,p,m):
            m["native_evidence"]["extra"]=0; p.write_bytes(model.encoded(m))
        self.case("reject-native-unknown-field", lambda: self.loaders_reject("native-extra", native_extra, native=True))
        def native_key(r,p,m):
            m["result_key_hex"] = "0002" + "33"*32 + "44"*32; p.write_bytes(model.encoded(m))
        self.case("reject-native-key-version", lambda: self.loaders_reject("native-key-version", native_key, native=True))
        self.case("marker-default-observe-and-readback", lambda: self.readback_positive("marker-default", MARKERS, marker=True))
        self.case("manifest-distinct-observe-and-readback", lambda: self.readback_positive("distinct", [b"A", b"B"]))
        self.case("manifest-eleven-duplicates-readback", lambda: self.readback_positive("duplicates", [b"A"] * 11))
        self.case("native-metadata-readback-not-crypto", lambda: self.readback_positive("native-metadata", [b"A"], native=True))
        self.case("exact-input-clone-path-does-not-change-identity", self.clone_manifest)

        def changed_copy(root,bundle,m):
            name = m["configuration"]["input_identity"]["messages"][0]["copied_path"]
            rehash_ref(bundle,m,name,b"forged-copy")
        self.case("reject-copied-payload-rehashed", lambda: self.mutation("copy-forged", changed_copy))
        self.case("copied-inputs-auto-validated-without-external-flag", lambda: self.mutation("copy-auto-forged", changed_copy, external=False))
        def changed_manifest(root,bundle,m):
            name = m["configuration"]["input_identity"]["manifest_path"]
            value = json.loads((bundle/name).read_bytes()); value["producer_candidate_sha"] = "c"*40
            rehash_ref(bundle,m,name,model.encoded(value))
        self.case("reject-recorded-manifest-rehashed", lambda: self.mutation("manifest-forged", changed_manifest))
        def changed_post(root,bundle,m):
            row = next(r for r in m["records"] if r["method"] == "POST")
            name = row["request"]["path"]; value=json.loads((bundle/name).read_bytes()); value["payload"]=model.b64(b"wrong-post")
            rehash_ref(bundle,m,name,model.encoded(value))
        self.case("reject-post-byte-mismatch", lambda: self.mutation("post-forged", changed_post))
        def swapped_submit_ids(root,bundle,m):
            messages=m["configuration"]["input_identity"]["messages"]
            messages[0]["submit_record_id"],messages[1]["submit_record_id"]=messages[1]["submit_record_id"],messages[0]["submit_record_id"]
        self.case("reject-ordinal-submit-map-swap", lambda: self.mutation("swapped-submit", swapped_submit_ids))
        def duplicate_ordinal(root,bundle,m):
            m["configuration"]["input_identity"]["messages"][1]["ordinal"]=0
        self.case("reject-recorded-duplicate-ordinal", lambda: self.mutation("duplicate-ordinal", duplicate_ordinal))
        def missing_position(root,bundle,m):
            m["records"]=[r for r in m["records"] if not(r["reader_id"]=="first" and r["path"]=="/v1/availability/transaction/1/1/noproof")]
        self.case("missing-flat-position-not-complete", lambda: self.mutation("missing-position", missing_position, incomplete=True))
        def missing_requery(root,bundle,m):
            m["records"]=[r for r in m["records"] if not(r["reader_id"]=="requery" and r["path"]=="/v1/availability/transaction/1/1/noproof")]
        self.case("requery-gap-cannot-use-first-reader", lambda: self.mutation("missing-requery", missing_requery, incomplete=True, reader="requery"))
        def ack_only(root,bundle,m):
            m["records"]=[r for r in m["records"] if not(r["reader_id"]=="first" and r["path"].startswith("/v1/availability/"))]
        self.case("ack-only-not-complete", lambda: self.mutation("ack-only", ack_only, incomplete=True))
        def report_corruption(root,bundle,m):
            name=m["configuration"]["input_identity"]["native_evidence"]["copied_path"]
            rehash_ref(bundle,m,name,b"altered-not-crypto-report")
        self.case("reject-native-report-copy-rehashed", lambda: self.mutation("report-forged", report_corruption, native=True))
        return self.rows


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo",type=Path,required=True)
    parser.add_argument("--candidate",required=True)
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args()
    require(subprocess.check_output(["git","rev-parse","HEAD"],cwd=args.repo,text=True).strip()==args.candidate,
            "candidate SHA mismatch")
    matrix=Matrix(args);rows=matrix.run()
    summary={"schema":RUNNER_SCHEMA,"candidate":args.candidate,"classification":"SYNTHETIC_OFFLINE_NO_NODE",
             "passed":sum(r["status"]=="PASS" for r in rows),"failed":sum(r["status"]=="FAIL" for r in rows),
             "native_finality":"NOT_RUN","genuine_M1_publication":"NOT_RUN","official_node":"NOT_RUN",
             "cases":rows}
    (matrix.out/"summary.json").write_bytes(model.encoded(summary))
    print(json.dumps({k:v for k,v in summary.items() if k!="cases"},sort_keys=True))
    raise SystemExit(1 if summary["failed"] else 0)


if __name__=="__main__":
    main()
