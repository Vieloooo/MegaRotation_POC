"""Fold the evidenced M5 observations using the M4 chain and Cut checks."""

from copy import deepcopy
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path

PATH = Path(__file__).resolve().parent.parent / "m4-resume-replay/model.py"
if not PATH.exists():
    PATH = Path(__file__).resolve().parents[1] / "m4-model.py"
spec = spec_from_file_location("m5_m4_replay_model", PATH)
m4 = module_from_spec(spec)
spec.loader.exec_module(m4)

SCHEMA = "M5-AUTO-RESUME-REPLAY-1"
U64_MAX = (1 << 64) - 1
STAGES = {
    **m4.STAGES,
    "ordering_fault": ("espresso", "Espresso 故障"),
    "ordering_recovered": ("espresso", "排序服务恢复"),
    "g_ordering_wait": ("g", "公共覆盖 WAIT"),
    "local_stop": ("regional", "本地 stop"),
    "signer_view": ("governance", "Signer 独立观察"),
    "resume_signed": ("governance", "治理签名包"),
    "resume_prepared": ("governance", "持久准备"),
}
require, uint = m4.require, m4.uint


def target(profile, detail):
    """Check the inclusive timing window on the integer L2 height grid."""
    heads = detail["heads"]
    require(set(heads) == {"g", "d", "inbox"} and all(map(uint, heads.values())),
            "invalid observed recovery heads")
    require(bool(detail["growth_basis"]), "missing growth estimate provenance")
    low, high = detail["inclusion"]
    require(uint(low) and uint(high) and low <= high, "invalid predicted L1 inclusion interval")
    prep, drift, epsilon = (profile[key] for key in ("prep_us", "drift_us", "epsilon_us"))
    genesis, slot = profile["genesis_timestamp_us"], profile["slot_us"]
    l1_slot = profile["l1_slot_us"]
    require(uint(l1_slot) and l1_slot > 0 and high % l1_slot == 0 and low % l1_slot == 0,
            "inclusion endpoints must identify L1 slots")
    b_hi = high // l1_slot
    e_hi = max(0, b_hi - profile["sequencing_window"] + 1)
    h_time = max(0, -((genesis - e_hi * l1_slot) // slot))
    require(uint(detail["d_origin"]) and uint(detail["inbox_growth"]), "invalid D/Inbox advancement input")
    extra = max(0, e_hi - detail["d_origin"]) + 1
    h_inbox = heads["inbox"] + detail["inbox_growth"]
    h_d = max(heads["d"], h_inbox, h_time) + extra
    h_g = (heads["g"] if detail["tail"] == "HALT" else
           max(heads["g"], (high + profile["clock_skew_us"] - genesis) // slot))
    parts = {"b_hi": b_hi, "E_hi": e_hi, "h_time": h_time, "extra_origin": extra,
             "H_D": h_d, "H_G": h_g, "H_Inbox": h_inbox}
    guard = max(h_g, h_d, h_inbox)
    require(detail["guard_parts"] == parts and detail["h_guard"] == guard,
            "H_guard components do not match the frozen advancement envelope")
    require(detail["signed_origin"] == low // l1_slot, "signed origin is not the predicted interval lower L1 slot")
    left, right = high + prep, low + drift - epsilon
    require(high + prep <= U64_MAX and low + drift <= U64_MAX and right >= 0,
            "RESUME timing arithmetic outside the supported range")
    first = max(guard + 1, -((genesis - left) // slot))
    last = (right - genesis) // slot
    ready = (prep >= profile["announcement_us"] and
             profile["announcement_us"] + profile["ready_us"] + profile["clock_skew_us"] <= prep + slot)
    expected = first if ready and first <= last else None
    require(detail["target"] == expected, "target is not the smallest feasible L2 height")
    return {"lower_timestamp_us": left, "upper_timestamp_us": right,
            "minimum_height": first, "maximum_height": last, "timing_ready": ready,
            "target_timestamp_us": genesis + expected * slot if expected is not None else None}


def tip(value, rows):
    if value is None:
        return
    q, theta = value["q"], value["theta_us"]
    require(uint(q) and q in rows and theta == rows[q]["theta_us"], "signer tip differs from public Espresso")


class Replay(m4.Replay):
    def __init__(self, profile):
        super().__init__(profile)
        for key in ("prep_us", "drift_us", "epsilon_us", "ready_us", "clock_skew_us",
                    "poll_us", "sign_us", "broadcast_us", "threshold"):
            require(uint(profile[key]), f"missing timing or signer profile {key}")
        members = profile["members"]
        require(1 <= profile["threshold"] <= len(members) and len(set(members)) == len(members),
                "invalid governance committee")
        self.ordering = "live"
        self.recovered = False
        self.fault_tip = None
        self.ordering_wait = None
        self.local_stops = set()
        self.signers = {}
        self.eligible = {}
        self.quorum = {}
        self.signed = {}
        self.prepared = {}
        self.submitted = {}
        self.halt_pc = None

    def public(self, position):
        if self.ordering_wait is not None:
            # The waiter does not suppress earlier or equal-watermark completed records.
            if m4.order(position) >= m4.order(self.ordering_wait):
                require(position == self.ordering_wait,
                        "public replay crossed an unapplied pending event")
                self.ordering_wait = None
        super().public(position)

    def view(self):
        return {**super().view(), "service": self.ordering, "blocked": self.ordering_wait or self.waiting,
                "watermark": deepcopy(self.espresso.get(len(self.espresso) - 1)),
                "signer_views": deepcopy(self.signers), "stopped": sorted(self.local_stops)}

    def decision(self, event):
        d = event["detail"]
        who = d["signer"]
        require(uint(who) and who < len(self.profile["members"]), "signer is outside the recorded committee")
        for key in ("published", "verified", "previous"):
            tip(d[key], self.espresso)
        published, verified, previous = (d[k] for k in ("published", "verified", "previous"))
        available = bool(verified and published and verified["q"] <= published["q"])
        complete = available and verified == published
        advancing = bool(available and previous and verified["q"] > previous["q"]
                         and verified["theta_us"] > previous["theta_us"])
        l1p, l1v = d["published_l1"], d["verified_l1"]
        for height in (l1p, l1v):
            require(height is None or (uint(height) and height in self.headers), "unobserved signer L1 tip")
        require(l1v is None or (l1p is not None and l1v <= l1p), "signer L1 verification exceeds published prefix")
        local = d["pc"]
        require(local is None or (self.pc is not None and m4.order(local) <= m4.order(self.pc)),
                "signer public position precedes availability")
        if local is not None:
            require(verified is not None and verified["theta_us"] >= local["key_us"],
                    "signer pc exceeds verified watermark")
        heads = d["heads"]
        require(all(key in heads and uint(heads[key]) for key in ("g", "d", "inbox")) and
                heads["g"] < len(self.chains["g"]) and heads["d"] < len(self.chains["d"])
                and heads["inbox"] <= self.inbox_cursor, "signer observation exceeds a known chain head")
        solution = target(self.profile, d) if d.get("inclusion") is not None else None
        reason = d["reason"]
        frozen = reason == "HALTED_D_ADVANCED" and d["tail"] == "HALT" and heads["d"] > heads["g"]
        if frozen:
            require(self.halt_pc is not None and local is not None and m4.order(local) >= m4.order(self.halt_pc),
                    "signer has no completed public HALT")
        difference = False
        if reason == "HASH_DIVERGENCE":
            h = d["difference_height"]
            require(uint(h) and h <= min(heads["g"], heads["d"]), "signer divergence must use a common height")
            g, derived = (self.chains[name][h]["native"] for name in ("g", "d"))
            difference = d["g_native"] == g and d["d_native"] == derived and g != derived
        key = d.get("request")
        if d["eligible"]:
            payload = d["payload"]
            require(payload["target"] == d["target"] and payload["signed_origin"] == d["signed_origin"]
                    and payload["branch"] == self.duty["branch"] and uint(payload["control_seq"]),
                    "signer intent does not match the solving inputs")
            require(heads["g"] == len(self.views[m4.order(local)]) - 1,
                    "signer G head differs from its completed public position")
            require(d["d_origin"] == self.chains["d"][heads["d"]]["origin"],
                    "solver origin differs from the observed D head")
            require(key and m4.digest(key) and self.ordering in ("live", "recovering") and self.recovered
                    and advancing and complete and l1v is not None and l1v == l1p
                    and local == {"kind": "Espresso", "index": [verified["q"]], "key_us": verified["theta_us"]}
                    and d["d_drained"] and d["clock_ready"] and d["paths_ready"]
                    and not d["pending"] and solution and d["target"] is not None and (frozen or difference),
                    "signer eligible without recovered ordering, material or a feasible trigger")
            observed = d.get("qualified_us", event["tick_us"])
            require(uint(observed) and observed <= event["tick_us"], "eligibility observation outside recorded inputs")
            request_key = key, d["target"]
            prior = self.eligible.get(who)
            if prior is None or prior[0] != request_key:
                self.eligible[who] = request_key, observed
            times = sorted(time for request, time in self.eligible.values() if request == request_key)
            if len(times) >= self.profile["threshold"]:
                self.quorum.setdefault(request_key, times[self.profile["threshold"] - 1])
        else:
            self.eligible.pop(who, None)
        self.signers[who] = {**d, "solver": solution, "tick_us": event["tick_us"], "seq": event["seq"], "suborder": event.get("suborder", 0),
                             "q_advanced": bool(previous and verified and verified["q"] > previous["q"]),
                             "theta_advanced": advancing, "refs": event["refs"]}

    def signature_pack(self, event):
        d = event["detail"]
        request, target_height, members = d["request"], d["target"], d["members"]
        key = request, target_height
        evidence = {who: time for who, (current, time) in self.eligible.items() if current == key}
        count = self.profile["threshold"]
        require(m4.digest(request) and request not in self.signed and len(evidence) >= count,
                "signature pack lacks a same-request eligible quorum")
        signers = [member["signer"] for member in members]
        addresses = [member["address"] for member in members]
        require(len(members) == count and len(set(signers)) == count
                and addresses == sorted(set(addresses)), "signature pack membership/order mismatch")
        require(all(self.signers[who]["inclusion"] == d["inclusion"]
                    and self.signers[who]["payload"] == d["payload"]
                    for who in signers if who in evidence), "signers used different inclusion interval")
        t_eligible = self.quorum[key]
        require(d["t_eligible_us"] == t_eligible, "incorrect first eligible quorum observation")
        for member in members:
            who = member["signer"]
            require(who in evidence and member["address"] == self.profile["members"][who]
                    and member["digest"] == request and m4.digest(member["signature_ref"])
                    and evidence[who] <= member["signed_us"] <= event["tick_us"],
                    "signature is not bound to eligible signer and request")
        self.signed[request] = {**d, "tick_us": event["tick_us"]}

    def step(self, event):
        kind, d, tick = event["kind"], event["detail"], event["tick_us"]
        if kind == "espresso":
            q, theta = d["q"], d["theta_us"]
            require(uint(d["proposal_tick_us"]) and d["proposal_tick_us"] <= tick
                    and d["finalized_tick_us"] == tick, "invalid Espresso proposal/finalized times")
            require(self.ordering != "stopped", "Espresso finalized during stopped interval")
            if self.ordering == "frozen":
                require(theta == self.fault_tip["theta_us"] and not d["objects"],
                        "frozen interval only finalizes empty heartbeat at the frozen key")
            super().step(event)
            if self.ordering == "recovering" and q > self.fault_tip["q"] and theta > self.fault_tip["theta_us"]:
                self.recovered = True
        elif kind == "ordering_fault":
            require(d["mode"] in ("stopped", "frozen") and self.ordering == "live" and self.espresso,
                    "invalid ordering outage")
            self.fault_tip = {key: self.espresso[len(self.espresso) - 1][key] for key in ("q", "theta_us")}
            require(d["last"] == self.fault_tip, "outage starts from a different public prefix")
            self.ordering = d["mode"]
            self.recovered = False
        elif kind == "ordering_recovered":
            require(self.ordering in ("stopped", "frozen"), "ordering resumed without an outage")
            self.ordering = "recovering"
        elif kind == "g_ordering_wait":
            position = d["position"]
            q = d["validated_q"]
            tip({"q": q, "theta_us": d["theta_us"]}, self.espresso)
            if d.get("reader") == "A" and self.pc and self.pc["kind"] == "Espresso":
                require(q >= self.pc["index"][0], "WAIT precedes this reader's completed Espresso prefix")
            require(m4.order(position) > (m4.order(self.pc) if self.pc else (-1, -1, ())) and
                    self.ordering_wait in (None, position) and d["missing"], "invalid pending WAIT")
            if d["missing"] == "ESPRESSO_WATERMARK":
                require(d["theta_us"] < position["key_us"], "WAIT requires missing local watermark")
            self.ordering_wait = position
        elif kind == "local_stop":
            cid, worker = d["cid"], d["worker"]
            require(cid in self.branch_cids and uint(worker) and (cid, worker) not in self.local_stops
                    and d["stop_us"] == self.timestamp(d["window_end"] + 1)
                    and d["clock_us"] >= d["stop_us"] and abs(d["clock_us"] - tick) <= self.profile["clock_skew_us"]
                    and bool(d["terminal"]),
                    "invalid local stop/signing gate")
            self.local_stops.add((cid, worker))
        elif kind == "g_timer":
            if d["position"]["kind"] == "SessionEnd":
                self.session_end(d)
            else:
                super().step(event)
                self.halt_pc = self.pc
        elif kind == "signer_view":
            self.decision(event)
        elif kind == "resume_signed":
            self.signature_pack(event)
        elif kind == "resume_prepared":
            request = d["request"]
            require(request in self.signed and request not in self.prepared and bool(d["durable"])
                    and d["target"] == self.signed[request]["target"], "prepared request differs from signed intent")
            self.prepared[request] = (d["tx"], d["target"])
        elif kind == "resume_submitted":
            request = d["request"]
            require(self.prepared.get(request) == (d["tx"], d["target"]) and request not in self.submitted,
                    "broadcast differs from durable intent")
            delay = tick - self.signed[request]["t_eligible_us"]
            budget = sum(self.profile[name] for name in ("poll_us", "sign_us", "broadcast_us"))
            require(delay >= 0 and d["submit_delay_us"] == delay and d["within_budget"] == (delay <= budget),
                    "incorrect submission timing")
            require(d["inclusion"] == self.signed[request]["inclusion"], "broadcast changed signed timing inputs")
            self.submitted[request] = d
            super().step(event)
        elif kind == "resume_included":
            submitted = self.submitted[d["request"]]
            observed_timestamp = d["header"]["timestamp_us"]
            low, high = submitted["inclusion"]
            payload = self.signed[d["request"]]["payload"]
            require(d["control_seq"] == payload["control_seq"] and d["policy"] == payload["policy"],
                    "included RESUME differs from signed authority fields")
            require(d["tx"] == submitted["tx"] and d["target"] == submitted["target"]
                    and d["signed_origin"] == low // self.profile["l1_slot_us"]
                    and d["act_us"] == observed_timestamp + self.profile["announcement_us"]
                    and d["within_inclusion"] == (low <= observed_timestamp <= high),
                    "included request or actual inclusion timing mismatch")
            super().step(event)
        else:
            return super().step(event)
        return None

    def session_end(self, detail):
        position = detail["position"]
        old, successor = self.duty, detail["duty"]
        require(old["tail"] == "NEXT" and position["index"] == [old["branch"], old["r"]]
                and position["key_us"] == self.timestamp(old["window"][1] + 1) + self.profile["grace_us"],
                "public SessionEnd does not apply to current duty")
        self.coverage(position, detail)
        require(successor["branch"] == old["branch"] and successor["r"] == old["r"] + 1
                and successor["window"] == [old["window"][1] + 1, old["window"][1] + self.profile["tb"]],
                "invalid SessionEnd successor")
        self.public(position)
        self.duty = successor
        if successor["cid"] is not None:
            self.branch_cids.add(successor["cid"])


def project(tape, mode, provenance):
    require(tape["schema"] == SCHEMA, "unsupported M5 projection version")
    replay = Replay(tape["profile"])
    frames = []
    prev, prev_tick = (-1, -1), -1
    for event in tape["events"]:
        seq, suborder, tick = event["seq"], event.get("suborder", 0), event["tick_us"]
        require(all(map(uint, (seq, suborder, tick))) and (seq, suborder) > prev and tick >= prev_tick
                and (seq != prev[0] or tick == prev_tick), "nonmonotonic source observation")
        require(event["refs"] and isinstance(event["detail"], dict), "missing source or detail")
        prev, prev_tick = (seq, suborder), tick
        lane, label = STAGES[event["kind"]]
        change = replay.step(event)
        frames.append({**event, "lane": lane, "label": label, "change": change, **replay.view()})
    require(frames, "empty M5 replay")
    if tape.get("expect_resume"):
        require(replay.accepted_successor, "successful recovery lacks first G acceptance")
    return {"mode": mode, "provenance": provenance, "frames": frames, "profile": tape["profile"],
            "label": tape["label"]}
