"""A deterministic contract-derived sample with synthetic diagnostic records."""

from model import assemble, event, historical_choice, seconds, signature_set, signer_view


PROFILE = {"seed": 42, "threshold": 2, "signer_count": 3, "stall_us": 2_000_000,
           "announcement_us": 10_000_000, "force_init_us": 3_000_000, "expiry_us": 30_000_000}
REFERENCE = "M3-auto-force-task-contract-zh.md · 主场景与时间预算"
DERIVED = "合同推导 · v10 GFR §4.4 与 Authority 历史 Plan"
SYNTHETIC = "合成样本 · 供交互和验证测试"


def tip(q, key):
    return {"q": q, "theta_us": key}


def poll(signer, previous, verified, advertised, eligible, reason):
    return {
        "signer": signer, "previous": previous, "espresso_verified": verified,
        "espresso_advertised": advertised, "l1_verified": 5, "l1_advertised": 5,
        "replay_position": {"kind": "Espresso", "index": {"Espresso": verified["q"]},
                            "key_us": verified["theta_us"]}, "head_height": 1,
        "clock_good": True, "clock_sample_us": 13_300_250, "clock_sample": {"offset_us": 0},
        "tail": "NEXT", "region": 0, "branch": 0, "cid": "示例 cid R0",
        "eligible": eligible, "reason": reason, "progress_us": 11_072_000,
        "observation_us": verified["theta_us"], "expiry_us": 42_000_000,
        "predicted_inclusion_us": 14_000_000,
        "policy": {"enabled_mask": 6, "order": [0, 1, 2], "signed_floor": 3},
    }


def sample_projection():
    events = []

    def add(lane, stage, label, tick, facts=(), ref=SYNTHETIC, **state):
        events.append(event(lane, stage, label, tick, len(events) + 1, facts, [ref], **state))

    add("regional", "old-result", "R0 physical h1", 11_067_000, [("类型", "区域 physical / BLS Final")])
    add("espresso", "old-public", "q5 publication", 11_073_000,
        [("公开键 θ", seconds(11_072_000)), ("消息", "正文 + 证明")], water="q5 · 11.072000 s")
    add("g", "old-accepted", "G 接纳 h1", 11_073_000, head=1, pc="Espresso q5 · 11.072000 s")
    add("governance", "outage", "R0 五个 worker 离线", 11_250_000,
        [("PID", [0, 1, 2, 3, 4]), ("G head", "h1")], REFERENCE)
    add("espresso", "heartbeat", "q6 空 heartbeat", 11_251_000,
        [("公开键 θ", seconds(11_250_000)), ("消息数", 0)], water="q6 · 11.250000 s")
    add("g", None, "处理 q6 · G 保持 h1", 11_251_000, pc="Espresso q6 · 11.250000 s")
    for i in range(7, 15):
        key = (i - 7) * 250_000 + 11_500_000
        add("espresso", None, f"q{i} 空 heartbeat", key + 1000,
            [("公开键 θ", seconds(key)), ("消息数", 0)], water=f"q{i} · {seconds(key)}")
        add("g", None, f"处理 q{i} · G 保持 h1", key + 1000, pc=f"Espresso q{i} · {seconds(key)}")
    verified = tip(14, 13_250_000)
    decisions = [
        poll(0, tip(13, 13_000_000), verified, verified, True, "ELIGIBLE"),
        poll(1, tip(13, 13_000_000), verified, verified, True, "ELIGIBLE"),
        poll(2, tip(12, 12_750_000), tip(13, 13_000_000), verified, False, "INPUTS_NOT_READY"),
    ]
    for detail in decisions:
        add("governance", "decision" if detail["signer"] == 1 else None,
            f"Signer {detail['signer']} · {detail['reason']}", 13_300_250,
            signer=signer_view(detail, PROFILE["stall_us"]))
    request = {"request_id": 1, "target_beta": 0, "enabled_mask": 6, "order": [0, 1, 2], "signed_floor": 3}
    sigs = [{"signer": i, "address": f"合同样本地址 {i}", "digest": "合同样本 digest", "request": request}
            for i in range(2)]
    for sig in sigs:
        add("governance", None, f"Signer {sig['signer']} 签署", 13_310_250, list(sig.items()), signedBy=sig["signer"])
    digest, req = signature_set(sigs, PROFILE["threshold"])
    add("governance", "signed", "治理阈值 2/3", 13_310_250, [("Digest", digest), ("请求", req)])
    add("l1", "submit", "提交 Force", 13_320_250, [("交易", "合同样本 · 未生成真实交易")])
    add("l1", "include", "ℓ7 收录", 14_000_000,
        [("L1 timestamp", seconds(14_000_000)), ("控制位置", "(7,0,0)")], REFERENCE)
    add("l1", "finalized", "ℓ7 finalized", 15_500_000, [("收录块", "ℓ7")])
    add("l1", "reader", "Reader A 收到 ℓ7", 15_500_000)
    add("l1", None, "Reader B 收到 ℓ7", 15_510_000)
    add("espresso", None, "q57 空 heartbeat", 24_001_000,
        [("公开键 θ", seconds(24_000_000)), ("消息数", 0)], water="q57 · 24.000000 s")
    selected, cycle, floor, history = historical_choice(request, 0, 3, 7, 3)
    add("g", "force", "Force 激活 · R0 → R1", 24_001_000,
        [("公开 act", seconds(24_000_000)), ("Anvil 公共覆盖", "ℓ7"),
         ("Espresso 水位", "q57 · θ 24.000000 s"), ("顺序", "同键 Force → SessionEnd → Espresso"),
         ("G", "保留 h1"), ("生命周期", "BranchClosed(β0) + SessionAssigned")], DERIVED,
        duty="R1 · β1/r1 [18,27]", pc="Force (7,0,0) · 24.000000 s",
        plan=[("来源", "R0"), ("policy mask", "0b110"), ("循环候选", cycle), ("有效 floor", floor),
              ("历史 L1 高度", history), ("选择", f"R{selected}"), ("anchor", "h1"), ("窗口", [18, 27])])
    add("g", None, "处理 q57 · G 保持 h1", 24_001_000, pc="Espresso q57 · 24.000000 s")
    add("regional", "new-result", "R1 physical h18", 28_035_000,
        [("物理父块", "h1"), ("区域 Final 阈值", "4/5")])
    add("espresso", None, "q74 publication", 28_041_000,
        [("公开键 θ", seconds(28_040_000)), ("消息", "正文 + 证明")], water="q74 · 28.040000 s")
    add("g", "accepted", "G 原子追加 h2–h18", 28_041_000,
        [("q_avail", 74), ("接纳公开键", seconds(28_040_000)), ("最早过期键", seconds(42_000_000)),
         ("物理父块", "h1"), ("执行前态", "padding h17 的执行后状态")],
        head=18, pc="Espresso q74 · 28.040000 s",
        appends=[{"height": h, "type": "padding" if h < 18 else "physical", "ref": SYNTHETIC}
                 for h in range(2, 19)])
    return assemble("sample", events, PROFILE, {"contract": REFERENCE, "evidence": SYNTHETIC})
