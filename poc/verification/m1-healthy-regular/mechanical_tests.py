#!/usr/bin/env python3
"""Targeted CLI and fail-closed tests in a disposable checkout."""

import hashlib
import json
import os
from copy import deepcopy
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[3]
RUNNER = Path("poc/scenarios/m1-healthy-regular/run.py")
CHECKER = Path("poc/verification/m1-healthy-regular/check.py")
sys.path.insert(0, str(ROOT / RUNNER.parent))

import evidence as ev


def write_json(path, data):
    path.write_bytes(ev.json_bytes(data))


def ref(path, root):
    data = path.read_bytes()
    return {"path": path.relative_to(root).as_posix(), "sha256": hashlib.sha256(data).hexdigest(),
            "bytes": len(data)}


def seal(bundle, manifest):
    path = bundle / "manifest.json"
    write_json(path, manifest)
    (bundle / "manifest.sha256").write_text(
        f"{ref(path, bundle)['sha256']}  manifest.json\n", encoding="ascii")


def replace_indexed_bytes(bundle, manifest, name, data):
    path = bundle / name
    path.write_bytes(data)
    fresh = ref(path, bundle)
    for item in manifest["source_files"] + manifest["artifacts"]:
        if item["path"] == name:
            item.update(fresh)
    for command in manifest["commands"]:
        for stream in ("stdout", "stderr"):
            if command[stream]["path"] == name:
                command[stream] = fresh


class MechanicalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        temp = tempfile.TemporaryDirectory(prefix="m1-mechanical-", dir="/tmp")
        cls.addClassCleanup(temp.cleanup)
        cls.temp = Path(temp.name)
        cls.checkout = cls.temp / "checkout"
        cls.git("clone", "--quiet", "--shared", str(ROOT), str(cls.checkout), cwd=cls.temp)
        for name in (RUNNER, RUNNER.parent / "evidence.py", CHECKER):
            target = cls.checkout / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, target)
        cls.git("add", "--", str(RUNNER.parent), str(CHECKER))
        if cls.git("status", "--porcelain"):
            cls.git("-c", "user.name=M1 mechanical fixture", "-c", "user.email=m1@example.invalid",
                    "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null",
                    "commit", "-qm", "Install test-only harness in disposable checkout")
        cls.candidate = cls.git("rev-parse", "HEAD").strip()

    @classmethod
    def git(cls, *args, cwd=None):
        return subprocess.check_output(["git", *args], cwd=cwd or cls.checkout, text=True)

    def setUp(self):
        scratch = tempfile.TemporaryDirectory(prefix="case-", dir=self.temp)
        self.addCleanup(scratch.cleanup)
        self.scratch = Path(scratch.name)
        self.inputs = self.scratch / "input"
        self.inputs.mkdir()
        self.input = self.inputs / "input.json"
        self.expected = self.inputs / "expected.json"
        contract = self.copy_frozen(ev.CONTRACT)
        source = self.copy_frozen(ev.EXPECTATIONS)
        self.expected_data = {
            "schema": "M1-EXPECTATIONS-1", "contract": contract, "source_expectations": source,
            "oracle_binding": None, "required_gates": list(ev.GATES[:-1]),
        }
        write_json(self.expected, self.expected_data)
        self.data = {
            "schema": "M1-INPUT-1", "contract": contract, "candidate": self.candidate,
            "m0": ev.M0, "dispatch": ev.DISPATCH_SHA,
            "components": dict.fromkeys(ev.COMPONENTS.split()),
            "scenario": "full-window", "seed": 7, "start_order": "forward", "wall_pause_ms": 0,
            "fixtures": dict.fromkeys(ev.FIXTURES.split()), "profile": None,
            "ports": [
                {"port_id": port, "owner": port.split(".")[0], "binding": None} for port in ev.PORTS
            ],
            "expectations": ref(self.expected, self.inputs), "source_run": None,
            "limits": {"max_virtual_tick_us": None, "max_events": 1000000,
                       "wall_timeout_s": 1800, "max_raw_bytes": ev.MAX_BYTES},
        }
        write_json(self.input, self.data)

    def copy_frozen(self, name):
        actual = self.inputs / Path(name).name
        shutil.copyfile(ROOT / name, actual)
        return ref(actual, self.inputs)

    def cli(self, script, args, status):
        command = [sys.executable, "-B", str(script), *map(str, args)]
        result = subprocess.run(command, cwd=self.checkout, capture_output=True, text=True,
                                timeout=40, check=False)
        self.assertEqual(result.returncode, status, f"{command!r}\n{result.stdout}\n{result.stderr}")
        return result

    def launch(self, name, status, *, operation="preflight", candidate=None):
        output = self.scratch / name
        self.cli(RUNNER, (operation, "--candidate", candidate or self.candidate,
                          "--input", self.input, "--output", output), status)
        return output

    def check(self, name, status, *args):
        output = self.scratch / name
        self.cli(CHECKER, (*args, "--expected", self.expected, "--output", output), status)
        return json.loads(output.read_bytes())

    def test_preparation_and_offline_check(self):
        output = self.launch("preflight", 3)
        manifest = json.loads((output / "manifest.json").read_bytes())
        self.assertEqual((manifest["status"], manifest["readiness"]), ("NOT_RUN", "NotReady"))
        self.assertEqual(manifest["revision_before"], manifest["revision_after"])
        self.assertEqual(manifest["revision_before"]["head"], self.candidate)
        self.assertEqual(manifest["revision_before"]["status"], [])
        self.assertEqual(manifest["tool_versions"]["commonware"], ev.COMMONWARE)
        self.assertEqual(manifest["tool_versions"]["bft_gitlink"], ev.BFT)
        self.assertIn("fixtures.schedule", manifest["reason"]["missing_inputs"])
        self.assertNotIn("expectations", manifest["reason"]["missing_inputs"])
        self.assertNotIn("h3.official_artifact", manifest["reason"]["missing_ports"])
        self.assertIsNone(manifest["cutoff"]["virtual_tick_us"])
        report = self.check("report.json", 3, "--bundle", output)
        self.assertEqual(report["commands"], [])
        self.assertEqual([gate["status"] for gate in report["gates"]],
                         ["PASS", "NOT_RUN", "NOT_RUN", "NOT_RUN"])

    def test_h09_adds_official_gate_and_requires_source(self):
        self.data["scenario"] = ev.SCENARIOS[3]
        self.expected_data["required_gates"] = list(ev.GATES)
        write_json(self.expected, self.expected_data)
        self.data["expectations"] = ref(self.expected, self.inputs)
        write_json(self.input, self.data)
        output = self.launch("h09", 3, operation="run")
        report = self.check("h09-report.json", 3, "--bundle", output)
        self.assertIn("h3.official_artifact", report["reason"]["missing_ports"])
        self.assertIn("source_run", report["reason"]["missing_inputs"])
        self.assertEqual((report["gates"][-1]["id"], report["gates"][-1]["status"]),
                         ("OfficialSameBytes", "NOT_RUN"))

    def test_h09_validates_copied_source_report_and_manifest_structure(self):
        source = self.launch("h01-source", 3, operation="run")
        prior = self.inputs / "prior-run"
        shutil.copytree(source, prior)
        manifest_path = prior / "manifest.json"
        manifest = json.loads(manifest_path.read_bytes())
        source_run = {}
        for role in ("public_messages", "regional_results"):
            path = prior / f"{role}.fixture"
            path.write_bytes(b"Mechanical fixture bytes")
            manifest["artifacts"].append({"role": role, "producer": "self-test", "schema": "raw",
                                          **ref(path, prior)})
            source_run[role] = [ref(path, self.inputs)]
        seal(prior, manifest)
        source_report = self.check("source-checked.json", 3, "--bundle", prior)
        report_path = self.inputs / "source-report.json"

        self.data["scenario"] = ev.SCENARIOS[3]
        self.expected_data["required_gates"] = list(ev.GATES)
        write_json(self.expected, self.expected_data)
        self.data["expectations"] = ref(self.expected, self.inputs)
        for name, change, exit_code, error in (
            ("control", None, 3, None),
            ("report-bool-status", lambda run, report: report.update(status=True), 2,
             "invalid checker status"),
            ("report-expected-escape", lambda run, report: report["expected"].update(path="../outside.json"),
             2, "relative evidence path"),
            ("report-expected-bool-size", lambda run, report: report["expected"].update(bytes=False),
             2, "FileRef.bytes"),
            ("manifest-bool-status", lambda run, report: run.update(status=True), 2, "invalid run status"),
            ("manifest-command-escape", lambda run, report: run["commands"][0]["stdout"].update(path="../out"),
             2, "relative evidence path"),
        ):
            with self.subTest(name=name):
                copied_manifest, copied_report = deepcopy(manifest), deepcopy(source_report)
                if change is not None:
                    change(copied_manifest, copied_report)
                seal(prior, copied_manifest)
                copied_report["bundles"] = [ref(manifest_path, prior)]
                write_json(report_path, copied_report)
                self.data["source_run"] = {**source_run, "manifest": ref(manifest_path, self.inputs),
                                           "checker_report": ref(report_path, self.inputs)}
                write_json(self.input, self.data)
                output = self.launch(name, exit_code)
                if error is not None:
                    rejection = json.loads((output / "rejection.json").read_bytes())
                    self.assertIn(error, rejection["reason"]["detail"])

    def test_resealed_listing_and_source_cannot_change_candidate_members(self):
        original = self.launch("original-candidate", 3)
        mutated = self.scratch / "forged-source"
        shutil.copytree(original, mutated)
        manifest = json.loads((mutated / "manifest.json").read_bytes())
        source_name = f"source/{RUNNER.as_posix()}"
        modified_source = (mutated / source_name).read_bytes() + b"\n# Not in candidate\n"
        replace_indexed_bytes(mutated, manifest, source_name, modified_source)

        listing = next(item["stdout"]["path"] for item in manifest["commands"]
                       if "ls-tree" in item["argv"] and "-r" in item["argv"])
        original_listing = (mutated / listing).read_bytes()
        original_blob = self.git("rev-parse", f"{self.candidate}:{RUNNER.as_posix()}").strip().encode()
        new_blob = ev.git_object_id("blob", modified_source).encode()
        old_entry = original_blob + b"\t" + RUNNER.as_posix().encode() + b"\0"
        self.assertIn(old_entry, original_listing)
        modified_listing = original_listing.replace(old_entry, new_blob + old_entry[40:])
        replace_indexed_bytes(mutated, manifest, listing, modified_listing)
        seal(mutated, manifest)
        report = self.check("forged-source.json", 2, "--bundle", mutated)
        self.assertIn("Git tree object ID mismatch", report["reason"]["detail"])

        modified_commit = self.scratch / "forged-commit"
        shutil.copytree(original, modified_commit)
        manifest = json.loads((modified_commit / "manifest.json").read_bytes())
        command = next(item for item in manifest["commands"] if "cat-file" in item["argv"]
                       and item["argv"][-2:] == ["commit", self.candidate])
        commit_name = command["stdout"]["path"]
        commit = (modified_commit / commit_name).read_bytes() + b"\nmodified commit message\n"
        replace_indexed_bytes(modified_commit, manifest, commit_name, commit)
        seal(modified_commit, manifest)
        report = self.check("forged-commit.json", 2, "--bundle", modified_commit)
        self.assertIn("Git commit object ID mismatch", report["reason"]["detail"])

    def test_stale_and_dirty_checkout(self):
        stale = self.git("rev-parse", "HEAD^").strip()
        self.data["candidate"] = stale
        write_json(self.input, self.data)
        rejected = self.launch("stale", 2, candidate=stale)
        manifest = json.loads((rejected / "manifest.json").read_bytes())
        self.assertEqual(manifest["readiness"], "Reject")
        self.assertIn("stale candidate", manifest["reason"]["detail"])
        self.assertEqual(manifest["revision_before"]["head"], self.candidate)

        self.data["candidate"] = self.candidate
        write_json(self.input, self.data)
        untracked = self.checkout / "m1-mechanical-dirty"
        untracked.write_text("preserve this input", encoding="utf-8")
        self.addCleanup(untracked.unlink)
        rejected = self.launch("dirty", 2)
        manifest = json.loads((rejected / "manifest.json").read_bytes())
        self.assertEqual(manifest["readiness"], "Reject")
        self.assertIn("dirty", manifest["reason"]["detail"])
        self.assertEqual(manifest["revision_before"]["status"], ["?? m1-mechanical-dirty"])
        self.assertEqual(untracked.read_text(encoding="utf-8"), "preserve this input")

    def test_missing_tools_and_wrong_rust_version(self):
        with patch.dict(os.environ, {"PATH": str(self.scratch)}):
            no_git = self.launch("no-git", 3)
        report = self.check("no-git.json", 3, "--bundle", no_git)
        self.assertIn("tool_versions.git", report["reason"]["missing_inputs"])
        self.assertEqual(report["gates"][0]["status"], "NOT_RUN")

        rust_bin = self.scratch / "rust-bin"
        rust_bin.mkdir()
        with patch.dict(os.environ, {"REGIONAL_BFT_RUST_BIN": str(rust_bin)}):
            no_rust = self.launch("no-rust", 3)
            report = self.check("no-rust.json", 3, "--bundle", no_rust)
            self.assertIn("tool_versions.rustc", report["reason"]["missing_inputs"])
            self.assertIn("tool_versions.cargo", report["reason"]["missing_inputs"])
            rustc = rust_bin / "rustc"
            rustc.write_text("#!/bin/sh\nprintf 'rustc 1.94.0\\n'\n", encoding="ascii")
            rustc.chmod(0o700)
            wrong_rust = self.launch("wrong-rust", 2)
        manifest = json.loads((wrong_rust / "manifest.json").read_bytes())
        self.assertIn("wrong rustc version", manifest["reason"]["detail"])

    def test_closed_json_duplicate_fields_and_integer_types(self):
        for index, (field, value) in enumerate((
            ("seed", True), ("seed", 7.0), ("wall_pause_ms", False), ("limits", None),
            ("unknown", 0), ("schema", "M1-INPUT-2"),
        )):
            with self.subTest(field=field, value=value):
                write_json(self.input, {**self.data, field: value})
                self.launch(f"invalid-{index}", 2)
        text = self.input.read_text(encoding="utf-8")
        self.input.write_text('{"scenario":"full-window",\n' + text[1:], encoding="utf-8")
        output = self.launch("duplicate", 2)
        rejection = json.loads((output / "rejection.json").read_bytes())
        self.assertIn("duplicate key", rejection["reason"]["detail"])

    def test_file_ref_path_symlink_hash_size_and_budget(self):
        raw = self.inputs / "data.bin"
        raw.write_bytes(b"abcd")
        raw_ref = ref(raw, self.inputs)
        (self.inputs / "link.bin").symlink_to(raw)
        (self.inputs / "link-dir").symlink_to(self.inputs, target_is_directory=True)
        for index, fields in enumerate((
            {"path": "../data.bin"}, {"path": str(raw)}, {"path": "./data.bin"},
            {"path": "link.bin"}, {"path": "link-dir/data.bin"},
            {"sha256": "f" * 64}, {"bytes": 3}, {"bytes": True}, {"bytes": ev.MAX_BYTES + 1},
        )):
            with self.subTest(fields=fields):
                self.data["fixtures"]["config"] = {**raw_ref, **fields}
                write_json(self.input, self.data)
                self.launch(f"ref-{index}", 2)

    def test_external_expected_must_be_frozen_and_include_all_gates(self):
        output = self.launch("original", 3)
        for index, replacement in enumerate((
            {"required_gates": list(ev.GATES[1:-1])},
            {"required_gates": [ev.GATES[0], *ev.GATES[:3]]},
            {"required_gates": [*ev.GATES[:3], "invented"]},
            {"schema": "M1-EXPECTATIONS-2"},
        )):
            with self.subTest(replacement=replacement):
                write_json(self.expected, {**self.expected_data, **replacement})
                self.check(f"rejected-{index}.json", 2, "--bundle", output)
        source = self.inputs / self.expected_data["source_expectations"]["path"]
        source.write_bytes(b"claims reconstructed from observed output")
        self.expected_data["source_expectations"] = ref(source, self.inputs)
        write_json(self.expected, self.expected_data)
        self.check("unfrozen.json", 2, "--bundle", output)

    def test_existing_outputs_are_unchanged(self):
        original = self.launch("original", 3)
        before = (original / "manifest.json").read_bytes()
        self.launch("original", 2)
        self.assertEqual((original / "manifest.json").read_bytes(), before)
        existing = self.scratch / "existing.json"
        existing.write_bytes(b"keep")
        self.cli(CHECKER, ("--bundle", original, "--expected", self.expected, "--output", existing), 2)
        self.assertEqual(existing.read_bytes(), b"keep")

    def test_damaged_and_unindexed_bundle_files(self):
        original = self.launch("original", 3)
        for index, mutation in enumerate(("hash", "missing", "size", "symlink", "unindexed",
                                          "duplicate", "seal")):
            with self.subTest(mutation=mutation):
                bundle = self.scratch / f"mutant-{index}"
                shutil.copytree(original, bundle)
                manifest_path = bundle / "manifest.json"
                manifest = json.loads(manifest_path.read_bytes())
                raw = bundle / manifest["commands"][0]["stdout"]["path"]
                if mutation == "hash":
                    raw.write_bytes(b"Z" * raw.stat().st_size)
                if mutation == "missing":
                    raw.unlink()
                if mutation == "size":
                    raw.write_bytes(b"")
                if mutation == "symlink":
                    raw.unlink()
                    raw.symlink_to(original / raw.relative_to(bundle))
                if mutation == "unindexed":
                    (bundle / "unindexed").write_bytes(b"hidden raw")
                if mutation == "duplicate":
                    manifest["artifacts"].append(manifest["artifacts"][0])
                    seal(bundle, manifest)
                if mutation == "seal":
                    manifest_path.write_bytes(manifest_path.read_bytes() + b" ")
                report = self.check(f"mutant-{index}.json", 2, "--bundle", bundle)
                self.assertEqual(report["gates"][0]["status"], "FAIL")

    def test_binding_never_executes_and_claimed_pass_stays_not_run(self):
        marker = self.scratch / "should-not-exist"
        binding = self.inputs / "binding.json"
        write_json(binding, {"argv": ["/usr/bin/touch", str(marker)], "status": "Ready"})
        binding_ref = ref(binding, self.inputs)
        self.expected_data["oracle_binding"] = binding_ref
        write_json(self.expected, self.expected_data)
        self.data["expectations"] = ref(self.expected, self.inputs)
        for port in self.data["ports"]:
            port["binding"] = binding_ref
        for key in self.data["components"]:
            self.data["components"][key] = {
                "owner": key, "delivery_sha": self.candidate,
                "review": binding_ref, "contract": binding_ref,
            }
        self.data["fixtures"] = dict.fromkeys(ev.FIXTURES.split(), binding_ref)
        self.data["profile"] = binding_ref
        self.data["limits"]["max_virtual_tick_us"] = 61_000_000
        write_json(self.input, self.data)
        source = self.launch("non-null-run", 3, operation="run")
        self.assertFalse(marker.exists())
        started = json.loads((source / "manifest.json").read_bytes())
        self.assertIsNone(started["readiness"])
        self.assertEqual(started["reason"]["missing_inputs"], [])
        self.assertTrue(all(port["status"] == "NotReady" for port in started["ports"]))

        fake = self.scratch / "fake-pass"
        shutil.copytree(source, fake)
        started["status"] = "PASS"
        started["reason"] = {"code": "ClaimedPass"}
        for port in started["ports"]:
            port["status"] = "Ready"
        for gate in started["gates"]:
            gate.update(status="PASS", reason={"code": "ClaimedPass"})
        raw = fake / "proof.json"
        write_json(raw, {"verified": True, "signers": 4})
        started["artifacts"].append({
            "role": "votes_and_proofs", "producer": "self-report", "schema": "claimed",
            **ref(raw, fake),
        })
        seal(fake, started)
        report = self.check("fake-pass.json", 3, "--bundle", fake)
        self.assertEqual(report["status"], "NOT_RUN")
        self.assertEqual([gate["status"] for gate in report["gates"]],
                         ["PASS", "NOT_RUN", "NOT_RUN", "NOT_RUN"])
        report = self.check("compare.json", 3, "--compare", fake, fake)
        self.assertEqual(report["operation"], "compare")
        self.assertEqual(report["status"], "NOT_RUN")
        raw.unlink()
        self.check("corrupt-compare.json", 2, "--compare", source, fake)
        self.assertFalse(marker.exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
