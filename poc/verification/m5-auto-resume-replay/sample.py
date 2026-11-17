"""Synthetic rule examples. No log carries an observed-run claim."""

from hashlib import sha256
from model import SCHEMA, m4, target


def symbol(name):
    return sha256(("M5 replay sample/" + name).encode()).hexdigest()


def build(mode, resume, include_height=13):
    s = 1_000_000
    late = include_height > 14
    act = include_height * 2 * s + 10 * s
    cut_tick = include_height * 2 * s + 1_501_000
    events = []
    genesis = {
        "height": 0, "timestamp_us": 10 * s, "origin": 3, "kind": "genesis",
        "native": symbol("genesis"), "pre": symbol("pre0"), "post": symbol("post0"),
        "physical": symbol("genesis anchor"), "provenance": "Genesis",
    }
    duty = {"branch": 0, "r": 0, "region": 0, "tail": "NEXT", "cid": "样本 R0", "window": [1, 10]}
    profile = {"seed": 42, "genesis_timestamp_us": 10 * s, "slot_us": s, "announcement_us": 10 * s,
               "expiry_us": 12 * s, "grace_us": 3 * s, "tb": 10, "sequencing_window": 4, "l1_slot_us": 2 * s,
               "genesis": genesis, "initial_duty": duty, "prep_us": 12 * s, "drift_us": 15 * s,
               "epsilon_us": s, "ready_us": 500_000, "clock_skew_us": 1000, "threshold": 2,
               "poll_us": 100_000, "sign_us": 10_000, "broadcast_us": 10_000,
               "members": ["0x" + str(i + 1).zfill(40) for i in range(3)]}

    def add(tick, kind, stage=None, rule=None, **detail):
        refs = [{"label": "合同样本 · 合成记录", "locator": "sample.py"}]
        if rule:
            refs.append({"label": "规则推导", "locator": rule})
        events.append({"tick_us": tick, "kind": kind, "stage": stage, "detail": detail, "refs": refs})

    headers = [{"height": h, "timestamp_us": 2 * h * s, "hash": symbol(f"header{h}"),
                "parent": symbol(f"header{h - 1}") if h else "0" * 64} for h in range(max(21, include_height + 5))]
    for header in headers:
        finalized = header["timestamp_us"] + 1_500_000
        if finalized > (54_100_000 if late else 41_100_000 if resume else 25 * s):
            break
        add(finalized, "l1_finalized", **header)
        add(finalized + 1000, "l1_available", height=header["height"], hash=header["hash"], reader="A")

    def pos(key, kind, *indices):
        return {"key_us": key, "kind": kind, "index": list(indices)}

    def block(h, parent, origin, kind, native_parent=None, physical_parent=None):
        label = f"{kind} h{h}"
        row = {"height": h, "timestamp_us": (10 + h) * s, "origin": origin, "kind": kind,
               "native": symbol(label), "pre": parent["post"], "post": symbol(label + " post"),
               "native_parent": native_parent or parent["native"],
               "provenance": "Derived" if physical_parent is None else "Regional"}
        if physical_parent:
            row.update(physical=symbol(label + " physical"), physical_parent=physical_parent, ordinary_count=1)
        return row

    def observation(q, theta):
        return {"q": q, "theta_us": theta}

    def espresso(q, theta, proposal, body=None, stage=None):
        proof = symbol(f"proof{q}") if body else None
        contents = ([{"physical": body["physical"], "proof": proof,
                      "body_index": 0, "proof_index": 1}] if body else [])
        add(proposal + 1000, "espresso", stage, q=q, theta_us=theta, proposal_tick_us=proposal,
            finalized_tick_us=proposal + 1000, objects=contents)
        return proof

    original = block(1, genesis, 3, "regional", physical_parent=genesis["physical"])
    proof = symbol("proof0")
    add(11_035_000, "regional_final", "old-final", cid=duty["cid"], block=original, proof=proof,
        signer_count=4, committee_size=5)
    first_key = 11_040_000
    espresso(0, first_key, first_key, original)
    first_pc = pos(first_key, "Espresso", 0)
    add(11_041_000, "g_accepted", cid=duty["cid"], block=original, proof=proof, q_avail=0,
        position=first_pc, coverage={"q": 0, "l1_height": 3})

    batch, commitment = "样本批次 h1", symbol("DA h1")
    add(11_800_000, "batch_submitted", tx=batch, commitment=commitment)
    add(12 * s, "batch_receipt", tx=batch, commitment=commitment, header=headers[6],
        transaction_position=[6, 0], status=1)
    add(13_501_000, "inbox_advanced", tx=batch, position=[6, 0, 0], previous=0, next=1,
        commitment=commitment, range=[1, 1], gpos=first_pc, channel="样本闭合批次", closed=True)
    add(13_501_000, "da_verified", position=[6, 0, 0], commitment=commitment,
        verified_commitment=commitment, object="样本 DA", valid=True)
    d1 = {**original, "kind": "batch", "provenance": "L1/Inbox/DA"}
    add(13_501_000, "d_append", position=[6, 0, 0], reason="batch", blocks=[d1], da_commitment=commitment)
    add(13_501_000, "gprime_update", position=[6, 0, 0], source="batch", head=d1["native"], reconstructed=True)
    d = [genesis, d1]
    for l1_height in range(7, max(13, include_height) if resume else 12):
        height = l1_height - 5
        tick = headers[l1_height]["timestamp_us"] + 1_501_000
        d.append(block(height, d[-1], d[-1]["origin"] + 1, "window-fallback"))
        add(tick, "d_append", "d-advance" if height == 2 else None, "v10 GFR §5.4",
            reason="window-fallback", blocks=[d[-1]], origin=d[-2]["origin"], window=4,
            processed_through=l1_height, next_origin=d[-1]["origin"])

    add(11_250_000, "ordering_fault", "fault", mode=mode, last=observation(0, first_key))
    pending = pos(24 * s, "Expire", 2)
    add(24 * s, "g_ordering_wait", "wait", "v10 GFR §3.4", position=pending, validated_q=4 if mode == "frozen" else 0,
        theta_us=first_key, missing="ESPRESSO_WATERMARK", reader="A")
    q = 0
    if mode == "frozen":
        for proposal in (11_500_000, 16 * s, 20 * s, 22 * s, 24 * s):
            q += 1
            espresso(q, first_key, proposal, stage="frozen-q" if q == 1 else None)
            add(proposal + 1000, "g_empty", position=pos(first_key, "Espresso", q),
                coverage={"q": q, "l1_height": 3})
    for worker, observed in enumerate((20_999_500, 21_000_000, 21_000_250, 21_000_500, 21_001_000)):
        add(observed, "local_stop", "local-stop" if worker == 0 else None, "v10 Session Program BFT-7",
            worker=worker, cid=duty["cid"], clock_us=21 * s, stop_us=21 * s,
            window_end=10, terminal=True, gate="deny_new_signatures")
    if not resume:
        poll = {"signer": 0, "heads": {"g": 1, "d": 6, "inbox": 1}, "tail": "NEXT",
                "pc": pos(first_key, "Espresso", q), "published": observation(q, first_key),
                "verified": observation(q, first_key), "previous": observation(q - 1, first_key),
                "published_l1": 11, "verified_l1": 11, "target": None, "reason": "WATERMARK_NOT_ADVANCING",
                "growth_basis": "样本收录前推进界限", "h_guard": 25,
                "guard_parts": {"b_hi":17,"E_hi":14,"h_time":18,"extra_origin":7,"H_D":25,"H_G":24,"H_Inbox":1},
                "signed_origin":13, "d_origin":8, "inbox_growth":0, "inclusion": [26 * s, 34 * s], "clock_ready": True, "paths_ready": True,
                "pending": False, "eligible": False}
        add(25 * s - 1, "signer_view", "target", **poll)
    else:
        add(24_500_000, "ordering_recovered", "recovery", mode=mode)
        q += 1
        espresso(q, 24_500_000, 24_500_000)
        add(24_501_000, "g_timer", "expire", "v10 GFR §4.5", position=pending, tail="HALT", session_end_key_us=24 * s,
            same_key_order=["Expire", "SessionEnd", "Espresso"],
            coverage={"q": q, "l1_height": 10})
        add(24_501_000, "g_empty", position=pos(24_500_000, "Espresso", q),
            coverage={"q": q, "l1_height": 10})
        q += 1
        seen_key = 24_750_000
        espresso(q, seen_key, seen_key)
        seen_pc = pos(seen_key, "Espresso", q)
        add(24_751_000, "g_empty", position=seen_pc, coverage={"q": q, "l1_height": 11})
        identity = symbol("request 1")
        solver = {"heads": {"g": 1, "d": 6, "inbox": 1}, "inbox_growth": 0, "tail":"HALT", "h_guard": 16,
                  "guard_parts": {"b_hi":14,"E_hi":11,"h_time":12,"extra_origin":4,"H_D":16,"H_G":1,"H_Inbox":1},
                  "d_origin":8, "signed_origin":13,
                  "growth_basis": "样本收录前推进界限", "inclusion": [26 * s, 28 * s], "target": 30}
        target(profile, solver)
        policy = {"mask": 7, "order": [2, 0, 1]}
        payload = {"branch":0, "target":30, "signed_origin":13, "control_seq":1, "policy":policy}
        for signer in range(3):
            add(24_790_000 + signer * 10_000, "signer_view",
                {0: "eligible", 1: "target"}.get(signer), "v2 §8.5–8.6",
                signer=signer, reason="HALTED_D_ADVANCED", request=identity, pc=seen_pc, payload=payload,
                published=observation(q, seen_key), verified=observation(q, seen_key),
                previous=observation(q - 1, 24_500_000), published_l1=11, verified_l1=11,
                clock_ready=True, paths_ready=True, d_drained=True, pending=False, eligible=True, **solver)
        add(24_810_000, "resume_signed", "signed", request=identity, target=30, t_eligible_us=24_800_000, inclusion=solver["inclusion"], payload=payload,
            members=[{"signer": i, "address": profile["members"][i], "signed_us": 24_810_000,
                      "digest": identity, "signature_ref": symbol(f"signed{i}")} for i in (0,1)])
        tx = "样本 RESUME 请求"
        add(24_820_000, "resume_prepared", "prepared", request=identity, tx=tx, target=30, durable=True)
        submit = 43_500_000 if late else 24_820_000
        if late:
            add(24_820_000, "resume_held", release_us=submit, tx=tx)
        add(submit, "resume_submitted", "submit", "M5 提交预算",
            request=identity, tx=tx, target=30, inclusion=solver["inclusion"],
            submit_delay_us=submit - 24_800_000, within_budget=not late)
        resume_pos, cursor_pos = [include_height, 0, 1], [include_height, 0, 0]
        add(include_height * 2 * s, "resume_included", "included", "v10 GFR §4.6", request=identity,
            tx=tx, status=1, position=resume_pos, cursor_position=cursor_pos, receipt_log_index=1,
            cursor_rpc_log_index=0, header=headers[include_height], target=30, previous=1, control_seq=1,
            signed_origin=13, policy=policy, act_us=act, within_inclusion=not late)
        fill = []
        for h in range(len(d), 31):
            origin = d[-1]["origin"]
            if origin < include_height and headers[origin + 1]["timestamp_us"] <= (10 + h) * s:
                origin += 1
            item = block(h, d[-1], origin, "cursor-fill")
            fill.append(item)
            d.append(item)
        anchor = m4.cut_anchor(d[-1])
        cut = symbol("cut 14 0 1")
        add(cut_tick, "resume_cut", "cut", "v10 GFR §5.5", position=resume_pos,
            cursor_position=cursor_pos, cut_id=cut, target=30, fill=fill,
            prefix=[item["native"] for item in d], anchor=anchor)
        add(cut_tick, "gprime_update", "gprime-cut", source="cut", position=cursor_pos,
            resume_position=resume_pos, cut_id=cut, head=anchor["native"], reconstructed=True)
        q += 1
        espresso(q, act, act)
        new_duty = {"branch": 1, "r": 0, "region": 2, "tail": "NEXT", "cid": "样本 R2", "window": [31, 40],
                    "floor": anchor["origin"], "plan_height": anchor["origin"], "ref": anchor["id"],
                    "source_region": None}
        if late:
            new_duty.update(tail="HALT", cid=None, region=None)
        add(act + 1000, "resume_applied", "resume", "v10 GFR §4.6",
            position=pos(act, "Resume", *resume_pos), coverage={"q": q, "l1_height": include_height},
            resume_position=resume_pos, cut_id=cut, before=[genesis["native"], original["native"]],
            after=[item["native"] for item in d], retain_through=1, revoked=[],
            adopted=[item["native"] for item in d[2:]], duty=new_duty, closed_branch=0,
            closed_cids=[duty["cid"]], lifecycle=["BranchClosed", "Resumed", "SessionClosed" if late else "SessionAssigned"],
            plan_header_hash=headers[anchor["origin"]]["hash"])
        add(act + 1000, "g_empty", position=pos(act, "Espresso", q),
            coverage={"q": q, "l1_height": include_height})
        if not late:
            add(act + 2000, "worker_lifecycle", "ready", kind_label="Ready", cid=new_duty["cid"], branch=1,
                region=2, workers=[10, 11, 12, 13])
            add(act + 5000, "worker_lifecycle", "enter", kind_label="Enter", cid=new_duty["cid"], branch=1,
                region=2, workers=[10, 11, 12, 13])
            q += 1
            next_block = block(31, d[-1], anchor["origin"] + 1, "regional", physical_parent=anchor["id"])
            proof = symbol(f"proof{q}")
            add(41_035_000, "regional_final", "new-result", cid=new_duty["cid"], block=next_block, proof=proof,
                signer_count=4, committee_size=5)
            espresso(q, 41_040_000, 41_040_000, next_block, stage="publish")
            add(41_041_000, "g_accepted", "accepted", cid=new_duty["cid"], block=next_block, proof=proof,
                q_avail=q, position=pos(41_040_000, "Espresso", q), coverage={"q": q, "l1_height": 18})

    events.sort(key=lambda event: event["tick_us"])
    for seq, event in enumerate(events, 1):
        event["seq"] = seq
    return {"schema": SCHEMA, "label": ("AR01 · finalized 停止后恢复" if mode == "stopped" else
                         "AR03 · q 继续增长、θ 解冻后恢复" if resume else "对照 · 水位仍冻结与无整数解"),
            "profile": profile, "events": events, "expect_resume": resume and not late}


def scenarios():
    return [build("stopped", True), build("frozen", True), build("frozen", False)]
