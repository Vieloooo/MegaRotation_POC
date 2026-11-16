"""Source-first ordered D oracle for finalized complete prefixes and verified DA.

Uses the existing independently transcribed 03 execution codec. The caller
supplies canonical receipt-derived events, never a production D snapshot.
"""

from copy import deepcopy
from itertools import groupby

import independent_reference as execution


def need(ok, detail):
    if not ok:
        raise ValueError(detail)


def byte_list(value):
    return list(bytes.fromhex(value.removeprefix("0x")))


def metadata(fields):
    return dict(zip(("height", "timestamp_us", "origin"), fields, strict=True))


def l1_pos(position):
    return dict(zip(("block", "tx", "log"), position, strict=True))


def genesis_from_bytes(encoded, wire):
    reader = wire.canonical(encoded, "genesis.bin")
    anchor = wire.anchor(reader)
    state_meta = metadata([reader.uint(8) for _ in range(3)])
    accounts = {}
    previous = -1
    for _ in range(reader.count(256)):
        account = reader.uint(1)
        nonce, balance = reader.uint(8), reader.uint(8)
        need(account > previous and (nonce or balance), "genesis account ordering/nonzero")
        accounts[str(account)] = {"nonce": nonce, "balance": balance}
        previous = account
    native = reader.digest()
    reader.finish(None)
    need(
        state_meta == {k: anchor[k] for k in state_meta},
        "genesis execution state metadata differs",
    )
    return {
        "anchor": {
            "block": {
                "height": anchor["height"],
                "origin": anchor["origin"],
                "block_id": byte_list(anchor["physical"]),
            },
            "timestamp_us": anchor["timestamp_us"],
        },
        "state": {"metadata": state_meta, "accounts": accounts},
        "native_hash": byte_list(native),
    }


def to_transfer(raw):
    need(len(raw) == 19 and raw[0] == 1, "non-transfer in verified frame")
    return (raw[1], raw[2], int.from_bytes(raw[3:11], "big"), int.from_bytes(raw[11:], "big"))


class Derive:
    def __init__(self, config, genesis, credits, headers, window):
        need(window > 0, "invalid sequencing window")
        self.config, self.genesis = config, genesis
        self.credits, self.headers, self.window = credits, headers, window
        self.meta = [genesis["state"]["metadata"][k] for k in ("height", "timestamp_us", "origin")]
        self.accounts = genesis["state"]["accounts"]
        self.native = bytes(genesis["native_hash"]).hex()
        self.physical = bytes(genesis["anchor"]["block"]["block_id"]).hex()
        self.complete = -1
        self.epoch_output = False
        self.pending = []
        self.blocks = []
        self.cuts = {}
        self.discarded = []
        self.at_end = {}

    def emit(self, origin, cause, transfers=()):
        height, timestamp = self.meta[0] + 1, self.meta[1] + self.config["l2_slot_us"]
        need(self.headers[origin] <= timestamp, "D selects origin after L2 timestamp")
        credits = [(row["to"], row["amount"]) for row in self.credits[height]]
        out = execution.execute(
            self.meta, self.accounts, self.native, [height, timestamp, origin], credits, transfers,
        )
        physical_bytes = (
            execution.be(1, 2) + bytes.fromhex(self.physical)
            + execution.metadata_bytes(out["metadata"])
            + bytes.fromhex(out["pre_state_root"])
            + execution.byte_vector([bytes.fromhex(tx) for tx in out["kept"]])
        )
        out["physical_parent"] = self.physical
        out["physical_hash"] = execution.digest("PoC/PhysicalBlock/v1", physical_bytes)
        out["cause"] = cause
        self.blocks.append(out)
        self.meta, self.accounts = out["metadata"], out["accounts"]
        self.physical, self.native = out["physical_hash"], out["native_hash"]
        self.epoch_output = True

    def drain(self):
        # In the complete-one-frame profile, pending records retain canonical inclusion/wire order.
        while self.pending:
            batch, index, record = self.pending[0]
            height, epoch = self.meta[0], self.meta[2]
            target, origin = record["height"], record["origin"]
            if target > height + 1:
                break
            if target <= height or batch.position[0] > origin + self.window:
                self.pending.pop(0)
                self.discarded.append({
                    "position": batch.position, "height": target,
                    "late": batch.position[0] > origin + self.window,
                    "already_derived": target <= height,
                })
                continue
            if origin not in self.headers:
                return
            timestamp = self.meta[1] + self.config["l2_slot_us"]
            transfers = [to_transfer(raw) for raw in record["transactions"]]
            drift = timestamp - self.headers[origin] > self.config["t_drift_us"]
            substitute = not epoch <= origin <= epoch + 1 or timestamp < self.headers[origin]
            if drift and not substitute:
                if not transfers and origin == epoch:
                    if epoch + 1 not in self.headers:
                        return
                    substitute = timestamp >= self.headers[epoch + 1]
                else:
                    substitute = bool(transfers)
            if substitute:
                if epoch + 1 not in self.headers:
                    return
                chosen = epoch + int(self.headers[epoch + 1] <= timestamp)
                self.emit(chosen, ("substitution", batch.position, batch.commitment, index))
            else:
                self.emit(origin, ("batch", batch.position, batch.commitment, index), transfers)
            self.pending.pop(0)

    def fallback(self):
        while len(self.blocks) < len(self.credits):
            self.drain()
            epoch, height = self.meta[2], self.meta[0] + 1
            if self.pending and self.pending[0][2]["height"] == height:
                break
            boundary = epoch + self.window
            if self.complete < boundary or epoch + 1 not in self.headers:
                break
            timestamp = self.meta[1] + self.config["l2_slot_us"]
            advance = self.epoch_output and self.headers[epoch + 1] <= timestamp
            self.emit(epoch + int(advance), ("fallback", boundary, self.complete))

    def override(self, resume):
        self.drain()
        need(resume.position not in self.cuts, "duplicate original success")
        while self.meta[0] < resume.target:
            epoch = self.meta[2]
            timestamp = self.meta[1] + self.config["l2_slot_us"]
            advance = epoch + 1 <= resume.set_position[0] and self.headers[epoch + 1] <= timestamp
            self.emit(epoch + int(advance), ("cursor_set", resume.set_position))
        self.pending.clear()
        self.cuts[resume.position] = deepcopy(self.blocks)

    def advance(self, complete, batches, resumes):
        events = (
            [(batch.position, batch) for batch in batches]
            + [(resume.set_position, resume) for resume in resumes]
        )
        events.sort(key=lambda row: row[0])
        by_block = {number: list(group) for number, group in groupby(events, key=lambda row: row[0][0])}
        for number in range(complete + 1):
            for _, event in by_block.get(number, ()):
                if hasattr(event, "records"):
                    self.pending.extend((event, index, record) for index, record in enumerate(event.records))
                    self.drain()
                else:
                    self.override(event)
            self.complete = number
            self.fallback()
            self.at_end[number] = self.meta[0]
        return self
