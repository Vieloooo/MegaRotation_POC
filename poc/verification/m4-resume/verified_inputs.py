"""Receipt and DA verification for the M4 offline oracle."""

import hashlib
from dataclasses import dataclass

import golden_vectors as golden
import independent_codec as crypto
import receipt_checker as receipt
from authority import CommitteeReceiptChecker, decode_deployment
from frame_vectors import decode as decode_frame


def need(ok, detail):
    if not ok:
        raise ValueError(detail)


@dataclass(frozen=True)
class Batch:
    position: tuple
    block_hash: str
    tx_hash: str
    log_index: int
    commitment: str
    gpos: bytes
    previous_height: int
    records: list
    payload_sha256: str


@dataclass(frozen=True)
class Resume:
    position: tuple
    set_position: tuple
    tx_hash: str
    block_hash: str
    set_log_index: int
    resume_log_index: int
    rid: str
    seq: int
    target: int
    signed_origin: int
    policy: dict
    signed_floor: int
    act_us: int
    signers: tuple


class ResumeReceiptChecker(CommitteeReceiptChecker):
    def control_args(self, name, args, sender):
        if name == "submitForce":
            return super().control_args(name, args, sender)
        need(name == "submitResume", "unknown control")
        rid, seq, policy, target, origin, packed = args
        digest, _ = self.resume_signers(sender, rid, seq, policy, target, origin, packed)
        return rid, seq, digest

    def resume_signers(self, sender, rid, seq, policy, target, origin, packed):
        need(policy[0] < 8 and sorted(policy[1:4]) == [0, 1, 2], "invalid Resume policy")
        policy_hash = golden.struct("Policy", golden.POLICY, policy)[1]
        signed = golden.struct(
            "Resume",
            ["address", "bytes32", "uint64", "bytes32", "uint64", "uint64"],
            [sender, rid, seq, policy_hash, target, origin],
        )[1]
        digest = crypto.typed_digest(self.domains["authority"], signed)
        need(len(packed) == self.committee.threshold * 65, "wrong Resume signer count")
        signers = tuple(
            crypto.recover_address(packed[i:i + 65], digest)
            for i in range(0, len(packed), 65)
        )
        need(
            signers == tuple(sorted(set(signers)))
            and set(signers) <= set(self.committee.members),
            "invalid or noncanonical Resume signer set",
        )
        return signed, tuple(crypto.hx(signer) for signer in signers)


def project(bundle, authority_artifact, da_index, da_root):
    """Open exact data bound by successful receipts, after verifying the receipt history."""
    committee = decode_deployment(bundle, authority_artifact)
    checker = ResumeReceiptChecker(bundle, committee)
    result = checker.run()
    headers = {receipt.q(block["number"]): block for block in bundle["blocks"]}
    receipts = {r["transactionHash"].lower(): r for r in bundle["receipts"]}
    da_root = da_root.resolve(strict=True)
    batches, resumes = [], []
    for row in result["events"]:
        kind = row["kind"]
        if kind not in ("CursorAdvanced", "ResumeRecorded"):
            continue
        position = tuple(row["position"])
        block = headers[position[0]]
        transaction = block["transactions"][position[1]]
        tx_hash = transaction["hash"].lower()
        tx_receipt = receipts[tx_hash]
        log = tx_receipt["logs"][position[2]]
        name, args = receipt.decode_call(transaction)
        if kind == "CursorAdvanced":
            need(name == "submitBatch", "CursorAdvanced carrier")
            _, (prev, next_, commitment, _, first, last, gpos, _) = args
            key = crypto.hx(commitment)
            need(key in da_index, f"missing authenticated batch DA object {key}")
            path = (da_root / da_index[key]).resolve(strict=True)
            need(path.is_relative_to(da_root) and path.is_file(), "DA object escapes evidence root")
            raw = path.read_bytes()
            need(crypto.keccak256(raw) == commitment, f"wrong DA commitment at {position}")
            records = decode_frame(raw)
            need(
                first == prev[0] + 1
                and last == next_[0]
                and [record["height"] for record in records] == list(range(first, last + 1))
                and records[-1]["origin"] == next_[1],
                "frame bounds disagree with successful cursor and attestation",
            )
            batches.append(Batch(
                position, block["hash"].lower(), tx_hash, receipt.q(log["logIndex"]),
                key, gpos, prev[0], records, hashlib.sha256(raw).hexdigest(),
            ))
        else:
            need(name == "submitResume", "ResumeRecorded carrier")
            rid, seq, policy, target, origin, packed = args
            _, signers = checker.resume_signers(
                crypto.as_bytes(transaction["from"]), rid, seq, policy, target, origin, packed,
            )
            set_position = tuple(row["setPosition"])
            need(
                set_position[:2] == position[:2] and set_position[2] < position[2],
                "CursorSet must precede its exact ResumeRecorded in the same transaction",
            )
            resumes.append(Resume(
                position, set_position, tx_hash, block["hash"].lower(),
                receipt.q(tx_receipt["logs"][set_position[2]]["logIndex"]),
                receipt.q(log["logIndex"]), crypto.hx(rid), seq, target, origin,
                {"enabled_mask": policy[0], "order": policy[1:4]}, policy[4],
                row["act"] * 1_000_000, signers,
            ))
    return committee, result, headers, batches, resumes
