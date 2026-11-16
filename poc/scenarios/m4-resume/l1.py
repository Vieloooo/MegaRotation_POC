#!/usr/bin/env python3
"""Launch the existing Anvil fixture with one M4 Inbox/derivation driver."""
import importlib.util
import json
import os
from pathlib import Path

upstream = Path(__file__).resolve().parents[1] / "m3-auto-force" / "l1.py"
module_spec = importlib.util.spec_from_file_location("m3_l1", upstream)
m3 = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(m3)

from run_anvil import RESUME_ARGS, codec, encode, rlp, spec


class ResumeL1(m3.CommitteeL1):
    def apply(self, command):
        # Reserve a bounded subrange for the gateway's finalized cursor/history queries.
        return super().apply({**command, "id": 128 * command["id"]})

    def step(self, command):
        op = command["op"]
        if op == "genesis":
            response = super().step(command)
            self.block("B", 0)
            return response
        if op == "m4_prepare_batch":
            self.ready(op="tick", reference_us=command["tick_us"])
            nonce = self.next_nonce["caller"]
            prepared = self.ready(op=op, snapshot=command["snapshot"], nonce=nonce)
            tx_hash = self.ready(op="submit", raw=prepared["raw"])["submitted_tx_hash"]
            if prepared["hash"] != tx_hash or prepared["nonce"] != nonce:
                raise RuntimeError("Anvil submitted a different batch outbox")
            self.next_nonce["caller"] += 1
            self.expectations[tx_hash] = {
                "case": f"batch-h{prepared['height']}", "status": 1, "logs": 1,
            }
            return prepared
        if op == "m4_resume":
            self.ready(op="tick", reference_us=command["tick_us"])
            rid = codec.word(1)
            seq = 1
            policy = [7, 2, 0, 1, 3]
            target, origin = command["target"], command["origin"]
            digest = self.digest("Resume", ["address", "bytes32", "uint64", "bytes32", "uint64", "uint64"],
                                 [self.caller, rid, seq, spec.struct("Policy", spec.POLICY, policy)[1], target, origin])
            signatures = [
                {"member": member, "signature": codec.hx(codec.sign_digest(key, digest))}
                for member, key in self.committee[:self.threshold]
            ]
            packed = b"".join(codec.as_bytes(row["signature"]) for row in signatures)
            calldata = encode(spec.FUNCTIONS["submitResume"], RESUME_ARGS,
                              [rid, seq, policy, target, origin, packed])
            nonce = self.next_nonce["caller"]
            chain = self.config["chain_id"]
            fields = [nonce, 1_000_000_000, 1_000_000, codec.as_bytes(self.authority), 0, calldata]
            tx_digest = codec.keccak256(rlp([*fields, chain, 0, 0]))
            tx_signature = self.signature("caller", tx_digest)
            raw = rlp([*fields, 2 * chain + 35 + tx_signature[64] - 27,
                       int.from_bytes(tx_signature[:32], "big"),
                       int.from_bytes(tx_signature[32:64], "big")])
            tx_hash = codec.hx(codec.keccak256(raw))
            result = {
                "case": "preset_resume", "request": command, "digest": codec.hx(digest),
                "signers": signatures, "signature_pack": codec.hx(packed),
                "hash": tx_hash, "nonce": nonce, "calldata": codec.hx(calldata), "raw": codec.hx(raw),
            }
            path = self.out / "resume-authority.json"
            if path.exists():
                raise RuntimeError("a preset RESUME already occupies the outbox")
            staged = path.with_suffix(".prepared")
            with staged.open("x", encoding="utf-8") as journal:
                json.dump(result, journal, sort_keys=True)
                journal.write("\n")
                journal.flush()
                os.fsync(journal.fileno())
            staged.replace(path)
            directory = os.open(self.out, os.O_DIRECTORY)
            try:
                os.fsync(directory)
            finally:
                os.close(directory)
            submitted = self.ready(op="submit", raw=result["raw"])["submitted_tx_hash"]
            if submitted != tx_hash:
                raise RuntimeError("Anvil submitted a different RESUME outbox")
            self.expectations[tx_hash] = {"case": "preset_resume", "status": 1, "logs": 2}
            self.next_nonce["caller"] += 1
            return result
        return super().step(command)


if __name__ == "__main__":
    m3.baseline.main(ResumeL1)
