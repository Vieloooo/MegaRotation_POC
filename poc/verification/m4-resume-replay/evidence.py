"""Sealed input adapter for the normalized replay v1 interchange.

A backend-specific mapper exports the normalized tape and binds it to its
independent report. This module does not guess raw logger event names.
"""

import hashlib
import json
from pathlib import Path, PurePosixPath

from model import l1pos, project, require, uint


TAPE = "resume-replay-v1.json"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def relpath(label):
    part = PurePosixPath(label)
    require(label == part.as_posix() and ".." not in part.parts and not part.is_absolute()
            and label not in ("", "."), f"invalid evidence path {label}")
    return part


def read_bound(root, path):
    label = relpath(path)
    file = root.joinpath(*label.parts)
    require(file.resolve().is_relative_to(root), f"{path}: evidence leaves bundle directory")
    return file.read_bytes()


class Sealed:
    def __init__(self, directory, manifest):
        self.root = directory.resolve()
        self.entries = manifest["files"]
        require(isinstance(self.entries, dict), "manifest files must be keyed by artifact path")
        self.data = {}
        self.lines = {}

    def read(self, relative):
        if relative not in self.data:
            require(relative in self.entries, f"unsealed reference {relative}")
            info = self.entries[relative]
            content = read_bound(self.root, relative)
            require(sha(content) == info["sha256"] and len(content) == info["bytes"],
                    f"{relative}: artifact digest or length mismatch")
            self.data[relative] = content
        return self.data[relative]

    def trace_row(self, relative, line):
        if relative not in self.lines:
            self.lines[relative] = self.read(relative).splitlines()
        require(uint(line) and 0 < line <= len(self.lines[relative]), f"{relative}: no line {line}")
        return json.loads(self.lines[relative][line - 1])

    def check_ref(self, ref, event):
        path = ref["path"]
        content = self.read(path)
        require(isinstance(ref["label"], str) and ref["label"]
                and isinstance(ref["locator"], str) and ref["locator"],
                f"{path}: missing evidence locator")
        if "line" in ref:
            line = ref["line"]
            require(uint(line) and 0 < line <= len(content.splitlines()), f"{path}: no line {line}")
        if "bytes" in ref:
            offset, length = ref["bytes"]
            require(uint(offset) and uint(length) and 0 < length <= len(content) - offset,
                    f"{path}: invalid byte range")
        if ref.get("trace_event"):
            row = self.trace_row(path, ref["line"])
            require(row["seq"] == event["seq"] and row["tick_us"] == event["tick_us"],
                    f"{path}: timeline tick/seq differs from source trace")
            return True
        return False

    def check_cut(self, detail, included):
        path = detail["cut_bytes_path"]
        cut = self.read(path)
        prefix = b"PoC/Derivation/ResumeCutCheckpoint/v1"
        computed = sha(len(prefix).to_bytes(8, "big") + prefix + len(cut).to_bytes(8, "big") + cut)
        require(computed == detail["cut_id"], f"{path}: ResumeCut exact byte digest mismatch")
        record = json.loads(cut)

        def position(pos):
            return [pos["block"], pos["tx"], pos["log"]]

        def hash32(value):
            require(isinstance(value, list) and len(value) == 32
                    and all(uint(octet) and octet < 256 for octet in value), f"{path}: invalid hash")
            return bytes(value).hex()

        target = record["target_height"]
        prefix = record["derived_prefix"]
        require(position(record["resume_position"]) == detail["position"]
                and position(record["cursor_set_position"]) == detail["cursor_position"]
                and target == detail["target"] and uint(target) and 0 < target <= len(prefix)
                and [hash32(record["genesis"]["native_hash"]), *(hash32(item["native_hash"]) for item in prefix)]
                    == detail["prefix"], f"{path}: raw cut position/target/prefix mismatch")
        raw_anchor = record["target_anchor"]
        block = raw_anchor["anchor"]["block"]
        item = prefix[target - 1]
        anchor = detail["anchor"]
        require(anchor["height"] == block["height"] == target and anchor["origin"] == block["origin"]
                and anchor["id"] == hash32(block["block_id"]) and anchor["native"] == hash32(raw_anchor["native_hash"])
                and anchor["native"] == hash32(item["native_hash"]) and raw_anchor["state"] == item["state"]
                and anchor["post"] == hash32(item["native_block"]["post_state_root"]),
                f"{path}: raw cut target anchor/state mismatch")
        witness = record["evidence"]
        require(hash32(witness["including_block"]["hash"]) == included["header"]["hash"]
                and witness["including_block"]["height"] == included["header"]["height"]
                and hash32(witness["transaction_hash"]) == included["tx"].removeprefix("0x")
                and witness["cursor_set_rpc_log_index"] == included["cursor_rpc_log_index"]
                and witness["resume_rpc_log_index"] == included["receipt_log_index"]
                and witness["sequence"] == included["control_seq"], f"{path}: raw cut receipt binding mismatch")


def load_formal(bundle, independent):
    bundle, independent = bundle.resolve(), independent.resolve()
    manifest_bytes = read_bound(bundle, "manifest.json")
    manifest_hash = sha(manifest_bytes)
    require(read_bound(bundle, "manifest.sha256").decode().split() == [manifest_hash, "manifest.json"],
            "manifest.sha256 does not bind manifest.json")
    manifest = json.loads(manifest_bytes)
    candidate = manifest["candidate"]
    require(isinstance(candidate, str) and len(candidate) == 40 and all(c in "0123456789abcdef" for c in candidate)
            and manifest["status"] == "COMPLETED" and uint(manifest["seed"]),
            "invalid completed M4 manifest")
    report_bytes = read_bound(independent, "report.json")
    report = json.loads(report_bytes)
    require(report["status"] == "PASS" and report["candidate"] == candidate
            and report["manifest_sha256"] == manifest_hash and report["scenario"] == "DR01",
            "checker PASS does not bind this DR01 candidate and manifest")
    tape_bytes = read_bound(independent, TAPE)
    require(sha(tape_bytes) == report["resume_replay_sha256"],
            "checker digest does not bind normalized M4 tape")
    tape = json.loads(tape_bytes)
    require(tape["mode"] == "observed" and tape["profile"]["seed"] == manifest["seed"],
            "checker tape must contain observed data for sealed seed")
    sealed = Sealed(bundle, manifest)
    require(tape["events"], "empty evidence tape")
    recorded = {}
    for event in tape["events"]:
        evidence = [sealed.check_ref(ref, event) for ref in event["refs"]]
        require(any(evidence), f"trace seq {event['seq']}: no bound raw timing evidence")
        detail = event["detail"]
        if event["kind"] == "resume_included":
            recorded[l1pos(detail["position"])] = detail
        if event["kind"] == "resume_cut":
            sealed.check_cut(detail, recorded[l1pos(detail["position"])])
    return project(tape, "sealed", {"candidate": candidate, "manifest": manifest_hash,
                                   "report": sha(report_bytes), "tape": sha(tape_bytes)})
