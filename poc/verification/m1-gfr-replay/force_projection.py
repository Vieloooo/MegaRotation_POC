"""Read the sealed M2 Healthy Force run and compare its public replay evidence."""

import json

from generate import (TRACE_FILE, Bundle, equal, index_unique, public_index, read_trace,
                      trace_anchor, trace_block_ref, trace_hash, trace_ref, trace_subject,
                      validate_coverage, verify_handoff)
from wire import (decode_config, decode_dep, decode_descriptor, decode_espresso, decode_l1,
                  decode_result, domain_hash, require, sha256)


REVISION = "ea4d853bd8b62ee04c4e8e470550648242073950"
BUNDLE_NAME = "merge-ea4d853b-seed42-force-run01"
VERIFY_NAME = BUNDLE_NAME + "-independent"
BUNDLE_DIGEST = "33a4c9d22845fe1150119113d04a56cb3a78759543b50a73064b963dd6561f9c"
PUBLIC_DIGEST = "26779e78d59636825851d9e562cbcf0c293c1b21252fa5915a474929903a9e45"
REPORT_DIGEST = "cc45a98325f9cb3f9b2eec8df2b5b6a626a333d212b1eb4c3a8f631430dcd44d"
CONTROLS_DIGEST = "2358cdc221d896b37f9da929956acfeafb5866889967a545e8c5d9469d51758e"
HUB_FILE = "raw/l1/hub-l1.jsonl"
RPC_FILE = "raw/l1/raw-rpc-bundle.json"
CASES_FILE = "raw/l1/receipts-by-case.json"
CLOSE_KINDS = ("Public(Complete)", "Public(Force)")


class ForceBundle(Bundle):
    def __init__(self, directory):
        self.root = directory
        raw = (directory / "manifest.json").read_bytes()
        equal(sha256(raw), BUNDLE_DIGEST, "sealed Force manifest digest")
        manifest = json.loads(raw)
        equal((manifest["candidate"], manifest["seed"], manifest["status"], manifest["schema"]),
              (REVISION, 42, "COMPLETED", "M2-HEALTHY-FORCE-RUN-1"), "sealed Force manifest")
        self.files = manifest["files"]
        self.digests = {}


def read_force_independent(directory):
    documents = {}
    for name, digest in (("report", REPORT_DIGEST), ("controls", CONTROLS_DIGEST),
                         ("public-replay", PUBLIC_DIGEST)):
        raw = (directory / f"{name}.json").read_bytes()
        equal(sha256(raw), digest, f"independent {name}.json digest")
        documents[name] = json.loads(raw)
    report = documents["report"]
    equal((report["status"], report["mode"], report["candidate"], report["run_manifest_sha256"],
           report["public_sha256"]), ("PASS", "force", REVISION, BUNDLE_DIGEST, PUBLIC_DIGEST),
          "independent report binding")
    return documents


def receipt_position(log):
    return [int(log[k], 16) for k in ("blockNumber", "transactionIndex", "logIndex")]


def project_l1(bundle, trace, fixture, documents, profile, config):
    rpc = json.loads(bundle.read(RPC_FILE))
    cases = json.loads(bundle.read(CASES_FILE))
    hub = [{"line": i, **json.loads(line)} for i, line in enumerate(bundle.read(HUB_FILE).splitlines(), 1)]
    [control] = documents["controls"]
    report = documents["report"]
    tx_hash = control["tx_hash"]
    block, tx, _ = control["position"]
    header = rpc["blocks"][block]
    transaction = header["transactions"][tx]
    receipt_index = next(i for i, r in enumerate(rpc["receipts"]) if r["transactionHash"] == tx_hash)
    receipt = rpc["receipts"][receipt_index]
    [log] = receipt["logs"]
    equal((receipt["blockHash"], receipt["transactionHash"], int(receipt["status"], 16),
           receipt_position(log), log["blockHash"], log["transactionHash"], log["removed"]),
          (header["hash"], transaction["hash"], 1, control["position"], header["hash"],
           tx_hash, False), "successful Force receipt")
    equal(cases["scheduled_healthy_force"], {"hash": tx_hash, "logs": 1, "status": 1}, "receipt case")
    [reported] = [r for r in report["receipts"]["receipt_report"]["events"] if r["kind"] == "ForceRecorded"]
    equal((reported["position"], reported["seq"], reported["act"] * 1_000_000),
          (control["position"], control["seq"], control["act_us"]), "independent ForceRecorded")
    equal(rpc["config"]["configRef"], "0x" + config["ref"], "receipt config")

    included = index_unique(trace["L1Included"], lambda r: r["detail"]["height"], "L1Included")
    finalized = index_unique(trace["L1Finalized"], lambda r: r["detail"]["height"], "L1Finalized")
    readers = index_unique(trace["L1ReaderAvailable"],
                           lambda r: (r["detail"]["height"], r["detail"]["reader"]), "L1ReaderAvailable")
    operations = index_unique((r for r in hub if r["command"]["op"] in {"genesis", "mine", "finalize", "deliver"}),
        lambda r: tuple(r["command"][k] for k in ("op", "height", "reader")), "hub L1 operation")
    equal(len(hub), report["stages"]["checked_hub_operations"], "sealed hub operation count")
    [submission] = [r for r in trace["L1Submitted"] if r["detail"]["case"] == "scheduled_healthy_force"]
    [hub_submit] = [r for r in hub if r["command"]["op"] == "submit"]
    [submitted] = hub_submit["result"]
    equal((submitted["hash"], transaction["input"], hub_submit["command"]["tick_us"]),
          (tx_hash, submission["detail"]["calldata"], submission["tick_us"]), "submitted Force calldatum/tick")
    equal((submission["detail"]["hash"], submitted["calldata"]), (tx_hash, transaction["input"]), "submitted Force transaction")

    rows = []
    previous_hash = "0x" + "00" * 32
    for i, raw in enumerate(rpc["blocks"]):
        ts = int(raw["timestamp"], 16)
        equal((int(raw["number"], 16), raw["parentHash"], ts), (i, previous_hash, fixture[i]["timestamp_s"]),
              f"actual Anvil ℓ{i} history / worker timestamp projection")
        previous_hash = raw["hash"]
        origin = operations[("mine" if i else "genesis", i, "A")]
        equal(origin["result"]["header_and_transactions"], raw, f"Anvil ℓ{i} RPC/Hub header")
        if i:
            observation = included[i]
            equal((observation["detail"]["hash"], observation["detail"]["timestamp_s"], observation["tick_us"]),
                  (raw["hash"], ts, origin["command"]["tick_us"]), f"ℓ{i} inclusion trace")
        final = finalized.get(i)
        final_op = operations.get(("finalize", i, "A"))
        if final:
            equal((final_op["result"]["finalized_through"], final_op["command"]["tick_us"]),
                  (i, final["tick_us"]), f"ℓ{i} finalization")
        available = {}
        for reader in ("A", "B"):
            delivery = operations.get(("deliver", i, reader)) if i else origin
            observed = readers.get((i, reader))
            equal(bool(delivery), bool(observed), f"ℓ{i} reader {reader} delivery presence")
            if delivery:
                equal((delivery["result"]["header_and_transactions"], delivery["command"]["tick_us"],
                       "0x" + trace_hash(observed["detail"]["hash"])),
                      (raw, observed["tick_us"], raw["hash"]), f"ℓ{i} reader {reader} delivery")
                equal(delivery["result"]["events"], origin["result"]["events"], f"ℓ{i} control prefix delivery")
                require(not i or final["tick_us"] <= observed["tick_us"], f"ℓ{i} delivery before finality")
                available[reader] = {"trace": trace_ref(observed), "hub_line": delivery["line"]}
        controls = [e for e in origin["result"]["events"] if e["event"] == "force"]
        rows.append({
            "height": i, "hash": raw["hash"], "parent": raw["parentHash"], "timestamp_s": ts,
            "timestamp_us": ts * 1_000_000, "included": trace_ref(included[i]) if i else None,
            "finalized_us": final["tick_us"] if final else None if i else 0,
            "finalized": trace_ref(final) if final else None, "mine_hub_line": origin["line"],
            "finalize_hub_line": final_op["line"] if final_op else None,
            "readers": available, "controls": controls,
        })
    spot = rows[block]
    [recorded] = spot["controls"]
    equal(sum(len(row["controls"]) for row in rows), 1, "one ForceRecorded in Anvil history")
    equal((recorded["position"]["block"], recorded["position"]["tx"], recorded["position"]["log"],
           recorded["position"]["tx_hash"], recorded["act"] * 1_000_000, recorded["branch"],
           recorded["seq"], recorded["policy"]["l1_floor"]),
          (*control["position"], tx_hash, control["act_us"], control["target"],
           control["seq"], control["l1_floor"]), "decoded ForceRecorded agreement")
    equal(control["act_us"], spot["timestamp_us"] + profile["announcement_us"], "Force activation key")
    stages = {"submit_us": submission["tick_us"], "include_us": spot["included"]["tick_us"],
              "finalize_us": spot["finalized_us"],
              "reader_us": {r: v["trace"]["tick_us"] for r, v in spot["readers"].items()}}
    equal(stages, {k: report["stages"][k] for k in stages}, "independent L1 stages")
    return rows, {
        **control, "receipt_index": receipt_index, "receipt_status": int(receipt["status"], 16),
        "receipt_log_index": 0, "address": log["address"], "topic": log["topics"][0],
        "submit": trace_ref(submission), "submit_hub_line": hub_submit["line"],
        "submit_head": submission["detail"]["head_height"], "request_id": recorded["request_id"],
        "hub_file": HUB_FILE, "rpc_file": RPC_FILE, "cases_file": CASES_FILE,
    }


def espresso_position(row):
    return {"index": {"Espresso": row["q"]}, "key_us": row["key_us"], "kind": "Espresso"}


def fixture_inputs(bundle, config, oracle):
    l1_raw = bundle.read("raw/l1.bin")
    dep_raw = bundle.read("raw/dep.bin")
    equal(domain_hash("PoC/L1Fixture/v1", l1_raw), config["l1_fixture_hash"], "worker L1 fixture binding")
    equal(domain_hash("PoC/DepFixture/v1", dep_raw), config["dep_fixture_hash"], "DEP fixture binding")
    fixture = decode_l1(l1_raw, "raw/l1.bin")
    equal([{"height": r["height"], "hash": r["hash"], "parent_hash": r["parent"],
            "timestamp_s": r["timestamp_s"], "finalized_us": r["finalized_us"]} for r in fixture],
          oracle["l1"], "independent execution fixture")
    return fixture, decode_dep(dep_raw, "raw/dep.bin")


def join_public(bundle, trace, oracle, contexts):
    espresso = bundle.bins("espresso", decode_espresso)
    results = bundle.bins("results", decode_result)
    first_bodies, proofs = public_index(espresso)
    equal([{"q": row["q"], "theta_us": row["key_us"], "finalized_us": row["finalized_us"],
            "proposed_us": row["proposal_us"]} for row in espresso],
          oracle["espresso"], "independent Espresso headers")
    equal((len(espresso), len(results), len(proofs), len(oracle["public_finals"]),
           oracle["signature_checks"], oracle["accepted_final_count"], len(oracle["expected"])),
          (12, 12, 12, 12, 12, 11, 16), "M2 public counts")
    public_finals = index_unique(enumerate(oracle["public_finals"]),
                                lambda row: (row[1]["q"], row[1]["message"]), "oracle public finals")
    regionals = {}
    for observed in trace["RegionalDelivered"]:
        regionals.setdefault(trace_subject(observed), []).append(observed)
    publishers = index_unique(trace["PublisherRecorded"], trace_subject, "PublisherRecorded")
    equal((len(regionals), len(publishers), len(trace["RegionalDelivered"])), (12, 12, 60), "M2 regional counts")
    entries = {}
    for result in results:
        b, d = result["body"], result["descriptor"]
        equal(d, contexts[d["cid"]], "sealed canonical result descriptor")
        key = (d["cid"], b["physical"])
        proof, proof_ref = proofs[key]
        q = proof_ref["q"]
        row = espresso[q]
        pub = publishers[key]
        deliveries = regionals[key]
        equal((proof["descriptor"], proof["proof"]), (d, result["proof"]), "regional/public proof binding")
        parent, ancestry = d["anchor"]["physical"], []
        for physical in result["chain"]:
            public, ref = first_bodies[physical["physical"]]
            equal((physical, physical["parent"]), (public, parent), "public ancestry body/parent")
            ancestry.append({"height": physical["height"], "physical": physical["physical"], **ref})
            parent = physical["physical"]
        q_avail = max(proof_ref["q"], *(ancestor["q"] for ancestor in ancestry))
        proof_index, independent = public_finals[(q, proof_ref["message"])]
        equal((independent["q"], independent["message"], independent["height"], independent["cid"],
               independent["target"], independent["proof"], independent["q_avail"],
               independent["q_avail_theta_us"], independent["session"]),
              (q, proof_ref["message"], b["height"], d["cid"], b["physical"], result["proof"], q_avail,
               espresso[q_avail]["key_us"], {"beta": d["beta"], "r": d["session"]}), "independent public FINAL")
        p = pub["detail"]
        equal((p["height"], p["q"], p["message_index"], p["q_avail"], p["theta_us"],
               p["proposal_tick_us"], pub["tick_us"]),
              (b["height"], q, proof_ref["message"], q_avail, row["key_us"], row["proposal_us"],
               row["finalized_us"]), "PublisherRecorded public position/time")
        equal(sorted(r["pid"] for r in deliveries), list(range(d["region"] * 5, d["region"] * 5 + 5)),
              "regional committee deliveries")
        require(all(r["detail"]["height"] == b["height"] and r["tick_us"] <= pub["tick_us"]
                    for r in deliveries), "regional delivery height/time")
        require(q not in entries, f"duplicate result for q{q}")
        entries[q] = {
            "body": b, "cid": d["cid"], "q": q, "q_avail": q_avail, "position": espresso_position(row),
            "result_file": result["file"], "anchor_evidence": result["anchor_evidence"],
            "ancestry": ancestry, "proof": proof_ref, "oracle": {"proof_index": proof_index},
            "deliveries": list(map(trace_ref, deliveries)), "publisher": trace_ref(pub), "handoff": None,
        }
    return espresso, [entries[q] for q in range(len(espresso))]


def verify_assignment(assignment, new):
    plan = assignment["plan"]
    equal((trace_hash(assignment["cid"]), trace_anchor(assignment["anchor"]), plan["cause"],
           plan["session"], plan["region"], plan["source_region"], trace_block_ref(plan["reference_block"]),
           plan["window"], trace_hash(plan["config_ref"]), trace_hash(plan["authorization_ref"])),
          (new["cid"], new["anchor"], "Force", {"beta": new["beta"], "r": new["session"]}, new["region"],
           new["source_region"], new["reference"], {"h_min": new["window"][0], "h_max": new["window"][1]},
           new["config_ref"], new["authorization_ref"]), "Force SessionAssigned")


def project_force(trace, contexts, entries, espresso, l1, control, oracle, report, config, profile, ordered):
    [activated] = trace["ForceActivated"]
    [record] = [r for r in ordered if r["detail"]["position"]["kind"] == "Force"]
    detail, summarized = record["detail"], activated["detail"]
    genesis, old, new = contexts
    [justified] = oracle["force"]
    x = control["act_us"]
    q_star = next(r["q"] for r in espresso if r["key_us"] >= x)
    boundary = entries[q_star - 1]
    anchor = {k: boundary["body"][k] for k in ("height", "origin", "physical", "timestamp_us")}
    ref = {k: anchor[k] for k in ("height", "origin", "physical")}
    slot = (x + profile["force"]["force_init_us"] - config["genesis"]["timestamp_us"]) // config["l2_slot_us"]
    h_min = max(anchor["height"] + 1, slot + 1)
    window = [h_min, h_min + old["window"][1] - old["window"][0]]
    # Regular replay preserves the floor initialized from the genesis origin.
    old_floor = config["genesis"]["origin"]
    floor = max(old_floor, control["position"][0])
    equal((new["anchor"], new["reference"], new["beta"], new["session"], new["region"],
           new["source_region"], new["window"], old["beta"], old["session"], old["region"]),
          (anchor, ref, old["beta"] + 1, old["session"], (old["beta"] + 1 + old["session"]) % 3,
           old["region"], window, control["target"], 1, 1), "Force duty from currently selected R1")
    block, tx, log = control["position"]
    position = {"kind": "Force", "key_us": x, "index": {"L1": {"block": block, "tx": tx, "log": log}}}
    equal((detail["position"], detail["append"], summarized["position"],
           summarized["old_branch"], summarized["new_branch"], summarized["head_height"],
           summarized["l1_floor"], summarized["espresso_q"], trace_hash(summarized["record_id"])),
          (position, [], position, old["beta"], new["beta"], anchor["height"], floor, q_star,
           trace_hash(detail["id"])), "Force replay keeps G head")
    require(len(detail["event_ids"]) == 2, "two Force lifecycle ids")
    close, assign = detail["events"]
    equal((close["body"], close["position"], assign["position"]),
          ({"BranchClosed": {"branch": old["beta"], "reason": "Force"}}, position, position),
          "Force branch closure")
    verify_assignment(assign["body"]["SessionAssigned"]["descriptor"], new)
    verify_assignment(summarized["descriptor"], new)
    equal((trace_anchor(justified["anchor"]), trace_hash(justified["native_hash"]),
           trace_hash(justified["anchor_root"]), justified["valid"], justified["closed_beta"],
           justified["beta"], justified["r"], justified["region"], justified["source_region"],
           [justified["h_min"], justified["h_max"]], justified["q_star"], justified["position"],
           justified["seq"], justified["act_us"], justified["floor"]),
          (anchor, trace_hash(summarized["anchor_native_hash"]), boundary["state_root"], True, old["beta"],
           new["beta"], new["session"], new["region"], old["region"], window, q_star,
           control["position"], control["seq"], x, floor), "independent Force state and duty")
    equal(trace_hash(summarized["anchor_native_hash"]), boundary["native"], "activation native anchor")
    equal(report["force"]["activation"], {"key_us": x, "q_star": q_star, "tick_us": record["tick_us"],
                                          "trace_seq": record["seq"]}, "independent Force processing")
    equal((record["tick_us"], activated["tick_us"]), (espresso[q_star]["finalized_us"],) * 2,
          "Force processing tick from first Espresso watermark")
    require(l1[block]["readers"]["A"]["trace"]["tick_us"] <= record["tick_us"] and
            l1[anchor["origin"]]["readers"]["A"]["trace"]["tick_us"] <= record["tick_us"],
            "Force control coverage and anchor metadata")
    require(boundary["handoff"]["trace"]["seq"] < record["seq"] < activated["seq"], "in-flight Regular precedes Force")
    equal((control["submit_head"], control["submit"]["tick_us"] < boundary["replay"]["tick_us"]),
          (1, True), "Force submitted during R0 production")
    closes = []
    for ctx, reason in zip((genesis, old), CLOSE_KINDS):
        relevant = [r for r in trace["Close"] if trace_hash(r["detail"]["cid"]) == ctx["cid"]]
        equal(sorted(r["pid"] for r in relevant), list(range(5 * ctx["region"], 5 * ctx["region"] + 5)),
              f"R{ctx['region']} local closures")
        equal([r["detail"]["cause"] for r in relevant], [reason] * 5, "worker closure reasons")
        expected_tick = boundary["replay"]["tick_us"] if reason == CLOSE_KINDS[0] else record["tick_us"]
        equal([r["tick_us"] for r in relevant], [expected_tick] * 5, "worker closure ticks")
        closes.append({"cid": ctx["cid"], "region": ctx["region"], "reason": reason,
                       "rows": list(map(trace_ref, relevant))})
    return {
        "position": position, "q_star": q_star, "old": old["cid"], "new": new["cid"],
        "anchor": anchor, "anchor_native": boundary["native"], "anchor_state": boundary["state_root"],
        "closes": closes, "record_id": trace_hash(detail["id"]), "floor": floor, "old_floor": old_floor, "window": window,
        "init_us": profile["force"]["force_init_us"], "slot": slot,
        "replay": trace_ref(record), "activated": trace_ref(activated), "event_ids": list(map(trace_hash, detail["event_ids"])),
        "head_height": anchor["height"],
    }


def attach_acceptance(entry, context, accept, replay, virtual, pad_trace, last, config, l1, dep, profile, espresso):
    body = entry["body"]
    height = body["height"]
    q = entry["q"]
    first = last["height"] + 1
    detail, record = accept["detail"], replay["detail"]
    position = entry["position"]
    equal((detail["height"], trace_hash(detail["parent"]), detail["position"], record["position"],
           record["append"]),
          (height, last["physical"], position, position,
           [{"cid": detail["cid"], "physical_id": detail["physical_id"], "first": first, "last": height}]),
          f"q{q} accepted virtual interval")
    equal((accept["tick_us"], replay["tick_us"]), (entry["publisher"]["tick_us"],) * 2, "public commit tick")
    require(entry["publisher"]["seq"] < accept["seq"] < replay["seq"], "public commit trace order")
    padding = []
    parent_origin = last["origin"]
    for h in range(first, height + 1):
        expected = virtual[h - 1]
        is_padding = h < height
        observed = pad_trace[h] if is_padding else accept
        item = observed["detail"]
        native, state = (trace_hash(item[k]) for k in ("native_hash", "state_root"))
        equal((expected["height"], expected["physical"], expected["parent"], expected["native"],
               expected["state_root"], expected["target"], expected["anchor"], expected["cid"],
               expected["beta"], expected["r"], expected["region"], expected["q"], expected["q_avail"],
               expected["theta_us"]),
              (h, None if is_padding else body["physical"], None if is_padding else body["parent"],
               native, state, body["physical"], context["anchor"]["physical"], entry["cid"], context["beta"],
               context["session"], context["region"], q, entry["q_avail"], position["key_us"]),
              f"h{h} independent virtual G row")
        equal((item["height"], trace_subject(observed), item["position"], observed["tick_us"]),
              (h, (entry["cid"], body["physical"]), position, replay["tick_us"]), f"h{h} trace source")
        if is_padding:
            require(entry["publisher"]["seq"] < observed["seq"] < accept["seq"], "atomic padding trace order")
            ts = config["genesis"]["timestamp_us"] + h * config["l2_slot_us"]
            origin_at = (ts - config["l1_genesis_timestamp_us"] - config["t_dep_us"]) // config["l1_block_us"]
            base_origin = context["anchor"]["origin"]
            origin = min(base_origin + h - context["anchor"]["height"], max(base_origin, origin_at))
            require(0 <= origin - parent_origin <= 1, f"padding h{h} origin continuity")
            parent_origin = origin
            padding.append({"height": h, "physical": None, "parent": None, "proof_target": body["physical"],
                            "native": native, "state_root": state, "trace": trace_ref(observed),
                            "expected_index": h - 1, "origin": origin,
                            "expiry_us": ts + profile["expiry_us"]})
    coverage = validate_coverage({"body": body, "descriptor": context}, config, l1, profile,
                                 dep[height], espresso[q], espresso[entry["q_avail"]]["key_us"],
                                 {"origin": parent_origin})
    earliest_expiry = min(row["expiry_us"] for row in padding) if padding else coverage["expiry_us"]
    require(position["key_us"] < earliest_expiry, "whole virtual interval remains unexpired")
    for l1_height in {coverage["l1_control_height"], coverage["l1_origin_height"],
                      coverage["l1_anchor_origin_height"], coverage["l1_reference_origin_height"],
                      *(item["origin"] for item in padding)}:
        require(l1[l1_height]["readers"]["A"]["trace"]["tick_us"] <= replay["tick_us"], "reader A coverage")
    entry.update({
        "native": trace_hash(detail["native_hash"]), "state_root": trace_hash(detail["state_root"]),
        "record_id": trace_hash(record["id"]), "append": [first, height], "padding": padding,
        "earliest_expiry_us": earliest_expiry, "accept": trace_ref(accept), "replay": trace_ref(replay),
        "coverage": coverage,
    })
    entry["oracle"]["expected_index"] = height - 1


def build_force_projection(bundle, documents):
    oracle, report = documents["public-replay"], documents["report"]
    config = decode_config(bundle.read("raw/config.bin"), "raw/config.bin")
    profile = json.loads(bundle.read("raw/profile.json"))
    equal((profile["schema"], profile["seed"], oracle["config_ref"], oracle["halted"]),
          ("M2-HEALTHY-FORCE-1", 42, config["ref"], False), "Force profile")
    fixture, dep = fixture_inputs(bundle, config, oracle)
    trace = read_trace(bundle.read(TRACE_FILE), ("L1Submitted", "L1Included", "L1Finalized", "L1ReaderAvailable",
        "ForceActivated", "LatePublicationHeld", "LatePublicationExcluded", "GPadding", "Close"))
    equal(bundle.digests[TRACE_FILE], report["raw_trace_sha256"], "independent raw trace binding")
    l1, control = project_l1(bundle, trace, fixture, documents, profile, config)
    context_files = ("raw/genesis-descriptor.bin", "raw/descriptor-r1.bin", "raw/descriptor-b1-r1.bin")
    contexts = [decode_descriptor(bundle.read(path), path) for path in context_files]
    equal([(c["beta"], c["session"], c["region"], c["cause"], c["config_ref"]) for c in contexts],
          [(0, 0, 0, 0, config["ref"]), (0, 1, 1, 1, config["ref"]), (1, 1, 2, 2, config["ref"])],
          "Force canonical descriptors")
    genesis, old, new = contexts
    context_by_id = {d["cid"]: d for d in contexts}
    espresso, entries = join_public(bundle, trace, oracle, context_by_id)
    equal((genesis["anchor"], genesis["window"]), (config["genesis"], [1, 10]), "genesis context")
    [record_late] = trace["LatePublicationExcluded"]
    [held] = trace["LatePublicationHeld"]
    [handoff] = trace["Handoff"]
    [public_regular] = oracle["regular"]
    [late] = oracle["late"]
    late_q = late["q_avail"]
    equal(late, report["force"]["late"][0], "independent old result exclusion")
    equal(len(trace["GfrReplayRecord"]), len(espresso) + 1, "all completed replay records")
    records = sorted(trace["GfrReplayRecord"], key=lambda r: r["seq"])
    equal([r["detail"]["source_seq"] for r in records], list(range(len(records))), "contiguous public replay sequence")
    equal([r["detail"]["position"] for r in records if r["detail"]["position"]["kind"] == "Espresso"],
          [e["position"] for e in entries], "complete Espresso replay sequence")
    replays = {r["detail"]["position"]["index"]["Espresso"]: r for r in records
               if r["detail"]["position"]["kind"] == "Espresso"}
    accepts = index_unique(trace["GAccepted"], trace_subject, "GAccepted")
    pad = index_unique(trace["GPadding"], lambda r: r["detail"]["height"], "GPadding")
    equal((len(accepts), len(pad)), (11, 5), "11 physical and 5 padding G rows")
    last, state = config["genesis"], config["genesis_state_root"]
    for q, entry in enumerate(entries):
        body = entry["body"]
        d = context_by_id[entry["cid"]]
        require(d["window"][0] <= body["height"] <= d["window"][1], "physical height in descriptor window")
        key = (entry["cid"], body["physical"])
        replay = replays[q]
        require(espresso[q]["finalized_us"] - espresso[q]["proposal_us"] ==
                profile["espresso_finalize_delay_us"], "Espresso finalized delay")
        if q == late_q:
            equal((late["height"], late["cid"], late["physical"], entry["q_avail"], d),
                  (body["height"], *key, q, old), "late old branch identity")
            equal((trace_subject(held), held["detail"]["height"], trace_subject(record_late),
                   record_late["detail"]["q"], record_late["detail"]["head_height"],
                   record_late["detail"]["height"], record_late["detail"]["branch"]),
                  (key, body["height"], key, q, last["height"], body["height"], new["beta"]),
                  "late publication held then excluded")
            equal((replay["detail"]["append"], replay["detail"]["events"], replay["detail"]["event_ids"]),
                  ([], [], []), "same-key excluded result appends nothing")
            equal((replay["tick_us"], record_late["tick_us"]), (entry["publisher"]["tick_us"],) * 2,
                  "late result commit tick")
            require(held["tick_us"] < record_late["tick_us"] and replay["seq"] < record_late["seq"] and key not in accepts,
                    "old branch has one exclusion and zero acceptance")
            entry.update({"append": [], "replay": trace_ref(replay), "held": trace_ref(held),
                          "excluded": trace_ref(record_late), "record_id": trace_hash(replay["detail"]["id"])})
            continue

        physical_height = body["height"]
        equal(d, genesis if q < late_q else new, "G only extends selected descriptor")
        equal(body["parent"], last["physical"], "accepted physical continuation")
        previous_row = oracle["expected"][physical_height - 2] if physical_height > 1 else None
        equal(body["pre_state_root"], previous_row["state_root"] if previous_row else state,
              "physical input state from preceding virtual G row")
        attach_acceptance(entry, d, accepts[key], replay, oracle["expected"], pad, last, config, l1,
                          dep, profile, espresso)
        if physical_height == genesis["window"][1]:
            entry["handoff"] = verify_handoff(d, old, entry, replay, handoff, context_files[1])
            equal((public_regular["cause"], public_regular["beta"], public_regular["r"], public_regular["q"],
                   public_regular["old_max"], public_regular["key_us"], trace_anchor(public_regular["anchor"])),
                  ("FullWindow", d["beta"], d["session"], q, physical_height,
                   espresso[q]["key_us"], entry["handoff"]["anchor"]), "independent Regular transition")
        else:
            equal((replay["detail"]["events"], replay["detail"]["event_ids"]), ([], []), "no lifecycle on ordinary G append")
        last, state = body, entry["state_root"]

    force = project_force(trace, contexts, entries, espresso, l1, control, oracle, report, config, profile, records)
    old_entry = entries[late_q]
    equal((force["q_star"], force["position"]["key_us"], force["replay"]["tick_us"]),
          (late_q, old_entry["position"]["key_us"], old_entry["replay"]["tick_us"]), "same-key Force/Espresso")
    require(old_entry["publisher"]["seq"] < force["replay"]["seq"] < force["activated"]["seq"] <
            old_entry["replay"]["seq"] < old_entry["excluded"]["seq"], "observed same-tick replay order")
    [completion] = trace["SmokeComplete"]
    head = entries[-1]
    equal(completion["detail"], json.loads(bundle.read("raw/summary.json")), "sealed run summary")
    equal((completion["detail"]["position"], completion["detail"]["height"],
           trace_subject(completion), completion["tick_us"], report["force"]["first_new_g_tick_us"]),
          (head["position"], 16, (head["cid"], head["body"]["physical"]),
           head["replay"]["tick_us"], head["replay"]["tick_us"]), "Force run completion")
    equal(report["force"]["first_new"], oracle["expected"][-1], "first new physical G row")
    return {
        "schema": "m2-healthy-force-gfr-replay-v1",
        "source": {"revision": REVISION, "seed": profile["seed"], "bundle": BUNDLE_NAME, "independent": VERIFY_NAME,
                   "manifest_sha256": BUNDLE_DIGEST, "public_replay_sha256": PUBLIC_DIGEST,
                   "report_sha256": REPORT_DIGEST, "controls_sha256": CONTROLS_DIGEST,
                   "input_sha256": bundle.digests, "trace_file": TRACE_FILE},
        "profile": {k: profile[k] for k in ("announcement_us", "expiry_us", "grace_us",
                                            "espresso_finalize_delay_us")},
        "config": config, "contexts": contexts, "context_files": context_files,
        "l1": l1, "worker_l1": fixture, "control": control, "force": force,
        "espresso": espresso, "entries": entries, "end_us": completion["tick_us"],
        "completion": trace_ref(completion),
    }
