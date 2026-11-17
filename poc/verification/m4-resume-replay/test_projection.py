import copy
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from evidence import Sealed, sha
from model import Replay, comparison, cut_anchor, derived_id, project
from sample import sample_tape, symbol


def stage(tape, name):
    return next(row for row in tape["events"] if row["stage"] == name)


def state_before(tape, name):
    replay = Replay(tape["profile"])
    for row in tape["events"]:
        if row["stage"] == name:
            return replay
        replay.step(row)
    raise ValueError(f"missing stage {name}")


class ProjectionTest(unittest.TestCase):
    def test_replay_landmarks_and_atomic_delta(self):
        tape = sample_tape()
        frames = project(tape, "sample", {})["frames"]
        staged = {frame["stage"]: frame for frame in frames if frame["stage"]}
        diff = staged["divergence"]
        self.assertEqual(diff["comparison"], {"common_height": 2, "common_prefix": 1, "first_diff": 2})
        cursor = staged["gprime-cut"]["chains"]
        self.assertEqual([len(cursor[name]) - 1 for name in ("g", "gprime", "d")], [9, 20, 20])
        before, after = staged["resume-wait"], staged["resume"]
        self.assertEqual(len(before["chains"]["g"]), 18)
        self.assertEqual(before["duty"]["region"], 1)
        self.assertEqual(len(after["chains"]["g"]), 21)
        self.assertEqual(after["duty"]["region"], 2)
        self.assertEqual(before["chains"]["d"], after["chains"]["d"])
        self.assertEqual(after["chains"]["g"][1], before["chains"]["g"][1])
        self.assertEqual(after["chains"]["g"][2]["provenance"], "DER")
        accepted = staged["accepted"]["chains"]["g"][-1]
        anchor = staged["cut"]["detail"]["anchor"]
        self.assertEqual(accepted["physical_parent"], derived_id(20, anchor["native"]))
        self.assertEqual(accepted["pre"], anchor["post"])
        self.assertEqual(accepted["height"], 21)

    def test_equal_and_height_lag(self):
        native = lambda h: {"native": str(h)}
        self.assertEqual(comparison([native(0), native(1)], [native(0)]),
                         {"common_height": 0, "common_prefix": 0, "first_diff": None})
        a = [native(0), native(1)]
        self.assertEqual(comparison(a, a)["first_diff"], None)

    def test_da_wait_exact_position_and_no_early_fallback(self):
        tape = sample_tape()
        replay = state_before(tape, "da-wait")
        wait = stage(tape, "da-wait")
        before = replay.view()
        replay.step(wait)
        self.assertEqual(replay.view(), before)
        with self.assertRaisesRegex(ValueError, "accepted DA"):
            replay.step(stage(tape, "derived"))
        for row in tape["events"]:
            if row["kind"] in ("l1_finalized", "l1_available") and row["detail"]["height"] == 7:
                replay.step(row)
        with self.assertRaisesRegex(ValueError, "accepted DA"):
            replay.step(stage(tape, "fallback"))
        wrong = copy.deepcopy(next(row for row in tape["events"] if row["kind"] == "da_verified"))
        wrong["detail"]["verified_commitment"] = "f" * 64
        with self.assertRaisesRegex(ValueError, "commitment mismatch"):
            replay.step(wrong)
        replay.step(next(row for row in tape["events"] if row["kind"] == "da_verified"))
        replay.step(stage(tape, "derived"))
        self.assertEqual(replay.d_position, (6, 0, 0))

    def test_same_block_cuts_and_old_halt(self):
        tape = sample_tape()
        replay = state_before(tape, "resume-included")
        original = stage(tape, "resume-included")["detail"]
        cut = stage(tape, "cut")["detail"]
        replay.step(next(row for row in tape["events"] if row["kind"] == "l1_finalized" and row["detail"]["height"] == 9))
        replay.step(next(row for row in tape["events"] if row["kind"] == "l1_available" and row["detail"]["height"] == 9))
        # A starting D-ahead reader already completed this distinct native prefix.
        ahead = replay.append(replay.chains["d"], cut["fill"])
        replay.chains["d"] = ahead
        for tx, target in enumerate((3, 8)):
            recorded = {**original, "position": [9, tx, 1], "cursor_position": [9, tx, 0],
                        "target": target, "previous": 1 if tx == 0 else 3, "tx": str(tx)}
            replay.resume_included(recorded)
            replay.capture_cut({**cut, "position": recorded["position"], "cursor_position": recorded["cursor_position"],
                                "target": target, "anchor": cut_anchor(ahead[target]), "cut_id": symbol(str(tx)), "fill": []})
        first, second = replay.cuts[(9, 0, 1)], replay.cuts[(9, 1, 1)]
        self.assertEqual([first["target"], second["target"]], [3, 8])
        self.assertEqual([len(first["chain"]), len(second["chain"])], [21, 21])
        self.assertIs(replay.chains["d"], second["chain"])
        for event in tape["events"]:
            if event["kind"] == "espresso" and event["detail"]["q"] not in replay.espresso and event["detail"]["q"] < 19:
                replay.step(event)
        act = {"kind": "Resume", "key_us": 28_000_000, "index": [9, 0, 1]}
        old, adopted = replay.chains["g"], ahead[:4]
        replay.duty = {**replay.duty, "tail": "HALT", "cid": None, "region": None}
        replay.closed.add(0)
        result = replay.apply_resume({"resume_position": [9, 0, 1], "cut_id": first["cut_id"],
                    "position": act, "coverage": {"q": 18, "l1_height": 9}, "retain_through": 1,
                    "before": [b["native"] for b in old], "after": [b["native"] for b in adopted],
                    "revoked": [b["native"] for b in old[2:]], "adopted": [b["native"] for b in adopted[2:]],
                    "closed_branch": 0, "closed_cids": ["样本 cid R0"],
                    "lifecycle": ["BranchClosed", "Resumed", "SessionClosed"],
                    "duty": {"branch": 1, "r": 0, "tail": "HALT", "region": None, "cid": None,
                             "window": [4, 13], "floor": first["anchor"]["origin"],
                             "plan_height": first["anchor"]["origin"], "ref": first["anchor"]["id"], "source_region": None}})
        self.assertEqual(len(replay.chains["g"]), 4)
        self.assertEqual(result["d_head"], 20)
        self.assertEqual(replay.duty["tail"], "HALT")
        self.assertEqual(first["anchor"], cut_anchor(ahead[3]))

    def test_reject_partial_replacement_and_wrong_recovery_parent(self):
        tape = sample_tape()
        stage(tape, "resume")["detail"]["adopted"].pop()
        with self.assertRaisesRegex(ValueError, "entire suffix"):
            project(tape, "sample", {})
        tape = sample_tape()
        replay = state_before(tape, "accepted")
        detail = stage(tape, "accepted")["detail"]
        detail["block"] = {**detail["block"], "physical_parent": detail["block"]["native_parent"]}
        physical = detail["block"]["physical"]
        replay.regional[physical] = (detail["cid"], detail["block"], detail["proof"])
        with self.assertRaisesRegex(ValueError, "exact derived anchor"):
            replay.accept(detail)

    def test_bound_raw_timing_and_failed_generation_preserve_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            data = b'{"seq":2,"tick_us":300}\n'
            path = "raw/trace.jsonl"
            (root / "raw").mkdir()
            (root / path).write_bytes(data)
            sealed = Sealed(root, {"files": {path: {"sha256": sha(data), "bytes": len(data)}}})
            ref = {"path": path, "line": 1, "label": "fixture", "locator": "line 1", "trace_event": True}
            self.assertTrue(sealed.check_ref(ref, {"seq": 2, "tick_us": 300}))
            with self.assertRaisesRegex(ValueError, "differs from source trace"):
                sealed.check_ref(ref, {"seq": 2, "tick_us": 301})
            output = root / "resume.html"
            output.write_text("prior output", encoding="utf-8")
            script = Path(__file__).with_name("generate_resume.py")
            result = subprocess.run([sys.executable, "-B", str(script), "--bundle", str(root),
                                     "--independent", str(root), "--output", str(output)], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(output.read_text(encoding="utf-8"), "prior output")


if __name__ == "__main__":
    unittest.main()
