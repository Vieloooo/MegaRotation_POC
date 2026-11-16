#!/usr/bin/env python3
"""M3 scenario port for configured governance members and a real Anvil Authority."""

import hashlib
import json
import os
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "m2-healthy-force"))
import l1 as baseline

from l1 import (
    ANNOUNCEMENT_S, CHAIN_ID, FORCE_ARGS, address, artifact, codec, rlp, spec,
)
from run_anvil import encode


class CommitteeL1(baseline.L1):
    def __init__(self, args, port):
        super().__init__(args, port)
        self.committee = sorted(
            (address(key), key)
            for index in range(3)
            for key in [hashlib.sha256(f"PoC/M3/governance/member/{index}".encode()).digest()]
        )
        self.threshold = 2
        self.config["regional_policy"] = True
        self.bundle_config["committee"] = [member for member, _ in self.committee]
        self.bundle_config["threshold"] = self.threshold

    def policy(self, request):
        return [request["enabled_mask"], *request["order"], request["signed_floor"]]

    def force_digest(self, request):
        policy = self.policy(request)
        args = [self.caller, codec.word(request["request_id"]), request["control_seq"],
                request["target_beta"], spec.struct("Policy", spec.POLICY, policy)[1]]
        return self.digest("Force", ["address", "bytes32", "uint64", "uint64", "bytes32"], args)

    def deploy(self):
        # The address array is the sole dynamic constructor component.
        addresses = [member for member, _ in self.committee]
        constructor = codec.abi_encode(
            ["bytes32", "uint256", "uint8", "address", "address", "address", "uint64", "uint64", "uint64"],
            [self.salt, 9 * 32, self.threshold, self.caller, self.addresses["certifier"],
             self.addresses["da"], ANNOUNCEMENT_S, 0, 3],
        )
        constructor += codec.word(len(addresses))
        constructor += codec.abi_encode(["address"] * len(addresses), addresses)
        data = artifact(self.args.contracts, "CommitteeControlAuthority") + constructor
        return [self.send_transaction("deploy_committee_authority", None, data, 8_000_000, 1, 1)]

    def sign_request(self, signer, request):
        if type(signer) is not int or not 0 <= signer < len(self.committee):
            raise ValueError("invalid governance signer index")
        member, key = self.committee[signer]
        digest = self.force_digest(request)
        signature = codec.sign_digest(key, digest)
        return {"signer": signer, "address": member, "digest": codec.hx(digest),
                "signature": codec.hx(signature), "request": request}

    def prepare(self, request, signers):
        if len(signers) != self.threshold:
            raise ValueError("wrong signature count")
        addresses = sorted(row["address"] for row in signers)
        if len(set(addresses)) != len(addresses):
            raise ValueError("duplicate signature")
        digest = self.force_digest(request)
        for row in signers:
            signer = row["signer"]
            if (row["request"] != request or row["digest"] != codec.hx(digest)
                    or not 0 <= signer < len(self.committee)
                    or row["address"] != self.committee[signer][0]):
                raise ValueError("signature output not bound to request and signer")
        packed = b"".join(codec.as_bytes(row["signature"]) for row in sorted(signers, key=lambda s: s["address"]))
        calldata = encode(spec.FUNCTIONS["submitForce"], FORCE_ARGS,
                          [codec.word(request["request_id"]), request["control_seq"], request["target_beta"],
                           self.policy(request), packed])
        nonce = self.next_nonce["caller"]
        fields = [nonce, 1_000_000_000, 1_000_000, codec.as_bytes(self.authority), 0, calldata]
        tx_digest = codec.keccak256(rlp([*fields, CHAIN_ID, 0, 0]))
        sig = self.signature("caller", tx_digest)
        raw = rlp([*fields, 2 * CHAIN_ID + 35 + sig[64] - 27,
                   int.from_bytes(sig[:32], "big"), int.from_bytes(sig[32:64], "big")])
        self.command += 1
        prepared = {
            "request": request, "signers": signers, "digest": codec.hx(digest),
            "signature_pack": codec.hx(packed), "nonce": nonce,
            "hash": codec.hx(codec.keccak256(raw)), "raw": codec.hx(raw),
            "calldata": codec.hx(calldata),
            "driver_command": {"id": self.command, "op": "submit", "raw": codec.hx(raw)},
        }
        path = self.out / "governance-transaction.json"
        staged = path.with_suffix(".prepared")
        if path.exists():
            raise RuntimeError("a governance transaction is already prepared")
        with staged.open("x") as journal:
            json.dump(prepared, journal, sort_keys=True)
            journal.write("\n")
            journal.flush()
            os.fsync(journal.fileno())
        staged.replace(path)
        data_dir = os.open(self.out, os.O_DIRECTORY)
        try:
            os.fsync(data_dir)
        finally:
            os.close(data_dir)
        self.next_nonce["caller"] += 1
        self.expectations[prepared["hash"]] = {"case": "automatic_force", "status": 1, "logs": 1}
        return prepared

    def send_prepared(self, recovery):
        with (self.out / "governance-transaction.json").open() as journal:
            prepared = json.load(journal)
        reply = self.apply(prepared["driver_command"])
        status = reply["status"]
        if status == "ready" and reply["result"]["submitted_tx_hash"] != prepared["hash"]:
            raise RuntimeError("driver resolved a different governance transaction")
        return {"status": status, "hash": prepared["hash"], "nonce": prepared["nonce"],
                "driver_id": prepared["driver_command"]["id"], "reloaded": recovery}

    def step(self, command):
        operation = command["op"]
        if operation == "governance_sign":
            return self.sign_request(command["signer"], command["request"])
        if operation == "governance_prepare":
            # Reserve the Submit command immediately after publishing this tick.
            self.ready(op="tick", reference_us=command["tick_us"])
            return self.prepare(command["request"], command["signers"])
        if operation == "governance_broadcast":
            return self.send_prepared(False)
        if operation == "governance_reconcile":
            self.ready(op="tick", reference_us=command["tick_us"])
            return self.send_prepared(True)
        if operation == "deploy":
            self.ready(op="tick", reference_us=command["tick_us"])
            return self.deploy()
        return super().step(command)


if __name__ == "__main__":
    baseline.main(CommitteeL1)
