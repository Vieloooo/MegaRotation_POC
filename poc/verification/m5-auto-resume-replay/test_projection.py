import copy
import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from generate_auto_resume import compact
from model import project, target
from sample import build, scenarios
from sealed import TAPE, load, sha


def events(tape, kind):
    return [event for event in tape["events"] if event["kind"] == kind]


class ProjectionTests(unittest.TestCase):
    def test_samples_keep_per_scenario_chain_and_timing_state(self):
        stopped, frozen, no_target = [project(tape, "sample", {}) for tape in scenarios()]
        for page in (stopped, frozen):
            frames = {event["stage"]: event for event in page["frames"] if event["stage"]}
            self.assertEqual(frames["local-stop"]["duty"]["tail"], "NEXT")
            self.assertEqual(frames["wait"]["watermark"]["theta_us"], 11_040_000)
            self.assertEqual(frames["expire"]["pc"]["key_us"], 24_000_000)
            self.assertEqual(frames["expire"]["duty"]["tail"], "HALT")
            self.assertEqual(frames["signed"]["detail"]["t_eligible_us"], 24_800_000)
            self.assertEqual(len(frames["resume"]["chains"]["g"]), 31)
            self.assertEqual(len(frames["accepted"]["chains"]["g"]), 32)
        self.assertGreater(
            next(frame["pc"]["index"][0] for frame in frozen["frames"] if frame["stage"] == "wait"),
            next(frame["pc"]["index"][0] for frame in stopped["frames"] if frame["stage"] == "wait"),
        )
        self.assertIsNone(no_target["frames"][-1]["signer_views"][0]["target"])

    def test_browser_pool_keeps_every_chain_and_signer_snapshot(self):
        page = project(scenarios()[1], "sample", {})
        packaged = compact(page)
        pools = packaged["pools"]
        self.assertLess(len(pools["chains"]), len(page["frames"]))
        for old, frame in zip(page["frames"], packaged["frames"], strict=True):
            self.assertEqual(frame["seq"], old["seq"])
            self.assertEqual({name: pools["chains"][key] for name, key in frame["chains"].items()},
                             old["chains"])
            for who, key in frame["signer_views"].items():
                observed = packaged["frames"][key]
                details = {field:pools["values"][value] if field in packaged["signer_detail"] else value
                           for field, value in observed["detail"].items()}
                fields = {**details, **observed["observation"],
                          "solver":pools["values"][observed["observation"]["solver"]],
                          "refs":[pools["refs"][i] for i in observed["refs"]],
                          "seq":observed["seq"],"suborder":observed.get("suborder",0),"tick_us":observed["tick_us"]}
                self.assertEqual(fields, old["signer_views"][who])

    def test_target_recomputes_integer_bound_and_advancement(self):
        tape = scenarios()[0]
        profile = tape["profile"]
        original = events(tape, "signer_view")[0]["detail"]
        solved = target(profile, original)
        self.assertEqual((solved["minimum_height"], solved["maximum_height"]), (30, 30))
        for changes in ({"h_guard":15}, {"signed_origin":14}, {"target":31},
                        {"guard_parts":{**original["guard_parts"], "extra_origin":3}}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                target(profile, {**original, **changes})

    def test_votes_use_latest_complete_same_request_quorum(self):
        for key, value in (("eligible", False), ("request", "f"*64),
                           ("pc", {"kind":"Expire", "index":[2], "key_us":24_000_000}),
                           ("d_drained", False)):
            tape = scenarios()[0]
            events(tape, "signer_view")[1]["detail"][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                project(tape, "sample", {})

    def test_stopped_and_frozen_intervals_reject_unlicensed_advance(self):
        tape = scenarios()[1]
        frozen = next(event for event in tape["events"] if event["stage"] == "frozen-q")
        for changes in ({"theta_us":12_000_000}, {"objects":[{"body": "unexpected"}]}):
            altered = copy.deepcopy(tape)
            next(event for event in altered["events"] if event["seq"] == frozen["seq"])["detail"].update(changes)
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                project(altered, "sample", {})

    def test_local_clock_cannot_apply_public_expire_early(self):
        tape = scenarios()[0]
        events(tape, "local_stop")[0]["detail"]["clock_us"] += 3_000_000
        with self.assertRaisesRegex(ValueError, "stop"):
            project(tape, "sample", {})
        unresolved = scenarios()[0]
        timer = events(unresolved, "g_timer")[0]
        timer["detail"]["coverage"]["q"] = 0
        with self.assertRaisesRegex(ValueError, "watermark"):
            project(unresolved, "sample", {})

    def test_sealed_normalized_input_checks_run_trace_and_checker(self):
        with TemporaryDirectory() as directory:
            bundle = Path(directory) / "bundle"
            checker = Path(directory) / "checker"
            bundle.mkdir()
            checker.mkdir()
            tape = scenarios()[-1]
            tape["mode"] = "observed"
            for number, event in enumerate(tape["events"], 1):
                event["refs"] = [{"label":"test source", "locator":f"line {number}", "path":"trace.jsonl",
                                  "line":number, "trace_event":True}]
            trace = b"".join((json.dumps(row) + "\n").encode() for row in tape["events"])
            (bundle / "trace.jsonl").write_bytes(trace)
            manifest = {"status":"COMPLETED", "seed":42, "candidate":"e"*40,
                        "schema":"M5-AUTO-RESUME-RUN-1", "before":{"head":"e"*40,"status":""},
                        "after":{"head":"e"*40,"status":""},
                        "files":{"trace.jsonl":{"sha256":sha(trace), "bytes":len(trace)}}}
            data = json.dumps(manifest).encode()
            (bundle/"manifest.json").write_bytes(data)
            (bundle/"manifest.sha256").write_text(sha(data) + "  manifest.json\n")
            tape_bytes = json.dumps(tape).encode()
            (checker/TAPE).write_bytes(tape_bytes)
            report = {"status":"PASS", "candidate":"e"*40, "manifest_sha256":sha(data),
                      "auto_resume_replay_sha256":sha(tape_bytes)}
            (checker/"report.json").write_text(json.dumps(report))
            self.assertEqual(len(load(bundle, checker)["frames"]), len(tape["events"]))
            (bundle/"trace.jsonl").write_bytes(trace + b"\n")
            with self.assertRaisesRegex(ValueError, "digest"):
                load(bundle, checker)
            (bundle/"trace.jsonl").write_bytes(trace)
            (checker/"report.json").write_text(json.dumps({**report,"candidate":"f"*40}))
            with self.assertRaisesRegex(ValueError, "candidate"):
                load(bundle, checker)

    def test_upper_slot_and_late_inclusion_keep_original_target(self):
        for height, act, tail in ((14, 38_000_000, "NEXT"), (22, 54_000_000, "HALT")):
            page = project(build("stopped", True, include_height=height), "sample", {})
            receipt = next(frame for frame in page["frames"] if frame["stage"] == "included")
            applied = next(frame for frame in page["frames"] if frame["stage"] == "resume")
            self.assertEqual(receipt["detail"]["target"], 30)
            self.assertEqual(receipt["detail"]["act_us"], act)
            self.assertEqual(applied["duty"]["tail"], tail)
            self.assertEqual(len(applied["chains"]["g"]), 31)

    def test_signed_window_and_derived_parent(self):
        tape = scenarios()[0]
        events(tape, "resume_submitted")[0]["detail"]["inclusion"] = [28_000_000, 28_000_000]
        with self.assertRaisesRegex(ValueError, "timing"):
            project(tape, "sample", {})
        bad = scenarios()[0]
        for kind in ("regional_final", "g_accepted"):
            events(bad, kind)[-1]["detail"]["block"]["physical_parent"] = "f"*64
        with self.assertRaisesRegex(ValueError, "anchor"):
            project(bad, "sample", {})


if __name__ == "__main__":
    unittest.main()
