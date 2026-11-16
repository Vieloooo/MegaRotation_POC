"""Validate and fold a normalized M4 evidence tape, without running consensus or L2 execution."""

import hashlib
import json
import re
from copy import deepcopy


SCHEMA = "M4-D-RESUME-REPLAY-1"
CHAINS = ("g", "gprime", "d")
ORDER = {"Force": 0, "Resume": 0, "Expire": 1, "SessionEnd": 2, "Espresso": 3}
STAGES = {
    "batch_submitted": ("l1", "提交批次"),
    "batch_receipt": ("l1", "批次收录"),
    "inbox_advanced": ("l1", "Inbox 接纳"),
    "l1_finalized": ("l1", "L1 finalized"),
    "l1_available": ("l1", "L1 reader 可读"),
    "da_wait": ("da", "等待已接纳 DA"),
    "da_verified": ("da", "DA 校验"),
    "d_append": ("d", "D 派生"),
    "gprime_update": ("gprime", "G′ 物化"),
    "compare": ("d", "共高比较"),
    "resume_submitted": ("l1", "提交 RESUME"),
    "resume_included": ("l1", "RESUME 收录"),
    "resume_cut": ("d", "历史 ResumeCut"),
    "espresso": ("espresso", "Espresso finalized"),
    "regional_final": ("regional", "区域 Final"),
    "g_wait": ("g", "GFR 等待输入"),
    "resume_applied": ("g", "RESUME 原子激活"),
    "g_accepted": ("g", "G 接纳"),
    "g_empty": ("g", "GFR 位置推进"),
    "d_skipped": ("d", "D 略过输入"),
    "g_timer": ("g", "GFR 定时器"),
    "worker_lifecycle": ("regional", "节点职责"),
}


def require(condition, label):
    if not condition:
        raise ValueError(label)


def digest(value):
    return isinstance(value, str) and re.fullmatch(r"[a-f0-9]{64}", value) is not None


def uint(value):
    return type(value) is int and value >= 0


def l1pos(value):
    require(isinstance(value, list) and len(value) == 3 and all(map(uint, value)), "invalid L1 position")
    return tuple(value)


def order(position):
    kind = position["kind"]
    index = position["index"]
    widths = {"Force": 3, "Resume": 3, "Expire": 1, "SessionEnd": 2, "Espresso": 1}
    require(uint(position["key_us"]) and kind in widths and isinstance(index, list)
            and len(index) == widths[kind] and all(map(uint, index)), "invalid public position")
    return position["key_us"], ORDER[kind], tuple(index)


def derived_id(height, native):
    data = b"\0\1" + height.to_bytes(8, "big") + bytes.fromhex(native)
    return hashlib.sha256(b"".join(len(part).to_bytes(8, "big") + part
                                   for part in (b"PoC/DerivedAnchor/v1", data))).hexdigest()


def comparison(left, right):
    common_height = min(len(left), len(right)) - 1
    first_diff = next((index for index in range(common_height + 1)
                       if left[index]["native"] != right[index]["native"]), None)
    return {"common_height": common_height,
            "common_prefix": common_height if first_diff is None else first_diff - 1,
            "first_diff": first_diff}


def cut_anchor(record):
    return {"height": record["height"], "origin": record["origin"], "native": record["native"],
            "id": derived_id(record["height"], record["native"]), "post": record["post"]}


def selected_region(policy):
    mask, cycle = policy["mask"], policy["order"]
    require(uint(mask) and mask <= 7 and sorted(cycle) == [0, 1, 2], "invalid RESUME policy")
    return next((region for region in cycle if mask & (1 << region)), None)


class Replay:
    def __init__(self, profile):
        self.profile = profile
        for name in ("genesis_timestamp_us", "slot_us", "announcement_us", "expiry_us", "grace_us", "tb", "seed"):
            require(uint(profile[name]), f"invalid profile field {name}")
        require(profile["slot_us"] > 0 and profile["tb"] == 10, "invalid L2 slot/window size")
        genesis = profile["genesis"]
        require(genesis["height"] == 0 and genesis["kind"] == "genesis", "invalid genesis")
        self.check_block(genesis, 0)
        self.chains = {name: [genesis] for name in CHAINS}
        self.blocks = {genesis["native"]: genesis}
        self.duty = profile["initial_duty"]
        require(self.duty["branch"] == 0 and self.duty["r"] == 0 and self.duty["window"] == [1, 10]
                and self.duty["tail"] == "NEXT" and isinstance(self.duty["cid"], str),
                "invalid initial duty")
        self.pc = None
        self.closed = set()
        self.branch_cids = {self.duty["cid"]}
        self.headers = {}
        self.readable = set()
        self.reader_prefixes = {"A": self.readable}
        self.batch_submission = {}
        self.resume_submission = {}
        self.batch_occurrence = {}
        self.batches = {}
        self.pending = {}
        self.da = {}
        self.inbox_cursor = 0
        self.resumes = {}
        self.cuts = {}
        self.regional = {}
        self.espresso = {}
        self.views = {}
        self.waiting = None
        self.d_position = None
        self.gprime_position = None
        self.active_resume = None
        self.accepted_successor = False

    def timestamp(self, height):
        return self.profile["genesis_timestamp_us"] + height * self.profile["slot_us"]

    def check_block(self, block, height):
        require(uint(height) and block["height"] == height and uint(block["origin"]), "noncontiguous block height/origin")
        require(block["timestamp_us"] == self.timestamp(height), "L2 block timestamp mismatch")
        require(all(digest(block[field]) for field in ("native", "pre", "post")), "invalid native hash/state root")
        require(block["kind"] in {"genesis", "regional", "batch", "window-fallback", "cursor-fill", "substitution", "padding"},
                "unrecognized block kind")

    def append(self, chain, blocks):
        result = list(chain)
        for block in blocks:
            self.check_block(block, len(result))
            parent = result[-1]
            require(block["native_parent"] == parent["native"] and block["pre"] == parent["post"],
                    "native parent or execution pre-state mismatch")
            require(block["origin"] in {parent["origin"], parent["origin"] + 1}, "noncanonical origin progression")
            known = self.blocks.get(block["native"])
            if known is None:
                self.blocks[block["native"]] = block
            else:
                require(all(known[field] == block[field] for field in
                            ("height", "origin", "timestamp_us", "pre", "post", "native_parent")),
                        "one native hash has conflicting native data")
            result.append(block)
        return result

    def chain_view(self, chain):
        return [{key: block.get(key) for key in ("height", "native", "kind", "provenance", "pre", "post",
                                                "origin", "physical", "physical_parent")}
                for block in chain]

    def view(self):
        return {"chains": {name: self.chain_view(chain) for name, chain in self.chains.items()},
                "duty": self.duty, "pc": self.pc,
                "d_position": self.d_position,
                "comparison": comparison(self.chains["g"], self.chains["d"])}

    def finalized(self, height):
        require(height in self.headers, f"missing finalized L1 header ℓ{height}")
        return self.headers[height]

    def coverage(self, position, detail):
        key = position["key_us"]
        q = detail["coverage"]["q"]
        threshold = key - self.profile["announcement_us"]
        l1_height = detail["coverage"]["l1_height"]
        require(self.finalized(l1_height)["timestamp_us"] >= threshold
                and l1_height in self.readable, "L1 prefix does not cover public position")
        block = self.espresso.get(q)
        require(block is not None and block["theta_us"] >= key, "Espresso watermark below public key")
        if position["kind"] == "Espresso":
            require(position["index"] == [q] and block["theta_us"] == key,
                    "Espresso event key/q mismatch")
        else:
            require(q == min(i for i, row in self.espresso.items() if row["theta_us"] >= key),
                    "first covering Espresso prefix mismatch")

    def public(self, position):
        require(self.pc is None or order(self.pc) < order(position), "GFR public order regressed")
        self.pc = position
        self.views[order(position)] = self.chains["g"]
        self.waiting = None

    def batch_receipt(self, detail):
        position = detail["transaction_position"]
        require(isinstance(position, list) and len(position) == 2 and all(map(uint, position)), "invalid receipt position")
        require(detail["status"] in (0, 1) and detail["tx"] not in self.batch_occurrence,
                "ambiguous batch receipt")
        require(digest(detail["commitment"]), "invalid batch commitment")
        self.batch_occurrence[detail["tx"]] = (tuple(position), detail)

    def inbox_advanced(self, detail):
        position = l1pos(detail["position"])
        observed_position, receipt = self.batch_occurrence[detail["tx"]]
        require(position[:2] == observed_position and receipt["status"] == 1
                and receipt["commitment"] == detail["commitment"], "Inbox input requires successful receipt")
        require(position[0] in self.readable and self.finalized(position[0]) == receipt["header"],
                "Inbox input requires the receipt’s actual finalized header")
        previous, target = detail["previous"], detail["next"]
        require(uint(target) and self.inbox_cursor == previous < target
                and detail["range"] == [previous + 1, target] and detail["closed"],
                "invalid Inbox cursor advance")
        source = self.views[order(detail["gpos"])]
        require(len(source) > target, "Inbox range missing from certified source")
        require(position not in self.batches, "duplicate batch position")
        self.batches[position] = (detail, source[previous + 1:target + 1])
        self.pending[position] = detail
        self.inbox_cursor = target

    def verify_da(self, detail):
        position = l1pos(detail["position"])
        batch, _ = self.batches[position]
        require(batch["commitment"] == detail["commitment"]
                and detail["commitment"] == detail["verified_commitment"] and detail["valid"] is True,
                "DA commitment mismatch")
        self.da[position] = detail

    def derive(self, detail):
        position = l1pos(detail["position"])
        blocks = detail["blocks"]
        require(blocks, "empty D append")
        require(self.d_position is None or self.d_position <= position, "D position regressed")
        require(position[0] in self.readable, "D input requires finalized L1 delivery")
        if detail["reason"] == "batch":
            require(self.pending and min(self.pending) == position and position in self.da,
                    "accepted DA must be available in L1 order")
            batch = self.pending[position]
            require([block["height"] for block in blocks] == list(range(batch["previous"] + 1, batch["next"] + 1))
                    and all(block["kind"] in {"batch", "substitution"} for block in blocks),
                    "batch-derived interval/type mismatch")
            del self.pending[position]
        else:
            require(detail["reason"] == "window-fallback"
                    and all(block["kind"] == "window-fallback" for block in blocks),
                    "invalid D derivation reason")
            require(not any(p <= position for p in self.pending), "accepted DA is unresolved")
            deadline = detail["origin"] + detail["window"]
            require(self.profile["sequencing_window"] == detail["window"]
                    and detail["processed_through"] >= deadline
                    and position[0] == detail["processed_through"]
                    and detail["origin"] + 1 in self.readable, "fallback lacks a closed native window or next origin")
        self.chains["d"] = self.append(self.chains["d"], blocks)
        self.d_position = position

    def resume_included(self, detail):
        resume, cursor = l1pos(detail["position"]), l1pos(detail["cursor_position"])
        header = detail["header"]
        require(detail["status"] == 1 and resume[:2] == cursor[:2] and cursor[2] < resume[2]
                and header["height"] == resume[0], "RESUME/CursorSet require a single successful receipt")
        require(resume not in self.resumes and all(l1pos(item["cursor_position"]) != cursor
                                                 for item in self.resumes.values()), "duplicate RESUME/CursorSet position")
        require(uint(detail["target"]) and uint(detail["previous"]) and detail["previous"] < detail["target"]
                and digest(header["hash"]) and uint(header["timestamp_us"])
                and uint(detail["control_seq"]), "invalid RESUME inclusion")
        selected_region(detail["policy"])
        self.resumes[resume] = detail

    def capture_cut(self, detail):
        resume = l1pos(detail["position"])
        recorded = self.resumes[resume]
        cursor = l1pos(detail["cursor_position"])
        require(recorded["cursor_position"] == detail["cursor_position"] and resume not in self.cuts,
                "cut is not uniquely bound to the paired CursorSet")
        require(self.d_position is None or self.d_position <= cursor, "historical capture precedes D cursor")
        require(self.finalized(cursor[0]) == recorded["header"] and cursor[0] in self.readable,
                "paired CursorSet is not readable on actual finalized L1")
        require(not any(pos < cursor for pos in self.pending), "prior accepted DA blocks CursorSet")
        target = recorded["target"]
        require(detail["target"] == target and self.inbox_cursor == recorded["previous"], "cut target/cursor mismatch")
        blocks = detail["fill"]
        require(len(blocks) == max(0, target - (len(self.chains["d"]) - 1))
                and all(block["kind"] == "cursor-fill" for block in blocks), "invalid CursorSet fill")
        parent = self.chains["d"][-1]
        for block in blocks:
            next_origin = parent["origin"] + 1
            if next_origin <= cursor[0]:
                header = self.finalized(next_origin)
                require(next_origin in self.readable, "CursorSet requires readable next L1 origin")
                eligible = header["timestamp_us"] <= block["timestamp_us"]
            else:
                eligible = False
            require(block["origin"] == parent["origin"] + int(eligible), "CursorSet chose wrong D origin")
            parent = block
        d = self.append(self.chains["d"], blocks)
        require(detail["prefix"] == [block["native"] for block in d] and detail["anchor"] == cut_anchor(d[target])
                and digest(detail["cut_id"]) and all(saved["cut_id"] != detail["cut_id"] for saved in self.cuts.values()),
                "cut prefix/anchor/identity mismatch")
        self.chains["d"] = d
        self.d_position = cursor
        self.inbox_cursor = target
        self.cuts[resume] = {"target": target, "anchor": detail["anchor"], "cut_id": detail["cut_id"], "chain": d}

    def gprime(self, detail):
        position = l1pos(detail["position"])
        require(self.gprime_position is None or self.gprime_position < position, "G′ cursor did not advance")
        if detail["source"] == "batch":
            batch, source = self.batches[position]
            current = self.chains["gprime"]
            require(position in self.da and len(current) - 1 == batch["previous"], "G′ Inbox sources not yet available")
            adopted = self.append(current, [{**block, "provenance": "Inbox"} for block in source])
        else:
            cut = self.cuts[l1pos(detail["resume_position"])]
            require(detail["source"] == "cut" and detail["cut_id"] == cut["cut_id"]
                    and list(position) == self.resumes[l1pos(detail["resume_position"])]["cursor_position"],
                    "G′ needs the matching CursorSet and cut")
            adopted = [{**block, "provenance": "CursorSet"} for block in cut["chain"][:cut["target"] + 1]]
        require(detail["head"] == adopted[-1]["native"], "G′ materialization mismatch")
        self.chains["gprime"] = adopted
        self.gprime_position = position

    def apply_resume(self, detail):
        resume = l1pos(detail["resume_position"])
        recorded = self.resumes[resume]
        cut = self.cuts[resume]
        target, anchor = cut["target"], cut["anchor"]
        old = self.chains["g"]
        new = cut["chain"][:target + 1]
        delta = comparison(old, new)
        retained = delta["common_prefix"] + 1
        position = detail["position"]
        act = recorded["header"]["timestamp_us"] + self.profile["announcement_us"]
        require(position == {"kind": "Resume", "key_us": act, "index": list(resume)}
                and detail["cut_id"] == cut["cut_id"], "RESUME public key or selected cut mismatch")
        self.coverage(position, detail)
        require(self.waiting is None or self.waiting == position, "incomplete earlier GFR event")
        require(detail["before"] == [block["native"] for block in old]
                and detail["after"] == [block["native"] for block in new]
                and detail["retain_through"] == delta["common_prefix"]
                and detail["revoked"] == [block["native"] for block in old[retained:]]
                and detail["adopted"] == [block["native"] for block in new[retained:]],
                "RESUME must replace the entire suffix atomically")
        successor = detail["duty"]
        hmin, hmax = target + 1, target + self.profile["tb"]
        expiry, session_end = self.timestamp(hmin) + self.profile["expiry_us"], self.timestamp(hmax + 1) + self.profile["grace_us"]
        past = min(expiry, session_end) < act
        region = selected_region(recorded["policy"])
        tail = "HALT" if past else "PENDING" if region is None else "NEXT"
        require(successor["branch"] not in self.closed
                and successor["branch"] == self.duty["branch"] + 1 and successor["r"] == 0
                and successor["source_region"] is None and successor["window"] == [hmin, hmax]
                and successor["floor"] == successor["plan_height"] == anchor["origin"]
                and successor["ref"] == anchor["id"] and successor["tail"] == tail
                and successor["region"] == (region if tail == "NEXT" else None)
                and (isinstance(successor["cid"], str) and successor["cid"] != self.duty["cid"]
                     if tail == "NEXT" else successor["cid"] is None),
                "invalid post-RESUME responsibility")
        if tail != "HALT":
            self.finalized(anchor["origin"])
            require(anchor["origin"] in self.readable and detail["plan_header_hash"] == self.headers[anchor["origin"]]["hash"],
                    "Plan reads the actual finalized anchor origin")
        expected = ["BranchClosed", "Resumed"]
        if tail in ("NEXT", "HALT"):
            expected.append("SessionAssigned" if tail == "NEXT" else "SessionClosed")
        require(detail["lifecycle"] == expected and detail["closed_branch"] == self.duty["branch"]
                and set(detail["closed_cids"]) == self.branch_cids,
                "unbound RESUME lifecycle")
        self.chains["g"] = old[:retained] + [{**block, "provenance": "DER"} for block in new[retained:]]
        self.public(position)
        self.closed.add(detail["closed_branch"])
        self.duty = successor
        self.branch_cids = {successor["cid"]} if successor["cid"] else set()
        self.active_resume = {"anchor": anchor, "cut_id": cut["cut_id"], "position": position}
        self.accepted_successor = False
        return {"old": old[retained:], "new": self.chains["g"][retained:], "retained": delta["common_prefix"],
                "d_head": self.chains["d"][-1]["height"], "expiry_us": expiry, "session_end_us": session_end}

    def accept(self, detail):
        position = detail["position"]
        block = detail["block"]
        self.coverage(position, detail)
        require(self.waiting is None, "pending GFR event cannot be bypassed")
        require(position["kind"] == "Espresso" and detail["cid"] == self.duty["cid"]
                and self.duty["tail"] == "NEXT" and self.duty["branch"] not in self.closed,
                "regional acceptance requires selected open duty")
        hmin, hmax = self.duty["window"]
        require(hmin <= block["height"] <= hmax and block["kind"] == "regional", "physical block outside duty")
        require(digest(block["physical"]) and digest(block["physical_parent"]), "invalid physical identity")
        require(self.regional[block["physical"]] == (detail["cid"], block, detail["proof"]),
                "missing regional final material")
        q = position["index"][0]
        first = next((i for i in range(q + 1) if any(item["physical"] == block["physical"]
                     and item["proof"] == detail["proof"] for item in self.espresso[i]["objects"])), None)
        require(first == detail["q_avail"], "incorrect first complete public position")
        require(position["key_us"] < self.timestamp(block["height"]) + self.profile["expiry_us"]
                and self.espresso[first]["theta_us"] < self.timestamp(hmax + 1) + self.profile["grace_us"],
                "physical result expired or missed session deadline")
        if self.active_resume and not self.accepted_successor:
            anchor = self.active_resume["anchor"]
            require(block["height"] == anchor["height"] + 1 and block["physical_parent"] == anchor["id"]
                    and block["native_parent"] == anchor["native"] and block["pre"] == anchor["post"],
                    "recovery first block must extend the exact derived anchor and its state")
            require(order(position) > order(self.active_resume["position"]), "recovery acceptance precedes RESUME")
            self.accepted_successor = True
        self.chains["g"] = self.append(self.chains["g"], [block])
        self.public(position)
        if "handoff" in detail:
            duty = detail["handoff"]
            require(block["height"] == hmax and duty["branch"] == self.duty["branch"]
                    and duty["r"] == self.duty["r"] + 1 and duty["tail"] == "NEXT"
                    and duty["window"] == [hmax + 1, hmax + self.profile["tb"]]
                    and duty["ref"] == block["physical"] and duty["cid"] not in self.branch_cids,
                    "regular handoff must use the current full-window tail")
            self.duty = duty
            self.branch_cids.add(duty["cid"])

    def step(self, event):
        kind, detail = event["kind"], event["detail"]
        if kind == "l1_finalized":
            height = detail["height"]
            require(uint(height) and height == len(self.headers) and uint(detail["timestamp_us"])
                    and digest(detail["hash"]) and digest(detail["parent"]),
                    "invalid finalized L1 header")
            if height:
                parent = self.headers[height - 1]
                require(detail["parent"] == parent["hash"] and detail["timestamp_us"] > parent["timestamp_us"],
                        "L1 header prefix is not contiguous")
            require(event["tick_us"] >= detail["timestamp_us"], "L1 finalized before its timestamp")
            self.headers[height] = detail
        elif kind == "l1_available":
            height = detail["height"]
            readable = self.reader_prefixes.setdefault(detail["reader"], set())
            require(self.finalized(height)["hash"] == detail["hash"] and height == len(readable),
                    "reader L1 prefix is not contiguous")
            readable.add(height)
        elif kind == "batch_submitted":
            require(detail["tx"] not in self.batch_submission, "duplicate batch submission")
            self.batch_submission[detail["tx"]] = detail["commitment"]
        elif kind == "batch_receipt":
            require(self.batch_submission.get(detail["tx"]) == detail["commitment"], "unbound batch receipt")
            self.batch_receipt(detail)
        elif kind == "inbox_advanced":
            self.inbox_advanced(detail)
        elif kind == "da_wait":
            pending = l1pos(detail["position"])
            require(self.pending and pending == min(self.pending) and pending not in self.da
                    and detail["commitment"] == self.pending[pending]["commitment"],
                    "DA WAIT must preserve the earliest unresolved accepted input")
        elif kind == "da_verified":
            self.verify_da(detail)
        elif kind == "d_append":
            self.derive(detail)
        elif kind == "gprime_update":
            self.gprime(detail)
        elif kind == "compare":
            require(detail == comparison(self.chains["g"], self.chains["d"]), "recorded native comparison differs")
        elif kind == "resume_submitted":
            require(detail["tx"] not in self.resume_submission, "duplicate RESUME submission")
            self.resume_submission[detail["tx"]] = detail["target"]
        elif kind == "resume_included":
            require(self.resume_submission.get(detail["tx"]) == detail["target"], "unbound RESUME receipt")
            self.resume_included(detail)
        elif kind == "resume_cut":
            self.capture_cut(detail)
        elif kind == "espresso":
            q = detail["q"]
            require(q == len(self.espresso) and uint(detail["theta_us"])
                    and isinstance(detail["objects"], list), "invalid Espresso row")
            require(q == 0 or detail["theta_us"] >= self.espresso[q - 1]["theta_us"], "Espresso watermark regressed")
            self.espresso[q] = detail
        elif kind == "regional_final":
            block = detail["block"]
            self.check_block(block, block["height"])
            require(digest(block["physical"]) and digest(detail["proof"]) and block["physical"] not in self.regional,
                    "invalid or duplicate regional evidence")
            self.regional[block["physical"]] = (detail["cid"], block, detail["proof"])
        elif kind == "g_wait":
            position = detail["position"]
            control = self.resumes[l1pos(position["index"])]
            require(position["kind"] == "Resume"
                    and position["key_us"] == control["header"]["timestamp_us"] + self.profile["announcement_us"]
                    and (self.pc is None or order(self.pc) < order(position))
                    and self.waiting in (None, position) and detail["missing"], "WAIT crossed public position")
            self.waiting = position
        elif kind == "resume_applied":
            return self.apply_resume(detail)
        elif kind == "g_accepted":
            self.accept(detail)
        elif kind == "g_empty":
            position = detail["position"]
            require(self.waiting is None and position["kind"] == "Espresso",
                    "pending GFR event cannot be bypassed")
            self.coverage(position, detail)
            self.public(position)
        elif kind == "d_skipped":
            position = l1pos(detail["position"])
            require(self.pending and min(self.pending) == position and position in self.da,
                    "cannot skip unresolved accepted input")
            require(self.d_position is None or self.d_position <= position, "D cursor regressed")
            batch = self.pending[position]
            require((detail["reason"] == "already-derived" and batch["next"] < len(self.chains["d"]))
                    or (detail["reason"] == "late" and position[0] > detail["origin"] + self.profile["sequencing_window"]),
                    "unsupported D skip reason")
            del self.pending[position]
            self.d_position = position
        elif kind == "g_timer":
            position = detail["position"]
            require(self.waiting is None and position["kind"] in ("Expire", "SessionEnd"),
                    "invalid timer while a public event is pending")
            self.coverage(position, detail)
            expected = (self.timestamp(len(self.chains["g"])) + self.profile["expiry_us"]
                        if position["kind"] == "Expire"
                        else self.timestamp(self.duty["window"][1] + 1) + self.profile["grace_us"])
            expected_index = ([len(self.chains["g"])] if position["kind"] == "Expire"
                              else [self.duty["branch"], self.duty["r"]])
            require(position["key_us"] == expected and position["index"] == expected_index,
                    "timer does not apply to current state")
            require(position["kind"] == "Expire" and detail["tail"] == "HALT"
                    and self.duty["tail"] == "NEXT", "unsupported timer transition")
            self.public(position)
            self.closed.add(self.duty["branch"])
            self.duty = {**self.duty, "tail": "HALT", "cid": None, "region": None}
        return None


def project(tape, mode, provenance):
    require(tape["schema"] == SCHEMA, "unsupported normalized M4 input version")
    replay = Replay(tape["profile"])
    frames = []
    last_seq = -1
    last_tick = -1
    for event in tape["events"]:
        seq, tick = event["seq"], event["tick_us"]
        require(uint(seq) and uint(tick) and seq > last_seq and tick >= last_tick, "nonmonotonic trace order")
        require(event["kind"] in STAGES and isinstance(event["refs"], list) and event["refs"], "invalid event/source")
        require(isinstance(event["detail"], dict), "invalid event detail")
        last_seq, last_tick = seq, tick
        lane, label = STAGES[event["kind"]]
        change = replay.step(event)
        frames.append({**event, "lane": lane, "label": label, "change": change, **replay.view()})
    require(frames, "empty M4 replay")
    if mode == "sealed":
        require(replay.accepted_successor, "sealed main scenario lacks recovered G admission")
    return {"mode": mode, "provenance": provenance, "frames": frames, "profile": deepcopy(tape["profile"])}
