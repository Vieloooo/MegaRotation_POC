#!/usr/bin/env python3
"""Write exact fixture transactions to private Anvil and exercise the 09/11/12 reader."""

from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

REPO = Path(__file__).resolve().parents[4]
SPEC = importlib.util.spec_from_file_location("fixture_l1", REPO / "poc/crates/l1-adapter/tests/run_anvil.py")
l1 = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(l1)
sys.path.insert(0, str(REPO / "poc/verification/batch-services"))

from frame_vectors import encode as encode_frame, tx

CASES = ("same_block", "before", "boundary", "after", "substitute", "future", "malformed")
AMOUNT = 3


def row(height, origin, amount=AMOUNT):
    return {"height": height, "origin": origin, "transactions": [tx(1, 2, 0, amount)]}


def scenario_class(case, vector_port, profile):
    class Scenario(l1.Scenario):
        def deploy_authority(self):
            roles = self.addresses
            types = ["bytes32", "address", "address", "address", "address", "uint64", "uint64", "uint64"]
            values = [l1.SALT, roles["governance"], self.caller, roles["certifier"], roles["da"], 6, 0, 0]
            self.tx(
                "deploy-authority-and-inbox", None,
                l1.artifact(self.args.contracts, "ControlAuthority") + l1.codec.abi_encode(types, values),
                gas=8_000_000, logs=1,
            )
            self.mine(1)

        def submit_batch(self, label, request, previous, rows, *, malformed=False):
            # Use exactly the frozen frame encoding (or the labeled malformed object).
            payload = b"invalid complete BS-FRAME-v1 bytes" if malformed else encode_frame(rows)
            gpos = b"m4-input-vector/" + case.encode("ascii")
            cursor = l1.global_cursor(rows[-1]["height"], gpos, rows[-1]["origin"])
            body, _, _ = self.batch_args(previous, cursor, payload=payload)
            commitment = l1.codec.hx(body[2])
            (self.out / "da" / f"{commitment}.bin").write_bytes(payload)
            self.tx(label, self.inbox, self.batch(request, body), logs=1)
            return commitment

        def read_through(self, start, last):
            self.ready(op="tick", reference_us=(last + 1) * 2_000_000)
            self.ready(op="finalize", through_height=last)
            self.ready(op="deliver", reader="A", through_height=last)
            for height in range(start, last + 1):
                self.ready(op="read", reader="A", query={"kind": "block", "height": height})
            if start + 1 <= last:
                self.ready(op="deliver", reader="B", from_height=start + 1, through_height=last)
                self.expect("wait", op="read", reader="B", query={"kind": "block", "height": last})
            self.ready(op="deliver", reader="B", from_height=start, through_height=start)
            for height in range(last, start - 1, -1):
                self.ready(op="read", reader="B", query={"kind": "block", "height": height})

        def run_vector(self, vector):
            self.driver.stdin.write(json.dumps({"vector": vector}) + "\n")
            self.driver.stdin.flush()
            raw = self.driver.stdout.readline()
            if not raw:
                raise RuntimeError(f"vector {vector} failed; inspect driver.stderr")
            result = json.loads(raw)
            assert result["status"] == "PASS", result
            l1.save(self.out / f"{vector}.json", result)

        def run(self):
            l1.save(self.out / "derivation-profile.json", profile)
            (self.out / "da").mkdir()
            with (self.out / "driver.stderr").open("w") as errors:
                self.driver = subprocess.Popen(
                    [
                        str(vector_port), "live", f"127.0.0.1:{self.port}",
                        str(self.out / "driver-config.json"), self.addresses["certifier"],
                        self.addresses["da"], str(self.out),
                    ],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors, text=True,
                )
                try:
                    startup = self.driver.stdout.readline()
                    if not startup:
                        raise RuntimeError("vector port exited; inspect driver.stderr")
                    entries = json.loads(startup)["startup_rpc"]
                    self.trace.append({"startup_rpc": entries})
                    self.blocks.append(next(
                        row["result"]["Ok"] for row in entries
                        if row["request"]["method"] == "eth_getBlockByNumber"
                    ))
                    self.deploy_authority()
                    self.mine(2)
                    if case == "same_block":
                        self.tx("resume-first", self.authority, self.resume(11, 1, 1, 0)[0], logs=2)
                        self.submit_batch("intervening-ordinary-batch", 12, l1.resume_cursor(1, 1), [row(2, 1)])
                        self.tx("resume-second", self.authority, self.resume(13, 2, 3, 1)[0], logs=2)
                        # The D port blocks before the second Set until its exact DA capability.
                        self.mine(3)
                    elif case in ("before", "substitute", "malformed"):
                        records = [row(1, 1, 1), row(2, 1)] if case == "substitute" else [row(1, 0)]
                        self.submit_batch(case, 12, l1.GENESIS, records, malformed=case == "malformed")
                        self.mine(3)
                    elif case == "boundary":
                        self.mine(3)
                        self.submit_batch(case, 12, l1.GENESIS, [row(1, 0)])
                        self.mine(4)
                    else:
                        if case == "future":
                            self.tx("resume-to-two", self.authority, self.resume(11, 1, 2, 1)[0], logs=2)
                            previous = l1.resume_cursor(2, 1, 1)
                            records = [row(3, 0, 1), row(4, 2)]
                        else:
                            previous, records = l1.GENESIS, [row(1, 0)]
                        self.mine(3)
                        self.mine(4)
                        self.submit_batch(case, 12, previous, records)
                        self.mine(5)
                    last = int(self.blocks[-1]["number"], 16)
                    for reader in ("A", "B"):
                        self.ready(op="read", reader=reader, query={"kind": "block", "height": 0})
                    self.read_through(1, last)
                    self.run_vector(case)
                    if case == "same_block":
                        for height in range(4, 7):
                            self.mine(height)
                        self.read_through(4, 6)
                        self.run_vector("same_block_tail")
                finally:
                    self.driver.stdin.close()
                    status = self.driver.wait(timeout=10)
                    self.driver.stdout.close()
                    if sys.exc_info()[0] is None and status:
                        raise RuntimeError(f"vector port exited {status}; inspect driver.stderr")

    return Scenario


def main():
    extra = argparse.ArgumentParser(add_help=False)
    extra.add_argument("--case", choices=CASES, required=True)
    extra.add_argument("--vector-port", type=Path, required=True)
    args, remaining = extra.parse_known_args()
    profile = json.loads(subprocess.check_output([args.vector_port, "profile"], text=True))
    l1.SALT = l1.codec.as_bytes(profile["config_ref"])
    l1.TIME = profile["config"]["public"]["l1_genesis_timestamp_us"] // 1_000_000
    l1.Scenario = scenario_class(args.case, args.vector_port, profile)
    sys.argv[1:] = [*remaining, "--drop-mine-answer", "0"]
    l1.main()


if __name__ == "__main__":
    main()
