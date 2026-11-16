#!/usr/bin/env python3
"""Exercise AF07 and the on-chain part of AF08 with the real Anvil L1 driver."""

import argparse
from dataclasses import dataclass, replace
import hashlib
import importlib.metadata
import json
from pathlib import Path
import re
import socket
import subprocess
import sys
import time
from urllib.error import URLError

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "poc/crates/l1-adapter/tests"))
from run_anvil import (
    CHAIN, FORCE_ARGS, TIME, Scenario, address, artifact, codec, encode, save, spec,
)
from receipt_checker import abi_decode, decode_call, decode_event

ANNOUNCEMENT = 10
RETURN_TYPES = ["uint64"] * 3
POLICY = (6, 0, 2, 1, 19)
NEXT_POLICY = (0, 0, 1, 2, 11)
AUTHORITY = "CommitteeControlAuthority"
LOCK = "/tmp/megaroation-poc-heavy.lock"
FILE = Path(__file__).resolve()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def tool(path):
    with path.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    return {
        "path": str(path),
        "sha256": digest,
        "version": subprocess.check_output([path, "--version"], text=True, timeout=5).strip(),
    }


def git(*args):
    return subprocess.check_output(["git", "-C", ROOT, *args], text=True).strip()


def revision(candidate):
    sha = git("rev-parse", "HEAD")
    require(sha == candidate, "checkout HEAD differs from candidate")
    status = git("status", "--porcelain", "--untracked-files=all")
    require(not status, f"checkout is dirty: {status}")
    return {"candidate": sha, "status": status, "worktree": str(ROOT),
            "branch": git("branch", "--show-current")}


def fixture_key(name):
    return hashlib.sha256(("PoC/M3/l1-vector/v1/" + name).encode()).digest()


MEMBER_KEYS = sorted((fixture_key(f"member:{i}") for i in range(3)), key=address)
MEMBERS = [address(key) for key in MEMBER_KEYS]
OUTSIDER = fixture_key("outsider")


@dataclass(frozen=True)
class Force:
    rid: int
    seq: int
    branch: int
    policy: tuple = POLICY

    @property
    def request(self):
        return codec.word(self.rid)

    def call(self, packet):
        return encode(spec.FUNCTIONS["submitForce"], FORCE_ARGS,
                      [self.request, self.seq, self.branch, self.policy, packet])

    def digest(self, cfg, caller, *, resume=False):
        policy_hash = spec.struct("Policy", spec.POLICY, self.policy)[1]
        if resume:
            body = spec.struct("Resume", ["address", "bytes32", "uint64", "bytes32", "uint64", "uint64"],
                               [caller, self.request, self.seq, policy_hash, 43, 0])[1]
        else:
            body = spec.struct("Force", ["address", "bytes32", "uint64", "uint64", "bytes32"],
                               [caller, self.request, self.seq, self.branch, policy_hash])[1]
        return codec.typed_digest(spec.domain(cfg, "authority")[1], body)


class Vectors(Scenario):
    def __init__(self, args, port, out, threshold):
        super().__init__(args, port, out)
        self.threshold = threshold
        self.salt = hashlib.sha256(
            b"PoC/M3/l1-vector-config/v1" + bytes([threshold]) +
            b"".join(codec.as_bytes(member) for member in MEMBERS)
        ).digest()
        self.config.update(config_ref=codec.hx(self.salt), announcement_seconds=ANNOUNCEMENT,
                           regional_policy=True)
        self.bundle_config.update(
            configRef=codec.hx(self.salt), announcementSeconds=ANNOUNCEMENT,
            signers={key: self.addresses[key] for key in ("certifier", "da")},
            governanceMembers=MEMBERS, governanceThreshold=threshold,
        )
        self.controls = []
        self.force_calls = {}
        self.queries = []

    def pack(self, force, keys, *, sender="caller", resume=False, **domain):
        digest = force.digest({**self.bundle_config, **domain}, self.addresses[sender],
                              resume=resume)
        return b"".join(codec.sign_digest(key, digest) for key in sorted(keys, key=address))

    def submit(self, label, force, packet, *, status, emitted, sender="caller"):
        data = force.call(packet)
        tx = self.tx(label, self.authority, data, sender=sender, ok=status, logs=emitted)
        self.force_calls[tx] = (force, packet, sender, status, emitted)
        return data

    def query(self, method, height, types=(), args=()):
        return codec.as_bytes(self.evm_call(self.authority, encode(method, types, args), height))

    def state(self, height, enabled, sequence):
        raw_flag = self.query("hasControl()", height)
        raw_seq = self.query("lastControlSeq()", height)
        require(raw_flag == codec.word(int(enabled)) and raw_seq == codec.word(sequence),
                f"unexpected Authority sequence at height {height}")
        self.queries.append({"kind": "sequence", "height": height,
                             "hasControl": enabled, "lastControlSeq": sequence})

    def expect_return(self, data, height, expected):
        actual = abi_decode(RETURN_TYPES, self.evm_call(self.authority, data, height))
        require(actual == expected, f"return value at {height}: {actual} != {expected}")
        self.queries.append({"kind": "force_return", "height": height, "result": actual,
                             "call": codec.hx(data)})

    def deploy(self):
        # The only dynamic constructor field is address[] at the second word.
        # The static head has nine slots and is followed by the array.
        types = ["bytes32", "uint256", "uint8", "address", "address", "address",
                 "uint64", "uint64", "uint64"]
        values = [self.salt, 32 * len(types), self.threshold, self.caller,
                  self.addresses["certifier"], self.addresses["da"], ANNOUNCEMENT, 0, 0]
        args = codec.abi_encode(types, values) + codec.abi_encode(
            ["uint256"] + ["address"] * len(MEMBERS), [len(MEMBERS), *MEMBERS])
        tx = self.tx("deploy-authority", None, artifact(self.args.contracts, AUTHORITY) + args,
                     gas=8_000_000, logs=1)
        self.mine(1)
        require(self.receipts[-1]["transactionHash"] == tx and
                self.receipts[-1]["contractAddress"] == self.authority, "wrong Authority deployment address")
        require(self.query("threshold()", 1) == codec.word(self.threshold) and
                self.query("memberCount()", 1) == codec.word(len(MEMBERS)),
                "deployed threshold or member count disagrees")
        for member in MEMBERS:
            require(self.query("member(address)", 1, ["address"], [member]) == codec.word(1),
                    f"configured signer missing: {member}")
        require(self.query("member(address)", 1, ["address"], [address(OUTSIDER)]) == codec.word(0),
                "unconfigured signer was admitted by the constructor")
        require(self.query("configRef()", 1) == self.salt, "wrong EIP-712 configuration reference")
        self.state(1, False, 0)

    def reject_two_of_three(self, force):
        first, second, third = MEMBER_KEYS
        digest = force.digest(self.bundle_config, self.caller)
        one = codec.sign_digest(first, digest)
        two = codec.sign_digest(second, digest)
        pair = one + two
        high = codec.SECP_N - int.from_bytes(one[32:64], "big")
        malleated = one[:32] + high.to_bytes(32, "big") + bytes([55 - one[64]])
        for label, packet in (
            ("empty", b""),
            ("one-of-three", one),
            ("three-signatures", self.pack(force, [first, second, third])),
            ("truncated", pair[:-1]),
            ("duplicate-member", one + one),
            ("descending-members", two + one),
            ("outsider", self.pack(force, [OUTSIDER, first])),
            ("high-s", malleated + two),
            ("invalid-v", one[:-1] + b"\x1d" + two),
            ("wrong-chain", self.pack(force, [first, second], chainId=CHAIN + 1)),
            ("wrong-contract", self.pack(force, [first, second], authority=self.inbox)),
            ("wrong-config", self.pack(force, [first, second], configRef=bytes([81]) * 32)),
            ("wrong-control-type", self.pack(force, [first, second], resume=True)),
        ):
            self.submit(label, force, packet, status=False, emitted=0)
        self.submit("wrong-caller", force, pair, sender="other", status=False, emitted=0)
        for label, changed in (
            ("changed-policy", replace(force, policy=(4, *POLICY[1:]))),
            ("changed-rid", replace(force, rid=force.rid + 1)),
            ("changed-sequence", replace(force, seq=force.seq + 1)),
            ("changed-branch", replace(force, branch=force.branch + 1)),
        ):
            self.submit(label, changed, pair, status=False, emitted=0)
        for label, policy in (
            ("mask-upper-bit", (8, *POLICY[1:])),
            ("repeated-cycle-region", (6, 0, 0, 2, 19)),
        ):
            malformed = replace(force, policy=policy)
            self.submit(label, malformed, self.pack(malformed, [first, second]),
                        status=False, emitted=0)

    def cases(self):
        first, second, third = MEMBER_KEYS
        force = Force(1, 7, 0)
        keys = MEMBER_KEYS[:self.threshold]
        pair = self.pack(force, keys)
        if self.threshold == 2:
            self.reject_two_of_three(force)
        else:
            self.submit("two-signatures-when-one-required", force, self.pack(force, [first, second]),
                        status=False, emitted=0)
        self.mine(2)
        self.state(2, False, 0)

        direct = self.query(
            spec.FUNCTIONS["forceDigest"], 2, ["address", *FORCE_ARGS[:-1]],
            [self.caller, force.request, force.seq, force.branch, force.policy])
        require(direct == force.digest(self.bundle_config, self.caller), "Force digest mismatch")
        fresh = self.submit("exact-threshold", force, pair, status=True, emitted=1)
        self.mine(3)
        self.state(3, True, force.seq)
        original = [force.seq, 3, TIME + 6 + ANNOUNCEMENT]
        self.expect_return(fresh, 3, original)

        self.submit("retry-exact-packet", force, pair, status=True, emitted=0)
        subset = [second, third] if self.threshold == 2 else [third]
        different_packet = self.pack(force, subset)
        require(pair != different_packet, "alternate quorum fixture reused the first packet")
        alternate = self.submit("retry-alternate-quorum", force, different_packet, status=True, emitted=0)
        self.submit("retry-invalid-length", force, pair[:-65], status=False, emitted=0)
        conflict = replace(force, seq=force.seq + 1, policy=NEXT_POLICY)
        self.submit("conflicting-request", conflict, self.pack(conflict, keys), status=False, emitted=0)
        stale = replace(force, rid=force.rid + 1)
        self.submit("stale-sequence", stale, self.pack(stale, keys), status=False, emitted=0)
        self.mine(4)
        self.state(4, True, force.seq)
        self.expect_return(alternate, 4, original)

        new = replace(stale, seq=force.seq + 1, policy=NEXT_POLICY)
        second_call = self.submit("higher-sequence-zero-mask", new, self.pack(new, keys),
                                  status=True, emitted=1)
        self.mine(5)
        self.state(5, True, new.seq)
        self.expect_return(second_call, 5, [new.seq, 5, TIME + 10 + ANNOUNCEMENT])
        self.expect_return(alternate, 5, original)

    def verify_evidence(self):
        require(len(self.acceptance) == len(self.expectations), "a submitted case has no receipt")
        blocks = {int(row["number"], 16): row for row in self.blocks}
        receipts = {row["transactionHash"]: row for row in self.receipts}
        for tx, (force, packet, sender, ok, emitted) in self.force_calls.items():
            receipt = receipts[tx]
            height = int(receipt["blockNumber"], 16)
            index = int(receipt["transactionIndex"], 16)
            block = blocks[height]
            transaction = block["transactions"][index]
            require(transaction["hash"] == tx and receipt["blockHash"] == block["hash"],
                    f"wrong inclusion provenance for {tx}")
            require(transaction["from"] == self.addresses[sender] and
                    transaction["to"] == self.authority, f"wrong sender or Authority for {tx}")
            name, args = decode_call(transaction)
            require(name == "submitForce" and args == [
                force.request, force.seq, force.branch, list(force.policy), packet], f"wrong calldata for {tx}")
            require(int(receipt["status"], 16) == ok and len(receipt["logs"]) == emitted,
                    f"wrong receipt outcome for {tx}")
            if ok:
                digest = force.digest(self.bundle_config, self.addresses[sender])
                recovered = [codec.hx(codec.recover_address(packet[i:i + 65], digest))
                             for i in range(0, len(packet), 65)]
                require(len(recovered) == self.threshold and
                        recovered == sorted(set(recovered)) and set(recovered) <= set(MEMBERS),
                        f"receipt accepted an invalid quorum: {tx}")
                case = self.expectations[tx]["case"]
                self.acceptance[case].update(digest=codec.hx(digest), recovered=recovered)
            if emitted:
                (log,) = receipt["logs"]
                name, values = decode_event(log)
                act = int(block["timestamp"], 16) + ANNOUNCEMENT
                require(name == "ForceRecorded" and log["address"] == self.authority and values == [
                    codec.as_bytes(self.addresses[sender]), force.request,
                    force.seq, force.branch, list(force.policy), act,
                ], f"wrong Force event for {tx}")
                require(log["transactionHash"] == tx and int(log["logIndex"], 16) == 0 and
                        int(log["transactionIndex"], 16) == index and log["blockHash"] == block["hash"],
                        f"wrong Force log location: {tx}")
                self.controls.append({"txHash": tx, "position": [height, index, 0],
                                      "seq": force.seq, "branch": force.branch,
                                      "policy": list(force.policy), "act": act})
        require([row["position"] for row in self.controls] == [[3, 0, 0], [5, 0, 0]],
                f"unexpected Force positions: {self.controls}")

    def run(self):
        argv = [str(self.args.driver), f"127.0.0.1:{self.port}", str(self.out / "driver-config.json")]
        save(self.out / "driver-command.json", argv)
        with (self.out / "driver.stderr").open("w") as errors, subprocess.Popen(
            argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors, text=True,
        ) as self.driver:
            try:
                startup = self.driver.stdout.readline()
                require(startup, "Rust driver failed at startup; inspect driver.stderr")
                startup_rpc = json.loads(startup)["startup_rpc"]
                self.trace.append({"startup_rpc": startup_rpc})
                self.blocks.append(next(
                    row["result"]["Ok"] for row in startup_rpc
                    if row["request"]["method"] == "eth_getBlockByNumber"
                ))
                self.deploy()
                self.cases()
                self.verify_evidence()
                self.expect("wait", op="deliver", reader="A", through_height=5)
                self.ready(op="tick", reference_us=12_000_000)
                self.ready(op="finalize", through_height=5)
                for reader in ("A", "B"):
                    self.ready(op="deliver", reader=reader, through_height=5)
                    for control in self.controls:
                        h, tx, log = control["position"]
                        result = self.ready(op="read", reader=reader,
                                            query={"kind": "event", "height": h, "tx": tx, "log": log})
                        record = result["record"]
                        mask, *order, floor = control["policy"]
                        expected_position = {
                            "block": h, "tx": tx, "log": log, "rpc_log_index": log,
                            "tx_hash": control["txHash"],
                        }
                        expected_policy = {"enabled_mask": mask, "order": order, "l1_floor": floor}
                        require(record["event"] == "force" and record["seq"] == control["seq"] and
                                record["act"] == control["act"] and record["branch"] == control["branch"] and
                                record["position"] == expected_position and record["policy"] == expected_policy,
                                f"wrong finalized record for {reader}")
            finally:
                self.driver.stdin.close()
                try:
                    self.driver.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.driver.kill()
                    self.driver.wait(timeout=5)
                self.driver.stdout.close()
        require(self.driver.returncode == 0, "L1 driver exit failed")


def stop(anvil):
    if anvil.poll() is None:
        anvil.terminate()
        try:
            anvil.wait(timeout=5)
        except subprocess.TimeoutExpired:
            anvil.kill()
            anvil.wait(timeout=5)


def run_profile(args, out, threshold):
    out.mkdir()
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    scenario = Vectors(args, port, out, threshold)
    save(out / "driver-config.json", scenario.config)
    genesis = out / "anvil-genesis.json"
    forks = ("homestead", "eip150", "eip155", "eip158", "byzantium",
             "constantinople", "petersburg", "istanbul", "berlin", "london")
    save(genesis, {
        "config": {"chainId": CHAIN, **{f"{fork}Block": 0 for fork in forks}, "shanghaiTime": 0},
        "timestamp": hex(TIME), "gasLimit": hex(100_000_000), "difficulty": "0x0",
        "alloc": {scenario.addresses[role]: {"balance": hex(10**22)} for role in ("caller", "other")},
    })
    argv = [
        str(args.anvil), "--host", "127.0.0.1", "--port", str(port), "--quiet", "--no-mining",
        "--order", "fifo", "--no-cors", "--accounts", "0", "--threads", "2", "--hardfork", "shanghai",
        "--timestamp", str(TIME), "--chain-id", str(CHAIN), "--gas-limit", "100000000",
        "--disable-default-create2-deployer", "--init", str(genesis),
    ]
    save(out / "anvil-command.json", argv)
    outcome = "FAIL"
    with (out / "anvil.log").open("w") as logs, subprocess.Popen(
        argv, stdout=logs, stderr=subprocess.STDOUT,
    ) as anvil:
        try:
            for _ in range(100):
                require(anvil.poll() is None, "Anvil exited during startup")
                try:
                    require(scenario.probe("eth_chainId") == hex(CHAIN), "wrong chain ID")
                    break
                except (URLError, ConnectionError):
                    time.sleep(0.025)
            else:
                raise RuntimeError("Anvil did not become ready")
            scenario.run()
            outcome = "PASS"
        finally:
            stop(anvil)
            scenario.write_evidence(outcome)
            save(out / "checked-controls.json", scenario.controls)
            save(out / "checked-queries.json", scenario.queries)
    return {"status": outcome, "threshold": threshold, "receipts": len(scenario.receipts),
            "failed_receipts": sum(row["status"] == "0x0" for row in scenario.receipts),
            "controls": scenario.controls}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--anvil", required=True, type=lambda value: Path(value).resolve())
    parser.add_argument("--driver", required=True, type=lambda value: Path(value).resolve())
    parser.add_argument("--contracts", required=True, type=lambda value: Path(value).resolve())
    parser.add_argument("--out", required=True, type=lambda value: Path(value).resolve())
    args = parser.parse_args()
    require(__debug__, "run with Python assertions enabled")
    require(re.fullmatch("[0-9a-f]{40}", args.candidate), "--candidate requires a full SHA")
    args.drop_mine_answer, args.wall_pause_ms = 0, 0
    before = revision(args.candidate)
    args.out.mkdir(mode=0o700)
    result = {"status": "FAIL", "candidate": args.candidate, "argv": sys.orig_argv,
              "before": before, "heavy_lock": LOCK, "profiles": [], "seed": "explicit vector"}
    try:
        result["tools"] = {"anvil": tool(args.anvil), "python": sys.version,
                           "coincurve": importlib.metadata.version("coincurve")}
        binary = args.contracts / f"{AUTHORITY}.sol/{AUTHORITY}.json"
        checksums = [FILE, FILE.with_name("expected.md"), args.driver, binary]
        result["inputs"] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
                            for path in checksums}
        for threshold in (2, 1):
            result["profiles"].append(run_profile(args, args.out / f"threshold-{threshold}", threshold))
        result["after"] = revision(args.candidate)
        require(result["after"] == before, "candidate changed during the vector run")
        result["status"] = "PASS"
    except BaseException as error:
        result["failure"] = str(error)
        raise
    finally:
        result["files"] = {str(path.relative_to(args.out)): hashlib.sha256(path.read_bytes()).hexdigest()
                           for path in sorted(args.out.rglob("*")) if path.is_file()}
        save(args.out / "manifest.json", result)
        digest = hashlib.sha256((args.out / "manifest.json").read_bytes()).hexdigest()
        (args.out / "manifest.sha256").write_text(f"{digest}  manifest.json\n")
    print(args.out / "manifest.json")


if __name__ == "__main__":
    main()
