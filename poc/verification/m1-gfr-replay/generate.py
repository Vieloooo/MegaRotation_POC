#!/usr/bin/env python3
"""Build/check the self-contained HTML for the sealed M1 Regular smoke."""

import argparse
import json
import sys
from pathlib import Path

from wire import (
    decode_config, decode_dep, decode_descriptor, decode_espresso,
    decode_l1, decode_result, domain_hash, require, sha256,
)


REVISION = "9154785732d803c1f2f69aee3b2af67220a12f58"
BUNDLE_DIGEST = "4e76e4eeb8e41e801db9689441b6fd82b94b7060b6ad2dbb47034e695e9f78ca"
PUBLIC_DIGEST = "641597c9438589efdfa1e4e1f3d99b24b8663656fcf452370738011528b2d6b6"
BUNDLE_NAME = "healthy-smoke-91547857-seed42-run01"
VERIFY_NAME = "independent-verification-91547857-seed42-run01"
TRACE_FILE = "raw/trace.jsonl"
PAGE_DIR = Path(__file__).resolve().parents[2] / "web" / "m1-gfr-replay"


class Bundle:
    def __init__(self, directory):
        self.root = directory
        manifest_bytes = (directory / "manifest.json").read_bytes()
        equal(sha256(manifest_bytes), BUNDLE_DIGEST, "sealed smoke manifest digest")
        manifest = json.loads(manifest_bytes)
        equal((manifest["candidate"], manifest["seed"], manifest["status"]),
              (REVISION, 42, "COMPLETED"), "sealed smoke manifest")
        self.files = {entry["path"]: entry for entry in manifest["artifacts"]}
        self.digests = {}

    def read(self, relative):
        record = self.files.get(relative)
        require(record is not None, f"{relative}: missing from smoke manifest")
        data = (self.root / relative).read_bytes()
        digest = sha256(data)
        equal((len(data), digest), (record["bytes"], record["sha256"]), relative)
        self.digests[relative] = digest
        return data

    def bins(self, family, decoder):
        prefix = f"raw/{family}/"
        names = sorted(path for path in self.files if path.startswith(prefix) and path.endswith(".bin"))
        return [decoder(self.read(path), path) for path in names]


def equal(actual, expected, label):
    require(actual == expected, f"{label}: differs from expected evidence")


def read_independent(directory):
    data = (directory / "public-replay.json").read_bytes()
    report = json.loads((directory / "report.json").read_bytes())
    equal((report["status"], report["source_revision"], report["seed"],
           report["bundle_manifest_sha256"], report["output_files"]["public-replay.json"]),
          ("PASS", REVISION, 42, BUNDLE_DIGEST, PUBLIC_DIGEST), "independent report binding")
    equal(sha256(data), PUBLIC_DIGEST, "sealed independent public-replay.json")
    return json.loads(data)


def trace_hash(value):
    require(isinstance(value, list) and len(value) == 32, "trace hash width")
    return bytes(value).hex()


def trace_ref(row):
    return {name: row[name] for name in ("kind", "line", "seq", "tick_us", "pid", "event_id")}


def trace_anchor(value):
    return {**trace_block_ref(value["block"]), "timestamp_us": value["timestamp_us"]}


def trace_block_ref(value):
    return {"height": value["height"], "origin": value["origin"], "physical": trace_hash(value["block_id"])}


def trace_subject(row):
    return (trace_hash(row["detail"]["cid"]), trace_hash(row["detail"]["physical_id"]))


def read_trace(data):
    kinds = ("RegionalDelivered", "PublisherRecorded", "GAccepted", "GfrReplayRecord", "Handoff",
             "SmokeComplete")
    groups = {kind: [] for kind in kinds}
    tick = 0
    for line, raw in enumerate(data.splitlines(), 1):
        row = json.loads(raw)
        equal(row["seq"], line, f"{TRACE_FILE}:{line} seq")
        require(row["tick_us"] >= tick, f"{TRACE_FILE}:{line} virtual time decreases")
        tick = row["tick_us"]
        kind = row["kind"]
        if kind in groups:
            groups[kind].append({"line": line, **row})
    return groups


def index_unique(rows, key, label):
    index = {}
    for row in rows:
        value = key(row)
        require(value not in index, f"{label}: duplicate key {value}")
        index[value] = row
    return index


def public_index(espresso):
    first_bodies, proofs = {}, {}
    for q, row in enumerate(espresso):
        equal(row["q"], q, "contiguous Espresso q")
        if q:
            previous = espresso[q - 1]
            equal(row["source"], previous["source"], f"q{q} source")
            require(row["key_us"] >= previous["key_us"] and
                    row["finalized_us"] >= previous["finalized_us"], f"q{q} ordering")
        for msg in row["messages"]:
            ref = {"q": q, "message": msg["index"]}
            if msg["kind"] == "body":
                first_bodies.setdefault(msg["body"]["physical"], (msg["body"], ref))
            else:
                key = msg["descriptor"]["cid"], msg["target"]
                require(key not in proofs, f"q{q}: more than one proof for result")
                proofs[key] = (msg, ref)
    return first_bodies, proofs


def compare_oracle(result, proof_ref, expected, public_final, q_avail, q, key_us):
    b, d = result["body"], result["descriptor"]
    equal(
        (public_final["cid"], public_final["target"], public_final["proof"],
         public_final["q"], public_final["message"]),
        (d["cid"], b["physical"], result["proof"], proof_ref["q"], proof_ref["message"]),
        f"h{b['height']} exact public proof",
    )
    equal(
        (expected["anchor"], expected["cid"], expected["height"], expected["parent"],
         expected["physical"], expected["q"], expected["q_avail"], expected["region"],
         expected["session"], expected["theta_us"], expected["proof_message"],
         expected["target_height"], expected["final_view"], expected["signers"]),
        (d["anchor"]["physical"], d["cid"], b["height"], b["parent"], b["physical"],
         q, q_avail, d["region"], d["session"], key_us,
         [proof_ref["q"], proof_ref["message"]], b["height"], public_final["view"], public_final["signers"]),
        f"h{b['height']} independent public replay row",
    )


def verify_handoff(old, new, entry, replay, handoff, descriptor_file):
    row = handoff["detail"]
    body = entry["body"]
    label = f"h{body['height']} regular handoff"
    selected_anchor = {key: body[key] for key in ("height", "origin", "physical", "timestamp_us")}
    start = old["window"][1] + 1
    length = old["window"][1] - old["window"][0] + 1
    equal(new["anchor"], selected_anchor, label + " new anchor")
    equal(new["reference"], {k: old["anchor"][k] for k in ("height", "origin", "physical")},
          label + " Plan reference")
    equal((new["window"], new["cause"], new["beta"], new["session"], new["source_region"]),
          ([start, start + length - 1], 1, old["beta"], old["session"] + 1, old["region"]),
          label + " successor")
    equal((trace_anchor(row["anchor"]), trace_block_ref(row["reference_block"]),
           trace_hash(row["old_cid"]), trace_hash(row["new_cid"]), trace_hash(row["record_id"]),
           row["window"], row["position"], handoff["tick_us"]),
          (selected_anchor, new["reference"], old["cid"], new["cid"], entry["record_id"],
           {"h_min": new["window"][0], "h_max": new["window"][1]}, replay["detail"]["position"],
           replay["tick_us"]), label + " trace summary")
    events = replay["detail"]["events"]
    require(len(events) == 2 and len(replay["detail"]["event_ids"]) == 2, label + " lifecycle len")
    equal(events[0]["body"]["SessionClosed"], {"reason": "Complete",
          "session": {"beta": old["beta"], "r": old["session"]}}, label + " closed")
    assignment = events[1]["body"]["SessionAssigned"]["descriptor"]
    assigned_plan = assignment["plan"]
    equal((trace_hash(assignment["cid"]), trace_anchor(assignment["anchor"]),
           assigned_plan["region"], assigned_plan["session"], assigned_plan["cause"],
           assigned_plan["source_region"], trace_block_ref(assigned_plan["reference_block"]),
           assigned_plan["window"], trace_hash(assigned_plan["config_ref"]),
           trace_hash(assigned_plan["authorization_ref"])),
          (new["cid"], selected_anchor, new["region"], {"beta": new["beta"], "r": new["session"]},
           "Handoff", new["source_region"], new["reference"], row["window"], new["config_ref"],
           new["authorization_ref"]), label + " assigned")
    equal([event["position"] for event in events], [row["position"]] * 2, label + " atomic position")
    equal(handoff["event_id"], replay["event_id"], label + " event id")
    require(handoff["seq"] > replay["seq"], label + " summary order")
    return {
        "old": old["cid"], "new": new["cid"], "anchor": selected_anchor,
        "reference": new["reference"], "window": new["window"], "descriptor_file": descriptor_file,
        "trace": trace_ref(handoff), "event_ids": list(map(trace_hash, replay["detail"]["event_ids"])),
    }


def validate_coverage(result, config, l1, profile, contents, row, public_key, last_body):
    b, d = result["body"], result["descriptor"]
    x = row["key_us"]
    threshold = x - profile["announcement_us"]
    control = next((header for header in l1 if header["timestamp_us"] >= threshold), None)
    require(control is not None and control["finalized_us"] <= row["finalized_us"], "L1 control coverage")
    origin = l1[b["origin"]]
    reference = l1[d["reference"]["origin"]]
    anchor_header = l1[d["anchor"]["origin"]]
    l2_ts = config["genesis"]["timestamp_us"] + (
        b["height"] - config["genesis"]["height"]
    ) * config["l2_slot_us"]
    scheduled_origin = (l2_ts - config["l1_genesis_timestamp_us"] - config["t_dep_us"]) // config["l1_block_us"]
    expected_origin = min(d["anchor"]["origin"] + b["height"] - d["anchor"]["height"],
                          max(d["anchor"]["origin"], scheduled_origin))
    drift = l2_ts - origin["timestamp_us"]
    equal((b["origin"], b["timestamp_us"]), (expected_origin, l2_ts), "fixed regional metadata")
    require(0 <= b["origin"] - last_body["origin"] <= 1, "origin continuity")
    require(0 <= drift <= config["t_drift_us"], "origin not-ahead/drift")
    require(all(w["finalized_us"] <= row["finalized_us"] for w in (origin, reference, anchor_header)),
            "origin/Plan fixture finality")
    end = l2_ts + (d["window"][1] + 1 - b["height"]) * config["l2_slot_us"] + profile["grace_us"]
    expiry = l2_ts + profile["expiry_us"]
    require(public_key < end and x < end, "publication cutoff or prior SessionEnd")
    require(x < expiry, "virtual height expiry")
    return {
        "threshold_us": threshold, "l1_control_height": control["height"],
        "l1_origin_height": origin["height"], "l1_anchor_origin_height": anchor_header["height"],
        "l1_reference_origin_height": reference["height"], "drift_us": drift,
        "dep": contents, "session_end_us": end, "expiry_us": expiry,
    }


def build_projection(bundle, independent):
    config = decode_config(bundle.read("raw/config.bin"), "raw/config.bin")
    profile = json.loads(bundle.read("raw/profile.json"))
    equal((profile["schema"], profile["seed"]), ("M1-HEALTHY-SMOKE-1", 42), "smoke profile")
    l1_raw, dep_raw = bundle.read("raw/l1.bin"), bundle.read("raw/dep.bin")
    equal(domain_hash("PoC/L1Fixture/v1", l1_raw), config["l1_fixture_hash"], "L1 fixture config binding")
    equal(domain_hash("PoC/DepFixture/v1", dep_raw), config["dep_fixture_hash"], "DEP config binding")
    l1, dep = decode_l1(l1_raw, "raw/l1.bin"), decode_dep(dep_raw, "raw/dep.bin")
    for header in l1:
        equal(header["timestamp_us"], config["l1_genesis_timestamp_us"] +
              header["height"] * config["l1_block_us"], "L1 fixture timestamp grid")
    espresso = bundle.bins("espresso", decode_espresso)
    results = bundle.bins("results", decode_result)
    trace = read_trace(bundle.read(TRACE_FILE))
    first_bodies, proofs = public_index(espresso)
    expected, independent_finals = independent["expected"], independent["public_finals"]
    equal((len(espresso), len(results), len(proofs), len(expected), len(independent_finals),
           independent["independently_verified_public_finalizations"]), (21,) * 6, "M1 scope")
    equal(tuple(map(len, trace.values())), (105, 21, 21, 21, 2, 1), "M1 trace counts")
    independent_indices = index_unique(range(len(independent_finals)),
        lambda i: (independent_finals[i]["q"], independent_finals[i]["message"]), "independent finals")

    regionals = {}
    for item in trace["RegionalDelivered"]:
        regionals.setdefault(trace_subject(item), []).append(item)
    pubs = index_unique(trace["PublisherRecorded"], trace_subject, "PublisherRecorded")
    accepts = index_unique(trace["GAccepted"], trace_subject, "GAccepted")
    replays = index_unique(trace["GfrReplayRecord"], lambda r: r["detail"]["source_seq"], "GfrReplayRecord")
    handoffs = index_unique(trace["Handoff"], lambda r: r["detail"]["position"]["index"]["Espresso"], "Handoff")

    context_files = ("raw/genesis-descriptor.bin", "raw/descriptor-r1.bin", "raw/descriptor-r2.bin")
    contexts = [decode_descriptor(bundle.read(path), path) for path in context_files]
    for i, c in enumerate(contexts):
        equal((c["beta"], c["session"], c["region"], c["config_ref"]),
              (0, i, i, config["ref"]), f"R{i} descriptor identity")
    equal(contexts[0]["anchor"], config["genesis"], "genesis context anchor")
    equal(contexts[0]["window"], [1, 10], "genesis context window")
    equal(contexts[0]["cause"], 0, "genesis context cause")
    equal(contexts[0]["reference"], {k: config["genesis"][k] for k in ("height", "origin", "physical")},
          "genesis context Plan reference")
    selected, last_body, last_state = contexts[0], config["genesis"], config["genesis_state_root"]
    projected = []
    for i, result in enumerate(results):
        height, q = i + 1, i
        b, d, row = result["body"], result["descriptor"], espresso[q]
        equal(d, selected, f"h{height} selected NEXT descriptor")
        equal((b["height"], b["parent"], b["pre_state_root"]),
              (height, last_body["physical"], last_state), f"h{height} contiguous G extension")
        h_min, h_max = d["window"]
        require(h_min <= height <= h_max, f"h{height} selected window")
        k = d["cid"], b["physical"]
        deliveries, pub, accept, replay = regionals[k], pubs[k], accepts[k], replays[q]
        msg, proof_ref = proofs[k]
        p, a, r = pub["detail"], accept["detail"], replay["detail"]
        equal((msg["descriptor"], msg["proof"], proof_ref["q"]), (d, result["proof"], q),
              f"h{height} regional/public result match")
        equal(sorted(x["pid"] for x in deliveries),
              list(range(d["region"] * 5, d["region"] * 5 + 5)), f"h{height} regional emitters")
        require(all(x["detail"]["height"] == height for x in deliveries), f"h{height} regional height")
        require(deliveries[0]["tick_us"] <= row["proposal_us"], f"h{height} first regional delivery")
        parent, ancestry = d["anchor"]["physical"], []
        for chain_height, block in enumerate(result["chain"], d["anchor"]["height"] + 1):
            public_body, ref = first_bodies[block["physical"]]
            equal((block, block["parent"], block["height"]), (public_body, parent, chain_height),
                  f"h{height} anchored public ancestry")
            ancestry.append({"height": block["height"], "physical": block["physical"], **ref})
            parent = block["physical"]
        q_avail = max(proof_ref["q"], *(ancestor["q"] for ancestor in ancestry))
        proof_index = independent_indices[(proof_ref["q"], proof_ref["message"])]
        compare_oracle(result, proof_ref, expected[i], independent_finals[proof_index],
                       q_avail, q, row["key_us"])
        equal((p["height"], p["q"], p["q_avail"], p["theta_us"], p["proposal_tick_us"], p["message_index"]),
              (height, q, q_avail, row["key_us"], row["proposal_us"], proof_ref["message"]),
              f"h{height} publication trace")
        position = {"index": {"Espresso": q}, "key_us": row["key_us"], "kind": "Espresso"}
        equal((a["height"], trace_hash(a["parent"]), a["position"], r["position"], r["append"]),
              (height, b["parent"], position, position,
               [{"cid": a["cid"], "physical_id": a["physical_id"], "first": height, "last": height}]),
              f"h{height} G acceptance trace")
        equal((trace_hash(a["native_hash"]), trace_hash(a["state_root"])),
              (expected[i]["native"], expected[i]["state_root"]), f"h{height} G execution vs independent")
        equal((pub["event_id"], accept["event_id"]), (replay["event_id"],) * 2,
              f"h{height} publication/acceptance event id")
        require(pub["seq"] < accept["seq"] < replay["seq"], f"h{height} trace observation order")
        equal((pub["tick_us"], accept["tick_us"], replay["tick_us"]),
              (row["finalized_us"],) * 3, f"h{height} finalization/trace link")
        equal(row["finalized_us"] - row["proposal_us"], profile["espresso_finalize_delay_us"],
              f"h{height} espresso profile")
        equal(q_avail, q, f"h{height} single result at completion")
        entry = {
            "body": b, "cid": d["cid"], "q": q, "q_avail": q_avail, "position": position,
            "native": trace_hash(a["native_hash"]), "state_root": trace_hash(a["state_root"]),
            "record_id": trace_hash(r["id"]), "append": [height, height], "result_file": result["file"],
            "anchor_evidence": result["anchor_evidence"], "ancestry": ancestry, "proof": proof_ref,
            "oracle": {"expected_index": i, "proof_index": proof_index},
            "deliveries": list(map(trace_ref, deliveries)),
            "publisher": trace_ref(pub), "accept": trace_ref(accept), "replay": trace_ref(replay),
            "coverage": validate_coverage(result, config, l1, profile, dep[height], row,
                                          espresso[q_avail]["key_us"], last_body),
            "handoff": None,
        }
        if height == h_max:
            new = contexts[d["session"] + 1]
            entry["handoff"] = verify_handoff(d, new, entry, replay, handoffs[q], context_files[new["session"]])
            selected = new
        else:
            equal((r["event_ids"], r["events"], q in handoffs), ([], [], False), f"h{height} no handoff")
        projected.append(entry)
        last_body, last_state = b, entry["state_root"]
    smoke = trace["SmokeComplete"][0]
    end_us = smoke["tick_us"]
    equal(end_us, projected[-1]["replay"]["tick_us"], "final observation tick")
    equal((smoke["detail"]["height"], smoke["detail"]["seed"], smoke["detail"]["position"],
           trace_hash(smoke["detail"]["physical_id"])),
          (21, profile["seed"], projected[-1]["position"], projected[-1]["body"]["physical"]), "run endpoint")
    return {
        "schema": "m1-regular-gfr-replay-v1", "source": {
            "revision": REVISION, "bundle": BUNDLE_NAME, "independent": VERIFY_NAME,
            "manifest_sha256": BUNDLE_DIGEST, "public_replay_sha256": PUBLIC_DIGEST,
            "trace_file": TRACE_FILE, "input_sha256": bundle.digests, "seed": profile["seed"],
        },
        "profile": {key: profile[key] for key in ("announcement_us", "expiry_us", "grace_us",
                                                   "espresso_finalize_delay_us")},
        "config": config, "contexts": contexts, "context_files": context_files,
        "l1": l1, "espresso": espresso, "entries": projected, "end_us": end_us,
        "completion": trace_ref(smoke),
    }


def render_html(projected):
    template = (PAGE_DIR / "template.html").read_text(encoding="utf-8")
    data = json.dumps(projected, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    data = data.replace("<", "\\u003c").replace(">", "\\u003e").replace("&", "\\u0026")
    assets = {
        "/* INLINE_CSS */": (PAGE_DIR / "replay.css").read_text(encoding="utf-8"),
        "/* INLINE_JS */": (PAGE_DIR / "replay.js").read_text(encoding="utf-8"),
        "INLINE_JSON": data,
    }
    for marker, content in assets.items():
        equal(template.count(marker), 1, f"template placeholder {marker}")
        template = template.replace(marker, content)
    return template.encode("utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True, help="sealed healthy M1 smoke directory")
    parser.add_argument("--independent", type=Path, help="independent report directory (defaults to adjacent)")
    parser.add_argument("--output", type=Path, default=PAGE_DIR / "index.html", help="standalone replay page")
    parser.add_argument("--check", action="store_true", help="verify existing output byte for byte")
    args = parser.parse_args()
    directory = args.independent or args.bundle.parent / VERIFY_NAME
    try:
        projected = build_projection(Bundle(args.bundle), read_independent(directory))
        html = render_html(projected)
        if args.check:
            equal(args.output.read_bytes(), html, str(args.output))
        else:
            args.output.write_bytes(html)
    except (OSError, ValueError, TypeError, KeyError, IndexError) as error:
        parser.exit(1, f"M1 replay generation failed: {error}\n")
    print(f"PASS: 21 G acceptances, 2 regular handoffs, 21 independently verified proofs; "
          f"{args.output} sha256={sha256(html)}")


if __name__ == "__main__":
    main()
