#!/usr/bin/env python3
"""Drive the production Rust command API against a private Anvil instance."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import sys
import time
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

from coincurve import PrivateKey
from coincurve._libsecp256k1 import ffi, lib

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "poc/verification/anvil-l1"))
import golden_vectors as spec
import independent_codec as codec

UINT64_MAX = 2**64 - 1
CHAIN = 31337
TIME = 1700000000
POLICY = [7, 0, 1, 2, 0]
ZERO = bytes(32)
SALT = bytes([17]) * 32
GENESIS = [0, 0, ZERO, 0, b""]
FORCE_ARGS = ["bytes32", "uint64", "uint64", spec.POLICY, "bytes"]
RESUME_ARGS = ["bytes32", "uint64", spec.POLICY, "uint64", "uint64", "bytes"]
BATCH_ARGS = ["bytes32", spec.BATCH]
FIXTURE_KEYS = {
    role: hashlib.sha256(("ANVIL-L1-COMPONENT-v1/" + role).encode()).digest()
    for role in ("caller", "other", "governance", "certifier", "da")
}


def save(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def address(key):
    return codec.hx(codec.public_address(PrivateKey(key).public_key))


def selector(signature):
    return codec.keccak256(signature.encode())[:4]


def encode(signature, types=(), args=()):
    return selector(signature) + codec.abi_encode(types, args)


def rlp(value):
    def prefix(data, base):
        if len(data) < 56:
            return bytes([base + len(data)]) + data
        size = len(data).to_bytes((len(data).bit_length() + 7) // 8, "big")
        return bytes([base + 55 + len(size)]) + size + data

    if isinstance(value, list):
        return prefix(b"".join(map(rlp, value)), 192)
    if isinstance(value, int):
        value = value.to_bytes((value.bit_length() + 7) // 8, "big")
    if len(value) == 1 and value[0] < 128:
        return value
    return prefix(value, 128)


def create_address(sender, nonce):
    return codec.hx(codec.keccak256(rlp([codec.as_bytes(sender), nonce]))[-20:])


def global_cursor(height, gpos, origin=0):
    return [height, origin, ZERO, 1, gpos]


def resume_cursor(height, seq, origin=0):
    return [height, origin, ZERO, 2, codec.word(seq)]


def artifact(path, contract):
    return codec.as_bytes(json.loads((path / (contract + ".sol") / (contract + ".json")).read_text())["bytecode"]["object"])


class Scenario:
    def __init__(self, args, port, out):
        self.args = args
        self.port = port
        self.out = out
        self.addresses = {role: address(key) for role, key in FIXTURE_KEYS.items()}
        self.caller = self.addresses["caller"]
        self.authority = create_address(self.caller, 0)
        self.inbox = create_address(self.authority, 1)
        self.harness = create_address(self.caller, 1)
        self.harness_inbox = create_address(self.harness, 1)
        self.config = {
            "chain_id": CHAIN,
            "config_ref": codec.hx(SALT),
            "authority": self.authority,
            "inbox": self.inbox,
            "submitter": self.caller,
            "genesis_height": 0,
            "genesis_origin": 0,
            "genesis_timestamp_seconds": TIME,
            "slot_seconds": 2,
            "announcement_seconds": 6,
            "finality_delay_us": 2_000_000,
            "readers": ["A", "B"],
        }
        self.bundle_config = {
            "chainId": CHAIN,
            "configRef": codec.hx(SALT),
            "authority": self.authority,
            "inbox": self.inbox,
            "submitter": self.caller,
            "signers": {role: self.addresses[role] for role in ("governance", "certifier", "da")},
            "genesisHeight": 0,
            "genesisOrigin": 0,
            "genesisTimestampSeconds": TIME,
            "slotSeconds": 2,
            "announcementSeconds": 6,
        }
        self.command = 0
        self.trace = []
        self.direct_reads = []
        self.next_nonce = {"caller": 0, "other": 0}
        self.blocks = []
        self.receipts = []
        self.history = []
        self.acceptance = {}
        self.expectations = {}

    def probe(self, method, params=(), require_result=True):
        # Scenario-only diagnostics. Mutating RPCs are exclusively in the Rust driver.
        allowed = {"eth_chainId", "eth_blockNumber", "eth_getTransactionReceipt", "eth_call"}
        assert method in allowed
        request = {"jsonrpc": "2.0", "id": len(self.direct_reads), "method": method, "params": params}
        http = Request(
            f"http://127.0.0.1:{self.port}", data=json.dumps(request).encode(),
            headers={"Content-Type": "application/json"},
        )
        with urlopen(http, timeout=3) as response:
            reply = json.load(response)
        self.direct_reads.append({"request": request, "response": reply})
        if not require_result:
            return reply
        assert "result" in reply, (method, reply)
        return reply["result"]

    def run_driver(self, **action):
        self.command += 1
        return self.apply({"id": self.command, **action})

    def apply(self, command):
        self.driver.stdin.write(json.dumps(command) + "\n")
        self.driver.stdin.flush()
        line = self.driver.stdout.readline()
        if not line:
            raise RuntimeError("Rust driver exited; inspect driver.stderr")
        output = json.loads(line)
        self.trace.append({"command": command, **output})
        return output["reply"]

    def ready(self, **action):
        response = self.run_driver(**action)
        assert response["status"] == "ready", (action.get("op"), response)
        return response["result"]

    def expect(self, outcome, **action):
        response = self.run_driver(**action)
        assert response["status"] == outcome, (action.get("op"), response)
        return response

    def signature(self, role, message, variant=False):
        if not variant:
            return codec.sign_digest(FIXTURE_KEYS[role], message)
        entropy = ffi.new("unsigned char [32]", bytes([17]) * 32)
        raw = PrivateKey(FIXTURE_KEYS[role]).sign_recoverable(
            message, hasher=None, custom_nonce=(lib.secp256k1_nonce_function_rfc6979, entropy)
        )
        return raw[:64] + bytes([raw[64] + 27])

    def digest(self, primary, types, args, domain=None, sign_domain=None):
        role = domain or ("authority" if primary in ("Force", "Resume") else "inbox")
        config = {**self.bundle_config, **(sign_domain or {})}
        return codec.typed_digest(spec.domain(config, role)[1], spec.struct(primary, types, args)[1])

    def force(self, rid, seq, branch, policy=POLICY, *, caller=None, signed=None, sig=None, signer="governance", variant=False, sign_domain=None):
        rid = codec.word(rid)
        args = [rid, seq, branch, policy]
        signed_args = signed or [caller or self.caller, rid, seq, branch, spec.struct("Policy", spec.POLICY, policy)[1]]
        digest = self.digest("Force", ["address", "bytes32", "uint64", "uint64", "bytes32"], signed_args, sign_domain=sign_domain)
        signature = sig if sig is not None else self.signature(signer, digest, variant)
        return encode(spec.FUNCTIONS["submitForce"], FORCE_ARGS, [*args, signature]), digest, signature

    def resume(self, rid, seq, height, origin=0, policy=POLICY, *, caller=None, sig=None, signer="governance", sign_domain=None):
        rid = codec.word(rid)
        digest = self.digest(
            "Resume", ["address", "bytes32", "uint64", "bytes32", "uint64", "uint64"],
            [caller or self.caller, rid, seq, spec.struct("Policy", spec.POLICY, policy)[1], height, origin],
            sign_domain=sign_domain,
        )
        signature = sig if sig is not None else self.signature(signer, digest)
        return encode(spec.FUNCTIONS["submitResume"], RESUME_ARGS, [rid, seq, policy, height, origin, signature]), digest, signature

    def batch_args(self, prev, next, *, first=None, last=None, gpos=None, payload=b"anvil-l1-v1-payload",
                   da=None, da_signer="da", certifier="certifier", variant=False, sign_domain=None):
        commitment = codec.keccak256(payload)
        da_digest = self.digest("DAReceipt", ["bytes32"], [commitment])
        da = da if da is not None else self.signature(da_signer, da_digest)
        body = [prev, next, commitment, da, prev[0] + 1 if first is None else first,
                next[0] if last is None else last, next[4] if gpos is None else gpos]
        digest = self.digest(
            "BatchAttestation", ["bytes32"] * 4 + ["uint64", "uint64", "bytes32"],
            [spec.cursor_hash(prev)[1], spec.cursor_hash(next)[1], commitment, codec.keccak256(da), *body[4:6], codec.keccak256(body[6])],
            sign_domain=sign_domain,
        )
        return [*body, self.signature(certifier, digest, variant)], digest, da_digest

    def batch(self, rid, body):
        return encode(spec.FUNCTIONS["submitBatch"], BATCH_ARGS, [codec.word(rid), body])

    def tx(self, label, to, data, *, sender="caller", gas=1_000_000, ok=True, logs=None):
        nonce = self.next_nonce[sender]
        self.next_nonce[sender] += 1
        fields = [nonce, 1_000_000_000, gas, codec.as_bytes(to) if to else b"", 0, data]
        message = codec.keccak256(rlp([*fields, CHAIN, 0, 0]))
        sig = self.signature(sender, message)
        raw = rlp([*fields, 2 * CHAIN + 35 + sig[64] - 27, int.from_bytes(sig[:32], "big"), int.from_bytes(sig[32:64], "big")])
        actual = self.ready(op="submit", raw=codec.hx(raw))["submitted_tx_hash"]
        assert actual == codec.hx(codec.keccak256(raw))
        self.expectations[actual] = {"case": label, "status": int(ok), "logs": logs}
        return actual

    def mine(self, slot):
        self.ready(op="tick", reference_us=slot * 2_000_000)
        if slot == self.args.drop_mine_answer:
            first = self.expect("wait", op="mine", slot=slot)
            lost = self.trace[-1]
            assert first["included_height"] == slot - 1
            assert lost["rpc"][-1]["request"]["method"] == "evm_mine"
            assert "Ok" in lost["rpc"][-1]["result"]
            self.expect("wait", op="mine", slot=slot + 1)
            reply = self.apply(lost["command"])
            assert reply["status"] == "ready", reply
            assert all(row["request"]["method"] not in ("evm_mine", "evm_setNextBlockTimestamp") for row in self.trace[-1]["rpc"])
        else:
            reply = self.run_driver(op="mine", slot=slot)
            assert reply["status"] == "ready", reply
        block = reply["result"]["included_block"]
        assert block["number"] == slot and block["timestamp"] == TIME + slot * 2
        self.blocks.append(block["header_and_transactions"])
        self.receipts.extend(block["receipts"])
        for receipt in block["receipts"]:
            expected = self.expectations[receipt["transactionHash"]]
            assert int(receipt["status"], 16) == expected["status"], (expected, receipt)
            if expected["logs"] is not None:
                assert len(receipt["logs"]) == expected["logs"], expected
            self.acceptance[expected["case"]] = {"hash": receipt["transactionHash"], "status": int(receipt["status"], 16), "logs": len(receipt["logs"])}
        return block

    def read_cursor(self, reader, height, position, cursor_height):
        result = self.ready(op="read", reader=reader, query={"kind": "cursor", "height": height})
        assert result["cursor"]["height"] == cursor_height
        written_at = [result["written_at"][key] for key in ("block", "tx", "log")]
        assert written_at == position, (result, position)
        self.history.append({"height": height, "writtenAt": written_at, "returnData": result["read_evidence"]["return_data"]})
        return result

    def evm_call(self, to, data, height, *, caller=None):
        return self.probe("eth_call", [{"to": to, "from": caller or self.caller, "data": codec.hx(data)}, hex(height)])

    def assert_hash(self, emitter, name, types, values, expected):
        encoded = encode(spec.FUNCTIONS[name], types, values)
        assert self.evm_call(emitter, encoded, 1) == codec.hx(expected), name

    def deploy(self):
        self.expect("reject", op="mine", slot=1)
        self.ready(op="tick", reference_us=1_999_999)
        self.expect("reject", op="mine", slot=1)
        self.expect("reject", op="tick", reference_us=1_999_998)
        auth_args = ["bytes32", "address", "address", "address", "address", "uint64", "uint64", "uint64"]
        addresses = self.addresses
        auth_values = [SALT, addresses["governance"], self.caller, addresses["certifier"], addresses["da"], 6, 0, 0]
        auth_tx = self.tx("deploy-authority-and-inbox", None, artifact(self.args.contracts, "ControlAuthority")
                          + codec.abi_encode(auth_args, auth_values), gas=8_000_000, logs=1)
        fixture_values = [SALT, self.caller, addresses["certifier"], addresses["da"]]
        self.tx("deploy-rollback-fixture", None, artifact(self.args.contracts, "RollbackFixture")
                + codec.abi_encode(["bytes32", "address", "address", "address"], fixture_values), gas=8_000_000, logs=1)
        submit = self.trace[-2]
        replay = self.apply(submit["command"])
        assert replay == submit["reply"] and not self.trace[-1]["rpc"]
        assert self.apply({**submit["command"], "raw": "0x01"})["status"] == "reject"
        assert self.ready(op="submit", raw=submit["command"]["raw"]) == replay["result"]
        assert not self.trace[-1]["rpc"]
        self.ready(op="tick", reference_us=2_000_000)
        time.sleep(self.args.wall_pause_ms / 1000)
        assert self.probe("eth_blockNumber") == "0x0"
        assert self.probe("eth_getTransactionReceipt", [auth_tx]) is None
        self.mine(1)
        assert self.receipts[0]["contractAddress"] == self.authority
        assert codec.as_bytes(self.evm_call(self.authority, encode("inbox()"), 1))[-20:] == codec.as_bytes(self.inbox)
        assert self.evm_call(self.authority, encode("configRef()"), 1) == codec.hx(SALT)
        self.expect("reject", op="finalize", through_height=1)
        self.expect("wait", op="deliver", reader="A", through_height=1)
        self.expect("wait", op="read", reader="A", query={"kind": "cursor", "height": 1})

    def accepted_batches(self):
        first = global_cursor(2, b"\x01\x02\x03\x04")
        second = global_cursor(3, b"\x05")
        force, force_digest, _ = self.force(1, 0, 7)
        resume, resume_digest, _ = self.resume(3, 4, 8, 4)
        assert resume[:4] == selector(spec.FUNCTIONS["submitResume"])
        first_batch, batch_digest, da_digest = self.batch_args(GENESIS, first)
        second_batch, _, _ = self.batch_args(first, second, payload=b"next-frame")
        self.assert_hash(self.authority, "hashPolicy", [spec.POLICY], [POLICY], spec.struct("Policy", spec.POLICY, POLICY)[1])
        self.assert_hash(self.inbox, "hashCursor", [spec.CURSOR], [first], spec.cursor_hash(first)[1])
        self.assert_hash(self.authority, "forceDigest", ["address", *FORCE_ARGS[:-1]],
                         [self.caller, codec.word(1), 0, 7, POLICY], force_digest)
        self.assert_hash(self.authority, "resumeDigest", ["address", *RESUME_ARGS[:-1]],
                         [self.caller, codec.word(3), 4, POLICY, 8, 4], resume_digest)
        self.assert_hash(self.inbox, "batchDigest", [spec.BATCH], [first_batch], batch_digest)
        self.assert_hash(self.inbox, "daDigest", ["bytes32"], [first_batch[2]], da_digest)
        self.tx("force-sequence-zero", self.authority, force, logs=1)
        self.tx("batch-two-heights", self.inbox, self.batch(2, first_batch), logs=1)
        self.tx("batch-one-height", self.inbox, self.batch(4, second_batch), logs=1)
        block = self.mine(2)
        assert [event["event"] for event in block["events"]] == ["force", "cursor_advanced", "cursor_advanced"]
        assert block["events"][-1]["position"]["rpc_log_index"] == 2
        assert block["events"][-1]["position"]["log"] == 0
        self.ready(op="finalize", through_height=1)
        self.expect("wait", op="read", reader="A", query={"kind": "cursor", "height": 1})
        self.ready(op="deliver", reader="A", through_height=1)
        self.read_cursor("A", 1, [1, 0, 0], 0)
        self.expect("wait", op="read", reader="B", query={"kind": "cursor", "height": 1})
        return first_batch, second

    def contract_rejections(self, prev):
        def reject_force(label, data, sender="caller"):
            self.tx(label, self.authority, data, sender=sender, ok=False, logs=0)

        _, force_digest, sig = self.force(100, 2, 9)
        for label, data in [
            ("request-id", self.force(101, 2, 9, sig=sig)[0]),
            ("sequence", self.force(100, 3, 9, sig=sig)[0]),
            ("target-branch", self.force(100, 2, 10, sig=sig)[0]),
        ]:
            reject_force("force-mutation-" + label, data)
        reject_force("force-mutation-caller", self.force(100, 2, 9, sig=sig)[0], sender="other")
        for field, value in [
            ("chainId", CHAIN + 1), ("authority", self.inbox), ("configRef", bytes([44]) * 32)
        ]:
            reject_force("force-domain-" + field, self.force(100, 2, 9, sign_domain={field: value})[0])
        wrong_type = self.resume(100, 2, 9)[2]
        reject_force("force-resume-type-confusion", self.force(100, 2, 9, sig=wrong_type)[0])
        personal = codec.keccak256(b"\x19Ethereum Signed Message:\n32" + force_digest)
        reject_force("force-personal-prefix", self.force(100, 2, 9, sig=self.signature("governance", personal))[0])
        for index, value in enumerate(POLICY):
            policy = list(POLICY)
            policy[index] = value + 1
            reject_force(f"force-policy-signature-field-{index}", self.force(100, 2, 9, policy, sig=sig)[0])
        for index, policy in enumerate(([0, 0, 1, 2, 0], [8, 0, 1, 2, 0], [7, 1, 0, 2, 0],
                                         [7, 0, 0, 2, 0], [7, 0, 1, 3, 0])):
            reject_force(f"force-policy-invalid-{index}", self.force(100, 2, 9, policy)[0])
        reject_force("force-wrong-signer", self.force(100, 2, 9, signer="certifier")[0])
        reject_force("force-old-sequence", self.force(100, 0, 9)[0])
        reject_force("force-conflicting-request", self.force(1, 2, 7)[0])
        reject_force("force-trailing-calldata", self.force(100, 2, 9)[0] + ZERO)

        def malformed(signature):
            r, s, v = signature[:32], signature[32:64], signature[64:]
            return {
                "length-64": signature[:64], "length-66": signature + b"\0",
                "v0": signature[:64] + b"\0", "v1": signature[:64] + b"\1", "v29": signature[:64] + b"\x1d",
                "r0": ZERO + s + v, "rOrder": codec.word(codec.SECP_N) + s + v,
                "s0": r + ZERO + v, "sAboveHalf": r + codec.word(codec.SECP_N // 2 + 1) + v,
                "high-s-alias": r + codec.word(codec.SECP_N - int.from_bytes(s, "big")) + bytes([55-v[0]]),
            }

        for label, bad in malformed(sig).items():
            reject_force("force-sig-" + label, self.force(100, 2, 9, sig=bad)[0])

        _, _, resig = self.resume(3, 4, 8, 3)
        for label, rid, seq, height, origin in [
            ("request-id", 6, 4, 8, 3), ("sequence", 3, 5, 8, 3),
            ("target-height", 3, 4, 9, 3), ("origin", 3, 4, 8, 0),
        ]:
            self.tx("resume-mutation-" + label, self.authority, self.resume(rid, seq, height, origin, sig=resig)[0], ok=False, logs=0)
        self.tx("resume-mutation-caller", self.authority, self.resume(3, 4, 8, 3, sig=resig)[0], sender="other", ok=False, logs=0)
        for field, value in [
            ("chainId", CHAIN + 1), ("authority", self.inbox), ("configRef", bytes([44]) * 32)
        ]:
            self.tx("resume-domain-" + field, self.authority,
                    self.resume(3, 4, 8, 3, sign_domain={field: value})[0], ok=False, logs=0)
        self.tx("resume-force-type-confusion", self.authority, self.resume(3, 4, 8, 3, sig=sig)[0], ok=False, logs=0)
        self.tx("resume-trailing-calldata", self.authority, self.resume(3, 4, 8, 3)[0] + ZERO, ok=False, logs=0)
        for index, value in enumerate(POLICY):
            policy = list(POLICY)
            policy[index] = value + 1
            self.tx(f"resume-policy-signature-field-{index}", self.authority, self.resume(3, 4, 8, 3, policy, sig=resig)[0], ok=False, logs=0)
        for label, bad in malformed(resig).items():
            self.tx("resume-sig-" + label, self.authority, self.resume(3, 4, 8, 3, sig=bad)[0], ok=False, logs=0)

        next = global_cursor(4, b"new")
        valid, _, _ = self.batch_args(prev, next)
        encoded = self.batch(10, valid)
        self.tx("batch-trailing-calldata", self.inbox, encoded + ZERO, ok=False, logs=0)
        # Both GPOS arguments hold identical bytes. Point the top-level GPOS
        # offset at the tail of next.source, leaving the signed content intact.
        alias = bytearray(encoded)
        tuple_start = 4 + 64
        cursor_start = int.from_bytes(alias[tuple_start + 32:tuple_start + 64], "big")
        gpos_offset = tuple_start + 6 * 32
        alias[gpos_offset:gpos_offset + 32] = codec.word(cursor_start + 5 * 32)
        self.tx("batch-aliased-calldata", self.inbox, bytes(alias), ok=False, logs=0)
        for field, value in [
            ("chainId", CHAIN + 1), ("inbox", self.authority), ("configRef", bytes([44]) * 32)
        ]:
            invalid, _, _ = self.batch_args(prev, next, sign_domain={field: value})
            self.tx("batch-domain-" + field, self.inbox, self.batch(10, invalid), ok=False, logs=0)
        self.tx("batch-unauthorized-caller", self.inbox, self.batch(10, valid), sender="other", ok=False, logs=0)
        for cursor_idx, cursor in enumerate((prev, next)):
            values = [cursor[0] + 1, cursor[1] + 1, bytes([1]) * 32, 0, b"changed"]
            for idx, value in enumerate(values):
                mutated = copy.deepcopy(valid)
                mutated[cursor_idx][idx] = value
                self.tx(f"batch-mutation-cursor-{cursor_idx}-{idx}", self.inbox, self.batch(10, mutated), ok=False, logs=0)
        for idx, value in ((2, bytes([2]) * 32), (3, valid[3][:-1]), (4, 3), (5, 5), (6, b"different")):
            mutated = copy.deepcopy(valid)
            mutated[idx] = value
            self.tx(f"batch-mutation-field-{idx}", self.inbox, self.batch(10, mutated), ok=False, logs=0)
        for idx, value in enumerate((prev[0]-1, 1, bytes([1]) * 32, 0, b"stale")):
            changed = copy.deepcopy(prev)
            changed[idx] = value
            if idx == 3:
                changed[4] = b""
            invalid, _, _ = self.batch_args(changed, next, first=4)
            self.tx(f"batch-signed-prev-{idx}", self.inbox, self.batch(10, invalid), ok=False, logs=0)
        for label, changes in [
            ("range-start", {"first": 3}), ("range-end", {"last": 5}), ("range-too-wide", {}),
            ("source-opaque-mismatch", {"gpos": b"other"}), ("wrong-da-signer", {"da_signer": "governance"}),
            ("wrong-certifier-signer", {"certifier": "governance"}),
        ]:
            candidate = global_cursor(7, b"new") if label == "range-too-wide" else next
            invalid, _, _ = self.batch_args(prev, candidate, **changes)
            self.tx("batch-" + label, self.inbox, self.batch(10, invalid), ok=False, logs=0)
        origin_future, _, _ = self.batch_args(prev, global_cursor(4, b"future", origin=4))
        self.tx("batch-future-origin", self.inbox, self.batch(10, origin_future), ok=False, logs=0)
        # A valid alternative DA signature on the same commitment changes the attestation.
        _, _, da_digest = self.batch_args(prev, next)
        invalid = [*valid]
        invalid[3] = self.signature("da", da_digest, variant=True)
        assert invalid[3] != valid[3]
        self.tx("batch-replaced-valid-da-certificate", self.inbox, self.batch(10, invalid), ok=False, logs=0)
        forged_commitment = [*valid]
        forged_commitment[3] = self.signature("da", self.digest("DAReceipt", ["bytes32"], [ZERO]))
        self.tx("da-wrong-commitment-signature", self.inbox, self.batch(10, forged_commitment), ok=False, logs=0)
        for field, value in [
            ("chainId", CHAIN + 1), ("inbox", self.authority), ("configRef", bytes([44]) * 32)
        ]:
            bad_da = self.signature("da", self.digest("DAReceipt", ["bytes32"], [valid[2]], sign_domain={field: value}))
            wrong_domain, _, _ = self.batch_args(prev, next, da=bad_da)
            self.tx("da-domain-" + field, self.inbox, self.batch(10, wrong_domain), ok=False, logs=0)
        for role, idx in (("certifier", 7), ("da", 3)):
            for label, bad in malformed(valid[idx]).items():
                invalid = [*valid]
                invalid[idx] = bad
                self.tx(f"{role}-sig-{label}", self.inbox, self.batch(10, invalid), ok=False, logs=0)

        # The real Authority writes its cache/seq, then Override rejects. Check persisted
        # state before the same rid and seq succeed in the next slot.
        for label, height, origin in (("target-equal", 3, 0), ("target-lower", 2, 0), ("origin-future", 8, 4)):
            self.tx("resume-" + label, self.authority, self.resume(3, 4, height, origin)[0], ok=False, logs=0)
        direct = encode(spec.FUNCTIONS["overrideCursor"], [spec.CURSOR], [resume_cursor(8, 4)])
        self.tx("resume-direct-override", self.inbox, direct, ok=False, logs=0)
        self.tx("override-authorized-trailing-calldata", self.harness,
                encode("overrideRaw(bytes)", ["bytes"], [direct + ZERO]), ok=False, logs=0)
        rollback = encode("resume(bytes32,uint64," + spec.CURSOR_SIG + ",bool)",
                          ["bytes32", "uint64", spec.CURSOR, "bool"], [codec.word(1), 1, resume_cursor(8, 1), True])
        self.tx("rollback-after-inner-write", self.harness, rollback, ok=False, logs=0)
        block = self.mine(3)
        assert not block["events"] and len(block["receipts"]) > 80
        assert self.evm_call(self.authority, encode("lastControlSeq()"), 3) == codec.hx(ZERO)
        assert self.evm_call(self.harness, encode("seq()"), 3) == codec.hx(ZERO)
        assert self.evm_call(self.harness, encode("seen(bytes32)", ["bytes32"], [codec.word(1)]), 3) == codec.hx(ZERO)
        assert self.evm_call(self.harness_inbox, encode("currentCursor()"), 3) == codec.hx(codec.abi_encode([spec.CURSOR], [GENESIS]))
        return valid

    def control_and_history(self, first_batch, previous, stale):
        policy = [7, 0, 1, 2, 90000]
        self.tx("force-sequence-skip", self.authority, self.force(20, 2, 9, policy)[0], logs=1)
        self.tx("resume-first-override", self.authority, self.resume(3, 4, 8, 4)[0], logs=2)
        self.tx("resume-second-override", self.authority, self.resume(21, 5, 11, 0)[0], logs=2)
        block = self.mine(4)
        assert [(e["event"], e["position"]["tx"], e["position"]["log"], e["position"]["rpc_log_index"]) for e in block["events"]] == [
            ("force", 0, 0, 0), ("cursor_set", 1, 0, 1), ("resume", 1, 1, 2),
            ("cursor_set", 2, 0, 3), ("resume", 2, 1, 4),
        ]
        assert [e["act"] for e in block["events"] if "act" in e] == [TIME + 14] * 3
        self.tx("stale-batch-pre-resume-attestation", self.inbox, self.batch(22, stale), ok=False, logs=0)
        old_force, _, signature = self.force(1, 0, 7)
        repeat_force, _, another = self.force(1, 0, 7, variant=True)
        assert signature != another
        self.tx("force-original-signature-retry", self.authority, old_force, logs=0)
        self.tx("force-alternate-signature-retry", self.authority, repeat_force, logs=0)
        self.tx("force-invalid-signature-retry", self.authority, self.force(1, 0, 7, sig=b"")[0], ok=False, logs=0)
        self.tx("resume-original-retry", self.authority, self.resume(3, 4, 8, 4)[0], logs=0)
        alternate, _, _ = self.batch_args(GENESIS, first_batch[1], variant=True)
        assert alternate[:7] == first_batch[:7] and alternate[7] != first_batch[7]
        self.tx("batch-original-signature-retry", self.inbox, self.batch(2, first_batch), logs=0)
        self.tx("batch-alternate-signature-retry", self.inbox, self.batch(2, alternate), logs=0)
        self.tx("batch-invalid-signature-retry", self.inbox, self.batch(2, [*first_batch[:7], b""]), ok=False, logs=0)
        self.tx("batch-conflicting-request", self.inbox, self.batch(2, stale), ok=False, logs=0)
        self.tx("batch-accepted-attestation-new-request", self.inbox, self.batch(23, first_batch), ok=False, logs=0)
        followup, _, _ = self.batch_args(resume_cursor(11, 5), global_cursor(12, b"after-resume"), payload=b"after")
        self.tx("batch-after-resume", self.inbox, self.batch(24, followup), logs=1)
        block = self.mine(5)
        assert [e["event"] for e in block["events"]] == ["cursor_advanced"]
        assert codec.as_bytes(self.evm_call(self.authority, repeat_force, 5)) == codec.abi_encode(["uint64"] * 3, [0, 2, TIME + 10])
        assert codec.as_bytes(self.evm_call(self.authority, self.resume(3, 4, 8, 4)[0], 5)) == codec.abi_encode(["uint64"] * 3, [4, 4, TIME + 14])
        assert codec.as_bytes(self.evm_call(self.inbox, self.batch(2, alternate), 5)) == codec.abi_encode(["bytes32", "uint64"], [spec.cursor_hash(first_batch[1])[1], 2])
        self.tx("force-max-sequence", self.authority, self.force(25, UINT64_MAX, 2)[0], logs=1)
        self.tx("force-max-repeat-new-request", self.authority, self.force(26, UINT64_MAX, 2)[0], ok=False, logs=0)
        self.tx("force-descending", self.authority, self.force(26, 4, 2)[0], ok=False, logs=0)
        success = encode("resume(bytes32,uint64," + spec.CURSOR_SIG + ",bool)",
                         ["bytes32", "uint64", spec.CURSOR, "bool"], [codec.word(1), 1, resume_cursor(8, 1), False])
        self.tx("rollback-fixture-same-request-success", self.harness, success, logs=2)
        self.mine(6)
        self.mine(7)
        self.ready(op="tick", reference_us=16_000_000)
        self.ready(op="finalize", through_height=7)
        self.expect("reject", op="finalize", through_height=6)
        self.expect("reject", op="mine", slot=7)
        self.expect("wait", op="deliver", reader="B", through_height=8)
        self.ready(op="deliver", reader="A", through_height=7)
        self.ready(op="deliver", reader="B", through_height=2)
        self.ready(op="deliver", reader="B", from_height=4, through_height=7)
        self.expect("wait", op="read", reader="B", query={"kind": "cursor", "height": 4})
        self.ready(op="deliver", reader="B", from_height=3, through_height=3)
        self.expect("wait", op="read", reader="A", query={"kind": "cursor", "height": 0})
        self.read_cursor("A", 2, [2, 2, 0], previous[0])
        self.read_cursor("A", 3, [2, 2, 0], previous[0])
        self.read_cursor("A", 4, [4, 2, 0], 11)
        a = self.read_cursor("A", 7, [5, 10, 0], 12)
        b = self.read_cursor("B", 7, [5, 10, 0], 12)
        assert a == b
        for tx, target in ((1, 8), (2, 11)):
            result = self.ready(op="read", reader="A", query={"kind": "event", "height": 4, "tx": tx, "log": 1})
            record = result["record"]
            assert record["height"] == target and record["set"]["tx"] == tx and record["set"]["log"] == 0

    def run(self):
        with (self.out / "driver.stderr").open("w") as errors:
            self.driver = subprocess.Popen(
                [str(self.args.driver), f"127.0.0.1:{self.port}", str(self.out / "driver-config.json"), str(self.args.drop_mine_answer)],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors, text=True,
            )
            try:
                startup = self.driver.stdout.readline()
                if not startup:
                    raise RuntimeError("Rust driver failed at startup; inspect driver.stderr")
                startup = json.loads(startup)["startup_rpc"]
                self.trace.append({"startup_rpc": startup})
                self.blocks.append(next(
                    row["result"]["Ok"] for row in startup if row["request"]["method"] == "eth_getBlockByNumber"
                ))
                self.deploy()
                first_batch, previous = self.accepted_batches()
                stale = self.contract_rejections(previous)
                self.control_and_history(first_batch, previous, stale)
                assert len(self.acceptance) == len(self.expectations)
            finally:
                self.driver.stdin.close()
                self.driver.wait(timeout=5)
                self.driver.stdout.close()

    def write_evidence(self, outcome):
        save(self.out / "driver-trace.json", self.trace)
        save(self.out / "diagnostic-rpc.json", self.direct_reads)
        save(self.out / "raw-rpc-bundle.json", {
            "config": self.bundle_config, "blocks": self.blocks, "receipts": self.receipts,
            "historicalCursorReads": self.history,
        })
        virtual = [{k: row[k] for k in ("command", "reply")} for row in self.trace if "command" in row]
        save(self.out / "virtual-trace.json", virtual)
        save(self.out / "result.json", {
            "status": outcome,
            "seed": "none (explicit script)",
            "wallPauseMilliseconds": self.args.wall_pause_ms,
            "droppedCompletedMineAnswerNumber": self.args.drop_mine_answer,
            "blocks": len(self.blocks),
            "receipts": len(self.receipts),
            "failedReceipts": sum(r["status"] == "0x0" for r in self.receipts),
            "historicalCursorReads": len(self.history),
            "virtualTraceSha256": hashlib.sha256((self.out / "virtual-trace.json").read_bytes()).hexdigest(),
            "cases": self.acceptance,
        })


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--anvil", type=Path, required=True)
    parser.add_argument("--driver", type=Path, required=True)
    parser.add_argument("--contracts", type=Path, required=True, help="Forge artifacts directory")
    parser.add_argument("--out", type=Path, required=True, help="new artifact directory")
    parser.add_argument("--wall-pause-ms", type=int, default=300)
    parser.add_argument("--drop-mine-answer", type=int, default=4, choices=range(0, 8))
    args = parser.parse_args()
    if args.wall_pause_ms < 250:
        parser.error("A14 requires a wall pause of at least 250 ms")
    args.out.mkdir(parents=True)
    with socket.socket() as socket_reservation:
        socket_reservation.bind(("127.0.0.1", 0))
        port = socket_reservation.getsockname()[1]
    scenario = Scenario(args, port, args.out)
    save(args.out / "driver-config.json", scenario.config)
    genesis = args.out / "anvil-genesis.json"
    save(genesis, {
        "config": {
            "chainId": CHAIN, "homesteadBlock": 0, "eip150Block": 0, "eip155Block": 0,
            "eip158Block": 0, "byzantiumBlock": 0, "constantinopleBlock": 0,
            "petersburgBlock": 0, "istanbulBlock": 0, "berlinBlock": 0, "londonBlock": 0,
            "shanghaiTime": 0,
        },
        "timestamp": hex(TIME),
        "gasLimit": hex(100_000_000),
        "difficulty": "0x0",
        "alloc": {scenario.addresses[role]: {"balance": hex(10**22)} for role in ("caller", "other")},
    })
    anvil_cmd = [
        str(args.anvil), "--host", "127.0.0.1", "--port", str(port), "--quiet", "--no-mining",
        "--order", "fifo", "--no-cors", "--accounts", "0", "--threads", "2", "--hardfork", "shanghai",
        "--timestamp", str(TIME), "--chain-id", str(CHAIN), "--gas-limit", "100000000",
        "--disable-default-create2-deployer", "--init", str(genesis),
    ]
    save(args.out / "anvil-command.json", anvil_cmd)
    outcome = "FAIL"
    with (args.out / "anvil.log").open("w") as logs:
        anvil = subprocess.Popen(anvil_cmd, stdout=logs, stderr=subprocess.STDOUT)
        try:
            for _ in range(100):
                if anvil.poll() is not None:
                    raise RuntimeError("Anvil exited during startup; inspect anvil.log")
                try:
                    assert scenario.probe("eth_chainId") == hex(CHAIN)
                    break
                except (URLError, ConnectionError):
                    time.sleep(0.025)
            else:
                raise RuntimeError("Anvil did not become ready")
            scenario.run()
            outcome = "PASS"
        finally:
            scenario.write_evidence(outcome)
            if anvil.poll() is None:
                anvil.terminate()
                try:
                    anvil.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    anvil.kill()
                    anvil.wait()
    print(json.dumps({
        "status": outcome, "blocks": len(scenario.blocks), "receipts": len(scenario.receipts), "output": str(args.out)
    }))


if __name__ == "__main__":
    main()
