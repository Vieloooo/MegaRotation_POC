import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from model import historical_choice, signature_set, signer_view
from sample import PROFILE, poll, sample_projection, tip
from projection import AutoBundle


def sha(data):
    return hashlib.sha256(data).hexdigest()


class ProjectionTests(unittest.TestCase):
    def test_contract_sample(self):
        page = sample_projection()
        self.assertEqual(page["mode"], "sample")
        stages = {row["stage"]: row for row in page["events"]}
        self.assertLess(stages["outage"]["seq"], stages["heartbeat"]["seq"])
        self.assertLess(stages["submit"]["seq"], stages["include"]["seq"])
        self.assertLess(stages["force"]["seq"], stages["accepted"]["seq"])
        self.assertEqual(stages["accepted"]["head"], 18)
        self.assertEqual([row["height"] for row in stages["accepted"]["appends"] if row["type"] == "padding"],
                         list(range(2, 18)))
        self.assertEqual([row["signedBy"] for row in page["events"] if "signedBy" in row], [0, 1])

    def test_lag_and_frozen_watermark(self):
        prior = tip(3, 11_250_000)
        verified = tip(4, 11_250_000)
        stalled = poll(0, prior, verified, verified, False, "ORDERING_NOT_ADVANCING")
        view = signer_view(stalled, PROFILE["stall_us"])
        self.assertTrue(view["q_advances"])
        self.assertFalse(view["theta_advances"])
        with self.assertRaisesRegex(ValueError, "eligibility contradicts"):
            signer_view({**stalled, "eligible": True, "reason": "ELIGIBLE"}, PROFILE["stall_us"])
        lag = poll(2, prior, verified, tip(5, 13_250_000), False, "INPUTS_NOT_READY")
        self.assertFalse(signer_view(lag, PROFILE["stall_us"])["ready"])

    def test_signature_duplicates(self):
        request = {"control_seq": 1}
        first = {"signer": 0, "address": "A", "digest": "digest", "request": request}
        second = {**first, "signer": 1, "address": "B"}
        self.assertEqual(signature_set([first, second], 2), ("digest", request))
        with self.assertRaisesRegex(ValueError, "duplicate"):
            signature_set([first, {**second, "address": "a"}], 2)
        with self.assertRaisesRegex(ValueError, "disagree"):
            signature_set([first, {**second, "digest": "changed"}], 2)

    def test_plan_uses_actual_floor_and_cycle(self):
        policy = {"order": [0, 2, 1], "enabled_mask": 6}
        self.assertEqual(historical_choice(policy, 0, 3, 7, 3), (2, [2, 1, 0], 7, 7))
        self.assertEqual(historical_choice({**policy, "enabled_mask": 0}, 0, 9, 7, 3),
                         (None, [2, 1, 0], 7, 9))


class SealTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        (self.root / "raw").mkdir()
        self.file = self.root / "raw" / "profile.json"
        self.file.write_bytes(b"{}")
        candidate = "a" * 40
        revision = {"head": candidate, "status": ""}
        manifest = {"candidate": candidate, "status": "COMPLETED", "scenario": "outage", "seed": 42,
                    "schema": "M3-AUTO-FORCE-RUN-1", "before": revision, "after": revision,
                    "files": {"raw/profile.json": {"bytes": 2, "sha256": sha(b"{}")}}}
        raw = json.dumps(manifest).encode()
        (self.root / "manifest.json").write_bytes(raw)
        (self.root / "manifest.sha256").write_text(sha(raw) + "  manifest.json\n")

    def test_missing_and_corrupted_raw_input(self):
        bundle = AutoBundle(self.root)
        self.assertEqual(bundle.read("raw/profile.json"), b"{}")
        self.file.write_bytes(b"[]")
        with self.assertRaisesRegex(ValueError, "raw/profile.json"):
            bundle.read("raw/profile.json")
        self.file.unlink()
        with self.assertRaisesRegex(FileNotFoundError, "profile.json"):
            bundle.read("raw/profile.json")

    def test_missing_checker_keeps_existing_output(self):
        out = self.root / "auto-force.html"
        out.write_bytes(b"prior page")
        result = subprocess.run(
            [sys.executable, "-B", str(Path(__file__).with_name("generate_auto_force.py")),
             "--bundle", str(self.root), "--independent", str(self.root / "not-present"), "--output", str(out)],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"report.json", result.stderr)
        self.assertEqual(out.read_bytes(), b"prior page")


if __name__ == "__main__":
    unittest.main()
