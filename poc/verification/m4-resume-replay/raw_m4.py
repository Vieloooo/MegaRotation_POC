"""Map sealed M4 preset-resume evidence and read-only checker outputs to replay v1."""

import json
import sys
from collections import defaultdict
from pathlib import Path

from evidence import sha
from model import SCHEMA, comparison, cut_anchor, l1pos, require

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "m1-gfr-replay"))
from wire import Reader, decode_config, decode_physical  # noqa: E402


TRACE = "raw/trace.jsonl"
D_TRACE = "raw/derivation/trace.jsonl"
RPC = "raw/l1/raw-rpc-bundle.json"
DA_INDEX = "raw/derivation/da/da-index.json"


def hx(value):
    if isinstance(value, str):
        result = value.lower().removeprefix("0x")
    else:
        result = bytes(value).hex()
    require(len(result) == 64 and all(c in "0123456789abcdef" for c in result), "invalid evidence Hash32")
    return result


def coordinates(value):
    return l1pos([value[x] for x in ("block", "tx", "log")]) if isinstance(value, dict) else l1pos(value)


def pc(value):
    kind = value["kind"]
    ix = value["index"]
    return {"kind": kind, "key_us": value["key_us"],
            "index": list(coordinates(ix["L1"])) if kind in ("Resume", "Force") else [ix["Espresso"]]}


def source(path, locator, label, **extra):
    return {"path": path, "locator": locator, "label": label, **extra}


def trace_ref(row, timing=False):
    return source(TRACE, f"行 {row['line']} · seq {row['seq']} · {row['kind']}",
                  "Hub 原始观测", line=row["line"], trace_event=timing)


def one(items, label):
    rows = list(items)
    require(len(rows) == 1, label)
    return rows[0]


def index(rows, key, label):
    out = {}
    for row in rows:
        ix = key(row)
        require(ix not in out, f"{label}: duplicate key {ix}")
        out[ix] = row
    return out


def espresso(raw, path):
    """Decode PELG and GFRP payloads shared by the M1–M4 runs."""
    reader = Reader(raw, path, "little")
    reader.expect(b"PELG\x01\x00")
    log_source = reader.frame(256).decode("ascii")
    q, key, proposed = reader.uint(8), reader.uint(8), reader.uint(8)
    reader.expect(b"\x01")
    finalized = reader.uint(8)
    reader.check(bool(log_source) and proposed <= finalized, "invalid public row")
    bodies, proofs = {}, []
    for ordinal in range(reader.count(1024)):
        reader.check(reader.uint(4) == 0x47465201, "invalid GFRP codec")
        obj = Reader(reader.frame(65_536), f"{path} item {ordinal}")
        obj.expect(b"GFRP\0\1")
        tag = obj.uint(1)
        if tag == 0:
            block = decode_physical(obj.frame(65_536), path)
            require(block["physical"] not in bodies, "duplicate body in Espresso row")
            bodies[block["physical"]] = (ordinal, block)
        else:
            obj.check(tag == 1, "invalid GFRP variant")
            descriptor = obj.frame(16_384)
            obj.check(descriptor.startswith(b"\0\1") and len(descriptor) > 32, "malformed descriptor envelope")
            target = obj.digest()
            proof = obj.frame(32_768).hex()
            proofs.append(dict(message=ordinal, cid=descriptor[-32:].hex(), target=target, proof=proof))
        obj.finish(None)
    return reader.finish(dict(q=q, theta_us=key, proposal_us=proposed, finalized_us=finalized, source=log_source,
                              bodies=bodies, proofs=proofs, path=path))


def u64(value):
    return value.to_bytes(8, "big")


def execution_anchor_bytes(value):
    anchor = value["anchor"]
    block = anchor["block"]
    state = value["state"]
    meta = state["metadata"]
    accounts = sorted((int(key), info) for key, info in state["accounts"].items())
    return (b"\0\1" + u64(block["height"]) + u64(block["origin"]) + bytes(block["block_id"])
            + u64(anchor["timestamp_us"]) + u64(meta["height"]) + u64(meta["timestamp_us"]) + u64(meta["origin"])
            + len(accounts).to_bytes(4, "big")
            + b"".join(bytes([pid]) + u64(account["nonce"]) + u64(account["balance"]) for pid,account in accounts)
            + bytes(value["native_hash"]))


def espresso_position_bytes(position):
    require(position["kind"] == "Espresso", "batch fixture must use committed Espresso position")
    return b"\0\1" + u64(position["key_us"]) + b"\4" + u64(position["index"][0])


def descriptor_duty(value):
    plan = value["plan"]
    return {"branch": plan["session"]["beta"], "r": plan["session"]["r"], "tail": "NEXT", "region": plan["region"],
            "window": [plan["window"]["h_min"], plan["window"]["h_max"]], "cid": hx(value["cid"]),
            "ref": hx(plan["reference_block"]["block_id"]), "anchor": hx(value["anchor"]["block"]["block_id"]),
            "source_region": plan["source_region"]}


def block_from_d(item, checked):
    raw = item["native_block"]
    meta = raw["metadata"]
    kind = {"accepted_batch": "substitution" if item["cause"].get("substituted") else "batch",
            "fallback": "window-fallback", "cursor_set": "cursor-fill"}[item["cause"]["kind"]]
    block = {**meta, "native_parent": hx(raw["parent"]), "native": hx(item["native_hash"]),
             "pre": hx(item["virtual_block"]["pre_state_root"]), "post": hx(raw["post_state_root"]),
             "ordinary_count": len(raw["kept"]), "kind": kind, "provenance": "D · L1/DA"}
    require((checked["metadata"], checked["native_hash"], checked["native_parent"], checked["pre_state_root"],
             checked["post_state_root"], checked["kept"]) ==
            ([meta["height"], meta["timestamp_us"], meta["origin"]], block["native"], block["native_parent"],
             block["pre"], block["post"], [bytes(tx).hex() for tx in raw["kept"]]), "D item differs from independent execution")
    return block


def open_rows(sealed, path):
    return [{**json.loads(line), "line": line_no}
            for line_no, line in enumerate(sealed.read(path).splitlines(), 1)]


class Mapper:
    def __init__(self, sealed, manifest, report, checker):
        self.sealed, self.report = sealed, report
        self.rows = open_rows(sealed, TRACE)
        kinds = defaultdict(list)
        for row in self.rows:
            kinds[row["kind"]].append(row)
        self.kinds = kinds
        self.allocated = [0, 0]
        self.events = []
        self.checked_g = checker["public"]
        self.checked_d = index(checker["d"]["blocks"], lambda d: d["metadata"][0], "independent D")
        self.cut_check = one(checker["cuts"], "one validated DR01 cut")
        self.verified_l1 = index(checker["headers"], lambda h: h["height"], "independent L1")
        self.config = decode_config(sealed.read("raw/config.bin"), "config.bin")
        raw_profile = self.json("raw/profile.json")
        require(raw_profile["seed"] == manifest["seed"] and self.config["ref"] == self.checked_g["config_ref"],
                "profile or configuration differs from independent result")
        activate = one(kinds["ResumeActivated"], "one observed activation")
        self.activation = activate
        first_cid = hx(kinds["GAccepted"][0]["detail"]["cid"])
        self.genesis = {**self.config["genesis"], "kind": "genesis", "provenance": "genesis",
                        "native": self.config["genesis_native_hash"], "physical": self.config["genesis"]["physical"],
                        "pre": self.config["genesis_state_root"], "post": self.config["genesis_state_root"]}
        self.profile = {
            "seed": manifest["seed"], "tb": 10, "announcement_us": raw_profile["announcement_us"],
            "expiry_us": raw_profile["expiry_us"], "grace_us": raw_profile["grace_us"],
            "genesis_timestamp_us": self.genesis["timestamp_us"], "slot_us": self.config["l2_slot_us"],
            "sequencing_window": raw_profile["resume"]["sequence_window_blocks"], "genesis": self.genesis,
            "initial_duty": {"region":0, "branch":0, "r":0, "window":[1,10], "cid":first_cid, "tail":"NEXT"},
        }
        rpc = self.json(RPC)
        self.headers = {}
        self.rpc_headers = rpc["blocks"]
        self.receipts = {row["transactionHash"].lower(): (i, row) for i,row in enumerate(rpc["receipts"])}
        for header_index, raw in enumerate(self.rpc_headers):
            height = int(raw["number"], 16)
            verified = self.verified_l1[height]
            require(height == header_index and int(raw["timestamp"], 16) == verified["timestamp_s"]
                    and hx(raw["hash"]) == hx(verified["hash"]), "actual Anvil differs from independently verified header")
            self.headers[height] = dict(height=height, timestamp_us=int(raw["timestamp"],16)*1_000_000,
                                        hash=hx(raw["hash"]), parent=hx(raw["parentHash"]))
        self.batches = index(report["batches"], lambda b: b["tx_hash"].lower(), "checked batch")
        self.batch_position = {coordinates(b["position"]): b for b in self.batches.values()}
        self.resume = one(report["resumes"], "exactly one checked RESUME")
        self.resume_pos = coordinates(self.resume["position"])
        self.reset = one(kinds["GReset"], "exactly one G reset")
        self.old_heads = [self.genesis]
        self.d_heads = [self.genesis]
        self.branch_cids = {first_cid}
        self.l1_available = -1
        self.pending_resume = None
        self.derivation = defaultdict(list)
        self.da_idx = next((p for p in sealed.entries if p.startswith("raw/") and p.endswith("/da-index.json")), None)
        self.da_files = self.json(self.da_idx)["payloads"] if self.da_idx else None
        self.source_by_tx = {}
        for row in open_rows(sealed, D_TRACE):
            self.derivation[(row["reader"], row["l1_height"], row["tick_us"])].append(row)
        self.bind_public()

    def json(self, path):
        return json.loads(self.sealed.read(path))

    def artifact(self, name):
        return one((p for p in self.sealed.entries if p.startswith("raw/") and p.endswith("/" + name)),
                   f"expected one sealed {name}")

    def emit(self, row, kind, detail, *extra, stage=None):
        if self.allocated[0] != row["seq"]:
            self.allocated = [row["seq"], 0]
        suborder = self.allocated[1]
        self.events.append({"seq":row["seq"], "suborder":suborder, "tick_us":row["tick_us"], "kind":kind, "stage":stage,
                            "detail":detail, "refs":[trace_ref(row, timing=True), *extra]})
        self.allocated[1] += 1

    def bind_public(self):
        pelg = [espresso(self.sealed.read(path), path) for path in sorted(self.sealed.entries)
                if path.startswith("raw/espresso/") and path.endswith(".bin")]
        self.pelg = index(pelg, lambda row: row["q"], "PELG")
        require(list(self.pelg) == list(range(len(pelg)))
                and [{"q": x["q"], "theta_us": x["theta_us"], "proposed_us": x["proposal_us"],
                      "finalized_us": x["finalized_us"]} for x in pelg] == self.checked_g["espresso"],
                "complete PELG header sequence differs from checker")
        self.bodies = {}
        for row in pelg:
            for physical, (ordinal, block) in row["bodies"].items():
                previous = self.bodies.setdefault(physical, (row["q"], ordinal, block))
                require(previous[2] == block, "one public body hash has conflicting objects")
        self.public_proofs = index(self.checked_g["public_finals"], lambda row: row["target"], "checked public proof")
        raw_proof_keys = set()
        for row in pelg:
            for proof in row["proofs"]:
                public = self.public_proofs[proof["target"]]
                require(all(public[key] == value for key,value in proof.items())
                        and public["q"] == row["q"], "checked PELG proof identity differs")
                raw_proof_keys.add(proof["target"])
        require(raw_proof_keys == set(self.public_proofs), "checker and PELG public proof inventory differ")
        self.regional_rows = {}
        self.results = {}
        for row in self.kinds["RegionalDelivered"]:
            d = row["detail"]
            physical = hx(d["physical_id"])
            if physical not in self.public_proofs or physical in self.regional_rows:
                continue
            self.regional_rows[physical] = row
        self.gblocks = {}
        state_root, parent = self.genesis["post"], self.genesis["native"]
        for raw, verified in zip(self.kinds["GAccepted"], self.checked_g["regional_entries"], strict=True):
            d = raw["detail"]
            physical = hx(d["physical_id"])
            q = verified["q"]
            require(d["height"] == verified["height"] and hx(d["cid"]) == verified["cid"]
                    and hx(d["native_hash"]) == verified["native"] and hx(d["state_root"]) == verified["state_root"]
                    and hx(d["parent"]) == verified["parent"] and physical == verified["physical"]
                    and pc(d["position"]) == {"kind":"Espresso", "key_us":verified["theta_us"], "index":[q]},
                    "G acceptance differs from independent replay")
            artifact = self.artifact(f"results/{d['height']:02}.bin")
            reader = Reader(self.sealed.read(artifact), artifact)
            reader.expect(b"H3RS\x00\x01")
            descriptor = reader.frame(16_384)
            require(len(descriptor) > 32 and descriptor[-32:].hex() == verified["cid"],
                    "regional result descriptor CID differs")
            result = decode_physical(reader.frame(186), artifact)
            proof = reader.frame(32_768).hex()
            chain = [decode_physical(reader.frame(186), artifact) for _ in range(reader.count(10))]
            require(result == chain[-1], "regional result path does not end in its body")
            public = self.public_proofs[physical]
            listed = [self.bodies[b["physical"]] for b in chain]
            require(all(a[2] == b for a,b in zip(listed, chain, strict=True)) and proof == public["proof"]
                    and public["q_avail"] == verified["q_avail"] == max(public["q"], *(b[0] for b in listed)),
                    "regional body/proof differ from finalized publication")
            require({key:result[key] for key in verified["metadata"]} == verified["metadata"]
                    and result["physical"] == physical and result["parent"] == verified["parent"],
                    "regional block metadata or physical identity differs")
            if verified["beta"]:
                exact_anchor = self.cut_check["anchor"]
                state_root = self.cut_check["prefix"][self.resume["target"]-1]["state_root"]
                parent = hx(exact_anchor["native_hash"])
                source_tag = reader.uint(1)
                source_position = Reader(reader.frame(66), artifact)
                source_position.expect(b"\x00\x01")
                source_pos = [source_position.uint(8) for _ in range(3)]
                source_position.finish(None)
                require(source_tag == 2 and source_pos == list(self.resume_pos), "recovered result needs its exact ResumeCut source")
                count = reader.count(16)
                require(count == 1 and reader.frame(4096) == execution_anchor_bytes(exact_anchor),
                        "recovered result anchor object differs from exact cut")
                reader.finish(None)
                require(result["parent"] == hx(exact_anchor["anchor"]["block"]["block_id"]), "recovered result parent differs from cut")
            require(result["pre_state_root"] == state_root, "regional pre-state differs from actual parent execution")
            block = {**verified["metadata"], "native":verified["native"], "pre":state_root, "post":verified["state_root"],
                     "native_parent":parent, "physical":physical, "physical_parent":result["parent"], "kind":"regional",
                     "ordinary_count":verified["kept"], "provenance":"Regional"}
            self.gblocks[q] = (raw, verified, block)
            self.results[physical] = (artifact, block, public)
            state_root, parent = block["post"], block["native"]

    def l1_ref(self, height):
        return source(RPC, f"blocks[{height}]", "实际 Anvil header")

    def receipt(self, tx, pos):
        idx, raw = self.receipts[tx]
        require(int(raw["blockNumber"],16) == pos[0] and int(raw["transactionIndex"],16) == pos[1]
                and hx(raw["blockHash"]) == self.headers[pos[0]]["hash"]
                and hx(self.rpc_headers[pos[0]]["transactions"][pos[1]]["hash"]) == hx(tx), "receipt canonical location mismatch")
        return raw, source(RPC, f"receipts[{idx}] · {tx}", "实际 Anvil 收据")

    def batch_ref(self, batch):
        return source(self.artifact(f"batch-h{batch['records'][0]['height']}.json"),
                      "实际源 G 快照与提交记录", "批次任务")

    def l1_included(self, row):
        height = row["detail"]["height"]
        header = self.headers[height]
        require(hx(row["detail"]["hash"]) == header["hash"]
                and row["detail"]["timestamp_s"]*1_000_000 == header["timestamp_us"]
                and self.report["l1"]["mine_us"][str(height)] == row["tick_us"], "Anvil inclusion stage differs")
        for tx, batch in self.batches.items():
            pos = coordinates(batch["position"])
            if pos[0] != height:
                continue
            receipt, ref = self.receipt(tx, pos)
            status = int(receipt["status"],16)
            require(status == 1 and int(receipt["logs"][pos[2]]["logIndex"],16) == batch["log_index"],
                    "checked CursorAdvanced requires actual accepted batch")
            record = batch["records"][0]
            deadline = record["origin"] + self.profile["sequencing_window"]
            self.emit(row, "batch_receipt", dict(tx=tx, transaction_position=list(pos[:2]), status=status,
                      commitment=hx(batch["commitment"]), header=header, window_end=deadline,
                      late=height>deadline), ref, self.batch_ref(batch), stage="late-batch" if height>deadline else "batch")
        pos = self.resume_pos
        if pos[0] == height:
            tx = self.resume["tx_hash"].lower()
            set_pos = self.resume["set_position"]
            receipt, ref = self.receipt(tx, pos)
            require(int(receipt["status"],16) == 1
                    and int(receipt["logs"][pos[2]]["logIndex"],16) == self.resume["resume_log_index"]
                    and int(receipt["logs"][set_pos[2]]["logIndex"],16) == self.resume["set_log_index"],
                    "checked RESUME requires paired logs on actual success receipt")
            prev = max(record["height"] for batch in self.batches.values() for record in batch["records"]
                       if coordinates(batch["position"]) < coordinates(set_pos))
            self.pending_resume = dict(tx=tx, position=list(pos), cursor_position=set_pos, status=1,
                                       receipt_log_index=self.resume["resume_log_index"],
                                       cursor_rpc_log_index=self.resume["set_log_index"], header=header,
                                       target=self.resume["target"], previous=prev, control_seq=self.resume["seq"],
                                       signed_origin=self.resume["signed_origin"], signed_floor=self.resume["signed_floor"],
                                       policy={"mask": self.resume["policy"]["enabled_mask"], "order": self.resume["policy"]["order"]},
                                       signers=self.resume["signers"], act_us=self.resume["act_us"])
            self.emit(row, "resume_included", self.pending_resume, ref, self.l1_ref(height), stage="resume-included")

    def derive_accepted(self, row, raw, deriv_ref):
        step = raw["step"]
        position = coordinates(step["position"])
        batch = self.batch_position[position]
        tx = batch["tx_hash"].lower()
        submitted = self.source_by_tx[tx]
        accepted_path = self.artifact("accepted-A-" + "-".join(str(field) for field in position) + ".json")
        accepted = self.json(accepted_path)
        header = self.headers[position[0]]
        commitment = hx(batch["commitment"])
        receipted, tx_ref = self.receipt(tx, position)
        require(accepted["evidence"]["receipt"] == receipted and accepted["evidence"]["header"] == self.rpc_headers[position[0]]
                and coordinates(accepted["position"]) == position and hx(accepted["identity"]["block_hash"]) == header["hash"]
                and hx(accepted["identity"]["transaction_hash"]) == hx(tx)
                and accepted["rpc_log_index"] == batch["log_index"]
                and accepted["submission"] == submitted["certified"], "accepted custody does not bind actual batch")
        da_path = (str(Path(self.da_idx).parent / self.da_files[batch["commitment"]])
                   if self.da_idx else f"raw/derivation/da/0x{commitment}.bin")
        data = self.sealed.read(da_path)
        require(sha(data) == batch["payload_sha256"], "DA bytes differ from checked accepted frame")
        previous = batch["previous_height"]
        target = batch["records"][-1]["height"]
        gpos = pc(submitted["snapshot"]["position"])
        accept_ref = source(accepted_path, "CursorAdvanced 与认证输入", "finalized Inbox 证据")
        da_ref = source(da_path, "完整帧字节", "经校验的 DA object")
        receipt_ref = source(da_path.removesuffix(".bin") + ".receipt", "DA receipt", "DA 持久收据")
        refs = [accept_ref, tx_ref, self.batch_ref(batch), deriv_ref]
        self.emit(row, "inbox_advanced", dict(tx=tx, position=list(position), receipt_log_index=batch["log_index"],
                  previous=previous, next=target, commitment=commitment, range=[previous+1,target], gpos=gpos,
                  frame_number=0, closed=True), *refs)
        self.emit(row, "da_verified", dict(position=list(position), commitment=commitment,
                  verified_commitment=commitment, valid=True, object=sha(data), length=len(data),
                  timing="随派生可读观测，使用 accepted record 与已封存 DA 对象"), *refs, da_ref, receipt_ref)
        head = submitted["execution"][-1]
        self.emit(row, "gprime_update", dict(position=list(position), source="batch", head=hx(head["native_hash"]),
                  gpos=gpos, reconstructed=True), *refs, da_ref)
        if raw["new_items"]:
            blocks = self.decode_d(raw)
            self.emit(row, "d_append", dict(position=list(position), reason="batch", blocks=blocks,
                      da_commitment=commitment, batch_disposition=step["outcome"]), *refs, da_ref,
                      stage="derived" if previous == 0 else None)
        else:
            reason = {"old": "already-derived", "late": "late"}[step["outcome"]]
            self.emit(row, "d_skipped", dict(position=list(position), reason=reason,
                      origin=batch["records"][0]["origin"], height=step["height"], source_result=step["outcome"]),
                      *refs, da_ref, stage="late-skip")

    def decode_d(self, raw):
        blocks = []
        for item in raw["new_items"]:
            height = item["state"]["metadata"]["height"]
            block = block_from_d(item, self.checked_d[height])
            blocks.append(block)
            self.d_heads.append(block)
        return blocks

    def derive_tape(self, row):
        detail = row["detail"]
        reader, l1 = detail["reader"], detail["l1_height"]
        matching = self.derivation[(reader,l1,row["tick_us"])]
        require(matching, "missing source D observations for Hub DerivationAvailable")
        rows = []
        for raw in matching:
            ref = source(D_TRACE, f"行 {raw['line']} · reader {reader} · ℓ{l1}",
                         "独立派生输入和步骤", line=raw["line"])
            rows.append(ref)
            if reader != "A" or "step" not in raw:
                continue
            step = raw["step"]
            if step["kind"] == "accepted_record":
                self.derive_accepted(row, raw, ref)
            elif step["kind"] == "fallback":
                [block] = self.decode_d(raw)
                cause = raw["new_items"][0]["cause"]
                frontier = cause["frontier"]["height"]
                require(hx(cause["frontier"]["hash"]) == self.headers[frontier]["hash"], "fallback frontier header")
                origin = cause["window_end"] - self.profile["sequencing_window"]
                self.emit(row, "d_append", dict(input_point=f"BlockEnd ℓ{frontier}",
                          reason="window-fallback", blocks=[block], origin=origin, window=self.profile["sequencing_window"],
                          processed_through=frontier, next_origin=origin+1), ref, self.l1_ref(frontier),
                          stage="fallback" if block["height"] == 2 else None)
            elif step["kind"] == "cursor_set":
                self.capture(row, raw, ref)
            else:
                require(not raw["new_items"], "unhandled D output step")
        head = detail["head"]
        if head:
            height = head["state"]["metadata"]["height"]
            expected = self.checked_d[height]["native_hash"] if height else self.genesis["native"]
            require(hx(head["native_hash"]) == expected, "D reader head differs from independent execution")
        if reader != "A":
            for recorded in detail["cuts"]:
                path = self.artifact(recorded["file"])
                require(hx(recorded["digest"]) == self.cut["cut_id"]
                        and self.sealed.read(path) == self.sealed.read(self.cut["cut_bytes_path"]),
                        "reader's historical ResumeCut differs from A")
                rows.append(source(path,"逐字节一致的历史切面",f"reader {reader}"))
        self.emit(row, "d_available", {"reader":reader, "l1_height":l1,
                  "head":head["state"]["metadata"]["height"] if head else None,
                  "native":hx(head["native_hash"]) if head else None, "wait":detail["wait"],
                  "progress":"D reader 实际输出与状态"}, rows[-1])

    def capture(self, row, raw, deriv_ref):
        step = raw["step"]
        position = coordinates(step["resume_position"])
        pair = coordinates(step["position"])
        require(position == self.resume_pos and pair == coordinates(self.resume["set_position"]), "D CursorSet linkage")
        recorded = one((d for d in row["detail"]["cuts"] if coordinates(d["position"]) == position),
                       "observed cut file and digest")
        path = self.artifact(recorded["file"])
        exact = self.json(path)
        digest_value = hx(recorded["digest"])
        fill = self.decode_d(raw)
        require(len(fill) == step["fill_count"], "recorded CursorSet fill count differs")
        target = self.resume["target"]
        block = cut_anchor(self.d_heads[target])
        cut_detail = dict(position=list(position), cursor_position=list(pair), cut_id=digest_value,
                          target=target, fill=fill, prefix=[item["native"] for item in self.d_heads],
                          anchor=block, cut_bytes_path=path)
        self.sealed.check_cut(cut_detail, self.pending_resume)
        require(self.cut_check["position"] == list(position) and self.cut_check["cut_digest"] == digest_value
                and self.cut_check["prefix"] == [
                    {"height": h["height"], "origin": h["origin"], "native": h["native"], "state_root": h["post"],
                     "kept": h["ordinary_count"]} for h in self.d_heads[1:]]
                and self.cut_check["anchor"] == exact["target_anchor"], "raw cut differs from independent cut projection")
        self.cut = cut_detail
        cut_ref = source(path, f"p_res={position} / CursorSet={pair}", "紧凑 ResumeCut 原始字节")
        self.cut_ref = cut_ref
        self.emit(row, "resume_cut", cut_detail, deriv_ref, cut_ref, stage="cut")
        self.emit(row, "gprime_update", dict(position=list(pair), source="cut", resume_position=list(position),
                  cut_id=digest_value, head=block["native"], reconstructed=True), cut_ref, deriv_ref, stage="gprime-cut")

    def coverage(self, position):
        q = (position["index"][0] if position["kind"] == "Espresso" else
             min(q for q,row in self.pelg.items() if row["theta_us"] >= position["key_us"]))
        require(self.headers[self.l1_available]["timestamp_us"] >= position["key_us"] - self.profile["announcement_us"],
                "actual L1 reader lacks public coverage")
        return dict(q=q, l1_height=self.l1_available)

    def g_record(self, row):
        raw = row["detail"]
        position = pc(raw["position"])
        detail = {"position": position, "coverage": self.coverage(position)}
        events = raw["events"]
        refs = [source(self.pelg[detail["coverage"]["q"]]["path"], "公共覆盖块", "Espresso finalized")]
        require(all(pc(event["position"]) == position for event in events), "lifecycle events have another public position")
        if position["kind"] == "Resume":
            activated = self.activation
            applied = activated["detail"]
            signed = self.resume
            check = self.report["protocol"]
            independent = one(self.checked_g["resumes"], "one checker RESUME record")
            digest_value = self.cut["cut_id"]
            native_old = [block["native"] for block in self.old_heads]
            target = self.cut["target"]
            new = self.cut["prefix"][:target+1]
            kept = comparison(self.old_heads, self.d_heads[:target+1])["common_prefix"]
            old_suffix = native_old[kept+1:]
            reset = self.reset["detail"]
            require((reset["old_height"],reset["common_height"],reset["target_height"],
                     [hx(v) for v in reset["old_hashes"]],hx(reset["cut_digest"]), coordinates(reset["resume_position"]))
                    == (len(native_old)-1,kept,target,old_suffix,digest_value,self.resume_pos), "GReset disagrees with complete suffix")
            derived = [r for r in self.kinds["GDerived"] if pc(r["detail"]["position"]) == position]
            adopted = self.d_heads[kept+1:target+1]
            require([(r["detail"]["height"],hx(r["detail"]["native_hash"]),hx(r["detail"]["state_root"]),
                      coordinates(r["detail"]["resume_position"])) for r in derived]
                    == [(block["height"],block["native"],block["post"],self.resume_pos) for block in adopted],
                    "GDerived disagrees with atomic adopted suffix")
            duty = descriptor_duty(applied["descriptor"])
            duty.update(floor=applied["l1_floor"], plan_height=applied["plan_basis_height"])
            require(raw["resume"]["target"]["anchor"] == applied["descriptor"]["anchor"] == self.cut_check["anchor"]["anchor"]
                    and hx(raw["resume"]["cut_digest"]) == hx(applied["cut_digest"]) == check["cut_digest"] == digest_value
                    and hx(raw["id"]) == hx(applied["record_id"]) and pc(applied["position"]) == position
                    and coordinates(raw["resume"]["position"]) == coordinates(applied["resume_position"]) == self.resume_pos
                    and coordinates(raw["resume"]["set_position"]) == coordinates(applied["set_position"]) == coordinates(signed["set_position"])
                    and position["key_us"] == signed["act_us"] == check["act_us"]
                    and row["tick_us"] == activated["tick_us"] == check["apply_us"]
                    and hx(applied["anchor_native_hash"]) == self.cut["anchor"]["native"]
                    and (independent["common_height"], independent["old_head"], independent["region"], independent["h_min"], independent["h_max"])
                    == (kept, len(native_old)-1, duty["region"], *duty["window"]), "GFR activation differs from checker")
            require(all(item["native"] == block["native"] and item["state_root"] == block["post"] and item["source"] == "DER"
                        for block,item in zip(adopted, self.checked_g["expected"][kept:target],strict=True)),
                    "checked G adopted DER prefix differs")
            lifecycle = [next(iter(e["body"])) for e in events]
            require(events[1]["body"]["Resumed"]["anchor"] == self.cut_check["anchor"]["anchor"]
                    and events[2]["body"]["SessionAssigned"]["descriptor"] == applied["descriptor"], "atomic lifecycle anchor/Plan differs")
            detail.update(resume_position=list(self.resume_pos), cut_id=digest_value, before=native_old, after=new,
                          retain_through=kept, revoked=old_suffix, adopted=new[kept+1:], duty=duty, lifecycle=lifecycle,
                          closed_branch=events[0]["body"]["BranchClosed"]["branch"], closed_cids=sorted(self.branch_cids),
                          plan_header_hash=self.headers[duty["plan_height"]]["hash"])
            refs += [self.cut_ref, trace_ref(self.reset), trace_ref(activated), *(trace_ref(r) for r in derived),
                     self.l1_ref(duty["plan_height"])]
            self.emit(row, "resume_applied", detail, *refs, stage="resume")
            self.old_heads = self.d_heads[:target+1]
        elif raw["append"]:
            q = position["index"][0]
            accepted, verified, block = self.gblocks[q]
            [span] = raw["append"]
            require(span["first"] == span["last"] == block["height"] and hx(span["cid"]) == verified["cid"]
                    and hx(span["physical_id"]) == block["physical"], "G append source mismatch")
            path, _, proof = self.results[block["physical"]]
            detail.update(block=block, cid=verified["cid"], q_avail=verified["q_avail"], proof=sha(bytes.fromhex(proof["proof"])))
            if events:
                duty = descriptor_duty(events[-1]["body"]["SessionAssigned"]["descriptor"])
                detail["handoff"] = duty
                self.branch_cids.add(duty["cid"])
            self.old_heads.append(block)
            self.emit(row, "g_accepted", detail, *refs, trace_ref(accepted), source(path,"regional result","区域 Final"),
                      stage="accepted" if verified["beta"] else None)
        else:
            self.emit(row, "g_empty", detail, *refs)

    def emit_public(self, row):
        detail = row["detail"]
        q = detail["q"]
        block = self.pelg[q]
        require(block["theta_us"] == detail["theta_us"] and block["finalized_us"] == row["tick_us"],
                "public observed time differs from raw PELG")
        objects = []
        for proof in block["proofs"]:
            target = proof["target"]
            public = self.public_proofs[target]
            require(public["q_avail"] == q and target in block["bodies"], "DR01 publication expects complete body and proof in its recorded q")
            objects.append(dict(physical=target, proof=sha(bytes.fromhex(proof["proof"])), message=proof["message"],
                                body_index=block["bodies"][target][0], signers=public["signers"]))
        if row["kind"] == "PublisherRecorded":
            [complete] = objects
            require(hx(detail["physical_id"]) == complete["physical"] and detail["q_avail"] == q
                    and detail["message_index"] == complete["message"], "PublisherRecord differs from sealed PELG")
        else:
            require(not objects and detail["message_count"] == len(block["bodies"]) == 0, "heartbeat carries publication")
        self.emit(row, "espresso", dict(q=q, theta_us=block["theta_us"], objects=objects,
                  proposal_tick_us=block["proposal_us"], finalized_tick_us=block["finalized_us"]),
                  source(block["path"], "PELG finalized", "Espresso 原始块"))

    def step(self, row):
        k, d = row["kind"], row["detail"]
        if k == "L1Included":
            self.l1_included(row)
        elif k == "L1Finalized":
            h = d["height"]
            require(self.report["l1"]["finalized_us"][str(h)] == row["tick_us"], "L1 finality differs from checker")
            self.emit(row, "l1_finalized", self.headers[h], self.l1_ref(h))
        elif k == "L1ReaderAvailable":
            h, reader = d["height"], d["reader"]
            if h == 0 and reader == "A":
                self.emit(row, "l1_finalized", {**self.headers[h], "genesis": True}, self.l1_ref(h))
            elif h > 0:
                require(self.report["l1"]["deliver_us"][reader][str(h)] == row["tick_us"],
                        "L1 read time differs from checker")
            if reader == "A":
                self.l1_available = h
            require(hx(d["hash"]) == self.headers[h]["hash"], "actual L1 reader observed another hash")
            self.emit(row, "l1_available", dict(reader=reader, height=h, hash=hx(d["hash"])), self.l1_ref(h))
        elif k == "DerivationAvailable":
            self.derive_tape(row)
        elif k == "DerivationConflict":
            point = comparison(self.old_heads, self.d_heads)
            h = point["first_diff"]
            require(h == d["height"] and self.d_heads[h]["native"] == hx(d["d_hash"])
                    and self.old_heads[h]["native"] == hx(d["g_hash"])
                    and self.old_heads[1]["native"] == self.d_heads[1]["native"] == hx(d["common_h1"]),
                    "actual common-height divergence differs")
            self.emit(row, "compare", point, stage="divergence")
        elif k == "BatchSubmitted":
            tx = d["hash"].lower()
            batch = self.batches[tx]
            require(hx(d["commitment"]) == hx(batch["commitment"])
                    and bytes(d["snapshot"]["gpos"]).hex() == batch["gpos"]
                    and d == self.json(self.batch_ref(batch)["path"])
                    and row["tick_us"] == self.report["l1"]["submit_us"][tx],
                    "batch source or submission differs from verified input")
            source_position = pc(d["snapshot"]["position"])
            old = self.old_heads[d["height"]]
            actual = self.gblocks[source_position["index"][0]][2]
            require(hx(d["execution"][0]["native_hash"]) == old["native"] == actual["native"]
                    and bytes(d["snapshot"]["gpos"]) == espresso_position_bytes(source_position),
                    "batch captured G source or canonical gpos differs")
            self.source_by_tx[tx] = d
            self.emit(row, "batch_submitted", dict(tx=tx, commitment=hx(d["commitment"]), height=d["height"],
                      gpos=source_position, source_native=old["native"]), self.batch_ref(batch))
        elif k == "ResumeSubmitted":
            tx = d["hash"].lower()
            artifact = self.artifact("resume-authority.json")
            require(d == self.json(artifact) and tx == self.resume["tx_hash"].lower()
                    and row["tick_us"] == self.report["l1"]["submit_us"][tx]
                    and [sig["member"].lower() for sig in d["signers"]] == self.resume["signers"],
                    "submitted governance differs from actual receipt")
            request = d["request"]
            self.emit(row, "resume_submitted", dict(tx=tx, target=request["target"], g=request["g_head"],
                      d=request["d_head"], inbox=request["inbox_height"], governance_threshold=f"{len(d['signers'])}/3",
                      signers=self.resume["signers"], request_digest=d["digest"]),
                      source(artifact,"治理签名与发送请求","Authority 交易"))
        elif k in ("EspressoHeartbeat", "PublisherRecorded"):
            self.emit_public(row)
        elif k == "GfrReplayRecord":
            self.g_record(row)
        elif k == "RegionalDelivered":
            physical = hx(d["physical_id"])
            if self.regional_rows.get(physical) == row and physical in self.results:
                path, block, proof = self.results[physical]
                require(hx(d["cid"]) == proof["cid"] and d["height"] == block["height"], "regional delivery differs")
                self.emit(row, "regional_final", dict(cid=proof["cid"], block=block, proof=sha(bytes.fromhex(proof["proof"])),
                          signer_count=len(proof["signers"]), signers=proof["signers"], committee_size=5),
                          source(path,"正文、祖先与证明","区域结果"), stage="new-result" if block["height"]>self.resume["target"] else None)
        elif k in ("Ready", "Enter", "Close"):
            self.emit(row, "worker_lifecycle", dict(kind_label=k, worker=row["pid"], cid=hx(d["cid"]),
                      region=row["pid"]//5, lifecycle_detail=d))

    def tape(self):
        for row in self.rows:
            self.step(row)
        require(self.old_heads[-1]["height"] == self.report["protocol"]["restored_height"]
                and self.old_heads[-1]["native"] == self.report["protocol"]["restored_native"],
                "projected recovery endpoint differs from independent PASS")
        return {"schema":SCHEMA, "mode":"observed", "profile":self.profile, "events":self.events}
