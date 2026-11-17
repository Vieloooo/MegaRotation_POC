#!/usr/bin/env python3
"""Reuse the single checked M4 port and the M3 authorization outbox for RESUME."""
import importlib.util
import json
import os
from pathlib import Path

upstream = Path(__file__).resolve().parents[1] / "m4-resume" / "l1.py"
loader = importlib.util.spec_from_file_location("m4_l1", upstream)
m4 = importlib.util.module_from_spec(loader)
loader.loader.exec_module(m4)


class AutomaticResumeL1(m4.ResumeL1):
    def __init__(self, args, port):
        super().__init__(args, port)
        self.config["readers"].extend(["S0", "S1", "S2"])

    def request_digest(self, request):
        resume = request["resume"]
        rid = m4.codec.word(request["request_id"])
        policy = m4.spec.struct("Policy", m4.spec.POLICY, self.policy(request))[1]
        return self.digest("Resume", ["address", "bytes32", "uint64", "bytes32", "uint64", "uint64"],
                           [self.caller, rid, request["control_seq"], policy,
                            resume["target"], resume["origin"]])

    def request_calldata(self, request, packed):
        resume = request["resume"]
        return m4.encode(m4.spec.FUNCTIONS["submitResume"], m4.RESUME_ARGS,
                         [m4.codec.word(request["request_id"]), request["control_seq"],
                          self.policy(request), resume["target"], resume["origin"], packed])

    def submission_expectation(self):
        return {"case": "automatic_resume", "status": 1, "logs": 2}

    def submission_command(self, raw):
        # A held, signed transaction owns no driver command number until actual release.
        if os.environ["M5_SCENARIO"] == "late":
            return None
        return super().submission_command(raw)

    def send_prepared(self, recovery):
        if os.environ["M5_SCENARIO"] != "late":
            return super().send_prepared(recovery)
        prepared = json.loads((self.out / "governance-transaction.json").read_text())
        path = self.out / "governance-release.json"
        if path.exists():
            decision = json.loads(path.read_text())
            command = decision["driver_command"]
            if command["raw"] != prepared["raw"] or decision["hash"] != prepared["hash"]:
                raise RuntimeError("late release differs from the signed outbox")
        else:
            command = super().submission_command(prepared["raw"])
            staged = path.with_suffix(".prepared")
            with staged.open("x", encoding="utf-8") as journal:
                json.dump({"hash": prepared["hash"], "driver_command": command}, journal, sort_keys=True)
                journal.write("\n")
                journal.flush()
                os.fsync(journal.fileno())
            staged.replace(path)
            directory = os.open(self.out, os.O_DIRECTORY)
            try:
                os.fsync(directory)
            finally:
                os.close(directory)
        reply = self.apply(command)
        status = reply["status"]
        if status == "ready" and reply["result"]["submitted_tx_hash"] != prepared["hash"]:
            raise RuntimeError("driver resolved a different held RESUME")
        return {"status": status, "hash": prepared["hash"], "nonce": prepared["nonce"],
                "driver_id": command["id"], "reloaded": recovery}


if __name__ == "__main__":
    m4.m3.baseline.main(AutomaticResumeL1)
