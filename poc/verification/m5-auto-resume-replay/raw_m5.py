"""Read the M5 logger and extend the sealed M4 chain/receipt projection."""

import json
import sys
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path

from model import SCHEMA, m4, require, uint
from sealed import sha

M4 = Path(__file__).resolve().parents[1] / "m4-resume-replay"


def load_m4():
    """Load the existing raw binder without shadowing this directory's reducer."""
    saved = {key: sys.modules.get(key) for key in ("model", "evidence")}
    original_path = sys.path[:]
    try:
        sys.modules["model"] = m4
        sys.path.insert(0, str(M4))
        for name, path in (("evidence", "evidence.py"), ("_m5_m4_raw", "raw_m4.py")):
            spec = spec_from_file_location(name, M4 / path)
            module = module_from_spec(spec)
            sys.modules[name] = module
            spec.loader.exec_module(module)
        return module
    finally:
        sys.path[:] = original_path
        for key, value in saved.items():
            if value is None:
                sys.modules.pop(key, None)
            else:
                sys.modules[key] = value


raw = load_m4()
hx, source, trace_ref, one = raw.hx, raw.source, raw.trace_ref, raw.one
old_pc = raw.pc


def pc(value):
    if value is None:
        return None
    kind = value["kind"]
    if kind in ("Expire", "SessionEnd"):
        index = value["index"]
        return {"kind":kind, "key_us":value["key_us"],
                "index":[index["Height"]] if kind == "Expire"
                         else [index["Session"]["beta"], index["Session"]["r"]]}
    return old_pc(value)


raw.pc = pc
def espresso(data, path):
    """Decode M5 finalized rows including non-live frozen heartbeats."""
    reader = raw.Reader(data, path, "little")
    reader.expect(b"PELG\x01\x00")
    log_source = reader.frame(256).decode("ascii")
    q, key, proposed = reader.uint(8), reader.uint(8), reader.uint(8)
    live = reader.uint(1)
    reader.check(live in (0, 1), "invalid proposal liveness flag")
    finalized = reader.uint(8)
    reader.check(bool(log_source) and proposed <= finalized, "invalid public row")
    bodies, proofs = {}, []
    for ordinal in range(reader.count(1024)):
        reader.check(reader.uint(4) == 0x47465201, "invalid GFRP codec")
        obj = raw.Reader(reader.frame(65_536), f"{path} item {ordinal}")
        obj.expect(b"GFRP\0\1")
        tag = obj.uint(1)
        if tag == 0:
            block = raw.decode_physical(obj.frame(65_536), path)
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
    require(live or not (bodies or proofs), "non-live frozen heartbeat carries publication")
    return reader.finish(dict(live=bool(live), q=q, theta_us=key, proposal_us=proposed, finalized_us=finalized, source=log_source,
                              bodies=bodies, proofs=proofs, path=path))


raw.espresso = espresso
LABEL = {"stopped": "AR01 · finalized 停止后恢复", "frozen": "AR03 · q 增长、θ 冻结后恢复",
         "slow-rpc": "AR09 · 提高 RPC 延迟", "late": "AR08 · 实际迟收录",
         "short": "对照 · 短停", "no-target": "对照 · 无整数目标",
         "inputs-wait": "对照 · 必需输入未就绪", "divergence": "共同高度分歧后恢复"}


def key(request):
    """Stable key for matching logger records, independent of map member order."""
    return json.dumps(request, sort_keys=True, separators=(",", ":"))


def payload(request):
    fields = request["resume"]
    return {"target": fields["target"], "signed_origin": fields["origin"], "branch": request["target_beta"],
            "control_seq": request["control_seq"], "request_id": request["request_id"],
            "signed_floor": request["signed_floor"], "policy": {
                "mask": request["enabled_mask"], "order": request["order"]}}


def frame_records(payload_bytes):
    reader = raw.Reader(payload_bytes, "closed DA frame")
    reader.expect(b"P0B1\x00\x01")
    reader.take(16)
    require(reader.uint(2) == 0, "M5 expected one closed frame")
    data = reader.frame(47_146)
    require(reader.uint(1) == 1, "M5 DA frame is not closed")
    reader.finish(None)
    body = raw.Reader(data, "DA frame records")
    n = body.uint(2)
    require(n in (1, 2), "unexpected DA record count")
    records = []
    for _ in range(n):
        h, origin = body.uint(8), body.uint(8)
        txs = [body.frame(19) for _ in range(body.count(1024))]
        require(all(len(tx) == 19 and tx[0] == 1 for tx in txs), "invalid ordinary transaction")
        records.append({"height":h,"origin":origin,"transactions":[tx.hex() for tx in txs]})
    return body.finish(records)


def bindings(sealed, manifest, report):
    """Join the independently validated receipt inventory to sealed originals."""
    if all(field in report for field in ("committee", "batches", "resumes", "l1", "seed")):
        details = report
    else:
        rows = raw.open_rows(sealed, raw.TRACE)
        rpc = json.loads(sealed.read(raw.RPC))
        stages = {"mine_us":{},"finalized_us":{},"deliver_us":{}, "submit_us":{}}
        for row in rows:
            kind, d = row["kind"], row["detail"]
            if kind == "L1Included":
                stages["mine_us"][str(d["height"])] = row["tick_us"]
            elif kind == "L1Finalized":
                stages["finalized_us"][str(d["height"])] = row["tick_us"]
            elif kind == "L1ReaderAvailable":
                stages["deliver_us"].setdefault(d["reader"],{})[str(d["height"])] = row["tick_us"]
            elif kind == "BatchSubmitted":
                stages["submit_us"][d["hash"].lower()] = row["tick_us"]
        by_tx = {receipt["transactionHash"].lower(): receipt for receipt in rpc["receipts"]}
        batches = []
        previous = 0
        resume = []
        for event in report["receipt"]["events"]:
            kind = event["kind"]
            if kind not in {"CursorAdvanced", "ResumeRecorded"}:
                continue
            block, tx, ordinal = pos = event["position"]
            header = rpc["blocks"][block]
            tx_hash = header["transactions"][tx]["hash"].lower()
            receipt = by_tx[tx_hash]
            require(int(receipt["status"],16) == 1 and receipt["blockHash"] == header["hash"],
                    "checked log does not occur in its actual success receipt")
            log_index = int(receipt["logs"][ordinal]["logIndex"],16)
            if kind == "CursorAdvanced":
                commitment = event["commitment"].lower()
                da = sealed.read(f"raw/derivation/da/{commitment}.bin")
                items = frame_records(da)
                cursor = event["cursor"]
                require(items[0]["height"] == previous + 1 and items[-1]["height"] == cursor[0]
                        and items[-1]["origin"] == cursor[1], "checked Inbox range and DA payload differ")
                batches.append({"block_hash": header["hash"].lower(),"commitment":commitment,
                    "gpos":cursor[4].removeprefix("0x"), "log_index":log_index,"payload_sha256":sha(da),
                    "position":pos,"previous_height":previous,"records":items,"tx_hash":tx_hash})
                previous = cursor[0]
            else:
                prepared = one((row["detail"] for row in rows if row["kind"] == "GovernanceSubmission"
                                and row["detail"]["hash"].lower() == tx_hash), "checked RESUME prepared bytes")
                request = payload(prepared["request"])
                set_pos = event["setPosition"]
                companion = one((row for row in report["receipt"]["events"] if
                                 row["kind"]=="CursorSet" and row["position"]==set_pos), "checked exact CursorSet")
                require(pos[:2] == set_pos[:2] and companion["resumePosition"] == pos
                        and companion["cursor"][0] == request["target"] == report["governance"]["target"]
                        and request["control_seq"] == event["seq"], "actual receipt differs from checked intent")
                resume.append({"position":pos,"set_position":set_pos,"tx_hash":tx_hash,
                    "block_hash":header["hash"].lower(),"resume_log_index":log_index,
                    "set_log_index":int(receipt["logs"][set_pos[2]]["logIndex"],16),
                    "target":request["target"],"seq":request["control_seq"],"signed_origin":request["signed_origin"],
                    "signed_floor":request["signed_floor"],"policy":{"enabled_mask":request["policy"]["mask"],
                    "order":request["policy"]["order"]},"act_us":event["act"]*1_000_000,
                    "signers":sorted(entry.get("member",entry.get("address")).lower() for entry in prepared["signers"])})
        details = {**report, "committee": {"members":rpc["config"]["committee"],
                                           "threshold":rpc["config"]["threshold"]},
                   "batches":batches,"resumes":resume,"l1":stages,"seed":manifest["seed"]}
    protocol = details["protocol"]
    return {**details, "protocol":{**protocol,"restored_height": protocol["new_height"],
                                    "restored_native":protocol["new_native_hash"],
                                    "act_us":protocol["actual_act_us"]}}


class Mapper(raw.Mapper):
    def __init__(self, sealed, manifest, report, checker):
        report = bindings(sealed, manifest, report)
        raw_profile = json.loads(sealed.read("raw/profile.json"))
        auto = raw_profile["auto_resume"]
        mode = auto["scenario"]
        require(raw_profile["schema"] == "M5-AUTO-RESUME-1" and mode == manifest["scenario"] == report["mode"],
                "sealed M5 run and raw profile differ")
        self.mode = mode
        self.public_rows = checker["public"]["espresso"]
        compat = {**checker["public"], "espresso": [
            {k: value[k] for k in ("q", "theta_us", "proposed_us", "finalized_us")}
            for value in self.public_rows]}
        super().__init__(sealed, manifest, report, {
            **checker, "public": compat, "d": checker["d"]["A"],
        })
        self.checked_signatures = report.get("governance", {})
        endpoint = report["protocol"]
        require(report["seed"] == raw_profile["seed"] == manifest["seed"]
                and endpoint["restored_height"] == self.resume["target"] + 1, "wrong M5 checker endpoint")
        self.original = report
        self.profile.update(
            prep_us=auto["t_prep_us"], ready_us=auto["t_ready_us"], epsilon_us=auto["epsilon_us"],
            drift_us=self.config["t_drift_us"], clock_skew_us=1000, l1_slot_us=self.config["l1_block_us"],
            poll_us=auto["poll_us"], sign_us=auto["sign_us"], broadcast_us=auto["broadcast_us"],
            threshold=auto["threshold"])
        self.g_position = None
        self.latest_q = None
        self.last_tip = {}
        self.motion = {}
        self.requests = {}
        self.members = {}
        for row in self.kinds["GovernanceSignature"]:
            detail = row["detail"]
            who = detail["signer"]
            address = detail.get("address", detail.get("member")).lower()
            require(self.members.setdefault(who, address) == address, "one signer has differing addresses")
            intent = key(detail["request"])
            digest = hx(detail["digest"])
            require(self.requests.setdefault(intent, digest) == digest, "request has differing digests")
        committee = report["committee"]
        member_list = [str(address).lower() for address in committee["members"]]
        # The module decodes actual signer-index identity from signer evidence.
        remaining = [address for address in member_list if address not in self.members.values()]
        for who in range(len(member_list)):
            if who not in self.members:
                self.members[who] = remaining.pop(0)
        require(sorted(self.members.values()) == sorted(member_list), "observed signers differ from checked committee")
        self.profile["members"] = [self.members[index] for index in range(len(member_list))]
        self.intent_rows = {}
        self.signer_inputs = {}
        self.intent = None
        self.prepared = None
        self.sent = False
        self.active_before = None
        self.old_duty = self.profile["initial_duty"]
        # A/B expose what was available. Signer observation keeps its own D table.
        self.derived_views = checker["d"]
        self.public_history = {}
        self.trigger_observations = []
        self.marked_stages = set()

    def emit_public(self, row):
        start = len(self.events)
        super().emit_public(row)
        result = self.events[start]
        q = result["detail"]["q"]
        self.latest_q = q
        result["detail"]["live_at_proposal"] = self.pelg[q]["live"]
        if self.active_before == "frozen" and "frozen-q" not in self.marked_stages:
            self.marked_stages.add("frozen-q")
            result["stage"] = "frozen-q"
        if row["kind"] == "PublisherRecorded" and row["detail"]["height"] == self.resume["target"]+1:
            result["stage"] = "publish"

    def l1_included(self, row):
        oldest = len(self.events)
        super().l1_included(row)
        for event in self.events[oldest:]:
            if event["kind"] == "resume_included":
                detail = event["detail"]
                require(self.prepared and self.sent and detail["tx"] == self.prepared["tx"],
                        "successful receipt has no durable actual broadcast")
                detail["request"] = self.prepared["request"]
                low, high = self.prepared["inclusion"]
                detail["within_inclusion"] = low <= detail["header"]["timestamp_us"] <= high
                event["stage"] = "included"

    def g_record(self, row):
        detail = row["detail"]
        position = pc(detail["position"])
        if position["kind"] in ("Expire", "SessionEnd"):
            checked = one((timer for timer in self.checked_g["timers"]
                           if timer["kind"] == position["kind"] and timer["key_us"] == position["key_us"]),
                          "timer independently reproduced")
            refs = [source(self.pelg[checked["q_star"]]["path"], "首个覆盖 θ", "Espresso finalized")]
            require(all(pc(event["position"]) == position for event in detail["events"]), "timer lifecycle has foreign position")
            if position["kind"] == "Expire":
                [closed] = detail["events"]
                require(closed["body"]["BranchClosed"] == {"branch": checked["branch"], "reason": "Expired"}
                        and position["index"] == [checked["height"]], "Expire position/branch disagreement")
                self.emit(row, "g_timer", {
                    "position":position, "coverage":self.coverage(position), "tail":"HALT",
                    "lifecycle":detail["events"], "closed_cids":sorted(self.branch_cids),
                    "session_end_key_us":self.profile["genesis_timestamp_us"]
                        + (self.old_duty["window"][1] + 1) * self.profile["slot_us"] + self.profile["grace_us"],
                }, *refs, stage="expire")
            else:
                duty = raw.descriptor_duty(detail["events"][-1]["body"]["SessionAssigned"]["descriptor"])
                self.branch_cids.add(duty["cid"])
                self.old_duty = duty
                self.emit(row, "g_timer", {"position":position, "coverage":self.coverage(position),
                                           "duty":duty, "lifecycle":detail["events"]}, *refs)
        else:
            super().g_record(row)
            event = self.events[-1]
            if event["detail"].get("handoff"):
                self.old_duty = event["detail"]["handoff"]
        self.g_position = position
        self.public_history[m4.order(position)] = len(self.old_heads) - 1

    def signer(self, row):
        d = row["detail"]
        i = d["signer"]
        previous, verified, published = d["previous"], d["espresso_verified"], d["espresso_advertised"]
        earlier = self.last_tip.get(i)
        motion_source = None
        if verified and previous and previous["q"] < verified["q"] and previous["theta_us"] < verified["theta_us"]:
            if d.get("ordering_live") and earlier is not None and earlier["detail"]["espresso_verified"] == previous:
                self.motion[i] = (verified, previous, earlier)
        if d.get("ordering_live") is False:
            self.motion.pop(i, None)
        if i in self.motion and self.motion[i][0] == verified:
            _, motion_tip, motion_source = self.motion[i]
        else:
            motion_tip = None
        if earlier is None or earlier["detail"]["espresso_verified"] != verified:
            complete = (verified and verified == published and d["replay_position"]
                        and pc(d["replay_position"]) == {"kind":"Espresso", "index":[verified["q"]],
                                                        "key_us":verified["theta_us"]})
            if complete and "ordering_live" in d:
                self.last_tip[i] = row
        state = d.get("d")
        position = pc(d["replay_position"]) if d["replay_position"] else None
        known_g = self.public_history.get(m4.order(position)) if position else None
        heads = {"g": d.get("g_head", known_g), "d":state["d_height"] if state else None,
                 "inbox":state["inbox_height"] if state else None}
        local_d_verified = False
        if state:
            local_input = self.signer_inputs.get(("l1",i,state["l1_height"]))
            checked = self.derived_views[f"S{i}"]
            height = state["d_height"]
            hashes = [self.genesis["native"]] + [block["native_hash"] for block in checked["blocks"][:height]]
            require(local_input and local_input["detail"]["d"] == state
                    and [hx(native) for native in state["hashes"]] == hashes
                    and checked["at_end"][str(state["l1_height"])] == height,
                    "signer independent D/Inbox view differs from its checked delivered source")
            local_d_verified = True
        reason, eligible = d["reason"], d["eligible"]
        result = {"signer":i, "published":published, "verified":verified, "previous":previous,
                  "motion_basis": motion_tip, "ordering_live":d.get("ordering_live"),
                  "published_l1":d["l1_advertised"], "verified_l1":d["l1_verified"], "pc":position,
                  "reason":reason, "eligible":eligible, "heads":heads, "tail":d.get("tail"),
                  "clock_ready":d["clock_good"], "clock_sample":d["clock_sample"], "d":state,
                  "d_verified":local_d_verified,
                  "paths_ready":bool(state and state["services_ready"]), "d_drained":bool(state and state["drained"]),
                  "pending":reason == "CONTROL_IN_FLIGHT",
                  "recovery_reason": d.get("recovery_reason"), "common_comparison":d.get("common_comparison")}
        check = d.get("solution")
        if check:
            mismatch = (d.get("common_comparison") or {}).get("first_mismatch")
            result.update(inclusion=[check["t_l_min_us"],check["t_l_max_us"]],
                d_origin=state["d_origin"],
                inbox_growth=max(0, state["pending_batch_through"]-state["inbox_height"]),
                growth_basis={"d_l1_height":state["l1_height"], "pending_batch_through":state["pending_batch_through"]},
                signed_origin=check["t_l_min_us"]//self.profile["l1_slot_us"], target=check["target"], h_guard=check["h_guard"],
                guard_parts={"b_hi":check["b_hi"],"E_hi":check["e_hi"],"h_time":check["h_time"],
                             "extra_origin":check["extra_origin"],"H_D":check["d_max"],
                             "H_G":check["g_max"],"H_Inbox":check["inbox_max"]})
            if mismatch:
                result.update(difference_height=mismatch["height"],g_native=hx(mismatch["g_hash"]),
                              d_native=hx(mismatch["d_hash"]))
        if eligible:
            matched = [cand for cand in self.kinds["ResumeEligibility"]
                       if cand["detail"]["request"]["resume"]["target"] == check["target"]
                       and cand["detail"]["request"]["target_beta"] == d["branch"]
                       and cand["detail"]["request"]["head_height"] == d["g_head"]
                       and cand["detail"]["request"]["resume"]["origin"] == result["signed_origin"]]
            if len({key(candidate["detail"]["request"]) for candidate in matched}) == 1:
                request = matched[0]["detail"]["request"]
                intent = key(request)
                result.update(request=self.requests.get(intent, sha(intent.encode())), payload=payload(request))
            else:
                # A signer may ratify a proposal that never meets quorum. Preserve the
                # observed outcome; there is no exact request digest in that poll.
                result.update(request=None, outside_packed_request=True)
        input_refs = []
        for source_key in (("l1", i, d["l1_verified"]), ("espresso", i, verified["q"] if verified else None)):
            actual = self.signer_inputs.get(source_key)
            if actual:
                input_refs.append(trace_ref(actual))
        if motion_source:
            input_refs.append(trace_ref(motion_source))
        self.emit(row, "signer_view", result, *input_refs,
                  stage="target" if reason == "NO_TARGET" or eligible and not self.trigger_observations else None)
        if eligible:
            self.trigger_observations.append(row)

    def step(self, row):
        kind, d = row["kind"], row["detail"]
        if kind == "OrderingFault":
            self.active_before = "frozen" if self.mode == "frozen" else "stopped"
            self.emit(row,"ordering_fault", {"mode":self.active_before,
                "last":{"q":d["q"],"theta_us":d["theta_us"]}}, stage="fault")
        elif kind == "OrderingRecovered":
            self.active_before = None
            self.emit(row,"ordering_recovered", {"scenario":d["scenario"]}, stage="recovery")
        elif kind == "GfrCoverageWait":
            position = pc(d.get("position", d.get("next_position", d.get("through"))))
            latest = self.pelg[self.latest_q]
            require(pc(d["pc"]) == self.g_position, "WAIT altered the observed GFR pc")
            observed = d.get("missing", d.get("status"))
            missing = "ESPRESSO_WATERMARK" if "EspressoWatermark" in str(observed) else observed
            self.emit(row, "g_ordering_wait", {"position":position, "validated_q":self.latest_q,
                "theta_us":latest["theta_us"], "reader":"A","missing":missing,"observed":observed,
                "prev_pc":self.g_position}, source(latest["path"],"已观察水位","PELG"), stage="wait")
        elif kind in ("LatePublicationHeld", "LatePublicationExcluded"):
            held = kind == "LatePublicationHeld"
            value = {**d, "cid":hx(d["cid"]), "physical_id":hx(d["physical_id"])}
            self.emit(row, "publication_held" if held else "publication_excluded", value,
                      stage="held-publication" if held else "excluded-publication")
        elif kind == "LocalStopped":
            self.emit(row, "local_stop", {"worker":row["pid"],"cid":hx(d["cid"]),
                "clock_us":d["local_time_us"],"stop_us":d["local_stop_us"],"terminal":True,
                "window_end":(d["local_stop_us"]-self.profile["genesis_timestamp_us"])//self.profile["slot_us"]-1,
                "logical_step_id":d["logical_step_id"],"ack":hx(d["ack"])}, stage="local-stop")
        elif kind == "GovernanceL1Available":
            self.signer_inputs["l1",d["signer"],d["height"]] = row
        elif kind == "GovernanceEspressoAvailable":
            self.signer_inputs["espresso",d["signer"],d["q"]] = row
        elif kind in ("GovernancePoll", "ResumeInputEvaluation"):
            self.signer(row)
        elif kind == "ResumeEligibility":
            intent = key(d["request"])
            self.intent_rows[intent] = row
            self.emit(row, "resume_eligibility", {"request": self.requests.get(intent, sha(intent.encode())),
                "t_eligible_us":d["t_eligible_us"],"signers":d["signers"],"solution":d["solution"],
                "payload":payload(d["request"])}, stage="eligible")
        elif kind == "GovernanceState":
            self.governance(row)
        elif kind == "DerivationAvailable":
            before = len(self.events)
            super().step(row)
            for event in self.events[before:]:
                if event["stage"] == "fallback" and "d-advance" not in self.marked_stages:
                    event["stage"] = "d-advance"
                    self.marked_stages.add("d-advance")
        elif kind in ("Ready", "Enter"):
            super().step(row)
            stage_name = kind.lower()
            if (hx(d["cid"]) != self.profile["initial_duty"]["cid"]
                    and stage_name not in self.marked_stages):
                self.marked_stages.add(stage_name)
                self.events[-1]["stage"] = stage_name
        elif kind == "ResumeTiming":
            require(row["tick_us"] == self.checked_signatures["actual_include_us"]
                    and self.prepared and hx(d["transaction_hash"]) == hx(self.prepared["tx"]) and
                    [d["t_l_min_us"], d["t_l_max_us"]] == self.prepared["inclusion"] and
                    d["actual_inclusion_us"] == row["tick_us"], "actual timing uses distinct signed intent")
            self.emit(row, "resume_timing", d)
        else:
            super().step(row)

    def governance(self, row):
        phase, d = row["detail"]["phase"], row["detail"]["evidence"]
        if phase == "Prepared":
            intent = key(d["request"])
            digest = hx(d["digest"])
            eligible = self.intent_rows[intent]
            proof_rows = []
            members = []
            for entry in d["signers"]:
                who = entry["signer"]
                reported = one((r for r in self.kinds["GovernanceSignature"] if r["detail"] == entry),
                               "prepared signature exists in raw log")
                proof_rows.append(trace_ref(reported))
                address = entry.get("address", entry.get("member")).lower()
                require(address == self.members[who] and hx(entry["digest"]) == digest,
                        "prepared signer index/digest mismatch")
                members.append({"signer":who, "address":address, "signed_us":reported["tick_us"],
                                "signature_ref":sha(bytes.fromhex(entry["signature"].removeprefix("0x"))),
                                "digest":digest})
            members.sort(key=lambda signer: signer["address"])
            require([entry["address"] for entry in members] == self.resume["signers"],
                    "signature pack differs from independently decoded actual receipt")
            solution = eligible["detail"]["solution"]
            self.intent = eligible["detail"]["t_eligible_us"]
            require(self.intent == self.checked_signatures["eligible_us"]
                    and row["tick_us"] == self.checked_signatures["prepared_us"]
                    and [solution["t_l_min_us"], solution["t_l_max_us"]] == self.checked_signatures["inclusion_interval_us"],
                    "prepared timing and original checker differ")
            fields = payload(d["request"])
            inclusion = [solution["t_l_min_us"],solution["t_l_max_us"]]
            self.emit(row, "resume_signed", {"request":digest, "target":fields["target"],
                "t_eligible_us":self.intent,"inclusion":inclusion,"members":members,"payload":fields},
                trace_ref(eligible), *proof_rows, stage="signed")
            self.prepared = {"request":digest,"tx":d["hash"].lower(),"target":fields["target"],
                             "durable":True,"payload":fields,"inclusion":inclusion}
            self.emit(row,"resume_prepared",self.prepared,stage="prepared")
        elif phase in ("Submitted", "SubmittedUnknown") and not self.sent:
            require(hx(d["hash"]) == hx(self.prepared["tx"]), "broadcast differs from durable transaction")
            delay = row["tick_us"] - self.intent
            budget = sum(self.profile[k] for k in ("poll_us", "sign_us", "broadcast_us"))
            require(row["tick_us"] == self.checked_signatures["submitted_us"], "actual submit time differs from checker")
            self.sent = True
            self.emit(row,"resume_submitted",{**self.prepared,"submit_delay_us":delay,
                      "within_budget":delay<=budget, "phase":phase},stage="submit")
        elif phase == "Submitted" and self.sent:
            self.emit(row,"resume_confirmation",{"tx":d["hash"],"phase":phase})
        elif phase == "HeldBeforeDriver":
            self.emit(row,"resume_held",d)

    def tape(self):
        tape = super().tape()
        return {**tape, "schema":SCHEMA, "label":LABEL[self.mode], "expect_resume":True}
