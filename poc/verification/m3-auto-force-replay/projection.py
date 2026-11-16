"""Bind the AF01 replay to a sealed M3 run and its independent checker."""

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "m1-gfr-replay"))

from force_projection import attach_acceptance, fixture_inputs, verify_assignment
from generate import Bundle, equal, index_unique, public_index, read_trace, trace_anchor, trace_hash, trace_subject
from wire import decode_config, decode_descriptor, decode_espresso, decode_result, domain_hash, sha256

from model import assemble, event, historical_choice, position_name, require, seconds, signature_set, signer_view


KINDS = ("WorkerOffline", "EspressoHeartbeat", "GovernancePoll", "GovernanceSignature", "GovernanceState",
         "GovernanceSubmission", "L1Included", "L1Finalized", "L1ReaderAvailable", "ForceActivated", "GPadding", "RunFailed")
RPC = "raw/l1/raw-rpc-bundle.json"
HUB = "raw/l1/hub-l1.jsonl"
GOV = "raw/governance.jsonl"
TRANSACTION = "raw/l1/governance-transaction.json"
ORACLE = "public-replay.json"
CONTROLS = "verified-controls.json"
HEADERS = "verified-l1-headers.json"
REPORT = "report.json"


class AutoBundle(Bundle):
    def __init__(self, directory):
        self.root = directory.resolve()
        raw = (directory / "manifest.json").read_bytes()
        self.manifest_hash = sha256(raw)
        equal((directory / "manifest.sha256").read_text().split(),
              [self.manifest_hash, "manifest.json"], "manifest.sha256")
        self.manifest = json.loads(raw)
        m = self.manifest
        equal((m["schema"], m["status"], m["scenario"], m["seed"]),
              ("M3-AUTO-FORCE-RUN-1", "COMPLETED", "outage", 42), "manifest.json AF01")
        require(bool(re.fullmatch(r"[0-9a-f]{40}", m["candidate"])), "manifest candidate SHA")
        equal(m["before"], m["after"], "sealed candidate checkout")
        equal((m["before"]["head"], m["before"]["status"]), (m["candidate"], ""), "clean candidate checkout")
        self.files = m["files"]
        self.digests = {}

    def read(self, relative):
        path = self.root / relative
        require(path.resolve().is_relative_to(self.root) and not path.is_symlink(),
                f"{relative}: unsafe evidence path")
        return super().read(relative)

    def json(self, relative):
        return json.loads(self.read(relative))

    def jsonl(self, relative):
        return [json.loads(row) for row in self.read(relative).splitlines()]


def ref(row):
    return f"raw/trace.jsonl:{row['seq']} · {row['kind']}"


def at(lane, stage, label, row, facts=(), refs=(), **state):
    return event(lane, stage, label, row["tick_us"], row["seq"], facts, [ref(row), *refs], **state)


def only(rows, label):
    items = list(rows)
    require(len(items) == 1, f"{label}: expected one row, found {len(items)}")
    return items[0]


def hex_value(value):
    return value.removeprefix("0x").lower() if isinstance(value, str) else trace_hash(value)


def load_checker(directory, bundle):
    result, hashes = {}, {}
    for name in (REPORT, ORACLE, CONTROLS, HEADERS):
        data = (directory / name).read_bytes()
        hashes[name] = sha256(data)
        result[name] = json.loads(data)
    report = result[REPORT]
    equal((report["status"], report["mode"], report["candidate"], report["manifest_sha256"]),
          ("PASS", "outage", bundle.manifest["candidate"], bundle.manifest_hash), "independent AF01 report binding")
    for name, field in ((ORACLE, "public_sha256"), (CONTROLS, "controls_sha256"), (HEADERS, "l1_headers_sha256")):
        if field in report:
            equal(hashes[name], report[field], f"checker authentication of {name}")
    control = only(result[CONTROLS], CONTROLS)
    return report, result[ORACLE], control, result[HEADERS], hashes


def project_l1(bundle, trace, control, report, profile, checker_headers):
    rpc = bundle.json(RPC)
    hub_rows = bundle.jsonl(HUB)
    hub = {(r["command"]["op"], r["command"].get("height"), r["command"].get("reader")): (i, r)
           for i, r in enumerate(hub_rows, 1) if r["command"]["op"] in ("mine", "finalize", "deliver")}
    block, tx, log = control["position"]
    headers = rpc["blocks"]
    equal([{"height": int(row["number"], 16), "timestamp_s": int(row["timestamp"], 16),
            "hash": list(bytes.fromhex(hex_value(row["hash"])))} for row in headers],
          checker_headers, "independent verified historical Anvil headers")
    header = headers[block]
    receipt_index, receipt = only(((i, r) for i, r in enumerate(rpc["receipts"])
                                   if r["transactionHash"] == control["tx_hash"]), "Force receipt")
    receipt_log = only((row for row in receipt["logs"] if int(row["logIndex"], 16) == log), "ForceRecorded")
    equal((receipt["status"], receipt["blockHash"], receipt["transactionHash"]),
          ("0x1", header["hash"], header["transactions"][tx]["hash"]), "actual Force receipt")
    equal([int(receipt_log[k], 16) for k in ("blockNumber", "transactionIndex", "logIndex")],
          control["position"], "ForceRecorded receipt log position")
    equal(receipt_log["transactionHash"], control["tx_hash"], "ForceRecorded tx")
    timestamp = int(header["timestamp"], 16) * 1_000_000
    equal(timestamp + profile["announcement_us"], control["act_us"], "act=ts_L1+M_a")

    include = index_unique(trace["L1Included"], lambda r: r["detail"]["height"], "L1Included")
    final = index_unique(trace["L1Finalized"], lambda r: r["detail"]["height"], "L1Finalized")
    deliveries = index_unique(trace["L1ReaderAvailable"], lambda r: (r["detail"]["height"], r["detail"]["reader"]), "L1ReaderAvailable")
    stage = report["l1_stages"]
    events, l1 = [], []
    for height, item in enumerate(headers):
        equal(int(item["number"], 16), height, "Anvil block height")
        if height:
            equal(item["parentHash"], headers[height - 1]["hash"], "Anvil parent header")
        readable = {}
        for reader in ("A", "B"):
            row = deliveries.get((height, reader))
            if row:
                equal(hex_value(row["detail"]["hash"]), hex_value(item["hash"]), "actual reader header hash")
                readable[reader] = {"trace": {"tick_us": row["tick_us"], "seq": row["seq"]}}
        hdr = {"height": height, "hash": item["hash"], "timestamp_us": int(item["timestamp"], 16) * 1_000_000,
               "finalized_us": final[height]["tick_us"] if height in final else None, "readers": readable}
        l1.append(hdr)
        if height in include:
            row = include[height]
            hub_line, mine = hub[("mine", height, "A")]
            equal((row["detail"]["hash"], row["tick_us"], mine["result"]["header_and_transactions"]["hash"]),
                  (item["hash"], mine["command"]["tick_us"], item["hash"]), f"ℓ{height} actual mine")
            facts = [("L1 timestamp", seconds(hdr["timestamp_us"])), ("Anvil header hash", item["hash"])]
            refs = [f"{RPC} · blocks[{height}]", f"{HUB}:{hub_line}"]
            if height == block:
                facts += [("真实交易 hash", control["tx_hash"]), ("Receipt status", receipt["status"]),
                          ("控制位置 (block,tx,log)", control["position"]), ("公开 act", seconds(control["act_us"]))]
                refs += [f"{RPC} · receipts[{receipt_index}] · ForceRecorded logIndex={log}", f"{CONTROLS}[0]"]
                equal(row["tick_us"], stage["include_us"], "actual inclusion tick")
            events.append(at("l1", "include" if height == block else None, f"ℓ{height} 收录", row, facts, refs))
        if height in final:
            row = final[height]
            hub_line, completion = hub[("finalize", height, "A")]
            equal((row["tick_us"], completion["result"]["finalized_through"]),
                  (completion["command"]["tick_us"], height), f"ℓ{height} actual finalize")
            if height == block:
                equal(row["tick_us"], stage["finalize_us"], "actual finalized tick")
            events.append(at("l1", "finalized" if height == block else None, f"ℓ{height} finalized", row,
                             [("L1 timestamp", seconds(hdr["timestamp_us"]))], [f"{HUB}:{hub_line}"]))
        for reader in readable:
            row = deliveries[(height, reader)]
            refs = [f"{RPC} · blocks[{height}]"]
            if height:
                hub_line, delivered = hub[("deliver", height, reader)]
                equal((row["tick_us"], delivered["result"]["header_and_transactions"]["hash"]),
                      (delivered["command"]["tick_us"], item["hash"]), f"Reader {reader} ℓ{height}")
                refs.append(f"{HUB}:{hub_line}")
            if height == block:
                equal(row["tick_us"], stage["reader_us"][reader], "actual reader delivery tick")
            events.append(at("l1", "reader" if height == block and reader == "A" else None,
                             f"Reader {reader} 已读 ℓ{height}", row,
                             [("实际 L1 hash", item["hash"]), ("L1 timestamp", seconds(hdr["timestamp_us"]))], refs))
    return events, l1, rpc["config"]["genesisOrigin"]


def project_governance(bundle, trace, report, control, profile):
    events = []
    offline = trace["WorkerOffline"]
    equal(sorted(r["pid"] for r in offline), list(range(5)), "R0 five offline workers")
    equal({str(r["pid"]): r["seq"] for r in offline}, report["force"]["offline_worker_trace_seq"],
          "checker fault records")
    for index, row in enumerate(offline):
        off = row["detail"]
        equal((row["tick_us"], off["region"], off["signal"]), (profile["outage_us"], 0, 9),
              "offline worker schedule")
        events.append(at("governance", "outage" if index == len(offline) - 1 else None,
                         "R0 五个 worker 离线" if index == len(offline) - 1 else f"R0 worker {row['pid']} 离线",
                         row, [("逻辑 pid", row["pid"]), ("OS pid", off["os_pid"]), ("退出 signal", off["signal"]),
                               ("五个退出证据", [ref(r) for r in offline]), ("cid", hex_value(off["cid"]))]))
    prepared = only(trace["GovernanceSubmission"], "GovernanceSubmission")
    transaction = bundle.json(TRANSACTION)
    equal(transaction, prepared["detail"], "durable prepared transaction")
    hub_rows = bundle.jsonl(HUB)
    hub_prepare = only((row for row in hub_rows if row["command"]["op"] == "governance_prepare"),
                       "one hub governance preparation")
    attempts = [(line, row) for line, row in enumerate(hub_rows, 1)
                if row["command"]["op"] in {"governance_broadcast", "governance_reconcile"}]
    equal((hub_prepare["result"], hub_prepare["command"]["tick_us"]), (transaction, prepared["tick_us"]),
          "prepared trace/journal matches actual driver request")
    approvals = transaction["signers"]
    digest, request = signature_set(approvals, profile["threshold"])
    equal((transaction["hash"], transaction["digest"], transaction["request"], digest, transaction["nonce"]),
          (control["tx_hash"], control["request_digest"], request, control["request_digest"], control["nonce"]),
          "prepared transaction matches receipt")
    ordered = sorted(approvals, key=lambda item: item["address"].lower())
    ordered_addresses = [item["address"].lower() for item in ordered]
    approval_ids = {item["signer"] for item in approvals}
    equal(ordered_addresses, control["signers"], "actual recovered addresses")
    require(set(ordered_addresses) <= set(report["committee"]["members"]), "threshold signer outside committee")
    equal([item["signature"] for item in ordered], control["signatures"], "canonical packed EIP-712 signatures")
    equal(transaction["signature_pack"], "0x" + "".join(hex_value(sig) for sig in control["signatures"]),
          "durable signature package matches mined transaction")
    equal((request["request_id"], request["control_seq"], request["target_beta"],
           request["enabled_mask"], request["order"], request["signed_floor"]),
          (int(control["request_id"], 16), control["seq"], control["target"],
           control["policy"]["enabled_mask"], control["policy"]["order"], control["l1_floor"]),
          "signed policy matches independently decoded actual receipt")
    approvals_ref = f"{CONTROLS}[0] · signers / signatures"
    eligible = {}
    first_decision = None
    for row in trace["GovernancePoll"]:
        view = signer_view(row["detail"], profile["stall_us"])
        if view["eligible"]:
            eligible[view["signer"]] = row
            if first_decision is None and len(approval_ids.intersection(eligible)) == profile["threshold"]:
                first_decision = row["seq"]
        events.append(at("governance", "decision" if row["seq"] == first_decision else None,
                         f"Signer {view['signer']} · {view['reason']}", row, signer=view))
    require(first_decision is not None, "signed request lacks independent eligible polls")
    raw_signed = {}
    for row in trace["GovernanceSignature"]:
        sig = row["detail"]
        who = sig["signer"]
        require(who not in raw_signed and sig["digest"] == digest and sig["request"] == request,
                "duplicate or inconsistent governance signature")
        raw_signed[who] = sig
        prior = [poll for poll in trace["GovernancePoll"] if poll["seq"] < row["seq"] and
                 poll["detail"]["eligible"] and poll["detail"]["signer"] == who]
        require(bool(prior), f"signer {who} lacks a prior eligible poll")
        equal(row["tick_us"], report["governance"]["vote_ticks_us"][sig["address"]], "checker governance vote tick")
        events.append(at("governance", None, f"Signer {who} 签署", row, [
            ("Signer 地址", sig["address"]), ("签署 digest", digest), ("签署请求", request),
            ("进入实际交易门槛包", who in approval_ids),
        ], [ref(prior[-1]), approvals_ref], signedBy=who))
    equal([raw_signed[item["signer"]] for item in approvals], approvals, "packed signature to raw trace")
    states = trace["GovernanceState"]
    durable = bundle.jsonl(GOV)
    equal(durable, [{"phase": r["detail"]["phase"], "tick_us": r["tick_us"], "detail": r["detail"]["evidence"]}
                    for r in states], "governance journal to trace")
    first_attempt = next(r["seq"] for r in states if r["detail"]["phase"] in {"SubmittedUnknown", "Submitted"})
    signature_seqs = {r["detail"]["signer"]: r["seq"] for r in trace["GovernanceSignature"]}
    for i, row in enumerate(states, 1):
        phase = row["detail"]["phase"]
        value = row["detail"]["evidence"]
        stage = ("submit" if row["seq"] == first_attempt else
                 {"Signed": "signed", "Prepared": "prepared", "Submitted": "submitted"}.get(phase))
        if phase == "Submitted":
            equal(value["hash"], control["tx_hash"], "reconciled actual broadcast uses prepared hash")
        facts = [("阶段", phase)]
        if phase == "Signed":
            initial = list(value["signatures"].values())
            equal(signature_set(initial, profile["threshold"]), (digest, request), "durable threshold signature set")
            require(all(raw_signed[sig["signer"]] == sig and signature_seqs[sig["signer"]] < row["seq"]
                        for sig in initial), "threshold state used a signature before its trace event")
            facts += [("Digest", digest), ("达到门槛时的 signer 地址", [sig["address"] for sig in initial]),
                      ("治理门槛", f"{profile['threshold']}/{profile['signer_count']}")]
        if phase in {"Prepared", "SubmittedUnknown", "Submitted"}:
            require(all(signature_seqs[sig["signer"]] < row["seq"] for sig in approvals),
                    "prepared transaction used a future signature")
            facts += [("Digest", digest), ("实际打包签名地址", ordered_addresses)]
            facts += [("交易 hash", control["tx_hash"]), ("持久准备 τ", seconds(prepared["tick_us"])),
                      ("Nonce", transaction["nonce"])]
        if row["seq"] == first_attempt:
            equal(row["tick_us"], report["l1_stages"]["submit_us"], "first actual broadcast attempt")
        refs = [f"{GOV}:{i}", f"{REPORT} · governance"]
        if phase.startswith("Submitted"):
            require(bool(attempts), "governance broadcast missing driver response")
            line, attempt = attempts.pop(0)
            equal((attempt["command"]["tick_us"], attempt["result"]), (row["tick_us"], value), "driver broadcast evidence")
            facts.extend([("首次实际广播尝试 τ（checker）", seconds(report["l1_stages"]["submit_us"])),
                          ("L1 driver 状态", value["status"]), ("重读持久交易", value["reloaded"])])
            refs.append(f"{HUB}:{line}")
        if phase in {"Prepared", "SubmittedUnknown", "Submitted"}:
            refs.extend([ref(prepared), TRANSACTION, approvals_ref])
        events.append(at("l1" if phase.startswith("Submitted") else "governance", stage,
                         f"治理 · {phase}", row, facts, refs))
    require(not attempts, "driver broadcasts omitted from governance trace")
    return events


def historical_ref(config, new, control, floor, history, actual_hash):
    def uint(value, size=8):
        return value.to_bytes(size, "big")

    block, tx, log = control["position"]
    reference = new["reference"]
    mask = bytes([control["policy"]["enabled_mask"], *control["policy"]["order"]])
    boundary = (bytes([new["cause"]]) + uint(new["beta"]) + uint(new["session"])
                + bytes([1, new["source_region"]]) + uint(new["window"][0]) + uint(new["window"][1])
                + uint(reference["height"]) + uint(reference["origin"]) + bytes.fromhex(reference["physical"]))
    legacy = domain_hash("PoC/Authorization/v1", b"\x00\x01" + bytes.fromhex(config["ref"]) +
                         bytes.fromhex(config["registry_hash"]) + boundary)
    return domain_hash("PoC/HistoricalPlan/v1", bytes.fromhex(legacy), uint(block), uint(tx, 4), uint(log, 4),
                       uint(control["seq"]), mask, uint(control["l1_floor"]), uint(floor), uint(history),
                       bytes.fromhex(actual_hash), bytes([new["region"]]))


def project_force(trace, raw, old, new, config, control, profile, oracle, report, espresso, l1, old_floor,
                  head, old_state, covering):
    observed = only(trace["ForceActivated"], "ForceActivated")
    d = observed["detail"]
    position = raw["detail"]["position"]
    witness = only(oracle["force"], "independent Force")
    equal(witness, report["force"]["state"], "checker historical Force state")
    bits = witness["historical_plan"]
    equal(d["position"], position, "actual activation position")
    equal(raw["tick_us"], report["force"]["activation_tick_us"], "actual activation tick")
    equal(raw["tick_us"], observed["tick_us"], "activation announcement tick")
    equal((d["old_branch"], d["new_branch"], d["head_height"], hex_value(d["record_id"])),
          (old["beta"], new["beta"], head["height"], hex_value(raw["detail"]["id"])), "activation matches GFR commit")
    equal((control["target"], position["kind"]), (old["beta"], "Force"), "Force targets open branch")
    equal(position["key_us"], control["act_us"], "Force public act")
    block, tx, log = control["position"]
    equal(position["index"]["L1"], {"block": block, "tx": tx, "log": log}, "Force position")
    q_star = next(row["q"] for row in espresso if row["key_us"] >= position["key_us"])
    equal((q_star, bool(espresso[q_star]["messages"])), (witness["q_star"], False),
          "first covering empty Espresso block")
    equal(d["espresso_q"], q_star, "activation covering q")
    require(covering["seq"] < raw["seq"], "covering heartbeat was not finalized before Force")
    old_end = config["genesis"]["timestamp_us"] + (old["window"][1] + 1) * config["l2_slot_us"] + profile["grace_us"]
    equal(old_end, position["key_us"], "old SessionEnd and public Force key tie")
    equal(oracle["regular"], [], "old SessionEnd suppressed after tied Force")
    require(not raw["detail"]["append"] and l1[block]["readers"]["A"]["trace"]["tick_us"] <= raw["tick_us"]
            and espresso[q_star]["finalized_us"] <= raw["tick_us"], "Force public coverage/order")
    [closed, assigned] = raw["detail"]["events"]
    equal(closed["body"], {"BranchClosed": {"branch": old["beta"], "reason": "Force"}}, "Force closes old branch")
    equal([closed["position"], assigned["position"]], [position, position], "atomic Force lifecycle positions")
    verify_assignment(d["descriptor"], new)
    equal(assigned["body"]["SessionAssigned"]["descriptor"], d["descriptor"], "atomic selected descriptor")
    selected, cycle, floor, history = historical_choice(control["policy"], old["region"],
                                                         new["reference"]["origin"], block, old_floor)
    anchor = trace_anchor(d["descriptor"]["anchor"])
    equal((anchor, new["anchor"], hex_value(witness["anchor_root"])),
          (trace_anchor(witness["anchor"]), anchor, old_state), "selected physical anchor")
    equal(anchor, {key: head[key] for key in ("height", "origin", "physical", "timestamp_us")},
          "activation anchor equals actual old G head")
    first = max(anchor["height"] + 1, (control["act_us"] + profile["force_init_us"]
                                      - config["genesis"]["timestamp_us"]) // config["l2_slot_us"] + 1)
    equal((new["window"], new["region"], new["beta"], new["session"], new["source_region"]),
          ([first, first + 9], selected, old["beta"] + 1, old["session"] + 1, old["region"]), "historical successor")
    actual_hash = hex_value(l1[history]["hash"])
    equal((d["plan_basis"]["height"], hex_value(d["plan_basis"]["block_hash"]),
           bits["historical_height"], hex_value(bits["historical_hash"]), d["l1_floor"]),
          (history, actual_hash, history, actual_hash, floor), "historical Anvil Plan support")
    equal((d["policy"]["enabled_mask"], d["policy"]["order"], d["policy"]["signed_floor"], d["policy"]["floor"]),
          (control["policy"]["enabled_mask"], control["policy"]["order"], control["l1_floor"], floor),
          "actual historical policy")
    computed = historical_ref(config, new, control, floor, history, actual_hash)
    equal((computed, new["historical_rule_ref"]), (new["authorization_ref"], computed),
          "historical authorization hash")
    refs = [ref(observed), f"{RPC} · blocks[{history}]", f"{ORACLE}.force[0].historical_plan", f"{HEADERS}[{history}]",
            f"{CONTROLS}[0]", "raw/descriptor-b1-r1.bin", espresso[q_star]["file"]]
    return at("g", "force", f"Force 激活 · R{old['region']} → R{selected}", raw, [
        ("公开 act", seconds(position["key_us"])), ("Force L1 位置", control["position"]),
        ("控制覆盖 x − Mₐ", seconds(position["key_us"] - profile["announcement_us"])),
        ("Espresso watermark", f"q{q_star} · θ {seconds(espresso[q_star]['key_us'])}"),
        ("水位 finalized τ", seconds(espresso[q_star]["finalized_us"])),
        ("旧 SessionEnd 公开键（推导）", seconds(old_end)),
        ("同键全序", "L1 controls → Expire → SessionEnd → Espresso"),
        ("生命周期", "BranchClosed(β0) + SessionAssigned"), ("G", f"保留 h{anchor['height']}"),
    ], refs, pc=position_name(position), duty=f"R{selected} · β{new['beta']}/r{new['session']} [{first},{first+9}]",
       plan=[("policy mask", f"0b{control['policy']['enabled_mask']:03b}"), ("policy order", control["policy"]["order"]),
             ("来源区域", f"R{old['region']}"), ("循环搜索序列", [f"R{i}" for i in cycle]),
             ("首个启用且具备资格的区域", f"R{selected}"), ("签署 floor", control["l1_floor"]),
             ("有效 floor", floor), ("历史 L1 高度 max(Ref.origin, floor)", history),
             ("实际历史 header hash", actual_hash), ("Ref.origin", new["reference"]["origin"]),
             ("固定等权 registry（空禁任）", config["registry_hash"]),
             ("authorization_ref 重算", computed), ("连接 anchor", anchor), ("cid", new["cid"]),
             ("后继窗口", new["window"])])


def project_public(bundle, trace, profile, config, control, report, oracle, l1, old_floor):
    espresso = bundle.bins("espresso", decode_espresso)
    old, new = [decode_descriptor(bundle.read(path), path)
                for path in ("raw/genesis-descriptor.bin", "raw/descriptor-b1-r1.bin")]
    equal((old["region"], old["beta"], old["session"], old["config_ref"], new["config_ref"]),
          (0, 0, 0, config["ref"], config["ref"]), "selected public configuration")
    contexts = {d["cid"]: d for d in (old, new)}
    equal([{"q": row["q"], "theta_us": row["key_us"], "proposed_us": row["proposal_us"],
            "finalized_us": row["finalized_us"]} for row in espresso],
          [{key: row[key] for key in ("q", "theta_us", "proposed_us", "finalized_us")}
           for row in oracle["espresso"]], "all Espresso headers vs independent replay")
    bodies, proofs = public_index(espresso)
    results = bundle.bins("results", decode_result)
    finals = index_unique(enumerate(oracle["public_finals"]),
                          lambda pair: (pair[1]["q"], pair[1]["message"]), "checker public proof")
    equal(len(proofs), len(results), "publication regional result count")
    equal(len(proofs), len(finals), "independent proof count")
    _, dep = fixture_inputs(bundle, config, oracle)
    deliveries = defaultdict(list)
    for row in trace["RegionalDelivered"]:
        deliveries[trace_subject(row)].append(row)
    pub = index_unique(trace["PublisherRecorded"], trace_subject, "PublisherRecorded")
    accept = index_unique(trace["GAccepted"], trace_subject, "GAccepted")
    pad = index_unique(trace["GPadding"], lambda r: r["detail"]["height"], "GPadding")
    heartbeat = index_unique(trace["EspressoHeartbeat"], lambda r: r["detail"]["q"], "EspressoHeartbeat")
    by_q, events = {}, []
    for result in results:
        body, d = result["body"], result["descriptor"]
        h = body["height"]
        key = d["cid"], body["physical"]
        equal(d, contexts[d["cid"]], f"h{h} selected result descriptor")
        p, proof_ref = proofs[key]
        q, msg = proof_ref["q"], proof_ref["message"]
        equal((p["descriptor"], p["proof"]), (d, result["proof"]), f"h{h} raw proof and result")
        parent, ancestry = d["anchor"]["physical"], []
        for candidate in result["chain"]:
            observed, location = bodies[candidate["physical"]]
            equal((candidate["parent"], candidate), (parent, observed), "public physical ancestry")
            parent = candidate["physical"]
            ancestry.append(location)
        q_avail = max(q, *(location["q"] for location in ancestry))
        proof_index, independent = finals[(q, msg)]
        equal((independent["cid"], independent["target"], independent["proof"], independent["q_avail"]),
              (d["cid"], body["physical"], result["proof"], q_avail), f"h{h} independent proof output")
        published = pub[key]
        detail = published["detail"]
        equal((detail["q"], detail["q_avail"], detail["theta_us"], detail["message_index"], published["tick_us"]),
              (q, q_avail, espresso[q]["key_us"], msg, espresso[q]["finalized_us"]), "publication placement")
        by_q[q] = (result, key, q_avail, proof_index, proof_ref)
        rows = deliveries[key]
        equal(sorted(row["pid"] for row in rows), list(range(d["region"] * 5, d["region"] * 5 + 5)),
              f"h{h} region deliveries")
        facts = [("类型", "区域 physical / BLS Final"), ("CID", d["cid"]), ("物理块", body["physical"]),
                 ("物理父块", body["parent"]), ("区域 Final 门槛", "4/5")]
        events.append(at("regional", "new-result" if d["beta"] else "old-result",
                         f"R{d['region']} physical h{h}", rows[0], facts,
                         [result["file"], *[ref(row) for row in rows[1:]]]))

    heartbeat_stage = True
    for q, row in enumerate(espresso):
        facts = [("公开键 θ", seconds(row["key_us"])), ("Finalized τ", seconds(row["finalized_us"])),
                 ("消息数", len(row["messages"]))]
        if q in by_q:
            result, key, q_avail, proof_index, proof_ref = by_q[q]
            traced = pub[key]
            refs = [row["file"], f"{ORACLE}.public_finals[{proof_index}]", f"{row['file']} · message {proof_ref['message']}"]
            facts += [("类型", "公开区域结果"), ("q_avail", q_avail)]
            stage, kind = None, "publication"
        else:
            traced = heartbeat[q]
            equal((traced["detail"]["theta_us"], traced["tick_us"], traced["detail"]["message_count"],
                   len(row["messages"])), (row["key_us"], row["finalized_us"], 0, 0), "empty heartbeat trace/binary")
            refs = [row["file"]]
            stage = "heartbeat" if heartbeat_stage and traced["tick_us"] > profile["outage_us"] else None
            if stage:
                heartbeat_stage = False
            kind = "空 heartbeat"
        events.append(at("espresso", stage, f"q{q} {kind}", traced, facts, refs,
                         water=f"q{q} · {seconds(row['key_us'])}"))
    expected = oracle["expected"]
    equal(len(trace["GAccepted"]) + len(pad), len(expected), "checker virtual G inventory")
    last, state = config["genesis"], config["genesis_state_root"]
    control_q = only(oracle["force"], "one independent Force")["q_star"]
    records = trace["GfrReplayRecord"]
    replay_order = [r["detail"]["position"]["kind"] for r in records]
    equal(replay_order, ["Espresso"] * control_q + ["Force"] + ["Espresso"] * (len(espresso) - control_q),
          "Force precedes its covering Espresso replay event")
    next_q = 0
    for source_seq, raw in enumerate(records):
        record = raw["detail"]
        equal(record["source_seq"], source_seq, "ordered source_seq")
        position = record["position"]
        kind = position["kind"]
        if kind == "Espresso":
            equal((position["index"]["Espresso"], position["key_us"]),
                  (next_q, espresso[next_q]["key_us"]), "actual replay position and PELG key")
            next_q += 1
        pc = position_name(position)
        if kind == "Force":
            events.append(project_force(trace, raw, old, new, config, control, profile, oracle, report, espresso,
                                        l1, old_floor, last, state, heartbeat[control_q]))
        elif not record["append"]:
            equal(kind, "Espresso", "unanticipated replay event")
            events.append(at("g", None, f"处理 q{position['index']['Espresso']} · G 保持 h{last['height']}", raw, pc=pc))
        else:
            q = position["index"]["Espresso"]
            result, key, q_avail, proof_index, proof_ref = by_q[q]
            body, context = result["body"], result["descriptor"]
            h = body["height"]
            equal(body["pre_state_root"], state if h == 1 else expected[h-2]["state_root"], f"h{h} execution pre-state")
            entry = {"body": body, "cid": context["cid"], "q": q, "q_avail": q_avail, "position": position,
                     "publisher": pub[key], "oracle": {}}
            attach_acceptance(entry, context, accept[key], raw, expected, pad, last, config, l1, dep, profile, espresso)
            rows = [{"height": item["height"], "native": item["native"], "state_root": item["state_root"],
                     "type": "padding", "ref": ref(item["trace"])} for item in entry["padding"]]
            rows.append({"height": h, "native": entry["native"], "state_root": entry["state_root"],
                         "type": "physical", "ref": ref(accept[key])})
            facts = [("虚拟 G 追加", entry["append"]), ("公开接纳键", seconds(position["key_us"])),
                     ("首次公开完整位置 q_avail", q_avail), ("物理父块", body["parent"]),
                     ("物理父高度", last["height"]), ("最早过期键", seconds(entry["earliest_expiry_us"]))]
            if entry["padding"]:
                facts.append(("physical 执行前态", f"padding h{h-1} 的执行后状态 · {body['pre_state_root']}"))
            events.append(at("g", "accepted" if context["beta"] else "old-accepted",
                             f"G 原子追加 h{entry['append'][0]}–h{h}", raw, facts,
                             [result["file"], f"{espresso[q]['file']} · message {proof_ref['message']}",
                              f"{ORACLE}.expected[{h-1}]", f"{ORACLE}.public_finals[{proof_index}]"],
                             head=h, pc=pc, appends=rows))
            last, state = body, entry["state_root"]
    equal((last["height"], expected[-1], accept[(new["cid"], last["physical"])]["tick_us"]),
          (18, report["force"]["first_new"], report["force"]["first_global_tick_us"]), "first qualifying recovery")
    return events


def formal_projection(path, independent):
    bundle = AutoBundle(path)
    report, oracle, control, checked_headers, hashes = load_checker(independent, bundle)
    raw_profile = bundle.json("raw/profile.json")
    auto = raw_profile["auto_force"]
    equal((raw_profile["schema"], raw_profile["seed"], auto["scenario"]), ("M3-AUTO-FORCE-1", 42, "outage"),
          "sealed AF01 profile")
    equal((auto["threshold"], auto["committee"], report["committee"]["threshold"], len(report["committee"]["members"])),
          (2, 3, 2, 3), "actual governance committee")
    profile = {**raw_profile, **auto, "signer_count": auto["committee"], "force_init_us": 3_000_000}
    trace = read_trace(bundle.read("raw/trace.jsonl"), KINDS)
    require(not trace["RunFailed"], "sealed AF01 trace contains a runtime failure")
    complete = only(trace["SmokeComplete"], "AF01 completion")
    equal((complete["detail"], bundle.json("raw/summary.json")),
          (bundle.manifest["summary"], bundle.manifest["summary"]), "sealed recovery summary")
    config = decode_config(bundle.read("raw/config.bin"), "raw/config.bin")
    equal(oracle["config_ref"], config["ref"], "actual execution config")
    l1_events, headers, old_floor = project_l1(bundle, trace, control, report, profile, checked_headers)
    governance = project_governance(bundle, trace, report, control, profile)
    public = project_public(bundle, trace, profile, config, control, report, oracle, headers, old_floor)
    provenance = {"candidate": bundle.manifest["candidate"], "manifest SHA-256": bundle.manifest_hash,
                  "bundle": path.name, "independent": independent.name,
                  **{f"{name} SHA-256": hash for name, hash in hashes.items()},
                  "T_init 来源": "冻结 M3-AUTO-FORCE-1 · 3 s", "checker": "AF01 PASS"}
    return assemble("sealed", [*l1_events, *governance, *public], profile, provenance)
