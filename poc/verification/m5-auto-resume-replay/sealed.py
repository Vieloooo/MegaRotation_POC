"""Manifest-bound intake for independently checked normalized M5 observations."""

import hashlib
import json
from pathlib import PurePosixPath

from model import m4, project, require, uint

TAPE = "auto-resume-replay-v1.json"


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def read_file(root, relative):
    part = PurePosixPath(relative)
    require(part.as_posix() == relative and relative not in {"", "."} and
            ".." not in part.parts and not part.is_absolute(), "invalid evidence path")
    file = root.joinpath(*part.parts).resolve()
    require(file.is_relative_to(root), "evidence path escapes its bundle")
    return file.read_bytes()


class Sealed:
    def __init__(self, bundle):
        self.root = bundle.resolve()
        raw = read_file(self.root, "manifest.json")
        self.sha = sha(raw)
        self.manifest = json.loads(raw)
        require(read_file(self.root, "manifest.sha256").decode().split() == [self.sha, "manifest.json"],
                "manifest does not match sidecar")
        self.files = self.manifest["files"]
        self.entries = self.files
        self.cache = {}
        self.lines = {}

    def read(self, path):
        if path not in self.cache:
            info = self.files[path]
            raw = read_file(self.root, path)
            require(sha(raw) == info["sha256"] and len(raw) == info["bytes"], f"{path}: digest or length mismatch")
            self.cache[path] = raw
        return self.cache[path]

    def ref(self, reference, event):
        path = reference["path"]
        raw = self.read(path)
        require(all(isinstance(reference[field], str) and reference[field] for field in ("label", "locator")),
                f"{path}: missing evidence locator")
        if "bytes" in reference:
            offset, length = reference["bytes"]
            require(uint(offset) and uint(length) and 0 < length <= len(raw) - offset,
                    f"{path}: invalid byte range")
        if "line" in reference:
            if path not in self.lines:
                self.lines[path] = raw.splitlines()
            line = reference["line"]
            require(uint(line) and 0 < line <= len(self.lines[path]), f"{path}: invalid line")
        if reference.get("trace_event"):
            raw_row = json.loads(self.lines[path][reference["line"] - 1])
            require(raw_row["seq"] == event["seq"] and raw_row["tick_us"] == event["tick_us"],
                    f"{path}: normalized time/seq differs from raw observation")
            return True
        return False

    def check_cut(self, detail, included):
        cut = self.read(detail["cut_bytes_path"])
        domain = b"PoC/Derivation/ResumeCutCheckpoint/v1"
        require(sha(len(domain).to_bytes(8, "big") + domain + len(cut).to_bytes(8, "big") + cut)
                == detail["cut_id"], "canonical cut bytes do not match digest")
        raw_cut = json.loads(cut)
        anchor = raw_cut["target_anchor"]
        def pos(value):
            return [value["block"], value["tx"], value["log"]]
        height = raw_cut["target_height"]
        prefix = raw_cut["derived_prefix"]
        require(pos(raw_cut["resume_position"]) == detail["position"]
                and pos(raw_cut["cursor_set_position"]) == detail["cursor_position"]
                and uint(height) and 0 < height <= len(prefix) and height == detail["target"]
                and anchor["anchor"]["block"]["height"] == height
                and bytes(anchor["anchor"]["block"]["block_id"]).hex() == detail["anchor"]["id"]
                and bytes(anchor["native_hash"]).hex() == detail["anchor"]["native"]
                and anchor["anchor"]["block"]["origin"] == detail["anchor"]["origin"]
                and bytes(prefix[height - 1]["native_block"]["post_state_root"]).hex() == detail["anchor"]["post"]
                and [bytes(raw_cut["genesis"]["native_hash"]).hex(),
                     *(bytes(item["native_hash"]).hex() for item in prefix)] == detail["prefix"]
                and bytes(prefix[height - 1]["native_hash"]).hex() == detail["anchor"]["native"]
                and anchor["state"] == prefix[height - 1]["state"],
                "canonical cut identity differs from projection")
        witness = raw_cut["evidence"]
        require(bytes(witness["including_block"]["hash"]).hex() == included["header"]["hash"]
                and witness["including_block"]["height"] == included["header"]["height"]
                and bytes(witness["transaction_hash"]).hex() == included["tx"].removeprefix("0x")
                and witness["cursor_set_rpc_log_index"] == included["cursor_rpc_log_index"]
                and witness["resume_rpc_log_index"] == included["receipt_log_index"]
                and witness["sequence"] == included["control_seq"],
                "canonical cut does not bind the exact successful receipt")


def load(bundle, independent):
    sealed = Sealed(bundle)
    manifest = sealed.manifest
    candidate, seed = manifest["candidate"], manifest["seed"]
    require(len(candidate) == 40 and set(candidate) <= set("0123456789abcdef")
            and uint(seed) and manifest["status"] == "COMPLETED" and manifest["schema"] == "M5-AUTO-RESUME-RUN-1",
            "invalid completed M5 manifest")
    require(manifest["before"] == manifest["after"] and manifest["before"]["head"] == candidate
            and not manifest["before"]["status"], "run does not bind a clean exact candidate")
    independent = independent.resolve()
    report_bytes = read_file(independent, "report.json")
    report = json.loads(report_bytes)
    require(report["status"] == "PASS" and report["candidate"] == candidate
            and report["manifest_sha256"] == sealed.sha, "checker PASS does not bind this candidate and run")
    provenance = {"candidate": candidate, "manifest": sealed.sha, "report": sha(report_bytes)}
    if "auto_resume_replay_sha256" in report:
        tape_bytes = read_file(independent, TAPE)
        require(sha(tape_bytes) == report["auto_resume_replay_sha256"], "checker digest does not bind normalized tape")
        tape = json.loads(tape_bytes)
        provenance["checker_tape"] = sha(tape_bytes)
    else:
        from raw_m5 import Mapper
        require(report["mode"] in ("stopped", "frozen"), "expected independent positive AR01 or AR03")
        outputs = {"public":"public-replay.json", "d":"independent-D.json",
                   "headers":"verified-l1-headers.json", "cuts":"verified-cuts.json"}
        checker = {}
        for key, name in outputs.items():
            data = read_file(independent, name)
            provenance[name] = sha(data)
            checker[key] = json.loads(data)
        tape = Mapper(sealed, manifest, report, checker).tape()
        provenance["normalized_view"] = sha(json.dumps(tape, sort_keys=True, separators=(",", ":")).encode())
    require(tape["mode"] == "observed" and tape["profile"]["seed"] == seed,
            "sealed tape must be observed and use the run seed")
    inclusions = {}
    for event in tape["events"]:
        refs = [sealed.ref(reference, event) for reference in event["refs"]]
        require(any(refs), f"seq{event['seq']}: missing raw timing reference")
        detail = event["detail"]
        if event["kind"] == "resume_included":
            inclusions[m4.l1pos(detail["position"])] = detail
        if event["kind"] == "resume_cut":
            sealed.check_cut(detail, inclusions[m4.l1pos(detail["position"])])
    return project(tape, "sealed", provenance)
