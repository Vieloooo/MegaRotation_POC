"""Verify canonical derived-cut bytes against an independently executed D prefix."""

import json

from derive_oracle import byte_list, l1_pos, metadata
from independent_reference import digest


def need(ok, detail):
    if not ok:
        raise ValueError(detail)


def hx(value):
    return bytes(value).hex()


def anchor(item):
    meta = metadata(item["metadata"])
    return {
        "anchor": {
            "block": {
                "height": meta["height"],
                "origin": meta["origin"],
                "block_id": byte_list(item["derived_anchor_id"]),
            },
            "timestamp_us": meta["timestamp_us"],
        },
        "state": {"metadata": meta, "accounts": item["accounts"]},
        "native_hash": byte_list(item["native_hash"]),
    }


def verify_item(item, expected):
    height = expected["metadata"][0]
    meta = metadata(expected["metadata"])
    physical, virtual, native = [item[name] for name in ("physical_block", "virtual_block", "native_block")]
    need(item["state"] == {"metadata": meta, "accounts": expected["accounts"]},
         f"independently executed D state differs at h{height}")
    need(hx(item["native_hash"]) == expected["native_hash"], f"D native hash differs at h{height}")
    need(native == {
        "parent": byte_list(expected["native_parent"]),
        "metadata": meta,
        "dep": [byte_list(raw) for raw in expected["dep"]],
        "kept": [byte_list(raw) for raw in expected["kept"]],
        "post_state_root": byte_list(expected["post_state_root"]),
    }, f"D native block bytes differ at h{height}")
    need(physical == {
        "parent": byte_list(expected["physical_parent"]),
        "metadata": meta,
        "pre_state_root": byte_list(expected["pre_state_root"]),
        "transactions": [byte_list(raw) for raw in expected["kept"]],
    }, f"D physical block differs at h{height}")
    need(virtual == {
        "metadata": meta,
        "dep": [byte_list(raw) for raw in expected["dep"]],
        "pre_state_root": byte_list(expected["pre_state_root"]),
        "transactions": [byte_list(raw) for raw in expected["kept"]],
    }, f"D virtual block differs at h{height}")
    actual_cause = item["cause"]
    cause = expected["cause"]
    match cause[0]:
        case "fallback":
            need(actual_cause["kind"] == "fallback" and actual_cause["window_end"] == cause[1]
                 and actual_cause["frontier"]["height"] == cause[2],
                 f"D fallback coverage differs at h{height}")
        case "cursor_set":
            need(actual_cause == {"kind": "cursor_set", "position": l1_pos(cause[1])},
                 f"D override provenance differs at h{height}")
        case "batch" | "substitution":
            need(actual_cause == {
                "kind": "accepted_batch", "position": l1_pos(cause[1]),
                "commitment": byte_list(cause[2]), "record_index": cause[3],
                "substituted": cause[0] == "substitution",
            }, f"D batch provenance differs at h{height}")


def check_positioned_cut(raw, cut, resume, prefix, genesis, config, bundle, driver_config, window):
    decoded = json.loads(raw)
    need(decoded == cut, "raw compact cut and decoded cut differ")
    need(json.dumps(decoded, separators=(",", ":"), ensure_ascii=False).encode() == raw,
         "cut evidence must retain compact deterministic JSON bytes")
    need(cut["resume_position"] == l1_pos(resume.position), "cut Resume position")
    need(cut["cursor_set_position"] == l1_pos(resume.set_position), "cut CursorSet position")
    need(cut["target_height"] == resume.target and cut["genesis"] == genesis, "cut target or genesis")
    expected_anchor = anchor(prefix[resume.target - genesis["anchor"]["block"]["height"] - 1])
    need(cut["target_anchor"] == expected_anchor, "cut anchor/state derived from wrong target")
    need(len(cut["derived_prefix"]) == len(prefix), "wrong D prefix length at paired CursorSet")
    for item, expected in zip(cut["derived_prefix"], prefix, strict=True):
        verify_item(item, expected)
    evidence = cut["evidence"]
    need(evidence["including_block"] == {
        "height": resume.position[0], "hash": byte_list(resume.block_hash),
    }, "cut including block differs from actual receipt")
    need(hx(evidence["transaction_hash"]) == resume.tx_hash[2:], "cut transaction hash")
    need(hx(evidence["request_id"]) == resume.rid[2:], "cut request id")
    need((evidence["sequence"], evidence["cursor_set_rpc_log_index"], evidence["resume_rpc_log_index"])
         == (resume.seq, resume.set_log_index, resume.resume_log_index), "cut sequence/log indices")
    identity = evidence["derivation"]
    need(hx(identity["config_ref"]) == config["ref"] and identity["sequence_window_blocks"] == window,
         "cut derivation config/window")
    l1 = identity["l1"]
    cfg = bundle["config"]
    need(l1["source"] == "driver" and l1["chain_id"] == cfg["chainId"]
         and hx(l1["config_ref"]) == config["ref"],
         "cut L1 public identity")
    for field in ("inbox", "authority", "submitter"):
        need(hx(l1[field]) == cfg[field][2:].lower(), f"cut L1 identity field {field}")
    need(hx(l1["l1_genesis_hash"]) == bundle["blocks"][0]["hash"][2:].lower()
         and l1["l1_genesis_timestamp_s"] == cfg["genesisTimestampSeconds"]
         and l1["slot_seconds"] == cfg["slotSeconds"]
         and l1["announcement_seconds"] == cfg["announcementSeconds"]
         and l1["finality_delay_us"] == driver_config["finality_delay_us"],
         "cut L1 clock/identity")
    return {
        "position": resume.position,
        "seq": resume.seq,
        "act_us": resume.act_us,
        "target": resume.target,
        "signed_floor": resume.signed_floor,
        "policy": resume.policy,
        "cut_digest": digest("PoC/Derivation/ResumeCutCheckpoint/v1", raw),
        "anchor": expected_anchor,
        "prefix": [
            {"height": row["metadata"][0], "origin": row["metadata"][2],
             "native": row["native_hash"], "state_root": row["post_state_root"],
             "kept": len(row["kept"])}
            for row in prefix
        ],
    }
