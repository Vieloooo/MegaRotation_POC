"""Independent committee and Force receipt verification for the M3 Anvil fixture."""

import json
from dataclasses import dataclass

import golden_vectors as golden
import independent_codec as crypto
import receipt_checker as receipt


@dataclass(frozen=True)
class Committee:
    members: tuple
    threshold: int


def decode_deployment(bundle, artifact):
    """Verify the actual deployment receipt and decode the frozen constructor ABI."""
    cfg = bundle["config"]
    authority = cfg["authority"].lower()
    deployments = [
        result for result in bundle["receipts"]
        if result.get("contractAddress") and result["contractAddress"].lower() == authority
    ]
    receipt.need(len(deployments) == 1, "expected one actual committee birth receipt")
    [deployment] = deployments
    receipt.need(receipt.q(deployment["status"]) == 1, "committee deployment failed")
    index = receipt.q(deployment["transactionIndex"])
    block = bundle["blocks"][receipt.q(deployment["blockNumber"])]
    tx = block["transactions"][index]
    receipt.need(tx["hash"].lower() == deployment["transactionHash"].lower() and not tx["to"],
                 "committee address does not correspond to a creation transaction")

    object_code = json.loads(artifact.read_bytes())["bytecode"]["object"]
    prefix = crypto.as_bytes(object_code)
    supplied = crypto.as_bytes(tx["input"])
    receipt.need(prefix and supplied.startswith(prefix),
                 "deployment init code differs from sealed committee artifact")
    data = supplied[len(prefix):]
    head_types = [
        "bytes32", "uint256", "uint8", "address", "address", "address",
        "uint64", "uint64", "uint64",
    ]
    head_size = 32 * len(head_types)
    salt, offset, threshold, submitter, certifier, da, announcement, h, origin = receipt.abi_decode(
        head_types, data[:head_size]
    )
    receipt.need(offset == head_size and len(data) >= offset + 32,
                 "noncanonical committee constructor array offset")
    count = receipt.w(data, offset)
    receipt.need(1 <= threshold <= count <= 16 and len(data) == offset + 32 * (1 + count),
                 "invalid committee constructor member count or threshold")
    members = tuple(receipt.abi_decode(["address"] * count, data[offset + 32:]))
    receipt.need(tuple(sorted(set(members))) == members and all(any(m) for m in members),
                 "invalid configured committee identity or order")
    receipt.need(certifier not in members and da not in members,
                 "governance keys overlap certifier or DA")
    receipt.need(salt == crypto.as_bytes(cfg["configRef"])
                 and submitter == crypto.as_bytes(cfg["submitter"])
                 and certifier == crypto.as_bytes(cfg["signers"]["certifier"])
                 and da == crypto.as_bytes(cfg["signers"]["da"])
                 and (announcement, h, origin) == (
                     cfg["announcementSeconds"], cfg["genesisHeight"], cfg["genesisOrigin"]),
                 "bundle config disagrees with constructor")
    receipt.need(tuple(crypto.as_bytes(m) for m in cfg["committee"]) == members
                 and cfg["threshold"] == threshold, "bundle committee differs from actual deployment")
    committee = Committee(members, threshold)
    return committee


class CommitteeReceiptChecker(receipt.Checker):
    def __init__(self, bundle, committee):
        super().__init__(bundle)
        self.committee = committee

    def control_args(self, name, args, sender):
        receipt.need(name == "submitForce", "unexpected RESUME in M3 Force raw evidence")
        rid, seq, target, policy, packed = args
        receipt.need(policy[0] < 8 and sorted(policy[1:4]) == [0, 1, 2],
                     "invalid regional policy in successful control")
        selection = self.force_signers(sender, rid, seq, target, policy, packed)
        return rid, seq, selection[0]

    def force_signers(self, sender, rid, seq, target, policy, packed):
        policy_hash = golden.struct("Policy", golden.POLICY, policy)[1]
        force_hash = golden.struct(
            "Force", ["address", "bytes32", "uint64", "uint64", "bytes32"],
            [sender, rid, seq, target, policy_hash],
        )[1]
        digest = crypto.typed_digest(self.domains["authority"], force_hash)
        receipt.need(len(packed) == 65 * self.committee.threshold,
                     "successful Force did not carry exactly the configured threshold")
        signers = tuple(
            crypto.recover_address(packed[start:start + 65], digest)
            for start in range(0, len(packed), 65)
        )
        receipt.need(tuple(sorted(set(signers))) == signers
                     and all(signer in self.committee.members for signer in signers),
                     "successful Force has repeated, misordered or unregistered signers")
        return force_hash, digest, signers


def verified_force(bundle, artifact):
    committee = decode_deployment(bundle, artifact)
    checker = CommitteeReceiptChecker(bundle, committee)
    verified = checker.run()
    controls = []
    for event in verified["events"]:
        if event["kind"] == "CursorInitialized":
            continue
        receipt.need(event["kind"] == "ForceRecorded", "unexpected Authority log")
        position = event["position"]
        inclusion = bundle["blocks"][position[0]]
        tx = inclusion["transactions"][position[1]]
        name, args = receipt.decode_call(tx)
        receipt.need(name == "submitForce", "Force event not bound to submitForce")
        rid, seq, target, policy, packed = args
        _, digest, signers = checker.force_signers(
            crypto.as_bytes(tx["from"]), rid, seq, target, policy, packed
        )
        controls.append({
            "position": position,
            "target": target,
            "seq": seq,
            "act_us": event["act"] * 1_000_000,
            "l1_floor": policy[4],
            "policy": {"enabled_mask": policy[0], "order": policy[1:4]},
            "request_id": crypto.hx(rid),
            "request_digest": crypto.hx(digest),
            "signers": [crypto.hx(signer) for signer in signers],
            "signatures": [
                crypto.hx(packed[start:start + 65]) for start in range(0, len(packed), 65)
            ],
            "tx_hash": tx["hash"].lower(),
            "nonce": receipt.q(tx["nonce"]),
        })
    return committee, verified, controls


def historical_headers(bundle):
    """This projection uses the verified actual RPC prefix as its source."""
    return [
        {"height": receipt.q(block["number"]), "timestamp_s": receipt.q(block["timestamp"]),
         "hash": list(crypto.as_bytes(block["hash"]))}
        for block in bundle["blocks"]
    ]
