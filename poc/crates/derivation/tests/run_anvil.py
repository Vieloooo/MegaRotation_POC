#!/usr/bin/env python3
"""Create fresh 09 receipts with explicit slots and independently named reader deliveries."""
import argparse
import importlib.util
from pathlib import Path
import json
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
HELPER = ROOT / "poc/crates/l1-adapter/tests/run_anvil.py"
spec = importlib.util.spec_from_file_location("feature09_anvil", HELPER)
l1 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(l1)


def main():
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--case", choices=("window", "ahead", "cap", "accepted_wait"), required=True)
    parser.add_argument("--derivation", type=Path, required=True)
    args, remaining = parser.parse_known_args()
    profile = json.loads(subprocess.check_output([args.derivation, "profile"], text=True))
    l1.TIME = profile["genesis"]["anchor"]["timestamp_us"] // 1_000_000
    l1.SALT = l1.codec.as_bytes(profile["config_ref"])

    class Scenario(l1.Scenario):
        def deploy(self):
            roles = self.addresses
            types = ["bytes32", "address", "address", "address", "address", "uint64", "uint64", "uint64"]
            values = [l1.SALT, roles["governance"], self.caller, roles["certifier"], roles["da"], 6, 0, 0]
            self.tx("deploy-authority-and-inbox", None, l1.artifact(self.args.contracts, "ControlAuthority")
                    + l1.codec.abi_encode(types, values), gas=8_000_000, logs=1)
            self.tx("unrelated-rollback-fixture-emitter", None, l1.artifact(self.args.contracts, "RollbackFixture")
                    + l1.codec.abi_encode(["bytes32", "address", "address", "address"],
                                          [l1.SALT, self.caller, roles["certifier"], roles["da"]]),
                    gas=8_000_000, logs=1)
            time.sleep(self.args.wall_pause_ms / 1000)
            self.mine(1)
            assert self.evm_call(self.authority, l1.encode("configRef()"), 1) == profile["config_ref"]

        def submit_resume(self, label, rid, seq, target):
            self.tx(label, self.authority, self.resume(rid, seq, target, 0)[0], logs=2)

        def at_slot(self, slot):
            if slot == 2:
                self.tx("force-no-derivation-effect", self.authority, self.force(100, 0, 0)[0], logs=1)
                self.tx("failed-malformed-call", self.authority, b"\x01", ok=False, logs=0)
                self.tx("successful-unrelated-call-without-event", self.addresses["other"], b"", logs=0)
            if args.case == "window" and slot == 12:
                self.submit_resume("fill-to-origin-ten", 101, 1, 20)
            if args.case == "ahead":
                if slot == 3:
                    self.submit_resume("cursor-five", 101, 1, 5)
                if slot == 45:
                    self.submit_resume("same-block-first-d-ahead", 102, 2, 8)
                    self.submit_resume("same-block-second-fills", 103, 3, 11)
            if args.case == "cap" and slot == 3:
                self.submit_resume("fill-above-including-l1-time-and-drift", 101, 1, 30)
            if args.case == "accepted_wait" and slot == 40:
                # A real Inbox receipt, used solely to demonstrate Stage A refuses to open it.
                # The opaque commitment has no feature-11 frame or verified DA content.
                next_cursor = l1.global_cursor(1, b"\x01", 0)
                body, _, _ = self.batch_args(l1.GENESIS, next_cursor, payload=b"feature12-stageA-opaque-receipt-only")
                self.tx("accepted-opaque-receipt-only", self.inbox, self.batch(101, body), logs=1)
                self.submit_resume("blocked-by-earlier-acceptance", 102, 1, 2)

        def read_history(self, last):
            self.ready(op="tick", reference_us=(last + 1) * 2_000_000)
            self.ready(op="finalize", through_height=last)
            for reader in ("A", "B"):
                self.ready(op="read", reader=reader, query={"kind": "block", "height": 0})
            self.ready(op="deliver", reader="A", through_height=last)
            for height in range(1, last + 1):
                self.ready(op="read", reader="A", query={"kind": "block", "height": height})
            self.ready(op="deliver", reader="B", from_height=2, through_height=last)
            self.expect("wait", op="read", reader="B", query={"kind": "block", "height": last})
            self.ready(op="deliver", reader="B", from_height=1, through_height=1)
            for height in range(last, 0, -1):
                self.ready(op="read", reader="B", query={"kind": "block", "height": height})

        def run(self):
            l1.save(self.out / "derivation-profile.json", profile)
            with (self.out / "driver.stderr").open("w") as errors:
                self.driver = subprocess.Popen(
                    [str(self.args.driver), f"127.0.0.1:{self.port}", str(self.out / "driver-config.json"), "0"],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors, text=True,
                )
                try:
                    startup = json.loads(self.driver.stdout.readline())["startup_rpc"]
                    self.trace.append({"startup_rpc": startup})
                    self.blocks.append(next(row["result"]["Ok"] for row in startup
                                            if row["request"]["method"] == "eth_getBlockByNumber"))
                    self.deploy()
                    last = {"window": 51, "ahead": 46, "cap": 4, "accepted_wait": 41}[args.case]
                    for slot in range(2, last + 1):
                        self.at_slot(slot)
                        self.mine(slot)
                    self.read_history(last)
                    assert len(self.acceptance) == len(self.expectations)
                finally:
                    self.driver.stdin.close()
                    self.driver.wait(timeout=5)
                    self.driver.stdout.close()

    l1.Scenario = Scenario
    sys.argv[1:] = [*remaining, "--drop-mine-answer", "0"]
    l1.main()


if __name__ == "__main__":
    main()
