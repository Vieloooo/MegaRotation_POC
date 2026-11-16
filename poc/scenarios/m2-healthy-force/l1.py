#!/usr/bin/env python3
"""The healthy Force scenario's synchronous SimHub-to-Anvil command port."""

import argparse
import json
from pathlib import Path
import socket
import subprocess
import sys
import time
from urllib.error import URLError

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "poc/crates/l1-adapter/tests"))
from run_anvil import (  # Reuse the locked Anvil component fixture's transaction codec.
    FIXTURE_KEYS,
    FORCE_ARGS,
    Scenario,
    address,
    artifact,
    codec,
    rlp,
    save,
    spec,
)

CHAIN_ID = 31337
GOVERNANCE_CASE = "scheduled_healthy_force"
ANNOUNCEMENT_S = 10
FINALITY_US = 1_500_000
POLICY = [7, 0, 1, 2, 3]


class L1(Scenario):
    def __init__(self, args, port):
        super().__init__(args, port, args.output)
        salt = codec.as_bytes(args.config_ref)
        if len(salt) != 32:
            raise ValueError("expected the PoC public configuration reference")
        self.salt = salt
        self.config.update(
            chain_id=CHAIN_ID,
            config_ref=codec.hx(salt),
            genesis_origin=3,
            genesis_timestamp_seconds=0,
            announcement_seconds=ANNOUNCEMENT_S,
            finality_delay_us=FINALITY_US,
        )
        self.bundle_config.update(
            chainId=CHAIN_ID,
            configRef=codec.hx(salt),
            genesisOrigin=3,
            genesisTimestampSeconds=0,
            announcementSeconds=ANNOUNCEMENT_S,
        )

    def send_transaction(self, label, to, data, gas, status, logs):
        nonce = self.next_nonce["caller"]
        self.next_nonce["caller"] += 1
        fields = [nonce, 1_000_000_000, gas, codec.as_bytes(to) if to else b"", 0, data]
        digest = codec.keccak256(rlp([*fields, CHAIN_ID, 0, 0]))
        sig = self.signature("caller", digest)
        raw = rlp(
            [*fields, 2 * CHAIN_ID + 35 + sig[64] - 27,
             int.from_bytes(sig[:32], "big"), int.from_bytes(sig[32:64], "big")]
        )
        tx_hash = self.ready(op="submit", raw=codec.hx(raw))["submitted_tx_hash"]
        if tx_hash != codec.hx(codec.keccak256(raw)):
            raise RuntimeError("driver did not submit the signed transaction")
        self.expectations[tx_hash] = {"case": label, "status": status, "logs": logs}
        return {
            "case": label, "hash": tx_hash, "nonce": nonce, "calldata": codec.hx(data),
        }

    def submit_force(self, label, rid, seq, branch, signer, status, logs):
        calldata, digest, signature = self.force(rid, seq, branch, POLICY, signer=signer)
        entry = self.send_transaction(label, self.authority, calldata, 1_000_000, status, logs)
        entry.update(digest=codec.hx(digest), signature=codec.hx(signature))
        return entry

    def block(self, reader, height):
        reply = self.ready(op="read", reader=reader, query={"kind": "block", "height": height})
        if reply["control_coverage_through"] != height:
            raise RuntimeError("L1 reader lacks complete control coverage")
        return reply["block"]

    def step(self, command):
        tick = command["tick_us"]
        self.ready(op="tick", reference_us=tick)
        operation = command["op"]
        if operation == "genesis":
            block = self.block("A", 0)
            self.blocks.append(block["header_and_transactions"])
            return block
        if operation == "deploy":
            args = ["bytes32", "address", "address", "address", "address", "uint64", "uint64", "uint64"]
            values = [self.salt, address(FIXTURE_KEYS["governance"]), self.caller,
                      self.addresses["certifier"], self.addresses["da"], ANNOUNCEMENT_S, 0, 3]
            data = artifact(self.args.contracts, "ControlAuthority") + codec.abi_encode(args, values)
            return [self.send_transaction("deploy_authority", None, data, 8_000_000, 1, 1)]
        if operation == "submit":
            return [self.submit_force(GOVERNANCE_CASE, 1, 1, 0, "governance", 1, 1)]
        if operation == "retry":
            return [
                self.submit_force("identical_retry", 1, 1, 0, "governance", 1, 0),
                self.submit_force("wrong_signer", 1, 1, 0, "other", 0, 0),
                self.submit_force("stale_sequence", 2, 0, 0, "governance", 0, 0),
                self.submit_force("conflicting_request", 1, 2, 0, "governance", 0, 0),
            ]
        if operation == "mine":
            slot = command["height"]
            block = self.ready(op="mine", slot=slot)["included_block"]
            if block["number"] != slot or block["timestamp"] * 1_000_000 != tick:
                raise RuntimeError("included L1 header disagrees with SimHub slot")
            self.blocks.append(block["header_and_transactions"])
            self.receipts.extend(block["receipts"])
            for receipt in block["receipts"]:
                tx_hash = receipt["transactionHash"]
                expected = self.expectations[tx_hash]
                actual = {"status": int(receipt["status"], 16), "logs": len(receipt["logs"])}
                if actual != {key: expected[key] for key in actual}:
                    raise RuntimeError(f"{expected['case']} receipt disagrees: {actual}")
                self.acceptance[expected["case"]] = {"hash": tx_hash, **actual}
            return block
        if operation == "finalize":
            return self.ready(op="finalize", through_height=command["height"])
        if operation == "deliver":
            height, reader = command["height"], command["reader"]
            result = self.ready(op="deliver", reader=reader, through_height=height)
            if result["delivered_through"] != height:
                raise RuntimeError("incomplete L1 reader prefix")
            return self.block(reader, height)
        raise ValueError(f"unknown L1 command {operation!r}")

    def record(self):
        save(self.out / "driver-trace.json", self.trace)
        save(self.out / "raw-rpc-bundle.json", {
            "config": self.bundle_config,
            "blocks": self.blocks,
            "receipts": self.receipts,
            "historicalCursorReads": [],
        })
        save(self.out / "receipts-by-case.json", self.acceptance)

    def run_port(self):
        with (self.out / "driver.stderr").open("w") as stderr:
            self.driver = subprocess.Popen(
                [str(self.args.driver), f"127.0.0.1:{self.port}", str(self.out / "driver-config.json")],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True,
            )
            try:
                self.trace.append(json.loads(self.driver.stdout.readline()))
                with (self.out / "hub-l1.jsonl").open("x") as trace:
                    for line in sys.stdin:
                        command = json.loads(line)
                        answer = self.step(command)
                        trace.write(json.dumps({"command": command, "result": answer}) + "\n")
                        trace.flush()
                        print(json.dumps(answer, separators=(",", ":")), flush=True)
            finally:
                self.driver.stdin.close()
                if self.driver.wait(timeout=5) != 0 and not sys.exc_info()[0]:
                    raise RuntimeError("L1 adapter exited unsuccessfully")
                self.driver.stdout.close()


def main(port_class=L1):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--anvil", required=True, type=Path)
    parser.add_argument("--driver", required=True, type=Path)
    parser.add_argument("--contracts", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--config-ref", required=True)
    args = parser.parse_args()
    args.output.mkdir()
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    l1 = port_class(args, port)
    save(args.output / "driver-config.json", l1.config)
    genesis = args.output / "anvil-genesis.json"
    forks = ("homestead", "eip150", "eip155", "eip158", "byzantium",
             "constantinople", "petersburg", "istanbul", "berlin", "london")
    save(genesis, {
        "config": {"chainId": CHAIN_ID, **{f"{fork}Block": 0 for fork in forks}, "shanghaiTime": 0},
        "timestamp": "0x0",
        "gasLimit": hex(100_000_000),
        "difficulty": "0x0",
        "alloc": {l1.caller: {"balance": hex(10**22)}},
    })
    argv = [
        str(args.anvil), "--host", "127.0.0.1", "--port", str(port), "--quiet", "--no-mining",
        "--order", "fifo", "--no-cors", "--accounts", "0", "--threads", "2", "--hardfork", "shanghai",
        "--timestamp", "0", "--chain-id", str(CHAIN_ID), "--gas-limit", "100000000",
        "--disable-default-create2-deployer", "--init", str(genesis),
    ]
    save(args.output / "anvil-command.json", argv)
    with (args.output / "anvil.log").open("w") as logs:
        anvil = subprocess.Popen(argv, stdout=logs, stderr=subprocess.STDOUT)
        try:
            for _ in range(100):
                if anvil.poll() is not None:
                    raise RuntimeError("Anvil exited during setup")
                try:
                    if l1.probe("eth_chainId") != hex(CHAIN_ID):
                        raise RuntimeError("Anvil chain ID mismatch")
                    break
                except (URLError, ConnectionError):
                    time.sleep(0.025)
            else:
                raise RuntimeError("Anvil setup timed out")
            l1.run_port()
        finally:
            l1.record()
            if anvil.poll() is None:
                anvil.terminate()
                try:
                    anvil.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    anvil.kill()
                    anvil.wait()


if __name__ == "__main__":
    main()
