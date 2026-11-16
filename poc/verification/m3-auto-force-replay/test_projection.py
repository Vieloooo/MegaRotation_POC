import unittest

from model import historical_choice, signature_set, signer_view
from sample import PROFILE, poll, sample_projection, tip


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


if __name__ == "__main__":
    unittest.main()
