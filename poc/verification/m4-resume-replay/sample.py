"""Deterministic contract sample. All records and IDs are synthetic."""

import hashlib

from model import SCHEMA, cut_anchor, derived_id


def symbol(label):
    return hashlib.sha256(("M4 contract sample / " + label).encode()).hexdigest()


def sample_tape():
    events = []
    profile = {
        "seed": 42, "tb": 10, "genesis_timestamp_us": 10_000_000, "slot_us": 1_000_000,
        "announcement_us": 10_000_000, "expiry_us": 30_000_000, "grace_us": 3_000_000,
        "sequencing_window": 4, "initial_duty": {
            "branch": 0, "r": 0, "region": 0, "tail": "NEXT", "cid": "样本 cid R0", "window": [1, 10],
        },
    }

    def add(tick, kind, stage=None, rule=None, **detail):
        refs = [{"label": "合同样本 · 合成事件与标识", "locator": "sample.py"}]
        if rule:
            refs.append({"label": "规则推导", "locator": rule})
        events.append(dict(tick_us=tick, kind=kind, stage=stage, detail=detail, refs=refs))

    genesis = {
        "height": 0, "timestamp_us": profile["genesis_timestamp_us"], "origin": 3, "kind": "genesis",
        "native": symbol("genesis"), "pre": symbol("pre genesis"), "post": symbol("post genesis"),
        "physical": symbol("genesis anchor"), "provenance": "genesis",
    }
    profile["genesis"] = genesis

    def block(height, parent, stream, kind, origin, physical_parent=None):
        record = {
            "height": height, "timestamp_us": 10_000_000 + height * 1_000_000,
            "native_parent": parent["native"], "pre": parent["post"], "post": symbol(f"{stream} post h{height}"),
            "native": symbol(f"{stream} native h{height}"), "kind": kind, "origin": origin,
            "provenance": "Regional" if kind == "regional" else "L1/DA",
        }
        if physical_parent:
            record["physical_parent"] = physical_parent
            record["physical"] = symbol(f"{stream} physical h{height}")
        return record

    def p(q, theta):
        return {"kind": "Espresso", "key_us": theta, "index": [q]}

    def regional(result, q, theta, cid, l1_height, stage=None, handoff=None):
        proof = symbol(f"proof q{q}")
        add(theta - 5_000, "regional_final", stage, cid=cid, block=result, proof=proof,
            signer_count=4, committee_size=5)
        add(theta + 1_000, "espresso", q=q, theta_us=theta, objects=[{
            "physical": result["physical"], "proof": proof, "body_index": 0, "proof_index": 1,
        }])
        add(theta + 1_000, "g_accepted", "accepted" if stage else None,
            rule="v10 GFR §4.2", cid=cid, block=result, proof=proof,
            position=p(q, theta), q_avail=q, coverage={"q": q, "l1_height": l1_height},
            **({"handoff": handoff} if handoff else {}))

    headers = [
        {"height": height, "timestamp_us": height * 2_000_000, "hash": symbol(f"L1 header {height}"),
         "parent": symbol(f"L1 header {height - 1}") if height else "0" * 64}
        for height in range(16)
    ]
    for header in headers:
        tick = header["timestamp_us"] + 1_500_000
        add(tick, "l1_finalized", **header)
        add(tick + 10_000, "l1_available", height=header["height"], hash=header["hash"], reader="A")

    old = [genesis]
    cid = profile["initial_duty"]["cid"]
    for height in range(1, 18):
        origin = max(old[-1]["origin"], (10 + height - 5) // 2)
        result = block(height, old[-1], "old G", "regional", origin, old[-1]["physical"])
        old.append(result)
        handoff = ({"branch": 0, "r": 1, "region": 1, "tail": "NEXT", "cid": "样本 cid R1",
                    "window": [11, 20], "anchor": result["physical"], "ref": genesis["physical"]} if height == 10 else None)
        regional(result, height - 1, result["timestamp_us"] + 40_000, cid, (height + 2) // 2, handoff=handoff)
        if handoff:
            add(result["timestamp_us"] + 48_000, "worker_lifecycle", kind_label="Closed", cid=cid,
                region=0, workers=[0, 1, 2, 3, 4], branch=0)
            cid = handoff["cid"]

    batch, commitment = "样本批次 h1", symbol("DA object h1")
    add(11_800_000, "batch_submitted", tx=batch, commitment=commitment)
    add(12_000_000, "batch_receipt", "batch", tx=batch, transaction_position=[6, 0],
        status=1, commitment=commitment, header=headers[6])
    add(13_510_000, "inbox_advanced", tx=batch, position=[6, 0, 0], receipt_log_index=0,
        previous=0, next=1, commitment=commitment, range=[1, 1], gpos=p(0, 11_040_000),
        channel="样本闭合 channel h1", frame_number=0, closed=True)
    add(13_510_000, "da_wait", "da-wait", "v10 GFR §5.4",
        position=[6, 0, 0], commitment=commitment, reason="已接纳，等待 DA 字节")
    add(13_700_000, "da_verified", position=[6, 0, 0], commitment=commitment, verified_commitment=commitment,
        object="样本 DA object h1", valid=True)
    d1 = {key: value for key, value in old[1].items() if key not in {"physical", "physical_parent"}}
    d1.update(kind="batch", provenance="L1/Inbox/DA", ordinary_count=1)
    add(13_700_000, "d_append", "derived", "v10 GFR §5.4", position=[6, 0, 0],
        reason="batch", blocks=[d1], da_commitment=commitment)
    add(13_710_000, "gprime_update", position=[6, 0, 0], source="batch", head=d1["native"],
        gpos=p(0, 11_040_000))

    late = "样本迟到批次 h2"
    add(15_880_000, "batch_submitted", tx=late, commitment=symbol("DA object h2"))
    add(16_000_000, "batch_receipt", "late-batch", tx=late, transaction_position=[8, 0], status=0,
        commitment=symbol("DA object h2"), header=headers[8], window_end=7,
        reason="样本的迟到批次未取得 Inbox 接纳")
    d = [genesis, d1]
    d2 = block(2, d1, "D", "window-fallback", 4)
    d2["ordinary_count"] = 0
    d.append(d2)
    add(15_510_000, "d_append", "fallback", "v10 GFR §5.4", input_point="BlockEnd ℓ7",
        reason="window-fallback", blocks=[d2], origin=3, window=4, processed_through=7, next_origin=4)
    add(15_510_000, "compare", "divergence", "v10 架构 §3.3", common_height=2,
        common_prefix=1, first_diff=2)

    d3 = block(3, d2, "D", "window-fallback", 5)
    d3["ordinary_count"] = 0
    d.append(d3)
    add(17_510_000, "d_append", reason="window-fallback", input_point="BlockEnd ℓ8", blocks=[d3],
        origin=4, window=4, processed_through=8, next_origin=5, rule="v10 GFR §5.4")

    resume_pos, cursor_pos = [9, 0, 1], [9, 0, 0]
    tx = "样本 RESUME 请求"
    add(17_000_000, "resume_submitted", tx=tx, target=20)
    add(18_000_000, "resume_included", "resume-included", "v10 GFR §4.6/§5.5",
        tx=tx, status=1, position=resume_pos, cursor_position=cursor_pos, receipt_log_index=1,
        cursor_rpc_log_index=0, header=headers[9], target=20, previous=1, control_seq=1,
        signed_origin=9, policy={"mask": 7, "order": [2, 0, 1]})
    for height in range(4, 21):
        next_origin = d[-1]["origin"] + 1
        origin = next_origin if next_origin <= 9 and 2 * next_origin <= 10 + height else d[-1]["origin"]
        result = block(height, d[-1], "D", "cursor-fill", origin)
        result["ordinary_count"] = 0
        d.append(result)
    anchor = cut_anchor(d[-1])
    cut_id = symbol("cut 9/0/1")
    add(19_510_000, "resume_cut", "cut", "v10 GFR §5.5",
        position=resume_pos, cursor_position=cursor_pos, cut_id=cut_id, target=20, fill=d[4:],
        prefix=[item["native"] for item in d], anchor=anchor)
    add(19_520_000, "gprime_update", "gprime-cut", "v10 Batch Inbox §3.7", position=cursor_pos,
        source="cut", resume_position=resume_pos, cut_id=cut_id, head=d[-1]["native"])
    add(27_751_000, "espresso", q=17, theta_us=27_750_000, objects=[])
    add(27_751_000, "g_empty", position=p(17, 27_750_000), coverage={"q":17, "l1_height":9})
    add(28_001_000, "espresso", q=18, theta_us=28_000_000, objects=[])
    position = {"kind": "Resume", "index": resume_pos, "key_us": 28_000_000}
    add(28_001_000, "g_wait", "resume-wait", "v10 GFR §4.6", position=position,
        missing="GFR reader A 等待校验并取得已保存的精确切面", reader="A")
    duty = {"branch": 1, "r": 0, "region": 2, "tail": "NEXT", "cid": "样本 cid R2", "window": [21, 30],
            "floor": anchor["origin"], "plan_height": anchor["origin"], "ref": anchor["id"], "source_region": None}
    add(28_002_000, "resume_applied", "resume", "v10 GFR §4.6",
        position=position, coverage={"q": 18, "l1_height": 9}, resume_position=resume_pos, cut_id=cut_id,
        before=[item["native"] for item in old], after=[item["native"] for item in d], retain_through=1,
        revoked=[item["native"] for item in old[2:]], adopted=[item["native"] for item in d[2:]],
        duty=duty, closed_branch=0, closed_cids=[profile["initial_duty"]["cid"], "样本 cid R1"],
        lifecycle=["BranchClosed", "Resumed", "SessionAssigned"], plan_header_hash=headers[anchor["origin"]]["hash"])
    add(28_002_000, "g_empty", position=p(18, 28_000_000), coverage={"q":18, "l1_height":9})
    add(28_005_000, "worker_lifecycle", kind_label="Closed", cid="样本 cid R1", region=1,
        workers=[5, 6, 7, 8, 9], branch=0)
    add(28_080_000, "worker_lifecycle", kind_label="Ready", cid=duty["cid"], region=2,
        workers=[10, 11, 12, 13], branch=1, anchor=anchor["id"])
    add(28_092_000, "worker_lifecycle", kind_label="Enter", cid=duty["cid"], region=2,
        workers=[10, 11, 12, 13], branch=1)
    new = block(21, d[-1], "new G", "regional", 10, derived_id(20, d[-1]["native"]))
    regional(new, 19, 31_040_000, duty["cid"], 11, "new-result")

    events.sort(key=lambda frame: frame["tick_us"])
    for seq, event in enumerate(events, 1):
        event["seq"] = seq
    return {"schema": SCHEMA, "profile": profile, "events": events}
